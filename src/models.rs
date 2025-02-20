use diesel::prelude::*;
use diesel::{Queryable, Insertable};
use diesel::internal::derives::multiconnection::chrono::NaiveDateTime;

use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};
use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;
use once_cell::sync::Lazy;
use crate::schema::users;
use crate::schema::sessions;

// lazy_static! {
//     static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
// }

static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()
});

#[derive(Debug, Queryable, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub role: String,
    pub status: String,
    pub login_attempts: i16,
    pub last_login_at: Option<NaiveDateTime>,
    pub password_changed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize, Validate)]
#[diesel(table_name = users)]
pub struct NewUser {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    #[validate(regex(path = *USERNAME_REGEX))]
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password_hash: String,  // Will be hashed before storage
    pub full_name: Option<String>,
    pub role: String,
    pub status: String,
}

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: i32,
    pub user_id: Uuid,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: NaiveDateTime,
    pub created_at: Option<NaiveDateTime>,
}



#[derive(Insertable,Deserialize)]
#[diesel(table_name = sessions)]
pub struct NewSession {
    pub user_id: Uuid,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: NaiveDateTime
}