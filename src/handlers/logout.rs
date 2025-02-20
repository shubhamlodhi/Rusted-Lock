use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use diesel::prelude::*;
use crate::schema::sessions::dsl::*;
use crate::db::PgPool;

pub async fn logout(
    State(pool): State<PgPool>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    let access_token = bearer.token();

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    // Delete the session from database to invalidate both tokens
    match diesel::delete(sessions.filter(token.eq(access_token)))
        .execute(&mut conn)
    {
        Ok(_) => {
            // Return Set-Cookie header to clear the refresh token cookie
            let cookie = "refresh_token=; HttpOnly; Path=/; Max-Age=0";
            (
                StatusCode::OK,
                [("Set-Cookie", cookie)],
                "Logged out successfully"
            ).into_response()
        },
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to logout"
        ).into_response(),
    }
}