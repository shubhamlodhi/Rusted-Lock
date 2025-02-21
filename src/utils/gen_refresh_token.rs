use std::env;
use std::error::Error;
use diesel::RunQueryDsl;
use diesel::prelude::*;
use chrono::{Utc, Duration};
use crate::models::Session;
use crate::schema::sessions::dsl::*;
use crate::schema::sessions::{expires_at, refresh_token, token};
use crate::utils::jwt::generate_jwt;
use crate::utils::jwt_validator::validate_refresh_token;

pub async fn refresh_tokens(refresh_token_str: &str, conn: &mut PgConnection) -> Result<(String, String), String> {
    // 1. Validate refresh token
    let token_data = validate_refresh_token(refresh_token_str).await
        .map_err(|e| format!("Invalid refresh token: {}", e))?;

    // 2. Check if refresh token is in database and not expired
    let session = sessions
        .filter(refresh_token.eq(refresh_token_str))
        .first::<Session>(conn)
        .map_err(|_| "Refresh token not found in database")?;

    if token_data.claims.exp < Utc::now().timestamp() as usize {
        return Err("Refresh token has expired".to_string());
    }

    // 3. Generate new tokens
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string());
    let jwt_secret_x = env::var("JWT_SECRET_X").unwrap_or_else(|_| "default_refresh_secret_key".to_string());

    let new_access_token = generate_jwt(session.user_id.to_string(), &jwt_secret, false)
        .map_err(|e| format!("Failed to generate access token: {}", e))?;
    let new_refresh_token = generate_jwt(session.user_id.to_string(), &jwt_secret_x, true)
        .map_err(|e| format!("Failed to generate refresh token: {}", e))?;

    diesel::update(sessions)
        .filter(id.eq(session.id))
        .set((
            token.eq(&new_access_token),
            refresh_token.eq(&new_refresh_token),
            expires_at.eq(Utc::now().naive_utc() + Duration::minutes(env::var("ACCESS_TOKEN_EXP_DURATION").unwrap().parse::<i64>().unwrap()))
        ))
        .execute(conn)
        .map_err(|e| format!("Failed to update session: {}", e))?;

    Ok((new_access_token, new_refresh_token))
}