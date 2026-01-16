pub mod user;
pub mod pet;
pub mod doctor;
pub mod check;
pub mod auth;

use async_trait::async_trait;

pub use user::UserRepository;
pub use pet::PetRepository;
pub use doctor::DoctorRepository;
pub use check::CheckRepository;
pub use auth::AuthRepository;

use crate::error::Result;
use surreal_core::{PaginatedResponse, PaginationParams};

#[async_trait]
pub trait Repository<T: Clone> {
    async fn create(&self, entity: &T) -> Result<T>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn find_all(&self) -> Result<Vec<T>>;
    async fn update(&self, entity: &T) -> Result<T>;
    async fn delete(&self, id: &str) -> Result<bool>;

    /// Find entities with pagination
    async fn find_paginated(&self, params: &PaginationParams) -> Result<PaginatedResponse<T>> {
        // Default implementation that uses find_all
        // Repositories should override this for better performance
        let all_items = self.find_all().await?;
        let total_items = all_items.len() as u64;

        let start = params.offset() as usize;
        let end = (start + params.limit() as usize).min(all_items.len());
        let data = all_items[start..end].to_vec();

        Ok(PaginatedResponse::new(data, params, total_items))
    }
}
