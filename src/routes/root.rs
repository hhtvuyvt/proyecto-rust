//! Rutas raíz del servicio.
//!
//! Contienen un mensaje de bienvenida útil para pruebas rápidas o documentación.

use axum::{routing::get, Router};
use sqlx::SqlitePool;

/// Devuelve un saludo sencillo que confirma el correcto despliegue.
async fn index() -> &'static str {
    "Bienvenido a la API en Rust 🚀"
}

/// Construye el router asociado a la ruta base `/`.
pub fn root_route() -> Router<SqlitePool> {
    Router::new().route("/", get(index))
}