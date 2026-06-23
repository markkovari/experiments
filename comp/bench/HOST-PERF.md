# Host performance — Node/jco vs native Rust (wasmtime)

The same vet-clinic app runs on three hosts (see `examples/jco-vet-domain`,
`host/`, `examples/vet-clinic-wasmcloud`). This compares the **HTTP request**
throughput / latency / memory of the hosts running it, to put numbers on the
"which host" choice. The capability components are identical Rust `.wasm` in
every case — what differs is the host and (for jco) the domain language.

> **Read this as stack-vs-stack, not language-vs-language.** The jco row runs
> the TypeScript `domain.ts` under Node+Fastify; the Rust rows run the Rust
> `vet-domain` component under a native wasmtime host. Different domain code AND
> different runtime. It is **not** a controlled apples-to-apples microbenchmark —
> it's "what does each deployable host actually deliver for this app."

## Method

- One machine (Apple Silicon, macOS), all hosts local, no k8s.
- Load: [`oha`](https://github.com/hatoo/oha), 15 s (reads) / 10 s (writes),
  50 conns (reads) / 20 conns (writes).
- Three operations:
  - **GET /pets** — hot read (auth introspect + a `records:store` indexed
    lookup). The cheapest path → best isolates per-request host overhead.
  - **POST /auth/login** — argon2 password verify (dominates; same wasm hash
    cost on every host) + session issue.
  - **POST /pets** — validate + `records:store` write + `search:index`.
- RSS sampled (`ps`) on the listening PID under sustained read load.
- Same seeded data; in-memory KV on all hosts for the latency runs.

## Results (representative, single machine)

| host / mode | GET /pets (req/s) | login (req/s) | POST /pets (req/s) | RSS under load | artifact |
|---|--:|--:|--:|--:|--|
| **Node + jco** (TS domain) | **3876** | 38 | 278 | 48 MB | 273 MB `node_modules` |
| **Rust host — on-demand alloc** | 503 | 165 | 514 | **24 MB** | 18.8 MB bin + 2.6 MB wasm |
| **Rust host — pooling alloc** (`--pool`) | 2957 | 175 | 1793 | 427 MB | 18.8 MB bin + 2.6 MB wasm |
| **wasmCloud (k8s, auth slice)** | — | **5.4** | — | per-host reservation | OCI image |

(The wasmCloud row is the **auth backend** — accounts-app + auth-guard — not the
full 19-component app; see the deploy findings below for why, and why 5.4 isn't
a runtime number.)

(Latency tracks throughput inversely: e.g. GET /pets mean was ~13 ms Node,
~100 ms Rust on-demand, ~17 ms Rust pooled.)

## What the numbers say

### The read-path gap is allocator, not language
The native Rust host serves each request in a **fresh wasmtime `Store`** (for
isolation) and instantiates the **19-component composed graph** per request.
With the **default on-demand allocator** that's fresh mmaps + table setup every
time → on a trivial read, *instantiation IS the cost* (503 req/s, 8× slower than
Node, which loads the wasm once and keeps it warm).

Switching to wasmtime's **pooling allocator** (`--pool`: pre-reserved,
recycled instance/memory/table slots — **the strategy wasmCloud uses**) collapses
that: **503 → 2957 req/s, ~6× faster**, into Node's ballpark. So the gap was the
host's allocation strategy, not Rust or wasmtime being slow.

### On work-heavy paths Rust already wins
Where the request does real work, per-request instantiation is amortized and the
native host is faster regardless of allocator:
- **login (argon2-bound): 165–175 vs 38 req/s — ~4.4× faster.** Node's argon2 is
  the bottleneck; the Rust component's is tighter.
- **POST /pets (write+index): 514 → 1793 vs 278 — up to 6.5× faster.**

### Memory is a real trade
- **On-demand Rust: 24 MB** under load vs **Node 48 MB** — ~2× lighter, and the
  artifact is a single 18.8 MB static binary + a 2.6 MB wasm vs **273 MB of
  `node_modules`**.
- **Pooling Rust: 427 MB** — the pooling allocator pre-reserves all its slots
  up front (here generous caps: ~10 k memory slots × up to 64 MiB). That's the
  trade: pooling buys instantiation speed with reserved memory. **Tunable** —
  shrink the caps in `host/src/main.rs` (`PoolingAllocationConfig`) to fit the
  real concurrency and the footprint drops accordingly.

## Estimated resource picture per deployment

| | Node + jco | Rust on-demand | Rust pooling |
|---|---|---|---|
| cold start | ~node boot + jco load | instant (single binary) | instant + slot pre-reserve |
| steady RSS | ~48 MB | ~24 MB | ~100–430 MB (cap-dependent) |
| read latency | low (warm module) | high (per-req instantiate) | low |
| write/auth latency | high (JS argon2) | low | low |
| image / footprint | 273 MB deps | ~21 MB total | ~21 MB total |
| best for | read-heavy, dev ergonomics | memory-tight, write/auth-heavy | balanced prod (the wasmCloud model) |

**Rule of thumb:** for a production deploy of a composed wasm app, use the
**pooling allocator** (or wasmCloud, which does this for you) and size the pool
to expected concurrency — you get Node-class read throughput, multiples better
write/auth throughput, a ~21 MB artifact, and a memory footprint you dial in.

## wasmCloud (k8s) — what actually happened

Deployed against the live in-cluster `wasmcloud-operator` host (v1.6.0,
JetStream NATS), `examples/vet-clinic-wasmcloud/k8s`. Two concrete findings:

### 1. The full 19-component app does NOT deploy on wasmCloud 1.6.0
```
failed to compile component:
  The component transitively contains 104 core module instances,
  which exceeds the configured maximum of 30
```
`vet_domain.full.composed.wasm` (19 components wac-plugged) flattens to **104
core module instances**, over the host's **hardcoded 30-instance cap**. That cap
is not exposed by the `WasmCloudHostConfig` CRD nor by a host flag/env in this
build (`--max-components`, `--max-linear-memory-bytes`, etc. exist; a
max-core-instances knob does not) — so a deep wac composition needs a host built
with a higher limit. **This is a real ceiling on composition depth for
wasmCloud, and the most important prod-parity finding here:** the same wasm that
runs fine under jco and the native wasmtime host is rejected by the wasmCloud
host's instance limit.

### 2. The shallow auth slice deploys + runs, but is provider-hop-bound
The **auth backend** (accounts-app + the composed auth-guard — a 2–3 component
graph, well under 30 instances) deploys cleanly: `register` → 201, `login` →
200 with real `sess_`/`ref_` tokens, persisted to NATS KV. Benched login:
**~5.4 req/s** — far below the local Rust host (165) or Node (38). That is **not
a runtime verdict**: it's dominated by

- a **per-KV-op round-trip to the `keyvalue-nats` provider** over NATS wrpc
  (argon2 login does several KV reads/writes; each is now a network hop vs the
  local in-memory store), plus
- `instances: 1` (a single host replica, no horizontal scaling), plus
- the `kubectl port-forward` in the measurement path.

So the wasmCloud number measures **durable networked KV + provider hops + single
replica**, the price of a real distributed deployment — orthogonal to the
host-runtime comparison above. To make it representative you'd scale
`spreadscaler` instances, run the load in-cluster (no port-forward), and note
that every persistence op is a deliberate network round-trip for durability.

### Takeaway
- For the **runtime** comparison, the local **pooling** Rust host is the right
  proxy for "what wasmtime can do" (it uses the same pooling allocator wasmCloud
  uses) — Node-class reads, multiples-better writes/auth, tunable memory.
- For a **real wasmCloud deploy**, two things bite before runtime does: the
  **composition-depth instance cap** (keep compositions shallow, or rebuild the
  host with a higher limit) and **provider round-trip latency** for durable KV
  (the cost of not being in-memory). Both are deployment-shape choices, not
  language or wasm-vs-native verdicts.

## Reproduce

```bash
# Rust host (add --pool for the pooling row):
just host-full            # on-demand
# edit the recipe / run directly with --pool for pooling

# Node/jco:
(cd examples/jco-vet-clinic && npm start)   # :3000

# load (seed + a token first):
oha -z 15s -c 50 -H "authorization: Bearer $TOKEN" http://127.0.0.1:PORT/pets
```
