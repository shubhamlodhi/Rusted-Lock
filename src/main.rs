// src/main.rs

mod db;
mod models;
mod schema; // This file is generated by Diesel.
mod handlers;
use axum::{
    response::IntoResponse,
    http::StatusCode,
    routing::{get, post, put, delete},
    Router,
};
use crate::handlers::user_handlers::{list_users, create_user, get_user_by_id, update_user, delete_user};
use db::establish_connection_pool;
use std::net::SocketAddr;
use tokio::net::TcpListener;



// Root handler.
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

#[tokio::main]
async fn main() {
    // Initialize the database connection pool
    let pool = establish_connection_pool();

    // Create router with routes
    let app = Router::new()
        .route("/",get(root))
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user_by_id))
        .route("/users/{id}", put(update_user))
        .route("/users/{id}", delete(delete_user))
        .with_state(pool);

    // Set up the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    // Create a TCP listener
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);

    // Start serving
    axum::serve(listener, app).await.unwrap();
}
