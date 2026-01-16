use async_trait::async_trait;

use surreal_core::{PaginatedResponse, PaginationParams, Pet};

use crate::connection::Database;
use crate::error::{DbError, Result};
use crate::repository::Repository;

const TABLE: &str = "pets";

pub struct PetRepository {
    db: Database,
}

impl PetRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn find_by_owner(&self, owner_id: &str) -> Result<Vec<Pet>> {
        let mut result = self
            .db
            .client
            .query("SELECT * FROM pets WHERE owner_id = $owner_id")
            .bind(("owner_id", owner_id.to_string()))
            .await?;

        Ok(result.take(0)?)
    }

    pub async fn find_by_owner_paginated(
        &self,
        owner_id: &str,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<Pet>> {
        let owner_id_owned = owner_id.to_string();
        let offset = params.offset();
        let limit = params.limit();

        // Get total count
        let mut count_result = self
            .db
            .client
            .query("SELECT count() FROM pets WHERE owner_id = $owner_id GROUP ALL")
            .bind(("owner_id", owner_id_owned.clone()))
            .await?;

        let count: Option<u64> = count_result.take("count")?;
        let total_items = count.unwrap_or(0);

        // Get paginated data
        let mut result = self
            .db
            .client
            .query("SELECT * FROM pets WHERE owner_id = $owner_id ORDER BY created_at DESC LIMIT $limit START $offset")
            .bind(("owner_id", owner_id_owned))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        let data: Vec<Pet> = result.take(0)?;

        Ok(PaginatedResponse::new(data, params, total_items))
    }
}

#[async_trait]
impl Repository<Pet> for PetRepository {
    async fn create(&self, pet: &Pet) -> Result<Pet> {
        let created: Option<Pet> = self.db.client.create(TABLE).content(pet.clone()).await?;

        created.ok_or_else(|| DbError::Other("Failed to create pet".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Pet>> {
        Ok(self.db.client.select((TABLE, id)).await?)
    }

    async fn find_all(&self) -> Result<Vec<Pet>> {
        Ok(self.db.client.select(TABLE).await?)
    }

    async fn update(&self, pet: &Pet) -> Result<Pet> {
        let id = pet
            .id
            .as_ref()
            .ok_or_else(|| DbError::Other("Pet ID is required for update".to_string()))?;

        let updated: Option<Pet> = self
            .db
            .client
            .update((TABLE, id.as_str()))
            .content(pet.clone())
            .await?;

        updated.ok_or_else(|| DbError::NotFound(format!("Pet with id {} not found", id)))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let deleted: Option<Pet> = self.db.client.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }

    async fn find_paginated(&self, params: &PaginationParams) -> Result<PaginatedResponse<Pet>> {
        let offset = params.offset();
        let limit = params.limit();

        // Get total count
        let mut count_result = self
            .db
            .client
            .query("SELECT count() FROM pets GROUP ALL")
            .await?;

        let count: Option<u64> = count_result.take("count")?;
        let total_items = count.unwrap_or(0);

        // Get paginated data
        let mut result = self
            .db
            .client
            .query("SELECT * FROM pets ORDER BY created_at DESC LIMIT $limit START $offset")
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        let data: Vec<Pet> = result.take(0)?;

        Ok(PaginatedResponse::new(data, params, total_items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surreal_core::PetSpecies;

    #[tokio::test]
    async fn test_create_and_find_pet() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = PetRepository::new(db);

        let owner_id = "users:test123".to_string();
        let pet = Pet::new(owner_id, "Buddy".to_string(), PetSpecies::Dog).unwrap();
        let created = repo.create(&pet).await.unwrap();

        assert_eq!(created.name, pet.name);
        assert!(created.id.is_some());

        let id = created.id.as_ref().unwrap();

        let found = repo.find_by_id(id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Buddy");
    }

    #[tokio::test]
    async fn test_find_by_owner() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = PetRepository::new(db);

        let owner_id = "users:test123".to_string();
        let pet1 = Pet::new(owner_id.clone(), "Max".to_string(), PetSpecies::Dog).unwrap();
        let pet2 = Pet::new(owner_id.clone(), "Whiskers".to_string(), PetSpecies::Cat).unwrap();

        repo.create(&pet1).await.unwrap();
        repo.create(&pet2).await.unwrap();

        let pets = repo.find_by_owner(&owner_id).await.unwrap();
        assert_eq!(pets.len(), 2);
    }
}
