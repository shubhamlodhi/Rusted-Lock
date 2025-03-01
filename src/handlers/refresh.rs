// src/handlers/refresh_token.rs

    use axum::{
        extract::State,
        http::StatusCode,
        response::IntoResponse,
        Json,
    };
    use axum_extra::headers::{Authorization, authorization::Bearer};
    use diesel::prelude::*;
    use crate::db::PgPool;
    use crate::utils::jwt::generate_jwt;
    use crate::models::User;
    use serde_json::json;
    use std::env;
    use crate::schema::users;
    use crate::utils::jwt_validator::validate_jwt;

    pub async fn refresh_token(
        State(pool): State<PgPool>,
        Authorization(bearer): Authorization<Bearer>,
    ) -> impl IntoResponse {
        let conn = &mut pool.get().expect("Failed to get DB connection");

        match validate_jwt(bearer.token()).await {
            Ok(token_data) => {
                // Check if the token is a refresh token
                if token_data.claims.refresh {
                    // Find the user by username
                    match users::table
                        .filter(users::username.eq(&token_data.claims.sub))
                        .first::<User>(conn)
                    {
                        Ok(user) => {
                            // Generate new access token
                            match generate_jwt(user.username.clone(), &env::var("JWT_SECRET").unwrap(), false) {
                                Ok(new_access_token) => {
                                    let response = json!({
                                        "message": "Token refreshed successfully",
                                        "token": new_access_token,
                                    });
                                    (StatusCode::OK, Json(response)).into_response()
                                },
                                Err(_) => (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "Failed to generate new token".to_string()
                                ).into_response()
                            }
                        },
                        Err(_) => (
                            StatusCode::UNAUTHORIZED,
                            "User not found".to_string()
                        ).into_response()
                    }
                } else {
                    (StatusCode::UNAUTHORIZED, "Invalid refresh token".to_string()).into_response()
                }
            }
            Err(err) => (StatusCode::UNAUTHORIZED, err.to_string()).into_response(),
        }
    }