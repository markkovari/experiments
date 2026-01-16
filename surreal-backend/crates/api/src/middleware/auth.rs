use axum::{extract::Request, http::header, middleware::Next, response::Response};
use surreal_core::{verify_token, Claims, UserRole};

use crate::error::{ApiError, ApiResult};

/// Extract and verify JWT from Authorization header
pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, ApiError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid authorization header format".to_string()))?;

    // Verify token and extract claims
    let claims =
        verify_token(token).map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))?;

    // Insert claims into request extensions for handlers to access
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Middleware to require specific role
pub fn require_role(
    required_role: UserRole,
) -> impl Fn(
    Request,
    Next,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, ApiError>> + Send>>
       + Clone {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            let claims = request.extensions().get::<Claims>().ok_or_else(|| {
                ApiError::Unauthorized("Missing authentication claims".to_string())
            })?;

            let user_role = match claims.role.as_str() {
                "user" => UserRole::User,
                "doctor" => UserRole::Doctor,
                _ => return Err(ApiError::Unauthorized("Invalid role".to_string())),
            };

            if user_role != required_role {
                return Err(ApiError::Forbidden(format!(
                    "Required role: {:?}",
                    required_role
                )));
            }

            Ok(next.run(request).await)
        })
    }
}

/// Extract current user claims from request
pub fn get_current_user(request: &Request) -> ApiResult<&Claims> {
    request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".to_string()))
}
