# Brief 726 — G15-C2: design the prepared-consumer context retirement

Stage: **design only**. `Scope: NONE — no source change`. The director
writes the implementation brief after this design gate.

## Goal

Specify, prove in a byte copy, and hand back as an implementation brief
one enabled-only ownership repair for the diagnostic lifecycle loss
recorded as `C10.prepared.profiled_consumer`. A prepared-argument
consumer completes its preparation boundary and then suspends again in
a later dispatch stage. The completed preparation segment must stay
correctly sampled as `None`, and the later suspension must not pause a
segment that boundary completion already ended, must not fault the
epoch, and must not retire live RHS/evaluator records as abandoned.
When you are done the design must name exactly one ownership rule with
its exact API and allowed terminal states, show it green in a copied
experiment against the frozen red panel, and carry a drafted
implementation brief in `docs/dev/codex/TEMPLATE.md` shape.

## Context pointers

- `crates/rum-runtime/src/runtime/eval_core/call.rs:519-531` and
  `:795-816`: the prepared dispatch resume completes the prepared
  arguments, then continues the suspended call dispatch.
- `crates/rum-runtime/src/runtime/suspension/core/profile_queue/prepared.rs:60-80`:
  the prepared boundary completes with `Outcome::None`.
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs:130-157`:
  completion ends the active segment before closing the record.
- `crates/rum-runtime/src/runtime/profiling/logical/segments.rs:521-542`
  and `:704-755`: `pause_admitted` faults when the stored top is not the
  active segment, and `finish_active_admitted` ends that exact segment.
- `crates/rum-runtime/src/runtime/suspension/core/driver.rs:26,155`:
  the context guard stays alive across the frame resume and is paused
  again on its `InputRequired` result.
- `docs/profiling.md:28-44,73-76,98-100`: queued/resumed parity, exact
  boundary sampling, visible invalid pairing, and the rule that an
  invalid epoch never certifies parity.
- The accepted evidence is `.temp/g15-c2/c11/loss-proof.json`,
  `.temp/g15-c2/c11/fault-min-*` and the design draft
  `.temp/g15-c2/c11/DRAFT-prepared-resuspend-context-repair.md`. The
  C11 scene is frozen evidence: read it, never write it.

## Scope

- Files you MAY modify: **NONE in the live tree**. All work lives under
  `.temp/brief-726/` (byte copies, probes, receipts, report).
- Files you MUST NOT touch: the live `crates/` tree, `scripts/`,
  `docs/`, `TODO.md`, `AGENTS.md`, `Cargo.toml`, `Cargo.lock`, `.git`,
  and every existing `.temp/g15-c2/` scene (C7, C8, C9, C10, C11,
  `brief-623`, `probe/` beyond the sanctioned cache).
- No git command in any form. No network.
- Build only through `scripts/probe-build.py` with a complete assembled
  tree under `.temp/brief-726/`; one build per (arm, kind), one
  `--force` sha256 spot check this round under the scene environment
  (`CARGO_HOME=.temp/g15-c2/cargo-home`, the scene `TMPDIR`,
  `RUSTUP_TOOLCHAIN=1.85.0`) — out-of-environment rebuilds differ only
  by the embedded registry path, so the spot check must use that
  environment. Rust 1.85.0, offline + locked, `-j 8`, one Cargo
  process at a time, bounded commands.

## Reproduction you must reproduce first

Freeze these ten cases and their full observations before any
experiment. Setup `c11f[83]:=Input["finish"];`, program
`c11f[Input["rule"]]`, answers 83 then 89:

| id | semantic | native |
| --- | --- | --- |
| builtin.two, direct.prep, direct.rhs, literal.two, replace.prep, replace.rhs, unknown.prep | SAME | SAME |
| direct.two, replace.two, unheld.two | SAME | DIFF / epoch invalid |

Requirements for the red arm: exactly 3 DIFF / 7 SAME, semantic SAME in
all ten, real and disabled-instrumentation records byte-identical, the
profile explicitly invalid in the three DIFF cases, and the exact
`C11_PAUSE_MISMATCH record=... index=0 segment=... top=None
expected_live=None top_live=None` markers. Also re-read all 120
original replacement-attempt differences in
`replace-frames-real-differences.json`: they are unbound discovery
evidence, not C1 losses, and no attempt is deleted or relabelled.

## Design duties

1. **Freeze and instrument.** Write the ten-case panel and its exact
   full observations into the new scene. Add a diagnostic marker that
   connects the prepared boundary's record identity, the captured
   `ContextGuard` segment generation, the explicit completion, and
   every later pause attempt. The existing mismatched-owner, active-slot
   reuse, stale-epoch, cancellation and diagnostic-cap falsifiers are
   retained unchanged.
2. **Class inventory.** Inventory every path where an explicit boundary
   completion (or any explicit `finish_active`) can end a segment still
   held by a live `ContextGuard`, and every path that pauses a captured
   context whose segment may have ended. Include repeated suspension,
   nested callers, `Error`/`Return`/`Throw`, snapshots, cross-thread
   resume, reset/cancel and old epochs. Each row gets a disposition
   (closed by the chosen rule, out of class with a proof, or open with
   the reason). The named instance is not the class.
3. **Choose exactly one ownership rule.** Either (a) explicitly
   transition/detach the guard when its owned record's segment
   completes, or (b) make a pause of a demonstrably completed
   same-generation segment a no-op while still rejecting a displaced
   live record. A third rule needs its own source proof. No blanket
   missing-top suppression, no fabricated guards, no suppressed
   instrumentation faults, no delay of the prepared sampling, no
   reopen/restart of a completed operation, no borrowed epoch. State
   the exact API, ownership transfer, and every allowed terminal state
   before coding. The rule is enabled-only: the disabled path stays
   instruction-identical (the round records the disabled instruction
   row of every touched function).
4. **Prove it in the copy.** Apply the rule to a byte copy; the three
   base failures must become valid and whole-record equal to queued
   execution, the seven controls unchanged, the actual producer's
   `None`/`Some`/error categories intact, and repeated suspensions and
   the five-mode parity must hold. Calibrate an independent omission of
   the new transition: it must restore the failure on a required key,
   with raw arm/hit markers and the exact restored record. A candidate
   that only passes because an invalid epoch is treated as parity is
   not a proof.
5. **Draft the implementation brief.** In `TEMPLATE.md` shape: exact
   source shape as code, the permitted files, every standing inventory
   replay, the permanent falsifier(s) with their red/green calibration,
   the disabled-path cost bar, the acceptance suites, and the stop
   points. The director rewrites it; your draft is raw material.

## Must not change

- No production edit, no live-tree edit, no git, no network, no unsafe.
- The prepared sampling stays exactly `None` for the completed
  boundary; its record is never deferred until after the consumer.
- The existing generation, owner, active-slot, stale-epoch,
  cancellation and diagnostic-cap checks stay as they are; an invalid
  epoch stays invalid and is never used to certify parity.
- Do not weaken an assertion, delete an attempt, relabel an invalid
  recipe, or re-pin an inventory.
- No default-path cost: the repair must be enabled-only, and any
  diagnostic-only state stays out of hot production frames.

## Verification

Design gate (this round):

- The red panel reproduces exactly: 3 DIFF / 7 SAME, ten semantic
  SAME, real == disabled byte-identical, the three explicit invalid
  epochs, and the exact state markers.
- The copied candidate: all ten cases valid and whole-record SAME,
  zero faults, the seven controls unchanged, real == disabled, and the
  queued arms byte-identical.
- The omission falsifier is watched red on the required key and green
  after restore, with raw receipts.
- The class inventory is complete with a disposition for every row;
  the source/Drop argument covers every retained carrier and guard.
- The disabled instruction row of every touched function is unchanged
  (documented per function with its call count).
- No standing inventory, fingerprint, certificate or position is
  edited; a later implementation run replays them, and this design
  round states which replays that implementation must run.

## Checkpointing and stop-loss

Keep `.temp/brief-726/` self-contained: frozen copy, source hashes,
build receipts, raw logs, failed attempts and the report. The stop-loss
is the standing one: two consecutive concrete attempts that gain none
of [proved rule, closed class row, calibrated falsifier, candidate
record] stop the round with the receipts retained. Source-only analysis
does not count against the clock; a compile error you are fixing is not
a wall.

## Hard rules

- All scratch under `.temp/brief-726/`, never `/tmp`.
- Never run cargo outside `scripts/probe-build.py`; never build into
  the live `./target`.
- Bounded commands, no network, no git, no scene writes.
- Report in `rum-report/1` at `.temp/brief-726/report.json` with the
  red/green receipts cited verbatim, the class table, the chosen rule
  with its API and terminal states, the drafted implementation brief,
  and every retained attempt. Run `python3 -I scripts/report.py check`
  on it before you finish.

## Report format

`summary`, `sections` (scope, reproduction, class inventory, chosen
rule, copied proof, omission falsifier, disabled cost, drafted brief),
`runs`, `measurements`, `checks` (PASS/FLAGGED/FAIL with evidence
lines), `questions`, `stop` if the stop-loss fires. Every cited line
must appear verbatim in its log.
