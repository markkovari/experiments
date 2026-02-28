// WIT-based circuit-breaker component.
// Targets the `circuit-breaker-component` world defined in
// wit/wasmcloud-circuit-breaker/circuit-breaker.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "circuit-breaker-component",
    path: "../../wit/wasmcloud-circuit-breaker",
    generate_all,
});

use circuit_breaker_core::{
    get_state as core_get_state, init as core_init, record_call as core_record_call,
    reset as core_reset, BreakerConfig as CoreConfig, BreakerError as CoreError,
    CallResult as CoreCallResult, CircuitState as CoreState,
};

// ---- time stub ---------------------------------------------------------------

#[allow(dead_code)]
fn now_ms() -> u64 {
    // Stub: returns 0 for native builds.
    // In a real WASM component use wasi:clocks/monotonic-clock.
    0
}

// ---- type conversions (wasm32 only) -----------------------------------------

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::circuit_breaker::types::CircuitError {
    use wasmcloud::circuit_breaker::types::CircuitError;
    match e {
        CoreError::NotInitialized => CircuitError::NotInitialized,
        CoreError::OpenCircuit => CircuitError::OpenCircuit,
        CoreError::InvalidConfig => CircuitError::InvalidConfig,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_state(s: CoreState) -> wasmcloud::circuit_breaker::types::CircuitState {
    use wasmcloud::circuit_breaker::types::CircuitState;
    match s {
        CoreState::Closed => CircuitState::Closed,
        CoreState::Open => CircuitState::Open,
        CoreState::HalfOpen => CircuitState::HalfOpen,
    }
}

#[cfg(target_arch = "wasm32")]
fn core_call_result(r: wasmcloud::circuit_breaker::types::CallResult) -> CoreCallResult {
    use wasmcloud::circuit_breaker::types::CallResult;
    match r {
        CallResult::Success => CoreCallResult::Success,
        CallResult::Failure => CoreCallResult::Failure,
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct BreakerComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::circuit_breaker::breaker::Guest for BreakerComponent {
    fn init(
        config: wasmcloud::circuit_breaker::types::CircuitConfig,
    ) -> Result<(), wasmcloud::circuit_breaker::types::CircuitError> {
        core_init(CoreConfig {
            failure_threshold: config.failure_threshold,
            success_threshold: config.success_threshold,
            timeout_ms: config.timeout_ms,
        })
        .map_err(core_error)
    }

    fn call(
        key: String,
        result: wasmcloud::circuit_breaker::types::CallResult,
    ) -> Result<(), wasmcloud::circuit_breaker::types::CircuitError> {
        core_record_call(&key, core_call_result(result), now_ms()).map_err(core_error)
    }

    fn state(
        key: String,
    ) -> Result<wasmcloud::circuit_breaker::types::CircuitState, wasmcloud::circuit_breaker::types::CircuitError>
    {
        core_get_state(&key).map(wit_state).map_err(core_error)
    }

    fn reset(
        key: String,
    ) -> Result<(), wasmcloud::circuit_breaker::types::CircuitError> {
        core_reset(&key).map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(BreakerComponent);

// ---- native helpers (cargo check / tests) -----------------------------------

pub fn breaker_init(ft: u64, st: u64, timeout_ms: u64) -> Result<(), CoreError> {
    core_init(CoreConfig { failure_threshold: ft, success_threshold: st, timeout_ms })
}

pub fn breaker_call(key: &str, result: CoreCallResult, now_ms: u64) -> Result<(), CoreError> {
    core_record_call(key, result, now_ms)
}

pub fn breaker_state(key: &str) -> Result<CoreState, CoreError> {
    core_get_state(key)
}

pub fn breaker_reset(key: &str) -> Result<(), CoreError> {
    core_reset(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use circuit_breaker_core::CallResult;

    #[test]
    fn roundtrip_closed_to_open() {
        breaker_init(2, 1, 500).unwrap();
        assert_eq!(breaker_state("comp_trip").unwrap(), CoreState::Closed);
        breaker_call("comp_trip", CallResult::Failure, 0).unwrap();
        breaker_call("comp_trip", CallResult::Failure, 0).unwrap();
        assert_eq!(breaker_state("comp_trip").unwrap(), CoreState::Open);
    }

    #[test]
    fn open_to_half_open_to_closed() {
        breaker_init(1, 1, 1000).unwrap();
        breaker_call("comp_probe", CallResult::Failure, 0).unwrap();
        assert!(
            breaker_call("comp_probe", CallResult::Success, 500).is_err(),
            "should be rejected while open before timeout"
        );
        // Timeout elapsed: probe succeeds → Closed
        breaker_call("comp_probe", CallResult::Success, 1000).unwrap();
        assert_eq!(breaker_state("comp_probe").unwrap(), CoreState::Closed);
    }
}
