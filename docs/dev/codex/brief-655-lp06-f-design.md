# Brief 655 — LP06 F design: the allocation-neutral attach

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Brief 654's design was calibrated by the director. Its E1 repairs the eight
`try_apply_function` status rows exactly and does not move the 21
`eval_control_data_builtin'2` rows; reverting only `branching_loops.rs` and
`nonlocal.rs` to their pre-A1 bytes restores all 21. The residual is
therefore the presence of the while/compound/catch attach inside the 38 KB
inlined dispatcher — a whole-function register-allocation effect — not the
result borrow. This brief designs the repair that removes it.

## Goal

Design, and prove with disposable builds under this brief's root, an attach
whose **hot-path calls keep the pre-A1 base's emitted shape** at every one of
the eight LP06 sites, with all session-dependent logical work confined to
cold code that the hot path reaches without a session argument, so that the
emitted own-instruction delta at every one of the 29 measured rows is <= 0
against the pre-A1 base. The logical profile's semantics, the four While
phase rows, the forwarded-pause ordering, every D shape and every passing
panel must survive. Then draft the implementation brief in `TEMPLATE.md`
shape.

The bar is R1 as written: every one of the 29 rows at or below the pre-A1
base's own slope, no offsetting saving, no net-instruction payment. Do not
propose a compile-time gate, a feature flag, a public API, a new module
boundary, `unsafe`, or a serde/type-shape change outside
`pub(in crate::runtime)`. The runtime `AtomicU8` env gate stays.

## Context pointers

- `.temp/director/calib-655-verdict.md` — read first. It has the 29-row
  table for base / E1 / bisect1 / E1+E2, the F7 reservations, the
  known-answer reproduction, and the error in the 654 design's E1 premise.
- `.temp/brief-654/report-design.json` — sections `q1_attribution` (per-PC
  attribution), `q2_candidates`, `q3_f7`, `q4_workload_design`, `q5_probes`,
  `source_acceptance_commands`, `standing_replay_contract`.
- `.temp/brief-653/native-bars/comparison.json`,
  `.temp/brief-653/instruments/measurement-contract.json` — the D shape and
  panel contract that the repair must not move.
- `.temp/brief-653/b/resume-harness/src/main.rs` — the label→source/prompt
  mapping the While package must extend.
- Source: `crates/rum-runtime/src/runtime/profiling/timing.rs` (start and
  boundary recorders), `.../profiling/control_timing.rs` (forwarded phases),
  `.../profiling/logical/mod.rs` (`open_phase`, `complete_phase`,
  `Call::start`, `PhaseKey`), `.../profiling/logical/segments.rs` (`owner`),
  `.../profiling/logical/epoch.rs` (`current`),
  `.../kernel_data/control_data/branching_loops.rs` (While and Compound
  sites), `.../kernel_data/control_data/nonlocal.rs` (Catch),
  `.../scoping/callable.rs`, `.../scoping/callable/status_application.rs`,
  `.../eval_dispatch/mod.rs`, `.../evaluation_cache/profile/source.rs` (the
  existing thread-local profiling-source pattern).
- Supplied instrument (read-only): `.temp/brief-655/supplied/` — the 29
  labels, the 58 frozen programs, the director's base binary (29/29 known
  answer), the serial Callgrind runner, the reducer, the F7 prologue checker
  and the exact build recipe. Copy it into your own probe root before use.

## Established by the director (reproduce before doubting)

1. E1 returns all eight status rows to the base own pair exactly and leaves
   all 21 control rows at the candidate's pairs byte for byte.
2. Reverting only `branching_loops.rs` and `nonlocal.rs` to `654f2ce5`
   restores all 21 control rows to base exactly and leaves the eight status
   rows at +1.
3. The `withcleanup-small` executed path contains no profiling code at all;
   the candidate's +2 is a Session register-copy chain
   (`mov %rsi,%r15`; `mov %r15,%rbx`; `mov %rbx,%r14`; `mov %r14,%rsi`)
   against the base's stack store and reload.
4. E2 gives CLI `try_dispatch_builtin_call` reservations 616/600 (base
   856/856) and moves no 29-row own pair. F7's bar is restated as "no
   specialization above the base reservation", not "exactly 856".
5. The 29 deltas are stable across brief 648 B (`1a4c44f3`), brief 653 D
   (`029c06ff`) and the director's rebuilds.

## Questions to answer

- **Q1 — the hot shape.** For each of the eight sites, give the exact
  pre-A1 call shape and show, by build probe, that it emits the base code.
  The non-forwarded sites have a base shape already (`runtime_path_timing_start`,
  `record_profile_runtime_path_timing`). The forwarded sites are the hard
  case: the suspend phase must complete at the suspension, before the
  `resume_or_suspend!` macro returns, which is why the current start takes
  `&Result`. Design the smallest shape that keeps that ordering, and measure
  whether it is allocation-neutral. If a shape cannot be both, say so with
  the measured rows.
- **Q2 — the cold context.** The logical start/complete need the session's
  `request_interrupt_control` and the logical queue (`Call::start`,
  `logical_insert_boundary`). Design how cold code obtains that context
  without a session argument on the hot path: a thread-local context owned
  by the profiling layer, registered only when the lexical profile is
  enabled, restored/popped on every exit, safe under reset, gate
  transitions, reentrancy, several sessions on one thread, and thread
  handoff; no new default-path work, no `unsafe`. Name the registration
  point and prove that it is reached on the enabled path only.
- **Q3 — forwarded ordering.** Preserve each of the four suspend phases'
  open/complete/pause ordering and the D forwarded-shape bars (24/56/24
  release and stack). Show the exact source sequence that keeps a second
  pause attributed to the same logical call.
- **Q4 — the eight status rows.** Either the same mechanism covers
  `try_apply_function`, or E1's calibrated adapter is retained for those
  eight rows. State which, and prove the choice by probe.
- **Q5 — probes.** For every proposed edit, the exact falsifier and the
  mutant it catches: the 29 own rows at N/2N, the incoming-call slopes, the
  D native shapes, the forwarded-shape bar, the enabled logical records, and
  the F7 reservation. Use the supplied package and state the exact commands.
- **Q6 — R5.** If no shape reaches <= 0 on every row, name the mechanism
  that makes it impossible with counter-evidence and quantify the best
  measured form. That is a director decision, not a design one. Do not
  invent a waiver and do not lower the bar.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-655/`: export `HEAD` with `git archive` into your own probe
root, patch only that export, build with the supplied recipe (release,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked, one common
source path), install each arm's binary, and measure with the supplied
runner serially, alone, on CPU 2, with every `RUM_PROFILE_*` cleared. The
supplied package is read-only; copy it before writing. Prior roots
(`.temp/brief-648/`, `.temp/brief-653/`, `.temp/brief-654/`,
`.temp/director/calib-655/`) stay read-only and are never executed in place.

## Deliverable

A `rum-report/1` report at `.temp/brief-655/report-design.json` with:

- the chosen hot shape per site with `path:line` and the exact source text;
- the cold-context design with its registration point, lifecycle and
  safety argument, and the call graph it changes;
- the probe receipts: exact commands, per-arm binary receipts, and the
  29-row own pairs for the base and every variant tried, including the
  falsified variants (a negative result is evidence);
- the forwarded-phase ordering proof and the four suspend-phase plan;
- the D shape and panel preservation argument with the exact bars;
- the drafted implementation brief in `TEMPLATE.md` shape (one source
  stage, or stages if the design requires them), with the exact bars;
- an explicit unpassed list.

Run `python3 -I scripts/report.py check 655` and return its summary and every
compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, live source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- One variable per probe; a single Callgrind pair never decides a
  seed-dependent quantity; state the distribution when a quantity needs one.
- A design that pays for the repair with a new mechanism on the default
  path, a wider public API, a compile-time gate or a lowered bar is not a
  candidate.
- Every claim carries a citation the checker can resolve.
