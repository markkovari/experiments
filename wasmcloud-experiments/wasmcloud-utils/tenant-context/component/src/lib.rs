// WIT-based tenant-context component.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "tenant-context-component",
    path: "../../wit/wasmcloud-tenant-context",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use tenant_context_core::{
    deregister as core_deregister, get as core_get, list_tenants as core_list,
    parse_tenant as core_parse_tenant, register as core_register, scope as core_scope,
    TenantError as CoreError,
};

#[allow(dead_code)]
fn now_ms() -> u64 { 0 }

#[cfg(target_arch = "wasm32")]
fn core_err(e: CoreError) -> wasmcloud::tenant_context::types::TenantError {
    use wasmcloud::tenant_context::types::TenantError;
    match e {
        CoreError::InvalidTenantId  => TenantError::InvalidTenantId,
        CoreError::NotFound         => TenantError::NotFound,
        CoreError::DuplicateTenant  => TenantError::DuplicateTenant,
        CoreError::InvalidKey       => TenantError::InvalidKey,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_info(i: tenant_context_core::TenantInfo) -> wasmcloud::tenant_context::types::TenantInfo {
    wasmcloud::tenant_context::types::TenantInfo {
        id: i.id,
        display_name: i.display_name,
        created_at_ms: i.created_at_ms,
    }
}

#[cfg(target_arch = "wasm32")]
struct TenantContextComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::tenant_context::tenant_api::Guest for TenantContextComponent {
    fn register(id: String, display_name: Option<String>) -> Result<(), wasmcloud::tenant_context::types::TenantError> {
        core_register(&id, display_name, now_ms()).map_err(core_err)
    }
    fn scope(tenant_id: String, key: String) -> Result<String, wasmcloud::tenant_context::types::TenantError> {
        core_scope(&tenant_id, &key).map_err(core_err)
    }
    fn parse_tenant(scoped_key: String) -> Result<String, wasmcloud::tenant_context::types::TenantError> {
        core_parse_tenant(&scoped_key).map_err(core_err)
    }
    fn get(id: String) -> Result<wasmcloud::tenant_context::types::TenantInfo, wasmcloud::tenant_context::types::TenantError> {
        core_get(&id).map(wit_info).map_err(core_err)
    }
    fn list_tenants() -> Result<Vec<String>, wasmcloud::tenant_context::types::TenantError> {
        core_list().map_err(core_err)
    }
    fn deregister(id: String) -> Result<(), wasmcloud::tenant_context::types::TenantError> {
        core_deregister(&id).map_err(core_err)
    }
}

#[cfg(target_arch = "wasm32")]
export!(TenantContextComponent);

// ── native re-exports ─────────────────────────────────────────────────────────

pub use tenant_context_core::{
    deregister, get, list_tenants, parse_tenant, register, scope, TenantError, TenantInfo,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        std::thread::spawn(|| {
            register("comp-tenant", Some("Test Corp".to_string()), 0).unwrap();
            let info = get("comp-tenant").unwrap();
            assert_eq!(info.display_name.as_deref(), Some("Test Corp"));

            let k = scope("comp-tenant", "cache:item-1").unwrap();
            assert_eq!(k, "comp-tenant:cache:item-1");

            assert_eq!(parse_tenant(&k).unwrap(), "comp-tenant");
        }).join().unwrap();
    }

    #[test]
    fn scope_is_pure() {
        std::thread::spawn(|| {
            // scope works without any registered tenant
            let k = scope("any-org", "lock:res").unwrap();
            assert_eq!(k, "any-org:lock:res");
        }).join().unwrap();
    }
}
