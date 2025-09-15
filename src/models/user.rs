use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: String, // texto RFC3339
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
}
