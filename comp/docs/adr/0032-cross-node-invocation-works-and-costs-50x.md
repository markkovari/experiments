# ADR-0032 — Cross-node invocation works, and costs 50×

- **Status:** accepted
- **Date:** 2026-08-08
- **Completes:** [ADR-0028](0028-cross-node-calls-are-wrpc.md), which chose wRPC and wired only the caller
- **Settles:** the deferral in [ADR-0025](0025-slice-one-on-the-lattice.md)

## It works

One application, graph split across two machines and two architectures:

```
MacBook   gate         (role=web)   imports records:store/store, shaper:limit/limiter
          shaper                    exports shaper:limit/limiter
Pi 5      record-store (role=data)  exports records:store/store   <- the other machine

gate bound 2 interface(s) over wrpc
record-store serves 8 function(s) to the lattice
shaper serves 2 function(s) to the lattice

HTTP to gate on the Mac:
  remaining 4.000   801.6 ms   (cold: compile + connect)
  remaining 3.790    67.2 ms
  remaining 2.857    39.7 ms
  remaining 1.897    40.8 ms
```

The counter advancing is the proof: rate-limit records can only be written and read through
`record-store`, which is on the Pi. A component called a component on another machine, and
neither knew — the import was bound in the linker, so the guest sees a function call.

## And it costs 50×

Same graph, same load, only placement changed:

| | co-located | split across machines |
|---|---|---|
| throughput | 2,788 rps | **55 rps** |
| p50 | 1.39 ms | **41.9 ms** |
| p99 | 2.40 ms | **613 ms** |

**Co-location stays the default and that is not a detail.** A `linked` graph co-locates
unless a component is given placement of its own; spanning is opted into, never fallen
into. The right reasons to span are the ones a 50× tax cannot outweigh — a GPU only one
node has, data that must not leave a jurisdiction — and never "it seemed tidier".

The tail is worse than the median suggests (613 ms p99 against 41.9 ms p50). That is a LAN
hop to a Pi whose own store is remote (ADR-0030 measured that separately at 78 rps), so
this figure is close to a worst case rather than a typical one. It has not been measured
between two comparable machines, and it should be before anyone quotes it.

## The design, in one line each

- **Serve side**: one subscription per exported function, on the instance's own subject
  prefix, in a **queue group named for the instance** — so N replicas of a component share
  invocations and a departing one needs no deregistration. Exports are read from the
  component's own type, never a manifest that could drift from it.
- **Call side**: at start, a link table splits into local and remote; only the remote half
  becomes wRPC clients, keyed by interface so one store reaches many targets.
- **A plug is now a first-class instance.** `Instance.pre` became `Option<ProxyPre>`:
  a component with no `wasi:http/incoming-handler` used to be unable to start at all, and
  now starts, serves its exports, and simply never appears in the route table.
- **A served invocation gets the same `Store` an HTTP request gets** — same scope, memory
  cap, CPU slice and egress allow-list. `store_for` is one function precisely so there is
  no second construction path for one of those to be forgotten from, which is what
  ADR-0023 is about.
- **Placement**: a component with its own constraints is placed independently; one without
  rides along with the root.

## Known wrong: local/remote is decided once, at start

The co-located measurement above still shows **one component bound over wRPC**. The reason
is start order: `gate` started before `shaper`, so when `gate`'s links were resolved the
local instance table did not yet contain `shaper`, and it was treated as remote. It then
talks to a component in its own process over the loopback bus.

Correct but slow — the co-located number above is therefore *pessimistic*, and true
all-local is faster still. Two fixes, neither done:

1. Re-resolve an instance's links when the local table changes. Precise, and it means
   rebuilding a linker for a running instance.
2. Have the reconciler order a graph's starts plugs-first. Cheaper, and only narrows the
   window rather than closing it — a plug on another node still arrives whenever it does.

## Still not carried: resources

`link_instance` is given empty resource maps. wRPC encodes a resource as opaque bytes whose
meaning is application-specific, and nothing here defines that meaning, so an interface
passing one must be refused at placement rather than handed a blob the far side cannot
read. **Nothing classifies interfaces yet**, so that refusal does not exist — a graph
spanning a resource-bearing interface will fail at first call rather than at deploy. That is
the next thing to build, and it is the last claim in ADR-0028's list still outstanding.
