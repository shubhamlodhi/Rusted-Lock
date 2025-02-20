// src/utils/jwt_validator.rs

use std::env;
use axum::extract::State;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use jsonwebtoken::{decode, DecodingKey, Validation, TokenData};
use serde::{Serialize, Deserialize};
use chrono::Utc;
use crate::db::PgPool;
use crate::schema::sessions::dsl::sessions;
use crate::schema::sessions::token;
use diesel::prelude::*;
use jsonwebtoken::errors::ErrorKind;
use crate::models::Session;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub(crate) sub: String,
    exp: usize,
    pub refresh: bool,
}

fn is_token_expired(exp: usize) -> bool {
    exp < Utc::now().timestamp() as usize
}

pub async fn validate_jwt(token_y: &str) -> Result<TokenData<Claims>, String> {
    // First validate JWT signature and expiration
    let validation = Validation::default();
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token_data = match decode::<Claims>(
        token_y,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation
    ) {
        Ok(token_data) => token_data,
        Err(err) => match *err.kind() {
            ErrorKind::ExpiredSignature => return Err("Token has expired".to_string()),
            ErrorKind::InvalidSignature => return Err("Invalid token signature".to_string()),
            _ => return Err("Invalid token format".to_string()),
        },
    };

    // Then check if token exists in database
    let mut conn = match PgConnection::establish(&env::var("DATABASE_URL").expect("DATABASE_URL must be set")) {
        Ok(conn) => conn,
        Err(_) => return Err("Database error".to_string()),
    };

    match sessions
        .filter(token.eq(token_y))
        .first::<Session>(&mut conn)
    {
        Ok(_) => Ok(token_data),
        Err(_) => Err("Token not found in database".to_string()),
    }
}

pub async fn invalidate_token(
    State(pool): State<PgPool>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<(), String> {
    let token_str = bearer.token();
    let mut conn = pool.get().map_err(|_| "Failed to get DB connection from pool".to_string())?;
    diesel::delete(sessions.filter(token.eq(token_str)))
        .execute(&mut conn)
        .map_err(|_| "Failed to invalidate token".to_string())?;
    Ok(())
}


pub async fn validate_refresh_token(refresh_token: &str) -> Result<TokenData<Claims>, String> {
    let validation = Validation::default();
    let jwt_secret = env::var("JWT_SECRET_X").expect("JWT_SECRET_X must be set");
    match decode::<Claims>(refresh_token, &DecodingKey::from_secret(jwt_secret.as_ref()), &validation) {
        Ok(token_data) => Ok(token_data),
        Err(err) => match *err.kind() {
            ErrorKind::ExpiredSignature => Err("Refresh Token has expired".to_string()),
            ErrorKind::InvalidSignature => Err("Invalid Refresh token signature".to_string()),
            _ => Err("Invalid Refresh token format".to_string()),
        },
    }
}