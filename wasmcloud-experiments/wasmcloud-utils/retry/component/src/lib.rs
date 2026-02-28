// WIT-based retry-with-backoff component.
// Targets the `retry-component` world defined in wit/wasmcloud-retry/retry.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "retry-component",
    path: "../../wit/wasmcloud-retry",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use retry_core::should_retry as core_should_retry;
use retry_core::{
    full_schedule as core_full_schedule, init as core_init, next_delay as core_next_delay,
    BackoffStrategy as CoreStrategy,
    RetryConfig as CoreConfig, RetryError as CoreError, RetrySchedule as CoreSchedule,
};

// ---- seed stub ---------------------------------------------------------------

#[allow(dead_code)]
fn jitter_seed() -> u64 {
    // Stub: fixed seed for deterministic behaviour in native tests.
    // In a real WASM component use wasi:clocks/monotonic-clock for entropy.
    42
}

// ---- type conversions (wasm32 only) -----------------------------------------

#[cfg(target_arch = "wasm32")]
fn core_strategy(s: wasmcloud::retry::types::BackoffStrategy) -> CoreStrategy {
    use wasmcloud::retry::types::BackoffStrategy;
    match s {
        BackoffStrategy::Fixed => CoreStrategy::Fixed,
        BackoffStrategy::Exponential => CoreStrategy::Exponential,
        BackoffStrategy::ExponentialJitter => CoreStrategy::ExponentialJitter,
    }
}

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::retry::types::RetryError {
    use wasmcloud::retry::types::RetryError;
    match e {
        CoreError::NotInitialized => RetryError::NotInitialized,
        CoreError::Exhausted => RetryError::Exhausted,
        CoreError::InvalidConfig => RetryError::InvalidConfig,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_schedule(s: CoreSchedule) -> wasmcloud::retry::types::RetrySchedule {
    wasmcloud::retry::types::RetrySchedule {
        attempt: s.attempt,
        delay_ms: s.delay_ms,
        is_last: s.is_last,
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct RetryComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::retry::retryer::Guest for RetryComponent {
    fn init(
        config: wasmcloud::retry::types::RetryConfig,
    ) -> Result<(), wasmcloud::retry::types::RetryError> {
        core_init(CoreConfig {
            max_attempts: config.max_attempts,
            base_delay_ms: config.base_delay_ms,
            max_delay_ms: config.max_delay_ms,
            strategy: core_strategy(config.strategy),
        })
        .map_err(core_error)
    }

    fn next_delay(
        attempt: u32,
    ) -> Result<wasmcloud::retry::types::RetrySchedule, wasmcloud::retry::types::RetryError> {
        core_next_delay(attempt, jitter_seed()).map(wit_schedule).map_err(core_error)
    }

    fn schedule(
    ) -> Result<Vec<wasmcloud::retry::types::RetrySchedule>, wasmcloud::retry::types::RetryError>
    {
        core_full_schedule(jitter_seed())
            .map(|v| v.into_iter().map(wit_schedule).collect())
            .map_err(core_error)
    }

    fn should_retry(attempt: u32) -> Result<bool, wasmcloud::retry::types::RetryError> {
        core_should_retry(attempt).map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(RetryComponent);

// ---- native helpers (cargo check / tests) -----------------------------------

pub fn retry_init(
    max_attempts: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    strategy: CoreStrategy,
) -> Result<(), CoreError> {
    core_init(CoreConfig { max_attempts, base_delay_ms, max_delay_ms, strategy })
}

pub fn retry_next(attempt: u32) -> Result<CoreSchedule, CoreError> {
    core_next_delay(attempt, 42)
}

pub fn retry_schedule() -> Result<Vec<CoreSchedule>, CoreError> {
    core_full_schedule(42)
}

#[cfg(test)]
mod tests {
    use super::*;
    use retry_core::BackoffStrategy;

    #[test]
    fn roundtrip_fixed() {
        retry_init(3, 100, 10_000, BackoffStrategy::Fixed).unwrap();
        let s = retry_next(1).unwrap();
        assert_eq!(s.delay_ms, 100);
        assert!(!s.is_last);
        let s3 = retry_next(3).unwrap();
        assert!(s3.is_last);
    }

    #[test]
    fn roundtrip_full_schedule() {
        retry_init(4, 200, 60_000, BackoffStrategy::Exponential).unwrap();
        let sched = retry_schedule().unwrap();
        assert_eq!(sched.len(), 4);
        // exponential: 200, 400, 800, 1600
        assert_eq!(sched[0].delay_ms, 200);
        assert_eq!(sched[1].delay_ms, 400);
        assert_eq!(sched[2].delay_ms, 800);
        assert_eq!(sched[3].delay_ms, 1600);
    }

    #[test]
    fn exhausted_returns_error() {
        retry_init(2, 50, 1000, BackoffStrategy::Fixed).unwrap();
        assert!(retry_next(3).is_err());
    }
}
