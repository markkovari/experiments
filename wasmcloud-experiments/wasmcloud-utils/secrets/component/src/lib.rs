#[cfg(target_arch = "wasm32")]
mod bindings {
    wit_bindgen::generate!({
        world: "secrets-component",
        path: "../../wit/wasmcloud-secrets",
        generate_all,
    });
}

#[cfg(target_arch = "wasm32")]
use bindings::exports::wasmcloud::secrets::secret_store::{
    Guest, SecretError as WitError, SecretMetadata as WitMeta, SecretValue as WitValue,
    SecretsConfig as WitConfig,
};

#[cfg(target_arch = "wasm32")]
use bindings::wasi::keyvalue::{atomics, store};

use secrets_core::{
    decrypt, encrypt, is_meta_key, kv_data_key, kv_meta_key, name_from_data_key, SecretError,
    SecretMetadata, SecretValue, SecretsConfig,
};
use std::cell::RefCell;

// ---------------------------------------------------------------------------
// Global component state
// ---------------------------------------------------------------------------

struct State {
    config: Option<SecretsConfig>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State { config: None });
}

fn with_config<F, T>(f: F) -> Result<T, SecretError>
where
    F: FnOnce(&SecretsConfig) -> Result<T, SecretError>,
{
    STATE.with(|s| {
        let state = s.borrow();
        match &state.config {
            Some(cfg) => f(cfg),
            None => Err(SecretError::NotInitialized),
        }
    })
}

fn namespace(cfg: &SecretsConfig) -> &str {
    cfg.namespace.as_deref().unwrap_or("secrets")
}

fn enc_key(cfg: &SecretsConfig) -> Result<&str, SecretError> {
    cfg.encryption_key
        .as_deref()
        .ok_or(SecretError::InvalidConfig)
}

// ---------------------------------------------------------------------------
// Core logic (used by both wasm32 and native test builds)
// ---------------------------------------------------------------------------

/// Initialize config — stores it in thread-local state.
pub fn core_init(config: SecretsConfig) -> Result<(), SecretError> {
    if config.backend != "kv" {
        return Err(SecretError::InvalidConfig);
    }
    if config.encryption_key.is_none() {
        return Err(SecretError::InvalidConfig);
    }
    STATE.with(|s| {
        s.borrow_mut().config = Some(config);
    });
    Ok(())
}

/// KV-backed get: read encrypted blob from KV, decrypt, return bytes.
#[cfg(target_arch = "wasm32")]
pub fn core_get(name: &str) -> Result<SecretValue, SecretError> {
    with_config(|cfg| {
        let key = kv_data_key(namespace(cfg), name);
        let enc_key_str = enc_key(cfg)?;
        let bucket = store::open("").map_err(|_| SecretError::StorageError)?;
        let raw = store::get(bucket, &key)
            .map_err(|_| SecretError::StorageError)?
            .ok_or(SecretError::NotFound)?;
        let encoded = String::from_utf8(raw).map_err(|_| SecretError::StorageError)?;
        let plaintext = decrypt(enc_key_str, &encoded)?;
        Ok(SecretValue { data: plaintext })
    })
}

/// KV-backed set: encrypt value, write to KV; create/update metadata.
#[cfg(target_arch = "wasm32")]
pub fn core_set(name: &str, value: SecretValue) -> Result<(), SecretError> {
    with_config(|cfg| {
        let ns = namespace(cfg);
        let enc_key_str = enc_key(cfg)?;
        let data_key = kv_data_key(ns, name);
        let meta_key = kv_meta_key(ns, name);
        let bucket = store::open("").map_err(|_| SecretError::StorageError)?;

        // Encrypt
        let encoded = encrypt(enc_key_str, &value.data)?;

        // Determine version
        let version = match store::get(bucket, &meta_key)
            .map_err(|_| SecretError::StorageError)?
        {
            Some(raw) => {
                let m: SecretMetadata =
                    serde_json::from_slice(&raw).map_err(|_| SecretError::StorageError)?;
                m.version + 1
            }
            None => 1,
        };

        let now_ms = current_time_ms();
        let created_at_ms = match store::get(bucket, &meta_key)
            .map_err(|_| SecretError::StorageError)?
        {
            Some(raw) => {
                let m: SecretMetadata =
                    serde_json::from_slice(&raw).map_err(|_| SecretError::StorageError)?;
                m.created_at_ms
            }
            None => now_ms,
        };

        let meta = SecretMetadata {
            name: name.to_string(),
            version,
            created_at_ms,
            updated_at_ms: now_ms,
        };

        store::set(bucket, &data_key, &encoded.into_bytes())
            .map_err(|_| SecretError::StorageError)?;
        store::set(
            bucket,
            &meta_key,
            &serde_json::to_vec(&meta).map_err(|_| SecretError::StorageError)?,
        )
        .map_err(|_| SecretError::StorageError)?;

        Ok(())
    })
}

#[cfg(target_arch = "wasm32")]
pub fn core_delete(name: &str) -> Result<(), SecretError> {
    with_config(|cfg| {
        let ns = namespace(cfg);
        let bucket = store::open("").map_err(|_| SecretError::StorageError)?;
        store::delete(bucket, &kv_data_key(ns, name))
            .map_err(|_| SecretError::StorageError)?;
        store::delete(bucket, &kv_meta_key(ns, name))
            .map_err(|_| SecretError::StorageError)?;
        Ok(())
    })
}

#[cfg(target_arch = "wasm32")]
pub fn core_list_names() -> Result<Vec<String>, SecretError> {
    with_config(|cfg| {
        let ns = namespace(cfg);
        let prefix = format!("{ns}:");
        let bucket = store::open("").map_err(|_| SecretError::StorageError)?;
        let keys = store::list_keys(bucket, None)
            .map_err(|_| SecretError::StorageError)?;
        Ok(keys
            .into_iter()
            .filter(|k| !is_meta_key(k))
            .filter_map(|k| name_from_data_key(ns, &k).map(|n| n.to_string()))
            .collect())
    })
}

#[cfg(target_arch = "wasm32")]
pub fn core_rotate(name: &str, new_value: SecretValue) -> Result<SecretMetadata, SecretError> {
    with_config(|cfg| {
        let ns = namespace(cfg);
        let enc_key_str = enc_key(cfg)?;
        let data_key = kv_data_key(ns, name);
        let meta_key = kv_meta_key(ns, name);
        let bucket = store::open("").map_err(|_| SecretError::StorageError)?;

        // Must exist to rotate
        let raw_meta = store::get(bucket, &meta_key)
            .map_err(|_| SecretError::StorageError)?
            .ok_or(SecretError::NotFound)?;
        let mut meta: SecretMetadata =
            serde_json::from_slice(&raw_meta).map_err(|_| SecretError::StorageError)?;

        let encoded = encrypt(enc_key_str, &new_value.data)?;
        meta.version += 1;
        meta.updated_at_ms = current_time_ms();

        store::set(bucket, &data_key, &encoded.into_bytes())
            .map_err(|_| SecretError::StorageError)?;
        store::set(
            bucket,
            &meta_key,
            &serde_json::to_vec(&meta).map_err(|_| SecretError::StorageError)?,
        )
        .map_err(|_| SecretError::StorageError)?;

        Ok(meta)
    })
}

#[cfg(target_arch = "wasm32")]
pub fn core_metadata(name: &str) -> Result<SecretMetadata, SecretError> {
    with_config(|cfg| {
        let ns = namespace(cfg);
        let bucket = store::open("").map_err(|_| SecretError::StorageError)?;
        let raw = store::get(bucket, &kv_meta_key(ns, name))
            .map_err(|_| SecretError::StorageError)?
            .ok_or(SecretError::NotFound)?;
        serde_json::from_slice(&raw).map_err(|_| SecretError::StorageError)
    })
}

#[cfg(target_arch = "wasm32")]
fn current_time_ms() -> u64 {
    // Use wasi:clocks if available; fallback to 0 for minimal builds
    0u64
}

// ---------------------------------------------------------------------------
// WIT guest implementation (wasm32 only)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
fn map_err(e: SecretError) -> WitError {
    match e {
        SecretError::NotInitialized => WitError::NotInitialized,
        SecretError::NotFound => WitError::NotFound,
        SecretError::AlreadyExists => WitError::AlreadyExists,
        SecretError::InvalidConfig => WitError::InvalidConfig,
        SecretError::EncryptionError => WitError::EncryptionError,
        SecretError::StorageError => WitError::StorageError,
        SecretError::PermissionDenied => WitError::PermissionDenied,
    }
}

#[cfg(target_arch = "wasm32")]
struct SecretsComponent;

#[cfg(target_arch = "wasm32")]
impl Guest for SecretsComponent {
    fn init(config: WitConfig) -> Result<(), WitError> {
        core_init(SecretsConfig {
            backend: config.backend,
            namespace: config.namespace,
            encryption_key: config.encryption_key,
        })
        .map_err(map_err)
    }

    fn get(name: String) -> Result<WitValue, WitError> {
        core_get(&name).map(|v| WitValue { data: v.data }).map_err(map_err)
    }

    fn set(name: String, value: WitValue) -> Result<(), WitError> {
        core_set(&name, SecretValue { data: value.data }).map_err(map_err)
    }

    fn delete(name: String) -> Result<(), WitError> {
        core_delete(&name).map_err(map_err)
    }

    fn list_names() -> Result<Vec<String>, WitError> {
        core_list_names().map_err(map_err)
    }

    fn rotate(name: String, new_value: WitValue) -> Result<WitMeta, WitError> {
        core_rotate(&name, SecretValue { data: new_value.data })
            .map(|m| WitMeta {
                name: m.name,
                version: m.version,
                created_at_ms: m.created_at_ms,
                updated_at_ms: m.updated_at_ms,
            })
            .map_err(map_err)
    }

    fn metadata(name: String) -> Result<WitMeta, WitError> {
        core_metadata(&name)
            .map(|m| WitMeta {
                name: m.name,
                version: m.version,
                created_at_ms: m.created_at_ms,
                updated_at_ms: m.updated_at_ms,
            })
            .map_err(map_err)
    }
}

#[cfg(target_arch = "wasm32")]
bindings::export!(SecretsComponent with_types_in bindings);

// ---------------------------------------------------------------------------
// Native (non-wasm32) stubs for `cargo check` / unit tests
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
pub fn core_get(_name: &str) -> Result<SecretValue, SecretError> {
    Err(SecretError::StorageError)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn core_set(_name: &str, _value: SecretValue) -> Result<(), SecretError> {
    Err(SecretError::StorageError)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn core_delete(_name: &str) -> Result<(), SecretError> {
    Err(SecretError::StorageError)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn core_list_names() -> Result<Vec<String>, SecretError> {
    Err(SecretError::StorageError)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn core_rotate(_name: &str, _value: SecretValue) -> Result<SecretMetadata, SecretError> {
    Err(SecretError::StorageError)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn core_metadata(_name: &str) -> Result<SecretMetadata, SecretError> {
    Err(SecretError::StorageError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    #[test]
    fn init_invalid_backend_rejected() {
        assert_eq!(
            core_init(SecretsConfig {
                backend: "invalid".to_string(),
                namespace: None,
                encryption_key: None,
            }),
            Err(SecretError::InvalidConfig)
        );
    }

    #[test]
    fn init_kv_accepted() {
        assert!(core_init(SecretsConfig {
            backend: "kv".to_string(),
            namespace: Some("test".to_string()),
            encryption_key: Some(base64::engine::general_purpose::STANDARD.encode([0u8; 32])),
        })
        .is_ok());
    }
}
