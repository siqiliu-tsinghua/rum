# brief-566 — erase the lazy ordinary-path cost, measured with one codegen unit

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

brief-565 tried to erase the ordinary-path cost G2 (briefs 563-564) added and
stopped at its stop-loss after two candidates, with the tree restored. Its
attribution (§2 of its report) is good. Its **measurement** was not, and that
was the director's error: the release profile has 16 codegen units, so adding
code anywhere in `rum-stdlib` repartitions them and moves inlining in code the
change never touched. Rebuilt with one codegen unit
(`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`), the director measured:

| Label | Pre-G2 | Now | Delta, sample 1 / 2 |
| --- | ---: | ---: | --- |
| `take-multispec-small` (ordinary list `Take`) | 49 924 | 49 922 | -0.003% / -0.004% |
| `dot-three-no-input`, `differences-no-input`, `table-small`, `do-count-null` (controls) | — | — | all within ±0.02% |
| `lazy-depth-drop-all-one-8` | 48 741 | 49 741 | +2.05% / +2.06% |
| `lazy-range-normal-small` | 53 959 | 54 845 | +1.64% / +1.42% |
| `lazy-range-take-small` | 42 153 | 42 686 | +1.26% / +1.26% |
| `lazy-subsets-take-small` | 25 330 | 25 625 | +1.17% / +0.98% |
| `lazy-foldlist-realize-small` | 112 692 | 113 839 | +1.02% / +1.07% |
| `lazy-table-construct-short` / `-full` | 23 599 / 24 135 | 23 767 / 24 254 | +0.71% / +0.98%; +0.49% / +0.99% |
| `lazy-permutations-take-small` | 28 180 | 28 390 | +0.74% / +0.94% |
| `lazy-table-realize-small` | 117 144 | 118 159 | +0.87% / +0.89% |
| `lazy-select-realize-small` | 114 677 | 115 489 | +0.71% / +0.83% |
| `lazy-depth-map-build-8` | 19 235 | 19 384 | +0.78% / +0.83% |

and `lazy-nestlist-realize-small`, `-depth-take-map-8`, `-part-small`,
`-tuples-take-small`, `-map-realize-small`, `-drop-realize-small`,
`-first-small`, `-transparent-adapters-small`, `-select-scan-small` between
+0.17% and +0.75% (raw rows in `.temp/director/gate-565/perf/`).

So: **G2 costs the ordinary non-lazy path nothing** — brief-565's +0.6% on
`Take` was partition noise, and so were the neighbour regressions that rejected
its candidate 02 (+0.07%, +0.31%, +0.18% in one sample each). **G2 does cost
shallow lazy realizations 0.2-2%**, and that is real. Erase it.

**Target:** every one of the 54 panel labels within +0.05% of the pre-G2 base,
both builds compiled with one codegen unit. A label counts as regressing when
**both** samples exceed +0.05%. Every correctness test stays unchanged and
green. If an honest effort leaves a residual, the report attributes it to the
specific work the flat machine does that the recursive one did not.

## Fixed — not negotiable in this brief

- **The flat representation and its depth contract.** One source plus a stage
  vector under an indexed cursor; no walk recurses in proportion to depth; the
  structural checks (`tests/association_depth_contract/lazy_depth_contract.rs`)
  and the 2 000-stage depth tests stay unchanged and green. **No regression is
  bought back by restoring recursion** — the recursion was a process abort.
- **Semantics.** `crates/rum-stdlib/tests/lazy_pull_continuations.rs` and its
  children, `loop_continuations.rs` and every other test stay exactly as they
  are. The cancellation checkpoint after each callback stays.

## Start here

1. **Re-measure brief-565's candidate 02 under one codegen unit** (source in
   `.temp/brief-565/candidate-02/source/`, diff in `source-change.diff`): it
   moves `LazyTable`'s bound recovery to the suspension edge and brought the
   full constructor to the base before partition noise rejected it.
2. **Attribute on one-unit binaries.** brief-565's callgrind tables were taken
   on 16-unit builds; repeat the attribution for the shallow lazy families on
   the one-unit base and current binaries (`valgrind --tool=callgrind`, each
   total calibrated against `perf stat` on the same binary). brief-565 §2
   already names the moved work: `machine_from_descriptor` and `push_stage`
   replacing `stepper_from_descriptor`, `drive_items` / `drive_pull` layered
   over source pulls, `Vec` drop, `Symbol::is_system_name` calls in descriptor
   parsing. Hypotheses, each to be measured: build per realization what is now
   built per item; a plain index on the ordinary entry rather than a cursor
   enum; a descriptor parse that does not re-test symbol names it has already
   classified; no `Vec` allocation or drop work for a stage-free machine where
   the representation allows it without a second code path that could drift.

## Measurement

- **Base:** `.temp/brief-566/base/rum-cli` and `rum-cli-alloc` — the pre-G2
  source (`ab78df14`'s crates, identical to `f8e73ac4`'s) built by the
  director with one codegen unit; hashes in `.temp/brief-566/base/sha256.txt`.
  Do not rebuild them.
- **Candidates:** build the working tree with
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into a fresh target directory per
  candidate under `.temp/brief-566/`, both default and
  `--features rum-cli/alloc-count`, and check the produced binary's hash
  changed (brief-564 caught a "fresh" build that left a stale binary). Keep
  every measured binary, its hash and its source. Measure the incoming tree
  once, the same way, as candidate 00.
- **Panel:** `.temp/brief-562/performance-panel.json`, 54 labels, workloads
  `.temp/brief-563/perf/<label>-<N>.rum`, result checks
  `.temp/brief-563/perf/<label>-check.rum` (check before measuring).
  `.temp/brief-565/metric-panel.py`, `compare-metrics.py` and `attribute.py`
  are a working harness to copy and adapt.
- **Method:** allocation counts first (`RUM_PROFILE_ALLOC=1`), then two
  samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label
  per process, stdin `/dev/null`, other `RUM_PROFILE_*` cleared,
  `(count(2N) - count(N)) / N`, raw output saved before parsing. An exploratory
  candidate may be screened on the lazy labels plus `take-multispec-small`,
  `dot-three-no-input`, `differences-no-input`, `table-small`; **any candidate
  you keep, and the final one, runs all 54.** No timing.

## Scope

- **Objective:** the ordinary-path shape of `crates/rum-stdlib/src/lazy.rs`,
  `crates/rum-stdlib/src/lazy/`, and the lazy branches of `eval_take`,
  `eval_first`, `eval_normal`.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`): replay
  the retention fingerprints and both certificates, reconcile the ownership
  inventories, keep rum-runtime's frame lists exact. New `UnknownConstruct`
  rows for code you add or change are expected and disclosed; STOP on a
  changed or vanished existing row identity, a new `UnknownHostPayload` or
  `self_reaching` row, or a row in code you did not touch.
- **Fence:** no semantic change and no test edit. No rum-runtime or rum-core
  production change, no `Expr` change, no other brief-548 group, no leases, no
  dependency, no `unsafe`, no depth cap, no recursion, no change to the release
  profile in `Cargo.toml` (one codegen unit is a measurement setting here, set
  by environment). Do not edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier
  briefs or docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  candidate — what it changed, its attribution, its rows.

## rum / Rust pitfalls (read twice)

1. **One variable per candidate.**
2. **A candidate that makes another label regress (both samples above +0.05%)
   is rejected**, a deep label pushed above the base included.
3. **Instruction counts decide.**
4. **Stop-loss is keyed on progress:** the number of regressing labels or the
   largest remaining delta must fall. Stop after two consecutive compiling
   candidates that reduce neither; at most ten measured candidates. Report the
   curve and the attribution of every residual.
5. `#[inline(always)]` is not a fix by itself (brief-560: +1.17% on
   `Differences`). Prefer moving cold work out over forcing hot work in.
6. A second ordinary code path kept alongside the general one (a "shallow
   specialization") is a drift risk; it needs its own falsifier proving both
   paths agree, or do not add it.

## Verification (MANDATORY — you run it; iterate to green)

Serial, every Cargo command through `python3 scripts/run-bounded.py --timeout
<s> -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-566/tmp …`. Bounds: a focused test target 150 s; a
one-unit release build 600 s (the director's took 98 s); a metric process 150 s;
`local_production_acceptance` and `cargo test -p rum-runtime` 300 s;
`cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test lazy_pull_continuations
cargo test -p rum-stdlib --lib lazy
cargo test -p rum-stdlib --test loop_continuations
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

Plus the retention replay in both feature sets. Paste commands, bounds, exit
statuses and counts verbatim; list native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-566/`. After each candidate write
`.temp/brief-566/PROGRESS.md`: its change, its attribution, its rows, the
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
2. The one-unit attribution for each lazy family, base against incoming.
3. Every candidate measured, with its rows (all 54 for kept candidates), both
   samples.
4. Verification — every command's verbatim result.
5. Progress counters, disclosures, and the attribution of anything left.

---
