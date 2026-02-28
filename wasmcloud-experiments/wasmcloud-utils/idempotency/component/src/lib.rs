#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "idempotency-component",
    path: "../../wit/wasmcloud-idempotency",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use idempotency_core::{
    check_or_create as core_check_or_create, complete as core_complete,
    delete as core_delete, fail as core_fail, get as core_get,
    IdempotencyError as CoreError, KeyStatus as CoreStatus,
};

#[allow(dead_code)]
fn now_ms() -> u64 { 0 }

// ── type conversions (wasm32 only) ────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::idempotency::types::IdempotencyError {
    use wasmcloud::idempotency::types::IdempotencyError;
    match e {
        CoreError::InvalidKey    => IdempotencyError::InvalidKey,
        CoreError::StorageError  => IdempotencyError::StorageError,
        CoreError::NotFound      => IdempotencyError::NotFound,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_status(s: CoreStatus) -> wasmcloud::idempotency::types::KeyStatus {
    use wasmcloud::idempotency::types::KeyStatus;
    match s {
        CoreStatus::Pending   => KeyStatus::Pending,
        CoreStatus::Completed => KeyStatus::Completed,
        CoreStatus::Failed    => KeyStatus::Failed,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_record(r: idempotency_core::IdempotencyRecord) -> wasmcloud::idempotency::types::IdempotencyRecord {
    wasmcloud::idempotency::types::IdempotencyRecord {
        key:           r.key,
        status:        wit_status(r.status),
        response:      r.response,
        created_at_ms: r.created_at_ms,
        expires_at_ms: r.expires_at_ms,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_check(c: idempotency_core::CheckResult) -> wasmcloud::idempotency::types::CheckResult {
    wasmcloud::idempotency::types::CheckResult {
        is_new:        c.is_new,
        cached_record: c.cached_record.map(wit_record),
    }
}

// ── WIT guest implementation ──────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
struct IdempotencyComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::idempotency::idempotency_api::Guest for IdempotencyComponent {
    fn check_or_create(
        key: String,
        ttl_ms: Option<u64>,
    ) -> Result<wasmcloud::idempotency::types::CheckResult, wasmcloud::idempotency::types::IdempotencyError> {
        core_check_or_create(&key, ttl_ms, now_ms()).map(wit_check).map_err(core_error)
    }

    fn complete(
        key: String,
        response: Option<String>,
    ) -> Result<(), wasmcloud::idempotency::types::IdempotencyError> {
        core_complete(&key, response).map_err(core_error)
    }

    fn fail(
        key: String,
        error_payload: Option<String>,
    ) -> Result<(), wasmcloud::idempotency::types::IdempotencyError> {
        core_fail(&key, error_payload).map_err(core_error)
    }

    fn get(
        key: String,
    ) -> Result<wasmcloud::idempotency::types::IdempotencyRecord, wasmcloud::idempotency::types::IdempotencyError> {
        core_get(&key).map(wit_record).map_err(core_error)
    }

    fn delete(key: String) -> Result<(), wasmcloud::idempotency::types::IdempotencyError> {
        core_delete(&key).map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(IdempotencyComponent);

// ── native helpers ────────────────────────────────────────────────────────────

pub use idempotency_core::{
    check_or_create, complete, delete, fail, get,
    CheckResult, IdempotencyError, IdempotencyRecord, KeyStatus,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn roundtrip_new_and_complete() {
        run(|| {
            let r = check_or_create("comp:001", None, 0).unwrap();
            assert!(r.is_new);
            complete("comp:001", Some("done".to_string())).unwrap();
            let rec = get("comp:001").unwrap();
            assert_eq!(rec.status, KeyStatus::Completed);
        });
    }

    #[test]
    fn roundtrip_idempotent_second_call() {
        run(|| {
            check_or_create("comp:002", None, 0).unwrap();
            complete("comp:002", Some("ok".to_string())).unwrap();
            let r = check_or_create("comp:002", None, 1).unwrap();
            assert!(!r.is_new);
            assert_eq!(r.cached_record.unwrap().status, KeyStatus::Completed);
        });
    }
}
