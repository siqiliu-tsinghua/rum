# Brief 724 — G15-C2: design the prepared-argument stage completion (design-only)

Scope: NONE — no source change. The deliverable is a reviewable plan, the
copied insertion experiments behind it, and TEMPLATE-shaped drafts of the
implementation briefs. The director writes the implementation brief from the
accepted design.

## Goal

Design the repair of the P0492 diagnostic loss: the logical
`call.stage.prepare_arguments` completion must be published at the actual
prepared-argument boundary — after the prepared carrier is installed and
normalized, before the later call dispatch can Return or Throw — so that the
ordinary and resumed paths record the same logical preparation outcome. The
ordinary path's correct `None` publication and every real producer
Return/Throw/InputRequired must be preserved. All guest behavior and the
installed fixed-point (brief 723), definition-resume (brief 721/719) and
P0248 repairs stay byte-unchanged on the live tree.

## Context pointers

- Read `AGENTS.md` and `docs/dev/codex/TEMPLATE.md`; the report shape is
  `rum-report/1` via `scripts/report.py`.
- Brief 623 continuation 9's report and scene under `.temp/g15-c2/c9/`:
  `arguments-loss.json` (the nine whole-record differences, seven controls,
  exact live-source packets, the own caught omission), `arguments-frontiers.json`
  (the actual prepared carrier followed by the consumer error),
  `arguments-differences.json`, `arguments-pause-attribution.json`, and
  `DRAFT-prepared-stage-repair.md`.
- `crates/rum-runtime/src/runtime/eval_core/call.rs` — the ordinary prepare
  stage's success publication (the adopted-none record), the suspended path's
  `PreparedDispatchFrame` stash, `resume_prepared_call_dispatch_frame`, the
  parallel resumed dispatcher P0496 and the typed consumer P0505.
- `crates/rum-runtime/src/runtime/profiling/logical/completion.rs` — the
  prepared-completion helpers, which today have no live caller; establish why
  and whether the current queue ownership API supports them.
- P0492's minimal witness: `Return[Input["c9arg"]]`, answer `73`. Queued
  `call.stage.prepare_arguments` is `[2,2,0,0,2,0,0,0,0,0]`; resumed is
  `[2,1,1,0,1,1,0,0,0,0]`. Replacing Return with Throw moves the
  consumer-error column. A surrounding Catch does not repair the phase.

## Scope

NONE. No live source, inventory, test or doc change; no git, no commit. Work
only on byte copies under a fresh `.temp/` root. The accepted C9, C8, C7 and
brief-623 scenes are immutable. The live tree is the accepted brief-723
source and must stay byte-identical.

## Deliverables

1. **Inventory.** The complete call-site, carrier and publication inventory
   for both preparation routes (ordinary and resumed), both helper routes,
   typed/legacy joins, repeated pauses, faults and every existing consumer of
   `ProfileBoundary`, with `path:line` evidence. Reach P0496 with a real
   earlier-stage continuation; never fabricate a prepared object or a
   suspended result. Name every function whose duty moves and every caller.
2. **Copied insertion experiments.** In copied trees, place candidate
   publications at the actual prepared-argument boundary and measure, for
   each candidate: the nine witnesses, the seven controls, complete ten-field
   phase records and complete language observations; whole-record equality of
   the resumed path against the queued path; and the adjacent fixed-point
   panel (90/12/470/2/1/4 records, 84 controls, the 20/14 omissions), the
   definition panel and the P0248/native controls unchanged. Preserve the
   exact request, epoch and generation; preserve real producer
   Return/Throw/InputRequired and the diagnostic-cap, cancellation and
   resource precedence. A candidate that moves one of those is refused, not
   adjusted.
3. **Falsifiers.** A calibrated own-row omission at each proposed insertion
   point that affects the named phase at the actual carrier boundary and is
   armed/hit/caught against the required tuple, plus independent falsifiers
   for before/after placement, repeated-pause publication and wrong-owner
   selection. The omitted work must be actual publication work; an
   identity-preserving mutation is not a falsifier.
4. **Cost.** The disabled-path instruction gate on the caller and the new
   helpers, with the standing capture protocol: profiling off, one codegen
   unit, one stable build path, fresh captures, the caller row at unchanged
   call counts, and the new symbols absent from the disabled captures. The
   existing fixed-point publication must not move.
5. **Drafts.** TEMPLATE-shaped implementation brief drafts for the accepted
   design, staged (shape and falsifiers first, the measurement campaign
   second), carrying the full ripple inventory, the standing-inventory replay
   order, the disabled-cost gate and the focused/runtime/workspace gates.

## Verification

Every claim carries a raw log path, a source/fixture/binary hash and whole
records. Report failed and invalid attempts with their reasons; never change
a receipt's meaning. One Cargo command at a time at `-j8`; copied builds in
private target directories; timed and instruction measurements serial and
alone; commands through `scripts/run-bounded.py` with bounds derived from
prior receipts. Standing inventories are replayed before the first acceptance
suite. The two-concrete-family zero-progress stop-loss applies.

## Hard rules and stop-loss

- No git, no network, no new `unsafe`, no live edit, no inventory or test
  change, no weakened assertion, no fabricated carrier.
- Keep sampling out of the disabled path: no default-path cost, no
  diagnostics work on the non-profiled path.
- Preserve the protected scenes byte-for-byte; nothing outside the repo and
  the brief's `.temp/` root.
- Stop for the director gate with the completed plan, the experiment matrix
  and the drafts; do not implement.

## Report

`REPORT.md` plus the `rum-report/1` envelope at the brief's `.temp/` root:
inventory, per-candidate matrix, falsifier calibration, cost receipts,
drafted briefs, and the exact residual obligations. C2 and S10 cannot close
ahead of this repair.

## Director ruling (2026-11-11) — run 1 STOP accepted; the P0496 reach is amended to the source invariant; continuation 1 finishes the design

Run 1 (log `.temp/logs/cdx_brief_724.log`, 31,587 lines, sha256
`ec8bfff8…`, memory-scope peak 2.87 GB, oom 0, rc=0) stopped at
deliverable 1's mandatory P0496 reach before any insertion experiment,
falsifier or cost capture, and left two conditional drafts. **The STOP is
well taken and accepted.** No live file moved: 2,715 live files replay 0
missing / 0 extra / 0 changed against `entry-hashes.json`, git is clean at
`ebcc8475`, and the four protected scenes replay 95,156 files 0 changed
(C9 33,294, C8 36,460, C7 25,251, brief-623 151). The fresh C9 baseline is
exact: 16 records, the nine witnesses each changing exactly one phase
(`Runtime { phase: "call.stage.prepare_arguments", source: None }`) with
queued/resumed tuples equal to C9's canonical differences, the seven
controls whole-list equal to C9, and every language observation equal. The
two real route families ran 9 attempts / 8 valid / 1 retained invalid with
zero P0496 hits; the head family enters the suspended dispatcher at stage 1
with prepared state, and the definition family takes the stage-0 prepared
skip. `report.py check` reads 80 citations. The five deviations and the
one invalid fixture stay recorded, not gated.

**My own source reading confirms the invariant behind G0.**
`build_call_dispatch_plan` (`eval_dispatch/mod.rs:51-66`) returns a
SpecialForm-only plan or pushes PrepareArguments first; the suspension
resumes pass `stage_index + 1` (`eval_core/call.rs:479`, `:499`); the
external LHS entry passes index 0 with `CallDispatchState::from_prepared`
(`definitions/definition_lhs_preparation.rs:230-234`), which `call.rs:350`
skips by `is_prepared`. The P0496 preparation branch (`call.rs:359`)
cannot execute from any of the three production callers, and the type
funnel makes the class duty structural: both prepared-suspension helpers
(`call.rs:26`, `:46`) construct the shared `PreparedDispatchFrame`, whose
sole resume is `resume_prepared_call_dispatch_frame` (`call.rs:462`), the
P0505 consumer's call. A single insertion at that resume covers both
construction sites; the dead branch keeps its existing success
publication, is not edited, and is never exercised with a fabricated plan,
carrier or result.

**G0 is resolved by the first option.** Deliverable 1's sentence "Reach
P0496 with a real earlier-stage continuation; never fabricate a prepared
object or a suspended result." is amended to: "P0496's preparation branch
is structurally unreachable from the current plan construction — a source
invariant established by the three-caller closure, never by a finite probe
count; never exercise it with a synthetic plan, carrier or result. Record
the shared-frame funnel and the single resume insertion as the class
discharge." The P0496 ledger row is not relabelled here; the C2 settlement
round owns its disposition, with this source proof as the evidence.

**Continuation 1** (this brief, forked from session
`01a1c77e-38b0-7f61-9181-e27806c9b71e`) completes deliverables 2-5 on
byte copies under the same Scope NONE and hard rules: the copied insertion
experiments at the actual prepared-argument boundary, the calibrated
falsifiers, the disabled-path instruction gate, and the TEMPLATE-shaped
drafts. The run-1 baseline and route evidence stand and are not rerun.
Rulings the continuation must carry:

1. The completion fires only on a real `Ok(PreparedCallArguments)` resumed
   outcome, after `install_prepared_call_arguments` installs and normalizes
   the carrier and before `resume_suspended_call_with_dispatch_plan` can
   Return or Throw; producer Return/Throw and the queued publication
   (`call.rs:277-279`) keep their current, correct paths; repeated pauses
   leave the boundary open until the real prepared boundary and complete
   it exactly once.
2. The retained boundary must be selected by exact queue ownership —
   request control, logical generation, epoch and record — never by a
   global phase-name scan; an independent wrong-owner falsifier is
   required, alongside before/after placement and repeated-pause
   falsifiers. An omission mutant must omit actual publication work.
3. Disabled path: the caller row exact at unchanged call counts, and the
   new symbols absent from every disabled capture. The 719-proven static
   separation is the standing route for enabled-only work; an ungated flag
   check added to the resume body is a STOP unless its disabled rows read
   exact. No duplicated evaluator body for a diagnostic call.
4. Any public TypedFrame trait or driver-order change is an explicit
   G-question with its full ripple, never a default. Preserve the
   fixed-point publication (`profile_queue.rs:368`), the definition
   publication (`:608`) and the P0248/native repairs; the adjacent panels
   (90/12/470/2/1/4, 84 controls, 20/14 omissions, definition, P0248 and
   native) stay unchanged.
5. Stop for the director gate with the experiment matrix, falsifier
   calibration, cost receipts and drafts. The two-concrete-family
   zero-progress stop applies.

## Director ruling (2026-11-11) — continuation 1 accepted: family2 is the shape, G1 and G2 are ruled, and brief 725 implements it

Run 2 (log `.temp/logs/cdx_brief_724.log`, 36,174 lines, sha256
`8e53de44…`, memory-scope peak 3.95 GB, oom 0, rc=0) completed the copied
design deliverables and stopped at G1/G2. **My gate:** live tree 2,715
files 0 missing / 0 extra / 0 changed against the continuation entry; the
five protected scenes replay 132,998 files 0 changed (C9 33,294, C8
36,460, C7 25,251, brief-623 151, brief-724 37,842). The 16 C9 records:
all 16 resumed/queued whole lists equal; exactly the nine witnesses differ
from run 1, and the omission mutant reproduces run 1's 16 records
whole-record exactly; the seven controls are unchanged. The matrix:
90/12/fresh2/nested1/definition470 (468 valid) have zero whole-record
changes; `extension4` has exactly two changed records — the two B722
extension duplicates — with only the prepare_arguments phase changed and
the queued arm and language byte-equal. The fixed-point panel reads 104
instrument-equal, 84 controls equal, P1470 20 and P1471 14 omissions exact
to C9; P0248 36 whole records equal to a fresh current-base run (the
historical 719 delta is debug symbol ids only — re-verified by
normalization); native 60 exact. The five falsifiers' raw event order
holds: installed < publication < dispatch; omit leaves the record open
after the actual producer pause; before/after move the publication and are
caught by the recorded order even with equal totals; wrong-owner closes
record 200 and leaves record 230 open with the same epoch, control and
generation; two real pauses yield one bind and one completion. My own
independent Callgrind parse of the six definition captures reads
self-sum == summary on every capture and reproduces every required row and
all 13 differing rows of the submitted table exactly; candidate2 and
force2 are one SHA (`a7c37947…`).

**G1 ruled: permit exactly the two extension repairs.** The extension
panel's two changed records are the same two witnesses as
`C9.arguments.B722.shape.surrounding-return` and
`C9.arguments.B722.shape.return-effects`; the P0492 defect is what put
them in the extension panel, so a repair that left them defective would be
wrong. The bar is amended, not waived: every adjacent record stays
whole-record exact except those two, whose only permitted change is the
`call.stage.prepare_arguments` phase moving to the queued value with the
queued arm, every other phase and the whole language observation
unchanged — verified phase-confined. The remaining two extension records
(one retained invalid, one exact) stay byte-equal.

**G2 ruled: the nine interner rows are the standing hash-seed class —
recorded, not excluded, not gated.** They are the three `rum-core`
interer functions (`Symbol::system`, `intern_symbol_text`, `intern_text`)
on all three panels, with identical call counts. The same row moves below
base on the arguments panel and above on the definition panel, so no
uniform added per-call work can explain it. The identical binary
(candidate2 == force2, one SHA) spans 16,878 Ir across four draws on the
definition `Symbol::system` row; the closest same-binary draw sits 199 Ir
above base's maximum, against that 16,878 spread, and the candidate-only
range is 4,899 above it. The only runtime rows that move are two
decreases (the cold suspension helper −252/−312 Ir at 21/26 calls, the
flag getter −21/−26 calls). The ordinary dispatch body, the unreachable
preparation branch, the fixed-point publication and the definition
publication are byte-exact. The implementation's cost gate re-runs the
same protocol and must reproduce this class or STOP; no new exclusion, no
universal no-cost claim, and no causality claim beyond the recorded
evidence.

**Family2 is accepted as the implementation shape:** six files (five
modified plus the new `profile_queue/prepared.rs`), the enabled-only
`ProfiledPreparedDispatchFrame` wrapper over the existing ordinary frame,
one shared const-generic resume body, the queue-owned binding of the exact
boundary to the just-inserted witnessed consumer cell, and the completion
that reads only that resuming queue's first generational cell while
validating epoch, request-control Arc, logical generation, record and
OptionalExpr policy. No public TypedFrame or driver change, no phase-name
or active-stack search, no evaluator-body duplication; the family1
const-generic-frame refusal on the +1 instruction/call neighbor is
accepted and recorded. Recorded, not gated: the extra-cancel self-check
that also fails on base; the missing-binary and builder-lock attempts; the
P0248 historical harness id delta; the probe-builder scratch adapter; the
initial seal ordering artifact.

Next: brief 725 implements this shape in two stages, and P0496's ledger
disposition stays with the C2 settlement round.
