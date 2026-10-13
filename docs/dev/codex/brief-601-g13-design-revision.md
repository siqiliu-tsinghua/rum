# brief-601 — design revision: G13, bind each frame to the type its insertion point returns

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

brief-600 inventoried G13 and proposed a repair: an out-of-band two-arm
channel (`Value(Expr)` for guest values, typed `State` for continuation
state) plus slot-typed frames bound at the insertion macro. The director
reproduced its 30 witnesses byte-identically (18 divergences across six
families, none panicking) and accepts its inventory and the channel. **The
binding rule it proposes does not reject the defect it was designed for**, and
three further points need a decision backed by evidence. This brief revises
the design; it does not repeat brief-600's verified inventory — read the
report (`docs/dev/codex/brief-600-g13-typed-continuation-state-design.md`
§2-§8) and extend its tables under `.temp/brief-600/`.

## Accepted from brief-600 (do not reopen)

- The out-of-band channel: guest values travel only as an `Expr` value arm;
  typed continuation state is a Rust value guest source cannot name. Sealed or
  renamed symbols are rejected (§3.4's evidence).
- A closed set of slot types, no blanket impl; `Expr` maps to the value arm.
- The class-wide falsifier lives in the oracle as a `marker-corpus` variant,
  without panic shapes; it is the single home of the witnesses.
- The final step deletes every continuation marker reference, the lenient
  decoders and `ContinuationValue`; no inert constants are kept.

## Correction 1 — the frame's output is bound to the wrong type

`resume_or_suspend!` (`crates/rum-runtime/src/runtime/suspension/core.rs:798-815`)
is an expression: its `InputRequired` arm inserts the frame and *evaluates to*
`Err(..)`. At every one of the five REAL sites, `$result` and `$continue` are
`Result<Expr, EvalError>` and the `Option` wrapping happens **after** the
macro:

- `scoping/callable.rs:59-75` — `resume_or_suspend!(…)` then
  `finish_function_application(outcome).map(Some)`;
- `kernel_data/rewrite_data/cases/collect.rs:441-448` — `resume_or_suspend!(…).map(Some)`;
- `eval_dispatch/mod.rs:206-239` — the macro, then
  `record_profile_builtin_timing`, `validate_builtin_memory_growth_slow`, then
  `result.map(Some)`; `suspend_threaded_call_dispatch` (`:160-170`) uses the
  insert arm;
- `definitions/part_assignment.rs:43-53` and
  `kernel_data/association_data.rs:747-763` — the macro, `?`, then
  `Ok(Some(value))`.

brief-600 §4.3 Layer 3 binds `T` from `$result`'s Ok type. That gives
`SlotFrame<Expr>`, under which every one of the five raw completions still
type-checks; §4.3's statement that they "no longer compile" contradicts its
own rule. A frame's completion is consumed as **the enclosing function's
return type** (it is what the caller's `$result` receives), so the
compile-time binding must be:

- the frame's **output** to the enclosing function's (or closure's) return
  type, and
- the frame's **input** to `$result`'s Ok type, decoded by the adapter before
  the frame body sees it (a typed frame with `Input` and `Output`, or an
  equivalent with the same guarantee).

One mechanism for the output: on suspension the macro **returns** from the
enclosing function through a typed helper — `return suspend_typed(session,
frame, request)` with `F: TypedFrame<Output = T>` and return type
`Result<T, EvalError>` — so `T` is inferred from the return position. Design
this or another mechanism with the same guarantee, and state why it is sound.
Then:

1. **Every insertion route.** The insert arm, the public
   `Session::insert_suspension_frame` (`suspension/core.rs:505`), the driver's direct pushes,
   `FrameAction::Evaluate { next }` and finalizers are bound the same way or
   removed. List each with its binding.
2. **Work after the macro on the suspension path.** An early return changes
   behaviour wherever code after the macro runs while the result is
   `Err(InputRequired)`. Classify every macro use (brief-600 counted 596
   insertion rows): (a) tail or return position; (b) the suspension result
   flows into further work. For each (b), list that work (profiling records,
   memory-growth validation, scope or state restoration, …), say where it
   moves — before the return, into the frame, into abandonment — and show with
   evidence that it stays correct. Each relocation is a behaviour obligation
   the implementation brief must test.
3. **Propagation without a frame.** A may-suspend call propagated by bare `?`
   (or `map`, `and_then`, a helper) across a change of Ok type delivers a
   completion the next frame does not expect. Inventory those edges with the
   Ok type on each side (brief-583's census tooling under `.temp/brief-583/`
   has the call edges). Each is a latent G13 instance or an existing lost
   continuation; give an executable value-divergence witness for any reachable
   one. Say which part this design closes and which E2's no-`Try` type closes.

## Correction 2 — one input and one output type per frame

Ruling on open question 2: **no per-chain slot enums.** A frame that decodes
different types at different stages (the call-dispatch frame) splits per
stage, so every frame has exactly one input and one output type the compiler
checks. List the frames this splits.

## Correction 3 — the representation of typed state

`Box<dyn ErasedSlotValue>` is a new type-erased payload that owns `Expr`. The
ownership discovery (`cargo test -p rum-stdlib --lib ownership_graph`) will
report it as a new `UnknownHostPayload` row — a STOP condition in every
brief's standing duty — and its destruction depth and frame-root tracing then
need a stated contract (see the disposition ledger the ownership line built).
Compare it with a **closed enum of slot states** (exhaustive, derivable
`FrameRootTrace` and destruction, no downcast), including:

- how slot types owned by `rum-stdlib` fit across the crate boundary;
- where state lives in transit — is a completion ever held past one driver
  step (abandonment, cancellation, reset, snapshot)? If so, how it is traced
  and dropped;
- the exact rows each choice adds to or changes in the ownership, retention
  and frame-root inventories.

Choose with evidence.

## Correction 4 — no panic on a type mismatch, in any build

A mismatch between a completion and its slot is a classified error returned
in **every** build — no `debug_assert!`, no `expect`. The oracle asserts that
it never occurs. If correction 1 makes a mismatch unreachable except through a
named route, say which.

## Rulings on brief-600's other open questions

- **Q3, falsifier breadth:** every E1 descriptor whose fixture has a guest
  value position that can be substituted with each corpus shape. Design the
  substitution as a descriptor-level mechanism, not hand-written copies;
  justify every exclusion. Under the out-of-band channel a wrong-arm
  completion is already a classified error on ordinary values; the corpus
  guards against in-band decoding coming back.
- **Q6** is decided by correction 1.
- **Q7:** granularity follows from the revised design; the channel flip is
  complete before any family converts.
- **Q8:** settle `FailureMessageFromTemplateSuspensionFrame` now, with a
  non-panicking value-divergence witness in the scratch test.

## Relation to E2 — re-derive

Under correction 1 the typed suspension helper sits at the function's return
position, which is where E2's may-suspend result type
(`.temp/brief-583/e2-outline.md`) would carry its suspension. State precisely
what G13 builds that E2 then keeps, replaces or changes (types, the macro, the
helpers, the propagation edges of correction 1.3), whether doing G13 first
creates rework, and the recommended sequence.

## What I want back

1. The insertion-route table and the macro-use classification
   (`.temp/brief-601/*.tsv`), with every relocation obligation.
2. The propagation-edge inventory, with witnesses for reachable ones.
3. The revised design: the typed frame, the binding mechanism and why it is
   sound, the per-stage splits, the representation choice and its inventory
   consequences, the mismatch error.
4. The E2 relation and sequence; the falsifier mechanism; the ripple; the
   instruction-gate plan — the macro change touches hot functions
   (`try_dispatch_builtin_call` and its neighbours), so name the labels and
   use the one-codegen-unit protocol.
5. Redrafted implementation briefs in `TEMPLATE.md` shape, one per gated step.
6. Open questions for the director.

## Context pointers

- brief-600 and its report; `.temp/brief-600/` (tables, scripts, `probe.rs`);
  `.temp/director/gate-600/witnesses.tsv` (the director's reproduction).
- `crates/rum-runtime/src/runtime/suspension/`; the five REAL sites above;
  `kernel_api`; `rum-derive`; `runtime/tests/frame_root_trace*`.
- The ownership discovery and its ledger
  (`crates/rum-stdlib/src/tests/ownership_graph/`), the retention inventories.
- `.temp/brief-583/` (census tooling, `e2-outline.md`),
  `.temp/brief-548/enforcement.md`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-601/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief601_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-601/probe.rs`.
- Everything else is read-only — including `.temp/brief-600/`, whose files
  you copy rather than edit.
- This worker has no OS-level write fence: these exceptions are the whole
  fence. Write nothing outside this repository.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-601/`.
- **Run no probe that panics or aborts.** A probe that would reach a panic is
  reported, not run.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief601_probe`
  (900 s); `cargo test -p rum-stdlib --lib ownership_graph` (900 s) if you
  need its current output; `python3` scripts (300 s each). File reads,
  `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.
- Checkpoint to `.temp/brief-601/PROGRESS.md` after each section with the
  exact next action.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. Insertion routes and macro-use classification, with relocations.
3. Propagation edges and witnesses.
4. The revised design and the representation choice.
5. E2 relation, falsifier, ripple, gate plan.
6. The redrafted implementation briefs.
7. Open questions for the director.

---
# brief-601 report — G13 design revision: bind each frame to the type its insertion point returns

## 1. What changed

Design only; no production source change. Scratch under `.temp/brief-601/`:
`ast-macro-positions.tsv` + `ast-macro-raw.tsv` + `ast-call-edges.tsv` (a new
syn-based census from the declared scratch test), `macro-use-classification.tsv`
(the 598-site classification with per-site verdicts), `insertion-routes.tsv`
(Correction 1.1), `propagation-edges.tsv` + `propagation-spot-witnesses.tsv`
(Correction 1.3), `q8-witnesses.tsv` (Q8), `classify_macro_uses.py`,
`finalize_macro_table.py`, `propagation_edges.py`, `probe.rs`, `PROGRESS.md`.
The one scratch integration test `crates/rum-stdlib/tests/zz_brief601_probe.rs`
was declared, executed green (3 tests: `ast_classification`,
`q8_failure_template_witnesses`, `propagation_spot_witnesses`), copied to
`.temp/brief-601/probe.rs`, and **deleted** before this report was appended.
brief-600's tables were read, never edited. No `git` command was run. No probe
that panics was run (the 2-arg `Some` shapes stay director-side, per brief-600
§3.3). Commands run through `scripts/run-bounded.py`, serial: the probe test
(900 s, several iterations), `cargo test -p rum-stdlib --lib ownership_graph`
(900 s; green, 20 passed; inventories: UnknownHostPayload **65** rows
[6 storage edges + 59 public input sites], UnknownConstruct **7680**,
self-reaching **59**), and `python3` scripts (300 s each).

**Q8 is settled by execution and changes the defect count: five REAL frames
become SIX.** `FailureMessageFromTemplateSuspensionFrame` completes
`finish_failure_message_from_template` raw over an `Option<Expr>` slot, and a
guest Failure **MessageTemplate** is that raw value whenever the template is
neither a `String` nor a `MessageName` expression
(`status.rs:268-270 return Ok(template)`). Witness (all non-panicking,
`.temp/brief-601/q8-witnesses.tsv`, 10 observations):

```
Failure["t",<|"MessageTemplate"->System`InternalContinuationNone[],"MessageParameters":>(InputString["w"];{})|>]["Message"]
  queued:  value=InternalContinuationNone[]   resumed: value=Failure["t",Association[...]]["Message"]  FALSE
Failure["t",<|"MessageTemplate"->System`InternalContinuationSome[5],...|>]["Message"]
  queued:  value=InternalContinuationSome[5]  resumed: value=5                            FALSE
Failure["t",<|"MessageTemplate"->System`InternalContinuationSome[],...|>]["Message"]
  queued:  value=InternalContinuationSome[]   resumed: value=Failure[...]["Message"]       FALSE
```

3 divergences (`none`, `some`, `empty_some`), 7 equal controls (`ordinary`,
`other_type`, all five `styled.*` — the styled frame wraps raw and never
decodes, which my source reading predicted and the probe confirms). Mechanism:
the raw marker passes `TryApplyFunctionSuspensionFrame` (identity) and
`RunCallDispatchStageSuspensionFrame` (raw) into
`Option::<Expr>::from_continuation_expr` at the dispatch plan
(`call.rs:328`): `None` decodes as "stage declined" and dispatch continues to
later stages (the unevaluated call is rebuilt), `Some[5]` unwraps to `5`. The
divergence totals across the class become **21** (brief-600's 18 + these 3).

## 2. Insertion routes and macro-use classification, with relocations

### 2.1 Insertion routes (Correction 1.1)

`insertion-routes.tsv` (full table). Summary — every route that can put a frame
into a queue or feed a completion, and its revised binding:

| route | sites | revised binding |
|---|---|---|
| macro ordinary arm | 346 expr-position | `return suspend_typed(session, $frame, request)`, `F: TypedFrame`, `F::Output` unified with the innermost enclosing fn/closure return Ok type; compile-checked at the return position |
| macro insert arm → return | 240 stmt-position (237 terminal `return Err(InputRequired{..})` incl. chained inserts, 3 arm-tail `Err(error)`) + 6 conditional `matches!` sites | `return suspend_insert(session, $frame, request)` — same Output binding |
| macro insert arm, non-returning | 3 (`frozen.rs:133`, `batch_file.rs:39`, `default_fallback.rs:47`) | retained non-returning `Session::insert_suspension_frame<F: TypedFrame>`; Output declared, pinned by the identity/cleanup-frame closed list |
| cold helper insert (error parameter) | 3 stdlib (`lazy/continuation.rs:42`, `lazy/table_continuation.rs:37`, `schema/continuation.rs:33`) | helper generic over `T`: `fn suspend_X<T>(..., error) -> Result<T, EvalError>` with the frame `TypedFrame<Output = T>`; each caller's return position binds T |
| public `insert_suspension_frame` | core.rs:505; production only via macros; direct callers are 3 runtime test files | retyped `F: TypedFrame` (adapter-wraps); test frames convert |
| driver Route E | 2 constructors (`pattern_matching/core/resume_frames.rs:785`, `resumable_merge_sort.rs:340`) | `FrameAction::evaluate<F: TypedFrame<Input = Expr>>` — constructor bound |
| finalizers | 59 impls | `SlotFinalizer<Out>` completes its **own** frame's Output (a deferred completion); `FrameAction::finalize<F: TypedFrame, S: SlotFinalizer<F::Output>>` constructor bound |
| install / restore_nested / append | core.rs:239-264, 770-788 | unchanged (queues move wholesale; pairings fixed at insertion) |
| seeding | core.rs:607 | `FrameOutcome::Value(Expr::string(input))`; front frame Input = Expr by construction |
| request boundary | core.rs:723-734 | last adapter must complete `Value`; a `State` at the boundary is a classified internal error |

The finalizer typing deserves emphasis: today a finalizer's output type is a
manual convention against the *next* frame's decode
(`FinishPreparedCallArgumentsFinalizer` encodes `PreparedCallArguments` because
the dispatch-plan frame decodes it — `eval_call_prepare/resume_frames.rs:75-83`
vs `call.rs:301-303`). Under the revision the finalizer completes **its own
frame's** Output, and the constructor bound checks it where the pairing is
known.

### 2.2 Macro-use classification (Correction 1.2)

Method: a syn-based positional walker (the probe's `ast_classification`)
classifies every `resume_or_suspend!` site by what its value flows into on the
`Err(InputRequired)` path; raw coverage is exact (**346 expression-position +
252 statement-position = 598 distinct sites, 0 unclassified**, reconciled
against a raw macro scan; brief-600's 596 insertion rows join onto these — 4
macro rows bind no named frame). Verdicts
(`macro-use-classification.tsv`, 599 rows):

| verdict | sites | meaning |
|---|---|---|
| a-insert-return | 233 | insert (incl. chained inserts, verified transitively) then `return Err(InputRequired{..})` unchanged |
| a-return | 168 | `return macro!(...)` — the macro value is the returned error |
| a-tail | 90 | function tail expression, returned unchanged |
| a-try | 52 | immediately `?`-propagated (incl. `list_plumbing.rs:67`, assignment RHS) |
| a-let-consumer | 28 | `let x = macro!..` then `x?`/passthrough-match/Ok-path map |
| a-conditional-insert | 6 | `matches!`-gated insert; arm value or `return outcome` unchanged |
| a-insert-armtail | 3 | insert; match-arm tail returns the caught `error` unchanged |
| a-helper-insert | 3 | stdlib cold helpers; tail returns the error (schema's via `finish_abort`'s `SchemaAbort::Eval(e) => Err(e)`, `schema/format.rs:202`) |
| a-chained-insert | 3 | `pattern_matching_interpreted.rs:92,176,261` — insert chains terminating in the plain error return |
| a-side-effect-insert | 2 | `()->()` cold helpers (`defer_public_file_rollback`, default-fallback `suspend`); no suspension-path work |
| a-closure-tail | 2 | macro is a closure body tail returned from the closure call |
| a-map-tail | 1 | `finish_cases_rule_match` `collect.rs:442` `.map(Some)` in fn tail (map is identity on Err) — REAL frame #2's site |
| **b-profiling-record** | **7** | profiling side effects run after the macro **on the suspension path** |
| **b-side-effect-insert** | **1** | `frozen.rs:133`: unconditional insert, then work |

**589 of 598 sites are early-return-equivalent**: on the suspension path the
error reaches the function's return unchanged and nothing else runs. The
dominant `let`-family is the documented twin — `match outcome { Err(InputRequired) => Err(InputRequired), outcome => self.finish_X(outcome) }`
(~50 sites) — whose finish work runs only on non-InputRequired outcomes and is
already duplicated in the frame body.

**The 8 (b) sites, their work, and where it moves:**

1. `eval_dispatch/mod.rs:207` and `:219` (`try_dispatch_builtin_call`) —
   `record_profile_builtin_timing(profile_builtin_start.flatten(), result.is_ok())`
   runs after the macro on the suspension path. This is not just fidelity: the
   record **pops the `BUILTIN_TIMING_STACK`** token
   (`profiling/timing.rs:342-352`); an early return that skips it desynchronizes
   the profiling stack when profiling is on. **Moves: into the suspend arm
   before the return**, recording the now-known not-ok outcome — identical
   record content, identical stack effect. Obligation: with
   `RUM_PROFILE_*` enabled, a suspending builtin's timing row and stack depth
   are byte-identical before/after.
2. `:219` also passes the Err to `validate_builtin_memory_growth_slow` — a
   no-op on Err (`eval_dispatch/mod.rs:261-263` early-returns); **no
   relocation**, stated.
3. `definitions/definitions.rs` (`eval_set`, the rhs macro) —
   `record_profile_runtime_path_timing(runtime_profile_suspend, …(&rhs_result)); rhs_result?`.
   **Moves: into the suspend arm** (same record, not-ok outcome). Obligation:
   `set-ownvalue`-shaped probe with profiling on.
4. `eval_call_prepare.rs` (`prepare_arguments_for_call_with_unevaluated_restore`)
   — same shape (`…(&result); result`). **Moves into the suspend arm.**
5. `kernel_data/control_data/branching_loops.rs` `resume_while_loop` (2 macro
   sites) and `resume_compound_expression_from_index` — same shape
   (`…(&outcome); outcome`). **Moves into the suspend arm.**
6. `accounting_mode/frozen.rs:133` — unconditional `RetentionFrame` insert,
   then receipt transfer, memory-limit enforcement, and a conditional
   `abandon_pending_suspension`. **No relocation**: this is one of the 3
   non-returning inserts; the work stays after the insert, and the order
   (insert before transfer) is load-bearing. Obligation: the retention S-owner
   checks (memory-limited tests) stay green.
7. (`batch_file.rs:39`, `default_fallback.rs:47` are (a)-trivial: `()->()`
   side-effect inserts with no suspension-path work; they need the
   non-returning route, nothing more.)

Two things the classification adds beyond the count: (i) the ordinary arm's
`$continue`-then-postcode structure means each converted frame's completion
must absorb the post-macro Ok-path transformation (`.map(Some)` moves **into**
the frame body — that is precisely what makes the five/six REAL completions
stop compiling); (ii) no macro site anywhere transforms the
`Err(InputRequired)` value itself — the error is passed strictly unchanged,
so the return-through-helper rewrite cannot change any error payload.

## 3. Propagation edges and witnesses (Correction 1.3)

`propagation-edges.tsv`: **51 edges** where a may-suspend callee (brief-583's
`may-functions.json`, 1731 ids) returns an Ok type different from its caller's,
resolved source-side from a fresh call census
(`ast-call-edges.tsv`); **30 of the 51 callers have no macro of their own**
(frameless). Type pairs: `Expr→Option<Expr>` (14), `Vec<Expr>→Expr` (8),
`Expr→Vec<Expr>` (8), `String→Expr` / `Expr→String` (6+6),
`Option<Expr>→Expr` (2), `bool`-returning guard evaluation reached from
`Vec<Expr>`/`String` callers (4), `()→Option<Expr>`,
`Option<Expr>→()`, `Expr→()` (1 each).

Family verdicts (source-read; spot witnesses executed,
`propagation-spot-witnesses.tsv`, **all three EQUAL**):

- **String rule guards** (`guarded.rs:14 evaluate_string_rule_guard_result ->
  Result<bool, EvalError>` called from `resume_collect_string_cases_from_candidate`
  and the replace siblings): a real frameless Ok-type change that runs guest
  code (definition rhs guards). Spot witness `StringCases["aba","b":>(…InputString…)]`:
  queued = resumed = `List["1"]`. Protected today because the guard frames
  re-encode and the caller is itself a frame's resume helper (intra-frame
  continuation — the frame's Input is the guard result's type).
- **Lazy realization** (`lazy.rs realize/realize_all/realize_at_most ->
  Option<Expr>`, called from `eval_normal`, `eval_first`, `eval_take_lazy_list`,
  `eval_differences`, …): the published-Option protocol
  (`LazyRealizeSuspensionFrame` encodes `Some(List[...])`; consumers decode
  `Option<Expr>` at `lazy_completion.rs:33`). Spot witness
  `First[LazyTable[i,{i,(InputString["lf"];1),3}]]`: queued = resumed = `1`.
- **Number theory** (`eval_prime*` -> `try_twin…` bool/Option family): the
  conversion is duplicated in the resume twins. Spot witness
  `Prime[4+(ToExpression[InputString["nt"]];0)]`: queued = resumed = `7`.
- **Association/structure resume helpers** (`delete_missing`, `compute_inner`,
  `differences_from_order`…): intra-frame continuations and/or guarded by the
  G-family frames; E1 certifies the constructs.

**No reachable edge diverges today** — every family is protected either by a
re-encoding producer (brief-600's verified-slot-edges) or by E1-certified
twins, and the three executed spot witnesses are equal controls. The G13
hazard on these edges is *latent*: it fires only when a completion crosses a
frame boundary whose slot was hand-paired to a different Ok type — exactly
what the typed channel checks. **What this design closes**: the frame-boundary
pairings (exact-type decode at every hop; mis-pairing = classified error, and
the Output side is compile-checked). **What it leaves open**: the *frameless*
`?`/combinator hops themselves — nothing today verifies that a function which
propagates `InputRequired` by bare `?` still has real work left to do or not;
that is E2's no-`Try` type (a bare `?` on a may-suspend result stops
compiling), and my inventory is E2's ready-made edge list. Reachable-instance
witnesses: any future divergence on these edges is caught by the
`marker-corpus` falsifier (§5) — the corpus substitution applied to the
descriptor covering the family.

## 4. The revised design and the representation choice

### 4.1 The typed frame and the binding mechanism

```rust
// suspension/core.rs (#[doc(hidden)], exported via kernel_api)
pub enum FrameOutcome {
    Value(Expr),                        // guest-visible values, Route E, the seed
    State(Box<dyn ErasedSlotValue>),    // typed continuation state, adapter-only
}
pub trait ErasedSlotValue: Send + Sync + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn type_name(&self) -> &'static str;
}

pub trait SlotValue: Sized + Send + Sync + 'static {
    fn into_outcome(self) -> FrameOutcome;
    fn from_outcome(outcome: FrameOutcome) -> Result<Self, SlotTypeError>;
}
// closed impl set (no blanket): Expr <-> Value; every other slot type <-> State
// by exact-type downcast. Pinned by a unit test; adding a slot type is a review
// point by construction.

pub trait TypedFrame: SuspensionFrame {
    type Input: SlotValue;
    type Output: SlotValue;
    fn resume_typed(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Self::Input, EvalError>,
    ) -> TypedAction<Self::Output>;
}

pub enum TypedAction<Out: SlotValue> {
    Complete(Result<Out, EvalError>),
    Evaluate { expr: Expr, next: Box<dyn SuspensionFrame> }, // next: TypedFrame<Input = Expr>
    Finalize(Box<dyn SlotFinalizer<Out>>),                   // completes THIS frame's Output
}
pub trait SlotFinalizer<Out: SlotValue> { fn finish(self: Box<Self>) -> Result<Out, EvalError>; }
```

`SuspensionFrame::resume` keeps its shape but carries
`Result<FrameOutcome, EvalError>`; `FrameAction` and `PureFrameFinalizer`
flip accordingly (brief-600's Layer 1, unchanged). The adapter
(`TypedFrameAdapter<F>` wrapping F, delegating `FrameRootTrace`/`FrameAbandon`,
extracting via `Arc::unwrap_or_clone` — no `try_unwrap` uniqueness demand on
suspension frames, which share Arcs legitimately) is the **only** code that
constructs or decodes `State`.

**The binding mechanism (Correction 1).** The macro's suspend arm returns
through a typed helper:

```rust
#[cold]
#[inline(never)]
pub fn suspend_typed<F: TypedFrame>(
    session: &mut Session, frame: F, request: RuntimeInputRequest,
) -> Result<F::Output, EvalError> {
    session.insert_suspension_frame(TypedFrameAdapter::wrap(frame));
    Err(EvalError::InputRequired { request })
}

macro_rules! resume_or_suspend {
    ($session:expr, $result:expr, ok($value:pat) => $continue:expr,
     suspend => $frame:expr $(,)?) => {
        match $result {
            Ok($value) => $continue,
            Err($crate::kernel_api::EvalError::InputRequired { request }) =>
                return $crate::kernel_api::suspend_typed($session, $frame, request),
            Err(error) => Err(error),
        }
    };
    (insert $session:expr, $frame:expr, request: $request:expr $(,)?) => {
        return $crate::kernel_api::suspend_typed($session, $frame, $request)
    };
}
```

**Why this is sound.** (i) `suspend_typed`'s return type is fully determined
by `F` (`Result<F::Output, EvalError>`); placing it in `return` position
forces `F::Output` to equal the innermost enclosing function's (or closure's)
Ok type — nominal types, no subtyping escape; a frame completing a different
type **does not compile at its insertion site**. (ii) The closure case is
exactly right: `return` exits the closure, and the frame replaces the
remainder of *the closure*, whose return type is what the closure's caller
receives — the 2 closure-tail sites in the classification confirm the shape.
(iii) The frame's completion is *consumed as the enclosing function's return
value* (it is what the caller of the insertion function receives on resume),
which is why the binding must be the return type, not `$result`'s Ok type —
brief-600 §4.3's error. Under this binding the six REAL completions stop
compiling: e.g. `TryApplyFunctionSuspensionFrame` gets `Output =
Option<Expr>` (`try_apply_function` returns `Result<Option<Expr>, _>`,
`callable.rs:5-9`), so its body must complete
`finish_function_application(outcome).map(Some)` — today's raw
`FrameAction::complete(resumed)` at `callable.rs:151` is a type error.

**Input is bound by declaration and checked at runtime, not at the macro.**
The macro cannot name `$result`'s Ok type without either an ordinary-path
value-level witness (impossible on the Err arm, where no `I`-typed value
exists) or constructing `$frame` outside the suspend arm (an ordinary-path
cost — several arms build frames with `args.clone()`; hard-line violation).
The adapter's `SlotValue::from_outcome` is therefore an **exact-type**
decode: correct pairing succeeds, mis-pairing is `SlotTypeError` — a
classified internal error in every build (Correction 4), never a panic, never
a lenient misread. The runtime check is reachable only through named routes:
(a) a frame whose declared `Input` is wrong (a kernel bug the exact-type check
converts from silent misbehaviour into a loud classified error — the entire
G13 class today misreads instead); (b) the driver seed and Route E
(constructor-bound to `Input = Expr`); (c) a frameless propagation chain
whose first edge's Ok type differs from the catching frame's declared Input
(the §3 inventory — each is either intra-frame or protected, and the corpus
falsifier covers the descriptor surface). No guest input can reach a
`SlotTypeError`: guest-authored values travel only as `FrameOutcome::Value`,
and a `Value` delivered to a state slot is rejected **by type**, not by shape
— the lenient-decoder class is unrepresentable.

**Insert-arm and helper variants** (§2.1 table): `suspend_insert` for the
insert-then-return families (the request is in scope at every one of the 246
sites — verified by the classification's follower analysis); the public
`insert_suspension_frame<F: TypedFrame>` kept as the single non-returning
primitive for the 3 side-effect inserts; the 3 stdlib cold helpers
genericized over `T` so the caller's return position binds the frame.

### 4.2 Per-stage splits (Correction 2)

Exactly **one** frame family decodes the channel outcome with more than one
type (scripted over brief-600's decode sites, subject = the outcome variable):
`resume_suspended_call_dispatch_frame` — `PreparedCallArguments` at the
PrepareArguments stage vs `Option<Expr>` at every other stage
(`call.rs:301-337`). Its two consumer frames split per stage:

- `ResumeSuspendedCallWithDispatchPlanAtPrepareStageSuspensionFrame` —
  `Input = PreparedCallArguments`, `Output = Expr`;
- `ResumeSuspendedCallWithDispatchPlanAtStageSuspensionFrame` —
  `Input = Option<Expr>`, `Output = Expr`;
- and the same split for `ResumeCallWithDispatchPlanSuspensionFrame`
  (same helper, `call.rs:409`/`:449`).

`stage_index` stops selecting a decoder; the insertion site selects the frame
type. `RunCallDispatchStageSuspensionFrame` needs **no** split and **no**
per-chain slot enum: all three of its constructors sit in functions returning
`Result<Option<Expr>, _>` and all three receive `Result<Expr, _>` — one Input
(`Expr`), one Output (`Option<Expr>`) — brief-600's `CallStageOutcome` device
is unnecessary and is withdrawn. Every other frame already has exactly one
outcome decode (the 70+ multi-decode rows in brief-600's table are *nested
frame-field* decodes, which become plain Rust destructuring).

### 4.3 The representation of typed state (Correction 3)

**Chosen: `Box<dyn ErasedSlotValue>` (exact-type downcast).** The comparison:

- **The closed enum is impossible across the crate boundary.** The
  channel-crossing payload set is ~40+ distinct types spanning both crates:
  rum-runtime's (`PreparedCallArguments`, `CompiledMatchOutcome`,
  `CasesTraversalConfig`/`Resolution`, `ReplaceOutcome`, `DeleteCasesOutcome`,
  `Vec<ReplacePartTarget>`, `EvaluatedPartSelectors`, the sequence/orderless/
  optional choice states, `BlockSymbolState`, `EvalCapturedOutcome`, guards,
  `Vec<SymbolExprMap>`, …) **and rum-stdlib's** (`ExactStringCaseOutput`
  `strings/patterns.rs:161`, the lazy/schema freeze states, the association
  suspended states), plus a *generic* frame
  (`SchemaValidationSuspensionFrame<Vec<usize>>`). A rum-runtime enum cannot
  name stdlib types (dependency direction), and pushing the enum below both
  crates would leak kernel internals into rum-core and still not name
  crate-private payload types. Derive support is not the obstacle
  (`FrameRootTrace` derives on enums — `EvaluatedPartSelectorStorage`,
  `evaluated_part_selectors.rs:14-18`); the crate boundary is.
- **Ownership consequences.** `Box<dyn ErasedSlotValue>` adds **one**
  `UnknownHostPayload` storage-edge row to the ownership discovery (65 → 66
  rows at the current tree — counts above). Its honest disposition is the
  same as the three rows the channel already has —
  `SuspendedEvaluation.frames` (core.rs:21), `FrameAction.Evaluate.next_frame`
  (core.rs:106), `FrameAction.Finalize.0` (core.rs:108), all
  `Unbounded`, owed to the `Expr` debt, external implementations unresolved
  (`host_payload_ledger.tsv` rows 53-55): the payloads own `Expr`, and a
  `BoundedDestruction` supertrait declaration would assert what the project
  records as false (the Step D ruling — a declaration that is not true is not
  made acceptable by being cheap). It is not a new hazard class: the erased
  channel already exists (`Arc<dyn SuspensionFrame>`), and the new box is
  *less* persistent than the frames it feeds.
- **Where state lives in transit.** A `State` payload exists only inside one
  `drive_resuming_frames` loop iteration: produced by an adapter, consumed by
  the next frame's adapter. It is never queued (queues hold frames), never
  snapshotted (transient execution state is intentionally absent from
  loader/rollback snapshots, `model/session.rs:84`), never abandoned
  (abandonment walks frames, not outcomes; a pending outcome is always
  consumed before the driver can re-enter). Destruction and frame-root
  tracing therefore stay where they are today: on the frames.
- **The closed-set property survives without the enum**: `SlotValue` has a
  closed impl set (no blanket; `Expr` ↔ `Value` only), pinned by a unit test
  — the same single review point an enum would give, and the frame bodies
  cannot touch `FrameOutcome` at all (only the adapter constructs/decodes
  `State`), so there is no way to smuggle an arbitrary type into the channel
  even though the box itself is open.

### 4.4 No panic on mismatch (Correction 4)

`SlotTypeError { expected, got }` is converted to a classified `EvalError`
internal error (existing internal-error channel) at the adapter, in **every**
build — no `debug_assert!`, no `expect`, no `panic!` anywhere in the decode
path. The oracle asserts it never occurs (the corpus variant's ordinary-value
controls plus the E1 equality checks — a classified internal error fails
either). Reachability is confined to the named routes in §4.1: mis-declared
`Input`, seed/Route E (constructor-bound), frameless chain mis-wiring. The
existing panicking release invariants brief-600 listed
(`CompiledMatchOutcome` non-List, `EvaluatedPartSelectors`, prepared-args
wrappers, binding maps) are **deleted with their decoders** during the family
conversions — their payload types become plain Rust struct moves, and the
`call.rs:322` `.expect` disappears with the prepare-stage split.

## 5. E2 relation, falsifier, ripple, gate plan

**E2 relation (re-derived under Correction 1).** `suspend_typed` sits exactly
where E2's may-suspend result type (`.temp/brief-583/e2-outline.md`) would
carry its suspension — the function's return position. What G13 builds and E2
**keeps**: `FrameOutcome`, `ErasedSlotValue`, `SlotValue`, `TypedFrame`,
`TypedAction`, `SlotFinalizer`, the adapter, the driver flip, the per-frame
Input/Output discipline, the corpus falsifier, and the relocation of
post-macro work into frames. What E2 **changes**: the `Result<T, EvalError>`
plumbing around the frame — the `Err(InputRequired)` arm of the macro, the
bare-`?` frameless hops (my 51-edge inventory is E2's migration list), and
`suspend_typed`'s return type (its body is two lines; the frame-side bound is
unchanged). What E2 **replaces**: nothing structural — the return-based macro
is *closer* to E2's shape than brief-600's expression-based macro was, so the
correction **reduces** E2 rework rather than creating it. Sequence stands:
**G13 first, then E2's measured prototype** (the outline's representation
choice is still open and untouched here). The `marker-corpus` falsifier and
E1 run throughout.

**The falsifier mechanism (Q3 ruling).** A descriptor-level corpus dimension
in the oracle, not hand-written fixtures: `Descriptor` grows
`corpus_positions: &'static [&'static str]` — placeholder tokens the factory
embeds at guest-value positions in `f.source` (factories already build
sources via `format!`). A new `suspension_oracle/marker_corpus.rs` runner
takes each corpus-enabled descriptor, substitutes each corpus shape for each
placeholder, and runs `accept` with equality-only checks (`Checks {
transcript: true, equality: true, expectations: false, queue: true }` — the
expected values belong to the ordinary shape). Corpus: the six guest-writable
shapes `InternalContinuationNone[]`, `InternalContinuationSome[5]`,
`InternalContinuationSome[]`, `InternalContinuationVec[1,2]`, `List[]`, plus
an ordinary-value control; the 2-arg `Some` panic shapes stay out (a red
oracle must not abort). **Exclusions, each with its reason**: (i) Boundary
entries — the host batch APIs are non-resumable by user ruling; (ii)
descriptors with no callback/body guest-value position (pure scope, loader
and structural frames whose slots are kernel-internal state); (iii) positions
that are prompt answers (the seed is always a String atom). Every included
descriptor is red today on at least one shape for the six REAL frames
(21 divergences, §1) and must be green after the conversion — and *stays*
green permanently: once markers cannot be decoded, the corpus shapes are
ordinary values that must pass through untouched, so the falsifier is the
standing net against any in-band decoder coming back.

**Ripple.** `suspension/core.rs` (channel, traits, adapter, driver, both
macro arms, helpers); `kernel_api.rs` exports (+`SlotValue`, `TypedFrame`,
`TypedAction`, `SlotFinalizer`, `FrameOutcome`, `SlotTypeError`,
`suspend_typed`, `suspend_insert`; `ContinuationValue` removed at the final
step); all 564 `SuspensionFrame` + 59 `PureFrameFinalizer` impls (mechanical
codemod first — compile errors are the progress counter — then family
conversions); `rum-derive` unchanged; the adapter + split frames enter
`runtime/tests/frame_root_trace/derived_inventory.rs` and the named-frame
count; the frame conversions delete their `ContinuationValue` impls (the
25-impl closed set is pinned by a unit test that shrinks stepwise); retention
fingerprints + ownership rows replayed per TEMPLATE standing duty at every
step (the one new `UnknownHostPayload` row lands with brief-602 and is
disclosed then); the oracle grows `marker_corpus.rs` + `corpus_positions`.
Host surfaces (cli/repl/jupyter) do not touch frames — unchanged.

**Instruction-gate plan.** The macro change touches hot functions:
`try_dispatch_builtin_call` (both sites), `eval_set` (Set rhs),
`prepare_arguments_for_call_with_unevaluated_restore`, plus
`run_call_dispatch_stage`/`try_apply_function` frame construction (cold).
Labels (verified spellings from `scripts/core-language-microbench.sh`):
`set-ownvalue`, `symbol-ownvalue-read`, `downvalue-exact-hit`,
`downvalue-pattern-hit`, `downvalue-guarded-hit`,
`downvalue-guarded-decline-fallback`, `matchq-integer-success`,
`matchq-integer-fail`, `matcher-single-tiny`, `cases-wide-no-match`,
`replaceall-simple`, `rule-rhs-deep-small`, `rewrite-guarded-hit`,
`rewrite-guarded-decline-fallback`, `replacepart-guarded-hit`,
`set-part-fixed100-len100`, `set-part-fixed100-len800`, `part-list`,
`block-init-read-repeat`, `sort-callback-small`; same-tier neighbours
`subvalue-unguarded-hit`, `default-guarded-hit`, `table-iterator-plus`,
`deletecases-wide-no-match`, `downvalue-exact-miss`. Protocol: base and
candidate from same-path source exports into separate target dirs, both
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`,
`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, N and 2N, stdin
`/dev/null`, raw rows under `.temp/brief-NNN/perf/`; gates run after
brief-602 (the macro flip — the only ordinary-path-shape change) and after
the final brief (full microbench A/B). The ordinary-path claim to check: on
the Ok path the new macro expands to the same match with the same arms — the
suspend arm's `return` is not taken — so expected movement is 0.00%;
anything beyond ±0.1% on a label is investigated, not shipped.

## 6. The redrafted implementation briefs

Six briefs, each gated, each landing green, oracle green in both builds at
every step. brief-600's five-brief shape is restructured: the channel flip
now carries the relocations (its behaviour obligations), the falsifier moves
into the six-REAL-frames brief (it must exist before any family converts,
per the Q7 ruling that the channel flip completes first), and the
dispatch-plan split gets its own brief because it is the one structural
frame change. Drafts in TEMPLATE shape — the director rewrites before
dispatch.

### brief-602 — G13 step 1: two-arm channel + typed core + returning macro

**Goal.** Land `FrameOutcome`, `ErasedSlotValue`, `SlotValue` (closed
primitive set), `TypedFrame`/`TypedAction`/`SlotFinalizer`, the adapter, the
driver flip, `suspend_typed`/`suspend_insert`, and the **return-form macro
arms**, with no behaviour change: every existing frame is mechanically an
identity typed frame (`Input = Expr`, `Output = Expr` — the encoded markers
still cross as `Value` and still round-trip), and the 8 (b)-site relocations
land (profiling records into the suspend arms; the 3 non-returning inserts
keep the typed primitive; the 3 stdlib helpers genericize).

**Scope (MAY modify).** `crates/rum-runtime/src/runtime/suspension/core.rs`
(+`primitive.rs` slot impls), every file with a `SuspensionFrame`/
`PureFrameFinalizer` impl or macro use (rum-runtime + rum-stdlib; the site
list is `.temp/brief-601/macro-use-classification.tsv` and
`ast-macro-positions.tsv`); `kernel_api.rs`; the 8 (b) sites' files
(`eval_dispatch/mod.rs`, `definitions/definitions.rs`,
`eval_call_prepare.rs`, `kernel_data/control_data/branching_loops.rs`,
`accounting_mode/frozen.rs`); the 3 runtime test files calling
`insert_suspension_frame` directly; standing-duty inventories per TEMPLATE
(the one new `UnknownHostPayload` row: disposition Unbounded / Expr-debt /
unresolved, same as ledger rows 53-55, disclosed).

**Key steps.** (1) Core types + adapter + closed-set unit test. (2) Flip
`resume`/`FrameAction`/`finish`/driver to `FrameOutcome`; codemod every impl
body to wrap/unwrap `Value`. (3) Flip both macro arms to the return form;
fix the 246 insert sites to `suspend_insert` (the request is in scope at
each — the classification table's evidence); keep the 3 side-effect inserts
on the typed primitive. (4) Relocate the 7 profiling records into their
suspend arms. (5) Genericize the 3 stdlib cold helpers. (6) Compile errors
to zero (progress counter); inventories replayed.

**Verification.** `cargo fmt --check`; `cargo test --workspace`; both
feature builds' `local_production_acceptance`; suspension suites + E1 oracle
green; a profiling-enabled probe asserting the suspending-builtin timing row
and stack depth are unchanged (the §2.2 obligation);
one-codegen-unit instruction gate on the §5 label set (expected ±0.1%).

**Hard rules.** TEMPLATE standard; no decoder semantics change; no marker
deletion; stop-loss keyed on compile-error progress.

### brief-603 — G13 step 2: the six REAL frames typed + the marker-corpus falsifier

**Goal.** Add the `marker_corpus` oracle variant (§5 mechanism, corpus
positions on the descriptors covering the six families); show it red on the
21 divergences; convert `TryApplyFunctionSuspensionFrame`,
`FinishCasesRuleMatchSuspensionFrame`,
`FinishPositionalPartSelectorEvaluationSuspensionFrame`,
`FinishAssociationSetPartSelectorEvaluationSuspensionFrame`,
`RunCallDispatchStageSuspensionFrame` (all three constructors, Output =
`Option<Expr>`), `FailureMessageFromTemplateSuspensionFrame` (+ the styled
sibling's Input/Output made explicit); falsifier green.

**Scope.** `scoping/callable.rs`; `kernel_data/rewrite_data/cases/collect/`;
`definitions/part_assignment.rs` (+resume_frames);
`kernel_data/association_data.rs` (+resume_frames); `eval_dispatch/mod.rs`;
`kernel_data/control_data/status.rs` (+resume_frames);
`eval_core/call.rs` (the dispatch-plan Option decodes these frames feed);
`kernel_data/evaluated_part_selectors.rs` (the `FinishEvaluatedPartSelectors`
identity hop folds into the caller's Input);
`crates/rum-stdlib/tests/suspension_oracle/` (`marker_corpus.rs`,
`corpus_positions` field, catalog wiring — the manifest's variant assertions
must not change); closed frame lists + inventories.

**Verification.** `cargo fmt --check`; `cargo test --workspace`; the
falsifier's before-red (21 rows) and after-green pasted verbatim; both
builds; instruction spot-check on `downvalue-exact-hit`,
`set-part-fixed100-len100`, `sort-callback-small` (cold-path sanity).

**Hard rules.** No panic shapes in the corpus; markers otherwise untouched.

### brief-604 — G13 step 3: dispatch-plan split + prepared-call family

**Goal.** Split the two dispatch-plan frames per stage (§4.2), type
`PreparedCallArguments` and the argument-resolution chain, delete the
`call.rs:322` `.expect`, type `FinishPreparedCallArgumentsFinalizer`
(`SlotFinalizer<PreparedCallArguments>`), delete the
`prepared_call_arguments_from_continuation_expr` decoder family.

**Scope.** `eval_core/call.rs`; `eval_call_prepare.rs` (+resume_frames);
`evaluated_part_selectors.rs` (fold the `Arc<OnceLock>` published API into
the typed channel — `EvaluatedPartSelectorsEvaluation`/
`EvaluatedPartSelectorsContinuation` deleted; `evaluate_part_selectors_`
owned_for_builtin` callers convert); oracle corpus positions for the family;
inventories.

**Verification.** as brief-603 (workspace, both builds, falsifier extended,
`cargo fmt --check`); no instruction gate (cold paths; state it).

### brief-605 / brief-606 — G13 steps 4-5: runtime, then stdlib codec families

**Goal.** Convert the remaining codec families to typed state and delete
their markers and lenient/panicking decoders — runtime first (matcher:
`CompiledMatchOutcome`, binding maps, sequence/orderless/optional choice
states; interpreted payloads; rhs guards; cases/replace/delete/position/
count traversal outcomes; `EvaluatedPartSelectors` codec;
`EvalCapturedOutcome`; message/render `String`; block/scoping; loader
frames), then stdlib (lazy realization incl. `LazyRealizeSuspensionFrame`,
string search/replace incl. `ExactStringCaseOutput`, association suspended
states, schema frames incl. the generic `SchemaValidationSuspensionFrame`,
`resumable_merge_sort`, remaining `kernel_api::ContinuationValue` users).
Family order by producer/consumer SCC so edges flip together; corpus
positions extend per family.

**Scope.** The family module trees enumerated from brief-600's
`decoders.tsv`/`slot-producers.tsv` + this brief's crossing-type census;
oracle + closed lists as usual.

**Verification.** as brief-603 per family; instruction slopes after the last
runtime family (matcher labels) and after the last stdlib family.

### brief-607 — G13 step 6: delete the in-band marker system

**Goal.** Delete `ContinuationValue`, `from_continuation_expr`/
`into_continuation_expr`, the lenient primitive decoders, and every
`System`InternalContinuation*`/`InternalInterpreted*` head reference; a
drift test asserts zero continuation heads in production source (fresh-scan
drift test, like the builtin-reference one); final instruction gate + full
microbench A/B.

**Scope.** `suspension/primitive.rs` (deleted), the codec modules, the drift
test; check `docs/reference` for documented heads (expectation: none —
undocumented internal symbols).

**Verification.** `cargo fmt --check`; `cargo test --workspace`; both
builds; `scripts/core-language-microbench.sh` A/B with the §5 labels +
neighbours, one-codegen-unit slopes, raw rows persisted.

## 7. Open questions for the director

1. **SlotTypeError's guest-visibility.** A classified internal error that
   only a kernel bug can reach: silent-log (`TerminatedEvaluation`-style) or
   a loud internal-error message? Recommend loud; the oracle catches it
   either way.
2. **The styled-message Some/None protocol.** Typing the styled frame
   (`Input = Option<Expr>`) makes the today-bypassed Some/None protocol
   explicit (`Some(m) => Ok(Some(styled(m))), None => Ok(None)`) — a
   behaviour-identical but *strictly* more total body. Confirm you want the
   explicit match rather than preserving the unconditional wrap via
   `Input = Expr` (which would keep the current shape but leaves the
   FailureMessageFromTemplate frame's Output as the odd one in its chain).
3. **Corpus-position granularity.** One placeholder per descriptor vs one
   per callback argument position (descriptors with several guest-value
   positions could substitute all simultaneously — the stronger falsifier;
   cost: N×shapes fixtures instead of 1×shapes). Recommend all-positions-
   simultaneously.
4. **brief-602 size.** The codemod touches every impl (compile-error volume
   ~623 bodies). The classification table is the site list; if you prefer
   smaller steps, split runtime-first/stdlib-second (safe: the channel type
   is identical across the split), as brief-600 also offered.
5. **The `Expr→()` and `()→Option<Expr>` propagation edges** (one each,
   `lazy/machine.rs drive_machine` family) are the only edges whose caller
   Ok type is `()` — they are covered by the published-Option protocol, but
   E2 will have to decide whether a `()`-returning may-suspend driver is a
   disposition-3 (narrow boundary) shape. Flagged for the E2 design brief,
   no action in G13.
