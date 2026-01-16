use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::error::{CoreError, Result};
use crate::models::{AuthUser, Claims};

/// Get JWT secret from environment variable
fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            eprintln!("WARNING: JWT_SECRET not set, using default (INSECURE for production)");
            "default-insecure-secret-key-change-this".to_string()
        })
        .into_bytes()
}

/// Get token expiration from environment variable
fn get_jwt_expiration() -> i64 {
    std::env::var("JWT_EXPIRATION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3600) // Default: 1 hour
}

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String> {
    if password.len() < 8 {
        return Err(CoreError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CoreError::ValidationError(format!("Failed to hash password: {}", e)))?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| CoreError::ValidationError(format!("Invalid password hash: {}", e)))?;

    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a JWT access token
pub fn generate_token(user: &AuthUser) -> Result<String> {
    let expiration = get_jwt_expiration();
    let claims = Claims::new(user, expiration);
    let secret = get_jwt_secret();

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|e| CoreError::ValidationError(format!("Failed to generate token: {}", e)))
}

/// Verify and decode a JWT token
pub fn verify_token(token: &str) -> Result<Claims> {
    let secret = get_jwt_secret();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret),
        &Validation::default(),
    )
    .map_err(|e| CoreError::AuthError(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

/// Get token expiration duration in seconds (for API responses)
pub fn token_expiration_seconds() -> i64 {
    get_jwt_expiration()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserRole;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(!hash.is_empty());
        assert_ne!(hash, password);
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_too_short() {
        let password = "short";
        let result = hash_password(password);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "test_password_123";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password(wrong_password, &hash).unwrap());
    }

    #[test]
    fn test_generate_and_verify_token() {
        let auth_user = AuthUser::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::User,
            "users:123".to_string(),
        )
        .unwrap();

        // Set ID for token generation
        let mut user_with_id = auth_user;
        user_with_id.id = Some("auth_users:abc".to_string());

        let token = generate_token(&user_with_id).unwrap();
        assert!(!token.is_empty());

        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
        assert_eq!(claims.ref_id, "users:123");
    }

    #[test]
    fn test_verify_invalid_token() {
        let invalid_token = "invalid.token.here";
        let result = verify_token(invalid_token);
        assert!(result.is_err());
    }
}
