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


---

# Codex report — brief-624 (2026-10-20)

Design-only deliverable. No production source, test, dependency or script was
changed. All probes/copies/drafts are under `.temp/brief-624/`; this appended
report is the only edited entry file. No version-control command or network
was used. The brief supplied base `692b2ab5`; identity was not independently
queried. Source hashes, not version-control state, protect this scene
(`.temp/brief-624/entry-hashes.json:1`, `.temp/brief-624/provenance.json:1`).

## 1. The correctness question

**Verdict: no semantic loss found.** The ordinary and resumed paths perform
the same remaining comparisons, stability tests and trace updates; only the
ordinary path records these phases. This was settled by executed probes
before choosing an accounting mechanism. The full comparator checks values,
typed failure, prints/messages/ordered trace, log/projections, output prefixes,
pending-owner/overlap handling, queue emptiness and a fresh request; it also
checks the independent continuation-fault probe
(`.temp/brief-624/oracle/support/observe.rs:1`,
`.temp/brief-624/oracle/support/drive.rs:31`,
`.temp/brief-624/oracle/support/faults.rs:1`).

Source mechanism: ordinary `resolution.rs:384`, `:392` and `:349` compare,
trace and query stability with profiling, while resumed `:404`, `:408` and
`:420` perform the same operations without those hooks (all in
`crates/rum-runtime/src/runtime/resolution.rs`). Both preserve pass/resource/
cycle checks at `:341`, `:347`, `:358` and `:413`, `:419`, `:424`. The saved
frames pass the actual outcome into that remaining loop
(`crates/rum-runtime/src/runtime/resolution/resume_frames.rs:35`, `:60`). G14
restores stack/depth without rewinding the changed evaluation epoch and
enforces return resources before the consumer
(`crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs:32`,
`:72`; `crates/rum-runtime/src/runtime/suspension/core.rs:714`, `:725`).
Stability observes current definitions/cache, without guest evaluation
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:267`,
`crates/rum-runtime/src/runtime/eval_core/stability.rs:502`). This explains
the profiler discrepancy; it does not justify adding semantic evaluation.

The executed panel uses the g→h→5 chain, an effectful value (append plus
Print), an ownvalue mutation h:=5→h=9, and a downvalue mutation
h[1]:=5→h[1]=9. It crosses direct, HoldAll identity and ordinary identity
callers, one/two pauses, and tracing off/on. These are finite witnesses,
supported by the matching source paths, not a proof for arbitrary programs
(`.temp/brief-624/correctness/plans.jsonl:1`). No G15 semantic repair brief is
drafted; LP01 is the profiler contract/oracle prerequisite.

Exact probes, verbatim from the executed plan file (setup, source, input
kind/prompt/answer and trace flag are all retained):

```jsonl
{"id": "624.correctness.further.direct.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.further.direct.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.further.direct.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.further.direct.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.further.held.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.further.held.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.further.held.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.further.held.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.further.unheld.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.further.unheld.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.further.unheld.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.further.unheld.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=h;h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.effect.direct.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.effect.direct.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.effect.direct.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.effect.direct.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.effect.held.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.effect.held.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.effect.held.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.effect.held.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.effect.unheld.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.effect.unheld.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.effect.unheld.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.effect.unheld.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];Print[\"g\"];5);f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.epoch.direct.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.epoch.direct.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.epoch.direct.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.epoch.direct.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "f[1]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.epoch.held.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.epoch.held.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.epoch.held.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.epoch.held.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.epoch.unheld.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.epoch.unheld.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.epoch.unheld.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.epoch.unheld.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g:=(log=Append[log,\"g\"];h=9;h);h:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g)", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.call-epoch.direct.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];g[])", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.call-epoch.direct.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];g[])", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.call-epoch.direct.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g[])", "source": "f[1]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.call-epoch.direct.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g[])", "source": "f[1]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.call-epoch.held.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];g[])", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.call-epoch.held.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];g[])", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.call-epoch.held.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g[])", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.call-epoch.held.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g[])", "source": "held[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
{"id": "624.correctness.call-epoch.unheld.1.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];g[])", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": false}
{"id": "624.correctness.call-epoch.unheld.1.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];g[])", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.correctness.call-epoch.unheld.2.trace0", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g[])", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": false}
{"id": "624.correctness.call-epoch.unheld.2.trace1", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation", "setup": "SetAttributes[held,HoldAll];held[x_]:=x;unheld[x_]:=x;g[]:=(log=Append[log,\"g\"];h[1]=9;h[1]);h[1]:=5;f[1]:=(InputString[\"p\"];InputString[\"q\"];g[])", "source": "unheld[f[1]]", "pauses": [["InputString", "p", "x"], ["InputString", "q", "y"]], "trace": true}
```

Verbatim results, including both values and trace event counts; SAME also requires exact ordered trace/effect equality, not just those counts (`.temp/brief-624/correctness/summary.tsv:1`, `results.jsonl:1`):

```text
id	valid	same	Rvalue	Qvalue	Rtrace	Qtrace	log_equal
624.correctness.further.direct.1.trace0	True	True	5	5	0	0	True
624.correctness.further.direct.1.trace1	True	True	5	5	6	6	True
624.correctness.further.direct.2.trace0	True	True	5	5	0	0	True
624.correctness.further.direct.2.trace1	True	True	5	5	7	7	True
624.correctness.further.held.1.trace0	True	True	5	5	0	0	True
624.correctness.further.held.1.trace1	True	True	5	5	8	8	True
624.correctness.further.held.2.trace0	True	True	5	5	0	0	True
624.correctness.further.held.2.trace1	True	True	5	5	9	9	True
624.correctness.further.unheld.1.trace0	True	True	5	5	0	0	True
624.correctness.further.unheld.1.trace1	True	True	5	5	8	8	True
624.correctness.further.unheld.2.trace0	True	True	5	5	0	0	True
624.correctness.further.unheld.2.trace1	True	True	5	5	9	9	True
624.correctness.effect.direct.1.trace0	True	True	5	5	0	0	True
624.correctness.effect.direct.1.trace1	True	True	5	5	11	11	True
624.correctness.effect.direct.2.trace0	True	True	5	5	0	0	True
624.correctness.effect.direct.2.trace1	True	True	5	5	12	12	True
624.correctness.effect.held.1.trace0	True	True	5	5	0	0	True
624.correctness.effect.held.1.trace1	True	True	5	5	13	13	True
624.correctness.effect.held.2.trace0	True	True	5	5	0	0	True
624.correctness.effect.held.2.trace1	True	True	5	5	14	14	True
624.correctness.effect.unheld.1.trace0	True	True	5	5	0	0	True
624.correctness.effect.unheld.1.trace1	True	True	5	5	13	13	True
624.correctness.effect.unheld.2.trace0	True	True	5	5	0	0	True
624.correctness.effect.unheld.2.trace1	True	True	5	5	14	14	True
624.correctness.epoch.direct.1.trace0	True	True	9	9	0	0	True
624.correctness.epoch.direct.1.trace1	True	True	9	9	13	13	True
624.correctness.epoch.direct.2.trace0	True	True	9	9	0	0	True
624.correctness.epoch.direct.2.trace1	True	True	9	9	14	14	True
624.correctness.epoch.held.1.trace0	True	True	9	9	0	0	True
624.correctness.epoch.held.1.trace1	True	True	9	9	15	15	True
624.correctness.epoch.held.2.trace0	True	True	9	9	0	0	True
624.correctness.epoch.held.2.trace1	True	True	9	9	16	16	True
624.correctness.epoch.unheld.1.trace0	True	True	9	9	0	0	True
624.correctness.epoch.unheld.1.trace1	True	True	9	9	15	15	True
624.correctness.epoch.unheld.2.trace0	True	True	9	9	0	0	True
624.correctness.epoch.unheld.2.trace1	True	True	9	9	16	16	True
624.correctness.call-epoch.direct.1.trace0	True	True	9	9	0	0	True
624.correctness.call-epoch.direct.1.trace1	True	True	9	9	15	15	True
624.correctness.call-epoch.direct.2.trace0	True	True	9	9	0	0	True
624.correctness.call-epoch.direct.2.trace1	True	True	9	9	16	16	True
624.correctness.call-epoch.held.1.trace0	True	True	9	9	0	0	True
624.correctness.call-epoch.held.1.trace1	True	True	9	9	17	17	True
624.correctness.call-epoch.held.2.trace0	True	True	9	9	0	0	True
624.correctness.call-epoch.held.2.trace1	True	True	9	9	18	18	True
624.correctness.call-epoch.unheld.1.trace0	True	True	9	9	0	0	True
624.correctness.call-epoch.unheld.1.trace1	True	True	9	9	17	17	True
624.correctness.call-epoch.unheld.2.trace0	True	True	9	9	0	0	True
624.correctness.call-epoch.unheld.2.trace1	True	True	9	9	18	18	True
```

Verbatim execution totals (`.temp/brief-624/correctness/run.log`, `results.jsonl`):

```text
test result: total=48 SAME=48 DIFF=0 invalid=0 boundary=0
continuation-slot fault-occurrence delta=0; observations=672
```

The scratch E1 crate built offline against the unchanged workspace. A second
scratch crate uses a current-source copy with brief-623's exhaustive observer
appended at three API/forwarding points; later layout-only additions stay in
that copy. One scratch layout build failed on Rust privacy, was corrected in
the owning queue module, and then built successfully; no production file was
used to fix it (`.temp/brief-624/build-e1.log`, `build-profile.log`,
`build-layout.log`, `build-layout-fixed.log`, `build-frame-layout.log`;
`.temp/brief-624/provenance.json:1`). Exact successful build lines:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 26.70s
```
```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 26.15s
```
```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.49s
```
```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.16s
```

Enabled profiling reproduced every minimized loss while preserving semantics; all 650 saved runnable programs were also valid. Verbatim reducer output (`.temp/brief-624/profile/run-summary.txt:1`):

```text
minima {"counter_DIFF": 16, "counter_SAME": 0, "semantic_SAME": 16, "total": 16, "valid": 16}
semantic-enabled {"counter_DIFF": 48, "counter_SAME": 0, "semantic_SAME": 48, "total": 48, "valid": 48}
frozen {"counter_DIFF": 650, "counter_SAME": 0, "semantic_SAME": 650, "total": 650, "valid": 650}
```

Verbatim independent fault-probe output for these three panels:

```text
continuation-slot fault-occurrence delta=0; observations=224
continuation-slot fault-occurrence delta=0; observations=672
continuation-slot fault-occurrence delta=0; observations=9490
```

All 16 frozen minimized probes were ported and re-executed (`.temp/brief-624/profile/minima-plans.jsonl:1`):

```jsonl
{"id": "624.min.P0256", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "f[1]:=InputString[\"p\"]", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.min.P0259", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "f[x_]:=InputString[\"p\"]", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.min.P0266", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "f[x_]/;True:=InputString[\"p\"]", "source": "f[1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.min.P0314", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "a/:h[a,x_]/;True:=InputString[\"p\"]", "source": "h[a,1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.min.P0315", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "a/:h[a,x_]:=InputString[\"p\"]", "source": "h[a,1]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.min.P0413", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "f[Input[\"p\"]]=1", "pauses": [["Input", "p", "1"]], "trace": true}
{"id": "624.min.P0431", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "f[Input[\"p\"]]=1", "pauses": [["Input", "p", "1"]], "trace": true}
{"id": "624.min.P0484", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "f[Input[\"p\"]]=1", "pauses": [["Input", "p", "1"]], "trace": true}
{"id": "624.min.P0487", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "Function[InputString[\"p\"]][]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.min.P0489", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "f[Input[\"p\"]]=1", "pauses": [["Input", "p", "1"]], "trace": true}
{"id": "624.min.P0509", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "f:=InputString[\"p\"]", "source": "f", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.min.P0511", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "f[Input[\"p\"]]=1", "pauses": [["Input", "p", "1"]], "trace": true}
{"id": "624.min.P0515", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "f[Input[\"p\"]]=1", "pauses": [["Input", "p", "1"]], "trace": true}
{"id": "624.min.P0814", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "Catch[Throw[1,t],t,Function[{x,t},InputString[\"p\"]]]", "pauses": [["InputString", "p", "x"]], "trace": true}
{"id": "624.min.P1472", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "f[Input[\"p\"]]=1", "pauses": [["Input", "p", "1"]], "trace": true}
{"id": "624.min.P1507", "descriptor": "AbortProtect.body", "variant": "Middle", "context": "phase_observation.trace", "setup": "", "source": "Module[{x=1},InputString[\"p\"]]", "pauses": [["InputString", "p", "x"]], "trace": true}
```

Verbatim per-row results (`.temp/brief-624/profile/minima-summary.tsv:1`):

```text
row	semantic_SAME	profile_SAME	difference_keys
624.min.P0256	True	False	16
624.min.P0259	True	False	16
624.min.P0266	True	False	16
624.min.P0314	True	False	16
624.min.P0315	True	False	16
624.min.P0413	True	False	18
624.min.P0431	True	False	18
624.min.P0484	True	False	18
624.min.P0487	True	False	14
624.min.P0489	True	False	18
624.min.P0509	True	False	15
624.min.P0511	True	False	18
624.min.P0515	True	False	18
624.min.P0814	True	False	16
624.min.P1472	True	False	18
624.min.P1507	True	False	17
```

The simplest witness has these exact local/full-wrapper deltas. Runtime vectors are `(calls,ok,err,some,none,return_err,throw_err,input_required)`; builtin/rule vectors are `(calls,ok,err)`. These are BASE observations, not the proposed new contract (`.temp/brief-624/profile/624.min.P0256.delta.json:1`):

```json
[
  {
    "key": "builtin|InputString|-|1",
    "resumed": [
      1,
      0,
      1
    ],
    "queued": [
      1,
      1,
      0
    ]
  },
  {
    "key": "definition-rule|down_value_rhs_eval|f|0",
    "resumed": [
      1,
      0,
      1
    ],
    "queued": [
      1,
      1,
      0
    ]
  },
  {
    "key": "runtime|call.stage.down_values|-",
    "resumed": [
      1,
      0,
      1,
      0,
      0,
      0,
      0,
      1
    ],
    "queued": [
      1,
      1,
      0,
      1,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|call.stage.ordinary_builtin|-",
    "resumed": [
      1,
      0,
      1,
      0,
      0,
      0,
      0,
      1
    ],
    "queued": [
      1,
      1,
      0,
      1,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|definition.rhs.evaluate|-",
    "resumed": [
      1,
      0,
      1,
      0,
      0,
      0,
      0,
      1
    ],
    "queued": [
      1,
      1,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|definition.rhs.total|-",
    "resumed": [
      1,
      0,
      1,
      0,
      0,
      0,
      0,
      1
    ],
    "queued": [
      1,
      1,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate.fixed_point_equal|-",
    "resumed": [
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate.once_unstable|-",
    "resumed": [
      2,
      0,
      2,
      0,
      0,
      0,
      0,
      2
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate.stability_check|-",
    "resumed": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ],
    "queued": [
      4,
      4,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate.total|-",
    "resumed": [
      2,
      0,
      2,
      0,
      0,
      0,
      0,
      2
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate.trace_update|-",
    "resumed": [
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate_call.dispatch|-",
    "resumed": [
      2,
      0,
      2,
      0,
      0,
      0,
      0,
      2
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate_call.resume_after_head|-",
    "resumed": [
      2,
      0,
      2,
      0,
      0,
      0,
      0,
      2
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate_call.total|-",
    "resumed": [
      2,
      0,
      2,
      0,
      0,
      0,
      0,
      2
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate_inner_unstable.call|-",
    "resumed": [
      2,
      0,
      2,
      0,
      0,
      0,
      0,
      2
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  {
    "key": "runtime|evaluate_inner_unstable.total|-",
    "resumed": [
      2,
      0,
      2,
      0,
      0,
      0,
      0,
      2
    ],
    "queued": [
      2,
      2,
      0,
      0,
      0,
      0,
      0,
      0
    ]
  }
]
```

## 2. The contract

One logical phase is one invocation of the operation identified by its
instrumentation site and input/cursor, through its actual continuation. A
native function reentry is not another invocation of that same operation.
Distinct recursive invocations, distinct operands/iterations and later
dispatch stages remain distinct calls. Aggregate keys retain phase,
source, builtin arity or definition kind/symbol/index; unique record IDs
are NOT aggregate keys. Existing key shapes and outcome accounting are at
`crates/rum-runtime/src/runtime/profiling/mod.rs:412` and
`crates/rum-runtime/src/runtime/profiling/timing.rs:415`.

For an isolated completed program, using identical setup, limits, flags and
answer sequence, the union of all logical keys must have identical
`(calls, ok, err, some, none, return_err, throw_err)` in queued and resumed
runs. Missing keys are zero only after validating exhaustive collection;
deleting a required key is a schema error, not a normalization. Builtin and
definition-rule `calls/ok/err`, rule-RHS `calls/changed/unchanged` and logical
definition-dispatch decisions also compare. Keep incoming-outcome phases'
existing meaning: `definition.rhs.finish` and `function.apply.finish`
observe Return before absorbing it; `catch.resume_after_body` observes its
incoming Throw even if a handler returns Ok. These sampling policies are
explicit, not inferred from the last value (`crates/rum-runtime/src/runtime/definitions/application/rhs.rs:123`,
`crates/rum-runtime/src/runtime/scoping/callable.rs:86`, `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:121`, all
under `crates/rum-runtime/src/runtime/`). Do not change P0817/P0818's already
correct incoming-Throw observation to disguise ancestor losses
(`.temp/brief-623/continuation2/draft-G15-C2-diagnostic-outcome-completion.md:15`).

Terminal counters are published once, at the actual specified completion
boundary. `InputRequired` is NOT an err or a terminal call. A paused open
record contributes `open=1`, no final outcome. Actual failure contributes
err; Return and both Throw variants contribute err plus their respective
subset. Some/None are subsets of ok, not inferred from final guest values.
For successful finished programs, calls=ok+err. For lifecycle observations,
calls=ok+err+abandoned; actual Cancelled adds err and cancelled, structural
discard without an actual result adds abandoned and a reason, never ok.
This extends today's eight-way outcome mapping, which currently classifies
InputRequired as err (`crates/rum-runtime/src/runtime/profiling/timing.rs:435`).

Separate execution telemetry includes per-record `pauses`, executing
`segments`, request input pauses/resumes, abandonment reasons and
instrumentation faults. These may differ across delivery modes. The old
`input_required` field moves into this telemetry explicitly; no duration or
pause field participates in logical parity. `module_resume_entries` and
`module_resume_max_index` describe native loop progress, not completed calls
(`crates/rum-runtime/src/runtime/profiling/records.rs:79`). Presence scans, allocation/copy/sharing,
fingerprint/reuse/cardinality, cache hits/misses/invalidation-site visits and
incremental-print frequency are physical execution telemetry. They remain
reported, with every existing schema field classified in
`.temp/brief-624/inventory/other-counters.tsv:1`, `.temp/brief-624/inventory/tls-storage.tsv:1` and `.temp/brief-624/inventory/auxiliary-storage.tsv:1`; they are
not a license to reclassify a failing runtime/builtin/definition call.
Logical entry/application/scan counts in evaluation-cache profiling remain
entry/decision events and are compared, with source restoration; they do not
need an open completion record (`crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:109`).

Durations are the sum of executing monotonic-Instant intervals for each
record, including child execution while that record is logically active.
Host waiting intervals are excluded. Self duration subtracts active child
intervals in the SAME profiling domain, once per segment. Paused ancestors
restart only when their actual saved context executes. Boundary-only
decision records do not acquire child execution retroactively. Report
`total_ns/self_total_ns/avg_ns` and segment counts, but never compare them.
The existing monotonic primitive is `crates/rum-runtime/src/runtime/profiling/counters.rs:131`; runtime and
builtin child accounting is currently `crates/rum-runtime/src/runtime/profiling/timing.rs:347` and `:396`.

`RumProfileSnapshot` still returns Null and writes diagnostics, as
`crates/rum-runtime/src/runtime/debug_tools/basic.rs:124` does now. Proposed schema v2 prints a labelled
epoch header, then EVERY logical key once in stable key order, with final
tuple, open, abandoned/cancelled, followed by separately labelled execution
telemetry and durations. Ranked top-64 timing displays may remain as extra
human summaries, but cannot be the exhaustive snapshot/oracle
(`crates/rum-runtime/src/runtime/profiling/report_timing.rs:188`, `:331`). Snapshot does not finish or restart
an open record; its own in-progress invocation appears only as open, and the
oracle captures directly before post-subject probes. No new guest builtin
or public host API is needed (`crates/rum-runtime/src/runtime/builtin_catalog/debug.rs:266`, `:277`).

Use one process-wide logical profiling epoch collector, behind the existing
enabled gates. This makes a record portable when its Session resumes on
another thread; physical per-thread diagnostic tables can remain telemetry.
The comparison isolates one epoch or filters the native receipt by owner;
other simultaneously profiled requests are not part of its program tuple.
This is a deliberate contract choice: currently scalar counters are global
atomics while runtime/builtin tables are TLS (`crates/rum-runtime/src/runtime/profiling/state.rs:33`, `:81`).
Reset retires the old epoch, explicitly discharges its open records as
profile-reset abandonment, and starts an empty epoch. Old in-flight tokens
and queue markers keep the old epoch identity and cannot publish in the new
one. Do not clear the native token stack underneath live tokens, as
`crates/rum-runtime/src/runtime/profiling/reset.rs:84` does now. Reset's own entered scopes belong to the
old epoch; subsequent operations belong to the new one.

## 3. The inventory

Counting convention: there are **97 executable path-start sites**, not 98.
The 98 textual matches include the start helper's own declaration at
`crates/rum-runtime/src/runtime/profiling/timing.rs:97`. All executable path
sites in this scene are in rum-runtime; stdlib-installed handlers feed the
shared builtin dispatcher. The 77 literal keys expand with four atom-kind
and eight stage keys to **89 runtime phase names**
(`crates/rum-runtime/src/runtime/eval_core/mod.rs:223`,
`crates/rum-runtime/src/runtime/eval_core/call.rs:359`). Dynamic source,
symbol/index and arity axes are not collapsed in receipts.

Across primary timing families, the total is **105 phase-name templates**:
89 runtime names + one builtin family + three definition-rule kinds + 12
rule-RHS phase names. **51 can span/defer a pause**: 47 runtime names +
builtin + three definition-rule kinds. At source-site granularity, **49** of
the 97 path sites need a retained/deferred logical record: **45** contain a
may-suspend producer and **four** forward an already-produced InputRequired;
**48** timed bodies are synchronous. The four forwarded boundaries are Set's
non-atom rhs.suspend and While test/body/Compound item .suspend, explicitly
distinguished from a new producer. There are **32 path completion-route
groups plus builtin and definition-rule routes = 34 groups**
(`.temp/brief-624/inventory/path-sites.tsv:1`, `expanded-paths.tsv:1`,
`builtin-sites.tsv:1`, `definition-sites.tsv:1`, `rule-rhs-sites.tsv:1`,
`routes.tsv:1`).

Verbatim inventory-count output (`.temp/brief-624/inventory/counts-output.txt:1`):

```text
{"atomic_u64_declarations": 90, "auxiliary_cache_storage_items": 9, "builtin_start_sites": 1, "completion_routes": 34, "definition_rule_start_sites": 8, "executable_path_sites": 97, "expanded_phase_keys": 89, "forwarded_pause_sites": 4, "literal_phase_keys": 77, "other_counter_declarations_and_fields": 189, "path_completion_routes": 32, "phase_keys_spanning_pause": 47, "primary_phase_families_spanning_pause": 51, "primary_timing_phase_families": 105, "producer_sites": 45, "rule_rhs_literal_phase_keys": 12, "rule_rhs_record_sites": 22, "sites_spanning_pause": 49, "synchronous_sites": 48, "textual_path_occurrences": 98, "tls_storage_items": 32}
```

Per-file inventory, with the first executable source site as evidence; every individual site has its own source/finish/census/transfer/dynamic-caller citations in path-sites.tsv:

| file | sites | can_span_pause | evidence |
|---|---|---|---|
| crates/rum-runtime/src/runtime/call_preparation.rs | 3 | 0 | crates/rum-runtime/src/runtime/call_preparation.rs:349 |
| crates/rum-runtime/src/runtime/definitions/application/down_values.rs | 11 | 2 | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:27 |
| crates/rum-runtime/src/runtime/definitions/application/rhs.rs | 5 | 4 | crates/rum-runtime/src/runtime/definitions/application/rhs.rs:14 |
| crates/rum-runtime/src/runtime/definitions/application/sub_values.rs | 10 | 1 | crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:25 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | 17 | 6 | crates/rum-runtime/src/runtime/definitions/definitions.rs:29 |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | 8 | 3 | crates/rum-runtime/src/runtime/eval_call_prepare.rs:241 |
| crates/rum-runtime/src/runtime/eval_core/call.rs | 9 | 8 | crates/rum-runtime/src/runtime/eval_core/call.rs:27 |
| crates/rum-runtime/src/runtime/eval_core/mod.rs | 6 | 5 | crates/rum-runtime/src/runtime/eval_core/mod.rs:142 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs | 6 | 6 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:155 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs | 5 | 3 | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:13 |
| crates/rum-runtime/src/runtime/resolution.rs | 5 | 2 | crates/rum-runtime/src/runtime/resolution.rs:325 |
| crates/rum-runtime/src/runtime/scoping/callable/status_application.rs | 1 | 1 | crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:33 |
| crates/rum-runtime/src/runtime/scoping/callable.rs | 2 | 1 | crates/rum-runtime/src/runtime/scoping/callable.rs:62 |
| crates/rum-runtime/src/runtime/scoping/lexical/module.rs | 9 | 7 | crates/rum-runtime/src/runtime/scoping/lexical/module.rs:25 |

The eight definition-rule starts are three DownValue, three UpValue and two
SubValue arms. The single builtin start feeds unchecked and memory-growth
test dispatch. Rule-RHS has 22 recorder expressions for 12 non-suspending
phase names; the iterative phase argument only forwards already listed
names (`.temp/brief-624/inventory/definition-sites.tsv:1`,
`builtin-sites.tsv:1`, `rule-rhs-sites.tsv:1`;
`crates/rum-runtime/src/runtime/substitution/binding_substitution/iterative.rs:157`).

The full inventory is not limited to the 16 known losses. The row-level
inventory links brief-621's calibrated census and brief-623's transfers and
dynamic callers, without treating a census outcome-helper false positive as
a real producer (`.temp/brief-621/population.tsv:1`,
`.temp/brief-623/continuation2/transfers.tsv:1`,
`.temp/brief-623/continuation2/dynamic-callers.tsv:1`). The 34 route rows name
the actual frames, channel, proposed completion location and source-specific
qualifications. They cover head versus whole call, Prepared versus Optional,
resource-return events, legacy Expr association/destructuring, finalizers,
Catch bypass and Module native reentry (`.temp/brief-624/inventory/routes.tsv:2`).

Every other counter is classified in other-counters.tsv: 90 AtomicU64
declarations and 99 stats fields (189 schema entries, not 189 independent
runtime phases). This includes lexical aggregate/substitution/presence/rebuild/
provenance/dispatch counters, evaluation-cache counts and by-source timing,
representation/sharing counters, and the CLI's four optional allocation
counters as telemetry. Profiling has 32 TLS storage items; evaluation-cache
context/timing/shared maps add nine storage items. Storage maps/guards and
derived groups are not mistaken for extra open logical calls
(`.temp/brief-624/inventory/other-counters.tsv:1`, `tls-storage.tsv:1`,
`auxiliary-storage.tsv:1`; `crates/rum-cli/src/alloc_count.rs:9`).

Synchronous phases need no retention mechanism. Nevertheless their actual
resumed occurrences must be observed: fixed-point comparison/trace/stability,
Set finish after an RHS pause, and the first resumed argument append are
specific missing-entry/mirror cases. Module's loop start on native reentry
must reuse its logical identity. These are required observations of real
operations, not fabricated counts
(`crates/rum-runtime/src/runtime/resolution.rs:404`,
`crates/rum-runtime/src/runtime/definitions/definitions.rs:64`,
`crates/rum-runtime/src/runtime/eval_call_prepare.rs:393`,
`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:197`).

## 4. The mechanism

Choose a profiler-side record registry plus enabled-only inline completion
entries in the existing mixed frame queue. This combines explicit boundary
ownership with no fields on existing semantic frames. The executable layout
experiment is in `.temp/brief-624/profile/layout.log` (the `624_LAYOUT` line); it
measures a projected enum, not a production candidate:

```
624_LAYOUT Entry=24 projected_Entry=24 QueuedEntry=40 projected_QueuedEntry=40 cell_with_record=48
```

Actual private-frame field comparisons also ran in the scratch copy
(`.temp/brief-624/profile/frame-layout.log`):

```
624_FRAME_LAYOUT EvaluateCallSuspensionFrame=8 with_record=16
624_FRAME_LAYOUT EvaluateDefinitionRhsSuspensionFrame=0 with_record=8
```

The current queue already admits non-frame events and preserves their
position across widening/splicing (`crates/rum-runtime/src/runtime/suspension/typed/entry.rs:9`,
`crates/rum-runtime/src/runtime/suspension/core/queue.rs:18`, `:72`, `:193`). Proposed
`Entry::ProfileBoundary(Arc<ProfileBoundary>)` uses a thin pointer and fits
the existing enum payload in this measured projection. It allocates a queue
cell only with profiling enabled AND a suspended logical record. Production
layout equality remains an implementation gate; the field-added alternatives were
measured using the actual private frame types, while the new enum is a projection. Existing concrete frames,
ResumeContext, Session and ordinary queue cells acquire no fields.

The enabled registry holds `(EpochId, OwnerId, RecordId)` plus immutable
aggregate key, parent record ID, outcome policy, segment accumulators and
completion state. IDs are monotonic, not source-key matches or expression
addresses. Owner identity is obtained only on the enabled path from the
existing request-control Arc, pinned by a retained handle and namespaced by
accepted-request generation or an enabled-only direct-evaluate root serial.
Never key by the movable Session address, a prompt, or a bare per-session
SuspensionGeneration (`crates/rum-runtime/src/runtime/model/request_cancellation.rs:36`,
`crates/rum-runtime/src/runtime/session/session_api.rs:177`, `crates/rum-runtime/src/runtime/suspension/core.rs:29`). No new field is needed
on accepted tickets or Session. Counters are collected into an epoch-owned
shared collector, permitting thread migration without a TLS dangling record.

Native tokens can become small optional identity handles. All allocations,
registry access and locks occur AFTER the existing start/finish enabled
gates (`crates/rum-runtime/src/runtime/profiling/timing.rs:97`, `:375`). Preserve lazy argument evaluation:
no side-table lookup, key formatting or capture construction is done before
the gate. Convert is_ok-only recorders to classify an actual Result on their
enabled side; is_ok erases suspension/control distinctions today
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:235`, `crates/rum-runtime/src/runtime/definitions/application/down_values.rs:232`).

At a non-suspending finish, verify the exact RecordId at stack top, finish
the executing interval, and publish the site's outcome. At InputRequired,
finish only that interval, remove it from the native stack, increment pause
telemetry and retain the record at its EXACT logical boundary. Never retain
an Instant across host waiting. A marker owns a registry lease; structural
abandon and last-drop fallback discharge that lease idempotently. The
registry must not strongly retain a marker that retains the registry, and
completed open-record entries are removed. Aggregate stats intentionally
survive until reset. Session Drop discharges its owners before the existing
summary prints, under its existing enabled slow path
(`crates/rum-runtime/src/runtime/session/session_state.rs:699`, `crates/rum-runtime/src/runtime/profiling/summary.rs:3`).

The marker reads the current `Result<Slot, EvalError>` by reference and passes
it through unchanged. It never consumes, clones, decodes/re-encodes a Slot,
evaluates guest work, installs a ContinueToken or changes output capture.
Policies are closed and source-specific: ExprOk, OptionalExpr, Bool,
PreparedOk, MatchOk, SomeOnApplicableExprSuccess, or a frozen incoming
outcome. OptionalExpr/Bool can be inspected as State payloads, which are
already their current transports (`crates/rum-runtime/src/runtime/suspension/typed/registry.rs:24`, `:120`,
`crates/rum-runtime/src/runtime/suspension/typed/channel.rs:14`). `set.try_list_destructuring` and
`set.try_association_lvalue` resume to Expr on accepted-target routes, so
their correct Some classification follows the proven applicability, not a
new Option decoder (`crates/rum-runtime/src/runtime/definitions/definitions.rs:461`,
`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:715`). A wrong channel yields a nonzero
instrumentation-fault receipt and invalidates the record, never a guessed
success or a changed guest result. The existing driver is the only consumer
of the raw channel (`crates/rum-runtime/src/runtime/suspension/core.rs:700`, `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:33`).

Every site's attachment is listed in `.temp/brief-624/inventory/routes.tsv:2`. Four
constraints require explicit insertion APIs instead of "attach to last frame":

1. head/RHS/argument/matcher phase markers run BEFORE their consuming frame;
   whole-call markers run AFTER the complete remainder, possibly with no
   frame belonging to the wrapper (`crates/rum-runtime/src/runtime/eval_core/call.rs:46`, `:54`, `:60`).
2. `evaluate.once_unstable` completes AFTER G14's EvaluationReturn resource
   check, but `evaluate_inner_unstable.*` completes before it. Defer the former
   marker until `suspend_evaluation_return` installs that return event, then
   put it before the fixed-point consumer (`crates/rum-runtime/src/runtime/suspension/typed/witness.rs:40`,
   `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs:72`).
3. `.suspend` phases begin after the child already returned InputRequired.
   Capture the producer frontier at their enabled start; close at that input
   outcome before consuming the value, not after Set/While/Compound's later
   work (`crates/rum-runtime/src/runtime/definitions/definitions.rs:41`,
   `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:161`, `:364`).
4. native Module initializer reentry at `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:197` is
   another segment of the original :96 span. An explicit continuation intent
   naming RecordId/route returns a non-owning segment token from the existing
   enabled start; it does not open another call or append another completion
   marker. Do not detect reentry by equality of the phase key.

The registry also retains enabled-only per-cell execution context: record
ancestry and evaluation/rebuild/lexical source scopes. Capture at each
existing cold semantic-frame insertion, keyed by a retained frame identity
and suspension-owner generation, and handle inline return events by their
owner plus event identity. Restore that context before resuming the cell,
excluding already-completed boundary scopes. Capture the initial input
producer's context in begin_input_suspension so parsing/evaluating Input's
reply also runs under its ancestors. Explicitly migrate metadata during
nested splice; never retain raw RefCell borrows or native guards across
execution (`crates/rum-runtime/src/runtime/suspension/core.rs:500`, `:521`, `:564`, `:714`, `:838`;
`crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs:33`). The table stores keys and IDs, not
Expr roots or mutable guest state. Its leases are cleaned by cell pop,
abandonment, epoch retirement and Session Drop; pin frame identities against
address reuse without keeping a reference cycle. This registry work is
enabled and suspension-only.

TLS stacks contain ONLY executing intervals for this native segment,
identified by `(EpochId, RecordId, SegmentId)`. Strictly verify identity and
parent before popping; remove `.filter(|frame| frame.key == token.key)` and
the corresponding builtin/by-source fallback (`crates/rum-runtime/src/runtime/profiling/timing.rs:354`,
`:403`, `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs:141`). An invalid pairing must
produce an instrumentation fault and isolate that record without popping an
unrelated frame. An unrelated request on the same thread cannot inherit
parked native tokens. Resume restores exact saved ancestry; at each yielded
host boundary its active stack is empty. Suspend/restoration is not a fresh
logical entry, and source guards from definitions are restored independently
of G14's semantic stack/depth (`crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs:25`).

Completion routes:

| Route | Record transition and evidence |
|---|---|
| Success | marker sees actual result at listed boundary, publishes once, passes exact Slot (`crates/rum-runtime/src/runtime/suspension/core.rs:817`) |
| Error | error propagates through the same marker; preserve concrete carrier, no default value (`crates/rum-runtime/src/runtime/suspension/core.rs:744`) |
| Return/Throw | inner marker sees control result before the handler; outer marker sees the handler result (`crates/rum-runtime/src/runtime/definitions/application/rhs.rs:124`, `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:122`) |
| Second pause | segment closes; existing outer marker stays in suffix while new owner/frames splice before it (`crates/rum-runtime/src/runtime/suspension/core.rs:838`) |
| Cancel | structural abandon marker retires err+cancelled when reason is CancelledRequest; it cannot resume guest work (`crates/rum-runtime/src/runtime/suspension/core/abandon.rs:45`) |
| Session reset/drop | old owner's open records close as abandoned; no current/new epoch success (`crates/rum-runtime/src/runtime/session/session_state.rs:699`, `crates/rum-runtime/src/runtime/suspension/core.rs:51`) |
| Failed restore/rollback/fault/helper discard | retain the exact AbandonReason; no fabricated return value (`crates/rum-runtime/src/runtime/suspension/core.rs:51`, `crates/rum-runtime/src/runtime/session/session_helpers.rs:243`) |
| Profiling reset inside an active call | retire old epoch; old tokens close only their old identity and cannot underflow new stacks (`crates/rum-runtime/src/runtime/profiling/reset.rs:84`, `crates/rum-runtime/src/runtime/debug_tools/basic.rs:119`) |

Marker abandonment integrates with Entry::abandon, trace_roots,
output_boundary, allocation_bytes, queue widen/append/push-front, and driver
Complete/Evaluate/Finalize branches (`crates/rum-runtime/src/runtime/suspension/typed/entry.rs:21`,
`:29`, `:59`, `:67`; `crates/rum-runtime/src/runtime/suspension/core/queue.rs:44`, `:65`;
`crates/rum-runtime/src/runtime/suspension/core.rs:785`, `:817`). The marker has no guest roots. Queue bytes
remain accounted using existing storage_bytes/projected_growth; enabled
marker/registry allocations need explicit bounded diagnostic storage accounting.
They must not turn an otherwise identical guest resource result into an error
solely because the resumed mode has profiler markers. Track this memory for
the host and enforce a separate diagnostic cap; exhausting that cap invalidates
the profiling epoch visibly and retires its leases without changing the guest
result. Do not silently drop receipts or claim parity for an invalid epoch
(`crates/rum-runtime/src/runtime/suspension/core/queue.rs:145`). No new semantic frame is necessary; if an
implementation instead derives a new frame/root type, TEMPLATE's closed-list
and retention duties apply.

Alternatives, including why this choice wins:

| Mechanism | Comparison |
|---|---|
| Option<Arc<Record>> on existing frames | Straight ownership but each frame may carry several ancestors, and the outer call is not the head-consumer input. Requires an exhaustive field/finish ripple. Measured actual head frame grows 8 to 16 bytes, actual zero-sized RHS frame 0 to 8, and queue-cell analogue 40 to 48. Rejected for default suspension storage (`.temp/brief-624/profile/layout.log`, `crates/rum-runtime/src/runtime/definitions/application/rhs.rs:135`, `crates/rum-runtime/src/runtime/eval_core/call.rs:373`). |
| Side table keyed only by frame/request | No frame bytes, but request identity cannot distinguish intermediate Some/None, Return handling, head versus whole call, or no-frame wrappers. A frame address alone is also reusable. It would need an ordered boundary tape synchronized with every queue edit. Rejected without explicit markers (`crates/rum-runtime/src/runtime/eval_core/call.rs:331`, `crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs:27`). |
| Keep TLS tokens across host wait / reopen by key | Mixes unrelated requests, adds host wait to duration and pairs recursive equal-key calls incorrectly; current fallback already hides mispairing. Rejected (`crates/rum-runtime/src/runtime/profiling/timing.rs:115`, `:403`). |
| Re-evaluate/replay or patch final counts arithmetically | Can repeat effects and invent unexecuted success or fixed-point phases. Rejected; source already performs resumed work (`crates/rum-runtime/src/runtime/resolution.rs:399`, `.temp/brief-624/correctness/summary.tsv:2`). |
| Registry plus inline queue boundaries (chosen) | Exact before/after ownership for every carrier without data decoding or semantic-frame fields. Projected Entry/Queued sizes remain 24/40; enabled-only registry/context work is more complex and needs explicit lifecycle and cost gates (`.temp/brief-624/profile/layout.log`, `.temp/brief-624/inventory/routes.tsv:2`). |

## 5. The disabled-path cost plan and per-step measurement

The existing disabled start/finish path short-circuits on the configured
AtomicU8 relaxed load before the active flag/TLS/clock (`crates/rum-runtime/src/runtime/profiling/config.rs:4`,
`:10`; `crates/rum-runtime/src/runtime/profiling/timing.rs:97`, `:375`). Keep that exact gate budget. Existing
recorders become gated Session methods or statically specialized helpers;
do not add `if token.is_some()` call-site branches, pre-gate capture
construction, Arc clones, TLS access, Mutex acquisition or per-evaluate
lookup. All new resume/cell lifecycle hooks are inside paths that already
require InputRequired; combine Session Drop cleanup with its existing
enabled summary gate. New resumed-only mirrors can add gates on the resumed
path, but the ordinary no-input labels must not gain loads. This is a
design constraint, NOT a measured claim of an implementation.

For each step, the director supplies base and candidate filesystem exports;
the executing agent does not use version control. Build both arms OFFLINE
from the same source paths into distinct target directories with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`. Preserve exact label statements and
N/2N sources; no sink assignments or extra Print. All subprocesses are
bounded. Run each campaign serially without other CPU-heavy jobs. Measure:

* One-codegen-unit instruction slopes `(I(2N)-I(N))/N` with stdin /dev/null,
  pinned CPU, and `perf stat -x, -e cpu_core/instructions/u`. Use the SAME
  192 ordinary labels and 121 resume labels copied in
  `.temp/brief-624/performance/labels.json:1`, plus any new profiler lifecycle controls.
  Start with one paired sample; at most one confirmation for a flagged row.
  No regression on an ordinary row or same-tier neighbor may be waved away
  as noise; attribute or report it. If perf is unavailable, say so and leave
  that required gate unpassed, without elevation or a timing substitute
  (`.temp/brief-622/run-perf.py:58`, `docs/dev/codex/TEMPLATE.md:131`).
* `scripts/core-language-microbench.sh` rum-vs-rum A/B, flags OFF, per-label
  timings.tsv including all target/hot rows and same-tier neighbors. Times
  come from in-kernel AbsoluteTiming, never wall-clock differences. Best of
  two only if needed (`AGENTS.md:28`, `scripts/core-language-microbench.sh:1078`).
* Copy census on all 192 ordinary labels: retain N/2N budgets, copy counts,
  requested bytes, clone/rebuild attribution and per-iteration slopes from
  the existing brief-622 reducer. Its budget is min(original N, 2000,
  floor(20M/base instruction slope)); disable debuginfod/network. No new
  ordinary copy/allocation slope is accepted merely because timings pass
  (`.temp/brief-622/run-copy.py:4`, `:9`, `:24`). Copy the scripts before
  adapting; do not execute them at their old writable paths.
* Debug/release stack fits on the same 22 shapes (44 fits), with the same
  probe and heights from `.temp/brief-624/performance/probe.rs:1` and slopes.json. Preserve
  zero-residual fits, no ordinary growth; inspect changed hot-function
  prologues and complete live helper chains. The prior campaign validated
  528 probes; those are historical counts, not results of this brief
  (`.temp/brief-622/stack.py:28`).
* Compiler layout gate: every existing frame, Entry, Queued<Entry>,
  FrameQueue, ResumeContext, SuspendedEvaluation, Session and profiling-token
  stack slots; unchanged ordinary frame sizes are required. The current
  diagnostic projection is supporting evidence only. Check enabled
  registry/marker allocation/drop balance and constant-space disabled
  registry (no entries/allocations).

LP1 establishes these baselines and may reuse a candidate arm only if its
production-source digest is identical. LP2-LP7 each rerun all named gates
after their own changes. LP8 does a final broad A/B if it changes any hot
recorder; otherwise it reuses the exact LP7 production binary digests and
still checks snapshot/lifecycle costs. Missing measurements are not green.
Budgets are calibrated from retained runs of the same shape; defaults are
not invented. Each full enabled oracle build must fit the existing
1500-second ceiling (`.temp/brief-621/MANIFEST-DESIGN.md:275`). No instruction,
copy, stack, candidate timing or enabled allocation campaign ran in this
design-only task.

## 6. The oracle and mutants

The scratch port preserves all 1115 frozen requirement rows, including
duplicates in different obligation/variant/route cells, in
`.temp/brief-624/profile/frozen-required-tuples.tsv:1` and `.temp/brief-624/profile/tuple-port.tsv:1`. Of these,
813 bind 650 saved runnable programs and were re-executed against the current
source copy. The other 302 had no saved executable plan in plans.json:
240 guest-head requirements (184 saved encoding receipts, 56 explicitly
encoding-blocked) and 62 pending non-encoding error/cycle requirements with
empty probe bindings. These do not become executed or green receipts. LP1
must supply concrete source/route-bound plans for all 62, preserving their
required keys; each must prove its actual designated producer. Original
contexts/variants/route provenance remain mandatory even if one program
can serve several keys (`.temp/brief-623/continuation2/profile-required-tuples.tsv:1`,
`.temp/brief-621/MANIFEST-DESIGN.md:100`). No new internal encodings are
executed, and the 240 retain their existing owner/classification.

Port all 16 minimized witnesses as independent own-row regression fixtures,
not just deduplicated final expressions (`.temp/brief-624/profile/minima-plans.jsonl:1`,
`.temp/brief-624/profile/frozen-16-losses.json:1`). Add the 48 correctness fixtures with trace
off/on and both pause counts. Require full values/effects/ordered trace,
actual requests/kinds/prompts/ordinals, prefix/capture, queue sentinel,
fresh request and zero continuation-fault delta. Counter reset is native,
after setup and before subject; snapshot is before queue/log/projections
(`.temp/brief-624/profile/src/main.rs:48`, `.temp/brief-624/oracle/support/drive.rs:31`).

Extend the actual comparison to exhaustive schema-v2 logical tables, not
the legacy top-64 text or all-stats Debug ordering. Preserve source attribution,
definition index and argument count. Run independently with lexical-only,
lexical+builtin+definition+RHS, lexical+evaluation-cache, and all detail/source
flags; profiling disabled must have empty timing tables. Test repeated keys,
same builtin/arity in recursion, nested sessions on one thread, interleaved
paused sessions, moved Session across threads, errors, Return/Throw before
and after handler boundaries, decline, second pause in the same phase and
second pause in a later operation. Add native fixed-point resource/cycle/
pass-limit returns, because `evaluate.once_unstable` must see the result
AFTER EvaluationReturn (`crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs:86`).

Lifecycle receipts separately assert zero open records/leases/native stacks
after cancel, Session reset/drop, failed restore, explicit helper discard,
checkpoint rollback and profile reset. They must assert no suffix effects
and no success fabricated. Exercise cancellation/reset at the first and
applicable second pause; run cleanup that itself pauses and is abandoned
(`crates/rum-runtime/src/runtime/suspension/core/abandon.rs:56`, `.temp/brief-621/MANIFEST-DESIGN.md:62`).

In diagnostic copies, one compiling own mutant per adopted phase/site/route
suppresses exactly its logical completion ON RESUME while keeping
computation, Slot, other phase counters and pause telemetry intact. Terminal
cleanup still retires the lease, so this is a counter-only mutant. An
attributed receipt must be red with value/effect/trace equality disabled and
durations/pause counts excluded. A queue/registry ID proves it hit the same
record; a source-row guard alone does not prove the second-pause route.
Mutate Some/None and Return/Throw classification separately, duplicate one
resume completion, omit a resumed stability/equality/trace hook, omit source
restoration, corrupt token identity between equal keys, and omit reset/drop
cleanup. Each must fail its corresponding exact tuple; a different row's
mutation or a surviving equivalent mutation is not credit. Current base
losses cannot serve as killed-candidate-mutant evidence
(`.temp/brief-621/MANIFEST-DESIGN.md:118`, `:126`).

Join `phase_observation.counters` to the obligation-keyed manifest as the
counter half alongside ordered trace. Required keys are
`(ObligationKey, phase-site identity, variant, leaf-context, consumer-route,
 flag/build mode)`, with applicability derived from source inventory BEFORE
fixture generation. Keep required/bound/executed collections and multiplicity
one. Share program execution only across predeclared actually-attributed
cells; preserve all consumer routes. An active unbound/unexecuted tuple,
missing phase key, unexpected DIFF, stale ownership or duplicate is red
(`.temp/brief-621/MANIFEST-DESIGN.md:96`, `:110`, `:132`).

Each adoption brief promotes only its exact phase cells. Keep all remaining
losses pinned with named open owners, and require already-adopted cells to
remain green. Do not require the whole legacy snapshot to be green halfway
through this sequence, and do not hide a changed old loss. Final LP8 promotes
the complete non-encoding counter half only after every active tuple passes
and every required own mutant is red. C2/S10 cannot close before then
(`.temp/brief-621/MANIFEST-DESIGN.md:174`,
`.temp/brief-623/continuation2/draft-G15-C2-diagnostic-outcome-completion.md:7`).

## 7. The steps and drafted briefs

Eight raw briefs were written in TEMPLATE.md shape, each with Goal, Context
pointers, explicit Scope, Deliverable, pitfalls, mandatory verification,
checkpointing, hard rules and report requirements. They are director review
material; no implementation was dispatched. All preserve the no-version-control,
no-network, no-new-unsafe rules and the standing generated-inventory duties
(`docs/dev/codex/TEMPLATE.md:22`,
`.temp/brief-624/draft_briefs.py:1`, `.temp/brief-624/draft-index.json:1`).

Verbatim drafted-brief file list (`.temp/brief-624/draft-files-output.txt:1`):

```text
.temp/brief-624/drafts/LP01-contract-and-oracle-port.md
.temp/brief-624/drafts/LP02-record-epochs-queue-lifecycle.md
.temp/brief-624/drafts/LP03-evaluator-dispatch-arguments.md
.temp/brief-624/drafts/LP04-definition-rhs-matching.md
.temp/brief-624/drafts/LP05-assignment-phase-boundaries.md
.temp/brief-624/drafts/LP06-builtins-control-functions.md
.temp/brief-624/drafts/LP07-module-and-source-aggregation.md
.temp/brief-624/drafts/LP08-snapshot-manifest-promotion.md
```

**LP01** — Define the approved logical/telemetry/epoch contract and port a loss-pinned exhaustive counter oracle before changing production counting. Preserve all 1115 frozen tuples; make every executable non-encoding tuple concrete and reviewable, and establish comparable disabled baselines.

Functions: manifest requirements/bindings/receipts; observe/drive; native exhaustive snapshot/reset observer. The source locations and route qualifications are pinned in .temp/brief-624/inventory/routes.tsv; the exact editable files and gates are in `.temp/brief-624/drafts/LP01-contract-and-oracle-port.md:1`.

Files:

- `crates/rum-stdlib/tests/suspension_oracle/manifest.rs`
- `crates/rum-stdlib/tests/suspension_oracle/descriptors.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/mod.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/observe.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs (new)`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile.rs (new)`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-required.tsv (new)`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-bindings.tsv (new)`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json (new)`
- `crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs (new)`
- `crates/rum-runtime/src/runtime/profiling/mod.rs (only feature/test observer wiring)`
- `crates/rum-runtime/src/runtime/mod.rs (only hidden feature/test observer forwarding)`
- `crates/rum-runtime/src/lib.rs (only hidden feature/test observer forwarding)`
- `docs/guide/profiling.md (new, approved contract plus explicit staged known losses)`
- `docs/guide/README.md (link only)`

Gate: The existing unmodified build must reproduce the exact 16 minima, keep all semantic receipts green, and reject every malformed requirement/receipt control. All active non-encoding required keys must have runnable, attributed bindings; the 62 blanks cannot remain a claimed complete port. No logical completion behavior changes in this step.

**LP02** — Build the enabled-only record/segment engine and raw-Slot-preserving inline queue markers, including portable source/execution context and every termination route, before adopting production phases. Existing phase accounting remains on its legacy path until each later migration.

Functions: runtime_path_timing_start/record slow paths; Entry boundaries; queue widen/append; drive_resuming_frames; abandon; Drop; epoch reset/context restore. The source locations and route qualifications are pinned in .temp/brief-624/inventory/routes.tsv; the exact editable files and gates are in `.temp/brief-624/drafts/LP02-record-epochs-queue-lifecycle.md:1`.

Files:

- `crates/rum-runtime/src/runtime/profiling/mod.rs`
- `crates/rum-runtime/src/runtime/profiling/timing.rs`
- `crates/rum-runtime/src/runtime/profiling/state.rs`
- `crates/rum-runtime/src/runtime/profiling/reset.rs`
- `crates/rum-runtime/src/runtime/profiling/summary.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/{mod,record,epoch,segments,context}.rs (new)`
- `crates/rum-runtime/src/runtime/profiling/tests/{logical_contract,lifecycle,identity,layout}.rs (new)`
- `crates/rum-runtime/src/runtime/suspension/typed/entry.rs`
- `crates/rum-runtime/src/runtime/suspension/core/queue.rs`
- `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs`
- `crates/rum-runtime/src/runtime/suspension/core/abandon.rs`
- `crates/rum-runtime/src/runtime/suspension/core.rs`
- `crates/rum-runtime/src/runtime/model/request_cancellation.rs (enabled-only identity accessor)`
- `crates/rum-runtime/src/runtime/session/session_state.rs (Drop cleanup through existing gate)`
- `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs (cold context save/restore helpers)`
- `crates/rum-host/tests/logical_profile_lifecycle.rs (new test-only)`

Gate: Require exact Slot identity through markers, every terminal route once, zero open records/context leases at teardown, repeated-key identity rejection, interleaved/moved-session correctness, profile reset mid-call and cleanup re-suspension. Measured production Entry/Queued/frame/Session layouts must match the existing baseline (projected scratch sizes were only 24/40). All named disabled-path gates run here; no source-level free-path assertion substitutes for them.

**LP03** — Adopt all evaluator/fixed-point/head/dispatch/argument logical phases, observe actual resumed fixed-point work, and retain exact optional/prepared-result boundaries while leaving unrelated builtin/definition/scoping losses explicitly owned.

Functions: evaluate/evaluate_profiled_fixed_point/resume_fixed_point_from; evaluate_inner_unstable/evaluate_atom_fast_path; evaluate_call/resume_call_*; prepare_arguments*/resume_suspended_call*. The source locations and route qualifications are pinned in .temp/brief-624/inventory/routes.tsv; the exact editable files and gates are in `.temp/brief-624/drafts/LP03-evaluator-dispatch-arguments.md:1`.

Files:

- `crates/rum-runtime/src/runtime/resolution.rs`
- `crates/rum-runtime/src/runtime/resolution/resume_frames.rs`
- `crates/rum-runtime/src/runtime/eval_core/mod.rs`
- `crates/rum-runtime/src/runtime/eval_core/call.rs`
- `crates/rum-runtime/src/runtime/eval_call_prepare.rs`
- `crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/{mod,record,segments,context}.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs (new)`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`

Gate: All evaluator/dispatch/argument tuples must match at their specific keys; values/effects/trace/lifecycle must stay green on all 48 correctness programs and the full current oracle. Test resource/pass/cycle failure after resume, held/unheld heads, None decline, prepared restores and second pauses. Run the full per-step measurements; inspect ordinary recursion stack and helper-chain changes, not only source frame sizes.

**LP04** — Adopt definition RHS/compiled-matcher/direct-default phases and all eight definition-rule timing starts, preserving source/kind/symbol/index and pre-/post-Return boundaries with scalar/by-source aliases finished once.

Functions: evaluate_definition_rhs/evaluate_definition_application_rhs/finish_definition_rhs_evaluation; Down/Up/Sub application arms; matcher/default finishes; scalar/by-source RHS records. The source locations and route qualifications are pinned in .temp/brief-624/inventory/routes.tsv; the exact editable files and gates are in `.temp/brief-624/drafts/LP04-definition-rhs-matching.md:1`.

Files:

- `crates/rum-runtime/src/runtime/definitions/application/rhs.rs`
- `crates/rum-runtime/src/runtime/definitions/application/down_values.rs`
- `crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs`
- `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs`
- `crates/rum-runtime/src/runtime/definitions/application/up_values.rs`
- `crates/rum-runtime/src/runtime/profiling/timing.rs`
- `crates/rum-runtime/src/runtime/profiling/records.rs`
- `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/definitions.rs (new)`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`

Gate: P0256/P0259/P0266/P0314/P0315 and all definition/direct-default/matcher source routes become green in their exact adopted phase fields. Test guarded match decline and later success, Return/Throw/error from RHS, no-match after input, exact definition index and second pause. Run own per-phase completion mutants and all default/no-default semantic, maintenance and cost gates.

**LP05** — Adopt Set RHS/forwarded-boundary/finish/list/association/LHS phases, including the set.finish phase not yet entered when RHS paused, without changing assignment semantics or accepted-target result channels.

Functions: eval_set/resume_set_with_rhs/finish_definition_assignment; LHS preparation continuation/store; association/destructuring remainder completions. The source locations and route qualifications are pinned in .temp/brief-624/inventory/routes.tsv; the exact editable files and gates are in `.temp/brief-624/drafts/LP05-assignment-phase-boundaries.md:1`.

Files:

- `crates/rum-runtime/src/runtime/definitions/definitions.rs`
- `crates/rum-runtime/src/runtime/definitions/definitions/resume_frames.rs`
- `crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs`
- `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`
- `crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/assignments.rs (new)`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`

Gate: P0413/P0431 and all assignment routes become green for adopted fields; stores, held targets, multi-selector association updates, destructuring order and concrete recovery stay unchanged. Exercise RHS pause then LHS pause in the same operation. Run full measurements, especially lhs-set-*, set-ownvalue, association-set/part, append and their neighbors; no extra ordinary ownvalue gate/copy/stack cost.

**LP06** — Adopt builtin handler timing and control/callable phases, recording inner control outcomes before handlers and preserving Catch incoming-outcome observations while completing outer logical calls once.

Functions: try_dispatch_builtin_call/finish_builtin_dispatch; resume_while_loop/resume_compound_expression*; eval_catch/resume_catch*; try_apply_function/evaluate_status_function_value. The source locations and route qualifications are pinned in .temp/brief-624/inventory/routes.tsv; the exact editable files and gates are in `.temp/brief-624/drafts/LP06-builtins-control-functions.md:1`.

Files:

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs`
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs`
- `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs`
- `crates/rum-runtime/src/runtime/scoping/callable.rs`
- `crates/rum-runtime/src/runtime/scoping/callable/status_application.rs`
- `crates/rum-runtime/src/runtime/profiling/timing.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/control.rs (new)`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`

Gate: All builtin/control/function adopted tuples and caught/noncaught control routes become green, with actual None/Some and error/Return/Throw outcomes. Run exact source counter mutants, catch-handler second pause and status projection-then-value pause; keep ordered trace and capture ownership green. Full per-step gates include catch/withcleanup/checkabort/abortprotect/function/status families plus all ordinary controls.

**LP07** — Adopt Module total/body/initializer and one logical initializer-loop record across native reentries; finish all scalar/by-source aliases and close remaining profile completion-route gaps without hiding physical telemetry.

Functions: eval_module/resume_module_initializers/resume_suspended_module_initializers/finish_module_scope; eval_module/by-source aggregates; source context and remaining logical entry counters. The source locations and route qualifications are pinned in .temp/brief-624/inventory/routes.tsv; the exact editable files and gates are in `.temp/brief-624/drafts/LP07-module-and-source-aggregation.md:1`.

Files:

- `crates/rum-runtime/src/runtime/scoping/lexical/module.rs`
- `crates/rum-runtime/src/runtime/profiling/records.rs`
- `crates/rum-runtime/src/runtime/profiling/counters.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/{mod,record,context,segments}.rs`
- `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs`
- `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/module.rs (new)`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`

Gate: P1507 and all Module/source-attribution tuples green; every primary pause-capable phase/route now has its real completion and own red mutant. Require multi-initializer then body pauses, nested Modules/Block, source flags on/off, cleanup/error and gensym lifetime checks. Full default/no-default, inventory replay, 192/121 instruction labels, ordinary copy census, stack and timing gates.

**LP08** — Expose schema-v2 exhaustive logical counts separately from execution/duration telemetry and activate the complete non-encoding phase_observation counter half only when full matrices, lifecycle and own mutants are green.

Functions: print_lexical_profile_snapshot/print_runtime_path_timing_summary/print_builtin_timing_summary; eval_rum_profile_snapshot/reset; exhaustive manifest promotion. The source locations and route qualifications are pinned in .temp/brief-624/inventory/routes.tsv; the exact editable files and gates are in `.temp/brief-624/drafts/LP08-snapshot-manifest-promotion.md:1`.

Files:

- `crates/rum-runtime/src/runtime/profiling/summary.rs`
- `crates/rum-runtime/src/runtime/profiling/report_timing.rs`
- `crates/rum-runtime/src/runtime/profiling/report_rebuild.rs`
- `crates/rum-runtime/src/runtime/profiling/state.rs`
- `crates/rum-runtime/src/runtime/profiling/reset.rs`
- `crates/rum-runtime/src/runtime/debug_tools/basic.rs`
- `crates/rum-runtime/src/runtime/evaluation_cache/profile/report.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/snapshot.rs (new)`
- `crates/rum-stdlib/tests/suspension_oracle/manifest.rs`
- `crates/rum-stdlib/tests/suspension_oracle/descriptors.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-required.tsv`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-bindings.tsv`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`
- `crates/rum-host/tests/logical_profile_lifecycle.rs`
- `docs/guide/profiling.md`

Gate: Full real candidate logical parity, every own mutant red, zero open records/leaks/instrumentation faults, all schema/collection negative controls and both oracle builds within 1500 seconds. Run full workspace/maintenance checks. If only reporting/test/doc code changed, reuse the identical LP7 hot production binary digests for instruction/copy/stack results and show why; if any hot recorder changes, rerun the full per-step A/B. State every gate that did not run.

Each step's tests include bounded offline workspace/default and no-default
suites, the relevant suspension/lifecycle tests, source-inventory replay and
the cost measurements in section 5. LP01 establishes baselines and genuine
bindings; LP02 tests the engine before adoption; LP03-LP07 promote only their
specific phase cells while remaining losses keep exact named owners. LP08
requires complete active non-encoding parity and authentic own mutants before
counter-manifest promotion; it does not settle unrelated G13 or semantic
obligations (`.temp/brief-624/drafts/LP01-contract-and-oracle-port.md:1`,
`.temp/brief-624/drafts/LP08-snapshot-manifest-promotion.md:1`).

## 8. Open questions and recommendations

1. **Performance is unmeasured for the proposed engine.** Accept the layout
   projection as feasibility only; require all per-step gates above before
   claiming the disabled path is free (`.temp/brief-624/profile/layout.log`,
   `docs/dev/codex/TEMPLATE.md:131`).
2. **62 required non-encoding tuples are presently unbound.** Make filling
   them LP1's first oracle task; never shrink the manifest. Keep all 240
   encoding-owned requirements separately visible (`.temp/brief-624/profile/tuple-port.tsv:1`).
3. **Diagnostic storage accounting.** Recommend a separate bounded, host-visible
   diagnostic category, including attributable queue-capacity growth, so guest
   limits cannot change solely due to profiling markers. LP2 must establish
   that accounting invariant before adoption into memory-limited requests;
   if it cannot, use the external boundary-tape alternative rather than
   shipping an unexplained resource-semantic difference (`crates/rum-runtime/src/runtime/suspension/core/queue.rs:145`).
4. **Snapshot compatibility and global epoch scope.** Adopt additive schema
   v2 exhaustive logical rows and explicitly labelled telemetry; retain old
   ranked displays for humans. Process-wide logical epochs are recommended
   for portable records; document the isolated-epoch comparison boundary
   (`crates/rum-runtime/src/runtime/profiling/state.rs:33`, `:81`, `crates/rum-runtime/src/runtime/profiling/report_timing.rs:188`).
5. **No new semantic repair is justified by this question.** Source and the
   executed 48/48 semantic panel agree; first brief is the .temp/brief-624/oracle/contract
   port, not a fabricated G15 semantic change (`.temp/brief-624/correctness/summary.tsv:2`,
   `crates/rum-runtime/src/runtime/resolution.rs:399`). New semantic DIFFs in added routes must become a
   minimized prerequisite repair before their profiler adoption proceeds.

**What did not run.** No production implementation, candidate/default-path
instruction or timing A/B, copy census, stack campaign, full workspace unit
test suite, candidate lifecycle/epoch implementation test, or production
resume-completion mutant ran in this design-only brief. No new internal-
encoding execution ran. The 62 unbound non-encoding frozen keys and 240
encoding-owned requirements were preserved, not represented as executed
successes. Current executed evidence is the 48 semantic cases, 16 minima,
650 saved programs (813 tuple bindings), and two one-witness layout probes
with offline scratch builds. The counter-panel budget was three enabled
panels (16/48/650), plus the initial disabled semantic panel and two single-
witness layout experiments; there was no open-ended probe loop
(`.temp/brief-624/profile/run-summary.txt:1`,
`.temp/brief-624/profile/layout.log`,
`.temp/brief-624/profile/frame-layout.log`,
`.temp/brief-624/PROGRESS.md:1`).

Stop-loss did not fire: correctness, inventory, mechanism/cost evidence and
drafts all advanced, with checkpoints after each. The scratch privacy compile
failure and its fix are retained. Entry hashes verify every non-report file
unchanged; final-verification.json records append-only preservation and the
audit result. The director retains ownership of source implementation,
the authoritative gate and version control.

## Director review (2026-10-20) — accepted; LP01-LP08, one step per brief

**Run.** One run (session `01a154e7-…`, 395k tokens). There was no filter or
capacity interruption.

**Accepted.**

- **The correctness verdict: there is no semantic loss.** The resumed
  fixed-point loop (`resolution.rs:399-424`) performs the same comparisons,
  stability queries and trace updates as the ordinary one
  (`:341-392`). It does so without the profiling hooks. Executed evidence:
  48/48 SAME, with tracing on and off, direct, held and unheld, and one and
  two pauses. No G15-class repair follows from this question.
- **The contract.**
  - Logical `(calls, ok, err, some, none, return_err, throw_err)` are equal
    per key between the queued and resumed runs of an isolated program.
  - `InputRequired` stops being an `err` and becomes pause telemetry. A
    paused record is `open`, with no outcome.
  - Cancellation and abandonment have their own accounting, never `ok`.
  - Incoming-outcome phases keep their sampling meaning (P0817/P0818).
  - Durations, pauses and segments are reported and never compared.
- **The mechanism.**
  - An enabled-only profiler-side registry of logical records, keyed by
    epoch, owner and record identity, never by phase key or address.
  - Inline `ProfileBoundary` entries in the existing mixed frame queue,
    inserted only when profiling is enabled *and* a record suspends.
  - No field on any existing frame. The projected queue entry size is
    unchanged at 24/40 B; that is an implementation gate, not yet a fact.
  - The thread-local token stack holds only executing intervals with
    verified identity. The silent `.filter(frame.key == token.key)`
    mis-pairing is removed, and an invalid pairing becomes a visible
    instrumentation fault.
- **The eight-step plan** (`.temp/brief-624/drafts/LP01-…LP08-….md`).
  Each step lands green and measured, and only LP03-LP07 change counting.

**Rulings on the open questions.**

1. **Performance.** The layout projection is feasibility evidence only.
   Every step that changes production code runs the full per-step
   measurement set (instruction slopes, copies, allocations, stack shapes,
   layouts, timing). A step that changes no default-build code proves it by
   identical release-binary hashes at identical paths.
2. **The 62 unbound non-encoding tuples.** LP01 binds them before
   anything else. The manifest never shrinks.
3. **Diagnostic storage.**
   - The profiler's enabled-path memory is a separate, bounded diagnostic
     category. It never counts against a guest's memory ceiling, so
     enabling profiling cannot change a guest's resource outcome.
   - Exhausting the diagnostic cap invalidates the profiling epoch
     visibly.
   - If LP02 cannot establish this invariant, it STOPs and reports. It
     does not switch mechanisms on its own.
4. **The snapshot.** It moves to an additive schema v2 (exhaustive logical
   rows, labelled telemetry) and keeps the ranked human display. Logical
   epochs are process-wide, and the isolated-epoch comparison boundary is
   documented.
5. **The contract's document** lives in `docs/dev/profiling.md`. The
   profiler is a developer diagnostic, so it does not go in the user guide.

**Sequence.**

- LP01 is brief-625: the contract and the oracle port, with no
  counting-behaviour change.
- LP02-LP08 follow, one per brief.
- Then come C2's remaining 353 rows on the final source, the manifest,
  `CHANGELOG.md`, and S10.
