use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = if let Some(db_err) = self.0.downcast_ref::<surreal_db::DbError>() {
            match db_err {
                surreal_db::DbError::NotFound(_) => (StatusCode::NOT_FOUND, db_err.to_string()),
                surreal_db::DbError::AlreadyExists(_) => (StatusCode::CONFLICT, db_err.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            }
        } else if let Some(_core_err) = self.0.downcast_ref::<surreal_core::CoreError>() {
            (StatusCode::BAD_REQUEST, self.0.to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
