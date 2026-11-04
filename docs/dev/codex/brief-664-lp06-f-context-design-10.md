# Brief 664 — LP06 F design round 10: the resume identity off the default path

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Round 9 restored the round-7 native entries and refined the gate to its
smallest form: root and abandonment add the check pair and nothing else, but
the resume entry pays 56 non-check own instructions per request under four
different gate encodings, because the check sits in a 4,693-instruction
inlined function and a single branch re-allocates it (session r12 -> rbp, two
prologue stores, 574 non-equal hunks). Round 10 keeps the native entries and
their semantics, removes the gate from the resume function entirely, and
designs how the resume's free starts see the session's current control
without any default-path branch.

## Goal

Design, and prove with disposable builds under this brief's root, a round-10
tree in which the resume entry
(`resume_eval_request_from_checkpoint`) is emitted exactly as at the red arm
— no check, no branch, no byte outside its live body after relocation — while
the six phase-only starts and the direct private resume still see the resumed
session's current control, and `evaluate_at_root` and
`drain_abandonment_worklist` keep a disabled-path cost of the gate's check
sequence and nothing else. The identity may travel in an enabled-only
carrier (the profile's own per-cell context, queue or suspension state), in
an existing outlined site that actually covers the frame drive, or in any
other mechanism; it may not be installed by a branch, call, TLS access or
frame slot inside the 4,693-instruction checkpoint body. All round-9
semantics (40 inherited falsifiers, four combined paths, lp06 36/36 including
the five round-8 losses and the direct private resume) must hold red on the
red arm and green on the design arm. The admission chain stays no larger than
red, the 29 rows exact, F7 at most 856, every 313-panel positive a gate-site
check, the **whole** D campaign measured (native, stack, copy, allocation,
layout, lifecycle, timing — the campaign rounds 7, 8 and 9 never ran), and
the design source fmt-clean at the moment it is measured. Then draft the
implementation brief in `TEMPLATE.md` shape.

## Context pointers

- `.temp/director/brief-663-gate.md` — the round-9 STOP, the four gate
  encodings, the 8 + 56 decomposition, the body-byte evidence, the root and
  abandonment +8 rows, the ruled bar. Read it first.
- `.temp/brief-663/report-design.json` — the round-9 evidence that still
  stands: 29 rows, F7, admission 77 -> 63, native bars, the 313 subset, the
  source gates, ownership, census 1076 -> 1130, both workspaces, the 40 + 4 +
  lp06 36/36 semantics, and the STOP row. Re-measure on the round-10 shape;
  do not re-derive.
- `.temp/brief-663/probe/` — the round-9 trees: `design-src` (the base for
  this round, semantics-proven), `red-src` (the R8 red arm, unchanged),
  `r7-src`, the instrument, `focus/`, `entry-cost/`, `d-instruments/`, the
  native objdumps and the logs. Copy before use.
- `.temp/director/brief-661-gate.md` and `.temp/director/brief-662-gate.md` —
  the round-7 and round-8 stops: the corrected bar, the boundary
  falsification, the five lp06 losses and the direct private resume cause.
- `.temp/brief-658/report-design.json` — the proven semantics
  (`q5_falsifier_matrix`, `q5_helper_bodies`, `q5_record_logs`), the entry
  inventory (`q1_entry_inventory`, `q1_registrations`,
  `q1_out_of_contract`), the context/match rule (`q2_context`,
  `q3_match_rule`).
- `.temp/brief-655/supplied/` — the supplied 29-label instrument and the
  build recipe. `.temp/brief-653/` — the D contract, native bars and the 313
  program set (read-only). `.temp/brief-654/report-design.json` — the While
  package and the probe contracts.
- Live source: `profiling/logical/context.rs`, `logical/mod.rs`
  (`Call::start`, `start_from_active`, `open_phase`, `push_phase`),
  `logical/segments.rs` (`owner`, `current_owner`), `resolution.rs`
  (`evaluate` at 315, `evaluate_at_root` at 330),
  `suspension/core.rs` (`resume_suspended_evaluation` at 626,
  `resume_suspended_slot` at 632), `suspension/core/driver.rs`
  (`drive_resuming_frames`, `drive_frames`),
  `suspension/core/profile_queue.rs` (`Context::capture`, `enter_front`,
  `observe_boundaries`), `suspension/core/abandon.rs`
  (`drain_abandonment_worklist` at 21), `session/session_requests.rs`
  (`resume_eval_request_with_input` at 283,
  `resume_eval_request_from_checkpoint` at 315),
  `session/session_api.rs` (`accept_cancellable_request`),
  `profiling/timing.rs` and `profiling/config.rs`.

## Established by the director (reproduce before doubting)

1. **The resume entry's gate cannot be tuned away.** Four encodings — the
   round-7 two-arm match (14 check), two 4-instruction `!= 2` forms, and the
   final design — all pay exactly 56 non-check own instructions per request
   in both draws (mov +58, movabs +1, lea -3), with the session register
   moving r12 -> rbp and two prologue stores. The residual is the branch's
   presence in the 4.6k body, not the check's shape.
2. **Root and abandonment are already right**: `evaluate_at_root` 223 -> 231
   (+8, check 8, other 0) and `drain_abandonment_worklist` 1262 -> 1270
   (+8, check 8, other 0), both modes, both draws. Keep them.
3. **The inlining is total.** `drive_frames`, `drive_resuming_frames`,
   `resume_suspended_slot` and `resume_suspended_evaluation` have no symbols
   in the release binary; their code is inside
   `resume_eval_request_from_checkpoint` (4,693 own Ir/request at the red
   arm). A forced outline is a call on the disabled path and is not
   permitted.
4. **The enabled-only structures already present across a resume.** The
   queue's per-cell `Context` is captured at pause and entered by
   `enter_front`/`observe_boundaries`/`abandon_boundaries`; the suspension's
   frames each restore a profile context in `drive_frames`. Whether every
   free start is covered by an entered context, and what the control's
   lifetime is across thread handoff, rollover, reset and unwind, is the
   coverage question — do not assume it; prove it with `path:line` and a
   falsifier.
5. **Round 9's tree is the semantics standard**: 40 inherited literal
   falsifiers, the four combined paths, lp06 36/36 (the five round-8 losses
   green, the direct private resume included), red on the red arm. LP655 and
   LP656 are the documented R7 bare-dispatch probes, red in both arms; they
   stay named and are not repaired.
6. **Everything round 9 established is re-measured**: 29/29 own and incoming
   pairs exact against the pre-A1 base, F7 600/632 (bar 856), admission
   77 -> 63, native root 904 / abandonment 856 / request resume 2184 /
   evaluator 1208 / while 744 / compound 648 / Catch 1000 / forwarded 24 x4,
   both source gates, ownership 48/48 retired=0, census 1076 -> 1130 with
   the 52 inherited rows and the two RET002 rows explained, both workspace
   configurations with only the census count failure, host suites 5/5 each.
7. **The whole D campaign is this round's work** and was not run by rounds
   7, 8 or 9. The four While rows remain 665's package and stay unpassed;
   the completion/forwarded literal gap and R7 stay unpassed and named.
8. **The calibrated surface is fixed**: ok-form E2, six phase-only starts,
   borrowed completions, E1 only at `try_apply_function`, four sessionful
   forwarded starts; the typed-resume macro and checked adapter unchanged;
   the admission design (`accept_after_request_exhaustion`, the shared
   `reserve_request_identity`, the cold outlined `refresh_execution_control`)
   unchanged in behavior and no larger than red.
9. **The design tree was not fmt-clean.** The measured source must be
   `cargo fmt --check` green; format before the accepted measurements, not
   after.

## Questions to answer

- **Q1 — the mechanism space.** Enumerate every way the resume's free starts
  can see the resumed session's current control with no branch in the
  checkpoint function. At minimum: (a) an enabled-only carrier in the
  profile's own state — the per-cell `Context`, the queue cells, the
  suspension; (b) an existing outlined site on the resume path that actually
  covers the frame drive; (c) a changed free-start lookup rule. For each
  candidate give the exact source, the disabled-path emission of
  `resume_eval_request_from_checkpoint` (own PCs and bytes versus the red
  arm, after relocation), the coverage argument for all six phase starts and
  the direct private resume, the lifetime of the carried value across
  suspension, thread handoff, rollover, reset and unwind, and the measured
  cost of the D resume row plus the simple entry harness. No candidate is
  selected unmeasured; the selected one is presented beside the measured
  others.
- **Q2 — the disabled path.** The resume function's own slope is exactly
  4,693 with no non-live PC and identical emitted bytes after relocation;
  root and abandonment add the check sequence only; the 29 own and incoming
  slopes exact; F7 at most 856; the 313 panel with every positive a gate-site
  check and every non-check PC unchanged; the admission chain no larger than
  red; no TLS access, call or frame slot on the disabled path. A non-check
  positive, a moved body byte, or a call is a defect, full stop.
- **Q3 — the semantics.** Re-run the 658 matrix with its exact records, red
  on the red arm and green on the design arm: the 40 inherited falsifiers,
  the four combined paths, lp06 36/36 including the five round-8 losses and
  the direct private resume, the deferred enable, the nested second-session
  resume with inner owner and outer restoration, and a carrier-specific
  falsifier that fails if a free start during a resume receives a stale or
  foreign control (rollover, thread handoff, reset). Record every event
  line.
- **Q4 — the scope's limits.** Every read and write of the TLS top under the
  new mechanism; which request kinds nest; why no in-contract free start can
  see another session's control or a stale entry; the same-session rollover:
  the guarded cold path, its cost and its propagation; the `Unwind`/panic
  path: what the Drop does, what remains after an unwind that crosses the
  resume, and whether any in-contract free start can observe it. Do not
  assert the R7 boundary.
- **Q5 — shape and bars.** The calibrated surface unchanged; the two source
  gates green; the ownership replay green with no HEAD row retired and every
  new design row paired; the census definition check passing, the design-tree
  count recorded for the source stage, and the two RET002 rows either
  justified by their checked provenance or removed; the **whole** D campaign
  — native, stack, copy, allocation, layout, lifecycle, timing — measured or
  explicitly unpassed with the reason; both workspace configurations with the
  stale-cell ordering artifacts classified; `cargo fmt --check` green on the
  exact measured source.
- **Q6 — the bar.** STOP only when the mechanism space is exhausted: every
  candidate measured, the minimal mechanism named with its exact cost and
  counter-evidence — the ruling is the director's. Do not invent a waiver,
  do not lower a bar, do not move the scope to a caller enumeration, do not
  reintroduce a guard local, a token field, a token `Drop`, a
  macro/checked-adapter change or a dispatcher guard, and do not cover the
  R7 boundary.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-664/`: copy the round-9 `design-src` tree (the base), the R8
`red-src` tree and the D instruments; build the red arm and the round-10
design arm at one common source path with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked); measure
with the supplied runner serially, alone, on CPU 2, every `RUM_PROFILE_*`
cleared. Supplied and prior roots are read-only and never executed in place
(`.temp/brief-648/`, `.temp/brief-653/`, `.temp/brief-654/`,
`.temp/brief-655/`, `.temp/brief-656/`, `.temp/brief-657/`,
`.temp/brief-658/`, `.temp/brief-659/`, `.temp/brief-660/`,
`.temp/brief-661/`, `.temp/brief-662/`, `.temp/brief-663/`,
`.temp/director/calib-655/`, `.temp/director/verify-655/`,
`.temp/director/probe-657/`, `.temp/director/probe-658-okform/`,
`.temp/director/probe-659-gen/`, `.temp/director/probe-660/`,
`.temp/director/gate-661/`).

**Do not** change the typed-resume macro or checked adapter, hand-edit an
inventory or census count, add a per-operation dispatcher guard, add a guard
local or token field, make the completions or forwarded starts session-free,
insert a branch in the checkpoint function, or change the calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-664/report-design.json` with:

- the round-9 baseline reproduced (the build receipt and hash, the 29 pairs,
  F7, the native bars, the admission chain, the semantics, the resume STOP
  row) and the red arm's failures;
- the mechanism-space table: every candidate, its exact source, its
  disabled-path emission at the resume function (own PCs and bytes after
  relocation), its coverage proof for the six free starts and the direct
  private resume, its lifetime argument, and its measured cost (D resume row,
  entry harness, 29 rows, F7, admission), with the selected candidate named
  and the others measured beside it;
- the design of the selected mechanism: the carrier/lookup, the enabled-only
  sites, every new call edge, the guarded cold rollover, and the resume
  function's byte-identity evidence;
- the design arm's receipts: the 29 rows, the 313 panel, F7, every D native
  and stack bar plus the copy, allocation, layout, lifecycle and timing bars,
  the admission chain, the per-entry check costs, the two source-proof gates,
  the ownership replay and the census verdict with the added-row explanation;
- the falsifier matrix with exact records, red on the red arm and green on
  the design arm, including the four combined-path falsifiers, the deferred
  case, lp06 36/36 and the carrier-specific stale/foreign-control falsifier;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list keeping R7, LP655/LP656 as documented reds, the
  completion/forwarded literal gap, and every bar not measured (including
  the four While rows as 666's package).

Run `python3 -I scripts/report.py check 664` and return its summary and every
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
