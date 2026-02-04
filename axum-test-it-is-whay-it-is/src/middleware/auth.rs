use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{
    auth::jwt::{verify_token, AuthUser},
    shared::error::AppError,
    AppState,
};

/// JWT authentication extractor
///
/// Use this in handlers to require authentication:
/// ```ignore
/// async fn handler(
///     RequireAuth(user): RequireAuth,
/// ) -> Result<Json<User>, AppError> {
///     // user is authenticated
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RequireAuth(pub AuthUser);

#[async_trait]
impl FromRequestParts<AppState> for RequireAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized("Missing authorization header".to_string()))?;

        // Verify the token
        let claims = verify_token(bearer.token(), &state.jwt_secret)?;

        // Convert claims to AuthUser
        let user = claims.to_auth_user()?;

        Ok(RequireAuth(user))
    }
}

/// Optional JWT authentication extractor
///
/// Use this in handlers for optional authentication:
/// ```ignore
/// async fn handler(
///     OptionalAuth(user): OptionalAuth,
/// ) -> Result<Json<Response>, AppError> {
///     if let Some(user) = user {
///         // user is authenticated
///     } else {
///         // no authentication provided
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthUser>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try to extract the Authorization header
        let auth_header = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .ok();

        if let Some(TypedHeader(Authorization(bearer))) = auth_header {
            // Try to verify the token
            if let Ok(claims) = verify_token(bearer.token(), &state.jwt_secret) {
                if let Ok(user) = claims.to_auth_user() {
                    return Ok(OptionalAuth(Some(user)));
                }
            }
        }

        Ok(OptionalAuth(None))
    }
}
