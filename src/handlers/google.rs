use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::{response::IntoResponse, Json, extract::State};
use oauth2::{basic::BasicClient, AuthUrl, TokenUrl, ClientId, ClientSecret, RedirectUrl, Scope, TokenResponse};
use oauth2::reqwest::async_http_client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use crate::db::PgPool;
use crate::utils::jwt;

#[derive(Serialize)]
pub struct GoogleAuthUrl {
    url: String
}

#[derive(Deserialize)]
pub struct GoogleAuthCode {
    code: String
}

fn create_oauth_client() -> BasicClient {
    let google_client_id = env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID");
    let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").expect("Missing GOOGLE_CLIENT_SECRET");
    let redirect_url = env::var("GOOGLE_REDIRECT_URL").expect("Missing GOOGLE_REDIRECT_URL");

    BasicClient::new(
        ClientId::new(google_client_id),
        Some(ClientSecret::new(google_client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap())
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

pub async fn google_auth_url() -> impl IntoResponse {
    let client = create_oauth_client();
    
    let (auth_url, _csrf_token) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string()))
        .url();

    Json(GoogleAuthUrl {
        url: auth_url.to_string()
    })
}

pub async fn google_callback(
    State(pool): State<PgPool>,
    Json(payload): Json<GoogleAuthCode>
) -> Response<Body>{
    let client = create_oauth_client();
    
    // Exchange authorization code for token
    let token = match client
        .exchange_code(oauth2::AuthorizationCode::new(payload.code))
        .request_async(async_http_client)
        .await {
            Ok(token) => token,
            Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Failed to exchange auth code: {}", e)}))).into_response()
        };

    // Get user info from Google
    let client = reqwest::Client::new();
    let user_info = match client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(info) => info,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to parse user info: {}", e)}))).into_response()
            },
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch user info: {}", e)}))).into_response()
        };

    let email_str = match user_info["email"].as_str() {
        Some(email_str) if !email_str.is_empty() => email_str,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "No email provided by Google"}))).into_response()
    };
    let name_str = user_info["name"].as_str().unwrap_or("");

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)}))).into_response()
    };

    // Check if user exists or create new one
    use diesel::prelude::*;
    use crate::schema::users::dsl::*;
    use crate::models::NewUser;

    let user = match users
        .filter(email.eq(email_str))
        .first::<crate::models::User>(&mut conn)
        .optional() {
            Ok(Some(user)) => user,
            Ok(None) => {
                let new_user = NewUser {
                    email: email_str.to_owned(),
                    username: email_str.to_owned(),
                    password_hash: "".to_string(),
                    full_name: Some(name_str.to_string()),
                    role: "user".to_string(),
                    status: "active".to_string(),
                };

                match diesel::insert_into(users)
                    .values(&new_user)
                    .get_result(&mut conn) {
                        Ok(user) => user,
                        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to create user: {}", e)}))).into_response()
                }
            },
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error: {}", e)}))).into_response()
        };

    // Generate JWT tokens
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string());
    let jwt_secret_x = env::var("JWT_SECRET_X").unwrap_or_else(|_| "default_refresh_secret_key".to_string());
    
    let access_token = match jwt::generate_jwt(user.username.clone(), &jwt_secret, false) {
        Ok(token) => token,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to generate access token: {}", e)}))).into_response()
    };
    let refresh_token = match jwt::generate_jwt(user.username.clone(), &jwt_secret_x, true) {
        Ok(token) => token,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to generate refresh token: {}", e)}))).into_response()
    };

    // Create new session
    let new_session = crate::models::NewSession {
        user_id: user.id,
        token: access_token.clone(),
        refresh_token: refresh_token.clone(),
        expires_at: chrono::Utc::now().naive_utc() + chrono::Duration::minutes(
            env::var("ACCESS_TOKEN_EXP_DURATION")
                .unwrap_or_else(|_| "15".to_string())
                .parse::<i64>()
                .unwrap_or(15)
        ),
    };

    if let Err(e) = diesel::insert_into(crate::schema::sessions::dsl::sessions)
        .values(&new_session)
        .execute(&mut conn) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to save session: {}", e)}))).into_response();
    }

    // Return tokens in response
    let mut headers = axum::http::HeaderMap::new();
    match axum::http::HeaderValue::from_str(&format!("Bearer {}", access_token)) {
        Ok(auth_header) => { headers.insert(axum::http::header::AUTHORIZATION, auth_header); },
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to create auth header: {}", e)}))).into_response()
    }

    match axum::http::HeaderValue::from_str(&format!(
        "refresh_token={}; HttpOnly; Path=/; Max-Age={}",
        refresh_token,
        env::var("REFRESH_TOKEN_EXP_DURATION")
            .unwrap_or_else(|_| "240".to_string())
            .parse::<i64>()
            .unwrap_or(240) * 60
    )) {
        Ok(cookie_header) => { headers.insert(axum::http::header::SET_COOKIE, cookie_header); },
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to create cookie header: {}", e)}))).into_response()
    }

    (StatusCode::OK, headers, Json(json!({
        "message": "Login successful",
        "token": access_token,
        "user": {
            "email": user.email,
            "username": user.username,
            "full_name": user.full_name
        }
    }))).into_response()
}