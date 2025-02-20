// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (id) {
        id -> Int4,
        user_id -> Uuid,
        token -> Text,
        refresh_token -> Text,
        expires_at -> Timestamp,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Citext,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 100]
        full_name -> Nullable<Varchar>,
        #[max_length = 100]
        role -> Varchar,
        #[max_length = 100]
        status -> Varchar,
        login_attempts -> Int2,
        last_login_at -> Nullable<Timestamptz>,
        password_changed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    users,
);
