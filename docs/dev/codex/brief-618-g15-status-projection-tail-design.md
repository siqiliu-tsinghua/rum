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


## Execution report — brief 618 (2026-10-18)

### 1. What changed

Design only. Production sources and checked-in inventories are read-only in this execution. Evidence, the code-derived source index, test sources, reports and draft briefs are under `.temp/brief-618/`. The sole temporary integration entry is `crates/rum-stdlib/tests/zz_brief618_probe.rs`; its final contents are preserved as `probe.rs` and it is deleted before this report is appended. No git or network operation was run. No third-party engine was used. No performance result is claimed for this design-only run.

### 2. Diagnosis

Paths in this section are relative to `crates/rum-runtime/src/runtime/` unless explicitly qualified. Queue descriptions are **source-derived**, not a new runtime queue instrumentation facility. The director binary and the scratch integration test independently reproduce the observations. The latter records complete values, errors, termination, messages, prints, trace and `log`, asserts the pause count, checks pending ownership/overlap, and checks an empty input queue and a fresh request.

`scoping/callable.rs:33` propagates projection suspension before reaching `:60` (evaluate the applied expression) and `:74` (absorb Return, then map Some). `kernel_data/control_data/status.rs:103` additionally skips its Some constructor. F0345 (`status/resume_frames.rs:29`) resumes parameters, renders the template, and completes an **Expr**, not an optional dispatch result. F0346 (`:53`) also completes an Expr; F0344 (`:7`) wraps an Expr in Style. The next Function-stage consumer is `eval_core/call.rs:430`, Input `Option<Expr>`, Output `Expr`. Its mode is still Legacy (`suspension/typed/registry.rs:24`); `typed/legacy.rs:6` delegates to `suspension/primitive.rs:36`. Recognized None and Some heads are interpreted as state; unrecognized expressions become Some(other). This is a missing continuation plus an output-boundary error, not a defect in string interpolation.

**Queue at the parameter pause.** Use `D(c)` for the `ResumeCallWithDispatchPlanSuspensionFrame` of call c (Option<Expr> -> Expr), `B(c)` for its `RunCallDispatchStageSuspensionFrame` (Expr -> Option<Expr>), and `E(e)` for an EvaluationReturn marker followed by `EvaluateProfiledFixedPointSuspensionFrame` for e (Expr -> Expr). The queue context restores the evaluation state; the marker checks active value limits (`suspension/core/evaluation_context.rs:72-88`) but does not evaluate guest code or wrap a value. These are individual entries, not one combined frame. Their insertion/consumption is in `eval_dispatch/mod.rs:230,468`, `eval_core/call.rs:174,430`, `resolution.rs:375` and `suspension/core.rs:699,725,737,782`. The input leaf begins the pending queue at `builtins_input.rs:246`; it has no extra InputString continuation. With the simple unguarded `f[]` definition in director.01, the inner-to-outer queue is:

```text
B(InputString), D(InputString), E(InputString),
ResumeCompoundExpressionFromIndex(parameters, index 0),
B(parameters CompoundExpression), D(parameters CompoundExpression), E(parameters),
FailureMessageFromTemplateSuspensionFrame,
D(status property call), E(status property call),
ResumeCompoundExpressionFromIndex(f body, index 0),
B(f body CompoundExpression), D(f body CompoundExpression), E(f body),
EvaluateDefinitionApplicationRhsSuspensionFrame,
D(f[]), E(f[]).
```

The top-level None witness and the Plus control have exactly the same prefix through `E(status property call)` and no f-body/definition suffix. There is **no** projection-to-Function-tail frame and no F0528 in this queue. F0528 is only installed if the later `evaluate(applied)` itself suspends (`callable.rs:65`). CompoundExpression frames and their error propagation are at `kernel_data/control_data/branching_loops.rs:349` and `branching_loops/resume_frames.rs:125`; the definition result frame is at `definitions/application/rhs.rs:153`.

**director.01, f[] with Return[3].** The answer is Expr string `"1"`. B wraps Some("1"), D unwraps it, and E completes "1". The parameter CompoundExpression frame discards that item and completes `{}`; B/D/E preserve the completed empty List. F0345 parses `{}` and calls `finish_failure_message_from_template`; a non-string, non-MessageName template is returned literally at `status.rs:268-269`, so F0345 completes `Return[3]`. D(status) decodes it as Some(Return[3]) and completes Return[3]. E(status) detects a changed value, evaluates Return, and completes `Err(Return { value: 3 })`. The f-body CompoundExpression frame receives that error and skips 7. Its B/D/E entries propagate the error. The definition RHS frame absorbs Return and emits Some(3), D(f) returns 3, E(f) completes 3. In the queued run parameters return synchronously, the projection becomes Some(Return[3]), `callable.rs:60` evaluates it, `:86` absorbs Return *at this inner call*, `:74` produces Some(3), and the enclosing body reaches 7. Observed resumed 3, queued 7.

**director.04, InternalContinuationNone[].** The same prefix produces `{}`. F0345 completes the guest expression `System`InternalContinuationNone[]`. D(status) decodes **None**, advances past Function (`call.rs:340-348`), and reconstructs the original call (`:102-117`, or the suspended twin `:199-215`). E(status) sees that exact original expression and terminates on equality (`resolution.rs:404-405`), so the result is the unevaluated property call. Queued evaluation returns the literal internal-head expression. Some[5] is decoded as Some(5); Some[] has no first item and becomes None. None of these guest values is legitimate continuation state. Wrapping the raw value in the genuine Some at the producer repairs the distinction without changing Option's global mode.

**director.07, held Plus[1,1] control.** F0345 completes Plus[1,1], D(status) decodes Some(Plus[1,1]), and E(status) re-evaluates the returned expression to 2 (`resolution.rs:399-453`). The queued inner Function tail already evaluated it to 2. Both produce 2. This validates the director's masking explanation for this control, not permission to omit that tail.

**Why all eight original controls pass.** Plus and x are normalized by E(status). A top-level Return is converted at the top-level request boundary, hiding the change in which Return boundary handles it. Throw propagates through Function's finish unchanged, so either route reaches the same outer Catch with 4. Sequence reaches the enclosing list's still-present argument-preparation/splice owner. StyledMessage wraps the guest marker in Style before it reaches D(status), so it cannot be mistaken for the outer Option tag. Confirm's message projection is consumed as Expr by F0326 (`confirmation/builtins.rs:303-314`), which constructs the confirmation failure; Enclose's string property handler is an Expr tail (`:259-270`). That path has no Function-stage Option consumer or inner Function Return absorber. The last Enclose control pauses inside an actual handler Function; its F0528 *does* absorb Return and ordinary numeric results survive the lenient optional decoder. Equal output in those contexts does not show that the Function-stage status tail exists.

**Correction to the historical explanation.** brief-548 did index the non-catalog caller: `.temp/brief-548/census.tsv` P1306 is `try_apply_function -> apply_status_object_to_args` at callable.rs:33, marked **Unsettled/U1**, with `<shared evaluator/frame/host or unregistered helper>` as its reach. P1307 called the later evaluate/F0528 path FrameCarries without checking its missing Some against the outer slot. Thus the class was named and the edge was recorded, but it was never settled/probed as a non-catalog caller obligation. “Not found by the census” is accurate for detection as a defect, not for source enumeration. The E1 manifest has only callPreparation and defaultFallback as shared evaluator entries today; top-level string descriptors for Failure cannot falsify Return absorption or raw Option-head confusion.

### 3. Code-derived census and observation model

#### Population and method

The source population came first: the scratch test enumerated both crates' Rust source trees and reused the existing **syntax walker**, not the frame/handler lists, from `typed_slot_census/scan.rs`. It saved function identities, signatures, complete tokenized bodies, call/method/macro uses, and every `?` operand. The original tables and director programs were cross-checks only after that scan.

```text
files=1084 production_functions=7565 production_uses=154891 unresolved=197
```

`functions.tsv`, `uses.tsv`, `questions.tsv` and `scan-unresolved.txt` preserve that population. Both cfg branches are included conservatively. This is not a rustc-resolved whole-program call graph: method-name/receiver candidates, function pointers and macros require review. The 197 raw walker diagnostics include test macros; their kinds are 126 `matches`, 27 `Token`, 19 `thread_local`, 17 `parse_quote`, three `parenthesized`, four generated inventories and one unreachable-source diagnostic for the reference generator. They are retained, not silently certified or counted as runtime defects.

The follow-on source screens were: propagate may-evaluate reach through result-bearing helpers; inspect bare `?`, early returns and catch-all arms; inspect transformations after a may-evaluate producer even when input/output are both Expr; and inspect optional-result helpers at **each caller's consuming slot**. `optional-source-helpers.tsv` retains all 57 nominal optional-expression helper signatures for caller-by-caller review. `effect-edges.tsv`, `effect-questions.tsv` (20 concentrated `?` sites), `frameless-direct-evaluate.tsv` (56 direct edges), and `frameless-transforms.tsv` (143 functions before semantic filtering) retain the screens. A resource-check Result, an Option iterator map, a pure parser and a continuation's already-completed input are not themselves suspending producers. Calls that delegate the entire remaining phase to an existing capture are not missing frames merely because the caller contains none. Conversely, a nearby frame is not proof that a nested helper's tail ran.

The table below is the resulting **semantic candidate/cut ledger**. All suspending candidate families have executed witnesses in `cases.tsv`; each listed identifier maps to exact setup/source, pause count and optional state projections. Synchronous reject/decline branches are separately identified: their control pauses in head preparation, not inside an impossible suspension in the branch. Paths prefixed `rt/` mean `crates/rum-runtime/src/runtime/`; `std/` means `crates/rum-stdlib/src/`. SAME means full recorded observations agree for these probes; it does not promote an unresolved source construct or certify all future contexts.

| Site | Caller | Skipped tail or proven owner | Consumer slot type | Probe | Result | Owning G13 step |
| --- | --- | --- | --- | --- | --- | --- |
| `rt/eval_dispatch/mod.rs:114,147,223` SpecialForm / OrdinaryBuiltin | ordinary and suspended call-stage loops | `finish_builtin_dispatch` and typed F0278 own Some and builtin completion | Expr -> Option<Expr> -> Expr | `ordinary.none`, `scan.tail`, `fold.named.*`, `nest.named.*` | SAME; anonymous Function callbacks still DIFF below | S3, complete |
| Same `:117` PrepareArguments stage | call loop intercepts this stage | unreachable generic arm; preparation has its own frame | PreparedCallArguments, then Expr | `prepare.*`, `head.sequence`, `cross.prepared.twice` | SAME | S3, complete |
| Same `:120`; `listable_dispatch.rs:45` | Listable stage | thread planner cannot suspend; actual evaluation at :124 has typed Expr -> Some frame | Option<Expr> | `listable.none` | SAME | S3 |
| Same `:138-145`; `definitions/application/{up_values,down_values,sub_values}.rs` | UpValues, DownValues, SubValues stages | scans carry cursor, guards, and correctly typed definition RHS completion | Option<Expr> | `up.return`, `down.return`, `sub.return`, `down.none`, `cross.guards.twice`, `cross.default.*` | SAME | S5/S6 (guard phases) |
| `rt/scoping/callable.rs:10-30,60-74,144` slot and explicit Functions | Function stage | existing F0528 absorbs Return but omits Some | Option<Expr> | `function.{none,some,empty_some,return}`; anonymous `fold.*`, `nest.*` | markers DIFF; Return SAME | S13: F0528/M0563 |
| Same `:31` Composition | Function stage | composition construction is pure; evaluation shares F0528 | Option<Expr> | `composition.return`, `composition.none` | Return SAME; None causes extra request | S13 |
| Same `:33`; `status.rs:103` single-property status application | Function stage | missing projection completion, evaluate, Return absorption and Some | Option<Expr> | `director.01-.06`, `status.projection_return`, `status.sideeffects`, `status.value_pause` | six DIFF; controls SAME | new G15; lower frames remain S10 |
| `status.rs:89-100,167` list-property status application | Function stage | list projections with non-None context are pure; common evaluation can pause and uses F0528 today | Option<Expr> | `status.list_property` | SAME for list result (wrapper masks raw marker) | G15 status value tail; F0528 remains S13 elsewhere |
| Same `:154,167`; callable :60 | Failure raw property / Success property | projection pure, delayed value pauses in common tail; missing F0528 Some | Option<Expr> | `status.raw_property`, `status.success` | DIFF | G15 status-specific completion, no F0528 typing |
| `callable.rs:35` association application | Function stage | lookup produces an expression synchronously; common evaluation owns Return, misses Some on resume | Option<Expr> | `association.return`, `association.none` | Return SAME, None DIFF | S13 |
| Same `:37,43`, :14 and :56 | invalid Key, invalid operator arity, invalid Function arity, no callable head | synchronous messages/held call/None; no inner may-suspend operation | Option<Expr> | `callable.invalid_key`, `.invalid_wrapper`, `.invalid_function`, `.decline` | SAME after asserted head pause | no new row |
| Same `:41`; `kernel_data/functional_data.rs:81` | operator-wrapper call | pure wrapper construction, then common evaluation | Option<Expr> | `operator.wrapper`, `operator.none` | SAME; enclosing List masks marker in this shape | S13 |
| Same `:53`; `control_data/failsafe.rs:18` | Failsafe wrapper | pure construction, common evaluation / nested Function | Option<Expr> | `failsafe.wrapper`, `failsafe.none` | ordinary value SAME; None extra request | S13 |
| `rt/eval_call_prepare.rs:235,385`; `eval_core/call.rs:305` | ordinary/resumed preparation | typed accumulator owns cursor, prefix, splice permission and Unevaluated restoration | PreparedCallArguments | `prepare.unevaluated`, `head.sequence`, `cross.prepared.twice`, `cross.head.map.twice` | SAME, including `hits` projection | S3 |
| `rt/eval_core/symbol.rs:88,114,149,172`; `definitions/application/rhs.rs:6,51` | OwnValue versus definition application | separate Expr and Option<Expr> completions both absorb Return | Expr / Option<Expr> | `own.return`, `own.none`, `down.return`, `down.none` | SAME | S5 |
| `rt/definitions/definition_lhs_preparation.rs:531` | driver :284/:338/:409, F0257 resume :498 | **cycle-to-original-symbol recovery is skipped**, even though both slots are Expr | Expr | `lhs.cycle`, `.cycle_head`, `.cycle_pattern`, `.cycle_twice`, `.noncycle` | five DIFF including `.after_prior_pause`; Throw/non-symbol/other-symbol controls SAME | **S5 F0257/M0260-M0262 semantic repair branch** |
| `rt/kernel_data/structure_data/selectors.rs:99-127` | Part lazy positive-index re-dispatch | builds First[LazyDrop[…]] then returns evaluate as an Expr tail | Expr | `part.redispatch`, `cross.lazy.first.realization` | SAME; logs prove realization | S44; no G15 frame |
| `rt/definitions/definition_updates.rs:234,253,382`; `definitions.rs:318-340` | held update, positional / association assignment | inner selector continuation completes the whole update/store; optional outer unwraps cancel on suspending accepted branches | Expr; internal HeldUpdateCurrent / Option<Expr> | `held.update`, `part.store`, `assoc.store`, `cross.held.part.twice` | SAME | S7 |
| `rt/messages` rendering; `status.rs:250-277` | Message, status Message/StyledMessage | rendered String -> Expr has F0346; message print/emit tail has its own owner | String / Expr | `message.render`, `status.template`, `status.twice`, `cross.input.effects` | SAME | status S10; messages S18 |
| `rt/kernel_data/control_data/status.rs:63,109,179-247` | Function, Enclose string handler :267, Confirm Failure :303 | **multi-consumer seam**: Function needs stage tail + Some; other two need raw projected Expr; internal Some/unwrap cancellation is presently valid only on guaranteed-present paths | Option<Expr> / Expr / Expr | `director.13-.14`, `status.enclose_none`, `.confirm_none`, `.twice` | Expr consumers SAME; Function defects above | G15 plus S10 F0344-F0346/M0355-M0357/T0030-T0034 |
| `rt/kernel_data/control_data/confirmation.rs:97-104` | seven Confirm-family information consumers | evaluate(info).map(Some) has no producer completion | Option<Expr> | `confirm.info.1-.7`, `cross.confirmation.info.twice` | seven marker DIFF; ordinary 19 SAME | S10 T0028 and F0325/F0328/F0330-F0332 plus ConfirmMatch's alternate path |
| `rt/kernel_data/rewrite_data/cases/collect.rs:427-449`, resume_frames :295 | Cases rule result collector | F0358 returns raw value while ordinary helper maps Some | Option<Expr> | `cases.none`, `.some`, `.empty_some` | three DIFF | S19 F0358/M0374 |
| `rt/options/core.rs:55,78`; `options/read_phase.rs:99-138` | bool/rule/runtime option continuations; OptionValue scalar/many | decoder and frozen consumer own completion; bool path consumes raw Expr intentionally, rule decoder constructs the genuine Rule; scalar OptionValue is Expr-tail | Expr -> bool / Option<Expr> / Expr, then Expr | `option.bool`, `.rule`, `.many`, `cross.options.twice` | SAME | S18; no new frame |
| `std/functional_ops/arrays/tensor.rs:180,257`; arrays/mod :19,91 | Inner and Dot | compute_inner Some is immediately unwrapped by Inner; Dot's existing frame deliberately receives raw Expr and resumes remaining operands | Expr on suspending accepted branch; Option only sync classification | `inner.none`, `dot.tail`, `cross.inner.cancelled.option` | SAME | S39, T0002/M0068; normalize before typing |
| `std/integer_digits.rs:104,180,235`; `number_theory.rs:137,167,193,219,250,276` | IntegerReverse, DigitCount, Prime/Pi/Nu/Omega/NextPrime | Some/unwrap cancels and caller immediately returns threaded Expr; declining branches cannot suspend | Expr (not an actual optional continuation slot) | `thread.integer`, `.number`, `.DigitCount`, `.PrimePi`, `.PrimeNu`, `.PrimeOmega`, `.NextPrime`, `cross.prime.actual.thread` | SAME; Listable explicitly cleared to reach helpers | S39; T0003/T0004 |
| `std/associations/missing.rs:76`; missing/target :107,234 | DeleteMissing | custom traversal stop carries advanced cursor and parent work; bounded completion owns rebuild | Expr; internal optional classifier | `deleteMissing.tail` | SAME, two pauses and retained RuleDelayed structure | S34, no new G15 frame |
| `std/lazy/{machine.rs:8,source.rs:7}`; lazy completion adapters | Normal, First, Take, lazy callbacks | explicit AwaitCursor/realization state, then consumer finish; no plain InputRequired leak from source cursor | internal pull result; final Expr | `lazy.tail`, `part.redispatch`, `cross.lazy.*` | SAME, one to three pauses | S44 |
| `std/schema/machine.rs:57,70` | ConformSchema / SchemaErrors | Leaf/default pending state, continuation stores traversal and output context | matcher outcome / Expr | `schema.tail`, `cross.schema.*` | SAME | S45 |
| `rt/debug_tools/trace/filter/leaf.rs:141` | trace filter driver | outer work owner preserves leaf/pattern cursor; the local ? is captured there | CompiledMatchOutcome -> trace Expr | `trace.filter` | SAME; both log List[3,3] | S17 |
| `rt/accounting_mode/rule.rs:121` and options driver; ordinary rule source | Rule and option completion | fail_unarmed/freeze capture whole finish phase; reserve/charge errors do not suspend | Expr | `rule.tail`, `cross.options.twice` | SAME | S18; no new G15 frame |
| `rt/kernel_data/structure_data/replace/replace_part.rs`; ordered matcher; string callback drivers | target collection, ordered bindings, StringCases/RHS | raw result handed to named driver, or existing frame completes remaining traversal/output | Vec targets / match outcome / String / Expr | `cross.replace.*`, `cross.except.chain.twice`, `cross.string.guard.twice`, `cross.differences.synthetic` | SAME | S25 targets, S27/S29 matcher, S48 strings, S53 differences |
| `rt/eval_core/mod.rs:76-97`; suspension/core/evaluation_context :72 | fixed-point value/stack/resource boundary | stack bookkeeping runs on unwind; EvaluationReturn explicitly checks active value limits after resume | Expr | all evaluator probes; `cross.input.effects` | no additional candidate after source cut | S3 / current evaluation-context owner |

The wider 143-function screen includes pure construction followed by a framed tail (Array/Outer, GroupBy/Merge, set operations, Scan, loader/package entrypoints), resumed finish helpers whose input has already completed, and option reentry clients whose `None` arm **returns the certified reader's whole result**. None establishes a new lost tail merely by containing `map` or `?`. `ValueQ` and best-effort monitor callbacks are explicit retirement/rollback boundaries, not host-suspending candidates: the E1 boundary descriptors must continue to test retirement rather than falsely assert a host pause inside them. Host request/inspection finalizers are API boundaries, with the corresponding resume entrypoint doing finalization; they do not use a dispatch-stage optional slot. These exclusions are semantic cuts, not a claim that the lexical screen is a proof of absence. The follow-up census must retain source edges outside the catalog and refuse to turn unresolved edges into “covered” because a frame or a descriptor exists nearby.

#### Newly exposed same-type tail: definition LHS

With `x := (InputString["w"]; x)`, the sources `f[x]=3`, `x[1]=3`, and `f[Blank[x]]=3` reach AwaitCallArgument, AwaitCallHead, and AwaitPatternAtom respectively. All return 3 with queued input. All resume to `TerminatedEvaluation["IterationLimit"]`, with an IterationLimit termination and `General::passlim` reporting re-entry of x. `f[x,y]=3` with similarly defined x and y pauses once before failing on resume but consumes both queued answers and succeeds synchronously. `Catch[f[x]=4]` with x throwing 3 after input returns 3 on both paths: the recovery is specifically for the matching symbol's evaluation cycle, not a general error catch.

The inner OwnValue and fixed-point frames deliver the cycle error to F0257. The ordinary helper catches it at :533-536. The resumed driver starts with `outcome.map_err(ResumeStop::failure)?` at :504 and exits before reading the retained `source`/`source_head`. F0257's correct `Expr -> Expr` declaration cannot express this missing semantic recovery. No kernel change in this execution repairs it.

#### Why previous work did not falsify these instances

`historical-crosswalk.tsv` retains exact 548 rows. Besides P1306/P1307 above, P0686/P0687 recorded the status application edges but did not settle the Function caller tail. P0307 recorded the exact LHS recovery helper as Unsettled; P0301/P0303 recorded surrounding captures. The copied 605 branch-transform table even records the recovery at its old line 494 (`input-605-result-branch-transformations.tsv:1378`) but says “required … not executed here.” It was not absent from all raw data: **enumeration never became a context-specific completion obligation**. S5 typed F0257 and M0260-M0262 and preserved the existing success-state transitions; this did not prove the nested same-type error transform was preserved.

P0614 recorded optional confirmation info; round 5 explicitly retained it as T0028. P0731 called the Cases RHS frame FrameCarries despite its omitted Some; round 5 correctly exposed F0358 and the three `cases.*` witnesses. Those are known planned repairs, not newly discovered defects. F0528/M0563 similarly were already S13 work. An M row records an insertion, so it cannot by itself enumerate a caller that inserts **nothing**. T0030-T0034 see local maps/unwraps; they do not establish the Function tail above them.

E1's `Failure.propertyParameters` and `.propertyTemplate` both expect a top-level string (`options_properties.rs:23-40`). Reaching F0345/F0346 is therefore insufficient. `manifest.rs` currently closes installed entries, descriptors and declared variants, but its shared evaluator entries cover call preparation and default fallback, not every Function-stage producer context. Exact equality over an incomplete observation space remains exact and inadequate.

The six historical `fold.*`/`nest.*` marker programs deserve a dependency correction. They still differ on this S9 tree when the callback is a Function. Named-downvalue versions of all six are equal. F0278 now wraps its builtin result correctly; an inner F0528 can already have corrupted/replayed the callback result before F0278 receives it. Keep S3's ownership/completion tests, but make the original composed witnesses conjunctive **S3 + S13** gates. Do not relabel these as six new S3 transport defects or erase them because S3 was completed.

#### Observation model and required contexts

Compare a request's complete value/error/termination, ordered messages/prints/trace, prescribed state projections, and ordered effect log. Every answered pause must retain pending ownership, reject overlapping evaluation without displacing that owner, and preserve cumulative public-output prefixes. Completion must empty the input queue, clear pending ownership, and permit a fresh request. An unexpected extra pause is a divergence in request behavior, not a value. Cancellation/reset/retirement and fault absence remain independent obligations. Do not compare elapsed profiling durations between blocked and queued execution.

The fixed point can redo **pure value normalization** (Plus, symbol lookup) and sometimes even effects represented by the returned expression, as `status.sideeffects` shows. This is why a result-only oracle misses the missing tail. It cannot reconstruct a disappeared Rust continuation: the correct Return boundary, Some construction, error recovery, pending loop/store work, prefix/cursor, profiler phase, scope/cleanup owner, or prepare/restore policy. Heldness and Sequence/Unevaluated/Nothing are not universally repaired by reevaluation: their interpretation depends on which call owns argument preparation and hold attributes. The passing controls here have a surviving enclosing owner; do not generalize that to a lost owner.

Required context families, selected from the source obligation rather than a descriptor author's preference:

| Context | Construction and observation | Applies to |
| --- | --- | --- |
| `value` | existing ordinary scalar plus exact documented result | every pausing producer |
| `return_boundary` | put Return in the value produced **after** the intended pause; `f[] := (subject; marker); f[]`; also loop/store after subject | every tail that absorbs Return or feeds such a consumer; include raw properties, Function, composition and association |
| `error_before_tail` | Return/Throw during projection versus in the projected expression; matching and nonmatching cycle errors | every error-transform tail; prevents moving Return absorption too early |
| `guest_heads` | literal InternalContinuationNone[], Some[5], Some[], plus a nested marker | every Expr producer reaching a non-Expr consumer, and pure-shape wrappers that can mask it |
| `held_and_splice` | Hold[Return[…]], Unevaluated[x], Sequence[1,2], Nothing; place in held and ordinary argument positions with sentinel siblings | preparation, callable application, rebuilding/traversal and returned held expressions |
| `prefix_and_tail` | distinct log markers before pause, after pause, and after caller; saved definitions/scope projections | every effectful loop, store, option reader, cleanup or reentry phase |
| `same_type_error_finish` | paused LHS symbol cycle plus Throw control; verify termination/messages, stored definition and second fresh request | every catch/match/map_err tail, including Expr -> Expr |
| `phase_observation` | ordered trace and enabled profiling phase/count observations, excluding elapsed durations; resource-context ownership at completion | evaluator/callable boundaries and any profiled/cleanup tail |
| `consumer_routes` | same helper through Function, Enclose-string, Confirm-Failure; optional producer through every information consumer; Inner and Dot separately | every helper with distinct consuming types or tail policies |

Cross these with existing Middle, Twice, Error, AlreadyResumed, Nested and applicable Decline variants. Include “projection pauses, projected evaluation pauses again” and “two separate operands/guards” as distinct Twice obligations; the current `status.value_pause` happens to mask the defect, so it cannot substitute for `guest_heads` after a projection-only pause. Run lifecycle cancellation/reset at the first and second pause. Intentional non-suspending decline/retirement cases carry explicit boundary expectations; they do not count as pausing witnesses.

**Exact context closure.** Keep installed-entry equality, but add a source-derived obligation key `(producer identity, consumer identity, tail policy)` for shared evaluator paths as well as catalog handlers. An exhaustive policy enum determines the required contexts; each entry must select a policy with a source justification, with no default/empty wildcard. Compute the required set of `(entry, descriptor, variant, context, consumer_route)` tuples from those policies and variant applicability. Require exact equality with the **executed** tuple set, reject duplicates and unexecuted declarations, and require an asserted pause at the designated phase for every non-boundary tuple. A newly discovered producer/consumer edge cannot enter “covered” unless its obligations join that required set. No equality based only on whatever contexts a fixture factory chose to return.

Negative controls must delete one applicable context, omit a consumer route, declare but not run a tuple, duplicate another tuple, move the pause before the target producer, and replace a two-pause source with one pause. Every mutation must be rejected by the manifest independently of final value. Semantic mutations must remove the new Function tail, omit Some, and bypass the shared LHS error finish; their specific context tests must go red. This is a stronger falsifier, not a theorem that all future same-type semantic errors are impossible.

#### Repair design and G13 ownership

**G15: complete the status Function stage, with two new typed frames.** Do not install a Some wrapper indiscriminately inside status rendering: that breaks the two Expr consumers. Do not change the optional decoder, reserve guest names, or make internal heads inert. Guest expressions remain ordinary data.

Split status-call **classification** from effectful projection/application. The classifier is pure (no `&mut Session`, no `Result<_, EvalError>`), and distinguishes non-applicable, raw/list projection, and virtual message projection. Existing arity, tag and fallback rules remain exactly the same. In `try_apply_function`, an accepted status branch returns the status application's **whole stage result** directly; it does not take an intermediate optional value with `?` and fall through to a separate tail. Raw/list projections should call pure property lookup, not a may-evaluate virtual helper with a flag that only happens to disable evaluation.

Suggested implementation home: new `scoping/callable/status_application.rs`, declared from `callable.rs`; property classification/rendering remains in `kernel_data/control_data/status.rs`. Keep ordinary head/args borrowed and reuse existing expressions. A checked borrowed plan is a type-level interface, not permission to add a materialized plan field, duplicate validation, extra branch or store on the ordinary path; the instruction/copy/stack gates below decide that.

| Frame / phase | Input | Output | Location and finish |
| --- | --- | --- | --- |
| New `StatusFunctionProjectionFrame` | Expr | Option<Expr> | callable/status_application.rs. On projection success, enter the shared **evaluate projected value → finish_function_application → Some** phase. On projection error, propagate unchanged. |
| New `StatusFunctionValueFrame` | Expr | Option<Expr> | same file. Finish only `finish_function_application(outcome).map(Some)`. Unit state; no guest-value decode. |
| Existing Enclose string-handler projection | Expr | Expr | confirmation/builtins.rs:267 remains an Expr tail. It does not gain Function-style Return absorption. |
| Existing Confirm Failure-message completion F0326 | Expr | Expr | confirmation/builtins.rs:303-314 constructs the thrown confirmation failure. Its later S10 typing preserves that whole tail. |

Take `slot_witness(&produced)` immediately after the actual `status_object_projection_value(...) -> Result<Expr, EvalError>`, before any map/unwrap/Some. Only an InputRequired arm crosses `#[cold] #[inline(never)] suspend_status_function_projection(witness, request)` and constructs the unit projection frame. Do the same immediately after the actual `evaluate(applied)` Result in the shared status-value phase, through `suspend_status_function_value(witness, request)`. No Result crosses either cold helper; no preallocated frame, Vec, Arc, saved profiling token, Option<frame> or large closure environment lives on the ordinary path.

Routes are explicit. Initial projection insertion is **witnessed**. Projected-value evaluation from ordinary status application is witnessed. If the projection frame resumes and the projected expression pauses again, the value helper inserts the **different** value frame with its own actual-result witness; the projection frame converts the returned stop via `ResumeStop<Option<Expr>>` and `TypedAction::complete_work`. This is **witnessed then ResumeStop**, not a same-frame token restart. The value frame has no further producer and completes directly. Existing no-Arc `ContinueToken` is unchanged; unused tokens are not cloned or stored. Pending suffix append and retirement remain the driver's responsibility.

The two phases must preserve the error boundary. A Return arising **while parameters/projection run** must propagate, as `status.projection_return` demonstrates (3 in both runs). A Return from evaluating the **projected value** must be absorbed, allowing f's marker 7. A single `and_then(...).or_else(absorb_return)` around both phases would be wrong. Profiling for `function.apply.evaluate` and `.finish` belongs to the same semantic phase as in the queued path; do not invent one duration spanning user input.

Both new frames read only Expr through ValueMode. Their final Some is genuine typed output; the existing stage consumer may remain in Option<Expr>'s current LegacyMode and will decode that **outer** wrapper, leaving guest markers inside it untouched. No new frame reads a guest Expr via `from_continuation_expr`. The still-existing global legacy modes are disclosed, not claimed to be eliminated by G15.

**Why two new frames rather than typing F0528 now?** One projection frame plus the old common-tail frame fails when evaluating the projected value pauses: F0528 still omits Some. A status-specific value completion closes that second edge without converting another step's frame. G15 uses the existing Return finisher, leaves F0528/M0563 legacy for non-status heads, and does not flip any registry mode. Raw Failure and Success property pauses then use the new status value frame too. The expected reduction in this fixed 136-pair set is **35 -> 27** (the six original G15 rows plus those two raw-property rows). This is a design expectation, not a repair measured in this execution.

**S10 after G15.** Keep F0344-F0346, M0355-M0357 and T0030-T0034 assigned to S10. G15 changes their caller obligation and adds two new rows; it does not type those three frames. Move q8's equality promotion to G15 and retain all three q8 rows as S10 regression gates. S10 still types their producer/finish phases, including twice-resumed paths.

Prefer eliminating the misleading *suspending* optional rendering API in S10: pure selection returns “no template/fallback” or a checked rendering plan; the effectful render helper returns Expr. Absence is selected **before** any guest evaluation. F0344 then consumes rendered Expr and completes styled Expr; F0345 consumes parameter Expr and completes rendered Expr; F0346's actual producer returns String, so its truthful type is String -> Expr. T0030/T0031's optional fallback is moved to synchronous selection, and T0032-T0034's Some maps disappear with that internal optional API. This is semantic phase normalization, not deleting a map because its token spelling looks like transport. Cover invalid-tag/no-template Missing and tag-string fallback explicitly, including Enclose and Confirm, before claiming those branches cannot suspend.

F0346 still crosses the existing String legacy transfer until its scheduled mode flip; do not introduce a new lenient decoder or guess Input=Expr merely because the current wire is an Expr string. If the director requires eliminating that pre-existing String decoder at S10 too, that is an additional producer/consumer migration requiring an explicit owner census, not an excuse to flip String globally here. G15 itself needs neither change.

T0028 is a real missing completion and needs a **new typed optional-information producer frame**, Input Expr, Output Option<Expr>, at the actual evaluate(info) Result, in `confirmation.rs` or its own small resume module. Its unit completion is `outcome.map(Some)`; None/auto-info is synchronous. Use a named cold witnessed insertion helper. When reached from a resumed confirmation frame, insert the new distinct frame witnessed, then convert to ResumeStop for the owning output. S10's consuming frames declare the actual Option<Expr> input and construct their full confirmation result; no direct optional decoding of raw info. All seven `confirm.info.*` rows must become equal, including the seventh's later message-render pause. Add the new frame to S10's counts; “18 existing frames” is not the final count anymore.

**LHS repair: reuse F0257's already-retained source.** Do not add a frame merely to recover a same-type finish. Extract a non-suspending `finish_definition_lhs_active_expr(source, outcome) -> Result<Expr, EvalError>` with the exact current matching-symbol-cycle rule. The ordinary helper calls it after evaluation. F0257's resumed driver calls it using `source` for AwaitPatternAtom/AwaitCallArgument and `source_head.as_ref()` for AwaitCallHead, **before** `map_err(ResumeStop::failure)` and before replacing work. All three variants already retain that source (`definition_lhs_preparation/resume_frames.rs:26-45`); no field is added. Errors other than the matching cycle propagate unchanged.

The existing actual-result witnesses at M0260-M0262 and `DefinitionResumeRoute<DefinitionLhsPreparationSuspensionFrame>` remain. Ordinary entry uses the witnessed route; the resumed driver uses its same-frame ContinueToken to capture advanced state at a later pause. No Arc enters that token, no raw InputRequired is synthesized, and no codec changes. This is a **semantic follow-up to S5**, not S6 and not a new transport flip. Expected fixed-set reduction: five rows. The added `lhs.after_prior_pause` probe reaches a recoverable symbol cycle after a successful first resumed argument: two pauses, still DIFF. The original two-cycle probe fails at the first one. `lhs.non_symbol_cycle` and `lhs.other_symbol_cycle` are equal negative controls: recovery must not broaden to a call expression or a different symbol's cycle.

**S13 and S19.** S13 converts F0528/M0563 as planned: witness at evaluate(applied), Input Expr, Output Option<Expr>, finish Return then Some on every completed path. It may share the already-typed status value finisher/frame if identity/receipts are explicitly reconciled; do not silently delete the new G15 frame or change unrelated row dispositions. Keep the status projection frame, which represents an earlier phase. Promote `function.*`, association/composition/failsafe marker contexts and the six composed Fold/Nest rows together. S19 converts F0358/M0374 to Expr -> Option<Expr>, completing `outcome.map(Some)`; the declining match remains synchronous None. Neither row needs typing during G15.

S18, S34, S39, S44 and S45 keep their modes/ownership. Amend S39's draft to separate optional synchronous applicability from effectful Expr production for Inner/Dot and explicit threading before type derivation; preserving the existing accepted-path Some/unwrap cancellation is essential. Amend every affected draft's oracle obligation list with applicable contexts/consumer routes. The new context matrix is additive to all existing 608/round-4, fault, lifecycle, cleanup and memory gates.

#### Enforceability: preferred structure and its limits

A frame per discovered site is necessary for execution but insufficient as a prevention rule. `TypedFrame<Input=Expr, Output=Expr>` compiles even when it omits cycle recovery. `Result<Option<Expr>, EvalError>` also allows `?` to propagate past an unrepresented caller tail. The witness proves the **actual producer type**, not that all caller semantics have been encoded.

Prefer a **closed phase coordinator with pure selection and explicit continuations**. Callable classification cannot evaluate. A status producer is private to a completion-taking API; Function, Enclose-string and Confirm-message are three sealed completion policies with their actual output types, rather than an exported raw optional getter. The coordinator is the only place allowed to combine projection and Function evaluation. A compile control should show that a classification helper cannot call evaluate, that an Expr producer cannot attach an Option-input frame, and that the Function completion cannot emit raw Expr where Option<Expr> is required.

For further dispatch closure, introduce an opaque, non-`Try` may-suspend stage result and sealed bind/tail operations *within the dispatch module*, after deriving every producer's obligation. A bind accepts the actual producer Result, paired ordinary/resumed finish policy and the declared output; only the coordinator may lower a completed stage to the existing loop's Result. Failure and suspended outcomes are distinct: generic `?` may carry ordinary failure but cannot bypass bind with InputRequired. Keep raw Session/evaluate access out of the restricted selector/producer facade. Add compile-fail controls for plain `?`, early `return producer`, and a match that attempts to forward a pending request without its completion policy. Public Session need not change; unrelated G13 modes do not change.

This is stronger than a convention or a `#[must_use]` annotation. It still cannot prove that an author wrote the right **same-type** finisher, and Rust affine values can be deliberately discarded; the independent source-edge ledger and mutation-tested context closure are still required. Do not claim “compiler-enforced class closure” merely because a wrapper is non-Try while raw evaluate remains reachable in its callers. The drafted enforcement brief requires a prototype with those negative controls before selecting the concrete API. Correctness/enforceability choose the structure; cost is separately gated, never a reason to accept a missing obligation.

#### Cost and measurement contract for implementation

No performance measurements were run here. The requirement is **zero additional ordinary-path work**, not “small overhead”: no frame/Arc/Vec allocation, new retained field, unconditional diagnostic branch/load, extra Result store/copy, or duplicate classification. A zero-sized witness is allowed only if the generated ordinary path actually removes it. Extracting the LHS finish should leave the existing ordinary instructions unchanged and adds no state. The ordinary callable selector/phase helpers must inline or otherwise preserve that property; prove it by instructions, copies and prologue/stack attribution.

On a projection pause G15 adds one unit typed projection entry plus its adapter and later genuine Some encoding. If evaluating the projected value pauses, the status value frame is the corresponding typed completion (replacing the legacy F0528 used on that path); no projection prefix is replayed. Existing lower status frames, pending suffix ownership and driver transfers remain. The established typed adapter is one 24-byte Arc per newly typed transfer in S9's accounting; a **new** frame also has its own allocation and queue impact. Do not summarize all of that as only 24 bytes. Report allocation **counts and bytes**, wrapper allocations, queue capacity changes, each adapter/frame and exact resumption count. LHS's shared recovery adds no frame allocation.

Every implementation draft below requires:

1. Director-supplied immutable base source/binaries and hashes, candidate built from the same source paths into separate scratch targets, both with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`. No git operations by the executor. All commands serial, bounded, full env set up front, stdin `/dev/null`, profiling disabled for timing. Calibrate against current S9/base sources, not blindly against old S8 numbers in brief 617.
2. Ordinary labels for pure/explicit Function, Composition, association and Failure/Success properties (raw, Message, StyledMessage, list/missing/fallback), normal/malformed callable arity, OwnValue/down/up/subvalue, preparation/Unevaluated/Listable and all three LHS regions. Include literal-integer, do-count-null, do-iterator-null and the existing whole same-tier panel. Each label's exact program is unchanged between arms; no sink assignment or added Print inside the measured statement.
3. Resume labels separately for parameter pause, message-definition pause, styled pause, raw-property value pause, projection then value pause, all three status consumers, Return after projection versus during projection, three LHS awaiting states and reentry, optional info and F0528/Cases when owned. Preflight values and actual frame resumes per iteration. Changed semantic paths (3 becoming 7, cycle failure becoming success) require semantic-work attribution and an equal-work string/value control, not a misleading equal-work claim.
4. Allocation/byte counts at N and 2N and slopes: ordinary exactly equal, all resume differences attributed. Instruction slopes `(I(2N)-I(N))/N` with pinned `perf stat -x, -e cpu_core/instructions/u`, two interleaved pairs per arm; flag any ordinary deterministic growth. Resolve flagged rows with at most five perf runs per arm or six callgrind processes per arm, naming every changed address; do not average a regression away. Resume callgrind uses six per arm and attributes every deterministic delta, including pre-existing driver copies without repairing unrelated debt.
5. **S9 copy bar:** count memcpy and implicit aggregate moves on every ordinary label and targeted resume path, with **libc symbolized**, including implementations/PLT aliases. An unresolved libc symbol makes this gate incomplete. Ordinary memcpy count/bytes may not grow. Inspect producer/helper prologues and loads/stores, so compiler-inlined copies cannot hide behind a zero libc-call count. The driver's known 136-byte outcome transfer remains separately attributed to R.
6. **S9 stack bar:** all 17 existing recursive shapes in debug and release, four heights and three samples per height, plus callable-status/LHS shapes that keep the changed helpers resident. Assert the result equals depth; never run a known native-stack-aborting depth. Tabulate each changed function's prologue and any new stack slots. No unexplained growth; any increase needs the director's explicit ruling. **After any copy/ABI/helper-boundary fix, rerun the stack shapes and instruction/copy attribution on the final code.** Earlier stack results do not validate the final layout.
7. Full rum-vs-rum microbench A/B with profiling off, target labels and same-tier neighbors, base/candidate/base/candidate (two pairs maximum). Timing uses the suite's monotonic in-kernel timing; never wall-clock subtraction. No concurrent build or other CPU-heavy job. A hot-label regression is fixed or reported, not shipped on a “no default-path cost” intention.

### 4. Executed witnesses and output

The final scratch test runs **136 pairs: 101 SAME, 35 DIFF, invalid=0**. Every pair observes at least one real host suspension in the resumed arm. Six divergent arms stop at an unexpected second request after answering the planned one; the support driver cancels/retires those requests and the fresh-request check passes. Their completed value is deliberately unavailable. The `lhs.cycle_twice` resumed arm fails after its first pause while the queued arm consumes two answers; that mismatch is part of the defect, not a weakened suspension assertion.

Exact programs and complete output are delivered together:

- [cases.tsv](../../../.temp/brief-618/cases.tsv): every setup, source, planned input count and state projection. Every response is the string `"1"`.
- [results.tsv](../../../.temp/brief-618/results.tsv): every pair's result, observed pause count, values and effect logs.
- [observations.txt](../../../.temp/brief-618/observations.txt): full values/errors/termination/messages/prints/trace, input requests and extra state projections. These are not the director binary's truncated value summaries.
- [director-run.txt](../../../.temp/brief-618/director-run.txt): independent replay of all original 14 director pairs, six DIFF/eight SAME, one pause each; [director-pairs.json](../../../.temp/brief-618/director-pairs.json) preserves arguments.
- [probe.rs](../../../.temp/brief-618/probe.rs), [probe_body.rs](../../../.temp/brief-618/probe_body.rs), [scan.rs](../../../.temp/brief-618/scan.rs): final temporary test and scratch implementation. The entry is deleted from `crates/rum-stdlib/tests/` before this report is appended.

Selected output rows, verbatim (columns are id, result, pauses, resumed, queued, resumed_log, queued_log):

```text
director.01	DIFF	1	3	7	List[]	List[]
director.02	DIFF	1	3	List[3, 5]	List[]	List[]
director.03	DIFF	1	3	8	List[]	List[]
director.04	DIFF	1	Failure["t", Association[Rule["MessageTemplate", InternalContinuationNone[]], RuleDelayed["MessageParameters", CompoundExpression[InputString["w"], List[]]]]]["Message"]	InternalContinuationNone[]	List[]	List[]
director.05	DIFF	1	5	InternalContinuationSome[5]	List[]	List[]
director.06	DIFF	1	Failure["t", Association[Rule["MessageTemplate", InternalContinuationSome[]], RuleDelayed["MessageParameters", CompoundExpression[InputString["w"], List[]]]]]["Message"]	InternalContinuationSome[]	List[]	List[]
director.07	SAME	1	2	2	List[]	List[]
function.none	DIFF	1	Function[Null, CompoundExpression[InputString["w"], InternalContinuationNone[]]][0]	InternalContinuationNone[]	List[]	List[]
function.some	DIFF	1	5	InternalContinuationSome[5]	List[]	List[]
function.empty_some	DIFF	1	Function[Null, CompoundExpression[InputString["w"], InternalContinuationSome[]]][0]	InternalContinuationSome[]	List[]	List[]
lhs.cycle	DIFF	1	TerminatedEvaluation["IterationLimit"]	3	List[]	List[]
cases.none	DIFF	1	List[]	List[InternalContinuationNone[]]	List[]	List[]
cases.some	DIFF	1	List[5]	List[InternalContinuationSome[5]]	List[]	List[]
status.value_pause	SAME	2	InternalContinuationNone[]	InternalContinuationNone[]	List[]	List[]
composition.none	DIFF	2	UnexpectedPause(answered=1, observed=2)	InternalContinuationNone[]	<not collected after retirement>	List[]
lhs.cycle_head	DIFF	1	TerminatedEvaluation["IterationLimit"]	3	List[]	List[]
lhs.cycle_pattern	DIFF	1	TerminatedEvaluation["IterationLimit"]	3	List[]	List[]
lhs.cycle_twice	DIFF	1	TerminatedEvaluation["IterationLimit"]	3	List[]	List[]
cases.empty_some	DIFF	1	List[]	List[InternalContinuationSome[]]	List[]	List[]
fold.InternalContinuationSome[5]	DIFF	1	5	InternalContinuationSome[5]	List[]	List[]
fold.named.some	SAME	1	InternalContinuationSome[5]	InternalContinuationSome[5]	List[]	List[]
nest.named.none	SAME	1	InternalContinuationNone[]	InternalContinuationNone[]	List[]	List[]
status.projection_return	SAME	1	3	3	List[]	List[]
lhs.after_prior_pause	DIFF	2	TerminatedEvaluation["IterationLimit"]	3	List[]	List[]
lhs.non_symbol_cycle	SAME	1	TerminatedEvaluation["IterationLimit"]	TerminatedEvaluation["IterationLimit"]	List[]	List[]
lhs.other_symbol_cycle	SAME	1	TerminatedEvaluation["IterationLimit"]	TerminatedEvaluation["IterationLimit"]	List[]	List[]
```

All seven `confirm.info.*` full Failure values are in results.tsv: None/empty Some becomes Information -> Null, Some[5] becomes Information -> 5 on resume, while queued input retains the literal marker. Their message/held-expression fields are compared too. The fifth row exercises a real Part message; the seventh pauses in info and then Failure message parameters. The 28 `cross.*` rows reuse the old round-4 programs after source-derived candidate selection, including actual Prime threading with Listable cleared, lazy realization, schema leaf/default, default fallback, rule guards, held selectors, replacement targets, message effects, and extra state projections. All 28 are equal. They are controls, not substitutes for the newly added semantic contexts.

Only the two authorized executable command forms were used, serially. The integration command was:

```text
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief618_probe
```

The parent Python subprocess timeout was 930 seconds; TMPDIR and CARGO_TARGET_DIR were under this brief's scratch. The director binary used the 600-second wrapper (outer timeout 620 seconds), pause cap 32 and setup/source pairs; it was neither rebuilt nor modified. Tool execution remained attached to those bounded parent processes and was polled to completion, not timed out by an earlier tool deadline.

Final run (`test-run10.log`) verification, verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.91s
     Running tests/zz_brief618_probe.rs (.temp/brief-618/target/debug/deps/zz_brief618_probe-7a357ff08a0b72f2)

running 2 tests
test observable_tail_pairs ... ok
test source_census ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.46s


```

The test is a diagnostic harness: DIFF is recorded, not asserted as expected semantics. Its green result means all fixtures were valid, all comparisons were collected, all prescribed pauses/projections/lifecycle checks were exercised, and retained FaultProbe assertions found no new continuation-slot fault. It does **not** mean the production defects are fixed. No production suite, no-default-features build or performance gate was run under this design-only command fence.

Intermediate run disclosure: run1 failed to compile the scratch scanner's module paths; run2 failed on a queue sentinel; run3 rejected a non-pausing Part fixture; run4 rejected an under-counted Trace callback, an invalid SchemaValidate name, and extra-pause observations. Run5 mistakenly serialized source cells with CSV quoting while the reader expected literal tabs; its results are invalid and discarded. Run6 still had a non-pausing option fixture and an effectful cycle fixture that changed the epoch; run7 detected the additional Fold/Nest requests. These were corrected or recorded as explicit request divergences, never converted into fabricated completed values. Run8 (128 pairs), run9 (133), and run10 (136) pass. All ten logs remain available; only the final tables are the acceptance evidence.

### 5. Repair sequence and drafted implementation briefs

The drafts are **raw material for the director to review and rewrite**, as TEMPLATE.md requires; this execution does not authorize or implement them. They contain Goal, Context pointers, explicit MAY/forbidden scope, maintenance/disclosure duty, Deliverable, Steps, rum/Rust pitfalls, mandatory correctness and measured gates, checkpointing, Hard rules and report format. Each requires red oracle fixtures, one-codegen-unit instruction gates, serial bounded runs and a stop-loss based on falling failing-check counts.

1. [Draft G15-A](../../../.temp/brief-618/draft-G15-A.md): pure status selection, two typed status Function completions, source/consumer context manifest and q8 promotion. Add the context falsifiers before repairing code; prove the eight owned failures disappear.
2. [Draft G15-B](../../../.temp/brief-618/draft-G15-B.md): S5 semantic follow-up using the retained source in F0257, including second-resume and negative recovery controls. No new frame/state or transport conversion.
3. Resume the normal G13 sequence with the director's rewritten S10/S13/S19 briefs. [Draft amendments](../../../.temp/brief-618/draft-amendments.md) specifies exact changed obligations, scope additions, counts, source normalization and composed witness dependencies, including S3/S5 records and S18/S34/S39/S44/S45 controls. Intervening G13 steps retain their order and ownership.
4. [Draft G15-C](../../../.temp/brief-618/draft-G15-C.md): finish compiler/source enforcement of the callable completion boundary after the relevant semantic repairs; prove plain-? / early-return / pending-forwarding escapes are rejected, and make the context matrix exact for existing descriptors. It must report a failed prototype honestly if the restricted interface still exposes a raw escape. Any newly exposed cross-owner defect gets a focused repair brief; it is not silently patched or waived here.

The fixed-set acceptance counter makes progress concrete:

| Relevant landing | Expected remaining differences among the same 136 pairs | Required newly equal group |
| --- | ---: | --- |
| Current tree (measured) | 35 | baseline: 101 SAME, 35 DIFF |
| G15-A | 27 | director.01-.06 and raw Failure/Success properties |
| G15-B | 22 | five matching-symbol LHS cycle contexts |
| S10 | 15 | seven information-marker contexts |
| S13 | 3 | three Function markers, association/composition/Failsafe, six composed Fold/Nest markers |
| S19 | 0 | three Cases markers |

Only the first row is measured. After each implementation attempt, report owned equality failures, missing/failed context/compile controls, allocation/instruction/copy/stack failures. Once compilation succeeds, two consecutive attempts reducing none of those counters triggers STOP with evidence. Compiler-error reduction remains progress while fixing a compiling transition. Do not use “a different reason” or “less code” as a substitute for a lower failure count. Added context cases are reported as a separate expanding population so the fixed baseline cannot be changed to manufacture progress.

### 6. Open questions for the director, with recommendations

1. **Sequence of the newly demonstrated LHS bug.** Recommend G15-B immediately after the status repair and before S10 resumes. It is the same missing-tail class, already typed in S5, and its source is already retained. Repairing it in the existing frame is preferable to allocating a redundant recovery frame. Do not label it S6.
2. **S10's internal optional rendering API.** Recommend pure presence/fallback selection followed by an Expr-returning effectful renderer, with all three consumers explicit. This removes the misleading Some/unwrap cancellation and avoids adding optional bridges at each level. F0346 truthfully consumes String; its pre-existing legacy transfer must remain disclosed until its assigned migration, unless the director explicitly expands that producer/consumer scope. Do not flip String/Option globally as part of G15.
3. **Compiler closure versus a patch-only branch.** Recommend the pure selector/private completion coordinator in G15-A, with the stronger restricted non-Try prototype in G15-C. Reject a cosmetic wrapper that leaves raw evaluate callable by its producers. The compiler can prevent omitted binds/type mismatches; same-type semantic completion still needs mutation-tested contexts. Neither cost nor nominal type equality justifies a weaker correctness claim.
4. **How staged oracle failures are presented.** Recommend executing every required context and retaining exact baseline defect identities with named promotion owners while keeping equality as the normative assertion. Never exclude a context to obtain a green total, broaden an accepted failure, or describe a known-divergence ledger as semantic conformance. New/unclassified divergences block the step. The six Fold/Nest witnesses need the explicit S3 + S13 prerequisite, with their named-downvalue controls separately green.
5. **Census closure claim.** Recommend keeping the raw conservative graph and unresolved diagnostics separate from demonstrated semantic cuts. Do not claim all 154,891 uses are compiler-resolved or that 143 lexical hits are 143 defects. The new ledger unit is the producer **and consumer tail**, not a builtin or an M insertion. A discovered unframed edge cannot be marked covered solely by a nearby frame, a typed Input/Output pair, or a descriptor reaching its callee. This execution supplies concrete candidate results and a prevention design; it does not claim a formal whole-program effect proof.
6. **Enabled profiling observations.** Recommend phase/outcome/count invariants using existing profiling hooks or test-only observation, with no new disabled-path load/branch. Compare semantic completion and balanced ownership, not elapsed duration across input waits. Keep resource-limit return checks, cancellation and abandonment as separate lifecycle assertions. In particular the existing EvaluationReturn marker's value-limit check must not be lost while changing the callable tail.

No permission was needed or requested during this design-only execution. No production fix, transport flip, performance result, broad suite result or commit is claimed.

---

## Director review (2026-10-18) — accepted, with three changes

**Runs.** Run 1 (session `01a14ce3-…`) was cut after 18 min by "Selected
model is at capacity" (log `cdx_brief_618.capacity-1.log`). Redispatch 1
forked it (session `01a14cf5-343a-7371-827f-ed20ebcb6f8f`) and finished in
37 min. Scope: only this file changed; the scratch test was deleted; no
capacity or turn error in the second log.

**Reproduced by the director.**
- All 136 pairs of `.temp/brief-618/cases.tsv` through the director's own
  probe binary (`.temp/director/gate-618/repro136.txt`): 35 DIFF, 101 SAME,
  every pair paused, and 136/136 agree with `results.tsv`.
- The LHS mechanism from source: the ordinary path recovers a matching
  symbol cycle in `evaluate_definition_lhs_active_expr`
  (`definitions/definition_lhs_preparation.rs:532-540`); the resumed driver
  propagates it at `:503` (`outcome.map_err(ResumeStop::failure)?`). The
  frame before S5 had the same `outcome?`, so this predates S5; it is not an
  S5 regression.
- The census rows: P1306, P0686, P0687 and P0307 are `Unsettled` in
  `.temp/brief-548/census.tsv`; P1307 is `FrameCarries`.

**Accepted.** The diagnosis, the observation model and its context table,
G15-A's design (a status classifier that takes no `Session`, the status
branch returning its whole stage result, two typed frames Expr →
`Option<Expr>`, witnessed then `ResumeStop` for the second pause), G15-B's
design (a shared finish on F0257's retained source, no new frame or field),
and q8's promotion moving to G15-A. Open questions 1, 4, 5 and 6 are
accepted as recommended; question 2 is accepted in direction and ruled in
S10's brief.

**Corrections.**
- The six Fold/Nest rows are already S13's: brief-609's redispatch 1
  reassigned them, and `divergence-step-gates.tsv` is stale. No S3
  dependency change is needed.
- The brief's "no census found it" is wrong. brief-548 enumerated the edge
  and left it `Unsettled`; nothing settled it.

**Changes.**
1. **G15-A does not build the context manifest.** It adds its own witnesses
   as strict-equality E1 fixtures, with the contexts they need, red on the
   base. The obligation-keyed manifest moves to the new G15-C.
2. **The compiler-enforced coordinator (draft G15-C) is E2's subject, not
   G15's.** E2 is the planned may-suspend result type with no `Try`. A
   dispatch-local `Pending` now would be a second type that E2 deletes, and
   the draft itself concedes it leaves raw `evaluate` reachable. The
   Function stage and the draft's compile-fail controls (plain `?`, early
   return, forwarding `InputRequired`, wrong input frame, `Expr` for an
   `Option` stage) become E2's acceptance criteria.
3. **A new G15-C settles the census before S10.** brief-548 closed with 495
   `Unsettled` pairs (233 U1, 201 U5, 46 U2, …) and said so; G1-G12 and E1
   never settled them. G15 and the LHS defect both come from that list, and
   this report cross-walked 6 of the 495. G15-C is design-only: rebase every
   one of the 495 and this report's screens onto the current tree, settle
   each with an executed rum-vs-rum probe in an observable context, group
   what is lost, and design the obligation-keyed manifest with its mutation
   controls. It precedes S10, so every later typing step is gated by an
   oracle that can see a skipped tail.

**Sequence.** brief-619 G15-A, brief-620 G15-B, brief-621 G15-C (design),
its repairs and the manifest, then `CHANGELOG.md` (user ruling), then S10.
