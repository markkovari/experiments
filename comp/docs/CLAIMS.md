# Claims register

Every load-bearing belief in this project, with its evidence status.

**Why this exists.** Every expensive mistake here followed one pattern: *a belief about a
mechanism became load-bearing before anything measured it.*
[ADR-0008](adr/0008-isolation-is-stamped-never-authored.md) said outright that its
mechanism was "only ever been exercised for blobstore" and therefore unproven — and shipped
anyway, through two deploys and a green test suite, because nothing turned that sentence
into a blocker. [ADR-0015](adr/0015-a-bucket-name-is-not-a-boundary.md) then found a
*documented* CRD field that does not work at all. Neither was a compromised principle;
both were unverified facts holding up real decisions.

This register is the blocker that sentence should have been.

## Status vocabulary

| status | meaning | what it may support |
|---|---|---|
| **MEASURED** | verified in this repo, traceable to an ADR or bench run | anything |
| **DOCUMENTED** | an upstream doc or spec says so; **not tested here** | design exploration only — **never a release gate** |
| **ASSUMED** | believed on reason or analogy; no evidence | nothing load-bearing |
| **FALSIFIED** | tested and false | nothing; kept so it is not re-proposed |

## The rules

1. **A `DOCUMENTED` claim gets an adversarial test before anything depends on it.**
   ADR-0015 is the monument: documented, plausible, required by its own docs, and broken.
2. **An `ASSUMED` claim cannot gate a release** and cannot be quoted externally.
3. **A `MEASURED` claim states its conditions** — cold or settled, idle or loaded, which
   runtime version (P2).
4. **A `FALSIFIED` claim is never deleted.** It is the cheapest defence against
   re-proposing a dead idea.
5. **Adding a dependency on a claim means checking its status first.** If a design rests on
   `ASSUMED`, that is the design's biggest risk and must be named in the design.

---

## Isolation

| # | claim | status | evidence | depended on by |
|---|---|---|---|---|
| I-1 | wasm sandboxing isolates compute between components | **MEASURED** | [ADR-0019](adr/0019-the-density-number.md), [ADR-0020](adr/0020-the-density-number-under-load.md) | everything |
| I-2 | A per-tenant bucket stamped on `hostInterfaces` isolates storage | **FALSIFIED** | [ADR-0012](adr/0012-keyvalue-isolation-needs-a-cooperative-component.md) — two tenants read the same record | ~~ADR-0008~~ |
| I-3 | `hostInterfaces[].name` selects a store per workload | **FALSIFIED** | [ADR-0015](adr/0015-a-bucket-name-is-not-a-boundary.md) — breaks the workload *and* is guest-forgeable | nothing |
| I-4 | A private data-plane NATS per app isolates storage and messaging | **MEASURED** | [ADR-0014](adr/0014-an-application-owns-a-host.md), confirmed [ADR-0015](adr/0015-a-bucket-name-is-not-a-boundary.md) | current production shape |
| I-5 | `wasi:blobstore`'s `buckets:` allow-list is host-enforced, not guest-chosen | **DOCUMENTED** | precedent in `bench-suite-v2.yaml`; asserted in ADR-0012, never adversarially tested | FR-2.5 — **needs its own test** |
| I-6 | Platform-authored **link config** carries app identity to a provider, unforgeable by the guest | **ASSUMED** | none | **[G-1](REQUIREMENTS-DENSITY.md#g-1--the-isolation-probe), and through it all of REQUIREMENTS-DENSITY and TOPOLOGY-MULTIREGION** |
| I-7 | The same link mechanism can namespace `wasmcloud:messaging` subjects | **ASSUMED** | none | FR-2.6, EX-2 |

> **I-6 is the single largest unverified dependency in the project.** Two documents rest on
> it. It is `ASSUMED`, which by rule 2 means nothing may ship on it — hence G-1 existing
> before any code. Note that I-3 was *more* credible than I-6 is now (it was documented)
> and still failed.

## Density and performance

| # | claim | status | evidence | conditions |
|---|---|---|---|---|
| D-1 | An extra component on a warm host costs ~2.3 Mi | **MEASURED** | [ADR-0019](adr/0019-the-density-number.md) | **cold start**, 75 KB component, wash 2.5.2, `poolSize: 8` |
| D-2 | A host floor is ~70 Mi cold, ~233 Mi settled | **MEASURED** | [ADR-0019](adr/0019-the-density-number.md), [ADR-0020](adr/0020-the-density-number-under-load.md) | quote the settled figure (P2) |
| D-3 | An in-cluster network hop costs ~1.2 ms p50 | **MEASURED** | [ADR-0019](adr/0019-the-density-number.md) | one cluster, one node |
| D-4 | Packing components costs no throughput and improves p99 | **MEASURED** | [ADR-0020](adr/0020-the-density-number-under-load.md) — identical rps and CPU, 3.2× less memory, 36% better p99 | 200 conns, one node, 15 s |
| D-5 | D-1's slope holds for large components | **ASSUMED** | ADR-0019 says explicitly it does not know — 75 KB measured, a 1 MB component will differ | any extrapolation to the real catalog |
| D-6 | 100 apps on shared cells ≈ 300 Mi | **ASSUMED** | arithmetic on D-1/D-2, never run | the density pitch — **do not quote** |
| D-7 | A cell holds ≥50 apps before refusing placement | **ASSUMED** | none | NFR-D3 |
| D-8 | Dynamic instantiation absorbs a 15× traffic swing without a placement change | **ASSUMED** | none | [Scenario A](SCENARIOS-ELASTICITY.md#1-scenario-a--the-diurnal-wave-the-common-case), U1 |
| D-9 | Scale-out to a pre-pulled cell completes in seconds | **ASSUMED** | none | [Scenario B](SCENARIOS-ELASTICITY.md#2-scenario-b--the-unpredicted-spike-exceeding-headroom), R3, U2 |

## Fairness and resource control

| # | claim | status | evidence | depended on by |
|---|---|---|---|---|
| F-1 | A per-app engine budget stops one app starving another's instances | **MEASURED** | [ADR-0014](adr/0014-an-application-owns-a-host.md) — the vet-clinic failure it fixed | current shape |
| F-2 | wasmCloud can fence CPU and memory **per app inside one host** | **ASSUMED** | none — Q2 | FR-3.1, FR-3.2 |
| F-3 | Proportional CPU shares are enforceable between apps in one wasmtime engine | **ASSUMED** | none, and **plausibly false** — co-tenant apps are not separate OS scheduling entities (Q2b) | FR-3b.1, FR-3b.2 |
| F-4 | Throttling a consumer without breaching `ack_wait` is achievable | **ASSUMED** | none — throttling past `ack_wait` causes redelivery storms | FR-3b.7, [U4](SCENARIOS-ELASTICITY.md#7-what-is-unproven-here) |
| F-5 | Correlating co-placed apps' latency distinguishes a sick cell from a slow app | **ASSUMED** | none | [R5](SCENARIOS-ELASTICITY.md#6-the-rules-extracted), U3 |

> **F-3 is the one to watch.** It is not merely untested — the mechanism it presumes may
> not exist. Fairness may have to be enforced at admission and at the edge, which is a
> weaker guarantee that changes what can be sold.

## Durability and messaging

| # | claim | status | evidence | depended on by |
|---|---|---|---|---|
| M-1 | KV-backed state survives a host restart | **MEASURED** | `SAGA.md` — host killed mid-saga, resumed | saga, jobs |
| M-2 | Unacked JetStream messages redeliver to another consumer after `ack_wait` | **DOCUMENTED** | NATS semantics; not tested *in this platform's failure shapes* | [EX-1](TOPOLOGY-MULTIREGION.md#5-unfinished-work-lives-in-the-bus-not-in-the-cell), all of Scenario E |
| M-3 | A wasmtime `Store` cannot be checkpointed or migrated mid-execution | **DOCUMENTED** | wasmtime's model; no serialisation exists | EX-9 — the reason in-flight HTTP is out of scope |
| M-4 | Graceful drain finishes in-flight work under load | **ASSUMED** | none — and it is the *common* case, since planned termination dominates | EX-5, FR-5.4 |
| M-5 | `golem-workflow` provides mid-body crash resumption | **DOCUMENTED** | Golem's contract; `JOBS.md` states it, this repo has not verified it | the durable-execution tier |

## Control plane

| # | claim | status | evidence |
|---|---|---|---|
| C-1 | Server-side apply reconciles drift idempotently | **MEASURED** | [ADR-0004](adr/0004-reconcile-by-server-side-apply-on-save.md), [ADR-0018](adr/0018-the-platform-deploys-a-running-app.md) |
| C-2 | Delete-by-label removes an app's whole footprint | **MEASURED** | [ADR-0016](adr/0016-deleting-an-app-is-reconciled-not-remembered.md), ADR-0018 |
| C-3 | Digest pinning makes deploys reproducible | **MEASURED** | [ADR-0006](adr/0006-artifacts-are-digest-pinned-oci.md), enforced in `render.rs` |
| C-4 | The platform can host itself | **ASSUMED** | not built — FR-5.6, and it may have a bootstrap cycle (Q7) |
| C-5 | A single app scales past one host | **FALSIFIED** for the current shape | [ADR-0014](adr/0014-an-application-owns-a-host.md) — ReadWriteOnce PVC + `Recreate`; needs a real NATS cluster |

## Competitive

| # | claim | status | evidence | depended on by |
|---|---|---|---|---|
| X-1 | The hop and floor advantages hold against **V8 isolates / Cloudflare Workers** | **ASSUMED** | none — D-1…D-4 are measured against *Kubernetes pods*. Against isolates with Service Bindings both gaps narrow | any Workers comparison — **do not quote until measured** |
| X-2 | The same advantages hold against **Fermyon Spin / SpinKube** | **ASSUMED** | none. Spin is also wasm-on-k8s, so the pod-comparison baseline does not apply | [STRATEGY.md](STRATEGY.md) §1 |
| X-3 | In-process linking of many components is a shape Workers and Spin cannot express | **DOCUMENTED** | Workers' unit is a script + Service Bindings; Spin's is an app of triggers. From their docs, not tested here | differentiator #1, [ADR-0021](adr/0021-this-is-not-a-faas.md) |
| X-4 | The same component bytes run on wasmCloud, `comp-host` and jco | **MEASURED** | [ROADMAP.md](../ROADMAP.md) — identical e2e passes on all three | differentiator #2 (portability) |

> **X-1 is the marketing risk.** Every headline number in [WHY.md](WHY.md) is measured
> against pods, which is the right comparison for the container audience and the wrong one
> for the wasm-native competitors. Quoting them at a Cloudflare or Fermyon buyer would be
> the [P2](PRINCIPLES.md) violation this register exists to prevent.

## Multi-region

Everything in [TOPOLOGY-MULTIREGION.md](TOPOLOGY-MULTIREGION.md) marked **[E]** is
`ASSUMED`. Nothing has been run across regions.

| # | claim | status | depended on by |
|---|---|---|---|
| R-1 | Inter-region RTT is 60–150 ms EU↔US | **ASSUMED** | every latency claim in the topology doc (T6) |
| R-2 | Pilot-light failover achieves 1–5 min RTO | **ASSUMED** | §3.1's cost/benefit |
| R-3 | DNS failover takes 30–120 s | **ASSUMED** | bottleneck B1's ranking |
| R-4 | A globally stable namespace key makes failover reads correct | **ASSUMED** | MR-2 — and it pulls against MR-3 (residency) |

---

## Working with this register

**Adding a claim.** Anything a design depends on that could be false. If it feels too
obvious to write down, ADR-0015's field was documented and required by its own docs.

**Promoting a claim.** `ASSUMED` → `DOCUMENTED` needs a citation. `DOCUMENTED` → `MEASURED`
needs a test in this repo, run against a hostile setup (ADR-0015's probe was deliberately
built to disprove its hypothesis, which is why it worked).

**Falsifying a claim.** Change the status, keep the row, link the ADR, and fix everything
listed in "depended on by" — that column exists to make the blast radius visible before you
start.

**Review point.** Before a release gate, before an external claim, and when writing any
design doc: check the status of everything it rests on. A design resting on `ASSUMED` is
not wrong — but it must say so, the way
[REQUIREMENTS-DENSITY.md](REQUIREMENTS-DENSITY.md) says G-1 gates it.
