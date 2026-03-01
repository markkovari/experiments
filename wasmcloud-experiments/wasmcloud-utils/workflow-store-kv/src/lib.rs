// workflow-store-kv: wasi:keyvalue adapter for wasmcloud:workflow-store/store.
// Works with any wasi:keyvalue provider: NATS KV, Redis, Vault, etc.
//
// Exports:  wasmcloud:workflow-store/store
// Imports:  wasi:keyvalue/store
//
// KV schema (bucket: "workflow"):
//   wf-def.<name>             → JSON bytes
//   wf-run.<run-id>           → JSON bytes
//   step.<run-id>.<step-name> → JSON bytes
//   evt.<event-name>          → JSON list<string> (subscriber fn-names)
//   sub-run.<parent-id>.<step>→ child run-id bytes

wit_bindgen::generate!({
    world: "workflow-store-kv-component",
    path: "wit",
    generate_all,
});

use exports::wasmcloud::workflow_store::store::{
    Guest, StoreError,
};

struct WorkflowStoreNats;

fn open_bucket() -> Result<wasi::keyvalue::store::Bucket, StoreError> {
    wasi::keyvalue::store::open("default")
        .map_err(|e| StoreError::IoError(format!("open bucket: {:?}", e)))
}

impl Guest for WorkflowStoreNats {
    fn put_workflow_def(name: String, json: Vec<u8>) -> Result<(), StoreError> {
        let bucket = open_bucket()?;
        let key = format!("wf-def.{}", name);
        bucket
            .set(&key, &json)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn get_workflow_def(name: String) -> Result<Option<Vec<u8>>, StoreError> {
        let bucket = open_bucket()?;
        let key = format!("wf-def.{}", name);
        bucket
            .get(&key)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn delete_workflow_def(name: String) -> Result<(), StoreError> {
        let bucket = open_bucket()?;
        let key = format!("wf-def.{}", name);
        bucket
            .delete(&key)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn list_workflow_names(page: u32, limit: u32) -> Result<Vec<String>, StoreError> {
        let bucket = open_bucket()?;
        let keys = bucket
            .list_keys(None)
            .map(|r| r.keys)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))?;
        let mut names: Vec<String> = keys
            .iter()
            .filter_map(|k| k.strip_prefix("wf-def.").map(|s| s.to_string()))
            .collect();
        names.sort();
        let limit = if limit == 0 { 50 } else { limit as usize };
        let start = ((page.max(1) - 1) as usize) * limit;
        Ok(names.into_iter().skip(start).take(limit).collect())
    }

    fn put_run(run_id: String, json: Vec<u8>) -> Result<(), StoreError> {
        let bucket = open_bucket()?;
        let key = format!("wf-run.{}", run_id);
        bucket
            .set(&key, &json)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn get_run(run_id: String) -> Result<Option<Vec<u8>>, StoreError> {
        let bucket = open_bucket()?;
        let key = format!("wf-run.{}", run_id);
        bucket
            .get(&key)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn list_runs(
        wf_name: String,
        state_filter: Option<String>,
        page: u32,
        limit: u32,
    ) -> Result<Vec<Vec<u8>>, StoreError> {
        let bucket = open_bucket()?;
        let keys = bucket
            .list_keys(None)
            .map(|r| r.keys)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))?;

        #[derive(serde::Deserialize)]
        struct RunRecord {
            wf_name: String,
            state: String,
            created_at_ms: u64,
        }

        let mut runs: Vec<(u64, Vec<u8>)> = keys
            .iter()
            .filter(|k| k.starts_with("wf-run."))
            .filter_map(|k| {
                let v = bucket.get(k).ok().flatten()?;
                let r: RunRecord = serde_json::from_slice(&v).ok()?;
                if r.wf_name != wf_name {
                    return None;
                }
                if let Some(ref sf) = state_filter {
                    if &r.state != sf {
                        return None;
                    }
                }
                Some((r.created_at_ms, v))
            })
            .collect();

        // Sort descending by created_at_ms
        runs.sort_by(|a, b| b.0.cmp(&a.0));

        let limit = if limit == 0 { 50 } else { limit as usize };
        let start = ((page.max(1) - 1) as usize) * limit;
        Ok(runs
            .into_iter()
            .skip(start)
            .take(limit)
            .map(|(_, v)| v)
            .collect())
    }

    fn put_step(run_id: String, step_name: String, json: Vec<u8>) -> Result<(), StoreError> {
        let bucket = open_bucket()?;
        let key = format!("step.{}.{}", run_id, step_name);
        bucket
            .set(&key, &json)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn get_step(run_id: String, step_name: String) -> Result<Option<Vec<u8>>, StoreError> {
        let bucket = open_bucket()?;
        let key = format!("step.{}.{}", run_id, step_name);
        bucket
            .get(&key)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn list_step_names(run_id: String) -> Result<Vec<String>, StoreError> {
        let bucket = open_bucket()?;
        let prefix = format!("step.{}.", run_id);
        let keys = bucket
            .list_keys(None)
            .map(|r| r.keys)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))?;
        let mut names: Vec<String> = keys
            .iter()
            .filter_map(|k| k.strip_prefix(&prefix).map(|s| s.to_string()))
            .collect();
        names.sort();
        Ok(names)
    }

    fn put_event_subs(event_name: String, subs: Vec<String>) -> Result<(), StoreError> {
        let bucket = open_bucket()?;
        let key = format!("evt.{}", event_name);
        let json = serde_json::to_vec(&subs)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))?;
        bucket
            .set(&key, &json)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn get_event_subs(event_name: String) -> Result<Vec<String>, StoreError> {
        let bucket = open_bucket()?;
        let key = format!("evt.{}", event_name);
        let v = bucket
            .get(&key)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))?;
        match v {
            None => Ok(vec![]),
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| StoreError::IoError(format!("{:?}", e))),
        }
    }

    fn list_event_names() -> Result<Vec<String>, StoreError> {
        let bucket = open_bucket()?;
        let keys = bucket
            .list_keys(None)
            .map(|r| r.keys)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))?;
        let mut names: Vec<String> = keys
            .iter()
            .filter_map(|k| k.strip_prefix("evt.").map(|s| s.to_string()))
            .collect();
        names.sort();
        Ok(names)
    }

    fn put_sub_run_link(
        parent_run_id: String,
        step_name: String,
        child_run_id: String,
    ) -> Result<(), StoreError> {
        let bucket = open_bucket()?;
        let key = format!("sub-run.{}.{}", parent_run_id, step_name);
        bucket
            .set(&key, child_run_id.as_bytes())
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))
    }

    fn get_sub_run_link(
        parent_run_id: String,
        step_name: String,
    ) -> Result<Option<String>, StoreError> {
        let bucket = open_bucket()?;
        let key = format!("sub-run.{}.{}", parent_run_id, step_name);
        let v = bucket
            .get(&key)
            .map_err(|e| StoreError::IoError(format!("{:?}", e)))?;
        match v {
            None => Ok(None),
            Some(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|e| StoreError::IoError(format!("{:?}", e))),
        }
    }
}

export!(WorkflowStoreNats);
