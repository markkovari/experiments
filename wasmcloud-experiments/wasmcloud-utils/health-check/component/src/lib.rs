// WIT-based health-check component.
// Targets the `health-check-component` world defined in wit/wasmcloud-health-check/health-check.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "health-check-component",
    path: "../../wit/wasmcloud-health-check",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use health_check_core::{
    all_probes as core_all_probes, deregister as core_deregister,
    record_result as core_record_result, register as core_register, status as core_status,
    HealthError as CoreError, OverallStatus as CoreStatus,
};

#[allow(dead_code)]
fn now_ms() -> u64 {
    0
}

#[cfg(target_arch = "wasm32")]
fn core_err(e: CoreError) -> wasmcloud::health_check::types::HealthError {
    use wasmcloud::health_check::types::HealthError;
    match e {
        CoreError::NotFound => HealthError::NotFound,
        CoreError::InvalidName => HealthError::InvalidName,
        CoreError::DuplicateProbe => HealthError::DuplicateProbe,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_status(s: CoreStatus) -> wasmcloud::health_check::types::OverallStatus {
    use wasmcloud::health_check::types::OverallStatus;
    match s {
        CoreStatus::Healthy => OverallStatus::Healthy,
        CoreStatus::Degraded => OverallStatus::Degraded,
        CoreStatus::Unhealthy => OverallStatus::Unhealthy,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_probe(p: health_check_core::ProbeResult) -> wasmcloud::health_check::types::ProbeResult {
    wasmcloud::health_check::types::ProbeResult {
        name: p.name,
        healthy: p.healthy,
        last_check_ms: p.last_check_ms,
        message: p.message,
    }
}

#[cfg(target_arch = "wasm32")]
struct HealthCheckComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::health_check::health_api::Guest for HealthCheckComponent {
    fn register(name: String) -> Result<(), wasmcloud::health_check::types::HealthError> {
        core_register(&name).map_err(core_err)
    }

    fn record_result(
        name: String,
        healthy: bool,
        message: Option<String>,
    ) -> Result<(), wasmcloud::health_check::types::HealthError> {
        core_record_result(&name, healthy, message, now_ms()).map_err(core_err)
    }

    fn status() -> Result<wasmcloud::health_check::types::OverallStatus, wasmcloud::health_check::types::HealthError> {
        core_status().map(wit_status).map_err(core_err)
    }

    fn all_probes(
    ) -> Result<Vec<wasmcloud::health_check::types::ProbeResult>, wasmcloud::health_check::types::HealthError> {
        core_all_probes().map(|v| v.into_iter().map(wit_probe).collect()).map_err(core_err)
    }

    fn deregister(name: String) -> Result<(), wasmcloud::health_check::types::HealthError> {
        core_deregister(&name).map_err(core_err)
    }
}

#[cfg(target_arch = "wasm32")]
export!(HealthCheckComponent);

// ── native helpers ────────────────────────────────────────────────────────────

pub use health_check_core::{
    all_probes, deregister, record_result, register, status, HealthError, OverallStatus,
    ProbeResult,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        std::thread::spawn(|| {
            register("api").unwrap();
            record_result("api", true, Some("ok".to_string()), 100).unwrap();
            assert_eq!(status().unwrap(), OverallStatus::Healthy);
            let probes = all_probes().unwrap();
            assert!(probes.iter().any(|p| p.name == "api" && p.healthy));
            deregister("api").unwrap();
        })
        .join()
        .unwrap();
    }

    #[test]
    fn degraded_mix() {
        std::thread::spawn(|| {
            register("x").unwrap();
            register("y").unwrap();
            record_result("x", true, None, 0).unwrap();
            record_result("y", false, None, 0).unwrap();
            assert_eq!(status().unwrap(), OverallStatus::Degraded);
        })
        .join()
        .unwrap();
    }
}
