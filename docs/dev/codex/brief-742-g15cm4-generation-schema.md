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
