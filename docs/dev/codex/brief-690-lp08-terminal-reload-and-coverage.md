# Brief 690 — LP08 residual: terminal reload and resumed-helper coverage

**Cost design on the brief-689 candidate2 shape: `Scope: NONE — no live
source, workload, manifest, inventory or version-control change.`
Disposable build probes only, under this brief's root. Candidate2 is the
byte-identical placement form measured in brief 689; the 686 form stays
the enabled/stack control.**

Brief 689 answered the placement question with measurement. Candidate2
publishes post-consumer inside the pause's enabled-only slow path and
inside the existing `Profiled` typed-push arm, removes all six site
calls, reproduces the 686 `R, G, B` tail in all 386 observations, keeps
20/20 pairs and 67/67 mirrors in two builds, and is instruction-sequence
identical to base at all three hit publication functions and at Guest
typed insertion. It stops on two cost items that are not placement
costs:

- the terminal restoration branch of
  `resume_suspended_call_with_dispatch_plan` emits one extra `movq`
  (base 54 → 55 per call) in the 686 form and in both candidates, with
  the same stack slot loaded twice; `call.rs` is byte-identical to base
  in candidate2, so the extra load is a codegen consequence of the 686
  decoded-view changes elsewhere, not of the publication placement;
- the three resumed helpers
  (`suspend_resumed_call_stage_with_boundary`,
  `suspend_resumed_evaluation_once_with_boundary`,
  `suspend_resumed_prepared_stage_with_boundary`) have zero calls in
  the disabled instruction workload, so their zero-cost claim is
  currently unproven, not false.

This round answers two questions with measured evidence:

1. **What exactly causes the terminal restoration reload, and can it be
   removed on the candidate2 shape with the terminal Some/None
   ownership invariant, the complete enabled battery, every cell-trace
   observation and the exact stack all preserved?**
2. **Can each of the three resumed helpers be exercised on a
   disabled-mode path and measured at zero versus base — or, if a
   helper is genuinely unreachable on every disabled path, proved so
   with `path:line` and recorded as a precise exclusion rather than an
   empty profile?**

## Stage A — calibration

1. **Control and basis.** Reproduce the 686 control before judging any
   candidate: the base instruments (22/22 keys, nine native
   differences, four mirrors, boundary groups 9/2/6), the raw 19/20
   frontier, the 686 form's 20/20 pairs, 67/67 mirrors, seven Throw
   kills, the 193-snapshot-per-draw trace with the `R, G, B` tail, the
   exact 18,304 / 18,896 stack, the +9/call fixed point and the
   `movq:+1` terminal row. Reproduce candidate2 byte-identically from
   `.temp/brief-689/src/candidate2/` as the second basis; the 689
   numbers are a claim, not evidence.
2. **The terminal branch map.** Re-derive from both bases, with
   `path:line`, every source change in the 686 form relative to base
   that can reach the generated body of
   `resume_suspended_call_with_dispatch_plan`: the declared inline /
   cold / never-inline attributes of every callee, every inlined
   wrapper, every changed function in the same codegen unit. State the
   candidate causes, then decide them by measurement — a bisect of the
   686 changes built as disposable arms, each measured at 50 and 100
   iterations, naming the smallest change that flips 54 to 55.
3. **The disabled suspension map.** Re-derive with `path:line` how a
   disabled-mode evaluation reaches each of the three resumed helpers:
   which builtin/request produces `InputRequired`, how a resumed call
   suspends again at each stage, and which existing fixture (if any)
   already does this with profiling off. Do not assume unreachability;
   the burden is on the measurement or the proof.
4. **The baseline.** Persist the 686 control's and candidate2's
   per-address cost panels (two builds, two draws) and their cell
   traces as the control for every candidate.

## Stage B — the two targets

5. **Terminal reload.** From the measured cause, produce at most one
   candidate shape that removes the extra load: the branch must execute
   the base instruction sequence at 50 and 100 iterations, in two
   independent builds and two draws, with the terminal Some/None
   ownership invariant stated and preserved — Some restores all
   prepared arguments and Unevaluated wrappers before normalization,
   None uses the final shared arguments, and the restoration plan is
   owned and dropped exactly once. If measurement shows no source shape
   removes it without violating that invariant, return a measured
   impossibility naming the exact opcode, branch, invariant, the
   entailing 686 change and the smallest alternative.
6. **Resumed-helper coverage.** Provide a disabled-mode workload that
   exercises each of the three resumed helpers and measure their
   per-address cost against base, requiring zero; or prove with
   `path:line` that a helper is unreachable on every disabled path and
   record the exclusion with that proof. An empty profile is not zero
   evidence and may not be reported as one. The workloads must be
   disposable probe fixtures under this brief's root.
7. **Constraints.**
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
     decoding, the typed adapter, `LegacyMode`, the transport, the
     guest carriers and the eight definition finishes stay
     byte-identical. No test or fixture is weakened.
   - **No hidden state.** No new evaluator-frame field, no session field
     carrying diagnostic state, no selector, no outlined guest adapter,
     no new `unsafe`, no `+64` publication frame.
   - **One variable per candidate.** At most two cost candidates, each
     starting from the byte-identical candidate2 basis; bisect arms are
     diagnosis, not candidates.
8. **Falsifiers.** Re-watch the four 689 placement mutants (position →
   `B, R, G`; owner-loss; wrong-owner/premature; exactly-once) on the
   final measured shape, and ship one mutant for the terminal invariant:
   a shape that drops or duplicates the Some restoration must be
   observed by its counter assertion, not merely compile differently.
   Each mutant carries its exact counter or trace assertion and its
   observed red.

## Stage C — acceptance on the green cost form

9. **Enabled.** Queue 9/9 with parser quiescence `(0,0,0,0,0)` and
   strong-count ownership equality; slot 2/2; record 6/6; direct
   definition flush `Outcome::Some`, `faults=0`, counter
   `[1,1,0,1,0,0,0,0,0,0]`, quiescent; six lifecycle actions
   `open=0 leases=0 fabricated_ok=0`; all 20 pairs `native_same=true
   faults=0`; mirrors 67/67; seven Throw counter-only kills; all 150 key
   rows with every prior PASS restored.
10. **Stack.** Function 18,304 / Module 18,896 B per level, residual 0,
    two independent CGU=1 builds and two draws, four heights twice each,
    on the same candidate that passed the battery.
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

1. **The reload answer.** The measured cause of the duplicate restoration
   load, the candidate that removes it (or the impossibility naming the
   opcode, branch, invariant and entailing change), and the terminal
   branch back at base cost in two builds and two draws.
2. **The coverage answer.** Each resumed helper measured at zero on a
   disabled-mode path, or a `path:line` unreachability proof recorded as
   an explicit exclusion; no empty-profile inference.
3. **The preserved green.** The placement, position trace, enabled
   battery, exact stack and four watched mutants on the final shape.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the ripple inventory, risks and open
   questions.

## Context pointers

- `.temp/brief-689/report-design.json` — the placement answer, the
  position proof, the four mutants and every signed row.
- `.temp/brief-689/src/candidate2/` — the byte-identical placement form;
  `.temp/brief-689/src/form/` — the 686 control.
- `.temp/brief-689/candidate2.patch` — the six-file placement patch.
- `.temp/brief-689/control-cell-trace.json`,
  `candidate2-cell-trace-results.json`,
  `all-arm-stack-results.json`, `all-delta-rows.json`,
  `candidate2-cost-answer.json`, `reload-branch-evidence.json`,
  `candidate2-address-panel.json`.
- `.temp/brief-689/inputs/extended-probe.rs`,
  `.temp/brief-689/active/crates/rum-runtime/src/runtime/profiling/tests/brief677_probe.rs`,
  `.temp/brief-689/watch_mutants.py` — probe fixtures to copy and
  extend for the disabled suspension workload.
- `.temp/director/brief-689-gate.md` — the stop and the two rulings.
- `docs/dev/profiling.md` — the contract.
- Live sites: `suspension/core.rs` (`begin_input_suspension`,
  `insert_typed_erased_frame`), `suspension/core/storage.rs`,
  `suspension/core/profile_queue.rs`,
  `profiling/logical/mod.rs` (`pause_phase_at_finish_slow`),
  `profiling/logical/collector.rs`,
  `profiling/logical/queue_api.rs`, `profiling/timing.rs`,
  `eval_call_prepare.rs`, `resolution.rs`, `eval_core/call.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-690/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-686/`, `.temp/brief-687/`, `.temp/brief-688/`,
  `.temp/brief-689/` or any other retained root, except to read and
  copy. Never execute a file in place;
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
  record. Owner/epoch/control authentication, exactly-once completion
  and the ownerless restore stay stated with `path:line`.
- **The enabled form.** 20/20, `faults=0`, the unchanged battery and the
  exact 18,304/18,896 stack in two independent CGU=1 builds and two
  draws. No assertion or fixture is weakened.
- **The excluded forms.** The 683 observe-only guard, the 684
  position-only shape, the 685 ambient-owner guard, the panicking
  take-path fallback, both 687 cost shapes, the 688 owner-position
  placement and the 689 candidate1 shape stay excluded. Candidate2 may
  change only for the terminal-reload shape.
- **Every arm reduced.** Every candidate's stack and battery runs are
  reduced into the persisted artifact; a run that exists only as a log
  is not evidence. No row dropped, no negative netted.
- Every count, key, route and `path:line` is re-derived from the
  candidate2 basis; a brief-689 number is a claim, never evidence.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit; two independent builds
and draws for every cost row; serial Callgrind, CPU-pinned, release,
every `RUM_PROFILE_*` cleared except the declared modes; four stack
heights twice each with the residual printed. A timeout is not data.
Persist raw output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-690/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-690/report-design.json` in `rum-report/1`, with the two
  measured answers (or the exact impossibilities), the candidate
  evidence, the re-watched falsifiers, the green enabled/stack battery
  and every delta row.
- `.temp/brief-690/drafts/implementation-lp08-residual-14.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 690` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the two
answers (measured) and the draft's bar list.
