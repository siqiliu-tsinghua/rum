# Brief 654 — LP06 E design: the residual disabled path, F7, and While coverage

**Design-only. `Scope: NONE — no source change, no workload change.`**
Brief 653 measured the pre-A1 tree (`654f2ce5`) against the repaired tree
(`029c06ff`) and showed that LP06's shapes hold while the disabled path
still costs instructions, one CLI specialization still reserves 16 more
bytes, and four mandatory While rows have no workload that can measure
them. This brief produces the repair plan and the drafted implementation
briefs. It writes no code and no workload.

## Goal

Produce a step-by-step design that finishes LP06: every one of the 29
measured owner rows returns to a non-positive own-instruction delta
against the pre-A1 base, `try_dispatch_builtin_call`'s CLI reservation
returns to 856 in both specializations, and the four While own-slope rows
become measurable with frozen, hash-pinned workloads. The plan must carry
`path:line` evidence for each cause, name the exact edit(s) per step,
show the disabled path returning to the base instruction and frame shape,
show the enabled path still recording, preserve every D shape and every
passing panel, and draft the follow-up implementation briefs in
`TEMPLATE.md` shape.

## Context pointers

- Read `AGENTS.md`, then `.temp/brief-653/report.json` sections `decision`,
  `instructions`, `F6`, `F7`, `timing` and `package`, and my verdict
  `.temp/director/brief-653-verdict.md`.
- The retained procedure is `docs/dev/codex/brief-653-lp06-d-repaired-campaign.md`
  (with amendment 1) and the B procedure behind it
  (`docs/dev/codex/brief-648-lp06-b-disabled-path.md`, amendments 1–3).
  The source stages: briefs 650 (C1), 651 (C2), 652 (C3).
- The decisive tables: `.temp/brief-653/tables/unpassed.json`,
  `native-bars/f6-all-own-positive-labels.json`,
  `tables/f6-executed-paths.json`, `site-evidence/mandatory-pc-tables.json`,
  `native-bars/comparison.json`, `tables/while-coverage-gap.json`,
  `tables/panel-verdicts.json`, `tables/converted-sites.json`.
- The pinned artifacts, read-only: `.temp/brief-653/instruments/bin/{base,candidate}/`
  and the raw Callgrind outputs under
  `.temp/brief-653/instruments/copy-census/`.
- Key source files: `crates/rum-runtime/src/runtime/profiling/timing.rs`
  (adopted and forwarded starts, boundary recorders),
  `.../profiling/control_builtin.rs` (builtin recorder),
  `.../profiling/control_timing.rs` (forwarded recorder),
  `.../profiling/config.rs` (`lexical_profile_enabled` and the three
  atomics), `.../profiling/mod.rs` (`RuntimePathTimingToken`),
  `.../kernel_data/control_data/branching_loops.rs` (While and Compound
  sites), `.../kernel_data/control_data/nonlocal.rs` (Catch),
  `.../eval_dispatch/mod.rs` (`try_dispatch_builtin_call`),
  `.../scoping/callable.rs` and `.../scoping/callable/status_application.rs`
  (`try_apply_function` path).

## Measured failure (evidence; do not re-derive from scratch)

- **F6, the residual disabled path.** 29 owner rows carry a positive own
  instruction slope: 21 in `eval_control_data_builtin'2` at +1..+3 per
  iteration, 8 in `try_apply_function` at +1. The director reproduced
  five rows directly from the raw Callgrind own pairs (N and 2N, `fn=`
  blocks, the `calls=` line consuming exactly one cost line):
  table-small 78 -> 80, appendto-part-outer1000-large 187/3 -> 190/3,
  set-part-fixed100-len800 79 -> 80, withcleanup-small 62 -> 64,
  failure-message-property 49 -> 50. Incoming-call slopes are equal in
  both arms. The same labels save 28 Ir/iteration in
  `try_dispatch_builtin_call'2`; that saving does not pay for this.
  The director's diff of the withcleanup executed path shows register
  reallocation rather than an added check: base
  `mov %rsi,0xX(%rsp)` / `mov %rdi,0xX(%rsp)` against candidate
  `mov %rsi,%r15` / `mov %rdi,(%rsp)`, plus two added register moves and
  one replaced load. `RuntimePathTimingToken` itself did not change
  (`start: Instant`, `key`).
- **F7.** CLI reservations base [856, 856], candidate [872, 856]
  (`native-bars/comparison.json`); 6 pushes (48) + `sub $0x338` (824) =
  872 at candidate entry `0x5d72e0`; 48 + 808 = 856 at `0x5d7bf0`; base
  856 at both. B was 888/872.
- **While coverage.** `tables/while-coverage-gap.json`: all 576 frozen
  `.rum` programs and the ten resume-harness files searched for
  `While\s*\[|resume_while_loop|while_test|while_body`; zero matches and
  zero observed `resume_while_loop` own rows.
- **Passing and not to be lost** (`tables/panel-verdicts.json`): copies
  192/192; allocations 194 rows with identical ordinary slopes; canonical
  stack 528 probes / 44 zero-residual fits; LP06 stack 216 / 18; layouts
  638 runtime + 810 stdlib entries unchanged, 563 covered frames; live
  chains 352; lifecycle 6 controls + 14 tests / 21 second-pause cases;
  disabled cold-edge census zero calls. Shapes: While 744/3528/744,
  Compound 648/1304/664, Catch 1000/792/1000, forwarded slow 24/56/24,
  release wrappers absent, token 56/56.
- **F5 is closed** (`-12` calls/iteration inside a 69..116 spread across
  five repeats). It is not a design input and must not be reopened.

## Director rulings (decided; do not reopen)

- **R1 — the bar.** The disabled path must return to the pre-A1 base's
  own-instruction cost at every one of the 29 rows. A net saving elsewhere
  in the same label is not a payment. Do not propose lowering the bar.
- **R2 — no new mechanism.** The fix lives inside the existing profiling
  machinery: no public API, no feature flag, no new crate, no `unsafe`, no
  serde or type-shape change outside `pub(in crate::runtime)`.
- **R3 — the D shapes are constraints.** Every listed shape and panel must
  stay at its D value; the fix may not buy instructions with frames,
  layouts or stack.
- **R4 — workloads are a separate deliverable.** Design the While
  workloads and their hash-manifest update, but do not edit them; their
  implementation is a separate brief after mine.
- **R5 — the design is not a stop.** If a zero-cost form cannot be found
  for a row, the design must say exactly which mechanism makes it
  impossible, with the counter-evidence, and quantify the best achievable
  form. That is a director decision, not a design decision.

## Questions to answer (each with `path:line` evidence)

- **Q1 — attribution.** For all 29 rows, attribute the delta to specific
  emitted instructions by diffing base and candidate executed paths at the
  owner entry (`tables/f6-executed-paths.json`,
  `site-evidence/mandatory-pc-tables.json`). Classify each row: token live
  across the body, extra disabled check, register pressure from a changed
  body, changed control-flow edge, or allocation. Give the per-row table
  and the instruction that carries the delta.
- **Q2 — the minimal edit.** Name the edit(s) that return every row to
  delta <= 0 with the predicted per-row instruction delta. Consider at
  least: a register-sized token (for example an index into a thread-local
  stack) instead of the 32+ byte `RuntimePathTimingToken` held across the
  body; a single combined enabled flag instead of the
  configured-and-active pair; and hoisting one flag read per evaluator
  entry instead of one per site. For each candidate, state what it does to
  the enabled path and to the passing panels.
- **Q3 — F7.** Attribute the 16-byte reservation in specialization 1 to
  the local or token that adds it, and name the edit that returns both
  specializations to 856. Keep the reservation bar independent of any
  instruction saving.
- **Q4 — While coverage.** Design the frozen workload additions: a CLI
  `.rum` While family (N, 2N and check programs) that drives
  `while.test.evaluate` and `while.body.evaluate`, and a resume-harness
  While scenario that drives `while.test.suspend` and
  `while.body.suspend` to a second pause (the harness maps labels to
  sources and expected prompts in
  `.temp/brief-653/b/resume-harness/src/main.rs`). Give exact sources, N
  selection with the iteration-proportional argument, expected slopes, the
  hash-manifest update, and the coverage proof that each of the four rows
  gets an observed owner. Name the label names, the N values and the
  exact files to be added.
- **Q5 — probes.** For every proposed edit, the exact probe that would
  falsify it before dispatch (own-instruction slopes at N/2N, native
  reservations, the D shapes, the passing panels), and the mutant each
  probe catches.

## Scope

**NONE.** No live source edit, no workload edit, no hash-manifest edit, no
inventory replay, no acceptance suite, no version-control command. The
report is the only output. Every artifact under `.temp/brief-654/`; every
retained root (`.temp/brief-648/`, `.temp/brief-653/`) stays read-only and
is never executed in place.

## Deliverable

A `rum-report/1` report at `.temp/brief-654/report-design.json` with:

- the Q1 per-row attribution table (29 rows, instruction-level);
- the Q2 and Q3 edit lists with `path:line`, predicted deltas and the
  effect on the enabled path;
- the Q4 workload design with exact sources, N values, label names and the
  hashes to be added;
- the Q5 probe list with the mutants each catches;
- the drafted implementation briefs in `TEMPLATE.md` shape;
- an explicit unpassed list.

Run `python3 -I scripts/report.py check 654` and return its summary and
every compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- A design that pays for the fix with a new mechanism, a wider public API,
  or a default-path cost elsewhere is not a candidate.
- Every claim carries a citation the checker can resolve; prose without a
  citation is not evidence.
