use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
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

#[derive(Debug, Clone)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct UserChanges {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Clone)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn push(&mut self, field: &'static str, message: &'static str) {
        self.errors.push(ValidationError { field, message });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.errors.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{}: {}", error.field, error.message)?;
        }

        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

impl TryFrom<CreateUser> for NewUser {
    type Error = ValidationErrors;

    fn try_from(value: CreateUser) -> Result<Self, Self::Error> {
        let mut errors = ValidationErrors::new();

        let name = value.name.trim().to_string();
        if name.is_empty() {
            errors.push("name", "Debe contener al menos un carácter");
        } else if name.len() > 100 {
            errors.push("name", "Debe tener 100 caracteres o menos");
        }

        let email = value.email.trim().to_lowercase();
        if email.is_empty() {
            errors.push("email", "Debe contener al menos un carácter");
        } else if !is_valid_email(&email) {
            errors.push("email", "Formato de correo inválido");
        }

        if errors.is_empty() {
            Ok(Self { name, email })
        } else {
            Err(errors)
        }
    }
}

impl TryFrom<UpdateUser> for UserChanges {
    type Error = ValidationErrors;

    fn try_from(value: UpdateUser) -> Result<Self, Self::Error> {
        let mut errors = ValidationErrors::new();

        let name = value
            .name
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty());

        if let Some(ref name_value) = name {
            if name_value.len() > 100 {
                errors.push("name", "Debe tener 100 caracteres o menos");
            }
        }

        let email = value
            .email
            .map(|email| email.trim().to_lowercase())
            .filter(|email| !email.is_empty());

        if let Some(ref email_value) = email {
            if !is_valid_email(email_value) {
                errors.push("email", "Formato de correo inválido");
            }
        }

        if name.is_none() && email.is_none() {
            errors.push("general", "Debe proporcionar al menos un campo para actualizar");
        }

        if errors.is_empty() {
            Ok(Self { name, email })
        } else {
            Err(errors)
        }
    }
}

fn is_valid_email(email: &str) -> bool {
    let at_position = email.find('@');
    let dot_position = email.rfind('.');

    matches!(at_position, Some(at) if at > 0)
        && matches!(dot_position, Some(dot) if dot > at_position.unwrap() + 1 && dot < email.len() - 1)
}
