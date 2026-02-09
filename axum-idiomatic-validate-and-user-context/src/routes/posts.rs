use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::errors::AppError;
use crate::models::post::{self, Post};
use crate::validate::ValidatedJson;
use crate::AppState;

#[derive(Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, message = "Title must not be empty"))]
    pub title: String,
    #[validate(length(min = 1, message = "Content must not be empty"))]
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_posts).post(create_post))
        .route(
            "/{id}",
            get(get_post).put(update_post).delete(delete_post),
        )
}

async fn list_posts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Post>>, AppError> {
    let posts = if auth.is_admin() {
        post::list_all_posts(&state.pool).await?
    } else {
        post::list_posts(&state.pool, auth.user_id).await?
    };
    Ok(Json(posts))
}

async fn create_post(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<CreatePostRequest>,
) -> Result<(StatusCode, Json<Post>), AppError> {
    let new_post = post::create_post(&state.pool, auth.user_id, &body.title, &body.content).await?;
    Ok((StatusCode::CREATED, Json(new_post)))
}

async fn get_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Post>, AppError> {
    let found = if auth.is_admin() {
        post::get_any_post(&state.pool, id).await?
    } else {
        post::get_post(&state.pool, id, auth.user_id).await?
    }
    .ok_or_else(|| AppError::NotFound("Post not found".into()))?;
    Ok(Json(found))
}

async fn update_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePostRequest>,
) -> Result<Json<Post>, AppError> {
    let updated = if auth.is_admin() {
        post::update_any_post(&state.pool, id, body.title.as_deref(), body.content.as_deref())
            .await?
    } else {
        post::update_post(
            &state.pool,
            id,
            auth.user_id,
            body.title.as_deref(),
            body.content.as_deref(),
        )
        .await?
    }
    .ok_or_else(|| AppError::NotFound("Post not found".into()))?;
    Ok(Json(updated))
}

async fn delete_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = if auth.is_admin() {
        post::delete_any_post(&state.pool, id).await?
    } else {
        post::delete_post(&state.pool, id, auth.user_id).await?
    };
    if !deleted {
        return Err(AppError::NotFound("Post not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}
