# brief-521 — maintain the session total unconditionally

## Goal

Replace the O(session) walk inside the armed checkpoint with a **maintained
session total**, updated as session state changes, and measure what it costs
on both paths.

brief-520 established the prize: a stubbed O(1) session term removes
**99.79%-99.99%** of the armed overhead, taking the worst shape from
801,714,469 instructions to 5,374,916, with a residual of ~50 instructions
per checkpoint that the disassembly fully accounts for. The budget it
derived is generous — a real implementation may spend about **2,495 / 3,017
/ 7,697** instructions per steady checkpoint for the three session sizes and
still retain 99% removal.

## Director decision: no accounting mode (2026-09-22)

brief-518 built a mode route so that **unarmed** sessions could skip
accounting. The user's corrected ruling removes its reason for existing: an
unarmed cost of up to about **0.7%** is now acceptable, because the memory
ceiling is a documented MUST for untrusted input rather than off-by-default
diagnostics. **If we are willing to pay unarmed, the mode buys nothing.**

So maintain the total **unconditionally**, with no mode, no specialization,
and no branch in the hot evaluator. This also avoids what brief-518 measured
as a side effect: `evaluate_once_unstable_after_input_check_with_cycle_tracking`
growing from 1,854 to 4,150 bytes because both specializations were inlined
into it.

brief-518's prototype stays available at
`target/accounting-route-518/prototype.patch` if this route fails. **Do not
restore it here.**

## Why this is not brief-519's refuted route

brief-519 failed on **fail-open**: it *inferred* freshness from existing
counters, and 42 of the 52 walked fields had no covering counter, so it
reused a stale low total on the very reproduction this line exists to fix.

This brief does not infer anything. It **maintains** the total at the sites
that change it. The failure mode is therefore different and must be guarded
differently: a missed charge or release is **drift**, and downward drift is
fail-open. That is what the falsifier below is for.

## The work

1. **Maintain the total.** Use brief-516's charge-and-release analysis and
   brief-517's per-field occurrence specification as the definition of what
   is being maintained. The maintained value must equal what the walker
   computes — the **occurrence** model, per the user's ruling, not a
   unique-allocation count.
2. **Enumerate charge and release separately**, as brief-516 required.
   Release paths include error and panic exits; a release that is skipped on
   an error path is a permanent upward drift, and a charge skipped anywhere
   is a permanent downward one.
3. **Use the maintained total in the armed checkpoint**, keeping the
   expression term exactly as it is — brief-520 showed the expression walk is
   the residual that must remain.
4. **Keep the walker.** brief-517 built it as an independent oracle
   specifically so this step could be checked against it. It stays in the
   tree and stays authoritative for `MemoryInUse[]`-style observation.

## The falsifier, which must be watched failing

Assert **maintained total == walker total** at every armed checkpoint under a
debug or test configuration, across a workload set that exercises
definitions, caches, contexts, the evaluation stack, prints, traces, reaps,
suspension, abandonment, and error exits.

Then **watch it fail**: delete one charge site, show the assertion red;
delete one release site, show it red the other way; restore both and show
green. Paste all of it. A drift detector nobody has seen fire is not known
to work — and this line has already been saved twice by insisting on that.

## The measurement, which decides acceptance

**Unarmed** — the maintenance cost, since it is now unconditional. Use
`target/brief-501/smoke50000.rum` (calibrated: base 1,376,080,468,
within-arm spread 0.00005%) plus at least two evaluation-heavy labels lifted
verbatim from `scripts/core-language-microbench.sh`. Report **scaled slopes**.
**The bar is ~0.7%**; above it, stop and report rather than tuning.

**Armed** — the nine shapes at `target/brief-514/armed-*.rum`, unchanged, so
the numbers compose with brief-514's table and brief-520's ceiling. Report
the fraction of overhead removed against both.

Recipe: `taskset -c 2 perf stat -e instructions:u -x, --`,
`cpu_core/instructions/u`, `100.00` multiplex, three runs per arm, release,
profiling env OFF, both arms built at the same path, stdin closed. Verify
counts scale with the workload. **Explain any anomaly before accepting it** —
brief-519's favourable numbers were an inlining artifact and were caught only
because explanation was required. **Do not run the timing panel.**

## Scope

MAY modify: the accounting modules under
`crates/rum-runtime/src/runtime/debug_tools/resources/`, the `Session`
mutation sites the inventory names, their module declarations, the
corresponding runtime tests, and `docs/guide/resource-limits-and-sandbox.md`.

Anything else the compiler forces is a **disclosure duty**: list it with the
reason. Not compiler-forced and not listed is a stop.

Do NOT touch: `Expr`/`SharedExpr`/`SharedArgs`, the interner, the builtin
catalog's growth declarations, `scripts/**`, `AGENTS.md`, `TODO.md`,
`PROGRESS.md`, `Cargo.toml`, `Cargo.lock`, `.git`, the ownership-graph
inventory fixtures. **Do not restore brief-518's prototype.** **Do not touch
the 165 per-builtin `Incremental` declarations** — that is a separate debt.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

**Expected on `main`:** 2 509 passed, 0 failed, 57 ignored on a host that can
bind loopback. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; report the
nine native ZeroMQ failures as environmental if your sandbox denies loopback
and say plainly they did not execute. Any other failure is yours.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`.

**Append your report to this file**, and keep raw evidence under
`target/brief-521/`.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **No accounting mode, no specialization, no hot-path branch on armed
  state.** If the total cannot be maintained without one, that is a
  refutation to report.
- **Do not weaken the occurrence model** to make maintenance easier. Shared
  subtrees are charged per occurrence; a cheaper different number is not an
  answer.
- **Do not delete or weaken the walker.** It is the oracle.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No network. Touch nothing outside this repo; read `target/brief-5*/` and
  `target/accounting-route-518/` and `target/measure-achievable-520/` freely,
  overwrite nothing there.
- **Stop-loss, keyed on progress.** Same step failing twice with no new
  information means stop and report.
- Paste real verbatim output, including failures.

## Report format

1. The charge and release inventory, the two listed separately.
2. The falsifier, with all three runs: green, red on a deleted charge, red on
   a deleted release.
3. Unarmed numbers with scaled slopes, against the ~0.7% bar.
4. Armed numbers against brief-514's table and brief-520's ceiling.
5. Verification, verbatim, with the complete failure set.
6. Anything you refused, and open questions.
