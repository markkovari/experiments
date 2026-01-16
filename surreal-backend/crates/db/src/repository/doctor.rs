use async_trait::async_trait;

use surreal_core::{Doctor, Specialization};

use crate::connection::Database;
use crate::error::{DbError, Result};
use crate::repository::Repository;

const TABLE: &str = "doctors";

pub struct DoctorRepository {
    db: Database,
}

impl DoctorRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn find_available(&self) -> Result<Vec<Doctor>> {
        let mut result = self
            .db
            .client
            .query("SELECT * FROM doctors WHERE is_available = true")
            .await?;

        Ok(result.take(0)?)
    }

    pub async fn find_by_specialization(&self, spec: &Specialization) -> Result<Vec<Doctor>> {
        let spec_owned = spec.clone();
        let mut result = self
            .db
            .client
            .query("SELECT * FROM doctors WHERE specialization = $spec")
            .bind(("spec", spec_owned))
            .await?;

        Ok(result.take(0)?)
    }
}

#[async_trait]
impl Repository<Doctor> for DoctorRepository {
    async fn create(&self, doctor: &Doctor) -> Result<Doctor> {
        let created: Option<Doctor> = self.db.client
            .create(TABLE)
            .content(doctor.clone())
            .await?;

        created.ok_or_else(|| DbError::Other("Failed to create doctor".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Doctor>> {
        Ok(self.db.client.select((TABLE, id)).await?)
    }

    async fn find_all(&self) -> Result<Vec<Doctor>> {
        Ok(self.db.client.select(TABLE).await?)
    }

    async fn update(&self, doctor: &Doctor) -> Result<Doctor> {
        let id = doctor.id.as_ref()
            .ok_or_else(|| DbError::Other("Doctor ID required for update".to_string()))?;

        let updated: Option<Doctor> = self.db.client
            .update((TABLE, id.as_str()))
            .content(doctor.clone())
            .await?;

        updated.ok_or_else(|| {
            DbError::NotFound(format!("Doctor {} not found", id))
        })
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let deleted: Option<Doctor> = self.db.client.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_find_doctor() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = DoctorRepository::new(db);

        let doctor = Doctor::new(
            "Dr. Smith".to_string(),
            "smith@clinic.com".to_string(),
            "+1234567890".to_string(),
            Specialization::GeneralPractice,
            "LIC-123".to_string(),
            10,
        )
        .unwrap();

        let created = repo.create(&doctor).await.unwrap();
        assert_eq!(created.name, doctor.name);
        assert!(created.id.is_some());

        let id = created.id.as_ref().unwrap();
        let found = repo.find_by_id(id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Dr. Smith");
    }

    #[tokio::test]
    async fn test_find_available_doctors() {
        let db = Database::new_in_memory().await.unwrap();
        let repo = DoctorRepository::new(db);

        let doctor1 = Doctor::new(
            "Dr. Available".to_string(),
            "avail@clinic.com".to_string(),
            "+111111111".to_string(),
            Specialization::Surgery,
            "LIC-111".to_string(),
            5,
        )
        .unwrap();

        let mut doctor2 = Doctor::new(
            "Dr. Busy".to_string(),
            "busy@clinic.com".to_string(),
            "+222222222".to_string(),
            Specialization::Cardiology,
            "LIC-222".to_string(),
            8,
        )
        .unwrap();

        doctor2.set_availability(false);

        repo.create(&doctor1).await.unwrap();
        repo.create(&doctor2).await.unwrap();

        let available = repo.find_available().await.unwrap();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].name, "Dr. Available");
    }
}
