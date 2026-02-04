use uuid::Uuid;

use super::{
    domain::{CreateUserInput, LoginInput, LoginResponse, UpdateUserInput, User},
    repository::UserRepositoryTrait,
};
use crate::{
    auth::{jwt::generate_token, password::verify_password},
    shared::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct UserService<R: UserRepositoryTrait> {
    repository: R,
    jwt_secret: String,
}

impl<R: UserRepositoryTrait> UserService<R> {
    pub fn new(repository: R, jwt_secret: String) -> Self {
        Self {
            repository,
            jwt_secret,
        }
    }

    pub async fn get_all(&self) -> AppResult<Vec<User>> {
        self.repository.find_all().await
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<User> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn create(&self, input: CreateUserInput) -> AppResult<User> {
        self.repository.create(input).await
    }

    pub async fn update(&self, id: Uuid, input: UpdateUserInput) -> AppResult<User> {
        self.repository
            .update(id, input)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let deleted = self.repository.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound("User not found".to_string()));
        }
        Ok(())
    }

    pub async fn login(&self, input: LoginInput) -> AppResult<LoginResponse> {
        // Find user by email
        let user = self
            .repository
            .find_by_email(&input.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        // Verify password
        let is_valid = verify_password(&input.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        // Generate JWT token
        let token = generate_token(user.id, user.email.clone(), user.name.clone(), &self.jwt_secret)?;

        Ok(LoginResponse {
            token,
            user: user.to_response(),
        })
    }
}
