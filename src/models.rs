// src/models.rs
use diesel::Queryable;
use diesel::Insertable;
use serde::{Serialize, Deserialize};
use crate::schema::users;

#[derive(Debug, Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}
