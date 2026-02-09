use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::user::Role;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    role: Role,
    exp: usize,
}

pub struct AuthUser {
    pub user_id: Uuid,
    pub role: Role,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }
}

pub fn encode_jwt(secret: &str, user_id: Uuid, role: Role) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        role,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("JWT encoding should not fail")
}

pub fn decode_jwt(secret: &str, token: &str) -> Result<(Uuid, Role), AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(e.to_string()))?;
    Ok((data.claims.sub, data.claims.role))
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing authorization header".into()))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization header".into()))?;

        let (user_id, role) = decode_jwt(&state.jwt_secret, token)?;
        Ok(AuthUser { user_id, role })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::user::Role;

    #[test]
    fn jwt_round_trip() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let token = encode_jwt(secret, user_id, Role::User);
        let (decoded_id, decoded_role) = decode_jwt(secret, &token).unwrap();
        assert_eq!(decoded_id, user_id);
        assert_eq!(decoded_role, Role::User);
    }

    #[test]
    fn jwt_round_trip_admin() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let token = encode_jwt(secret, user_id, Role::Admin);
        let (decoded_id, decoded_role) = decode_jwt(secret, &token).unwrap();
        assert_eq!(decoded_id, user_id);
        assert_eq!(decoded_role, Role::Admin);
    }

    #[test]
    fn invalid_token_rejected() {
        let result = decode_jwt("secret", "not-a-real-token");
        assert!(result.is_err());
    }

    #[test]
    fn wrong_secret_rejected() {
        let user_id = Uuid::new_v4();
        let token = encode_jwt("secret-a", user_id, Role::User);
        let result = decode_jwt("secret-b", &token);
        assert!(result.is_err());
    }
}
