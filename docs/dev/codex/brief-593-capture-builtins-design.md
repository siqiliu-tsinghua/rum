# brief-593 — design: capture builtins under suspension (G11), and the uniqueness class

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

The E1 oracle's sixth drain (brief-592) found the first defects the census
never saw — two capture builtins whose frames exist but do not resume into
their work. Witnesses: `.temp/brief-592/defects/{EvaluationData.body,TestReport.input,TestReport.expectedOutput,TestReport.sameTest}.md`.

1. **`EvaluationData[expr, props]`** — `EvalEvaluationDataSuspensionFrame`
   (`runtime/debug_tools/evaluation_data.rs:30-52`, resume `:225-245`) does not
   hold the property selector, so the resumed path returns the whole capture
   association where the queued path returns the selected properties. And the
   prints the capture owns surface in the pause's response and then vanish from
   the final cumulative output — the queued path never surfaces them.
2. **`TestReport`** — the resumed path loses the surfaced print prefix of a
   test case, and when a second case suspends, finalization **panics the
   kernel**: `TestReportResultState::into_vec`
   (`runtime/debug_tools/test_report/report.rs:112-118`) demands
   `Arc::try_unwrap` uniqueness of an accumulator node that a suspension frame
   legitimately still shares. Guest code reaches a panic. This is the class of
   the resumed-`Table` panic the ownership branch found.

The design settles the repair of both, the **output semantics of a capture
that is suspended** (which prints and messages belong to the pause's response
and which to the capture), and the **class** behind the panic.

## What the design must settle

1. **EvaluationData.** Which state the frame lacks (the selector, and anything
   else the eager path uses after the capture), and how the resumed path reaches
   the same completion helper as the eager one.
2. **Captured output at a pause.** Read the request checkpoint
   (`session_requests.rs`), the debug capture checkpoint and the pause
   response. State what the contract should be so resumed equals queued: output
   a capture owns is not surfaced at a pause and is not lost, and output outside
   any capture surfaces exactly once, in order. Say where that rule lives (one
   place, not per builtin), which other capture builtins it touches (`Check`,
   `Quiet`, `Reap`, `TestReport`, `Timing` family, … — they passed the oracle,
   so say why), and how it relates to the queued item Q1 in `TODO.md` (one
   ordered print/message sequence from the kernel through the host) without
   absorbing Q1's scope.
3. **TestReport.** The lost print prefix and the panic; the repair that makes
   finalization correct when a frame shares the accumulator — without breaking
   the iterative drain the ownership branch built for these chains (the
   `AccumulatorChain` members must still drop without recursion).
4. **The uniqueness class.** Every production `Arc::try_unwrap` (32 sites in
   `rum-runtime` and `rum-stdlib`, tests excluded): classify each as (a)
   unique by construction, with the reason, (b) falls back safely when shared,
   or (c) **panics or misbehaves when a suspension frame shares it** — with a
   reachability argument, and for every (c) a witness through the rum-vs-rum
   oracle in the scratch test (resumed vs queued, never an aborting probe; a
   kernel panic surfaces as a caught test failure, record it, do not run it in
   a loop). Propose the structural repair for the class — a shape in which a
   shared node cannot reach a uniqueness assumption — not a per-site patch.
5. **The oracle consequence.** The four entries stay owed until repaired; say
   how the repair brief(s) attach their descriptors, and whether any already
   drained capture entry deserves a stronger descriptor (twice inside one
   capture, output on both sides of a pause).

## What I want back

1. The diagnosis of both defects with `path:line` evidence.
2. The captured-output contract, where it lives, and its reach.
3. The uniqueness-class table (all 32 sites) and the witnesses for class (c).
4. The repair design and its sequence; drafted implementation brief(s) in
   `TEMPLATE.md` shape — red tests through the oracle, the standing duty (a new
   frame enters rum-runtime's frame lists), one-codegen-unit instruction gates
   with base-vs-base noise floors where an ordinary path changes, serial runs.
5. Open questions for the director.

## Context pointers

- brief-592's report and its defect files; brief-586's oracle
  (`crates/rum-stdlib/tests/suspension_oracle/`), its runner and
  `support/normalize.rs`.
- `runtime/debug_tools/{evaluation_data.rs,test_report/}`,
  `runtime/session/session_requests.rs`, the debug capture checkpoint and
  `restore_evaluation_data_capture`, the suspension core.
- The ownership branch's accumulator chains (`AccumulatorChain`, the
  participation gate) and the resumed-`Table` repair; memory of that line is in
  `TODO.md` and `PROGRESS.md` (2026-09-21).
- `docs/guide/{evaluation,embedding}.md` (pause responses, cumulative output),
  `TEMPLATE.md`, `AGENTS.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-593/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief593_probe.rs` (it may reuse the oracle's
  support through a `#[path]` module), **deleted before the report is
  appended**, its final content copied to `.temp/brief-593/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository; all
  scratch and `TMPDIR` under `.temp/brief-593/`.
- Do NOT modify any file outside the exceptions; STOP and report instead.
- **Run no probe that aborts a process.** A caught kernel panic inside a test
  is a recorded witness, run once.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief593_probe`
  (300 s). File reads, `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The two diagnoses.
3. The captured-output contract.
4. The uniqueness class.
5. The repair design, sequence and drafted brief(s).
6. Open questions for the director.

---

**1. What changed — design only.**

No production code, documentation contract, oracle inventory or dependency file
was changed. This report is the only addition outside `.temp/brief-593/`.
The authorized scratch integration test was copied byte-for-byte to
[probe.rs](../../../.temp/brief-593/probe.rs) and deleted before this report was
appended. `Cargo.lock` retains its recorded SHA-256. No git, network, external
engine, aborting probe or new unsafe was used. Commands ran serially through
`python3 scripts/run-bounded.py`, with scratch, `TMPDIR` and the Cargo target
inside `.temp/brief-593/`.

The source audit found **25 class-(a) sites, four class-(b) sites and three
class-(c) sites** among all 32 production `Arc::try_unwrap` calls. Two of the 25
are dormant matcher-tail sites, explicitly qualified below. Each class-(c)
site has a once-executed, caught kernel-panic witness through the rum-vs-rum
oracle. Association Set selectors and DeleteCases retained siblings are
additional defects beyond the reported TestReport panic. A Quiet witness also
exposes missing intermediate-message checks in the current oracle.

Evidence and drafts:

- [Corrected probe log](../../../.temp/brief-593/probe-test.log),
  [first attempt log](../../../.temp/brief-593/probe-test-attempt-1.log),
  [Quiet pause evidence](../../../.temp/brief-593/quiet-message.txt).
- [32-site enumeration](../../../.temp/brief-593/try-unwrap-sites.tsv) and
  [classification with reasons](../../../.temp/brief-593/uniqueness-classification.tsv).
- [Draft A: shared accumulator completion](../../../.temp/brief-593/draft-A-shared-accumulator-completion.md).
- [Draft B: capture output and descriptors](../../../.temp/brief-593/draft-B-capture-output-and-descriptors.md).

The authorized test command was run twice, with the same environment and a
300-second bound:

```bash
TMPDIR=/home/remi/dev/rum/.temp/brief-593 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-593/target python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief593_probe > .temp/brief-593/probe-test.log 2>&1
```

The first build completed, but all tests stopped at the scratch evidence write:
Cargo's integration-test working directory is the crate directory, so relative
scratch paths were wrong. Its log was retained as `probe-test-attempt-1.log`.
None of the three resumed panic paths ran in that attempt. Only the evidence
paths were corrected to the authorized absolute scratch directory. The second
run recorded each kernel panic once; no further run was made. Its exit status
was 101, and its key output was:

```text
running 10 tests
test c_association_shared_selectors ... FAILED
test c_delete_cases_shared_kept_siblings ... FAILED
test c_test_report_shared_results ... FAILED
test control_prepared_arguments_and_restores ... ok
test control_reap_two_buckets ... ok
test control_table_twice ... ok
test evaluation_data_print_prefix ... FAILED
test evaluation_data_selector ... FAILED
test quiet_pause_message_visibility ... FAILED
test test_report_print_prefix ... FAILED

test result: FAILED. 3 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

These are deliberately red diagnosis tests, not a passing implementation gate.
The three controls exercise repeated Table input, two Reap combiner buckets,
and prepared arguments with Unevaluated restoration suffixes. This design-only
scope did not run workspace, ownership-regeneration or performance gates and
makes no performance claim.

For source references below, `R/` means `crates/rum-runtime/src/runtime/`,
`S/` means `crates/rum-stdlib/src/`, and `O/` means
`crates/rum-stdlib/tests/suspension_oracle/`. Line numbers describe the source
read for this audit.

**2. The two diagnoses.**

**EvaluationData loses the prepared selector.** The eager entry reads `expr`
and `properties` at `R/debug_tools/evaluation_data.rs:9-12`. After capturing it
calls `finish_evaluation_data_capture`, then
`project_evaluation_data_properties(data, properties.cloned())` at `:43-51`.
The frame constructor at `:30-36` and frame fields at `:207-214` omit
`properties`; resume calls only the capture finisher at `:232-242`. Thus the
resume path skips the final property operation entirely.

The scratch source `EvaluationData[InputString["selector"],"Result"]`, with
answer `a`, gives queued `"a"` and a resumed full association; the oracle
rejects `Value`. `EvaluationData` has `HoldFirst`
(`docs/reference/builtins.md:1612-1620`), so this selector is already prepared
when the builtin starts. Save an `Option<Expr>` on actual suspension and pass
it to a common completion helper; do not evaluate it again on resume.

No other eager post-capture input is absent from the frame: the four offsets
(message, print, trace, handled error), prior tracing flag, monotonic start
instant, original input expression and original input string are retained
(`evaluation_data.rs:188-214`). The resumed outcome is the resume argument.
Keep this state, finish the capture, and apply projection once from both paths.
Projection already handles absent/All, Properties, scalar/list selectors,
missing properties and invalid selectors through
`R/debug_tools/test_report/report.rs:533-544` and
`R/debug_tools/properties.rs:4-59`. Projection diagnostics occur after capture
completion; moving them into the capture would change ownership.

**The print-prefix defect belongs to request packaging.**
`R/session/session_requests.rs:1-40` records the request's original buffer
checkpoints. Initial input suspension stores that checkpoint at `:234-247`;
resume takes it back at `:287-318`, and another pause preserves it at
`:352-367`. Both pause paths call `finish_request_failure`, which unconditionally
copies every message and print after the request checkpoint at `:493-494`.
It cannot distinguish output still owned by a capture.

`EvaluationData` later consumes its messages/prints at
`R/debug_tools/evaluation_data.rs:74-75`. Therefore a captured pre-pause print
is prematurely public, then absent from the final response. It is not evidence
that the capture forgot that print: the checkpoint remains and the capture
finisher still gathers it. The scratch before/after-print case has queued
`{"Result" -> 7, "OutputLog" -> {"before", "after"}}` in association form and
empty host prints; resumed execution is rejected as `OutputPrefix`.

**TestReport has the same output defect plus an independent ownership panic.**
Its input evaluation starts message/print/handled-error checkpoints at
`R/debug_tools/test_report/evaluation.rs:197-210`, retains them in
`EvalExprWithStreamCaptureSuspensionFrame` at `:450-498`, and on success splits
the buffered data into `EvalCapturedOutcome` at `:223-250`. A single printing
case is sufficient for the scratch `OutputPrefix` failure. Expected-output
evaluation and application of `SameTest` are outside this input stream capture
(`evaluation.rs:104-123,149-162`); their output is public unless an enclosing
capture owns it. Hiding output for the entire TestReport would be wrong.

The source name of the result state is `TestReportResultAccumulator`
(`R/debug_tools/test_report/report.rs:32-44`). Its frozen prefix is SharedArgs;
new results form an `AccumulatorChain` suffix. `appended` clones the previous
link at `:100-109`; `into_vec` insists every link is uniquely owned at `:115-116`.
The resume frames clone their case/report state
(`R/debug_tools/test_report/resume_frames.rs:17,41,64,87`). Completion appends
at `report.rs:462` and materializes at `:502-507` inside resume.

This sharing is required by the suspension driver. The active frame remains
Session-rooted while its `resume` runs at `R/suspension/core.rs:777`; the driver
pops and drops it only at `:778-786`. `PureFrameFinalizer::finish` runs later,
at `:800`. After the first resumed case adds result A, the next suspended case
holds A. Final resume appends B -> A, unwraps B, then reaches A while the current
frame still owns it. The panic is legitimate sharing meeting an invalid
uniqueness assumption, not a leaked frame. A first pause with an empty suffix
can conceal it. The same lifetime explains the two new witnesses below.

There is also a pre-existing TestReport policy wrinkle: successful input
capture drains output, while the handled-error branch copies slices and leaves
them in the raw buffers (`evaluation.rs:252-273`). This is a static finding,
not an additional executed failure witness. The proposed repair preserves that
eager completion policy and explicitly tests it; changing failed cases to
exclusive capture needs a director ruling.

**3. The captured-output contract.**

For buffered request responses, output within an unsettled capture is
**provisional**. At a pause it stays in the kernel buffer, available to that
capture and nested observers, but is absent from the host response. Successful
completion applies the builtin's normal consume/filter/release policy exactly
once. Abandonment applies its rollback policy without reevaluating the body,
Check fallback or confirmation-info expression. Output outside every active
capture is public at the next response, in its original channel order.

Responses remain cumulative for the original request. A later response may
repeat a public prefix as representation; adapters deliver only the new suffix.
No print or message may be newly exposed twice, disappear from an already
exposed prefix, or be appended again merely because a pause returned it. This
matches the current print-prefix invariant in `O/support/drive.rs:10-11,35-42`.
The embedding guide's pause/resume explanation at
`docs/guide/embedding.md:285-315` should state capture ownership and cumulative
suffix consumption explicitly.

For example, with an outside `Print["outer-before"]`, an EvaluationData body
that prints `cap-before`, pauses, prints `cap-between`, pauses again, then
prints `cap-after`, followed by outside `Print["outer-after"]`:

| Observation | Public buffered prints | Captured OutputLog |
|---|---|---|
| First pause | `{"outer-before"}` | Not completed; raw capture retained |
| Second pause | `{"outer-before"}` | Not completed; raw capture retained |
| Completion | `{"outer-before","outer-after"}` | `{"cap-before","cap-between","cap-after"}` |

This is the intended repair contract and a required follow-up test, not a
claim that the current implementation passes it.

**One mechanism and its location.** Add a small declarative output-boundary
value (optional absolute message/print offsets) to the hidden suspension-frame
protocol. Its default reports no ownership. Actual capture frames report their
existing checkpoints; the query never evaluates a selector or guest expression.
A cohesive `R/suspension/output.rs` helper, invoked by a distinct pause-response
builder in `R/session/session_requests.rs`, walks the assembled pending frames
and clips each response channel at the earliest active boundary. Keep raw
buffers and their indices intact; keep the original request checkpoint.
Initial and repeated pauses use the same helper. Nested capture completion
cannot expose a suffix still owned by an outer capture.

Use the same declaration for output rollback in the abandonment driver
(`R/suspension/core.rs:713-717`), respecting inner-to-outer frame order.
Existing restoration helpers retain tracing, handled-error and dynamic-state
cleanup, while output rollback happens once through the common rule. Current
helpers already discard EvaluationData, Check/Quiet and test-input suffixes
(`core.rs:393-459`). ConfirmQuiet currently derives PureFrameAbandon
(`R/kernel_data/control_data/confirmation/builtins/resume_frames.rs:91-120`);
its declared provisional messages must also be discarded on cancellation.

A pause must preserve capture bookkeeping. In particular, the existing generic
failure builder truncates handled errors at `session_requests.rs:500-504`;
the new pause builder must not perform that terminal cleanup. Keep `$MessageList`,
trace restoration and memory ownership covered by tests, including nested
cleanup/cancel. Public views and internal capture buffers have distinct roles.
No per-emission counter, ID, allocation, mutex or new diagnostic branch is
needed in `push_print` or message emission. Frame inspection happens at a pause
or abandonment. Any ordinary-path change still requires measurement.

The builtin reach is as follows:

| Family | Provisional channels and completion | Why the present oracle can pass / required stronger coverage |
|---|---|---|
| EvaluationData | Messages and prints; consume into the capture result, then project. | The four G11 roles are still owed; add selectors and output before/between/after two pauses. |
| TestReport input | Messages and prints; success consumes into case data; preserve the existing error copy/release branch unless directed otherwise. | Input role is owed; require both two pauses in one case and pauses in two cases, inspecting complete OutputLog/ActualMessages. |
| TestReport expectedOutput / SameTest | No test-input capture boundary in these phases. Enclosing captures still apply. | These roles are owed; two cases expose the result-chain panic. Test their public output separately. |
| TestReport default / explicit options | No input-capture ownership during option preparation. | Existing descriptors in `O/fixtures/debug_roles/reports.rs:4-39` pause during options; they do not exercise a shared case-result tail or captured prefix. |
| Check | Messages provisional while primary is active; completion observes and releases them. Prints pass through. | Existing body fixture emits its message after the pause-bearing sequence (`O/fixtures/debug_roles/symbols.rs:8-11`). Add pre-pause messages, fallback suspension and cancellation. |
| Quiet | Conservatively defer the entire message suffix until filtering finishes, including currently unselected messages. Prints pass through. | Its fixture also emits the message after all pauses (`symbols.rs:12-14`), and intermediate messages are unchecked. The new scratch witness proves leakage. |
| ConfirmQuiet | Message boundary during its tested body; filtering occurs before optional info evaluation. Prints pass through. | Existing message follows the sequence (`O/fixtures/control_roles/bodies.rs:72-80`). Add two pauses, selected/unselected messages, info suspension and abandonment. |
| Reap | Owns sown values, not buffered prints/messages. | Scope/bucket results finalize after frame retirement (`R/debug_tools/reap.rs:267-287,482-505`). The two-bucket scratch control passes; add two pauses in one combiner and visible-output controls. |
| Timing / AbsoluteTiming / RepeatedTiming | Own timing/sample/result state, not buffered output (`R/debug_tools/timing.rs:51-125`). | Declared duration-leaf normalization permits equality. Keep prints/messages exact and strengthen repeated pauses within one measured body and across samples. |
| Monitor / time and memory constraints | No print/message consumption merely because evaluation is wrapped. | They need passthrough-output and lifecycle controls, not an invented capture boundary. |
| Trace / TraceDetail / TracePrint | Trace state is separate. TracePrint intentionally emits rendered trace as print output. | Preserve exact semantic trace values apart from permitted durations and tracing restoration. The current oracle does not observe raw host trace arrays; see the director question below. |

Check deliberately defers messages even though it normally releases them:
its abandonment currently rolls them back. This conservative provisional rule
preserves nested Check/Quiet observation and stable message order without
filtering at emission or deciding which later cleanup will occur.

The Quiet scratch case is:

```rum
Quiet[(Message[Global`probe::tag];InputString["quiet"];Message[Global`probe::tag];7)]
```

With the oracle's `probe::tag` setup and answer `a`,
`descriptors::accept(..., Checks::ALL)` currently passes. Direct observation
through the same support records one message at the pause and none at
completion. The additional assertion fails with
`Quiet-owned messages must not surface at a pause`. `O/support/drive.rs:35-42`
checks print prefixes only; `O/support/observe.rs:60-75` compares final
messages. Strengthen the driver with message-prefix checks **and explicit
per-pause visible-output expectations**. Prefix checks alone cannot detect all
premature exposure when a scope eventually releases its messages.

Two scope boundaries are explicit. First, `PrintSink` is currently exclusive
streaming: `R/session/session_helpers.rs:383-389` sends to the sink instead of
buffering, as documented in `docs/guide/embedding.md:337-348`. Eager captures
therefore do not put streamed lines in buffered OutputLog. Preserve that route
in this repair and compare identical queued/resumed sink transcripts; deciding
to capture sink events would be a separate contract change. Second, this work
guarantees per-channel order. TODO Q1 (`TODO.md:161-167`) owns one combined
ordered print/message sequence through kernel and host. No event sequence,
response/serde schema or adapter rewrite is part of G11.

**4. The uniqueness class.**

Class **a** means unique at the actual production extractor under the current
construction/finalizer protocol, with the reason shown; it is not a claim that
arbitrary future uses of these private wrappers would be safe. **a*** marks
unreachable extractor bodies rather than a proven live unique-tail protocol.
Class **b** explicitly handles a shared value. Class **c** has reachable frame
sharing and a recorded oracle failure. The census excludes test-only source.

| # | Production site | Class | Reachability / ownership evidence |
|---|---|---|---|
| 1 | `R/accounting_mode/options/buffer.rs:174` | **a** | OptionBuffer::Growable is private, non-Clone working storage. Its only shared ActiveValues view is popped by OptionBridge::drop (R/accounting_mode/options.rs:77-95) when guest() returns (:173-192), before freeze() calls into_vec (:216). A suspension stores the resulting owned values, not this Arc. |
| 2 | `R/call_preparation.rs:81` | **a** | The strict PreparedArgumentAccumulator path is consumed by finish_suspended_prepared_call (R/eval_call_prepare.rs:122), only through FinishPreparedCallArgumentsFinalizer (R/eval_call_prepare/resume_frames.rs:64-82). The originating frame has retired; repeated pauses replace that owner. LHS preparation uses the separate safe method in row 3. |
| 3 | `R/call_preparation.rs:98` | **b** | Err(current) clones value and previous link (R/call_preparation.rs:103-106), preserving the immutable snapshot and ordered suffix; this is used by definition LHS preparation (:604 and :683 in definitions/definition_lhs_preparation.rs). |
| 4 | `R/debug_tools/reap_state.rs:46` | **a** | Scope and bucket collectors return the accumulator, not Vec. Both consumers are post-retirement finalizers (R/debug_tools/reap.rs:267-287,482-505); nested scope/bucket accumulators are distinct. The two-bucket scratch control passes. |
| 5 | `R/debug_tools/test_report/report.rs:115` | **c** | TestReport resumes a cloned case while the frame retains the existing result tail; report.rs:462,502-507 consumes that tail during resume, before frame retirement. Witness C1 below. |
| 6 | `R/definitions/definition_catalog/arguments.rs:46` | **a** | Both resolution frames return FinishArgumentResolutionFinalizer; only its finish consumes the accumulator (R/definitions/definition_catalog/arguments.rs:188-232). No caller frame owns that accumulator. |
| 7 | `R/definitions/value_table_assignment.rs:72` | **b** | Shared node branch clones rule and previous link (R/definitions/value_table_assignment.rs:72-81); unique node moves both. Completion does not assume the active assignment frame has gone. |
| 8 | `R/eval_call_prepare.rs:75` | **a** | Same post-retirement FinishPreparedCallArgumentsFinalizer as row 2; R/eval_call_prepare.rs:122-128 materializes restores there. The scratch control pauses with both a prepared suffix and an Unevaluated restore suffix and passes. |
| 9 | `R/kernel_data/association_data.rs:660` | **c** | The second selector frame clones state (R/kernel_data/association_data/resume_frames.rs:73-78); association_data.rs:847-855 appends and flattens while that frame still owns the earlier selector. Witness C2. |
| 10 | `R/kernel_data/control_data/branching_loops/iteration/driver/continuation.rs:10` | **a** | Saved iterator state is frozen from an owned BlockSymbolState and serialized only by PrepareLoopIteratorScopeFinalizer (R/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:130-165). The following continuation decodes owned state; it does not retain this Arc. |
| 11 | `R/kernel_data/control_data/branching_loops/iteration/mod.rs:98` | **b** | Existing Table repair: Err(current) shallow-clones the node (:98-103). Its previous link remains AccumulatorChain. Table with two input suspensions passes the scratch oracle control. |
| 12 | `R/kernel_data/control_data/cleanup.rs:233` | **a** | A fresh primary-outcome Arc belongs to ResumeWithCleanupAfterPrimarySuspensionFrame; it is extracted only in FinishWithCleanupFinalizer after retirement (R/kernel_data/control_data/cleanup.rs:207-235). Earlier cleanup frames do not own that Arc. |
| 13 | `R/kernel_data/rewrite_data/suspended_state.rs:36` | **a** | The production consuming use is the owned option-scope conversion in R/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:4-9, called by its initial/guard-filter finalizers (:60-70,:270-280). Cases shares its FrozenRewriteValue only for borrowed access and does not unwrap it. |
| 14 | `R/kernel_data/rewrite_data/suspended_state.rs:80` | **a** | PersistentRewriteQueue is frozen from owned remaining bindings and consumed by the guard-filter/rule-result finalizers (same matching/resume_frames.rs:270-280,366-375); pop_front is otherwise its own into_vec loop. The old frame is gone and no remaining caller aliases its queue. |
| 15 | `R/kernel_data/rewrite_data/suspended_state.rs:154` | **c** | PersistentRewriteExprs is safe at Replace/ReplaceList finalizers, and Cases refreezes an empty tail before installing each frame (R/kernel_data/rewrite_data/cases/collect.rs:317-324). DeleteCases instead carries a nonempty kept-output tail across the next pause and flattens inside resume (cases/delete/traversal.rs:526,559,606,667,703,741). Witness C3. |
| 16 | `R/kernel_data/structure_data/replace/suspended_state.rs:68` | **a** | ReplacePart returns a SuspendedReplacePartTargetsCompletion; only FinishSuspendedReplacePartTargetsFinalizer invokes finish (R/kernel_data/structure_data/replace/replace_part/resume_frames.rs:183-200). Rule and candidate collectors own separate accumulators, so the retired frame was the suffix alias. |
| 17 | `R/kernel_data/structure_data/replace/suspended_state.rs:71` | **a** | Each target chunk is a fresh Arc::new(values) (same suspended_state.rs:52-55). Only its node owns that Arc; acquiring the unique node in row 16 also leaves a unique chunk. No payload-Arc clone operation exists. |
| 18 | `R/kernel_data/structure_data/replace/suspended_state.rs:78` | **a** | Prefix is freshly frozen at same suspended_state.rs:35-43 and is shared only by snapshots of that one collector; the post-retirement finalizer in row 16 is its last owner. |
| 19 | `R/pattern_matching/suspended_state.rs:25` | **a** | Owned matcher/guard/interpreted snapshots extract bindings only in post-retirement finalizers (R/pattern_matching/core/resume_frames.rs:73-80,825-827; definitions/rhs_guards/resume_frames.rs:103-124; pattern_matching_interpreted.rs:591-601,673-681,945-952). True/false maps are separately frozen. Guarded ReplacePart can retain an additional FrozenMatchBindings alias, but passes it to the read-only boolean guard path, never into_owned (structure_data/replace/guarded.rs:477-480 and definitions/rhs_guards/resume_frames.rs:61-80). |
| 20 | `R/pattern_matching/suspended_state.rs:41` | **a** | Each FrozenMatcherSequence wraps a fresh owned Vec; materializing callers are finalizers, including ordered arguments, alternatives and flat/orderless/optional state serialization (R/pattern_matching/sequence/ordered.rs:168-175; core/resume_frames.rs:121-129; sequence/resume_frames.rs:116-124). Outer frames have independent frozen sequences. |
| 21 | `R/pattern_matching/suspended_state.rs:62` | **a** | Fresh owned recursive matcher values are extracted while serializing post-retirement finalizers (R/pattern_matching/core/resume_frames.rs:332-340; sequence/optional_sequence/pattern.rs:166-177,262-271; sequence/resume_frames.rs:116-124). No shared read-only caller invokes this extractor. |
| 22 | `R/pattern_matching/suspended_state.rs:101` | **a*** | Dormant, not a demonstrated unique tail: PersistentMatchResults::freeze always installs an empty tail (same file:88-94). The node type/fields are private and production has no node constructor or append operation; the loop cannot execute. Do not mistake this for proof that future shared tails would be safe. |
| 23 | `R/pattern_matching/suspended_state.rs:104` | **a*** | Same unreachable node loop as row 22; no production chunk allocation exists. A future append implementation must not inherit this uniqueness assertion. |
| 24 | `R/pattern_matching/suspended_state.rs:112` | **a** | The actually reachable PersistentMatchResults prefix is a fresh Arc<Vec<SymbolExprMap>>. Its callers serialize only after retirement (R/pattern_matching/sequence/orderless/collect.rs:340-355; optional_sequence/collect.rs:348; optional_sequence/optional/resume_frames.rs:171; rewrite_data/replace/list/matching/resume_frames.rs:270-280). |
| 25 | `R/scoping/lexical/block/continuation.rs:476` | **a** | Single Block saved-state Arc is extracted by either the initializer or body post-retirement finalizer (R/scoping/lexical/block/resume_frames.rs:15-44,149-177); continuation frames hold decoded owned state, not another alias. |
| 26 | `R/scoping/lexical/block/continuation.rs:519` | **a** | Multi-symbol Block body uses PrepareBlockBodyFinalizer (R/scoping/lexical/block/resume_frames.rs:215-242) before the next restoration frame consumes its owned payload. |
| 27 | `R/scoping/lexical/block/continuation.rs:556` | **a** | Initializer items are freshly frozen; PrepareBlockInitializerFinalizer serializes them only after the originating initializer frame drops (R/scoping/lexical/block/resume_frames.rs:76-110). |
| 28 | `R/scoping/lexical/block/continuation.rs:558` | **a** | Same initializer handoff as row 27 for its independently allocated saved-state map. The completion object is the sole remaining holder at extraction. |
| 29 | `R/scoping/packages/resume_frames.rs:65` | **a** | ResumePackageInstallSuspensionFrame returns PreparePackageInstallFinalizer (:30-67); ContinuePackageInstallFrame is a unit frame (:71-72). Plan serialization precedes the next guest form; no other frame holds that plan Arc. |
| 30 | `R/tree_walkers/position.rs:62` | **b** | Shared branch copies the SharedArgs chunk values and clones the previous wrapper link (R/tree_walkers/position.rs:62-70); no loss or uniqueness assumption when frames retain a suffix. |
| 31 | `S/structure_ops/resumable_merge_sort.rs:359` | **a** | Sort wraps owned items once and keeps one comparison-state owner. FrameAction::Evaluate replaces the previous frame, while Finish selects a PureFrameFinalizer (S/structure_ops/resumable_merge_sort.rs:321-375). The driver drops the comparison frame before extraction, even after repeated comparator inputs. |
| 32 | `S/structure_ops/resumable_merge_sort.rs:361` | **a** | The completion object is moved into its own fresh Arc and follows the same single-comparison-frame transfer as row 31. No callback Arc is exposed to a guest or outer frame; the finalizer is its sole holder. |

**Class-(c) oracle witnesses and reachability.** All three tests run their
queued control first through `descriptors::run(..., true, Checks::ALL)`, assert
the expected value and record it; only then do they drive explicit resume.
The native Rust test harness catches the kernel panic. Each resumed panic path
ran once, with no process-abort test. Full sources and evidence are retained
in `probe.rs`, `probe-test.log` and the named `.txt` files. C2 uses this setup:

```rum
Global`assoc=<|"a"-><|"b"->0|>|>
```

| ID | Guest source and input | Queued control | Resumed failure |
|---|---|---|---|
| C1 | `TestReport[{TestCreate[InputString["report:1"],"a"],TestCreate[InputString["report:2"],"b"]},"TestsSucceededCount"]`; answers `a`, `b` | `2`, empty public output | `R/debug_tools/test_report/report.rs:116:37`: `completed TestReport result state must be unique` |
| C2 | After the setup above, ``Global`assoc[InputString["assoc:1"],InputString["assoc:2"]]=9``; answers `a`, `b` | `9`; post-state `assoc["a","b"]` is `9` | `R/kernel_data/association_data.rs:661:37`: `completed association selector state must be unique` |
| C3 | `DeleteCases[{1,2},_?Function[x,(InputString["delete"];False)]]`; same prompt answered `a`, then `b` | `List[1, 2]` | `R/kernel_data/rewrite_data/suspended_state.rs:155:37`: `completed rewrite output must be uniquely owned` |

C1's lifetime is detailed in section 2. The brief-592 expectedOutput and
SameTest files record the same two-case result-tail defect; they were read,
not rerun. Draft A must cover each producer, not just input cases.

C2 stores a persistent selector suffix at
`R/kernel_data/association_data.rs:612-671`. After one selector resumes, the
next selector frame retains that suffix; its resume clones state in
`association_data/resume_frames.rs:73-78`. Final resume appends at
`association_data.rs:847`, then calls `into_vec` at `:855` while the frame is
still rooted. Attach the repair regression as a supplemental descriptor of
existing `Set.target` (`O/inventory/effects.tsv:336`); do not invent a new
catalog effect identity.

C3 keeps the first rejected sibling, then suspends while testing the next
sibling. `PersistentRewriteExprs` stores kept chunks in a shared suffix
(`R/kernel_data/rewrite_data/suspended_state.rs:103-167`). DeleteCases clones
state at `cases/delete/resume_frames.rs:443-465`; its traversal retains/appends
kept output at `cases/delete/traversal.rs:514-521`, completes through `into_vec`
at `:526,559`, and carries the suffix into another suspension at `:606`.
Association-child paths have the same pattern at `:667,703,741`. By contrast,
Replace/ReplaceList complete through finalizers, and Cases refreezes into a
prefix with an empty suffix before each frame (`cases/collect.rs:317-324`).
That explains the shared raw site without declaring every consumer broken.

The drained DeleteCases descriptors split the necessary conditions across
variants: `O/fixtures/rewrite_roles/patterns.rs:66-89` makes Decline return
False, while `rewrite_roles/support.rs:20-25,34-42,75-89` gives repeat pauses to
Twice/AlreadyResumed/Nested, not Decline. Repeated matching deletes the siblings,
so no retained suffix reaches the assertion. Repeated rejection keeps a suffix
and panics. Add their cross product, plus mixed keep/delete and association
children, to the existing patternTest/guard roles. Neither a passing Twice
variant nor a passing Decline variant proves their combination.

**5. Repair design, sequence and drafted briefs.**

**First, shared completion as a structural operation.** Extend the private
accumulator machinery with a completion cursor for `AccumulatorChain<N>` where
`N: Clone`. The cursor owns its remaining spine through AccumulatorChain. Each
step obtains one node: move it if `Arc::try_unwrap` succeeds; otherwise
shallow-clone it. Detach that node's previous link into the cursor before
returning the payload-bearing node. Clients receive an acquired node with an
empty link, never a raw Arc that they must assert unique. Unique acquisition
stays a move; shared acquisition preserves the retained snapshot.

The three defective node payloads are Expr or SharedArgs with a wrapper-owned
previous link, so Clone need only share those payloads and clone the link.
Migrate all three class-(c) loops plus Table's existing safe move-or-clone loop
to the same mechanism. Append stays O(1); finalization stays O(n). Do not flatten
or clone a whole prefix at every pause. The scope does not force Clone onto
generic matcher state or arbitrary host completion callback objects whose
current finalizer protocol establishes uniqueness.

Preserve `R/accumulator_chain.rs:74-98`: cold iterative destruction detaches
one link at a time using `Arc::into_inner`, stopping at a shared node. Keep
production nodes free of manual Drop, every owning previous link wrapped, and
new cursors wrapper-owned even when dropped early. The structural participation
gate in `S/tests/ownership_graph/accumulator_coverage.rs:7-43` checks every
self-edge rather than a type-name allowlist. Existing destruction-depth
certificates, retained snapshots, shared-tail release and an early-dropped
cursor must continue to pass. This proves bounded link destruction, not
arbitrary Expr-tree destruction.

Unrooting the active frame to manufacture uniqueness would violate the driver
invariant. Moving each panic independently into a new finalizer would also
miss the reusable class, and association completion needs Session work that a
PureFrameFinalizer cannot perform. The completion API handles the sharing the
protocol intentionally permits.

**Then, capture completion and response visibility.** Add the prepared selector
to EvaluationData's frame and share its eager/resumed final operation. Add the
common output-boundary mechanism and pause builder described in section 3;
retain raw buffers, capture bookkeeping and ordinary emission paths. Extend
abandonment using the same declaration, preserving existing cleanup order.
Strengthen the oracle before attaching the four G11 roles.

The complete drafts, in `docs/dev/codex/TEMPLATE.md` shape, are:

| Order | Draft | Concrete gate and ledger result |
|---|---|---|
| A | [Shared accumulator completion](../../../.temp/brief-593/draft-A-shared-accumulator-completion.md) | Red-to-green rum-vs-rum witnesses for all three panics; TestReport input/expectedOutput/SameTest producers; association Set state; retained/mixed/association DeleteCases; host non-poisoning and iterative-drain tests. All four G11 rows stay owed. |
| B | [Capture output and descriptors](../../../.temp/brief-593/draft-B-capture-output-and-descriptors.md) | Selector and output reds become exact equality; new per-pause message/print expectations and falsifiers; capture and host lifecycle coverage. Attach exactly EvaluationData.body, TestReport.input, TestReport.expectedOutput and TestReport.sameTest only after full acceptance. |

These are raw drafts for the director to settle and rewrite into numbered
implementation briefs, as TEMPLATE requires. They are not production work
performed under brief-593. A and B explicitly scope files, separate coherent
helper modules, spell out pitfalls, checkpoint counters and two-attempt
stop-loss, and retain failed gate evidence.

The four existing G11 effects/owed entries are unchanged. Admission in B needs
Middle, Twice, Error, AlreadyResumed, Nested and each declared Decline variant,
Checks::ALL, both feature configurations and lifecycle checks at the first and
second relevant pause. TestReport must cover two pauses in one input capture
and suspension in two distinct cases, then repeat the two-case requirement
for expectedOutput and SameTest. Observe complete case output and state, not
only a selected success count. EvaluationData needs absent/All/Properties,
scalar/list/missing/invalid selectors and a selector that pauses during
preparation; its effects and body must run once.

Stronger accepted descriptors cover Check/Quiet/ConfirmQuiet messages before,
between and after two pauses, selected/unselected messages, passthrough prints,
fallback/info paths, nesting and cancellation. Preserve Reap and timing
passthrough behavior. Duration normalization is restricted to explicitly
selected nonnegative duration leaves (`O/support/normalize.rs:5-23`); it must
not erase message, OutputLog, selected-value, result-shape or semantic Trace
property differences. Test overlap rejection, cancel, reset, queue exhaustion,
fresh requests, memory accounting and restored dynamic state at both pauses.

Both implementation drafts carry the standing maintenance duty: regenerate
local-contract fingerprints and default/no-default-feature
local_production_acceptance certificates, reconcile discovery inventories,
disclose every changed row and keep unknowns unknown. Every new frame or
FrameRootTrace/PureFrameAbandon type, even in stdlib, enters runtime's
`tests/frame_root_trace/derived_inventory.rs` and named-type count. The drafts
retain the stop rules for unexplained removed/changed existing rows, new
UnknownHostPayload and new discoveries in untouched code. The specific planned
row removals below require the director to settle that scope before dispatch.

Both drafts require serial bounded fmt/runtime/workspace/oracle checks, plus
rum-vs-rum `scripts/core-language-microbench.sh` BASE/NEW timings on the full
panel and per-label review of hot/same-tier neighbors. No CPU-heavy overlap.
For instruction gates, both builds use
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, matching source-path exports supplied
by the director and separate target directories. Freeze exact workload
statements; do not add a Set, sink or Print to consume a result. Run
`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, stdin `/dev/null`, at N
and 2N; use `(count(2N)-count(N))/N`. Two independent BASE slopes establish each
label's absolute/relative BASE-vs-BASE noise floor before one NEW slope.
Ordinary-path/same-tier regressions beyond that floor reject the change;
unavailable counters are an unmet gate. No assertion of zero ordinary cost
substitutes for this evidence.

Draft A names the existing DeleteCases, association Set and Table hot labels,
plus exact two-case TestReport/two-key Set instruction workloads. Draft B adds
ordinary arithmetic, no-output wrappers, buffered emission and captured output
workloads. Probe caps are one red execution per panic witness, at most two
wall A/B pairs and six instruction-count processes per workload (two BASE
slopes and one NEW slope). All generated guest files use `.rum`.

**6. Open questions for the director.**

- **Failed TestReport input:** retain the current handled-error copy/release
  policy, as Draft B proposes, or intentionally change failed cases to consume
  their captured output? The eager source evidence is
  `R/debug_tools/test_report/evaluation.rs:252-273`. A change needs an explicit
  documented contract and a queued/resumed error witness; the successful-case
  prefix fix must not silently decide it.
- **Exclusive PrintSink:** the draft preserves the documented route that sends
  every Print to the sink and leaves buffered output empty. If captures should
  intercept streamed lines too, issue that broader contract change explicitly;
  it affects eager embedding behavior as well as suspension.
- **Raw host trace arrays:** EvaluationData temporarily enables tracing and
  may truncate it at completion, while the generic pause response copies trace
  events (`evaluation_data.rs:76-82`, `session_requests.rs:495`). The oracle
  Observation currently omits raw trace arrays. Should a separate trace boundary
  join this repair, or stay a separately diagnosed contract? The draft requires
  trace-state restoration and exact semantic Trace properties but makes no
  raw-trace prefix guarantee.
- **Known ownership-inventory transitions before Draft A dispatch:** removing
  the three invalid uniqueness panic closures necessarily removes existing
  UnknownConstruct records: `S/tests/ownership_graph/inventory/unknown_macros.tsv`
  rows 154, 426, 781 and `unknown_captures.tsv` rows 237, 617, 900. Their source
  sites are TestReport `report.rs:116`, association `association_data.rs:661`,
  rewrite `suspended_state.rs:155`. If Clone is added as a derive, the three
  existing node-derive observations at `unknown_attributes.tsv:94,256,480`
  also change. TEMPLATE's standing duty stops on removed or reidentified rows;
  the issued brief must authorize the precise generated transitions, while
  retaining the stop rule for unexpected drift. No inventories were edited here.

The proposed conservative buffering of Check/selected Quiet messages is a
contract clarification for the director to adopt with Draft B. It preserves
existing raw-buffer rollback and nested observation, at the cost of deferring
messages that normal completion will eventually release. The implementation
brief should state that choice explicitly in the guide and host tests.
