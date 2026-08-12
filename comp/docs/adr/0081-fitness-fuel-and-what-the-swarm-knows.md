# ADR-0081 — fitness, fuel, and what the swarm knows

*Three mechanisms a parallel agentic graph cannot run without: how a branch is
judged, how branches share what they learn, and how they are stopped.*

**Status: proposed.** Nothing here is built. It is written before the code
because all three decisions are expensive to reverse once agents depend on them,
and because two of the three have a version that looks obviously right and is
wrong.

Much of this is taken from reading alpha-swarm2 rather than from first
principles. Where it got something right, this says so and copies it. Where it
built a knob that does nothing, this says that too — because that failure mode is
the one worth designing against.

## Where this starts from

Already built and proven here:

| piece | what it gives this |
|---|---|
| environments (ADR-0078/0079) | a branch is a derived app with its own store and its own lineage |
| `knowledge:graph` (ADR-0080) | nodes, edges, traversal against SurrealDB |
| `comp:store/cas` (ADR-0065) | atomic compare-and-set — the only correct way to decrement a shared counter |
| `llm:inference` | `completion.usage` already reports prompt and completion tokens |
| `quota:meter` | period-based rate limiting, keyed by subject string |
| `lattice/src/lease.rs` | a lease whose TTL *is* the KV `max_age` — a dead holder frees its own slot |
| secrets + signing | a branch runs as an identity, and what it publishes is attributable |

Two things that look like they already solve this and do not:

- **`quota:meter` is not a budget.** It is a rate limit: subject, limit, period,
  reset. No parent granting to a child, no unspent allowance returning, no
  conservation. Used as fuel it gives every node an independent allowance, which
  is a licence to spend `nodes × limit`.
- **Nothing meters inference.** `completion.usage` returns token counts and no
  component reads them. The number this whole system needs is already arriving
  over the wire and being dropped on the floor.

## The lesson that shapes everything below

alpha-swarm2 has, in its shipped code:

- an agent and a worker tier, each with `time_limit_secs`, `token_limit` and
  `max_iterations` in config — **and no code path that reads them.** Only the
  orchestrator tier's fuel is enforced.
- `max_sub_plan_depth`, passed into `SwarmRunner::with_depth`, where `self.depth`
  is *assigned and never read*. Recursive sub-planning is not implemented.
- a `quality_passed` in the wave runner that is a stub evaluating to `true`, with
  a `// TODO` beside it. The real gate lives elsewhere; anything trusting that
  field is trusting a constant.
- a daily run cap that `continuous = true` bypasses entirely.

None of these are bugs of carelessness. They are what happens when a limit is
declared in configuration before anything enforces it: the config file reads like
a system with budgets, and the running system has one budget. This repo has been
bitten by the identical shape twice already — `comp:secrets/reader` shipped
unlinked, and `openai-provider` was never once called.

So the rule for this ADR, applied to every mechanism in it:

> **A limit that nothing enforces is documentation. Every limit here ships with a
> test that spends past it and watches it refuse — or it does not ship.**

---

## 1. Fitness: how a branch is judged

### The failure mode to design against

The tempting design is one scalar from an LLM judge: ask a model how good the
solution is, take the number, select on it. It fails specifically: **the loop's
stopping condition becomes non-reproducible.** Same graph, same inputs, two runs,
two different stopping points, and "why did it stop here" has no answer that
survives a re-run. Worse, selecting on a model's judgement is a closed loop — the
population evolves toward what the judge likes, which is not the goal.

### The decision: a hard gate, a soft score, and a veto

alpha-swarm2 gets this right and it is worth stating exactly how, because the
shape is not obvious:

- **The gate is code.** `run_quality_gate` materialises the changed files into a
  throwaway git worktree and runs `cargo check -p <pkg>`, then `cargo test -p
  <pkg>`, then a security scan over added lines. `Passed` iff that returns `Ok`.
  A model is not consulted.
- **The LLM judge is a veto, not a gate.** `adversarial_verify` returns
  `Accept | Reject(reason)`, and it can only **downgrade** Passed → Failed. It
  can never promote a failing candidate. An inference error defaults to `Accept`
  — the judge being unavailable must not become a source of rejections.
- **The evidence bar is "something changed".** A run counts only if
  `modified_files` is non-empty; `tasks_passed > 0` is explicitly rejected as
  evidence of work, and Cargo.lock churn without a Cargo.toml change is stripped.

That third point is the one most likely to be skipped and the one that catches
the most embarrassing failure: an agent that reports success having done nothing.

So, three tiers rather than two:

1. **Gate** — deterministic, reproducible, binary. The only thing that may end a
   branch *successfully*.
2. **Veto** — may reject what the gate passed. May be a model. Fails open.
3. **Score** — ranks candidates that already passed both, to decide where the
   next fuel goes. May be fuzzy; being wrong costs efficiency, not correctness.

### What comp adds: selection

alpha-swarm2 has **no tournament, no best-of-N, and no numeric fitness at all** —
`quality_gate_passed` is an `Option<bool>`. Its parallel tasks are different
sub-tasks of one DAG that get *merged*, not competing candidates that get
*selected between*. Its only "which is better" mechanism is a UCB1 bandit
choosing a planner tier, attributed on the real gate verdict.

That is the gap comp's environments exist to fill: a fork is a competing
candidate. Which means comp needs the thing alpha-swarm2 does not have — a score,
so that two branches that both pass the gate can be ordered.

    package graph:fitness@0.1.0;

    interface evaluator {
        record verdict {
            /// The GATE. Deterministic. The only thing that may end a branch
            /// successfully.
            accepted: bool,
            /// The SCORE, 0..=1000 milli-units. Meaningful only AMONG accepted
            /// candidates; orders where to spend next.
            score: u32,
            /// Why — for the graph, and for a person reading it later.
            reason: string,
            /// What was measured: tests-passed, diff-lines, wall-ms. Named, so a
            /// later generation can compare like with like.
            measures: list<tuple<string, string>>,
        }

        /// This evaluator's identity and version. A score from evaluator A is not
        /// comparable to one from B, and a graph that mixes them silently has
        /// selection pressure made of noise.
        describe: func() -> tuple<string, bool>;

        evaluate: func(candidate: string, context: string) -> result<verdict, string>;
    }

An evaluator is a **component**: swappable per graph, sandboxed, and grantable
egress to a CI endpoint without being granted anything else.

Every verdict is a node in the knowledge graph, related to the attempt that
produced it, stamped with the evaluator's id and version. That is what makes "did
generation 4 actually beat generation 3" answerable rather than remembered.

### Completion: five endings, because the parent's next move differs

1. **`accepted`** — gate passed, veto did not fire. Stop; promote what it learned.
2. **`exhausted`** — fuel ran out. A good answer may be one step away, and the
   parent may refuel it. Recorded distinctly from failure, because a parent that
   cannot tell "wrong" from "ran out of money" abandons branches that were
   working.
3. **`plateaued`** — K consecutive generations with no score gain beyond ε. The
   honest end of most searches. K ≥ 2: one barren round is noise.
4. **`refuted`** — the branch proved its own approach cannot work. **After
   acceptance this is the most valuable outcome**, and the one a naive design
   discards; it stops every sibling re-walking the dead end. alpha-swarm2 keeps
   these in an `errors` namespace and injects them as "RECENT FAILED ATTEMPTS (do
   NOT repeat these approaches)".
5. **`abandoned`** — a human, or a parent reallocating.

The graph as a whole terminates when: the gate passes, or fuel is exhausted, or
every live branch has plateaued. All three are checkable without asking a model.

---

## 2. Knowledge: what the swarm shares

alpha-swarm2's design here is better than anything I would have proposed cold,
and most of this section is taken from it.

### The load-bearing idea: agents cannot write the trusted namespace

Five namespaces — `patterns`, `solutions`, `errors`, `trajectories`, `feedback` —
and the agent-facing tool permits writes to **only `solutions` and `errors`**.
`patterns` is written by the system, on a hook, and only after the quality gate
passed: the verified diff is distilled by a second, cheap LLM call into ≤900
characters and stored under a key that is the SHA-256 of the normalised goal.

That is the whole answer to knowledge poisoning, and it is worth naming as a
principle:

> **An agent may record what it observed. Only a passing gate may promote
> something to what the swarm believes.**

Raw model output never reaches the trusted pool. Everything in `patterns` is
downstream of a `cargo test` that actually passed.

### Scope: shared by project, not by agent

Knowledge is keyed by `(namespace, project)` — **not** per-agent and **not**
per-run. Every agent on a project reads and writes one pool. This is the right
default: isolation per branch would defeat the entire purpose, which is that
sibling 7 does not repeat sibling 3's mistake.

The exception is instructive: **error entries are keyed per-run on purpose**, so
that the UPSERT does not collapse failure history into one row. Successes should
converge; failures should accumulate.

For comp this maps onto environments cleanly:

- one SurrealDB **database per project**, shared — matching alpha-swarm2;
- writes carry the environment and attempt that produced them as provenance;
- promotion to the trusted namespace happens only on a gate pass, performed by
  the platform rather than by the branch.

Copy-on-fork was my first instinct and it is wrong. It gets isolation nobody
asked for and loses the only property that makes a swarm better than one agent
run N times.

### Confidence is derived from outcomes, never asserted

There is **no confidence field** on an entry. Instead a `pattern_effectiveness`
table records `(pattern_id, run_id, run_succeeded)` for every pattern that was
injected into a run, and retrieval reranks by it:

    similarity *= 0.5 + 0.5 * success_weight        // floor 0.5, neutral 0.75

A pattern that keeps being present when runs fail sinks. Nothing has to decide
how confident it is; the outcomes decide. **Copy this exactly.** A
self-reported confidence score is a number an agent optimises against.

### Dedup, decay, and travelling with the repo

- **Dedup** by UPSERT on `(namespace, project, key)`, key = hash of the
  normalised goal. Re-learning the same thing reinforces one row rather than
  growing the pool.
- **Skip duplicate work** entirely: `task_already_done` returns a past passing run
  above cosine 0.9 and the task is skipped.
- **Decay**: entries with `use_count < 2` last used over 30 days ago are deleted,
  plus a TTL sweep. Note the honest gap found while reading — *nothing schedules
  `decay`*; it is exposed but not driven by a loop. Another declared-but-not-run
  mechanism, and comp should schedule it or not claim it.
- **Export to the repo**: `.swarm/memory/patterns/<key>.md` and `errors/<key>.md`
  as markdown with frontmatter, plus `KNOWLEDGE.md`. Embeddings are never
  committed and are recomputed on import. This is a genuinely good idea — the
  knowledge is reviewable in a pull request, and a human can delete a pattern
  they disagree with.

### Retrieval is three layers, not one

1. **Semantic** — hybrid dense HNSW + BM25-lite, fused with reciprocal rank
   fusion; the reported similarity stays the dense cosine so a `min_similarity`
   threshold keeps meaning something. Positive guidance from
   `patterns`+`solutions`, negative from `errors`, with a character budget
   (1200) on what reaches the prompt.
2. **Co-edit statistics** — files historically changed together, min
   co-occurrence 2. Pure counting, no model.
3. **Code-graph traversal** — goal-named files → entities → 1 hop over
   `defines | implements | extends | imports` → structurally related files.

Only the third needs `knowledge:graph`. The first needs embeddings, which comp
does not have wired: `llm:inference/embed` exists and, like everything else in
that stack, has never been called. **That is the dependency to close first** —
without retrieval, a knowledge store is a write-only log.

---

## 3. Fuel: budgeting with propagation

alpha-swarm2 has fuel in three dimensions — time, tokens, iterations — checked in
that order at the top of the retry loop, with exponential backoff between
attempts. It has **no parent→child propagation** (tokens aggregate *upward*,
never decrement downward), **no per-node budget**, and **no monetary cost at all**
— zero occurrences of price or cost across the codebase. And, as noted, it
enforces only the orchestrator tier.

So propagation is the part comp has to design rather than copy.

### The invariant everything follows from

> **Fuel is conserved. A node cannot mint it. Σ(live balances) + spent + refunded
> = the original grant, at every instant.**

Stated first because it is the only property here that can be *tested*, and
because every plausible budget design lacking it leaks. Without conservation,
"this run has a budget of N" is a hope; with it, it is arithmetic, and a test can
assert it after an arbitrary interleaving of spawns, spends, crashes and refunds.

### Mechanics

- A run begins with one grant, held by the root.
- **Spawning transfers.** A parent moves fuel from its own balance into the
  child's. It cannot create fuel; with none, it cannot spawn. Depth becomes
  self-limiting — though a depth cap exists anyway, because splitting into
  thousands of one-unit children is conservative and useless. (And a depth cap
  that is assigned and never read is alpha-swarm2's bug, so it gets a test.)
- **Spend reserves first, settles after.** Reserve the estimate, do the work,
  settle the actual, return the difference. `quota:meter` already has this shape;
  it is the *hierarchy* that is missing, not the operation.
- **Death refunds.** Any of the five endings returns the remaining balance to the
  parent. A crashed child must refund too, or a run leaks fuel every time
  something fails — exactly when it can least afford it. This is what
  `lattice/src/lease.rs` is for: a balance held under a lease whose expiry *is*
  the refund, so a dead branch cannot strand fuel.
- **Every mutation is a CAS.** Two children settling against one parent
  concurrently is the default case. This is what ADR-0065 was for.

### How much to give a child

- **Equal split** — at branching `b`, depth `d`, a leaf holds `grant / b^d`. With
  `b=3, d=5` that is 0.4% each: every leaf starves before reaching the depth
  where the answer is.
- **Proportional to fitness** — all fuel follows the current best branch. Pure
  exploitation; the search never learns that the runner-up was second only
  because it was unfunded.
- **Floor plus proportional (the choice).** Every child gets a floor — enough for
  one honest attempt and an evaluation, or it should not have been spawned — and
  the remainder is split by expected value. The floor is exploration, the
  remainder is exploitation, and the trade-off is explicit instead of emergent.

A parent also **keeps a reserve**, because refuelling a plateaued-but-close child
is the highest-value thing it does, and a parent that distributed everything
cannot.

### The unit, and the thing alpha-swarm2 does not have

Three separate limits (time, tokens, iterations) cannot be traded against each
other. A branch that is one cheap test-run from an answer cannot spend leftover
tokens on it.

So: **one abstract unit, with a price list in config.** `1000 prompt tokens = N
units`, `1 CPU-second = M units`. The price list is versioned with the run. Then a
run costs one number, comparable across generations, with the exchange rates
visible rather than implied — and a monetary figure is one more multiplication
away, which is the thing alpha-swarm2 cannot do at all.

Iteration count stays as a separate hard cap. It is not a resource, it is a
loop-guard.

### Enforcement at the chokepoint, not by good behaviour

A budget agents must choose to check is not a budget. Enforcement goes where the
spending physically happens.

The mechanism is a **metered decorator**: a component that *exports*
`llm:inference/inference` and *imports* `llm:inference/inference`, sitting between
caller and provider. The caller cannot tell, and cannot bypass it — its import is
wired by composition, and a component cannot dial what its manifest does not
allow.

Verified before proposing: WIT accepts a world importing and exporting the same
interface, and `wasm-tools` resolves it. Whether `wac plug` wires it without
self-satisfying the import is the one thing to confirm at build time.

That decorator is the only place that needs to reserve, settle and refund — and
the only place that has ever read `completion.usage`.

### When fuel runs out

The node stops, keeps its best result, records `exhausted`, refunds nothing (it
has nothing). It does **not** fail. A parent must distinguish "this approach is
wrong" from "this ran out of money"; conflating them abandons the most expensive
and often most promising lines of work.

---

## What has to be tested before any of this is believed

Not a wish-list. This is the ADR's own rule applied to itself.

- **Conservation under concurrency** — spawn a tree, spend randomly, kill nodes
  at random, assert `Σ balances + spent + refunded == grant`.
- **A crashed child refunds** — kill it without a clean exit; the lease expires
  and the parent gets its fuel back.
- **A child cannot overspend** — racing a sibling settling against the same
  parent.
- **The decorator cannot be bypassed** — a component that tries to reach a
  provider directly is refused by egress, and the test asserts the refusal.
- **Every declared limit refuses something** — one test per limit that spends past
  it. This is the rule from the top of this document, and it is what separates
  this from a configuration file that describes a system nobody built.
- **A gate stub cannot pass for a gate** — assert the gate actually failed
  something, so a `-> true` can never sit behind it unnoticed.
- **Decay runs** — or is not claimed.

## Open questions this does not answer

- **Does a fork inherit its parent's knowledge, or start blank?** ADR-0080 left
  this open and it is still open. alpha-swarm2's answer — one shared pool per
  project — is the recommendation, but comp's environments make per-branch
  isolation *possible*, which means the choice has to be made deliberately.
- **Embeddings are unwired.** Semantic retrieval is the largest of the three
  retrieval layers and `llm:inference/embed` has never been called. Nothing else
  here matters much until that works.
- **Who runs the loop?** This ADR describes the mechanisms, not the driver. A
  component that spawns, evaluates, selects and refuels is a separate decision,
  and ADR-0079 only established that a component *can* fork its own app.
