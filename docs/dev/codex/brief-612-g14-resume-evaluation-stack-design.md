# brief-612 — design: G14, resume never unwinds the evaluation stack it restores

`Scope: NONE — no production change.` This brief is design-only, with one
declared scratch test (see Scope). **Do not run `git` in any form**,
including before you have read this brief.

## Goal

A resumed evaluation does not leave the evaluator in the state that the
same program reaches when its input is queued. After one pause, the
resumed path reports a false cycle on repeated identical expressions. On
expressions that change, it leaks one recursion level per resume. The
queued-input path is correct in every case below.

The director found this on 2026-10-16, while gating brief-611 (G13 step S5).
Its 2 000-resume harness stopped with `RecursionLimit` at 1 021 resumes.
The defect is already present on S4, and it is as old as resumable input
itself:

- `b51dd794` (2026-03-20) dropped only the top stack entry at a pause.
- `eee42d2f` (2026-03-21) widened that to two, after false positives. That
  patched the instances seen, not the class.

The design must settle the repair of the class, not of the shapes below.

### The mechanism, as far as the director has read it

- **`begin_input_suspension`** (`runtime/suspension/core.rs:492-518`).
  It snapshots `evaluation_stack` minus its top two entries into the pending
  suspension. The comment explains the "two": the `InputString` call and
  its immediate caller are resumed from frames, and keeping them caused
  false cycles.
- **`resume_suspended_slot`** (`core.rs:615-648`). It installs that
  snapshot as the live stack and sets `evaluation_depth` to its length.
  The native-stack poll depth is derived from that depth.
- **`drive_resuming_frames`** (`core.rs:704`). It completes the queued
  frames, innermost first. Nothing pops a restored entry or lowers the
  depth as a frame completes and hands its value outward.
- **The evaluation level** (`runtime/eval_core/mod.rs:28-96`):
  - Its cycle check (`:38-47`) refuses any `expr` already on the stack in
    the same epoch.
  - Its depth check (`:49-53`) refuses at `recursion_limit`.
  - On `Ok`, its epilogue (`:91-93`) runs
    `enforce_active_resource_limits_for_expr`, the active time and memory
    limits (`debug_tools/resources/constraints.rs:139`).

  A level that a frame completes on resume never runs that epilogue.

### Witnesses

Each witness pauses once, through the resumed path. The director's probes
are in `.temp/director/depth-leak-probe/` (built against the working tree)
and `.temp/director/depth-leak-probe-s4/` (S4). They are rum-host programs,
resumed against queued. Argument 1 is the pause cap, then setup/source
pairs; outputs are in `run1.txt` and `run2.txt`.

- **False cycle.** These report `General::passlim` +
  `TerminatedEvaluation["IterationLimit"]`:
  - `f[1]+f[1]` with `f[x_] := (InputString["rhs"]; 7)`;
  - `Table[f[1], {3}]` and `Do[f[1], {1100}]`;
  - SubValue `k[1][2]` in a `Table`;
  - OwnValue `v := (InputString["x"]; 7); Table[v, {3}]`;
  - `Table[{{InputString["x"]}}, {3}]`;
  - `Table[h[{InputString["x"]}], {3}]`;
  - `Table[h[k[InputString["x"]]], {3}]`;
  - `Table[Plus[1, Length[{InputString["x"]}]], {3}]`;
  - `Table[g[g[InputString["x"]]], {3}]` with `g[x_] := x`.
- **Depth leak.** `Table[(t = i; f[1]), {i, 1100}]` hits `RecursionLimit`
  at 1 021 resumes.
- **Pass.**
  - `Table[InputString["x"], {1100}]`;
  - `If`, `Module` and `Map[f, Range[3]]` shapes;
  - `Table[(v = (InputString["x"]; 7)), {3}]`.

  The drop-two heuristic covers exactly the shallow shapes that every
  earlier suspension probe used.

## What the design must settle

1. **The diagnosis, with `path:line` evidence.** For three witnesses, trace
   the live `evaluation_stack` and `evaluation_depth` at the pause, at
   install, and after each resumed frame completes, against the queued run
   of the same program:
   - one false cycle;
   - the depth leak;
   - one passing shape.

   Say why each passing shape passes.
2. **The correct model.** While a frame resumes, the evaluator's stack,
   depth, epoch and native-stack poll state must equal what the queued path
   has at that frame's level.
   - **The candidate model.** Each queued frame records the stack length
     and depth at its insertion. The driver truncates the stack and sets the
     depth to those values before it resumes that frame.
   - **Settle it or replace it.** Say where the record lives. It should be
     the queue entry, structurally, so that none of the ~600 frame types can
     forget it. Cover every insertion route:
     - `insert_suspension_frame` (`core.rs:528`);
     - `insert_typed_suspension_frame` (`:555`);
     - the typed token and witness routes;
     - `FrameChain`, which inserts several frames at once;
     - the resume driver's own pushes.
   - **The "top two".** Does the precise model make the drop-two heuristic
     unnecessary? Show it on the matcher-backtracking re-entry that
     `eee42d2f` was written for. If anything of the heuristic must remain,
     say exactly what and why.
3. **Accounting.** The armed path keeps a maintained total that includes
   evaluation-stack entries:
   - `push_accounted_evaluation_entry` / `pop_accounted_evaluation_entry`;
   - `replace_accounted_evaluation_stack`;
   - the shadow stack (`mode.stack_push`).

   Truncation on resume must keep that total exact. Show how, and name the
   falsifier that proves it (`debug_tools/resources/maintained.rs`).
4. **The skipped epilogue.** Decide whether each frame-completed level must
   run `enforce_active_resource_limits_for_expr` on resume, so that
   `TimeConstrained`/`MemoryConstrained` behave as they do on the queued
   path. Write a rum-vs-rum witness for any observable difference (a
   constrained region whose limit is crossed only at a resumed level's
   completion). If there is none, say why it is unobservable.
5. **Re-suspension.** A resumed frame may pause again. Then
   `begin_input_suspension` snapshots a stack that contains restored
   entries. Show that the second pause's snapshot equals the queued path's,
   suspending twice and mid-loop.
6. **Interaction with G13.** The branch is mid-way through typing 600
   frames (S6-S54, then F1-F23 and R):
   - The repair must live in the driver and the queue, transparent to frame
     implementations, so that no typing step changes for it.
   - Say what the typed driver (S2) and the legacy arm each need, and where
     in the remaining 73 steps nothing moves.
   - If the repair cannot be transparent, say which steps it touches and
     why.
7. **The class falsifier.** Extend the E1 oracle
   (`crates/rum-stdlib/tests/suspension_oracle/`) with descriptors that:
   - vary the depth between the pause and a construct that repeats an
     identical expression;
   - suspend mid-loop and twice;
   - include a long run of changing expressions past `recursion_limit`
     resumes;
   - cover each definition kind (DownValue, SubValue, UpValue, OwnValue),
     nested lists and heads, and arithmetic.

   They are rum against rum: resumed equals queued, in value and side
   effects, with the suspension asserted. Say which of the 133 current
   oracle tests and of brief-548's 195 probes already cover a shape that
   passes only by the heuristic.
8. **Cost.** The ordinary path must not pay: no field, branch or store on a
   non-suspending evaluation. State the resume-path cost (a record per
   inserted frame, a truncate per resumed frame) and how the implementation
   brief measures it. The measures are allocations, then one-codegen-unit
   instruction slopes on the resume labels, plus the 107 ordinary labels
   unchanged.

## What I want back

1. The diagnosis of the three traced witnesses, with `path:line` evidence.
2. The model: where the record lives, every insertion route, the fate of
   the heuristic, accounting, the epilogue, re-suspension and G13.
3. The witnesses you ran in the scratch test, with their output (see
   Scope).
4. The repair sequence, and drafted implementation brief(s) in
   `TEMPLATE.md` shape. Each one has:
   - red tests through the oracle;
   - a MAY list with a disclosure duty;
   - one-codegen-unit instruction gates;
   - serial runs;
   - a stop-loss keyed on whether the failure count falls.
5. Open questions for the director, each with your recommendation.

## Context pointers

- `runtime/suspension/core.rs`, from `begin_input_suspension` through
  `drive_resuming_frames`, and the typed driver under
  `runtime/suspension/typed/`.
- `runtime/eval_core/mod.rs:15-96` and `debug_tools/resources/{constraints,maintained}.rs`.
- `git log` is unavailable. The two historical commits are summarised under
  Goal; the comment at `core.rs:493-498` is the surviving rationale.
- The E1 oracle (`crates/rum-stdlib/tests/suspension_oracle/`), its runner
  and `support/`; `TODO.md`'s G13 section and its G14 entry;
  `docs/guide/{evaluation,embedding}.md` on pauses and resume.
- `TEMPLATE.md` and `AGENTS.md`.

## Scope

- **Files you MAY modify: NONE,** except these:
  - append your report to this file;
  - scratch under `.temp/brief-612/`;
  - **one** scratch integration test,
    `crates/rum-stdlib/tests/zz_brief612_probe.rs`. It may reuse the
    oracle's support through a `#[path]` module. **Delete it before the
    report is appended**, and copy its final content to
    `.temp/brief-612/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.** No network, and nothing outside this
  repository. All scratch and `TMPDIR` go under `.temp/brief-612/`.
- **Do NOT modify any file outside the exceptions.** STOP and report
  instead.
- **Run no probe that aborts a process.**
- **Commands are serial,** each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief612_probe`
  (900 s). Your tool timeout must exceed every bound you pass. You may also
  read files, use `rg`/`grep`, and run `python3`.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum
  only.
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed: "design only", the scratch files, and the deleted test.
2. The diagnosis.
3. The model, and items 2-8 above.
4. The witnesses and their output.
5. The repair sequence and the drafted brief(s).
6. Open questions for the director.

---

## brief-612 report — 2026-10-16

### 1. What changed

**Design only. No production change.** This report is appended to the assigned
brief. Evidence, the final probe source, a draft implementation brief and a
checkpoint are under `.temp/brief-612/`. The only temporary integration test,
`crates/rum-stdlib/tests/zz_brief612_probe.rs`, was copied to
`.temp/brief-612/probe.rs` and **deleted before this report was appended**.
No git, network, other engine, aborting probe, production instrumentation,
inventory refresh or benchmark was run. Builds, logs and `TMPDIR` stayed under
the permitted scratch directory. Commands ran serially.

The final scratch run reports three passing **diagnostic** tests: they record
32 ordinary witnesses (23 divergent, nine equal), and two additional resource
witnesses (both divergent). They do not assert that the defect is repaired.

Evidence links: [final source](../../../.temp/brief-612/probe.rs),
[final run](../../../.temp/brief-612/run4.txt),
[ordinary matrix](../../../.temp/brief-612/matrix.tsv),
[time witness](../../../.temp/brief-612/epilogue.txt),
[memory witness](../../../.temp/brief-612/memory-epilogue.txt),
[historical selections](../../../.temp/brief-612/historical-selected.json),
[implementation draft](../../../.temp/brief-612/draft-G14.md).

### 2. Diagnosis, including the three traces

In the source references below, `R/` means
`crates/rum-runtime/src/runtime/`; `O/` means
`crates/rum-stdlib/tests/suspension_oracle/`. Line numbers describe the tree
read for this brief. **The internal traces below are source-derived, not
runtime instrumentation.** The permitted integration test cannot read private
Session fields. Its observed results are separately reported in section 4.

The failure is a missing evaluation-return protocol, not merely a wrong
constant in the input snapshot:

* `R/suspension/core.rs:492-505` snapshots the tracked expressions minus two.
  It does not change the live native evaluation stack at that point. The
  native unwind subsequently pops real entries and restores real depth in
  `R/eval_core/mod.rs:76-90`.
* `R/suspension/core.rs:632-647` transfers the snapshot into the live vector,
  assigns **vector length** to depth and sets the next native poll to that
  depth plus four (`R/metadata.rs:17`).
* Both driver arms only remove **queue entries**, not evaluation levels:
  `R/suspension/core.rs:733-757,761-798`. Their calls into the evaluator may
  push/pop temporary levels, but the installed prefix remains. Completion of
  the request finally replaces it with the saved host stack at `:672-673`;
  this is too late for a sibling evaluation inside the same request.
* Depth and tracked-stack length are different quantities even before this
  bug. `R/resolution.rs:475-498` tracks only expressions with at most eight
  nodes, counting heads. `R/eval_core/mod.rs:60-75` increments depth regardless
  of tracking. Deriving depth from vector length is independently incorrect.
* Re-entry into a retained same-epoch expression fails at
  `R/eval_core/mod.rs:38-47`. Mutations advance the global epoch
  (`R/evaluation_cache/stability.rs:104-113`), which can hide that failure
  without releasing any retained level. The recursion check at
  `R/eval_core/mod.rs:49-53` then exposes the accumulated depth.

For the traces, write `S / d` for tracked entries, oldest first, and logical
depth. `B(X)` is `RunCallDispatchStageSuspensionFrame` for builtin X
(`R/eval_dispatch/mod.rs:230-238,468-495`); `D(X)` is
`ResumeCallWithDispatchPlanSuspensionFrame`
(`R/eval_core/call.rs:163-187`); `P(X)` is the fixed-point continuation
(`R/resolution.rs:375-382`, `R/resolution/resume_frames.rs:23-39`). These are
helper continuations, not three distinct evaluation levels. The queued column
is the state at return from the corresponding helper. An evaluation epilogue
between helpers is shown explicitly.

#### False cycle: `Table[f[1],{3}]`

Setup: `f[x_] := (InputString["rhs"]; 7)`. Let T be the whole Table, F be
`f[1]`, C be its `CompoundExpression`, and I be `InputString["rhs"]`.
Their node counts are 8, 3, 6 and 3, respectively: all are tracked.

At the input boundary, queued and suspending execution both have
`[T,F,C,I] / 4`. The pending snapshot is `[T,F]`. Native unwinding leaves
`[] / 0` at the host. Installation produces `[T,F] / 2`.

| Completed frame/helper, innermost first | Queued state | Current resumed state |
| --- | --- | --- |
| B(I) | `[T,F,C,I] / 4` | `[T,F] / 2` |
| D(I) | `[T,F,C,I] / 4` | `[T,F] / 2` |
| I's native evaluation epilogue | `[T,F,C] / 3` | absent |
| P(I) | `[T,F,C] / 3` | `[T,F] / 2` |
| `ResumeCompoundExpressionFromIndexSuspensionFrame` for C | `[T,F,C] / 3` | `[T,F] / 2` |
| B(C) | `[T,F,C] / 3` | `[T,F] / 2` |
| D(C) | `[T,F,C] / 3` | `[T,F] / 2` |
| C's native evaluation epilogue | `[T,F] / 2` | absent |
| P(C) | `[T,F] / 2` | `[T,F] / 2` |
| `EvaluateDefinitionApplicationRhsSuspensionFrame` | `[T,F] / 2` | `[T,F] / 2` |
| D(F) | `[T,F] / 2` | `[T,F] / 2` |
| F's native evaluation epilogue | `[T] / 1` | absent |
| P(F) | `[T] / 1` | `[T,F] / 2` |
| `ResumeTableIterationsSuspensionFrame` | `[T] / 1`; next F is legal | `[T,F] / 2`; next F fails before pushing |
| B(T), then D(T), each | `[T] / 1` | `[T,F] / 2`, propagating the error |
| T's native evaluation epilogue | `[] / 0` | absent |
| P(T) | `[] / 0` | `[T,F] / 2`, propagating the error |
| Driver exits to request boundary | `[] / 0` | saved `[] / 0` restored |

The compound frame resumes the tail at
`R/kernel_data/control_data/branching_loops.rs:349-395` and its
`resume_frames.rs:124-143`. The simple pattern takes the fast DownValue RHS
route at `R/definitions/application/down_values.rs:288-342`; the RHS frame
is installed at `application/rhs.rs:106-115` and returns `Some(value)` at
`:155-167`. There is no option-scope frame for this definition:
`R/options/patterns.rs:16-18,66-76`. The Table continuation increments its
cursor and calls the next body at
`R/kernel_data/control_data/branching_loops/iteration/driver.rs:310-340`.

The queued run returns `List[7, 7, 7]`. The resumed run pauses once and returns
`TerminatedEvaluation["IterationLimit"]`, with `General::passlim`. The same
retained F explains the two-addend witness; changing the outer construct
does not repair the missing return.

#### Depth leak: `Table[(t=i;f[1]),{i,1100}]`

Keep the same definition. T now exceeds eight nodes. W, the outer compound
`(t=i;f[1])`, has nine nodes. Neither is tracked; both consume depth.
At the first input boundary the state is `[F,C,I] / 5`. The saved snapshot
is `[F]`, the host state is `[] / 0`, and installation produces `[F] / 1`.

| Completed frame/helper, innermost first | Queued state | Current resumed state on first resume |
| --- | --- | --- |
| B(I), then D(I), each | `[F,C,I] / 5` | `[F] / 1` |
| I epilogue | `[F,C] / 4` | absent |
| P(I), compound frame for C, B(C), D(C), each | `[F,C] / 4` | `[F] / 1` |
| C epilogue | `[F] / 3` | absent |
| P(C), definition-application RHS frame, D(F), each | `[F] / 3` | `[F] / 1` |
| F epilogue | `[] / 2` | absent |
| P(F), compound frame for W, B(W), D(W), each | `[] / 2` | `[F] / 1` |
| W epilogue, which has no tracked entry to pop | `[] / 1` | absent |
| P(W), dynamic iterator-scope frame, each | `[] / 1` | `[F] / 1` |
| Table iteration frame | `[] / 1`; next body starts here | `[F] / 1`; next body starts here |
| B(T), D(T) | `[] / 1` if loop finishes | still queued when next input pauses |
| T epilogue, then P(T) | `[] / 0` | suffix remains until loop finishes/errors |

The clean iterator uses `EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame`
(`iteration/driver.rs:403-420`, `driver/resume_frames.rs:186-206`). Its dynamic
binding operations do not bump the epoch (`R/eval_core/symbol.rs:179-201`).
The explicit assignment `t=i` does. Thus old `F@epoch_j` entries do not trip
the same-epoch cycle check when the next iteration evaluates F.

Let `S_n=[F@e1,...,F@en]` after n pauses. Installing the nth suspension sets
depth to n. All returning frames above retain `S_n / n`. The next iteration
enters W, F, C, I, so its next snapshot is `S_(n+1)` after dropping C and I.
The queued run instead reaches each input with the same `[F,C,I] / 5` and
each loop handoff with `[] / 1`.

At n=1,021, the next I checks depth `n+3=1,024` and is refused before it can
pause. The observed count is exactly 1,021 resumes; `t` is 1,022 versus 1,100
in the queued run. With a configured ceiling of 32, the same derivation
predicts 29 resumes and `t=30`; the scratch test observes both. This is
logical-depth leakage, not deep native recursion or a leaking depth across
completed host requests.

#### Passing shape: `Table[InputString["x"],{1100}]`

T has eight nodes; I has three. At the first input, both paths have
`[T,I] / 2`. The heuristic saves `[]`; host unwind and installation both
end at `[] / 0`.

| Completed frame/helper | Queued state | Current resumed state |
| --- | --- | --- |
| B(I), D(I), each | `[T,I] / 2` | `[] / 0` |
| I epilogue | `[T] / 1` | absent |
| P(I) | `[T] / 1` | `[] / 0` |
| Table iteration frame | `[T] / 1` | `[] / 0` |
| Next input, if any | `[T,I] / 2` | `[I] / 1`, again snapshots `[]` |
| B(T), D(T), each, after final iteration | `[T] / 1` | `[] / 0` |
| T epilogue, then P(T) | `[] / 0` | epilogue absent; P(T) leaves `[] / 0` |

The value passes because there is no stale entry and depth never accumulates.
Its logical context nevertheless differs throughout the resumed segment.
The observed `Stack[]` variant makes that loss visible: resumed results contain
empty lists, while queued results contain the active compound expression.

Other passes are also explicable, without treating them as evidence of a
correct general model:

* `Table[If[True,I,0],{3}]` and `Table[Module[{},I],{3}]` have an untracked
  Table and two tracked inner calls. Dropping two empties the saved stack.
* `Table[(v=(I;7)),{3}]` has an untracked Table and Set, and only the compound
  and I are tracked. Again, no saved entries survive. The assignment also
  advances the epoch, but is not needed to explain the empty prefix here.
* `Map[f,Range[3]]` retains the Map and completed `f[1]`, then `f[2]`, etc.
  The later calls differ, so a three-element run does not expose the leak.
  **It is not evidence that Map unwinds correctly.** The added
  `Map[f,{1,1,1}]` control fails after one pause.

### 3. The model and the remaining design requirements

#### Queue-owned contexts plus explicit evaluation returns

Adopt the candidate's insertion-time context, **with a full snapshot and an
explicit return event for each suspended evaluator level**. A queue of helper
frames alone cannot tell where an evaluator epilogue belongs. Calling that
epilogue after every helper would check intermediate transport values, check
too early, and confuse G13's non-Expr slots with guest results.

Suggested private shape, not production code in this brief:

```rust
struct ResumeContext { stack_len: usize, depth: usize }
struct Queued<P> { context: ResumeContext, payload: P }
// Payloads include Legacy(frame), Typed(frame), and EvaluationReturn.
// EvaluationReturn is an inline queue event, not an Arc-backed frame.
```

Keep the existing Legacy/Mixed queue distinction if its measured advantage
remains; both cell types must structurally contain the context, and both
must support a return event. Do not add context fields to 600 frame types,
Session, Expr, a typed slot, a result carrier or an ordinary evaluation local.
The queue's private constructors require a context; no default context or
unrecorded insertion overload is allowed. Exact cell sizes are measured,
not assumed from enum layout.

1. `begin_input_suspension` clones the **complete** tracked stack, including
   each entry's original epoch. Delete the drop-two rationale and operation.
2. Every frame insertion captures the current `stack.len()` and the current
   logical depth, after the suspended child has natively unwound to the
   installing helper. The driver restores that context **before** invoking
   either kind of frame, using an accounted truncate and assigning depth.
   Equal contexts on consecutive frames are normal. A frame completion is
   not necessarily an evaluation-level completion.
3. At the two existing fixed-point **suspension-only construction sites**
   (`R/resolution.rs:379-381,443`), append an `EvaluationReturn` event before
   inserting the fixed-point frame. These are the only two callers of
   `evaluate_once_unstable_after_input_check_with_cycle_tracking`
   (`resolution.rs:367,434`). By then the evaluator has already restored
   depth, popped its tracked entry and dropped its shadow guard
   (`R/eval_core/mod.rs:77-96`), so capture the current parent context there.
   The marker follows the level's helper frames and precedes its fixed-point
   frame. It is created even for `track_cycle=false`, even for a tail level
   without a local helper frame, exactly once per suspended evaluator
   activation. Keep a closed source-boundary test for these two callers and
   the marker-before-consumer rule, so a future caller cannot silently omit it.
4. When that event is driven, restore the parent context first. On
   `Ok(Slot::Value(expr))`, enforce active resource limits for expr and
   continue with the result. On an error, forward it without a successful
   epilogue, just as `eval_core` does. `InputRequired` is spliced before a
   suffix return event is consumed. A non-Expr success at this boundary is
   a fatal typed-slot fault, not a guest-encoded value.
5. At ordinary completion, the outermost return event has already restored
   the parent context. The outer resume entry still restores its displaced
   stack/depth/native state on every exit. Do not rely on that final restore
   to do intermediate unwinding.

This is a small necessary addition to a literal “driver and queue only”
repair: two already-cold evaluator suspension sites identify return boundaries.
The rest is queue/driver/accounting plumbing. **No frame implementation or
G13 family typing step needs to identify, save, restore or finish a level.**
Identifying fixed-point frame types by name, TypeId, or legacy marker payload
would be a weaker substitute and would tie the repair to the migration.

Concrete cold sequencing: give the typed macro an internal
`evaluation_return => frame` form at these two sites, backed by a private
`SlotWitness<Expr>::suspend_evaluation_return` helper constrained to
`TypedFrame<Input=Expr, Output=Expr>`. It inserts the inline return event,
then delegates to ordinary witnessed suspension. The existing macro forms
remain unchanged; this is compile-time choice, not a runtime boolean or
new slot field. Construct the frame only in the selected InputRequired
closure. Do not put a mutable Session call inside the frame argument of
`witness.suspend(session, request, frame)`: that would overlap the Session
argument's mutable borrow. The private helper sequences the two insertions
after argument evaluation. Its positive/negative compile controls protect
Expr-only boundary ownership. This small macro/helper addition belongs to
the central evaluator plumbing, not the remaining family migrations.

Insertion and transfer inventory:

| Route | Current evidence | Required behavior |
| --- | --- | --- |
| Legacy direct insertion and `resume_or_suspend!` | `R/suspension/core.rs:528-539,845-849` | Capture context once at the common cold insertion boundary. |
| Typed insertion / erased typed insertion | `R/suspension/core.rs:555-574` | Same context envelope around the erased frame; erase does not own context. |
| Typed macro and `SlotWitness::suspend` | `R/suspension/typed/witness.rs:16-23,33-70` | Continue funneling into the common typed insertion; witness remains zero-sized. |
| `ContinueToken::suspend` | `R/suspension/typed/resume_token.rs:15-23` | Capture current resumed helper context for the fresh frame; do not reuse the removed frame's old record. |
| `FrameChain::suspend` | `R/suspension/typed/chain.rs:20-29` | Each inserted cell gets the same current context. No guest work occurs between its insertions. No depth decrement per chain element. |
| Legacy driver's `FrameAction::Evaluate` | `R/suspension/core.rs:746-754` | Record the context immediately before the deferred evaluate, after the old frame has completed; enqueue its next frame with that record. |
| Typed driver's `Action::Evaluate` | `R/suspension/core.rs:786-795` | Identical rule, preserving the `Input=Expr` type relation. |
| Complete / Finalize | `R/suspension/core.rs:744-756,781-797` | Retain existing frame drop-before-finalizer ordering; process a return event only after the finalizer's outcome exists. |
| Legacy-to-Mixed widening | `R/suspension/core/queue.rs:18-22` | Move whole records; never recapture or discard context. |
| Nested append | `R/suspension/core.rs:802-828`, `core/queue.rs:153-168` | Append the old recorded suffix after the newly recorded inner frames and returns, with no rebasing by frame count. |
| Abandonment / fault ownership transfers | `R/suspension/core/abandon.rs:35-64`, `core/fault.rs:99-126` | Preserve records; restore context before structural abandonment/authorized cleanup; consume return events without running success epilogues. |

Remove raw frame-only push methods from the private queue implementation.
`visit`, output-boundary inspection, front/pop, root traversal, allocation
projection and append must understand the envelope/event. A return event
owns no guest roots or output capture. Existing frame root tracing and
abandonment classifications remain unchanged.

For valid queues, every outward context is a prefix of the installed full
stack. Assert that the requested length is available; silently clamping it
would recreate the bug. Use test-only full-prefix observations to verify
content, depth and entry epochs, not just lengths. Existing guest-file scope
restoration is a special **already implemented** prefix transfer:
`R/session/guest_loading/scope.rs:18-29,180-195`. Retain that responsibility
and test it with the new context restoration; remove only its obsolete
comment about the omitted two entries. Do not invent a new guest-file stack
or restore historical definition state from the queue record.

#### Epoch and native-stack poll state

Preserve the entries' captured epochs; **do not restore the global epoch
from a frame record**. A completed inner assignment must remain visible
when the outer frame runs. The queued execution also keeps that epoch
advance. Resetting it would turn a context repair into a definition-cache
and cycle-detection bug.

Logical stack/depth/epoch equality with the queued path is the requirement.
Literal equality of the native-stack base address or of its old polling
high-water mark is not meaningful after flattening native activations into
frames. The byte guard measures actual addresses
(`R/eval_core/mod.rs:100-110`); preserving an expired native address is wrong.

Recommended native policy: save/restore the displaced base and next-check
value at the resume entry; rebase to the live resume segment; on entering a
resumed frame, check current native headroom on this cold path and set the
next depth poll to the restored logical depth plus the existing stride of
four. Nested real evaluation retains its existing polling implementation.
Do the same when the driver schedules an `Evaluate` action at a newly
restored context. There is no entry recursion-limit rejection for an already
active resumed level; newly entered child evaluations still perform the
ordinary recursion-limit check. This policy never lets stale high-water
depth from a previous segment defer the new segment's polls.

This is safety equivalence under a new physical stack, not a claim of
identical native byte consumption or identical raw poll thresholds. If the
director wants exact historical polling cadence instead, that is a separate
choice requiring a precise native-segment contract; see section 6. Do not
declare the scalar candidate sufficient for that stronger, physical claim.

#### Why drop-two disappears, including matcher re-entry

The scratch witness
`MatchQ[{1,2},{x___,y_ /; (InputString["guard"];False),z___}]`
pauses twice and returns False in both arms. Its guard compound C has six
nodes and its input I three. The outer MatchQ is untracked. The heuristic
currently erases C and I, allowing a retry of the same guard.

The precise model initially preserves `[C,I]` and their real depths. It
removes I at I's return event, keeps C while its compound continuation
finishes, then removes C at C's return event **before** the matcher consumes
the false result and tries another binding. The matcher frames are at the
outer MatchQ level, not C's level. Re-evaluating C is then legal, while
actual recursion into an active same-epoch C remains illegal.

The relevant joins are `R/definitions/rhs_guards.rs:202-224` and
`R/pattern_matching/sequence/optional_sequence/choices.rs:181-207,224-230`.
The latter installs multiple helper frames at one level and retries on a
false match. This demonstrates the mechanism described by the surviving
historical comment; it does not pretend to recover the historical commit's
unavailable exact testcase. No portion of the top-two heuristic remains.

#### Accounting and the independent falsifier

Introduce a cold `truncate_accounted_evaluation_stack(target)` used by both
driver arms and return events. With no active memory limit, truncate the
vector. With a limit, release **each removed entry's expression occurrence**
using the same arithmetic as `pop_cold_armed_evaluation_entry` /
`pop_accounted_evaluation_entry`
(`R/debug_tools/resources/maintained.rs:174-194`). Preserve capacity: the
new contribution is

```
capacity * size_of::<EvaluationStackEntry>() + sum(E(entry.expr) for retained entries)
```

Do not subtract the removed entry shell when vector capacity stays allocated,
and do not discount shared expression occurrences. Update both the evaluation
stack contribution and the maintained session total through
`replace_accounted_stack_bytes` (`maintained.rs:219-224`). Sticky invalid
arithmetic stays invalid. Never raw-truncate and repair the total at a later
checkpoint.

Installation/final restore continue using
`replace_accounted_evaluation_stack` (`maintained.rs:196-205`). Refresh pending,
resuming and abandonment owner charges when cells or snapshots move, as the
driver already does. Extend `FrameQueue::storage_bytes/projected_growth`
(`core/queue.rs:108-135`), the existing pending-frame admission projections
(`core.rs:542-552,578-586`) and the independently written occurrence model
(`R/debug_tools/resources/occurrence/suspension.rs:12-25,28-49`) for the actual
larger cells, return events and full saved snapshot. A marker costs queue
storage, not an invented frame Arc allocation. The extra two saved roots
also count while pending.

`mode.stack_push` is **not the maintained byte total**. Its public snapshot
is explicitly representative diagnostics (`R/accounting_mode/counter.rs:5-18`).
The native guard is created at `R/eval_core/mod.rs:75` and dropped at `:90`,
including on InputRequired. Those guards are gone before the next host
resume. A restored expression entry is not a surviving StackGuard: truncating
it must not decrement an already-dropped guard or synthesize a second drop.
Preserve native-guard push/pop balance; new nested evaluator calls continue
to use `mode.stack_push`. No new per-entry shadow field or ordinary-path
instrumentation is needed. Redefining the representative shadow to count
logical suspended activations would be a separate diagnostics change.

The byte-accounting falsifier is
`Session::assert_maintained_memory_accounting`
(`R/debug_tools/resources/maintained.rs:109-149`), including
`assert_evaluation_stack_accounting` (`:227-247`). It compares against
`occurrence::session_breakdown`, not against the new truncation helper's own
formula. Exercise it before/after every context change in runtime unit tests,
including locally armed/unarmed ancestors, nested constraints, guest files,
second suspension, fault, cancellation and final cleanup. The current
`maintained_stack_counts_shared_occurrences_and_retained_capacity`
(`R/tests/session_memory_maintained.rs:374-409`) supplies the shared-root and
capacity control. Also assert shadow `pushes == pops`, `live == 0` after native
unwind/completion, with a fresh-arm control. The stdlib scratch integration
build does not enable rum-runtime's `cfg(test)` falsifier; no such coverage is
claimed for this design run.

#### The skipped epilogue is observable

Run it once per completed **evaluation level**, after restoring its parent
context, before its fixed-point consumer or enclosing constraint can run.
Do not run it once per helper frame. This matches
`R/eval_core/mod.rs:77-96`. An error return does not gain a success check.

The fixed-point resumer often hides the omission by checking the changed
value on its next pass (`R/resolution.rs:419`). But when `next == current`,
it returns at `:404-405` without that check. Constraint finishing then removes
the active local limit (`R/debug_tools/resources/constraints.rs:249,275`).
Both time and memory bypasses are therefore real, rather than an
unobservable duplicate-check omission.

The scratch test registers two test-only native builtins, never production
builtins. Each evaluates `InputString["epilogue"]`, then completes with the
exact unchanged original call. On a later evaluation it returns 77, without
asking for another input. In both arms, after the input child's last check:

* `Brief612Same[]` sleeps 150 ms. Guest source is
  `TimeConstrained[Brief612Same[],1/10,99]`.
* `Brief612Grow[]` installs an 8,000,000-byte string OwnValue. Guest source is
  `MemoryConstrained[Brief612Grow[],5000000,99]`.

Queued execution checks the completed builtin level while the local limit
is active and returns 99. Resumed execution skips that check, exits the local
constraint, then reevaluates the escaped unchanged call outside the local
limit and returns 77. Both resumed arms assert one actual input pause. The
memory witness is deterministic; the time witness uses a generous crossing
margin and does not use host waiting time as the differentiating operation.
These are rum-versus-rum embedding-extension witnesses, not claims that those
two test builtin names are installed language features.

Resume metadata changes the modeled size of suspended state. Very tight
MemoryConstrained thresholds can consequently differ from queued execution
because the documented occurrence model charges queue storage. Preserve the
model, do not normalize that difference away. The 5 MB / 8 MB witness leaves
ample separation from this bookkeeping cost.

#### Re-suspension proof and concrete second-pause states

Invariant at entry to each resumed helper: its live prefix and depth equal
the queued helper's; the global epoch reflects all work completed so far.
Its remaining work executes normally. If a child suspends, the full snapshot
therefore consists of exactly that prefix plus the newly entered child
levels. Its native unwind records fresh inner contexts and return events.
Append the still-recorded old suffix (`core.rs:817`); do not append the
completed old frame, and do not duplicate its level's still-pending return
event. The next installation satisfies the same invariant. This proves the
second pause and subsequent pauses by induction, not by a drop count.

Concrete checks of that reasoning:

* Direct Table: both pauses must snapshot `[T,I] / 2`; after each I return,
  the Table frame runs at `[T] / 1`. Current code's second snapshot is empty.
* Changing Table: both pauses must snapshot `[F@current_epoch,C,I] / 5`,
  and the Table handoff must be `[] / 1`. Current second snapshot retains
  `[F@e1,F@e2]` and restores depth two.
* Mid-loop twice, the observed passing control:
  `Table[(t=i;If[Or[i==2,i==4],h[k[InputString["x"]]],i]),{i,5}]`.
  T, its outer compound and If exceed eight nodes. At iterations two and
  four the queued tracked prefix is `[h[k[I]],k[I],I]`, depth six. The
  repaired model snapshots that same prefix/depth each time, with the
  current epoch. Present code retains the outer h at each pause; the
  intervening `t=i` changes epoch, so this short control still passes.
* Two pauses within one body:
  `Table[{h[k[InputString["a"]]],h[k[InputString["b"]]]},{3}]`.
  T and the two-item list are untracked. At either child input the queued
  tracked prefix is its current `[h[k[I]],k[I],I]`, depth five. Before
  starting the second sibling, h and k from the first sibling have returned.
  The precise model removes them; current code carries the first h into
  the second snapshot and eventually fails on repetition. The probe
  actually reaches both first-body pauses before that failure.

`Evaluate` actions and token-driven self-repetition obey the same invariant:
the newly inserted next/fresh frame records the level currently executing,
while the older evaluation-return event stays in the outer suffix. A
second InputRequired is not an evaluation completion.

#### Interaction with G13 and closure of the class

S2's typed driver needs context restoration around its existing erased
dispatch, deferred Evaluate and Finalize paths, plus recognition of the
private return event. Its exact decoding, fatal fault latch, rooted consumer
and drop order remain intact. The legacy arm needs the same restoration
around its Expr transport. Neither arm guesses whether a helper's output is
a guest value. The explicit event is where an evaluator `Expr` is required.

S6-S54 (49 steps), F1-F23 (23 steps), and R (one step) remain **73 steps**.
None changes frame signatures, witnesses, tokens, output bindings, frame
ownership, producer classifications or flip order for G14. The event is not
a new `TypedFrame` implementation, and does not advance the S5 receipt.
R removes the obsolete Legacy payload/arm but retains the context envelope
and return event. Source positions, fingerprints and layout assertions will
move and must be replayed/disclosed; this is maintenance, not a new family
typing obligation. The two necessary cold evaluator suspension-site hooks are stated
explicitly above rather than hidden as a claimed queue-only implementation.

Extend E1 with a dedicated fixture module using `Fixture`, planned `Pause`s
and `accept(Checks::ALL)`. Preserve unconditional FaultProbe checks. Port
all 32 ordinary rows below with **correct queued expectations**, and both
resource witnesses with expected 99; do not port diagnostic assertions of
the broken outcome. Assert exact pause kinds/prompts/counts, exhausted input,
values, termination, messages, prints, raw traces, log and state projections,
and a fresh subsequent request (`O/descriptors.rs:131-184,191-203`). A fixture
that terminates after one of three intended pauses must remain red.

Add the following explicit dimensions, retaining generated descriptor IDs
and a failure-count ledger:

* Zero, one, two, three, four and six wrappers between input and repetition;
  nested heads and lists, arithmetic, and DownValue/SubValue/UpValue/OwnValue
  bodies. Keep mutation-free identical-expression controls alongside effect
  controls. Logging with AppendTo changes epoch and can mask this defect.
* First/middle/last loop position, two pauses in the same body, two separated
  mid-loop pauses, and a pause in a guard followed by a pause in its RHS.
* Changing expressions for 1,100 and 2,000 resumes at the default limit,
  plus 100 resumes at limit 32. The queued run must finish too; do not raise
  the recursion ceiling to make the resumed run pass.
* Boundary sizes of eight and nine nodes; tracked and untracked parents
  mixed at the same logical depth; exact Stack[] output after input.
* Matching backtracking with an identical guard, genuine active cycles that
  still fail, nested TimeConstrained/MemoryConstrained with completion-only
  breaches, and meaningful side effects that do not mutate the epoch.
* Both queue modes, widening, all witness/token/chain routes, deferred
  Evaluate, Finalize, errors, cancellation and reset at each of two pauses,
  guest-file completion, and a structural slot fault. Internal route/stack
  assertions belong in runtime tests; public observations remain in E1.

Class mutation falsifiers must independently restore drop-two, omit context
restoration, derive depth from length, lose a record in append/widen, rewind
the global epoch, and omit the evaluation-return check. Each mutation must
make its assigned red descriptors fail; an ordinary no-input control must
continue passing. Do not add a waiver list for failed descriptors or weaken
the catalog's existing coverage equality.

**What the existing coverage already exercises.** The reported 133 are
Rust test functions, not 133 individual source programs. In particular,
`oracle::fixtures::historical_label_ledger_and_equality`
(`O/fixtures/mod.rs:86-115`) runs 209 fixtures, including all 195 brief-548
labels. The historical artifact records 104 equal controls, 91 divergent
continuation probes and 11 rows with repeated actual suspension; these are
historical results, not a fresh execution here.

| Current test/descriptor family | Relevant earlier passing shapes and why they miss G14 |
| --- | --- |
| `historical_label_ledger_and_equality` | `brief-548.Do`, `Table`, `Map`, `Select`, `AllTrue`, `AnyTrue`, `NoneTrue` (`O/fixtures/loops.rs:15-25`): shallow input paths, changing callback arguments and log mutations; no long unchanged-expression repetition. |
| Same historical aggregate | `brief-548.Block`, `Module`, `With` (`fixtures/scoping.rs:12-14`), `Input-nested` (`loops.rs:29-42`), `Set-destructure` (`historical-labels.tsv` and selected artifact): scope/assignment/output checks, not retained stack/depth checks. |
| Same historical aggregate | `Match-repeated`, `Match-orderless`, `Match-sequence-rest`, `ReplaceList-sequence-rest` (`fixtures/patterns.rs:19,22,58-59`): genuine matcher re-entry, but logging/seen mutation and shallow tracked guards protect them from stale same-epoch entries. |
| Same historical aggregate | `TimeConstrained-fallback`, `MemoryConstrained-fallback` (`loops.rs:62-75`): pause in the fallback **after** the local breach was consumed, not in a still-constrained completion. |
| `installed_catalog_and_descriptor_coverage_are_exact` | Executes `Table.body/bounds`, `Do.body/bounds`, `If.test` through control-role descriptors (`manifest.rs:93-115`, `control_roles/mod.rs:45-58`). Five visits, with count/log/state mutation (`control_roles/loops.rs:16-29`, `support.rs:43-68`), cover middle/twice but not this depth/identity axis. |
| `runner_scan_middle_and_twice` | `crates/rum-stdlib/tests/suspension_oracle.rs:18-38`: input at two/four in a five-item callback; logs change the epoch. |
| `s3_builtin_none/some/empty_some`, `prepared_twice`, `head_map_twice`, `s4_argument_twice` | Existing If/argument/head continuations (`fixtures/evaluation_dispatch.rs:32-112`, `argument_traversal_resolution.rs:20`); they protect slot transport and a small number of input handoffs. |
| `shared_completion_variants_and_lifecycle`, `original_panic_witnesses_complete` | `fixtures/shared_completion.rs:313-370`: includes two identical `(InputString["delete"];False)` predicate bodies. Here the heuristic removes the tracked compound and InputString; retaining the compound without a precise return would reject the second predicate. |
| S5 `down_*`, `sub_*`, `up_*` RHS tests | `fixtures/definition_application.rs:5-21,61-148`: one application, sometimes a guard pause then an RHS pause, with AppendTo; no repeated unchanged application after its completion. |

These are concrete overlaps and mechanisms, **not a claim that every green
historical probe causally requires exactly two deletions**. Some would pass
with one deletion or with leaked entries because their expressions/epochs
change. Determining that counterfactual exhaustively requires a permitted
mutation run. This brief did not modify the heuristic. A runtime control
outside the 133, `table_iteration_accumulator_completes_after_multiple_input_suspensions`
(`R/tests/iterative_loop_drivers.rs:155-165`), is the exact shallow direct-Table
shape that the heuristic covers.

#### Cost and gates

The ordinary successful evaluator receives no context field, new store,
allocation, counter load or new test for suspension. Context capture happens
only in already-cold frame insertion, return-event creation inside the two already-selected
InputRequired arms, and the resume/abandon drivers. No extra InputRequired
discriminator is added to `eval_core`, including its other error exits. The successful epilogue stays where
it is. Check generated code: source placement is not performance evidence.

Resume cost is two scalar context words per frame cell, an inline return
event per suspended evaluator level, a context restore per frame/event,
native polls on resumed entries, and one formerly skipped resource check per
successful completed level. Truncation releases only removed entries, not a
walk of every retained prefix. Restored tracked entries have at most eight
nodes; all costs, including queue capacity growth and two additional saved
roots, must be attributed. There is no new Arc allocation per return event.

Use the final S5 panel at `.temp/brief-611/r3/perf/labels.json`: it contains
107 ordinary CLI labels and 32 resume labels. The implementation brief below
requires allocations first, then one-codegen-unit instruction slopes for
both arms, all 107 ordinary labels unchanged, and an ordinary microbench
timing A/B. Keep old failing long-run semantics out of a “faster” performance
comparison: correctness must be checked at N and 2N before a label counts.
G14's new 2,000-resume correctness run cannot be timed against an old build
that stops at 1,021 and called an A/B speed result.


### 4. Witnesses run and their output

The only cargo command shape run was the brief's prescribed scratch target,
with the additional scratch-location environment variables. It ran four
times, serially, with logs `run1.txt` through `run4.txt`:

```text
TMPDIR=/home/remi/dev/rum/.temp/brief-612/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-612/target python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief612_probe
```

The command used a persistent tool session, with no shorter command timeout
than the 900-second process-group bound. No overlapping command ran while
cargo was active. Run 1 failed to compile the test because six derive-related
imports/trait obligations were missing. Run 2 compiled and failed all three
diagnostic tests: one scratch output path was relative to cargo's crate cwd,
and the two extension witnesses requested another input after their unchanged
call escaped its constraint. Both were test-harness defects. The final source
uses an absolute in-repository output path and a one-shot native completion
flag, so the escaped call reevaluates to 77 and both arms finish. Run 3 passed
3/3 in 0.60s. Run 4 added the identical-Map, changed-epoch mid-loop-twice and
print-side-effect controls; its complete output is:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.51s
     Running tests/zz_brief612_probe.rs (.temp/brief-612/target/debug/deps/zz_brief612_probe-a0c7fde4e640a5c7)

running 3 tests
test brief612_diagnostic_matrix ... ok
test brief612_skipped_epilogue ... ok
test brief612_skipped_memory_epilogue ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s

```

The following is the **verbatim** final matrix output. `equal` compares the
complete `Observation` (value/error/termination/messages/prints/raw trace) and
`{Global`log,Global`t}` state, not the shortened rendered value. Long values
are abbreviated by the probe with their rendered byte count. `pauses` counts
actual resumed input requests; queued runs consume the same planned answers
up front. All resumed seeds assert at least one actual suspension. Exact
planned completion counts are intentionally recorded rather than required in
this diagnostic matrix, because early termination is the defect; the drafted
permanent descriptors require the full planned counts. Sources and setup for
every label are in the preserved `probe.rs`; the unwrapped definition setup
for the `false-cycle-*`, `depth-*`, `pass-map`, `map-identical` and
`low-limit-changing` rows is `f[x_]:=(InputString["rhs"];7)`.

```text
label	pauses	equal	resumed	queued	state
false-cycle-plus	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=14; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
false-cycle-table	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
false-cycle-do	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=Null; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
subvalue	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
ownvalue	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
upvalue	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
nested-lists	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[List[List["x"]], List[List["x"]], List[List["x"]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
head-list	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[h[List["x"]], h[List["x"]], h[List["x"]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
nested-heads	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[h[k["x"]], h[k["x"]], h[k["x"]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
arithmetic	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[2, 2, 2]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
nested-identities	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List["x", "x", "x"]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
depth-leak-1100	1021	false	value=TerminatedEvaluation["RecursionLimit"]; termination=Some(RecursionLimit); error=None; messages=["reclim"]; prints=[]	value=List[7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,...[3304 bytes]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], 1022] / List[List[], 1100]
pass-direct-1100	1100	true	value=List["x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", ...[5504 bytes]; termination=None; error=None; messages=[]; prints=[]	value=List["x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", "x", ...[5504 bytes]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
pass-if	3	true	value=List["x", "x", "x"]; termination=None; error=None; messages=[]; prints=[]	value=List["x", "x", "x"]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
pass-module	3	true	value=List["x", "x", "x"]; termination=None; error=None; messages=[]; prints=[]	value=List["x", "x", "x"]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
pass-map	3	true	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
pass-set	3	true	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
mid-loop-twice	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[1, h[k["x"]], 3, h[k["x"]], 5]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
twice-one-body	2	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[List[h[k["x"]], h[k["x"]]], List[h[k["x"]], h[k["x"]]], List[h[k["x"]], h[k["x"]]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
matcher-backtrack	2	true	value=False; termination=None; error=None; messages=[]; prints=[]	value=False; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
stack-pass	3	false	value=List[List[], List[], List[]]; termination=None; error=None; messages=[]; prints=[]	value=List[List[Hold[CompoundExpression[InputString["x"], Stack[]]]], List[Hold[CompoundExpression[InputString["x"], Stack[]]]], List[Hold[CompoundExpression[InputStr...[181 bytes]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
wrapper-depth-0	3	true	value=List["x", "x", "x"]; termination=None; error=None; messages=[]; prints=[]	value=List["x", "x", "x"]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
wrapper-depth-1	3	true	value=List[h["x"], h["x"], h["x"]]; termination=None; error=None; messages=[]; prints=[]	value=List[h["x"], h["x"], h["x"]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
wrapper-depth-2	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[h[h["x"]], h[h["x"]], h[h["x"]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
wrapper-depth-3	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[h[h[h["x"]]], h[h[h["x"]]], h[h[h["x"]]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
wrapper-depth-4	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[h[h[h[h["x"]]]], h[h[h[h["x"]]]], h[h[h[h["x"]]]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
wrapper-depth-5	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[h[h[h[h[h["x"]]]]], h[h[h[h[h["x"]]]]], h[h[h[h[h["x"]]]]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
wrapper-depth-6	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[h[h[h[h[h[h["x"]]]]]], h[h[h[h[h[h["x"]]]]]], h[h[h[h[h[h["x"]]]]]]]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
low-limit-changing	29	false	value=TerminatedEvaluation["RecursionLimit"]; termination=Some(RecursionLimit); error=None; messages=["reclim"]; prints=[]	value=List[7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,...[304 bytes]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], 30] / List[List[], 100]
map-identical	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=[]	value=List[7, 7, 7]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], t] / List[List[], t]
mid-loop-twice-changing	2	true	value=List[1, h[k["x"]], 3, h[k["x"]], 5]; termination=None; error=None; messages=[]; prints=[]	value=List[1, h[k["x"]], 3, h[k["x"]], 5]; termination=None; error=None; messages=[]; prints=[]	state=List[List[], 5] / List[List[], 5]
false-cycle-side-effects	1	false	value=TerminatedEvaluation["IterationLimit"]; termination=Some(IterationLimit); error=None; messages=["passlim"]; prints=["before"]	value=Null; termination=None; error=None; messages=[]; prints=["before", "after"]	state=List[List[], t] / List[List[], t]
```

The two additional constrained-completion outputs, also verbatim:

```text
local-time-epilogue
queued=false; pauses=1; value=77; termination=None; error=None; messages=[]; prints=[]
queued=true; pauses=0; value=99; termination=None; error=None; messages=[]; prints=[]
local-memory-epilogue
queued=false; pauses=1; value=77; termination=None; error=None; messages=[]; prints=[]
queued=true; pauses=0; value=99; termination=None; error=None; messages=[]; prints=[]
```

No full workspace/oracle suite, runtime unit instrumentation or performance
A/B was run: this brief authorizes only the one scratch integration target.
The earlier 133-test and 195-probe coverage discussion is source/artifact
review, explicitly separate from the current execution evidence.

### 5. Repair sequence and implementation brief

Recommend **one atomic implementation brief**, before S6, with these internal
milestones. Splitting stack repair from the completion epilogue would leave
known constraint bypasses and an incomplete evaluator model at a gate.

1. Port the 34 seeds to strict E1 descriptors and collect a full red ledger
   before production changes. Brief-612's evidence predicts 25 differing
   pairs and nine equal controls. Add the remaining route/accounting/native
   invariants and record their own baseline failures separately.
2. Introduce the mandatory queue envelope, full snapshot, cold accounted
   context restoration and two suspension-only return-event insertions.
   Preserve every legacy/typed/witness/token/chain/deferred route and nested
   transfer. Check the three internal traces and the node-count boundary.
   Context and Stack[] divergences should disappear; completion witnesses
   remain red until the next milestone. Treat this as an intermediate work
   state, not a passing final gate.
3. Drive successful evaluation-return epilogues before enclosing consumers;
   preserve errors, finalizers, faults, capture scopes and abandonment. Apply
   the approved native-segment polling policy. Reconcile all owner charges
   with the independent falsifier. The full G14 failure count must reach zero,
   including both completion witnesses and 2,000 resumes.
4. Run class mutants/lifecycle/guest-file controls and both-feature E1 gates,
   replay inventories, then run allocation and instruction gates followed by
   the required ordinary timing A/B. If any ordinary label regresses or any
   resume delta is unexplained, fix or report it; do not trade it for a new
   correctness waiver.

The draft below is review material for the director to rewrite and number,
not an authorization to implement during brief-612. Its standalone copy is
`.temp/brief-612/draft-G14.md`.

---

# DRAFT — G14: restore evaluation context and completion checks on resume

Director review/rewrite required before dispatch. This is one atomic repair
brief; do not ship a stack-only intermediate fix. The brief number and the
director-provided base export must be assigned before execution. In this draft,
`SCRATCH` means the assigned `.temp/brief-NNN/` directory inside this repository.

## Goal

Queued-input and resumed execution must reach the same logical evaluation
stack, depth and epoch at each continuation level, and run each successful
evaluation level's resource epilogue before its result leaves that level.
Remove G14 false cycles, depth growth and completion-only constraint bypasses
through a private queue-owned context and evaluation-return event, without
changing frame implementations or imposing ordinary-path work. Preserve G13's
S5 typing receipt and all remaining family/flip steps.

## Context pointers

Read `AGENTS.md`, `docs/testing-strategy.md`,
`docs/code-organization-guidelines.md`, `docs/guide/evaluation.md:268-300`,
`docs/guide/resource-limits-and-sandbox.md:294-331`, and the complete brief-612
report first. Its three static traces are the acceptance model, not debug
instrumentation already available in production.

Key owners, relative to `crates/rum-runtime/src/runtime/`:

* `suspension/core.rs:492-828`, `core/queue.rs`, `typed/entry.rs`: snapshot,
  envelope, dispatch, deferral and nested ownership transfer.
* `resolution.rs:367-382,434-444`: the two evaluator suspension-only sites
  that must insert a return event before their fixed-point consumer.
* `eval_core/mod.rs:28-96`: the unchanged native level/epilogue contract.
* `debug_tools/resources/maintained.rs:154-247` and
  `resources/occurrence/suspension.rs`: byte maintenance and its independent
  falsifier.
* `accounting_mode/counter.rs:5-18,62-102`: native shadow guards, not the byte
  ledger and not suspended logical activations.
* `suspension/core/{abandon,fault}.rs` and `session/guest_loading/scope.rs`:
  structural cleanup and caller-prefix restoration.
* `crates/rum-stdlib/tests/suspension_oracle/{descriptors.rs,support/,fixtures/}`:
  the established rum-versus-rum runner.
* `.temp/brief-612/probe.rs`, `matrix.tsv`, `epilogue.txt`,
  `memory-epilogue.txt`: 32 ordinary seeds and two native completion witnesses.
* `.temp/brief-611/r3/perf/labels.json`: 107 ordinary and 32 resume labels.
  Copy the final S5 harness and label programs into SCRATCH before use. Never
  modify the preserved earlier evidence or silently substitute a statement.

## Scope

Objective: repair evaluation-context ownership and returns, not individual
continuations or G13 transport types.

Files/areas you MAY modify, with a disclosure duty for every changed file and
every moved or added inventory row:

1. `crates/rum-runtime/src/runtime/suspension/core.rs`,
   `suspension/core/{queue,abandon,fault}.rs`, and a new sibling
   `suspension/core/evaluation_context.rs` for the new responsibility.
2. `crates/rum-runtime/src/runtime/suspension/typed/entry.rs` for the private
   payload/envelope traversal; `typed/witness.rs` for a crate-private
   `SlotWitness<Expr>::suspend_evaluation_return` helper and an internal
   `evaluation_return => frame` macro form selecting it. Constrain that
   helper to `TypedFrame<Input=Expr, Output=Expr>`. Existing macro forms,
   public types, witnesses/tokens and all frame contracts stay unchanged.
3. `crates/rum-runtime/src/runtime/resolution.rs`: only the two existing
   InputRequired construction branches. Do not add a new success-path field,
   store or branch. The ordinary `eval_core` implementation is read-only.
4. `crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs`,
   `resources/memory.rs` and `resources/occurrence/suspension.rs`: cold
   truncation, queue storage and independent accounting only.
5. `crates/rum-runtime/src/runtime/session/guest_loading/scope.rs`: retire its
   obsolete top-two comment only, unless a demonstrated prefix-restoration
   defect makes a behavioral change necessary. Such a change requires stopping
   and reporting its exact counterexample before extending this scope.
6. Runtime tests: new `runtime/tests/resume_evaluation_context.rs` and its
   module declaration; `tests/session_memory_maintained.rs`; the existing
   `suspension/typed/tests/{live,accounting,driver_faults,file_faults}.rs` and
   their test module declarations where needed. Test-only observers must not
   introduce any code or field into ordinary release evaluation.
7. Stdlib tests: new
   `tests/suspension_oracle/fixtures/evaluation_stack.rs`, its registration in
   `fixtures/mod.rs`, and a test-only mutation/helper module beside it if
   required. Use existing descriptor and runner contracts. Register additional
   descriptors against real existing roles if they are included in the catalog
   registry; do not invent a guest-effect role just to accommodate a fixture.
8. SCRATCH artifacts, this brief's appended report, and the standing inventory
   maintenance described below.

Every source change carries TEMPLATE.md's maintenance duty. Replay retention
contract fingerprints; run `local_production_acceptance` in both feature
configurations and copy its generated certificate cells; reconcile ownership
discovery against actual source. Disclose every changed row. New
UnknownConstruct rows for changed code remain unknown and are disclosed by
file/line. STOP for a changed/disappearing existing identity, status,
disposition or exclusion, a new UnknownHostPayload, or an unknown row pointing
outside touched code. Do not hand-write fingerprints or certificate chains.
If a genuinely new traced/frame type is introduced, add exactly its required
runtime root-trace closed-list entries and explain it. Prefer an inline queue
return event, which is not a new named SuspensionFrame and does not advance
the G13 frame census.

MUST NOT modify unrelated evaluator/matcher/builtin semantics, the ~600 frame
implementations, Expr/Symbol representation, public continuation/serde shapes,
G13 step assignments, dependency manifests/lockfile, AGENTS.md, TODO.md,
PROGRESS.md, other briefs or ordinary label statements. No new diagnostics
load on the default path. No git. A needed change outside this MAY list is a
STOP with concrete evidence, not permission inferred from a broad objective.

## Deliverable

1. **Red tests first.** Before production edits, port the 34 brief-612 seeds
   through `Fixture` and `accept(Checks::ALL)`, using correct queued outcomes.
   Record descriptor ID, setup/source, actual versus planned pauses, queued
   result, resumed result, all differing observation channels and projected
   state. The brief-612 evidence predicts 25 unequal pairs; rerun and record
   rather than treating that number as a substitute for the red run. Preserve
   passing controls. Do not add ignored/expected-bug tests to the final suite.
2. Extend the descriptors over depths 0/1/2/3/4/6, all four definition kinds,
   nested lists/heads, arithmetic, guard backtracking, exact Stack[] output,
   eight/nine-node tracking boundaries, first/middle/last iteration, same-body
   twice, separated mid-loop twice, and 1,100/2,000 changing-expression resumes
   with the default recursion ceiling. Include the limit-32/100-resume case.
   Keep mutation-free controls: AppendTo/log assignments can hide same-epoch
   cycles. Include nonempty prints/state projections and actual planned pauses.
3. Record full saved stack entries and original entry epochs in test-only
   observations; compare frame-entry contexts to queued helper levels for the
   three brief-612 traces. Cover Legacy, Mixed, widening, witness, fresh token,
   FrameChain, both deferred Evaluate arms, Finalize, errors and nested append.
   Direct API/typed-state controls may be runtime unit tests; E1 remains the
   public behavior check. Add a closed source-boundary control for the only
   two evaluator callers and their marker-before-consumer insertion.
4. Implement the private context envelope and full input snapshot. Capture
   stack length and logical depth at every actual insertion, including driver
   pushes. Transfer records unchanged during widening/splicing. Truncate with
   accounting before dispatch. Do not reset the global epoch, and do not derive
   logical depth from tracked length. Do not count helper frames as levels.
5. Select the internal evaluation-return macro form at the two fixed-point
   sites. Its private Expr witness helper inserts one inline EvaluationReturn
   event after native cleanup and before witnessed insertion of the fixed-point
   frame. Do not sequence a mutable Session call inside a later frame argument
   while the earlier Session argument is borrowed. There is no runtime flag
   and existing macro success/error expansions stay unchanged. At that event, restore parent context, then check only
   successful Expr results. Propagate errors without a success epilogue.
   A wrong successful slot is a structural driver fault. Retain the existing
   fault latch, Session-rooted consumer and drop-before-finalizer ordering.
6. Rebase native polling to the live resume segment, preserving/restoring the
   displaced base and threshold at the outer entry. On each resumed callable
   frame/deferred Evaluate entry, poll actual headroom on the cold path and
   set the next logical-depth threshold to restored depth + four. Route any
   limit failure through the existing error/cleanup protocol. Never restore
   an expired native address or mistake an already active level for a new
   recursion entry. This policy is the director's explicit approval of
   physical-safety equivalence rather than identical native addresses/cadence;
   if not approved in the rewrite, STOP before implementing a different policy.
7. Account every removed expression occurrence, preserved vector capacity,
   larger queue cell, inline event and additional saved root. Refresh owner
   transfers synchronously. Test `assert_maintained_memory_accounting` against
   its independent occurrence walker after each transition; test local arming,
   rearming, shared roots, retained capacity, second input, normal/error/fault
   completion, cancellation, authorized cleanup and guest-file prefix restore.
   Native shadow guards remain balanced; do not drop them a second time when
   deleting a restored expression entry.
8. Run class falsifiers that independently lose context, restore drop-two,
   set depth from length, lose metadata during widen/append, rewind epoch and
   skip the resource return event. Each strict descriptor must reject its
   assigned mutation; the no-input control still succeeds. Final production
   contains none of the mutants.
9. Produce per-label allocation/byte and instruction tables, ordinary timing
   A/B rows, native-stack evidence, exact verification output, inventories and
   a before/after failure-count ledger. State that S6-S54/F1-F23/R frame work
   is unchanged; disclose line/fingerprint/layout updates separately.

## rum / Rust pitfalls (read twice)

* Restore contexts before helpers execute, and evaluation parent context before
  an epilogue. Checking after TimeConstrained/MemoryConstrained removes its
  limit repeats the defect. Checking after every helper misinterprets typed
  or legacy transport state as an evaluated expression.
* An input re-suspension does not finish the restored level. Splice fresh
  inner work before its existing outer return event, exactly once.
* Native depth and tracked-stack length differ. Global epoch must reflect
  intervening mutations; captured entry epochs remain original.
* Context capture belongs to an already-selected suspension branch. Do not
  add speculative records, always-live locals, new Session fields, a disabled
  counter load or a per-evaluate suspension check. Even a cold source edit can
  change spills/inlining; the one-codegen-unit gates decide.
* Use safe Arc sharing and inline queue metadata. Do not deep-clone every
  prefix at every frame or add an Arc per return event. Keep original frame
  lifetime and typed-slot fatal-fault semantics intact.
* Existing resource accounting counts occurrences and queue storage. Do not
  weaken a memory assertion or discount sharing to hide larger queue cells.
  Large-margin completion witnesses must still fail for the intended reason.
* Timing comes from in-kernel monotonic AbsoluteTiming and the scripts.
  Program text in each label is exact: no added assignment/sink/Print.
  Instrumentation is off for timing. Never compare a prematurely terminated
  baseline to a completing candidate as a performance improvement.
* All instruction bases and candidates use
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, matching toolchain/flags/source
  paths and separate target directories. Repartitioning 16 codegen units is
  not evidence of default-path cost or its absence.

## Verification (MANDATORY — run to green / to evidence)

Commands are serial, offline, one build/test worker, bounded through
`python3 scripts/run-bounded.py --timeout <seconds> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<SCRATCH>/tmp
...`. Tool/session lifetime must exceed the whole bound. Use the prior S5
bounds: 900 seconds focused/build commands, 1,500 per oracle feature build,
3,600 workspace; revise only with measured evidence before a rerun. Keep
targets and raw output under SCRATCH. No CPU-heavy concurrent workload.

First run the new G14 oracle test filter on the unchanged production base;
save its expected failing output and an aggregate descriptor failure count.
The aggregate must attempt all descriptors, not stop on its first unwrap.
Then iterate focused correctness tests with counts after each compiled attempt.

Final verification, each command separately bounded:

```text
cargo fmt --check
cargo test -p rum-runtime --lib resume_evaluation_context
cargo test -p rum-runtime --lib session_memory_maintained
cargo test -p rum-runtime --lib suspension::typed::tests
cargo test -p rum-runtime --lib frame_root_trace
cargo test -p rum-runtime
cargo test -p rum-stdlib --test typed_slot_compile_controls
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls
cargo test -p rum-stdlib --test typed_slot_census
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
scripts/check-suspension-oracle.sh 1500
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
cargo test --workspace --no-fail-fast
```

Use bounded logical recursion controls and the unchanged below-native-limit
regression; no native-overflow/aborting probes. Report native ZeroMQ skips as
skips. Report every test count and both oracle fault-occurrence deltas.
Neither oracle build may introduce warnings. Preserve unrelated director
changes if a broad check fails; report their evidence instead of editing them.

Performance sequence:

1. Director supplies the unmodified S5 source export inside SCRATCH; the
   executing agent never obtains it with git. Freeze hashes and copy the final
   139-label panel/harness. Preflight result correctness and actual resume
   counts for every N/2N run. If an old resume label terminates early, report
   that pair as ineligible; do not silently change its source or shrink N.
   The director may specify a separately named valid measurement workload.
2. Build base/candidate CLI, allocation CLI, resume and allocation-resume
   harnesses with one codegen unit in separate targets. Use the already
   available allocation-counter machinery; add no unsafe.
3. Allocations/bytes first: all 107 ordinary labels must match exactly in
   whole-program N/2N counts and per-iteration slopes, not just the difference
   of two counts. Resume deltas must be completely attributed to context cell
   capacities, inline return-event capacity growth and full snapshot storage.
   There is no adapter/Arc rebox justification for a new return event.
4. Run two interleaved instruction pairs per arm across all labels, with
   stdin `/dev/null`, `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`.
   Slope is `(I(2N)-I(N))/N`; also report instructions per actual resume.
   Flag a delta above `max(0.05% of base slope, base range, candidate range)`.
   Flagged ordinary rows must be fixed or reported, not accepted as G14 cost.
   A flagged row receives either five samples per arm or at least six
   callgrind processes per arm, chosen before running; attribute every
   deterministic delta by address. Do not silently keep sampling noise.
5. Resume rows necessarily gain correct bookkeeping/checks. Attribute their
   deltas to queue/return/truncation/native-poll/resource operations and report
   them for the director's gate, including unchanged neighboring resume
   families. No arbitrary percentage is preapproved. Compare N and 2N slopes
   for bounded per-frame/per-level cost; demonstrate no depth-correlated
   growth in the new long correctness run. Do not use an old failing run as
   its performance baseline.
6. All ordinary memcpy counts must be no larger than base; literal-integer,
   do-count-null and do-iterator-null must show no extra deterministic
   instructions. Record debug/release prologue sizes for changed hot owners
   and bounded peel/plus stack controls. A code-size/spill increase caused by
   a suspension-only edit is still reported as a default-path regression.
7. Run the required `scripts/core-language-microbench.sh` base/candidate A/B
   serially, with distinct output directories and the same source statements.
   Paste per-label timings.tsv rows for all hot labels and same-tier neighbors.
   Best-of-two is the maximum timing repetition. Instruction attribution is
   the fine-grained gate; monotonic timing remains the repository hot-path gate.

Stop-loss and run budget: at most six compiled semantic repair attempts.
After each, record the total failing G14 descriptor/invariant IDs and any
new existing-suite regressions. **STOP after two consecutive compiled attempts
that reduce none of those failure counts**, or at the attempt cap, and hand
back the exact remaining cases. Reducing compiler errors is progress before
the tree compiles. Once correctness is green, two consecutive attempts that
reduce neither flagged ordinary rows nor unattributed resume deltas trigger
the same stop-loss. Mutant rejection tests are expected failures of mutants,
not permission to count a broken production test as green. Full final gates
run once per final candidate; repeat only for a subsequent relevant change.

## Checkpointing — protect the scene

After the red baseline, each compiled attempt, correctness closure, allocation
gate and instruction gate, write SCRATCH/PROGRESS.md. Record completed work,
exact next action, compile/test state, failure-count history, owned edits,
raw evidence paths and any takeover issue. Keep a readable checkpoint if a
bound, tool restriction or stop-loss ends execution. Leave source uncommitted.

## Hard rules

* Do not run git in any form. No network or outside-repository scratch.
* Do not edit outside the MAY list. STOP with evidence if the scope is
  insufficient. Do not infer a blanket exception from the maintenance duty.
* Testing Policy is permanent: rum behavior against its documented contract;
  resumed versus queued rum only. No third-party engine oracle or output.
* No production unsafe; no subprocess-aborting probes; bounded serial runs.
* No default-path field/branch/store/counter/allocation. Preserve ordinary
  semantics, frame lifetime, capture output and exact typed-fault behavior.
* Do not weaken tests, raise the recursion limit, remove a failing label,
  refresh a ledger by hand, or reclassify an existing ownership row to pass.
* Keep the patch within the queue/evaluation-boundary/accounting responsibility.
  New context logic belongs in a coherent sibling module. Disclose any
  touched file in the 20k-35k split-plan range and its chosen boundary.
* The director owns review, gate and commits; none of these draft instructions
  authorizes a commit, merge or publication.

## Report format (your final message)

1. Files/lines changed and why, plus every inventory row disclosure.
2. Red baseline and failure-count progression, exact final test output and
   both unconditional oracle fault-occurrence deltas.
3. Before/after internal contexts for the three traced witnesses, all route
   controls, resource/fault/lifecycle outcomes, and successful 2,000 resumes.
4. Allocation/byte N/2N rows, one-codegen-unit instruction slopes and
   attribution, unchanged 107 ordinary rows, stack/copy controls and timing A/B.
5. G13 transparency, limits of the native policy, unresolved costs or scope
   issues, and the stop-loss ledger. No claim of a green gate without output.


---

### 6. Open questions for the director, with recommendations

1. **Native polling equivalence.** Literal old native addresses/high-water
   cadence cannot represent the flattened resumed native stack. Approve the
   draft's cold actual-headroom poll and depth-plus-four rebase at resumed
   callable entries, with displaced state restored on exit. Require exact
   logical stack/depth/entry-epoch correspondence; require safety and bounded
   poll spacing for the physical stack. Do not silently call these identical
   physical states. This recommendation is designed, not measured here.
2. **Return-boundary ownership.** Approve two hooks in the existing evaluator
   InputRequired construction branches in `resolution.rs`, in addition to
   queue/driver changes. This avoids a new discriminator even on non-input
   error exits in `eval_core`, and avoids teaching hundreds of frames about
   epilogues. Keep a closed boundary-caller test so the hook cannot drift.
   No G13 typing/flip step changes.
3. **Resume cost acceptance.** Correct stack context and previously omitted
   checks necessarily add resume work. Recommend zero tolerated ordinary
   allocation/work regression, and director review of fully attributed resume
   deltas after one-codegen-unit measurement. Do not preapprove an arbitrary
   percentage or use a prematurely terminated baseline as a speed result.
4. **Resource witness choice.** Keep the deterministic native memory-growth
   witness as the primary completion-only red test, plus the bounded native
   delay witness for time. Both run the same rum program/extension in both
   arms and assert a pause. Native extensions are part of the embedding
   contract; a builtin-only replacement is welcome only if it preserves the
   unchanged-result fast exit and crosses the limit strictly after the final
   child checkpoint. Do not replace it with a loose time-at-input test that a
   different checkpoint already catches.

No user permission is needed to finish this design task. The questions are
specific decisions for the director's implementation-brief rewrite. The
working tree is left with no production edits and no scratch integration test.
