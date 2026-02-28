// WIT-based OAuth authentication component.
// Targets the `auth-oauth-component` world defined in wit/wasmcloud-auth/auth.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "auth-oauth-component",
    path: "../../wit/wasmcloud-auth",
    generate_all,
});

use auth_oauth_core::{
    build_token_exchange_body, parse_token_response, token_response_to_auth_token,
    AuthConfig, AuthError as CoreError, TokenType as CoreTokenType,
};

// ---- type conversions -------------------------------------------------------

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

// ---- default config ---------------------------------------------------------

#[allow(dead_code)]
fn default_config() -> AuthConfig {
    AuthConfig {
        token_type: CoreTokenType::OauthAccess,
        session_ttl_ms: None,
        jwt_secret: None,
        oauth_token_url: Some("https://provider.example.com/token".to_string()),
        oauth_client_id: Some("client_id".to_string()),
        oauth_client_secret: Some("client_secret".to_string()),
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct AuthOauthComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::auth::authenticator::Guest for AuthOauthComponent {
    fn init(
        _config: wasmcloud::auth::types::AuthConfig,
    ) -> Result<(), wasmcloud::auth::types::AuthError> {
        Ok(())
    }

    fn authenticate(
        creds: wasmcloud::auth::types::Credentials,
    ) -> Result<wasmcloud::auth::types::AuthToken, wasmcloud::auth::types::AuthError> {
        use wasmcloud::auth::types::Credentials;
        let (code, redirect_uri) = match creds {
            Credentials::OauthCode(o) => (o.code, o.redirect_uri),
            _ => return Err(wasmcloud::auth::types::AuthError::InvalidCredentials),
        };
        let config = default_config();
        let _body = build_token_exchange_body(
            &code,
            &redirect_uri,
            config.oauth_client_id.as_deref().unwrap_or(""),
            config.oauth_client_secret.as_deref().unwrap_or(""),
        );
        // In production, `_body` would be sent via wasi:http/outgoing-handler.
        // Return UpstreamError as a stub (no HTTP client in the stub).
        Err(wasmcloud::auth::types::AuthError::UpstreamError)
    }

    fn validate(
        _req: wasmcloud::auth::types::ValidateRequest,
    ) -> Result<wasmcloud::auth::types::ValidateResponse, wasmcloud::auth::types::AuthError> {
        Err(wasmcloud::auth::types::AuthError::InvalidToken)
    }

    fn refresh(
        _token: String,
    ) -> Result<wasmcloud::auth::types::AuthToken, wasmcloud::auth::types::AuthError> {
        Err(wasmcloud::auth::types::AuthError::InvalidToken)
    }

    fn revoke(_token: String) -> Result<(), wasmcloud::auth::types::AuthError> {
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
export!(AuthOauthComponent);

// ---- native helpers (for cargo check / tests) -------------------------------

pub fn prepare_token_exchange(
    code: &str,
    redirect_uri: &str,
    client_id: &str,
    client_secret: &str,
) -> String {
    build_token_exchange_body(code, redirect_uri, client_id, client_secret)
}

pub fn handle_token_response(json: &str) -> Result<String, CoreError> {
    let resp = parse_token_response(json)?;
    let token = token_response_to_auth_token(resp);
    Ok(token.token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exchange_body_is_correct() {
        let body = prepare_token_exchange("code123", "https://app.example/cb", "cid", "csec");
        assert!(body.contains("grant_type=authorization_code"));
        assert!(body.contains("code=code123"));
    }

    #[test]
    fn parse_valid_response() {
        let json =
            r#"{"access_token":"at_abc","token_type":"Bearer","expires_in":7200,"sub":"user42"}"#;
        let token = handle_token_response(json).unwrap();
        assert_eq!(token, "at_abc");
    }
}
