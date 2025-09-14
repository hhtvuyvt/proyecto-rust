use axum::{routing::get, Router};
use sqlx::SqlitePool;

async fn health_check() -> &'static str {
    "OK"
}

pub fn health_routes() -> Router<SqlitePool> {
    Router::new().route("/health", get(health_check))
}
