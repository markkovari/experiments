use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;
use thiserror::Error;

/// Custom application error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation failed")]
    Validation(Vec<ValidationError>),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Password hashing error")]
    PasswordHash,

    #[error("Internal server error")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Convert AppError into HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, details): (StatusCode, String, Option<Vec<serde_json::Value>>) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg, None),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, None),
            AppError::Validation(errors) => {
                let error_details: Vec<_> = errors
                    .iter()
                    .map(|e| json!({ "field": e.field, "message": e.message }))
                    .collect();
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Validation failed",
                        "details": error_details
                    })),
                )
                    .into_response();
            }
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                    None,
                )
            }
            AppError::Jwt(err) => {
                tracing::error!("JWT error: {:?}", err);
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string(), None)
            }
            AppError::PasswordHash => {
                tracing::error!("Password hashing error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Authentication error".to_string(),
                    None,
                )
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg, None)
            }
        };

        let body = if let Some(details) = details {
            json!({
                "error": message,
                "details": details
            })
        } else {
            json!({ "error": message })
        };

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
