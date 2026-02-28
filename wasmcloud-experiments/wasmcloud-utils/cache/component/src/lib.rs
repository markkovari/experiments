// WIT-based cache component.
// Targets the `cache-component` world defined in wit/wasmcloud-cache/cache.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "cache-component",
    path: "../../wit/wasmcloud-cache",
    generate_all,
});

use cache_core::{
    delete as core_delete, exists as core_exists, flush as core_flush, get as core_get,
    set as core_set, CacheError as CoreError,
};

// ---- current time stub -------------------------------------------------------

/// In a real WASM component use `wasi:clocks/wall-clock` or monotonic-clock.
/// For native `cargo check` we fall back to a simple counter.
#[allow(dead_code)]
fn now_ms() -> u64 {
    // Stub: return 0 — callers that need real time pass it explicitly in tests.
    0
}

// ---- type conversions -------------------------------------------------------

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::cache::types::CacheError {
    use wasmcloud::cache::types::CacheError;
    match e {
        CoreError::NotInitialized => CacheError::NotInitialized,
        CoreError::StorageError => CacheError::StorageError,
        CoreError::SerializationError => CacheError::SerializationError,
        CoreError::InvalidKey => CacheError::InvalidKey,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_entry(e: cache_core::CacheEntry) -> wasmcloud::cache::types::CachedEntry {
    wasmcloud::cache::types::CachedEntry {
        value: e.value,
        created_at_ms: e.created_at_ms,
        expires_at_ms: e.expires_at_ms,
        hit_count: e.hit_count,
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct CacheComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::cache::cacher::Guest for CacheComponent {
    fn get(
        key: String,
    ) -> Result<Option<wasmcloud::cache::types::CachedEntry>, wasmcloud::cache::types::CacheError>
    {
        core_get(&key, now_ms()).map(|opt| opt.map(wit_entry)).map_err(core_error)
    }

    fn set(
        key: String,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
    ) -> Result<(), wasmcloud::cache::types::CacheError> {
        core_set(&key, value, ttl_ms, now_ms()).map_err(core_error)
    }

    fn delete(key: String) -> Result<(), wasmcloud::cache::types::CacheError> {
        core_delete(&key).map_err(core_error)
    }

    fn exists(key: String) -> Result<bool, wasmcloud::cache::types::CacheError> {
        core_exists(&key, now_ms()).map_err(core_error)
    }

    fn flush() -> Result<u64, wasmcloud::cache::types::CacheError> {
        core_flush(now_ms()).map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(CacheComponent);

// ---- native helpers (cargo check / tests) -----------------------------------

pub fn cache_set(key: &str, value: Vec<u8>, ttl_ms: Option<u64>, now_ms: u64) -> Result<(), CoreError> {
    core_set(key, value, ttl_ms, now_ms)
}

pub fn cache_get(key: &str, now_ms: u64) -> Result<Option<cache_core::CacheEntry>, CoreError> {
    core_get(key, now_ms)
}

pub fn cache_delete(key: &str) -> Result<(), CoreError> {
    core_delete(key)
}

pub fn cache_exists(key: &str, now_ms: u64) -> Result<bool, CoreError> {
    core_exists(key, now_ms)
}

pub fn cache_flush(now_ms: u64) -> Result<u64, CoreError> {
    core_flush(now_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        cache_set("comp_key", b"hello".to_vec(), None, 0).unwrap();
        let entry = cache_get("comp_key", 0).unwrap().unwrap();
        assert_eq!(entry.value, b"hello");
        cache_delete("comp_key").unwrap();
        assert!(cache_get("comp_key", 0).unwrap().is_none());
    }

    #[test]
    fn ttl_expiry_component() {
        cache_set("comp_ttl", b"data".to_vec(), Some(100), 1000).unwrap();
        assert!(cache_get("comp_ttl", 1050).unwrap().is_some());
        assert!(cache_get("comp_ttl", 1100).unwrap().is_none());
    }
}
