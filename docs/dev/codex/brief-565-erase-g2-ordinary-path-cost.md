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

## Execution report — brief 565

**Target not met; required stop-loss reached.** Two compiling candidates were measured and rejected. The retained tree is byte-for-byte the incoming flat G2 implementation. It still has 29 labels above pre-G2 +0.05% in at least one sample. All mandatory correctness commands and both retention feature-set replays passed on that restored tree. The following is the final five-part report; no optimization remains in the working source.

### 1. What changed — files, ranges, and retained state

The only retained authored change outside scratch is this append to `docs/dev/codex/brief-565-erase-g2-ordinary-path-cost.md`, lines 203–1493. The original brief is preserved verbatim. Scratch, source snapshots, all measured binaries, scripts, raw outputs, and the checkpoint are under `.temp/brief-565/`. Cargo and the retention tests also produced their normal ignored build/replay artifacts under `target/` and `.temp/brief-546/certificates/`; those are not source changes.

All 1,226 entry `.rs`, `.py`, `.toml`, and `.tsv` files under `crates/` match the entry manifest; no added or missing files. The six other paths in the preserved incoming manifest (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, both bounded/benchmark scripts, and `docs/guide/evaluation.md`) also match. Tests, ownership inventories, retention fingerprints/certificates, runtime frame lists, and runtime/core production are unchanged. The flat representation, 2,000-stage contract, and callback cancellation checkpoints remain intact. Evidence: [final/source-and-binary-audit.json](../../../.temp/brief-565/final/source-and-binary-audit.json) and [pre/source-sha256.json](../../../.temp/brief-565/pre/source-sha256.json).

The rejected source experiments were:

| Candidate | File and candidate line ranges | Change and disposition |
| --- | --- | --- |
| 01 | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:59–60` | Added only `#[cold]` and `#[inline(never)]` to `eval_take_lazy_list`. Both attributes were removed by restoring the whole entry file after measurement. |
| 02 | `crates/rum-stdlib/src/lazy.rs:248,255,261,268,274,281` | Moved each normalized bound into `session.evaluate`, passing only the two remaining retained values on suspension. Restored the entry file after measurement. |
| 02 | `crates/rum-stdlib/src/lazy/table_continuation.rs:32,37–43,52–67,106,121` | Cold `suspend_bounds` rebuilt the pending normalized bound from the held original arguments, using new `normalized_bound` only on suspension; adjusted its two resume call sites. Restored the entry file after measurement. |

Exact before/after spans, including insertion coordinates, are in [final/candidate-source-changes.json](../../../.temp/brief-565/final/candidate-source-changes.json). Candidate snapshots and manifests are in `candidate-01/source/`, `candidate-02/source/`, and each `source-sha256.json`. `candidate-02/source-change.diff` is a Python `difflib` output. No frame type or owning edge was added. Candidate 01 reached 21,163 characters including tests (17,544 implementation characters), the pre-existing split-plan size range; this narrow, reverted experiment did not reorganize it. Candidate 02 kept the facade at 14,693 characters and its cold continuation module at 4,338; no size debt remains from this brief.

Authored scratch Python tooling (all under `.temp/brief-565/`; complete-file ranges):

| File | Lines | Purpose |
| --- | ---: | --- |
| `attribute.py` | 1–53 | Collect matched scaled Callgrind/perf profiles and annotations. |
| `base-measure.py` | 1–10 | Orchestrate the one fresh base panel. |
| `build-candidate.py` | 1–23 | Keep candidate binaries, hashes and source snapshots. |
| `build-report.py` | 1–217 | Assemble evidence, command transcripts and this five-part report. |
| `check-panel.py` | 1–16 | Check all expected workload values before measuring a build. |
| `compare-metrics.py` | 1–40 | Compute both sample deltas and the either-sample threshold gate. |
| `finish-maintenance.py` | 1–39 | Replay local fingerprints and both feature-set certificate cells. |
| `incoming-measure.py` | 1–10 | Orchestrate the fresh incoming panel. |
| `measure-candidate.py` | 1–17 | Run candidate screens and disjoint full-panel remainders. |
| `metric-panel.py` | 1–47 | Save raw allocation/instruction processes before parsing N/2N counts. |
| `reconcile-ownership.py` | 1–76 | Reconcile all ownership partitions by identity, stopping on forbidden drift. |
| `reduce-attribution.py` | 1–51 | Reduce raw self/inclusive costs and calibration. |
| `run.py` | 1–21 | Bound, serialize and capture command environment, raw log, status and elapsed time. |
| `verify.py` | 1–15 | Run the six mandatory commands serially on restored source. |

Generated `.rum` workloads are copies or explicitly scaled attribution inputs; their results, raw metric/profile outputs, copied binaries, source snapshots and report transcripts are evidence artifacts rather than production edits. Their paths and commands are indexed in the process manifest in §4.

Measured binaries were copied to fixed paths before use and never overwritten. The pre-G2 pair was copied from `.temp/brief-563/base/` without rebuilding. The incoming pair was copied from `.temp/brief-564/candidate/` after checking all 1,211 existing crate source files against its manifest, with no added/missing files. Final hashes of both originals and all copies still match:

| Build | Binary | SHA-256 |
| --- | --- | --- |
| base | `rum-cli` | `c08b67e8f53c9651c8d139e8c39bdb392320c82376b0c4eec7f03c26397c80cc` |
| base | `rum-cli-alloc` | `4285dd8a3ccded49ae769b166b04282057cc2a6a0ceb67c1c9d81202b561ac03` |
| incoming | `rum-cli` | `75483887d622bf73959de3414e247d4f2669c9ca8f725feefedecc7a610b8ca9` |
| incoming | `rum-cli-alloc` | `de938e06c0ad0a6843ca9d03df5be8c733d614db5a0f0ea6cddbfcaaaa604b43` |
| candidate-01 | `rum-cli` | `e6b8ee69a05a93e5aef01fd1d3b34b600e230ae8962bae0667077dee8b977332` |
| candidate-01 | `rum-cli-alloc` | `7a25768b2a257d89e7c9191988e722c4822ace3393a8ecb1785ccb61146bfc75` |
| candidate-02 | `rum-cli` | `e11e4883d953a7fd78f824546c223c0beb277dd57b1e523126aacf27fe7f7ddd` |
| candidate-02 | `rum-cli-alloc` | `47d57d059e35620712d9906ba8ba7f703726bba5a6012d709650b068d37a8af7` |

Candidate 01 used separate fresh default and alloc target directories. Candidate 02 used a fresh default target and then the incremental alloc feature build in that candidate directory, copying the default binary before the alloc build. Both default hashes and both alloc hashes differ from incoming and from one another; no stale build was measured. Provenance: [base/source-provenance.json](../../../.temp/brief-565/base/source-provenance.json), [incoming/source-provenance.json](../../../.temp/brief-565/incoming/source-provenance.json).

### 2. Attribution — pre-G2 against incoming, before editing

All 29 freshly regressing labels were profiled before the first source edit. There are 116 Callgrind runs and 116 matching hardware calibration runs: base/incoming × N/2N × 29 labels. Each command used the same binary and exact `.rum` file for its Callgrind/perf pair, was pinned to CPU 2, had stdin `/dev/null`, cleared `RUM_PROFILE_*`, disabled debuginfod networking, and was bounded at 150 seconds. Each exited 0 with `Null`; 232 self/inclusive annotation commands also completed. Raw profiles, command files, stdout/stderr and annotations live under `attribution/<label>/`.

For bounded attribution, the unique outer repetition count was scaled to N=1,000 (2N=2,000), except `lazy-depth-fold-through-source-64`, which used N=10/20. Workload setup and measured statement were otherwise unchanged. These are attribution runs, not substitutions for the full acceptance panel: that panel retained its original N=20,000 for shallow workloads and N=1,000 for depth workloads. Calibration was against these exact scaled files, not another panel run. The original panel result checks passed before profiling.

The reducer checked that summed self costs equal every Callgrind program total. Self and inclusive slopes below are `(cost(2N)-cost(N))/N`. The old recursive profile can charge one instruction through many callers, so inclusive costs can exceed program totals and must not be summed. Compiler clones with apostrophe suffixes and disappeared symbols are kept as recorded; inlining can move their attribution. All function rows are available in [attribution/differences.tsv](../../../.temp/brief-565/attribution/differences.tsv) and the complete profiles in [attribution/parsed-profiles.json](../../../.temp/brief-565/attribution/parsed-profiles.json).

Calibration `(Callgrind/perf-1)×100` spans -0.087248% to +0.575780% on raw totals, and -0.593709% to +0.347059% on per-iteration slopes. This spread limits claims about small residuals. In particular, the scaled profiles do not reproduce every small positive full-panel delta. Acceptance and the stop-loss use only the prescribed two full hardware samples; no Callgrind count was substituted or favorable sample selected.

**Function evidence and the three hypotheses.** Ordinary `Take` has +308.976 Callgrind instructions per iteration overall, while `eval_take` self falls 101→100 and its inclusive cost rises 27,144.119→27,455.273. `IntoIter::try_fold` appears with 496 self instructions, `from_iter_in_place` moves 1,344→1,056, and `try_process'2` moves 210→455. This places the observed change in compiled iterator/selector callees; it does not establish a cost in the inlined lazy predicate. The cold lazy-branch experiment did not remove the regression. Direct changes to ordinary selector/collection logic would exceed this brief; none were made.

For both `LazyTable` construction forms, `Expr::clone` adds 117 self instructions and `Expr` drop adds 51. `eval_lazy_table` and the newly separated `finish_lazy_table` also redistribute costs. This motivated moving bound recovery into the cold suspension helper. Candidate 02 brought the full constructor within threshold in both samples, but the short constructor still failed one sample and other labels newly crossed the gate.

For source-only range `Take`, old `PullStepper::next` self 1,200 is replaced by source work 1,176 plus `drive_pull` 608 and `drive_items` 473; descriptor/realization shape also changes. Range `Normal` has `drive_pull` 1,052 and `drive_items` 421. The source work remains similar while the new driver layers add ordinary-path work. Combinatorial, callback and composed depth families show the corresponding driver/descriptor transitions in the rows below. Depth-drop-8 also has `Symbol::is_system_name` 1,323→1,855. Depth-fold-64 includes cancellation checks 4→8,324; those checkpoints are required semantics and were not candidates for removal. Hypothesis 3 was attributed, but no source experiment was attempted after the required second non-progress stop.

**Raw-total calibration for every regressing label.** Counts below are the recorded integer totals; percentage formatting is rounded only for display.

| Label | Build | N | Callgrind total | perf total | Difference |
| --- | --- | ---: | ---: | ---: | ---: |
| `lazy-depth-drop-all-one-64` | base | 1000 | 350576638 | 350516121 | +0.017265% |
| `lazy-depth-drop-all-one-64` | base | 2000 | 695746603 | 695682531 | +0.009210% |
| `lazy-depth-drop-all-one-64` | incoming | 1000 | 354019156 | 353938186 | +0.022877% |
| `lazy-depth-drop-all-one-64` | incoming | 2000 | 702633016 | 702521975 | +0.015806% |
| `lazy-depth-drop-all-one-8` | base | 1000 | 52167479 | 52106730 | +0.116586% |
| `lazy-depth-drop-all-one-8` | base | 2000 | 100415243 | 100354251 | +0.060777% |
| `lazy-depth-drop-all-one-8` | incoming | 1000 | 53412654 | 53326648 | +0.161281% |
| `lazy-depth-drop-all-one-8` | incoming | 2000 | 102900386 | 102791267 | +0.106156% |
| `lazy-depth-fold-through-source-64` | base | 10 | 347128503 | 347046924 | +0.023507% |
| `lazy-depth-fold-through-source-64` | base | 20 | 686578293 | 686466524 | +0.016282% |
| `lazy-depth-fold-through-source-64` | incoming | 10 | 347352726 | 347280945 | +0.020669% |
| `lazy-depth-fold-through-source-64` | incoming | 20 | 687060965 | 686963549 | +0.014181% |
| `lazy-depth-fold-through-source-8` | base | 1000 | 604759899 | 604724357 | +0.005877% |
| `lazy-depth-fold-through-source-8` | base | 2000 | 1206583193 | 1206553223 | +0.002484% |
| `lazy-depth-fold-through-source-8` | incoming | 1000 | 604964680 | 604885587 | +0.013076% |
| `lazy-depth-fold-through-source-8` | incoming | 2000 | 1206954478 | 1206925879 | +0.002370% |
| `lazy-depth-map-build-8` | base | 1000 | 23423907 | 23350230 | +0.315530% |
| `lazy-depth-map-build-8` | base | 2000 | 42879150 | 42810185 | +0.161095% |
| `lazy-depth-map-build-8` | incoming | 1000 | 23551747 | 23467162 | +0.360440% |
| `lazy-depth-map-build-8` | incoming | 2000 | 43153442 | 43039162 | +0.265526% |
| `lazy-depth-map-first-8` | base | 1000 | 135588616 | 135697342 | -0.080124% |
| `lazy-depth-map-first-8` | base | 2000 | 267226207 | 267168357 | +0.021653% |
| `lazy-depth-map-first-8` | incoming | 1000 | 135651482 | 135559730 | +0.067684% |
| `lazy-depth-map-first-8` | incoming | 2000 | 267363690 | 267597163 | -0.087248% |
| `lazy-depth-nest-map-8` | base | 1000 | 146324826 | 146265021 | +0.040888% |
| `lazy-depth-nest-map-8` | base | 2000 | 288690441 | 288645443 | +0.015589% |
| `lazy-depth-nest-map-8` | incoming | 1000 | 146624992 | 146373873 | +0.171560% |
| `lazy-depth-nest-map-8` | incoming | 2000 | 288961661 | 288853330 | +0.037504% |
| `lazy-depth-select-restart-256` | base | 1000 | 3689664016 | 3689655888 | +0.000220% |
| `lazy-depth-select-restart-256` | base | 2000 | 7369149262 | 7368696846 | +0.006140% |
| `lazy-depth-select-restart-256` | incoming | 1000 | 3690249188 | 3678740373 | +0.312847% |
| `lazy-depth-select-restart-256` | incoming | 2000 | 7347876026 | 7347141551 | +0.009997% |
| `lazy-depth-select-restart-8` | base | 1000 | 183755291 | 183676669 | +0.042805% |
| `lazy-depth-select-restart-8` | base | 2000 | 363518310 | 363396759 | +0.033449% |
| `lazy-depth-select-restart-8` | incoming | 1000 | 183913666 | 183826643 | +0.047340% |
| `lazy-depth-select-restart-8` | incoming | 2000 | 363332073 | 363227936 | +0.028670% |
| `lazy-depth-take-map-64` | base | 1000 | 1037503019 | 1037400028 | +0.009928% |
| `lazy-depth-take-map-64` | base | 2000 | 2068868125 | 2068682652 | +0.008966% |
| `lazy-depth-take-map-64` | incoming | 1000 | 1038122798 | 1038052134 | +0.006807% |
| `lazy-depth-take-map-64` | incoming | 2000 | 2070167356 | 2069913828 | +0.012248% |
| `lazy-depth-take-map-8` | base | 1000 | 145474862 | 145415513 | +0.040813% |
| `lazy-depth-take-map-8` | base | 2000 | 286916942 | 286850270 | +0.023243% |
| `lazy-depth-take-map-8` | incoming | 1000 | 146327755 | 146248915 | +0.053908% |
| `lazy-depth-take-map-8` | incoming | 2000 | 288623806 | 288510465 | +0.039285% |
| `lazy-drop-realize-small` | base | 1000 | 194977079 | 194943235 | +0.017361% |
| `lazy-drop-realize-small` | base | 2000 | 386700107 | 386535785 | +0.042511% |
| `lazy-drop-realize-small` | incoming | 1000 | 196206163 | 196165618 | +0.020669% |
| `lazy-drop-realize-small` | incoming | 2000 | 389063038 | 388916637 | +0.037643% |
| `lazy-first-small` | base | 1000 | 45159129 | 45102183 | +0.126260% |
| `lazy-first-small` | base | 2000 | 86709310 | 86560809 | +0.171557% |
| `lazy-first-small` | incoming | 1000 | 45445962 | 45335199 | +0.244320% |
| `lazy-first-small` | incoming | 2000 | 87167973 | 87082382 | +0.098287% |
| `lazy-foldlist-realize-small` | base | 1000 | 116908379 | 116867836 | +0.034691% |
| `lazy-foldlist-realize-small` | base | 2000 | 230246564 | 230202453 | +0.019162% |
| `lazy-foldlist-realize-small` | incoming | 1000 | 118536014 | 118458777 | +0.065202% |
| `lazy-foldlist-realize-small` | incoming | 2000 | 233713502 | 233379010 | +0.143326% |
| `lazy-map-realize-small` | base | 1000 | 182005643 | 181981393 | +0.013326% |
| `lazy-map-realize-small` | base | 2000 | 361187600 | 360543633 | +0.178610% |
| `lazy-map-realize-small` | incoming | 1000 | 183393876 | 183348596 | +0.024696% |
| `lazy-map-realize-small` | incoming | 2000 | 363418226 | 363538891 | -0.033192% |
| `lazy-nestlist-realize-small` | base | 1000 | 148707668 | 148646976 | +0.040830% |
| `lazy-nestlist-realize-small` | base | 2000 | 293846458 | 293816473 | +0.010205% |
| `lazy-nestlist-realize-small` | incoming | 1000 | 149928180 | 149870326 | +0.038603% |
| `lazy-nestlist-realize-small` | incoming | 2000 | 296374710 | 296282400 | +0.031156% |
| `lazy-part-small` | base | 1000 | 92297986 | 92218679 | +0.085999% |
| `lazy-part-small` | base | 2000 | 181122190 | 181026491 | +0.052865% |
| `lazy-part-small` | incoming | 1000 | 92937639 | 92837216 | +0.108171% |
| `lazy-part-small` | incoming | 2000 | 182356663 | 182292949 | +0.034951% |
| `lazy-permutations-take-small` | base | 1000 | 32265389 | 32192884 | +0.225221% |
| `lazy-permutations-take-small` | base | 2000 | 61158043 | 61069052 | +0.145722% |
| `lazy-permutations-take-small` | incoming | 1000 | 32695887 | 32619811 | +0.233220% |
| `lazy-permutations-take-small` | incoming | 2000 | 61966167 | 61882258 | +0.135595% |
| `lazy-range-normal-small` | base | 1000 | 57408860 | 57368715 | +0.069977% |
| `lazy-range-normal-small` | base | 2000 | 111288888 | 111204479 | +0.075904% |
| `lazy-range-normal-small` | incoming | 1000 | 58580608 | 58505971 | +0.127572% |
| `lazy-range-normal-small` | incoming | 2000 | 113602028 | 113568131 | +0.029847% |
| `lazy-range-take-small` | base | 1000 | 45588220 | 45549908 | +0.084110% |
| `lazy-range-take-small` | base | 2000 | 87561713 | 87496707 | +0.074295% |
| `lazy-range-take-small` | incoming | 1000 | 46479609 | 46416317 | +0.136357% |
| `lazy-range-take-small` | incoming | 2000 | 89395783 | 89311006 | +0.094923% |
| `lazy-select-realize-small` | base | 1000 | 117911149 | 117826231 | +0.072071% |
| `lazy-select-realize-small` | base | 2000 | 232308950 | 232165238 | +0.061901% |
| `lazy-select-realize-small` | incoming | 1000 | 119113108 | 118998831 | +0.096032% |
| `lazy-select-realize-small` | incoming | 2000 | 234662293 | 234682875 | -0.008770% |
| `lazy-select-scan-small` | base | 1000 | 801081861 | 801033538 | +0.006033% |
| `lazy-select-scan-small` | base | 2000 | 1601009560 | 1601009896 | -0.000021% |
| `lazy-select-scan-small` | incoming | 1000 | 802805378 | 802737348 | +0.008475% |
| `lazy-select-scan-small` | incoming | 2000 | 1604575283 | 1604593129 | -0.001112% |
| `lazy-subsets-take-small` | base | 1000 | 29544777 | 29466742 | +0.264824% |
| `lazy-subsets-take-small` | base | 2000 | 55763827 | 55619692 | +0.259144% |
| `lazy-subsets-take-small` | incoming | 1000 | 29973568 | 29876598 | +0.324568% |
| `lazy-subsets-take-small` | incoming | 2000 | 56478755 | 56391858 | +0.154095% |
| `lazy-table-construct-full` | base | 1000 | 27680402 | 27632267 | +0.174199% |
| `lazy-table-construct-full` | base | 2000 | 51764142 | 51722128 | +0.081230% |
| `lazy-table-construct-full` | incoming | 1000 | 27867963 | 27779382 | +0.318873% |
| `lazy-table-construct-full` | incoming | 2000 | 52056316 | 52028927 | +0.052642% |
| `lazy-table-construct-short` | base | 1000 | 27224042 | 27140973 | +0.306065% |
| `lazy-table-construct-short` | base | 2000 | 50784748 | 50721834 | +0.124037% |
| `lazy-table-construct-short` | incoming | 1000 | 27364083 | 27303129 | +0.223249% |
| `lazy-table-construct-short` | incoming | 2000 | 51104966 | 51047645 | +0.112289% |
| `lazy-table-realize-small` | base | 1000 | 120513024 | 120449017 | +0.053140% |
| `lazy-table-realize-small` | base | 2000 | 237433351 | 237302417 | +0.055176% |
| `lazy-table-realize-small` | incoming | 1000 | 121826777 | 121752895 | +0.060682% |
| `lazy-table-realize-small` | incoming | 2000 | 239947026 | 239852134 | +0.039563% |
| `lazy-transparent-adapters-small` | base | 1000 | 185917888 | 185847644 | +0.037797% |
| `lazy-transparent-adapters-small` | base | 2000 | 368167960 | 368136015 | +0.008677% |
| `lazy-transparent-adapters-small` | incoming | 1000 | 186734899 | 186513690 | +0.118602% |
| `lazy-transparent-adapters-small` | incoming | 2000 | 369539598 | 369509948 | +0.008024% |
| `lazy-tuples-take-small` | base | 1000 | 30815212 | 30638800 | +0.575780% |
| `lazy-tuples-take-small` | base | 2000 | 57992707 | 57978614 | +0.024307% |
| `lazy-tuples-take-small` | incoming | 1000 | 31098213 | 31035537 | +0.201949% |
| `lazy-tuples-take-small` | incoming | 2000 | 58776630 | 58738178 | +0.065463% |
| `take-multispec-small` | base | 1000 | 55467615 | 55379855 | +0.158469% |
| `take-multispec-small` | base | 2000 | 107579287 | 107464456 | +0.106855% |
| `take-multispec-small` | incoming | 1000 | 55782056 | 55696946 | +0.152809% |
| `take-multispec-small` | incoming | 2000 | 108202704 | 108083807 | +0.110004% |

**Per-iteration calibration.**

| Label | Build | Callgrind / iteration | perf / iteration | Difference |
| --- | --- | ---: | ---: | ---: |
| `lazy-depth-drop-all-one-64` | base | 345169.965 | 345166.41 | +0.001030% |
| `lazy-depth-drop-all-one-64` | incoming | 348613.86 | 348583.789 | +0.008627% |
| `lazy-depth-drop-all-one-8` | base | 48247.764 | 48247.521 | +0.000504% |
| `lazy-depth-drop-all-one-8` | incoming | 49487.732 | 49464.619 | +0.046726% |
| `lazy-depth-fold-through-source-64` | base | 33944979 | 33941960 | +0.008895% |
| `lazy-depth-fold-through-source-64` | incoming | 33970823.9 | 33968260.4 | +0.007547% |
| `lazy-depth-fold-through-source-8` | base | 601823.294 | 601828.866 | -0.000926% |
| `lazy-depth-fold-through-source-8` | incoming | 601989.798 | 602040.292 | -0.008387% |
| `lazy-depth-map-build-8` | base | 19455.243 | 19459.955 | -0.024214% |
| `lazy-depth-map-build-8` | incoming | 19601.695 | 19572 | +0.151722% |
| `lazy-depth-map-first-8` | base | 131637.591 | 131471.015 | +0.126702% |
| `lazy-depth-map-first-8` | incoming | 131712.208 | 132037.433 | -0.246313% |
| `lazy-depth-nest-map-8` | base | 142365.615 | 142380.422 | -0.010400% |
| `lazy-depth-nest-map-8` | incoming | 142336.669 | 142479.457 | -0.100217% |
| `lazy-depth-select-restart-256` | base | 3679485.246 | 3679040.958 | +0.012076% |
| `lazy-depth-select-restart-256` | incoming | 3657626.838 | 3668401.178 | -0.293707% |
| `lazy-depth-select-restart-8` | base | 179763.019 | 179720.09 | +0.023887% |
| `lazy-depth-select-restart-8` | incoming | 179418.407 | 179401.293 | +0.009540% |
| `lazy-depth-take-map-64` | base | 1031365.106 | 1031282.624 | +0.007998% |
| `lazy-depth-take-map-64` | incoming | 1032044.558 | 1031861.694 | +0.017722% |
| `lazy-depth-take-map-8` | base | 141442.08 | 141434.757 | +0.005178% |
| `lazy-depth-take-map-8` | incoming | 142296.051 | 142261.55 | +0.024252% |
| `lazy-drop-realize-small` | base | 191723.028 | 191592.55 | +0.068102% |
| `lazy-drop-realize-small` | incoming | 192856.875 | 192751.019 | +0.054919% |
| `lazy-first-small` | base | 41550.181 | 41458.626 | +0.220835% |
| `lazy-first-small` | incoming | 41722.011 | 41747.183 | -0.060296% |
| `lazy-foldlist-realize-small` | base | 113338.185 | 113334.617 | +0.003148% |
| `lazy-foldlist-realize-small` | incoming | 115177.488 | 114920.233 | +0.223855% |
| `lazy-map-realize-small` | base | 179181.957 | 178562.24 | +0.347059% |
| `lazy-map-realize-small` | incoming | 180024.35 | 180190.295 | -0.092094% |
| `lazy-nestlist-realize-small` | base | 145138.79 | 145169.497 | -0.021153% |
| `lazy-nestlist-realize-small` | incoming | 146446.53 | 146412.074 | +0.023534% |
| `lazy-part-small` | base | 88824.204 | 88807.812 | +0.018458% |
| `lazy-part-small` | incoming | 89419.024 | 89455.733 | -0.041036% |
| `lazy-permutations-take-small` | base | 28892.654 | 28876.168 | +0.057092% |
| `lazy-permutations-take-small` | incoming | 29270.28 | 29262.447 | +0.026768% |
| `lazy-range-normal-small` | base | 53880.028 | 53835.764 | +0.082220% |
| `lazy-range-normal-small` | incoming | 55021.42 | 55062.16 | -0.073989% |
| `lazy-range-take-small` | base | 41973.493 | 41946.799 | +0.063638% |
| `lazy-range-take-small` | incoming | 42916.174 | 42894.689 | +0.050088% |
| `lazy-select-realize-small` | base | 114397.801 | 114339.007 | +0.051421% |
| `lazy-select-realize-small` | incoming | 115549.185 | 115684.044 | -0.116575% |
| `lazy-select-scan-small` | base | 799927.699 | 799976.358 | -0.006083% |
| `lazy-select-scan-small` | incoming | 801769.905 | 801855.781 | -0.010710% |
| `lazy-subsets-take-small` | base | 26219.05 | 26152.95 | +0.252744% |
| `lazy-subsets-take-small` | incoming | 26505.187 | 26515.26 | -0.037989% |
| `lazy-table-construct-full` | base | 24083.74 | 24089.861 | -0.025409% |
| `lazy-table-construct-full` | incoming | 24188.353 | 24249.545 | -0.252343% |
| `lazy-table-construct-short` | base | 23560.706 | 23580.861 | -0.085472% |
| `lazy-table-construct-short` | incoming | 23740.883 | 23744.516 | -0.015300% |
| `lazy-table-realize-small` | base | 116920.327 | 116853.4 | +0.057274% |
| `lazy-table-realize-small` | incoming | 118120.249 | 118099.239 | +0.017790% |
| `lazy-transparent-adapters-small` | base | 182250.072 | 182288.371 | -0.021010% |
| `lazy-transparent-adapters-small` | incoming | 182804.699 | 182996.258 | -0.104679% |
| `lazy-tuples-take-small` | base | 27177.495 | 27339.814 | -0.593709% |
| `lazy-tuples-take-small` | incoming | 27678.417 | 27702.641 | -0.087443% |
| `take-multispec-small` | base | 52111.672 | 52084.601 | +0.051975% |
| `take-multispec-small` | incoming | 52420.648 | 52386.861 | +0.064495% |

**Functions for every residual label.** Each table includes the program total, the eight largest absolute self-cost changes, and relevant driver/descriptor, clone/drop, cancellation or ordinary-`Take` entries where they moved. These selected functions are attribution evidence rather than an additive explanation; the raw reducer table contains every function.

`lazy-range-take-small` — full hardware panel +2.132071% / +2.025830%; Callgrind total change 942.681 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 41973.493 | 42916.174 | 942.681 | 41973.493 | 42916.174 |
| `rum_stdlib::lazy::PullStepper::next` | 1200 | 0 | -1200 | 19835.355 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 1176 | 1176 | 0 | 19811.098 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 608 | 608 | 0 | 20419.098 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 473 | 473 | 0 | 21253.338 |
| `rum_stdlib::lazy::realize` | 376 | 89 | -287 | 21979.599 | 22869.347 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 122 | 122 | 0 | 790 |
| `rum_stdlib::lazy::stepper_from_descriptor` | 74 | 0 | -74 | 742 | 0 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::state::PullMachine>` | 0 | 63 | 63 | 0 | 165 |

`lazy-range-normal-small` — full hardware panel +2.330816% / +2.220177%; Callgrind total change 1141.392 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 53880.028 | 55021.42 | 1141.392 | 53880.028 | 55021.42 |
| `rum_stdlib::lazy::PullStepper::next` | 1338 | 0 | -1338 | 19976.76 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 1176 | 1176 | 0 | 19813.063 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1052 | 1052 | 0 | 20865.063 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 421 | 421 | 0 | 21644.687 |
| `rum_stdlib::lazy::realize` | 376 | 89 | -287 | 23384.91 | 24501.86 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 196 | 196 | 0 | 2018.072 |
| `rum_stdlib::lazy::stepper_from_descriptor` | 128 | 0 | -128 | 1923.832 | 0 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 79 | 79 | 0 | 148.24 |

`lazy-map-realize-small` — full hardware panel +0.751169% / +0.861222%; Callgrind total change 842.393 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 179181.957 | 180024.35 | 842.393 | 179181.957 | 180024.35 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1708 | 1708 | 0 | 134191.546 |
| `rum_stdlib::lazy::PullStepper::next'2` | 1200 | 0 | -1200 | 19825.463 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 1176 | 1176 | 0 | 19801.644 |
| `rum_stdlib::lazy::PullStepper::next` | 874 | 0 | -874 | 133438.699 | 0 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 421 | 421 | 0 | 134977.625 |
| `rum_core::expr::Symbol::system` | 2051.193 | 1636.879 | -414.314 | 11822.075 | 11281.305 |
| `rum_stdlib::lazy::realize` | 376 | 89 | -287 | 137237.34 | 138604.754 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 259 | 259 | 0 | 2713.49 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 222 | 222 | 0 | 608.732 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 32 | 64 | 32 | 32 | 64 |

`lazy-select-realize-small` — full hardware panel +0.761379% / +0.974923%; Callgrind total change 1151.384 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 114397.801 | 115549.185 | 1151.384 | 114397.801 | 115549.185 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1424 | 1424 | 0 | 70266.956 |
| `rum_stdlib::lazy::PullStepper::next'2` | 1200 | 0 | -1200 | 19856.18 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 1176 | 1176 | 0 | 19832.331 |
| `rum_stdlib::lazy::PullStepper::next` | 846 | 0 | -846 | 69618.709 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 265 | 265 | 0 | 2783.258 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 237 | 237 | 0 | 70633.801 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 229 | 229 | 0 | 625.62 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 194 | 0 | -194 | 1902.552 | 0 |
| `rum_stdlib::lazy::realize` | 244 | 89 | -155 | 73127.14 | 74305.576 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 32 | 64 | 32 | 32 | 64 |

`lazy-select-scan-small` — full hardware panel +0.231477% / +0.221403%; Callgrind total change 1842.206 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 799927.699 | 801769.905 | 1842.206 | 799927.699 | 801769.905 |
| `rum_stdlib::lazy::PullStepper::next'2` | 4800 | 0 | -4800 | 79533.403 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 4704 | 4704 | 0 | 79429.877 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 4092 | 4092 | 0 | 768645.085 |
| `rum_stdlib::lazy::PullStepper::next` | 2536 | 0 | -2536 | 767028.939 | 0 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 253 | 253 | 0 | 769023.239 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 191 | 191 | 0 | 1247.84 |
| `rum_stdlib::lazy::realize` | 236 | 89 | -147 | 769317.062 | 771171.803 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 128 | 256 | 128 | 128 | 256 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 79 | 79 | 0 | 148.24 |

`lazy-nestlist-realize-small` — full hardware panel +0.771399% / +0.852421%; Callgrind total change 1307.74 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 145138.79 | 146446.53 | 1307.74 | 145138.79 | 146446.53 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1052 | 1052 | 0 | 114096.097 |
| `rum_stdlib::lazy::PullStepper::next` | 938 | 0 | -938 | 113026.786 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 816 | 816 | 0 | 113044.097 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 421 | 421 | 0 | 114882.049 |
| `rum_stdlib::lazy::realize` | 376 | 89 | -287 | 115826.634 | 117190.809 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 199 | 199 | 0 | 1529.8 |
| `rum_stdlib::lazy::stepper_from_descriptor` | 128 | 0 | -128 | 1432.56 | 0 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 79 | 79 | 0 | 148.24 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 32 | 64 | 32 | 32 | 64 |

`lazy-foldlist-realize-small` — full hardware panel +1.416905% / +1.375035%; Callgrind total change 1839.303 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 113338.185 | 115177.488 | 1839.303 | 113338.185 | 115177.488 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1879 | 1879 | 0 | 68143.756 |
| `rum_stdlib::lazy::PullStepper::next'2` | 1050 | 0 | -1050 | 17365.48 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 1029 | 1029 | 0 | 17352.99 |
| `rum_stdlib::lazy::PullStepper::next` | 995 | 0 | -995 | 67030.769 | 0 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 421 | 421 | 0 | 68921.142 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 294 | 294 | 0 | 2973.672 |
| `rum_stdlib::lazy::realize` | 376 | 89 | -287 | 70984.235 | 72815.286 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 250 | 250 | 0 | 700.62 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 28 | 56 | 28 | 28 | 56 |

`lazy-drop-realize-small` — full hardware panel +0.590184% / +0.482540%; Callgrind total change 1133.847 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 191723.028 | 192856.875 | 1133.847 | 191723.028 | 192856.875 |
| `rum_stdlib::lazy::PullStepper::next'2` | 1936 | 0 | -1936 | 153307.512 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1660 | 1660 | 0 | 134478.826 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 1176 | 1176 | 0 | 19789.033 |
| `rum_stdlib::lazy::PullStepper::next` | 342 | 0 | -342 | 133904.509 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 340 | 340 | 0 | 4041.233 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 322 | 0 | -322 | 4145.195 | 0 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 306 | 306 | 0 | 822.621 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 237 | 237 | 0 | 134846.794 |
| `rum_stdlib::lazy::realize` | 244 | 89 | -155 | 138639.864 | 139789.871 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 32 | 64 | 32 | 32 | 64 |

`lazy-first-small` — full hardware panel +0.813181% / +0.505455%; Callgrind total change 171.83 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 41550.181 | 41722.011 | 171.83 | 41550.181 | 41722.011 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 185 | 185 | 0 | 1166.936 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 163 | 163 | 0 | 11363.623 |
| `rum_stdlib::lazy::PullStepper::next'2` | 150 | 0 | -150 | 2476.994 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 147 | 147 | 0 | 2473.994 |
| `rum_stdlib::lazy::stepper_from_descriptor` | 115 | 0 | -115 | 1084.877 | 0 |
| `rum_stdlib::lazy::PullStepper::next` | 92 | 0 | -92 | 11304.878 | 0 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 11576.155 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 79 | 79 | 0 | 148.24 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 13397.081 | 13634.097 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 4 | 8 | 4 | 4 | 8 |

`lazy-part-small` — full hardware panel +0.704711% / +0.530184%; Callgrind total change 594.82 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 88824.204 | 89419.024 | 594.82 | 88824.204 | 89419.024 |
| `rum_stdlib::lazy::PullStepper::next'2` | 726 | 0 | -726 | 41275.042 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 510 | 510 | 0 | 34142.185 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 441 | 441 | 0 | 7415.396 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 266 | 266 | 0 | 2831.903 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 222 | 222 | 0 | 597.52 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 189 | 0 | -189 | 2006.544 | 0 |
| `rum_stdlib::lazy::stepper_from_descriptor` | 133 | 0 | -133 | 2422.234 | 0 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 34360.437 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 37436.829 | 38080.724 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 12 | 24 | 12 | 12 | 24 |

`lazy-transparent-adapters-small` — full hardware panel +0.496487% / +0.515740%; Callgrind total change 554.627 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 182250.072 | 182804.699 | 554.627 | 182250.072 | 182804.699 |
| `rum_stdlib::lazy::PullStepper::next'2` | 1794 | 0 | -1794 | 214591.758 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1548 | 1548 | 0 | 100106.744 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 882 | 882 | 0 | 14851.919 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 578 | 0 | -578 | 13214.147 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 495 | 495 | 0 | 6779.812 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 460 | 460 | 0 | 1125.217 |
| `rum_core::expr::Symbol::is_system_name` | 4045 | 4294 | 249 | 6532 | 6879 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::PullStepper>'2` | 187 | 0 | -187 | 1301 | 0 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 100305.744 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 107436.725 | 107982.556 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 24 | 48 | 24 | 24 | 48 |

`lazy-table-construct-short` — full hardware panel +0.985015% / +0.925182%; Callgrind total change 180.177 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 23560.706 | 23740.883 | 180.177 | 23560.706 | 23740.883 |
| `rum_stdlib::lazy::finish_lazy_table` | 0 | 507 | 507 | 0 | 14479.591 |
| `rum_stdlib::lazy::lazy_table_count` | 238 | 0 | -238 | 6756.6 | 0 |
| `rum_stdlib::lazy::eval_lazy_table` | 354 | 145 | -209 | 15226.591 | 15410.591 |
| `<rum_core::expr::Expr as core::clone::Clone>::clone` | 552 | 669 | 117 | 552 | 669 |
| `core::ptr::drop_in_place<rum_core::expr::Expr>` | 106 | 157 | 51 | 774 | 825 |
| `core::ptr::drop_in_place<rum_core::expr::Rational>` | 51 | 0 | -51 | 177 | 0 |
| `rum_core::expr::Symbol::system` | 301.866 | 326.196 | 24.33 | 4952.227 | 4978.023 |
| `num_bigint::biguint::convert::<impl core::convert::From<u64> for num_bigint::biguint::BigUint>::from` | 21 | 0 | -21 | 128 | 0 |

`lazy-table-construct-full` — full hardware panel +1.007616% / +0.615421%; Callgrind total change 104.613 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 24083.74 | 24188.353 | 104.613 | 24083.74 | 24188.353 |
| `rum_stdlib::lazy::finish_lazy_table` | 0 | 507 | 507 | 0 | 15103.575 |
| `rum_stdlib::lazy::lazy_table_count` | 238 | 0 | -238 | 7402.525 | 0 |
| `rum_stdlib::lazy::eval_lazy_table` | 356 | 147 | -209 | 16004.575 | 16114.575 |
| `<rum_core::expr::Expr as core::clone::Clone>::clone` | 574 | 691 | 117 | 574 | 691 |
| `core::ptr::drop_in_place<rum_core::expr::Expr>` | 89 | 140 | 51 | 757 | 808 |
| `core::ptr::drop_in_place<rum_core::expr::Rational>` | 51 | 0 | -51 | 177 | 0 |
| `rum_core::expr::intern_symbol_text` | 418.885 | 391.567 | -27.318 | 3704.05 | 3677.938 |
| `rum_core::expr::Symbol::system` | 326.771 | 302.427 | -24.344 | 5002.717 | 4952.146 |

`lazy-table-realize-small` — full hardware panel +1.159520% / +1.074176%; Callgrind total change 1199.922 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 116920.327 | 118120.249 | 1199.922 | 116920.327 | 118120.249 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1093 | 1093 | 0 | 84364.291 |
| `rum_stdlib::lazy::PullStepper::next'2` | 750 | 0 | -750 | 12379.351 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 735 | 735 | 0 | 12364.274 |
| `rum_stdlib::lazy::PullStepper::next` | 559 | 0 | -559 | 83625.702 | 0 |
| `rum_stdlib::lazy::finish_lazy_table` | 0 | 507 | 507 | 0 | 14572.932 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 286 | 286 | 0 | 85012.723 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 259 | 259 | 0 | 2716.116 |
| `rum_stdlib::lazy::lazy_table_count` | 238 | 0 | -238 | 6764.432 | 0 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 222 | 222 | 0 | 602.458 |
| `rum_stdlib::lazy::realize` | 280 | 89 | -191 | 87510.712 | 88624.395 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 20 | 40 | 20 | 20 | 40 |

`lazy-permutations-take-small` — full hardware panel +1.374572% / +1.262690%; Callgrind total change 377.626 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 28892.654 | 29270.28 | 377.626 | 28892.654 | 29270.28 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 228 | 228 | 0 | 4040.264 |
| `rum_stdlib::lazy::PullStepper::next` | 204 | 0 | -204 | 3818.341 | 0 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 199 | 199 | 0 | 4374.397 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 198 | 198 | 0 | 3812.264 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 196 | 196 | 0 | 986.064 |
| `rum_stdlib::lazy::stepper_from_descriptor` | 152 | 0 | -152 | 942.064 | 0 |
| `rum_stdlib::lazy::realize` | 202 | 89 | -113 | 5776.807 | 6201.73 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::state::PullMachine>` | 0 | 76 | 76 | 0 | 183 |

`lazy-subsets-take-small` — full hardware panel +1.431834% / +1.523624%; Callgrind total change 286.137 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 26219.05 | 26505.187 | 286.137 | 26219.05 | 26505.187 |
| `rum_stdlib::lazy::PullStepper::next` | 318 | 0 | -318 | 2188.448 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 305 | 305 | 0 | 2175.448 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 228 | 228 | 0 | 2403.448 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 199 | 199 | 0 | 2749.546 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 187 | 187 | 0 | 530.464 |
| `rum_stdlib::lazy::realize` | 202 | 89 | -113 | 3752.356 | 4150.356 |
| `<T as alloc::slice::hack::ConvertVec>::to_vec` | 105 | 0 | -105 | 313.464 | 0 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::state::PullMachine>` | 0 | 84 | 84 | 0 | 212 |

`lazy-tuples-take-small` — full hardware panel +1.446731% / +1.241760%; Callgrind total change 500.922 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 27177.495 | 27678.417 | 500.922 | 27177.495 | 27678.417 |
| `rum_stdlib::lazy::PullStepper::next` | 335 | 0 | -335 | 2999.526 | 0 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 326 | 326 | 0 | 2990.526 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 250 | 250 | 0 | 790.546 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 228 | 228 | 0 | 3218.526 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 199 | 199 | 0 | 3572.586 |
| `rum_stdlib::lazy::stepper_from_descriptor` | 126 | 0 | -126 | 781.546 | 0 |
| `rum_stdlib::lazy::realize` | 202 | 89 | -113 | 4816.947 | 5203.87 |
| `<T as alloc::slice::hack::ConvertVec>::to_vec` | 79 | 0 | -79 | 180.466 | 0 |

`take-multispec-small` — full hardware panel +0.584335% / +0.611458%; Callgrind total change 308.976 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 52111.672 | 52420.648 | 308.976 | 52111.672 | 52420.648 |
| `<alloc::vec::into_iter::IntoIter<T,A> as core::iter::traits::iterator::Iterator>::try_fold` | 0 | 496 | 496 | 0 | 1275 |
| `alloc::vec::in_place_collect::from_iter_in_place` | 1344 | 1056 | -288 | 2811 | 2939 |
| `core::iter::adapters::try_process'2` | 210 | 455 | 245 | 46941.491 | 49689.93 |
| `rum_stdlib::structure_ops::structural_head_expr` | 112 | 0 | -112 | 408 | 0 |
| `rum_stdlib::structure_ops::selectors::apply::apply_part_selectors_to_selected_shape` | 312 | 240 | -72 | 859 | 779 |
| `core::iter::adapters::try_process` | 35 | 70 | 35 | 15943.465 | 16490.572 |
| `rum_stdlib::structure_ops::selectors::apply::apply_part_selectors'2` | 176 | 168 | -8 | 411 | 403 |
| `rum_stdlib::structure_ops::selectors::apply::apply_part_selectors` | 688 | 696 | 8 | 9979.922 | 10292.076 |
| `rum_stdlib::structure_ops::sequence::take_drop::eval_take` | 101 | 100 | -1 | 27144.119 | 27455.273 |

`lazy-depth-map-build-8` — full hardware panel +0.613412% / +0.547084%; Callgrind total change 146.452 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 19455.243 | 19601.695 | 146.452 | 19455.243 | 19601.695 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 879 | 0 | -879 | 14291.221 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 626 | 626 | 0 | 3724.209 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 606 | 606 | 0 | 1259.623 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::PullStepper>'2` | 294 | 0 | -294 | 2940 | 0 |
| `rum_core::expr::Symbol::is_system_name` | 967 | 1170 | 203 | 1305 | 1641 |
| `mi_theap_malloc_aligned` | 459.311 | 313.939 | -145.372 | 983.372 | 608.513 |
| `0x00000000001b1220` | 401.218 | 534.08 | 132.862 | 401.218 | 534.08 |
| `<alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop` | 0 | 128 | 128 | 0 | 264 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 33 | 33 | 0 | 33 |
| `rum_stdlib::lazy::realize` | 92 | 89 | -3 | 4711.107 | 4822.941 |

`lazy-depth-map-first-8` — full hardware panel -0.119224% / +0.414634%; Callgrind total change 74.617 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 131637.591 | 131712.208 | 74.617 | 131637.591 | 131712.208 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 879 | 0 | -879 | 14052 | 0 |
| `rum_stdlib::lazy::PullStepper::next'2` | 782 | 0 | -782 | 402052.747 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 737 | 737 | 0 | 114084.789 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 626 | 626 | 0 | 3716.983 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 606 | 606 | 0 | 1258.983 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::PullStepper>'2` | 298 | 0 | -298 | 3140 | 0 |
| `rum_core::expr::Symbol::is_system_name` | 4125 | 4328 | 203 | 6910 | 7246 |
| `mi_theap_malloc_aligned` | 4058.898 | 3914.86 | -144.038 | 7561.136 | 7060.91 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 135 | 135 | 0 | 2070.832 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 114283.789 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 118887.283 | 119051.772 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 4 | 36 | 32 | 4 | 36 |

`lazy-depth-select-restart-8` — full hardware panel +0.227557% / -0.047594%; Callgrind total change -344.612 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 179763.019 | 179418.407 | -344.612 | 179763.019 | 179418.407 |
| `rum_stdlib::lazy::PullStepper::next'2` | 2136 | 0 | -2136 | 649712.112 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1750 | 1750 | 0 | 161228.672 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 914 | 0 | -914 | 15924.835 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 674 | 674 | 0 | 4300.09 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 655 | 655 | 0 | 1434.09 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 429 | 429 | 0 | 7042.688 |
| `rum_stdlib::logic::comparison::numeric_comparable_value_from_expr` | 364 | 0 | -364 | 4016.902 | 0 |
| `rum_stdlib::predicates::exact_numeric_value` | 0 | 364 | 364 | 0 | 4016.902 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 161454.084 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 12 | 80 | 68 | 12 | 80 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 167127.119 | 166845.974 |

`lazy-depth-drop-all-one-8` — full hardware panel +2.518332% / +2.517386%; Callgrind total change 1239.968 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 48247.764 | 49487.732 | 1239.968 | 48247.764 | 49487.732 |
| `rum_stdlib::lazy::PullStepper::next'2` | 2955 | 0 | -2955 | 183384 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 2059 | 2059 | 0 | 24079.852 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 1311 | 1311 | 0 | 21884.852 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 1005 | 0 | -1005 | 39822.704 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 770 | 770 | 0 | 11676.942 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 746 | 746 | 0 | 1966.09 |
| `rum_core::expr::Symbol::is_system_name` | 1323 | 1855 | 532 | 1752 | 2571 |
| `0x00000000001b1220` | 447.993 | 735.198 | 287.205 | 447.993 | 735.198 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 24278.852 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 35699.204 | 36904.294 |

`lazy-depth-take-map-8` — full hardware panel +0.587272% / +0.588882%; Callgrind total change 853.971 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 141442.08 | 142296.051 | 853.971 | 141442.08 | 142296.051 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 1890 | 0 | -1890 | 98097.146 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 1218 | 1218 | 0 | 13599.962 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 1209 | 1209 | 0 | 2522.281 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 1073 | 1073 | 0 | 114645.884 |
| `rum_stdlib::lazy::PullStepper::next'2` | 873 | 0 | -873 | 402888.06 | 0 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::PullStepper>'2` | 564 | 0 | -564 | 9888 | 0 |
| `rum_core::expr::Symbol::is_system_name` | 5061 | 5612 | 551 | 7998 | 8834 |
| `mi_theap_malloc_aligned` | 4901.754 | 4585.716 | -316.038 | 8684.437 | 8303.69 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 135 | 135 | 0 | 2072.612 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 114844.884 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 128897.93 | 129664.851 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 4 | 36 | 32 | 4 | 36 |

`lazy-depth-fold-through-source-8` — full hardware panel +0.107123% / +0.117739%; Callgrind total change 166.504 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 601823.294 | 601989.798 | 166.504 | 601823.294 | 601989.798 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 4982 | 4982 | 0 | 577687.255 |
| `rum_stdlib::lazy::PullStepper::next'2` | 3701 | 0 | -3701 | 1346876.354 | 0 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 1110 | 0 | -1110 | 18840 | 0 |
| `rum_stdlib::lazy::PullStepper::next` | 973 | 0 | -973 | 578783.909 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 906 | 906 | 0 | 5827.113 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 802 | 802 | 0 | 1985.113 |
| `rum_core::expr::Symbol::is_system_name` | 19062 | 19657 | 595 | 34258 | 34986 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 531 | 531 | 0 | 579019.821 |
| `rum_stdlib::lazy::realize` | 414 | 89 | -325 | 585746.075 | 586130.04 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 4 | 148 | 144 | 4 | 148 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 135 | 135 | 0 | 2071.722 |

`lazy-depth-nest-map-8` — full hardware panel +0.083343% / +0.081183%; Callgrind total change -28.946 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 142365.615 | 142336.669 | -28.946 | 142365.615 | 142336.669 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 879 | 0 | -879 | 11084 | 0 |
| `rum_stdlib::lazy::PullStepper::next'2` | 744 | 0 | -744 | 492482.001 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 737 | 737 | 0 | 125195.712 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 629 | 629 | 0 | 3376.09 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 606 | 606 | 0 | 1286.09 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::PullStepper>'2` | 274 | 0 | -274 | 2548 | 0 |
| `rum_core::expr::Symbol::is_system_name` | 4564 | 4772 | 208 | 7649 | 8012 |
| `0x00000000001b1220` | 3644.146 | 3798.897 | 154.751 | 3644.146 | 3798.897 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 102 | 102 | 0 | 13433.144 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 125394.712 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 129643.962 | 129743.802 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 4 | 40 | 36 | 4 | 40 |

`lazy-depth-drop-all-one-64` — full hardware panel +0.988889% / +0.988252%; Callgrind total change 3443.895 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 345169.965 | 348613.86 | 3443.895 | 345169.965 | 348613.86 |
| `rum_stdlib::lazy::PullStepper::next'2` | 96615 | 0 | -96615 | 13956573.376 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 85807 | 85807 | 0 | 247455.846 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 9543 | 9543 | 0 | 160560.846 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 8453 | 0 | -8453 | 2527539.558 | 0 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 5450 | 5450 | 0 | 13635.343 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 5306 | 5306 | 0 | 86564.283 |
| `rum_core::expr::Symbol::is_system_name` | 8939 | 13167 | 4228 | 11664 | 18475 |
| `0x00000000001b1220` | 2744.058 | 5327.038 | 2582.98 | 2744.058 | 5327.038 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 247654.846 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 332629.794 | 336034.135 |

`lazy-depth-take-map-64` — full hardware panel +0.087568% / +0.069033%; Callgrind total change 679.452 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 1031365.106 | 1032044.558 | 679.452 | 1031365.106 | 1032044.558 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 15498 | 0 | -15498 | 6180594.88 | 0 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 9105 | 9105 | 0 | 17142.174 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 8890 | 8890 | 0 | 101033.485 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 8017 | 8017 | 0 | 914318.618 |
| `rum_stdlib::lazy::PullStepper::next'2` | 6753 | 0 | -6753 | 28746205.736 | 0 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::PullStepper>'2` | 4372 | 0 | -4372 | 534272 | 0 |
| `rum_core::expr::Symbol::is_system_name` | 39109 | 43356 | 4247 | 62206 | 68866 |
| `mi_theap_malloc_aligned` | 36587.296 | 33920.666 | -2666.63 | 62961.981 | 58874.052 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 4 | 260 | 256 | 4 | 260 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 135 | 135 | 0 | 2071.544 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 914549.037 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 1018614.981 | 1019266.556 |

`lazy-depth-fold-through-source-64` — full hardware panel +0.066378% / +0.060974%; Callgrind total change 25844.9 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 33944979 | 33970823.9 | 25844.9 | 33944979 | 33970823.9 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 250766 | 250766 | 0 | 33900497.7 |
| `rum_stdlib::lazy::PullStepper::next'2` | 236829 | 0 | -236829 | 710271576.9 | 0 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 9398 | 0 | -9398 | 1047128 | 0 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 4 | 8324 | 8320 | 4 | 8324 |
| `rum_stdlib::lazy::PullStepper::next` | 7469 | 0 | -7469 | 33881883 | 0 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 6394 | 6394 | 0 | 39797.6 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 5954 | 5954 | 0 | 13779.6 |
| `rum_core::expr::Symbol::is_system_name` | 1041202 | 1045997 | 4795 | 1908578 | 1914570 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 3567 | 3567 | 0 | 33906240.9 |
| `rum_stdlib::lazy::realize` | 2330 | 89 | -2241 | 33925021.7 | 33950383.9 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 135 | 135 | 0 | 2063 |

`lazy-depth-select-restart-256` — full hardware panel +0.050842% / +0.312149%; Callgrind total change -21858.408 instructions/iteration.

| Function | Self base | Self incoming | Self delta | Inclusive base | Inclusive incoming |
| --- | ---: | ---: | ---: | ---: | ---: |
| `PROGRAM TOTALS` | 3679485.246 | 3657626.838 | -21858.408 | 3679485.246 | 3657626.838 |
| `rum_stdlib::lazy::PullStepper::next'2` | 59672 | 0 | -59672 | 455558930.186 | 0 |
| `rum_stdlib::lazy::machine::drive_pull` | 0 | 48870 | 48870 | 0 | 3531558.622 |
| `rum_stdlib::lazy::stepper_from_descriptor'2` | 30674 | 0 | -30674 | 12351676.847 | 0 |
| `rum_stdlib::lazy::descriptor::push_stage` | 0 | 18263 | 18263 | 0 | 34000.34 |
| `rum_stdlib::lazy::descriptor::machine_from_descriptor` | 0 | 17786 | 17786 | 0 | 104830.088 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::PullStepper>'2` | 8978 | 0 | -8978 | 2417792 | 0 |
| `rum_core::expr::Symbol::is_system_name` | 126193 | 134678 | 8485 | 234004 | 247590 |
| `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 214202.681 | 208431.52 | -5771.161 | 774007.556 | 762635.006 |
| `rum_runtime::runtime::model::request_cancellation::CancellationObserver::is_cancelled` | 12 | 2064 | 2052 | 12 | 2064 |
| `rum_stdlib::lazy::source::pull_source` | 0 | 429 | 429 | 0 | 7020.707 |
| `rum_stdlib::lazy::realization::drive_items` | 0 | 91 | 91 | 0 | 3531757.622 |
| `rum_stdlib::lazy::realize` | 134 | 89 | -45 | 3666932.618 | 3645078.71 |

### 3. Every measured candidate — full rows and both samples

The preserved pre-G2 base was measured once at the start, using all 54 labels. Incoming and both candidates also have all 54 labels, allocations and exactly two instruction samples; nothing is missing. Each build passed all 54 expected-value checks before measurement. Allocation processes ran before instruction samples for each phase, using only `RUM_PROFILE_ALLOC=1`; all other profiling flags were cleared. Instruction processes used `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, stdin `/dev/null`, and a 150-second bound. Raw output was saved before parsing. The result is the prescribed N/2N difference, with each label/count in its own process.

Candidate screens used the prescribed 38-label union plus four additional fresh incoming regressions (42 labels). A disjoint 12-label remainder then completed each candidate to 54; this was not a rerun or retry. Sample directories `1`/`2` and `remainder-1`/`remainder-2` preserve that provenance. There are 432 allocation processes and 864 full-panel instruction processes across four builds. Both samples of every row are shown, including unfavorable and newly regressing neighbors. No allocation count, total-byte or peak-live-byte regression occurs anywhere; raw peaks for N/2N are included below.

The copied comparison harness was adapted to this brief: **either** sample above +0.05% fails, and the largest remaining delta is the maximum of either sample, not the older harness’s both-sample/minimum rule. `instruction_regression=True` below is this exact gate. Tables are the saved TSV output verbatim, retaining the original precision.

**Incoming G2 baseline against pre-G2 — all 54 rows.** The retained final source and its measured binary are exactly this incoming build.

```tsv
label	N	base_count	new_count	delta_count	base_bytes_total	new_bytes_total	delta_bytes_total	base_peak	new_peak	base_instructions_1	new_instructions_1	instruction_delta_pct_1	base_instructions_2	new_instructions_2	instruction_delta_pct_2	allocation_regression	instruction_regression
lazy-range-take-small	20000	141.0	141.0	0.0	2714.0	2714.0	0.0	[11428045, 12068045]	[11428045, 12068045]	41866.46305	42759.0857	2.1320708389767073	41908.2281	42757.21765	2.0258302211541013	False	True
lazy-range-normal-small	20000	157.0	157.0	0.0	3444.0	3388.0	-56.0	[11428491, 12068491]	[11428491, 12068491]	53600.5615	54849.89205	2.3308161613194933	53633.99805	54824.7679	2.220177300394255	False	True
lazy-map-realize-small	20000	333.0	333.0	0.0	13780.0	13740.0	-40.0	[11429646, 12069646]	[11429646, 12069646]	178455.3983	179795.9007	0.7511694310006201	178117.98815	179651.9789	0.8612216912691428	False	True
lazy-select-realize-small	20000	236.0	236.0	0.0	6874.0	6834.0	-40.0	[9594202, 10234202]	[9594202, 10234202]	114571.3081	115443.6305	0.7613794539542251	114257.5359	115371.4587	0.9749228278254973	False	True
lazy-select-scan-small	20000	1471.0	1471.0	0.0	62849.0	62793.0	-56.0	[9595818, 10235818]	[9595818, 10235818]	798979.8237	800829.28165	0.23147742848317332	798976.24295	800745.198	0.22140270948090013	False	True
lazy-nestlist-realize-small	20000	197.0	197.0	0.0	11372.0	11316.0	-56.0	[11429099, 12069099]	[11429099, 12069099]	145308.41995	146429.32715	0.7713986569984677	145170.22345	146407.6849	0.8524209859236187	False	True
lazy-foldlist-realize-small	20000	216.0	216.0	0.0	7644.0	7604.0	-40.0	[11429094, 12069094]	[11429094, 12069094]	113064.68195	114666.7016	1.4169054583361929	113104.2033	114659.42605	1.375035325499696	False	True
lazy-drop-realize-small	20000	352.0	351.0	-1.0	14300.0	14204.0	-96.0	[9595465, 10235465]	[9595465, 10235465]	191379.34535	192508.83575	0.5901840650224699	191608.3584	192532.94605	0.4825403535214434	False	True
lazy-first-small	20000	64.0	64.0	0.0	2924.0	2868.0	-56.0	[9595677, 10235677]	[9595621, 10235621]	41346.6924	41682.91565	0.8131805242056034	41428.11455	41637.5151	0.5054551776602523	False	True
lazy-part-small	20000	152.0	152.0	0.0	6222.0	6182.0	-40.0	[9596551, 10236551]	[9596439, 10236439]	88374.442	88997.2267	0.7047113236652747	88477.99015	88947.086	0.53018366398776	False	True
lazy-transparent-adapters-small	20000	322.0	319.0	-3.0	13084.0	12876.0	-208.0	[949519, 1589519]	[949239, 1589239]	182146.00455	183050.3362	0.49648722860222705	182051.116	182990.0272	0.5157404253429476	False	True
lazy-table-construct-short	20000	63.0	63.0	0.0	1537.0	1537.0	0.0	[942784, 1582784]	[942784, 1582784]	23558.9366	23790.99555	0.9850145358428319	23557.7234	23775.6751	0.9251815054420742	False	True
lazy-table-construct-full	20000	64.0	64.0	0.0	1449.0	1449.0	0.0	[942664, 1582664]	[942664, 1582664]	24031.96275	24274.11255	1.0076155764680683	24067.89445	24216.01335	0.6154210967964424	False	True
lazy-table-realize-small	20000	251.0	251.0	0.0	9230.0	9190.0	-40.0	[10053760, 10693760]	[10053648, 10693648]	116575.23	117926.9434	1.1595202514290692	116586.67535	117839.02095	1.0741755833077793	False	True
lazy-permutations-take-small	20000	29.0	29.0	0.0	1729.0	1729.0	0.0	[9593774, 10233774]	[9593774, 10233774]	28232.62225	28620.7	1.3745721051469095	28259.2998	28616.12715	1.2626899906415856	False	True
lazy-subsets-take-small	20000	29.0	29.0	0.0	1417.0	1417.0	0.0	[8446869, 9086869]	[8446869, 9086869]	25496.62385	25861.6933	1.4318344740376299	25497.21845	25885.70015	1.5236238445452965	False	True
lazy-tuples-take-small	20000	29.0	29.0	0.0	1361.0	1361.0	0.0	[8905896, 9545896]	[8905896, 9545896]	26678.86535	27064.83675	1.446730942026364	26678.87655	27010.1642	1.2417601220168217	False	True
scan-no-input	20000	33.0	33.0	0.0	2536.0	2536.0	0.0	[942414, 1582414]	[942414, 1582414]	30010.35365	29999.4831	-0.03622266543999997	30010.95075	30000.1989	-0.03582642246013901	False	False
optionvalue-three-no-input	20000	9.0	9.0	0.0	1128.0	1128.0	0.0	[9134068, 9774068]	[9134068, 9774068]	9739.0381	9738.3558	-0.007005825349426509	9739.8172	9737.90935	-0.01958814997061742	False	False
replacerepeated-no-input	20000	61.0	61.0	0.0	7691.0	7691.0	0.0	[948049, 1588049]	[948049, 1588049]	40106.7715	40106.888	0.00029047463967479814	40106.7864	40117.82025	0.02751117950452464	False	False
dot-three-no-input	20000	596.0	596.0	0.0	23504.0	23504.0	0.0	[8678236, 9318236]	[8678236, 9318236]	256453.59025	256429.1003	-0.009549466621283642	256452.30475	256428.77655	-0.009174493488350866	False	False
differences-no-input	20000	13.0	13.0	0.0	704.0	704.0	0.0	[9591399, 10231399]	[9591399, 10231399]	9990.1536	9986.1365	-0.04021059295824525	9989.7249	9985.58525	-0.04143907906812849	False	False
do-count-null	20000	0.0	0.0	0.0	0.0	0.0	0.0	[299668, 299668]	[299668, 299668]	496.9623	497.1148	0.030686432351090254	497.0491	497.1471	0.0197163620254015	False	False
do-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	[940146, 1580146]	[940146, 1580146]	1137.1301	1137.10595	-0.002123767544293109	1136.9838	1137.0044	0.0018118112149023702	False	False
table-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	[940025, 1580025]	[940025, 1580025]	349.09155	349.0101	-0.02333198841392159	349.18965	349.1538	-0.010266627318411548	False	False
replaceall-simple	20000	5.0	5.0	0.0	1352.0	1352.0	0.0	[943388, 1583388]	[943388, 1583388]	11403.31145	11403.35405	0.00037357569497853405	11403.19295	11403.2911	0.0008607238378699478	False	False
table-small	20000	22.0	22.0	0.0	977.0	977.0	0.0	[9132887, 9772887]	[9132887, 9772887]	14756.54255	14757.30225	0.0051482249139755965	14756.32255	14757.5099	0.008046381447512907	False	False
table-iterator-plus	20000	2.0	2.0	0.0	72.0	72.0	0.0	[940209, 1580209]	[940209, 1580209]	565.4326	565.2056	-0.040146252621442446	565.1597	565.2473	0.015500043615990222	False	False
table-iterator-pair-list	20000	2.0	2.0	0.0	136.0	136.0	0.0	[8574075, 11294075]	[8574075, 11294075]	1726.86805	1726.8747	0.0003850902215773999	1726.84505	1726.74535	-0.0057735348055643065	False	False
take-multispec-small	20000	109.0	109.0	0.0	5856.0	5856.0	0.0	[9594023, 10234023]	[9594023, 10234023]	50794.89165	51091.70375	0.5843345469563666	50781.30325	51091.80955	0.6114579188158276	False	True
lazy-depth-map-build-8	1000	22.001	16.001	-6.0	2255.864	1879.864	-376.0	[762116, 1220100]	[762116, 1220100]	19443.04	19562.306	0.6134123058945562	19451.873	19558.291	0.5470835636239402	False	True
lazy-depth-map-first-8	1000	194.0	188.0	-6.0	11552.0	11176.0	-376.0	[339418, 371418]	[338970, 370970]	131660.098	131503.128	-0.11922366942185292	131489.295	132034.494	0.41463375402537306	False	True
lazy-depth-map-eight-8	1000	1470.001	1464.001	-6.0	83152.864	82776.864	-376.0	[990724, 1678084]	[990724, 1678084]	961516.907	959577.181	-0.2017360262600132	961546.743	959588.569	-0.2036483420338464	False	False
lazy-depth-select-restart-8	1000	333.0	327.0	-6.0	17356.0	16980.0	-376.0	[341305, 373305]	[340857, 372857]	179733.558	180142.554	0.2275568372156922	179773.534	179687.973	-0.04759376872460752	False	True
lazy-depth-drop-all-one-8	1000	176.0	170.0	-6.0	3504.0	3128.0	-376.0	[337909, 369909]	[337461, 369461]	48247.576	49462.61	2.5183316981561843	48248.029	49462.618	2.5173857361095564	False	True
lazy-depth-take-map-8	1000	234.0	220.0	-14.0	12968.0	12144.0	-824.0	[342603, 374603]	[341707, 373707]	141434.063	142264.666	0.587272247138948	141433.169	142266.043	0.5888816646680839	False	True
lazy-depth-fold-through-source-8	1000	749.001	743.001	-6.0	58816.864	58440.864	-376.0	[1219923, 2136659]	[1219923, 2136659]	601546.861	602191.255	0.10712282646254678	601497.956	602206.152	0.11773872096083604	False	True
lazy-depth-nest-map-8	1000	198.0	192.0	-6.0	12340.0	11964.0	-376.0	[339730, 371730]	[339282, 371282]	142362.834	142481.484	0.08334338160196353	142352.318	142467.884	0.08118308266675811	False	True
lazy-depth-map-build-64	1000	78.001	16.001	-62.0	9423.864	5911.864	-3512.0	[770631, 1228615]	[770631, 1228615]	41633.736	39803.867	-4.395159252583047	41635.901	39800.492	-4.40823653606055	False	False
lazy-depth-map-first-64	1000	1426.0	1364.0	-62.0	88832.0	85320.0	-3512.0	[355101, 387101]	[351517, 383517]	957598.715	954310.411	-0.34339060281634026	956255.544	952904.696	-0.3504134455507013	False	False
lazy-depth-map-eight-64	1000	10934.001	10872.001	-62.0	651216.864	647704.864	-3512.0	[999239, 1686599]	[999239, 1686599]	7417452.079	7417827.756	0.005064771514518007	7417829.989	7416842.865	-0.013307449772559377	False	False
lazy-depth-select-restart-64	1000	1733.0	1671.0	-62.0	102924.0	99412.0	-3512.0	[356988, 388988]	[353404, 385404]	978226.333	976081.571	-0.21925007819227904	978635.006	974086.489	-0.46478175950310785	False	False
lazy-depth-drop-all-one-64	1000	1240.0	1178.0	-62.0	21032.0	17520.0	-3512.0	[353592, 385592]	[350008, 382008]	345167.106	348580.424	0.9888885530129121	345168.131	348579.261	0.9882517224627518	False	True
lazy-depth-take-map-64	1000	1746.0	1620.0	-126.0	100160.0	93064.0	-7096.0	[373966, 405966]	[366798, 398798]	1031116.302	1032019.225	0.087567522523746	1031278.008	1031989.928	0.06903279178624988	False	True
lazy-depth-fold-through-source-64	1000	43396.001	43334.001	-62.0	3406161.864	3402649.864	-3512.0	[4771185, 9218609]	[4767601, 9215025]	34505198.91	34528102.899	0.06637837115426315	34506023.299	34527063.031	0.060974085068243866	False	True
lazy-depth-nest-map-64	1000	1430.0	1368.0	-62.0	89620.0	86108.0	-3512.0	[355413, 387413]	[351829, 383829]	968791.52	966866.978	-0.19865388582261945	968789.646	967100.016	-0.1744062817946368	False	False
lazy-depth-map-build-256	1000	270.001	16.001	-253.99999999999997	33999.864	19735.864	-14264.0	[799818, 1257802]	[799818, 1257802]	117652.304	108691.845	-7.6160505960002345	117656.486	108689.184	-7.621595973893025	False	False
lazy-depth-map-first-256	1000	5650.0	5396.0	-254.0	353792.0	339528.0	-14264.0	[408864, 440864]	[394528, 426528]	3758682.306	3754393.033	-0.11411640172815929	3758755.197	3754392.806	-0.11605946041609583	False	False
lazy-depth-map-eight-256	1000	43382.001	43128.001	-254.0	2598864.864	2584600.864	-14264.0	[1028426, 1715786]	[1028426, 1715786]	29548568.386	29530710.558	-0.060435509993983505	29548541.642	29530188.58	-0.06211156618950442	False	False
lazy-depth-select-restart-256	1000	6533.0	6279.0	-254.0	396300.0	382036.0	-14264.0	[410751, 442751]	[396415, 428415]	3666140.216	3668004.155	0.050841999764905665	3677705.209	3689185.122	0.312148808770929	False	True
lazy-depth-drop-all-one-256	1000	4888.0	4634.0	-254.0	81128.0	66864.0	-14264.0	[407355, 439355]	[393019, 425019]	2339175.932	2302001.788	-1.5891982937861315	2339170.184	2302000.937	-1.5889928511503282	False	False
lazy-depth-take-map-256	1000	6930.0	6420.0	-510.0	399104.0	370504.0	-28600.0	[481489, 513489]	[452817, 484817]	4087067.525	4082910.66	-0.10170776417499638	4087373.546	4082482.82	-0.11965449071290601	False	False
lazy-depth-fold-through-source-256	1000	689574.001	689320.001	-254.0	54023121.864	54008857.864	-14264.0	[17119095, 33854519]	[17104759, 33840183]	538291027.625	538549689.19	0.048052364190676045	538311717.17	538511933.209	0.03719332732576852	False	False
lazy-depth-nest-map-256	1000	5654.0	5400.0	-254.0	354580.0	340316.0	-14264.0	[409176, 441176]	[394840, 426840]	3775404.676	3765035.346	-0.2746547957075185	3775396.271	3765038.319	-0.2743540348217888	False	False
```

**Candidate 01 — cold out-of-line lazy `Take` branch.** One code-shape variable: the two attributes on `eval_take_lazy_list`, with no other source change. Attribution preceded the edit (§2). Ordinary `take-multispec-small` remained +0.5866283799820016% / +0.6137940148277066%. The full panel had 29 failing labels, worst +2.520369106211673%, and newly failed `lazy-depth-nest-map-64` at +0.08004002760055862% in sample 1. Rejected; incoming source restored before candidate 02. Both release builds and all 54 value checks passed; no candidate-specific Cargo correctness suite was run for this attribute-only probe.

```tsv
label	N	base_count	new_count	delta_count	base_bytes_total	new_bytes_total	delta_bytes_total	base_peak	new_peak	base_instructions_1	new_instructions_1	instruction_delta_pct_1	base_instructions_2	new_instructions_2	instruction_delta_pct_2	allocation_regression	instruction_regression
lazy-range-take-small	20000	141.0	141.0	0.0	2714.0	2714.0	0.0	[11428045, 12068045]	[11428045, 12068045]	41866.46305	42784.70325	2.193259552170357	41908.2281	42816.53175	2.1673635254457313	False	True
lazy-range-normal-small	20000	157.0	157.0	0.0	3444.0	3388.0	-56.0	[11428491, 12068491]	[11428491, 12068491]	53600.5615	54826.5977	2.2873570083775885	53633.99805	54848.85935	2.26509554418719	False	True
lazy-map-realize-small	20000	333.0	333.0	0.0	13780.0	13740.0	-40.0	[11429646, 12069646]	[11429646, 12069646]	178455.3983	179663.5962	0.6770307379376117	178117.98815	179647.9828	0.8589781783923778	False	True
lazy-select-realize-small	20000	236.0	236.0	0.0	6874.0	6834.0	-40.0	[9594202, 10234202]	[9594202, 10234202]	114571.3081	115372.17525	0.6990119631880232	114257.5359	115441.94815	1.0366163077738744	False	True
lazy-select-scan-small	20000	1471.0	1471.0	0.0	62849.0	62793.0	-56.0	[9595818, 10235818]	[9595818, 10235818]	798979.8237	800858.54095	0.23513951094533692	798976.24295	800825.84205	0.2314961322468001	False	True
lazy-nestlist-realize-small	20000	197.0	197.0	0.0	11372.0	11316.0	-56.0	[11429099, 12069099]	[11429099, 12069099]	145308.41995	146445.33415	0.7824145361921886	145170.22345	146401.10795	0.847890476950286	False	True
lazy-foldlist-realize-small	20000	216.0	216.0	0.0	7644.0	7604.0	-40.0	[11429094, 12069094]	[11429094, 12069094]	113064.68195	114679.9936	1.4286615609234499	113104.2033	114633.10965	1.3517679320411258	False	True
lazy-drop-realize-small	20000	352.0	351.0	-1.0	14300.0	14204.0	-96.0	[9595465, 10235465]	[9595465, 10235465]	191379.34535	192482.73695	0.5765468567060283	191608.3584	192612.3366	0.5239741148995725	False	True
lazy-first-small	20000	64.0	64.0	0.0	2924.0	2868.0	-56.0	[9595677, 10235677]	[9595621, 10235621]	41346.6924	41636.727	0.7014698955701615	41428.11455	41637.14205	0.5045547022124897	False	True
lazy-part-small	20000	152.0	152.0	0.0	6222.0	6182.0	-40.0	[9596551, 10236551]	[9596439, 10236439]	88374.442	88947.5528	0.6485028782416524	88477.99015	89001.7866	0.5920076271081598	False	True
lazy-transparent-adapters-small	20000	322.0	319.0	-3.0	13084.0	12876.0	-208.0	[949519, 1589519]	[949239, 1589239]	182146.00455	183086.43255	0.5163044900838409	182051.116	183010.5643	0.5270213778859612	False	True
lazy-table-construct-short	20000	63.0	63.0	0.0	1537.0	1537.0	0.0	[942784, 1582784]	[942784, 1582784]	23558.9366	23675.36305	0.4941922972873103	23557.7234	23791.5284	0.9924770574392561	False	True
lazy-table-construct-full	20000	64.0	64.0	0.0	1449.0	1449.0	0.0	[942664, 1582664]	[942664, 1582664]	24031.96275	24215.0049	0.7616612588166705	24067.89445	24241.5259	0.7214235144695103	False	True
lazy-table-realize-small	20000	251.0	251.0	0.0	9230.0	9190.0	-40.0	[10053760, 10693760]	[10053648, 10693648]	116575.23	117886.84195	1.125120619534714	116586.67535	117948.52425	1.1680999530277791	False	True
lazy-permutations-take-small	20000	29.0	29.0	0.0	1729.0	1729.0	0.0	[9593774, 10233774]	[9593774, 10233774]	28232.62225	28665.11575	1.5318927734387033	28259.2998	28592.1844	1.17796478453438	False	True
lazy-subsets-take-small	20000	29.0	29.0	0.0	1417.0	1417.0	0.0	[8446869, 9086869]	[8446869, 9086869]	25496.62385	25894.4416	1.5602761853507063	25497.21845	25910.8132	1.6221171372518883	False	True
lazy-tuples-take-small	20000	29.0	29.0	0.0	1361.0	1361.0	0.0	[8905896, 9545896]	[8905896, 9545896]	26678.86535	27065.95895	1.4509372678400023	26678.87655	27138.0634	1.7211626176965034	False	True
scan-no-input	20000	33.0	33.0	0.0	2536.0	2536.0	0.0	[942414, 1582414]	[942414, 1582414]	30010.35365	30003.98695	-0.021215011573183506	30010.95075	30000.7934	-0.03384547888740652	False	False
optionvalue-three-no-input	20000	9.0	9.0	0.0	1128.0	1128.0	0.0	[9134068, 9774068]	[9134068, 9774068]	9739.0381	9738.34865	-0.0070792412240394675	9739.8172	9738.6526	-0.011957103260629154	False	False
replacerepeated-no-input	20000	61.0	61.0	0.0	7691.0	7691.0	0.0	[948049, 1588049]	[948049, 1588049]	40106.7715	40117.943	0.02785439860197858	40106.7864	40106.80485	4.600218981298809e-05	False	False
dot-three-no-input	20000	596.0	596.0	0.0	23504.0	23504.0	0.0	[8678236, 9318236]	[8678236, 9318236]	256453.59025	256429.1491	-0.009530437837179662	256452.30475	256429.1062	-0.009045951067832014	False	False
differences-no-input	20000	13.0	13.0	0.0	704.0	704.0	0.0	[9591399, 10231399]	[9591399, 10231399]	9990.1536	9986.0851	-0.040725099562022304	9989.7249	9987.59415	-0.021329416188409223	False	False
do-count-null	20000	0.0	0.0	0.0	0.0	0.0	0.0	[299668, 299668]	[299668, 299668]	496.9623	497.0444	0.01652036784278721	497.0491	496.85165	-0.03972444573383527	False	False
do-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	[940146, 1580146]	[940146, 1580146]	1137.1301	1136.9752	-0.013622012116287774	1136.9838	1137.06835	0.007436341661160029	False	False
table-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	[940025, 1580025]	[940025, 1580025]	349.09155	349.0673	-0.006946601829804777	349.18965	349.11555	-0.02122056023138308	False	False
replaceall-simple	20000	5.0	5.0	0.0	1352.0	1352.0	0.0	[943388, 1583388]	[943388, 1583388]	11403.31145	11403.33785	0.00023151169830093465	11403.19295	11381.3146	-0.19186161363691223	False	False
table-small	20000	22.0	22.0	0.0	977.0	977.0	0.0	[9132887, 9772887]	[9132887, 9772887]	14756.54255	14733.3269	-0.15732445402666384	14756.32255	14757.39415	0.0072619719199584765	False	False
table-iterator-plus	20000	2.0	2.0	0.0	72.0	72.0	0.0	[940209, 1580209]	[940209, 1580209]	565.4326	565.3321	-0.017774001711257092	565.1597	565.3288	0.029920746295242218	False	False
table-iterator-pair-list	20000	2.0	2.0	0.0	136.0	136.0	0.0	[8574075, 11294075]	[8574075, 11294075]	1726.86805	1726.5085	-0.020820930701692664	1726.84505	1726.70375	-0.008182552337276938	False	False
take-multispec-small	20000	109.0	109.0	0.0	5856.0	5856.0	0.0	[9594023, 10234023]	[9594023, 10234023]	50794.89165	51092.8689	0.5866283799820016	50781.30325	51092.99585	0.6137940148277066	False	True
lazy-depth-map-build-8	1000	22.001	16.001	-6.0	2255.864	1879.864	-376.0	[762116, 1220100]	[762116, 1220100]	19443.04	19591.745	0.7648238135600005	19451.873	19570.635	0.6105427482484549	False	True
lazy-depth-map-first-8	1000	194.0	188.0	-6.0	11552.0	11176.0	-376.0	[339418, 371418]	[338970, 370970]	131660.098	131504.084	-0.11849755724775424	131489.295	131858.679	0.2809232493032887	False	True
lazy-depth-map-eight-8	1000	1470.001	1464.001	-6.0	83152.864	82776.864	-376.0	[990724, 1678084]	[990724, 1678084]	961516.907	959628.363	-0.19641297893475684	961546.743	959609.655	-0.20145541692090374	False	False
lazy-depth-select-restart-8	1000	333.0	327.0	-6.0	17356.0	16980.0	-376.0	[341305, 373305]	[340857, 372857]	179733.558	179644.77	-0.04939978988230953	179773.534	179612.12	-0.08978740997549473	False	False
lazy-depth-drop-all-one-8	1000	176.0	170.0	-6.0	3504.0	3128.0	-376.0	[337909, 369909]	[337461, 369461]	48247.576	49463.593	2.520369106211673	48248.029	49461.975	2.5160530391821645	False	True
lazy-depth-take-map-8	1000	234.0	220.0	-14.0	12968.0	12144.0	-824.0	[342603, 374603]	[341707, 373707]	141434.063	142246.321	0.5743015386611594	141433.169	142264.007	0.5874421155054543	False	True
lazy-depth-fold-through-source-8	1000	749.001	743.001	-6.0	58816.864	58440.864	-376.0	[1219923, 2136659]	[1219923, 2136659]	601546.861	602206.612	0.10967574477958042	601497.956	602232.121	0.12205610886564244	False	True
lazy-depth-nest-map-8	1000	198.0	192.0	-6.0	12340.0	11964.0	-376.0	[339730, 371730]	[339282, 371282]	142362.834	142461.656	0.06941558918389656	142352.318	142306.327	-0.0323078687064382	False	True
lazy-depth-map-build-64	1000	78.001	16.001	-62.0	9423.864	5911.864	-3512.0	[770631, 1228615]	[770631, 1228615]	41633.736	39830.463	-4.331278365218038	41635.901	39813.813	-4.376242512441353	False	False
lazy-depth-map-first-64	1000	1426.0	1364.0	-62.0	88832.0	85320.0	-3512.0	[355101, 387101]	[351517, 383517]	957598.715	954246.909	-0.3500219818068584	956255.544	954118.539	-0.2234763514218141	False	False
lazy-depth-map-eight-64	1000	10934.001	10872.001	-62.0	651216.864	647704.864	-3512.0	[999239, 1686599]	[999239, 1686599]	7417452.079	7417837.013	0.005189571781527569	7417829.989	7417490.499	-0.004576675395684404	False	False
lazy-depth-select-restart-64	1000	1733.0	1671.0	-62.0	102924.0	99412.0	-3512.0	[356988, 388988]	[353404, 385404]	978226.333	976724.117	-0.1535652792532205	978635.006	976728.629	-0.19479959211679	False	False
lazy-depth-drop-all-one-64	1000	1240.0	1178.0	-62.0	21032.0	17520.0	-3512.0	[353592, 385592]	[350008, 382008]	345167.106	348584.466	0.9900595800110823	345168.131	348583.611	0.9895119778598582	False	True
lazy-depth-take-map-64	1000	1746.0	1620.0	-126.0	100160.0	93064.0	-7096.0	[373966, 405966]	[366798, 398798]	1031116.302	1031961.895	0.08200752896252439	1031278.008	1031911.343	0.06141263510779371	False	True
lazy-depth-fold-through-source-64	1000	43396.001	43334.001	-62.0	3406161.864	3402649.864	-3512.0	[4771185, 9218609]	[4767601, 9215025]	34505198.91	34527010.017	0.0632110745308001	34506023.299	34526472.674	0.059263204058046526	False	True
lazy-depth-nest-map-64	1000	1430.0	1368.0	-62.0	89620.0	86108.0	-3512.0	[355413, 387413]	[351829, 383829]	968791.52	969566.941	0.08004002760055862	968789.646	966866.004	-0.19856137066931456	False	True
lazy-depth-map-build-256	1000	270.001	16.001	-253.99999999999997	33999.864	19735.864	-14264.0	[799818, 1257802]	[799818, 1257802]	117652.304	108700.902	-7.608352489212622	117656.486	108685.085	-7.625079844726956	False	False
lazy-depth-map-first-256	1000	5650.0	5396.0	-254.0	353792.0	339528.0	-14264.0	[408864, 440864]	[394528, 426528]	3758682.306	3754388.173	-0.11424570236077125	3758755.197	3754321.817	-0.11794809099402448	False	False
lazy-depth-map-eight-256	1000	43382.001	43128.001	-254.0	2598864.864	2584600.864	-14264.0	[1028426, 1715786]	[1028426, 1715786]	29548568.386	29530016.529	-0.06278428368391431	29548541.642	29530157.425	-0.0622170028651059	False	False
lazy-depth-select-restart-256	1000	6533.0	6279.0	-254.0	396300.0	382036.0	-14264.0	[410751, 442751]	[396415, 428415]	3666140.216	3666388.206	0.006764334842346287	3677705.209	3689030.773	0.30795192535508775	False	True
lazy-depth-drop-all-one-256	1000	4888.0	4634.0	-254.0	81128.0	66864.0	-14264.0	[407355, 439355]	[393019, 425019]	2339175.932	2302008.735	-1.5889013088563275	2339170.184	2302011.137	-1.5885567990806648	False	False
lazy-depth-take-map-256	1000	6930.0	6420.0	-510.0	399104.0	370504.0	-28600.0	[481489, 513489]	[452817, 484817]	4087067.525	4082815.863	-0.10402720224202966	4087373.546	4082366.91	-0.12249029709798798	False	False
lazy-depth-fold-through-source-256	1000	689574.001	689320.001	-254.0	54023121.864	54008857.864	-14264.0	[17119095, 33854519]	[17104759, 33840183]	538291027.625	538541872.83	0.04660029465970439	538311717.17	538558497.726	0.0458434301406907	False	False
lazy-depth-nest-map-256	1000	5654.0	5400.0	-254.0	354580.0	340316.0	-14264.0	[409176, 441176]	[394840, 426840]	3775404.676	3765038.459	-0.2745723409704248	3775396.271	3759399.476	-0.42371168088702893	False	False
```

**Candidate 02 — move LazyTable bound recovery to the suspension edge.** This experiment started from restored incoming source, without candidate 01. It addressed the measured clone/drop work (§2); both release builds, formatting and all 69 lazy continuation tests passed before measurement. The full constructor reached +0.034550028586410875% / -0.21818090530972833%, but the short constructor was +1.0669503223672772% / -0.06012465533915545%. Newly failing neighbors were `table-iterator-plus` (+0.07082953720867202% in sample 2), `lazy-depth-select-restart-64` (+0.3110174867380522% in sample 2), and `lazy-depth-map-first-256` (+0.17656504220657787% in sample 1). It had 29 failing labels and worst +2.5201535513410978%. Rejected and fully restored; no further optimization attempted.

```tsv
label	N	base_count	new_count	delta_count	base_bytes_total	new_bytes_total	delta_bytes_total	base_peak	new_peak	base_instructions_1	new_instructions_1	instruction_delta_pct_1	base_instructions_2	new_instructions_2	instruction_delta_pct_2	allocation_regression	instruction_regression
lazy-range-take-small	20000	141.0	141.0	0.0	2714.0	2714.0	0.0	[11428045, 12068045]	[11428045, 12068045]	41866.46305	42724.36215	2.049132020002342	41908.2281	42744.0306	1.9943637273464265	False	True
lazy-range-normal-small	20000	157.0	157.0	0.0	3444.0	3388.0	-56.0	[11428491, 12068491]	[11428491, 12068491]	53600.5615	54850.87755	2.332654761461761	53633.99805	54829.4425	2.228893040726798	False	True
lazy-map-realize-small	20000	333.0	333.0	0.0	13780.0	13740.0	-40.0	[11429646, 12069646]	[11429646, 12069646]	178455.3983	179731.46775	0.7150635184791865	178117.98815	179655.5916	0.8632499535673688	False	True
lazy-select-realize-small	20000	236.0	236.0	0.0	6874.0	6834.0	-40.0	[9594202, 10234202]	[9594202, 10234202]	114571.3081	115378.8923	0.7048747311980907	114257.5359	115376.75345	0.9795568766506113	False	True
lazy-select-scan-small	20000	1471.0	1471.0	0.0	62849.0	62793.0	-56.0	[9595818, 10235818]	[9595818, 10235818]	798979.8237	800787.69	0.22627183395294015	798976.24295	800723.49025	0.21868576386561145	False	True
lazy-nestlist-realize-small	20000	197.0	197.0	0.0	11372.0	11316.0	-56.0	[11429099, 12069099]	[11429099, 12069099]	145308.41995	146409.66065	0.7578643415012909	145170.22345	146405.7025	0.8510554166265116	False	True
lazy-foldlist-realize-small	20000	216.0	216.0	0.0	7644.0	7604.0	-40.0	[11429094, 12069094]	[11429094, 12069094]	113064.68195	114769.44695	1.5077785304821179	113104.2033	114630.6375	1.3495822042539407	False	True
lazy-drop-realize-small	20000	352.0	351.0	-1.0	14300.0	14204.0	-96.0	[9595465, 10235465]	[9595465, 10235465]	191379.34535	192514.1097	0.5929398221760751	191608.3584	192587.52815	0.51102663692566	False	True
lazy-first-small	20000	64.0	64.0	0.0	2924.0	2868.0	-56.0	[9595677, 10235677]	[9595621, 10235621]	41346.6924	41630.39285	0.686150290464349	41428.11455	41631.716	0.4914571957028713	False	True
lazy-part-small	20000	152.0	152.0	0.0	6222.0	6182.0	-40.0	[9596551, 10236551]	[9596439, 10236439]	88374.442	88997.89225	0.7054644260158449	88477.99015	88929.51305	0.5103222838069854	False	True
lazy-transparent-adapters-small	20000	322.0	319.0	-3.0	13084.0	12876.0	-208.0	[949519, 1589519]	[949239, 1589239]	182146.00455	183028.45075	0.4844718950492988	182051.116	183349.83035	0.7133789556115655	False	True
lazy-table-construct-short	20000	63.0	63.0	0.0	1537.0	1537.0	0.0	[942784, 1582784]	[942784, 1582784]	23558.9366	23810.29875	1.0669503223672772	23557.7234	23543.5594	-0.06012465533915545	False	True
lazy-table-construct-full	20000	64.0	64.0	0.0	1449.0	1449.0	0.0	[942664, 1582664]	[942664, 1582664]	24031.96275	24040.2658	0.034550028586410875	24067.89445	24015.3829	-0.21818090530972833	False	False
lazy-table-realize-small	20000	251.0	251.0	0.0	9230.0	9190.0	-40.0	[10053760, 10693760]	[10053648, 10693648]	116575.23	117687.2445	0.9539029002987975	116586.67535	117708.93885	0.9626001398795436	False	True
lazy-permutations-take-small	20000	29.0	29.0	0.0	1729.0	1729.0	0.0	[9593774, 10233774]	[9593774, 10233774]	28232.62225	28637.09475	1.4326423398379085	28259.2998	28779.17115	1.8396469611041022	False	True
lazy-subsets-take-small	20000	29.0	29.0	0.0	1417.0	1417.0	0.0	[8446869, 9086869]	[8446869, 9086869]	25496.62385	25877.01965	1.4919457659881452	25497.21845	25925.55415	1.6799310906794318	False	True
lazy-tuples-take-small	20000	29.0	29.0	0.0	1361.0	1361.0	0.0	[8905896, 9545896]	[8905896, 9545896]	26678.86535	27058.9595	1.4247013319852453	26678.87655	27066.05265	1.4512458921363436	False	True
scan-no-input	20000	33.0	33.0	0.0	2536.0	2536.0	0.0	[942414, 1582414]	[942414, 1582414]	30010.35365	29993.3389	-0.056696266223443814	30010.95075	29996.9255	-0.0467337743373486	False	False
optionvalue-three-no-input	20000	9.0	9.0	0.0	1128.0	1128.0	0.0	[9134068, 9774068]	[9134068, 9774068]	9739.0381	9739.2382	0.002054617693714178	9739.8172	9738.7716	-0.010735314416365771	False	False
replacerepeated-no-input	20000	61.0	61.0	0.0	7691.0	7691.0	0.0	[948049, 1588049]	[948049, 1588049]	40106.7715	40117.75545	0.027386771832271783	40106.7864	40106.75455	-7.941299430269666e-05	False	False
dot-three-no-input	20000	596.0	596.0	0.0	23504.0	23504.0	0.0	[8678236, 9318236]	[8678236, 9318236]	256453.59025	256429.40945	-0.00942891849414762	256452.30475	256429.39405	-0.008933707974412286	False	False
differences-no-input	20000	13.0	13.0	0.0	704.0	704.0	0.0	[9591399, 10231399]	[9591399, 10231399]	9990.1536	9987.13995	-0.030166202849968293	9989.7249	9987.2006	-0.02526896411331192	False	False
do-count-null	20000	0.0	0.0	0.0	0.0	0.0	0.0	[299668, 299668]	[299668, 299668]	496.9623	497.07315	0.022305514925369252	497.0491	496.8174	-0.04661511307434418	False	False
do-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	[940146, 1580146]	[940146, 1580146]	1137.1301	1137.10875	-0.0018775336260934417	1136.9838	1137.1656	0.015989673731509058	False	False
table-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	[940025, 1580025]	[940025, 1580025]	349.09155	348.90265	-0.0541118798206397	349.18965	349.0824	-0.03071396875593546	False	False
replaceall-simple	20000	5.0	5.0	0.0	1352.0	1352.0	0.0	[943388, 1583388]	[943388, 1583388]	11403.31145	11403.19145	-0.0010523259013406339	11403.19295	11403.2198	0.00023546036727051245	False	False
table-small	20000	22.0	22.0	0.0	977.0	977.0	0.0	[9132887, 9772887]	[9132887, 9772887]	14756.54255	14741.54805	-0.10161255557793458	14756.32255	14741.3588	-0.10140568525320859	False	False
table-iterator-plus	20000	2.0	2.0	0.0	72.0	72.0	0.0	[940209, 1580209]	[940209, 1580209]	565.4326	565.1473	-0.050456942171361874	565.1597	565.56	0.07082953720867202	False	True
table-iterator-pair-list	20000	2.0	2.0	0.0	136.0	136.0	0.0	[8574075, 11294075]	[8574075, 11294075]	1726.86805	1726.7373	-0.007571510747450549	1726.84505	1726.77675	-0.003955189841720053	False	False
take-multispec-small	20000	109.0	109.0	0.0	5856.0	5856.0	0.0	[9594023, 10234023]	[9594023, 10234023]	50794.89165	51083.05795	0.567313544018555	50781.30325	51083.66885	0.5954270187030097	False	True
lazy-depth-map-build-8	1000	22.001	16.001	-6.0	2255.864	1879.864	-376.0	[762116, 1220100]	[762116, 1220100]	19443.04	19553.954	0.5704560603691622	19451.873	19555.16	0.5309874272775739	False	True
lazy-depth-map-first-8	1000	194.0	188.0	-6.0	11552.0	11176.0	-376.0	[339418, 371418]	[338970, 370970]	131660.098	131502.085	-0.12001586084191551	131489.295	132034.094	0.41432954675131306	False	True
lazy-depth-map-eight-8	1000	1470.001	1464.001	-6.0	83152.864	82776.864	-376.0	[990724, 1678084]	[990724, 1678084]	961516.907	959615.611	-0.19773921666464966	961546.743	959606.214	-0.20181327783874714	False	False
lazy-depth-select-restart-8	1000	333.0	327.0	-6.0	17356.0	16980.0	-376.0	[341305, 373305]	[340857, 372857]	179733.558	179385.422	-0.1936956035778259	179773.534	179647.709	-0.06999083635971237	False	False
lazy-depth-drop-all-one-8	1000	176.0	170.0	-6.0	3504.0	3128.0	-376.0	[337909, 369909]	[337461, 369461]	48247.576	49463.489	2.5201535513410978	48248.029	49460.391	2.512770003516618	False	True
lazy-depth-take-map-8	1000	234.0	220.0	-14.0	12968.0	12144.0	-824.0	[342603, 374603]	[341707, 373707]	141434.063	142263.747	0.5866224743893644	141433.169	142253.625	0.5801015460524805	False	True
lazy-depth-fold-through-source-8	1000	749.001	743.001	-6.0	58816.864	58440.864	-376.0	[1219923, 2136659]	[1219923, 2136659]	601546.861	602208.29	0.10995469229120314	601497.956	602184.167	0.11408367944645281	False	True
lazy-depth-nest-map-8	1000	198.0	192.0	-6.0	12340.0	11964.0	-376.0	[339730, 371730]	[339282, 371282]	142362.834	142795.014	0.30357642360505643	142352.318	142805.41	0.3182891619650352	False	True
lazy-depth-map-build-64	1000	78.001	16.001	-62.0	9423.864	5911.864	-3512.0	[770631, 1228615]	[770631, 1228615]	41633.736	39801.618	-4.4005611218748015	41635.901	39781.756	-4.453236162704865	False	False
lazy-depth-map-first-64	1000	1426.0	1364.0	-62.0	88832.0	85320.0	-3512.0	[355101, 387101]	[351517, 383517]	957598.715	956929.873	-0.06984574953193823	956255.544	954254.63	-0.2092446953698368	False	False
lazy-depth-map-eight-64	1000	10934.001	10872.001	-62.0	651216.864	647704.864	-3512.0	[999239, 1686599]	[999239, 1686599]	7417452.079	7417147.986	-0.004099696186254942	7417829.989	7417021.652	-0.010897216587590908	False	False
lazy-depth-select-restart-64	1000	1733.0	1671.0	-62.0	102924.0	99412.0	-3512.0	[356988, 388988]	[353404, 385404]	978226.333	973905.528	-0.44169788261055887	978635.006	981678.732	0.3110174867380522	False	True
lazy-depth-drop-all-one-64	1000	1240.0	1178.0	-62.0	21032.0	17520.0	-3512.0	[353592, 385592]	[350008, 382008]	345167.106	348576.905	0.9878690468262619	345168.131	348579.129	0.9882134802300158	False	True
lazy-depth-take-map-64	1000	1746.0	1620.0	-126.0	100160.0	93064.0	-7096.0	[373966, 405966]	[366798, 398798]	1031116.302	1031829.981	0.06921420974683734	1031278.008	1032016.65	0.07162394565480845	False	True
lazy-depth-fold-through-source-64	1000	43396.001	43334.001	-62.0	3406161.864	3402649.864	-3512.0	[4771185, 9218609]	[4767601, 9215025]	34505198.91	34528148.24	0.06650977454112361	34506023.299	34527918.963	0.0634546143154946	False	True
lazy-depth-nest-map-64	1000	1430.0	1368.0	-62.0	89620.0	86108.0	-3512.0	[355413, 387413]	[351829, 383829]	968791.52	967101.52	-0.1744441363400906	968789.646	968274.579	-0.05316603063694192	False	False
lazy-depth-map-build-256	1000	270.001	16.001	-253.99999999999997	33999.864	19735.864	-14264.0	[799818, 1257802]	[799818, 1257802]	117652.304	108661.79	-7.641596207074707	117656.486	108666.998	-7.640452562895684	False	False
lazy-depth-map-first-256	1000	5650.0	5396.0	-254.0	353792.0	339528.0	-14264.0	[408864, 440864]	[394528, 426528]	3758682.306	3765318.825	0.17656504220657787	3758755.197	3754014.715	-0.1261184022780748	False	True
lazy-depth-map-eight-256	1000	43382.001	43128.001	-254.0	2598864.864	2584600.864	-14264.0	[1028426, 1715786]	[1028426, 1715786]	29548568.386	29527954.805	-0.06976169109352837	29548541.642	29532169.823	-0.05540652123667833	False	False
lazy-depth-select-restart-256	1000	6533.0	6279.0	-254.0	396300.0	382036.0	-14264.0	[410751, 442751]	[396415, 428415]	3666140.216	3667701.918	0.04259798883807786	3677705.209	3667448.794	-0.27888083511696804	False	False
lazy-depth-drop-all-one-256	1000	4888.0	4634.0	-254.0	81128.0	66864.0	-14264.0	[407355, 439355]	[393019, 425019]	2339175.932	2302007.308	-1.5889623132459563	2339170.184	2302014.063	-1.5884317119869595	False	False
lazy-depth-take-map-256	1000	6930.0	6420.0	-510.0	399104.0	370504.0	-28600.0	[481489, 513489]	[452817, 484817]	4087067.525	4082597.153	-0.10937847179317561	4087373.546	4082587.106	-0.11710307233073758	False	False
lazy-depth-fold-through-source-256	1000	689574.001	689320.001	-254.0	54023121.864	54008857.864	-14264.0	[17119095, 33854519]	[17104759, 33840183]	538291027.625	538554661.084	0.048976008417445094	538311717.17	538539293.598	0.042275956614235	False	False
lazy-depth-nest-map-256	1000	5654.0	5400.0	-254.0	354580.0	340316.0	-14264.0	[409176, 441176]	[394840, 426840]	3775404.676	3776297.583	0.023650630240412696	3775396.271	3765040.495	-0.27429639848791654	False	False
```

### 4. Verification — commands, bounds, exits, and verbatim results

All Cargo commands were serial, offline, one build job and one test thread, through `scripts/run-bounded.py`, with TMPDIR inside `.temp/brief-565/tmp`. The command blocks below reproduce the executed bounded command; the following result blocks copy actual output lines verbatim. Compiler progress, warnings, expected compile-fail diagnostics, and individual successful test names are omitted from these excerpts, not from the preserved raw logs. [Complete bounded-command transcript](../../../.temp/brief-565/final/command-output-transcript.txt) includes the full unedited log for every logged driver/build/test command. [Individual process command manifest](../../../.temp/brief-565/final/process-command-manifest.json) and [Individual process raw outputs](../../../.temp/brief-565/final/process-output-transcript.txt) preserve every allocation, instruction, value-check and attribution execution; full Callgrind profiles and annotation text remain under `attribution/`.

**`candidate-01/build-default` — timeout 1200 s; exit 0; elapsed 128.1105686679948 s.** Raw log: [candidate-01/build-default.log](../../../.temp/brief-565/candidate-01/build-default.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 1200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo build --release -p rum-cli --target-dir .temp/brief-565/candidate-01/target-default < /dev/null
```

```text
    Finished `release` profile [optimized] target(s) in 2m 07s
```

**`candidate-01/build-alloc` — timeout 1200 s; exit 0; elapsed 124.76322845599498 s.** Raw log: [candidate-01/build-alloc.log](../../../.temp/brief-565/candidate-01/build-alloc.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 1200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo build --release -p rum-cli --target-dir .temp/brief-565/candidate-01/target-alloc --features rum-cli/alloc-count < /dev/null
```

```text
    Finished `release` profile [optimized] target(s) in 2m 04s
```

**`candidate-02-fmt` — timeout 150 s; exit 0; elapsed 1.7416573730006348 s.** Raw log: [candidate-02-fmt.log](../../../.temp/brief-565/candidate-02-fmt.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo fmt --check < /dev/null
```

Output was empty (successful formatting check).

**`candidate-02-lazy-continuations` — timeout 150 s; exit 0; elapsed 40.61577512099757 s.** Raw log: [candidate-02-lazy-continuations.log](../../../.temp/brief-565/candidate-02-lazy-continuations.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-stdlib --test lazy_pull_continuations < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.05s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.43s
```

**`candidate-02/build-default` — timeout 1200 s; exit 0; elapsed 122.80277425399981 s.** Raw log: [candidate-02/build-default.log](../../../.temp/brief-565/candidate-02/build-default.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 1200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo build --release -p rum-cli --target-dir .temp/brief-565/candidate-02/target < /dev/null
```

```text
    Finished `release` profile [optimized] target(s) in 2m 02s
```

**`candidate-02/build-alloc` — timeout 300 s; exit 0; elapsed 3.4393607520032674 s.** Raw log: [candidate-02/build-alloc.log](../../../.temp/brief-565/candidate-02/build-alloc.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo build --release -p rum-cli --target-dir .temp/brief-565/candidate-02/target --features rum-cli/alloc-count < /dev/null
```

```text
    Finished `release` profile [optimized] target(s) in 3.33s
```

**`inventory/discovery` — timeout 300 s; exit 0; elapsed 23.12296446900291 s.** Raw log: [inventory/discovery.log](../../../.temp/brief-565/inventory/discovery.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.51s
Production files: 640; test-only files: 408; nominal definitions: 1612; aliases: 40; owning edges: 3660; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 5.48s
```

**`inventory/reconciliation` — timeout 150 s; exit 0; elapsed 0.23586912699101958 s.** Raw log: [inventory/reconciliation.log](../../../.temp/brief-565/inventory/reconciliation.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp python3 .temp/brief-565/reconcile-ownership.py .temp/brief-565/inventory/discovery.log < /dev/null
```

```text
host_payloads.tsv 63 -> 63 changed 0 added 0 removed 0
unknown_attributes.tsv 1445 -> 1445 changed 0 added 0 removed 0
unknown_captures.tsv 2938 -> 2938 changed 0 added 0 removed 0
unknown_macros.tsv 3114 -> 3114 changed 0 added 0 removed 0
unknown_storage.tsv 88 -> 88 changed 0 added 0 removed 0
self_reaching.tsv 59 -> 59 changed 0 added 0 removed 0
```

**`inventory/reconciliation-details` — timeout 150 s; exit 0; elapsed 0.14389678499719594 s.** Raw log: [inventory/reconciliation-details.log](../../../.temp/brief-565/inventory/reconciliation-details.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp python3 .temp/brief-565/reconcile-ownership.py .temp/brief-565/inventory/discovery.log < /dev/null
```

```text
host_payloads.tsv 63 -> 63 changed 0 added 0 removed 0
unknown_attributes.tsv 1445 -> 1445 changed 0 added 0 removed 0
unknown_captures.tsv 2938 -> 2938 changed 0 added 0 removed 0
unknown_macros.tsv 3114 -> 3114 changed 0 added 0 removed 0
unknown_storage.tsv 88 -> 88 changed 0 added 0 removed 0
self_reaching.tsv 59 -> 59 changed 0 added 0 removed 0
```

**`inventory/fingerprints` — timeout 300 s; exit 0; elapsed 0.3234437130013248 s.** Raw log: [inventory/fingerprints.log](../../../.temp/brief-565/inventory/fingerprints.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 0.18s
```

**`inventory/certificates-default-replay` — timeout 300 s; exit 0; elapsed 45.689749423996545 s.** Raw log: [inventory/certificates-default-replay.log](../../../.temp/brief-565/inventory/certificates-default-replay.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 27.97s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 17.58s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

**`inventory/certificates-no-default-replay` — timeout 300 s; exit 0; elapsed 60.62554282099882 s.** Raw log: [inventory/certificates-no-default-replay.log](../../../.temp/brief-565/inventory/certificates-no-default-replay.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 42.98s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 17.47s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

**`final/fmt` — timeout 150 s; exit 0; elapsed 1.7215826100000413 s.** Raw log: [final/fmt.log](../../../.temp/brief-565/final/fmt.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo fmt --check < /dev/null
```

Output was empty (successful formatting check).

**`final/lazy-pull-continuations` — timeout 150 s; exit 0; elapsed 23.531895988999167 s.** Raw log: [final/lazy-pull-continuations.log](../../../.temp/brief-565/final/lazy-pull-continuations.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-stdlib --test lazy_pull_continuations < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.36s
```

**`final/lazy-lib` — timeout 150 s; exit 0; elapsed 0.42559376200370025 s.** Raw log: [final/lazy-lib.log](../../../.temp/brief-565/final/lazy-lib.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-stdlib --lib lazy < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.30s
```

**`final/loop-continuations` — timeout 150 s; exit 0; elapsed 0.378922228992451 s.** Raw log: [final/loop-continuations.log](../../../.temp/brief-565/final/loop-continuations.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-stdlib --test loop_continuations < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
```

**`final/runtime` — timeout 300 s; exit 0; elapsed 27.627821214002324 s.** Raw log: [final/runtime.log](../../../.temp/brief-565/final/runtime.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp cargo test -p rum-runtime < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 22.52s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.86s
```

**`final/workspace` — timeout 2400 s; exit 0; elapsed 669.5611418379995 s.** Raw log: [final/workspace.log](../../../.temp/brief-565/final/workspace.log).

```bash
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-565/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture < /dev/null
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 14s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.88s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.13s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.06s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.99s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 22.69s
Production files: 640; test-only files: 408; nominal definitions: 1612; aliases: 40; owning edges: 3660; resolved Drop owners: 18
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1034 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 458.24s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.83s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.54s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.18s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.44s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The workspace command exited 0 in 669.5611418379995 seconds. Its 42 `test result:` lines sum to **2,955 harness-reported passed, 0 failed, 58 ignored**. Nine native tests returned early after printing an explicit skip; they are included in the harness pass count, so they are **skips, not nine executed native successes**. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` allowed the documented unavailable-loopback skip and did not force it. All nine raw lines:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

The unchanged structural/depth and frame-list checks ran inside the workspace suite. Selected raw successful lines:

```text
test runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph ... ok
test lazy::tests::lazy_flat_abandonment_depth ... flat abandonment depth=2000 complete
test lazy::tests::lazy_flat_all_variant_roots_depth ... all source/stage/cursor roots depth=2000 complete
test lazy::tests::lazy_flat_clone_and_working_copy_depth ... flat clone and working copy depth=2000 complete
test lazy::tests::lazy_flat_last_owner_drop_depth ... last-owner flat drop depth=2000 complete
test tests::association_depth_contract::lazy_depth_contract::lazy_ownership_is_not_self_reaching ... ok
test tests::association_depth_contract::lazy_depth_contract::lazy_source_has_no_recursive_edges ... ok
```

Ownership reconciliation after discovery had no new, vanished or changed identity in any partition. The raw reconciliation result (also quoted with its command above) is:

```text
host_payloads.tsv 63 -> 63 changed 0 added 0 removed 0
unknown_attributes.tsv 1445 -> 1445 changed 0 added 0 removed 0
unknown_captures.tsv 2938 -> 2938 changed 0 added 0 removed 0
unknown_macros.tsv 3114 -> 3114 changed 0 added 0 removed 0
unknown_storage.tsv 88 -> 88 changed 0 added 0 removed 0
self_reaching.tsv 59 -> 59 changed 0 added 0 removed 0
```

All 68 local contract fingerprints and all four certificate cells (C01/C02 × default/hash feature true and no-default/hash feature false) replayed unchanged. Both feature-set `local_production_acceptance` commands exited 0 without copying a changed value. Structured maintenance result:

```json
{
  "fingerprint_rows_changed": 0,
  "certificate_cells_replayed": 4,
  "certificate_cells_changed": 0,
  "feature_set_replay_exits": {
    "default": 0,
    "no-default": 0
  }
}
```

The first reconciliation review used the same scratch filename as its command metadata, so the logger replaced that review JSON. The review was rerun once with the distinct `ownership-reconciliation.json` path; the six results remained identical. Both command logs and the corrected review are preserved. This repeated only a file comparison, not a benchmark or Cargo test.

**Other bounded driver commands.** These are the commands whose raw stdout contains the per-process JSON receipts. Each metadata file records the same environment and bound as the commands above; the complete logs are in the transcript. The table lists each command verbatim with its own bound and exit. Each lower-level metric/profiling process had its independent bound recorded in the manifest.

| Log prefix | Command after bounded environment | Bound (s) | Exit |
| --- | --- | ---: | ---: |
| `base/alloc-panel` | `python3 .temp/brief-565/metric-panel.py alloc base 1` | 1200 | 0 |
| `base/check-panel` | `python3 .temp/brief-565/check-panel.py base` | 300 | 0 |
| `base/instructions-1` | `python3 .temp/brief-565/metric-panel.py instructions base 1` | 1200 | 0 |
| `base/instructions-2` | `python3 .temp/brief-565/metric-panel.py instructions base 2` | 1200 | 0 |
| `incoming/alloc-panel` | `python3 .temp/brief-565/metric-panel.py alloc incoming 1` | 1200 | 0 |
| `incoming/check-panel` | `python3 .temp/brief-565/check-panel.py incoming` | 300 | 0 |
| `incoming/instructions-1` | `python3 .temp/brief-565/metric-panel.py instructions incoming 1` | 1200 | 0 |
| `incoming/instructions-2` | `python3 .temp/brief-565/metric-panel.py instructions incoming 2` | 1200 | 0 |
| `candidate-01/alloc-panel` | `python3 .temp/brief-565/metric-panel.py alloc candidate-01 1 .temp/brief-565/screen-labels.json` | 1200 | 0 |
| `candidate-01/check-panel` | `python3 .temp/brief-565/check-panel.py candidate-01` | 300 | 0 |
| `candidate-01/instructions-1` | `python3 .temp/brief-565/metric-panel.py instructions candidate-01 1 .temp/brief-565/screen-labels.json` | 1200 | 0 |
| `candidate-01/instructions-2` | `python3 .temp/brief-565/metric-panel.py instructions candidate-01 2 .temp/brief-565/screen-labels.json` | 1200 | 0 |
| `candidate-01/remainder-alloc-panel` | `python3 .temp/brief-565/metric-panel.py alloc candidate-01 remainder-1 .temp/brief-565/remainder-labels.json` | 1200 | 0 |
| `candidate-01/remainder-instructions-1` | `python3 .temp/brief-565/metric-panel.py instructions candidate-01 remainder-1 .temp/brief-565/remainder-labels.json` | 1200 | 0 |
| `candidate-01/remainder-instructions-2` | `python3 .temp/brief-565/metric-panel.py instructions candidate-01 remainder-2 .temp/brief-565/remainder-labels.json` | 1200 | 0 |
| `candidate-02/alloc-panel` | `python3 .temp/brief-565/metric-panel.py alloc candidate-02 1 .temp/brief-565/screen-labels.json` | 1200 | 0 |
| `candidate-02/check-panel` | `python3 .temp/brief-565/check-panel.py candidate-02` | 300 | 0 |
| `candidate-02/instructions-1` | `python3 .temp/brief-565/metric-panel.py instructions candidate-02 1 .temp/brief-565/screen-labels.json` | 1200 | 0 |
| `candidate-02/instructions-2` | `python3 .temp/brief-565/metric-panel.py instructions candidate-02 2 .temp/brief-565/screen-labels.json` | 1200 | 0 |
| `candidate-02/remainder-alloc-panel` | `python3 .temp/brief-565/metric-panel.py alloc candidate-02 remainder-1 .temp/brief-565/remainder-labels.json` | 1200 | 0 |
| `candidate-02/remainder-instructions-1` | `python3 .temp/brief-565/metric-panel.py instructions candidate-02 remainder-1 .temp/brief-565/remainder-labels.json` | 1200 | 0 |
| `candidate-02/remainder-instructions-2` | `python3 .temp/brief-565/metric-panel.py instructions candidate-02 remainder-2 .temp/brief-565/remainder-labels.json` | 1200 | 0 |
| `attribution/first-families` | `python3 .temp/brief-565/attribute.py take-multispec-small lazy-range-take-small lazy-range-normal-small lazy-table-construct-short lazy-table-construct-full` | 1800 | 0 |
| `attribution/remaining-families` | `python3 .temp/brief-565/attribute.py lazy-map-realize-small lazy-select-realize-small lazy-select-scan-small lazy-nestlist-realize-small lazy-foldlist-realize-small lazy-drop-realize-small lazy-first-small lazy-part-small lazy-transparent-adapters-small lazy-table-realize-small lazy-permutations-take-small lazy-subsets-take-small lazy-tuples-take-small lazy-depth-map-build-8 lazy-depth-map-first-8 lazy-depth-select-restart-8 lazy-depth-drop-all-one-8 lazy-depth-take-map-8 lazy-depth-fold-through-source-8 lazy-depth-nest-map-8 lazy-depth-drop-all-one-64 lazy-depth-take-map-64 lazy-depth-fold-through-source-64 lazy-depth-select-restart-256` | 3600 | 0 |

Completed lower-level executions (all exits 0):

| Execution | Processes | Individual bound (s) |
| --- | ---: | ---: |
| alloc | 432 | 150 |
| instructions | 864 | 150 |
| value-check | 216 | 60 |
| attribution-callgrind | 116 | 150 |
| callgrind-annotate | 232 | 150 |
| attribution-perf | 116 | 150 |

Annotation exits were asserted by `attribute.py` before the completed per-profile metadata was written; they are not independent saved exit-status files. The exact annotation commands are reconstructed from that script in the manifest, with full unedited annotation output retained. The raw `.command` files for metrics and Callgrind/perf were written before those runs.

### 5. Progress counters, disclosures, and remaining attribution

The progress comparator is the **retained** source baseline. Candidate 01 was rejected and restored before candidate 02, so the tiny decrease from candidate 01’s worst sample to candidate 02’s is not accepted-tree progress. Neither candidate reduced 29 failing labels or the incoming worst delta +2.5183316981561843%. Each also independently violated the prohibition on newly failing neighbors. Required stop-loss is **2 consecutive non-progress candidates / 2**; measured candidates are **2 / 10 maximum**. Curve, verbatim:

```tsv
build	state	labels_above_threshold	largest_instruction_pct	measured_labels	consecutive_nonprogress
incoming	reference	29	2.5183316981561843	54	0
candidate-01	rejected	29	2.520369106211673	54	1
candidate-02	rejected	29	2.5201535513410978	54	2
```

Residual attribution for all 29 retained failures is in §2, with full acceptance rows in §3. The following groups describe what the evidence supports; they do not claim an unmeasured fix:

| Residual family | Evidence and remaining limit |
| --- | --- |
| Ordinary `Take` | Iterator/selector code attribution moves by about 309 instructions; the lazy-branch attribute probe failed. Changing ordinary selector logic would be outside scope. |
| `LazyTable` construction | Extra clone/drop work is measured; moving it cold helped the full constructor but failed the full-panel gate and was restored. |
| Source-only range/combinatorial realization | New `drive_pull` / `drive_items` work is layered over source work similar to old `next`; no third implementation was attempted. |
| Map/Select/Fold/Nest, table callbacks and composed consumers | Driver/descriptor transitions and required callback checks account for identifiable movements; their per-label self/inclusive rows are recorded. Cancellation checks cannot be removed. |
| Shallow and deep descriptor chains | Flat descriptor scans, stage setup, driver costs and removed recursive work trade off differently by label. The 8/64/256-depth tables show both additions and savings. |
| Very small or inconsistent residuals | Scaled Callgrind totals do not reproduce the sign of every small full-panel regression (including depth-nest-map-8 and depth-select-restart-8/256). The calibration and sample spread prevent assigning a precise source-only cause; the hardware gate still counts them as failures. |

**What did not run or remain:** no third candidate; no implementation of the shallow-driver hypothesis; no separate First/Normal code-shape experiment; no timing benchmark (the brief explicitly makes instructions decisive); no pre-G2 rebuild; no final rebuild of the byte-identical restored incoming tree; no repeated full-panel sample; no candidate-specific full workspace/retention suite on either rejected experiment. Candidate 01 had compilation/value checks only; candidate 02 additionally had formatting and the 69 continuation tests. All mandatory suites and retention work above ran on the restored final source. The 58 ignored tests were not all explicitly run; only the mandated ignored fingerprint regeneration was invoked separately. Nine native ZeroMQ tests skipped. No git command, network, test edit, semantic relaxation, runtime/core production edit, new dependency, unsafe, recursion or depth cap was introduced.

All measured binaries and source snapshots remain available for director review. [Final checkpoint](../../../.temp/brief-565/PROGRESS.md) records the stop and exact tree state. The ordinary-path target is **unfulfilled**; this report hands back the required failed-candidate evidence without retaining either regression.

