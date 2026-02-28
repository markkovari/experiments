wit_bindgen::generate!({
    world: "rate-limiter-component",
    path: "../../wit",
});

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use exports::wasmcloud::ratelimit::rate_limiter::Guest;
use exports::wasmcloud::ratelimit::rate_limiter::{
    RateLimitConfig, RateLimitError, RateLimitRequest, RateLimitResponse,
};

struct RequestRecord {
    timestamp_ms: u64,
    tokens: u64,
}

struct SlidingWindow {
    capacity: u64,
    window_size_ms: u64,
    requests: VecDeque<RequestRecord>,
}

impl SlidingWindow {
    fn new(capacity: u64, window_size_ms: u64) -> Self {
        Self {
            capacity,
            window_size_ms,
            requests: VecDeque::new(),
        }
    }

    fn cleanup_old_requests(&mut self, current_time_ms: u64) {
        let window_start = current_time_ms.saturating_sub(self.window_size_ms);

        while let Some(record) = self.requests.front() {
            if record.timestamp_ms < window_start {
                self.requests.pop_front();
            } else {
                break;
            }
        }
    }

    fn count_tokens_in_window(&self) -> u64 {
        self.requests.iter().map(|r| r.tokens).sum()
    }

    fn can_add(&self, tokens: u64) -> bool {
        let current_count = self.count_tokens_in_window();
        current_count + tokens <= self.capacity
    }

    fn add_request(&mut self, tokens: u64, timestamp_ms: u64) -> bool {
        if !self.can_add(tokens) {
            return false;
        }

        self.requests.push_back(RequestRecord {
            timestamp_ms,
            tokens,
        });
        true
    }

    fn tokens_remaining(&self) -> u64 {
        self.capacity.saturating_sub(self.count_tokens_in_window())
    }

    fn retry_after_ms(&self, tokens_needed: u64) -> Option<u64> {
        let current_count = self.count_tokens_in_window();

        if current_count + tokens_needed <= self.capacity {
            return None;
        }

        // Find the oldest request(s) that need to expire to make room
        let tokens_to_free = (current_count + tokens_needed).saturating_sub(self.capacity);
        let mut freed = 0;

        for record in &self.requests {
            freed += record.tokens;
            if freed >= tokens_to_free {
                // This request needs to age out of the window
                let retry_at_ms = record.timestamp_ms + self.window_size_ms;
                return Some(retry_at_ms.saturating_sub(record.timestamp_ms));
            }
        }

        Some(self.window_size_ms)
    }
}

struct Component {
    config: Option<RateLimitConfig>,
    windows: HashMap<String, SlidingWindow>,
}

thread_local! {
    static STATE: RefCell<Component> = RefCell::new(Component {
        config: None,
        windows: HashMap::new(),
    });
}

struct SlidingWindowRateLimiter;

impl Guest for SlidingWindowRateLimiter {
    fn init(config: RateLimitConfig) -> Result<(), RateLimitError> {
        if config.capacity == 0 || config.window_size_ms == 0 {
            return Err(RateLimitError::InvalidConfig);
        }

        STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.config = Some(config);
            s.windows.clear();
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
            let window_size_ms = config.window_size_ms;

            let window = s
                .windows
                .entry(request.user_id.clone())
                .or_insert_with(|| SlidingWindow::new(capacity, window_size_ms));

            // Remove expired requests from the window
            window.cleanup_old_requests(request.timestamp_ms);

            let allowed = window.add_request(request.tokens_requested, request.timestamp_ms);
            let tokens_remaining = window.tokens_remaining();
            let retry_after_ms = if !allowed {
                window.retry_after_ms(request.tokens_requested)
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
            s.windows.remove(&user_id);
        });

        Ok(())
    }
}

export!(SlidingWindowRateLimiter);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_basic() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 0,       // not used in sliding window
            window_size_ms: 5000, // 5 second window
        };

        assert!(SlidingWindowRateLimiter::init(config).is_ok());

        let request = RateLimitRequest {
            user_id: "user1".to_string(),
            tokens_requested: 5,
            timestamp_ms: 1000,
        };

        let response = SlidingWindowRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);
        assert_eq!(response.tokens_remaining, 5);
    }

    #[test]
    fn test_sliding_window_limit() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 0,
            window_size_ms: 5000,
        };

        SlidingWindowRateLimiter::init(config).unwrap();

        // Use up the limit
        let request = RateLimitRequest {
            user_id: "user2".to_string(),
            tokens_requested: 10,
            timestamp_ms: 1000,
        };
        let response = SlidingWindowRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);

        // Try to exceed limit - should fail
        let request = RateLimitRequest {
            user_id: "user2".to_string(),
            tokens_requested: 5,
            timestamp_ms: 2000,
        };
        let response = SlidingWindowRateLimiter::check_rate_limit(request).unwrap();
        assert!(!response.allowed);
        assert!(response.retry_after_ms.is_some());
    }

    #[test]
    fn test_sliding_window_expiry() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 0,
            window_size_ms: 5000,
        };

        SlidingWindowRateLimiter::init(config).unwrap();

        // Use up the limit
        let request = RateLimitRequest {
            user_id: "user3".to_string(),
            tokens_requested: 10,
            timestamp_ms: 1000,
        };
        let response = SlidingWindowRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);

        // Wait for window to expire (after 5 seconds)
        let request = RateLimitRequest {
            user_id: "user3".to_string(),
            tokens_requested: 10,
            timestamp_ms: 7000,
        };
        let response = SlidingWindowRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);
    }

    #[test]
    fn test_sliding_window_partial_expiry() {
        let config = RateLimitConfig {
            capacity: 10,
            refill_rate: 0,
            window_size_ms: 5000,
        };

        SlidingWindowRateLimiter::init(config).unwrap();

        // Add 5 tokens at t=1000
        let request = RateLimitRequest {
            user_id: "user4".to_string(),
            tokens_requested: 5,
            timestamp_ms: 1000,
        };
        SlidingWindowRateLimiter::check_rate_limit(request).unwrap();

        // Add 5 more tokens at t=3000
        let request = RateLimitRequest {
            user_id: "user4".to_string(),
            tokens_requested: 5,
            timestamp_ms: 3000,
        };
        let response = SlidingWindowRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);

        // At t=7000, first 5 tokens should have expired, so we can add 5 more
        let request = RateLimitRequest {
            user_id: "user4".to_string(),
            tokens_requested: 5,
            timestamp_ms: 7000,
        };
        let response = SlidingWindowRateLimiter::check_rate_limit(request).unwrap();
        assert!(response.allowed);
    }
}
