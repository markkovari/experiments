use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum PetSpecies {
    Dog,
    Cat,
    Bird,
    Rabbit,
    Hamster,
    Fish,
    Reptile,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Pet {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::serde_helpers::serialize",
        deserialize_with = "crate::serde_helpers::deserialize"
    )]
    pub id: Option<String>,
    pub owner_id: String,
    pub name: String,
    pub species: PetSpecies,
    pub breed: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub weight_kg: Option<f32>,
    pub medical_notes: Option<String>,
    #[serde(
        serialize_with = "crate::serde_helpers::serialize_datetime",
        deserialize_with = "crate::serde_helpers::deserialize_datetime"
    )]
    pub created_at: DateTime<Utc>,
    #[serde(
        serialize_with = "crate::serde_helpers::serialize_datetime",
        deserialize_with = "crate::serde_helpers::deserialize_datetime"
    )]
    pub updated_at: DateTime<Utc>,
}

impl Pet {
    pub fn new(owner_id: String, name: String, species: PetSpecies) -> Result<Self> {
        Self::validate_name(&name)?;

        let now = Utc::now();
        Ok(Self {
            id: None,
            owner_id,
            name,
            species,
            breed: None,
            birth_date: None,
            weight_kg: None,
            medical_notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_breed(mut self, breed: String) -> Self {
        self.breed = Some(breed);
        self
    }

    pub fn with_birth_date(mut self, birth_date: NaiveDate) -> Result<Self> {
        if birth_date > Utc::now().date_naive() {
            return Err(CoreError::InvalidDate(
                "Birth date cannot be in the future".to_string(),
            ));
        }
        self.birth_date = Some(birth_date);
        Ok(self)
    }

    pub fn with_weight(mut self, weight_kg: f32) -> Result<Self> {
        if weight_kg <= 0.0 {
            return Err(CoreError::ValidationError(
                "Weight must be positive".to_string(),
            ));
        }
        self.weight_kg = Some(weight_kg);
        Ok(self)
    }

    pub fn with_medical_notes(mut self, notes: String) -> Self {
        self.medical_notes = Some(notes);
        self
    }

    fn validate_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(CoreError::ValidationError(
                "Pet name cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    pub fn age_years(&self) -> Option<i64> {
        self.birth_date.map(|birth| {
            let today = Utc::now().date_naive();
            let age = today.years_since(birth).unwrap_or(0);
            age as i64
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pet() {
        let owner_id = "users:test123".to_string();
        let pet = Pet::new(owner_id.clone(), "Buddy".to_string(), PetSpecies::Dog).unwrap();
        assert_eq!(pet.owner_id, owner_id);
        assert_eq!(pet.name, "Buddy");
        assert_eq!(pet.species, PetSpecies::Dog);
    }

    #[test]
    fn test_invalid_pet_name() {
        let owner_id = "users:test123".to_string();
        let result = Pet::new(owner_id, "".to_string(), PetSpecies::Cat);
        assert!(result.is_err());
    }

    #[test]
    fn test_pet_with_breed() {
        let owner_id = "users:test123".to_string();
        let pet = Pet::new(owner_id, "Max".to_string(), PetSpecies::Dog)
            .unwrap()
            .with_breed("Golden Retriever".to_string());
        assert_eq!(pet.breed, Some("Golden Retriever".to_string()));
    }

    #[test]
    fn test_pet_with_weight() {
        let owner_id = "users:test123".to_string();
        let pet = Pet::new(owner_id, "Whiskers".to_string(), PetSpecies::Cat)
            .unwrap()
            .with_weight(4.5)
            .unwrap();
        assert_eq!(pet.weight_kg, Some(4.5));
    }

    #[test]
    fn test_invalid_weight() {
        let owner_id = "users:test123".to_string();
        let result = Pet::new(owner_id, "Whiskers".to_string(), PetSpecies::Cat)
            .unwrap()
            .with_weight(-1.0);
        assert!(result.is_err());
    }
}
