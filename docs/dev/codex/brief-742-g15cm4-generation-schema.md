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

## Continuation 28 ruling (director) — continuation 28 is a compliant Q28 STOP; the ordered P0002 correction and the successor promotion are accepted; `PortBlocked::JoinConfirmationFullIdentityAuthentication` is admitted; continuation 29 is the `Scope: NONE` transported-identity consumer inventory at 1,200 s, then continuation 30 implements Q34 and resumes the bar

Continuation 28 is gated as a compliant Q28 STOP. The ordered corrections are accepted. P0002's fixture now snapshots the actual route at entry and exit, requires exactly the outer retained prefix at the nested boundary, moves the root-cleanup obligation to the test boundary, and keeps the panic/unwind assertion byte-exact; its nested control establishes `(0, 2*armed)` arms/hits, two calls, `Some42`, the outer prefix and bytes, one nested record with its own prefix and bytes, and the empty root; fifteen controls pass in six forms at eight and one thread, and the four deliberate weaknesses each fail their named assertion with the corrected render green: route-leak at the root-cleanup assertion, nested-crossing and skipped-inner-store at `(0,1)` vs `(0,2)`, and nested-prefix-restoration at `[]` vs `[7,8]` (the director reproduced all four at one thread, and the corrected armed and live matrices at 15/0 for both P0001 and P0002). P0001's fifteen are replayed on its unchanged continuation-27 binaries in all twelve runs. The successor control's `bindings` and `proofs` are now asserted, not printed: `bindings == !live`, `proofs == 2*!live` (armed 1/2, disabled 1/2, live 0/0), with every existing assertion untouched; both rows pass 10/0 in all twelve runs. P1566's whole checkpoint caller and P1805's three owned drivers pass their 17-control matrices in six forms at eight and one thread (408 executions), the E expanded 22-control matrices pass 528, the J base 18-control matrices pass 648, and the fixed families plus the two complex paired rows total 1,008 passing control executions. The entry hashes re-derive: 5,400 maintained and 590 design unchanged except the three director document drifts committed in `8b94f5d4`, and 8,093 accepted files rehash exactly; 163 copied-function instances pass the independent signature/body audit; 52 binaries and 395 source/recipe artifacts match their hashes; the 235-file stop snapshot is byte-identical to its manifest; the 335 enumerated copied outputs (108 frozen copies, 94 partial postimages, 39 source copies, 94 restored preimages) touch 13 partial paths over 394 pin comparisons with zero outside the admitted 94; the schema stays `f9604852…` with 151 cells and `new_schema=null`; no maintained source changed and no inventory was hand-edited.

The finding is admitted: `PortBlocked::JoinConfirmationFullIdentityAuthentication`. The new scratch J transport renders the actual pause, completion action, installed generation, registered namespace, append and retained publication omission; its consumer `omit_join` checks the installed queue record, the installed generation and namespace, and `current_episode`'s row/program/pair, but never compares the transported token's owner path or producer occurrence against an independent record. Two new controls change only `confirmed.token` after the actual `TypedAction::complete_work` returns `Action::Complete(Err(InputRequired))`: one appends `("wrong-source", 0)` to the owner path, the other changes occurrence 0 to 1. Both still publish the omission once where zero is required, while the installed owner retains the original `[("P0267", 0)]`, the actual producer, pause, append, generation and namespace are unchanged, and the original eighteen controls stay green. The director reproduced the same-binary failure at eight and one thread (18 passed / 2 failed, the asserted omission count 1 vs 0 at `candidate.rs:1968`), verified the 235-file freeze and manifest, and confirmed that no candidate, helper, control or copied source was repaired after the failure. The two failures are confined to P0267 armed/hash; no result is inferred for P0312/P0323, other cfgs, disabled or live. This is a defect in the new scratch transport's consumer, not a demonstrated production regression and not a stale-inventory ordering artifact.

Q34 rules the consumer contract. Every consumer boundary that confers an effect from a transported token must authenticate the token's complete identity — episode, origin, owner path including the source path, producer occurrence, frame identity, and the installed generation and namespace where applicable — against an independent record written at the actual producer, pause or construction, never against the transported copy it just read. At the join confirmation the installed owner retains the original source path and the construction proof records the producer's identity, so the consumer must compare the whole token against both; the producer occurrence has no independent record at that boundary today and must get one. An absent or unauthenticated identity yields no effect: zero omissions with the disabled and live output equal. A consumer that cannot authenticate the full identity is a named PortBlocked, never a weakened control. Q29, Q30, Q31, Q32 and Q33 stand unchanged.

Because this is another finding in the identity-and-inheritance class and the remaining transports are consumer boundaries of the same shape, continuation 29 is a `Scope: NONE` transported-identity consumer inventory at 1,200 s of the focused remainder, with unused time returned and no workspace command. Over the 94 admitted paths and every declared transport it enumerates every consumer that confers an effect from a transported token — the direct-frame claim, the deferred Box/type-edge transport, the operation-owned one-use protocol, abandonment retirement, producer-entry arming, inherited construction binding, the join confirmation and publication omission, the evaluation-queue publication, and every other consumer those paths make reachable — with, for each, the actual object, the source edge, the independent record available at that boundary, whether the consumer compares the complete identity, the executed corrupted-transport falsifier or a named limitation, and its composition order, with zero unknown consumers. A consumer the mechanism cannot serve is a named PortBlocked. The design states which independent record each consumer will use, why the transported copy cannot be its own evidence, and drafts continuation 30's implementation brief. No maintained edit and no workspace command.

Continuation 30 then implements Q34's consumers and resumes the remaining bar. The frozen `join-queue-stop/` tree (235 files) is immutable; the corrected consumer is a separate render with its own hashes, and the stopped candidate is never repaired. The fixed J controls stay: P0267's actual installed-owner control publishes once, its two full-identity controls require zero omissions, and the original eighteen controls per row stay unchanged. The corrected render's expanded 20-control matrices must pass for P0267, P0312 and P0323 in all six forms at eight and one thread, and a deliberately weakened consumer that does not compare the full identity must fail exactly the two full-identity controls while the legitimate render stays green. The E/J composition is replayed after the J consumer change, because E was tested before the J extension. The remaining bar resumes: all other declared transports, the complete 151-plan generator with zero unknown edges, the 94/188 double generation, restoration and omission audits, the mechanically emitted 94-row schema with all 151 cells and the exact-byte digest, the ordered replay, `cargo fmt --check`, and both workspace modes at failed=0, filtered=0. The maintained edit surface and immutable set are unchanged; a consumer-boundary edit that cannot be expressed in the declared surface is a STOP. The stop-loss is the Q28 counter: one retired distinct compile error is progress; removing or weakening a control stops; a rejected copied production source or instrumentation edit is a finding that stops; two consecutive non-reducing attempts stop; the envelope is the hard bound. No new allocation, no fifth overlay, no subset, no placeholder digest, no hand-edited derived cell, no weakened control, no git, no network, no `unsafe`, no dependency or lockfile change. Budget: 5,304.579927531027 s focused remainder plus the untouched 6,000 s workspace; the 1,440 s verification envelope and its second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The director pauses after this gate for a host reboot and dispatches continuation 29 after it. No campaign, epoch, new catch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.
## Continuation 29 ruling (director) — the `Scope: NONE` transported-identity consumer inventory is accepted: 151 rows, 152 forms, 14 transports, 94 paths, 134 boundaries, 4,422 obligations and zero unknown consumers, with 56 hash-identical replays at 1,088 passed and the four expected J failures; the Q34 design and the continuation-30 draft are adopted (amended only in the postimage arithmetic), and continuation 30 implements the corrected consumers and resumes the bar under 5,173.269646620027 s

Continuation 29 is accepted as DONE. The inventory is gated on its own artifacts: 151 frozen rows and 152 forms (P2593 has both) across 14 declared transport families and 94 admitted paths; 134 consumer boundaries in 29 groups, each with a source/hash anchor, actual object, available independent record, current full-identity comparison, proposed record, named limitation and composition order, with no empty field; 4,422 form/boundary obligations with source edges, actual object, target record and named limit in every cell and zero unknown classifications; 142 helper functions re-derive from the four canonical helper sources (97 core, 25 raw, 16 paired, 4 loop); 15 explicit exclusions; 1,992 partial-helper instances with exactly two distinct non-canonical bodies (E's pre-J `installing` and `retire`, J's `paused`/`confirmed` cleanup) and no combined-execution claim; 989 rooted call sites over 49 rooted names; 316 source files parsed into 272,392 syntax nodes with the parser unchanged; all 94 admitted paths `not_emitted` with zero byte delta; the 235-file frozen J tree exact; the schema `f9604852…` unchanged with 151 cells and `new_schema=null`; 5,400 maintained, 590 design and 9,336 accepted files rehash clean, the only differences being the three director document drifts committed in `60c25ad4`. The 56 replayed existing-binary runs are hash-identical to their pins and re-derive at 1,088 passed, 4 failed, 0 ignored and 0 filtered: 24 E runs at 22/0, 24 corrected successor runs at 10/0, and the four failures are exactly P0267 armed/hash's two full-identity controls at eight and one thread (18/2 each); no other J row or form is inferred. The FLAGGED items are settled, not reds: the inherited J failures are the admitted finding, and the replay collector's env-form assumption plus the root census's missing derived empty `Permit` default are instrument repairs with both attempts retained, as is the later addition of the 27 accepted raw-source copies and the exact namespace/impl-qualified root matching, with no source or test contract weakened. The inventory names three open PortBlocked classes for implementation — `RawTransportFullIdentityAuthentication` over 23 boundaries, `JoinConfirmationFullIdentityAuthentication` over 8 and `HostConsumerFullIdentityNotImplemented` over 5 — while every other cell carries an executed receipt or its named limitation, and no cell is relabelled as executed.

The Q34 design is accepted: distinct immutable proof records with a private authenticated constructor, appended only by source-bound writers; transported values carry a proof id and identity; authentication requires complete identity equality plus the exact proof kind and source edge, the actual object or operation key, the current episode and side, and the lifetime or claim state; a wrong id with otherwise plausible fields rejects; consumers may read, claim or revoke but never replace a record; the writer/read pairs are fixed per record kind; construction preserves Q33's absent/invalid/barrier rules and Q32's claim comparison, and producer provenance keeps Q31.

Continuation 30 is ordered with the continuation-29 draft adopted as its brief, amended only in the postimage arithmetic: two fresh roots with exactly 94 targets each and exactly 188 postimages in total (one per target per root), never 188 per root. It corrects J in a separate render first while the frozen `join-queue-stop/` tree stays immutable, writing the independent install, pause and queue-pause records and comparing the whole received identity at both `join_result` and `omit_join`, checking the append's own generation and registered namespace, keeping the one arm, the one legitimate hit, queued zero and the existing quiescence predicate, and passing the twenty fixed controls for P0267/P0312/P0323 in all six forms at eight and one thread (36 runs, 720 control executions) with both corruptions at zero omissions, the eighteen originals exact, a deliberately weakened consumer red on exactly those two controls in each armed cfg and row, and the disabled and live records exact; corruption is extended to origin, episode, frame kind, generation, namespace and proof id with the independent fields fixed. It then composes E and J, reruns E's 22-control matrices (24 runs, 528 executions) and the interleaved-owner, same-generation-foreign, wrong-namespace, second-claim and retirement controls, keeping the E and J proof kinds distinct; applies the complete 134-boundary consumer table rather than only the reproduced join defect; re-executes every affected per-row matrix (all fifty typed rows, the twenty-five raw owner rows and every declared transport, including P0001/P0002/P1566/P1805) with hash-identity reuse only for unchanged evidence; composes the complete 151-plan deterministic generator with zero unknown edges, rejecting overlapping, duplicate or unknown anchors; generates the two roots, proves byte identity, restores registered instrumentation to whole-file bytes and AST in both cfg arms and proves the pinned omission separately; mechanically emits the six-column 94-row schema, hashes its exact bytes, binds all and only the 151 `source_schema` cells and independently compares every other inventory field; then runs the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the full unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. The fixed controls stand unchanged: P0239 armed at 1/1 arms, bindings 1, proofs 2, hits 1, body 0 and Some42; P0245 at 1/2 with the same other values; disabled at 1/2 with hits 0, body 1 and Some43; live at 0/0; the wrong-identity controls at hits 0, body 1 and Some43; P0396's twenty, P0278/P0281's fourteen, P0001/P0002's fifteen and P1566/P1805's seventeen. No expectation is edited. Budget: 5,173.269646620027 s focused plus the untouched 6,000 s workspace envelope; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The stop-loss is the Q28 counter: one retired distinct compile error is progress; a rejected copied production or registered instrumentation edit, a failing compiled candidate control, a removed or weakened control, an out-of-scope proof need or an exhausted envelope stops. No new catch, no new allocation, no fifth overlay, no subset, no placeholder schema, no provisional digest, no hand-edited cell, no git, no network, no `unsafe`, no dependency or lockfile change. No campaign, epoch, acceptance or CM5. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.
## Continuation 30 brief (adopted from the continuation-29 draft; director amendment: 94 targets per root and 188 postimages in total)

The director adopts this draft as the continuation-30 brief. It is read with the continuation-29 ruling above, whose fixed controls, budget, stop-loss and report rules are authoritative. The only amendment is the postimage arithmetic, corrected here.

# Draft continuation 30 — independent identity at every consumer

Director-review draft, not a dispatch or new allocation. Implements Q34 and
resumes the remaining CM4 bar. Incorporates the current brief's Q10–Q34 rules,
`Q34-DESIGN.md`, `consumers.json`, `consumer-instances.json`, `row-forms.json`,
`admitted-paths.json` and the accepted Q25 design. The director rewrites/adopts
this draft before implementation. No existing finding or expectation is changed.

## Goal

Every effect-conferring transported-token consumer authenticates complete
identity against an independent record written at the actual source event.
Deliver all 151 plans, zero unknown edges, ten classes/79 subtypes, the exact
94-path/188-postimage deterministic double generation, independent restoration
and omission audits, mechanically emitted schema and all 151 source_schema
cells, ordered standing replay and both workspace modes. No new catch or epoch.

## Context and inputs

- `docs/dev/codex/brief-742-g15cm4-generation-schema.md`, final Q34 ruling;
  AGENTS.md, TEMPLATE.md, testing policy and canonical organization guide.
- Immutable accepted Q25 inputs under `.temp/brief-747/continuation-1/`,
  authenticated against verified-final-a/manifest.json and identical root B.
- C29 `entry-hashes.json`, `source-authentication.json`, `consumers.tsv`,
  `consumer-instances.json`, `row-forms.json`, `source-calls.json`,
  `helper-variants.json`, `evidence-by-group.json` and `replays.json`.
- Frozen `.temp/brief-742/continuation-28/join-queue-stop/manifest.json`,
  235 exact files. The stop tree is never repaired. New render has a new path
  and hashes. The same applies to all earlier accepted/frozen evidence.
- C28 full 20-control J fixture and fixed successor/P0002 controls; C27 full
  fifty-row typed matrices; C19 raw retirement source and source controls.

## Scope

Production scope NONE. Maintained changes are restricted to generation/{mod,
plan,resolve,graph,transport,render,compose,audit,tests}.rs and recipes.tsv,
paths.tsv; mutations/mod.rs and schema.rs for generation binding;
source_auth/{mod,schema,tests}.rs; mechanical source_auth/overlays.tsv;
only source_schema in the 151 mutants.tsv rows; obligations/mod.rs only if
a crate-private discovery export is necessary. These are under
crates/rum-stdlib/tests/suspension_oracle/. Mechanical standing maintenance
may import only instrument-produced cells as already authorized.

Copied instrumentation may target only the exact 94 admitted paths. New record
types and private proof APIs live only in the generated cm740_probe helper.
Use existing source-bound hooks where possible; any added hook is a registered
edit with inverse erasure and a producer/return/construction anchor already
inside the admitted set. No fifth overlay or new production interface.

Immutable: live production, external projected resume files, source_worker,
runner/observer/predicate/control bodies, ordinary guard, exact omission,
selectors, phases, source_key/tail_digest, coverage/promotions, manifest, lock,
dependencies, and every frozen artifact. No git or network. No hand-edited
inventory, placeholder schema, subset or weak control.

## Target shape and staged implementation

1. Pin entry and all inherited artifacts without git. Import C29's inventory
   as declarations, not execution receipts. Resolve all source edges from the
   current preimages; unknown binding or unavailable proof is a named
   PortBlocked. Keep named limitations and do not relabel cells as executed.

2. Implement the `Identity`, `ProofRecord`, `Transported` and private
   `Authenticated` shape in Q34-DESIGN.md. Records have exact identity and
   actual object/edge keys. Issuance factories are restricted to begin/entry,
   actual pause, actual return and allocation/install hooks. Readers cannot
   replace records. Separate consumed/retired flags from immutable identity.
   An absent or unauthenticated value yields no permission. Do not create
   proof by copying the received value in a consumer. A ProofId alone is not
   sufficient. Preserve origin across owner rebasing; exact None frame kind
   is distinct from actual allocation/type. Maintain Weak liveness for raw
   owners without strong ownership or lifetime extension.

3. Correct J in a separate render first. At actual install/registration keep
   InstalledRecord's independent owner/source path and namespace/generation.
   At the existing actual producer pause, append PauseRecord with occurrence,
   then QueuePauseRecord joins that source record to InstalledRecord before
   the producer retires. `paused` and `confirmed` are transported copies.
   Both join_result and omit_join compare the entire received identity to
   these records and its authenticated producer/construction lineage. The
   actual append must match own generation and registered namespace. Do not
   require a legitimately dropped executing frame to stay live at queue
   publication; do not attribute a descendant frame to the request caller.
   Keep one arm, one own legitimate hit, queued zero and existing quiescence
   predicate. Preserve source order and all error/None/constructor/drop nodes.

4. Run corrected J's twenty controls for each P0267/P0312/P0323 in armed,
   disabled and live, each hash/nohash, at eight and one thread (36 runs,
   720 control executions). The actual owner publishes once; both transported
   source-path/occurrence corruptions publish zero; eighteen original controls
   remain exact. A deliberately weakened full-identity consumer must fail
   exactly those two controls in each armed cfg/row while the legitimate
   render passes. Keep disabled/live records exact. Extend corruption to
   origin/episode/frame-kind/generation/namespace/ProofId as additions, with
   independently recorded fields fixed; no original control is replaced.

5. Compose E and J in the same helper/source transform, then rerun E's full
   22-control matrices for P1470/P1471 in all six forms at eight/one (24 runs,
   528 controls). Keep E and J proof kinds distinct. Test interleaved owners,
   same-generation foreign queues, E scope during J setup and J cleanup
   during E scope, wrong namespace, second claim and independent retirement.
   E's prior pre-J receipts do not pay this gate. Assert full combined-source
   erasure and independent omission pins; no added publication or reordered
   push/return is permitted.

6. Apply the complete consumer table, not just the reproduced join defect.
   Authenticate resume before restoring provenance; retain Q32 claim checks.
   Preserve Q33 absent/invalid/barrier distinctions and actual successor proof.
   Authenticate operation bind and keep immutable operation records after
   claim so each call ticket, callee claim, walker step, omit and hit can check
   independently. Raw resume/grant/Flow/return-to-successor receive independent
   full construction identity; raw and paired origins are no longer discarded.
   Return/request/pair-issued and profile proofs use the exact writer/read
   pairs in the design. The host output and adapter paths get their declared
   source-bound return/pair/call proofs within the existing copied paths.
   Immediate-only P2593 and initial-only P0813 remain static failures.

7. Re-execute every affected per-row source-bound matrix after changes to its
   shared helper: all fifty typed rows, all twenty-five raw owner rows and
   every other declared transport, including P0001/P0002/P1566/P1805. Inherit
   unchanged evidence only by exact source/helper/control/binary hashes.
   Typed route proofs still show actual adapter abandonment before delegation
   and final-drop backstop; raw insertion still takes Q26/Q27. No raw route
   is invented for the typed census. If an actual route needs a fifth path,
   stop with its named class rather than reaching into production.

8. Compose one complete deterministic generator from live preimages and frozen
   declarations. Merge annotations on the same call before rewriting once;
   reject overlap/duplicate/unknown anchors. Only after all 151 plans and
   paths authenticate, generate two fresh roots with exactly 94 targets each
   and exactly 188 postimages in total (one per target per root). Prove byte identity. Erase registered instrumentation
   to whole-file bytes/AST in both cfg arms; separately prove the exact pinned
   omission and preserved nodes. Enumerate every emitted path and every byte
   delta versus current pins, including the four admitted raw files.

9. Mechanically emit the six-column 94-row schema. Hash its exact bytes, bind
   all and only 151 source_schema cells, and independently compare every other
   inventory field. No new digest before the complete bundle passes. Run the
   ordered standing maintenance before the first acceptance suite, then fmt
   and both workspace modes as specified below. Preserve all attempted runs.

## Fixed controls and calibration duty

Do not edit any existing asserted result. P0239 armed successor stays arms1/1,
bindings1/proofs2/hits1/body0/Some42; P0245 arms1/2 with the same other values.
Disabled keeps bindings1/proofs2/hits0/body1/Some43; live bindings0/proofs0.
Wrong occurrence and wrong activation stay hits0/body1/Some43. P0396's fixed
twenty controls, P0278/P0281's fourteen, P0001/P0002's fifteen and P1566/P1805's
seventeen remain. P0002 nested boundary carries exactly the outer prefix;
root cleanup is asserted after the outer call. Its four calibrated weaknesses
remain, as do actual abandonment, ordinary alias drop and nested producer arms.

For every consumer listed by C29, inject corruption only into the transported
copy after its writer, keeping the independent record unchanged. Challenge
each applicable identity component, actual object/type, source edge and proof
kind/ProofId; recheck after each handoff. Reject before writing a successor
proof or granting a capability. Keep valid, nested, two-independent-owner,
second-consumption, retired and disabled/live counterparts. Deliberate
self-authentication, empty-scope veto, ambient inheritance and blanket resume
suppression must be red on their named controls and valid variants green.
Record a named limitation where no source route exists, never a generic green.
An ordinary Rust assert does not prove a disabled complete-record equality.

## Mandatory standing replay and workspace gates

Use the exact DRAFT-COMMON.md commands and script mirror procedure; import
only instrument-produced cells. Ordered replay: fingerprint regeneration;
local_production_acceptance in default/no-default, certificate import and
revalidation in both; ownership discovery/exactness; rejection census; LP01
both modes; full typed-slot census both modes. Quote every verdict. A red
caused only by stale fingerprints/certificates/positions is an ordering
artifact: replay, do not hand-edit or weaken. Existing identity/status/
disposition change, UnknownHostPayload, changed LP01 statement or moved
rejection census is STOP.

Then bounded `cargo fmt --check` and serial offline/locked `cargo test -j8
--workspace --no-fail-fast`, with default and --no-default-features, using
RUST_TEST_THREADS=8 and exactly these seven --skip arguments:

```
lp03_adopted_phases_are_exact_in_completed_and_resumed_runs
lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs
lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs
lp658_host_eval_reset
lp658_host_resume_reset
lp655_root_context_falsifier
lp656_public_root_context_falsifier
```

Run the same lp03_phase_rows/lp04_definition_rows/lp05_assignment_rows binaries
for each mode with RUM_PROFILE_LEXICAL_SUBSTITUTION=1; verify their hashes before
and after. Clear all other profile/CM inherited environment as DRAFT-COMMON
requires. Full suspension_oracle and typed_slot_census must be unfiltered in
both modes, failed=0 and filtered=0. Eight observer, ten source-auth and four
Q7 predicate controls remain unchanged. No live hot-path change is authorized.

## Budget, concurrency, STOP and checkpoint

Focused budget is exactly C29 budget.json's implementation remainder after
return (starting parent remainder 5304.579927531027 seconds minus C29's actual
charge). The assigned workspace envelope is one cumulative 6000 seconds for
this continuation; bound each serial command by its remaining balance. The
1440-second verification envelope and second 6000-second workspace remain
held. No new allocation; CM4 committed28500 and 743–746 retain40500. One cargo
at a time, jobs8, tests8, inherited16G/no-swap scope, offline/locked. Every
long command/child tree uses run-bounded. Reproduce a genuine red on the same
binary at one thread. A scope OOM gets the standing one-job retry only.

Q28 stop-loss: one retired distinct compile error is progress for fixture-only
construction; two nonreducing attempts stop. Rejected copied production or
registered instrumentation, failing compiled candidate control, removed or
weakened control, out-of-scope proof need or exhausted envelope is STOP.
Keep failures and candidate hashes; do not repair a frozen finding after STOP.
Record every command, source change, replay, hash, file delta, done/next/tree
and remaining budget to continuation30 scratch and PROGRESS.md. A pass is the
complete bar, never a 138-row subset or provisional schema.

## Hard rules and report

No git/network/unsafe/new dependency/lock/manifest change. No campaign, mutant
epoch, acceptance packet, catch, settlement/SAME/promotion, encoding execution
or CM5. Pair(45,5,false) zero/Open; four structural controls and five
armed-without-hit Error cells per row unchanged; C2 118 Residual + 0 Lost;
51 owners/five duties Open; ordinary guard, CHANGELOG and S10 remain fixed.

Write rum-report/1 with all changed paths, exact old/new schema digests,
enumerated94 outputs/byte deltas and generator/audit/replay/workspace verdicts.
Validate via python3 -B scripts/report.py check 742 without --against-git.
Final response is only the output of python3 -B scripts/report.py summary 742.

## Continuation 30 ruling (director) — continuation 30 is a compliant Q28 STOP; the fixed J20 (720), expanded J54 (1,944), E22 (528), the twenty-four live/disabled composition baselines (504) and the weakened-consumer calibration (216 passed / 24 deliberate failures) are accepted, with `PortBlocked::JoinInterleavedOwnerTransportOverwritten` admitted as a defect of the generated instrument's transported-carrier cardinality; Q35 rules every transported carrier owner-keyed, and continuation 31 is the `Scope: NONE` transported-carrier cardinality inventory at 1,200 s, then continuation 32 implements Q35 and resumes the bar under 4,965.445755380028 s

Continuation 30 is gated as a compliant Q28 STOP. It ran `cdx` (unit `brief-742-cdx-20261117-144625-15435`, rc 0, peak 754,032,640 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 2,471,514 B, sha256 `c5db3855…`; report STOP, 1,407 changes — 1,405 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` paths — 19 PASS / 2 FAIL / 2 NA, 1,565 citations verified, 0 questions, 1 deviation; 207.82389123999883 s charged — 59.82389123999883 s measured over 176 commands plus 88.0 s bookkeeping and the 60 s reserve — of the focused remainder, 4,965.445755380028 s returned; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The STOP is the ordered one: the interleaved-owner control fired on its first armed reach, and the candidate, helpers, controls and copied sources were frozen immediately at `.temp/brief-742/continuation-30/interleaved-join-stop/manifest.json` (574 files, every hash exact; the director rehashed all 574), the continuation-28 tree (235 files) rehashes exactly, and no candidate, helper, control or copied source was repaired after the failure. The director re-derived the totals exactly: 134 test runs at 3,952 passed, 26 failed, 0 ignored and 0 filtered — 720 fixed J20 control executions (36 runs), 1,944 expanded J54 executions (36 runs), 528 E22 executions (24 runs), 504 live and disabled composition controls (24 runs), 216 passed and 24 deliberate failures in the weakened-consumer calibration (12 runs at 18/2), and 40 passed with the 2 interleaved failure instances (2 armed/hash P0267 runs) — with the 26 failures being exactly the 24 deliberate identity-calibration failures and those 2 interleaved instances. The two FAIL checks are accepted: `interleaved-owners` is the admitted finding, and `complete-generator` is not delivered because the stop preceded maintained integration, with no subset, placeholder digest, complete graph, all-consumer proof or weakened expectation substituted; the two NA checks (standing replay, workspace) follow from the stop and the untouched 6,000 s envelope. The deviation is accepted on the same grounds: the complete 134-consumer implementation, the 151-plan/94-target deterministic generator, the mechanical schema, the ordered replay and both workspace modes were not delivered because the mandatory stop fired, and nothing was claimed in their place. The entry is clean: 5,400 maintained, 590 design and 9,649 accepted files rehash unchanged; `production_changes=0` and `maintained_changes=0`; the `f9604852…` schema is unchanged with 151 cells and `new_schema=null`; all 94 admitted paths remain `not_emitted` with zero maintained byte delta; the three entry drifts are the director documents committed in `501dc09f` and the director verified all 23 pre-existing change paths against the entry hashes; no maintained source, inventory or expectation was edited, and the 2 modified `.temp` files are the scratch progress log and report checkpoint.

The finding is admitted as `PortBlocked::JoinInterleavedOwnerTransportOverwritten`, a defect of the generated instrument's transported-carrier cardinality, not a production regression: the transported confirmation is a single slot — `State { installed: Option<Installed>, paused: Option<PassProof>, confirmed: Option<PassProof>, join_owner: Option<&'static str>, … }` in the generated queue helper (`queue_helper.rs.in:19-22`, the same shape in `joined_queue_helper.rs.in`, and the defaults' `JoinState { installed, confirmed, owner }`) — set at `join_result`, cleared when the append's own scope is dropped, and taken by `omit_join`; a second independently live queue's confirmation replaces the first, consumes and clears its own copy, and the first queue's later append then publishes where it must omit. The control drives two actual selected resume methods through the unchanged producer, install, pause, complete, append and publication functions on two live Sessions with distinct actual queue roots at the same generation, with the second queue appending before the first; at eight and one thread P0267 armed/hash reproduces 20 passed / 1 failed with `arms=2`, `hits=1` against the required 2, the failure landing on the named assertion and the record diff showing the first owner's publication retained. The director reproduced the same-binary failure at both thread counts, and also established that P0312 and P0323 fail identically at both thread counts — the class is row-independent, though no other armed cfg is inferred because only armed/hash was built. Live and disabled forms pass 21/0 with complete record equality, the fixed J20 passes 20/0, the expanded J54 passes 54/0 with all seventeen transported-field challenges at both the confirmation and publication boundaries and the independent record snapshots held fixed, the weakened consumer is red on exactly the two original source-path and occurrence controls (18/2, its `J_CORRUPTION` records intact), and E22 passes 22/0 on the common helper. Production has no shared carrier: `ProfileQueue::append` publishes per queue from that queue's own `root`, `epoch` and `control` in the owners registry (`crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs:610`, `:654`, `:686`), and no `cm740_probe`, `queue_transport`, `PassProof`, `join_hits` or `confirmed` carrier exists anywhere under `crates/` (the one `confirmed` hit is an unrelated profiling test string), so the instrument models per-queue independence it does not itself provide. The defect is nevertheless in scope: the omission model is exactly what the schema counts, the c30 bar ordered interleaved owners, and a carrier that silently drops one live owner's value makes every zero-omission claim conditional on a single-owner sequence.

Q35 rules the carrier contract. Every transported carrier in the test-layer instrument that holds a value used to confer an effect is keyed by the complete identity of the owner it serves, so that two independently live owners cannot overwrite each other: a queue confirmation under its installed generation and registered namespace together with the owner activation; a pause record under its source-edge identity; an operation permit under its operation key. A single-valued carrier is admissible only where an invariant proves that at most one live owner can reach it, and that invariant is enforced at insertion — a second insertion is a failure, never a silent overwrite. A consumer takes only the record keyed by its own actual scope and then authenticates it under Q34; the transported copy is never evidence of its own identity or of who owns it. Every carrier that today can silently drop a live owner's value is corrected, not declared out of scope; a carrier the mechanism cannot key is a named PortBlocked. The interleaved-owner control requires every independently live installed queue to retain exactly its own publication omission in every form at both thread counts, with disabled and live parity and exact records. Q34 and Q29–Q33 stand unchanged.

Because the class is the instrument's carriers rather than one consumer, continuation 31 is a `Scope: NONE` transported-carrier cardinality inventory at 1,200 s of the focused remainder, with unused time returned and no workspace command; the c30 binaries and every earlier frozen artifact are reused by hash identity and nothing is rebuilt or repaired. Over every instrument state in the CM4 test layer — the queue helpers, the evaluation-queue scaffold, the identity records, the queue storage registries and every defaults helper, in template and in each rendered candidate and live copy — it enumerates every transported carrier with its declaration site and type, every writer and reader at `path:line` in the copied source and the generated helper, the identity available at each write, the set of independently live owners that can actually reach it (with an interleaving witness built from the copied production call graph and the campaign's rows and transports), today's second-insertion behavior, the Q35 keyed form or the enforced single-owner invariant, an executed falsifier or a named new control for continuation 32, and its composition order with the other carriers, with zero unknown carriers; a carrier the mechanism cannot key is a named PortBlocked. It re-derives the finding from the frozen artifacts and confirms the production counterpart, and it designs the keyed carrier for each class with its insertion, take and retirement paths and the exact keys, preserving Q29–Q34. It names the new controls continuation 32 must execute — a second live owner's insertion without overwrite, interleaved pause and confirmation, per-scope take, second claim and retirement — each calibrated red against a deliberately weakened last-writer-wins variant, and drafts continuation 32's implementation brief. The inventory edits no maintained source, runs no workspace command and performs no candidate repair.

Continuation 32 then implements Q35 and resumes the remaining bar: every reachable carrier keyed or single-owner-enforced, every fixed control byte-identical, the interleaved matrix green for P0267/P0312/P0323 in all six forms at eight and one thread with each owner retaining its own omission, the new interleaved-carrier controls green with the weakened variant red on exactly the named controls, and then the c30 bar resumed in full — the complete 134-boundary consumer table, the E/J composition, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges, the two roots with exactly 94 targets each and exactly 188 postimages in total, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips and the unfiltered suites at failed=0 and filtered=0. The maintained edit surface and immutable set are unchanged; the frozen `join-queue-stop/` (235) and `interleaved-join-stop/` (574) trees are immutable; a carrier that cannot be keyed within the declared surface is a STOP. The stop-loss is the Q28 counter: one retired distinct compile error is progress; a rejected copied production or registered instrumentation edit, a failing compiled candidate control, a removed or weakened control, an out-of-scope proof need or an exhausted envelope stops. No new catch, no new allocation in maintained source, no fifth overlay, no subset, no placeholder schema, no provisional digest, no hand-edited cell, no git, no network, no `unsafe`, no dependency or lockfile change. Budget: 1,200 s for the inventory from the 4,965.445755380028 s focused remainder, unused returned, then the implementation from the returned remainder plus the untouched 6,000 s workspace; the 1,440 s verification envelope and its second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 31 ruling (director) — the `Scope: NONE` transported-carrier cardinality inventory is accepted (133 carriers in 26 groups, zero unknown, 26 named calibration controls, 151 rows / 152 forms / 14 transports / 94 paths / 134 boundaries preserved, 54 hash-identical replays at 1,152 passed with exactly the 6 inherited interleaved instances), and Q35's owner-keyed design plus the continuation-32 draft are adopted (amended only in the budget remainder: 4,772.014322774031 s); continuation 32 implements Q35 across every reachable carrier and resumes the full remaining bar

Continuation 31 is accepted as DONE. It ran `cdx` (unit `brief-742-cdx-20261117-153251-32519`, rc 0, peak 871,587,840 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 3,122,144 B, sha256 `0cf14e1a…`; report DONE, 218 changes — 216 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` paths — 14 PASS / 2 FLAGGED / 3 NA, 950 citations verified, 0 deviations, 0 questions; 193.43143260599663 s charged — 93.93143260599663 s measured over 79 commands plus 39.5 s bookkeeping and the 60 s reserve — of the 1,200 s inventory envelope, 1,006.5685673940034 s returned, leaving 4,772.014322774031 s focused; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The inventory is gated on its own artifacts: 133 transported carriers in 26 groups with zero unclassified carriers, each with its declaration site and type, the identity available at write, the independently live owners that can reach it, today's second-insertion behavior, the Q35 insertion/take/retirement rule, a named control and calibration, a composition order and its limitation — every field non-empty and independently re-derived; 12,679 declaration instances over 11,777 scanned sources, 719 source instances with 327 unique bodies, 177,273 member references with exact source/function anchors, and 10,570 explicit exclusions in eight classified reasons that each name why the field is not an effect-conferring carrier (5699 immutable identity/payload fields, 1857 counters, 1692 per-owner flags, 549 enclosing registries, 439 enclosing aggregates, 270 read-once configuration fields, 58 helper payloads and 6 fixture observation seams); 26 named controls, one per group, in `controls.json`; the 151 rows, 152 forms, 14 transports, 94 admitted paths, 134 consumer boundaries and 4,422 obligations are preserved and joined per consumer in `consumer-carrier-join.json` (134 rows, each naming its consumer, carriers, Q34 and Q35 duty); the `f9604852…` schema is unchanged with `new_schema=null`, no generator was emitted and no candidate was built. The 54 frozen replays are hash-identical to their pins — all 11 binary and source bindings rehash exactly, zero rebuilds — and re-derive at 1,152 passed / 6 failed / 0 ignored / 0 filtered, the six failures being exactly the P0267/P0312/P0323 armed/hash interleaved instances at eight and one thread; the 24 live/disabled composition runs pass 504 controls with complete record equality, and E's 24 runs pass 528. Both frozen trees rehash exactly (continuation-28 235 files and continuation-30 574 files). The entry is clean: 5,400 maintained, 590 design and 11,053 accepted files rehash unchanged, `production_changes=0` and `maintained_changes=0`, all 94 admitted paths `not_emitted`, all 23 pre-existing change paths match their entry hashes, and the three entry drifts are the director documents committed in `8f1e1698`. The two FLAGGED items are settled, not reds: the inherited J failures are the admitted `PortBlocked::JoinInterleavedOwnerTransportOverwritten` being reproduced, not a new catch or a production regression, and the replay collector's four-count/three-count tuple-arity bug is a diagnostic-instrument correction with both attempts retained and no candidate, control or source repaired; the three NA checks (generator, standing replay, workspace) follow from the `Scope: NONE` order.

Q35's design is accepted. Keys are private nominal ids inside the generated helper: an Episode with row, program, pair and actual side; an Activation with the source-scoped runtime path; a SourceKey with episode, origin, owner, producer occurrence, exact source edge and proof kind; an InstalledKey adding the actual generation, root/epoch/control namespace and an install nonce issued at the actual install; a ConstructionKey adding the actual allocation/type and its independent construction id; an OperationId/CallId/StepId with the actual operator, target, SharedArgs allocation/length, callee edge or walker ordinal; and a SegmentKey with epoch, index, segment id, actual completion object, thread, step and episode. The owner is never selected from the transported copy: the actual producer, construction, queue or call supplies the lookup ticket, the append resolves its key from the actual generation and registered namespace through a reverse index, zero or multiple matches fail closed, and generation or row alone is never a key; if a concrete insertion route cannot supply these existing source facts, `PortBlocked::CarrierOwnerKeyUnavailable` is returned with its path. Insertion uses a vacant entry under the full key; an occupied live key rejects before mutation; replacement occurs only after the independently observed retirement; different keys coexist and taking one never clears another; one-use consumption and retirement stay separate from the immutable Q34 records, and an invalid present transport rejects without selecting which other key to retire or causing inheritance. The concrete corrections are named and accepted: the queue installation becomes a map with an exact source-install scope stack and an independent reverse index, `paused` becomes a map under the source key, the join scope records EntryId/InstallId rather than a row string, confirmation and append consume only their own key, and wrong namespace, foreign same-generation append, swallowed result and unrelated cleanup cannot clear another owner; E's global `proof` becomes scope-keyed with the independent E PassRecord as authority; the immutable proof maps' secondary indexes get the same cardinality discipline (`pause_at` must not invalidate an older occurrence, `entry_at` gets an enforced unique-entry invariant, queue lookup includes source occurrence and kind, and queued lookup cannot choose the last entry as authority); the default ledger's pending/return/request maps carry exact source tickets instead of Activation-only indexes; typed/Box/operation registries use the actual allocation/type with construction/operation ids and explicit duplicate rejection while Q32/Q33 and Q31 stand; `raw::State.ready` is the only permitted single-valued carrier and only with its proven omit→hit edge and second-insertion rejection, otherwise a keyed local receipt; the operation and paired permits enforce empty-receipt insertion so a repeated `omit` cannot erase an unreported hit; `ProfileState.current` becomes Segment-keyed with a finish scope stack; host output uses a HostCallId with an independent return/pair issue record rather than a row-only Cell; and the legacy `JoinState` — still called by `arm` at `defaults/helper.rs.in:221` — is either migrated on every reachable root or removed with its calls as registered instrumentation with erasure, never declared harmless. I verified the underlying code facts myself: `InstallGuard::drop` silently assigns `state.installed` (`queue_helper.rs.in:61-74`, assignment at `:68`), `OperationPermit::omit` assigns `self.hit` even when `take()` yields nothing (`defaults/operation_helper.rs.in:101-105`, assignment at `:103`), `join_paused` is reached from `arm`, and the legacy `JoinState` holds the single `confirmed`/`installed`/`owner` slots.

Continuation 32 is ordered with the continuation-31 draft adopted as its brief, amended only in the budget arithmetic: the focused remainder after the return is exactly 4,772.014322774031 s. It implements Q35 across every reachable carrier in a fresh root, preserving Q34 authentication at every boundary and every earlier rule; every map insert rejects an occupied live key before mutation; every single-valued carrier needs a proved and enforced insertion invariant; independent records are never rebuilt from a received copy, and secondary indexes preserve every live occurrence. It corrects J's installed, pause and confirmation stores together (the actual scope chooses the key, append resolves exactly one installed owner from the actual generation and registered namespace, JoinOwner carries a source scope id, and equal generation under another root cannot consume or clear it), migrates or removes every reachable legacy `JoinState` root with registered erasure, then implements every carrier group rather than only J, including the E scope-keyed proof, the default ledger's source tickets, the typed/raw/operation/profile/host carriers, the permitted raw singleton, and the permit empty-receipt rule, with `PortBlocked::CarrierOwnerKeyUnavailable` for any route that cannot supply its key. It preserves all J20 and J54 controls byte-identically and executes P0267/P0312/P0323 in all six forms (armed, disabled, live × hash, nohash) at eight and one thread — 36 J20 runs / 720 executions, 36 J54 runs / 1,944 executions, and the 21-control interleaved matrix at 36 runs / 756 executions with `arms=2` and each live owner retaining its one omission in armed, exact disabled/live records, and the deliberately weakened Q34 consumer still red on exactly its two identity controls; the C30 armed/hash red is fixed by carrier cardinality, never by changing an expected count. It composes E and J before paying E's gate (24 runs / 528 controls plus E-during-J, J-during-E, nested E, foreign namespace, two-owner, second-claim and independent-retirement controls, preserving publication/push/return order and distinct proof kinds), executes every named C31 control with its valid, nested, two-owner, wrong-key, second-insertion, per-scope take, second-claim and retired counterparts and a separate last-writer-wins variant per class that must fail exactly its named controls, applies the full 134-boundary table and 4,422 form/boundary obligations, replays every affected row matrix (all fifty typed rows, the twenty-five raw owner rows, all other transports including P0001/P0002/P1566/P1805) with hash-identity reuse only for unchanged evidence, composes the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, generates two fresh roots with exactly 94 targets each and exactly 188 postimages in total with byte identity, restores instrumentation to whole-file bytes and AST in both cfg arms and proves the pinned omission, mechanically emits the six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, then runs the ordered standing replay, `cargo fmt --check` and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the full unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. The fixed controls stand unchanged: P0239 armed 1/1 arms, bindings 1, proofs 2, hits 1, body 0, Some42; P0245 1/2 with the same other values; disabled 1/2 hits 0 body 1 Some43; live 0/0; the wrong-identity controls hits 0 body 1 Some43; P0396's twenty, P0278/P0281's fourteen, P0001/P0002's fifteen, P1566/P1805's seventeen; the eight observer, ten source-auth and four Q7 controls; the two frozen trees (235 and 574) remain read-only. The maintained surface is unchanged — the generation modules and data, the module/schema bindings, the source_auth files, the mechanically emitted `overlays.tsv`, only the 151 `source_schema` cells and the minimal discovery re-export — and production scope stays NONE. The stop-loss is the Q28 counter: one retired distinct compile error is progress; a rejected copied production or registered instrumentation edit, a failing compiled candidate control, a removed or weakened control, an unavailable owner key, an out-of-scope proof need or an exhausted envelope stops; a genuine red is reproduced on the same binary at one thread and frozen without repair. No new catch, no new allocation in production, no fifth overlay, no subset, no placeholder schema, no provisional digest, no hand-edited cell, no git, no network, no `unsafe`, no dependency or lockfile change. Budget: 4,772.014322774031 s focused plus the untouched 6,000 s workspace (one cumulative balance for both modes and the switched checks); the 1,440 s verification envelope and its second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 32 ruling (director) — continuation 32 is a compliant Q28 STOP: the Q35 carrier implementation passes J20 (720 executions), J54 (1,944), the interleaved J21 matrix (756, `arms=2` with each live owner keeping its own omission), the weakened-consumer calibration (18/2 on exactly the two identity controls) and E22 (528), and stops on the first compiled added nested-E control, whose inner preparation takes the P1471 witness route while P1470 is selected; the finding is admitted as a fixture-contract error (`PortBlocked::NestedEvaluationFixtureRouteMismatch`), Q35 stands unchanged, and continuation 33 corrects the fixture route with an explicit route assertion and resumes the full remaining bar under 4,540.691416336026 s

Continuation 32 is gated as a compliant Q28 STOP. It ran `cdx` (unit `brief-742-cdx-20261117-160815-41708`, rc 0, peak 728,399,872 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 1,888,499 B, sha256 `40755793…`; report STOP, 2,558 changes — 2,556 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` change paths — 20 PASS / 2 FLAGGED / 2 FAIL / 2 NA, 1,806 citations verified, 0 questions, 1 deviation; 231.3229064380048 s charged — 71.8229064380048 s measured over 199 commands plus 99.5 s bookkeeping and the 60 s reserve — of the 4,772.014322774031 s focused remainder, leaving exactly 4,540.691416336026 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held).

The green evidence is re-derived from the logs and accepted: J20 ran 36 times at 20/0 (720 executions), J54 36 times at 54/0 (1,944) with all seventeen transported fields at both boundaries, J21 36 times at 21/0 (756) — P0267/P0312/P0323 in armed/disabled/live × hash/nohash at eight and one thread, each interleaved control reporting `arms=2`, `hits=2`, `required_hits=2`, so the continuation-30 overwrite red is fixed by carrier cardinality and never by an adjusted expectation — E22 24 times at 22/0 (528) on the combined helper, and the deliberately weakened Q34 consumer 12 times at 18/2 with the two failures exactly `wrong_transported_source_path_rejects` and `wrong_transported_producer_occurrence_rejects`. The control-name lists are identical to continuation 30's (54/21/20 plus the result line), the fixed 20/54/21 bodies and expectations match continuation 30, and every E22 non-probe byte is continuation-30-exact; 304 J function instances across four families and two forms keep their signatures, bodies and omission sites. The totals re-derive exactly: 146 runs, 4,208 passed, 30 failed = the 24 deliberate calibration failures plus the 6 nested-control instances, 0 ignored, 0 filtered. The entry is clean: 5,400 maintained, 590 design and 11,268 accepted files unchanged, `production_changes=0` and `maintained_changes=0`, the `f9604852…` schema unchanged with all 151 cells and `new_source_schema=null`, all 94 admitted paths `not_emitted` with zero maintained byte delta, the 23 pre-existing change paths matching their entry hashes (32 file instances after expanding the untracked `source_auth/` directory), and the three entry drifts being the director documents committed in `e8427550`. Both older frozen trees rehash exactly (continuation-28 235 files, continuation-30 574).

The STOP is admitted, not a red: the three new nested-E controls fail at 22 passed / 3 failed on P1470 armed/hash at eight and one thread, at the inner assertion `candidate.rs:2380` (`left: 0`, `right: 1`), with two actual selected producer arms and no P1470 hit. I reproduced it at one thread on the frozen binary and then ran the same binary under P1471: 25 passed, 0 failed. The source route explains both results: `Session::new` leaves `nested_initial=false` (`candidate.rs:34`), the fixed E22 scene sets it for the selected row (`candidate.rs:2274`), and the new inner preparation at `candidate.rs:2358` does not; the initial witness helper carries `evaluation_owner("P1470")` (`candidate.rs:553`) while the resumed helper carries `evaluation_owner("P1471")` (`candidate.rs:574`), and `producer_owner` requires the selected row (`candidate.rs:794`), so the mis-routed inner owner's E proof is rejected before it can omit. The retained pins bind P1470 to the initial route and P1471 to the resumed route. The finding is therefore a fixture-construction error of the new control, not a carrier, mechanism or production defect: no nested-E disabled/live, P1471 or nohash result, and no outer completion, is inferred or claimed.

The candidate was frozen immediately: `.temp/brief-742/continuation-32/nested-e-stop/manifest.json` preserves 826 files from the first failure and `nested-e-stop-closed/manifest.json` separately seals the same 826 paths after the in-progress `freeze-stop.log` completed; I rehashed both trees (all frozen copies exact, the closed tree's originals exact, the first tree's only original drift being that still-open command log) and the stopped candidate's sha256 is `1e3a59ee…` exactly as the finding anchors record. No candidate, helper, control or copied source was repaired. The two FLAGGED items are settled as metadata ordering and route diagnosis, not reds; the one deviation (the remaining bar undelivered because the stop preceded maintained integration) and the complete-generator FAIL are accepted as the stop's consequence, with no subset, placeholder digest or weakened expectation substituted.

Q35 and Q34 stand unchanged. Continuation 33 is ordered: in a fresh render, correct the new inner preparation to select the same per-row route the fixed scene uses — the initial route for P1470 (the `nested_initial` switch the fixed scene sets) and the explicitly resumed route for P1471 — and assert the chosen route explicitly before the omission expectation, so a route mismatch fails at the route assertion rather than at the hit count; calibrate that assertion with the mis-routed variant (the current continuation-32 preparation) failing it while the corrected control stays green. Keep the existing expectation formulas (inner one in armed and zero in disabled/live, total two in armed and zero in disabled/live), all 22 base expectations, the source bodies and the omission pins unchanged. Rerun the nested/composition controls in every form and row at eight and one thread (2 rows × 6 forms × 2 threads = 24 runs) with the E-during-J, J-during-E, foreign-namespace, two-owner, second-claim and independent-retirement controls, and then resume the remaining bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. Budget: 4,540.691416336026 s focused plus the untouched 6,000 s workspace; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574) and both `nested-e-stop` trees (826 each) are immutable; the maintained surface and production scope stay NONE; the stop-loss is the Q28 counter unchanged. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 33 ruling (director) — continuation 33 is a compliant Q28 STOP: the ordered route correction is accepted (both prepared sessions take the per-row witness route, the explicit route assertion passes, the inner hit is one and the total two in armed), and the stop then exposes a second control-authoring error (`PortBlocked::NestedEvaluationPublicationCount`: required zero publications where the accepted fixed-scene baseline of the same form has exactly one, because the selected witness omits and the non-selected resumed witness publishes); Q36 rules that every composition control's expectations are derived from the accepted fixed-scene baseline and its nested window must reproduce that scene's own event shape, and continuation 34 corrects the two publication assertions and resumes the full remaining bar under 4,387.87299200803 s

Continuation 33 is gated as a compliant Q28 STOP. It ran `cdx` (unit `brief-742-cdx-20261117-165539-57532`, rc 0, peak 466,604,032 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 2,041,436 B, sha256 `97089540…`; report STOP, 953 changes — 951 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` change paths — 16 PASS / 3 FLAGGED / 2 FAIL / 3 NA, 213 citations verified, 0 questions, 1 deviation; 152.81842432799567 s charged — 83.81842432799567 s measured over 18 commands plus 9.0 s bookkeeping and the 60 s reserve — of the 4,540.691416336026 s focused remainder, leaving exactly 4,387.87299200803 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held).

The ordered correction is accepted on its own evidence: the new inner preparation now selects the same per-row route as the fixed scene — the initial route for P1470 (`candidate.rs:2364-2365`) and the explicitly resumed route for P1471 — and the explicit route assertion (`candidate.rs:2371`) passes before the omission expectation; the inner hit is one and the total two in armed, as ordered, and the outer completion is still not reached only because the next assertion fails. The route-calibration NA (not executed because the corrected candidate reached the next failing assertion first) is accepted and moves to continuation 34. The entry is clean: 5,400 maintained, 590 design and 13,823 accepted files unchanged, `production_changes=0` and `maintained_changes=0`, the `f9604852…` schema unchanged with all 151 cells and `new_source_schema=null`, all 94 admitted paths `not_emitted` with zero maintained byte delta, the 23 pre-existing change paths matching their entry hashes (32 file instances after expanding the untracked `source_auth/` directory), and the three entry drifts being the director documents committed in `46e429dd`; the earlier frozen trees rehash exactly (continuation-28 235, continuation-30 574, both continuation-32 trees 826) and the new candidate is frozen (292 files from the first failure, and a 292-file closed snapshot that seals the in-progress log; I rehashed both trees, all frozen copies exact, the closed tree's originals exact, the first tree's only original drift being its still-open `freeze-stop.log`), with the stopped candidate's sha256 `c436e3b2…` exactly as the finding anchors record and no candidate, helper, control or copied source repaired.

The STOP is admitted: the three nested controls fail at 22 passed / 3 failed on P1470 armed/hash at eight and one thread, at the unchanged inner-publication assertion `candidate.rs:2394` (`left: 1`, `right: 0`), after the route assertion, the inner hit and the total hits pass. The required zero is a control-authoring error, not a mechanism defect, and the accepted fixed-scene baseline decides it: the same binary in the same run prints, for the fixed E22 scene under P1470 armed, `hits=1` with the twelve-event window `["guest-return", "cell-return", "return-inserted", "guest-typed", "cell-consumer", "guest-return", "cell-return", "return-inserted", "boundary-taken", "cell-boundary", "guest-typed", "cell-consumer"]` — two return insertions with exactly one publication, the first insertion omitted and the second, the non-selected resumed witness, published — and under disabled and live, `hits=0` with two publications. The nested inner window reproduces that accepted sequence exactly, so the correct expectation is the fixed scene's own window: one publication in armed and two in disabled/live, with the selected witness's insertion followed by no publication and the non-selected resumed witness's insertion by exactly one. The two FLAGGED items are settled (the publication diagnosis, which the fixed-scene baseline upgrades from inference to established source fact, and the snapshot-closure metadata ordering) and the third (a read-only diagnostic's 30 s bound with the retained 34.525 s retry) is accepted; the one deviation (the remaining bar undelivered because the stop preceded maintained integration) and the complete-generator FAIL are accepted as the stop's consequence, with no subset, placeholder digest or weakened expectation substituted.

Q36 rules the control-construction contract: every nested or composition control's expected values — route selection, hit counts, publication counts and event shape — are derived from the accepted fixed-scene control of the same form, executed on the same helper and recorded, and the nested window must reproduce that scene's own event shape for that form, with the selected owner's omission replacing exactly its own publication and every non-selected owner behaving as the fixed scene does; an expectation that departs from the fixed scene is evidence about the mechanism and must be reproduced and frozen as a finding, never written as a new expectation, and a control whose expectation cannot be derived from a fixed-scene baseline is a named PortBlocked. Q35 and Q34 stand unchanged. Continuation 34 is ordered: correct the two publication assertions to the fixed-scene-derived semantics — per window exactly `1 + usize::from(armed()==0)` publications, the selected witness's insertion followed by none and the non-selected resumed witness's insertion by exactly one, and the window's event sequence equal to the fixed scene's own recorded sequence for that form — keeping the route assertion, the hit expectations, all 22 base expectations, the source bodies and the omission pins; calibrate the corrected assertion with a deliberately weakened variant that either publishes the selected witness or suppresses the non-selected one, failing the assertion while the corrected control stays green; then rerun the nested/composition controls in every form and row at eight and one thread (2 rows × 6 forms × 2 threads = 24 runs) with the E-during-J, J-during-E, foreign-namespace, two-owner, second-claim and independent-retirement controls, and resume the remaining bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. Budget: 4,387.87299200803 s focused plus the untouched 6,000 s workspace; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 `nested-e-stop` trees (826 each) and both continuation-33 trees (292 each) are immutable; the maintained surface and production scope stay NONE; the stop-loss is the Q28 counter unchanged. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 34 ruling (director) — continuation 34 is a compliant Q36 STOP and the conflict is a director order error: the publication formula I ordered in continuation 33 was derived from P1470's two-insertion window and contradicts P1471's accepted fixed scene (one insertion, zero armed and one disabled/live publication), so the worker correctly refused to encode an expectation Q36 forbids; Q36 stands unchanged, the per-row and per-form fixed-scene baseline is the contract, and continuation 35 derives the nested expectations from each row's own baseline, calibrates them, and resumes the full remaining bar under 4,276.316098010036 s

Continuation 34 is gated as a compliant Q36 STOP. It ran `cdx` (unit `brief-742-cdx-20261117-171513-62305`, rc 0, peak 487,641,088 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 1,978,705 B, sha256 `333bcf86…`; report STOP, 763 changes — 761 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` change paths — 10 PASS / 2 FAIL / 3 NA, 1,146 citations verified, 0 questions, 1 deviation; 111.55689399799485 s charged — 29.81455579399517 s measured over 41 commands plus 20.5 s bookkeeping, the 60 s reserve and 1.2423382039996795 s closing — of the 4,387.87299200803 s focused remainder, leaving exactly 4,276.316098010036 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The unchanged E22 fixed scene was re-executed fresh in all six forms for both rows at eight and one thread: 24 runs, 528 passed, 0 failed, 0 ignored, 0 filtered, with the fixture binaries unchanged between the two thread counts; the entry is clean (5,400 maintained, 590 design, 14,773 accepted, the 90/180 pins and 151 rows exact, `production_changes=0` and `maintained_changes=0`, `f9604852…` unchanged before and after, all 94 admitted paths `not_emitted`, the 23 pre-existing change paths matching their entry hashes, and the three entry drifts being the director documents committed in `3daeb8c4`); the earlier frozen trees rehash exactly (235, 574, 826 ×2, 292 ×2) and the new 344-file snapshot is exact in both its frozen copies and its originals.

The STOP is admitted and the error is mine, not the worker's: the continuation-33 order prescribed `1 + usize::from(armed()==0)` publications per nested window and required each window's event sequence to equal the same-form accepted fixed scene, with a control whose expectation cannot be derived from that scene being a named PortBlocked. The fresh fixed-scene evidence shows the prescription holds only for P1470: that row's window has two return insertions — the selected initial return (omitted in armed) and the resumed return (published) — giving one armed and two disabled/live publications, and the twelve-event window recorded before. P1471's window has exactly one insertion (the explicitly resumed route), `["guest-return", "cell-return", "return-inserted", "guest-typed", "cell-consumer"]`, with **zero** armed and one disabled/live publication. The prescribed 1/2 therefore conflicts with P1471's accepted baseline, and applying it would have created exactly the kind of unfounded expectation Q36 forbids. The worker stopped, retained the failed baseline contract, bound every observation to its log, source and binary hash, and changed no expectation, fixed scene or assertion; the two FAIL checks (the contract conflict and the undelivered complete generator) and the three NA checks (nested controls and calibrations, standing replay, workspace) are accepted as the stop's consequence, with no subset, substitute expectation or placeholder digest, and the one deviation is accepted on the same grounds.

Q36 stands unchanged, and its per-row, per-form reading is confirmed: the accepted fixed scene of **each row and form** is the expectation, not a formula fitted to one row. Continuation 35 is ordered: correct the nested publication assertions to the row-wise baseline — P1470's window as two insertions with one armed and two disabled/live publications, P1471's as one insertion with zero armed and one disabled/live publication — and assert each window's return/publication subsequence (`return-inserted`, `boundary-taken`, `cell-boundary`) equals that row's own recorded fixed-scene subsequence for the same form, with any additional composition event named and source-anchored rather than absorbed into the count; keep the explicit route assertion, the hit expectations, all 22 base expectations, the source bodies and the omission pins. Calibrate with row-wise weakened variants — publishing the selected witness, or suppressing a non-selected one — that fail the corrected assertion while the corrected controls stay green. Then rerun the nested/composition controls in every form and row at eight and one thread (2 rows × 6 forms × 2 threads = 24 runs) with the E-during-J, J-during-E, foreign-namespace, two-owner, second-claim and independent-retirement controls, and resume the remaining bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. Budget: 4,276.316098010036 s focused plus the untouched 6,000 s workspace; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each) and the continuation-34 `baseline-contract-stop/` (344) are immutable; the maintained surface and production scope stay NONE; the stop-loss is the Q28/Q36 counter unchanged. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 35 ruling (director) — continuation 35 is a compliant Q28 STOP: the ordered Q36 correction is accepted and closed (24 fresh E22 runs at 528/0, the nested candidate is the E22 candidate plus 81 additive lines with the per-row prepared route, the recorded per-row/per-form baseline re-executed on the same helper before composition, and inner/outer windows equal to their own baselines; 24 nested runs / 600 controls green, 16 calibration runs / 48 rejections on exactly the named assertions, J replay 3,420, E22 528), and `PortBlocked::IndependentEpisodeBeginContractConflict` is admitted as the worker's generated-instrument implementation error — the new `SecondLiveRun` guard at begin rejects Q18's required replacement of a stale unpaired episode, so the unchanged 170-control suite fails 168/2 at 8 and 1 thread on the same binary; Q18 and Q35 both stand, and continuation 36 repairs the guard at the insertion sites, calibrates every retained assertion, reruns the 170 controls in both cfgs and resumes the full bar under 3,998.3284827930283 s

Continuation 35 is gated as a compliant Q28 STOP. It ran `cdx` (unit `brief-742-cdx-20261117-173552-68071`, rc 0, peak 719,425,536 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 2,581,236 B, sha256 `dd34f7bb…`; report STOP, 1,746 changes — 1,744 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` change paths — 14 PASS / 2 FAIL / 2 NA, 8,765 citations verified, 0 questions, 1 deviation, 0 disclosures; 277.98761521700726 s charged — 91.99334730500777 s measured over 249 commands plus 124.5 s bookkeeping, the 60 s reserve and 1.4942679119994864 s closing — of the 4,276.316098010036 s focused remainder, leaving exactly 3,998.3284827930283 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The report's own accounting re-derives: 202 runs at 5,620 passed / 52 failed / 0 ignored / 0 filtered, the 52 being exactly the 48 planned nested-calibration rejections and the 4 candidate episode-guard instances (2 controls at 8 and 1 thread on the hash binary).

The Q36 correction is accepted and closed. The fresh E22 authority is 24 runs at 22/0 (528), with P1470 at two return insertions and one armed / two disabled-live publications and P1471 at one insertion and zero armed / one disabled-live. The nested candidate is the frozen E22 candidate (`51403e1d…`) plus exactly 81 added lines and zero removed: `q35_prepared_e_owner` selects the per-row witness route with the explicit `Q33_NESTED_ROUTE` assertion, `q36_recorded_baseline` records the six row/form shapes from the accepted fixed-scene logs, `q36_fixed_scene_baseline` re-executes the fixed scene on the same helper and asserts the recorded table before any composition, and `q36_assert_window` compares each inner and outer `return-inserted`/`boundary-taken`/`cell-boundary` subsequence with that row's own baseline, with the two `q35-inner-begin`/`q35-inner-end` fixture callback delimiters named and source-anchored. All 24 runs (two rows, six forms, eight and one thread) pass 25 controls (600). The calibration is exact: 16 full-suite runs at 22/3, the misrouted-initial variant (4 runs) failing `Q33_NESTED_ROUTE`, and the publish-selected (8 runs) and suppress-nonselected (4 runs) variants failing `Q36_WINDOW` — each rejecting exactly its three named nested controls while the 22 base controls stay green (352). The J replay is 108 runs at 3,420 (36 × 20, 36 × 21, 36 × 54) after exact source/helper/control/binary authentication, and E22 re-runs 24 at 528.

The finding is admitted and the responsibility is the worker's: the typed-auth render added ten registered helper edits, and the first, the `SecondLiveRun` guard at `typed_auth/helper.rs.in:71` (rendered at `inherited.rs:84` and `:1848`), is too strong — an existing `Run` is not proof of a live producer or scope. The immutable `stale_unpaired_episode_clears_on_new_begin` control (continuation 14) closes its producer guard, then calls a new unqueued begin without end, and Q18 explicitly replaces that unpaired episode (brief-742:927: "a further unqueued `begin` with that identity opens a new episode and clears any unpaired one"). Q35 says the same: real Q18 begin/end may retire an unpaired predecessor as already specified, while the assertion that no live run/scope/grant is overwritten belongs at insertion (`continuation-31/Q35-DESIGN.md:123`). The unchanged 170-control suite reproduces exactly two failures at eight and one thread on the same hash binary (168 passed / 2 failed), `armed::tests::stale_unpaired_episode_clears_on_new_begin` and its disabled counterpart, panicking on `PortBlocked::SecondLiveRun`; no other control fails and no nohash run was taken after the mandatory stop. The existing `LiveScopeAtBegin` assertion already rejects a begin under a live scope, and the other nine edits are at insertion or consumption sites. This is therefore a generated-instrument implementation error, not a director-order conflict, a production regression, a stale inventory or a changed expectation: Q18 and Q35 both stand.

Continuation 36 is ordered: render fresh from pinned inputs and repair the typed-auth `begin` so a new unqueued begin clears a stale unpaired predecessor exactly as Q18 requires, keeping the `LiveScopeAtBegin` protection and every one of the other nine registered edits at its insertion or consumption site; do not touch the frozen `episode-guard-stop` tree. Calibrate every retained new assertion: a deliberately weakened variant that omits each guard must fail exactly the named control it protects, the corrected candidate must keep the whole 170-control suite green, and the `LiveScopeAtBegin` assertion must be shown to fire (a begin attempted under a live scope, added as a calibration-only control if the fixed 170 cannot supply one — a check nobody has watched fail is not known to work). Then rerun the inherited 170 controls in hash and nohash at eight and one thread (4 runs, 680 controls), the typed defaults (24 runs / 240) and contexts (4 runs / 144), and any affected matrix; reuse the completed Q36 evidence (24/600, 16/48, 108/3,420, E22 24/528) and the J-identity evidence only by exact hash identity with every input unchanged, otherwise rerun it. Then resume the remaining bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. Budget: 3,998.3284827930283 s focused plus the untouched 6,000 s workspace; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344) and the continuation-35 `episode-guard-stop/` (832) are immutable; the maintained surface and production scope stay NONE; the stop-loss is the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 36 ruling (director) — continuation 36 is a compliant Q28 STOP: the ordered begin correction is accepted and closed (only the `SecondLiveRun` guard removed, `LiveScopeAtBegin` retained, the other nine registered edits exact, all ten guards calibrated with 80 valid controls and 76 expected rejections, the `live_scope_at_begin_fires_before_retirement` control now firing, the inherited 170 controls 680/0 in hash and nohash at eight and one thread), and `PortBlocked::ProfileControlLedgerResetReusesSourceEntry` is admitted as the worker's new-control setup error — the 14 added profile controls repeat `start()` on one thread, which resets only the `Ledger` while the independent `identity_proofs::RECORDS` keeps the previous entry, so the accepted `DuplicateSourceEntry` uniqueness guard fires and three armed and two disabled added controls fail at 79/5 on the same binary while all 70 fixed controls pass unchanged; continuation 37 corrects that setup without weakening the guard, calibrates the new profile assertions, reruns the profile matrix and resumes the bar under 2,895.527983784991 s

Continuation 36 is gated as a compliant Q28 STOP. It ran `cdx` (unit `brief-742-cdx-20261117-181546-83552`, rc 0, peak 3,113,058,304 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 6,215,912 B, sha256 `ef1c4ee2…`; report STOP, 6,346 changes — 6,344 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` change paths — 14 PASS / 2 FLAGGED / 2 FAIL / 3 NA, 33,197 citations verified, 0 questions, 1 deviation, 0 disclosures; 1,102.8004990080371 s charged — 509.5095211460357 s measured over 1,063 commands plus 531.5 s bookkeeping, the 60 s reserve and 1.790977862001455 s closing — of the 3,998.3284827930283 s focused remainder, leaving exactly 2,895.527983784991 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The report's accounting re-derives: 790 test runs at 20,306 passed / 106 failed / 0 ignored, the 106 being exactly the 96 deliberate rejections (76 retained-guard calibrations plus 20 raw weakness controls) and the 10 candidate instances (5 added profile controls at eight and one thread); the 700 filtered controls occur only in the 20 explicitly selected raw weakness tests, and every candidate and fixed matrix reports 0 filtered.

The continuation-35 finding is repaired and closed. The fresh render's begin audit shows only `SecondLiveRun` removed, `LiveScopeAtBegin` retained, the other nine registered edits exact and all ten inverse edits exact; the ten guard calibrations pass in hash and nohash at eight and one thread (4 valid runs / 80 controls, 40 weakened runs rejecting exactly 76 controls with no unexpected failure), including the `live_scope_at_begin_fires_before_retirement` control that now makes the retained live-scope assertion fire; the inherited 170 controls pass 680/0 in both cfgs at both thread counts, and the fixed assertions are unchanged. The bar's green evidence is accepted as delivered: the initial typed matrices give fifty rows / 600 runs / 9,864 passed with 0 failed and 0 filtered; the raw families pass (raw12 48 controls per run over three forms, frontier 224, helper 20, identity v1 8/256 and v2/v3 8/288 each); the profile keyed matrix passes 280 controls in four hash/nohash runs with all 70 fixed control bodies byte-identical to the accepted C27 source; the accepted E/J evidence is inherited by exact hash (Q36 nested 24/600, E22 24/528, Q36 calibration 16 runs with 352 passes and 48 deliberate failures, J20/J21/J54 108 runs / 3,420 with all 17 identity fields); twelve registered core edits restore exactly; 63 typed, source-record and raw fixtures restore whole bytes and AST over 181 parsed files / 1,136,571 nodes; all 94 admitted paths remain `not_emitted` with zero maintained byte delta.

The finding is admitted and the responsibility is the worker's: the 14 added profile controls repeatedly call `start()` inside one Rust test thread; `start()` replaces only `LEDGER` with a default ledger, resets `next_pair` to zero and repeats episode 1 and producer path 0, while `identity_proofs::RECORDS` is a separate thread-local registry that retains the previous immutable source entry, so the next `enter` reaches the accepted `DuplicateSourceEntry` uniqueness guard at `identity_proofs.rs.in:112`. Three armed added controls (two-owner orders, independent retirement, transported-token corruption) and two disabled counterparts fail, at 79 passed / 5 failed at eight and one thread on the same hash binary; the first iterations reach their intended expectations and later iterations are not established. All 70 inherited profile controls pass in those runs and in the prior 280-execution matrix, and their bodies are byte-identical to the accepted C27 source; the uniqueness guard is not weakened and no expected result is edited. This is a new-helper-control construction error, not a production regression, inventory-ordering artifact or director-order conflict, and no nohash or weakened profile result is inferred. The candidate is frozen in `profile-control-stop/` (2,555 files, copies and originals exact, candidate sha256 `7a6e28ad…`, binary sha256 `50678256…`, same-binary reproduction, no repair after STOP); the eight prior frozen trees (235, 574, 826 ×2, 292 ×2, 344, 832) rehash exactly; the entry is clean (5,400 maintained, 590 design, 17,276 accepted, the three drifts being the director documents committed in `fd350c8d`, `production_changes=0`, `maintained_changes=0`, the 90/180 pins and 151 rows exact, `f9604852…` unchanged before and after with `new_source_schema=null`, all 151 `source_schema` cells and every inventory byte exact). The two FLAGGED items are settled, not reds — the three raw render failures and two profile pin lookups are retained construction attempts, and the first final-evidence pass that omitted `guards-candidate` from its label classifier stopped and was corrected to reconcile all 790 runs — while the two FAIL checks (the added profile controls and the undelivered complete generator) and the three NA checks (the profile-new calibration, standing replay and workspace) are accepted as the stop's consequence, with no subset, placeholder digest or weakened expectation, and the one deviation is accepted on the same grounds.

Continuation 37 is ordered: render fresh from pinned inputs and correct the added profile controls' repeated-case setup so every repeated case starts from the same empty independent-registry state the fixed controls use (or a fresh identity), without weakening `DuplicateSourceEntry`, editing any expectation or touching the fixed 70, and without repairing the frozen `profile-control-stop` tree; deliver the profile-new calibration with deliberately weakened variants failing exactly their named controls, then rerun the profile keyed matrix (70 fixed plus 14 added) in hash and nohash at eight and one thread and any affected matrix; reuse the continuation-36 green evidence (the begin correction's 680, the ten guard calibrations, the typed and raw matrices, and the Q36/J/E22 evidence) only by exact hash identity with every input unchanged, otherwise rerun it. Then resume the remaining bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. Budget: 2,895.527983784991 s focused plus the untouched 6,000 s workspace; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344), the continuation-35 `episode-guard-stop/` (832) and the continuation-36 `profile-control-stop/` (2,555) are immutable; the maintained surface and production scope stay NONE; the stop-loss is the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 37 ruling (director) — continuation 37 is a compliant Q28 STOP: the continuation-36 profile setup correction is accepted and closed (the four-run profile matrix at 84/0 per run and 336 controls, the added 14 bodies and the independent uniqueness guard unchanged, 24 exact calibration rejections; the classifier flag is a corrected diagnostic), and `PortBlocked::PairedPendingSlotRejectsNestedActualPause` is admitted as the worker's new paired-instrument implementation error — the new `SecondPendingPairedPause` guard rejects the parent's first actual pause because the child's propagated token legitimately occupies the parent `EvalEntry.pending` slot, while the fixed `two_nested_producers_keep_own_proofs` control is byte-identical to its accepted continuation-28 body and passed 17/0 then; the guard must be corrected to distinguish a propagated child proof from a repeated actual parent pause, and continuation 38 repairs it, reruns P1566 in every form with P1805/P0001/P0002 and the new paired falsifiers, and resumes the bar under 824.1858909309767 s

Continuation 37 is gated as a compliant Q28 STOP. It ran `cdx` (unit `brief-742-cdx-20261117-192008-128960`, rc 0, peak 4,425,129,984 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 4,018,099 B, sha256 `091363fd…`; report STOP, 11,991 changes — 11,989 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` change paths — 16 PASS / 2 FLAGGED / 2 FAIL / 3 NA, 81,814 citations verified, 0 questions, 1 deviation, 0 disclosures; 2,071.3420928540145 s charged — 1,000.0061367780145 s measured over 2,019 commands plus 1,009.5 s bookkeeping, the 60 s reserve and 1.8359560760000022 s closing — of the 2,895.527983784991 s focused remainder, leaving exactly 824.1858909309767 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The report's accounting re-derives: 1,574 fresh test runs at 42,836 passed / 210 failed / 0 ignored / 0 filtered, the 210 being exactly the 208 deliberate rejections (each named against its frozen expectation manifest) and the 2 candidate instances (the one failing control at eight and one thread); all rustc builds succeeded.

The continuation-36 finding is repaired and closed. The profile setup correction changes only the repeated `start()` setup so the monotonic `next_pair` survives `Ledger` resets, its inverse restores the continuation-36 tree exactly, and the fixed 70 and added 14 bodies and assertions plus the independent uniqueness guard are unchanged; the profile matrix passes 84/0 per run in both hash runs and the four-run matrix at 336 controls with 0 filtered, and the four weak variants reject exactly their 24 named controls. The one FLAGGED classifier is a corrected diagnostic instrument — the first driver misread successful control names as failures and saved a false stop flag, both original hash runs were 84/0 at exit 0, and only the classifier changed while the candidate, binary and logs were reused by exact hash — so it is settled, not a red, together with the three retained renderer attempts (an E/J renderer that assumed a whole common helper, a premature matrix launch with no plan, and a retained-guard renderer whose empty removed string could not be inverted uniquely; no compiler rejection). The rest of the bar's evidence is accepted as delivered: the inherited 170 controls and the initial composed typed matrices (fifty rows, 600 runs, 9,864 passes across six forms and both thread counts, 0 failed / 0 filtered); the operation authority (214 × 4 = 856 passes and 8 weak variants over 32 full runs at 6,796 passes with 52 exact rejections, the direct duplicate check preceding every pair-proof write); the loop authority (four registered source fragments, 214 existing controls plus 16 new, candidate 4/920, four weak variants over 16 full runs at 3,648 passes with 32 exact rejections); the latest typed matrices (defaults 24/240 plus the remaining 48 rows 576/9,624, contexts 4/144, all required forms unfiltered); the retained ten guards (22 × 4 = 88 passes, ten weak variants over 40 runs at 76 exact rejections, `LiveScopeAtBegin` firing, the nine other edits and the original 20 controls exact); the E/J and Q36 source matrices (156 runs / 4,548: J20/J21/J54 108/3,420, E22 24/528, Q36 nested 24/600); and the historical evidence authentication (raw v6 28/2,528, raw v3 identity 8/288, raw v2 with 20 exact rejections and 700 intentional filtered tests, the historical Q36 calibration 16 runs with 352 passes and 48 rejections over 10,641 files). The paired-source candidate is green except the stop: the inherited 42 × 4 = 168 and the P0001/P0002 six-form source matrix 24/360 pass, and P1566 armed/hash fails exactly one control at 16/1.

The finding is admitted and the responsibility is the worker's. The unchanged `two_nested_producers_keep_own_proofs` control primes the nested and outer evaluators in mode 1; the child actually pauses and returns `InputRequired`, `Producer::Drop` rebases its authenticated returned token to the parent and fills `EvalEntry.pending` (`checkpoint/candidate.rs:119`, rebased at `:154`), and the outer evaluator then legitimately calls `pause_actual` (`:21`). The new `assert!(l.entries.last().unwrap().pending.is_none(), "PortBlocked::SecondPendingPairedPause")` at `:114` fires before the outer source record is written or its ordinal incremented, conflating a propagated child proof with a second actual parent pause. The control body is byte-identical to its accepted continuation-28 source (553 bytes, 17/0 there), the fixed source and expectation are unchanged, and both eight and one thread on the same armed/hash binary report 16/1/0/0 (candidate sha256 `2adcf43c…`, binary sha256 `9b5afa4b…`); no additional form was run after the STOP. This is a generated paired-instrument implementation error, not a production regression, an inventory-ordering artifact or a changed expectation; Q35 stands, and its source and return authority must be preserved when the guard is corrected. The candidate is frozen in `paired-pause-stop/` (4,751 files, copies and originals exact, `no_repair=true`, the open freeze log and the post-exit receipt excluded and separately hashed); the nine prior frozen trees rehash exactly (235, 574, 826 ×2, 292 ×2, 344, 832, 2,555); the entry is clean (5,400 maintained, 590 design, 23,619 accepted, the three drifts being the director documents committed in `ba2c58a6`, `production_changes=0`, `maintained_changes=0`, the 90/180 pins and 151 rows exact, `f9604852…` unchanged before and after with `new_source_schema=null`, all 151 `source_schema` cells and every other inventory byte exact, all 94 admitted paths `not_emitted` with zero maintained byte delta, and the 23 pre-existing change paths matching their entry hashes). The two FAIL checks (the paired-source candidate and the undelivered complete generator) and the three NA checks (the remaining paired controls, standing replay and workspace) are accepted as the stop's consequence, with no subset, placeholder digest or weakened expectation, and the one deviation is accepted on the same grounds.

Continuation 38 is ordered: render fresh from pinned inputs and correct the paired instrumentation so a propagated child proof is distinguished from a repeated actual parent pause — the parent's first actual `pause_actual` must write its own source record and ordinal even when the child's rebased token occupies `EvalEntry.pending`, while a genuinely repeated actual parent pause for the same owner must still be rejected, with the source and return authority and every fixed control unchanged and the frozen `paired-pause-stop` tree untouched. Calibrate the corrected guard with deliberately weakened variants that fail exactly the named control (a variant that admits the repeated actual parent pause must be red on its named control, and the fixed `two_nested_producers_keep_own_proofs` must stay green). Then rerun the P1566 checkpoint group in every form and cfg at eight and one thread with P1805 and P0001/P0002 and the new paired falsifiers, and reuse the continuation-37 green evidence (profile 336, typed 9,864, operation 6,796/52, loop 3,648/32, retained guards 88/76, E/J/Q36 4,548, historical 10,641 files) only by exact hash identity with every input unchanged, otherwise rerun it. Then resume the remaining bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. Budget: 824.1858909309767 s focused plus the untouched 6,000 s workspace; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344), the continuation-35 `episode-guard-stop/` (832), the continuation-36 `profile-control-stop/` (2,555) and the continuation-37 `paired-pause-stop/` (4,751) are immutable; the maintained surface and production scope stay NONE; the stop-loss is the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

## Continuation 38 ruling (director) — continuation 38 is a compliant Q28 STOP: the continuation-37 paired-pause guard correction is accepted and closed (the single registered guard replacement checks canonical actual-owner source records instead of the pending slot, the fourteen registered edits and the inverse to C28 exact, the fixed 42/62 bodies byte-identical, the candidate matrix 248/0 in four runs, six weakened variants over 24 runs at 64 exact rejections and 0 unexpected, and the paired-source matrix 48 runs / 768 controls green in every form and cfg at eight and one thread), and `PortBlocked::PairedReturnTransportRevivesInvalidatedReturn` is admitted as a genuine candidate-contract lifecycle finding — the accepted continuation-27 retirement (a fabricated second return retires the first) is carried only by the absence of the token in the transported `EvalPair.proofs` map, while the immutable Q34 `sources`/`returns` records keep the token and the queued `enter` authenticates against exactly those records, so restoring the transport token recreates the queued issue; Q37 rules the invalidation recorded in an independent owner-keyed current-return carrier checked at issuance and eligibility, and continuation 39 implements and calibrates it, reruns the lifecycle and P1566/P1805/P0001/P0002 matrices, and resumes the bar under 596.3096664039977 s

Continuation 38 ran `cdx` (unit `brief-742-cdx-20261117-203934-208453`, rc 0, peak 937,566,208 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 1,752,518 B, sha256 `6d445b3f…`; report STOP, 847 changes — 845 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` change paths — 11 PASS / 0 FLAGGED / 2 FAIL / 2 NA, 4,199 citations verified, 0 questions, 1 deviation, 0 disclosures; 227.87622452697906 s charged — 104.69010826098383 s measured over 124 commands plus 62 s bookkeeping, the 60 s reserve and 1.1861162659952242 s closing — of the 824.1858909309767 s focused remainder, leaving exactly 596.3096664039977 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The report's accounting re-derives: 78 fresh test runs at 2,568 passed / 76 failed / 0 ignored / 0 filtered, the 76 being exactly the 64 deliberate rejections (the six weakened variants, each named against its plan) and the 12 candidate instances (the six failing controls at eight and one thread), with every rustc build succeeding.

The continuation-37 finding is repaired and closed. The rendered paired helper differs from the continuation-37 tree by exactly one registered line: the `SecondPendingPairedPause` guard now rejects only a source record that is the entry owner's own canonical actual pause (`sources` contains a token whose owner and origin both equal the entry owner), so the child's rebased token — owner rebased to the parent, origin retained — no longer blocks the parent's first actual `pause_actual`, while a genuinely repeated actual parent pause still rejects before the ordinal or source write, including after `pending` has been consumed by a return. The registered audit is exact (`edits=14 inherited13_exact=true correction=actual_source_owner_guard source_return_issue_authority_exact=true inverse_to_C28_exact=true`), the four-run candidate matrix passes 248/0 with `fixed42_exact=true`, and the six weakened variants are calibrated in both cfgs at both thread counts over 24 full runs: 1,424 passed and 64 exact rejections, 0 unexpected, 0 ignored, 0 filtered. The two decisive calibrations are exact — `weak-pending-only` (the continuation-37 guard restored) is red on exactly `propagated_child_then_actual_parent_keeps_both_proofs` and `repeated_actual_parent_pause_after_return_is_rejected` in both modules, and `weak-repeated-pause` (the corrected guard removed) is red on exactly the two repeated-actual-pause controls in both modules, with the fixed `two_nested_producers_keep_own_proofs` green. The paired-source matrix is green in every form and cfg at both thread counts: 48 source runs and 768 controls (P1566 204, P1805 204, P0001/P0002 360), 0 failed / 0 filtered, single latest common helper, and the rendered source and fixed control bodies exact with a whole-byte inverse.

The rest of the bar's evidence is accepted as delivered by exact hash identity. 11,862 files authenticate, including 10,641 historical files, with `source_helper_controls_binary_logs_exact=true` in every group: profile 336, typed defaults and contexts 384 plus the remaining 48 rows 9,624, operation 856 plus 6,796/52, loop 920 plus 3,648/32, the retained guards with 76 exact rejections, and E/J/Q36 4,548. Seven paired fixtures restore by whole bytes and AST over 22 parsed files and 73,013 syntax nodes with the fixed 42 and all paired source controls exact and the added 20 the only additions; the return audit restores the prior 62 controls whole and shows the registered helper unchanged, so the lifecycle fixture is the accepted 62-control candidate plus exactly eight additive tests (two insertion points, no deletions). The candidate is frozen in `paired-return-stop/` (331 files, originals and copies exact, candidate sha256 `ba35d117…`, binary sha256 `571fb03f…`, same-binary reproduction at eight and one thread, `no_repair=true`, the open freeze log and the post-exit wrapper receipt excluded and separately hashed); the ten prior frozen trees rehash exactly (235, 574, 826 ×2, 292 ×2, 344, 832, 2,555, 4,751), the two open trees' single differing original each recorded as before. The entry is clean — 5,400 maintained, 590 design, 35,607 accepted, the three drifts being the director documents committed in `19dbe5bf`, `production_changes=0` and `maintained_changes=0`, the 90/180 pins and 151 rows exact, `f9604852…` unchanged before and after with `new_source_schema=null`, all 151 `source_schema` cells and every other inventory byte exact, all 94 admitted paths `not_emitted` with zero maintained byte delta and zero complete postimages, and the 23 pre-existing change paths (32 file instances) matching their entry hashes.

The finding is admitted, and it is not a construction error in the added controls. The lifecycle fixture is additive (no deletion), the 62 fixed bodies and the registered helper are byte-identical, and the valid counterpart `valid_actual_return_accepts_identical_transport` passes in both modules; the failing assertion is the accepted continuation-27 duty (`fabricated_second_input_retires_first`) extended to the Q34 transport model. After a first actual return authenticates the token, a second `returned` call with no new actual pause — a fabricated second `InputRequired`, an `Ok`, or another error — consumes nothing and removes the token from the transported `EvalPair.proofs` map only (`candidate.rs:70`/`:315`), while the immutable `sources` and `returns` records keep it (`:71`/`:316`); the paired `Drop` and `enter` then treat the pair as transportable, and a queued `enter` that receives the restored token authenticates it against exactly those records (`:34`/`:279`) and recreates the queued issue at `:35`/`:280`. The invalidation therefore has no durable carrier separate from the immutable Q34 records, which is the carrier class Q35 classifies; the six failures are the three invalidated modes in both armed and disabled modules, at 64/6/0/0 on the same binary at eight and one thread, and no nohash or additional form was run after the STOP. This is a candidate-contract lifecycle gap, not a production regression, an inventory-ordering artifact or a changed expectation; Q34's identity authentication and Q35's source/return authority stand, the immutable records stay immutable, and the repair must add the missing carrier rather than weaken either rule.

The two FAIL checks (the paired-return lifecycle and the undelivered complete generator) and the two NA checks (standing replay and workspace) are accepted as the stop's consequence, with no subset, placeholder digest or weakened expectation, and the one deviation (the remaining closure, generator, schema, replay and workspace gates undelivered because the stop preceded maintained integration) is accepted on the same grounds. No FLAGGED check was reported.

Q37 rules the return-invalidation lifecycle: an invalidation of an actual-return proof must be recorded in an independent current-return carrier keyed by the Q35 source key (pair, origin owner, owner, occurrence and actual return edge), never inferred from the absence of the token in the transported `EvalPair.proofs` map alone; every boundary that issues or confers the return authority — the queued `enter` issue, `valid`, `operation`/`omit` and `hit` — authenticates the current carrier together with the immutable Q34 `sources`/`returns`, so restoring, replaying or corrupting the transported proof map cannot revive a revoked return, while the valid actual return keeps issuing and hitting, one-use consumption and retirement stay separate from the Q34 records, and no fixed control body or expectation changes.

Continuation 39 is ordered: render fresh from pinned inputs and implement Q37 in the paired instrumentation with the independent current-return/revocation carrier, without mutating or reinterpreting the immutable Q34 records, without changing the 62 fixed control bodies or the registered helper's source/return authority, and without repairing the frozen `paired-return-stop` tree. Calibrate with deliberately weakened variants that fail exactly the named controls: a variant omitting the revocation check must be red on exactly the six invalidated lifecycle controls with `valid_actual_return_accepts_identical_transport` green in both modules, a variant restoring the continuation-37 pending-slot guard must stay red on exactly the named propagation and repeated-pause controls, and the six existing weaknesses' expected rejections must be unchanged. Then rerun the lifecycle fixture in hash and nohash at eight and one thread, the 62-control paired matrix in both cfgs at both thread counts, and the P1566/P1805/P0001/P0002 source matrices in every form and cfg at eight and one thread; reuse the continuation-38 green evidence (the hash-identity groups listed above) only with every input unchanged by exact hash, otherwise rerun it. Then resume the remaining bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. Budget: 596.3096664039977 s focused plus the untouched 6,000 s workspace; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344), the continuation-35 `episode-guard-stop/` (832), the continuation-36 `profile-control-stop/` (2,555), the continuation-37 `paired-pause-stop/` (4,751) and the continuation-38 `paired-return-stop/` (331) are immutable; the maintained surface and production scope stay NONE; the stop-loss is the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Continuation 39 is ordered but its dispatch waits for the user's Codex account switch; the work pauses here.

## Continuation 39 ruling (director) — continuation 39 is a compliant Q28 STOP: Q37 is implemented and accepted — the independent owner-keyed current-return carrier (`ReturnKey` over the full Q35 source identity and the actual `InputRequired` edge, `EvalPair.current_returns` recording current and revoked state) authenticates the queued `enter` issue, `valid` and transitively `operation`/`omit`/`hit` and the `Drop` propagation, and an actual return revokes only its owner's matching keys before recording the valid return, with the immutable Q34 `sources`/`returns` untouched, the twenty-one registered edits and their inverse to C28 exact, the fixed 42/62 bodies byte-identical, the lifecycle 280, the fixed-62 matrix 248, the consumers 320 and the paired-source 768 all green, and the calibrations 24/16/64 exact — while `PortBlocked::RequestReturnTransportRevivesInvalidatedReturn` is admitted as a genuine candidate-contract lifecycle finding: the request-return invalidation is carried only by the removal of the token from the transported `Pair.request_returns` map, the immutable `request_records`/`return_records` keep it, and the unchanged common helper's `request_token` (the `ordinary_request` boundary) reads the restored transport and authenticates it against those historical records, so restoring the transport revives the armed request permission; Q38 rules the invalidation recorded in an independent current-request-return carrier checked at the request-return boundaries, and continuation 40 inventories every affected helper and fixture, implements and calibrates it, reruns every affected matrix and resumes the bar under 323.9135218299516 s

Continuation 39 ran `cdx` (unit `brief-742-cdx-20261117-212151-224679`, rc 0, peak 1,049,178,112 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 3,326,969 B, sha256 `0c18c0aa…`; report STOP, 1,278 changes — 1,276 added, 2 modified, 0 deleted, all in `.temp` with zero non-`.temp` change paths — 14 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 588 citations verified, 0 questions, 1 deviation, 0 disclosures; 272.39614457404605 s charged — 128.21224255604466 s measured over 166 commands plus 83.0 s bookkeeping, the 60 s reserve and 1.1839020180013904 s closing — of the 596.3096664039977 s focused remainder, leaving exactly 323.9135218299516 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The report's accounting re-derives: 94 fresh test runs at 4,038 passed / 110 failed / 0 ignored / 0 filtered, the 110 being exactly the 104 deliberate rejections (the six weaknesses' 64, the Q37 lifecycle calibration's 24 and the consumer calibration's 16) and the 6 candidate instances (the three failing armed controls at eight and one thread), plus three static expected rejections and one settled launch build error; every rustc build succeeded.

The Q37 implementation is accepted. It is exactly seven registered additions to the inherited fourteen: `ReturnEdge::ActualInputRequired` and `ReturnKey { source: EvalToken, edge }` over the full `EvalToken` source identity; `EvalPair.current_returns: BTreeMap<ReturnKey,bool>`; the queued-issue filter; `valid`; the `pause_actual` owner-keyed revocation, which clears every matching `current_returns` key and asserts `PortBlocked::RepeatedActualReturnKey` before inserting the new key; and the `Drop` propagation filter. The registered audit is exact (`edits=21 inherited14_exact=true correction=independent_current_return_carrier source_return_issue_authority_exact=true inverse_to_C28_exact=true`); no immutable Q34 record is removed or reinterpreted, and the fixed 42/62 control bodies are byte-identical. The lifecycle fixture is the accepted continuation-38 candidate plus the seven carrier edits at both copies (492 to 524 lines, no control-body change), and its four runs pass 70 controls each (280); the fixed-62 matrix passes four runs at 248; the consumer fixture is the exact 70 plus ten additive controls (80) and passes four runs at 320; the paired-source matrix is green in every form and cfg at eight and one thread (48 source runs, 768 controls: checkpoint 204, paired_bridge 360, association_target 204, 0 failed / 0 filtered). I reproduced the candidate lifecycle at 70/0 and the fixed-62 at 62/0 in all four runs, the consumer at 80/0, and three source rows (P1566 17/0, P0001 15/0, P1805 17/0).

The calibrations are exact. `weak-current-return` is the byte-derived variant of the lifecycle candidate with the `current_return` body replaced at both copies; four runs pass 256 and reject exactly 24 (the six invalidated controls — fabricated-second-input, successful and other-error returns in armed and disabled modules), with `valid_actual_return_accepts_identical_transport` green in both modules and the prior 62 green; I reproduced 64/6 at eight and one thread in hash and nohash and the valid counterpart green under the variant. `weak-consumer-current` removes only the `valid` current check at both copies; four runs pass 304 and reject exactly 16 (operation issue armed and disabled, omit armed, pre-hit armed), the prior 70 green; I reproduced 76/4. The six existing weaknesses are the accepted byte-derived variants of the 62-control candidate and run 24 times at 1,424 passed and 64 exact rejections with 0 unexpected: `weak-pending-only` is red on exactly `propagated_child_then_actual_parent_keeps_both_proofs` and `repeated_actual_parent_pause_after_return_is_rejected` in both modules, `weak-repeated-pause` on exactly the two repeated-actual-pause controls in both modules, and both reproduce.

The rest of the bar's evidence is accepted as delivered by exact hash identity. 11,862 files authenticate, including 10,641 historical files, with `source_helper_controls_binary_logs_exact=true` in every group (profile 336 plus calibration 1,320/24; typed defaults and contexts 384 plus the remaining 48 rows 9,624; operation 856 plus 6,796/52; loop 920 plus 3,648/32; retained guards 100/76; E/J/Q36 4,548). Seven paired fixtures restore by whole bytes and AST over 22 parsed files and 73,983 syntax nodes with the fixed 42 and the added 20 unchanged; the separate additive audit over eight files and 91,610 nodes shows the request fixture is the accepted continuation-37 candidate plus exactly eight tests at two insertion points with zero deletions (its erased form is byte-identical to `777c01fc…`), the lifecycle fixture erases to its 62, and the consumer fixture to its 70. The partial generator delivers 21 declarative paired recipes over two scratch roots with deterministic bytes, exact immutable fields, whole-byte/AST restoration, the pinned omission and the three named static rejections; it is a partial, not the complete 151-plan bundle.

The finding is admitted, and it is not a construction error in the added controls. The request fixture is the accepted continuation-37 operation candidate (sha256 `777c01fc…`) byte-exact with eight additive tests, the common helper is the exact accepted `operation_authority_v2/combined.rs.in` (sha256 `28f580f8…`), the fixed 214 controls pass, the valid counterpart `valid_request_return_accepts_identical_transport` and the disabled modal counterparts pass, and the fixed duty `fabricated_second_return_lacks_pending_proof` is green. The added controls save the authenticated transport, execute a second fabricated `InputRequired`, a successful return or another error, restore only `Pair.request_returns` and assert full independent-record and other-state equality; the second return's invalidation removes the transport at `:508` and never revokes the immutable `return_records` (`:516`) or `request_records` (`:517`), and `request_token` (`:525`) reads the restored transport at `:531` and accepts the historical record at `:534`, so `ordinary_request` returns true where false is required. The three armed invalidated controls fail at `candidate.rs:1620` on the same binary at eight and one thread (candidate sha256 `54c1a70a…`, binary sha256 `868d3d50…`, 219/3/0/0 each); no nohash or additional form was run after the STOP. The continuation-31 inventory declares `core::Pair.request_returns` as a carrier whose take is the exact boundary ticket and whose retirement removes only its SourceKey, so the invalidation's only carrier is transport absence — the carrier class Q35 classifies. The candidate is frozen in `request-return-stop/` (479 files, originals and copies exact, `no_repair=true`, the open freeze log and the post-exit receipt excluded and separately hashed); the eleven prior frozen trees rehash exactly (235, 574, 826 ×2, 292 ×2, 344, 832, 2,555, 4,751, 331), with the two open trees' single freeze-log drift each recorded as before. The one FLAGGED launch error is settled: the first request-fixture build omitted the already-used `rum_core` `-L`/`--extern` flags (E0433), the original driver and log are retained, and the separately named driver restores those exact flags and compiles the identical candidate at the same source sha with zero source or control changes.

The two FAIL checks (the request-return lifecycle and the undelivered complete generator) and the two NA checks (standing replay and workspace) are accepted as the stop's consequence, with no subset, placeholder digest or weakened expectation, and the one deviation (the remaining bar undelivered because the stop preceded its completion) is accepted on the same grounds; no FLAGGED check remains unsettled.

Q38 rules the request-return invalidation lifecycle: an invalidation of a request-return proof must be recorded in an independent current-request-return carrier keyed by the Q35 source key (pair, origin owner, owner, occurrence and request-return edge), never inferred from the absence of the token in the transported `Pair.request_returns` map alone; every boundary that confers or consumes the request-return authority — the `ordinary_request`/`request_token` permission and the request-return operation, omit and hit consumers — authenticates the current carrier together with the immutable Q34 `request_records`/`return_records`, so restoring, replaying or corrupting the transported request proof cannot revive a revoked request return, while the valid request return keeps granting and hitting, one-use consumption and retirement stay separate from the immutable records, and no fixed control body or expectation changes. Q37 stands unchanged for the actual-return carrier.

Continuation 40 is ordered: render fresh from pinned inputs and implement Q38 in every accepted helper and fixture that carries the request-return boundary, after first deriving a complete zero-unknown inventory of that set from the accepted sources (the Q35 carrier list declares `core::Pair.request_returns`; every helper containing the request-return writer/consumer and every fixture and matrix it renders into must be named), without mutating or reinterpreting the immutable Q34 records, without changing the fixed 62/70/80/214 control bodies, the Q37 actual-return carrier or the source/return/issue authority, and without repairing any frozen tree. Calibrate with deliberately weakened variants that fail exactly the named controls: a variant omitting the current-request-return check must be red on exactly the invalidated request-return controls derived from the fixture (per Q36), with `valid_request_return_accepts_identical_transport` and the fixed 214 green; the continuation-39 Q37 calibration and the six existing weaknesses' expected rejections must be unchanged. Then rerun the request lifecycle fixture in hash and nohash at eight and one thread, and every affected family matrix — operation, loop, profile, typed, retained guards, E/J/Q36 and the paired source matrices — in every form and cfg whose rendered helper bytes change; reuse the continuation-39 green evidence only by exact hash identity, otherwise rerun it. Then resume the remaining bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, every affected per-row matrix with hash-identity reuse only for unchanged evidence, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips, `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0. Budget: 323.9135218299516 s focused plus the untouched 6,000 s workspace; the 1,440 s verification envelope and second 6,000 s workspace stay held; CM4's committed share stays 28,500 s and 743–746 retain 40,500 s. The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344), the continuation-35 `episode-guard-stop/` (832), the continuation-36 `profile-control-stop/` (2,555), the continuation-37 `paired-pause-stop/` (4,751), the continuation-38 `paired-return-stop/` (331) and the continuation-39 `request-return-stop/` (479) are immutable; the maintained surface and production scope stay NONE; the stop-loss is the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair. No campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

The entry is clean — 5,400 maintained, 590 design, 36,451 accepted, the three drifts being the director documents committed in `19dbe5bf` and `9f5aaf5b` (TODO.md `b5522339…`, brief-623 `2856b40e…`, brief-742 `e0d8e315…`), `production_changes=0` and `maintained_changes=0`, the 90/180 pins and 151 rows exact, `f9604852…` unchanged before and after with `new_source_schema=null`, all 151 `source_schema` cells and every other inventory byte exact, all 94 admitted paths `not_emitted` with zero maintained byte delta and zero complete postimages, and the 23 pre-existing change paths (32 file instances) matching their entry hashes.

## Continuation 40 ruling (director) — continuation 40 is a compliant allocation STOP: Q38 is implemented and accepted — the independent current-request-return carrier (`RequestReturnKey` over the full `Token` source identity and the `ActualInputRequired` edge in `Pair.current_request_returns`) records current and revoked state, an actual request return revokes every matching owner key before it issues the new key, and the `request_token` permission, the hit lookup and the request-return propagation authenticate the current carrier together with the immutable Q34 `source_records`/`return_records`/`request_records`, with the seven registered edits and their inverse byte-exact, the fixed 214 and the eight added lifecycle bodies unchanged, the 222-control lifecycle green in four runs, the carrier-omitting variant red on exactly the three invalidated controls (12 exact rejections) with the valid counterpart and the fixed 214 green, all 95 affected sources rendered with exact byte/AST audits and zero unknown shapes, and the 120 completed changed-input runs plus the 124 hash-identical unchanged runs matching their frozen expectations with zero candidate failures — while the focused allocation, the last residue of CM4's exhausted committed share, stopped before the next complete build and eight/one-thread pair plus closure bound: the remaining 800 changed-input runs, the complete 151-plan generator, the 94-target/188-postimage roots and the mechanical schema, the ordered standing replay and both workspace modes stay undelivered, so continuation 41 resumes the exact pending plan and the remaining bar under a fresh focused pool drawn from an extension of CM4's committed share

Continuation 40 ran `cdx` (unit `brief-742-cdx-20261117-220835-247662`, rc 0, peak 1,252,319,232 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 4,346,207 B, sha256 `f3fd0420…`; report STOP, 1,286 changes — 1,284 added, 2 modified (`PROGRESS.md`, `report.json`), 0 deleted, all under `.temp/brief-742` with zero non-`.temp` change paths — 12 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 729 citations verified, 0 questions, 1 deviation, 0 disclosures; 306.0892920769911 s charged — 144.85304461299165 s measured over 200 charged receipts plus 100.0 s bookkeeping, the 60 s preparation/reporting reserve and 1.2362474639994616 s closing — of the 323.9135218299516 s focused carry, leaving exactly 17.82422975296049 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held). The accounting re-derives: the 200 charged receipts' measured durations sum to 144.853045 s exactly, the changed-matrix driver's 120 child runs take 5.256 s of it, the command envelope is `323.9135218299516 − 60 = 263.9135218299516` s, the stop fired at 223.25917265697717 s spent because the next complete build and pair bound (9.5 s) plus the 32 s closure reserve no longer fit, and the report records `candidate_failure=false` and `no_envelope_overrun=true`. The fresh total is 128 test runs at 15,720 passed / 112 failed / 0 ignored / 0 filtered — the 120 completed changed-input runs at 13,956 passed / 100 exact rejections (operation calibration 52, loop calibration 32, profile calibration 16) plus the four request candidate runs at 888 and the four weak runs at 876/12 — and the stale-free reuse adds 124 runs at 6,056 passed / 124 exact rejections.

The Q38 implementation is accepted. It is exactly seven registered edits, each old text unique in the accepted `operation_authority_v2/combined.rs.in` (`28f580f8…`): the `RequestReturnEdge::ActualInputRequired` and `RequestReturnKey { source: Token, edge }` declarations over the full `Token` source identity; `Pair.current_request_returns: BTreeMap<RequestReturnKey, bool>`; the `current_request_return` reader binding the queried owner, the origin episode and the pair episode to the exact key; the `return_authorized` request-propagation conjunct; the `request_return` owner-keyed revocation of every matching key while the transport removal is kept; the issue-time `PortBlocked::RepeatedRequestReturnKey` vacancy assert and `(key, true)` insert before the immutable records; and the `request_token` conjunct requiring the current carrier plus `source_records`, the `Request`-kind `return_records` and `request_records`, which the `ordinary_request` permission and the hit fallback consume. I applied the seven edits to the accepted helper myself: the forward render is byte-identical to the c40 helper (`f2e011c0…`), the inverse restores the continuation-37 helper byte-exactly (`28f580f8…`), and each old text occurs exactly once. The fixture is the accepted continuation-39 candidate (`54c1a70a…`) plus exactly those seven blocks at both copies and nothing else — the diff is purely additive, so the 222 controls are the fixed 214 plus the eight request-return lifecycle controls, byte-identical, no fixed body moved. The four lifecycle runs pass 222/0/0/0 in hash and nohash at eight and one thread (I reproduced all four); the calibration replaces only the carrier-map lookup with `&& true` at both copies (`02deaf0e…`), and its four runs pass 219 and fail exactly `fabricated_second_request_return_cannot_be_revived`, `other_error_request_return_cannot_be_revived` and `successful_request_return_cannot_be_revived` in the armed module — 12 exact rejections — with `valid_request_return_accepts_identical_transport` and the fixed 214 green (I reproduced the exact three names at 219/3 in all four forms). The hash and nohash builds compile to identical bytes because this fixture does not branch on `feature="hash"`; both cfgs and both thread counts are nevertheless exercised. The immutable Q34 records are untouched, the Q37 actual-return carrier is unchanged, and the fixed 62/70/80/214 bodies and every expectation are exact.

The affected-source census is accepted as the complete zero-unknown inventory the order required: 21 matrix groups, 122 active sources, 95 affected sources, 1,203 accepted historical occurrences, 92 current Q34 shapes and three declared historical shapes with zero unknowns; the three historical sources use the already accepted continuation-37 component rebind and their fixed controls are exact. The 95 sources render with 127 helper copies, zero unknown anchors, exact pre-edit hashes and whole-byte inverse (I re-hashed all 95 sources and targets and re-read every render-plan flag), and the independent AST audit passes all 95 files with the declarations, the owner revocation, the immutable-record checks and the original branches exact and every other node unchanged (I reran the compiled auditor myself: `PASS files=95 declarations_exact=true owner_revocation_exact=true immutable_records_preserved=true original_branches_exact=true all_other_nodes_exact=true`).

The completed changed-input evidence is accepted: 120 runs match their original named results exactly — operation candidate 856, loop candidate 920, profile 336, paired bridge P0001/P0002 360, typed defaults/contexts 384, operation calibration 6,796/52, loop calibration 3,648/32 and the first eight profile-calibration runs 656/16 — with `cfg_forms_threads_exact=true`, 100 exact rejections and zero unexpected; I reran twelve of them across all five completed candidate and calibration groups at eight and one thread and every exit, count and failing name matched. The 124 unchanged runs authenticate by exact hash identity only — Q37 fixed-62 and the six weaknesses 28 runs / 1,672 / 64, lifecycle-70 280, consumers-80 320, return calibration 256/24, current-consumer calibration 304/16, P1566/P1805 408, raw-v6 2,528, raw-v3 288 and the twenty weakness rejections — every source exists with its recorded sha256, none contains the new Q38 symbol, and no changed helper or source is reused; the fresh P0001/P0002 evidence pairs with the retained P1566/P1805 rows.

The one FLAGGED check is settled as instrument construction: the historical helper lacked `ReturnKind`, the first contiguous helper match was rejected by the accepted queue extensions and the accepted component recipe replaced it, the audit build retired E0432 and E0277 to an already cached matching syn/quote pair with no dependency or manifest change, the first AST eraser dropped two `Pair` trailing commas (diagnosed as the sole token delta and corrected), and the matrix plan v2 scans command dictionaries without weakening any check; no candidate control failed and no expectation moved. The two FAIL checks (the 800 remaining changed-input runs and the undelivered complete generator) and the two NA checks (standing replay and workspace) are accepted as the allocation stop's consequence, with the one deviation (the remaining bar undelivered) accepted on the same grounds; nothing not run is credited. No new candidate finding is admitted: Q38 is implemented and its falsifier fires exactly, and Q37 stands.

Continuation 41 is ordered. Resume the exact pending plan: authenticate the c40 `changed-matrix-plan.json`, its rendered sources and its binaries, start from `final-weak-profile-return-hash-t8`, and complete all 800 remaining changed-input runs — profile calibration 8, retained guards 44, C35/Q36 nested calibration 16, E/J/Q36 156 and the remaining 48 typed rows 576 — in their recorded cfgs, forms, selectors and eight/one-thread commands, without rerunning consumed references and without substituting stale evidence. Then the complete bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips at `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0, with the switched same-binary checks. Reuse any continuation-39/40 green evidence only by exact hash identity with every input unchanged, otherwise rerun it; no frozen tree is repaired, no production or maintained source changes, no expectation, inventory or derived cell is edited; the stop-loss stays the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair.

Budget. The carried 17.82422975296049 s plus a fresh 8,000 s focused allocation gives continuation 41 an 8,017.82422975296049 s focused envelope; the assigned 6,000 s workspace envelope stays untouched for the workspace gates, and the 1,440 s verification envelope and second 6,000 s workspace stay held for the verification stage. The exhausted carry shows CM4 has committed the whole 28,500 s of its share, so the share is extended by 8,000 s to 36,500 s and the stage reserve from 69,000 s to 77,000 s; 743–746 retain their 40,500 s unchanged. The extension is an allocation from the project's reserve, not a bar change: the bar, the stop-loss and the authorization limits are unchanged.

The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344), the continuation-35 `episode-guard-stop/` (832), the continuation-36 `profile-control-stop/` (2,555), the continuation-37 `paired-pause-stop/` (4,751), the continuation-38 `paired-return-stop/` (331) and the continuation-39 `request-return-stop/` (479) are immutable; I rehashed all 12,337 manifest entries and found zero bad copies or originals, with only the two known open `freeze-stop.log` drifts in the continuation-32 and continuation-33 open trees (their closed copies exact). The maintained surface and production scope stay NONE; no campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

The entry is clean — 5,400 maintained, 590 design and 37,726 accepted files authenticate, `production_changes=0` and `maintained_changes=0`, the three inherited director-document drifts are the c39 gate's committed `TODO.md` (`d6c7e092…`), brief-623 (`2f829a21…`) and brief-742 (`981a88c7…`), the 90/180 pins and 151 rows are exact, `f9604852…` is unchanged before and after with `new_source_schema=null`, all 94 admitted paths are `not_emitted` with zero maintained byte delta and zero complete postimages, and the 32 pre-existing change instances match their entry hashes (I verified all 32 myself).

## Continuation 41 ruling (director) — continuation 41 is a compliant Q28 STOP with the pending Q38 plan fully delivered: the 800 remaining changed-input runs complete the c40 plan exactly — profile calibration 8, retained guards 44, C35/Q36 nested calibration 16, E/J/Q36 156 and the remaining 48 typed rows 576, every run in its recorded cfg, form, selector and eight/one-thread command, 15,288 passed and 132 exact rejections with zero unexpected and zero candidate failures — so the combined 920-run plan stands at 29,244 passed / 232 exact rejections / 0 ignored / 0 filtered and the 124 unchanged references authenticate at 6,056 / 124 / 700 filtered without a rerun, while the one FLAGGED construction attempt is settled as instrument correction; the next initial-return lifecycle control then exposes `PortBlocked::InitialReturnTransportRevivesInvalidatedReturn` as a genuine candidate-contract lifecycle finding: a second `InputRequired` without a new pause, a success or another error invalidates the first initial return only by removing the token from the transported `Run.returning` map while the immutable Q34 `return_records` keep it, so restoring only that transport revives the invalidated authorization — `(false,1,0)` becomes `(true,1,1)` with the permission and a hit — on the same 227/3 binary at eight and one thread; Q39 rules the invalidation recorded in an independent current-initial-return carrier checked at the initial permission and the hit consumer, and continuation 42 inventories every remaining carrier's invalidation mechanism class-wide, implements Q39 and any further transport-only carrier the inventory proves, reruns every affected matrix and finishes the remaining bar under the carried 7,150.0300783879675 s focused envelope.

Continuation 41 ran `cdx` (unit `brief-742-cdx-20261117-224956-281247`, rc 0, peak 2,516,201,472 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 3,386,268 B, sha256 `1aef13a2…`; report STOP, 5,542 changes — 5,540 added, 2 modified (`PROGRESS.md`, `report.json`), 0 deleted, all under `.temp/brief-742` with zero non-`.temp` change paths — 11 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 2,971 citations verified, 0 questions, 1 deviation, 0 disclosures; 867.794151364993 s charged — 300.3073031679887 s measured over 1,012 charged receipts plus 506.0 s bookkeeping, the 60 s preparation/reporting reserve and 1.4868481970042922 s closing — of the 8,017.82422975296049 s focused envelope, leaving exactly 7,150.0300783879675 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held; `no_envelope_overrun=true`, `allocation_stop=false`, `candidate_control_stop=true`). The accounting re-derives: the 1,012 charged receipts' measured durations sum to 300.3073031679887 s exactly, the 506.0 s bookkeeping is the 1,012 × 0.5 s parent charge, and the charged total plus the 60 s reserve and the 1.4868481970042922 s closing is 867.794151364993 s, exactly the reported charge; the 802 fresh test runs take 15,742 passed / 138 failed / 0 ignored / 0 filtered — the 800 matrix runs at 15,288/132 plus the two initial-return runs at 454/6 — where the 138 are the 132 deliberate rejections and the six candidate instances (three armed controls at eight and one thread).

The 800-run plan is accepted as delivered. The plan is the exact continuation-40 `changed-matrix-plan.json` (`76136f97f6a133b1b8a8ed76abb2970427fe87c47f190d4748be7bdc6e2753dc`), and its 800 pending labels plus the 120 consumed continuation-40 labels are disjoint and together cover the 920-run plan exactly; every pending run keeps its recorded cfg, form, selector and eight/one-thread command, and every result matches its plan entry — the target source sha256, the changed command (the original command with the changed binary), the threads, and the expected exit, counts and failing-name set — with `expected_exact=true` on all 800. I rehashed all 264 distinct changed sources and binaries and all 994 distinct original binaries and logs (0 bad), and the verdict records 0 reference reruns, 0 remaining runs and 0 candidate failures. The group subtotals are exact: profile calibration 8 runs at 664/8, retained guards 44 at 100/76, the C35/Q36 nested calibration 16 at 352/48, E/J/Q36 156 at 4,548/0 and the remaining 48 typed rows 576 at 9,624/0 — 800 runs at 15,288/132/0/0, and combined with the 120 consumed runs 920 at 29,244/232. The 124 unchanged references authenticate by exact hash identity only (6,056 passed / 124 expected rejections / 700 filtered, 0 reruns; the 25 distinct sources, binaries and logs rehash exact, and none contains the new Q38 symbol).

The initial-return finding is admitted as a genuine candidate-contract lifecycle finding, not a construction error, a production regression, a source-bound whole-row receipt, a semantic catch or an inventory-ordering artifact. The candidate is the accepted continuation-40 candidate (`26987cbc…`, 248,691 B) plus exactly two copies of the 4,430-byte added module (`a65c774d…`) at byte offsets 77,975 and 206,510, giving `bd7dbf94…` (257,551 B); I erased both copies myself and restored the accepted source byte-exactly, and the compiled independent auditor passes on rerun: `fixed_tests=222 candidate_tests=230 added_tests=8 insertion_points=2 whole_byte_restoration=true whole_AST_restoration=true helper_and_fixed_controls_exact=true`. The accepted Q38 helper (`f2e011c0…`) and all 222 fixed control bodies are unchanged, and the eight added controls are four per armed/disabled module. The control first executes the fixed initial-return scene on the same helper and then a fresh episode with the identical sequence plus a transport-only restoration; a second `InputRequired` without a new pause, a success and another error each leave the baseline `(eligible=false, arms=1, hits=0)`, while restoring only `Run.returning` gives `(true,1,1)` in all three armed cases — the valid counterpart stays `(true,1,1)` and the disabled counterparts stay `(false,1,0)` — and the challenge asserts that only the transported map changed by comparing every record the permission reads and the separately enumerated other state before and after the insertion. In the armed helper, `Guard::input_return` (`candidate.rs:486`) consumes pending (`:493`) and removes the transported return (`:494`), and appends the historical `return_records` entry only for the first authentic return (`:499`); `initial_token` (`:506`) reads the restored map (`:512`) and calls `return_authorized` (`:250`), whose current-return conjunct applies only to `ReturnKind::Request` (`:252`), so the `Evaluation` kind still accepts the invalidated historical token; `initial` (`:516`) grants the permission and `hit` (`:223`) records the observed extra hit. The same binary (`d3f544a9…`) reports 227/3/0/0 at eight and one thread with the three armed controls failing at `candidate.rs:1702` (`(true,1,1)` against `(false,1,0)`), the valid and disabled counterparts green, and I reproduced both runs and the exact failing names myself. This boundary is already declared — `core::Run.returning` is a carriers.tsv entry in the `pause_return` group, and all five consumers (`core::Guard::drop`, `core::hit`, `core::Guard::input_return`, `core::initial_token`, `core::initial`) list it — and the finding is frozen without repair: 2,241 files in `initial-return-stop/`, candidate `bd7dbf94…` and binary `d3f544a9…` exact; no nohash or further candidate form ran after the same-binary reproduction. The FLAGGED construction attempt is settled as instrument correction (the first preparation matched `pub fn hit(` at two methods, and the v2 full-signature anchor changed only the script, never the candidate bytes or an expectation); the two FAIL checks (the initial-return lifecycle and the undelivered complete generator) and the two NA checks (standing replay and workspace) and the one deviation are accepted as the stop's consequence, with no subset, placeholder digest or weakened expectation.

Q39 rules: an invalidation of an initial (`Evaluation`) return is recorded in an independent current-initial-return carrier keyed by the Q35 source key — the episode, the origin activation, the owner activation, the occurrence, the actual `InputRequired` return edge and the proof kind — and never inferred from the absence of the token in the transported `Run.returning` map alone; the `initial` permission through `initial_token` and the `hit` consumer authenticate the current carrier together with the immutable Q34 `source_records`/`return_records`, so the fabricated second `InputRequired`, the success and the other error revoke the key durably and restoring, replaying or corrupting the transport cannot revive the authority, while the valid initial return keeps granting and hitting; one-use consumption and retirement stay separate from the immutable records, the Q37 paired-return and Q38 request-return carriers stand unchanged, and no fixed control body or expectation changes.

Continuation 42 is ordered. Begin with the complete zero-unknown class inventory the stops have earned: for every one of the 133 carriers in the 26 groups and the 134 boundaries, classify whether an invalidation exists and whether it is durably recorded in an independent carrier or only in transported state, including the already-repaired Q37 and Q38 carriers as controls, with a zero-unknown statement. Implement Q39 as the independent current-initial-return carrier, and repair in the same pass every further transport-only invalidation the inventory proves, each by the same rule with its own named control and omission calibration; only a genuine contract ambiguity — a carrier with no declared invalidation semantic — stops for a ruling. Preserve the accepted helper, the fixed 222, the Q37/Q38 carriers and every expectation, and do not mutate the immutable Q34 records or repair a frozen tree. Calibrate an omission variant that is red on exactly the three invalidated controls with the valid counterpart and the fixed 222 green, rerun the initial-return lifecycle in hash and nohash at eight and one thread, rerun every affected family matrix in its recorded cfgs, forms, selectors and thread counts, reuse evidence only by exact hash identity, then finish the complete bar: every reachable Q35 carrier and Q34 boundary, the full 134-boundary table and 4,422 obligations, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips at `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0, with the switched same-binary checks. The stop-loss stays the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair.

Budget. The carried 7,150.0300783879675 s is continuation 42's focused envelope; no extension is granted at this gate. CM4's committed share stands at 36,500 s with 7,132.205848635007 s of it unspent, the stage reserve stands at 77,000 s and 743–746 retain their 40,500 s unchanged; the assigned 6,000 s workspace envelope stays untouched for the workspace gates, and the 1,440 s verification envelope and second 6,000 s workspace stay held for the verification stage. If the ordered work exhausts the carried pool, it stops on allocation and the next gate allocates from the stage reserve.

The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344), the continuation-35 `episode-guard-stop/` (832), the continuation-36 `profile-control-stop/` (2,555), the continuation-37 `paired-pause-stop/` (4,751), the continuation-38 `paired-return-stop/` (331), the continuation-39 `request-return-stop/` (479) and the continuation-41 `initial-return-stop/` (2,241) are immutable; I rehashed all 14,578 manifest entries and found zero bad copies or originals, with only the two known open `freeze-stop.log` drifts in the continuation-32 and continuation-33 open trees (their closed copies exact). The maintained surface and production scope stay NONE; no campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

The entry is clean — 5,400 maintained, 590 design and 39,009 accepted files authenticate, `production_changes=0` and `maintained_changes=0`, the three inherited director-document drifts are the c40 gate's committed `TODO.md` (`d6c7e092…` prior → `35d32195…` entry), brief-623 (`2f829a21…` → `b5a41937…`) and brief-742 (`981a88c7…` → `51c572a2…`), the 90/180 pins and 151 rows are exact, `f9604852…` is unchanged before and after with `new_source_schema=null` and `complete_plan_hash=null`, all 94 admitted paths are `not_emitted` with zero maintained byte delta and zero complete postimages, and the 32 pre-existing change instances match their entry hashes (I verified all 44,999 entry hashes myself).

## Continuation 42 ruling (director) — continuation 42 is a compliant Q28 STOP with the ordered class-wide repair delivered: Q39 is implemented as the independent current-initial-return carrier (six registered edits, forward and inverse byte-exact, the 230-control lifecycle green in hash and nohash at eight and one thread, and the carrier-omitting variant red on exactly the three armed invalidated-return controls with the fixed 222 and the valid/disabled counterparts green), and the zero-unknown invalidation inventory v6 classifies all 133 carriers in the 26 groups, the 134 boundaries and the 4,422 declared obligations — 57 durable, 48 transport-only, 20 superseded, 4 immutable and 4 boundary-guarded, with zero unknown classifications and zero declared-semantic ambiguities — whose 13 required repairs are implemented with named controls, omission calibrations, byte inverses and independent AST audits; the 920-run family replay is exact at 29,244 passed / 232 deliberate rejections / 0 ignored / 0 filtered, the 124 remaining raw/paired runs are freshly executed at 6,056 / 124 / 700 filtered, 2,236 test executions total 99,072 / 1,084 / 0 / 700 with 1,070 deliberate rejections, two first-attempt construction failures and twelve candidate instances, and the three partial host/adapter source witnesses are exact — while the final additive actual-source Guard control then exposes `PortBlocked::ProducerGuardTransportImpersonatesLiveSourceEntry` as a genuine candidate-contract finding: the current-entry predicate authenticates the received activation/profile tuple against the live top entry, but no independent actual-source Guard identity ties that tuple to the actual outer object, so substituting another live entry's tuple lets the actual outer Guard issue one construction frame proof, one evaluation-return proof with permission and a hit, and one request-return proof for that entry on the same 314/6 binary at eight and one thread, with the fixed 308 and the six valid/ordinary/unknown controls green; Q40 rules the independent actual-source Guard identity, and continuation 43 implements it, reruns every affected matrix and finishes the complete bar under an 11,956.6271620999205 s focused envelope.

Continuation 42 ran `cdx` (unit `brief-742-cdx-20261117-233901-320698`, rc 0, peak 7,141,314,560 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 15,016,550 B, sha256 `98804ac2…`; report STOP, 24,765 changes — 24,763 added, 2 modified (`PROGRESS.md`, `report.json`), 0 deleted, all under `.temp/brief-742` with zero non-`.temp` change paths — 13 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 11,056 citations verified, 0 questions, 1 deviation, 0 disclosures; 3,193.402916288047 s charged — 1,609.644804730051 s measured over 3,040 charged receipts plus 1,520.0 s bookkeeping, the 60 s preparation/reporting reserve and 3.758111557996017 s closing — of the 7,150.0300783879675 s focused envelope, leaving exactly 3,956.6271620999205 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held; `no_envelope_overrun=true`, `allocation_stop=false`, `candidate_control_stop=true`). The accounting re-derives: the 3,040 charged receipts' measured durations sum to 1,609.644804730051 s exactly, the 1,520.0 s bookkeeping is the 3,040 × 0.5 s parent charge, and the charged total plus the 60 s reserve and the 3.758111557996017 s closing is 3,193.402916288047 s, exactly the reported charge.

The Q39 implementation is accepted. It is exactly six registered edits — the `InitialReturnEdge::ActualInputRequired` and `InitialReturnKey { source: Token, edge }` declarations; the `Pair.current_initial_returns: BTreeMap<InitialReturnKey, bool>` carrier; the `current_initial_return` reader binding the queried owner, the origin episode and the pair episode to the exact key; the `return_authorized` Evaluation conjunct; the `input_return` owner-keyed revocation independent of `Run.returning`; and the issue-time vacancy assert and `(key, true)` insert before the immutable record — each old text unique in the accepted continuation-41 helper (`f2e011c0…`, 77,483 B). I applied the six edits myself: the forward render is byte-identical to the continuation-42 helper (`7d251d08…`, 78,796 B) and the inverse restores the accepted helper byte-exactly. The initial matrix is exact in all eight runs: the candidate is 230/0 at hash and nohash, eight and one thread, and the weakness that replaces the carrier lookup with `&& true` at both copies is 227/3 in every run with exactly `fabricated_second_initial_return_cannot_be_revived`, `other_error_initial_return_cannot_be_revived` and `successful_initial_return_cannot_be_revived` in the armed module, the fixed 222 and the valid/disabled counterparts green; I reproduced the matrix from its result records and the render-plan hashes.

The class inventory and its repairs are accepted. `invalidation-inventory-v6.json` classifies all 133 carriers (26 groups) and associates all 134 boundaries and 4,422 obligations: 57 durable, 48 transport-only, 20 superseded, 4 immutable and 4 boundary-guarded, with zero unknown classifications and zero declared-semantic ambiguities; the 13 required repairs are Q39 plus the frame-retirement consumer, current pending source, episode/pair current authority, evaluation-scope, host-call, lexical-scope, local operation, paired-local, queue-pause, raw construction, raw current source/return and raw local consumption. The repairs are implemented as registered scratch-only edits with named controls, omission calibrations, byte inverses and independent AST erasers; twenty-five audit PASS lines and zero failures cover the initial, source-current, queue-current, raw-current, paired-local, operation-local, lexical, episode, host, adapter and guard-actor families (I reran the guard-actor, initial, episode, source-additions, host and adapter-v3 audit binaries myself and every PASS line repeated), and all fourteen render plans' sources and targets — about 1,300 entries — rehash exact. The source-current overreach is retained honestly: the first weak variant disabled the check in both enable arms and produced 240/4 instead of the declared 241/3 because the disabled consumed-return reached `RepeatedInitialReturnKey`; it is recorded as non-exact and superseded by the corrected calibration, whose weak-pending variant is 241/3 in all four runs with the three armed names and whose weak-frame variant is 242/2 in all four runs with the two named frame controls. The two host files and one adapter file are partial full-source witnesses only, with exact live/partial/old-armed-pin hashes and `complete_overlay=false`, so no complete-arm claim is made.

The 920-run family replay and the 124 remaining runs are accepted. The family plan's 920 labels are exactly the continuation-40 plan's 920 labels, every result matches its plan entry (the target source and sha256, the changed command, the threads and the expected exit, counts and failing names), every record is `expected_exact`, and I rehashed all 3,184 distinct executed sources, binaries and logs (0 bad); the totals are 29,244 passed / 232 deliberate rejections / 0 ignored / 0 filtered. The 124 remaining raw/paired runs keep the continuation-41 labels, are freshly built and executed at 6,056 / 124 / 0 / 700 filtered with every expectation exact. The full accounting is 2,236 test executions at 99,072 passed / 1,084 failed / 0 ignored / 700 filtered, and the 1,084 failures are exactly the 1,070 deliberate calibration rejections, the two resolved first-attempt calibration-construction failures and the twelve genuine final candidate-failure instances (three boundaries in two forms), every one named in `executed-results.json`.

The new finding is admitted as a genuine candidate-contract finding, not a construction error, a production regression, a source-bound whole-row catch or an inventory-ordering artifact. The candidate is the exact tested 308-control core candidate (`9fac2919…`, 343,465 B) plus exactly two copies of the 6,202-byte added module (`a87b9cde…`) at two insertion points, giving `964e8355…` (355,869 B); I erased both copies myself and restored the base byte-exactly, and the independent auditor passes on rerun: `fixed_tests=308 candidate_tests=320 added_tests=12 insertion_points=2 whole_byte_restoration=true whole_AST_restoration=true helper_and_fixed_controls_exact=true source_guard_actors_distinct=true`. The fixture keeps two actual nested Guard values live: the actual inner Guard records one construction frame, one evaluation return with permission and a hit in armed, and one request return; the unmodified actual outer Guard is rejected at all three boundaries, and the unknown-tuple challenge is rejected; but changing only the outer Guard's received tuple to the inner tuple — with every independent source/current record byte-identical through the debug snapshot, no ledger or immutable record touched at the seam, and the tuple restored solely for cleanup — makes the actual outer object write the inner entry's authority: one frame proof for construction, one evaluation-return proof plus permission and one hit in armed, and one request-return proof. The same binary (`b532fbf8…`) reports 314/6/0/0 at eight and one thread with the three `another_live_guard_tuple_cannot_issue_*` controls failing in both armed and disabled forms, and I reproduced both runs and the exact names myself. This is the actual-object complement of the Q34 source-writer boundary already declared by the c31 consumer anchors: the current-entry predicate authenticates the received tuple against the live top entry, but no independent identity binds that tuple to the actual Guard object. The candidate is frozen without repair in `guard-actor-stop/` (9,391 files, originals and copies exact); no nohash or further form ran after the same-binary reproduction. The one FLAGGED construction check is settled as retained instrument work — inventory anchor/hash parsing, the source-calibration overreach, missing fixture helpers, AST eraser and preparation corrections, the five copied parent-driver script-name annotations and the one incorrect finally-status field, all annotated with `original_receipt_unchanged=true` and no control, expectation or standing inventory weakened — and the two FAIL checks (the Guard authority and the undelivered complete bar) and the two NA checks (standing replay and workspace) and the one deviation are accepted as the stop's consequence, with no subset or placeholder digest substituted.

Q40 rules: every Guard method that writes source authority authenticates an independent actual-source Guard identity — minted at and bound to the source entry, carried by the actual Guard object and distinct from the mutable received activation/profile tuple — against the live entry, never the received tuple alone; substituting the received tuple with another live entry's tuple cannot make the actual object issue construction, evaluation-return, request-return, permission or hit authority for that entry; the actual object's own entry keeps working, and the unmodified wrong-owner object, the unknown tuple and the actual inner object stay as the controls require; the immutable Q34 records, the Q37/Q38/Q39 carriers, and every fixed control body and expectation stay unchanged.

Continuation 43 is ordered. Inventory every source-writer and actual-object boundary's identity mechanism — the c31 consumer anchors, all Q34 source writers and the Guard entry/exit/return/hit consumers — with a zero-unknown statement, preserving the fixed 308 and all twelve Guard controls. Implement Q40 as the independent actual-source Guard identity and calibrate an omission variant that is red on exactly the six cross-live-entry controls with the actual inner, unmodified outer and unknown-tuple controls green; rerun the Guard lifecycle in hash and nohash at eight and one thread, rerun every affected family matrix in its recorded cfgs, forms, selectors and thread counts, and reuse evidence only by exact hash identity with every unchanged input. Then finish the complete bar: the complete final-helper calibration integration and every 134-boundary / 4,422 per-form obligation, all 133-carrier execution duties, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips at `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0, with the switched same-binary checks. No frozen tree is repaired, no production or maintained source changes, no expectation, inventory or derived cell is edited; the stop-loss stays the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair.

Budget. The carried 3,956.6271620999205 s plus a fresh 8,000 s focused allocation gives continuation 43 an 11,956.6271620999205 s focused envelope; the assigned 6,000 s workspace envelope stays untouched for the workspace gates, and the 1,440 s verification envelope and second 6,000 s workspace stay held for the verification stage. CM4 has committed 32,561.19706765304 s of its share, so the share is extended by 8,000 s to 44,500 s and the stage reserve from 77,000 s to 85,000 s; 743–746 retain their 40,500 s unchanged. The extension is an allocation from the project's reserve, not a bar change: the bar, the stop-loss and the authorization limits are unchanged.

The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344), the continuation-35 `episode-guard-stop/` (832), the continuation-36 `profile-control-stop/` (2,555), the continuation-37 `paired-pause-stop/` (4,751), the continuation-38 `paired-return-stop/` (331), the continuation-39 `request-return-stop/` (479), the continuation-41 `initial-return-stop/` (2,241) and the continuation-42 `guard-actor-stop/` (9,391) are immutable; I rehashed all 23,969 manifest entries and found zero bad copies or originals, with only the two known open `freeze-stop.log` drifts in the continuation-32 and continuation-33 open trees (their closed copies exact). The maintained surface and production scope stay NONE; no campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

The entry is clean — 5,400 maintained, 590 design and 43,534 accepted files authenticate (I verified all 49,524 entry hashes myself), `production_changes=0` and `maintained_changes=0`, the three inherited director-document drifts are the continuation-41 gate's committed `TODO.md` (`35d32195…` prior → `5988671e…` entry), brief-623 (`b5a41937…` → `1401680f…`) and brief-742 (`51c572a2…` → `76cbe621…`), the 90/180 pins and 151 rows are exact, `f9604852…` is unchanged before and after with `new_source_schema=null` and `complete_plan_hash=null`, all 94 admitted paths are `not_emitted` with zero maintained byte delta and zero complete postimages, and the 32 pre-existing change instances match their entry hashes.

## Continuation 43 ruling (director) — continuation 43 is a compliant Q28 STOP with the ordered actual-object inventory and the Q40 implementation delivered: the independent actual-source Guard identity is implemented as thirteen registered edits (a fresh `Rc<GuardSource>` minted at actual source entry, registered in independent `ScopeAuthority` state, carried by the actual Guard separately from the mutable activation/profile tuple and authenticated by `Rc::ptr_eq` plus full binding at every authority writer, with teardown retirement), the forward render of the accepted 308+12 candidate is byte-exact to `b8ff4c04…` (355,869 → 360,785 B) and the inverse restores the base byte-exactly, the 320-control fixture is green in hash and nohash at eight and one thread, and the omission variant that reduces `guard_source_matches` to `true` at both copies is red on exactly the six cross-live-entry controls in all four runs with the actual inner, unmodified outer and unknown-tuple controls green; the source-object inventory authenticates all 133 carriers in 26 groups, all 134 boundaries and all 4,422 obligations with zero unknowns and classifies the ten core Guard writers as `requires-Q40-Guard-binding` and 124 boundaries as `declared-other-source-or-consumer`; all 106 affected sources in 150 copies re-render through the thirteen edits forward and invert byte-exactly, and the Guard audit and the affected audit both pass on rerun; the 920-run family replay is exact at 29,244 passed / 232 deliberate rejections, the 32 host runs are 1,032/56, the 40 adapter runs are 1,280/80, the 76 integrated lifecycle calibrations are 24,128/192 with all nineteen omission declarations copied exactly from their authenticated prior plans, the eight guard-current runs are 2,536/24, and the 124 unchanged runs authenticate by exact hash identity at 6,056/124/700 with no fresh credit — while the final additive raw Producer control then exposes `PortBlocked::RawProducerTransportImpersonatesLiveSourceEntry` as a genuine candidate-contract finding: the indexed raw `Producer` carries only a received slot index and its `construct`, `input_return` and `input_error` authenticate `index + 1 == entries.len()` against the live top entry without any independent actual-object source identity, so substituting the outer object's received index with the inner object's index makes the actual outer issue one construction frame proof, one Result-return proof and one EvalError-return proof for the inner source, with permission and a hit, on the same 60/3 binary at eight and one thread with the 51 fixed and the nine valid/ordinary/unknown controls green; Q41 rules the independent actual-object source identity for every indexed/owner-bearing source-object mechanism, and continuation 44 implements it class-wide, reruns every affected matrix and finishes the complete bar under the carried 10,221.403497958907 s focused envelope.

Continuation 43 ran `cdx` (unit `brief-742-cdx-20261118-021917-495118`, rc 0, peak 4,018,417,664 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 3,335,744 B, sha256 `6a635a91…`; report STOP, 7,792 changes — 7,790 added, 2 modified (`PROGRESS.md`, `report.json`), 0 deleted, all under `.temp/brief-742` with zero non-`.temp` change paths — 12 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 4,235 citations verified, 0 questions, 1 deviation, 0 disclosures; 1,735.2236641410127 s charged — 942.428635308017 s measured over 1,431 charged receipts plus 715.5 s bookkeeping, the 60 s preparation/reporting reserve and 17.29502883299574 s closing — of the 11,956.6271620999205 s focused envelope, leaving exactly 10,221.403497958907 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held; `no_envelope_overrun=true`, `allocation_stop=false`, `candidate_control_stop=true`). The accounting re-derives: the 1,431 charged receipts' measured durations sum to 942.428635308017 s exactly, the 715.5 s bookkeeping is the 1,431 × 0.5 s parent charge, and the charged total plus the 60 s reserve and the 17.29502883299574 s closing is 1,735.2236641410127 s exactly, the reported charge; the six matrix-runner bundle entries charge only their own overhead plus .5 s and their children's measured seconds separately, so no second is charged twice; all 1,433 ledger receipts exist and match their recorded sha256.

The Q40 implementation is accepted. It is thirteen registered edit roles at two copies — the `GuardSource { owner, profile_step }` declaration with `mint_guard_source`, `guard_source_matches` and `retire_guard_source` helpers, the `ScopeAuthority.guard_sources` map, the empty-Guard constructors, the actual-entry mint, the `Guard(Some(activation), profile_step, Some(source))` carrier, the exit-before-propagation authentication, the drop/defer retirement, the construction authentication, the deferred-transfer authentication, the authenticated deferred edge minting a fresh Guard, the evaluation/request/profile return authentication and the queue `source_scope` authentication — each replacing a unique accepted text at two copies. I applied all thirteen myself to the frozen continuation-42 candidate (`964e8355…`, 355,869 B): every edit's occurrence count equals the plan's count, the forward render is byte-identical to `b8ff4c04…` (360,785 B), and the inverse restores the base byte-exactly. The 160 test functions at two copies — the fixed 308 and the twelve Guard controls — are byte-identical between base and target, and the fixture is green in all four runs at 320 passed; the weakness that neuters the identity predicate at both copies (`af210772…`, 360,137 B) reports 314/6 in all four runs with exactly `another_live_guard_tuple_cannot_issue_{construction,evaluation_return,request_return}` in the armed and disabled modules, which I reproduced at eight and one thread in both cfgs. The independent AST audit passes (`files=1 actual_entry_mint=true independent_live_object_identity=true fixed_controls_exact=true Q34_records_exact=true Q37_Q38_Q39_exact=true all_other_nodes_exact=true`), and no production allocation, field, hook, accessor or `unsafe` is introduced.

The source-object inventory and the affected render are accepted. The inventory authenticates the c31 writer/reader declarations and the 134 boundary ids in exactly the accepted continuation-42 order, with the same anchors, plus the independent identity mechanism, actual object and Q40 duty per boundary: 133 carriers in 26 groups, 134 boundaries, 4,422 associated obligations, zero unknown boundaries and zero unknown source-writer classifications; the ten `requires-Q40-Guard-binding` boundaries are exactly the core Guard writers (`enter`, `Guard::drop`, `Guard::suspending`, `Guard::defer`, `construct_deferred`, `Guard::input_return`, `Guard::request_return`, `Guard::profile_return`, `Guard::suspending_operation`, `enter_selected`) and 124 are `declared-other-source-or-consumer`; the twelve declared source writers, the adapter's fourth return writer and the host/adapter contained-Guard record boundaries are named; all 27 authenticated source files hash exact. The affected render plan covers 106 sources in 150 copies (62 one-copy, 44 two-copy, five with the adapter wrapper conjunct); I applied the thirteen edits and the wrapper edits to all 106 sources with strict occurrence counts and reproduced every target byte-exactly, and the inverse restores every source byte-exactly; the independent affected audit passes (`files=106`) and every source and target hash is in the verified evidence set. The inventory honestly says that zero unknown classifications does not assert complete dynamic execution or a closed continuation graph: `execution_credit=false`, the three raw Producer methods are declared source-writer obligations, and the newly observed identity defect leaves execution closure open.

The matrix replays are accepted. The 920-run family plan keeps exactly the continuation-40 plan's 920 labels and every field — command, cfg, form, selector, threads, expected exit, counts and failing names — and every result matches its plan entry with `expected_exact=true`; the 92 distinct re-rendered targets and the 920 × 3 provenance hashes (the continuation-40 render, the continuation-41 target and the continuation-42 target) are all exact by hash identity, so the edit chain is authenticated, not assumed; the totals are 29,244 passed / 232 deliberate rejections / 0 ignored / 0 filtered. The 32 host runs are 1,032/56, the 40 adapter runs are 1,280/80 and the 76 integrated runs are 24,128/192, every run `expected_exact`; the nineteen integrated omission variants are each rendered from the accepted 320-control candidate at their recorded spans and invert byte-exactly at those spans, and every expected-failure set is copied exactly from its authenticated declaration. The eight guard-current runs are 2,536/24 (candidate 320/0 and weakness 314/6, each at hash and nohash, eight and one). I verified all 3,065 execution evidence files and all 1,433 receipts by hash, and I reproduced the green and red samples myself: family `guard-family-test-0001` 214/0, family `guard-family-test-0109` 227/3 with exactly its three `armed::loop_identity_controls` names, host 34/0, adapter 34/0 and the integrated `weak-current-initial` variant 317/3 with exactly its three armed initial-return names. The fresh total is 1,078 executions at 58,340 passed / 590 failed / 0 ignored / 0 filtered, and the 590 failures are exactly the 584 deliberate calibration rejections and the six raw candidate instances. The 124 unchanged runs keep the continuation-42 labels in order and authenticate at 6,056 / 124 / 0 / 700 filtered across 461 files with `execution_in_c43=false` and every recorded original input exact; they earn no fresh or unfiltered-workspace credit.

The new finding is admitted as a genuine candidate-contract finding, not a production regression, a construction error, a whole-row catch or an inventory-ordering artifact. The candidate is the accepted continuation-42 raw helper (`06031a84…`, 57,090 B, 51 tests) plus exactly one copy of the 4,868-byte added module (`f4923d23…`) followed by one newline at the single insertion point, giving `df110194…` (61,959 B, 63 tests) with twelve added tests and no fixed test removed; I erased the module plus newline myself and restored the base byte-exactly, and the independent auditor passes (`fixed_tests=51 candidate_tests=63 added_tests=12 insertion_points=1 whole_AST_restoration=true helper_and_fixed_controls_exact=true`). The declared boundary is `raw::Producer::construct`, `raw::Producer::input_return` and `raw::Producer::input_error`: `Producer` carries only `index: Option<usize>`, and the three methods authenticate that received index as the current top entry (`index + 1 == entries.len()`) without any independent actual-object source identity. The fixture keeps the actual inner and the actual outer raw Producer values live together; the actual inner creates one valid construction or return proof and, in armed, one permission and one hit; the unmodified outer and the unknown index are rejected at all three boundaries; but changing only the outer's received index to the inner index — with the independent entry, pending, source, return, construction and current state byte-identical at the injection seam, asserted by a full debug snapshot, and the index restored solely for orderly destructor cleanup, with no independent record restored or repaired — makes the actual outer object issue the inner source's authority: one construction frame proof with permission and a hit (`issued=1 frame_proofs=1 permission=true hits=1`), one Result-return proof with permission and a hit (`issued=1 return_proofs=1 permission=true hits=1`) and one EvalError-return proof with permission and a hit, in all three modes. The same binary (`d6095fff…`) reports 60 passed / 3 failed / 0 ignored / 0 filtered at eight and one thread with exactly `another_live_source_index_cannot_{construct,return_result,return_error}` failing, the 51 fixed and the nine actual-inner/ordinary-outer/unknown-index controls green; I reproduced both thread counts and the exact names myself. This is the raw source-object complement of Q40: the received index is authenticated against the live entry, but nothing binds the actual object to its own entry. The candidate is frozen without repair in `raw-actor-stop/` (3,573 files, originals and copies exact); no nohash or disabled form ran after the same-binary reproduction. The one FLAGGED construction check is settled as retained instrument work — two inventory parsing attempts (`inventory.log`, `inventory-v2.log`) and one affected-render shape attempt (`affected-render.log`) failed before compilation or at a shape assert, their originals are retained, and the later exact inventories and renders pass with no expectation, control or standing inventory weakened — and the two FAIL checks (the raw source object and the undelivered complete bar) and the two NA checks (standing replay and workspace) and the one deviation are accepted as the stop's consequence, with no subset or placeholder digest substituted.

Q41 rules: every method of every declared source-object mechanism that writes source authority — the raw `Producer`'s `construct`, `input_return` and `input_error`, every indexed or owner-bearing source-object mechanism the inventory declares now or later, and the paired Producer/Permit writers — authenticates an independent actual-object source identity, minted at and bound to the source entry, carried by the actual object and distinct from the mutable received index or tuple, against the live entry, never the received index alone; substituting the received index with another live entry's index cannot make the actual object issue construction, Result-return, EvalError-return, permission or hit authority for that entry; the actual object's own entry keeps working, and the unmodified other object, the unknown index and the actual inner object stay as the controls require; the immutable Q34 records, the Q37/Q38/Q39 carriers, the Q40 Guard identity and every fixed control body and expectation stay unchanged. A mechanism whose existing binding is only inventory evidence — the paired queued-invocation/permit binding is one — carries an open obligation until a cross-live-entry control runs it.

Continuation 44 is ordered. Inventory every declared source-object mechanism's identity mechanism — the raw `Producer`, the paired `Producer`/`Permit` and every other independently declared mechanism among the 134 boundaries — with a zero-unknown statement, preserving the fixed 308 and the twelve Guard controls and the 51 raw controls with their anchors. Implement Q41 class-wide in the same pass, with a named control per mechanism at each source-writer boundary (construction, Result-return and EvalError-return for the raw Producer, and the paired writers in their declared shapes), and calibrate an omission variant that is red on exactly the cross-live-entry controls of each mechanism with the actual inner, unmodified other-object and unknown-index controls green; rerun the Guard lifecycle and the raw lifecycle in hash and nohash at eight and one thread, rerun every affected family matrix in its recorded cfgs, forms, selectors and thread counts, and reuse evidence only by exact hash identity with every unchanged input. Then finish the complete bar: the complete final-helper calibration integration and every 134-boundary / 4,422 per-form obligation, all 133-carrier execution duties, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips at `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0, with the switched same-binary checks. No frozen tree is repaired, no production or maintained source changes, no expectation, inventory or derived cell is edited; the stop-loss stays the Q28 counter, and a failing compiled candidate control stops after same-binary one-thread reproduction and is frozen without repair.

Budget. The carried 10,221.403497958907 s is continuation 44's focused envelope with no extension: CM4 has committed 34,296.42073179405 s of its 44,500 s share, the assigned 6,000 s workspace envelope stays untouched for the workspace gates, and the 1,440 s verification envelope and second 6,000 s workspace stay held for the verification stage; the stage reserve stays 85,000 s and 743–746 retain their 40,500 s unchanged.

The frozen `join-queue-stop/` (235), `interleaved-join-stop/` (574), both continuation-32 trees (826 each), both continuation-33 trees (292 each), the continuation-34 `baseline-contract-stop/` (344), the continuation-35 `episode-guard-stop/` (832), the continuation-36 `profile-control-stop/` (2,555), the continuation-37 `paired-pause-stop/` (4,751), the continuation-38 `paired-return-stop/` (331), the continuation-39 `request-return-stop/` (479), the continuation-41 `initial-return-stop/` (2,241), the continuation-42 `guard-actor-stop/` (9,391) and the continuation-43 `raw-actor-stop/` (3,573) are immutable; I rehashed all 27,542 manifest entries and found zero bad copies or originals, with only the two known open `freeze-stop.log` drifts in the continuation-32 and continuation-33 open trees (their closed copies exact). The maintained surface and production scope stay NONE; no campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

The entry is clean — 5,400 maintained, 590 design and 65,254 accepted files authenticate (I verified all 71,244 entry hashes myself), `production_changes=0` and `maintained_changes=0`, the three inherited director-document drifts are the continuation-42 gate's committed `TODO.md` (`d52a1e68…`), brief-623 (`ab8a454e…`) and brief-742 (`45b3f1b2…`), `f9604852…` is unchanged before and after with `new_source_schema=null` and `complete_plan_hash=null`, all 94 admitted paths are `not_emitted` with zero maintained byte delta and zero complete postimages, every one of the 690 changed `.rs` files is enumerated in the 696 fragment byte deltas with exact before/after hashes and `complete_overlay=false`, and the 32 pre-existing change instances match their entry hashes.

## Continuation 44 ruling (director) — continuation 44 is a compliant calibration-declaration STOP with Q41 delivered: the independent actual-object source identity is implemented for both remaining declared source-object mechanisms (the raw `Producer`'s four writers and the paired `Producer`/`Permit` writers, against a fresh `Rc<RawSource>`/`Rc<EvalSource>` minted at the actual source entry, stored on the entry and carried by the actual object, authenticated by pointer identity plus full binding before every authority write), the raw and paired valid candidates are green in both cfgs at eight and one thread with the identity-neutered variants red on exactly the cross-live-entry controls, all 27 affected sources in 38 copies render forward and invert byte-exactly, the 148 affected matrix runs and the 1,044 hash-identity reuse are exact, and the complete final-helper calibration stops on the worker's own complete-fixture declaration error: the guard weak-2 omission reports 332 passed / 4 failed against a declared 334 / 2 because the declaration was copied from a `guard_only+guard_2` build that excluded two `guard_3`-gated veto controls which participate in the complete 336-control fixture and are forced to fail by the unchanged control bodies. The worker stopped before normalizing any expectation, froze 852 files without repair and returned the remaining allocation; the director gate settles the complete-fixture declaration and orders continuation 45 to derive, record and execute the whole 42-variant calibration against the complete fixture before finishing the complete bar.

Continuation 44 ran `cdx` (unit `brief-742-cdx-20261118-031644-588887`, rc 0, peak 1,507,065,856 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 2,695,025 B, sha256 `6e54907bf355f71c…`; report STOP, 1,912 changes — 1,910 added, 2 modified (`.temp/brief-742/PROGRESS.md`, `.temp/brief-742/report.json`), 0 deleted, all under `.temp/brief-742` with zero non-`.temp` change paths — 12 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 1,424 citations verified, 0 questions, 2 deviations, 0 disclosures, 0 measurements; 594.8012415540143 s charged — 375.7438516900147 s measured over 314 charged receipts plus 157.0 s bookkeeping, the 60 s preparation/reporting reserve and 2.0573898639995605 s closing over two closing receipts — of the carried 10,221.403497958907 s, leaving exactly 9,626.602256404893 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held; `no_envelope_overrun=true`, `allocation_stop=false`, `candidate_control_stop=false`, `calibration_declaration_stop=true`). The accounting re-derives exactly: the 314 charged receipts sum to 375.7438516900147 s, the bookkeeping is the 314 × 0.5 s parent charge, the two closing receipts sum to 1.0573898639995605 s plus 1.0 s bookkeeping, and the total plus the 60 s reserve is exactly 594.8012415540143 s; the five matrix bundles charge their own overhead only (48, 186, 30, 6 and 15 child commands, child seconds already charged), and no child is double-charged.

Q41 is delivered and accepted for the raw `Producer`. Six registered edits at the one copy replace the accepted continuation-43 raw helper (`df110194…`, 61,959 B) with `937bbd9f…` (63,034 B): a fresh `Rc<RawSource>` (owner activation) is minted at the actual source entry, stored on the `Entry` separately from the mutable received index and carried by the actual `Producer`; `raw_source_matches` authenticates the index into the live entry, `index.checked_add(1) == Some(entries.len())`, the activation, the episode and `Rc::ptr_eq(&entry.source, source)`; and the check runs before construction, before the Result-return, before the EvalError-return and before the teardown pop, each beside the unchanged top-entry assertion. I applied all six edits myself with every occurrence count equal to the plan, reproduced the target byte-exactly and inverted it byte-exactly. The 63 controls (the fixed 51 plus the twelve continuation-43 actual-object controls) are 63 / 0 / 0 / 0 in the armed and the disabled form, in hash and nohash, at eight and one thread (eight runs), and the identity-neutering variant `23844c29…` (62,650 B) is 60 / 3 / 0 / 0 in all eight runs with exactly `another_live_source_index_cannot_{construct,return_result,return_error}` failing; the actual inner, ordinary outer and unknown-index controls stay green. I reran all 32 Q41 runs with the exact recorded commands and environments (including `RUM_CM740_OMIT=C3.evaluate_call.unstable_head` where recorded) and reproduced every count and failure set; the recorded armed-hash binary `d55d7e35…` rebuilds byte-exactly from the recorded absolute-path command, and the independent AST auditor restores the pre-edit AST from approved literal nodes rather than trusting the renderer plan (`fixed_tests=63`, `all_other_nodes_exact=true`, PASS).

Q41 is delivered and accepted for the paired `Producer` and `Permit`. Thirteen registered edits at two copies replace the accepted continuation-39 paired fixture (`9a6dfb43…`) with `bf259b97…`: a fresh `Rc<EvalSource>` (owner) is minted at the actual queued or ordinary entry, stored on the `EvalEntry` independently and carried by the actual `Producer` and, at issuance, by the `Permit`; `paired_source_matches` authenticates the owner, the live active stack and the last entry's owner plus `Rc::ptr_eq`, and it gates the return writer, eligibility, operation issuance, the teardown pop and (through `paired_permit_source_matches`) the omission and hit writers, while the free paired hit obtains the actual current entry identity rather than a fabricated one. I applied all thirteen edits with the exact per-copy counts, reproduced the target byte-exactly and inverted it byte-exactly. The 48 added controls carry a named control at each of the six declared writers (return, eligibility, operation, producer hit, permit omission, permit hit) for four actors (actual inner, unmodified ordinary outer, unknown owner, and the cross-live substitution that changes only the received owner at the injection seam while every independent entry/source/current record stays byte-identical), in both copies; the 110-control fixture is 110 / 0 / 0 / 0 in hash and nohash at eight and one thread, and the identity-neutering variant `8942316a…` (54,077 B) is 102 / 8 / 0 / 0 in all four runs with exactly six armed cross-live-entry writers and the disabled return/operation proof writers failing, while the disabled omission/hit controls stay inactive as required and the actual-inner, ordinary-outer and unknown-owner controls stay green. The recorded binary `b1d234c9…` rebuilds byte-exactly from the recorded command, and the independent AST auditor passes (`fixed_tests=62 added_tests=48`, `renderer_plan_is_not_authority=true`).

The inventory is zero-unknown and preserves the accepted records exactly. It authenticates the 133 carriers in 26 groups, the 134 boundaries in the accepted continuation-42 order, the 4,422 associated obligations, the 12 declared source writers, the 27 authenticated sources, and the three additional declared host/adapter contained-Guard boundaries, with zero unknown boundaries and zero unknown source-writer classifications; the continuation-43 declarations (`c31_writer_reader_declarations`, `source_writers`, `additional_active_source_boundaries`, `source_writer_shape_counts`, `limitations`) compare equal as records. It names four source-object mechanisms with zero unknown actual-object mechanisms: the core Guard (ten boundaries, Q40 `Rc<GuardSource>` plus independent `ScopeAuthority` binding), the raw `Producer` (seven boundaries: `enter`, `enter_at`, `pause`, `construct`, `input_return`, `input_error`, `drop`), the paired `Producer` (ten boundaries including `pause_actual` and the free `hit`) and the paired `Permit` (the omission and hit writers); the fixed counts are 308 + 12 Guard, 51 + 12 raw and 62 paired. The inventory states `q41_classification_only=true`, `q41_full_execution=false` and `execution_credit=false`; it is not a closed continuation graph and credits no work.

The affected render is exact in both directions. All 27 affected sources in 38 copies (ten raw at one copy, 17 paired at two copies; 16 one-copy and 11 two-copy entries) carry exact pre-edit and target hashes, and I applied every edit with the exact per-copy occurrence counts, reproduced every target byte-exactly and inverted every source byte-exactly, including the retained legacy missing-precheck omission shape; the independent affected AST auditor re-verified all 27 with the renderer plan excluded as authority. The 24 affected-family runs (360 / 0 / 0 / 0) keep their original cfgs, forms, selectors and threads.

The fresh matrices and the reuse are exact. The 32 Q41 runs are 4,368 passed / 80 deliberate rejections / 0 ignored / 0 filtered; the 124 remaining raw/paired runs are 6,056 / 124 / 700 filtered with their original 700 filtered selections; the 24 affected-family runs are 360 / 0; the four 336-control final-core runs are 1,344 / 0; the ten executed final-calibration runs are 3,336 / 24. The fresh total is 194 executions at 15,464 passed / 228 failed / 0 ignored / 700 filtered — 192 exact and the two weak-2 runs mismatched — where the 228 failures are 224 declared omission rejections plus the four additional weak-2 rejections, every one in a deliberately weakened variant, with zero valid-candidate failure instances. All 3,842 execution-evidence files, all 194 executions' source/binary/log/receipt/build hashes, all 316 ledger entries and receipts, and all 46 artifacts authenticate. The 1,044 reused runs (896 family, 32 host, 40 adapter, 76 integrated lifecycle) authenticate by exact source, binary, build-command, test-command, receipt and log hashes across 3,144 files at 55,324 passed / 560 deliberate rejections / 0 ignored / 0 filtered, every record marks `execution_in_c44=false`, no reused source contains either Q41 symbol and none lies under the continuation-44 path; the reuse therefore earns no fresh or workspace credit. The final-core fixture composes the accepted 320-control candidate (`b8ff4c04…`, 360,785 B) with the byte-exact extracted loop module (`loop_controls.rs.in`, 4,566 B) at two insertion points (each with its newline), yielding `779a3654…` (369,919 B) with 336 tests = the fixed 320 + 16 loop controls; erasing the additions restores the accepted candidate byte-exactly, and the addition audit passes (`fixed_tests=320 total_tests=336 whole_AST_restoration=true`). Every candidate and weak binary I rebuilt from its recorded exact command (absolute input path included) reproduces byte-identically: raw `d55d7e35…`, raw weak `92623d2e…`, paired `b1d234c9…`, paired weak `f997dd17…`, guard `b653ea55…`, final-core `69f258de…`, weak-2 `33d6dc2e…` and the affected-family build `7c9f6bad…`.

The stop is a genuine worker calibration-integration declaration error, not a candidate-contract finding. The weak-2 variant `cba02adc…` applies the registered omission at its two spans (I inverted both spans to the 336 candidate byte-exactly): `saved` becomes `received.clone()` and `invalid_present` becomes `false`. Its prior declaration came from a `--cfg guard_only --cfg guard_2` build of `affected-v2/014.rs`, where the frozen two `resume_checks_independent_record_before_restoring_owner` controls participate but the `invalid_present_resume_is_explicit_veto` controls (gated `#[cfg(any(not(guard_only), guard_3))]`) do not. In the complete fixture both controls participate in both copies, and the unchanged bodies force them to fail: `saved = received.clone()` restores the unauthenticated received token so `assert!(!restored, "C36_RESUME_AUTH")` fails, and `invalid_present = false` means no `EvaluationProvenance::InvalidPresent` is pushed so `assert!(veto, "C36_INVALID_PRESENT")` fails. The observed 332 / 4 at eight and one thread is exactly the two declared names plus the armed and disabled veto names; the valid 336-control candidate is 336 / 0 in all four runs; no expectation, control, omission, candidate byte or prior declaration was edited after the mismatch; the same-binary evidence and its 852 frozen files (originals and copies) authenticate; 10 of the 168 planned cfg/thread runs executed (eight exact, two mismatched) and 158 were not run; no nohash weak-2 and no later variant ran. The two FAIL (final calibration, complete bar) and two NA (standing replay, workspace) and both deviations are accepted as the stop's consequence, and nothing not run is credited.

The FLAGGED construction check is settled as retained instrument work. Three failed preparation attempts are kept: the first entry authentication used the repository root for a continuation-43-relative artifact manifest (exit 1), the first propagation stopped on a legacy missing-precheck calibration shape before compilation (exit 1), and the first extraction-auditor build selected `syn`/`quote` without span locations (exit 1, E0599); each original is retained, the corrected follow-ups passed, and no compiled candidate was repaired, no fixed control or standing inventory was weakened, and the later calibration error is a separate FAIL.

The ruling on the class. (1) Q41 is accepted as the campaign's requirement for every declared source-object mechanism: an independent actual-object source identity minted at and bound to the source entry, carried by the actual object itself and never by the mutable received index, owner or tuple, authenticated by pointer identity plus full binding before every authority write. The raw and paired implementations and their three-name and eight-name falsifiers are accepted; Q40 and the immutable Q34/Q37/Q38/Q39 records stand unchanged. (2) The complete-fixture calibration declaration is a function of the complete 336-control participation and the registered omission, never of a cfg-restricted prior run: for every one of the 42 variants the expected failure set must be re-derived and recorded before execution, each name carrying a causal anchor (the control's body, the omitted span and the assertion it violates), and every participating control not in the set classified as not-forced with its reason; the guard weak-2 complete-fixture set is the four names — the two `resume_checks_independent_record_before_restoring_owner` names and the armed and disabled `invalid_present_resume_is_explicit_veto` names — and this is a declaration correction, not an expectation edit: the continuation-44 plan stays frozen, the corrected plan is a new artifact that must be written and hashed before the runs, and no control body, omission span, candidate byte or prior declaration may change. (3) The stop-loss remains the Q28 counter: a failing compiled candidate control stops after same-binary one-thread reproduction; a calibration run whose observed set differs from its derived set also stops, and neither a missing nor an extra failure may be absorbed by rewriting the expectation.

Continuation 45 is ordered. First build the complete-fixture calibration participation inventory for all 42 variants with zero unknowns: for each variant, the participating controls under the complete fixture, the controls that reference the omitted predicate, the forced expected failures with their causal anchors, and every remaining participating control classified as not-forced with its reason; record the derivation in a new plan with exact source, span, declaration and control anchors, freeze it before any calibration run, and retain the continuation-44 plan and the 10-run mismatch evidence frozen. Then execute all 168 planned cfg/thread runs (the 42 variants in hash and nohash at eight and one thread) and require every observed count and failure set to equal its derived set exactly, with the valid 336-control candidate green in all four runs and no control, omission or candidate edit; then finish the complete bar: every 134-boundary / 4,422 per-form obligation, all 133-carrier execution duties, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips at `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0, with the switched same-binary checks. Reuse evidence only by exact hash identity with every unchanged input. No frozen tree is repaired, no production or maintained source changes, and no expectation, inventory or derived cell is edited outside the new, separately frozen derivation plan; the stop-loss is the Q28 counter and the cross-live-entry controls of every mechanism stay green.

Budget. Continuation 44 charged 594.8012415540143 s of the carried 10,221.403497958907 s, leaving 9,626.602256404893 s, which is continuation 45's focused envelope with no extension: CM4 has committed 34,891.22197334806 s of its 44,500 s share, the assigned 6,000 s workspace envelope stays untouched for the workspace gates, and the 1,440 s verification envelope and second 6,000 s workspace stay held for the verification stage; the stage reserve stays 85,000 s and 743–746 retain their 40,500 s unchanged.

The fifteen earlier frozen trees (`join-queue-stop/` 235, `interleaved-join-stop/` 574, both continuation-32 trees 826 each, both continuation-33 trees 292 each, the continuation-34 `baseline-contract-stop/` 344, the continuation-35 `episode-guard-stop/` 832, the continuation-36 `profile-control-stop/` 2,555, the continuation-37 `paired-pause-stop/` 4,751, the continuation-38 `paired-return-stop/` 331, the continuation-39 `request-return-stop/` 479, the continuation-41 `initial-return-stop/` 2,241, the continuation-42 `guard-actor-stop/` 9,391 and the continuation-43 `raw-actor-stop/` 3,573) are immutable; I rehashed all 27,542 entries myself and found zero bad copies or originals, with only the two known open `freeze-stop.log` drifts in the continuation-32 and continuation-33 open trees (their closed copies exact); the new continuation-44 `calibration-integration-stop/` tree (852 files) is frozen with originals and copies exact. The maintained surface and production scope stay NONE; no campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

The entry is clean. The 5,400 maintained, 590 design and 73,043 accepted files authenticate — I verified all 79,033 hashes myself — `production_changes=0` and `maintained_changes=0`; against the continuation-43 entry the maintained set is unchanged except exactly the three documents the continuation-43 gate committed (`TODO.md`, brief-623, brief-742), the design set is unchanged, and the accepted set grows by exactly the 7,789 continuation-43 scratch files its accepted category admits (the continuation-43 report file itself lies outside that category); the three inherited director-document drifts are recorded with their prior and entry hashes and match the entry; `f9604852…` is unchanged before and after with `new_source_schema=null` and `complete_plan_hash=null`; all 94 admitted paths are `not_emitted` with zero maintained byte delta and zero complete postimages and their entry hashes match; the 174 scratch Rust fragments carry exact before/after hashes and 164 unique content-addressed diffs (87 zero-delta copies) with `complete_overlay=false`; and the 32 pre-existing change instances match their entry hashes.

## Continuation 45 ruling (director) — continuation 45 is a compliant Q28 candidate-control STOP that delivers the complete 42-variant / 168-run calibration against the complete 336-control fixture and then admits the next candidate-contract finding at the queue installation writer: every variant's expected set was derived from the complete participation with causal anchors and frozen before execution, all 168 runs match their derived counts and exact failure names (56,076 passed / 372 deliberate rejections / 0 ignored / 0 filtered) with the valid 336-control candidate green in all four runs, and the four variants whose omissions the complete fixture cannot force (guard weak-6, operation-record, operation-call, operation-ticket) are disclosed as non-firing limits with no falsifier credit; the added InstallGuard controls then show that `queue::InstallGuard::generation` authenticates only the received scope number, so the actual outer guard carrying the live inner guard's scope id writes `Some(7)` into the inner installation entry while every independent install, source and queue record stays fixed — 6 passed / 2 failed at eight and one thread on the same hash binary, the actual inner, ordinary outer and unknown-scope controls green. The two FAIL and two NA and both deviations are the stop's consequence; the complete bar is not delivered and nothing unexecuted is credited.

Continuation 45 ran `cdx` (unit `brief-742-cdx-20261118-041645-643700`, rc 0, peak 1,792,929,792 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 2,903,802 B, sha256 `d69817f865418e08…`; report STOP, 1,389 changes — 1,387 added, 2 modified (`.temp/brief-742/PROGRESS.md`, `.temp/brief-742/report.json`), 0 deleted, all under `.temp/brief-742` with zero non-`.temp` change paths — 10 PASS / 2 FLAGGED / 2 FAIL / 2 NA, 975 citations verified, 0 questions, 2 deviations, 0 disclosures, 0 measurements; 676.6896332529868 s charged — 470.26873774799606 s recorded over 279 charged receipts (the 1.0 s conservative preflight included), 139.5 s bookkeeping, the 60 s preparation/reporting reserve and 6.920895504990767 s closing over two closing receipts — of the carried 9,626.602256404893 s, leaving exactly 8,949.912623151906 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held; `no_envelope_overrun=true`, `allocation_stop=false`, `candidate_control_stop=true`, `calibration_declaration_stop=false`). The accounting re-derives exactly: the 279 charged receipts sum to 470.26873774799606 s including the 1.0 s preflight charge, the bookkeeping is 279 × 0.5 s, the two closing receipts sum to 5.920895504990767 s plus 1.0 s bookkeeping, and the total plus the 60 s reserve is exactly 676.6896332529868 s; the one matrix bundle charges its own 17.354734481996275 s of parent overhead while its 258 child commands' 207.91693873900658 s are already charged exactly once, and no child is double-charged.

The complete-fixture derivation is accepted. The new plan covers all 42 variants over the complete 336 participating controls in both cfg modes (the `guard_only` cfg is absent), with 14,112 participation cells — every control of every variant classified `forced` or `not-forced`, zero unknown cells — and 93 forced cells in total, each carrying its control body, omitted span anchor and the assertion it violates; every not-forced cell carries its reason and local helper anchors. The reference relation is recorded as conservative (receiver/destructor expansion, not a production continuation graph or an execution proof), and `execution_credit=false` is stated. The plan was written at 04:27:31.215 and its freeze at 04:27:31.243, the calibration AST audit ran at 04:28:28 and the first candidate build at 04:28:44 — the plan and its 99 recorded input/output artifacts (plan `9cbeac29…`, freeze `adae4cee…`, both rehashed by me) precede every execution. The derivation changed exactly five variants against the frozen continuation-44 plan and left the other 37 unchanged: weak-2 gains the armed and disabled `invalid_present_resume_is_explicit_veto` names (2 → 4), while guard weak-6 (2 → 0), operation-record (1 → 0), operation-call (2 → 0) and operation-ticket (1 → 0) are newly non-firing. The four zero sets are genuine fixture limits, not expectation edits: the historical `current_token_rechecks_construction_at_its_boundary` control is absent from the 336 (I confirmed zero occurrences in the base) and the retained `lexical_current` precheck still rejects the `every_handoff_checks_independent_construction_again` corruption; `another_valid_operation_record_is_not_authority` changes only `pa.operation` and is rejected by the independent `LocalPermitRecord.operation` comparison in `permit_matches`; `callee_checks_the_actual_call_record` is rejected by the preceding local permit comparison; and `call_ticket_is_checked_again_before_omit` is rejected by the independent local permit ticket identity. No expectation, control body, omission span, candidate byte or prior declaration was edited; the continuation-44 plan (`f0da3b98…`) and its ten-run mismatch evidence stay frozen.

The 168-run calibration matrix is exact. The four valid-candidate runs are 336 / 0 / 0 / 0 (1,344 passed in total); the 42 variants × 2 cfgs × 8/1 threads are 56,076 passed / 372 deliberate rejections / 0 ignored / 0 filtered, exactly 93 forced names × 4 runs, with zero declaration mismatches. I re-derived all 172 calibration and candidate runs from their logs myself: every recorded source, binary and log hash is exact, every printed count matches its plan cell, and every failure name equals its derived set — zero discrepancies. The 42 variants each invert byte-exactly to the accepted 336-control base `779a3654…` at their registered spans (I replayed all 42 inverse renders), and the independent calibration AST audit confirms all 336 test bodies and attributes across the variants. All 645 execution-evidence files for the 172 runs authenticate, and I rebuilt the valid, weak-2 (hash and nohash, both `33d6dc2e…`) and install binaries byte-identically from their recorded absolute-path commands. The four non-firing variants passed 336/0 in all four runs exactly as derived; they earn no falsifier-execution credit, and the mainline may not count them as 42 firing calibrations.

The InstallGuard finding is admitted. The boundary is the already-declared C29 `OBJECT_AUTHORITY` consumer `queue::InstallGuard::generation` (forms P0267, P0312, P0323, P1470 and P1471, all form-1), whose production form writes `State.installing.last_mut()` after `assert_eq!(install.scope, self.0)` — the received scope number is the only authentication. The eight added controls (four actors × two copies) preserve all 336 existing test bodies and attributes and every helper byte; the candidate `6dff3001…` (380,723 B) is the accepted base `779a3654…` (369,919 B) plus the 5,401-byte module at both insertion points with its newline (I re-rendered and inverted both byte-exactly). The injection changes only the actual outer guard's received `self.0` to the live inner guard's scope while the independent installing entries, owner activations, source records, active scopes and queue proof status stay byte-identical at the seam; `generation` then accepts the outer object and writes `Some(7)` into the inner installation entry, the actual inner writes its own `(None, Some(7))`, and the ordinary outer and unknown scope are rejected before mutation. I rebuilt the hash binary myself (`ae0fc97e…`, byte-identical) and ran it with the recorded `RUM_CM740_OMIT=P0267` and filter at eight and one thread: 6 passed / 2 failed / 0 ignored / 336 filtered in both, the two failing controls are exactly `armed::queue_transport::install_actual_source_controls::another_live_scope_does_not_authorize_outer_install_guard` and its disabled copy, and the printed observation is `rejected=false, observed=(None, Some(7))` with `independent_source_exact=true`. No nohash form, no other declared form and no later candidate work ran; the 671-file `install-source-stop/` tree (originals and copies) and its 35 artifact hashes authenticate, and the same-binary record binds the t8 log, the t1 log's candidate, the plan and the binary. The stop is compliant: Q28 requires same-binary one-thread reproduction, freeze without repair, and a return to the director, and the worker did exactly that.

The class ruling (Q42). The continuation-44 inventory classified `queue::InstallGuard::generation` and `queue::InstallGuard::drop` as `declared-other-source-or-consumer` with `q41_role=declared-source-route-or-transport-consumer`, and justified the classification by the independence of the install/pause/queue records. That justification is insufficient: the records are independent, but the *write authorization* is the mutable received scope number, so the boundary is an owner-bearing source-object mechanism under Q41's own class text, not a transport consumer. The classification rule is corrected: a boundary may be classified as a consumer only when none of its writers writes authority (installation, owner, source, queue or proof state) under a received value that a different live object could carry; every writer that does is in the Q41/Q42 actual-object class regardless of the earlier label. Concretely, every declared installation or scope writer — `installing()`, `InstallGuard::generation`, `InstallGuard::drop`, `registered(...)` and every related declared scope writer — must mint an independent installation identity at `installing()`, store it in the installation entry, carry it in the guard object and authenticate it by pointer/equality plus the full entry binding (entry identity, owner, scope, generation and namespace state) before any generation write, namespace registration, drop/completion or retirement, with teardown retirement; the received scope id, any transported copy of it, and the ambient `last()` entry are never authority. Substituting the received scope with another live guard's scope cannot let one live guard write, register, complete or retire another live installation; the actual inner keeps its authority; the ordinary outer, unknown scope, retired/stale identity and cross-live substitutions are rejected before mutation; `registered` must stop being a free function that writes the top entry without an object identity (its duty moves, and every caller must be listed and updated); the Q34 records, the Q37/Q38/Q39 carriers, the Q40/Q41 identities and every fixed body and expectation stay unchanged.

The non-firing limits are accepted as limits. The four variants are recorded in the plan with their historical declarations, their newly derived empty sets, the named masking checks and `calibration_execution_used=false`; the matrix executes them and observes 336/0 four times each. The final bar report must state 38 firing variants plus these four masked limits with their masking checks, and may not claim 42 firing calibrations; no dedicated control is ordered for them in this line, and their historical declarations stay frozen.

Continuation 46 is ordered. First re-open the classification: over the 134 boundaries and 4,422 obligations, re-classify every boundary previously recorded as `declared-other-source-or-consumer` (the continuation-43 124 plus any continuation-44 addition) by whether its writers write installation, owner, source, queue or proof authority under a mutable received value, with zero unknown classifications and a named reason for every consumer classification; record the corrected inventory as a new artifact and keep the continuation-43 and continuation-44 inventories frozen. Then implement Q42 class-wide in the complete fixture: the independent installation identity at `installing()`, carried by the guard and authenticated at generation, registration, drop/completion and every related declared scope writer, with the actual-inner, ordinary-outer, unknown-scope, another-live-scope and retired/stale-identity controls in both copies, an omission calibration that is red on exactly the cross-live controls with every other control green, the byte inverse and the independent AST audit, and a new separately frozen plan. Then re-render the 42 calibration variants from the new accepted base, re-derive the complete participation including the new installation controls (the expected set of every variant is a function of the new complete control set and the registered omission, with causal anchors, not of the continuation-45 plan), freeze, and execute the full cfg/thread matrix with exact agreement and zero declaration mismatches. Then finish the complete bar: every 134-boundary / 4,422 per-form obligation (including all five InstallGuard forms in hash and nohash at eight and one), all 133-carrier execution duties, the complete 151-plan generator with zero unknown edges and duplicate/overlap/unknown-anchor rejection, two fresh roots with exactly 94 targets each and exactly 188 postimages with byte identity, the byte and AST restoration and omission audits, the mechanical six-column 94-row schema with all and only the 151 `source_schema` cells and the exact-byte digest, the ordered standing replay, `cargo fmt --check`, and both workspace modes with the seven skips at `RUST_TEST_THREADS=8` and the unfiltered `suspension_oracle` and `typed_slot_census` at failed=0 and filtered=0, with the switched same-binary checks. Reuse evidence only by exact hash identity with every unchanged input. No frozen tree is repaired, no production or maintained source changes, no still-existing expectation, inventory or derived cell is edited outside the new separately frozen plans, and the stop-loss is the Q28 counter.

Budget. Continuation 45 charged 676.6896332529868 s of the carried 9,626.602256404893 s, leaving 8,949.912623151906 s, which is continuation 46's focused envelope with no extension: CM4 has committed 35,567.9116066010468 s of its 44,500 s share, the assigned 6,000 s workspace envelope stays untouched for the workspace gates, and the 1,440 s verification envelope and second 6,000 s workspace stay held for the verification stage; the stage reserve stays 85,000 s and 743–746 retain their 40,500 s unchanged.

The fifteen earlier frozen trees and the continuation-44 tree are immutable: I rehashed all 29,065 prior manifest entries myself with zero missing files and exactly the two known open `freeze-stop.log` drifts in the continuation-32 and continuation-33 open trees (their closed copies exact), and all 29,065 frozen copies are exact; the new continuation-45 `install-source-stop/` tree (671 files) is frozen with originals and copies exact and no repair. The maintained surface and production scope stay NONE; no campaign, epoch, acceptance or CM5 is authorized. Pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked.

The entry is clean. The 5,400 maintained, 590 design and 74,952 accepted files authenticate — I verified all 80,942 hashes myself — `production_changes=0` and `maintained_changes=0`; against the continuation-44 entry the maintained set is unchanged except exactly the three documents the continuation-44 gate committed (`TODO.md`, brief-623, brief-742), the design set is unchanged, and the accepted set grows by exactly the 1,909 continuation-44 scratch files its accepted category admits (the continuation-44 report copy itself lies outside that category); the three inherited director-document drifts are recorded with their prior and entry hashes and match the entry; `f9604852…` is unchanged before and after with `new_source_schema=null` and `complete_plan_hash=null`; all 94 admitted paths are `not_emitted` with zero maintained byte delta and zero complete postimages and their entry hashes match; the six scratch Rust fragment deltas carry exact before/after hashes and content-addressed diffs (five zero-delta frozen copies/new files and the additive install module) with `complete_overlay=false`; and the pre-existing change layer still matches the tree exactly.

## Continuation 46 ruling (director) — continuation 46 is a compliant Q28 fixture-composition STOP that delivers the Q42 reclassification and the installation component, then exposes the complete fixture's selected-episode-row conflict: the complete fixture's inherited C3 selector makes both installation owners None, so the eight frozen continuation-45 controls fail their distinct-owner setup assertion before any injection — reproduced at one thread on the same binary and frozen without repair. Q43 rules that every added control scene derives its episode row from the fixture's own selection; the complete bar is not delivered and nothing unexecuted is credited.

Continuation 46 ran `cdx` (unit `brief-742-cdx-20261118-050537-718672`, rc 0, peak 785,780,736 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 1,572,570 B, sha256 `4908b33f9bbcd2ad…`; report STOP, 314 changes — 312 added, 2 modified (`.temp/brief-742/PROGRESS.md`, `.temp/brief-742/report.json`), 0 deleted, all under `.temp/brief-742`, the only change paths outside `continuation-46` being those two report artifacts and `report-continuation-46.json` — 10 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 495 citations, 0 questions, 2 deviations, 0 disclosures; 320.208300333019 s charged — 238.8621355760115 s over 40 charged receipts including the 1.0 s conservative preflight, 20.0 s bookkeeping, the fixed 60 s reserve and 1.3461647570075002 s closing — of the carried 8,949.912623151906 s, leaving exactly 8,629.704322818887 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held; `no_envelope_overrun=true`, `allocation_stop=false`, `candidate_control_stop=true`, `calibration_declaration_stop=false`). The two FAIL (`complete-fixture-contract`, `complete-bar`) and two NA (standing replay, workspace) and both deviations are the stop's consequence; the one FLAGGED construction check is settled as retained instrument work (two inventory name mappings, the syn `visit-mut` feature, an AST trailing-comma parse and the E0040 explicit-destructor helpers, all corrected before the first test execution with the 374 test bodies AST-exact).

The Q42 reclassification is accepted. The new `source-object-inventory.json` keeps all 134 boundary identities in the accepted order, all 4,422 obligation indices contiguous and partitioned (every index 0–4,421 exactly once), 133 carriers in 26 groups, and classifies the 124 formerly `declared-other-source-or-consumer` boundaries with zero unknown labels: 23 individually justified read-only consumers, 55 actual-object writers (the 26 explicit scope mechanisms, the ten accepted Q40/Q41 core Guard writers and the further indexed-source mechanisms) and 56 conservatively classified source-or-proof writers; the continuation-43 and continuation-44 inventories stay frozen and this inventory claims no execution credit (`execution_credit=false`), no absence proof and no dynamic closure; the ten core Guard boundaries that carried `requires-Q40-Guard-binding` map to actual-object writers, the four InstallGuard boundaries and `queue::registered` are now in the authority class, and the historical `queue::current_episode` is mapped to the current read inside `authenticate` with no frozen id change. I verified the 23 consumer bodies myself: their call lists contain no insert/push/pop/remove/retain/clear/set call and the only `= Some` hits are local bindings, so the read-only declarations are consistent with the code.

The installation component is accepted. Eight registered edit roles at two copies mint a fresh `Rc<InstallationIdentity>` at `installing()` — scope, owner, current flag and the generation/namespace cells — store it on the `Installing` entry, carry it in the actual `InstallGuard`, and authenticate before every write by `Rc::ptr_eq` plus the full entry binding (scope, owner, generation, namespace and both current flags); `generation` and the real destructor pop authenticate before mutation, `registered` moves from the free top-entry function to the authenticated `InstallGuard` method, the destructor retires the identity, and the two fixture install callers carry the guard. I re-rendered the eight edits forward from the accepted continuation-45 candidate (`6dff3001…`, 380,723 B) to `ac2ff1e1…` (384,333 B) and inverted them byte-exactly, and composed the tested complete candidate `705b1c49…` (399,895 B) as that source plus exactly two copies of the 7,780-byte control module, each followed by a newline; the frozen continuation-45 module appears exactly twice and byte-identically, the independent AST eraser passes (`fixed_tests=344 added_tests=30 copies=2 whole_AST_restoration=true full_entry_binding=true actual_identity_mint=true generation_registration_completion_before_write=true Q34_Q37_Q38_Q39_Q40_Q41_exact=true renderer_plan_is_not_authority=true`), and the fixture-control AST audit passes with all 374 test bodies and attributes exact across the E0040 helper correction. The compiler correction is confined to the added fixture helpers (safe owned drop in an isolated TLS scene); no fixed test body, expectation, arming rule or assertion was edited, the first uncompiled E0040 candidate and its declaration stay frozen, and the v2 declaration is separate.

The component executions are exact and I reproduced them. The P0267 component is 38 / 0 / 0 / 336 in hash and nohash at eight and one (152 passed in total), and the pointer-omission variant is 30 / 8 / 0 / 336 in all four runs with exactly the eight declared cross-live names (the armed and disabled `another_live_scope_does_not_authorize_outer_install_guard` plus `generation`, `registration` and `completion` `_cross_live_scope`), every other component control green; the weak source differs from the valid candidate in exactly two lines (`installation_object_matches` returns `true` instead of `Rc::ptr_eq`), so the omission is the pointer association alone and the full-binding checks still reject the unknown-scope, ordinary-outer and retired-identity actors. I rebuilt both binaries byte-identically from their recorded absolute-path rustc commands (`807c733a…` valid, `e466f3b5…` weak; a relative-path rebuild embeds a different source path and differs only for that reason) and reran the component at eight and one myself: 38 / 0 / 0 / 336 and 30 / 8 / 0 / 336 with the same names. The hash and nohash builds are necessarily byte-identical because this fixture has no `feature="hash"`-gated site (zero occurrences in the candidate); the four runs are real executions but the two cfg arms are not independent configurations, and that instrument fact is recorded with no separate credit.

The fixture-composition finding is admitted. The complete 374-control fixture under the inherited `C3.evaluate_call.unstable_head` selector (the continuation-45 calibration authority, `run_calibrations.py` line 56) reports 366 passed / 8 failed / 0 ignored / 0 filtered at eight and one thread on the same hash binary `807c733a…` (source `705b1c49…`), and the eight failures are the unchanged continuation-45 installation controls (all four actors in both the armed and disabled copies). Every one fails the fixed distinct-owner setup assertion `assert_ne!(s.installing[0].owner, s.installing[1].owner)` at the module's second copy (candidate line 6,120) with `left: None, right: None` — before the received-scope injection and before any generation, registration or completion write. The cause is the fixture's single selected episode row: `begin` stamps the episode row from `RUM_CM740_OMIT` (C3 under the complete fixture), `installing` resolves its owner through `producer_owner` against that row, and `producer_owner` rejects any arming scope whose episode row differs from the requested row; the frozen continuation-45 scene enters `P0267`, so neither installation gets an owner. The P0267-filtered component is green, the 336 fixed controls and the 30 added controls are green, and no identity failure is inferred from this mismatch. I reproduced the complete fixture at eight and one with my own rebuild: 366 / 8 / 0 / 0 each, the same eight names. The 129-file `installation-selector-stop/` tree (originals and copies) is frozen with no repair, no selector, fixed body, expectation, arming rule or independent record changed, and no complete nohash run or later variant, generator or workspace command followed.

The class ruling (Q43). The fixture has exactly one selected episode row — the value `begin` reads — and the installation owner chain (`installing` → `producer_owner`) resolves against it; an added control's scene row is therefore not a free choice. Q43 rules: every added control scene derives the row it enters from the fixture's own selection, and no added control may enter a hardcoded program row; a control set whose scene enters a row other than the selection is component-scoped only, cannot compose with the complete fixture, and its red in the complete fixture is not a candidate-contract finding; the frozen continuation-45 and continuation-46 control modules remain frozen evidence and are not edited; an adapted control set with identical actors, boundaries, assertions, injections and test bodies, differing only in the row derivation, is minted and registered in a new separately frozen plan before execution, with an independent AST audit proving that the frozen module differs only at the row derivation and that the distinct-owner setup assertion is preserved; the adapted set replaces the frozen modules in the new complete fixture, which composes the 336 fixed controls, the Q42 implementation and the adapted 8 and 30 controls (374 total) and must be green under the standard C3 selector in both cfg arms at eight and one, with the pointer-omission variant red on exactly its declared cross-live controls; and the five declared InstallGuard forms (`P0267/form-1`, `P0312/form-1`, `P0323/form-1`, `P1470/form-1` and `P1471/form-1`) are executed by the adapted component under each form's selector in both cfg arms at eight and one, with the valid set green, the omission calibration red on exactly the cross-live controls in each run, and the actual-inner, ordinary-outer, unknown-scope and retired-identity controls green. The complete-fixture composition and the five per-form executions are separate duties; neither substitutes for the other. The continuation-45 finding stands, and its repair is validated by the component pair: the same continuation-45 cross-live control is green with the Q42 identity and red on the pointer omission at both thread counts.

The four masked calibration limits stand unchanged: guard weak-6 is masked by the retained `lexical_current` precheck and lacks its historical dedicated control, operation-record by `LocalPermitRecord.operation`, operation-call by the preceding local permit comparison and operation-ticket by the independent local permit ticket identity; they keep `calibration_execution_used=false` and no falsifier credit, and the final bar must state 38 firing variants plus these four masked limits.

Continuation 47 is ordered. First, mint and separately freeze the adapted control set under Q43 as above, and execute the complete 374-control fixture under C3 in hash and nohash at eight and one (374 / 0 each) and the five form selections in hash and nohash at eight and one (valid 38 / 0; omission 30 / 8 with exactly the cross-live names per run). Then complete Q42 class-wide: the 22 remaining scope mechanisms (PassGuard and `evaluation_owner`; JoinOwner, `join_owner`, `join_result` and its destructor; JoinAppend, `join_append`, `join_queue` and its destructor; ConstructionGuard; ResumeGuard, `resume_frame` and its destructor; DeferredEvaluationGuard; ProfileConstruction, `profile_construct` and its destructor; ProfileFinish, `profile_finish` and its destructor; OperationConstruction; OperationCallScope; the raw Construction, RawResume and CallScope destructors; and the host CM740OutputScope), each minting an independent actual-object identity at its source entry, carrying it in the actual object, authenticating by pointer/equality plus full binding before every write and retiring at teardown, with named controls for every actor × writer, an omission calibration red on exactly the cross-live controls, byte inverses and an independent AST audit; and the source-route registration caller migration, so the copied `profile_queue::register_owner` observation reaches the actual installing guard in `begin_input_suspension` with no ambient write authority and no new runtime accessor, every caller listed. Then re-render the 42 variants from the new accepted base and re-derive the complete participation on the new complete control set (42 × 374 = 15,708 cells, a causal anchor per expected name and a not-forced classification per remaining participant, zero unknown), freeze it before execution and run all 168 runs in both cfg arms at eight and one with exact agreement. Then finish the complete bar: 134 boundaries / 4,422 per-form obligations including all five InstallGuard forms in hash and nohash at eight and one, the 133 carrier duties, the 151-plan generator with duplicate/overlap/unknown-anchor rejection, two fresh 94-target / 188-postimage roots, the byte/AST/omission audits, the six-column 94-row schema with all 151 `source_schema` cells, the ordered standing replay, `cargo fmt --check`, both workspace modes with the seven skips at eight threads and the unfiltered `suspension_oracle` / `typed_slot_census` at failed=0 and filtered=0, and the switched same-binary checks; reuse only by exact hash identity and repair no frozen tree, production or maintained source.

The budget is honest. Continuation 46 charged 320.208300333019 s — the 40 charged receipts sum to 238.8621355760115 s exactly (including the 1.0 s conservative preflight), the bookkeeping is 40 × 0.5 s, the closing receipts sum to 0.3461647570075 s with their 2 × 0.5 s bookkeeping inside the fixed 60 s reserve, and the reserve and closing total 1.3461647570075002 s — of the carried 8,949.912623151906 s, leaving exactly 8,629.704322818887 s, which is continuation 47's focused envelope with no extension. CM4's share stays 44,500 s with 35,888.1199069340658 s committed; the stage reserve stays 85,000 s, the assigned 6,000 s workspace envelope stays untouched, the 1,440 s verification envelope and second 6,000 s workspace stay held, and 743–746 retain their 40,500 s unchanged.

The frozen trees and entry are clean. The fifteen earlier trees plus the continuation-44 and continuation-45 trees rehash 29,065 entries with zero bad and exactly the two known open `freeze-stop.log` drifts in the continuation-32 and continuation-33 open trees (their closed copies exact), all 29,065 frozen copies are exact, and the new 129-file `installation-selector-stop/` tree is frozen with originals and copies exact and no repair. The 5,400 maintained, 590 design and 76,338 accepted files authenticate — I verified all 82,328 hashes myself — `production_changes=0` and `maintained_changes=0`, and the maintained file set equals the entry exactly (no extra or missing file under `crates`, `docs`, `scripts` or the top level); against the continuation-45 entry the maintained set is unchanged except exactly the three documents the continuation-45 gate committed (`TODO.md`, brief-623, brief-742), the design set is unchanged, and the accepted set grows by exactly the 1,386 continuation-45 scratch files its accepted category admits with no removal; the three inherited director-document drifts are recorded with their prior and entry hashes and match the entry; `f9604852…` is unchanged before and after with `new_source_schema=null` and `complete_plan_hash=null`, all 151 `source_schema` cells and the 90 preimages / 180 armed and disabled postimages authenticate against their pinned files; all 94 admitted paths are `not_emitted` with zero maintained byte delta and zero complete postimages and their entry hashes match; the 22 scratch Rust fragment deltas carry exact before/after hashes and 22 content-addressed diffs (11 non-zero and 11 zero-delta copies); the 42 ledger receipts, the 83 evidence files and the 45 artifacts authenticate; and the pre-existing change layer still matches the tree exactly (HEAD `db411620`, the same 23 porcelain entries / 32 expanded instances).

Scope and status stay. No campaign, epoch, acceptance or CM5 is authorized; pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 47 as ordered above, then the verification stage (1,440 s + held second 6,000 s workspace), 743–746 and CM5.

## Continuation 47 ruling (director) — continuation 47 is a compliant Q28 calibration-declaration STOP that delivers Q43's selected-row adaptation and the complete 374-control installation matrix, plus the first actual-object scope components for the core/queue, raw and host families, then exposes a frozen host omission declaration that contradicts the actual accepted source: the retained Q40 `guard_source_matches` inside `HostCall::record` rejects the cross-live substitution before the first host write, so the two `call_cross_live_scope` controls stay green and only the four conversion/output names fire — 60/4 against a declared 58/6, reproduced at eight and one on the same binary and frozen without repair. The six-name declaration is a declaration error, not a candidate-contract or production finding; the corrected set is derived from the actual check order, not rewritten after the run. The complete bar remains open and nothing unexecuted is credited.

Continuation 47 ran `cdx` (unit `brief-742-cdx-20261118-054724-732752`, rc 0, peak 911,224,832 B, oom 0 / oom_kill 0; log `.temp/logs/cdx_brief_742.log`, 3,197,746 B, sha256 `8e2ef99a4cb7cb0d59023fd1f935c83c29452ca182a84444b635408b983b0bae`; report STOP, 932 changes — 930 added, 2 modified (`.temp/brief-742/PROGRESS.md`, `.temp/brief-742/report.json`), 0 deleted, all under `.temp/brief-742` — 12 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 3,784 citations, 0 questions, 2 deviations, 0 disclosures; 440.89446853600384 s charged — 301.74423872899933 s over 155 charged receipts, 77.5 s bookkeeping, the fixed 60 s reserve and 1.6502298070045072 s closing — of the carried 8,629.704322818887 s, leaving exactly 8,188.809854282883 s; workspace 0 of 6,000 s; the 1,440 s verification envelope and second 6,000 s workspace stay held; `no_envelope_overrun=true`, `allocation_stop=false`, `candidate_control_stop=false`, `calibration_declaration_stop=true`). The two FAIL (`host-calibration-declaration`, `complete-bar`), the two NA (standing replay, workspace) and both deviations are the stop's consequence; the one FLAGGED check is settled as retained instrument work (the first Q43 auditor link reused a syn artifact without `visit-mut` and failed E0432; the same auditor source links with the existing featureful artifacts and passes, the failed command and log are retained, and no dependency, manifest or lock changed).

Q43 is delivered and accepted. The adapted modules `install_source_controls.rs.in` (`7a973afd6ce54b7d…`, from the frozen continuation-45 `e5446b25ff0f7710…`) and `install_controls_v2.rs.in` (`9b5033328cb99f85…`, from the frozen continuation-46 `8b948e2628f660a8…`) each differ from their originals by exactly one line: `const ROW: &str = "P0267";` becomes `let ROW: &'static str = super::super::SELECTED.with(|row| &*Box::leak(row.clone().into_boxed_str()));`, the very thread-local `begin` reads; the complete candidate `f3856b96e75c5c21…` (400,195 B) is the accepted base `705b1c4923138af2…` with four copies of that derivation (two modules at two insertion points) and no hardcoded `P0267` program row, and the completion adds exactly the 300 bytes the four longer declarations account for. The plan and its `selection-freeze.json` were written at 05:51:31 and the first candidate build ran at 05:52:03, so the adapted set was frozen before execution as Q43 requires. The independent whole-AST audit passes: `test_bodies=374 attributes_exact=true row_derivations=4 whole_AST_restoration=true distinct_owner_setup_assertion_preserved=true actors_boundaries_injections_exact=true renderer_plan_is_not_authority=true`; the frozen continuation-45 and continuation-46 modules are untouched in their own trees. The complete C3 fixture is 374 / 0 / 0 / 0 in hash and nohash at eight and one, and the pointer-omission variant is 366 / 8 / 0 / 0 in all four runs with exactly the eight declared installation names (the armed and disabled `another_live_scope_does_not_authorize_outer_install_guard` plus `generation`, `registration` and `completion` `_cross_live_scope`); each of the five declared forms (`P0267/form-1`, `P0312/form-1`, `P0323/form-1`, `P1470/form-1`, `P1471/form-1`) is 38 / 0 / 0 / 336 valid and 30 / 8 / 0 / 336 weak in both cfg arms at eight and one. The 48 runs total 4,320 passed / 192 deliberate rejections / 0 ignored / 13,440 filtered and every observed set equals its declared set. I rebuilt the selection binaries byte-identically (`14f443a72d3f5d46…` valid, `17800de027175475…` weak) from the recorded absolute-path commands and reran the complete fixture myself at eight and one: 374/0 and 366/8 with the same eight names. The complete fixture and the five per-form runs remain separate duties and neither substitutes for the other.

The core/queue scope component is accepted at its declared partial scope. Forty-eight registered edits at two copies render forward and invert byte-exactly; the fixed 374 contracts stay 374 / 0 in hash and nohash at eight and one; the 120 additive controls (12 writer scenes × five actors × two copies: ConstructionGuard, ResumeGuard, DeferredEvaluationGuard, ProfileConstruction, ProfileFinish, OperationConstruction, OperationCallScope, PassGuard, JoinOwner and JoinAppend) are 494 / 0 / 0 / 0 in four runs and the pointer-omission variant is 470 / 24 / 0 / 0 in four runs with exactly the declared 24 names (seven core and five queue cross-live names per copy); the additive AST audit restores the 374 base; `join_result` and `join_queue` now require the actual guard and their fixture callers pass it, and the OperationPermit call setup is extracted verbatim into a private instrumentation factory shared by the ordinary call and the controls. Safe `ManuallyDrop` storage allows rejection before any contained-guard teardown while a successful teardown still drops the original guard. I rebuilt `6b8088bb67d9c99e…` and `300dcbcbdcbba616…` byte-identically and reproduced 494/0 and 470/24 at eight and one. The full independent scope AST audit, the affected-source propagation and the per-form closure remain open and are not credited.

The raw scope component is accepted at its declared partial scope. Three objects (Construction, RawResume, CallScope) in 13 registered edits render forward and invert byte-exactly; the fixed 63 controls are 63 / 0 in armed and disabled at hash and nohash, eight and one (eight draws), the 15 additive controls give 78 / 0 in all eight draws and the pointer omission gives 75 / 3 in all eight draws with exactly `call_cross_live_scope`, `construction_cross_live_scope` and `resume_cross_live_scope`; RawResume keeps its source-pinned non-stack own-id removal and its ended-episode cleanup, so an ordinary outer object may retire itself while leaving the inner live. I rebuilt `e68e05fbf5a53e5c…` and `ad31a65cb686b322…` byte-identically and reproduced 78/0 and 75/3 at eight and one. The full scope AST, affected-source and per-form duties remain open.

The host scope component is accepted at its declared partial scope. Three current objects (HostCall, ConversionScope and OutputScope) in 65 registered edits, with five callers absent from the fixture, render and invert byte-exactly; the historical CM740OutputScope boundary id is retained and explicitly mapped to the accepted C43 HostCall/ConversionScope/OutputScope route; the fixed 34 controls are 34 / 0 in four runs and the 30 additive controls are 64 / 0 in four runs. I rebuilt `ed15030a4d381aec…` byte-identically and reproduced 64/0 at eight and one. The full scope AST audit and the complete route closure remain open, and then the omission calibration stops.

The finding is admitted and is a declaration error. The frozen host plan declares six failures — `call`, `conversion` and `output` `_cross_live_scope` in both the armed and disabled copies — at 58 / 6. The hash weak binary instead reports 60 passed / 4 failed / 0 ignored / 0 filtered at eight and one on the same binary `1b3873583f504ae6…` (source `f0fc19f1ef225e68…`), the four failures being exactly the conversion and output cross-live names in both copies; the two call cross-live controls stay green with `rejected=true unchanged=true inner_current=true`, and no unexpected failure name appears. No nohash weak run or later candidate command ran; the 417-file `host-scope-calibration-stop/` tree (originals and copies) is frozen with no repair, its open freeze log and pending wrapper receipt excluded, and the six-name expectation, every control body and every candidate byte remain unchanged. I reproduced the stop at eight and one with my own rebuild of the same binary (60/4, the same four names) and verified the diagnosis in the source: `HostCall::record` checks `self.source` against the object registry, the record's object/type, `guard.0` against the record activation, `guard.1` against the record profile, and then the retained Q40 `guard_source_matches`; `HostCall::drop` calls `record` before `active_calls.pop` and `live_calls.remove`, and `guard_source_matches` binds `source.owner == guard.0`, `source.profile_step == guard.1` and `Rc::ptr_eq(scope_authority.guard_sources[owner], guard.2)`. The cross-live actor substitutes `guard.0` and `guard.1` but keeps `guard.2`, the outer `GuardSource` Rc, so the retained check looks up the inner owner and finds the inner source, fails the pointer equality and rejects before any host write even when the Q42 pointer predicate is omitted. The conversion and output scopes have no such earlier guard-source check, so their identities are the ones the omission falsifies. The declaration's error is that it placed the retained Q40 rejection after the host mutation; the accepted source puts it inside `record`, before the first mutation.

The ruling. The corrected host derivation is a function of the actual accepted source's check order, never of the frozen six-name expectation: the weak omission's expected failure set is exactly the four `conversion` and `output` cross-live names in the armed and disabled copies (60 / 4), and the two `call_cross_live_scope` controls are classified as masked by the retained Q40 `guard_source_matches` before the first host write, with the causal anchor the `guard.2` pointer-equality check on the inner owner lookup. The two call names are not-forced and earn no falsifier credit; the host-call actual-object identity stays required by Q42 class-wide (it also carries the retirement check in `HostCall::drop`), so the masking is an instrument limit, not an exemption, not a candidate-contract finding and not a production regression. A new, separately frozen corrected declaration plan must record, before any execution, a causal anchor for each of the four expected names and a not-forced classification for every remaining actor, including both call cross-live names; the original six-name plan and the 417-file scene stay frozen, and no control, source or expectation is edited. The host weak calibration is then rerun in hash and nohash at eight and one requiring exactly 60 / 4 with those four names, the valid 64 / 0 stays unchanged, and the two call controls stay green and are recorded as the masked pair. This is not silently admitted as a new empty expectation and it does not change the four inherited masked limits. The final bar must state 38 firing variants plus the four masked limits (guard weak-6, operation-record, operation-call and operation-ticket), the host-call calibration masked with two names and no credit, and the Q42 component firing names: core/queue 24, raw 3, host conversion/output 4. No partial component is counted as a complete overlay, an affected-source closure or a per-form matrix.

Continuation 48 is ordered. First, mint and separately freeze the corrected host declaration, rerun the host weak and valid in both cfg arms at eight and one with the exact 60 / 4, and finish the host route closure. Then complete Q42 class-wide: the full independent scope AST and omission audits and the affected-source propagation/per-form matrices for the core/queue, raw and host families, every affected source copy enumerated, and every caller of the moved duties listed; and the source-route registration caller migration, so the copied `profile_queue::register_owner` observation reaches the actual installing guard in `begin_input_suspension`, with no ambient write authority, no new runtime accessor, and every caller listed. Then re-render the 42 variants from the new accepted base and re-derive the complete participation on the new complete 374-control set (42 × 374 = 15,708 cells, a causal anchor per expected name and a not-forced classification per remaining participant, zero unknown), freeze it before execution, and run all 168 runs in both cfg arms at eight and one with exact agreement. Then finish the complete bar: 134 boundaries / 4,422 per-form obligations including all five InstallGuard forms in hash and nohash at eight and one, the 133 carrier duties, the 151-plan generator with duplicate/overlap/unknown-anchor rejection, two fresh 94-target / 188-postimage roots, the byte/AST/omission audits, the six-column 94-row schema with all 151 `source_schema` cells, the ordered standing replay, `cargo fmt --check`, both workspace modes with the seven skips at eight threads and the unfiltered `suspension_oracle` / `typed_slot_census` at failed=0 and filtered=0, and the switched same-binary checks. Reuse only by exact hash identity; repair no frozen tree, production or maintained source.

The budget is honest. Continuation 47 charged 440.89446853600384 s — the 155 charged receipts sum to 301.74423872899933 s exactly, the bookkeeping is 155 × 0.5 s, the closing receipts measure 0.6502298070045071 s with 1.0 s bookkeeping, so the reserve and closing total 1.6502298070045072 s beyond the recorded sum — of the carried 8,629.704322818887 s, leaving exactly 8,188.809854282883 s, which is continuation 48's focused envelope with no extension. CM4's share stays 44,500 s with 36,329.01437547007 s committed; the stage reserve stays 85,000 s, the assigned 6,000 s workspace envelope stays untouched, the 1,440 s verification envelope and second 6,000 s workspace stay held, and 743–746 retain their 40,500 s unchanged.

The surfaces are preserved. All 932 report changes are under `.temp/brief-742`; the maintained 5,400, design 590 and accepted 76,649 sets authenticate — I rehashed all 5,400 maintained and 590 design files with zero differences, the three inherited document drifts are exactly `TODO.md`, brief-623 and brief-742 at their committed hashes, `production_changes=0` and `maintained_changes=0`. The fifteen earlier frozen trees plus the continuation-44, continuation-45 and continuation-46 trees rehash 29,194 entries with zero bad and exactly the two known open `freeze-stop.log` drifts (c32 and c33; all closed copies exact), and the new 417-entry tree I rehashed in full with originals and copies exact. `f9604852…` is unchanged with `new_source_schema=null` and `complete_plan_hash=null`, all 90 preimages / 180 postimages and the 151 `source_schema` cells authenticate, all 94 admitted paths are `not_emitted` with zero maintained byte delta and zero postimages, the 58 scratch Rust fragments carry exact before/after hashes and 58 content-addressed diffs, and the 155 charged plus two closing receipts, 342 evidence files and artifact hashes authenticate. The pre-existing change layer is unchanged (HEAD `a5afef60`, the same 23 porcelain entries / 32 expanded instances). No campaign, epoch, acceptance or CM5; pair `(45,5,false)` Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 48, then the verification stage, 743–746 and CM5.

## Continuation 48 ruling (director) — continuation 48 is a worker execution-protocol STOP that still delivers and freezes the corrected host declaration, the host route closure, the full scope AST/omission audits, the source-route migration's static evidence, the current matrices and the complete 42-variant calibration, while five `CARGO_TARGET_DIR` commands and every dependent run earn no credit and the complete bar remains unexecuted.

Status: **STOP accepted as a worker execution-protocol violation.** The
corrected host declaration, the host route closure, the full scope
AST/omission audits, the source-route migration's static evidence, the
current matrices and the complete 42-variant calibration are accepted as
verified. The five `CARGO_TARGET_DIR` commands and every dependent run earn
no cargo, parity, workspace, generator or fixture credit, and the optional
full-runtime baseline is admitted as a worker setup/declaration error with no
gate meaning. The complete bar remains unexecuted and uncounted.

### 1. Run record

`cdx` unit `brief-742-cdx-20261118-064444-769402`, rc 0, peak
6,425,288,704 B, oom 0 / oom_kill 0; log 14,382,599 B, sha256
`6790905747b76c4734de62f9ecaee00188a217d9d0f6bc457999a7738638521c`. Report
STOP, 56,021 changes — 56,019 added, 2 modified (`PROGRESS.md`,
`report.json`), 0 deleted, every change under `.temp/brief-742` — 12 PASS /
2 FLAGGED / 3 FAIL / 2 NA, 392,371 citations verified, 0 questions, 3
deviations, 0 disclosures. `allocation_stop=false`,
`candidate_control_stop=false`, `calibration_declaration_stop=false`,
`worker_execution_protocol_stop=true`, `no_envelope_overrun=true`.

The three FAIL are `execution-protocol` (worker), `optional-full-runtime-baseline`
(worker setup/declaration) and `complete-bar` (not delivered); the two NA are
`standing-replay` and `workspace-gates` (not run); the two FLAGGED are the
retained construction attempts and the unadmitted cargo evidence, both
settled as instrument/protocol facts with no credit. All 56,021 changes are
under `.temp/brief-742`; the only non-report changes in the tree are the
pre-existing 32 expanded instances of the unchanged dirty layer (23
porcelain entries), and HEAD stays `81987da0`.

### 2. The protocol violation is admitted as a worker execution error

The brief's standing-replay and workspace section says: "Use the exact
DRAFT-COMMON.md commands and script mirror procedure" (brief-742, line 2706),
and the adopted contract at `.temp/brief-747/continuation-1/DRAFT-COMMON.md`
(sha256 `f2e9a57c39e02349420995e10661c47919f3b1d510aa6a6ae9401e2b8cb44551`,
line 34) says exactly: "No CARGO_TARGET_DIR or codegen/optimization
changes." The worker set `CARGO_TARGET_DIR` in five cargo commands:

- `route-real-source-typecheck-default` (cargo check, exit 0)
- `route-real-source-typecheck-nohash` (cargo check, exit 0)
- `runtime-routes-live-default-build` (cargo test build, exit 0)
- `runtime-routes-disabled-default-build` (cargo test build, exit 0)
- `runtime-routes-fresh-disabled-default-build` (cargo test build, exit 0)

I verified the rule text and hash, and I scanned all 4,722 receipts: exactly
five commands contain `CARGO_TARGET_DIR`, and no other command does. The
worker stopped when it recognized the violation, froze the scene and claimed
nothing. The 27,451-file / 11,080,763,517-byte `protocol-stop/` tree freezes
the originals and copies, excluding only its own then-open freeze log and
pending wrapper receipt; I rehashed every entry and both sides: zero
mismatches.

Ruling: the five commands and every dependent unit run — the runtime-routes
live/disabled list and 35-test runs, the fresh disabled build and its runs,
and the optional full baselines — receive no admitted cargo, parity, workspace,
generator or fixture credit. Their costs stay charged and their logs stay
recorded. This is not a candidate-contract finding, not allocation
exhaustion, not an inventory artifact, and not a regression. It is a worker
execution error.

### 3. The optional full-runtime baseline is admitted as a setup/declaration error

The optional unchanged-live full-unit attempt declared 1440 passed / 0
failed / 0 ignored / 0 filtered. The same unchanged live binary
`c555b648f476acf264404acb309609572e83e19b469360be04d4f9368a201d92`
reports 1430 passed / 5 failed / 5 ignored / 0 filtered at eight and one
thread. Three failures are the missing copied `scripts/run-bounded.py` in
the retention, option-read-phase and option-reentry compile controls; two
are the already explicitly excluded `lp655_root_context_falsifier` and
`lp656_public_root_context_falsifier`. The full disabled candidate never
ran; the first claimed disabled binary reused the live Cargo artifact and is
uncredited; the later distinct disabled binary
`60c7475b16cc10b9859d61354f2b1ee556418e7f2f255752486705c4f1cc882f`
ran 35 selected tests at 8/1 but under the prohibited environment, so those
results are also uncredited.

No expected set, fixture, control, declaration or frozen scene was repaired
after the failure; the incorrect declaration stays frozen and wrong. This is
not a candidate regression and not a stale-fingerprint/position ordering
artifact. The full disabled suite remains unexecuted.

### 4. Accepted: the corrected host declaration and the host route closure

The new plan `host-corrected-plan.json` (`f6c6dc67…`, 388,818 B) is a
separate derivation from the accepted source check order: 64 participation
entries, exactly 4 forced names (`conversion_cross_live_scope` and
`output_cross_live_scope`, armed and disabled), each with its control body,
omitted span and violated assertion anchors; 60 not-forced entries with
per-actor reasons; the two `call_cross_live_scope` names recorded as the
masked pair by the retained Q40 `guard_source_matches` on `guard.2` before
the first host write; zero unknown; `observed_logs_used_as_authority=false`;
`source_edits=0`; `control_edits=0`; the prior six-name plan and all 417
files of the c47 scene unchanged. The freeze `host-corrected-freeze.json`
(`a039cc92…`) was written at 06:49:17.562, before the first build at
06:49:30.367. The valid binary `ed15030a…` is 64/0 in hash and nohash at
eight and one, and the weak binary `1b387358…` is 60/4 in both, with exactly
the four conversion/output names. I reran all eight draws myself and
reproduced the counts and names.

The eight-variant host family is also accepted: the plan
(`7eae136d…`) and freeze were written at 06:51:25, before the first build at
06:51:40; the candidate is 34/0 and the seven weak variants are 32/2 with
their named retired/consumed transport pairs; 16 entries and 32 runs are
exact. The host route closure is delivered as eight affected sources
(099–106) in four stage renders each (`factory`, `host`, `installation`,
`scope`), `pending=[]`, and 32 independent stage audits, all exit 0, with
the stages frozen at 07:48:29 before the audits.

### 5. Accepted: the full scope AST and omission audits

All independent audits pass: core/queue 10 objects in two copies, raw 3
objects in one copy and host 3 objects in two copies restore their
accepted pre-edit ASTs with every fixed test body and attribute exact and
the registered callers preserved; the additive audits restore the fixed
bases with 120 core/queue, 15 raw and 30 host added controls whose actors
equal the frozen declarations; the omission audits (494, 78 and 64 tests)
omit only the pointer association with all other nodes exact; the factory
audit extracts the original OperationPermit call body verbatim; and the
scope writer-order audit covers 36 sites (24 core/queue, 4 raw, 8 host)
with checks before every authority write, own retirement after the write,
103 pure falsifiers and zero unknown. These are static evidence; they earn
no cargo, parity or workspace credit.

### 6. Accepted: the source-route migration's static evidence (no cargo credit)

The route migration is accepted as scratch static evidence only. The
propagation is 337 independent stage audits (registration 109, result 114,
append 114) over 114 sources in 170 copies, every audit exit 0 and every
inverse exact. The current caller inventory covers 150 files and 1,762 call
sites with zero unknown, actual receivers recorded, zero free registration
authority calls, and the source wrapper calls preserved. The 46-site order
audit checks the actual-object check before the write, capture retirement
before the guard pop and the old empty-scope guard before reset, rejects 250
deliberately corrupted auditor inputs, and reads no renderer plan. The five
registration sources and three join sources are separately migrated with 48
and 72 exact rustc-built runs. The 70 new route actors (24 registration, 22
join-result, 24 join-append) form the additive 444-control fixture, 444/0 in
four draws, whose AST audit erases the additions back to the fixed 374; it
is not the required complete 374 calibration fixture. The migration's
cargo-level validation — the whole-source typechecks and the runtime-route
builds — is exactly the inadmissible set and must be redone.

### 7. Accepted: the current matrices

The current main family is 920 records at 29,244/232/0/0 with zero
unexpected, all expected sets exact, 848 fresh shared-helper runs and 72
unchanged J runs reused by exact hash identity, and 334 validated file
hashes exact. The raw/paired matrix is 124 records at 6,056/124/0/700, with
460 file hashes exact. The settled component matrices are core/queue 494/0
and 470/24, host 64/0 and 60/4, host-family 34/0 and 32/2, the complete
installation omission 366/8 in four draws, and the five declared InstallGuard
forms 38/0 valid and 30/8 weak with 336 filtered in both cfg arms at eight
and one. I reran four family records and three raw records with identical
counts, and all five form pairs plus the complete weak draw; every
reproduction matched. These are rustc-built fixtures and are independent of
the five prohibited cargo commands; they earn fixture credit only.

### 8. Accepted: the complete 42-variant calibration

The calibration is accepted in full. The participation is 42 variants × 374
controls = 15,708 cells: 93 forced cells, each with its control body,
omitted span and violated assertion anchors; 15,615 not-forced cells, each
with its reason; 0 unknown; 1,596 new installation cells. The declaration
freeze `13a22a04…` lists 99 artifacts and pins the plan
`c90be807…` and participation `578e4769…`; it was written at 08:36:27.07,
before any build or run (the first build follows at 08:39). The independent
declaration audit passes with `all_causal_anchors_exact=true`,
`whole_byte_inverses=true`, `frozen_source_and_control_hashes_exact=true` and
`execution_used=false`; the test AST audit preserves all 374 bodies and
attributes across all 42 variants. All 172 draws are exact and total
63,956/372/0/0: the valid 374-control candidate is 374/0 in all four draws,
and the 168 variant draws total 62,460/372/0/0. There are 38 firing
variants; the four zero-failure variants — guard weak-6, operation-record,
operation-call and operation-ticket — are the declared masked limits with no
falsifier credit, and the masked host-call pair is separate. The Q42
component omissions fire 24 core/queue names, 3 raw names and 4 host
conversion/output names. I re-derived the 15,708-cell and 172-draw
arithmetic from the frozen files, reran the valid candidate at 374/0 and
guard weak-2 at 370/4 in both threads, and reproduced them exactly.

### 9. Accepted, at its stated partial scope: the scope-form matrix

The scope-form declaration binds the unchanged actor scenes to every
declared form: 520 items, 1,040 runs, 79 core/queue forms, 25 raw forms, 1
host form, 22 scope boundaries and 373 scope-object actor obligations. The
plan and freeze were written at 08:57:27.36, before the first run at
08:57:39.41; every expectation is derived from immutable accepted inputs
with `expectation_inputs` recording that no observed execution was used;
every source and binary hash is asserted against the freeze. All 1,040 runs
are exact: 73,880 passed / 8,200 deliberate rejections / 0 ignored /
261,840 declared filtered. This is the scope-object actor subset; it is not
the full affected-source per-form matrix, not a complete 134-boundary or
133-carrier closure, and earns no complete-overlay credit.

### 10. Not credited; still open

- The complete bar: full 134-boundary / 4,422-obligation / 133-carrier
  execution closure; the 151-plan generator with duplicate, overlap and
  unknown-anchor rejection; two fresh 94-target / 188-postimage roots; the
  byte/AST/omission and disabled-parity audits; the six-column 94-row schema
  with all 151 `source_schema` cells; the ordered standing replay; `cargo
  fmt --check`; both workspace modes with the seven declared skips and the
  unfiltered `suspension_oracle`/`typed_slot_census` at failed=0 and
  filtered=0; and the switched same-binary checks. Nothing of this ran:
  `standing-replay` and `workspace-gates` are NOT_RUN, and workspace spent
  0 of 6,000 s.
- The five prohibited commands and every dependent run; the optional
  full-runtime baseline; the full disabled suite.
- The route migration's cargo-level validation, which must be redone in the
  prescribed environment.
- The complete 151-plan/overlay closure and the full affected-source
  per-form matrices.

The schema is unchanged (`f9604852…`, `new_source_schema=null`,
`complete_plan_hash=null`), all 94 admitted paths are `not_emitted` with zero
maintained byte delta and zero complete postimages, the 11,902 scratch
fragments carry exact before/after hashes and 1,081 unique content-addressed
diffs, and 1,110 artifacts authenticate.

### 11. Continuation 49 is ordered

1. Redo the five excluded checks in the prescribed environment: offline,
   locked, `-j8`, `RUST_TEST_THREADS=8`, inherited 16 G/no-swap scope, one
   cargo at a time, no `CARGO_TARGET_DIR` and no codegen/optimization
   change. Use the exact DRAFT-COMMON.md commands and script mirror
   procedure; if a copied crate's default target directory is used, it
   stays under `.temp` and is disclosed. Record the full environment.
2. Then finish the complete bar — the 151-plan generator with rejection,
   two fresh 94-target/188-postimage roots, the six-column 94-row schema
   with 151 cells, the audits, the ordered standing replay, `cargo fmt
   --check`, both workspace modes with the seven skips and the unfiltered
   oracle/census, and the switched same-binary checks — and the full
   134/4,422/133-carrier execution closure with the affected-source per-form
   matrices and every moved-duty caller listed.
3. Reuse only by exact hash identity; repair no frozen tree, production or
   maintained source; never weaken an expectation or re-label an omission.
4. Envelope: the CM4 share remainder is **1,820.41086498898 s**
   (42,679.58913501102 committed of 44,500), which is the binding limit;
   the focused line's 1,838.2350947419345 s exceeds it by 17.824 s and is
   not additive. No extension. Workspace is 0/6,000 s; the 1,440 s
   verification and second 6,000 s workspace stay held; 743–746 retain
   40,500 s.

### 12. Budget and accounting

Charged 6,350.574759540948 s = 3,918.3346526989335 s over 4,720 charged
receipts + 2,360.0 s bookkeeping + the 60 s reserve + 2 closing receipts at
12.240106842014939 s. All five prohibited commands remain charged. The
receipt count, the per-receipt seconds, the bookkeeping, the reserve and
the closing sum all reproduce exactly from the receipts themselves.

### 13. Freezes, entry and status

The 19 prior frozen trees rehash 29,611 entries / 23,548,300,905 bytes with
zero bad copies and zero unaccounted originals — only the two known open
`freeze-stop.log` drifts (continuations 32 and 33), whose actual hashes
match the recorded ones, and whose closed copies are exact. The new
27,451-entry tree rehashes zero-bad on both sides. Entry: 5,400 maintained
files, 590 design files and 77,578 accepted files authenticate — I rehashed
all 83,568 hashes (54.5 GB accepted) with zero differences — and the three
inherited document drifts (TODO.md, brief-623, brief-742) match their
committed entry hashes and differ from their priors. No git or network
command ran in the worker; all writes are under `.temp/brief-742`.

Status is unchanged: pair `(45,5,false)` Open with zero admitted receipts;
C2 Open at 118 Residual + 0 Lost; 51 owners and five source duties Open;
`CHANGELOG.md` and S10 blocked; CM5 unauthorized; no campaign, epoch,
acceptance or settlement.

Next: continuation 49 completes the bar; then the verification stage
(1,440 s + held second 6,000 s workspace), 743–746 and CM5.

## Continuation 49 ruling (director) — continuation 49 re-earns the five excluded cargo checks in the prescribed environment and then stops under Q28 on the fresh disabled full-runtime arm, where three ordinary runtime tests fail that the live copy passes; the stop is compliant, the five checks are accepted as focused source-route validation only, both failure classes are ruled scratch-instrument and scene-construction defects rather than production regressions, and the complete bar remains unexecuted.

Continuation 49 redid the five checks that continuation 48 excluded from credit,
then ran the optional full-runtime baseline it had declared, and stopped under
Q28 when the fresh disabled copy failed three ordinary runtime tests that the
live copy passes. The five checks are re-earned in the prescribed environment
and are accepted with focused source-route credit only. The stop is compliant
and its cost stands. Both failure classes are ruled instrument/scene
construction defects in scratch, not production regressions: the append-route
probe is not inert when `ENABLED=false`, and the probe/control module's text
sits in a copied `src/lib.rs` that two unchanged maintained source gates scan.
The complete bar is not delivered and earns nothing. Continuation 50 must make
the disabled instrument inert at every entry point, classify the direct
`FrameStorage::append` callers with zero unknown, keep both source gates
unchanged and green on the copied tree, and then finish the complete bar and
the full closure under the CM4 share's remaining 1,201.5615750949778 s.

### 1. Run record

`cdx` ran as unit `brief-742-cdx-20261118-095208-1204692`, rc 0, peak
2,685,403,136 B, oom 0 / oom_kill 0; log 1,374,354 B, sha256
`6f6ae18eb29e63c07e3981e6d62609014eef2d9ac3e9ecc50a291b67219a3376`. The
report is STOP: 48,533 changes (48,531 added, 2 modified — `PROGRESS.md` and
`report.json` — 0 deleted), all under `.temp/brief-742`; 16 checks (11 PASS, 3
FAIL, 2 NA), 174 citations verified, 0 questions, 1 deviation, 0 disclosures.
No git command ran (0 executed git calls, no `.git-attempts`), no network. I
rehashed every entry hash — 139,586 files, 71.86 GiB — with zero differences,
and the working tree shows exactly the same 23 porcelain entries / 32 expanded
instances at HEAD `f49a96c6`.

### 2. Accepted: the five excluded checks re-earned

The two source typechecks and three runtime test builds ran in the prescribed
environment: `cargo check/test --offline --locked -j8` with the c49 roots'
own default target directories under `.temp`, no `CARGO_TARGET_DIR` in any of
the 30 run receipts (cmd or env), 16 G/no-swap scope inherited, scripts
mirrored, and no prior cargo evidence reused. Exit 0 throughout; the six
selected route unit draws are 35/0 each (210 passed) at eight and one thread
on the same binary per arm. The plan and freeze were written at 09:57:14
before the first check at 09:57:37.

I reproduced them myself: clean-target replays of both typechecks (11.51 s /
2.41 s, exit 0) and of the fresh disabled build (28.55 s, exit 0, test binary
produced) in director copies under `.temp/director/gate-c49/`, and all six
route draws 35/0 from the frozen binaries. Credit is limited to focused
source-route validation: this is not workspace credit, not complete-record
parity, and not a complete overlay.

### 3. Admitted: the Q28 STOP is compliant

The runtime-full plan and freeze were written at 09:59:51, before the first
run at 10:00:06, and declare 1433/0/5/2 for both arms (five ignored names
resolved from source attributes, all seven brief skips supplied, two matching
runtime names, `lp655`/`lp656` filtered, no observed execution used as
authority). The live arm — a byte-exact copy of the maintained source, 4,716
source files compared with zero mismatches — is 1433/0/5/2 at eight and one.
The fresh disabled arm, binary `6476f56d…`, is 1430/3/5/2 at both thread
counts with exactly
`runtime::profiling::tests::queue_boundaries::logical_queued_frames_have_one_owner_in_both_modes`,
`runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold`
and
`runtime::tests::hygiene::kernel_crates_do_not_write_stdout_outside_tests_and_bins`.
The worker reproduced at one thread on the same binary, claimed no production
regression, no ordering artifact and no repair, and froze 24,244 files /
6,362,142,797 bytes. I reran both arms at eight and one myself and reproduced
1433/0/5/2 and 1430/3/5/2 with the same three names. The stop is compliant
and no unexecuted work is credited.

### 4. Ruled: the disabled instrument is not inert (`disabled-runtime-contract`)

The failure is a scratch-instrument defect, not a candidate-contract finding
and not a production regression:

- The unchanged test calls `session.pending_suspension.frames.append(...)`
  directly (`crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs:253–258`).
  The generated actual-object hooks exist only in the wrapper
  `SuspendedEvaluation::append` (copied `core.rs:213–219`), which mints the
  `JoinAppend`, calls `begin_append_call`, binds the profiled queue and calls
  `finish_append_call` around `self.frames.append`. The direct storage caller
  bypasses all of it, while `ProfileQueue::append` still calls
  `observe_append_queue`, so the observation resolves a registered namespace
  with no bound actual and asserts at copied `lib.rs:2626`
  (`PortBlocked::MissingActualAppendQueueSource`).
- The append-route public entry points (`begin_append_call`,
  `bind_append_queue`, `observe_append_queue`, `finish_append_call`, copied
  `lib.rs:2677–2679`) do not have the `if !ENABLED { return }` early return
  that the other probe entry points already use (and `ENABLED` is `false` at
  copied `lib.rs:30`). The call sites also mint the `Rc` unconditionally, so
  the disabled build pays a per-append allocation and TLS work and can assert
  on ordinary execution. That is both the observed behavioral failure and a
  default-path cost the standing rule forbids.

The direct-caller inventory is zero-unknown: a whitespace-compacted scan of
`crates/rum-runtime/src`, `crates/rum-core/src` and `crates/rum-stdlib/src`
for the six queue-mutation needles finds exactly one maintained-tree direct
`FrameStorage::append` caller — the unchanged test above — while the other
hits are `core.rs` (the instrumented wrapper and the queue API) and the
scanner's own needle text in `frame_root_trace.rs`. Production's only
`FrameStorage::append` caller is the instrumented wrapper; `driver.rs:171`
appends through `SuspendedEvaluation`, and `core/queue.rs:232–244` appends
container vectors, not frame storage.

Ruling: when `ENABLED=false`, every probe entry point returns before any
state — no minted `Rc`, no TLS, no bookkeeping, no assertion — and the
disabled full-runtime arm must reproduce exactly 1433/0/5/2 at eight and one.
The direct `FrameStorage::append` callers must be enumerated and classified
with zero unknown in the enabled declaration (the maintained test is a
declared bypass, not something to edit), no maintained test may be edited,
and the enabled invariant is stated over production routes only.

### 5. Ruled: the two source-gate collisions are scene construction (`copied-source-guards`)

Both failing tests are unchanged maintained source scanners:

- `hygiene.rs:30–95` walks `crates/rum-{core,runtime,stdlib}/src` and rejects
  `println!`/`print!`/`io::stdout`/`stdout()` in any `.rs` whose path does not
  contain `/tests/` or `/bin/`.
- `frame_root_trace.rs:713–723, 190–206` walks the `.rs` files of
  `crates/rum-runtime/src` and `crates/rum-stdlib/src` skipping `/tests/`,
  and asserts no `.frames.insert(` and the other queue mutations anywhere
  outside `core.rs`.

The copied root's `src/lib.rs` carries the probe and control module: retained
`#[cfg(any())]` fixture bodies with stdout macros, and the helper ledger's
`.frames.insert(` at `lib.rs:782`. The maintained tree keeps both gates green
and neither gate was edited. These failures are source-scanner collisions
with scratch text in a scanned path, not proof of a live queue mutation and
not a stale-fingerprint/certificate/position ordering artifact. Ruling: the
corrected copied tree must keep both gates unchanged and green — the probe
and control module text is placed under a path both scanners skip (or the
tree is otherwise arranged so the unchanged scanners see the maintained
text) — and no scanner may be weakened, edited or skipped.

### 6. Not delivered: the complete bar

The 151-plan generator with duplicate/overlap/unknown-anchor rejection, two
fresh 94-target/188-postimage roots, the six-column 94-row schema with all
151 `source_schema` cells, the byte/AST/omission audits, the ordered standing
replay, `cargo fmt --check`, both workspace modes with the seven skips and
the unfiltered `suspension_oracle`/`typed_slot_census` at failed=0 and
filtered=0, the switched same-binary checks, and the full 134-boundary /
4,422-obligation / 133-carrier execution closure with the affected-source
per-form matrices and every moved-duty caller listed: none ran and none is
credited. Standing replay and workspace gates are NOT_RUN; workspace spent 0
of 6,000 s. The schema is unchanged (`f9604852…`, `new_source_schema=null`,
`complete_plan_hash=null`), all 94 admitted paths are `not_emitted` with zero
postimages, and the 90 preimages / 180 postimages / 151 cells authenticate.

### 7. Budget, freeze and entry

Charged 618.8492898940021 s: 544.3492898940021 s over 29 receipts (every log
hash exact), 14.5 s bookkeeping, the fixed 60 s reserve; two closing receipts
fall inside the reserve. CM4 now stands at 43,298.43842490502 of 44,500 s,
leaving **1,201.5615750949778 s** as the binding limit — no extension; the
1,440 s verification envelope, the second 6,000 s workspace, 743–746's
40,500 s and workspace 0/6,000 stay held.

The new scene is 24,244 files / 6,362,142,797 bytes with the freeze log and
its pending receipt excluded; all 24,244 frozen copies are byte-exact to the
manifest. My own full-suite replay regenerated 116 transient compile-control
and incremental files in the two copied roots' nested `.temp/brief-579` and
`target/brief-538` (originals only, mtimes 10:16–10:19; frozen copies exact);
that drift is the director's replay side effect and is recorded, not
repaired. No candidate, control, source, expectation, predicate or observer
changed: the 30 copied-source rows are exact against the accepted c48 sources
(11 unique content diffs, `candidate_edits=0`, `maintained_edits=0`). Entry
5,400 maintained / 590 design / 133,596 accepted authenticate with zero
differences, the three inherited document drifts (TODO.md, brief-623,
brief-742) are at their committed entry hashes (verified against HEAD blobs),
and the pre-existing change layer is unchanged.

### 8. Continuation 50 is ordered

1. Make the disabled instrument inert: every append-route entry point returns
   before any state when `ENABLED=false`, including the call-site mint, and
   the disabled full-runtime arm reproduces exactly 1433/0/5/2 at eight and
   one thread on one binary. Classify every direct `FrameStorage::append`
   caller with zero unknown (the maintained test is a declared bypass; no
   maintained test is edited). Keep both unchanged source gates green on the
   copied tree without weakening them.
2. Then finish the complete bar and the full closure: the 151-plan generator
   with rejection, two fresh 94-target/188-postimage roots, the six-column
   94-row schema with 151 cells, the byte/AST/omission and disabled-parity
   audits, the ordered standing replay, `cargo fmt --check`, both workspace
   modes with the seven skips and the unfiltered oracle/census, the switched
   same-binary checks, and the 134/4,422/133-carrier execution closure with
   the affected-source per-form matrices and every moved-duty caller listed.

Under the binding 1,201.5615750949778 s, reusing only by exact hash identity
and repairing no frozen tree, production or maintained source. No campaign,
epoch, acceptance or CM5; pair `(45,5,false)` Open with zero admitted
receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open;
`CHANGELOG.md` and S10 blocked.

## Continuation 50 ruling (director) — continuation 50 delivers all three ordered prerequisites (the append-route instrument is inert when `ENABLED=false`, the direct `FrameStorage::append` callers are classified with zero unknown, and both unchanged source gates are green on the corrected copied tree with the probe/control text under `src/tests`) and then stops on a compliant allocation limit when the 848-run / 230-build changed-source matrix cannot fit the CM4 share's remaining 612.5448643459799 s; the three deliverables are accepted with focused scratch credit only, the complete bar remains unexecuted, and this gate allocates a fresh 16,000 s focused envelope from the project's reserve (CM4's share 60,500 s, stage reserve 101,000 s, 743–746 unchanged) for continuation 51.

Continuation 50 delivered all three ordered prerequisites: the append-route
instrument is inert when `ENABLED=false`, the direct `FrameStorage::append`
callers are classified with zero unknown, and both unchanged source gates are
green on the corrected copied tree with the probe/control text under
`src/tests`. It then stopped on allocation when the next required group — the
848-run / 230-build changed-source matrix — could not fit in the CM4 share's
remaining 612.5448643459799 s even at its prior measured charge of
782.9024138279947 s. The stop is compliant; the three deliverables are
accepted with focused scratch credit only; the complete bar is not delivered
and earns nothing. This gate allocates a fresh 16,000 s focused allocation
from the project's reserve — CM4's share becomes 60,500 s and the stage
reserve 101,000 s, with 743–746's 40,500 s unchanged — and orders
continuation 51 to finish the complete bar and the full closure under
16,396.6435590189917 s.

### 1. Run record

`cdx` ran as unit `brief-742-cdx-20261118-103011-1239230`, rc 0, peak
2,631,008,256 B, oom 0 / oom_kill 0; log 2,095,909 B, sha256
`7d87767c47643092315c36375b2dfc60ef5c182f3bf977a9e5ab528a914f8140`. The
report is STOP: 36,930 changes (36,928 added, 2 modified — `PROGRESS.md` and
`report.json` — 0 deleted), all under `.temp/brief-742`; 20 checks (15 PASS,
2 FLAGGED, 1 FAIL — the complete bar — 2 NA), 2,114 citations verified, 0
questions, 1 deviation (the complete bar not delivered), 0 disclosures. The
stop rule is the envelope limit: "stop before a required group exceeds the
envelope; no extension or held allocation borrowing", with the reason that
the next complete required matrix does not fit the remaining allowance even
at its prior measured charge, before mandatory closing authentication, and
that no candidate control failed. No git command ran and no network. I
rehashed every entry hash — 188,116 files across 5,400 maintained, 590 design
and 182,126 accepted — with zero differences and zero missing, and the
working tree shows exactly the same 23 porcelain entries at HEAD `dabf605d`.

### 2. Accepted: the append-route instrument is inert when disabled

Eight registered helper edits make the six protocol primitives private and
add four public source boundaries that return before any state; the
constructor's `ENABLED` gate precedes `Rc::new`, so a disabled append
allocates nothing; `SuspendedEvaluation::append` enters through
`source_append` and binds and finishes only when it returns `Some`;
`ProfileQueue::append`'s namespace and owners-mutex computation sits inside
`if ENABLED`; and the probe/control module moved out of `src/lib.rs`
(226,973 → 764 B) into a new `src/tests/cm740_probe.rs` (226,862 B). The AST
audit passes with the byte inverse exact on all three edited files, four
public source gates, six private primitives, `mint_gate_before_Rc=true`,
`ordinary_append_once=true`, every fixed control body and all other nodes
exact, and seven firing falsifiers — wrong-enable-condition,
state-before-gate, mint-before-gate, duplicate-ordinary-append,
fixture-attribute-change, namespace-lock-before-enable, scanner-visible-
helper — each rejected.

The declaration (plan and edits at 10:46:41) preceded the build at 10:47:55
and the runs at 10:48:12 and 10:48:52, and the disabled full-runtime arm is
exactly 1433/0/5/2 at eight and one thread on one binary, with all seven
inherited skips supplied and the five ignored names resolved from source
attributes. I reproduced it independently: I copied the v3 root to
`.temp/director/gate-c50/runtime-disabled-v3`, rebuilt the test binary there
(28.81 s, exit 0) and ran it at eight (16.34 s) and one (40.64 s) — 1433
passed, 0 failed, 5 ignored, 2 filtered, exit 0 both times, with
`logical_queued_frames_have_one_owner_in_both_modes`,
`suspension_protocol_repository_invariants_hold` and
`kernel_crates_do_not_write_stdout_outside_tests_and_bins` green. Credit is
focused source-route validation of the append boundary only: not complete
inertness, not execution closure of all 134 boundaries, not workspace,
overlay or parity credit.

### 3. Accepted: the direct-caller classification is zero-unknown

The whitespace-compacted six-needle census over the maintained runtime, core
and stdlib sources finds 12 hits classified with zero unknown: one production
wrapper, five `SuspendedEvaluation` wrapper/queue-API hits, six scanner
needles that are not executed calls, and exactly one direct
`FrameStorage::append` caller — the unchanged `queue_boundaries` test,
declared the enabled bypass — with `maintained_tests_edited=0` and
`test_or_scanner_edits=0`. The caller inventory is explicit about its own
limits: 125 source files, 1,599 indexed calls, 444 append-duty calls, four
current external source calls (exactly the four public boundaries in the
copied `core.rs` and `profile_queue.rs`), zero external calls to the now
private primitives, and 114 older source copies pending;
`exhaustive_within_declared_source_list=true` but
`finite_dispatch_graph_proven=false`, `complete_closure=false` and
`source_propagation_reused=false`. That is a pending inventory, not a
closure certificate, and it earns no execution credit.

### 4. Accepted: both unchanged source gates are green on the corrected tree

The relocation is the c49 remedy: the helper lives at
`crates/rum-runtime/src/tests/cm740_probe.rs`, a path both unchanged scanners
skip, and neither scanner was edited, weakened or skipped — they are two of
the 1,433 maintained tests that pass in the disabled arm at eight and one.
The scene adds exactly one scratch module (`additional_scratch_module=1`) and
claims no generated overlay addition (`generated_overlay_additions=0`,
`complete_overlay=false`).

### 5. Accepted: the preserved route controls and the AST restoration

The source-boundary fixture keeps all 444 accepted control bodies and
attributes byte-exact, adds two counterparts (the same
`actual_source_boundaries_follow_compile_time_enable_without_state_when_disabled`
test in both `ENABLED` copies), and passes 446/0/0/0 in both feature modes at
eight and one thread; the whole-AST restoration, byte inverse and enable
values are exact, and the declaration preceded execution
(`execution_used=false`). The added control proves both branches: with
`ENABLED` true it constructs, binds, observes and finishes through the public
entries and asserts the recorded namespace and completion; with `ENABLED`
false `source_append` is `None`, all four public entries stay inert even with
a fixture-owned private core object and the state mutably borrowed, no append
source is recorded and the fixture's own entry is untouched. I reran all four
draws from the frozen binaries: 446 passed, 0 failed, 0 ignored, 0 filtered
in both modes at eight and one, each binary printing both its `enabled=true`
and `enabled=false` branch. These are fixture controls; they produce no
catch, admission or acceptance receipt.

### 6. Admitted: the allocation STOP is compliant

The 46 receipts sum to exactly 721.9180160759861 s; with 23.0 s of
bookkeeping (46 × 0.5) and the fixed 60 s reserve the continuation charged
804.9180160759861 s of the binding 1,201.5615750949778 s, leaving
396.6435590189917 s. At the decision point it had charged
589.0167107489979 s with 612.5448643459799 s remaining, while the next
required group — 848 runs, 230 distinct builds, pinned in
`pending-source-closure.json` with 114 sources and 170 copies — costs at
least its previously measured 782.9024138279947 s and at most the
conservative 3,210 s. The group was not started, no extension or held
allocation was borrowed, no candidate control failed, and no observed
execution was used as an expectation. The stop is the prescribed behaviour
and its costs stand. I verified the ledger myself: 46 receipts in the focused
group, no `CARGO_TARGET_DIR` in any of them, no codegen or optimization
override, both cargo commands `--offline --locked -j8`, the 16 G/no-swap
scope inherited with oom 0, and the arithmetic exact to the recorded
decimals.

### 7. Not delivered: the complete bar

The 151-plan generator with duplicate/overlap/unknown-anchor rejection, the
two fresh 94-target/188-postimage roots, the six-column 94-row schema with
all 151 `source_schema` cells, the byte/AST/omission and disabled-parity
audits, the ordered standing replay, `cargo fmt --check`, both workspace
modes with the seven skips and the unfiltered `suspension_oracle`/
`typed_slot_census` at failed=0 and filtered=0, the switched same-binary
checks, and the full 134-boundary / 4,422-obligation / 133-carrier execution
closure with the affected-source per-form matrices and every moved-duty
caller listed: none ran and none is credited. Standing replay and workspace
gates are NOT_RUN; workspace spent 0 of 6,000 s. The schema is unchanged
(`f9604852…`, `new_source_schema=null`, `complete_plan_hash=null`), all 94
admitted paths are `not_emitted` with zero postimages, and the 90 preimages /
180 postimages / 151 cells authenticate.

### 8. Budget, freeze and entry, and the allocation decision

Charged 804.9180160759861 s: 721.9180160759861 s over 46 receipts (every log
hash exact), 23.0 s bookkeeping, the fixed 60 s reserve; the closing receipt
falls inside the reserve. CM4 stands at 44,103.356440981006 of 44,500 s.

Allocation decision: a fresh 16,000 s focused allocation from the project's
reserve extends CM4's share to 60,500 s and the stage reserve from 85,000 s
to 101,000 s; 743–746 retain their 40,500 s, the 1,440 s verification
envelope and the second 6,000 s workspace stay held, and the assigned 6,000 s
workspace envelope stands untouched. Continuation 51's focused envelope is
therefore 396.6435590189917 + 16,000 = **16,396.6435590189917 s**. This is an
allocation from the project's reserve at a new gate, not an extension granted
inside the c49 envelope — the c43 precedent — and the bar, the stop-loss and
the authorization limits are unchanged.

The new scene is 18,419 files / 5,194,445,665 bytes, covering the three
scratch roots (v1 4,717 files — the failed first render retained — and v2/v3
6,770 each) and the continuation's artifacts. I rehashed both sides of every
manifest entry — 36,838 hashes — with zero differences and zero missing. The
21 prior frozen trees hold 81,306 entries with their copies exact, the two
known open log drifts and the 116 director-regenerated transients (108
missing and 8 changed on the originals side at entry, unexplained 0);
nothing was repaired, and my sample of 400 entries each from the c48 and c49
manifests is exact. Entry: 5,400 maintained, 590 design and 182,126 accepted
files authenticate, 188,116 hashes rehashed with zero differences and zero
missing. The three inherited document drifts (TODO.md, brief-623, brief-742)
are at their committed entry hashes, verified against HEAD blobs, and the
pre-existing change layer is unchanged at HEAD `dabf605d`. The maintained
test build carries its three pre-existing dead-code warnings; the copied
instrument adds one `unused_must_use` warning at `cm740_probe.rs:1319` that
has been present in the instrument since continuation 48 — it is scratch,
not maintained, and the complete overlay's audit must settle it rather than
carry it.

### 9. Continuation 51 is ordered

1. Propagate the guarded append boundary/private-core split to the 114
   listed affected sources and their 170 copies, independently audit each
   render, and execute the 848 required family draws under their recorded
   expected sets at eight and one thread.
2. Complete the full 134-boundary / 4,422-obligation / 133-carrier execution
   closure with the affected-source per-form matrices and every moved-duty
   caller listed.
3. Complete the 151-plan generator with duplicate/overlap/unknown-anchor
   rejection, the two fresh 94-target/188-postimage roots, the six-column
   94-row schema with all 151 `source_schema` cells, and the byte/AST/
   omission and disabled-parity audits.
4. Then the ordered standing replay, `cargo fmt --check`, both workspace
   modes with the seven skips and the unfiltered oracle/census at failed=0
   and filtered=0, and the switched same-binary checks.

Under the 16,396.6435590189917 s focused envelope and the untouched 6,000 s
workspace envelope, reusing only by exact hash identity and repairing no
frozen tree, production or maintained source. If a required group cannot fit,
stop before starting it and report the exact remainder. No campaign, epoch,
acceptance or CM5.

## Continuation 51 ruling (director) — continuation 51 delivers the 114-source / 170-copy propagation, the 848-run / 230-build family group, the three J routes, the 48 component draws and the 640 fresh scope draws (with 400 unchanged raw draws reused only by exact hash identity), and then stops under Q28 when the new 450-control disabled-entry fixture fails exactly its four additive held-borrow controls at eight and one on one binary: the four public lookups borrow the TLS ledger before any `ENABLED` check, violating the c49 disabled invariant as written; the repair is four guards, proved in a director copy to yield 450/0/0/0; the delivered evidence is accepted with focused source-route credit only, the complete bar remains unexecuted, and continuation 52 runs under the remaining 14,070.369583264994 s.

Continuation 51 propagated the accepted c50 append boundary/private-core split
to the 114 listed sources and their 170 copies, re-ran the 848-run / 230-build
family group that the c50 envelope could not hold, added the three copied J
source routes with 72 draws, re-established the 48 component draws and the 640
fresh scope draws (with 400 unchanged raw draws reused only by exact hash
identity), and then stopped under Q28 when four additive held-borrow controls
in the new 450-control disabled-entry fixture failed at eight and one thread on
one binary. The stop is compliant: the fixture is a compiled candidate, its
declared expectation was 450/0/0/0, and the observed 446/4/0/0 is exactly the
four new controls, reproduced by me from a director copy with a causal
four-guard repair in a copy yielding 450/0/0/0. The finding is real and is
ruled an instrument-surface inertness defect, not a production regression and
not a semantic catch. The delivered propagation and matrices are accepted with
focused source-route credit only. The complete bar is not delivered and earns
nothing. Continuation 52 must repair the four lookups under a class-wide
public-surface enumeration, re-render and re-run the affected evidence, and
then finish the bar under the remaining 14,070.369583264994 s.

### 1. Run record

`cdx` ran as unit `brief-742-cdx-20261118-111832-1277497`, rc 0, peak
3,606,089,728 B, oom 0 / oom_kill 0; log 1,837,004 B, sha256
`9b4d8488de79447d299e900890556b72211b6afad8017373615875f1209a4181`. The
report is STOP: 9,621 changes (9,619 added, 2 modified — `PROGRESS.md` and
`report.json` — 0 deleted), all under `.temp/brief-742`; 22 checks (17 PASS, 1
FLAGGED, 2 FAIL, 2 NA), 2,526 citations verified, 0 questions, 1 deviation, 0
disclosures. The expanded `report-all-command-runs.json` carries all 2,066
child runs and validates at 6,326 citations. No git command ran (0 executed
git calls, no `.git-attempts`), no network. I scanned the ledger myself: 2,066
focused receipts, no `CARGO_TARGET_DIR` in any command or environment, no
codegen or optimization override, every cargo command the ledger holds is
offline/locked at `-j8`, all cwd values are the repo root, zero git and zero
network commands, the 16 G/no-swap scope with oom 0. The arithmetic is exact:
1,233.2739757539966 s measured + 1,033.0 s bookkeeping (2,066 × 0.5) + 60 s
reserve = 2,326.2739757539966 s charged of the 16,396.6435590189917 s
envelope, leaving 14,070.369583264994 s; CM4's share stands at
46,429.630416735 of 60,500 s; workspace 0 of 6,000; the 1,440 s verification
and the second 6,000 s workspace stay held; no envelope overrun, and the stop
is a candidate failure, not an allocation stop.

### 2. Accepted: the 114-source / 170-copy propagation and its audits

The c51 plan reuses the accepted c50 declaration by exact hash identity
(`disabled-v3-edits.json`, `39403a0d…`; the plan's eight edits are byte-equal
to the c50 `helper_edits`) and renders 114 sources with 170 copies, 8
registered edits per copy, byte inverse exact, fixed control bodies unchanged,
0 unknown source matches, 0 expectation changes. The independent auditor
(`22e1f740…`) ran once per source: 114 logs, all exit 0, and the driver's PASS
line reports whole-AST restoration, fixed controls and attributes exact,
source boundaries matching the accepted c50 pins, and 0 unknown edits. I
re-ran five sampled audits (sources 1, 40, 57, 80, 114) from the frozen
commands and reproduced every PASS line, and I confirmed the declaration hash
against the c50 file.

### 3. Accepted: the 848-run family group, the J routes, the components and the scope draws

The main family is 848 runs over 230 distinct builds, declared before
execution (plan and freeze at 11:23:58, first audit at 11:24:16, first build
at 11:25:42), and every run matches its declared counts, exit and failure set:
26,964 passed / 232 deliberate rejections / 0 ignored / 0 filtered / 0
unexpected. The three copied J source routes add 72 runs (2,280 passed) with
six audited actual append methods; I re-ran all three v2 whole-file audits and
reproduced their PASS lines, and the first auditor's retained failure is
settled as construction: it matched the unchanged `runtime::real` homonym and
rejected the reference body, while the v2 auditor qualifies `runtime::actual`,
restores only its two declared append bodies from the accepted c50 pins, and
compares the whole file including the reference; the failed attempt, log and
candidate stay frozen. The component matrix re-ran 24 items / 48 draws exact
at 494/0, 470/24, 64/0, 60/4, 34/0 and 32/2. The scope matrix re-ran 320 fresh
items / 640 draws exact, and the 200 reused items / 400 runs are the accepted
c47 raw draws whose source and binary hashes still match exactly; the total
decomposes exactly to 73,880/8,200/0/261,840 (fresh 68,480/7,600/236,640 plus
reused 5,400/600/25,200). With the J runs, the source-family total is 920
draws at 29,244/232/0/0. I re-ran all four drivers myself from copies under
`.temp/director/c51-{route,join,components,scope}`: the route driver
reproduced all 12 group PASS lines and 26,964/232/0/0 over 848 runs and 230
builds (471.34 s), the join driver reproduced 72 runs / 2,280 passed, the
component driver reproduced 24 items / 48 exact draws, and the scope driver
reproduced 320 items / 640 exact draws. The calibration declarations are
preserved: 42 × 374
= 15,708 cells, 93 forced, 15,615 not-forced, zero unknown; the 172 new
calibration draws did not run and are not claimed.

### 4. Compliance of the Q28 stop

The fixture declared `counts: [450, 0, 0, 0]` and `failures: []` in
`disabled-entry-plan.json`, frozen at 11:49:23.921 together with the candidate
`disabled-entry-controls.rs` (`555ce84c…`); the build ran at 11:49:50.549 and
the two runs at 11:49:50.836 (t8) and 11:49:51.305 (t1) on the one binary
`07d5dea4…`, both exit 101 with 446 passed / 4 failed / 0 ignored / 0
filtered and exactly the four `disabled_lookup_entry_contract` names, each
log carrying four `already mutably borrowed: BorrowError` panics. I copied the
frozen candidate into `.temp/director/gate-c51/repro.rs`, rebuilt it with the
same rustc invocation and the same maintained `librum_core` rlib, and
reproduced 446/4 at eight and one on my binary with the same four names; the
nohash draw is correctly NOT_RUN after the same-binary stop, no candidate or
expectation was repaired, and the stopped scene is frozen. The AST audit is
non-vacuous: it parses the c50 base and the candidate, asserts 446 vs 450
tests, proves every base test's token stream is present unchanged, then erases
the added module and asserts the whole-file token stream equals the base
exactly, with `helper_edits=0`.

### 5. The finding: the four disabled lookups touch TLS before any check

The four failing controls are a valid detector, not an artifact. Each runs in
a fresh thread, asserts `!ENABLED`, holds the thread-local ledger's
`borrow_mut()` live across the call and asserts the lookup returns false; a
disabled entry that returns before any state cannot panic, while one that
borrows the same TLS cell panics. The four functions in the unchanged c50
helper are:

- `pub fn ordinary(row: &str) -> bool { LEDGER.with(|c| eligible(&c.borrow(), row, true).is_some()) }` (`cm740_probe.rs:455`)
- `pub fn frame(row: &str) -> bool { LEDGER.with(|c| eligible(&c.borrow(), row, false).is_some()) }` (`:456`)
- `pub fn initial(row: &str) -> bool { LEDGER.with(|c| initial_token(&c.borrow(), row).is_some()) }` (`:939`)
- `pub fn ordinary_request(row: &str) -> bool { LEDGER.with(|c| request_token(&c.borrow(), row).is_some()) }` (`:1002`)

None has an `ENABLED` guard, so each borrows the TLS ledger before checking
the compile-time flag. That violates the c49 ruling's disabled invariant as
written in brief-742 lines 3442–3444 — "when `ENABLED=false`, every probe
entry point returns before any state — no minted `Rc`, no TLS, no
bookkeeping, no assertion" — and the c51 plan declared and froze that reading
before execution. The repair is causal and minimal: adding `if !ENABLED {
return false; }` before the borrow in each function, in a director copy,
turns the same fixture into 450 passed / 0 failed / 0 ignored / 0 filtered at
eight and one. The finding is classified as a scratch instrument-surface
inertness defect: the four are declared public lookups of the 134-boundary
consumer surface (59 declared duties over 40 rows and 40 forms, anchored at
their definitions by `lookup-obligations.json`), my scans of the copied runtime
tree and the maintained tree find zero external calls to the four (only the
probe's own fixtures call them), and no maintained code calls them, so this is
not a production regression, not a default-path cost in the maintained
runtime, and not a semantic catch; the c50 credit covered only the append
boundaries and is not extended. The disabled full-runtime arm was
not re-run after the stop, and the four lookups' inertness is the one open
contract item.

### 6. Not delivered

The complete bar remains unexecuted and earns nothing: the full
134-boundary/4,422-obligation/133-carrier execution closure with the
affected-source per-form matrices and every moved-duty caller, the 151-plan
generator with duplicate/overlap/unknown-anchor rejection, the two fresh
94-target/188-postimage roots, the six-column 94-row schema with all 151
`source_schema` cells, the byte/AST/omission and disabled-parity audits, the
ordered standing replay, `cargo fmt --check`, both workspace modes with the
seven skips and the unfiltered `suspension_oracle`/`typed_slot_census` at
failed=0/filtered=0, and the switched same-binary checks. Standing replay and
workspace are NOT_RUN; workspace spent 0 of 6,000 s; complete plans 0 of 151;
complete calibration runs 0 of 172; emitted overlay paths 0; `new_source_schema=null`
and `complete_plan_hash=null`. The schema is unchanged (`f9604852…`), all 94
admitted paths are `not_emitted`, and the 90 preimages / 180 postimages / 151
cells authenticate. No campaign, epoch, acceptance, settlement or CM5.

### 7. Allocation and continuity

No new allocation is made. Continuation 52 runs under continuation 51's
remaining focused envelope of 14,070.369583264994 s and the assigned 6,000 s
workspace envelope (spent 0); CM4's share stays 60,500 s with
46,429.630416735 s committed, 743–746 retain their 40,500 s, and the 1,440 s
verification and the second 6,000 s workspace stay held. This is the carried
pool's remainder at a new gate, not an extension; the bar, the stop-loss and
the authorization limits are unchanged.

### 8. Freeze and entry

The stopped scene is 4,798 files / 3,212,799,257 bytes in
`disabled-lookup-stop/`, frozen after the stop at 11:54:55 with the failed
controls, logs, candidate and binaries retained; I rehashed both sides of
every entry (9,596 hashes) with zero bad or missing. The entry authenticates
5,400 maintained, 590 design and 219,053 accepted files. I rehashed all 590
design and 219,053 accepted hashes — 219,643 files, 100,179,110,800 bytes —
with zero differences and zero missing, recomputed the 5,400 maintained
hashes (121,623,089 bytes, zero bad), and verified that the only maintained
deltas against the c50 entry are the three inherited documents I committed in
`4c7c7f6f` — `TODO.md`, brief-623 and brief-742 — whose working-tree hashes
equal their HEAD blobs. The accepted carry is exact: all 182,126 c50 accepted
hashes are unchanged and 36,927 files are new, which is every file under
`continuation-50` (the 36,928th added path is the excluded top-level
`report-continuation-50.json`, outside the accepted categories by
construction). The prior scenes hold 22 trees / 99,725 entries; my sample of
400 entries each from the c50 and c48 manifests is exact, the two known open
`freeze-stop.log` drifts remain open and unrepaired, and the 116
director-regenerated transients are unchanged (108 missing, 0 changed,
unexplained 0). The three inherited document drifts are at their HEAD blobs;
the pre-existing layer is unchanged at HEAD `4c7c7f6f` (23 porcelain
entries); the maintained test build's three dead-code warnings are
pre-existing and the inherited `unused_must_use` at `cm740_probe.rs:1319`
remains scratch, to be settled by the complete overlay's audit.

### 9. Continuation 52 ordered

1. Class-wide correction, fresh inputs, no repair of any frozen scene: from
   the accepted c50 helper, produce a new candidate whose registered edits add
   the disabled early return to `ordinary`, `frame`, `initial` and
   `ordinary_request` before any TLS or bookkeeping, and enumerate every
   public item of the probe module with a per-item classification and anchor —
   source-route entry (must return before all state when disabled, with a
   held-borrow control), declared lookup consumer (same requirement), fixture
   scaffolding (exempt only with an anchor proving it is called solely by
   probe-aware fixtures), or internal/private — with zero unknown. The four
   lookups are repaired, not reclassified; if the enumeration finds any other
   unguarded entry of the same class, it is repaired the same way. No
   maintained test, scanner, frozen tree or accepted control is edited, and no
   maintained or production source changes.
2. Re-render and audit: propagate the corrected helper to the 114 sources and
   170 copies and the three J source routes with the same registered edits;
   audit every render independently (byte inverse and whole AST, fixed bodies
   and attributes exact); the 446 inherited controls plus the four new ones
   must be 450/0/0/0 in hash and nohash at eight and one on the same binary,
   with the nohash draw now run.
3. Re-run the affected evidence — the 848 main draws / 230 builds, the 72 J
   draws, the 48 component draws and the 640 fresh scope draws — after
   establishing by exact hash identity which sources the correction changed,
   and reuse the unchanged 400 raw draws only by exact hash identity, with
   every declaration frozen before execution and every expected set derived
   from the accepted declarations, never from observed execution.
4. Then complete the bar: the full 134/4,422/133 closure with the
   affected-source per-form matrices and every moved-duty caller, the 151-plan
   generator with duplicate/overlap/unknown-anchor rejection, the two fresh
   94-target/188-postimage roots, the six-column 94-row schema with all 151
   `source_schema` cells, the byte/AST/omission and disabled-parity audits,
   the ordered standing replay, `cargo fmt --check`, both workspace modes with
   the seven skips and the unfiltered `suspension_oracle`/`typed_slot_census`
   at failed=0/filtered=0, and the switched same-binary checks.
5. Under the remaining 14,070.369583264994 s focused envelope and the assigned
   6,000 s workspace envelope, repairing no frozen tree, production or
   maintained source, and stopping before starting any required group that
   cannot fit and reporting the exact remainder. No campaign, epoch,
   acceptance or CM5.

## Continuation 52 ruling (director) — the four ordered lookup guards are delivered and green at 450/0/0/0 in both feature modes at eight and one, the 144-item public census is zero-unknown but incomplete in its fixture-call anchors, and the first further correction stops under Q28 because a literal leading return in `pause` breaks 70 unchanged disabled controls at eight and one on one binary: `pause` is both a runtime-reached entry and a fixture primitive the frozen contract requires to record, so the c50 boundary/private core split is the repair; the four-lookup repair is accepted with focused scratch credit only, the class rule in the c52 order is withdrawn and replaced by the call-path rule, the complete bar remains unexecuted, and continuation 53 runs under the remaining 13,579.116278609985 s.

Continuation 52 implemented the four ordered lookup guards (all 450 accepted
controls green in both feature modes at eight and one thread), completed a
zero-unknown public-item census of the probe surface (144 items), and then
stopped under Q28 when the first further correction — a literal leading
disabled return in `pause` — made a newly added held-borrow control pass but
broke 70 unchanged disabled controls at eight and one on one binary. The stop
is compliant and the finding is real: `pause` is the first item that is both
reached unconditionally from the copied runtime and required by the frozen
fixture to record while the probe is disabled, so the literal guard is
incompatible with the fixed contract by construction. This is not universal
impossibility and not a production regression: the c50 boundary/private-core
split resolves it without editing any fixed control. The class rule I wrote in
the c52 order was wrong — it classified by visibility and a leading-guard
test, not by call path and the fixture's own assertions — and this ruling
replaces it. The four-lookup repair is accepted with focused scratch credit;
the census is accepted as a surface classification with one disclosed missing
dimension; the propagation, the draws and the complete bar earn nothing.
Continuation 53 must complete the classification by call path, correct `pause`
and every other runtime-reached entry by the c50 pattern, then re-render,
re-audit and re-run the affected evidence, and then finish the bar under the
remaining 13,579.116278609985 s.

## 1. Run record

`cdx` ran as unit `brief-742-cdx-20261118-122031-1702880`, rc 0, peak
983,846,912 B, oom 0 / oom_kill 0; log 3,584,285 B, sha256
`0890b27308de784a1b36170aebe13f66094a9b64f34bf767a3fbf25dceacd1be`; no
`.git-attempts` file. The report is STOP: 294 changes (292 added, 2 modified —
`.temp/brief-742/PROGRESS.md` and `.temp/brief-742/report.json` — 0 deleted),
all under `.temp/brief-742`; 20 checks (12 PASS, 2 FLAGGED, 3 FAIL, 3 NA),
114 citations verified, 0 questions, 1 deviation, 0 disclosures. I ran
`check --against-git` myself: the 294 scope problems are all `.temp`-ignored
paths (the known artifact), and the only tracked-tree differences are the 23
pre-existing porcelain entries, unchanged at HEAD `7c6e10fd`. The expanded
run set is 40 receipts. The arithmetic is exact: 376.25330465500883 s measured
plus 35.0 s conservatively charged read-only inspections = 411.25330465500883 s
of receipts, plus 20.0 s bookkeeping (40 × 0.5) and the fixed 60 s reserve =
491.25330465500883 s charged of the 14,070.369583264994 s envelope, leaving
13,579.116278609985 s; CM4's share stands at 46,920.88372139001 of 60,500 s;
workspace 0 of 6,000; the 1,440 s verification and the second 6,000 s
workspace stay held; no envelope overrun and no allocation stop. The two
FLAGGED items are the instrument-construction disclosure (lookup auditor v1
matched a private `frame` homonym; the surface enumerator retired an E0425
typo and an E0599 cached-feature mismatch before v3 with span-enabled rlibs;
no candidate or fixed control was edited) and the inspection accounting (an
earlier preflight count of 43 is superseded as an unverified estimate and
35 s is charged conservatively instead; every bounded work command has its own
receipt). Every cargo-free build is a bounded `rustc` invocation at the
declared path; the ledger holds no `CARGO_TARGET_DIR`, no codegen or
optimization override, no git and no network command, all cwds the repo root
where recorded, and the inherited 16 G/no-swap scope with oom 0.

## 2. Accepted: the four lookup guards close the c51 finding

The c52 helper is the accepted c50 helper with exactly four registered edits,
one leading disabled return in each of `ordinary`, `frame`, `initial` and
`ordinary_request`; a byte diff against the c50 helper shows only those four
insertions, and the fixture render is the c51 450-control candidate with the
same four insertions in both embedded copies (the enabled copy and the
`disabled` copy) and no other change. The independent syn auditor v2 asserts
each guard is the first statement of its function, erases the four, and
compares the whole-file token stream to the base exactly, while requiring
every one of the 187 helper tests and all 450 fixture controls to be
token-identical to the base; it reports 1 copy in the helper and 2 in the
fixture. The v1 attempt failed on a private `frame` homonym before any
candidate was run; v2 restricts to public functions and is non-vacuous.
The matrix is green: 450 passed / 0 failed / 0 ignored / 0 filtered, exit 0,
in hash and nohash at eight and one thread. I copied the frozen fixture and
rebuilt it with the declared rustc invocation and the maintained `librum_core`
rlib; my first attempt reproduced 269/181 because I omitted the declared
selector `RUM_CM740_OMIT=C3.evaluate_call.unstable_head`, and with the
declared environment my binaries reproduce 450/0/0/0 in both modes at eight
and one. My binary hashes differ from the worker's because the source path is
embedded in panic locations (relative versus absolute), and the counts and
failure sets are exact. This closes the c51 open item: the four are repaired,
not reclassified, and every fixed control and attribute stays byte-exact.

## 3. Accepted with limits: the 144-item public census

`public-surface-classification.json` enumerates 144 items — 99 `pub` and 45
`pub(super)`, 91 exposed and 53 internal — with definition text, hash and line
anchors, declarations and obligation counts: 85 source-route items (20
non-callable declarations, two value-only constructors, six existing leading
guards and 57 further callable duties), four declared lookup consumers, two
fixture-only observations and 53 internal/private items, unknown 0. The
copied-tree reference scan covers 1,623 Rust files with zero unknown imports,
and the census states its own limits (method evidence is lexical, not type
resolution; no older copy is declared current; no arbitrary dispatch or
complete inertness is claimed). I checked the enumeration against the helper
text: the item kinds and visibility counts close to the file's explicit
surface. The census is accepted as a surface classification, but it is
incomplete in the one dimension the repair decision needs: it records a
`fixture_call_anchors` list per item and leaves it empty for `pause` even
though the frozen fixture calls `pause` in both copies, so the 57-duty rule
was derived without the fixture's own contract. The worker's later static
conflict analysis found the collision by scanning the rendered fixture; the
census itself did not. Continuation 53 must close that dimension.

## 4. Compliance of the Q28 stop

The pause candidate (the four-guard helper plus one leading
`if !ENABLED { return; }` in `pause`, and the 450-control fixture plus one
additive held-borrow control) was declared `451/0/0/0` in `pause-plan.json`,
frozen at 12:37:51.665 with the byte-inverse edits; a static conflict record
had already been written at 12:35:59.768 naming the fixed control and its
expected value. The build ran at 12:38:07.179 and the full runs at
12:38:07.425 (t8) and 12:38:07.720 (t1) on the one binary `8128c3c2…`, both
exit 101 at 381 passed / 70 failed / 0 ignored / 0 filtered with exactly the
70 disabled-module controls in the declaration and no other difference; the
held-borrow control passed 1/0/0/450 in both modes at eight and one; the 223
enabled controls all passed; the nohash full draw is NOT_RUN after the
same-binary stop; no candidate, expectation, control, scanner or frozen tree
was repaired. I copied the frozen fixture, rebuilt it with the declared
invocation and selector, and reproduced 381/70/0/0 at eight and one in hash
and nohash with exactly the declared 70 names, and the held-borrow control at
1/0/0/450; the fixed control reports `left: (0, 0), right: (1, 0)`. The
AST auditor parses base and candidate, requires every base test to be
token-identical, the additive module to be the only new test, erases the
module and the two `pause` guards, and compares the whole file to the base
exactly; it is non-vacuous. Q28 is satisfied: compiled candidate, declared
expectation, same-binary one-thread reproduction, no repair.

## 5. The finding: `pause` is both a runtime entry and a fixture primitive

The frozen fixture asserts, in both copies, `start(); let g = enter(ROW);
pause("finish"); g.input_return(&request()); record();
assert_eq!(counts(), (1, u64::from(ENABLED)))`. In the `disabled` copy
`ENABLED` is false: the control requires one pause occurrence from a direct
`pause` call and zero initial returns from the boundary call. The copied
runtime calls `crate::cm740_probe::pause(&request.prompt)` unconditionally at
`crates/rum-runtime/src/runtime/builtins_input.rs:248` (immediately followed by
`queue_transport::pause_installed()`), and `pause` calls `SELECTED.with(...)`
then `arm`, whose first action is `LEDGER.with(|cell| cell.borrow_mut())`
before `producer_owner` can return early. So in a disabled build the ordinary
input-request path reaches TLS and a mutable ledger borrow, and a held borrow
panics; that is the c49-class default-path violation. A literal leading return
in `pause` fixes the runtime path and the held-borrow duty, but it also fires
for the fixture's direct call in the disabled copy and removes the occurrence
the fixed contract requires. The two duties cannot both be carried by one
function. The c50 accepted pattern already answers this: keep the recording
primitive with its exact name and body for the fixed fixture, add a public
boundary whose first statement returns before any state when disabled, and
redirect the runtime callers to it — exactly as `SuspendedEvaluation::append`
enters through `source_append` while the six protocol primitives stay with
the controls. That is a repair of the runtime path, not a reclassification of
`pause`: the declaration (152 frozen obligations, "actual selected producer
entry at InputRequired") stays, the fixture call sites stay, and the new
boundary inherits the entry duty with its own held-borrow control. The finding
is ruled a scratch instrument-surface conflict tied to the frozen fixture
contract: not a production regression, not a default-path cost in the
maintained runtime (the copied tree is scratch), not a semantic catch, not an
inventory ordering artifact, and no universal impossibility. The class rule in
the c52 order — "if the enumeration finds any other unguarded entry of the
same class, it is repaired the same way" — is withdrawn and replaced by the
call-path rule below. That rule was mine and it was wrong; the worker's stop
falsified it at the cheapest possible point (one edit, one additive control,
no fixed control touched), and the finding earns no correction credit of its
own.

## 6. Not delivered

Nothing beyond the four-lookup repair and the census is delivered, and no
later work earns anything: the pause correction, the class-wide correction,
the re-render of the 114 sources / 170 copies and the three J routes, the
re-audits, the 848/72/48/640 draws and the 400 raw draws (no reuse claim), the
134-boundary/4,422-obligation/133-carrier closure with the per-form matrices
and every moved-duty caller, the 151-plan generator with
duplicate/overlap/unknown-anchor rejection, the two fresh 94-target/188-
postimage roots, the six-column 94-row schema with all 151 `source_schema`
cells, the byte/AST/omission and disabled-parity audits, the ordered standing
replay, `cargo fmt --check`, both workspace modes with the seven skips and the
unfiltered `suspension_oracle`/`typed_slot_census` at failed=0/filtered=0, and
the switched same-binary checks. Propagation, standing replay and workspace
are NOT_RUN; workspace 0 of 6,000; complete plans 0 of 151; complete
calibration runs 0 of 172; emitted overlay paths 0; the schema is unchanged
(`f9604852…`, `new_source_schema=null`, `complete_plan_hash=null`), all 94
admitted paths are `not_emitted`, and the 90 preimages / 180 postimages /
151 cells authenticate.

## 7. Allocation and continuity

No new allocation is made. Continuation 53 runs under continuation 52's
remaining focused envelope of 13,579.116278609985 s and the assigned 6,000 s
workspace envelope (spent 0); CM4's share stays 60,500 s with
46,920.88372139001 s committed, 743–746 retain their 40,500 s, and the 1,440 s
verification and the second 6,000 s workspace stay held. This is the carried
pool's remainder at a new gate, not an extension; the bar, the stop-loss and
the authorization limits are unchanged.

## 8. Freeze and entry

The stopped scene is 135 files / 185,421,600 bytes in `pause-contract-stop/`,
frozen at 12:41:26 with the candidate fixture, the paused helper render, the
auditors, the binaries, the plan, the edits, the static conflict record, the
stop declaration and both run logs; I rehashed both sides of every entry (270
hashes) with zero bad or missing. The entry authenticates 5,400 maintained,
590 design and 228,671 accepted files. I verified the carry by exact hash
identity against the c51 entry: all 219,053 accepted hashes are unchanged and
the 9,618 new ones are every file under `continuation-51`
(6,436,751,948 bytes, hashed by me, zero bad), the one extra c51 added path
being the excluded top-level `report-continuation-51.json`; the 590 design
hashes are unchanged; the only three maintained deltas are my own committed
documents (`TODO.md`, brief-623, brief-742) whose working-tree hashes equal
their HEAD `7c6e10fd` blobs, and I rehashed all 5,400 maintained and all 590
design entries with zero bad. The prior scenes hold 23 trees / 104,523
entries: I rehashed the whole c51 scene (4,798 entries, both sides) and
400-entry samples from the c50 and c49 manifests with zero bad, and the 116
director transients are byte-identical to the c51 record (108 missing originals
and 8 changed, zero unexplained). The two known open `freeze-stop.log` drifts
stand unrepaired. The pre-existing layer is unchanged at HEAD `7c6e10fd`
(23 porcelain entries); the maintained test build's three dead-code warnings
are pre-existing and the inherited `unused_must_use` at `cm740_probe.rs:1319`
remains scratch, to be settled by the complete overlay's audit.

## 9. Continuation 53 ordered

The class rule is: the duty is on the **runtime-reached path**, and the
frozen fixture's own assertions define what the fixture requires; an item's
mechanism is chosen by call path, not by visibility.

1. Complete the classification with fixture anchors. Extend the 144-item
   census so that every item carries its definition anchor, every runtime
   (copied-crate) call anchor, every call anchor inside the **entire frozen
   450-control fixture** (both the enabled and the `disabled` copy), its
   declaration and obligation count, and for each fixture call the disabled-arm
   value the surrounding control asserts, with zero unknown. Classify each
   item's mechanism: a runtime-reached item whose disabled recording the
   fixture does not require takes a leading disabled return inside the entry
   (the four-lookup pattern); a runtime-reached item whose disabled recording
   the fixture requires (starting with `pause`, and including
   `queue_transport::pause_installed` in the same runtime function) takes the
   c50 boundary/private-core split — a new public boundary that returns before
   any state when disabled, the runtime callers redirected to it, and the
   recording primitive keeping its exact name and body; a fixture-only item
   takes no leading guard and keeps its anchors; non-callable declarations and
   stateless constructors carry no duty. The fixture's assertions are the
   contract: no deliberately red candidate may be used as the classifier, no
   fixed control, expectation or declaration is edited, and every moved duty
   lists its callers. Any item whose classification is ambiguous is reported,
   not guessed.
2. Correct `pause` and every other runtime-reached entry by the c50 pattern,
   adding one additive held-borrow control per corrected entry and completing
   the held-borrow evidence for the six existing boundaries. The 450 fixed
   controls stay byte-exact and green in both modes at eight and one, the
   additive controls pass, and the disabled full-runtime arm stays at its
   accepted 1433/0/5/2 at eight and one on one binary, with the nohash full
   draw run.
3. Re-render and audit: propagate the corrected helper to the 114 sources /
   170 copies and the three J source routes with registered edits; audit every
   render independently (byte inverse, whole AST, fixed bodies and attributes,
   call-site redirects); disclose every copied-source delta with an exact diff
   and a structural reason.
4. Re-run the affected evidence — the 848 main draws / 230 builds, the 72 J
   draws, the 48 component draws and the 640 fresh scope draws — after
   establishing by exact hash identity which sources the correction changed,
   and treat the 400 raw draws only by exact hash identity; every declaration
   is frozen before execution and every expected set is derived from immutable
   accepted inputs, never from observed execution.
5. Then complete the bar: the full 134-boundary/4,422-obligation/133-carrier
   execution closure with the affected-source per-form matrices and every
   moved-duty caller, the 151-plan generator with duplicate/overlap/
   unknown-anchor rejection, the two fresh 94-target/188-postimage roots, the
   six-column 94-row schema with all 151 `source_schema` cells, the
   byte/AST/omission and disabled-parity audits, the ordered standing replay,
   `cargo fmt --check`, both workspace modes with the seven skips and the
   unfiltered `suspension_oracle`/`typed_slot_census` at failed=0/filtered=0,
   and the switched same-binary checks.
6. Under the remaining 13,579.116278609985 s focused envelope and the assigned
   6,000 s workspace envelope, repairing no frozen tree, production or
   maintained source, and stopping before starting any required group that
   cannot fit and reporting the exact remainder. No campaign, epoch,
   acceptance or CM5.

## Continuation 53 ruling (director) — the call-path census closes the c52 gap (144 items, 2,078 fixture anchors in both fixture copies, 1,024 assertion anchors, 42 runtime anchors, zero unknown), the 13 new public boundaries and four leading returns hold the accepted 450-control fixture byte-exact at 496/0/0/0 in both feature modes at eight and one, and the runtime render reaches 1433/0/5/2 at eight and one; the propagation then stops statically because the P0558 extracted source's frozen disabled assertions require the producer's `enter_selected` recording while the redirect returns `Guard(None, None, None)` before state — and the 13 new privacies would also make the witness calls fail to compile (E0603). The ruling separates a witness render (recording seam retained, never installed) from the runtime render (privacies and redirects installed), re-opens the four leading returns against the union of frozen fixtures, and orders continuation 54 to complete the witness classification and the bar under the remaining 12,856.178777779012 s.

Continuation 53 completed the call-path census (closing the c52 fixture-anchor
gap), installed the 13 new public boundaries and four leading disabled returns
for the 17 runtime-called items, and held the accepted matrix at 496/0/0/0 in
both modes at eight and one while the disabled full-runtime arm stayed at
1433/0/5/2. It then stopped — before rendering or executing any propagation
candidate — on a static ambiguity that my c53 order did not cover: the P0558
extracted source's frozen disabled assertions require the producer's
`enter_selected` recording, while the runtime redirect returns
`Guard(None, None, None)` before state. The stop is compliant: no candidate was
rendered, no deliberately red classifier ran, no expectation or fixed control
changed. The finding is real, and the classification in it is incomplete in the
direction that matters — the 450-control fixture is not the only frozen
authority; the 114 extracted sources are frozen witnesses too, and their
disabled contracts were never consulted. I also found a second, mechanical
blocker the worker did not name: the 13 new privacies make the extracted
fixtures' own primitive calls fail to compile (E0603), because those calls sit
outside the probe module. The ruling below resolves both with one principle: a
source has a **witness render** and a **runtime render**, and each frozen
fixture's own assertions choose what its render must do. The census, the 17
corrections as 450-control evidence, and the runtime render are accepted with
focused scratch credit; the four leading returns are reopened against the
witness family; the propagation, the draws and the complete bar earn nothing.

## 1. Run record

`cdx` ran as unit `brief-742-cdx-20261118-125833-1725633`, rc 0, peak
2,573,783,040 B, oom 0 / oom_kill 0; log 2,193,182 B, sha256
`13f87851a0bd8052b7b106b7500d256cb2dc4498f695243eeabf6ba0be08e2b6`; no
`.git-attempts` file. The report is STOP: 13,906 changes (13,904 added, 2
modified — `.temp/brief-742/PROGRESS.md` and `.temp/brief-742/report.json` — 0
deleted), all under `.temp/brief-742`; 29 checks (24 PASS, 1 FLAGGED, 2 FAIL, 2
NA), 172 citations verified, 1 question (`Q53-source-fixture-boundary`), 1
deviation, 0 disclosures. I ran `check --against-git` myself: 13,906 of the
13,938 scope problems are the `.temp`-ignored artifact ("in changes but
unchanged in the tree"), and the remaining 32 are the pre-existing tracked
modifications (the 23 porcelain entries, expanded through the untracked
directories) — no new tracked change, HEAD still `b39d7f8f`. The two FAILs are
the worker's own honest verdicts (`propagation_contract`: the static
ambiguity; `complete_bar`: NOT_RUN after the stop); the FLAGGED item is the
tooling-construction disclosure. The expanded run set is 50 receipts.

## 2. Accepted: the completed call-path census

`public-call-classification.json` re-enumerates the 144 explicit public items
over the copied tree and the **entire 450-control fixture** (both the `armed`
and the `disabled` copy): 2,078 fixture call anchors, 1,024 assertion anchors
and 42 runtime call anchors, unknown 0, with every item carrying its definition
anchor, its call anchors, its declaration and obligation count, and for each
fixture call the surrounding controls and assertions. `pause` now carries 134
fixture anchors in both copies (171 `armed` / 171 `disabled` control
references) and one runtime anchor (`builtins_input.rs:248`); `enter_selected`
carries 14 fixture and five runtime anchors. The mechanism counts close to 144:
31 non-callable declarations, 42 fixture-only items, 4 accepted leading
returns, 13 new public boundaries, 42 internal, 2 stateless constructors, 6
existing boundaries and 4 leading returns. This closes the c52 gap (the empty
`fixture_call_anchors` for `pause`) and is accepted as the surface
classification it declares itself to be. Its scope is the 450-control fixture,
not the extracted-source family; the stop shows why that distinction decides
the repair.

## 3. Accepted with limits: the 17 corrections and the 496 matrix

The c53 helper differs from the accepted c52 helper by exactly the declared
edits: 13 `pub fn` → `fn` privacies, 13 public boundaries (`source_pause`,
`source_enter_selected`, `source_installing`, `source_evaluation_owner`,
`source_pause_installed`, `source_observe_registration`,
`source_observe_join_result`, `source_join_owner`, `source_generation`,
`source_expect_registration`, `source_finish_registration`,
`source_expect_result`, `source_finish_result`), four leading
`if !ENABLED { return …; }` statements (`queue_transport::source_scope`,
`retire_root`, `retire_control`, `clear_registry`), and the additive
`#[cfg(any())]` control modules — no other change. The fixture render differs
from the c52 450-control fixture by the same edits in both copies: a byte diff
shows only the 26 visibility substitutions and the declared insertions, and
every fixed control body is untouched. The matrix is 450 fixed controls plus 23
additive held-borrow controls per copy (46 total, 496): **496/0/0/0, exit 0, in
hash and nohash at eight and one**, and I rebuilt the fixture myself and
reproduced all four draws. The independent syn auditor passes on the helper
(1 copy) and the fixture (2 copies): each boundary's first statement is the
disabled return, erasure restores the whole AST, primitive bodies and
attributes are exact, and all 450 fixed controls are token-identical.

The limit is decisive: the 450-control fixture is one authority, not the
authority. The 23 additive controls prove inertness on the paths that fixture
exercises; they do not prove that the 114 extracted sources' frozen disabled
assertions survive the same corrections. The stop is exactly a counterexample
in that second family. The 13 boundaries keep every primitive body byte-exact —
the c50 pattern proper — and are accepted as the runtime render's correction.
The four leading returns are **reopened**: a leading return changes the
primitive's body, so if any frozen witness requires that item's disabled
recording, the item must take the boundary/private-core split instead, and the
450-control matrix alone cannot choose between them.

## 4. Accepted with limits: the runtime render

The copied runtime render changes 10 files (nine copied sources plus the helper
copy) with 27 registered edits: function redirects (`pause`,
`enter_selected`, `evaluation_owner`, `installing`, `observe_join_result`,
`join_owner`, `pause_installed`), method redirects on `InstallGuard` and
`JoinOwner`, and compile-time `ENABLED` argument gates that keep the added
argument computation (locks, `Arc::as_ptr`) out of the disabled path. Nine
independent AST audits pass with whole-AST restoration, ordinary nodes exact,
fixed bodies and attributes exact, and the redirects counted per family; the
helper has its own boundary AST audit. The disabled full-runtime arm reproduces
**1433/0/5/2 at eight and one** on the one binary `a2fe2a83…` — I reran both
receipts at both thread counts myself and reproduced every count — with the
frozen 1,440-name set, the five ignored and the seven skips. One honest limit:
the "nohash" cargo command passes `--no-default-features`, but `rum-runtime`'s
default feature set is empty, so cargo legitimately reuses the same artifact
(the worker disclosed `runtime_same_artifact_both_cargo_modes=true`); the
nohash draw is a rerun, not an independent feature mode, and the independent
mode coverage lives in the fixture matrix. The worker's first driver failed on
a `fresh=false` demand for exactly this reason and was retired with a
nohash-only driver that authenticated the artifact; that is a construction
failure, retained and settled, not a candidate failure.

## 5. The stop: P0558's frozen disabled contract versus the redirect

The stopped source is `.temp/brief-742/continuation-51/append-propagation/022.rs`
(sha256 `fc4fc19a…`), a c37/51 extracted copy of the association-selector
operation. Its producer `resume_suspended_association_set_lvalue` (line 526)
calls `crate::cm740_probe::enter_selected("P0558")` at line 550; its fixture
leaf `evaluate` (line 34) calls `cm740_probe::pause("association-selector")` at
line 38 when `arm` is set; and the frozen test
`actual_continue_route_constructs_new_selected_occurrence` (lines 3099–3103)
pins `initial == usize::from(!cfg!(live))`, `after == 2 * usize::from(!cfg!(live))`,
`bindings == usize::from(!cfg!(live))` and `proofs == 2 * usize::from(!cfg!(live))`
— i.e. in the `disabled` build, one arm after `ready`, two after the resumed
episode, one construction binding and two proofs. The four frozen disabled
draws declare `16/0/0/0` (`operation-affected-P0558-disabled-{hash,nohash}-t{8,1}`,
c37). I read the mechanism through: `enter_selected` → `selected(row)` is true
under `RUM_CM740_OMIT=P0558` → `enter(row)` mints the activation and pushes it
on `arming_scopes`; `pause` arms for the `SELECTED` row and `arm`'s
`producer_owner` finds that activation and increments `next_occurrence`;
`count_arms` reads it. If the producer call is redirected to
`source_enter_selected`, `ENABLED=false` returns `Guard(None, None, None)`
before any state, no activation is pushed, `producer_owner` returns `None`,
`arm` returns before recording, and the frozen counts fall to 0 — the exact
contradiction the worker reported. The stop is static: no candidate was
rendered, no deliberately red classifier ran, and no expectation, control or
frozen tree was touched. The deviation (the complete ordered bar not
delivered) is the honest consequence.

## 6. The second blocker: the privacies break the witness calls

The 13 privacies are correct for the runtime render and invisible to the
450-control fixture, whose tests are children of the probe module. They are not
invisible to the extracted sources: there the probe is `pub mod cm740_probe`
at the crate root and the fixture leaves and runtime functions call it from
sibling modules. I confirmed the language rule with a synthetic two-module
check (E0603 on a sibling call to a private `fn`), and the propagation preflight
enumerates 148 external calls to the 13 privatized names across the 58 sources
(61 `pause` in 53 sources, 62 `enter_selected` in 28, 25 more across
`installing`, `evaluation_owner`, `observe_registration` and `join_owner`). A
witness render that simply inherits the privacies cannot compile, let alone
pass. So the render pair is not a stylistic choice; without it the family is
unbuildable.

## 7. Ruling on `Q53-source-fixture-boundary`

The extracted-source fixture at a dual-use call path is a **core-protocol
witness**. Its frozen assertions define the recording contract, and the
recording primitive keeps its exact name, body and attributes for it. The
runtime entry is separately bound by the boundary. The two are not the same
authority and must not be collapsed:

1. **Two renders, one source.** Each affected source is rendered twice as a
   declared pair: the **witness render** (compiled with `--test` for the
   848-family draws) and the **runtime render** (the installed copied-crate
   source). The witness render keeps every witnessed call on the recording
   primitive and keeps that primitive reachable to the frozen fixture with its
   exact body; the runtime render keeps the primitive private and applies
   every registered redirect to the boundary. The installed artifact is the
   runtime render; the witness render is never installed.
2. **The union of frozen fixtures chooses the mechanism.** An item takes the
   c50 boundary/private-core split whenever **any** frozen witness requires its
   disabled recording — not only the 450-control fixture. A leading disabled
   return is allowed only when no frozen witness observes that item's disabled
   recording. The four leading returns in this continuation are re-decided
   under this rule, and the 450 fixed controls stay byte-exact either way.
3. **P0558 is the declared dual-use site.** The witness render keeps
   `enter_selected("P0558")` at the producer; the frozen suite stays
   `16/0/0/0` and the counts stay 1/2/1/2. The runtime render redirects the
   same call site to `source_enter_selected`, whose first statement returns
   before any state. No assertion, control or declaration changes.
4. **The render pair is audited, not asserted.** For every pair: primitive
   bodies and attributes token-identical; fixed assertions byte-exact; the
   delta between the two renders exactly the registered visibility and
   redirect edits; the witness render never installed; the runtime render's
   redirects enumerated with zero unknown callers.
5. **Credit is split honestly.** The 848/72/48/640 source draws credit the
   **recording** contract in the witness render. The corrected entries'
   **inertness** is credited by the 496-control matrix, the nine runtime AST
   audits, the redirect census with zero unknown, and the 1433/0/5/2 arm. No
   single render is claimed to witness both, and every closure row must state
   which render its evidence came from.

This is a repair of the classification, not a reclassification of any item:
the primitives keep their names and bodies, the declaration (152 obligations)
and the frozen P0558 row stand, and no fixed expectation moves. It is also not
a weakening: the witness render is the only render that can hold the frozen
disabled contracts, and the runtime render is the only one that is inert.

## 8. Not delivered

Nothing beyond the census, the 17 corrections as 450-control evidence, and the
runtime render is delivered, and nothing else earns anything: the witness
classification, the re-decided mechanisms, the propagation to 114 sources /
170 copies and the three J routes with their independent audits, the affected
848/72/48/640 draws and the 400 raw draws (no reuse claim), the
134-boundary/4,422-obligation/133-carrier closure with the per-form matrices
and every moved-duty caller, the 151-plan generator with
duplicate/overlap/unknown-anchor rejection, the two 94-target/188-postimage
roots, the six-column 94-row schema with all 151 `source_schema` cells, the
byte/AST/omission and disabled-parity audits, the ordered standing replay,
`cargo fmt --check`, both workspace modes with the seven skips and the
unfiltered `suspension_oracle`/`typed_slot_census`, and the switched
same-binary checks are all open. Standing replay and workspace are NOT_RUN;
workspace 0 of 6,000; complete plans 0 of 151; complete calibration runs 0 of
172; emitted overlay paths 0; the schema is unchanged (`f9604852…`,
`new_source_schema=null`, `complete_plan_hash=null`), all 94 admitted paths are
`not_emitted`, and the 90 preimages / 180 postimages / 151 cells authenticate.

## 9. Allocation

No new allocation is made. 50 receipts charge 572.9375008309726 s measured
plus 65.0 s conservatively charged inspections (10 s entry + 55 s closing) plus
25.0 s bookkeeping (50 × 0.5) and the fixed 60 s reserve, so
**722.9375008309726 s charged** of the 13,579.116278609985 s envelope, leaving
**12,856.178777779012 s**; CM4's share stands at 47,643.821222220984 of 60,500
s, the assigned 6,000 s workspace is untouched, and the 1,440 s verification
and the second 6,000 s workspace stay held. The stop is a classification stop,
not an allocation stop (`allocation_stop=false`, `candidate_failure=false`).

## 10. Freeze and entry

The stopped scene is 6,944 files / 2,747,144,034 bytes in
`propagation-classification-stop/`, frozen at 13:39 with the helper, the
450-control fixture, the auditors, the binaries, the plans, the edits, the
census, the preflight and the stop declaration. Every frozen copy is
byte-exact (I rehashed all 6,944). My own rerun of the frozen runtime binaries
regenerated 56 transient compile-control/incremental originals under
`runtime-disabled` (54 deleted, two rewritten: `private-pass.stderr` and
`target/brief-538/compile-proof/library-build.stderr`) — the same
director-replay transient class declared at continuation 49; the frozen copies
preserve every byte and nothing is repaired. The entry authenticates 5,400
maintained, 590 design and 228,962 accepted files (`+291` = every file of the
c52 stage); I rehashed all 5,400 maintained, all 590 design and the 291 new
accepted hashes with zero bad, and every carried accepted hash is unchanged by
construction. The 24 prior trees hold 104,658 entries: I rehashed the whole
c52 scene (135 entries, both sides) and 400-entry samples of the c50 and c49
manifests with zero bad; the two known open `freeze-stop.log` drifts stand
unrepaired and the 116 director transients are unchanged. The pre-existing
layer is unchanged at HEAD `b39d7f8f` (23 porcelain entries); the inherited
`unused_must_use` at `cm740_probe.rs:1319` remains scratch.

## 11. Continuation 54 ordered

The rule is the one above: the extracted source is a witness; the runtime entry
is bound by the boundary; each has its own render; every frozen fixture's own
assertions choose what its render must do.

1. **Complete the witness classification with zero unknown.** For every
   external probe reference in the 58 sources (and any source the preflight
   missed), in every copy, record path, function, line, callee, the enclosing
   frozen test(s), the assertion anchor(s) and the disabled-arm value they pin,
   and the render role. Report any ambiguous site; do not guess and do not use
   a deliberately red candidate as the classifier.
2. **Re-decide the 17 mechanisms against the union of frozen fixtures.** Any
   item whose disabled recording a frozen witness requires takes the c50
   boundary/private-core split (primitive name, body and attributes exact;
   public boundary returns before any state; runtime callers redirected).
   A leading disabled return is allowed only when no frozen witness observes
   that item's disabled recording; revisit the four leading returns under this
   rule. The 450 fixed controls stay byte-exact and green in both modes at
   eight and one; only additive controls may be added.
3. **Render and audit the pair.** Witness render and runtime render per source,
   with the delta exactly the registered visibility and redirect edits;
   primitive bodies token-identical; fixed assertions byte-exact; the witness
   render never installed; every redirect's callers enumerated.
4. **P0558 first.** The witness keeps `enter_selected("P0558")` and the frozen
   `16/0/0/0` with counts 1/2/1/2; the runtime redirects the same site to
   `source_enter_selected`; both renders audited. No expectation change.
5. **Then the ordered bar.** Propagate to the 114 sources / 170 copies and the
   three J routes with independent per-render audits and a disclosure of every
   copied-source delta; re-run the affected 848/72/48/640 draws after
   establishing by exact hash identity which sources changed (the 400 raw
   draws only by identity); complete the 151-plan generator with rejection,
   the two 94-target/188-postimage roots, the six-column 94-row schema with 151
   cells, the byte/AST/omission and disabled-parity audits, the ordered
   standing replay, `cargo fmt --check`, both workspace modes with the seven
   skips and the unfiltered gates, and the switched same-binary checks, with
   the full 134/4,422/133-carrier closure and every moved-duty caller.
6. **Under the remaining 12,856.178777779012 s and the assigned 6,000 s
   workspace**, repairing no frozen tree, production or maintained source, and
   stopping before starting any required group that cannot fit and reporting
   the exact remainder. No campaign, epoch, acceptance or CM5.

## Continuation 54 ruling (director) — the witness classification is completed with zero unknown (2,693 references, 13,106 assertion anchors, 15 known unexercised over 117 sources / 173 copies), the 17 mechanisms are re-decided against the union of frozen fixtures (14 boundary/private-core splits, 3 leading returns, `source_scope` reopened to a split), all 117 witness/runtime pairs are rendered and independently audited (234 renders, 468 audits, witness never installed, runtime redirects exactly the caller census), P0558 leads at 16/0/0/0 in all eight draws, the refreshed 848/72/48/640 draws and 172 calibrations are exact, the 400 raw draws are reused only by exact hash identity, and both copied runtime placements reproduce 1433/0/5/2 in both modes at eight and one; the new source-ticket challenge then fails 502/6/0/0 in both copies at eight and one on the same binary — a captured ticket with mutable entry/owner fields lets entry-only retire another live queue, owner-only revoke its return, and both fields confirm it — so the ruling accepts the delivered work with focused witness/source-route credit, repairs the source-ticket consumer design before the bar, and orders continuation 55 under the remaining 8,826.28290035392 s.

Continuation 54 completed the work the c53 ruling ordered and then stopped on a
new compiled candidate control. It classified every external probe reference
over the 117 sources and 173 copies with zero unknown, re-decided the 17
mechanisms against the union of frozen fixtures (14 boundary/private-core
splits, 3 leading returns, with `source_scope` reopened to a split), rendered
and independently audited all 234 witness/runtime renders (468 audits), led
with P0558, propagated the pair discipline, re-ran the 848 / 72 / 48 / 640
draws and the 172 calibrations, and reused the 400 raw draws only by exact
hash identity. Both copied runtime placements reproduce 1433/0/5/2 in both
modes at eight and one. Then the new source-ticket challenge failed: the
captured `SourceScope` transports only mutable entry/owner fields, and
`JoinOwner::drop` retires the entry and revokes the owner without
authenticating either against the recorded producer, so a substituted ticket
can retire, revoke or confirm another live queue. The stop is compliant under
Q28 (502/6/0/0 at eight and one on the same binary, frozen without repair),
the finding is a genuine design-level defect of the source-ticket consumer
model rather than a production regression, and the complete bar still earns
nothing. The delivered work earns focused witness/source-route credit.

## 1. Run record

`cdx` ran as unit `brief-742-cdx-20261118-140525-1764879`, rc 0, peak
4,795,764,736 B, oom 0 / oom_kill 0; log 5,509,406 B, sha256
`0e32975ed5d7e3a0a67ae7055df369037d65e5b723e51fd10a5e3e168a9754e0`; the
worker header records `model: gpt-6-astra`, reasoning effort `max`. The
Codex config was changed to `gpt-6.2-astra` / `max` at 16:18, after this run
started, so continuation 54 ran on the previous Astra; continuation 55 will
pick up GPT-6.2-Astra max from the config. Report: STOP, 42,128 changes
(42,126 added, 2 modified — `PROGRESS.md` and `report.json` — 0 deleted, all
under `.temp/brief-742`), 31 PASS / 1 FLAGGED / 2 FAIL / 2 NA, 3,858
citations, 0 questions, 1 deviation, 0 disclosures; no git or network command
ran and no `.git-attempts` exists. `report.py check --against-git` reports
42,160 scope problems: 42,128 are the `.temp`-ignored additions and 32 are the
pre-existing tracked layer (the same 23 porcelain entries, expanded), so the
worker changed nothing outside its scratch root. HEAD stayed `56606749`.

## 2. Accepted: the witness classification

The classification is frozen at `witness-classification.json` and its freeze
log: 2,693 references = 2,446 explicit probe paths + 247 receiver methods;
13,106 assertion anchors; 15 known-unexercised references over five sources
(the only `FrameAbandon::abandon` callers are the wrapper and the adapter, with
no frozen caller of `FixtureFrame::abandon`; recorded as unexercised support
with no execution credit); unknown references 0; ambiguous probe targets 0; 247
method references resolved from declared receiver types with 0 unresolved. The
198 ordinary method edges are declared explicitly as `not_claimed`, not
silently resolved, and the file states three limits (witness classification,
not the complete 151-plan graph; reachability is possible call/teardown
participation, not an executed receipt; unexercised support earns no
execution credit). The 17 decisions are exactly 14 boundary/private-core
splits and 3 leading disabled returns; `queue_transport::source_scope` is
reopened as a split: the witness retains the recording body, and a new public
`source_capture_scope` returns `None` before any state when disabled. Four
source anchors across the two frozen component copies pin that decision
(`make`, `scene`, `join_result_actual_inner`, and the retained primitive). I
sampled 60 anchors across the 2,693 references and 13,106 assertions against
the sources: all exact. The classification completes the c52/c53 gap without
reclassifying any earlier control.

## 3. Accepted: the 117 pairs, 234 renders and 468 audits

The plan declares 117 sources (114 main + 3 J), 173 copies (170 + 3), 234
renders (117 witness + 117 runtime), zero unknown corrected-entry callers,
`witness_installed=false`, `complete_overlay=false`,
`observed_execution_used=false`. The independent auditor ran AST and
assertion audits twice per source (468 records, all exit 0):

- witness: every fixed test body token-identical to the source; 14 new
  boundaries restorable, 14 originals' visibility, 7 leading returns; 0
  external redirects in all 117; the per-source fixed tests are exact.
- runtime: the same restoration, and `audit.redirects == callers.counts` —
  every original external caller of the corrected entries redirected, with no
  extra redirect. The redirect census reconciles exactly: 53 preflight sources
  call a privatized name (5 of the 58 call only non-private helpers:
  `count_hits`, fixture helpers, `paired_eval`), plus the three J sources = 56
  runtime renders with redirects; the other 61 have none.
- deltas: 295 records; `maintained_delta=0` everywhere; 46 byte-inverse-exact
  helper/fixture records; 0 `complete_generated_overlay` claims. The helper
  differs from the accepted c52 helper by exactly the 23 declared edits
  (14 privacies, 3 leading returns, 6 boundary/control blocks); against the
  accepted c53 helper the delta is exactly 3 hunks (add `source_capture_scope`,
  make `source_scope` private, point its guard control at the boundary). The
  496-control fixture differs from c53 by the same three hunks in each of its
  two embedded copies — no fixed control body was touched. I rehashed the 595
  freeze entries and all 234 render hashes: zero mismatches.
- P0558 first: the witness and runtime AST audits pass (`external_redirects`
  0 and 2), the 197 assertion literals are byte-exact in both roles, and the
  eight draws are 16/0/0/0 (armed/disabled × hash/nohash × eight/one) on the
  frozen P0558 source `fc4fc19a…`.
- installation: the installed runtime audit covers nine copied sources, keeps
  the primitives private, redirects three `source_scope` capture sites, and
  reports `witness_render_installed=false`; the placement declares 94 admitted
  paths, 3 modified, 0 additional overlay paths, the scratch-only helper
  removed, and 0 scanner edits.

## 4. Accepted: the refreshed draws and both runtime arms

The groups are exact at the declared counts, and I verified them
independently: 1,800 recorded runs parsed from their logs against the command
ledger (0 mismatches, 362 distinct binaries rehashed exact), plus my own
re-runs from copies under `.temp/director/gate-c54/`:

- boundary 4 draws, 496/0/0/0 in both modes at eight and one (I reran all four);
- P0558 8 draws, 128/0/0/0 (I reran all eight);
- main 848 draws, 26,964/232/0/0 over 230 builds (sample rerun: 84/0);
- J 72 draws, 2,280/0/0/0 (sample rerun: 20/0);
- components 48 draws, 5,384/168/0/0 (sample rerun: 494/0);
- scope fresh 640 draws, 68,480/7,600/0/236,640 (sample rerun: 120/0/0/374);
- raw reuse 400 draws, 5,400/600/0/25,200, reused only by exact identity: I
  rehashed all 200 raw sources and 200 raw binaries and all still equal the
  c47 executed hashes; no source-path or overlay credit is claimed;
- calibrations 172 draws, 63,956/372/0/0: 42 variants, 374 controls, 15,708
  cells, 93 forced, 4 masked, 38 firing, 0 unknown, 86 source audits over 43
  sources / 86 copies with the fixed assertions byte-exact (sample rerun:
  374/0);
- runtime 4 draws and placement runtime 4 draws, 1,433/0/5/2 each
  (5,732/0/20/8 total) in hash and nohash at eight and one; I reran all eight
  draws from copies and reproduced 1,433 passed / 0 failed / 5 ignored /
  2 filtered.

The runtime numbers are the disabled full-runtime arm on the copied trees; the
`--no-default-features` build legitimately produces the same artifact because
`rum-runtime`'s default feature set is empty, as disclosed since c53. The
placement oracle compiled but did not execute; no execution credit is claimed
for it.

## 5. The stop: the source-ticket challenge

The candidate is `source-ticket-candidate.rs` (sha256 `a6513067…`, binary
`00b857db…`), declared 508/0/0/0 in `source-ticket-plan.json` before its first
run. It adds 12 controls to the 496 fixed; the 496 stay green and the 12 split
6 pass / 6 fail. Both `armed` and `disabled` copies fail the same three
classes, and the t8 and t1 runs are identical: **502 passed / 6 failed /
0 ignored / 0 filtered, exit 101**, on one binary:

- `another_live_entry_cannot_retire_the_other_queue`
- `another_live_owner_cannot_revoke_the_other_queue`
- `another_live_ticket_cannot_confirm_the_other_queue`

The observed effects are explicit in the printed scenes: substituting only the
entry removes the other queue's paused proof (`other_pending=false`); only the
owner revokes the other queue's return and leaves it unconfirmable
(`other_confirmed=false`, `other_rejected=false`); substituting both confirms
the other queue (`foreign_confirmed=true`). The valid two-owner scene and the
unknown-entry/owner challenges stay green, and the independent producer,
installation, pause and queue snapshots are unchanged at the substitution
seam, so the challenge changes only the captured copy. The mechanism is in the
helper: `SourceScope` carries only `entry: ProofId` and `owner: Activation`;
`join_owner` records that value; `join_result_kind` accepts any source whose
`entry_key(owner) == entry` (a consistent foreign pair passes), removes
`paused_at[entry]` and `paused[key]`, and confirms on the matching proof; and
`JoinOwner::drop` unconditionally calls
`revoke_unconfirmed_queue_return(&source.owner)` and removes
`paused_at[source.entry]` / `paused[key]` — neither consumer authenticates the
captured producer independently. The stop is compliant: the plan and source
were frozen before execution, the same binary reproduced at one thread, no
candidate, expectation, fixed control or frozen tree was repaired, and the
nohash draw is correctly NOT_RUN after the same-binary stop. No execution
credit extends to the three J source forms; the control scope is the helper
boundary in both witness copies.

## 6. Ruling on `Q54-source-ticket-authentication`

The finding is genuine and is a **design-level defect of the source-ticket
consumer model**, not a production regression: nothing in the maintained
runtime evaluates these probe paths, and no maintained or production file was
changed. It is not a reclassification and it is not an inventory-ordering
artifact: a ticket whose only transport is mutable entry/owner fields lets a
consumer act on a foreign live queue, which is exactly the duty the Q34/Q35
consumer contracts were meant to carry. The repair is ordered, not deferred:

1. the captured actual producer must be bound independently at capture
   (identity recorded at the producer, not reconstructed from the ticket's
   fields) and authenticated at every consumer before confirmation or
   retirement;
2. no invalid ticket may select a retirement, revocation or confirmation key
   belonging to another live queue, whether it substitutes the entry, the
   owner, or both;
3. the valid two-owner path and the unknown-entry/owner rejections must stay
   green, and the six failing controls become the acceptance for the repair;
4. the 496 fixed control bodies and the 12 additive control bodies stay
   byte-exact; new controls are additive only; no fixed expectation may be
   weakened.

The delivered c54 work earns focused witness/source-route credit only: the
classification, the 117/173 propagation, the renders and audits, the refreshed
draws, the reuse by identity, the calibrations and both runtime arms. The
source-ticket finding earns no new own catch and no admission (it is a defect
of the scratch instrument's design, not a catch in maintained code). The
complete bar earns nothing.

## 7. Not delivered

The full 134-boundary / 4,422-obligation / 133-carrier execution closure (this
stage reached 22 scope-actor boundaries / 373 obligations), the 151-plan
generator with duplicate/overlap/unknown-anchor rejection, the two fresh
94-target / 188-postimage roots (0 postimages emitted), the six-column 94-row
schema with all 151 `source_schema` cells (schema unchanged
`f9604852…`, new_source_schema=null, complete_plan_hash=null; all 94 admitted
paths `not_emitted`), the byte/AST/omission and disabled-parity audits of a
complete overlay, the ordered standing replay, `cargo fmt --check`, both
workspace modes with the seven skips and the unfiltered
`suspension_oracle`/`typed_slot_census`, and the switched same-binary checks
are all open. Standing replay and workspace are NOT_RUN; workspace spent 0 of
6,000 s. The complete overlay is not claimed (`complete_generated_overlay`
false in all 295 delta records).

## 8. Allocation

No new allocation. 2,247 receipts charge 4,029.895877425093 s = 2,646.395877425093 s
measured + 200.0 s conservatively charged inspections/setup + 1,123.5 s
bookkeeping + the fixed 60 s reserve, of the 12,856.178777779012 s envelope,
leaving **8,826.28290035392 s**. CM4's share becomes 51,673.71709964608 of
60,500 s; the assigned 6,000 s workspace is untouched (0/6,000); the 1,440 s
verification and the second 6,000 s workspace stay held. The stop is a
candidate failure, not an allocation limit (`allocation_stop=false`), and no
extension or held envelope was borrowed. I verified the ledger independently:
2,247 focused receipts, no `CARGO_TARGET_DIR`, no codegen or optimization
override, every cargo command offline/locked at `-j8`, the 16 G/no-swap scope
with oom 0/0, no git or network command, and the arithmetic exact to the
recorded decimals.

## 9. Freeze and entry

The stopped scene is 21,025 files / 15,433,042,841 bytes in
`source-ticket-stop/` (freeze PASS at 16:09, originals and copies exact). My
own rehash of all 21,025 entries confirms every frozen copy is byte-exact. My
own replay of the frozen runtime and placement binaries at 16:28 regenerated
**114 transient originals** under `.temp/brief-579/compile-controls` in the two
copied roots — 108 deleted and 6 rewritten (`rule-root-pass.stderr`,
`syntax-root-pass.stderr` under `placement-runtime`;
`missing-pass.stderr`, `private-pass.stderr`, `root-pass.stderr`,
`syntax-root-pass.stderr` under `runtime-disabled`) — the known
director-replay transient class, recorded and unrepaired; the frozen copies of
all 114 remain exact. Entry authenticates 5,400 maintained / 590 design /
242,865 accepted files (+13,903 = every file of the c53 stage; none under
continuation-54). I independently rehashed all 590 design files
(278,559,491 B) and all 242,865 accepted files (112,206,490,975 B): 0 missing,
0 bad. The 5,400 maintained files were not rehashed (git covers them; the
23 porcelain entries are unchanged and HEAD is `56606749`). The 25 prior
frozen trees hold 111,602 entries with the c53 scene (6,944) and its 56
director-replay transients recorded; the 116 inherited director transients
are unchanged and the two known `freeze-stop.log` drifts stand unrepaired.

## 10. Continuation 55 ordered

Continuation 55 repairs the source-ticket consumer design and then resumes the
bar, under the remaining 8,826.28290035392 s and the assigned 6,000 s
workspace:

1. **Inventory first, zero unknown.** Enumerate every `SourceScope` producer
   (capture points), every transport, and every consumer: `join_owner` mint,
   `join_result_kind` confirmation, `JoinOwner::drop` retirement,
   `identity_proofs::revoke_unconfirmed_queue_return`,
   `confirm_queue_return`, and every state-key write, in the helper and in all
   117 sources / 173 copies; record per item the enclosing frozen assertion,
   the disabled-arm value and the render role; report ambiguities rather than
   guessing.
2. **Design the independent binding.** Capture the actual producer's identity
   at capture through a path that does not reconstruct it from the ticket's
   mutable fields; authenticate at each consumer before any confirmation or
   retirement key is selected; cover entry-only, owner-only, both-fields and
   unknown-entry/owner invalid classes; keep the valid two-owner path
   unchanged.
3. **Implement and certify.** Additive controls per consumer and per invalid
   class; the six failing controls become the acceptance; the 496 fixed and
   the 12 additive control bodies stay byte-exact; no fixed expectation
   weakened; the witness/runtime pair discipline continues (witness never
   installed; runtime renders private and redirected; the delta exactly the
   registered visibility and redirect edits, audited per source).
4. **Re-earn the matrices.** Source-ticket matrix in both feature modes at
   eight and one; boundary 496/0/0/0 ×4; P0558 16/0/0/0 ×8; the 117/173 pairs
   re-rendered/re-audited if changed; runtime and placement 1,433/0/5/2 ×8;
   the affected 848/72/48/640 draws by exact hash identity, the 400 raw draws
   only by identity, and the 172 calibrations re-earned if their sources
   changed.
5. **Complete the bar.** Full 134/4,422/133 closure, the 151-plan generator
   with rejection, the two 94-target/188-postimage roots, the six-column
   94-row schema with 151 cells, the byte/AST/omission/disabled-parity audits,
   the ordered standing replay, `cargo fmt --check`, both workspace modes with
   the seven skips and the unfiltered gates, and the switched same-binary
   checks. Repair no frozen tree, production or maintained source; stop
   before starting any group that cannot fit and report the exact remainder.
   No campaign, epoch, acceptance or CM5; pair `(45,5,false)` Open, C2 118
   Residual + 0 Lost, 51 owners and five source duties Open, `CHANGELOG.md` and
   S10 blocked.

## Continuation 55 ruling (director) — the zero-unknown ticket inventory (237 inputs, 93,315 sites, 79,728 assertion anchors), the ticket binding for seven single-component classes with 578 controls green in both feature modes at eight and one (all six prior failures green), the 702 pair audits, the re-earned 848/72/48/640 draws (the 400 raw draws only by exact identity), the 172 calibrations and both runtime placements at 1433/0/5/2 are delivered and accepted with focused witness/source-route credit. The actual-completion whole-issued-ticket challenge then fails 0/2/0/51 in both thread counts on one binary (`9f88930f…`): the inner InputRequired confirms and a swallowed inner result revokes the other live producer's pending return, because `join_owner` receives no independent actual producer identity and the ticket authenticates only against its own embedded capture. The ruling is `Q55-actual-completion-issuer-authentication` (genuine scratch-consumer design defect, not a production regression); the complete bar stays unexecuted and continuation 56 is ordered under the remaining 5,171.014417151946 s.

## 1. Run record

- `cdx` unit `brief-742-cdx-20261118-164205-2132002`, rc 0, peak
  5,294,469,120 B, oom 0 / oom_kill 0; log 2,552,335 B, sha256
  `d989a9a62a5726d5ccfbe2af598e0e7bf400321b1803e79e32b62cbb539cf73d`;
  worker model `gpt-6.2-astra`, reasoning effort `max` (first run on the
  updated Codex default).
- Report STOP: 39,503 changes = 39,501 added + 2 modified (`PROGRESS.md`,
  `report.json`), 0 deleted, every path under `.temp/brief-742`; 35 PASS /
  1 FLAGGED / 2 FAIL / 10 NA; 3,701 citations verified; 0 questions, 1
  deviation (the incomplete bar), 0 disclosures; no `.git-attempts`, no git
  or network command in any of the 2,273 ledger receipts.
- `check --against-git`: 39,535 problems = 39,503 `.temp`-ignored change
  entries + the 32 pre-existing tracked-tree entries (the same 23 porcelain
  entries expanded); HEAD stands at `41af6ad0`.

## 2. Delivered and accepted (focused witness/source-route credit)

### 2.1 Zero-unknown ticket inventory

`ticket-inventory.json` classifies 237 inputs (234 witness/runtime renders =
117 sources x 2 roles, 56 of the sources carrying two copies, plus the helper,
the boundary fixture and the controls fixture rows, representing five physical
copies) into 93,315 sites and 79,728 assertion anchors with
`unknown=[]` and `ambiguous=[]`; `classification_sha256` is the byte digest
of the accepted c54 `witness-classification.json`. The state census reports
351 captures and 351 each of join-owner mints, join-result consumers, drop
consumers, confirm writers and revoke writers, all role sources anchored and
the fixture writes retained. I recomputed all 237 output hashes and the
classification link: exact, 93,315 sites and 79,728 anchors as declared.

### 2.2 The binding design and its controls

The helper records the producer at the lexical enter
(`EntryRecord.producer: Weak<GuardSource>`, `source_entry(&Rc<GuardSource>)`),
builds a `CapturedEntry { entry, producer: Rc<GuardSource> }` at the capture
point, carries it on `SourceScope`, and checks it at mint, result, drop,
confirmation and revocation. The fixture holds 508 fixed + 70 additive = 578
controls covering 7 ticket classes (Valid, OtherEntry, OtherOwner, Both,
UnknownEntry, UnknownOwner, OtherIssuer) x 5 consumers (Mint, Result,
DropOwner, Confirm, Revoke) x 2 copies. The six c54 failures are green in
both copies; the fixture is 578/0/0/0 in both modes at eight and one on four
draws (I reran all four). The independent audit reports 508 fixed tests
byte-exact, 70 additive tests, whole-AST restoration and 11 registered items
per copy; the helper audit reports 210 fixed tests byte-exact; the boundary
fixture still carries its 496 fixed tests byte-exact and runs 496/0/0/0 in
both modes at eight and one (I reran all four). The c54+12 control bodies
and the 496 boundary bodies are untouched: every removed line in both fixture
diffs is helper-internal.

### 2.3 Pairs, matrices and runtime arms

- 117 sources / 173 copies / 234 witness-runtime renders, 702 independent
  audits (117 sources x 3 kinds x 2 roles), every audit exit 0 and PASS; the
  audit results' source and candidate hashes reconcile with the frozen scene.
- Main 848 draws = 26,964/232/0/0 over 230 builds; J 72 = 2,280/0/0/0;
  components 48 = 5,384/168/0/0; scope fresh 640 = 68,480/7,600/0/236,640;
  the 400 raw draws = 5,400/600/0/25,200 reused only by exact source and
  binary identity; calibrations 172 = 63,956/372/0/0 (42 variants, 374
  controls, 15,708 cells, 93 forced, 4 masked, 38 firing, 0 unknown); runtime
  and placement 4 draws each at 1,433/0/5/2 in both modes at eight and one.
- I verified every recorded run from its log against the ledger: 1,854 runs,
  0 problems, 372 distinct binaries rehashed exact. My own reruns reproduced
  the signatures: source-ticket 578/0 x4, boundary 496/0 x4, P0558 128/0 over
  eight, runtime and placement 1,433/0/5/2 x8, and samples of main (84, 230),
  J (20), components (494), scope (120/0/0/374) and calibration (374).

### 2.4 The declared handoff classes

The handoff plan declares 96 draws over 3 forms (P0267, P0312, P0323) x 2
roles x 2 arms x 2 modes x 2 thread counts, frozen at 17:30:00.7, built at
17:30:03.3, executed after. The 48 declared-class draws all PASS at
14/0/0/39, and the witness and runtime renders audit at 21 fixed tests
byte-exact, 32 additive tests, 3 actual resume sites, read-only snapshots and
`complete_work` unmodified.

## 3. The stop

The first whole-issued-ticket item (P0267/form-1, witness, armed, hash) was
declared `[2,0,0,51]` and reproduced `[0,2,0,51]` at eight and one threads on
the one binary `9f88930f…` (source `05f90276…`), with
`otherissuedticket_input_required` and `otherissuedticket_swallowed_return`.
I reproduced both from a director copy: same binary, same two names, same
counts at t8 and t1. The failure: the inner `InputRequired` confirms the
other live producer's pending return (`confirmed_at` present, confirmed,
not revoked) and a swallowed inner result revokes it — `outer_after` goes
`(false,false,true,true,false)` and `(false,false,false,false,true)` while
the outer pending return was `Some(true)`; both producer GuardSource
allocations and queue records are unchanged at the seam, only the transported
ticket differs. No candidate, expectation or fixed control was repaired; the
remaining 46 handoff draws are NOT_RUN; the scene was frozen at 17:32; the
candidate source is unchanged since the declaration (only reporting scripts
and logs move after 17:30:21).

Mechanism, from the code: `join_owner(source: Option<SourceScope>)` receives
only the ticket. Its mint check `ticket.authentic(&ticket.capture)` compares
the ticket's capture against itself (`Rc::ptr_eq` and entry/owner equality are
trivially true), so the only real checks are `recorded()` against the records
and the ledger's current-run predicate. A fully issued ticket from another
live producer is internally consistent and passes; `JoinOwner` then stores
that capture and result/drop/confirm/revoke all reuse it. The actual producer
identity is never supplied at the completion seam, so the design's own claim
that authority is the original checked Guard is not enforced.

## 4. Ruling: `Q55-actual-completion-issuer-authentication`

The finding is genuine: a design-level defect of the scratch instrument's
source-ticket consumer model at the actual-object level — the same class as
Q54 one layer deeper. It is not a production regression, not a
reclassification, and it re-pins nothing; the 134/4,422/133 structural index
stands. The stop is compliant under Q28 (declared expectation, same-binary
reproduction at eight and one, no repair, scene frozen, remaining draws
unexecuted).

Credit: the inventory, the binding for the seven single-component classes,
the 578-control fixture, the 702 audits, the re-earned matrices, the reused
raw draws and the runtime/placement arms earn focused witness/source-route
credit; the whole-ticket consumer claim earns nothing until repaired; the
complete bar earns nothing.

Continuation 56 is ordered to:

1. Carry the actual producer identity from the lexical enter to the actual
   completion seam independently of the ticket argument — the actual
   `Guard`/`GuardSource` allocation, not any field of the transported ticket
   — and validate the ticket against it at mint.
2. Carry the actual capture through result, drop, confirmation and
   revocation; no consumer may select a state key from the transported ticket
   alone, and every consumer must repeat the actual-identity check before any
   confirmation or retirement effect.
3. Make the two frozen whole-ticket controls the acceptance (both green in
   every affected render), keep the 578 controls and the 496 fixed / 12 c54
   additive bodies byte-exact, and keep the 48 declared-class draws green.
4. Re-run the full 96 handoff draws (both roles, arms, modes, thread counts),
   the 578 controls, boundary, P0558, the 848/72/48/640 draws (the 400 raw
   draws only by exact identity), the 172 calibrations and both runtime
   placements, and re-audit every affected render.
5. Then complete the bar: the 134/4,422/133 execution closure, the 151-plan
   generator with rejection, the two 94-target/188-postimage roots, the
   six-column 94-row schema with 151 cells, the byte/AST/omission and
   disabled-parity audits, the ordered standing replay, `cargo fmt --check`,
   both workspace modes with the seven skips and the unfiltered
   `suspension_oracle`/`typed_slot_census`, and the switched same-binary
   checks.

Under the remaining 5,171.014417151946 s and the assigned 6,000 s workspace,
repairing no frozen tree, production or maintained source, and stopping
before starting any group that cannot fit and reporting the exact remainder.

## 5. Not delivered

The complete bar earns nothing: the full 134/4,422/133 closure is a
structural index only (152 forms, 3 selected J handoff forms); complete plans
0/151; the two 94-target/188-postimage roots have 0 postimages; the schema is
unchanged (`f9604852…`, `new_source_schema=null`, `complete_plan_hash=null`)
with all 94 admitted paths `not_emitted`; the complete-overlay audits, the
ordered standing replay, `cargo fmt --check`, both workspace modes and the
switched checks are unexecuted; standing replay and workspace are NOT_RUN,
workspace 0/6,000. The 90/180/151 cells authenticate.

## 6. Allocation

No new allocation. 2,273 focused receipts charge 3,655.268483201973 s
(2,258.768483201973 s measured + 200.0 s conservatively charged
inspections/setup + 1,136.5 s bookkeeping + the fixed 60 s reserve) of the
8,826.28290035392 s envelope, leaving **5,171.014417151946 s**. CM4's share
stands at 55,328.98558284805 of 60,500 s; the assigned 6,000 s workspace is
untouched (0/6,000); the 1,440 s verification and the second 6,000 s
workspace stay held; no extension or held envelope was borrowed.
`candidate_failure=true`, `allocation_stop=false`.

## 7. Freeze

The stopped scene is 19,455 files / 10,352,524,268 bytes in
`actual-handoff-stop/`; I rehashed all 19,455 pairs before my replays with
zero bad. My own replay of the frozen runtime and placement binaries then
regenerated 119 director-replay transients: 108 originals missing and 11
rewritten, 117 under the two roots' `.temp/brief-579/compile-controls` and 2
`target/brief-538/compile-proof/library-build.stderr`, all under
`placement-runtime` and `runtime-disabled`; the frozen copies of all 119 are
byte-exact. Recorded and unrepaired — the known director-replay transient
class. The 26 prior trees hold 132,627 entries with the c54 scene (21,025)
and its 114 recorded transients, the 116 inherited and 56 c53 transients
unchanged, and the two `freeze-stop.log` drifts (stages 32 and 33) still
open.

## 8. Entry

Entry authenticates 5,400 maintained / 590 design / 284,990 accepted files
(hashes taken at 16:46, before this continuation's outputs). I rehashed all
590 design files (278,559,491 B) and all 284,990 accepted files
(143,104,969,028 B) with zero missing and zero bad; the 42,125 accepted new
against the c54 entry are every continuation-54 output and none under
continuation-55. The only three maintained deltas are my own committed
documents (`TODO.md`, brief-623, brief-742) whose working-tree hashes equal
the HEAD `41af6ad0` blobs; the pre-existing layer is the same 23 porcelain
entries; no worker edit touched a maintained or production file.

## 9. What I verified and what stays open

Verified by me: the red (same binary, t8 and t1, same two names and counts),
the 578/496/P0558/runtime/placement signatures, six matrix samples, all 1,854
recorded runs against logs and ledger (0 problems, 372 binaries exact), the
237 inventory outputs and classification link, the 702 audits, the full
freeze, the full design and accepted entry hashes and the budget arithmetic.
The two retired v1 attempts are construction failures preserved as evidence
(the syn `visit-mut` feature and the close-evidence parser), not evidence
changes. The one FLAGGED check is the receipt-provenance note: the runtime
logs carry a nested one-test harness (1,438 outer tests plus two one-test
sub-runs), and the declared counts are the outer totals.

Open: everything in section 5, plus the Q55 repair itself. Continuation 56
carries both.

## 10. Process

Ruling written here; appended to brief-742 and brief-623 (Continuation 143),
TODO.md extended, `state.md` rebuilt, committed and pushed; clean-temp run;
dispatch log archived; memory updated; continuation 56 dispatched on
`gpt-6.2-astra` at effort max.

## Continuation 56 ruling (director) — the Q55 issuer repair (the actual capture held in a separate `CompletionIssuer` at the lexical enter, the ticket authenticated against it at the two-argument mint and carried through result, drop, confirmation and revocation) is delivered and accepted: both frozen whole-ticket controls are green in every affected render (96/96 handoff draws exact, 48 declared_classes at 14/0/0/39 and 48 issued_substitution at 2/0/0/51), the 598-control whole-consumers fixture (578 fixed + 20 additive) is green 598/0/0/0 in both modes at eight and one, the 578 and 496 fixtures stay byte-exact, seven issuer falsifiers are rejected by an independent AST auditor, and every matrix is re-earned (848/72/48/640, 400 raw by exact identity, 172 calibrations, P0558, both runtime placements at 1433/0/5/2) with 1,954 runs verified and 382 binaries hash-exact. The new append class then stops under Q28 on its first foreign-index control: P0267/form-1 (witness, armed, hash) was declared [1,0,0,30] and reproduced [0,1,0,30] at eight and one on one binary (`308aa8d9…`), because `JoinAppend::drop` removes the record the confirmation index points at without checking that the key belongs to the append registration or that the proof is bound to the actual source — the other live queue's confirmed payload disappears while its index and proof status survive and the ordered stream equals ordinary execution. The ruling is `Q56-append-confirmation-key-authentication` (genuine scratch-consumer design defect, not a production regression); the complete bar stays unexecuted, and this gate allocates a fresh 8,000 s envelope so continuation 57 runs under 9,187.649237259971 s.

Continuation 56 completed the ordered Q55 issuer repair, re-earned every
matrix and audit, and then stopped under Q28 on the first control of the new
append class: at P0267/form-1 (witness, armed, hash) `JoinAppend::drop`
removed the other live queue's confirmed payload through a corrupted
confirmation index, while that queue kept its index and proof status and the
ordered stream equalled ordinary execution. The stop is compliant and the
scene is frozen. This gate accepts the issuer repair and the re-earned
matrices with focused scratch credit, rules the finding
`Q56-append-confirmation-key-authentication`, and allocates a fresh focused
envelope for continuation 57.

## 1. Run record

`cdx` ran as unit `brief-742-cdx-20261118-175949-2481216`, rc 0, peak
2,501,341,184 B, oom 0 / oom_kill 0; log 2,336,954 B, sha256
`79dbd6f47f0dc048b511a2f7758b603c3cb278417e7804778f8ffad30336b78c`; the
worker model is `gpt-6.2-astra` at effort `max`. The report
(`report-continuation-56.json`, 15,779,597 B, sha256 `ccb7df84…`) is STOP:
39,664 changes (39,662 added, 2 modified — `PROGRESS.md` and `report.json` —
0 deleted, all under `.temp/brief-742`), 47 PASS / 4 FLAGGED / 2 FAIL / 9 NA,
3,940 citations verified, 0 questions, 1 deviation (the complete bar), 0
disclosures. The stop rule is Q28 with reason P0267/form-1 witness armed/hash
and the finding `PortBlocked::AppendRetirementConsumesForeignConfirmation`.
No git or network command ran and no `.git-attempts` exists. HEAD stayed
`d4b45685` with the same 23 porcelain entries.

## 2. Delivered and accepted (focused scratch credit)

### 2.1 The Q55 issuer repair

The helper (`cm740_probe.rs`, 240,648 → 242,342 B; six hunks, 96 diff lines)
implements the ordered design exactly. The lexical enter now mints both
parts of an issuance: `fn source_scope(guard: &Guard) -> Option<IssuedSource>`
returns `IssuedSource { capture: CapturedEntry, ticket: SourceScope }`, and
only this site can construct it. `CompletionIssuer(Option<CapturedEntry>)`
consumes the issuance with `retain`, keeps the actual capture and returns the
transported ticket. The completion seam takes the issuer by value:
`pub fn source_join_owner_at(issuer: CompletionIssuer, source:
Option<SourceScope>) -> Option<JoinOwner>`, and `join_owner_at` asserts
`ticket.authentic(actual)` — the ticket is authenticated against the issuer's
retained actual capture, never against itself. The old single-argument
`source_join_owner` now takes `Option<Infallible>`, so a transported ticket
cannot even be passed to it. `JoinOwner` keeps the actual capture
(`JoinOwner(ScopeId, Object, Option<CapturedEntry>)`) and result, drop,
confirmation and revocation reuse it.

### 2.2 The Q55 acceptance

Both frozen whole-ticket controls — `otherissuedticket_input_required` and
`otherissuedticket_swallowed_return` — are green in every affected render.
The handoff fixture's 96 draws are all exact: 48 `declared_classes` at
14/0/0/39 each and 48 `issued_substitution` at 2/0/0/51 each, over three
forms × two roles × two arms × two feature commands × thread counts eight and
one. The nested inner mint is refused with `PortBlocked::ActualSourceTicketMint`,
and the outer pending return survives `(indexed=true, present=true,
confirmed=false, revoked=false)`; neither the inner InputRequired nor a
swallowed inner result touches the other live producer.

### 2.3 The controls, fixtures and audits

The new whole-consumers fixture adds two modules and 20 additive tests (five
consumers × `Valid` / `OtherIssuedTicket` × two copies) on top of the 578
fixed controls and is green at **598/0/0/0** in hash and nohash at eight and
one. The 578 source-ticket fixture (four draws, 2,312/0/0/0) and the 496
boundary fixture (four draws, 1,984/0/0/0) are green; the c54 six failures
are green in both copies; the 578 source-ticket bodies, the 496 boundary and
12 c54 additive bodies stay byte-exact, and the AST audits report
`fixed_test_bodies_exact` =
578 / 496 / 210 with the whole prior AST restored. Seven isolated
one-node mutations of the positive witness (self-authenticated mint,
ticket-as-issuer, result/drop/confirmation/retirement using the ticket,
altered completion) are each rejected by the independent AST auditor with
exit 101; the positive witness and runtime audits PASS with
`separate_mint_arguments=true`, `consumer_checks_before_keys=true`, three
lexical sites and 53 fixed test bodies. I reran the append, whole-consumer,
witness-issuer and runtime-issuer audits myself and reproduced every PASS
line.

### 2.4 Re-earned matrices

main 848 = 26,964/232/0/0 over 230 builds; J 72 = 2,280; components 48 =
5,384/168; scope fresh 640 = 68,480/7,600/0/236,640; the 400 raw draws
reused only by exact identity (two raw sources and eight raw binaries; all
ten distinct paths and all 400 logs hash-exact); calibrations 172 =
63,956/372 (42 variants / 374 controls / 15,708 cells / 93 forced / 4
masked / 38 firing / 0 unknown) with 86 calibration audits; P0558 128/0 over
eight; both runtime placements 1433/0/5/2 in both modes at eight and one,
each placement's two feature commands selecting one binary; 702 independent
pair audits (117 sources × 3 kinds × 2 roles, all exit 0). The 117 sources /
173 copies / 234 renders are re-audited whole-AST-restored.

## 3. The stop

The append class is a new scratch fixture (`c56_append_controls`): 31 tests
per copy (21 fixed + 10 additive), three registered carriers
(`queue_transport::JoinAppend.0`, `State.appending`, `State.confirmed`), four
source boundaries and 12 source-form obligations. The plan declares 48 items
and 96 draws (three forms × two roles × two arms × two modes; per item 48
`counterparts` at 4/0/0/27 and 48 `foreign_index` at 1/0/0/30). The plan and
freeze are at 18:48:11.512/.540, the declaration at 18:48:11.630, the first
build at 18:48:14.207 and the first run at 18:48:14.396 — frozen before
execution. All 48 counterpart draws pass at 4/0/0/27. The first foreign-index
draw (P0267/form-1, witness, armed, hash; source `75523205…`, binary
`308aa8d9…`) was declared `[1,0,0,30]` and reproduced `[0,1,0,30]` at
18:48:32 at eight and one thread on that one binary; the driver froze the
scene at 18:49 with the remaining 46 foreign-index draws NOT_RUN.
`candidate_repair=false`, `expectation_repair=false`; no candidate,
expectation, control or frozen tree was repaired. I reproduced the red from a
director copy at both thread counts with an observation line byte-identical
to the worker's.

The mechanism is in `JoinAppend::drop`:

```rust
if let Some(id) = scope.install {
    if let Some(key) = state.confirmed_at.remove(&id) { state.confirmed.remove(&key); }
}
```

The drop removes whatever record the confirmation index points at, without
checking that the key's own install is `id` or that the proof at that key is
bound to the actual append's source. `ForeignIndex` rewrites
`confirmed_at[inner_install]` to the other live queue's key, so the inner
append's retirement removes the outer queue's confirmed payload. The
independent authority records stay unchanged — the outer keeps `indexed=true`,
`confirmed=true`, but `present=false` — and the ordered stream equals
ordinary execution. The four counterpart changes (`Valid`, `IdenticalIndex`,
`UnknownIndex`, `ForeignProof`) all pass, so the fixture is non-vacuous and
the failure is specific to the unauthenticated index. This is a genuine
design-level defect of the scratch consumer model — retirement by an
unauthenticated state selector, the same discipline class as Q54/Q55 one
layer deeper — not a production regression and not a reclassification.

## 4. Ruling: `Q56-append-confirmation-key-authentication`

No consumer may select, confirm, retire or remove any record through a state
index or any other transported or indirect selector without authenticating
the record's own registration identity and its independently bound source
against the actual append registration and the actual bound source. A
mismatch must leave every record untouched.

Continuation 57 is ordered to:

1. Inventory every state-key selection and record removal in the append path
   with zero unknown — `JoinAppend.0`, `State.appending`, `State.confirmed`,
   `State.confirmed_at`, `InstallId`, `QueueKey`, `PassProof` — each with its
   actual object, selector source, state effect, consumer (`join_append`,
   `join_queue`, `observe`, `finish_append_call`, `Drop`, `omit_join`,
   `retire`) and isolation falsifier.
2. Repair `JoinAppend::drop` so that the confirmation key is validated
   against the append's actual registration (`key.install` equals the
   append's own install) and its proof against the independently bound
   actual source before any payload is removed; a failing record leaves both
   the index and the payload untouched.
3. Add the falsifier: an independent AST mutation auditor that rejects the
   unauthenticated removal and its sibling mutations, and make the frozen
   foreign-index control the acceptance (green in every affected render),
   while the 96-draw append fixture (21 fixed + 10 additive per copy), the
   578/598/496 bodies and the 96 handoff draws stay exact.
4. Re-run the full 96 append draws, the 96 handoff draws, the 578/598/496
   fixtures, P0558, the 848/72/48/640 draws (the 400 raw draws only by exact
   identity), the 172 calibrations and both runtime placements, and re-audit
   every affected render.
5. Then complete the bar: the 134/4,422/133 execution closure, the 151-plan
   generator with rejection, the two 94-target/188-postimage roots, the
   six-column 94-row schema with 151 cells, the byte/AST/omission and
   disabled-parity audits, the ordered standing replay, `cargo fmt --check`,
   both workspace modes with the seven skips and the unfiltered
   `suspension_oracle`/`typed_slot_census`, and the switched same-binary
   checks.

Under the remaining envelope and the assigned 6,000 s workspace, repairing no
frozen tree, production or maintained source, and stopping before starting
any group that cannot fit and reporting the exact remainder.

## 5. Not delivered

The append consumer claim and the complete bar earn nothing. The 134/4,422/133
closure is a structural index only (152 forms), complete plans 0/151, the two
94-target/188-postimage roots have 0 postimages, the schema is unchanged
(`f9604852…`, `new_source_schema=null`, `complete_plan_hash=null`) with all 94
admitted paths `not_emitted`; the complete-overlay audits, the ordered
standing replay, `cargo fmt --check`, both workspace modes and the switched
checks are unexecuted; standing replay and workspace are NOT_RUN, workspace
0/6,000; the 90/180/151 cells authenticate.

## 6. Allocation

Continuation 56 charged 3,983.3651798919745 s (2,524.3651798919745 s measured
+ 200.0 s conservative inspections/setup + 1,199.0 s bookkeeping + the fixed
60 s reserve) of the 5,171.014417151946 s carry, leaving
**1,187.6492372599714 s**; CM4's share stands at 59,312.350762740025 of
60,500 s, the assigned 6,000 s workspace is untouched (0/6,000), and the
1,440 s verification and second 6,000 s workspace stay held.
`candidate_failure=true`, `allocation_stop=false`.

This gate allocates a fresh 8,000 s focused envelope from the project's
reserve: continuation 57 runs under **9,187.649237259971 s**, CM4's share
becomes 68,500 s, the stage reserve 109,000 s, and 743–746 keep their
40,500 s unchanged. The extension is an allocation from the project's
reserve, not a bar change: the bar, the stop-loss and the authorization
limits are unchanged.

## 7. Freeze

The stopped scene is 19,533 files / 10,394,003,758 bytes in
`continuation-stop/`. I rehashed all 19,533 pairs myself with zero bad. The
27 prior trees hold 152,082 entries with the c55 scene (19,455) and its 119
recorded transients — 108 originals missing and 11 rewritten, every frozen
copy exact; I rehashed all 119 records, the 11 present originals exact to
their recorded hashes and all 119 frozen copies exact; the 286 inherited
transients are unchanged and the two `freeze-stop.log` drifts (stages 32 and
33) stay open and unrepaired. Nothing was repaired. My own runs this gate
were the red and the signature greens from copies outside the scene, so the
scene was untouched by the gate (the re-verification after them stayed 0
bad).

## 8. Entry

Entry authenticates 5,400 maintained / 590 design / 324,490 accepted files
(hashes taken at 18:05, before this continuation's outputs). I rehashed all
590 design files (278,559,491 B) and all 324,490 accepted files
(163,846,610,573 B) with zero missing and zero bad. The 39,500 accepted new
against the c55 entry are the c55 continuation outputs. The only three
maintained deltas are my own committed documents (`TODO.md`, brief-623,
brief-742) whose working-tree hashes equal the HEAD `d4b45685` blobs; the
pre-existing layer is the same 23 porcelain entries; the worker changed no
maintained or production file (maintained_changes=0, production_changes=0,
new_own_catches=0).

## 9. What I verified and what stays open

Verified by me: the red (same binary, eight and one threads, byte-identical
observation), 1,954 recorded runs against their logs and the 2,398-receipt
ledger with zero problems and 382 binaries hash-exact, the group totals
(848/72/48/640, 400 raw, 172, 578, 598, 496, 128, 5,732 ×2, 768 handoff, 192
append), the four AST audit reruns, the signature greens (598/0, 578/0, the
append counterparts 4/0/0/27), all 702 pair audits and 86 calibration audits
exit 0, the plan and freeze timestamps, both c55 controls green in witness
and runtime, armed and disabled, hash and nohash, the raw-reuse identity, the
full freeze, the full design and accepted entry hashes and the budget
arithmetic. The four FLAGGED items are retained construction failures with
no evidence change: the first append-declaration script assertion, the 45 s
live-discovery timeout (72,711 calls; the same executable finished under
120 s), the first installed auditor's generic-caller rejection (the corrected
comparator passed the same nine sources), and the first J declaration's
shared-destructor match (retained separately and rebound). The `drift_label`
FLAGGED is a wording error — the 119 transients are c55 director-replay
transients, not "continuation54 entry" ones — with their paths, hashes and
count authenticated at entry and close.

Open: everything in section 5, plus the Q56 repair. Continuation 57 carries
both.

## 10. Process

Ruling written here; appended to brief-742 and brief-623 (Continuation 144),
TODO.md extended, `state.md` rebuilt, committed and pushed; clean-temp run;
dispatch log archived; memory updated; continuation 57 dispatched on
`gpt-6.2-astra` at effort max.

## Continuation 57 ruling (director) — the Q56 append-confirmation repair (the index proposes a candidate key, the append's actual registration supplies the installation, the independently recorded source supplies the confirmation, all reads precede removal or claim, and a mismatch leaves both aliases untouched) is delivered and accepted: all 96 append draws are exact (48 counterparts at 4/0/0/27 and 48 foreign-index draws at 1/0/0/30 over three forms x two roles x two arms x two modes x eight and one), the 96 handoff draws are exact (48 declared_classes at 14/0/0/39 and 48 issued_substitution at 2/0/0/51), the 578/598/496/210/53/31/374 bodies stay byte-exact, eleven independent AST falsifiers are rejected by name, and every matrix is re-earned (848/72/48/640, 400 raw by exact identity, 172 calibrations, P0558, both runtime placements at 1,433/0/5/2 per draw) with 2,497 receipts verified and 62 binaries hash-exact. The new class-wide registry-retirement falsifier then stops under Q28 on its first foreign-root control: P0267/form-1 (witness, armed, hash) was declared [1,0,0,42] and reproduced [0,1,0,42] at eight and one on one binary (`52620b90…`), because `retire_root` selects from the mutable transported `Installed` copy, so the inner teardown erases the unrelated live outer registration, its confirmation index and payload while its independent proof survives; the four counterparts pass and 94 declared draws are NOT_RUN. The ruling is `Q57-registry-retirement-root-authentication` (genuine scratch-consumer design defect, Q35's accepted carrier rule not yet carried into the teardown selector, not a production regression); the complete bar stays unexecuted, and this gate allocates a fresh 8,000 s envelope so continuation 58 runs under 12,993.017385754993 s.

Brief 742's continuation 57 ran `cdx` as unit
`brief-742-cdx-20261118-191100-2812115`, rc 0, and returned a compliant
Q28 STOP. The Q56 append-confirmation repair is delivered and accepted with
focused scratch credit: all 96 append draws are exact, the frozen
foreign-index control is green in every role, arm and mode, the eleven
independent AST falsifiers are rejected, and every matrix is re-earned. The
class-wide registry-retirement falsifier then fails on
`PortBlocked::RegistryRetirementSelectsForeignInstalledRoot`, ruled
`Q57-registry-retirement-root-authentication`, and this gate allocates a
fresh focused envelope for continuation 58.

## 1. Run record

`cdx` ran as unit `brief-742-cdx-20261118-191100-2812115`, rc 0, peak
2,582,536,192 B, oom 0 / oom_kill 0; log 2,242,652 B, sha256
`ec4dc07d10753e9686a96eb5d564fe9752546b54d3460932b284ee02307e8aca`; the
worker model is `gpt-6.2-astra` at effort `max`. The report
(`report-continuation-57.json`, 16,844,644 B, sha256 `21253a04…`) is STOP:
40,708 changes (40,706 added, 2 modified — `PROGRESS.md` and `report.json` —
0 deleted, all under `.temp/brief-742`), 36 PASS / 3 FLAGGED / 2 FAIL /
8 NA, 4,096 citations verified, 0 questions, 1 deviation (the complete
bar), 0 disclosures. The stop rule is Q28 with reason P0267/form-1
ForeignRoot, witness, armed, hash, and the finding
`PortBlocked::RegistryRetirementSelectsForeignInstalledRoot`. No git or
network command ran and no `.git-attempts` exists. HEAD stayed `261da953`
with the same 23 porcelain entries.

## 2. Delivered and accepted (focused scratch credit)

### 2.1 The Q56 append repair

The helper differs from the accepted c56 helper by exactly four hunks
(93 diff lines; 242,342 → 243,765 B, +1,423 B; the same repair appears in
both `cm740_probe.rs` and the rendered `append-witness.rs`):

- a new `confirmed_append(state, scope, namespace)` takes the installation
  from the append scope, requires `scope.namespace == Some(Some(namespace))`
  and `state.reverse[(generation, namespace)] == install`, requires the
  transported record's own proof, generation and namespace to match, and
  only then reads the index as a candidate key: `key.install == install`,
  `proof.installed == installed`, `key.source == SourceKey::pause(&proof.token)`,
  `authenticate(proof, QueuePause)` and
  `identity_proofs::confirmed_append_source(install.0, key.source,
  authenticated)`;
- `retire_append_confirmation` removes the index and the payload only after
  `confirmed_append` returns `Some`; a missing, mismatched, unknown or
  foreign record leaves both aliases untouched and skips the queue-proof
  retirement through that path (the proof still expires at the original
  generation/namespace teardown);
- `omit_join` (publication) uses the same authenticated lookup and only then
  `identity_proofs::claim`;
- every read precedes the claim or removal, and the lexical scope and
  observation still close normally.

This is exactly the ordered seam: the index proposes a key, the actual
append registration supplies the installation, and the independently
recorded source supplies the confirmation. The old one-step
`confirmed_at.remove(&id)` path cannot recur.

### 2.2 Acceptance

The new `c56_append_controls` fixture (21 fixed + 10 additive = 31 tests
per copy) is exact in all 96 draws: 48 counterpart draws at 4/0/0/27 and 48
foreign-index draws at 1/0/0/30, over three forms × two roles × two arms ×
two modes × thread counts eight and one. The c56 failing control
(`ForeignIndex`) is green in witness and runtime, armed and disabled, hash
and nohash. The 96 handoff draws are exact (48 `declared_classes` at
14/0/0/39 and 48 `issued_substitution` at 2/0/0/51). The fixed bodies are
byte-exact in every affected render — append 31, boundary 496, helper 210,
handoff 53 ×2, source-ticket 578, whole-consumers 598, calibration 374 —
with the whole prior AST restored.

The independent AST auditor rejects eleven isolated one-node weakenings of
the repair, each at its declared obligation with exit 101: unchecked
removal, key with another install, registration from the proof, foreign
source, wrong edge, record selector from the key, unconfirmed source,
remove-before-authentication, retire-on-mismatch, claim through a foreign
namespace, and claim before the proof. I read the falsifier construction
and the auditor: it compares the authorized edits with the old source,
forces the read/claim/removal order, and rejects each mutation by name.

### 2.3 Inventories

The append inventory is complete with zero unknown: 286 inputs, 42
consumers, 44 body variants, 18,648 bound functions, 299 member sites and
7 carriers. The wider candidate-selector census (54 consumers, 23,976 bound
functions, 57 distinct bodies, 125 selector sites) resolves every current
render's actual keys, readers and removals with unknown=0. The admitted
placement embeds the authenticated helper byte-for-byte.

### 2.4 Matrices

Every matrix is re-earned on the repaired helper: source-ticket 4 draws =
2,312/0/0/0; boundary 4 = 1,984/0/0/0; P0558 8 = 128/0/0/0; main 848 =
26,964/232/0/0; J 72 = 2,280/0/0/0; components 48 = 5,384/168/0/0; scope
fresh 640 = 68,480/7,600/0/236,640; the 400 raw draws reuse only by exact
identity (5,400/600/0/25,200; two sources, eight binaries, all logs
hash-exact); calibrations 172 = 63,956/372/0/0 over 42 variants / 374
controls / 15,708 cells; whole-consumers 4 = 2,392/0/0/0; both runtime
placements 4 draws each = 5,732/0/20/8 (1,433/0/5/2 per draw, one binary);
the 702 pair audits and 86 calibration audits all exit 0. The class-wide
retirement fixture adds 31 fixed + 12 additive tests; the three J forms
carry 12 source-form obligations and 144 declared draws.

## 3. The stop

### 3.1 Ordering

The retirement class is a new declared scratch fixture
(`queue_transport::State.installed`, group `queue_install`, control
`q35_second_live_install_preserves_first`). Its plan and freeze are at
19:58:55.513/.541, the declaration at 19:58:55.594, the eight builds from
19:58:58.374 to 19:59:15.460, the first runs after that and the stop at
19:59:17 — frozen before execution. The plan declares 3 forms × 2 roles ×
2 arms × 2 modes × 2 threads × 3 groups = 144 draws over the P0267, P0312
and P0323 forms, with the class declaration naming five forms
(P1470/P1471 declared but not rendered) and `full_execution=false`. All 48
counterpart draws pass at 4/0/0/39. The first foreign-root draw —
P0267/form-1, witness, armed, hash (source `8d4a60ee…`, binary
`52620b90…`) — was declared `[1,0,0,42]` and reproduced `[0,1,0,42]` at
eight and one thread on that one binary; the scene froze at 20:04:16 with
the remaining 94 draws NOT_RUN, `candidate_repair=false`,
`expectation_repair=false`, and nothing repaired. The post-stop
reproduction at 20:03:35 is byte-identical.

### 3.2 The failure

At P0267 the inner root's actual teardown erased the unrelated live outer
registration, its confirmation index and payload while its independently
registered proof stayed live. The observation is `registration=false,
reverse=true, indexed=false, payload=false, registered_source=true,
confirmed_source=(true,false,false,false), inner_registration_present=false`
against the expected `registration=true, reverse=true, indexed=true,
payload=true, registered_source=true, confirmed_source=(true,false,false,false),
inner_registration_present=false`. The armed run then publishes once
instead of twice and the ordered stream differs from ordinary execution.

### 3.3 The mechanism and the authority

`retire_root(root)` performs two independent operations:

    identity_proofs::retire_namespace(|n| n.0 == root);
    STATE.with(|cell| cell.borrow_mut().retire(|i| i.namespace.root == root));

The first selects from the immutable identity records and is correct. The
second selects from the mutable transported `Installed` copy. The control
rewrites only `State.installed[outer].namespace.root` to the actual inner
root; the inner teardown then matches the outer record, removes the outer
installation, and removes its confirmation index and payload by install id.
The reverse alias survives because the removal uses the mutated namespace
key, and the independent identity records survive — so the transport and
the authority disagree, and a live unrelated registration is destroyed.

This is the same discipline class as Q54, Q55 and Q56 one layer deeper: a
consumer selects a record for removal through a transported or mutable
selector without authenticating it against the independent actual
registration. Q35's accepted carrier rules already forbid it ("Invalid-
present transport rejects; it cannot select which other key to retire";
"unrelated owner's cleanup cannot clear it"; "Taking one key never clears
another"). It is a genuine design-level defect of the scratch consumer
model, not a production regression and not a reclassification.

## 4. Ruling: `Q57-registry-retirement-root-authentication`

No teardown may select, retire or remove a record through a transported or
mutable copy of a registration — its root, control, generation, namespace,
index or proof — without authenticating the selection against the
independently registered actual installation and its actual retiring
object. A transported record that disagrees with the independent
registration is not a retirement target: the actual inner object retires
only its own registration, and every unrelated live registration, its
reverse alias, confirmation aliases, payload and independent proof must
survive untouched.

Continuation 58 is ordered to:

1. Inventory every registry and teardown selector and removal with zero
   unknown across every render — `State::retire`, `retire_root`,
   `retire_control`, `clear_registry`, `InstallGuard::drop`,
   `JoinAppend::drop`, and every consumer of `State.installed`, `unpaused`,
   `reverse`, `sources`, `paused`, `paused_at`, `confirmed` and
   `confirmed_at` — each with its actual object, selector source, state
   effect, consumer and isolation falsifier, building on the accepted
   54-consumer / 125-selector-site census.
2. Repair the teardown selection so the survivors are derived from the
   independent actual registration records (or each transported record is
   authenticated against them) before any removal; a mismatched transported
   slot leaves all live registrations, aliases, confirmations and proofs
   untouched; execute the declared carrier calibration — collapse only the
   carrier, the named two-owner/second-insertion control must fail, all
   unrelated controls green.
3. Add the independent AST falsifier that rejects the unauthenticated
   selection and its sibling mutations, and make the frozen foreign-root
   control the acceptance — green in every affected render, all 144
   declared retirement draws including the 94 outstanding at eight and one,
   the 48 counterparts exact — while the 96 append draws, the 96 handoff
   draws and the 578/598/496/53/31/210/374 bodies stay exact.
4. Re-run every matrix: the 848/72/48/640 draws (the 400 raw draws only by
   exact identity), the 172 calibrations, P0558, both runtime placements,
   the whole-consumers, source-ticket and boundary fixtures, and re-audit
   every affected render.
5. Then complete the bar: the 134/4,422/133 execution closure, the
   151-plan generator with rejection, the two 94-target/188-postimage
   roots, the six-column 94-row schema with 151 cells, the byte/AST/
   omission and disabled-parity audits, the ordered standing replay,
   `cargo fmt --check`, both workspace modes with the seven skips and the
   unfiltered `suspension_oracle`/`typed_slot_census`, and the switched
   same-binary checks.

Under the allocated envelope and the assigned 6,000 s workspace, repairing
no frozen tree, production or maintained source, and stopping before
starting any group that cannot fit and reporting the exact remainder.

## 5. Not delivered

The shared-retirement-consumer claim and the complete bar earn nothing. The
134/4,422/133 closure is a structural index only (152 forms), complete
plans 0/151, the two 94-target/188-postimage roots have 0 postimages, the
schema is unchanged (`f9604852…`, `new_source_schema=null`,
`complete_plan_hash=null`) with all 94 admitted paths `not_emitted`; the
complete-overlay audits, the ordered standing replay, `cargo fmt --check`,
both workspace modes and the switched checks are unexecuted; standing
replay and workspace are NOT_RUN, workspace 0/6,000; the 90/180/151 cells
authenticate. The retirement class is executed only for 50 of its 144
declared draws, its declared carrier calibration is unexecuted, and the
P1470/P1471 forms are declared but not rendered.

## 6. Allocation

Continuation 57 charged 4,194.631851504979 s (2,686.131851504979 s measured
+ 200.0 s conservative inspections/setup + 1,248.5 s bookkeeping + the
fixed 60 s reserve) of the 9,187.649237259971 s envelope, leaving
**4,993.017385754993 s**; CM4's share stands at 63,506.982614245004 of
68,500 s, the assigned 6,000 s workspace is untouched (0/6,000), and the
1,440 s verification and second 6,000 s workspace stay held.
`candidate_failure=true`, `allocation_stop=false`.

This gate allocates a fresh 8,000 s focused envelope from the project's
reserve: continuation 58 runs under **12,993.017385754993 s**, CM4's share
becomes 76,500 s, the stage reserve 117,000 s, and 743–746 keep their
40,500 s unchanged. The extension is an allocation from the project's
reserve, not a bar change: the bar, the stop-loss and the authorization
limits are unchanged.

## 7. Freeze

The stopped scene is 20,344 files / 10,561,408,515 bytes in
`continuation-stop/`. I rehashed all 20,344 pairs myself with zero missing
and zero bad. The 28 prior trees hold 171,615 entries with the c56 scene
(19,533) and its 119 recorded transients; the 286 inherited and 119 c55
director-replay transients (405 records) are unchanged and unexplained=0;
the two `freeze-stop.log` drifts (stages 32 and 33) stay open and
unrepaired. Nothing was repaired. My own runs this gate were the red and
the 242 draws from copies outside the scene, so the scene was untouched by
the gate.

## 8. Entry

Entry authenticates 5,400 maintained / 590 design / 364,151 accepted files
(hashes taken at 19:18, before this continuation's outputs). I rehashed all
590 design files (278,559,491 B) and all 39,661 accepted files new against
the c56 entry (20,822,838,953 B) with zero missing and zero bad; the
324,490 carried accepted files are hash-identical to the c56 entry (zero
changed, zero removed), and my c56 gate rehashed that whole set. The only
three maintained deltas are my own committed documents (`TODO.md`,
brief-623, brief-742) at HEAD `261da953`; the pre-existing layer is the
same 23 porcelain entries; the worker changed no maintained or production
file (maintained_changes=0, production_changes=0, new_own_catches=0).

## 9. What I verified and what stays open

Verified by me: all 2,497 receipts against their logs (2,483 command
markers, 14 driver logs, zero problems; elapsed sum exactly
2,686.131851504979 s) and 62 distinct binaries hashed; all 242 executed
draws re-run from director copies outside the scene — 96 append, 96 handoff
and 50 retirement — with zero mismatches; the red reproduced at eight
threads with the observation line byte-identical (exit 101, 0 passed / 1
failed / 42 filtered); the full freeze (20,344 pairs); the full design and
new-accepted entry hashes and the carried-set identity; the four append
audit groups and the confirmed fixed-body counts; the eleven rejected
falsifiers by name; all 702 pair audits and 86 calibration audits exit 0;
the group totals (848/72/48/640, 400 raw, 172, 578, 598, 496, 128, 5,732
×2, 768 handoff, 240 append, 192 retirement); the plan, freeze, declaration
and build timestamps; and the budget arithmetic. The three FLAGGED items
are retained construction failures with no evidence change: the first
inventory attempted to open a standalone helper in the installed macro
placement, the second found six uncatalogued reference spellings (both
resolved, final inventory unknown=0), and the inherited C55-to-C56
comparator expected a second issuer migration (the corrected comparator
compared every current owner and assertion byte with zero allowed
replacements and all 43 frozen source pairs passed).

Open: everything in section 5, plus the Q57 repair and the retirement
class's own calibration and 94 unexecuted draws. Continuation 58 carries
both.

## 10. Process

Ruling written here; appended to brief-742 and brief-623 (Continuation
145), TODO.md extended, `state.md` rebuilt, committed and pushed;
clean-temp run; dispatch log archived; memory updated; continuation 58
dispatched on `gpt-6.2-astra` at effort max.

## Continuation 58 ruling (director) — the Q57 registry-retirement root authentication is repaired and every declared retirement draw is exact (144/144: 48 counterparts at 4/0/0/39, 48 foreign_root and 48 other_registration at 1/0/0/42, so the c57 stop is gone in both roles, both arms and both modes at eight and one), the declared carrier calibration executes (80 full unfiltered draws across five forms: 40 valid, 40 collapse rejected by the named control, 0 unrelated failures, 40 named rejections), the 96 append draws and 96 handoff draws stay exact, every matrix is re-earned, ten registry AST audits pass and seventeen static falsifiers are rejected by name. The new source-lookup class is then declared with zero unknown and frozen at 21:09:57.677 before its 21:11 builds, and its first E-family foreign draw stops under Q28: P1470/form-1 (witness, armed, hash; source `9fa6c927…`, binary `89baddae…`) was declared [1,0,0,25] and reproduced [0,1,0,25] at eight and one on that one binary, because `evaluation_owner` removes `unpaused[token.owner]` and inserts the source alias into `State.sources` before `source_proof` authenticates the installed owner, so an unrelated live outer installation gains the inner producer's reusable source key (`foreign_aliases_before=1 foreign_aliases_after=2`, both installations survive); 30 of 160 declared draws are NOT_RUN and nothing was repaired. The ruling is `Q58-evaluation-source-alias-authentication` (a genuine scratch-consumer design defect, the same class as Q54–Q57 one layer deeper, not a production regression and not a reclassification); the retirement repair, the carrier calibration, the inventories, the audits and the matrices earn focused scratch credit while the lookup claim and the complete bar earn nothing, and this gate allocates a fresh 12,000 s envelope so continuation 59 runs under 20,416.921829276976 s.

## Run record

Continuation 58 ran `cdx` as unit `brief-742-cdx-20261118-202437-3143164`: rc 0,
peak 2,586,988,544 B, oom 0 / oom_kill 0. The log is 2,743,253 B, sha256
`7e962fdc3ac7eb0e550870a98f02283d3fc7e3ee4ea3541df2ae1f448f785537`; the worker
header reads `model: gpt-6.2-astra`, `reasoning effort: max`. The report is
17,668,647 B, sha256
`cd62c93bb7151037d6e13f8733913836ff1423520f1daeb68fbe251d46b8d6c3`, verdict
STOP: 41,779 changes (41,777 added, 2 modified `PROGRESS.md`/`report.json`,
0 deleted, every path under `.temp/brief-742`), 40 PASS / 2 FAIL / 8 NA,
4,608 citations, 0 questions, 1 deviation, 0 disclosures. No git or network
command ran and no `.git-attempts` exists.

## Delivered and accepted

**The Q57 registry-retirement root authentication is repaired and every
declared retirement draw is exact.** The helper differs from the accepted c57
helper by 100 diff lines (90 changed): a new `identity_proofs::Retirement` and
`retire_registered(matches_object, cleanup)` select from the immutable
registration table (skipping already-retired registrations) and carry each
registration's install, owner, generation, namespace, entry and independently
recorded queue-source records to the cleanup; `State::retire(registration)`
rebuilds the expected `Installed` from that registration and returns without
removing anything if the transported slot differs, then removes `reverse`,
`unpaused`, `sources`, `paused`, `paused_at`, `confirmed` and `confirmed_at`
only where each alias points at that install and the payload proof matches the
independently recorded queue-at source, edge and proof id; `retire_root`,
`retire_control` and `clear_registry` all route through `retire_registered`
with their object predicates. All 144 retirement draws pass — 48 counterparts
at 4/0/0/39, 48 foreign_root at 1/0/0/42 and 48 other_registration at 1/0/0/42
over three forms x two roles x two arms x two modes x eight and one — so the
c57 stop (P0267/form-1 at 0/1/0/42) is gone in witness and runtime, armed and
disabled, hash and nohash. I reproduced the foreign_root control from a
director copy at eight and one on binary `6baa1b5e…`: exit 0, and the
observation is now `registration=true, reverse=true, indexed=true,
payload=true, registered_source=true, confirmed_source=(true,false,false,false)`.

**The declared carrier calibration executed.** All 80 full unfiltered draws
across the five E/J forms match their declarations: 40 valid draws at
22/0/0/0 or 23/0/0/0, 40 collapse draws rejected by the named control
`q35_second_live_install_preserves_first` (14 with the named failure alone,
six also firing the pre-declared armed J interleaved publication check that
depends on this carrier), and 0 unrelated failures. The collapse changed only
`State.installed` at the second insertion; the actual installations and their
reverse aliases were retained.

**Every prior matrix is re-earned exact.** Main 848 = 26,964/232/0/0; J 72 =
2,280; components 48 = 5,384/168; scope fresh 640 = 68,480/7,600/0/236,640;
the 400 raw draws reused only by exact identity (5,400/600/0/25,200; two
sources and eight binaries, all 400 logs hash-exact); calibrations 172 =
63,956/372/0/0; source-ticket 4 = 2,312/0/0/0; boundary 4 = 1,984/0/0/0;
P0558 8 = 128/0/0/0; both runtime placements 4 draws each = 5,732/0/20/8
(1,433/0/5/2 per draw, one binary); whole-consumers 4 = 2,392. The 96 append
draws are exact (48 counterparts at 4/0/0/27, 48 foreign-index at 1/0/0/30)
and the 96 handoff draws are exact (48 declared_classes at 14/0/0/39, 48
issued_substitution at 2/0/0/51). The 702 pair audits and 86 calibration
audits all exit 0; the closure account authenticates 17 groups, 117 source
pairs, 173 copies, 373 scope-actor obligations and 4,422 indexed obligations.

**The registries are inventoried with zero unknown.** The registry inventory
resolves 405 files, 53,765 bound functions, 150 consumers, 164 bodies, 360
selector sites and 221 carrier members against 125 prior selectors with
`embedded_helper_exact=true` and `unknown=0`; the wider candidate-registry
inventory resolves 415 files, 61,155 bound functions, 163 consumers, 206
bodies and 473 selector sites against 405 prior render paths with
`embedded_payload_exact=true` and `unknown=0`. Both macro placements are
verified by byte identity.

**The independent audits hold.** Ten projections pass the registry AST audit
(copies 1 or 2, fixed tests 210/43/43/31/31/53/53/578/598/496,
`independent_object_selector=true`, `registration_bound_aliases=true`,
`foreign_slot_preserved=true`, `original_guards_exact=true`,
`entire_prior_AST_restored=true`) and the fixed-body audit restores every one
of them (`projections=10`, `entire_prior_bodies_exact=true`). Seventeen
one-node static falsifiers of the repair — transported-root, control-uses-root,
clear-ignores-registry, all-objects, retired-registration, foreign-source,
proof-as-install, unchecked-slot, remove-before-check, reverse-other-value,
unpaused-other-value, source-other-value, wrong-edge, foreign-payload,
wrong-entry, foreign-confirmation, remove-all-slots — are each rejected with
exit 101 at the declared obligation, calibrated against the positive source
`99229c57…`.

**The admitted-source comparison completed its first half.** All 94 admitted
paths are compared (548 path/role pairs: 166 equal, 382 differing, every
differing byte disclosed), with `complete_generated_files=0` and
`additional_generated_paths=0`.

## The stop

The new independent source-lookup class is declared with zero unknown and
frozen before execution: the declaration (`consumers=2`, `actual_source_forms=5`,
`roles=2`, `arms=2`, `modes=2`, `threads=8,1`, `groups=2`, `draws=160`,
`positive_identical_missing_foreign=true`, `fixed_bodies_untouched=true`,
`pre_execution=true`) is written at 21:09:57.677, the first lookup build starts
at 21:11, the matrix runs at 21:11:44, and the scene freezes at 21:12:46 with
`candidate_failure=true candidate_repair=false`. The first E-family foreign
draw fails: P1470/form-1, witness, armed, hash (source `9fa6c927…`, 194,904 B;
binary `89baddae3c45da940ce1956ccb6003066b4a40a28022810de27d9f97a5352cb4`)
was declared `[1,0,0,25]` and reproduced `[0,1,0,25]` at eight and one on that
one binary. The remaining 30 of 160 declared draws are NOT_RUN,
`candidate_repair=false` and `expectation_repair=false`.

The observation is `CM742C58 ACTUAL_INSTALL_LOOKUP change=Foreign
recorded_own=false recorded_foreign=false source_is_actual=false
source_is_foreign=true foreign_aliases_before=1 foreign_aliases_after=2
registrations_preserved=true` and the assertion is
`Q35_FOREIGN_INSTALLATION_CANNOT_ACQUIRE_ANOTHER_PRODUCERS_SOURCE_ALIAS`. The
input substitution changes only `unpaused[inner owner]`; the immutable records
issue no `EvaluationReturn` proof for either installation and both
installations survive. The outer installation nevertheless gains the inner
producer's reusable `State.sources` key. The mechanism is in `evaluation_owner`:

```rust
let source = SourceKey::pause(&token);
let id = if let Some(id) = state.sources.get(&source) { *id }
    else {
        let id = state.unpaused.remove(&token.owner)?;
        insert(&mut state.sources, source, id);
        id
    };
source_proof(&state, token, id, SourceEdge::EvaluationReturn)
```

The alias is removed from `unpaused` and inserted into `sources` **before**
`source_proof` authenticates the installed owner, so a producer's pause can
select an unrelated live installation's source alias. The four declared
counterparts and the whole J family pass, so the fixture is non-vacuous and
the failure is specific to the unauthenticated publication.

## Ruling: `Q58-evaluation-source-alias-authentication`

The finding is a genuine design-level defect of the scratch consumer model,
the same class as Q54–Q57 one layer deeper: the producer-to-install source
lookup publishes a reusable alias from mutable transported state before the
actual installation is authenticated. Q35's accepted rule — a producer's
`SourceKey` can select only its authenticated installation, and all unrelated
live registrations and source aliases stay exact — has not been carried into
this seam. It is not a production regression and not a reclassification, and
it claims no incorrect publication or new semantic catch. The delivered
retirement repair, the carrier calibration, the inventories, the audits and
the matrices earn focused scratch credit; the lookup consumer claim and the
complete bar earn nothing.

## Ordered continuation 59

1. Complete the zero-unknown inventory of every producer-to-install source
   lookup and transport publication: `evaluation_owner`, `pause`,
   `source_scope`/`source_capture_scope`, `current_pending`,
   `SourceKey::pause`, `source_proof`, `insert(&mut state.sources, ...)`,
   `state.unpaused`, and every consumer of `State.sources`,
   `State.evaluation_sources`, `State.evaluation`, `State.unpaused`,
   `QueueKey`, `PassProof` and `SourceEdge`, in both witness and runtime
   renders, both feature modes and both arms, over the E and J families and
   the four ticket classes. Reuse the c58 registry inventories (415 files /
   61,155 bound functions / 163 consumers / 206 bodies / 473 selector sites,
   `unknown=0`) and extend them to the publication seam with the same
   zero-unknown rows.
2. Authenticate the actual producer and its installation before publishing:
   the candidate installation must equal the independent registration of the
   actual producer at the source occurrence, bound to the registered object
   and the issued edge; a foreign, missing or altered slot supplies no
   authority over another installation's aliases; unrelated live
   registrations, sources, pauses, confirmations and proofs stay exact.
   Execute the declared carrier calibration and add the independent AST
   falsifier.
3. Make the frozen foreign-root control the acceptance: all 160 lookup draws
   (130 executed in c58, 30 outstanding) in both families, both roles, both
   arms, both modes and both thread counts; the 144 retirement draws
   (including the c57 foreign_root at 1/0/0/42), the 96 append draws, the 96
   handoff draws and the 578/598/496/53/31/210/374 bodies stay exact.
4. Re-run every matrix: source-ticket, boundary, P0558, main, J, components,
   scope fresh and reused, calibrations, both runtime placements,
   whole-consumers.
5. Complete the bar under the envelope: the 134/4,422/133 execution closure,
   151 plans with rejection, the two 94-path/188-postimage roots, schema
   emission with 151 cells, the complete-overlay audits, the ordered standing
   replay, `cargo fmt --check`, both workspace modes with the seven skips and
   the unfiltered gates, and the switched same-binary checks. Stop before any
   group that cannot fit and report the exact remainder; repair no frozen
   tree, production or maintained source.

## Not delivered

The complete bar stays unexecuted. The 134/4,422/133 closure is indexed only
(`complete_source_execution=false`); plans are 0/151; the two 94-path roots
have 0 postimages of 188; the schema is unchanged
(`f9604852…`, `new_source_schema=null`, `complete_plan_hash=null`) with
`original_preimages=90`, `original_postimages=180`, 151 immutable rows and 0
emitted overlay paths; the standing replay is NOT_RUN with
`covered_maintained_edits=0 acceptance_suites=0`; the workspace is NOT_RUN
(`fmt=false hash=false nohash=false unfiltered_suspension_oracle=false
unfiltered_typed_slot_census=false switched_same_binary=false`,
`workspace_seconds=0`); the Q58 repair and 30 lookup draws are open; the
generator reports `complete_plans=0 required_plans=151 full134 false
full133_carriers false double94_roots false postimages188 false
schema_emission=false`.

## Allocation

Continuation 58 charged 4,576.095556478016 s — 2,896.095556478016 s measured
from 2,840 receipts (plus 16 driver-style records, zero problems, elapsed sum
exactly 2,896.095556478016) + 200.0 s conservative inspections/setup +
1,420.0 s bookkeeping (each serial command carries 0.5 s) + the fixed 60 s
reporting reserve — of the 12,993.017385754993 s envelope, leaving
8,416.921829276976 s; CM4 stood at 63,506.982614245004 of 76,500 s and now
stands at 68,083.07817072302 s. The stop is a candidate failure, not an
allocation limit.

This gate allocates a fresh 12,000 s focused envelope from the project's
reserve so the Q58 repair and the complete bar can run in one continuation:
continuation 59 runs under **20,416.921829276976 s**, CM4's share becomes
88,500 s, the stage reserve 105,000 s, and 743–746 keep their 40,500 s
unchanged. The assigned 6,000 s workspace is untouched (0/6,000), and the
1,440 s verification and second 6,000 s workspace stay held.

## Freeze

The stopped scene is 20,642 files / 11,002,709,004 bytes in
`continuation-stop/`; I rehashed all 20,642 frozen copies against the manifest
with zero missing and zero bad. The 29 prior trees hold 191,959 entries — the
c57 scene (20,344) is exact, the known director-replay transients (405) are
unchanged with `unexplained=0`, and the two `freeze-stop.log` drifts (stages
32/33) are unrepaired.

## Entry

Entry authenticates 5,400 maintained / 590 design / 404,856 accepted files. I
rehashed all 590 design and the 40,705 accepted files new against the c57
entry (21,432,529,908 B together) with zero missing and zero bad; the 364,151
carried accepted files are hash-identical to the c57 entry (zero changed, zero
removed). The only maintained deltas are my three committed documents at HEAD
`5e91d611` (TODO.md, brief-623, brief-742), recorded in `entry-drift.json`,
with the same 23 porcelain entries.

## What I verified

- All 2,840 receipts against their logs (2,824 command markers, 16
  driver-style records without their own marker, zero problems; elapsed sum
  exactly 2,896.095556478016 s) and 94 binaries hashed.
- The red from a director copy outside the scene at eight and one: exit 101,
  the same observation line and assertion; the only log differences are the
  ASLR pointer values, the elapsed line and the appended receipt.
- The retirement foreign_root control from a copy at eight and one: exit 0,
  the same observation with registration/reverse/indexed/payload all true.
- The retirement matrix totals from the logs: 72 passing configurations,
  counts 4/0/0/39, 1/0/0/42 and 1/0/0/42, driver `draws=144 builds=8`.
- The carrier matrix: 40 configurations, 20 valid and 20 collapse, the named
  control in every collapse and no unrelated failure; driver `full_draws=80
  builds=16`.
- The append and handoff logs: 48 configurations each, counts 4/0/0/27 and
  1/0/0/30; 14/0/0/39 and 2/0/0/51; drivers `draws=96 builds=8`.
- The lookup matrix: 64 passing configurations (E 16, J 48), one failing
  configuration, driver `planned_draws=160 executed_draws=130 builds=16`, and
  `no_repair=true`.
- The declaration, freeze, build and first-execution timestamps for every
  family (plans/freezes 20:34:39.110/.138, first declaration 20:34:39.411,
  first audits 20:34:44.131, first build 20:34:59.397; carrier declaration
  20:38:47.421, carrier matrix 20:39:23.794; lookup declaration
  21:09:57.677, first lookup build 21:11, lookup matrix 21:11:44, scene freeze
  21:12:46).
- The registry and candidate inventories (`unknown=0`), the ten registry AST
  audits, the 17 static falsifiers at exit 101, and the 548-entry
  admitted-source comparison (166 equal, 382 differing, all disclosed).
- The budget arithmetic exactly, and the scene and entry rehashes above.

## Process

The ruling is written here; the continuation-58 sections are appended to
brief-742 and brief-623 and the G15-CM4 block in TODO.md; `state.md` is
rebuilt; the three documents are committed and pushed; `clean-temp.py` runs
dry-run then apply (expected to refuse on the dirty `.temp` layer); the
dispatch log is archived as `.prev-20261118-2122`; memory is updated; and
continuation 59 is dispatched on `gpt-6.2-astra` at effort `max`.

## Continuation 59 ruling (director) — the Q58 evaluation-source alias authentication is repaired and all 160 declared lookup draws pass (80 configurations: 16 at [1,0,0,25], 24 at [1,0,0,46], 16 at [3,0,0,23], 24 at [3,0,0,44] over both families, both roles, both arms, both modes and eight and one; both P1470 foreign runs preserve the foreign installation's single alias), and every prior matrix is re-earned (144 retirement, 80 carrier, 96 append, 96 handoff, the 848 main, the 72 J, the 48 component and the 640 scope-form draws), with ten publication AST audits passing and seventeen static falsifiers rejected. The new append-namespace class is declared and frozen at 22:20:19 before its 22:21:54 builds, and its first ForeignRegistration draw stops under Q28: P0267/form-1 (witness, armed, hash; source `9aeb0299…`, 224,866 B; binary `c444e6b0…`) was declared [1,0,0,49] and reproduced [0,1,0,49] at eight and one on that one binary, because the actual receiver token forwards the proposed namespace, `join_queue` resolves it through the reverse index, and the inner `JoinAppend` destructor retires the unrelated outer confirmation (index and payload gone, retired=true, zero hits against one expected) while both ordinary return values stay exact; the six counterpart cases pass in all 48 draws, 46 of 96 declared draws are NOT_RUN, and nothing was repaired. The ruling is `Q59-append-receiver-namespace-authentication` (a genuine scratch-consumer design defect, the same class as Q54–Q58 one layer deeper, not a production regression); the lookup repair, the matrices, the audits and the falsifiers earn focused scratch credit while the append-namespace claim and the complete bar earn nothing, and this gate makes no new allocation: continuation 60 runs under the remaining 15,631.120352672991 s.

## Run record

Continuation 59 ran `cdx` as unit `brief-742-cdx-20261118-213157-3477078`: rc 0,
peak 2,573,291,520 B, oom 0 / oom_kill 0. The log is 3,233,687 B, sha256
`4666428d28df7979a467f6ee537cfa001dc917bd17b1a706bb54a207eb3f3b3d`; the worker
header reads `model: gpt-6.2-astra`, `reasoning effort: max`. The report is
18,127,311 B, verdict STOP: 42,465
changes (42,463 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted, every
path under `.temp/brief-742`), 46 PASS / 2 FAIL / 8 NA, 4,836 citations,
0 questions, 2 deviations, 0 disclosures.

## Delivered and accepted

**The Q58 evaluation-source alias authentication is repaired and all 160
declared lookup draws pass.** The helper differs from the accepted c58 helper
by 55 diff lines (45 changed): a new `identity_proofs::registered_at_pause`
reads the immutable records and returns the unique unqueued registration for
the occurrence between the preceding producer pause (or entry) and this pause,
requiring the entry, the owner, uniqueness and that the installation is not
retired; `source_proof` now requires the transported slot's proof id and owner,
the independent occurrence registration, the generation, the namespace and the
reverse alias to agree before issuing a `QueuePause` or `EvaluationReturn`
proof; `evaluation_owner` reads the proposed `unpaused` alias, issues the
proof, and only then removes the alias and publishes the source (a cached
alias is returned only through the same proof); `pause_installed` does the
same on the queue edge; `pending_construction` requires the token owner to
equal the construction owner. All 160 lookup draws are exact — 80
configurations: 16 at `[1,0,0,25]`, 24 at `[1,0,0,46]`, 16 at `[3,0,0,23]`,
24 at `[3,0,0,44]` over both families, both roles, both arms, both modes and
eight and one — so both P1470 foreign runs preserve the foreign installation's
single alias. Ten publication AST audits pass with
`independent_occurrence_registration=true`,
`registered_object_and_nonce=true`,
`proof_precedes_consumption_and_publication=true`, `both_edges_exact=true`,
`original_guards_exact=true` and `entire_prior_AST_restored=true`; the
fixed-body audit restores all ten (`projections=10`,
`entire_prior_bodies_exact=true`). Seventeen one-node static falsifiers of the
repair are each rejected with exit 101 at the declared obligation against the
calibrated positive `8baa26c7…`.

**Every prior matrix is re-earned exact.** The 144 retirement draws
(counterparts 4/0/0/39, foreign_root and other_registration 1/0/0/42), the 80
carrier calibration draws (40 valid, 40 collapse, 0 unrelated failures), the
96 append draws (4/0/0/27 and 1/0/0/30), the 96 handoff draws (14/0/0/39 and
2/0/0/51), the 848 main draws (26,964/232), the 72 J draws, the 48 component
draws, the 640 scope-form runs (640/640, 68,480/7,600/0/236,640), the 172
calibrations, both runtime placements and the route-family and join-run
families all match their declarations; the remaining driver reports
`child_commands=1,078 serial_seconds=515.612672`.

## The stop

The new append-namespace class is declared and frozen before execution: the
plan (48 items, 96 draws, two cases — six counterpart cases `Valid`,
`Identical`, `Missing`, `ForeignRoot`, `UnknownEpoch`, `ForeignControl` and the
`ForeignRegistration` control — `pre_execution=true`) and its freeze are
written at 22:20:19, the first of the eight fixture binaries is built at
22:21:54, the matrix runs at 22:22:14, and the scene freezes at 22:23:25 with
`candidate_failure=true candidate_repair=false`. The 24 counterpart
configurations pass in all 48 draws. The first `other_registration` draw fails:
P0267/form-1, witness, armed, hash (source `9aeb0299…`, 224,866 B; binary
`c444e6b07719a559595fbd3f17815062dcf36a6ea9aa1a32ec6ccefcd0f24ee8`) was
declared `[1,0,0,49]` and reproduced `[0,1,0,49]` at eight and one on that one
binary. Fifty of 96 declared draws were executed; 46 are NOT_RUN;
`candidate_repair=false` and `expectation_repair=false`.

The observation is `CM742C59 ACTUAL_APPEND_NAMESPACE row=P0267
change=ForeignRegistration witness=true enabled=true returned_same=true
visits=1 hits=0 expected_hits=1
outer_after(registration,source,index,payload,confirmed,revoked,retired,claimed)=Some((true, true, false, false, true, false, true, false))
records_equal_ordinary=true` and the assertion is
`Q35_UNRELATED_OUTER_CONFIRMATION_SURVIVES_INNER_RECEIVER_NAMESPACE`. The inner
append received its own queue and the other live installation's complete
same-generation namespace. The actual receiver token remains valid in
`append_observation::observe`, which forwards the proposed namespace;
`join_queue` resolves that namespace through the reverse index; the inner
`JoinAppend` destructor then retires the confirmation selected by this scope,
so the outer confirmation index and payload are gone and the outer queue loses
its eligible omission while both ordinary return values remain exact and both
executions record zero hits against the one expected.

## Ruling: `Q59-append-receiver-namespace-authentication`

The finding is a genuine design-level defect of the scratch consumer model,
the same class as Q54–Q58 one layer deeper: the append receiver resolves a
proposed namespace through `reverse` and retires the selected confirmation
without authenticating the namespace against the actual receiving queue's own
independent registration. Q35's accepted carrier rule — an actual append may
retire only its independently registered receiver's confirmation, and every
unrelated live confirmation survives — has not been carried into this receiver
namespace resolution. It is not a production regression and not a
reclassification; it claims no incorrect return value, only the retirement of
an unrelated live confirmation. The delivered lookup repair, the matrices, the
audits and the falsifiers earn focused scratch credit; the append-namespace
claim and the complete bar earn nothing. The first static-falsifier run
crashed on its second case (the pattern matched twice in the searched source
text); the corrected run scopes the search to the owner function body and
passes all 17 cases — a retained construction failure with no evidence change.

## Ordered continuation 60

1. Complete the zero-unknown inventory of every append-receiver namespace
   resolution and confirmation retirement: `append_observation::observe`,
   `join_queue`, `JoinAppend::drop`, `retire_append_confirmation`,
   `confirmed_append`, the proposed-namespace and token transport,
   `State.reverse`, `State.confirming`/`confirmed`/`confirmed_at`, and every
   consumer, in both witness and runtime renders, both feature modes and both
   arms, over the six counterpart cases and the `ForeignRegistration` control.
2. Authenticate the proposed namespace against the receiver's independently
   registered installation before resolving it through `reverse` or retiring
   any confirmation: only the actual receiving queue's own registration may
   authorize retirement; a foreign, mixed, absent or unknown namespace
   supplies no authority; unrelated live registrations, sources, confirmations
   and payloads stay exact.
3. Make the frozen append-namespace control the acceptance: all 96 draws (50
   executed in c59, 46 outstanding), the six counterpart cases and the
   `ForeignRegistration` control, in both roles, arms, modes and thread counts;
   the 160 lookup draws, the 144 retirement draws, the 80 carrier draws, the 96
   append draws, the 96 handoff draws and the 578/598/496/53/31/210/374 bodies
   stay exact.
4. Re-run every matrix: source-ticket, boundary, P0558, main, J, components,
   scope fresh and reused, calibrations, both runtime placements,
   whole-consumers.
5. Complete the bar under the envelope: the 134/4,422/133 execution closure,
   151 plans with rejection, the two 94-path/188-postimage roots, schema
   emission with 151 cells, the complete-overlay audits, the ordered standing
   replay, `cargo fmt --check`, both workspace modes with the seven skips and
   the unfiltered gates, and the switched same-binary checks. Stop before any
   group that cannot fit and report the exact remainder; repair no frozen
   tree, production or maintained source.

## Not delivered

The complete bar stays unexecuted. The 134/4,422/133 closure is indexed only;
plans are 0/151; emitted overlay paths are 0/94; postimages are 0/188; the
schema is unchanged (`f9604852…`, `new_source_schema=null`,
`complete_plan_hash=null`, 90 preimages, 180 postimages, 151 immutable rows);
the standing replay is NOT_RUN with `covered_maintained_edits=0
acceptance_suites=0`; the workspace is NOT_RUN (`fmt=false hash=false
nohash=false unfiltered_suspension_oracle=false
unfiltered_typed_slot_census=false switched_same_binary=false`,
`workspace_seconds=0`); the generator is NOT_DELIVERED with every flag false;
the append-namespace repair and 46 draws are open.

## Allocation

Continuation 59 charged 4,785.801476603985 s — 3,035.801476603985 s measured
from 2,980 receipts (plus 18 driver-style records, zero problems, elapsed sum
exactly 3,035.801476603985) + 200.0 s conservative inspections/setup +
1,490.0 s bookkeeping + the fixed 60 s reporting reserve — of the
20,416.921829276976 s envelope, leaving 15,631.120352672991 s; CM4 stood at
68,083.07817072302 of 88,500 s and now stands at 72,868.879647327 s.

**No new allocation.** Continuation 60 runs under the remaining
15,631.120352672991 s, which already exceeds the 12,993.017385754993 s
envelope continuation 58 ran under. If the bar cannot fit, continuation 60
stops before the group that does not fit and reports the exact remainder; a
further envelope is decided then, not automatically. The assigned 6,000 s
workspace is untouched (0/6,000), and the 1,440 s verification and second
6,000 s workspace stay held.

## Freeze

The stopped scene is 20,984 files / 11,174,741,745 bytes in
`continuation-stop/`; I rehashed all 20,984 frozen copies against the manifest
with zero missing and zero bad. The 30 prior trees hold 212,601 entries — the
c58 scene (20,642) is exact, the known director-replay transients (405) are
unchanged with `unexplained=0`, and the two `freeze-stop.log` drifts (stages
32/33) are unrepaired.

## Entry

Entry authenticates 5,400 maintained / 590 design / 446,632 accepted files. I
rehashed all 590 design and the 41,776 accepted files new against the c58
entry (22,321,786,336 B together) with zero missing and zero bad; the 404,856
carried accepted files are hash-identical to the c58 entry (zero changed, zero
removed). The only maintained deltas are my three committed documents at HEAD
`9ea39931` (TODO.md, brief-623, brief-742), recorded in `entry-drift.json`,
with the same 23 porcelain entries.

## What I verified

- All 2,980 receipts against their logs (2,962 command markers, 18
  driver-style records, zero problems; elapsed sum exactly
  3,035.801476603985 s) and 94 binaries hashed.
- The red from a director copy outside the scene at eight and one: exit 101,
  the same observation line and assertion.
- The lookup matrix: 80 passing configurations with the four declared count
  sets, driver `planned_draws=160 executed_draws=160 builds=16`.
- The retirement, carrier, append and handoff drivers: 144, 80, 96 and 96
  declared draws, all matching their declared counts.
- The append-namespace matrix: 24 passing counterpart configurations, the
  failing foreign configuration at draw 25, `executed_draws=50`, declared
  `[1,0,0,49]`, observed `[0,1,0,49]`, `no_repair=true`.
- The plan, freeze, build and first-execution timestamps (plan/freeze
  22:20:19, first binary 22:21:54, matrix 22:22:14, scene freeze 22:23:25).
- The ten publication AST audits, the 17 static falsifiers of the corrected
  run (`rejected=17 positive=8baa26c7…`), and the retained first-run crash.
- The budget arithmetic exactly, and the scene and entry rehashes above.

## Process

The ruling is written here; the continuation-59 sections are appended to
brief-742 and brief-623 and the G15-CM4 block in TODO.md; `state.md` is
rebuilt; the three documents are committed and pushed; `clean-temp.py` runs
dry-run then apply (expected to refuse on the dirty `.temp` layer); the
dispatch log is archived; memory is updated; and continuation 60 is dispatched
on `gpt-6.2-astra` at effort `max`.

## Continuation 60 ruling (director) — the Q59 append receiver namespace authentication is repaired and all 96 declared append-namespace draws pass (48 counterparts at [6,0,0,44] and 48 ForeignRegistration draws at [1,0,0,49] over both roles, both arms, both modes and eight and one; the former P0267 red is green with hits=1 expected_hits=1 and the outer installation preserved), every prior matrix is re-earned (160 lookup, 144 retirement, 80 carrier, 96 append, 96 handoff, the 848 main, the 72 J, the 48 component and the 640 scope-form draws), the receiver and namespace AST audits pass and 15 static falsifiers are rejected — and then the frozen literal auditor rejects the candidate calibration source: 972 of 974 assertion-and-owner entries are byte-exact and the two changed entries are the `source_install` insertion assertion, whose owner moved to `install_receiver` and whose record literal gained a `receiver` field. The same binary reproduces the rejection at eight and one (exit 101), the accepted source's self-comparison passes 974 of 974, nothing is repaired (`candidate_repair=false`, `expectation_repair=false`), the calibration's 172 draws and both runtime placements do not run, and the scene freezes at 23:23:55 with `candidate_failure=true`. The ruling is `Q60-frozen-assertion-identity-under-repair`: a repair may not satisfy its contract by changing a frozen assertion, even an inline production invariant inside the calibration source; the receiver binding must be carried outside the frozen insertion path so the assertion bytes and owner stay identical. The repair, the inventory, the 96 draws, the audits and the matrices earn focused scratch credit; the calibration, both runtime placements and the complete bar earn nothing, and this gate makes no new allocation: continuation 61 runs under the remaining 11,712.469021044915 s.

## Run record

Continuation 60 ran `cdx` as unit `brief-742-cdx-20261118-224108-3816447`:
rc 0, peak 1,757,835,264 B, oom 0 / oom_kill 0. The log is 2,943,257 B,
sha256 `74b09fd4a69c05cafd2a8695ce7c4c6359cdc85b0f546b371fc200b1e7bb9240`;
the worker header reads `model: gpt-6.2-astra`, `reasoning effort: max`,
sandbox `workspace-write`. The report is a STOP: 13,663 changes (13,661
added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted, every path under
`.temp/brief-742`), 37 PASS / 1 FLAGGED / 3 FAIL / 9 NA, 4,325 citations,
0 questions, 2 deviations, 0 disclosures, no git or network command and no
`.git-attempts`.

## Delivered and accepted with focused scratch credit

**The Q59 append-receiver namespace authentication is repaired.** The
registered source deltas are 435 with 117 corrected bases and 234 paired
renders over the 94 admitted paths and 180 pins, all exact (the probe diff
is 9,301 B). The repair introduces an `AppendReceiver` identity
(`Queue { address, kind }` or `Logical(namespace)`), records it on the
installation record, and resolves the append receiver from the captured
queue observations (`registered_receiver`) at the actual installation
edge. `join_queue` now resolves the proposed namespace only through the
receiver's independently registered installation
(`receiver_installation(receiver, generation, namespace)`) **and** requires
the `state.reverse` alias to equal that same install; an absent, foreign,
mixed or unknown receiver supplies no authority. `retire_append` selects
only the actual install's queue-at record, `retire_append_confirmation`
requires the scope's own `install` and retires through the authenticated
`confirmed_append` lookup, and replayed events carry no receiver so they
acquire no retirement authority.

**The zero-unknown receiver inventory is complete**: 423 input files,
228,746 function spans, 135,339 bound sites, 156 receiver bodies, 15,941
complete definitions, 277 declarations, 3 forms, 7 cases, 2 roles, 2 arms,
2 modes, threads 8 and 1, 336 source cells, `State.confirming` recorded
absent, `unknown=0`.

**All 96 append-namespace draws are exact** — 48 counterparts at
`[6,0,0,44]` and 48 `ForeignRegistration` draws at `[1,0,0,49]`, the 48
declared configurations matching their declared counts exactly
(`changed_expectations=0`, `pre_execution=true`). The c59 P0267/form-1 red
is green: in my own replay from a director copy at eight and one the
observation is `hits=1 expected_hits=1` with
`outer_after(...)=Some((true, true, true, true, true, false, false, false))`
— registration, reverse source, index, payload and confirmation all
preserved, nothing retired.

**The audits and matrices are re-earned.** The receiver AST audits pass
for the witness and runtime renders (`fixed_tests=50` each) and for both
calibration copies (`fixed_tests=374`), with
`independent_registration=true`, `actual_receiving_queue=true`,
`nonce_before_reverse=true`, `retirement_by_nonce=true`,
`unbound_supplies_no_authority=true`, `original_guards_exact=true` and
`entire_prior_AST_restored=true`. The namespace fixture AST preserves its
43 fixed test bodies (7 added) with `ordinary_body_bytes_exact=true`. All
15 one-node static falsifiers of the repair are rejected with exit 101 at
the declared obligation against the positive `73948151…`. Every matrix is
re-earned: lookup 160/160, retirement 144, carrier 80, append 96, handoff
96, append-namespace 96, main 848 (route-family driver 1,078 children,
524.278469 s), J 72, components 48 and the 640 scope-form runs. All six
declaration groups are frozen at 22:54:40–22:54:42, before their
22:54:54–22:57:40 builds.

## The stop

The frozen literal auditor (`audit-all-literals`, sha256 `d416f07d…`,
the same binary as c59) rejects the candidate calibration source
(`839a283f…` → `2e537299…`, 480,007 → 486,403 B): 972 of 974
assertion-and-owner entries are byte-exact and entries 154 (armed) and 641
(disabled) changed. Both are the installation insertion assertion, whose
owner moved from `source_install` to `install_receiver` and whose record
literal gained a `receiver` field:

```
prior     assert!(records.installed.insert(id, InstalledRecord {
              owner: owner.clone(), generation, namespace, entry }).is_none())
candidate assert!(records.installed.insert(id, InstalledRecord {
              owner: owner.clone(), generation, namespace, entry, receiver }).is_none())
```

The same binary reproduced the rejection at eight and one threads
(exit 101); the accepted source's self-comparison passes 974/974
(`whole_order_and_owner_byte_exact=true`, `assertion_waivers=0`). Neither
the candidate nor any expectation was repaired: `candidate_repair=false`,
`expectation_repair=false`; the calibration's 172 draws and both runtime
placements did not run (0/172, 0/2), and the scene froze at 23:23:55 with
`candidate_failure=true candidate_repair=false`.

The two changed entries still check that the insertion found no previous
value. The contract at issue is not the control assertions — the 374
controls' fixed bodies are byte-exact and the receiver calibration reports
`frozen_assertions_changed=0` — but the broader zero-waiver rule this
campaign adopted in c57: every frozen assertion, its owning function and
its order must match byte for byte in both calibration copies. The
candidate satisfied its semantic contract by editing a frozen assertion,
which is precisely the move the rule exists to reject.

## Ruling: `Q60-frozen-assertion-identity-under-repair`

A repair may not satisfy its contract by changing a frozen assertion —
including an inline production invariant embedded in the calibration
source. The Q59 repair's semantics are right, but its expression is
inadmissible: the receiver binding must be carried outside the frozen
insertion path so that the `source_install` insertion assertion and its
owner stay byte-identical, with zero waivers. This is a candidate-contract
violation under Q28, not a production regression and not a
reclassification; the frozen auditor is working as designed and caught the
change on its first run. The append-namespace repair, the inventory, the
96 draws, the audits and the matrices earn focused scratch credit; the
calibration claim, both runtime placements and the complete bar earn
nothing.

## Ordered continuation 61

1. Re-express the append-receiver authentication so every frozen assertion
   byte, owner and order stay identical in both calibration copies
   (974/974, zero waivers) — in particular the `source_install` insertion
   assertion — while all 96 append-namespace draws stay exact and the
   former P0267 red stays green.
2. Re-run the frozen literal audit, the 15 static falsifiers, the receiver
   and namespace AST audits (witness, runtime, both calibration copies,
   `fixed_tests=374`) and the body audits (578/598/496/53/31/210/374).
3. Complete the frozen calibration: all 172 draws and both runtime
   placements, with its 86 audits; repair no expectation, control or
   candidate.
4. Re-run every matrix on the final source: lookup 160, retirement 144,
   carrier 80, append 96, handoff 96, append-namespace 96, main 848, J 72,
   components 48, scope 640, whole-consumers and the route-family/join-run
   families.
5. Complete the bar under the envelope: the 134/4,422/133 execution
   closure, 151 plans, the two 94-path/188-postimage roots, schema emission
   with 151 cells, the complete-overlay audits, the ordered standing
   replay, `cargo fmt --check`, both workspace modes with the seven skips
   and the unfiltered gates, and the switched same-binary checks. Stop
   before any group that cannot fit and report the exact remainder; repair
   no frozen tree, production or maintained source.

## Not delivered

The calibration is 0/172 with both runtime placements at 0/2; the complete
bar stays unexecuted (closure indexed only, plans 0/151, emitted 0/94,
postimages 0/188, schema unchanged `f9604852…` with 90 preimages, 180
postimages and 151 immutable rows, standing replay NOT_RUN, workspace
NOT_RUN at 0/6,000 s).

## Allocation

Continuation 60 charged 3,918.651331628076 s — 2,311.151331628076 s
measured from 2,695 receipts (14 driver records and 2,646 embedded child
runs, zero problems, elapsed sum exactly 2,311.151331628076) + 200.0 s
conservative inspections/setup + 1,347.5 s bookkeeping (0.5 s per receipt)
+ the fixed 60 s reporting reserve — of the 15,631.120352672991 s
envelope, leaving 11,712.469021044915 s. CM4 stood at 72,868.879647327 of
88,500 s and now stands at 76,787.53097895508 s.

**No new allocation.** Continuation 61 runs under the remaining
11,712.469021044915 s. The assigned 6,000 s workspace is untouched
(0/6,000), and the 1,440 s verification and second 6,000 s workspace stay
held.

## Freeze

The stopped scene is 6,820 files / 4,363,161,895 bytes in
`continuation-stop/`; I rehashed all 6,820 frozen copies against the
manifest with zero missing and zero bad. The c59 tree (20,984 files) is
authenticated exact at entry with `original_drifts=0`; the known
director-replay transients (405) are unchanged with `unexplained=0`, and
the two `freeze-stop.log` drifts (stages 32/33) are unrepaired.

## Entry

Entry authenticates 5,400 maintained / 590 design / 489,094 accepted
files, with `inherited_document_drifts=3` (my three committed documents at
HEAD `a583d155`) and `production_changes=0`; the authentication pins 90
preimages, 180 postimages, 151 immutable rows and the unchanged schema
`f9604852…`. Entry took 429.467858 s.

## What I verified

- All 2,695 ledger receipts against their logs: 2,681 command markers
  with exit and elapsed exactly equal and 14 driver records, zero
  problems; the measured sum is exactly 2,311.151331628076 s and the
  charged arithmetic is exact.
- The 48 declared append-namespace configurations against the observed
  matrix lines: every form, role, arm, mode, class and count tuple equal;
  `changed_expectations=0`.
- The red from a director copy outside the scene at eight and one: exit
  101, the same assertion-owner panic, and the prior-versus-prior
  self-comparison green at 974/974.
- The former P0267 `ForeignRegistration` red replayed from a director copy
  at eight and one: exit 0, `hits=1 expected_hits=1`, the outer
  installation's registration, source, index, payload and confirmation all
  preserved.
- All 6,820 frozen copies rehashed (0 missing, 0 bad) and the entry
  authentication lines (5,400/590/489,094; 90/180/151; no production
  change).
- The declaration order: every group's freeze precedes its first build;
  the stop and freeze timestamps (23:17:41 rejection, 23:21:31 stop,
  23:23:55 freeze).

## Process

The ruling is written here; the continuation-60 sections are appended to
brief-742 and brief-623 and the G15-CM4 block in TODO.md; `state.md` is
rebuilt; the three documents are committed and pushed; `clean-temp.py`
runs dry-run then apply (expected to refuse on the dirty `.temp` layer);
the dispatch log is archived; memory is updated; and continuation 61 is
dispatched on `gpt-6.2-astra` at effort `max`.

## Continuation 61 ruling (director) — the Q60 frozen-assertion contract is repaired and all 974 frozen assertion-and-owner entries match byte for byte with zero waivers, all 96 append-namespace draws stay exact, the calibration completes green (172/172 draws, 2/2 runtime placements, 86/86 source audits, 374 fixed tests and fixed_assertions_byte_exact), the 15 static falsifiers are rejected, and every prior matrix is re-earned — and then the new publication-receiver class stops under Q28 on its first `EligibleRegistration` draw: P1470/form-1 (witness, armed, hash; source `03f8aa2e…`, 195,445 B; binary `e460b0d6…`) was declared `[1,0,0,28]` and reproduced `[0,1,0,28]` at eight and one, because `omit_evaluation_pass` compares the proposed namespace with the active proof's installation, claims the proof and skips ordinary publication without authenticating the actual publishing queue against the eligible installation's independently registered identity, so a different live same-generation queue records one omission hit and produces neither boundary-taken nor cell-boundary. All 16 counterpart configurations pass in 32 draws at [6,0,0,23], 30 of 64 declared draws are NOT_RUN, and nothing is repaired. The ruling is `Q61-evaluation-publication-receiver-authentication` (the same class as Q54–Q60 one layer deeper, Q35's rule not yet carried into the publication/omission seam, not a production regression); the re-expression, the calibration, the audits, the 96 append-namespace draws and the matrices earn focused scratch credit while the publication consumer claim and the complete bar earn nothing, and this gate makes no new allocation: continuation 62 runs under the remaining 6,589.274375287914 s.

## Run record

Continuation 61 ran `cdx` as unit `brief-742-cdx-20261118-234145-4055512`:
rc 0, peak 2,572,500,992 B, oom 0 / oom_kill 0. The log is 2,228,821 B,
sha256 `a48d7d2acf409780fff8a504bb6648573a837f9dc3899f52d3142699ebed574c`;
the worker header reads `model: gpt-6.2-astra`, `reasoning effort: max`,
sandbox `workspace-write`. The report is a STOP: 43,854 changes (43,852
added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted, every path under
`.temp/brief-742`), 36 PASS / 2 FLAGGED / 2 FAIL / 8 NA, 5,267 citations,
0 questions, 2 deviations, 0 disclosures, no git or network command and no
`.git-attempts`.

## Delivered and accepted with focused scratch credit

**The Q60 repair is delivered.** The receiver binding now lives in a
separate `receivers: BTreeMap<ProofId, AppendReceiver>` side table: the
`InstalledRecord` struct and its insertion assertion are byte-identical to
the accepted source, `source_install` takes `impl Into<ReceiverRegistration>`
and writes the optional receiver after the unchanged assertion, and
`receiver_installation` filters the immutable registration table by that
side table. The frozen literal auditor passes at eight and one with
`frozen_assertions=974 whole_order_and_owner_byte_exact=true
assertion_waivers=0` — I reproduced both runs from a director copy outside
the scene — and all 96 append-namespace draws remain exact. The former
P0267 red is still green in my replay at eight and one (`hits=1
expected_hits=1`, the outer installation's registration, source, index,
payload and confirmation all preserved).

**The calibration is complete and green.** 172/172 draws and 2/2 runtime
placements pass; the calibration driver runs 258 child commands in
292.147779 s. The calibration source audits pass **86/86** over 43 sources
and 86 copies with `whole_AST_restoration=true`,
`fixed_controls_attributes_exact=true` and `fixed_assertions_byte_exact=true`;
the receiver AST audits pass for both copies (`fixed_tests=374`,
`independent_registration=true`, `actual_receiving_queue=true`,
`nonce_before_reverse=true`, `retirement_by_nonce=true`,
`unbound_supplies_no_authority=true`, `original_guards_exact=true`,
`entire_prior_AST_restored=true`), and the 15 one-node static falsifiers
are each rejected with exit 101 at the declared obligation.

**Every matrix is re-earned** on the re-expressed source: lookup 160,
retirement 144, carrier 80, append 96, handoff 96, append-namespace 96,
main 848 (route-family driver 1,078 children, 534.904401 s), J 72
(84 children), components 48, scope 640, install-forms 40,
confirmed-carrier 96, source-ticket, boundary, P0558, whole-consumers, the
runtime driver (8 commands) and the placement runtime (8 commands). The
draw-closure line records `completed_calibration_draws=172/172
runtime_placements=2/2 source_duty_index=134/4422/133` over 21
authenticated groups and 702 pair audits.

**The new publication-receiver class is declared and frozen before
execution** (32 items, 64 declared draws, groups `counterparts` and
`eligible_registration`, `pre_execution=true`, frozen at 00:25:29 before
its 00:32:43 builds). All 16 counterpart configurations pass in 32 draws at
`[6,0,0,23]` with boundary-taken and cell-boundary and zero hits, the 40
installation draws and 96 confirmation draws pass, and then the first
`eligible_registration` draw stops the run.

## The stop

P1470/form-1, witness, armed/hash. The observation is
`CM742C60 PUBLICATION RECEIVER row=P1470 change=EligibleRegistration
generation=2 actual_foreign=true registry_exact=true returned=true
hits_before=0 hits_after=1 expected_hits_after=0 new_events=[]`; it was
declared `[1,0,0,28]` and reproduced `[0,1,0,28]` at eight and one on the
one binary `e460b0d6…` (source `03f8aa2e…`, 195,445 B; plan `d63507e1…`,
31,318 B; freeze `1fc80558…`). I reproduced the red from a director copy
outside the scene at eight and one: exit 101, the same observation line,
`0 passed; 1 failed; 28 filtered`.

The two live queues have distinct registered roots, epochs and controls.
The actual publishing queue is the other live same-generation queue; only
its proposed namespace argument changes to the eligible queue's namespace.
`omit_evaluation_pass` compares `Some(proof.installed.namespace) ==
namespace`, authenticates the proof and claims it, then removes the
evaluation and increments `state.hits` while skipping ordinary boundary
publication. It never authenticates the actual publishing queue against the
eligible installation's independently registered receiver, so the foreign
call returns `true`, the omission count rises from 0 to 1, and the
`boundary-taken` and `cell-boundary` events never occur. The six
counterpart cases (`Original`, `Identical`, `Absent`, `OtherRoot`,
`UnknownEpoch`, `OtherControl`) all produce boundary-taken and
cell-boundary with zero hits, so the fixture is non-vacuous. Thirty of 64
declared draws are NOT_RUN; `candidate_repair=false` and
`expectation_repair=false`.

## Ruling: `Q61-evaluation-publication-receiver-authentication`

The finding is the same class as Q54–Q60 one layer deeper: the evaluation
publication/omission seam authenticates the proposed namespace against the
active proof's installation but not the actual publishing queue against the
eligible installation's independently registered identity. Q35's accepted
rule — an actual queue may omit only its own independently registered
installation's publication, and every unrelated live queue keeps its
ordinary publication — has not been carried into this seam. It is a genuine
design-level defect of the scratch consumer model, not a production
regression and not a reclassification. The re-expression under the frozen
assertion contract, the calibration, the audits, the 96 append-namespace
draws and the matrices earn focused scratch credit; the publication
consumer claim and the complete bar earn nothing.

## Ordered continuation 62

1. Complete the zero-unknown inventory of every evaluation-pass
   publication and omission seam: `omit_evaluation_pass`,
   `publication_receiver`, `State.evaluation`, `State.current_evaluation_scopes`,
   `claim`, `authenticate(proof, SourceEdge::EvaluationReturn)`,
   `pending_construction`, `producer_owner`, `state.hits`, the
   `boundary-taken` and `cell-boundary` events, and every consumer, in both
   witness and runtime renders, both feature modes and both arms, over the
   six counterpart cases and the `EligibleRegistration` control.
2. Authenticate the actual publishing receiver against the eligible
   installation's independently registered identity before claiming the
   proof or skipping ordinary publication, so a foreign, mixed, absent or
   unknown receiver supplies no authority and every unrelated live queue
   keeps its own ordinary boundary publication with zero omission hits.
3. Make the frozen publication-receiver control the acceptance: all 64
   draws (34 executed, 30 outstanding), the 16 counterpart configurations
   and the 16 `eligible_registration` configurations, while the 160 lookup,
   144 retirement, 80 carrier, 96 append, 96 handoff, 96 append-namespace,
   40 install-forms, 96 confirmed-carrier draws, the 172 calibration draws,
   the 2 runtime placements, the frozen-assertion audit (974/974, zero
   waivers) and the 578/598/496/53/31/210/374 bodies stay exact.
4. Re-run every matrix on the final source.
5. Complete the bar under the envelope: the 134/4,422/133 execution
   closure, 151 plans, the two 94-path/188-postimage roots, schema emission
   with 151 cells, the complete-overlay audits, the ordered standing
   replay, `cargo fmt --check`, both workspace modes with the seven skips
   and the unfiltered gates, and the switched same-binary checks. Stop
   before any group that cannot fit and report the exact remainder; repair
   no frozen tree, production or maintained source.

## Not delivered

The publication-receiver group is 34 of 64 draws; 30 are NOT_RUN. The
complete bar stays unexecuted: the closure is indexed only, plans 0/151,
emitted 0/94, postimages 0/188, the schema is unchanged (`f9604852…`,
90 preimages, 180 postimages, 151 immutable rows), the standing replay is
NOT_RUN and the workspace is NOT_RUN at 0/6,000 s.

## Allocation

Continuation 61 charged 5,123.194645757001 s — 3,266.694645757001 s
measured from 3,193 receipts (20 driver records and 3,173 command markers,
zero problems, elapsed sum exactly 3,266.694645757001) + 200.0 s
conservative inspections/setup + 1,596.5 s bookkeeping + the fixed 60 s
reporting reserve — of the 11,712.469021044915 s envelope, leaving
6,589.274375287914 s. CM4 stood at 76,787.53097895508 of 88,500 s and now
stands at 81,910.72562471208 s.

**No new allocation.** Continuation 62 runs under the remaining
6,589.274375287914 s. The assigned 6,000 s workspace is untouched
(0/6,000), and the 1,440 s verification and second 6,000 s workspace stay
held.

## Freeze

The stopped scene is 21,464 files / 11,514,088,955 bytes in
`continuation-stop/`; I rehashed all 21,464 frozen copies against the
manifest with zero missing and zero bad. The c60 tree (6,820 files)
authenticates exact at entry with `original_drifts=0`; the 405
director-replay transients are unchanged with `unexplained=0`, and the two
`freeze-stop.log` drifts (stages 32/33) are unrepaired.

## Entry

Entry authenticates 5,400 maintained / 590 design / 502,754 accepted files
with `inherited_document_drifts=3` (my three committed documents at HEAD
`836603e6`) and `production_changes=0`; the authentication pins 90
preimages, 180 postimages, 151 immutable rows and the unchanged schema
`f9604852…`. Entry took 424.354056 s.

## What I verified

- All 3,193 ledger receipts against their logs (3,173 command markers with
  exit and elapsed exactly equal and 20 driver records, zero problems; the
  measured sum is exactly 3,266.694645757001 s) and the charged arithmetic
  exactly.
- The frozen literal audit from a director copy at eight and one: 974/974
  byte-exact with zero waivers.
- The P0267 append-namespace green replay from a director copy at eight
  and one: exit 0, `hits=1 expected_hits=1`, the outer installation
  preserved.
- The publication red from a director copy at eight and one: exit 101, the
  same `EligibleRegistration` observation line and the same 0-passed
  1-failed result.
- The 48 declared append-namespace configurations and the 32 publication
  configurations against their declared counts; the 16 counterpart
  publication configurations at `[6,0,0,23]`; the first
  `eligible_registration` stop at `[0,1,0,28]` against `[1,0,0,28]`.
- All 21,464 frozen copies rehashed (0 missing, 0 bad) and the entry
  authentication lines (5,400/590/502,754; 90/180/151; no production
  change).
- The declaration order: every group's freeze precedes its first build.

## Process

The ruling is written here; the continuation-61 sections are appended to
brief-742 and brief-623 and the G15-CM4 block in TODO.md; `state.md` is
rebuilt; the three documents are committed and pushed; `clean-temp.py`
runs dry-run then apply (expected to refuse on the dirty `.temp` layer);
the dispatch log is archived; memory is updated; and continuation 62 is
dispatched on `gpt-6.2-astra` at effort `max`.

## Continuation 62 ruling (director) — the Q61 publication receiver authentication is repaired, all 64 publication draws match, every prior draw and matrix is re-earned and the 974/974 frozen assertions hold with zero waivers — and then the new nested E carrier group's first runtime build is rejected with E0308: the registered redirect R02277 replaces `join_owner` (returns `JoinOwner`) with `source_join_owner` (returns `Option<JoinOwner>`) while the unchanged `join_result` requires `&JoinOwner`; all five registered runtime bases fail the same way and 88 of 96 E draws are NOT_RUN. The ruling is `Q62-registered-runtime-projection-type-identity` (an instrument-validity defect under Q28, not a scratch consumer defect and not a production regression); the publication receiver repair, the inventory, the 64 draws, the calibration, both placements and the matrices earn focused scratch credit while the E carrier claim and the complete bar earn nothing, and this gate allocates a fresh 12,000 s envelope so continuation 63 runs under 13,460.9335666599763 s.

## Run record

Continuation 62 ran `cdx` as unit `brief-742-cdx-20261119-005209-205450`:
rc 0, peak 5,644,967,936 B, oom 0 / oom_kill 0. The log is 3,347,442 B,
sha256 `47c58dd6d27addcef9cb02bba65e68749559ec1bd7184e8af5141c1d35fe91fa`;
the worker header reads `model: gpt-6.2-astra`, `reasoning effort: max`,
sandbox `workspace-write`. The report is a STOP: 44,234 changes (44,232
added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted, every path under
`.temp/brief-742`), 37 PASS / 1 FLAGGED / 2 FAIL / 8 NA, 5,347 citations,
0 questions, 2 deviations, 0 disclosures, no git or network command and no
`.git-attempts`.

## Delivered and accepted with focused scratch credit

**The Q61 repair is delivered.** `omit_evaluation_pass` now takes a
`PublicationReceiver { queue, registered, proposed }` instead of a bare
namespace: `publication_receiver` captures the physical queue's typed
address and kind, and the consumer requires `Some(registered) == proposed`
and `identity_proofs::receiver_installation(queue,
proof.installed.generation, registered.key()) == Some(proof.installed.proof)`
before it authenticates the proof, claims it and skips ordinary
publication. The physical site resolves a unique live owner for the queue's
root and epoch — two registered owners on the same root/epoch return `None`
— and every earlier return falls through to the ordinary boundary-taken and
cell-boundary publication. The `From<Option<Namespace>>` path preserves the
logical receiver kind.

**All 64 publication draws match.** 32 declared configurations (16
counterparts at `[6,0,0,23]`, 16 `eligible_registration` at `[1,0,0,28]`)
at eight and one threads, 8 builds, 224 individual cases. The c61 red is
green in my replay at eight and one on the same fixture and binary class:
`CM742C60 PUBLICATION RECEIVER row=P1470 change=EligibleRegistration
generation=2 actual_foreign=true registry_exact=true returned=true
hits_before=0 hits_after=0 expected_hits_after=0
new_events=["boundary-taken", "cell-boundary"]` — the foreign same-generation
queue keeps its own ordinary publication and records no omission hit. All
six counterpart substitutions (Original, Identical, Absent, OtherRoot,
UnknownEpoch, OtherControl) also keep zero hits and both boundary events.
The acceptance is the c61 frozen plan (`d63507e1…`, unchanged), re-frozen at
01:05:20 before the 01:05:33 builds.

**The zero-unknown publication inventory is complete.** 437 source files,
54,228 bound lexical sites, 324 source shapes, 43 declarations, 2 forms,
2 roles, 2 arms, 2 modes, 7 cases, 32 configurations, 64 draws, 224
individual cases, `unknown=0`; the survey isolates 428 sources plus 10
runtime sources and authenticates 23 observed terms.

**Every prior draw and matrix is re-earned.** 172/172 calibration draws
(258 child commands, 268.632845 s), 2/2 runtime placements (`1433/0/5/2`,
`same_binary=true`), the frozen assertion audit **974/974** with zero
waivers (I reproduced it with my own auditor copy against the accepted c61
calibration source and the c62 candidate), the 86/86 calibration source
audits over 43 sources and 86 copies, the fixed bodies
578/598/496/53/31/210/374, and the matrices: lookup 160, retirement 144,
carrier 80, append 96, handoff 96, append-namespace 96, install-forms 40,
confirmed-carrier 96, main 848 (route-family 1,078 children, 541.353378 s),
J 72 (84 children, 32.513951 s), components 48 (72 children,
57.442368 s), scope 640 (202.437010 s), runtime driver 8 commands,
placement runtime 8 commands. The publication AST audits pass in every
copy and site, the 15 one-node static falsifiers and the 7 independent
publication falsifiers are rejected at exit 101, and the receiver
calibration rejects 15 against the positive `a6d8214a…` with
`frozen_assertions_changed=0`.

The FLAGGED `examiner_setup` row is settled: the initial physical-receiver
mutation matched two identical address expressions, the locator was
narrowed before compilation, and all seven predeclared falsifiers then
completed. This is a retained construction note with no evidence change.

## The stop

The additional nested E carrier group is declared with `carriers=2 forms=2
roles=2 arms=2 modes=2 classes=3 draws=96 fixed_tests=25 added=1
independent_actual_session_views=true both_registered_sources_required=true
pre_execution=true`, frozen at 01:44:59.957 and declared at 01:45:00.005
before its 01:45:06.857+ builds; its 8 witness draws pass at `[26,0,0,0]`
and its separate live group passes 8 draws with 25 tests each. Its first
runtime build (`evaluation-carriers-fixture-005`, runtime / armed / hash)
is rejected with E0308: at line 3984
`join_result::<()>(&_join, &Err(EvalError::Other))` expects `&JoinOwner`,
but `_join` comes from the registered redirect R02277
(`join_owner(` → `source_join_owner(`) whose replacement returns
`Option<JoinOwner>`. No binary is emitted; `candidate_repair=false`,
`expectation_repair=false`; 8/96 draws executed and 88 are NOT_RUN
(24 valid, 32 payload-collapse, 32 scope-collapse). The provenance checks
pass: `frozen_role_edit_exact=true preceding_runtime_body_exact=true
observation_did_not_edit_control=true matching_registered_runtime_sources=5
compiled_rejections=1`. I reproduced the rejection from a director copy at
line 3984 with exit 1, and I extended it: **all five registered runtime
bases (083, 098, 099, 100, 101) contain the same redirect and each fails
with exactly one E0308, while all five witness bases compile at exit 0.**
The projection set, not just this fixture, is unbuildable at this site.

## Ruling: `Q62-registered-runtime-projection-type-identity`

A registered runtime redirect is valid only if it is type-identical at
every projected call site: the replacement must have the same callable
shape, result type, borrow shape and generic arity as the call it replaces,
so that the unchanged control compiles and runs under both renders. A
redirect that changes the type of a local binding or a call result is not a
render of the unchanged control — it is an invalid projection and it stops.
The repair must live in the probe entry or the registered edit set (a
type-preserving replacement whose absence path maps to the production call's
existing representation); it may not edit the frozen control body before its
declared edits, weaken or move an assertion, or delete or reclassify a draw.
This is an instrument-validity defect under Q28, not a defect of the scratch
consumer model, not a production regression and not a reclassification.

The publication receiver repair, the inventory, the 64 draws, the
calibration, both runtime placements, the 974/974 frozen assertions and the
matrices keep their focused scratch credit. The E carrier claim and the
complete bar earn nothing.

## Ordered continuation 63

1. Repair R02277's projection type-identically, and prove every one of the
   five registered runtime sources (083, 098, 099, 100, 101) builds — the
   bar is a build per source, not a single fixture — while the witness
   renders stay byte-exact and the edit records stay byte-invertible.
2. Re-run the complete E carrier schedule: 96 draws (8 executed, 88
   outstanding) plus the 8 live draws and the E AST audits.
3. Keep the frozen assertion audit 974/974 with zero waivers, the byte-exact
   bodies 578/598/496/53/31/210/374, the 64 publication draws, the 172
   calibration draws (86/86 source audits), both runtime placements and the
   install-forms 40 and confirmed-carrier 96 draws exact; re-run every
   matrix (160 lookup, 144 retirement, 80 carrier, 96 append, 96 handoff,
   96 append-namespace, 848 main, 72 J, 48 components, 640 scope) on the
   final source.
4. Then continue the bar from 407/4,422 qualified source obligations,
   31/134 actors, 2/133 fields and 26 named carrier groups: complete
   134/4,422/133, the 151 plans, the 94 paths, the 188 postimages and the
   151-cell schema, then the audits, the ordered standing replay,
   `cargo fmt --check`, both workspace modes and the switched checks. Stop
   before any group that cannot fit and report the exact remainder.

## Not delivered

The E carrier group has 88/96 draws NOT_RUN and its runtime and collapse
classes are unbuilt. The complete bar is unexecuted: 407/4,422 source
obligations qualified, 31/134 actors observed, 2/133 fields separately
calibrated, 26 named carrier groups, 4,015 obligations outside the named
union; plans 0/151, emitted paths 0/94, postimages 0/188; the schema is
unchanged (`f9604852…`, 90 preimages, 180 postimages, 151 immutable rows);
typed construction draws, weak-store receipts and complete-class credit are
all zero; the standing replay and the workspace are NOT_RUN (0/6,000).

## Allocation

Continuation 62 charged 5,128.340808627938 s — 3,249.340808627938 s measured
over 3,238 receipts (3,216 command markers and 22 driver records, zero
problems) + 200.0 s conservative inspections/setup + 1,619.0 s bookkeeping
+ the fixed 60 s reserve — of 6,589.274375287914 s, leaving
1,460.9335666599763 s; CM4 stood at 81,910.72562471208 of 88,500 s and now
stands at 87,039.06643334002 s.

The remaining 1,460.9335666599763 s cannot cover the five-source projection
repair, the 96-draw E schedule, the re-earned matrices and the bar. This
gate allocates a fresh 12,000 s focused envelope from the project's stage
reserve: continuation 63 runs under 13,460.9335666599763 s, CM4's share
becomes 100,500 s, the stage reserve 93,000 s, and 743–746 keep their
40,500 s unchanged. The assigned 6,000 s workspace is untouched (0/6,000),
and the 1,440 s verification and second 6,000 s workspace stay held.

## Freeze

The stopped scene is 22,107 files / 11,543,417,952 bytes;
`originals_and_copies_exact=true`. I rehashed all 22,107 manifest entries
on both sides with zero missing and zero bad. The c61 tree (21,464 files)
authenticates exact against its own manifest at entry (`original_drifts=0`),
the 405 director-replay transients are unchanged with `unexplained=0`, and
the two `freeze-stop.log` drifts (stages 32/33) are unrepaired.

## Entry

Entry authenticates 5,400 maintained / 590 design / 546,605 accepted files
with `inherited_document_drifts=3` — my three committed documents
(`TODO.md`, brief-623, brief-742) at HEAD `714ba022` — and
`production_changes=0`. The authentication pins 90 preimages, 180
postimages, 151 immutable rows and the unchanged schema `f9604852…`.

## What I verified

I verified all 3,238 ledger receipts against their logs (3,216 command
markers with exit and elapsed exact, 22 driver records, zero problems;
measured sum exactly 3,249.340808627938 s); the plain `check` (OK, 5,347
citations); every declared draw count against its matrix log; the 974/974
audit with my own auditor copy outside the scene; the P1470 green replay
from a director copy at eight and one; the E0308 rejection from a copy and
the five runtime / five witness base builds; all 22,107 frozen pairs; the
c61 tree; entry; and the budget arithmetic exactly.

## Process

`.temp/director/ruling-c62.md`; brief-742 appended (`## Continuation 62
ruling (director)`), brief-623 appended (`### Continuation 150`), TODO's
G15-CM4 block extended; state.md rebuilt from bullet 75 + 74 + 73 +
compacted 72; commit with `git add` and `git commit` in separate calls,
ending `Co-Authored-By: Claude Code <noreply@anthropic.com>`; push; clean-temp
dry-run then `--apply`; the dispatch log archived; memory updated; then
continuation 63 dispatched.

## Continuation 63 ruling (director) — the Q62 registered runtime projection type is repaired with a type-identical redirect and all 40 builds pass, the E carrier schedule completes at 96/96 draws plus 8 live, the typed construction group passes 96/96 on two forms and every prior draw and matrix is re-earned — and then the new typed retirement-receiver group stops under Q28: physically abandoning adapter B with A's live key transported to `core::forget_frame` retires A's operation, invalidates A's construction record and removes A's frame binding, so A's next resume records zero hits against the required one. The ruling is `Q63-typed-frame-retirement-actual-receiver-authentication` (the Q35 class one layer deeper: a genuine design-level defect of the scratch consumer model, not an instrument defect, not a production regression); the projection repair, the E schedule, the typed construction group and every re-earned matrix earn focused scratch credit while the retirement-receiver claim and the complete bar earn nothing, and this gate allocates a fresh 12,000 s envelope so continuation 64 runs under 19,993.105053099 s.

## Run record

Continuation 63 ran `cdx` as unit `brief-742-cdx-20261119-021017-550950`:
rc 0, peak 6,212,231,168 B, oom 0 / oom_kill 0. The log is 2,427,328 B,
sha256 `9089dfe75d57cf0c3c496f24999e01331f690191a11e0b1c899d8a33b3c82863`;
the worker header reads `model: gpt-6.2-astra`, `reasoning effort: max`,
sandbox `workspace-write`. The report is a STOP: 43,790 changes (43,788
added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted, every path under
`.temp/brief-742`), 39 PASS / 2 FAIL / 8 NA, 5,801 citations, 0 questions,
2 deviations, 0 disclosures; no git or network command ran and no
`.git-attempts` exists.

## Delivered and accepted with focused scratch credit

**The Q62 repair is delivered.** Registered reference R02277 no longer
changes the projection's type: the runtime render now redirects to
`source_completion_owner(issued: Option<IssuedSource>) -> JoinOwner`,
type-identical to `join_owner` (same argument type, result type, borrow
shape and generic arity), pinned by `const _: fn(Option<IssuedSource>) ->
JoinOwner = join_owner;`, `... = source_completion_owner;` and `const _:
fn(&JoinOwner, &Result<(), crate::EvalError>) = join_result::<()>;`. The
runtime handle is re-expressed as `completion_handle::JoinOwner(ScopeId,
JoinIdentity, Option<CapturedEntry>)` behind the same `JoinOwner` name and
constructor call sites (`old_constructor_unchanged=true`), where
`JoinIdentity` derefs to the registered `scope_object::Object` and carries
`None` only for the inactive handle; `Drop` and `join_result_kind` early-
return only when `!ENABLED` and the registration is absent, so the disabled
render acquires no scope, object or ticket while the armed render calls the
unchanged production constructor. All 40 builds pass (5 bases × 2 roles ×
2 arms × 2 modes): bases 083, 098, 099, 100 and 101 as runtime and witness
in armed and disabled, hash and nohash. The five witness renders are byte-
exact to c62 (`73b613a1…`, `65aefe0b…`, `6346dbcd…`, `077cc8b9…`,
`3e0fea66…`); the runtime renders carry `2230ce02…`, `aeba5d00…`,
`1049a425…`, `96f68d69…`, `5ee32b49…` and build to `916697bd…`,
`1362a8ae…`, `f8d55e3e…`, `8eb7a339…`, `b4e08da6…` (armed) and
`f13af276…`, `f6356be1…`, `3d894a4a…`, `cb27fcfd…`, `555523b0…`
(disabled). The per-base AST audits pass (`original_inputs_return_generics_
exact`, `producer_body_exact`, `actual_sink_borrow_exact`,
`disabled_return_before_state`, `destructor_and_sink_restore`,
`nested_control_single_callee_delta`), the assertion byte audit reads 180
assertions per base byte-exact with `renderer_manifest_used=false`, and the
restoration audit passes with `whole_prior_AST_exact=true allowed_changes=13
issuance_and_absence_checked=true old_constructor_unchanged=true`. The all-
source-pair audit covers 117 sources, 234 renders, 173 copies and 702
independent inherited audits with `changed_runtime_inverses=5
whole_prior_AST_and_bytes_exact=true all_frozen_assertions_exact=true`.

**The E carrier schedule completes.** All 96 draws of the unchanged c62
frozen plan (`5fa734bc…`) pass at eight and one on 24 builds: 48
configurations (2 forms × 2 roles × 2 arms × 2 modes × 3 classes) —
`valid` at `[26,0,0,0]` in every recording configuration and `[1,0,0,25]`
in the runtime-disabled render, `payload-collapse` and `scope-collapse` at
their declared rejection counts and exit codes (witness armed/disabled and
runtime armed reject at 101; runtime disabled keeps `[1,0,0,25]`). The
eight live draws pass with 25 tests each, the six E AST audits pass
(`independent_actual_receivers=2 issuer_view_readonly=true
fixed_source_restored=true frozen_tests=25 added_test=1`), and the
execution audit reads `draws=96 valid=32 payload_collapse=32 scope_collapse=32
independent_two_owner_snapshots_exact=96 live_draws=8 AST_audits=6
frozen_fixed_tests=25 new_own_catches=0`. The frozen-schedule equivalence
audit reads `configurations=48 draws=96 old_plan_hash=5fa734bc…
witness_bytes_exact=3 runtime_inverses=3 observer_bytes_exact=true
fixed_counts_names_exit_selectors_filters_and_order_exact=true`.

**The typed construction group is delivered on two forms.** 96/96 draws at
eight and one on 24 builds over two forms (P0278/form-1, P0281/form-1), two
stores (`core::Ledger.frames`, `core::Pair.tokens`) and three variants; the
declaration is `class=construction_records fields=2 forms=2 roles=2
variants=3 draws=96 live_hash_identical=8 existing_tests=14 added_test=1
full_73_form_class=false pre_execution=true`. The six typed AST audits pass
(`fixed_tests=14`, `physical_arc_and_type=true`,
`independent_direct_record=true`, `whole_ast_restored=true`), the live
group is hash-identical across eight draws, and the obligation audit reads
`actual_source_forms=2 independently_challenged_stores=2 additional_draws=96
exact_live_draws=8 two_actual_allocations=true
weak_loses_only_first_owner=true full_73_form_class=false`. The class is
explicitly partial: the two stores are calibrated on two of 73 forms and
`not_credited` names the other construction forms and deferred storage.

**Every prior draw and matrix is re-earned on the final source.** Lookup
160, retirement 144, carrier 80 (16 builds), append 96, handoff 96,
append-namespace 96, install-forms 40 (4 builds), confirmed-carrier 96
(16 builds), publication-receiver 64 (8 builds), main 848 (route-family
driver 1,078 children, 542.733238 s), J 72 (84 children, 34.886544 s),
components 48 (72 children, 56.884248 s), scope 640 (207.007271 s), the
calibration 172 draws (258 child commands, 261.909481 s) with 86/86 source
audits over 43 sources and 86 copies, the runtime driver (8 commands,
140.109029 s), the placement runtime (8 commands, 138.108107 s) at
`1433/0/5/2` with `same_binary=true` in both modes, and source-ticket,
boundary, P0558 and whole-consumers. The frozen bodies stay
578/598/496/53/31/210/374, the frozen-assertion audit stays 974/974 with
`whole_order_and_owner_byte_exact=true assertion_waivers=0` (I reproduced
it with my own build of the auditor and confirmed it fires at exit 101 on a
one-byte mutation), and the declaration audit reads `variants=42
controls=374 cells=15708 forced=93 not_forced=15615
new_installation_cells=1596 unknown=0 firing_variants=38 masked_variants=4
all_causal_anchors_exact=true whole_byte_inverses=true
frozen_source_and_control_hashes_exact=true execution_used=false`.

## The stop

The new typed retirement-receiver group is declared at 03:00:44.629 (16
configurations × 2 thread counts = 32 draws, four cases `Actual`, `Missing`,
`ForeignType`, `OtherLive`, two forms P0278/form-1 and P0281/form-1, two
roles, two arms, two modes) and built at 03:00:48.591. Its first
configuration (P0278/form-1, witness, armed, hash) returns 18 passed and 1
failed against the declared `[19,0,0,0]` at both eight and one threads on
the same binary `fa642405a0606ebf7cbcda8a80ad62a406b142dfb38781aa1f96f46c7cae5c60`.
The failing test is
`foreign_live_key_preserves_actual_outer`: physically abandoning adapter B
while transporting A's live key to `core::forget_frame` marks A's operation
retired, invalidates A's construction record and removes A's frame binding
(`outer_after=[true,false,false,false,true]` against the required
`[true,true,true,true,true]`), so A's next resume records zero hits against
the required one. The other three cases pass (Actual, Missing, ForeignType
all keep `[true,true,true,true,true]`, first hit one, second hit zero), all
15 prior controls pass, and the provenance reads
`only_proposed_retirement_key_changed=true`,
`complete_prior_AST_and_15_original_tests_exact=true`,
`actual_B_Arc_and_type_preserved=true`. The group is sealed immediately
(17 files, `plan_and_runtime_sources_exact=true`,
`actual_diagnostics_sealed=true`, `repairs=0`); `candidate_repair=false` and
`expectation_repair=false`; 2 of 32 draws are executed and 30 are NOT_RUN.

## Ruling: `Q63-typed-frame-retirement-actual-receiver-authentication`

At typed-frame retirement the consumer must authenticate the proposal
against the actual abandoning adapter's independently recorded identity
before any effect. A proposed key that identifies another live adapter of
the same concrete frame type supplies no authority: the foreign, missing,
mixed-type and foreign-live-key proposals must all leave the actual outer
adapter's operation record, construction record, frame binding, paired
source token and omission hit untouched. The repair lives in the probe
entry or the registered edit set of the typed construction consumer — the
retirement entry must carry the actual adapter's identity (the `Arc` it
abandons) alongside the proposed key, in the shape Q61 established for
publication, and compare the proposed key's independently registered
frame/adapter binding with the actual binding before mutating the ledger; an
explicit before-effect rejection is the declared allowed outcome. It may
not edit the frozen control body or its assertions, delete or reclassify a
draw, or reclassify the failure.

This is a genuine design-level defect of the scratch consumer model — the
Q35 class one layer deeper (Q35's accepted rule that an actual append may
retire only its own independently registered receiver's confirmation, not
yet carried into typed-frame retirement) — not an instrument-validity
defect, not a production regression and not a reclassification.

The projection repair, the E schedule, the typed construction group, the
calibration, the audits and every re-earned matrix keep their focused
scratch credit. The retirement-receiver claim, the complete typed class and
the complete bar earn nothing.

## Ordered continuation 64

1. Repair the typed-frame retirement entry under Q63 and re-earn the full
   32-draw retirement-receiver schedule (2 executed, 30 outstanding),
   preserving the frozen P0278/P0281 control and its declarations and the
   15 prior controls; keep the two typed-construction stores and their 96
   draws exact.
2. Keep the frozen-assertion audit 974/974 with zero waivers, the bodies
   578/598/496/53/31/210/374, the 64 publication draws, the 172 calibration
   draws (86/86 source audits), both runtime placements, the 96 E draws
   plus 8 live, the 40 install-form and 96 confirmed-carrier draws and the
   five runtime projections at 40 builds exact; re-run every matrix on the
   final source.
3. Then continue the bar from 407/4,422 qualified source obligations,
   31/134 actors, 4/133 fields and 26 named carrier groups: complete
   134/4,422/133, the 151 plans, the 94 paths, the 188 postimages and the
   151-cell schema, then the audits, the ordered standing replay,
   `cargo fmt --check`, both workspace modes and the switched checks. Stop
   before any group that cannot fit and report the exact remainder.

## Not delivered

The retirement-receiver class has 30/32 draws NOT_RUN and its claim is
unearned. The complete bar is unexecuted: plans 0/151, emitted paths 0/94,
postimages 0/188; the schema is unchanged (`f9604852…`, 90 preimages, 180
postimages, 151 immutable rows); 4,015 obligations outside the named union;
typed construction is 2 of 73 forms with other construction forms and
deferred storage not credited; the standing replay and the workspace are
NOT_RUN (0/6,000); complete-class credit is zero.

## Allocation

Continuation 63 charged 5,467.828513560977 s — 3,437.8285135609767 s
measured over 3,540 receipts (3,515 command markers with exit and elapsed
exact and 25 driver records, zero problems) + 1,770.0 s bookkeeping + 200.0
s conservative inspections/setup + the fixed 60 s reserve — of
13,460.933566659976 s, leaving 7,993.105053099 s; CM4 stood at
87,039.06643334002 of 100,500 s and now stands at 92,506.894946901 s.

The remaining 7,993.105053099 s cannot cover the typed retirement repair,
the 32-draw schedule, the re-earned matrices and the bar. This gate
allocates a fresh 12,000 s focused envelope from the project's stage
reserve: continuation 64 runs under 19,993.105053099 s, CM4's share becomes
112,500 s, the stage reserve 81,000 s, and 743–746 keep their 40,500 s
unchanged. The assigned 6,000 s workspace is untouched (0/6,000), and the
1,440 s verification and second 6,000 s workspace stay held.

## Freeze

The stopped scene is 21,885 files / 12,385,779,711 bytes;
`originals_and_copies_exact=true`. I rehashed all 21,885 manifest entries
on both sides with zero missing and zero bad. The c62 tree (22,107 files)
authenticates exact against its own manifest with zero missing and zero
bad, the 405 director-replay transients are unchanged with
`unexplained=0`, and the two `freeze-stop.log` drifts (stages 32/33) are
unrepaired.

## Entry

Entry authenticates 5,400 maintained / 590 design / 590,836 accepted files
with `inherited_document_drifts=3` — my three committed documents
(`TODO.md`, brief-623, brief-742) at HEAD `c4b1306f` — and
`production_changes=0`. The authentication pins 90 preimages, 180
postimages, 151 immutable rows and the unchanged schema `f9604852…`.

## What I verified

I verified all 3,540 ledger receipts against their logs (3,515 command
markers with exit and elapsed exact, 25 driver records, zero problems;
measured sum exactly 3,437.8285135609767 s); the plain `check` (OK, 5,801
citations); every declared draw count against its matrix log; the
974/974 audit with my own build of the auditor, plus a one-byte mutation of
one assertion that the same build rejects at exit 101; the retirement red
from my own copy of the fixture at eight and one (18/1/0/0, same binary,
same assertion); all 40 projection builds declared and my own rebuilds of
083 runtime (byte-identical `916697bd…`) and all five witnesses; all
21,885 frozen pairs and the c62 tree; entry; and the budget arithmetic
exactly.

## Process

`.temp/director/ruling-c63.md`; brief-742 appended (`## Continuation 63
ruling (director)`), brief-623 appended (`### Continuation 151`), TODO's
G15-CM4 block extended; state.md rebuilt from bullet 76 + 75 + 74 +
compacted 73; commit with `git add` and `git commit` in separate calls,
ending `Co-Authored-By: Claude Code <noreply@anthropic.com>`; push;
clean-temp dry-run then `--apply`; the dispatch log archived; memory
updated; then continuation 64 dispatched.

## Continuation 64 ruling (director) — the Q63 frame-retirement receiver is repaired and propagates over 69 registered renders, the typed carriers and the last-drop group are green and every prior matrix is re-earned — and then the new typed-abandon-entries group stops under Q28 at the sibling operation-retirement entry. The ruling is `Q64-typed-teardown-entry-actual-receiver-authentication`; the Q63 repair, its propagation, the typed carriers, the last-drop group and every re-earned matrix earn focused scratch credit while the teardown-entry claim, the complete typed class and the complete bar earn nothing. Full record in `.temp/director/ruling-c64.md`.

`retirement_for_adapter(actual: &Arc<F>, proposed: (usize, &'static str)) -> Option<(usize, &'static str)>` carries the actual abandoning adapter's Arc and requires the proposal to equal its key, a live `OperationKind::Direct` record of that key, `token.frame_identity == Some(actual_key)` and `current_frame_records[token] == Some(true)` before `forget_frame` runs; both `FrameAbandon::abandon` and `Drop` route through it and a rejection has no effect, with the frozen exercise and its assertions unchanged (`fixed_exercise_and_assertions_exact=true full_AST_inverse=true`). The registered edits (`construction_record_check`, `typed_abandon`, `typed_final_drop`) propagate over 2 retirement-receiver renders, 6 typed-carrier renders and 69 rebinds (`registered_renders=69 every_abandon_and_final_drop_checked=true`); the all-source-pair audit covers 117 sources / 234 renders / 702 inherited audits with `registered_repair_inverses_exact=true`. The retirement-receiver group is green at 32/32 draws (16 configurations x 2 threads, 8 builds): the c63 red P0278/form-1 witness armed/hash reads `[19,0,0,0]`, `OtherLive` keeps `outer_after=[true,true,true,true,true]` with one first hit and zero repeats at eight and one on my own rebuild. The typed carriers pass 96/96 on 24 builds, the new last-drop group 32/32 on 8 builds (128 observations), the 11 retirement static falsifiers are rejected with `execution_used=false`, and every prior matrix is re-earned on the final source: lookup 160, retirement 144, carrier 80, append 96, handoff 96, append-namespace 96, install-forms 40, confirmed-carrier 96, publication-receiver 64, evaluation-carriers 96 plus 8 live with 25 tests each, the remaining 848 + 72 runs (1,078 + 84 children), components 48, scope 640, calibration 172 in 258 child commands with 86/86 source audits over 43 sources and 86 copies, runtime 8 commands and placement 8 commands at `1433/0/5/2 same_binary=true`; the bodies stay 578/598/496/53/31/210/374 and the frozen-assertion audit 974/974 with zero waivers, and the declaration audit reads `variants=42 controls=374 cells=15708 forced=93 not_forced=15615 new_installation_cells=1596 unknown=0 firing_variants=38 masked_variants=4 all_causal_anchors_exact=true whole_byte_inverses=true execution_used=false`.

The additional `typed-abandon-entries` group (48 items x 2 threads = 96 draws; entries `Operation` and `DirectClaim`; cases `Actual`/`Missing`/`ForeignType`/`OtherLive`; 8 builds) is declared at 04:18:40.098, frozen at 04:18:40.064 and built from 04:18:43.767. All 16 counterpart configurations pass (32 draws) and the first foreign-operation configuration (P0278/form-1, witness, armed, hash) declares `[1,0,0,30]` and reproduces `[0,1,0,30]` at eight and one threads on the same binary `756a06a5…` with exit 101: the challenge replaces only the argument at the `Operation` entry with A's live key and the unguarded `core::forget_operation` forwards it to `retire_operation_frame`, which marks A's operation ID retired and removes A's frame binding. The survivor keeps its record identity, construction record, registered frame and paired token; only the operation-liveness bit changes, so `claim_typed_frame` rejects the retired ID and A's first legitimate resume records zero hits against the required one. The previously repaired `forget_frame` entry reads `first_hits=1` on the same binary; the layer is the scratch consumer with `production_changes=0`; the group is sealed (14 files, `repairs=0`), `candidate_repair=false`, `expectation_repair=false`; 34 of 96 draws are executed and 62 are NOT_RUN (30 foreign-operation, 32 foreign-direct). I reproduced the red from my own copy of the sealed source (`c377feb7…`) at eight and one (0 passed / 1 failed / 30 filtered, the same observation lines).

The ruling `Q64-typed-teardown-entry-actual-receiver-authentication`: at every typed teardown entry that retires an adapter's operation or direct claim the consumer must authenticate the proposal against the actual abandoning adapter's independently recorded identity before any effect, carrying the actual adapter identity alongside the proposed key in the Q63 shape; a live key of another adapter of the same concrete type supplies no authority, and the foreign, missing, mixed-type and foreign-live-key proposals leave the actual outer adapter's operation record, construction record, frame binding, paired token and omission hit untouched. The repair lives in the probe entry or the registered edit set, an explicit before-effect rejection is the declared allowed outcome, and it may not edit the frozen exercise body or its assertions, delete or reclassify a draw, or reclassify the failure; the sealed draw and the queued direct-claim control stay exactly as declared. This is the Q35/Q63 class one entry deeper — a genuine design-level gap of the scratch consumer model, not an instrument-validity defect, not a production regression and not a reclassification.

Continuation 65 must repair the operation and direct-claim retirement entries under Q64 and re-earn the full 96-draw schedule (34 executed, 62 outstanding) preserving the frozen P0278/P0281 control, the sealed draw and the 23 prior controls, keep the repaired retirement-receiver 32/32, the typed carriers 96/96, the last-drop 32/32, the 974/974 assertions, the bodies, the 64 publication draws, the 172 calibration draws with 86/86 source audits, both placements at `1433/0/5/2`, the 96 E draws plus 8 live and the 40/96 install-form and confirmed-carrier draws exact, re-run every matrix on the final source, and then continue the bar from 409/4,422 qualified obligations, 32/134 actors, 4/133 fields and 0/26 complete carrier groups through 134/4,422/133, the 151 plans, the 94 paths, the 188 postimages and the 151-cell schema, then the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder.

The complete bar stays unexecuted: plans 0/151, emitted paths 0/94, postimages 0/188, the schema is unchanged (`f9604852…`, 90 preimages, 180 postimages, 151 immutable rows), 4,015 obligations outside the named union, typed construction is 2 of 73 forms with other construction forms and deferred storage not credited, the standing replay is NOT_RUN and the workspace is NOT_RUN at 0/6,000 s. Allocation: continuation 64 charged 5,703.509053393995 s (3,613.5090533939947 s measured over 3,660 receipts — 77 direct/driver and 3,583 embedded child runs, zero problems — + 1,830.0 s bookkeeping + 200.0 s conservative inspections/setup + the fixed 60 s reserve) of 19,993.105053099 s, leaving 14,289.595999705005 s; CM4 stood at 92,506.894946901 of 112,500 s and now stands at 98,210.40400029499 s. No new allocation: continuation 65 runs under the remaining 14,289.595999705005 s and the assigned 6,000 s workspace (0/6,000), and the 1,440 s verification and second 6,000 s workspace stay held. The stopped scene is 22,305 files / 12,645,053,095 bytes; I rehashed all 22,305 original/copy pairs (44,610 hashes), the 14 sealed stopped-draw entries (28 hashes) and the c63 tree (21,885 entries, 43,770 hashes) with zero missing and zero bad; the 405 director-replay transients are unchanged. Entry authenticates 5,400 maintained / 590 design / 634,623 accepted files (590,836 carried re-hashed plus 43,787 c63 outputs) with `inherited_document_drifts=3` (exactly my three committed c63 documents at HEAD `95f80c2c`) and `production_changes=0`; the authentication pins 90 preimages, 180 postimages, 151 immutable rows and the unchanged schema. I verified the plain `check` (OK, 6,131 citations), all 3,660 receipts with the measured sum exactly 3,613.5090533939947 s, the red at eight and one from my own build of the sealed fixture (`c377feb7…`), the repaired frame-retirement group from my own build (`28207408…`, 19/19 at eight and one), the last-drop group (`0c8da868…`, 4/4) and the runtime-role counterpart class (`fa68d092…`, 29/29 with 2 filtered), every declared draw count against its matrix log, the 974/974 audit line, the bodies, the declaration and source-pair audits, both placements, the eight live traces, and the scope of all 44,630 changes (all under `.temp/brief-742`, zero deleted, zero outside). Run record: `cdx` unit `brief-742-cdx-20261119-032647-902386`, rc 0, peak 7,127,490,560 B, oom 0 / oom_kill 0; log 2,865,597 B, sha256 `536ea8e4cd74fb65056d9e2e6688e0e7d1f47b0eba2f571a5d07ea2af12e7edb`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 44,630 changes (44,628 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted, all under `.temp/brief-742`), 51 PASS / 2 FLAGGED / 2 FAIL / 2 NA, 6,131 citations, 0 questions, 2 deviations, 0 disclosures; no git or network command ran and no `.git-attempts` exists.

## Continuation 65 ruling (director) — the Q64 teardown-entry receiver repair is delivered at 96/96 draws, the retirement family passes 736/736 and every prior matrix is re-earned — and then the new family-frame-bindings calibration stops under Q28 on an under-declared field dependency. The ruling is `Q65-frame-binding-calibration-failure-set-derivation`; the Q64 repair, its 96 draws, the repaired groups, the calibration, the audits and the matrices earn focused scratch credit while the field, the complete groups and the complete bar earn nothing. Full record in `.temp/director/ruling-c65.md`.

Both typed teardown entries now authenticate before any effect: `live_teardown_for_adapter<F: 'static>(actual: &Arc<F>, proposed: (usize, &'static str)) -> Option<(usize, &'static str)>` calls the Q63 `retirement_for_adapter` (the actual adapter's Arc key must equal the proposal, a live `OperationKind::Direct` record of that key must exist, `token.frame_identity == Some(actual_key)`, `current_frame_records[token] == Some(true)`) and then requires the operation ID not to be already retired; `abandon` and `Drop` compute the operation and direct-claim retirements before the first effect and call `forget_operation`/`forget_direct_claim` only through the authenticated result, with the frozen exercise, the `sibling_retire` tests and every assertion byte-identical (`whole_ast_and_byte_inverses=2`). The typed-abandon-entries schedule completes at 96/96 draws (48 configurations x 2 threads, 8 builds): counterparts `[29,0,0,2]` (and `[15,0,0,16]` runtime-disabled), all 16 `foreign-operation` and all 16 `foreign-direct` configurations `[1,0,0,30]`; the 23 prior controls, the frozen P0278/P0281 control and the sealed c64 draw are preserved. Retirement-receiver passes 32/32 (16 configurations, 8 builds; P0278/form-1 witness armed/hash `[19,0,0,0]`), typed carriers 96/96 (24 builds), last-drop 32/32 (128 observations), typed construction 96 plus 8 live, and the retirement family 736/736 (368 configurations, 160 builds). Every prior draw and matrix is re-earned on the final source: lookup 160, retirement 144, carrier 80, append 96, handoff 96, append-namespace 96, install-forms 40, confirmed-carrier 96, publication-receiver 64, evaluation-carriers 96 plus 8 live with 25 tests each, the remaining 848 + 72 runs, components 48, scope 640, the calibration 172/172 in 258 child commands with 86/86 source audits over 43 sources and 86 copies, the 40 projection builds, the runtime driver (8 commands) and the placement runtime (8 commands) at `1433/0/5/2 same_binary=true`; the bodies stay 578/598/496/53/31/210/374 and the literal auditor reads 974/974 with zero waivers, and the declaration audit reads `variants=42 controls=374 cells=15708 forced=93 not_forced=15615 new_installation_cells=1596 unknown=0 firing_variants=38 masked_variants=4 all_causal_anchors_exact=true whole_byte_inverses=true execution_used=false`.

The new `family-frame-bindings` calibration (46 forms x 8 configurations x 2 threads = 736 draws; one typed field `core::Ledger.frames`; one inserted writer statement `l.frames.clear();` at `family-frame-bindings/witness/024.rs:1466`, `changed_writer_statements=1`, `fixed_control_byte_exact=true`, pre-mutation source `18085581…`, mutated source `b7bd3e52…`) passes 45 forms (720 draws) and stops on P1818/form-1, witness/armed/hash: declared `[26,4,0,0]` with the four `retirement_family_controls` and observed `[25,5,0,0]` at eight and one on the same binary `1068e4bf…` (10,863,504 B, exit 101). The additional failure is the existing frozen test `different_actual_targets_have_independent_projected_owners`, which reads `TWO_OWNERS hits=1 required_hits=2` and fails at `024.rs:3798:277` with `left: 1, right: 2`: both source occurrences arm before either owner resumes, the single registered clear in `bind_frame` erases the first adapter's frame binding on the second insertion, the first resume reads an absent frame, `claim_typed_operation` returns an empty permit, and only occurrence 1 reaches the omission. The full-store counterpart passes 30/30 at both thread counts; the four declared two-frame controls each observe only the first frame-binding bit falling, as declared. The declaration's `fixed_source_dependencies` enumerate exactly one fixed test (`actual_nested_producers_keep_one_arm_two_independent_hits` on 046.rs) and `[]` for 024.rs — the analyzer recognized direct two-entry `construction_counts` assertions and omitted the semantic two-target consumer of the same field. The group is sealed immediately (14 files, `repairs=0`), `candidate_repair=false`, `expectation_repair=false`, `stop-context` classifies it `calibration-declaration` with `valid_candidate_failure=false` and `named_additional_failure=1`, and 720 draws matched, 2 were rejected and 14 are NOT_RUN.

The ruling `Q65-frame-binding-calibration-failure-set-derivation`: a collapsed-field calibration is valid only if each configuration's declared failure set is derived from the source as the complete set of frozen tests whose outcome depends on the mutated field — following the field's readers transitively through construction insertion, resume lookup, claim rejection and the hit observation — with each dependent test named, its assertion and its path stated, and the derivation frozen before execution with zero unknown. A declaration that under-enumerates a dependency stops under Q28 even when the mutation fires and every other declaration matches; this is a control-declaration defect, not a candidate-contract violation, not a production regression, not an instrument-validity defect of the frozen tests (they fired correctly) and not a reclassification. The successor declaration may not adopt the observed run post hoc, may not edit the frozen tests, their assertions or the model, may not weaken the group's bar, and may not delete or reclassify the stopped draw; the stopped declaration, both source copies and the stopped binary stay sealed.

Continuation 66 must derive the complete per-form dependency set for `core::Ledger.frames` across all 46 forms with zero unknown and freeze the successor plan before execution, re-run the full 736-draw schedule (720 matched, 2 rejected, 14 outstanding) with the 45 matching forms exact and P1818's eight configurations matching the derived five-failure set at eight and one on one binary per configuration, preserve the stopped declaration and the sealed 14 files, and keep exact by recomputed hash identity wherever every input is byte-identical to c65 (the 2026-11-14 ruling: hash identity exempts repeated checks) — the Q64 96 draws, retirement-receiver 32, typed carriers 96, last-drop 32, the retirement family 736, typed construction 96 plus 8 live, the 974/974 audit (auditor `d416f07d…`, candidate `64c6a43e…`), the bodies, publication 64, the calibration 172 with 86/86 audits, both placements, E 96 plus 8 live, install-forms 40, confirmed-carrier 96, the 40 projection builds and every matrix — re-running anything whose input hash differs, then continue the bar with the paired-token 46-form group, the additional sibling 46-form group, the two-form writer group, the deferred storage handoffs and the 21 remaining retirement forms, and then from 459/4,422 obligations, 34/134 actors, 4/133 fields and 0/26 complete carrier groups through 134/4,422/133, the 151 plans, 94 paths, 188 postimages and the 151-cell schema, then the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder.

The complete bar stays unexecuted: plans 0/151, emitted paths 0/94, postimages 0/188, the schema unchanged (`f9604852…`, 90 preimages, 180 postimages, 151 immutable rows), 4,013 obligations outside the named union, 5 construction fields of 73 forms with 365 cells and 0 complete groups, the standing replay NOT_RUN and the workspace NOT_RUN at 0/6,000 s. Allocation: continuation 65 charged 7,668.439746931021 s (4,653.439746931021 s measured over 5,510 receipts — 30 driver records and 5,480 individual runs, zero problems — + 2,755.0 s bookkeeping + 200.0 s conservative inspections/setup + the fixed 60 s reserve) of 19,993.105053099 s, leaving 6,621.156252773984 s; CM4 stood at 98,210.40400029499 of 112,500 s and now stands at 105,878.84374722601 s; stage reserve 81,000 and 743-746 40,500 unchanged. No new allocation: continuation 66 runs under 6,621.156252773984 s and the assigned 6,000 s workspace (0/6,000), with the 1,440 s verification and second 6,000 s workspace held. The stopped scene is 26,520 files / 16,081,287,014 bytes with `originals_and_copies_exact=true`; I rehashed all 26,520 original/copy pairs (53,040 hashes), the 14 sealed stopped-draw entries and the c64 scene (22,305 pairs, 44,610 hashes) with zero missing and zero bad; the 405 director-replay transients are unchanged. Entry authenticates 5,400 maintained / 590 design / 634,623 accepted files with `inherited_document_drifts=3` (my three committed c64 documents at HEAD `87776228`, verified through git) and `production_changes=0`; the authentication pins 90 preimages, 180 postimages, 151 immutable rows and the unchanged schema. I verified the plain `check` (OK, 9,075 citations), all 5,510 receipts with the measured sum exactly 4,653.439746931021 s and every arithmetic step, the stop from my own builds outside the scene (sealed source `b7bd3e52…` 25/5 at eight and one with the same five names and observations; pre-mutation `18085581…` 30/0; a matching form 10/4), the Q64 repair from my own builds of both roles (counterparts 29/0/2, both foreign entries 1/0/30) plus my build of the c64 unguarded source failing `foreign::operation` with `[true,false,true,true,true]`, the retirement-receiver 19/19, last-drop 4/4, retirement-family 14/14 and typed-carrier 15/15 rebuilds, the 974/974 audit with the inherited auditor, every declared draw count against its matrix log, the declaration ordering, the scope of all 53,060 changes (all under `.temp/brief-742`, zero deleted, zero outside) and the working tree (only the 23 inherited entries). Run record: `cdx` unit `brief-742-cdx-20261119-044439-1256480`, rc 0, peak 9,284,947,968 B, oom 0 / oom_kill 0; log 2,290,600 B, sha256 `8961faefefd8df2169e5fcc664d5222ccd67464f3082a49959e524125e5ea41f`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 53,060 changes (53,058 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted), 60 PASS / 2 FAIL / 2 NA, 9,075 citations, 0 questions, 2 deviations, 0 disclosures; no git or network command ran and no `.git-attempts` exists.

## Continuation 66 ruling (director) — the Q65 successor is delivered at 736/736 draws including P1818's derived five-failure set, the paired-token and sibling entry-receiver groups pass 736/736 each and the bar rises to 551/4,422 — and then the new two-form typed-construction-writer group stops under Q28 on its first missing-key control. The ruling is `Q66-typed-construction-writer-actual-allocation-authentication`; the successor declaration, the two family groups and the writer's 64 matched draws earn focused scratch credit, while the writer claim and the complete bar earn nothing. Full record in `.temp/director/ruling-c66.md`.

The source-derived `core::Ledger.frames` calibration now names every dependent frozen test with its assertion: `family-frame-bindings-dependency-slices.json` (`aaa2ae57...`, 1,933,586 B) indexes 40 positive sources, 808 source-level test entries and 5,996 selected test cells with `unknown=0` and `calibration_execution_used=false`, and the successor plan (`5058298e...`, 1,610,622 B) is frozen at 06:37:36.970, 19.9 s before the first build. P1818's four armed configurations declare five failures — `different_actual_targets_have_independent_projected_owners` (`assert_eq!(hits,2*armed());`, left 1 / right 2) plus the four `retirement_family_controls` — and its four disabled configurations read `[4,0,0,26]`; all eight rows re-ran exact at eight and one on the same binary `1068e4bf...`, byte-identical to the c65 stopped binary, and the full schedule is 736/736 with 368 configurations exact (45 forms / 364 configurations unchanged, exactly four configurations gaining the armed hit dependency). Seven source falsifiers x two roles reject the under-enumeration shapes with `source_only=true candidate_execution=false`. The paired-token group (`core::Pair.tokens`, 46 forms) passes 736/736 with its plan frozen before its build and `failure_sets_and_assertions_exact=true unknown=0`; the `actual-and-foreign-sibling-entries` group passes 736/736 over 46 forms, two registered actors and eight added physical proposals, joining 92 source obligations so the union rises from 459 to 551/4,422. The `typed-construction-writer` group (two forms P0278/form-1 and P0281/form-1 x 32 configurations x 2 threads = 128 draws; actor `core::bind_frame`; 31 existing fixed tests) passes its 64-draw `actual`/`other_live` prefix on one binary per configuration and then stops on P0278/form-1, witness, armed, hash, class `missing`: declared `[1,0,0,34]`, observed `[0,1,0,34]` at eight and one on the same binary `41b1bd66...` (10,792,568 B, exit 101). The challenge transports `(0, actual_type)` into `bind_frame`; the empty key passes the occupied-key check; the writer stamps the proposal into the pending token and inserts the direct operation, paired token, live construction record and frame binding under the missing key (`foreign_records_changed=true actual_issued=false explicit_rejection=false`), while the actual adapter receives none and the surviving outer owner keeps its five properties and its first hit. The group is sealed immediately (14 files, `repairs=0`); `stop.json` classifies it `candidate-contract` with `candidate_failure=true`, `candidate_repair=false`, `expectation_repair=false` and `production_changes=0`; 66 draws executed (64 matched, 2 rejected) and 62 NOT_RUN (30 Missing, 32 MixedType). The ruling `Q66-typed-construction-writer-actual-allocation-authentication`: at the typed construction writer the consumer must authenticate the proposal against the actual newly allocated adapter's recorded identity (its physical `Arc` and genuine pending source) before any direct operation, frame binding, construction record or paired token is issued; a missing / foreign / mixed-type proposal supplies no authority, and the compliant outcomes are a well-bound physical allocation that writes the records for the actual adapter or an explicit before-effect rejection that leaves every prior record set unchanged and the other live owner's five properties, native results and once-only hit intact. This is the Q35/Q63/Q64 receiver-authentication class one layer deeper — a genuine design-level defect of the scratch consumer model, not an instrument-validity defect (the control compiled and fired correctly), not a production regression and not a reclassification; the repair lives in the probe entry or the registered edit set and may not edit the frozen controls or assertions, delete or reclassify a draw, or weaken the bar. Continuation 67 must repair the writer under Q66, re-earn the complete 128-draw schedule (66 executed, 62 outstanding) with both forms and the frozen declaration exact, preserve the sealed c66 and c65 draws, keep the Q65 successor declarations exact (family-frame-bindings 736/736 including P1818's five-failure set, family-paired-tokens 736/736, family-entry-receivers 736/736), keep everything prior exact by recomputed hash identity wherever every input is byte-identical (the 2026-11-14 ruling) — the Q64 96 draws, retirement-receiver 32, typed carriers 96, last-drop 32, the retirement family 736, typed construction 96 plus 8 live, the 974/974 audit (auditor `d416f07d...`, candidate `64c6a43e...`), the bodies, publication 64, the calibration 172 with 86/86 audits, both placements, E 96 plus 8 live, install-forms 40, confirmed-carrier 96, the 40 projection builds and every matrix — re-running anything whose input hash differs, then continue the bar with the deferred storage handoffs and the 21 remaining retirement forms (P0256, P0259, P0292, P0314, P0315, P0394, P0396, P0410, P0413, P0494, P0498, P0554, P0814, P1139, P1142, P1147, P1155, P1477, P1814, P1817, P1937), and then from 551/4,422 obligations, 34/134 actors, 4/133 fields and 0/26 complete groups through 134/4,422/133, the 151 plans, 94 paths, 188 postimages and the 151-cell schema, then the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder. The complete bar stays unexecuted: plans 0/151, emitted paths 0/94, postimages 0/188, the schema unchanged (`f9604852...`, 90 preimages, 180 postimages, 151 immutable rows), 3,871 obligations outside the named union, the 21 remaining retirement forms, the standing replay NOT_RUN and the workspace NOT_RUN at 0/6,000 s. Allocation: continuation 66 charged 4,110.386052918038 s (2,387.386052918038 s measured over 2,926 receipts — 2,918 individual runs and 8 drivers with 2,884 child links — + 1,463.0 s bookkeeping + 200.0 s inspections/setup + the 60 s reserve) of 6,621.156252773984 s, leaving 2,510.770199855946 s; CM4 stood at 105,878.84374722601 of 112,500 s and now stands at 109,989.22980014405 s. **New allocation**: the remaining 2,510.77 s cannot cover the writer repair, the 128-draw re-earn, the 21 retirement forms, the deferred handoffs and the complete-bar generation, so this gate allocates a fresh 12,000 s focused envelope from the project stage reserve — continuation 67 runs under 14,510.770199855946 s, CM4's share becomes 124,500 s, the stage reserve 69,000 s, and 743-746 keep 40,500 s; the assigned 6,000 s workspace stays at 0/6,000 and the 1,440 s verification and second 6,000 s workspace stay held. The stopped scene is 6,661 files / 5,557,997,066 bytes with `originals_and_copies_exact=true` and `candidate_repair=false`; entry authenticates 5,400 maintained / 590 design / 732,307 accepted files with `inherited_document_drifts=3` (my three committed c65 documents at HEAD `7c364a21`) and `production_changes=0`, pinning 90 preimages, 180 postimages, 151 immutable rows and the unchanged schema. I reproduced the stop from my own build of the sealed writer source at eight and one (0/1/0/34, the same observation and the same line-750 assertion) and its actual/other_live passes, the Q65 successor from my own build of `b7bd3e52...` (25/5 at eight and one, same five names, same left 1 / right 2 at line 3798), one paired-token draw (10/4 with its four controls) and one entry-receiver draw (22/0), verified the plain `check` (1,491 citations), all 2,926 receipts summing exactly 2,387.386052918038 s, all 6,554 inherited-evidence hashes, the 6,661 scene pairs (13,322 hashes), the 14 sealed c66 entries (28 hashes), every declared draw count, the declaration ordering, the scope of all 13,342 changes and the working tree. No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts, C2 118 Residual + 0 Lost, 51 owners and five source duties Open, `CHANGELOG.md` and S10 blocked.

## Continuation 67 ruling (director) — the Q66 writer repair passes its complete 128-draw schedule and the 11 group matrices re-run green on the post-edit renders, and then the frozen source-pair examiner stops the stage because the repair added one assertion literal to a frozen pair. The ruling is `Q67-typed-construction-writer-assertion-free-control-flow-refusal`; continuation 67 earns nothing, because every draw ran on renders derived from an inadmissible edit. Full record in `.temp/director/ruling-c67.md`.

The Q66 repair is functionally correct. `bind_allocated_frame<F: 'static>(actual: &Arc<F>, proposed: (usize, &'static str))` derives the physical key from the actual allocation's address and the concrete type, and the frozen `erase` call site now passes the actual `Arc`; `bind_frame` is reached only when the proposal equals the physical identity. The group passes **128/128 draws** over 64 configurations (both forms P0278/form-1 and P0281/form-1, both roles, both arms, both modes) at eight and one on one binary per configuration (8 builds): `actual` issues the five records to the physical allocation, `other_live` / `missing` / `mixed_type` change no prior record set, issue nothing to the actual allocation, keep the surviving outer owner's five properties, native results and once-only hit, and the 64 `actual`/`other_live` traces are byte-identical to the c66 passing traces. The 11 group matrices re-ran on the post-edit renders: retirement-family 736/736 (160 builds), family-paired-tokens 736/736, family-frame-bindings 736/736 including P1818's five-failure set, family-entry-receivers 736/736, typed-carriers 96/96, evaluation-carriers 96/96, typed-abandon-entries 96/96, typed-last-drop 32/32, retirement-receiver 32/32, publication-receiver 64/64 and typed-construction-writer 128/128 — 1,744 configurations, 3,488 draws, zero stops. The repair's shape has its own instruments: 239 candidate renders with byte diffs and inverses, 57 allocation AST inverse pairs, and an allocation calibration rejecting 14 mutated shapes and accepting 2 before execution; the literal recheck schedule was declared before execution as 646 unchanged inputs (hash-identity exempt) plus 56 directly changed inputs × 2 threads = 112 draws.

The stop is honest and reproduced by me. All 56 changed inputs carry the new writer, and the first examined — pair 022/witness, candidate `8429ee53...` (180,661 B) — is rejected by the unchanged frozen examiner (`4590281f...`): required **199** assertion literals, found **200**, exit 101 at eight and one on the same auditor, while the prior c65 render (`22366ffb...`) passes at 199. My own run of the sealed examiner on director copies reads left 199 / right 200, with the inserted literal `assert!(!active, "PortBlocked::ForeignFrameConstructionAllocation")` at ordinal 28 (0-based) in `cm740_probe::bind_allocated_frame` (candidate line 1292) and every original assertion text preserved in order. The `active` check exists only to produce that panic, and the panic requirement was authored by the run itself: the c67 `observations.py` (new — the c66 comparator compared no writer line) requires `explicit_rejection=true` for every foreign recording case. The group is sealed (15 files, `originals_and_copies_exact=true`), `stop.json` is `candidate-contract` `PortBlocked::TypedConstructionWriterFrozenAssertionSequence` with `candidate_repair=false`, `expectation_repair=false` and `production_changes=0`; 2 of 112 draws executed, 110 NOT_RUN.

The ruling `Q67-typed-construction-writer-assertion-free-control-flow-refusal`: at the typed construction writer the authenticated refusal is **control flow**, not an unwind — the writer derives the physical identity from the actual allocation (its address and concrete type) and, when the proposal differs, issues nothing and returns; it may not add any `assert`/`assert_eq`/`assert_ne` macro, or any other panicking termination, to a frozen source pair. A foreign proposal is a state the types allow; the receiver-authentication shape accepted at Q63/Q64/Q65 is the authenticated absence, and the frozen source-pair assertion sequence is binding on every repair, including its own instruments. The frozen writer's own assertions — including `PortBlocked::DuplicateLiveFrameConstruction` for an occupied frame key — stay frozen; because the authentication refusal precedes the writer, the c66 `other_live` trace's `explicit_rejection=true` (produced by that frozen occupied-key assertion, not by the defect) becomes `explicit_rejection=false`, while every contract field stays identical: prior record sets unchanged, no records for the actual allocation, the survivor's five properties, native results and once-only hit, and `actual` still issuing to the physical allocation. The writer group's comparator must be re-derived: `explicit_rejection` is a diagnostic of unwinding, expected false in every class, and may not be required as a panic. The registered edit set and its allocation auditor are re-derived to the assertion-free text before execution, and the auditor's calibration must still reject under-authenticated shapes. This is not an instrument-validity defect (the examiner ran exactly as contracted), not a production regression and not a reclassification; the frozen declaration, both 022 sources, the candidate render, the examiner, the two draws and their receipts stay sealed.

I demonstrated the admissible representation rather than asserting it: on director copies outside every sealed scene I replaced the `active`/`assert!` block with `if proposed != physical { return; }` in both the pair render and the writer witness. The unchanged frozen examiner then reads `199 assertions byte_exact=true` at exit 0, and my build of that witness passes all four classes at eight and one — `actual` issuing to the physical allocation, the three foreign classes changing nothing (`foreign_records_changed=false`, `actual_issued=false`), the survivor keeping `[true,true,true,true,true]`, the hits `1,0`, and `explicit_rejection=false` everywhere.

Continuation 68 must re-render the writer edit assertion-free (template, registered edits, allocation auditor, 239 renders), re-freeze the plan and the re-derived comparator, and **run the literal examiner first** on all 56 changed inputs (112 draws) against the frozen bases at eight and one; only then the allocation auditor and its calibration, then the 128 writer draws with both forms and the frozen counts/failures, then the 11 group matrices on the new renders, then the pending matrices (source-lookup 64 of 160, carrier-calibration 32 of 80, p0558 8, route-family-runs 448 of 848, the 32 pending projection builds with 4 controls / 8 control draws, and the deferred 3 source boundaries / 4 forms / 12 obligations); then the deferred storage handoffs and the 21 remaining retirement forms (P0256, P0259, P0292, P0314, P0315, P0394, P0396, P0410, P0413, P0494, P0498, P0554, P0814, P1139, P1142, P1147, P1155, P1477, P1814, P1817, P1937, all form-1), and from 551/4,422, 34/134, 4/133 and 0/26 through 134/4,422/133, the 151 plans, 94 paths, 188 postimages and the 151-cell schema, then the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder. The ordering is binding: the cheapest frozen contracts run first, so an inadmissible representation stops in minutes, not after the matrices. Everything prior stays exact by recomputed hash identity wherever every input is byte-identical (the 2026-11-14 ruling); anything whose input hash differs is re-run.

Continuation 67 earns nothing: the writer's 128 draws and the 11 re-earned matrices ran on renders derived from an inadmissible edit, so the writer group's claim has not been earned. The accepted bar stands at **551/4,422** qualified obligations, **34/134** actors, **4/133** fields and **0/26** complete groups; the two writer obligations (489, 521) and the actor `core::bind_frame` are recorded provisionally (553/4,422, 35/134) and count only when re-earned on the admissible render. The complete bar stays unexecuted: plans 0/151, emitted paths 0/94, postimages 0/188, the schema unchanged (`f9604852...`, 90 preimages, 180 postimages, 151 immutable rows), 3,869 obligations outside the named union, the standing replay NOT_RUN and the workspace NOT_RUN at 0/6,000 s.

Allocation: continuation 67 charged **5,314.931663071038 s** (2,901.931663071038 s measured over 4,289 receipts — 4,276 individual runs and 13 driver records with 4,221 verified child links, zero problems — + 2,153.0 s bookkeeping (0.5 s × 4,289 receipts plus 0.5 s × 17 nested auditor invocations whose elapsed is inside their parents) + 200.0 s conservative inspections/setup + the fixed 60 s reserve) of 14,510.770199855946 s, leaving **9,195.838536784908 s**; CM4 stood at 109,989.22980014405 of 124,500 s and now stands at **115,304.16146321509 s**. No new allocation: continuation 68 runs under the remaining 9,195.838536784908 s and the assigned 6,000 s workspace (0/6,000), with the 1,440 s verification and second 6,000 s workspace held; stage reserve 69,000 s and 743-746 keep 40,500 s. The stopped scene is **9,983 files / 8,154,360,979 bytes** with `originals_and_copies_exact=true`, `candidate_failure=true` and `candidate_repair=false`; I rehashed all 9,983 original/copy pairs (19,966 hashes) and the 15-file sealed stopped draw (30 hashes) with zero bad, the c65 and c66 sealed draws (14 + 14 entries) are intact, the c66 tree has no file modified after 07:10:21, and the 405 director-replay transients are unchanged with `unexplained=0`. Entry authenticates 5,400 maintained / 590 design / 745,646 accepted files with `inherited_document_drifts=3` (my three committed c66 documents at HEAD `41c3697b`, verified through git) and `production_changes=0`, pinning 90 preimages, 180 postimages, 151 immutable rows and the unchanged schema; the working tree carries only the 23 inherited porcelain entries. I verified the plain `check` (2,134 citations), all 4,289 receipts summing exactly 2,901.931663071038 s and every arithmetic step of the charge, the remainder and the CM4 total, the stop from the sealed examiner on the sealed sources (prior 199 PASS at exit 0, candidate 199 vs 200 at exit 101 at eight and one, the inserted literal at ordinal 28), my own build of the writer witness `a913e7a6...` passing all four classes at eight and one with exact evidence lines, my assertion-free copy passing the examiner at 199 and the same four classes with `explicit_rejection=false`, all 11 matrix logs' PASS counts equal to their plans' items with exit-0 drivers, every plan and freeze preceding its first build, the 56 changed inputs hash-exact and all containing the new writer, the c65/c66 sealed draws, the auditor `d416f07d...` and the calibration candidate `64c6a43e...` unchanged so the 974/974 audit stands by hash identity, the three document drifts through git, the scope of all 19,986 changes (all under `.temp/brief-742`, zero deleted, zero outside, zero missing on disk) and the working tree. Continuation 67 ran `cdx` (unit `brief-742-cdx-20261119-072529-1740578`, rc 0, peak 6,545,141,760 B, oom 0 / oom_kill 0; log 2,311,500 B, sha256 `e61be9b1...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 19,986 changes — 19,984 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted, all under `.temp/brief-742` — 60 PASS / 2 FAIL / 2 NA, 2,134 citations, 0 questions, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts, C2 118 Residual + 0 Lost, 51 owners and five source duties Open, `CHANGELOG.md` and S10 blocked. Next: continuation 68.

## Continuation 68 ruling (director) — the Q67 assertion-free writer is re-rendered and every schedule is re-earned (112 literal draws, 128 writer draws, the 11 group matrices, 552 rebind draws and the projection set), and then the new deferred-cursor writer stops on its first missing-key control under Q28. The ruling is `Q68-deferred-cursor-writer-physical-cursor-authentication`; the writer obligations (489, 521) and the actor `core::bind_frame` are earned (553/4,422, 35/134), and the deferred claim earns nothing. Full record in `.temp/director/ruling-c68.md`.

The Q67 repair is delivered on the admissible render. `bind_allocated_frame<F: 'static>(actual: &Arc<F>, proposed: (usize, &'static str))` derives the physical key from the actual allocation's address and concrete type and refuses with `if proposed != physical { return; }` before `bind_frame(physical.0, physical.1)`; the frozen `erase` call site passes the actual `Arc`; no assertion was added and the frozen writer's own assertions are unchanged. The literal examiner ran **first**: 56 changed role renders × 2 threads = **112 draws** declared before execution (646 unchanged inputs exempt by hash identity), all PASS with the unchanged examiner `4590281f...`; pair 022/witness — the input that stopped c67 at 200 — now reads **199 assertions byte_exact=true** at exit 0 at eight and one. The typed-construction-writer group passes **128/128 draws** over 64 configurations (P0278/form-1 and P0281/form-1 × witness/runtime × armed/disabled × hash/nohash) at eight and one on one binary per configuration (8 builds), all 128 traces exact with `explicit_rejection=false` everywhere; my own build of the witness render passes both forms × four classes × eight and one. The 11 group matrices re-ran on the admissible renders: retirement-family 736/736, family-paired-tokens 736/736, family-frame-bindings 736/736 including P1818's five-failure set, family-entry-receivers 736/736, typed-carriers 96/96, evaluation-carriers 96/96, typed-abandon-entries 96/96, publication-receiver 64/64, typed-last-drop 32/32, retirement-receiver 32/32 and typed-construction-writer 128/128 — 3,488 draws, zero stops. The pending matrices executed: lookup-writer-rebind 64 of 160 (96 unchanged), carrier-writer-rebind 32 of 80 (48 unchanged), p0558-writer-rebind 8 and main-writer-rebind 448 of 848 (400 unchanged) — 552 draws matched, with the 272 identical-input declarations (544 logs) hash-exact. The projection set passes (32 changed builds, 8 unchanged, 4 typed-contract builds, 8 typed-contract draws), and the allocation auditor was re-derived assertion-free: calibration `positive=2 rejected=14 before_execution=true`, all 239 inverse pairs exact; my own build passes its positive and rejects the earlier-effects falsifier at exit 101.

The stop is honest and reproduced by me. The new deferred-cursor-writer group (P2132–P2135, all form-1, × 2 roles × 2 arms × 2 modes × 5 classes = 160 configurations / **320 draws**; 35 frozen original tests plus 5 added controls per source; actor `core::Guard::defer`) was declared before execution (plan 09:15:43, freeze 09:15:44, first build 09:16:00; admitted at required 1,818.5 of 3,839.604 s) and matched 96 configurations (32 ordinary, 32 actual, 32 other_live) before stopping on **P2132/form-1, witness, armed, hash, class missing** (item 97): declared `[1,0,0,39]`, observed `[0,1,0,39]` at eight and one on the same binary `cf2013219e...`. The transport presented address zero with the actual concrete type while both cursors stayed live and the pending source was genuine; `Guard::defer` inserted both deferred records under the proposed key — `foreign_records_changed=true` — and issued nothing to the actual Box, which kept its native result and zero hits while the outer kept both records, its native result and one hit. The failing assertion is the contract (`a deferred cursor writer may only issue both records for its actual Box and genuine pending source`, witness.rs:789). The group is sealed immediately (18 files, `originals_and_copies_exact=true`); 194 of 320 draws executed (192 matched + 2 rejected), 126 NOT_RUN; `candidate_repair=false`, `expectation_repair=false`, `production_changes=0`; the sources were last written 09:15:43 and the seal is at 09:17:37, so nothing was repaired after the failure. My own build of the sealed source (`754bbd63...`) reproduces it at eight and one: `actual` and `other_live` pass, `missing` and `mixed_type` both exit 101 with `foreign_records_changed=true actual_issued=false` — the class is the whole foreign-proposal family, and `other_live` is protected only by the frozen duplicate assertion, not by authentication. The deferred auditor is calibrated (`positive=2 falsifiers_rejected=12 original_controls=35`); my own build passes the positive and rejects the self-copied-proof falsifier. The group's comparator is already unwinding-agnostic — it accepts a non-unwinding refusal and never requires a panic — so the stopped draw differs only in `foreign_records_changed`.

The ruling `Q68-deferred-cursor-writer-physical-cursor-authentication`: at the deferred construction writer (`Guard::defer`) the consumer must authenticate the proposal against the actual boxed cursor's physical identity — its address and concrete type — and its genuine pending source before issuing either deferred record (`deferred_proofs` and `deferred`). A missing key (even with the actual concrete type and a genuine pending source), a foreign key, or a mixed-type key supplies no authority. The compliant outcomes are a well-bound physical write that issues both records for the actual Box, or an **assertion-free pre-effect refusal** — control flow, not an unwind, per Q67 — that issues nothing, leaves every prior record set unchanged, and leaves the actual cursor with its native result while the other live Box keeps both prior records, its native behavior and its single armed hit. The refusal may not be implemented with `assert`/`assert_eq`/`assert_ne` or any other panicking termination; the frozen duplicate assertion `PortBlocked::DuplicateLiveDeferredConstruction` stays frozen, and because authentication precedes the writer the `other_live` trace's `inner_rejected=true` (produced by that frozen assertion, not by the defect) becomes false under the admissible representation with `inner_native=Some(true)`, while every contract field stays identical. The comparator stays unwinding-agnostic (`inner_rejected` a diagnostic, never required), the registered edit set and the deferred auditor are re-derived to the assertion-free text before execution, and the auditor's calibration must still reject under-authenticated shapes. This is the Q63/Q64/Q65/Q66/Q67 receiver-authentication class one layer deeper — the deferred storage handoff — a genuine design-level defect of the scratch consumer model, not an instrument-validity defect, not a production regression and not a reclassification: the candidate writer wrote to a foreign key under a genuine pending source. The successor may not adopt the observed run post hoc, may not edit the frozen 35 tests or their assertions, may not weaken the bar, and may not delete or reclassify the stopped draw; the stopped declaration, both sources, the fixture binary, the two draws and their receipts stay sealed.

Continuation 69 must re-render the deferred writer edit assertion-free (the physical identity derived from the actual cursor and required to equal the transported proposal before either record is issued; foreign/missing/mixed proposals return without effect), re-derive the registered edits and the deferred auditor before execution with the calibration still rejecting under-authenticated shapes, re-freeze the plan and comparator and run the full 320-draw schedule (194 executed: 192 matched + 2 rejected; 126 outstanding) at eight and one on one binary per configuration (8 builds), preserving the sealed 18 files; then the `construct_deferred` and `DeferredEvaluationGuard::drop` handoffs on all four forms (8 obligations: construct_deferred ×4 unpaid, DeferredEvaluationGuard::drop ×4 already in the union but the four-form gate incomplete), then the 21 remaining retirement forms (P0256, P0259, P0292, P0314, P0315, P0394, P0396, P0410, P0413, P0494, P0498, P0554, P0814, P1139, P1142, P1147, P1155, P1477, P1814, P1817, P1937, all form-1), and from 553/4,422, 35/134, 4/133 and 0/26 through 134/4,422/133, the 151 plans, 94 paths, 188 postimages and the 151-cell schema, then the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder. Everything prior stays exact by recomputed hash identity wherever every input is byte-identical (the 2026-11-14 ruling); anything whose input hash differs is re-run.

The accepted bar rises to **553/4,422** qualified obligations, **35/134** actors, **4/133** fields and **0/26** complete groups — the writer obligations (489, 521) and the actor `core::bind_frame` are now earned on the admissible render rather than provisional. The deferred group earns nothing: 96 of 160 configurations matched, 194 of 320 draws executed; the 12 deferred rows stay unpaid (8 outside the union — `core::Guard::defer` ×4 and `core::construct_deferred` ×4 — and 4 `DeferredEvaluationGuard::drop` rows in the union whose four-form gate is not completed). The complete bar stays unexecuted: plans 0/151, emitted paths 0/94, postimages 0/188, the schema unchanged (`f9604852...`, 90 preimages, 180 postimages, 151 immutable rows), 3,869 obligations outside the named union, the standing replay NOT_RUN and the workspace NOT_RUN at 0/6,000 s.

Allocation: continuation 68 charged **6,415.2339194878 s** (3,465.733919487800449 s measured over 5,348 receipts — 5,330 individual runs and 18 driver records with 5,252 verified child links, zero problems — + 2,689.5 s bookkeeping (0.5 s × 5,348 receipts plus 0.5 s × 31 nested auditor invocations whose elapsed is inside their parents) + 200.0 s conservative inspections/setup + the fixed 60 s reserve) of 9,195.838536784908 s, leaving **2,780.6046172971073 s**; CM4 stood at 115,304.16146321509 of 124,500 s and now stands at **121,719.39538270289 s**. **New allocation**: a fresh **12,000 s** focused envelope from the project stage reserve — continuation 69 runs under **14,780.6046172971073 s**, CM4's share becomes **136,500 s**, the stage reserve **57,000 s**, and 743-746 keep 40,500 s; the assigned 6,000 s workspace stays at 0/6,000, and the 1,440 s verification and second 6,000 s workspace stay held. The stopped scene is **12,620 files / 11,252,306,416 bytes** (12,620 original/copy pairs; 22.5 GB on disk) with `originals_and_copies_exact=true`, `candidate_failure=true`, `candidate_repair=false` and `previous_scenes_repaired=false`; I rehashed all 12,620 pairs and the 18-file sealed stopped draw (36 hashes) with zero bad, the c65/c66 (14 + 14 entries) and c67 (15 entries) sealed draws are exact, the c67 scene rehashes clean (9,983 pairs), and the 405 director-replay transients are unchanged with `unexplained=0`. Entry authenticates 5,400 maintained / 590 design / 765,629 accepted files with `inherited_document_drifts=3` (my three committed c67 documents at HEAD `ea31d6df`, verified through git) and `production_changes=0`, pinning 90 preimages, 180 postimages, 151 immutable rows and the unchanged schema; the working tree carries only the 23 inherited porcelain entries. I verified the plain `check` (2,845 citations), all 5,348 receipts summing exactly 3,465.733919487800449 s with the charge, remainder and CM4 total exact, all 112 literal draws on the declared 56 changed renders at eight and one, my own build of the writer witness `c685e931...` passing both forms × four classes × eight and one, all 15 full matrix logs' PASS counts equal to their plans' items with exit-0 drivers and the deferred log's 96 PASS + 1 STOP at 194 draws, every plan's source hashes on disk and every declaration preceding its first build, the 272 identical-input declarations hash-exact, my own build of the sealed deferred source reproducing the stop at eight and one (missing and mixed_type 101, actual and other_live 0), both auditors' positives and falsifiers, the scene and sealed draws, the entry drifts through git, the 405 director drifts, the scope of all 25,264 changes (all under `.temp/brief-742`, zero deleted, zero outside) and the working tree. Continuation 68 ran `cdx` (unit `brief-742-cdx-20261119-083155-1850718`, rc 0, peak 7,800,332,288 B, oom 0 / oom_kill 0; log 3,057,693 B, sha256 `3792eaf3...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 25,264 changes — 25,262 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted, all under `.temp/brief-742` — 79 PASS / 3 FAIL / 2 NA, 2,845 citations, 0 questions, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts, C2 118 Residual + 0 Lost, 51 owners and five source duties Open, `CHANGELOG.md` and S10 blocked. Next: continuation 69.

## Continuation 69 ruling (director) — the Q68 writer repair is delivered at 320/320 draws, the deferred writer obligations and the actor `core::Guard::defer` are earned (557/4,422, 36/134), and then the new deferred-cursor reader stops under Q28 on the foreign live cursor because `construct_deferred` removes both deferred records before authenticating the proposal. The ruling is `Q69-deferred-constructor-physical-cursor-authentication`; the reader claim earns nothing. Full record in `.temp/director/ruling-c69.md`.

The Q68 repair is delivered on the admissible render. `Guard::defer` derives `physical = (cursor as *const T as usize, type_name::<T>())`, transports it as the proposal, and refuses with `if key != physical { return; }` **before** the frozen duplicate assertion and before either deferred map is written. The diff against the sealed c68 stopped render is +80 bytes per role (`754bbd63...` -> `4a586cf6...`, 240,253 B; `4468aaf7...` -> `df955c10...`, 240,284 B; q68 diffs 973 B each). No assertion was added: the literal examiner reads **266 assertion literals byte_exact=true** on both roles at eight and one, and the frozen 35 tests restore byte-exactly with 5 added controls.

The deferred-cursor-writer group passes **320/320 draws** over 160 configurations (P2132-P2135, all form-1, x 2 roles x 2 arms x 2 modes x 5 classes) at eight and one on one binary per role and mode (8 builds). Every class — ordinary, actual, other_live, missing, mixed_type — contributes 64 draws, all at `[1,0,0,39]`; `nonunwinding_traces=256`, `intact_survivors=256`, `repeated_resume_hits=0`; the foreign classes emit `inner_rejected=false inner_native=Some(true)`, the Q67/Q68 assertion-free representation. My own builds of the two renders reproduce all 320 draws exactly, and all 256 non-ordinary observation lines (`CM742C68 DEFERRED WRITER ...`) are byte-identical to the worker's sealed logs. The writer auditor was re-derived to the assertion-free text (2 positives accepted, 26 falsifiers rejected, 35 original controls preserved); my own build reproduces all 28, and four falsifiers I generated myself (missing-physical-check, unwinding-refusal, post-effect refusal, transport-dropped) are all rejected at exit 101. The four `Guard::defer` obligations (3703, 3726, 3749, 3772) and the actor `core::Guard::defer` are **earned**: the accepted bar rises from 553 to **557/4,422** qualified obligations, from 35 to **36/134** actors, **4/133** fields and **0/26** complete groups.

The stop is honest and reproduced by me. The new deferred-cursor-reader group (P2132-P2135 x 2 roles x 2 arms x 2 modes x 5 classes = 160 configurations / **320 draws**; 40 frozen original tests plus 4 added controls per source; actor `core::construct_deferred`; obligations 3705/3728/3751/3774, outside the union; source boundaries callback, pending.take, constructor and real_frame_resume) was declared before execution (plan, freeze and declaration 10:05:37, auditor build 10:05:39, calibration 10:05:45, first build 10:05:54) and matched 64 configurations (ordinary 32, actual 32; 128 draws) before stopping on **P2132/form-1, witness, armed, hash, class other_live** (item 65): declared `[1,0,0,43]`, observed `[0,1,0,43]` at eight and one on the same binary `73f23db3...` (141,462,328 B). The observation is `physical_equals_proposed=false genuine_source=true previous_occupied=true claim=Foreign actual_consumed=false foreign_changed=true survivor=false inner_native=false inner_forwarded=true inner_hits=1 outer_native=true outer_forwarded=false outer_hits=0 repeated_resume_hits=0`, and the failing assertion is `construct_deferred may consume only the actual pending Box's records` (witness.rs:771:9). At the real pending.take handoff both live boxes had distinct addresses, the same concrete type and distinct genuine pending tokens; the control varied only the constructor's proposed key. `construct_deferred` computes the key from the cursor, transports it, and then **removes both deferred records at the transported key before evaluating the token, the proof and `pending_construction`**. Proposing the other live cursor's key therefore consumed that Box's two records (`foreign_changed=true`, `survivor=false`) and installed construction and evaluation provenance naming the foreign source (`claim=Foreign`); the inner resume forwarded and took the hit while the outer resume took the ordinary path with zero hits. The group is sealed immediately: **27 files** (stop record, plan, freeze, matrix log, results, edits, driver record, preflight, both renders and their diffs, the fixture binary and its build records, both failing draw logs and run records, the matrix script, the reader auditor and its calibration results, and the control template), `originals_and_copies_exact=true`; **130 of 320 draws executed** (128 matched + 2 rejected), 190 NOT_RUN; `candidate_repair=false`, `expectation_repair=false`, `production_changes=0`; the sources were last written 10:05:37, the failing draws 10:07:14 and the seal 10:07:15. My own build of the sealed source (`9e1c26c7...`, 250,214 B; binary `79d4ce2a...`, 141,462,328 B) reproduces the stop at eight and one with the identical observation line; ordinary and actual pass; `missing` and `mixed_type` also pass (32/32 configurations each, because the proposed key is absent, nothing is removed and the claim is `Empty`), so the defect is the whole **foreign-live-key** family: 24 of the 32 other_live configurations fail (witness both arms and runtime armed, all four forms and both modes), and the 8 runtime-disabled passes are vacuous (`genuine_source=false previous_occupied=false` — no records existed to consume). The reader auditor is calibrated (2 positives accepted, 16 falsifiers rejected, 40 preserved tests); my own build reproduces all 18, and four falsifiers I generated myself (foreign-as-actual, omitted-reader-check, remove-survivor-check, wrong-physical-type) are all rejected at exit 101. The comparator is already unwinding-agnostic, so no comparator re-derivation is required for the class.

The ruling `Q69-deferred-constructor-physical-cursor-authentication`: at the deferred constructor the consumer must authenticate the transported proposal against the actual boxed cursor's physical identity — its address and concrete type — and its genuine pending source **before it removes either deferred record or issues any construction authority**. A missing key (even with the actual concrete type and a genuine pending source), a foreign key naming another live cursor, or a mixed-type key supplies no authority. The compliant outcomes are a well-bound physical consumption that removes both records for the actual Box and issues the construction and evaluation provenance for that Box, or an **assertion-free pre-effect refusal** — control flow, not an unwind (Q67/Q68) — that removes nothing, issues no construction authority, leaves every prior record set unchanged, leaves the actual cursor native and leaves the other live Box with both prior records, its native behavior and its single armed hit. The refusal may not be implemented with `assert`/`assert_eq`/`assert_ne` or any other panicking termination; the frozen 40-test base and the four added controls stay as declared; the comparator stays unwinding-agnostic; the registered edit set and the reader auditor are re-derived to the assertion-free text before execution, with the calibration still rejecting under-authenticated shapes. This is the Q63/Q64/Q65/Q66/Q67/Q68 receiver-authentication class one layer deeper — the deferred storage **removal** handoff, where Q68 was the insertion side — a genuine design-level defect of the scratch consumer model, not instrument-validity, not a production regression and not a reclassification. The successor may not adopt the observed run post hoc, edit the frozen tests or their assertions, weaken the bar, or delete or reclassify the stopped draw; the sealed 27 files, the declaration, both sources, the fixture binary, the two draws and their receipts stay sealed. The reader group earns nothing — 64 of 160 configurations matched, 130 of 320 draws executed — the four `construct_deferred` obligations are outside the union, and the four `DeferredEvaluationGuard::drop` rows in the union stay unpaid until their four-form gate completes.

Continuation 70 must re-render `construct_deferred` assertion-free under Q69 (derive the physical identity from the actual cursor and require the transported proposal to equal it before either record is removed; foreign, missing and mixed proposals issue no construction authority and return in control flow), re-derive the registered edits and the reader auditor before execution with the calibration still rejecting under-authenticated shapes, re-freeze the plan and comparator and run the full 320-draw reader schedule at eight and one on one binary per configuration (8 builds), including the 24 failing other_live configurations and the 8 vacuous ones, preserving the sealed 27 files and the stopped declaration; then the four `DeferredEvaluationGuard::drop` handoffs on all four forms (the four in-union obligations), then the 21 remaining retirement forms (P0256, P0259, P0292, P0314, P0315, P0394, P0396, P0410, P0413, P0494, P0498, P0554, P0814, P1139, P1142, P1147, P1155, P1477, P1814, P1817, P1937, all form-1), and from 557/4,422, 36/134, 4/133 and 0/26 through 134/4,422/133, the 151 plans, 94 paths, 188 postimages and the 151-cell schema, then the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder. Everything prior stays exact by recomputed hash identity wherever every input is byte-identical (the 2026-11-14 ruling); anything whose input hash differs is re-run.

The accepted bar rises to **557/4,422** qualified obligations, **36/134** actors, **4/133** fields and **0/26** complete groups: the four `core::Guard::defer` rows are now paid (`full_four_form_gate=true`), the four `core::construct_deferred` rows are outside the union and the four `DeferredEvaluationGuard::drop` rows in the union remain unpaid. The complete bar stays unexecuted: plans 0/151, emitted paths 0/94, postimages 0/188, the schema unchanged (`f9604852...`, 90 preimages, 180 postimages, 151 immutable rows), 3,865 obligations outside the qualified union, the standing replay NOT_RUN and the workspace NOT_RUN (0/6,000).

Allocation: continuation 69 charged **1,783.328948774986240739 s** (1,254.328948774986240739 s measured over 492 receipts — 490 carrying their own `CM742C69 COMMAND` marker and exit, plus the 2 driver records with 328 and 138 verified child links whose child sums are 83.43869892938528 s and 72.17274952188018 s and whose `elapsed + child == total_serial_elapsed` — + 269.0 s bookkeeping at 0.5 s x 492 receipts plus 0.5 s x 46 nested auditor invocations inside their parents + 200.0 s conservative inspections/setup + the fixed 60 s reserve) of 14,780.6046172971073 s, leaving **12,997.275668522121059261 s**; CM4 stood at 121,719.39538270289 of 136,500 s and now stands at **123,502.724331477876240739 s**. **No new allocation** — continuation 70 runs under the remaining 12,997.275668522121059261 s and the assigned 6,000 s workspace (0/6,000); the 1,440 s verification, the second 6,000 s workspace, the 57,000 s stage reserve and the 40,500 s for 743-746 stay held. The stopped scene is **1,195 files / 2,637,783,810 bytes** with `originals_and_copies_exact=true`, `candidate_failure=true` and `candidate_repair=false`; I rehashed all 1,195 pairs and the 27-file sealed stopped draw with zero bad, the c68 scene rehashes clean (12,620 pairs) with its 18-file sealed draw exact, the c65/c66/c67 sealed draws (14, 14, 15 entries) are exact, and the 405 director-replay transients are identical to c68's list (27 live files hash exactly, 378 are absent transients, 116 link to the c49 freeze with exact frozen hashes). Entry authenticates 5,400 maintained / 590 design / 790,890 accepted files with `inherited_document_drifts=3` — exactly my three committed c68 documents (entry = the `787e3ee9:` blobs = the files on disk; prior = the `ea31d6df:` blobs) — and `production_changes=0`, pinning 90 preimages, 180 postimages, 151 immutable rows and the unchanged schema; the working tree carries only the 23 inherited porcelain entries, and the entry report is byte-identical to the saved c68 report. The change scope is 2,410 files, all under `.temp/brief-742` (2,408 added + 2 modified), zero deleted, zero outside. I verified the plain `check` (415 citations), all 492 receipts (exact decimal sum, charge, remainder and CM4), all 320 writer draws on my own builds with all 256 observation lines identical, the 266-assertion literal audit on both roles at eight and one, my own writer and reader auditor builds against their 28 and 18 calibration controls and eight falsifiers I generated myself, the reader stop and the whole other_live class, the freeze pins and manifests, the director and entry drifts through git, the predecessor identity (15 groups, 25,535 verified entries, 419-file sample, `changed` = the two c68 writer renders), the accounting records, the declaration ordering and the close-time-only freshness. Continuation 69 ran `cdx` (unit `brief-742-cdx-20261119-094951-1982531`, rc 0, peak 4,244,426,752 B, oom 0 / oom_kill 0; log 1,528,641 B, sha256 `bcf37299...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 2,410 changes — 2,408 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted — 30 PASS / 2 FAIL / 2 NA, 415 citations, 0 questions, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 70.

## Continuation 70 ruling (director) — the Q69 constructor repair is delivered at 320/320 draws, the deferred constructor obligations and the actor `core::construct_deferred` are earned (561/4,422, 37/134), and then the new deferred-evaluation-drop group stops under Q28 on the mismatched transported provenance because `DeferredEvaluationGuard::drop` pops both stacks and retires before comparing the owners. The ruling is `Q70-deferred-evaluation-drop-provenance-authentication`; the drop claim earns nothing. Full record in `.temp/director/ruling-c70.md`.

The Q69 repair is delivered on both roles. Against the sealed c69 stopped render (`9e1c26c7...`, 250,214 B / `8a6ade81...`, 250,245 B) the diff is +218 bytes per role (`e81cf92c...`, 250,432 B / `085c2359...`, 250,463 B) and is exactly three registered edits: `construct_deferred` derives `physical = (cursor as *const T as usize, type_name::<T>())`, transports it as the proposal, reads `candidate` and `proof` at the physical key, computes `present = key != physical || candidate.is_some() || proof.is_some()`, and only when `key == physical` and `candidate.is_some_and(|token| proof == Some(token) && pending_construction(&l, token))` removes the proof and the record — both removals after the complete authentication; the match arms become `Some(token) => Deferred`, `None if present => InvalidPresent`, `None => Absent`. No assertion was added: the literal examiner reads **291 assertion literals byte_exact=true** on both roles at eight and one (the c69 stopped render versus the c70 render; I re-ran the sealed examiner on my copies), the `deferred_reader_control` module is token-identical to the frozen c69 one, and the 40 original tests plus the four added controls are preserved.

The deferred-cursor-reader group passes **320/320 draws** over 160 configurations (P2132-P2135, all form-1, x 2 roles x 2 arms x 2 modes x 5 classes) at eight and one on one binary per role and mode (8 builds), all at `[1,0,0,43]`. All 192 foreign, missing and mixed draws decline authority (`claim=Empty`, `foreign_changed=false`, `survivor=true`, inner native, `inner_hits=0`, `repeated_resume_hits=0`); all 256 instrumented draws keep their survivor; `other_live` has 48 genuine-record draws and 16 inactive ones; `actual` gives 32 authorized draws (`inner_forwarded=true`, `inner_hits=1`), 16 recording-not-armed draws (native) and 16 non-recording draws (`claim=Empty`). My own 8 builds of the two renders reproduce **all 320 draws**, and all 320 sealed draw logs (every `CM742C69 CURSOR READER`, armed, hit and hit-id line) are byte-identical to mine. The reader auditor was re-derived (`e61bab4a...`; 38 controls — 2 positives accepted, 18 falsifiers rejected, per role); my own build reproduces all 38, and four falsifiers I generated myself (physical-inequality-dropped, proof-omitted, pending-omitted, assert-refusal) are all rejected at exit 101. The four `core::construct_deferred` obligations (3705, 3728, 3751, 3774) and the actor `core::construct_deferred` are **earned**: the accepted bar rises from 557 to **561/4,422** qualified obligations, from 36 to **37/134** actors, **4/133** fields and **0/26** complete groups; the four rows move `in_current_union` false -> true with `full_four_form_gate=true`, exactly the shape in which `Guard::defer` was earned at c69.

The stop is honest and reproduced by me. The new deferred-evaluation-drop group (P2132-P2135, all form-1, x 2 roles x 2 arms x 2 modes x 7 classes — ordinary, actual_inner, ordinary_outer, unknown_slot, cross_live_slot, retired_identity, other_provenance — = 224 configurations / **448 draws**; 44 frozen original tests plus 6 added controls per source; actor `core::DeferredEvaluationGuard::drop`; obligations 3704/3727/3750/3773, in the incoming union with the four-form gate incomplete; source boundaries callback, constructor, registered frame insert and natural destruction) was declared before execution (plan, freeze and declaration 10:53:55-10:53:56, auditor build 10:53:57, calibration 10:54:07, first build 10:54:16) and matched 192 configurations (six classes x 32; 384 draws) before stopping on **P2132/form-1, witness, armed, hash, class other_provenance** (item 193): declared `[1,0,0,49]`, observed `[0,1,0,49]` at eight and one on the same binary `54a581d4...` (141,583,856 B). The observation is `records_present=true seam_exact=true conflicting=true refused=false unchanged=false source_pop=true transport_pop=true own_pop=false inner_current=false outer_current=true inner_exact=true outer_exact=true inner_hits=1 outer_hits=1`, and the failing assertion is `deferred teardown must preserve both record sets when its transported slot or owner disagrees` (witness.rs:835:13). Two live evaluation guards came from distinct genuine callback pauses; the test supplied the outer producer's complete deferred token at the inner transported stack top; the independent source stack, both scope registrations, frame records, pending records, slot and actual inner object were intact at entry. `deferred_scope_binding` reads only the independently recorded top and the actual object passes its check; `Drop` then pops the independent stack, pops the transported stack testing only the enum variant (`Deferred(_) | InvalidPresent`), and retires the inner identity — so the inner object was retired even though the transported token named the other producer. Both native results matched and both resumes recorded their single hit. The group is sealed immediately: **27 files** (stop record, plan, freeze, matrix log, results, edits, driver record, preflight, both renders and their diffs, the fixture binary and its build records, both failing draw logs and run records, the matrix script, the drop auditor and its calibration results, and the control template), `originals_and_copies_exact=true`; **386 of 448 draws executed** (384 matched + 2 rejected), 62 NOT_RUN; `candidate_repair=false`, `expectation_repair=false`, `production_changes=0`; the sources were last written 10:53:55, the failing draws 10:56:24.96 and 10:56:25.19 and the seal 10:56:25.79. My own build of the sealed render (`97809d168...`, 262,877 B; binary `1468ca93...`, 141,583,856 B) reproduces the stop at eight and one with the identical observation lines; all 386 worker-executed draws are identical to mine; and I ran the 62 NOT_RUN draws. The class is the whole **recording other_provenance family**: 24 of the 32 configurations fail at both threads (witness armed and disabled, runtime armed; all four forms and both modes) with `records_present=true conflicting=true unchanged=false source_pop=true transport_pop=true inner_current=false outer_current=true`, and the 8 runtime-disabled passes are vacuous (`records_present=false conflicting=false` — no records exist to consume). The drop auditor (`7631d78c...`; 28 controls — 2 positives accepted, 12 falsifiers rejected, per role) reproduces on my build; three falsifiers I generated myself (shared source and transport, relaxed expectation, foreign top not installed) are rejected at exit 101. My fourth (`conflicting=false`) is accepted by the auditor because the flag is cross-checked inside `seam_exact`, which the compiled test asserts: I built that mutant and its `other_provenance` draw exits 101 at the `assert!(o.seam_exact)` line, so it cannot produce a false green.

The ruling `Q70-deferred-evaluation-drop-provenance-authentication`: at the deferred evaluation guard's teardown, **before** popping either the independently recorded source provenance or the transported evaluation provenance, and **before** retiring the guard's scope object, the consumer must authenticate that the complete transported top equals the guard's own independently registered source top — the value its binding was minted from, owner and token, not the enum variant alone. A transported provenance naming another genuine producer, or any transported top that disagrees with the independently recorded top, supplies no authority. The compliant outcomes are a well-bound retirement that pops one matching source top and one matching transported top and retires only the actual object, or an **assertion-free pre-effect refusal** — control flow, not an unwind (Q67/Q68/Q69) — that pops nothing, issues no authority, leaves both record sets unchanged, leaves the actual guard's object and the other live guard's object current, and leaves both native results and their single armed hits exact. The refusal may not be implemented with `assert`/`assert_eq`/`assert_ne` or any other panicking termination; the frozen 44-test base and the six added controls stay as declared; the comparator stays unwinding-agnostic (`refused` is a diagnostic, never required); the registered edit set and the drop auditor are re-derived to the assertion-free text before execution, with the calibration still rejecting under-authenticated shapes (variant-only comparison, comparison after a pop, refusal by panic, refusal that still retires, missing independent check, wrong object). This is the Q63/Q64/Q65/Q66/Q67/Q68/Q69 receiver-authentication class one layer deeper — the deferred evaluation guard's **retirement** handoff, where Q68 was the insertion side and Q69 the removal side — a genuine design-level defect of the scratch consumer model, not instrument-validity, not a production regression and not a reclassification. The successor may not adopt the observed run post hoc, edit the frozen tests or their assertions, weaken the bar, or delete or reclassify the stopped draw; the sealed 27 files, the declaration, both sources, the fixture binary, the two draws and their receipts stay sealed. The drop group earns nothing — 384 of 448 draws matched, 386 executed, 62 NOT_RUN — and the four `DeferredEvaluationGuard::drop` rows stay with `full_four_form_gate=false`.

Continuation 71 must re-render `DeferredEvaluationGuard::drop` assertion-free under Q70 (compare the complete transported top with the independently registered top before either pop or the retirement; a mismatch returns in control flow leaving both stacks and both identities untouched; a match pops both and retires only the actual object), register the edit set against the sealed c70 `deferred-drop` renders and the admissible c70 `deferred-reader` renders, re-derive the drop auditor before execution with the calibration still rejecting under-authenticated shapes, re-freeze the plan and comparator and run the full 448-draw schedule at eight and one on one binary per configuration (8 builds), including the 24 failing other_provenance configurations and the 8 vacuous ones, preserving the sealed 27 files and the stopped declaration; then the 21 remaining retirement forms (P0256, P0259, P0292, P0314, P0315, P0394, P0396, P0410, P0413, P0494, P0498, P0554, P0814, P1139, P1142, P1147, P1155, P1477, P1814, P1817, P1937, all form-1), and from 561/4,422, 37/134, 4/133 and 0/26 through 134/4,422/133, the 151 plans, 94 paths, 188 postimages and the 151-cell schema, then the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder. Everything prior stays exact by recomputed hash identity wherever every input is byte-identical (the 2026-11-14 ruling); anything whose input hash differs is re-run.

The accepted bar rises to **561/4,422** qualified obligations, **37/134** actors, **4/133** fields and **0/26** complete groups: the four `core::Guard::defer` rows and the four `core::construct_deferred` rows are paid (`full_four_form_gate=true`) and the four `DeferredEvaluationGuard::drop` rows stay with their four-form gate incomplete; 3,861 obligations, 97 actors and 129 fields remain outside their qualifications. The complete bar stays unexecuted: plans 0/151, emitted paths 0/94, postimages 0/188, the schema unchanged (`f9604852...`, 90 preimages, 180 postimages, 151 immutable rows), the standing replay NOT_RUN and the workspace NOT_RUN (0/6,000).

Allocation: continuation 70 charged **1,913.142792478029153992 s** (1,246.642792478029153992 s measured over 747 receipts — 745 individual runs plus the 2 driver records with 328 and 394 verified child links whose child sums are 84.118872821301921894 s and 89.089884415763665338 s and whose `elapsed + child == total_serial_elapsed` — + 406.5 s bookkeeping at 0.5 s x 747 receipts plus 0.5 s x 66 nested auditor invocations inside their parents + 200.0 s conservative inspections/setup + the fixed 60 s reserve) of 12,997.275668522121059261 s, leaving **11,084.132876044091905269 s**; CM4 stood at 123,502.724331477876240739 of 136,500 s and now stands at **125,415.867123955905394731 s**. **No new allocation** — continuation 71 runs under the remaining 11,084.132876044091905269 s and the assigned 6,000 s workspace (0/6,000); the 1,440 s verification, the second 6,000 s workspace, the 57,000 s stage reserve and the 40,500 s for 743-746 stay held. The current scene is **1,742 files / 2,637,500,700 bytes** with `originals_and_copies_exact=true`, `candidate_failure=true` and `candidate_repair=false`; I rehashed all 1,742 pairs and the 27-file sealed stopped draw with zero bad, the c69 scene rehashes clean (1,195 pairs) with its 27-file sealed draw exact and the two stopped reader sources unchanged, the c68/c67/c66/c65 sealed draws (18, 15, 14, 14 entries) are exact, and the 405 director-replay transients are byte-identical to c69's list (27 live files hash exactly, 378 are absent transients, 116 link to the c49 freeze with exact frozen hashes). Entry authenticates 5,400 maintained / 590 design / 793,297 accepted files with `inherited_document_drifts=3` — exactly my three committed c69 documents (entry = the `5997383b:` blobs = the files on disk; prior = the `787e3ee9:` blobs) — and `production_changes=0`, pinning 90 preimages, 180 postimages, 151 immutable rows and the unchanged schema; I rehashed all 5,990 maintained+design files and a 601-file accepted sample with zero bad. The change scope is 3,504 files, all under `.temp/brief-742` (3,502 added + 2 modified), zero deleted, zero outside; the working tree carries only the 23 inherited porcelain entries. The predecessor identity authenticates 16 groups and 26,135 linked inputs — I rehashed all 26,135 with zero bad — with `changed` = the two c69 sealed reader sources, unchanged on disk and the Q69 edit's declared before. I verified the plain `check` (538 citations), all 747 receipts (exact decimal sum, charge, remainder and CM4), all 320 reader draws on my own 8 builds with every sealed instrument line identical, all 386 worker-executed drop draws identical plus the 62 NOT_RUN draws run, the 291-assertion literal audit on both roles at eight and one with the sealed examiner, my own reader and drop auditor builds reproducing all 66 calibration controls and seven of eight self-generated falsifiers rejected (the eighth caught by the compiled test, demonstrated), the freeze pins and both declarations, the director and entry drifts through git, the predecessor identity, the accounting records, the declaration ordering and the close-time-only freshness. Continuation 70 ran `cdx` (unit `brief-742-cdx-20261119-103730-2002884`, rc 0, peak 4,131,774,464 B, oom 0 / oom_kill 0; log 1,688,030 B, sha256 `dddcc945...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 3,504 changes — 3,502 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted — 32 PASS / 2 FAIL / 2 NA, 538 citations, 0 questions, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 71.

## Continuation 71 ruling (director) — the Q70 refusal is delivered assertion-free and the drop group stops under Q28 on an unsatisfiable resume duty; Q71 rules fail-closed forwarding and the drop claim earns nothing. Full record in `.temp/director/ruling-c71.md`.

The Q70 refusal is delivered on both roles. Against the sealed c70 stopped render (`97809d16...`, 262,877 B / `843ced69...`, 262,908 B) the diff is +373 bytes per role (`ca5d528b...`, 263,250 B / `3cf7fa93...`, 263,281 B) and is exactly one registered edit per role: `impl Drop for DeferredEvaluationGuard` reads `source = l.scope_authority.provenance.last()` and returns before every effect unless the actual object matches its minted binding, `l.evaluation_provenance.len() == self.0`, the source top is a `Deferred(_)`/`InvalidPresent` variant, and `l.evaluation_provenance.last() == source` — the complete transported top equals the independently registered source top; on a match it pops the source top, asserts the transported variant, pops it and retires the actual object. No assertion was added: the sealed c54 examiner reads **320 assertion literals byte_exact=true** on both roles at eight and one (the c70 render versus the c71 render; I re-ran the sealed examiner myself), and the whole `deferred_drop_control` module — the frozen comparator and the six added controls — is byte-identical to the c70 one. The drop auditor `audit_drop.rs` was re-derived (`725011f5...`) and forces this exact drop body as a token stream, then asserts the inverse to the un-instrumented predecessor; its 54-entry calibration is 2 positives accepted and 52 falsifiers rejected at exit 101 (`preserved_tests=44`, `frozen_added_controls=6`, `candidate_execution=false`), and my own auditor build reproduces all 54 controls.

The group is the c70 one re-declared: 224 configurations / 448 draws (P2132-P2135 form-1 x 2 roles x 2 arms x 2 modes x 7 classes), 44 frozen original tests plus 6 added controls per source, actor `core::DeferredEvaluationGuard::drop`, obligations 3704/3727/3750/3773. Plan, freeze and declaration are written 11:31:59.34-11:31:59.44, auditor build 11:32:03, calibration 11:32:28, first build 11:32:37 and first draw 11:32:38, so the declaration precedes every executed schedule. Six classes match (384 draws). The matrix stops on **P2132/form-1, witness, armed, hash, class other_provenance** (item 193): the declared counts `[1,0,0,49]` are observed `[1,0,0,49]` at eight and one on binary `e463a774...` (141,588,024 B); `observation_errors=[]` — the frozen c70 comparator is exact — but the separate Q70 resume duty records six differences per draw (`inner_exact true->false`, `outer_exact true->false`, `inner_hits 1->0`, `outer_hits 1->0`, the hit identities `[(1,0,0),(2,1,0),(2,0,0)] -> [(1,0,0)]`, the source hits `3 -> 1`). The observation reads `records_present=true seam_exact=true conflicting=true refused=false unchanged=true source_pop=false transport_pop=false own_pop=false inner_current=true outer_current=true inner_exact=false outer_exact=false inner_hits=0 outer_hits=0`, with three `CM740_SELECTOR_ARMED` lines and exactly one `CM740_SOURCE_HIT` and one `CM742_HIT_ID` — the priming session hit. The group is sealed immediately: **34 files**, `original_and_copy_bytes_exact=true`; **386 of 448 draws executed** (384 matched + 2 rejected), 62 NOT_RUN; `candidate_repair=false`, `expectation_repair=false`, `production_changes=0`; sources last written 11:31:59, failing draws 11:34:46.4, seal 11:34:47.4.

The stop is genuine against the declared bar and the bar is unsatisfiable. My own 8 builds from the sealed sources reproduce **all 386 executed draws** with every `CM740_`/`CM742_` instrument line byte-identical to the sealed logs; I ran the 62 NOT_RUN draws myself; and I re-derived both verdict layers from the sealed logs: the frozen c70 comparator (`drop_verdicts.py`, byte-identical `16b21127...`) has zero observation errors on all 386 executed draws, and under the per-arm Q71 duty **all 448 draws** pass — the 2 stopped and 30 remaining armed-recording draws decline, 16 witness-disabled recording draws decline with their native expectation, and 16 runtime-disabled inactive draws take the ordinary actual path. The mechanism is the refusal's required consequence: the corruption makes the transported vector disagree with the independent source vector at one entry; `lexical_current` requires their equality, `resume_frame` filters its saved frame token by it, and `frame()`/`eligible()` and the omission gate require it; the refusal pops nothing, so no registered frame authenticates and both genuine resumes take `resume_realization` — the native path — with no forwarding and no armed hit. In c70 the exact resumes were produced by the defect: the unauthorized pop restored the pair equality and thereby authorized the resumes while retiring the inner object under a foreign token. So the Q70 clause "both resumed results and their single armed hits exact" is reachable only by the effect Q70 forbids, and the frozen comparator itself skips the value and hit assertions on a conflict for exactly that reason; the clause was mine, derived from the defective run's incidental by-product.

Ruling **`Q71-deferred-evaluation-drop-fail-closed-forwarding`**: Q70's authentication stands unchanged and exactly as rendered — the complete transported top must equal the guard's own independently registered source top (owner and token, not the enum variant) and the actual object must match its minted binding and depth, before either pop and before the retirement; a mismatch returns in control flow, pops nothing, retires nothing and issues no authority (no `assert`, no panic). Q70's clause that a refusal leaves both native results and their single armed hits exact is **withdrawn for the conflicting class**: the compliant outcome is fail-closed forwarding — the mismatched transported top stays in place, the pair invariant stays false, and every forwarding that depends on it (the registered-frame authorization and the omission gate) declines until the mismatch is removed; the affected run completes on the native realization path and nothing derived from the forged vector forwards. Per-slot relaxation is refused: forwarding an unaffected slot under a false pair invariant would issue authority inside the Q63-Q70 receiver-authentication class, and the negative scene exists to prove that one forged transported entry supplies no authority to anything in that run. The compliant contract per arm: armed recording — `conflicting=true`, `refused=false`, `unchanged=true`, no pops, both objects current, `inner_exact=false` and `outer_exact=false`, zero hits, three armed lines, exactly one prime source hit and hit identity; disabled-but-recording (witness disabled) — the same declined shape with `inner_exact=true` and `outer_exact=true` because the expectation is the native value, zero hit lines, three armed lines; inactive (runtime disabled) — not a conflict, the ordinary actual path with exact native results, zero hits and zero armed lines. The frozen 44 + 6 stay as declared, `drop_verdicts.py` stays byte-exact, the c71 sources stay the candidate text, the c70 sealed 27 and c71 sealed 34 files stay sealed, the stopped draws are not deleted or reclassified, and no continuation may adopt those runs post hoc or weaken the bar. The drop group earns nothing — 384 of 448 draws matched, 386 executed, 62 NOT_RUN — and the four `DeferredEvaluationGuard::drop` rows keep `full_four_form_gate=false`.

Continuation 72 must re-declare the deferred-evaluation-drop group under Q71 with the same two candidate sources (`ca5d528b...` / `3cf7fa93...`; the refusal text needs no edit), keep the frozen 44 + 6 and `drop_verdicts.py` byte-exact, re-derive the resume duty as the per-arm contract above, register and calibrate it before execution with falsifiers for every deviation (a forwarded inner, a forwarded outer, an armed hit, a source or transport pop, a retirement, an unwind, a lost prime hit, a lost or extra armed line, the inactive path treated as a conflict), re-freeze the plan and comparator and run the full 448-draw schedule at eight and one on 8 builds including the 62 unexecuted draws, preserving both sealed scenes; if every draw passes, pay the four `DeferredEvaluationGuard::drop` obligations (3704, 3727, 3750, 3773) and the actor `core::DeferredEvaluationGuard::drop` (bar **565/4,422**, **38/134**, **4/133**, **0/26**); then the 21 remaining retirement forms (P0256, P0259, P0292, P0314, P0315, P0394, P0396, P0410, P0413, P0494, P0498, P0554, P0814, P1139, P1142, P1147, P1155, P1477, P1814, P1817, P1937, all form-1), and from the bar through 134/4,422/133, the 151 plans, 94 paths, 188 postimages and the 151-cell schema, then the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder; everything prior stays exact by recomputed hash identity wherever every input is byte-identical (the 2026-11-14 ruling) and anything whose input hash differs is re-run.

The bar stands at **561/4,422** qualified obligations, **37/134** actors, **4/133** fields and **0/26** complete groups; 3,861 obligations, 97 actors and 129 fields remain outside; plans 0/151, paths 0/94, postimages 0/188, the schema unchanged (`f9604852...`, 90 preimages, 180 postimages, 151 immutable rows), the standing replay NOT_RUN and the workspace NOT_RUN (0/6,000). Allocation: continuation 71 charged **1,579.039068450074398884 s** (1,085.039068450074398884 s measured over 414 receipts — 410 individual runs plus 1 matrix driver with 394 verified child links whose sum is 89.206668787897796994 s and whose `elapsed + child == total_serial_elapsed`, plus 3 close-time receipts — + 234.0 s bookkeeping at 0.5 s x 414 receipts plus 0.5 s x 54 nested auditor invocations + 200.0 s conservative inspections/setup + the fixed 60 s reserve) of 11,084.132876044091905269 s, leaving **9,505.093807594017506385 s**; CM4 stood at 125,415.867123955905394731 of 136,500 s and now stands at **126,994.906192405979793615 s**. **No new allocation** — continuation 72 runs under the remaining 9,505.093807594017506385 s and the assigned 6,000 s workspace (0/6,000); the 1,440 s verification, second 6,000 s workspace, 57,000 s stage reserve and 40,500 s for 743-746 stay held. The frozen scene is **1,037 files / 1,503,683,910 bytes** with all pairs exact (the 16 post-freeze accounting outputs are listed and accounted, and the freeze's finding agrees with the manifest); the sealed stopped draw is **34 files**, exact; I rehashed all 1,037 frozen pairs, the 34 stopped pairs, the c70 scene (1,742) and draw (27), the c69 scene (1,195) and draw (27) and the c68/67/66/65 draws (18, 15, 14, 14) with zero bad, the 405 director drifts are identical to c70's, and the predecessor identity authenticates 27,031 linked inputs — I rehashed all 27,031 and the 22 external entries with zero bad; `changed` = the two c70 sealed drop renders, the declared before-images. Entry authenticates 5,400 maintained / 590 design / 796,798 accepted files with `inherited_document_drifts=3` — my three committed c70 documents (entry = the `9dc01f65:` blobs = disk; prior = the `5997383b:` blobs) — and `production_changes=0`; the scope is 2,094 files, all under `.temp/brief-742` (2,092 added + 2 modified), zero deleted, zero outside, and the working tree carries only the 23 inherited porcelain entries. I verified the plain `check` (366 citations), all 414 receipts with the exact decimal sum, charge, remainder and CM4 step and the driver's child and serial identities, the 54 auditor controls on my own auditor build, the 10 resume controls and the six stop differences re-derived from the sealed observations, the 320-assertion literal audit, all 386 executed draws on my own 8 builds, the 62 NOT_RUN draws run, the per-arm Q71 duty over all 448 draws, the freeze pins, the declaration ordering and the close-time-only freshness. Continuation 71 ran `cdx` (unit `brief-742-cdx-20261119-111955-2020430`, rc 0, peak 3,083,104,256 B, oom 0 / oom_kill 0; log 1,668,455 B, sha256 `4d2c0504...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 2,094 changes — 2,092 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted — 27 PASS / 2 FAIL / 2 NA, 366 citations, 0 questions, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 72.

## Continuation 72 ruling (director) — all 448 draws pass on 8 builds, the four-form gates close, and the run stops on a qualify-payment port; Q72 rules the distinct-identity baseline. Full record in `.temp/director/ruling-c72.md`.

Continuation 72 re-declared the deferred-evaluation-drop group under Q71 with the two unchanged sources (witness `ca5d528b...`, 263,250 B; runtime `3cf7fa93...`, 263,281 B), the frozen 44 original tests plus 6 added controls, the byte-exact comparator `drop_verdicts.py` (`16b21127...`) and the new per-arm duty `resume_evidence.py` (`41485881...`), with the plan (`43f45eae...`, 295,134 B) and freeze (`ad7fe044...`, 27,018 B) written before execution and the duty calibrated before the first build (controls `e3d38341...`, 922 entries; receipt `92dcae07...`). All **448/448 fresh draws pass** over 224 configurations and 8 builds at eight and one — seven classes at 64/64, `matched=448`, `rejected=0`, `not_run=0`; the 386 continuation-71 draws reproduce their raw events exactly, the two formerly stopped draws now pass under Q71, the 62 formerly unexecuted draws ran for the first time, all four form gates are 112/112 (`3704`, `3727`, `3750`, `3773`) and `full_deferred_gates=12/12`. My replay rebuilt all 8 fixtures from the sealed sources, reproduced all 8 binary hashes and all 448 draws with every `CM740_`/`CM742_HIT_ID`/`CM742C70 DEFERRED DROP` line byte-identical, and the sealed comparator and duty report zero errors on every draw; the calibration rerun is byte-identical (34 positives, 888 falsifiers rejected, 3 profiles, 2 sealed examples used only as calibration). The inherited controls are exact: 320 assertion literals byte_exact on both roles at eight and one, 2 positives / 52 rejected over 54 AST controls, declared counts `[1,0,0,49]`, and the fail-closed profile passes 32 armed-recording, 16 disabled-recording and 16 inactive draws. The 21 retirement inputs are authenticated but unexecuted (`core::forget_frame`; 19 `Q19-PairedConstructedFrame`, 2 `Q18-Q19-TypedFrame`; `qualification_credit=0`, `new_executions=0`). The group is sealed immediately: **1,013 files / 1,320,914,483 B**, every original and copy exact; `candidate_failure=false`, `candidate_repair=false`, `production_changes=0`.

The run then stops on a qualification-payment port: the brief ordered **565/4,422, 38/134** from the four drop obligations and their actor, while the independent payment audit returns **561/4,422, 37/134** with `already_present=[3704, 3727, 3750, 3773]` and `actor_already_present=true` (`PortBlocked::DeferredDropQualificationAlreadyCounted`). The audit is right and the ordered arithmetic was mine. The four indices and `core::DeferredEvaluationGuard::drop` are members of the qualified union in every ledger from continuation 66 (551/34; c68 553/35, c69 557/36, c70 561/37, c71 561/37); their evidence is the continuation-65 actual-scope schedule — 16 items x 2 threads = 32 draws over four components (`core-component-valid/weak x hash/nohash`), each with exactly one of the four indices, the actor in its boundaries, the correct form, exact source, binary and log hashes and `expected_exact=true` at eight and one (I rehashed all 32). The duty rows carry `full_source_bound_execution=false`: the missing thing was the four-form source-bound proof, not a qualification. The audit's positive control is real (c69 557/36 -> c70 561/37 from the four `construct_deferred` additions through the same distinct join). The c71 count already held the four drop rows while the gate table listed them with `full_four_form_gate=false`; the qualified union and the gate table are different instruments and I conflated them. **Ruling `Q72-distinct-qualification-baseline`: the frozen distinct-identity ledger is the only qualification baseline; a payment is the distinct cardinality of its evidence set and a group's gate closure may not be added to it; an ordered count that disagrees with the ledger is a port, never a licence to inflate, and never a reason to reclassify prior evidence.** The four drop obligations and the actor are already qualified; the deferred-evaluation-drop claim is accepted at its gate bar (4 x 112/112, 448/448, 12/12 gates) and earns no distinct increment; the bar stands at **561/4,422, 37/134, 4/133, 0/26**. Continuation 71's ordered 565/38 is withdrawn. Everything sealed stays sealed — the c65 scope schedule, the c66-c71 ledgers, the c69 scene (1,195) and draw (27), the c70 scene (1,742) and draw (27), the c71 scene (1,037) and draw (34), the c65-c68 draws (14, 14, 15, 18), the c72 scene (1,013), both sources, plan, freeze, comparator and duty — and no continuation may re-litigate the c65 evidence, count the gate closure, re-run the group to earn what is already in the ledger, adopt a run post hoc, edit frozen tests or assertions, or weaken the bar.

Continuation 73 resumes the stage under the remaining allocation: the 21 retirement forms from `retirement-inputs.json`, each with its own executed source-bound evidence (P0256 and P0259 must distinguish the RHS producer frame from the matcher frame in the separate branch; the authentication credits nothing), then the generator — the 151 plans, 94 emitted paths, 188 postimages and the 151-cell schema emission — then the remaining indexed obligations through the complete bar 134/4,422/133, the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder; everything prior stays exact by recomputed hash identity wherever every input is byte-identical (the 2026-11-14 ruling) and anything whose input hash differs is re-run.

The bar stands at **561/4,422** qualified obligations, **37/134** actors, **4/133** fields and **0/26** complete groups; 3,861 obligations, 97 actors and 129 fields remain outside; plans 0/151, paths 0/94, postimages 0/188, the schema unchanged (`f9604852...`, 90 preimages, 180 postimages, 151 immutable rows, 94 admitted paths), the standing replay NOT_RUN and the workspace NOT_RUN (0/6,000). Allocation: continuation 72 charged **1,206.318310182919958498 s** (708.818310182919958498 s measured over 475 receipts — 471 individual, 1 matrix driver with 456 verified child links and its serial identity, 3 close-time receipts — + 237.5 s bookkeeping at 0.5 s x 475 + 200.0 s conservative inspections/setup + the fixed 60 s reserve) of 9,505.093807594017506385 s, leaving **8,298.775497411097547887 s**; CM4 stood at 126,994.906192405979793615 of 136,500 s and now stands at **128,201.224502588899752113 s**; **no new allocation** — continuation 73 runs under the remaining 8,298.775497411097547887 s and the assigned 6,000 s workspace (0/6,000); the 1,440 s verification, second 6,000 s workspace, 57,000 s stage reserve and 40,500 s for 743-746 stay held. The current scene is **1,013 files / 1,320,914,483 bytes** with every pair exact; the c71 scene 1,037 files with its 34-file draw, the c70 scene 1,742 and draw 27, the c69 scene 1,195 and draw 27, the c68/67/66/65 draws 18/15/14/14 all rehashed with zero bad; the 405 director drifts are byte-identical to c71's list and still resolve; the predecessor identity authenticates 27,613 linked inputs and 22 external entries (all rehashed by me, zero bad) with `changed` = the brief rebinding (`ad03c557...` -> `376dc848...`) and the replacement of the c71 duty (`5a25e57b...` -> `41485881...`). Entry authenticates 5,400 maintained / 590 design / 796,798 accepted files with `inherited_document_drifts=3` — my three committed c71 documents (entry = the `d2be1795:` blobs = disk; prior = the `9dc01f65:` blobs) — and `production_changes=0`; the accepted map grows to 798,891 with the 2,093 c71 files; the scope is 2,046 files, all under `.temp/brief-742`, and the working tree carries only the 23 inherited porcelain entries. I verified the plain `check` (401 citations), all 19 runs and 475 receipts with the exact decimal sum, charge, remainder and CM4 step and the driver's child and serial identities, the byte-identical calibration controls, all 448 draws on my own 8 builds with all 8 binary hashes, the 320-assertion literal audit, the 54-control AST audit, the 32-draw c65 scope evidence, the payment lineage c66-c71, the freeze pins and declaration ordering, the c72 scene pairs (1,013), the c71 pairs and 34-file draw, the c70 (1,742) and c69 (1,195) scenes and their draws, the c68/67/66/65 draws, the 27,613 identity entries and 22 external files, the 405 drifts, the entry maps and the three drifts through git, the schema pins, the closure ledger and the retirement inputs, the scope and the working tree. Continuation 72 ran `cdx` (unit `brief-742-cdx-20261119-115844-2035365`, rc 0, peak 3,093,614,592 B, oom 0 / oom_kill 0; log 1,954,820 B, sha256 `a6618bb1...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 2,046 changes — 2,044 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted — 30 PASS / 2 FAIL / 2 NA, 401 citations, 1 question, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 73.

## Continuation 73 ruling (director) — the retirement resolution's 30/30 source-witness draws are exact and the run stops on `PairedConstructedFrameRetirementOwner`: P0256/P0259 have no caller-owned post-result construction on their own edges; the two obligations stay Open and named, and the population is not re-pinned. Full record in `.temp/director/ruling-c73.md`.

Continuation 73 rendered a pre-render source-witness resolution of the two required retirement forms plus a positive constructor control and two separate-branch controls: five copied files under the admitted path set, five registered edits with exact inverse erasure, and the real `Session` exercised with two independently suspended sessions, the real input request, typed frame allocation, explicit abandonment, final drop, resume and queued input. **30/30 draws are exact** over 15 configurations x 2 threads (P0292/ground the positive control; P0256/matcher and P0256/ground; P0259/matcher and P0259/fast; each at abandon, drop and resume), on plan `b1c4df59...` with the declaration and calibration before the build and the first draw. The join is a calibrated instrument: its calibration accepts one post-result construction and rejects ten falsifiers, and on the executed logs it returns P0292 6 draws -> 12 live allocations and 12 actual `forget_frame` calls with receiver and type exact (`REAL POSITIVE PASS`), while **P0256 and P0259 each return 0/12 own matches**, their actual retired type being `EvaluateDefinitionApplicationRhsSuspensionFrame` against the declared `TryApplyDownValuesFromSymbolSuspensionFrame`; on the matcher controls the declared matcher type is allocated but `selected_caller_entries_on_matcher=0`. The stop is declared `PortBlocked::PairedConstructedFrameRetirementOwner` with `qualification_credit=0`, `complete_bar=false`, `no_descendant_attribution=true` and `no_transport_reclassification=true`; the scene seals immediately at **5,747 files / 1,308,057,854 B** with every original and copy exact, and the stopped draw at 98 files (manifest sha256 `f610c5bc...`, 49,855 B). The independent post-stop survey found eight forms sharing the resolved callee and flagged P0314/P0315 as Q19 rows that also return the observed RHS result; the first survey attempt stopped on P0320's `ResumeStop::failure` mapping and both attempts are retained. The generator is not delivered (0/151 plans, 0/94 paths, 0/188 postimages, schema `f9604852...` unchanged); the standing replay and the workspace are NOT_RUN.

The facts are read from the source and the executed logs. (1) Both required rows are keyed to the two occurrences of `self.evaluate_definition_application_rhs(option_scope, rhs)` inside `try_apply_down_values_from_symbol` — occurrence 0 at `down_values.rs:234` (ground, P0256) and occurrence 1 at `down_values.rs:339` (fast, P0259) — and both branches observe the producer result and immediately `return result.map_err(Route::failure);`: the caller never constructs after the return. (2) The producer's frame `EvaluateDefinitionApplicationRhsSuspensionFrame` is constructed by `suspend_definition_application_rhs_evaluation` inside the call, before the caller observes the returned `InputRequired` — a descendant construction, which Q21 rule 4 and the row's own transport text ("no descendant frame attributed to the caller") forbid attributing to the caller's activation; the join's post-result criterion is the faithful Q19 reading of that rule. (3) The declared constructor `suspend_down_value_application` is reachable only from the `match_compiled_owned` `InputRequired` arm of the same function — the separate branch where neither RHS caller site is entered; the matcher route never executes the row's statement and the RHS evaluation runs only after a resume through `finish_suspended_down_value_application`, which is P0266's edge. (4) The pinned armed statements for P0256 and P0259 are shape-identical to P0266's (a `Q747-PairedEvalReturn` row) — `enter`, `ordinary`, `hit(..., OrdinaryActualSuccessBeforeTail)`, the same `Ok`/`Err` match, the same producer, call expression, phase and observer — so the family was keyed from the consumer function's constructor set, not from the row's own call edge; P0314/P0315 carry the same immediate-return shape.

**Ruling `Q73-paired-construction-satisfiability`.** (1) The claim is refuted, not unproven: for P0256 and P0259 the `Q19-PairedConstructedFrame` transport — a caller-owned construction after the producer result, retained at the caller's own activation path and carried to the queued occurrence — does not exist on the row's own source branch, and the producer's frame may not be attributed to the caller; the two obligations (indices **363** and **392**) are unpayable as declared, stay Open and named, and take no credit. (2) No reclassification and no re-pin: the family determines the consumer boundary set — all 19 `Q19-PairedConstructedFrame` forms carry 29 `core::` boundaries and all 25 `Q747-PairedEvalReturn` forms carry 27 `paired::` boundaries (the 4,422 total is the sum over the 152 forms) — so a family change for these two forms would alter the population; a population re-basing may only be authorized explicitly, recorded as such, and never inferred by a worker or taken silently at a gate. (3) The retirement group proceeds: the other 19 rows are executed and paid only on their own source-bound evidence, and because the derivation rule that produced these two declarations is the defect, every one of the 21 rows must first be audited against its own edge shape — declared family, actual call edge, post-result construction, producer pre-return construction, returned-proof kind, mechanism — with zero unknown rows before any of them executes; a row like these two is named and held Open. (4) The generator proceeds on the 151 plans: the source edges of P0256/P0259 are resolved and unambiguous and the plan set does not depend on the qualification payment; the complete bar is accounted separately and reports the two named obligations as the exact shortfall. (5) The bar stands at **561/4,422, 37/134, 4/133, 0/26**; the complete bar cannot be declared while indices 363/392 are unpayable, and no campaign, epoch, acceptance or CM5 is authorized. (6) Everything sealed stays sealed — the c73 scene (5,747 files / 1,308,057,854 B), the stopped draw (98 files), the plan, the freeze and the five registered edits — and no continuation may edit the frozen forms, weaken the join, normalize the two rows, or adopt this run post hoc. I verified the plain `check` (213 citations) and the expanded check (2,283), all 30 draws on a copy of the sealed binary (`9662724...`, 151,164,064 B) with 2,028 instrument lines exact after per-run address normalization, the byte-identical `source-join.json` and calibration on my own copies of the judgement scripts, the five registered-edit inverses, the 27/2/2 checks, the 11,515-file scope, the 5,747-pair scene, the 98-pair stopped draw, the 15 freeze pins, entry (5,400 maintained / 590 design / 800,934 accepted; the three document drifts equal the `83f6873f:` blobs with priors at `d2be1795:`), the predecessor identity (28,105 linked inputs, 24 external, only the brief rebinding `376dc848...` -> `0d90a028...`) and the exact budget arithmetic (46 receipts; 633.41499745607142589 s measured; + 23.0 + 200 + 60 = 916.41499745607142589 s charged; remaining **7,382.360499955026121997 s**; CM4 **129,117.639500044971178003 s**).

Continuation 74 is ordered: (1) a derivation audit of all 21 retirement rows with zero unknown rows, including every row whose consumer function contains a sibling-edge constructor (at least P0314/P0315); (2) execution of the satisfiable rows with their own calibration, falsifiers and 8/1 draws, paying only after each join passes, with P0256 and P0259 named and unpaid; (3) the generator — the 151 plans, 94 paths, 188 postimages and the 151-cell schema emission — then the remaining indexed obligations toward 134/4,422/133, the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks, stopping before any group that cannot fit and reporting the exact remainder; everything prior exact by recomputed hash identity wherever every input is byte-identical (the 2026-11-14 ruling) and anything whose input hash differs is re-run. Budget: the remaining 7,382.360499955026121997 s plus the assigned 6,000 s workspace (0/6,000); the 1,440 s verification, second 6,000 s workspace, 57,000 s stage reserve and 40,500 s for 743-746 stay held; no new allocation. Continuation 73 ran `cdx` (unit `brief-742-cdx-20261119-123532-2049430`, rc 0, peak 2,841,399,296 B, oom 0 / oom_kill 0; log 1,942,239 B, sha256 `6cf27316...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 11,515 changes — 11,513 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted — 27 PASS / 2 FAIL / 2 NA, 213 citations, 1 question `retirement_owner`, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 74.


## Continuation 74 ruling (director) — the 21-row derivation is complete with zero unknown rows and the run stops on the queued-pair identity; Q74 corrects the attribution and orders independent pairs

Continuation 74 delivered the ordered derivation audit: **all 21 retirement rows resolved, zero unknown**, each with its declared family and transport, actual call edge, occurrence, statement, producer relationship, returned-proof kind and mechanism. Ten rows have a post-result typed route (P0292/641, P0394/1191, P0396/1228, P0410/1318, P0494/1799, P0498/1828, P0554/2021, P1477/2973, P1814/3585, P1817/3614), six return the producer result without a caller-owned construction (P0256/363, P0259/392, P0314/769, P0315/798, P0413/1347, P0814/2259, all `caller_post_result=false` with the declared constructor on the sibling branch), and five construct raw frames (P1139/2655, P1142/2684, P1147/2713, P1155/2742, P1937/3680, `frame_route=Entry::Legacy`). Credit is 0 for every row. The plan (108 cases over the ten executable rows, four builds, threads 8/1, 864 draws) was frozen and calibrated before the build and the first draw (204 controls, 57 positives, 147 falsifiers), the scene sealed at 10,785 files / 2,400,958,391 B with every pair exact, and the stopped draw at 60 files.

The run then stops on its own queued control: `PortBlocked::RetirementWitnessQueuedPairIdentity`, P0292, case 0, build recording-hash, threads 8 and 1 — `queued_physical=2`, `queued_survivor=0`, `expected_both_eligible=true`, after 2 of 864 draws. What passed on both draws is the product evidence: one independently issued record per scene (physical record 1/occurrence 0, survivor record 2/occurrence 1), typed allocation, alias drop, resume, drop, forget_enter and forget_exit exact in both scenes, both values exact and equal, no omission, both native libtests exit 0. The mechanism of the failure is the fixture's grouping, read from the probe and the logs: one queued `begin` consumes the one Q18 pair parked by the preceding unqueued begin, the pair's tokens are keyed by activation (episode plus the run's entry path), and `enter()` numbers entries through a single monotone map shared by every session in the window — the unqueued witness tokens sit at ordinals 1 and 3, and in the queued run the first session's calls occupy ordinals 0-4 and match both while the second session's occupy 5-9 and match none. The worker's own reconstruction records the same numbers (`eligible_global=1,3`, `corresponding_per_session=1,6`, `unexpected_reuse=3`, `missed=6`).

**Ruling `Q74-queued-ownership-attribution`.** (1) The run and the STOP stand; no qualification credit; all 21 retirement indices remain Open and unpaid, including P0256/P0259; the generator is not delivered; no campaign, epoch, acceptance, settlement or CM5. (2) The derivation audit is accepted — Q73 clause (3) is satisfied: zero unknown rows, the 10/6/5 classification, the Q73 rows confirmed on their own edges, the executable rows' post-result constructions confirmed against the live source; credit 0; no re-pin, no reclassification, no silent population change. (3) The failure is an attribution over-reach in the declared join, not a candidate failure: the Q19 transport's consume side is a matching queued activation at the original ordinary omission — an activation-level statement — while the probe's Q18 pair is one per program identity and carries no session; the clause "each queued execution sees its own eligible occurrence" is withdrawn as stated. (4) The successor fixture keeps the two duties separate: independent Q18 pairs for the queued-occurrence check — one unqueued begin and one queued begin per actual program execution, one execution per window, exactly one witness token per pair, the queued execution required to match its own token at its own activation (1/1), no fictitious queued frame, the queued value equal to the resumed value, no omission — and the simultaneous two-live-receiver retirement control as a separate window with the same foreign/missing/kind receiver falsifiers at the retirement boundary; both controls re-declared, frozen and calibrated before the build and the first draw. The alternative of reproducing the resumed entry order in the queued half is refused: under the shared monotone ordinal sequence it cannot be arranged, and it restates the failed clause. (5) Everything sealed stays sealed (the scene, the stopped draw, the plan, the freeze, the 204-control table, the derivation, both failed identity attempts, both independent-evidence attempts and every receipt); no continuation may adopt the run, delete attempts, relabel the case, edit the frozen fixture or join, or weaken the bar. (6) The bar stands at 561/4,422, 37/134, 4/133, 0/26; 3,861 obligations, 97 actors and 129 fields outside; generator 0/151 plans, 0/94 paths, 0/188 postimages; schema `f9604852...` unchanged.

Continuation 75 is ordered: reuse the derivation by hash identity (every input byte-identical); re-declare the two controls of clause (4); run the complete schedule at 8/1 on the four builds; on a pass pay the ten executable obligations (indices 641, 1191, 1228, 1318, 1799, 1828, 2021, 2973, 3585, 3614) to 571/4,422 with actors 37/134 and fields 4/133 unchanged, the eleven Open (363, 392, 769, 798, 1347, 2259, 2655, 2684, 2713, 2742, 3680) named and unpaid; then the generator (151 plans, 94 paths, 188 postimages, 151-cell schema emission); then the remaining indexed duties toward 134/4,422/133, the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks; stop before any group that cannot fit and report the exact remainder; hash identity reuse wherever every input is byte-identical (the 2026-11-14 ruling), anything with a changed input re-run. Allocation: continuation 74 charged **905.77682179401747919 s** (633.27682179401747919 s measured over 25 receipts plus 12.5 s bookkeeping at 0.5 s x 25 + 200 + 60) of 7,382.360499955026121997 s, leaving **6,476.583678161008642807 s**; CM4 now stands at **130,023.416321838988657193 s** of 136,500; no new allocation; the workspace stays 0/6,000 with the 1,440 s verification, second 6,000 s workspace, 57,000 s stage reserve and 40,500 s for 743-746 held. Continuation 74 ran `cdx` (unit `brief-742-cdx-20261119-132346-2063202`, rc 0, peak 2,890,977,280 B, oom 0 / oom_kill 0; log 3,308,936 B, sha256 `8791d040...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 21,668 changes — 21,666 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted — 30 PASS / 2 FAIL / 2 NA, 244 citations, 1 question `retirement_pair`, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 75.


## Continuation 75 ruling (director) — the Q74 successor fixture passes 72 draws and stops at P0494, whose single pause is three nested constructions; Q75 corrects the one-token clause and orders the per-activation cardinality control

Continuation 75 re-declared the Q74 successor fixture: independent Q18 pairs (two per draw, one execution per half, exactly the declared tokens), the separate simultaneous two-live-receiver retirement window with the foreign/missing/kind receiver falsifiers, the plan `2b0a3f24...` (108 cases over the ten rows, four builds, threads 8/1, 864 draws), the freeze `66aca184...` (20 pins) and the calibrated control `a8b299e1...` (282 controls: 61 positives, 221 falsifiers; the c74 204 retained byte-exact), all declared before the build and the first draw. The derivation was reused by hash identity (40 inputs identical, `dcdb1cc7...`, 21 rows, zero unknown, 10/6/5, credit 0). Cases 1-36 (P0292, P0394, P0396, P0410, nine cases each) passed at threads 8 and 1 on the recording-hash build: **72 exact draws**, each with its own receivers, both libtests green, no omission.

The run then stops on its own declared control: `PortBlocked::RetirementWitnessNestedPairCardinality`, P0494, index 1799, case 36 — `actual_pauses_per_pair=1`, `distinct_typed_witnesses_per_pair=3`, `actual_queued_matches_per_pair=3`, declared 1/1, after 74 of 864 draws (790 remaining). The selected program `CM739.batch4.P0494.7a01cd2d...` has one producer pause (`Input "AbortProtect.body:2"`, ordinal 0, answer "2") but three nested activations of the row's own edge `resume_call_with_dispatch_plan` -> `run_call_dispatch_stage` (`call.rs:239/323`); each invocation observes `InputRequired`, and each constructs its own `ResumeCallWithDispatchPlanSuspensionFrame` at its own `suspend_call_stage_with_boundary` (`call.rs:86/345`) while its activation is live: physical records 1/2/3 with owners `P0494@0/P0494@0/P0494@6`, `P0494@0/P0494@0`, `P0494@0`; survivor records 4/5/6. Each frame is retired by its own receiver (resume -> drop -> forget_enter -> forget_exit per record, live/bound/retired exact), the values are `Atom(Integer(Small(73)))` and equal, no omission executes, and the queued half constructs no frame and issues no request; the three candidate tokens match at visits 63/56/55, exactly the three owner paths, with full identity equal. The pair's token map is keyed by activation: `pause()` issues one ProducerPause token (occurrence 0) and each parent receives a ParentReturn token from its child with the same occurrence (`lib.rs:312-325`), so each activation holds exactly one frame token and the occurrence identifies the pause, not the frame.

**Ruling `Q75-nested-construction-cardinality`.** (1) The run and the STOP stand; no qualification credit; all 21 retirement indices remain Open and unpaid; the generator is not delivered; no campaign, epoch, acceptance, settlement or CM5. (2) The derivation reuse and the 72 exact draws are accepted; credit stays 0; Q73/Q74 dispositions unchanged. (3) The P0494 stop is a declaration over-reach in the control, not a candidate failure: the Q74 clause "exactly one witness token per pair" and the join clauses "one independently issued typed witness" and "distinct producer occurrence per issued source" are withdrawn as stated. (4) The source proof is corrected: requiring the constructing invocation to be the producer-request issuer holds only for the innermost activation; for a parent the evidence is its own dispatch call returning `InputRequired`, its own suspension branch constructing after that return, the frame owned by its activation with the activation scope live until the frame's own retirement, the chain anchored in exactly one actual ProducerPause per pair with the origin preserved, and no descendant attribution. (5) Continuation 76 shall predeclare the activation cardinality per case from the frozen program (P0494 = 3, the single-activation rows = 1), frozen before the build; per pair require a bijection between the n issued tokens and the n queued activations by full identity, one match per token at its own activation, one visit per match, no unmatched/extra/duplicate/cross-pair match, no non-pair match, no fictitious queued frame or request, equal values, no omission; per frame the clause-4 source proof with its ParentReturn chain and the exact lifecycle with its own receiver; the foreign/missing/kind receiver falsifiers, the two-live-receiver window and the disabled parity retained; calibrate the control (including duplicate token, missing middle frame, missing parent frame, child-attributed token, non-constructing-activation token, rewritten occurrence, cross-pair match, non-pair-activation match, extra visit, fabricated frame, and the single-activation 1/1 baseline) before execution. The per-construction rule is chosen over holding P0494 Open, which would record a correct candidate as unsatisfied. (6) Everything sealed stays sealed (scene 10,980 files / 2,431,682,417 B, stopped draw 61 files `e7ac5c91...`, plan, freeze, 282-control table, derivation, registered edits, all 89 receipts and the sealed binary `8a9a0672...`); no adoption, deletion, relabel, frozen-form edit or bar weakening. (7) The bar stands at 561/4,422, 37/134, 4/133, 0/26; generator 0/151 plans, 0/94 paths, 0/188 postimages; schema `f9604852...` unchanged; standing replay NOT_RUN; workspace 0/6,000.

Continuation 76 is ordered: derive the per-case activation cardinality (static, zero unknown), re-declare the nested control, freeze and calibrate before the build, run the **complete 864-draw schedule** at 8/1 on the four builds (the control and join change, so the hash-identity exemption does not apply to the draw evidence), and on a pass pay the ten executable obligations (indices 641, 1191, 1228, 1318, 1799, 1828, 2021, 2973, 3585, 3614) to 571/4,422 with actors 37/134 and fields 4/133 unchanged, the eleven Open (363, 392, 769, 798, 1347, 2259, 2655, 2684, 2713, 2742, 3680) named and unpaid; then the generator; then the remaining indexed duties, audits, ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks; stop before any group that cannot fit and report the exact remainder. Allocation: continuation 75 charged **939.90290051104968168 s** (630.99019389710157868 s child receipts + 4.412706613948103 s matrix-driver exclusive, plus 44.5 s bookkeeping at 0.5 s x 89 + 200 + 60) of 6,476.583678161008642807 s, leaving **5,536.680777649958961127 s**; CM4 now **130,963.319222350038338873 s** of 136,500; no new allocation; workspace 0/6,000 with the 1,440 s verification, second 6,000 s workspace, 57,000 s stage reserve and 40,500 s for 743-746 held. Continuation 75 ran `cdx` (unit `brief-742-cdx-20261119-140712-2076549`, rc 0, peak 3,308,417,024 B, oom 0 / oom_kill 0; log 1,524,241 B, sha256 `9ce7e1a8...`; worker model `gpt-6.2-astra` at effort `max`; report STOP, 22,076 changes — 22,074 added, 2 modified `PROGRESS.md`/`report.json`, 0 deleted — 28 PASS / 2 FAIL / 2 NA, 314 + 25,690 citations, 1 question `nested_pair_cardinality`, 0 deviations, 0 disclosures; no git or network, no `.git-attempts`). No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 76.

## Continuation 76 resume order (2026-11-19) — the attempt-1 dispatch died to the Codex safety filter after the complete schedule, the payment and the local-handoff stop were sealed; this resume authenticates that evidence by hash identity, re-derives the payment and the stop, and reports without repair or adoption

This order resumes continuation 76; the continuation-76 order above stands and this order restates it only where it must. **Attempt 1** (`cdx`, unit `brief-742-cdx-20261119-143908-2087131`) ran 14:39:08-15:27:58 and was killed at 15:27:58 by the Codex safety filter (`Turn error: This content was flagged for possible cybersecurity risk`, log line 32246) while composing its report: rc 1, peak 7,889,698,816 B, oom 0 / oom_kill 0, log 2,772,814 B sha256 `23053ff134b70755eac3f52f9e3d1c1e623de02a64358dc11bac8650b1069124`, archived at `.temp/logs/cdx_brief_742.log.prev-20261119-1535`; 953 receipts, 883.177818829950408605 s measured; no report, no close and no budget record; nothing written outside `.temp/brief-742`, and the working tree carries exactly the 23 inherited porcelain entries. Before it died it ran the complete 864-draw schedule at 8/1 on the four builds, re-derived and paid the ten executable obligations, and sealed the generator's local-handoff stop.

**The sealed evidence is read-only.** Every file attempt 1 left under `.temp/brief-742/continuation-76/` is sealed evidence. Compute a manifest (path, bytes, sha256) over that whole tree before any work and again at close; the two manifests must be identical. Write nothing into that tree. This resume works under `.temp/brief-742/continuation-76-resume/`, appends `.temp/brief-742/PROGRESS.md` and writes `.temp/brief-742/report.json`. Any write into the sealed tree, and any manifest difference, is a STOP.

**Hash-identity reuse.** The control, the join, the plan, the freeze, the builds and every draw input are byte-identical between the two attempts, so the 2026-11-14 ruling applies: the 864-draw schedule is **not** re-run and no build is remade. Authenticate instead — every receipt equals its recorded `{path, sha256, bytes}`; every log equals its receipt's `log.sha256`; the four results sets (`aaf18a52...` recording-hash, `4a578f25...` recording-nohash, `7f5a9f7a...` disabled-hash, `d399fcb8...` disabled-nohash) and their four binaries equal the recorded hashes; the plan `9b53ff55...`, the controls `0ea2bbfa...`, the freeze `4fbe4bfb...`, the calibrated freeze `cc301552...`, the derivation `dcdb1cc7...` and the stopped-draw manifest `88938b2f...` (25,291 B) equal theirs. Any mismatch is a STOP. The Q75 parenthetical ("the hash-identity exemption does not apply to the draw evidence") governs reuse from continuation 75, whose control and join changed; it does not forbid authenticating the fresh execution attempt 1 performed under the new control.

**Re-derive the payment.** Re-run the frozen schedule verification from the sealed logs into the resume tree and require: 864/864 exact draws over the four builds at threads 8 and 1, the ten indices 641, 1191, 1228, 1318, 1799, 1828, 2021, 2973, 3585, 3614 paid and disjoint from the ledger, exactly 571 qualified and 3,851 unpaid, actors 37/134, fields 4/133, groups 0/26, and the eleven Open (363, 392, 769, 798, 1347, 2259, 2655, 2684, 2713, 2742, 3680) named and unpaid. A divergent verdict is a STOP.

**Re-execute the stop.** The generator stopped at `PortBlocked::QueuedOmissionReceiptAmbientReselection`, index 625, P0292, case `ambient-occurrence`, after 4 of 16 planned draws: the `intact` pair matched and the `ambient-occurrence` pair exited 101 at the probe's own `hit without own activation token` (`lib.rs:405`), with the handoff line `decision=true genuine_occurrence=0 ambient_occurrence=1` over the two expected scenes. Re-execute exactly those four recording-hash draws — threads 8 and 1 on the binary `44d7b28d41e1974b18bca7f2267c06b94182d52e8b92430827bf9f17624a08ee` (268,004,528 B), the same fixture trees, the same `CM742_LOCAL_HANDOFF` settings, the same arguments and normalized keys — and require the same outcomes, the same two scenes' lines and the same panic. Run no other handoff draw: the stop ends the stage. Seal the four fresh logs, receipts and their hashes as this resume's stopped draw; the attempt-1 stopped draw stays sealed by its own manifest. A divergence is a STOP of a different kind: report it and stop; do not adjust the fixture, the binary, the probe, the judge or the case label.

**Re-derive the generator's stages.** From the sealed logs, re-derive with the frozen judgments the discovery, the construction bindings, the pair stage (48/48), the class stage (52 children) and the grant calibration at their recorded verdicts. The generator's delivered set stays 0/151 plans, 0/94 paths, 0/188 postimages and the schema `f9604852...` unchanged: the stop precedes the emission, and nothing of the generator earns credit.

**No repair, no adoption.** The stop is `candidate-contract` with `candidate_repair=false` and `expected_repair=false`: do not repair the candidate, the fixture, the probe or the plan, and do not relabel the case. The schedule, the payment and the stop are claimed only through this resume's own re-derivation from the sealed evidence; the report states plainly which artifacts attempt 1 produced and which this resume re-executed. Everything sealed stays sealed — the c65-c75 sealed draws, the attempt-1 schedule, payment and stopped draw, the plan, the freeze, the 282-control table, the derivation, the registered edits, the receipts and the binaries — and no continuation may adopt the run, delete attempts, relabel the case, edit the frozen fixture or join, or weaken the bar.

**Entry and close.** Authenticate the live tree as continuation 75 did — the accepted 834,111, the maintained 5,400 and the design 590 files, the three inherited document drifts (TODO `aa4eb998...` <- prior `03cc9a96...`, brief-623 `9cec0b7d...` <- `ed0704fe...`, brief-742 `6efb5ee8...` <- `54589caf...`), the C65-C75 sealed draws, the 405 director drifts and the schema; attempt 1's entry evidence (`entry-hashes.json`, `entry.log`) is the baseline, and identity that is unchanged is reused per the 2026-11-14 ruling (mtime+size prefilter; hash whatever differs; never re-hash git-tracked source). The live tree's delta against that baseline is exactly this resume order's append to brief-742, the continuation-76 and continuation-76-resume scratch trees, the PROGRESS append and the report; any other delta is a STOP. Close with the same identity and `production_changes=0`; an unexplained delta is a STOP.

**Report.** One `rum-report/1` record with `report.py check 742` passing, covering the whole continuation: attempt 1's run record from the archived log (the unit line, rc 1, peak, the filter line), the sealed schedule and payment, the generator's stages, the stop, the questions it raises and the exact remainder. Changes are everything under `.temp/brief-742` since `report-at-entry.json` (the c75 report `9b9754f5...`). Copy every verdict line; do not paraphrase evidence.

**Budget.** The continuation-76 allocation is 5,536.680777649958961127 s. Attempt 1 spent 1,359.677818829950408605 s (883.177818829950408605 s measured + 0.5 s x 953 receipts) and is charged with the continuation's fixed 260 s, leaving **3,917.002958820008552522 s**. This resume charges measured + 0.5 s per receipt only — no second 260 — with the tracker bound `LIMIT=4177.002958820008552522` and `RESERVE=60.0`; admit before the first command, run no group that cannot fit, and close with the exact remainder so the next continuation starts from it.

**Scope.** No live edit, no git, no network and no `unsafe`; scratch only under `.temp/brief-742/continuation-76-resume/` plus the report and the PROGRESS append; the working tree keeps exactly the 23 inherited porcelain entries. Stop for the director's gate.

**Dispatch:** `BRIEF_WORKER=mst scripts/dispatch-brief.sh 742`.

## Continuation 76 ruling (director) — continuation 76 runs the complete 864-draw schedule at 571/4,422 under the predeclared per-activation cardinality and then stops at the generator's local-handoff control: P0292's `core::hit` reselects the ambient copy of an omission receipt instead of consuming the authenticated local receipt. Q76 rules the Q34-shaped consumer repair (option 1) and orders the class-wide authentication, the re-rendered schedule and the generator

Continuation 76 predeclared the per-case activation cardinality (108 cases over 12 sites, zero unknown, P0494 = 3 and 99 one-activation cases; `97b35c1d...`), declared and calibrated the nested control before the build (14:46 < 14:48) with **366 controls — 69 positives and 297 falsifiers rejected** (plan `9b53ff55...`, freeze `4fbe4bfb...`, calibrated freeze `cc301552...`), and ran the **complete 864-draw schedule** at 8/1 on the four builds: 864/864 exact, every pair a full-identity bijection with one match per token at its own activation, zero omission, all receivers exact (results `aaf18a52...`/`4a578f25...`/`7f5a9f7a...`/`d399fcb8...`). The payment was re-derived and the ten executable obligations (641, 1191, 1228, 1318, 1799, 1828, 2021, 2973, 3585, 3614) paid: **571/4,422, 37/134, 4/133, 0/26**, 3,851 unpaid, the eleven Open (363, 392, 769, 798, 1347, 2259, 2655, 2684, 2713, 2742, 3680) named and unpaid (payments `7cc28422...`, qualification `3a2787e2...`, identical across the re-derivation). Attempt 1 then entered the generator — discovery, construction bindings, pair 48/48, class 52 children, grant 84 controls — and stopped at the local-handoff control: `PortBlocked::QueuedOmissionReceiptAmbientReselection`, index 625, P0292, case ambient-occurrence, 4 of 16 draws, matched 2, distinct failed 2 (binary `44d7b28d...`, 268,004,528 B; freeze `9cfd5a32...`; stopped draw `88938b2f...`, 25,291 B). At `core::hit` the queued scene had constructed the genuine local omission receipt (occurrence 0) and its permit; the probe mutated only the ambient copy (occurrence 1) and the hit reselected the ambient occurrence and failed its own authentication at `lib.rs:405` ("hit without own activation token") at both threads, exactly the frozen record's PARTIAL. Attempt 1 died at 15:27:58 to the Codex safety filter while composing its report (rc 1, 953 receipts, no report/close/budget); the routed `mst` resume authenticated the sealed tree by a twice-identical manifest (47,867 files / 13,656,745,486 B), re-derived the payment byte-identically, re-executed exactly the four handoff draws (exits 0,0,101,101, same observations and panic), sealed its own stopped draw (`d779d5c0...`, 8 files), re-derived the generator's stages byte-identically, and reported (39 checks, 4 retained deviations, 1 question `ambient_receipt_occurrence`).

**Ruling `Q76-queued-omission-receipt-ambient-authentication`.** (1) The run and the STOP stand; the schedule, the payment and the generator's re-derived stages are accepted; the bar rises to **571/4,422, 37/134, 4/133, 0/26** with 3,851 unpaid and the eleven Open named. (2) The generator earns nothing: 0/151 plans, 0/94 paths, 0/188 postimages, schema `f9604852...` unchanged; the stop precedes emission. (3) The stop is a genuine candidate-contract violation, not an instrument over-reach; the frozen record already declared the PARTIAL and the probe changed only the ambient copy. (4) The question is ruled option 1: bind the hit to the independent local-source record with the ambient copy read-only, in the Q34 consumer shape — the full identity (episode, origin pair/program/row/path, owner path, occurrence, frame kind) re-derived or compared at the omission boundary, with corruption calibrations at ambient occurrence, origin, episode and frame kind — class-wide, so every consumer that reselects an ambient permission authenticates it against its local source and the class falsifier is that the ambient copy is inert at every such consumer and the resumed path equals the queued path; option 2 is refused (the defect is repairable in the declared surface and holding it would record a correct target as unsatisfied) and option 3 is forbidden (an expectation would be weakened). (5) The payment stands (no sealed draw ever exercised `hit`; no `CM740_SOURCE_HIT` line), but the `hit` insertion is one of the ten registered paths in both arms, so the repair changes the rendered source: the successor re-renders, rebuilds the four schedule builds and re-runs the complete 864-draw schedule; a divergence reopens the payment and is a STOP. (6) Everything sealed stays sealed — the c76 evidence tree (manifest `b134d24a...`), the attempt-1 stopped draw, the resume stopped draw, the plan, the 366-control table, the freeze, the calibrated freeze, the derivation, the payments, the registered edits, the four schedule binaries and the handoff binary — and no continuation may adopt the run, delete attempts, relabel the case, edit the frozen fixture, judge or plan, or weaken the bar.

Continuation 77 is ordered: the `hit` repair at index 625/P0292 in the Q34 shape with its own calibrated controls and falsifiers declared before execution (ambient occurrence, origin, episode and frame kind each inert; the genuine local receipt consumed; missing/foreign/wrong-kind receivers refused); the class-wide audit of the ambient-permission consumers with the class falsifier; the re-declared handoff control over the plan's declared cases on the four builds, calibrated before the build; the re-render, rebuild and **complete 864-draw schedule** at 8/1 (the input changed, so the hash-identity exemption does not apply); then the generator — resolve all 151 carrier plans on the current source, compose and independently audit the two 94-file output sets, emit the six-column schema and the 151 schema cells, 151/151 plans, 94/94 paths, 188/188 postimages on a pass; then the remaining indexed duties toward 134/4,422/133, the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks; stop before any group that cannot fit and report the exact remainder; hash-identity reuse wherever every input is byte-identical (the 2026-11-14 ruling), anything with a changed input re-run. Allocation: continuation 76 charged **1,918.853685601992759835 s** (attempt 1: 883.177818829950408605 s measured + 476.5 s bookkeeping + 260; attempt 2: 282.17586677204235123 s measured + 17.0 s bookkeeping) of 5,536.680777649958961127 s, leaving **3,617.827092047966201292 s**; CM4 now **132,882.172907952031098708 s** of 136,500; no new allocation — continuation 77 runs under the remaining 3,617.827092047966201292 s with its own fixed 200 + 60 inside that bound (`LIMIT=3617.827092047966201292`, `RESERVE=60.0`), workspace 0/6,000 with the 1,440 s verification, second 6,000 s workspace, 57,000 s stage reserve and 40,500 s for 743-746 held. Continuation 76 ran `cdx` (attempt 1, unit `brief-742-cdx-20261119-143908-2087131`, rc 1, peak 7,889,698,816 B, log 2,772,814 B `23053ff1...`, killed at the safety filter) and `mst` (attempt 2, unit `brief-742-mst-20261119-153717-2115920`, rc 0, peak 2,580,938,752 B, oom 0/0, log 7,832 B `d163c46a...`, session `3b87e57d...`, $41.754987, 29,224,023/191,217 tokens; 13 git-fence attempts, all app-server `check-attr eol`); the report is 16,813,229 B `73af5815...` with 39 checks, 4 deviations, 2 disclosures and the one question; no git or network ran, the tracked tree kept exactly the 23 inherited porcelain entries, and `production_changes=0`. Dispatch: `BRIEF_WORKER=mst scripts/dispatch-brief.sh 742`. No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 77.

Continuation 77 record, ruled by `Q77-generator-emission-stale-register`. The Q76 repair is accepted: `core::hit` binds the hit to the independent local-source record with `local_source_token` in the Q34 consumer shape and the ambient copy read-only; the re-declared handoff control (8 cases, 182 controls, 64 draws on four builds at 8/1) keeps the real hit in the six inertness cases (intact, ambient occurrence/origin/episode/frame-kind, foreign-present), refuses missing-receipt and foreign-frame-kind-receipt at the own-activation-token panic (exit 101), leaves the disabled arm inert, and both mutants are load-bearing (M1 refuses on the corrupted ambient copy at exit 101; M2 consumes the foreign sibling receipt and is rejected). The class audit enumerates 4,422 consumer-instance rows: 40 ambient-reselection rows, all `core::hit` (one consumer, six surfaces); index 625/P0292 form-1 is evidenced and the other 39 are listed and unclaimed. The repaired render re-earns the complete 864-draw schedule exactly at 8/1 on the four builds (108 cases = 99 one-activation + 9 three-activation, P0494 = 3; declaration 17:42:03, builds 17:42:47-17:44:37, draws 17:45:26-17:47:43): the ten executable indices re-earned, union 571/4,422, 37/134, 4/133, 0/26 unchanged, divergence false, payment stands, no new obligations. The run then declares the generator emission `budget-fits-portblocked-not-executed` on the frozen continuation-75 register's `PortBlocked::RetirementWitnessNestedPairCardinality` on all 94 admitted overlay paths and stops at the remaining-indexed-duties frontier. Q77 rules: the run and the STOP stand; the repair, the handoff control, the class audit and the schedule are accepted; the generator earns nothing and the bar stays 0/151, 0/94, 0/188; the stop is an ordering defect — an unexecuted group plus a stale-register citation — not a candidate failure and not a standing blocker: the register's reason is the c75 stop that Q75 withdrew, the register is read only for its path set (`prepare77.py:13`, `source_audit.py:9`), no emission script exists, the resolve re-derived all 151 plans fresh, and `full_composition=false` is a hard-coded literal (`generation/resolve.py:107`). Continuation 78 is ordered: execute the emission on the current source — compose and independently audit the two 94-file output sets, emit the six-column schema and the 151 schema cells; 151/151 plans, 94/94 paths, 188/188 postimages on a pass; if the generated bytes cannot reproduce a retained carrier, demonstrate it with the failing path, the preimage and the differing bytes and return the named PortBlocked (the register alone is not that demonstration); any supersession or re-pin of a retained carrier is declared with proof that the difference is exactly the declared repair (the hit binding), never silent; replace or remove the `full_composition=false` literal; then the remaining indexed duties toward 134/4,422/133, the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks; stop before any group that cannot fit and report the exact remainder; hash-identity reuse wherever every input is byte-identical, anything with a changed input re-run. Allocation: continuation 77 charged **3,132.355036869034 s** (1,168 receipts: 2,288.355036869034 s measured + 584.0 s bookkeeping + 260) of 3,617.827092047966201292 s, leaving **485.47205517893235 s**; CM4 now **136,014.52794482108 s** of 136,500; **new allocation** per the 2026-11-19 user ruling: a fresh **12,000 s** focused envelope from the stage reserve, so continuation 78 runs under **12,485.472055178932 s** (`LIMIT=12485.472055178932`, `RESERVE=60.0`), CM4's share becomes **148,500 s**, the stage reserve becomes **45,000 s**, and the 1,440 s verification, second 6,000 s workspace and 40,500 s for 743-746 stay held; workspace 0/6,000. Continuation 77 ran `mst` (unit `brief-742-mst-20261119-164914-2129385`, rc 0, peak 3,922,202,624 B, oom 0/0, log 6,178 B `932f5d80...`, session `545978ae...`, $28.520695, 85,711,801/337,216 tokens; 17 git-fence attempts, all app-server `check-attr eol`); the report is 4,629,170 B `6d472f67...` with 19 checks, 6 retained deviations, 2 disclosures and no question; the close block's 530.282797 s and the narrative's 486.145874 s are superseded by the gate's 485.472055 s (the two receipts written after each); no git or network ran, the tracked tree kept exactly the 23 inherited porcelain entries, and `production_changes=0`. Dispatch: `BRIEF_WORKER=mst scripts/dispatch-brief.sh 742`. No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 78.

Continuation 78 record, ruled by `Q78-retained-carrier-render-recipes`. Continuation 78 executed the Q77 emission. The hard-coded `full_composition=false` literal is removed; the resolve re-derived all 151 plans fresh on the live source (151 rows / 152 forms / 10 classes / 79 subtypes / 94 surface preimages / 152 omission pins; outputs hash-identical to the sealed c77 derivation, `3c5afc22...` and `8c1479a8...`); all 90 retained preimages and 180 postimages re-authenticate against `overlays.tsv` (`f9604852...`, 5,588,150 B; 87 of 90 paths arms-equal; 315 armed and 315 disabled hunks); and the compose rendered 94/94 admitted paths × 2 arms = 188 postimages per root into two byte-identical roots with the independent erasure audit restoring every path to its exact preimage (94/94; 399 artifacts frozen and re-hashed by the director at 0 bad). The generated bytes reproduce only **4/90** retained carriers (the four one-to-two-hunk paths `pattern_resolution/continuation.rs`, `suspension/core/profile_queue.rs`, `associations/basic/lazy_completion.rs`, `structure_ops/sequence/take_drop/lazy_completion.rs`). The demonstration is `crates/rum-host/src/model/mod.rs`: preimage `ec87f29a...` (920 B), pinned `ae81be3c...` (965 B), composed = preimage, 0 declared records, the undeclared pinned line `pub(crate) use outcome::cm740_output_scope;`, 177 differing bytes (`3a0b4c75...`), named `PortBlocked::RetainedCarrierRenderRecipeUndeclared`. The audit's breakdown: 23 RENDER-RULE-OWED (all pinned delta lines exactly declared; the one-line insert/replace rule cannot place them), 49 UNDECLARED-RECIPE (10 coverage-none + 39 partial), 14 symbol-heuristic "declared supersession"; 246 of 315 hunks carry an exact declared statement (247 an exact anchor, 67 neither); the carrier-site join declares 241 and leaves 74 uncovered (94 distinct lines). No schema was emitted: `overlays.tsv` unchanged, no `source_schema` binding, no provisional digest, `production_changes=0`.

**Ruling `Q78-retained-carrier-render-recipes`.** (1) The run and the STOP stand; the literal fix, the 151-plan resolution, the pin authentication, the 94/94 composition, the 188/188 double-root postimages, the 94/94 erasure and the 399-artifact freeze are accepted as the emission's record; the schema transition is not earned. (2) The PortBlocked is admitted as a genuine generator-design gap, not a candidate failure: the design's own record says the statement extraction is unimplemented (`extracted_statement_controls.implemented_in_this_design=false` for 149/152 forms), the per-class contracts are semantic rather than text templates, and the frozen declaration sources determine at most 246/315 hunks exactly — so the statement-copy compose could reproduce only where a declared text matched a pinned hunk. (3) The demonstrated line is a genuine missing-recipe line, not an obsolete pin: brief-741 declares P2600's call `crate::model::cm740_output_scope("P2600")` in `crates/rum-host/src/session.rs`, but the definition (`crates/rum-host/src/model/outcome.rs:856`) and the export (`model/mod.rs`) are declared nowhere, and the emitted `session.rs` cannot compile without them. (4) The 14 symbol-heuristic supersessions are not accepted: the heuristic is not proof and `CM740_OUTPUT_ROW` is a host-side thread-local defined in the pinned overlay file itself (`outcome.rs:854`), a false positive; the accepted c77 helper (158,156 B `c7149582...`) does drop `option_scope`, `omit_own_join`, `actual_join_stop`, `fixedpoint_owner` and `cm740_adopt_outcome`, so interface-replacement supersessions are plausible for those paths, but each owes per-line proof that the accepted replacement carries the pinned omitted operation at the same site with preserved nodes; `lib.rs`'s entry-helper replacement (4,502 B `106440a6...`) is the one self-evident candidate. (5) Continuation 79 is ordered: (a) the complete residue inventory (Scope: NONE) over all 315 pinned armed hunks with zero unknown, each classified as exact statement, class-template statement with its template and row-field substitutions, host-side/helper scaffolding with the recipe item, structural reflow of a declared replacement, or interface-replacement supersession with the accepted API and mapped carrier, and per path the 4 reproduced, the 23 render-rule failures, the 49 undeclared residues and the 14 candidates' status; (b) the recipe surfaces — the class-template registry for the ten classes and 79 subtypes producing exact text from the row declarations (never copied from a historical postimage), the helper/scaffolding recipe with its inverse, the declared multi-line structural replacement rule and the extended erase/inverse, and the per-path supersession proofs; (c) the emission on the current source — two fresh roots byte-identical, erasure 94/94, every retained carrier reproduced or its difference proven as a declared repair; on a pass emit the six-column schema (94 rows = the 90 retained + 4 proposed extensions) and bind the 151 `source_schema` cells mechanically from the exact table bytes; then the remaining indexed duties toward 134/4,422/133, the audits, the ordered standing replay, `cargo fmt --check`, both workspace modes and the switched checks; stop before any group that cannot fit and report the exact remainder; hash-identity reuse wherever every input is byte-identical, anything with a changed input re-run. (6) No weakening: the four extension paths stay in the set, no re-pin without proof, no silent normalization, no hand-edited cell, no placeholder digest, no row relabeling, no adoption or deletion of attempts. (7) Everything sealed stays sealed — the c76 scene, the c77 class/handoff/schedule seals, the c78 freeze (399 artifacts), the c78 pins (90/90, `f9604852...`), the two roots, the emission audit/erasure/demonstration and all 25 receipts. (8) The bar stands at **571/4,422, 37/134, 4/133, 0/26** with 3,851 obligations, 97 actors and 129 fields outside; the generator pass is not earned (151 plans resolved, 94/94 composed, 188/188 postimages, 4/90 retained carriers reproduced, 0 schema rows emitted); schema `f9604852...` unchanged; standing replay NOT_RUN; workspace 0/6,000.

**Allocation.** Continuation 78 charged **284.0800406759954 s** (25 receipts: 271.5800406759954 s measured + 12.5 s bookkeeping) of 12,485.472055178932 s, leaving **12,201.392014502937 s** as continuation 79's limit (`LIMIT=12201.392014502937`, `RESERVE=60.0`; spendable 11,941.392014502937 s after the fixed 200 and reserve 60); CM4 now **136,298.60798549707 s** of 148,500; the stage reserve is **45,000 s**; no new allocation is needed; the 1,440 s verification, second 6,000 s workspace and 40,500 s for 743-746 stay held; workspace 0/6,000. Continuation 78 ran `mst` (unit `brief-742-mst-20261119-190331-2190973`, 19:03:31-20:21, rc 0, peak 1,141,039,104 B, oom 0/0, log 7,241 B `c318c1c8...`, session `5e738514...`, $16.7767332, 48,946,356/208,473 tokens; 13 git-fence attempts, all app-server `check-attr eol`); the report is 178,797 B `5a23ae80...` with 19 PASS + 1 FAIL + 2 measurements, 5 retained deviations, 7 disclosures and no question; the budget.json's 226.0421481919766 / 11,999.429906986956 predate the report's own receipt and the report's 11,941.392015 is the exact spendable figure; one disclosed deviation briefly created `/tmp/pre-resolve.sha` and moved it into the continuation tree before anything but reading it (the `.temp`-only rule is restated); no git or network ran, the tracked tree kept exactly the 23 inherited porcelain entries, and `production_changes=0`. Dispatch: `BRIEF_WORKER=mst scripts/dispatch-brief.sh 742`. No campaign, epoch, acceptance or CM5; pair (45,5,false) Open with zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; `CHANGELOG.md` and S10 blocked. Next: continuation 79.

## Continuation 79 ruling (director) — Q79: the emission is mechanically exact but two omission supersessions are provably inert and the class-template registry is absent; the schema transition is not earned

Continuation 79 is accepted as a STOP. Run: `mst` (unit `brief-742-mst-20261119-203620-2210076`, 20:36:20-23:16, rc 0, peak 15,581,335,552 B, oom 0/0; log 11,699 B `ac3e105a...`; session `4e5b97af...`, $28.65706876, 64,222,453/218,848 tokens; 7 git-fence attempts, all `check-attr eol`). Report: 63,761 B `bb0ff14f...`, STOP, 8 PASS + 2 FAIL + 1 measurement, 5 retained deviations, 0 disclosures, 0 questions, 119 citations verified. Budget: focused 58 receipts — 1,137.5599972250347 s measured + 29.0 s bookkeeping = 1,166.5599972250347 s of 12,201.392014502937, leaving 11,034.832017277902 s; workspace envelope 13 receipts — 2,745.2728506270214 s measured + 6.5 s = 2,751.7728506270214 s of 6,000, leaving 3,248.2271493729786 s; CM4 137,465.1679827221 s of 148,500; stage reserve 45,000 s.

**Delivered and accepted.** (a) The residue inventory classifies all 315 pinned armed hunks with zero unknown: my independent diff against the pinned corpus finds 315/315 matched, 300 identical added blocks, 15 declared-diff hunks each with an accepted API, 1 declared scaffold hunk, 0 undeclared differences — 316 recipe hunks and 771 classified lines (the frozen aggregates still say 315/761 and are corrected by the ruling). (b) The recipe programs, the Q26/Q27 extension recipe, the declared multi-line replacement rule and the extended erase/inverse; the class-template registry is not delivered. (c) The compose rendered 94 paths x 2 arms into two byte-identical roots with erasure 94/94 (93 mechanical, the helper arm by hash identity against the sealed c76/c77 audits) and 78 REPRODUCED / 12 SUPERSESSION-DECLARED / 4 EXTENSION-DECLARED / 0 mismatch; my gate re-run on a copy is byte-identical (manifest `bbf09e8c...`, erasure `dfc0dab9...`, reproduction `94d47948...`, residue `7ad8a9cd...`, 376/376 root files). The emitted `overlays.tsv` (94 rows, digest `56a17177...`) reconstructs byte-exactly from root-a plus the live tree, and `mutants.tsv` equals the entry bytes with exactly the 151 cells rebound (revert-and-hash `f55108fc...`). Both arms compile on the drive surface (armed-check5 and disabled-check exit 0 over rum-runtime/rum-stdlib/rum-host plus the oracle test with the nine mutated manifests); the standing replay is green end to end, `cargo fmt --check` is clean, the switched checks are green with identical verdict lines, and the scope is exactly the two tables against the 69-file entry snapshot with the tracked tree at the 23 inherited porcelain entries and `production_changes=0`.

**The two stops.** The workspace red is structural: exactly two tests per mode fail (`cm_receipts_exact`, `cm_p0543_receipts_exact`) on the stripped `RUM_CM739_LANE_*` identity (my one-thread rerun of `cm_p0543_receipts_exact`: FAILED in 75.12 s with `Err(Program)`); the bar's failed=0 is not met and the authenticated lane cannot fit. Two declared supersessions are provably inert: `profile_queue.rs:371` `omit_evaluation_pass(Option::<Namespace>::None)` returns false for every state (`registered?` at lib.rs:2491 is None), `profile_queue.rs:612` `omit_join(Option::<Namespace>::None)` returns false (`namespace?` at lib.rs:2950 is None), while the pinned calls were state-dependent (`PASS_OWNER`/`OWN_JOIN`); their carrier classes are the `carrier` of five mutant rows (P1470/P1471 `OwnFixedPointPassPublicationAfterEvaluationReturn`, P0267/P0312/P0323 `OwnDeferredDefinitionPublicationAtActualProfileQueueJoin`), and `JoinOwner::expect_result`/`finish_result` have no call sites. The proof's "omit_join resolves the receiver from the independently registered installation" is contradicted by the code — the caller supplies it. The class-template registry does not exist: the six class-template lines are pinned text classified by regex, and `emission_build.py` builds the retained recipe by diffing the maintained table's own postimage cells plus the declared substitutions.

**Ruling `Q79-carried-omission-receivers`.** (1) The run and the STOP stand; the mechanical products above are accepted. (2) The schema transition is NOT earned: `source_schema` remains `f9604852165e377f3be8faf2ee1ec3c7ffc2c67d555dcd17337c8f7a65a4ade9`, the `56a17177...` tables are this composition's working record only, and the bar stays **571/4,422, 37/134, 4/133, 0/26** with 3,851 obligations, 97 actors and 129 fields outside. (3) The two omission supersessions are rejected: no inert call may be declared as carrying an operation; for the five rows the pinned operation must be carried at the site with the actual receiver supplied and the join result consumed, or declared as moved to an accepted replacement carrier with proof the class is observed there. (4) The class-template registry must be delivered from the declarations: template id plus row-field substitutions, generated text equal to the pin byte-for-byte, no reading of the maintained table's postimage cells for those lines. (5) The inventory aggregates are corrected to 316 recipe hunks and 771 lines (322 exact-statement / 338 structural-reflow / 67 helper-scaffold (59 declaration + 8 struct-field) / 26 control-statement / 6 class-template / 3 interface-supersession / 9 blank), zero unknown. (6) Continuation 80 is ordered in sequence with stops: the carried-operation repair and the registry with code-level proofs; a focused drive observation of the five carrier classes on the composed instrument before any re-pin; the re-composition into two fresh byte-identical roots with erasure 94/94 and the per-path reproduction audit; on that pass the re-emission of the six-column 94-row table and the mechanical 151-cell rebind from the exact bytes (the digest will change again; `56a17177...` is not accepted); then the authenticated P0543 lane and the remaining indexed duties toward 134/4,422/133; then both workspace modes with the lane configured (the exact seven skips; full unfiltered `suspension_oracle` and `typed_slot_census` at failed=0, filtered=0), `cargo fmt --check` and the switched checks; stop before any group that cannot fit and report the exact remainder; hash-identity reuse wherever every input is byte-identical, anything with a changed input re-run. (7) No weakening: no re-pin without a proof that holds, no silent normalization, no hand-edited cell, no placeholder digest, no row relabeling, no adoption or deletion of attempts. (8) Everything sealed stays sealed — the c76 scene, the c77 seals, the c78 freeze (399 artifacts) and pins, the c79 artifacts (two roots, recipe programs, manifest/erasure/reproduction/residue/schema-emission records, compile and replay logs, the two workspace logs) and all 71 receipts. (9) Allocation: c79 charged 1,166.5599972250347 s of 12,201.392014502937, leaving `LIMIT=11034.832017277902` (`RESERVE=60.0`, `FIXED=200`, spendable 10,774.832017277902); the workspace envelope has 3,248.2271493729786 s of its 6,000 remaining; CM4 now 137,465.1679827221 s of 148,500; the stage reserve is 45,000 s; the 1,440 s verification, second 6,000 s workspace and 40,500 s for 743-746 stay held. (10) Routing: continuation 80 dispatches to `mst` — the Codex safety filter refused this brief's task shape at c76 attempt 1.

**Chronology.** Recipes/compose 21:31, schema-emit 21:33, compile gates 21:31, standing replay and fmt 21:58, workspace 21:58-23:06, report 23:15, dispatch end 23:16; the schema was emitted before the workspace run, as the Q78 order directed.

**Verified by me.** The dispatch and report records; the compose on a gate copy; the schema tables cell-by-cell and byte-for-byte; the residue inventory against the pinned corpus; the declared APIs and the inert `None` receivers by reading the helper's implementations; the compile, fmt and standing-replay logs; the workspace per-binary results (the oracle 577/2/5 in both modes, the census green unfiltered, the runner's env stripping) and the one-thread lane rerun; the 69-file entry scope; and the exact budget arithmetic. Deviations accepted: the stray census filter (rerun unfiltered), the beyond-letter compile gate (productive — it surfaced the six defects), the zero-test one-thread red-verify (my rerun supplies the determinism), the external workbench disclosure (outside the fence, not adopted), and the maintenance-runner restarts.

Next: continuation 80.

## Continuation 80 ruling (director) — Q80: the class-template registry, the corrected inventory and the mechanically exact re-emission execute, but the carried-operation repair is still inert because every producer of the consumers' authentication state has no call sites; the schema transition is not earned and the producer wiring is ordered

Continuation 80 is accepted as a STOP. Run: `mst` (unit `brief-742-mst-20261119-233515-3578480`, 2026-11-19T23:35:15–2026-11-20T02:27, rc 0, peak 16,313,430,016 B, oom 0/0; log 11,463 B `35afad7b...`; session `66ac5563...`, $28.6383852, 86,849,442/318,252 tokens; 13 git-fence attempts, all `check-attr eol`). Report: 198,078 B `74b00d76...`, STOP, 17 PASS + 1 FLAGGED + 2 measurements, 6 retained deviations, 1 disclosure, 0 questions, 176 citations verified. Budget: focused 41 receipts — 2,274.3769229829195 s measured + 20.5 s bookkeeping = 2,294.8769229829195 s of 11,034.832017277902, leaving 8,739.955094294983 s (`LIMIT` for continuation 81, spendable 8,479.955094294983); workspace ledger 27 entries (13 inherited c79 + 14 new) — 4,859.558886640967 s measured + 13.5 s = 4,873.058886640967 s of 6,000, leaving 1,126.941113359033 s, topped up by 3,000 s from the stage reserve to 9,000; CM4 139,760.04490570502 s of 148,500; stage reserve 42,000 s.

The mechanics executed. The class-template registry is delivered from the declarations (10 classes / 79 subtypes / 6 entries; each generated text equals the pinned corpus line byte-for-byte — my check 6/6). The inventory is corrected to 316 recipe hunks / 771 lines, zero unknown. The compose rendered 94 paths x 2 arms into two byte-identical roots with erasure 94/94 and 78 REPRODUCED / 12 SUPERSESSION-DECLARED / 4 EXTENSION-DECLARED / 0 mismatch; the emitted overlays.tsv (94 rows) binds every preimage to the live tree and every armed/disabled sha256 and hex cell to the composed root (my independent check: 94/94 rows, 0 bad), digest `2032dd20...`; mutants.tsv's 151 source_schema cells all carry the working digest, and reverting them to `f9604852...` reproduces the c78-entry bytes `f55108fc...` exactly. Both arms compile; the standing replay is green end to end, fmt is clean; the workspace is green in both modes under the exact seven skips (oracle 579/0/5, census 10/0/1, 0 filtered; the aggregate's two FAILED lines are the census's inner forced_red_probe children); the authenticated lane is built on the current tree (5,806 files / 5,797 byte-identical, nine manifest edits, binary `85f1eb78...`) and passes P0543 55/55 and the campaign 151 rows / 9,060 bound / multiplicity 1 / remaining 0 (PortBlocked=1445 standing, full_CM_green=false honest). Scope: the tracked tree is exactly the 23 inherited porcelain entries, the tables untracked, local-contracts.tsv byte-identical (`dac98a5b...`), production_changes=0.

The stop stands because the repair does not hold at the code level. The composed tree calls only six pre-existing helper APIs (`source_evaluation_owner` x2, `source_join_owner` x3, `source_observe_join_result` x1) and the two repaired consumers (`profile_queue.rs:390-391`, `:658`); every producer of the consumers' authentication state has zero call sites outside the helper: `source_installing` (lib.rs:2073; `state.installed` written only in `InstallGuard::drop` :2374), the registration observers (`records.receivers` only via `source_install` :1799-1801), `source_append` (`state.appending` pushed only in `join_append` :2903), `source_finish_append_call`, and `source_expect_result`/`source_finish_result` (`state.confirmed`/`confirmed_at` written only in `join_result_kind` :2714-2715, called only by `finish_result` :2643 / `join_result` :2691). The E site's `omit_evaluation_pass` passes its `Some(registered)` check but fails at `state.evaluation.get(&scope)?` (`source_proof` :2390 needs `state.installed` — empty) and at `receiver_installation` (:1807, `records.receivers` empty); the J site's `omit_join` fails at `state.appending.last()?` (empty) and `confirmed_append` (:2723, :2736/:2738 — empty). Both return false for every reachable state — the same silent loss c79 rejected, with an unpopulated authentication state in place of the `None` short-circuit. The five rows (P1470/P1471 `OwnFixedPointPassPublicationAfterEvaluationReturn`, P0267/P0312/P0323 `OwnDeferredDefinitionPublicationAtActualProfileQueueJoin`) remain unobserved. The focused drive is accepted as recorded and is evidence against the repair: zero probe lines in six runs, the composed arms bind-reject at `SourceIdentity` (`coverage/execution_tests.rs:347`/`:215`, `schedule.rs:96`), and the live controls run a helper-free live tree. The c79 switched-check false green is corrected: lp655/lp656's c79-era stale path matched zero tests; the real path fails with and without the switch and at baseline (`control.rs:1137`) — inherited-red permanent exclusions, recorded; lp03/04/05 remain green. My own gate-side incident is disclosed: the composer's output dir is hard-coded to continuation-80, so my re-run rewrote the compose records and roots in place; no evidence changed (the residue file hash-identical to the pre-run copy; the other records' byte sizes equal the pre-run listing; the roots byte-identical to the emitted table cells), and the gate-copy rule is restated.

**Ruling `Q80-carried-omission-producer-reachability`.** (1) The run and the STOP stand. (2) The class-template registry (Q79(4)), the corrected inventory (Q79(5)) and the mechanical composition/erasure/reproduction are accepted as delivered; the two repaired omission supersessions are rejected: no call may be declared as carrying an operation while every producer of its authentication state has zero call sites — the install/registration, append and `JoinOwner` completion producers must be wired at their sites, with the mechanical inverse, the code-level reachability table and a calibrated falsifier. (3) The schema transition is NOT earned: `source_schema` remains `f9604852165e377f3be8faf2ee1ec3c7ffc2c67d555dcd17337c8f7a65a4ade9`; the `2032dd20...` tables are the working record only and must not be cited as accepted; the bar stays **571/4,422, 37/134, 4/133, 0/26** with 3,851 obligations, 97 actors and 129 fields outside. (4) The census `ActiveRequiredBag` path is inadmissible on any composed identity (`SourceIdentity` by design); the carrier observation must use a same-binary instrument that binds by construction. (5) Continuation 81 is ordered in sequence with stops: (i) the producer wiring (install/registration, append, `JoinOwner` completion) as declared supersessions with their inverses, both arms; (ii) the reachability proof per consumer predicate plus the calibrated falsifier — remove one producer call, the class goes silent and the drive reddens; restore it, the class fires; (iii) the focused same-binary drive of the five carrier classes on the composed arm with the hit lines recorded verbatim; (iv) the re-composition into two fresh roots with erasure 94/94 and the per-path reproduction audit; (v) on that pass the re-emission of the six-column 94-row table and the 151-cell rebind from the exact bytes (the digest changes again; `2032dd20...` is not accepted); (vi) the standing replay, `cargo fmt --check`, both workspace modes with the lane and the exact seven skips, and the switched checks; the remaining indexed duties if any; stop before any group that cannot fit and report the exact remainder; hash-identity reuse wherever every input is byte-identical. (6) No weakening: no re-pin without a drive that holds, no hand-edited cell, no placeholder digest, no row relabeling, no adoption or deletion of attempts, no silent normalization. (7) Everything sealed stays sealed — the c76 scene, the c77 seals, the c78 freeze (399 artifacts) and pins, the c79 artifacts and all 71 receipts, and the c80 artifacts (two roots, recipe program, registry, manifest/erasure/reproduction/residue/schema-emission records, drive logs, lane build and drives, workspace logs) with its 55 new receipts. (8) Allocation: c80 charged 2,294.8769229829195 s of 11,034.832017277902, leaving `LIMIT=8739.955094294983` (`RESERVE=60.0`, `FIXED=200`, spendable 8,479.955094294983); the workspace envelope is topped up by 3,000 s from the stage reserve to 9,000 (4,873.058886640967 spent, 4,126.941113359033 remaining); CM4 now 139,760.04490570502 s of 148,500; the stage reserve is 42,000 s; the 1,440 s verification, second 6,000 s workspace and 40,500 s for 743-746 stay held. (9) Routing: continuation 81 dispatches to `mst` — the Codex safety filter refused this brief's task shape at c76 attempt 1.

**Chronology.** Build 00:19, compose 00:20, compile 00:20-00:21, focused drive 00:23-00:31, schema-emit 00:32 (the observation precedes the re-pin), standing replay 00:36-01:05, lane 01:05-01:19, workspace 01:25-02:12, fmt 02:15/02:20, report 02:26, dispatch end 02:27.

**Verified by me.** The dispatch and report records; the composer re-run on a gate copy (identical verdict) and the roots through the emitted overlay cells (94/94 rows, every sha256 and hex cell); the schema tables independently (overlays digest `2032dd20...`, 151 cells rebound, the revert to `f9604852...` yielding the c78-entry `f55108fc...` exactly); the registry (6/6 generated text equals the pin; the 10-class / 79-subtype declaration); the producer gap by scanning every helper API over all 94 composed paths and reading the consumers' predicates and producers; the drive logs; the workspace per-binary results and the exact seven skips; the lane verdict lines; the switched checks including the lp655/lp656 stale-path versus real-path and baseline reds; the standing replay r4 and its instruments; fmt; the compile gates; the scope directly; and the exact budget arithmetic. Deviations accepted: the warm-started c79 mirror (byte-identical regeneration disclosed), the r2/r3 runner-assert stops (replaced by the full unfiltered runs), the 300 s-killed first no-default attempt (re-run), the stale-path switch defect (corrected here), the warm-started compile surfaces, and the two seeded note lines (disclosed; not to be repeated — the log is written by the command). My own gate-side composer incident is disclosed in the ruling; no evidence changed.

Next: continuation 81.

## Continuation 81 ruling (director) — Q81: the producer wiring carries the two repaired omissions, the calibrated falsifier and the by-construction drive observe all five carrier classes, and the schema transition is earned

Continuation 81 is accepted as DONE. Run: `mst` (unit `brief-742-mst-20261120-024259-3701841`, 2026-11-20T02:42:59–05:38, rc 0, peak 14,733,082,624 B, oom 0/0; log 17,285 B `c9e090ff...`; session `82267832...`, $27.51866612, 81,849,887/299,936 tokens; 7 git-fence attempts, all `check-attr eol`). Report: 489,970 B `0f45cdac...`, DONE, 15 PASS + 3 measurements PASS, 5 retained deviations, 5 disclosures, 0 questions, 352 citations verified. Budget: focused 92 receipts — 3,982.599862453877 s measured + 46.0 s = 4,028.599862453877 s of 8,739.955094294983, leaving 4,451.355231841106 s spendable; workspace 41 receipts (27 inherited + 14 new) — 7,439.418026114028 s of 9,000, c81's own 2,566.359139473061 s, remaining 1,560.5819738859718 s; CM4 146,355.00390763195 s of 148,500; stage reserve 42,000 s.

The wiring executed. The recipe program carries 326 hunks / 860 armed lines over 90 retained paths, zero unknown, 45 producer-wiring lines over 6 wired paths. Every zero-call-site producer now has its site: `source_pause_installed()` after the pinned pause; `source_installing()`/`source_generation` at the actual install and `source_expect_registration`/`source_finish_registration` around `register_profile_owner` (the InstallGuard drop completes it); `source_observe_registration`/`retire_control` at `register_owner`, `clear_registry`/`retire_control`/`retire_root` at the registry hooks, `source_observe_append_queue` at the actual `ProfileQueue::append` edge; `source_append`/`source_bind_append_queue`/`source_finish_append_call` around the append edge; `source_capture_scope`/`source_join_owner_at`/`source_expect_result`/`source_finish_result` at the three resume sites (the `?` desugared into the tuple-typed early return). Both arms compile. The reachability proof per consumer predicate is delivered, and the calibrated falsifier removed exactly one producer call (`source_observe_append_queue`, occurrences=1, other 93 paths verified): three sampled plans ran canonically (exits={0:3}) with zero hit lines and the drive reddened (exit 101); restored, the class fires. The by-construction drive fired all five carrier classes on the composed armed instrument with the verbatim identities (P0267/P0312/P0323 `CM742_JOIN_HIT_ID` with producer_occurrence=0 generation=2; P1470/P1471 `CM742_QUEUE_HIT_ID` generation 3/4) and the disabled arm ran the full cohorts (60/60/60/45/45) canonically with zero class activity; the armed JOIN children exit 101 at the epoch teardown (left (8,8,1,0,0)) — the runtime's own catch of the destructive omission, disclosed and never normalized. The re-composition gave two byte-identical roots with erasure 94/94 and 76 REPRODUCED / 14 SUPERSESSION-DECLARED / 4 EXTENSION-DECLARED / 0 mismatch (the two moved paths exactly `builtins_input.rs` and `suspension/core.rs`), and the re-emission rebound the 151 cells to `9dd3687b...` (the 94-row table's file sha256). Standing replay all_steps_pass=true; fmt clean; workspace both modes 4,371/2/66 (oracle 579/0/5, census 10/0/1, 0 filtered; the two FAILED lines are the census's inner forced_red children); lane tree `a5084dd2...`/binary `f29e51ff...` with P0543 55/55 and the campaign 151 rows / 9,060 at multiplicity 1 / remaining 0 (PortBlocked=1445, full_CM_green=false); lp03/04/05 green, lp655/656 recorded inherited-red. Scope: the 23 inherited porcelain entries, `local-contracts.tsv` `dac98a5b...`, production_changes=0; entry 5,806 files at 03:01.

**Ruling `Q81-producer-wiring-and-earned-schema`.** (1) The run and the DONE stand. (2) The producer wiring — install/registration, append, pause-installed, JoinOwner completion, registry retirement — is accepted as declared supersessions with their mechanical inverses, both arms, with the 326-hunk/860-line/zero-unknown recipe program; the reachability proof and the calibrated falsifier are accepted. (3) The by-construction drive is accepted as Q80(4)'s carrier observation: all five carrier classes fire on the composed armed instrument with the verbatim hit identities, the disabled arm is canonical with zero class activity, the census path was never entered; the armed per-row early exit is the instrument's design and the full cohorts are the disabled arm's and the campaign's. (4) The re-composition (byte-identical roots, erasure 94/94, 76/14/4/0) and the re-emission (six-column 94-row table, 151-cell rebind) are accepted. (5) **The schema transition is earned and accepted**: the accepted `source_schema` is `9dd3687bf3c6909bfe3898c3c40be7e8a28a3d0f6a3bd6f664e1c60d9fc24855`; `f9604852...` remains the c78 pins' historical anchor and is superseded; `2032dd20...` was never accepted and is withdrawn; no further re-emission is ordered unless the composed identity changes. (6) No weakening and everything sealed stays sealed (c76/c77/c78/c79/c80 and the c81 artifacts with its 92 focused + 14 workspace receipts). (7) The bar is unchanged — 571/4,422, 37/134, 4/133, 0/26, 3,851 obligations / 97 actors / 129 fields outside; pair (45,5,false) Open; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; no campaign, epoch, acceptance, CM5 or new catch. (8) CM4's implementation line is complete at this gate; the held verification stage (Q82) is ordered next: the two compose roots rebuilt fresh, the full workspace in both modes on the composed tree under the exact seven skips, the authenticated P0543 + campaign lane on the composed binary, and the five carrier rows re-observed with the disabled cohorts canonical, under the held 1,440 s focused envelope and the second 6,000 s workspace, with no re-derivation of the inventory, no new schema emission unless the composed identity changes and no repo mutation; then 743 (observation policy + final epoch + class admission), 744 (shakedown), 745 (full campaign), 746 (acceptance); CM5 remains unauthorized. (9) Allocation: c81 charged 4,028.599862453877 s focused and 2,566.359139473061 s workspace; CM4 now 146,355.00390763195 s of 148,500; the 1,440 s verification, second 6,000 s workspace and 40,500 s for 743-746 stay held; any shortfall gets an explicit carve from the stage reserve with the charge recorded, per the 2026-11-19 user ruling. (10) Routing: continuation 82 dispatches to `mst`.

**Chronology.** Entry 03:01, first compose red (anchor drift line 758) 03:11-03:12, rebuilt recipe + compose PASS 03:29, copies/overlays/compile gates 03:26-03:31, drive builds 03:32-03:54, drive runs 03:55-03:58, falsifier 04:01-04:04, restore/re-fire 04:04-04:07, schema re-emission 04:08, replay 04:12-04:23, lane 04:26-04:42, workspace 04:44-05:27, switches 05:27-05:28, close 05:28, report 05:36, dispatch end 05:38.

**Verified by me.** The dispatch and report; the composer re-run on a gate copy under `.temp/director/gate-c81/` (patched output dir; identical verdict, the four records byte-identical, roots diff-empty); the tables independently (94/94 overlay cells, 151 cells at `9dd3687b...`, the revert to `f9604852...` yielding `f55108fc...` exactly); the producer call sites in the composed render and the consumer sites at `profile_queue.rs:390`/`:658`; the falsifier needle's uniqueness; my own three drive re-runs (armed P1470 fired; armed P0267 fired with the teardown red; disabled P0267 60/60 canonical zero hits); the lane verdict lines; the workspace per-binary results and the exact seven skips; the switched checks (lp03/04/05 green; lp655/656 zero-test stale paths vs the real baseline red); the standing replay and fmt; the compile gates; the scope directly; and the exact budget arithmetic. Deviations and disclosures as listed are accepted; the first compose and first build-recipes reds are retained in the run record.

Next: continuation 82 (the verification stage).

## Continuation 82 ruling (director) — Q82: the verification stage runs; the composed identity does not build

Continuation 82 is accepted as STOP. Run: `grk` (unit `brief-742-grk-20261120-073821-3913435`, 2026-11-20T07:38:21–08:53, rc 0, success, 78 turns, $7.2999, refused=0, peak 16,713,207,808 B, oom 0/0; session `a2e7494b-…`; census green: 9 tools, no MCP, bypassPermissions inside `rum-worker` strict/network-restricted/.git-denied; 3 git-fence attempts). Report 152,501 B `cf5dbe0d…`, STOP (`SandboxLoopbackBindRefused`), 9 PASS / 2 FAIL / 1 NA, 2 measurements, 4 deviations, 0 disclosures, 0 questions, 43 citations verified. Before it: the `mst` dispatch died on the exhausted Vibe pool (`402`, 06:13, $6.93, only scaffolds; log archived `.prev-20261120-0613`), the `qcn` dispatch died at the Qoder account wall (`403 error_code 112`, 345.31 credits, 55 turns, no repo mutation; archived `.prev-20261120-0722`; session `ba135635-…` resumable), and the user authorized grok for this run.

The delivered groups. The fresh compose roots are **188/188 identical to the sealed c81 roots** (`IDENTITY fresh_vs_c81 same=188 differ=0`) with 76 REPRODUCED / 14 SUPERSESSION-DECLARED / 4 EXTENSION-DECLARED / 0 mismatch, erasure 94/94, helper `c7149582…`; my composer re-run on a gate copy reproduces all four records byte-identically. The schema is unchanged (`overlays` `9dd3687b…`, `mutants` `b0830955…`, `local-contracts` `dac98a5b…`, `production_changes=0`) and no re-emission was ordered. The five carrier rows re-fired verbatim (P0267/P0312/P0323 `CM742_JOIN_HIT_ID … producer_occurrence=0 generation=2`; P1470/P1471 `CM742_QUEUE_HIT_ID … generation=3/4`; the armed JOIN children exit 101 at the epoch teardown, `left: (8,8,1,0,0)` / `(28,28,1,0,0)`), and the disabled cohorts are canonical (60/60/60/45/45, every `class_hits=queue_hits=join_hits=0`, `inert=true`). The lane re-run on the unchanged live-tree binary `f29e51ff…` is P0543 55/55 and campaign 151 rows / 9,060 / multiplicity 1 / remaining 0 (PortBlocked=1445, full_CM_green=false). The worker's workspace ran the **live tree**, not the composed tree, and its sandbox refused loopback binding (4,357 passed / 11 failed / 66 ignored per mode — the nine native ZeroMQ tests plus the two accepted inner probes); my host rerun of the identical command is green in both modes: **4,366 passed / 2 failed (the two accepted inner probes) / 66 ignored**, the nine native tests executing.

The finding. The ordered **composed-tree workspace was not run**, so I constructed it on the host (a fresh copy of the live tree plus the 94 armed postimages from the sealed c82 root-a), and it **fails to build**: `crates/rum-jupyter/src/adapter/worker.rs:359/370/391/394` reference `rum_runtime::cm740_probe` and `rum-jupyter` has no `rum-runtime` dependency (E0433 ×4). The disabled arm reproduces it (a disabled overlay copy fails `cargo check -p rum-jupyter` with the same four errors), and a full scan of the 94 rendered paths against every crate's dependency graph finds exactly this one unresolved cross-crate reference (`crate::cm740_probe` occurs only inside `rum-runtime`, 62 files; `rum_runtime::cm740_probe` resolves in `rum-host` 2 and `rum-stdlib` 21). No stage had ever compiled the jupyter render: c79 disclosed “rum-jupyter armed not compiled”, and the c81 compile gates were `cargo check -p rum-runtime -p rum-stdlib -p rum-host` on copies whose jupyter file had been restored — the c81 “both arms compile” holds for those three crates only, and the jupyter carrier sites were never exercised. My own Q82 clause “the lane on the composed binary” was ill-posed: the census/lane path is inadmissible on composed identities by the Q80(4) rule. My composed lane build succeeded (tree `0ea9ed5a…`, binary `112aa6cd…`, 5,806 files / 5,797 byte-identical, the nine manifest edits) and both lane tests reddened with `SourceIdentity` (`execution_tests.rs:277`, `:109`, `schedule.rs:96`); the live-tree authenticated lane stands, and the worker's deviation #3 (not rebuilding the lane binary) was correct. A recording correction: the workspace aggregate carried since c79 as “4,371” counts the five `forced_red_probe` instances twice; the log-derived figure is 4,366/2/66.

**Ruling `Q82-verification-and-composed-build`.** (1) The run and its STOP stand. (2) The delivered groups are accepted as recorded. (3) Deviation #2 is not accepted as compliance with the order; deviation #3 is accepted as correct and my composed-binary lane clause is withdrawn. (4) **Q82's verification fails**: the composed identity must build and pass the workspace in both modes, and it does not compile in `rum-jupyter` in either arm — not for sandbox reasons but for a defect. (5) **CM4's implementation line is reopened**; it cannot be complete while the composed corpus fails to compile. (6) The accepted `source_schema` `9dd3687b…` remains the accurate record of the composed **source bytes** as rendered, not a certificate that the corpus builds; the repair decides the schema consequence — a composed-build manifest admission (no source-byte change) leaves the identity and the schema as they are, a rendered text change makes a new identity and requires the mechanical re-emission and 151-cell rebind. (7) **Continuation 83 is ordered**, inventory-first: (i) the complete rendered-reference inventory over all 94 paths against every crate's dependency graph with a zero-unknown statement; (ii) the minimal repair with its mechanical inverse — a declared composed-build manifest admission or a rendered text change, with the schema consequence stated and no hand edit; (iii) the composed workspace in both modes **green** on a loopback-capable host, the `rum-jupyter` render compiled and the nine native tests executing (the grk sandbox cannot bind loopback; the director runs the stage on the host if no worker route can); (iv) on a pass with a changed source identity, the mechanical re-emission and the 151-cell rebind; (v) no composed-binary lane (withdrawn), the live-tree authenticated lane remains; (vi) the standing checks as ordered; stop before any group that cannot fit and report the exact remainder. (8) No weakening and everything sealed stays sealed — c76–c81 and the c82 artifacts with its 16 focused + 2 workspace receipts. (9) The bar is unchanged — **571/4,422, 37/134, 4/133, 0/26**, 3,851 obligations / 97 actors / 129 fields outside; pair `(45,5,false)` Open; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; no campaign, epoch, acceptance, CM5 or new catch; CM5 remains unauthorized. (10) Allocation: c82 charged 944.9538995410257 s focused (16 receipts: 936.9538995410257 + 8.0) and 2,145.5789097879897 s workspace (2 receipts: 2,144.5789097879897 + 1.0) = **3,090.5328093290154 s**; CM4 had 2,144.99609236805 s unspent, so per the 2026-11-19 user ruling a fresh **945.5367169609654 s** is carved from the stage reserve (42,000 → **41,054.46328303904 s**) and recorded; CM4 now **149,445.536716961 s**; the Q82 envelopes' unused remainders (435.04610045897425 s focused, 3,794.4210902120103 s workspace) return to the reserve; the 40,500 s for 743–746 stays held. (11) Routing: continuation 83's route needs the user's decision — `mst` is dry, the Qoder account is walled, and `grk` requires the user's word per run; my host run is the fallback for the loopback-capable groups.

**Chronology.** mst 05:51:38–06:13 (402); qcn 07:13–07:19 (403); grk 07:38:21–08:53 (STOP); my host control 08:55–09:35; composed construction 09:33–09:35; composed workspace 09:35–09:36 (build red); composed lane tree/build 09:36–09:40; composed lane tests 09:40–09:50 (SourceIdentity); disabled jupyter check 09:55.

**Verified by me.** The dispatch, census and report; the composer re-run on a gate copy (four records byte-identical, roots diff-empty); the records' counts and the helper hash; the schema hashes and `production_changes=0`; the scope; the armed hit lines and the disabled cohort summaries; the lane verdicts; the workspace aggregations re-derived from the logs; the composed-tree build failure in both arms; the 94-path reference scan; the composed lane's SourceIdentity rejection; and the exact budget arithmetic.

Next: the continuation-83 route decision; then dispatch and gate as c50–c82.

## Continuation 83 ruling (director) — Q83: the inventory is complete and the build is repaired; the admission is inadmissible and the composed workspace is red

Continuation 83 is gated. Run: `grk` (unit `brief-742-grk-20261120-141027-4175007`, 2026-11-20T14:10:34–15:09, rc 0, `status=success`, 116 turns, $9.420448, refused=0, peak 10,355,527,680 B, oom 0/0; session `bf542f52-…`; census green: 9 tools, no MCP, bypassPermissions inside `rum-worker` strict/network-restricted/.git-denied; 3 git-fence attempts), under the user's second authorization ("grok 再跑一次（推荐）"). Report 50,633 B `b918d086…`, **STOP** (`ComposedWorkspaceNotGreen`), 8 PASS / 3 FAIL / 1 NA, 2 measurements PASS, 3 deviations, 2 disclosures, 0 questions; `report.py check` OK with 92 citations verified.

Delivered. **The reference inventory** (Q82(7)(i)): both arms of the sealed c82 roots scanned — 94 paths, 188 files, root-b identical 94/94, 10 workspace crates, 131 lock packages, 2,430 crate references, 2,422 resolved, **8 unresolved, unknown=0**; the eight rows are `rum_runtime` at `crates/rum-jupyter/src/adapter/worker.rs:359/370/391/394` in both arms — exactly the one edge c82 found. **The repair** (Q82(7)(ii)): a composed-build manifest admission, two lines (`rum-jupyter/Cargo.toml` gains `rum-runtime = { path = "../rum-runtime" }`; `Cargo.lock` gains `"rum-runtime"`), live pre-images `2d157db0…`/`e3c81a52…` unchanged, the inverse byte-exact, schema consequence declared `none`, `source_schema` stays `9dd3687b…`, no table cell edited. Both jupyter arms compile (`cargo check -p rum-jupyter --all-targets`, 11.87 s armed / 2.63 s disabled). The composed tree is the live working tree plus the 94 armed postimages (94/94 byte-identical to the c82 `armed_sha` cells, my own check) plus the two admission lines; nothing else differs. The live tree is the same 23 inherited porcelain entries; `production_changes=0`.

The workspace. `cargo test --workspace` on the armed composed tree, the exact seven skips, 8 threads, the sealed live lane binary `f29e51ff…`, no native skip allowance: default 1,024.201368 s exit 101, `--no-default-features` 615.273687 s exit 101; cargo-level 4,196 passed / **113 failed** / 66 ignored in both modes; my per-binary summary parse confirms the **same 113 names** in both modes. Nine are sandbox loopback panics (on the host `cargo test -p rum-jupyter --lib zeromq::tests::native` is 9/9 green). The rest include one `crate_manifest_keeps_jupyter_on_host_facade` failure, 26 ordinary tests dying at the probe's `result_observation` oracle (`actual complete_work observation`, composed `lib.rs:2616`, 49 panic sites), 31 census `SourceIdentity` panics, 8 `typed_slot_census` reds, one LP01 producer fingerprint (`59913209…` vs `de7f618c…`), 8 missing `UnknownHostPayload` rows (3 probe-module sites, 5 from new call edges), the frontier count 1,241 vs 1,229, the stdout hygiene violation (12 `println!` in the module), the `.frames.insert(` queue-mutation scan, and the native-stack limit (`TerminatedEvaluation["StackLimit"]` vs `done`). The four `g7_continuation_contract` reds are **isolation-sensitive**: they fail identically on the live tree when the module runs alone and passed in the live full suite at c82.

**The finding and the ruling.** The admission is mechanically exact but **not admissible**: `crate_manifest_keeps_jupyter_on_host_facade` (`rum-jupyter/src/tests/mod.rs:85`) asserts the jupyter manifest contains no `rum-runtime` edge, and it was not weakened; a composed instrument cannot satisfy its own suite by violating an architecture invariant the suite enforces. **Q83's verification fails** — the composed workspace is red in both modes, nine reds are sandbox-only, and the rest include deterministic instrument defects: the c80/c81 wiring records and asserts outside a driven run, the module's text violates two corpus invariants, and the wiring's frames move the native-stack boundary. **The Q82(7)(iii) "green" bar is corrected**: the composed suite contains live-source identity/fingerprint audits (the `SourceIdentity` census path per Q80(4), the `typed_slot_census` earned baseline, the LP01 producer fingerprint) that no composed identity can pass; the bar is the **composed-vs-live A/B under identical commands** — the composed red set equals the live red set plus a **declared, pre-enumerated inadmissible set**, with **zero unexplained differences**; every declared test is green on the live control, no test is modified, and every other red is fixed. This corrects my own clause; it weakens nothing.

**Continuation 84 is ordered**: (i) replace the admission with a facade-preserving rendered-text repair — `rum-jupyter` reaches the probe through `rum-host` (a declared supersession or extension with its mechanical inverse; the jupyter manifest stays clean); (ii) make the instrument inert outside driven runs and fix the module's textual violations, with no test change; (iii) re-compose 94 paths × 2 arms into two fresh roots with erasure 94/94 and the per-path reproduction audit; (iv) on the changed identity, the mechanical re-emission of the six-column table and the 151-cell rebind; (v) replay the derived standing inventories on the composed tree and record the exact inventory delta; (vi) the composed-vs-live workspace A/B on the host, both feature modes, the exact seven skips, the nine native tests executing, the inadmissible set enumerated before the run; (vii) the five carrier rows re-fire with the disabled cohorts canonical; no composed-binary lane; (viii) stop before any group that cannot fit and report the exact remainder. No weakening, and everything sealed stays sealed — c76–c82 and the c83 artifacts with its 12 focused + 4 workspace receipts. The bar is unchanged — **571/4,422, 37/134, 4/133, 0/26**, 3,851 obligations / 97 actors / 129 fields outside; pair `(45,5,false)` Open; C2 118 Residual + 0 Lost; 51 owners and five source duties Open; no campaign, epoch, acceptance, CM5 or new catch; CM5 unauthorized.

**Allocation**: c83 charged 52.90756404493004 s focused (12 receipts: 46.90756404493004 + 6.0) + 1,670.237306060997 s workspace (4 receipts: 1,668.237306060997 + 2.0) = **1,723.144870105927 s**. The Q82 envelope remainders return (4,229.467190670935 s), so the stage reserve was **45,283.930473710025 s** before the charge, not the 41,054.46328303904 the worker's carve assumed (a bookkeeping correction; no charge changes); the 7,800 s carve leaves **6,076.855129894073 s** returning, and the stage reserve after the gate is **43,560.7856036041 s** with the 40,500 s for 743–746 covered and 3,060.7856036041 s unallocated; **CM4 is now 151,168.68158706694 s**. Q84's envelopes are **3,000 s focused + 7,200 s workspace**, carved from the stage reserve with the same return rule; any further shortfall gets an explicit carve, never a scope cut.

**Chronology.** grk dispatch 14:10:34; inventory and admission 14:26; both jupyter arms 14:26–14:29; workspace default 14:30–14:46; no-default 14:47–14:59; switches 15:02–15:03; fmt and falsifiers 15:04; report 15:09. My gate: the facade, the nine native tests, the oracle/stack/inventory/census reproductions, fmt and the tree-scope checks.

**Verified by me.** The dispatch, the report and its check; the live pre-image hashes, the two-line admission, the byte-exact inverse and the unchanged live tree; the 94/94 armed-postimage identity of the composed tree with no other tracked difference; the facade failure; the nine native tests green on the host; the oracle panic, the stdout violation, the stack-limit red, the 8 missing inventory rows and the `SourceIdentity` rejection; the identical 113-name set in both modes; the isolation-sensitive g7 reds on both trees; the live fmt (0 diffs); the scope; and the exact budget arithmetic.

Next: the Q84 route decision; then dispatch and gate as c50–c83.
