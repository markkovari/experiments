use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = Value)
    )
)]
pub async fn health_check() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "veterinary-clinic-api"
        })),
    )
}
