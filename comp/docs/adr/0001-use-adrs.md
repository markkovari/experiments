# ADR-0001 — Record architecture decisions as ADRs

- **Status:** accepted
- **Date:** 2026-07-27
- **Supersedes:** —

## Context

Everything in this repo so far has been decided in prose: `PLATFORM.md` holds a
five-phase plan, `ROADMAP.md` holds tiers, and each app's `*.md` argues its own
design. That works for one author on one machine, and it has already failed twice
in ways worth naming:

- Decisions that were *measured* got re-derived later because the measurement
  wasn't attached to a decision. The `wasm32-wasip2` question was answered three
  separate times in one week, twice from memory and wrongly.
- Decisions that were *reversed* left no trail. `host` was pinned to wasmtime 27
  for ~20 releases; nothing recorded whether that was deliberate or drift, so the
  bump had to re-establish why it was safe.

The platform work about to start is the first thing here with a shape that
outlives a single sitting: multi-tenant, cluster-side, with security boundaries
where "why is it like this" has consequences. That needs decisions that are
individually citable, dated, and reversible-with-a-record.

## Decision

Architecture decisions live in `docs/adr/NNNN-kebab-title.md`, numbered
monotonically, never renumbered. Each one is:

```
# ADR-NNNN — Title
- Status: proposed | accepted | superseded by ADR-MMMM | rejected
- Date: YYYY-MM-DD
- Supersedes: ADR-MMMM (or —)
- Evidence: what this decision KNOWS vs ASSUMES — see below

## Context      what forces the decision, including measurements and constraints
## Decision     the choice, in the active voice
## Consequences what this now obliges us to do, and what it costs
## Alternatives what was rejected and the specific reason
```

**The `Evidence` field** (added 2026-08-02, after
[ADR-0008](0008-isolation-is-stamped-never-authored.md),
[ADR-0012](0012-keyvalue-isolation-needs-a-cooperative-component.md) and
[ADR-0015](0015-a-bucket-name-is-not-a-boundary.md)). Every load-bearing belief in the
decision is listed with its status from [CLAIMS.md](../CLAIMS.md) — `MEASURED`,
`DOCUMENTED`, or `ASSUMED` — and any `DOCUMENTED` or `ASSUMED` claim names the test that
would settle it.

This exists because ADR-0008 *did* say its mechanism was unproven, in prose, in the
Context — and shipped a storage leak through two deploys and a green test suite anyway. A
sentence in the Context is not a blocker; a field that must be filled in is closer to one.
A `DOCUMENTED` claim is not safe by virtue of being documented: ADR-0015 killed a field
that was documented *and* required by its own docs.

Rules that keep them useful rather than ceremonial:

1. **One decision per ADR.** If the title needs "and", it's two ADRs.
2. **Never edit an accepted ADR's decision.** Supersede it with a new one and
   mark the old `superseded by`. The wrong answer stays readable — that's the
   point.
3. **Cite evidence inline.** A number in an ADR needs a source: a bench round, a
   command output, an upstream issue. "Faster" without a measurement is not a
   context.
4. **Record rejected alternatives with their reason**, not a list of names. The
   reason is what stops the option being re-proposed.
5. **ADRs decide; design docs describe.** `PLATFORM.md` stays the narrative plan
   and the phase order; ADRs own the forks inside it. Where they disagree, the
   ADR wins and `PLATFORM.md` gets updated.
6. **An ADR is judged against [PRINCIPLES.md](../PRINCIPLES.md).** Violating a
   principle is allowed; doing it silently is not — name the principle and say
   what the violation buys.
7. **A claim an ADR depends on goes in [CLAIMS.md](../CLAIMS.md)**, with its
   status. An `ASSUMED` claim cannot gate a release, and a `DOCUMENTED` one gets
   an adversarial test before anything depends on it.

An ADR may be written *before* the code (to settle a fork) or *after* (to record
one that got settled by discovery). Both are legitimate; the date says which.

## Consequences

- A new fork in the platform work means a new file, and the PR that implements it
  links the ADR. Cheap.
- `PLATFORM.md`'s "Isolation model (the core design decision)" section becomes a
  summary of ADR-0002/0008 rather than the authority.
- We now have somewhere to record the constraints that keep biting: no CAS in
  `wasi:keyvalue`, one environment per v2 host, wasmtime's ~30 nested-instance
  ceiling, anonymous p2 artifacts. Each belongs in the ADR whose decision it
  shapes, not in a folklore list.

## Alternatives

- **Keep deciding in the app docs.** Rejected: the platform's decisions cut
  across apps, and a decision buried in a narrative doc can't be cited from a PR
  or superseded cleanly.
- **A single DECISIONS.md log.** Rejected: it grows to one unreadable file, and
  "supersede without editing" is unenforceable in a single document.
- **GitHub issues as the record.** Rejected: the repo's design history should
  clone with the repo. Issues are for work, not for the reasons behind it.
