wit_bindgen::generate!({
    world: "rate-limiter-component",
    path: "../../wit/wasmcloud-ratelimit",
});

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use exports::wasmcloud::ratelimit::rate_limiter::Guest;
use exports::wasmcloud::ratelimit::rate_limiter::{
    RateLimitConfig, RateLimitError, RateLimitRequest, RateLimitResponse,
};

struct LeakyBucket {
    capacity: u64,
    leak_rate: u64,       // requests per second
    queue: VecDeque<u64>, // timestamps of queued requests
    last_leak_ms: u64,
}

impl LeakyBucket {
    fn new(capacity: u64, leak_rate: u64) -> Self {
        Self {
            capacity,
            leak_rate,
            queue: VecDeque::new(),
            last_leak_ms: 0,
        }
    }

    fn leak(&mut self, current_time_ms: u64) {
        if self.last_leak_ms == 0 {
            self.last_leak_ms = current_time_ms;
            return;
        }

        let elapsed_ms = current_time_ms.saturating_sub(self.last_leak_ms);
        let requests_to_leak = (elapsed_ms * self.leak_rate) / 1000; // leak_rate per second

        for _ in 0..requests_to_leak.min(self.queue.len() as u64) {
            self.queue.pop_front();
        }

        self.last_leak_ms = current_time_ms;
    }

    fn can_add(&self, tokens: u64) -> bool {
        (self.queue.len() as u64 + tokens) <= self.capacity
    }

    fn add_request(&mut self, tokens: u64, timestamp_ms: u64) -> bool {
        if !self.can_add(tokens) {
            return false;
        }

        for _ in 0..tokens {
            self.queue.push_back(timestamp_ms);
        }
        true
    }

    fn queue_size(&self) -> u64 {
        self.queue.len() as u64
    }

    fn retry_after_ms(&self, tokens_needed: u64) -> Option<u64> {
        let current_size = self.queue.len() as u64;
        let space_needed = (current_size + tokens_needed).saturating_sub(self.capacity);

        if space_needed == 0 {
            return None;
        }

        // Time needed to leak enough requests to make space
        let ms_needed = (space_needed * 1000) / self.leak_rate;
        Some(ms_needed)
    }
}

struct Component {
    config: Option<RateLimitConfig>,
    buckets: HashMap<String, LeakyBucket>,
}

thread_local! {
    static STATE: RefCell<Component> = RefCell::new(Component {
        config: None,
        buckets: HashMap::new(),
    });
}

struct LeakyBucketRateLimiter;

impl Guest for LeakyBucketRateLimiter {
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
                .or_insert_with(|| LeakyBucket::new(capacity, refill_rate));

            // Leak requests first
            bucket.leak(request.timestamp_ms);

            let allowed = bucket.add_request(request.tokens_requested, request.timestamp_ms);
            let tokens_remaining = capacity.saturating_sub(bucket.queue_size());
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

export!(LeakyBucketRateLimiter);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaky_bucket_basic() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 1, // leak rate: 1 req/sec
            window_size_ms: 0,
        };

        assert!(LeakyBucketRateLimiter::init(config).is_ok());

        let request = RateLimitRequest {
            user_id: "user1".to_string(),
            tokens_requested: 5,
            timestamp_ms: 1000,
        };

        let response = LeakyBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);
        assert_eq!(response.tokens_remaining, 5);
    }

    #[test]
    fn test_leaky_bucket_overflow() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 2, // leak rate: 2 req/sec
            window_size_ms: 0,
        };

        LeakyBucketRateLimiter::init(config).unwrap();

        // Fill the bucket
        let request = RateLimitRequest {
            user_id: "user2".to_string(),
            tokens_requested: 10,
            timestamp_ms: 1000,
        };
        let response = LeakyBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);

        // Try to add more - should fail
        let request = RateLimitRequest {
            user_id: "user2".to_string(),
            tokens_requested: 5,
            timestamp_ms: 1500,
        };
        let response = LeakyBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(!response.allowed);
        assert!(response.retry_after_ms.is_some());
    }

    #[test]
    fn test_leaky_bucket_leak() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 2, // leak rate: 2 req/sec
            window_size_ms: 0,
        };

        LeakyBucketRateLimiter::init(config).unwrap();

        // Fill the bucket
        let request = RateLimitRequest {
            user_id: "user3".to_string(),
            tokens_requested: 10,
            timestamp_ms: 1000,
        };
        let response = LeakyBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);

        // Wait 3 seconds (6 requests should leak)
        let request = RateLimitRequest {
            user_id: "user3".to_string(),
            tokens_requested: 5,
            timestamp_ms: 4000,
        };
        let response = LeakyBucketRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);
    }
}
