# Brief 695 — LP08 residual: the keyed completion interior

**Cost design on the brief-694 candidate2 basis: `Scope: NONE — no live
source, workload, manifest, inventory or version-control change.`
Disposable build probes only, under this brief's root. Candidate2 is the
byte-identical placement form measured in briefs 689–694; the 686 form
stays the enabled/stack control.**

Brief 694 excluded both of its shapes. Shape 1 was excluded before build
because the `InputRequired` arm of `complete_phase` is not pause-only:
`record_profile_runtime_path_timing_adopted` (`timing.rs:599–630`) passes
`T::timing_outcome(result)` to `complete_phase`, and that outcome is
`InputRequired` for `Err(EvalError::InputRequired)`; the adopted
completion in `definitions/application/rhs.rs:55–66` calls it before
matching the very InputRequired it will propagate. Its defer-pause
sibling (`timing.rs:637–665`) is the variant with the guard. Shape 2 —
the one-completion suspension relocation — was enabled-green but
measured terminal 56 against base 54 (two extra `movq`), an unmatched
helper `Result` copy of 4 self instructions per call with no removed
caller copy, and stack 18,256/18,848 against the exact 18,304/18,896.

What remained unmeasured is a form that keeps the base pause slow body
while making the publication pause-only **by the identity of the phase
being completed**. Candidate2 already branches on that identity: its
pause body distinguishes `PhaseKey::CallStageDownValues` and skips
`PhaseKey::EvaluateOnceUnstable`. The call-stage phase tokens are created
only in the two dispatch-plan resume loops (`eval_core/call.rs:230,353`),
and there they reach the defer-pause completion (`:279`, which skips
`InputRequired`), the Error completion (`:257,380`), the None completion
(`:275,400`), a bare timing record (`:270`), and the suspend helpers
(which only record) — never an adopted `InputRequired` completion. That
makes a keyed arm a distinct shape from 694's unconditional one.

This round answers one question with measured evidence: **can the
DownValues attachment and the non-fixed-point publication live inside
`complete_phase`'s `InputRequired` arm, keyed to the eight
`PhaseKey::CallStage*` variants — the same identity the accepted
candidate2 pause body already branches on — leaving
`pause_phase_at_finish_slow` byte-identical to the base source and every
call site and suspension helper untouched, with exactly one phase
completion, no new gate, no new state, and the terminal branch on the
base sequence; or is that measured impossible?**

## Stage A — calibration and the phase-key reachability

1. **Control and basis.** Reproduce the 694 control before judging any
   candidate: base 54 with one `0x328(%rsp)` load; form and candidate2 55
   with two; the excluded `suspension_single_completion` at 56; the 20
   cost cells in two builds and two draws; the stability pair
   (`inert_stability` 55, `minimal_support` 54, `minimal_pause` 55) and
   the 690/691/692/693 diagnosis cuts; the 686 form's 20/20 pairs and
   67/67 mirrors; the 150 key rows (131 PASS / 19 OPEN); the seven Throw
   kills; the 386-snapshot trace; the exact 18,304 / 18,896 stack; the 94
   and 96 resumed-helper witnesses; the five watched control falsifiers.
   Reproduce candidate2 byte-identically from `.temp/brief-694/src/candidate2/`;
   every 694 number is a claim, not evidence.
2. **The phase-key reachability derivation.** This is the shape's
   semantic precondition and it must be exhaustive, not illustrative.
   - Enumerate every `complete_phase` call site with `path:line` and the
     outcome it can pass, as brief 694 did.
   - For each site that can pass `InputRequired`, enumerate the **phase
     keys** it can pass: every `runtime_path_timing_start` /
     `runtime_path_timing_start_adopted` call, every phase literal and
     every `profile_name()` source, and the `PhaseKey::from_name` mapping
     for each. Cover all 50 `PhaseKey::ALL` entries and say which have no
     token source.
   - Show that `InputRequired` with any of the eight `PhaseKey::CallStage*`
     keys can originate only at the six pause entry points
     (`eval_core/call.rs:253,311,376,436` via the pause, and
     `resolution.rs:542,562`, which pass `evaluate.once_unstable`). Show
     that the two dispatch-plan loops pass their stage token only to the
     defer-pause completion, the Error completion, the None completion, a
     bare timing record and the suspend helpers — and that the suspend
     helpers call only the bare timing record.
   - Show the same for the other `InputRequired` caller:
     `record_profile_runtime_path_timing_adopted_slow` receives tokens
     from the non-call-stage literal sites and from
     `head_kind.profile_name()` / `stage.profile_name()` only in the
     places listed above.
   - A call-stage key reaching `complete_phase` with `InputRequired` from
     any non-pause site invalidates shape 1 before it is built. The
     derivation's verdict line names the counterexample or states
     `pause_only_by_key=true`.
   - Show the disabled reachability: `pause_phase_at_finish` gates before
     the slow call, and every other `complete_phase` caller is behind a
     `lexical_profile_enabled` gate or a token that can only exist with
     profiling on. A caller reachable with profiling off invalidates the
     disabled-cost argument.
3. **The retained green.** Persist the control's per-address cost panels,
   cell traces and stack rows as the control for the candidate. The two
   resumed-helper witnesses and the prepared-helper source exclusion are
   re-stated from the retained roots; the final candidate re-measures the
   witnesses.

## Stage B — at most two cost candidates

4. **Shape 1, the keyed completion interior (primary).** Start from the
   byte-identical candidate2 basis. Restore
   `pause_phase_at_finish_slow` to the byte-identical base source
   (`PhaseKey::from_name` plus `complete_phase(session, key,
   InputRequired, |_| {})`, exactly as `src/base` has it). In
   `complete_phase`'s `InputRequired` arm, keep the candidate2 stash for
   every key and add, keyed to the call-stage variants:
   - for `PhaseKey::CallStageDownValues`: `call.pause()`, then
     `session.logical_insert_down_values_before_consumer(boundary)` — the
     attachment, at the same point in the same order as the candidate2
     pause body;
   - for the other seven `PhaseKey::CallStage*` keys: `call.pause()`,
     then `collector::stash_boundary(boundary)`;
   - for every other key (including `EvaluateOnceUnstable`): the
     candidate2 stash and **no publication**;
   - after the attachment or stash, publish the stashed phases
     (`publish_stashed_phases_slow`) for the call-stage keys only, never
     for any other key.
   The six pause call sites, the four suspend helpers, the queue, the
   collector and every other file stay byte-identical. The worker chooses
   the mechanism inside the arm (inline or a cold helper called from it);
   the goal is the shape — a base-source pause slow body, one completion,
   and the publication keyed to the phase identity. Changing the
   `complete_phase` signature, adding a parameter, or carrying state is
   not this shape.
5. **Shape 2, only if shape 1 fails.** The entry split: bind the four
   `eval_core/call.rs` call sites to a new pause entry with the same
   inline-gate shape whose slow body is the candidate2 pause body, and
   keep `pause_phase_at_finish` byte-identical for the two
   `resolution.rs` `evaluate.once_unstable` sites. The call sites then
   differ from base only in the callee name. This distinguishes whether
   the terminal delta follows the body of the function the call site
   reaches or the identity of `pause_phase_at_finish_slow` itself. Its
   disabled cost is a same-statement rename, not a new gate: account for
   the call-site rows row by row; a call setup the base site did not have
   is a red, not an offset.
6. **Ordering and semantics.** The attachment still runs immediately
   after the consumer push and before any later push that could replace
   its insertion handle; the publication follows the attachment or the
   stash; `EvaluateOnceUnstable` never publishes at completion; every
   early exit of the candidate2 pause body is preserved; exactly one
   phase completion per phase and a second `take_phase` of the same phase
   is a red, not a harmless no-op. Owner/epoch/control authentication,
   the exactly-once drain and the ownerless restore stay byte-identical.
   The enabled battery and the position trace decide.
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
   - **Zero at every former site.** The pause slow body is byte-identical
     to base and the changed arm is unreachable with profiling off: the
     disabled run must show the base sequence at every former site and
     zero instructions in the changed arm. A gate or a call added on the
     disabled path is not allowed, even at +1.
   - **Position.** The 386-observation cell trace must equal the 686
     control's in every case and every draw. The trace is the proof.
   - **Identity.** The candidate2 ownership authentication, the
     exactly-once drain, the ownerless restore, the DownValues consumer
     frontier, the pause / suspend order, the decoded view, the canonical
     `Option<Expr>` decoding, the typed adapter, `LegacyMode`, the
     transport, the guest carriers and the eight definition finishes stay
     byte-identical. No test or fixture is weakened.
   - **No hidden state.** No new evaluator-frame field, no session field
     carrying diagnostic state, no selector in the earlier sense, no
     outlined guest adapter, no new `unsafe`, no `+64` publication frame.
     The phase-key match is authorized: it dispatches on the completed
     phase's identity, which candidate2's pause body already branches on,
     inside a cold function unreachable with profiling off. It is not a
     gate, a state carrier or an off-by-default feature.
   - **One variable per candidate.** At most two cost candidates, each
     starting from the byte-identical candidate2 basis; diagnosis arms
     are not candidates.
9. **Falsifiers.** Re-watch the five 692 falsifiers on the final measured
   shape: position (`R,G,B` → `B,R,G`), owner-loss, wrong-owner /
   premature, exactly-once, and the terminal invariant (dropped or
   duplicated Some restoration observed by its counter assertion, not
   merely compiling differently). Each carries its exact counter or trace
   assertion and its observed red.

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
    operand sequences in two builds and two draws — the helpers are
    untouched in shape 1. The prepared helper stays a source-proved
    exclusion with its revalidation triggers restated.
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

1. **The keyed reachability answer.** Every `complete_phase` caller, the
   outcome and the phase-key set it can pass, with `path:line`, and
   whether `InputRequired` with a call-stage key is pause-only and
   disabled-unreachable.
2. **The relocation answer.** A shape with a base-source pause slow body
   and a keyed completion interior that returns the terminal branch to
   the base sequence with one completion and no new default-path
   instruction, or the exact measured impossibility.
3. **The preserved green.** The position trace, enabled battery, exact
   stack, coverage witnesses and five watched falsifiers on the final
   shape.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the ripple inventory, risks and open
   questions.

## Context pointers

- `.temp/brief-694/report-design.json` — the shape-1 exclusion, the
  measured shape-2 reds and every signed row.
- `.temp/brief-694/src/candidate2/` — the byte-identical placement form;
  `src/form/` — the 686 control; `src/suspension_single_completion/` —
  the excluded one-completion move; `src/minimal_support/`,
  `src/minimal_pause/`, `src/inert_stability/` — the stability pair.
- `.temp/brief-694/move-accounting-addresses.json`,
  `all-delta-rows.json`, `all-arm-stack-results.json`,
  `candidate2-cell-trace-results.json`, `costs/`, `diagnosis/`,
  `coverage/` and `runs/`.
- `.temp/brief-693/report-design.json` — the stability pair and the
  duplicate-pause exclusion; `.temp/brief-693/src/suspension_local_phase/`
  — the 54-but-duplicate shape whose `complete_phase` arm match is the
  cost basis for shape 1.
- `.temp/director/brief-694-gate.md` — the stop and the rulings.
- `docs/dev/profiling.md` — the contract.
- Live sites: `profiling/logical/mod.rs` (`complete_phase`,
  `pause_phase_at_finish(_slow)`, `publish_stashed_phases_slow`,
  `take_phase`, `PhaseKey`), `profiling/timing.rs` (both adopted
  variants and the defer-pause guard), `profiling/control_timing.rs`,
  `profiling/logical/queue_api.rs`
  (`logical_insert_down_values_before_consumer`),
  `profiling/logical/collector.rs`, `suspension/core/storage.rs`
  (`push_back_typed`), `suspension/core/profile_queue.rs`,
  `eval_core/call.rs`, `resolution.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-695/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-686/` … `.temp/brief-694/` or any other retained root,
  except to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing live.

## Hard bars

- **The disabled cost bar.** The default path pays only the queued
  path's own standing `runtime_path_timing_start` /
  `record_profile_runtime_path_timing` calls, measured in the same run.
  The pause slow body is byte-identical to base and the changed arm is
  unreachable with profiling off; a gate or a call added on the disabled
  path is not allowed, even at +1, wherever it lives. A source-identical
  function is not a cost claim by itself.
- **The position bar.** The queue tail and every intermediate snapshot
  equal the 686 control's in all 20 trace cases, in both draws.
- **Publication semantics.** A publication may not be dropped,
  postponed past its owner, or merged into a path that loses an open
  record. The DownValues attachment must stay before its publication and
  before any push that can replace its insertion handle. Exactly one
  phase completion; owner/epoch/control authentication, exactly-once
  completion and the ownerless restore stay stated with `path:line`. A
  publication that can fire for a non-call-stage key at an adopted
  completion is a red.
- **The enabled form.** 20/20, `faults=0`, the unchanged battery and the
  exact 18,304/18,896 stack in two independent CGU=1 builds and two
  draws. No assertion or fixture is weakened.
- **The excluded forms.** The 683 observe-only guard, the 684
  position-only shape, the 685 ambient-owner guard, the panicking
  take-path fallback, both 687 cost shapes, the 688 owner-position
  placement, the 689 candidate1 shape, the 690 reload1 and reload2
  terminal shapes, the 691 `attach_helper` and `attach_publish` shapes,
  the 692 `whole_body` and `whole_phase_first` shapes, the 693
  `suspension_local_phase` duplicate-pause shape and the 694
  `suspension_single_completion` one-completion move stay excluded.
  Every diagnosis cut is diagnosis only. The 694 unconditional
  completion-interior form stays excluded: shape 1 here is legal only
  because of the keyed reachability derivation, not because the 694
  counterexample was argued away.
- **Every arm reduced.** Every candidate's stack and battery runs are
  reduced into the persisted artifact; a run that exists only as a log
  is not evidence. No row dropped, no negative netted.
- Every count, key, route and `path:line` is re-derived from the
  candidate2 basis; a brief-694 number is a claim, never evidence.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit; two independent builds
and draws for every cost row; serial Callgrind, CPU-pinned, release,
every `RUM_PROFILE_*` cleared except the declared modes; four stack
heights twice each with the residual printed. A timeout is not data.
Persist raw output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-695/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-695/report-design.json` in `rum-report/1`, with the
  phase-key reachability derivation, the candidate evidence (or the exact
  impossibility), the re-watched falsifiers, the green enabled/stack
  battery and every delta row.
- `.temp/brief-695/drafts/implementation-lp08-residual-19.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 695` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the measured
answers and the draft's bar list.
