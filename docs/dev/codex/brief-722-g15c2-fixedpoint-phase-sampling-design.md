# Brief 722 — the fixed-point phase record's producer-before-consumer sampling (design)

**Shape: design-only. `Scope: NONE` — no source change.** The director's
gate of brief 623's continuation 8 accepted two `Lost` rows and the
retained fixed-point draft. This round turns that draft into a reviewed,
executable plan. You write no code outside `.temp/brief-722/`.

## Goal

Make this true, with evidence, on the current tree: for the two
fixed-point producer rows P1470 and P1471, a valid keyed native
observation made on the resumed path equals the queued path's
observation in every counter, including

1. **the paused `evaluate.once_unstable` phase record.** The logical
   record that the resumed fixed-point frame emits must sample the
   actual completed pass before the fixed-point consumer runs another
   pass, so its ten counters match the queued record for the same key.
2. **the two-pause and three-pause shapes.** Both the `Throw` and the
   `Return` second replies read SAME, and the surrounding `Catch`,
   `Return` and nested shapes keep their language results, messages and
   effects.

The guest language result, evaluation count and order, effects, trace
ordering, request freshness, evaluation-return resource check,
cancellation and abandonment ownership, public carriers and the
disabled path's cost do not change.

## The measured defect (accepted evidence, 2026-11-11)

Continuation 8 of brief 623 found ten valid, source-reached bound native
differences, five on P1470 (`runtime/resolution/resume_frames.rs:37`)
and five on P1471 (`:62`), every one with `semantic_same=true`, and
twelve minimized cases (8 DIFF, 4 SAME, all semantic-SAME):

- The single changed phase is `Runtime { phase: "evaluate.once_unstable",
  source: None }`. The two-pause witness
  `C8FixedPointStep[Input["c8step1"],1,2]` with replies `73` then
  `Throw[83,c8error]` reads queued `[5,2,3,0,0,0,3,0,0,0]` and resumed
  `[5,1,4,0,0,0,4,0,0,0]`.
- The three-pause witness (limit 3, replies `73`, `73`, `Throw`) reaches
  both rows and reads queued `[7,4,3,0,0,0,3,0,0,0]` against resumed
  `[7,2,5,0,0,0,5,0,0,0]`; replacing `Throw` with `Return` changes the
  Return column instead.
- A surrounding `Catch` does not change the phase count. One-pause
  error, first-pass-already-resolved two-pass error, ordinary result and
  held `Return` are SAME controls.
- Both arms report the same uncaught throw and message; this is
  diagnostic accuracy, not language loss.

The evidence is frozen at `.temp/g15-c2/c8/`:
`port/fixedpoint-losses.json` (the ten bound whole-record witnesses and
the twelve minimized cases with their raw logs),
`port/fixedpoint-min-summary.json`, `port/fixedpoint-min-plans.json`,
`port/fixedpoint-execution.json`, `port/fixedpoint-schema.json`, and the
draft at `.temp/g15-c2/c8/port/DRAFT-fixedpoint-phase-repair.md`. Never
overwrite that scene.

## Context pointers

- Read `AGENTS.md` (operational notes, the Testing Policy hard lines),
  `docs/testing-strategy.md`, the evaluation audit and host hardening
  contract, and `docs/dev/codex/TEMPLATE.md` before touching anything.
- The draft names the source surface: `runtime/resolution.rs:377`
  samples the ordinary pass before continuation, `:465` starts the
  resumed pass, `:551` and `:574` are the two suspend helpers;
  `runtime/resolution/resume_frames.rs:37` (P1470) and `:62` (P1471) run
  the subsequent fixed-point work before completing their outer typed
  actions; `suspension/typed/witness.rs:32`,
  `suspension/core/storage.rs:142`, `profiling/logical/mod.rs:1248` and
  `suspension/core/profile_queue.rs:362` show the
  stash → evaluation return → typed consumer → stash publication order.
- The installed definition-resume repair is the neighbouring shape and
  must not be disturbed: publication rides the enabled-only profiled
  queue join through the cold `#[cold] #[inline(never)]` helper, the
  shared constructor has exactly two callers, and the disabled caller
  `resume_eval_request_from_checkpoint` is 81,588 instructions / 30
  calls. The brief-719 repaired neighbour is
  `down_values/default_fallback.rs:41`.
- Acceptance is mechanical: the named witnesses and the named measured
  gates decide, not prose.

## Scope

`Scope: NONE — no source change.` Your work lives in
`.temp/brief-722/`: a byte copy of the live tree and probe-only
diagnostic additions made to that copy, never to the live tree.

## What the plan must contain

1. **A closed inventory** of every paused `evaluate.once_unstable`
   logical record producer and its consumer — the initial and resumed
   fixed-point passes, the two suspend helpers, the stash publication
   and the typed consumer — each with `path:line`, the owning frame,
   the actual result carrier, the queue frontier at that point, and
   whether the record can observe a later pass. Include every caller of
   every function whose duty would move.
2. **Per row, a precise insertion point and signature**: the copied
   experiment's exact edit, why it is the actual completed-pass
   frontier (not merely a convenient return), what it does for
   `Ok(Some)`, `Ok(None)`, `InputRequired`, `Error`, `Return`,
   cancellation and abandonment, and its ordering relative to the
   evaluation-return resource check. A generic queue-marker move is not
   a design; name what samples what, when, and against whose pass
   boundary.
3. **The class duty.** Every site that stashes a pass boundary before a
   witnessed suspension and publishes it after pushing a consumer:
   classify each as covered by this repair, already correct with the
   proof, or a STOP with its reason. No new call edge is left
   unreachable.
4. **The falsifiers.** All ten bound witnesses retained as base-red /
   candidate-green on the copied experiment; the twelve minimized
   cases with eight DIFF → SAME and four controls staying SAME; a fresh
   focused witness per row that the implementation must keep; and a
   per-row omission or misplacement mutant traced to its predicted
   observer (the same red-first calibration as briefs 719-721).
5. **The disabled-path cost bar.** The fixed-point frames and the
   resolution seams are on the resumed path, and the disabled path must
   not pay. Measure instructions per call with Callgrind at identical
   call counts on the copied base and candidate (at least three draws,
   ranges as the floor); pin the bar and name the enabled-only cold
   path that keeps the disabled caller unchanged. A measurable
   default-path increase is a STOP, not a recorded deviation. State the
   caller rows and the same-source control explicitly.
6. **Risks and open questions**, with the exact question when the
   director must decide something.
7. **The drafted implementation brief(s)** in `docs/dev/codex/TEMPLATE.md`
   shape: staged so the copied experiment and its red/green calibration
   come first, then the live insertion with the cost gate and the
   standing replays, each with its own base-red/candidate-green gate.

## Reproduce first

On a fresh byte copy of the live tree with a probe-only diagnostic
package (the C8 `c2-probe` pattern), reproduce the ten bound witnesses
and the twelve minimized cases from `port/fixedpoint-losses.json` with
their exact ten-counter records and language observations. A rerun that
does not reproduce them is a STOP with the raw log; never redefine a
witness. Then reproduce the C8 definition-resume panel's 470-record
parity as the untouched control (the repaired build must stay equal to
the brief-721 accepted panel).

## Instruments and standing exclusions

- Callgrind row `default_builtin_dispatch_spec_for_symbol` is draw-noise
  and is never cost evidence.
- The brief-620 excluded association key-read label
  (`association-part-key-read-width4000` and its sibling) is outside the
  timing lane; instruction rows decide.
- Both arms are built at one path with one codegen unit
  (`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`) and the probe-build cache;
  one `--force` spot check per round.
- A ruled exclusion belongs to the instrument: restate it in every
  reused table.

## Hard rules

- No git in any form, no network, no writes outside `.temp/brief-722/`.
- Never edit `crates/`, `docs/`, `scripts/`, tests or inventories on the
  live tree; scratch copies only. Preserve `.temp/g15-c2/c7/`,
  `.temp/g15-c2/c8/` and `.temp/brief-623/` untouched.
- Rust 1.85.0, offline, locked; one cargo process at a time at eight
  build jobs; native diagnostics serial; every command bounded.
- Every claim carries `path:line` or a raw receipt. Copied log lines for
  every verdict.
- Two consecutive executed bounded experiments with no new evidence
  stop the round; failed launches and pure source reviews do not count.
- No `unsafe`, no dependency change, no public API change, no foreign
  engine comparison, no fabricated carrier, no broad stash drain, no
  weakened diagnostic or inventory.

## Deliverable

A `rum-report/1` JSON at `.temp/brief-722/report.json` (written with
`scripts/report.py`, checked before finish) containing: the plan with
the closed inventory table, the two-row mechanism designs with exact
signatures, the falsifier matrix, the cost rows with raw captures, the
risks and open questions, and the drafted implementation brief(s). Also
leave every raw log, fixture and copied-experiment diff under
`.temp/brief-722/` and name them in the report.

## Verification of this design round

- The witnesses reproduce on the copy with exact raw records; a negative
  control (a deliberately wrong read of one field) must fail the same
  check.
- Each proposed insertion point is exercised in the copy: red before,
  green after (or the measured reason it cannot be green yet), quoted
  from raw logs.
- The disabled-path rows read equal at identical call counts on the
  copied base, with the candidate's range pinned.
- The 470-record definition-resume control stays whole-record equal to
  the brief-721 accepted panel.
- The report cites every claim; `report.py check` passes; every drafted
  implementation brief is TEMPLATE-shaped and names its own gate.

Stop and report; the director reviews the plan, settles the open
questions and writes the implementation brief.

## Director gate (2026-11-11) — accepted; the questions are ruled and brief 723 is written

**Run integrity.** `.temp/logs/cdx_brief_722.log`: 13,748 lines, sha256
`b97f8dfe432fba63c50bfa6d46214f8726ddf5a18782663e8747371b414a7b5e`,
memory-scope peak 2.58 GB, `codex exit=0`. All 58 ledger commands bounded
(max 68.3 s), zero git or network invocations. **My own reproductions:**
the live tree is 2,712 files equal to the entry manifest (0 missing, 0
extra, 0 differing; digest `388f7e69…`); the protected scenes replay
whole (`c7` 25,251, `c8` 36,460, `brief-623` 151 entries hash-equal); the
five build receipts are rustc 1.85.0, offline+locked, `-j8`, one path
(with CGU=1), base `a4df42c1…` and candidate `5e171a15…` equal to the
forced same-source control; all 172 artifact hashes match; `report.py
check` reads OK with 448 citations.

**The reproduction is exact, whole-record.** All 90 base records equal the
frozen C8 `fixedpoint-real-1.log` records field-for-field, and all 12
minimized records equal `fixedpoint-min-real-1.log` (not only the ten
bound records in `port/fixedpoint-losses.json`): 10 DIFF / 80 SAME and
8 DIFF / 4 SAME, the 10 DIFF ids exactly the five P1470 and five P1471
bound records, the only changed phase `evaluate.once_unstable`, every
record valid and semantic-SAME. The candidate reads 0 DIFF over both
panels with the 84 non-DIFF records whole-record exact, the queued arms
byte-identical and every language observation identical; the 470-record
definition panel is exact on both arms and equal to the brief-721
accepted `sample-all-records.json` (468 valid, the same two invalid
controls); the two fresh witnesses are base-red/candidate-green in the
single phase; the explicit nested witness is base-red/candidate-green
with exact language; the two surrounding-Return records are whole-record
identical between arms and retain only the pre-existing
`call.stage.prepare_arguments` difference; the exploratory nested-reply
fixture stays invalid on both arms. Both omission arms reproduce: the
probe-disabled 104 records are byte-identical to the uninstrumented
candidate, P1470 changes exactly 20 records and P1471 exactly 14, every
changed record carries exactly one one-pass signature
(Throw `[0,-1,1,0,0,0,1,0,0,0]`, Return `[0,-1,1,0,0,1,0,0,0,0]`) with
the queued arm untouched, and P1471 leaves the two-pause Throw green.
The frontier receipts read 714 P1470 + 215 P1471 at
`after=EvaluationReturn before=fixed_point_consumer`, and the own-row hit
counts match them one for one.

**The disabled cost bar holds under my own parse.** I re-read all 18 raw
Callgrind captures (resolving callgrind ids and summing self Ir and
incoming calls myself): three draws are identical per arm for every rum
function; every deterministic `rum`/`rum_core` row is equal across base,
candidate and the forced control; every named function's call count is
equal; the 13 selected rows are exact, including `resume_eval_request_
from_checkpoint` 58,208/26 (fixed-point) and 81,588/30 (definition); both
new helpers occur zero times in every disabled capture; the forced
rebuild's sha equals the candidate's. The only differing rows are
symbol-less libc/ld address rows and two shared std helpers (16 rows in
all) — the standing layout/address noise class, never product signal. The
worker's 671-row table contains only rum rows and does not list the
standing `default_builtin_dispatch_spec_for_symbol` draw-noise symbol.

**The mechanism is coherent with the live source.** I read `push`,
`return_inserted`, `publish_stashed_after_consumer`, `add`, `register_
owner`, `take_belonging`/`drain_attached_for`, `belongs_to_admitted`,
`Call::retain`, `has_runtime_key`, both suspend helpers, the typed
witness, `complete_evaluation_return`, `drive_frames` and
`Profiled::resume`. The accepted selector is strictly narrower than the
existing enabled drain (exact epoch Arc + `belongs_to_admitted` control
and generation + the once-pass key + uniqueness), and the relocated
boundary is observed by `Profiled::resume` with the completed pass's
actual `Result<Slot, EvalError>` after `complete_evaluation_return`
(resource check applied) and before the fixed-point consumer frame is
dispatched. The closed inventory holds on the live tree: `stash_boundary`
has one production caller (`logical/mod.rs:1264`),
`pause_phase_at_finish` and `suspend_evaluation_return` each have exactly
the two resolution helpers, `ProfileQueue::push` has exactly the five
storage callers, and `register_owner` keeps one registered queue per
control.

**Ruled:** accepted in full. Q1 yes — stage A executes the fresh
resource-return rejection, both-pause cancel/abandon/restart,
foreign/interleaved generation and cap/registration-race probes with real
producers before any live edit; the queued arm's ten counters are the
numeric oracle. Q2 no — the unchanged `call.stage.prepare_arguments`
difference stays outside this selector; it is recorded as a new measured
finding and continuation 9 of brief 623 takes it up. Q3 keep the exact
epoch/control/generation matching: duplicate match faults visibly and
transfers nothing, zero match and a missing owner leave the stash intact
so the existing publisher still publishes (no boundary loss), and stage A
must watch each branch. Q4 the three-file organization exception is
accepted for the live insertion; the responsibility split is recorded as
deferred organizational debt with its own gate.

**Recorded, not gated:** the three failed harness runs (attribution
first-line parse, the Return-id delta parser, the intentionally stronger
surrounding-Return check) and the one launch error are instrumentation
failures with raw logs retained; the plan's sentence crediting the
attributed arm with the frontier receipts names the wrong arm — the 929
receipts come from the mutant build's probe-disabled calibration, and the
route is still proven; the mandatory `--force` overwrote the same-key
cargo log, so that one compiler stream is not separately retained
(recorded, not reconstructed); one exploratory nested-reply fixture
stays invalid.

**State.** Brief 723 is written: stage A is the copied failure-frontier
proof and permanent-regression calibration, stage B the live insertion
with the standing replays, the cost gate and the acceptance suites. After
its gate, continuation 9 of brief 623 re-settles P1470/P1471 and takes up
the prepare-arguments finding. C2 stays Open at 1,038 rows: 334
residuals + 2 Lost + 35 `NoObservableTail`, 77,940 keys, 618 accepted
carries.
