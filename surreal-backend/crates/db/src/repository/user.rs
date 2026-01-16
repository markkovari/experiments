use async_trait::async_trait;

use surreal_core::{PaginatedResponse, PaginationParams, User};

use crate::connection::Database;
use crate::error::{DbError, Result};
use crate::repository::Repository;

const TABLE: &str = "users";

pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let email_owned = email.to_string();
        let mut result = self
            .db
            .client
            .query("SELECT * FROM users WHERE email = $email")
            .bind(("email", email_owned))
            .await?;

        let users: Vec<User> = result.take(0)?;
        Ok(users.into_iter().next())
    }
}

#[async_trait]
impl Repository<User> for UserRepository {
    async fn create(&self, user: &User) -> Result<User> {
        let created: Option<User> = self.db.client.create(TABLE).content(user.clone()).await?;

        created.ok_or_else(|| DbError::Other("Failed to create user".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>> {
        Ok(self.db.client.select((TABLE, id)).await?)
    }

    async fn find_all(&self) -> Result<Vec<User>> {
        Ok(self.db.client.select(TABLE).await?)
    }

    async fn update(&self, user: &User) -> Result<User> {
        let id = user
            .id
            .as_ref()
            .ok_or_else(|| DbError::Other("User ID is required for update".to_string()))?;

        let updated: Option<User> = self
            .db
            .client
            .update((TABLE, id.as_str()))
            .content(user.clone())
            .await?;

        updated.ok_or_else(|| DbError::NotFound(format!("User with id {} not found", id)))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let deleted: Option<User> = self.db.client.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }

    async fn find_paginated(&self, params: &PaginationParams) -> Result<PaginatedResponse<User>> {
        let offset = params.offset();
        let limit = params.limit();

        // Get total count
        let mut count_result = self
            .db
            .client
            .query("SELECT count() FROM users GROUP ALL")
            .await?;

        let count: Option<u64> = count_result.take("count")?;
        let total_items = count.unwrap_or(0);

        // Get paginated data
        let mut result = self
            .db
            .client
            .query("SELECT * FROM users ORDER BY created_at DESC LIMIT $limit START $offset")
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        let data: Vec<User> = result.take(0)?;

        Ok(PaginatedResponse::new(data, params, total_items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_find_user() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = UserRepository::new(db);

        let user = User::new("test@example.com".to_string(), "John Doe".to_string()).unwrap();
        let created = repo.create(&user).await.unwrap();

        assert_eq!(created.email, user.email);
        assert!(created.id.is_some());

        let id = created.id.as_ref().unwrap();
        let found = repo.find_by_id(id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, user.email);
    }

    #[tokio::test]
    async fn test_find_by_email() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = UserRepository::new(db);

        let user = User::new("unique@example.com".to_string(), "Jane Doe".to_string()).unwrap();
        repo.create(&user).await.unwrap();

        let found = repo.find_by_email("unique@example.com").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Jane Doe");
    }
}
