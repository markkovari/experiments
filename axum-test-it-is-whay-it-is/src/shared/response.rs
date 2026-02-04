use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

/// Standard JSON response wrapper
#[derive(Serialize)]
pub struct JsonResponse<T: Serialize> {
    pub data: T,
}

impl<T: Serialize> JsonResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: Serialize> IntoResponse for JsonResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

/// Created response (201)
pub struct Created<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for Created<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::CREATED, Json(JsonResponse::new(self.0))).into_response()
    }
}

/// No content response (204)
pub struct NoContent;

impl IntoResponse for NoContent {
    fn into_response(self) -> axum::response::Response {
        StatusCode::NO_CONTENT.into_response()
    }
}
