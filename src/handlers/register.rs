// src/handlers/register.rs

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use bcrypt::hash;
use diesel::prelude::*;
use crate::models::NewUser;
use crate::schema::users::dsl::users;
use crate::db::PgPool;

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct RegisterRequest {
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    pub email: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        if self.email.is_empty() || !self.email.contains('@') {
            return Err("Invalid email address".to_string());
        }
        Ok(())
    }
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(register_info): Json<RegisterRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = register_info.validate() {
        return (StatusCode::BAD_REQUEST, format!("Validation errors: {:?}", validation_errors));
    }

    let password_hash = match hash(&register_info.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()),
    };

    let new_user = NewUser {
        email: register_info.email,
        username: register_info.username,
        password_hash,
        full_name: None,
        role: "user".to_string(),
        status: "active".to_string(),
    };

    let mut conn = pool.get().expect("Failed to get DB connection from pool");
    match diesel::insert_into(users).values(&new_user).execute(&mut conn) {
        Ok(_) => (StatusCode::CREATED, "User registered successfully".to_string()),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to register user".to_string()),
    }
}