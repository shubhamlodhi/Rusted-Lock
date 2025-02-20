use std::env;
use diesel::RunQueryDsl;
use diesel::prelude::*;
use chrono::{Utc, Duration};
use crate::models::Session;
use crate::schema::sessions::dsl::sessions;
use crate::schema::sessions::{expires_at, refresh_token, token};
use crate::utils::jwt::generate_jwt;
use crate::utils::jwt_validator::validate_refresh_token;

pub async fn refresh_tokens(refresh_token_str: &str, conn: &mut PgConnection) -> Result<(String, String), String> {
    // 1. Validate refresh token
    let token_data = validate_refresh_token(refresh_token_str).await?;

    // 2. Check if refresh token is in database
    let session = sessions
        .filter(refresh_token.eq(refresh_token_str))
        .first::<Session>(conn)
        .map_err(|_| "Invalid refresh token")?;

    // 3. Generate new tokens
    let username = token_data.claims.sub;
    let new_access_token = generate_jwt(
        username.clone(),
        env::var("JWT_SECRET").unwrap().as_str(),
        false
    );
    let new_refresh_token = generate_jwt(
        username,
        env::var("JWT_SECRET_X").unwrap().as_str(),
        true
    );

    let expiry = Utc::now().naive_utc() + Duration::minutes(
        env::var("ACCESS_TOKEN_EXP_DURATION")
            .unwrap()
            .parse::<i64>()
            .unwrap()
    );
    println!("Refresh Token is called : new {:?}, old {:?}", new_refresh_token, refresh_token_str);
    // 4. Update session with new tokens
    diesel::update(sessions.find(session.id))
        .set((
            token.eq(&new_access_token),
            refresh_token.eq(&new_refresh_token),
            expires_at.eq(expiry)
        ))
        .execute(conn)
        .map_err(|_| "Failed to update session")?;

    Ok((new_access_token, new_refresh_token))
}