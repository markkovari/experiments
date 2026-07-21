# golem-workflow — a native wRPC→Golem durable-worker provider

The first thing in this repo that isn't a wasm component: a **native wasmCloud
capability provider** that lets a component call a durable [Golem](https://golem.cloud)
worker through the typed `durable:workflow/orchestrator` contract. See
[`../../GOLEM.md`](../../GOLEM.md) for the design.

## What's verified (and what isn't)

- ✅ **Contract + mapping** — `wit/durable-workflow.wit` + the `Value` mapping in
  `src/lib.rs`, unit-tested (6 tests) against the real `golem_wasm_rpc::Value`.
- ✅ **Provider compiles** — `src/main.rs`: a real provider (wit-bindgen-wrpc
  `Handler` + `serve_provider_exports`). The tricky bit was pinning
  `wit-bindgen-wrpc 0.10` so its `wrpc-transport` (0.28) matches
  `wasmcloud-provider-sdk 0.17.1` — otherwise `WrpcClient: Serve` fails to resolve.
- ✅ **Live Golem e2e** — the bridge call (`invoke_golem`) invoked against a
  **running Golem 1.5** with a real deployed agent, asserting durable state
  advances. Automated: `GOLEM_E2E=1 cargo test`.
- ◻︎ **wasmCloud front-half** — a component calling the provider over wRPC on a
  lattice is **not run here**: the installed `wash 2.3` is the new component-shell
  (no classic host/`par`/wadm to run a native provider). The provider compiles;
  running it live needs the classic wasmCloud host.

## Reproduce

**Unit tests (no infra):**
```bash
cargo test              # 6 mapping tests; the live one skips without GOLEM_E2E
```

**Live e2e against a real Golem** (`bash e2e.sh`, or by hand):
```bash
# 1. Golem 1.5 (single self-contained binary — Golem's own local-dev path).
#    Prebuilt for this arch; no build. (Docker alt below.)
curl -fsSL -o .bin/golem \
  https://github.com/golemcloud/golem/releases/download/v1.5.5/golem-$(uname -m)-apple-darwin
chmod +x .bin/golem
.bin/golem server run --clean &          # gateway :9006, worker svc :9007

# 2. deploy the bundled demo agent (a durable counter — stands in for a workflow)
cd golem-agent && ../.bin/golem build && ../.bin/golem deploy -Y && cd ..

# 3. run the provider's bridge against it
GOLEM_E2E=1 cargo test bridge_invokes_a_real_durable_golem_worker
```

### Why the binary, not docker?

Golem 1.5's docker path is a multi-service `compose` (postgres + component /
worker / shard-manager / worker-executor services + nginx). The single `golem`
binary runs that whole platform in one process with embedded sqlite — it's
Golem's own quickstart path, starts in seconds, and there's no clean all-in-one
Golem image. For a reproducible CI-style run you can instead use Golem's
`docker-examples/published-postgres/compose.yaml`; the provider only needs the
gateway URL (`GOLEM_URL`) + `Host` header, so either backend works.

## Config (provider link-time)

| key | default | meaning |
|---|---|---|
| `GOLEM_URL` | `http://127.0.0.1:9006` | Golem API-gateway base |
| `GOLEM_HOST` | — | `Host` header for gateway subdomain routing (e.g. `bookapp.localhost:9006`) |
| `GOLEM_PATH_TEMPLATE` | `/counters/{workflow-id}/increment` | agent endpoint; `{workflow-id}` is substituted |
