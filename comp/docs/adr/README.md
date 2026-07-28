# Architecture decisions

Numbered, dated, one decision each, superseded rather than edited. Format and rules
in [ADR-0001](0001-use-adrs.md).

`PLATFORM.md` remains the narrative plan and the phase order; these own the forks
inside it. Where they disagree, the ADR wins.

| # | decision | status |
|---|---|---|
| [0001](0001-use-adrs.md) | Record architecture decisions as ADRs | accepted |
| [0002](0002-tenant-is-a-namespace.md) | A tenant is a Kubernetes namespace | accepted; isolation unit revised by [0014](0014-an-application-owns-a-host.md) |
| [0003](0003-control-plane-is-wasm-plus-applier.md) | The control plane is a wasm app plus a small native applier | accepted |
| [0004](0004-reconcile-by-server-side-apply-on-save.md) | Reconcile by server-side apply on save | accepted |
| [0005](0005-deployment-strategy-is-a-tenant-choice.md) | Deployment strategy is a tenant choice: fused or linked | accepted |
| [0006](0006-artifacts-are-digest-pinned-oci.md) | Artifacts are digest-pinned OCI; the WIT surface is the contract | accepted |
| [0007](0007-component-visibility-and-sharing.md) | Component visibility: private, org, public — and what public costs | accepted |
| [0008](0008-isolation-is-stamped-never-authored.md) | Isolation is stamped by the platform, never authored by tenants | storage half superseded by [0012](0012-keyvalue-isolation-needs-a-cooperative-component.md) |
| [0009](0009-identity-reuses-auth-guard.md) | Sign-in reuses `auth-guard`; OIDC is a later swap | accepted |
| [0010](0010-config-and-secrets.md) | Config is `wasi:config`; secrets never enter a manifest | accepted |
| [0011](0011-slice-one-scope.md) | Slice 1 is single-tenant, both strategies, one cluster | accepted |
| [0012](0012-keyvalue-isolation-needs-a-cooperative-component.md) | Per-tenant keyvalue isolation needs a cooperative component | accepted |
| [0013](0013-unenforceable-capabilities-are-denied-by-omission.md) | A capability the host cannot partition is denied by omission | superseded by [0014](0014-an-application-owns-a-host.md) |
| [0014](0014-an-application-owns-a-host.md) | An application owns a host | accepted |

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
| renderer (`(graph, strategy, tenant, plan) → manifests`) | `components/platform-domain/src/render.rs` | **done** — pure, 17 unit tests |
| control plane (accounts, catalog, deployments, revisions) | `components/platform-domain/src/lib.rs` | **done** |
| applier (SSA + validation + re-apply loop) | `applier/` | **done** — 7 unit tests, validate-only mode needs no cluster |
| both strategies, planner-validated | ADR-0005 | **done** — refuses a strategy the graph can't support |
| digest pinning enforced | ADR-0006 | **done** — a save with no digest is a 409 |
| isolation stamp (namespace, egress, blobstore containers) | ADR-0008 | **done** — and now per-app rather than per-tenant (ADR-0014) |
| a host per application (private data NATS, own engine, own endpoint) | ADR-0014 | **rendered and validated, not yet run on a cluster** — every host interface the operator binds is granted again |
| image allow-list on the applier | ADR-0014 | **done** — a `Deployment` may only run the platform's two pinned images, and no host namespaces, privilege, hostPath or service account |
| e2e | `examples/platform/tests/platform.rs` | **done** — no cluster required |
| registry push (the digest source) | — | **the gap.** `POST /api/internal/pushed` is the seam; nothing pushes yet |
| `public` visibility | ADR-0007 | refused with `501` until signing exists |
| tenant secrets | ADR-0010 | refused until `secretFrom` is proven |
| studio canvas as the editor | ADR-0011 item 9 | not wired — the API is what exists |
| a second tenant, apps that touch keyvalue | ADR-0008 gate | **blocked** — adversarial test run on a cluster and FAILED (ADR-0012); enforced in code |
| a second tenant, apps that do not | ADR-0013 | **open** — HTTP/config/blobstore-only graphs are host-partitioned, so the gate does not apply to them |
| dedicated host per tenant (the escape hatch, and the tier) | ADR-0013 | **available** — `grant-shared-state=true` plus `template.spec.environment`, no catalog change needed |
| tenant config (`localResources.config`) | ADR-0010 | not wired — a deployed app cannot be configured yet, so `mesh` on the cluster answers `no route configured` |
| namespace scaffolding applied | ADR-0002 | **done** — it rides along with every save, because the app's host pod needs it (ADR-0014) |

Run it: `just host-platform` (applier in validate-only — it builds no Kubernetes
client, so the default loop cannot touch a cluster), `just e2e-platform`,
`just host-platform-live` to actually apply.

## Open risks these ADRs name rather than solve

- ~~The keyvalue `buckets:` allow-list has never been exercised~~ → **tested on a
  real cluster, and it does not isolate.** Two tenants running the same app read the
  same records. The bucket is chosen by the guest's `store::open(name)`, not by
  manifest config, and every capability here hardcodes `"default"`. See
  [ADR-0012](0012-keyvalue-isolation-needs-a-cooperative-component.md); the gate is
  now **resolved** by ADR-0014 rather than worked around: the interface is bindable
  again because each application runs on its own host, whose data plane
  (`--data-nats-url`) is a loopback NATS sidecar in the app's own pod. Nothing else
  can reach the bus, so there is no bucket to allow-list. ADR-0013's default-deny was
  the interim answer and is superseded.
- ~~Namespace `NetworkPolicy` is inert for shared-host workloads~~ → **fixed by
  ADR-0014**: the app's host pod runs in the tenant's own namespace, so the policy
  selects it. It now has to allow the host's own egress (control-plane NATS, registry)
  or the app never registers.
- ~~The operator may not reconcile namespaces created after install~~ → **it does.**
  It holds ClusterRoles and runs with `-allow-shared-hosts=true`, and
  `template.spec.environment` schedules a workload onto a host in another namespace.
  ADR-0002 survives contact.
- **`secretFrom` has never been exercised** (ADR-0010). Until it is, no tenant
  secrets.
- **`wasi:keyvalue` has no CAS**, so all RMW state is best-effort and `lock:mutex`
  is advisory (ADR-0008). A published consistency envelope, not a bug to fix.
- **Per-workload CPU isolation is weak** (fuel traps composed apps), so
  noisy-neighbour risk is metered, not prevented *within* an app (ADR-0008). Between
  apps it is now a pod boundary (ADR-0014).
- **A rendered host pod has never been run.** The flags come from `wash host --help`
  and the chart's own `hostgroup-default`, the CRD facts from the cluster — but
  ADR-0014's manifests have only been validated, not applied. Deploy before believing.
- **Host plugins are not an authoring surface** — "plugin" in the v2 model means the
  host's built-ins, and nothing here has ever registered one (ADR-0005). Per-tenant
  KV backends wait on upstream #5051.
