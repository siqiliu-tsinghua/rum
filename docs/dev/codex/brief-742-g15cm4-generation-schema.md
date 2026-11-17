# Brief 742 — G15-CM4: generated carrier plans and authenticated schema

**Director-dispatched, one stage, one gate.** This executes the first
stage of the accepted carrier-generation design (brief 741, Q9 ruling,
commit `159101b6`). It is test-layer only: replace manual postimage
placement with one deterministic generator over all 151 frozen own rows,
and prove the generation shape. It does not attempt a campaign, build a
mutant epoch or run acceptance, and it claims no new semantic catch.

## Goal

Produce, from the current source and the frozen identities, one complete
declarative carrier plan for each of the 151 own rows; render the
disabled/armed overlay set from live preimages through the ten generation
classes and the 79 retained operation subtypes; and audit the emitted
bytes independently of the renderer. The delivered schema must make the
Q6/Q7 defect shapes (P0813 initial-match-only, P2593 immediate-only)
static generator failures, and must be the input brief 743 rebinds before
the final epoch. `Scope: NONE` of production.

## Context pointers

- First stops: `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, the testing
  policy (`DISCLAIMER.md`, `AGENTS.md`) and
  `docs/dev/code-organization-guidelines.md`.
- Design evidence, all read-only, under `.temp/brief-741/`: `PLAN.md`
  (sections 1, 2, 4), `ROW-EVIDENCE.md`, `row-taxonomy.tsv`,
  `carrier-subtypes.tsv`, `SIGNATURES.md`, `attempt-plan.json`,
  `taxonomy-summary.json`.
- `crates/rum-stdlib/tests/suspension_oracle/obligations/identity.rs:36`
  and `discovery.rs:649` (identity and discovery),
  `mutations/schema.rs:6`, `mutations/source_auth/{schema,mod,tests}.rs`,
  `mutations/source_auth/overlays.tsv` (90 rows; schema sha256
  `f9604852165e377f3be8faf2ee1ec3c7ffc2c67d555dcd17337c8f7a65a4ade9`),
  and `inventory/obligations/mutants.tsv`.
- `syn` and `quote` are already test dependencies
  (`crates/rum-stdlib/Cargo.toml:26`); no new dependency is needed or
  allowed.
- The working tree carries the uncommitted CM4 test layer. Preserve it;
  never stitch, restore or reset it. No git command in this brief.

## Scope

**May change** only the test layer:

- `crates/rum-stdlib/tests/suspension_oracle/mutations/mod.rs` (module
  wiring);
- new `crates/rum-stdlib/tests/suspension_oracle/mutations/generation/
  {mod,plan,resolve,graph,render,audit,tests}.rs` (names may adapt within
  this new directory);
- `mutations/schema.rs` (generation binding only; no acceptance-bar
  change);
- `mutations/source_auth/{schema,mod,tests}.rs` (generated-schema
  authentication and export checks only);
- `obligations/mod.rs` only for a crate-private discovery re-export if
  current visibility requires it;
- new test data `mutations/generation/{recipes,paths}.tsv` (or equivalent
  declarative files in that new directory);
- mechanically regenerated `mutations/source_auth/overlays.tsv` and only
  the `source_schema` cells of `inventory/obligations/mutants.tsv`.

**Must not touch** live production, `guest.rs`/`native.rs`/adapter or host
bodies, `source_worker.rs`, observer policy, `records.rs`,
`observer_receipts.rs`, `receipts.rs`, semantic acceptance or control
bodies, `mutant-coverage.tsv`, `promotions.tsv`, or any existing
`source_key`/`tail_digest`/`phase`/`selector`/`observer` cell. No
campaign, mutant-build epoch, acceptance run, manifest permission, runtime
accessor, settlement, promotion, encoding execution or CM5. Scratch only
under `.temp/brief-742/`; the design scratch is never written.

The 90-file overlay set is the starting output set. Any additional
generated file, any preimage that leaves that set, and any byte-level
delta in an existing file must be enumerated with its structural
justification for the director at this gate before 743 freezes. No
unrestricted output path.

## Deliverable

1. **Resolve and authenticate.** For every frozen row, resolve the
   producer call, its enclosing function/closure, the named tail and its
   destructor boundary, and the relevant macro/typed-frame nodes, using
   the existing discovery identity and current source. Reject zero or
   multiple matches, unknown macro shapes, changed tails, wrong
   destructor boundaries and unaccounted continuations before rendering.
   Line numbers are evidence, never identity.
2. **Finite continuation graph.** From the producer call, enumerate the
   immediate completion, suspension-frame construction, raw `resume`,
   typed resume/finalizer and complete/decode successors that can reach
   the declared tail. Emit a path table with each successor's anchor and
   type conversion. Unknown or dynamic dispatch blocks the family; a
   plan is not complete because its first call was wrapped. P0813 must
   include the actual body pause -> resumed catch dispatch; P2593 must
   include both the immediate `Ok(Some)` and the resumed decoded
   `Some(value)`.
3. **Arming from the actual episode.** Arm only on the actual
   `InputRequired` belonging to the producer scope (or its returned
   request/witness-suspend branch), carry the token across the paired
   execution, and clear it at a new independent episode. A resumed frame
   inherits the episode; an initial match alone never arms.
4. **Render only the operation.** Emit disabled and armed copies of the
   same structure with one compile-time enable constant; evaluate the
   producer exactly once; preserve actual error/None branches, carrier
   constructors, witnesses, drops and non-target finalizers. The
   immutable selector and declared phase sit at the actual omission,
   after the token check. Repeated legitimate hits stay distinguishable
   by occurrence and constituent.
5. **Disabled parity.** The disabled arm follows the ordinary runner
   with omission disabled, and its complete record sequence must equal
   real. The only exception is P0492's already-admitted declared-duration
   comparison view; raw records stay byte-exact and everything else is
   exact.
6. **Independent audit and schema emission.** Audit each edit against its
   class's allowed AST delta and preserved nodes, resolve the emitted
   paths against the unchanged source, generate the six-column schema
   twice in separate scratch roots and require byte identity, authenticate
   the exact file set and distinct disabled/armed postimages, and
   recompute `source_schema = SHA256(exact overlays.tsv bytes)`
   mechanically. The renderer's assertion is not authority: the audit's
   authority is the pre-edit pin and the frozen declarations. A
   historical full-file postimage is never generator logic.

**Pin conditions.** For every retained carrier, the audit must prove the
same omission site and preserved nodes as the current pin. The 18 caught
rows and the four stop rows (P0267, P0312, P0323, P0410) keep their exact
arming, hit, phase and predicate semantics; the generated carriers are
re-earned on the final epoch in 743/744, and any signature drift is a
STOP, not a new expectation. A byte delta in an existing overlay file is
disclosed per file; if the generated bytes cannot reproduce a retained
carrier, return the named PortBlocked instead of normalizing silently.

## rum / Rust pitfalls (read twice)

Every omitted operation keeps its actual carrier type and error/None
branches. No fabricated successful value, emptied collection, duplicated
producer, swallowed throw or inline/optimization trick. `syn` does not
prove arbitrary control flow; an unsupported macro or dynamic
continuation is a named PortBlocked class finding. Initial-match-only
P0813 and immediate-only P2593 are invalid generators and must fail the
static gate. Do not route, select or omit by row-name substring; class
membership is structural. Do not add a dependency, a new manifest entry
or a runtime interface.

## Verification (MANDATORY — you run it, iterate to green or to evidence)

Allocation: this stage owns **4 × 1,500 s focused/maintenance envelopes
plus 2 × 6,000 s workspace envelopes = 18,000 s**. The twelve final-epoch
build/run blocks belong to 743 and are not available here. One cargo
command at a time, offline/locked, jobs 8, test threads 8, inside the
16G scope, every long command through `scripts/run-bounded.py`. Charge
each group serially; stop before exceeding an envelope.

- Add pure gates covering: all 151 plans and complete path tables; zero
  unknown/ambiguous edges; row-set/tail/selector/phase equality; the
  deterministic double generation and exact file-set authentication; the
  source-identity join; the preserved carrier nodes; and the P0813/P2593
  construction proofs. Calibrate each falsifier so it fires: wrong or
  duplicate anchor, wrong tail/destructor, missing resumed edge,
  initial-match-only P0813, immediate-only P2593, overlapping edits,
  fabricated None/error substitution, cross-constituent token, disabled
  difference; valid complete counterparts green. A pure gate is never a
  caught receipt.
- Replay the standing instruments after the edits and before the
  workspace gate, in this order, quoting verdict lines: fingerprint
  regeneration (`regenerate_local_contract_fingerprints`), certificate
  import/revalidation (`local_production_acceptance`, both modes),
  ownership discovery/exactness plus the rejection census, LP01 and the
  typed-slot census. Fingerprint-, certificate- or position-only reds are
  ordering artifacts: replay, never hand-edit or weaken an inventory. A
  pre-existing identity/status/disposition change, a new unknown host
  payload, a moved rejection census or a changed LP01 statement is a
  STOP.
- `cargo fmt --check`, then the whole workspace in both feature modes
  through the bounded runner with `RUST_TEST_THREADS=8 --no-fail-fast`,
  the exact seven skips (`lp03_adopted_phases_...`,
  `lp04_adopted_definition_rows_...`,
  `lp05_adopted_assignment_rows_...`, `lp658_host_eval_reset`,
  `lp658_host_resume_reset`, `lp655_root_context_falsifier`,
  `lp656_public_root_context_falsifier`), then the same lp03/lp04/lp05
  binaries under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` with hash checks.
- Preserve the eight observer, ten source-auth and four Q7 predicate
  controls unchanged. The full suspension_oracle and typed_slot_census
  runs must be unfiltered in both modes.
- The stage passes only with: 151 plans, zero unknown edges, deterministic
  byte-identical double generation, exact emitted file set, mechanically
  regenerated `source_schema`, immutable-field comparison green, all
  focused controls green, both workspace modes green, and no unexplained
  entry drift. Missing authentic source or unsupported topology blocks
  its family and returns for design; no provisional incomplete schema is
  called complete.

## Checkpointing — protect the scene

Save the entry hashes, the candidate plan/schema hashes, the audited
file-set delta with per-file byte-delta justifications, every source and
replay change, every command result and the exact next action to
`.temp/brief-742/PROGRESS.md`. This schema is intermediate: 743 rebinds
it after its own test-source changes and no epoch identity is reused
across that edit.

## Hard rules

- Test layer only; no production, manifest, lock, dependency, predicate,
  acceptance-bar, selector, phase, observer, coverage or promotion cell
  change. `production_changes=0`.
- No git, network, `unsafe`, external engine, new dependency or write
  outside this repo and `.temp/brief-742/`.
- No campaign, mutant-build epoch, acceptance, settlement, SAME, owner
  promotion, encoding execution or CM5; `CHANGELOG.md` and S10 stay
  blocked.
- Pair `(45, 5, false)` stays Open at zero admitted receipts; four
  structural controls and five armed-without-hit `Error` cells per row
  preserved.
- The ordinary guard keeps its position and force; a required runtime
  accessor or an env switch that disables an ordinary assertion is a
  STOP.
- C2 stays Open at 118 Residual + 0 Lost; the 51 owners and five source
  duties stay Open.

## Report format

`rum-report/1` under `.temp/brief-742/`. Quote generator, audit, replay
and workspace verdicts; list all changed files and the exact
`source_schema` transition; enumerate every generated overlay path and
every byte delta from the current pins, including any additional file;
state no new own catches, pair zero/Open and owners unchanged. Validate
with `python3 -B scripts/report.py check 742` (no `--against-git`); the
final message is only `python3 -B scripts/report.py summary 742`.

## Continuation 1 ruling (director) — Q10: the option-reader generic-continuation finding is upheld; the stage continues under a declared generic-continuation summary

The STOP is correct and earns no credit: the 151 source/tail/destructor
joins and the 90 preimages / 180 postimages authenticate, `source_schema`
is unchanged at `f9604852…`, no repository file was written, and the four
option rows' seven successor anchors return zero concrete targets because
the producers are generic over `OptionReadContinuation<V>` and
`FrozenOptionReadContinuation<D::Value>`. No schema or plan was emitted.
The 1,200 s conservative charge to focused envelope 1 is accepted; 742's
remaining focused budget is 4,800 s (the rest of envelope 1 plus three
envelopes), and both workspace envelopes are untouched.

The missing piece is not unknown. The frozen inventory already declares
each binding: `consumer_route`
`C6.OptionReadContinuation.DeleteCasesOptionWorking.ordinary` (P1139),
`...ArrayFilterOptionWorking` (P1142), `...NamesOptionWorking.IgnoreCase`
(P1147) and `...MonitorOptionWorking.Syntax` (P1155); the source has
`impl OptionReadContinuation<bool> for DeleteCasesOptionWorking` with
`type Frozen = DeleteCasesOptionRead` and the paired
`FrozenOptionReadContinuation<bool> for DeleteCasesOptionRead`, and the
same Working/Frozen pattern for the other three (bool / Option<Expr> /
Option<Expr> / Expr). `obligations/routes.rs:32` freezes the five generic
gaps — P0403 `DefinitionResumeRoute`, P0508 `AccountingMode`, P1147 and
P1155 `OptionReadContinuation<...>`, P2604 `HostSession::guard<T>` — and
`validate` asserts that exact set. A generic call whose frozen
declaration names its concrete binding is a known frame shape, not
unknown dispatch; the resolver's zero targets are an instrument gap.

The stage may continue, and only under this class-wide rule:

1. **Declaration is the authority.** A generic successor may be
   summarized only when the row's frozen declaration names the concrete
   binding (the `consumer_route` concrete type, or a `GENERIC_GAPS`
   entry plus the row's route). No declaration -> the family stops
   PortBlocked; never invent a binding.
2. **Summary record.** Each protocol summary is declarative plan data, not
   a per-row code patch and not a row-name conditional: the trait
   declaration anchors and associated types (`C::Frozen`, `D::Value`),
   the immediate `finish` anchors, the `freeze` anchor, the frame
   construction/resume anchors, the resumed `finish` anchor, the error
   bypass, and the concrete Working/Frozen pair with `type Frozen` and
   both `finish` bodies. Every row bound to the protocol shares the one
   summary.
3. **Audit.** The concrete type must implement the trait for the row's
   declared `V`; `Working::freeze` must return the named Frozen type;
   `FrozenOptionReadContinuation<V>` must be implemented for it; the
   resumed `finish` must be the Frozen impl's method and the immediate
   `finish` the Working impl's; the frame bound
   `F: FrozenOptionReadContinuation<D::Value>` must match. A same-spelled
   method cannot substitute. Static falsifiers: a dropped resumed edge, an
   immediate-finish substitution, a mismatched `V` or a missing Frozen
   impl fails; the four option rows must resolve through one shared
   summary.
4. **No widening.** Any generic anchor without a frozen declaration stops
   its family and returns for a ruling. Any additional declaration found
   later is reported at the gate as a new binding, never added silently.
   The summary is not permission to omit the resumed path, to treat the
   first call wrapper as complete, or to change a predicate.

All other terms stand: the 90-file set is the starting set and the
pre-edit pin is the audit authority; the 18 caught and four stop rows
keep their signatures; the generator must still deliver 151 plans, ten
classes, 79 subtypes, byte-identical double generation, the exact emitted
file set, the mechanical `source_schema` transition, all focused controls
and both workspace modes after the ordered replay. No new allocation; a
required extra group stops for a ruling.

## Continuation 2 ruling (director) — Q11: the P0001/P0002 retention bindings are admitted as program-authenticated declarations

The STOP is correct under Q10 and earns no credit: the option-reader
summary now resolves all four declared pairs (six protocol tests and four
declaration controls pass), all 151 source joins and the 90/180 pins
authenticate, `source_schema` is unchanged, no repository file was
written, and no plan or schema was emitted. The reported changes are 59
scratch files plus the three pre-existing document drifts disclosed at
entry (the Q10 commit); no tracked file moved.

The missing authority is supplied by the selected program's own source,
not by a route string. P0001's selected program runs `callback_charge`
(`native.rs:132`), whose `with_retention` entry API fixes
`M = AccountingShadow` (`retained.rs:127`) and whose suspension expression
is `failure.suspend(|| CallbackFinish)`; `CallbackFinish` implements
`RetentionContinuation` at `native.rs:111`. P0002's selected program runs
`retained_prefix` (`native.rs:174`), whose `with_unarmed_retention` entry
API fixes `M = Unarmed` (`retained.rs:109`) and whose suspension
expression is `failure.suspend(|| PrefixFinish)`; `PrefixFinish`
implements `RetentionContinuation` at `native.rs:157`. The generic
producer `RetentionContext<'_, 'id, M>::evaluate` and the frame
`RetentionFrame<C>::resume` then bind exactly those types. This is the
exact program the row's own packet executes, so it is stronger evidence
than a generic route, not a weakening of the declaration rule.

Q11 admitted, as one class-wide clause added to Q10 rule 1:

1. **Program-authenticated binding.** A generic successor may also be
   summarized when the row's selected program uniquely fixes the binding
   in its own source: the entry API signature fixes the mode `M`, the
   suspension expression names the concrete continuation type in the same
   program, that type implements the trait with the exact `resume`
   signature, and the frame construction binds `C` to it. The binding
   must be unique: no other mode/continuation pair in that program, and no
   same-named type from another module, is admissible.
2. **Audit and falsifiers.** The summary records the entry API anchor and
   its mode bound, the suspension expression anchor and its type, the
   trait impl and `resume` body, the frame construction and raw resume
   anchors. Static falsifiers: the wrong mode (Unarmed for P0001), the
   wrong continuation (PrefixFinish for P0001), a candidate from another
   program, a missing or mismatched impl, or a changed entry-API bound
   fails. A second matching pair rejects as ambiguous.
3. **No widening beyond this shape.** Any generic anchor with neither a
   frozen declaration nor a unique selected-program source binding still
   stops its family. P0508's `AccountingMode` gap remains its own and does
   not authorize these rows. The summary is declarative plan data shared
   by every row whose selected program carries the same proof, never a
   per-row patch.
4. **No new allocation.** The 4,800 s focused budget and both workspace
   envelopes remain; the stage continues to the full 151-plan resolution.

All other Q10 and brief terms stand: the 90-file set and pre-edit pin
authority, the caught/stop-row signature pins, 151 plans, ten classes, 79
subtypes, deterministic double generation, the exact emitted file set,
the mechanical `source_schema` transition, all focused controls and both
workspace modes after the ordered replay.

## Continuation 3 ruling (director) — Q12: the closed-implementation protocol is the third declaration-authority form; the fourteen `DefinitionResumeRoute` rows, `DownValueScan` and `ExceptionResumeRoute` close, the census protocols that do not close stay stopped

The STOP is correct under Q10 rule 4 and Q11 rule 3 and earns no credit:
the fourteen rows have neither a frozen concrete route binding nor a
per-row generic-gap entry, all 151 source/tail/destructor joins and the
90 preimages / 180 postimages authenticate, `source_schema` is unchanged
at `f9604852…`, no repository file was written, and no plan or schema was
emitted. The 417.92 s conservative charge is accepted; 742's remaining
focused budget is 4,382.08 s and both workspace envelopes are untouched.

The authority the fourteen rows need is not another inventory entry.
`DefinitionResumeRoute` is declared `pub(in crate::runtime)`
(`application/mod.rs:151`), so the compiler admits implementations only
inside the runtime module; the source contains exactly two, and they are
precisely the initial and resumed shapes: `()` with `Error = EvalError`,
identity `failure` and `witness.suspend(...)` (`:165`), and
`ContinueToken<'_, F>` with `Error = ResumeStop<F::Output>`,
`ResumeStop::failure` and `Err(self.suspend(...))` (`:182`). The trait's
own documentation states the contract: the same driver is entered both
initially and by its own typed frame, and only a resumed instantiation
can consume a token. A sealed trait with a finite, statically assertable
implementation set whose members are determined from in-source call sites
is a declaration authority of the same rank as a frozen route and
stronger than a route string: the closure is compiler-enforced, not
recorded.

Q12 admitted, as the third evidence form for Q10 rule 1:

1. **Closed-implementation authority.** A generic successor may be
   summarized without a per-row frozen declaration when (a) the trait is
   sealed by visibility, (b) its complete implementation set is finite and
   asserted exactly by the generator, and (c) every instantiation that can
   enter the row's graph is determined from in-source evidence. The audit
   records the trait declaration anchor, the exact implementation set and
   each implementation's method anchors. Any added, removed or moved
   implementation, any changed signature, associated type or body, and any
   visibility widening stops every row bound to that protocol until
   re-ruled.
2. **The three verified protocols.** `DefinitionResumeRoute`
   (`application/mod.rs:151`; `()` at `:165`, `ContinueToken<'_, F>` at
   `:182`) covers P0256, P0259, P0260, P0266, P0281, P0284, P0304, P0305,
   P0311, P0455, P0459, P0470, P0558 and P1615. `DownValueScan`
   (`model/definitions/down_value_dispatch.rs:25`; `LinearDownValueScan`
   at `:278`, `IndexedDownValueScan` at `:294`, selected by
   `start_indexed_down_value_scan` at `:236` under the ground-count
   threshold at `:8` and branched at `down_values.rs:155`) covers P0256,
   P0259, P0260, P0266 and P0267. `ExceptionResumeRoute`
   (`kernel_data/control_data/exceptions/resume_frames.rs:5`; `()` at
   `:15`, `ContinueToken<'_, F>` at `:29`) covers P0769 and P0773. Each
   row's summary records the trait anchor, the exact implementation set,
   each implementation's anchors and its error/ownership shape; the
   summary is declarative plan data shared by every bound row.
3. **Per-edge instantiation; both shapes required.** The route is a
   property of the edge, not of the row. Every call edge into a
   `Route`-generic function carries exactly one declared instantiation,
   proved from its own argument: a literal `()` is the initial route, the
   frame's `ContinueToken` is the resumed route, and a forwarded parameter
   inherits its caller's proof, with the fixpoint over call sites
   terminating. A plan that models only the initial or only the resumed
   instantiation is incomplete — the P0813/P2593 defect generalized — and
   must contain the initial `suspend` edge and the resumed frame-to-`finish`
   edge. Where the choice is made at runtime, as with `DownValueScan`'s
   ground-count branch, the plan carries the discriminating condition and
   both branches, each with its own anchors; it may not silently pick one.
4. **Error and ownership preservation.** The initial implementation
   converts the actual `EvalError` through identity
   (`DefinitionResumeRoute`) or `ResumeStop::failure`
   (`ExceptionResumeRoute`) and returns what the witness produces; the
   token implementation moves itself into `ResumeStop<F::Output>` and
   returns `Err`. The rendered carriers preserve both branch shapes: no
   unified error, no dropped token, no re-entry of the immediate tail
   through the resumed path, no invented successful value.
5. **Falsifiers.** Each protocol ships static falsifiers that fire: a
   third or out-of-module implementation fails the closure assertion; a
   changed `Error`, `failure` or `suspend` body fails its anchor; an edge
   with an unproved or wrong instantiation fails; a graph missing the
   initial `suspend` edge or the resumed `finish` edge fails; a swapped
   pair fails; a runtime-branched protocol with only one arm fails; the
   valid complete counterparts pass.
6. **No widening beyond the verified set.** `CallBuiltinSpec`
   (`eval_dispatch/mod.rs:321`) is a metadata bound with no continuation
   method and is not admitted; it may only be reported as an edge if a
   row's relevance analysis proves one. P1577's erased-driver dispatch
   (`suspension/core.rs:639`; `core/driver.rs:75/101/110/150`) is not
   admitted: an `Expr` decoder binding does not enumerate the queue's
   frame kinds. If that frame-kind set is finite and specifiable from
   source, the generator may prove it under rule 1 and report it at the
   gate as a new closure; otherwise the row stops. Any other protocol
   found later returns for a ruling.
7. **Inventory and budget.** `GENERIC_GAPS` stays the exact five entries
   and `validate` keeps asserting the set; P0403's entry remains its own
   and is neither borrowed nor duplicated; no route or inventory file is
   edited. No new allocation: 4,382.08 s focused remain and both
   workspace envelopes are untouched.

All other Q10, Q11 and brief terms stand: the 90-file starting set and
pre-edit pin authority, the caught/stop-row signature pins, 151 plans,
ten classes, 79 subtypes, zero unknown edges, deterministic byte-identical
double generation, the exact emitted file set, the mechanical
`source_schema` transition, all focused controls and both workspace modes
after the ordered replay.

## Continuation 4 ruling (director) — Q13: the continuation graph is scoped by the declared tail; a caller-side post-call omission whose tail lies strictly after an unmodified producer call does not need the producer's interior, and P1577 proceeds under that topology

The STOP is correct under Q12 rule 6 as written and earns no credit. The
Q12 work before it is accepted as delivered: three shared protocols with
their exact six implementations authenticated against independent
pre-edit AST pins, 100 initial/resumed route edges expanded with zero
unknown edges inside that restricted graph, 54 shared typed-frame path
anchors (initial suspension, resumed completion, constructors and the
guard-completion helper), both `DownValueScan` branches recorded, the
bound-row join, 21 protocol controls and seven closure controls. All 151
source/tail/destructor joins and the 90/180 pins authenticate;
`source_schema` is unchanged at `f9604852…`; no repository file was
written; no plan or schema was emitted. The 206.65 s conservative charge
is accepted; 742's remaining focused budget is 4,175.43 s (envelope 2 has
1,175.43 s), focused envelopes 3 and 4 and both workspace envelopes are
untouched.

The ruling that P1577's family stops unless the whole erased frame-kind
set closes was scoped to a tail inside or beyond an open dispatch. P1577's
declared tail is neither. Its producer call is
`self.drive_resuming_frames::<O>(outcome)` at
`crates/rum-runtime/src/runtime/suspension/core.rs:690`, and its declared
tail is `self.replace_accounted_evaluation_stack(saved_stack);` at
`:696` — strictly after the call, in the same caller
(`resume_suspended_slot`, from `:609`), together with the following
`self.evaluation_depth = saved_depth;`. The retained pin has exactly this
shape: the arm site wraps the call as
`{ let _cm740_producer = crate::cm740_probe::enter("P1577"); self.drive_resuming_frames::<O>(outcome) }`
so the producer runs in full, and the tail site guards only the two
restoration statements. Every path from the call to the tail is the
call's typed return plus the caller's own branches; a frame's `resume`
cannot jump into the caller's later lines except by returning, and the
caller's own entry guard (`debug_assert!` plus
`take_accounted_pending_suspension_for_resume().expect(...)`) makes a
nested invocation a fault rather than a path. The open payload protocols
therefore block nothing here: they are opened interior, not an
unenumerated continuation to the tail.

Q13 admitted, as the scoping rule for step 2's continuation graph:

1. **Tail scope.** Step 2 enumerates the successors that can reach the
   declared tail. When the declared tail lies strictly after the producer
   call in the caller and the call itself is unmodified, the call's
   interior successors cannot reach the tail except through its return.
   The interior is recorded as one opaque node with the call's full
   signature and return type and is explicitly not claimed closed; no
   other row may borrow the interior.
2. **Conditions.** (a) The producer call executes in full in both arms
   with unchanged arguments and exactly one evaluation. (b) The plan
   enumerates the call's complete typed return variants and every caller
   branch and site from the call to the tail, with anchors. (c) Nothing
   between the arm and the tail is omitted, duplicated or reordered
   beyond the declared omission. (d) The plan cites the caller's entry
   guard as the reason no nested invocation can reach the tail while the
   producer runs. (e) The carrier preserves the producer's own credited
   pauses (P1577: the `InputString:enter-resume` and `Input:finish`
   episode), the nested `Result`, the guest-form restoration, the fault
   path and the `finish_completed_resumption` branches.
3. **No interior claim.** The plan must not assert which frames or
   finalizers are queued or resumed, must not substitute the three
   `EntryRef` variants (`suspension/typed/entry.rs:15`), the `Expr`
   decoder binding or the AST candidate census for a frame set, and must
   record the four driver dispatch sites
   (`crates/rum-runtime/src/runtime/suspension/core/driver.rs:75`, `:101`,
   `:110`, `:150`) as opened and not enumerated. The erased protocol stays
   open; this ruling closes nothing.
4. **Falsifiers.** A tail inside the producer; an arm that alters the
   call, its arguments or its evaluation count; a missing return variant
   or caller branch; an extra dropped or reordered statement between the
   call and the tail; a claimed interior set without a Q12 closure; a
   re-entrancy argument without its guard anchor; a disabled/armed
   nondifference — all reject; the valid caller-side counterpart passes.
5. **Scope.** This form applies only to a caller-side post-call tail. Any
   row whose declared tail is inside or beyond an open dispatch still
   stops under Q12 rule 6. `CallBuiltinSpec` remains unadmitted and
   P0543/P0544 are unaffected.
6. **Stage.** No new allocation. P1577's plan proceeds under this
   topology and the 151-plan bar stands; if it fails any falsifier the
   family returns with its named block.

All other Q12, Q11, Q10 and brief terms stand: the 90-file starting set and
pre-edit pin authority, the caught/stop-row signature pins, 151 plans, ten
classes, 79 subtypes, zero unknown edges, deterministic byte-identical
double generation, the exact emitted file set, the mechanical
`source_schema` transition, all focused controls and both workspace modes
after the ordered replay.

## Continuation 5 ruling (director) — Q14: the argument-position lexical callable is the fourth declaration-authority form; `finish_builtin_dispatch`'s four function-item/closure bindings cover P0543/P0544

The STOP is correct under Q10 rule 4, Q11 rule 3 and Q12 rule 6 and earns
no credit. The Q13 work before it is accepted as delivered: P1577 now
passes with 17 caller-path anchors and 20 controls, the producer recorded
opaque with its full nested `Result` signature, the driver interior left
open at the four recorded sites, both retained arms reducing to the
original caller AST after removing only the declared producer scope and
restoration guard, and every Q13 falsifier firing. All 151 source joins,
the 90 preimages / 180 postimages and the 5,399 maintained files
authenticate; `source_schema` is unchanged at `f9604852…`; no repository
file was written; no plan or schema was emitted. The 126.48 s conservative
charge is accepted; 742's remaining focused budget is 4,048.95 s
(envelope 2 has 1,048.95 s), focused envelopes 3 and 4 and both workspace
envelopes are untouched.

The authority P0543/P0544 need is neither a route nor a trait
implementation. `impl FnOnce` parameters have anonymous, unnameable types;
each value is supplied at exactly one argument position of an in-source
call, and its body is fully visible there. `finish_builtin_dispatch`
(`eval_dispatch/mod.rs:223`, private, exactly two call sites at `:188`
and `:198`) takes `freeze: impl FnOnce() -> RunCallDispatchStageSuspensionFrame`
(`:229`) and
`finish: impl FnOnce(&mut Self, Result<Expr, EvalError>) -> Result<Option<Expr>, EvalError>`
(`:230`), and invokes them inside `resume_typed_or_suspend!` at `:233`-`:237`
as `ok(value) => finish(self, Ok(value))` and `suspend => freeze()`, with
`Err(error) => Err(error)` bypassing both. The four bound callables are
the function item `RunCallDispatchStageSuspensionFrame::unchecked_builtin`
(`:442`) and the closure `|_, result| result.map(Some)` in the unchecked
arm, and, in the checked arm, the frame-constructing closure calling
`::builtin` (`:451`) with `spec`/`args`/`projection_count_before` captured
and the validating closure calling `validate_builtin_memory_growth_slow`
and mapping the actual `result`. Each type has one definition site and
cannot be implemented anywhere else; the source literally shows the code
that runs. That is declaration authority of the strongest kind.

Q14 admitted, as the fourth evidence form:

1. **Argument-position binding.** A generic parameter whose type is
   unnameable or anonymous and whose value is supplied at exactly one
   argument position of an in-source call is declared by that argument.
   The summary records the callee and its signature anchors, the parameter
   position and type, the call site, and for a function item its
   definition anchor and body; for a closure its body operations and its
   exact capture set with the source binding of each capture.
2. **Invocation and branch enumeration.** Every invocation of each
   callable inside the callee is enumerated with its call anchor and the
   branch that reaches it. For `finish_builtin_dispatch` that is the
   macro's three arms: success invoking `finish(self, Ok(value))`,
   `InputRequired` constructing the frame through the actual witness via
   `freeze()`, and every other error bypassing both callables. A callable
   that escapes the callee — stored, returned, passed onward or
   type-erased — stops the family.
3. **Capture integrity.** The audit proves each captured value is the
   actual producer's binding (here `spec`, `args`, `projection_count_before`
   from the enclosing dispatch), not a reset, default or fabricated value,
   and that the function item resolves to the named definition.
4. **Preservation.** The row's carriers keep the real error/`None`
   branches: other errors bypass both callbacks, the `InputRequired`
   branch constructs the concrete frame, the `Some` conversion is the
   actual `result.map(Some)`, and no callable is duplicated or evaluated
   twice. P0543's unchecked bypass and P0544's statement guard are
   recorded as preserved carrier proof, not as an opening of
   `run_builtin_handler`'s interior.
5. **No widening.** This form covers only positionally bound, in-source
   callables. It does not admit `CallBuiltinSpec` metadata, trait-generic
   dispatch, erased or dynamic dispatch, a callable received from a
   generic caller, or any callable whose invocation set is not fully
   enumerated.
6. **Falsifiers.** A dropped `freeze` or resumed path; swapped
   checked/unchecked bindings; a fabricated `None`; a changed constructor,
   signature or typed output; a missing immediate invocation; an
   unrecorded capture; an escaping callable; a second binding site; a
   disabled/armed nondifference — all reject; the valid counterparts pass.
   The eleven existing controls are the minimum set.
7. **Stage.** No new allocation. P0543/P0544 proceed under this form and
   the 151-plan bar stands; a failure returns with the named block.

All other Q13, Q12, Q11, Q10 and brief terms stand: the 90-file starting
set and pre-edit pin authority, the caught/stop-row signature pins, 151
plans, ten classes, 79 subtypes, zero unknown edges, deterministic
byte-identical double generation, the exact emitted file set, the
mechanical `source_schema` transition, all focused controls and both
workspace modes after the ordered replay.

## Continuation 6 ruling (director) — Q15: `profile_mode::Mode` is admitted as the fourth closed protocol for C10, and the closure form is delegated to the generator under mechanical conditions and mandatory gate reporting

The STOP is correct under Q12 rule 6 and earns no credit. The Q14 work
before it is accepted as delivered: four argument-position bindings
resolved across two call sites, six capture-source anchors, 26 total
anchors, the success / actual `InputRequired` / error-bypass / resumed
typed-completion branches recorded, and 20 controls passing; the retained
P0543/P0544 carriers reduce to the original caller, helper, constructors
and typed frame after removing only their declared producer scopes and
omission guards. All 151 source joins, the 90 preimages / 180 postimages
and the 5,399 maintained files authenticate; `source_schema` is unchanged
at `f9604852…`; no repository file was written; no plan or schema was
emitted. The 123.39 s conservative charge is accepted; 742's remaining
focused budget is 3,925.56 s (envelope 2 has 925.56 s), focused envelopes
3 and 4 and both workspace envelopes are untouched.

`profile_mode::Mode` is the Q12 form exactly: `pub(super) trait Mode`
(`suspension/core/profile_mode.rs:8`), sealed to its module, with exactly
two implementations — `Ordinary` (`:15`; `Guard = ()`, no-op `resume`,
`abandon` and `pause`) and `Profiled` (`:25`; `Guard = Option<ContextGuard>`,
`resume`/`abandon` installing the profile, `pause` forwarding a present
guard). The driver selects the instantiation at runtime:
`drive_resuming_frames` (`core/driver.rs:5`) chooses
`drive_frames::<O, profile_mode::Profiled>` when `frames.is_profiled()`
and `::<O, profile_mode::Ordinary>` otherwise (`:15`/`:17`);
`M::resume` provides the guard at `:25` and `M::pause(&profile_context)` is
called only in the actual `Err(InputRequired)` branch at `:156`. The
omission the retained C10 carrier makes is inside that callee chain, not
after an unmodified producer call, so Q13's opaque-node exception does not
apply: `M::pause` -> `ContextGuard::pause`
(`profiling/logical/context.rs:390`) -> `Segment::pause`
(`segments.rs:536`) -> `pause_admitted` (`:554`) -> the
`ExplicitlyFinished` early return (`:563`) is anchored in full.

Q15 admitted:

1. **Closure.** The generator asserts the trait's declared visibility and
   the exact two-implementation set with each implementation's `Guard`
   type and `resume`/`abandon`/`pause` bodies. A third implementation, a
   visibility widening, or any changed body, associated type or signature
   stops C10 and every row bound to the protocol.
2. **Runtime branch.** The plan carries both mode arms with the
   discriminating condition (`frames.is_profiled()`), each with its own
   anchors; this row's frozen route fixes the profiled arm
   (`C10.ProfiledPreparedDispatchFrame.true`), and the ordinary arm is
   recorded but not claimed executed. Neither arm may be dropped.
3. **Guard and pause edges.** `M::resume`'s guard and the
   `Err(InputRequired)`-only `M::pause` call are both recorded; the
   callee chain to the `ExplicitlyFinished` return is anchored in full and
   may not be replaced by an opaque node.
4. **Both shapes preserved.** The distinct `Guard` types, the no-op
   ordinary hooks and the profiled guard forwarding stay distinct; no
   unified guard, no fabricated guard, no skipped real pause.
5. **Falsifiers.** Changed visibility, implementation set, `Guard`,
   `pause` body or mode instantiation; a missing runtime branch or generic
   edge; a wrong tail; a fabricated error; missing actual-pause arming; a
   disabled/armed nondifference; an unearned erased-frame closure — all
   reject; the valid source and retained-carrier counterparts pass.
6. **Standing closure admission (amends Q12 rule 6).** A protocol found
   later may be admitted by the generator without a further director
   ruling only when it satisfies Q12 rules 1-5 identically: visibility no
   wider than `pub(in crate::runtime)` / `pub(super)` / private within the
   runtime, an exactly asserted implementation set, every instantiation
   proved per call edge (or as an explicit runtime branch with both arms),
   all falsifier controls firing, and a gate report that names the
   closure, its anchors and its verdict lines. Anything that does not fit
   this form — a different binding shape, an open or dynamic payload, a
   tail inside or beyond an open dispatch, an instantiation not provable
   per edge, or a trait visible outside the crate's runtime — still stops
   the family and returns for a ruling. Erased frame protocols remain
   open, `CallBuiltinSpec` remains unadmitted, and no inventory entry is
   added.
7. **Stage.** No new allocation. C10 proceeds and the 151-plan bar stands;
   a failure returns with the named block.

All other Q14, Q13, Q12, Q11, Q10 and brief terms stand: the 90-file
starting set and pre-edit pin authority, the caught/stop-row signature
pins, 151 plans, ten classes, 79 subtypes, zero unknown edges,
deterministic byte-identical double generation, the exact emitted file
set, the mechanical `source_schema` transition, all focused controls and
both workspace modes after the ordered replay.

## Continuation 7 ruling (director) — Q16: a caller-side post-call tail needs activation scoping, not a non-reentrancy guard; reentrant activations and sibling branches that reach the same tail are admitted under per-activation episode identity

The STOP is correct under Q13 condition 2(d) as written and earns no
credit. The Q15 work before it is accepted as delivered: the exact
`Ordinary`/`Profiled` implementation set with distinct `Guard` types, both
runtime selection branches, actual `InputRequired` arming and the complete
`Mode::pause` -> `ContextGuard::pause` -> `Segment::pause` ->
`pause_admitted` chain, 24 path anchors, 24 controls, and both retained
C10 carriers preserving their producer and non-target nodes. All 151
source joins, the 90 preimages / 180 postimages and the 5,399 maintained
files authenticate; `source_schema` is unchanged at `f9604852…`; no
repository file was written; no plan or schema was emitted. The 127.04 s
conservative charge is accepted; 742's remaining focused budget is
3,798.52 s (envelope 2 has 798.52 s), focused envelopes 3 and 4 and both
workspace envelopes are untouched.

The guard requirement in Q13 condition 2(d) was a sufficient condition,
not a necessary one. For C3 the tail at `eval_core/call.rs:188` is
reached in two ways inside `Session::evaluate_call` (`:157`): the
`StableSymbol`/`Atom` arm calls `resume_call_after_head`, the
`UnstableSymbol`/`Call` arm calls the declared producer
`evaluate_unstable_call_head` (`:179`), and both converge on the
total-profile tail after the match. Re-entry through `evaluate`
(`resolution.rs:325`), the unstable pass (`:378`), the accounting-mode
selection (`eval_core/mod.rs:23`/`:25`) and the `Expr::Call` branch
(`:152`) creates further activations of that same site set; it does not
create a path to the tail that bypasses the producer call inside an
activation. What re-entry does break is episode identity: a nested
activation's tail could see an outer activation's arm. That is a scoping
requirement, and it is checkable, whereas a non-reentrancy guard does not
exist in this caller and may not be fabricated.

Q16 admitted, superseding Q13 condition 2(d) with an activation-scoping
rule:

1. **Activation scoping.** A caller-side post-call tail does not require
   an entry guard against re-entry. A nested activation is the same
   enumerated site set one level down, not a new edge. The plan records
   the recursion explicitly and proves that, within every activation, the
   tail is reached only through that activation's producer call or a
   declared sibling branch.
2. **Sibling branches.** Every branch that reaches the tail is
   enumerated and marked with whether it passes through the producer call.
   A non-producer branch executes the tail unarmed, as the ordinary
   statement, and may neither consume nor clear an episode.
3. **Per-activation episode identity.** The arm, episode, occurrence and
   hit identities are keyed to the activation's own stored or frame
   identity, never to a global, static or per-session flag. The plan must
   prove: an outer armed episode is not cleared or consumed by a nested
   activation; a nested activation's episode neither arms nor hits the
   outer tail; after the nested activation returns, the outer episode
   still hits its own tail; and two nested episodes are attributed
   separately. Falsifiers: a shared token, a nested hit attributed to the
   outer activation, an outer episode lost after nesting, and a
   sibling-branch tail consuming an episode all reject.
4. **Open payload stays opaque.** The producer interior — here
   `run_builtin_handler` (`eval_dispatch/mod.rs:317`) and the public
   `BuiltinFn` (`metadata.rs:332`, `register_builtin` at
   `session_api.rs:349`/`:365`) — is one opaque node with its signature.
   No handler target set, whitelist, fabricated guard, production guard,
   runtime accessor or inventory entry is added. Re-entry through it is
   recursion of the enumerated caller, not a new edge.
5. **All other Q13 conditions stand.** The producer call runs in full with
   unchanged arguments and exactly one evaluation; every return variant
   and caller branch to the tail is anchored; nothing between arm and
   tail is changed beyond the declared omission; the credited producer
   pauses and the real error/`None` branches are preserved; no interior
   claim is made.
6. **Falsifiers.** The fourteen controls plus the per-activation set
   above. P1577 remains the guarded counterpart and must still pass;
   removing its guard must still reject. The reentrant counterpart must
   pass with per-activation identities, and a shared-token variant of it
   must reject.
7. **Stage.** No new allocation. C3 proceeds under this rule and the
   151-plan bar stands; a failure returns with the named block.

All other Q15, Q14, Q13, Q12, Q11, Q10 and brief terms stand: the 90-file
starting set and pre-edit pin authority, the caught/stop-row signature
pins, 151 plans, ten classes, 79 subtypes, zero unknown edges,
deterministic byte-identical double generation, the exact emitted file
set, the mechanical `source_schema` transition, all focused controls and
both workspace modes after the ordered replay.

## Continuation 8 ruling (director) — Q17: the activation transport is instrumentation-owned and source-anchored; the retained shared-bit helper is replaced as generated output under the existing pins, and the episode identities must be carried from the actual execution

The STOP is upheld and earns no credit. The Q16 work before it is
accepted as delivered: 42 authenticated caller-topology anchors covering
both arms, the producer's real success / `InputRequired` / other-error
branches and the recursion recorded as recurrence of the same site set;
the public `BuiltinFn` interior kept opaque; P1577 still green with its
17 anchors and two real guard anchors, and removing its guard still
rejects; the 17-control scoped-token model; the two calibrated
counterexamples against the retained helper bytes; and the three
typed-handoff anchors (`EvaluateCallSuspensionFrame` stores
`args_for_resume`, resumes through
`session.resume_call_after_head(value, args_for_resume)`, and its
`ContinueToken` is an invariant `PhantomData` proof, not a stored runtime
key). All 151 source/tail/destructor joins, the 90 preimages / 180
postimages and the 5,399 maintained files authenticate;
`production_changes=0`; `source_schema` is unchanged at `f9604852…`; no
plan or schema was emitted. The 126.11 s conservative charge is accepted;
742's remaining focused budget is 3,672.41 s (envelope 2 has 672.41 s),
focused envelopes 3 and 4 and both workspace envelopes are untouched.

The retained `cm740_probe` — one thread-local `ARMED` bit, `ordinary` and
`frame` reading it globally, and `pause` arming whenever any enclosing
entered row matches — is exactly the "one boolean across the pair"
weakness the accepted design already names (`.temp/brief-741/PLAN.md:42`,
citing the armed `rum-runtime/src/lib.rs:39`). The two counterexamples
demonstrate it: an outer pause arms an unpaused nested activation, and a
nested pause arms the outer activation. That shape cannot satisfy Q16
rule 3. It is also not a preserved node: the module exists only in the
mutated builds (`// Copied mutation attribution, absent from every live
runtime build.`, `#[doc(hidden)] pub mod cm740_probe`), is absent from
production, and its text is generator output the stage must render from
recipes. The 18 caught / 4 stop pins bind the omission site, the
preserved nodes, the arming event, the hit phase, the selector and the
predicate — not the helper's internal storage — so replacing the helper
is in scope, with the `lib.rs` overlay's byte delta disclosed per file.

Q17 admits the activation transport under these conditions:

1. **Scope by the activation's own guard.** The instrumented source's
   `enter`/`Guard` placement is the activation boundary. An arm, a
   lookup, a hit or a frame check is answered by the entry that owns that
   activation: for `pause`, the innermost enclosing entry whose row is
   the selected row; for `ordinary`, `frame` and `hit`, the current
   activation's own entry. No process-global, per-session,
   per-row-unscoped or ambient-stack-wide flag may answer for another
   activation. The two retained counterexamples are rejection controls on
   the rendered helper: the same nested sequences must yield the isolated
   answers (an ordinary lookup in an unpaused nested activation is false;
   a nested pause does not arm the outer activation), and a regenerated
   shared-bit variant must fail.
2. **Episode transport from the actual execution.** The token carries the
   row, root program, constituent, run episode, producer occurrence and
   the activation's own identity. Program, constituent, run episode and
   occurrence come from the actual run — the runner's existing
   attempt/program/constituent identity and the actual producer
   occurrence — never from the plan, the renderer or a synthetic model
   value, and never from a prompt string. The resumed activation is
   matched to the episode that armed it using only those identities and
   the declared resumed-then-queued order, and the episode is cleared at
   a new independent episode. Falsifiers: a source-row string alone, a
   boolean, a renderer-supplied constant or synthetic identity, a token
   matched by position without an identity, a cross-program or
   cross-constituent match, a stale episode consumed twice, and two
   occurrences collapsing into one.
3. **The model is not the proof.** The 17 model controls must be re-run
   against the rendered helper and its source-bound binding; a
   model-only pass is not evidence for the transport. The rendered
   helper's control set includes the two retained cases, the Q16 rule 3
   set and the typed-handoff inheritance above.
4. **No production surface.** The helper stays test-only and
   mutated-build-only; the single compile-time enable constant remains
   the only disabled/armed difference; no production call site, runtime
   accessor, predicate, permission, dependency, inventory entry or
   `unsafe` is added, and the disabled arm's complete record sequence
   keeps equalling real.
5. **Retained-row non-dependence.** For each retained caught/stop row the
   plan must show from the rendered source that the omission predicate is
   satisfied by that row's own activation, with no cross-activation
   leakage. A retained row that cannot satisfy this stops and returns
   with its name. The final-epoch re-earn and its STOP on signature drift
   are unchanged.
6. **Stage.** C3 proceeds only after the transport is source-bound and
   audited, and the remaining all-row generator work continues from the
   same partial tree. No new allocation. The 151-plan bar, the 90-file
   starting set, the mechanical `source_schema` transition,
   deterministic double generation, the focused controls and both
   workspace gates stand.

All other Q16, Q15, Q14, Q13, Q12, Q11, Q10 and brief terms stand.

## Continuation 9 ruling (director) — Q18: the actual execution identity enters through the existing copied runner hooks, and the run episode is the actual pair occurrence at that boundary; no worker hook and no attempt-name channel

The STOP is upheld and earns no credit. The Q17 work before it is
accepted as delivered: the scope interpretation is correct — Q17 admits
the generated helper's internal storage but not an unadmitted hook in a
protected body — and the audit is right that all five copied `begin`
sites transmit only `queued`. The 22-anchor ingress audit and its 12
controls pass, the inherited 36 controls pass, and the finding honestly
labels its own evidence limits: no rendered source-bound helper was
produced, the 17 model controls were not re-run against one, and no
all-row zero-unknown-edge verdict is claimed. All 151 source joins, the
90 preimages / 180 postimages and the 5,399 maintained files
authenticate; `production_changes=0`; `source_schema` is unchanged at
`f9604852…`; no plan or schema was emitted. The 124.03 s conservative
charge is accepted; 742's remaining focused budget is 3,548.38 s
(envelope 2 has 548.38 s), focused envelopes 3 and 4 and both workspace
envelopes are untouched.

The identity must come from the runner, and the runner already has it at
the exact boundary where the helper must be keyed. The four own-row
hooks all take the actual program: `guest::run_observed` (`program:
&Program`), `guest::run_adapter`, `native::run_accepted_request` and
`native::run_resuming_driver`; the fifth, `semantic_controls::run`, has
its own `Plan` with an `id`. `Program.id` is constructed by
`paired_program` as `{primary.id}:paired:{constituent id}` and keeps the
root prefix, so one opaque value carries the root/constituent join. All
five files are already among the 90 retained overlays, and erasing the
hook calls restores the live runner AST in both arms. No new file, no
`source_worker.rs` change and no attempt-name ingress is needed, and none
is authorized.

Q18 resolves the ingress under these conditions:

1. **Ingress.** `begin` carries the actual identity the runner is
   executing: `program.id` at the four own-row hooks and `plan.id` at the
   structural control, passed as opaque runtime values. A literal, a
   renderer- or plan-supplied constant, a row or prompt string, the
   selected-row environment value or the build receipt may not stand in
   for it. The helper may not parse the identity into narrower fields;
   it keys on the whole value.
2. **Episode.** The independent run episode is the actual pair occurrence
   at the same boundary: each unqueued `begin` for a (selected row,
   identity) opens an episode, the following queued `begin` with the same
   identity is its pair, and a further unqueued `begin` with that
   identity opens a new episode and clears any unpaired one. The
   derivation is an execution fact of the runner's own sequence, not a
   global position, a timer, or a plan value; it must be stated as an
   equivalence rule and controlled. The attempt dimension is carried by
   the process/thread in which the helper's ledger starts empty; if the
   generator can show one process carrying two attempts for one row, it
   stops and says so.
3. **Producer occurrence.** Counted at the actual `pause`/`returned`
   sites inside the episode, not from the plan or the pause tuple.
4. **Falsifiers.** `begin(queued)` alone must fail the source-binding
   checker; a literal, renderer-supplied, row or prompt identity must
   fail; two independent pairs of the same program must keep their
   episodes distinct, with the second queued run unable to consume the
   first pair's arm; a queued run with no unqueued predecessor must not
   hit; interleaved different programs and a constituent chain must not
   cross-consume; an unpaired or stale episode must not survive into a
   new pair; the disabled arm stays record-exact.
5. **Scope.** The 90-file set stays exact. `source_worker.rs`, observer
   policy, `records.rs`, `observer_receipts.rs`, `receipts.rs` and the
   semantic acceptance/control bodies stay unchanged; the new identity is
   generated instrumentation inside files already in the set. Q17's
   helper replacement, guard-owned activation identity, rendered-helper
   control re-run, no-production-surface and retained-row
   non-dependence conditions stand unchanged.
6. **Stage.** No new allocation. C3 proceeds only after the transport is
   source-bound and audited under Q17 and this rule, and the remaining
   all-row generator work continues from the same partial tree.

All other Q17, Q16, Q15, Q14, Q13, Q12, Q11, Q10 and brief terms stand.

## Continuation 10 ruling (director) — Q19: an episode is inherited by every suspension frame it actually constructs; P0492's one-arm/two-hit pin stands, the single producer pause remains the only arm event, and a second `returned` arm is not authorized

The STOP is upheld and earns no credit. The Q18 work before it is
accepted as delivered: the four own-runner hooks now carry
`begin(queued, &program.id)` and the structural hook `plan.id`; the
rendered helper core has activation-owned tokens, opaque program
identities and the Q18 pair-occurrence rule; producer occurrence is
counted at actual `pause`/`returned` calls; two roots produce identical
bytes and disabled/armed differ only in the compile-time constant; the
17 former model obligations plus nine new controls execute the generated
helper bytes (52 tests), and the retained aliasing counterexamples still
reject a deliberately shared-token variant; the typed-adapter binding
adds no field and keeps both allocation counts; 15 ingress, 8
frame-binding, 5 source-premise and 8 nested controls pass. The two
initial scratch failures and their corrections are retained and are
correctly classified as instrumentation errors, not source or inventory
regressions. All 151 source joins, the 90 preimages / 180 postimages and
the 5,399 maintained files authenticate; `production_changes=0`;
`source_schema` is unchanged at `f9604852…`; two-root partial generation
is byte-identical over the five partial targets; no complete plan or
schema was emitted. The 130.89 s conservative charge is accepted; 742's
remaining focused budget is 3,417.49 s (envelope 2 has 417.49 s),
focused envelopes 3 and 4 and both workspace envelopes are untouched.

The P0492 trace is verified: the second constituent is
`Return[Accumulate[Input["b724tuple"]]]`, both `Return` and `Accumulate`
are ordinary catalog entries without a hold attribute, so Return's
preparation activation encloses Accumulate's while Input requests the
answer; the single pause arms once, and each of the two prepared frames
resumes once through `resume_prepared_call_dispatch_frame::<true>`,
whose omission site the carrier changes. The retained signature — one
`CM740_SELECTOR_ARMED … phase=producer-InputRequired` and two
`CM740_SOURCE_HIT … phase=OmitEnabledPreparedStageCompletion`, with
`Err(Native)` and equal public output — is therefore the signature of one
suspension that constructs two frames. The strict innermost-only reading
suppresses the outer frame's genuine execution of the same omitted line
in the same episode; it makes the mutant's fidelity depend on the
helper's arming rather than on the declared operation. The shared bit is
still rejected, and no second arming event may be invented.

Q19 resolves the boundary; the pin stands unchanged:

1. **Frame inheritance.** An episode is created by the actual producer
   pause and belongs to the suspension, not to one activation. Every
   suspension frame the episode actually constructs inherits a copy of
   its token at that frame's construction site, keyed to the frame's own
   activation identity. The arm event remains the single actual pause; no
   `returned` arming event is added for P0492 and no retained signature
   may gain an arming line.
2. **Several frames per episode.** One episode may construct several
   frames; each inherits its own copy, and each resumed frame that
   executes the omission records a hit attributed to its own activation.
   P0492's one-arm/two-hit signature is exactly this and stands; the
   first constituent's one-arm/one-hit signature stands as well.
3. **Inheritance is not leakage.** The copy travels only at the actual
   frame-construction site inside the armed episode. No ambient, shared,
   static or per-session state; an activation that did not suspend in the
   episode never answers true; the queued paired run inherits only
   through its own frames; a new episode clears stale copies; the binding
   is retired when the frame is erased. A hit attributed to an activation
   other than the frame that executed the omission still rejects.
4. **Placement.** The generated arm scope must cover the actual
   frame-construction site so the frame is constructed with its own
   activation alive; the naive capture at frame erasure captures the
   dropped Guard's parent and is a placement error, not a semantics
   choice. Erasing only the declared instrumentation must still restore
   the live AST, and the omission site, preserved nodes, phase, selector
   and predicate are unchanged.
5. **Counterexamples re-expressed.** The forbidden cases are: an
   activation that did not suspend answering from the episode (no frame
   constructed from it, including a swallowed suspension and a queued-side
   activation without its own paired frame); a shared or static token; a
   nested activation's episode hitting an activation that did not suspend
   in it; a stale episode surviving into a new pair; a hit attributed to
   an activation other than the frame that executed the omission. The
   retained pure controls are updated to model the chain, the P0492
   one-arm/two-hit trace becomes the positive control, and an unarmed
   two-frame chain must yield zero hits.
6. **Per-row proof.** Q17 rule 5 now reads: for each retained row, every
   pinned arm and hit must map to that row's own episode and to the
   frame(s) that episode actually constructed, with no dependence on an
   activation outside the chain. A retained row whose pin cannot be
   reproduced this way stops with its name; it is not normalized and is
   not given a second arm event.
7. **Stage.** No new allocation. C3 and the remaining rows proceed only
   after the frame-inheritance transport is audited, and the 151-plan bar
   stands.

All other Q18, Q17, Q16, Q15, Q14, Q13, Q12, Q11, Q10 and brief terms
stand.

## Continuation 11 ruling (director) — Q20: an initial-return tail is owned through the actual returned `InputRequired`, with the episode token travelling along the actual return edges; P0508's one-arm/two-hit pin stands and no resume scope is required

The STOP is upheld and earns no credit. The Q19 work before it is
accepted as delivered: the P0492 producer Guard now encloses its actual
`InputRequired` branch and the unchanged witness/frame constructor, each
actual frame's own activation receives the token at construction, the
audit erases the new instrumentation to the live AST and compares the
omission method to its pin, and 13 placement/preservation controls pass
with eight anchors. The frozen `Return[Accumulate[Input["b724tuple"]]]`
trace yields one arm and two hits on two activations; a single-frame
chain yields one; an unarmed two-frame chain and the disabled arm yield
zero; swallowed, missing, retired or stale bindings, wrong-frame
attribution and both retained aliasing counterexamples reject; the
shared-token variant still fails isolation; and the 15 nested controls
pass against the latest helper. The four lazy callback rows
P2132/P2133/P2134/P2135 are resolved through the existing
`Box<AwaitCursor>` allocation without a new field, allocation change,
witness, error, phase or selector change, with 27 anchors and 15
controls; the latest helper has 70 controls. All 151 source joins, the
90 preimages / 180 postimages and the 5,399 maintained files
authenticate; `production_changes=0`; `source_schema` is unchanged at
`f9604852…`; nine partial targets render byte-identically from two roots
and two helper versions are disclosed separately; no complete plan or
schema was emitted. The 138.27 s conservative charge is accepted; 742's
remaining focused budget is 3,279.22 s (envelope 2 has 279.22 s),
focused envelopes 3 and 4 and both workspace envelopes are untouched.

P0508's frozen source `7+(Input["finish"])` executes two
`evaluate_once_in_accounting_mode` levels, each with the producer `enter`
around `evaluate_inner_unstable` and the omission immediately after an
actual `Err(InputRequired)` return, before `leave_level`, the entry pop
and the shadow-stack drop; the pin is one arm and two same-phase hits
with `Err(Native)` and exact public output. The site is an initial-unwind
tail, not a resume method, and the frames the suspension constructs
appear only at the callers' suspension branches, after the tail has
executed. A resume-scope lookup therefore cannot reproduce the pin — and
the superseded shared bit could only by answering from ambient state.

Q20 resolves it. The episode belongs to the suspension (Q19), and the
suspension has two actual edges; inheritance travels along both:

1. **Return-edge inheritance.** An activation whose own producer call
   actually returned `InputRequired` inside the armed episode inherits
   the episode. Eligibility at an initial-return tail is owned by that
   activation: the generated site checks its actual returned result and
   the current activation's own inherited token. No resume scope and no
   frame-resume fiction.
2. **Propagation.** The token travels from the armed activation along
   the actual return edges: each enclosing activation whose producer
   returns the propagated `InputRequired` receives its own copy at that
   edge, so each initial-return hit is attributed to the activation that
   executed its own tail. The propagation is silent — no arming line and
   no `returned` event — so P0508's one-arm/two-hit pin stands unchanged.
3. **Not leakage.** Eligibility requires the armed episode in this run,
   the site's actual `InputRequired` return, the unqueued side and a
   token received along an actual return or construction edge. No
   ambient, shared, static or per-session state; a swallowed or consumed
   suspension never qualifies an enclosing activation that does not
   itself return `InputRequired`; the queued side never qualifies; a new
   episode clears stale tokens; the transport retires with the
   activation; and a hit attributed to another activation rejects.
4. **Both edges together.** Q19's frame-construction inheritance remains
   for resumed executions and this rule covers initial tails. A row may
   use either or both, and Q17 rule 5's per-row proof now reads: every
   pinned arm and hit maps to the row's own episode through its actual
   return edge or its actual constructed frame, with no dependence on an
   activation outside the chain.
5. **Controls.** A two-level chain yields one arm and two hits; a
   one-level chain one and one; unarmed or disabled yields zero; the
   queued side yields zero; a swallowed suspension leaves the outer
   activation ineligible; a fabricated second suspension, a stale
   episode, a retired activation and a misattributed hit reject. The
   P0508 trace becomes the positive control and the P0492 controls stay.
6. **Stage.** No new allocation. P0508 proceeds only after the
   return-edge transport is audited, and the remaining rows and the
   151-plan bar continue from the same partial tree.

All other Q19, Q18, Q17, Q16, Q15, Q14, Q13, Q12, Q11, Q10 and brief
terms stand.

## Continuation 12 ruling (director) — Q21: a queued-side terminal tail is owned through the Q18 pair's actual request-level suspension return; the pair carries the unqueued occurrence's verified return proofs to its queued occurrence's matching activation, and no descendant frame is attributed to the request caller

The twelfth run executed Q20. P0508's generated guard spans the actual
`evaluate_inner_unstable(expr)` call and the pinned initial omission;
`Guard::input_return` borrows the actual `Result`, consumes only that
activation's pending proof, qualifies only its own unqueued
`InputRequired`, transfers the token silently to the enclosing activation
when the actual edge returns, and records no second arming event. The
independent source audit uses the entry-pinned live source and the
retained omission, not the renderer's assertion: one producer invocation,
the exact original result and error, the exact selector, phase and
early-return operation, unchanged depth, stack, shadow-stack destructor
and non-target finalizers, and erasing the declared instrumentation
restores the whole live AST; two roots emit byte-identical partial files.
Nine source anchors and 20 source controls pass (fabricated `None`/error,
duplicated producer, wrong owner, missing edge, shifted destructor, extra
arm, resumed-scope substitutions). The generated helper passes 114
controls (70 inherited, 44 return-edge), and nine further controls
execute the actual statement extracted from the audited generated P0508
source against a pure recursive producer: one arm and two own initial
hits, one and one, unarmed/disabled/queued zero, and a swallowed
suspension leaving the outer tail unqualified. Q19's controls re-run
intact (nested 15, ingress 15, adapter 8, prepared 13, lazy cursor 15,
retained source 5; P0492 keeps one arm and two frame hits); no field,
allocation count, runtime interface, witness, guard, phase or selector
changed; both historical P0508 mode signatures authenticate and are not
re-earned.

The stage then stopped at P1562/P1563 with
`PortBlocked::QueuedCallerReturnTransport`. Both retained omissions are
the queued-side `ordinary(row)` terminal tail — `return result` before
`finish_accepted_value_request` — in `eval_request_admitted` and
`eval_expr_request_admitted`. Their producer
`eval_expr_request_from_checkpoint` returns
`Result<SessionEvalResponse, SessionRequestFailure>`; its actual
`EvalError::InputRequired` is converted by
`finish_input_required_request` into
`SessionRequestFailure { error: SessionRequestError::InputRequired { request }, .. }`
and returned to that caller, while the unqueued resumption goes through
`resume_eval_request_with_input`/`resume_eval_request_from_checkpoint`
and never re-enters the initial caller's tail. The caller bodies contain
no direct suspension-frame construction; the real interior frame is the
`EvalInputSuspensionFrame` that the descendant `eval_input` constructs in
the unqueued occurrence, and the queued occurrence constructs no frame at
all, because `drive_input_request` consumes the queued answer and returns
without suspending. The pin in both modes is one arm
(`producer-InputRequired`, prompt `"finish"`) and one queued hit
(`OmitAcceptedRequestTerminalRetirement`), predicate `Err(Native)` with
the public output exact, and the current helper's queued eligibility
requires an own constructed-frame token at the activation path, which the
request caller never has: widening the caller guard, a stronger pure
return control, or an unbound descendant frame each yield zero queued
hits while the pin requires one, and an explicit owned-frame helper
counterpart passes only as a synthetic control.

Q21 rules that the Q18 pair is one attempt spanning two actual executions,
and that the episode's ownership evidence crosses the pair boundary at
exactly one point, the queued `begin`, carrying the unqueued occurrence's
verified return proofs together with its constructed-frame tokens:

1. **Return evidence at the request boundary.** The request producer's
   suspension-flavored return is `SessionRequestFailure` whose `error` is
   `SessionRequestError::InputRequired { request }` — the exact value
   `finish_input_required_request` constructs and returns. An activation
   whose own producer call actually returned that value holds a return
   proof, exactly as Q20's `EvalError::InputRequired` return proof is held
   at the eval boundary. The proof is read from the actual returned value
   (the variant itself), never from a boolean, a flag, the
   pending-request slot or the renderer's assumption; a success or any
   other error yields no proof.
2. **Pair transport.** The unqueued occurrence's return proofs, like its
   constructed-frame tokens, are retained on the pair and joined to the
   queued occurrence at `begin(queued=true)` for the same Q18 identity. A
   proof is issued to the activation whose own source path recorded it and
   to no other; the queued occurrence's matching activation is that
   same-path activation under the established activation identity. The
   transport is silent — no arming line, no `returned` event, no second
   pause — so the pinned arm stays the unqueued occurrence's actual
   producer pause and the pinned hit stays the queued occurrence's
   declared omission.
3. **Queued-side eligibility.** A queued-side (`ordinary`) site is
   eligible only when its own activation in this queued occurrence holds a
   pair-issued token from the same episode — a constructed-frame token at
   its own path, as before, or a return proof at its own path — and the
   site is the row's declared omission. The token's kind is part of the
   proof: the per-row audit must name, for every pinned queued hit, which
   actual edge from the unqueued occurrence owns it, and a row may not
   silently switch kinds. A queued activation with no token, or a token
   from another path, another episode, another row or another pair, is
   ineligible.
4. **What is not authorized.** Attributing the interior
   `EvalInputSuspensionFrame` or any other descendant frame of the
   unqueued occurrence to the request caller's activation is a false
   ownership edge: the queued occurrence constructs no frame, and Q19
   already rejects a hit attributed to a frame the activation did not
   construct. Widening the caller guard, allocating a descendant frame
   without a construction binding, a producer-wide constructor scope, or a
   return token made globally sufficient for queued `ordinary` are not
   substitutes and stay rejected. The pair transport is one-directional
   (unqueued to queued), never re-arms, and never makes `frame`
   (resume-scope) eligibility true; Q20's "the queued side never
   qualifies" remains in force for in-run return-edge inheritance and is
   not reversed here.
5. **Falsifiers.** A queued run with no unqueued predecessor yields zero;
   an unqueued producer that returned `Ok` or another error (a swallowed
   or consumed suspension) yields no proof and zero; a wrong program
   identity, row or episode yields zero; a second queued run for one
   unqueued occurrence yields zero; a proof at a descendant or sibling
   path used for the caller rejects; a fabricated boolean, flag or
   renderer-supplied value in place of the actual returned value rejects;
   the pinned one-arm/one-hit signatures of P1562 and P1563 must reproduce
   in both modes, and any extra arm, extra hit, changed phase or changed
   predicate is a STOP.
6. **Scope and stage.** No production, protected
   runner/observer/predicate, inventory, manifest or dependency change;
   `session_requests.rs` stays one of the 90 retained overlays, the helper
   remains generated test-only output, and erasing the declared
   instrumentation restores the live AST in both arms. No new allocation.
   P1562/P1563 proceed only after the return-proof pair transport is
   source-bound and audited; the remaining rows and the 151-plan bar
   continue from the same partial tree; a row whose pinned hit cannot be
   mapped to an actual edge is named and stopped, never normalized.

All other Q20, Q19, Q18, Q17, Q16, Q15, Q14, Q13, Q12, Q11, Q10 and brief
terms stand.

## Continuation 13 ruling (director) — Q22: C10's completed-generation pause is owned by the `Mode::Guard`'s own `Segment` object; the token is bound at that segment's construction, proven by the frame's actual returned `InputRequired`, and consumed only by the same segment identity — the shared bit, the typed frame and the new suspension frames are all rejected carriers

The thirteenth run executed Q21. Each request row now has one source
recipe: the guard encloses the exact original result statement, the
borrowed actual `SessionRequestFailure` return and the pinned terminal
omission; the helper keeps request-return proofs in a separate pair map,
only a source-bound `ordinary_request` omission can use that kind, and
`ordinary`/`frame` still require a constructed-frame token. A queued
success preserves its predecessor's proof; a new episode, another source
path, program, row or constituent, a success/other-error predecessor, a
swallowed request or a second queued run cannot qualify, and no
descendant frame is attributed to the caller. The entry-pinned audit
checks the live source and the retained omission (26 anchors, 28
controls; exact producer once, actual errors/results, exact
phase/selector, unchanged finalizer and destructor boundary, full-file
AST restoration after instrumentation erasure), two roots emit identical
partial `session_requests.rs`, and the generated helper passes 170
controls; 12 controls per request row execute the extracted audited
statements and each yields one producer arm and one queued hit, with the
disabled fragment records equal to the live statements. The inherited
bindings replay (15 ingress, 8 typed adapter, 13 prepared, 15 boxed
cursor, 5 retained source, 20 P0508 source, 9 P0508 extracted-statement,
15 P0492 nested; P0492 keeps one pause and two own-frame hits, P0508 one
pause and two initial-return hits). All 151 source/tail/destructor
joins, 90 preimages/180 postimages and 5,399 maintained files
authenticate; `production_changes=0`; `source_schema` stays
`f9604852…`; eleven partial targets render byte-identically from two
roots; no complete plan or schema was emitted.

The stage then stopped at `C10.prepared.profiled_consumer` with
`PortBlocked::ProfileGuardReturnTransport`. The retained C10 omission is
`Segment::pause_admitted`'s `ExplicitlyFinished` early return
(`profiling/logical/segments.rs:563`), the pinned signature is two arm
lines (the actual producer pause `producer-InputRequired` prompt
`"finish"`, and `profiled-consumer-InputRequired` emitted right after
`ProfiledPreparedDispatchFrame::resume`'s producer returns
`Err(EvalError::InputRequired)`, `eval_core/call.rs:816`), one hit
(`completed-generation-inert-pause`) and `Err(Native)` with the public
output exact. When the early return is skipped, `pause_admitted` falls
into the active-stack block and faults on
`stack.top != Some(self.index)`, which is the native difference the
predicate observes. The actual lifetime order is: `drive_frames` calls
`M::resume` at the top of the iteration and creates the guard
(`profile_mode::Profiled` → `ProfileQueue::enter_front_with_control` →
`Context::enter_with_control` → `segments::restore_admitted` →
`install`); the typed frame resumes, `TypedAction::complete(result)`s,
and is dropped with its queue owner before `M::pause(&profile_context)`
runs on the surviving guard; `ContextGuard { segment: Option<Segment> }`
is a private field and `ContextGuard::pause` calls that segment's
`pause`. The hit's segment is `ExplicitlyFinished` because
`segments::finish_active` removed its active frame and set the state on
the shared completion; the retained carrier's thread-local
`CM740_C10_ARMED` bit is consumed by whichever explicitly-finished
segment pauses first, and the two-completed-segment control shows the
unrelated segment consuming it and the intended one seeing zero.

Q22 rules that the owner is the `Mode::Guard` instance of the resumption
step, and the carrier is the `Segment` object that guard constructed:

1. **Owner object.** The row's operation executes on the `Segment` the
   `Mode::Guard` owns. The owner identity is the segment's own
   construction identity — the epoch id, the active-stack index and the
   unique `SegmentId` (`SegmentId(epoch::next_id())`, never reused) — and
   its completion, never a name, a counter, a variant, the guard's
   `Context` (an `Arc` shared with the queue cell) or the executing
   frame. A `Segment` value duplicated for the same identity (the
   `finish_active` token) is not a second owner and its retirement must
   not consume or retire the guard's token.
2. **Evidence.** The token is issued only when the profiled consumer's
   own producer actually returned `Err(EvalError::InputRequired)` — the
   exact predicate that emits the pinned second arm line — and it is
   issued to the segment constructed by the `Mode::resume` of that same
   resumption step. The guard-construction call
   (`ProfileQueue::enter_front`/`enter_front_with_control`) is the actual
   source edge from the closing `Mode` protocol to the segment, and
   `Mode::resume` is its only production caller; the binding must work
   whether or not the segment is still present in the active stack at
   the producer's return, because the completion that marks it
   `ExplicitlyFinished` may already have run. An identity recovered only
   from the active-stack top at the producer's return is therefore not
   the rule.
3. **Consumption.** Only `Segment::pause_admitted` on that exact identity
   may consume the token, only in its `ExplicitlyFinished` branch, and at
   most once. A step's unused token is cleared at the next
   `Mode::resume`/`Mode::abandon` construction on that thread; a token
   never survives its step.
4. **Rejected carriers and rules.** The thread-local armed bit (its
   isolation counterexample is the failure this rule exists to
   prevent); the typed frame or adapter (they retire before the pause and
   carry no guard identity); the new suspension frames (different
   objects and records, Q19's); a "first completed segment" or "most
   recent pause" rule; the guard's `Context`; any ambient, shared or
   static state; any use of `path.state.active` as a proxy without the
   object identity check. No field, accessor, production surface or
   allocation change is authorized, `driver.rs`, `profile_mode.rs` and
   `context.rs` stay outside the exact 90-file overlay set and
   uninstrumented, no new overlay file is authorized, and the binding
   must be built from `segments.rs`'s construction/finish/end/pause
   sites, `profile_queue.rs`'s guard-construction call and `call.rs`'s
   producer-return check.
5. **Falsifiers.** An unrelated explicitly-finished segment paused first
   takes the live early return and records no hit, and the intended
   segment still records its one; the same segment paused twice records
   one hit; a producer that returned `Ok` or another error issues no
   token, and a fabricated `true`, a boolean, a flag or the pending slot
   in place of the actual returned value rejects; a resumption step whose
   guard has no segment (`enter_control`) inherits nothing from a
   previous step; the abandon path's guard supplies no resume token; a
   token bound to a nested `Call` segment, to a newly constructed
   suspension frame, to another epoch or another thread rejects; an
   identity reused after retirement (impossible by `SegmentId`, and
   asserted) still records no hit; a token that survives into the next
   step is dropped and unusable. Both historical modes must reproduce
   two arms, one hit, the phase, the selector, `Err(Native)` and the
   exact public output, and the queued run must stay at zero arms and
   zero hits; any extra arm, extra hit or changed line is a STOP.
6. **Stage.** No new allocation. C10 proceeds only after this
   guard-segment identity transport is source-bound and audited in both
   modes; the remaining rows and the 151-plan bar continue from the same
   partial tree, and a row whose pinned hit cannot be mapped to an actual
   object identity is named and stopped, never normalized.

All other Q21, Q20, Q19, Q18, Q17, Q16, Q15, Q14, Q13, Q12, Q11, Q10
and brief terms stand.

## Continuation 14 ruling (director) — Q23: the J class's deferred publication belongs to the nested suspension the driver actually appends; its `SuspensionGeneration` — assigned by the install, read from the object at its own `append` — carries the token, and only that suspension's append scope may own the publication

The fourteenth run executed Q22. The guard-Segment transport is now
source-bound: the token records the exact epoch, active-stack index,
`SegmentId`, completion and thread at the successful `Segment`
construction under `ProfileQueue`'s resume guard construction, the
producer `Guard` captures the step id at entry so a later step cannot
receive an earlier producer's proof, only the borrowed actual
`Err(EvalError::InputRequired)` from that activation issues the token,
`Segment::pause_admitted` consumes it once in its `ExplicitlyFinished`
branch, the temporary `finish_active` `Segment` shares the identity
without retiring the guard's token, and the guard end/drop, the next
resume/abandon construction and a new run retire it. The pre-edit AST
audit has 47 anchors and 29 controls; the rendered helper passes 170
inherited controls and 70 profile controls; 10 extracted-source
controls per arm execute the audited construction, producer, finish,
retirement and pause statements; both cfg branches compile and both
armed traces reproduce the historical two arm lines and one hit; the
unrelated completed segment keeps the live early return; both
historical default and no-default NativeEqual signatures authenticate
without re-earning. No production field, accessor, allocation or file
was introduced. All 151 source/tail/destructor joins, 90 preimages and
180 postimages and the 5,399 maintained files authenticate;
`production_changes=0`; `source_schema` stays `f9604852…`; thirteen
distinct partial targets render byte-identically from two roots; no
complete plan or schema was emitted.

The stage then stopped at `P0267`, `P0312` and `P0323` — the entire J
class — with `PortBlocked::DeferredJoinOwnerTransport`. The pinned
recipe is one `c7tail` arm, one
`OwnDeferredDefinitionPublicationAtActualProfileQueueJoin` hit and the
existing quiescence stop, in both modes, with the sealed fixture path
`down-value-guard>Input.c7guard>TryApplyDownValuesFromSymbolSuspensionFrame::resume>finish_suspended_down_value_application>Input.c7tail;tail=TypedAction::complete_work`
(and the analogous sub-value and up-value paths). The actual path is:
`complete_work` returns `Action::Complete(Err(EvalError::InputRequired))`
from the producer; the adapter returns the action; the driver removes
and drops the executing frame and its queue owner, then matches
`Action::Complete(next_outcome)` into `outcome`; the driver's actual
`InputRequired` branch takes `pending_suspension` into `nested` and
`resuming_suspension` into `suffix` and calls
`nested.append(&mut suffix)` (`driver.rs:171`); `SuspendedEvaluation::
append` calls `self.frames.append(&mut suffix.frames)`
(`suspension/core.rs:213`); `FrameStorage::append` routes to
`append_profiled` and then `a.append(b)` (`core/storage.rs:240-281`);
`ProfileQueue::append` (`core/profile_queue.rs:654`) reaches
`self.publish_deferred_definitions_before_join()`
(`core/profile_queue.rs:686`), whose retained first statement is the
omission (`core/profile_queue.rs:612`). The retained helper's
`actual_join_stop` stores only the row in `OWN_JOIN`, and
`omit_own_join()` takes no arguments, so any queue that reaches the
method first consumes the omission: the isolation control shows an
unrelated queue taking it while the intended queue loses its hit. The
frontier facts are that the selected frame and its `ResumeGuard` have
retired before the join, that the new nested frames do not execute the
method, that `Action::Complete(Result<Slot, EvalError>)` carries no
queue identity, and that `ResumeStop` is owned by value and its address
does not survive the moves; guard widening, a stronger pure return
control and an unbound descendant frame all yield zero, and assigning
the publication to a descendant frame is a false ownership edge.

What those facts do not establish is that the receiver is anonymous.
The nested object the driver appends is the pending
`SuspendedEvaluation` that the armed input suspension installed, and
its `frames` storage is the queued `ProfileQueue` that publishes. The
missing piece is an identity for that object, not a value on the
action and not a new production structure.

Q23 rules that the owner is the nested suspension the driver actually
appends, identified by its own `SuspensionGeneration`:

1. **Owner object.** The J row's deferred-definition publication
   executes on the queue that is the `frames` storage of the nested
   suspension the driver appends — the `SuspendedEvaluation` that the
   actual input suspension installed during the row's producer
   execution, which the driver takes as `nested` and passes to
   `SuspendedEvaluation::append`. The suspension's construction
   identity is `SuspendedEvaluation.generation: SuspensionGeneration`,
   an existing field (`suspension/core.rs:34-41`): a `Copy + PartialEq +
   Eq` wrapper over a `u64` that `SuspensionSlot::install` assigns
   monotonically with `checked_add` and never reuses, asserting the
   slot is empty before replacing (`suspension/core.rs:269-281`). It is
   not the typed frame, its `ResumeGuard`, the adapter, the `Action` or
   `ResumeStop` value or its address, the queue's `Box` address (which
   moves), a `CellId`, the row label, any string or `Debug` rendering,
   or any ambient, shared or static store.
2. **Issue.** The token is the pair (row, generation) and may be
   obtained only from the actual install. `Session::
   begin_input_suspension`'s `self.suspension_access().install(request,
   active_stack)` is the sole install path and already returns the
   `SuspensionGeneration` (discarded today, `suspension/core.rs:519`);
   the existing `pause(&request.prompt)` statement in
   `builtins_input.rs`'s `drive_input_request` — immediately after that
   install — remains the only site that may emit the pinned
   `producer-InputRequired` arm line. The token is confirmed only by the
   producer's actual returned
   `Action::Complete(Err(EvalError::InputRequired { .. }))` at
   `complete_work` under the row's `JoinOwner` scope
   (`suspension/typed/action.rs:65-73`). A suspension the producer
   swallows, a success, another error, or a fabricated boolean,
   generation or renderer value issues no token. Reading the generation
   anywhere other than the installed object's own install site, or
   emitting the arm line anywhere other than that pause site, is not
   the token.
3. **Carry.** The transport is the object itself: the pending
   `SuspendedEvaluation` with generation G is taken by the driver at the
   actual `InputRequired` branch and appended, so the suspension's own
   identity reaches the join without any value written into the action,
   the frame, the adapter, the queue or any production structure. The
   helper carries only `(row, G)`. The binding does not depend on the
   frame, its activation or any resume scope, which is exactly why it
   survives the frame's retirement.
4. **Consume.** The only consumption site is
   `ProfileQueue::publish_deferred_definitions_before_join`'s retained
   omission when it runs inside `SuspendedEvaluation::append`'s scope
   for the suspension whose own `generation` equals the token's. The
   scope is entered for exactly the duration of
   `self.frames.append(&mut suffix.frames)`, and its value must be read
   from that same `self.generation` (the module that owns the private
   `u64` is `suspension/core.rs`, which is instrumentable). A publish
   outside any append scope, inside another suspension's append scope,
   or for a different generation is ineligible. The receiver is that
   suspension's own queue by construction: the only production path to
   the method is `SuspendedEvaluation::append → FrameStorage::append →
   append_profiled → ProfileQueue::append → self.publish…`, there is no
   other call site of either method, and no other queue's publish is
   reachable inside the scope. The armed install itself profiles the
   pending storage (`begin_input_suspension` calls
   `enable_pending_profile` under `resume_profile_configured()` and then
   `register_profile_owner` with the actual request control), which is
   what makes the publish reachable at all and registers this queue's
   `(root, epoch)` against the actual control; the audit must pin that
   registration and require that the queue which publishes is the queue
   constructed at that install (`enable_execution_context` early-returns
   for a profiled queue with a valid epoch, and the deprofile and
   arena-absent paths do not reach the publish; a storage not profiled
   at the install is not the armed object and is dropped unused rather
   than normalized). Consumption is at most once; the token is
   invalidated at the next install, when the armed suspension is taken
   for abandon instead of joined, when the append scope ends without
   consumption, and at run begin and end; it never survives its step or
   its run.
5. **Rejected carriers and rules.** The row-only latch (`OWN_JOIN`) and
   its zero-argument `omit_own_join`, because its isolation
   counterexample — an unrelated queue consuming the omission while the
   intended queue loses its hit — is the defect this rule removes; the
   `Action`/`ResumeStop` value or address; the typed frame, its
   `ResumeGuard`, the adapter or any new suspension frame; the
   descendant frame's resume scope; a first-completed or
   most-recent-publish rule; the queue's `Box` address; string or
   derived-`Debug` identities; any new production field, accessor or
   allocation; any new overlay path; and any instrumented `driver.rs`,
   `core/storage.rs` or `profiling/logical/completion.rs`. Only
   `suspension/core.rs`, `builtins_input.rs`,
   `suspension/core/profile_queue.rs` and the already-instrumented
   `suspension/typed/action.rs` and producer files may carry the new
   calls, and the live statements and lifetimes must remain
   byte-identical.
6. **Falsifiers.** The helper and the extracted-statement controls must
   show: an unrelated queue's publish inside another suspension's append
   scope records no hit while the intended queue still records its one;
   a publish outside any append scope records none; a second suspension
   (different generation) appended first does not match; the armed
   suspension abandoned or swallowed issues no hit and leaves no usable
   token; a fabricated generation, or a scope injected at a non-append
   site, is rejected by the pre-edit audit, which pins the extraction to
   `self.generation` in the pinned `SuspendedEvaluation::append` and to
   the install's return value; two appends of the same suspension yield
   one hit; a stale token across steps or runs is dropped; the queued
   run stays at zero arms and zero hits. Both historical modes keep
   exactly one `c7tail` arm, one
   `OwnDeferredDefinitionPublicationAtActualProfileQueueJoin` hit, the
   phase, the selector, the public output and the existing quiescence
   stop; any extra arm, extra hit or changed line is a STOP. The three
   rows' source frontier (23 anchors, 12 controls) is extended, not
   relaxed, with the install, append-scope and consume anchors and
   their controls, entry-pinned from live sources, with two-root byte
   identity and full-file AST restoration after instrumentation erasure.
7. **Stage.** No new allocation or production surface. Class J proceeds
   only after this pending-suspension identity transport is source-bound
   and audited per row, with extracted-statement controls that execute
   the audited install, append-scope and consume statements rather than
   only helper simulations; the remaining rows and the 151-plan bar
   continue from the same partial tree, and a row whose pinned hit
   cannot be mapped to its own actual object identity is named and
   stopped, never normalized.

All other Q22, Q21, Q20, Q19, Q18, Q17, Q16, Q15, Q14, Q13, Q12, Q11,
Q10 and brief terms stand.

## Continuation 15 ruling (director) — Q24: the Q23 suspension identity fails its own cross-slot falsifier; the stage moves to a complete ownership inventory, and the identity domain is decided there for every dependent row at once

The fifteenth run implemented Q23 locally and correctly. The shared J
recipe is source-bound in scratch: it captures the actual
`SuspensionAccess::install` return inside `Session::begin_input_suspension`,
requires the installed storage's valid profile after the unchanged
request-control registration, keeps the existing `pause(&request.prompt)`
site, borrows the actual `Action::Complete` result under the row's
`JoinOwner`, and consumes only inside the scope around
`self.frames.append(&mut suffix.frames)` whose key is that same
`self.generation`; success, other errors, abandonment, the next install,
append-scope exit and run boundaries leave no usable token. The audit has
38 anchors (all 23 inherited) and 32 controls; the inherited 12 frontier
controls still pass; two roots emit identical partial source for seven
paths; the helper passes 52 controls per J row; extracted-source controls
execute the actual `SuspensionSlot::empty`/`install`, access-install,
input install/pause, typed completion, append scope and omission
statements; the ordinary valid-owner traces keep exactly one `c7tail` arm
and one pinned hit in both compiled cfg shapes, the queued traces keep
neither, and both historical default/no-default quiescence signatures
authenticate without re-earning. No production field, accessor,
allocation, overlay path or lifetime change was introduced.

The J class then failed the falsifier Q23 rule 6 requires. `SuspensionSlot::
empty` sets `next_generation` to zero (`suspension/core.rs:242`),
`install` increments it with `checked_add` (`:269`), `Session::empty`
builds that slot independently for each session
(`session/session_state.rs:691`) and `Session` owns it by value
(`model/session.rs:115`). The counter is therefore monotonic within one
slot, not a construction identity across slots: two distinct slots both
install generation 1. In the extracted-source fixture the unrelated
slot's real append scope at generation 1 consumes the token and the
intended append executes the live publication with no hit — for all three
rows at one test thread (`unrelated_hits=1 intended_hits=0`); the
different-generation counterpart omits only the intended publication and
all 15 disabled controls per row pass with exact fragment records. No
generation is fabricated at any instrumentation site, the earlier
13-control sets are disclosed as having omitted the alias case, the four
failing attempts are preserved, and no expectation was weakened. This is
a generator/extracted-control finding, not a production regression, and
the worker made no universal impossibility claim.

Q24 rules that the Q23 carrier is not accepted as a class-level transport
yet, and that its identity domain is a cross-row design question rather
than a patch: every row that would use the suspension identity shares the
slot-local aliasing, and the answer decides how those rows are generated.
Under the standing discipline adopted this day (a `Scope: NONE` complete
inventory before any implementation pass), the decision is made in the
full ownership inventory brief for this stage, with entry-pinned evidence,
and must select exactly one of:

1. **An authenticated single-slot domain.** Prove from the actual runner
   sources that a selected run has one live slot — one `HostSession`
   binding, the driver taking `nested` and `suffix` from its own session,
   no second live pending slot within a run or thread episode — and
   explicitly rule the equal-generation cross-slot control outside that
   domain, with the domain proof entry-pinned and the falsifier retained
   in its original form.
2. **A source-bound owner namespace.** Disambiguate actual slots or
   suspensions with an existing object on an existing path — for example
   the request-interrupt control that `begin_input_suspension` already
   registers against the queue (`suspension/core.rs:525-530`,
   `core/storage.rs:66-73`, `core/profile_queue.rs:744`), read at both the
   arm and the publication under the same owner lookup — with no new
   field, accessor, allocation, overlay path or production surface, and
   with the cross-slot falsifier passing (unrelated zero, intended one).
3. **A named stop.** If neither can be authenticated, the J class and
   every dependent row stay `PortBlocked` by name; the bar is not
   lowered, the counter is not called a global identity, and the
   required falsifier is not replaced.

The inventory must retain both the same-generation and the
different-generation evidence, keep the pinned arm and hit signatures and
the queued-zero rule, and treat any unproved claim of global uniqueness
as a STOP. Brief 747 is that inventory: it takes the Q10–Q23 recipe
ladder as its candidate set, answers Q24 for all dependent rows at once,
and produces the complete per-row ownership matrix — actual object, actual
source edge, transport recipe, isolation falsifier, preserved nodes,
shared-file composition (including `call.rs`'s prepared and profile
forms) and a named `PortBlocked` class where a binding cannot be
expressed — with a zero-unknown-rows statement. Brief 742's remaining
implementation waits on it; this ruling does not authorize another
single-frontier implementation run.

Budget: the run's 198.28 s charge is accepted (138.22 s measured plus the
0.5 s bookkeeping and 60 s preparation/final-reporting reserve), leaving
2,576.41 s focused in 742 (envelope 3 at 1,076.41 s, envelope 4 at
1,500 s, both workspace envelopes untouched). This ruling charges
1,136.41 s (envelope 3 closed at its 1,076.41 s remainder plus the 60 s
envelope-4 reserve), leaving 1,440 s focused in 742 (envelope 4 at
1,440 s, both workspace envelopes untouched); brief 747 is granted one
fresh 1,500 s focused envelope from the stage's 69,000 s allocation, so
CM4's committed share becomes 19,500 s and 743–746 retain 49,500 s.

All other Q23, Q22, Q21, Q20, Q19, Q18, Q17, Q16, Q15, Q14, Q13, Q12,
Q11, Q10 and brief terms stand.

## Continuation 16 ruling (director) — Q25 adopted: implement Drafts A and B in one pass (the complete 151-plan generator and the 94-row schema)

Q25 is adopted as option 2 for all four frontier classes. The instrumentable
surface extends by exactly four raw resume files —
`crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs`,
`crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs`,
`crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs`
and `crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs` —
from 90 to 94 paths and from 180 to 188 postimages. The maintained source
files stay unchanged; the extension files are instrumented copies in the
generated overlay; no fifth overlay, no new production field, accessor or
allocation, and no 138-row subset is admissible. The owner-qualified call
handoff specified in `.temp/brief-747/continuation-1/Q25.md` and
`frontier.json` is the candidate transport, not yet admitted: its per-class
extracted-statement controls and emitted-overlay controls must execute in
both cfg arms over the real allocation, raw clones/borrows/decoders, call
and target statements before any class is credited. The scoped-call-only
remedy stays rejected.

Design of record (read-only authority; authenticate against
`verified-final-a/manifest.json` before use): `.temp/brief-747/continuation-1/`
with `Q25.md`, `DRAFT-A-GENERATOR.md`, `DRAFT-B-SCHEMA.md`,
`DRAFT-COMMON.md`, `DRAFT-C-REPLAY-GATE.md`, `GENERATOR-PLAN.md`, and
`verified-final-a/{frontier.json,schema-plan.json,controls.json,scoped-call-decision.json,inventory.json,inventory.tsv,generated-file-set.tsv,composition.json,citations.json}`;
`verified-final-b` is the byte-identical control root. The accepted first-run
artifacts under `.temp/brief-747/` (273 files, `inventory.tsv` sha256
`86912a3d…`) and the accepted Q24 artifacts are inherited by hash and are not
re-derived, re-earned or hand-edited.

Goal. Implement Drafts A and B consolidated into one source-editing pass: one
deterministic declarative plan for each of the 151 frozen own rows, with every
immediate/resumed/returned-request edge and actual owner identity, then
render both complete overlay arms for the 94-path set and independently
audit all preserved nodes; then mechanically emit and authenticate the
six-column 94-row overlay table, rebind all and only the 151 `source_schema`
cells, and print the new digest computed from the exact table bytes only
after all 151 plans pass. Two fresh roots must be byte-identical. No semantic
catch, campaign packet or acceptance result is claimed.

Scope. May modify only: the eleven generation files enumerated in
`GENERATOR-PLAN.md` under `crates/rum-stdlib/tests/suspension_oracle/mutations/generation/`;
`mutations/mod.rs` and `mutations/schema.rs` for generation binding;
`mutations/source_auth/{mod,schema,tests}.rs`; the mechanically emitted
`mutations/source_auth/overlays.tsv`; only the 151 `source_schema` cells in
`inventory/obligations/mutants.tsv`; and `obligations/mod.rs` solely for a
necessary crate-private discovery export. The standing mechanical maintenance
from `DRAFT-COMMON.md` may import only instrument-produced cells. Do not
modify live production, `source_worker.rs`, runner/observer/predicate bodies,
retained overlay bytes, CM3 selection, coverage or any unrelated inventory;
do not add a fifth overlay, a production accessor/allocation, a placeholder
schema digest, or a hand-edited derived cell.

The bar. 151 plans, zero unknown edges, ten classes and 79 subtypes; the 94
targets and 188 postimages; the 13 frontier rows' per-class extracted and
emitted controls over the real statements in both cfg arms; the 19 control
families (unrelated 0 / intended 1, nested unarmed, two armed nested owners,
direct helper reentry, second consumption, retired Weak, wrong key
components, wrong type/allocation, P2348 guard-immediate and resuspension
paths, P1070 loop reentry, queued zero, disabled exactness, static
duplicate/tail/drop rejection); the inherited Q24 reds and greens by hash;
the 22 historical signatures authenticated; whole-file erasure and full-file
AST restoration in both arms; the pinned omission AST and preserved nodes
separately audited; the new schema digest independently re-hashed and all
non-schema table cells byte-identical. After the last covered edit, run the
standing replay in order — fingerprints, both production-acceptance
certificate modes, ownership exactness, rejection census, LP01 and typed
slots — then `cargo fmt --check`, then both workspace modes with the exact
seven exclusions in `DRAFT-COMMON.md`, with the switched checks on the same
binaries; full unfiltered suspension_oracle and typed_slot_census require
failed=0 and filtered=0. Derived-cell-only reds are ordering artifacts and
are replayed, never hand-edited; a semantic red is reproduced at one thread;
failures are retained and quoted.

Budget and stop rules. This continuation's implementation envelope is 9,000 s
focused, allocated from the stage's 69,000 s reserve (CM4's committed share
becomes 28,500 s; 743–746 retain 40,500 s); the Q25 ruling's 60 s preparation
charge comes from it, leaving 8,940 s at dispatch, and the run charges its
measured serial seconds plus 0.5 s bookkeeping against that balance. One
workspace envelope of 6,000 s is assigned here; the second stays held for the
verification stage. One cargo command at a time, offline and locked, jobs 8,
inside the 16G scope, every long command through `scripts/run-bounded.py`.
Stop-loss keyed on progress: two consecutive compiled attempts with no new
plan or proof stop and report the remaining set. Stop before exceeding the
envelope; checkpoint done/next/tree/budget after each milestone. No campaign,
epoch, new catch, acceptance, CM5 or bar change. No network, no git, no
`unsafe`, no dependency or lockfile change; leave the authorized candidate
uncommitted for the director's gate; report in `rum-report/1` and return only
its validated summary.

## Continuation 17 ruling (director) — Q26: abandonment is observed at the actual `FrameAbandon` implementation of every token-holding frame type, inside the four admitted raw files; the derive is replaced in the copy, never in production

Continuation 16 is accepted as a STOP. Run: `cdx` (unit
`brief-742-cdx-20261117-012829-1871047`, rc 0, peak 1,188,102,144 B,
oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 4,644,841 B,
sha256 `cbff9c95…`; report STOP, 8,784 changes (8,782 added, 2 modified,
0 deleted — both modified in `.temp`), 0 disclosures, 2 deviations,
0 questions, 15 PASS / 2 FAIL / 2 NA, 660 citations verified; 142.92 s
charged — 114.92 s measured serial plus 28.0 s bookkeeping at 0.5 s per
command — leaving 8,797.08 s focused in the implementation envelope;
workspace spent 0 of the assigned 6,000 s). Both deviations are accepted: the
complete generator and 94-row schema are absent because the mandatory
retirement control fails, and the early per-command remaining cells
under-charged bookkeeping until the ledger was normalized with every original
receipt preserved. Scope holds: `git status` is the same 23-entry partial CM4
tree, 5,400 maintained files and the 380 accepted design files are unchanged,
`production_changes=0`, no schema cell changed, `source_schema` stays
`f9604852…`.

The finding is accepted as a candidate-contract finding, not an impossibility
claim and not a production regression: `PortBlocked::RawAbandonmentWithRet-
ainedArc`. The specified Weak bind, raw resume read and direct-call handoff
cannot distinguish an allocation whose actual `FrameAbandon::abandon` already
ran (`Discard`, its `Arc` consumed, one strong alias retained) from an
ordinary live allocation; the retired control therefore hits once where Q25
requires zero, while the independent live allocation correctly hits once, in
both armed cfg forms at one thread (13 passed / 13 failed each) and both
disabled forms pass 26/0. The director reproduced all four binaries under
`.temp/director/gate-742c16/`: the armed forms fail exactly the thirteen
`*_actual_abandon` controls and the disabled forms pass every row. The worker
did not change the expected retired hit to one, did not clear every token
(global clearing would break the live-owner control), and did not credit the
eleven synthetic producer seams as class evidence. The earlier 96 fragment
records per arm are parity evidence only.

Q26. The missing observation is the actual abandonment implementation of each
frame type a frontier row's token can bind. In each of the four admitted raw
files, and only where the token-holding frame type's declaration and its
`FrameAbandon` implementation already live in that file, the copied postimage
replaces the `PureFrameAbandon` derive with an explicit `impl FrameAbandon`
that invokes the one-use retirement for that allocation before returning
exactly the derive's `FrameAbandonAction::Discard`. The retirement runs
before the lifetime-ending move, extends no lifetime, adds no strong owner,
clears no unrelated token and changes no production `Drop`. The derive's
expansion is fixed and is quoted from `crates/rum-derive/src/lib.rs:123-159`
(an unconditional `Discard`), so equivalence is proven against the macro
source, not assumed, and the same-file precedent for a side-effecting manual
abandonment implementation is `EvalStructuredTraceCaptureExprSuspensionFrame`
at `crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:41-49`
(`context.restore_trace_capture(self.prior_enabled)` then `Discard`). A
manual (non-derive) abandonment implementation on a token-holding frame type
receives the same retirement statement in its own actual implementation.

The replacement is registered instrumentation: the attribute-list edit and
the inserted impl item are two registered source edits with their own erasure
rule, whole-file AST restoration must reproduce the live file exactly in both
cfg arms, and the pinned-omission audit stays separate. If any token-holding
frame type's declaration or abandonment implementation lies outside the four
files, that is a class finding and a STOP — no fifth path is added without a
ruling. The control stays at full strength: `abandon_and_weak_retirement`
requires zero hits for the abandoned allocation with a retained Arc and one
for an independent live allocation, in both cfg arms and the disabled form;
the other eighteen Q25 control families and the inherited Q24 artifacts are
unchanged. The eleven remaining rows' whole producer/constructor paths must
be extracted and executed per row before any class admission; synthetic
seams are corroboration only.

This continuation is design plus executed extracted controls, `Scope: NONE`
— no maintained source, table, inventory or schema change; no 151-plan
bundle, no 94-path/188-postimage emission, no schema digest; output under
`.temp/brief-742/continuation-17/`, with the 23-entry partial CM4 tree and
every accepted artifact byte-identical; no git, no network, no `unsafe`, no
dependency or lockfile change. Budget: 1,500 s focused carved from the
implementation envelope's remaining 8,797.08 s, leaving 7,297.08 s for the
resumed implementation; the assigned 6,000 s workspace envelope stays with
the implementation and the second stays held for verification; no workspace
command runs here. Stop-loss keyed on progress: two consecutive compiled
attempts that certify no new control or path stop and report the remainder.
The next step is this continuation's gate and then the resumed implementation
of the complete 151-plan generator and the 94-row schema. Q25's four-file
extension, the 94/188 surface, the owner-qualified handoff, Q24, the 151-plan
bar and the non-shipment of the 138 subset all stand.

## Continuation 18 ruling (director) — the Q26 comparator is corrected and the resumed implementation runs to the full bar; the progress stop-loss counts completed audit stages, certified controls and closed rows

Continuation 17 is accepted as a compliant STOP under the rule it was given.
Run: `cdx` (unit `brief-742-cdx-20261117-025547-1892929`, rc 0, peak
330,010,624 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`,
681,155 B, sha256 `c37aedf3…`; report STOP, 3,312 changes (3,310 added,
2 modified, 0 deleted — all in `.temp`), 0 disclosures, 1 deviation,
0 questions, 8 PASS / 1 FAIL / 3 NA, 66 citations verified; 68.64 s charged —
4.64 s measured serial plus 4.0 s bookkeeping and the 60 s
preparation/final-reporting reserve — leaving 1,431.36 s of the 1,500 s Q26
sub-envelope; workspace 0). The deviation is accepted: the Q26 erasure,
source-path execution and retirement controls are incomplete because the
stop-loss fired, and nothing was substituted. Scope holds: the 23-entry
partial CM4 tree, the 5,400 maintained files and the 380 accepted design
files are unchanged, `production_changes=0`, `source_schema` stays
`f9604852…`, and the three entry drifts are this director's own gate commit,
correctly disclosed as predating the run.

The stop is an instrument-comparator stop, not a design or ownership fact.
Both compiled audit attempts exited 101 on the auditor's own expected-value
construction: attempt 1 compared the derive expansion with a
whitespace-sensitive token substring and rejected `quote!`'s joint
punctuation; attempt 2 passed that stage and rejected the emitted impl on the
trailing comma `ToTokens` preserves in the parameter list, which the expected
`parse_quote!` signature omits. Two compiled attempts were made and neither
certified a new stage; the assertion was not weakened, the second mismatch was
not edited after the stop, and both attempts and binaries are retained. The
scratch design it delivered is real: eleven token frame types declared in the
four admitted raw files (one trace, two guarded, four guarded-replace, four
rules), two byte-identical 11-file partial roots, exactly four files differing
from Q25 by the derive removal and the explicit retirement impl (no path
beyond the four), and a retirement design in which the helper removes the
allocation's pending binding and records its `TypeId` and Weak identity in an
instrumentation-only retired set that every extant owner also consults, so a
token already transferred into a call or flow is covered without a strong
reference or a global clear. None of it is admitted: no retirement control
has executed for any row.

The stop-loss is refined for the resumed implementation, because counting only
a certified control fired on a converging mechanical repair — the class this
project already ruled must not stop a converging instrument. Forward progress
is any of: a completed audit stage, a certified control or path, or a closed
extracted row. Two consecutive compiled attempts with none of those stop and
report the remainder; the envelope stays the hard bound.

Continuation 18 is the resumed implementation and runs to the full bar.
First close Q26: correct the expected signature to the exact emitted tokens
(trailing comma included), keep the rejected attempts as artifacts, complete
whole-file erasure of the Q26 edits in both partial roots and both cfg arms
and re-derive the live source from whole files, derive the token frame set
independently from the authenticated raw-resume annotations and compare it
exactly with the eleven-frame declaration, and add the rejecting controls —
changed derive, extra impl, missing, wrong and late retirement, changed
`Discard`, overlap, non-target drift. Then extract and execute all eleven
remaining rows' whole producer and constructor paths (P1070, P2348, P2354,
P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418) in addition to the
inherited P0214/P0216 paths, with no synthetic seam credited, and execute the
full control set: `abandon_and_weak_retirement` with zero hits for the
abandoned allocation carrying a retained Arc and one for an independent live
allocation, the other eighteen Q25 families at full strength (including
P2348's immediate and resuspension paths and P1070's loop reentry), and the
inherited Q24 reds and greens by hash. Only then implement the complete
151-plan generator over the 94-path set, mechanically emit and authenticate
the 94-row schema with all 151 `source_schema` cells and the new digest from
the exact table bytes, prove whole-file AST restoration and the separate
pinned-omission audit in both cfg arms, run the standing replay in order,
`cargo fmt --check`, and both workspace modes with the seven exclusions in
`DRAFT-COMMON.md`, with the switched checks on the same binaries and the full
unfiltered `suspension_oracle` and `typed_slot_census` at failed=0,
filtered=0.

Budget: the Q26 sub-envelope's remaining 1,431.36 s merges with the
implementation reserve of 7,297.08 s into one 8,728.44 s focused envelope,
with the assigned 6,000 s workspace envelope; the second workspace envelope
stays held for the verification stage, which uses brief 742's remaining
1,440 s. CM4's committed share stays 28,500 s and 743–746 retain 40,500 s.
Entries, files and the 23-entry partial tree are the authorized test-layer
surface; no production, runner, observer, predicate, selector or phase
change, no fifth overlay, no 138-row subset, no placeholder digest and no
hand-edited derived cell. No git, no network, no `unsafe`, no dependency or
lockfile change; the run ends with the tree left for the director's gate and
a `rum-report/1` report whose validated summary is the only returned text.

## Continuation 19 ruling (director) — Q27: the retirement observation belongs to the raw-frame transport, so it covers all twenty-five raw-owner rows inside the already-admitted 94-path set; the Q26 frontier contract is admitted on executed evidence and the implementation runs to completion

Continuation 18 is accepted. Run: `cdx` (unit
`brief-742-cdx-20261117-030757-1896462`, rc 0, peak 1,066,164,224 B,
oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 2,109,978 B,
sha256 `1722708f…`; report STOP, 8,898 changes (8,896 added, 2 modified,
0 deleted — all in `.temp`), 0 disclosures, 1 deviation, 0 questions,
15 PASS / 3 FAIL / 2 NA, 1,542 citations verified; 139.05 s charged —
48.55 s measured serial plus 30.5 s bookkeeping and the 60 s
preparation/final-reporting reserve — leaving 8,589.39 s focused in the
merged envelope; workspace 0 of the assigned 6,000 s). The deviation is
accepted: the complete generator and schema are absent because the scope STOP
fired, and neither the four-file permission was silently widened nor the
138-row subset shipped. Scope holds: the 23-entry partial CM4 tree, the 5,400
maintained files, the 380 accepted design files and the 12,094 earlier
artifacts are unchanged, `production_changes=0`, `source_schema` stays
`f9604852…`.

The Q26 frontier contract is now admitted on executed evidence, and the
director reproduced the load-bearing binaries at one thread: the expanded
suite passes 224/0 with filtered=0 in each of armed and disabled crossed with
hash and no-hash, the helper controls pass 20/0 per enable arm, the two
11-file partial roots are byte-identical, and the P2593 counterexample
reproduces retired_hits=1 against the required 0 with the independent live
allocation at 1 and the disabled forms at 2/0. The audit chain is complete:
the exact emitted signature (trailing comma included), the unconditional
`Discard` expansion, an independently derived eleven-frame set, eleven
derive-list edits and eleven actual retirement impls per root, erasure to the
Q25 AST (token respacing is not a byte claim; whole-file AST restoration is
the bar, and the byte diff between the erased root and the Q25 partial is
only resyntaxed attributes and trailing blank lines), the Q25 eraser's
whole-file restore of the eleven live files with the five pinned omission
sites, nine rejecting controls (changed derive, extra impl, missing, wrong and
late retirement, changed `Discard`, overlap, changed macro, non-target drift),
47 extracted whole functions with thirteen producer calls and eleven actual
retirement impls, eighty fragment traces and final values per form matching
the disabled fixture in all four forms, and both real crate type modes. All
thirteen frontier rows now execute whole producer and constructor paths; no
synthetic seam remains and none was credited.

Q27. The retirement observation is a property of the raw-frame transport, not
of the four added files: the twelve further raw owners in ten already-retained
paths (P0145, P0936, P1037, P1038, P1132, P1193, P1708, P2299, P2544, P2547,
P2591, P2593, in `debug_tools/reap/sow.rs`,
`kernel_data/rewrite_data/cases/pattern_resolution/continuation.rs`,
`kernel_data/rewrite_data/replace/repeated.rs`, `options/builtins.rs`,
`pattern_matching/core/resume_frames.rs`,
`associations/basic/lazy_completion.rs`, `schema/continuation.rs`,
`structure_ops/list_plumbing.rs`,
`structure_ops/sequence/take_drop/lazy_completion.rs` and
`structure_ops/surface/lazy_completion.rs` — the director verified all ten are
inside the retained ninety-path set) receive the same Q26 substitution in their
copied postimages: `PureFrameAbandon` replaced by an explicit
`impl FrameAbandon` that retires that allocation before returning exactly
`Discard`, with the same exactness, erasure and rejecting controls. This adds
no path and changes no count — 94 paths and 188 postimages stand, these ten
postimages change content, and the `source_schema` digest is still computed
only after all 151 plans pass. The census must be re-derived independently from
the authenticated inventory and raw annotations and must cover all twenty-five
`Q747-RawFrame` rows; a token-holding frame type whose declaration or
abandonment implementation lies outside the admitted 94 paths remains a STOP.
For each of the twelve rows the control is the same and stays at full strength:
the actual abandonment with a retained Arc yields zero, an independent live
allocation yields one, both cfg arms and the disabled form, with the
ordinary-Arc-drop positive control; the P2593 helper counterexample is
corroboration only and is not a whole-path or class receipt.

Continuation 19 completes the implementation under the remaining envelope:
admit the twelve rows' retirement substitution and controls; then the complete
151-plan generator over the 94-path set, the mechanically emitted and
authenticated 94-row schema with all 151 `source_schema` cells and the new
digest from the exact table bytes, whole-file AST restoration and the separate
pinned-omission audit in both cfg arms, the ordered standing replay,
`cargo fmt --check`, and both workspace modes with the seven exclusions in
`DRAFT-COMMON.md`, the switched checks on the same binaries, and the full
unfiltered `suspension_oracle` and `typed_slot_census` at failed=0,
filtered=0. Budget: the 8,589.39 s focused remainder with the assigned
6,000 s workspace envelope; the second workspace envelope stays held for the
verification stage, which uses brief 742's remaining 1,440 s. Stop-loss: the
refined progress counter — a completed audit stage, a certified control or
path, or a closed row; two consecutive compiled attempts with none stop, and
the envelope is the hard bound. No production, runner, observer, predicate,
selector or phase change, no fifth overlay, no 138-row subset, no placeholder
digest, no hand-edited derived cell, no git, no network, no `unsafe`, no
dependency or lockfile change.

## Continuation 20 ruling (director) — Q28: the progress stop-loss counts converging instrument construction, and continuation 20 resumes the stopped fixture and runs the implementation to the full bar

Continuation 19 ran `cdx` (unit `brief-742-cdx-20261117-034546-1909079`, rc 0,
peak 1,392,050,176 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`,
3,363,952 B, sha256 `acc55323…`; report STOP, 10,322 changes (10,320 added,
2 modified, 0 deleted — all in `.temp`), 0 disclosures, 1 deviation, 0
questions, 20 PASS / 2 FAIL / 2 NA, 363 citations verified; 217.34 s charged —
114.84 s measured serial plus 42.5 s bookkeeping and the 60 s
preparation/final-reporting reserve — leaving 8,372.05 s focused; workspace 0 of
the assigned 6,000 s). The deviation is accepted: the complete generator and
schema are absent because the progress STOP fired, and neither the 138-row
subset nor a provisional schema was substituted. Scope holds: the director
re-hashed the 5,400 maintained and 380 design files against the entry record and
found no change (the 23-entry partial CM4 tree included), the 20,992 earlier
artifacts are unchanged, `production_changes=0`, `source_schema` stays
`f9604852…`; the three document drifts are this director's own gate commit and
were disclosed at entry.

The Q27 work is admitted on executed evidence, and the director reproduced the
load-bearing binaries at one thread: the twelve new rows pass 48/0/0 in armed
hash and in disabled hash, the inherited whole suite passes 224/0/0 and the
helper controls 20/0. The census covers all 25 `Q747-RawFrame` rows, 23 frame
types and 14 declaration files, and the director verified against the retained
ninety-path set plus the four Q25 files that all 14 files lie inside the
admitted 94 (`outside_rows=0`). Each root carries 23 derive-list edits and 23
actual retirement impls whose expansion is the unconditional `Discard` with the
generic bounds preserved; the eleven retirement rejecting controls and the
twelve path rejecting controls each reject, including the parse-valid `None`
substitution that whole-file restoration refuses. The two 14-file retirement
roots and the two 16-file raw-path roots are byte-identical (verified by
recursive diff), one root file differs from its live source by exactly the
substitution, whole-file erasure restores all 14 live files, and the raw path
audit matches 16 whole files, twelve producer rows, twelve pinned omissions,
three exact owner-qualified handoffs and three local claims with the original
errors and drops unchanged. The twelve rows execute their actual
producer/constructor/resume paths — actual abandonment with a retained Arc is
zero, an independent live allocation is one, the ordinary-Arc-drop positive
control hits, queued and unarmed are zero — and all 48 records per cfg match
the live source exactly. Both copied real-crate type modes pass. The partial
work toward the generator is real but unadmitted: 151 source/tail/destructor
joins, 1,032 declared graph anchors resolving to 286 nodes in 92 files, 81
lexical producer shapes with zero unknown grammar, a paired-EvalReturn helper
at 16 armed controls with 21 rows in ten whole files, and the P0396 typed
selector at two whole copied files; none is a complete plan, graph or class.

Q28. The progress stop-loss is written in evidence units — a completed audit
stage, a certified control or path, or a closed row — and instrument
construction has no such unit before its fixture runs. It has now stopped two
converging mechanical repairs: continuation 17's audit comparator, and this
run's typed fixture, where the second compile retired nine of ten distinct
errors and left one fixture-only stub signature mismatch (`Expr` where the call
passes `&Expr`), retained both logs and both fixture sources, and was not
repaired after the stop. For an instrument under construction — a fixture or
scaffold that has not yet produced a runnable binary — a compile attempt is a
progress unit when its retained compiler verdict retires at least one
previously outstanding distinct error and reintroduces none previously retired
(canonical form: the error code plus the normalized message, deduplicated), with
the repair confined to fixture-only scaffolding or to the copied instrumentation
through its registered edits; removing or weakening a declared control is never
a repair but a contract change that stops. Two consecutive compiled attempts
without such a reduction still stop, and the envelope is still the hard bound. A
compiler rejection of the copied production source text, or of an
instrumentation edit applied to it, is not a mechanical repair: it is a finding
to report, and it stops. Nothing else in the counter changes: a fixture that
compiles but whose control fails against a candidate contract is still a STOP
carrying the finding, and the evidence bar still requires executed controls.

Continuation 20 resumes the stopped work and runs to the full bar: repair the
fixture-only scaffold stub the compiler rejected and execute the five P0396
source-path controls with their existing expectations, reproducing any genuine
semantic failure at one thread before any class conclusion (the nested
held-symbol-update reentry remains an unexecuted hypothesis and is not credited
either way); complete every declared ownership transport, including the four
complex paired rows P0001/P0002/P1566/P1805 that lie outside the partial paired
renderer; then the complete 151-plan generator with zero unknown edges over the
94-path set, the double generation producing 94 targets and 188 postimages,
whole-file AST restoration and the separate pinned-omission audit in both cfg
arms, the mechanically emitted and authenticated 94-row schema with all 151
`source_schema` cells and the new digest from the exact table bytes, the ordered
standing replay, `cargo fmt --check`, and both workspace modes with the seven
exclusions in `DRAFT-COMMON.md`, the switched checks on the same binaries, and
the full unfiltered `suspension_oracle` and `typed_slot_census` at failed=0,
filtered=0. Budget: the 8,372.05 s focused remainder with the assigned 6,000 s
workspace envelope; the second workspace envelope stays held for the
verification stage, which uses brief 742's remaining 1,440 s; CM4's committed
share stays 28,500 s and 743–746 retain 40,500 s. No production, runner,
observer, predicate, selector or phase change, no fifth overlay, no 138-row
subset, no placeholder digest, no hand-edited derived cell, no git, no network,
no `unsafe`, no dependency or lockfile change.

## Continuation 21 ruling (director) — Q29: the typed transport's owner is the actual operation, not the activation; a design continuation resolves P0396's nested-operation leak under fixed controls

Continuation 20 ran `cdx` (unit `brief-742-cdx-20261117-051403-1932192`, rc 0,
peak 390,582,272 B, oom 0 / oom_kill 0; report STOP, 3,329 changes (3,327
added, 2 modified, 0 deleted — all in `.temp`), 0 disclosures, 1 deviation, 0
questions, 11 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 176 citations verified; 89.34 s
charged by the director's own re-derivation — 14.84 s measured serial over 29
runs plus 14.5 s bookkeeping and the 60 s preparation/final-reporting reserve —
leaving 8,282.71 s focused; workspace 0 of the assigned 6,000 s). The deviation
is accepted: the complete generator and schema are absent because the executed
P0396 control failed under Q28. The FLAGGED launch error is settled and is not a
red: the first execution set `RUM_CM740_TARGET` instead of the helper's existing
`RUM_CM740_OMIT`, selected no row and returned 2 passed / 3 failed with zero
arms; the corrected environment on the same binary gives the load-bearing
result, and no source, expectation or binary was changed for the correction.
Scope holds: the director re-hashed the 5,400 maintained and 380 design files
against the entry record with no change (the 23-entry partial CM4 tree
included), the 31,311 earlier artifacts are unchanged, `production_changes=0`,
`source_schema` stays `f9604852…` with all 151 cells byte-identical, the 94
declared paths are enumerated as not emitted, and the three document drifts are
this director's own gate commits. The fixture work is exactly the Q28 repair:
the fixture-only stub's second parameter became `&Expr` and the fixture's empty
pending-suspension construction moved behind `Pending::empty()` in the module
that owns the element trait; five lines changed, no control or expectation
moved, and the compiler error set retired under Q28 rather than stopping.

The executed finding is admitted as a candidate-contract finding and a named
`PortBlocked::TypedResumeScopeReentryLeak` for P0396: the director reproduced
both armed cfg forms at one thread at 4 passed / 1 failed and both disabled and
both live forms at 5/0. The unrelated non-suspending symbol update records
`nested_hits=1`, `total_hits=2` against the unchanged requirement 0 and 1, and
its store is omitted — `b` stays 40 where the disabled and live forms store 41
— so the leak both miscounts and suppresses real work; the intended outer store
then records the second hit on the same activation and the same producer
occurrence. The mechanism is the one the finding names: the typed adapter
restores the producer activation for the entire call to `F::resume`, and the
helper's `eligible` accepts the resume token for any omission executed during
that call, so a nested held update that never executes the selected producer
and never constructs the selected frame still claims the token.

Q29. The typed transport's owner is the actual operation the producer suspended
for, not the activation in which it happens to execute. The token must be
minted at the producer's actual branch from identities that operation carries
and checked at the unchanged pinned omission against the operation actually
being completed, so that a nested operation in the same activation can neither
claim it nor be omitted by it, and an operation that never executes the
selected producer is neither counted nor suppressed. The named omission site
(`store_held_update`) and the accepted matrix's `Q18-Q19-TypedFrame` form do
not move; the five existing P0396 controls and their expectations are fixed and
may not be edited to pass. The design must be established by executed extracted
controls in both cfg arms and the disabled and live forms, with the existing
five and these rejecting controls: a nested operation with the same operator
and the same target as the suspended operation, a nested operation reached
through a different callee of the same activation, repeated resume, second
consumption, a wrong occurrence, a wrong activation, and disabled/live parity
of every record. Any new instrumentation edit takes the standing discipline: it
is registered with its erasure rule and the whole-file AST restoration of both
cfg arms, and no production source, runner hook, predicate or phase may change.
The design also inventories every typed-transport row with a shared omission
site and whether a nested operation can reach it, with zero unknown rows; a row
the mechanism cannot serve is a named PortBlocked, never a weakened control.

Continuation 21 is that design: `Scope: NONE`, 1,500 s focused carved from the
implementation envelope (leaving 6,782.71 s for the resumption), with no
workspace command; it delivers the operation-ownership protocol, the evidence
that each of its checks has been watched to fail where it must, the typed-row
inventory, and the drafts of the implementation brief. Continuation 22 is the
resumed implementation and runs to the full bar under the remainder: the
operation-ownership protocol executed for P0396 and every typed row the
inventory names, then every declared ownership transport including the four
complex paired rows P0001/P0002/P1566/P1805, the complete 151-plan generator
with zero unknown edges over the 94-path set, the double 94-path/188-postimage
generation, whole-file AST restoration and the separate pinned-omission audit
in both cfg arms, the mechanically emitted and authenticated 94-row schema with
all 151 `source_schema` cells and the new digest from the exact table bytes, the
ordered standing replay, `cargo fmt --check`, and both workspace modes with the
seven exclusions, the switched checks on the same binaries and the full
unfiltered `suspension_oracle` and `typed_slot_census` at failed=0, filtered=0.
The second workspace envelope stays held for the verification stage, which uses
brief 742's remaining 1,440 s; CM4's committed share stays 28,500 s and 743–746
retain 40,500 s. No fifth overlay, no 138-row subset, no placeholder digest, no
hand-edited derived cell, no git, no network, no `unsafe`, no dependency or
lockfile change.

## Continuation 22 ruling (director) — the Q29 protocol is admitted on executed evidence, with a per-row insertion-route and executed-control duty, and continuation 22 is the implementation

Continuation 21 ran `cdx` (unit `brief-742-cdx-20261117-053232-1937059`, rc 0,
peak 723,222,528 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`,
2,015,824 B, sha256 `6db8482b…`; report DONE, 4,143 changes (4,141 added, 2
modified, 0 deleted — all in `.temp`), 0 disclosures, 0 deviations, 0 questions,
16 PASS / 1 FLAGGED / 3 NA, 988 citations verified; 231.89 s charged — 0.60 s of
it reporting and every receipt carrying its 0.5 s, with parent and child
wrappers conservatively double-charged — leaving 1,268.11 s of the 1,500 s
design envelope, which merges back into the implementation remainder: 6,782.71 s
plus 1,268.11 s is 8,050.82 s focused, with the assigned 6,000 s workspace
envelope and the second held for verification). The FLAGGED item is settled and
is not a red: a fixture-only `E0282` Arc-slice inference error, retired by an
explicit `Arc<[i32]>` annotation, both attempts retained, all fifteen controls
passing afterwards and no copied production or instrumentation rejection.
Scope holds: the director re-hashed the 5,400 maintained and 380 design files
against the entry record with no change (the 23-entry partial CM4 tree
included), `production_changes=0`, `source_schema` stays `f9604852…` with all
151 cells byte-identical, the 94 declared paths are enumerated as not emitted,
the three document drifts are this director's own gate commits, and the design's
three replaced paths (`runtime/lib.rs`, `definitions/definition_updates.rs`,
`suspension/typed/adapter.rs`) are all inside the retained ninety-path set.

The Q29 protocol is admitted on executed evidence, and the director reproduced
the load-bearing binaries at one thread: all fifteen controls pass 15/0/0 in
armed, disabled and live each crossed with hash and no-hash, and six weakened
helper variants each reject at one thread (the ambient activation-scope token,
the operation-key components for operator, target, allocation and length, a
repeated claim, a second consumption, a wrong occurrence and a wrong
activation). The protocol is the one Q29 required: the operation's capability is
minted under the actual Part-selector `InputRequired` branch, bound to the
existing constructed frame's own `operation`, `target` and `SharedArgs`
allocation data pointers, and never recovered from an enclosing activation; a
projection installed at the adapter's actual `Arc<F>` creation reads those
private fields through the parent module, holds no frame, `Expr` or `SharedArgs`
strong reference and compares pointer values only; the first statement of the
copied callee claims by those values AND the current frame token, so a second
resume cannot claim it; the permit then moves once through the three actual
direct calls, each call creating a keyed ticket that the callee claims at entry
before any callback, with the original call expression preserved inside the
`FnOnce` wrapper; the copied store consumes its local capability at the
unchanged pinned omission, after the write-protection guard and before the
target match, with the original `HeldUpdateReturn` expression, selector and
phase pinned separately; and a nested operation of any operator, target or
callee has no incoming ticket, executes its real store and records zero hits.
Adapter drop erases the operation binding before address reuse, a new
run or episode clears the transport, and ordinary Arc alias drop does not retire
a live owner. Fourteen edits are declared against entry-pinned source, the
renderer rejects overlap, a missing anchor and a duplicate anchor, the erasure
audit restores both whole source files byte-for-byte with the independent
omission pin, and the AST audit restores all three files. The two design roots
are byte-identical and armed differs from disabled only in the `ENABLED`
constant. The five original controls and the entire original `exercise` body
are byte-identical to continuation 20's fixture; the ten added controls are
additions. The typed inventory is admitted as a map, not as a receipt: 50 rows
and 50 forms with zero unknown dispositions and zero unknown entry routes (24
shared, 12 `DirectCall`, 32 `DirectFrame`, 2 `LoopOperation`, 4
`ProjectedOperation`; 8 ordinary-bypass, 10 typed-entry-only, 32
explicit-frame), and only P0396 has executed ownership evidence.

Continuation 22 is the implementation and runs to the full bar under the merged
8,050.82 s focused plus the 6,000 s workspace envelope. It extends the admitted
protocol with per-row executed evidence: every typed row the inventory names
carries its own controls at full strength in both cfg arms and the disabled and
live forms, and for each row the frame's insertion route is proved — a typed
frame that enters through the typed adapter is retired by the adapter's drop,
which erases the binding before address reuse; a frame type that can also be
inserted as a raw frame takes the Q26 discipline in that route, and a route
whose only instrumentable file lies outside the admitted 94 paths is a STOP,
not a fifth path. The same pass completes every declared ownership transport,
including the four complex paired rows P0001/P0002/P1566/P1805 outside the
partial paired renderer, then the complete 151-plan generator with zero unknown
edges over the 94-path set, the double generation producing 94 targets and 188
postimages, whole-file AST restoration and the separate pinned-omission audit
in both cfg arms, the mechanically emitted and authenticated 94-row schema with
all 151 `source_schema` cells and the new digest from the exact table bytes, the
ordered standing replay, `cargo fmt --check`, and both workspace modes with the
seven exclusions, the switched checks on the same binaries and the full
unfiltered `suspension_oracle` and `typed_slot_census` at failed=0, filtered=0.
The second workspace envelope stays held for the verification stage, which uses
brief 742's remaining 1,440 s; CM4's committed share stays 28,500 s and 743–746
retain 40,500 s. The stop-loss is the Q28 counter. No fifth overlay, no 138-row
subset, no placeholder digest, no hand-edited derived cell, no weakened control,
no git, no network, no `unsafe`, no dependency or lockfile change.

## Continuation 23 ruling (director) — Q30: the typed transport's permission is retired at the frame's actual abandonment, not deferred to the adapter's drop; continuation 23 implements the corrected teardown and runs to the full bar

Continuation 22 ran `cdx` (unit `brief-742-cdx-20261117-061533-1949553`, rc 0,
peak 444,588,032 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`,
1,423,844 B, sha256 `e13d3a73…`; report STOP, 3,338 changes (3,336 added, 2
modified, 0 deleted — all in `.temp`), 0 disclosures, 1 deviation, 0 questions,
11 PASS / 2 FAIL / 2 NA, 300 citations verified; 89.676839 s charged — 16.176839 s
measured serial over 27 runs plus 13.5 s of per-receipt 0.5 s and the 60 s
preparation/reporting reserve — leaving 7,961.143161 s of the 8,050.82 s focused
envelope; workspace 0 of the assigned 6,000 s; the 1,440 s verification envelope
and the second 6,000 s workspace stay held). The deviation is accepted: the
complete generator and the mechanical 94-row schema are absent because the Q28
stop fired before any maintained edit, and no subset, weakened control or
placeholder schema was substituted. Scope holds: the director found the working
tree at the same 23-entry partial CM4 test layer as entry with no maintained
change, `production_changes=0`, `source_schema` still `f9604852…` with
`new_source_schema` null, the 94 admitted paths all enumerated `not_emitted`
(90 retained + 4 proposed-extension) with two stopped partials and both inside
the admitted set, and the three document drifts are the director's own gate
commits.

The finding is `PortBlocked::TypedAdapterAbandonmentWithRetainedArc` for P0396,
and it is admitted as an executed candidate-contract finding, not a production
regression or a new catch. The director reproduced all six forms at one thread:
armed hash and armed no-hash each 16 passed / 1 failed with `retired_hits=1`
against the required 0 and `live_hits=1`; disabled and live, hash and no-hash,
each 17 passed / 0 failed, with the semantic baseline recorded — with the alias
retained, the abandoned frame's store executes and the target moves
`List[6]` → `List[7]`; without it, `List[5]` → `List[6]`. In the armed form the
abandoned frame's real store is suppressed (`List[5]` stays), so the defect is
semantic, not a counter artifact. The control is exact: it runs the actual
P0396 producer, construction, typed insertion, adapter allocation and actual
`abandon` (returning `Discard`), retains the adapter alias, then resumes that
alias and measures the omission; an independent live allocation in the same run
measures its own one hit. The fixture's copied abandonment is the production
text `self.inner.clone().abandon(context)`, and the live source has the same
shape (`suspension/typed/adapter.rs:27-31`), reached from the driver through
`Session::abandon_frames` → `Entry::abandon` → `TypedAdapter::abandon`
(`core/abandon.rs:92`, `typed/entry.rs:30-39`), and `Entry` is `Clone`, so an
abandoned adapter can remain alive through another strong reference. The
fixture's only erasure is the registered `Drop`, so an abandoned-and-retained
adapter keeps its operation binding and can claim it. The inherited fifteen
controls and their source functions are byte-identical to continuation 21's
fixture (the whole diff is one added control function and two added tests), and
the five original controls still pass.

Q30 rules that the typed transport's permission is retired at the frame's
actual abandonment, not deferred to the adapter's Drop. The retained-alias
falsifier is the same one Q26/Q27 impose on the raw transport, and the typed
transport must satisfy it: after an actual `abandon`, a retained adapter alias
can neither claim the operation nor have its store omitted, in both cfg arms,
while an independent live allocation keeps exactly its one hit and an ordinary
Arc alias drop retires nothing. The retirement happens at the adapter's actual
`FrameAbandon` implementation before it delegates to the inner frame; the drop
erasure remains the backstop for allocations that never reach abandonment. This
refines Q29 rather than reopening it: the operation-owned one-use token, the
mint/claim/carry/consume sites, the unchanged pinned omission, the five original
controls, the fifteen inherited controls, the admitted 50-row map and the
per-row insertion-route duty all stand unchanged. The fix is one registered
instrumentation edit inside the admitted 94 (`suspension/typed/adapter.rs`),
declared against entry-pinned source with its erasure rule and whole-file AST
restoration in both cfg arms; no production source, runner hook, predicate or
phase changes, and no control or expectation may be edited to pass.

Continuation 23 resumes the implementation and runs to the full bar under the
remaining 7,961.143161 s focused plus the untouched 6,000 s workspace envelope.
First the corrected teardown: the abandonment-site retirement, its registered
edit and erasure audit, then all seventeen controls in armed, disabled and live
each crossed with hash and no-hash at eight and at one thread — the two new
controls at their fixed required values, the retired store executing exactly as
its disabled/live counterpart, and no other control moving. Then the duty
continuation 22 did not reach: every typed row's own executed controls at full
strength in both cfg arms and the disabled and live forms, with each row's
insertion route proved (a typed frame entering through the typed adapter is
retired at its abandonment and its drop remains the backstop; a frame type that
can also be inserted as a raw frame takes the Q26 discipline in that route; a
route whose only instrumentable file lies outside the admitted 94 paths is a
STOP, not a fifth path); every declared ownership transport including the four
complex paired rows P0001/P0002/P1566/P1805; the complete 151-plan generator
with zero unknown edges over the 94-path set; the double generation producing 94
targets and 188 postimages; whole-file AST restoration and the separate
pinned-omission audit in both cfg arms; the mechanically emitted and
authenticated 94-row schema with all 151 `source_schema` cells and the new
digest from the exact table bytes; the ordered standing replay; `cargo fmt
--check`; and both workspace modes with the seven exclusions, the switched
checks on the same binaries and the full unfiltered `suspension_oracle` and
`typed_slot_census` at failed=0, filtered=0. The second workspace envelope stays
held for the verification stage, which uses brief 742's remaining 1,440 s;
CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The stop-loss
is the Q28 counter: a compile attempt retiring at least one outstanding distinct
error counts as progress; removing or weakening a control is a contract change
that stops; a rejected copied production source or instrumentation edit is a
finding that stops; two consecutive non-reducing attempts stop; the envelope
stays the hard bound. No fifth overlay, no 138-row subset, no placeholder
digest, no hand-edited derived cell, no weakened control, no git, no network,
no `unsafe`, no dependency or lockfile change.

## Continuation 24 ruling (director) — Q31: arming is scoped to the actual producer entry, so a pause under a resume-restored activation neither arms nor advances the occurrence; continuation 24 implements it and runs to the full bar

Continuation 23 ran `cdx` (unit `brief-742-cdx-20261117-063532-1956045`, rc 0,
peak 742,801,408 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`,
2,432,504 B, sha256 `21c3e0c6…`; report STOP, 3,887 changes (3,885 added, 2
modified, 0 deleted — all in `.temp`), 0 disclosures, 1 deviation, 0 questions,
14 PASS / 2 FLAGGED / 2 FAIL / 2 NA, 1,986 citations verified; 220.760382 s
charged — 69.260382 s measured serial over 183 runs plus 91.5 s of per-receipt
0.5 s and the 60 s preparation/reporting reserve — leaving 7,740.382779 s of the
7,961.143161 s focused envelope; workspace 0 of the assigned 6,000 s; the
1,440 s verification envelope and the second 6,000 s workspace stay held). The
deviation is accepted: the complete generator and the mechanical schema are
absent because the Q28 stop fired before any maintained edit, and no subset,
weakened control or placeholder schema was substituted. Scope holds: the
working tree is the same 23-entry partial CM4 test layer as entry, zero
non-`.temp` change paths, no `.temp/brief-747` artifact touched,
`production_changes=0`, `source_schema` still `f9604852…` with
`new_source_schema` null, all 94 admitted paths enumerated as complete output
not emitted, the ten intermediate roots' 44 outputs enumerated with 52
current-pin byte comparisons, and the three document drifts are the director's
own gate commits.

Q30 is implemented and admitted. The director reproduced all six base forms at
one thread: 17/0/0 in armed, disabled and live each crossed with hash and
no-hash, with the retained-alias control at `retired_hits=0` and `live_hits=1`,
the abandoned frame's store actually executing (`List[6]`), the live selected
operation's store omitted, and exact disabled/live record parity. Fifteen
registered edits (the fourteen inherited Q29 edits plus the abandonment-site
retirement) render from entry-pinned preimages; both whole source files restore
byte-for-byte and all three restore as ASTs in both cfg arms; two roots each
hold three paths/six postimages byte-identical with only the `ENABLED` constant
differing; the eighteen controls are exact and an added active-resume
retirement check passes; the one fixture-only `E0603` visibility error was
repaired inside fixture scaffolding with both attempts retained. The shared
partial instruments the actual P0394 outer call and P0396 selector call in the
same copied file and both rows' eighteen controls pass all six forms, which is
partial path evidence, not row admission.

The finding is `PortBlocked::TypedResumeProducerScopeLeak` for P0396, and it is
admitted as an executed candidate-instrument failure, not a production
regression and not a new own catch. The director reproduced all six forms at
one thread: armed and disabled, hash and no-hash, each 18 passed / 1 failed
with `initial_arms=1`, `after_resume_arms=2`, required 1; live 19/0/0 with
`initial_arms=0`, `after_resume_arms=0`; and in every form `final_hits=0`,
`final_value=List[6]`, `result=6`. The route is authenticated by the worker's
own source audit (the second store is distinct, it never reaches the pinned
omission, and the resume stack is used by `arm`): the actual producer branch in
`current_held_update_value` enters `enter_selected("P0396")` around the
selector evaluation and its actual `InputRequired` constructs the typed frame;
on resume the copied adapter's `resume_frame` pushes the saved token owner into
the ledger stack; the resumed evaluation of the held update then reaches its
own operand `Input[]`, which genuinely pauses; `pause()` calls `arm()`, whose
`owner()` finds the resume-restored activation, increments `next_occurrence`,
overwrites the pending token and calls `join_paused`. The second frame is
`FinishHeldUpdateSuspensionFrame` and its `store_suspended_held_update` is a
distinct body that never reaches the pinned omission, so its zero hits and
`List[6]` are correct. The earlier extra control `actual_operation_resuspends`
with `required_final_hits=1` at that distinct store was invalid; its four armed
reds are instrument errors, it is withdrawn with every artifact retained, and
it earns no credit.

Q31 rules that Deliverable 3 and Q19 already fix the contract: an episode is
armed only by the actual `InputRequired` belonging to the producer scope, and
an inherited episode's later constructions are silent, so the single pause
stays the only arm event. A pause executed under a resume-restored activation
therefore neither arms nor advances the occurrence counter for the selected
row, and the frame constructed there inherits silently; the actual producer
entry still arms exactly once. The arming scope is the actual producer entry
(the `enter_selected` guard at the branch), never the resume-restored
activation. The mechanism may not be a blanket suppression of arming for the
whole resume: a genuine producer entry executed inside a resumed evaluation
still arms, and the implementation states the executed route that shows it, or
records the absence of such a route as a named limitation — an unexecuted
generality is credited neither way. Q29 and Q30 stand unchanged: the
operation-owned one-use capability, the unchanged pinned omission, the
abandonment retirement at the adapter's actual `FrameAbandon`, the drop
backstop, the fifteen registered edits and their erasure, and the seventeen
fixed controls with their six-form results. The arm-scope control is fixed at
`initial_arms=1`, `after_resume_arms=1` in armed and disabled, 0/0 in live,
`final_hits=0`, `final_value=List[6]`, `result=6`, in both cfg arms at eight and
one threads. The withdrawn extra control stays withdrawn and is never credited,
and no control or expectation may be edited to pass.

Continuation 24 resumes the implementation under the remaining 7,740.382779 s
plus the untouched 6,000 s workspace envelope: the arming-scope fix, its
registered edit and erasure audit, then all eighteen valid controls plus the
fixed arm-scope control re-executed in six forms at eight and one threads; then
every typed row's own executed controls at full strength in both cfg arms and
the disabled and live forms with each row's insertion route proved (a typed
frame entering through the typed adapter is retired at its abandonment and its
drop is the backstop; a frame type that can also be inserted as a raw frame
takes the Q26 discipline in that route; a route whose only instrumentable file
lies outside the admitted 94 paths is a STOP, not a fifth path); then every
declared ownership transport including the four complex paired rows
P0001/P0002/P1566/P1805, the complete 151-plan generator with zero unknown
edges over the 94-path set, the double generation producing 94 targets and 188
postimages, whole-file AST restoration and the separate pinned-omission audit
in both cfg arms, the mechanically emitted and authenticated 94-row schema with
all 151 `source_schema` cells and the new digest from the exact table bytes, the
ordered standing replay, `cargo fmt --check`, and both workspace modes with the
seven exclusions, the switched checks on the same binaries and the full
unfiltered `suspension_oracle` and `typed_slot_census` at failed=0, filtered=0.
The second workspace envelope stays held for the verification stage, which uses
brief 742's remaining 1,440 s; CM4's committed share stays 28,500 s and 743–746
retain 40,500 s. The stop-loss is the Q28 counter: a compile attempt retiring at
least one outstanding distinct error counts as progress; removing or weakening
a control is a contract change that stops; a rejected copied production source
or instrumentation edit is a finding that stops; two consecutive non-reducing
attempts stop; the envelope stays the hard bound. No fifth overlay, no 138-row
subset, no placeholder digest, no hand-edited derived cell, no weakened control,
no git, no network, no `unsafe`, no dependency or lockfile change.

## Continuation 25 ruling (director) — Q32: the direct-frame claim authenticates the transported identity against an independent construction record, not its own restored copy; continuation 25 implements it and runs to the full bar

Continuation 24 is gated as a compliant Q28 STOP. Q31 is implemented and admitted: ten declared helper edits give the probe a producer-provenance stack (an actual producer entry opens an arming scope, a resumed or deferred owner is a boundary, and `arm` requires an actual producer owner), the helper's inverse audit restores its pre-edit pin and the fixed nineteen controls pass in all six forms at eight and one thread at 19/0/0, with the arm-scope control at `initial_arms=1 after_resume_arms=1` in both instrumented arms and 0/0 in live, `final_hits=0`, `final_value=List[6]`, `result=6` throughout; the director reproduced the twelve fixed-nineteen runs, the three rejecting binaries and the 170 inherited plus 70 profile controls at both thread counts. The scope-control set with the added nested-producer control passes 20/0/0 in all six forms at both thread counts. The three weakened helpers reject exactly their named control — `actual_pause_remains_in_producer_scope` for the ambient-arm weakness and `actual_producer_entry_inside_resume_arms` for the blanket-resume and no-inheritance weaknesses — so a blanket suppression of arming during resume is excluded by an executed falsifier. A genuine producer entered inside a resumed call still arms: `initial_arms=1`, `nested_arms=2`, occurrences `[0, 0, 1]`, and exactly the nested operation's one hit, executed through the copied producer, constructor and resume functions with the evaluator leaf recorded as fixture code and no general interpreter claim. Both partial roots hold three paths and six postimages byte-identical with only the compile-time enable differing; the sixteen registered edits are unchanged from continuation 23's manifest; whole-file byte restoration, the helper restoration, the independent omission pins and the AST restoration of all three files pass in both cfg arms. Q29 and Q30 stand unchanged: the operation-owned one-use capability, the abandonment retirement at the adapter's actual `FrameAbandon`, the drop backstop, the sixteen registered edits, the ten declared helper edits with their erasure, and the fixed controls with their six-form results.

The executed finding `PortBlocked::TypedDirectFrameIdentitySelfAuthentication` for P0239 and P0245 is admitted. The two required wrong-identity controls change only the transported frame token after the actual construction — a wrong producer occurrence and, separately, a wrong activation — and the independent construction proof recorded in `run.pair.tokens` stays unchanged. `bind_frame` records that proof and inserts the token in `LEDGER.frames`; `resume_frame` restores the token from that same entry; `claim_typed_frame` reads the same entry and `current_token` requires `frames.get(&frame) == Some(token)` — the same map — so the claim authenticates the restored token against its own copy and never consults the construction proof; the worker's own source audit records exactly this (`claim_compares_restored_token_to_same_map=true`, `construction_proof_not_checked=true`). Both rows, both cfg arms and both thread counts reproduce armed 7 passed / 2 failed / 0 filtered (the director reproduced all twenty-four runs); each wrong-identity control records `hits=1` against required 0, `body_calls=0` against required 1, and the retained RHS `Some42` instead of the real leaf `Some43`, so the accepted capability suppresses real work; disabled and live each pass 9/0/0 with all nine records identical, and the six initial controls per row plus the abandonment isolation stay green in every form. No candidate repair followed the first failure; the candidate is frozen and no control was weakened.

Q32 rules the direct-frame identity contract. The transported frame identity must be authenticated against a second, independently recorded copy of the identity made at the actual construction site and not derived from, supplied by, or rewritten by the resume path — or against the identity re-derived at claim time from the actual constructed frame's own operation and target data, as Q29 binds the typed operation; the ledger's restored copy may not be its own evidence. A transported occurrence or activation that disagrees with that record yields no capability: zero hits, the real accepted-body evaluation executing (`body_calls=1`, `result=Some43`), in both cfg arms, with disabled and live record parity exact. The fix must keep the actual control's single claim (`hits=1`, `body_calls=0`, `result=Some42`, the store omitted), the repeated resume's single claim, the nested unarmed frame's rejection, the independent episode's rejection, the abandonment retirement with the retained-arc isolation, and the six initial controls per row at their fixed values; the two wrong-identity controls are fixed at `hits=0`, `body_calls=1`, `result=Some43`, and no control or expectation may be edited. The implementation states which record or re-derivation it chose and why the resume path cannot rewrite it, and the falsifier is these two controls applied to the ledger copy with that record unchanged, rejecting at eight and one thread in both cfg arms. The same identity duty covers every later direct-frame transport: each row's insertion-route proof includes the wrong-occurrence and wrong-activation falsifiers with disabled/live parity, not only the six initial controls; a transport that cannot meet it is a named PortBlocked, never a weakened control.

Continuation 25 resumes under the remaining 7,557.702523 s plus the untouched 6,000 s workspace envelope: the corrected direct-frame identity claim and its registered edit and erasure audit, then the two default rows' full controls in six forms at eight and one thread with the fixed wrong-identity expectations; then every remaining typed row's own executed controls at full strength in both cfg arms and the disabled and live forms with each row's insertion route proved (a typed frame entering through the typed adapter is retired at its abandonment and its drop is the backstop; a frame type that can also be inserted as a raw frame takes the Q26 discipline in that route; a route whose only instrumentable file lies outside the admitted 94 paths is a STOP, not a fifth path); then every declared ownership transport including the four complex paired rows P0001/P0002/P1566/P1805 outside the partial paired renderer, the complete 151-plan generator with zero unknown edges over the 94-path set, the double generation producing 94 targets and 188 postimages, whole-file AST restoration and the separate pinned-omission audit in both cfg arms, the mechanically emitted and authenticated 94-row schema with all 151 `source_schema` cells and the new digest from the exact table bytes, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven exclusions, the switched checks on the same binaries and the full unfiltered `suspension_oracle` and `typed_slot_census` at failed=0, filtered=0. The second workspace envelope stays held for the verification stage, which uses brief 742's remaining 1,440 s; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The stop-loss is the Q28 counter: a compile attempt retiring at least one outstanding distinct error counts as progress; removing or weakening a control is a contract change that stops; a rejected copied production source or instrumentation edit is a finding that stops; two consecutive non-reducing attempts stop; the envelope stays the hard bound. No fifth overlay, no 138-row subset, no placeholder digest, no hand-edited derived cell, no weakened control, no git, no network, no `unsafe`, no dependency or lockfile change. The next step is dispatching brief 742 continuation 25 and then its gate, followed by the verification stage and then 743–746. No campaign, epoch, new catch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 26 ruling (director) — Q33: a construction with no token of its own inherits the enclosing episode, and the explicit construction proof may not shadow it; continuation 26 is the `Scope: NONE` construction-context inventory at 1,200 s

Continuation 25 is gated as a compliant Q28 STOP. Q32 is implemented and admitted: one new registered helper edit at `claim_typed_frame` compares the transported token's full identity with `run.pair.tokens`, which `bind_frame` wrote at the actual Arc construction, and also checks the claim's actual allocation and type; `resume_frame` can read the transported map but cannot replace the construction record (`Q32 INDEPENDENCE PASS proof_writer=bind_frame actual_construction_only=true actual_allocation_type_checked=true resume_cannot_replace_record=true full_token_equality=true falsifier_changes_only_transported_copy=true`), and the source audit keeps 134 of 135 functions exact with the one registered claim change (`Q32 SOURCE PASS ... controls_exact=true actual_producer_constructor_resume_abandon_bodies_exact=true`). The director reproduced all twenty-four default runs at 9/0/0 in six forms at eight and one thread with both wrong-identity controls at `hits=0`, `body_calls=1` and `result=Some43`, the actual control's single claim unchanged; the deliberately weakened helper rejects exactly the two identity controls in both cfg arms for both rows (`Q32 CALIBRATION PASS ... other7_green=true`); the whole-file erasure audit restores all three files byte-exactly in both roots and arms, the double generation is byte-identical with only the compile-time enable differing, and the independent omission pins and AST restorations pass. The inherited P0396 twenty controls are 20/0/0 in all six forms and the P0278/P0281 replacement transports 14/0/0 in all six forms, at both thread counts (the director reproduced both full matrices). The deviation is accepted: the complete generator and the mechanical schema are absent because the Q28 stop fired before any maintained edit, and no subset, placeholder schema or weakened control was substituted. The FLAGGED construction errors are settled and are not a red: append-anchor handling in the renderer, an absolute-path launch, a fixture-private return type, an owner-aware AST comparator and a command-trailer collector were corrected across retained attempts, with no copied production source or registered-edit rejection and no expectation changed.

The executed finding `PortBlocked::TypedResuspensionInheritanceShadowed` for P0239 is admitted. The added control is an actual-path succession, not the withdrawn distinct-store case: a selected producer is entered and constructs its frame; that frame resumes with a rejecting guard outcome; the next actual guard call pauses and the actual `ContinueToken::suspend` constructs a successor of the same concrete frame type; the successor resumes with a matching outcome into the same pinned accepted-body omission. With P0239 selected, `enter_selected` at the resumed P0245 site returns `Guard(None, None)`; `Guard::suspending` pushes an explicit `None` construction; `bind_frame`'s `Some(explicit)` branch chooses it and returns without binding the new frame instead of reaching the inherited resume-token branch, so the successor is never bound. The director reproduced all twenty-four resuspension runs: armed hash and no-hash at eight and one thread each give 9 passed / 1 failed / 0 filtered with `initial_arms=1`, `after_arms=1`, `bindings=0`, `proofs=1`, `hits=0` against required 1, `body_calls=1` and `result=Some43`; the counterpart P0245 gives `initial_arms=1`, `after_arms=2`, `bindings=1`, `proofs=2`, `hits=1`, `body_calls=0`, `result=Some42` and passes 10/0 in every form; disabled and live pass 10/0 for both rows at their required hits 0, with the nine original records exact and the raw arm and binding diagnostics retained separately rather than claimed byte-identical. The source audit authenticates the original functions, the actual state movement, constructor, adapter, witness, token and callee, with the evaluator, host and codec leaves recorded as fixture code; the nine original default controls are unchanged and the candidate is frozen after the first red with no repair and no weakened assertion. This is a candidate-instrument failure, not a production regression, a new own catch or a universal-impossibility claim.

Q33 rules the construction contract. A construction that has no token of its own and occurs under an inherited evaluation must inherit the enclosing episode's identity silently — no arm and no occurrence advance — and the constructed frame must be bound so that it reaches the same pinned omission exactly once; the explicit construction proof records identity and may not shadow that inheritance. A construction with no token and no enclosing inherited evaluation stays unbound and claims nothing. An actual selected producer entry still supplies its own token and arms exactly once. The Q32 identity protocol stands: the inherited identity is recorded in the construction proof at bind with the new frame's own identity, and the claim still checks the full identity against the proof. The successor control is fixed at `initial_arms=1`, `after_arms=1`, `bindings=1`, `proofs=2`, `hits=1`, `body_calls=0`, `result=Some42` for P0239 in both armed cfg arms at eight and one thread; the P0245 counterpart stays at `initial_arms=1`, `after_arms=2`, `bindings=1`, `proofs=2`, `hits=1`, `body_calls=0`, `result=Some42`; disabled and live keep required hits 0 with the exact records, and the raw arm and binding diagnostics stay recorded in their own fields. The two wrong-identity controls stay at `hits=0`, `body_calls=1`, `result=Some43`; the actual control keeps its single claim; the six initial controls per row and the abandonment isolation stay at their fixed values; no control or expectation may be edited.

Because this is the fourth finding in the same identity-and-inheritance class, continuation 26 is a `Scope: NONE` construction-context inventory, not an implementation: 1,200 s allocated from the focused envelope, with unused time returned; the remainder is 6,130.267849942131 s before any return. It inventories every typed-frame construction and every context in which it occurs — an actual selected producer entry with a pending token; an actual non-selected producer entry inside a fresh evaluation; an actual non-selected producer entry inside another row's resumed evaluation; a resumed owner's own construction; a deferred owner; a nested operation; a successor constructed by `ContinueToken::suspend`; a fresh run or episode; and abandonment or drop — with, for each, the actual object, the source edge, whether it arms, inherits silently or is unbound, the identity the construction proof carries, and an executed falsifier or a named limitation. Zero unknown contexts, over the admitted 94 paths and the fifty typed rows; a context the mechanism cannot serve is a named PortBlocked. The design states the complete inheritance rule, shows that the Q32 claim still authenticates an inherited identity, and drafts the implementation brief. No maintained edit, no workspace command; read-only inventories and existing fixture binaries may be used within the envelope.

Continuation 27 then implements the rule under the remainder and resumes the full bar: the successor control fixed as above, all original default controls, the inherited P0396 and P0278/P0281 transports, then every remaining typed row's own executed controls at full strength in both cfg arms and the disabled and live forms with each row's insertion route proved (a typed frame entering through the typed adapter is retired at its abandonment and its drop is the backstop; a frame type that can also be inserted as a raw frame takes the Q26 discipline in that route; a route whose only instrumentable file lies outside the admitted 94 paths is a STOP, not a fifth path); then every declared ownership transport including the four complex paired rows P0001/P0002/P1566/P1805 outside the partial paired renderer, the complete 151-plan generator with zero unknown edges over the 94-path set, the double generation producing 94 targets and 188 postimages, whole-file AST restoration and the separate pinned-omission audit in both cfg arms, the mechanically emitted and authenticated 94-row schema with all 151 `source_schema` cells and the new digest from the exact table bytes, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven exclusions, the switched checks on the same binaries and the full unfiltered `suspension_oracle` and `typed_slot_census` at failed=0, filtered=0. The second workspace envelope stays held for the verification stage, which uses brief 742's remaining 1,440 s; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The stop-loss is the Q28 counter: a compile attempt retiring at least one outstanding distinct error counts as progress; removing or weakening a control is a contract change that stops; a rejected copied production source or instrumentation edit is a finding that stops; two consecutive non-reducing attempts stop; the envelope stays the hard bound. No fifth overlay, no 138-row subset, no placeholder digest, no hand-edited derived cell, no weakened control, no git, no network, no `unsafe`, no dependency or lockfile change. The next step is dispatching brief 742 continuation 26 and then its gate, then continuation 27 and its gate, followed by the verification stage and then 743–746. No campaign, epoch, new catch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.
## Continuation 27 ruling (director) — continuation 26's construction-context inventory is accepted (99 sites, 900 cells, zero unknown contexts); Q33 is implemented as the absent-vs-invalid-present resolution with parent-proof authentication and an independent successor record; continuation 27 resumes the full bar

Continuation 26 is gated as DONE. The `Scope: NONE` construction-context inventory is accepted as ordered: 50 typed rows over 45 concrete frame types and 99 named construction/factory sites — 94 inside the admitted paths, five outside and read-only — with 57 sites carrying typed rows and 312 named references; the fifty-by-eighteen cross-product is 900 cells with zero unknown contexts, each carrying its actual object, source construction ids, producer and omission sites, arm/inherit/unbound effect, proof identity and either executed evidence or a named limitation (232 cells quote receipts, 719 name `Limit::PerRowContextNotExecuted`-class limits, and the design states plainly that this is finite-inventory classification, not dynamic closure). The director re-derived the counts from `constructions.tsv`, `typed-contexts.tsv`, `typed-routes.json`, `admitted-paths.tsv` and `reference-census.json`; verified all 94 admitted paths byte-identical to their entry pin and `not_emitted`; verified the four projected rows P0394/P0396/P0435/P1818 are the only external-read-only routes and that no typed row declares a raw implementation; rehashed all 5,400 maintained, 590 accepted-design and 3,813 continuation-25 files with zero mismatches; confirmed the three document differences are this director's own gate commits; re-executed the P0239 armed red (9 passed / 1 failed with `initial_arms=1 after_arms=1 bindings=0 proofs=1 hits=0 required_hits=1 body_calls=1 result=Some43`) and the P0245 counterpart (10/0 at `initial_arms=1 after_arms=2 bindings=1 proofs=2 hits=1 body_calls=0 result=Some42`) at one thread; verified all 24 fixture binaries against the recorded sha256 and the 84 replay executions as exactly four reds (P0239 armed, both cfg arms, both thread counts) and eighty green with no missing or mismatched log; and checked the ledger arithmetic exactly (45.67156936204992 s measured over 104 commands plus 52.0 s bookkeeping plus the 60 s reserve = 157.67156936204992 s charged; 1,042.32843063795 s returned; 7,172.596280580081 s remainder). Run `brief-742-cdx-20261117-085419-2002084`, rc 0, peak 287,141,888 B, oom 0 / oom_kill 0, log 1,603,350 B sha256 `bad5f04a…`; report DONE, 270 changes (268 added, 2 modified, all in `.temp`, zero non-`.temp` paths), 0 disclosures, 0 deviations, 0 questions, 7 PASS / 2 FLAGGED / 1 NA, 1,656 citations verified; workspace 0 of 6,000 s, no cargo, build or workspace command. Both FLAGGED items are settled and neither is a red: the P0239 non-pinned successor records are the already-admitted `PortBlocked::TypedResuspensionInheritanceShadowed` being inventoried, and the enumeration correction from 98 to 99 sites adds the read-only `FinishHeldUpdateSuspensionFrame` auxiliary construction plus its three references, with the initial attempt retained in `inventory-attempt-1`. The final inventory is the 99-site set; the earlier prose that still says 98 is stale wording, not a data cell, and no table may be reverted to it.

Q33 is implemented as follows, and the design's rule is adopted. Resolve every typed-frame construction at its actual witnessed Arc creation with a three-state distinction that may never be conflated: an absent token is not an invalid present token, and an unbound resume boundary is not an empty producer construction scope. An actual valid own pending token — minted by the selected producer's actual pause — supplies the identity and a genuine nested selected producer wins with its own occurrence. An explicit construction scope with no token supplies no identity and does not veto inheritance: consult the immediately enclosing authenticated evaluation provenance; a producer scope without a token stays transparent to inheritance while remaining a distinct arming scope under Q31. An immediately enclosing resumed owner supplies its token only after validating episode, full identity and live allocation against its independent construction proof; never search through an unbound nested resume boundary for an older outer token; a valid deferred owner supplies only the proof actually transported by its declared Box/type edge, and a missing deferred token is absence while a wrong or stale present token is a rejection that may not borrow another owner's identity. With no own and no authenticated enclosing identity the construction stays unbound. Binding happens once at the new actual `Arc<F>`, preserves episode, origin and producer occurrence, attributes the construction to its actual scoped owner (an empty selected Guard must not synthesize one), sets `frame_identity` from the new Arc and concrete type and never from a plan, writes the new full identity to the independent construction record and the separate transported map, and neither binding nor an inherited construction may change the arm count or the occurrence counter. Before inheriting, the parent must be authenticated against its already-recorded independent proof and a live binding; the successor's record is appended independently, resume may restore copies but never replace either record, and a changed parent must fail before a successor proof is written. Rebase to a different actual child owner only at its proven source edge and only with its own executed control; if no such route exists in the admitted set, the code rejects and the case remains a named limitation — a relaxed stack assertion or a search for any selected ancestor is forbidden. No typed row declares a declared Boxed cursor route, so a typed deferred inheritance may not be invented from the eighteen helper controls; those receipts stay separate evidence. Abandonment keeps Q30: actual `TypedAdapter::abandon` retires before delegating, Drop is the backstop, an ordinary Arc alias drop retires nothing, and a new independent run resets eligibility. Q29's operation-owned capability, Q30's retirement, Q31's arming scope and Q32's claim identity all stand unchanged.

The controls are fixed and may not be edited: P0239's corrected successor must reach `initial_arms=1`, `after_arms=1`, `bindings=1`, `proofs=2`, `hits=1`, `body_calls=0`, `result=Some42` in both armed cfg arms at eight and one thread; P0245 stays at `initial_arms=1`, `after_arms=2`, `bindings=1`, `proofs=2`, `hits=1`, `body_calls=0`, `result=Some42`; disabled and live keep required hits 0 with the exact records and the raw arm, binding and proof diagnostics in their own fields; the two wrong-identity controls stay at `hits=0`, `body_calls=1`, `result=Some43`; the actual control keeps its single claim; the nine original default records, the six initial controls per row, the abandonment isolation, the P0396 twenty controls with the nested-producer arm-scope control and the distinct-store control, and the P0278/P0281 fourteen replacement controls stay at their fixed values. Beside them — never replacing them — continuation 27 adds the executed controls the design names: empty own scope with and without an inherited owner; actual non-selected producer entry in a fresh evaluation and inside another row's resumed evaluation; selected own-token precedence; the nested unbound-resume barrier; resumed and deferred construction; stale and cross-constituent identity; corrupted parent occurrence/activation before successor bind with no new proof and no hit; corrupted successor occurrence/activation after bind rejected by the Q32 claim; a real child-owner construction only if its route exists; abandonment while resuming and a later construction; ordinary alias drop versus final drop. The weaknesses are calibrated red against their named controls: empty-scope veto, ambient inheritance, self-authentication and blanket resume suppression each reject exactly their control, with the valid complete counterparts green. `PortBlocked::InheritedConstructionIdentityNotExecuted` is closed only by executed corrected-successor, corrupted-parent and corrupted-successor receipts; if the envelope stops first it stays a named PortBlocked and is reported as such, never claimed.

Continuation 27 then resumes the full bar under the returned remainder: the rule implemented in the generated helper recipes with every edit registered against the entry pin and its inverse erasure proved; the fixed successor and all original controls in six forms at eight and one thread; the new controls above; every remaining typed row's own executed controls at full strength in both cfg arms and the disabled and live forms with each row's insertion route proved (a typed frame entering through the typed adapter is retired at its abandonment and its drop is the backstop; a frame type that can also be inserted as a raw frame takes the Q26 discipline in that route; a route whose only instrumentable file lies outside the admitted 94 paths is a STOP, not a fifth path); then every declared ownership transport including the four complex paired rows P0001/P0002/P1566/P1805 outside the partial paired renderer; the complete 151-plan generator with zero unknown edges over the 94-path set; the double generation producing 94 targets and 188 postimages; whole-file byte and AST restoration and the separate pinned-omission audit in both cfg arms; the mechanically emitted and authenticated 94-row schema with all 151 `source_schema` cells and the new digest taken from the exact table bytes; the ordered standing replay before the first acceptance suite; `cargo fmt --check`; and both workspace modes with the seven exclusions, the switched checks on the same binaries and the full unfiltered `suspension_oracle` and `typed_slot_census` at failed=0, filtered=0. The maintained edit surface is exactly continuation 16's generation directory, the module and schema bindings, the source_auth authentication and export files, the mechanically emitted `overlays.tsv`, only the 151 `source_schema` cells in `mutants.tsv`, and a minimal discovery re-export if required; generated copied instrumentation is restricted to the 94 declared paths; live production, the external projected resume files, `source_worker`, runners, observer/predicate/control bodies, selectors, phases, source keys, tail digests, coverage and promotions, manifests, the lockfile and dependencies are immutable. Every inventory cell stays as classified: no cell is relabeled executed without its own receipt, no table is hand-edited, and the 719 named-limitation cells are not claimed as execution.

The budget is the exact returned remainder: 7,172.596280580081 s focused from continuation 26's ledger, plus the untouched 6,000 s workspace envelope; the 1,440 s verification envelope and its second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The stop-loss is the Q28 counter: a compile attempt retiring at least one outstanding distinct error counts as progress; removing or weakening a control is a contract change that stops; a rejected copied production source or instrumentation edit is a finding that stops; two consecutive non-reducing attempts stop; the envelope stays the hard bound. No fifth overlay, no 138-row subset, no placeholder digest, no hand-edited derived cell, no weakened control, no git, no network, no `unsafe`, no dependency or lockfile change. The next step is dispatching brief 742 continuation 27 and then its gate, followed by the verification stage and then 743–746. No campaign, epoch, new catch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 28 ruling (director) — continuation 27 is a compliant Q28 STOP: Q33 is implemented in scratch with the fixed successor and all 50 typed-row matrices green, and `PortBlocked::PairedBridgeNestedFixtureRouteAssertion` is admitted as a fixture-contract error whose correction is now ordered; continuation 28 corrects the control and resumes the remaining bar

Continuation 27 is gated as a compliant Q28 STOP, not a failure of nerve: the worker stopped on a compiled control that failed against the candidate contract, exactly as Q28 requires. The delivered evidence is accepted as scratch, not as a completed carrier. Q33 is implemented with eleven registered helper edits and inverse erasure to the independent continuation-25 pins (`CM742C27 Q33 INVERSE PASS registered_edits=11 restores_independent_pre_edit_helper_pin=true Q32_claim_unchanged=true controls_unchanged=true`); an absent own token is transparent, a present invalid token rejects, an unbound resumed owner masks older parents, inheritance authenticates the parent's full token against its independent proof and live binding before appending the successor's own actual allocation/type record, the Q32 claim is unchanged, binding neither arms nor advances occurrence, no child-owner rebase or typed Boxed-cursor route is invented, and actual adapter abandonment still retires before delegation with Drop as the backstop. The fixed successor control passes: the director re-executed P0239 armed at one thread and read `initial_arms=1 after_arms=1 bindings=1 proofs=2 hits=1 required_hits=1 body_calls=0 result=Some42`, and P0245 as `initial_arms=1 after_arms=2 bindings=1 proofs=2 hits=1 body_calls=0 result=Some42`; the disabled forms record `bindings=1 proofs=2 hits=0 body_calls=1 result=Some43` and the live forms `0/0/0` with the probe absent. The twenty context controls pass 20/0/0 in all twenty-four forms at eight and one thread, including the corrupted-parent, corrupted-successor, nested-unbound-barrier, fresh-empty-producer, abandon-during-resume and alias-versus-final-drop controls; the four deliberate weaknesses fail their named controls (16/4, 4/26, 18/2 and 8/12 on the full suites; the named filtered runs are 0/1) with the corrected counterparts green. All fifty typed rows completed their own six-form matrices: 600 runs, 822 controls × 12 runs = 9,864 passed, zero failed or filtered, confirmed both by the receipt log's arithmetic and by the director's parse of the report's run records; P0396's twenty, P0278/P0281's fourteen, the held and positional and association families are all green; the director re-ran P0001 at 15/0. The whole-file erasure restores all three defaults files byte-exactly in both roots and arms, the default omission pins pass for both rows, the double generation is 3 paths / 6 postimages byte-identical with only the enable difference, the 911-instance AST audit passes across nineteen fixture families, the 770 enumerated copied outputs (312 partial postimages, 116 source copies, 314 restored preimages, 28 frozen copies) touch 34 admitted paths with 892 exact pin comparisons and zero additional paths, and `source_schema` stays `f9604852…` with 151 cells, `new_schema=null` and no hand-edited derived cell.

The finding is admitted as `PortBlocked::PairedBridgeNestedFixtureRouteAssertion`, a fixture-contract error, not a production regression. The failing assertion is fixture-only, in the `actual()` observation wrapper: `s.accounting_route.as_ref().is_none_or(|r| r.prefixes.is_empty())` (`live.rs:162`, candidate `candidate.rs:182`). P0002's copied `RetentionContext::evaluate` pushes its prefix before calling `Session::evaluate` and pops it when the callback returns — `crates/rum-runtime/src/runtime/accounting_mode/bridge.rs:103` and `:116` — and the production comment states the contract in so many words: "A later arm can see the actual prefix, including every outer bridge in a nested callback chain" (`bridge.rs:98`). In the nested scenario the inner operation returns while the outer callback is still running, so the outer prefix is still on the route; the wrapper's demand for a globally empty route at that boundary contradicts the live contract. The director reproduced the failure in live and armed at one thread (14 passed / 1 failed, panic at the wrapper assertion), confirmed the failure is identical across all six forms including live (12 runs, 14/1, `live_failure=true`), and confirmed the same-binary reproduction (6/6 forms, 0 passed / 1 failed / 14 filtered, `same_binary=true`), the byte-identical freeze of 157 files, and that the failure lands in the unqueued prime before the nested claim or result record, so two P0002 nested hits are not established. P0001 passes all fifteen in six forms.

Continuation 28 corrects the fixture and finishes the bar. The correction is not a weakening: the control's expectation contradicts the live source, so it can never be a control while it demands global emptiness. The wrapper must stop asserting global emptiness and instead assert the actual route invariant — the route depth and prefix contents it observed at entry are unchanged at exit for the ordinary single-bridge calls, and, in the nested call, the route at the inner boundary carries exactly the outer retained prefix (compared by value against the outer prefix, not merely nonempty) — while the root-cleanup obligation moves to the test boundary and is asserted explicitly: after the outer call returns, `accounting_route` is empty. The nested control must establish all of: `(arms, hits) == (0, 2*armed())`, `record.calls == 2`, `record.value == Some(42)`, the outer record's prefix and bytes exactly as prescribed, `nested_records.len() == 1` with the nested record's own prefix and bytes equal to the outer record's (the nested producer returns only its own proof), and the root route empty afterward. The panic/unwind route-cleanup assertion stays exactly as it is, because it concerns the root after an unwind, not a nested boundary. The corrected control must be shown at least as strong: a deliberately route-leaking fixture variant must be red against the root-cleanup assertion, a nested-crossing variant (inner claims the outer proof or occurrence) must be red against the records and counts, a skipped-inner-store variant must be red against `calls`/`hits`, and a nested-prefix-restoration variant must be red against the nested record; the valid candidate and the live form stay green apart from the design's disabled/live values. The frozen `paired-bridge-stop/` directory is immutable evidence and is never edited. In the same pass, the successor control's `bindings` and `proofs` are promoted from printed diagnostics to assertions at the Q33-fixed values: P0239 and P0245 armed `bindings=1`, `proofs=2`, `hits=1`, `body_calls=0`, `Some42`; disabled `bindings=1`, `proofs=2`, `hits=0`, `body_calls=1`, `Some43`; live `0/0` with the probe absent. Existing asserted values are not edited; the additions sit beside them.

All evidence green in continuation 27 is preserved by hash identity and need not be re-executed: the 600 typed-row receipts, the twenty context runs per row, P0396's twenty, P0278/P0281's fourteen, P0001's fifteen, the four weakness calibrations and the 911-instance AST audit. Continuation 28 re-executes only what it touches: the corrected P0002 matrix in all six forms at eight and one thread, P0001's fifteen unchanged controls, the successor matrix with the promoted assertions, and the stopped-path reproductions that the correction replaces. It then resumes the remaining bar: P1566 and P1805 with their own generated source and executed controls, all declared ownership transports, the complete 151-plan generator with zero unknown edges over the 94-path set, the 94-target/188-postimage double generation, whole-file byte and AST restoration with the separate pinned-omission audit in both cfg arms, the mechanically emitted and authenticated 94-row schema with all 151 `source_schema` cells and the new digest from the exact table bytes, the ordered standing replay before the first acceptance suite, `cargo fmt --check`, and both workspace modes with the seven exclusions, the switched checks on the same binaries and the full unfiltered `suspension_oracle` and `typed_slot_census` at failed=0, filtered=0. The maintained edit surface and the immutable set are unchanged from continuation 27's ruling; no fifth overlay, no subset, no placeholder digest, no hand-edited cell, no weakened control.

The run is `brief-742-cdx-20261117-093116-2012564`, rc 0, peak 1,877,667,840 B, oom 0 / oom_kill 0; log 38,194,886 B sha256 `30497d48…`; report STOP, 4,016 changes (4,014 added, 2 modified, zero non-`.temp` paths), 14 PASS / 3 FLAGGED / 2 FAIL / 2 NA, 16,224 citations verified, 0 questions, 2 deviations. Both deviations are accepted: the complete generator's absence is the STOP's consequence and no subset or placeholder was substituted; the unbounded read-only `SequenceMatcher` probe ran without the bounded runner (interrupted at exit 130, no child, write, test or verdict) and is charged at a conservative 300 s upper bound. The director rehashed all 5,400 maintained, 590 accepted-design and 4,080 continuation-25/26 files with zero mismatches, confirmed the three document drifts are this director's own gate commit `0d38d359`, and confirmed the working tree is the same 23-entry partial CM4 layer with `production_changes=0`. The budget is exact: 560.7452842445637 s measured over 1,209 commands plus the 300 s bound plus 604.5 s bookkeeping plus the 60 s reserve = 1,525.2452842445637 s charged, leaving 5,647.350996335517 s of the focused envelope; the 6,000 s workspace is untouched and the 1,440 s verification plus its second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The stop-loss is the Q28 counter: one retired distinct compile error is progress; removing or weakening a control stops; a rejected copied production source or instrumentation edit is a finding that stops; two consecutive non-reducing attempts stop; the envelope stays the hard bound. No new allocation. The next step is dispatching brief 742 continuation 28 and then its gate, followed by the verification stage and then 743–746. No campaign, epoch, new catch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

