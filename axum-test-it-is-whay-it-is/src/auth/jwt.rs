use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};

const JWT_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub email: String,
    pub name: String,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, name: String) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(JWT_EXPIRATION_HOURS);

        Self {
            sub: user_id.to_string(),
            email,
            name,
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        }
    }

    pub fn to_auth_user(&self) -> AppResult<AuthUser> {
        Ok(AuthUser {
            id: Uuid::parse_str(&self.sub)
                .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?,
            email: self.email.clone(),
            name: self.name.clone(),
        })
    }
}

/// Generate a JWT token for a user
pub fn generate_token(user_id: Uuid, email: String, name: String, secret: &str) -> AppResult<String> {
    let claims = Claims::new(user_id, email, name);

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::Jwt)
}

/// Verify and decode a JWT token
pub fn verify_token(token: &str, secret: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(AppError::Jwt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_token() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let name = "Test User".to_string();

        let token = generate_token(user_id, email.clone(), name.clone(), secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.name, name);
    }

    #[test]
    fn test_verify_invalid_token() {
        let secret = "test_secret_key_at_least_32_characters_long";
        let result = verify_token("invalid.token.here", secret);

        assert!(result.is_err());
    }
}
