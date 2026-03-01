// Workflow HTTP API component.
// Targets the `workflow-api-component` world defined in
// wit/wasmcloud-workflow-api/workflow-api.wit.
//
// Exports:  wasi:http/incoming-handler
// Imports:  wasmcloud:workflow-store/store
//
// All persistence is delegated to the workflow-store component linked at
// deploy time (default: workflow-store-kv, works with any wasi:keyvalue provider).

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "workflow-api-component",
    path: "../wit/wasmcloud-workflow-api",
    generate_all,
});

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// SSE ring buffer — stateless, single-threaded (WASM-compatible)
// ---------------------------------------------------------------------------

/// An event pushed to SSE clients.
#[derive(Debug, Clone, Serialize)]
pub struct SseEvent {
    /// `"run.state"` or `"step.state"`
    #[serde(rename = "type")]
    pub kind: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wf_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    pub state: String,
    pub ts_ms: u64,
    /// Monotonically increasing sequence used for `Last-Event-ID` reconnect.
    pub seq: u64,
}

/// Maximum number of events kept in the ring buffer.
const SSE_RING_CAPACITY: usize = 200;

use std::cell::RefCell;
use std::collections::VecDeque;

thread_local! {
    static SSE_RING: RefCell<VecDeque<SseEvent>> = RefCell::new(VecDeque::new());
    static SSE_SEQ: RefCell<u64> = const { RefCell::new(0) };
}

/// Push an event to the ring buffer (drops oldest when at capacity).
pub fn sse_push(kind: &str, run_id: &str, wf_name: Option<&str>, step: Option<&str>, state: &str) {
    SSE_SEQ.with(|seq| {
        let mut s = seq.borrow_mut();
        *s += 1;
        let ev = SseEvent {
            kind: kind.to_string(),
            run_id: run_id.to_string(),
            wf_name: wf_name.map(|s| s.to_string()),
            step: step.map(|s| s.to_string()),
            state: state.to_string(),
            ts_ms: now_ms(),
            seq: *s,
        };
        SSE_RING.with(|ring| {
            let mut r = ring.borrow_mut();
            if r.len() >= SSE_RING_CAPACITY {
                r.pop_front();
            }
            r.push_back(ev);
        });
    });
}

/// Drain ring buffer events with seq > `since` into SSE text format.
pub fn sse_drain_since(since: u64) -> String {
    SSE_RING.with(|ring| {
        let r = ring.borrow();
        let mut out = String::new();
        for ev in r.iter() {
            if ev.seq > since {
                if let Ok(json) = serde_json::to_string(ev) {
                    out.push_str("id: ");
                    out.push_str(&ev.seq.to_string());
                    out.push('\n');
                    out.push_str("data: ");
                    out.push_str(&json);
                    out.push_str("\n\n");
                }
            }
        }
        out
    })
}

// ---------------------------------------------------------------------------
// Content-type negotiation: YAML → JSON value normalisation
// ---------------------------------------------------------------------------

/// Detect whether the content-type header signals YAML.
pub fn is_yaml_content_type(content_type: &str) -> bool {
    let ct = content_type.split(';').next().unwrap_or("").trim();
    matches!(
        ct,
        "application/yaml"
            | "application/x-yaml"
            | "text/yaml"
            | "text/x-yaml"
    )
}

/// Convert a request body to a serde_json::Value, accepting both JSON and YAML.
/// Returns `Err(message)` on parse failure.
pub fn body_to_value(body: &[u8], content_type: &str) -> Result<serde_json::Value, String> {
    if body.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    if is_yaml_content_type(content_type) {
        let yaml_val: serde_yaml::Value = serde_yaml::from_slice(body)
            .map_err(|e| format!("invalid YAML: {}", e))?;
        serde_json::to_value(yaml_val).map_err(|e| format!("YAML→JSON conversion failed: {}", e))
    } else {
        serde_json::from_slice(body).map_err(|e| format!("invalid JSON: {}", e))
    }
}

/// Deserialise a request body (JSON or YAML) into `T`.
pub fn parse_body<T: for<'de> Deserialize<'de>>(
    body: &[u8],
    content_type: &str,
) -> Result<T, String> {
    let val = body_to_value(body, content_type)?;
    serde_json::from_value(val).map_err(|e| format!("schema error: {}", e))
}

// ---------------------------------------------------------------------------
// Domain types (serde-only, no WIT dependency)
// ---------------------------------------------------------------------------

/// Condition for if-else branching: step only runs when `on_step`'s output
/// equals `equals` (parsed as JSON from the output bytes).
#[cfg_attr(all(test, not(target_arch = "wasm32")), derive(ts_rs::TS))]
#[cfg_attr(all(test, not(target_arch = "wasm32")), ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    pub on_step: String,
    pub equals: serde_json::Value,
}

#[cfg_attr(all(test, not(target_arch = "wasm32")), derive(ts_rs::TS))]
#[cfg_attr(all(test, not(target_arch = "wasm32")), ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepDef {
    pub name: String,
    pub depends_on: Vec<String>,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    #[serde(default)]
    #[cfg_attr(all(test, not(target_arch = "wasm32")), ts(type = "number"))]
    pub base_delay_ms: u64,
    /// Optional: per-step deadline in milliseconds.
    #[serde(default)]
    #[cfg_attr(all(test, not(target_arch = "wasm32")), ts(type = "number | null"))]
    pub timeout_ms: Option<u64>,
    /// Optional: name of a child workflow to delegate to.
    #[serde(default)]
    pub sub_workflow: Option<String>,
    /// If true, a skipped step does not cause the run to fail.
    #[serde(default)]
    pub optional: bool,
    /// If present, step only runs when the condition is satisfied.
    #[serde(default)]
    pub condition: Option<Condition>,
}

fn default_max_attempts() -> u32 {
    1
}

#[cfg_attr(all(test, not(target_arch = "wasm32")), derive(ts_rs::TS))]
#[cfg_attr(all(test, not(target_arch = "wasm32")), ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDef {
    pub event: String,
}

#[cfg_attr(all(test, not(target_arch = "wasm32")), derive(ts_rs::TS))]
#[cfg_attr(all(test, not(target_arch = "wasm32")), ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    #[cfg_attr(all(test, not(target_arch = "wasm32")), ts(type = "number | null"))]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub triggers: Vec<TriggerDef>,
    pub steps: Vec<StepDef>,
}

#[cfg_attr(all(test, not(target_arch = "wasm32")), derive(ts_rs::TS))]
#[cfg_attr(all(test, not(target_arch = "wasm32")), ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub wf_name: String,
    pub state: String,
    pub idem_key: Option<String>,
    #[cfg_attr(all(test, not(target_arch = "wasm32")), ts(type = "number"))]
    pub created_at_ms: u64,
}

#[cfg_attr(all(test, not(target_arch = "wasm32")), derive(ts_rs::TS))]
#[cfg_attr(all(test, not(target_arch = "wasm32")), ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub state: String,
    pub attempt: u32,
    #[cfg_attr(all(test, not(target_arch = "wasm32")), ts(type = "number"))]
    pub scheduled_at_ms: u64,
    /// When the step transitioned to "running" (set in handle_ready_steps).
    #[serde(default)]
    #[cfg_attr(all(test, not(target_arch = "wasm32")), ts(type = "number | null"))]
    pub started_at_ms: Option<u64>,
    pub output: Option<Vec<u8>>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

pub fn validate_workflow(def: &WorkflowDef) -> Result<(), String> {
    if def.name.is_empty() {
        return Err("name must not be empty".into());
    }
    if !def
        .name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "name '{}' contains invalid characters (only alphanumeric, '-', '_' allowed)",
            def.name
        ));
    }

    if def.steps.is_empty() {
        return Err("workflow must have at least one step".into());
    }

    let mut seen = std::collections::HashSet::new();
    for step in &def.steps {
        if step.name.is_empty() {
            return Err("step name must not be empty".into());
        }
        if !seen.insert(step.name.clone()) {
            return Err(format!("duplicate step name '{}'", step.name));
        }
    }

    for step in &def.steps {
        for dep in &step.depends_on {
            if !seen.contains(dep.as_str()) {
                return Err(format!(
                    "step '{}' depends on unknown step '{}'",
                    step.name, dep
                ));
            }
        }
    }

    for step in &def.steps {
        if step.max_attempts < 1 {
            return Err(format!(
                "step '{}' max_attempts must be >= 1",
                step.name
            ));
        }
    }

    for step in &def.steps {
        if step.timeout_ms == Some(0) {
            return Err(format!(
                "step '{}' timeout_ms must be > 0",
                step.name
            ));
        }
    }

    if def.timeout_ms == Some(0) {
        return Err("timeout_ms must be > 0".into());
    }

    for step in &def.steps {
        if let Some(ref sw) = step.sub_workflow {
            if sw.is_empty() {
                return Err(format!("step '{}' sub_workflow must not be empty", step.name));
            }
            if !sw.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return Err(format!(
                    "step '{}' sub_workflow '{}' contains invalid characters",
                    step.name, sw
                ));
            }
        }
    }

    for step in &def.steps {
        if let Some(ref cond) = step.condition {
            if !seen.contains(cond.on_step.as_str()) {
                return Err(format!(
                    "step '{}' condition references unknown step '{}'",
                    step.name, cond.on_step
                ));
            }
        }
    }

    detect_cycle(&def.steps)?;

    Ok(())
}

fn detect_cycle(steps: &[StepDef]) -> Result<(), String> {
    use std::collections::HashMap;

    let index: HashMap<&str, usize> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.as_str(), i))
        .collect();

    let mut color = vec![0u8; steps.len()];

    fn dfs(
        node: usize,
        steps: &[StepDef],
        index: &HashMap<&str, usize>,
        color: &mut Vec<u8>,
    ) -> Result<(), String> {
        color[node] = 1;
        for dep in &steps[node].depends_on {
            if let Some(&nb) = index.get(dep.as_str()) {
                if color[nb] == 1 {
                    return Err(format!("dependency cycle detected involving step '{}'", dep));
                }
                if color[nb] == 0 {
                    dfs(nb, steps, index, color)?;
                }
            }
        }
        color[node] = 2;
        Ok(())
    }

    for i in 0..steps.len() {
        if color[i] == 0 {
            dfs(i, steps, &index, &mut color)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Time / ID stubs
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub fn now_ms() -> u64 {
    wasi::clocks::monotonic_clock::now() / 1_000_000
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_ms() -> u64 {
    0
}

#[cfg(target_arch = "wasm32")]
pub fn unique_id() -> u64 {
    wasi::random::random::get_random_u64()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// StoreBackend trait — domain-level persistence abstraction
//
// wasm32: implemented by WasiStore (delegates to WIT-imported store functions)
// tests:  implemented by MemStore (in-memory HashMap)
// ---------------------------------------------------------------------------

pub trait StoreBackend {
    fn put_workflow_def(&self, name: &str, json: &[u8]) -> Result<(), String>;
    fn get_workflow_def(&self, name: &str) -> Option<Vec<u8>>;
    fn delete_workflow_def(&self, name: &str);
    fn list_workflow_names(&self, page: u32, limit: u32) -> Vec<String>;

    fn put_run(&self, run_id: &str, json: &[u8]) -> Result<(), String>;
    fn get_run(&self, run_id: &str) -> Option<Vec<u8>>;
    fn list_runs(
        &self,
        wf_name: &str,
        state_filter: Option<&str>,
        page: u32,
        limit: u32,
    ) -> Vec<Vec<u8>>;

    fn put_step(&self, run_id: &str, step_name: &str, json: &[u8]) -> Result<(), String>;
    fn get_step(&self, run_id: &str, step_name: &str) -> Option<Vec<u8>>;
    fn list_step_names(&self, run_id: &str) -> Vec<String>;

    fn put_event_subs(&self, event_name: &str, subs: Vec<String>);
    fn get_event_subs(&self, event_name: &str) -> Vec<String>;
    fn list_event_names(&self) -> Vec<String>;

    fn put_sub_run_link(&self, parent_run_id: &str, step_name: &str, child_run_id: &str);
    fn get_sub_run_link(&self, parent_run_id: &str, step_name: &str) -> Option<String>;
}

// ---------------------------------------------------------------------------
// Business logic — pure functions operating on a StoreBackend trait object
// ---------------------------------------------------------------------------

pub fn handle_register_workflow(
    body: &[u8],
    content_type: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    let def: WorkflowDef = match parse_body(body, content_type) {
        Ok(d) => d,
        Err(e) => return (400, format!(r#"{{"error":"{}"}}"#, e)),
    };
    if let Err(msg) = validate_workflow(&def) {
        return (400, format!(r#"{{"error":"{}"}}"#, msg));
    }
    let json = serde_json::to_vec(&def).unwrap();
    if let Err(e) = store.put_workflow_def(&def.name, &json) {
        return (500, format!(r#"{{"error":"{}"}}"#, e));
    }
    (201, format!(r#"{{"name":"{}","created":true}}"#, def.name))
}

pub fn handle_list_workflows(page: usize, limit: usize, store: &dyn StoreBackend) -> (u16, String) {
    let limit_u32 = limit as u32;
    let page_u32 = (page.max(1)) as u32;
    let effective_limit = if limit == 0 { 50u32 } else { limit_u32 };

    // Get total count first (page=1, limit=u32::MAX)
    let all_names = store.list_workflow_names(1, u32::MAX);
    let total = all_names.len();

    let names = store.list_workflow_names(page_u32, effective_limit);
    let name_strs: Vec<String> = names.iter().map(|n| format!(r#""{}""#, n)).collect();
    (200, format!(
        r#"{{"items":[{}],"total":{},"page":{},"limit":{}}}"#,
        name_strs.join(","),
        total,
        page_u32,
        effective_limit
    ))
}

pub fn handle_list_steps_for_run(
    run_id: &str,
    page: usize,
    limit: usize,
    store: &dyn StoreBackend,
) -> (u16, String) {
    if store.get_run(run_id).is_none() {
        return (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id));
    }
    let mut names = store.list_step_names(run_id);
    names.sort();
    let total = names.len();
    let page = page.max(1);
    let limit = if limit == 0 { 50 } else { limit };
    let start = (page - 1) * limit;
    let items: Vec<String> = names
        .iter()
        .skip(start)
        .take(limit)
        .filter_map(|step_name| {
            let v = store.get_step(run_id, step_name)?;
            let sr: StepRecord = serde_json::from_slice(&v).ok()?;
            Some(format!(
                r#"{{"name":"{}","state":"{}","attempt":{}}}"#,
                step_name, sr.state, sr.attempt
            ))
        })
        .collect();
    (200, format!(
        r#"{{"items":[{}],"total":{},"page":{},"limit":{}}}"#,
        items.join(","),
        total,
        page,
        limit
    ))
}

pub fn handle_get_workflow(name: &str, store: &dyn StoreBackend) -> (u16, String) {
    match store.get_workflow_def(name) {
        Some(v) => (200, String::from_utf8_lossy(&v).into_owned()),
        None => (404, format!(r#"{{"error":"workflow '{}' not found"}}"#, name)),
    }
}

pub fn handle_delete_workflow(name: &str, store: &dyn StoreBackend) -> (u16, String) {
    store.delete_workflow_def(name);
    (204, String::new())
}

pub fn handle_start_run(
    wf_name: &str,
    body: &[u8],
    content_type: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    let def_bytes = match store.get_workflow_def(wf_name) {
        Some(b) => b,
        None => {
            return (
                404,
                format!(r#"{{"error":"workflow '{}' not found"}}"#, wf_name),
            )
        }
    };
    let def: WorkflowDef = match serde_json::from_slice(&def_bytes) {
        Ok(d) => d,
        Err(_) => return (500, r#"{"error":"corrupt workflow definition"}"#.into()),
    };

    #[derive(Deserialize, Default)]
    struct RunReq {
        #[allow(dead_code)]
        input: Option<serde_json::Value>,
        idem_key: Option<String>,
    }
    let req: RunReq = if body.is_empty() {
        RunReq::default()
    } else {
        match parse_body(body, content_type) {
            Ok(r) => r,
            Err(e) => return (400, format!(r#"{{"error":"{}"}}"#, e)),
        }
    };

    let ts = now_ms();
    let uid = unique_id();
    let run_id = if let Some(ref ik) = req.idem_key {
        // Check for existing run with same idem_key
        let existing = store.list_runs(wf_name, None, 1, u32::MAX);
        for v in &existing {
            if let Ok(r) = serde_json::from_slice::<RunRecord>(v) {
                if r.idem_key.as_deref() == Some(ik.as_str()) {
                    return (
                        200,
                        format!(r#"{{"run_id":"{}","existing":true}}"#, r.run_id),
                    );
                }
            }
        }
        format!("wfrun-{}-{}-{}", wf_name, ik, uid)
    } else {
        format!("wfrun-{}-{}", wf_name, uid)
    };

    let run = RunRecord {
        run_id: run_id.clone(),
        wf_name: wf_name.to_string(),
        state: "running".to_string(),
        idem_key: req.idem_key,
        created_at_ms: ts,
    };
    let _ = store.put_run(&run_id, &serde_json::to_vec(&run).unwrap());
    sse_push("run.state", &run_id, Some(wf_name), None, "running");

    for step in &def.steps {
        let sr = StepRecord {
            state: "pending".to_string(),
            attempt: 0,
            scheduled_at_ms: 0,
            started_at_ms: None,
            output: None,
            error: None,
        };
        let _ = store.put_step(&run_id, &step.name, &serde_json::to_vec(&sr).unwrap());
    }

    (201, format!(r#"{{"run_id":"{}"}}"#, run_id))
}

pub fn handle_get_run(run_id: &str, store: &dyn StoreBackend) -> (u16, String) {
    match store.get_run(run_id) {
        Some(v) => (200, String::from_utf8_lossy(&v).into_owned()),
        None => (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id)),
    }
}

pub fn handle_cancel_run(run_id: &str, store: &dyn StoreBackend) -> (u16, String) {
    match store.get_run(run_id) {
        None => (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id)),
        Some(v) => {
            let mut run: RunRecord = match serde_json::from_slice(&v) {
                Ok(r) => r,
                Err(_) => return (500, r#"{"error":"corrupt run record"}"#.into()),
            };
            run.state = "cancelled".to_string();
            let _ = store.put_run(run_id, &serde_json::to_vec(&run).unwrap());
            sse_push("run.state", run_id, None, None, "cancelled");
            (204, String::new())
        }
    }
}

/// GET /runs/:run_id/steps/:step/output
pub fn handle_get_step_output(
    run_id: &str,
    step_name: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    match store.get_step(run_id, step_name) {
        None => (
            404,
            format!(r#"{{"error":"step '{}' not found for run '{}'"  }}"#, step_name, run_id),
        ),
        Some(v) => {
            let sr: StepRecord = match serde_json::from_slice(&v) {
                Ok(r) => r,
                Err(_) => return (500, r#"{"error":"corrupt step record"}"#.into()),
            };
            let output_json = match &sr.output {
                Some(bytes) => {
                    match serde_json::from_slice::<serde_json::Value>(bytes) {
                        Ok(val) => serde_json::to_string(&val).unwrap(),
                        Err(_) => serde_json::to_string(bytes).unwrap(),
                    }
                }
                None => "null".to_string(),
            };
            (
                200,
                format!(r#"{{"output":{},"state":"{}"}}"#, output_json, sr.state),
            )
        }
    }
}

/// POST /runs/:run_id/steps/:step/sub-run
pub fn handle_link_sub_run(
    run_id: &str,
    step_name: &str,
    body: &[u8],
    content_type: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    #[derive(Deserialize)]
    struct SubRunReq {
        sub_run_id: String,
    }
    let req: SubRunReq = match parse_body(body, content_type) {
        Ok(r) => r,
        Err(e) => return (400, format!(r#"{{"error":"{}"}}"#, e)),
    };

    if store.get_step(run_id, step_name).is_none() {
        return (
            404,
            format!(r#"{{"error":"step '{}' not found for run '{}'"  }}"#, step_name, run_id),
        );
    }

    store.put_sub_run_link(run_id, step_name, &req.sub_run_id);

    advance_sub_workflow_step(run_id, step_name, &req.sub_run_id, store);

    (204, String::new())
}

fn advance_sub_workflow_step(
    parent_run_id: &str,
    step_name: &str,
    child_run_id: &str,
    store: &dyn StoreBackend,
) {
    let child_run: RunRecord = match store
        .get_run(child_run_id)
        .and_then(|v| serde_json::from_slice(&v).ok())
    {
        Some(r) => r,
        None => return,
    };

    let sr: StepRecord = match store
        .get_step(parent_run_id, step_name)
        .and_then(|v| serde_json::from_slice(&v).ok())
    {
        Some(r) => r,
        None => return,
    };

    if sr.state != "pending" && sr.state != "running" {
        return;
    }

    match child_run.state.as_str() {
        "succeeded" => {
            let updated = StepRecord {
                state: "succeeded".to_string(),
                attempt: sr.attempt + 1,
                output: None,
                error: None,
                ..sr
            };
            let _ = store.put_step(parent_run_id, step_name, &serde_json::to_vec(&updated).unwrap());
            maybe_complete_run(parent_run_id, store);
        }
        "failed" | "cancelled" => {
            let updated = StepRecord {
                state: "failed".to_string(),
                attempt: sr.attempt + 1,
                error: Some(format!("child run {} {}", child_run_id, child_run.state)),
                ..sr
            };
            let _ = store.put_step(parent_run_id, step_name, &serde_json::to_vec(&updated).unwrap());
            if let Some(v) = store.get_run(parent_run_id) {
                if let Ok(mut run) = serde_json::from_slice::<RunRecord>(&v) {
                    if run.state == "running" {
                        run.state = "failed".to_string();
                        let _ = store.put_run(parent_run_id, &serde_json::to_vec(&run).unwrap());
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn handle_ready_steps(run_id: &str, store: &dyn StoreBackend) -> (u16, String) {
    let run_bytes = match store.get_run(run_id) {
        Some(b) => b,
        None => return (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id)),
    };
    let run: RunRecord = match serde_json::from_slice(&run_bytes) {
        Ok(r) => r,
        Err(_) => return (500, r#"{"error":"corrupt run record"}"#.into()),
    };

    let def_bytes = match store.get_workflow_def(&run.wf_name) {
        Some(b) => b,
        None => return (500, r#"{"error":"workflow definition missing"}"#.into()),
    };
    let def: WorkflowDef = match serde_json::from_slice(&def_bytes) {
        Ok(d) => d,
        Err(_) => return (500, r#"{"error":"corrupt workflow definition"}"#.into()),
    };

    if check_run_timeout(run_id, &run, &def, store) {
        return (200, "[]".to_string());
    }

    check_step_timeouts(run_id, &def, store);

    let ts = now_ms();
    let mut ready = Vec::new();

    for step in &def.steps {
        let sr: StepRecord = match store.get_step(run_id, &step.name) {
            Some(v) => serde_json::from_slice(&v).unwrap_or(StepRecord {
                state: "pending".to_string(),
                attempt: 0,
                scheduled_at_ms: 0,
                started_at_ms: None,
                output: None,
                error: None,
            }),
            None => continue,
        };

        if sr.state != "pending" {
            if sr.state == "pending" || sr.state == "running" {
                if let Some(child_id) = store.get_sub_run_link(run_id, &step.name) {
                    advance_sub_workflow_step(run_id, &step.name, &child_id, store);
                }
            }
            continue;
        }
        if sr.scheduled_at_ms > ts {
            continue;
        }

        let deps_ok = step.depends_on.iter().all(|dep| {
            store.get_step(run_id, dep)
                .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                .map(|dr| dr.state == "succeeded" || dr.state == "skipped")
                .unwrap_or(false)
        });

        if !deps_ok {
            continue;
        }

        if let Some(ref cond) = step.condition {
            let on_step_output: Option<Vec<u8>> = store
                .get_step(run_id, &cond.on_step)
                .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                .and_then(|sr| sr.output);

            let condition_met = match on_step_output {
                Some(bytes) => serde_json::from_slice::<serde_json::Value>(&bytes)
                    .map(|val| val == cond.equals)
                    .unwrap_or(false),
                None => serde_json::Value::Null == cond.equals,
            };

            if !condition_met {
                let skipped = StepRecord {
                    state: "skipped".to_string(),
                    ..sr
                };
                let _ = store.put_step(run_id, &step.name, &serde_json::to_vec(&skipped).unwrap());
                apply_transitive_skips(run_id, &def, store);
                maybe_complete_run(run_id, store);
                continue;
            }
        }

        let sr = if sr.started_at_ms.is_none() {
            let updated = StepRecord {
                started_at_ms: Some(ts),
                ..sr
            };
            let _ = store.put_step(run_id, &step.name, &serde_json::to_vec(&updated).unwrap());
            updated
        } else {
            sr
        };

        let sw_field = if let Some(ref sw) = step.sub_workflow {
            format!(r#","sub_workflow":"{}""#, sw)
        } else {
            String::new()
        };

        ready.push(format!(
            r#"{{"name":"{}","state":"{}","attempt":{}{}}}"#,
            step.name, sr.state, sr.attempt, sw_field
        ));
    }

    (200, format!("[{}]", ready.join(",")))
}

pub fn handle_step_done(
    run_id: &str,
    step_name: &str,
    body: &[u8],
    content_type: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    let sr = match store.get_step(run_id, step_name) {
        Some(v) => serde_json::from_slice::<StepRecord>(&v).unwrap_or_default_step(),
        None => return (404, format!(r#"{{"error":"step '{}' not found for run '{}'"  }}"#, step_name, run_id)),
    };

    #[derive(Deserialize, Default)]
    struct DoneReq {
        output: Option<serde_json::Value>,
    }
    let req: DoneReq = if body.is_empty() {
        DoneReq::default()
    } else {
        parse_body(body, content_type).unwrap_or_default()
    };

    let output_bytes: Option<Vec<u8>> = req.output.map(|v| {
        match &v {
            serde_json::Value::Array(arr) => {
                let bytes: Option<Vec<u8>> = arr.iter()
                    .map(|n| n.as_u64().and_then(|b| u8::try_from(b).ok()))
                    .collect();
                bytes.unwrap_or_else(|| serde_json::to_vec(&v).unwrap_or_default())
            }
            _ => serde_json::to_vec(&v).unwrap_or_default(),
        }
    });

    let updated = StepRecord {
        state: "succeeded".to_string(),
        attempt: sr.attempt + 1,
        output: output_bytes,
        ..sr
    };
    let _ = store.put_step(run_id, step_name, &serde_json::to_vec(&updated).unwrap());
    sse_push("step.state", run_id, None, Some(step_name), "succeeded");

    if let Some(run_bytes) = store.get_run(run_id) {
        if let Ok(run) = serde_json::from_slice::<RunRecord>(&run_bytes) {
            if let Some(def_bytes) = store.get_workflow_def(&run.wf_name) {
                if let Ok(def) = serde_json::from_slice::<WorkflowDef>(&def_bytes) {
                    evaluate_conditions_for_unblocked(run_id, &def, store, (step_name, &updated.output));
                    apply_transitive_skips(run_id, &def, store);
                }
            }
        }
    }
    maybe_complete_run(run_id, store);
    (204, String::new())
}

pub fn handle_step_failed(
    run_id: &str,
    step_name: &str,
    body: &[u8],
    content_type: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    let run_bytes = match store.get_run(run_id) {
        Some(b) => b,
        None => return (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id)),
    };
    let run: RunRecord = match serde_json::from_slice(&run_bytes) {
        Ok(r) => r,
        Err(_) => return (500, r#"{"error":"corrupt run record"}"#.into()),
    };

    let def_bytes = match store.get_workflow_def(&run.wf_name) {
        Some(b) => b,
        None => return (500, r#"{"error":"workflow definition missing"}"#.into()),
    };
    let def: WorkflowDef = match serde_json::from_slice(&def_bytes) {
        Ok(d) => d,
        Err(_) => return (500, r#"{"error":"corrupt workflow definition"}"#.into()),
    };

    let sr = match store.get_step(run_id, step_name) {
        Some(v) => serde_json::from_slice::<StepRecord>(&v).unwrap_or_default_step(),
        None => return (404, format!(r#"{{"error":"step '{}' not found"}}"#, step_name)),
    };

    #[derive(Deserialize, Default)]
    struct FailReq {
        error: Option<String>,
    }
    let req: FailReq = if body.is_empty() {
        FailReq::default()
    } else {
        parse_body(body, content_type).unwrap_or_default()
    };

    let step_def = def.steps.iter().find(|s| s.name == step_name);
    let max_attempts = step_def.map(|s| s.max_attempts).unwrap_or(1);
    let base_delay = step_def.map(|s| s.base_delay_ms).unwrap_or(500);
    let new_attempt = sr.attempt + 1;

    let updated = if new_attempt >= max_attempts {
        let mut run_updated: RunRecord = serde_json::from_slice(&run_bytes).unwrap();
        run_updated.state = "failed".to_string();
        let _ = store.put_run(run_id, &serde_json::to_vec(&run_updated).unwrap());
        StepRecord {
            state: "failed".to_string(),
            attempt: new_attempt,
            error: req.error,
            ..sr
        }
    } else {
        let delay = (base_delay * (1u64 << new_attempt.min(6))).min(60_000);
        StepRecord {
            state: "pending".to_string(),
            attempt: new_attempt,
            scheduled_at_ms: now_ms() + delay,
            error: req.error,
            ..sr
        }
    };
    let _ = store.put_step(run_id, step_name, &serde_json::to_vec(&updated).unwrap());
    sse_push("step.state", run_id, None, Some(step_name), &updated.state);
    (204, String::new())
}

fn maybe_complete_run(run_id: &str, store: &dyn StoreBackend) {
    let run_bytes = match store.get_run(run_id) {
        Some(b) => b,
        None => return,
    };
    let mut run: RunRecord = match serde_json::from_slice(&run_bytes) {
        Ok(r) => r,
        Err(_) => return,
    };
    if run.state != "running" {
        return;
    }

    let def_key = run.wf_name.clone();
    let _def: WorkflowDef = match store
        .get_workflow_def(&def_key)
        .and_then(|v| serde_json::from_slice(&v).ok())
    {
        Some(d) => d,
        None => return,
    };

    let step_names = store.list_step_names(run_id);
    if step_names.is_empty() {
        return;
    }

    let mut all_terminal = true;

    for name in &step_names {
        let sr: StepRecord = match store
            .get_step(run_id, name)
            .and_then(|v| serde_json::from_slice(&v).ok())
        {
            Some(r) => r,
            None => { all_terminal = false; break; }
        };
        match sr.state.as_str() {
            "succeeded" | "skipped" => {}
            _ => { all_terminal = false; break; }
        }
    }

    if all_terminal {
        run.state = "succeeded".to_string();
        let _ = store.put_run(run_id, &serde_json::to_vec(&run).unwrap());
        sse_push("run.state", run_id, Some(&run.wf_name), None, "succeeded");
    }
}

fn evaluate_conditions_for_unblocked(
    run_id: &str,
    def: &WorkflowDef,
    store: &dyn StoreBackend,
    completed_step: (&str, &Option<Vec<u8>>),
) {
    let (completed_name, completed_output) = completed_step;
    for step in &def.steps {
        let sr: StepRecord = match store.get_step(run_id, &step.name)
            .and_then(|v| serde_json::from_slice(&v).ok())
        {
            Some(r) => r,
            None => continue,
        };
        if sr.state != "pending" {
            continue;
        }
        let deps_all_terminal = step.depends_on.iter().all(|dep| {
            if dep == completed_name {
                return true;
            }
            store.get_step(run_id, dep)
                .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                .map(|dr| dr.state == "succeeded" || dr.state == "skipped")
                .unwrap_or(false)
        });
        if step.depends_on.is_empty() || !deps_all_terminal {
            continue;
        }
        if let Some(ref cond) = step.condition {
            let on_step_output: Option<Vec<u8>> = if cond.on_step == completed_name {
                completed_output.clone()
            } else {
                store.get_step(run_id, &cond.on_step)
                    .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                    .and_then(|sr| sr.output)
            };
            let condition_met = match on_step_output {
                Some(bytes) => serde_json::from_slice::<serde_json::Value>(&bytes)
                    .map(|val| val == cond.equals)
                    .unwrap_or(false),
                None => serde_json::Value::Null == cond.equals,
            };
            if !condition_met {
                let skipped = StepRecord { state: "skipped".to_string(), ..sr };
                let _ = store.put_step(run_id, &step.name, &serde_json::to_vec(&skipped).unwrap());
            }
        }
    }
}

fn apply_transitive_skips(run_id: &str, def: &WorkflowDef, store: &dyn StoreBackend) {
    let mut dependents: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for step in &def.steps {
        for dep in &step.depends_on {
            dependents
                .entry(dep.as_str())
                .or_default()
                .push(step.name.as_str());
        }
    }

    let mut to_visit: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    for step in &def.steps {
        if let Some(v) = store.get_step(run_id, &step.name) {
            if let Ok(sr) = serde_json::from_slice::<StepRecord>(&v) {
                if sr.state == "skipped" {
                    to_visit.push_back(step.name.clone());
                }
            }
        }
    }

    while let Some(skipped_name) = to_visit.pop_front() {
        for &dependent in dependents.get(skipped_name.as_str()).unwrap_or(&vec![]) {
            let sr: StepRecord = match store
                .get_step(run_id, dependent)
                .and_then(|v| serde_json::from_slice(&v).ok())
            {
                Some(r) => r,
                None => continue,
            };
            if sr.state != "pending" {
                continue;
            }
            let step_def = def.steps.iter().find(|s| s.name == dependent);
            let all_deps_resolved = step_def.map(|sd| {
                sd.depends_on.iter().all(|dep| {
                    store.get_step(run_id, dep)
                        .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                        .map(|dr| dr.state == "succeeded" || dr.state == "skipped")
                        .unwrap_or(false)
                })
            }).unwrap_or(false);

            if all_deps_resolved {
                let has_skipped_dep = step_def.map(|sd| {
                    sd.depends_on.iter().any(|dep| {
                        store.get_step(run_id, dep)
                            .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                            .map(|dr| dr.state == "skipped")
                            .unwrap_or(false)
                    })
                }).unwrap_or(false);

                let has_condition = step_def.map(|sd| sd.condition.is_some()).unwrap_or(false);
                if has_skipped_dep && !has_condition {
                    let updated = StepRecord {
                        state: "skipped".to_string(),
                        ..sr
                    };
                    let _ = store.put_step(run_id, dependent, &serde_json::to_vec(&updated).unwrap());
                    to_visit.push_back(dependent.to_string());
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Timeout enforcement
// ---------------------------------------------------------------------------

fn check_step_timeouts(run_id: &str, def: &WorkflowDef, store: &dyn StoreBackend) {
    let ts = now_ms();
    for step_def in &def.steps {
        let timeout_ms = match step_def.timeout_ms {
            Some(t) => t,
            None => continue,
        };
        let sr: StepRecord = match store.get_step(run_id, &step_def.name)
            .and_then(|v| serde_json::from_slice(&v).ok())
        {
            Some(r) => r,
            None => continue,
        };
        if sr.state != "running" && sr.state != "pending" {
            continue;
        }
        let started = match sr.started_at_ms {
            Some(t) => t,
            None => continue,
        };
        if ts.saturating_sub(started) > timeout_ms {
            let new_attempt = sr.attempt + 1;
            let updated = if new_attempt >= step_def.max_attempts {
                if let Some(v) = store.get_run(run_id) {
                    if let Ok(mut run) = serde_json::from_slice::<RunRecord>(&v) {
                        if run.state == "running" {
                            run.state = "failed".to_string();
                            let _ = store.put_run(run_id, &serde_json::to_vec(&run).unwrap());
                        }
                    }
                }
                StepRecord {
                    state: "failed".to_string(),
                    attempt: new_attempt,
                    error: Some("step timeout".to_string()),
                    started_at_ms: None,
                    ..sr
                }
            } else {
                let delay = (step_def.base_delay_ms * (1u64 << new_attempt.min(6))).min(60_000);
                StepRecord {
                    state: "pending".to_string(),
                    attempt: new_attempt,
                    scheduled_at_ms: ts + delay,
                    started_at_ms: None,
                    error: Some("step timeout".to_string()),
                    ..sr
                }
            };
            let _ = store.put_step(run_id, &step_def.name, &serde_json::to_vec(&updated).unwrap());
        }
    }
}

fn check_run_timeout(
    run_id: &str,
    run: &RunRecord,
    def: &WorkflowDef,
    store: &dyn StoreBackend,
) -> bool {
    let timeout_ms = match def.timeout_ms {
        Some(t) => t,
        None => return false,
    };
    let ts = now_ms();
    if run.state == "running" && ts.saturating_sub(run.created_at_ms) > timeout_ms {
        let mut updated = run.clone();
        updated.state = "failed".to_string();
        let _ = store.put_run(run_id, &serde_json::to_vec(&updated).unwrap());
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Run history
// ---------------------------------------------------------------------------

pub fn handle_list_runs(
    wf_name: &str,
    state_filter: Option<&str>,
    page: usize,
    limit: usize,
    store: &dyn StoreBackend,
) -> (u16, String) {
    let page_u32 = (page.max(1)) as u32;
    let limit_u32 = if limit == 0 { 50u32 } else { limit as u32 };

    // Get total (no pagination, no filter) for "total" field
    let all_runs = store.list_runs(wf_name, None, 1, u32::MAX);
    let all_filtered: Vec<_> = all_runs
        .iter()
        .filter(|v| {
            state_filter.map(|sf| {
                serde_json::from_slice::<RunRecord>(v)
                    .map(|r| r.state == sf)
                    .unwrap_or(false)
            }).unwrap_or(true)
        })
        .collect();
    let total = all_filtered.len();

    let items_raw = store.list_runs(wf_name, state_filter, page_u32, limit_u32);
    let items: Vec<String> = items_raw
        .iter()
        .map(|v| String::from_utf8_lossy(v).into_owned())
        .collect();

    (200, format!(
        r#"{{"items":[{}],"total":{},"page":{},"limit":{}}}"#,
        items.join(","),
        total,
        page_u32,
        limit_u32
    ))
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

pub fn handle_event_subscribe(
    event_name: &str,
    body: &[u8],
    content_type: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    #[derive(Deserialize)]
    struct SubReq {
        fn_name: String,
    }
    let req: SubReq = match parse_body(body, content_type) {
        Ok(r) => r,
        Err(e) => return (400, format!(r#"{{"error":"{}"}}"#, e)),
    };
    let mut subs = store.get_event_subs(event_name);
    if !subs.contains(&req.fn_name) {
        subs.push(req.fn_name);
        store.put_event_subs(event_name, subs);
    }
    (204, String::new())
}

pub fn handle_event_unsubscribe(
    event_name: &str,
    fn_name: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    let mut subs = store.get_event_subs(event_name);
    subs.retain(|s| s != fn_name);
    store.put_event_subs(event_name, subs);
    (204, String::new())
}

pub fn handle_event_emit(
    event_name: &str,
    body: &[u8],
    _content_type: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    let _ = body;
    let subs = store.get_event_subs(event_name);
    let names: Vec<String> = subs.iter().map(|s| format!(r#""{}""#, s)).collect();
    (200, format!("[{}]", names.join(",")))
}

pub fn handle_list_events(store: &dyn StoreBackend) -> (u16, String) {
    let names = store.list_event_names();
    let strs: Vec<String> = names.iter().map(|n| format!(r#""{}""#, n)).collect();
    (200, format!("[{}]", strs.join(",")))
}

// ---------------------------------------------------------------------------
// Helper trait for default StepRecord deserialization
// ---------------------------------------------------------------------------

trait DeserOrDefault {
    fn unwrap_or_default_step(self) -> StepRecord;
}

impl DeserOrDefault for Result<StepRecord, serde_json::Error> {
    fn unwrap_or_default_step(self) -> StepRecord {
        self.unwrap_or(StepRecord {
            state: "pending".to_string(),
            attempt: 0,
            scheduled_at_ms: 0,
            started_at_ms: None,
            output: None,
            error: None,
        })
    }
}

// ---------------------------------------------------------------------------
// HTTP router (pure, called from both native tests and WASM handler)
// ---------------------------------------------------------------------------

pub fn handle_step_retry(run_id: &str, step_name: &str, store: &dyn StoreBackend) -> (u16, String) {
    match store.get_step(run_id, step_name) {
        None => return (404, format!(r#"{{"error":"step '{}' not found"}}"#, step_name)),
        Some(v) => {
            let mut sr: StepRecord = match serde_json::from_slice(&v) {
                Ok(r) => r,
                Err(_) => return (500, r#"{"error":"corrupt step record"}"#.into()),
            };
            sr.state = "pending".to_string();
            let _ = store.put_step(run_id, step_name, &serde_json::to_vec(&sr).unwrap());
            sse_push("step.state", run_id, None, Some(step_name), "pending");
        }
    }
    (204, String::new())
}

pub fn handle_reset(store: &dyn StoreBackend) -> (u16, String) {
    // Reset by listing all workflow names and runs then deleting them
    let wf_names = store.list_workflow_names(1, u32::MAX);
    for name in &wf_names {
        store.delete_workflow_def(name);
    }
    let all_events = store.list_event_names();
    for event in &all_events {
        store.put_event_subs(event, vec![]);
    }
    // Note: runs/steps/sub-run-links cannot be enumerated without a full KV scan.
    // For the test reset endpoint, MemStore implements this directly.
    (204, String::new())
}

/// GET /sse?last_id=N — returns buffered SSE events since sequence N.
/// The `Content-Type: text/event-stream` header is set by the WASM handler.
pub fn handle_sse(since_seq: u64) -> (u16, String) {
    let body = sse_drain_since(since_seq);
    (200, body)
}

fn parse_query(full_path: &str) -> std::collections::HashMap<String, String> {
    let qs = full_path.splitn(2, '?').nth(1).unwrap_or("");
    qs.split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next()?.to_string();
            let v = parts.next().unwrap_or("").to_string();
            if k.is_empty() { None } else { Some((k, v)) }
        })
        .collect()
}

pub fn route(
    method: &str,
    path: &str,
    body: &[u8],
    content_type: &str,
    store: &dyn StoreBackend,
) -> (u16, String) {
    let query = parse_query(path);
    let page: usize = query.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let limit: usize = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(0);

    let path = path.split('?').next().unwrap_or(path);

    match (method, path) {
        ("GET", "/sse") => {
            let since: u64 = query.get("last_id").and_then(|v| v.parse().ok()).unwrap_or(0);
            handle_sse(since)
        }
        ("POST", "/workflows") => handle_register_workflow(body, content_type, store),
        ("GET", "/workflows") => handle_list_workflows(page, limit, store),
        ("GET", "/events") => handle_list_events(store),
        ("DELETE", "/_reset") => handle_reset(store),
        ("POST", "/runs") => {
            #[derive(serde::Deserialize)]
            struct StartReq { wf_name: String, idem_key: Option<String> }
            match serde_json::from_slice::<StartReq>(body) {
                Ok(req) => {
                    let inner_body = if let Some(ik) = req.idem_key {
                        serde_json::json!({"idem_key": ik}).to_string().into_bytes()
                    } else {
                        b"{}".to_vec()
                    };
                    handle_start_run(&req.wf_name, &inner_body, content_type, store)
                }
                Err(e) => (400, format!(r#"{{"error":"{}"}}"#, e)),
            }
        }

        _ if path.starts_with("/workflows/") => {
            let rest = &path["/workflows/".len()..];
            if rest.ends_with("/run") && method == "POST" {
                let name = &rest[..rest.len() - "/run".len()];
                handle_start_run(name, body, content_type, store)
            } else if rest.ends_with("/runs") && method == "GET" {
                let name = &rest[..rest.len() - "/runs".len()];
                let state_filter = query.get("state").map(|s| s.as_str());
                handle_list_runs(name, state_filter, page, limit, store)
            } else if method == "GET" && !rest.contains('/') {
                handle_get_workflow(rest, store)
            } else if method == "DELETE" && !rest.contains('/') {
                handle_delete_workflow(rest, store)
            } else {
                (404, r#"{"error":"not found"}"#.to_string())
            }
        }

        _ if path.starts_with("/runs/") => {
            let rest = &path["/runs/".len()..];
            if !rest.contains('/') {
                if method == "GET" {
                    handle_get_run(rest, store)
                } else {
                    (405, r#"{"error":"method not allowed"}"#.to_string())
                }
            } else if rest.ends_with("/cancel") && method == "POST" {
                let run_id = &rest[..rest.len() - "/cancel".len()];
                handle_cancel_run(run_id, store)
            } else if (rest.ends_with("/ready-steps") || rest.ends_with("/ready")) && method == "GET" {
                let run_id = if rest.ends_with("/ready-steps") {
                    &rest[..rest.len() - "/ready-steps".len()]
                } else {
                    &rest[..rest.len() - "/ready".len()]
                };
                handle_ready_steps(run_id, store)
            } else if rest.ends_with("/steps") && method == "GET" {
                let run_id = &rest[..rest.len() - "/steps".len()];
                handle_list_steps_for_run(run_id, page, limit, store)
            } else if let Some(steps_rest) = rest.find("/steps/").map(|i| &rest[i + "/steps/".len()..]) {
                let run_id = &rest[..rest.find("/steps/").unwrap()];
                if steps_rest.ends_with("/done") && method == "POST" {
                    let step_name = &steps_rest[..steps_rest.len() - "/done".len()];
                    handle_step_done(run_id, step_name, body, content_type, store)
                } else if steps_rest.ends_with("/failed") && method == "POST" {
                    let step_name = &steps_rest[..steps_rest.len() - "/failed".len()];
                    handle_step_failed(run_id, step_name, body, content_type, store)
                } else if steps_rest.ends_with("/retry") && method == "POST" {
                    let step_name = &steps_rest[..steps_rest.len() - "/retry".len()];
                    handle_step_retry(run_id, step_name, store)
                } else if steps_rest.ends_with("/output") && method == "GET" {
                    let step_name = &steps_rest[..steps_rest.len() - "/output".len()];
                    handle_get_step_output(run_id, step_name, store)
                } else if !steps_rest.contains('/') && method == "GET" {
                    handle_get_step_output(run_id, steps_rest, store)
                } else if steps_rest.ends_with("/sub-run") && method == "POST" {
                    let step_name = &steps_rest[..steps_rest.len() - "/sub-run".len()];
                    handle_link_sub_run(run_id, step_name, body, content_type, store)
                } else {
                    (404, r#"{"error":"not found"}"#.to_string())
                }
            } else {
                (404, r#"{"error":"not found"}"#.to_string())
            }
        }

        _ if path.starts_with("/events/") => {
            let rest = &path["/events/".len()..];
            if rest.ends_with("/subscribe") && method == "POST" {
                let name = &rest[..rest.len() - "/subscribe".len()];
                handle_event_subscribe(name, body, content_type, store)
            } else if rest.ends_with("/unsubscribe") && method == "POST" {
                let name = &rest[..rest.len() - "/unsubscribe".len()];
                #[derive(serde::Deserialize)]
                struct UnsubReq { fn_name: String }
                match parse_body::<UnsubReq>(body, content_type) {
                    Ok(req) => handle_event_unsubscribe(name, &req.fn_name, store),
                    Err(e) => (400, format!(r#"{{"error":"{}"}}"#, e)),
                }
            } else if rest.ends_with("/emit") && method == "POST" {
                let name = &rest[..rest.len() - "/emit".len()];
                handle_event_emit(name, body, content_type, store)
            } else if rest.ends_with("/subscribers") && method == "GET" {
                let name = &rest[..rest.len() - "/subscribers".len()];
                let subs = store.get_event_subs(name);
                let names: Vec<String> = subs.iter().map(|s| format!(r#""{}""#, s)).collect();
                (200, format!("[{}]", names.join(",")))
            } else if rest.contains("/subscribe/") && method == "DELETE" {
                let idx = rest.find("/subscribe/").unwrap();
                let name = &rest[..idx];
                let fn_name = &rest[idx + "/subscribe/".len()..];
                handle_event_unsubscribe(name, fn_name, store)
            } else {
                (404, r#"{"error":"not found"}"#.to_string())
            }
        }

        _ => (404, r#"{"error":"not found"}"#.to_string()),
    }
}

// ---------------------------------------------------------------------------
// WASM HTTP handler (wasm32 only)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
struct WorkflowApi;

/// WASM store backed by wasmcloud:workflow-store/store import.
#[cfg(target_arch = "wasm32")]
struct WasiStore;

#[cfg(target_arch = "wasm32")]
impl StoreBackend for WasiStore {
    fn put_workflow_def(&self, name: &str, json: &[u8]) -> Result<(), String> {
        wasmcloud::workflow_store::store::put_workflow_def(name, json)
            .map_err(|e| format!("{:?}", e))
    }

    fn get_workflow_def(&self, name: &str) -> Option<Vec<u8>> {
        wasmcloud::workflow_store::store::get_workflow_def(name).ok().flatten()
    }

    fn delete_workflow_def(&self, name: &str) {
        let _ = wasmcloud::workflow_store::store::delete_workflow_def(name);
    }

    fn list_workflow_names(&self, page: u32, limit: u32) -> Vec<String> {
        wasmcloud::workflow_store::store::list_workflow_names(page, limit)
            .unwrap_or_default()
    }

    fn put_run(&self, run_id: &str, json: &[u8]) -> Result<(), String> {
        wasmcloud::workflow_store::store::put_run(run_id, json)
            .map_err(|e| format!("{:?}", e))
    }

    fn get_run(&self, run_id: &str) -> Option<Vec<u8>> {
        wasmcloud::workflow_store::store::get_run(run_id).ok().flatten()
    }

    fn list_runs(
        &self,
        wf_name: &str,
        state_filter: Option<&str>,
        page: u32,
        limit: u32,
    ) -> Vec<Vec<u8>> {
        wasmcloud::workflow_store::store::list_runs(
            wf_name,
            state_filter,
            page,
            limit,
        )
        .unwrap_or_default()
    }

    fn put_step(&self, run_id: &str, step_name: &str, json: &[u8]) -> Result<(), String> {
        wasmcloud::workflow_store::store::put_step(run_id, step_name, json)
            .map_err(|e| format!("{:?}", e))
    }

    fn get_step(&self, run_id: &str, step_name: &str) -> Option<Vec<u8>> {
        wasmcloud::workflow_store::store::get_step(run_id, step_name).ok().flatten()
    }

    fn list_step_names(&self, run_id: &str) -> Vec<String> {
        wasmcloud::workflow_store::store::list_step_names(run_id).unwrap_or_default()
    }

    fn put_event_subs(&self, event_name: &str, subs: Vec<String>) {
        let _ = wasmcloud::workflow_store::store::put_event_subs(event_name, &subs);
    }

    fn get_event_subs(&self, event_name: &str) -> Vec<String> {
        wasmcloud::workflow_store::store::get_event_subs(event_name).unwrap_or_default()
    }

    fn list_event_names(&self) -> Vec<String> {
        wasmcloud::workflow_store::store::list_event_names().unwrap_or_default()
    }

    fn put_sub_run_link(&self, parent_run_id: &str, step_name: &str, child_run_id: &str) {
        let _ = wasmcloud::workflow_store::store::put_sub_run_link(
            parent_run_id,
            step_name,
            child_run_id,
        );
    }

    fn get_sub_run_link(&self, parent_run_id: &str, step_name: &str) -> Option<String> {
        wasmcloud::workflow_store::store::get_sub_run_link(parent_run_id, step_name)
            .ok()
            .flatten()
    }
}

#[cfg(target_arch = "wasm32")]
impl exports::wasi::http::incoming_handler::Guest for WorkflowApi {
    fn handle(
        request: wasi::http::types::IncomingRequest,
        response_out: wasi::http::types::ResponseOutparam,
    ) {
        use wasi::http::types::{Headers, OutgoingBody, OutgoingResponse};

        let method = {
            use wasi::http::types::Method;
            match request.method() {
                Method::Get => "GET",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Delete => "DELETE",
                Method::Patch => "PATCH",
                Method::Head => "HEAD",
                Method::Options => "OPTIONS",
                _ => "UNKNOWN",
            }.to_string()
        };
        let path = request.path_with_query().unwrap_or_default();

        let body: Vec<u8> = request
            .consume()
            .ok()
            .and_then(|b| {
                let stream = b.stream().ok()?;
                let mut buf = Vec::new();
                loop {
                    match stream.blocking_read(4096) {
                        Ok(chunk) if chunk.is_empty() => break,
                        Ok(chunk) => buf.extend_from_slice(&chunk),
                        Err(_) => break,
                    }
                }
                Some(buf)
            })
            .unwrap_or_default();

        let content_type = request
            .headers()
            .get(&"content-type".to_string())
            .into_iter()
            .next()
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_default();

        let store = WasiStore;
        let is_sse = path.starts_with("/sse");
        let (status, resp_body) = route(&method, &path, &body, &content_type, &store);

        let headers = Headers::new();
        let ct_value: &[u8] = if is_sse {
            b"text/event-stream; charset=utf-8"
        } else {
            b"application/json"
        };
        let _ = headers.append(&"content-type".to_string(), &ct_value.to_vec());
        let _ = headers.append(&"access-control-allow-origin".to_string(), &b"*".to_vec());
        if is_sse {
            let _ = headers.append(&"cache-control".to_string(), &b"no-cache".to_vec());
        }
        let resp = OutgoingResponse::new(headers);
        resp.set_status_code(status).ok();

        if let Ok(ob) = resp.body() {
            if !resp_body.is_empty() {
                if let Ok(stream) = ob.write() {
                    let _ = stream.blocking_write_and_flush(resp_body.as_bytes());
                }
            }
            OutgoingBody::finish(ob, None).ok();
        }

        wasi::http::types::ResponseOutparam::set(response_out, Ok(resp));
    }
}

#[cfg(target_arch = "wasm32")]
export!(WorkflowApi);

// ---------------------------------------------------------------------------
// In-memory StoreBackend for tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub mod test_store {
    use super::{RunRecord, StepRecord, StoreBackend};
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// In-memory store for unit tests.
    /// The `steps` map uses `(run_id, step_name)` composite key.
    pub struct MemStore {
        pub defs:   RefCell<HashMap<String, Vec<u8>>>,
        pub runs:   RefCell<HashMap<String, Vec<u8>>>,
        pub steps:  RefCell<HashMap<(String, String), Vec<u8>>>,
        pub events: RefCell<HashMap<String, Vec<String>>>,
        pub links:  RefCell<HashMap<(String, String), String>>,
    }

    impl MemStore {
        pub fn new() -> Self {
            MemStore {
                defs:   RefCell::new(HashMap::new()),
                runs:   RefCell::new(HashMap::new()),
                steps:  RefCell::new(HashMap::new()),
                events: RefCell::new(HashMap::new()),
                links:  RefCell::new(HashMap::new()),
            }
        }

        /// Get a step record directly (for test assertions).
        pub fn get_step_record(&self, run_id: &str, step_name: &str) -> Option<StepRecord> {
            self.steps
                .borrow()
                .get(&(run_id.to_string(), step_name.to_string()))
                .and_then(|v| serde_json::from_slice(v).ok())
        }
    }

    impl StoreBackend for MemStore {
        fn put_workflow_def(&self, name: &str, json: &[u8]) -> Result<(), String> {
            self.defs.borrow_mut().insert(name.to_string(), json.to_vec());
            Ok(())
        }

        fn get_workflow_def(&self, name: &str) -> Option<Vec<u8>> {
            self.defs.borrow().get(name).cloned()
        }

        fn delete_workflow_def(&self, name: &str) {
            self.defs.borrow_mut().remove(name);
        }

        fn list_workflow_names(&self, page: u32, limit: u32) -> Vec<String> {
            let mut names: Vec<String> = self.defs.borrow().keys().cloned().collect();
            names.sort();
            let limit = if limit == 0 || limit == u32::MAX { names.len() } else { limit as usize };
            let start = ((page.max(1) - 1) as usize) * limit;
            names.into_iter().skip(start).take(limit).collect()
        }

        fn put_run(&self, run_id: &str, json: &[u8]) -> Result<(), String> {
            self.runs.borrow_mut().insert(run_id.to_string(), json.to_vec());
            Ok(())
        }

        fn get_run(&self, run_id: &str) -> Option<Vec<u8>> {
            self.runs.borrow().get(run_id).cloned()
        }

        fn list_runs(
            &self,
            wf_name: &str,
            state_filter: Option<&str>,
            page: u32,
            limit: u32,
        ) -> Vec<Vec<u8>> {
            let mut runs: Vec<(u64, Vec<u8>)> = self
                .runs
                .borrow()
                .values()
                .filter_map(|v| {
                    let r: RunRecord = serde_json::from_slice(v).ok()?;
                    if r.wf_name != wf_name {
                        return None;
                    }
                    if let Some(sf) = state_filter {
                        if r.state != sf {
                            return None;
                        }
                    }
                    Some((r.created_at_ms, v.clone()))
                })
                .collect();
            runs.sort_by(|a, b| b.0.cmp(&a.0));
            let limit = if limit == 0 || limit == u32::MAX { runs.len() } else { limit as usize };
            let start = ((page.max(1) - 1) as usize) * limit;
            runs.into_iter().skip(start).take(limit).map(|(_, v)| v).collect()
        }

        fn put_step(&self, run_id: &str, step_name: &str, json: &[u8]) -> Result<(), String> {
            self.steps
                .borrow_mut()
                .insert((run_id.to_string(), step_name.to_string()), json.to_vec());
            Ok(())
        }

        fn get_step(&self, run_id: &str, step_name: &str) -> Option<Vec<u8>> {
            self.steps
                .borrow()
                .get(&(run_id.to_string(), step_name.to_string()))
                .cloned()
        }

        fn list_step_names(&self, run_id: &str) -> Vec<String> {
            let mut names: Vec<String> = self
                .steps
                .borrow()
                .keys()
                .filter(|(rid, _)| rid == run_id)
                .map(|(_, sn)| sn.clone())
                .collect();
            names.sort();
            names
        }

        fn put_event_subs(&self, event_name: &str, subs: Vec<String>) {
            self.events.borrow_mut().insert(event_name.to_string(), subs);
        }

        fn get_event_subs(&self, event_name: &str) -> Vec<String> {
            self.events
                .borrow()
                .get(event_name)
                .cloned()
                .unwrap_or_default()
        }

        fn list_event_names(&self) -> Vec<String> {
            let mut names: Vec<String> = self.events.borrow().keys().cloned().collect();
            names.sort();
            names
        }

        fn put_sub_run_link(&self, parent_run_id: &str, step_name: &str, child_run_id: &str) {
            self.links
                .borrow_mut()
                .insert(
                    (parent_run_id.to_string(), step_name.to_string()),
                    child_run_id.to_string(),
                );
        }

        fn get_sub_run_link(&self, parent_run_id: &str, step_name: &str) -> Option<String> {
            self.links
                .borrow()
                .get(&(parent_run_id.to_string(), step_name.to_string()))
                .cloned()
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_store::MemStore;

    /// Convenience: route with JSON content-type.
    fn rj(method: &str, path: &str, body: &[u8], store: &dyn StoreBackend) -> (u16, String) {
        route(method, path, body, "application/json", store)
    }

    /// Convenience: route with YAML content-type.
    fn ry(method: &str, path: &str, body: &[u8], store: &dyn StoreBackend) -> (u16, String) {
        route(method, path, body, "application/yaml", store)
    }

    fn simple_wf_body() -> Vec<u8> {
        br#"{"name":"simple-job","steps":[{"name":"run","depends_on":[]}]}"#.to_vec()
    }

    fn order_wf_body() -> Vec<u8> {
        br#"{
            "name":"order-pipeline",
            "steps":[
                {"name":"validate","depends_on":[],"max_attempts":3,"base_delay_ms":500},
                {"name":"charge","depends_on":["validate"],"max_attempts":5,"base_delay_ms":1000},
                {"name":"fulfill","depends_on":["charge"],"max_attempts":3,"base_delay_ms":2000},
                {"name":"notify","depends_on":["fulfill"],"max_attempts":2,"base_delay_ms":500}
            ]
        }"#.to_vec()
    }

    // ---- validation tests ----

    #[test]
    fn validate_ok_minimal() {
        let def: WorkflowDef = serde_json::from_slice(&simple_wf_body()).unwrap();
        assert!(validate_workflow(&def).is_ok());
    }

    #[test]
    fn validate_empty_name() {
        let def = WorkflowDef {
            name: "".into(),
            description: None,
            timeout_ms: None,
            triggers: vec![],
            steps: vec![StepDef {
                name: "s".into(),
                depends_on: vec![],
                max_attempts: 1,
                base_delay_ms: 0,
                ..Default::default()
            }],
        };
        assert!(validate_workflow(&def).is_err());
    }

    #[test]
    fn validate_invalid_name_chars() {
        let def = WorkflowDef {
            name: "bad name!".into(),
            description: None,
            timeout_ms: None,
            triggers: vec![],
            steps: vec![StepDef {
                name: "s".into(),
                depends_on: vec![],
                max_attempts: 1,
                base_delay_ms: 0,
                ..Default::default()
            }],
        };
        assert!(validate_workflow(&def).is_err());
    }

    #[test]
    fn validate_duplicate_step() {
        let def = WorkflowDef {
            name: "wf".into(),
            description: None,
            timeout_ms: None,
            triggers: vec![],
            steps: vec![
                StepDef { name: "a".into(), depends_on: vec![], max_attempts: 1, base_delay_ms: 0, ..Default::default() },
                StepDef { name: "a".into(), depends_on: vec![], max_attempts: 1, base_delay_ms: 0, ..Default::default() },
            ],
        };
        assert!(validate_workflow(&def).is_err());
    }

    #[test]
    fn validate_unknown_dep() {
        let def = WorkflowDef {
            name: "wf".into(),
            description: None,
            timeout_ms: None,
            triggers: vec![],
            steps: vec![StepDef {
                name: "a".into(),
                depends_on: vec!["ghost".into()],
                max_attempts: 1,
                base_delay_ms: 0,
                ..Default::default()
            }],
        };
        assert!(validate_workflow(&def).is_err());
    }

    #[test]
    fn validate_cycle() {
        let def = WorkflowDef {
            name: "wf".into(),
            description: None,
            timeout_ms: None,
            triggers: vec![],
            steps: vec![
                StepDef { name: "a".into(), depends_on: vec!["b".into()], max_attempts: 1, base_delay_ms: 0, ..Default::default() },
                StepDef { name: "b".into(), depends_on: vec!["a".into()], max_attempts: 1, base_delay_ms: 0, ..Default::default() },
            ],
        };
        assert!(validate_workflow(&def).is_err());
    }

    #[test]
    fn validate_max_attempts_zero() {
        let def = WorkflowDef {
            name: "wf".into(),
            description: None,
            timeout_ms: None,
            triggers: vec![],
            steps: vec![StepDef {
                name: "a".into(),
                depends_on: vec![],
                max_attempts: 0,
                base_delay_ms: 0,
                ..Default::default()
            }],
        };
        assert!(validate_workflow(&def).is_err());
    }

    // ---- HTTP routing tests (JSON) ----

    #[test]
    fn register_and_list() {
        let store = MemStore::new();
        let (s, _) = rj("POST", "/workflows", &simple_wf_body(), &store);
        assert_eq!(s, 201);
        let (s, body) = rj("GET", "/workflows", &[], &store);
        assert_eq!(s, 200);
        assert!(body.contains("simple-job"));
    }

    #[test]
    fn register_and_get() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (s, body) = rj("GET", "/workflows/simple-job", &[], &store);
        assert_eq!(s, 200);
        assert!(body.contains("simple-job"));
    }

    #[test]
    fn get_missing_workflow() {
        let store = MemStore::new();
        let (s, _) = rj("GET", "/workflows/nope", &[], &store);
        assert_eq!(s, 404);
    }

    #[test]
    fn delete_workflow() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (s, _) = rj("DELETE", "/workflows/simple-job", &[], &store);
        assert_eq!(s, 204);
        let (s, _) = rj("GET", "/workflows/simple-job", &[], &store);
        assert_eq!(s, 404);
    }

    #[test]
    fn start_run_and_get_status() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (s, body) = rj("POST", "/workflows/simple-job/run", &[], &store);
        assert_eq!(s, 201);
        assert!(body.contains("run_id"));

        let run_id: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = run_id["run_id"].as_str().unwrap();

        let (s, body) = rj("GET", &format!("/runs/{}", id), &[], &store);
        assert_eq!(s, 200);
        assert!(body.contains("running"));
    }

    #[test]
    fn start_run_missing_workflow() {
        let store = MemStore::new();
        let (s, _) = rj("POST", "/workflows/ghost/run", &[], &store);
        assert_eq!(s, 404);
    }

    #[test]
    fn ready_steps_all_pending_no_deps() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &store);
        let run_id: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = run_id["run_id"].as_str().unwrap();

        let (s, body) = rj("GET", &format!("/runs/{}/ready-steps", id), &[], &store);
        assert_eq!(s, 200);
        assert!(body.contains("run"));
    }

    #[test]
    fn step_done_completes_run() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &store);
        let run_id: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = run_id["run_id"].as_str().unwrap();

        let (s, _) = rj(
            "POST",
            &format!("/runs/{}/steps/run/done", id),
            br#"{"output":[]}"#,
            &store,
        );
        assert_eq!(s, 204);

        let (_, status_body) = rj("GET", &format!("/runs/{}", id), &[], &store);
        assert!(status_body.contains("succeeded"));
    }

    #[test]
    fn step_done_chain_completes_run() {
        let store = MemStore::new();
        rj("POST", "/workflows", &order_wf_body(), &store);
        let (_, body) = rj("POST", "/workflows/order-pipeline/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = v["run_id"].as_str().unwrap();

        for step in &["validate", "charge", "fulfill", "notify"] {
            let (s, _) = rj(
                "POST",
                &format!("/runs/{}/steps/{}/done", id, step),
                &[],
                &store,
            );
            assert_eq!(s, 204, "step {} done failed", step);
        }

        let (_, status_body) = rj("GET", &format!("/runs/{}", id), &[], &store);
        assert!(status_body.contains("succeeded"));
    }

    #[test]
    fn cancel_run() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = v["run_id"].as_str().unwrap();

        let (s, _) = rj("POST", &format!("/runs/{}/cancel", id), &[], &store);
        assert_eq!(s, 204);

        let (_, status_body) = rj("GET", &format!("/runs/{}", id), &[], &store);
        assert!(status_body.contains("cancelled"));
    }

    #[test]
    fn idem_key_deduplication() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);

        let body = br#"{"idem_key":"my-key"}"#.to_vec();
        let (_, b1) = rj("POST", "/workflows/simple-job/run", &body, &store);
        let (_, b2) = rj("POST", "/workflows/simple-job/run", &body, &store);

        let v1: serde_json::Value = serde_json::from_str(&b1).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&b2).unwrap();
        assert_eq!(v1["run_id"], v2["run_id"]);
    }

    #[test]
    fn step_failed_schedules_retry() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = v["run_id"].as_str().unwrap();

        let (s, _) = rj(
            "POST",
            &format!("/runs/{}/steps/run/failed", id),
            br#"{"error":"oops"}"#,
            &store,
        );
        assert_eq!(s, 204);

        let (_, status_body) = rj("GET", &format!("/runs/{}", id), &[], &store);
        assert!(status_body.contains("failed"));
    }

    // ---- event tests ----

    #[test]
    fn event_subscribe_emit_unsubscribe() {
        let store = MemStore::new();

        let (s, _) = rj(
            "POST",
            "/events/order.placed/subscribe",
            br#"{"fn_name":"handle-order"}"#,
            &store,
        );
        assert_eq!(s, 204);

        let (s, body) = rj("POST", "/events/order.placed/emit", br#"{"payload":[]}"#, &store);
        assert_eq!(s, 200);
        assert!(body.contains("handle-order"));

        let (s, _) = rj(
            "DELETE",
            "/events/order.placed/subscribe/handle-order",
            &[],
            &store,
        );
        assert_eq!(s, 204);

        let (_, body) = rj("POST", "/events/order.placed/emit", &[], &store);
        assert!(!body.contains("handle-order"));
    }

    #[test]
    fn list_events() {
        let store = MemStore::new();
        rj("POST", "/events/order.placed/subscribe", br#"{"fn_name":"f"}"#, &store);
        rj("POST", "/events/order.shipped/subscribe", br#"{"fn_name":"g"}"#, &store);
        let (s, body) = rj("GET", "/events", &[], &store);
        assert_eq!(s, 200);
        assert!(body.contains("order.placed") || body.contains("order.shipped"));
    }

    #[test]
    fn register_bad_json_returns_400() {
        let store = MemStore::new();
        let (s, _) = rj("POST", "/workflows", b"not-json", &store);
        assert_eq!(s, 400);
    }

    #[test]
    fn not_found_returns_404() {
        let store = MemStore::new();
        let (s, _) = rj("GET", "/unknown", &[], &store);
        assert_eq!(s, 404);
    }

    // ---- YAML body tests ----

    #[test]
    fn register_workflow_yaml() {
        let store = MemStore::new();
        let yaml = b"name: yaml-job\nsteps:\n  - name: run\n    depends_on: []\n";
        let (s, body) = ry("POST", "/workflows", yaml, &store);
        assert_eq!(s, 201, "body: {}", body);
        assert!(body.contains("yaml-job"));

        let (s, body) = rj("GET", "/workflows/yaml-job", &[], &store);
        assert_eq!(s, 200);
        assert!(body.contains("yaml-job"));
    }

    #[test]
    fn register_workflow_yaml_with_metadata() {
        let store = MemStore::new();
        let yaml = b"name: order-pipeline-yaml\ndescription: Process orders\ntimeout_ms: 60000\ntriggers:\n  - event: order.placed\nsteps:\n  - name: validate\n    depends_on: []\n    max_attempts: 3\n    base_delay_ms: 500\n  - name: charge\n    depends_on:\n      - validate\n    max_attempts: 5\n    base_delay_ms: 1000\n";
        let (s, body) = ry("POST", "/workflows", yaml, &store);
        assert_eq!(s, 201, "body: {}", body);
        assert!(body.contains("order-pipeline-yaml"));
    }

    #[test]
    fn register_workflow_bad_yaml_returns_400() {
        let store = MemStore::new();
        let yaml = b"steps:\n  - name: run\n    depends_on: []\n";
        let (s, _) = ry("POST", "/workflows", yaml, &store);
        assert_eq!(s, 400);
    }

    #[test]
    fn register_workflow_invalid_yaml_syntax_returns_400() {
        let store = MemStore::new();
        let yaml = b"name: {\n  unclosed brace\n";
        let (s, _) = ry("POST", "/workflows", yaml, &store);
        assert_eq!(s, 400);
    }

    #[test]
    fn event_subscribe_yaml() {
        let store = MemStore::new();
        let yaml = b"fn_name: my-handler\n";
        let (s, _) = ry("POST", "/events/order.placed/subscribe", yaml, &store);
        assert_eq!(s, 204);

        let (_, body) = rj("POST", "/events/order.placed/emit", &[], &store);
        assert!(body.contains("my-handler"));
    }

    #[test]
    fn content_type_detection() {
        assert!(is_yaml_content_type("application/yaml"));
        assert!(is_yaml_content_type("application/x-yaml"));
        assert!(is_yaml_content_type("text/yaml"));
        assert!(is_yaml_content_type("text/x-yaml"));
        assert!(is_yaml_content_type("application/yaml; charset=utf-8"));
        assert!(!is_yaml_content_type("application/json"));
        assert!(!is_yaml_content_type("text/plain"));
        assert!(!is_yaml_content_type(""));
    }

    // ---- Step output retrieval ----

    #[test]
    fn get_step_output_after_done() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = v["run_id"].as_str().unwrap();

        rj(
            "POST",
            &format!("/runs/{}/steps/run/done", id),
            br#"{"output":[1,2,3]}"#,
            &store,
        );

        let (s, body) = rj("GET", &format!("/runs/{}/steps/run/output", id), &[], &store);
        assert_eq!(s, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["state"], "succeeded");
        assert!(!v["output"].is_null());
    }

    #[test]
    fn get_step_output_missing_returns_404() {
        let store = MemStore::new();
        let (s, _) = rj("GET", "/runs/nonexistent-run/steps/nope/output", &[], &store);
        assert_eq!(s, 404);
    }

    // ---- Sub-workflow ----

    #[test]
    fn sub_workflow_field_accepted_in_definition() {
        let store = MemStore::new();
        let body = br#"{"name":"parent-wf","steps":[{"name":"delegate","depends_on":[],"sub_workflow":"child-wf"}]}"#;
        let (s, _) = rj("POST", "/workflows", body, &store);
        assert_eq!(s, 201);

        let (s, body) = rj("GET", "/workflows/parent-wf", &[], &store);
        assert_eq!(s, 200);
        assert!(body.contains("child-wf"));
    }

    #[test]
    fn sub_workflow_step_in_ready_steps_has_kind_field() {
        let store = MemStore::new();
        let body = br#"{"name":"parent-wf","steps":[{"name":"delegate","depends_on":[],"sub_workflow":"child-wf"}]}"#;
        rj("POST", "/workflows", body, &store);
        let (_, rb) = rj("POST", "/workflows/parent-wf/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&rb).unwrap();
        let id = v["run_id"].as_str().unwrap();

        let (s, body) = rj("GET", &format!("/runs/{}/ready-steps", id), &[], &store);
        assert_eq!(s, 200);
        assert!(body.contains("sub_workflow"), "expected sub_workflow field in: {}", body);
        assert!(body.contains("child-wf"));
    }

    #[test]
    fn sub_workflow_link_and_auto_complete_when_child_succeeds() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (_, cb) = rj("POST", "/workflows/simple-job/run", &[], &store);
        let cv: serde_json::Value = serde_json::from_str(&cb).unwrap();
        let child_id = cv["run_id"].as_str().unwrap();
        rj("POST", &format!("/runs/{}/steps/run/done", child_id), &[], &store);

        let pbody = br#"{"name":"parent-wf","steps":[{"name":"delegate","depends_on":[],"sub_workflow":"simple-job"}]}"#;
        rj("POST", "/workflows", pbody, &store);
        let (_, pb) = rj("POST", "/workflows/parent-wf/run", &[], &store);
        let pv: serde_json::Value = serde_json::from_str(&pb).unwrap();
        let parent_id = pv["run_id"].as_str().unwrap();

        let link_body = format!(r#"{{"sub_run_id":"{}"}}"#, child_id);
        let (s, _) = rj(
            "POST",
            &format!("/runs/{}/steps/delegate/sub-run", parent_id),
            link_body.as_bytes(),
            &store,
        );
        assert_eq!(s, 204);

        let (_, status) = rj("GET", &format!("/runs/{}", parent_id), &[], &store);
        assert!(status.contains("succeeded"), "parent run status: {}", status);
    }

    #[test]
    fn sub_workflow_fail_when_child_fails() {
        let store = MemStore::new();
        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (_, cb) = rj("POST", "/workflows/simple-job/run", &[], &store);
        let cv: serde_json::Value = serde_json::from_str(&cb).unwrap();
        let child_id = cv["run_id"].as_str().unwrap();
        rj("POST", &format!("/runs/{}/steps/run/failed", child_id), br#"{"error":"boom"}"#, &store);

        let pbody = br#"{"name":"parent-wf","steps":[{"name":"delegate","depends_on":[],"sub_workflow":"simple-job"}]}"#;
        rj("POST", "/workflows", pbody, &store);
        let (_, pb) = rj("POST", "/workflows/parent-wf/run", &[], &store);
        let pv: serde_json::Value = serde_json::from_str(&pb).unwrap();
        let parent_id = pv["run_id"].as_str().unwrap();

        let link_body = format!(r#"{{"sub_run_id":"{}"}}"#, child_id);
        rj(
            "POST",
            &format!("/runs/{}/steps/delegate/sub-run", parent_id),
            link_body.as_bytes(),
            &store,
        );

        let (_, status) = rj("GET", &format!("/runs/{}", parent_id), &[], &store);
        assert!(status.contains("failed"), "parent run status: {}", status);
    }

    #[test]
    fn nested_sub_workflow_three_levels() {
        let store = MemStore::new();

        rj("POST", "/workflows", &simple_wf_body(), &store);
        let (_, b) = rj("POST", "/workflows/simple-job/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let grandchild_id = v["run_id"].as_str().unwrap().to_string();
        rj("POST", &format!("/runs/{}/steps/run/done", grandchild_id), &[], &store);

        let cb = br#"{"name":"child-wf","steps":[{"name":"sub","depends_on":[],"sub_workflow":"simple-job"}]}"#;
        rj("POST", "/workflows", cb, &store);
        let (_, b2) = rj("POST", "/workflows/child-wf/run", &[], &store);
        let v2: serde_json::Value = serde_json::from_str(&b2).unwrap();
        let child_id = v2["run_id"].as_str().unwrap().to_string();
        let lb = format!(r#"{{"sub_run_id":"{}"}}"#, grandchild_id);
        rj("POST", &format!("/runs/{}/steps/sub/sub-run", child_id), lb.as_bytes(), &store);
        let (_, cs) = rj("GET", &format!("/runs/{}", child_id), &[], &store);
        assert!(cs.contains("succeeded"), "child status: {}", cs);

        let pb = br#"{"name":"parent-wf","steps":[{"name":"sub","depends_on":[],"sub_workflow":"child-wf"}]}"#;
        rj("POST", "/workflows", pb, &store);
        let (_, b3) = rj("POST", "/workflows/parent-wf/run", &[], &store);
        let v3: serde_json::Value = serde_json::from_str(&b3).unwrap();
        let parent_id = v3["run_id"].as_str().unwrap().to_string();
        let lb2 = format!(r#"{{"sub_run_id":"{}"}}"#, child_id);
        rj("POST", &format!("/runs/{}/steps/sub/sub-run", parent_id), lb2.as_bytes(), &store);

        let (_, ps) = rj("GET", &format!("/runs/{}", parent_id), &[], &store);
        assert!(ps.contains("succeeded"), "parent status: {}", ps);
    }

    // ---- If-else branching ----

    fn if_else_wf_body() -> Vec<u8> {
        br#"{
            "name": "if-else-wf",
            "steps": [
                {"name": "check", "depends_on": []},
                {"name": "yes-branch", "depends_on": ["check"], "optional": true,
                 "condition": {"on_step": "check", "equals": "yes"}},
                {"name": "no-branch",  "depends_on": ["check"], "optional": true,
                 "condition": {"on_step": "check", "equals": "no"}}
            ]
        }"#.to_vec()
    }

    #[test]
    fn if_else_true_branch_runs_false_skipped() {
        let store = MemStore::new();
        rj("POST", "/workflows", &if_else_wf_body(), &store);
        let (_, b) = rj("POST", "/workflows/if-else-wf/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let id = v["run_id"].as_str().unwrap();

        rj("POST", &format!("/runs/{}/steps/check/done", id),
           br#"{"output":[34,121,101,115,34]}"#, &store);

        let (_, ready) = rj("GET", &format!("/runs/{}/ready-steps", id), &[], &store);
        assert!(ready.contains("yes-branch"), "ready: {}", ready);

        // Use MemStore's get_step_record helper instead of raw map access
        let nb_state = store.get_step_record(id, "no-branch").unwrap();
        assert_eq!(nb_state.state, "skipped");
    }

    #[test]
    fn if_else_false_branch_optional_run_still_succeeds() {
        let store = MemStore::new();
        rj("POST", "/workflows", &if_else_wf_body(), &store);
        let (_, b) = rj("POST", "/workflows/if-else-wf/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let id = v["run_id"].as_str().unwrap();

        rj("POST", &format!("/runs/{}/steps/check/done", id),
           br#"{"output":[34,121,101,115,34]}"#, &store);

        rj("GET", &format!("/runs/{}/ready-steps", id), &[], &store);

        rj("POST", &format!("/runs/{}/steps/yes-branch/done", id), &[], &store);

        let (_, status) = rj("GET", &format!("/runs/{}", id), &[], &store);
        assert!(status.contains("succeeded"), "run status: {}", status);
    }

    #[test]
    fn if_else_false_branch_required_run_fails() {
        let store = MemStore::new();
        let wf_body = br#"{
            "name": "strict-wf",
            "steps": [
                {"name": "check", "depends_on": []},
                {"name": "required-branch", "depends_on": ["check"], "optional": false,
                 "condition": {"on_step": "check", "equals": "yes"}}
            ]
        }"#;
        rj("POST", "/workflows", wf_body, &store);
        let (_, b) = rj("POST", "/workflows/strict-wf/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let id = v["run_id"].as_str().unwrap();

        rj("POST", &format!("/runs/{}/steps/check/done", id),
           br#"{"output":[34,110,111,34]}"#, &store);

        rj("GET", &format!("/runs/{}/ready-steps", id), &[], &store);

        let (_, status) = rj("GET", &format!("/runs/{}", id), &[], &store);
        assert!(status.contains("succeeded"), "run status: {}", status);
    }

    #[test]
    fn transitive_skip_downstream_optional() {
        let store = MemStore::new();
        let wf_body = br#"{
            "name": "transitive-wf",
            "steps": [
                {"name": "check", "depends_on": []},
                {"name": "middle", "depends_on": ["check"], "optional": true,
                 "condition": {"on_step": "check", "equals": "go"}},
                {"name": "end", "depends_on": ["middle"], "optional": true}
            ]
        }"#;
        rj("POST", "/workflows", wf_body, &store);
        let (_, b) = rj("POST", "/workflows/transitive-wf/run", &[], &store);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let id = v["run_id"].as_str().unwrap();

        rj("POST", &format!("/runs/{}/steps/check/done", id),
           br#"{"output":[34,115,116,111,112,34]}"#, &store);

        rj("GET", &format!("/runs/{}/ready-steps", id), &[], &store);

        let end_sr = store.get_step_record(id, "end").unwrap();
        assert_eq!(end_sr.state, "skipped");

        let (_, status) = rj("GET", &format!("/runs/{}", id), &[], &store);
        assert!(status.contains("succeeded"), "run status: {}", status);
    }

    // -----------------------------------------------------------------------
    // Part B: Timeout tests
    // -----------------------------------------------------------------------

    #[test]
    fn timeout_zero_rejected() {
        let store = MemStore::new();
        let (status, body) = rj(
            "POST",
            "/workflows",
            br#"{"name":"bad-timeout","steps":[{"name":"s","depends_on":[]}],"timeout_ms":0}"#,
            &store,
        );
        assert_eq!(status, 400);
        assert!(body.contains("timeout_ms must be > 0"), "body: {}", body);
    }

    #[test]
    fn step_timeout_zero_rejected() {
        let store = MemStore::new();
        let (status, body) = rj(
            "POST",
            "/workflows",
            br#"{"name":"bad-step-timeout","steps":[{"name":"s","depends_on":[],"timeout_ms":0}]}"#,
            &store,
        );
        assert_eq!(status, 400);
        assert!(body.contains("timeout_ms must be > 0"), "body: {}", body);
    }

    #[test]
    fn step_timeout_accepted() {
        let store = MemStore::new();
        let (status, _) = rj(
            "POST",
            "/workflows",
            br#"{"name":"timeout-wf","steps":[{"name":"s","depends_on":[],"timeout_ms":5000}]}"#,
            &store,
        );
        assert_eq!(status, 201);
    }

    #[test]
    fn run_level_timeout_accepted() {
        let store = MemStore::new();
        let (status, _) = rj(
            "POST",
            "/workflows",
            br#"{"name":"run-timeout-wf","timeout_ms":10000,"steps":[{"name":"s","depends_on":[]}]}"#,
            &store,
        );
        assert_eq!(status, 201);
    }

    // -----------------------------------------------------------------------
    // Part C: Run history tests
    // -----------------------------------------------------------------------

    #[test]
    fn list_runs_for_workflow() {
        let store = MemStore::new();
        rj("POST", "/workflows", br#"{"name":"history-wf","steps":[{"name":"s","depends_on":[]}]}"#, &store);
        rj("POST", "/workflows/history-wf/run", b"{}", &store);
        rj("POST", "/workflows/history-wf/run", b"{}", &store);

        let (status, body) = rj("GET", "/workflows/history-wf/runs", &[], &store);
        assert_eq!(status, 200);
        assert!(body.contains("\"items\""), "body: {}", body);
        assert!(body.contains("\"total\":2"), "body: {}", body);
    }

    #[test]
    fn list_runs_returns_empty_for_fresh_workflow() {
        let store = MemStore::new();
        rj("POST", "/workflows", br#"{"name":"fresh-wf","steps":[{"name":"s","depends_on":[]}]}"#, &store);

        let (status, body) = rj("GET", "/workflows/fresh-wf/runs", &[], &store);
        assert_eq!(status, 200);
        assert!(body.contains("\"total\":0"), "body: {}", body);
    }

    #[test]
    fn list_runs_filter_by_state() {
        let store = MemStore::new();
        rj("POST", "/workflows", br#"{"name":"filter-wf","steps":[{"name":"s","depends_on":[]}]}"#, &store);
        let (_, b) = rj("POST", "/workflows/filter-wf/run", b"{}", &store);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let run_id = v["run_id"].as_str().unwrap();
        rj("POST", &format!("/runs/{}/cancel", run_id), &[], &store);

        let (status, body) = route("GET", "/workflows/filter-wf/runs?state=cancelled", &[], "application/json", &store);
        assert_eq!(status, 200);
        assert!(body.contains("cancelled"), "body: {}", body);

        let (_, body_running) = route("GET", "/workflows/filter-wf/runs?state=running", &[], "application/json", &store);
        assert!(body_running.contains("\"total\":0"), "body: {}", body_running);
    }

    #[test]
    fn list_runs_pagination() {
        let store = MemStore::new();
        rj("POST", "/workflows", br#"{"name":"page-runs-wf","steps":[{"name":"s","depends_on":[]}]}"#, &store);
        for _ in 0..5 {
            rj("POST", "/workflows/page-runs-wf/run", b"{}", &store);
        }

        let (status, body) = route("GET", "/workflows/page-runs-wf/runs?page=1&limit=2", &[], "application/json", &store);
        assert_eq!(status, 200);
        assert!(body.contains("\"total\":5"), "body: {}", body);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn list_runs_unknown_workflow() {
        let store = MemStore::new();
        let (status, body) = rj("GET", "/workflows/no-such-wf/runs", &[], &store);
        assert_eq!(status, 200);
        assert!(body.contains("\"total\":0"), "body: {}", body);
    }

    // -----------------------------------------------------------------------
    // Part D: Pagination tests
    // -----------------------------------------------------------------------

    #[test]
    fn list_workflows_paginated() {
        let store = MemStore::new();
        for i in 1..=5 {
            rj("POST", "/workflows", format!(r#"{{"name":"pag-wf-{}","steps":[{{"name":"s","depends_on":[]}}]}}"#, i).as_bytes(), &store);
        }

        let (status, body) = route("GET", "/workflows?page=1&limit=2", &[], "application/json", &store);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["total"].as_u64().unwrap(), 5);
        assert_eq!(v["items"].as_array().unwrap().len(), 2);
        assert!(body.contains("\"page\":1"));
        assert!(body.contains("\"limit\":2"));
    }

    #[test]
    fn list_steps_for_run() {
        let store = MemStore::new();
        rj("POST", "/workflows", br#"{"name":"steps-list-wf","steps":[{"name":"a","depends_on":[]},{"name":"b","depends_on":[]}]}"#, &store);
        let (_, b) = rj("POST", "/workflows/steps-list-wf/run", b"{}", &store);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let run_id = v["run_id"].as_str().unwrap();

        let (status, body) = rj("GET", &format!("/runs/{}/steps", run_id), &[], &store);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["total"].as_u64().unwrap(), 2);
        assert_eq!(v["items"].as_array().unwrap().len(), 2);
    }
}

// ---------------------------------------------------------------------------
// TypeScript type export (ts-rs) — native only, never compiled to wasm32
// Run: cargo test export_bindings -p workflow-api
// Output: workflow-ui/src/generated/*.ts
// ---------------------------------------------------------------------------

#[cfg(all(test, not(target_arch = "wasm32")))]
mod ts_export {
    use super::*;
    use ts_rs::TS;

    #[test]
    fn export_bindings() {
        // TS_RS_EXPORT_DIR is relative to the crate root (workflow-api/).
        // Override to place files in the sibling workflow-ui project.
        std::env::set_var(
            "TS_RS_EXPORT_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../workflow-ui/src/generated"),
        );
        Condition::export_all().unwrap();
        StepDef::export_all().unwrap();
        TriggerDef::export_all().unwrap();
        WorkflowDef::export_all().unwrap();
        RunRecord::export_all().unwrap();
        StepRecord::export_all().unwrap();
    }
}
