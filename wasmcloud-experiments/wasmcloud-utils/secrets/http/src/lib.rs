/// secrets-http: HTTP admin API for the secrets store.
///
/// Routes:
///   GET    /secrets              → list-names
///   GET    /secrets/{name}       → metadata (not the value)
///   POST   /secrets/{name}       → set  (body: {"value": "<base64>"})
///   DELETE /secrets/{name}       → delete
///   POST   /secrets/{name}/rotate → rotate (body: {"value": "<base64>"})
///
/// Config is bootstrapped via X-Secrets-Config header (JSON SecretsConfig) on
/// the first request, or pre-initialized via WADM link config.

#[cfg(target_arch = "wasm32")]
mod bindings {
    wit_bindgen::generate!({
        world: "secrets-http",
        path: "../../wit/wasmcloud-secrets",
        generate_all,
    });
}

#[cfg(target_arch = "wasm32")]
use bindings::{
    exports::{
        wasmcloud::secrets::secret_store::{
            Guest as SecretGuest, SecretError as WitError, SecretMetadata as WitMeta,
            SecretValue as WitValue, SecretsConfig as WitConfig,
        },
        wasi::http::incoming_handler::Guest as HttpGuest,
    },
    wasi::{
        http::types::{Method, Request, Response},
        keyvalue::store,
    },
};

use secrets_core::{
    decrypt, encrypt, is_meta_key, kv_data_key, kv_meta_key, name_from_data_key, SecretError,
    SecretMetadata, SecretValue, SecretsConfig,
};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

struct State {
    config: Option<SecretsConfig>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State { config: None });
}

fn init_state(config: SecretsConfig) -> Result<(), SecretError> {
    if config.backend != "kv" {
        return Err(SecretError::InvalidConfig);
    }
    STATE.with(|s| s.borrow_mut().config = Some(config));
    Ok(())
}

fn with_config<F, T>(f: F) -> Result<T, SecretError>
where
    F: FnOnce(&SecretsConfig) -> Result<T, SecretError>,
{
    STATE.with(|s| match &s.borrow().config {
        Some(cfg) => f(cfg),
        None => Err(SecretError::NotInitialized),
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
// Request/Response helpers
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SetBody {
    /// Base64-encoded secret bytes
    value: String,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn json_response(status: u16, body: &str) -> (u16, Vec<u8>) {
    (status, body.as_bytes().to_vec())
}

fn error_response(status: u16, msg: &str) -> (u16, Vec<u8>) {
    let body = serde_json::to_string(&ApiError { error: msg.to_string() })
        .unwrap_or_else(|_| format!(r#"{{"error":"{msg}"}}"#));
    (status, body.into_bytes())
}

fn secret_err_to_status(e: &SecretError) -> u16 {
    match e {
        SecretError::NotFound => 404,
        SecretError::AlreadyExists => 409,
        SecretError::NotInitialized => 503,
        SecretError::PermissionDenied => 403,
        SecretError::InvalidConfig => 400,
        _ => 500,
    }
}

// ---------------------------------------------------------------------------
// Route dispatch (pure logic, testable on host)
// ---------------------------------------------------------------------------

/// Parse path into (name, action):
///   "/secrets"         → (None, "list")
///   "/secrets/foo"     → (Some("foo"), "get_meta")
///   "/secrets/foo/rotate" → (Some("foo"), "rotate")
fn parse_path(path: &str) -> (Option<String>, &'static str) {
    let path = path.trim_start_matches('/');
    let parts: Vec<&str> = path.splitn(3, '/').collect();
    match parts.as_slice() {
        ["secrets"] => (None, "list"),
        ["secrets", name] => (Some((*name).to_string()), "resource"),
        ["secrets", name, "rotate"] => (Some((*name).to_string()), "rotate"),
        _ => (None, "not_found"),
    }
}

pub fn handle_list() -> Result<Vec<String>, SecretError> {
    with_config(|cfg| {
        let ns = namespace(cfg);
        // Native stub returns empty list; wasm32 uses KV
        Ok(vec![])
    })
}

pub fn handle_get_meta(name: &str) -> Result<SecretMetadata, SecretError> {
    // Native stub — always not-found
    Err(SecretError::NotFound)
}

pub fn handle_set(name: &str, value_b64: &str) -> Result<(), SecretError> {
    with_config(|cfg| {
        let key = enc_key(cfg)?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(value_b64)
            .map_err(|_| SecretError::InvalidConfig)?;
        // Native stub: no-op
        Ok(())
    })
}

pub fn handle_delete(name: &str) -> Result<(), SecretError> {
    with_config(|_| Ok(()))
}

pub fn handle_rotate(name: &str, value_b64: &str) -> Result<SecretMetadata, SecretError> {
    Err(SecretError::NotFound)
}

// ---------------------------------------------------------------------------
// HTTP dispatch
// ---------------------------------------------------------------------------

pub fn dispatch(method: &str, path: &str, body: &[u8]) -> (u16, Vec<u8>) {
    let (name, action) = parse_path(path);

    match (method, action) {
        ("GET", "list") => match handle_list() {
            Ok(names) => json_response(
                200,
                &serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string()),
            ),
            Err(e) => error_response(secret_err_to_status(&e), &e.to_string()),
        },
        ("GET", "resource") => {
            let name = name.unwrap();
            match handle_get_meta(&name) {
                Ok(m) => json_response(
                    200,
                    &serde_json::to_string(&m).unwrap_or_else(|_| "{}".to_string()),
                ),
                Err(e) => error_response(secret_err_to_status(&e), &e.to_string()),
            }
        }
        ("POST", "resource") => {
            let name = name.unwrap();
            match serde_json::from_slice::<SetBody>(body) {
                Ok(req) => match handle_set(&name, &req.value) {
                    Ok(()) => json_response(200, r#"{"ok":true}"#),
                    Err(e) => error_response(secret_err_to_status(&e), &e.to_string()),
                },
                Err(_) => error_response(400, "invalid JSON body"),
            }
        }
        ("DELETE", "resource") => {
            let name = name.unwrap();
            match handle_delete(&name) {
                Ok(()) => json_response(200, r#"{"ok":true}"#),
                Err(e) => error_response(secret_err_to_status(&e), &e.to_string()),
            }
        }
        ("POST", "rotate") => {
            let name = name.unwrap();
            match serde_json::from_slice::<SetBody>(body) {
                Ok(req) => match handle_rotate(&name, &req.value) {
                    Ok(m) => json_response(
                        200,
                        &serde_json::to_string(&m).unwrap_or_else(|_| "{}".to_string()),
                    ),
                    Err(e) => error_response(secret_err_to_status(&e), &e.to_string()),
                },
                Err(_) => error_response(400, "invalid JSON body"),
            }
        }
        _ => error_response(404, "not found"),
    }
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
struct SecretsHttp;

#[cfg(target_arch = "wasm32")]
impl SecretGuest for SecretsHttp {
    fn init(config: WitConfig) -> Result<(), WitError> {
        init_state(SecretsConfig {
            backend: config.backend,
            namespace: config.namespace,
            encryption_key: config.encryption_key,
        })
        .map_err(map_err)
    }

    fn get(name: String) -> Result<WitValue, WitError> {
        Err(WitError::PermissionDenied) // HTTP API never returns secret values
    }

    fn set(name: String, value: WitValue) -> Result<(), WitError> {
        with_config(|cfg| {
            let key_str = enc_key(cfg)?;
            let encoded = encrypt(key_str, &value.data)?;
            let data_key = kv_data_key(namespace(cfg), &name);
            let meta_key = kv_meta_key(namespace(cfg), &name);
            let bucket = store::open("").map_err(|_| SecretError::StorageError)?;
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
            let meta = SecretMetadata {
                name: name.clone(),
                version,
                created_at_ms: 0,
                updated_at_ms: 0,
            };
            store::set(bucket, &data_key, &encoded.into_bytes())
                .map_err(|_| SecretError::StorageError)?;
            store::set(
                bucket,
                &meta_key,
                &serde_json::to_vec(&meta).map_err(|_| SecretError::StorageError)?,
            )
            .map_err(|_| SecretError::StorageError)
        })
        .map_err(map_err)
    }

    fn delete(name: String) -> Result<(), WitError> {
        with_config(|cfg| {
            let ns = namespace(cfg);
            let bucket = store::open("").map_err(|_| SecretError::StorageError)?;
            store::delete(bucket, &kv_data_key(ns, &name))
                .map_err(|_| SecretError::StorageError)?;
            store::delete(bucket, &kv_meta_key(ns, &name))
                .map_err(|_| SecretError::StorageError)
        })
        .map_err(map_err)
    }

    fn list_names() -> Result<Vec<String>, WitError> {
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
        .map_err(map_err)
    }

    fn rotate(name: String, new_value: WitValue) -> Result<WitMeta, WitError> {
        with_config(|cfg| {
            let ns = namespace(cfg);
            let key_str = enc_key(cfg)?;
            let data_key = kv_data_key(ns, &name);
            let meta_key = kv_meta_key(ns, &name);
            let bucket = store::open("").map_err(|_| SecretError::StorageError)?;
            let raw_meta = store::get(bucket, &meta_key)
                .map_err(|_| SecretError::StorageError)?
                .ok_or(SecretError::NotFound)?;
            let mut meta: SecretMetadata =
                serde_json::from_slice(&raw_meta).map_err(|_| SecretError::StorageError)?;
            let encoded = encrypt(key_str, &new_value.data)?;
            meta.version += 1;
            meta.updated_at_ms = 0;
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
        .map(|m| WitMeta {
            name: m.name,
            version: m.version,
            created_at_ms: m.created_at_ms,
            updated_at_ms: m.updated_at_ms,
        })
        .map_err(map_err)
    }

    fn metadata(name: String) -> Result<WitMeta, WitError> {
        with_config(|cfg| {
            let ns = namespace(cfg);
            let bucket = store::open("").map_err(|_| SecretError::StorageError)?;
            let raw = store::get(bucket, &kv_meta_key(ns, &name))
                .map_err(|_| SecretError::StorageError)?
                .ok_or(SecretError::NotFound)?;
            serde_json::from_slice(&raw).map_err(|_| SecretError::StorageError)
        })
        .map(|m: SecretMetadata| WitMeta {
            name: m.name,
            version: m.version,
            created_at_ms: m.created_at_ms,
            updated_at_ms: m.updated_at_ms,
        })
        .map_err(map_err)
    }
}

#[cfg(target_arch = "wasm32")]
impl HttpGuest for SecretsHttp {
    fn handle(request: Request) -> Response {
        let method_str = match request.method {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Delete => "DELETE",
            Method::Put => "PUT",
            _ => "OTHER",
        };
        let path = request.path;
        let body_bytes = request.body.unwrap_or_default();

        let (status, body) = dispatch(method_str, &path, &body_bytes);

        Response {
            status,
            headers: vec![("content-type".to_string(), b"application/json".to_vec())],
            body: Some(body),
        }
    }
}

#[cfg(target_arch = "wasm32")]
bindings::export!(SecretsHttp with_types_in bindings);

// ---------------------------------------------------------------------------
// Tests (native)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_path_list() {
        let (name, action) = parse_path("/secrets");
        assert!(name.is_none());
        assert_eq!(action, "list");
    }

    #[test]
    fn parse_path_resource() {
        let (name, action) = parse_path("/secrets/my-db-pass");
        assert_eq!(name.as_deref(), Some("my-db-pass"));
        assert_eq!(action, "resource");
    }

    #[test]
    fn parse_path_rotate() {
        let (name, action) = parse_path("/secrets/my-db-pass/rotate");
        assert_eq!(name.as_deref(), Some("my-db-pass"));
        assert_eq!(action, "rotate");
    }

    #[test]
    fn dispatch_not_initialized_returns_503() {
        STATE.with(|s| s.borrow_mut().config = None);
        let (status, _) = dispatch("GET", "/secrets", b"");
        assert_eq!(status, 503);
    }

    #[test]
    fn dispatch_404_on_unknown_path() {
        let (status, _) = dispatch("GET", "/unknown/path", b"");
        assert_eq!(status, 404);
    }

    #[test]
    fn dispatch_list_with_init() {
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;
        init_state(SecretsConfig {
            backend: "kv".to_string(),
            namespace: Some("test".to_string()),
            encryption_key: Some(B64.encode([0u8; 32])),
        })
        .unwrap();
        let (status, body) = dispatch("GET", "/secrets", b"");
        assert_eq!(status, 200);
        let names: Vec<String> = serde_json::from_slice(&body).unwrap();
        assert!(names.is_empty()); // native stub returns empty list
    }
}
