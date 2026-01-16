use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    InternalServerError(String),
    Database(surreal_db::DbError),
    Core(surreal_core::CoreError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Database(db_err) => match db_err {
                surreal_db::DbError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                surreal_db::DbError::AlreadyExists(msg) => (StatusCode::CONFLICT, msg),
                surreal_db::DbError::Conflict(msg) => (StatusCode::CONFLICT, msg),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, db_err.to_string()),
            },
            ApiError::Core(core_err) => match core_err {
                surreal_core::CoreError::AuthError(msg) => (StatusCode::UNAUTHORIZED, msg),
                surreal_core::CoreError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
                _ => (StatusCode::BAD_REQUEST, core_err.to_string()),
            },
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<surreal_db::DbError> for ApiError {
    fn from(err: surreal_db::DbError) -> Self {
        ApiError::Database(err)
    }
}

impl From<surreal_core::CoreError> for ApiError {
    fn from(err: surreal_core::CoreError) -> Self {
        ApiError::Core(err)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::InternalServerError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(err.to_string())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
