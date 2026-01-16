use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct User {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::serde_helpers::serialize",
        deserialize_with = "crate::serde_helpers::deserialize"
    )]
    pub id: Option<String>,
    pub email: String,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
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

impl User {
    pub fn new(email: String, name: String) -> Result<Self> {
        Self::validate_email(&email)?;
        Self::validate_name(&name)?;

        let now = Utc::now();
        Ok(Self {
            id: None,
            email,
            name,
            phone: None,
            address: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_phone(mut self, phone: String) -> Result<Self> {
        Self::validate_phone(&phone)?;
        self.phone = Some(phone);
        Ok(self)
    }

    pub fn with_address(mut self, address: String) -> Self {
        self.address = Some(address);
        self
    }

    fn validate_email(email: &str) -> Result<()> {
        if !email.contains('@') || email.len() < 5 {
            return Err(CoreError::InvalidEmail(email.to_string()));
        }
        Ok(())
    }

    fn validate_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(CoreError::ValidationError("Name cannot be empty".to_string()));
        }
        Ok(())
    }

    fn validate_phone(phone: &str) -> Result<()> {
        if phone.trim().is_empty() {
            return Err(CoreError::InvalidPhone("Phone cannot be empty".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let user = User::new("test@example.com".to_string(), "John Doe".to_string()).unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "John Doe");
        assert!(user.phone.is_none());
        assert!(user.address.is_none());
    }

    #[test]
    fn test_invalid_email() {
        let result = User::new("invalid-email".to_string(), "John Doe".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_name() {
        let result = User::new("test@example.com".to_string(), "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_user_with_phone() {
        let user = User::new("test@example.com".to_string(), "John Doe".to_string())
            .unwrap()
            .with_phone("+1234567890".to_string())
            .unwrap();
        assert_eq!(user.phone, Some("+1234567890".to_string()));
    }

    #[test]
    fn test_user_with_address() {
        let user = User::new("test@example.com".to_string(), "John Doe".to_string())
            .unwrap()
            .with_address("123 Main St".to_string());
        assert_eq!(user.address, Some("123 Main St".to_string()));
    }
}
