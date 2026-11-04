# Brief 659 — LP06 F design round 5: the zero-cost context installation

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Brief 658 proved the session-execution context's semantics for the first time
and then stopped correctly: the entry-guard prototypes fail the disabled-path
and source-proof bars. The guard local in an entry frame grows root and
abandonment reservations by 16 bytes (off-by-default state in a hot frame);
outlining the guard removes the frame but adds a hot-path call (+44 Ir/resume)
and moves the abandonment body out of the function the unchanged source
proofs read. This brief finds the installation that pays nothing on the
disabled path while keeping the proven semantics.

## Goal

Design, and prove with disposable builds under this brief's root, a
session-execution context installation that adds **no disabled-path work at
any measured per-call function** — every one of the 313 disabled-panel labels
stays at its base instruction slope (or every positive is shown, with its
row's own code unchanged, to be an attribution/layout artifact), the 29
retained rows stay exactly equal to the pre-A1 base, F7 stays at most 856,
and every D native and stack bar holds — while keeping the 658-proven
semantics on the ok-form baseline: the five LP06 failures and the
deferred-enable/reset path, reentrancy, two sessions on one thread, thread
handoff, reset, panic/unwind and off-to-on/on-to-off gate transitions all
green, red on the ok-form red arm, with the same exact records. Then draft
the implementation brief in `TEMPLATE.md` shape.

The lever is **fusion**: install the context from a cold-load check that
already executes on the disabled path (the `runtime_path_timing_start_
adopted*` fast check, the `evaluate.total` slow path, the native stack
entry, a `record_profile_*` recorder, or an equivalent), so the three entry
bodies stay byte-identical and the panel sees no new instruction. Whether
every host-reachable route can be covered that way is the design question —
not an assumption.

No per-operation dispatcher guard, no wider public API, compile-time gate,
`unsafe`, new module boundary, borrowed `Session` or raw pointer in TLS,
cached generation/root/epoch, shared mutable queue, or new guard state in a
hot frame. No TLS access when the profile is not configured. Do not change
the typed-resume macro definition, its invocation grammar, the checked
adapter, or any inventory/census cell; the E2 form is fixed (R8). If a route
cannot be covered without new default-path work, STOP with the minimal
mechanism and its measured per-entry cost for a director ruling (Q6) — do
not waive a bar.

## Context pointers

- `.temp/director/brief-658-gate.md` — the 658 finding and this direction.
- `.temp/brief-658/report-design.json` — the proven semantics
  (`q5_falsifier_matrix`, `q5_helper_bodies`, `q5_record_logs`), the entry
  inventory (`q1_entry_inventory`, `q1_registrations`,
  `q1_out_of_contract`), the context/match rule (`q2_context`,
  `q3_match_rule`), and the failed cost shapes (`q4_disabled_path`, the
  panel in `disabled-distributions.json`, `logs/TAIL-*`). Read it first; do
  not re-derive the matrix.
- `.temp/brief-658/probe/tail-src/` — the semantics-proven design tree
  (red baseline + context). `.temp/brief-658/d-instruments/` — the 313
  panel runner, reader, program set and the two-arm binaries; copy before
  use. `.temp/brief-658/logs/` — the panel, attribution, frame and
  source-gate logs.
- `.temp/director/probe-658-okform/` — the director's ok-form build (binary
  `e6fc002a…`), the supplied-package copy, the 29-row receipts, `f7.log`,
  `census-test.log`, `discovery.log`. Known answer to reproduce, not to
  reuse.
- `.temp/brief-655/supplied/` — the supplied 29-label instrument and build
  recipe. `.temp/brief-653/` — the D contract, native bars and the 313
  program set (read-only).
- Live source: `profiling/timing.rs` (`runtime_path_timing_start`,
  `runtime_path_timing_start_adopted*` and their cold slow paths),
  `profiling/logical/mod.rs` (`Call::start`, `start_owned`,
  `start_from_active`, `open_phase_from_active`, `close_phase`),
  `profiling/logical/segments.rs` (`owner`, `current_owner`, `enter`),
  `profiling/logical/context.rs`, `profiling/config.rs` (the cold-load
  `AtomicU8` gate), `resolution.rs` (`evaluate_at_root`,
  `evaluate_at_root_body`, the `evaluate.total` start),
  `suspension/core.rs` (`resume_suspended_slot`, the forwarded starts in
  the resume continuations), `suspension/core/abandon.rs`
  (`drain_abandonment_worklist`), the native stack entry, and
  `session/session_eval.rs`, `session/session_requests.rs`,
  `session/session_api.rs` (the 16 route groups' host entries).

## Established by the director (reproduce before doubting)

1. **R8 — the baseline is the ok-form.** `witness.rs` at HEAD; the
   `eval_dispatch` site `ok(value) => finish(self, Ok(value))`; binary
   `e6fc002a…`; 29/29 own and incoming pairs equal to pre-A1; F7 632/600;
   ownership 48/48 with retired=0; the census definition check passes (the
   exact count 1076 vs 1067 is the F3 frontier move the source stage
   pairs). Do not reintroduce the hand expansion or the fourth macro arm.
2. **The 658 semantics are proven** on the entry-guard tail: 36 LP06 +
   deferred-enable, 11 transition, 23 added runtime and 2 host falsifiers
   green; red on the ok-form red arm; the host-root control green on both.
   The five LP06 failures reproduce serially on the red arm.
3. **The 658 cost findings**: the in-place guard grows root 904→920 and
   abandonment 856→872; the outlined abandonment body breaks
   `suspension_protocol_repository_invariants_hold` and
   `finite_profile_modes_are_closed_and_selected_at_both_calls`; the
   outlined body call costs net +44 Ir/resume; 23 panel labels positive in
   both draws. These are the shapes to avoid, not the bar.
4. The adopted starts already execute a short-circuited cold-load check on
   the disabled path (`lexical_profile_enabled()`: one load when
   unconfigured), and their slow paths are `#[cold] #[inline(never)]` with
   the session in hand. `evaluate_at_root_body` starts the
   `evaluate.total` adopted start; the resume continuations start
   forwarded/adopted starts; the root and abandonment bodies already hold
   a native stack entry.
5. R7: a bare `eval_control_data_builtin` outside an evaluation is out of
   contract; the A1 record there is a documented boundary, not a bar.

## Questions to answer

- **Q1 — the zero-cost installation.** The exact placement and mechanism:
  which existing cold-load check the installation rides, how the guard's
  state avoids every hot frame, and why the disabled path executes the
  same instructions as the calibrated R8 arm (measure it, do not argue).
  Name every new call edge and where it executes.
- **Q2 — the semantics.** The context type and ownership (the 658-proven
  semantics unless the placement forces a change), the install/restore
  across suspension, resume, reentrancy, two sessions on one thread,
  handoff, panic/unwind, reset and off-to-on/on-to-off, including the
  deferred-enable case (installation must not require `active`). No
  borrowed `Session`, raw pointer, `unsafe`, or cached generation/root/
  epoch in TLS.
- **Q3 — coverage.** The 16 route groups: every host-reachable route that
  can reach one of the six session-free starts is covered by the new
  placement, with `path:line` and the matrix state that proves it. Name the
  out-of-contract routes under R7 and do not assert them.
- **Q4 — the disabled path.** The measured added work at every entry and in
  every one of the 313 panel labels and 29 rows; no measured entry
  prologue grows; the root/abandon/resume bodies are byte-identical to the
  R8 arm. A positive per-call delta on an iteration-proportional function
  is a defect unless the PC evidence shows that row's own code unchanged.
- **Q5 — shape and bars.** The calibrated surface unchanged (ok-form E2,
  six phase-only starts, borrowed completions, E1 adapter only at
  `try_apply_function`, four sessionful forwarded starts); the two
  unchanged source-proof gates green with the entry bodies in place; the
  ownership replay green with no HEAD row retired; the census definition
  check passing and the design-tree count recorded for the source stage;
  the D native and stack bars measured; the workspace green except the
  known stale-cell ordering artifacts. The completion/forwarded literal
  gap and R7 stay in the unpassed list.
- **Q6 — R5 / the bar.** If any route needs new default-path work that the
  panel can see, STOP with the minimal mechanism, the exact per-entry cost,
  and the counter-evidence — the ruling is the director's, and no bar is
  lowered. Do not invent a waiver; do not cover the R7 boundary with a
  dispatcher guard.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-659/`: copy the 658 tail and instruments and the director's
ok-form tree; build the red arm (ok-form) and the design arm at one common
source path with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked); measure
with the supplied runner serially, alone, on CPU 2, every `RUM_PROFILE_*`
cleared. Supplied and prior roots are read-only and never executed in place
(`.temp/brief-648/`, `.temp/brief-653/`, `.temp/brief-654/`,
`.temp/brief-655/`, `.temp/brief-656/`, `.temp/brief-657/`,
`.temp/brief-658/`, `.temp/director/calib-655/`,
`.temp/director/verify-655/`, `.temp/director/probe-657/`,
`.temp/director/probe-658-okform/`).

**Do not** change the typed-resume macro or checked adapter, hand-edit an
inventory or census count, add a per-operation dispatcher guard, design a
shared queue cursor, make the completions or forwarded starts session-free,
or change the calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-659/report-design.json` with:

- the R8 baseline reproduced (build receipt and hash, 29 own/incoming
  pairs, F7, ownership verdicts, the five red-arm failures);
- the installation design with every new call edge, the guard/state
  placement and the exact fusion check;
- the entry coverage with `path:line` and the R7 out-of-contract list;
- the design arm's receipts: the 29 rows, the 313 panel (every positive
  attributed), F7, every D native and stack bar, the two source-proof
  gates, the ownership replay and the census definition verdict;
- the falsifier matrix with exact records, red on the ok-form red arm and
  green on the design arm;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list keeping R7, the completion/forwarded literal
  gap, and every bar not measured (including the four While rows if the
  package is not yet created).

Run `python3 -I scripts/report.py check 659` and return its summary and every
compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, live source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- One variable per probe; a single Callgrind pair never decides a
  seed-dependent quantity; state the distribution when a quantity needs
  one.
- An installation that pays for the context with new default-path work, a
  wider public API, a compile-time gate, a macro-grammar change or a
  lowered bar is not a candidate.
- Every claim carries a citation the checker can resolve.
