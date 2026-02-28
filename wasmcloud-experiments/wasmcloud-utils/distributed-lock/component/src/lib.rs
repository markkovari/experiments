// WIT-based distributed-lock component.
// Targets the `distributed-lock-component` world defined in wit/wasmcloud-distributed-lock/distributed-lock.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "distributed-lock-component",
    path: "../../wit/wasmcloud-distributed-lock",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use distributed_lock_core::{
    acquire as core_acquire, extend as core_extend, get_lock as core_get_lock,
    is_locked as core_is_locked, release as core_release, LockError as CoreError,
};

#[allow(dead_code)]
fn now_ms() -> u64 {
    0
}

#[cfg(target_arch = "wasm32")]
fn core_err(e: CoreError) -> wasmcloud::distributed_lock::types::LockError {
    use wasmcloud::distributed_lock::types::LockError;
    match e {
        CoreError::AlreadyLocked => LockError::AlreadyLocked,
        CoreError::NotFound => LockError::NotFound,
        CoreError::InvalidToken => LockError::InvalidToken,
        CoreError::InvalidKey => LockError::InvalidKey,
        CoreError::StorageError => LockError::StorageError,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_lock(info: distributed_lock_core::LockInfo) -> wasmcloud::distributed_lock::types::LockInfo {
    wasmcloud::distributed_lock::types::LockInfo {
        key: info.key,
        owner_id: info.owner_id,
        acquired_at_ms: info.acquired_at_ms,
        expires_at_ms: info.expires_at_ms,
    }
}

#[cfg(target_arch = "wasm32")]
struct DistributedLockComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::distributed_lock::lock_api::Guest for DistributedLockComponent {
    fn acquire(
        key: String,
        owner_id: String,
        ttl_ms: u64,
    ) -> Result<String, wasmcloud::distributed_lock::types::LockError> {
        core_acquire(&key, &owner_id, ttl_ms, now_ms()).map_err(core_err)
    }

    fn release(
        key: String,
        token: String,
    ) -> Result<(), wasmcloud::distributed_lock::types::LockError> {
        core_release(&key, &token).map_err(core_err)
    }

    fn extend(
        key: String,
        token: String,
        ttl_ms: u64,
    ) -> Result<(), wasmcloud::distributed_lock::types::LockError> {
        core_extend(&key, &token, ttl_ms, now_ms()).map_err(core_err)
    }

    fn is_locked(key: String) -> Result<bool, wasmcloud::distributed_lock::types::LockError> {
        core_is_locked(&key, now_ms()).map_err(core_err)
    }

    fn get_lock(
        key: String,
    ) -> Result<wasmcloud::distributed_lock::types::LockInfo, wasmcloud::distributed_lock::types::LockError>
    {
        core_get_lock(&key).map(wit_lock).map_err(core_err)
    }
}

#[cfg(target_arch = "wasm32")]
export!(DistributedLockComponent);

// ── native helpers ────────────────────────────────────────────────────────────

pub use distributed_lock_core::{
    acquire, extend, get_lock, is_locked, release, LockError, LockInfo,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        std::thread::spawn(|| {
            let token = acquire("comp-lock-1", "svc-a", 500, 0).unwrap();
            assert!(is_locked("comp-lock-1", 100).unwrap());
            let info = get_lock("comp-lock-1").unwrap();
            assert_eq!(info.owner_id, "svc-a");
            release("comp-lock-1", &token).unwrap();
            assert!(!is_locked("comp-lock-1", 100).unwrap());
        })
        .join()
        .unwrap();
    }

    #[test]
    fn token_mismatch_rejected() {
        std::thread::spawn(|| {
            acquire("comp-lock-2", "svc-b", 1000, 0).unwrap();
            assert_eq!(release("comp-lock-2", "wrong").unwrap_err(), LockError::InvalidToken);
        })
        .join()
        .unwrap();
    }
}
