# Brief 663 — LP06 F design round 9: the native entries, the resume gate's cost

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Round 8 moved the scope to the request boundary: the boundary set missed the
direct private resume, five in-contract lp06 falsifiers went red, and even a
clean boundary paid 7 check PCs plus 2 save/restore. Round 9 restores the
round-7 native entries — coverage by construction — and demands the one thing
round 7 could not show: the resume entry's gate costs its check sequence and
nothing else.

## Goal

Design, and prove with disposable builds under this brief's root, the
zero-sized execution scope at the three native entries of the round-7 shape
(`evaluate_at_root`, `resume_suspended_slot`, the non-recursive
`drain_abandonment_worklist`), with **no body re-allocation**: each entry's
disabled-path own delta consists solely of the gate's check sequence — the
load/compare/branch at the enter and at the drop plus the address
materialization that performs the second check — while every other own PC's
executed count is unchanged and the emitted bytes outside the gate site are
identical after relocation. The cold request-exhaustion refresh must not
touch TLS when unconfigured. The admission chain stays no larger than red,
the 29 rows exact, F7 at most 856, every 313-panel positive a gate-site
check, every D native/stack/copy/allocation/layout/lifecycle/timing bar
measured, and every 658-proven semantic record green on the design arm and
red on the red arm — the 40 inherited falsifiers, the four combined paths,
and lp06 36/36. Then draft the implementation brief in `TEMPLATE.md` shape.

## Context pointers

- `.temp/director/brief-662-gate.md` — the round-8 STOP, the incomplete
  boundary, the 7 + 2 measure and the ruling. Read it first.
- `.temp/director/brief-661-gate.md` — the round-7 STOP, the emitted-code
  evidence and the corrected bar.
- `.temp/brief-661/report-design.json` — the round-7 native-entry shape and
  its evidence: 29 rows, F7, admission 77 -> 63, the native bars, 313,
  source gates, ownership, census, workspace, the semantics. Re-measure on
  the round-9 shape; do not re-derive.
- `.temp/brief-661/probe/` — `red-src` (R8), `design-src` (the round-7
  native-entry shape, the base for this round), the instrument copies and
  the D instruments; copy before use.
- `.temp/brief-662/report-design.json` and
  `.temp/brief-662/logs/BOUNDARY-ATTRIBUTION-1.log` — the round-8 boundary
  measurements (7 check + 2 save at `abandon_pending_suspension`), the five
  lp06 reds and the direct-private-resume cause. The round-8 shape is
  falsified; use its evidence, not its tree.
- `.temp/brief-658/report-design.json` — the proven semantics
  (`q5_falsifier_matrix`, `q5_helper_bodies`, `q5_record_logs`), the entry
  inventory and the context/match rule.
- `.temp/brief-659/probe/` — the 29-label receipts, F7 and the native
  objdumps. `.temp/brief-655/supplied/` — the supplied instrument and build
  recipe. `.temp/brief-653/` — the D contract and the 313 program set
  (read-only).
- Live source: `profiling/logical/context.rs`, `logical/mod.rs`
  (`Call::start`, `start_from_active`, `open_phase_from_active`,
  `push_phase`), `resolution.rs` (`evaluate_at_root` at 330),
  `suspension/core.rs` (`resume_suspended_evaluation` at 626,
  `resume_suspended_slot` at 632), `suspension/core/abandon.rs`
  (`drain_abandonment_worklist` at 21), `suspension/core/fault.rs:126`,
  `session/session_api.rs` (`accept_cancellable_request`),
  `profiling/config.rs`, `profiling/timing.rs`, and the 16 host entry groups.

## Established by the director (reproduce before doubting)

1. **The native entry is the only complete layer.** `resume_suspended_slot`
   is reached from `resume_eval_request_from_checkpoint` (session_requests.rs
   329) and from the direct private `resume_suspended_evaluation`
   (core.rs:626); a boundary at the host request misses the direct route,
   and a gate on the direct route is inlined into the 4.6k checkpoint body.
   Round 8's five lp06 reds are that hole. The round-7 placement is the
   standard.
2. **The bar's unit is the check sequence.** At each entry the executed own
   delta may be only: the `lea`/`movzbl`/`cmp`/`jne` at the enter, the same
   at the drop, and the address materialization between them (a callee-saved
   register or a repeated `lea`). Round 8 measured that floor at 7 + 2.
   Anything else — a re-allocated body, a spill, a call, a TLS access, a new
   frame slot — is a defect.
3. **Two of the three entries already meet it.** Round 7, both draws:
   `evaluate_at_root` +14 (check PCs only; it is `#[cold]`) and
   `drain_abandonment_worklist` +14 (check PCs only). Only
   `resume_eval_request_from_checkpoint` fails: 4693 -> 4763, 14 check PCs
   plus 56 others (mov +58, movabs +1, lea -3, je +1, jne -1), with the
   session register moving r12 -> rbp and the spill slots shifting.
4. **The mechanism is emitted code.** At the entry the disabled path runs
   `lea; movzbl; cmp $1; jne; movzbl; cmp $2; je`; the check's result and
   address are live across the body for the exit check, and the inlined
   region's register allocation is re-done. The design space for round 9 is
   therefore the check's own form, how the second check materializes its
   address, whether anything from the first check stays live, the guard's
   drop shape and the push/pop call form — the worker's choice, measured.
5. **The cold rollover must be guarded**: `refresh_execution_control` may
   not touch TLS when unconfigured; when unconfigured no scope exists.
6. **Round 7's semantics are the acceptance standard**: 40 literal
   falsifiers, the four combined paths, lp06 36/36. LP655 and LP656 are the
   documented R7 bare-dispatch probes, red in both arms; they stay named and
   are not repaired. Round 8's five lp06 reds must not survive.
7. **Everything round 7 established is re-measured**: 29/29 pairs, F7
   632/600, admission 77 -> 63, native root 904/abandonment 856/request
   resume 2184/evaluator 1208/while 744/compound 648/Catch 1000/forwarded
   24 x4, 313/313, both source gates, ownership 48/48 retired=0, census
   1076 -> 1128 with the 52 rows explained, workspace 4133/1/61 both
   configurations. The D stack/copy/allocation/layout/lifecycle/timing bars
   were not run in rounds 7 or 8 and are this round's work.

## Questions to answer

- **Q1 — coverage.** Map the 16 route groups to the native entries with
  `path:line`: root evaluation, text/parsed/script/file/request entries,
  request resume, direct private resume, typed-slot resume, inspection and
  resume, cancellation and abandonment (host, nested and fault path),
  snapshot restore, evaluation cache, random state wrapper, host facade and
  request-control exhaustion. Show that every in-contract free start sees a
  control, including the five lp06 cases round 8 lost. R7 stays excluded.
- **Q2 — the disabled path.** Measure both draws: at each entry, the
  executed delta with its PC classes (check sequence only); the three bulk
  bodies' emitted bytes outside the gate identical after relocation; the 29
  own and 29 incoming slopes exactly equal to pre-A1; F7 at most 856; the
  313 panel with every positive a gate-site check and every non-check PC
  unchanged; the admission chain no larger than red; the native prologues at
  root 904, abandonment 856, request resume 2184, evaluator 1208, compound
  648, Catch 1000, four forwarded slow bodies 24, release wrappers absent.
  A non-check own delta, a moved body byte, a TLS access or a call on the
  disabled path is a defect, full stop.
- **Q3 — the semantics.** Re-run the 658 matrix with its exact records, red
  on the red arm and green on the design arm: the 40 inherited falsifiers,
  the four combined paths, lp06 36/36, the deferred case, the nested
  second-session resume with inner owner and outer restoration, and the
  direct private resume route that round 8 lost. Record every event line.
- **Q4 — the scope's limits.** Every read and write of the TLS top; which
  request kinds nest; why no in-contract free start can see another
  session's control or a stale entry; the same-session rollover: the guarded
  cold path, its cost and its propagation; the `Unwind`/panic path: what the
  Drop does, what remains after an unwind that crosses an entry, and whether
  any in-contract free start can observe it. Do not assert the R7 boundary.
- **Q5 — shape and bars.** The calibrated surface unchanged (ok-form E2, six
  phase-only starts, borrowed completions, E1 only at `try_apply_function`,
  four sessionful forwarded starts); the typed-resume macro and checked
  adapter unchanged; the two source gates green with the entry bodies in
  place; the ownership replay green with no HEAD row retired and every new
  design row paired; the census definition check passing, the design-tree
  count recorded for the source stage and the 52 added identity rows
  explained; the **whole** D campaign — native, stack, copy, allocation,
  layout, lifecycle, timing — measured or explicitly unpassed with the
  reason; both workspace configurations with the stale-cell ordering
  artifacts classified; `cargo fmt --check` green on the exact reproduced
  source.
- **Q6 — the bar.** If the resume entry cannot hold its check-sequence-only
  cost, STOP with the minimal mechanism, the exact cost and the
  counter-evidence — the ruling is the director's. Do not invent a waiver,
  do not lower a bar, do not move the scope to a caller enumeration, do not
  reintroduce a guard local, a token field, a token `Drop`, a
  macro/checked-adapter change or a dispatcher guard, and do not cover the
  R7 boundary.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-663/`: copy the round-7 native-entry tree and the D instruments;
build the red arm (R8) and the round-9 design arm at one common source path
with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked); measure
with the supplied runner serially, alone, on CPU 2, every `RUM_PROFILE_*`
cleared. Supplied and prior roots are read-only and never executed in place
(`.temp/brief-648/`, `.temp/brief-653/`, `.temp/brief-654/`,
`.temp/brief-655/`, `.temp/brief-656/`, `.temp/brief-657/`,
`.temp/brief-658/`, `.temp/brief-659/`, `.temp/brief-660/`,
`.temp/brief-661/`, `.temp/brief-662/`, `.temp/director/calib-655/`,
`.temp/director/verify-655/`, `.temp/director/probe-657/`,
`.temp/director/probe-658-okform/`, `.temp/director/probe-659-gen/`,
`.temp/director/probe-660/`).

**Do not** change the typed-resume macro or checked adapter, hand-edit an
inventory or census count, add a per-operation dispatcher guard, add a guard
local or token field, make the completions or forwarded starts session-free,
move the scope to the request boundary, or change the calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-663/report-design.json` with:

- the R8 baseline reproduced (build receipt and hash, 29 own/incoming pairs,
  F7, ownership verdicts, the five red-arm failures);
- the native-entry design: type, TLS stack, every entry edge, the free-start
  rule, the guarded cold rollover, every new call edge, and the emitted
  check-sequence evidence per entry;
- the entry coverage with `path:line` and the R7 out-of-contract list;
- the design arm's receipts: the 29 rows, the 313 panel (every positive a
  gate-site check, every other PC unchanged), F7, every D native and stack
  bar plus the copy, allocation, layout, lifecycle and timing bars, the
  admission chain, the per-entry check costs, the two source-proof gates,
  the ownership replay and the census verdict with the added-row
  explanation;
- the falsifier matrix with exact records, red on the red arm and green on
  the design arm, including the four combined-path falsifiers, the deferred
  case and lp06 36/36;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list keeping R7, LP655/LP656 as documented reds, the
  completion/forwarded literal gap, and every bar not measured (including
  the four While rows if the package is not yet created).

Run `python3 -I scripts/report.py check 663` and return its summary and every
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
