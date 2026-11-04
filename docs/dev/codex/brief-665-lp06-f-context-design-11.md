# Brief 665 — LP06 F design round 11: the free-start lookup off the diagnostic storage

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Round 10 measured the three required mechanism families and two lifetime
repairs. All five reach the exact zero checkpoint emission (4,693 own Ir in
both draws, byte-identical after decoded address relocation); capture and
lookup fail inherited semantics; the existing-hook family passes every
inherited falsifier but loses a phase after confirmed diagnostic-cap
retirement, pause, external profile reset and private resume. Its configured
repair also adds one non-check instruction per `drive_input_request` call on
all 121 resume labels. The report names one unmeasured mechanism — a shared
Session-control indirection. Round 11 designs and measures the free-start
lookup that reads the session's current control without the diagnostic
storage, keeping the checkpoint body byte-identical and every disabled path
instruction-identical to red.

## Goal

Design, and prove with disposable builds under this brief's root, a round-11
tree in which:

- the resume entry (`resume_eval_request_from_checkpoint`) is emitted exactly
  as at the red arm — no check, no branch, no call, no TLS access, no frame
  slot, no byte outside its live body after decoded address relocation, own
  slope 4,693 in both draws;
- `evaluate_at_root` and `drain_abandonment_worklist` pay the check sequence
  and nothing else (223 -> 231 each);
- the six phase-only free starts and the direct private resume see the
  resumed session's **current** execution control in every in-contract state,
  including after a confirmed diagnostic-cap retirement, a pause, an external
  profile reset issued by another session, a same-session rollover, a thread
  handoff and an unwind;
- **no disabled-path instruction is added in any function**: the 121 resume
  labels' `drive_input_request` own Ir/call stays 88, the six free starts keep
  their red own slopes, and no new non-check own instruction appears in any
  owner on any default path; every new instruction sits behind an existing
  enabled gate, measured;
- admission is no larger than red (77 Ir/admission on both routes);
- every round-9 semantics holds: lp06 36/36 including the five round-8 losses
  and the direct private resume, combined 4/4, transitions 11/11, entries
  23/23, host 3/3, the 39+1 literal split (39 red on red, one green host-root
  control, no assertion rewritten), and the round-10 counterexample
  `lp664_carrier_cap_pause_external_reset` red on red and green on design;
- the 29 own and incoming pairs are exact, F7 stays at most 856, the native
  bars equal red, the whole D campaign (native, stack, copy, allocation,
  layout, lifecycle, timing, the 313 panel) is measured on the selected
  shape, the two source gates, the ownership replay and the census definition
  check are green, and the measured source is `cargo fmt --check` clean.

Then draft the implementation brief in `TEMPLATE.md` shape with the exact bars.

## Context pointers

- `.temp/director/brief-664-gate.md` — this round's ruling, the reproduced
  rows, the bar corrections. Read it first.
- `.temp/brief-664/report-design.json` — the mechanism table, the hook
  design and its source anchors, the cap counterexample, the input-driver +1,
  the 313 panel and the whole D receipts. Re-measure on the round-11 shape;
  do not re-derive.
- `.temp/brief-664/probe/` — the measured arms: `hook-configured-src` (the
  closest arm; the base unless you justify another), `hook-src`,
  `hook-reset-src`, `capture-src`, `lookup-src`, `r9-src`, `red-src`; the
  instrument; `focus/`, `entry-cost/`, `d-instruments/`, the native objdumps.
- `.temp/brief-663/probe/design-src` — round 9's tree, the semantics
  standard and the fallback base.
- `.temp/brief-664/cap-reset-counterexample.json` and
  `.temp/brief-664/input-drive-proof/comparison.json` — the two hard
  counterexamples to close.
- `.temp/brief-658/report-design.json` — the proven semantics, the entry
  inventory, the context/match rule.
- `.temp/brief-655/supplied/` — the 29-label instrument and the build recipe.
  `.temp/brief-653/` — the D contract, native bars and the 313 program set
  (read-only). `.temp/brief-654/report-design.json` — the While package and
  probe contracts.
- Live source: `profiling/logical/context.rs`, `logical/mod.rs`
  (`Call::start`, `start_from_active`, `open_phase`), `logical/segments.rs`
  (`owner`), `resolution.rs` (`evaluate`, `evaluate_at_root`),
  `suspension/core.rs` (`resume_suspended_evaluation`,
  `resume_suspended_slot`), `suspension/core/driver.rs` (`drive_frames`,
  `drive_resuming_frames`), `suspension/core/profile_mode.rs`,
  `suspension/core/profile_queue.rs` (`Context::enter_with_control`,
  `enter_front`, `enter_front_with_control`), `session/session_requests.rs`,
  `session/session_api.rs`, `builtins_input.rs` (`drive_input_request`),
  `profiling/timing.rs`, `profiling/config.rs`.

## Established by the director (reproduce before doubting)

1. **The zero checkpoint emission is real.** My own disassembly: capture,
   hook, hook-reset, hook-configured and lookup are all 4,606 instructions /
   25,861 B, with 4,271 identical raw bytes and 335 instructions differing
   only in address operands (direct call targets, RIP displacements) — zero
   real text differences; slopes 4,693/4,693 and deltas 0/0 in both draws.
   Round 9's arm is the control: 4,661 instructions, 25,797 B, +64.
2. **The retained costs are real**: 29/29 own and incoming exact in 14
   arm/draw variants; admission 77 -> 63 on both routes and both draws; simple
   entry 2,223; root 223 -> 231 (+8 check).
3. **The cap counterexample is real and reproduced in both trees**:
   `r9-cap-sem` passes `lp664_carrier_cap_pause_external_reset` 1/1;
   `hook-configured-sem` fails 0/1 with the resumed
   `compound.item.evaluate` record missing.
4. **The +1 is real**: from the raw disabled-path panel captures,
   `drive_input_request` own Ir/call is 88 in red/round 9/capture/hook/lookup/
   hook-reset and 89 in hook-configured, on all 121 resume labels, both draws,
   with equal incoming slopes; the extra `movzbl` stack-byte reload is
   introduced by the configured-deferred repair.
5. **Why the hook family fails.** The mode selection
   (`s.frames.is_profiled()` in `drive_resuming_frames`) is inlined inside the
   4.6k checkpoint body and cannot change there. After confirmed cap
   retirement the pause leaves Guest storage, so `Profiled::resume` is never
   called and the queue `Context` is absent. The session's current control
   (`session.request_interrupt_control`) remains live across that retirement
   and across another session's reset — round 9's `ExecutionScope::enter(self)`
   reads exactly it at the resume entry and passes the counterexample. The
   remaining mechanism space is the free-start lookup, not the checkpoint
   body.
6. **The literal 40 is 39+1**: one green host-root control in the red arm; do
   not rewrite the assertion.
7. **The 313 panel is decided on deterministic rows.** Its per-label
   non-check deltas are both-signed and as large on untouched functions as on
   any target (`Symbol::system` +9.06 / -1.54 per request across draws). Report
   both draws and the untouched-function band per label; an unattributed
   positive inside the band is not evidence, a deterministic positive that is
   not a gate check is a defect, and the `drive_input_request` row and the six
   free starts' own slopes are hard.
8. **The D campaign paid this round** and is measured again on the selected
   shape. The four While own-slope rows remain package 667's; R7 and
   LP655/LP656 stay named and are not repaired.
9. **The calibrated surface is fixed**: ok-form E2, six phase-only starts,
   borrowed completions, E1 only at `try_apply_function`, four sessionful
   forwarded starts; the typed-resume macro and checked adapter unchanged.

## Questions to answer

- **Q1 — the remaining mechanism space.** Enumerate every way the free starts
  can see the resumed session's current control without the diagnostic
  storage and without a checkpoint byte: at minimum (a) the free-start lookup
  reads the session's current control (the value that survives cap
  retirement) behind the existing enabled gate, instead of or in addition to
  the per-cell `Context`; (b) a suspension-carried control; (c) a per-cell
  `Context` that survives cap retirement; (d) another measured mechanism.
  For each candidate: the exact source, the checkpoint emission (own PCs,
  own slope, bytes after decoded address relocation), the disabled-path
  emission of every changed function (`drive_input_request` 88/call, the six
  free starts, the simple entry, root and abandonment), the coverage proof
  for all six phase starts and the direct private resume, the lifetime across
  cap retirement, external reset, rollover, handoff, unwind and the nested
  second session, and the measured cost (resume row, entry harness, 29 rows,
  F7, admission). No candidate is selected unmeasured; the selected one is
  presented beside the measured others.
- **Q2 — the disabled path.** The resume function's own slope is exactly
  4,693 with no non-check own PC and identical emitted bytes after
  relocation; root and abandonment add the check sequence only; 29/29 exact;
  F7 at most 856; admission no larger than 77; `drive_input_request` exactly
  88 own Ir/call on all 121 resume labels; the 313 panel reported per label
  in both draws with the untouched-function band and every deterministic
  positive attributed. A non-check positive, a moved body byte, a call, a TLS
  access or a frame slot in the checkpoint function is a defect, full stop.
- **Q3 — the semantics.** Re-run the 658 matrix with its exact records, red
  on the red arm and green on the design arm: the 39+1 literal split, the four
  combined paths, lp06 36/36 including the five round-8 losses and the direct
  private resume, the deferred enable, the nested second-session resume with
  inner owner and outer restoration, the cap/pause/external-reset/private-
  resume falsifier (byte-identical test body on both arms), and a
  carrier-specific falsifier that fails if a free start during a resume
  receives a stale or foreign control (rollover, handoff, reset, unwind).
  Record every verdict line.
- **Q4 — the scope's limits.** Every read and write of the carrier under the
  new mechanism; which request kinds nest; why no in-contract free start can
  see another session's control or a stale entry; the same-session rollover:
  the guarded cold path, its cost and its propagation; the `Unwind`/panic
  path: what remains after an unwind and whether any in-contract free start
  can observe it. Do not assert the R7 boundary.
- **Q5 — shape and bars.** The calibrated surface unchanged; the two source
  gates green; the ownership replay green with no HEAD row retired and every
  new design row paired; the census definition check passing with the actual
  count recorded; the whole D campaign measured or explicitly unpassed with
  the reason; both workspace configurations with the stale-cell ordering
  artifacts classified; `cargo fmt --check` green on the exact measured
  source; the implementation draft in `TEMPLATE.md` shape.
- **Q6 — the bar.** STOP only when the remaining mechanism space is
  exhausted: every candidate measured, the minimal mechanism named with its
  exact cost and counter-evidence — the ruling is the director's. The
  round-10 hook and its configured repair are measured counter-evidence, not
  a base to re-argue. Do not invent a waiver, do not lower a bar, do not move
  the scope to a caller enumeration, do not reintroduce a guard local, a
  token field, a token `Drop`, a macro/checked-adapter change or a dispatcher
  guard, do not add a default-path instruction, and do not cover the R7
  boundary.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-665/`: copy the round-10 `hook-configured-src` tree (the closest
measured arm) and, if you choose it, the round-9 `design-src` (the semantics
standard); copy the R8 `red-src` tree and the D instruments; build the red
arm and the round-11 design arm at one common source path with the supplied
recipe (release, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline,
locked); measure with the supplied runner serially, alone, on CPU 2, every
`RUM_PROFILE_*` cleared. Supplied and prior roots are read-only and never
executed in place (`.temp/brief-648/`, `.temp/brief-653/`,
`.temp/brief-654/`, `.temp/brief-655/`, `.temp/brief-656/`,
`.temp/brief-657/`, `.temp/brief-658/`, `.temp/brief-659/`,
`.temp/brief-660/`, `.temp/brief-661/`, `.temp/brief-662/`,
`.temp/brief-663/`, `.temp/brief-664/`, `.temp/director/calib-655/`,
`.temp/director/verify-655/`, `.temp/director/probe-657/`,
`.temp/director/probe-658-okform/`, `.temp/director/probe-659-gen/`,
`.temp/director/probe-660/`, `.temp/director/gate-661/`,
`.temp/director/gate-663/`, `.temp/director/gate-664/`).

**Do not** change the typed-resume macro or checked adapter, hand-edit an
inventory or census count, add a per-operation dispatcher guard, add a guard
local or token field, make the completions or forwarded starts session-free,
insert a branch in the checkpoint function, add a default-path instruction,
or change the calibrated surface.

## Deliverable

A `rum-report/1` report at `.temp/brief-665/report-design.json` with:

- the round-10 baseline reproduced (the checkpoint emission, the 29 pairs,
  F7, admission, the cap counterexample, the +1) and the red arm's failures;
- the mechanism-space table: every candidate, its exact source, its
  checkpoint emission (own PCs, slope, bytes after relocation), its
  disabled-path emission (every changed owner, `drive_input_request` per
  call, the six free starts, root and abandonment, admission), its coverage
  proof for the six free starts and the direct private resume, its lifetime
  argument, and its measured cost, with the selected candidate named and the
  others measured beside it;
- the design of the selected mechanism: the carrier/lookup, the enabled-only
  sites, every new call edge, the guarded cold rollover, and the checkpoint's
  byte-identity evidence;
- the design arm's receipts: the 29 rows, the 313 panel with the noise band,
  F7, the native bars, the whole D campaign (native, stack, copy, allocation,
  layout, lifecycle, timing), the admission chain, the per-entry check costs,
  the two source-proof gates, the ownership replay and the census verdict;
- the falsifier matrix with exact records, red on the red arm and green on
  the design arm, including the cap/pause/external-reset falsifier and the
  stale/foreign-control falsifier;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact bars;
- an explicit unpassed list keeping R7, LP655/LP656, the completion/forwarded
  literal gap, the literal 39+1 split, the census count and every bar not
  measured (including the four While rows as package 667's).

Run `python3 -I scripts/report.py check 665` and return its summary and every
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
