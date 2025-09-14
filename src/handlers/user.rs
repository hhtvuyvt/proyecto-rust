use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::models::user::User;

pub async fn list_users(State(pool): State<Pool<Sqlite>>) -> Result<Json<Vec<User>>, String> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(users))
}

pub async fn create_user(
    State(pool): State<Pool<Sqlite>>,
    Json(user): Json<User>,
) -> Result<Json<User>, String> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, name, email) VALUES (?, ?, ?)")
        .bind(id)
        .bind(&user.name)
        .bind(&user.email)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(User { id, ..user }))
}

pub async fn get_user(
    Path(id): Path<Uuid>,
    State(pool): State<Pool<Sqlite>>,
) -> Result<Json<User>, String> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(user))
}

pub async fn delete_user(
    Path(id): Path<Uuid>,
    State(pool): State<Pool<Sqlite>>,
) -> Result<String, String> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("User {} eliminado", id))
}
