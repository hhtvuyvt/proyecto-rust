//! Rutas HTTP relacionadas con usuarios.
//!
//! Define las rutas y métodos soportados para operar sobre el recurso `/users`.

use axum::{
    routing::get,
    Router,
};
use sqlx::{Pool, Sqlite};

use crate::handlers::user::{create_user, delete_user, get_user, list_users, update_user};

/// Devuelve un router con todas las operaciones disponibles para usuarios.
pub fn user_routes() -> Router<Pool<Sqlite>> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
}