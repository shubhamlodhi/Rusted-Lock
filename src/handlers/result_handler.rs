use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use diesel::result::Error as DieselError;
use serde::Serialize;
use serde_json::json;

pub fn handle_db_result<T>(result: Result<T, DieselError>) -> Response 
where 
    T: Serialize,
{
    match result {
        Ok(data) => Json(data).into_response(),
        Err(err) => {
            let (status, error_message) = match err {
                DieselError::NotFound => (
                    StatusCode::NOT_FOUND,
                    "Record not found".to_string(),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                ),
            };

            (status, Json(json!({
                "error": error_message
            })))
            .into_response()
        }
    }
}
