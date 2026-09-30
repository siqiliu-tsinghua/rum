# brief-561 — design: make the lazy pull machine and its consumers resumable

`Scope: NONE — no production change.` Design-only, with one declared scratch
exception for probes (see Scope). Survey the lazy-sequence machinery, design how
a suspended pull resumes into it, and return drafted implementation briefs.
**Do not run `git` in any form**, including before you have read this brief.

## Goal

Step 4, group G2 of the suspension-continuation branch. brief-548 found 14 sites
where a lazy sequence that asks for input mid-realization resumes **past** its
work: the pull stepper propagates `InputRequired` with a plain `?`, its state
lives on the Rust stack, and the realization loop above it is lost. The resumed
request completes with the inner callback's value instead of the realized list.

| Site | Role |
| --- | --- |
| `crates/rum-stdlib/src/lazy.rs:163,167,175,179,193,211,214` | `PullStepper::next` arms: `NestList`, `Map`, `Select`, `FoldList`, `Take`, `Drop` |
| `crates/rum-stdlib/src/lazy.rs:437-439` | `eval_lazy_table` evaluating its bounds |
| `crates/rum-stdlib/src/lazy.rs:584` | `realize` driving `next` |
| `crates/rum-stdlib/src/associations/basic.rs:217` | `Normal` realizing |
| `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:79` | `Take` on a lazy list |
| `crates/rum-stdlib/src/structure_ops/surface.rs:99` | `First` on a lazy list |

`Part` on a lazy list re-dispatches to `First[LazyDrop[…]]` through the kernel
seam, so it inherits the `First` repair. brief-548's witnesses
(`.temp/brief-548/probes.json`, `lost.tsv`) show, for example,
`Normal[LazyTake[LazyMap[f, LazyRange[1, 1]], 3]]` with `f` asking for input at
the second element: resumed `"two"`, queued `{1, "two", 3}`. The director
re-ran all 195 probes on `6614e59b` and every G2 divergence still diverges.

**Design the change; write no production code.** The repair is structural — the
stepper's state has to become something a frame can own — so the design comes
first.

## What the design must settle

1. **Ownership of the pull machine.** Today a `PullStepper` tree is driven
   through a borrowed `Session` and `ctx` with its state on the stack. Say how a
   pending pull becomes owned state a frame can hold — the whole adapter tree
   and the source position, `NestList`'s lookahead value, `Select`'s pending
   candidate, `FoldList`'s accumulator, `Take`/`Drop` counters, the
   realization prefix and limit, and the pull-budget / ceiling context charged
   at the source — and how resuming applies the one pending callback result,
   unwinds through the adapters that were waiting on it, and continues the
   realization driver. **One frame owning the machine, or a frame per adapter?**
   Choose, with reasons.
2. **Zero cost on the ordinary path.** A pull that never suspends must not
   clone the stepper, allocate, or pay a branch it did not pay before. Move
   state into a frame only when a callback actually returns `InputRequired`
   (`Option::take`, moving out of the stack frame). brief-560 learned that
   carrying resumable state in a hot loop's error arm can change the loop's
   codegen (`Differences` kept +0.82% after ten candidates, from an inlining
   change); design so the ordinary `next` loop keeps its shape, and name the
   microbench labels that would expose a regression.
3. **The consumers.** `Normal`, `Take`, `First` (and so `Part`) each finish
   the realization — wrap the list, return the element, or fall back to the
   original call — after the machine resumes. Say what each consumer's frame
   carries.
4. **`LazyTable`'s bounds** are evaluated eagerly at construction
   (`lazy.rs:437-439`); a suspension in the second bound must resume into the
   third and then build the plan.
5. **Semantics to preserve**: `NestList`'s eager lookahead; lazy lists re-derive
   on each realization (no memo in v1); the ceiling and cancellation are
   charged at the source; infinite sources still fail closed.
6. **Suspension twice and abandonment**: a second suspension inside the resumed
   pull; cancellation while suspended runs no further pull and leaves the
   session usable.
7. **The retention line**: lazy realization holds a growing prefix — say which
   brief-547 region it falls in and what state your frames carry so that
   conversion can build on it. No leases here.

## What I want back

1. The current structure, with `path:line`: `PullStepper` and its arms, `ctx`
   and the budget charge, `realize`/`realize_at_most`, and every consumer.
2. The design, per the seven points above, with the frame inventory (fields
   and roots).
3. Red tests: every brief-548 G2 witness and its twice-position variant
   (`LazyMap-twice`, `LazyNestList-twice`, `LazySelect-twice`,
   `LazyFoldList-twice`, `LazyDrop-twice`), plus an ordinary error after a
   resume, cancellation mid-realization, and an infinite source still failing
   closed — each through the oracle (resumed equals queued in value, `log` and
   messages, suspension asserted). Run them in the scratch test and record the
   pre-fix values.
4. Drafted implementation briefs in `TEMPLATE.md` shape — one step or several,
   along the design's real boundaries — each with its red tests, its predicted
   lost-continuation count, its performance gate (preserved base binary,
   allocation counts first, then per-iteration `cpu_core/instructions:u` with
   `taskset -c 2`, `(count(2N) - count(N)) / N`, all labels measured for every
   candidate), and the standing inventory duty. Scope as objective, fence and
   disclosure duty; serial runs (`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- `AGENTS.md`, `docs/code-organization-guidelines.md`; `docs/guide/evaluation.md`
  §Lazy Sequences; `docs/dev/lazy-sequences-design.md`.
- brief-548's report (mechanism, oracle), `.temp/brief-548/draft-G2.md`,
  `probes.json`, `lost.tsv`, `probe.rs`.
- brief-559 and brief-560 — how a loop's frame carries its state, and what the
  ordinary path cost.
- `crates/rum-runtime/src/runtime/suspension/core.rs` (`:515`, `:709`, `:784`),
  `eval_dispatch/mod.rs:466`.
- `crates/rum-stdlib/tests/loop_continuations.rs` and
  `guest_loading_resume.rs` — the permanent oracle harness shape.

## Scope

- Files you MAY modify: **NONE**, with three exceptions:
  - append your report to **this** file;
  - write scratch under `.temp/brief-561/`;
  - create **one** scratch integration test,
    `crates/rum-stdlib/tests/zz_brief561_probe.rs`, and **delete it before
    appending the report**, copying its final content to
    `.temp/brief-561/probe.rs`. Say in the report that you did.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- Touch nothing outside this repository. No network. Commands: file reads,
  `rg`/`grep`, `python3`, edits to the one scratch test, and bounded
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 cargo test -p rum-stdlib --test zz_brief561_probe`
  through `python3 scripts/run-bounded.py`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch artifacts, deletion of the scratch
   test.
2. The current structure.
3. The design and the frame inventory.
4. The red tests with their pre-fix values.
5. The drafted implementation briefs.
6. Open questions for the director.

---

## Execution report — brief 561 (2026-09-30)

### 1. What changed

**Design only; no production change.** Appended this report to the brief and
created scratch evidence/design/drafts under `.temp/brief-561/`. No git
command, network operation, unrelated edit, new unsafe, lease, or dependency
was used. The only source edit was the authorized temporary integration test
`crates/rum-stdlib/tests/zz_brief561_probe.rs`. Its final content was copied to
[`.temp/brief-561/probe.rs`](../../../.temp/brief-561/probe.rs), then the test
was **deleted before this report was appended**.

The final pre-fix oracle run is **39 tests: 2 passed, 37 failed**, exit 101.
All 20 saved G2 cases diverge; all five saved twice-position cases reach only
the first prompt. The two passes are controls, not claimed repairs. No
performance measurement or implementation gate was run in this design scope.

Primary artifacts:

- [Design and source/frame inventory](../../../.temp/brief-561/design.md).
- [All sources and inputs](../../../.temp/brief-561/cases.json),
  [all pre-fix values/messages](../../../.temp/brief-561/red-results.md),
  [machine-readable rows](../../../.temp/brief-561/red-results.tsv).
- [Draft A — machine and consumers](../../../.temp/brief-561/draft-A-pull-machine.md),
  [draft B — eager bounds](../../../.temp/brief-561/draft-B-table-bounds.md).
- [Complete proposed performance gate](../../../.temp/brief-561/performance-gate.md),
  [fixed 30-label manifest](../../../.temp/brief-561/performance-panel.json).
- [Raw final test output](../../../.temp/brief-561/probe-run-02.txt),
  [run metadata](../../../.temp/brief-561/probe-run-02.json),
  [cleanup verification](../../../.temp/brief-561/cleanup-verification.json).

A Python source/doc/script hash audit recorded 1,731 baseline files. All
1,731 were unchanged after deleting the scratch test and before appending
this report. The final audit checks that only this brief changed, by append,
and that no new file remains under crates/docs/scripts. Build caches from the
explicitly authorized Cargo tests are generated artifacts, not source edits.
The original brief prefix is saved and hash-checked; no existing user edit
was reverted.

### 2. The current structure


| Current location | Responsibility and state |
| --- | --- |
| `crates/rum-stdlib/src/lazy.rs:12` | `PullBudget::{AtMost(usize),UntilExhausted}` bounds **emitted outputs** using `items.len()`; it is not the source-work ceiling. |
| `crates/rum-stdlib/src/lazy.rs:27`, `:42` | `PullContext` snapshots `iteration_work_limit`, starts `source_pulls=0`, and obtains a `CancellationObserver`. Every charged source pull checks cancellation, then `source_pulls >= iteration_limit`, then increments once. |
| `crates/rum-stdlib/src/lazy.rs:57`, `:105` | Owned `PullStepper` enum; the recursive `next(&mut self,&mut Session,&mut PullContext)` borrows it. Adapters own `Box<PullStepper>` children. No borrowed Session is embedded in a node. |
| `crates/rum-stdlib/src/lazy.rs:111` | Range: Rational `current,step`; charge, yield old current, advance. |
| `crates/rum-stdlib/src/lazy.rs:117` | Permutations: `Vec<Expr> current, exhausted`; check exhaustion before charge; clone yielded arrangement and advance canonical cursor. |
| `crates/rum-stdlib/src/lazy.rs:126` | Subsets: `elements,k,combo`; check exhaustion before charge; yield selected elements and advance subset indices. |
| `crates/rum-stdlib/src/lazy.rs:138` | Tuples: `inner_lists,digits,exhausted`; charge before yielding; advance mixed-radix digits. |
| `crates/rum-stdlib/src/lazy.rs:160` | NestList: `function,current`; charge; save old value; evaluate **next** current at `:163`; only then yield saved value. This is eager lookahead, including a final requested element. |
| `crates/rum-stdlib/src/lazy.rs:166` | Map: await child at `:167`, evaluate function at `:171`, wrap callback result in Some. |
| `crates/rum-stdlib/src/lazy.rs:174` | Select: await child at `:175`; retain candidate while predicate evaluates at `:179`; emit only exact truthiness true, otherwise continue scanning. |
| `crates/rum-stdlib/src/lazy.rs:184` | Take: if nonzero, **decrement before** returning the child result at `:189`. |
| `crates/rum-stdlib/src/lazy.rs:191` | Drop: skipped-child call at `:193`; decrement only after Some; exhaustion sets remaining=0. Once remaining is zero, tail-return the next child at `:199`. |
| `crates/rum-stdlib/src/lazy.rs:201` | FoldList: `function,accumulator,emitted_init,inner`; emit init without a source charge; subsequent child at `:211`, fold callback at `:214`, then replace accumulator and yield it. |
| `crates/rum-stdlib/src/lazy.rs:406` | LazyTable: held body/spec, normalized defaults at `:427`; evaluate lower, upper, step at `:437-439`, **all three before numeric validation**; `lazy_table_count` at `:629`; construct inert LazyTake[LazyMap[Function[…],LazyRange[…]],count] at `:451`. |
| `crates/rum-stdlib/src/lazy.rs:551`, `:558`, `:566` | `realize_all` and `realize_at_most` are tail wrappers around `realize`. Fully parse descriptor first, build fresh stepper at `:574`, new ctx at `:577`, empty prefix at `:578`; test output budget, call next at `:584`, push at `:587`; finish one List or return None for malformed descriptors. |
| `crates/rum-stdlib/src/lazy.rs:685` | `stepper_from_descriptor` validates and creates the whole tree afresh on every realization. Cursor helpers are `:649` and `:670`; wrapper is `:797`. |
| `crates/rum-stdlib/src/associations/basic.rs:200`, `:217` | Normal calls realize_all, unwraps Some(List), or rebuilds original head/args on None. Other Normal forms do not realize lazy values. |
| `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:20`, `:57`, `:79` | Take chooses lazy path before ordinary slicing; accepts nonnegative count or supported UpTo count, realizes at most limit, unwraps or rebuilds original call. |
| `crates/rum-stdlib/src/structure_ops/surface.rs:91`, `:99`, `:368` | First realizes at most one and calls resolve_first_like with optional already-evaluated default. Empty list returns default or emits nofirst and returns First[List[]]; malformed descriptor returns First[original args]. |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:113`, `:127` | Positive scalar lazy Part re-dispatches through evaluate(First[LazyDrop[lazy,i-1]]) as a true tail. It needs the First repair, not a runtime→stdlib dependency. Zero/negative/multidimensional Part behavior is unchanged. |
| `crates/rum-stdlib/src/structure_ops/surface.rs:160`, `crates/rum-stdlib/src/lazy.rs:543` | Rest only builds a LazyDrop descriptor. It performs no pull and requires no pull frame. Ordinary Drop has no lazy realization branch. |

The complete production call-site search found exactly the three direct
`realize_all`/`realize_at_most` consumers above. LazyTable **body** evaluation
uses Map after construction; it is not a fourth realization loop.

Current continuation anchors have moved since brief 548:
`runtime/suspension/core.rs:552` inserts at the back of the pending queue;
`:764` drives resumption, and `:803-829` appends the still-rooted outer suffix
after a nested suspension. `:839` is resume_or_suspend. The generic dispatch
frame at `runtime/eval_dispatch/mod.rs:466` completes with the received value;
it cannot reconstruct a dropped pull or prefix.

Frozen census obligations (all paths are crates/rum-stdlib/src):

| Implementation boundary | Frozen Lost IDs | Count |
| --- | --- | ---: |
| Machine and driver | P1898, P1899, P1901, P1902, P1904, P1906, P1907, P1913 | 8 |
| Normal / Take / First completions | P1483, P2353, P2355 | 3 |
| LazyTable eager bounds | P1908, P1909, P1910 | 3 |

Necessary adjacent edges are disclosed, not silently counted as additional
Lost repairs: Map callback `lazy.rs:171` (P1900), Take's child tail `:189`
(P1903), Drop's child tail `:199` (P1905), and wrapper tails `:555`/`:563`
(P1911/P1912) were **Unsettled** in brief 548. Capture the Map callback, and
prove the transparent tails in the design below. Do not revise their census
statuses in this branch. The old G2 draft's 85→71 counter assumed another
execution order; the current confirmed counter after brief 559 is **80**, so
these two drafted steps predict **80→69→66**, G2 **14→3→0**.


### 3. The design and frame inventory

### Ownership decision: one machine frame, separate caller completion

Choose **one LazyRealizeSuspensionFrame owning the whole machine**, rather
than a runtime frame per adapter. A source pull has one unfinished callback
and a unary chain of waiting adapters. Their counters and ctx form one unit;
per-adapter runtime frames would fragment source accounting, require an
Option<Expr> transport at every level, and make a second suspension and
consumer wrapping harder to audit. The source of truth remains the owned
stepper tree and one ordered pending-pull record, not a re-evaluated descriptor.

The frame owns the tree, realized prefix, output budget, saved source budget,
and pending callback/path. A Session borrow is used only while driving; no
reference into a Rust stack frame survives. The inert LazyList Expr remains
unchanged. A new realization still starts from its descriptor, even if this
one was cancelled, completed, or suspended previously.

#### Capture only on InputRequired; keep the ordinary next loop

Keep the ordinary `next` recursion, source arms, Select loop, Drop loop and
realize loop. Do not replace them with a phase interpreter entered on every
pull. Do not add a pending-state test, path push/pop, depth counter, frame
allocation, closure capture, clone of the tree, or Arc/Mutex owner per pull.

Append one cold scratch slot to the existing live PullContext:
`pending: Option<Box<PendingPull>>`, initially None. This is one stack word
per realization, not a per-element heap allocation. At the four guest
callback sites (NestList/Map/Select/FoldList), use the existing Result split;
only its Err edge calls a cold, non-inlined helper. That helper recognizes
InputRequired, moves the saved Nest yield or Select candidate into the
pending record, and initializes its leaf phase and empty waiter list. An
ordinary error is propagated without capturing anything.

At the existing non-tail child-Result checks (`:167,:175,:193,:211`), their
Err edges append a typed waiter to that same record, only for InputRequired:
`MapInput`, `SelectInput`, `DropSkip`, or `FoldInput`. The record is ordered
from the deepest waiter outward. No path bookkeeping executes on Ok.

**Do not add an error check to the tail arms.** Take `:189` and exhausted-skip
Drop `:199` still return inner.next directly. Their waiting effect is the
identity: Take already decremented; a Drop in its yielding phase has zero
remaining skips. At the top-level error boundary, a cold traversal of the
still-owned tree reconstructs the complete root-to-leaf path by consuming the
recorded waiters in reverse order, inserting transparent Take and zero-Drop
nodes as encountered. Stop at the recorded callback phase after all waiter
entries are consumed. This is unambiguous because every node has at most one
child, all other waiting ancestors recorded their phase, and callback leaves
are never Take or Drop. Validate node kind/phase in this cold code; reject an
internal inconsistency rather than applying an answer to a guessed node.

Example path at suspension:
`Take → Map(await child) → Drop(skip) → Take → Select(await predicate)`.
Only `[DropSkip,MapInput]` is recorded during unwinding, with
`SelectPredicate(candidate)` as the leaf. Cold expansion recovers both Takes
without new successful-tail branches. A zero-Drop is similarly transparent;
a positive-Drop **must** have its DropSkip waiter.

The realize Err arm calls a cold function by moving `stepper,ctx,items,budget`.
Taking `ctx.pending.take()` consumes the unique pending record; the stepper
can move directly out of the local after its mutable borrow ends. Do not wrap
it in an Option requiring an unwrap on each next. If an implementation uses
an outer owner slot, its take is restricted to this error path. This is the
first point that installs a lazy frame, behind the callback's existing
continuation frames. There is no whole-tree clone when first suspending.

The cold helper design is a code-shape constraint, **not measured evidence**.
Adding a cold arm can still alter register pressure, enum/result layout or
inlining, as brief 560 demonstrated. Keep the original next signature and
success ordering; isolate resumed driving and capture helpers. Measure the
new ctx stack word too. If it regresses, report/fix it; do not claim zero cost
because the source appears cold. Never solve it with unsafe or an ordinary
path diagnostic counter.

#### Applying the pending result and unwinding

A private resumed driver follows the saved path to the leaf, applies exactly
one completed callback result, and unwinds one `Result<Option<Expr>,EvalError>`
through the waiting ancestors. It does not call next at the root again until
this in-flight pull has completed.

| Phase | Apply once on resume |
| --- | --- |
| `NestLookahead { yield_value }` | Replace node.current with resumed value; yield saved old current. The source charge already happened. Do not yield the new value or recompute the callback. |
| `MapCallback` | Yield Some(resumed value); its child has already advanced. |
| `SelectPredicate { candidate }` | Convert predicate result using existing truthiness, drop that predicate result before any retry; yield candidate only for true. On false/non-true, discard candidate and continue the same Select's scan. |
| `FoldCallback` | Replace old accumulator with resumed value and yield it once. |
| `MapInput` | On child Some, run this Map's callback; on None, return None. A new callback suspension establishes a new leaf phase at this Map. |
| `SelectInput` | On Some, retain candidate and evaluate predicate, then apply the same decision/retry; on None, exhaust. |
| `FoldInput` | Keep emitted_init=true; on Some run fold with saved accumulator, then replace/yield; on None, exhaust. |
| transparent `Take` | Forward child's result without decrementing again, even when remaining is now zero. |
| `DropSkip` | Child Some consumes exactly one skip, so decrement once and continue remaining skips/next yielded child. Child None sets remaining=0 and exhausts. |
| transparent yielding `Drop` | Forward child result with no skip decrement. |

On a resumed callback Err, forward the error without another source pull,
prefix append, wrapping into a partial list, or conversion to a normal value.
On Some from the root, append one item; on None, finish; otherwise drive the
ordinary realization loop with the carried prefix and budget. Test budget
against total prefix length, not the number of answers or suffix items.

When a retry or an outer adapter suspends again, use a fresh pending record
and the same error-unwind protocol. The recursive resumed helper adds only
ancestors still waiting on this new operation; completed lower hops are not
copied into its path. A retry through ordinary next already records its own
inner waiters and must not add a duplicate waiter for the retrying node.
Move the updated running tree, prefix, counts and new path into a new machine
frame. The existing suspension driver places that new queue before the
original consumer/outer suffix. It does not enqueue the consumer twice.

An inner callback can itself suspend more than once before producing its
answer. In that case the machine frame stays in the suffix; it sees only the
callback's final value or error, never an intermediate input string.

#### Immutable Arc ownership is a real constraint

`SuspensionFrame::resume(self: Arc<Self>)` is immutable (`core.rs:79`).
`SuspendedEvaluation::front` clones the queued Arc (`:156`); the original
stays Session-rooted until after resume returns (`:777-786`). Therefore
`Arc::try_unwrap(self)` **cannot** be the design's normal ownership transfer.
Nor may a frame hide a Mutex/RefCell and take its contents out from under
root tracing.

Use an immutable frozen frame and create a mutable working copy **only after
resumption with Ok**, in the cold resumed entry. Clone the stepper nodes,
source cursors, prefix container and saved path there; Expr copies use their
existing semantics. The original frame remains a root during any subsequent
evaluation. Cloning on a genuine resume is allowed; cloning on the ordinary
path or first suspension is not. Err completion needs no working copy. The
working copy is moved into the next frame if it suspends again. This keeps
the runtime trait/driver unchanged and honestly exposes the overlapping
frozen/working ownership that the retention line must later charge.

### Frame inventory: fields, roots and completion

Every new frame derives FrameRootTrace and PureFrameAbandon. The private
Stepper, PendingPull, phase enums and scalar budget records also derive
FrameRootTrace; no ignored owned Expr field or permissive blanket tracing.

| Proposed frame/record | Fields | Expr/symbol roots and disposition |
| --- | --- | --- |
| `LazyRealizeSuspensionFrame` | `stepper: PullStepper`, `items: Vec<Expr>`, `budget: PullBudget`, `limits: SavedPullLimits { iteration_limit,source_pulls }`, `pending: PendingPull` | Every prefix item and every nested node/pending root below. No Session or borrowed ctx. Resume completes with encoded Option<Expr>; pure abandon drops state. |
| `PullStepper::Range` | current, step | Rational has no Expr/symbol roots, but numeric limbs remain a later capacity/accounting obligation. |
| `Permutations / Subsets / Tuples` | Current vectors, source elements/factors, k/combo or digits/exhausted | Trace every Expr in current/elements/inner_lists; primitive indices/flags have no roots. |
| `NestList / Map / Select / FoldList / Take / Drop` | Existing functions/predicate/current/accumulator/emitted_init/remaining/Box child | Trace functions, predicate, current, accumulator and child recursively. Keep every existing counter. |
| `PendingPull` | Leaf phase; saved Nest yield or Select candidate when applicable; expanded path/waiter enum sequence | Trace saved yield/candidate; scalar route has no roots. Map and Fold callback input is already owned by the inner evaluator's continuation, not copied again into a lazy-specific pending-input field. |
| `NormalLazyCompletionFrame` | Original evaluated `head: Expr,args: SharedArgs` captured cold | Decode Option<Expr>. Some returns the realized List; None rebuilds original call. Trace head and every arg. |
| `TakeLazyCompletionFrame` | Original evaluated head,args | Same decode/fallback. Limit is already in machine.budget, not recomputed from the spec. Trace head/args. |
| `FirstLazyCompletionFrame` | Original evaluated args, including optional default | Decode Option. Some calls existing resolve_first_like on realized List and saved default, preserving empty-list residual/message behavior; None rebuilds First[original args]. Trace all args. |
| `LazyTableBoundsFrame` | Original held `args: SharedArgs`, `iterator: Expr`, `bounds: [Expr;3]`, `pending_slot: Lower/Upper/Step` | Trace args (including held body/spec), iterator and all slots. Slots before cursor contain evaluated results; pending/current and later slots retain normalized expressions. Original args preserve invalid-spec diagnostic/fallback. |

The live observer has no FrameRootTrace implementation today
(`model/request_cancellation.rs:123`; it contains only the interrupt control
Arc). Do not smuggle it into a derived frame or add an arbitrary rootless
host-payload exemption. Freeze the two scalar source-budget fields, drop the
live observer, and reacquire `session.cancellation_observer()` when resuming
**the same request**, restoring the saved counters and ceiling. Do not call
PullContext::new and reset the work budget. Observer semantics are documented
as generation-agnostic (`request_cancellation.rs:116`). The request keeps its
own deadline; no new timeout starts in the lazy frame.

The realization/consumer boundary continues to have type
`Result<Option<Expr>,EvalError>`. A resumed machine must explicitly encode its
Option with the existing `ContinuationValue` codec
(`suspension/primitive.rs:36`); consumer frames decode it before completing.
Encode only on suspension/resume, never on a normal successful call. This
prevents a raw callback Expr or InternalContinuationSome wrapper from being
mistaken for a finished user result. Once a machine exists, descriptor parsing
has succeeded and normal resumed success is always Some(List); None remains
the pre-pull malformed-descriptor fallback, not a recoverable callback error.

Normal and Take thus have deliberately small completion frames. First has a
real post-pull extraction/default/message step. Part inherits that frame via
its symbolic seam. No Rust dependency from runtime to stdlib is added.

### LazyTable bounds are a separate continuation

Keep its ordinary lower→upper→step straight-line evaluations. Capture original
held args, iterator, normalized slots and pending slot only on InputRequired.
For a suspension in upper, the lower slot is already evaluated and remains
unchanged; consume the upper outcome exactly once, then evaluate step, then
validate **all** three values and build the plan. A suspension in step freezes
the first two values and resumes directly into validation/construction.
Default 1 bounds are normalized once, not reparsed from the original spec.

On ordinary error, forward it; on invalid numeric/zero-step input, use the
original invalid_lazy_table_spec helper and original held args. In particular,
validation must not move earlier and skip side effects in later bounds.
Never evaluate the held body while building a descriptor. The separate pull
frame later evaluates that body through the existing Function/Map lowering.
Cold resumed entry copies only frozen bound state under the same immutable
Arc constraint, then moves it into a replacement bounds frame on suspension.

### Safety, reruns and abandonment

Source charges remain exactly at Range, Permutations, Subsets, Tuples and
NestList. Filters' rejected candidates and Drop's skipped values consume
source work; adapters never charge again for unwinding. Fold init consumes no
source pull. Exhausted finite sources preserve their current exhaustion-before-
charge order, and output budget is checked before requesting another element.
An infinite source under Normal, or a never-true Select under First, must hit
the saved source ceiling even after one or many suspensions.

PureFrameAbandon for the machine and consumers only discards roots; it never
pulls, invokes a predicate or advances a counter. Existing inner scope and
cleanup frames retain their own abandonment behavior. Request cancellation
while suspended must take the established abandon route before any lazy
resume; the scratch cancellation oracle verifies no further callback and
`1+1 == 2`. A second-suspension cancellation test must first prove the second
prompt was reached; today's early completion is a failure, not a cancellation
pass. Continue to respect reset/snapshot lifecycle at the owning host layer.

No memo or position is stored on LazyList. The repeated-realization witness
must run the original callbacks twice, from the source origin each time.

### Retention handoff, without leases

The **growing realized prefix** belongs to brief 547's **R02,
callback-result prefixes and histories** (`.temp/brief-547/regions.tsv`, R02).
The machine exposes it as one named field instead of hiding it on the stack.
Select's candidate, NestList's current/lookahead, and FoldList's accumulator
are needed-value slots; they need the R02/R05 adoption/slot machinery later.
LazyTable's completed lower/upper bounds are already assigned **R05** in
brief 547. Source combinatorial indices/capacities and Rational limbs need
separate R14/R09-style ownership/projection review, not an assumption that
an output count bounds bytes.

FrameRootTrace provides roots, not a lease, a LocalCovered certificate, or
an allocation bound. On resume the immutable frame remains S while the copied
working prefix/tree exists; future conversion needs independent charged work,
move/freeze handoff, and exactly-once receipt retirement on success/error/
abandonment. Replacing Vec with a frozen retained prefix must preserve that
ownership overlap rather than steal from the Session-rooted Arc.

Census caveat for the director: brief 547 left First and Take **Unsettled**;
Normal's recorded projectable row described only its association-normalizer
branch. The lazy path in this survey is evidence that the whole Normal
surface needs reconsideration on the later retention line. Do not rewrite
those classes, numbers, statuses or exclusions here. The original lazy design
also says realized elements are never retained; `realize`'s growing output
Vec disproves that wording for realizers. Descriptor/cursor space can be
bounded separately from output-prefix space. Record this documentation debt;
no unrelated document was edited.

### Implementation boundaries and module assessment

A: machine, driver and all three consumer completions together: 11 Lost sites.
Do not ship a pull frame that leaves an internal Option wrapper exposed at a
consumer. B: eager LazyTable bounds: 3 Lost sites. Body+bound composition is
verified after B on top of A.

Current `lazy.rs` is 23,887 chars; `take_drop.rs` 20,978; `surface.rs` 14,618;
`associations/basic.rs` 9,737. Keep existing owner identities and hot ordinary
bodies in place for this repair; put **new cold** capture/resume code under
`lazy/continuation.rs` and `lazy/table_continuation.rs`. Keep consumer frames
next to their three owning call sites. This is a split assessment, not a claim
that lazy.rs is already small: its eventual true split is constructors/plan
parsing versus source/adapter pulling versus realization/completion. Existing
ownership-inventory identities must not be silently renamed by moving bodies.
If a mechanical split requires that, the director must authorize the exact
moves in a rewritten brief. New modules must stay under the guideline limits
or be split at the next real responsibility boundary.

A private continuation submodule can borrow/refer to parent private types;
expose only pub(super) helpers needed by lazy.rs. Do not widen the public
kernel API to support this design. Consumer frames use the existing public
kernel_api continuation traits/codecs, just as loop_continuations does.

The two implementation drafts and fixed performance panel are separate
artifacts. They are raw material for director rewrite, not approved dispatches.

### 4. Red tests and observed pre-fix values

The final scratch test uses rum's own queued-input execution as the oracle,
following `loop_continuations.rs:13` and `guest_loading_resume.rs:82`.
It compares Expr values (not only formatted strings), ordered full
RuntimeMessage vectors, error/termination, prints, and final `log`. It
asserts the exact number and order of `InputRequired` prompts. Each completed
path checks no pending input owner and that a new `1+1` request returns `2`.
The tables display actual Expr formatting; `<error>` means no success value,
with the exact failure recorded below and in the raw log. None of the observed
wrong values is asserted as expected language behavior.

All 20 saved G2 sources and input lists were copied **verbatim** from
`.temp/brief-548/probes.json` and checked for exact equality. The original
14-site inventory is a call-site count, not a count of these 20 tests.

| Probe | Resumed value | Resumed log | Queued value | Queued log | Prompts reached / required |
| --- | --- | --- | --- | --- | --- |
| `LazyMap` | `"two"` | `List[1, 2]` | `List[1, "two", 3]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyNestList` | `3` | `List[1, 2]` | `List[1, 2, 3, 4]` | `List[1, 2, 3, 4]` | 1 / 1 |
| `LazySelect` | `True` | `List[1, 2]` | `List[1, 2, 3]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyFoldList` | `3` | `List[1, 2]` | `List[0, 1, 3, 6]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyDrop-inner` | `"two"` | `List[1, 2]` | `List[3, 4]` | `List[1, 2, 3, 4]` | 1 / 1 |
| `LazyMap-inner` | `"two"` | `List[1, 2]` | `List[List[1], List["two"], List[3]]` | `List[1, 2, 3]` | 1 / 1 |
| `LazySelect-inner` | `"two"` | `List[1, 2]` | `List[1, "two", 3]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyFoldList-inner` | `"two"` | `List[1, 2]` | `List[List[], List[1], List[1, "two"], List[1, "two", 3]]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyTake-consumer` | `"two"` | `List[1, 2]` | `List[1, "two", 3]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyFirst-consumer` | `"two"` | `List[1, 2]` | `List["two"]` | `List[1, 2]` | 1 / 1 |
| `LazyPart-consumer` | `"two"` | `List[1, 2]` | `List["two"]` | `List[1, 2]` | 1 / 1 |
| `LazyTable-body` | `"two"` | `List[1, 2]` | `List[1, "two", 3]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyTable-bound-1` | `1` | `List[1]` | `LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyTable-bound-2` | `3` | `List[1, 2]` | `LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyTable-bound-3` | `1` | `List[1, 2, 3]` | `LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyMap-twice` | `"two"` | `List[1, 2]` | `List[1, "two", 3, "four", 5]` | `List[1, 2, 3, 4, 5]` | 1 / 2 |
| `LazyNestList-twice` | `3` | `List[1, 2]` | `List[1, 2, 3, 4, 5]` | `List[1, 2, 3, 4, 5]` | 1 / 2 |
| `LazySelect-twice` | `True` | `List[1, 2]` | `List[1, 2, 3, 4, 5]` | `List[1, 2, 3, 4, 5]` | 1 / 2 |
| `LazyFoldList-twice` | `3` | `List[1, 2]` | `List[0, 1, 3, 6, 10, 15]` | `List[1, 2, 3, 4, 5]` | 1 / 2 |
| `LazyDrop-twice` | `"two"` | `List[1, 2]` | `5` | `List[1, 2, 3, 4, 5]` | 1 / 2 |

For all 20 rows above, both paths have status OK, no termination, no messages,
and no prints. Every row diverges in value; most also lose a log suffix.
The third-bound case shows why matching the log alone is insufficient.

Additional cases exercise the design's state and lifecycle boundaries:

| Probe | Resumed value | Resumed log | Queued value | Queued log | Prompts reached / required |
| --- | --- | --- | --- | --- | --- |
| `LazyMap-error-next` | `2` | `List[1, 2]` | `<error>` | `List[1, 2, 3]` | 1 / 1 |
| `LazyMap-error-in-callback` | `<error>` | `List[1, 2]` | `<error>` | `List[1, 2]` | 1 / 1 |
| `Lazy-infinite-after-resume` | `2` | `List[1, 2]` | `TerminatedEvaluation["IterationWorkLimit"]` | `List[1, 2, 3, 4]` | 1 / 1 |
| `LazySelect-never-after-resume` | `False` | `List[1, 2]` | `TerminatedEvaluation["IterationWorkLimit"]` | `List[1, 2, 3, 4]` | 1 / 1 |
| `Lazy-source-budget-exact` | `2` | `List[1, 2]` | `List[3, 4]` | `List[1, 2, 3, 4]` | 1 / 1 |
| `LazyFirst-empty-default` | `False` | `List[1, 2]` | `99` | `List[1, 2, 3]` | 1 / 1 |
| `LazyFirst-empty-message` | `False` | `List[1, 2]` | `First[List[]]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyNestList-lookahead-first` | `2` | `List[1]` | `1` | `List[1]` | 1 / 1 |
| `LazyTable-bounds-twice` | `3` | `List[1, 2]` | `LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]` | `List[1, 2, 3]` | 1 / 2 |
| `LazyTable-invalid-step` | `3` | `List[1, 2]` | `LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input["p: "]], CompoundExpression[AppendTo[log, 3], 0]]]` | `List[1, 2, 3]` | 1 / 1 |
| `LazyTable-body-and-bounds` | `3` | `List[]` | `List[1, "two", 3]` | `List[1, 2, 3]` | 1 / 2 |
| `Lazy-adapter-unwind-twice` | `2` | `List[List[inner, 1], List[outer, 1], List[inner, 2]]` | `List[List[1], List[2], List[3]]` | `List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]` | 1 / 2 |
| `LazySelect-reject-resumed-candidate` | `False` | `List[1, 2]` | `List[1, 3, 4]` | `List[1, 2, 3, 4]` | 1 / 1 |
| `Lazy-rerederive` | `List["two", "two"]` | `List[1, 2, 1, 2]` | `List[List[1, "two", 3], List[1, "two", 3]]` | `List[1, 2, 3, 1, 2, 3]` | 2 / 2 |
| `LazyPermutations-cursor` | `List[1, 3, 2]` | `List[List[1, 2, 3], List[1, 3, 2]]` | `List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]` | `List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]` | 1 / 1 |
| `LazySubsets-cursor` | `List[1]` | `List[List[], List[1]]` | `List[List[], List[1], List[2]]` | `List[List[], List[1], List[2]]` | 1 / 1 |
| `LazyTuples-cursor` | `List[1, 4]` | `List[List[1, 3], List[1, 4]]` | `List[List[1, 3], List[1, 4], List[2, 3]]` | `List[List[1, 3], List[1, 4], List[2, 3]]` | 1 / 1 |
| `Lazy-cancel-at-1` | `TerminatedEvaluation["Cancelled"]` | `List[1, 2]` | `TerminatedEvaluation["Cancelled"]` | `List[1, 2]` | 1 / 1 |
| `Lazy-cancel-at-2` | `2` | `List[1, 2]` | `TerminatedEvaluation["Cancelled"]` | `List[1, 2, 3, 4]` | 1 / 2 |

The exact nonempty message/error/output observations are:

- `LazyMap-error-next`: resumed status OK/value `2`, one `probe::tag`, prints
  `["before"]`; queued fails with
  `Other { message: "uncaught throw: 9 with tag None" }`, ordered messages
  `probe::tag`, `probe::tag`, `Throw::nocatch`, prints `["before","after"]`.
  The error after the first resumed element is skipped today.
- `LazyMap-error-in-callback` is a **passing control**: both fail with that
  same uncaught-throw error after one suspension; log `List[1, 2]`, messages
  `probe::tag`, `Throw::nocatch`, no prints. The inner callback's own error
  already propagates; new lazy frames must not turn it into a partial list.
- `Lazy-infinite-after-resume` and `LazySelect-never-after-resume`: resumed has
  no messages/termination; queued terminates as `Some(IterationWorkLimit)`
  with `General::reslim`, text `evaluation exceeded iteration work limit (4)`.
  First with a never-true predicate must still fail closed despite an output
  budget of one. `Lazy-source-budget-exact` is the finite counterpart: four
  source pulls, including two skips, must produce precisely `List[3, 4]`.
- `LazyFirst-empty-message`: resumed `False`, no messages; queued
  `First[List[]]` with **two** `First::nofirst` entries, each text
  `List[] has zero length and no first element.` The equality oracle preserves
  the current message multiplicity; this branch does not change it.
- `LazyTable-invalid-step`: resumed `3` with no message; queued returns the
  original held call and `LazyTable::argform`, text
  `expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step`.
- `Lazy-cancel-at-1` is the second **passing control**: both terminate as
  `Some(Cancelled)`, message `General::cancel` / `evaluation cancelled by host`,
  one captured `boundary` print, log `List[1, 2]`; no further callback runs.
  `Lazy-cancel-at-2` fails because the resumed path ends at `2` after one
  prompt. Its queued control reaches the second boundary and cancels, with
  two prints and log `List[1, 2, 3, 4]`. That is an unreached second
  cancellation point, not evidence that second-boundary abandonment works.

All remaining supplemental rows have OK status, no termination/messages or
prints. `LazyNestList-lookahead-first` proves that even First must await the
lookahead callback and then return old current `1`, not new current `2`.
The combinatorial cases expose each source cursor beneath a suspending Map;
`Lazy-rerederive` requires two independent realizations of the same descriptor.

Only these two bounded Cargo invocations were run, serially:

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-561/tmp cargo test -p rum-stdlib --test zz_brief561_probe -- --nocapture
```

The same-shape serial integration tests in `.temp/brief-560/commands.jsonl`
completed in 11.44–15.11 seconds under an 80-second bound; that is the bound
used here. The measured durations below are command bookkeeping, not
performance results or wall-clock benchmark decisions.

First invocation, elapsed 7.483834 seconds, child exit 101:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.22s
running 39 tests
test result: FAILED. 1 passed; 38 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
error: test failed, to rerun pass `-p rum-stdlib --test zz_brief561_probe`
```

This run exposed a **harness mistake**, not an extra product defect:
cancellation's queued path installed a Print sink (which consumes print
output), while its resumed path used buffered output. Corrected the scratch
harness to install identical sinks and compare captured output on both paths;
no assertion was weakened and no production code changed. The first probe
and log remain in `probe-v1.rs` and `probe-run-01.txt`.

Final invocation, elapsed 6.919637 seconds, child exit 101:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.64s
running 39 tests
test result: FAILED. 2 passed; 37 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
error: test failed, to rerun pass `-p rum-stdlib --test zz_brief561_probe`
```

No formatter, runtime/stdlib/workspace suite, retention replay, release build,
perf/taskset process or microbenchmark ran: this brief permits only the one
scratch integration-test target. No source change remains that would require
an inventory refresh. Follow-up implementation gates are drafted separately.

### 5. Drafted implementation briefs

Both full drafts have TEMPLATE.md's Goal, Context pointers, Scope,
Deliverable, pitfalls, Verification, Checkpointing, Hard rules and Report
sections. They are review material for the director, not approved dispatches.

| Order and artifact | Objective | Recorded pre-fix gate | Predicted frozen Lost count |
| --- | --- | --- | --- |
| [A — owned pull and consumers](../../../.temp/brief-561/draft-A-pull-machine.md) | Entire machine, prefix/budget, Normal/Take/First completion; Part through the existing seam. All 11 obligations ship together. | 33 recorded cases: 31 red, 2 passing controls. Plus mandatory pre-code probes for last-item Take, alternating transparent adapters, and two inputs within one callback. | Global 80→69; G2 14→3. |
| [B — eager LazyTable bounds](../../../.temp/brief-561/draft-B-table-bounds.md) | Lower/upper/step continuation, validation and inert plan construction. | Six recorded red cases; retain all A cases. Add bound-specific error/cancel and nonnumeric-lower/later-effect probes before production edits. | Global 69→66; G2 3→0. |

The boundaries follow the actual machines: all pull adapters and consumers
must compose before their typed continuation is useful; the eager bounds
machine can then be fixed independently. A excludes B's three bound calls.
B retains A unchanged and checks the body-plus-bounds composition.

Each Scope states its objective, named edit surface, fence and disclosure
duty. Each includes the **current** standing inventory duty: replay contract
fingerprints and default/no-default certificates; reconcile ownership rows;
allow disclosed new UnknownConstruct rows in changed code; STOP on an existing
row identity/status/disposition/exclusion change or disappearance, a new
UnknownHostPayload, or a row in untouched code. There is no silent retention
promotion or source relocation exemption.

Both require `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`, offline Cargo,
repository-local scratch, bounded serial commands, real pre-fix equality
failures, focused green tests, fmt, runtime, stdlib, workspace and both-feature
retention replay. No existing correctness test may be weakened.

Their [complete performance gate](../../../.temp/brief-561/performance-gate.md)
preserves base default/alloc binaries before edits and every measured
candidate. It measures **all 30 fixed labels for every candidate**, allocation
counts first, then per-iteration `cpu_core/instructions:u`, `taskset -c 2`,
N=20,000 and 40,000, `(count(2N)-count(N))/N`, two samples per tuple, one label
per process and stdin `/dev/null`. It also requires full
core-language-microbench A/B timings.tsv rows from monotonic AbsoluteTiming;
no competing CPU workload and no substituted wall timing.

The panel includes 17 exact new no-input statements for range, normal, map,
select/scan, nest, fold, drop, First, Part, transparent adapters, both LazyTable
construction forms and realization, permutations, subsets and tuples; all nine
brief-559 labels including Differences; and `table-small`,
`table-iterator-plus`, `table-iterator-pair-list`, `take-multispec-small`.
Exact statements/shapes are in the linked manifest. No zero-cost result is
claimed here. Compare against the incoming tree, so brief 560's existing
Differences residual is inherited rather than wrongly attributed to G2.

The drafts set progress counters, stop after two consecutive compiling
non-improving attempts, and cap measured candidates/process runs. A new
regression or missing gate is reported, not accepted as an implementation
success. Bounds for full builds/suites must come from their own measured
serial histories in the director's rewritten briefs.

### 6. Open questions for the director

1. Ratify the proposed incremental **+0.05% in each instruction sample** noise
   threshold inherited from brief 560, or specify a stricter decision rule in
   the implementation briefs. The no-new-allocation/branch design is settled;
   its codegen cost is still unmeasured. Preserve the incoming base, including
   its already-recorded Differences residual, before either implementation.
2. The design respects immutable Session-rooted frames by copying only on
   actual resume. This makes an O(prefix + tree) cold working copy possible.
   Is that acceptable for this continuation branch? Recommended: yes; charge
   both overlapping owners in the later R02/R05 conversion. If moving out of
   the rooted Arc is required instead, that needs a separate runtime protocol
   design and broader scope, not a hidden Mutex or try_unwrap assumption.
3. Record the later retention follow-up: First/Take were Unsettled and Normal's
   projectable entry covered only its association path. The lazy output prefix
   is an R02 mechanism, and the original lazy design's “never retain realized
   elements” wording excludes this growing Vec incorrectly. No census or
   unrelated doc changed here; decide where to correct those records after
   the suspension branch.
4. Ratify keeping existing hot owner identities in place while adding only
   cold continuation submodules. A broader constructors/pull/realization split
   is sensible, but changing existing inventory identities conflicts with the
   standing STOP rule unless the director explicitly authorizes the exact
   relocation map. The current file sizes and suggested boundaries are above.

No design work is blocked on these questions; the report, executed pre-fix
oracle and both rewrite-ready drafts are complete. Production repair and its
measured gates remain future work.
