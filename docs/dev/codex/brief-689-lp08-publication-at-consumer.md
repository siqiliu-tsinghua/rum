# Brief 689 — LP08 residual: publication appended at the consumer

**Cost design on the brief-686 form: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. The 686 form is the
enabled-green, exact-stack control; briefs 687's two cost forms and
brief 688's owner-position form are excluded.**

Brief 688 measured the queue position exactly: the 686 site
publication **appends at the back**, after the evaluation-return and
continuation consumer cells (`R, G, B`), and publishing at the
owner-establishment point moves the boundary in front of them
(`B, R, G`), closing `evaluate.once_unstable` one invocation early at
`none-two` pause 2. The position is a trace fact; it is not derivable
from the insertion call's arguments.

This round answers one question with measured evidence: **can every
site-level publish call be removed and publication appended
post-consumer inside the two enabled-only owner branches base already
executes — the pause's InputRequired slow path (already behind the
lexical gate) and the typed-frame push's existing `Profiled` arm —
so that the 686 `R, G, B` tail is reproduced in every trace case, the
enabled battery stays 20/20, the exact stack stays 18,304 / 18,896
with residual 0, and every former site is zero delta versus base?**

## Stage A — calibration

1. **Control.** Reproduce the 686 control before judging any
   placement: the base instruments (22/22 keys, nine native
   differences, four mirrors, boundary groups 9/2/6), the raw 19/20
   frontier, the 686 form's 20/20 pairs (`none-two` none=2, `down`
   some=1, `faults=0`), the exact 18,304 / 18,896 stack, the +9/call
   fixed-point cost and the `movq:+1` terminal reload. The 686 form
   tree must be reproduced byte-identically as `src/form`; a prior
   round's number is a claim, not evidence.
2. **The position and owner map.** Re-derive from the 686 tree, with
   `path:line`, and keep the 688 cell trace as the instrument:
   - every push that can place a cell between the stash and the
     consumer (`insert_evaluation_return`, `insert_typed_suspension_frame`,
     `FrameStorage::push_back_typed`, `push_evaluation_return`,
     `push_front*`, the queue hook paths), and which of them run
     between a pause and the owner's consumer at each site;
   - every pause that stashes and whether the owner's queue already
     holds the consumer at that moment;
   - the existing `Guest`/`Profiled` match in `FrameStorage` and the
     `lexical_profile_enabled` gate in `complete_phase` /
     `pause_phase_at_finish_slow`: these are the only branches a new
     publication may rely on for a free disabled path;
   - `collector::drain_attached` / `attach_boundary`: they append at
     the back today, and their callers are the phase-completion slow
     path only.
   Do not inherit 688's list; re-derive it.
3. **The baseline.** Persist the 686 form's per-address cost panel
   (two builds, two draws) and its 386 cell-trace snapshots as the
   control for every placement.

## Stage B — the placement

4. **Design.** Remove the six site-level `publish_stashed_phases`
   calls (`resolution.rs:569,587`; `eval_core/call.rs:42,66,95,124` in
   the 686 form). Publication happens at exactly two points, each
   inside a branch base already executes:
   - **Pause with consumer present.** In the pause's enabled-only
     InputRequired slow path, when the owner's queue already holds
     the consumer cell, append the paused boundary at the back now
     (the `drain_attached`/`attach_boundary` mechanism if it fits).
   - **Pause before its consumer.** In `FrameStorage::push_back_typed`'s
     existing `Profiled` arm, after a successful push, drain any
     stashed boundary whose owner matches, appending at the back.
     This is where the fixed-point pause's boundary is attached after
     its evaluation-return and continuation consumer cells.
   The exact discriminator, the ordering between the two points, and
   the memory-owner accounting are yours to design and prove.
5. **Constraints.**
   - **Position.** The queue tail must equal the 686 control's in
     every one of the 20 trace cases (`R, G, B` at the fixed point),
     proved by the retained cell trace, not by argument.
   - **Exactly once.** A boundary attached at the pause may not be
     attached again at the push, and vice versa.
   - **No loss.** An ownerless stash stays stashed in order; a
     foreign-control boundary is not attached; a boundary is never
     published after its owner is gone.
   - **Identity.** The 686 decoded view, the canonical `Option<Expr>`
     decoding, the typed adapter, `LegacyMode`, the transport, every
     guest carrier and the eight definition finishes stay
     byte-identical. The DownValues consumer-frontier exception, the
     pause/suspend order at every site and the deferred definition
     flush stay unchanged.
   - **No hidden state.** No new evaluator-frame field, no session
     field carrying diagnostic state, no selector, no outlined
     adapter, no +64 publish, no new `unsafe`.
   - **Zero disabled cost.** Every former site executes the base
     instruction sequence; a publication moved into a callee or into
     a fresh branch is not a waiver. The `Guest` arm, the ordinary
     mode, and every default-mode path are unchanged.
6. **Candidates.** At most two placements, each starting from the
   byte-identical 686 form and each preserving the position and the
   pause order. For each:
   - first re-run the full enabled battery and the 386-snapshot cell
     trace, then the exact stack;
   - only then measure the per-address panel with two independent
     CGU=1 builds and two draws, serial, CPU-pinned, release, every
     `RUM_PROFILE_*` cleared;
   - record every delta row, including regressions and negative
     values; reduce every arm into the persisted artifact.
   A candidate red on the enabled battery, the position trace or the
   stack is recorded and stopped; do not cost-measure it.
7. **Falsifiers.** Ship, and watch fire, one mutant per rule:
   - **position:** attach before the consumer push in the fixed-point
     shape — the cell trace must go red with `B, R, G`;
   - **owner-loss:** skip the attachment and the ownerless fixture
     must show the boundary unobserved or faulted;
   - **premature / wrong-owner:** a boundary with a foreign control
     must stay stashed until its own owner exists;
   - **exactly-once:** attach at both points and the record must
     double or fault.
   Each carries its exact counter or trace assertion and its observed
   red.
8. **The +1 reload row.** Remove the terminal argument branch's extra
   `movq` (restoration pointer loaded twice) with the same rebuild and
   prove it, or keep it as a measured open row naming the branch, the
   invariant and the opcode. Do not waive it as layout.

## Stage C — acceptance on the green cost form

9. **Enabled.** Queue 9/9 with parser quiescence `(0,0,0,0,0)` and
   strong-count ownership equality; slot 2/2; record 6/6; direct
   definition flush `Outcome::Some`, `faults=0`, counter
   `[1,1,0,1,0,0,0,0,0,0]`, quiescent; six lifecycle actions
   `open=0 leases=0 fabricated_ok=0`; all 20 pairs `native_same=true`
   `faults=0`; mirrors 67/67; seven Throw counter-only kills; all 150
   key rows with every prior PASS restored. Assertions and probes stay
   byte-identical; a red stays red.
10. **Stack.** Function 18,304 / Module 18,896 B per level, residual 0,
    two independent CGU=1 builds and two draws, four heights twice
    each, on the same form that passed the battery.
11. **Cost.** Every former publication site is zero versus base in the
    same run. Every remaining disabled delta is attributed per site to
    the queued path's own start/record calls, or removed. The
    `builtin_catalog` lookup variation is qualified by a layout control
    or a same-build repeated comparison. No netting of unrelated
    negative deltas against positive cost.
12. **If Stage C is green** — do not run Stage C's downstream
    obligations (mispair, the 15 key routes, the contract fixtures) in
    this round; record the green cost form and draft the next round
    for them. If it is red, record the measured exclusion and stop.

## Goal

When this round is done, the design record states, with measurement
and not argument:

1. **The placement answer.** Publication appended post-consumer inside
   the pause's slow path and the typed-frame push's `Profiled` arm,
   every former site at zero delta versus base, the 686 tail in every
   trace case; or a measured STOP naming the site, the invariant and
   the smallest correct alternative.
2. **The control.** The 686 enabled 20/20, exact stack and cost panel
   reproduced before any candidate is judged.
3. **The candidates.** At most two, each with its enabled/position/
   stack/cost evidence, every arm reduced, no row dropped, no negative
   netted.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the placement, its invariants, the gate
   tests and the re-watched mutants, the ripple inventory, risks and
   open questions.

## Context pointers

- `.temp/brief-688/report-design.json`, sections `Measured placement
  answer`, `Position proof and observation-order failure`, `Four
  watched placement mutants`, `Every signed control delta row`;
  `cell-trace-results.json`, `candidate1-build1-complete-battery.json`,
  `stack-results.json`, `written-file-inventory.json`.
- `.temp/brief-687/report-design.json` — the two cost forms and the
  fused pause technique that measured zero at the call sites.
- `.temp/brief-687/src/form/` — the byte-identical 686 control.
  `.temp/brief-688/src/form/` — the same control reproduced this round.
- `.temp/director/brief-688-gate.md` — the position STOP and the
  post-consumer ruling.
- `docs/dev/profiling.md` — the contract.
- Live sites: `suspension/core/storage.rs` (`push_back_typed`, the
  `Guest`/`Profiled` match), `suspension/core.rs`
  (`insert_typed_erased_frame`, `begin_input_suspension`),
  `profiling/logical/mod.rs` (`complete_phase`,
  `pause_phase_at_finish_slow`, `publish_stashed_phases`),
  `profiling/logical/collector.rs` (`drain_attached`, `attach_boundary`,
  `stash_boundary`), `profiling/logical/queue_api.rs`,
  `suspension/core/profile_queue.rs`, `suspension/typed/witness.rs`,
  `resolution.rs`, `eval_core/call.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-689/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control
  command;
- `.temp/brief-686/`, `.temp/brief-687/`, `.temp/brief-688/` or any
  other retained root, except to read and copy. Never execute a file
  in place;
- the public API, features, serde, manifests, dependencies,
  `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs
and probes; it repairs nothing.

## Hard bars

- **The disabled cost bar.** The default path pays only the queued
  path's own standing `runtime_path_timing_start` /
  `record_profile_runtime_path_timing` calls, measured in the same
  run. A publication-only gate is not allowed, even at +1, wherever it
  lives. A negative delta is not a waiver.
- **The position bar.** The queue tail and every intermediate snapshot
  equal the 686 control's in all 20 trace cases. A boundary may not
  enter before a cell it follows today.
- **Publication semantics.** A publication may not be dropped,
  postponed past its owner, or merged into a path that loses an open
  record. Owner/epoch/control authentication, exactly-once completion
  and the ownerless restore stay stated with `path:line`.
- **The enabled form.** 20/20, `faults=0`, the unchanged battery and
  the exact 18,304/18,896 stack in two independent CGU=1 builds and
  two draws. No assertion or fixture is weakened.
- **The excluded forms.** The 683 observe-only guard, the 684
  position-only shape, the 685 ambient-owner guard, the panicking
  take-path fallback, both 687 cost shapes and the 688 owner-position
  placement stay excluded.
- **No hidden state.** No new evaluator-frame field, no session field
  carrying diagnostic state, no selector, no outlined adapter, no +64
  publish, no new `unsafe`. The decoded `Box`/`Expr` copy stays
  enabled-only and charged.
- Every count, key, route and `path:line` is re-derived from the 686
  tree; a brief-687/688 number is a claim, never evidence.

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

All scratch under `.temp/brief-689/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-689/report-design.json` in `rum-report/1`, with the
  placement answer, the candidate evidence (or the measured
  impossibility), the re-watched falsifiers, the green
  enabled/stack battery and every delta row.
- `.temp/brief-689/drafts/implementation-lp08-residual-13.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 689` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
