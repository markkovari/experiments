use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::domain::{CreateUserInput, UpdateUserInput, User};
use crate::{
    auth::password::hash_password,
    shared::error::{AppError, AppResult},
};

#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn find_all(&self) -> AppResult<Vec<User>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn create(&self, input: CreateUserInput) -> AppResult<User>;
    async fn update(&self, id: Uuid, input: UpdateUserInput) -> AppResult<Option<User>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn find_all(&self) -> AppResult<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn create(&self, input: CreateUserInput) -> AppResult<User> {
        // Check if email already exists
        if let Some(_) = self.find_by_email(&input.email).await? {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        // Hash the password
        let password_hash = hash_password(&input.password)?;

        // Insert user
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, name, password_hash) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&input.email)
        .bind(&input.name)
        .bind(&password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update(&self, id: Uuid, input: UpdateUserInput) -> AppResult<Option<User>> {
        // Check if user exists
        if self.find_by_id(id).await?.is_none() {
            return Ok(None);
        }

        // Check email uniqueness if updating email
        if let Some(ref email) = input.email {
            if let Some(existing) = self.find_by_email(email).await? {
                if existing.id != id {
                    return Err(AppError::Conflict("Email already exists".to_string()));
                }
            }
        }

        // Build dynamic update query
        let mut query = String::from("UPDATE users SET ");
        let mut updates = Vec::new();
        let mut bind_count = 1;

        if input.email.is_some() {
            updates.push(format!("email = ${}", bind_count));
            bind_count += 1;
        }

        if input.name.is_some() {
            updates.push(format!("name = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            // No updates provided
            return self.find_by_id(id).await;
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(" WHERE id = ${} RETURNING *", bind_count));

        let mut query_builder = sqlx::query_as::<_, User>(&query);

        if let Some(email) = input.email {
            query_builder = query_builder.bind(email);
        }

        if let Some(name) = input.name {
            query_builder = query_builder.bind(name);
        }

        query_builder = query_builder.bind(id);

        let user = query_builder.fetch_one(&self.pool).await?;

        Ok(Some(user))
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
