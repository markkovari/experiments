use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::{
    domain::{CreatePostInput, PostResponse, UpdatePostInput},
    use_cases,
};
use crate::{
    middleware::{auth::RequireAuth, validation::validate_request},
    shared::{
        error::AppResult,
        response::{Created, JsonResponse, NoContent},
    },
    AppState,
};

// Functional handlers

pub async fn get_all_posts_handler(
    State(state): State<AppState>,
) -> AppResult<Json<JsonResponse<Vec<PostResponse>>>> {
    let posts = use_cases::get_all_posts(&state.post_repository).await?;
    let response: Vec<PostResponse> = posts.into_iter().map(|p| p.to_response()).collect();
    Ok(Json(JsonResponse::new(response)))
}

pub async fn get_published_posts_handler(
    State(state): State<AppState>,
) -> AppResult<Json<JsonResponse<Vec<PostResponse>>>> {
    let posts = use_cases::get_published_posts(&state.post_repository).await?;
    let response: Vec<PostResponse> = posts.into_iter().map(|p| p.to_response()).collect();
    Ok(Json(JsonResponse::new(response)))
}

pub async fn get_post_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<JsonResponse<PostResponse>>> {
    let post = use_cases::get_post_by_id(&state.post_repository, id).await?;
    Ok(Json(JsonResponse::new(post.to_response())))
}

pub async fn get_posts_by_author_handler(
    State(state): State<AppState>,
    Path(author_id): Path<Uuid>,
) -> AppResult<Json<JsonResponse<Vec<PostResponse>>>> {
    let posts = use_cases::get_posts_by_author(&state.post_repository, author_id).await?;
    let response: Vec<PostResponse> = posts.into_iter().map(|p| p.to_response()).collect();
    Ok(Json(JsonResponse::new(response)))
}

pub async fn create_post_handler(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Json(input): Json<CreatePostInput>,
) -> AppResult<Created<PostResponse>> {
    validate_request(&input)?;
    let post = use_cases::create_post(&state.post_repository, input, user.id).await?;
    Ok(Created(post.to_response()))
}

pub async fn update_post_handler(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdatePostInput>,
) -> AppResult<Json<JsonResponse<PostResponse>>> {
    validate_request(&input)?;
    let post = use_cases::update_post(&state.post_repository, id, input, user.id).await?;
    Ok(Json(JsonResponse::new(post.to_response())))
}

pub async fn delete_post_handler(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
) -> AppResult<NoContent> {
    use_cases::delete_post(&state.post_repository, id, user.id).await?;
    Ok(NoContent)
}
