use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use surreal_core::User;
use surreal_db::{Repository, UserRepository};

use crate::dto::{CreateUserRequest, UpdateUserRequest};
use crate::error::ApiResult;
use crate::state::AppState;

pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> ApiResult<(StatusCode, Json<User>)> {
    let mut user = User::new(req.email, req.name)?;

    if let Some(phone) = req.phone {
        user = user.with_phone(phone)?;
    }

    if let Some(address) = req.address {
        user = user.with_address(address);
    }

    let repo = UserRepository::new(state.db);
    let created = repo.create(&user).await?;

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<User>> {
    let repo = UserRepository::new(state.db);
    let user = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("User {} not found", id)))?;

    Ok(Json(user))
}

pub async fn list_users(State(state): State<AppState>) -> ApiResult<Json<Vec<User>>> {
    let repo = UserRepository::new(state.db);
    let users = repo.find_all().await?;

    Ok(Json(users))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> ApiResult<Json<User>> {
    let repo = UserRepository::new(state.db);
    let mut user = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| surreal_db::DbError::NotFound(format!("User {} not found", id)))?;

    if let Some(name) = req.name {
        user.name = name;
    }

    if let Some(phone) = req.phone {
        user.phone = Some(phone);
    }

    if let Some(address) = req.address {
        user.address = Some(address);
    }

    user.updated_at = chrono::Utc::now();
    let updated = repo.update(&user).await?;

    Ok(Json(updated))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let repo = UserRepository::new(state.db);
    let deleted = repo.delete(&id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(surreal_db::DbError::NotFound(format!("User {} not found", id)).into())
    }
}
