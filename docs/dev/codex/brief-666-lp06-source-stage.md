# Brief 666 — LP06 source stage: execution control through the existing resume hook

**Shape: implementation.** The LP06 F design line closed at brief 665 with a
measured carrier: the checkpoint body byte-identical to HEAD, every inherited
and new falsifier green, the ordinary admission path 14 Ir cheaper, and three
rules I made at the gate (`.temp/director/brief-665-gate.md`). This brief
lands that measured shape in the live tree in one stage: the context/execution
control mechanism plus the calibrated surface (ok-form E2, the six phase-only
free starts, borrowed completions, E1 at `try_apply_function`, four sessionful
forwarded starts). The tree is left for the director's gate; no commit.

## Goal

When this brief is done, the live tree implements the brief-665 rebound-carrier
production shape, and on the exact source being proposed:

- `resume_eval_request_from_checkpoint` is emitted exactly as at HEAD after
  decoded address relocation: 4,606 live instructions / 25,861 B, 4,693 own
  Ir per request in both draws, zero real text differences, unchanged
  direct/indirect targets and own-PC counts; no new guard, field, local,
  `Drop`, call, branch or TLS access in that body;
- `evaluate_at_root` and the abandonment entry add exactly the approved check
  sequence and nothing else: 223 -> 231 (+8) on the root harness and
  1262 -> 1270 on the supplied abandonment harness;
- the six phase-only free starts and the direct private resume read the
  resumed session's **current** request control through cap retirement,
  external reset, rollover, thread handoff, nested second-session resume and
  unwind;
- no default-path instruction is added: `drive_input_request` stays 88 own
  Ir/call on all 121 resume labels in both draws with equal incoming slopes,
  the simple entry stays 2,223, and admission is 63 Ir/admission on both
  routes and both draws;
- **the fixed debug root returns to HEAD's 2,528 B** in `evaluate_at_root`
  for every measured shape. The design's 2,544 B is not accepted. The carrier
  semantics are pinned; the scope marker's attachment form may change to
  achieve this;
- every round-9 semantics holds: lp06 36/36, transitions 11/11, entries 23/23,
  combined 4/4, host 3/3, the literal 39 red falsifiers plus one green
  host-root control on HEAD and all 40 green here, with no assertion
  rewritten;
- the round-665 cap falsifier and the three new stale/foreign falsifiers are
  red on HEAD and green here with identical formatted bodies;
- the 29 own and incoming pairs are exact, F7 is at most 856, the native
  shapes equal HEAD, the whole D campaign is measured on the proposed source,
  the four repeated panel positives do not become scaling positives, both
  workspace configurations and the required-switch host suites run, every
  affected inventory is replayed, and `cargo fmt --check` is green on the
  exact measured source.

## Context pointers

- `.temp/director/brief-665-gate.md` — the ruling, the reproduced rows, the
  three cost dispositions. Read it first.
- `.temp/brief-665/report-design.json` — the eleven-arm comparison, every
  bar, every unpassed item.
- `.temp/brief-665/rebound-carrier-from-live.patch` — the exact 19-file
  production and test surface; `source-proof-rebound-carrier.json` — the
  surface, hashes, six phase sites and every TLS access.
- `.temp/brief-665/probe/rebound-carrier-src` — the measured production shape.
  `.temp/brief-665/probe/rebound-carrier-sem` — the same production plus the
  literal falsifiers and the replayed inventories. `.temp/brief-665/probe/red-src`
  is HEAD's arm.
- `.temp/brief-665/semantic-proof-rebound-carrier.json` and
  `cap-stale-foreign-proof.json` — the exact records and red/design contrasts.
- `.temp/brief-665/input-drive-121.json`, `hard-rows-red-rebound-carrier.json`,
  `resume-emission-rebound-carrier.json`, `admission-chain.json`,
  `physical-frame-differences.json`, `panel-two-draw-bands.json`,
  `panel-positive-attribution.json`, `rollover-cost/`, `d-instruments/` — the
  measured rows and the instruments.
- `.temp/brief-665/brief-666-lp06-source-stage-draft.md` — the worker's draft
  (raw material only; this brief is authoritative).
- Live HEAD sites: `profiling/logical/context.rs`, `logical/mod.rs`
  (`Call::start`, `open_phase`), `logical/segments.rs`, `resolution.rs`
  (`evaluate_at_root`), `suspension/core.rs`, `suspension/core/driver.rs`,
  `suspension/core/profile_mode.rs`, `suspension/core/profile_queue.rs`,
  `suspension/core/storage.rs`, `suspension/core/abandon.rs`,
  `session/session_api.rs`, `session/session_requests.rs`,
  `builtins_input.rs` (`drive_input_request`), `profiling/timing.rs`,
  `profiling/config.rs`, `eval_dispatch/mod.rs`,
  `kernel_data/control_data/branching_loops.rs`,
  `kernel_data/control_data/nonlocal.rs`, `scoping/callable.rs`,
  `scoping/callable/status_application.rs`.

## Scope

Files you MAY modify — exactly the brief-665 source surface (19 files):

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs`
- `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/context.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/control.rs` (additive tests)
- `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs` (additive tests)
- `crates/rum-runtime/src/runtime/profiling/timing.rs`
- `crates/rum-runtime/src/runtime/resolution.rs`
- `crates/rum-runtime/src/runtime/scoping/callable.rs`
- `crates/rum-runtime/src/runtime/scoping/callable/status_application.rs`
- `crates/rum-runtime/src/runtime/session/session_api.rs`
- `crates/rum-runtime/src/runtime/suspension/core.rs`
- `crates/rum-runtime/src/runtime/suspension/core/abandon.rs`
- `crates/rum-runtime/src/runtime/suspension/core/profile_mode.rs`
- `crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs`
- `crates/rum-runtime/src/runtime/suspension/core/profile_queue/tests.rs` (additive tests)
- `crates/rum-runtime/src/runtime/suspension/core/storage.rs`
- `crates/rum-host/tests/lp03_phase_rows.rs` (additive host falsifiers)

Plus the standing-inventory replays the template makes always in scope
(contracts, certificates, positions, ownership, LP01, typed-slot, census
definition). Replays are generated; never hand-edit a derived cell.

MUST NOT touch: the suspension driver (`suspension/core/driver.rs`), the
typed-resume macro, the checked adapter, `session_requests.rs`, `builtins_input.rs`,
the public API, features, serde, manifests, workloads, the R7 boundary, the
director's census assertion (1067 stays untouched), and anything outside the
list above. A second implementation stage may only be opened by the director.

## Target shape (pinned)

Use the complete bodies and field order in
`.temp/brief-665/rebound-carrier-from-live.patch`. The carrier representation
and call structure are pinned; a different representation is a STOP, not an
invitation to tune it in the source stage. The one amendment: the
`ExecutionScope` marker's attachment form may change **only** to restore the
fixed debug root (ruling 3), and it may not change the pushed value, the
push/pop conditions, or the two attachment sites.

```rust
pub(in crate::runtime) struct ContextGuard {
    context: Arc<Context>, // mandatory: preserve the existing outer Option niche
    segment: Option<segments::Segment>,
    evaluation: SourceStack<crate::runtime::Symbol>,
    rebuild: SourceStack<crate::runtime::Symbol>,
    lexical: SourceStack<&'static str>,
    sites: SourceStack<super::super::PresenceArgSite>,
}

pub(in crate::runtime) struct ProfileQueue {
    guest: GuestQueue,
    root: Option<CellId>,
    arena: Option<Arc<QueueArena>>,
    epoch: Option<Arc<Epoch>>,
    _storage: Option<Storage>,
}

// Context: optionalize epoch and _storage only; every other field and its
// order stay as supplied.
// epoch: Option<Arc<Epoch>>,
// _storage: Option<Storage>,

// Profiled::resume, still #[cold] #[inline(never)], still called by the
// existing driver line:
if let Some(suspension) = &mut session.resuming_suspension {
    if let FrameStorage::Profiled(profile) = &mut suspension.frames {
        profile.observe_boundaries(outcome);
        return profile.enter_front_with_control(Arc::clone(
            &session.request_interrupt_control.borrow(),
        ));
    }
}
```

Keep `Ordinary::Guard = ()` and its empty hooks; keep `drive_frames`'s
`let profile_context = M::resume(self, &outcome)` and `M::pause(&profile_context)`
untouched. `enter_front_with_control` enters the captured `Context` when
available and otherwise returns `ContextGuard::enter_control(control)`; that
fallback allocates an execution-only `Context` with no epoch or storage lease
and is an enabled cost, never claimed free. `enable_execution_context`
reattaches a fresh valid epoch only when the retained carrier lacks one;
`append_profiled` prefers a valid epoch from either side before its fallback.
Preserve the order of boundary observation, control entry, frame execution,
pause and guard drop.

Keep the measured `resume_profile_configured` code shape
(`configured && (enabled || cold configured fallback)`) and add a comment
stating why it exists: round 10's syntactic simplification moved
`drive_input_request` by one instruction on all 121 labels. Any change to
this shape must be re-measured against that row.

## Steps

1. **Calibrated surface.** Land ok-form E2 in `eval_dispatch/mod.rs`, the six
   phase-only starts (`while.test.evaluate`, `while.body.evaluate`,
   `compound.item.evaluate`, `catch.eval.total`, `catch.body.evaluate`,
   `function.apply.evaluate`), the borrowed completion and the value-form
   finish helper, and E1 only at `try_apply_function`. The typed macro and
   checked adapter stay untouched; the four forwarded starts stay sessionful.
2. **Carrier.** Land the TLS execution scope, the root and abandonment
   scopes, current-session lookup, the guard niche, optional epoch/storage in
   `Context` and `ProfileQueue`, `enter_front_with_control` and
   `ContextGuard::enter_control`, `retire_diagnostics`/`with_control`,
   `enable_execution_context`/`append_profiled`, and the cold
   `refresh_execution_control` in `accept_after_request_exhaustion`. Every TLS
   read and write is the one in `source-proof-rebound-carrier.json`; no other
   read or write is added.
3. **Fixed debug root.** Before any campaign, measure the debug frame of
   `evaluate_at_root` for every shape in
   `physical-frame-differences.json`. It must be 2,528 B, not the design's
   2,544 B. You may change only the scope marker's attachment form (for
   example, attach the enter/pop to the guard that already exists at the two
   sites) and no other part of the carrier. If no form reaches 2,528 B, STOP
   with the exact shape, size and counterexample.
4. **Falsifiers.** Add the unchanged literal bodies of
   `lp664_carrier_cap_pause_external_reset`,
   `lp664_carrier_external_reset_invalidates_context`,
   `lp664_carrier_inactive_pause_external_enable`,
   `lp664_carrier_rollover_handoff_reset_private_resume`,
   `lp664_carrier_unwind_restores_outer`, `lp664_rare_rollover_cost`, and
   `lp665_carrier_cap_external_reset_second_pause`,
   `lp665_carrier_cap_handoff_external_reset`,
   `lp665_carrier_cap_nested_rollover_external_reset`,
   `lp665_carrier_cap_unwind_restores_outer`, plus the two lp658 host
   falsifiers, with the same assertion bodies as
   `.temp/brief-665/probe/rebound-carrier-sem`. No assertion is relaxed.
5. **Replay.** After the last source edit, replay the standing inventories
   (contracts, certificates, positions, ownership, LP01, typed-slot, census
   definition) and copy the generated cells; disclose every new row. Keep the
   census assertion at 1,067; report the full kind/file/callee/reason
   comparison for the director's ruling.
6. **Measure.** Run the focused gates, then the full A/B campaign on the exact
   final source, then the two workspace configurations and the required-switch
   host suites. Timing runs last, serially and alone.

## Hard bars (deterministic)

- Checkpoint: 4,606 live instructions / 25,861 B; 4,693 own Ir both draws;
  zero normalized byte differences; unchanged direct and indirect targets;
  identical executed own-PC counts. Address masks cannot hide a frame offset,
  internal displacement or data change.
- Simple entry 2,223; root 223 -> 231 (+8); supplied abandonment harness
  1,262 -> 1,270 (+8 check only). Preserve the literal 223 -> 231 abandonment
  discrepancy for the director; do not relabel the workload.
- 29/29 own and incoming N/2N pairs exactly equal the supplied pre-A1
  reference, both draws. All six free-start owner slopes stay red; the While
  owner is still package 667's and must be reported, not claimed.
- `drive_input_request` 88 own Ir/call on all 121 resume labels, both draws,
  equal incoming slopes.
- Admission 63 Ir/admission on resume and abandonment, both draws; no larger
  than 77, and no owner may offset another.
- F7 <= 856 per specialization; CLI reservations 600/632 individually equal
  HEAD, no cross-frame offsets. Native shapes equal HEAD: While 776/3528/776,
  Compound 696/1304/680, Catch 1000/792/1000, forwarded 24/88/24
  (CLI/debug/release).
- **Fixed debug root 2,528 B**, red-equal in every shape; the design's
  2,544 B is a failing row.
- Cold rollover: measured separately, both draws, inclusive and per-helper own
  PCs; not larger than red 184/184 (disabled) versus the design's 213/213 and
  242/242 (enabled). This is the only permitted disabled-path increase and it
  is cold-only; it may not become a hot-path allowance.
- Semantics: lp06 36/36, transitions 11/11, entries 23/23, combined 4/4,
  host 3/3; 39 HEAD red falsifiers plus the one green host-root control on
  HEAD and 40/40 here; the cap and three stale/foreign falsifiers red on HEAD
  and green here, byte-identical bodies. No assertion rewritten.
- D campaign on the proposed source: 313-label instruction panel (two draws,
  per-label untouched band), 192 copy labels, 194 allocation labels, 44+18
  stack rows, 352 physical-frame captures, layouts, native prologues,
  lifecycle controls, canonical 120+9 timing labels. The four repeated
  positives may not become scaling positives; a moved decoded byte is a
  defect.
- Two source gates, ownership replay, census definition/grammar, both
  workspaces and `cargo fmt --check` green. The census count (1130) is
  reported, not accepted; the director rules it.

## Verification

Run one cargo command at a time inside the dispatcher's scope; jobs 8, test
threads 8, offline/locked. Bound anything that may spawn children with
`scripts/run-bounded.py`. Reproduce every unexpected red at
`--test-threads=1` before calling it a defect; an interference red is fixed,
not serialized away. Performance captures run serially and alone on CPU 2,
release, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, both arms at one common
source path, every `RUM_PROFILE_*` cleared. Reuse completed reference receipts;
never redraw an accepted reference. Run the shape/proof stop before the
campaign: a checkpoint byte, a driver row, a debug-root size or a semantics
red stops before the expensive panels.

Report `.temp/brief-666/report.json` in `rum-report/1`, with every run, every
attempt and every verdict line quoted; run `python3 -I scripts/report.py check
666` and return its summary and all compliance verdict lines.

## Checkpointing and concurrency

All scratch under `.temp/brief-666/`; keep `PROGRESS.md` with the exact next
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
