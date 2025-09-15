#[allow(unused_imports)]
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::{Pool, Sqlite};

use crate::handlers::user::{create_user, delete_user, get_user, list_users, update_user};

pub fn user_routes() -> Router<Pool<Sqlite>> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
}
