use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Doctor,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::User => "user",
            UserRole::Doctor => "doctor",
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "user" => Ok(UserRole::User),
            "doctor" => Ok(UserRole::Doctor),
            _ => Err(CoreError::ValidationError(format!("Invalid role: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct AuthUser {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::serde_helpers::serialize",
        deserialize_with = "crate::serde_helpers::deserialize"
    )]
    pub id: Option<String>,
    pub email: String,
    pub password_hash: String,  // Note: Never send this in API responses!
    pub role: UserRole,
    pub reference_id: String, // Links to users or doctors table
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

impl AuthUser {
    pub fn new(email: String, password_hash: String, role: UserRole, reference_id: String) -> Result<Self> {
        Self::validate_email(&email)?;

        if password_hash.is_empty() {
            return Err(CoreError::ValidationError("Password hash cannot be empty".to_string()));
        }

        let now = Utc::now();
        Ok(Self {
            id: None,
            email,
            password_hash,
            role,
            reference_id,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_email(email: &str) -> Result<()> {
        if !email.contains('@') || email.len() < 5 {
            return Err(CoreError::InvalidEmail(email.to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterDoctorRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub phone: String,
    pub specialization: String,
    pub license_number: String,
    pub years_experience: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub token: AuthToken,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub role: UserRole,
    pub reference_id: String,
}

impl From<AuthUser> for UserInfo {
    fn from(auth_user: AuthUser) -> Self {
        Self {
            id: auth_user.id.unwrap_or_default(),
            email: auth_user.email,
            role: auth_user.role,
            reference_id: auth_user.reference_id,
        }
    }
}

// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // User ID
    pub email: String,
    pub role: String,
    pub ref_id: String,    // Reference ID
    pub exp: usize,        // Expiration time
    pub iat: usize,        // Issued at
}

impl Claims {
    pub fn new(user: &AuthUser, expiration: i64) -> Self {
        let now = Utc::now().timestamp() as usize;
        Self {
            sub: user.id.clone().unwrap_or_default(),
            email: user.email.clone(),
            role: user.role.as_str().to_string(),
            ref_id: user.reference_id.clone(),
            exp: (now as i64 + expiration) as usize,
            iat: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_as_str() {
        assert_eq!(UserRole::User.as_str(), "user");
        assert_eq!(UserRole::Doctor.as_str(), "doctor");
    }

    #[test]
    fn test_user_role_from_str() {
        use std::str::FromStr;
        assert_eq!(UserRole::from_str("user").unwrap(), UserRole::User);
        assert_eq!(UserRole::from_str("doctor").unwrap(), UserRole::Doctor);
        assert_eq!(UserRole::from_str("USER").unwrap(), UserRole::User);
        assert!(UserRole::from_str("admin").is_err());
    }

    #[test]
    fn test_create_auth_user() {
        let auth_user = AuthUser::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::User,
            "users:123".to_string(),
        )
        .unwrap();

        assert_eq!(auth_user.email, "test@example.com");
        assert_eq!(auth_user.role, UserRole::User);
        assert_eq!(auth_user.reference_id, "users:123");
    }

    #[test]
    fn test_invalid_email() {
        let result = AuthUser::new(
            "invalid-email".to_string(),
            "hashed_password".to_string(),
            UserRole::User,
            "users:123".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_password_hash() {
        let result = AuthUser::new(
            "test@example.com".to_string(),
            "".to_string(),
            UserRole::User,
            "users:123".to_string(),
        );
        assert!(result.is_err());
    }
}
