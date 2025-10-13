//! Punto de entrada de la aplicación.
//!
//! Aquí se realiza la configuración inicial del entorno, la conexión a la base de datos,
//! la ejecución de migraciones y el arranque del servidor HTTP basado en Axum.

use anyhow::{Context, Result};
use axum::Router;
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePool;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod handlers;
mod models;
mod routes;

/// Arranca el runtime principal, inicializando trazas, conexión a la base de datos
/// y ejecutando las migraciones antes de levantar el servidor HTTP.
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_tracing();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://db.sqlite".to_string());

    let database_pool = SqlitePool::connect(&database_url)
        .await
        .with_context(|| format!("No se pudo conectar a la base de datos en {}", database_url))?;

    sqlx::migrate!("./migrations")
        .run(&database_pool)
        .await
        .context("Fallo al ejecutar migraciones")?;

    let application_router = Router::new()
        .merge(routes::user_routes())
        .merge(routes::health_routes())
        .merge(routes::root_route())
        .nest_service("/public", ServeDir::new("public"))
        .with_state(database_pool.clone());

    let listener_address = build_socket_addr()?;
    let tcp_listener = TcpListener::bind(listener_address)
        .await
        .with_context(|| format!("No se pudo abrir el puerto {}", listener_address))?;

    info!("Servidor corriendo en http://{}", listener_address);

    axum::serve(tcp_listener, application_router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Error al ejecutar el servidor")?;

    Ok(())
}

/// Configura la suscripción de trazas leyendo el filtro desde variables de entorno
/// y utilizando un formato compacto apto para consola.
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

/// Construye la dirección en la que escuchará el servidor a partir de las variables
/// de entorno `HOST` y `PORT`, aplicando valores por defecto cuando corresponda.
fn build_socket_addr() -> Result<SocketAddr> {
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);

    format!("{host}:{port}")
        .parse::<SocketAddr>()
        .with_context(|| format!("HOST o PORT inválidos: {host}:{port}"))
}

/// Espera la señal de `Ctrl+C` para realizar un apagado ordenado del servidor.
async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!(?error, "Error al esperar la señal Ctrl+C");
    }

    info!("Señal de apagado recibida, cerrando servidor…");
}