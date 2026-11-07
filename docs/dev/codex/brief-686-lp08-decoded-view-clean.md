# Brief 686 — LP08 residual: the decoded view without the ambient guard

**Design continuation on the brief-685 form: `Scope: NONE — no live
source, workload, manifest, inventory or version-control change.`
Disposable build probes only, under this brief's root. Brief 685
excluded one added guard, not the decoded view; its form tree is the
starting point, not evidence to re-derive from zero.**

Brief 685 added a `belongs_to_executing_owner` precondition to the
decoded view. At the `observe_boundaries` position the ambient
execution control does not exist yet (`suspension/core/profile_mode.rs:32`
runs before `:34` `enter_front_with_control`; `execution_control()` at
`profiling/logical/context.rs:243` is empty), so the guard faulted
valid bound consumers before the canonical decoder ran
(`canonical_decode_reached=0`). Six pair cases failed with `faults=1`;
guest values stayed unchanged. The decode itself was never executed.

This round answers one question with measured evidence: **with that
guard removed and the remaining provenance checks kept, does the
canonical decoded view classify the stage's own Option correctly —
`none-two` none=2, `down` some=1 on both sides, 20/20
`native_same=true`, `faults=0`, the full enabled battery green — while
the guest slot, carrier, consumer, completion order and all eight
definition finishes stay byte-identical?**

## Stage A — the decoded view and the battery

1. **Calibration.** Before judging the form, reproduce the retained
   state: base 22/22 keys, nine native differences, four mirrors,
   boundary groups 9/2/6; the 684 raw form 19/20 with `none-two`
   queued `[2,2,0,0,2,0,0,0,0,0]` vs resumed
   `[2,2,0,2,0,0,0,0,0,0]`, `down` green, `faults=0`; and 685's fault
   trace (`current_control_present=false`, `canonical_decode_reached=0`).
   A prior round's number is a claim, not evidence.
2. **The form.** Start from `.temp/brief-685/src/form` (the two-file
   decoded view on the 684 frontier). Remove the
   `belongs_to_executing_owner` helper and its use in
   `DownValuesView::observe`; remove the panicking `_ =>
   unreachable!()` fallbacks in the take path and use a non-panicking
   fallback. Keep everything else: the owned `DownValuesView`, the
   `size_of::<Self>` storage charge, the epoch validity and
   `Arc::ptr_eq` check, the `Policy::OptionalExpr` check,
   `data.first(root) == consumer`, root membership, Guest kind, the
   canonical `ContinuationValue::from_continuation_expr` decode of a
   temporary `Expr` copy, and the temporary
   `Slot::State(ContinuationState::OptionalExpr(..))` view passed only
   to the unchanged `ProfileBoundary::observe`.
   - Do not add any other precondition, selector, phase-key aggregate
     or ambient-state check. The insertion path's authenticated
     pending-owner validation plus the retained checks are the whole
     guard.
   - The original outcome, guest slot, consumer, pushed values and
     completion order are delivered unchanged; `LegacyMode`, the typed
     adapter, the transport and every guest carrier stay
     byte-identical; no `StateMode` flip, no outlined adapter, no +64
     publish, no new evaluator-frame field, no new `unsafe`.
3. **The battery.** Run the whole enabled battery with the 684/685
   probes, fixtures, instruments and assertions byte-identical:
   - `queue_boundaries` 9/9, including the parser-error quiescent
     `(0,0,0,0,0)` and the strong-count ownership equality;
   - direct definition finish `Outcome::Some`, `faults=0`, counter
     `[1,1,0,1,0,0,0,0,0,0]`, then quiescent;
   - cancel, reset, drop and invalid leave `open=0`, `leases=0`,
     `fabricated_ok=0`;
   - all 20 pair cases, each `native_same=true`, `faults=0`; quote the
     `none-two` and `down` verdict lines verbatim; the probe's own
     PASS does not waive `native_same`;
   - mirrors 67/67 over all 20 cases (the exhaustive reducer, since
     failed cases have no final PASS line); the seven Throw mutants
     killed at the per-invocation counter with
     `semantics_enabled=false`; all 150 key rows, with every formerly
     passing row restored or an explicit measured regression.
4. **The answer.** If Stage A is green on every control, go to Stage
   B. If it is red, name the failing control with `path:line`, the
   record/state that trips it and the invariant it violates, record
   the measured exclusion, and **stop** — do not try a second
   classification, placement or adapter change in this round.

## Stage B — only if Stage A is green on every control

5. **The stack campaign.** Function and Module recursive stack equal
   base exactly: 18,304 / 18,896 B per level, residual 0, in two
   independent CGU=1 builds and two draws, four stack heights twice
   each. The form must be green on Stage A in both builds.
6. **The disabled deltas.** Every delta on the measured tree is
   attributed per site to the queued path's own
   `runtime_path_timing_start` / `record_profile_runtime_path_timing`
   calls in the same run, or removed. The decoded view is enabled-only
   and must not appear in a disabled slope.

## Stage C — only if Stage B is green

7. **Mispair.** Make the three mispair mutants turn the exact counter
   red with `semantics_enabled=false` without changing the eight
   call-site signatures' observable finish. If that cannot be done,
   return a measured impossibility with the exact reason and a
   proposed falsifier; do not drop the row silently.
8. **The 15 open key rows.** Exercise each at pause, error and
   outcome, or record source-impossibility with `path:line`. The 12
   `phase_absent` rows stay open unless a fixture exercises the route;
   `set.finish` pause, `catch.resume_after_body` pause and
   `catch.eval.total` error need a probe, not an argument.
   `evaluate.total` pause must not return to `open=1`. The four
   `evaluate_atom_fast_path.stable_symbol` / `.atom` pause and error
   rows are source-impossible at `eval_core/mod.rs:174` and stay
   recorded as such.
9. **The contract obligations.** State, with `path:line`, how the
   collector meets `docs/dev/profiling.md`: one enabled, process-wide
   epoch collector; cross-thread Session resume without losing
   records; epoch isolation; reset retirement; nested/interleaved
   owners. Probe what a fixture can exercise; record the rest as open
   obligations for the implementation stage. `current_slot()` is
   process-wide and the bearer of that rule; do not claim it passed
   unprobed.

## Goal

When this round is done, the design record states, with measurement
and not argument:

1. **The decoded view.** The exact diagnostic decode shape, its
   provenance, its validation and its cost semantics, with
   `path:line`; or the measured impossibility with the reason.
2. **The pair.** 20/20 `native_same=true` with `faults=0`, `none-two`
   none=2 and `down` some=1 on both sides, every verdict line quoted —
   or the measured exclusion with the failing control.
3. **The battery.** Every Stage A control green in the same round,
   assertions byte-identical.
4. **The cost rows.** Stack 18,304/18,896 residual 0 in two
   independent CGU=1 builds and two draws on the form that is green,
   and the disabled slopes attributed per site.
5. **The rest.** The mispair kill or its measured impossibility; every
   key row closed or OPEN with a director question; the contract
   obligations recorded as probed or OPEN.
6. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions
   — plus the updated drafted implementation brief in `TEMPLATE.md`
   shape, replacing the brief-685 draft.

If no decoded view can satisfy the pair at the exact stack, the record
is the measured exclusion with its exact rows and the minimum-cost
alternative; the collector stays the reference at 19/20. Promotion
stays blocked until the implementation stage proves every bar.

## Context pointers

- `.temp/brief-685/report-design.json`, sections `answer`, `shape`,
  `root`, `battery`, `cost`, `plan`; `decoded-view.patch`,
  `analysis-fault-trace.log`, `analysis-battery-verdicts.log`,
  `analysis-calibration-raw.log`, `analysis-mirrors.log`, `call-sites.json`,
  `drafts/implementation-lp08-residual-9.md` — the state this round
  continues. Read-only; copy before adapting a script.
- `.temp/brief-685/src/form/` — the two-file decoded view on the 684
  frontier, the starting tree. `.temp/brief-685/src/raw_form/` — the
  684 form. `.temp/brief-685/src/collector/` — the unmodified 682
  collector (19/20 reference). `.temp/brief-685/src/fault_trace/` —
  the read-only trace clone, not a form.
- `.temp/director/brief-685-gate.md` — the exclusion of the
  ambient-owner guard and the shape risk.
- `.temp/director/brief-684-gate.md` — the frontier identity and the
  Q1–Q4 rulings.
- `.temp/director/brief-682-gate.md` — the collector form at 19/20,
  the +64 root cause, the pair bar.
- `docs/dev/profiling.md` — the contract.
- Live sites: `profiling/logical/{mod,context,queue_api,epoch,
  record,completion}.rs`, `profiling/timing.rs`,
  `suspension/typed/{registry,channel,legacy,adapter}.rs`,
  `suspension/primitive.rs`, `suspension/core/{driver,profile_mode,
  profile_queue}.rs`, `suspension/core/profile_queue/arena.rs`,
  `eval_core/{call,mod}.rs`, and the queue-boundary tests.
- Host reset: `crates/rum-host/src/session.rs:176`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-686/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control
  command;
- `.temp/brief-685/` or any other retained root, except to read and
  copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies,
  `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs
and probes; it repairs nothing.

## Hard bars

- **The cost bar.** The disabled path pays only the standing per-site
  gate (measured against the queued path's own call in the same run);
  the recursive Function and Module stack rows equal base exactly
  (18,304 / 18,896 B per level, residual 0) in two independent CGU=1
  builds and two draws. No selection machinery; no off-by-default
  state in a hot frame; no new field in an evaluator frame. The
  excluded forms (selector, +64 session-in-generic, the +64 publish in
  `adopted_slow` and `complete_phase`, outlined adapter, +16/+37, the
  683 observe-only guard, the 684 position-only shape and the 685
  ambient-owner guard) stay excluded.
- **The sampling-position exception.** Only the down-values
  diagnostic marker may move to its actual consumer frontier; every
  other phase stays at its current position. The exception must be
  stated, and an implementation must record it in
  `docs/dev/profiling.md`.
- **The decoded view is diagnostic.** It uses the canonical
  `ContinuationValue` decoder, never a hand-written
  `InternalContinuationSome/None` matcher, never a blanket
  `Slot::Value -> None`, and never a phase-key aggregate. It never
  changes the guest slot, the consumer, the pushed value or the
  completion order. Do not add a precondition beyond the retained
  epoch/policy/first/membership/Guest-kind checks; the ambient
  execution control does not exist at the observe position
  (`profile_mode.rs:32` before `:34`).
- **The controls.** The parser test's assertions, the owner identity,
  the pair check and the guest completion order stay byte-identical.
  No assertion is weakened to make the form green; a red stays a red.
- The instruments are calibrated before any form is judged: the
  corrected nine native vectors, the four mirrors and the boundary
  controls reproduce the current tree.
- Every count, key, route and `path:line` is re-derived from the
  current source; a brief-680 through -685 number is a claim, never
  evidence.
- A form that changes a guest carrier, a pushed value, an adopted key,
  a sampling boundary or a token-first interface is rejected unless
  the director names the exception first. This brief names it for
  `call.stage.down_values` only.
- No live source edit, no version-control command, no network, no new
  `unsafe`, no manifest or workload change.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's
cargo runs plain inside the dispatcher's 16G scope (never nested).
Build both arms at one path with one release codegen unit; two
independent builds and draws for every cost row; serial Callgrind,
CPU-pinned, release, every `RUM_PROFILE_*` cleared except the declared
modes; four stack heights twice each with the residual printed. A
timeout is not data. Persist raw output before parsing; derive any
bound from data.

## Checkpointing

All scratch under `.temp/brief-686/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-686/report-design.json` in `rum-report/1`, with the
  decoded-view shape, the pair verdicts (`none-two`, `down`), the
  battery, the exact-stack answer, the cost rows, the mispair and key
  table, the contract obligations and every open question.
- `.temp/brief-686/drafts/implementation-lp08-residual-10.md` — the
  updated implementation brief in `TEMPLATE.md` shape, plus the probe
  trees, binaries and receipts.
- Run `python3 -I scripts/report.py check 686` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
