use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::encode_jwt;
use crate::errors::AppError;
use crate::models::user::{create_user, find_by_username, UserResponse};
use crate::validate::ValidatedJson;
use crate::AppState;

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 1, message = "Username must not be empty"))]
    pub username: String,
    #[validate(
        length(min = 1, message = "Email must not be empty"),
        email(message = "Email must be a valid email address")
    )]
    pub email: String,
    #[validate(length(min = 1, message = "Password must not be empty"))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username must not be empty"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password must not be empty"))]
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

async fn register(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    let user = create_user(&state.pool, &body.username, &body.email, &body.password).await?;
    let token = encode_jwt(&state.jwt_secret, user.id, user.role);
    let user_response = UserResponse::from(user);

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user_response,
        }),
    ))
}

async fn login(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = find_by_username(&state.pool, &body.username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".into()))?;

    let valid = bcrypt::verify(&body.password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized(
            "Invalid username or password".into(),
        ));
    }

    let token = encode_jwt(&state.jwt_secret, user.id, user.role);
    let user_response = UserResponse::from(user);

    Ok(Json(AuthResponse {
        token,
        user: user_response,
    }))
}
