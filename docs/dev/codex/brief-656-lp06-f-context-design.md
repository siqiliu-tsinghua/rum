# Brief 656 — LP06 F design round 2: the session-execution cold context

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes are required and live
only under this brief's root.**
Brief 655 established a numerically exact but semantically incomplete form
(F3+E2): the six phase-only starts reproduce all 29 base own pairs and the
incoming slopes, and E2 keeps the dispatch reservation at 616/600, yet five
enabled checks fail because the cold start cannot recover the session
identity at a root, after a deferred enable, and in three resumed tests. This
brief designs that context and proves it, without changing the calibrated
surface.

## Goal

Design, and prove with disposable builds under this brief's root, the
session-execution context that makes the six session-free starts **equal to
the sessionful form in every reachable execution state** — root, deferred
enable, resume, reentrancy, several sessions on one thread, thread handoff,
reset and gate transitions — while every one of the 29 retained rows stays at
or below the pre-A1 base's own slope, the incoming-call slopes stay equal,
the F7 reservation stays at most 856, every D native and stack bar holds, and
every existing enabled test is green. The calibrated F3+E2 surface is the
baseline; the design adds the context and nothing else. Then draft the
implementation brief in `TEMPLATE.md` shape.

The context may not be paid for with a new per-operation branch on the
disabled path, a wider public API, a compile-time gate, `unsafe`, a new
module boundary, or a new shared mutable queue. The runtime `AtomicU8` env
gate and the existing thread-local patterns stay.

## Context pointers

- `.temp/director/brief-655-gate.md` — my gate: my own build, the reproduced
  29/29, F7 616/600, the five failures reproduced at one thread, and the
  generation-predicate experiment that does **not** fix them. Read first.
- `.temp/brief-655/report-design.json` — sections `q1_hot_shapes`,
  `q2_cold_context`, `q4_status`, `standing_replay_contract`, `unpassed`,
  `sections.design` and `sections.implementation-draft`.
- `.temp/brief-655/probe/f3e2-src/` — the F3+E2 tree (read-only reference;
  copy before use). Its eight edited files are the baseline surface. Its
  `verify-f3/.../tests/control.rs` carries the two falsifiers.
- `.temp/brief-655/supplied/` — the supplied instrument (read-only: 29
  labels, 58 frozen programs, known-answer base binary, serial Callgrind
  runner, reducer, F7 prober, build recipe). Copy it into your own probe
  root before use.
- Live source: `crates/rum-runtime/src/runtime/resolution.rs` (the
  evaluation entry and `evaluate_at_root`),
  `.../profiling/logical/mod.rs` (`Call::start`, `open_phase`,
  `complete_phase`, the phase LIFO),
  `.../profiling/logical/segments.rs` (`owner`, `restore`, the active
  stack), `.../profiling/logical/context.rs` (`Context::capture`/`enter`,
  `ContextGuard`), `.../profiling/logical/queue_api.rs`,
  `.../profiling/reset.rs` (the gate activation), `.../profiling/config.rs`
  (the cold-load gate), `.../suspension/core/profile_mode.rs` (resume
  context), `.../session/session_api.rs` (the public evaluation entries),
  `.../evaluation_cache/profile/source.rs` (the existing TLS guard pattern).
- `.temp/brief-653/native-bars/comparison.json`,
  `.temp/brief-653/instruments/measurement-contract.json` — the D shape and
  panel contract that may not move.

## Established by the director (reproduce before doubting)

1. My build of the eight-file F3+E2 change set from `git archive HEAD`
   reproduces **29/29** base own pairs and incoming-call pairs, with F7
   616/600 against base 856/856 (`fd3148ec…`, `.temp/director/verify-655/`).
2. The five failures are real and reproduce at `--test-threads=1`:
   `lp06_compound_items`, `lp06_status_projection_value`,
   `lp06_while_body_continue`, `lp655_root_context_falsifier`,
   `lp655_deferred_enable_context_falsifier`.
3. Removing `owner.generation == owner.control.logical_generation()` from
   `segments::current_owner` does **not** fix any of the five. The missing
   piece is the session identity itself, not that predicate.
4. The sessionful candidate passes the three LP06 tests; the difference is
   `Call::start`'s owner-match-or-root choice against an actual session.
5. F1/F2/F3/F3+E2 all reach 29/29 exactly; E1 alone moves only the eight
   `try_apply_function` rows. The 21-row residual is the six start call
   shapes. The completion methods and the four forwarded starts are
   measured-neutral on all 29 rows and inside the D native bars.

## Questions to answer

- **Q1 — the diagnosis.** For each of the five failures, name the exact
  start whose `current_owner()` returns `None`, the state that makes it
  `None` (no active segment, closed path, invalid epoch, no matching owner),
  and what `Call::start(session, …)` does in that same state (owner match,
  parent attach, or root). Cite `path:line` on both the live and the probe
  trees.
- **Q2 — the context.** The exact type and ownership of the execution
  context: what it holds (the control `Arc`, its generation, a root id, or
  less), where it is installed and popped, and how it survives suspension,
  resume, panic/unwind and thread handoff. Name every registration site with
  `path:line` — every guest-evaluation entry, the resume entry, and every
  gate-on transition — and prove each is reached only when the profile is
  configured or enabled. No borrowed `Session` in thread-local storage, no
  raw pointer, no `unsafe`.
- **Q3 — the match rule.** The exact predicate that reproduces
  `Call::start`'s two cases: match the active owner against the context's
  control and open a child; otherwise create a root with the context's
  control and a fresh root id. State what makes a root a root, how repeated
  root creation inside one evaluation is avoided or bounded, and how the
  rule behaves when the context is absent (disabled or stale).
- **Q4 — the disabled path.** The exact added work on the disabled path. If
  a gate check is added at an evaluation entry, show it is the established
  cold-load pattern, one relaxed load plus a branch per evaluation, with no
  TLS access when not configured; and measure that no row of the 313-label
  panel and none of the 29 own slopes moves.
- **Q5 — the transitions.** A disposable falsifier per state, each with its
  exact body, red on F3+E2 and green on the design arm: root (a real
  reachable path, not only a direct helper call), deferred enable, resume
  for all three LP06 tests, reentrancy, two sessions on one thread, thread
  handoff, reset and off-to-on/on-to-off gate transitions. State the
  expected records for each, not only `calls > 0`.
- **Q6 — the shape gap.** Confirm that the calibrated surface is unchanged:
  the six phase-only starts, borrowed completions, the by-value adapter only
  at `try_apply_function`, and the E2 dispatch body. State the remaining
  shape gap honestly in the unpassed list; do not claim base shape for the
  completions or the forwarded starts.
- **Q7 — R5.** If a transition cannot be made green without a new
  per-operation default-path cost, `unsafe`, a shared queue, or a lowered
  bar, STOP with the mechanism and counter-evidence. Do not invent a waiver.

## Scope

**NONE for the live tree.** No live source, workload, hash-manifest,
inventory, docs, roadmap or version-control change. All work under
`.temp/brief-656/`: export `HEAD` with `git archive` into your own probe
root, apply the eight-file F3+E2 surface, build with the supplied recipe
(release, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, jobs=8, offline, locked,
one common source path), install each arm's binary, and measure with the
supplied runner serially, alone, on CPU 2, with every `RUM_PROFILE_*`
cleared. The supplied package is read-only; copy it before writing. Prior
roots (`.temp/brief-648/`, `.temp/brief-653/`, `.temp/brief-654/`,
`.temp/brief-655/`, `.temp/director/calib-655/`,
`.temp/director/verify-655/`) stay read-only and are never executed in
place.

**Do not design a shared diagnostic queue cursor.** Do not make the
completions or the forwarded starts session-free. Do not add a new module,
a feature, a public API, a serde/type-shape change outside
`pub(in crate::runtime)`, or `unsafe`.

## Deliverable

A `rum-report/1` report at `.temp/brief-656/report-design.json` with:

- the Q1 diagnosis per failure with `path:line` on both trees;
- the context design: type, ownership, registration/pop sites, guard and
  Drop semantics, suspension/resume/handoff carriage, reset and gate
  transition behavior, and the Q3 match rule;
- the probe receipts: exact commands, per-arm binary hashes, the 29 own and
  incoming pairs for base, F3+E2 (red) and the design arm (green), the 313
  panel rows, F7, and every D native/stack bar measured;
- the falsifier matrix: each of the five existing failures plus the Q5
  transitions, with the red F3+E2 result and the green design-arm result;
- the drafted implementation brief in `TEMPLATE.md` shape with the exact
  bars;
- an explicit unpassed list that keeps the completion/forwarded shape gap.

Run `python3 -I scripts/report.py check 656` and return its summary and every
compliance verdict line.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, live source edit, workload edit or prior-evidence mutation.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.
- One variable per probe; a single Callgrind pair never decides a
  seed-dependent quantity; state the distribution when a quantity needs one.
- A design that pays for the context with a new per-operation default-path
  cost, a wider public API, a compile-time gate or a lowered bar is not a
  candidate.
- Every claim carries a citation the checker can resolve.
