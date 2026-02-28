use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LockInfo {
    pub key: String,
    pub owner_id: String,
    pub acquired_at_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LockError {
    AlreadyLocked,
    NotFound,
    InvalidToken,
    InvalidKey,
    StorageError,
}

#[derive(Debug, Clone)]
struct LockEntry {
    owner_id: String,
    token: String,
    acquired_at_ms: u64,
    expires_at_ms: u64,
}

// ── Key validation ─────────────────────────────────────────────────────────────

fn validate_key(key: &str) -> Result<String, LockError> {
    let k = key.trim().to_string();
    if k.is_empty() {
        return Err(LockError::InvalidKey);
    }
    if !k.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':') {
        return Err(LockError::InvalidKey);
    }
    Ok(k)
}

/// Deterministic token: no OS entropy.
fn make_token(key: &str, owner_id: &str, acquired_at_ms: u64) -> String {
    format!("{}:{}:{}", key, owner_id, acquired_at_ms)
}

// ── Thread-local state ────────────────────────────────────────────────────────

thread_local! {
    static LOCKS: RefCell<HashMap<String, LockEntry>> = RefCell::new(HashMap::new());
}

fn with_locks<R>(f: impl FnOnce(&mut HashMap<String, LockEntry>) -> R) -> R {
    LOCKS.with(|l| f(&mut l.borrow_mut()))
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Acquire a lock. Evicts expired locks first. Returns a token on success.
pub fn acquire(key: &str, owner_id: &str, ttl_ms: u64, now_ms: u64) -> Result<String, LockError> {
    let k = validate_key(key)?;
    with_locks(|m| {
        // Evict expired lock if present
        if let Some(existing) = m.get(&k) {
            if now_ms < existing.expires_at_ms {
                return Err(LockError::AlreadyLocked);
            }
            m.remove(&k);
        }
        let token = make_token(&k, owner_id, now_ms);
        m.insert(k.clone(), LockEntry {
            owner_id: owner_id.to_string(),
            token: token.clone(),
            acquired_at_ms: now_ms,
            expires_at_ms: now_ms + ttl_ms,
        });
        Ok(token)
    })
}

/// Release a lock. Returns InvalidToken if token doesn't match.
pub fn release(key: &str, token: &str) -> Result<(), LockError> {
    let k = validate_key(key)?;
    with_locks(|m| {
        let entry = m.get(&k).ok_or(LockError::NotFound)?;
        if entry.token != token {
            return Err(LockError::InvalidToken);
        }
        m.remove(&k);
        Ok(())
    })
}

/// Extend a lock's TTL. Returns InvalidToken if token doesn't match.
pub fn extend(key: &str, token: &str, ttl_ms: u64, now_ms: u64) -> Result<(), LockError> {
    let k = validate_key(key)?;
    with_locks(|m| {
        let entry = m.get_mut(&k).ok_or(LockError::NotFound)?;
        if entry.token != token {
            return Err(LockError::InvalidToken);
        }
        entry.expires_at_ms = now_ms + ttl_ms;
        Ok(())
    })
}

/// Check if a key is currently locked (false if expired).
pub fn is_locked(key: &str, now_ms: u64) -> Result<bool, LockError> {
    let k = validate_key(key)?;
    Ok(with_locks(|m| {
        m.get(&k).map_or(false, |e| now_ms < e.expires_at_ms)
    }))
}

/// Get lock info for a key.
pub fn get_lock(key: &str) -> Result<LockInfo, LockError> {
    let k = validate_key(key)?;
    with_locks(|m| {
        m.get(&k)
            .map(|e| LockInfo {
                key: k.clone(),
                owner_id: e.owner_id.clone(),
                acquired_at_ms: e.acquired_at_ms,
                expires_at_ms: e.expires_at_ms,
            })
            .ok_or(LockError::NotFound)
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_acquire_and_release() {
        run(|| {
            let token = acquire("res-1", "owner-a", 1000, 0).unwrap();
            assert!(!token.is_empty());
            assert!(is_locked("res-1", 500).unwrap());
            release("res-1", &token).unwrap();
            assert!(!is_locked("res-1", 500).unwrap());
        });
    }

    #[test]
    fn test_already_locked() {
        run(|| {
            acquire("res-2", "owner-a", 1000, 0).unwrap();
            assert_eq!(acquire("res-2", "owner-b", 1000, 500).unwrap_err(), LockError::AlreadyLocked);
        });
    }

    #[test]
    fn test_expired_lock_evicted_on_acquire() {
        run(|| {
            acquire("res-3", "owner-a", 100, 0).unwrap();
            // Lock expires at 100ms; acquire at 200ms should succeed
            let token2 = acquire("res-3", "owner-b", 500, 200).unwrap();
            assert!(!token2.is_empty());
        });
    }

    #[test]
    fn test_is_locked_false_after_expiry() {
        run(|| {
            acquire("res-4", "owner-a", 50, 0).unwrap();
            assert!(!is_locked("res-4", 50).unwrap()); // expired at 50
            assert!(!is_locked("res-4", 100).unwrap()); // still expired
        });
    }

    #[test]
    fn test_release_invalid_token() {
        run(|| {
            acquire("res-5", "owner-a", 1000, 0).unwrap();
            assert_eq!(release("res-5", "wrong-token").unwrap_err(), LockError::InvalidToken);
        });
    }

    #[test]
    fn test_extend() {
        run(|| {
            let token = acquire("res-6", "owner-a", 100, 0).unwrap();
            extend("res-6", &token, 1000, 50).unwrap();
            // Now expires at 50+1000=1050, still locked at 500
            assert!(is_locked("res-6", 500).unwrap());
        });
    }

    #[test]
    fn test_extend_invalid_token() {
        run(|| {
            acquire("res-7", "owner-a", 1000, 0).unwrap();
            assert_eq!(extend("res-7", "bad-token", 1000, 0).unwrap_err(), LockError::InvalidToken);
        });
    }

    #[test]
    fn test_get_lock() {
        run(|| {
            acquire("res-8", "owner-x", 500, 100).unwrap();
            let info = get_lock("res-8").unwrap();
            assert_eq!(info.owner_id, "owner-x");
            assert_eq!(info.acquired_at_ms, 100);
            assert_eq!(info.expires_at_ms, 600);
        });
    }

    #[test]
    fn test_release_not_found() {
        run(|| {
            assert_eq!(release("nonexistent", "tok").unwrap_err(), LockError::NotFound);
        });
    }

    #[test]
    fn test_invalid_key() {
        run(|| {
            assert_eq!(acquire("", "owner", 1000, 0).unwrap_err(), LockError::InvalidKey);
            assert_eq!(acquire("bad key!", "owner", 1000, 0).unwrap_err(), LockError::InvalidKey);
        });
    }

    #[test]
    fn test_token_deterministic() {
        run(|| {
            let token1 = acquire("res-9", "owner-a", 1000, 42).unwrap();
            release("res-9", &token1).unwrap();
            // Re-acquire at same time produces same token
            let token2 = acquire("res-9", "owner-a", 1000, 42).unwrap();
            assert_eq!(token1, token2);
        });
    }
}
