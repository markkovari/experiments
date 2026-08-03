# Strategy — where this competes, and in what order to build it

- **Status:** draft, for revision as evidence arrives
- **Date:** 2026-08-02
- **Companions:** [WHY.md](WHY.md) (the value claim), [ADR-0021](adr/0021-this-is-not-a-faas.md)
  (the category), [CLAIMS.md](CLAIMS.md) (what is actually proven)

> **Read [CLAIMS.md](CLAIMS.md) alongside this.** Several competitive claims below rest on
> `ASSUMED` beliefs, and they are marked. A strategy built on unproven claims is how
> `PLATFORM.md` got priced on density that
> [ADR-0012](adr/0012-keyvalue-isolation-needs-a-cooperative-component.md) then falsified.

---

## 1. The competitive picture

### Cloudflare Workers

**What they win, plainly.** A comparison that skips this is not useful.

| | Workers | here |
|---|---|---|
| edge locations | 330+ | 3, hypothetical ([SCENARIOS](SCENARIOS-ELASTICITY.md)) |
| cold start | ~5 ms, V8 isolates | ms-scale instantiation, one node **[M]** |
| proven scale | millions of Workers | **never run on two nodes** ([WHY.md](WHY.md)) |
| ecosystem | D1, R2, KV, Queues, Vectorize, AI, Hyperdrive | a component catalog, no managed services |
| ops burden | zero | a cluster, cells, placement |
| isolate density | ~3 MB, shared runtime, proven at scale | 2.3 Mi per component, one node **[M]** |

**For a stateless edge API, Workers wins. Do not compete there.**

The four differentiators that survive contact:

1. **Composition inside the deployment unit.** Workers' unit is a script; multiple Workers
   means Service Bindings or HTTP — better than a network hop, still a call boundary with
   separate deploys, versions and config. `vet-domain` is 16 components and 104 core
   modules in **one artifact**, one version, zero internal boundaries, pieces still
   separately authored and reusable. Their composition is at the platform level; this one
   is at link time, before deployment. **The only structural difference — matching it
   would mean changing their model.**
2. **Portability.** Workers code is written against Workers APIs; leaving means a rewrite.
   Components here import `wasi:keyvalue`, `wasi:http`, `auth:identity` — and the same
   bytes already run three ways (wasmCloud on k8s, `comp-host` native, jco in Node). That
   is demonstrated, not aspirational.
3. **Self-hosting.** [SELFHOST.md](SELFHOST.md) — own machines, `comp-host` + systemd,
   growing to k3s. **Cloudflare has no story here and structurally cannot.**
4. **A component catalog, not a service catalog.** They give services to *call* (each a
   network boundary, each separately billed); this gives 109 components to *compose*
   in-process. A failing component is a function returning an error, not a service that is
   down.

### Fermyon — the actual competitor

Independent, not Akamai. **Fermyon Spin is closer to this than Cloudflare is**, and further
along: SpinKube for wasm-on-Kubernetes, a component-model story, real tooling, funding.

The gap: **Spin's unit is an application of *triggers*, not a composed graph with
in-process linking.** `fused`/`linked` ([ADR-0005](adr/0005-deployment-strategy-is-a-tenant-choice.md))
and the 109-component catalog are more decomposition-native. But that gap is narrower than
against Cloudflare and they could close it.

**Study Fermyon, not Cloudflare.**

### The uncomfortable parts

- **These differentiators are architectural; theirs are operational.** Architecture is
  copyable by a funded team. 330 PoPs and a decade of hardening are not. This is the more
  copyable position. If the Component Model wins broadly, Cloudflare adopts it.
- **The measured advantages target the wrong competitor.** The 1.2 ms hop and 70 Mi floor
  **[M]** are measured against *Kubernetes pods*. Against V8 isolates with Service
  Bindings, or against Spin, both gaps narrow — and **nobody has measured it**. See
  [CLAIMS.md](CLAIMS.md) X-1; do not quote a Workers comparison until it exists.
- **The defensible core is the intersection**: composition + portability + self-hosting.
  Cloudflare can copy any one. Copying all three means abandoning their business model,
  since portability and self-hosting work against it.

---

## 2. Four paths, and what each actually needs

| path | effort | win | risk |
|---|---|---|---|
| **A — catalog + toolchain** | low | components run on Workers, Fermyon, wasmCloud, self-host | no moat; catalogs are copyable |
| **B — self-host lane** | low–med | works today; serves the segment CF cannot | smaller market |
| **C — full multi-tenant platform** | **high** | the density and tenancy story | competing with funded teams on their strength |
| **D — prove the thesis** | **lowest** | answers a publicly open question | not a business by itself |

**The observation that decides the order:** paths A, B and D need almost none of
[REQUIREMENTS-DENSITY](REQUIREMENTS-DENSITY.md),
[TOPOLOGY-MULTIREGION](TOPOLOGY-MULTIREGION.md) or
[SCENARIOS-ELASTICITY](SCENARIOS-ELASTICITY.md). Those four documents serve **C** — the
highest-effort, most-contested path — and C is gated on a claim that is currently
`ASSUMED`.

### Why "just build on Cloudflare" is not a strategy

Workers supports WASI P2. A `comp` component could plausibly run there — and *should*, since
portability is differentiator #2. So Cloudflare is a **deployment target, not a foundation.**

What cannot be built on it: placement control, the cell model, the density argument (their
isolates already win), self-hosting. Building "on CF" means shipping **A only** and dropping
tenancy, density and placement. That is a legitimate product — just a different one.

---

## 3. The sequence: D → A → B → C

Chosen because each step gates the next, and because **business, learning and credibility
are not competing goals here — they are the same sequence.** A measured answer to an open
question is publishable whether it passes or fails, and there is no business if the thesis
does not hold.

### D — Run G-1 (days)

[G-1](REQUIREMENTS-DENSITY.md#g-1--the-isolation-probe), extended to cover
`wasmcloud:messaging` subjects in the same pass since EX-2 depends on the same mechanism.

**Why first:** [CLAIMS.md](CLAIMS.md) rates I-6 `ASSUMED` with two full documents resting
on it. And I-3 was *more* credible than I-6 is now — documented, and required by its own
docs — and [ADR-0015](adr/0015-a-bucket-name-is-not-a-boundary.md) still found it broken.

- **Pass:** C becomes possible; the result is an upstream contribution.
- **Fail:** C is dead as specified, months saved, and the finding still publishes.

Either way it answers something nobody has publicly answered: *can wasmCloud be made
multi-tenant without a private data plane per app?*

### A — Catalog and toolchain (weeks)

Deliberately independent of G-1's outcome, which is what makes it the right second step —
it proceeds while the answer settles.

109 WIT-contracted composable components is the asset **neither Cloudflare nor Fermyon
has**. Valuable whether or not a platform ever runs.

### B — Self-host lane (polish, not architecture)

`comp-host`, systemd and [SELFHOST.md](SELFHOST.md) already exist. This needs documentation
and rough edges, not design. It is differentiator #3 and the segment Cloudflare cannot
serve — regulated, on-prem, sovereign.

### C — The platform (months, gated)

Only after D passes. And a G-1 pass is **not** sufficient: C still rests on

- **F-3** — proportional CPU shares between apps in one wasmtime engine: `ASSUMED` and
  **plausibly false**, since co-tenant apps are not separate OS scheduling entities (Q2b)
- **D-6** — 100 apps ≈ 300 Mi: arithmetic on D-1/D-2, never run
- **M-2** — JetStream redelivery in this platform's failure shapes: `DOCUMENTED`, untested here

Each needs its own measurement before anything is sold on it.

---

## 4. The business path

Ordered by what a buyer will believe, and each step independently valuable:

| # | milestone | makes true |
|---|---|---|
| 1 | **G-1 result** | the isolation model is real, not hoped for |
| 2 | **Density measured at 50+ apps** | D-6 becomes `MEASURED` instead of arithmetic |
| 3 | **Self-host lane shipped** | serves on-prem and regulated buyers, where CF cannot follow |
| 4 | **Managed platform** | the hardest sell, against funded incumbents |

Steps 1–3 are months, not years. Step 4 is where Fermyon is the competitor.

**Where revenue plausibly comes from first:** not the managed platform. On-prem and
regulated buyers who need wasm workloads on their own hardware have no Cloudflare option
at all, and the self-host lane already works.

---

## 5. What would change this

- **G-1 fails** → C is dead as specified. Fall back to a multi-application `comp-host` with
  store-scoped state, or accept pod-per-app permanently. A and B are unaffected.
- **F-3 turns out impossible** → fairness is enforced at admission and the edge only. That
  is a weaker guarantee and it changes what can be sold, not whether anything can be.
- **A Workers or Spin comparison gets measured and the gap is small** → composition alone
  is not enough; portability and self-hosting carry the whole case.
- **Cloudflare ships full component-model composition** → differentiator #1 is gone.
  Portability and self-hosting remain, and both are structurally unavailable to them.
- **The FaaS lane becomes most of the deployed workload** → the density argument has
  stopped applying to the real user base ([ADR-0021](adr/0021-this-is-not-a-faas.md)).
