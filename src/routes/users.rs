use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::SqlitePool;

use crate::handlers::user::{create_user, delete_user, get_user, list_users};

pub fn user_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).delete(delete_user))
}
