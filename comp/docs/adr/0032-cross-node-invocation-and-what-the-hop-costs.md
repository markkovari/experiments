# ADR-0032 — Cross-node invocation works, and the hop is nearly free

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

## What the hop actually costs

**An earlier version of this ADR said 50×. That was a bad measurement and the number was
wrong.** It is kept below as the third column, because the mistake is more instructive than
the correction.

Same graph, same load, three placements:

| | co-located | split, two local nodes | split, Mac ↔ Pi |
|---|---|---|---|
| throughput | 2,788 rps | **2,682 rps** | 55 rps |
| p50 | 1.39 ms | **1.43 ms** | 41.9 ms |
| p99 | 2.40 ms | **2.56 ms** | 613 ms |

**The wRPC hop costs about 4% of throughput and 0.04 ms of median latency.** Between
comparable nodes it is very nearly free, which is roughly what ADR-0019 implied when it
priced an in-process link at 1.2 ms saved per hop against wasmCloud's lattice.

The 55 rps column measures something else entirely. That node is a Raspberry Pi **whose own
store is on the other machine** — so every `records:store` call went Mac → Pi over the LAN,
and then Pi → Mac again for each NATS KV read and write. A double crossing, terminating in
the slowest hardware in the fleet, which ADR-0030 had already measured standalone at 78 rps.

Attributing that to the RPC layer was wrong. It measured [ADR-0030](0030-least-outstanding.md)'s
finding — that a node whose store is remote is dramatically slower — a second time, in a
place where the transport happened to be in frame.

## What that means for placement

Co-location stays the default, but for a smaller reason than "50×": it is simpler, it needs
no bus, and it keeps a graph's failure modes together. Spanning is now a real option rather
than a last resort — a GPU-pinned component or a jurisdiction-bound one costs a few percent,
not two orders of magnitude.

The thing that *is* expensive is unchanged and was already known: **putting a component far
from its store.** That is a storage decision, not an RPC one, and ADR-0027's shared-store
requirement is what forces the trade.

## How the wrong number happened

Worth writing down, because it is the fourth measurement in this line of work to read
convincingly and mean something else:

* two nodes both returning `4.0` looked like isolation working, and also meant state was
  per-node (ADR-0027);
* a token bucket refilling looked like data loss after a node died (it had not);
* three nodes balancing perfectly looked like a passing five-node test, with two nodes
  silently absent on a stale binary;
* and here, a split graph looked like an RPC cost when it was a storage cost.

The common shape: **the measurement had more than one variable in it, and the result was
attributed to the interesting one.** The fix each time was the same — isolate one variable
and re-run. Two local nodes was a five-minute test that should have come before the
cross-machine one, not after it.

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
