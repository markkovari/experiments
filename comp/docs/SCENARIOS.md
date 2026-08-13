# Scenarios — how a graph run succeeds, and the ways it does not

Three levels of difficulty, each with what success looks like, what failure looks
like, and — the part that matters — **which of them can be tried today**.

The distinction this document keeps is between:

| | |
|---|---|
| ✅ **covered** | there is a test, and it has been seen to fail against the bug it exists for |
| ⚠️ **runnable** | the pieces exist and nobody has put them together |
| ❌ **blocked** | something named below does not exist yet |

**The loop now runs.** `graph:agent/writer` turns a goal and a tree into a
candidate; `graph:run/driver` attempts, judges, repairs and stops. The repair is
driven by what the checks actually found — proven without planting anything,
since the scripted model matches only on check ids, and a check id reaches it
only if the driver lifted it off a verdict `comp-checks` produced by running the
command. What is still ❌ is everything ABOVE one branch: which of N runs wins,
what a generation costs, and whether the branches were ever different.

---

## Level 1 — simple: one goal, one branch, one gate

A person writes `.comp/goals/cache.md`, starts the goal, one branch tries it,
the checks run, a pull request opens.

    goal ──▶ branch ──▶ candidate ──▶ checks ──▶ PR ──▶ human merges

### Success

| step | how it is known to work |
|---|---|
| the goal is queued and only a human starts it | ✅ `projects.rs` — waits six seconds and asserts it is still queued |
| the branch gets its own store | ✅ `environments.rs` |
| the candidate is judged by real commands | ✅ `fitness.rs` — 1000 for a candidate that fixes it |
| the winner becomes a commit and a PR | ✅ `forge.rs` — six calls, branch created last |
| a recompiled artifact actually replaces the running one | ✅ `newversion.rs` |

### Failure modes

| what goes wrong | what happens now |
|---|---|
| the agent produces no diff | ✅ refused — `git:forge` rejects a proposal with no changes, because a diff-less PR is what "reported success having done nothing" looks like |
| the gate is empty | ✅ refused — an empty check list would be *vacuously* accepted, which is how a swarm accepts everything |
| a check hangs forever | ✅ killed after the timeout, reported as killed rather than failed |
| a check names `rm -rf` | ✅ refused by the allow-list, as a failed check naming why |
| the candidate writes `../../etc/passwd` | ✅ refused by both the runner and the forge |
| the base moved while the branch worked | ❌ **unhandled.** `git:forge/base-commit` pins the start; nothing detects that the base moved before the PR opens. Rebase or refuse is an open question (ADR-0082) |
| the model is down | ✅ **ends the run rather than spending the budget on it** — distinct from an unusable *answer*, which costs one attempt and is retried with the next seed. Nothing degrades to another tier yet |
| the model repeats itself | ✅ `driver.rs` — an attempt reproducing a candidate already on record stops the run as a `plateau`, at 2 of a 5-attempt budget |
| a repair is worse than what it repaired | ✅ the best candidate by score is kept, not the last, and a tie keeps the *earlier* one — otherwise the answer depends on how many times it happened to tie |

**The honest summary of level 1:** one branch now goes from a goal to a scored,
repaired candidate. What is left is the base-drift question, and that the
candidate still has to be handed to the forge by whoever compares branches.

---

## Level 2 — complex: one goal, many branches, selection

A generation of eight branches explores the same goal; each is judged; the best
is promoted; the rest are closed.

    goal ──┬─▶ branch 1 ──▶ 1000  ← wins
           ├─▶ branch 2 ──▶  500
           ├─▶ …
           └─▶ branch 8 ──▶    0

### Success

| step | how it is known to work |
|---|---|
| eight branches spawn concurrently | ✅ `stress_env.rs` — 8/8 accepted, converged in 3.0s |
| each has its own store, none shares | ✅ asserted by name; the bucket-name collision at depth six is fixed and unit-tested |
| candidates are ranked when none is acceptable | ✅ `fitness.rs` — 1000 / 500 / 333, with the last two both failing the gate |
| derived work is computed once for the generation | ✅ `artifacts.rs` — twelve concurrent lookups, exactly one producer |
| closing a branch closes what grew from it | ✅ `stress_env.rs` |

### Failure modes

| what goes wrong | what happens now |
|---|---|
| every branch fails the gate | ✅ **the score still orders them** — this is the whole reason the runner reports a vector rather than a verdict |
| two branches produce identical candidates | ⚠️ `artifact:cache` would dedupe the derived work, but nothing dedupes the *candidates*, so both get gated and both get scored. They are at least counted once in `distinct` |
| all eight converge on the same idea | ⚠️ **it now announces itself**: every selection reports `distinct`, and 1 means the parallelism bought nothing (`select.rs`). Detecting it is not fixing it — none of ADR-0081's mitigations is built |
| scores tie | ✅ `graph:select` — score, then the smaller change, then the cheaper run, then the earlier branch. The last exists to be DETERMINISTIC: a selection that varied between identical runs could not be argued with afterwards |
| the fleet cannot place eight more branches | ✅ refused with a 429 naming the lag and the limit, rather than accepted and never started |
| a burst outruns the limit | ✅ counted against the last report, so 625 spawns are cut to 435 |
| the generation costs more than the budget | ❌ **nothing spends against the budget.** `max-attempts` bounds tries per branch, which is not a cost — a run of three cheap attempts and a run of three expensive ones are the same number. Every attempt is now recorded with a digest and a score, which is the raw material a real budget needs and did not have |

**The honest summary of level 2:** a branch decides when to stop and a generation
decides which branch won. Nothing decides how MANY branches, or extends one into
the next generation.

---

## Level 3 — very complex: a search, over time, with things breaking

Generations of generations, over hours, with hardware failing underneath and a
human in the loop.

    gen 1 ──▶ pick 2 of 8 ──▶ gen 2 ──▶ pick 1 of 4 ──▶ gen 3 ──▶ human ──▶ PR
                   │                         │
              6 closed                  3 closed

### Success

| step | how it is known to work |
|---|---|
| branches of branches | ✅ depth 4 measured, 5 generations side by side, ~3s a level |
| 341 apps across four generations | ✅ `stress_tree.rs` |
| two of three nodes SIGKILLed mid-run | ✅ recovered in 18s and 24s; nothing told the lattice |
| closing one first-level branch closes 85 descendants | ✅ measured |
| a human starts, the loop runs, a human lands | ✅ every component on the path exists and is tested end to end — `comp goal start`, the driver's loop, the selector, the pull request. What is untested is the whole path in ONE run, because nothing yet fans a goal out into N branches |

### Failure modes

| what goes wrong | what happens now |
|---|---|
| a node dies holding half the tree | ✅ inventory expires, the reconciler sees a gap, work is re-placed |
| desired state is larger than the platform will report | ✅ **fixed, and it was silent**: a fleet asked for 3906 apps sat at exactly 500 forever, every one past the cap accepted and never placed |
| the search never converges | ⚠️ **within a branch it does**: a repeated candidate stops the run. Across generations there is still no plateau detection and no `loop-until-dry` |
| a branch runs out of fuel | ❌ **no fuel exists.** ADR-0081 designs conservation, escrow and refund-on-death; none is built. `quota:meter` is a rate limit and not a budget |
| a branch waits on a human for two days | ❌ suspension is designed (`awaiting-human`, environment released, fuel in escrow) and unbuilt. Today a branch would simply sit there holding a node |
| the knowledge pool fills with a wrong lesson | ❌ the graph stores and traverses; nothing promotes, weights by outcome, or decays |
| two runs race the same repository | ✅ **cannot happen by construction** — one active run per project, which is the entire answer to concurrent pull requests until somebody raises the limit |
| a run's base goes stale mid-search | ❌ unhandled, and it bites *with* a serial queue — serialising does not avoid it |
| the loop asks a human fifty times | ❌ **the interruption rate is unmeasured**, and every argument about interfaces is really an argument about that number |

**The honest summary of level 3:** the substrate survives things breaking. The
search does not exist — no fuel, no stopping rule, no selection strategy, no
memory that improves.

---

## What the matrix says, taken together

Read down the ❌ column and the shape is consistent. **Everything that carries
work is built and has been broken on purpose to prove it. Everything that
DECIDES is designed and unbuilt.**

    carrying    environments, vgit, forge, artifact cache, checks, admission
                → survived 341 apps and two dead machines

    deciding    the agent, its loop, and selection
                → built; stops for a stated reason, and only an accepted
                  branch can reach a pull request
                fuel, generation size, knowledge promotion
                → ADR-0081, marked proposed, none of it running

That is a deliberate order rather than an accident: a wrong decision on a
substrate that loses work is impossible to debug, and every mechanism above was
built by breaking it first. But it means the honest answer to "can the graph
succeed" today is:

> **Every component between a goal and a pull request now exists and is tested,
> including the rule that picks a winner and the guarantee that a branch which
> failed its checks cannot reach a repository. What is missing is the thing that
> RUNS them together: nothing fans one goal out into N branches and collects the
> results.**

The smallest thing that would change that is a generation runner — spawn N
environments, run the driver in each, hand the results to the selector. Every
piece it would call is built, and the fan-out itself is measured already:
`stress_env.rs` puts eight branches up concurrently in three seconds.
