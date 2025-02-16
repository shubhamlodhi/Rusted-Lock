use axum::{
    extract::{Json,State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use bcrypt::{hash, DEFAULT_COST, verify};
use diesel::prelude::*;
use chrono::{Utc, Duration, NaiveDateTime};
use crate::models::{NewUser, User, Session};
use crate::schema::users::dsl::{users,username,login_attempts,last_login_at};
use crate::schema::sessions::dsl::{sessions,token};
use crate::db::PgPool;
use jsonwebtoken::{encode, Header, EncodingKey};
use uuid::Uuid;
use diesel::Insertable;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn generate_jwt(user_name: &str, secret: &str) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(1))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_name.to_owned(),
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("JWT token generation failed").to_string()
}






#[derive(Deserialize,Serialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

// pub async fn login(Json(login_info): Json<LoginRequest>) -> impl IntoResponse {
//     // TODO: Validate the user credentials
//     // TODO: Query the database to check if the user exists and the password is correct
//     // TODO: Generate a session token or JWT if the login is successful
//     // TODO: Return an appropriate response
//     println!("Login info: {:?}", login_info);
//     (StatusCode::OK, "Login handler is not yet implemented")
// }



pub async fn login(
    State(pool): State<PgPool>,
    Json(login_info): Json<LoginRequest>,
) -> impl IntoResponse {
    // Query the database to check if the user exists
    let mut conn = pool.get().expect("Failed to get DB connection from pool");

    let user = users
        .filter(username.eq(&login_info.username))
        .first::<User>(&mut conn)
        .optional()
        .expect("Error loading user");


        if let Some(user) = user {

            let now = Utc::now().naive_utc();
            let mut login_attempts_count = user.login_attempts + 1;

            // Check if the account is locked
            if login_attempts_count > 3 {
                if let Some(last_attempt) = user.last_login_at {
                    if now.signed_duration_since(last_attempt) < Duration::minutes(1) {
                        return (StatusCode::FORBIDDEN, "Account locked. Try again later.".to_string());
                    }
                    else{
                        login_attempts_count = 1;
                    }
                }
            }

            // Verify the password
            if verify(&login_info.password, &user.password_hash).unwrap() {
                // Generate a session token or JWT (this is a placeholder, implement your own logic)
                update_login_attempts(&mut conn, &user.username, 0);

                let token_x = generate_jwt(&user.username, "your_secret_key");

                let new_session = Session {
                    id: 0,
                    user_id: user.id,
                    token: token_x.clone(),
                    expires_at: Utc::now().naive_utc()+Duration::minutes(1),
                    created_at: Some(Utc::now().naive_utc()),
                };


                diesel::insert_into(sessions)
                    .values(&new_session)
                    .execute(&mut conn)
                    .expect("Error saving session");




                return (StatusCode::OK, format!("Login successful, token: {:?}", token_x));
            }
            else {
                update_login_attempts(&mut conn, &user.username, login_attempts_count);
                if login_attempts_count >= 3 {
                    return (StatusCode::FORBIDDEN, "Account locked. Too many failed attempts.".to_string());
                } else {
                    return (StatusCode::UNAUTHORIZED, "Invalid password".to_string());
                }
            }
        }
        else {
            return (StatusCode::UNAUTHORIZED, "User not found".to_string());
        }
    }





fn update_login_attempts(conn: &mut PgConnection, user_name: &str,attempts_count: i16) {
    diesel::update(users.filter(username.eq(user_name)))
        .set((
            login_attempts.eq(attempts_count),
            last_login_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)
        .expect("Error incrementing login attempts");
}




#[derive(Deserialize,Serialize, Debug, Validate)]
pub struct RegisterRequest {
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    pub email: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        if self.email.is_empty() || !self.email.contains('@') {
            return Err("Invalid email address".to_string());
        }
        Ok(())
    }
}

// pub async fn register(Json(register_info): Json<RegisterRequest>) -> impl IntoResponse {
//     // TODO: Implement user registration logic
//     println!("Register info: {:?}", register_info);
//     (StatusCode::OK, "Register handler is not yet implemented")
// }



pub async fn register(
    State(pool): State<PgPool>,
    Json(register_info): Json<RegisterRequest>,

) -> impl IntoResponse {
    // Validate the input data
    if let Err(validation_errors) = register_info.validate() {
        return (StatusCode::BAD_REQUEST, format!("Validation errors: {:?}", validation_errors));
    }

    // Hash the password
    let password_hash = match hash(&register_info.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()),
    };

    // Create a new user instance
    let new_user = NewUser {
        email: register_info.email,
        username: register_info.username,
        password_hash,
        full_name: None,
        role: "user".to_string(),
        status: "active".to_string(),
    };




    // Insert the new user into the database
    let mut conn = pool.get().expect("Failed to get DB connection from pool");
    match diesel::insert_into(users).values(&new_user).execute(&mut conn) {
        Ok(_) => (StatusCode::CREATED, "User registered successfully".to_string()),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to register user".to_string()),
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LogoutRequest {
    token: String,
}

pub async fn logout(
    State(pool): State<PgPool>,
    Json(logout_info): Json<LogoutRequest>,
) -> impl IntoResponse {
    let mut conn = pool.get().expect("Failed to get DB connection from pool");




    //Invalidate the session token
    let result = diesel::delete(sessions.filter(token.eq(&logout_info.token)))
        .execute(&mut conn);

    match result {
        Ok(_) => (StatusCode::OK, "Logout successful".to_string()),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to logout".to_string()),
    };

    (StatusCode::OK, "Logout successful".to_string())
}