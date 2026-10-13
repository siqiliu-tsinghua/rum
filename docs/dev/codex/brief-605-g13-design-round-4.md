# brief-605 — design round 4: G13's typed frame channel, ready to implement

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

rum's suspension frames pass completed values to each other through a channel
typed only by convention: typed continuation state (an `Option<Expr>`, a match
outcome, a prepared-argument set, …) is encoded into `Expr` under internal
``System`InternalContinuation*`` heads and decoded by the consuming frame with
lenient or panicking decoders, and nothing checks that a frame completes the
type its consumer decodes. Six frames complete a value of the wrong type, which
the lenient `Option` decoder happens to accept for ordinary values (21 recorded
cases where the resumed path and the queued-input path disagree). This is G13.

The repair keeps typed state out of the `Expr` channel and makes the compiler
check every frame's input and output. Three design rounds and two reviews have
settled most of it:

- brief-603 (`docs/dev/codex/brief-603-g13-typed-composition-design.md`):
  the current design — a closed runtime-owned `ContinuationState` enum, a
  sealed `SlotValue`, `TypedFrame` with `Input`/`Output`, a zero-sized
  `SlotWitness` binding the input, the return position binding the output,
  checked constructors for `Evaluate` and finalizers, a chain builder, a
  continuation token, a named mismatch variant, an eleven-step plan.
- **brief-604** (`docs/dev/codex/brief-604-g13-design-review-2.md`): your own
  review of it — ten findings F1-F10 and seven required changes. The director
  verified the token, table-join and error-capture findings in source and
  **adopts the review in full. It is the core input of this brief.**
- Earlier: brief-600, brief-601 (inventories, witnesses), brief-602 (your
  first review). Their tables are under `.temp/brief-600/`,
  `.temp/brief-601/`, `.temp/brief-603/`.

This round produces the design and the implementation briefs that can be
dispatched as they stand.

## Settled (do not reopen)

Typed state travels out of band; `Expr` is the only value-arm type. The closed
runtime-owned enum; the sealed conversion with a hidden supertrait; the
invariant zero-sized input witness; the return-position output binding; typed
finalizers completing their own frame's output; `Evaluate` requiring
`Input = Expr, Output = parent's Output`; one input and one output type per
frame (the dispatch-plan frames split per stage); a mismatch is a classified
error in every build, never a panic; the rum-vs-rum regression corpus of
internal encoding shapes in the oracle, tested per position and per shape; the
end state has no legacy decoder, no `ContinuationValue`, no internal
continuation heads and no unrestricted insertion. E2 (a may-suspend result type
with no `Try`) keeps only the global ban on frameless `?` propagation and its
measured representation choice; it may change `TypedAction`, token helpers and
frame return types as well as the macro — say which, honestly.

## What the design must settle — brief-604's required changes

1. **The continuation token** (F1): adapter-minted evidence for a frame
   *type*, accepting a fresh `F` value with the advanced state; non-`Clone`,
   scoped to one typed resume, no public minting; initial and repeated
   suspension evidence paths for every deferred-completion family (prepared
   call, lazy and schema callback helpers, synthetic-`Err` sites such as
   `list_plumbing.rs:47`, `:164`, the lazy machine's callback evidence at
   `lazy.rs:446`); drop-before-finalizer preserved.
2. **Chains and the remaining insertion routes** (F2): `FrameChain<I, O>`
   with the first input witnessed from the actual producer; no exported
   unbound insert; the default-fallback, retention and public-file-rollback
   routes each bound at their real obligation or narrowed to a structural
   operation, as F2 describes.
3. **One Rust visibility arrangement** (F5) that compiles without relying on
   lints or `#[doc(hidden)]`: public typed APIs and the public carriers
   `rum-stdlib` needs (including a public opaque selector consumer), sealed
   hidden conversions, opaque actions, erased construction restricted to the
   suspension module and its adapters. List every public item.
4. **The mismatch route** (F4): detected before the failing consumer is
   discarded; the consumer and the active suffix unwound structurally with
   output rollback and nested owners handled; the classified error surfaces at
   the request boundary and cannot be turned into data by `EvaluationData`,
   test-report capture or any other catch-all (audit them all); an absence
   check that cannot be suppressed; fault-injection tests under capture,
   cleanup and rollback scopes, followed by a fresh request, in both builds.
5. **The inventory, at dataflow level** (F6, F7, F8):
   - one row per actual whole-slot Rust type, including composite and sum
     carriers (e.g. the sequence matcher's completed-outcome versus
     pending-rest slot), with every producer and finalizer branch, the
     consumer, visibility, re-suspension behaviour and migration step;
     payload fields and helper accumulators separated;
   - the propagation edges with the table join fixed (`sites_reconcile.py`
     writes `yes`, `propagation_edges2.py` reads `YES`), and per edge the
     producer → conversion → consumer chain, re-suspension, disposition and
     step; unresolved and indirect calls listed, not dropped;
   - the chained-insert rows themselves;
   - every transformation between a callee's result and the macro, found by
     dataflow — including same-type and error transformations and work after
     the macro that a return-form macro would skip (e.g.
     `branching_loops/iteration/driver.rs:383`, `:396`) — each with its
     resumed implementation, repeated-suspension behaviour and a witness that
     would observe its omission or duplication;
   - every remaining frame of the populations no step migrated (G-value,
     G-vec — e.g. `arithmetic/stats.rs:233`) assigned to a step.
6. **A green sequence built from those edges** (F3): legacy exports and
   control flow kept, by name, until their last user migrates; the typed
   adapter staged with a compatible driver; connected migrations combined or
   joined by named temporary bridges with the step that deletes each; no
   codec that accepts either arm to hide mixed protocols.
7. **Accounting, transit and gates** (F9, F10): the adapter's second
   allocation in queue accounting, admission and the independent occurrence
   model (`suspension/core.rs:191`, `:519`, `suspension/trace.rs:110`,
   `resources/occurrence/suspension.rs:24`); the synchronous transit contract;
   inventory row transitions disclosed as planned migrations (old public
   constructors change their `PublicInput` rows); the full per-label
   microbench A/B on **every** step that changes a function on an evaluator,
   matcher, traversal, block or selector path, instruction slopes as a
   supplement; compile-fail controls that differ from a passing control only
   in the intended invalidity.

## Evidence rules

- Witnesses are rum-vs-rum equality controls with **ordinary values only**:
  the resumed path against the queued-input path, compared as the oracle does
  (`crates/rum-stdlib/tests/suspension_oracle/descriptors.rs:191` — value,
  messages, prints, trace, projections), suspension asserted at an attributable
  point (say which source line pauses, from evaluation order), distinct answers
  per pause, repeated suspension where the family re-suspends. This round runs
  no internal-encoding shapes; those are established.
- Every count reconciles against a raw source scan; say how.

## What I want back

1. The tables (`.temp/brief-605/*.tsv`) and the witnesses with output.
2. The design: Rust signatures and visibility, the token, chains and routes,
   the mismatch route, accounting and transit — each with why it is sound,
   including against every counterexample in brief-602 and brief-604.
3. The E2 boundary.
4. The green sequence and the implementation briefs in `TEMPLATE.md` shape,
   complete enough to dispatch: Goal, Scope with every file, steps, gates
   (correctness, compile controls, inventories, performance), hard rules.
5. Open questions for the director.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-605/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief605_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-605/probe.rs`.
- Everything else is read-only, including the earlier `.temp/brief-60*/`
  directories (copy, do not edit).

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-605/`.
- Run no probe that panics or aborts.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief605_probe`
  (900 s); `cargo test -p rum-stdlib --lib ownership_graph` (900 s) if needed;
  `python3` scripts (300 s each). File reads, `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.
- Checkpoint to `.temp/brief-605/PROGRESS.md` after each section with the
  exact next action.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. Tables and witnesses.
3. The design, section by section, with soundness arguments.
4. The E2 boundary.
5. The green sequence and the implementation briefs.
6. Open questions for the director.

---
