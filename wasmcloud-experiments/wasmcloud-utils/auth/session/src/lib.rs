// WIT-based session authentication component.
// Targets the `auth-component` world defined in wit/wasmcloud-auth/auth.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "auth-component",
    path: "../../wit/wasmcloud-auth",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use auth_session_core::refresh;
use auth_session_core::{
    authenticate, revoke, validate, AuthConfig, AuthError as CoreError,
    Credentials as CoreCreds, TokenType as CoreTokenType, ValidateRequest,
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
fn wit_token(t: auth_session_core::AuthToken) -> wasmcloud::auth::types::AuthToken {
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
        token_type: CoreTokenType::Session,
        session_ttl_ms: Some(3_600_000),
        jwt_secret: None,
        oauth_token_url: None,
        oauth_client_id: None,
        oauth_client_secret: None,
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct AuthSessionComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::auth::authenticator::Guest for AuthSessionComponent {
    fn init(
        _config: wasmcloud::auth::types::AuthConfig,
    ) -> Result<(), wasmcloud::auth::types::AuthError> {
        Ok(())
    }

    fn authenticate(
        creds: wasmcloud::auth::types::Credentials,
    ) -> Result<wasmcloud::auth::types::AuthToken, wasmcloud::auth::types::AuthError> {
        let config = default_config();
        authenticate(core_creds(creds), &config)
            .map(wit_token)
            .map_err(core_error)
    }

    fn validate(
        req: wasmcloud::auth::types::ValidateRequest,
    ) -> Result<wasmcloud::auth::types::ValidateResponse, wasmcloud::auth::types::AuthError> {
        let config = default_config();
        validate(
            ValidateRequest { token: req.token, required_claims: req.required_claims },
            &config,
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
        token: String,
    ) -> Result<wasmcloud::auth::types::AuthToken, wasmcloud::auth::types::AuthError> {
        let config = default_config();
        refresh(&token, &config).map(wit_token).map_err(core_error)
    }

    fn revoke(token: String) -> Result<(), wasmcloud::auth::types::AuthError> {
        revoke(&token).map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(AuthSessionComponent);

// ---- native helpers (for cargo check / tests) -------------------------------

pub fn session_authenticate(username: &str, password: &str) -> Result<String, CoreError> {
    let creds = CoreCreds::UsernamePassword {
        username: username.to_string(),
        password: password.to_string(),
    };
    authenticate(creds, &default_config()).map(|t| t.token)
}

pub fn session_validate(token: &str) -> Result<String, CoreError> {
    validate(
        ValidateRequest { token: token.to_string(), required_claims: vec![] },
        &default_config(),
    )
    .map(|r| r.subject)
}

pub fn session_revoke(token: &str) -> Result<(), CoreError> {
    revoke(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let token = session_authenticate("alice", "pw").unwrap();
        let subject = session_validate(&token).unwrap();
        assert_eq!(subject, "alice");
        session_revoke(&token).unwrap();
        assert!(session_validate(&token).is_err());
    }
}
