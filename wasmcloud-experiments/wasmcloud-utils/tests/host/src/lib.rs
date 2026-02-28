// Host-side tests: load each WASM component with wasmtime and exercise the
// exported WIT interfaces without a live wasmCloud/NATS runtime.

use anyhow::Result;
use wasmtime::{
    component::{Component, Linker, ResourceTable},
    Config, Engine, Store,
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

// ── Shared boilerplate ─────────────────────────────────────────────────────

fn make_engine() -> Engine {
    let mut cfg = Config::new();
    cfg.wasm_component_model(true);
    Engine::new(&cfg).expect("engine")
}

pub struct HostState {
    pub table: ResourceTable,
    pub ctx: WasiCtx,
    /// In-memory key-value store: bucket_name → (key → value)
    pub kv: std::collections::HashMap<String, std::collections::HashMap<String, Vec<u8>>>,
}

impl WasiView for HostState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

pub fn make_store(engine: &Engine) -> Store<HostState> {
    let ctx = WasiCtxBuilder::new().build();
    Store::new(
        engine,
        HostState {
            table: ResourceTable::new(),
            ctx,
            kv: std::collections::HashMap::new(),
        },
    )
}

/// Path to a pre-built WASM binary.  Tests skip gracefully if the binary is
/// absent (run `cargo build --release --target wasm32-wasip2 -p <crate>` first).
pub fn wasm_path(name: &str) -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()   // tests/
        .unwrap()
        .parent()   // workspace root
        .unwrap();
    root.join("target/wasm32-wasip2/release").join(name)
}

// ── Health-check ───────────────────────────────────────────────────────────

mod health {
    wasmtime::component::bindgen!({
        world: "health-check-component",
        path: "../../wit/wasmcloud-health-check",
    });
}

#[test]
fn test_health_check_component() -> Result<()> {
    let engine = make_engine();
    let path = wasm_path("health_check_component.wasm");
    if !path.exists() {
        eprintln!("SKIP: {:?} not found", path);
        return Ok(());
    }
    let component = Component::from_file(&engine, &path)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    let mut store = make_store(&engine);
    let bindings =
        health::HealthCheckComponent::instantiate(&mut store, &component, &linker)?;

    let api = bindings.wasmcloud_health_check_health_api();

    // register two probes
    api.call_register(&mut store, "db")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    api.call_register(&mut store, "redis")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // both healthy → Healthy
    api.call_record_result(&mut store, "db", true, None)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    api.call_record_result(&mut store, "redis", true, None)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    use health::wasmcloud::health_check::types::OverallStatus;
    let s = api.call_status(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(matches!(s, OverallStatus::Healthy), "expected Healthy, got {s:?}");

    // one failure → Degraded
    api.call_record_result(&mut store, "redis", false, Some("conn refused"))?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let s = api.call_status(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(matches!(s, OverallStatus::Degraded), "expected Degraded, got {s:?}");

    // both failures → Unhealthy
    api.call_record_result(&mut store, "db", false, Some("timeout"))?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let s = api.call_status(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(matches!(s, OverallStatus::Unhealthy));

    let probes = api.call_all_probes(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(probes.len(), 2);

    // deregister one
    api.call_deregister(&mut store, "db")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let probes = api.call_all_probes(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(probes.len(), 1);

    Ok(())
}

// ── Tracing ────────────────────────────────────────────────────────────────

mod tracing_wasm {
    wasmtime::component::bindgen!({
        world: "tracing-component",
        path: "../../wit/wasmcloud-tracing",
    });
}

#[test]
fn test_tracing_component() -> Result<()> {
    let engine = make_engine();
    let path = wasm_path("tracing_component.wasm");
    if !path.exists() {
        eprintln!("SKIP: {:?} not found", path);
        return Ok(());
    }
    let component = Component::from_file(&engine, &path)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    let mut store = make_store(&engine);
    let bindings =
        tracing_wasm::TracingComponent::instantiate(&mut store, &component, &linker)?;

    let api = bindings.wasmcloud_tracing_tracing_api();

    // start root span
    let root_id = api.call_start_span(&mut store, "request", None)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(!root_id.is_empty());

    // current span is root
    let cur = api.call_current_span(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(cur.as_deref(), Some(root_id.as_str()));

    // start child span
    let child_id = api.call_start_span(&mut store, "db-query", Some(&root_id))?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // add tags
    api.call_add_tag(&mut store, &child_id, "db.type", "postgres")?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    api.call_add_tag(&mut store, &child_id, "db.rows", "42")?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // get span
    let span = api.call_get_span(&mut store, &child_id)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(span.name, "db-query");
    assert_eq!(span.parent_id.as_deref(), Some(root_id.as_str()));
    assert!(span.tags.iter().any(|(k, v)| k == "db.type" && v == "postgres"));
    assert!(span.tags.iter().any(|(k, v)| k == "db.rows" && v == "42"));

    // active spans: root + child
    let active = api.call_active_spans(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(active.len(), 2);

    // end child
    api.call_end_span(&mut store, &child_id)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let active = api.call_active_spans(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(active.len(), 1);

    // end root
    api.call_end_span(&mut store, &root_id)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let cur = api.call_current_span(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(cur.is_none());

    Ok(())
}

// ── Cron ───────────────────────────────────────────────────────────────────

mod cron_wasm {
    wasmtime::component::bindgen!({
        world: "cron-component",
        path: "../../wit/wasmcloud-cron",
    });
}

#[test]
fn test_cron_component() -> Result<()> {
    let engine = make_engine();
    let path = wasm_path("cron_component.wasm");
    if !path.exists() {
        eprintln!("SKIP: {:?} not found", path);
        return Ok(());
    }
    let component = Component::from_file(&engine, &path)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    let mut store = make_store(&engine);
    let bindings =
        cron_wasm::CronComponent::instantiate(&mut store, &component, &linker)?;

    let api = bindings.wasmcloud_cron_cron_api();

    // register every-minute task
    api.call_register(&mut store, "heartbeat", "* * * * *")?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // duplicate should fail
    let err = api.call_register(&mut store, "heartbeat", "* * * * *")?;
    assert!(err.is_err(), "expected DuplicateTask error");

    // parse a specific expression
    let sched = api.call_parse(&mut store, "30 6 * * 1")?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(sched.minutes, vec![30u8]);
    assert_eq!(sched.hours, vec![6u8]);
    assert!(sched.days_of_month.is_empty(), "wildcard dom → empty");
    assert!(sched.days_of_week.contains(&1u8));

    // 2024-01-01 00:00:00 UTC = 1704067200000 ms — every-minute task is due
    let t = 1_704_067_200_000u64;
    let due = api.call_is_due(&mut store, "heartbeat", t)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(due);

    // tick
    api.call_tick(&mut store, "heartbeat", t)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let task = api.call_get_task(&mut store, "heartbeat")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(task.run_count, 1);
    assert_eq!(task.last_run_ms, Some(t));
    // next run should be t + 60_000
    assert_eq!(task.next_run_ms, Some(t + 60_000));

    // due-tasks at t: not due (already ticked)
    let due_list = api.call_due_tasks(&mut store, t)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(due_list.is_empty());

    // due-tasks one minute later: should fire
    let due_list = api.call_due_tasks(&mut store, t + 60_000)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(due_list.len(), 1);

    // disable
    api.call_set_enabled(&mut store, "heartbeat", false)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let due_list = api.call_due_tasks(&mut store, t + 60_000)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(due_list.is_empty(), "disabled task should not appear in due-tasks");

    // list
    let tasks = api.call_list_tasks(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(tasks.len(), 1);
    assert!(!tasks[0].enabled);

    // deregister
    api.call_deregister(&mut store, "heartbeat")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let tasks = api.call_list_tasks(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(tasks.is_empty());

    Ok(())
}

// ── Tenant-context ─────────────────────────────────────────────────────────

mod tenant_wasm {
    wasmtime::component::bindgen!({
        world: "tenant-context-component",
        path: "../../wit/wasmcloud-tenant-context",
    });
}

#[test]
fn test_tenant_context_component() -> Result<()> {
    let engine = make_engine();
    let path = wasm_path("tenant_context_component.wasm");
    if !path.exists() {
        eprintln!("SKIP: {:?} not found", path);
        return Ok(());
    }
    let component = Component::from_file(&engine, &path)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    let mut store = make_store(&engine);
    let bindings =
        tenant_wasm::TenantContextComponent::instantiate(&mut store, &component, &linker)?;

    let api = bindings.wasmcloud_tenant_context_tenant_api();

    // register
    api.call_register(&mut store, "acme", Some("ACME Corp"))?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    api.call_register(&mut store, "globex", None)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // duplicate
    let err = api.call_register(&mut store, "acme", None)?;
    assert!(err.is_err());

    // get
    let info = api.call_get(&mut store, "acme")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(info.display_name.as_deref(), Some("ACME Corp"));

    // scope
    let k1 = api.call_scope(&mut store, "acme", "cache:item")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let k2 = api.call_scope(&mut store, "globex", "cache:item")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(k1, "acme:cache:item");
    assert_eq!(k2, "globex:cache:item");
    assert_ne!(k1, k2);

    // parse-tenant
    assert_eq!(api.call_parse_tenant(&mut store, &k1)?.map_err(|e| anyhow::anyhow!("{e:?}"))?, "acme");
    assert_eq!(api.call_parse_tenant(&mut store, &k2)?.map_err(|e| anyhow::anyhow!("{e:?}"))?, "globex");

    // list-tenants
    let list = api.call_list_tenants(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(list.contains(&"acme".to_string()));
    assert!(list.contains(&"globex".to_string()));

    // deregister
    api.call_deregister(&mut store, "acme")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let err = api.call_get(&mut store, "acme")?;
    assert!(err.is_err());

    Ok(())
}

// ── Config-loader ──────────────────────────────────────────────────────────
// This component imports wasi:keyvalue/store.  We provide an in-memory stub.

mod config_wasm {
    wasmtime::component::bindgen!({
        world: "config-loader-component",
        path: "../../wit/wasmcloud-config-loader",
    });
}

impl config_wasm::wasi::keyvalue::store::Host for HostState {
    fn open(&mut self, name: String) -> anyhow::Result<u32, config_wasm::wasi::keyvalue::store::Error> {
        let idx = self.kv.len() as u32;
        self.kv.entry(name).or_default();
        Ok(idx)
    }

    fn get(&mut self, bucket: u32, key: String) -> anyhow::Result<Option<Vec<u8>>, config_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values().nth(bucket as usize)
            .ok_or_else(|| config_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        Ok(store.get(&key).cloned())
    }

    fn set(&mut self, bucket: u32, key: String, value: Vec<u8>) -> anyhow::Result<(), config_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values_mut().nth(bucket as usize)
            .ok_or_else(|| config_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        store.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, bucket: u32, key: String) -> anyhow::Result<(), config_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values_mut().nth(bucket as usize)
            .ok_or_else(|| config_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        store.remove(&key);
        Ok(())
    }

    fn exists(&mut self, bucket: u32, key: String) -> anyhow::Result<bool, config_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values().nth(bucket as usize)
            .ok_or_else(|| config_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        Ok(store.contains_key(&key))
    }

    fn list_keys(&mut self, bucket: u32, _cursor: Option<u64>) -> anyhow::Result<Vec<String>, config_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values().nth(bucket as usize)
            .ok_or_else(|| config_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        Ok(store.keys().cloned().collect())
    }
}

#[test]
fn test_config_loader_component() -> Result<()> {
    let engine = make_engine();
    let path = wasm_path("config_loader_component.wasm");
    if !path.exists() {
        eprintln!("SKIP: {:?} not found", path);
        return Ok(());
    }
    let component = Component::from_file(&engine, &path)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    config_wasm::wasi::keyvalue::store::add_to_linker(&mut linker, |s| s)?;

    let mut store = make_store(&engine);
    let bindings =
        config_wasm::ConfigLoaderComponent::instantiate(&mut store, &component, &linker)?;

    let api = bindings.wasmcloud_config_loader_config_api();

    // set a plain value
    api.call_set(&mut store, "db.host", "localhost", false)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // set a secret
    api.call_set(&mut store, "db.password", "s3cr3t", true)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // get
    let v = api.call_get(&mut store, "db.host")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(v.as_deref(), Some("localhost"));

    // get-or-default for missing key
    let v = api.call_get_or_default(&mut store, "missing.key", "default-val")?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(v, "default-val");

    // contains
    let ok = api.call_contains(&mut store, "db.host")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(ok);
    let nope = api.call_contains(&mut store, "nope")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(!nope);

    // list-keys
    let keys = api.call_list_keys(&mut store)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(keys.contains(&"db.host".to_string()));
    assert!(keys.contains(&"db.password".to_string()));

    // delete
    api.call_delete(&mut store, "db.host")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let v = api.call_get(&mut store, "db.host")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(v.is_none());

    Ok(())
}

// ── Distributed-lock ───────────────────────────────────────────────────────

mod lock_wasm {
    wasmtime::component::bindgen!({
        world: "distributed-lock-component",
        path: "../../wit/wasmcloud-distributed-lock",
    });
}

impl lock_wasm::wasi::keyvalue::store::Host for HostState {
    fn open(&mut self, name: String) -> anyhow::Result<u32, lock_wasm::wasi::keyvalue::store::Error> {
        let idx = self.kv.len() as u32;
        self.kv.entry(name).or_default();
        Ok(idx)
    }

    fn get(&mut self, bucket: u32, key: String) -> anyhow::Result<Option<Vec<u8>>, lock_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values().nth(bucket as usize)
            .ok_or_else(|| lock_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        Ok(store.get(&key).cloned())
    }

    fn set(&mut self, bucket: u32, key: String, value: Vec<u8>) -> anyhow::Result<(), lock_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values_mut().nth(bucket as usize)
            .ok_or_else(|| lock_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        store.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, bucket: u32, key: String) -> anyhow::Result<(), lock_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values_mut().nth(bucket as usize)
            .ok_or_else(|| lock_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        store.remove(&key);
        Ok(())
    }

    fn exists(&mut self, bucket: u32, key: String) -> anyhow::Result<bool, lock_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values().nth(bucket as usize)
            .ok_or_else(|| lock_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        Ok(store.contains_key(&key))
    }

    fn list_keys(&mut self, bucket: u32, _cursor: Option<u64>) -> anyhow::Result<Vec<String>, lock_wasm::wasi::keyvalue::store::Error> {
        let store = self.kv.values().nth(bucket as usize)
            .ok_or_else(|| lock_wasm::wasi::keyvalue::store::Error::Other(format!("bucket {bucket}")))?;
        Ok(store.keys().cloned().collect())
    }
}

#[test]
fn test_distributed_lock_component() -> Result<()> {
    let engine = make_engine();
    let path = wasm_path("distributed_lock_component.wasm");
    if !path.exists() {
        eprintln!("SKIP: {:?} not found", path);
        return Ok(());
    }
    let component = Component::from_file(&engine, &path)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    lock_wasm::wasi::keyvalue::store::add_to_linker(&mut linker, |s| s)?;

    let mut store = make_store(&engine);
    let bindings =
        lock_wasm::DistributedLockComponent::instantiate(&mut store, &component, &linker)?;

    let api = bindings.wasmcloud_distributed_lock_lock_api();

    let t = 1_000u64;

    // acquire
    let token = api.call_acquire(&mut store, "res-a", "owner-1", 60_000)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(!token.is_empty());

    // is-locked
    let locked = api.call_is_locked(&mut store, "res-a")?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(locked);

    // get-lock
    let info = api.call_get_lock(&mut store, "res-a")?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(info.owner_id, "owner-1");

    // second acquire fails
    let err = api.call_acquire(&mut store, "res-a", "owner-2", 60_000)?;
    assert!(err.is_err(), "expected AlreadyLocked");

    // extend with wrong token fails
    let err = api.call_extend(&mut store, "res-a", "bad-token", 30_000)?;
    assert!(err.is_err());

    // extend with correct token
    api.call_extend(&mut store, "res-a", &token, 120_000)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // release with wrong token fails
    let err = api.call_release(&mut store, "res-a", "bad-token")?;
    assert!(err.is_err());

    // release with correct token
    api.call_release(&mut store, "res-a", &token)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // now not locked
    let locked = api.call_is_locked(&mut store, "res-a")?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(!locked);

    // re-acquire succeeds
    let _token2 = api.call_acquire(&mut store, "res-a", "owner-2", 60_000)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    let _ = t; // suppress unused warning
    Ok(())
}

// ── Batch component ────────────────────────────────────────────────────────

mod batch_wasm {
    wasmtime::component::bindgen!({
        world: "batch-component",
        path: "../../wit/wasmcloud-batch",
    });
}

#[test]
fn test_batch_component() -> anyhow::Result<()> {
    let engine = make_engine();
    let path = wasm_path("batch_component.wasm");
    if !path.exists() {
        eprintln!("SKIP: {:?} not found", path);
        return Ok(());
    }
    let component = wasmtime::component::Component::from_file(&engine, &path)?;
    let mut linker: wasmtime::component::Linker<HostState> = wasmtime::component::Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    let mut store = make_store(&engine);
    let bindings =
        batch_wasm::BatchComponent::instantiate(&mut store, &component, &linker)?;

    let api = bindings.wasmcloud_batch_batch_api();

    // open a window: max 3 items, no age limit
    api.call_open(&mut store, "events", 3, 0)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    // not due yet
    assert!(!api.call_is_due(&mut store, "events", 0)?.map_err(|e| anyhow::anyhow!("{e:?}"))?);

    // enqueue 3 items
    use batch_wasm::wasmcloud::batch::types::BatchItem;
    for i in 0u32..3 {
        api.call_enqueue(&mut store, "events", &BatchItem {
            id: format!("evt-{i}"),
            payload: vec![i as u8],
            enqueued_at_ms: 1000 + i as u64,
        })?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    }

    // now due (size reached)
    assert!(api.call_is_due(&mut store, "events", 0)?.map_err(|e| anyhow::anyhow!("{e:?}"))?);

    // flush
    let items = api.call_flush(&mut store, "events", 5000)?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].id, "evt-0");

    // record results
    use batch_wasm::wasmcloud::batch::types::ItemResult;
    let summary = api.call_record_results(&mut store, "events", &items.iter().map(|it| ItemResult {
        id: it.id.clone(),
        ok: true,
        detail: None,
    }).collect::<Vec<_>>())?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(summary.total, 3);
    assert_eq!(summary.succeeded, 3);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.flushed_at_ms, 5000);

    // test age-based trigger with a second window
    api.call_open(&mut store, "aged", 0, 200)? // 200 ms max age
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    api.call_enqueue(&mut store, "aged", &BatchItem { id: "a".into(), payload: vec![], enqueued_at_ms: 1000 })?
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(!api.call_is_due(&mut store, "aged", 1100)?.map_err(|e| anyhow::anyhow!("{e:?}"))?);
    assert!(api.call_is_due(&mut store, "aged", 1200)?.map_err(|e| anyhow::anyhow!("{e:?}"))?);

    // due-batches
    let due = api.call_due_batches(&mut store, 1200)?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert!(due.contains(&"aged".to_string()));

    // discard
    api.call_discard(&mut store, "aged")?.map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(api.call_pending_count(&mut store, "aged")?.map_err(|e| anyhow::anyhow!("{e:?}"))?, 0);

    Ok(())
}
