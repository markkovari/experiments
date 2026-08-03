# Requirements — density and metering on a shared host

- **Status:** draft, not yet decided
- **Date:** 2026-08-02
- **Scope:** the execution and storage substrate for multi-tenant `comp`, plus the
  metering that makes its cost defensible
- **Relates to:** [ADR-0012](adr/0012-keyvalue-isolation-needs-a-cooperative-component.md),
  [ADR-0014](adr/0014-an-application-owns-a-host.md),
  [ADR-0015](adr/0015-a-bucket-name-is-not-a-boundary.md),
  [ADR-0019](adr/0019-the-density-number.md),
  [ADR-0020](adr/0020-the-density-number-under-load.md)

> This document states **what must be true**, not how. Where a requirement implies a
> design fork, the fork is named and left to an ADR. Nothing here is accepted until the
> gate in [G-1](#g-1--the-isolation-probe) passes; if it fails, most of FR-2 and all of
> the density NFRs are void and this document is superseded rather than edited.

---

## 1. Problem

The platform renders **one `wash host` Deployment plus one NATS sidecar per
application** ([ADR-0014](adr/0014-an-application-owns-a-host.md)). That shape was
forced, not chosen: `wasi:keyvalue` buckets are named by the guest
([ADR-0012](adr/0012-keyvalue-isolation-needs-a-cooperative-component.md)) and the
manifest field that appeared to fix it, `hostInterfaces[].name`, is both guest-forgeable
and broken on wash 2.5.2 ([ADR-0015](adr/0015-a-bucket-name-is-not-a-boundary.md)).

The cost of that shape is measured: **~70 Mi floor per app cold, ~233 Mi settled under
load**, against **2.3 Mi per extra component inside an existing host**
([ADR-0019](adr/0019-the-density-number.md),
[ADR-0020](adr/0020-the-density-number-under-load.md)). At 100 apps that is roughly
7 GB and 100 pods where a shared host would be ~300 Mi and one pod — the 24× that
ADR-0019 recorded as "blocked upstream".

It is only blocked in the *component-facing* contract. The mechanism that is **not**
guest-controlled is the **link between a component and a capability provider**: link
configuration is authored per workload by the platform, and a guest cannot choose which
link its call arrives on. A provider that derives its storage namespace from the link,
and ignores the identifier the guest passes to `open()`, restores isolation without
requiring any component in `components/` to cooperate — which is the requirement
ADR-0012 could not meet.

**This document specifies the platform that follows from that, and the evidence required
before believing it.**

### 1.1 Non-goals

- Replacing wasmCloud, wadm, or the operator. The value here is reusing them.
- Replacing `comp-host`. It stays the self-hosting and local-development lane
  ([docs/SELFHOST.md](SELFHOST.md)); it is not the multi-tenant substrate.
- Changing the control plane. The applier holding the only credential
  ([ADR-0003](adr/0003-control-plane-is-wasm-plus-applier.md)), reconcile on save
  ([ADR-0004](adr/0004-reconcile-by-server-side-apply-on-save.md)), digest pinning
  ([ADR-0006](adr/0006-artifacts-are-digest-pinned-oci.md)) and delete-by-label
  ([ADR-0016](adr/0016-deleting-an-app-is-reconciled-not-remembered.md)) are
  substrate-agnostic and stay as they are.
- Modifying catalog components. A change that requires editing all 109 is out of scope
  by construction — that constraint is what rules the ADR-0012 "cooperative component"
  fix out as the primary path.
- Billing, invoicing, payment. Metering produces the record; charging for it is later.

### 1.2 Definitions

| term | meaning here |
|---|---|
| **application** | the isolation unit ([ADR-0014](adr/0014-an-application-owns-a-host.md)): one or more components, one endpoint, one data namespace |
| **tenant** | the owner of applications; a Kubernetes namespace ([ADR-0002](adr/0002-tenant-is-a-namespace.md)) |
| **cell** | one `wash host` process and the set of applications placed on it; the failure, upgrade and blast-radius unit |
| **link** | the platform-authored binding between a component and a capability provider, carrying configuration the guest cannot author or forge |
| **namespace key** | the storage prefix a provider derives from the link, never from a guest argument |

---

## 2. Functional requirements

Priority: **M** must (slice), **S** should, **C** could, **W** won't (this slice).

### FR-1 — Placement

| id | requirement | pri |
|---|---|---|
| FR-1.1 | An application is placed onto a **cell**; multiple applications from **different tenants** may share one cell. | M |
| FR-1.2 | Placement is recorded in the app's desired state and is **visible to the tenant** — a tenant can see which cell class runs their app, and its blast-radius implications. | S |
| FR-1.3 | An application may be **pinned to a dedicated cell**, expressed as a field on the app spec, not a different code path. | M |
| FR-1.4 | An application may be moved between cells without changing its artifact, its config, or its endpoint. | S |
| FR-1.5 | The platform refuses placement onto a cell that would exceed that cell's declared component or memory budget, with a message naming the budget. | S |
| FR-1.6 | Placement is deterministic and reproducible from desired state — re-rendering an unchanged app yields the same cell. | M |
| FR-1.7 | **A tenant's apps are spread across cells** (anti-affinity by tenant). Concentrating one tenant's apps on one cell turns that cell's loss into a total outage for them while every other tenant is unaffected. Highest-value placement rule, and cheap. | M |
| FR-1.8 | Placement filters on **hard constraints first** — region and residency (MR-3), plan, and the cell's core-instance budget — then scores. The budget is arithmetic, not a heuristic: sum `total_modules` across the cell's apps against `WASMTIME_POOLING_TOTAL_CORE_INSTANCES` (`render.rs`). | M |
| FR-1.9 | Bin-packing is **worst-fit, not best-fit**: place on the emptiest viable cell. Best-fit maximises density and leaves no headroom, so the first burst has nowhere to go. Slack is what absorbs bursts without a placement decision. | S |
| FR-1.10 | Each cell reserves **explicit headroom** (a stated fraction of budget) so dynamic instantiation has somewhere to land. A fully-packed cell cannot serve the on-demand path at all. | S |
| FR-1.11 | **Stability over optimality.** Apps are never relocated for a marginal density gain — only on cell drain, budget violation, or explicit request. A move is a cold start, a dropped connection, and possibly a lost lease. | M |

### FR-2 — Storage isolation

> Gated on [G-1](#g-1--the-isolation-probe). Every requirement here is void if the probe
> shows link config does not carry app identity to the provider.

| id | requirement | pri |
|---|---|---|
| FR-2.1 | Two applications on **one cell**, sharing one storage backend, MUST NOT be able to read or write each other's records — by any key, any bucket identifier, or any sequence of calls. | M |
| FR-2.2 | The namespace key is derived **solely** from platform-authored link configuration. No guest-supplied value — including the argument to `store::open()` — may influence it. | M |
| FR-2.3 | The isolation holds for a component that **hardcodes** `open("default")`, which is what every catalog component does today (`components/record-store/src/lib.rs:47`). No catalog change may be required. | M |
| FR-2.4 | `hostInterfaces[].name` MUST NOT be emitted, per [ADR-0015](adr/0015-a-bucket-name-is-not-a-boundary.md). | M |
| FR-2.5 | The same guarantee applies to `wasi:blobstore`. Its existing host-enforced `buckets:` allow-list may satisfy this without new work; that must be **verified, not assumed**. | M |
| FR-2.6 | The same guarantee applies to `wasmcloud:messaging` subjects: an application must not be able to subscribe to or publish on another application's subjects. | M |
| FR-2.7 | Deleting an application deletes its storage namespace, reconciled not remembered ([ADR-0016](adr/0016-deleting-an-app-is-reconciled-not-remembered.md)). | S |
| FR-2.8 | An application may bring an **external** database instead, reached through per-tenant egress. Platform-provided storage is then not used and not billed. | C |

### FR-3 — Compute and resource fencing

| id | requirement | pri |
|---|---|---|
| FR-3.1 | No application may starve another on the same cell of CPU. A runaway or non-terminating guest must be bounded and must not stall unrelated applications. | M |
| FR-3.2 | No application may exhaust cell memory. Each application has a memory ceiling; exceeding it fails that application, not the cell. | M |
| FR-3.3 | Exceeding a limit produces a **tenant-visible, attributable event** — not a silent 500 and not a bare host log line. | S |
| FR-3.4 | Per-application concurrency is bounded (`poolSize`, `maxInvocations`), and the bound is part of the app's plan. | M |
| FR-3.5 | Egress stays default-deny with a per-application allow-list, as today (`allowedHosts`, both bare and port-qualified — `render.rs:393`). | M |

### FR-3b — Fairness between tenants

> **Placement is not fairness.** FR-1 decides where an app *sits*, once, at deploy time.
> It says nothing about who gets CPU at 15:00 when a co-placed neighbour is hot. Spreading
> apps evenly across cells and then letting them fight for a cell's resources is not a
> fair platform — it is an evenly-distributed unfair one. Fairness is a **runtime
> scheduling** property, and it needs its own mechanism.

The three failure modes, which need different answers:

| mode | what it looks like | answer |
|---|---|---|
| **noisy neighbour** | one app saturates a cell; co-placed apps see p99 blow out | runtime scheduling — FR-3b.1 to FR-3b.4 |
| **greedy tenant** | one tenant deploys 200 apps and consumes a cell class | admission quota — FR-3b.5, FR-3b.6 |
| **starvation** | a small app never gets scheduled next to a large one | floor guarantee — FR-3b.2 |

| id | requirement | pri |
|---|---|---|
| FR-3b.1 | Each application on a shared cell has a **CPU share**, not only a cap. Under contention, available CPU divides in proportion to share; under no contention an app may exceed its share and use idle capacity. Work-conserving: unused capacity is never wasted to enforce a limit nobody is waiting on. | M |
| FR-3b.2 | Each application has a **guaranteed floor** it receives even when every neighbour is saturating. The floor is what the tenant is sold; the share is how surplus divides. | M |
| FR-3b.3 | Fairness is enforced **per application**, and an app's share does not grow by splitting into more components. A `linked` app of 12 components and a `fused` app of 1 with the same plan get the same share, or the strategy choice ([ADR-0005](adr/0005-deployment-strategy-is-a-tenant-choice.md)) becomes a way to buy CPU. | M |
| FR-3b.4 | A tenant's total share on one cell is the sum of their apps' shares, and **that sum is bounded**. Otherwise deploying more apps is a way to take a larger slice of the same cell. | M |
| FR-3b.5 | Per-tenant admission quotas: max apps, max total components, max aggregate share, per cell class. Enforced at placement, refused with a message naming the limit (extends FR-1.5). | M |
| FR-3b.6 | Quotas are **per plan**, not global constants. A paying tenant gets a larger quota by configuration, not by a code path. | S |
| FR-3b.7 | Fairness applies to the **message bus** as well as CPU: consumer concurrency and delivery rate per app are bounded, so one app's backlog cannot monopolise a shared JetStream ([EX-4](TOPOLOGY-MULTIREGION.md#5-unfinished-work-lives-in-the-bus-not-in-the-cell)). | M |
| FR-3b.8 | Fairness applies to the **storage provider**: operations per second per app are bounded, so one app cannot saturate the provider its co-tenants share (B3 in the topology doc). | M |
| FR-3b.9 | Throttling is **observable and attributable**: a tenant can see that they were throttled, by which limit, and how often. Silent throttling is indistinguishable from a slow platform and generates support load instead of upgrades. | S |
| FR-3b.10 | Fairness violations are measurable in CI: a saturating app and a measured app on one cell, with a stated bound on the measured app's p99 degradation ([G-4](#g-4--the-noisy-neighbour-measurement)). | M |
| FR-3b.11 | **Free-tier apps are preemptible; paid apps are not.** Under sustained cell pressure the platform sheds the cheapest work first, and says so in the plan. | C |

#### Where the mechanism comes from

Three layers, and only the first is missing:

1. **CPU share and floor** — not in the catalog and not obviously in wasmCloud (Q2).
   Candidates: cgroup weights if a cell is a pod and apps are somehow separable within it
   (they are not — one host process); wasmtime **fuel** as a proportional budget, refilled
   per interval per app; or an admission-time concurrency cap per app, which is coarse but
   trivially correct. **This is the open design question of the whole fairness story.**
2. **Rate and quota** — `rate-limiter` (`ratelimit:guard`, fixed-window with lockout),
   `quota` (`quota:meter`, cumulative budget over a period), `throttle-domain` and
   `shaper` already exist and are exactly FR-3b.7 and FR-3b.8. They currently guard
   *tenant application* traffic; here they would guard *platform* resources. Same
   contracts, different subject.
3. **Admission quota** — `platform-domain` already refuses saves (the multi-tenant gate);
   FR-3b.5 is the same shape with a different predicate.

> **The honest gap:** the platform can already limit *how often* an app is called and *how
> much* it has consumed this period. It cannot yet make two apps sharing one wasmtime
> engine divide CPU proportionally under contention, because they are not separate OS
> scheduling entities. Until [Q2](#5-open-questions) is answered, fairness is enforced at
> admission and at the edge, not inside the engine — and that is a weaker guarantee that
> must be stated, not glossed.

### FR-4 — Metering

| id | requirement | pri |
|---|---|---|
| FR-4.1 | The platform records, **per application**: request count, CPU time, peak and time-integrated memory, storage operations, stored bytes, egress bytes. | M |
| FR-4.2 | Every metered quantity is attributed by the **runtime or the provider** — the component that actually observed the work — never inferred from pod-level cgroup counters, which stop being attributable at density. | M |
| FR-4.3 | The **70 Mi floor is visible** as its own line, not amortised into per-request cost. ADR-0019 requires this be honest rather than hidden. | M |
| FR-4.4 | Metering records survive a cell restart; a lost cell must not lose the usage that preceded it. | M |
| FR-4.5 | A tenant can read their own usage, per application, over a time range. | S |
| FR-4.6 | Usage is exportable in a form a billing system can consume, without that system understanding wasm, wasmCloud, or Kubernetes. | S |
| FR-4.7 | Metering overhead is measurable and bounded — see [NFR-P4](#nfr-p--performance). Instrumentation that costs more than it can bill for is a defect. | M |
| FR-4.8 | A tenant cannot see, infer, or influence another tenant's usage. | M |

### FR-5 — Control plane and lifecycle

| id | requirement | pri |
|---|---|---|
| FR-5.1 | `render.rs` emits **links** to shared providers instead of a per-app Host Deployment and NATS sidecar. Fewer resources of existing kinds, not new kinds. | M |
| FR-5.2 | `fused` and `linked` strategies keep working unchanged ([ADR-0005](adr/0005-deployment-strategy-is-a-tenant-choice.md)); density is orthogonal to intra-app composition. | M |
| FR-5.3 | Deploying an app to a shared cell must not restart, disturb, or drop traffic from other apps on that cell. | M |
| FR-5.4 | Cells are drainable: an operator can evacuate a cell for upgrade or node maintenance without tenant action. | S |
| FR-5.5 | The multi-tenant gate in `platform-domain` (refuse a second tenant unless `allow-multi-tenant=true`) stays shut until [G-1](#g-1--the-isolation-probe) and [G-2](#g-2--the-adversarial-suite) both pass. The gate stays **enforced in code**, per ADR-0012's own lesson that prose does not stop a deploy. | M |
| FR-5.6 | The platform can deploy **itself** onto itself: `platform-domain`, `auth-guard`, metering and catalog are wasm components on the same substrate. | S |
| FR-5.7 | Pod-per-app remains available as a placement choice for tenants who require it. It becomes a plan, not the only shape. | S |

### FR-6 — Won't, this slice

| id | requirement |
|---|---|
| FR-6.1 | Cross-node scale-out of a single application (still capped at one host, per ADR-0014). |
| FR-6.2 | Live migration of a running application between cells without a request drop. |
| FR-6.3 | Autoscaling cells by load. |
| FR-6.4 | Charging money. Metering only. |
| FR-6.5 | The docker lane from `PLATFORM.md`. |

---

## 3. Non-functional requirements

### NFR-D — Density

Baselines are ADR-0019 (cold) and ADR-0020 (settled under load). **Targets must be
restated against a settled host, not a cold one** — ADR-0019's own consequences say to
quote 0020's numbers publicly.

| id | requirement |
|---|---|
| NFR-D1 | 10 single-component applications on one cell fit in materially less memory than 10 pod-per-app deployments. Baseline to beat: ~700 Mi cold / ~2.3 Gi settled. |
| NFR-D2 | Marginal cost of an additional application on a warm cell approaches the marginal cost of a component (~2.3 Mi cold for a small component), not the cost of a host (~70 Mi). This is the central claim; failing it voids the design. |
| NFR-D3 | A cell supports at least 50 applications before placement must refuse, on a node of a size to be stated with the measurement. |
| NFR-D4 | The saved network hop (1.2 ms p50 in-cluster) is preserved: co-placed apps must not gain a hop relative to today. |
| NFR-D5 | Every density figure is measured with the existing `bench/` harness on real workloads, reported cold **and** settled, and recorded in an ADR. No projected or extrapolated number is quotable. |

### NFR-S — Security and isolation

| id | requirement |
|---|---|
| NFR-S1 | The storage boundary is enforced by a mechanism the guest cannot address, name, or forge. A configuration field that nothing reads is worse than no field — ADR-0012 shipped through two deploys and a green test suite on exactly that. |
| NFR-S2 | The adversarial test is **permanent CI**, not a one-off. A regression in isolation must fail a build. |
| NFR-S3 | Default-deny for every capability: absent explicit grant, an application reaches nothing. |
| NFR-S4 | Compromise of one application's guest code must not yield another application's data, config, or secrets. |
| NFR-S5 | Secrets never enter a manifest ([ADR-0010](adr/0010-config-and-secrets.md)); link configuration carrying namespace keys is not a secret and must not be treated as one. |
| NFR-S6 | The provider is a trust boundary. Its namespace-derivation path is small enough to audit by reading, and is reviewed as security-relevant code. |

### NFR-R — Reliability and blast radius

| id | requirement |
|---|---|
| NFR-R1 | Blast radius is the **cell**, stated plainly to tenants. A shared cell is cheaper and less isolated; that trade is the tenant's to make, and hiding it is not acceptable. |
| NFR-R2 | A single application's crash, trap, or resource exhaustion must not take down its cell. |
| NFR-R3 | A cell loss loses no committed storage and no metering record. |
| NFR-R4 | Cell restart time is bounded and stated; recovery must not require tenant action. |
| NFR-R5 | Drift detection and reconcile keep working per [ADR-0004](adr/0004-reconcile-by-server-side-apply-on-save.md) at cell granularity. |

### NFR-P — Performance

| id | requirement |
|---|---|
| NFR-P1 | Per-request latency on a shared cell is no worse than pod-per-app today. ADR-0020 measured a 36% better p99 for co-location; that must not regress. |
| NFR-P2 | Storage operation latency through the shared provider is within a stated budget of the per-app NATS it replaces. If it is worse, the number is published, not omitted. |
| NFR-P3 | A noisy application's load must not degrade a co-placed application's p99 beyond a stated bound. Untested, this is the most likely failure of the whole design. |
| NFR-P4 | Metering adds under 5% to request latency and under 5% to cell memory. Exceeding either makes the instrumentation a cost centre rather than a revenue enabler. |

### NFR-O — Operability

| id | requirement |
|---|---|
| NFR-O1 | An operator can answer "which cell is app X on, and what else is there" from platform state alone. |
| NFR-O2 | Per-application metrics, logs and traces are attributable at density, where the pod boundary no longer distinguishes tenants. |
| NFR-O3 | A cell upgrade (wash version, provider version) is rollable and reversible. |
| NFR-O4 | The platform runs on one node for development, per [docs/SELFHOST.md](SELFHOST.md). Density must not require a cluster to exercise. |

### NFR-C — Cost

| id | requirement |
|---|---|
| NFR-C1 | The cost of running an application is derivable from its metering record and the cell's cost. |
| NFR-C2 | Unit economics are stated per plan: what a shared-cell app costs to serve versus what a dedicated-cell app costs. |
| NFR-C3 | The fixed floor per cell is amortised across its applications, and that amortisation is visible rather than folded into a per-request price. |

---

## 4. Gates

Nothing downstream is built until the gate above it passes. Each gate is a
**measurement**, and each produces an ADR whether it passes or fails.

### G-1 — The isolation probe

**Question:** does platform-authored link configuration reach a capability provider with
the calling application's identity, such that the provider can derive a namespace the
guest cannot influence?

**Method:** extend `components/kv-probe` — which exists for precisely this and takes its
bucket from the query string because every catalog component hardcodes `open("default")`.
Two workloads, **one** cell, **one** provider, distinct link configuration, deliberately
hostile to the hypothesis, as in ADR-0015:

- workload A writes `k`; workload B attempts to read it under every identifier it can construct
- both workloads call `open()` with identical, guest-chosen arguments
- B attempts identifiers belonging to A, absent, empty, and adversarially malformed

**Pass:** the provider observes distinct identities; B cannot reach A's record by any
attempted path.

**Fail:** FR-2 is unimplementable as specified. The design falls back to Option A of the
earlier analysis (a multi-application `comp-host` with store-scoped host state), or to
pod-per-app remaining permanent. Either way this document is superseded, not patched.

> ADR-0015 killed one documented, plausible-looking mechanism after measurement. The same
> scepticism applies here: **link configuration is a hypothesis until this probe passes.**

### G-2 — The adversarial suite

ADR-0012's failed test, made permanent and broadened: two tenants, one cell, keyvalue and
blobstore and messaging, run in CI. Satisfies NFR-S2 and unlocks FR-5.5.

### G-3 — The density measurement

NFR-D1 through NFR-D5, using `bench/`, cold and settled. Produces the number that either
justifies this work or ends it.

### G-4 — The noisy-neighbour measurement

NFR-P3. One saturating application, one measured application, same cell. Published
whatever it shows.

---

## 5. Open questions

| # | question | blocks |
|---|---|---|
| Q1 | Does wasmCloud's link configuration actually carry per-workload identity to a provider on wash 2.5.2? | everything — this is G-1 |
| Q2 | Are wasmCloud's per-component CPU and memory limits fine-grained enough for FR-3.1 and FR-3.2, or does fencing need host changes? | FR-3 |
| Q2b | **How is a proportional CPU share (FR-3b.1, FR-3b.2) enforced between apps inside one wasmtime engine, where apps are not separate OS scheduling entities?** Fuel-as-budget, admission-time concurrency caps, or a cell-per-plan-class so the fairness boundary is the OS's problem again. Hardest open question in this document. | FR-3b |
| Q3 | Does the existing `buckets:` allow-list already satisfy FR-2.5 for blobstore, or does blobstore need the same provider treatment? | FR-2.5 |
| Q4 | What backs the shared provider — SQLite, Redis/Valkey, Postgres? `host/src/kv.rs` already implements three; which survives at density? | FR-2, NFR-P2 |
| Q5 | Where do metering records land, and does that store become a shared-fate dependency for every cell? | FR-4.4, NFR-R3 |
| Q6 | Is a cell one per node, or several per node? Per-node maximises density; several bound blast radius. | FR-1, NFR-R1 |
| Q7 | Does the platform hosting itself (FR-5.6) create a bootstrap cycle — can the platform deploy the provider its own storage depends on? | FR-5.6 |
| Q8 | What is the upstream story? ADR-0015 found `hostInterfaces[].name` broken and worth an upstream issue. Does the provider path make that issue moot, or still worth filing? | none; strategic |

---

## 6. What this replaces if accepted

- **ADR-0014** ("an application owns a host") — narrowed from a universal rule to one
  placement option among several. Its *reasoning* stands; its scope shrinks.
- **PLATFORM.md's isolation model** — already marked falsified. This would give it a
  successor rather than leaving it as a warning.
- **ADR-0019's "blocked upstream"** — reclassified. The 24× is reachable without an
  upstream fix, by a provider rather than by `hostInterfaces[].name`.

Nothing here is accepted until G-1 passes.
