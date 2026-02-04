use sqlx::PgPool;
use uuid::Uuid;

use super::domain::{CreatePostInput, Post, UpdatePostInput};
use crate::shared::error::AppResult;

// Functional approach: Factory function that returns repository functions
pub fn create_post_repository(pool: PgPool) -> PostRepository {
    PostRepository { pool }
}

#[derive(Clone)]
pub struct PostRepository {
    pool: PgPool,
}

impl PostRepository {
    pub async fn find_all(&self) -> AppResult<Vec<Post>> {
        let posts = sqlx::query_as::<_, Post>("SELECT * FROM posts ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(posts)
    }

    pub async fn find_published(&self) -> AppResult<Vec<Post>> {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE published = true ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Post>> {
        let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(post)
    }

    pub async fn find_by_author(&self, author_id: Uuid) -> AppResult<Vec<Post>> {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE author_id = $1 ORDER BY created_at DESC",
        )
        .bind(author_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    pub async fn create(&self, input: CreatePostInput, author_id: Uuid) -> AppResult<Post> {
        let published = input.published.unwrap_or(false);

        let post = sqlx::query_as::<_, Post>(
            "INSERT INTO posts (title, content, published, author_id) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(&input.title)
        .bind(&input.content)
        .bind(published)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn update(&self, id: Uuid, input: UpdatePostInput) -> AppResult<Option<Post>> {
        // Check if post exists
        if self.find_by_id(id).await?.is_none() {
            return Ok(None);
        }

        // Build dynamic update query
        let mut query = String::from("UPDATE posts SET ");
        let mut updates = Vec::new();
        let mut bind_count = 1;

        if input.title.is_some() {
            updates.push(format!("title = ${}", bind_count));
            bind_count += 1;
        }

        if input.content.is_some() {
            updates.push(format!("content = ${}", bind_count));
            bind_count += 1;
        }

        if input.published.is_some() {
            updates.push(format!("published = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            // No updates provided
            return self.find_by_id(id).await;
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(" WHERE id = ${} RETURNING *", bind_count));

        let mut query_builder = sqlx::query_as::<_, Post>(&query);

        if let Some(title) = input.title {
            query_builder = query_builder.bind(title);
        }

        if let Some(content) = input.content {
            query_builder = query_builder.bind(content);
        }

        if let Some(published) = input.published {
            query_builder = query_builder.bind(published);
        }

        query_builder = query_builder.bind(id);

        let post = query_builder.fetch_one(&self.pool).await?;

        Ok(Some(post))
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
