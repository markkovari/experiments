# Workflow API — Architecture

## Component Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                        wasmCloud Host                        │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │           workflow-api (WASM Component)               │  │
│  │                                                       │  │
│  │  exports: wasi:http/incoming-handler                  │  │
│  │  imports: wasmcloud:workflow-store/store              │  │
│  │                                                       │  │
│  │  ┌──────────┐    ┌──────────────────────┐            │  │
│  │  │  Router  │───▶│  Business Logic      │            │  │
│  │  │ (HTTP)   │    │  (pure Rust fns)     │            │  │
│  │  └──────────┘    └────────┬─────────────┘            │  │
│  └───────────────────────────│──────────────────────────┘  │
│                              │ wasmcloud:workflow-store      │
│  ┌───────────────────────────▼──────────────────────────┐  │
│  │         workflow-store-kv (WASM Component)         │  │
│  │                                                       │  │
│  │  exports: wasmcloud:workflow-store/store              │  │
│  │  imports: wasi:keyvalue/store                        │  │
│  └───────────────────────────│──────────────────────────┘  │
│                              │ wasi:keyvalue                 │
│  ┌───────────────────────────▼──────────────────────────┐  │
│  │              NATS JetStream KV                        │  │
│  │              bucket: "workflow"                       │  │
│  └───────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

HTTP requests arrive via the wasmCloud HTTP provider, are dispatched to the component's `wasi:http/incoming-handler` export, routed by path/method, and handled by pure Rust business-logic functions.  Persistence is delegated to a linked **store component** through the typed `wasmcloud:workflow-store/store` WIT interface.  The default store component (`workflow-store-kv`) translates those domain calls into flat `wasi:keyvalue/store` operations against NATS KV.  Any other store component that exports the same interface can be substituted in the WADM manifest — see [PLUGGABLE_STORAGE.md](./PLUGGABLE_STORAGE.md).

---

## Run State Machine

```
         ┌─────────┐
  start  │         │
 ───────▶│ running │
         │         │
         └────┬────┘
              │
     ┌────────┼──────────┐
     │        │          │
     ▼        ▼          ▼
┌─────────┐ ┌──────┐ ┌───────────┐
│succeeded│ │failed│ │ cancelled │
└─────────┘ └──────┘ └───────────┘
```

- `running` → `succeeded`: all non-optional, non-skipped steps succeed
- `running` → `failed`: any non-optional step reaches `failed` state
- `running` → `cancelled`: explicit POST to `/runs/{run_id}/cancel`

---

## Step State Machine

```
           ┌─────────┐
  create   │         │
 ─────────▶│ pending │◀──── retry
           │         │
           └────┬────┘
                │ execute
       ┌────────┼──────────┐
       │        │          │
       ▼        ▼          ▼
 ┌─────────┐ ┌──────┐ ┌─────────┐
 │succeeded│ │failed│ │ skipped │
 └─────────┘ └──────┘ └─────────┘
```

- Steps begin as `pending`.
- A step becomes **ready** when all its `depends_on` steps are `succeeded` (or `skipped`).
- `failed` steps can be reset to `pending` via the retry endpoint.
- `skipped` steps occur when a `condition` is not met or a depended-on step was skipped (transitive skip).

---

## Sub-Workflow DAG

A step with `sub_workflow: "child-wf"` delegates execution to a child workflow run:

```
parent run
  └── step "delegate" (sub_workflow: "child-wf")
        └── child run (wf_name: "child-wf")
              ├── step "leaf-a"
              └── step "leaf-b"
```

When the child run reaches `succeeded`, the parent step is automatically marked `succeeded`. When the child run reaches `failed`, the parent step is marked `failed`.

Sub-workflows can be nested to arbitrary depth. Each level creates an independent run record linked by the step's `sub_workflow` field.

---

## If-Else Branching

Conditional steps use the `condition` field:

```json
{
  "name": "on-approved",
  "depends_on": ["review"],
  "condition": {"on_step": "review", "equals": "approved"}
}
```

When the `review` step completes, the API reads its `output` bytes, parses them as JSON, and compares with `equals`. If the comparison fails, the step is set to `skipped`.

### Transitive Skip Algorithm

1. When a step is skipped, all steps that depend **exclusively** on skipped steps are also skipped.
2. Steps marked `optional: true` are excluded from the run-failure check.
3. A run succeeds when all non-optional, non-skipped steps are `succeeded`.

---

## Retry Backoff Formula

```
delay_ms = min(base_delay_ms × 2^attempt, 60_000)
```

| attempt | base_delay_ms=500 | base_delay_ms=1000 |
|---------|-------------------|--------------------|
| 0 | 500 ms | 1 000 ms |
| 1 | 1 000 ms | 2 000 ms |
| 2 | 2 000 ms | 4 000 ms |
| 3 | 4 000 ms | 8 000 ms |
| 7+ | 60 000 ms (cap) | 60 000 ms (cap) |

The `attempt` field in `StepRecord` is incremented by the retry endpoint before the backoff is applied.

---

## Storage Layer

The engine uses the `wasmcloud:workflow-store/store` WIT interface for all
persistence.  The default implementation (`workflow-store-kv`) maps domain
operations onto a NATS JetStream KV bucket with the following key schema:

```
workflow/  (bucket name)
  wf-def.<name>               WorkflowDef JSON
  wf-run.<run_id>             RunRecord JSON
  step.<run_id>.<step_name>   StepRecord JSON
  evt.<event_name>            JSON array of subscriber fn-names
  sub-run.<run_id>.<step>     child run-id (raw bytes)
```

Alternative backends (PostgreSQL, Redis, …) can be substituted by deploying a
different store component and updating the WADM link target — no changes to the
engine are required.  See [PLUGGABLE_STORAGE.md](./PLUGGABLE_STORAGE.md) for a
step-by-step guide.
