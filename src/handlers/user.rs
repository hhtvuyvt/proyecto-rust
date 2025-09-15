use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::models::user::{CreateUser, UpdateUser, User};

pub async fn list_users(State(pool): State<Pool<Sqlite>>) -> Result<Json<Vec<User>>, StatusCode> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

pub async fn get_user(
    Path(id): Path<Uuid>,
    State(pool): State<Pool<Sqlite>>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}

pub async fn create_user(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO users (id, name, email, created_at) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&now)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = User {
        id,
        name: payload.name,
        email: payload.email,
        created_at: now,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user(
    Path(id): Path<Uuid>,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, StatusCode> {
    let current = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let name = payload.name.unwrap_or(current.name);
    let email = payload.email.unwrap_or(current.email);

    sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
        .bind(&name)
        .bind(&email)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated = User {
        id,
        name,
        email,
        created_at: current.created_at,
    };

    Ok(Json(updated))
}

pub async fn delete_user(
    Path(id): Path<Uuid>,
    State(pool): State<Pool<Sqlite>>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
