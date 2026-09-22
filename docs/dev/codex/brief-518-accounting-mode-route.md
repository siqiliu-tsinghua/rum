# brief-518 — prove an accounting-mode route exists, or refute it

## Goal

The `Incremental` line needs an armed operation to carry accounting context
through the places evaluation actually goes. Before any ledger is built,
prove that a route exists **and costs the unarmed path nothing** — or
demonstrate that it does not.

This is a **feasibility gate, not a cutover**. `current_memory_proxy_bytes()`
still decides every memory limit throughout, so this prototype cannot create
an unchecked gap. A shadow counter runs beside it; nothing switches over.

## The four crossings that decide it

A route is only real if it survives all of these. Prove each, with evidence:

1. **Recursive evaluation** — the ordinary descent.
2. **A builtin callback** — guest code re-entering the evaluator from inside
   a builtin.
3. **A host mutation** — the embedder changing session state.
4. **A suspended and resumed continuation**, including host entry **while
   suspended**, and a nested `MemoryConstrained` entered from an **unarmed**
   call. Those last two are mode transitions too, and are the ones most
   likely to be missed.

## The crux, stated so you cannot route around it

`BuiltinFn` is `fn(&mut Session, &Expr, &SharedArgs) -> Result<Expr, EvalError>`
(`crates/rum-runtime/src/runtime/metadata.rs:332`) — an erased function
pointer with nowhere to put a mode. **A generic outer evaluator that calls an
erased `fn` has lost the mode.** Solve that or report that it cannot be
solved; do not paper over it with a global, a thread-local, or a
pointer-identity cache. Those are explicitly forbidden below.

## Director ruling on the API boundary (2026-09-22)

My earlier ruling was that public extension behavior is preserved. Concretely,
for this brief:

- **Additive is allowed.** A new registration or adapter path that carries
  the mode may be added alongside the existing one.
- **Breaking is not.** The existing public registration surface keeps its
  shape and its meaning. A host registering a builtin the way it does today
  must keep working.
- **A builtin registered the old way is accounted by the walker** — the
  fallback — and that is a **stated, documented gap**, never silent
  mis-accounting. Say exactly which registrations land in which case.

If you cannot satisfy those three at zero default cost, that is the
refutation this brief asks for.

## Deliverable

- The mode route itself, plus a **small representative shadow counter** for
  stack push/pop and one builder or continuation boundary. Shadow only: it
  observes, it does not decide.
- One pilot builtin — a `Table` wrapper is the design's suggestion — with its
  catalog registration, to prove the route reaches a real builtin.
- **A concrete adapter API and ripple inventory**: what changes, what a host
  sees, and which registrations fall back to the walker.
- **Compiled-code evidence that unused accounting erases from unarmed
  operations.** Not an argument that it should — evidence that it does.
- A **proceed or refute verdict**, stated plainly.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

plus focused tests for each of the four crossings above.

**Expected state on `main`:** 2 509 passed, 0 failed, 57 ignored on a host
that can bind loopback. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if
your sandbox denies loopback, report those nine as environmental and say
plainly they did not execute. Any other failure is yours.

**Both A/B gates are required**, using the instruction-count recipe
(`taskset -c 2 perf stat -e instructions:u -x, --`, `cpu_core/instructions/u`,
`100.00` multiplex, three runs per arm, release, profiling env OFF, both arms
built at the same path, stdin closed):

- **Unarmed**: `target/brief-501/smoke50000.rum` (calibrated: base reads
  1 376 080 468, within-arm spread 0.00005%) plus at least two
  evaluation-heavy labels lifted verbatim from
  `scripts/core-language-microbench.sh`. Report **scaled slopes**, not one
  point. **Any unarmed regression voids the route** — it does not get traded
  against an armed benefit.
- **Armed**: including the **cold-arm cost**, i.e. what arming itself costs
  before any work happens.

Verify instruction counts scale with the workload's `N` before trusting
them; a `Do` over 200 000 iterations silently hits the 100 000 iteration
ceiling and measures process startup.

**Do not run the timing panel.** Its self-comparison moves further than the
effects being measured here.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`; a recorded
run took about 160 s.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **No global state, no thread-local, no pointer-identity cache** as a
  shortcut for carrying the mode. If the mode cannot be carried explicitly,
  that is the refutation.
- **Do not cut over `current_memory_proxy_bytes`.** The walker decides every
  limit for the whole of this brief.
- **Do not migrate unrelated builtin families.** One pilot, named.
- **Accounting guards must not borrow `Session` across guest evaluation.**
- **Layout and instruction-cache regressions count** even when a source
  branch disappeared.
- If the route fails, **report the refutation and leave correct walking in
  place**; do not generalize a failed prototype across the tree.
- Anything the compiler forces outside the files you name is a **disclosure
  duty**: list it with the reason. Unlisted and not compiler-forced is a stop.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No network. Touch nothing outside this repo; read `target/brief-5*/` freely
  and overwrite nothing there.
- **Stop-loss, keyed on progress.** Same step failing twice with no new
  information means stop and report.
- Paste real verbatim output, including failures.

## Report format

1. The proceed-or-refute verdict, first.
2. The four crossings, each with its evidence.
3. The adapter API and ripple inventory, including which registrations fall
   back to the walker.
4. Erasure evidence for the unarmed path.
5. Both A/B gates, with the scaling check.
6. Verification, verbatim, with the complete failure set.
7. Anything you refused, and open questions.
