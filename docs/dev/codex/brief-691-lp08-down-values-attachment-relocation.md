# Brief 691 — LP08 residual: relocate the DownValues attachment

**Cost design on the brief-690 candidate2 basis: `Scope: NONE — no live
source, workload, manifest, inventory or version-control change.`
Disposable build probes only, under this brief's root. Candidate2 is the
byte-identical placement form measured in briefs 689 and 690; the 686
form stays the enabled/stack control.**

Brief 690 measured the cause of the terminal restoration reload. On the
candidate2-derived down-only cut, replacing the single statement
`session.logical_insert_down_values_before_consumer(boundary)` with
`drop(boundary)` in the cold `pause_phase_at_finish_slow` body flips the
terminal branch from 55 to 54 in two independent builds and two draws,
with the cold pause executing zero times. The cut drops required
publication and is diagnosis only, and it was measured with the
publication call absent, so it does not yet prove the same statement is
sufficient with every publisher present. Both terminal-side shapes are
excluded: reload1 (loop restructure) stays at 55; reload2
(`match unevaluated_restore.take()`) measures 48 but still loads the
restoration slot twice and changes the opcode and operand sequences.
Coverage is settled by brief 690: two resumed helpers are dynamically
witnessed at base-identical 94 and 96 self instructions, and the third
is a source-proved exclusion.

This round answers one question with measured evidence: **can the
authenticated DownValues attachment move out of the cold pause body so
the terminal branch returns to the base instruction sequence, with the
attachment, its ordering and every publication and position invariant
preserved — or is that measured impossible?**

## Stage A — calibration

1. **Control and basis.** Reproduce the 690 control before judging any
   candidate: the base instruments (22/22 keys, nine native differences,
   four mirrors, boundary groups 9/2/6), the raw 19/20 frontier, the 686
   form's 20/20 pairs and 67/67 mirrors, the 193-snapshot-per-draw trace
   with the `R, G, B` tail, the exact 18,304 / 18,896 stack, and the
   terminal 54 (base) / 55 (form, candidate2) / 55 and 54 (the two
   diagnosis cuts) in two builds and two draws. Reproduce candidate2
   byte-identically from `.temp/brief-690/src/candidate2/`; every 690
   number is a claim, not evidence.
2. **The full-basis cause arm.** On the byte-identical candidate2 basis,
   with its publication call present, replace only the attachment
   statement with `drop(boundary)` as a diagnosis arm and measure the
   terminal branch in two builds and two draws. If it measures 54, the
   attachment is the cause with every publisher present. If it stays 55,
   publication contributes a second cause; state that before building
   any candidate. The arm is diagnosis only and may not be promoted.
3. **The retained green.** Persist the control's and candidate2's
   per-address cost panels, cell traces and stack rows as the control
   for every candidate. The two resumed-helper witnesses and the source
   exclusion are re-stated from the retained roots, not re-derived
   silently; the final candidate re-measures them.

## Stage B — at most two cost candidates

4. **The shape.** Each candidate starts from the byte-identical
   candidate2 basis and relocates the DownValues attachment out of the
   cold `pause_phase_at_finish_slow` body. The worker chooses the two
   placements; the goal is the shape, not a prescribed mechanism. The
   attachment must still run on the same evaluation path, before the
   publication of its stashed boundary and before any push that could
   replace its insertion handle, and must keep the same owner, epoch,
   control, generation and exactly-once authentication.
5. **The acceptance bar.** The terminal branch executes the base
   instruction sequence at 50 and 100 iterations, in two independent
   CGU=1 builds and two draws, with the terminal Some/None ownership
   invariant stated and preserved: Some restores all prepared arguments
   and Unevaluated wrappers before normalization, None uses the final
   shared arguments, and the restoration plan is owned and dropped
   exactly once. If measurement shows no relocated shape reaches it
   without violating the invariant, return a measured impossibility
   naming the exact opcode, branch, invariant, entailing statement, the
   two excluded shapes and the smallest alternative. No universal
   impossibility beyond the measurement.
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
   - **No hidden state.** No new evaluator-frame field, no session field
     carrying diagnostic state, no selector, no outlined guest adapter,
     no new `unsafe`, no `+64` publication frame.
   - **One variable per candidate.** At most two cost candidates, each
     starting from the byte-identical candidate2 basis; diagnosis arms
     are not candidates.
7. **Falsifiers.** Re-watch the five 690 falsifiers on the final measured
   shape: position (`R,G,B` → `B,R,G`), owner-loss, wrong-owner /
   premature, exactly-once, and the terminal invariant (dropped or
   duplicated Some restoration observed by its counter assertion, not
   merely compiling differently). Each carries its exact counter or
   trace assertion and its observed red.

## Stage C — acceptance on the green cost form

8. **Enabled.** Queue 9/9 with parser quiescence `(0,0,0,0,0)` and
   strong-count ownership equality; slot 2/2; record 6/6; direct
   definition flush `Outcome::Some`, `faults=0`, counter
   `[1,1,0,1,0,0,0,0,0,0]`, quiescent; six lifecycle actions
   `open=0 leases=0 fabricated_ok=0`; all 20 pairs `native_same=true
   faults=0`; mirrors 67/67; seven Throw counter-only kills; all 150 key
   rows with every prior PASS restored.
9. **Stack.** Function 18,304 / Module 18,896 B per level, residual 0,
   two independent CGU=1 builds and two draws, four heights twice each,
   on the same candidate that passed the battery.
10. **Coverage.** The two resumed-helper witnesses measure at the
    base-identical 94 and 96 self instructions with equal opcode and
    operand sequences in two builds and two draws; the third stays a
    source-proved exclusion with its revalidation triggers restated.
11. **Cost.** Every former publication site zero versus base in the same
    run; the terminal branch at the base instruction sequence; every
    remaining disabled delta attributed to the queued path's own
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

1. **The full-basis cause.** Whether the attachment statement alone
   accounts for the terminal reload with every publisher present.
2. **The relocation answer.** A shape that moves the attachment out of
   the cold pause body and returns the terminal branch to the base
   sequence, or the exact measured impossibility.
3. **The preserved green.** The placement, position trace, enabled
   battery, exact stack, coverage witnesses and five watched falsifiers
   on the final shape.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the ripple inventory, risks and open
   questions.

## Context pointers

- `.temp/brief-690/report-design.json` — the terminal cause, the two
  excluded shapes, the coverage answer and every signed row.
- `.temp/brief-690/terminal-reload-cause.json` — the four diagnosis
  cells and the paired `0x328(%rsp)` loads.
- `.temp/brief-690/minimal-cause.patch`,
  `.temp/brief-690/cut_downonly.patch`,
  `.temp/brief-690/cut_drop_consumer.patch` and
  `.temp/brief-690/cut_pause.patch` — the diagnosis arms; copy, never
  execute in place.
- `.temp/brief-690/src/candidate2/` — the byte-identical placement form;
  `.temp/brief-690/src/form/` — the 686 control.
- `.temp/brief-690/disabled-suspension-map.md` and
  `disabled-helper-exclusion.json` — the resumed-helper witnesses and
  the source proof.
- `.temp/brief-690/reload{1,2}-falsifier-results.json`,
  `candidate2-falsifier-results.json`,
  `*-terminal-falsifier.json`, `all-arm-stack-results.json`,
  `all-delta-rows.json`, `all-publication-terminal-sequences.json`.
- `.temp/director/brief-690-gate.md` — the stop and the rulings.
- `docs/dev/profiling.md` — the contract.
- Live sites: `profiling/logical/mod.rs`
  (`pause_phase_at_finish_slow`, `publish_stashed_phases_slow`),
  `profiling/logical/queue_api.rs`
  (`logical_insert_down_values_before_consumer`),
  `profiling/logical/collector.rs`, `suspension/core/storage.rs`,
  `suspension/core/profile_queue.rs`, `eval_core/call.rs`,
  `resolution.rs`, `eval_call_prepare.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-691/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-686/`, `.temp/brief-687/`, `.temp/brief-688/`,
  `.temp/brief-689/`, `.temp/brief-690/` or any other retained root,
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
  placement, the 689 candidate1 shape, and the 690 reload1 and reload2
  terminal shapes stay excluded. The 690 diagnosis cuts are not
  candidates.
- **Every arm reduced.** Every candidate's stack and battery runs are
  reduced into the persisted artifact; a run that exists only as a log
  is not evidence. No row dropped, no negative netted.
- Every count, key, route and `path:line` is re-derived from the
  candidate2 basis; a brief-690 number is a claim, never evidence.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit; two independent builds
and draws for every cost row; serial Callgrind, CPU-pinned, release,
every `RUM_PROFILE_*` cleared except the declared modes; four stack
heights twice each with the residual printed. A timeout is not data.
Persist raw output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-691/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-691/report-design.json` in `rum-report/1`, with the
  full-basis cause answer, the candidate evidence (or the exact
  impossibility), the re-watched falsifiers, the green enabled/stack
  battery and every delta row.
- `.temp/brief-691/drafts/implementation-lp08-residual-15.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 691` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the measured
answers and the draft's bar list.
