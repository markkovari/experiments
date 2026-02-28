// WIT-based JWT authentication component.
// Targets the `auth-component` world defined in wit/wasmcloud-auth/auth.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "auth-component",
    path: "../../wit/wasmcloud-auth",
    generate_all,
});

use auth_jwt_core::{
    authenticate, validate, AuthConfig, AuthError as CoreError, Credentials as CoreCreds,
    TokenType as CoreTokenType, ValidateRequest,
};

// ---- type conversions -------------------------------------------------------

#[cfg(target_arch = "wasm32")]
fn core_creds(c: wasmcloud::auth::types::Credentials) -> CoreCreds {
    use wasmcloud::auth::types::Credentials;
    match c {
        Credentials::UsernamePassword(u) => CoreCreds::UsernamePassword {
            username: u.username,
            password: u.password,
        },
        Credentials::BearerToken(t) => CoreCreds::BearerToken(t),
        Credentials::OauthCode(o) => CoreCreds::OauthCode {
            code: o.code,
            redirect_uri: o.redirect_uri,
            state: o.state,
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::auth::types::AuthError {
    use wasmcloud::auth::types::AuthError;
    match e {
        CoreError::NotInitialized => AuthError::NotInitialized,
        CoreError::InvalidCredentials => AuthError::InvalidCredentials,
        CoreError::ExpiredToken => AuthError::ExpiredToken,
        CoreError::InvalidToken => AuthError::InvalidToken,
        CoreError::RevokedToken => AuthError::RevokedToken,
        CoreError::InsufficientPermissions => AuthError::InsufficientPermissions,
        CoreError::StorageError => AuthError::StorageError,
        CoreError::UpstreamError => AuthError::UpstreamError,
        CoreError::InvalidConfig => AuthError::InvalidConfig,
    }
}

#[cfg(target_arch = "wasm32")]
fn core_token_type(t: CoreTokenType) -> wasmcloud::auth::types::TokenType {
    use wasmcloud::auth::types::TokenType;
    match t {
        CoreTokenType::Session => TokenType::Session,
        CoreTokenType::Jwt => TokenType::Jwt,
        CoreTokenType::OauthAccess => TokenType::OauthAccess,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_token(t: auth_jwt_core::AuthToken) -> wasmcloud::auth::types::AuthToken {
    wasmcloud::auth::types::AuthToken {
        token: t.token,
        token_type: core_token_type(t.token_type),
        expires_at_ms: t.expires_at_ms,
        subject: t.subject,
        claims: t.claims,
    }
}

// ---- default config ---------------------------------------------------------

fn default_config() -> AuthConfig {
    AuthConfig {
        token_type: CoreTokenType::Jwt,
        session_ttl_ms: Some(3_600_000),
        jwt_secret: Some("default-secret-change-me".to_string()),
        oauth_token_url: None,
        oauth_client_id: None,
        oauth_client_secret: None,
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct AuthJwtComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::auth::authenticator::Guest for AuthJwtComponent {
    fn init(
        config: wasmcloud::auth::types::AuthConfig,
    ) -> Result<(), wasmcloud::auth::types::AuthError> {
        // Secret can be injected via init; for now accept and ignore.
        let _ = config;
        Ok(())
    }

    fn authenticate(
        creds: wasmcloud::auth::types::Credentials,
    ) -> Result<wasmcloud::auth::types::AuthToken, wasmcloud::auth::types::AuthError> {
        authenticate(core_creds(creds), &default_config())
            .map(wit_token)
            .map_err(core_error)
    }

    fn validate(
        req: wasmcloud::auth::types::ValidateRequest,
    ) -> Result<wasmcloud::auth::types::ValidateResponse, wasmcloud::auth::types::AuthError> {
        validate(
            ValidateRequest { token: req.token, required_claims: req.required_claims },
            &default_config(),
        )
        .map(|r| wasmcloud::auth::types::ValidateResponse {
            valid: r.valid,
            subject: r.subject,
            claims: r.claims,
            expires_at_ms: r.expires_at_ms,
        })
        .map_err(core_error)
    }

    fn refresh(
        _token: String,
    ) -> Result<wasmcloud::auth::types::AuthToken, wasmcloud::auth::types::AuthError> {
        // JWT refresh: re-issue a new token for the same subject (simplified).
        Err(wasmcloud::auth::types::AuthError::InvalidToken)
    }

    fn revoke(_token: String) -> Result<(), wasmcloud::auth::types::AuthError> {
        // JWT revocation requires a blocklist (wasi:keyvalue); stub for now.
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
export!(AuthJwtComponent);

// ---- native helpers (for cargo check / tests) -------------------------------

pub fn jwt_authenticate(username: &str, password: &str, secret: &str) -> Result<String, CoreError> {
    let creds = CoreCreds::UsernamePassword {
        username: username.to_string(),
        password: password.to_string(),
    };
    let mut config = default_config();
    config.jwt_secret = Some(secret.to_string());
    authenticate(creds, &config).map(|t| t.token)
}

pub fn jwt_validate(token: &str, secret: &str) -> Result<String, CoreError> {
    let mut config = default_config();
    config.jwt_secret = Some(secret.to_string());
    validate(
        ValidateRequest { token: token.to_string(), required_claims: vec![] },
        &config,
    )
    .map(|r| r.subject)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let token = jwt_authenticate("alice", "pw", "test-secret").unwrap();
        let subject = jwt_validate(&token, "test-secret").unwrap();
        assert_eq!(subject, "alice");
    }

    #[test]
    fn wrong_secret_rejected() {
        let token = jwt_authenticate("bob", "pw", "good-secret").unwrap();
        assert!(jwt_validate(&token, "bad-secret").is_err());
    }
}
