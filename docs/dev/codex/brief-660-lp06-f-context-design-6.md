# Brief 660 — LP06 F design round 6: the zero-sized execution scope

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Brief 659 rejected the timing-token lifetime carrier with a decisive
disabled-path counterexample (+26 own Ir/call at `evaluate_inner_unstable`,
both draws, every executed PC reconciled) and with native and semantic
failures. The director then probed a different carrier on the same ok-form
tree and it holds: the context is the execution scope itself, marked by a
zero-sized RAII value whose owned state lives in a TLS stack. The probe
passes the whole 658 falsifier matrix and keeps the two entry prologues at
their R8 bytes (904 / 856). This brief makes that shape answerable to every
measured bar and drafts the source stage.

## Goal

Design, and prove with disposable builds under this brief's root, the
zero-sized execution scope on the ok-form baseline (R8): the free starts
reproduce the sessionful owner-or-root rule, the TLS scope is entered at the
root evaluation, resume and abandonment entries, and the result adds **no
measured disabled-path work at any per-call function** — the 29 retained rows
exactly equal to the pre-A1 base, F7 at most 856, the 313-label disabled panel
with no positive row (or every positive attributed to emitted PCs with its
row's own code unchanged), and every D native and stack bar at its limit —
while every 658-proven semantic record holds, red on the ok-form red arm and
green on the design arm, including the combined transition paths named in
Q3. Then draft the implementation brief in `TEMPLATE.md` shape.

The director's probe is the known answer to reproduce, not to reuse:
`.temp/director/probe-659-gen/` (7 files, ~57 lines; `zst-scope.patch`).
Its shape: a zero-sized `ExecutionScope` (`enter` pushes the session's owned
control onto a TLS `Vec` when configured; `Drop` pops when configured); the
free start inherits `segments::current_owner` (without the generation
predicate) else mints a root from the TLS top; the request-control rollover
refreshes every matching TLS entry. The only disabled-path additions are the
two `lexical_profile_configured()` checks per entry (enter and drop); no TLS
when unconfigured; no guard local; no token field or `Drop`; no token,
macro, adapter, census or calibrated-surface change.

## Context pointers

- `.temp/director/brief-659-gate.md` — the token rejection, the reproduced
  +26 Ir/call, and this direction.
- `.temp/brief-659/report-design.json` — the token arm's full evidence
  (`q1_installation`, `q4_disabled_path`, `q5_falsifier_matrix`, the native
  and panel logs) and the 658 matrix it reused.
- `.temp/director/probe-659-gen/` — the director's probe tree
  (`b/active` sources, `zst-scope.patch`, the test logs `lp06-v4`,
  `lp655-v4`, `lp657-v4`, `lp658-v4`, `host-v4`, the release build, my
  objdump). Read the patch first; reproduce, then improve only with
  counter-evidence.
- `.temp/brief-658/report-design.json` — the proven semantics
  (`q5_falsifier_matrix`, `q5_helper_bodies`, `q5_record_logs`), the entry
  inventory (`q1_entry_inventory`, `q1_registrations`,
  `q1_out_of_contract`), the context/match rule (`q2_context`,
  `q3_match_rule`). Do not re-derive the matrix.
- `.temp/brief-658/probe/tail-src/` — the semantics-proven guard design (for
  the record expectations only). `.temp/brief-658/d-instruments/` — the 313
  panel runner and programs; copy before use. `.temp/brief-659/probe/` — the
  red/token trees, the instrument (`instrument/`), `red`/`token` binaries,
  the 29-label receipts, F7, native objdumps.
- `.temp/director/probe-658-okform/` — the R8 ok-form build (binary
  `e6fc002a…`), the 29-row receipts, `f7.log`, `census-test.log`,
  `discovery.log`.
- `.temp/brief-655/supplied/` — the supplied 29-label instrument and build
  recipe. `.temp/brief-653/` — the D contract, native bars and the 313
  program set (read-only).
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
   census definition check passes (the count 1076 vs 1067 is the F3 frontier
   move). Do not reintroduce the hand expansion or the fourth macro arm.
2. **The token carrier is dead.** Its `Drop` on `RuntimePathTimingToken`
   changes the evaluator's emitted disabled path: +26 own Ir/call at
   `evaluate_inner_unstable` in both draws, +35.5 Ir per root entry, native
   bars fail, and the token does not span the first resumed free phase. Do
   not retest it.
3. **The zero-sized scope passes the semantics.** On the probe, with the
   product tests at `--test-threads=1`: `lp06_` 36/36,
   `lp655_deferred_enable_context_falsifier` pass, `lp657_` 11/11,
   `lp658_` 23/23, host `lp03_phase_rows` 3/3 (with
   `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`). The only failing test is
   `lp655_root_context_falsifier`, the R7 out-of-contract bare dispatch;
   R7 stays excluded, not weakened.
4. **The entry frames are intact.** The probe's release objdump gives
   `evaluate_at_root` prologue 904 and `drain_abandonment_worklist` 856 —
   exactly R8 (the guard's 920/872 do not recur).
5. **The free-start rule is the sessionful rule.** `Call::start` matches
   `segments::owner(&control)` without a generation predicate — an open
   owner is inherited across a request-generation advance. `current_owner`
   must drop its generation predicate; the 658 match rule's "even if the
   live control generation has advanced" is the contract.
6. R7: a bare `eval_control_data_builtin` outside an evaluation is out of
   contract; its A1 record is a documented boundary, not a bar.

## Questions to answer

- **Q1 — the scope and its coverage.** The exact type, ownership, TLS stack
  discipline, enter/drop placement, and why each of the 16 route groups
  reaches a scope: root evaluation, text/parsed/script/file/request entries,
  request resume, typed-slot resume, inspection/resume, cancellation,
  snapshot restore, evaluation cache, random state wrapper, host facade and
  request-control exhaustion. Name every new call edge and its execution
  path; state the `#[inline]`/`#[cold]` choice for enter and drop with its
  measured disabled-path effect. No guard state in a frame; no TLS when
  unconfigured; no borrowed `Session`, raw pointer or `unsafe`.
- **Q2 — the disabled path.** Measure the two `configured()` checks per
  entry: the root-entry own cost (the `withcleanup-small` root-cost pair, two
  draws), the resume entry, and the abandonment entry. Then measure that no
  per-call function moves: the 29 own and 29 incoming slopes exactly equal to
  the pre-A1 base; F7 at most 856; the 313-label panel with every positive
  attributed to emitted PCs and its row's own code shown unchanged (the
  brief-653 contract, `positive_disposition` unchanged); no measured entry
  prologue grows (root 904, abandonment 856, request resume 2184, compound
  696, Catch 1000, four forwarded slow bodies 24, hot evaluator 1208,
  release wrappers absent). A positive per-call delta on an
  iteration-proportional function is a defect unless the PC evidence shows
  that row's own code unchanged.
- **Q3 — the semantic matrix, including the combined paths.** Re-run the 658
  matrix with its exact records, red on the ok-form red arm and green on the
  design arm. Add disposable falsifiers for the combinations the matrix has
  not covered, each with exact records: (a) deferred entry, reset, nested
  different-session evaluation, then a free start — the free start must
  record under the outer session's control; (b) reset, then suspend, then
  resume on another thread, then a free start; (c) deferred entry, reset,
  suspend, then resume on another thread with no open segment; (d) a nested
  resume of a second session inside an evaluation. The scope's Drop must
  restore the enclosing evaluation's control in all four. These are host
  routes: a registered builtin may own and drive a second session.
- **Q4 — the scope's limits.** Every point where the TLS top is read, every
  point where it is written, and why no read can see a different session's
  control in contract. The request-generation rollover refresh: its branch,
  cost, and the same-session nested propagation. The `Unwind`/panic path:
  what the Drop does, what remains after an unwind that crosses a scope, and
  whether any in-contract free start can observe it. Do not assert the R7
  boundary.
- **Q5 — shape and bars.** The calibrated surface unchanged (ok-form E2, six
  phase-only starts, borrowed completions, E1 only at `try_apply_function`,
  four sessionful forwarded starts); the two unchanged source gates green
  with the entry bodies in place (`suspension_protocol_repository_invariants_
  hold`, `finite_profile_modes_are_closed_and_selected_at_both_calls`); the
  ownership replay green with no HEAD row retired and the design tree's new
  rows paired; the census definition check passing and the design-tree count
  recorded for the source stage; the workspace results with the stale-cell
  ordering artifacts classified; every D native/stack/copy/allocation/layout/
  lifecycle/timing bar measured or explicitly unpassed. The completion/
  forwarded literal gap and R7 stay in the unpassed list.
- **Q6 — R5 / the bar.** If the zero-sized scope cannot hold a bar without
  new default-path work, a wider surface, or a lowered bar, STOP with the
  minimal mechanism, the exact per-entry cost and the counter-evidence — the
  ruling is the director's. Do not invent a waiver; do not cover the R7
  boundary with a dispatcher guard; do not reintroduce a guard local, a token
  field, a token `Drop`, or a macro/checked-adapter change.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-660/`: copy the director's probe tree and the 658 tail and
instruments; build the red arm (ok-form) and the design arm at one common
source path with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked); measure
with the supplied runner serially, alone, on CPU 2, every `RUM_PROFILE_*`
cleared. Supplied and prior roots are read-only and never executed in place
(`.temp/brief-648/`, `.temp/brief-653/`, `.temp/brief-654/`,
`.temp/brief-655/`, `.temp/brief-656/`, `.temp/brief-657/`,
`.temp/brief-658/`, `.temp/brief-659/`, `.temp/director/calib-655/`,
`.temp/director/verify-655/`, `.temp/director/probe-657/`,
`.temp/director/probe-658-okform/`, `.temp/director/probe-659-gen/`).

**Do not** change the typed-resume macro or checked adapter, hand-edit an
inventory or census count, add a per-operation dispatcher guard, add a guard
local or token field, make the completions or forwarded starts session-free,
or change the calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-660/report-design.json` with:

- the R8 baseline reproduced (build receipt and hash, 29 own/incoming pairs,
  F7, ownership verdicts, the five red-arm failures);
- the zero-sized scope design: type, TLS stack, every enter/drop edge, the
  free-start rule, the rollover refresh, and every new call edge;
- the entry coverage with `path:line` and the R7 out-of-contract list;
- the design arm's receipts: the 29 rows, the 313 panel (every positive
  attributed), F7, every D native and stack bar, the two source-proof gates,
  the ownership replay and the census definition verdict;
- the falsifier matrix with exact records, red on the ok-form red arm and
  green on the design arm, including the four combined-path falsifiers;
- the per-entry disabled-path costs (root, resume, abandonment) with their
  two-draw measurements;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list keeping R7, the completion/forwarded literal
  gap, and every bar not measured (including the four While rows if the
  package is not yet created).

Run `python3 -I scripts/report.py check 660` and return its summary and every
compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, live source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- One variable per probe; a single Callgrind pair never decides a
  seed-dependent quantity; state the distribution when a quantity needs one.
- A design that pays for the context with new default-path work, a wider
  public API, a compile-time gate, a macro-grammar change or a lowered bar is
  not a candidate.
- Every claim carries a citation the checker can resolve.
