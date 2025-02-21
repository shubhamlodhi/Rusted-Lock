// src/utils/jwt.rs

use std::env;
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use crate::schema::users::dsl::{users, username, login_attempts, last_login_at};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
    pub refresh: bool,
}

pub fn generate_jwt(user_name: String, secret: &str, refresh: bool) -> Result<String, Box<dyn std::error::Error>> {
    let duration = if refresh {
        env::var("REFRESH_TOKEN_EXP_DURATION")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<i64>()
            .unwrap_or(60)
    } else {
        env::var("ACCESS_TOKEN_EXP_DURATION")
            .unwrap_or_else(|_| "15".to_string())
            .parse::<i64>()
            .unwrap_or(15)
    };

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(duration))
        .ok_or("Invalid timestamp calculation")?
        .timestamp();

    let claims = Claims {
        sub: user_name.to_owned(),
        exp: expiration as usize,
        refresh,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref())
    ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}




pub fn update_login_attempts(conn: &mut PgConnection, user_name: &str, attempts_count: i16) {
    diesel::update(users.filter(username.eq(user_name)))
        .set((
            login_attempts.eq(attempts_count),
            last_login_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)
        .expect("Error incrementing login attempts");
}