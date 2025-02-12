use axum::{extract::{Path, State, Form}, response::IntoResponse};
use diesel::prelude::*;
use crate::models::{User, NewUser};  // Make sure NewUser is imported
use crate::schema::users::dsl::*;    // Import schema this way
use crate::schema;                   // Also import the schema module itself
use serde::Deserialize;
use crate::handlers::result_handler::handle_db_result;
use tokio::task;  // Add this for spawn_blocking
use crate::db;

type DbPool = db::PgPool;

// CreateUser struct to handle the request body for creating a new user.
#[derive(Deserialize)]
pub struct CreateUser {
    name: String,
    email: String,
}

pub async fn list_users(State(pool): State<DbPool>
) -> impl IntoResponse {
    let result = task::spawn_blocking(move || {
        use schema::users::dsl::*;
        let mut conn = pool.get().expect("Failed to get DB connection");
        users.load::<User>(&mut conn)
    })
    .await
    .unwrap();

    handle_db_result(result)
}

// Handler to create a new user.
pub async fn create_user(
    State(pool): State<DbPool>,
    Form(payload): Form<CreateUser>,
) -> impl IntoResponse {
    let new_user = NewUser {
        name: payload.name,
        email: payload.email,
    };

    let result = task::spawn_blocking(move || {
        use schema::users::dsl::*;
        let mut conn = pool.get().expect("Failed to get DB connection");
        diesel::insert_into(users)
            .values(&new_user)
            .get_result::<User>(&mut conn)
    })
    .await
    .unwrap();

    handle_db_result(result)
}

// Handler to delete a user.
pub async fn delete_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    let result = task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get DB connection");
        diesel::delete(users.find(user_id)).execute(&mut conn)
    })
    .await
    .unwrap();

    handle_db_result(result)
}


// UpdateUser struct to handle the request body for updating a user.
#[derive(Deserialize)]
pub struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
}

// First, create a changeset struct
#[derive(AsChangeset)]
#[diesel(table_name = schema::users)]
struct UserChangeset {
    name: Option<String>,
    email: Option<String>,
}

// Handler to update a user.    
pub async fn update_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
    Form(payload): Form<UpdateUser>,
) -> impl IntoResponse {
    let changes = UserChangeset {
        name: payload.name,
        email: payload.email,
    };

    let result = task::spawn_blocking(move || {
        use crate::schema::users::dsl::*;
        let mut conn = pool.get().expect("Failed to get DB connection");
        
        diesel::update(users.find(user_id))
            .set(&changes)
            .get_result::<User>(&mut conn)
    })
    .await
    .unwrap();

    handle_db_result(result)
}

// Handler to get a user by ID.
pub async fn get_user_by_id(
    State(pool): State<DbPool>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    let result = task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get DB connection");
        users.find(user_id).first::<User>(&mut conn)
    })
    .await
    .unwrap();

    handle_db_result(result)
}




