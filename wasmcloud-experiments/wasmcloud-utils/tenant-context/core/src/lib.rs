use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TenantInfo {
    pub id: String,
    pub display_name: Option<String>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TenantError {
    InvalidTenantId,
    NotFound,
    DuplicateTenant,
    InvalidKey,
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Tenant IDs: alphanumeric + `-` `_`.  No colons — colon is the scope separator.
fn validate_tenant_id(id: &str) -> Result<String, TenantError> {
    let s = id.trim().to_string();
    if s.is_empty() { return Err(TenantError::InvalidTenantId); }
    if !s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(TenantError::InvalidTenantId);
    }
    Ok(s)
}

/// Keys: same charset as tenant ID plus `.` `:` (for sub-namespacing within a tenant).
fn validate_key(key: &str) -> Result<String, TenantError> {
    let s = key.trim().to_string();
    if s.is_empty() { return Err(TenantError::InvalidKey); }
    if !s.chars().all(|c| c.is_alphanumeric() || "-_.:".contains(c)) {
        return Err(TenantError::InvalidKey);
    }
    Ok(s)
}

// ── Thread-local state ────────────────────────────────────────────────────────

thread_local! {
    static TENANTS: RefCell<HashMap<String, TenantInfo>> = RefCell::new(HashMap::new());
}

fn with_tenants<R>(f: impl FnOnce(&mut HashMap<String, TenantInfo>) -> R) -> R {
    TENANTS.with(|t| f(&mut t.borrow_mut()))
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Register a tenant. Returns DuplicateTenant if already present.
pub fn register(id: &str, display_name: Option<String>, now_ms: u64) -> Result<(), TenantError> {
    let tid = validate_tenant_id(id)?;
    with_tenants(|m| {
        if m.contains_key(&tid) { return Err(TenantError::DuplicateTenant); }
        m.insert(tid.clone(), TenantInfo { id: tid, display_name, created_at_ms: now_ms });
        Ok(())
    })
}

/// Produce a scoped key: `"{tenant_id}:{key}"`.
/// Validates both parts; does NOT require the tenant to be registered
/// (scoping is a pure string operation — registration is optional metadata).
pub fn scope(tenant_id: &str, key: &str) -> Result<String, TenantError> {
    let tid = validate_tenant_id(tenant_id)?;
    let k   = validate_key(key)?;
    Ok(format!("{}:{}", tid, k))
}

/// Extract the tenant ID from a scoped key (everything before the first `:`).
pub fn parse_tenant(scoped_key: &str) -> Result<String, TenantError> {
    let s = scoped_key.trim();
    let colon = s.find(':').ok_or(TenantError::InvalidKey)?;
    let tid = &s[..colon];
    validate_tenant_id(tid)
}

/// Retrieve tenant metadata.
pub fn get(id: &str) -> Result<TenantInfo, TenantError> {
    let tid = validate_tenant_id(id)?;
    with_tenants(|m| m.get(&tid).cloned().ok_or(TenantError::NotFound))
}

/// List all registered tenant IDs.
pub fn list_tenants() -> Result<Vec<String>, TenantError> {
    Ok(with_tenants(|m| m.keys().cloned().collect()))
}

/// Remove a tenant registration.
pub fn deregister(id: &str) -> Result<(), TenantError> {
    let tid = validate_tenant_id(id)?;
    with_tenants(|m| m.remove(&tid).ok_or(TenantError::NotFound).map(|_| ()))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_scope_produces_prefixed_key() {
        run(|| {
            let k = scope("acme", "user:alice").unwrap();
            assert_eq!(k, "acme:user:alice");
        });
    }

    #[test]
    fn test_scope_isolates_tenants() {
        run(|| {
            let k1 = scope("acme", "cache:session-1").unwrap();
            let k2 = scope("globex", "cache:session-1").unwrap();
            assert_ne!(k1, k2);
            assert!(k1.starts_with("acme:"));
            assert!(k2.starts_with("globex:"));
        });
    }

    #[test]
    fn test_parse_tenant_roundtrip() {
        run(|| {
            let scoped = scope("contoso", "lock:resource-a").unwrap();
            let tid = parse_tenant(&scoped).unwrap();
            assert_eq!(tid, "contoso");
        });
    }

    #[test]
    fn test_parse_tenant_no_colon() {
        run(|| {
            assert_eq!(parse_tenant("notscoped").unwrap_err(), TenantError::InvalidKey);
        });
    }

    #[test]
    fn test_register_and_get() {
        run(|| {
            register("tenant-a", Some("Tenant A".to_string()), 1000).unwrap();
            let info = get("tenant-a").unwrap();
            assert_eq!(info.id, "tenant-a");
            assert_eq!(info.display_name.as_deref(), Some("Tenant A"));
        });
    }

    #[test]
    fn test_duplicate_tenant() {
        run(|| {
            register("dup-t", None, 0).unwrap();
            assert_eq!(register("dup-t", None, 0).unwrap_err(), TenantError::DuplicateTenant);
        });
    }

    #[test]
    fn test_not_found() {
        run(|| {
            assert_eq!(get("ghost-tenant").unwrap_err(), TenantError::NotFound);
        });
    }

    #[test]
    fn test_deregister() {
        run(|| {
            register("remove-me", None, 0).unwrap();
            deregister("remove-me").unwrap();
            assert_eq!(get("remove-me").unwrap_err(), TenantError::NotFound);
        });
    }

    #[test]
    fn test_invalid_tenant_id() {
        run(|| {
            assert_eq!(scope("bad id!", "key").unwrap_err(), TenantError::InvalidTenantId);
            assert_eq!(scope("", "key").unwrap_err(), TenantError::InvalidTenantId);
            assert_eq!(scope("tenant:colon", "key").unwrap_err(), TenantError::InvalidTenantId);
        });
    }

    #[test]
    fn test_invalid_key() {
        run(|| {
            assert_eq!(scope("t1", "").unwrap_err(), TenantError::InvalidKey);
            assert_eq!(scope("t1", "bad key!").unwrap_err(), TenantError::InvalidKey);
        });
    }

    #[test]
    fn test_list_tenants() {
        run(|| {
            register("list-t1", None, 0).unwrap();
            register("list-t2", None, 0).unwrap();
            let all = list_tenants().unwrap();
            assert!(all.contains(&"list-t1".to_string()));
            assert!(all.contains(&"list-t2".to_string()));
        });
    }

    #[test]
    fn test_scope_no_registration_needed() {
        run(|| {
            // scope() is pure string work — no registration required
            let k = scope("unregistered-org", "rate-limit:user-42").unwrap();
            assert_eq!(k, "unregistered-org:rate-limit:user-42");
        });
    }
}
