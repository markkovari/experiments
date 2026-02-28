wit_bindgen::generate!({
    world: "rate-limiter-component",
    path: "../../wit",
});

use std::cell::RefCell;
use std::collections::HashMap;

use exports::wasmcloud::ratelimit::rate_limiter::Guest;
use exports::wasmcloud::ratelimit::rate_limiter::{
    RateLimitConfig, RateLimitError, RateLimitRequest, RateLimitResponse,
};

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
        let tokens_to_add = (elapsed_ms * self.refill_rate) / 1000; // refill_rate per second

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

        // Calculate time needed to refill the deficit
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

struct TokenBucketRateLimiter;

impl Guest for TokenBucketRateLimiter {
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

            let bucket = s
                .buckets
                .entry(request.user_id.clone())
                .or_insert_with(|| TokenBucket::new(capacity, refill_rate));

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

export!(TokenBucketRateLimiter);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_basic() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 1,
            window_size_ms: 0,
        };

        assert!(TokenBucketRateLimiter::init(config).is_ok());

        let request = RateLimitRequest {
            user_id: "user1".to_string(),
            tokens_requested: 5,
            timestamp_ms: 1000,
        };

        let response = TokenBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);
        assert_eq!(response.tokens_remaining, 5);
    }

    #[test]
    fn test_token_bucket_refill() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 2,
            window_size_ms: 0,
        };

        TokenBucketRateLimiter::init(config).unwrap();

        // Consume all tokens
        let request = RateLimitRequest {
            user_id: "user2".to_string(),
            tokens_requested: 10,
            timestamp_ms: 1000,
        };
        let response = TokenBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);

        // Try to consume more - should fail
        let request = RateLimitRequest {
            user_id: "user2".to_string(),
            tokens_requested: 5,
            timestamp_ms: 1500,
        };
        let response = TokenBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(!response.allowed);

        // Wait for refill (2 tokens/sec * 3 seconds = 6 tokens)
        let request = RateLimitRequest {
            user_id: "user2".to_string(),
            tokens_requested: 5,
            timestamp_ms: 4000,
        };
        let response = TokenBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);
    }
}
