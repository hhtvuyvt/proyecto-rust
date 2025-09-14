use axum::Router;
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

mod handlers;
mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // conexión a la base de datos
    let db_pool = SqlitePool::connect("sqlite://db.sqlite").await?;

    // construir app
    let app = Router::new()
        .merge(routes::user_routes())
        .merge(routes::health_routes())
        .merge(routes::root_route())
        .nest_service("/public", ServeDir::new("public"))
        .with_state(db_pool);

    // dirección
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Servidor corriendo en http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}
