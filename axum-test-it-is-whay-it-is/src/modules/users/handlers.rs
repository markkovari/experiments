use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::domain::{CreateUserInput, LoginInput, UpdateUserInput, UserResponse};
use crate::{
    middleware::{auth::RequireAuth, validation::validate_request},
    shared::{
        error::AppResult,
        response::{Created, JsonResponse, NoContent},
    },
    AppState,
};

// Handler functions
pub async fn register_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUserInput>,
) -> AppResult<Created<UserResponse>> {
    validate_request(&input)?;
    let user = state.user_service.create(input).await?;
    Ok(Created(user.to_response()))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> AppResult<Json<JsonResponse<super::domain::LoginResponse>>> {
    validate_request(&input)?;
    let response = state.user_service.login(input).await?;
    Ok(Json(JsonResponse::new(response)))
}

pub async fn get_all_users_handler(
    State(state): State<AppState>,
    RequireAuth(_user): RequireAuth,
) -> AppResult<Json<JsonResponse<Vec<UserResponse>>>> {
    let users = state.user_service.get_all().await?;
    let response: Vec<UserResponse> = users.into_iter().map(|u| u.to_response()).collect();
    Ok(Json(JsonResponse::new(response)))
}

pub async fn get_user_by_id_handler(
    State(state): State<AppState>,
    RequireAuth(_user): RequireAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<JsonResponse<UserResponse>>> {
    let user = state.user_service.get_by_id(id).await?;
    Ok(Json(JsonResponse::new(user.to_response())))
}

pub async fn update_user_handler(
    State(state): State<AppState>,
    RequireAuth(_user): RequireAuth,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUserInput>,
) -> AppResult<Json<JsonResponse<UserResponse>>> {
    validate_request(&input)?;
    let user = state.user_service.update(id, input).await?;
    Ok(Json(JsonResponse::new(user.to_response())))
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    RequireAuth(_user): RequireAuth,
    Path(id): Path<Uuid>,
) -> AppResult<NoContent> {
    state.user_service.delete(id).await?;
    Ok(NoContent)
}

pub async fn get_profile_handler(
    RequireAuth(user): RequireAuth,
    State(state): State<AppState>,
) -> AppResult<Json<JsonResponse<UserResponse>>> {
    let user_data = state.user_service.get_by_id(user.id).await?;
    Ok(Json(JsonResponse::new(user_data.to_response())))
}
