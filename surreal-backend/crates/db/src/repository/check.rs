use async_trait::async_trait;

use surreal_core::{CheckStatus, HealthCheck, PaginatedResponse, PaginationParams};

use crate::connection::Database;
use crate::error::{DbError, Result};
use crate::repository::Repository;

const TABLE: &str = "health_checks";

pub struct CheckRepository {
    db: Database,
}

impl CheckRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn find_by_pet(&self, pet_id: &str) -> Result<Vec<HealthCheck>> {
        let pet_id_owned = pet_id.to_string();
        let mut result = self
            .db
            .client
            .query("SELECT * FROM health_checks WHERE pet_id = $pet_id ORDER BY scheduled_at DESC")
            .bind(("pet_id", pet_id_owned))
            .await?;

        Ok(result.take(0)?)
    }

    pub async fn find_by_doctor(&self, doctor_id: &str) -> Result<Vec<HealthCheck>> {
        let doctor_id_owned = doctor_id.to_string();
        let mut result = self
            .db
            .client
            .query("SELECT * FROM health_checks WHERE doctor_id = $doctor_id ORDER BY scheduled_at DESC")
            .bind(("doctor_id", doctor_id_owned))
            .await?;

        Ok(result.take(0)?)
    }

    pub async fn find_by_status(&self, status: CheckStatus) -> Result<Vec<HealthCheck>> {
        let mut result = self
            .db
            .client
            .query("SELECT * FROM health_checks WHERE status = $status ORDER BY scheduled_at ASC")
            .bind(("status", status))
            .await?;

        Ok(result.take(0)?)
    }

    pub async fn find_by_pet_paginated(
        &self,
        pet_id: &str,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<HealthCheck>> {
        let pet_id_owned = pet_id.to_string();
        let offset = params.offset();
        let limit = params.limit();

        // Get total count
        let mut count_result = self
            .db
            .client
            .query("SELECT count() FROM health_checks WHERE pet_id = $pet_id GROUP ALL")
            .bind(("pet_id", pet_id_owned.clone()))
            .await?;

        let count: Option<u64> = count_result.take("count")?;
        let total_items = count.unwrap_or(0);

        // Get paginated data
        let mut result = self
            .db
            .client
            .query("SELECT * FROM health_checks WHERE pet_id = $pet_id ORDER BY scheduled_at DESC LIMIT $limit START $offset")
            .bind(("pet_id", pet_id_owned))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        let data: Vec<HealthCheck> = result.take(0)?;

        Ok(PaginatedResponse::new(data, params, total_items))
    }

    pub async fn find_by_doctor_paginated(
        &self,
        doctor_id: &str,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<HealthCheck>> {
        let doctor_id_owned = doctor_id.to_string();
        let offset = params.offset();
        let limit = params.limit();

        // Get total count
        let mut count_result = self
            .db
            .client
            .query("SELECT count() FROM health_checks WHERE doctor_id = $doctor_id GROUP ALL")
            .bind(("doctor_id", doctor_id_owned.clone()))
            .await?;

        let count: Option<u64> = count_result.take("count")?;
        let total_items = count.unwrap_or(0);

        // Get paginated data
        let mut result = self
            .db
            .client
            .query("SELECT * FROM health_checks WHERE doctor_id = $doctor_id ORDER BY scheduled_at DESC LIMIT $limit START $offset")
            .bind(("doctor_id", doctor_id_owned))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        let data: Vec<HealthCheck> = result.take(0)?;

        Ok(PaginatedResponse::new(data, params, total_items))
    }
}

#[async_trait]
impl Repository<HealthCheck> for CheckRepository {
    async fn create(&self, check: &HealthCheck) -> Result<HealthCheck> {
        let created: Option<HealthCheck> = self.db.client
            .create(TABLE)
            .content(check.clone())
            .await?;

        created.ok_or_else(|| DbError::Other("Failed to create health check".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<HealthCheck>> {
        Ok(self.db.client.select((TABLE, id)).await?)
    }

    async fn find_all(&self) -> Result<Vec<HealthCheck>> {
        Ok(self.db.client.select(TABLE).await?)
    }

    async fn update(&self, check: &HealthCheck) -> Result<HealthCheck> {
        let id = check.id.as_ref()
            .ok_or_else(|| DbError::Other("Health check ID required for update".to_string()))?;

        let updated: Option<HealthCheck> = self.db.client
            .update((TABLE, id.as_str()))
            .content(check.clone())
            .await?;

        updated.ok_or_else(|| {
            DbError::NotFound(format!("Health check {} not found", id))
        })
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let deleted: Option<HealthCheck> = self.db.client.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }

    async fn find_paginated(&self, params: &PaginationParams) -> Result<PaginatedResponse<HealthCheck>> {
        let offset = params.offset();
        let limit = params.limit();

        // Get total count
        let mut count_result = self
            .db
            .client
            .query("SELECT count() FROM health_checks GROUP ALL")
            .await?;

        let count: Option<u64> = count_result.take("count")?;
        let total_items = count.unwrap_or(0);

        // Get paginated data
        let mut result = self
            .db
            .client
            .query("SELECT * FROM health_checks ORDER BY scheduled_at DESC LIMIT $limit START $offset")
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        let data: Vec<HealthCheck> = result.take(0)?;

        Ok(PaginatedResponse::new(data, params, total_items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_create_and_find_check() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = CheckRepository::new(db);

        let pet_id = "pets:test123".to_string();
        let doctor_id = "doctors:test456".to_string();
        let scheduled = Utc::now() + Duration::hours(2);

        let check = HealthCheck::new(pet_id.clone(), doctor_id, scheduled).unwrap();
        let created = repo.create(&check).await.unwrap();

        assert_eq!(created.pet_id, pet_id);
        assert_eq!(created.status, CheckStatus::Scheduled);
        assert!(created.id.is_some());

        let id = created.id.as_ref().unwrap();
        let found = repo.find_by_id(id).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_find_by_pet() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = CheckRepository::new(db);

        let pet_id = "pets:test789".to_string();
        let doctor_id = "doctors:test101".to_string();

        let check1 = HealthCheck::new(pet_id.clone(), doctor_id.clone(), Utc::now() + Duration::hours(1)).unwrap();
        let check2 = HealthCheck::new(pet_id.clone(), doctor_id, Utc::now() + Duration::hours(2)).unwrap();

        repo.create(&check1).await.unwrap();
        repo.create(&check2).await.unwrap();

        let checks = repo.find_by_pet(&pet_id).await.unwrap();
        assert_eq!(checks.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_status() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = CheckRepository::new(db);

        let pet_id = "pets:test202".to_string();
        let doctor_id = "doctors:test303".to_string();

        let check = HealthCheck::new(pet_id, doctor_id, Utc::now() + Duration::hours(3)).unwrap();
        repo.create(&check).await.unwrap();

        let scheduled = repo.find_by_status(CheckStatus::Scheduled).await.unwrap();
        assert_eq!(scheduled.len(), 1);
    }
}
