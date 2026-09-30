# brief-561 — design: make the lazy pull machine and its consumers resumable

`Scope: NONE — no production change.` Design-only, with one declared scratch
exception for probes (see Scope). Survey the lazy-sequence machinery, design how
a suspended pull resumes into it, and return drafted implementation briefs.
**Do not run `git` in any form**, including before you have read this brief.

## Goal

Step 4, group G2 of the suspension-continuation branch. brief-548 found 14 sites
where a lazy sequence that asks for input mid-realization resumes **past** its
work: the pull stepper propagates `InputRequired` with a plain `?`, its state
lives on the Rust stack, and the realization loop above it is lost. The resumed
request completes with the inner callback's value instead of the realized list.

| Site | Role |
| --- | --- |
| `crates/rum-stdlib/src/lazy.rs:163,167,175,179,193,211,214` | `PullStepper::next` arms: `NestList`, `Map`, `Select`, `FoldList`, `Take`, `Drop` |
| `crates/rum-stdlib/src/lazy.rs:437-439` | `eval_lazy_table` evaluating its bounds |
| `crates/rum-stdlib/src/lazy.rs:584` | `realize` driving `next` |
| `crates/rum-stdlib/src/associations/basic.rs:217` | `Normal` realizing |
| `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:79` | `Take` on a lazy list |
| `crates/rum-stdlib/src/structure_ops/surface.rs:99` | `First` on a lazy list |

`Part` on a lazy list re-dispatches to `First[LazyDrop[…]]` through the kernel
seam, so it inherits the `First` repair. brief-548's witnesses
(`.temp/brief-548/probes.json`, `lost.tsv`) show, for example,
`Normal[LazyTake[LazyMap[f, LazyRange[1, 1]], 3]]` with `f` asking for input at
the second element: resumed `"two"`, queued `{1, "two", 3}`. The director
re-ran all 195 probes on `6614e59b` and every G2 divergence still diverges.

**Design the change; write no production code.** The repair is structural — the
stepper's state has to become something a frame can own — so the design comes
first.

## What the design must settle

1. **Ownership of the pull machine.** Today a `PullStepper` tree is driven
   through a borrowed `Session` and `ctx` with its state on the stack. Say how a
   pending pull becomes owned state a frame can hold — the whole adapter tree
   and the source position, `NestList`'s lookahead value, `Select`'s pending
   candidate, `FoldList`'s accumulator, `Take`/`Drop` counters, the
   realization prefix and limit, and the pull-budget / ceiling context charged
   at the source — and how resuming applies the one pending callback result,
   unwinds through the adapters that were waiting on it, and continues the
   realization driver. **One frame owning the machine, or a frame per adapter?**
   Choose, with reasons.
2. **Zero cost on the ordinary path.** A pull that never suspends must not
   clone the stepper, allocate, or pay a branch it did not pay before. Move
   state into a frame only when a callback actually returns `InputRequired`
   (`Option::take`, moving out of the stack frame). brief-560 learned that
   carrying resumable state in a hot loop's error arm can change the loop's
   codegen (`Differences` kept +0.82% after ten candidates, from an inlining
   change); design so the ordinary `next` loop keeps its shape, and name the
   microbench labels that would expose a regression.
3. **The consumers.** `Normal`, `Take`, `First` (and so `Part`) each finish
   the realization — wrap the list, return the element, or fall back to the
   original call — after the machine resumes. Say what each consumer's frame
   carries.
4. **`LazyTable`'s bounds** are evaluated eagerly at construction
   (`lazy.rs:437-439`); a suspension in the second bound must resume into the
   third and then build the plan.
5. **Semantics to preserve**: `NestList`'s eager lookahead; lazy lists re-derive
   on each realization (no memo in v1); the ceiling and cancellation are
   charged at the source; infinite sources still fail closed.
6. **Suspension twice and abandonment**: a second suspension inside the resumed
   pull; cancellation while suspended runs no further pull and leaves the
   session usable.
7. **The retention line**: lazy realization holds a growing prefix — say which
   brief-547 region it falls in and what state your frames carry so that
   conversion can build on it. No leases here.

## What I want back

1. The current structure, with `path:line`: `PullStepper` and its arms, `ctx`
   and the budget charge, `realize`/`realize_at_most`, and every consumer.
2. The design, per the seven points above, with the frame inventory (fields
   and roots).
3. Red tests: every brief-548 G2 witness and its twice-position variant
   (`LazyMap-twice`, `LazyNestList-twice`, `LazySelect-twice`,
   `LazyFoldList-twice`, `LazyDrop-twice`), plus an ordinary error after a
   resume, cancellation mid-realization, and an infinite source still failing
   closed — each through the oracle (resumed equals queued in value, `log` and
   messages, suspension asserted). Run them in the scratch test and record the
   pre-fix values.
4. Drafted implementation briefs in `TEMPLATE.md` shape — one step or several,
   along the design's real boundaries — each with its red tests, its predicted
   lost-continuation count, its performance gate (preserved base binary,
   allocation counts first, then per-iteration `cpu_core/instructions:u` with
   `taskset -c 2`, `(count(2N) - count(N)) / N`, all labels measured for every
   candidate), and the standing inventory duty. Scope as objective, fence and
   disclosure duty; serial runs (`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- `AGENTS.md`, `docs/code-organization-guidelines.md`; `docs/guide/evaluation.md`
  §Lazy Sequences; `docs/dev/lazy-sequences-design.md`.
- brief-548's report (mechanism, oracle), `.temp/brief-548/draft-G2.md`,
  `probes.json`, `lost.tsv`, `probe.rs`.
- brief-559 and brief-560 — how a loop's frame carries its state, and what the
  ordinary path cost.
- `crates/rum-runtime/src/runtime/suspension/core.rs` (`:515`, `:709`, `:784`),
  `eval_dispatch/mod.rs:466`.
- `crates/rum-stdlib/tests/loop_continuations.rs` and
  `guest_loading_resume.rs` — the permanent oracle harness shape.

## Scope

- Files you MAY modify: **NONE**, with three exceptions:
  - append your report to **this** file;
  - write scratch under `.temp/brief-561/`;
  - create **one** scratch integration test,
    `crates/rum-stdlib/tests/zz_brief561_probe.rs`, and **delete it before
    appending the report**, copying its final content to
    `.temp/brief-561/probe.rs`. Say in the report that you did.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- Touch nothing outside this repository. No network. Commands: file reads,
  `rg`/`grep`, `python3`, edits to the one scratch test, and bounded
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 cargo test -p rum-stdlib --test zz_brief561_probe`
  through `python3 scripts/run-bounded.py`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch artifacts, deletion of the scratch
   test.
2. The current structure.
3. The design and the frame inventory.
4. The red tests with their pre-fix values.
5. The drafted implementation briefs.
6. Open questions for the director.

---
