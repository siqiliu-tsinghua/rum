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
