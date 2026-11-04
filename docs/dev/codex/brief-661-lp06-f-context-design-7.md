# Brief 661 — LP06 F design round 7: the corrected zero-sized execution scope

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Brief 660 stopped on two defects in the round-6 shape: request admission paid
9 extra own instructions per call, and a nested resume of a second session
inherited the outer session's owner. The director corrected both on the same
ok-form tree, and the corrected shape passes the whole falsifier matrix,
keeps every native reservation at R8, removes the admission cost (chain 77 ->
63 own Ir), and pays only the two cold-load checks per scope entry. This
brief makes that shape answerable to every measured bar and drafts the source
stage.

## Goal

Design, and prove with disposable builds under this brief's root, the
corrected zero-sized execution scope on the ok-form baseline (R8): the free
starts reproduce the sessionful owner-or-root rule from the thread's own
execution control, the TLS scope is entered at the root evaluation, resume
and abandonment entries, and the result adds **no measured disabled-path work
at any per-call function** — the 29 retained rows exactly equal to the pre-A1
base, F7 at most 856, the 313-label disabled panel with no positive row (or
every positive attributed to emitted PCs with its row's own code unchanged),
the admission chain no larger than the red arm, and every D native and stack
bar at its limit — while every 658-proven semantic record holds, red on the
ok-form red arm and green on the design arm, including the four combined
transition paths. Then draft the implementation brief in `TEMPLATE.md`
shape.

The corrected probe is the known answer to reproduce, not to reuse:
`.temp/director/probe-660/` (sources `probe/design-src`, patch
`zst-scope-r2.patch`, the built common-path copy with the disposable
falsifiers, the native prologues, the admission comparison). Read the patch
first; improve only with counter-evidence.

## Context pointers

- `.temp/director/brief-660-gate.md` — the round-6 STOP, the two confirmed
  defects, the corrected probe and every number below.
- `.temp/brief-660/report-design.json` — round-6 evidence that still stands:
  the 29 rows, F7, the native bars, the two source gates, the ownership
  replay, the census pairs, both workspace runs, and the 658 matrix it
  reused. Do not re-derive what it establishes; re-measure it on the
  corrected shape.
- `.temp/brief-660/combined-probes.rs` — the four combined falsifiers
  (a) deferred + nested different-session, (b) reset + suspend + handoff,
  (c) deferred + reset + suspend + handoff, (d) nested resume of a second
  session. (d) is red on the round-6 shape and green on the corrected one.
- `.temp/director/probe-660/` — `probe/design-src` (the corrected
  seven-file production), `zst-scope-r2.patch`, `logs/` (the corrected
  matrix run), `native/` (prologues), `entry-cost/`,
  `admission-compare.json`, `admission_compare.py`, `entry_cost.py`,
  `build_harness.py`, `native.py`, `b/entry-harness`.
- `.temp/brief-658/report-design.json` — the proven semantics
  (`q5_falsifier_matrix`, `q5_helper_bodies`, `q5_record_logs`), the entry
  inventory (`q1_entry_inventory`, `q1_registrations`,
  `q1_out_of_contract`), the context/match rule (`q2_context`,
  `q3_match_rule`). `.temp/brief-658/probe/tail-src/` — the semantics-proven
  guard design (record expectations only). `.temp/brief-658/d-instruments/`
  — the 313 panel runner and programs; copy before use.
- `.temp/brief-659/probe/` — the red/token trees and the instrument
  (`instrument/`, `red`/`token` binaries, the 29-label receipts, F7, native
  objdumps). `.temp/director/probe-658-okform/` — the R8 ok-form build and
  its receipts. `.temp/brief-655/supplied/` — the supplied 29-label
  instrument and build recipe. `.temp/brief-653/` — the D contract, native
  bars and the 313 program set (read-only).
- Live source: `profiling/logical/context.rs`, `segments.rs`,
  `logical/mod.rs` (`Call::start`, `start_from_active`, `open_phase`),
  `resolution.rs` (`evaluate_at_root`), `suspension/core.rs`
  (`resume_suspended_slot`), `suspension/core/abandon.rs`
  (`drain_abandonment_worklist`), `session/session_api.rs`
  (`accept_cancellable_request`), `profiling/config.rs` (the cold-load
  `AtomicU8` gate), and the 16 host entry groups in `session_eval.rs`,
  `session_requests.rs`, `session_api.rs`.

## Established by the director (reproduce before doubting)

1. **R8 — the baseline is the ok-form.** `witness.rs` at HEAD; the
   `eval_dispatch` site `ok(value) => finish(self, Ok(value))`; 29/29 own and
   incoming pairs equal to pre-A1; F7 632/600; ownership 48/48 retired=0; the
   census definition check passes (the count 1076 is the F3 frontier move).
   Do not reintroduce the hand expansion or the fourth macro arm.
2. **The free start is the sessionful rule.** `Call::start_from_active`
   reads `context::execution_control()` and then `segments::owner(&control)`
   — epoch valid, path open, `Arc::ptr_eq` — the same call `Call::start`
   makes with the session's control; with no owner it mints a root from the
   thread's own control; with no control it returns None.
   `segments::current_owner` must not exist. The two starts differ only in
   where the control comes from. The round-6 failure — a nested second
   session inheriting the outer owner, `owner=session:false` — is the
   counter-evidence for the old rule.
3. **Admission keeps its ordinary path.** The request-control rollover is a
   cold terminal path: a `#[cold]` `#[inline(never)]` helper replaces the
   control, refreshes every matching TLS scope entry, and admits the
   request; `refresh_execution_control` is cold and outlined. On the
   corrected build the admission chain (`request_cancel_handle` +
   `accept_cancellable_request` own Ir per admission) is **77 red, 63
   design**, in both draws on both routes: the design removes work and the
   ordinary body may inline. A fixed symbol is the wrong meter once inlining
   changes; the chain total is the meter.
4. **The disabled scope checks are inline.** `ExecutionScope::enter` and
   `Drop::drop` are `#[inline(always)]` over the cold-load check; the
   configured bodies are cold and outlined. Per-entry own Ir on the
   corrected build, both draws: `evaluate_at_root` +14,
   `drain_abandonment_worklist` +14,
   `resume_eval_request_from_checkpoint` +36 (one enter check and the drop
   check at each exit). No TLS and no call when unconfigured.
5. **Native bars hold at R8**: root 904, abandonment 856, request resume
   2184, evaluator 1208, while 744, compound 648, Catch 1000, forwarded slow
   bodies 24 x4, F7 600/632.
6. **The semantics hold on the corrected shape**: `lp06_` 36/36, `lp655_`
   1/2 (only the R7 out-of-contract bare dispatch fails), `lp657_` 11/11,
   `lp658_` 23/23, `lp660_` combined 4/4, host `lp03_phase_rows` 3/3 with
   `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`; `--test-threads=1`.
7. **The token carrier is dead** (brief 659): +26 own Ir/call at
   `evaluate_inner_unstable` in both draws, native bars fail, and the token
   does not span the first resumed free phase. Do not retest it.
8. R7: a bare `eval_control_data_builtin` outside an evaluation is out of
   contract; its A1 record is a documented boundary, not a bar.

## Questions to answer

- **Q1 — the scope and its coverage.** The exact type, ownership, TLS stack
  discipline, enter/drop placement and inline/cold choices, and why each of
  the 16 route groups reaches a scope: root evaluation, text/parsed/script/
  file/request entries, request resume, typed-slot resume,
  inspection/resume, cancellation, snapshot restore, evaluation cache,
  random state wrapper, host facade and request-control exhaustion. Name
  every new call edge and its execution path. No guard state in a frame; no
  TLS when unconfigured; no borrowed `Session`, raw pointer or `unsafe`;
  `segments::current_owner` absent from the design.
- **Q2 — the disabled path.** Measure, on the corrected shape, both draws:
  the admission chain (`request_cancel_handle` + `accept_cancellable_request`
  own Ir per admission) on the two host routes; the three entry per-entry
  costs; the 29 own and 29 incoming slopes exactly equal to the pre-A1 base;
  F7 at most 856; the 313-label panel with every positive attributed to
  emitted PCs and its row's own code shown unchanged (the brief-653 contract,
  `positive_disposition` unchanged); and no measured entry prologue grows
  (root 904, abandonment 856, request resume 2184, compound 696, Catch 1000,
  four forwarded slow bodies 24, hot evaluator 1208, release wrappers
  absent). A positive per-call delta on an iteration-proportional function is
  a defect unless the PC evidence shows that row's own code unchanged; a
  positive admission-chain delta is a defect, full stop.
- **Q3 — the semantic matrix, including the combined paths.** Re-run the 658
  matrix with its exact records, red on the ok-form red arm and green on the
  design arm. Re-run the four combined falsifiers with exact records,
  including (d): a nested resume of a second session inside an evaluation
  must record under the inner session's control and must restore the outer
  evaluation's control when it returns. These are host routes: a registered
  builtin may own and drive a second session.
- **Q4 — the scope's limits.** Every point where the TLS top is read, every
  point where it is written, and why no read can see a different session's
  control in contract. The request-generation rollover: the cold terminal
  path, its cost, and the same-session nested propagation. The
  `Unwind`/panic path: what the Drop does, what remains after an unwind that
  crosses a scope, and whether any in-contract free start can observe it. Do
  not assert the R7 boundary.
- **Q5 — shape and bars.** The calibrated surface unchanged (ok-form E2, six
  phase-only starts, borrowed completions, E1 only at `try_apply_function`,
  four sessionful forwarded starts); the two unchanged source gates green
  with the entry bodies in place
  (`suspension_protocol_repository_invariants_hold`,
  `finite_profile_modes_are_closed_and_selected_at_both_calls`); the
  ownership replay green with no HEAD row retired and the design tree's new
  rows paired; the census definition check passing, the design-tree count
  recorded for the source stage, and the 54 added identity rows explained
  (why unmodified `budget.rs`/`epoch.rs` sites surface in the design scan);
  every D native/stack/copy/allocation/layout/lifecycle/timing bar measured
  or explicitly unpassed; the workspace results with the stale-cell ordering
  artifacts classified. The completion/forwarded literal gap and R7 stay in
  the unpassed list.
- **Q6 — the bar.** If the corrected scope cannot hold a bar without new
  default-path work, a wider surface or a lowered bar, STOP with the minimal
  mechanism, the exact cost and the counter-evidence — the ruling is the
  director's. Do not invent a waiver; do not cover the R7 boundary with a
  dispatcher guard; do not reintroduce a guard local, a token field, a token
  `Drop`, or a macro/checked-adapter change.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-661/`: copy the director's corrected probe and the 658 tail and
instruments; build the red arm (ok-form) and the design arm at one common
source path with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked); measure
with the supplied runner serially, alone, on CPU 2, every `RUM_PROFILE_*`
cleared. Supplied and prior roots are read-only and never executed in place
(`.temp/brief-648/`, `.temp/brief-653/`, `.temp/brief-654/`,
`.temp/brief-655/`, `.temp/brief-656/`, `.temp/brief-657/`,
`.temp/brief-658/`, `.temp/brief-659/`, `.temp/brief-660/`,
`.temp/director/calib-655/`, `.temp/director/verify-655/`,
`.temp/director/probe-657/`, `.temp/director/probe-658-okform/`,
`.temp/director/probe-659-gen/`, `.temp/director/probe-660/`).

**Do not** change the typed-resume macro or checked adapter, hand-edit an
inventory or census count, add a per-operation dispatcher guard, add a guard
local or token field, make the completions or forwarded starts session-free,
or change the calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-661/report-design.json` with:

- the R8 baseline reproduced (build receipt and hash, 29 own/incoming pairs,
  F7, ownership verdicts, the five red-arm failures);
- the corrected zero-sized scope design: type, TLS stack, every enter/drop
  edge, the free-start rule, the cold rollover path, and every new call edge;
- the entry coverage with `path:line` and the R7 out-of-contract list;
- the design arm's receipts: the 29 rows, the 313 panel (every positive
  attributed), F7, every D native and stack bar, the admission chain, the
  three entry costs, the two source-proof gates, the ownership replay and the
  census verdict with the added-row explanation;
- the falsifier matrix with exact records, red on the ok-form red arm and
  green on the design arm, including the four combined-path falsifiers;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list keeping R7, the completion/forwarded literal
  gap, and every bar not measured (including the four While rows if the
  package is not yet created).

Run `python3 -I scripts/report.py check 661` and return its summary and every
compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, live source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- One variable per probe; a single Callgrind pair never decides a
  seed-dependent quantity; state the distribution when a quantity needs one.
  Inlining is allowed; the admission meter is the chain total, not a fixed
  symbol.
- A design that pays for the context with new default-path work, a wider
  public API, a compile-time gate, a macro-grammar change or a lowered bar is
  not a candidate.
- Every claim carries a citation the checker can resolve.
