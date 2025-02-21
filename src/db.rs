// src/db.rs
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use crate::config::get_database_url;
pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool() -> PgPool {
    dotenv().ok();
    let manager = ConnectionManager::<PgConnection>::new(get_database_url());
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
