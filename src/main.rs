// src/main.rs
mod db;
mod models;
mod schema;
mod handlers;
mod utils;
mod middleware;
mod server;
mod config;
mod routes;

use db::establish_connection_pool;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();
    config::load_env();

    // Initialize the database connection pool
    let pool = establish_connection_pool();

    // Create router with routes and await it immediately
    let app = routes::create_routes(pool).await;
    // Run the server
    server::run_server(app).await;
}