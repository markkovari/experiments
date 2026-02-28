use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FlagValue {
    Boolean(bool),
    Text(String),
    Integer(i64),
}

#[derive(Debug, Clone)]
pub struct Flag {
    pub key: String,
    pub value: FlagValue,
    pub description: Option<String>,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlagError {
    NotInitialized,
    NotFound,
    InvalidKey,
    StorageError,
    TypeMismatch,
}

// ── Key validation ────────────────────────────────────────────────────────────

fn validate_key(key: &str) -> Result<String, FlagError> {
    let k = key.trim().to_lowercase();
    if k.is_empty() {
        return Err(FlagError::InvalidKey);
    }
    // Allow alphanumeric, hyphens, underscores, and dots only.
    if !k.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(FlagError::InvalidKey);
    }
    Ok(k)
}

// ── In-memory store (test stub) ───────────────────────────────────────────────
// NOTE: In a deployed WASM component, flag state is persisted via
// wasi:keyvalue/store imported in the feature-flags-component world.
// This thread_local is a test-only stub — same pattern as cache-core.

thread_local! {
    static FLAGS: RefCell<HashMap<String, Flag>> = RefCell::new(HashMap::new());
}

fn with_flags<R>(f: impl FnOnce(&mut HashMap<String, Flag>) -> R) -> R {
    FLAGS.with(|m| f(&mut m.borrow_mut()))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return `true` when a boolean flag exists and is enabled.
/// Returns `false` (not an error) when the flag is absent.
pub fn is_enabled(key: &str) -> Result<bool, FlagError> {
    let k = validate_key(key)?;
    with_flags(|m| match m.get(&k) {
        Some(f) => match &f.value {
            FlagValue::Boolean(v) => Ok(*v),
            _ => Err(FlagError::TypeMismatch),
        },
        None => Ok(false),
    })
}

/// Retrieve the full flag record.
pub fn get(key: &str) -> Result<Flag, FlagError> {
    let k = validate_key(key)?;
    with_flags(|m| m.get(&k).cloned().ok_or(FlagError::NotFound))
}

/// Create or overwrite a flag.
pub fn set(
    key: &str,
    value: FlagValue,
    description: Option<String>,
    now_ms: u64,
) -> Result<(), FlagError> {
    let k = validate_key(key)?;
    with_flags(|m| {
        m.insert(k.clone(), Flag { key: k, value, description, updated_at_ms: now_ms });
        Ok(())
    })
}

/// Delete a flag. Succeeds even if the key does not exist.
pub fn delete(key: &str) -> Result<(), FlagError> {
    let k = validate_key(key)?;
    with_flags(|m| { m.remove(&k); Ok(()) })
}

/// Return all flags currently stored.
pub fn list() -> Result<Vec<Flag>, FlagError> {
    with_flags(|m| Ok(m.values().cloned().collect()))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_boolean_flag_enabled() {
        run(|| {
            set("new-ui", FlagValue::Boolean(true), None, 0).unwrap();
            assert!(is_enabled("new-ui").unwrap());
        });
    }

    #[test]
    fn test_boolean_flag_disabled() {
        run(|| {
            set("old-ui", FlagValue::Boolean(false), None, 0).unwrap();
            assert!(!is_enabled("old-ui").unwrap());
        });
    }

    #[test]
    fn test_missing_flag_returns_false_not_error() {
        run(|| {
            assert!(!is_enabled("does-not-exist").unwrap());
        });
    }

    #[test]
    fn test_text_flag() {
        run(|| {
            set("api-version", FlagValue::Text("v2".to_string()), Some("API version".to_string()), 100).unwrap();
            let f = get("api-version").unwrap();
            assert_eq!(f.value, FlagValue::Text("v2".to_string()));
            assert_eq!(f.description, Some("API version".to_string()));
            assert_eq!(f.updated_at_ms, 100);
        });
    }

    #[test]
    fn test_integer_flag() {
        run(|| {
            set("rollout-pct", FlagValue::Integer(25), None, 0).unwrap();
            let f = get("rollout-pct").unwrap();
            assert_eq!(f.value, FlagValue::Integer(25));
        });
    }

    #[test]
    fn test_type_mismatch_is_enabled_on_non_bool() {
        run(|| {
            set("limit", FlagValue::Integer(10), None, 0).unwrap();
            assert_eq!(is_enabled("limit").unwrap_err(), FlagError::TypeMismatch);
        });
    }

    #[test]
    fn test_delete() {
        run(|| {
            set("temp", FlagValue::Boolean(true), None, 0).unwrap();
            delete("temp").unwrap();
            assert_eq!(get("temp").unwrap_err(), FlagError::NotFound);
            // Delete non-existent is a no-op
            delete("temp").unwrap();
        });
    }

    #[test]
    fn test_overwrite() {
        run(|| {
            set("feat", FlagValue::Boolean(false), None, 0).unwrap();
            set("feat", FlagValue::Boolean(true), None, 50).unwrap();
            let f = get("feat").unwrap();
            assert_eq!(f.value, FlagValue::Boolean(true));
            assert_eq!(f.updated_at_ms, 50);
        });
    }

    #[test]
    fn test_list() {
        run(|| {
            set("a", FlagValue::Boolean(true), None, 0).unwrap();
            set("b", FlagValue::Text("x".to_string()), None, 0).unwrap();
            let flags = list().unwrap();
            assert_eq!(flags.len(), 2);
        });
    }

    #[test]
    fn test_invalid_key() {
        run(|| {
            assert_eq!(set("", FlagValue::Boolean(true), None, 0).unwrap_err(), FlagError::InvalidKey);
            assert_eq!(set("  ", FlagValue::Boolean(true), None, 0).unwrap_err(), FlagError::InvalidKey);
            assert_eq!(set("bad key!", FlagValue::Boolean(true), None, 0).unwrap_err(), FlagError::InvalidKey);
            assert_eq!(get("").unwrap_err(), FlagError::InvalidKey);
        });
    }

    #[test]
    fn test_key_normalised_lowercase() {
        run(|| {
            set("MY-FLAG", FlagValue::Boolean(true), None, 0).unwrap();
            // Lookup with original case works — normalised to lowercase
            assert!(is_enabled("MY-FLAG").unwrap());
            assert!(is_enabled("my-flag").unwrap());
        });
    }
}
