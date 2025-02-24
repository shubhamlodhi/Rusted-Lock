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
pub struct GithubAuthUrl {
    url: String
}

#[derive(Deserialize)]
pub struct GithubAuthCode {
    code: String
}

fn create_oauth_client() -> BasicClient {
    let github_client_id = env::var("GITHUB_CLIENT_ID").expect("Missing GITHUB_CLIENT_ID");
    let github_client_secret = env::var("GITHUB_CLIENT_SECRET").expect("Missing GITHUB_CLIENT_SECRET");
    let redirect_url = env::var("GITHUB_REDIRECT_URL").expect("Missing GITHUB_REDIRECT_URL");

    BasicClient::new(
        ClientId::new(github_client_id),
        Some(ClientSecret::new(github_client_secret)),
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap())
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

pub async fn github_auth_url() -> impl IntoResponse {
    let client = create_oauth_client();
    
    let (auth_url, _csrf_token) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    Json(GithubAuthUrl {
        url: auth_url.to_string()
    })
}

use axum::extract::Query;

#[derive(Deserialize)]
pub struct GithubCallbackParams {
    code: String,
    state: Option<String>,
}

// Handler for query parameters (GET request)
pub async fn github_callback_params(
    State(pool): State<PgPool>,
    Query(params): Query<GithubCallbackParams>
) -> Response<Body> {
    handle_github_auth(pool, params.code).await
}

// Handler for JSON body (POST request)
pub async fn github_callback_json(
    State(pool): State<PgPool>,
    Json(payload): Json<GithubAuthCode>
) -> Response<Body> {
    handle_github_auth(pool, payload.code).await
}

// Common handler function
async fn handle_github_auth(pool: PgPool, code: String) -> Response<Body> {
    let client = create_oauth_client();
    
    // Move the existing callback logic here
    let token = match client
        .exchange_code(oauth2::AuthorizationCode::new(code))
        .add_extra_param("accept", "application/json")
        .request_async(async_http_client)
        .await {
            Ok(token) => token,
            Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Failed to exchange auth code: {}", e)}))).into_response()
        };

    // Get user info from GitHub
    let client = reqwest::Client::new();
    let user_info = match client
        .get("https://api.github.com/user")
        .header("User-Agent", "Rust-App")
        .bearer_auth(token.access_token().secret())
        .send()
        .await {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(info) => info,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to parse user info: {}", e)}))).into_response()
            },
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch user info: {}", e)}))).into_response()
        };

    // Get user's email from GitHub
    let emails_response = match client
        .get("https://api.github.com/user/emails")
        .header("User-Agent", "Rust-App")
        .bearer_auth(token.access_token().secret())
        .send()
        .await {
            Ok(response) => match response.json::<Vec<serde_json::Value>>().await {
                Ok(emails) => emails,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to parse email info: {}", e)}))).into_response()
            },
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch email info: {}", e)}))).into_response()
        };

    let primary_email = emails_response.iter()
        .find(|e| e.get("primary").and_then(|v| v.as_bool()).unwrap_or(false))
        .and_then(|e| e.get("email").and_then(|v| v.as_str()))
        .unwrap_or("");

    if primary_email.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "No primary email found"}))).into_response();
    }
    let github_login = user_info["login"].as_str().unwrap_or("");
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
        .filter(email.eq(primary_email))
        .first::<crate::models::User>(&mut conn)
        .optional() {
            Ok(Some(user)) => user,
            Ok(None) => {
                let new_user = NewUser {
                    email: primary_email.to_owned(),
                    username: github_login.to_owned(),  // Use GitHub login as username
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