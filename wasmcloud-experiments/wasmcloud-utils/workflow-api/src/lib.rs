// Workflow HTTP API component.
// Targets the `workflow-api-component` world defined in
// wit/wasmcloud-workflow-api/workflow-api.wit.
//
// Exports:  wasi:http/incoming-handler
// Imports:  wasi:keyvalue/store
//
// KV schema (bucket: "workflow"):
//   wf-def:<name>             → JSON WorkflowDef
//   wf-run:<run-id>           → JSON RunRecord
//   step:<run-id>:<step-name> → JSON StepRecord
//   evt:<event-name>          → JSON list<string>  (subscriber fn-names)

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "workflow-api-component",
    path: "../wit/wasmcloud-workflow-api",
    generate_all,
});

use serde::{Deserialize, Serialize};

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
        // Parse YAML then round-trip through JSON Value so downstream
        // serde_json::from_value calls work unchanged.
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    pub on_step: String,
    pub equals: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepDef {
    pub name: String,
    pub depends_on: Vec<String>,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    #[serde(default)]
    pub base_delay_ms: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDef {
    pub event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub triggers: Vec<TriggerDef>,
    pub steps: Vec<StepDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub wf_name: String,
    pub state: String,
    pub idem_key: Option<String>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub state: String,
    pub attempt: u32,
    pub scheduled_at_ms: u64,
    pub output: Option<Vec<u8>>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

pub fn validate_workflow(def: &WorkflowDef) -> Result<(), String> {
    // 1. Name non-empty and alphanumeric + `-_`
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

    // 2. At least one step
    if def.steps.is_empty() {
        return Err("workflow must have at least one step".into());
    }

    // 3. No duplicate step names
    let mut seen = std::collections::HashSet::new();
    for step in &def.steps {
        if step.name.is_empty() {
            return Err("step name must not be empty".into());
        }
        if !seen.insert(step.name.clone()) {
            return Err(format!("duplicate step name '{}'", step.name));
        }
    }

    // 4. All depends_on reference existing steps
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

    // 5. max_attempts >= 1
    for step in &def.steps {
        if step.max_attempts < 1 {
            return Err(format!(
                "step '{}' max_attempts must be >= 1",
                step.name
            ));
        }
    }

    // 6. Validate sub_workflow names (alphanumeric + `-_`)
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

    // 7. Validate condition.on_step references existing steps
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

    // 8. No dependency cycles (DFS topological sort)
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

    // 0 = unvisited, 1 = in-stack, 2 = done
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
// Simple time stub
// ---------------------------------------------------------------------------

pub fn now_ms() -> u64 {
    // In WASM: use wasi:clocks/wall-clock. For tests, return 0.
    0
}

// ---------------------------------------------------------------------------
// KV helpers (native-only stubs used by tests; wasm32 uses the real import)
// ---------------------------------------------------------------------------

/// KV operations abstracted so unit tests can use in-memory store.
pub trait KvStore {
    fn kv_get(&self, key: &str) -> Option<Vec<u8>>;
    fn kv_set(&self, key: &str, value: Vec<u8>);
    fn kv_delete(&self, key: &str);
    fn kv_list_prefix(&self, prefix: &str) -> Vec<String>;
}

// ---------------------------------------------------------------------------
// Business logic — pure functions operating on a KvStore trait object
// ---------------------------------------------------------------------------

pub fn handle_register_workflow(
    body: &[u8],
    content_type: &str,
    kv: &dyn KvStore,
) -> (u16, String) {
    let def: WorkflowDef = match parse_body(body, content_type) {
        Ok(d) => d,
        Err(e) => return (400, format!(r#"{{"error":"{}"}}"#, e)),
    };
    if let Err(msg) = validate_workflow(&def) {
        return (400, format!(r#"{{"error":"{}"}}"#, msg));
    }
    let key = format!("wf-def:{}", def.name);
    let json = serde_json::to_vec(&def).unwrap();
    kv.kv_set(&key, json);
    (201, format!(r#"{{"name":"{}","created":true}}"#, def.name))
}

pub fn handle_list_workflows(kv: &dyn KvStore) -> (u16, String) {
    let keys = kv.kv_list_prefix("wf-def:");
    let names: Vec<String> = keys
        .iter()
        .filter_map(|k| k.strip_prefix("wf-def:").map(|s| format!(r#""{}""#, s)))
        .collect();
    (200, format!("[{}]", names.join(",")))
}

pub fn handle_get_workflow(name: &str, kv: &dyn KvStore) -> (u16, String) {
    let key = format!("wf-def:{}", name);
    match kv.kv_get(&key) {
        Some(v) => (200, String::from_utf8_lossy(&v).into_owned()),
        None => (404, format!(r#"{{"error":"workflow '{}' not found"}}"#, name)),
    }
}

pub fn handle_delete_workflow(name: &str, kv: &dyn KvStore) -> (u16, String) {
    let key = format!("wf-def:{}", name);
    kv.kv_delete(&key);
    (204, String::new())
}

pub fn handle_start_run(
    wf_name: &str,
    body: &[u8],
    content_type: &str,
    kv: &dyn KvStore,
) -> (u16, String) {
    // Check workflow exists
    let def_key = format!("wf-def:{}", wf_name);
    let def_bytes = match kv.kv_get(&def_key) {
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
    let run_id = if let Some(ref ik) = req.idem_key {
        // Check for existing run with same idem_key
        let existing_keys = kv.kv_list_prefix("wf-run:");
        for k in &existing_keys {
            if let Some(v) = kv.kv_get(k) {
                if let Ok(r) = serde_json::from_slice::<RunRecord>(&v) {
                    if r.idem_key.as_deref() == Some(ik.as_str())
                        && r.wf_name == wf_name
                    {
                        return (
                            200,
                            format!(r#"{{"run_id":"{}","existing":true}}"#, r.run_id),
                        );
                    }
                }
            }
        }
        format!("wfrun-{}-{}-{}", wf_name, ik, ts)
    } else {
        format!("wfrun-{}-{}", wf_name, ts)
    };

    let run = RunRecord {
        run_id: run_id.clone(),
        wf_name: wf_name.to_string(),
        state: "running".to_string(),
        idem_key: req.idem_key,
        created_at_ms: ts,
    };
    kv.kv_set(
        &format!("wf-run:{}", run_id),
        serde_json::to_vec(&run).unwrap(),
    );

    // Initialise step records for all steps
    for step in &def.steps {
        let sr = StepRecord {
            state: "pending".to_string(),
            attempt: 0,
            scheduled_at_ms: ts,
            output: None,
            error: None,
        };
        kv.kv_set(
            &format!("step:{}:{}", run_id, step.name),
            serde_json::to_vec(&sr).unwrap(),
        );
    }

    (201, format!(r#"{{"run_id":"{}"}}"#, run_id))
}

pub fn handle_get_run(run_id: &str, kv: &dyn KvStore) -> (u16, String) {
    let key = format!("wf-run:{}", run_id);
    match kv.kv_get(&key) {
        Some(v) => (200, String::from_utf8_lossy(&v).into_owned()),
        None => (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id)),
    }
}

pub fn handle_cancel_run(run_id: &str, kv: &dyn KvStore) -> (u16, String) {
    let key = format!("wf-run:{}", run_id);
    match kv.kv_get(&key) {
        None => (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id)),
        Some(v) => {
            let mut run: RunRecord = match serde_json::from_slice(&v) {
                Ok(r) => r,
                Err(_) => return (500, r#"{"error":"corrupt run record"}"#.into()),
            };
            run.state = "cancelled".to_string();
            kv.kv_set(&key, serde_json::to_vec(&run).unwrap());
            (204, String::new())
        }
    }
}

/// GET /runs/:run_id/steps/:step/output
pub fn handle_get_step_output(run_id: &str, step_name: &str, kv: &dyn KvStore) -> (u16, String) {
    let step_key = format!("step:{}:{}", run_id, step_name);
    match kv.kv_get(&step_key) {
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
                    // Try to re-parse as JSON value; fall back to base64-like array.
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
/// Body: {"sub_run_id": "wfrun-..."}
/// Links a child run to the given step and auto-advances if the child is done.
pub fn handle_link_sub_run(
    run_id: &str,
    step_name: &str,
    body: &[u8],
    content_type: &str,
    kv: &dyn KvStore,
) -> (u16, String) {
    #[derive(Deserialize)]
    struct SubRunReq {
        sub_run_id: String,
    }
    let req: SubRunReq = match parse_body(body, content_type) {
        Ok(r) => r,
        Err(e) => return (400, format!(r#"{{"error":"{}"}}"#, e)),
    };

    // Ensure the parent step exists
    let step_key = format!("step:{}:{}", run_id, step_name);
    if kv.kv_get(&step_key).is_none() {
        return (
            404,
            format!(r#"{{"error":"step '{}' not found for run '{}'"  }}"#, step_name, run_id),
        );
    }

    // Store the sub-run link
    let link_key = format!("sub-run:{}:{}", run_id, step_name);
    kv.kv_set(&link_key, req.sub_run_id.as_bytes().to_vec());

    // Auto-advance: if child run already succeeded/failed, reflect it now.
    advance_sub_workflow_step(run_id, step_name, &req.sub_run_id, kv);

    (204, String::new())
}

/// Check whether a sub-workflow step's child run has completed and advance
/// the parent step accordingly. Called from both link_sub_run and ready_steps.
fn advance_sub_workflow_step(
    parent_run_id: &str,
    step_name: &str,
    child_run_id: &str,
    kv: &dyn KvStore,
) {
    let child_run_key = format!("wf-run:{}", child_run_id);
    let child_run: RunRecord = match kv
        .kv_get(&child_run_key)
        .and_then(|v| serde_json::from_slice(&v).ok())
    {
        Some(r) => r,
        None => return,
    };

    let step_key = format!("step:{}:{}", parent_run_id, step_name);
    let sr: StepRecord = match kv
        .kv_get(&step_key)
        .and_then(|v| serde_json::from_slice(&v).ok())
    {
        Some(r) => r,
        None => return,
    };

    // Only act if parent step is still pending/running
    if sr.state != "pending" && sr.state != "running" {
        return;
    }

    match child_run.state.as_str() {
        "succeeded" => {
            // Copy child's final output (last step output). For simplicity,
            // we just mark the parent step succeeded with no output.
            let updated = StepRecord {
                state: "succeeded".to_string(),
                attempt: sr.attempt + 1,
                output: None,
                error: None,
                ..sr
            };
            kv.kv_set(&step_key, serde_json::to_vec(&updated).unwrap());
            maybe_complete_run(parent_run_id, kv);
        }
        "failed" | "cancelled" => {
            let updated = StepRecord {
                state: "failed".to_string(),
                attempt: sr.attempt + 1,
                error: Some(format!("child run {} {}", child_run_id, child_run.state)),
                ..sr
            };
            kv.kv_set(&step_key, serde_json::to_vec(&updated).unwrap());
            // Fail the parent run too
            let run_key = format!("wf-run:{}", parent_run_id);
            if let Some(v) = kv.kv_get(&run_key) {
                if let Ok(mut run) = serde_json::from_slice::<RunRecord>(&v) {
                    if run.state == "running" {
                        run.state = "failed".to_string();
                        kv.kv_set(&run_key, serde_json::to_vec(&run).unwrap());
                    }
                }
            }
        }
        _ => {} // still running
    }
}

pub fn handle_ready_steps(run_id: &str, kv: &dyn KvStore) -> (u16, String) {
    // Load the run
    let run_key = format!("wf-run:{}", run_id);
    let run_bytes = match kv.kv_get(&run_key) {
        Some(b) => b,
        None => return (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id)),
    };
    let run: RunRecord = match serde_json::from_slice(&run_bytes) {
        Ok(r) => r,
        Err(_) => return (500, r#"{"error":"corrupt run record"}"#.into()),
    };

    // Load workflow definition
    let def_key = format!("wf-def:{}", run.wf_name);
    let def_bytes = match kv.kv_get(&def_key) {
        Some(b) => b,
        None => return (500, r#"{"error":"workflow definition missing"}"#.into()),
    };
    let def: WorkflowDef = match serde_json::from_slice(&def_bytes) {
        Ok(d) => d,
        Err(_) => return (500, r#"{"error":"corrupt workflow definition"}"#.into()),
    };

    let ts = now_ms();
    let mut ready = Vec::new();

    for step in &def.steps {
        let step_key = format!("step:{}:{}", run_id, step.name);
        let sr: StepRecord = match kv.kv_get(&step_key) {
            Some(v) => serde_json::from_slice(&v).unwrap_or(StepRecord {
                state: "pending".to_string(),
                attempt: 0,
                scheduled_at_ms: 0,
                output: None,
                error: None,
            }),
            None => continue,
        };

        if sr.state != "pending" {
            // Sub-workflow auto-advance: check if a linked child run finished.
            if sr.state == "pending" || sr.state == "running" {
                let link_key = format!("sub-run:{}:{}", run_id, step.name);
                if let Some(child_id_bytes) = kv.kv_get(&link_key) {
                    let child_id = String::from_utf8_lossy(&child_id_bytes).into_owned();
                    advance_sub_workflow_step(run_id, &step.name, &child_id, kv);
                }
            }
            continue;
        }
        if sr.scheduled_at_ms > ts {
            continue;
        }

        // Check all deps are succeeded (or skipped-optional)
        let deps_ok = step.depends_on.iter().all(|dep| {
            let dep_key = format!("step:{}:{}", run_id, dep);
            kv.kv_get(&dep_key)
                .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                .map(|dr| dr.state == "succeeded" || dr.state == "skipped")
                .unwrap_or(false)
        });

        if !deps_ok {
            continue;
        }

        // Evaluate condition for if-else branching
        if let Some(ref cond) = step.condition {
            let on_step_key = format!("step:{}:{}", run_id, cond.on_step);
            let on_step_output: Option<Vec<u8>> = kv
                .kv_get(&on_step_key)
                .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                .and_then(|sr| sr.output);

            let condition_met = match on_step_output {
                Some(bytes) => serde_json::from_slice::<serde_json::Value>(&bytes)
                    .map(|val| val == cond.equals)
                    .unwrap_or(false),
                None => serde_json::Value::Null == cond.equals,
            };

            if !condition_met {
                // Skip this step immediately
                let skipped = StepRecord {
                    state: "skipped".to_string(),
                    ..sr
                };
                kv.kv_set(&step_key, serde_json::to_vec(&skipped).unwrap());
                // Apply transitive skipping and check run completion
                apply_transitive_skips(run_id, &def, kv);
                maybe_complete_run(run_id, kv);
                continue;
            }
        }

        // Build ready-step JSON
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
    kv: &dyn KvStore,
) -> (u16, String) {
    let step_key = format!("step:{}:{}", run_id, step_name);
    let sr = match kv.kv_get(&step_key) {
        Some(v) => serde_json::from_slice::<StepRecord>(&v).unwrap_or_default_step(),
        None => return (404, format!(r#"{{"error":"step '{}' not found for run '{}'"  }}"#, step_name, run_id)),
    };

    #[derive(Deserialize, Default)]
    struct DoneReq {
        output: Option<Vec<u8>>,
    }
    let req: DoneReq = if body.is_empty() {
        DoneReq::default()
    } else {
        parse_body(body, content_type).unwrap_or_default()
    };

    let updated = StepRecord {
        state: "succeeded".to_string(),
        attempt: sr.attempt + 1,
        output: req.output,
        ..sr
    };
    kv.kv_set(&step_key, serde_json::to_vec(&updated).unwrap());
    maybe_complete_run(run_id, kv);
    (204, String::new())
}

pub fn handle_step_failed(
    run_id: &str,
    step_name: &str,
    body: &[u8],
    content_type: &str,
    kv: &dyn KvStore,
) -> (u16, String) {
    let run_key = format!("wf-run:{}", run_id);
    let run_bytes = match kv.kv_get(&run_key) {
        Some(b) => b,
        None => return (404, format!(r#"{{"error":"run '{}' not found"}}"#, run_id)),
    };
    let run: RunRecord = match serde_json::from_slice(&run_bytes) {
        Ok(r) => r,
        Err(_) => return (500, r#"{"error":"corrupt run record"}"#.into()),
    };

    let def_key = format!("wf-def:{}", run.wf_name);
    let def_bytes = match kv.kv_get(&def_key) {
        Some(b) => b,
        None => return (500, r#"{"error":"workflow definition missing"}"#.into()),
    };
    let def: WorkflowDef = match serde_json::from_slice(&def_bytes) {
        Ok(d) => d,
        Err(_) => return (500, r#"{"error":"corrupt workflow definition"}"#.into()),
    };

    let step_key = format!("step:{}:{}", run_id, step_name);
    let sr = match kv.kv_get(&step_key) {
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
        // Mark failed, fail the whole run
        let mut run_updated: RunRecord = serde_json::from_slice(&run_bytes).unwrap();
        run_updated.state = "failed".to_string();
        kv.kv_set(&run_key, serde_json::to_vec(&run_updated).unwrap());
        StepRecord {
            state: "failed".to_string(),
            attempt: new_attempt,
            error: req.error,
            ..sr
        }
    } else {
        // Schedule retry with exponential backoff (capped at 60s)
        let delay = (base_delay * (1u64 << new_attempt.min(6))).min(60_000);
        StepRecord {
            state: "pending".to_string(),
            attempt: new_attempt,
            scheduled_at_ms: now_ms() + delay,
            error: req.error,
            ..sr
        }
    };
    kv.kv_set(&step_key, serde_json::to_vec(&updated).unwrap());
    (204, String::new())
}

fn maybe_complete_run(run_id: &str, kv: &dyn KvStore) {
    let run_key = format!("wf-run:{}", run_id);
    let run_bytes = match kv.kv_get(&run_key) {
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

    // Load workflow def to know which steps are optional
    let def_key = format!("wf-def:{}", run.wf_name);
    let def: WorkflowDef = match kv
        .kv_get(&def_key)
        .and_then(|v| serde_json::from_slice(&v).ok())
    {
        Some(d) => d,
        None => return,
    };

    let step_keys = kv.kv_list_prefix(&format!("step:{}:", run_id));
    if step_keys.is_empty() {
        return;
    }

    let mut all_terminal = true;
    let mut any_required_skipped = false;

    for k in &step_keys {
        let sr: StepRecord = match kv
            .kv_get(k)
            .and_then(|v| serde_json::from_slice(&v).ok())
        {
            Some(r) => r,
            None => { all_terminal = false; break; }
        };
        match sr.state.as_str() {
            "succeeded" => {}
            "skipped" => {
                // Find step def to check optional flag
                let step_name = k
                    .strip_prefix(&format!("step:{}:", run_id))
                    .unwrap_or("");
                let is_optional = def
                    .steps
                    .iter()
                    .find(|s| s.name == step_name)
                    .map(|s| s.optional)
                    .unwrap_or(false);
                if !is_optional {
                    any_required_skipped = true;
                }
            }
            _ => { all_terminal = false; break; }
        }
    }

    if all_terminal {
        run.state = if any_required_skipped {
            "failed".to_string()
        } else {
            "succeeded".to_string()
        };
        kv.kv_set(&run_key, serde_json::to_vec(&run).unwrap());
    }
}

/// Transitively skip any downstream steps whose only unblocked path goes
/// through a skipped step (DFS/BFS propagation).
fn apply_transitive_skips(run_id: &str, def: &WorkflowDef, kv: &dyn KvStore) {
    // Build adjacency: step → steps that depend on it
    let mut dependents: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for step in &def.steps {
        for dep in &step.depends_on {
            dependents
                .entry(dep.as_str())
                .or_default()
                .push(step.name.as_str());
        }
    }

    // Collect currently-skipped steps as seeds
    let mut to_visit: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    for step in &def.steps {
        let sk = format!("step:{}:{}", run_id, step.name);
        if let Some(v) = kv.kv_get(&sk) {
            if let Ok(sr) = serde_json::from_slice::<StepRecord>(&v) {
                if sr.state == "skipped" {
                    to_visit.push_back(step.name.clone());
                }
            }
        }
    }

    while let Some(skipped_name) = to_visit.pop_front() {
        for &dependent in dependents.get(skipped_name.as_str()).unwrap_or(&vec![]) {
            let dep_key = format!("step:{}:{}", run_id, dependent);
            let sr: StepRecord = match kv
                .kv_get(&dep_key)
                .and_then(|v| serde_json::from_slice(&v).ok())
            {
                Some(r) => r,
                None => continue,
            };
            if sr.state != "pending" {
                continue;
            }
            // Check if ALL non-skipped predecessors are succeeded; if there's
            // any skipped predecessor and no succeeded path, propagate skip.
            let step_def = def.steps.iter().find(|s| s.name == dependent);
            let all_deps_resolved = step_def.map(|sd| {
                sd.depends_on.iter().all(|dep| {
                    let dk = format!("step:{}:{}", run_id, dep);
                    kv.kv_get(&dk)
                        .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                        .map(|dr| dr.state == "succeeded" || dr.state == "skipped")
                        .unwrap_or(false)
                })
            }).unwrap_or(false);

            if all_deps_resolved {
                // If any dep is skipped and there is no condition overriding, skip this too
                let has_skipped_dep = step_def.map(|sd| {
                    sd.depends_on.iter().any(|dep| {
                        let dk = format!("step:{}:{}", run_id, dep);
                        kv.kv_get(&dk)
                            .and_then(|v| serde_json::from_slice::<StepRecord>(&v).ok())
                            .map(|dr| dr.state == "skipped")
                            .unwrap_or(false)
                    })
                }).unwrap_or(false);

                // Only auto-skip if step has no condition of its own
                // (conditioned steps are handled in ready_steps)
                let has_condition = step_def.map(|sd| sd.condition.is_some()).unwrap_or(false);
                if has_skipped_dep && !has_condition {
                    let updated = StepRecord {
                        state: "skipped".to_string(),
                        ..sr
                    };
                    kv.kv_set(&dep_key, serde_json::to_vec(&updated).unwrap());
                    to_visit.push_back(dependent.to_string());
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

pub fn handle_event_subscribe(event_name: &str, body: &[u8], content_type: &str, kv: &dyn KvStore) -> (u16, String) {
    #[derive(Deserialize)]
    struct SubReq {
        fn_name: String,
    }
    let req: SubReq = match parse_body(body, content_type) {
        Ok(r) => r,
        Err(e) => return (400, format!(r#"{{"error":"{}"}}"#, e)),
    };
    let key = format!("evt:{}", event_name);
    let mut subs: Vec<String> = kv
        .kv_get(&key)
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();
    if !subs.contains(&req.fn_name) {
        subs.push(req.fn_name);
        kv.kv_set(&key, serde_json::to_vec(&subs).unwrap());
    }
    (204, String::new())
}

pub fn handle_event_unsubscribe(
    event_name: &str,
    fn_name: &str,
    kv: &dyn KvStore,
) -> (u16, String) {
    let key = format!("evt:{}", event_name);
    let mut subs: Vec<String> = kv
        .kv_get(&key)
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();
    subs.retain(|s| s != fn_name);
    kv.kv_set(&key, serde_json::to_vec(&subs).unwrap());
    (204, String::new())
}

pub fn handle_event_emit(event_name: &str, body: &[u8], _content_type: &str, kv: &dyn KvStore) -> (u16, String) {
    let _ = body; // payload stored but not processed further in this component
    let key = format!("evt:{}", event_name);
    let subs: Vec<String> = kv
        .kv_get(&key)
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();
    let names: Vec<String> = subs.iter().map(|s| format!(r#""{}""#, s)).collect();
    (200, format!("[{}]", names.join(",")))
}

pub fn handle_list_events(kv: &dyn KvStore) -> (u16, String) {
    let keys = kv.kv_list_prefix("evt:");
    let names: Vec<String> = keys
        .iter()
        .filter_map(|k| k.strip_prefix("evt:").map(|s| format!(r#""{}""#, s)))
        .collect();
    (200, format!("[{}]", names.join(",")))
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
            output: None,
            error: None,
        })
    }
}

// ---------------------------------------------------------------------------
// HTTP router (pure, called from both native tests and WASM handler)
// ---------------------------------------------------------------------------

pub fn route(method: &str, path: &str, body: &[u8], content_type: &str, kv: &dyn KvStore) -> (u16, String) {
    // Strip query string
    let path = path.split('?').next().unwrap_or(path);

    match (method, path) {
        ("POST", "/workflows") => handle_register_workflow(body, content_type, kv),
        ("GET", "/workflows") => handle_list_workflows(kv),
        ("GET", "/events") => handle_list_events(kv),

        _ if path.starts_with("/workflows/") => {
            let rest = &path["/workflows/".len()..];
            if rest.ends_with("/run") && method == "POST" {
                let name = &rest[..rest.len() - "/run".len()];
                handle_start_run(name, body, content_type, kv)
            } else if method == "GET" && !rest.contains('/') {
                handle_get_workflow(rest, kv)
            } else if method == "DELETE" && !rest.contains('/') {
                handle_delete_workflow(rest, kv)
            } else {
                (404, r#"{"error":"not found"}"#.to_string())
            }
        }

        _ if path.starts_with("/runs/") => {
            let rest = &path["/runs/".len()..];
            // /runs/:run_id
            if !rest.contains('/') {
                if method == "GET" {
                    handle_get_run(rest, kv)
                } else {
                    (405, r#"{"error":"method not allowed"}"#.to_string())
                }
            }
            // /runs/:run_id/cancel
            else if rest.ends_with("/cancel") && method == "POST" {
                let run_id = &rest[..rest.len() - "/cancel".len()];
                handle_cancel_run(run_id, kv)
            }
            // /runs/:run_id/ready-steps
            else if rest.ends_with("/ready-steps") && method == "GET" {
                let run_id = &rest[..rest.len() - "/ready-steps".len()];
                handle_ready_steps(run_id, kv)
            }
            // /runs/:run_id/steps/:step/{done|failed|output|sub-run}
            else if let Some(steps_rest) = rest.find("/steps/").map(|i| &rest[i + "/steps/".len()..]) {
                let run_id = &rest[..rest.find("/steps/").unwrap()];
                if steps_rest.ends_with("/done") && method == "POST" {
                    let step_name = &steps_rest[..steps_rest.len() - "/done".len()];
                    handle_step_done(run_id, step_name, body, content_type, kv)
                } else if steps_rest.ends_with("/failed") && method == "POST" {
                    let step_name = &steps_rest[..steps_rest.len() - "/failed".len()];
                    handle_step_failed(run_id, step_name, body, content_type, kv)
                } else if steps_rest.ends_with("/output") && method == "GET" {
                    let step_name = &steps_rest[..steps_rest.len() - "/output".len()];
                    handle_get_step_output(run_id, step_name, kv)
                } else if steps_rest.ends_with("/sub-run") && method == "POST" {
                    let step_name = &steps_rest[..steps_rest.len() - "/sub-run".len()];
                    handle_link_sub_run(run_id, step_name, body, content_type, kv)
                } else {
                    (404, r#"{"error":"not found"}"#.to_string())
                }
            } else {
                (404, r#"{"error":"not found"}"#.to_string())
            }
        }

        _ if path.starts_with("/events/") => {
            let rest = &path["/events/".len()..];
            // /events/:name/subscribe  (POST)
            if rest.ends_with("/subscribe") && method == "POST" {
                let name = &rest[..rest.len() - "/subscribe".len()];
                handle_event_subscribe(name, body, content_type, kv)
            }
            // /events/:name/emit  (POST)
            else if rest.ends_with("/emit") && method == "POST" {
                let name = &rest[..rest.len() - "/emit".len()];
                handle_event_emit(name, body, content_type, kv)
            }
            // /events/:name/subscribe/:fn_name  (DELETE)
            else if rest.contains("/subscribe/") && method == "DELETE" {
                let idx = rest.find("/subscribe/").unwrap();
                let name = &rest[..idx];
                let fn_name = &rest[idx + "/subscribe/".len()..];
                handle_event_unsubscribe(name, fn_name, kv)
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

/// WASM KV store backed by wasi:keyvalue/store import.
#[cfg(target_arch = "wasm32")]
struct WasiKv {
    bucket: wasi::keyvalue::store::Bucket,
}

#[cfg(target_arch = "wasm32")]
impl WasiKv {
    fn open() -> Self {
        let bucket = wasi::keyvalue::store::open("workflow")
            .expect("failed to open workflow KV bucket");
        WasiKv { bucket }
    }
}

#[cfg(target_arch = "wasm32")]
impl KvStore for WasiKv {
    fn kv_get(&self, key: &str) -> Option<Vec<u8>> {
        self.bucket.get(key).ok().flatten()
    }

    fn kv_set(&self, key: &str, value: Vec<u8>) {
        let _ = self.bucket.set(key, &value);
    }

    fn kv_delete(&self, key: &str) {
        let _ = self.bucket.delete(key);
    }

    fn kv_list_prefix(&self, prefix: &str) -> Vec<String> {
        self.bucket
            .list_keys(None)
            .map(|r| r.keys)
            .unwrap_or_default()
            .into_iter()
            .filter(|k: &String| k.starts_with(prefix))
            .collect()
    }
}

#[cfg(target_arch = "wasm32")]
impl exports::wasi::http::incoming_handler::Guest for WorkflowApi {
    fn handle(
        request: wasi::http::types::IncomingRequest,
        response_out: wasi::http::types::ResponseOutparam,
    ) {
        use wasi::http::types::{Headers, OutgoingBody, OutgoingResponse};

        let method = format!("{:?}", request.method());
        let path = request.path_with_query().unwrap_or_default();

        let body: Vec<u8> = request
            .consume()
            .ok()
            .and_then(|b| {
                let stream = b.stream().ok()?;
                let mut buf = Vec::new();
                loop {
                    match stream.read(4096) {
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

        let kv = WasiKv::open();
        let (status, resp_body) = route(&method, &path, &body, &content_type, &kv);

        let headers = Headers::new();
        let _ = headers.append(
            &"content-type".to_string(),
            &b"application/json".to_vec(),
        );
        let resp = OutgoingResponse::new(headers);
        resp.set_status_code(status).ok();

        if !resp_body.is_empty() {
            if let Ok(ob) = resp.body() {
                if let Ok(stream) = ob.write() {
                    let _ = stream.blocking_write_and_flush(resp_body.as_bytes());
                }
                OutgoingBody::finish(ob, None).ok();
            }
        }

        wasi::http::types::ResponseOutparam::set(response_out, Ok(resp));
    }
}

#[cfg(target_arch = "wasm32")]
export!(WorkflowApi);

// ---------------------------------------------------------------------------
// In-memory KvStore for tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub mod test_kv {
    use super::KvStore;
    use std::cell::RefCell;
    use std::collections::HashMap;

    pub struct MemKv(pub RefCell<HashMap<String, Vec<u8>>>);

    impl MemKv {
        pub fn new() -> Self {
            MemKv(RefCell::new(HashMap::new()))
        }
    }

    impl KvStore for MemKv {
        fn kv_get(&self, key: &str) -> Option<Vec<u8>> {
            self.0.borrow().get(key).cloned()
        }
        fn kv_set(&self, key: &str, value: Vec<u8>) {
            self.0.borrow_mut().insert(key.to_string(), value);
        }
        fn kv_delete(&self, key: &str) {
            self.0.borrow_mut().remove(key);
        }
        fn kv_list_prefix(&self, prefix: &str) -> Vec<String> {
            self.0
                .borrow()
                .keys()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect()
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_kv::MemKv;

    /// Convenience: route with JSON content-type (used by most existing tests).
    fn rj(method: &str, path: &str, body: &[u8], kv: &dyn KvStore) -> (u16, String) {
        route(method, path, body, "application/json", kv)
    }

    /// Convenience: route with YAML content-type.
    fn ry(method: &str, path: &str, body: &[u8], kv: &dyn KvStore) -> (u16, String) {
        route(method, path, body, "application/yaml", kv)
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
        let kv = MemKv::new();
        let (s, _) = rj("POST", "/workflows", &simple_wf_body(), &kv);
        assert_eq!(s, 201);
        let (s, body) = rj("GET", "/workflows", &[], &kv);
        assert_eq!(s, 200);
        assert!(body.contains("simple-job"));
    }

    #[test]
    fn register_and_get() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (s, body) = rj("GET", "/workflows/simple-job", &[], &kv);
        assert_eq!(s, 200);
        assert!(body.contains("simple-job"));
    }

    #[test]
    fn get_missing_workflow() {
        let kv = MemKv::new();
        let (s, _) = rj("GET", "/workflows/nope", &[], &kv);
        assert_eq!(s, 404);
    }

    #[test]
    fn delete_workflow() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (s, _) = rj("DELETE", "/workflows/simple-job", &[], &kv);
        assert_eq!(s, 204);
        let (s, _) = rj("GET", "/workflows/simple-job", &[], &kv);
        assert_eq!(s, 404);
    }

    #[test]
    fn start_run_and_get_status() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (s, body) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        assert_eq!(s, 201);
        assert!(body.contains("run_id"));

        let run_id: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = run_id["run_id"].as_str().unwrap();

        let (s, body) = rj("GET", &format!("/runs/{}", id), &[], &kv);
        assert_eq!(s, 200);
        assert!(body.contains("running"));
    }

    #[test]
    fn start_run_missing_workflow() {
        let kv = MemKv::new();
        let (s, _) = rj("POST", "/workflows/ghost/run", &[], &kv);
        assert_eq!(s, 404);
    }

    #[test]
    fn ready_steps_all_pending_no_deps() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        let run_id: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = run_id["run_id"].as_str().unwrap();

        let (s, body) = rj("GET", &format!("/runs/{}/ready-steps", id), &[], &kv);
        assert_eq!(s, 200);
        assert!(body.contains("run"));
    }

    #[test]
    fn step_done_completes_run() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        let run_id: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = run_id["run_id"].as_str().unwrap();

        let (s, _) = rj(
            "POST",
            &format!("/runs/{}/steps/run/done", id),
            br#"{"output":[]}"#,
            &kv,
        );
        assert_eq!(s, 204);

        let (_, status_body) = rj("GET", &format!("/runs/{}", id), &[], &kv);
        assert!(status_body.contains("succeeded"));
    }

    #[test]
    fn step_done_chain_completes_run() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &order_wf_body(), &kv);
        let (_, body) = rj("POST", "/workflows/order-pipeline/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = v["run_id"].as_str().unwrap();

        for step in &["validate", "charge", "fulfill", "notify"] {
            let (s, _) = rj(
                "POST",
                &format!("/runs/{}/steps/{}/done", id, step),
                &[],
                &kv,
            );
            assert_eq!(s, 204, "step {} done failed", step);
        }

        let (_, status_body) = rj("GET", &format!("/runs/{}", id), &[], &kv);
        assert!(status_body.contains("succeeded"));
    }

    #[test]
    fn cancel_run() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = v["run_id"].as_str().unwrap();

        let (s, _) = rj("POST", &format!("/runs/{}/cancel", id), &[], &kv);
        assert_eq!(s, 204);

        let (_, status_body) = rj("GET", &format!("/runs/{}", id), &[], &kv);
        assert!(status_body.contains("cancelled"));
    }

    #[test]
    fn idem_key_deduplication() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);

        let body = br#"{"idem_key":"my-key"}"#.to_vec();
        let (_, b1) = rj("POST", "/workflows/simple-job/run", &body, &kv);
        let (_, b2) = rj("POST", "/workflows/simple-job/run", &body, &kv);

        let v1: serde_json::Value = serde_json::from_str(&b1).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&b2).unwrap();
        assert_eq!(v1["run_id"], v2["run_id"]);
    }

    #[test]
    fn step_failed_schedules_retry() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = v["run_id"].as_str().unwrap();

        let (s, _) = rj(
            "POST",
            &format!("/runs/{}/steps/run/failed", id),
            br#"{"error":"oops"}"#,
            &kv,
        );
        assert_eq!(s, 204);

        let (_, status_body) = rj("GET", &format!("/runs/{}", id), &[], &kv);
        assert!(status_body.contains("failed"));
    }

    // ---- event tests ----

    #[test]
    fn event_subscribe_emit_unsubscribe() {
        let kv = MemKv::new();

        let (s, _) = rj(
            "POST",
            "/events/order.placed/subscribe",
            br#"{"fn_name":"handle-order"}"#,
            &kv,
        );
        assert_eq!(s, 204);

        let (s, body) = rj("POST", "/events/order.placed/emit", br#"{"payload":[]}"#, &kv);
        assert_eq!(s, 200);
        assert!(body.contains("handle-order"));

        let (s, _) = rj(
            "DELETE",
            "/events/order.placed/subscribe/handle-order",
            &[],
            &kv,
        );
        assert_eq!(s, 204);

        let (_, body) = rj("POST", "/events/order.placed/emit", &[], &kv);
        assert!(!body.contains("handle-order"));
    }

    #[test]
    fn list_events() {
        let kv = MemKv::new();
        rj("POST", "/events/order.placed/subscribe", br#"{"fn_name":"f"}"#, &kv);
        rj("POST", "/events/order.shipped/subscribe", br#"{"fn_name":"g"}"#, &kv);
        let (s, body) = rj("GET", "/events", &[], &kv);
        assert_eq!(s, 200);
        assert!(body.contains("order.placed") || body.contains("order.shipped"));
    }

    #[test]
    fn register_bad_json_returns_400() {
        let kv = MemKv::new();
        let (s, _) = rj("POST", "/workflows", b"not-json", &kv);
        assert_eq!(s, 400);
    }

    #[test]
    fn not_found_returns_404() {
        let kv = MemKv::new();
        let (s, _) = rj("GET", "/unknown", &[], &kv);
        assert_eq!(s, 404);
    }

    // ---- YAML body tests ----

    #[test]
    fn register_workflow_yaml() {
        let kv = MemKv::new();
        let yaml = b"name: yaml-job\nsteps:\n  - name: run\n    depends_on: []\n";
        let (s, body) = ry("POST", "/workflows", yaml, &kv);
        assert_eq!(s, 201, "body: {}", body);
        assert!(body.contains("yaml-job"));

        let (s, body) = rj("GET", "/workflows/yaml-job", &[], &kv);
        assert_eq!(s, 200);
        assert!(body.contains("yaml-job"));
    }

    #[test]
    fn register_workflow_yaml_with_metadata() {
        let kv = MemKv::new();
        let yaml = b"name: order-pipeline-yaml\ndescription: Process orders\ntimeout_ms: 60000\ntriggers:\n  - event: order.placed\nsteps:\n  - name: validate\n    depends_on: []\n    max_attempts: 3\n    base_delay_ms: 500\n  - name: charge\n    depends_on:\n      - validate\n    max_attempts: 5\n    base_delay_ms: 1000\n";
        let (s, body) = ry("POST", "/workflows", yaml, &kv);
        assert_eq!(s, 201, "body: {}", body);
        assert!(body.contains("order-pipeline-yaml"));
    }

    #[test]
    fn register_workflow_bad_yaml_returns_400() {
        let kv = MemKv::new();
        // Valid YAML syntax but missing required `name` field
        let yaml = b"steps:\n  - name: run\n    depends_on: []\n";
        let (s, _) = ry("POST", "/workflows", yaml, &kv);
        assert_eq!(s, 400);
    }

    #[test]
    fn register_workflow_invalid_yaml_syntax_returns_400() {
        let kv = MemKv::new();
        let yaml = b"name: {\n  unclosed brace\n";
        let (s, _) = ry("POST", "/workflows", yaml, &kv);
        assert_eq!(s, 400);
    }

    #[test]
    fn event_subscribe_yaml() {
        let kv = MemKv::new();
        let yaml = b"fn_name: my-handler\n";
        let (s, _) = ry("POST", "/events/order.placed/subscribe", yaml, &kv);
        assert_eq!(s, 204);

        let (_, body) = rj("POST", "/events/order.placed/emit", &[], &kv);
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
        let kv = MemKv::new();
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (_, body) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let id = v["run_id"].as_str().unwrap();

        // Mark step done with some output
        rj(
            "POST",
            &format!("/runs/{}/steps/run/done", id),
            br#"{"output":[1,2,3]}"#,
            &kv,
        );

        let (s, body) = rj("GET", &format!("/runs/{}/steps/run/output", id), &[], &kv);
        assert_eq!(s, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["state"], "succeeded");
        assert!(!v["output"].is_null());
    }

    #[test]
    fn get_step_output_missing_returns_404() {
        let kv = MemKv::new();
        let (s, _) = rj("GET", "/runs/nonexistent-run/steps/nope/output", &[], &kv);
        assert_eq!(s, 404);
    }

    // ---- Sub-workflow ----

    #[test]
    fn sub_workflow_field_accepted_in_definition() {
        let kv = MemKv::new();
        let body = br#"{"name":"parent-wf","steps":[{"name":"delegate","depends_on":[],"sub_workflow":"child-wf"}]}"#;
        let (s, _) = rj("POST", "/workflows", body, &kv);
        assert_eq!(s, 201);

        let (s, body) = rj("GET", "/workflows/parent-wf", &[], &kv);
        assert_eq!(s, 200);
        assert!(body.contains("child-wf"));
    }

    #[test]
    fn sub_workflow_step_in_ready_steps_has_kind_field() {
        let kv = MemKv::new();
        let body = br#"{"name":"parent-wf","steps":[{"name":"delegate","depends_on":[],"sub_workflow":"child-wf"}]}"#;
        rj("POST", "/workflows", body, &kv);
        let (_, rb) = rj("POST", "/workflows/parent-wf/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&rb).unwrap();
        let id = v["run_id"].as_str().unwrap();

        let (s, body) = rj("GET", &format!("/runs/{}/ready-steps", id), &[], &kv);
        assert_eq!(s, 200);
        assert!(body.contains("sub_workflow"), "expected sub_workflow field in: {}", body);
        assert!(body.contains("child-wf"));
    }

    #[test]
    fn sub_workflow_link_and_auto_complete_when_child_succeeds() {
        let kv = MemKv::new();
        // Register child workflow and start a run of it
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (_, cb) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        let cv: serde_json::Value = serde_json::from_str(&cb).unwrap();
        let child_id = cv["run_id"].as_str().unwrap();
        // Complete child
        rj("POST", &format!("/runs/{}/steps/run/done", child_id), &[], &kv);

        // Register parent workflow that delegates to simple-job
        let pbody = br#"{"name":"parent-wf","steps":[{"name":"delegate","depends_on":[],"sub_workflow":"simple-job"}]}"#;
        rj("POST", "/workflows", pbody, &kv);
        let (_, pb) = rj("POST", "/workflows/parent-wf/run", &[], &kv);
        let pv: serde_json::Value = serde_json::from_str(&pb).unwrap();
        let parent_id = pv["run_id"].as_str().unwrap();

        // Link child run to parent step
        let link_body = format!(r#"{{"sub_run_id":"{}"}}"#, child_id);
        let (s, _) = rj(
            "POST",
            &format!("/runs/{}/steps/delegate/sub-run", parent_id),
            link_body.as_bytes(),
            &kv,
        );
        assert_eq!(s, 204);

        // Parent run should now be succeeded (child was already succeeded)
        let (_, status) = rj("GET", &format!("/runs/{}", parent_id), &[], &kv);
        assert!(status.contains("succeeded"), "parent run status: {}", status);
    }

    #[test]
    fn sub_workflow_fail_when_child_fails() {
        let kv = MemKv::new();
        // Register child workflow and start a run
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (_, cb) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        let cv: serde_json::Value = serde_json::from_str(&cb).unwrap();
        let child_id = cv["run_id"].as_str().unwrap();
        // Fail child (max_attempts=1, so first failure => failed state)
        rj("POST", &format!("/runs/{}/steps/run/failed", child_id), br#"{"error":"boom"}"#, &kv);

        // Register and start parent
        let pbody = br#"{"name":"parent-wf","steps":[{"name":"delegate","depends_on":[],"sub_workflow":"simple-job"}]}"#;
        rj("POST", "/workflows", pbody, &kv);
        let (_, pb) = rj("POST", "/workflows/parent-wf/run", &[], &kv);
        let pv: serde_json::Value = serde_json::from_str(&pb).unwrap();
        let parent_id = pv["run_id"].as_str().unwrap();

        let link_body = format!(r#"{{"sub_run_id":"{}"}}"#, child_id);
        rj(
            "POST",
            &format!("/runs/{}/steps/delegate/sub-run", parent_id),
            link_body.as_bytes(),
            &kv,
        );

        let (_, status) = rj("GET", &format!("/runs/{}", parent_id), &[], &kv);
        assert!(status.contains("failed"), "parent run status: {}", status);
    }

    #[test]
    fn nested_sub_workflow_three_levels() {
        let kv = MemKv::new();

        // Level 3 (grandchild) — plain workflow
        rj("POST", "/workflows", &simple_wf_body(), &kv);
        let (_, b) = rj("POST", "/workflows/simple-job/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let grandchild_id = v["run_id"].as_str().unwrap().to_string();
        rj("POST", &format!("/runs/{}/steps/run/done", grandchild_id), &[], &kv);

        // Level 2 (child) — delegates to simple-job
        let cb = br#"{"name":"child-wf","steps":[{"name":"sub","depends_on":[],"sub_workflow":"simple-job"}]}"#;
        rj("POST", "/workflows", cb, &kv);
        let (_, b2) = rj("POST", "/workflows/child-wf/run", &[], &kv);
        let v2: serde_json::Value = serde_json::from_str(&b2).unwrap();
        let child_id = v2["run_id"].as_str().unwrap().to_string();
        // Link grandchild to child step
        let lb = format!(r#"{{"sub_run_id":"{}"}}"#, grandchild_id);
        rj("POST", &format!("/runs/{}/steps/sub/sub-run", child_id), lb.as_bytes(), &kv);
        // child should now be succeeded
        let (_, cs) = rj("GET", &format!("/runs/{}", child_id), &[], &kv);
        assert!(cs.contains("succeeded"), "child status: {}", cs);

        // Level 1 (parent) — delegates to child-wf
        let pb = br#"{"name":"parent-wf","steps":[{"name":"sub","depends_on":[],"sub_workflow":"child-wf"}]}"#;
        rj("POST", "/workflows", pb, &kv);
        let (_, b3) = rj("POST", "/workflows/parent-wf/run", &[], &kv);
        let v3: serde_json::Value = serde_json::from_str(&b3).unwrap();
        let parent_id = v3["run_id"].as_str().unwrap().to_string();
        let lb2 = format!(r#"{{"sub_run_id":"{}"}}"#, child_id);
        rj("POST", &format!("/runs/{}/steps/sub/sub-run", parent_id), lb2.as_bytes(), &kv);

        let (_, ps) = rj("GET", &format!("/runs/{}", parent_id), &[], &kv);
        assert!(ps.contains("succeeded"), "parent status: {}", ps);
    }

    // ---- If-else branching ----

    fn if_else_wf_body() -> Vec<u8> {
        // check step → yes-branch (condition: output=="yes") + no-branch (condition: output=="no")
        // Both branches are optional so we can test individual scenarios.
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
        let kv = MemKv::new();
        rj("POST", "/workflows", &if_else_wf_body(), &kv);
        let (_, b) = rj("POST", "/workflows/if-else-wf/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let id = v["run_id"].as_str().unwrap();

        // Complete "check" with output "yes"
        rj("POST", &format!("/runs/{}/steps/check/done", id),
           br#"{"output":[34,121,101,115,34]}"#, &kv); // JSON bytes for "yes"

        // Trigger ready-steps evaluation (which skips false branches)
        let (_, ready) = rj("GET", &format!("/runs/{}/ready-steps", id), &[], &kv);
        // yes-branch should appear in ready steps
        assert!(ready.contains("yes-branch"), "ready: {}", ready);
        // no-branch should be skipped
        let sk = kv.0.borrow();
        let nb_key = format!("step:{}:no-branch", id);
        let nb_state: StepRecord = serde_json::from_slice(sk.get(&nb_key).unwrap()).unwrap();
        assert_eq!(nb_state.state, "skipped");
    }

    #[test]
    fn if_else_false_branch_optional_run_still_succeeds() {
        let kv = MemKv::new();
        rj("POST", "/workflows", &if_else_wf_body(), &kv);
        let (_, b) = rj("POST", "/workflows/if-else-wf/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let id = v["run_id"].as_str().unwrap();

        // Complete "check" with output "yes"
        rj("POST", &format!("/runs/{}/steps/check/done", id),
           br#"{"output":[34,121,101,115,34]}"#, &kv);

        // Trigger evaluation
        rj("GET", &format!("/runs/{}/ready-steps", id), &[], &kv);

        // Complete yes-branch
        rj("POST", &format!("/runs/{}/steps/yes-branch/done", id), &[], &kv);

        let (_, status) = rj("GET", &format!("/runs/{}", id), &[], &kv);
        assert!(status.contains("succeeded"), "run status: {}", status);
    }

    #[test]
    fn if_else_false_branch_required_run_fails() {
        let kv = MemKv::new();
        // Workflow with non-optional false branch
        let wf_body = br#"{
            "name": "strict-wf",
            "steps": [
                {"name": "check", "depends_on": []},
                {"name": "required-branch", "depends_on": ["check"], "optional": false,
                 "condition": {"on_step": "check", "equals": "yes"}}
            ]
        }"#;
        rj("POST", "/workflows", wf_body, &kv);
        let (_, b) = rj("POST", "/workflows/strict-wf/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let id = v["run_id"].as_str().unwrap();

        // Complete "check" with output "no" — condition will be false, step skipped
        rj("POST", &format!("/runs/{}/steps/check/done", id),
           br#"{"output":[34,110,111,34]}"#, &kv); // JSON bytes for "no"

        // Trigger evaluation
        rj("GET", &format!("/runs/{}/ready-steps", id), &[], &kv);

        let (_, status) = rj("GET", &format!("/runs/{}", id), &[], &kv);
        assert!(status.contains("failed"), "run status: {}", status);
    }

    #[test]
    fn transitive_skip_downstream_optional() {
        let kv = MemKv::new();
        // check → middle (optional, condition) → end (optional, no condition)
        // When check output doesn't match, middle is skipped → end is transitively skipped
        // Both optional, so run succeeds.
        let wf_body = br#"{
            "name": "transitive-wf",
            "steps": [
                {"name": "check", "depends_on": []},
                {"name": "middle", "depends_on": ["check"], "optional": true,
                 "condition": {"on_step": "check", "equals": "go"}},
                {"name": "end", "depends_on": ["middle"], "optional": true}
            ]
        }"#;
        rj("POST", "/workflows", wf_body, &kv);
        let (_, b) = rj("POST", "/workflows/transitive-wf/run", &[], &kv);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        let id = v["run_id"].as_str().unwrap();

        // Complete check with "stop" — condition "go" won't match
        rj("POST", &format!("/runs/{}/steps/check/done", id),
           br#"{"output":[34,115,116,111,112,34]}"#, &kv); // "stop"

        // Evaluate: middle skipped, end transitively skipped
        rj("GET", &format!("/runs/{}/ready-steps", id), &[], &kv);

        let sk = kv.0.borrow();
        let end_key = format!("step:{}:end", id);
        let end_sr: StepRecord = serde_json::from_slice(sk.get(&end_key).unwrap()).unwrap();
        assert_eq!(end_sr.state, "skipped");
        drop(sk);

        // Run should succeed because all skipped steps are optional
        let (_, status) = rj("GET", &format!("/runs/{}", id), &[], &kv);
        assert!(status.contains("succeeded"), "run status: {}", status);
    }
}
