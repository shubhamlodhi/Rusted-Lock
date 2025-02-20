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

pub fn generate_jwt(user_name: String, secret: &str, refresh: bool) -> String {

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(
            if refresh {
                env::var("REFRESH_TOKEN_EXP_DURATION")
                    .unwrap()
                    .parse::<i64>()
                    .unwrap()
            } else {
                env::var("ACCESS_TOKEN_EXP_DURATION")
                    .unwrap()
                    .parse::<i64>()
                    .unwrap()
            }
        ))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_name.to_owned(),
        exp: expiration as usize,
        refresh,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("JWT token generation failed").to_string()
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