use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct BreakerConfig {
    pub failure_threshold: u64,
    pub success_threshold: u64,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BreakerError {
    NotInitialized,
    OpenCircuit,
    InvalidConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallResult {
    Success,
    Failure,
}

#[derive(Debug, Clone)]
struct BreakerEntry {
    state: CircuitState,
    failure_count: u64,
    success_count: u64,
    opened_at_ms: u64,
}

impl BreakerEntry {
    fn new() -> Self {
        BreakerEntry { state: CircuitState::Closed, failure_count: 0, success_count: 0, opened_at_ms: 0 }
    }
}

// NOTE: Thread-local in-memory state — test stub only.
// In a deployed WASM component with shared state, use wasi:keyvalue.
// thread_local! compiles to a plain static in wasm32 targets and is used here
// solely for Rust borrow-checker compatibility on native test targets.
struct BreakerState {
    config: Option<BreakerConfig>,
    entries: HashMap<String, BreakerEntry>,
}

thread_local! {
    static BREAKER: RefCell<BreakerState> = RefCell::new(BreakerState {
        config: None,
        entries: HashMap::new(),
    });
}

fn with_breaker<R>(f: impl FnOnce(&mut BreakerState) -> R) -> R {
    BREAKER.with(|b| f(&mut b.borrow_mut()))
}

/// Initialise global configuration. Must be called before any other function.
/// Returns `InvalidConfig` if either threshold is zero.
pub fn init(config: BreakerConfig) -> Result<(), BreakerError> {
    if config.failure_threshold == 0 || config.success_threshold == 0 {
        return Err(BreakerError::InvalidConfig);
    }
    with_breaker(|b| {
        b.config = Some(config);
        Ok(())
    })
}

/// Record the outcome of a protected call for the named circuit.
///
/// State transitions:
/// - Closed + Failure × failure_threshold  → Open
/// - Closed + Success                      → reset failure counter
/// - Open   (timeout elapsed)              → Half-Open, then process result
/// - Open   (timeout not elapsed)          → Err(OpenCircuit)
/// - Half-Open + Success × success_threshold → Closed
/// - Half-Open + Failure                   → Open
pub fn record_call(key: &str, result: CallResult, now_ms: u64) -> Result<(), BreakerError> {
    with_breaker(|b| {
        let config = b.config.as_ref().ok_or(BreakerError::NotInitialized)?.clone();
        let entry = b.entries.entry(key.to_string()).or_insert_with(BreakerEntry::new);

        // Open → possibly probe via Half-Open
        if entry.state == CircuitState::Open {
            if now_ms >= entry.opened_at_ms + config.timeout_ms {
                entry.state = CircuitState::HalfOpen;
                entry.failure_count = 0;
                entry.success_count = 0;
            } else {
                return Err(BreakerError::OpenCircuit);
            }
        }

        match (&entry.state.clone(), &result) {
            (CircuitState::Closed, CallResult::Failure) => {
                entry.failure_count += 1;
                if entry.failure_count >= config.failure_threshold {
                    entry.state = CircuitState::Open;
                    entry.opened_at_ms = now_ms;
                    entry.failure_count = 0;
                    entry.success_count = 0;
                }
            }
            (CircuitState::Closed, CallResult::Success) => {
                entry.failure_count = 0;
            }
            (CircuitState::HalfOpen, CallResult::Success) => {
                entry.success_count += 1;
                if entry.success_count >= config.success_threshold {
                    entry.state = CircuitState::Closed;
                    entry.failure_count = 0;
                    entry.success_count = 0;
                }
            }
            (CircuitState::HalfOpen, CallResult::Failure) => {
                entry.state = CircuitState::Open;
                entry.opened_at_ms = now_ms;
                entry.failure_count = 0;
                entry.success_count = 0;
            }
            _ => {}
        }
        Ok(())
    })
}

/// Return the current state of the named circuit (Closed if never seen).
pub fn get_state(key: &str) -> Result<CircuitState, BreakerError> {
    with_breaker(|b| {
        if b.config.is_none() {
            return Err(BreakerError::NotInitialized);
        }
        Ok(b.entries.get(key).map(|e| e.state.clone()).unwrap_or(CircuitState::Closed))
    })
}

/// Force the named circuit back to Closed and clear all counters.
pub fn reset(key: &str) -> Result<(), BreakerError> {
    with_breaker(|b| {
        if b.config.is_none() {
            return Err(BreakerError::NotInitialized);
        }
        b.entries.remove(key);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(ft: u64, st: u64, tm: u64) -> BreakerConfig {
        BreakerConfig { failure_threshold: ft, success_threshold: st, timeout_ms: tm }
    }

    #[test]
    fn test_closed_trips_to_open_after_threshold() {
        init(cfg(3, 2, 1000)).unwrap();
        for _ in 0..2 {
            record_call("trip1", CallResult::Failure, 0).unwrap();
        }
        assert_eq!(get_state("trip1").unwrap(), CircuitState::Closed);
        record_call("trip1", CallResult::Failure, 0).unwrap();
        assert_eq!(get_state("trip1").unwrap(), CircuitState::Open);
    }

    #[test]
    fn test_open_rejects_calls_immediately() {
        init(cfg(1, 1, 5000)).unwrap();
        record_call("reject1", CallResult::Failure, 0).unwrap();
        let err = record_call("reject1", CallResult::Success, 100).unwrap_err();
        assert_eq!(err, BreakerError::OpenCircuit);
    }

    #[test]
    fn test_open_transitions_to_half_open_after_timeout() {
        init(cfg(1, 1, 500)).unwrap();
        record_call("probe1", CallResult::Failure, 0).unwrap();
        assert_eq!(get_state("probe1").unwrap(), CircuitState::Open);
        // Before timeout — still rejected
        assert_eq!(record_call("probe1", CallResult::Success, 499).unwrap_err(), BreakerError::OpenCircuit);
        // At timeout boundary — probe allowed → Half-Open → Success → Closed
        record_call("probe1", CallResult::Success, 500).unwrap();
        assert_eq!(get_state("probe1").unwrap(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_closes_on_success_threshold() {
        init(cfg(1, 2, 100)).unwrap();
        record_call("close1", CallResult::Failure, 0).unwrap();
        // Probe: first success → still Half-Open
        record_call("close1", CallResult::Success, 100).unwrap();
        assert_eq!(get_state("close1").unwrap(), CircuitState::HalfOpen);
        // Second success → Closed
        record_call("close1", CallResult::Success, 100).unwrap();
        assert_eq!(get_state("close1").unwrap(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_reopens_on_failure() {
        init(cfg(1, 2, 100)).unwrap();
        record_call("reopen1", CallResult::Failure, 0).unwrap();
        record_call("reopen1", CallResult::Success, 100).unwrap(); // → HalfOpen
        assert_eq!(get_state("reopen1").unwrap(), CircuitState::HalfOpen);
        record_call("reopen1", CallResult::Failure, 100).unwrap(); // → Open
        assert_eq!(get_state("reopen1").unwrap(), CircuitState::Open);
    }

    #[test]
    fn test_reset_clears_entry() {
        init(cfg(1, 1, 1000)).unwrap();
        record_call("rst1", CallResult::Failure, 0).unwrap();
        assert_eq!(get_state("rst1").unwrap(), CircuitState::Open);
        reset("rst1").unwrap();
        assert_eq!(get_state("rst1").unwrap(), CircuitState::Closed);
    }

    #[test]
    fn test_not_initialized_error() {
        // Fresh thread_local — config is None if init was never called in this thread.
        // Use a separate thread to guarantee isolation.
        let result = std::thread::spawn(|| get_state("x")).join().unwrap();
        assert_eq!(result.unwrap_err(), BreakerError::NotInitialized);
    }

    #[test]
    fn test_invalid_config_zero_threshold() {
        assert_eq!(init(cfg(0, 1, 1000)).unwrap_err(), BreakerError::InvalidConfig);
        assert_eq!(init(cfg(1, 0, 1000)).unwrap_err(), BreakerError::InvalidConfig);
    }

    #[test]
    fn test_success_in_closed_resets_failure_count() {
        init(cfg(3, 1, 1000)).unwrap();
        record_call("succ_rst", CallResult::Failure, 0).unwrap();
        record_call("succ_rst", CallResult::Failure, 0).unwrap();
        // Success resets failure counter — 2 more failures needed to trip
        record_call("succ_rst", CallResult::Success, 0).unwrap();
        record_call("succ_rst", CallResult::Failure, 0).unwrap();
        record_call("succ_rst", CallResult::Failure, 0).unwrap();
        assert_eq!(get_state("succ_rst").unwrap(), CircuitState::Closed);
        record_call("succ_rst", CallResult::Failure, 0).unwrap();
        assert_eq!(get_state("succ_rst").unwrap(), CircuitState::Open);
    }

    #[test]
    fn test_independent_keys() {
        init(cfg(2, 1, 1000)).unwrap();
        record_call("ind_a", CallResult::Failure, 0).unwrap();
        record_call("ind_a", CallResult::Failure, 0).unwrap();
        assert_eq!(get_state("ind_a").unwrap(), CircuitState::Open);
        assert_eq!(get_state("ind_b").unwrap(), CircuitState::Closed);
    }
}
