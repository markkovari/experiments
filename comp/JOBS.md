# jobs — a durable background-job queue (with a swappable execution backend)

The last backend *class* the catalog was missing. The others are request/response,
streams, convergence, or a fixed compensation flow; this is **arbitrary
background work**: enqueue a job, it runs later, **fails → retries with backoff →
dead-letters**, and you can **replay** it. The Sidekiq/Temporal-lite shape.

And it's the first showcase whose *execution* is a **compose-time choice**: each
job runs through the `durable:workflow/orchestrator` contract, satisfied by an
**in-process backend** by default, or by the **`golem-workflow` provider** for
crash-resumable durable execution — with no change to the queue.

![The jobs board: a burst of jobs marches Queued → Running → Done; a flaky job retries with backoff (its attempt count climbs) then completes; a boom job exhausts its attempts and lands in the Dead-letter column; then Replay requeues it — a live recording of the running app, jobs advancing on their own over SSE](docs/media/jobs.gif)

## Two kinds of durability, at two layers

The recurring confusion about "durable queues" is that *durable* means two
different things. jobs keeps them separate:

- **Durable *timing* + *retry*** — owned by the queue. `outbox:dispatch` is the
  substrate: enqueue with a delay, claim under a crash-safe lease, `fail` with
  exponential backoff, dead-letter after the attempt cap, `replay`. All in
  key-value, so it survives a host restart (like `saga`).
- **Durable *execution*** — owned by the workflow backend. Whether a single
  long-running job body survives a crash *mid-run* and resumes is the job of
  whatever satisfies `durable:workflow` — the in-process backend doesn't (it
  re-runs from the top on retry, which is why handlers should be idempotent);
  Golem does.

`wasi:clocks` sits under both (reading time, pacing the SSE loop). It's not an
alternative to either — it's the primitive they're built on.

## The composition

| jobs concern | contract | how |
|---|---|---|
| the durable work queue | `outbox:dispatch` | enqueue(+delay) / claim(lease) / fail(backoff) / dead-letters / replay — the hard durable-queue mechanics, reused not rebuilt |
| running the job body | `durable:workflow` | each drain calls `orchestrator.trigger(workflow-id=type, payload)`; **in-process by default, Golem-swappable** |
| recurring jobs | `cron:expr` | on success, reschedule at the next cron fire time |
| exactly-once enqueue | `idempotency:guard` | a client `key` replays the first response instead of enqueuing again |
| the live board | `records:store` | one mirrored job record per work item (state, attempts, result/error) — the SSE dashboard's source |

A `tick` drains one batch (`claim → run → ack`/`fail`); the SSE board **ticks
itself**, so once you enqueue, jobs advance with no external driver — the same
in-guest streaming loop `pulse` uses.

## Swapping in Golem for real durable execution

The default compose plugs the **in-process** orchestrator:

```bash
just host-jobs        # jobs-domain + outbox + inproc-workflow + cron + idempotency + records
```

The in-process backend only implements the blocking `trigger` (a synchronous
decision from workflow-id + payload); `start`/`status` return `unavailable` —
async, crash-recovering runs are the Golem backend's domain. To get those, the
same `durable:workflow` import is satisfied by the **`golem-workflow` provider**
([`providers/golem-workflow`](providers/golem-workflow), see [GOLEM.md](GOLEM.md))
running on a **classic wasmCloud host** over wRPC on the lattice. The queue
component is byte-for-byte unchanged — that's the point of the seam.

> The provider is verified end-to-end against a **running Golem 1.5** (its bridge
> advances real durable state). The *front half* — a wasm component calling the
> provider over the lattice — needs the classic wasmCloud host and is **not run
> here** (the installed `wash 2.3` is the component-shell). So rung 1 (this
> showcase, in-process) is fully runnable; the Golem path is composed + documented,
> not claimed as verified live.

## Live on Kubernetes (v2 operator) with the Golem backend

The native `golem-workflow` provider needs a **classic** wasmCloud host + `wash
par` — neither exists under `wash 2.3`. But the **v2 runtime operator** provides
capabilities to a *component* as host interfaces (`wasi:http`, `wasi:keyvalue`),
and the provider's Golem call is just an HTTP POST — so there's a v2-native way
to run the front-half:

- **`golem-bridge`** ([`components/golem-bridge`](components/golem-bridge)) — a
  WASM component that satisfies the same `durable:workflow/orchestrator` contract
  by POSTing to a durable Golem worker over `wasi:http/outgoing-handler`.
- **`just compose-jobs-golem`** composes it into the queue in place of the
  in-process backend → `jobs_domain.golem.wasm`, which now imports
  `wasi:http/outgoing-handler` + `wasi:keyvalue` + `wasi:config` (all v2 host
  interfaces). `durable:workflow` is fully satisfied in-wasm.
- **`examples/jobs/k8s/jobs.yaml`** is the `WorkloadDeployment`: the fused
  component with host interfaces (incoming/outgoing http, keyvalue, config), the
  Golem address in `config`, and `allowedHosts` opened to the Golem gateway
  (egress is fail-closed). `just k8s-jobs` pushes the image to the in-cluster
  registry and applies it.

The bridge dials `golem-url` (default `host.docker.internal:9006` — the Golem
already running on the laptop, reachable from orbstack pods), targeting a
`counters` demo agent that stands in for a durable job body. So a job on the
board executes as a **real durable Golem worker**, on the v2 operator, with the
queue component byte-identical to the local build. The native provider remains
the option for a classic-host lattice.

### Verified live (2026-07-24)

Brought up end to end on the orbstack cluster: `runtime-operator` 2.5.2 (host +
NATS) in `jobs`, the in-cluster registry, and the `jobs` WorkloadDeployment
(READY 1/1). The board serves over cluster DNS
(`http://jobs.jobs.svc.cluster.local`), enqueue persists via the host's
NATS-backed `wasi:keyvalue`, and the `jobs-pump` drives ticks.

The front-half is **confirmed live**: an enqueued job drove the composed
component, on the v2 host, to make a real `wasi:http/outgoing-handler` call that
**reached the laptop's Golem** (`host.docker.internal:9006` via orbstack) — proven
by Golem's own `ROUTE_NOT_FOUND` response flowing back through the bridge into the
job's error and the retry→dead-letter path. Transport, egress, and durable
queue mechanics all work on-cluster.

What's left for a **green** run is Golem-side only: deploy an agent at the route
the bridge targets (`golem-path-template`) on the `:9006` instance and set
`golem-host` to its gateway vhost — the demo job types don't map to the currently
deployed `book:flight` agent. The bridge now sends `golem-host` as a distinct Host
header (connecting to `golem-url`), which is what gateway subdomain routing needs.

## The demo jobs

The in-process backend recognizes a few workflow ids so the lifecycle is
visible: `email` / `resize` / `report` / `echo` succeed; `flaky` fails while
`attempt < fail_until` (the queue passes the attempt number) then succeeds; `boom`
always fails (→ dead-letter). Real jobs would be real workflows behind the same
contract.

## Run it

```bash
just host-jobs        # native host + board on http://127.0.0.1:3038
just e2e-jobs         # lifecycle e2e: done / retry-then-succeed / DLQ / replay / exactly-once
```

`CFG_MAX_ATTEMPTS` (default here 2) and `CFG_BASE_BACKOFF` (1s) tune the outbox so
retries and dead-lettering happen within seconds.

## Rungs left

- **Golem execution live** — land the classic-host front-half so a job body is a
  real durable Golem worker (the provider is ready; needs the host).
- **Scheduled-job UI** — surface cron/next-run on the board (the backend already
  reschedules recurring jobs via `cron:expr`).
- **Concurrency + priorities** — multiple lanes, weighted claim.
