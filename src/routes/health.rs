//! Rutas de salud del servicio.
//!
//! Exponen un endpoint simple que permite verificar que la API está viva.

use axum::{routing::get, Router};
use sqlx::SqlitePool;

/// Responde con `OK` indicando que la API está operativa.
async fn health_check() -> &'static str {
    "OK"
}

/// Devuelve el router con los endpoints de salud.
pub fn health_routes() -> Router<SqlitePool> {
    Router::new().route("/health", get(health_check))
}