# Workflow Engine — Pluggable Storage Guide

The workflow engine uses a **pluggable storage layer** built on the
`wasmcloud:workflow-store/store` WIT interface.  The engine component
(`workflow-api`) never speaks to a database directly.  Instead, it calls a
separately deployed **store component** that is linked at runtime.  Swapping
the database is a one-line change in the WADM manifest — no recompilation of
the engine is needed.

---

## Architecture

```
HTTP client
    │
    ▼  wasi:http/incoming-handler
workflow-api.wasm          (engine — owns all business logic)
    │
    ▼  wasmcloud:workflow-store/store   ← WIT import
workflow-store-kv.wasm   (default, backed by NATS JetStream KV)
workflow-store-postgres.wasm           (alternative — you write this)
workflow-store-redis.wasm              (alternative — you write this)
    │
    ▼  wasi:keyvalue/store  |  wasi:sql  |  custom WIT
NATS KV  /  PostgreSQL  /  Redis  / …
```

Each store component is a standalone `.wasm` file.  It **exports**
`wasmcloud:workflow-store/store` and **imports** whatever low-level
capability its backing database requires.  The engine and the store
communicate only through the typed WIT interface; neither side cares how
the other is implemented.

---

## The `wasmcloud:workflow-store` WIT Interface

Located at `wit/wasmcloud-workflow-store/workflow-store.wit`:

```wit
package wasmcloud:workflow-store@0.1.0;

interface store {
    variant store-error {
        not-found(string),
        conflict(string),
        io-error(string),
    }

    // Workflow definitions
    put-workflow-def:    func(name: string, json: list<u8>) -> result<_, store-error>;
    get-workflow-def:    func(name: string)                 -> result<option<list<u8>>, store-error>;
    delete-workflow-def: func(name: string)                 -> result<_, store-error>;
    list-workflow-names: func(page: u32, limit: u32)        -> result<list<string>, store-error>;

    // Runs
    put-run:   func(run-id: string, json: list<u8>)   -> result<_, store-error>;
    get-run:   func(run-id: string)                   -> result<option<list<u8>>, store-error>;
    list-runs: func(wf-name: string, state-filter: option<string>, page: u32, limit: u32)
                                                       -> result<list<list<u8>>, store-error>;

    // Steps
    put-step:        func(run-id: string, step-name: string, json: list<u8>) -> result<_, store-error>;
    get-step:        func(run-id: string, step-name: string)                  -> result<option<list<u8>>, store-error>;
    list-step-names: func(run-id: string)                                     -> result<list<string>, store-error>;

    // Event subscriptions
    put-event-subs:   func(event-name: string, subs: list<string>) -> result<_, store-error>;
    get-event-subs:   func(event-name: string)                     -> result<list<string>, store-error>;
    list-event-names: func()                                       -> result<list<string>, store-error>;

    // Sub-run links
    put-sub-run-link: func(parent-run-id: string, step-name: string, child-run-id: string) -> result<_, store-error>;
    get-sub-run-link: func(parent-run-id: string, step-name: string)                        -> result<option<string>, store-error>;
}
```

**Design principles:**

| Decision | Rationale |
|----------|-----------|
| JSON bytes (`list<u8>`) for payloads | Serialization stays in the engine; the store is schema-agnostic. Adding a field to `WorkflowDef` does not require changing the store interface. |
| `list-runs` accepts `state-filter` + pagination | A SQL backend can push `WHERE state = ? LIMIT ? OFFSET ?` to the database instead of loading all rows. |
| `list-workflow-names` returns names only | Avoids deserializing every definition just to enumerate them. |
| `store-error` variant | The engine turns each variant into an appropriate HTTP status code (404, 409, 503). |

---

## Default Store: `workflow-store-kv`

Source: `workflow-store-kv/src/lib.rs`

Exports `wasmcloud:workflow-store/store`, imports `wasi:keyvalue/store`
(fulfilled by the wasmCloud NATS KV provider).

### Key schema (bucket: `workflow`)

| Key pattern | Value |
|-------------|-------|
| `wf-def.<name>` | JSON `WorkflowDef` |
| `wf-run.<run-id>` | JSON `RunRecord` |
| `step.<run-id>.<step-name>` | JSON `StepRecord` |
| `evt.<event-name>` | JSON `list<string>` (subscriber fn-names) |
| `sub-run.<parent-run-id>.<step-name>` | child run-id (raw bytes) |

---

## Deploying with the Default Store

```yaml
# wadm/workflow-api.yaml (excerpt)
spec:
  components:
    - name: workflow-api-component
      type: component
      properties:
        image: file://../target/wasm32-wasip2/release/workflow_api.wasm
      traits:
        - type: link
          properties:
            target: workflow-store-kv       # ← store component
            namespace: wasmcloud
            package: workflow-store
            interfaces: [store]

    - name: workflow-store-kv
      type: component
      properties:
        image: file://../target/wasm32-wasip2/release/workflow_store_kv.wasm
      traits:
        - type: link
          properties:
            target: nats-kv
            namespace: wasi
            package: keyvalue
            interfaces: [store]
            target_config:
              - name: bucket
                properties:
                  bucket: workflow

    - name: nats-kv
      type: capability
      properties:
        image: ghcr.io/wasmcloud/keyvalue-nats:0.3.1
```

Build and deploy:

```bash
cargo component build -p workflow-api --release
cargo component build -p workflow-store-kv --release
wash app deploy wadm/workflow-api.yaml
```

---

## Writing a Custom Store Backend

### Step 1 — Create a new Rust crate

```bash
mkdir -p workflow-store-postgres/src
mkdir -p workflow-store-postgres/wit/deps
```

Copy or symlink the shared WIT dependencies:

```bash
# The typed domain interface (same for all stores)
ln -s ../../wit/wasmcloud-workflow-store \
      workflow-store-postgres/wit/deps/wasmcloud-workflow-store

# Whatever low-level interface your DB provider exposes, e.g.:
ln -s ../../wit/wasmcloud-workflow-api/deps/wasi-keyvalue-0.2.0-draft \
      workflow-store-postgres/wit/deps/wasi-keyvalue-0.2.0-draft
# or your own SQL WIT package
```

### Step 2 — Write the WIT world

`workflow-store-postgres/wit/workflow-store-postgres.wit`:

```wit
package wasmcloud:workflow-store-postgres@0.1.0;

world workflow-store-postgres-component {
    export wasmcloud:workflow-store/store@0.1.0;  // must export this
    import wasmcloud:postgres/query@0.1.0;        // your DB import
}
```

### Step 3 — Write `Cargo.toml`

```toml
[package]
name = "workflow-store-postgres"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.39.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[package.metadata.component]
package = "wasmcloud:workflow-store-postgres"

[package.metadata.component.target]
path = "wit"
world = "workflow-store-postgres-component"

[package.metadata.component.target.dependencies]
"wasmcloud:workflow-store" = { path = "wit/deps/wasmcloud-workflow-store" }
"wasmcloud:postgres"       = { path = "wit/deps/wasmcloud-postgres" }
```

### Step 4 — Implement `src/lib.rs`

```rust
wit_bindgen::generate!({
    world: "workflow-store-postgres-component",
    path: "wit",
    generate_all,
});

use exports::wasmcloud::workflow_store::store::{Guest, StoreError};

struct WorkflowStorePostgres;

impl Guest for WorkflowStorePostgres {
    fn put_workflow_def(name: String, json: Vec<u8>) -> Result<(), StoreError> {
        wasmcloud::postgres::query::execute(
            "INSERT INTO workflow_defs (name, data) VALUES ($1, $2) \
             ON CONFLICT (name) DO UPDATE SET data = $2",
            &[&name, &json],
        )
        .map_err(|e| StoreError::IoError(e.to_string()))
    }

    fn get_workflow_def(name: String) -> Result<Option<Vec<u8>>, StoreError> {
        let rows = wasmcloud::postgres::query::query(
            "SELECT data FROM workflow_defs WHERE name = $1",
            &[&name],
        )
        .map_err(|e| StoreError::IoError(e.to_string()))?;
        Ok(rows.into_iter().next().map(|r| r.get(0)))
    }

    fn delete_workflow_def(name: String) -> Result<(), StoreError> {
        wasmcloud::postgres::query::execute(
            "DELETE FROM workflow_defs WHERE name = $1",
            &[&name],
        )
        .map_err(|e| StoreError::IoError(e.to_string()))
    }

    fn list_workflow_names(page: u32, limit: u32) -> Result<Vec<String>, StoreError> {
        let offset = (page.saturating_sub(1) as i64) * (limit as i64);
        let rows = wasmcloud::postgres::query::query(
            "SELECT name FROM workflow_defs ORDER BY name LIMIT $1 OFFSET $2",
            &[&(limit as i64), &offset],
        )
        .map_err(|e| StoreError::IoError(e.to_string()))?;
        Ok(rows.into_iter().map(|r| r.get(0)).collect())
    }

    fn put_run(run_id: String, json: Vec<u8>) -> Result<(), StoreError> { todo!() }
    fn get_run(run_id: String) -> Result<Option<Vec<u8>>, StoreError> { todo!() }
    fn list_runs(
        wf_name: String,
        state_filter: Option<String>,
        page: u32,
        limit: u32,
    ) -> Result<Vec<Vec<u8>>, StoreError> { todo!() }
    fn put_step(run_id: String, step_name: String, json: Vec<u8>) -> Result<(), StoreError> { todo!() }
    fn get_step(run_id: String, step_name: String) -> Result<Option<Vec<u8>>, StoreError> { todo!() }
    fn list_step_names(run_id: String) -> Result<Vec<String>, StoreError> { todo!() }
    fn put_event_subs(event_name: String, subs: Vec<String>) -> Result<(), StoreError> { todo!() }
    fn get_event_subs(event_name: String) -> Result<Vec<String>, StoreError> { todo!() }
    fn list_event_names() -> Result<Vec<String>, StoreError> { todo!() }
    fn put_sub_run_link(parent_run_id: String, step_name: String, child_run_id: String) -> Result<(), StoreError> { todo!() }
    fn get_sub_run_link(parent_run_id: String, step_name: String) -> Result<Option<String>, StoreError> { todo!() }
}

export!(WorkflowStorePostgres);
```

> **Tip — `list-runs` with a SQL backend:**
>
> ```sql
> SELECT data FROM workflow_runs
> WHERE wf_name = $1
>   AND ($2::text IS NULL OR state = $2)
> ORDER BY created_at DESC
> LIMIT $3 OFFSET $4
> ```
>
> Pass `state_filter` as a nullable parameter.  The database evaluates the
> `WHERE` clause; no in-process filtering is needed.

### Step 5 — Schema (PostgreSQL example)

```sql
CREATE TABLE workflow_defs (
    name TEXT PRIMARY KEY,
    data BYTEA NOT NULL
);

CREATE TABLE workflow_runs (
    run_id     TEXT PRIMARY KEY,
    wf_name    TEXT NOT NULL,
    state      TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    data       BYTEA NOT NULL
);
CREATE INDEX ON workflow_runs (wf_name, state, created_at DESC);

CREATE TABLE workflow_steps (
    run_id    TEXT NOT NULL,
    step_name TEXT NOT NULL,
    data      BYTEA NOT NULL,
    PRIMARY KEY (run_id, step_name)
);

CREATE TABLE workflow_event_subs (
    event_name TEXT PRIMARY KEY,
    subs       JSONB NOT NULL DEFAULT '[]'
);

CREATE TABLE workflow_sub_run_links (
    parent_run_id TEXT NOT NULL,
    step_name     TEXT NOT NULL,
    child_run_id  TEXT NOT NULL,
    PRIMARY KEY (parent_run_id, step_name)
);
```

### Step 6 — Build

```bash
cargo component build -p workflow-store-postgres --release
```

### Step 7 — Wire up in WADM

```yaml
spec:
  components:
    - name: workflow-api-component
      type: component
      properties:
        image: file://../target/wasm32-wasip2/release/workflow_api.wasm
      traits:
        - type: link
          properties:
            target: workflow-store-postgres   # ← swap here
            namespace: wasmcloud
            package: workflow-store
            interfaces: [store]

    - name: workflow-store-postgres
      type: component
      properties:
        image: file://../target/wasm32-wasip2/release/workflow_store_postgres.wasm
      traits:
        - type: link
          properties:
            target: postgres-provider
            namespace: wasmcloud
            package: postgres
            interfaces: [query]
            target_config:
              - name: pg-conn
                properties:
                  url: "postgres://user:pass@host:5432/workflow_db"

    - name: postgres-provider
      type: capability
      properties:
        image: ghcr.io/wasmcloud/postgres:0.1.0   # hypothetical provider image
```

Deploy:

```bash
wash app deploy wadm/workflow-api-postgres.yaml
```

The engine (`workflow-api.wasm`) is **not rebuilt**.

---

## Choosing the Right Backend

| Backend | Best for | Notes |
|---------|----------|-------|
| **NATS KV** (`workflow-store-kv`) | Development, edge, lightweight prod | Zero extra infra if you already run NATS. No SQL queries. List operations scan all keys. |
| **PostgreSQL** | High-volume production | Full SQL: indexed queries, `WHERE state = ?`, efficient pagination. Requires a Postgres provider. |
| **Redis** | Low-latency, short-lived runs | Hash + sorted-set data model. TTL support for auto-expiring runs. |
| **In-memory** (test only) | Unit tests | Implemented as `MemStore` in `workflow-api/src/lib.rs`. Never deployed. |

---

## Testing Your Store

The engine unit tests use `MemStore` (an in-memory implementation of
`StoreBackend`), so they run without any external process.  To test your new
store component in isolation, write integration tests that start the store
component in a wasmtime runtime:

```rust
// tests/integration.rs (inside your store crate)
use wasmtime::component::*;
use wasmtime_wasi::*;

#[tokio::test]
async fn put_and_get_workflow_def() {
    // load your .wasm, link mock DB provider, call put_workflow_def / get_workflow_def
    // assert round-trip fidelity
}
```

For end-to-end BDD tests, deploy both components with `wash app deploy` and
run the existing Gherkin feature files:

```bash
cargo test -p workflow-api-cucumber
```

The cucumber suite probes `http://localhost:8080` and skips automatically if
the stack is not running.

---

## Summary: What You Need to Implement

To add a new storage backend, you must implement exactly **12 functions**:

| Function | Description |
|----------|-------------|
| `put-workflow-def` | Upsert a workflow definition (JSON bytes) |
| `get-workflow-def` | Fetch a workflow definition by name |
| `delete-workflow-def` | Remove a workflow definition |
| `list-workflow-names` | Paginated list of workflow names |
| `put-run` | Upsert a run record |
| `get-run` | Fetch a run record by run-id |
| `list-runs` | Paginated, optionally filtered list of run records for a workflow |
| `put-step` | Upsert a step record |
| `get-step` | Fetch a step record by (run-id, step-name) |
| `list-step-names` | All step names for a given run |
| `put-event-subs` | Store the subscriber list for an event |
| `get-event-subs` | Retrieve the subscriber list for an event |
| `list-event-names` | All event names that have subscribers |
| `put-sub-run-link` | Store a parent-step → child-run-id link |
| `get-sub-run-link` | Retrieve the child-run-id for a parent step |

You do **not** touch `workflow-api`, the WIT interface, the WADM manifests for
other components, or any Gherkin feature files.
