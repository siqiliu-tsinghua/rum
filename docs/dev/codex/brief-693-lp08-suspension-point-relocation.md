# Brief 693 — LP08 residual: the attachment and publication at the suspension point

**Cost design on the brief-692 candidate2 basis: `Scope: NONE — no live
source, workload, manifest, inventory or version-control change.`
Disposable build probes only, under this brief's root. Candidate2 is the
byte-identical placement form measured in briefs 689–692; the 686 form
stays the enabled/stack control.**

Brief 692 measured that the whole pause body behind a thin cold
dispatcher still reads 55 against base's 54 (`whole_body`,
`whole_phase_first`), and that splitting the publication by phase
(`diag_general_publish`, `diag_down_publish`) leaves both at 55. The
resume function's symbol is 0x12d3 bytes in base and 0x130f in every
non-base arm, with an identical external call-target set: the extra
`0x328(%rsp)` load is one stable codegen form of the whole candidate2
form, not a reload-style variant. The placement family inside
`pause_phase_at_finish_slow` is exhausted; what keeps the reload is that
the pause function can reach the attachment and the publication at all.

This round answers one question with measured evidence: **can the
DownValues attachment and the non-fixed-point phase publication move to
the suspension point — inside the suspension helper, immediately after
the consumer push — leaving `pause_phase_at_finish_slow` byte-identical
to the base source, so the terminal branch returns to the base
instruction sequence with ordering, position and every invariant
preserved; or is that measured impossible?**

## Stage A — calibration and stability

1. **Control and basis.** Reproduce the 692 control before judging any
   candidate: base 54 with one `0x328(%rsp)` load; form, candidate2,
   whole_body and whole_phase_first 55 with two loads; the 20 cost cells
   in two builds and two draws; the five 690/691 diagnosis cuts and the
   two phase splits; the 686 form's 20/20 pairs and 67/67 mirrors; the
   150 key rows (131 PASS / 19 OPEN); the seven Throw kills; the
   386-snapshot trace with the `R, G, B` tail; the exact 18,304 / 18,896
   stack; the 94 and 96 resumed-helper witnesses. Reproduce candidate2
   byte-identically from `.temp/brief-692/src/candidate2/`; every 692
   number is a claim, not evidence.
2. **The stability probe.** On the byte-identical candidate2 basis, add
   a semantically inert perturbation inside `pause_phase_at_finish_slow`
   (for example a call to a new empty `#[cold] #[inline(never)]`
   function, or `core::hint::black_box` on a value the function already
   has) and measure the terminal branch in two builds and two draws. If
   the count moves away from 55, the base-sequence bar is not a stable
   function of the source, and the round states that measured
   instability and stops Stage B. If it stays 55, the bar is stable
   under an inert perturbation and the round may proceed.
3. **The minimal-form probe.** On the base tree, add only the
   pause-path attachment and publication statements (the candidate2
   pause body) without the rest of the profiling form, as a diagnosis
   arm; measure the terminal branch in two builds and two draws. It
   decides whether the pause statements alone flip 54 to 55 or whether
   the rest of the form is required. It is diagnosis only, need not be
   semantically complete, and may not be promoted.
4. **The retained green.** Persist the control's per-address cost
   panels, cell traces and stack rows as the control for every
   candidate. The two resumed-helper witnesses and the source exclusion
   are re-stated from the retained roots; the final candidate
   re-measures them.

## Stage B — at most two cost candidates

5. **The shape.** Each candidate starts from the byte-identical
   candidate2 basis and moves the pause's DownValues attachment and the
   non-fixed-point publication to the suspension point where the
   dispatch consumer is pushed: inside the `suspend_resumed_*_with_boundary`
   helper, immediately after that push. `pause_phase_at_finish_slow`
   becomes the base source again — `PhaseKey::from_name` and
   `complete_phase(session, key, InputRequired, |_| {})` — so its
   codegen and its reachable callees match base. The worker chooses the
   two placements; the goal is the shape, not a prescribed mechanism
   (for example the phase name passed into the helper, or a cold helper
   called from it).
6. **Ordering and semantics.** The attachment still runs immediately
   after the consumer push and before any later push that could replace
   its insertion handle; the publication follows the attachment; the
   `EvaluateOnceUnstable` exception keeps publishing in the Profiled
   typed-push arm after its push; every early exit of the original body
   is preserved. Owner/epoch/control authentication, the exactly-once
   drain and the ownerless restore stay byte-identical. If the base
   pause's `complete_phase` on the DownValues key is not a no-op after
   the helper has taken the phase, the enabled battery decides whether
   the shape is admissible; a red records the exclusion.
7. **The acceptance bar.** The terminal branch executes the base
   instruction sequence at 50 and 100 iterations, in two independent
   CGU=1 builds and two draws, with the terminal Some/None ownership
   invariant stated and preserved: Some restores all prepared arguments
   and Unevaluated wrappers before normalization, None uses the final
   shared arguments, and the restoration plan is owned and dropped
   exactly once. If measurement shows no shape reaches it without
   violating the invariant, return a measured impossibility naming the
   exact opcode, branch, invariant, entailing statement, the excluded
   shapes and the smallest alternative. No universal impossibility
   beyond the measurement.
8. **Constraints.**
   - **Position.** The 386-observation cell trace must equal the 686
     control's in every case and every draw (`R, G, B` at the fixed
     point). The trace is the proof.
   - **Zero at every former site.** Every former publication call site
     and Guest typed insertion stays at the base instruction sequence,
     measured in the same run as the candidate.
   - **Identity.** The candidate2 ownership authentication, the
     exactly-once drain, the ownerless restore, the DownValues consumer
     frontier, the pause / suspend order, the decoded view, the
     canonical `Option<Expr>` decoding, the typed adapter, `LegacyMode`,
     the transport, the guest carriers and the eight definition
     finishes stay byte-identical. No test or fixture is weakened.
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
    operand sequences in two builds and two draws; the third stays a
    source-proved exclusion with its revalidation triggers restated.
13. **Cost.** Every former publication site zero versus base in the
    same run; the terminal branch at the base instruction sequence;
    every remaining disabled delta attributed to the queued path's own
    standing start/record calls or removed; the `builtin_catalog`
    lookup variation qualified by a layout control or a same-build
    repeated comparison. No netting of unrelated negative deltas.
14. **If Stage C is green** — do not run the downstream obligations
    (mispair, the 15 key routes, the contract fixtures) in this round;
    record the green cost form and draft the next round for them. If it
    is red, record the measured exclusion and stop.

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **The stability answer.** Whether the terminal count moves under an
   inert perturbation of the pause body, and whether the pause
   statements alone flip base's 54.
2. **The relocation answer.** A shape that moves the attachment and the
   publication to the suspension point and returns the terminal branch
   to the base sequence with the pause function byte-identical to base,
   or the exact measured impossibility.
3. **The preserved green.** The placement, position trace, enabled
   battery, exact stack, coverage witnesses and five watched falsifiers
   on the final shape.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the ripple inventory, risks and open
   questions.

## Context pointers

- `.temp/brief-692/report-design.json` — the whole-body exclusion, the
  phase-split answer, the stability-relevant symbol sizes and every
  signed row.
- `.temp/brief-692/terminal-relocation-answer.json` — the entailing
  sites, the invariant and the smallest next legal question.
- `.temp/brief-692/src/candidate2/` — the byte-identical placement
  form; `src/form/` — the 686 control; `src/whole_body/` and
  `src/whole_phase_first/` — the excluded 692 shapes.
- `.temp/brief-692/*-terminal-falsifier.json`,
  `*-falsifier-results.json`, `all-arm-stack-results.json`,
  `all-delta-rows.json`, `coverage/` and `costs/`.
- `.temp/director/brief-692-gate.md` — the stop and the rulings.
- `docs/dev/profiling.md` — the contract.
- Live sites: `profiling/logical/mod.rs`
  (`pause_phase_at_finish_slow`, `publish_stashed_phases_slow`,
  `take_phase`, `complete_phase`), `profiling/logical/queue_api.rs`
  (`logical_insert_down_values_before_consumer`),
  `profiling/logical/collector.rs`, `suspension/core/storage.rs`,
  `suspension/core/profile_queue.rs`, `eval_core/call.rs`
  (`suspend_resumed_call_stage_with_boundary`,
  `suspend_resumed_prepared_stage_with_boundary`,
  `suspend_resumed_evaluation_once_with_boundary`),
  `resolution.rs`, `eval_call_prepare.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-693/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-686/` … `.temp/brief-692/` or any other retained root,
  except to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing live.

## Hard bars

- **The disabled cost bar.** The default path pays only the queued
  path's own standing `runtime_path_timing_start` /
  `record_profile_runtime_path_timing` calls, measured in the same run.
  A publication-only gate is not allowed, even at +1, wherever it lives.
  A negative delta is not a waiver; a source-identical function is not a
  cost claim either.
- **The position bar.** The queue tail and every intermediate snapshot
  equal the 686 control's in all 20 trace cases, in both draws.
- **Publication semantics.** A publication may not be dropped,
  postponed past its owner, or merged into a path that loses an open
  record. The DownValues attachment must stay before its publication
  and before any push that can replace its insertion handle.
  Owner/epoch/control authentication, exactly-once completion and the
  ownerless restore stay stated with `path:line`.
- **The enabled form.** 20/20, `faults=0`, the unchanged battery and the
  exact 18,304/18,896 stack in two independent CGU=1 builds and two
  draws. No assertion or fixture is weakened.
- **The excluded forms.** The 683 observe-only guard, the 684
  position-only shape, the 685 ambient-owner guard, the panicking
  take-path fallback, both 687 cost shapes, the 688 owner-position
  placement, the 689 candidate1 shape, the 690 reload1 and reload2
  terminal shapes, the 691 `attach_helper` and `attach_publish` shapes,
  and the 692 `whole_body` and `whole_phase_first` shapes stay excluded.
  Every diagnosis cut is diagnosis only.
- **Every arm reduced.** Every candidate's stack and battery runs are
  reduced into the persisted artifact; a run that exists only as a log
  is not evidence. No row dropped, no negative netted.
- Every count, key, route and `path:line` is re-derived from the
  candidate2 basis; a brief-692 number is a claim, never evidence.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit; two independent builds
and draws for every cost row; serial Callgrind, CPU-pinned, release,
every `RUM_PROFILE_*` cleared except the declared modes; four stack
heights twice each with the residual printed. A timeout is not data.
Persist raw output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-693/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-693/report-design.json` in `rum-report/1`, with the
  stability and minimal-form answers, the candidate evidence (or the
  exact impossibility), the re-watched falsifiers, the green
  enabled/stack battery and every delta row.
- `.temp/brief-693/drafts/implementation-lp08-residual-17.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 693` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the measured
answers and the draft's bar list.
