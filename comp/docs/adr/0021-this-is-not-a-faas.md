# ADR-0021 — This is not a FaaS, and the difference is the product

- **Status:** accepted
- **Date:** 2026-08-02
- **Decides:** what category this platform is sold as, and what workload it turns away
- **Builds on:** [ADR-0019](0019-the-density-number.md), [ADR-0020](0020-the-density-number-under-load.md)
- **Relates to:** [REQUIREMENTS-DENSITY.md](../REQUIREMENTS-DENSITY.md), [SCENARIOS-ELASTICITY.md](../SCENARIOS-ELASTICITY.md)

## Context

The elasticity design in [SCENARIOS-ELASTICITY.md](../SCENARIOS-ELASTICITY.md) — on-demand
instantiation, warm floors, scale-to-floor on idle, per-invocation metering, event-driven
consumers, multi-tenant packing — reads as a FaaS. The observation is fair: **mechanically
it is FaaS machinery.** Someone will therefore propose selling it as one, and that
proposal needs an answer before a landing page exists rather than after.

The answer matters commercially, not just taxonomically. This project has already been
burned once by a category error: `PLATFORM.md` was priced on multi-tenant density that
[ADR-0012](0012-keyvalue-isolation-needs-a-cooperative-component.md) and
[ADR-0014](0014-an-application-owns-a-host.md) falsified, and ADR-0019 had to reposition
the whole product around what was actually measured. Choosing "FaaS" would repeat the
mistake in the opposite direction — adopting a category whose defining constraint is the
one thing this platform is built to remove.

## The structural mismatch

**A FaaS's defining unit is one function.** Lambda gives you a handler; Workers gives you a
script. Decomposition therefore means N functions, N cold starts, and a network hop
between each pair.

The measured advantage here is precisely the inverse
([ADR-0019](0019-the-density-number.md)):

> A four component app linked in-process avoids three hops: **~3.6 ms per request and
> ~210 Mi.**

`vet-domain` is 16 components and 104 core modules in one deployment unit with zero
internal hops. Expressed as functions that is 16 units and 15 hops. **The workload this
platform is best at is the one a FaaS structurally cannot express.**

Three further mismatches, each independently sufficient:

| axis | FaaS | here |
|---|---|---|
| unit of deployment | a function | an application of many components ([ADR-0014](0014-an-application-owns-a-host.md)) |
| state | stateless by doctrine; state is someone else's service | first-class, with a per-application boundary (FR-2) |
| composition | over the network, between units | in-process, at link or fuse time ([ADR-0005](0005-deployment-strategy-is-a-tenant-choice.md)) |
| billing/isolation/failure unit | the invocation | the application, consistently across all four |

And the conclusion ADR-0019 already reached, which a FaaS pitch directly contradicts:

> **A one-component app has no business on this platform** — it should be a container.

A FaaS pitch attracts exactly that workload: single small functions, where this loses to
Lambda on price, on ecosystem, and on cold start.

## Decision

**The category is a component PaaS with FaaS-grade elasticity. Not a FaaS.**

- **The pitch stays what [WHY.md](../WHY.md) already says**: decompose as far as the design
  wants and pay almost nothing for the split. Elasticity is a *property* of the platform,
  not the product's identity.
- **The comparison set is not Lambda.** Against Fly.io or Render (container per app, a
  language-runtime floor each) the claim is 2.3 Mi per component. Against vanilla
  wasmCloud the claim is tenancy, metering and placement. Against Lambda or Workers there
  is no claim worth making for a single function, and pretending otherwise invites a
  comparison this platform loses.
- **The closest real analogue is Cloudflare Workers + Durable Objects + Queues** — three
  products with separate boundaries and separate billing. The distinguishing claim here is
  that it is *one* composition model, not that any individual piece is better.
- **Turning away single-component apps is a feature, and it is said out loud.** ADR-0019
  established this; this ADR keeps it in the marketing, not only in the docs.

### The FaaS lane, deliberately scoped

**A single-exported-function contract is worth building as an on-ramp, and only as an
on-ramp.** A WIT-defined function world where the platform supplies HTTP and lifecycle is
cheap — composition already does the wiring — and it lets a user arrive with one function,
discover that decomposition costs nothing, and grow into the shape this platform is good
at.

It is an acquisition path, not a category. If the FaaS lane ever becomes the majority of
deployed workloads, the density argument has stopped applying to the actual user base and
the positioning needs revisiting — that is the signal to watch, and it is a real one.

## Consequences

- **Marketing does not use the word FaaS**, except to say what this is not. The elasticity
  mechanisms are described as what they are; the category label is not borrowed.
- **The catalog is the proof, not a bonus.** 109 components exist to make decomposition the
  obvious move. In a FaaS framing they would read as 109 things to deploy separately.
- **Single-component deployments should be flagged at admission**, pointing at the honest
  advice that a container is cheaper. Refusing them is too strong — a one-component app is
  often the first step toward a ten-component one, which is exactly the FaaS-lane on-ramp
  above.
- **Elasticity still has to be genuinely good.** Rejecting the label does not lower the
  bar: [SCENARIOS-ELASTICITY.md](../SCENARIOS-ELASTICITY.md)'s R1–R10 and its unproven list
  (U1–U6) stand regardless of what the category is called.
- **This ADR is positioning, not architecture.** No code changes. If it is wrong, the cost
  is a landing page, which is the cheapest thing here to change — unlike the density bet,
  where being wrong cost a rebuild.

## Alternatives

- **Sell it as a FaaS.** Larger, better-understood market with existing buyer intent.
  Rejected: it attracts the workload with the worst unit economics here, competes
  head-on with incumbents on their strongest axis, and buries the measured advantage.
- **Sell it as a wasm PaaS.** Honest but leads with the implementation. Nobody's budget
  line item is "wasm". The decomposition economics are what a buyer recognises.
- **Sell it as microservices infrastructure.** Closest to the truth and the most
  poisoned term in the vocabulary — it presumes the network boundary this platform
  removes.
- **Name no category.** Tempting given that two category claims have now been revised, but
  a product with no category gets assigned one by its readers, and they will pick FaaS.
