use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Credentials {
    UsernamePassword { username: String, password: String },
    BearerToken(String),
    OauthCode { code: String, redirect_uri: String, state: Option<String> },
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Session,
    Jwt,
    OauthAccess,
}

#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: String,
    pub token_type: TokenType,
    pub expires_at_ms: Option<u64>,
    pub subject: String,
    pub claims: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub token_type: TokenType,
    pub session_ttl_ms: Option<u64>,
    pub jwt_secret: Option<String>,
    pub oauth_token_url: Option<String>,
    pub oauth_client_id: Option<String>,
    pub oauth_client_secret: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidateRequest {
    pub token: String,
    pub required_claims: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidateResponse {
    pub valid: bool,
    pub subject: String,
    pub claims: Vec<(String, String)>,
    pub expires_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthError {
    NotInitialized,
    InvalidCredentials,
    ExpiredToken,
    InvalidToken,
    RevokedToken,
    InsufficientPermissions,
    StorageError,
    UpstreamError,
    InvalidConfig,
}

/// Serialise the token-exchange POST body (application/x-www-form-urlencoded).
pub fn build_token_exchange_body(
    code: &str,
    redirect_uri: &str,
    client_id: &str,
    client_secret: &str,
) -> String {
    format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
        url_encode(code),
        url_encode(redirect_uri),
        url_encode(client_id),
        url_encode(client_secret),
    )
}

fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
    pub sub: Option<String>,
}

/// Parse an OAuth token response JSON body.
pub fn parse_token_response(json: &str) -> Result<TokenResponse, AuthError> {
    serde_json::from_str(json).map_err(|_| AuthError::UpstreamError)
}

/// Produce an AuthToken from a parsed OAuth response.
pub fn token_response_to_auth_token(resp: TokenResponse) -> AuthToken {
    let expires_at_ms = resp.expires_in.map(|secs| secs * 1000);
    let subject = resp.sub.unwrap_or_default();
    let claims = resp.scope
        .map(|s| vec![("scope".to_string(), s)])
        .unwrap_or_default();
    AuthToken {
        token: resp.access_token,
        token_type: TokenType::OauthAccess,
        expires_at_ms,
        subject,
        claims,
    }
}

/// Validate a bearer token (opaque check — just verify it's non-empty).
/// Real validation would introspect via the OAuth server.
pub fn validate(req: ValidateRequest, _config: &AuthConfig) -> Result<ValidateResponse, AuthError> {
    if req.token.is_empty() {
        return Err(AuthError::InvalidToken);
    }
    Ok(ValidateResponse {
        valid: true,
        subject: String::new(),
        claims: vec![],
        expires_at_ms: None,
    })
}

pub fn revoke(_token: &str) -> Result<(), AuthError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_token_exchange_body() {
        let body = build_token_exchange_body("mycode", "https://example.com/cb", "client1", "secret1");
        assert!(body.contains("grant_type=authorization_code"));
        assert!(body.contains("code=mycode"));
        assert!(body.contains("client_id=client1"));
    }

    #[test]
    fn test_parse_token_response() {
        let json = r#"{"access_token":"tok123","token_type":"Bearer","expires_in":3600,"sub":"user1"}"#;
        let resp = parse_token_response(json).unwrap();
        assert_eq!(resp.access_token, "tok123");
        assert_eq!(resp.sub.unwrap(), "user1");
    }
}
