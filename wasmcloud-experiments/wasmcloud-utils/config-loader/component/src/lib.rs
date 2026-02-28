// WIT-based config-loader component.
// Targets the `config-loader-component` world defined in wit/wasmcloud-config-loader/config-loader.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "config-loader-component",
    path: "../../wit/wasmcloud-config-loader",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use config_loader_core::{
    contains as core_contains, delete as core_delete, get as core_get,
    get_or_default as core_get_or_default, list_keys as core_list_keys, set as core_set,
    ConfigError as CoreError,
};

#[allow(dead_code)]
fn now_ms() -> u64 {
    0
}

#[cfg(target_arch = "wasm32")]
fn core_err(e: CoreError) -> wasmcloud::config_loader::types::ConfigError {
    use wasmcloud::config_loader::types::ConfigError;
    match e {
        CoreError::InvalidKey => ConfigError::InvalidKey,
        CoreError::NotFound => ConfigError::NotFound,
        CoreError::StorageError => ConfigError::StorageError,
    }
}

#[cfg(target_arch = "wasm32")]
struct ConfigLoaderComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::config_loader::config_api::Guest for ConfigLoaderComponent {
    fn set(
        key: String,
        value: String,
        is_secret: bool,
    ) -> Result<(), wasmcloud::config_loader::types::ConfigError> {
        core_set(&key, &value, is_secret, now_ms()).map_err(core_err)
    }

    fn get(key: String) -> Result<Option<String>, wasmcloud::config_loader::types::ConfigError> {
        core_get(&key).map_err(core_err)
    }

    fn get_or_default(
        key: String,
        default: String,
    ) -> Result<String, wasmcloud::config_loader::types::ConfigError> {
        core_get_or_default(&key, &default).map_err(core_err)
    }

    fn contains(key: String) -> Result<bool, wasmcloud::config_loader::types::ConfigError> {
        core_contains(&key).map_err(core_err)
    }

    fn list_keys() -> Result<Vec<String>, wasmcloud::config_loader::types::ConfigError> {
        core_list_keys().map_err(core_err)
    }

    fn delete(key: String) -> Result<(), wasmcloud::config_loader::types::ConfigError> {
        core_delete(&key).map_err(core_err)
    }
}

#[cfg(target_arch = "wasm32")]
export!(ConfigLoaderComponent);

// ── native helpers ────────────────────────────────────────────────────────────

pub use config_loader_core::{contains, delete, get, get_or_default, list_keys, set, ConfigError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        std::thread::spawn(|| {
            set("app.env", "production", false, 0).unwrap();
            assert_eq!(get("app.env").unwrap(), Some("production".to_string()));
            assert!(contains("app.env").unwrap());
            delete("app.env").unwrap();
            assert_eq!(get("app.env").unwrap(), None);
        })
        .join()
        .unwrap();
    }

    #[test]
    fn default_fallback() {
        std::thread::spawn(|| {
            let v = get_or_default("no-such", "default-val").unwrap();
            assert_eq!(v, "default-val");
        })
        .join()
        .unwrap();
    }
}
