# e2e — six manifests, one fleet, real requests

```
SP=/tmp/e2e bash e2e/run.sh
```

Everything in `bench/` measures one property at a time on a stack built for that
measurement. This one deploys **six apps to one fleet at once** and asserts what each
should do. Takes about a minute; meant to be run before pushing.

| fixture | strategy | must |
|---|---|---|
| `fused` | fused | serve — a composed artifact over HTTP |
| `linked` | linked | serve — `gate`'s two imports bound to `record-store` and `shaper` at runtime |
| `zero` | fused, `min: 0` | serve — activated by the request itself (ADR-0042) |
| `conflict` | linked | be refused — two providers of one interface |
| `ungrantable` | fused | be refused — a capability no host grants |
| `unplaceable` | fused | be refused — a constraint no node advertises |

## Why it is shaped like this

**Positives are checked by invoking them.** An app that is placed but does not answer
is exactly the failure a status check misses — inventory would show it running.

**Negatives are checked by their reason, not just by failing.** Asserting "it was
refused" passes for a refusal with the wrong reason, and a reason nobody can act on is
barely better than a crash. Each expects specific substrings: `conflict` must name the
interface *and both providers*, `unplaceable` must name the constraint it could not
meet.

**They deploy together on purpose.** Three broken manifests alongside three healthy
ones is the assertion that a bad manifest cannot stop good apps from being placed —
a property no single-app test can show, and the kind of thing that breaks when the
planner grows a new early return.

**It polls rather than sleeping on a fixed number.** Inventory is a heartbeat behind
reality and a cold app has to be activated first; a test that asserts on a snapshot
taken at the wrong moment fails on a working system. That mistake has been made twice
in this repo (ADR-0042, ADR-0045) and cost more than the polling does.

## Adding a fixture

Edit `fixtures.json`, then add the app to either `SERVES` or `REFUSED` in `check.py`.
`REFUSED` entries carry the substrings the reason must contain — write the ones an
operator would need to fix it, not the ones that happen to be in the string today.
