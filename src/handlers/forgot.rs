use std::env;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    body::Body
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use diesel::prelude::*;
use chrono::{NaiveDateTime, Utc, Duration};
use crate::db::PgPool;
use crate::schema::users::dsl::*;
use crate::schema::sessions::dsl::sessions;
use crate::utils::email::send_password_reset_email;
use crate::utils::jwt::generate_jwt;
use crate::models::User;
use crate::utils::error::AppError;

#[derive(Deserialize, Serialize, Debug)]
pub struct ForgotPasswordRequest {
    email: String,
}

pub async fn forgot_password(
    State(pool): State<PgPool>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<(StatusCode, &'static str), AppError> {
    let mut conn = pool.get().map_err(|e| AppError::InternalServerError(format!("Database connection error: {}", e)))?;

    let user = users
        .filter(email.eq(req.email))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|e| AppError::DbError(e))?;

    if let Some(user) = user {
        // Generate a password reset token
        let reset_token = generate_jwt(
            user.username.clone(),
            &env::var("JWT_SECRET").map_err(|e| AppError::ConfigError(e.to_string()))?,
            false
        ).map_err(|e| AppError::InternalServerError(format!("Failed to generate reset token: {}", e)))?;

        let token_expiration = Utc::now().naive_utc() + Duration::hours(1);

        // Store the reset token in sessions table
        let new_session = crate::models::NewSession {
            user_id: user.id,
            token: reset_token.clone(),
            refresh_token: String::new(), // Not needed for password reset
            expires_at: token_expiration,
        };

        diesel::insert_into(sessions)
            .values(&new_session)
            .execute(&mut conn)
            .map_err(|e| AppError::DbError(e))?;

        // Send password reset email
        send_password_reset_email(&user.email, &reset_token, token_expiration)
            .await
            .map_err(|e| AppError::EmailError(format!("Failed to send reset email: {}", e)))?;

        Ok((StatusCode::OK, "Password reset email sent"))
    } else {
        // Don't reveal if the user exists or not for security reasons
        Ok((StatusCode::OK, "If an account with that email exists, you will receive a password reset email"))
    }
}