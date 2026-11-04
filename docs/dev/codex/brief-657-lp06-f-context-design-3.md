# Brief 657 — LP06 F design round 3: the session-execution context on the repaired baseline

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Brief 656 stopped on a real standing-inventory obligation: the prescribed
F3+E2 hand expansion retires the `resume_typed_or_suspend!` identity at
`eval_dispatch/mod.rs:233`, and the unchanged ownership reconciler rejected it
before any semantic acceptance. The director resolved the baseline by
expressing the E2 delta as a new arm of that inventoried macro; that form
reproduces the whole calibration and passes the ownership replay (R6). This
brief designs the still-missing piece — the session-execution context — on
that repaired baseline.

## Goal

Design, and prove with disposable builds under this brief's root, the
session-execution context that makes the six session-free starts **equal to
the sessionful form in every host-reachable evaluation state** — root,
deferred enable and reset mid-evaluation, resume, reentrancy, several sessions
on one thread, thread handoff, panic/unwind, and off-to-on/on-to-off gate
transitions — while every one of the 29 retained rows stays at or below the
pre-A1 base's own slope, the incoming-call slopes stay equal, F7 stays at most
856, the 313-label disabled panel has no positive row, every D native and
stack bar holds, and every existing enabled test is green. The calibrated
surface is the repaired F3+E2 surface (the macro-arm E2 form); the design adds
the context and nothing else. Then draft the implementation brief in
`TEMPLATE.md` shape.

The context may not be paid for with a new per-operation branch on the
disabled path. No guard at a per-operation dispatcher — including
`eval_control_data_builtin` and the individual `eval_*` helpers — and no
wider public API, compile-time gate, `unsafe`, new module boundary, borrowed
`Session` or raw pointer in TLS, cached generation/root/epoch, or shared
mutable queue. At most one cold-load check per evaluation, resume or
abandonment entry, and no TLS access when the profile is not configured.

## Context pointers

- `.temp/director/brief-656-gate.md` — R6 (the macro-arm baseline) and R7
  (the coverage boundary), with the director's probe receipts. Read first.
- `.temp/brief-656/report-design.json` — `q1_diagnosis`, `q2_context`,
  `q3_match_rule`, `q4_disabled_path`, `q5_falsifier_matrix`, `q6_shape`,
  `q7_r5`, `unpassed`, `sections.design`, `sections.implementation-draft`.
- `.temp/director/probe-657/` — the director's macro-arm build (binary
  `43f855ef…`), the supplied-instrument copy, `all-measurements.json`,
  `f7.log`, `ownership-replay.log`, the red-arm test logs. Read-only.
- `.temp/brief-655/probe/f3e2-src/` — the eight-file F3+E2 tree (read-only
  reference; copy before use). `.temp/brief-656/probe/f3e2-src/` carries the
  same surface plus the three disposable falsifiers in
  `profiling/tests/control.rs`, including
  `lp655_deferred_enable_context_falsifier` (host-reachable) and
  `lp656_public_root_context_falsifier` (out of contract under R7).
- `.temp/brief-655/supplied/` — the supplied instrument (29 labels, 58 frozen
  programs, known-answer base binary, serial Callgrind runner, reducer, F7
  prober, build recipe). Copy it into your own probe root before use.
- Live source: `crates/rum-runtime/src/runtime/resolution.rs` (`evaluate`,
  `evaluate_at_root`), `.../suspension/core.rs`
  (`resume_suspended_slot`), `.../suspension/core/abandon.rs`
  (`drain_abandonment_worklist`), `.../profiling/logical/mod.rs`
  (`Call::start`, `start_owned`, `start_from_active`,
  `open_phase_from_active`), `.../profiling/logical/segments.rs` (`owner`,
  `current_owner`, `enter`), `.../profiling/logical/context.rs`,
  `.../profiling/config.rs` (the cold-load `AtomicU8` gate),
  `.../kernel_data/control_data/mod.rs:64` (the out-of-contract direct
  dispatcher).
- `.temp/brief-653/native-bars/comparison.json` and
  `.temp/brief-653/instruments/measurement-contract.json` — the D shape and
  the 313-label panel contract. `.temp/brief-654/` — the While package design
  whose four rows are still unmeasured.

## Established by the director (reproduce before doubting)

1. **R6.** The E2 delta expressed as a new `resume_typed_or_suspend!` arm
   (`produced(value) => finish(self, value)`, `suspend => freeze()`) builds
   to binary `43f855ef…`, gives F7 reservations 616/600 against base
   856/856, reproduces all 29 base own and incoming pairs exactly, and passes
   the ownership replay with 73 position moves, 3 new `UnknownConstruct` at
   touched files and **0 retired identities**. The macro row at
   `eval_dispatch/mod.rs:233` survives (48 `resume_typed_or_suspend!` rows in
   the HEAD inventory, 48 discovered). The hand expansion is not landable.
2. **R7.** A bare `eval_control_data_builtin` call outside an evaluation is
   out of the profiling contract. Its only production caller is
   `crates/rum-stdlib/src/control_flow.rs:13`, inside the evaluator; the
   logical profile's roots are evaluation entries. Covering it needs a
   per-operation check on the hot control dispatcher, which is not a
   candidate. The required root transition is the host-reachable
   reset/deferred-enable state (`lp655_deferred_enable_context_falsifier`),
   not a direct helper dispatch.
3. The repaired red arm reproduces the five failures at `--test-threads=1`;
   the `queued=true` LP06 iterations pass, so a plain host evaluation is
   green on the red arm.
4. `segments::owner(&control)` matches by `Arc::ptr_eq` and requires a valid
   epoch and an open path; a root is `epoch::current()?` with the context
   control, its `logical_generation()`, and `epoch::next_id()`.
5. The six session-free starts are `runtime_path_timing_start` at
   `branching_loops.rs:156/183/365`, `nonlocal.rs:81/93` and
   `scoping/callable.rs:63`. The E1 by-value adapter stays only at
   `try_apply_function`.

## Questions to answer

- **Q1 — the entry inventory.** Every host-reachable entry that can reach one
  of the six starts without an enclosing guard: `evaluate` at depth 0 and
  every `eval_*`/request/script route that reaches it,
  `resume_suspended_slot`, `drain_abandonment_worklist`, and any other
  (snapshot restore, evaluation cache, typed-slot resume, request handling).
  For each: `path:line`, the guard placement, and why it is reached only
  when the profile is configured. Name the out-of-contract entries under R7
  with their production callers.
- **Q2 — the context.** The exact type and ownership: what the TLS holds (the
  owned control Arc only, or more), the native guard's save and restore, its
  Drop on every exit including unwind, and how it survives suspension,
  resume, reentrancy and thread handoff. State the reset behavior and the
  off-to-on/on-to-off gate transitions. No borrowed `Session`, raw pointer,
  `unsafe`, cached generation, cached root or epoch pointer.
- **Q3 — the match rule.** The exact predicate reproducing `Call::start`:
  `segments::owner(&context_control)` yields a child with the inherited
  epoch, owner and parent; otherwise `epoch::current()?` plus
  `Owner { control, logical_generation(), root: epoch::next_id() }` with
  parent `None`. State what makes a root a root, whether repeated roots are
  bounded, and what happens when the context is absent. Do not add the
  generation-equality predicate to `current_owner`.
- **Q4 — the disabled path.** The exact added work at each entry: the
  configured cold-load check, the Arc clone only when configured, no TLS
  access when not configured. Measure that no row of the 313-label panel and
  none of the 29 own slopes moves, and that no measured entry prologue grows.
- **Q5 — the transitions.** A disposable falsifier per state, each with its
  exact body, red on the repaired F3+E2 arm and green on the design arm: the
  three LP06 resumed tests; the deferred-enable/reset falsifier (host path);
  reentrancy; two sessions on one thread; thread handoff; reset; off-to-on
  and on-to-off gate transitions; panic/unwind; and a host-root control that
  is green on both arms. State the expected records for each (keys, calls,
  owner control and generation, root ids, open=0, faults=0), not only
  `calls > 0`. Do not assert the out-of-contract direct dispatch.
- **Q6 — the shape and baseline.** The calibrated surface is unchanged: the
  repaired macro-arm E2 form, the six phase-only starts, borrowed
  completions, the by-value adapter only at `try_apply_function`, and the
  four forwarded starts sessionful. The ownership replay is green with no
  HEAD row retired. The completion/forwarded literal shape gap and the R7
  boundary stay in the unpassed list; do not claim base shape for them.
- **Q7 — R5.** If a transition cannot be made green without a new
  per-operation default-path cost, `unsafe`, a shared queue, a borrowed
  Session in TLS, or a lowered bar, STOP with the mechanism and
  counter-evidence. Do not invent a waiver, and do not cover the R7 boundary
  by adding a dispatcher guard.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-657/`: export `HEAD` with `git archive` into your own probe
root, apply the eight-file F3+E2 surface and the macro-arm E2 form (R6),
build with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked, one common
source path), install each arm's binary, and measure with the supplied
runner serially, alone, on CPU 2, with every `RUM_PROFILE_*` cleared. The
supplied package and the director's probe tree are read-only; copy them
before writing. Prior roots (`.temp/brief-648/`, `.temp/brief-653/`,
`.temp/brief-654/`, `.temp/brief-655/`, `.temp/brief-656/`,
`.temp/director/calib-655/`, `.temp/director/verify-655/`,
`.temp/director/probe-657/`) stay read-only and are never executed in place.

**Do not** design a shared diagnostic queue cursor, make the completions or
forwarded starts session-free, add a dispatcher guard, or change the
calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-657/report-design.json` with:

- the repaired baseline's receipts: the macro-arm build and binary hash, the
  ownership replay verdict lines, the 29 own and incoming pairs, F7, and the
  five red-arm failures;
- the entry inventory with `path:line` and the out-of-contract list;
- the context design: type, ownership, every registration and pop site,
  guard and Drop semantics, suspension/resume/panic/handoff carriage, reset
  and gate-transition behavior, and the Q3 match rule;
- the design arm's receipts: the 29 own and incoming pairs, the 313 panel,
  F7, and every D native and stack bar measured;
- the falsifier matrix with exact records, red on the repaired F3+E2 arm and
  green on the design arm;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list that keeps the R7 boundary, the
  completion/forwarded shape gap, and every bar not measured (including the
  four While rows if the package is not yet created).

Run `python3 -I scripts/report.py check 657` and return its summary and every
compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, live source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- One variable per probe; a single Callgrind pair never decides a
  seed-dependent quantity; state the distribution when a quantity needs one.
- A design that pays for the context with a new per-operation default-path
  cost, a wider public API, a compile-time gate or a lowered bar is not a
  candidate.
- Every claim carries a citation the checker can resolve.
