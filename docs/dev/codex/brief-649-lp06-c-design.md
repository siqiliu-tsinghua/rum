# Brief 649 — LP06 C design: the disabled-path repair after B

**Design-only. `Scope: NONE — no source change.`** Brief 648 measured the
pre-A1 tree (`654f2ce5`) against the post-A2 tree (`1a4c44f3`) and found
real disabled-path work at the converted sites. This brief produces the
repair plan and the implementation-brief drafts. It writes no code.

## Goal

Produce a step-by-step design that removes the B-measured disabled-path
costs at the LP06 converted sites while preserving every A1/A2 semantic
guarantee and the enabled diagnostic lifecycle. The plan must carry
`path:line` evidence for each cause, name the exact edit(s) per step, show
the disabled path returning to the base instruction and frame shape, show
the enabled path still recording, and draft the follow-up implementation
briefs in `TEMPLATE.md` shape.

## Measured failure (evidence; do not re-derive from scratch)

- `.temp/brief-648/report-b.json` — B report, `check: OK, 573 citations
  verified`. My verdict and gate note:
  `.temp/director/brief-648-b-verdict.md`,
  `.temp/director/gate-648b/gate.json`.
- **Builtin recorder**: `Session::record_profile_builtin_timing_adopted_boundary`
  is outlined in the candidate CLI (body entry `0x729500`) and costs
  **19 instructions per incoming call with profiling off**: prologue (3
  pushes + 3 moves), `LEXICAL_PROFILE_ENABLED` load + `cmp $1/je` +
  `cmp $2/je`, the token `test/je`, epilogue (3 pops + `ret`). Callgrind own
  slope 19, call slope 1. PC table:
  `.temp/brief-648/instruments/attribution-final-4/valueq-undefined-symbol-definitions-8000.json`.
- **22 ordinary + 2 resume labels** retain positive iteration-proportional
  rum work (`attribution-final-4/summary.json`). The unpassed functions
  cluster as: the recorder (most labels); `<Expr as PartialEq>::eq`
  (+4.5 calls/iteration at unchanged per-call cost 33.93); 
  `eval_control_data_builtin` (3 labels); `try_apply_function` (2 labels).
- **CLI frames grow** (`lp06-prologues/comparison.json`):
  `try_dispatch_builtin_call` 856->888 (+32) and 856->872 (+16); the Catch
  finish path (`resume_...` under
  `runtime/kernel_data/control_data/nonlocal`) 1016->1032 (+16) in CLI and
  stack-release, 792->808 (+16) in stack-debug; `runtime_path_timing_start`
  24->40 (+16) in CLI and stack-release. `finish_builtin_dispatch` itself
  shrinks 184->40/72/0, which is not a license.
- **Passing B gates**: copies (768), allocations (776; ordinary slopes
  exactly unchanged), canonical stack (528 probes/44 fits per arm), LP06
  stack (216/18), layouts (563/563 frames, 638 runtime + 810 stdlib shared
  entries unchanged), live chains (352), lifecycle (six controls + 14 LP06
  tests/21 second-pause cases). Timing is non-deciding (Q5).

## Pointers

- B's brief and amendments: `docs/dev/codex/brief-648-lp06-b-disabled-path.md`.
  A1/A2: `docs/dev/codex/brief-646-lp06-a1-loops-functions.md`,
  `docs/dev/codex/brief-647-lp06-a2-builtin-catch.md`.
- Source: `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
  (`try_dispatch_builtin_call`, `finish_builtin_dispatch:223`),
  `crates/rum-runtime/src/runtime/profiling/control_builtin.rs`,
  `.../profiling/control_timing.rs`, `.../profiling/config.rs`,
  `.../profiling/logical/*`, the Catch sites under
  `.../kernel_data/control_data/nonlocal.rs` and the `resume_typed_or_suspend!`
  macro.
- Read-only B evidence roots: `.temp/brief-648/instruments/` (report,
  attribution, prologue comparison, PC tables, raw receipts). Never modify
  or execute a prior campaign script in place.

## Required diagnosis (each claim with `path:line`)

1. **The recorder.** Trace `try_dispatch_builtin_call` ->
   `builtin_timing_start_adopted` -> `finish_builtin_dispatch` ->
   `record_profile_builtin_timing_adopted_boundary`. Why is the boundary
   function outlined, and why is it called when the token is None? What is
   the `Option<Option<AdoptedBuiltinTimingToken>>` for, and which caller
   depends on the outer `None` vs `Some(None)` distinction? Name a source
   shape that makes the disabled path a single inlined branch (or no call)
   while the enabled path still records. Consider hoisting the token test,
   inlining, and removing the now-redundant flag check.
2. **The caller frame.** Why does `try_dispatch_builtin_call` reserve
   +32/+16 bytes? Which locals/temporaries live across the call? How do the
   nested `Option` and the `freeze` closure contribute? Propose the shape
   that keeps the disabled frame at the base size, without moving work into
   a new outlined call.
3. **The Catch finish +16.** Trace the incoming token through
   `finish_builtin_dispatch`/the Catch finish site, `Policy::Incoming`, the
   frozen outcome, `call.pause()`, and the nonlocal resume path. Why +16 in
   all three binary kinds, and how to carry the token without growing the
   frame?
4. **`runtime_path_timing_start` +16.** The A1 wrapper: why did the CLI and
   stack-release frame grow, and what is the disabled shape?
5. **`Expr::eq`.** Find the new call site/edge the candidate added (call
   slope +4.5/iteration, per-call cost unchanged) and how to avoid it when
   profiling is disabled.
6. **`eval_control_data_builtin` and `try_apply_function`.** The new edges,
   their callers and branches; the disabled-path shape.
7. **Per finding:** the enabled-path cost, the disabled-path target (zero or
   byte-identical to base), the A1/A2 invariants that must not change
   (semantics, lifecycle records, InputRequired pause, all Catch forms), the
   falsifier that goes red before the fix and green after, and the B panels
   that would catch a regression.

## Deliverable

A plan with a call-site/ripple inventory; per-step edits with `path:line`;
the disabled shape as code (types, signatures, call structure); the
enabled-still-records proof; stack/ABI accounting; the test and B-panel
re-measurement plan (which B panels to re-run on the repaired tree, with
the standing exclusions); risks; open questions. Draft the implementation
briefs in `TEMPLATE.md` shape, one per stage, each with its `must not
change` list and its calibrated falsifier. Do not write code.

## Verification (you run)

Read-only analysis only: source reads, greps, the B receipts. No cargo
build, test, measurement or source edit is required or allowed. A claim that
would need a build is an open question for the implementation stage.

## Hard rules

- No source edit, no version-control command, no network, no dispatch, no
  `unsafe`.
- All new scratch and the report under `.temp/brief-649/`; `.temp/brief-648/`
  is read-only evidence.
- English; every claim carries `path:line`; no invented numbers. A B number
  you quote is copied from the cited receipt, not rounded.

## Report format

Write a `rum-report/1` report at `.temp/brief-649/report-design.json`:
the diagnosis with `path:line`, the per-finding mechanism, the options with
trade-offs, the recommended design, the drafted implementation briefs, the
re-measurement plan, the risks and open questions. Check it with
`scripts/report.py check` (without its version-control option) and return
its summary and the full compliance verdict lines.
