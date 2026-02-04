use uuid::Uuid;

use super::{
    domain::{CreatePostInput, Post, UpdatePostInput},
    repository::PostRepository,
};
use crate::shared::error::{AppError, AppResult};

// Functional approach: Pure functions that take repository as parameter

pub async fn get_all_posts(repo: &PostRepository) -> AppResult<Vec<Post>> {
    repo.find_all().await
}

pub async fn get_published_posts(repo: &PostRepository) -> AppResult<Vec<Post>> {
    repo.find_published().await
}

pub async fn get_post_by_id(repo: &PostRepository, id: Uuid) -> AppResult<Post> {
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))
}

pub async fn get_posts_by_author(repo: &PostRepository, author_id: Uuid) -> AppResult<Vec<Post>> {
    repo.find_by_author(author_id).await
}

pub async fn create_post(
    repo: &PostRepository,
    input: CreatePostInput,
    author_id: Uuid,
) -> AppResult<Post> {
    repo.create(input, author_id).await
}

pub async fn update_post(
    repo: &PostRepository,
    id: Uuid,
    input: UpdatePostInput,
    user_id: Uuid,
) -> AppResult<Post> {
    // Get existing post
    let post = get_post_by_id(repo, id).await?;

    // Check if user is the author
    if post.author_id != user_id {
        return Err(AppError::Unauthorized(
            "You can only edit your own posts".to_string(),
        ));
    }

    // Update post
    repo.update(id, input)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))
}

pub async fn delete_post(repo: &PostRepository, id: Uuid, user_id: Uuid) -> AppResult<()> {
    // Get existing post
    let post = get_post_by_id(repo, id).await?;

    // Check if user is the author
    if post.author_id != user_id {
        return Err(AppError::Unauthorized(
            "You can only delete your own posts".to_string(),
        ));
    }

    // Delete post
    let deleted = repo.delete(id).await?;
    if !deleted {
        return Err(AppError::NotFound("Post not found".to_string()));
    }

    Ok(())
}
