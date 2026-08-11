# 0067 — One copy is not a backup

Status: accepted, and built. Two of the four gaps in `CURRENT.md`'s durability
story; the other two (index writes, a repair pass) are named at the end.

## What was true before this

Every bucket this platform has ever created was made like this:

```rust
create_key_value(kv::Config { bucket: name, history: 1, ..Default::default() })
```

`num_replicas` defaults to 0, which JetStream reads as **one**, on `File` storage.
So one server's disk held the only copy of every tenant's data, and `history: 1`
meant there was not even a previous version of a key to go back to.

There was also no backup. Grepping the repo for backup, snapshot or restore
turned up `wasi_snapshot_preview1` and an ADR about inventory sizes.

[ADR-0035](0035-losing-a-machine.md) measured this fleet losing a **host** —
zero failed requests, replicas back in 16 seconds. That result is real and it is
about compute. Nothing had ever measured losing the **store**, and at one replica
the answer would have been: everything, permanently.

## Backup first, because it is the floor

`just backup` and `just restore`, over `nats stream backup` — the vendor's own
snapshot protocol, which streams a stream's messages *and* its configuration.
Writing our own would be re-implementing a wire format for nothing.

Verified the only way a backup can be: by destroying the data.

```
just backup                    KV_b-app-acme-shop -> backups/<utc>/
nats stream rm KV_b-app-…       deleted the bucket
nats kv get … order-1           nats: error: bucket not found
just restore                    KV_b-app-acme-shop
nats kv get … order-1           {"total":4200}
nats kv get … order-2           {"total":99}
just restore                    SKIP — it already exists
```

The refusal to overwrite an existing bucket is deliberate. Restoring over live
data is how a backup becomes an outage; an operator who means it can delete the
stream first and say so.

`REPLICAS=` on a restore overrides the copy count, which makes restore the way to
re-replicate a bucket that was created before any of this.

## Then replication

`--kv-replicas` (default 1) is passed to every bucket the host creates. Three is
the smallest number that survives a loss, because quorum needs a majority.

The default stays at 1 because 3 against a single-server NATS simply fails to
create buckets, and that would break every existing single-node deployment and
every test. Instead the host says so, once, loudly, at startup — the failure it
describes is total and silent until the day it happens.

## Measured: losing the server that holds the data

Three `nats-server`s clustered on one box — a real R3 cluster, same quorum code
as three machines — with `comp-host --kv-replicas 3` serving `gate-domain`'s rate
limiter, whose state is a counter in that bucket.

```
Replicas: 3    Leader: n1    Replica: n2, current    Replica: n3, current

before the kill:  remaining 93, 92, 91
                  *** killed n1, the stream leader ***
after the kill:   remaining 90, 89, 88, 87, 86

Leader: n3    Replica: n1, outdated, OFFLINE, 73 operations behind
```

Zero errors, and the counter kept counting down rather than resetting: the state
survived the loss of the server that led it. At one replica that same kill is
every tenant's data, gone.

Two things worth noting from the run. The host was given **one** NATS URL and
kept working after that server died — NATS clients learn the cluster from the
server's own INFO and fail over. That is a nice property to have and a bad one to
depend on: a host started while its only listed server is down cannot bootstrap,
so a real deployment lists all three.

And a cluster of three processes on one machine proves the *code*, not the
*hardware*. It exercises the same quorum, replication and election paths; what it
cannot show is a disk dying, a power loss, or a network partition between rooms.
For that the three copies have to be on three machines — which this fleet has.

## Still open, and both are `comp:store/cas` work

- **Index maintenance is unguarded.** `record-store`'s `ids_insert` is a
  read-modify-write over the chunked id list. Because `list`, `count` and `query`
  page over `idx_{collection}`, an id lost there makes a record that still exists
  **invisible** — indistinguishable from data loss, and silent.
  [ADR-0066](0066-the-guard-moves-into-the-store.md) built the primitive that
  fixes this and pointed it only at the record.
- **Nothing repairs an inconsistency.** A record and its indexes are separate
  writes; a crash between them leaves them disagreeing, with no way to notice or
  fix it. The records are authoritative, so a rebuild is mechanical — it just does
  not exist.
