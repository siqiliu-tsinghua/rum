# brief-565 — erase the ordinary-path cost G2 added

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

briefs 563 (`79b42d54`) and 564 (`fd542ab3`) closed group G2: the lazy pull
machine is flat and depth-safe, a suspended lazy callback resumes into its
realization and its consumer, and `LazyTable`'s bounds resume into the plan.
Both are correct and gated. But the **ordinary path** — a call that never
suspends — got dearer. Per-iteration user instructions (pinned `taskset -c 2`,
`cpu_core/instructions/u`, `(count(2N) - count(N)) / N`), the current tree
against the pre-G2 binary, from brief-564's report §3:

| Label | Pre-G2 | Now | Delta, sample 1 / 2 |
| --- | ---: | ---: | --- |
| `take-multispec-small` (**ordinary list `Take`**, no lazy value) | 50 782 | 51 092 | +0.61% / +0.60% |
| `lazy-range-take-small` (`Take[LazyRange[1,1],8]`) | 41 864 | 42 741 | +2.09% / +2.28% |
| `lazy-range-normal-small` | 53 656 | 54 898 | +2.31% / +1.99% |
| `lazy-subsets-take-small` | 25 474 | 25 886 | +1.62% / +1.36% |
| `lazy-tuples-take-small` | 26 639 | 27 065 | +1.60% / +1.45% |
| `lazy-permutations-take-small` | 28 226 | 28 640 | +1.47% / +1.57% |
| `lazy-foldlist-realize-small` | 113 099 | 114 594 | +1.32% / +1.49% |
| `lazy-select-realize-small` | 114 232 | 115 382 | +1.01% / +1.23% |
| `lazy-table-realize-small` | 116 678 | 117 823 | +0.98% / +1.05% |
| `lazy-table-construct-full` / `-short` | 24 056 / 23 582 | 24 264 / 23 726 | +0.86% / +0.85%; +0.61% / +0.68% |

and fifteen more lazy labels between +0.05% and +0.9% (the full 26-row list is
in brief-564's report §3, "Cumulative pre-G2 delta"). Allocation counts, bytes
and peaks did **not** regress anywhere — they fell — and the deep labels got
faster (`lazy-depth-map-build-256` -7.6%). This is instruction cost.

The director reproduced the key rows independently on brief-563's binaries:
`take-multispec-small` +0.658% / +0.669%, `lazy-range-take-small` +2.135% /
+2.208%, `lazy-depth-drop-all-one-8` +2.518% / +2.533%, with
`dot-three-no-input` at -0.042% as the control.

The design said a pull that never suspends pays nothing for the continuation
machinery. Make that true. **Target:** every one of the 54 panel labels at or
below the pre-G2 base in both samples (a delta no larger than +0.05% in either
sample counts as at the base), with every correctness test unchanged and green.

## Fixed — not negotiable in this brief

- **The flat representation and its depth contract.** One source plus a stage
  vector under an indexed cursor; no walk recurses in proportion to depth.
  The structural checks (`tests/association_depth_contract/lazy_depth_contract.rs`:
  no recursive call edge in the lazy sources, per file and as one unit; no
  self-reaching lazy type) and the 2 000-stage depth tests stay unchanged and
  green. **A regression is never bought back by restoring recursion** — the
  recursion was a process abort.
- **Semantics.** `crates/rum-stdlib/tests/lazy_pull_continuations.rs` and its
  children, `loop_continuations.rs`, and every other test stay exactly as they
  are. The cancellation checkpoint after each callback stays.

## Where the director would look first — hypotheses, measure each

1. **Ordinary `Take`, +~310 instructions per call, no lazy value involved.**
   `eval_take` (`structure_ops/sequence/take_drop.rs:22`) tests
   `is_lazy_list` and otherwise takes the ordinary path, which G2 did not
   touch. Suspect code shape: the lazy branch `eval_take_lazy_list` (`:59`)
   now carries a capture call and may be inlined into `eval_take`, changing its
   frame or register allocation. Keep the lazy branch out of line and cold;
   check `is_lazy_list` did not change cost when `lazy.rs` was restructured.
   The same shape applies to `eval_first` (`surface.rs:~98`) and `eval_normal`.
2. **`LazyTable` construction.** Each bound is cloned on the success path
   (`session.evaluate(imin.clone())`, `lazy.rs:~248-274`) only so the error
   edge can capture it. Re-derive the normalized bound from the held
   arguments inside the cold helper instead, so the ordinary path evaluates
   the moved value as before.
3. **Shallow realizations.** A source-only `Take[LazyRange[1,1],8]` now builds
   a `PullMachine` (a `Vec` even with zero stages), a `PullContext` carrying
   `pending: Option<Box<AwaitCursor>>`, and enters `drive_pull` through a
   `DriveCursor` per item. Compare against the old recursive `next` for depth
   0-1 and remove what the ordinary path does not need — e.g. what is built per
   item that could be built per realization, a cursor enum where a plain index
   suffices on the ordinary entry, bounds checks the loop can prove away
   without `unsafe`.

**Attribute before you change.** For each label family, run
`valgrind --tool=callgrind` on the pre-G2 base and the current binary for the
same workload, calibrate the total against `perf stat` on that binary, and
compare inclusive and self counts per function. brief-560 found its
`Differences` residual this way (`difference_expr` lost its inlining) after
nine timing-blind candidates; attribution first is cheaper and more honest.

## Measurement — the preserved base is the reference

- **Base:** `.temp/brief-563/base/rum-cli` and `rum-cli-alloc` (pre-G2,
  sha256 `c08b67e8…` and `4285dd8a…`). Do not rebuild them. Measure the base
  once at the start in this brief (same window as the candidates).
- **Panel:** `.temp/brief-562/performance-panel.json` — 54 labels; workloads
  are `.temp/brief-563/perf/<label>-<N>.rum`, with result checks in
  `.temp/brief-563/perf/<label>-check.rum`. Check each workload's result before
  measuring it; a wrong result measured nothing.
- **Method:** allocation counts first (alloc binary, `RUM_PROFILE_ALLOC=1`),
  then two samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`
  (the `/u` spelling; `:u` is rejected by this host), one label per process,
  stdin `/dev/null`, all other `RUM_PROFILE_*` cleared. Per-iteration
  `(count(2N) - count(N)) / N`. Raw output saved before parsing.
  `.temp/brief-563/metric-panel.py` and `compare-metrics.py` are a working
  harness; copy and adapt them into `.temp/brief-565/`.
- **Candidates:** build each into its own directory under `.temp/brief-565/`,
  keep every binary you measure, record its hash and its source. An
  exploratory candidate may be screened on the 26 regressing labels plus the
  nine brief-559 labels and `table-small`, `table-iterator-plus`,
  `table-iterator-pair-list`; **any candidate you keep, and the final one,
  runs all 54 labels.**
- **Timing:** not needed; instruction counts decide.

## Scope

- **Objective:** the ordinary-path shape of the G2 code — `crates/rum-stdlib/src/lazy.rs`,
  `crates/rum-stdlib/src/lazy/`, and the lazy branches and completion capture
  of `eval_take`, `eval_first` and `eval_normal` — so that all 54 labels meet
  the threshold.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and both certificates, reconcile the
  ownership inventories, and keep rum-runtime's frame lists exact.
  Positions may move; new `UnknownConstruct` rows for code you add or change
  are expected and disclosed. STOP on a changed or vanished existing row
  identity, a new `UnknownHostPayload` or `self_reaching` row, or a row in code
  you did not touch.
- **Fence:** no semantic change and no test edit. No other brief-548 group, no
  rum-runtime or rum-core production change, no `Expr` change, no leases, no
  new dependency, no `unsafe`, no depth cap, no recursion. Do not edit
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier briefs or docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  candidate — what it changed, its attribution evidence, and its rows.

## rum / Rust pitfalls (read twice)

1. **One variable per candidate.** Change one thing, then measure.
2. **An improvement that pushes another label above the threshold is
   rejected**, including a deep label pushed above the base.
3. **Instruction counts decide.** Do not substitute timing.
4. **Stop-loss is keyed on progress:** the number of labels above the
   threshold must fall, or the largest remaining delta must. Stop after two
   consecutive compiling candidates that reduce neither, and report the curve
   with the attribution for each residual. At most ten measured candidates.
5. `#[inline(always)]` is not a fix by itself: brief-560 measured it making
   `Differences` worse (+1.17%). Prefer moving cold work out of hot functions
   over forcing hot work in.

## Verification (MANDATORY — you run it; iterate to green)

Serial, every Cargo command through `python3 scripts/run-bounded.py --timeout
<s> -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-565/tmp …`. Bounds from measured serial history: a
focused test target 150 s; a release `rum-cli` build 300 s incremental, 1 200 s
into a fresh target directory (check the produced binary's hash changed —
brief-564 found a "fresh" build that left a stale binary); a metric process
150 s (the 256-stage fold label takes ~72 s at 2N); `local_production_acceptance`
and `cargo test -p rum-runtime` 300 s; `cargo test --workspace --no-fail-fast`
2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test lazy_pull_continuations
cargo test -p rum-stdlib --lib lazy
cargo test -p rum-stdlib --test loop_continuations
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

Plus the retention replay in both feature sets as the template describes.
Paste commands, bounds, exit statuses and counts verbatim; list native ZeroMQ
skips as skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-565/`. After each candidate write
`.temp/brief-565/PROGRESS.md`: its change, its attribution, its rows, the
counters, the exact next action and the tree's state.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository. No network. Commands: file reads and
  edits within scope, `rg`/`grep`, `python3`, bounded `cargo`, the release
  `rum-cli`, bounded `taskset`/`perf`, bounded `valgrind --tool=callgrind`
  with `callgrind_annotate`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges.
2. The attribution: for each regressing family, the functions whose counts
   moved, base against the incoming tree, with the callgrind calibration.
3. Every candidate measured, with its rows (all 54 for kept candidates), both
   samples.
4. Verification — every command's verbatim result.
5. Progress counters, disclosures, and the attribution of anything left.

---
