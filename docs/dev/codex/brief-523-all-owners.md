# brief-523 — maintain every owner, and measure the curve as you go

## Goal

Complete the `Incremental` line: maintain **all 52 accounted owners** behind
the accounting mode, so the armed checkpoint stops walking the session
entirely, and **measure the unarmed cost after every batch** so the curve is
known rather than discovered at the end.

## Standing user ruling this brief operates under (2026-09-22)

> 我们是研究型项目，所有路径都要确定地趟过，量过真实的代价才能得出结论，
> 可以回退，可以浪费时间，但不能不知道。

Concretely, and this **overrides the usual budget-stop convention**:

- **The ~0.7% unarmed bar is a data point, not a stop condition.** If a batch
  pushes the unarmed cost past it, **record the number and keep going**. Do
  not stop, do not tune, do not reorder to flatter the curve.
- **Report the curve**: unarmed cost after each batch, so the shape of the
  growth is visible and attributable to owners.
- Stop only for **correctness** — a drift the falsifier catches and you
  cannot explain, a test failure you cannot attribute, or a design conflict
  that would require changing what is being measured.

## Where the work is

**brief-522's slice is already in your working tree**, applied by the
director from branch `spike/brief-522-slice`. Do not reconstruct it and do
not use `git` to switch, merge or restore. It contains:

- the accounting-mode route restored from brief-518,
- the evaluation-stack owner maintained behind that mode,
- the drift falsifier for that owner, watched failing both directions,
- `docs/guide/resource-limits-and-sandbox.md` updates.

It also contains **five change-caused test failures** that brief-522 reported
and did not fix. **Fix them first**, and say what each was — a prototype
defect and a real invariant conflict are different findings.

## What to build

The remaining **51 owners**, using **brief-521's charge-and-release
inventory** (in `docs/dev/codex/brief-521-maintained-session-total.md`),
which enumerates all 52 with their charge boundaries and owning modules. It
survived its own refutation and is the specification for this work.

Work in **batches by the families that inventory already groups** — builtins
and limits; attributes and their caches; definitions and dispatch indexes;
the four caches; options and syntax information; stubs, removals, generations
and once-values; contexts and paths; providers and sinks; and whatever
remains. After each batch:

1. Run the **drift falsifier** extended to that batch's owners: maintained
   total == walker total at every armed checkpoint. **Watch it fail** for at
   least one owner per batch — delete a charge, show red; delete a release,
   show red the other way; restore, show green.
2. Measure the **unarmed** slopes and record them in the running curve.
3. Only then start the next batch.

When every owner is maintained, **remove the session walk from the armed
checkpoint** and measure the armed shapes.

## The measurements

Recipe throughout: `taskset -c 2 perf stat -e instructions:u -x, --`,
`cpu_core/instructions/u`, `100.00` multiplex, three runs per arm, release,
profiling env OFF, both arms built at the same path, stdin closed. Verify
counts scale with the workload. **Explain any anomaly before accepting it** —
brief-519's favourable numbers were an inlining artifact. **Do not run the
timing panel.**

- **Unarmed, after every batch** — `target/brief-501/smoke50000.rum` plus the
  same two evaluation-heavy labels brief-522 used, scaled slopes. Baseline is
  `main`; brief-522's slice reads +0.526024% / +0.838311% / +0.645868%.
- **Armed, at the end** — the nine shapes at `target/brief-514/armed-*.rum`,
  unchanged, against brief-514's table and brief-520's ceiling of
  99.79%-99.99% removal.
- **Cold arm, at the end** — single arming, and
  `Do[MemoryConstrained[Null, 1000000000], {N}]` at N = 16, 128, 1024.
  brief-522's slice regressed 96.7%-98.8% there because it walked twice per
  arming; with every owner maintained it should walk once. **Confirm that, or
  report that it does not.**

## Scope

MAY modify: everything brief-522's slice touches, plus the charge and release
sites brief-521's inventory names across
`crates/rum-runtime/src/runtime/` and the `crates/rum-stdlib` sites it lists,
their module declarations, the corresponding tests, and
`docs/guide/resource-limits-and-sandbox.md`.

Anything else the compiler forces is a **disclosure duty**: list it with the
reason. Not compiler-forced and not listed is a stop.

Do NOT touch: `Expr`/`SharedExpr`/`SharedArgs`; the interner; the 165
per-builtin `Incremental` declarations, which are a separate debt;
`scripts/**`; `AGENTS.md`; `TODO.md`; `PROGRESS.md`; `Cargo.toml`;
`Cargo.lock`; `.git`; the ownership-graph inventory fixtures **except** to
regenerate them if source locations move, in which case every content change
is listed with its witness and classified, and anything unclassifiable is a
stop.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. Leave changes in the
  working tree; the director gates.
- **No new `unsafe`.**
- **No inferring freshness from counters** — brief-519's refuted route.
- **Do not weaken the occurrence model.** Shared subtrees are charged per
  occurrence; a cheaper different number is not an answer.
- **Do not delete or weaken the walker.** It is the oracle the falsifier
  compares against, and it stays authoritative for `MemoryInUse[]`.
- **Do not tune for the unarmed bar.** Record and continue.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No network. Touch nothing outside this repo; read `target/brief-5*/`,
  `target/accounting-route-518/` and `target/measure-achievable-520/` freely,
  overwrite nothing there.
- **Stop-loss, keyed on progress**, and only for correctness as described
  above. Same step failing twice with no new information means stop and
  report.
- Paste real verbatim output, including failures.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

**Expected on `main`:** 2 509 passed, 0 failed, 57 ignored on a host that can
bind loopback. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; report the
nine native ZeroMQ failures as environmental if your sandbox denies loopback,
and say plainly they did not execute. **The five slice defects must be fixed**
— any remaining failure is yours.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`.

**Append your report to this file**; keep raw evidence under
`target/brief-523/`.

## Report format

1. The five slice defects, what each was, and how each was fixed.
2. **The unarmed curve**: cost after every batch, attributable to owners.
3. The drift falsifier per batch, with the watched failures.
4. Armed numbers against brief-514 and brief-520.
5. Cold-arm numbers, and whether the double walk is gone.
6. Verification, verbatim, with the complete failure set.
7. Anything you refused, and open questions.
