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

## Report — brief-605, design round 4

### 1. What changed

**Design only; no production change.** The deliverables are under
`.temp/brief-605/`: source/dataflow inventories, ordinary-value witnesses with
actual output, a compiled positive API model, the design, four full
implementation drafts, and reconciliation scripts/logs. The final scratch
integration test was copied to `.temp/brief-605/probe.rs` and
`crates/rum-stdlib/tests/zz_brief605_probe.rs` was **deleted before this report
was appended**. Its imported helper/model sources are archived alongside it.

No git, network, external-engine comparison, production edit, new unsafe,
internal-encoding-shape probe, panic or abort was used. Commands were serial
and bounded as authorized, with TMPDIR under `.temp/brief-605/`.

Start with the [evidence index](../../../.temp/brief-605/README.md),
[design](../../../.temp/brief-605/design.md), and
[final audit](../../../.temp/brief-605/final-audit.json). The baseline check
examines 1,971 existing files. Its two preexisting differences are
`scripts/dispatch-brief.sh` and brief-600's report; neither was edited here.
Production Rust sources remain unchanged.

### 2. Tables and witnesses

The inventories retain source bodies and unresolved cases, rather than
treating syntax as a compiler effect certificate.

| Population | Reconciled result | Evidence |
|---|---:|---|
| Macro sites | 600 = 348 result + 252 insert | [Raw source/AST reconciliation](../../../.temp/brief-605/sites-reconciliation.tsv) |
| Frame/finalizer implementations | 564 + 59 = 623; all assigned 608 | [Full migration rows](../../../.temp/brief-605/all-frame-migrations.tsv), [independent raw scan](../../../.temp/brief-605/raw-frame-scan.tsv) |
| Whole-slot identities | 67 = Expr + 66 state variants | [Whole slots](../../../.temp/brief-605/whole-slots.tsv) |
| Decoder identity partition | 65 whole-slot mappings + 69 nested/helper identities = 134 | [Nested/helper rows](../../../.temp/brief-605/nested-fields-and-codec-helpers.tsv) |
| Helper accumulators | 12, separately classified | [Helper obligations](../../../.temp/brief-605/helper-accumulators.tsv) |
| Completion expressions | 1,295 AST expressions, including nested Ok/Err constructors | [Branch/source/output obligations](../../../.temp/brief-605/completion-branches.tsv) |
| Original resolver with repaired join | 619 edges; 485 frameless callers, replacing the incorrect 617 | [Exact control](../../../.temp/brief-605/join-fixed-original-resolver.tsv), [output](../../../.temp/brief-605/join-fixed.log) |
| Fresh may-domain calls | 16,497 = 2,289 may edges + 10,692 unresolved + 3,516 resolved non-may | [Edges](../../../.temp/brief-605/propagation-edges.tsv), [unresolved](../../../.temp/brief-605/propagation-unresolved.tsv), [non-may](../../../.temp/brief-605/resolved-nonmay-calls.tsv) |
| Same-carrier / indirect calls | 1,664 same-carrier edges; all 349 indirect calls retained as unresolved | Same raw call partition; these counts are subsets |
| Actual insert chains | 62 = 61 adjacent pairs + driver.rs:383/396 | [Every chain, frames and middle slot](../../../.temp/brief-605/chained-inserts.tsv) |
| Deferred families / remaining routes | 14 / 10 | [Deferred obligations](../../../.temp/brief-605/deferred-obligations.tsv), [routes](../../../.temp/brief-605/insertion-routes.tsv) |
| Direct producer method transforms | 90, with concrete family dispositions | [Transform dispositions](../../../.temp/brief-605/transformation-dispositions.tsv) |
| All method candidates | 5,011 raw AST occurrences retained one-for-one; 359 expanded local receivers | [Full candidate table](../../../.temp/brief-605/all-transform-candidates.tsv) |
| Result-pattern branch candidates | 2,272 selected from 9,929 raw match arms | [Branch transformations](../../../.temp/brief-605/result-branch-transformations.tsv) |
| Macro whole paths | All 600, including owner body, aliases, assignments and outer tails | [Whole-path rows](../../../.temp/brief-605/macro-whole-paths.tsv) |
| Catch-all audit | 2,599 error/wildcard AST arms, plus 1,558 method candidates and the raw Result-pattern scan | [Arms](../../../.temp/brief-605/catchall-audit.tsv), [methods](../../../.temp/brief-605/catchall-methods.tsv), [patterns](../../../.temp/brief-605/catchall-patterns.tsv) |

The 67 slots are concrete **proposed** Rust identities; composite/sum rows are
new nominal carriers, not rebranding of old Expr payloads. Whole-slot fields
and nested codecs are separated. The 643 old decode-use rows are retained as
209 syntactically whole-slot uses and 434 nested/helper or unresolved uses;
that classification is explicitly heuristic. Completion rows carry actual
source branches and their design output obligation; default/delegated Expr
rows are not falsely labelled compiler-proved.

The 90 direct maps are not the entire transformation audit. The method,
branch and complete-owner tables preserve alias-mediated, same-type, error
and non-method transformations, including work outside the innermost macro
block. Uncertain receiver types and indirect calls remain visible. Executed
family controls and future required per-path controls are labelled separately.
Global Rust name/effect resolution is not claimed. Raw partitions and exact
checks are in [reconciliation.json](../../../.temp/brief-605/reconciliation.json),
[dataflow-counts.json](../../../.temp/brief-605/dataflow-counts.json), and
[transform-reconciliation.json](../../../.temp/brief-605/transform-reconciliation.json).

**28 ordinary-value witnesses pass.** Their
[sources/prompts/attributions](../../../.temp/brief-605/witness-cases.tsv) and
[actual observations/logs/projections](../../../.temp/brief-605/witnesses.tsv)
are retained in full. They call `descriptors::accept(Checks::ALL)`, compare
resumed against queued input, assert actual suspension count and prompt,
supply distinct answers, and compare value/error/termination, messages,
prints, trace and projections. A fresh request is included by the oracle.

Coverage includes repeated prepared arguments with Unevaluated restore;
actual First/Normal lazy realization and lazy bounds; schema Leaf/Default;
Differences' synthetic-error helpers; guards; Reap rendering/finalization;
Replace heads/levels and Except chains; held Part selectors; true string
guards; iterator-scope restoration; Inner's cancelling Option wrapper; the
G-value median wrapper; message/print effects; default decline and actual
delayed RHS; options; evaluated-head wrapping; confirmation info; and Failure
Message/StyledMessage rendering.

Two attribution corrections matter. `prime.actual.thread` removes Listable
and installs only a scalar downvalue, so the outer list reaches the explicit
threading helper at number_theory.rs:289. `replace.arguments.twice` is labelled
as argument preparation; the separate `replace.targets.twice` uses position
PatternTest callbacks at replace_part.rs:290/:451 and pauses three times
inside target collection. It is not credited to an earlier argument pause.

The final authorized command was:

```text
TMPDIR=/home/remi/dev/rum/.temp/brief-605 python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief605_probe
```

Verbatim final output from [probe-run8.log](../../../.temp/brief-605/probe-run8.log):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.03s
     Running tests/zz_brief605_probe.rs (target/debug/deps/zz_brief605_probe-75664b16a2412803)

running 4 tests
test api_prototype::downstream::public_surface_and_zst_witness_compile ... ok
test ast_inventory ... ok
test flow_probe::dataflow_inventory ... ok
test ordinary_family_witnesses ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.06s
```

Eight serial invocations were made. Run 2 returned an error because the test
reader passed CSV-quoted source literally, producing zero pauses; it did not
panic or abort. The reader was fixed, and runs 3–8 pass. Earlier raw logs are
preserved. No ownership/workspace/performance gate, production negative
compile control, fault injection or internal-shape corpus was run in this
design brief. Those are explicit implementation gates, not claimed results.


The final attribution check strengthened two more controls: nested Condition
terms now give two distinct RHS guard entries (rhs_guards.rs:203/:280, cursor
advance :263), and Except now pauses in its inner matcher before exclusion,
crossing the actual compiled two-frame chain at core.rs:212/:213. Lazy bounds
are attributed to initial imin at lazy.rs:248 and resumed imax at
table_continuation.rs:76; the literal step does not pause at :91.

### 3. Design and soundness

The [full design](../../../.temp/brief-605/design.md) supplies Rust signatures,
visibility, all newly public items, construction authority, ownership and
failure sequencing. The [compiled API model](../../../.temp/brief-605/api_prototype.rs)
checks the positive visibility/type arrangement and zero-sized witness; it
does not simulate the production driver.

**Closed channel and visibility.** Only Expr constructs the Value arm. Every
other registered type has one exact State variant, with no dual-arm decoder,
Any/downcast registration or Value fallback. Runtime carriers become visible
to the central runtime enum without exposing their fields publicly. Stdlib
gets the public opaque selector type and an actual typed selector producer.
SlotValue is a sealed marker with a hidden Codec supertrait. Codec operations
also require an unconstructible private key: a generic public bound must not
make hidden conversion methods callable. Opaque TypedAction exposes only
checked complete/evaluate/finalize/finish_work operations. Erased construction
stays inside suspension/adapters.

**Witness, chains and fresh-state token.** An invariant ZST witness borrows
the actual producer Result before transformation; the return position binds
the output. FrameChain retains both endpoints, witnesses the first input,
checks every adjacency, and returns its last output once. ContinueToken is
adapter-minted, non-Clone, lifetime/type invariant, stores no Arc, and accepts
a fresh F. Prepared cursor/prefix/restore/splice state therefore advances
before reinsertion. ResumeStop preserves the owning output through helper
accumulators; a typed finalizer completes that same output after both
originating Arcs are dropped. Lazy/schema stop records retain evidence from
the real callback evaluator. Differences receives witnesses from its actual
Expr/Vec producers, not a guessed synthetic Err annotation. Iterator body
and scope restoration at :383/:396 become one Expr→LoopScopeSlot→Expr chain.

**Other insertion routes.** Default fallback returns its actual
Result<Option<Expr>> instead of discarding insertion output. Retention's
input/output are fixed to Expr and its receipt finishes into RetainedResult;
the private hook preserves pre-admission, installation, receipt transfer,
post-check and abandonment order. Initial and repeated retention failures
use the actual callback witness and a fresh continuation factory/advanced
prefix, without exposing FrozenValues construction. Public-file rollback
becomes a generation-bound structural obligation with no value completion.
Input seeding is Value; ordinary request completion requires Value. The
rewrite-inspection completion boundary consumes its separate typed result.

**Fatal mismatch.** Exact decoding fails before invoking or discarding the
consumer. A private DriverFault bypasses ordinary frame error completion.
The active consumer, suffix and nested pending owners enter structural
abandonment with output rollback and existing approved cleanup ordering;
cleanup that requests input has its nested owner abandoned too. The first
fault survives cleanup failures. Only after unwind does the request boundary
expose named EvalError/SessionRequestError variants. EvaluationData and
TestReport cannot consume the private fault. Explicit passthrough and a
request latch defend direct public-error conversions as well.

FaultProbe records only at cold creation, retains a per-session occurrence
handle through reset/rollback, and is checked unconditionally outside oracle
Checks flags. Step 607 explicitly adds the runtime test observer feature and
stdlib dev-feature wiring; injection remains private to runtime unit tests.
Production builds acquire no per-evaluate diagnostic counter or allocation.
Fault tests cover capture, cleanup, output/scope/option restoration, memory
file rollback, nested cleanup input and a fresh request, in both builds.

**Accounting and transit.** The adapter's outer Arc and inner Arc<F> are both
charged in queue storage, pre-admission and the independent occurrence model;
roots are delegated once. Legacy entries keep their one-Arc layout until
removed. No guest/checkpoint occurs in the short completion-to-consumer
handoff. Before consumer guest evaluation, an activation-local TransitRoots
record on the active queue entry keeps incoming Expr roots visible and
accounts for input/container/root-buffer storage. It does not retain frozen
accumulator Arcs across finalization. Transfers into a pending owner are
explicitly reconciled to avoid double charging. Cold GC/memory-threshold
tests and the independent model gate this new record. Existing Expr
destruction debt remains; old public constructor/insertion inventory rows
are planned removals/replacements, not merely hash refreshes.

**Whole outputs and transformations.** Ordered match vs collection,
argument/rest match vs collection, and orderless-optional match vs collection
need distinct sums/finalizers; early completed variants are included. Reap's
bucket finalizer returns a guest List Expr, not a Vec slot. Dispatch stages
split by output, and the unused PrepareArguments wrapper/branch is removed
without inventing a SharedArgs slot. Semantic maps and post-macro work enter
a shared synchronous/resumed phase; transport-only maps disappear. The
prepared outer profiling record remains outside its existing closure.

[counterexample-closure.tsv](../../../.temp/brief-605/counterexample-closure.tsv)
maps all nine 602 and ten 604 findings to these decisions and concrete gates.
G13 proves checked local construction within the audited implementation;
safe Rust's affine use and the conservative call graph are not presented as
global suspension-effect enforcement.

### 4. E2 boundary

G13 owns local input/output equality, chains, deferred completion, repeated
state, transformations, fatal mismatch, ownership and accounting now. E2
retains the global no-Try/no-frameless-`?` restriction, rejection of discarded
or laundered obligations and uncertified tails in its restricted source
domain, and the measured return-representation choice.

E2 may change TypedFrame inputs/returns, TypedAction, ContinueToken,
ResumeStop/finish_work, callback/helper signatures, witness acquisition,
driver suspension actions and the macro. This is not promised as a two-line
macro replacement. Host request outcomes remain a separately reviewed
boundary.

### 5. Green sequence and full implementation briefs

| Draft | Green intermediate state | Explicit scope |
|---|---|---:|
| [606](../../../.temp/brief-605/draft-606.md) | Dormant typed declarations, sealed conversions, opaque actions/chains and phase-appropriate compile controls; existing driver unchanged, no nonfunctional suspension stubs | 28 files |
| [607](../../../.temp/brief-605/draft-607.md) | Compatible Legacy/Typed driver, real witnessed/token insertion, typed macro, fatal unwind, test observer, two-Arc accounting and transit; production frames remain legacy | 470-file fence, including error-enum ripple |
| [608](../../../.temp/brief-605/draft-608.md) | One atomic production conversion of all frames/finalizers/macros/state/helper/callback edges, including G-value/G-vec; remaining legacy users are only enumerated tests | 528 files |
| [609](../../../.temp/brief-605/draft-609.md) | Migrate old tests, delete all legacy exports/codecs/heads/raw insertion and the temporary macro name, then run final closure gates | 296 files |

Each draft has Goal, Context, every allowed file, Deliverable, Steps, pitfalls,
bounded commands, correctness/compile/inventory/performance gates,
checkpointing, Hard rules and report requirements. The
[scope table](../../../.temp/brief-605/implementation-file-scopes.tsv) and
[large-file assessments](../../../.temp/brief-605/file-size-assessments.tsv)
are included. A ripple fence authorizes only the specified work, not unrelated
cleanup or editing every listed file.

The sole temporary transport bridge is LegacyQueueEntry: Value→old resume→
Value, rejecting State. SuspensionFrame, FrameAction, PureFrameFinalizer,
ContinuationValue, old resume_or_suspend, raw insertion and old selector
exports stay named and available until their last users migrate. Selectors
move together in 608; the other legacy APIs disappear in 609. During 607–608
the new macro is `resume_typed_or_suspend!`; 609 replaces the old canonical
macro and removes that temporary name. No codec accepts both protocols.

The wide atomic step avoids an unsupported claim that unresolved edges form
small independent SCCs. All connected kernel/stdlib migrations finish before
that step is green; outer host request Result conversions retain their public
shape. Splitting 608 requires additional named bridges and edge evidence.

[26 compile-control pairs](../../../.temp/brief-605/compile-controls.tsv)
compile a complete positive fixture first, then change one intended
invalidity and check the primary diagnostic. They include first/adjacent/
last chain types, return output, Evaluate, finalizer, fresh token type,
minting/clone/lifetime, deferred output, generics/closures, external sealing,
same-crate sibling privacy, opaque selectors, fallback and retention.
[11 inventory transition groups](../../../.temp/brief-605/inventory-transitions.tsv)
make planned identities/removals explicit and preserve existing debt.

607, 608 and 609 each run the entire per-label microbench A/B, including
same-tier neighbors, using an archived pre-edit base binary and candidate
built from the same source path into separate target directories. No git
export is needed. Profiling is off; instruction slopes with one codegen unit
are supplemental. 606's exemption applies only while hot function bodies
remain unchanged. Both feature builds run correctness, lifecycle, absence,
compile and inventory gates. The established internal-shape corpus is run
per descriptor, substituted position and shape during implementation.

### 6. Open questions for the director

1. Accept the deliberately wide atomic 608 scope, or commission a separate
   decomposition with reviewed temporary bridges. The supplied four-step
   sequence can be executed as written; no smaller-SCC claim is assumed.
2. Review the cold input-transit root/storage record and test-only observer
   feature particularly closely. They are concrete additions with ownership,
   admission, privacy and absence gates; the compiled model does not validate
   their production implementation or cost.

No alternative transport architecture or premature E2 effect system is needed
for this proposal. The negative controls, fatal injection, complete production
conversion and performance gates remain implementation work, explicitly
assigned rather than reported as already passed.
