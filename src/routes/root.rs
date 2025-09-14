use axum::{routing::get, Router};
use sqlx::SqlitePool;

async fn index() -> &'static str {
    "Bienvenido a la API en Rust 🚀"
}

pub fn root_route() -> Router<SqlitePool> {
    Router::new().route("/", get(index))
}
