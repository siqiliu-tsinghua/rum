# Brief 673 — LP06 source stage: the shared-token adapter (the frame back to base)

**Shape: implementation.** Brief 672 measured the form; this brief lands
it in the live tree and runs the source-stage campaign on the exact final
source. The tree is left for the director's gate; no commit.

**Director ruling (read before anything else).** 672 stopped on my
brief's literal wording — ordinary CLI 1,048 B and six exactly equal
slopes — while measuring 1,032 B and four equal plus two 32-B-below rows.
I rule the bar as **no growth over base**, not literal equality: a
smaller frame is the hard line satisfied more strongly, and every
measured form that lands on literal 1,048 fails the eight Function F6
rows with +1 own instruction per call. The two −32 rows are not target
growth: the physical-owner attribution puts `try_apply_function` at delta
0 in all six shapes and the −32 belongs to
`evaluate_status_function_value` (704 → 672), a saving. Do not add
padding, a different shape or an instruction to reach 1,048. The target
values below are the measured contract.

## Goal

When this brief is done, the live tree contains exactly the measured
adapter form and nothing else, and on the exact final source:

- the ordinary release CLI `try_apply_function` prologue is 6 pushes +
  `sub $0x408` = **1,032 B** in two independent builds, no padding; the
  release stack probe stays `sub $0x418` = 1,048 B static / 1,104 B
  physical; the debug stack probe stays 2,808 + 8 return address =
  **2,816 B per level** at every height, in two builds;
- the six ordinary rows, both draws, zero residual (base `654f2ce5` →
  final): debug `function-body` 49,632 → 49,632, `status-body` 51,136 →
  51,136, `message-body` 51,136 → 51,136; release `function-body` 18,304
  → 18,304, `status-body` 19,424 → 19,392, `message-body` 19,424 →
  19,392. All 44 ordinary rows fit with zero residual and **none above
  base**; the `try_apply_function` physical-owner delta is 0 in all six
  shapes and the two −32 rows attribute to
  `evaluate_status_function_value`;
- the 8 Function and 21 builtin F6 labels return exact fresh base
  own+incoming N/2N pairs, both draws; the by-value control still reads
  red (`own` 25,050 against base 24,549 at N=500, +1 per call), so the
  instrument is seen to fire;
- F7 is `[632, 600]` with each specialization under 856; the three While
  owners are 202/564/623 against base 234/656/741 with equal calls
  (1/1/4), each label and both draws independently; the enabled controls
  are 12/12 with `open=0 leases=0 duplicate_finish=0`; the unchanged
  lease falsifier and lease-release test pass at 8 threads;
- the carrier semantics are untouched and the two workspace
  configurations, the required-switch host suites, the standing inventory
  replays, `cargo fmt --check` and the rum-vs-rum core-language microbench
  A/B are green on the exact final source.

## Context pointers

- `.temp/director/brief-672-gate.md` — my ruling and my reproductions.
  Read it first.
- `.temp/brief-672/report-design.json`, `decisions.log`,
  `form-matrix.json` — the form families, the calibration and the
  measured rows.
- `.temp/brief-672/src/selected/` — the measured form tree. Copy from
  it; never edit it in place, never build in it.
- `.temp/brief-672/forms/selected.diff` and `selected.json` — the exact
  two-file change and its receipt.
- `.temp/brief-672/inputs/` — frozen programs, `labels29.json`, the
  Callgrind reader, the frame-CFI reader and the perf harness. Copy;
  never execute in place.
- `.temp/brief-672/measure/` and `.temp/brief-672/bin/` — the design's
  receipts and pinned binaries. Read/copy only; do not redraw the
  accepted base references unless the brief's own protocol actually
  rebuilds that arm (it does — see Steps 3–4).
- `.temp/brief-671/src/{base,candidate}` — the `654f2ce5` and
  `19b61027` exports, read-only, with their hash manifests.
- Live sites: `crates/rum-runtime/src/runtime/scoping/callable.rs` (the
  call site) and `crates/rum-runtime/src/runtime/profiling/timing.rs`
  (the adapter). `scoping/callable/status_application.rs`, the logical
  carrier/epoch modules and the suspension modules stay byte-identical.

## Scope

Files you MAY modify:

1. `crates/rum-runtime/src/runtime/scoping/callable.rs`
2. `crates/rum-runtime/src/runtime/profiling/timing.rs`

Plus the standing-inventory replays the template makes always in scope
(contract fingerprints, default/no-default certificates, ownership
positions/unknowns, LP01 producer receipts, typed-slot receipts and the
census discovery). Replays are generated; **never hand-edit a derived
cell**. Disclose every changed row and every new unknown row.

MUST NOT touch: `scoping/callable/status_application.rs`, the recorder
`record_profile_runtime_path_timing_adopted_boundary_slow`, the existing
`record_profile_runtime_path_timing_adopted_boundary`, the logical
carrier/epoch modules, the suspension core and driver, the typed macro,
the checked adapter, the public API, features, serde, manifests,
`Cargo.lock`, workloads, docs, or anything outside the two files above.
A second implementation stage may only be opened by the director.

## Target shape (pinned)

Materialize the two files from `.temp/brief-672/src/selected/` and verify
each copy byte-for-byte against that measured tree. Do not re-derive the
form. The shape is:

```rust
let runtime_profile_evaluate = runtime_path_timing_start("function.apply.evaluate");
let applied_result = self.evaluate(applied);
self.finish_profile_runtime_path_timing_adopted_boundary(
    &runtime_profile_evaluate,
    &applied_result,
);
```

The continuation consumes `applied_result` exactly once, unchanged. In
`timing.rs`, the inline adapter takes
`token: &Option<RuntimePathTimingToken>` and
`produced: &Result<T, EvalError>` and returns `()`; its
`let Some(token) = token else { return; };` guard delegates to the cold,
never-inlined helper, which takes references, reconstructs the token from
its two Copy fields (`start`, `key`) and calls the byte-identical
recorder. `RuntimePathTimingToken` has no Drop and no other fields.

**Must not change:** the start → evaluate → finish-adopted → continuation
order; the pushed value; the push/pop conditions; the two attachment
sites; the checkpoint body; the root/abandon check sequences; the
`ContextGuard` niche; the optional epoch/storage; the resume hook; the
lease drain; the phase/epoch behaviour. **No** duplicated body,
silent-drop helper, lazy initialization, heap substitution, artificial
frame padding, `debug_assertions`-keyed attribute, or default-path
allocation/call.

## Steps

1. **Materialize.** Copy the two files from the measured tree, verify
   byte identity, then compile the exact form offline/locked, jobs 8.
2. **Replay.** After the last source edit, replay the standing
   inventories (contract fingerprints, default/no-default certificates,
   ownership positions/unknowns, LP01 producer receipts, typed-slot
   receipts, census discovery) before the first acceptance suite. Stop
   for the template's identity/disposition/`UnknownHostPayload`/census
   decision conditions; an inventory-only red is an ordering artifact —
   replay the generator, never weaken the test.
3. **Shape stop.** Before any campaign: the two file copies, the ordinary
   CLI prologue `sub $0x408`, the release probe `sub $0x418`, the debug
   probe `sub $0xaf8`, one focused runtime run. A moved file, a different
   reservation or a red stops before the expensive panels.
4. **Campaign.** Build base `654f2ce5` and the final source independently
   at one common source path with release CGU=1, two independent builds
   per arm; measure both arms in two draws: the six rows at the four
   retained heights, all 44 ordinary rows, the 29 F6 pairs, F7, the three
   While owners and the enabled controls. Calibrate the F6 instrument on
   the by-value control before judging the form.
5. **Lease and acceptance.** The unchanged lease falsifier and
   lease-release test at 8 threads; the focused runtime suite, then both
   workspace configurations at 8 threads with the seven standing skips
   (below), the required-switch host suites and `cargo fmt --check`.
6. **Timing.** The rum-vs-rum core-language microbench A/B on base and
   the exact final source, same protocol as the 668 stage, serially and
   alone, after every deterministic panel.

## Hard bars

- Ordinary release CLI `try_apply_function` 1,032 B (`sub $0x408`), no
  row above base; release stack probe 1,048 B / 1,104 B physical; debug
  2,816 B per level, all heights, two builds. The values are a ceiling
  and an exactness on the same row, never a sum.
- The six rows at the measured values above, both draws, zero residual;
  all 44 ordinary rows zero residual and none above base; the
  `try_apply_function` owner delta 0 in all six shapes; the −32 rows
  attributed to `evaluate_status_function_value`, never offset by another
  row and never hidden in a sum.
- 8 Function + 21 builtin F6 labels exact fresh base own and incoming
  N/2N pairs, both draws — not merely equal fitted slopes. The by-value
  control red is reproduced first.
- F7 `[632, 600]`, each under 856. While owners 202/564/623, incoming
  calls equal, three labels × both draws. Enabled 12/12, frozen N/2N
  workloads, four phase counts exact, all six prompts per resumed item,
  stable epoch/adopted identities, zero open/lease/duplicate-finish.
- Lease falsifier and lease-release test green at 8 threads; assertion
  text and the 8-thread default unchanged.
- Both workspace configurations green at 8 threads with the seven
  standing skips: `lp03_adopted_phases_are_exact_in_completed_and_resumed_runs`,
  `lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs`,
  `lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs`,
  `lp658_host_eval_reset`, `lp658_host_resume_reset`,
  `lp655_root_context_falsifier`, `lp656_public_root_context_falsifier`.
  The lease test is **not** skipped.
- The required-switch host suites run those skips: `lp03_phase_rows`
  alone, `lp03_phase_rows` + `lp04_definition_rows` +
  `lp05_assignment_rows` in default and `--no-default-features`, the
  `rum-runtime` `lp657_host_root_control`, and the two `lp658` resets
  exact.
- `cargo fmt --check` green; the standing inventories replayed and
  disclosed; public contracts, manifests and workloads unchanged.
- No new `unsafe`, no network, no version-control command, no
  third-party comparison.

## Verification

One cargo command at a time inside the dispatcher's 16G scope, plain
(never nested); jobs 8, test threads 8, offline/locked. Bound anything
that may spawn children with `scripts/run-bounded.py`; derive the bound
from a prior measured run of the same shape. A red is reproduced at
`--test-threads=1` before it is called a defect; an interference red is
fixed, never serialized away. A timeout is not data. Performance
captures run serially and alone on CPU 2, release, CGU=1, both arms at
one common source path, every `RUM_PROFILE_*` cleared except the declared
enabled controls. Raw output is saved before parsing; every failed
attempt is retained.

## Checkpointing

All scratch under `.temp/brief-673/`; keep `PROGRESS.md` with the exact
next action, the compiled source state and the receipts already saved.
Leave the tree compiling at each checkpoint.

## Hard rules

- No version-control command, network, new `unsafe`, public
  API/feature/serde change, workload change, inventory weakening or
  hand-edited derived cell.
- Apply only the exact two-file measured form and the canonical generated
  inventory refresh. A departure from that shape is a STOP for director
  review.
- Every claim carries a citation the checker can resolve; every FLAGGED
  row carries a note that settles it. The director's gate owns
  acceptance.

## Deliverable

- The two production files plus the canonical generated inventory
  updates, and the complete raw-backed row set.
- `.temp/brief-673/report.json` in `rum-report/1`, every run and verdict
  line quoted, every attempt listed.
- Run `python3 -I scripts/report.py check 673` and return its summary and
  every compliance verdict line.
