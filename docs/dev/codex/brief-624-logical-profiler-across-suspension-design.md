# brief-624 — design: the profiler counts logical calls across suspension

Scope: **NONE — no source change.** This is a design-only brief. Codex plans
and drafts the implementation briefs, and writes no production code. Base:
`692b2ab5`.

## Goal

The runtime profiler (`RUM_PROFILE_*`, `crates/rum-runtime/src/runtime/profiling/`)
records each profiled phase as a token opened by `runtime_path_timing_start`
and closed by `record_profile_runtime_path_timing` with an outcome. Builtin
and definition-rule timing work the same way. When a phase's body pauses for
input, `Err(InputRequired)` propagates out, and each enclosing phase closes
with the outcome `InputRequired`. On resume, the continuation runs in a
frame's `resume`, which reopens none of those phases. The logical call's
completion is therefore never recorded, and the counts differ from the same
program run with its input queued.

**User ruling (2026-10-20): the profiler counts logical calls.**

- A call that pauses and resumes records **one** final outcome, the same as
  on the queued path.
- Pauses are counted separately, as their own telemetry.
- Elapsed durations are never part of the comparison.

When this brief is done, the following exist:

- **An answer to the correctness question below**, settled by executed
  probes before anything else.
- **The profiler's contract, defined.** Logical counters (calls, ok, err,
  some, none, return, throw) are equal between the queued and resumed runs
  of one program. Pause telemetry and durations are defined separately.
- **A step-by-step implementation plan.**
  - Files and functions per step.
  - How the disabled path stays free.
  - How every profiled phase that can span a pause keeps its logical
    record across the pause.
  - The gate per step.
  - A call-site inventory, risks and open questions.

  Every claim carries `path:line` evidence.
- **Drafted implementation briefs**, in `TEMPLATE.md` shape.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`.
- **The finding**: brief-623's continuation 2, *Lost group
  `C2-DiagnosticOutcomeCompletion`*. The witnesses and full counter deltas
  are in `.temp/brief-623/continuation2/profile-losses.json`; the raw draft
  is `continuation2/draft-G15-C2-diagnostic-outcome-completion.md`. The
  director's review of continuation 2 records the user's ruling.
- **The simplest witness.** `f[1] := InputString["p"]`, then `f[1]`.
  - The resumed run records `definition.rhs.evaluate`, `evaluate.total`,
    `call.stage.down_values` and others as `[calls 1, ok 0, err 1, …,
    input_required 1]`; the queued run records them as `ok 1`.
  - `evaluate.stability_check` counts 2 resumed against 4 queued, and
    `evaluate.fixed_point_equal` and `evaluate.trace_update` count 0
    against 2.
- **The profiler.**
  - `profiling/timing.rs`: the token stack (`RUNTIME_PATH_TIMING_STACK`),
    the stats map, and the outcome mapping at `:375-449`.
  - `profiling/{state,reset,summary,counters,records}.rs`.
  - The guest hooks `RumProfileSnapshot`/`RumProfileReset`
    (`builtin_catalog/debug.rs:266-279`).
  - There are 98 `runtime_path_timing_start` call sites across the
    runtime and stdlib, plus builtin timing (`eval_dispatch/mod.rs`) and
    definition-rule profiling.
- **The suspension machinery.**
  - The frame queue, `TypedFrame`/`ContinueToken`, `ResumeStop`.
  - G14's evaluation-context restore and inline return events (brief-613).
  - The frames that carry each paused phase's remainder.
- **The observer.** brief-623 built a comparator that runs a program with
  profiling enabled and compares non-duration counters
  (`.temp/brief-623/`, its native and E1 probe crates). Reuse it.

## The correctness question, first

In the witness, the resumed run performs fewer `evaluate.stability_check`,
`evaluate.fixed_point_equal` and `evaluate.trace_update` phases. Before
designing any counting, establish whether that reflects **skipped semantic
work**. Does the resumed path ever deliver a value that the queued path
would have evaluated further, or omit a trace update or a fixed-point
comparison a program can observe?

- **Probe it rum against rum,** with tracing on and off. Use resumed values
  that are not yet stable:
  - a value that evaluates further, such as `f[1] := (InputString["p"]; g)`
    with `g := h` and `h := 5`;
  - a value whose evaluation has effects;
  - a value whose stability check itself observes state;
  - each of these inside held and unheld callers, and with a second pause.
- **Read the paths**: the resumed fixed point (G14's restore and return
  events) against the ordinary one in `eval_core`.
- **If a semantic difference exists**, it is a G15-class loss. Report it
  with a minimized witness, its mechanism and a repair design, as the
  **first** drafted brief. It precedes the profiler work.
- **If no semantic difference exists**, explain from source why the resumed
  path does less of this work, and what that means for the profiler's
  logical counts of these phases.

## Scope

- **Files you MAY modify**: none, except appending your report to this
  brief.
- **Scratch.** All scratch goes under `.temp/brief-624/`: probe crates
  outside the workspace, copied trees, logs and tables. No scratch test
  under `crates/`.
- Do not edit `.temp/brief-62[123]/`. Copy what you adapt.

## What the plan must settle

1. **The contract.**
   - Which counters are logical: per logical call, equal across delivery
     modes.
   - Which are execution telemetry: pause and segment counts that
     legitimately differ, reported separately and excluded from parity
     like durations.
   - What a logical call's duration means when it spans a pause (for
     example, the sum of its executing segments). Durations are reported,
     never compared.
   - What `RumProfileSnapshot` prints for each.
2. **The inventory.**
   - Every profiled phase: the 98 path-timing sites, builtin timing,
     definition-rule timing, and any other profile counter.
   - For each phase:
     - whether its body can pause (it contains a may-suspend producer;
       reuse brief-621's census and brief-623's transfer and dynamic-caller
       tables);
     - which frame or route carries its remainder on resume;
     - where its logical completion must then be recorded.
   - Phases that cannot pause need nothing.
3. **The mechanism.** Choose one, with evidence, and explain why the
   others lose.
   - **A frame-owned record.** An enabled-only field the inserting site
     fills and the resuming frame closes. This adds frame bytes even when
     profiling is off; measure that.
   - **A profiler-side table** keyed by the frame or request identity,
     enabled-only. Frames do not grow.
   - **Something better.**

   **The mechanism must cover every route a paused phase can end by:**
   - success;
   - an error;
   - `Return` and `Throw`;
   - a second pause;
   - cancellation, reset and abandonment.

   It must not fabricate a success, and it must not leak an open record.
   **What the profiler's thread-local token stack must do across a pause.**
   It must not mis-pair a token with the wrong frame, as the
   `.filter(|frame| frame.key == token.key)` at `timing.rs` silently does
   today.
4. **The disabled path.**
   - Profiling off must cost nothing: no allocation, no `Mutex`, no
     per-call branch beyond the existing cold-load `AtomicU8` gate, and no
     new frame bytes unless measured and justified.
   - State the measurement each implementation step must run: the
     one-codegen-unit instruction slopes, the copy census and the stack
     shapes on the ordinary labels, as briefs 619-622 ran them.
5. **The oracle.**
   - Which probes and tuples prove the contract. Port brief-623's 16
     witnesses and every frozen profiling-required tuple that runs without
     internal encodings.
   - The mutants: suppressing one phase's logical completion on resume
     must turn its counter receipt red.
   - How this joins the obligation-keyed manifest (brief-621's design) as
     the `phase_observation` context's counter half.
6. **The steps.** Order them so that each lands green and measured, and
   draft each step's brief.

## Verification (MANDATORY — the evidence is the deliverable)

Paste verbatim:

- the correctness-question probes and their results;
- the inventory counts (phases total; phases that can pause; routes);
- the mechanism comparison;
- the drafted briefs' file list.

Every claim cites `path:line`. Run rum against rum only.

## Checkpointing — protect the scene

Write `.temp/brief-624/PROGRESS.md` after each section. It records:

- the evidence so far;
- the next action;
- that the working tree is untouched.

## Stop-loss

STOP and report if two consecutive work sessions advance none of these:

- the correctness question;
- the inventory;
- the mechanism;
- the drafts.

## Hard rules

- **No `git`, in any form.** No network, no new `unsafe`.
- **No tracked file is modified** except appending the report to this
  brief.
- Rum against rum only (`AGENTS.md` Testing Policy).
- Paste real output. Say plainly what did not run.

## Report format (append to this brief)

1. **The correctness question**: verdict, witnesses, and mechanism.
2. **The contract.**
3. **The inventory.**
4. **The mechanism**, with the alternatives and why they lose.
5. **The disabled-path cost plan and the per-step measurement.**
6. **The oracle and mutants.**
7. **The steps and drafted briefs**, the files under `.temp/brief-624/`.
8. **Open questions**, each with a recommendation.
