use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum Specialization {
    GeneralPractice,
    Surgery,
    Dentistry,
    Dermatology,
    Cardiology,
    Neurology,
    Oncology,
    Orthopedics,
    Ophthalmology,
    Emergency,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Doctor {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::serde_helpers::serialize",
        deserialize_with = "crate::serde_helpers::deserialize"
    )]
    pub id: Option<String>,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub specialization: Specialization,
    pub license_number: String,
    pub years_experience: u32,
    pub is_available: bool,
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

impl Doctor {
    pub fn new(
        name: String,
        email: String,
        phone: String,
        specialization: Specialization,
        license_number: String,
        years_experience: u32,
    ) -> Result<Self> {
        Self::validate_name(&name)?;
        Self::validate_email(&email)?;
        Self::validate_phone(&phone)?;
        Self::validate_license(&license_number)?;

        let now = Utc::now();
        Ok(Self {
            id: None,
            name,
            email,
            phone,
            specialization,
            license_number,
            years_experience,
            is_available: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn set_availability(&mut self, available: bool) {
        self.is_available = available;
        self.updated_at = Utc::now();
    }

    fn validate_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(CoreError::ValidationError(
                "Doctor name cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<()> {
        if !email.contains('@') || email.len() < 5 {
            return Err(CoreError::InvalidEmail(email.to_string()));
        }
        Ok(())
    }

    fn validate_phone(phone: &str) -> Result<()> {
        if phone.trim().is_empty() {
            return Err(CoreError::InvalidPhone("Phone cannot be empty".to_string()));
        }
        Ok(())
    }

    fn validate_license(license: &str) -> Result<()> {
        if license.trim().is_empty() {
            return Err(CoreError::ValidationError(
                "License number cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_doctor() {
        let doctor = Doctor::new(
            "Dr. Smith".to_string(),
            "smith@clinic.com".to_string(),
            "+1234567890".to_string(),
            Specialization::GeneralPractice,
            "LIC-12345".to_string(),
            10,
        )
        .unwrap();

        assert_eq!(doctor.name, "Dr. Smith");
        assert_eq!(doctor.email, "smith@clinic.com");
        assert_eq!(doctor.years_experience, 10);
        assert!(doctor.is_available);
    }

    #[test]
    fn test_invalid_doctor_email() {
        let result = Doctor::new(
            "Dr. Smith".to_string(),
            "invalid-email".to_string(),
            "+1234567890".to_string(),
            Specialization::Surgery,
            "LIC-12345".to_string(),
            5,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_set_availability() {
        let mut doctor = Doctor::new(
            "Dr. Jones".to_string(),
            "jones@clinic.com".to_string(),
            "+9876543210".to_string(),
            Specialization::Cardiology,
            "LIC-67890".to_string(),
            15,
        )
        .unwrap();

        assert!(doctor.is_available);
        doctor.set_availability(false);
        assert!(!doctor.is_available);
    }
}
