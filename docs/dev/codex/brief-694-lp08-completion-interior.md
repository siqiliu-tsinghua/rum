# Brief 694 — LP08 residual: the completion interior

**Cost design on the brief-693 candidate2 basis: `Scope: NONE — no live
source, workload, manifest, inventory or version-control change.`
Disposable build probes only, under this brief's root. Candidate2 is the
byte-identical placement form measured in briefs 689–693; the 686 form
stays the enabled/stack control.**

Brief 693 measured two things with four cells each: an inert
`black_box(phase)` inside `pause_phase_at_finish_slow` leaves the
terminal at 55, so the +1 is stable under a semantically inert
perturbation; and base plus the support closure reads 54 while adding
only the exact candidate2 pause statements flips it to 55, so the +1 is
keyed on that body, not on the rest of the form. Its one cost candidate
reached the base 54 sequence but was excluded as a duplication: it added
`pause_phase_after_consumer` in the four suspend helpers **without
removing the base `pause_phase_at_finish` call at the call site**, so the
pause completed twice, `take_phase` faulted, and the enabled battery was
0/20 with queue, flush and all six lifecycle actions exiting 101.

That exclusion leaves a target the earlier rounds did not measure. The
693 candidate's `complete_phase` `InputRequired` arm was already changed
(it stashes the boundary) and the terminal still read 54 once
`pause_phase_at_finish_slow` was back to base source. So the +1 follows
the pause slow body, not the cold function it calls.

This round answers one question with measured evidence: **can the
DownValues attachment and the non-fixed-point publication live in
`complete_phase`'s `InputRequired` arm — the cold completion function the
base pause slow body already calls, reachable only with profiling on —
leaving `pause_phase_at_finish_slow` byte-identical to the base source,
with exactly one phase completion, no new gate, no suspension-helper
change and the terminal branch on the base sequence; or is that measured
impossible?**

## Stage A — calibration and reachability

1. **Control and basis.** Reproduce the 693 control before judging any
   candidate: base 54 with one `0x328(%rsp)` load; form and candidate2 55
   with two loads; the 20 cost cells in two builds and two draws; the
   stability pair (`inert_stability` 55, `minimal_support` 54,
   `minimal_pause` 55) and the 690/691/692 diagnosis cuts; the 686 form's
   20/20 pairs and 67/67 mirrors; the 150 key rows (131 PASS / 19 OPEN);
   the seven Throw kills; the 386-snapshot trace with the `R, G, B` tail;
   the exact 18,304 / 18,896 stack; the 94 and 96 resumed-helper
   witnesses; the five watched control falsifiers. Reproduce candidate2
   byte-identically from `.temp/brief-693/src/candidate2/`; every 693
   number is a claim, not evidence.
2. **The reachability derivation.** From the source, enumerate every
   caller of `complete_phase` with the exact `path:line` and the outcome
   each passes. Show that the `InputRequired` arm is reachable only from
   `pause_phase_at_finish_slow` (`timing.rs` gates its defer-pause call
   with `!matches!(outcome, InputRequired)`; the other calls pass
   Ok/None/Error), and that `pause_phase_at_finish` gates the slow call on
   `lexical_profile_enabled`, so the arm is unreachable when profiling is
   off. This derivation is the shape's semantic precondition; a caller
   that can pass `InputRequired` invalidates shape 1 before it is built.
3. **The retained green.** Persist the control's per-address cost panels,
   cell traces and stack rows as the control for every candidate. The
   two resumed-helper witnesses and the source exclusion are re-stated
   from the retained roots; the final candidate re-measures them.

## Stage B — at most two cost candidates

4. **Shape 1, the completion interior (primary).** Start from the
   byte-identical candidate2 basis. Move the pause's DownValues
   attachment and the non-fixed-point publication into
   `complete_phase`'s `InputRequired` arm, and restore
   `pause_phase_at_finish_slow` to the byte-identical base source
   (`PhaseKey::from_name` plus `complete_phase(session, key,
   InputRequired, |_| {})`, exactly as `src/base` has it). The four
   suspend helpers and the six pause call sites stay unchanged; the
   attachment and publication are reached at the same point in the same
   order, one level down the call chain. The worker chooses the mechanism
   inside the arm (inline or a cold helper called from it); the goal is
   the shape — a base-source pause slow body and one completion.
5. **Shape 2, only if shape 1 fails.** The one-completion suspension
   relocation: the 693 candidate minus the duplicate — move the pause
   into the suspend helpers **and remove** the four call-site
   `pause_phase_at_finish` calls, so exactly one completion runs. Its
   disabled cost must be accounted as a move, not as a new gate: the
   helper's added rows (the phase computation, the `LEXICAL_PROFILE_ENABLED`
   gate, the call setup and the Result spill) must equal the caller's
   removed rows for the same statements, row by row, with no unrelated
   negative row used as an offset. The 18,304/18,896 stack, the enabled
   battery, the position trace and the five falsifiers are the same bars.
6. **Ordering and semantics.** The attachment still runs immediately
   after the consumer push and before any later push that could replace
   its insertion handle; the publication follows the attachment; the
   `EvaluateOnceUnstable` exception keeps publishing in the Profiled
   typed-push arm after its push; every early exit of the original body
   is preserved. Owner/epoch/control authentication, the exactly-once
   drain and the ownerless restore stay byte-identical. The base pause's
   `complete_phase` is the only completion; a second `take_phase` of the
   same phase is a red, not a harmless no-op. The enabled battery decides.
7. **The acceptance bar.** The terminal branch executes the base
   instruction sequence at 50 and 100 iterations, in two independent
   CGU=1 builds and two draws, with the terminal Some/None ownership
   invariant stated and preserved: Some restores all prepared arguments
   and Unevaluated wrappers before normalization, None uses the final
   shared arguments, and the restoration plan is owned and dropped
   exactly once. If measurement shows no shape reaches it without
   violating the invariant, return a measured impossibility naming the
   exact opcode, branch, invariant, entailing statement, the excluded
   shapes and the smallest alternative. No universal impossibility beyond
   the measurement.
8. **Constraints.**
   - **Zero at every former site.** For shape 1, the pause slow body is
     byte-identical to base and `complete_phase` is unreachable with
     profiling off — the disabled run must show the base sequence at the
     former sites and zero instructions in the changed arm. For shape 2,
     the move accounting of #5.
   - **Position.** The 386-observation cell trace must equal the 686
     control's in every case and every draw (`R, G, B` at the fixed
     point). The trace is the proof.
   - **Identity.** The candidate2 ownership authentication, the
     exactly-once drain, the ownerless restore, the DownValues consumer
     frontier, the pause / suspend order, the decoded view, the canonical
     `Option<Expr>` decoding, the typed adapter, `LegacyMode`, the
     transport, the guest carriers and the eight definition finishes stay
     byte-identical. No test or fixture is weakened.
   - **No hidden state.** No new evaluator-frame field, no session field
     carrying diagnostic state, no selector, no outlined guest adapter,
     no new `unsafe`, no `+64` publication frame.
   - **One variable per candidate.** At most two cost candidates, each
     starting from the byte-identical candidate2 basis; diagnosis arms
     are not candidates.
9. **Falsifiers.** Re-watch the five 692 falsifiers on the final
   measured shape: position (`R,G,B` → `B,R,G`), owner-loss,
   wrong-owner / premature, exactly-once, and the terminal invariant
   (dropped or duplicated Some restoration observed by its counter
   assertion, not merely compiling differently). Each carries its exact
   counter or trace assertion and its observed red.

## Stage C — acceptance on the green cost form

10. **Enabled.** Queue 9/9 with parser quiescence `(0,0,0,0,0)` and
    strong-count ownership equality; slot 2/2; record 6/6; direct
    definition flush `Outcome::Some`, `faults=0`, counter
    `[1,1,0,1,0,0,0,0,0,0]`, quiescent; six lifecycle actions
    `open=0 leases=0 fabricated_ok=0`; all 20 pairs `native_same=true`
    `faults=0`; mirrors 67/67; seven Throw counter-only kills; all 150
    key rows with every prior PASS restored.
11. **Stack.** Function 18,304 / Module 18,896 B per level, residual 0,
    two independent CGU=1 builds and two draws, four heights twice each,
    on the same candidate that passed the battery.
12. **Coverage.** The two resumed-helper witnesses measure at the
    base-identical 94 and 96 self instructions with equal opcode and
    operand sequences in two builds and two draws when the helpers are
    untouched (shape 1); for shape 2 the moved helper's rows are
    accounted against the removed caller rows as in #5. The third stays a
    source-proved exclusion with its revalidation triggers restated.
13. **Cost.** Every former publication site zero versus base in the same
    run; the terminal branch at the base instruction sequence; every
    remaining disabled delta attributed to the queued path's own standing
    start/record calls or removed; the `builtin_catalog` lookup variation
    qualified by a layout control or a same-build repeated comparison. No
    netting of unrelated negative deltas.
14. **If Stage C is green** — do not run the downstream obligations
    (mispair, the 15 key routes, the contract fixtures) in this round;
    record the green cost form and draft the next round for them. If it
    is red, record the measured exclusion and stop.

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **The reachability answer.** Every `complete_phase` caller and the
   outcome it passes, with `path:line`, and whether the `InputRequired`
   arm is reachable only from the pause and only with profiling on.
2. **The relocation answer.** A shape with a base-source pause slow body
   (shape 1) or the one-completion suspension move (shape 2) that returns
   the terminal branch to the base sequence with one completion and no
   new default-path instruction, or the exact measured impossibility.
3. **The preserved green.** The position trace, enabled battery, exact
   stack, coverage witnesses and five watched falsifiers on the final
   shape.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the ripple inventory, risks and open
   questions.

## Context pointers

- `.temp/brief-693/report-design.json` — the stability pair, the
  duplicate-pause exclusion and every signed row.
- `.temp/brief-693/terminal-relocation-answer.json`,
  `final-qualification.json` — the entailing statement and the
  qualification limits.
- `.temp/brief-693/src/candidate2/` — the byte-identical placement form;
  `src/form/` — the 686 control; `src/suspension_local_phase/` — the
  excluded duplicate-pause shape; `src/minimal_support/`,
  `src/minimal_pause/`, `src/inert_stability/` — the stability pair.
- `.temp/brief-693/inert_stability.patch`, `minimal_pause.patch`,
  `minimal_support.*`, `*-terminal-cut.json`, `all-arm-stack-results.json`,
  `all-delta-rows.json`, `coverage/` and `costs/`.
- `.temp/director/brief-693-gate.md` — the stop and the rulings.
- `docs/dev/profiling.md` — the contract.
- Live sites: `profiling/logical/mod.rs` (`complete_phase`,
  `pause_phase_at_finish_slow`, `publish_stashed_phases_slow`,
  `take_phase`), `profiling/timing.rs` (the defer-pause guard),
  `profiling/logical/queue_api.rs`
  (`logical_insert_down_values_before_consumer`),
  `profiling/logical/collector.rs`, `suspension/core/storage.rs`,
  `suspension/core/profile_queue.rs`, `eval_core/call.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-694/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-686/` … `.temp/brief-693/` or any other retained root,
  except to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing live.

## Hard bars

- **The disabled cost bar.** The default path pays only the queued
  path's own standing `runtime_path_timing_start` /
  `record_profile_runtime_path_timing` calls, measured in the same run.
  For shape 1 the pause slow body is byte-identical to base and the
  changed arm is unreachable with profiling off; a gate or a call added
  on the disabled path is not allowed, even at +1, wherever it lives.
  For shape 2 the move is accounted row by row against the removed caller
  rows; an unrelated negative delta is not a waiver. A source-identical
  function is not a cost claim by itself.
- **The position bar.** The queue tail and every intermediate snapshot
  equal the 686 control's in all 20 trace cases, in both draws.
- **Publication semantics.** A publication may not be dropped,
  postponed past its owner, or merged into a path that loses an open
  record. The DownValues attachment must stay before its publication and
  before any push that can replace its insertion handle. Exactly one
  phase completion; owner/epoch/control authentication, exactly-once
  completion and the ownerless restore stay stated with `path:line`.
- **The enabled form.** 20/20, `faults=0`, the unchanged battery and the
  exact 18,304/18,896 stack in two independent CGU=1 builds and two
  draws. No assertion or fixture is weakened.
- **The excluded forms.** The 683 observe-only guard, the 684
  position-only shape, the 685 ambient-owner guard, the panicking
  take-path fallback, both 687 cost shapes, the 688 owner-position
  placement, the 689 candidate1 shape, the 690 reload1 and reload2
  terminal shapes, the 691 `attach_helper` and `attach_publish` shapes,
  the 692 `whole_body` and `whole_phase_first` shapes and the 693
  `suspension_local_phase` duplicate-pause shape stay excluded. Every
  diagnosis cut is diagnosis only.
- **Every arm reduced.** Every candidate's stack and battery runs are
  reduced into the persisted artifact; a run that exists only as a log
  is not evidence. No row dropped, no negative netted.
- Every count, key, route and `path:line` is re-derived from the
  candidate2 basis; a brief-693 number is a claim, never evidence.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit; two independent builds
and draws for every cost row; serial Callgrind, CPU-pinned, release,
every `RUM_PROFILE_*` cleared except the declared modes; four stack
heights twice each with the residual printed. A timeout is not data.
Persist raw output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-694/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-694/report-design.json` in `rum-report/1`, with the
  reachability derivation, the candidate evidence (or the exact
  impossibility), the re-watched falsifiers, the green enabled/stack
  battery and every delta row.
- `.temp/brief-694/drafts/implementation-lp08-residual-18.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 694` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the measured
answers and the draft's bar list.
