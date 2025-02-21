use dotenvy::dotenv;
use std::env;
use std::path::Path;

pub fn load_env() {
    // Load default .env file first
    dotenv().ok();
    
    // Then try to load environment-specific file if RUST_ENV is set
    if let Ok(env_name) = env::var("RUST_ENV") {
        let env_file = format!(".env.{}", env_name);
        if Path::new(&env_file).exists() {
            dotenvy::from_filename(env_file).ok();
        }
    }
}

pub fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file")
}

pub fn get_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number")
}

pub fn get_host() -> String {
    env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
}

pub fn get_rust_log() -> Option<String> {
    env::var("RUST_LOG").ok()
}

pub fn get_max_db_connections() -> Option<u32> {
    env::var("MAX_DB_CONNECTIONS").ok().map(|s| s.parse().expect("MAX_DB_CONNECTIONS must be a number"))
}