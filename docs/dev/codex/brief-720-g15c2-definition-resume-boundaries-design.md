# Brief 720 — definition-resume logical boundaries: publish the deferred RHS record, sample the declining stage before its consumer, and order the Return recovery (design)

**Shape: design-only. `Scope: NONE` — no source change.** The director's
gate of brief 623's continuation 7 accepted three `Lost` rows and the C7
repair draft. This round turns that draft into a reviewed, executable
plan. You write no code outside `.temp/brief-720/`.

## Goal

Make this true, with evidence, on the current tree: for every
definition-application resume path — down-values, sub-values, up-values —
a valid keyed native observation made on the resumed path equals the
queued path's observation, including

1. **the definition RHS record.** A `down_value_rhs_eval`,
   `sub_value_rhs_eval` or `up_value_rhs_eval` definition record must not
   stay terminal-open when the resumed RHS work completes, errors, returns
   or is abandoned; its ten counters must match the queued path's record
   for the same key.
2. **the declining stage's Option.** A sub-/up-value guard that declines
   must let its own stage sample the actual `None` before the dispatch
   consumer reconstructs a later `Expr`; the stage's `Some`/`None`
   counters must match the queued path.
3. **the repeated-RHS Return.** A second inner RHS suspension that returns
   through the carried recovery owner must close the definition record
   with the post-recovery `Some`, not with `Return`.

The guest language result, evaluation count and order, effects, trace
ordering, request freshness, cancellation/abandonment ownership, public
carriers and the disabled path's cost do not change.

## The measured defects (accepted evidence, 2026-11-10)

Continuation 7 of brief 623 found 147 valid, source-reached bound native
differences, 49 per row on P0267 (`down_values.rs:521`), P0312
(`sub_values.rs:365`) and P0323 (`up_values.rs:451`), every one with
`semantic_same=true`:

- 144 leave the definition record terminal-open: resumed
  `[0,0,0,0,0,0,0,0,0,1]` against queued `[1,1,0,1,0,0,0,0,0,0]`; the
  two-pause case (guard then RHS, or RHS then a second RHS pause) is the
  trigger — one pause alone is SAME.
- 3 repeated-RHS Return cases close with `[1,0,1,0,0,1,0,0,0,0]` against
  the queued `[1,1,0,1,0,0,0,0,0,0]` (Twice ×
  `return_boundary.last-pause-return`).
- 2 minimized declining-stage cases (P0312/P0323 `false_guard`) sample
  `[2,2,0,1,1,0,0,0,0,0]` where queued samples `[2,2,0,0,2,0,0,0,0,0]`;
  the down-value decline control passes, consistent with its explicit
  before-consumer exception.

The minimized set is 21 cases (8 DIFF, 13 SAME); every case keeps
`semantic_same=true` and real == disabled. The director reran the
`optional-real` campaign and reproduced all 470 records, including the
147 native differences. The evidence is frozen at `.temp/g15-c2/c7/`:
`port/loss-category-audit.json`, `port/loss-minimized-witnesses.json`,
`port/loss-return-witnesses.json`, `port/loss-evidence.json`,
`port/fixtures/`, and the draft at
`.temp/g15-c2/c7/draft-repair-definition-resume-logical-boundaries-complete.md`.
Never overwrite that scene.

## Context pointers

- `crates/rum-runtime/src/runtime/definitions/application/down_values.rs`:
  `finish_suspended_down_value_application` (:418), the adopted RHS rule
  record open (:473), the RHS evaluation (:478) and its recording (:479),
  the resume owner (:511-524). The brief-719 repaired neighbour:
  `down_values/default_fallback.rs:29-45` (flush on a second
  `InputRequired`).
- `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs`:
  the analogous adopted RHS start (:322), evaluation (:327), recording
  (:328), typed resume (:356-367).
- `crates/rum-runtime/src/runtime/definitions/application/up_values.rs`:
  the analogous adopted RHS start (:315), evaluation (:320), recording
  (:321), typed resume (:442-453).
- `crates/rum-runtime/src/runtime/definitions/application/rhs.rs:109-121`
  (Return consumption) and `:146-158` (the typed RHS owner mapping the
  recovered result to `Some`).
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs:509-521`
  (`complete_definition_rule`: `InputRequired` defers the record),
  `logical/completion.rs:12-18` (`defer_definition`) and `:22-45`
  (`flush_definition_boundaries`), `logical/mod.rs:1272-1282`
  (`publish_stashed_phases_slow` flushes deferred definitions).
- `crates/rum-runtime/src/runtime/eval_core/call.rs:66` (dispatch
  appends an Option-consuming frame) and `:489` (its `None` mapping);
  `profiling/logical/mod.rs:1526` (down-values inserted before that
  consumer), `profiling/logical/queue_api.rs:8` (the validated
  before-consumer insertion API), and the generic after-consumer
  placement for sub-/up-values.
- The repaired neighboring path's gate is brief 719's; do not regress it.

## Scope

`Scope: NONE — no source change.` Your work lives in
`.temp/brief-720/`: a byte copy of the live tree and probe-only
diagnostic additions made to that copy, never to the live tree.

## What the plan must contain

1. **A closed inventory** of every resumed definition-application typed
   frontier (down-, sub-, up-values, the default fallback, and the shared
   prepared-definition/rule paths), each with `path:line`, the owning
   frame, the actual result carrier, the queue frontier at that point,
   and whether a deferred definition record can exist there. Include
   every caller of every function whose duty would move.
2. **Per defect, a precise insertion point and signature**: the copied
   experiment's exact edit, why it is the actual suspension frontier (not
   merely a convenient return), what it does for `Ok(Some)`,
   `Ok(None)`, `InputRequired`, `Error`, `Return`, cancellation and
   abandonment, and its ordering relative to the recovery owner. A blind
   flush of every typed return is not a design; name what publishes what,
   when, and to whose queue.
3. **The class duty.** Every site that publishes deferred definition
   boundaries or samples an Option before a dispatch consumer: classify
   each as covered by this repair, already correct with the proof, or a
   STOP with its reason. No new call edge is left unreachable.
4. **The falsifiers.** The eight minimized native differences as base-red
   / candidate-green on the copied experiment; the thirteen controls stay
   SAME; all 147 bound witnesses retained; the declining-stage cases and
   the three Return witnesses; and a fresh focused witness per mechanism
   that the implementation must keep. The gate must also catch the
   omission or misplacement of each new publication/sampling tail
   (independently omit it and watch the observer go red).
5. **The disabled-path cost bar.** The three typed frontiers and the
   phase helpers are hot. Measure instructions per call with Callgrind on
   the disabled path before and after each copied edit at identical call
   counts (at least three draws, ranges as the floor); pin the bar and
   name the env-gated cold path that keeps the disabled caller unchanged.
   A measurable default-path increase is a STOP, not a recorded
   deviation.
6. **Risks and open questions**, with the exact question when the
   director must decide something.
7. **The drafted implementation brief(s)** in `docs/dev/codex/TEMPLATE.md`
   shape: staged so each mechanism has its own base-red/candidate-green
   gate; the copied experiment first, the live edit only after the
   director's review.

## Reproduce first

On a fresh byte copy of the live tree with a probe-only diagnostic
package (the C7 `c2-probe` pattern), reproduce the three witnesses from
`.temp/g15-c2/c7/port/loss-minimized-witnesses.json` and
`loss-return-witnesses.json` with the exact ten-counter records, the
declining-stage records and the language observations. A rerun that does
not reproduce them is a STOP with the raw log; never redefine the
witness.

## Hard rules

- No git in any form, no network, no writes outside `.temp/brief-720/`.
- Never edit `crates/`, `docs/`, `scripts/`, tests or inventories on the
  live tree; scratch copies only. Preserve `.temp/g15-c2/c7/` untouched.
- Rust 1.85.0, offline, locked; one cargo process at a time at eight
  build jobs; native diagnostics serial; every command bounded.
- Every claim carries `path:line` or a raw receipt. Copied log lines for
  every verdict.
- Two consecutive executed bounded experiments with no new evidence
  stop the round; failed launches and pure source reviews do not count.
- No `unsafe`, no dependency change, no public API change, no foreign
  engine comparison.

## Deliverable

A `rum-report/1` JSON at `.temp/brief-720/report.json` (written with
`scripts/report.py`, checked before finish) containing: the plan with
the closed inventory table, the three mechanism designs with exact
signatures, the falsifier matrix, the cost rows with raw captures, the
risks and open questions, and the drafted implementation brief(s). Also
leave every raw log, fixture and copied-experiment diff under
`.temp/brief-720/` and name them in the report.

## Verification of this design round

- The witnesses reproduce on the copy with exact raw records; a negative
  control (a deliberately wrong read of one field) must fail the same
  check.
- Each proposed insertion point is exercised in the copy: red before,
  green after (or the measured reason it cannot be green yet), quoted
  from raw logs.
- The disabled-path rows read equal at identical call counts on the
  copied base, with the candidate's range pinned.
- The report cites every claim; `report.py check` passes; every drafted
  implementation brief is TEMPLATE-shaped and names its own gate.

Stop and report; the director reviews the plan, settles the open
questions and writes the implementation brief.
