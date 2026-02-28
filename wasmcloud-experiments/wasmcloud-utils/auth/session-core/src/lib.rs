use std::collections::HashMap;

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

/// Generate a pseudo-random session ID as hex-encoded bytes.
/// Uses a simple LCG seeded from a counter — suitable for non-security-critical IDs
/// in a WASM environment without access to OS entropy.
fn generate_session_id(seed: u64) -> String {
    let mut state = seed.wrapping_add(0x9e3779b97f4a7c15);
    let mut bytes = [0u8; 16];
    for chunk in bytes.chunks_mut(8) {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let b = state.to_le_bytes();
        for (d, s) in chunk.iter_mut().zip(b.iter()) {
            *d = *s;
        }
    }
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

use std::cell::RefCell;

// NOTE: This in-memory store is a stub for native `cargo test` only.
// In a deployed WASM component, session state lives in wasi:keyvalue
// (imported in the auth-component WIT world). There are no OS threads
// in the component model; thread_local! compiles to a plain static in
// wasm32 targets and is used here solely for Rust borrow-checker
// compatibility on native test targets.
type SessionEntry = (String, Option<u64>, Vec<(String, String)>);

/// In-memory session store (keyed by session token → (subject, expires_at_ms, claims)).
struct SessionState {
    store: HashMap<String, SessionEntry>,
    counter: u64,
}

thread_local! {
    static SESSION: RefCell<SessionState> = RefCell::new(SessionState {
        store: HashMap::new(),
        counter: 0,
    });
}

/// Run a closure with exclusive access to the session state.
fn with_session<R>(f: impl FnOnce(&mut SessionState) -> R) -> R {
    SESSION.with(|s| f(&mut s.borrow_mut()))
}

pub fn authenticate(creds: Credentials, config: &AuthConfig) -> Result<AuthToken, AuthError> {
    match creds {
        Credentials::UsernamePassword { username, password } => {
            if username.is_empty() || password.is_empty() {
                return Err(AuthError::InvalidCredentials);
            }
            with_session(|s| {
                s.counter = s.counter.wrapping_add(1);
                let token = generate_session_id(s.counter);
                let expires_at_ms = config.session_ttl_ms;
                let claims = vec![("sub".to_string(), username.clone())];
                s.store.insert(token.clone(), (username.clone(), expires_at_ms, claims.clone()));
                Ok(AuthToken {
                    token,
                    token_type: TokenType::Session,
                    expires_at_ms,
                    subject: username,
                    claims,
                })
            })
        }
        _ => Err(AuthError::InvalidCredentials),
    }
}

pub fn validate(req: ValidateRequest, _config: &AuthConfig) -> Result<ValidateResponse, AuthError> {
    with_session(|s| match s.store.get(&req.token) {
        Some((subject, expires_at_ms, claims)) => Ok(ValidateResponse {
            valid: true,
            subject: subject.clone(),
            claims: claims.clone(),
            expires_at_ms: *expires_at_ms,
        }),
        None => Err(AuthError::InvalidToken),
    })
}

pub fn refresh(token: &str, config: &AuthConfig) -> Result<AuthToken, AuthError> {
    with_session(|s| {
        let entry = s.store.get(token).cloned().ok_or(AuthError::InvalidToken)?;
        let (subject, _, claims) = entry;
        s.counter = s.counter.wrapping_add(1);
        let new_token = generate_session_id(s.counter);
        let expires_at_ms = config.session_ttl_ms;
        s.store.remove(token);
        s.store.insert(new_token.clone(), (subject.clone(), expires_at_ms, claims.clone()));
        Ok(AuthToken { token: new_token, token_type: TokenType::Session, expires_at_ms, subject, claims })
    })
}

pub fn revoke(token: &str) -> Result<(), AuthError> {
    with_session(|s| { s.store.remove(token); Ok(()) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> AuthConfig {
        AuthConfig {
            token_type: TokenType::Session,
            session_ttl_ms: Some(3_600_000),
            jwt_secret: None,
            oauth_token_url: None,
            oauth_client_id: None,
            oauth_client_secret: None,
        }
    }

    #[test]
    fn test_authenticate_and_validate() {
        let config = default_config();
        let creds = Credentials::UsernamePassword {
            username: "alice".to_string(),
            password: "secret".to_string(),
        };
        let token = authenticate(creds, &config).unwrap();
        assert!(!token.token.is_empty());

        let resp = validate(
            ValidateRequest { token: token.token.clone(), required_claims: vec![] },
            &config,
        )
        .unwrap();
        assert!(resp.valid);
        assert_eq!(resp.subject, "alice");
    }

    #[test]
    fn test_revoke() {
        let config = default_config();
        let creds = Credentials::UsernamePassword {
            username: "bob".to_string(),
            password: "pw".to_string(),
        };
        let token = authenticate(creds, &config).unwrap();
        revoke(&token.token).unwrap();
        let result = validate(
            ValidateRequest { token: token.token, required_claims: vec![] },
            &config,
        );
        assert!(result.is_err());
    }
}
