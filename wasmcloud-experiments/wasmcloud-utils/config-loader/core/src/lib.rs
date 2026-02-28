use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub loaded_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    InvalidKey,
    NotFound,
    StorageError,
}

// ── Key validation ─────────────────────────────────────────────────────────────

fn validate_key(key: &str) -> Result<String, ConfigError> {
    let k = key.trim().to_string();
    if k.is_empty() {
        return Err(ConfigError::InvalidKey);
    }
    if !k.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':') {
        return Err(ConfigError::InvalidKey);
    }
    Ok(k)
}

// ── Thread-local state ────────────────────────────────────────────────────────

thread_local! {
    static STORE: RefCell<HashMap<String, ConfigEntry>> = RefCell::new(HashMap::new());
}

fn with_store<R>(f: impl FnOnce(&mut HashMap<String, ConfigEntry>) -> R) -> R {
    STORE.with(|s| f(&mut s.borrow_mut()))
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Set a configuration entry.
pub fn set(key: &str, value: &str, is_secret: bool, now_ms: u64) -> Result<(), ConfigError> {
    let k = validate_key(key)?;
    with_store(|m| {
        m.insert(k.clone(), ConfigEntry {
            key: k,
            value: value.to_string(),
            is_secret,
            loaded_at_ms: now_ms,
        });
        Ok(())
    })
}

/// Get the value for a key, or None if not found.
pub fn get(key: &str) -> Result<Option<String>, ConfigError> {
    let k = validate_key(key)?;
    Ok(with_store(|m| m.get(&k).map(|e| e.value.clone())))
}

/// Get the value for a key or return the default.
pub fn get_or_default(key: &str, default: &str) -> Result<String, ConfigError> {
    Ok(get(key)?.unwrap_or_else(|| default.to_string()))
}

/// Check whether a key is present.
pub fn contains(key: &str) -> Result<bool, ConfigError> {
    let k = validate_key(key)?;
    Ok(with_store(|m| m.contains_key(&k)))
}

/// List all keys (including secrets; masking is caller concern).
pub fn list_keys() -> Result<Vec<String>, ConfigError> {
    Ok(with_store(|m| m.keys().cloned().collect()))
}

/// Delete a key. Succeeds even if not found.
pub fn delete(key: &str) -> Result<(), ConfigError> {
    let k = validate_key(key)?;
    with_store(|m| {
        m.remove(&k);
        Ok(())
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
    fn test_set_and_get() {
        run(|| {
            set("db.host", "localhost", false, 0).unwrap();
            assert_eq!(get("db.host").unwrap(), Some("localhost".to_string()));
        });
    }

    #[test]
    fn test_get_nonexistent() {
        run(|| {
            assert_eq!(get("no-such-key").unwrap(), None);
        });
    }

    #[test]
    fn test_get_or_default() {
        run(|| {
            assert_eq!(get_or_default("missing-key", "fallback").unwrap(), "fallback");
            set("present", "value", false, 0).unwrap();
            assert_eq!(get_or_default("present", "fallback").unwrap(), "value");
        });
    }

    #[test]
    fn test_contains() {
        run(|| {
            assert!(!contains("before-set").unwrap());
            set("before-set", "x", false, 0).unwrap();
            assert!(contains("before-set").unwrap());
        });
    }

    #[test]
    fn test_list_keys() {
        run(|| {
            set("k1", "v1", false, 0).unwrap();
            set("k2", "v2", true, 0).unwrap();
            let keys = list_keys().unwrap();
            assert!(keys.contains(&"k1".to_string()));
            assert!(keys.contains(&"k2".to_string()));
        });
    }

    #[test]
    fn test_delete() {
        run(|| {
            set("del-me", "bye", false, 0).unwrap();
            delete("del-me").unwrap();
            assert_eq!(get("del-me").unwrap(), None);
        });
    }

    #[test]
    fn test_delete_nonexistent_is_ok() {
        run(|| {
            delete("never-existed").unwrap();
        });
    }

    #[test]
    fn test_invalid_key_empty() {
        run(|| {
            assert_eq!(set("", "v", false, 0).unwrap_err(), ConfigError::InvalidKey);
            assert_eq!(get("").unwrap_err(), ConfigError::InvalidKey);
        });
    }

    #[test]
    fn test_invalid_key_bad_chars() {
        run(|| {
            assert_eq!(set("bad key!", "v", false, 0).unwrap_err(), ConfigError::InvalidKey);
        });
    }

    #[test]
    fn test_overwrite() {
        run(|| {
            set("overwrite-key", "first", false, 0).unwrap();
            set("overwrite-key", "second", false, 1).unwrap();
            assert_eq!(get("overwrite-key").unwrap(), Some("second".to_string()));
        });
    }
}
