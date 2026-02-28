use hmac::{Hmac, Mac};
use sha2::Sha256;

type JwtClaims = Vec<(String, String)>;
type VerifyResult = Option<(String, Option<u64>, JwtClaims)>;

pub use crate::types::{
    AuthConfig, AuthError, AuthToken, Credentials, TokenType, ValidateRequest, ValidateResponse,
};

mod types {
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
}

type HmacSha256 = Hmac<Sha256>;

/// Encode bytes as URL-safe base64 without padding.
fn base64url_encode(data: &[u8]) -> String {
    let encoded = base64_encode(data);
    encoded.replace('+', "-").replace('/', "_").trim_end_matches('=').to_string()
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        result.push(CHARS[b0 >> 2] as char);
        result.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((b1 & 15) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[b2 & 63] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64url_decode(s: &str) -> Option<Vec<u8>> {
    let padded = {
        let mut p = s.replace('-', "+").replace('_', "/");
        while !p.len().is_multiple_of(4) {
            p.push('=');
        }
        p
    };
    base64_decode(&padded)
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let s = s.trim_end_matches('=');
    let mut result = Vec::new();
    let chars: Vec<u8> = s.bytes().collect();
    for chunk in chars.chunks(4) {
        let decode = |c: u8| -> Option<u8> { CHARS.iter().position(|&x| x == c).map(|i| i as u8) };
        let b0 = decode(chunk[0])?;
        let b1 = decode(chunk[1])?;
        result.push((b0 << 2) | (b1 >> 4));
        if chunk.len() > 2 {
            let b2 = decode(chunk[2])?;
            result.push(((b1 & 15) << 4) | (b2 >> 2));
            if chunk.len() > 3 {
                let b3 = decode(chunk[3])?;
                result.push(((b2 & 3) << 6) | b3);
            }
        }
    }
    Some(result)
}

/// Build a minimal HS256 JWT: `{"alg":"HS256","typ":"JWT"}.<payload>.<sig>`
fn build_jwt(subject: &str, claims: &[(String, String)], expires_at_ms: Option<u64>, secret: &str) -> String {
    let header = base64url_encode(br#"{"alg":"HS256","typ":"JWT"}"#);

    let mut payload_obj = format!(r#"{{"sub":"{}""#, subject);
    if let Some(exp) = expires_at_ms {
        payload_obj.push_str(&format!(r#","exp":{}"#, exp / 1000));
    }
    for (k, v) in claims {
        payload_obj.push_str(&format!(r#","{}":"{}""#, k, v));
    }
    payload_obj.push('}');
    let payload = base64url_encode(payload_obj.as_bytes());

    let signing_input = format!("{}.{}", header, payload);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(signing_input.as_bytes());
    let sig = base64url_encode(&mac.finalize().into_bytes());

    format!("{}.{}.{}", header, payload, sig)
}

fn verify_jwt(token: &str, secret: &str) -> VerifyResult {
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        return None;
    }
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(signing_input.as_bytes());
    let expected = base64url_encode(&mac.finalize().into_bytes());
    if expected != parts[2] {
        return None;
    }
    let payload_bytes = base64url_decode(parts[1])?;
    let payload_str = String::from_utf8(payload_bytes).ok()?;
    // Minimal JSON parsing: extract "sub" field
    let sub = extract_json_str(&payload_str, "sub")?;
    let exp = extract_json_u64(&payload_str, "exp");
    Some((sub, exp.map(|e| e * 1000), vec![]))
}

fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let search = format!(r#""{}":""#, key);
    let start = json.find(&search)? + search.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_json_u64(json: &str, key: &str) -> Option<u64> {
    let search = format!(r#""{}":"#, key);
    let start = json.find(&search)? + search.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

pub fn authenticate(creds: Credentials, config: &AuthConfig) -> Result<AuthToken, AuthError> {
    let secret = config.jwt_secret.as_deref().ok_or(AuthError::InvalidConfig)?;
    match creds {
        Credentials::UsernamePassword { username, password } => {
            if username.is_empty() || password.is_empty() {
                return Err(AuthError::InvalidCredentials);
            }
            let claims = vec![("sub".to_string(), username.clone())];
            let expires_at_ms = config.session_ttl_ms;
            let token = build_jwt(&username, &claims, expires_at_ms, secret);
            Ok(AuthToken {
                token,
                token_type: TokenType::Jwt,
                expires_at_ms,
                subject: username,
                claims,
            })
        }
        Credentials::BearerToken(token) => {
            match verify_jwt(&token, secret) {
                Some((sub, exp, claims)) => Ok(AuthToken {
                    token,
                    token_type: TokenType::Jwt,
                    expires_at_ms: exp,
                    subject: sub,
                    claims,
                }),
                None => Err(AuthError::InvalidToken),
            }
        }
        _ => Err(AuthError::InvalidCredentials),
    }
}

pub fn validate(req: ValidateRequest, config: &AuthConfig) -> Result<ValidateResponse, AuthError> {
    let secret = config.jwt_secret.as_deref().ok_or(AuthError::InvalidConfig)?;
    match verify_jwt(&req.token, secret) {
        Some((sub, exp, claims)) => Ok(ValidateResponse {
            valid: true,
            subject: sub,
            claims,
            expires_at_ms: exp,
        }),
        None => Err(AuthError::InvalidToken),
    }
}

pub fn refresh(token: &str, config: &AuthConfig) -> Result<AuthToken, AuthError> {
    let secret = config.jwt_secret.as_deref().ok_or(AuthError::InvalidConfig)?;
    let (sub, _, claims) = verify_jwt(token, secret).ok_or(AuthError::InvalidToken)?;
    let expires_at_ms = config.session_ttl_ms;
    let new_token = build_jwt(&sub, &claims, expires_at_ms, secret);
    Ok(AuthToken {
        token: new_token,
        token_type: TokenType::Jwt,
        expires_at_ms,
        subject: sub,
        claims,
    })
}

pub fn revoke(_token: &str) -> Result<(), AuthError> {
    // JWT revocation requires a blocklist; delegated to keyvalue in the WIT wrapper
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> AuthConfig {
        AuthConfig {
            token_type: TokenType::Jwt,
            session_ttl_ms: Some(3_600_000),
            jwt_secret: Some("test-secret-key".to_string()),
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
        assert_eq!(token.subject, "alice");

        let resp = validate(
            ValidateRequest { token: token.token, required_claims: vec![] },
            &config,
        )
        .unwrap();
        assert!(resp.valid);
        assert_eq!(resp.subject, "alice");
    }

    #[test]
    fn test_invalid_secret_fails_validation() {
        let config = default_config();
        let creds = Credentials::UsernamePassword {
            username: "bob".to_string(),
            password: "pw".to_string(),
        };
        let token = authenticate(creds, &config).unwrap();

        let bad_config = AuthConfig {
            jwt_secret: Some("wrong-secret".to_string()),
            ..config
        };
        let result = validate(
            ValidateRequest { token: token.token, required_claims: vec![] },
            &bad_config,
        );
        assert!(result.is_err());
    }
}
