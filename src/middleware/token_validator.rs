use std::env;
use axum::{
    extract::{State},
    http::{StatusCode,Request},
    response::IntoResponse,
    middleware::Next,
    body::Body,
};
use axum::http::{header, HeaderMap, HeaderValue};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, Cookie};
use axum_extra::headers::authorization::Bearer;
use diesel::prelude::*;
use crate::db::PgPool;
use crate::models::Session;
use crate::schema::sessions::dsl::sessions;
use crate::schema::sessions::{refresh_token, token};
use crate::utils::jwt_validator::validate_jwt;
use crate::utils::gen_refresh_token::refresh_tokens;

pub async fn auth_middleware<T>(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    cookie: Option<TypedHeader<Cookie>>,
    State(pool): State<PgPool>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let access_token = bearer.token();
    println!("Access token is here {:?}", access_token);
    match validate_jwt(access_token).await {
        Ok(_) => next.run(req).await,
        Err(err) if err == "Token has expired" => {
            let mut conn = match pool.get() {
                Ok(conn) => conn,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            };

            // 1. Get refresh token from cookie
            let refresh_token_str = match cookie.and_then(|c| c.get("refresh_token").map(|s| s.to_string())) {
                Some(rt) => rt,
                None => return (StatusCode::UNAUTHORIZED, "No refresh token").into_response(),
            };

            // 2. Verify session exists with this access token and refresh token pair
            let session = match sessions
                .filter(token.eq(access_token))
                .filter(refresh_token.eq(&refresh_token_str))
                .first::<Session>(&mut conn) {
                Ok(s) => s,
                Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid session").into_response(),
            };
            println!("Refresh tokens");
            // 3. Try to refresh tokens
            match refresh_tokens(&refresh_token_str, &mut conn).await {
                Ok((new_access_token, new_refresh_token)) => {
                    println!("Refresh tokens are here {:?},  {:?}", new_access_token, new_refresh_token);
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        header::AUTHORIZATION,
                        HeaderValue::from_str(&format!("{}", new_access_token)).unwrap()
                    );
                    headers.insert(
                        header::SET_COOKIE,
                        HeaderValue::from_str(&format!(
                            "refresh_token={}; HttpOnly; Path=/; Max-Age={}",
                            new_refresh_token,
                            env::var("REFRESH_TOKEN_EXP_DURATION").unwrap().parse::<i64>().unwrap() * 60
                        )).unwrap(),
                    );

                    // Create new request with new access token
                    let mut new_req = req;
                    new_req.headers_mut().insert(
                        header::AUTHORIZATION,
                        HeaderValue::from_str(&format!("Bearer {}", new_access_token)).unwrap()
                    );

                    let mut response = next.run(new_req).await;
                    response.headers_mut().extend(headers);
                    response
                },
                Err(e) => {println!("This Error");(StatusCode::UNAUTHORIZED, e).into_response()},

            }
        },
        Err(err) => (StatusCode::UNAUTHORIZED, err).into_response(),
    }
}