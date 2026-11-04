# Brief 658 — LP06 F design round 4: the session-execution context on the ok-form baseline

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Brief 657 stopped on a real, structural checked-expansion obligation: the R6
fourth macro arm changes the typed-resume macro definition, and the unchanged
checked adapter
(`crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs`,
`production_rejection_census_is_complete`) rejects it by exact tokens before
any semantic acceptance. The director resolved the baseline differently: E2's
delta is expressible with the existing three-arm macro as
`ok(value) => finish(self, Ok(value))`, leaving the macro definition
byte-identical to HEAD. This brief designs the still-missing piece — the
session-execution context — on that ok-form baseline.

## Goal

Design, and prove with disposable builds under this brief's root, the
session-execution context that makes the six session-free starts **equal to
the sessionful form in every host-reachable evaluation state** — root,
deferred enable and reset mid-evaluation, resume, reentrancy, several sessions
on one thread, thread handoff, panic/unwind, and off-to-on/on-to-off gate
transitions — while every one of the 29 retained rows stays at or below the
pre-A1 base's own slope, the incoming-call slopes stay equal, F7 stays at most
856, the 313-label disabled panel has no positive row, and every D native and
stack bar holds. The calibrated surface is the ok-form F3+E2 surface; the
design adds the context and nothing else. Then draft the implementation brief
in `TEMPLATE.md` shape.

The context may not be paid for with a new per-operation branch on the
disabled path. No guard at a per-operation dispatcher — including
`eval_control_data_builtin` and the individual `eval_*` helpers — and no
wider public API, compile-time gate, `unsafe`, new module boundary, borrowed
`Session` or raw pointer in TLS, cached generation/root/epoch, or shared
mutable queue. At most one cold-load check per evaluation, resume or
abandonment entry, and no TLS access when the profile is not configured.
Do not change the typed-resume macro definition, its invocation grammar, or
the checked adapter; the E2 form is fixed (R8). If the design needs a
macro-grammar or checked-adapter change, STOP (Q7).

## Context pointers

- `.temp/director/brief-656-gate.md` — R6 (the superseded macro arm) and R7
  (the coverage boundary).
- `.temp/brief-657/report-design.json` — the R6 reproduction, the static
  entry inventory (`q1_entry_inventory`, `q1_out_of_contract`), the unbuilt
  context sketch (`q2_context`, `q3_match_rule`, `q4_disabled_path`), the
  prepared falsifier matrix (`q5_falsifier_matrix`, `q5_helper_bodies`,
  `q5_limitations`), `q6_shape`, `q7_r5`, and `unpassed`. Read it first; do
  not re-derive it.
- `.temp/brief-657/probe/design-src/` — the context sketch tree (red
  baseline + context edits; `witness.rs` still carries the fourth arm, and
  the `eval_dispatch` site is still `produced(value)`).
  `.temp/brief-657/probe/red-src/` — the red tree with the disposable
  falsifiers. `.temp/brief-657/transitions.rs` — the prepared transition
  probes. `.temp/brief-657/probe/instrument/` — the worker's instrument
  copy.
- `.temp/director/probe-658-okform/` — the director's ok-form build (binary
  `e6fc002a…`), the supplied-package copy, the 29-row calibration receipts,
  `f7.log`, `census-test.log`, `discovery.log`, `reduce-okform.log`. It is
  the known answer to reproduce, not to reuse; copy before writing.
- `.temp/brief-655/supplied/` — the supplied instrument (29 labels, 58
  frozen programs, known-answer base binary, serial Callgrind runner,
  reducer, F7 prober, build recipe). Copy it into your own probe root before
  use.
- Live source: `crates/rum-runtime/src/runtime/resolution.rs` (`evaluate`,
  `evaluate_at_root`), `.../suspension/core.rs` (`resume_suspended_slot`),
  `.../suspension/core/abandon.rs` (`drain_abandonment_worklist`),
  `.../profiling/logical/mod.rs` (`Call::start`, `start_owned`,
  `start_from_active`, `open_phase_from_active`), `.../profiling/logical/
  segments.rs` (`owner`, `current_owner`, `enter`),
  `.../profiling/logical/context.rs`, `.../profiling/config.rs` (the
  cold-load `AtomicU8` gate),
  `.../kernel_data/control_data/mod.rs:64` (the out-of-contract direct
  dispatcher).
- `.temp/brief-653/native-bars/comparison.json` and
  `.temp/brief-653/instruments/measurement-contract.json` — the D shape and
  the 313-label panel contract. `.temp/brief-654/` — the While package design
  whose four rows are still unmeasured.

## Established by the director (reproduce before doubting)

1. **R8 — the baseline is the ok-form.** With `witness.rs` exactly at HEAD
   and the site written `ok(value) => finish(self, Ok(value))`, the binary
   `e6fc002a…` (my build: one common path `b/active`, release, CGU=1, jobs
   8, offline, locked) gives 29/29 own pairs exactly equal to base (0
   positive, 0 below), 29/29 incoming slopes equal, and F7 `try_dispatch_
   builtin_call` **632/600** against base 856/856 (the superseded macro arm
   was 616/600). The two `finish` call sites are identity on `Err` (the
   unchecked closure maps; the growth validator early-returns on `Err`), so
   the non-suspending error path is observationally identical to the hand
   expansion's `produced => finish(self, produced)`.
2. **The checked adapter accepts the ok-form.** The census definition check
   passes; the remaining exact-count assertion is 1076 vs 1067, the F3
   surface's ordinary frontier move, which the source stage pairs and rules
   (as the test's own comment history does: 1040 -> 1039 -> 1067). It is not
   a baseline blocker and must not be hand-edited in a probe tree.
3. Ownership discovery on the ok-form tree: **48/48** `resume_typed_or_
   suspend!` rows, **retired=0**, 3 new `UnknownConstruct` at the F3-touched
   files (`segments.rs` x2, `timing.rs` x1); the `eval_dispatch/mod.rs` row
   survives at 233 (start column 22 -> 9, end 240 -> 238).
4. The E2 hand expansion and the fourth macro arm are both superseded; do
   not reintroduce either.
5. The red arm reproduces the five failures at `--test-threads=1`; the
   `queued=true` LP06 iterations pass, so a plain host evaluation is green.
6. `segments::owner(&control)` matches by `Arc::ptr_eq` and requires a valid
   epoch and an open path; a root is `epoch::current()?` with the context
   control, its `logical_generation()`, and `epoch::next_id()`.
7. The six session-free starts are `runtime_path_timing_start` at
   `branching_loops.rs:156/183/365`, `nonlocal.rs:81/93` and
   `scoping/callable.rs:63`. The E1 by-value adapter stays only at
   `try_apply_function`.

## Questions to answer

- **Q1 — entry coverage, dynamically confirmed.** Start from the 657
  inventory (16 static route groups). For every host-reachable entry that can
  reach one of the six starts without an enclosing guard — depth-0
  `evaluate`/`evaluate_at_root` and every `eval_*`/request/script route,
  `resume_suspended_slot`, `drain_abandonment_worklist`, snapshot restore,
  evaluation cache, typed-slot resume, request handling — show `path:line`,
  the guard placement, and that the unconfigured path returns before any Arc
  clone or TLS access. Name the out-of-contract entries under R7 with their
  production callers and do not assert them.
- **Q2 — the context and guard.** The exact type and ownership (TLS holds
  only the owned control Arc; the native guard holds the displaced and
  installed Arcs and is thread-bound), every registration and pop site, Drop
  on normal return, error, `InputRequired` and unwind, suspension/resume
  carriage, thread handoff, the rare request-generation rollover (a matching
  refresh of the TLS control; same-session nested Drop propagates a new
  control instead of restoring an obsolete Arc), reset, and off-to-on/on-to-
  off gate transitions (registration checks configured, not active). No
  borrowed `Session`, raw pointer, `unsafe`, cached generation/root/epoch.
- **Q3 — the match rule.** The exact predicate reproducing `Call::start`:
  `segments::owner(&context_control)` yields a child with the inherited
  epoch, owner and parent; otherwise `epoch::current()?` plus
  `Owner { control, logical_generation(), root: epoch::next_id() }` with
  parent `None`. State what makes a root, whether repeated roots are
  bounded, and what happens when the context is absent. Do not add a
  generation-equality predicate to `current_owner`.
- **Q4 — the disabled path.** The exact added work at each entry: the
  configured cold-load check, the Arc clone only when configured, no TLS
  access when not configured. Measure that no row of the 313-label panel and
  none of the 29 own slopes moves, and that no measured entry prologue grows.
- **Q5 — the transitions.** A disposable falsifier per state, each with its
  exact body, red on the ok-form red arm and green on the design arm: the
  three LP06 resumed tests; the deferred-enable/reset falsifier (host path);
  reentrancy; two sessions on one thread; thread handoff; reset; off-to-on
  and on-to-off gate transitions; panic/unwind; and a host-root control that
  is green on both arms. State the expected records for each (keys, calls,
  owner control and generation, root ids, open=0, faults=0), not only
  `calls > 0`. Do not assert the out-of-contract direct dispatch.
- **Q6 — shape and bars.** The calibrated surface is unchanged: the ok-form
  E2 body, the six phase-only starts, borrowed completions, the by-value
  adapter only at `try_apply_function`, and the four forwarded starts
  sessionful. The ownership replay is green with no HEAD row retired; the
  census definition/grammar check passes and the design tree's exact count
  and its identity pairing are recorded for the source stage. The
  completion/forwarded literal shape gap and the R7 boundary stay in the
  unpassed list; do not claim base shape for them. Measure the design arm's
  29 own/incoming pairs, the 313 panel, F7, and every D native and stack bar.
- **Q7 — R5.** If a transition cannot be made green without a new
  per-operation default-path cost, `unsafe`, a shared queue, a borrowed
  Session in TLS, a macro-grammar/checked-adapter change, or a lowered bar,
  STOP with the mechanism and counter-evidence. Do not invent a waiver, and
  do not cover the R7 boundary by adding a dispatcher guard.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-658/`: copy the 657 red/design trees and the director's ok-form
tree into your own probe root; rebase the design sketch to the ok-form by
exactly two edits (`witness.rs` to HEAD; the `eval_dispatch` site to the
ok-form); build with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked, one common
source path), install each arm's binary, and measure with the supplied
runner serially, alone, on CPU 2, with every `RUM_PROFILE_*` cleared. The
supplied package and every prior root are read-only; copy them before
writing. Prior roots (`.temp/brief-648/`, `.temp/brief-653/`,
`.temp/brief-654/`, `.temp/brief-655/`, `.temp/brief-656/`,
`.temp/brief-657/`, `.temp/director/calib-655/`,
`.temp/director/verify-655/`, `.temp/director/probe-657/`,
`.temp/director/probe-658-okform/`) stay read-only and are never executed in
place.

**Do not** change the typed-resume macro or the checked adapter, hand-edit an
inventory or census count, design a shared diagnostic queue cursor, make the
completions or forwarded starts session-free, add a dispatcher guard, or
change the calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-658/report-design.json` with:

- the ok-form baseline reproduced: the build receipt and binary hash, the 29
  own and incoming pairs, F7, the ownership replay verdict lines, and the
  five red-arm failures;
- the entry coverage with `path:line` and the out-of-contract list;
- the context design: type, ownership, every registration and pop site,
  guard and Drop semantics, suspension/resume/panic/handoff carriage, reset
  and gate-transition behavior, and the Q3 match rule;
- the design arm's receipts: the 29 own and incoming pairs, the 313 panel,
  F7, and every D native and stack bar measured;
- the falsifier matrix with exact records, red on the ok-form red arm and
  green on the design arm;
- the census definition check verdict and the design tree's exact count with
  its identity pairing;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list that keeps the R7 boundary, the
  completion/forwarded shape gap, and every bar not measured (including the
  four While rows if the package is not yet created).

Run `python3 -I scripts/report.py check 658` and return its summary and every
compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, live source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- One variable per probe; a single Callgrind pair never decides a
  seed-dependent quantity; state the distribution when a quantity needs one.
- A design that pays for the context with a new per-operation default-path
  cost, a wider public API, a compile-time gate, a macro-grammar change or a
  lowered bar is not a candidate.
- Every claim carries a citation the checker can resolve.
