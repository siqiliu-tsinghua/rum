# Brief 642 — LP05: assignment RHS, finish, list, association and LHS phases

**Status:** implementation brief, staged. Director-written from the
brief-624 design draft (`.temp/brief-624/drafts/LP05-assignment-phase-boundaries.md`);
the draft is raw material, this file is the authorization.
**Sequence:** LP01 (`3fd1885c`), LP02 (`17576575`), LP03 (`e924bb4f`), LP04
(`brief-638`, gate GREEN). LP04 established the adopted-record mechanism this
brief reuses; read its final gate and its shape rules.
**Base:** `ab3c1c89` (the tree this brief was written on). The base source
export for the measurement stage is `.temp/brief-642/base/` (git archive of
that commit, 2,582 files).

## Goal

Adopt the assignment routes `set-rhs`, `set-finish`, `set-list`,
`set-association` and `set-lhs` — phase strings `set.rhs.evaluate`,
`set.rhs.suspend`, `set.finish`, `set.try_list_destructuring`,
`set.try_association_lvalue`, `set.prepare_lhs` — so that one logical
assignment produces exactly one record per route it actually enters, opened
at the legacy start site and closed at the route's own consumer boundary,
across the first (RHS) and, in the same operation, the second (LHS) pause.

The assignment phases are the first adopted cells whose continuation spans a
real suspension: the RHS may pause before the LHS begins, and the LHS
preparation may pause after the RHS resumed. The record must follow that
continuation: a pause is telemetry, not a terminal call and not a second
call, and a resumed operation must complete the one record it opened.

Nothing about evaluation changes. Values, errors, traces, output, the legacy
recorders, `phase-sites.tsv` and every assertion contract stay exactly as
they are. The disabled path gains no work beyond the relaxed loads it
already pays.

The draft's five shape decisions are binding:

1. `set.rhs.evaluate` and `set.rhs.suspend` end **before** assignment. On the
   suspend branch both are completed at that boundary, before
   `EvalSetSuspensionFrame` consumes the RHS; `set.rhs.suspend` is the
   forwarded-pause boundary, not a second evaluation.
2. `set.finish` begins on the resumed RHS-success branch where the original
   `definitions.rs:64` start was bypassed, then is retained across the
   assignment's own pauses and completes at the complete assignment's result.
   An errored RHS fabricates no `set.finish` record.
3. The list and association targets' completion carries `Expr`, and logical
   success is `Some` because the applicable branch has already committed. Do
   not add an `OptionalExpr` decoder to these `Expr` frames.
4. `set.prepare_lhs` spans the integrated
   `DefinitionLhsPreparationContinuation::Set` remainder and completes after
   the prepared assignment/store, with concrete recovery/error behavior
   unchanged. It is not complete merely because an intermediate LHS structure
   was prepared.
5. Leave symbol own-value/protected/store/trace/epoch and
   `set.normalize_target` pure intervals synchronous. Promote only the six
   adopted phase strings; `builtin-total` and every other remaining cell keep
   their current owners.

## Context pointers

- `.temp/brief-624/DESIGN.md` — the contract. Read §"Contract" (lines 8–96)
  and §"Disabled-path gate and measurements for every implementation step"
  (265–328). The route table is
  `.temp/brief-624/inventory/routes.tsv`; the site list is
  `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/phase-sites.tsv`
  (the `set-*` route column is authoritative for which sites belong to this
  brief).
- `docs/dev/codex/brief-638-lp04-definition-rhs-matching.md` — the shape
  rules, the inventory replay, the falsifier pattern and the measurement
  campaign. Reuse its procedure; do not re-read the LP03 brief unless the
  engine's `open_phase`/defer-pause surface is unclear.
- The engine: `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`
  (`PhaseKey` 30 variants, `open_phase`, `complete_phase`,
  `complete_logical_expr_defer_pause`, `pause_phase_at_finish`,
  `ProfileBoundary`, `finish_logical_*`, `PhaseCompletion`) and
  `profiling/timing.rs` (`runtime_path_timing_start_adopted`, the
  `record_profile_runtime_path_timing_adopted*` family).
- The oracle: `crates/rum-stdlib/tests/suspension_oracle/` —
  `fixtures/logical_profile/mod.rs`, `support/logical_profile.rs`,
  `inventory/logical-profile-required.tsv`,
  `inventory/logical-profile-bindings.tsv`,
  `inventory/logical-profile-losses.json`. `P0413`/`P0431` are the
  `Set.destructuring` rows the draft names.
- My falsifier, calibrated red at `ab3c1c89`:
  `.temp/brief-642/falsifier/lp05_assignment_rows.rs`, with the two-pause
  probe log `.temp/brief-642/falsifier/pause-probe.log`.

Every line number in this brief was verified on `ab3c1c89`. A line that has
moved by the time you read this is located by the phase string and the route
rule, never by the number alone.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` — append the six
  phase keys and their `name()`/`from_name()` arms; wire the new unit-test
  module at the foot. No other change to this file.
- `crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs` — one hunk
  only, by the A2 Q1 decision below: the `debug_assert!` in
  `logical_insert_boundary` admits `Policy::ApplicableExpr` beside
  `Policy::Expr`. No other line of this file changes.
- `crates/rum-runtime/src/runtime/profiling/timing.rs` — **only** if a route
  genuinely needs a new adopted helper or a signature it already lacks; state
  why in the report. Prefer the existing family.
- `crates/rum-runtime/src/runtime/definitions/definitions.rs`
- `crates/rum-runtime/src/runtime/definitions/definitions/resume_frames.rs`
- `crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs`
- `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`
- `crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/assignments.rs` (new; your
  unit tests)
- `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs` — the
  `lp03_adopted_site_table_is_exact` test only. Add exactly the six new names
  to its `expected` set; no other line changes, no existing name removed.
- `crates/rum-host/tests/lp05_assignment_rows.rs` (new; my falsifier,
  inserted verbatim from the `.temp` copy)
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/mod.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`
- the standing generated inventories named under **Inventory replay**.

Files and areas you MUST NOT touch:

- the legacy recorder bodies and their call sites: `runtime_path_timing_start`
  for the six adopted strings stays, converted to its adopted form; every
  other legacy recorder stays byte-for-byte;
- `fixtures/logical_profile/phase-sites.tsv` (no edit at all);
- the identity columns of `logical-profile-required.tsv` (first 16), the row
  count 1115, the binding count 875 and the frozen fingerprint
  `fnv1a64:b41ac6a5dbf0e9a4`;
- `Expr`/`Symbol` representation, any public type or serde shape, codecs,
  guest encodings, semantics, error values, trace output, `crates/rum-core`;
- `AGENTS.md`, `TODO.md`, `Cargo.lock`, other briefs, earlier scratch.

## Shape rules (read twice; a departure is a STOP)

1. **One record per logical invocation.** A native reentry of the same
   operation is not a second invocation; a distinct recursive invocation, a
   distinct operand/iteration or a later stage is.
2. **Start exactly at the legacy start site** of the same phase string, by
   converting that call to the adopted form. Every start for each of the six
   strings is listed in `phase-sites.tsv`; an unconverted start for an
   adopted string is a STOP.
3. **Completion only at the route's named boundary**, with the route's own
   carrier (`Result<Expr, _>` for the evaluated and finish routes; the
   route's actual result for the attempt routes; `Expr` for the completion
   shapes). A record must never observe a carrier the route does not name.
4. **A pause is telemetry, never a terminal call and never a second call.**
   The logical contract: `InputRequired` is not an error; at quiescence
   `calls = ok + err`, `open = 0`, `abandoned = 0`. The phases that span a
   pause retain their record through the suspension frame and complete it
   once on the resumed continuation; do not leave it open, do not reopen it
   on resume, and do not count the pause as an err. Use the engine's boundary
   mechanism; a new mechanism is a STOP with the reason.
5. **The RHS routes end before assignment.** `set.rhs.evaluate` completes at
   the RHS evaluation result; on the suspend branch it is completed there,
   before the frame consumes the RHS. `set.rhs.suspend` is the forwarded
   pause: opened before the frame, completed at the frame's resume match
   (`definitions/definitions/resume_frames.rs:21`) before
   `resume_set_with_rhs`.
6. **Finished exactly once.** After a completed program
   `Session::logical_profile_test_assert_quiescent()` passes and no row has
   non-zero `open` or `abandoned`.
7. **No default-path cost.** All new work is behind the existing cold-load
   gate; the slow path is `#[cold] #[inline(never)]`. No TLS lookup,
   allocation, `Mutex`, `Arc` clone or capture construction on the disabled
   path. No new `if token.is_some()` call-site branches.
8. **Keys are aggregate, never identity.** Runtime routes use
   `AggregateKey::Runtime { phase, source }` with the phase string exactly as
   in `phase-sites.tsv`. Append the six keys; `from_name` is index-based, so
   reordering would silently remap every earlier key.
9. **Legacy recorders stay.** The legacy output, timing stack and
   `phase-sites.tsv` do not change.
10. **No semantic edit.** If a change would alter whether, when or with what
    value guest code runs, it is a STOP, not a cost fix.
11. **No `.expect`/`unwrap` on a state the types allow**, in the code you
    touch.
12. **When the exact consumer boundary cannot be named, STOP and report the
    consumer you traced.** Do not approximate a completion site.

## Routes

Sites verified on `ab3c1c89`; the phase strings and route column are in
`phase-sites.tsv`.

| Route / phase keys | Sites | Boundary |
|---|---|---|
| `set-rhs`: `set.rhs.evaluate`, `set.rhs.suspend` | `definitions.rs:29`, `:31` (self-evaluating atom branch); `:35`, `:41` (evaluated branch) | Both end before assignment. `evaluate` on the RHS evaluation result; `suspend` at the frame resume match (`resume_frames.rs:21`), before `resume_set_with_rhs`. The atom branch records both as `Ok` synchronously. |
| `set-finish`: `set.finish` | `definitions.rs:64`; on the RHS-suspended path the resumed branch in `resume_frames.rs:21-25` | After the complete assignment: the result of `resume_set_with_rhs` (`definitions.rs:167`), including its LHS preparation/store and every pause. An errored RHS fabricates no record. |
| `set-list`: `set.try_list_destructuring` | `definitions.rs:317`; the suspension frame `definitions/destructuring.rs:129`; completion `:139` | The route's final `Expr` on the resumed branch. The applicable branch already committed, so logical success is `Some`; no `OptionalExpr` decoder on the `Expr` frame. |
| `set-association`: `set.try_association_lvalue` | `definitions.rs:328`; frames `association_data/resume_frames.rs:6`, `:38`, `:63`; completion region `association_data.rs:715`, `:749` | The route's final `Expr` on the resumed branch; logical success is `Some` on the applicable branch; no `OptionalExpr` decoder. |
| `set-lhs`: `set.prepare_lhs` | `definitions.rs:343`; frame `definition_lhs_preparation/resume_frames.rs:155`; continuation `definition_lhs_preparation.rs:141-144`; completion `definitions.rs:355` | The final `Expr` result after the prepared assignment/store, spanning the `DefinitionLhsPreparationContinuation::Set` remainder across pauses. Not the intermediate prepared structure. |

Rules that are easy to get wrong, named:

- **`set.finish` is not the LHS preparation.** `set.prepare_lhs` measures the
  integrated LHS remainder; `set.finish` measures the whole assignment and
  encloses it. The two may complete in the same resumed continuation, in
  inner-first order. Preserve the LIFO order the engine already enforces.
- **The attempt routes may decline.** `set.try_list_destructuring` and
  `set.try_association_lvalue` are called on every non-delayed Set after the
  symbol-own-value branch; a declining `None` is still a completed record
  with `None` classification, not a missing row.
- **`set.try_symbol_own_value*`, `set.normalize_target`,
  `set.store_definition` and the symbol own-value sub-phases are not
  adopted.** Leave them synchronous and legacy.
- **The double pause is real.** `f[StringLength[InputString["l"]]] =
  StringLength[InputString["r"]]; f[3]` pauses on the RHS and again on the
  LHS at `ab3c1c89` (probe log). Both records must survive it.

## Engine additions (as code)

Append six variants to `PhaseKey` and to `PhaseKey::ALL` (which becomes
`[PhaseKey; 36]`), in this order:

```rust
    SetRhsEvaluate,
    SetRhsSuspend,
    SetFinish,
    SetTryListDestructuring,
    SetTryAssociationLvalue,
    SetPrepareLhs,
```

`name()` arms, same order:

```rust
            PhaseKey::SetRhsEvaluate => "set.rhs.evaluate",
            PhaseKey::SetRhsSuspend => "set.rhs.suspend",
            PhaseKey::SetFinish => "set.finish",
            PhaseKey::SetTryListDestructuring => "set.try_list_destructuring",
            PhaseKey::SetTryAssociationLvalue => "set.try_association_lvalue",
            PhaseKey::SetPrepareLhs => "set.prepare_lhs",
```

`from_name` arms, indices 30–35, after the existing 30. Append only.

If a phase string turns out not to be reachable from the sites the
`phase-sites.tsv` route column names, STOP with the string and the sites you
traced; do not invent a site.

## Inventory replay (standing duty; last step of the edit stage)

Follow `brief-638`'s §Inventory replay exactly, in this order, after the
edits compile:

1. **Retention/contract replay** per `TEMPLATE.md`: regenerate the local
   contract fingerprints and the certificate cells by replaying
   `local_production_acceptance` in both builds, and reconcile the ownership
   rows. Disclose every changed row. New `UnknownConstruct` rows for the new
   record-holding locals and match arms are expected; a new
   `UnknownHostPayload` row is a STOP.
2. **LP01 producer replay.** Recompute column 19 of
   `logical-profile-required.tsv` for every row whose producer file you
   edited, with LP01's own `fingerprint` function; update column 17 and the
   matching `logical-profile-bindings.tsv` column 3 only where a statement
   actually moved. The phase strings, the first 16 columns, the row count
   1115, the binding count 875 and the frozen fingerprint must not change.
3. **Loss-ledger replay.** Regenerate `logical-profile-losses.json` with the
   pin procedure, extend `verify_losses.py` (copy it into
   `.temp/brief-642/`) and make it print **all** of these **PASS**, keeping
   the existing receipt check:
   - **queued-unchanged** — for every `(mode, program, key)` in both ledgers,
     the queued vector is identical;
   - **mode-confinement** — the `lexical` and `timing` rows are identical
     old to new at the producer positions this brief moved;
   - **mass-never-up** — per `(mode, program)`, the L1 disagreement mass over
     its loss rows is `<=` the old ledger's; print lower/equal/higher counts
     and both totals;
   - **executed-receipts** — the receipt count is unchanged.

   Re-derive the bar at `ab3c1c89`; the LP04 gate's last recorded bar was
   queued 77,564 shared / 0 changed, mass 610 lower / 2,574 equal / 0
   higher, 4,375 receipts, 714 zero-offset vectors. Any increase under
   **mass-never-up** is a STOP. LP05's own cells may move only under these
   checks. Never weaken one of them to reach green.
4. `cargo test -p rum-stdlib --test suspension_oracle` must be green in both
   the default and `--no-default-features` builds.

## Falsifier (director-written, calibrated red)

Insert `.temp/brief-642/falsifier/lp05_assignment_rows.rs` verbatim at
`crates/rum-host/tests/lp05_assignment_rows.rs`. It asserts, for six
completed/paused programs, that each adopted row exists, is closed exactly
once (`calls = ok + err`), is not abandoned and is not open; the paused
programs must demand their inputs in order and record at least one pause.

Calibration at `ab3c1c89`, with the switch on:

```
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp05_assignment_rows
set.rhs.evaluate missing for `x = 1; x`: [ ... the 15 LP03 rows ... ]
```

The row dump is not empty: every LP03/LP04 route that ran is present, and
only the `set.*` routes are missing. The two-pause program was separately
probed at HEAD and observed two `InputRequired` outcomes before completing
(`.temp/brief-642/falsifier/pause-probe.log`). The falsifier must be green
after LP05 and is never weakened to reach green. If it fails for a route you
implemented but could not make reachable, that is a STOP with the program and
the consumer you traced.

## Verification — Stage A1 (engine keys, set-rhs, set-finish)

Run and paste verbatim:

```
cargo fmt --check
cargo test -p rum-runtime --lib profiling --offline
cargo check --workspace --all-targets --offline
```

Your unit tests in `profiling/tests/assignments.rs` mirror
`profiling/tests/evaluator.rs`: a row finder over
`Session::logical_profile_test_rows()`, an ordinary completed Set, a
self-evaluating-atom Set, and an RHS-paused-then-resumed Set. Each must show
one record per entered route, `calls = ok + err`, no `open`/`abandoned`, and
a pause recorded without a second record. Watch each new assertion fail
before it passes and paste both.

The 48 brief-624 correctness programs must stay semantically green with
profiling off and on (`.temp/brief-624/correctness/`; copy any runner into
your scratch, never write earlier evidence).

**Stop after A1 and report.** A1 is the engine plus the RHS pair and
`set.finish`, ordinary and RHS-paused. Do not start A2 in the same
continuation.

## Verification — Stage A2 (the attempt routes, set-lhs, the double pause)

Run and paste verbatim:

```
cargo fmt --check
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp05_assignment_rows --offline
cargo test -p rum-stdlib --test suspension_oracle --offline -- logical_counter_port
cargo test -p rum-stdlib --test suspension_oracle --offline
# the same oracle command with --no-default-features
```

Then the inventory replay. The **testing tier (user ruling 2026-11-01)**
applies: while editing, `cargo check --workspace --all-targets` plus the
touched crate's `-p` tests plus the named test is the gate.

The two full workspace suites belong to the director's gate: the dispatcher's
sandbox cannot safely own a workspace-sized OOM domain, so do not run
`cargo test --workspace` at all; the gate runs both builds at 14G and eight
threads. Do not add a global `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` to any
suite; only the falsifier and the host observers run under the switch.

The oracle's `all` and `timing` modes are the authority for the adopted
cells: every adopted `set.*` cell must be SAME and every previously adopted
cell must stay green.

**Stop after A2 and report.**

## Verification — Stage B (measurement campaign; only after A2 is gated)

Read `.temp/brief-624/DESIGN.md:265-328` and run its campaign with the arms
supplied here:

- **base** = `.temp/brief-642/base/` (git archive of `ab3c1c89`, already
  verified clean);
- **candidate** = copy your edited tree to `.temp/brief-642/cand/`,
  excluding `.git`, `target` and `.temp`, at the same relative depth as the
  base copy.

Build both arms offline from those copies into distinct target directories
with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`; profiling OFF; exact N/2N
sources; `/dev/null` stdin; serial CPU work; bounded subprocesses. Measure:

- one-codegen-unit instruction slopes for all 192 ordinary and 121 resume
  labels (`.temp/brief-624/performance/labels.json`), with the
  assignment-heavy labels named explicitly in the report:
  `set-ownvalue`, `lhs-set-deep-path`, `lhs-set-pattern`,
  `association-set`, `association-part-set`, `appendto-list-incremental-*`,
  plus each label's same-tier neighbours;
- `scripts/core-language-microbench.sh` rum-vs-rum per-label `timings.tsv`
  on those labels and their same-tier neighbours, profiling OFF;
- the copy/allocation census on all 192 ordinary labels (brief-622 reducer);
- the 22 debug/release stack shapes (44 zero-residual fits).

The disabled path must not move: the existing gate is the bar. Any per-call
increase on an ordinary label is a defect to attribute or report, never
noise. An external row (`__memcmp_*` and the like) is recorded, not charged,
but the external class lapses on any non-zero rum-side row in the same
table (LP03's ruling). Missing perf/stack/copy access leaves that gate
unpassed; no elevation and no wall-clock surrogate.

## Stop conditions and checkpointing

- STOP and report (do not improvise) if: a route's consumer boundary cannot
  be named; a record would need a carrier its route does not name; a change
  would alter guest semantics or a concrete error; a paused record would be
  counted as a terminal call or reopened on resume; the identity columns of
  the required inventory would move; mass-never-up fails; a new
  `UnknownHostPayload` row appears; the falsifier fails for a route you
  believe you implemented; or a falsifier route cannot be made reachable.
- Write `.temp/brief-642/PROGRESS.md` after every stop and every green stage:
  what is done and verified with raw paths, the exact next action, the tree's
  compile/test state, changed files, and the counters (adopted phases, route
  coverage, receipts, losses, ledger mass).
- Stop-loss is keyed on progress: two consecutive attempts that advance none
  of the brief's counters end the stage with a takeover checkpoint.

## Hard rules

- No version-control command in any form; leave the tree for the director.
- No network, no new dependency, no `unsafe`, no default-path cost.
- Keep every legacy recorder and `phase-sites.tsv` untouched.
- Timing, perf and callgrind runs are serial and alone, profiling off.
- Paste real output; say plainly what did not run.
- The report is `rum-report/1` at `.temp/brief-642/report-a1.json`,
  `report-a2.json`, `report-b.json`; `python3 scripts/report.py check 642
  --stage a1` (etc.) must pass before each stage's final message.

## Report format

Final message is exactly `python3 scripts/report.py summary 642 --stage a1`
(etc.). The report lists every run with its command, log, exit code and test
counts; `changes` lists every touched file with line ranges and the reason;
`disclosures` lists every inventory row that moved; `questions` carries any
STOP.

## Director gate — stage A1 (2026-11-03): PASS

`director-gate.py run 642 --stage a1`: scope (report check OK, 122
citations), fmt, check (3 warnings, 0 new), ownership (61 / 79 / 8,001, 0
rounds), census (1067 = 10/1022/35, every row paired), oracle (1 passed),
standing (LP01 1/0, typed-slot 10/0), fingerprints (68 contracts, 0 changed),
certificates (both builds), inventories (19 passed; frontier 1067) all PASS.
The workspace step was not run at this stage by the per-brief testing tier;
it runs at the A2 gate.

Reproduced by hand: the ten focused tests were red on the base tree with the
missing `set.*` row in every case (`a1-assignments-red-3.log`) and green
after the wiring (`10 passed`).

**Open at this gate, carried to brief-643 before A2.** My rerun of
`cargo test -p rum-runtime --lib` at eight threads hit an intermittent red in
`profiling::logical::budget::logical_expired_queue_arena_stays_charged_until_
weak_drop` (`parked - initial` underflow at `budget.rs:176`): 3/20 red with
the ten new tests, 0/20 with `--skip logical::assignments::`, and green at
`--test-threads=1`. The mechanism is the 4,232-byte TLS registration charged
by `segments::install` (`size_of::<RefCell<Stack>>() + 4096`): libtest runs
each test on its own thread and that registration is released only at thread
destruction, after the lease has been handed on. A scratch probe that
releases it under `TestLease::drop` (test-gated) gives 20/20 green, and its
falsifier is red on the unfixed tree with `left: 4232, right: 0` and green on
the fixed one. No production behaviour change; brief-643 owns the repair.

## Director decision — A2 questions (2026-11-03)

The A2 preflight STOP was correct: neither proposal had been applied, and the
tree is untouched. Both questions are answered; proceed with A2.

- **Q1 — authorized, narrowly.** `crates/rum-runtime/src/runtime/profiling/
  logical/queue_api.rs` has joined Scope for exactly one hunk: the
  `debug_assert!` in `logical_insert_boundary` admits
  `Policy::Expr | Policy::ApplicableExpr`, as
  `.temp/brief-642/a2/queue-applicable-policy-proposal.patch` shows. The
  assertion stays a `debug_assert!`; the owner, generation, frontier and
  memory-owner checks are unchanged; no other line of the file changes.
  `ProfileBoundary::observe` already decodes
  `(ApplicableExpr, Slot::Value(_)) => Some`, which is the attempt routes'
  applicable-success classification, and every route frame returns `Expr`;
  the guard was the only obstacle. Do not add an insertion bypass or a
  second mechanism.
- **Q2 — authorized, formatting only.** The canonical `.temp` copy
  `.temp/brief-642/falsifier/lp05_assignment_rows.rs` has been replaced with
  the rustfmt-clean bytes; the only difference is the four-element `phases`
  array wrapped with a trailing comma. I verified the two differ by that
  layout alone — programs, assertions and literals are token-identical — and
  `rustfmt --check` passes. Insert that file verbatim, as Scope says.
- Everything else stands: the acceptance commands, the inventory-replay
  order, and the stop after A2.

## Director gate — stage A2 (2026-11-03): GREEN

`python3 -I scripts/director-gate.py run 642 --stage a2` (log
`.temp/director/gate-642a2/`):

| step | verdict | detail |
| --- | --- | --- |
| scope | PASS | 274 citations verified (`report-a2.json`) |
| fmt | PASS | formatted |
| check | PASS | 3 warnings, 0 new against HEAD, 0 gone |
| ownership | PASS | 61 self-reaching types, 79 `UnknownHostPayload`, 8,006 `UnknownConstruct`; 0 replay rounds |
| census | PASS | 1,067 `{RET001: 10, RET002: 1,022, RET003: 35}`; paired 1,067, retired 0, new 0 |
| oracle | PASS | 1 passed |
| standing | PASS | LP01 1/0; typed-slot 10/0 |
| fingerprints | PASS | 68 contracts, 0 changed |
| certificates | PASS | both builds pass, 0 promoted rows merged |
| inventories | PASS | 19 passed, 0 failed |
| workspace | PASS | 62 binaries, **4,059 passed / 0 failed / 61 ignored**, peak 7.0 GB of 14, `oom_kill` 0 |

**The first workspace run failed, and it was the gate's own gap.** The LP05
host falsifier records rows only under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`;
the gate's `SWITCH_GATED` list knew LP03 and LP04 but not LP05, so the
ungated run saw empty rows (`set.rhs.evaluate missing for x = 1; x: []`).
The test name is now in `SWITCH_GATED` (`417fb11c`), matching the LP03/LP04
entries; the rerun from `--from workspace` is the PASS above.

Reproduced by the director under the switch, on the tree:

    RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp05_assignment_rows
    test lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs ... ok
    test result: ok. 1 passed; 0 failed; 0 ignored

Log `.temp/director/gate-642a2/director-falsifier/run-1.log`; peak 24 MB.
The red direction remains the A2 preflight calibration at `ab3c1c89`, where
the same file fails with the missing `set.rhs.evaluate` row.

A2 is accepted. Stage B (the measurement campaign) is next.
