// WIT-based feature-flags component.
// Targets the `feature-flags-component` world defined in
// wit/wasmcloud-feature-flags/feature-flags.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "feature-flags-component",
    path: "../../wit/wasmcloud-feature-flags",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use feature_flags_core::{
    delete as core_delete, get as core_get, is_enabled as core_is_enabled,
    list as core_list, set as core_set, Flag as CoreFlag, FlagError as CoreError,
    FlagValue as CoreValue,
};

// ── time stub ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn now_ms() -> u64 {
    // Stub: returns 0 for native builds.
    // In a real WASM component use wasi:clocks/wall-clock.
    0
}

// ── type conversions (wasm32 only) ────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn core_value(v: wasmcloud::feature_flags::types::FlagValue) -> CoreValue {
    use wasmcloud::feature_flags::types::FlagValue;
    match v {
        FlagValue::Boolean(b) => CoreValue::Boolean(b),
        FlagValue::Text(s) => CoreValue::Text(s),
        FlagValue::Integer(i) => CoreValue::Integer(i),
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_value(v: CoreValue) -> wasmcloud::feature_flags::types::FlagValue {
    use wasmcloud::feature_flags::types::FlagValue;
    match v {
        CoreValue::Boolean(b) => FlagValue::Boolean(b),
        CoreValue::Text(s) => FlagValue::Text(s),
        CoreValue::Integer(i) => FlagValue::Integer(i),
    }
}

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::feature_flags::types::FlagError {
    use wasmcloud::feature_flags::types::FlagError;
    match e {
        CoreError::NotInitialized => FlagError::NotInitialized,
        CoreError::NotFound => FlagError::NotFound,
        CoreError::InvalidKey => FlagError::InvalidKey,
        CoreError::StorageError => FlagError::StorageError,
        CoreError::TypeMismatch => FlagError::TypeMismatch,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_flag(f: CoreFlag) -> wasmcloud::feature_flags::types::Flag {
    wasmcloud::feature_flags::types::Flag {
        key: f.key,
        value: wit_value(f.value),
        description: f.description,
        updated_at_ms: f.updated_at_ms,
    }
}

// ── WIT guest implementation ──────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
struct FlagsComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::feature_flags::feature_flags_api::Guest for FlagsComponent {
    fn is_enabled(key: String) -> Result<bool, wasmcloud::feature_flags::types::FlagError> {
        core_is_enabled(&key).map_err(core_error)
    }

    fn get(
        key: String,
    ) -> Result<wasmcloud::feature_flags::types::Flag, wasmcloud::feature_flags::types::FlagError>
    {
        core_get(&key).map(wit_flag).map_err(core_error)
    }

    fn set(
        key: String,
        value: wasmcloud::feature_flags::types::FlagValue,
        description: Option<String>,
    ) -> Result<(), wasmcloud::feature_flags::types::FlagError> {
        core_set(&key, core_value(value), description, now_ms()).map_err(core_error)
    }

    fn delete(key: String) -> Result<(), wasmcloud::feature_flags::types::FlagError> {
        core_delete(&key).map_err(core_error)
    }

    fn list_all(
    ) -> Result<Vec<wasmcloud::feature_flags::types::Flag>, wasmcloud::feature_flags::types::FlagError>
    {
        core_list().map(|v| v.into_iter().map(wit_flag).collect()).map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(FlagsComponent);

// ── native helpers (cargo check / tests) ──────────────────────────────────────

pub use feature_flags_core::{delete, get, is_enabled, list, set, FlagError, FlagValue};

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn roundtrip_boolean() {
        run(|| {
            set("comp-flag", FlagValue::Boolean(true), None, 0).unwrap();
            assert!(is_enabled("comp-flag").unwrap());
            delete("comp-flag").unwrap();
            assert!(!is_enabled("comp-flag").unwrap());
        });
    }

    #[test]
    fn roundtrip_text_and_list() {
        run(|| {
            set("comp-txt", FlagValue::Text("beta".to_string()), Some("desc".to_string()), 0).unwrap();
            let flags = list().unwrap();
            assert!(flags.iter().any(|f| f.key == "comp-txt"));
        });
    }
}
