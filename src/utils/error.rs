// src/utils/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    DbError(diesel::result::Error),
    EmailError(String),
    ConfigError(String),
    ValidationError(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DbError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::EmailError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::ConfigError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::ValidationError(e) => (StatusCode::BAD_REQUEST, e),
            AppError::InternalServerError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

// Implement `From` traits for easy error conversion
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::DbError(err)
    }
}

impl From<lettre::transport::smtp::Error> for AppError {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        AppError::EmailError(err.to_string())
    }
}

impl From<lettre::address::AddressError> for AppError {
    fn from(err: lettre::address::AddressError) -> Self {
        AppError::EmailError(err.to_string())
    }
}

impl From<lettre::error::Error> for AppError {
    fn from(err: lettre::error::Error) -> Self {
        AppError::EmailError(err.to_string())
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        AppError::ConfigError(err.to_string())
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DbError(e) => write!(f, "Database error: {}", e),
            AppError::EmailError(e) => write!(f, "Email error: {}", e),
            AppError::ConfigError(e) => write!(f, "Configuration error: {}", e),
            AppError::ValidationError(e) => write!(f, "Validation error: {}", e),
            AppError::InternalServerError(e) => write!(f, "Internal server error: {}", e),
        }
    }
}
// ... add other `From` implementations as needed