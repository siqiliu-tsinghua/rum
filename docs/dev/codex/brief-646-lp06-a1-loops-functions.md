# Brief 646 — LP06 A1: loop, Compound and callable-value records

**Source stage. One stop.** The design round is brief-645; its plan is
`.temp/brief-645/plan.md` and its route/site/cell inventories are under
`.temp/brief-645/`. This brief is the director's rewrite of the A1 draft; the
draft is raw material, not authority. A2 (builtin and Catch) and the B
campaign are separate briefs.

## Goal

Adopt five LP06 routes as seven appended logical phases — `while-test`,
`while-body`, `compound-item`, `function-evaluate` and `status-evaluate` —
with one record per logical invocation, completing on the produced `Expr`
before the consumer changes phase, consumes `Break`/`Continue`, or absorbs
`Return`; a real input pause transfers the open record and never reopens or
double-counts it. Guest evaluation, legacy rows, frames, the trace and the
standing inventories must not change. This stage carries no performance
claim; the disabled-path cost is gated by the B campaign after A2.

## Context pointers

- `AGENTS.md`, `docs/dev/profiling.md`, `docs/dev/codex/TEMPLATE.md`.
- `.temp/brief-645/plan.md` — the design, especially "A1 attachment design".
- `.temp/brief-645/inventory/current-sites.md` and `.tsv` — the literal
  start/finish lines, carriers and classifiers for every site.
- `.temp/brief-645/inventory/oracle-cells.tsv` and `.md` — the 13 A1 cells,
  their expected records and the assertion contract. Read-only raw material.
- LP05 precedent: `docs/dev/codex/brief-642-lp05-assignment-phases.md` and
  `docs/dev/codex/brief-644-lp05-disabled-path-cost.md` (the recorder shape:
  inline gate, `Some(token)` destructured, bare token to a cold callee).

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` — append the seven
  `PhaseKey` variants and their `ALL`, `name` and `from_name` arms; declare the
  test module. Nothing else.
- `crates/rum-runtime/src/runtime/profiling/mod.rs` — one `mod` declaration
  for the new sibling module. Nothing else.
- `crates/rum-runtime/src/runtime/profiling/control_timing.rs` — NEW; the
  forwarded recorder only.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs`
- `crates/rum-runtime/src/runtime/scoping/callable.rs`
- `crates/rum-runtime/src/runtime/scoping/callable/status_application.rs`
- `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs` — append the
  seven names to the exact site-table assertion; nothing else.
- `crates/rum-runtime/src/runtime/profiling/tests/control.rs` — NEW; native
  intermediate-state tests. Wire it from `logical/mod.rs` as
  `#[path = "../tests/control.rs"] mod control_tests;` so the name `control`
  stays free for a later glue module.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/mod.rs`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/control.rs`
  — NEW; the A1 cells.
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/manifest.rs`
- The standing-inventory regeneration outputs named below, under their
  existing paths, only as the replay commands generate them.

Files you MUST NOT touch: `branching_loops/resume_frames.rs`,
`kernel_data/control_data/nonlocal.rs`, `eval_dispatch/**`,
`profiling/logical/queue_api.rs`, `profiling/logical/epoch.rs`,
`profiling/logical/record.rs`, `profiling/timing.rs`, every semantic frame
type/field, `Cargo.toml`/`Cargo.lock`, `docs/**` except this brief, the frozen
LP01 plan/requirement identities, `phase-sites.tsv`, and every prior brief's
evidence under `.temp/`.

**Written size exception (director, Q3).** `logical/mod.rs` is 33,949
characters. The seven appended keys and their three table arms may take it
over 35k. This is the written table-only exception the code-organization
guideline allows: move no existing type or function in LP06 — a moved
inventory identity is a STOP — and name the future boundary (`PhaseKey` and
its three tables into `logical/phase.rs`) in the report. A2 appends three more
keys under the same exception. Do not split anything else in this brief.

**Director rulings (Q5, Q6).** The test-owned output roots are canonical and
stay unchanged: the oracle's LP01 observation root
(`fixtures/logical_profile/mod.rs` `scratch()`), the certificate cells
(`retention_closure/local/production.rs`), the fingerprint review copy
(`contract_tests.rs`) and the frontier roots are test behaviour, not prior
evidence; do not edit their path constants. This brief's own evidence and
scratch go only under `.temp/brief-646/`. Bounds for the commands below are
60/600/900/1500 seconds; the whole-workspace form was observed at 564 s in
the same shape (`gate-644`), so 1500 s is the settled bound, not an invented
one. Q1, Q2 and Q4 (the Catch guard, the builtin family and the
checked/unchecked harness) belong to A2 and nothing here depends on them.

## Deliverable

### 1. The keys

Append, in this order, after `SetPrepareLhs` (index 35), keeping every prior
index and the existing order:

| Index | Variant | Site string |
|---|---|---|
| 36 | `WhileTestEvaluate` | `while.test.evaluate` |
| 37 | `WhileTestSuspend` | `while.test.suspend` |
| 38 | `WhileBodyEvaluate` | `while.body.evaluate` |
| 39 | `WhileBodySuspend` | `while.body.suspend` |
| 40 | `CompoundItemEvaluate` | `compound.item.evaluate` |
| 41 | `CompoundItemSuspend` | `compound.item.suspend` |
| 42 | `FunctionApplyEvaluate` | `function.apply.evaluate` |

`ALL` becomes `[PhaseKey; 43]`. Function and status share the one
`FunctionApplyEvaluate` key: both current sites already use the identical
phase string, so do not fabricate a `status_function.evaluate` row.
`site()` stays the wildcard `_ => Policy::Expr`; all seven are `Expr` policies.
Append the seven names to the exact site-table assertion in
`profiling/tests/evaluator.rs`; the frozen `ALL`/`from_name` round-trip must
stay exact.

### 2. Converting the five evaluate starts

At each of the five value starts, replace the existing legacy
`runtime_path_timing_start(phase)` with `runtime_path_timing_start_adopted`
at the same statement position, and replace the existing legacy
`record_profile_runtime_path_timing(token, classifier(result))` with
`self.record_profile_runtime_path_timing_adopted_boundary(token, &result)`
at the same record point. The sites, with their literal current lines, are in
`current-sites.md`: `branching_loops.rs` 155/157, 179/181, 358/360;
`callable.rs` 62/64; `status_application.rs` 33/36-38.

- The adopted starter already creates the same legacy token and opens the
  logical call; the adopted boundary recorder feeds the same legacy row with
  the same outcome mapping (`PhaseCompletion for Expr` is the existing
  classifier, `logical/mod.rs:338`). One legacy row per invocation, same
  phase, same source, same outcome.
- The record contract at every converted `Expr` boundary: `Ok` (including
  `False`, `Null` and a residual invalid call) publishes `ok`; `Err(Return)`
  publishes `err + return_err`; `Err(Thrown | ThrownBypassingCatch)` publishes
  `err + throw_err`; `Err(Cancelled)` publishes `err + cancelled`; any other
  `Err` (including `Break`/`Continue` before the loop consumes them)
  publishes `err`; `Err(InputRequired)` transfers the open record and
  publishes no terminal counter. `Some`/`None` cannot occur here.
- On `Err(InputRequired)` the recorder pauses the call and inserts the marker
  at the producer frontier, before the consumer frame is appended. That
  insertion is what makes the resumed completion observe the pending `Expr`
  rather than a mapped `Option`.
- Do not move the `evaluate` call, the `slot_witness` capture, or any
  producer statement. The witness stays captured from the actual
  `applied_result`.
- `Some`/`None` are impossible at these boundaries; do not add a second
  completion path, a `finish` field, or an `Option<Expr>` carrier.
- `function.apply.finish`, `return.eval` and `throw.eval` stay synchronous and
  untouched.

### 3. The three forwarded suspend starts

At the three `.suspend` sites (`branching_loops.rs` 161/171, 185/195,
364/374), replace the legacy start with
`self.runtime_path_timing_start_adopted_forwarded(phase, &result)` at the
same position, before the consumer macro. If the result is `InputRequired`
the starter transfers the open call at the producer frontier before the
consumer frame exists; otherwise the call stays open.

After the macro, replace the legacy record call with the new helper at the
same statement position, keeping the existing local classifier call and its
argument exactly:

```
self.record_profile_runtime_path_timing_adopted_forwarded(
    runtime_profile_suspend,
    &outcome,
    runtime_profile_outcome_for_expr_result(&outcome),
);
```

The helper lives in the new `profiling/control_timing.rs`:

```
impl Session {
    #[inline]
    pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted_forwarded(
        &mut self,
        token: Option<RuntimePathTimingToken>,
        result: &Result<Expr, EvalError>,
        legacy_outcome: RuntimeProfileOutcome,
    );
}
```

- The inline wrapper returns at the existing `lexical_profile_enabled()` gate
  and destructures `Some(token)` itself; the cold, never-inline callee takes
  the bare token. Do not pass the `Option` aggregate to the cold callee.
- On `InputRequired` the starter already moved the call: skip logical
  completion; on any terminal result complete through the existing `Expr`
  phase completion with `legacy_outcome`.
- Make the single original legacy recording call with `legacy_outcome`, once,
  at the unchanged record point. The loop file's local classifier keeps its
  three `.suspend` uses; do not delete or move it.
- Do not reuse `record_profile_runtime_path_timing_adopted_defer_pause`
  unchanged; it passes the `Option` into its cold callee
  (`profiling/timing.rs:585`).

The legacy `resume_or_suspend!` macro returns an `InputRequired` value to this
function; it does not return from it. Every statement after the macro still
runs on the pause path, and the legacy row is still recorded once.

### 4. Function and status

`callable.rs` and `status_application.rs` convert exactly as in §2, at their
existing start/record points. The two rules the design settled:

- The projection path already opens value evaluation only on projection
  success: `StatusFunctionProjectionFrame::resume` calls
  `evaluate_status_function_value` on `Ok`, and an error propagates through
  the `Err` arm without entering it. That existing call is the only start —
  do not add a second start on the resumed value path, and do not open a value
  record during the projection.
- The value boundary is the raw applied `Expr` result, before
  `finish_function_application` absorbs `Return`: `Err(Return)` completes the
  value record as `return_err`, not `Ok`. Do not add fields or finish calls to
  `TryApplyFunctionSuspensionFrame`, `StatusFunctionValueFrame` or
  `StatusFunctionProjectionFrame`.

### 5. What must not change

- The existing semantic frames — `ResumeWhileLoopSuspensionFrame1/2`,
  `ResumeCompoundExpressionFromIndexSuspensionFrame`,
  `TryApplyFunctionSuspensionFrame`, `StatusFunctionProjectionFrame` and
  `StatusFunctionValueFrame` — their fields and their `resume` bodies; no new
  frame, derive or `FrameRootTrace`/`PureFrameAbandon` type.
- Evaluation order, input/output types, `SlotWitness`es, `ContinueToken`s,
  finalizers, captures, guest memory accounting, trace order and every
  producer statement's text.
- Every existing legacy row, source capture and recording point; every
  existing phase string.
- No new `unsafe` in `rum-core`/`rum-runtime`; no public API, serde, or
  `Cargo` change.

## Tests and oracle cells

Implement exactly the 13 A1 rows of
`.temp/brief-645/inventory/oracle-cells.tsv` under the contract in
`oracle-cells.md`:

- Guest cells run through the existing `Fixture`/`Pause`/`Checks::ALL`
  harness; the additive cells are wired into the existing
  `logical_counter_port` child before its legacy exact-ledger comparison.
  Compare the complete selected key map (phase and source), queued against
  resumed; assert required-key presence and exact totals independently; reject
  extra selected keys.
- Observe the new runtime rows in every lexical-enabled child mode
  (`lexical`, `timing`, `cache`, `all`) and assert their absence in
  `disabled`. Do not add a new workspace exclusion; use the oracle child
  modes or the existing test lease.
- Native intermediate-state tests drive one input at a time with the existing
  lease and epoch rows, and assert the open/calls state at each listed
  request; the generic drive loop has no callback, so its final snapshot does
  not prove an intermediate open record.
- Keep the five oracle modes, the frozen 1115/875/240 identity numbers, the
  frozen identity fingerprint, the 4,375 executed receipts, the six LP05
  `ASSIGNMENT_PHASES` checks and every LP03/LP04 control. Publish the new
  cell receipts separately. Do not delete a legacy loss because an adopted
  row now exists.
- If a proposed cell is unreachable or its expected record contradicts the
  engine's actual contract, STOP with the evidence; never weaken an
  expectation silently.

## Falsifiers

Every new check must be shown failing on a candidate-only mutant before it
counts. Produce each mutant as a transient edit, quote the red test line,
restore the tree byte-identically, and quote the green. The mutant may not
touch the base tree or another phase's expected row. Required red
observations, each attributed to its own cell:

1. **Resumed transfer** — suppress the resumed completion of one loop,
   Compound, Function or status record; the cell must fail on that key's
   open/calls/terminal counts, not on a semantic difference.
2. **Duplicate completion** — complete one record twice, or insert its
   boundary twice; the cell must fail with `calls=2` where it expects 1.
3. **Body before `Break`/`Continue` consumption** — complete `while.body.*`
   after the loop consumes the control error; the row must become `Ok` where
   the body cell expects `err` (Break/Continue).
4. **Function before `Return` absorption** — complete
   `function.apply.evaluate` after `finish_function_application`; the row must
   become `Ok` where the function cell expects `return_err`.
5. **Compound item ordering** — defer the first item's completion until after
   the next item starts; the first item's record must be open at the second
   pause, or its counts must move, in the compound cell.
6. **Status start** — open a value record during projection, or start a
   second one on the resumed value path; the projection-return cell must see a
   value row that should be absent, or the projection-value cell must see
   `calls=2`.

A two-input sequence with an uncaught `Throw` before the second input is not a
falsifier; it never reaches the advertised pause.

## Verification (mandatory)

Set `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`, `CARGO_NET_OFFLINE=true`.
One cargo process at a time. Do not invoke `scripts/memory-scope.sh`: the
dispatcher already holds the scope and a nested scope fails. Reproduce a
substantive red once at one test thread before calling it a defect; an exit
137 with a risen `oom_kill` gets one rerun at one build job. Use
`scripts/run-bounded.py` with the settled bounds. Run, in order:

```
python3 scripts/run-bounded.py --timeout 60   -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 600  -- cargo check --workspace --all-targets --offline
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-stdlib --lib --offline regenerate_local_contract_fingerprints -- --ignored --nocapture
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-stdlib --lib --offline production_inventory_is_exact -- --nocapture
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-stdlib --lib --offline local_production_acceptance -- --nocapture
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-stdlib --lib --offline --no-default-features local_production_acceptance -- --nocapture
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-stdlib --lib --offline production_rejection_census_is_complete -- --nocapture
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-stdlib --test typed_slot_census --offline
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-stdlib --lib --offline exact_types_and_constructor_claims_match_rustc -- --nocapture
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-runtime --lib --offline frame_root_trace
python3 scripts/run-bounded.py --timeout 600  -- cargo test -p rum-runtime --lib --offline profiling
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-runtime --offline
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-runtime --offline --no-default-features
python3 scripts/run-bounded.py --timeout 1500 -- cargo test -p rum-stdlib --test suspension_oracle --offline -- logical_counter_port
python3 scripts/run-bounded.py --timeout 1500 -- cargo test -p rum-stdlib --test suspension_oracle --offline
python3 scripts/run-bounded.py --timeout 1500 -- cargo test -p rum-stdlib --test suspension_oracle --offline --no-default-features
python3 scripts/run-bounded.py --timeout 1500 -- cargo test --workspace --offline --no-fail-fast -- --skip lp03_adopted_phases_are_exact_in_completed_and_resumed_runs --skip lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs --skip lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs
python3 scripts/run-bounded.py --timeout 1500 -- cargo test --workspace --offline --no-default-features --no-fail-fast -- --skip lp03_adopted_phases_are_exact_in_completed_and_resumed_runs --skip lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs --skip lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs
```

The three skipped host tests are the instrument's standing switch-only
exclusions, not weakened contracts; run them separately with the switch on, in
both builds, quoting each verdict:

```
python3 scripts/run-bounded.py --timeout 900 -- env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp03_phase_rows --offline -- lp03_adopted_phases_are_exact_in_completed_and_resumed_runs
python3 scripts/run-bounded.py --timeout 900 -- env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp03_phase_rows --offline --no-default-features -- lp03_adopted_phases_are_exact_in_completed_and_resumed_runs
python3 scripts/run-bounded.py --timeout 900 -- env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp04_definition_rows --offline -- lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs
python3 scripts/run-bounded.py --timeout 900 -- env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp04_definition_rows --offline --no-default-features -- lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs
python3 scripts/run-bounded.py --timeout 900 -- env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp05_assignment_rows --offline -- lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs
python3 scripts/run-bounded.py --timeout 900 -- env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp05_assignment_rows --offline --no-default-features -- lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs
```

**Replay before the first acceptance suite and after every source-edit
round.** The refresh commands above are the replay; run them before the
suites, save a stale first discovery and settle it by replay. A red whose
only cause is a stale fingerprint, certificate, position or receipt is an
ordering artifact, never a regression. Never hand-edit an expected cell.
New `UnknownConstruct` rows for code this brief added are expected and
disclosed; an existing row that changes identity/status/disposition/
exclusion, a disappearance, a new `UnknownHostPayload`, or a new row pointing
at untouched code is a STOP.

At acceptance, quote every verdict line of every command; a run that did not
run is reported as not run, not as green. The workspace run is once per build
at the end; mid-change is `check --all-targets` plus the named narrower
tests. The full workspace suite does not substitute for this stage's
acceptance, and this stage does not claim the B campaign.

## Checkpointing — protect the scene

After every edit/replay/test milestone, save under `.temp/brief-646/`: the
source state, the exact next command, the key count (target 43), the A1
sites and cells covered, the mutants killed, the generated inventory deltas,
and the full logs. After compilation succeeds, two attempts with no progress
in those counters STOP and come back as a question.

## Hard rules

- No version-control command in any form; no network; no `unsafe`; no source
  outside Scope; no new semantic carrier, frame, continuation mechanism or
  public API; no guest-visible change; no inventory identity move.
- All scratch and evidence under `.temp/brief-646/`; every prior brief's
  evidence is read-only. Never `/tmp`.
- New mechanism, new unknown host payload, a missed second pause, an
  instrumentation fault, a count conflict, or a proposed cell that cannot be
  reached: STOP and ask, do not improvise a repair.
- Do not run the B campaign, `perf-panel`, Callgrind, stack probes or the
  microbench; no measurement is part of this stage.

## Report format

Write a `rum-report/1` report at `.temp/brief-646/report-a1.json`: changed
files and lines, every run (failed discovery attempts included), exact replay
verdicts, the rejection comparison, the current size and split assessment
(the Q3 exception), the mutant receipts (red line, restoration, green), the
cell receipts, the semantic result, the unrun performance gates and any
remaining question. Check it with `scripts/report.py check` (without its
version-control option) and return its summary and the full compliance
verdict lines.

## Amendment 1 — the test-instrument parser repair (after the attempt-1 STOP)

**The STOP was correct in kind and is accepted.** The fault is in a helper
this brief added, `support/logical_profile.rs` `control_legacy_keys`, not in
production code, an inventory, or an expected row. The worker left the
candidate unchanged, reproduced the panic at one thread, and asked. The
repair below is authorized; nothing else in this amendment moves.

**Cause and exact rule.** The legacy snapshot key is
`runtime|{phase}|{profile_optional_symbol_name(source)}`, and that helper is
`Symbol::to_string()` = `qualified_name()`: a global or system symbol renders
as its bare name with no backtick. `control_legacy_keys` assumed every
non-`-` field contains a backtick, so the cache-mode legacy source `f`
panicked at `rsplit_once('`').unwrap()`. Repair exactly:

```rust
let source = (parts[2] != "-").then(|| match parts[2].rsplit_once('`') {
    Some((context, name)) => rum_core::Symbol::new(format!("{context}`"), name),
    None => rum_core::Symbol::new("", parts[2]),
});
```

A bare field is reconstructed as the global symbol. If that reconstruction
still differs from the native key, STOP: do not guess a context, do not
special-case a mode, and do not weaken the comparison. No expected count,
cell, production line, or inventory identity changes.

**What to rerun after the repair.** The repair edits a file under `crates/`,
so the standing inventories expire again. Replay them before the first
remaining acceptance suite, exactly as the brief requires. Then run, in
order: the full `logical_counter_port` (all five child modes, not just cache),
the full `suspension_oracle` in both builds, the whole workspace in both
builds, and the six switch-gated `rum-host` tests with
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`. Do not redo the green production work
the attempt already proved — the runtime suites and the six mutant receipts
stand because the repair touches only the test helper. Quote every verdict
line of the resumed runs.

The disclosed 3.740-second cargo overlap of attempt 1 is accepted as
procedural noise for deterministic tests, because the worker reran the
affected checks serially; the exclusive command lock is now required for the
rest of the run and an overlap must not recur.

Update `.temp/brief-646/report-a1.json` in place for the resumed run,
retaining the attempt-1 STOP as a recorded run. If the repaired comparison
fails, or any remaining acceptance command is red, STOP with the same
evidence standard instead of repairing beyond this rule.

## Amendment 2 — the bare-name reconstruction is `from_name` (after the resume STOP)

**The second STOP is correct in kind and is accepted.** Amendment 1 removed
the panic; the strict comparison then failed at cache-mode
`lp06.status.value_return`. The worker quoted the exact mismatch: the native
key holds `Symbol { id: 670, identity: None, text: SymbolText { id: 670,
context: "Global`", name: "f" } }` and `Symbol::new("", "f")` produced the
same name in the empty context (id 671). The reproduction is at one thread.

**Cause.** `Symbol::to_string()` is `qualified_name()`, which renders both
`Global`` context symbols and `System`` context symbols as a bare name. The
bare field therefore names a real symbol in one of the two implicit
contexts; the empty context is not one. The native evidence fixes this case:
the source is the user-defined `f`, i.e. `Global`f`.

**Exact rule, one line.** Replace only the bare branch of
`control_legacy_keys`:

```rust
let source = (parts[2] != "-").then(|| match parts[2].rsplit_once('`') {
    Some((context, name)) => rum_core::Symbol::new(format!("{context}`"), name),
    None => rum_core::Symbol::from_name(parts[2]),
});
```

`Symbol::from_name(name)` is exactly `Symbol::new(GLOBAL_CONTEXT, name)`, the
default context in which guest source resolves a bare name. Because that
interned `(context, name)` pair already exists in the test process, the
reconstruction is the identical `SymbolText` and carries the same id, so the
comparison stays byte-exact and is not weakened. This is not a guess for an
arbitrary symbol: a `System`` source would render bare as well, reconstruct
in `Global``, and fail the exact comparison — a loud STOP, not a silent
conflation. No expected row, cell, production line, or inventory identity
changes.

**What to rerun.** The helper is under `crates/`, so the standing
inventories expire again: replay them before the first remaining acceptance
suite. Then run the full `logical_counter_port` in all five child modes, the
full `suspension_oracle` in both builds, the whole workspace in both builds,
and the six switch-gated `rum-host` tests with
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`. The production work, the runtime
suites, and the six mutant receipts still stand; the amended line is test
instrumentation only. Update `.temp/brief-646/report-a1.json` in place,
retaining both earlier STOPs as recorded runs and quoting every verdict
line.

If this exact reconstruction still fails for any cell, STOP. Do not pick a
different context by hand, do not special-case a mode, and do not relax the
comparison to rendered names.

## Amendment 3 — the native-span pause totals move with the tree; ZeroMQ is sandbox-only

**The STOP is correct in kind and is accepted.** The worker reproduced both
host failures at one test thread after the five-mode and oracle runs were
green, and changed no source, expectation, exclusion or skip flag. I checked
the baseline myself: at HEAD `18b7d918` the same test binary is green, 3
passed / 0 failed (`.temp/baseline-646.log`, a clean worktree, one thread).
So the two failures are candidate-induced, and they are not inventory
staleness.

**Ruling on Q3: the new pause totals are correct and the two standing
expectations must move.** `LpSpan` wraps its argument in a `Policy::Expr`
native call, so every logical record started inside that argument is a child
of the `native` row. A1 now starts `compound.item.evaluate`, and its
forwarded `.suspend` sibling, around each compound item.
`Segment::pause` walks from the pausing record to the nearest open, not
locally closed ancestor with its per-event cursor, and `Totals::child` folds
a settled child's pauses into its parent
(`crates/rum-runtime/src/runtime/profiling/logical/segments.rs:207`,
`crates/rum-runtime/src/runtime/profiling/logical/totals.rs:15`). The
`native` row's pause counter is a tree total, not an own count, so a deeper
tree changes it by construction. The old numbers were written against the
shallower tree. Do not revert, suppress or carve out propagation for adopted
children; that would be a false observer, not a repair.

**Authorized change: one file, one constant family.** Add
`crates/rum-host/tests/logical_profile_lifecycle.rs` to the files this brief
may modify. Change only the expected pause counts in that file: the two
per-arrangement constants in the nested test's input table, and the expected
arguments of the seven `Session::logical_profile_test_pauses_for("native")`
assertions (nested lines 100, 102, 105; lifecycle lines 196, 200, 203, 291).
An adjacent comment naming the new child level is allowed. Every other
assertion in the file — `counts()`, open, `closed()`, the
cancellation/reset/drop/restore cases and the leases — stays byte-identical.
Do not add a skip, a filter, an import, a helper or a mode branch.

**Derivation duty, not fitting.** For every changed number the report must
show, at that assertion point: the open record set with its key chain, the
pause event, which record receives it under the cursor, and the resulting
count. Print the full Runtime row set at each assertion during derivation.
If the derived number differs from the observed one, STOP; do not write the
observed value and invent a story for it. A number that cannot be derived
from the tree is a STOP, not a measurement.

**Ruling on Q4: the nine native ZeroMQ tests are a sandbox limitation.** The
worker sandbox denies loopback binding (`Operation not permitted`, reproduced
at one thread); the support code requires the explicit
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` to let such a test skip. For the two
workspace acceptance commands only, set that variable, and quote the flag and
all nine skip/ignore verdict lines. Change no ZeroMQ source, test or support
code. The director gate reruns the whole workspace on the host with loopback
available and without that variable; those nine must pass there. This is an
environment exemption for the sandbox run, not an acceptance exclusion.

**Reruns.** After the edit: the two host tests at one test thread; the full
workspace in both builds with the three standing `--skip` flags and the
ZeroMQ variable above; then the six switch-gated `rum-host` tests with
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`. Amendment 2's helper edit already
replayed the standing inventories; this file is test-only and is not a
standing-inventory source — verify that claim and replay if any inventory
does cover it. Update `.temp/brief-646/report-a1.json` in place, retaining
all three STOPs. Any other red is a STOP under the same evidence standard.

## Director gate — 2026-11-03: GREEN

`python3 -I scripts/director-gate.py run 646` (`.temp/director/gate-646/`):

| step | verdict | detail |
| --- | --- | --- |
| scope | SKIP | director edit after the report; `report.py check --against-git` passes, 24,305 citations |
| fmt | PASS | formatted |
| check | PASS | 3 warnings, 0 new against HEAD, 0 gone |
| ownership | PASS | 61 / 79 / 8,010; 0 replay rounds |
| census | PASS | 1,067 `{RET001: 10, RET002: 1,022, RET003: 35}`, paired, 0 new |
| oracle | PASS | 1 passed |
| standing | PASS | LP01 1/0; typed-slot 10/0 |
| fingerprints | PASS | 68 contracts, 68 fingerprints changed, copied in |
| certificates | PASS | both builds, 2 promoted rows merged |
| inventories | PASS | 19 passed, 0 failed |
| workspace | PASS | 62 binaries, **4,072 passed / 0 failed / 61 ignored**, peak 14.0 GB of 14, `oom_kill` 0 |

The `scope` step skips because I edited two stale diagnostic literals in
`logical_profile_lifecycle.rs` after the report (`pauses=2` -> `pauses=4`).
The report check passes before and after that edit; no other line moved.

**Reproduced by the director**: the clean-HEAD worktree is green on the same
host test binary (3 passed / 0 failed, `.temp/baseline-646.log`); the six
switch-gated host tests pass in both builds
(`.temp/director/gate-646-switch/`); the no-default workspace on the host,
without `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`, is 4,072 / 0 / 61
(`.temp/director/gate-646-nodefault.log`); mutant `compound_order` goes red
with exit 101 and `left [0,0,0,0,0,0,0,0,0,2]` against
`[1,1,0,0,0,0,0,0,0,1]`, restores byte-identically, and greens
(`.temp/director/gate-646-mutant/`). The ten pre-registered pause derivations
match the observed values.

**Accepted worker evidence**: the five child modes and both oracle builds
(499/0/0 each, 13 cells, 130 trace receipts, 4,375 legacy receipts per run);
the six mutant families and the key roundtrip; the derived host constants
2/4/4, 4/8/8, 4/6/6 and the unchanged cleanup 3; the sandbox ZeroMQ skips
(support flag only, nine quoted) with no source or test change.
