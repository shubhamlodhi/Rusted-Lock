// src/handlers/login.rs

use std::env;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue, Response};
use axum::middleware::map_response;
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
) -> Response<Body> {
    let mut conn = pool.get().expect("Failed to get DB connection from pool");

    match find_user(&mut conn, &login_info.username) {
        Some(user) => handle_user_login(&mut conn, user, &login_info.password).await,
        None => (StatusCode::UNAUTHORIZED, "User not found".to_string()).into_response(),
    }
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

    let access_token = generate_jwt(user.username.clone(), env::var("JWT_SECRET").unwrap().as_str(), false);
    let refresh_token = generate_jwt(user.username.clone(), env::var("JWT_SECRET_X").unwrap().as_str(),true);


    let new_session = crate::models::NewSession {
        user_id: user.id,
        token: access_token.clone(),
        refresh_token: refresh_token.clone(),
        expires_at: Utc::now().naive_utc() + Duration::minutes(env::var("ACCESS_TOKEN_EXP_DURATION").unwrap().parse::<i64>().unwrap()),
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
    // Add Authorization header with Bearer token
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap()
    );
    // Add Set-Cookie header with refresh token
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "refresh_token={}; HttpOnly; Path=/; Max-Age={}",
            refresh_token,
            env::var("REFRESH_TOKEN_EXP_DURATION").unwrap().parse::<i64>().unwrap() * 60
        )).unwrap(),
    );

    (StatusCode::OK, headers,Json(login_resp)).into_response()
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