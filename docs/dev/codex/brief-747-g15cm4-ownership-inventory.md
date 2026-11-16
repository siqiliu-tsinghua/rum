# Brief 747 — G15-CM 4: the complete ownership inventory for the 151-row generator (design-only)

Worker: `cdx`. Dispatched by `scripts/dispatch-brief.sh 747`.

## Goal

Produce the **complete per-row ownership and transport inventory** for the
CM4 generator over all 151 frozen own rows, plus the composition plan and
the drafted implementation briefs, so that the implementation can be
dispatched **once** instead of discovering one ownership frontier per run.
This is a design-only brief: `Scope: NONE` — no maintained source, inventory
or schema change. The deliverable is an entry-pinned, machine-readable
matrix with a **zero-unknown-rows** statement, a decision on Q24 for every
dependent row, and a named `PortBlocked` class wherever a binding cannot be
expressed under the standing constraints.

The immediate question the inventory must settle is Q24 from brief 742's
fifteenth run: the Q23 carrier keys a token by
`SuspendedEvaluation.generation`, but `SuspensionSlot::next_generation` is
slot-local (`Session::empty` builds a fresh slot per session,
`model/session.rs:115`), so two distinct slots both install generation 1
and the unrelated append scope consumes the token while the intended one
loses its hit. The inventory must select exactly one of: (1) an
authenticated single-slot execution domain proved from the actual runner
sources, with the equal-generation cross-slot control explicitly ruled
outside that domain and the falsifier retained in its original form; (2) a
source-bound owner namespace that disambiguates actual slots or suspensions
at both the arm and the publication using existing objects on existing
paths (for example the request-interrupt control that
`begin_input_suspension` already registers against the queue), with no new
field, accessor, allocation, overlay path or production surface and with
the cross-slot falsifier passing; or (3) the J class and every dependent
row named `PortBlocked`, with the bar unchanged. No global uniqueness may
be claimed from a per-slot counter, and the required falsifier may not be
replaced.

## Context pointers

- Read `AGENTS.md`; `docs/dev/codex/TEMPLATE.md`; brief 741 (the Q9
  generation method: 151-row/ten-class taxonomy, five-step generation
  rule, 38-row class observation policy, 510-case attempt plan); brief 742
  in full, especially the Continuation 1–15 rulings (Q10–Q24) — they are
  the recipe ladder to be applied, not re-litigated.
- Brief 623's `### Continuation 90–101` sections carry the run-by-run
  state and the standing campaign invariants.
- The 90-file overlay set:
  `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/overlays.tsv`
  (single source of truth for the instrumentable surface; every recipe
  must stay inside it, and every maintained source must be read from the
  live tree).
- The current partial generator and its audits:
  `crates/rum-stdlib/tests/suspension_oracle/mutations/` (the uncommitted
  23-entry partial CM4 test layer) and the scratch audits under
  `.temp/brief-742/continuation-*/` (read-only raw material; reuse the
  entry-pinned audit form: live source + entry hashes + extracted-statement
  controls).
- The frozen pins: the 18 caught + 4 stop rows per mode with their
  historical signatures (brief 740's c7 campaign logs), plus every later
  pin named by Q10–Q24; the 151 source/tail/destructor joins, the 90
  preimages / 180 postimages and the 5,399 maintained files must
  authenticate unchanged.

## Scope

- Files you MAY modify: `NONE — design only`. All output under
  `.temp/brief-747/` (scratch logs, generators, matrix files, report).
- Files you MUST NOT touch: everything under `crates/`, `docs/`,
  `scripts/`, `Cargo.toml`, `Cargo.lock`, `.git`, and any `.temp/` content
  outside `.temp/brief-747/`. You may READ anything.
- No network. No `git`. No campaign, epoch, acceptance, mutant build or
  inventory rewrite. No production change of any kind: the live sources
  and the working tree must be byte-identical to entry (the 23-entry
  partial CM4 test layer stays exactly as found).
- If you believe a binding requires a new production field, accessor,
  allocation or overlay path, that row is `PortBlocked` and named — never
  an exception.

## Deliverable

1. **The matrix** (machine-readable, e.g. `.temp/brief-747/inventory.tsv`
   + a JSON index): one row per frozen own row (151), with, for each
   pinned arm and each pinned hit and each generated recipe form:
   - row id, class, run family (default/no-default), side
     (queued/unqueued), and the frozen signature references;
   - the pinned arm event and hit event: site (`path:line`), phase,
     prompt, selector, observer/predicate;
   - the **actual executing object** and its construction identity (the
     runtime object, not a name or a label);
   - the **actual source edge** connecting them, as a chain of existing
     call edges with `path:line`, including the retirements, drops,
     borrows and lifetime bounds that the recipe must survive;
   - the transport recipe (one of Q10–Q23, or a newly specified one with
     the same form: issue site, carry, consume site, invalidation);
   - the isolation falsifier(s) and expected outcomes (`unrelated 0 /
     intended 1`, wrong generation/row/episode/path, queued zero), and
     the extracted-statement controls that must execute the audited
     statements rather than helper simulations;
   - the preserved nodes and the full-file AST restoration claim;
   - the shared-file composition: which other rows' recipes touch the
     same file, statement or call expression (e.g. `call.rs`'s prepared
     and profile forms, `core.rs`'s multiple sites, `profile_queue.rs`'s
     append/publication), and the exact order they compose in;
   - status: `expressible` with the recipe name, or
     `PortBlocked::<name>` with the missing edge/fact named.
2. **The Q24 decision** as a separate, argued section: the option chosen
   (single-slot domain, owner namespace, or named PortBlocked), the
   entry-pinned evidence, the retained cross-slot falsifier and its
   result, and the list of every row whose recipe depends on it.
3. **The generator plan**: the exact generated file set (paths), the
   renderer inputs and outputs, the deterministic double-generation
   procedure (two roots byte-identical), the mechanical `source_schema`
   transition (`f9604852…` to the new value, with its derivation), and
   the standing replay order (fingerprints, production acceptance both
   modes, ownership exactness, rejection census, LP01, typed-slot census)
   before the workspace gates.
4. **Zero-unknown-rows statement**: every one of the 151 rows classified;
   no row normalized; any row that cannot be classified is itself a named
   STOP with the reason.
5. **Drafted implementation briefs** in `TEMPLATE.md` shape: (a) the
   generator implementation (per-class recipes, helper, audits, the
   complete 151-plan bar), (b) the schema emission and authentication,
   (c) the replay + workspace gate stage. These are drafts; the director
   rewrites and commits them.

## Verification (MANDATORY — you run it)

Design-only, so the evidence is the deliverable and its reproducibility:

- A script under `.temp/brief-747/` regenerates the matrix from the live
  sources and the entry hashes, and the run pastes the reproduced table's
  checksum twice (two roots byte-identical).
- Every `path:line` citation resolves in the live tree; every quoted
  statement is the live statement (entry-pinned hashes recorded).
- The failed/attempted instrumentation runs and their corrections are
  retained, with the verdict lines; nothing is deleted or relabelled.
- `git status --porcelain` at exit is identical to entry (the 23-entry
  partial tree); `source_schema` unchanged; `production_changes=0`.
- Report as `.temp/brief-747/report.json` in `rum-report/1`, with STOP,
  checks, citations and the budget block; paste the key verdict lines
  verbatim.

## Budget and stop rules

- One focused envelope of **1,500 s** (allocated from the stage's
  69,000 s; no workspace envelope is granted and no workspace-wide test
  run is authorized). Charge measured serial command seconds plus the
  0.5 s bookkeeping and 60 s preparation/final-reporting reserve; stop
  before exceeding the envelope.
- One cargo command at a time, offline/locked, jobs 8, inside the 16G
  scope, every long command through `scripts/run-bounded.py`.
- Stop-loss keyed on progress: if two consecutive attempts classify no
  new row, stop and report the classified set and the remainder. Do not
  keep re-deriving the same frontier; a row that resists classification
  is `PortBlocked` with the exact missing fact, and the report names it.
- No implementation follows this brief. If the inventory cannot be
  completed, checkpoint `.temp/brief-747/PROGRESS.md` (done, next action,
  tree state) and say exactly what remains.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any maintained source, inventory or schema; your output
  is scratch and a report.
- Testing Policy is a hard line (`AGENTS.md`): no third-party engine
  comparisons, no committed foreign outputs.
- No new `unsafe` anywhere you touch.
- Minimal footprint: no drive-by edits, no reformatting, nothing outside
  `.temp/brief-747/`.

## Continuation 1 ruling (director) — the first run accepted; Q24 option 2; the 13-row frontier (Q25)

The first run is accepted as the complete ownership inventory. It classified
all 151 frozen own rows in `inventory.tsv`/`inventory.json` — 138 expressible
recipes, 13 named `PortBlocked`, 0 unknown, 0 normalized — over ten classes
(A 2, E 2, F 30, G 1, H 1, J 3, N 4, R 34, T 73, U 1) and 79 operation
subtypes; it authenticated the 151 source/tail/destructor joins, the 90 live
preimages and 180 exact postimages, the 22 historical signature rows, and
`source_schema` `f9604852…` unchanged; it left all 5,400 maintained files and
the 23-entry partial CM4 test layer byte-identical (`production_changes=0`,
`git_invoked=false`, hash evidence in `entry-hashes.json`, `entry-state.json`
and `immutable.log`), produced 273 scratch deliverables under
`.temp/brief-747/` with 1,767 live citations verified and two byte-identical
roots (11 files), and stopped with no schema emitted. Run: `cdx` (unit
`brief-747-cdx-20261117-000409-1850502`, rc 0, peak 1,214,103,552 B, oom 0 /
oom_kill 0; log `.temp/logs/cdx_brief_747.log`, 3,170,095 B, sha256
`c0ae9e15…`; report STOP, 273 changes (273 added), 0 disclosures, 2
deviations, 0 questions, 10 PASS / 4 FLAGGED / 1 NA, 175 citations verified;
159.82 s charged — 94.32 s measured plus 0.5 s bookkeeping, the 60 s
preparation/final-reporting reserve and 5 s conservative short-IO — leaving
1,340.18 s focused in this envelope). Both deviations are accepted: no git is
invoked because the standing rules forbid it and the maintained-file hashes
carry the byte-identity evidence; no new schema digest is emitted because a
complete 151-plan renderer does not exist and prefilling a digest would be a
fabrication. The four FLAGGED items are settled as stated: the 13 named
binding blocks are the stop; the original Q23 identity red (14 passed / 1
failed per J row) is the retained falsifier; the namespace fixture is
extracted-statement design evidence only, not a runtime or E acceptance; and
the failed instruments (function lookup, matrix TypeError, empty
boundary-site assertion, the resumed audit's 5→13 blocks) are retained, not
relabelled.

Q24 is accepted as option 2 — the existing registered owner namespace. The J
class keeps the slot-local generation only as a component and never claims
global uniqueness; the namespace is the actual queue's existing root, epoch
and request-control Arc allocation identities, already on the installation
and publication paths (`register_owner` at `profile_queue.rs:744`; the
publication lookup by `Arc::ptr_eq(root)` and `Arc::ptr_eq(epoch)` at
`:619-627`; the control taken at `:626` and passed to
`take_deferred_definition_boundary` at `:631-634`; retirement at `:792`,
`:807`, `:847`). The instrumentation key is `(row, opaque-program,
pair-episode, own-path, occurrence, generation, root-identity, epoch-identity,
control-identity)`. The capture is the actual installed suspension's
generation inside its actual install/registration scope; the annotation is
inside the actual `ProfileQueue::append` scope for that queue
(`profile_queue.rs:654`, the publication and its retained omission first at
`:610`); the consumption is only a full-key match at that publication, at
most once, invalidated at registry clear, control replacement/unregister,
root unregister/drop, the next install, abandonment, and the owning
append/run boundary. No new production field, accessor, allocation, overlay
path or production surface is authorized for it, and the helper stores
identity, not a new strong owner. The retained falsifiers stand: the original
same-generation collision remains a red artifact (14/1 per row), the candidate
extracted fixture passes 18/18 in armed, disabled and admitted forms with
unrelated 0 / intended 1, wrong generation, next install, abandon, unprofiled
install, swallowed/other error, publication outside append, second
consumption, queued zero, registry clear and owner retirement. The fixture is
not credited as runtime evidence; the implementation must reproduce the
red/green pair in the emitted overlay, and the E rows P1470/P1471 need their
own extracted and emitted controls through the fixed-point
return/publication path (`suspend_evaluation_return` /
`insert_evaluation_return` → `publish_evaluation_pass_after_return`), not
J's `complete_work`/append confirmation.

Q25 — the 13-row frontier. The first run's stop (13 named `PortBlocked`) is
not accepted as a final disposition and the 151-plan bar is not reduced. The
blocks are four classes of missing identity evidence, each with the actual
frame allocation in scope only inside a raw resume file outside the retained
90-file surface (`runtime/debug_tools/trace/resume_frames.rs`;
`runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs`;
`strings/patterns/search/guarded/replace/resume_frames.rs`;
`strings/patterns/search/rules/resume_frames.rs`), where the raw resume
clones or borrows state into the hit helper and the allocation identity does
not cross. This continuation must resolve each class with entry-pinned
evidence, choosing exactly one of:

1. an in-surface recipe — including a newly specified transport form with the
   same form as Q10–Q23 (issue site, carry, consume site, invalidation) and
   the retained `unrelated 0 / intended 1` falsifier, proven by an
   extracted-statement fixture that executes the live statements and includes
   the reentrancy/nesting and second-consumption controls;
2. a minimal instrumentable-surface extension, specified exactly — which of
   the four raw resume files, the exact added statements at
   construction/arm/resume/hit, the schema and table consequences (90→94
   rows, 180→188 postimages, the `source_schema` derivation and the 151
   `source_schema` cells), entry-pinned preimages, full-file AST restoration,
   both cfg arms and the retained falsifiers — specified only, not
   implemented;
3. a named `PortBlocked::<name>` with the exact missing fact and
   `universal_impossibility_claim: false`.

The continuation must also decide whether a newly specified transport form
(for example a scoped-call token with a stack-discipline guard over the exact
helper call) is admissible under the standing no-ambient-state rule; if
admissible, prove it with the extracted fixture including nested/reentrant
calls; if not, name the reason. The Q24 option-2 decision stands; no pinned
arm, hit, phase, selector, observer, coverage, source key or tail changes; no
row is normalized or dropped; the 138-row subset is not shipped.

Continuation 1 is design-only (`Scope: NONE`), under
`.temp/brief-747/continuation-1/`, with its report at
`.temp/brief-747/continuation-1/report.json`; every existing artifact under
`.temp/brief-747/` stays byte-identical and the accepted first-run report is
not overwritten. Budget: the remaining 1,340.18 s of this brief's 1,500 s
focused envelope; no workspace envelope; one cargo command at a time,
offline/locked at `-j8`, inside the 16G scope through
`scripts/run-bounded.py`; stop-loss keyed on progress (two consecutive
attempts that resolve no new class stop and report the four class results);
no network, no git, no campaign, epoch, acceptance, production or inventory
change; no new production field, accessor, allocation or overlay implemented.
After its gate I rule Q25 and the implementation funding; nothing in this
continuation authorizes implementation.

## Continuation 2 ruling (director) — Q25 adopted: option 2, the four-file extension, and the implementation order

Continuation 1 is accepted. It resolved all four frontier classes as option 2
— a specified extension of the instrumentable surface by exactly four raw
resume files — and changed no covered source. Run: `cdx` (unit
`brief-747-cdx-20261117-010058-1864907`, rc 0, peak 386,174,976 B, oom 0 /
oom_kill 0; log `.temp/logs/cdx_brief_747.log`, 1,190,702 B, sha256
`fd6f309a…`; report STOP, 107 changes (107 added), 0 disclosures, 0
deviations, 0 questions, 11 PASS / 3 FLAGGED / 1 NA, 101 citations verified;
81.45 s charged — 10.95 s measured plus 0.5 s bookkeeping, 5 s conservative
short-IO, the 60 s preparation/final-reporting reserve and the 5 s
remaining-report reserve — leaving 1,258.73 s unspent in the design
envelope). Scope holds: the 23-entry partial CM4 tree and the 273 accepted
first-run artifacts are byte-identical (changed=0, added=0), the 5,400
maintained files and `source_schema` are unchanged, `production_changes=0`,
`git_invoked=false`; the two 11-file design roots are byte-identical (table
sha256 `a35dd8b1…`); 2,751 live citations authenticated; all 151 rows remain
classified with zero unknown and zero normalized, the 138 expressible / 13
blocked status preserved pending implementation proofs. The three FLAGGED
items are settled as stated: the scoped-call-only remedy is not admitted
(a stack guard separates invocations but supplies no authenticated join from
the constructor's Arc identity into a helper receiving a cloned or borrowed
state; a nearest-scope token would be the ambient ownership answer prohibited
by Q17/Q19); the extension controls and full-file restoration are specified,
not executed; and the failed extraction attempt is retained with its
correction (two classes before, four after; no zero-progress pair).

Q25 is ruled and adopted as option 2 for all four classes. The instrumentable
surface extends by exactly four raw resume files —
`crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs` (P0214,
P0216), `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs`
(P1070), `crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs`
(P2348, P2354) and `crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs`
(P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418) — from 90 to 94
paths and from 180 to 188 postimages, with all 151 `source_schema` cells
rebound only after all 151 plans pass. The maintained source files stay
unchanged; the extension files are instrumented copies in the generated
overlay, no fifth overlay is authorized, and no 138-row subset may ship. The
owner-qualified call handoff is the transport: at the existing
`insert_suspension_frame` allocation (`suspension/core.rs:561`, the one
`Arc::new(frame)` and the one `push_back`) a weak binding records the actual
`Arc<F>`'s allocation identity, never a strong clone; each new raw method
reads the already executing `self: Arc<Self>` and issues a local guard; the
exact original helper call receives a direct-call grant, claimed into a local
callee activation before any guest code or reentry; only the local one-use
owner predicate may answer the unchanged pinned omission, preserving its
emission, selector, phase, operation, decoder, error/None branches and
non-target finalizers. The key includes the selected row, opaque actual
program, pair episode, own activation path, actual producer occurrence,
concrete frame type and the Weak allocation identity of the actual Arc; type
names, row names and edge labels never replace it. Eligibility is invalidated
on consumption, every raw resume exit including `Err` and unwind, an exact
call-scope exit without claim, abandonment or a dead Weak, a new pair/episode,
and run boundaries; nested empty entries mask outer entries and may neither
consume nor clear an outer owner; the helper adds no strong owner, no payload
lifetime extension and no production `Drop` change. The transport is a
candidate extension contract, not yet admitted: the implementation must
execute its per-class extracted-statement controls and emitted-overlay
controls, in both cfg arms, over the real audited allocation, raw
clones/borrows/decoders, call and target statements; helper-only simulations
and stack models are not credited. The 19 control families stand, including
unrelated 0 / intended 1, nested unarmed, two armed nested owners, direct
helper reentry, second consumption, retired Weak, wrong key components, wrong
type/allocation, P2348's guard-immediate and resuspension paths, P1070's loop
reentry, queued zero and disabled exactness. The P2348 path refinement is
accepted: the actual guard frame and both output paths are specified with the
frozen events, source keys and tails unchanged. Q24 option 2 stands for
P0267/P0312/P0323/P1470/P1471, and the accepted Q24 artifacts are inherited
by hash (the three original J reds remain 14 passed / 1 failed; the candidate
fixtures remain 18/18 armed/disabled/admitted); the E rows still need their
own fixed-point controls.

Budget: this ruling closes the 747 design envelope — its 1,258.73 s unspent
remainder is charged — and charges the 60 s preparation reserve from the
implementation envelope. The implementation envelope is 9,000 s focused,
allocated from the stage's 69,000 s reserve; CM4's committed share becomes
28,500 s and 743–746 retain 40,500 s. One workspace envelope of 6,000 s is
assigned to the implementation pass; the second stays held for the
verification stage, which uses brief 742's remaining 1,440 s focused. The
implementation is brief 742 continuation 16, committed before dispatch; after
its gate I rule the result and dispatch the verification. Nothing here
authorizes a campaign, epoch, new catch, acceptance, CM5 or a bar change.
