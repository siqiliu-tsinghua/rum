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
