// src/handlers/login.rs

use std::env;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue, Response};

use serde::{Deserialize, Serialize};
use bcrypt::verify;
use diesel::prelude::*;
use chrono::{Utc, Duration};
use crate::models::User;
use crate::schema::users::dsl::{users, username};
use crate::db::PgPool;
use crate::schema::sessions::dsl::sessions;
use crate::utils::jwt::{generate_jwt, update_login_attempts};

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(login_info): Json<LoginRequest>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let mut conn = pool.get().map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error".to_string())
    })?;

    let login_attempts_count = match find_user(&mut conn, &login_info.username) {
        Some(user) => {
            let now = Utc::now().naive_utc();
            if is_account_locked(&user, now) {
                return Ok((StatusCode::FORBIDDEN, "Account locked. Try again later.".to_string()).into_response());
            }

            if verify(login_info.password, &user.password_hash).unwrap_or(false) {
                Ok(successful_login(&mut conn, &user).await)
            } else {
                let attempts = user.login_attempts + 1;
                Ok(failed_login(&mut conn, &user, attempts).await)
            }
        },
        None => Ok((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()).into_response()),
    }?;

    Ok(login_attempts_count)
}

fn find_user(conn: &mut PgConnection, user_name: &str) -> Option<User> {
    users
        .filter(username.eq(user_name))
        .first::<User>(conn)
        .optional()
        .expect("Error loading user")
}

async fn handle_user_login(
    conn: &mut PgConnection,
    user: User,
    password: &str,
) -> Response<Body> {
    let now = Utc::now().naive_utc();
    let mut login_attempts_count = user.login_attempts + 1;

    if is_account_locked(&user, now) {
        return (StatusCode::FORBIDDEN, "Account locked. Try again later.".to_string()).into_response();
    }

    if verify(password, &user.password_hash).unwrap() {
        successful_login(conn, &user).await
    } else {
        failed_login(conn, &user, login_attempts_count).await
    }
}

fn is_account_locked(user: &User, now: chrono::NaiveDateTime) -> bool {
    user.login_attempts > 3 && user.last_login_at.map_or(false, |last_attempt| {
        now.signed_duration_since(last_attempt) < Duration::minutes(env::var("ACCOUNT_LOCK_DURATION").unwrap().parse::<i64>().unwrap())
    })
}

async fn successful_login(conn: &mut PgConnection, user: &User) -> Response<Body> {
    update_login_attempts(conn, &user.username, 0);

    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string());
    let jwt_secret_x = env::var("JWT_SECRET_X").unwrap_or_else(|_| "default_refresh_secret_key".to_string());
    
    let access_token = match generate_jwt(user.username.clone(), &jwt_secret, false) {
        Ok(token) => token,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate access token".to_string()).into_response()
    };
    let refresh_token = match generate_jwt(user.username.clone(), &jwt_secret_x, true) {
        Ok(token) => token,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate refresh token".to_string()).into_response()
    };

    let new_session = crate::models::NewSession {
        user_id: user.id,
        token: access_token.clone(),
        refresh_token: refresh_token.clone(),
        expires_at: Utc::now().naive_utc() + Duration::minutes(
            env::var("ACCESS_TOKEN_EXP_DURATION")
                .unwrap_or_else(|_| "15".to_string())
                .parse::<i64>()
                .unwrap_or(15)
        ),
    };

    diesel::insert_into(sessions)
        .values(&new_session)
        .execute(conn)
        .expect("Error saving session");

    let login_resp = serde_json::json!({
        "message": "Login successful",
        "token": access_token,
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap()
    );
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "refresh_token={}; HttpOnly; Path=/; Max-Age={}",
            refresh_token,
            env::var("REFRESH_TOKEN_EXP_DURATION")
                .unwrap_or_else(|_| "240".to_string())
                .parse::<i64>()
                .unwrap_or(240) * 60
        )).unwrap(),
    );

    (StatusCode::OK, headers, Json(login_resp)).into_response()
}

async fn failed_login(
    conn: &mut PgConnection,
    user: &User,
    login_attempts_count: i16
) -> Response<Body> {
    update_login_attempts(conn, &user.username, login_attempts_count);

    if login_attempts_count >= 3 {
        (StatusCode::FORBIDDEN, "Account locked. Too many failed attempts.".to_string()).into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid password".to_string()).into_response()
    }
}