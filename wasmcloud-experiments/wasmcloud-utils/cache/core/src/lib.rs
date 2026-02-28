use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub value: Vec<u8>,
    pub created_at_ms: u64,
    pub expires_at_ms: Option<u64>,
    pub hit_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheError {
    NotInitialized,
    StorageError,
    SerializationError,
    InvalidKey,
}

/// Normalise a cache key: lowercase and trim whitespace.
pub fn normalise_key(key: &str) -> Result<String, CacheError> {
    let k = key.trim().to_lowercase();
    if k.is_empty() {
        return Err(CacheError::InvalidKey);
    }
    Ok(k)
}

// NOTE: Thread-local in-memory store — test stub only.
// Deployed WASM components use wasi:keyvalue imported via the WIT world.
struct CacheState {
    store: HashMap<String, CacheEntry>,
}

thread_local! {
    static CACHE: RefCell<CacheState> = RefCell::new(CacheState {
        store: HashMap::new(),
    });
}

fn with_cache<R>(f: impl FnOnce(&mut CacheState) -> R) -> R {
    CACHE.with(|c| f(&mut c.borrow_mut()))
}

/// Return a cached entry if it exists and has not expired.
/// Expired entries are removed and `None` is returned.
pub fn get(key: &str, now_ms: u64) -> Result<Option<CacheEntry>, CacheError> {
    let key = normalise_key(key)?;
    with_cache(|c| {
        if let Some(entry) = c.store.get(&key) {
            if let Some(exp) = entry.expires_at_ms {
                if now_ms >= exp {
                    c.store.remove(&key);
                    return Ok(None);
                }
            }
            let mut updated = entry.clone();
            updated.hit_count += 1;
            c.store.insert(key, updated.clone());
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    })
}

/// Insert or overwrite an entry. `ttl_ms` is relative to `now_ms`.
pub fn set(key: &str, value: Vec<u8>, ttl_ms: Option<u64>, now_ms: u64) -> Result<(), CacheError> {
    let key = normalise_key(key)?;
    let expires_at_ms = ttl_ms.map(|t| now_ms + t);
    with_cache(|c| {
        c.store.insert(
            key,
            CacheEntry { value, created_at_ms: now_ms, expires_at_ms, hit_count: 0 },
        );
        Ok(())
    })
}

/// Remove a key. Succeeds even if the key does not exist.
pub fn delete(key: &str) -> Result<(), CacheError> {
    let key = normalise_key(key)?;
    with_cache(|c| {
        c.store.remove(&key);
        Ok(())
    })
}

/// Return true if the key exists and has not expired.
pub fn exists(key: &str, now_ms: u64) -> Result<bool, CacheError> {
    Ok(get(key, now_ms)?.is_some())
}

/// Remove all expired entries and return the eviction count.
pub fn flush(now_ms: u64) -> Result<u64, CacheError> {
    with_cache(|c| {
        let before = c.store.len() as u64;
        c.store.retain(|_, entry| match entry.expires_at_ms {
            Some(exp) => now_ms < exp,
            None => true,
        });
        Ok(before - c.store.len() as u64)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each test uses distinct keys to avoid cross-test pollution with thread_local.

    #[test]
    fn test_set_and_get() {
        set("hello", b"world".to_vec(), None, 1000).unwrap();
        let entry = get("hello", 2000).unwrap().unwrap();
        assert_eq!(entry.value, b"world");
        assert_eq!(entry.hit_count, 1);
    }

    #[test]
    fn test_key_normalisation() {
        set("  UPPER  ", b"v".to_vec(), None, 0).unwrap();
        assert!(get("upper", 0).unwrap().is_some());
    }

    #[test]
    fn test_invalid_key() {
        assert_eq!(set("", b"v".to_vec(), None, 0).unwrap_err(), CacheError::InvalidKey);
        assert_eq!(set("   ", b"v".to_vec(), None, 0).unwrap_err(), CacheError::InvalidKey);
        assert_eq!(get("", 0).unwrap_err(), CacheError::InvalidKey);
    }

    #[test]
    fn test_ttl_expiry() {
        set("ttl_key", b"data".to_vec(), Some(100), 1000).unwrap();
        // Before expiry
        assert!(get("ttl_key", 1050).unwrap().is_some());
        // At or after expiry
        assert!(get("ttl_key", 1100).unwrap().is_none());
    }

    #[test]
    fn test_delete() {
        set("del_key", b"bye".to_vec(), None, 0).unwrap();
        assert!(get("del_key", 0).unwrap().is_some());
        delete("del_key").unwrap();
        assert!(get("del_key", 0).unwrap().is_none());
    }

    #[test]
    fn test_exists() {
        set("exists_key", b"yes".to_vec(), None, 0).unwrap();
        assert!(exists("exists_key", 0).unwrap());
        assert!(!exists("nonexistent_key", 0).unwrap());
    }

    #[test]
    fn test_flush_evicts_expired() {
        set("flush_a", b"1".to_vec(), Some(50), 0).unwrap();
        set("flush_b", b"2".to_vec(), Some(50), 0).unwrap();
        set("flush_c", b"3".to_vec(), None, 0).unwrap();
        let evicted = flush(100).unwrap();
        assert_eq!(evicted, 2);
        assert!(get("flush_c", 100).unwrap().is_some());
    }

    #[test]
    fn test_overwrite() {
        set("ow_key", b"first".to_vec(), None, 0).unwrap();
        set("ow_key", b"second".to_vec(), None, 0).unwrap();
        let entry = get("ow_key", 0).unwrap().unwrap();
        assert_eq!(entry.value, b"second");
    }
}
