// Pattern 2: Import Rate Limiter as Library
// This component IMPORTS the rate limiter and uses it directly

wit_bindgen::generate!({
    world: "consumer-with-ratelimit",
    path: "../../wit",
    generate_all,
});

use exports::wasi::http::incoming_handler::Guest;
use wasi::http::types::{Request, Response};
use wasmcloud::ratelimit::rate_limiter::{RateLimitConfig, RateLimitRequest};

struct ConsumerApp;

impl Guest for ConsumerApp {
    fn handle(request: Request) -> Response {
        // Extract user ID from request
        let user_id = extract_user_id(&request);

        // Initialize rate limiter (in production, do this once at startup)
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 1,
            window_size_ms: 0,
        };

        if let Err(e) = wasmcloud::ratelimit::rate_limiter::init(config) {
            return error_response(&format!("Failed to init rate limiter: {:?}", e));
        }

        // Check rate limit by IMPORTING the rate limiter
        let rate_request = RateLimitRequest {
            user_id: user_id.clone(),
            tokens_requested: 1,
            timestamp_ms: get_current_time_ms(),
        };

        match wasmcloud::ratelimit::rate_limiter::check_rate_limit(&rate_request) {
            Ok(rate_response) => {
                if rate_response.allowed {
                    // Rate limit passed - handle the actual request
                    handle_business_logic(request, user_id, rate_response.tokens_remaining)
                } else {
                    // Rate limited!
                    rate_limit_response(rate_response.retry_after_ms)
                }
            }
            Err(e) => error_response(&format!("Rate limit check failed: {:?}", e)),
        }
    }
}

fn extract_user_id(request: &Request) -> String {
    for (key, value) in &request.headers {
        if key.eq_ignore_ascii_case("x-user-id") {
            if let Ok(user_id) = String::from_utf8(value.clone()) {
                return user_id;
            }
        }
    }
    "anonymous".to_string()
}

fn get_current_time_ms() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1000, Ordering::SeqCst)
}

fn handle_business_logic(request: Request, user_id: String, tokens_remaining: u64) -> Response {
    // This is where your actual application logic goes
    let body = format!(
        r#"{{"message":"Hello from consumer app!","user":"{}","path":"{}","tokens_remaining":{}}}"#,
        user_id, request.path, tokens_remaining
    );

    Response {
        status: 200,
        headers: vec![
            ("content-type".to_string(), b"application/json".to_vec()),
            (
                "x-ratelimit-remaining".to_string(),
                tokens_remaining.to_string().into_bytes(),
            ),
        ],
        body: Some(body.into_bytes()),
    }
}

fn rate_limit_response(retry_after_ms: Option<u64>) -> Response {
    let retry_secs = retry_after_ms.unwrap_or(1000) / 1000;
    let body = format!(
        r#"{{"error":"rate_limit_exceeded","retry_after_seconds":{}}}"#,
        retry_secs
    );

    Response {
        status: 429,
        headers: vec![
            ("content-type".to_string(), b"application/json".to_vec()),
            (
                "retry-after".to_string(),
                retry_secs.to_string().into_bytes(),
            ),
        ],
        body: Some(body.into_bytes()),
    }
}

fn error_response(message: &str) -> Response {
    let body = format!(r#"{{"error":"{}"}}"#, message);

    Response {
        status: 500,
        headers: vec![("content-type".to_string(), b"application/json".to_vec())],
        body: Some(body.into_bytes()),
    }
}

export!(ConsumerApp);
