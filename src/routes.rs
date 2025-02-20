// src/routes.rs

use axum::{
    extract::State,
    routing::{get, post},
    Router,
    body::Body,
    middleware::from_fn_with_state,
};
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::response::IntoResponse;
use crate::handlers;
use crate::middleware::token_validator::auth_middleware;
use crate::db::PgPool;

pub async fn root() -> impl IntoResponse {
    let welcome_message = r#"

    ╭──────────────────────────────────────────────────────────────╮
    │                                                              │
    │   ╦═╗╦ ╦╔═╗╔╦╗  ╔═╗╔═╗╦    ╔═╗╔═╗╦═╗╦  ╦╦╔═╗╔═╗              │
    │   ╠╦╝║ ║╚═╗ ║   ╠═╣╠═╝║    ╚═╗║╣ ╠╦╝╚╗╔╝║║  ║╣               │
    │   ╩╚═╚═╝╚═╝ ╩   ╩ ╩╩  ╩    ╚═╝╚═╝╩╚═ ╚╝ ╩╚═╝╚═╝              │
    │                                                              │
    │              Welcome to Rust API Service v1.0                │
    │                                                              │
    ├──────────────────────────────────────────────────────────────┤
    │                                                              │
    │  🚀 API ROUTES:                                              │
    │                                                              │
    │  📌 USERS                                                    │
    │     GET    /users          List all users                    │
    │     POST   /users          Create new user                   │
    │     GET    /users/{id}     Get user details                  │
    │     PUT    /users/{id}     Update user                       │
    │     DELETE /users/{id}     Delete user                       │
    │                                                              │
    │                                                              │
    ├──────────────────────────────────────────────────────────────┤
    │                                                              │
    │  🔧 Made with Rust, Axum, and PostgreSQL                     |
    │                                                              │
    │                                                              │
    ╰──────────────────────────────────────────────────────────────╯

    "#;

    (StatusCode::OK, welcome_message)
}



pub async fn protected_root() -> impl IntoResponse {
    let protected_message = r#"I am Protected Here"#;
    (StatusCode::OK, protected_message)
}



pub async fn create_routes(pool: PgPool) -> Router {

    let public_routes = Router::new()
        .route("/", get(root));

    let login_routes = Router::new()
        .route("/login", post(handlers::login::login))
        .route("/register", post(handlers::register::register));

    let protected_routes = Router::new()
        .route("/logout", post(handlers::logout::logout))
        .route("/protected", get(protected_root))
        .layer(from_fn_with_state(pool.clone(), auth_middleware::<Body>));

    Router::new()
        .nest("/api", public_routes)
        .nest("/api", login_routes)
        .nest("/api", protected_routes)
        .with_state(pool)
}