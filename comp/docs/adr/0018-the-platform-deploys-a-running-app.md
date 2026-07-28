# ADR-0018 — The platform deploys a running app, and what that took

- **Status:** accepted
- **Date:** 2026-07-28
- **Confirms:** [ADR-0014](0014-an-application-owns-a-host.md), [ADR-0016](0016-deleting-an-app-is-reconciled-not-remembered.md), [ADR-0017](0017-the-applier-pushes-and-the-registry-is-a-cache.md)

## Context

Everything up to here was measured in pieces. The renderer had unit tests, the applier
had unit tests, the e2e ran with no cluster, and the individual cluster mechanisms —
host pods registering, `environment` pinning, loopback buses isolating, orphaned Hosts
reaping — had each been checked by hand with `kubectl`. What had never happened was
**the platform itself driving an upload all the way to a serving application.**

The registry manifest from ADR-0017 had also never been applied.

## Decision

Both are now done, and this ADR records the run and the three bugs it found — because
each is a bug no amount of manifest review would have produced, and that is the point
worth writing down.

**The run**, against the real cluster with a real apply (no `--dry-run`):

```
register → upload kv-probe.wasm → reflected + staged
  → applier's loop pushes it to registry.platform.svc.cluster.local:5000
  → oci_ref recorded, deployable: true
  → create deployment → save
  → 7 objects applied: Namespace, ResourceQuota, NetworkPolicy,
    PersistentVolumeClaim, Deployment (the app's host + NATS sidecar),
    WorkloadDeployment, Service
  → host pod 2/2, registers as environment app-live-probe
  → workload Ready, EndpointSlice populated
  → GET /who   {"bucket":"mine","open":"ok"}
  → GET /put   {"put":"hello","value":"deployed-by-the-platform"}
  → GET /get   {"found":true,"value":"deployed-by-the-platform"}
```

**Per-app isolation, the adversarial test ADR-0008 demanded, at the level that
matters.** Two apps of the *same tenant*, from the *same component*, writing the *same
bucket and key*:

| | app `probe` | app `twin` |
|---|---|---|
| reads `mine/hello` after `probe` wrote it | `found: true` | **`found: false`** |
| after each writes its own value | `deployed-by-the-platform` | `written-by-twin` |

Two values, same key, same bucket name, neither visible to the other. That is
ADR-0014's claim, measured through the platform rather than by hand.

**Deletion, on a real cluster.** `DELETE` without the token → `428` with the message.
With it: `WorkloadDeployment`, `Service`, the host `Deployment`, the `PersistentVolumeClaim`
and **`Host/silky-magic-9662`** — the self-registered object, found by environment, which
is the thing ADR-0016 exists for. The sibling app kept serving throughout, and the
tenant's namespace, quota and policy survived, as ADR-0016 says they must.

## The three bugs it found

**1. The app was deployed and unreachable.** The workload ran, the host served on
`:9191`, and the Service had no endpoints. The operator's route controller said:

```
pod lookup failed for non-IP host.Hostname, skipping workload
err: no pod labelled "wasmcloud.com/hostgroup" matches hostname "app-live-probe-host-..."
```

The renderer omitted `--host-name`, so the host advertised its **pod name**; the
controller resolves a host to a pod either by an IP hostname or by the
`wasmcloud.com/hostgroup` label, and had neither. The chart's own hostgroup passes
`--host-name=$(WASMCLOUD_HOST_IP)` via the downward API — matching it, and adding the
label, fixes it. *Deployed and unreachable is the worst failure shape available:*
every status field said healthy.

**2. The tenant NetworkPolicy allowed egress to one infra namespace.** It permitted the
platform namespace, but the scheduler NATS lives with the operator, which need not be
the same namespace — on this cluster the operator is in `jobs` and the registry in
`platform`. An app whose host cannot reach the scheduler bus never registers, for a
reason nothing in the app explains. `control_plane_ns` is now separate from
`platform_ns` and both are allowed.

**3. `NetworkPolicy` is not enforced on this cluster at all.** A pod in an unlabelled
namespace reached the registry; `kube-system` holds only coredns, local-path-provisioner
and metrics-server — no CNI daemonset, no policy controller. This does not break the
run (bug 2 was therefore invisible here, which is exactly why it is dangerous), but it
**removes a layer ADR-0017 counted on**: the registry's policy is documentation, and the
only control actually keeping tenant code away from the registry is `allowedHosts`,
enforced by the wasmCloud host in the runtime. ADR-0017 has been corrected in place.

## Consequences

- **Slice 1's exit test (ADR-0011) is met for one strategy**: sign in, pick a component,
  save, get a URL that serves it. `fused` is proven live; `linked` is proven only in the
  renderer and the no-cluster e2e.
- **The registry is applied and PVC-backed**, 20Gi on `local-path`, no NodePort, holding
  `live/probe` from the run.
- **Bug 1 is a class, not an instance.** The failure was in the *contract between our
  pod and the operator's controller*, which no manifest golden test can see, because
  both sides are valid on their own. The renderer's tests now assert the label and the
  IP hostname, and that is the pattern to repeat: when the operator has to find
  something we created, assert on how it finds it.
- **What is still unproven live**: the `linked` strategy, the re-apply loop actually
  correcting drift, the push queue recovering from a registry wiped mid-flight, and a
  second *tenant* (as opposed to a second app).
- **Cold start, measured**: an app went from `save` to serving in roughly a minute, most
  of it the image pull for `wash` on first use per node. The `oci-cache` volume is
  per-pod, so component pulls are never warm; kubelet caches the container images, so
  the second app's pod started faster than the first.
- **The tenant's namespace outlives its apps**, which means a tenant with zero
  deployments still holds a namespace, a quota and a policy. Tenant teardown remains
  unimplemented (ADR-0016 scoped it out deliberately).

## Alternatives

- **Keep validating without deploying.** Rejected by this run's own results: three real
  bugs, one of which produced a healthy-looking app that answered nothing. Validation
  cannot see contracts between two systems that are each internally consistent.
- **Deploy the platform itself into the cluster first** (ADR-0011's dogfood milestone).
  Deferred: it would have added the platform's own hosting to the list of things that
  could be wrong, while the question here was whether the render→apply→run path works at
  all. It is the right next step now that it does.
- **Fix reachability by giving the Service a selector** instead of matching how the
  operator resolves hosts. Rejected: it would work by bypassing the operator's route
  controller, leaving us maintaining endpoints it also wants to manage, and would drift
  the moment a host moved.
