// HTTP middleware facade for the auth package.
// Targets the `auth-http` world defined in wit/auth.wit.
//
// At runtime an authenticator backend (session / jwt / oauth) is linked via
// WADM.  This component imports that backend and exports an HTTP handler that
// validates the Authorization or Cookie header before forwarding.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "auth-http",
    path: "../../wit/wasmcloud-auth",
    generate_all,
});

/// Extract the bearer token from an `Authorization: Bearer <token>` header
/// value, or return the raw value if it doesn't start with "Bearer ".
pub fn extract_bearer(header_value: &str) -> Option<&str> {
    let v = header_value.trim();
    if v.to_ascii_lowercase().starts_with("bearer ") {
        Some(v["bearer ".len()..].trim())
    } else if !v.is_empty() {
        Some(v)
    } else {
        None
    }
}

/// Extract a session token from a `Cookie: session=<token>` header value.
pub fn extract_session_cookie(cookie_header: &str) -> Option<&str> {
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("session=") {
            return Some(val.trim());
        }
    }
    None
}

/// Determine the auth token from request headers (Authorization takes priority).
/// Returns `None` when no recognisable token is present.
pub fn token_from_headers<'a>(
    headers: &'a [(&'a str, &'a str)],
) -> Option<&'a str> {
    let mut cookie_token: Option<&str> = None;
    for (name, value) in headers {
        let lc = name.to_ascii_lowercase();
        if lc == "authorization" {
            if let Some(t) = extract_bearer(value) {
                return Some(t);
            }
        } else if lc == "cookie" && cookie_token.is_none() {
            cookie_token = extract_session_cookie(value);
        }
    }
    cookie_token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_extracted() {
        assert_eq!(extract_bearer("Bearer mytoken"), Some("mytoken"));
        assert_eq!(extract_bearer("bearer  spaced "), Some("spaced"));
        assert_eq!(extract_bearer(""), None);
    }

    #[test]
    fn cookie_extracted() {
        assert_eq!(extract_session_cookie("session=abc123"), Some("abc123"));
        assert_eq!(extract_session_cookie("other=x; session=tok; foo=bar"), Some("tok"));
        assert_eq!(extract_session_cookie("no-session=1"), None);
    }

    #[test]
    fn auth_header_takes_priority() {
        let headers = [
            ("authorization", "Bearer jwt_token"),
            ("cookie", "session=sess_token"),
        ];
        assert_eq!(token_from_headers(&headers), Some("jwt_token"));
    }

    #[test]
    fn fallback_to_cookie() {
        let headers = [("cookie", "session=sess_token")];
        assert_eq!(token_from_headers(&headers), Some("sess_token"));
    }

    #[test]
    fn no_token_returns_none() {
        let headers: [(&str, &str); 0] = [];
        assert_eq!(token_from_headers(&headers), None);
    }
}
