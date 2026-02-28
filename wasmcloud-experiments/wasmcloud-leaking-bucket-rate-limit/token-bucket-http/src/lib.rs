// Pattern 1: HTTP Middleware - Rate limiter as HTTP proxy
wit_bindgen::generate!({
    world: "rate-limiter-http",
    path: "../wit",
    generate_all,
});

use std::cell::RefCell;
use std::collections::HashMap;

use exports::wasmcloud::ratelimit::rate_limiter::Guest as RateLimiterGuest;
use exports::wasmcloud::ratelimit::rate_limiter::{
    RateLimitConfig, RateLimitRequest, RateLimitResponse, RateLimitError,
};
use exports::wasi::http::incoming_handler::Guest as HttpGuest;
use wasi::http::types::{Request, Response, Headers};

// Token Bucket implementation (same as before)
struct TokenBucket {
    capacity: u64,
    tokens: u64,
    refill_rate: u64,
    last_refill_ms: u64,
}

impl TokenBucket {
    fn new(capacity: u64, refill_rate: u64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill_ms: 0,
        }
    }

    fn refill(&mut self, current_time_ms: u64) {
        if self.last_refill_ms == 0 {
            self.last_refill_ms = current_time_ms;
            return;
        }

        let elapsed_ms = current_time_ms.saturating_sub(self.last_refill_ms);
        let tokens_to_add = (elapsed_ms * self.refill_rate) / 1000;

        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill_ms = current_time_ms;
    }

    fn consume(&mut self, tokens: u64) -> bool {
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn tokens_remaining(&self) -> u64 {
        self.tokens
    }

    fn retry_after_ms(&self, tokens_needed: u64) -> Option<u64> {
        if tokens_needed > self.capacity {
            return None;
        }

        let tokens_deficit = tokens_needed.saturating_sub(self.tokens);
        if tokens_deficit == 0 {
            return None;
        }

        let ms_needed = (tokens_deficit * 1000) / self.refill_rate;
        Some(ms_needed)
    }
}

struct Component {
    config: Option<RateLimitConfig>,
    buckets: HashMap<String, TokenBucket>,
}

thread_local! {
    static STATE: RefCell<Component> = RefCell::new(Component {
        config: None,
        buckets: HashMap::new(),
    });
}

struct TokenBucketHttp;

// Implement rate limiter interface
impl RateLimiterGuest for TokenBucketHttp {
    fn init(config: RateLimitConfig) -> Result<(), RateLimitError> {
        if config.capacity == 0 || config.refill_rate == 0 {
            return Err(RateLimitError::InvalidConfig);
        }

        STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.config = Some(config);
            s.buckets.clear();
        });

        Ok(())
    }

    fn check_rate_limit(request: RateLimitRequest) -> Result<RateLimitResponse, RateLimitError> {
        if request.tokens_requested == 0 {
            return Err(RateLimitError::InvalidRequest);
        }

        STATE.with(|state| {
            let mut s = state.borrow_mut();

            let config = s.config.as_ref().ok_or(RateLimitError::InvalidConfig)?;
            let capacity = config.capacity;
            let refill_rate = config.refill_rate;

            let bucket = s.buckets.entry(request.user_id.clone()).or_insert_with(|| {
                TokenBucket::new(capacity, refill_rate)
            });

            bucket.refill(request.timestamp_ms);

            let allowed = bucket.consume(request.tokens_requested);
            let tokens_remaining = bucket.tokens_remaining();
            let retry_after_ms = if !allowed {
                bucket.retry_after_ms(request.tokens_requested)
            } else {
                None
            };

            Ok(RateLimitResponse {
                allowed,
                tokens_remaining,
                retry_after_ms,
            })
        })
    }

    fn reset(user_id: String) -> Result<(), RateLimitError> {
        STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.buckets.remove(&user_id);
        });

        Ok(())
    }
}

// NEW: Implement HTTP handler for middleware pattern
impl HttpGuest for TokenBucketHttp {
    fn handle(request: Request) -> Response {
        // Extract user ID from headers (X-User-Id or Authorization)
        let user_id = extract_user_id(&request.headers);

        // Get current timestamp (milliseconds since epoch)
        let timestamp_ms = get_current_time_ms();

        // Check rate limit
        let rate_request = RateLimitRequest {
            user_id: user_id.clone(),
            tokens_requested: 1,
            timestamp_ms,
        };

        match Self::check_rate_limit(rate_request) {
            Ok(rate_response) => {
                if rate_response.allowed {
                    // Rate limit OK - forward to upstream
                    // In production, this would proxy to another component
                    create_success_response(&user_id, rate_response.tokens_remaining)
                } else {
                    // Rate limited!
                    create_rate_limit_response(rate_response.retry_after_ms)
                }
            }
            Err(_) => {
                create_error_response("Rate limiter not initialized")
            }
        }
    }
}

// Helper functions
fn extract_user_id(headers: &Headers) -> String {
    // Try X-User-Id header first
    for (key, value) in headers {
        if key.eq_ignore_ascii_case("x-user-id") {
            if let Ok(user_id) = String::from_utf8(value.clone()) {
                return user_id;
            }
        }
    }

    // Try Authorization header
    for (key, value) in headers {
        if key.eq_ignore_ascii_case("authorization") {
            if let Ok(auth) = String::from_utf8(value.clone()) {
                // Extract from "Bearer <token>" or similar
                return auth.split_whitespace().last()
                    .unwrap_or("anonymous")
                    .to_string();
            }
        }
    }

    // Default to anonymous
    "anonymous".to_string()
}

fn get_current_time_ms() -> u64 {
    // In a real implementation, this would call a WASI clock function
    // For now, use a simple counter
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1000, Ordering::SeqCst)
}

fn create_success_response(user_id: &str, tokens_remaining: u64) -> Response {
    let body = format!(
        r#"{{"status":"ok","user_id":"{}","tokens_remaining":{}}}"#,
        user_id, tokens_remaining
    );

    Response {
        status: 200,
        headers: vec![
            ("content-type".to_string(), b"application/json".to_vec()),
            ("x-ratelimit-remaining".to_string(), tokens_remaining.to_string().into_bytes()),
        ],
        body: Some(body.into_bytes()),
    }
}

fn create_rate_limit_response(retry_after_ms: Option<u64>) -> Response {
    let retry_after_secs = retry_after_ms.unwrap_or(1000) / 1000;
    let body = format!(
        r#"{{"error":"rate_limit_exceeded","retry_after_seconds":{}}}"#,
        retry_after_secs
    );

    Response {
        status: 429,
        headers: vec![
            ("content-type".to_string(), b"application/json".to_vec()),
            ("retry-after".to_string(), retry_after_secs.to_string().into_bytes()),
            ("x-ratelimit-remaining".to_string(), b"0".to_vec()),
        ],
        body: Some(body.into_bytes()),
    }
}

fn create_error_response(message: &str) -> Response {
    let body = format!(r#"{{"error":"{}"}}"#, message);

    Response {
        status: 500,
        headers: vec![
            ("content-type".to_string(), b"application/json".to_vec()),
        ],
        body: Some(body.into_bytes()),
    }
}

export!(TokenBucketHttp);
