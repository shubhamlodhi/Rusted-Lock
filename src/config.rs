// src/config.rs

use std::env;

// Load the appropriate .env.development file based on the environment
fn load_dotenv_file(environment: &str) {
    let filename = match environment {
        "production" => ".env.production",
        "test" => ".env.test",
        _ => ".env.development",
    };
    dotenvy::from_filename(filename).ok();
}

// Load environment variables
pub fn load_env() {
    // Determine the environment
    let environment = env::var("RUST_ENV").expect("RUST_ENV must be set");
    load_dotenv_file(&environment);
}

// Helper function to get environment variables
fn get_env_var(key: &str) -> String {
    env::var(key).expect(&format!("{} must be set", key))
}


// Get the database URL from environment variables
pub fn get_database_url() -> String {
    get_env_var("DATABASE_URL")
}

// Get the port number from environment variables
pub fn get_port() -> u16 {
    get_env_var("PORT").parse().expect("PORT must be a number")
}

// Get the host address from environment variables
pub fn get_host() -> String {
    get_env_var("HOST")
}

// Get the RUST_LOG environment variable if set
pub fn get_rust_log() -> Option<String> {
    env::var("RUST_LOG").ok()
}

// Get the maximum number of database connections from environment variables
pub fn get_max_db_connections() -> Option<u32> {
    env::var("MAX_DB_CONNECTIONS").ok().map(|s| s.parse().expect("MAX_DB_CONNECTIONS must be a number"))
}