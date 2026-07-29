# Self-hosting, in three tiers

For running **your own** apps on **your own** machines. Every number here is measured
(see [ADR-0019](adr/0019-the-density-number.md), [ADR-0020](adr/0020-the-density-number-under-load.md));
every limitation is one that was hit rather than guessed.

The tiers are deliberately progressive: **one app spec, three backends.** `apps/<name>.toml`
is the only hand-authored file, and moving up a tier is an edit, not a rewrite — because
each tier is the same shape of thing, a pure function from that spec to whatever the
substrate needs. That is the property that made the Kubernetes renderer reliable, so tier 1
copies it.

| | scheduling | per component boundary | control plane per box |
|---|---|---|---|
| **tier 1** — `comp-host` + systemd + Caddy | you pick the box | **0** (fused, in-process) | **none** |
| **tier 2** — many apps per host | you pick the box | 0 (in-process) | ~200 Mi |
| **tier 3** — k3s + wasmCloud operator | declarative, cross-machine | 0 (in-process) | ~800 Mi – 1 GB |

Start at tier 1. Go up when a measurement tells you to, not before.

---

## Tier 1 — `comp-host` + systemd, one URL per app

**This is built and it works today.** One app, one process, one hostname.

```bash
just compose-gate                    # components -> one .wasm
just selfhost-render gate            # see the unit, env file and route
just selfhost-deploy gate my-vps     # ship it
just selfhost-status gate my-vps
```

The spec:

```toml
name = "gate"
domain = "gate.example.com"
artifact = "components/target/gate_domain.composed.wasm"
kv = "memory"                        # or redis / nats
components = ["gate-domain", "record-store", "shaper"]   # tier 3 reads these
strategy = "fused"
[config]                             # KEEP TABLES LAST (TOML: later keys join the table)
grace-period-secs = "5"
```

From that, `selfhost` renders three files: a hardened systemd unit, a `CFG_*` environment
file, and a Caddy site (or `--router traefik`) so the app gets its own URL with automatic
TLS.

**Per-app URLs.** Every app binds `127.0.0.1:<port>` and nothing else — the unit is tested
to never emit `0.0.0.0`. The proxy is the only public listener, it routes by hostname, and
it obtains certificates itself. Ports are derived from the app name, *stably*, so a
re-render never moves a running app out from under its route; `just selfhost-check` refuses
two apps landing on the same port, domain or name, which is the one collision a single spec
cannot see.

**Why nothing collides.** Nothing is shared:

| | isolated by |
|---|---|
| port | its own loopback port; the proxy routes by hostname |
| keyvalue | its own process (`memory`), or a Redis DB / NATS bucket per app |
| config | its own `EnvironmentFile` |
| state on disk | `StateDirectory=comp/<app>` — a private `/var/lib` path per unit |
| crashes, logs | its own unit; `Restart=always`, journald per app |

That is isolation by process and filesystem, which is what Unix has always done. The
platform's per-app hosts and private buses exist to defend *strangers* from each other; on
your own box you do not need them.

**Hardening**, because this serves the internet: `DynamicUser` (a transient uid per app),
`ProtectSystem=strict`, `ProtectHome`, `NoNewPrivileges`, `PrivateTmp`, `PrivateDevices`,
`RestrictNamespaces`, `RestrictAddressFamilies`, `LockPersonality`. The one thing that
cannot be tightened is `MemoryDenyWriteExecute` — wasmtime JITs, so it needs W^X, and the
unit says so where a reader will find it.

**What tier 1 gives up:** many apps in one process. Each app costs a `comp-host` — about
**70 Mi idle, ~230 Mi once it has served traffic**. Five apps is fine on a 2 GB box; twenty
is not. `fused` still packs as many *components* as you like into one app at **2.3 Mi
each**, so you are not giving up the component model, only the sharing of one runtime
between apps.

### The open question, and it is real

`kv = "memory"` **loses all state when the process restarts** — and `Restart=always` means
restarts happen. So today tier 1 is honest only for stateless apps, or with `kv = "redis"`
pointed at a Redis you run.

Three ways out, none built yet:

1. **A sqlite backend for `comp-host`.** One file per app under `StateDirectory`, no daemon,
   survives restarts. Cheapest and best fit for a VPS. **Recommended.**
2. **One Redis per box**, a DB number per app. Works now, costs a daemon (~10 Mi) and gives
   you an eviction policy you must think about.
3. **`wasmcloud:postgres`.** `comp-host` has no postgres plugin, but the wasmCloud host
   does (`wash host --postgres-url`) — worth knowing for tier 2/3, and it would want
   components written against that interface rather than `wasi:keyvalue`.

---

## Tier 2 — many apps per host

**Not built, and the design is decided.** When per-app processes cost too much RAM, put
many apps in one runtime: **~70 Mi once, then 2.3 Mi per component**, measured, with
identical throughput and a *better* p99 than separate processes (ADR-0020).

The blocker is not the runtime, it is naming: every storage component in this catalog
hardcodes `open("default")`, so apps sharing one host would share one bucket. ADR-0012
rejected fixing that by convention **because tenant code cannot be trusted to honour it**.
Your own code can. So tier 2 is:

1. make `record-store` and its siblings read their bucket from `wasi:config` (defaulting to
   `"default"`, so nothing existing breaks);
2. give each app its own bucket name in its spec;
3. run one runtime per box with the apps linked into it.

Note the honest caveat: **one crash takes every app on that box**, and blast radius is the
thing tier 1 buys you. That is the trade, and it is why RAM pressure — not neatness —
should be what moves you.

*A correction worth recording: this cannot be done with a v2 `wash host` alone. There is no
way to place a v2 workload without the Kubernetes operator — wash 2.x has no `app`, `start`
or `link` commands, and `--user-config` is a settings file, not a workload spec. Tier 2
therefore means either `comp-host` learning to serve several artifacts behind a router
component, or accepting tier 3.*

---

## Tier 3 — k3s + the wasmCloud operator

**Built, and proven live** ([ADR-0018](adr/0018-the-platform-deploys-a-running-app.md)):
upload → push → deploy → serve, both strategies, per-app isolation, delete, drift
correction.

What it buys: **declarative placement across machines** and automatic rescheduling when a
box dies. What it costs, measured on a running cluster before any of your apps exist:

```
wasmCloud stack     320 Mi   host 170, operator 59, gateway 34, nats 32, registry 10
k8s control plane   ~500 Mi+ (apiserver, etcd, scheduler — k3s on a real VPS)
                   --------
                   ~800 Mi – 1 GB per cluster
```

Worth it when you have enough machines that deciding *where* an app runs is a chore. With
two or three, that decision is one argument to a deploy recipe.

**Do not use wadm for this.** `infra/wadm.yaml` in this repo is the v1 OAM lane, driven by
`wash app put` — a command wash 2.x removed. More importantly, v1 links components over
NATS/wrpc, so **every component boundary becomes a network hop** (measured: 1.2 ms), which
forfeits the one advantage the whole approach is for. Tier 3 is the v2 operator, not wadm.

---

## Where each piece lives

| | |
|---|---|
| `apps/<name>.toml` | the app spec — the only file you write |
| `selfhost/` | tier-1 renderer, pure and tested (10 tests, incl. one that checks the flags it emits actually exist on `comp-host`) |
| `host/` | `comp-host` — the runtime for tiers 1 and 2 |
| `components/platform-domain/src/render.rs` | the tier-3 renderer |
| `applier/` | tier-3 apply, reconcile, prune, registry push |
