# Workflow API — REST Reference

Base URL: `http://localhost:8080` (deployed wasmCloud component)

Both JSON (`application/json`) and YAML (`application/yaml`, `text/yaml`, `application/x-yaml`, `text/x-yaml`) request bodies are accepted on all write endpoints.

---

## Endpoints

### Workflow Definition

| Method | Path | Description |
|--------|------|-------------|
| POST | `/workflows` | Register a new workflow |
| GET | `/workflows` | List all registered workflows |
| GET | `/workflows/{name}` | Get a workflow definition by name |
| DELETE | `/workflows/{name}` | Delete a workflow definition |

### Workflow Execution

| Method | Path | Description |
|--------|------|-------------|
| POST | `/runs` | Start a new workflow run |
| GET | `/runs/{run_id}` | Get run status |
| POST | `/runs/{run_id}/cancel` | Cancel a running workflow |
| GET | `/runs/{run_id}/ready` | List steps ready to execute |
| POST | `/runs/{run_id}/steps/{step}/done` | Mark a step as succeeded |
| POST | `/runs/{run_id}/steps/{step}/failed` | Mark a step as failed |
| POST | `/runs/{run_id}/steps/{step}/retry` | Reset a failed step to pending |
| GET | `/runs/{run_id}/steps/{step}/output` | Get step output bytes (base64) |

### Events

| Method | Path | Description |
|--------|------|-------------|
| POST | `/events/{event}/subscribe` | Subscribe a function to an event |
| POST | `/events/{event}/unsubscribe` | Unsubscribe a function from an event |
| POST | `/events/{event}/emit` | Emit an event to all subscribers |
| GET | `/events/{event}/subscribers` | List subscribers for an event |

---

## Schemas

### `WorkflowDef`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Alphanumeric + `-_`, unique identifier |
| `description` | string | no | Human-readable description |
| `timeout_ms` | integer | no | Overall run timeout in milliseconds |
| `triggers` | `TriggerDef[]` | no | Events that auto-start this workflow |
| `steps` | `StepDef[]` | yes | Ordered list of steps (min 1) |

### `StepDef`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | yes | — | Unique within the workflow |
| `depends_on` | string[] | yes | — | Names of steps that must complete first |
| `max_attempts` | integer | no | 1 | Retry limit (must be >= 1) |
| `base_delay_ms` | integer | no | 0 | Base backoff delay in milliseconds |
| `sub_workflow` | string | no | null | Delegate to a child workflow by name |
| `optional` | boolean | no | false | If true, skip does not fail the run |
| `condition` | `Condition` | no | null | Conditional execution rule |

### `Condition`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `on_step` | string | yes | Name of the step whose output to check |
| `equals` | any JSON value | yes | Expected output value (parsed from bytes) |

### `TriggerDef`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event` | string | yes | Event name that triggers this workflow |

### `RunRecord`

| Field | Type | Description |
|-------|------|-------------|
| `run_id` | string | UUID identifying the run |
| `wf_name` | string | Name of the workflow definition |
| `state` | string | `running` \| `succeeded` \| `failed` \| `cancelled` |
| `idem_key` | string \| null | Idempotency key (if provided on start) |
| `created_at_ms` | integer | Unix timestamp in milliseconds |

### `StepRecord`

| Field | Type | Description |
|-------|------|-------------|
| `state` | string | `pending` \| `succeeded` \| `failed` \| `skipped` |
| `attempt` | integer | Current attempt number |
| `scheduled_at_ms` | integer | When the step was scheduled |
| `output` | bytes (base64) \| null | Output set by done endpoint |
| `error` | string \| null | Error message set by failed endpoint |

---

## KV Key Schema

All keys are stored in the `workflow` bucket:

| Key Pattern | Value | Description |
|-------------|-------|-------------|
| `wf-def:{name}` | JSON `WorkflowDef` | Workflow definition |
| `wf-run:{run_id}` | JSON `RunRecord` | Run state |
| `step:{run_id}:{step_name}` | JSON `StepRecord` | Individual step state |
| `evt:{event_name}` | JSON `string[]` | List of subscriber function names |

---

## Status Codes

| Code | Meaning |
|------|---------|
| 200 | OK — operation succeeded |
| 201 | Created — resource registered or run started |
| 400 | Bad Request — validation error (body contains `"error"`) |
| 404 | Not Found — resource does not exist |
| 405 | Method Not Allowed |

---

## Error Response Format

All error responses return JSON:

```json
{"error": "human-readable description"}
```

---

## Start Run Request

```json
{
  "wf_name": "order-pipeline",
  "idem_key": "optional-unique-key"
}
```

When `idem_key` is provided and a run with that key already exists, the existing run is returned with status `200` (not `201`).

---

## Ready Steps Response

```json
[
  {"name": "validate", "kind": "normal"},
  {"name": "delegate", "kind": "sub_workflow", "sub_workflow": "child-wf"}
]
```

Steps are returned only when all their `depends_on` steps have succeeded (or been skipped).

---

## Retry Backoff Formula

```
delay = min(base_delay_ms * 2^attempt, 60_000)
```

Where `attempt` is the zero-based attempt count.
