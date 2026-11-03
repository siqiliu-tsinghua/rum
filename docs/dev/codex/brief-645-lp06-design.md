# Brief 645 — LP06 design: builtins, loops, Catch, Function and status

**Status: DESIGN-ONLY. `Scope: NONE — no source change.`** Director-written
from the brief-624 draft; the draft is raw material and its line numbers may
be stale. This brief re-probes the routes at HEAD and drafts the
implementation brief(s) that I will rewrite and dispatch.

## Goal

LP05 closed the assignment line: the phrase record engine now carries the
evaluator, dispatch, argument, definition, matcher and assignment phases
(briefs 637-644; the last repair is `a51ae7ac`). LP06 adopts the remaining
non-module phase families, exactly as
`.temp/brief-624/inventory/routes.tsv` names them:

- `builtin-handler` — the builtin handler's own outcome, not today's
  boolean-only finish;
- `while-test`, `while-body`, `compound-item` — loop and Compound item
  phases finishing at the produced `Expr` before the consumer changes
  phase or consumes `Break`/`Continue`;
- `catch-body`, `catch-total`, `catch-finish` — the Catch body completion
  before `Throw` handling, the total span across resolution, tag match,
  handler and bypass, and the `catch.resume_after_body` record retaining
  the frozen incoming outcome;
- `function-evaluate`, `status-evaluate` — the function and status value
  phases finishing before `finish_function_application` absorbs `Return`;
  a status projection pause must start its value phase on the resumed
  projection path without losing projection-error propagation;
- `Return`/`Throw` bodies stay synchronous: no retained phase field, and
  their existing counts are preserved.

Nothing about evaluation may change. The task is to produce the inventory
and the plan, with a design precise enough that the implementation brief
has no invented specifics left to discover.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md` (pitfalls; standing inventory
  duty), `docs/dev/profiling.md`.
- `.temp/brief-624/DESIGN.md` — the contract, especially §"Contract"
  (8-96) and §"Disabled-path gate and measurements" (265-328).
- `.temp/brief-624/inventory/`: `routes.tsv` (the route table above),
  `path-sites.tsv` (phase expressions, can-span-pause, finish evidence,
  classification evidence), `builtin-sites.tsv`, `other-counters.tsv`,
  `expanded-paths.tsv`, `tls-storage.tsv`. These are the boundary
  inventory, not an implemented design.
- The raw draft `.temp/brief-624/drafts/LP06-builtins-control-functions.md`
  (its Scope and Deliverable are the starting list; verify every citation).
- The LP05 precedent, as the shape to copy: `docs/dev/codex/brief-642-*`
  (staged A1/A2, the adopted starter, the `PhaseCompletion` carrier) and
  `docs/dev/codex/brief-644-*` (the disabled-path repair: the adopted
  recorders destructure `Some(token)` in the inline gate so the caller
  never stages the 56-byte aggregate; measured 108 -> 94 own Ir/call).
- The current sites, verified at HEAD `78df58f3` before writing anything:
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:185` (the builtin
  start and its boolean finish), `.../kernel_data/control_data/
  branching_loops.rs:155,161,179` and `branching_loops/resume_frames.rs`,
  `.../kernel_data/control_data/nonlocal.rs:13,35,81,93` and the resume
  paths below, `.../scoping/callable.rs:62,85,130` (a local duplicate of
  the outcome classifier), `.../scoping/callable/status_application.rs:
  33,42`, and `crates/rum-runtime/src/runtime/profiling/timing.rs`.
- The oracle: `crates/rum-stdlib/tests/suspension_oracle/` —
  `fixtures/logical_profile/mod.rs`, `support/logical_profile.rs`,
  `manifest.rs`, and `inventory/logical-profile-{required,bindings,
  losses.json}`. It has no builtin/loop/Catch/Function cells today; LP06
  adds them.

## What the design must answer

For every route above, with `path:line` evidence at HEAD:

1. **Site and carrier.** The start site, the exact consumer boundary, the
   finish evidence, and the carrier type the route already uses for its
   remainder. Say which of these can span a pause today and where the
   retained state lives (`resume_frames.rs`, continuation enum, frame).
2. **Record contract.** The new `PhaseKey` variant and its site string;
   the outcome set (`Ok`/`Some`/`None`/`Return`/`Throw`/`InputRequired`/
   `Error`) and the exact `PhaseCompletion` implementation per carrier;
   opens, completes, transfers and the pause boundary. One record per
   logical invocation; a paused record is transferred, never double-counted
   and never reopened.
3. **The builtin outcome.** Today's finish is boolean-only. Name the
   handler result shape and where the actual outcome is available, and how
   the marker is attached before `map(Some)` on both the unchecked and the
   memory-growth-check routes, waiting through handler continuation and
   finalizers but not through caller stage work.
4. **Catch's frozen outcome.** `catch.resume_after_body` must retain the
   incoming outcome at the point the draft names (`:121` in the raw draft;
   re-locate it) and complete the actual lifecycle exactly once across the
   body, handler and bypass routes. P0817/P0818 stay as they are.
5. **Status projection.** The projection pause must open the value phase
   on the resumed path, with the projection error still propagating; name
   the frame and the exact reopen rule.
6. **Disabled path.** Every new recorder must be a cold-load `AtomicU8`
   gate with no per-call token aggregate on the disabled path (brief-644's
   rule). State the expected added cost per site and how the campaign will
   measure it.
7. **Oracle cells.** The fixture stubs and assertions that exercise each
   new cell, with their expected records (counts, outcomes, pauses) and
   the loss-ledger consequences. Name the cells that must stay SAME.
8. **Falsifiers.** For each route, the red observation that shows the
   record is missing or misplaced on the unfixed tree, and how to reach
   it. A second pause must be genuine: a route where an early `Throw`
   prevents the planned second pause is not a receipt.
9. **Staging.** Whether LP06 fits one brief or must be staged (LP05 needed
   A1 and A2). If staged, draft the stages so each can be gated alone,
   with the campaign deferred to the last stage.
10. **Risks and open questions.** Every place where the raw draft may be
    stale, wrong or under-specified; anything that would need a new
    carrier, a new mechanism, a changed guest-visible behaviour, or an
    inventory identity move is a STOP for me, not a decision for you.

## Deliverable

A plan in this file's shape, in `.temp/brief-645/plan.md`, plus draft
implementation brief(s) in `docs/dev/codex/TEMPLATE.md` shape under
`.temp/brief-645/drafts/`. Every claim carries `path:line` evidence
verified at HEAD. Do not write source code, do not touch the tree, do not
run the campaign.

Also produce the mechanical inventory of the current sites in a table the
implementation brief can quote: route, phase string, start site, finish
site, carrier, can-span-pause, current recorder, and the existing legacy
duplicate of the outcome classifier if there is one.

## Verification

None of the campaign gates run in a design-only brief. Verify instead:

- every cited `path:line` exists at HEAD and says what you claim (quote
  the line);
- the route table is complete: every `routes.tsv` row not yet adopted by
  LP03-LP05 is covered by either LP06 or LP07 (`module-*` is LP07's), and
  nothing is silently dropped;
- the oracle cells you propose are reachable from the existing fixture
  harness (say how);
- the drafted briefs' acceptance commands all exist and are bounded.

## Hard rules

- No version-control command in any form; no source change; no network.
- All scratch under `.temp/brief-645/`, never `/tmp`; earlier brief
  evidence stays read-only.
- Paste real output; say plainly what did not run.
- The plan is raw material for me. Where you are unsure, write a question,
  not an assumption.
