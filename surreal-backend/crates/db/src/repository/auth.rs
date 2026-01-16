use async_trait::async_trait;
use surreal_core::AuthUser;

use crate::connection::Database;
use crate::error::{DbError, Result};
use crate::repository::Repository;

const TABLE: &str = "auth_users";

pub struct AuthRepository {
    db: Database,
}

impl AuthRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Find auth user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<AuthUser>> {
        let email_owned = email.to_string();
        let mut result = self
            .db
            .client
            .query("SELECT * FROM auth_users WHERE email = $email LIMIT 1")
            .bind(("email", email_owned))
            .await?;

        let users: Vec<AuthUser> = result.take(0)?;
        Ok(users.into_iter().next())
    }

    /// Check if email already exists
    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        Ok(self.find_by_email(email).await?.is_some())
    }

    /// Find auth user by reference ID (user or doctor ID)
    pub async fn find_by_reference_id(&self, reference_id: &str) -> Result<Option<AuthUser>> {
        let ref_id_owned = reference_id.to_string();
        let mut result = self
            .db
            .client
            .query("SELECT * FROM auth_users WHERE reference_id = $ref_id LIMIT 1")
            .bind(("ref_id", ref_id_owned))
            .await?;

        let users: Vec<AuthUser> = result.take(0)?;
        Ok(users.into_iter().next())
    }

    /// Update password
    pub async fn update_password(&self, id: &str, new_password_hash: String) -> Result<bool> {
        let updated: Option<AuthUser> = self
            .db
            .client
            .update((TABLE, id))
            .merge(serde_json::json!({
                "password_hash": new_password_hash,
                "updated_at": surrealdb::sql::Datetime::default()
            }))
            .await?;

        Ok(updated.is_some())
    }
}

#[async_trait]
impl Repository<AuthUser> for AuthRepository {
    async fn create(&self, auth_user: &AuthUser) -> Result<AuthUser> {
        // Check if email already exists
        if self.email_exists(&auth_user.email).await? {
            return Err(DbError::Conflict(format!(
                "Email {} already exists",
                auth_user.email
            )));
        }

        let created: Option<AuthUser> = self
            .db
            .client
            .create(TABLE)
            .content(auth_user.clone())
            .await?;

        created.ok_or_else(|| DbError::Other("Failed to create auth user".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<AuthUser>> {
        Ok(self.db.client.select((TABLE, id)).await?)
    }

    async fn find_all(&self) -> Result<Vec<AuthUser>> {
        Ok(self.db.client.select(TABLE).await?)
    }

    async fn update(&self, auth_user: &AuthUser) -> Result<AuthUser> {
        let id = auth_user
            .id
            .as_ref()
            .ok_or_else(|| DbError::Other("Auth user ID required for update".to_string()))?;

        let updated: Option<AuthUser> = self
            .db
            .client
            .update((TABLE, id.as_str()))
            .content(auth_user.clone())
            .await?;

        updated.ok_or_else(|| DbError::NotFound(format!("Auth user {} not found", id)))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let deleted: Option<AuthUser> = self.db.client.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surreal_core::{hash_password, UserRole};

    #[tokio::test]
    async fn test_create_and_find_auth_user() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = AuthRepository::new(db);

        let password_hash = hash_password("test_password_123").unwrap();
        let auth_user = AuthUser::new(
            "test@example.com".to_string(),
            password_hash,
            UserRole::User,
            "users:test123".to_string(),
        )
        .unwrap();

        let created = repo.create(&auth_user).await.unwrap();
        assert_eq!(created.email, "test@example.com");
        assert_eq!(created.role, UserRole::User);
        assert!(created.id.is_some());

        // Find by email
        let found = repo.find_by_email("test@example.com").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_duplicate_email() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = AuthRepository::new(db);

        let password_hash = hash_password("test_password_123").unwrap();
        let auth_user1 = AuthUser::new(
            "duplicate@example.com".to_string(),
            password_hash.clone(),
            UserRole::User,
            "users:test1".to_string(),
        )
        .unwrap();

        repo.create(&auth_user1).await.unwrap();

        // Try to create another with same email
        let auth_user2 = AuthUser::new(
            "duplicate@example.com".to_string(),
            password_hash,
            UserRole::Doctor,
            "doctors:test2".to_string(),
        )
        .unwrap();

        let result = repo.create(&auth_user2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_by_reference_id() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = AuthRepository::new(db);

        let password_hash = hash_password("test_password_123").unwrap();
        let auth_user = AuthUser::new(
            "ref@example.com".to_string(),
            password_hash,
            UserRole::Doctor,
            "doctors:abc123".to_string(),
        )
        .unwrap();

        repo.create(&auth_user).await.unwrap();

        let found = repo.find_by_reference_id("doctors:abc123").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().reference_id, "doctors:abc123");
    }

    #[tokio::test]
    async fn test_update_password() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = AuthRepository::new(db);

        let password_hash = hash_password("old_password").unwrap();
        let auth_user = AuthUser::new(
            "update@example.com".to_string(),
            password_hash,
            UserRole::User,
            "users:update".to_string(),
        )
        .unwrap();

        let created = repo.create(&auth_user).await.unwrap();
        let id = created.id.unwrap();

        let new_password_hash = hash_password("new_password").unwrap();
        let updated = repo
            .update_password(&id, new_password_hash.clone())
            .await
            .unwrap();
        assert!(updated);

        let found = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(found.password_hash, new_password_hash);
    }
}
