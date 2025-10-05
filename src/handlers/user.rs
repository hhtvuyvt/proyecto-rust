use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::{Pool, Sqlite};
use tracing::error;
use uuid::Uuid;

use crate::models::user::{
    CreateUser, NewUser, UpdateUser, User, UserChanges, ValidationError, ValidationErrors,
};

pub async fn list_users(State(pool): State<Pool<Sqlite>>) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email, created_at FROM users")
        .fetch_all(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(users))
}

pub async fn get_user(
    Path(id): Path<Uuid>,
    State(pool): State<Pool<Sqlite>>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT id, name, email, created_at FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => AppError::not_found(),
            other => AppError::from(other),
        })?;

    Ok(Json(user))
}

pub async fn create_user(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let validated = NewUser::try_from(payload).map_err(AppError::validation)?;

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO users (id, name, email, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&validated.name)
    .bind(&validated.email)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(AppError::from)?;

    let user = User {
        id,
        name: validated.name,
        email: validated.email,
        created_at: now,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user(
    Path(id): Path<Uuid>,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    let changes = UserChanges::try_from(payload).map_err(AppError::validation)?;

    let mut transaction = pool.begin().await.map_err(AppError::from)?;
    let current = sqlx::query_as::<_, User>(
        "SELECT id, name, email, created_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => AppError::not_found(),
        other => AppError::from(other),
    })?;

    let name = changes.name.unwrap_or(current.name);
    let email = changes.email.unwrap_or(current.email);

    sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
        .bind(&name)
        .bind(&email)
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(AppError::from)?;

    transaction.commit().await.map_err(AppError::from)?;

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
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(AppError::from)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found());
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<FieldError>>,
}

#[derive(Debug, Serialize)]
struct FieldError {
    field: &'static str,
    message: &'static str,
}

#[derive(Debug)]
pub struct AppError {
    kind: AppErrorKind,
}

#[derive(Debug)]
enum AppErrorKind {
    Validation(ValidationErrors),
    NotFound,
    Sqlx(sqlx::Error),
}

impl AppError {
    fn validation(errors: ValidationErrors) -> Self {
        Self {
            kind: AppErrorKind::Validation(errors),
        }
    }

    fn not_found() -> Self {
        Self {
            kind: AppErrorKind::NotFound,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self {
            kind: AppErrorKind::Sqlx(error),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self.kind {
            AppErrorKind::Validation(errors) => {
                let details = errors
                    .errors
                    .into_iter()
                    .map(|ValidationError { field, message }| FieldError { field, message })
                    .collect::<Vec<_>>();

                let body = Json(ErrorResponse {
                    message: "Datos de entrada inválidos",
                    errors: Some(details),
                });

                (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
            }
            AppErrorKind::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    message: "Recurso no encontrado",
                    errors: None,
                }),
            )
                .into_response(),
            AppErrorKind::Sqlx(error) => {
                error!(?error, "Error en la base de datos");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        message: "Ocurrió un error inesperado",
                        errors: None,
                    }),
                )
                    .into_response()
            }
        }
    }
}
