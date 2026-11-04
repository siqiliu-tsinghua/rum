# Brief 662 — LP06 F design round 8: the boundary scope

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Round 7 reproduced the corrected seven-file shape, kept every semantic record
and every retained bar, and stopped on the D resume row: the gate's enter and
drop are inlined into a 4.6k-instruction hot body, the configured result is
kept live across that body, and the emitted stream is re-allocated by +56 own
instructions per request. Round 8 installs the same scope at the smallest
request boundary that covers each request kind, so the bulk bodies are
emitted exactly as at R8, and pays only the sanctioned cold-load check at the
boundary.

## Goal

Design, and prove with disposable builds under this brief's root, the same
zero-sized execution scope with its gate at the request boundary — the root
evaluation, the resume request and abandonment — so that every bulk
evaluation body is source- and emission-identical to the live tree and the
disabled path pays **only** the sanctioned cold-load check at each boundary
site: zero non-check own-PC deltas in every measured function, no TLS access,
no call and no frame state when unconfigured, no check result live across a
body, the cold request-exhaustion refresh unable to touch TLS when
unconfigured, the admission chain no larger than the red arm, the 29 retained
rows exact, F7 at most 856, every 313-panel positive a gate-site check, every
D native/stack/copy/allocation/layout/lifecycle/timing bar measured, and
every 658-proven semantic record green on the design arm and red on the red
arm, the 40 inherited falsifiers and the four combined paths included. Then
draft the implementation brief in `TEMPLATE.md` shape.

## Context pointers

- `.temp/director/brief-661-gate.md` — the round-7 STOP, the corrected bar,
  the emitted-code evidence and the boundary sites. Read it first.
- `.temp/brief-661/report-design.json` — round-7 evidence that still stands:
  the 29 rows, F7, the admission chain, the native bars, the 313 panel, the
  two source gates, the ownership replay, the census pairs, both workspace
  runs, the 40 inherited and four combined falsifiers. Re-measure it on the
  boundary shape; do not re-derive what it establishes.
- `.temp/brief-661/panel-attribution/disposition.json` and
  `.temp/brief-661/logs/D-DISPOSITION-2.log` — the four positive entries,
  the 14-check/56-other split and the STOP line.
- `.temp/brief-661/probe/` — `red-src` (the R8 red arm), `design-src` (the
  round-7 shape), the instrument copies and the D instruments; copy before
  use. The round-7 shape is the semantics-proven answer, not the target.
- `.temp/brief-658/report-design.json` — the proven semantics
  (`q5_falsifier_matrix`, `q5_helper_bodies`, `q5_record_logs`), the entry
  inventory (`q1_entry_inventory`, `q1_registrations`,
  `q1_out_of_contract`), the context/match rule (`q2_context`,
  `q3_match_rule`).
- `.temp/brief-659/probe/` — the 29-label receipts, F7 and the native
  objdumps. `.temp/director/probe-658-okform/` — the R8 build.
  `.temp/brief-655/supplied/` — the supplied 29-label instrument and build
  recipe. `.temp/brief-653/` — the D contract, native bars and the 313
  program set (read-only).
- Live source: `profiling/logical/context.rs`, `segments.rs`,
  `logical/mod.rs` (`Call::start`, `open_phase`), `resolution.rs`
  (`Session::evaluate` at 315, `evaluate_at_root` at 330),
  `session/session_requests.rs` (`resume_eval_request_with_input` at 283,
  `resume_eval_request_from_checkpoint` at 315), `suspension/core.rs`
  (`resume_suspended_slot`), `suspension/core/abandon.rs`
  (`abandon_pending_suspension` at 5, `drain_abandonment_worklist` at 21),
  `suspension/core/fault.rs:126`, `session/session_api.rs`
  (`accept_cancellable_request`), `profiling/config.rs` (the cold-load
  `AtomicU8` gate), `profiling/timing.rs` (the six adopted phase starts),
  and the 16 host entry groups in `session_eval.rs`, `session_requests.rs`,
  `session_api.rs`.

## Established by the director (reproduce before doubting)

1. **Round 7's STOP is real and its cause is emitted code.** The disabled
   entry check is `lea; movzbl; cmp $1; jne; movzbl; cmp $2; je`; the
   configured result is held in `%r14b` across the body; the session
   register moves r12 -> rbp and spill slots shift; the function's emitted
   stream is re-allocated throughout (+58 mov, +1 movabs, -1 lea, +5 je,
   -1 jne, +4 cmp, +4 movzbl = +70 executed per request, of which 14 are
   check PCs and 56 are not). The four positive entries: resume-s9 +70,
   lhs-head-value `evaluate_at_root` +42 (three entries x 14, zero other)
   and its resume +55, resume-do-chain +32, resume-fold +38.
2. **The corrected bar.** The sanctioned cold-load check is the only
   permitted disabled-path cost. For every measured function the executed
   own delta must consist solely of the gate-site check PCs; every other
   own PC's executed count must be unchanged; the emitted bytes outside the
   gate site must be identical after relocation; no TLS access, no call and
   no frame slot on the disabled path; no value from the check may live
   across a body. A positive that is not a gate-site check is a defect.
3. **The gate belongs at the boundary.** The three bulk bodies
   (`evaluate_at_root`, `resume_eval_request_from_checkpoint`,
   `drain_abandonment_worklist`) must stay at their live source and live
   emission. The natural boundaries are `Session::evaluate`'s depth-0
   branch, `resume_eval_request_with_input` and
   `abandon_pending_suspension`; the nested abandon callers inside an
   evaluation (e.g. `builtins_input.rs:234`, `eval_call_prepare.rs:142`) and
   the fault-path drain (`fault.rs:126`) must be enumerated and either
   covered by an active scope or shown to need one.
4. **The cold rollover may not touch TLS when unconfigured.** The terminal
   exhaustion path must be guarded by the same cold-load check; unconfigured
   means no scope exists, so the guard is semantic. This closes round 7's
   Q4-TLS FAIL without an ordinary-path cost.
5. **Round 7's other results stand and are re-measured**: 29/29 pairs exact;
   F7 600/632; admission chain 77 red -> 63 design; native root 904,
   abandonment 856, request resume 2184, evaluator 1208, while 744, compound
   648, Catch 1000, forwarded 24 x4; 313/313 correct; the four combined
   falsifiers green and the red arm red; both source gates; ownership 48/48
   retired=0; census definition passes, 1076 -> 1128 with 52 added identity
   rows (11 unchanged `budget.rs`, 35 unchanged `epoch.rs`, 3 `context.rs`,
   3 `Call`), removed 0, all paired, no hand edit, the count assertion 1067
   the F3 frontier move; workspace 4133/1/61 in both configurations with
   only the untouched census count red; required-profile host 5/5 each.
6. **Not yet measured**: the D stack, copy, allocation, layout, lifecycle
   and timing bars stopped after the round-7 STOP and are this round's work.
   The four While rows remain 664's package.
7. **The unpassed list is not a waiver**: R7's bare dispatch out of contract
   and the completion/forwarded literal gap stay unpassed and named.
8. **The admission body is already right**: the cold terminal
   `accept_after_request_exhaustion`, the cold outlined
   `refresh_execution_control`, the shared `reserve_request_identity`, chain
   63 under red. Keep it; only add the configured guard in front of the TLS
   walk.

## Questions to answer

- **Q1 — coverage under the boundary sites.** Map the 16 route groups to the
  boundary that covers each: root evaluation, text/parsed/script/file/request
  entries, request resume, typed-slot resume, inspection/resume,
  cancellation, abandonment (host and nested), snapshot restore, evaluation
  cache, random state wrapper, host facade and request-control exhaustion.
  For every free start that is in contract, show the control it will find;
  for every route that begins an evaluation, show its boundary. Name every
  new call edge. `segments::current_owner` stays absent; no borrowed
  `Session` escapes its call; no raw pointer, no `unsafe`.
- **Q2 — the disabled path, corrected bar.** Measure both draws: per
  boundary site, the executed check PCs and nothing else; the three bulk
  bodies source- and emission-identical to live (disassembly diff per
  function after relocation, not a label aggregate); the 29 own and 29
  incoming slopes exactly equal to pre-A1; F7 at most 856; the 313 panel
  with every positive a gate-site check and every non-check PC unchanged;
  the admission chain not larger than red; the native prologues at root
  904, abandonment 856, request resume 2184, evaluator 1208, compound 648,
  Catch 1000, four forwarded slow bodies 24, release wrappers absent. A
  non-check own delta, a body byte that moved, a TLS access or a call on the
  disabled path is a defect, full stop.
- **Q3 — the semantics on the boundary shape.** Re-run the 658 matrix with
  its exact records, red on the red arm and green on the design arm: the 40
  inherited falsifiers and the four combined paths — deferred enable,
  reset + suspend + handoff, deferred + reset + suspend + handoff, and the
  nested second-session resume with inner owner and outer restoration. The
  deferred case starts its evaluation while the profile is configured but
  inactive: the boundary gate must push there or the post-activation free
  start has no control. Record every event line.
- **Q4 — the scope's limits.** Every read and write of the TLS top under the
  boundary sites; which request kinds can be nested; why no in-contract
  free start can see another session's control or a stale entry; the
  same-session rollover: the guarded cold path, its cost and its propagation
  across nested boundaries; the `Unwind`/panic path: what the Drop does,
  what remains after an unwind that crosses a boundary, and whether any
  in-contract free start can observe it. Do not assert the R7 boundary.
- **Q5 — shape and bars.** The calibrated surface unchanged (ok-form E2, six
  phase-only starts, borrowed completions, E1 only at `try_apply_function`,
  four sessionful forwarded starts); the typed-resume macro and checked
  adapter unchanged; the two unchanged source gates green with the entry
  bodies in place; the ownership replay green with no HEAD row retired and
  every new design row paired; the census definition check passing, the
  design-tree count recorded for the source stage and the 52 added identity
  rows explained; the **whole** D campaign this round — native, stack, copy,
  allocation, layout, lifecycle, timing — measured or explicitly unpassed
  with the reason; both workspace configurations with the stale-cell
  ordering artifacts classified; `cargo fmt --check` green on the exact
  reproduced source.
- **Q6 — the bar.** If the boundary cannot hold a gate-site check without
  another measured cost, STOP with the minimal mechanism, the exact cost and
  the counter-evidence — the ruling is the director's. Do not invent a
  waiver, do not lower a bar, do not reintroduce a guard local, a token
  field, a token `Drop`, a macro/checked-adapter change or a dispatcher
  guard, and do not cover the R7 boundary.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-662/`: copy the round-7 red and design trees and the D
instruments; build the red arm (R8) and the boundary design arm at one
common source path with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked); measure
with the supplied runner serially, alone, on CPU 2, every `RUM_PROFILE_*`
cleared. Supplied and prior roots are read-only and never executed in place
(`.temp/brief-648/`, `.temp/brief-653/`, `.temp/brief-654/`,
`.temp/brief-655/`, `.temp/brief-656/`, `.temp/brief-657/`,
`.temp/brief-658/`, `.temp/brief-659/`, `.temp/brief-660/`,
`.temp/brief-661/`, `.temp/director/calib-655/`,
`.temp/director/verify-655/`, `.temp/director/probe-657/`,
`.temp/director/probe-658-okform/`, `.temp/director/probe-659-gen/`,
`.temp/director/probe-660/`).

**Do not** change the typed-resume macro or checked adapter, hand-edit an
inventory or census count, add a per-operation dispatcher guard, add a guard
local or token field, make the completions or forwarded starts session-free,
insert the scope inside any bulk body, or change the calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-662/report-design.json` with:

- the R8 baseline reproduced (build receipt and hash, 29 own/incoming pairs,
  F7, ownership verdicts, the five red-arm failures);
- the boundary-scope design: type, TLS stack, every boundary enter/drop edge,
  the free-start rule, the guarded cold rollover, every new call edge, and
  the source/emission identity of the three bulk bodies;
- the entry coverage with `path:line`, the nested abandon callers, the
  fault-path drain and the R7 out-of-contract list;
- the design arm's receipts: the 29 rows, the 313 panel (every positive a
  gate-site check, every other PC unchanged), F7, every D native and stack
  bar plus the copy, allocation, layout, lifecycle and timing bars, the
  admission chain, the per-site check costs, the two source-proof gates, the
  ownership replay and the census verdict with the added-row explanation;
- the falsifier matrix with exact records, red on the red arm and green on
  the design arm, including the four combined-path falsifiers and the
  deferred case;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list keeping R7, the completion/forwarded literal
  gap, and every bar not measured (including the four While rows if the
  package is not yet created).

Run `python3 -I scripts/report.py check 662` and return its summary and every
compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, live source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- One variable per probe; a single Callgrind pair never decides a
  seed-dependent quantity; state the distribution when a quantity needs one.
  A design that pays for the context with a re-allocated body, new
  default-path work, a wider public API, a compile-time gate, a
  macro-grammar change or a lowered bar is not a candidate.
- Every claim carries a citation the checker can resolve.
