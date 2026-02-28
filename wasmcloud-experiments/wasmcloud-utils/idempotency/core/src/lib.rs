use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum KeyStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    pub key: String,
    pub status: KeyStatus,
    pub response: Option<String>,
    pub created_at_ms: u64,
    pub expires_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyError {
    InvalidKey,
    StorageError,
    NotFound,
}

#[derive(Debug)]
pub struct CheckResult {
    pub is_new: bool,
    pub cached_record: Option<IdempotencyRecord>,
}

// ── Key validation ─────────────────────────────────────────────────────────────

fn validate_key(key: &str) -> Result<String, IdempotencyError> {
    let k = key.trim().to_string();
    if k.is_empty() {
        return Err(IdempotencyError::InvalidKey);
    }
    if !k.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':') {
        return Err(IdempotencyError::InvalidKey);
    }
    Ok(k)
}

// ── In-memory store (test stub) ───────────────────────────────────────────────

thread_local! {
    static STORE: RefCell<HashMap<String, IdempotencyRecord>> = RefCell::new(HashMap::new());
}

fn with_store<R>(f: impl FnOnce(&mut HashMap<String, IdempotencyRecord>) -> R) -> R {
    STORE.with(|s| f(&mut s.borrow_mut()))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Atomically check/create an idempotency key.
/// Returns is_new=true and inserts a Pending record if the key is new.
/// Returns is_new=false with the existing record if already seen.
/// Expired records are evicted and treated as new.
pub fn check_or_create(key: &str, ttl_ms: Option<u64>, now_ms: u64) -> Result<CheckResult, IdempotencyError> {
    let k = validate_key(key)?;
    with_store(|m| {
        if let Some(existing) = m.get(&k) {
            // Evict if expired
            if let Some(exp) = existing.expires_at_ms {
                if now_ms >= exp {
                    m.remove(&k);
                } else {
                    return Ok(CheckResult { is_new: false, cached_record: Some(existing.clone()) });
                }
            } else {
                return Ok(CheckResult { is_new: false, cached_record: Some(existing.clone()) });
            }
        }
        let record = IdempotencyRecord {
            key: k.clone(),
            status: KeyStatus::Pending,
            response: None,
            created_at_ms: now_ms,
            expires_at_ms: ttl_ms.map(|t| now_ms + t),
        };
        m.insert(k, record);
        Ok(CheckResult { is_new: true, cached_record: None })
    })
}

/// Mark a key as completed with an optional response payload.
pub fn complete(key: &str, response: Option<String>) -> Result<(), IdempotencyError> {
    let k = validate_key(key)?;
    with_store(|m| {
        let rec = m.get_mut(&k).ok_or(IdempotencyError::NotFound)?;
        rec.status = KeyStatus::Completed;
        rec.response = response;
        Ok(())
    })
}

/// Mark a key as failed with an optional error payload.
pub fn fail(key: &str, error_payload: Option<String>) -> Result<(), IdempotencyError> {
    let k = validate_key(key)?;
    with_store(|m| {
        let rec = m.get_mut(&k).ok_or(IdempotencyError::NotFound)?;
        rec.status = KeyStatus::Failed;
        rec.response = error_payload;
        Ok(())
    })
}

/// Retrieve the current record for a key.
pub fn get(key: &str) -> Result<IdempotencyRecord, IdempotencyError> {
    let k = validate_key(key)?;
    with_store(|m| m.get(&k).cloned().ok_or(IdempotencyError::NotFound))
}

/// Delete a key record.
pub fn delete(key: &str) -> Result<(), IdempotencyError> {
    let k = validate_key(key)?;
    with_store(|m| { m.remove(&k); Ok(()) })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_new_key_is_new() {
        run(|| {
            let r = check_or_create("pay:001", None, 0).unwrap();
            assert!(r.is_new);
            assert!(r.cached_record.is_none());
        });
    }

    #[test]
    fn test_existing_key_not_new() {
        run(|| {
            check_or_create("pay:002", None, 0).unwrap();
            let r = check_or_create("pay:002", None, 1).unwrap();
            assert!(!r.is_new);
            let rec = r.cached_record.unwrap();
            assert_eq!(rec.status, KeyStatus::Pending);
        });
    }

    #[test]
    fn test_complete() {
        run(|| {
            check_or_create("pay:003", None, 0).unwrap();
            complete("pay:003", Some(r#"{"id":42}"#.to_string())).unwrap();
            let rec = get("pay:003").unwrap();
            assert_eq!(rec.status, KeyStatus::Completed);
            assert_eq!(rec.response, Some(r#"{"id":42}"#.to_string()));
        });
    }

    #[test]
    fn test_fail() {
        run(|| {
            check_or_create("pay:004", None, 0).unwrap();
            fail("pay:004", Some("insufficient funds".to_string())).unwrap();
            let rec = get("pay:004").unwrap();
            assert_eq!(rec.status, KeyStatus::Failed);
        });
    }

    #[test]
    fn test_completed_key_returned_on_second_call() {
        run(|| {
            check_or_create("pay:005", None, 0).unwrap();
            complete("pay:005", Some("ok".to_string())).unwrap();
            let r = check_or_create("pay:005", None, 1).unwrap();
            assert!(!r.is_new);
            assert_eq!(r.cached_record.unwrap().status, KeyStatus::Completed);
        });
    }

    #[test]
    fn test_ttl_expiry_treats_as_new() {
        run(|| {
            check_or_create("pay:006", Some(100), 0).unwrap();
            // now_ms=200 > expires_at=100 → evicted, treated as new
            let r = check_or_create("pay:006", Some(100), 200).unwrap();
            assert!(r.is_new);
        });
    }

    #[test]
    fn test_ttl_not_expired() {
        run(|| {
            check_or_create("pay:007", Some(1000), 0).unwrap();
            let r = check_or_create("pay:007", Some(1000), 500).unwrap();
            assert!(!r.is_new);
        });
    }

    #[test]
    fn test_delete() {
        run(|| {
            check_or_create("pay:008", None, 0).unwrap();
            delete("pay:008").unwrap();
            assert_eq!(get("pay:008").unwrap_err(), IdempotencyError::NotFound);
        });
    }

    #[test]
    fn test_invalid_key_empty() {
        run(|| {
            assert_eq!(check_or_create("", None, 0).unwrap_err(), IdempotencyError::InvalidKey);
        });
    }

    #[test]
    fn test_invalid_key_bad_chars() {
        run(|| {
            assert_eq!(check_or_create("bad key!", None, 0).unwrap_err(), IdempotencyError::InvalidKey);
        });
    }

    #[test]
    fn test_complete_not_found() {
        run(|| {
            assert_eq!(complete("nonexistent", None).unwrap_err(), IdempotencyError::NotFound);
        });
    }
}
