use axum::{
    extract::{State},
    http::{StatusCode,Request},
    response::IntoResponse,
    middleware::Next,
    body::Body,
};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;




use crate::utils::jwt_validator::validate_jwt;

pub async fn auth_middleware<T>(
    State(secret): State<String>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    match validate_jwt(bearer.token(), &secret).await {
        Ok(_) => next.run(req).await,
        Err(err) => (StatusCode::UNAUTHORIZED, err).into_response(),
    }
}