// WIT-based observability component.
// Targets the `observability-component` world defined in
// wit/wasmcloud-observability/observability.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "observability-component",
    path: "../../wit/wasmcloud-observability",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use observability_core::{
    counters as core_counters, drain as core_drain, gauge_set as core_gauge_set,
    gauges as core_gauges, increment as core_increment, log as core_log, log_msg as core_log_msg,
    reset as core_reset, set_level as core_set_level, CounterSnapshot as CoreCounter,
    GaugeSnapshot as CoreGauge, LogEntry as CoreEntry, LogLevel as CoreLevel,
    ObsError as CoreError,
};

// ── type conversions (wasm32 only) ────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn core_level(l: wasmcloud::observability::types::LogLevel) -> CoreLevel {
    use wasmcloud::observability::types::LogLevel;
    match l {
        LogLevel::Trace => CoreLevel::Trace,
        LogLevel::Debug => CoreLevel::Debug,
        LogLevel::Info => CoreLevel::Info,
        LogLevel::Warn => CoreLevel::Warn,
        LogLevel::Error => CoreLevel::Error,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_level(l: CoreLevel) -> wasmcloud::observability::types::LogLevel {
    use wasmcloud::observability::types::LogLevel;
    match l {
        CoreLevel::Trace => LogLevel::Trace,
        CoreLevel::Debug => LogLevel::Debug,
        CoreLevel::Info => LogLevel::Info,
        CoreLevel::Warn => LogLevel::Warn,
        CoreLevel::Error => LogLevel::Error,
    }
}

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::observability::types::ObsError {
    use wasmcloud::observability::types::ObsError;
    match e {
        CoreError::NotInitialized => ObsError::NotInitialized,
        CoreError::InvalidName => ObsError::InvalidName,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_entry(e: CoreEntry) -> wasmcloud::observability::types::LogEntry {
    wasmcloud::observability::types::LogEntry {
        level: wit_level(e.level),
        target: e.target,
        message: e.message,
        fields: e.fields,
        timestamp_ms: e.timestamp_ms,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_counter(c: CoreCounter) -> wasmcloud::observability::types::CounterSnapshot {
    wasmcloud::observability::types::CounterSnapshot {
        name: c.name,
        value: c.value,
        labels: c.labels,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_gauge(g: CoreGauge) -> wasmcloud::observability::types::GaugeSnapshot {
    wasmcloud::observability::types::GaugeSnapshot {
        name: g.name,
        value: g.value,
        labels: g.labels,
    }
}

// ── WIT guest — logger ────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
struct ObsComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::observability::logger::Guest for ObsComponent {
    fn set_level(
        level: wasmcloud::observability::types::LogLevel,
    ) -> Result<(), wasmcloud::observability::types::ObsError> {
        core_set_level(core_level(level)).map_err(core_error)
    }

    fn log(
        entry: wasmcloud::observability::types::LogEntry,
    ) -> Result<(), wasmcloud::observability::types::ObsError> {
        core_log(CoreEntry {
            level: core_level(entry.level),
            target: entry.target,
            message: entry.message,
            fields: entry.fields,
            timestamp_ms: entry.timestamp_ms,
        })
        .map_err(core_error)
    }

    fn log_msg(
        level: wasmcloud::observability::types::LogLevel,
        target: String,
        message: String,
    ) -> Result<(), wasmcloud::observability::types::ObsError> {
        core_log_msg(core_level(level), &target, &message).map_err(core_error)
    }

    fn drain(
    ) -> Result<Vec<wasmcloud::observability::types::LogEntry>, wasmcloud::observability::types::ObsError>
    {
        core_drain().map(|v| v.into_iter().map(wit_entry).collect()).map_err(core_error)
    }
}

// ── WIT guest — metrics ───────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::observability::metrics::Guest for ObsComponent {
    fn increment(
        name: String,
        delta: u64,
        labels: Vec<(String, String)>,
    ) -> Result<(), wasmcloud::observability::types::ObsError> {
        core_increment(&name, delta, labels).map_err(core_error)
    }

    fn gauge_set(
        name: String,
        value: i64,
        labels: Vec<(String, String)>,
    ) -> Result<(), wasmcloud::observability::types::ObsError> {
        core_gauge_set(&name, value, labels).map_err(core_error)
    }

    fn counters(
    ) -> Result<
        Vec<wasmcloud::observability::types::CounterSnapshot>,
        wasmcloud::observability::types::ObsError,
    > {
        core_counters().map(|v| v.into_iter().map(wit_counter).collect()).map_err(core_error)
    }

    fn gauges(
    ) -> Result<
        Vec<wasmcloud::observability::types::GaugeSnapshot>,
        wasmcloud::observability::types::ObsError,
    > {
        core_gauges().map(|v| v.into_iter().map(wit_gauge).collect()).map_err(core_error)
    }

    fn reset() -> Result<(), wasmcloud::observability::types::ObsError> {
        core_reset().map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(ObsComponent);

// ── native helpers (cargo check / tests) ──────────────────────────────────────

pub use observability_core::{
    counters, drain, gauge_set, gauges, increment, log, log_msg, reset, set_level, LogLevel,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn roundtrip_log_and_drain() {
        run(|| {
            set_level(LogLevel::Info).unwrap();
            log_msg(LogLevel::Info, "comp", "hello from component").unwrap();
            let entries = drain().unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].message, "hello from component");
        });
    }

    #[test]
    fn roundtrip_counter_and_gauge() {
        run(|| {
            increment("reqs", 3, vec![]).unwrap();
            gauge_set("conns", -1, vec![]).unwrap();
            let cs = counters().unwrap();
            let gs = gauges().unwrap();
            assert!(cs.iter().any(|c| c.name == "reqs" && c.value == 3));
            assert!(gs.iter().any(|g| g.name == "conns" && g.value == -1));
            reset().unwrap();
        });
    }
}
