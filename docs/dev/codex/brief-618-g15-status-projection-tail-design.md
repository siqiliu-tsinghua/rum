# brief-618 — design: G15, a status-object call resumes past the Function stage's tail

`Scope: NONE — no production change.` This brief is design-only, with one
declared scratch test (see Scope). **Do not run `git` in any form**,
including before you have read this brief.

## Goal

A `Failure` object applied to a property, `Failure[tag, assoc]["Message"]`,
is dispatched in the evaluator's Function stage. When the property's
rendering pauses for input (a delayed `MessageParameters`, or a delayed
message definition), the resumed path skips work that the queued-input path
does:

- **The tail is lost.** The ordinary path evaluates the projected value and
  absorbs a `Return` into a value. After a pause, a `Return` in the projected
  value escapes the enclosing body instead.
- **The value reaches a slot of a different type.** The projected `Expr`
  lands in a slot that holds an `Option<Expr>`. The legacy decoder for that
  slot reads a guest-written `System`InternalContinuationNone[]` or
  `InternalContinuationSome[…]` as state.

The director found this on 2026-10-18, while preparing G13 step S10. **No
census found it.** brief-548 named this exact mechanism, and the E1 oracle
reaches the frame. The design must settle the repair of the **class**, and
explain why the census and the oracle missed it, so that the same blind spot
cannot hide another instance.

This is a branch inserted into G13. S1-S9 are done (S9 is `f4a7c6b0`; the
tree is `72abbfa7`). S10 (confirmation and status properties) waits for this
branch.

### The mechanism, as far as the director has read it

All paths are under `crates/rum-runtime/src/runtime/`.

- **The call site.** `try_apply_function` (`scoping/callable.rs:5`) tries
  each callable-head kind in turn. The status branch is
  `else if let Some(status) = self.apply_status_object_to_args(head, args)? { status }`
  (`:33`). It has no frame.
- **The tail it skips.** Every branch falls through to a common tail:
  - `self.evaluate(applied)` (`:60`), whose own suspension inserts
    `TryApplyFunctionSuspensionFrame` (`:69`, row F0528, owned by S13);
  - `finish_function_application(outcome).map(Some)` (`:74`). That function
    turns `Err(Return { value })` into `Ok(value)` (`:78-91`).
- **The producers below.**
  - `apply_status_object_to_args` (`kernel_data/control_data/status.rs:71`)
    returns `Ok(Some(self.status_object_projection_value(head, property, None)?))`
    (`:103`).
  - `status_object_projection_value` (`:109`) calls
    `failure_virtual_message_value(status)?` (`:135`) and
    `failure_virtual_styled_message_value` (`:196`).
  - `failure_message_from_template` (`:211`) inserts
    `FailureMessageFromTemplateSuspensionFrame` (F0345, `:236`).
    `finish_failure_message_from_template` (`:250`) inserts
    `FinishFailureMessageFromTemplateSuspensionFrame` (F0346, `:276`). The
    styled path inserts `FailureVirtualStyledMessageValueSuspensionFrame`
    (F0344, `:204`).
  - Each of these legacy frames completes with an `Expr`.
- **The consumer.** The stage loop (`eval_core/call.rs:174`) suspends into
  `ResumeCallWithDispatchPlanSuspensionFrame`. S3 typed it, and its `Input`
  is `Option<Expr>` (`:430-432`). `Option<Expr>` is `LegacyMode`
  (`suspension/typed/registry.rs:24`). Its decoder
  (`typed/legacy.rs:6-13` → `suspension/primitive.rs:36-66`) maps any
  unrecognised `Expr` to `Some(other)` (`:63`), and maps a guest-written
  internal head to state.
- **The same helpers have three consumers.** `failure_property_value`
  (`status.rs:63`) reaches the same projection from `Enclose`'s string
  handler (`confirmation/builtins.rs:267`) and from `Confirm` on a `Failure`
  value (`:303`, frame F0326, whose input is the message `Expr`). Only the
  Function-stage consumer expects an `Option<Expr>`.

### Witnesses

The director's harness, `.temp/director/depth-leak-probe/`, is a rum-host
program that runs each program through the resumed path and again with its
input queued. It has a large pause cap, and it answers `"1"` to every
prompt. Its release binary was built against this tree, and the director
confirmed it fresh: a no-op rebuild recompiled nothing. Its arguments are the
pause cap, then setup/source pairs.

The witnesses are in `.temp/director/g15-probe/pairs.py`, with their output
in `run1.txt`: **14 pairs, 6 DIFF, 8 SAME, and every one paused.** Here
`W = "MessageParameters" :> (InputString["w"]; {})`.

| Program | Resumed | Queued |
| --- | --- | --- |
| `f[] := (Failure["t", <\|"MessageTemplate" :> Return[3], W\|>]["Message"]; 7); f[]` | `3` | `7` |
| `Do[r = Failure[…Return[3]…]["Message"]; s = 5, {1}]; {r, s}` | `3` | `{3, 5}` |
| `g[] := (Failure[…Return[3]…]["StyledMessage"]; 8); g[]` | `3` | `8` |
| `Failure["t", <\|"MessageTemplate" -> System`InternalContinuationNone[], W\|>]["Message"]` | the call, unevaluated | `InternalContinuationNone[]` |
| the same with `InternalContinuationSome[5]` | `5` | `InternalContinuationSome[5]` |
| the same with `InternalContinuationSome[]` | the call, unevaluated | `InternalContinuationSome[]` |

**The controls pass.** They have held templates `(1 + 1)` and `x`, a
top-level `Return[3]`, a `Throw` caught outside, a `Sequence` spliced into a
list, `"StyledMessage"` on an internal head, and two paths through `Enclose`
and `Confirm`. The director reads this as follows: the evaluator's fixed point
re-evaluates the value that the tail skipped, so only work that the fixed
point cannot redo is visible. Settle whether that reading is right.

The last three rows are the historical q8 rows of round 5
(`.temp/brief-606/divergence-step-gates.tsv:20-22`), assigned to S10 with
the dependency "S3 dispatch output fix precedes each later repair". They
still diverge at `72abbfa7`. The analogous rows for F0528,
`function.{none,some,empty_some}`, are assigned to S13. They diverge too,
because F0528 completes with `finish_function_application(outcome)`, which
has no `Some`.

### Why nothing caught it, as far as the director can see

- **brief-548** states this mechanism as the class: "a Rust function with
  work remaining after a may-suspend call must push a frame … If it pushes
  none — it propagates with a plain `?` — … the remaining work is skipped".
  Its census swept builtins. `try_apply_function` is not a builtin.
- **The E1 oracle** reaches F0345 and F0346 through the descriptors
  `Failure.propertyParameters` and `Failure.propertyTemplate`
  (`crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/options_properties.rs:23-40`).
  Both compare a top-level string. A skipped tail cannot change that string.
- **The typed-slot census** (round 5) recorded F0344-F0346 and the q8 rows,
  but not the caller's missing frame.

## What the design must settle

1. **The diagnosis, with `path:line` evidence.** Trace these witnesses, frame
   by frame, against the queued run: `f[]`, the `InternalContinuationNone[]`
   row, and one control. For each one, give:
   - the queue at the pause;
   - what each frame completes with;
   - where the tail is lost;
   - what the stage loop's frame decodes.

   Say why each control passes.
2. **The class census.** The class is a suspension that propagates out of a
   Rust function that still has work to do after the call, in either of two
   ways:
   - it has no frame: a plain `?`, an early `return`, or a match arm that
     passes `InputRequired` through;
   - its frame completes with a value that skips the caller's tail, or
     that lands in a slot of a different type.

   Derive the census from the code. **Do not key it to a list.** Cover
   `rum-runtime` and `rum-stdlib`, and the evaluator's paths that are not a
   catalog entry's handler in particular:
   - every `CallDispatchStage` producer (`eval_dispatch/mod.rs:102-151`);
   - every branch of `try_apply_function`;
   - call preparation;
   - OwnValue and definition application;
   - the `Part` re-dispatch seam;
   - message rendering;
   - any helper that is reached from consumers of more than one slot type.

   Cross-check against brief-548's census and the current typed-slot census
   (the M rows). For each instance either missed, say why.

   **Execute every candidate rum against rum** (resumed against queued, the
   suspension asserted), and run each one in a context where its tail is
   observable. Report a table:

   | Site | Caller | Skipped tail | Consumer slot type | Probe | Result | Owning G13 step |
   | --- | --- | --- | --- | --- | --- | --- |
3. **What the fixed point hides.** Classify the evaluator's tails:
   - those whose skip the fixed point repairs, because it re-evaluates the
     returned value;
   - those it cannot repair: `Return` absorption, profiling balance,
     `Unevaluated` and `Sequence` handling, held results, and anything else
     you find.

   Then define the **context variants** an oracle descriptor needs, so that
   a skipped tail cannot pass by being redone. One example is wrapping the
   descriptor in `f[] := (…; marker)` with a `Return` in the paused value.
4. **The repair.** Design it for G15 and for each instance the census finds.
   - Each new frame is typed from birth, under S5-S9's rules:
     - the witness is taken at the producer's actual `Result`;
     - the `(witness, request)` pair crosses a named `#[cold]
       #[inline(never)]` helper, never the producer's `Result`;
     - the routes: witnessed, `DefinitionResumeRoute`, or witnessed then
       `ResumeStop`;
     - the token carries no `Arc`;
     - no value is read through a lenient decoder.
   - Say where each frame lives, and what its `Input` and `Output` are,
     for all three consumers of the status helpers.
   - **Prefer a structure the compiler enforces.** Example: can a dispatch
     stage's producers be shaped so that a stage helper cannot propagate a
     suspension past its stage's tail? Weigh that against a frame at each
     site, on correctness and enforceability. Cost does not count.
5. **Interaction with G13.** The rows involved belong to two steps:
   - S10 owns F0344-F0346, M0355-M0357, T0030-T0034 and the q8 promotion
     (`.temp/brief-606/draft-S10.md`).
   - S13 owns F0528, M0563 and the `function.*` rows (`draft-S13.md`).

   Say which rows the repair touches and whether it types any of them. Say
   whether q8 becomes equal at G15 or stays with S10. Say what changes in
   draft-S10 and draft-S13, and in any other draft the census touches. The
   repair must not type or flip another step's rows unless it must. Where it
   must, say why.
6. **The class falsifier.** Extend the E1 oracle with the context variants,
   for the instances found and for the existing descriptors they apply to.
   The oracle's manifest enforces an exact equality between declared entries
   and executed descriptors. Say how that equality can also require each
   applicable context variant, so that a new entry cannot skip them.
7. **Cost.** The ordinary path must not pay: no field, branch, store or copy
   on a call that does not suspend. Say what the resume path costs and how
   the implementation brief measures it:
   - allocations;
   - one-codegen-unit instruction slopes on resume labels and on the
     ordinary labels;
   - the S9 bars: the memcpy census with libc symbolized, and stack shapes
     rerun after any copy fix.

## What I want back

1. The diagnosis of the traced witnesses, with `path:line` evidence.
2. The census table, and why each earlier census missed what it missed.
3. The observation model and the context variants.
4. The repair: frames, types, routes, enforcement, and G13 interaction.
5. The witnesses you ran, with their output.
6. The repair sequence, and drafted implementation brief(s) in `TEMPLATE.md`
   shape. Each one has:
   - red tests through the oracle;
   - a MAY list with a disclosure duty;
   - one-codegen-unit instruction gates;
   - serial runs;
   - a stop-loss keyed on whether the failure count falls.
7. Open questions for the director, each with your recommendation.

## Context pointers

- `docs/dev/codex/brief-548-suspension-continuation-sweep.md` (the class and
  its census method) and brief-586 / brief-599 (the E1 oracle, its catalog
  and its manifest equality).
- brief-606 with `.temp/brief-606/design.md`, `census-spec.md`,
  `divergence-step-gates.tsv`, `transformation-dispositions.tsv`,
  `draft-S10.md` and `draft-S13.md`.
- brief-617 (S9, the latest step) for the typed-frame rules as built, and
  `crates/rum-runtime/src/runtime/suspension/typed/`.
- The director's probes: `.temp/director/g15-probe/` and
  `.temp/director/s10-probe/`. The S10 pre-probe also found seven more
  misread `Option<Expr>` slots on `Confirm`-family information values
  (T0028). They are S10's planned typing, given here as context only.
- `TEMPLATE.md` and `AGENTS.md`.

## Scope

- **Files you MAY modify: NONE,** except these:
  - append your report to this file;
  - scratch under `.temp/brief-618/`;
  - **one** scratch integration test,
    `crates/rum-stdlib/tests/zz_brief618_probe.rs`. It may reuse the
    oracle's support through a `#[path]` module. **Delete it before the
    report is appended**, and copy its final content to
    `.temp/brief-618/probe.rs`.
- You may **execute** the director's probe binary,
  `.temp/director/depth-leak-probe/target/release/director-depth-leak-probe`.
  Do not rebuild it or write under `.temp/director/`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.** No network, and nothing outside this
  repository. All scratch and `TMPDIR` go under `.temp/brief-618/`.
- **Do NOT modify any file outside the exceptions.** STOP and report
  instead.
- **Run no probe that aborts a process.**
- **Commands are serial,** each through `python3 scripts/run-bounded.py`:
  - `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief618_probe`
    (900 s, as for brief-612's probe);
  - the director's probe binary (600 s per invocation).

  Your tool timeout must exceed every bound you pass. You may also read
  files, use `rg`/`grep`, and run `python3`.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum
  only.
- **No production code.** No new `unsafe`.
- **Checkpoint as you go.** After each item, write
  `.temp/brief-618/PROGRESS.md`, and append each finished report section to
  `.temp/brief-618/REPORT.md`. Copy that file into this brief at the end. A
  usage limit cut brief-616 after its last gate and before its report.

## Report format (your final message, appended to this file)

1. What changed: "design only", the scratch files, and the deleted test.
2. The diagnosis.
3. The census, the observation model, and items 4-7 above.
4. The witnesses and their output.
5. The repair sequence and the drafted brief(s).
6. Open questions for the director.

---
