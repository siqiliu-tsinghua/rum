# brief-604 — review 2: is the G13 typed composition design ready to implement?

`Scope: NONE — review only.` **Do not run `git` in any form**, including
before you have read this brief. You write no code and run no probes: this is
a reading review of a design another worker produced.

## Background

G13 concerns rum's suspension frames. When an evaluation suspends for input,
frames are queued and later resumed; each frame hands its completed value to
the next. Typed continuation state (an `Option<Expr>`, a match outcome, …) is
currently encoded in-band as `Expr` under ``System`InternalContinuation*``
heads and decoded by the consumer; the heads are ordinary symbols that guest
source can also write, the decoders are lenient or panic on unexpected shapes,
and nothing checks that a frame completes the type its consumer decodes. Six
frames complete a guest value raw into an `Option<Expr>` slot (21 executed
divergences between the resumed path and the queued-input path).

You reviewed the second design round in brief-602 and rejected it as ready for
implementation (nine findings F1-F9, seven required changes). Your review was
adopted in full. The third round answers it:

- `docs/dev/codex/brief-603-g13-typed-composition-design.md` — the director's
  brief (top: rulings and what it had to settle) and GLM's report (§1-§7):
  a closed `ContinuationState` enum carried out of band, a sealed `SlotValue`,
  `TypedFrame` with `Input`/`Output`, a zero-sized witness binding `Input`, the
  return position binding `Output`, checked constructors for `Evaluate` and
  finalizers, a `FrameChain` builder for chained inserts, a `ContinueToken` for
  helper re-suspension, a named uncatchable mismatch variant, and an eleven-step
  plan (604-614 in its numbering; implementation numbering will shift).
- Its tables are under `.temp/brief-603/`; earlier rounds' under
  `.temp/brief-600/` and `.temp/brief-601/`; your own review is
  `docs/dev/codex/brief-602-g13-design-review.md`.

## What to review

Answer each with `path:line` evidence from the source or the tables, and say
whether it is verified in source or inferred.

1. **Your seven required changes.** For each, is it met? Where not, what is
   missing.
2. **`ContinueToken` and state.** The token stores `Arc<F>` and its `suspend`
   takes no frame value. The prepared-call frames carry the loop's progress
   (`state: SuspendedCallArgumentPreparation`,
   `crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:4-8`,
   `:34-38`). Does re-suspension through the token resume from the right
   progress? If not, what form should the token take so the frame type (and so
   its `Output`) stays bound while the state is new?
3. **Visibility and sealing in Rust.** The erased layer becomes `pub(crate)`,
   `ContinuationState` wraps module-private runtime types, and `SlotValue` is
   public with methods mentioning `FrameOutcome`. Does the proposed surface
   compile under Rust's private-in-public rules and still let `rum-stdlib`
   frames name their `Input`/`Output` and implement `TypedFrame`? If not,
   what arrangement does (for example conversions on the private sealed
   supertrait)?
4. **The remaining declared bindings.** Three side-effect inserts keep a
   declared `Output` (`insert_typed_frame`), one of them
   `DirectDefaultFallbackFrame` with a non-trivial output. Can each be bound
   by the compiler instead?
5. **Transformed results.** 29 macro sites transform `$result` before the
   macro; the witness then binds the transformed type. Where the transform
   keeps the type (`Expr → Expr`), nothing fails to compile but the resumed
   path skips the transform. Is the design's handling (audit per family
   brief) adequate?
6. **The closed enum.** Is the crossing population (35 types, all
   runtime-owned or primitive) established? Is the enum's extension policy
   and its ownership/inventory consequence stated honestly?
7. **The step plan.** Can each step compile and land green on its own, in the
   order given, with every crossing type flipped SCC-complete? Are the
   performance gates placed on every step that changes a hot evaluator
   function?
8. Anything else that is wrong, missing or over-scoped.

## Verdict

End with one of: **accept**, **accept with required changes** (list them
precisely enough to fold into implementation briefs), or **reject** (say why
and what instead). Rank findings by severity.

## Scope

- Files you MAY modify: **NONE**, except: append your review to this file;
  scratch notes under `.temp/brief-604/`.
- No scratch test, no Cargo command, no probe of any kind.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository.
- Commands: file reads, `rg`/`grep`, `python3` over the `.temp` tables
  (through `python3 scripts/run-bounded.py --timeout 300 -- …`).
- **Testing Policy is a hard line** (`AGENTS.md`).

## Report format (append to this file)

1. Findings, ranked by severity, each with evidence and verified/inferred.
2. Answers to questions 1-8.
3. Verdict and the required changes.

---

## 1. Findings, ranked by severity

Reading review only. I read the reports, production sources and existing
tables, and used bounded Python only to query the `.temp` tables. No git,
Cargo, compiler experiment, test, runtime probe or network operation was run.
Only this appended review was written. **Verified** means verified by reading
source/tables; **inferred** means a consequence of the proposed API or control
flow, not an executed result. Brief-603's reported executions are its evidence,
not new executions by this review. Step numbers below are that report's draft
numbers, not implementation assignments.

**F1 — P1 / blocker: `ContinueToken` preserves the frame type by preserving
the wrong frame value; its construction authority is also unenforced.**

Verified: the proposal stores `Arc<F>`, accepts no replacement frame at
`suspend`, and deletes prepared-call Frame B in favor of reinserting Frame A
(`docs/dev/codex/brief-603-g13-typed-composition-design.md:458`, `:496`). Both
current frames contain progress, not just a continuation identity
(`crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:5`, `:35`).
Resume clones that progress; the helper appends the completed argument,
processes subsequent arguments, and advances the cursor before freezing the
next frame (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:384`, `:409`,
`:435`, `:472`). `allow_sequence_splice` is also selected for the new pending
argument at `:424`.

Inferred: reinserting the original Arc loses the new prefix, cursor, restore
entries and pending-argument mode. The next answer is applied at the old
position and intervening work can run again. This is incorrect even when
`Input = Expr` and `Output = PreparedCallArguments` are perfectly checked.
The declared token mechanism therefore does not resolve the prepared-call
counterexample or the analogous loop families in
`.temp/brief-603/deferred-completion-families.tsv:2`.

Required form: a token certifies **F**, not a particular `Arc<F>`. Give it a
private invariant marker for F and a method taking `self`, the
real input witness, **`replacement: F`**, the session and request. Its bound
remains `F: TypedFrame<Input = I>`; its output obligation remains `F::Output`.
The prepared helper supplies `F_A { state: advanced_state,
allow_sequence_splice: new_mode }`. Keeping no old Arc also avoids making the
token an extra owner across finalization. Preserve the current drop-before-
materialization order and check it with the restore-accumulator case
(`eval_call_prepare.rs:71`, `eval_call_prepare/resume_frames.rs:64`).

The statement that only a running frame can mint a token is not implemented by
`pub fn of(&Arc<F>)` (brief-603:462, :582): any caller can construct an Arc of
a frame it owns. Mint the token privately in the typed adapter and supply it
to that frame's typed resume invocation; expose no public `of`/`Default`.
Make it non-Clone and scoped to that invocation so it cannot be retained as
authority to insert into an unrelated later suspension.
The displayed `suspend` also needs an actual receiver to support the proposed
`token.suspend(...)` calls. Test rejection of a different replacement frame
type/output and public token minting, plus repeated suspension with different
answers, intervening effects and `Unevaluated` restores. These are future
implementation gates, not probes run here.

**F2 — P1 / blocker: the composition API still has an unbound first input and
an unrestricted insertion route.**

Verified: `FrameChain<Out>` forgets the first frame's Input, and
`suspend_chain` takes no producer witness
(`docs/dev/codex/brief-603-g13-typed-composition-design.md:448`). `then` checks
only internal adjacencies. A real chain follows a result of
`match_expr_interpreted_owned` at
`crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:87`, `:92`,
`:100`; another follows `Option<CasesLevelSpec>` at
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:169`,
`:183`.

Inferred: a chain with a wrong first Input, mutually compatible intermediate
types and the correct final Output satisfies every shown signature. Preserve
both endpoints in an opaque `FrameChain<I, O>` with invariant markers;
`chain(first)` returns `FrameChain<F::Input, F::Output>`, `then` preserves I,
and `suspend_chain` requires `SlotWitness<I>` from the actual producer. Bind
O at the final return position. The shown `FrameChain<Out>` also needs a real
type-level use of Out; its erased vector alone leaves an unused type
parameter.

Verified: the three side-effect routes explicitly retain declared Output
and runtime-only adjacency checks
(`.temp/brief-603/insertion-routes-v2.tsv:4`), while the draft exports
`insert_typed_frame` (brief-603:758). This is an unrestricted insertion escape
for any `TypedFrame`; enumerating three intended uses does not restrict its
API. The three cases can be bound:

- **Default fallback:** take the witness before matching `direct_default`,
  pass it and the request to the cold helper, and return the helper's
  `Result<F::Output, EvalError>` from the caller's `Result<Option<Expr>, _>`
  position. All required data is available at
  `crates/rum-runtime/src/runtime/definitions/application/down_values.rs:69`,
  `:78`, `:90`. The helper at
  `crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs:41`
  can return that result instead of `()`; no side effect requires discarding
  its type. Its fallback evaluation must retain the same output obligation
  on another suspension (`default_fallback.rs:25`).
- **Retention:** preserve insertion, receipt transfer and admission/abandonment
  order (`crates/rum-runtime/src/runtime/accounting_mode/frozen.rs:111`,
  `:133`, `:140`, `:148`). A typed suspension receipt can remain local through
  those operations and then finish into the owning typed result/action.
  Bind the continuation's Output to that result, and Input to the callback
  result; merely adding an associated Output to `RetentionContinuation` does
  not bind its insertion. For the current `RetainedResult`, the output
  equality is concretely **`Output = Expr`**, because that carrier owns
  `Result<Expr, EvalError>`
  (`crates/rum-runtime/src/runtime/accounting_mode/retained.rs:20`). Consume
  the receipt into that typed result before any conversion to a bare error.
  The callback already has a structured failure carrier that can carry the
  Expr input evidence
  (`crates/rum-runtime/src/runtime/accounting_mode/bridge.rs:19`, `:31`).
- **Public-file rollback:** this is expressly abandonment-only; `resume` is
  unreachable (`crates/rum-runtime/src/runtime/session/batch_file.rs:5`,
  `:14`). Use a narrow structural rollback operation/private abandonment
  entry rather than granting it arbitrary value completion. If represented
  as a value frame, require an identity `I -> I` operation bound to the
  surrounding typed suspension and batch boundary. Do not count an
  unreachable, arbitrarily declared Output as a compiler proof.

These corrections need not impose a premature E2 no-`Try` representation.
They close ordinary typed construction routes. As already acknowledged by
E2, safe Rust alone cannot force a hostile caller to consume a returned
receipt rather than discard it (`.temp/brief-583/e2-outline.md:34`).

**F3 — P1 / blocker: the numbered migration cannot land independently green
as specified.**

Verified: step 604 removes the old kernel exports while claiming no
production flip (brief-603:753), but stdlib directly imports and implements
them today, for example
`crates/rum-stdlib/src/lazy/continuation.rs:13`, `:53`, and
`crates/rum-stdlib/src/structure_ops/surface/lazy_completion.rs:5`, `:16`.
Keeping a legacy macro only "in-crate" cannot keep those implementations
compiling. Furthermore the adapter specified at brief-603:486 completes
`FrameOutcome`, whereas the driver trait/action/finalizer remain Expr-based
until step 605 (`crates/rum-runtime/src/runtime/suspension/core.rs:81`, `:96`,
`:102`; brief-603:771). An unused implementation must still type-check.

Required: keep explicitly named legacy exports and control flow until their
last users migrate. Either delay the live typed adapter until the driver
channel conversion, or specify a separate compatible scaffold/bridge. Do
not make codecs accept either State or Value globally to conceal mixed
protocols, and do not claim enforcement while legacy insertion is available.

Verified cross-group dependency: step 606 converts all runtime
`Option<Expr>` producers/consumers, including downvalues; step 609 converts
`CompiledMatchOutcome` (brief-603:740, :743). But the very same downvalue frame
consumes `CompiledMatchOutcome` and completes `Option<Expr>`
(`crates/rum-runtime/src/runtime/definitions/application/down_values.rs:350`,
`:377`, `:479`). The proposed sealed mapping sends the former exclusively
through State. After 606 that typed consumer cannot decode a still-legacy
matcher completion. Conversely, 609 affects the guards deferred until 611
and interpreted matching deferred until 610
(`crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:19`,
`:122`; `crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:541`,
`:546`). A legacy expression-form macro does not convert either channel.

This is a verified dependency with an inferred intermediate-build failure,
not a claim that every repeated Rust payload type must always migrate in
one enormous step. Disconnected uses may remain legacy together. The
lazy-realize/capture pair can plausibly remain such a pair while dispatch
changes, because its enclosing builtin still completes Expr
(`crates/rum-stdlib/src/lazy/continuation.rs:87`,
`crates/rum-stdlib/src/structure_ops/surface/lazy_completion.rs:31`). But that
must be established by edges; brief-603:834's unsupported "verified
compatible" does not settle every stdlib boundary.

There is also no numbered assignment for the complete G-value/G-vec
populations (`.temp/brief-603/crossing-payloads.tsv:2`, `:4`, `:5`, `:7`).
Step 605 leaves legacy frame implementations, and 614's deletion scope does
not migrate them. For example the ordinary Expr wrapper at
`crates/rum-stdlib/src/arithmetic/stats.rs:233` still needs a typed-frame
migration. Produce a full site/frame-to-step mapping, include every crossing
edge's temporary protocol, and either combine connected work or enumerate
private adapters with introduction/removal steps. A STOP upon discovering a
leftover is useful during execution, but is not the missing green plan.

**F4 — P1 / blocker: the proposed mismatch is neither automatically
uncatchable nor covered by the claimed cleanup path.**

Verified: brief-603:629 argues that a non-Thrown variant cannot be consumed
by guest error handling. `EvaluationData` converts arbitrary errors to data
at `crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:90`, `:127`.
Test-report capture does the same at
`crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:193`,
`:268`, and its resumed frame calls that handler at `:483`. Thus a named
`ContinuationSlotMismatch` passed as an ordinary Err to subsequent frames
can disappear from the final request error. A terminal-only absence check
does not detect that occurrence.

Verified: the current driver resumes each remaining frame with the outcome,
then pops/drops it; an ordinary Err at the end calls
`finish_completed_resumption`, not abandonment
(`crates/rum-runtime/src/runtime/suspension/core.rs:736`, `:747`, `:627`).
The normal abandonment entry takes **pending** suspension at `:638`; it is
not already a fatal-resumption unwind for the active queue. A consumer whose
decode failed also still owns cleanup obligations. Simply returning an
adapter error and dropping it can skip its restoration; capture restoration
and rollback are explicit abandonment operations (`core.rs:319`, `:671`).

Required: specify an actual fatal driver route. Detect the mismatch before
discarding the failing consumer, retain/unwind that consumer and the active
suffix through structural abandonment with output rollback, handle any
nested owner, and only then expose the named EvalError at the request
boundary. Keep the fault out of ordinary guest-handler input, or explicitly
audit every catch-all and provide a non-suppressible occurrence check. Include
fault-injection tests under capture/cleanup/rollback scopes, followed by a
fresh request, in both feature builds. No default-path diagnostic counter is
needed; record a mismatch at its cold creation point if a test latch is used.

**F5 — P2 / API prerequisite: visibility and sealing are directionally right
but the displayed Rust surface does not implement the stated privacy.**

Verified: the design declares public `ContinuationState`/`FrameOutcome`,
with conversion methods on public `SlotValue`, while describing private
conversion and private runtime payloads (brief-603:352, :360, :367, :601,
:611). `CompiledMatchOutcome` is runtime-visible and
`PreparedCallArguments` is parent-visible today
(`crates/rum-runtime/src/runtime/pattern_matching/outcome.rs:4`,
`crates/rum-runtime/src/runtime/eval_call_prepare.rs:10`). Making payloads
visible to a central runtime module does not make them public to stdlib.

Rust inference, not a compiler run: public items mentioning less-visible
types are not uniformly hard compiler errors. In particular enum fields and
trait method interfaces can trigger `private_interfaces` diagnostics;
`#[doc(hidden)]` supplies no privacy. Leaking a private associated type through
a publicly reachable implementation is a separate private-in-public problem.
The design must not depend on warnings or on inaccessible paths as its
sealing mechanism. If the channel is public as shown, its public codecs and
variants are not adapter-private. If it is made private without moving those
methods, the public interface still exposes the private channel type.

A coherent arrangement is: public, sealed **marker** `SlotValue`; the encode/
decode methods on its non-exported sealed supertrait; channel types and
erased construction restricted to the suspension implementation; public
`TypedFrame`, typed constructors and the necessary public carrier types.
Runtime-only frames and payload types retain appropriately restricted
visibility. Stdlib can use `Expr`, `Option<Expr>`, `Vec<Expr>`, `bool`, `String`
and publicly exported opaque carriers without naming the enum or implementing
the codec. In particular, the selector migration needs public
`EvaluatedPartSelectors` and a typed producer available to its stdlib caller
(`crates/rum-runtime/src/runtime/kernel_api.rs:32`,
`crates/rum-stdlib/src/associations/mutation/resume_frames.rs:27`).

`pub(crate)` is also broader than "driver-private": other runtime modules
can still implement an erased trait or use a crate-visible raw constructor.
Restrict those operations to the suspension module/adapters or seal the
erased implementation boundary. A public enum's fields cannot individually
be private; brief-603:390's comment is insufficient. A public opaque
`ErasedNext<Out>` constructible only through the checked constructor can
enforce the intended equality, but it needs a definition; an opaque
`TypedAction<Out>` around a private action enum is simpler to specify.

Keep the positive controls: an ordinary stdlib frame, a state-carrying stdlib
frame and a public opaque selector consumer must compile using only the
public API. Negative tests must show that raw construction and external codec
implementation fail for the intended reasons. An external doctest proving
`pub(crate)` is inaccessible does not prove module privacy inside runtime.

**F6 — P2 / inventory prerequisite: 35 table rows are not 35 established
crossing Rust types. Several rows are nested codec fields, and some actual
crossings still have no specified Rust carrier.**

Verified: `.temp/brief-603/crossing-payloads.tsv` has 35 data rows, but row 28
groups three interpreted payloads, row 29 groups twelve choice states, and
rows 30, 32, 34-36 group further populations. Row 2 is Expr, which belongs to
Value, not a State variant. Therefore neither "35 types" nor "one variant
per row" follows (brief-603:348, :356, :818).

More seriously, the old decoder list is not itself a crossing inventory.
Row 35 admits `Vec<RuntimeMessage>` on the strength of D002, but the sole
call decodes one field **inside** `EvalCapturedOutcome`
(`crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:66`,
`:281`). That is not evidence of a `Vec<RuntimeMessage>` frame slot. Similarly,
the symbol-vector helpers in row 34 decode fields of `PackageInstallPlan`
(`crates/rum-runtime/src/runtime/scoping/packages.rs:111`, `:193`).

Conversely, `SequenceFramePayload` in row 36 is only an inventory label:
the cited source helper accepts a tag and returns an Expr iterator
(`crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:471`).
Different tags contain different tuples. At `:119` a complete crossing holds
state **plus** rest-values **plus** match outcome; listing each component as
a separate crossing type does not describe this slot.
`MatchCompiledArgAndRestOwnedSuspensionFrame` additionally completes either
an already finished match or a prepared-rest payload (`:23`, `:35`), and its
consumer currently distinguishes them by shape at `:72`. It needs one
explicit typed sum, such as completed-outcome versus pending-rest, shared by
both producer branches and the consumer, or an appropriate frame split.
No single-output rule can be implemented by assigning those branches unrelated
State variants.

Required: one row per actual whole-slot Rust identity (including newly
designed nominal structs/enums), with producer/finalizer, consumer, all
branches, ownership/visibility, re-suspension behavior and migration step.
Separate payload fields and helper accumulators. Reconcile this against
completion/decode uses, not just enclosing function return types. The current
evidence supports a **feasible runtime-owned closed enum direction**; it does
not establish its exact population. No verified example here requires an
arbitrary downstream-owned slot type.

The extension policy should say explicitly that downstream authors cannot
register new slot types: adding one requires a runtime-owned/neutral public
carrier where needed, a central State variant, sealed conversions and renewed
edge/ownership review. The report usefully acknowledges the separate trust
limit for arbitrary external Expr-to-Expr computations (brief-603:609).

**F7 — P2 / inventory prerequisite: the repaired edge table has a concrete
join bug and still lacks the migration dispositions it is said to supply.**

Verified by bounded table queries: the macro reconciliation contains 598
`yes` entries and two `NO` entries. Its writer deliberately emits lowercase
`yes` (`.temp/brief-603/sites_reconcile.py:93`), but the propagation extractor
accepts only uppercase `YES` or `NO`
(`.temp/brief-603/propagation_edges2.py:85`). As a result 617 of the 619 edge
rows say `caller_frameless=YES`; the only populated macro-site fields are
the two formerly missing nested-token sites. These flags cannot support the
claimed per-edge analysis.

Even after correcting that comparison, "any macro in the caller" is not an
edge disposition (`propagation_edges2.py:113`). The table's header has no
producer/conversion/consumer chain, resuspension, class, migration disposition
or step columns (`.temp/brief-603/propagation-edges-v2.tsv:1`), although the
implementation drafts instruct workers to execute dispositions "from" that
table (brief-603:797). Indirect calls are still omitted from the unresolved
output (`propagation_edges2.py:95`), and same-type transformations are skipped
at `:106`. Owner-qualified identities and the 600-site reconciliation are
real improvements; they are not a completed dataflow inventory.

The promised `chained-inserts.tsv` is absent from `.temp/brief-603/`; the
report at brief-603:314 and `.temp/brief-603/PROGRESS.md:11` provide a count
and examples, not its rows. The non-Result table also conflates the raw lazy
realization helper with the First capture helper
(`.temp/brief-603/non-result-sites.tsv:2`): the former actually returns a raw
EvalError and inserts an Expr-to-Option frame
(`crates/rum-stdlib/src/lazy/continuation.rs:30`, `:87`); the latter returns
`Result<Option<Expr>, _>` but completes the whole First call as Expr
(`crates/rum-stdlib/src/structure_ops/surface/lazy_completion.rs:46`).
Additional synthetic-Err helpers already identified in review 602 remain
without their specific input-evidence disposition; see
`crates/rum-stdlib/src/structure_ops/list_plumbing.rs:47`, `:164`.

The proposed `()` deferral consequently needs a narrower statement. The
machine's ordinary completion really is `()`
(`crates/rum-stdlib/src/lazy/machine.rs:16`), but the outstanding child result
is Expr at `:68`, propagated via `await_callback` and the realization helper
(`crates/rum-stdlib/src/lazy.rs:446`). G13 need not make `()` a slot or
implement E2 globally. It **does** need to carry the original callback's
`SlotWitness<Expr>`/typed suspension evidence to the realization insertion,
and bind that insertion's output to `Option<Expr>`. A witness of
`Result<(), _>` at `drive_machine` would bind the wrong Input. Hoisting only
First/Normal's capture does not solve this lower boundary.

Some witness attributions are also still wrong. Verified source facts:

- The lazy witnesses put InputString in LazyTable **bounds**, with body `i`
  (`.temp/brief-603/probe.rs:46`, `:52`, `:58`). First evaluates its first
  argument (`crates/rum-stdlib/src/catalog/structure.rs:70`), and Normal is
  ordinary (`crates/rum-stdlib/src/catalog/associations_transform.rs:123`).
  Bounds suspension is handled by
  `crates/rum-stdlib/src/lazy/table_continuation.rs:75`; First's capture is
  installed only when later realization suspends
  (`crates/rum-stdlib/src/structure_ops/surface.rs:83`). Inferred: these
  examples exercise bounds/preparation, not the claimed suspended lazy
  capture. Put suspension in a realized body/callback for that obligation.
- `Prime[{2, (InputString; 3)}]` still suspends during argument evaluation
  (`.temp/brief-603/probe.rs:112`; Prime's ordinary/Listable registration is
  `crates/rum-stdlib/src/catalog/number_theory.rs:50`). It does not establish
  suspension within `session.evaluate(threaded)` at
  `crates/rum-stdlib/src/number_theory.rs:289`. Infer the pause location from
  the source evaluation order, not merely the fact that a list occurs.
- The probe compares reduced text observations at `probe.rs:17`, `:147`,
  not the oracle's complete value/effect comparison at
  `crates/rum-stdlib/tests/suspension_oracle/descriptors.rs:191`. Its generic
  rejection predicate can accept matching non-Pause rejections
  (`probe.rs:143`). All input answers are `"a1"` at `:136`, despite the
  comment promising different answers; `hits` is not projected.

The recorded equal rows are useful baseline controls for their actual paths.
They do not certify every claimed family edge or the proposed migration.
Keep the corrected genuine string guard, and require successful full oracle
observations, attributable suspension points, distinct answers and relevant
effect projections for the remaining witnesses.

**F8 — P2: a per-family transformed-result audit is acceptable only as a
mandatory pre-flip obligation with real sites and behavior checks.**

The type argument is correct: a witness of `Result<T, E>` after `map(T -> T)`
cannot prove the suspended continuation executes the map. Merely moving the
map into the ordinary ok-arm is insufficient unless the resumed frame runs
the same remaining computation. This is a semantic obligation even when all
types agree; G13 cannot defer transformations needed by its migration to E2.

Verified inventory qualification: the report's "29" comes from
`a-let-consumer`/`a-map-tail` (brief-603:828). Those classifications describe
uses **of the macro result**, not a list of transformations **before** its
input. For example `builtins_input.rs:65` receives the untransformed result
from `:64` and finishes it after the macro at `:74`
(`.temp/brief-601/macro-use-classification.tsv:210`). The Cases map is after
the macro (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:441`,
`:448`; classification row 375). Thus the supplied tables do not establish
the asserted 29 pre-macro transformations. They must be inventoried by actual
dataflow, including same-type and error transformations and helper-local
transforms.

Do the global enumeration before dispatching family briefs; permit the
actual repairs/tests to land with the owning family before it flips. Each
entry must state raw producer type, transform, resumed implementation,
repeated-suspension behavior, and a witness that would observe its omission
or duplication. Also audit work skipped by the new return-form macro: the
post-macro `ContinueLoopIteratorScopeFrame` insertion at
`crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:383`,
`:396` is load-bearing and is not found by merely counting nearby insert-arm
pairs. This needs composition in the same conversion, not an assertion that
all let-bound sites are return-equivalent.

**F9 — P2: adapter ownership improves, but transit and allocation accounting
are not settled by delegation.**

Verified positives: the Arc-backed adapter avoids requiring `F: Clone`,
delegates `output_boundary`, and finalizers have Send/'static bounds
(brief-603:403, :486). A consumed enum match needs neither `Any` downcast nor
unsafe. Keeping Expr destruction debt is correct
(`crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv:53`).

Verified omission: the queue currently charges an Arc header plus the dynamic
size of the **frame** (`crates/rum-runtime/src/runtime/suspension/core.rs:191`);
admission uses `size_of::<F>()` at `:519`. With an outer
`Arc<TypedFrameAdapter<F>>` containing an inner `Arc<F>`, unchanged code
charges only the outer header and pointer-sized adapter. Root delegation
does not charge the second allocation or F's inline storage
(`crates/rum-runtime/src/runtime/suspension/trace.rs:110`). The independent
occurrence oracle makes the same current frame-layout assumption
(`crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs:24`).
Inferred: adopting the adapter without updating both maintained accounting
and the independent model undercounts the introduced ownership layout;
retention pre-admission also needs the new projection. Specify this work in
the introducing step, not just "replay inventories" later.

Transit is still described inconsistently: brief-603:644 says within one
iteration, then `:648` acknowledges a cross-iteration handoff. The actual
outcome survives producer drop and enters the next resume
(`core.rs:736`, `:745`, `:747`); queued tracing only visits frame storage
(`:184`). State the synchronous transit lifetime and the consumer's rooting/
accounting obligation if it evaluates while owning the decoded input. Plain
Rust ownership settles memory safety, not the logical root/accounting
contract. This need not add a persistent Session field or turn acknowledged
transient debt into a new hot-path walk.

Inventory changes should be reported as actual new, removed or changed edges.
A concrete closed enum does not automatically imply an UnknownHostPayload
row for every Expr-owning field; the new erased/generic storage and public
inputs require the inventory's actual classification. The report's prohibition
on *any* existing identity/status change (brief-603:670) also needs planned
exceptions: privatizing/removing the old public constructors necessarily
changes their PublicInput entries (ledger `:56`, `:57`, `:58`). Preserve the
underlying destruction-debt disposition and disclose intentional migrations;
do not require obsolete rows to remain literally unchanged.

**F10 — P2: the concrete performance and compile gates still fall short of
the stated blanket policy.**

Verified: brief-603:718 promises a full per-label A/B for any hot-function
change, but step 605 specifies only a two-label spot-check (`:778`), and the
607-613 draft labels 608/609/611/612/613 "cold, not gated" (`:801`). This
contradicts both the earlier summary (613 is gated at `:724`) and source:
609 changes ordinary matcher insertion sites
(`crates/rum-runtime/src/runtime/pattern_matching/core.rs:22`, `:33`);
611 changes traversal loops (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:209`,
`:243`); 612 covers block/options; 613 changes ordinary selector evaluation
(`crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:117`).
The benchmark already includes matcher, block, Part and traversal labels
(`scripts/core-language-microbench.sh:492`, `:508`, `:569`, `:610`). Changing
a cold arm inside those functions is still a hot-function change.

Required: attach the full suite's per-label base/candidate comparison,
including same-tier neighbors, to every such step; codegen-unit-controlled
instruction slopes supplement it. A truly unused scaffold may omit this
gate. Do not decide the other steps are cold from the storage family's name.
The revised profiling obligation (identity/count/classification/balanced
depth, excluding elapsed values) is appropriate.

The new compile-fail/pass list and per-position corpus policy are useful
(brief-603:699). Add the missing first-chain-input, token-mint/replacement,
side-effect-output and private-surface cases from F1/F2/F5, with positive
controls that differ only in the intended invalidity. A compile-fail doctest
must not pass merely because the example's API is otherwise inaccessible or
incomplete. Both feature builds' oracle/lifecycle checks and independent
no-mismatch detection remain required after the F4 repair.

## 2. Answers to questions 1-8

1. **The seven required changes from review 602:**

   | Required change | Assessment and evidence |
   |---|---|
   | 1. Opaque typed composition, including repeated suspension | **Partly met, not closed.** Witness/return binding, typed finalizers and `Evaluate<Input=Expr, Output=Out>` are sound local checks (brief-603:397, :419); first chain Input, direct insertion and the token remain open/incorrect (F1/F2/F5; `.temp/brief-603/insertion-routes-v2.tsv:3`, `:4`, `:12`). |
   | 2. Sealed conversion and actual crossing population | **Partly met.** The sealed supertrait closes external implementations and an enum is feasible; public conversion/channel visibility and the grouped/nested-field census remain unresolved (F5/F6; `.temp/brief-603/crossing-payloads.tsv:28`, `:35`, `:36`). |
   | 3. Repaired inventory | **Not met.** The 600-site reconciliation and owner-qualified identities improve it, but the case-sensitive join invalidates macro flags; edge dispositions, chain rows and full synthetic/non-Result coverage are missing (F7; `.temp/brief-603/propagation_edges2.py:85`, `.temp/brief-603/propagation-edges-v2.tsv:1`). |
   | 4. Resolve deferred-finalization/lazy counterexamples | **Partly met.** Hoisting First/Normal capture to the real realize result is right; the prepared token loses progress, and the lower realization callback evidence is unaddressed (F1/F7; `crates/rum-runtime/src/runtime/eval_call_prepare.rs:435`, `crates/rum-stdlib/src/lazy.rs:446`). |
   | 5. Independently green steps and enumerated adapters | **Not met.** Early export removal breaks stdlib, the initial adapter lacks a compatible channel, and Option/matcher/guard steps cross protocols without adapters (F3; `crates/rum-stdlib/src/lazy/continuation.rs:13`, `crates/rum-runtime/src/runtime/definitions/application/down_values.rs:479`). |
   | 6. Ownership, transit, delegation and mismatch cleanup | **Partly met.** Arc ownership, all three delegations and Send/'static improve it; enum consumption avoids downcast. Fatal cleanup, adapter allocation accounting and transit policy remain incomplete (F4/F9; `crates/rum-runtime/src/runtime/suspension/core.rs:191`, `:627`, `:736`). |
   | 7. Compile/behavior/lifecycle/performance gates and E2 boundary | **Partly met.** Broad gate categories and corpus-position rules are good; several claimed edge witnesses miss their edges, fatal absence is suppressible, and hot steps lack full A/B (F4/F7/F10; `.temp/brief-603/probe.rs:46`, `:112`; brief-603:801). |

2. **ContinueToken and state:** no, the proposed token resumes stale progress.
   Use a privately minted type/obligation token accepting a fresh F value,
   not reinsertion of the original Arc. Carry the original input witness and
   keep F's output bound to the owning frame. F1 gives the replacement form
   and source proof (`eval_call_prepare.rs:391`, `:435`).

3. **Visibility/sealing:** not a satisfactory compile-and-privacy design as
   written. Distinguish private-interface warnings from hard private-type
   errors; neither doc-hidden items nor `pub(crate)` means driver-private.
   Put conversions on the hidden sealed supertrait, keep the erased channel
   inside the suspension implementation, and export typed APIs plus public
   carriers needed downstream. Stdlib does not need all runtime internals
   public to implement TypedFrame (F5; `kernel_api.rs:32`, `:129`). Verify
   this arrangement with downstream pass/fail controls during implementation.

4. **Declared side-effect outputs:** each can be compiler-bound or removed
   from the value-completion protocol. Default fallback can return its typed
   error directly; retention can keep a typed receipt through its required
   post-insert operations; public-file rollback can be a private structural
   abandonment entry. None justifies a general exported unbound insert
   (F2; `default_fallback.rs:41`, `accounting_mode/frozen.rs:133`,
   `session/batch_file.rs:20`).

5. **Transformed results:** family-by-family repair is acceptable only with
   the upfront site inventory and mandatory pre-flip semantic gates in F8.
   Same-type transformations are invisible to the witness, and an ok-arm
   edit must have a matching resumed computation. The supplied 29
   classifications do not establish 29 pre-macro transforms
   (`.temp/brief-601/macro-use-classification.tsv:210`, `:375`).

6. **Closed enum:** feasible, but the asserted exact population is not
   established. There are 35 grouped rows, including nested fields and
   unspecified composite/sum payloads, not 35 concrete crossing types (F6).
   State explicitly that adding a downstream payload requires a runtime
   carrier/variant/codec review. Ownership consequences are predictions to
   verify, including the adapter's second allocation; concrete enum storage
   is not automatically opaque host storage (F9; ledger `:53`, `:58`).

7. **Step plan:** no. Steps 604 and 606 already have concrete incompatibilities;
   later compiled/interpreted/guard boundaries and the unassigned populations
   lack protocol transitions. Preserve necessary legacy exports, establish
   a compatible scaffold/channel sequence, and migrate complete connected
   edges or named private adapters with deletion steps (F3). Full A/B gates
   must cover every hot-function-changing step, including matcher/traversal/
   block/selector work (F10; brief-603:739, :743, :801).

8. **Other issues and scope:** keep the out-of-band design, closed sealed
   conversion, typed route-E/finalizer checks, full oracle/lifecycle policy
   and honest external-Expr trust boundary. Repair fatal cleanup, admission
   accounting and the composite matcher payloads before implementation
   (F4/F6/F9). Keep E2's global no-`Try`/propagation enforcement deferred, but
   do the callback-witness and transformation work needed for G13 here. Do
   not promise E2 can leave every signature unchanged: its measured result
   representation may affect TypedAction, token helpers and frame return
   types as well as the macro (`.temp/brief-583/e2-outline.md:54`; contrast
   brief-603:677). No need to broaden this repair into arbitrary extension
   certification or a new general effect system.

## 3. Verdict and required changes

**Reject as ready to implement.** The local type checks and out-of-band
direction are substantially better than round 2, but the specified token
changes behavior, the chain/direct-insertion surface does not enforce its
claim, the first steps cannot remain green, and the evidence needed to build
the remaining connected migration groups is still incomplete. These are
design prerequisites, not details to discover under a compile-error budget.

Before dispatch, revise this design and its implementation briefs as follows:

1. Replace the Arc-capturing token with adapter-minted type evidence accepting
   a fresh frame of the same F. Specify initial and repeated-suspension
   evidence paths for every deferred-completion family, including lazy/schema
   callback helpers and synthetic-Err sites; preserve drop-before-finalizer.
2. Preserve both chain endpoints and witness the first Input. Eliminate the
   unrestricted insertion export; give fallback, retention and abandonment
   their checked/narrow routes, with output binding at the actual obligation.
3. Settle one Rust visibility arrangement: public typed APIs and accessible
   carriers, sealed hidden conversions, opaque actions/next wrappers and
   module-restricted erased construction. Add compile controls for every
   public route, including state-bearing stdlib implementations.
4. Implement a specified uncatchable mismatch route that structurally unwinds
   the failing frame and active suffix before returning the classified error.
   Gate rollback/restoration, fresh-session usability and independent absence
   detection under guest capture and error handlers.
5. Repair the table join; enumerate actual whole-slot types, including
   composite/sum carriers; supply chain rows and resolved edge dispositions,
   real input-witness provenance and step assignments. Retain unresolved
   calls honestly. Enumerate transformations globally, then repair and test
   them in the responsible family before its flip. Replace wrongly attributed
   witnesses with successful full-oracle controls for the actual edges.
6. Rebuild the green sequence from those edges. Keep enumerated legacy APIs
   until their users migrate, stage the typed adapter with a compatible
   driver, and combine connected migrations or name exact temporary bridges
   and their removal steps. Cover G-value/G-vec and every surviving old frame;
   verify no old codec/raw insertion remains at the end.
7. Specify the two-Arc accounting/admission change and synchronous transit
   contract, replay maintained and independent inventories, disclose actual
   row transitions, and preserve acknowledged Expr destruction debt. Require
   full per-label microbench A/B for every hot evaluator/matcher step, with
   instruction slopes supplementary and both feature builds' behavioral and
   lifecycle gates throughout.

The corrected design can retain the closed enum, invariant input witness,
return-position output binding and typed finalizers. No new unsafe, global
E2 migration or alternative transport architecture is required to address
these findings. **Verdict: reject.**
