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

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_tracing();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://db.sqlite".to_string());
    let db_pool = SqlitePool::connect(&database_url)
        .await
        .with_context(|| format!("No se pudo conectar a la base de datos en {}", database_url))?;

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .context("Fallo al ejecutar migraciones")?;

    let app = Router::new()
        .merge(routes::user_routes())
        .merge(routes::health_routes())
        .merge(routes::root_route())
        .nest_service("/public", ServeDir::new("public"))
        .with_state(db_pool.clone());

    let addr = build_socket_addr()?;
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("No se pudo abrir el puerto {}", addr))?;

    info!("Servidor corriendo en http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Error al ejecutar el servidor")?;

    Ok(())
}

fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

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

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!(?error, "Error al esperar la señal Ctrl+C");
    }

    info!("Señal de apagado recibida, cerrando servidor…");
}
