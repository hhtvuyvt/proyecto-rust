//! Handlers HTTP para gestionar usuarios.
//!
//! Cada función expone la lógica necesaria para responder a solicitudes relacionadas con
//! el recurso `users`, incluído listado, consulta, creación, actualización y eliminación.

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
    CreateUser,
    NewUser,
    UpdateUser,
    User,
    UserChanges,
    ValidationError,
    ValidationErrors,
};

/// Devuelve la lista completa de usuarios registrados.
pub async fn list_users(State(database_pool): State<Pool<Sqlite>>) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email, created_at FROM users")
        .fetch_all(&database_pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(users))
}

/// Recupera un usuario concreto identificado por su UUID.
pub async fn get_user(
    Path(user_id): Path<Uuid>,
    State(database_pool): State<Pool<Sqlite>>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, created_at FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(&database_pool)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => AppError::not_found(),
        other => AppError::from(other),
    })?;

    Ok(Json(user))
}

/// Crea un nuevo usuario validando los datos de entrada antes de persistirlos.
pub async fn create_user(
    State(database_pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let validated_user = NewUser::try_from(payload).map_err(AppError::validation)?;

    let user_id = Uuid::new_v4();
    let created_timestamp = chrono::Utc::now();

    sqlx::query("INSERT INTO users (id, name, email, created_at) VALUES (?, ?, ?, ?)")
        .bind(user_id)
        .bind(&validated_user.name)
        .bind(&validated_user.email)
        .bind(created_timestamp)
        .execute(&database_pool)
        .await
        .map_err(AppError::from)?;

    let user = User {
        id: user_id,
        name: validated_user.name,
        email: validated_user.email,
        created_at: created_timestamp,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

/// Actualiza un usuario existente aplicando solo los campos proporcionados en la solicitud.
pub async fn update_user(
    Path(user_id): Path<Uuid>,
    State(database_pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    let requested_changes = UserChanges::try_from(payload).map_err(AppError::validation)?;

    let mut transaction = database_pool.begin().await.map_err(AppError::from)?;
    let current_user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, created_at FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => AppError::not_found(),
        other => AppError::from(other),
    })?;

    let merged_name = requested_changes.name.unwrap_or(current_user.name);
    let merged_email = requested_changes.email.unwrap_or(current_user.email);

    sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
        .bind(&merged_name)
        .bind(&merged_email)
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(AppError::from)?;

    transaction.commit().await.map_err(AppError::from)?;

    let updated_user = User {
        id: user_id,
        name: merged_name,
        email: merged_email,
        created_at: current_user.created_at,
    };

    Ok(Json(updated_user))
}

/// Elimina un usuario concreto si existe.
pub async fn delete_user(
    Path(user_id): Path<Uuid>,
    State(database_pool): State<Pool<Sqlite>>,
) -> Result<StatusCode, AppError> {
    let deletion_result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(&database_pool)
        .await
        .map_err(AppError::from)?;

    if deletion_result.rows_affected() == 0 {
        return Err(AppError::not_found());
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Forma serializada del error que se devolverá en las respuestas HTTP.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<FieldError>>,
}

/// Error por campo utilizado para describir el detalle de validaciones fallidas.
#[derive(Debug, Serialize)]
struct FieldError {
    field: &'static str,
    message: &'static str,
}

/// Error personalizado que agrupa distintas situaciones a nivel aplicación.
#[derive(Debug)]
pub struct AppError {
    kind: AppErrorKind,
}

/// Enumeración interna para clasificar los errores posibles.
#[derive(Debug)]
enum AppErrorKind {
    Validation(ValidationErrors),
    NotFound,
    Sqlx(sqlx::Error),
}

impl AppError {
    /// Construye un error de validación.
    fn validation(errors: ValidationErrors) -> Self {
        Self {
            kind: AppErrorKind::Validation(errors),
        }
    }

    /// Construye un error de tipo "recurso no encontrado".
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