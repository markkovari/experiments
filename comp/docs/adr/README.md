# Architecture decisions

Numbered, dated, one decision each, superseded rather than edited. Format and rules
in [ADR-0001](0001-use-adrs.md).

`PLATFORM.md` remains the narrative plan and the phase order; these own the forks
inside it. Where they disagree, the ADR wins.

| # | decision | status |
|---|---|---|
| [0001](0001-use-adrs.md) | Record architecture decisions as ADRs | accepted |
| [0002](0002-tenant-is-a-namespace.md) | A tenant is a Kubernetes namespace | accepted |
| [0003](0003-control-plane-is-wasm-plus-applier.md) | The control plane is a wasm app plus a small native applier | accepted |
| [0004](0004-reconcile-by-server-side-apply-on-save.md) | Reconcile by server-side apply on save | accepted |
| [0005](0005-deployment-strategy-is-a-tenant-choice.md) | Deployment strategy is a tenant choice: fused or linked | accepted |
| [0006](0006-artifacts-are-digest-pinned-oci.md) | Artifacts are digest-pinned OCI; the WIT surface is the contract | accepted |
| [0007](0007-component-visibility-and-sharing.md) | Component visibility: private, org, public — and what public costs | accepted |
| [0008](0008-isolation-is-stamped-never-authored.md) | Isolation is stamped by the platform, never authored by tenants | accepted |
| [0009](0009-identity-reuses-auth-guard.md) | Sign-in reuses `auth-guard`; OIDC is a later swap | accepted |
| [0010](0010-config-and-secrets.md) | Config is `wasi:config`; secrets never enter a manifest | accepted |
| [0011](0011-slice-one-scope.md) | Slice 1 is single-tenant, both strategies, one cluster | accepted |

## The shape these add up to

```
   browser ──▶ platform-domain (wasm)          ──http──▶ applier (native) ──▶ k8s API
               auth-guard · records · policy             SSA, namespace +
               quota · blob · wit:reflect                prefix validation
                     │                                          │
                     │ renderer: (graph, strategy,              ▼
                     │  tenant, plan) → manifests      ns/tenant-<slug>
                     │                                   WorkloadDeployment
                     ▼                                   Service · Quota · NetPol
               registry (OCI, digest-pinned)
```

## Implementation status (slice 1, ADR-0011)

| piece | where | state |
|---|---|---|
| renderer (`(graph, strategy, tenant, plan) → manifests`) | `components/platform-domain/src/render.rs` | **done** — pure, 12 unit tests |
| control plane (accounts, catalog, deployments, revisions) | `components/platform-domain/src/lib.rs` | **done** |
| applier (SSA + validation + re-apply loop) | `applier/` | **done** — 7 unit tests, validate-only mode needs no cluster |
| both strategies, planner-validated | ADR-0005 | **done** — refuses a strategy the graph can't support |
| digest pinning enforced | ADR-0006 | **done** — a save with no digest is a 409 |
| isolation stamp (namespace, bucket, fail-closed egress) | ADR-0008 | **done** in the renderer |
| e2e | `examples/platform/tests/platform.rs` | **done** — no cluster required |
| registry push (the digest source) | — | **the gap.** `POST /api/internal/pushed` is the seam; nothing pushes yet |
| `public` visibility | ADR-0007 | refused with `501` until signing exists |
| tenant secrets | ADR-0010 | refused until `secretFrom` is proven |
| studio canvas as the editor | ADR-0011 item 9 | not wired — the API is what exists |
| a second tenant | ADR-0008 gate | data model ready, adversarial test not written |

Run it: `just host-platform` (applier in validate-only — it builds no Kubernetes
client, so the default loop cannot touch a cluster), `just e2e-platform`,
`just host-platform-live` to actually apply.

## Open risks these ADRs name rather than solve

- **The operator may not reconcile namespaces created after it was installed**
  (ADR-0002). First thing to test; if it doesn't, ADR-0002 gets superseded.
- **The keyvalue `buckets:` allow-list has never been exercised** — only blobstore
  has (ADR-0008). It is the mechanism per-tenant storage isolation depends on, and
  it gates the second tenant.
- **`secretFrom` has never been exercised** (ADR-0010). Until it is, no tenant
  secrets.
- **`wasi:keyvalue` has no CAS**, so all RMW state is best-effort and `lock:mutex`
  is advisory (ADR-0008). A published consistency envelope, not a bug to fix.
- **Per-workload CPU isolation is weak** (fuel traps composed apps), so
  noisy-neighbour risk is metered, not prevented (ADR-0008).
- **Host plugins are not an authoring surface** — "plugin" in the v2 model means the
  host's built-ins, and nothing here has ever registered one (ADR-0005). Per-tenant
  KV backends wait on upstream #5051.
