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

## Director gate (2026-11-11) — brief 726 accepted: rule (b) is the ownership repair, and brief 727 implements it

Run: log `.temp/logs/cdx_brief_726.log`, 52,678 lines, sha256
`0aef79f62f4f73852feff053c3300492c9199d56b7e216753c0a78c14fab3db2`,
memory-scope peak 2,601,328,640 B, oom 0, oom_kill 0, rc=0; report
`.temp/brief-726/report.json`, 118 runs, 14 PASS / 3 FLAGGED / 1 NA,
438 citations (my own `report.py check`: OK). The live tree was not
touched: git clean at `b0e62cf4`, no file outside `.temp/`, `target/`
and `.git` newer than the run start, the C7–C11 and brief-623 scenes
show no file newer than the run start, and every one of the 41,102
`changes` entries is under `.temp/brief-726/` (the sanctioned
`.temp/g15-c2/probe/` cache is the sole disclosure). The dispatch log
contains no git or network command; only the worker prompt's own
prohibitions mention git.

**My reproductions.** The frozen red panel is exact against
`.temp/g15-c2/c11/loss-proof.json`: for all ten cases the resumed and
queued rows and `native_same` compare equal to the C11 records; the
three DIFF are exactly `direct.two`, `replace.two`, `unheld.two`;
semantic SAME is 10/10; the three `C11_PAUSE_MISMATCH` lines are
byte-identical to C11 (`RecordId(238)/SegmentId(333)`,
`4474/4583`, `7923/8032`), and the red-state log carries
`epoch=216 valid=false reason=InstrumentationFault`. The red arms'
record files `red-real-records.json`, `red-disabled-records.json` and
`red-state-records.json` are one sha256 (`fecd02bd…`), so real ==
disabled is file equality, not a claim. `candidate2-records.json` and
`candidate-disabled2-records.json` are one sha256 (`4b587f62…`).
Candidate2 changes exactly the three failing resumed arms (each now
equals its queued arm) and leaves the seven controls and all ten
queued arms byte-identical; semantic SAME is 10/10. The omission arm
`omission2-records.json` is sha-identical to the frozen red records,
with the raw `B726_ARM`/`B726_HIT` lines, and the restore is green.
The forced spot check rebuilt the red-real source to `bf9d9e07…`,
exactly continuation 11's forced spot-check sha, so the base arm is
the accepted live source. The 120 original differences are all
present in `original-120-review.json` as "unbound discovery evidence;
not a C1 loss", and the C11 source artifact hashes to its frozen
value `f2f3b297…`. The candidate2 tree differs from the base copy in
exactly `segments.rs`, `segments/stack.rs`, the new test module and a
test-only `B726_EPOCH` print in `mod.rs`; no diagnostic marker is in
a production file. The class table has 18 rows (14 `closed`, 4 `out
of class`, 0 open) and its ten direct finish/pause sites match my own
grep of the live source (`completion.rs:189`, `mod.rs:140/189`,
`segments.rs:685/692/704`, `context.rs:393`, `driver.rs:156`,
`profile_mode.rs:52`, `mod.rs:891`). The disabled-cost table has the
twelve functions at unchanged calls and Ir, the honest
inlined-`Epoch::rows` attribution (40 calls, 5,680 Ir both arms and
both draws), and the `Vec<ActiveSlot>` destructor at 1 call / 24 Ir in
both draws; no exclusion is declared. `report.py check` verifies all
438 citations.

**The one red I do not take as calibrated.** The permanent unit
falsifier is red on the pre-fix source, but it fails at the helper's
`open().unwrap()` (`brief726.rs:10:114`) after the first pause faulted
`epoch=1` with `InstrumentationFault`, not at a named validity or
fault-count assertion. The mechanism is the intended one; the
observation is a consequence. Brief 727 asserts `is_valid()` and
`faults == 0` immediately after the repeated pause, before any reuse
open, and requires the red log to quote that assertion failing at 8
and 1.

**Rulings.** (Q1) Rule (b) and candidate2 are accepted as the
ownership repair: one `Rc<SegmentCompletion>` shared by `Active` and
`Segment`, each replacing its existing epoch pointer;
`SegmentState { Live, Ended, ExplicitlyFinished }`; promotion only on
the exact active receipt after an ordered explicit finish while the
epoch is valid; pause inert only for `ExplicitlyFinished`; displaced
live, ended-without-explicit-finish, missing-slot, foreign-owner,
stale-epoch, reuse, cancel and cap checks unchanged; one charged
receipt per install, released on last Drop; no guest or public
carrier. (Q2) Candidate1 is superseded, not waived: its extra Active
pointer moved the generated empty `Vec<ActiveSlot>` destructor from 24
to 23 Ir at one call; candidate2 restores 24/1 exactly. (Q3) The
enabled-path charged allocation is accepted and disclosed; the
disabled path keeps zero calls and Ir, and the inlined reader's
whole-containing-function attribution is the honest form; no
whole-program claim. (Q4) The malformed-answer probe
`B726.class.error` (resumed Syntax vs queued Other, both arms,
candidate native SAME) is a pre-existing semantic observation
mismatch, outside this ownership class; it stays flagged and
unrelabelled, is not a parity certificate and is not part of brief
727's tests; continuation 12/settlement must classify it as an
existing ledger row or a new Lost/residual row. (Q5) The retained
harness and cost failures are recorded, not gated; no assertion was
weakened. (Q6) The design round's standing-replay NA is correct;
brief 727 names and runs all seven families before acceptance. (Q7)
Brief 727 installs the accepted shape, then re-runs the panel,
omission, five modes, prepared16/56, the permanent falsifiers, the
seven replays and the acceptance; continuation 12 resumes only after
727's gate.

**Recorded, not gated:** the test-only `B726_EPOCH` observer print and
the `B726_*` PASS lines stay out of the installed production shape;
the failed harness versions, first cost reducers, failed candidate2
compile tree, retained invalid recipes and all 15 build receipts stay
in `.temp/brief-726/`; the candidate2 test module is renamed and its
validity assertion moved ahead of the reuse open in 727.
