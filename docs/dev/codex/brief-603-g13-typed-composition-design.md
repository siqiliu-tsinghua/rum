# brief-603 — design round 3: G13 as a typed composition boundary

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

G13: typed continuation state crosses suspension frames in-band, as `Expr`
under ``System`InternalContinuation*`` heads guest source can write, decoded
leniently or by panic, and nothing checks that a frame completes the type its
consumer decodes (six frames complete a guest value raw into an `Option<Expr>`
slot; 21 executed divergences). Two design rounds produced a direction that is
accepted and a migration that is not:

- brief-600 (inventory, first design) and brief-601 (revision: out-of-band
  channel, typed frames, output bound at the enclosing return position).
- brief-602, an independent review by another model, **rejected the revision
  as ready for implementation** with nine findings (F1-F9) and seven required
  changes. The director verified its counterexamples in source and adopts the
  review. Read `docs/dev/codex/brief-602-g13-design-review.md` §1-§3 in full;
  it is the core input of this brief.
- The director's own verdict, written before the review:
  `.temp/director/gate-601/director-verdict.md`. Where it and brief-602
  overlap, brief-602's formulation governs.

This round designs G13's repair as **a typed composition boundary** — checked
adjacency between frames, not only a local binding at each insertion — and an
independently green migration to it.

## Settled (do not reopen)

- The out-of-band channel: `Expr` is the only type that travels as a guest
  value; typed state is a Rust value guest source cannot name. Sealed or renamed
  symbols are rejected.
- A slot mismatch is a classified error in every build, never a panic.
- One input and one output type per frame; the dispatch-plan frames split per
  stage.
- The rum-vs-rum `marker-corpus` oracle falsifier, without panic shapes.
- The end state deletes every continuation marker, the lenient decoders,
  `ContinuationValue`, and any unrestricted insertion escape.

## Director rulings

1. **Adopt brief-602's required changes 1-7.** Each is a section of this
   design (below).
2. **Typed composition belongs to G13, not to E2.** Whatever propagation G13's
   migration depends on to work at all (F2, F6) is designed and migrated here.
   E2 keeps only what remains after G13: making a bare `?` on a may-suspend
   result a compile error, and the result representation its measured
   prototype chooses. State that boundary exactly.
3. **brief-601's open questions:** (1) a loud classified error, through a
   **named** `EvalError` variant you specify with its cleanup and propagation
   policy (F9); (2) the styled-message frame gets the explicit typed body;
   (3) corpus positions are tested **each separately** and per shape, with
   explicit per-descriptor inclusion/exclusion; (4) splitting a migration by
   crate is allowed only with a temporary legacy path that is enumerated and
   scheduled for deletion; (5) the `()`-carrier edges are resolved here if
   G13's migration depends on them, otherwise handed to E2 with a reason.
4. **Input binding** uses a zero-sized invariant witness of the original
   `Result`'s Ok type where that `Result` is available (F7); synthetic-`Err`
   helpers carry the producer's type evidence in, or are redesigned.
5. **Performance:** any step that changes a hot evaluator function carries the
   per-label microbench A/B (`AGENTS.md:46-48`, `:117-121`), with one-codegen-unit
   instruction slopes as a supplement; profiling-record obligations compare
   record identity, count, classification and balanced stack depth, not
   elapsed values (F9).

## What the design must settle

1. **The typed composition boundary** (required change 1). An opaque typed
   frame/action layer whose public surface cannot complete a wrong type:
   checked `Input` and `Output`; checked adjacency between consecutive frames;
   Route E (`Evaluate`) preserving the parent's output (`next.Output ==
   parent.Output`); finalization and conversion that survive a repeated
   suspension; erased queue and action construction private to the driver and
   its adapters; a public typed trait separate from the erased driver trait.
   **Deferred completion obligations** (F2): distinguish a helper's ordinary
   result from the obligation its frame discharges — the prepared-call
   finalization (`eval_call_prepare.rs:231-436`,
   `eval_call_prepare/resume_frames.rs:26-79`, `eval_core/call.rs:301`) and the
   lazy `First`/`Normal` captures (`structure_ops/surface/lazy_completion.rs`,
   `associations/basic/lazy_completion.rs`) are the two verified examples; find
   the rest of their families and say, for each, where the obligation is
   created and how its type is checked there. **Chained inserts** (F4,
   `pattern_matching_interpreted.rs:92-104` and its siblings,
   `replace/once.rs:183`): composed, adjacent types checked, returned once.
2. **Sealed conversion and the slot population** (required change 2). Only
   `Expr` uses the value arm; every state carrier uses the state arm by exact
   Rust identity; implementors choose a type, never an arm. Settle the actual
   **crossing** payload types (not stored frame fields — F8) and compare a
   closed enum with controlled erasure on that set. State the downstream
   extension policy and the honest trust boundary for arbitrary `Expr -> Expr`
   Rust implementations.
3. **The inventory, repaired** (required change 3): nested macro tokens
   (`confirmation/builtins.rs:406`, `replace/once.rs:212` were missed),
   closure return scopes, owner-qualified method identities (the 51-edge table
   dropped every method), non-`Result` carriers
   (`evaluated_part_selectors.rs:133-149`), chained inserts, synthetic-`Err`
   helpers, and every producer → conversion → consumer edge with its
   resuspension behaviour and migration disposition. Unresolved edges are
   listed as unresolved. Witnesses must exercise the edge they are cited for
   (F6: the string-guard witness had no guard; the Prime witness was the wrong
   edge).
4. **The verified counterexamples resolved** (required change 4): the
   prepared-argument finalization and the lazy captures, plus their analogous
   families, before their payloads move out of band.
5. **An independently green step plan** (required change 5). A first step of
   representation scaffolding may keep legacy control flow and makes **no
   enforcement claim**; every later step migrates a complete connected
   producer/consumer group; any temporary adapter is enumerated with the step
   that deletes it; the final state has no legacy decoder and no unrestricted
   insertion. Show the producer/consumer groups and their order.
6. **Ownership, transit and the adapter** (required change 6): safe consuming
   downcast (no `unsafe`); the adapter's ownership model without a `Clone`
   requirement or a uniqueness assumption, preserving drop-before-finalizer;
   delegation of `FrameRootTrace`, abandonment **and `output_boundary`**
   (`debug_tools/trace/resume_frames.rs:12`, `suspension/output.rs:51`,
   `suspension/core.rs:671`); the outcome's real lifetime across driver
   iterations (`core.rs:723-747`) and its rooting, accounting and drop;
   mismatch cleanup; `Send`/`'static` bounds. Ownership and retention
   inventory consequences stated as predictions, with how each implementation
   step verifies them; the `Expr` destruction debt stays acknowledged
   (`host_payload_ledger.tsv:53-55`).
7. **Gates** (required change 7): compile-fail and compile-pass controls for a
   wrong `Input`, a wrong `Output`, a wrong `Evaluate` output, a wrong
   finalizer output, generic and closure sites, direct insertion and an
   external codec implementation (this repository proves `compile_fail` with
   doctests — use the existing pattern); both feature builds' oracle and
   lifecycle; justified corpus coverage; an explicit assertion that no slot
   mismatch occurred, including where guest error handling would consume it;
   microbench A/B and instruction slopes per ruling 5.

## What I want back

1. The repaired inventory tables (`.temp/brief-603/*.tsv`), with counts that
   reconcile against a raw source scan, and witnesses tied to their edges.
2. The typed composition design (sections 1, 2, 4, 6) with Rust signatures,
   the sealing mechanism, and why each bound is sound — including the cases
   brief-602 broke the last design with.
3. The E2 boundary (ruling 2).
4. The step plan as producer/consumer groups, and redrafted implementation
   briefs in `TEMPLATE.md` shape, one per gated step, each with its gates.
5. Open questions for the director.

## Context pointers

- brief-600, brief-601 and brief-602 with their reports; `.temp/brief-600/`,
  `.temp/brief-601/` (tables, scripts, `probe.rs`);
  `.temp/director/gate-600/`, `.temp/director/gate-601/`.
- `crates/rum-runtime/src/runtime/suspension/`, `kernel_api.rs`,
  `rum-derive`, `runtime/tests/frame_root_trace*`.
- `crates/rum-stdlib/src/tests/ownership_graph/` and its ledger; the retention
  inventories; the oracle under `crates/rum-stdlib/tests/suspension_oracle/`.
- `.temp/brief-583/` (`e2-outline.md`, census tooling,
  `may-functions.json`), `.temp/brief-548/enforcement.md`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-603/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief603_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-603/probe.rs`.
- Everything else is read-only — including `.temp/brief-600/` and
  `.temp/brief-601/`, whose files you copy rather than edit.
- This worker has no OS-level write fence: these exceptions are the whole
  fence. Write nothing outside this repository.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-603/`.
- **Run no probe that panics or aborts.** A probe that would reach a panic is
  reported, not run.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief603_probe`
  (900 s); `cargo test -p rum-stdlib --lib ownership_graph` (900 s) if you
  need its current output; `python3` scripts (300 s each). File reads,
  `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.
- Checkpoint to `.temp/brief-603/PROGRESS.md` after each section with the
  exact next action.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The repaired inventory and witnesses.
3. The typed composition design and the resolved counterexamples.
4. Sealing, the slot population and the trust boundary; ownership and transit.
5. The E2 boundary; the gates.
6. The step plan and the redrafted implementation briefs.
7. Open questions for the director.

---
# brief-603 report — G13 as a typed composition boundary

## 1. What changed

Design only; no production source change. Scratch under `.temp/brief-603/`:
`sites_reconcile.py`, `propagation_edges2.py`, `build_tables.py` and their
outputs (`sites-reconciliation.tsv`, `propagation-edges-v2.tsv`,
`propagation-unresolved.tsv`, `crossing-payloads.tsv`,
`deferred-completion-families.tsv`, `insertion-routes-v2.tsv`,
`non-result-sites.tsv`, `chained-inserts.tsv` below in §2.5, `witnesses.tsv`,
`probe.rs`, `PROGRESS.md`). The one scratch integration test
`crates/rum-stdlib/tests/zz_brief603_probe.rs` was declared, run green
(**1 passed**; 14 family-edge equality witnesses), copied to
`.temp/brief-603/probe.rs`, and **deleted** before this report was appended.
Also run, within the allowed commands: `cargo test -p rum-stdlib --lib
ownership_graph` (green, **20 passed**, 60.46 s; ledger 65 rows — the
baseline for §4.5's predictions). No `git` command was run. No probe that
panics was run (the 2-arg `Some` shapes remain director-side). Commands were
serial, each through `python3 scripts/run-bounded.py`.

## 2. The repaired inventory and witnesses

### 2.1 Macro sites — exact reconciliation (required change 3)

A raw text scan (`rg -o 'resume_or_suspend!'`) over `crates/rum-runtime/src`
+ `crates/rum-stdlib/src` finds **600 production occurrences** (599 excluded:
`src/tests/` trees, `tests.rs` modules, and one inventory `.tsv` that merely
mentions the name). brief-601's AST census has 598 distinct real sites (599
classification rows; `builtin_adapters.rs:934` is duplicated — F4 verified).
The raw-minus-census diff is **exactly the two nested-token sites**
`confirmation/builtins.rs:406` and `replace/once.rs:212` (macros inside
another macro's ok-arm IIFE), which the walker recorded only at the outer
macro. 598 + 2 = 600: the census is now exact, and the two misses'
**classification**: nested ordinary-arm sites inside an immediately-invoked
closure whose Ok type equals the enclosing function's, so the return-form
binding applies unchanged (source-read, both files).

Producer side: the 600 sites' enclosing functions have **30 distinct Ok
types** + **8 non-`Result` sites** (enumerated with dispositions in
`non-result-sites.tsv`): the 3 cold helpers (`lazy/continuation.rs:42`,
`lazy/table_continuation.rs:37`, `schema/continuation.rs:33`), 3 side-effect
inserts (`frozen.rs:133`, `default_fallback.rs:47`, `batch_file.rs:39`), the
published-selector API (`evaluated_part_selectors.rs:143`), and 2 frame-resume
closure sites (`confirmation/builtins/resume_frames.rs:353`,
`replace/once/resume_frames.rs:243`).

### 2.2 The crossing payload population (F8 settled on evidence)

Joining brief-600's slot-decode table against the producer-side Ok types
yields a decisive test: **the 8 producer-side Ok types that never appear as a
slot decode are exactly the 8 intermediate accumulator types**
(`SuspendedPreparedCallArguments`, `ReapRenderAccumulator`,
`SuspendedReapScopeCompletion`, `SuspendedGuardCompletion`,
`SuspendedReplaceCompletion`, `SuspendedReplacePartTargetsCompletion`,
`ResolvedArgumentAccumulator`, `HeldUpdateCurrent`). They are helper ordinary
results converted by finalizers/matches before crossing — the F2 class,
enumerated by join rather than by example (§3.3, `deferred-completion-
families.tsv`). Removing them, the actual crossing set is **35 rows**
(`crossing-payloads.tsv`) — every one **rum-runtime-owned or a Rust
primitive**. The stdlib families cross only runtime-owned types
(`Option<Expr>`, `Vec<Expr>`, `String`, `Expr`); brief-601's stdlib
"counterexamples" (`ExactStringCaseOutput`, lazy/schema freeze states) are
stored frame fields, confirming F8. **Consequence: the crate boundary does
NOT block a closed enum** — the director's gate-601 acceptance of
`Box<dyn ErasedSlotValue>` rested on a false premise; §4.1 re-decides on the
actual set.

### 2.3 Propagation edges, owner-qualified (F6 repaired)

Rebuilt with caller/callee identities joined directly on the AST function
`id` (the format `may-functions.json` uses; 1726/1731 ids match — the 5
misses are `<dynamic:*>` pseudo-ids): **619** type-changing may→may edges
(brief-601's 51 were free functions only). Classes: **A** 364 edges whose
caller is a frame's `resume` (the frame-body boundary itself — typed by the
frame's declared Input, not frameless propagation); **B** 224 edges in **44
module families** (the true helper hops — `propagation-edges-v2.tsv` carries
per-edge caller macro sites and frameless flags); **C** 31 host-surface
edges (out of scope by user ruling). Non-`Result` carriers: 391 caller-side,
29 callee-side. **10228** non-indirect call targets remain unresolved
(`propagation-unresolved.tsv`) — listed as unresolved, not silently dropped.
The dominant B-class carrier pairs are `Expr→Option<Expr>` / `Option<Expr>→Expr`
across ~15 modules (the lenient-accident family), `CompiledMatchOutcome→{Expr,
Option<Expr>, Vec<SymbolExprMap>, bool}`, and the traversal-outcome ladders.

### 2.4 Witnesses — each exercising its cited edge (F6's requirement)

`.temp/brief-603/witnesses.tsv` (also printed by the probe), all
**equal** (queued == resumed, suspension asserted by pause count, queue
drained):

```
prepared.double   List["a1","a1",7]   (2 pauses — Frame A, resuspend->Frame B, finalizer)
lazy.first        1                   lazy.first.double 1  (2 pauses — machine+bounds)
lazy.normal       List[1,2,3]         (LazyTable bounds helper)
guards.deferred   2   guards.deferred.double 2  (GuardOutcomeFinalizer chain)
reap.buckets      List[2,List[1,1]]   (2 pauses — bucket loop resuspension)
args.resolution.heads List[0,0]       (Heads option -> resolve_argument_exprs + once.rs:183 chain)
replace.levels    List[0,0]           (Option<CasesLevelSpec> producer, once.rs:259)
chained.except    List[]              (interpreted Prepare->Continue chain)
held.part         List[11,2,3]        (FinishHeldPartSelectorEvaluation, HeldUpdateCurrent)
string.guard      List["B"]           (CORRECTED: "b" :> "B" /; (InputString;True) — a real guard)
prime.thread      List[3,5]           (CORRECTED: Prime[{2,(InputString;3)}] — the threading edge)
replace.targets   List[1,9,3]         (replace-part target collection)
```

The two brief-601 witnesses F6 broke are replaced with shapes that reach the
named edge: the string guard now goes through `split_definition_rhs_guards`
(`strings/patterns.rs:597`) into `evaluate_string_rule_guard_result`
(`guarded.rs:14`), and the Prime witness now suspends inside
`session.evaluate(threaded)` (`number_theory.rs:289`), the only suspending
branch. These are **controls**: they certify each family's current
resuspension round-trip — the premise the migration must preserve and the
marker-corpus falsifier will re-assert.

### 2.5 Chained inserts (F4 complete)

A scan for insert-arm sites followed by another macro within 6 lines finds
**58 Prepare→Continue pairs** (`chained-inserts.tsv` content in §2 of
`.temp/brief-603/`): the interpreted matcher (6), compiled matcher
sequences/orderless/optional (~34), cases/delete/replace families (12), block
scoping (4), packages (1), replace/once setup (2). brief-601's classification
called only 3 of these "chained"; the rest were folded into `a-insert-return`.
All 58 are compositions whose adjacent types the chain builder (§3.2) checks.

## 3. The typed composition design

### 3.1 The boundary in one paragraph

Every frame names its `Input` and `Output` from a closed, sealed set; every
route by which a frame enters a queue or a completion reaches a frame binds
those types at compile time **at the construction site** — the ordinary arm
binds Input by a zero-sized witness of the matched result's Ok type (F7) and
Output by the return position (F1/F2's binding, now sound because §3.3
removes the helper-typed insertions); Route E and finalizers bind by checked
constructors (F3); chained inserts compose through a `then`-checked builder
(F4); helpers re-suspend only through a token whose frame type was already
bound (F2). Adjacency between queue neighbours then holds **by construction
through shared source-level types** — the producer's frame Output is bound to
the callee's return type at its own site, and the consumer's Input is bound
to the same type at the call site — and the one place types are erased (the
driver's queue) re-checks by exact variant, failing with a classified error
in every build, never a panic.

### 3.2 Signatures

```rust
// ---- rum-runtime/src/runtime/suspension/core.rs ----

/// The closed crossing-payload set (one variant per row of
/// .temp/brief-603/crossing-payloads.tsv). Adding a variant is the single
/// review point for a new state type.
#[doc(hidden)]
pub enum ContinuationState {
    OptionExpr(Option<Expr>),
    VecExpr(Vec<Expr>),
    MatchOutcome(CompiledMatchOutcome),
    // ... 32 more, one per crossing type
}

#[doc(hidden)]
pub enum FrameOutcome {
    Value(Expr),                 // the ONLY guest-visible arm
    State(ContinuationState),    // constructed/decoded only by the adapter
}

mod sealed { pub trait Sealed {} }

pub trait SlotValue: Sized + Send + Sync + 'static + sealed::Sealed {
    fn into_outcome(self) -> FrameOutcome;
    fn from_outcome(outcome: FrameOutcome) -> Result<Self, SlotTypeError>;
}
// Exactly one impl per crossing type (each maps to/from its own State
// variant) plus `impl SlotValue for Expr` (the only impl that touches
// Value). No blanket impl. `Sealed` is implemented for those types only,
// here — so no external or stdlib crate can implement SlotValue and pick an
// arm. The impl set is pinned by a unit test.

pub trait TypedFrame: FrameRootTrace + FrameAbandon + Send + Sync + 'static {
    type Input: SlotValue;
    type Output: SlotValue;
    fn resume_typed(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Self::Input, EvalError>,
    ) -> TypedAction<Self::Output>;
    fn output_boundary(&self) -> OutputBoundary { OutputBoundary::default() }
}

pub enum TypedAction<Out: SlotValue> {
    Complete(Result<Out, EvalError>),
    Evaluate { expr: Expr, next: ErasedNext<Out> },  // private field
    Finalize(Box<dyn SlotFinalizer<Out>>),
}

impl<Out: SlotValue> TypedAction<Out> {
    /// Route E: the successor replaces this frame's remaining computation,
    /// so it must consume the evaluated Expr and complete the SAME output.
    pub fn evaluate<F>(expr: Expr, next: F) -> Self
    where
        F: TypedFrame<Input = Expr, Output = Out>,
    { /* wraps next in an adapter; field stays opaque */ }
}

pub trait SlotFinalizer<Out: SlotValue>: Send + 'static {
    /// Completes the OWN frame's Output; runs after the driver drops the frame.
    fn finish(self: Box<Self>) -> Result<Out, EvalError>;
}

/// Zero-sized invariant witness of a Result's Ok type (F7/director ruling 4).
pub struct SlotWitness<I> { never: PhantomData<fn(I) -> I> }

impl<T> SlotWitness<T> {
    #[doc(hidden)]
    #[inline(always)]
    pub fn of<E>(result: &Result<T, E>) -> Self { Self { never: PhantomData } }
}

#[cold]
#[inline(never)]
pub fn suspend_typed<F, I>(
    session: &mut Session,
    _input: SlotWitness<I>,
    frame: F,
    request: RuntimeInputRequest,
) -> Result<F::Output, EvalError>
where
    F: TypedFrame<Input = I>,
{
    session.insert_typed_frame(frame);
    Err(EvalError::InputRequired { request })
}

macro_rules! resume_or_suspend {
    ($session:expr, $result:expr, ok($value:pat) => $continue:expr,
     suspend => $frame:expr $(,)?) => {{
        let __rum_result = $result;
        let __rum_input = $crate::kernel_api::SlotWitness::of(&__rum_result);
        match __rum_result {
            Ok($value) => $continue,
            Err($crate::kernel_api::EvalError::InputRequired { request }) =>
                return $crate::kernel_api::suspend_typed(
                    $session, __rum_input, $frame, request),
            Err(error) => Err(error),
        }
    }};
}

/// Chained inserts: adjacency checked pairwise; composed and returned ONCE.
pub struct FrameChain<Out: SlotValue> { frames: Vec<Arc<dyn SuspensionFrame>> } // private field
pub fn chain<F: TypedFrame>(first: F) -> FrameChain<F::Output>;
impl<Out: SlotValue> FrameChain<Out> {
    pub fn then<F: TypedFrame<Input = Out>>(self, next: F) -> FrameChain<F::Output>;
}
#[cold]
pub fn suspend_chain<Out: SlotValue>(
    session: &mut Session, chain: FrameChain<Out>, request: RuntimeInputRequest,
) -> Result<Out, EvalError>;

/// Helper re-suspension token: only a frame's own body can mint one.
pub struct ContinueToken<F: TypedFrame> { frame: Arc<F> }
impl<F: TypedFrame> ContinueToken<F> {
    #[doc(hidden)]
    pub fn of(frame: &Arc<F>) -> Self;
    /// Expression-form (no return binding): inserts this token's frame and
    /// evaluates to the InputRequired error, which the helper propagates as
    /// its ordinary error. Output was bound once, at the original insertion.
    #[cold]
    pub fn suspend<I>(
        session: &mut Session,
        _w: SlotWitness<I>,
        request: RuntimeInputRequest,
    ) -> EvalError
    where
        F: TypedFrame<Input = I>;
}
// Session helpers that may re-suspend take `token: &ContinueToken<F>` and
// match their inner results with an expression-form macro whose suspend arm
// calls token.suspend — the frame type (hence Output) is already bound at the
// original insertion; the witness re-checks Input at the re-insert.
```

The erased layer stays driver-private: `SuspensionFrame`, `FrameAction`,
`PureFrameFinalizer` become `pub(crate)` (removed from `kernel_api`); the
adapter is the only implementor boundary:

```rust
pub(crate) struct TypedFrameAdapter<F: TypedFrame> { inner: Arc<F> }
// implements SuspensionFrame by: decode (exact-variant, SlotTypeError ->
// classified EvalError), F::resume_typed(self.inner.clone(), ..), erase the
// action (Complete(Ok(t)) -> Complete(Ok(t.into_outcome())); Finalize ->
// ErasedFinalizer { inner }). Delegates FrameRootTrace, FrameAbandon (via
// inner Arc clone — no F: Clone, no uniqueness), and output_boundary.
```

### 3.3 The counterexamples resolved (required change 4)

**F2-1 prepared-call** (`eval_call_prepare.rs:231/:269/:401/:436`,
`resume_frames.rs:26-83`, `call.rs:301`). Frame B
(`ResumeSuspendedCallArgumentPreparationSuspensionFrame`) is a field-for-field
duplicate of Frame A that exists only because the loop helper returns the
intermediate `SuspendedPreparedCallArguments`. Resolution: Frame B is
**deleted**; the loop helper takes `&ContinueToken<F_A>` and re-suspends
through it; Frame A becomes `TypedFrame<Input = Expr, Output =
PreparedCallArguments>` (bound at `prepare_arguments_for_call_with_
unevaluated_restore`'s return position — the obligation type); the finalizer
becomes `SlotFinalizer<PreparedCallArguments>` whose `finish` is today's
`finish_suspended_prepared_call` minus the `Expr` encoding (plain
accumulator→struct materialization). Drop-before-finalizer is preserved
exactly: the state is cloned into the finalizer inside `resume_typed`, the
driver drops the adapter (and with it the frame's `Arc<F>` copy) before
`finish` runs, so `UnevaluatedRestoreAccumulator::into_vec`'s
`Arc::try_unwrap` uniqueness holds as today (`eval_call_prepare.rs:75`). The
ad-hoc decoder `prepared_call_arguments_from_continuation_expr` and the
`call.rs:322` `.expect` are deleted with the group.

**F2-2 lazy captures** (`surface/lazy_completion.rs:46-57`,
`basic.rs`, `take_drop/`). The synthetic-`Err` helper `capture` annotates
`Err::<Option<Expr>, _>` while its frame's obligation is the whole builtin's
`Expr` — the type annotation is a lie about the producer. Resolution: the
helper is **deleted**; the suspension match hoists to the real call site, and
because the macro evaluates `$result` once, the site is the `realize_*` call
itself: `resume_or_suspend!(session, realize_at_most(session, expr, 1),
ok(v) => v, suspend => FirstLazyCompletionFrame { .. })` inside `eval_first`
— Input binds to `Option<Expr>` (the realize chain's completion) from the
real result, Output to `Expr` from the builtin's return. The frame is an
honest converter. `lazy/table_continuation.rs:37` keeps a typed helper
generic over the obligation with the producer's Ok type named in its
signature (ruling 4's "carry the producer's type evidence in"); `schema/
continuation.rs:33` is genuinely `Result<Expr,_>`-shaped and needs only the
ordinary binding.

**F1/F4 step-plan soundness** follows: no frame is ever "made identity
`Expr→Expr`" — each step converts a complete producer/consumer group to its
real types (§6), so no step contradicts its own binding, and no consumer is
asked to straddle two protocols.

**F3's frames.** `DirectDefaultFallbackFrame` is `TypedFrame<Input =
Option<Expr>, Output = Option<Expr>>` and converts with its group (G-option).
Route E's two constructors use `TypedAction::evaluate` (both sites: the
sequence-check frame and the merge-sort callback consumer are
`Input = Expr`; their Output is the parent's by construction). The 3
side-effect inserts keep a declared-Output `insert_typed_frame<F:
TypedFrame>` — enumerated (3), adjacency checked at the erasure boundary;
`RetentionFrame<C>` gains a typed continuation twin (`type Output:
SlotValue` on the continuation trait) so its Output is per-continuation and
not blanket-declared.

**F4's relocations are per-site, not one rule.** The 7 `b-profiling-record`
sites whose records sit after the macro in the same statement flow move into
the suspend arm (identity/count/classification + balanced timing-stack depth
obligations, F9); `eval_call_prepare.rs:269`'s record wraps the whole IIFE
closure and already observes the suspension outcome — **no relocation**
(brief-602's correction adopted); the 2 frame-resume closure sites need none.

### 3.4 Why each bound is sound

- **Output** — `suspend_typed`'s return type is fully determined by `F`;
  placed in `return` position it unifies `F::Output` with the innermost
  enclosing fn/closure Ok type. Nominal types; no subtyping escape
  (`SlotValue` types are nominal or sealed-marker types; the witness is a
  ZST `PhantomData<fn(I) -> I>`, invariant in `I`). A wrong completion stops
  compiling at the insertion site — including in generics (the merge-sort
  frame's `Output = Expr` is concrete despite generic items) and in closures
  (`return` binds the closure; the two frame-resume closure sites' closures
  return the channel type, verified).
- **Input** — `SlotWitness::of(&__rum_result)` fixes `I` to the matched
  result's Ok type *before* the match moves it; `F: TypedFrame<Input = I>`
  then requires the frame to consume exactly what the callee's chain
  completes (the callee's own frame's Output is bound to that same type at
  its site). No frame is constructed on the Ok path (F7: the witness is
  zero-sized, non-borrowing); the added `let` binding is the only ordinary-
  path delta, gated per §5.
- **Route E** — the constructor's `F: TypedFrame<Input = Expr, Output = Out>`
  makes "next replaces the parent's remaining computation" a type fact; a
  wrong-output successor does not compile at the action construction.
- **Finalizers** — `SlotFinalizer<Out>` is parameterised by the owning
  frame's Output through the `TypedAction<Out>` it must inhabit; a finalizer
  for another type does not compile. It completes its OWN frame's output
  (removing the hand-paired finalizer→next-decoder convention).
- **Chains** — `then`'s `F: TypedFrame<Input = Out>` checks each adjacency;
  `suspend_chain`'s return position checks the last Output; the frames are
  pushed once, in order.
- **Tokens** — `ContinueToken` is minted only from `&Arc<F>` inside `F`'s own
  `resume_typed`; the re-inserted frame is the same `F`, so its Output (bound
  once, at the original insertion) cannot change, and the witness re-checks
  Input at the re-insert. A helper cannot mint a frame of its own — which is
  precisely the F2 invariant: **insertion exists only where the enclosing
  return type is the obligation.**

## 4. Sealing, slot population, trust boundary, ownership

### 4.1 Representation: closed enum, not `Box<dyn ErasedSlotValue>` (F8)

Decided on the actual crossing set (§2.2): every crossing type is
rum-runtime-owned or a Rust primitive, so **a closed enum is possible**, and
it dominates controlled erasure on the project's own standard (compiler- or
derive-enforced beats convention; cost does not count): exhaustive driver
matching instead of `Any` downcast; no per-crossing allocation; no
consuming-downcast protocol to design; the closed set is a type-level fact
(one variant per type) rather than a pinned impl list; and the extension
policy is physical — a new state type edits the enum, which is exactly the
review point. Erasure's only advantages (module privacy, no central
registry) are not correctness advantages; ~32 private types become
`pub(in crate::runtime)`-visible to the enum's module — organization churn,
disclosed per row, no crate-boundary crossing. The director's gate-601
acceptance of the box was premised on stdlib-owned crossing types; that
premise is false (§2.2), so the ruling is re-submitted rather than followed
(open question 1).

### 4.2 The trust boundary (F5, honestly stated)

Sealed `SlotValue` + the pub(crate) erased layer close the repository: no
in-tree or external code can construct `FrameOutcome::State` with a guest
value, map a state type through `Value`, implement the erased traits, or
insert a raw frame. What no sealed transport can forbid: an **arbitrary
external `Expr → Expr` frame** deliberately implementing a symbolic protocol
*inside* its own computation (e2-outline:49's limit). The certified domain:
in-tree frames plus external frames whose Input/Output are drawn from the
closed set and whose bodies treat `Expr` as data — the same domain E1/E2
certify behaviorally. This is stated, not unit-tested: a repository test
cannot prove properties of external implementations (F5's required honesty).

### 4.3 SlotTypeError — named variant, cleanup and propagation (F9)

```rust
#[error("internal continuation slot mismatch: expected {expected}, got {got}")]
ContinuationSlotMismatch { expected: &'static str, got: &'static str },
```

in `EvalError` (an ordinary uncatchable failure variant — not `Thrown`, so
no guest `Catch`/handler can consume it; it surfaces loudly like
`RecursionLimitExceeded`). Cleanup: the mismatched `FrameOutcome` is dropped
in place at the decode (releasing its roots — the payload's `Drop` is plain
Rust and its `Expr` ownership falls under the acknowledged destruction debt);
the remaining queue is abandoned through the normal abandonment path when the
request fails. Propagation: frame bodies' `Err(error) => …Complete(Err(error))`
arms pass it through unchanged. No `debug_assert!`, no `expect`, in any
build. Reachability: only a mis-bound insertion (a kernel bug), a bad chain
`then`, the seed/Route-E boundary, or a splice-point mismatch — all internal;
guest-authored values travel only as `Value`, which state slots reject **by
type, not by shape** — the lenient-decoder class becomes unrepresentable.

### 4.4 Transit, rooting, adapter ownership (F8/F9)

A `State` exists only inside one `drive_resuming_frames` iteration
(`core.rs:723-747`): produced by an adapter's decode, consumed by the next
adapter — never queued (queues hold frames), never in a loader/rollback
snapshot (`model/session.rs:84`), never abandoned (abandonment walks frames).
This corrects brief-601:598 — the driver *does* carry the outcome across the
loop iteration past the producer's drop, so the enum (like `Expr` today)
transits unrooted for exactly one step; the ownership consequence is the same
as today's in-band `Expr` transit, and the queued-root guarantee stays on the
frames. The adapter holds `Arc<F>` (director ruling F): no `F: Clone`, no
uniqueness assumption, drop-before-finalizer preserved (§3.3). `Send`/
`'static`: `SlotValue: Send + Sync + 'static` (payloads inhabit Arc'd
frames); `SlotFinalizer<Out>: Send + 'static` (it inhabits
`Box<dyn PureFrameFinalizer>` which is `Send + 'static`, `core.rs:96`).

### 4.5 Inventory consequences — predictions and their verification

Baseline (this run): ownership graph green, 20 passed, ledger 65 rows; rows
53-55 are the channel's `Unbounded`/Expr-debt rows. Predicted per step:
the enum and `FrameOutcome` add **new** `UnknownHostPayload` rows for their
`Expr`-owning storage (same Unbounded/Expr-debt/unresolved disposition as
rows 53-55 — **not** a `BoundedDestruction` assertion, Step-D ruling), and
new `PublicInput` rows for the new public constructors (`suspend_typed`'s
`frame`, `TypedAction::evaluate`'s `next`, the chain builder). Exact counts
are NOT claimed (F8); each implementation brief replays
`cargo test -p rum-stdlib --lib ownership_graph` + both feature builds'
`local_production_acceptance`, discloses every new/changed row, and STOPs on
any identity/status change to an existing row. The adapter (one generic
type) and any split frames enter `runtime/tests/frame_root_trace/`
`derived_inventory.rs` and the named-frame count exactly once, at the step
that introduces them. Retention fingerprints expire per standing duty.

## 5. The E2 boundary (ruling 2) and the gates (required change 7)

**E2 boundary, stated exactly.** G13 builds, and E2 keeps unchanged:
`FrameOutcome`/`ContinuationState`, sealed `SlotValue`, `TypedFrame`/
`TypedAction`/`SlotFinalizer`, the adapter, the witness/return-form macro,
the chain builder, `ContinueToken`, the falsifier, and the restructured
obligation-typed call sites. G13 additionally forces, as part of its own
migration, the propagation work it needs to function: the token restructures
(§3.3) and the SCC-complete family flips — these are G13's, not E2's (ruling
2). **E2 keeps only**: (a) the no-`Try` may-suspend result type that makes a
bare `?`/combinator hop of `InputRequired` a compile error — after G13,
every *frame* boundary is checked, but a frameless chain of ordinary
functions still propagates suspension unchecked, and that is exactly E2's
subject (the class-B edge table is its ready-made migration list); (b) the
result representation choice (transparent wrapper vs split) via its
authorized measured prototype — G13 touches `Result` only in the macro's
match arms, `suspend_typed`'s signature (two lines), and `SlotWitness::of`
(one impl), each enumerated for E2 so the prototype is unconstrained. The
`()`-carrier edges (ruling 3(5)): **handed to E2** — `drive_machine ->
Result<(), EvalError>` (`lazy/machine.rs:8-16`) is a frameless propagator
whose `()` never reaches a slot (verified: no `()` slot decode exists; its
caller inserts the realize frame), so G13's migration does not depend on
typing it.

**Gates.** Every implementation brief runs `cargo fmt --check`, `cargo test
--workspace`, both feature builds' oracle (all E1 descriptors +
`marker-corpus`) and lifecycle checks, and replays the inventories (§4.5).
Compile gates (brief-604, using the existing `compile_fail` doctest pattern
from `runtime/native_stack_guard.rs:19/:79`): wrong Input (frame inserted
where the result's Ok type differs), wrong Output, wrong Route-E output,
wrong finalizer output, chain `then` mismatch, `SlotValue` impl for a
foreign type (sealed), direct `FrameAction::Complete(State)` outside the
driver (pub(crate)); compile-PASS controls: a generic frame site, a closure
site, an external (stdlib) `Expr→Expr` frame via the public API. Oracle
gates: `corpus_positions` per descriptor with **each position substituted
each shape separately** (ruling 3(3)) plus the all-positions variant;
per-descriptor inclusion/exclusion with reasons (boundary entries excluded —
batch APIs non-resumable by ruling; no-guest-value-position descriptors
excluded; prompt-answer positions excluded — the seed is a String atom); an
explicit **no-`ContinuationSlotMismatch`** assertion in the oracle runner
(checked on every run including guest-error-handling descriptors — the
variant is uncatchable so it always surfaces; the assertion is independent
of `accept`'s equality path, `descriptors.rs:191`'s gap). Panic shapes stay
out of the corpus (director-side). Performance (ruling 5): any step touching
a hot evaluator function carries the full per-label microbench A/B
(`scripts/core-language-microbench.sh`, base vs candidate, same-tier
neighbours) with one-codegen-unit instruction slopes as supplement — the
steps so marked are brief-605 (driver flip — asserted cold but gated
anyway), 606 (macro flip at `try_dispatch_builtin_call`, `eval_set`,
`prepare_arguments…`), 613/614 (final deletions); profiling-record
obligations compare record identity, count, classification and balanced
stack depth — never elapsed values (`timing.rs:347`).

## 6. The step plan and the redrafted implementation briefs

Producer/consumer groups, each landing green; briefs **604-614** (602/603 are
taken). Order: falsifier before the first semantic flip; every flip is
SCC-complete over its crossing type so no consumer straddles protocols; the
legacy expression-form macro + in-band codecs are the enumerated temporary
path, deleted at 614.

| # | group (crossing types / frames) | content |
|---|---|---|
| 604 | scaffolding | all §3.2 types, adapter, macro forms, compile-fail/pass gate doctests, closed-set unit test. **No production flip; no enforcement claim** |
| 605 | channel flip | `resume`/`FrameAction`/`finish`/driver carry `FrameOutcome`; every impl body wraps/unwraps `Value` (codemod, compile errors = progress); legacy macro retained |
| 606 | G-option (runtime SCC) | `marker-corpus` falsifier (before-red = the 21 documented divergences) + six REAL frames + all runtime `Option<Expr>` producers/consumers (dispatch stage ×3, dispatch-plan split at PrepareArguments vs stage, default/up/sub/replace_all/once/repeated suspends, status family, `DirectDefaultFallbackFrame`) |
| 607 | G-prepared | §3.3 restructure: Frame B deleted, token, `SlotFinalizer<PreparedCallArguments>`, ad-hoc decoder + `call.rs:322` expect deleted, `EvaluateCall` head chain |
| 608 | G-option (stdlib tail) | lazy realize family + `First`/`Normal`/`Take` capture restructure + `LazyTableBoundsFrame` helper; `table_continuation` typed helper |
| 609 | G-matcher-compiled | `CompiledMatchOutcome`, `Vec<SymbolExprMap>`, the 12 choice states, `MatcherNode/Args`, ordered/optional/orderless/flat/key-value + their ~40 chains |
| 610 | G-matcher-interpreted | `InternalInterpreted*` payloads, `decode_prepared_interpreted_*` deleted, the 6 interpreted chains |
| 611 | G-traversal | cases/delete/position/count/replace outcomes, `CasesTraversal*`, `Option<CasesLevelSpec>`, replace-part targets + the `Suspended*Completion` token restructures (reap ×2, guards, replace children ×2, replace-part ×2, argument resolution) |
| 612 | G-block+accounting | block/package scoping states, `OptionValueScope`, `RetentionFrame` typed continuation, the 3 side-effect inserts' routes, loader/guest-loading frames |
| 613 | g-misc | `EvaluatedPartSelectors` + **published `OnceLock` API deleted** (callers migrate with it), `EvalCapturedOutcome`, `bool`/`String` (strings search/guarded/rules, message render), symbols/usize/`EvalError`/`Result` payloads, trace/test-report |
| 614 | deletion | legacy macro, `ContinuationValue`, `primitive.rs` lenient decoders, every `InternalContinuation*`/`InternalInterpreted*` head; zero-continuation-heads drift test; full microbench A/B + slopes |

Redrafted briefs (TEMPLATE shape; the director rewrites before dispatch —
the full seven sections live in these drafts):

**brief-604 — G13 scaffolding.** *Goal:* land §3.2's types, adapter, macro
arms, and the compile gate suite with **zero** production behaviour change.
*Scope (MAY):* `suspension/core.rs` (+ a `suspension/typed.rs` if size
demands), `kernel_api.rs` exports (`SlotValue`, `SlotWitness`,
`TypedFrame`, `TypedAction`, `SlotFinalizer`, `FrameChain`, `ContinueToken`,
`suspend_typed`, `suspend_chain`, `insert_typed_frame`; `ContinuationValue`/
`SuspensionFrame`/`FrameAction`/`PureFrameFinalizer` **not** re-exported —
their removal from `kernel_api` happens here for new code while the legacy
macro in-crate keeps using them), the compile-fail/pass doctests, the
closed-set unit test, standing-duty inventories (new rows disclosed per
§4.5). *Key steps:* enum + sealed trait + impls (macro-generated over the
variant list); adapter + erased-finalizer bridge; `#[cfg(test)]` round-trip
tests incl. a deliberate-mismatch `ContinuationSlotMismatch` check; doctest
gates. *Verification:* `cargo fmt --check`; `cargo test --workspace`;
inventories replayed; **no perf gate (nothing on any path yet — state it)**.
*Hard rules:* TEMPLATE standard; no `git`; no new `unsafe`; the report must
state "no enforcement claim" verbatim.

**brief-605 — channel flip (representation only).** *Goal:* the driver,
`FrameAction`, `resume`, `finish` carry `Result<FrameOutcome, EvalError>`;
every existing frame mechanically wraps/unwraps the `Value` arm; no `State`
is produced; markers still round-trip. *Scope:* `suspension/core.rs` driver
+ seed + Route-E wrap; every `SuspensionFrame`/`PureFrameFinalizer` impl
file (enumerated from brief-600's `ast-frames-raw.tsv`, 564+59);
`kernel_api.rs`. *Verification:* `cargo fmt --check`; `cargo test
--workspace`; both builds' oracle + lifecycle; inventories; a microbench A/B
spot-check on `symbol-ownvalue-read`/`downvalue-exact-hit` (cold-path claim
sanity). *Hard rules:* stop-loss keyed on compile-error progress; no
decoder semantics change; no marker deletion.

**brief-606 — falsifier + G-option.** *Goal:* `marker_corpus.rs` +
`corpus_positions` on the descriptors covering the six families (before-red:
the 21 divergences, pasted); convert the group table's frames to `TypedFrame`
with real Input/Output; falsifier green after. *Scope:* the §6-table 606 row's
files + `tests/suspension_oracle/` + closed frame lists (adapter enters once)
+ inventories. *Verification:* workspace; both builds; before-red/after-green
verbatim; **full microbench A/B + one-codegen-unit slopes** (macro flip in
`try_dispatch_builtin_call`); profiling-record obligations (identity/count/
classification/stack depth) for the 2 relocated dispatch records.

**briefs 607-613 — one per group.** *Goal/Scope/Key steps:* per the §6
table; each carries: the family's crossing types flip together (SCC-complete
— a leftover producer or consumer is a STOP, not a defer); the family's
`ContinuationValue` impls and ad-hoc decoders deleted; corpus positions
extended to the family's descriptors; the class-B edges touching the family
get their disposition executed (frame, restructure, or documented tail
normalization — from `propagation-edges-v2.tsv`); token restructures where
`deferred-completion-families.tsv` names them; inventories + closed lists.
*Verification:* workspace; both builds; falsifier extended and green; NO
instruction gate except where a hot function changed (607 touches
`prepare_arguments…` — gated; 608/609/611/612/613 state "cold, not gated"
with the reason). *Hard rules:* TEMPLATE; no panic shapes in the corpus; a
temporary adapter (if any) is named in the brief with the deleting step.

**brief-614 — deletion.** *Goal:* delete the legacy macro,
`ContinuationValue`, `suspension/primitive.rs`, and every continuation-head
reference; add the zero-continuation-heads drift test (fresh-scan pattern,
like the builtin-reference drift test); check `docs/reference` (expectation:
none documented). *Verification:* workspace; both builds; full microbench
A/B with same-tier neighbours + instruction slopes; raw rows persisted.

## 7. Open questions for the director

1. **Re-decide the representation** (§4.1): the closed enum reopens your
   gate-601 acceptance of `Box<dyn ErasedSlotValue>` because its
   crate-boundary premise is false (all 35 crossing types are
   rum-runtime-owned/primitives). Recommend the enum; if you keep the box,
   §3-§6 stand with `ContinuationState` replaced by the erased payload plus
   a consuming-downcast protocol (`into_any(self: Box<Self>)`).
2. **`ContinuationSlotMismatch` guest message**: loud uncatchable failure
   (recommended, §4.3) vs silent `TerminatedEvaluation`-style. The oracle
   asserts absence either way.
3. **Enum size/visibility churn**: ~32 module-private crossing types become
   visible to `suspension/`; acceptable, or prefer a `continuation_state/`
   module per family re-exporting into one enum module?
4. **The 29 `a-let-consumer`/`a-map-tail` sites** where `$result` is
   transformed before the macro: the witness binds the transformed type, so
   each site must move the macro to the raw callee result or the transform
   into the ok-arm (director verdict B's list). Assigned per family brief;
   confirm you want them audited brief-by-brief rather than in one upfront
   pass.
5. **606's stdlib boundary**: the lazy pair lags one brief (legacy in-band
   round-trip, verified compatible §6); alternatively fold 608 into 606 for
   a single larger step. Recommend keeping them split (smaller gates).
6. **`insert_typed_frame` for the 3 side-effect inserts** stays declared-
   Output (§3.3/F3): confirm the enumeration + boundary-check honesty is
   acceptable, or require the `SuspendToken` device for
   `DirectDefaultFallbackFrame` only (it is the one of the three with a
   non-trivial Output).
7. **brief count**: 11 briefs; 605 and 609 are the mechanical-risk steps
   (~623 bodies; ~100 matcher frames). Split points named in each if the
   stop-loss fires.

