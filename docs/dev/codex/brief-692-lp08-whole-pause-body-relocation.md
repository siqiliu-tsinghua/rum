# Brief 692 — LP08 residual: the whole pause body out of the pause

**Cost design on the brief-691 candidate2 basis: `Scope: NONE — no live
source, workload, manifest, inventory or version-control change.`
Disposable build probes only, under this brief's root. Candidate2 is the
byte-identical placement form measured in briefs 689, 690 and 691; the
686 form stays the enabled/stack control.**

Brief 691 measured that the terminal reload has **two independent
triggers** in the cold `pause_phase_at_finish_slow` body. With every
publisher present, the attachment alone keeps 55 (`cut_downonly`), the
publication call alone keeps 55 (`cut_full_drop`), and only removing
both returns 54 (`cut_drop_consumer`). Both relocation shapes measured
55: `attach_helper` outlines only the attachment, and `attach_publish`
moves the DownValues branch into a cold helper but leaves the general
non-fixed-point `publish_stashed_phases_slow(session)` call in the pause
body. The worker's answer names the smallest next legal experiment:
move the remaining general publication out of the cold pause as well,
preserving attachment first, all early exits, the ownerless restore and
the next-push frontier.

This round answers one question with measured evidence: **can the
entire pause semantics move out of the pause function's direct code —
one cold implementation behind a thin dispatcher — so the terminal
branch returns to the base instruction sequence, with attachment,
publication, ordering and every invariant preserved; or is that
measured impossible?**

## Stage A — calibration

1. **Control and basis.** Reproduce the 691 control before judging any
   candidate: the base terminal 54 with one `0x328(%rsp)` load,
   candidate2/form/attach_helper/attach_publish 55 with two loads in two
   builds and two draws; the 12 diagnosis cells (`cut_downonly` 55,
   `cut_drop_consumer` 54, `cut_full_drop` 55); the 686 form's 20/20
   pairs and 67/67 mirrors; the 150 key rows (131 PASS / 19 OPEN); the
   seven Throw kills; the 386-snapshot-per-arm trace with the `R, G, B`
   tail; the exact 18,304 / 18,896 stack; the 94 and 96 resumed-helper
   witnesses. Reproduce candidate2 byte-identically from
   `.temp/brief-691/src/candidate2/`; every 691 number is a claim, not
   evidence.
2. **The isolating diagnosis arms.** On the byte-identical candidate2
   basis, split the single trailing publication by phase as diagnosis:
   one arm publishes only on the general non-fixed-point branch (the
   DownValues branch attaches without publishing), and, if the worker
   judges it necessary to pin the trigger, one arm publishes only on the
   DownValues branch (the general branch completes without publishing).
   Measure the terminal branch in two builds and two draws. Each arm is
   diagnosis only, may not be promoted, and its publication loss is
   stated as such.
3. **The retained green.** Persist the control's per-address cost
   panels, cell traces and stack rows as the control for every
   candidate. The two resumed-helper witnesses and the source exclusion
   are re-stated from the retained roots, not re-derived silently; the
   final candidate re-measures them.

## Stage B — at most two cost candidates

4. **The shape.** Each candidate starts from the byte-identical
   candidate2 basis and moves the whole pause semantics out of
   `pause_phase_at_finish_slow`'s direct code. The natural first shape
   is the pause function reduced to one `#[cold] #[inline(never)]`
   implementation call (or tail-jump) containing the current body
   verbatim: phase lookup, take, pause, attachment, complete and the
   general publication, with the early exits preserved. The worker
   chooses the second shape; the goal is the shape, not a prescribed
   mechanism. No publication or attachment call may remain in the pause
   function's direct code.
5. **The acceptance bar.** The terminal branch executes the base
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
6. **Constraints.**
   - **Position.** The 386-observation cell trace must equal the 686
     control's in every case and every draw (`R, G, B` at the fixed
     point). The trace is the proof.
   - **Zero at every former site.** Every former publication call site
     and Guest typed insertion stays at the base instruction sequence,
     measured in the same run as the candidate.
   - **Identity.** The candidate2 ownership authentication
     (root/epoch/control/generation), the exactly-once drain, the
     ownerless restore, the DownValues consumer frontier, the pause /
     suspend order, the decoded view, the canonical `Option<Expr>`
     decoding, the typed adapter, `LegacyMode`, the transport, the guest
     carriers and the eight definition finishes stay byte-identical. No
     test or fixture is weakened.
   - **Attachment ordering.** The attachment still runs before its
     publication and before any push that could replace its insertion
     handle; all early exits (`take_phase` None, no call, no boundary)
     keep their exact behaviour.
   - **No hidden state.** No new evaluator-frame field, no session field
     carrying diagnostic state, no selector, no outlined guest adapter,
     no new `unsafe`, no `+64` publication frame.
   - **One variable per candidate.** At most two cost candidates, each
     starting from the byte-identical candidate2 basis; diagnosis arms
     are not candidates.
7. **Falsifiers.** Re-watch the five 691 falsifiers on the final
   measured shape: position (`R,G,B` → `B,R,G`), owner-loss,
   wrong-owner / premature, exactly-once, and the terminal invariant
   (dropped or duplicated Some restoration observed by its counter
   assertion, not merely compiling differently). Each carries its exact
   counter or trace assertion and its observed red.

## Stage C — acceptance on the green cost form

8. **Enabled.** Queue 9/9 with parser quiescence `(0,0,0,0,0)` and
   strong-count ownership equality; slot 2/2; record 6/6; direct
   definition flush `Outcome::Some`, `faults=0`, counter
   `[1,1,0,1,0,0,0,0,0,0]`, quiescent; six lifecycle actions
   `open=0 leases=0 fabricated_ok=0`; all 20 pairs `native_same=true`
   `faults=0`; mirrors 67/67; seven Throw counter-only kills; all 150
   key rows with every prior PASS restored.
9. **Stack.** Function 18,304 / Module 18,896 B per level, residual 0,
   two independent CGU=1 builds and two draws, four heights twice each,
   on the same candidate that passed the battery.
10. **Coverage.** The two resumed-helper witnesses measure at the
    base-identical 94 and 96 self instructions with equal opcode and
    operand sequences in two builds and two draws; the third stays a
    source-proved exclusion with its revalidation triggers restated.
11. **Cost.** Every former publication site zero versus base in the
    same run; the terminal branch at the base instruction sequence;
    every remaining disabled delta attributed to the queued path's own
    standing start/record calls or removed; the `builtin_catalog`
    lookup variation qualified by a layout control or a same-build
    repeated comparison. No netting of unrelated negative deltas.
12. **If Stage C is green** — do not run the downstream obligations
    (mispair, the 15 key routes, the contract fixtures) in this round;
    record the green cost form and draft the next round for them. If it
    is red, record the measured exclusion and stop.

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **The isolated trigger.** Which pause-body publication occurrence
   keeps the terminal reload, measured on the full basis.
2. **The relocation answer.** A shape that moves the whole pause
   semantics out of the pause function's direct code and returns the
   terminal branch to the base sequence, or the exact measured
   impossibility.
3. **The preserved green.** The placement, position trace, enabled
   battery, exact stack, coverage witnesses and five watched falsifiers
   on the final shape.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the ripple inventory, risks and open
   questions.

## Context pointers

- `.temp/brief-691/report-design.json` — the two-trigger cause, the two
  excluded shapes and the smallest next legal experiment.
- `.temp/brief-691/terminal-relocation-answer.json` — the entailing
  statement, the invariant and the worker's named next experiment.
- `.temp/brief-691/full-basis-cause.json`,
  `diagnosis/` — the 12 diagnosis cells.
- `.temp/brief-691/src/candidate2/` — the byte-identical placement
  form; `src/form/` — the 686 control; `src/attach_helper/` and
  `src/attach_publish/` — the excluded 691 shapes.
- `.temp/brief-691/*-terminal-falsifier.json`,
  `*-falsifier-results.json`, `all-arm-stack-results.json`,
  `all-delta-rows.json`, `all-publication-terminal-sequences.json`,
  `coverage/` and `costs/`.
- `.temp/director/brief-691-gate.md` — the stop and the rulings.
- `docs/dev/profiling.md` — the contract.
- Live sites: `profiling/logical/mod.rs`
  (`pause_phase_at_finish_slow`, `publish_stashed_phases_slow`,
  `take_phase`, `complete_phase`), `profiling/logical/queue_api.rs`
  (`logical_insert_down_values_before_consumer`),
  `profiling/logical/collector.rs`, `suspension/core/storage.rs`,
  `suspension/core/profile_queue.rs`, `eval_core/call.rs`,
  `resolution.rs`, `eval_call_prepare.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-692/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-686/` … `.temp/brief-691/` or any other retained root,
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
  terminal shapes, and the 691 `attach_helper` and `attach_publish`
  shapes stay excluded. The 690/691 diagnosis cuts are not candidates.
- **Every arm reduced.** Every candidate's stack and battery runs are
  reduced into the persisted artifact; a run that exists only as a log
  is not evidence. No row dropped, no negative netted.
- Every count, key, route and `path:line` is re-derived from the
  candidate2 basis; a brief-691 number is a claim, never evidence.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit; two independent builds
and draws for every cost row; serial Callgrind, CPU-pinned, release,
every `RUM_PROFILE_*` cleared except the declared modes; four stack
heights twice each with the residual printed. A timeout is not data.
Persist raw output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-692/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-692/report-design.json` in `rum-report/1`, with the
  isolating diagnosis answer, the candidate evidence (or the exact
  impossibility), the re-watched falsifiers, the green enabled/stack
  battery and every delta row.
- `.temp/brief-692/drafts/implementation-lp08-residual-16.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 692` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the measured
answers and the draft's bar list.
