# Brief 668 — LP06 source stage: the rebound carrier with the measured fixed root

**Shape: implementation.** The LP06 F design line closed at brief 665 with a
measured carrier; 666 landed it and stopped at the fixed debug root; 667
found the root form and measured it. This brief lands the 666 surface plus
the 667 form in the live tree in one stage and runs the full source-stage
campaign on the exact final source. The tree is left for the director's
gate; no commit.

**Baseline ruling (read before anything else).** The literal red/green
contrast is measured **on the surface-only arm**
`.temp/brief-665/probe/red-sem` against the final tree — never against clean
HEAD. Clean HEAD does not contain `Call::start_from_active` (part of the
calibrated surface), so the unchanged literal bodies do not compile there,
and its two host reset falsifiers are green because the surface is what
exposes the missing carrier. The 39 red + 1 green control bar stands,
measured on that arm; the 665 records hold `literal_lp658_host_eval_reset`
`red_passed=False, design_passed=True`, `literal_lp658_host_resume_reset` the
same, and `literal_lp657_host_root_control` `red_passed=True,
design_passed=True`. A clean-HEAD run is not evidence of the contrast and is
not to be substituted.

## Goal

When this brief is done, the live tree implements the brief-665
rebound-carrier production shape with the brief-667 fixed-root form, and on
the exact source being proposed:

- `resume_eval_request_from_checkpoint` is emitted exactly as the red arm
  after decoded address relocation: 4,606 live instructions / 25,861 B,
  4,693 own Ir per request in both draws, zero real text differences,
  unchanged direct/indirect targets and own-PC counts; no new guard, field,
  local, `Drop`, call, branch or TLS access in that body;
- `evaluate_at_root` and the abandonment entry add exactly the approved check
  sequence and nothing else: 223 -> 231 (+8) on the root harness and
  1262 -> 1270 on the supplied abandonment harness;
- **`evaluate_at_root`'s fixed debug root is 2,528 B at every one of the 88
  supplied shape/height points, equal to the red arm.** The form is pinned:
  `ExecutionScope::enter` is `#[inline]` (not `#[inline(always)]`); its Drop
  hint stays forced inline; the marker stays zero-sized; no other carrier
  line moves;
- the six phase-only free starts and the direct private resume read the
  resumed session's **current** request control through cap retirement,
  external reset, rollover, thread handoff, nested second-session resume and
  unwind;
- no default-path instruction is added: `drive_input_request` stays 88 own
  Ir/call on all 121 resume labels in both draws with equal incoming slopes,
  the simple entry stays 2,223, and admission is 63 Ir/admission on both
  routes and both draws;
- every round-9 semantics holds: lp06 36/36, transitions 11/11, entries 23/23,
  combined 4/4, host 3/3; the literal 39 red falsifiers plus one green
  host-root control **on the surface-only arm**, and all 40 green here, with
  no assertion rewritten; the round-665 cap falsifier and the three new
  stale/foreign falsifiers red on that arm and green here with identical
  formatted bodies;
- the two workspace configurations are green at 8 test threads, including
  the lease test (interference named and fixed, not serialized away) and the
  census assertion (its constants ruled below);
- the 29 own and incoming pairs are exact, F7 is at most 856, the native
  shapes equal the red arm, the whole D campaign is measured on the proposed
  source, the four repeated panel positives do not become scaling positives,
  every affected inventory is replayed, and `cargo fmt --check` is green on
  the exact measured source.

## Context pointers

- `.temp/director/brief-665-gate.md`, `brief-666-gate.md`,
  `brief-667-gate.md` — the rulings, the reproduced rows, the baseline
  correction. Read all three first.
- `.temp/brief-667/report-design.json` — the seven-form family table, the
  calibration and the selected form's rows; `family-rows.json`; the
  `probe/inline-enter` tree (the measured final source, byte-identical to my
  independent build); `head-vs-retained-red.diff` and the HEAD-contrast log.
- `.temp/brief-666/report.json` — the stopped stage: the pinned surface, the
  88 captures, the replays, the census comparison
  (`census-comparison.json`: 1067 paired, 0 removed, 63 added, kinds
  10/1083/37).
- `.temp/brief-666/live-source.diff` and `proposed-source.patch` — the 666
  surface archive; `.temp/director/gate-666/head/` — a clean HEAD tree.
- `.temp/brief-665/probe/rebound-carrier-sem` — the measured semantic arm;
  `.temp/brief-665/probe/red-sem` — the surface-only red arm.
- `.temp/brief-665/d-instruments/` — the frame and matrix instruments;
  `physical-frame-differences.json` — the 22 x 4 shapes.
- `.temp/brief-665/{semantic-proof-rebound-carrier.json,cap-stale-foreign-proof.json,panel-two-draw-bands.json,input-drive-121.json,hard-rows-red-rebound-carrier.json,resume-emission-rebound-carrier.json,admission-chain.json,rollover-cost/,d-instruments/}` — the reference rows; reuse, never redraw.
- Live sites: `profiling/logical/context.rs`, `logical/mod.rs`,
  `resolution.rs`, `suspension/core.rs`, `suspension/core/driver.rs`,
  `suspension/core/{profile_mode.rs,profile_queue.rs,storage.rs,abandon.rs}`,
  `session/session_api.rs`, `session/session_requests.rs`,
  `builtins_input.rs`, `profiling/timing.rs`, `profiling/config.rs`,
  `eval_dispatch/mod.rs`, `kernel_data/control_data/branching_loops.rs`,
  `kernel_data/control_data/nonlocal.rs`, `scoping/callable.rs`,
  `scoping/callable/status_application.rs`.

## Scope

Files you MAY modify:

1. Exactly the brief-665 source surface (19 files), materialized from
   `.temp/brief-667/probe/inline-enter` and verified byte-for-byte:
   - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
   - `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs`
   - `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs`
   - `crates/rum-runtime/src/runtime/profiling/logical/context.rs`
   - `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`
   - `crates/rum-runtime/src/runtime/profiling/tests/control.rs`
   - `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs`
   - `crates/rum-runtime/src/runtime/profiling/timing.rs`
   - `crates/rum-runtime/src/runtime/resolution.rs`
   - `crates/rum-runtime/src/runtime/scoping/callable.rs`
   - `crates/rum-runtime/src/runtime/scoping/callable/status_application.rs`
   - `crates/rum-runtime/src/runtime/session/session_api.rs`
   - `crates/rum-runtime/src/runtime/suspension/core.rs`
   - `crates/rum-runtime/src/runtime/suspension/core/abandon.rs`
   - `crates/rum-runtime/src/runtime/suspension/core/profile_mode.rs`
   - `crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs`
   - `crates/rum-runtime/src/runtime/suspension/core/profile_queue/tests.rs`
   - `crates/rum-runtime/src/runtime/suspension/core/storage.rs`
   - `crates/rum-host/tests/lp03_phase_rows.rs`
2. The census assertion constants in
   `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` — **only**
   the total (1067 -> 1130) and the kind map (10/1022/35 -> 10/1083/37), plus
   one comment line recording this ruling. This is the director's ruling, not
   a hand edit: it holds only if the final replay's comparison equals the 666
   comparison row for row. Any difference is a STOP with the diff.
3. The lease interference fix, test-only code only:
   `crates/rum-runtime/src/runtime/profiling/tests/lease_release.rs`,
   `crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs`,
   `crates/rum-runtime/src/runtime/profiling/lease.rs`, and — only if the
   shared cell is mutated there — one other test module, disclosed with the
   reason. The assertion text and the default 8-thread concurrency stay
   unchanged. A fix that needs production code is a STOP: name the cell and
   the required change instead.

Plus the standing-inventory replays the template makes always in scope
(contracts, certificates, positions, ownership, LP01, typed-slot, census
definition). Replays are generated; never hand-edit a derived cell.

MUST NOT touch: the suspension driver (`suspension/core/driver.rs`), the
typed-resume macro, the checked adapter, `session_requests.rs`,
`builtins_input.rs`, the public API, features, serde, manifests, workloads,
the R7 boundary, and anything outside the lists above. A second
implementation stage may only be opened by the director.

## Target shape (pinned)

Materialize the 19 files from `.temp/brief-667/probe/inline-enter` — that tree
is the measured 2528 B form and is byte-identical to my independent build of
HEAD + the 666 archive + the one-line change. Do not re-derive the form. The
carrier representation, field order, bodies, call structure, pushed value,
push/pop conditions, the two attachment sites, the resume hook and
`drive_frames`'s `let profile_context = M::resume(self, &outcome)` /
`M::pause(&profile_context)` stay exactly as in that tree. The one form line
is `ExecutionScope::enter`'s `#[inline]`; its `Drop` hint remains
`#[inline(always)]`.

Keep the measured `resume_profile_configured` code shape
(`configured && (enabled || cold configured fallback)`) and its comment;
round 10's syntactic simplification moved `drive_input_request` by one
instruction on all 121 labels.

## Steps

1. **Materialize.** Copy the 19 files from the measured form tree, verify
   each copy byte-for-byte, then confirm the tree compiles.
2. **Replay.** After the last source edit, replay the standing inventories
   (contracts, certificates, positions, ownership, LP01, typed-slot, census
   definition), copy the generated cells, and disclose every new row. Then
   rule the census constants under the scope item above.
3. **Shape stop.** Before any campaign: checkpoint byte identity, the driver
   row, the debug root and one semantics run. A moved checkpoint byte, a
   driver +1, a 2544 root or a semantics red stops before the expensive
   panels.
4. **Semantics with the corrected baseline.** Run the 40-case literal set on
   the final source (all green) and on `.temp/brief-665/probe/red-sem`
   (39 red + the one green host-root control), plus the cap and three
   stale/foreign falsifiers red on that arm and green here, byte-identical
   bodies.
5. **Deterministic rows.** Checkpoint, entry/admission, 29/29, driver,
   native/rollover.
6. **Lease.** Name the shared cell behind the 8-thread failure, fix the
   interference in test-only code, and prove both workspaces green at 8
   threads with the assertion and the concurrency unchanged.
7. **Campaign.** The whole D campaign on the exact final source, then both
   workspace configurations and the required-switch host suites, then timing
   last, serially and alone.

## Hard bars (deterministic)

- Checkpoint: 4,606 live instructions / 25,861 B; 4,693 own Ir both draws;
  zero normalized byte differences; unchanged direct and indirect targets;
  identical executed own-PC counts.
- Simple entry 2,223; root 223 -> 231 (+8); supplied abandonment harness
  1,262 -> 1,270 (+8 check only). Preserve the literal 223 -> 231
  abandonment discrepancy for the director; do not relabel the workload.
- **Fixed debug root 2,528 B at all 88 points**, red-equal, in two
  independent builds.
- 29/29 own and incoming N/2N pairs exactly equal the supplied pre-A1
  reference, both draws. All six free-start owner slopes stay red; the While
  owner is 669's and must be reported, not claimed.
- `drive_input_request` 88 own Ir/call on all 121 resume labels, both draws,
  equal incoming slopes.
- Admission 63 Ir/admission on resume and abandonment, both draws; no larger
  than 77, and no owner may offset another.
- F7 <= 856 per specialization; CLI reservations 600/632 individually equal
  the red arm. Native shapes equal the red arm: While 776/3528/776,
  Compound 696/1304/680, Catch 1000/792/1000, forwarded 24/88/24
  (CLI/debug/release).
- Cold rollover: measured separately, both draws; not larger than 184/184
  (disabled) versus 213/213 and 242/242 (enabled). Cold-only; never a
  hot-path allowance.
- Semantics: lp06 36/36, transitions 11/11, entries 23/23, combined 4/4,
  host 3/3; 39 red on the surface-only arm plus the one green host-root
  control, 40/40 here; the cap and three stale/foreign falsifiers red on the
  surface arm and green here, byte-identical bodies. No assertion rewritten.
- D campaign: 313-label instruction panel (two draws, per-label untouched
  band), 192 copy labels, 194 allocation labels, 44+18 stack rows, 352
  physical-frame captures, layouts, native prologues, lifecycle controls,
  canonical 120+9 timing labels. The four repeated positives may not become
  scaling positives; a moved decoded byte is a defect.
- Both workspaces green at 8 threads (default and no-default), the census
  constants at the replayed values, the lease fix proven, the required-switch
  host suites green, both source gates, ownership replay, census
  definition/grammar and `cargo fmt --check` green.

## Verification

One cargo command at a time inside the dispatcher's scope; jobs 8, test
threads 8, offline/locked. Bound anything that may spawn children with
`scripts/run-bounded.py`. Reproduce every unexpected red at
`--test-threads=1` before calling it a defect; an interference red is fixed,
not serialized away. Performance captures run serially and alone on CPU 2,
release, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, both arms at one common
source path, every `RUM_PROFILE_*` cleared. Reuse completed reference
receipts; never redraw an accepted reference.

Report `.temp/brief-668/report.json` in `rum-report/1`, with every run, every
attempt and every verdict line quoted; run `python3 -I scripts/report.py
check 668` and return its summary and all compliance verdict lines.

## Checkpointing and concurrency

All scratch under `.temp/brief-668/`; keep `PROGRESS.md` with the exact next
action, the compiled source state and the receipts already saved. Leave the
tree compiling at each checkpoint. One cargo command at a time; tests at
`RUST_TEST_THREADS=8 --no-fail-fast`; builds at `CARGO_BUILD_JOBS=8`. On an
exit 137 with `oom_kill` risen, rerun that one command once with one job.

## Hard rules

- No version-control command, network, new `unsafe`, public API/feature/serde
  change, workload change, inventory weakening or hand-edited derived cell.
- No new checkpoint guard/local/token/`Drop`, dispatcher guard, typed-macro or
  checked-adapter change, branch in the checkpoint function, or default-path
  instruction outside the approved root/abandon check sequence.
- No `debug_assertions`-keyed attribute or debug-specific policy.
- No external-engine comparison or committed third-party output; rum-native
  A/B only.
- No R7 repair; no completion/forwarded literal change beyond the measured
  shape.
- Every claim carries a citation the checker can resolve; every FLAGGED row
  carries a note that settles it. The director's gate owns acceptance.
