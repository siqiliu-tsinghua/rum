# brief-602 — review: is the G13 repair design acceptable?

`Scope: NONE — review only.` **Do not run `git` in any form**, including
before you have read this brief. You write no code and run no probes: this is
a reading review of a design another worker produced.

## Background

G13 is a defect in rum's suspension machinery. When an evaluation suspends for
input, frames are queued and later resumed; a frame hands its completed value
to the next frame as an `Expr`. Typed continuation state (an `Option<Expr>`, a
`bool`, a match outcome, …) is encoded in-band as `Expr` under
``System`InternalContinuation*`` heads and decoded by the next frame. Those
heads are ordinary symbols matched by text, so guest source can write them;
the decoders are lenient or panic on unexpected shapes; and nothing checks
that a frame completes the type its consumer decodes. Six frames complete a
guest value raw into an `Option<Expr>` slot, so a guest value such as
``System`InternalContinuationSome[5]`` resumes to `5`, while the same program
with its input queued in advance returns `InternalContinuationSome[5]`
(21 executed divergences, reproduced by the director).

Two design briefs produced the repair under review:

- `docs/dev/codex/brief-600-g13-typed-continuation-state-design.md` — the
  inventory (§2-§3, verified) and a first design (§4), whose binding rule the
  director rejected.
- `docs/dev/codex/brief-601-g13-design-revision.md` — the director's
  corrections (top) and the revised design (its report, §1-§7): an
  out-of-band channel `FrameOutcome { Value(Expr), State(Box<dyn
  ErasedSlotValue>) }`, typed frames with `Input`/`Output`, the output bound by
  a `return suspend_typed(…)` in the macro's suspend arm, a per-stage split of
  the dispatch-plan frames, a no-panic mismatch error, a `marker-corpus`
  oracle falsifier, and six drafted implementation steps (602-607 in its
  numbering).

Supporting tables are under `.temp/brief-600/` and `.temp/brief-601/`
(`macro-use-classification.tsv`, `ast-macro-positions.tsv`,
`insertion-routes.tsv`, `propagation-edges.tsv`, …).

## The three requirements the repair must meet

1. A frame cannot complete a slot with the wrong type — **a compile error**,
   not a convention.
2. Guest code cannot construct anything a decoder accepts as typed state —
   **out of band**, not a renamed symbol.
3. Decoding cannot panic or misread on guest input.

Plus the project's standing rules: no ordinary-path cost (the hot evaluator
path must not slow down), no new `unsafe`, and a fix the compiler enforces is
preferred over one that relies on review.

## What to review

Judge the revised design (brief-601's report, read with brief-600's inventory)
and answer, each with `path:line` evidence from the source or the tables:

1. **Soundness of the binding.** Does `return suspend_typed(session, frame,
   request)` really force `F::Output` to equal the enclosing function's or
   closure's Ok type in every macro site? Consider closures, generic frames,
   `where` clauses, sites whose enclosing function does not return
   `Result<_, EvalError>`, and any way the binding can be satisfied while the
   frame still completes the wrong thing.
2. **The input side.** The report says a frame's `Input` cannot be bound at
   compile time without an ordinary-path cost, so it is declared and checked
   at runtime. Is that true?
3. **Coherence of the step plan.** Can each drafted step compile and land
   green on its own, in the order given? In particular, check the first step's
   claim that every frame becomes an identity `Expr → Expr` typed frame with no
   behaviour change while the return-form macro lands in the same step.
4. **Requirement 2 in practice.** Can any implementor — in the kernel, the
   standard library, or an external crate using the exported API — still put
   typed state through the `Value` arm, or otherwise reintroduce in-band
   encoding?
5. **Coverage.** Is every route by which a frame enters a queue or a
   completion reaches a frame accounted for, with a binding? Are there macro
   sites the classification misses?
6. **Propagation without a frame.** `propagation-edges.tsv` lists 51 edges
   where a may-suspend call is propagated across a change of Ok type. The
   report says none diverges today. What happens to those edges when typed
   state moves out of band?
7. **Representation and ownership.** Is `Box<dyn ErasedSlotValue>` the right
   choice over a closed enum, and is its stated ownership/destruction
   disposition honest?
8. **Relation to E2** (`.temp/brief-583/e2-outline.md`, a may-suspend result
   type with no `Try`): is the stated sequence right?
9. Anything else that is wrong, missing or over-scoped.

## Verdict

End with one of: **accept**, **accept with required changes** (list them), or
**reject** (say why and what instead). Rank findings by severity; separate
what you verified in source from what you infer.

## Scope

- Files you MAY modify: **NONE**, except: append your review to this file;
  scratch notes under `.temp/brief-602/`.
- No scratch test, no Cargo command, no probe of any kind.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository.
- Commands: file reads, `rg`/`grep`, `python3` over the `.temp` tables
  (through `python3 scripts/run-bounded.py --timeout 300 -- …`).
- **Testing Policy is a hard line** (`AGENTS.md`).

## Report format (append to this file)

1. Findings, ranked by severity, each with evidence and whether verified or
   inferred.
2. Answers to questions 1-9.
3. Verdict and the required changes.

---

## 1. Findings, ranked by severity

Reading review only. I read the two design reports, queried their existing
tables, and checked the relevant production sources. No git, Cargo, network,
test, compiler experiment, or runtime probe was run. The only file changed is
this appended review. **Verified** below means verified by reading source or
tables; **inferred** means a consequence of the proposed signatures/control
flow, not an executed compiler or behavioral result. Earlier workers' witness
results remain their evidence, not new executions by this review.

**F1 — P1 / blocker: step 1 contradicts its own output binding, and steps 2–3
do not form independent green migrations.**

Verified: brief-601 requires every frame to have `Input = Expr, Output = Expr`
while introducing the return-form macro, with no decoder change
(`docs/dev/codex/brief-601-g13-design-revision.md:723`, `:744`, `:759`). But
`try_apply_function` returns `Result<Option<Expr>, EvalError>` and inserts its
frame at the macro
(`crates/rum-runtime/src/runtime/scoping/callable.rs:5`, `:65`). Argument
preparation returns `PreparedCallArguments`
(`crates/rum-runtime/src/runtime/eval_call_prepare.rs:226`), and selector
evaluation returns `EvaluatedPartSelectors`
(`crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:117`).
These are existing, required sites, not hypothetical extensions.

Inferred: precisely the proposed check rejects `return Result<Expr, _>` at
those sites. Compilation cannot reach zero while preserving both the identity
conversion and the new binding. Giving the frames their real output types is
already a semantic/channel conversion and needs their consumers converted.

The next steps also have a dependency inversion. Step 2 emits
`State(Option<Expr>)` into dispatch, while the stage split and prepared-call
conversion are deferred to step 3 (brief-601:762, :792). The same dispatch
consumer currently handles both prepared arguments and optional call results
(`crates/rum-runtime/src/runtime/eval_core/call.rs:291`). It cannot remain an
identity `Expr` consumer, become a single `Option<Expr>` consumer, and accept
both protocols under the accepted one-input-type rule. Other dispatch
producers must migrate with it or have explicit temporary adapters. An
unchanged channel representation does not make a runtime-first/stdlib-second
split safe; semantic payload types also have to agree.

Required: separate representation scaffolding from enforcement, or convert
complete producer/consumer components together. A temporary legacy path must
be explicit, bounded, and removed; it must not be called the enforced repair.

**F2 — P1 / blocker: the enclosing return type is only a local binding; it is
not always the type the queued continuation must ultimately deliver.**

Verified counterexample: initial argument preparation inserts its first frame
in a function returning `PreparedCallArguments`
(`crates/rum-runtime/src/runtime/eval_call_prepare.rs:231`, `:269`). On resume,
the helper returns `SuspendedPreparedCallArguments`; another suspension inserts
a different frame inside that helper (`:401`, `:436`). Both frame bodies call
`finish_prepared_call_action`, whose finalizer materializes the completed
prepared-call payload
(`crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:26`, `:56`,
`:64`, `:79`). The dispatch consumer expects that completed payload
(`crates/rum-runtime/src/runtime/eval_core/call.rs:301`).

Inferred: the new macro binds the second frame's output to
`SuspendedPreparedCallArguments`, while step 3's
`SlotFinalizer<PreparedCallArguments>` requires that same frame's output to be
`PreparedCallArguments`. These cannot both hold. If the finalizer is instead
removed to satisfy the enclosing helper type, the next suspension can deliver
the accumulator to a dispatch frame expecting completed arguments. The driver
has already removed the previous frame before appending the nested queue to
its suffix (`crates/rum-runtime/src/runtime/suspension/core.rs:736`, `:762`);
the missing conversion is not retained automatically.

A second concrete example is the lazy `First` capture helper. It returns
`Result<Option<Expr>, _>`, but its frame deliberately completes the entire
`First` call as `Expr`
(`crates/rum-stdlib/src/structure_ops/surface/lazy_completion.rs:27`, `:46`).
The caller propagates the helper error with `?` from `Result<Expr, _>`
(`crates/rum-stdlib/src/structure_ops/surface.rs:75`, `:85`). Merely changing
that frame's output to `Option<Expr>` to satisfy the helper creates a state
completion where the builtin dispatch wrapper expects `Expr`. `Normal` has
the same structure
(`crates/rum-stdlib/src/associations/basic/lazy_completion.rs:27`, `:41`;
`crates/rum-stdlib/src/associations/basic.rs:219`).

Required: distinguish a helper's intermediate ordinary result from its
deferred completion obligation. Refactor captures into correctly typed tail
helpers, and represent deferred finalization/conversion as a typed continuation
that survives another suspension. Audit these compositions now, not only in
E2. A correct local return equality does not prove queue adjacency.

**F3 — P1 / blocker: insertion and action APIs still permit wrong-type
completion despite a correctly bound macro.**

Verified: the proposed non-returning public insertion requires only
`F: TypedFrame`; its output is explicitly left “by declaration” or pinned by a
unit list (`.temp/brief-601/insertion-routes.tsv:5`, `:7`). Neither binds it to
the producer or consumer. `DirectDefaultFallbackFrame`, one claimed cleanup/
identity exception, actually consumes `Option<Expr>`, can execute fallback
evaluation, and produces `Option<Expr>`
(`crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs:16`).

The Route E constructor binds only `Input = Expr`
(`.temp/brief-601/insertion-routes.tsv:8`). Its required additional constraint
is `next.Output = current.Output`: the next frame replaces the current
frame's remaining computation. The proposed public
`TypedAction<Out>::Evaluate` stores an erased frame with neither constraint
in its field type (brief-601:452). A comment on the field cannot enforce them.
The present driver pops the originating frame before pushing `next_frame`
(`crates/rum-runtime/src/runtime/suspension/core.rs:736`, `:749`).

Inferred: a frame with a correctly checked `Output = Option<Expr>` can return
an Evaluate action whose successor completes `Expr`; all stated constructor
bounds can hold. Direct construction of the public variant bypasses even the
Input constraint. The `TypedFrame: SuspensionFrame` supertrait also preserves
an implementor-supplied erased `resume`, instead of making erasure exclusively
an adapter responsibility (brief-601:442; current raw trait at
`crates/rum-runtime/src/runtime/suspension/core.rs:81`).

Required: make erased queue/action construction private or sealed; separate
the public typed trait from the erased driver trait; make typed actions opaque
with checked constructors. Route E must bind both its input and final output;
finalizers must bind the owning output. Replace unrestricted insertion with
typed composition or narrowly typed structural operations. Preserve the
retention insertion's ordering, but do not exempt its type obligation.

**F4 — P1 / blocker: returning from each insertion skips required frames;
the classification also misses sites and misidentifies a closure boundary.**

Verified: `match_except_interpreted_owned` queues a prepare frame and then a
continue frame before returning its error
(`crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:92`, `:100`).
There are analogous chains at `:176`/`:183` and `:261`/`:270`, plus chains
outside that family, for example
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:183`.
The classification explicitly calls chained insertion return-equivalent
(`.temp/brief-601/macro-use-classification.tsv:553`, `:554`, `:548`).

Inferred: replacing the first insertion by `return suspend_insert(...)`
skips the rest. Further, intermediate frames in a chain need an intermediate
output type, not the enclosing function's final output. The chain must be
composed, with adjacent types checked, and returned once.

Verified coverage defects: the classification has 599 data rows but only 598
distinct sites; `builtin_adapters.rs:934` is duplicated
(`.temp/brief-601/macro-use-classification.tsv:206`, `:207`). Two additional
production invocations are absent:
`crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:406`
and `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:212`.
Both occur inside an outer macro's continuation tokens. The walker records
the outer macro without descending into those tokens
(`.temp/brief-601/probe.rs:74`). A source search therefore exposes at least
these omissions in the claimed exact reconciliation.

The selector publishing API is present but misclassified: it returns
`EvaluatedPartSelectorsEvaluation::Suspended { error, continuation }`, not
`Result<_, EvalError>`
(`crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:133`,
`:149`; `.temp/brief-601/macro-use-classification.tsv:360`). Its deletion is
deferred to step 3, too late for step 1's macro replacement.

Finally, the argument-preparation insertion is inside the immediately invoked
closure starting at `eval_call_prepare.rs:233`. Its existing `return` at
`:270` already exits that closure; the outer profiling record at `:377` still
runs. Brief-601:345 incorrectly requires moving that record into the suspend
arm. The builtin timing records really do require preservation: their recorder
pops the timing stack
(`crates/rum-runtime/src/runtime/profiling/timing.rs:342`). These cases must
not share one relocation rule.

**F5 — P1 / blocker: the proposed slot API does not enforce the out-of-band
requirement or its claimed closed implementation set.**

Verified: `SlotValue`, its encoding/decoding methods, `FrameOutcome` variants,
and `ErasedSlotValue` are public in the design (brief-601:424). There is no
sealed supertrait. The report substitutes “no blanket impl” and a unit test
for closure of the set (brief-601:438, :606). The kernel extension API already
exports the raw frame/action traits
(`crates/rum-runtime/src/runtime/kernel_api.rs:129`), and the draft adds these
new exports (brief-601:670).

Inferred: an external crate can implement the public trait for its own state
newtype, encode it as `Value(Expr)`, and decode an ordinary guest expression
as that state. Runtime and stdlib implementors can do the same. No type in the
proposal prevents this; `#[doc(hidden)]` changes documentation, not access.
The public erased action path provides another bypass. Thus “only the adapter
constructs/decodes State” is a convention contradicted by the signatures.

Required: centralize and seal channel conversion so only `Expr` uses Value
and registered state carriers always use State with exact Rust identity.
Resolve how downstream payloads are admitted before promising a closed set.
Even a sealed transport cannot forbid arbitrary Rust authors from deliberately
implementing a symbolic protocol inside an `Expr -> Expr` computation. State
the certified implementation domain and enforce its decoder/registration
rules; do not claim a repository unit test proves properties of arbitrary
external implementations. E2 explicitly recognizes that limit
(`.temp/brief-583/e2-outline.md:34`, `:49`).

**F6 — P1 / migration blocker: the 51-edge propagation table is neither a
complete inventory nor evidence that these edges can wait for E2.**

Verified: the extractor constructs every function identity as
`file:line:::name` (`.temp/brief-601/propagation_edges.py:41`) and filters both
endpoints against that identity (`:88`, `:94`). The actual may-suspend list
uses owner-qualified identities for methods, e.g.
`file:line:Session::prepare_arguments_for_call`
(`.temp/brief-583/may-functions.json:332`). Consequently it drops the method
domain, including the argument-preparation problem in F2. It also skips
unresolved/indirect calls and non-Result carriers (`propagation_edges.py:34`,
`:56`). Counting all macros in a caller (`:104`) does not establish that any
one call's suspension is caught by a frame. All 51 retained rows are stdlib
free-function edges (`.temp/brief-601/propagation-edges.tsv:2` onward).

The three spot witnesses do not prove the family claims. In particular the
“string guard” witness is a delayed RHS without a guard, and the Prime witness
is a scalar argument evaluation
(`.temp/brief-601/propagation-spot-witnesses.tsv:2`, `:4`). The relevant Prime
edge is list threading, whose only suspending branch is
`session.evaluate(threaded).map(Some)`
(`crates/rum-stdlib/src/number_theory.rs:276`, `:289`). These witnesses do not
exercise all the edges their family summaries purport to protect.

Inferred: some cross-type paths remain correct after migration because a
real conversion frame is already installed; others require changing the
capture helper or installing a conversion now (F2). A third class has
canceling wrappers across a longer path: `compute_inner` applies `.map(Some)`
and `eval_inner` immediately extracts Some
(`crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:257`;
`crates/rum-stdlib/src/functional_ops/arrays/mod.rs:31`). Leaving the whole
tail path as Expr can work; typing only its intermediate completion changes
that obligation. Exact downcasts reveal a missing conversion as an error;
they do not supply the conversion. “None diverges today” is not a sufficient
migration disposition.

**F7 — P2: compile-time input binding does not inherently require an
ordinary-path value or frame allocation.**

Verified: the impossibility claim is at brief-601:515. The current macro
already matches a typed `Result` while constructing the frame solely on its
error arm (`crates/rum-runtime/src/runtime/suspension/core.rs:805`).

Design inference: borrow that Result before matching to obtain a zero-sized,
non-borrowing, invariant type witness for its Ok type `I`. A cold insertion
method on the witness can require `F: TypedFrame<Input = I>` and return
`Result<F::Output, EvalError>`. The match still moves the result once, and the
frame is still built only on suspension. `PhantomData<fn(I) -> I>` can carry
the equality without an `I` value, a heap allocation, or a retained borrow of
the Result. This is a type-level counterexample to the impossibility claim,
not measured code-generation evidence; implementation must undergo the normal
performance gate.

This checks the local Result's type, not provenance already erased to an
`EvalError`. Cold helpers using synthetic `Err(error)` need explicit input
types or a witness carried from the real call. For example the Differences
order frame consumes `Vec<Expr>` although the current synthetic macro's Ok
arm is inferred as Expr
(`crates/rum-stdlib/src/structure_ops/list_plumbing.rs:47`, `:89`). Repeated
suspension and unframed propagation still require F2/F6's composition work.
Runtime exact-type checks remain useful at erasure boundaries; they need not
replace available static checks.

**F8 — P2: the representation comparison and ownership disposition contain
unsupported claims.**

Verified: the alleged stdlib counterexamples to a closed enum (brief-601:570)
include stored frame fields, not demonstrated channel payloads.
`ExactStringCaseOutput` is a field of a string rule
(`crates/rum-stdlib/src/strings/patterns.rs:142`, `:161`). Lazy state is stored
in `LazyRealizeSuspensionFrame`, which consumes Expr and completes encoded
`Option<Expr>` (`crates/rum-stdlib/src/lazy/continuation.rs:20`, `:53`, `:87`).
Schema state is stored in its generic frame, whose implemented resume path
consumes and returns Expr
(`crates/rum-stdlib/src/schema/continuation.rs:16`, `:38`, `:53`). Generic
frame storage does not imply generic channel state. The inventoried decoder
definitions in `.temp/brief-600/decoders.tsv` are runtime definitions. The
report does not establish an unavoidable downstream-owned slot type.

Inference: an enum cannot directly name inaccessible downstream types with
the current dependency direction, but “a closed enum is impossible” does not
follow. Compare actual crossing types, with the cost of extracting neutral
carriers or exposing runtime-owned carriers, against controlled erasure. The
box may still be a reasonable choice, but its stated justification is not
sufficient.

Verified: the driver carries `outcome` across loop iterations, removes/drops
the producer, and then passes the outcome into the next resume call
(`crates/rum-runtime/src/runtime/suspension/core.rs:723`, `:736`, `:745`,
`:747`). Queued root tracing visits frames
(`:184`). Thus brief-601:598's assertion that state lives only within one
iteration and that its tracing/destruction therefore remain “on the frames”
is inaccurate. Absence from loader snapshots is supported
(`crates/rum-runtime/src/runtime/model/session.rs:84`), but does not settle
ownership during the synchronous handoff or resumed evaluation.

The existing Unbounded/Expr-debt disposition is honestly identified
(`crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv:53`).
It must not become a bounded-destruction assertion. However, “exactly one new
row” is unsubstantiated: the drafted TypedAction adds erased Evaluate and
Finalize storage as well as FrameOutcome's State storage (brief-601:425,
:452), and new public generic inputs also need classification. The shown
`as_any(&self)` supplies only a borrow; safe ownership-preserving downcast
also needs a consuming erasure operation or equivalent. Its mismatch branch
must drop the owned payload correctly without panic or `unsafe`.

Required: record actual storage/public-input edges and a transit lifetime,
rooting/accounting, error, and destruction contract. Distinguish verified
existing debt from predicted new rows; do not infer the latter's exact count
without an implementation inventory.

**F9 — P2: adapter and verification obligations are incomplete.**

The adapter must delegate `output_boundary`, not only tracing/abandonment.
Trace frames override it
(`crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:12`),
pause packaging reads it
(`crates/rum-runtime/src/runtime/suspension/output.rs:51`), and abandonment
uses it (`crates/rum-runtime/src/runtime/suspension/core.rs:671`). Defaulting
it silently loses capture visibility/rollback behavior. Also, the proposed
`Arc::unwrap_or_clone` extraction (brief-601:463) lacks a Clone bound, and
existing frames such as `TryApplyFunctionSuspensionFrame` do not implement
Clone (`crates/rum-runtime/src/runtime/scoping/callable.rs:141`). The driver
legitimately retains the queued Arc during resume. Specify the adapter's
actual ownership model and preserve drop-before-finalizer behavior; do not
introduce a uniqueness assumption or mandatory whole-frame clone implicitly.
`SlotFinalizer` also needs the Send/'static requirements demanded by the
erased finalizer it will inhabit (compare brief-601:457 with
`crates/rum-runtime/src/runtime/suspension/core.rs:96`).

The mandatory microbench A/B cannot wait until the final step while an earlier
step changes hot evaluator functions. Brief-601:753 gates step 1 with
instruction counts, whereas `AGENTS.md:117` requires per-label microbench
A/B. The E2 protocol expressly makes instruction slopes supplementary
(`.temp/brief-583/e2-outline.md:92`). A cold branch rewrite can still change
ordinary-path code generation. Profiling timing *values* also cannot be
required byte-identical between executions: they include measured elapsed
nanoseconds (`crates/rum-runtime/src/runtime/profiling/timing.rs:347`). Compare
record identity/count/classification and balanced stack behavior instead.

The descriptor corpus is useful, but six shapes do not prove all in-band
decoding impossible. Brief-601:664's “every included descriptor is red” is
not established by 21 observations of selected witnesses. The corpus must
have explicit per-descriptor inclusion/exclusion and preserve meaningful
pauses for each substituted position. Testing all positions simultaneously
does not subsume testing them independently when an earlier value changes
control flow. Finally, `accept` with expectations disabled checks equality,
not a universal prohibition on internal mismatch errors
(`crates/rum-stdlib/tests/suspension_oracle/descriptors.rs:191`). Define a
separate assertion that no SlotTypeError occurred, including if guest error
handling would otherwise consume it. The current EvalError enum has no named
internal slot-mismatch variant
(`crates/rum-runtime/src/runtime/session/session_state.rs:499`); select a
concrete classification and cleanup/propagation policy rather than leaving
an “existing internal-error channel” unspecified.

## 2. Answers to questions 1–9

1. **Binding soundness.** For an actual `Result<T, EvalError>` return position,
   the proposed helper binds its associated output to T for the intended
   owning slot types. Generic frames and where clauses do not evade that
   equality; they must provide the required associated-type constraints.
   For example the generic merge-sort frame has concrete Expr completion
   despite generic stored items
   (`crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:174`, `:321`).
   `return` binds the innermost closure, as the explicit closure at
   `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:241`
   illustrates. It does not bind a surrounding function's later conversion,
   a queued successor, or another suspension's completion. Non-Result sites
   cannot use it unchanged. F2–F4 identify concrete counterexamples to the
   broader guarantee, including a correctly typed action that delegates to
   the wrong output type.

2. **Input side.** The impossibility claim is false. An invariant zero-sized
   witness of the existing Result's Ok type supplies a static bound without
   constructing a frame on success (F7;
   `crates/rum-runtime/src/runtime/suspension/core.rs:805`). Its optimized
   cost must be measured. An error-only helper needs the original type
   evidence carried in or a redesigned typed helper; an invented annotation
   on a synthetic Err is not evidence about the producer.

3. **Green step plan.** No. Step 1 contradicts the binding at existing typed
   return sites; chained insertions and the selector enum require structural
   changes, and steps 2–3 share consumers that cannot straddle the proposed
   protocols (F1, F4). The prepared-call family additionally needs a changed
   deferred-finalization design (F2;
   `crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:64`).
   The drafts need to be replaced, not merely relabeled or assigned a larger
   compile-error budget.

4. **Requirement 2 in practice.** Not enforced. Public, unsealed SlotValue
   methods can encode/decode state through Value; public erased actions can
   bypass typed constructors (F3, F5; brief-601:424, :452). Closing transport
   conversion and removing raw construction is necessary. State the trust
   boundary for arbitrary Expr-processing Rust extensions honestly.

5. **Coverage.** The major physical routes are identified: insertion,
   Complete/Finalize, Evaluate's front insertion, input seeding, nested queue
   splicing, and request completion
   (`crates/rum-runtime/src/runtime/suspension/core.rs:505`, `:607`, `:723`).
   Their *bindings* are incomplete: non-returning insertion has none for
   adjacency; Evaluate lacks output equality; seed/end checks are runtime
   checks; splicing preserves order but cannot create a missing type proof.
   The source-site census misses the two nested macros in F4, and the
   propagation census drops methods (F6). A unit-list pin is not a
   compile-time binding. Closure scopes, chained intermediate slots,
   custom return carriers, and synthetic-Err helpers need explicit entries.

6. **Frameless propagation.** Out-of-band state makes mismatched chains fail
   at an exact-type check instead of enjoying a lenient Expr interpretation.
   Whether a listed edge needs a new frame depends on its actual chain:
   Differences already installs conversion frames through its cold helpers
   (`crates/rum-stdlib/src/structure_ops/list_plumbing.rs:30`, `:47`, `:151`);
   lazy captures must change as in F2; canceling Option wrappers may be
   normalized as a whole tail path as in F6. Rebuild the edge inventory with
   producer, conversion, consumer, resuspension behavior, and migration
   disposition. Do not move state across an unresolved edge and defer its
   correctness to E2. The three recorded controls establish only their
   specific executions.

7. **Representation and ownership.** Boxed erasure is viable in principle,
   but neither its necessity nor its closed-set property is established.
   Separate stored frame fields from crossing payloads before comparing an
   enum with erasure (F8;
   `crates/rum-stdlib/src/schema/continuation.rs:16`, `:38`). The Unbounded
   debt acknowledgement is appropriate; the one-iteration lifetime and
   exact-one-row claims are not. Specify safe consuming downcast, all adapter
   delegation, synchronous transit accounting/rooting, and drop behavior.
   No new unsafe is needed or justified by the design.

8. **E2 relationship.** A corrected out-of-band transport and typed frame/
   action layer can precede E2 and should be reusable. But blanket G13
   completion followed by all propagation work in E2 is not sound: F2/F6
   expose propagation that must be resolved for G13 to work at all. E2
   explicitly includes FrameAction, helpers, callbacks, restricted
   dispositions, and checked tail position in its design scope
   (`.temp/brief-583/e2-outline.md:19`, `:34`, `:54`). It may change action and
   adapter suspension signatures as well as the two-line helper. Keep its
   representation choice open until the authorized measured prototype.
   Integrate the necessary typed-composition work into G13 now, or coordinate
   the overlapping migration with that prototype; do not claim the present
   Result-preserving design proves all suspension propagation correct.

9. **Other issues.** Retain the accepted out-of-band direction, mismatch-as-
   error requirement, stage-specific consumer types, and rum-vs-rum corpus.
   Add compile-fail/pass obligations for wrong Input/Output, Evaluate output,
   finalizer output, generic/closure sites, direct insertion, and external
   codec implementations. Preserve lifecycle checks and adapter output
   boundaries (F9;
   `crates/rum-runtime/src/runtime/suspension/output.rs:51`). Replacing the
   existing published-selector capability is justified only with its callers
   and typed replacement included in the same migration; it is not an
   incidental mechanical macro edit. Performance gates and honest corpus
   coverage claims need the corrections in F9.

## 3. Verdict and required changes

**Reject the revised design as ready for implementation.** Its central
direction is worth keeping, but the claimed compiler guarantee has open
construction routes, the first green step cannot compile under its own rules,
and the return binding is applied to helpers whose deferred result differs
from their ordinary result. These require a revised design and migration
graph before the drafted implementation steps can be dispatched.

Required changes:

1. Define an opaque typed composition boundary: checked Input and Output,
   checked adjacent frames, Route E preserving the parent's output, and
   finalization/conversion that survives repeated suspension. Keep erased
   queue operations inside the driver and its adapters.
2. Seal slot-to-channel conversion and settle the actual slot population and
   downstream extension policy. Only Expr may use Value. Compare an enum and
   controlled erasure using crossing payloads, not all stored frame fields.
3. Repair the inventory: nested macro tokens, closure return scopes,
   owner-qualified method identities, non-Result carriers, chained inserts,
   synthetic-Err helpers, and every producer/conversion/consumer edge. Record
   unresolved edges explicitly instead of treating them as covered by E1.
4. Resolve the prepared-argument finalization and lazy-capture examples, plus
   their analogous families, before moving those payloads out of band.
   Compose multi-frame insertions and return once; use the zero-sized input
   witness where the original typed Result is available.
5. Redraft independently green steps. A possible first step is representation
   scaffolding with unchanged legacy control flow, explicitly making no
   enforcement claim. Subsequent steps must migrate complete connected
   producer/consumer groups with temporary adapters, if any, enumerated and
   scheduled for deletion. The final state must have no legacy in-band
   continuation decoder or unrestricted insertion escape.
6. Specify safe owning downcast, adapter ownership and all delegation,
   payload transit/rooting/accounting, classified mismatch cleanup, and actual
   ownership/retention inventory consequences. Preserve acknowledged
   destruction debt without asserting it away.
7. Gate each relevant migration with compile-fail/pass controls, both feature
   builds' oracle and lifecycle checks, justified descriptor corpus coverage,
   explicit absence of SlotTypeError, and the required microbench A/B whenever
   hot code changes. Instruction slopes supplement that gate. Coordinate the
   remaining propagation guarantee with E2's measured design.

No implementation or experiment was performed in reaching this verdict.
**Verdict: reject.**
