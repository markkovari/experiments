pub mod user;
pub mod pet;
pub mod doctor;
pub mod check;

use async_trait::async_trait;

pub use user::UserRepository;
pub use pet::PetRepository;
pub use doctor::DoctorRepository;
pub use check::CheckRepository;

use crate::error::Result;

#[async_trait]
pub trait Repository<T> {
    async fn create(&self, entity: &T) -> Result<T>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn find_all(&self) -> Result<Vec<T>>;
    async fn update(&self, entity: &T) -> Result<T>;
    async fn delete(&self, id: &str) -> Result<bool>;
}
