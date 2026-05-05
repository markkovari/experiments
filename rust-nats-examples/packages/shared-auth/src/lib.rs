use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub struct JwtUser {
    pub user_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for JwtUser
where
    S: Send + Sync + HasJwtSecret,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid Authorization header".to_string()));
        }

        let token = &auth_header[7..];
        let jwt_secret = state.jwt_secret();

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

        Ok(JwtUser {
            user_id: token_data.claims.sub,
        })
    }
}

pub trait HasJwtSecret {
    fn jwt_secret(&self) -> &str;
}
