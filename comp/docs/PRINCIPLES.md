# Principles

Seven. They are the standard an ADR is judged against, not decisions themselves — an ADR
picks a mechanism, a principle says what makes a mechanism acceptable.

**What these are not.** They will not stop the next ADR-0012. Every expensive mistake in
this repo so far was a *false belief about a mechanism*, not a compromised principle —
[ADR-0008](adr/0008-isolation-is-stamped-never-authored.md) had the right principle and
shipped a leak anyway. The defence against being wrong about facts is
[CLAIMS.md](CLAIMS.md), not this page. These guard against drift: shipping something that
contradicts what the project is, under deadline pressure, one small compromise at a time.

Violating one is allowed. Doing it silently is not — a violation is an ADR that names the
principle and says what is bought.

---

### P1 — Isolation is enforced by a mechanism the guest cannot address

Not by convention, not by a field nothing reads, not by asking components to cooperate. If
a boundary can be crossed by a guest naming something, it is not a boundary.

> A field that nothing reads but looks like a boundary is worse than an absent one — it is
> the reason this shipped through two deploys and a passing test suite.
> — [ADR-0012](adr/0012-keyvalue-isolation-needs-a-cooperative-component.md)

**Test:** name the thing that would have to be compromised for the boundary to fail. If the
answer is "the tenant would have to not do X", it fails.

### P2 — A number is measured, or it is marked unmeasured

No projections quoted as facts, no extrapolations without the word, no idle figures passed
off as load figures. Cold-start and settled are different numbers and are labelled.

**Test:** every quotable figure traces to an ADR or a bench run. Everything else carries a
marker.

### P3 — A gate is code, not prose

A rule that must not be violated is enforced by something that fails. A release gate in a
document is a suggestion.

> ADR-0008's gate was prose; prose does not stop a deploy.
> — [ADR-0012](adr/0012-keyvalue-isolation-needs-a-cooperative-component.md)

**Test:** try to violate it. If you succeed and nothing fails, it is not a gate.

### P4 — The contract is the product

WIT first. Implementations, hosts, and backends are swappable behind it; the contract is
what is promised. A capability that cannot be expressed as a contract is not a platform
feature yet.

**Test:** could this be reimplemented by someone else against the same WIT and still work?

### P5 — The tenant sees the real cost

Including the floor, including what is amortised, including what a plan does not cover.
A cost hidden in per-request pricing is a cost the tenant cannot make decisions about.

> A per-app 70 Mi floor should be visible to the tenant, because it is most of the cost of
> a small app and it does not shrink. Metering that is honest; hiding it is not.
> — [ADR-0019](adr/0019-the-density-number.md)

**Test:** can a tenant predict their bill from their own usage and the published model?

### P6 — Say what this is bad at

In the docs, in the pitch, at admission time. A one-component app should be a container and
the product says so. Turning away a bad fit is cheaper than acquiring it and losing it.

**Test:** does the public description of this platform contain the workload it loses on?

### P7 — Being wrong is cheap; staying wrong is not

Record the decision, measure it, and supersede it when the measurement disagrees. A
falsified ADR is kept with its reasoning intact, marked, and linked from what replaced it —
never quietly edited into correctness.

**Test:** can a reader tell what was believed, when, and why it changed?

---

## Applying these

- An ADR that touches isolation cites P1 and names the mechanism.
- An ADR quoting a number cites P2 and its source.
- An ADR that sets a rule says how P3 enforces it.
- A principle that keeps getting violated for good reasons is a wrong principle — change it
  by ADR rather than by erosion.
