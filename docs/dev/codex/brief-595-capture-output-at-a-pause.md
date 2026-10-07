# brief-595 — G11 step B: output a suspended capture owns, and the four G11 entries

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-594 removed G11's three kernel panics. Step B repairs what remains of
brief-593's diagnosis and attaches the four owed oracle entries
(`EvaluationData.body`, `TestReport.input`, `TestReport.expectedOutput`,
`TestReport.sameTest`):

1. **`EvaluationData[expr, props]`** — `EvalEvaluationDataSuspensionFrame`
   (`runtime/debug_tools/evaluation_data.rs:30-36, 207-214`) omits the
   already-prepared selector, so the resumed path skips
   `project_evaluation_data_properties` (`:43-51`) and returns the whole
   capture association. Carry the selector; eager and resumed paths share one
   completion helper (finish the capture, then project once).
2. **Output a capture owns leaks at a pause.** Both pause paths in
   `session/session_requests.rs` (`:234-247`, `:352-367`) package every print,
   message and trace event after the *request* checkpoint (`:493-495`) — they
   cannot see that a still-running capture owns them. The capture then consumes
   them, so they vanish from the final cumulative output (`EvaluationData`,
   `TestReport` input), or a scope that suppresses messages has already leaked
   them (**`Quiet` surfaces a message at the pause** — `.temp/brief-593/quiet-message.txt`).

**The contract (director ruling).** Output inside an unsettled capture is
*provisional*: at a pause it stays in the kernel buffer, available to that
capture and nested observers, and is absent from the host response; the
capture settles it exactly once on completion (its existing consume, filter or
release policy) or rolls it back on abandonment. Output outside every active
capture is public at the next response, in order, per channel. Responses stay
cumulative; nothing already exposed disappears and nothing is exposed twice.
**The rule covers three channels — prints, messages and the raw trace array**
(`EvaluationData` enables and truncates tracing; ruling: trace joins the same
boundary). It lives in **one place**: a declarative output boundary that
suspension frames report (default: none) and one pause-response builder that
clips each channel at the earliest active boundary.

brief-593's report (§2, §3) and `.temp/brief-593/draft-B-capture-output-and-descriptors.md`
are the raw material; **this brief governs** where they differ.

## Rulings carried into this brief

- Failed `TestReport` cases keep their current copy-and-release policy
  (`test_report/evaluation.rs:252-273`) — tested and documented, not changed.
- `PrintSink` stays exclusive streaming (`docs/guide/embedding.md:337-348`);
  captures do not intercept streamed lines. Queued and resumed sink
  transcripts must be identical.
- `Check`, `Quiet` and `ConfirmQuiet` declare **messages** provisional while
  their body is unsettled (prints pass through); `EvaluationData` and
  `TestReport`'s input capture declare all three channels. `Reap`, the timing
  family, the constraints and `Monitor` own no buffered output.
- Q1 (one combined print/message stream) stays out of scope: per-channel order
  only, no response or serde shape change.

## Context pointers

- brief-593 and its report; `.temp/brief-593/{probe.rs,probe-test.log,quiet-message.txt}`;
  brief-594 and its report.
- `runtime/debug_tools/{evaluation_data.rs,properties.rs,test_report/evaluation.rs}`;
  `runtime/session/session_requests.rs:217-511`;
  `runtime/kernel_data/control_data/check_quiet.rs`,
  `confirmation/builtins.rs:436-472`, `confirmation/builtins/resume_frames.rs:91-120`;
  `runtime/suspension/core.rs:393-459, 713-717, 764-829`.
- The oracle: `support/{drive.rs:35-42,observe.rs:60-75,lifecycle.rs}` check
  print prefixes and final messages only — they need per-pause message and
  trace expectations.
- `docs/guide/{evaluation,embedding}.md`, `docs/testing-strategy.md`.

## Scope

**Objective:** the selector, the output boundary and pause builder, the
abandonment rollback through the same boundary, the strengthened oracle, the
four entries, and the standing and instruction gates.

**Edit surface:** `debug_tools/evaluation_data.rs`,
`debug_tools/test_report/evaluation.rs`; `runtime/suspension.rs`,
`suspension/core.rs` (capture-related abandonment only) and a new
`suspension/output.rs`; `session/session_requests.rs` (a distinct pause
builder; terminal behaviour unchanged); `control_data/check_quiet.rs`,
`confirmation/builtins/resume_frames.rs` (boundary declarations);
`runtime/tests/suspension_abandonment.rs` and one focused capture-output test
module; the oracle's `descriptors.rs`, `support/{drive,observe,lifecycle}.rs`
(observation gains the raw trace array and per-pause message and trace
expectations), `fixtures/` and `mutations/` (message- and trace-prefix
falsifiers); `inventory/{effects,owed}.tsv` — attach exactly the four G11
entries and delete exactly their four owed rows **only after all their gates
pass in both builds**, plus supplemental descriptors on existing entries;
`crates/rum-host/tests/` (one capture-output lifecycle module);
`docs/guide/{evaluation,embedding}.md` and `docs/testing-strategy.md` (state
the contract).

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed; a new frame or
derive type enters rum-runtime's frame lists. **STOP** on any existing row
change or disappearance, any new `UnknownHostPayload` or `self_reaching` row,
any new row in untouched code.

**Fence:** `push_print` and message emission unchanged — no per-emission
counter, ID, allocation or lock; raw buffers never split, cleared or reindexed
at a pause; `Expr`, response shapes, adapters, the lockfile unchanged; no other
owed entry; `scripts/` unchanged. No edits to `AGENTS.md`, `TODO.md`, root
`PROGRESS.md` or earlier briefs.

**Disclosure duty:** every changed function and type, every inventory row,
every measured label, any adjacent defect.

## Deliverable

The boundary query reads frame metadata only — it evaluates nothing and
mutates nothing — and runs only when packaging `InputRequired` (initial and
repeated pauses through the same helper) and on abandonment (inner to outer,
existing restoration helpers keep tracing, handled-error and dynamic-state
work; output rollback happens once through the common rule; never a `Check`
fallback or confirmation info callback). A pause is not a terminal failure:
capture buffers and handled-error bookkeeping survive it. Nested captures:
an inner completion cannot expose output an outer capture still owns.

## Tests

Red first, recorded before any production edit: `EvaluationData` scalar and
list selectors and captured print prefixes; `TestReport` input print prefix;
the `Quiet` message passing the old checker and failing the strengthened one.
Then each of the four entries with `Middle`, `Twice`, `Error`,
`AlreadyResumed`, `Nested` and its declared `Decline`, through `Checks::ALL`,
in both builds, lifecycle at the first and second pause. `TestReport` pauses
twice inside one input capture **and** in two distinct cases (and again for
expected output and `SameTest`, whose output belongs to the surrounding
request), with `OutputLog`, `ActualMessages` and aggregates compared — plus
wrong output, false `SameTest`, a handled failure, a terminal error, an empty
report. `EvaluationData`: no selector, `All`, `Properties`, scalar, list,
missing and invalid selectors, a selector whose preparation pauses; output
before, between and after two pauses inside, and outside before and after;
effects run once. `Check`/`Quiet`/`ConfirmQuiet`: messages before, between and
after two pauses, matching and non-matching, nested, fallback and info paths,
cancellation. A `PrintSink` control. Abandonment keeps the outside prefix and
discards the provisional capture. Falsifiers for the new message- and
trace-prefix checks, each watched to fail.

## rum / Rust pitfalls (read twice)

1. Never fix a prefix failure by relaxing the oracle or re-appending leaked
   output; keep the raw capture data and change visibility.
2. Cumulative responses repeat a prefix by design; never concatenate whole
   response arrays.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: reds turned green, entries attached,
   falsifiers watched, regressing labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-595/tmp …`. Bounds: focused 300 s; the oracle 900 s
per build; runtime, ownership and retention 300 s; a one-unit release build
900 s; a metric process 150 s; `cargo test --workspace --no-fail-fast` 3 000 s.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-host
scripts/check-suspension-oracle.sh 900
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

The oracle's `CATALOG` lines must show `owed` reduced by exactly four in both
builds. Report native ZeroMQ skips as skips.

**Instruction gate.** **Before any production edit**, build the base from the
repository tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into
`.temp/brief-595/base/target-default` and `target-alloc` (`--features
rum-cli/alloc-count`, two arguments), binaries copied with hashes; candidates
the same way in fresh directories. Panel (scratch workloads, each result
checked first): ordinary arithmetic; no-output `Check`, `Quiet`,
`EvaluationData`, `TestReport`; buffered `Print` and `Message`; captured
`Print` and `Message` inside `EvaluationData` and `Check`; the controls
`dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). **Noise floor first** (base against itself, two
pairs); a label regresses when both candidate samples exceed `max(+0.05%,
floor)`. Allocation counts (timing-valued results allocate a varying amount —
compare against base run-to-run spread), then two paired samples of `taskset -c
2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
`/dev/null`, `(count(2N) - count(N)) / N`, raw output first. No timing suite.
At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-595/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the oracle gate script, the release `rum-cli` on passing programs,
  `taskset`/`perf`, `valgrind --tool=callgrind`. No probe that aborts a
  process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files, functions and types; the boundary and where it lives.
2. Verbatim red and green output, the oracle in both builds, ownership,
   certificates and the broad suites; every inventory row change.
3. The four entries attached (owed before and after); the noise floors and
   every measured row.
4. Stop-loss counters, residuals, open questions.

---

## Execution report — brief-595 (stopped at the scope fence)

### 1. Changed files, functions and types; boundary location

**Implementation is incomplete. No production source edit was made.** The required strengthened raw-trace prefix check exposes an adjacent defect in the existing `Trace` capture. Fixing it under the required declarative-frame design needs a boundary declaration on `EvalStructuredTraceCaptureExprSuspensionFrame` in `crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs`, which is outside this brief's edit surface. Its completion and abandonment trace restoration in `debug_tools/trace.rs` also need review against the common rollback rule. The template's hard rule says: “Do NOT modify files outside Scope. If you believe you must, STOP and write that in the report instead.” I stopped before production implementation, rather than exempting Trace from the stronger checker or changing unauthorized files.

The exact witness is:

```rum
Trace[(1+2;InputString["capture"];4+5)]
```

At the pause, the response contains the raw `EvaluationPass` from `Plus[1,2]` to `3`. After resuming with `answer`, its final raw trace array is empty. `finish_trace_event_capture` truncates the captured suffix when tracing was previously disabled. The same mandatory prefix checker that detects EvaluationData's provisional trace leak therefore rejects this existing capture with `TracePrefix`. The shared trace capture frame currently has the checkpoint needed for a declaration, but this brief does not authorize editing its implementation. This is a concrete adjacent defect, not a speculative scope expansion.

Only this execution report is retained outside `.temp/brief-595/`. The temporary red harness and stronger oracle were saved, with exact source, hashes and a unified review patch, then **only this turn's five test edits** were restored to their initial bytes. No user's pre-existing edits were reverted. The source comparison uses saved bytes and SHA-256, without version-control commands.

Saved artifacts:

- `.temp/brief-595/red-harness/`: exact five test files used for the strict red.
- `.temp/brief-595/red-harness.patch`, `red-harness-sha256.json`: review patch and hashes.
- `.temp/brief-595/red-old.log`, `red-strict.log`: complete original output, with command JSON beside each.
- `.temp/brief-595/base/`: both one-unit binaries, their build commands/logs, and SHA-256 hashes.
- `.temp/brief-595/initial/`, `initial-sha256.json`, `pre-report-audit.log`: initial state and restoration audit.
- `.temp/brief-595/PROGRESS.md`: takeover checkpoint.

The temporary changes, all now saved in scratch rather than retained in crate source, were:

| File under `crates/rum-stdlib/tests/suspension_oracle/` | Changed functions/types |
| --- | --- |
| `fixtures/capture_output.rs` | New `witness`; tests `evaluation_data_scalar_selector`, `evaluation_data_list_selector`, `evaluation_data_print_prefix`, `test_report_input_print_prefix`, `quiet_message_prefix`, `adjacent_trace_capture_prefix`. |
| `fixtures/mod.rs` | Temporary `capture_output` module declaration. |
| `descriptors.rs` | `Fixture` gained final message and exact per-pause print/message/trace expectations; `Fixture::new`, new `Fixture::check_pause`, `run`, `accept`. |
| `support/observe.rs` | `Observation` gained the unmodified raw trace array; `Observation::from_output`, `Rejection`, `compare`, new per-channel `prefix`. |
| `support/drive.rs` | `Run` retained pause observations; `execute_planned` checked cumulative prefixes on all three channels using `observe::prefix`. |

No output-boundary type, production completion helper, abandonment change, lifecycle extension, synthetic falsifier, or new production frame was implemented. There is no retained source split or frame-inventory change to assess. Scratch helpers `build.py`, `run.py`, `strengthen-oracle.py`, `measure.py` and `floor.py` are local evidence/workflow aids, not repository scripts; the last two were prepared but never run.

### 2. Verbatim red output, builds, and standing/broad gate status

Both red runs used `python3 scripts/run-bounded.py`, a 300-second focused Cargo bound, `CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1`, `CARGO_NET_OFFLINE=true`, and `TMPDIR=/home/remi/dev/rum/.temp/brief-595/tmp`. Cargo's test target was isolated under the same scratch directory. Cargo ran serially, after both base builds had finished. No production edit preceded either red run. The failures below are test assertions identifying incorrect observations; no kernel-panic or process-abort probe was run.

Old-checker command:

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::capture_output -- --nocapture
```

Verbatim output:

```text
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling proc-macro2 v1.0.106
   Compiling syn v2.0.117
   Compiling version_check v0.9.5
   Compiling generic-array v0.14.7
   Compiling typenum v1.20.1
   Compiling autocfg v1.5.0
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling serde_derive v1.0.228
   Compiling thiserror v2.0.18
   Compiling cfg-if v1.0.4
   Compiling digest v0.10.7
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling num-bigint v0.4.6
   Compiling cpufeatures v0.2.17
   Compiling crc32fast v1.5.0
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:12:8
   |
12 | pub fn execute(
   |        ^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: `rum-stdlib` (test "suspension_oracle") generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 08s
     Running tests/suspension_oracle.rs (.temp/brief-595/target-tests/debug/deps/suspension_oracle-08d116f9343550b3)

running 5 tests
test oracle::fixtures::capture_output::evaluation_data_list_selector ... 
thread 'oracle::fixtures::capture_output::evaluation_data_list_selector' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:25:165:
called `Result::unwrap()` on an `Err` value: Value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::capture_output::evaluation_data_print_prefix ... 
thread 'oracle::fixtures::capture_output::evaluation_data_print_prefix' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:30:219:
called `Result::unwrap()` on an `Err` value: OutputPrefix
FAILED
test oracle::fixtures::capture_output::evaluation_data_scalar_selector ... 
thread 'oracle::fixtures::capture_output::evaluation_data_scalar_selector' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:20:104:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::capture_output::quiet_message_prefix ... ok
test oracle::fixtures::capture_output::test_report_input_print_prefix ... 
thread 'oracle::fixtures::capture_output::test_report_input_print_prefix' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:35:154:
called `Result::unwrap()` on an `Err` value: OutputPrefix
FAILED

failures:

failures:
    oracle::fixtures::capture_output::evaluation_data_list_selector
    oracle::fixtures::capture_output::evaluation_data_print_prefix
    oracle::fixtures::capture_output::evaluation_data_scalar_selector
    oracle::fixtures::capture_output::test_report_input_print_prefix

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

The identical Quiet witness passes that old checker. After adding the message and raw-trace prefix checks, the strict run used the same command and also included the adjacent Trace witness. Verbatim output:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: `rum-stdlib` (test "suspension_oracle") generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.10s
     Running tests/suspension_oracle.rs (.temp/brief-595/target-tests/debug/deps/suspension_oracle-08d116f9343550b3)

running 6 tests
test oracle::fixtures::capture_output::adjacent_trace_capture_prefix ... TRACE_CONTROL paused_trace=[EvaluationPass { before: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 19, identity: None, text: SymbolText { id: 19, context: "System`", name: "Plus" } }))), args: SharedArgs([Atom(Integer(Small(1))), Atom(Integer(Small(2)))]) }, after: Atom(Integer(Small(3))) }] completed_trace=[]

thread 'oracle::fixtures::capture_output::adjacent_trace_capture_prefix' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:51:29:
called `Result::unwrap()` on an `Err` value: TracePrefix
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::capture_output::evaluation_data_list_selector ... 
thread 'oracle::fixtures::capture_output::evaluation_data_list_selector' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:25:165:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::capture_output::evaluation_data_print_prefix ... 
thread 'oracle::fixtures::capture_output::evaluation_data_print_prefix' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:30:219:
called `Result::unwrap()` on an `Err` value: OutputPrefix
FAILED
test oracle::fixtures::capture_output::evaluation_data_scalar_selector ... 
thread 'oracle::fixtures::capture_output::evaluation_data_scalar_selector' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:20:104:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::capture_output::quiet_message_prefix ... 
thread 'oracle::fixtures::capture_output::quiet_message_prefix' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:40:134:
called `Result::unwrap()` on an `Err` value: MessagePrefix
FAILED
test oracle::fixtures::capture_output::test_report_input_print_prefix ... 
thread 'oracle::fixtures::capture_output::test_report_input_print_prefix' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs:35:154:
called `Result::unwrap()` on an `Err` value: OutputPrefix
FAILED

failures:

failures:
    oracle::fixtures::capture_output::adjacent_trace_capture_prefix
    oracle::fixtures::capture_output::evaluation_data_list_selector
    oracle::fixtures::capture_output::evaluation_data_print_prefix
    oracle::fixtures::capture_output::evaluation_data_scalar_selector
    oracle::fixtures::capture_output::quiet_message_prefix
    oracle::fixtures::capture_output::test_report_input_print_prefix

test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

Before these tests, both base builds completed from the untouched production tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, offline, one build job, a 900-second bound each, and fresh `.temp/brief-595/base/target-default` and `target-alloc` directories. The allocation feature used the two arguments `--features rum-cli/alloc-count`. The binaries were copied and hashed. Verbatim build completion lines:

```text
default:     Finished `release` profile [optimized] target(s) in 1m 39s
alloc:     Finished `release` profile [optimized] target(s) in 1m 39s
```

```text
rum-cli  bf4ea66f76406dee9ea6143e65698db83fb78e125a15de4cad291c73ba845058
rum-cli-alloc  d7eed5cea6fea9130c89f33a4320439bb514046cc52223dee44af67e28a55a29
```

No green production test is claimed. Following the scope stop, these gates **did not run**: `cargo fmt --check`, the runtime and host suites, `scripts/check-suspension-oracle.sh 900` in either build, ownership discovery/reconciliation, either local-production certificate replay, and `cargo test --workspace --no-fail-fast`. The strict red was run in the default build only; no no-default test result is claimed. Native ZeroMQ tests did not run, so there were no observed native skips to report.

Every source edit was temporary and restored after preserving its evidence. Thus no changed-source fingerprint or certificate update is left pending. Ownership inventories, local-contract fingerprints, certificate cells, frame lists, and every oracle inventory file remain byte-identical to their initial state. **Every inventory row change: none.** The four owed entries remain attached to no new descriptors. No output emission path, response/serde shape, adapter, lockfile, repository script, unsafe code, AGENTS.md, TODO.md, root PROGRESS.md, or earlier brief changed. No version-control command or network access was used.

Verbatim restoration audit:

```text
PASS: all initial source, docs, scripts, inventories, fingerprints, certificates, root Cargo files, toolchain, AGENTS.md, TODO.md and root PROGRESS.md byte-identical after preserving/restoring temporary red harness. No extra source files.
```

### 3. Four G11 entries, floors and measured rows

Entries attached: **0/4**. Owed ledger: **114 before, 114 after**; the required reduction of four was not performed because the admission gates have not passed. No `CATALOG` output from a new gate run is claimed. The unchanged rows are:

```text
EvaluationData.body	debug	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
TestReport.expectedOutput	debug	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
TestReport.input	debug	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
TestReport.sameTest	debug	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
```

Instruction and allocation samples: **none**. Noise floors: **not measured**. Measured candidate rows: **none**, and no performance verdict is claimed. The base binaries are preserved for the resumed task. Eleven workload definitions, including the three copied controls, were prepared under `.temp/brief-595/perf/`; none was executed or declared passing. No timing suite was run.

There is a second concrete gate limitation: `crates/rum-cli/src/main.rs:270` unconditionally installs the stdout `PrintSink` before each evaluation request. That sink routes Print exclusively, as required by the embedding contract. Therefore this unmodified CLI cannot execute the panel's **buffered Print**, **EvaluationData captured buffered Print**, and **Check buffered Print** paths. Streaming labels cannot honestly stand in for them. The brief forbids CLI changes. An explicitly authorized scratch Rust measurement driver or other specified buffered-output harness is needed for those rows. This finding is based on source inspection; no substitute metric was measured. Details are in `.temp/brief-595/perf/print-gate-limitation.md`.

### 4. Stop-loss counters, residuals and next action

- Production compiling candidates: 0; measured candidates: 0/8. The two completed builds after the bases were red-test harness builds, not production candidates.
- Required red shapes recorded: scalar selector, list selector, EvaluationData print prefix, TestReport input print prefix, and Quiet passing old/failing strict checker.
- Additional red: standalone Trace loses its exposed raw-trace prefix.
- Reds turned green: 0. Entries attached: 0/4. New synthetic falsifiers watched: 0. Regressing labels/worst regression: unmeasured. The two-candidate no-progress stop-loss was not reached; the scope fence caused this stop.
- Selector repair, common output boundary/pause builder, abandonment rollback, the complete descriptor/lifecycle matrix, host coverage, documentation contract changes, standing gates and performance gates remain undone.

The next director action is to resolve the scope conflict for the shared Trace capture frame (including its rollback policy), and specify an authorized buffered-Print measurement harness. With those rulings, restore the saved test-only patch by applying its source files, complete the required base panel/floors, then implement the selector and common boundary. The exact initial sources, reusable base binaries, red-test sources and raw logs are preserved; no partial production implementation or stale source certificate is left in the tree.

## Director amendment — resume from the scope stop (2026-10-07)

The stop was correct; the scope was mine and under-enumerated. Resume from
`.temp/brief-595/PROGRESS.md`: restore the saved test-only patch
(`.temp/brief-595/red-harness/`), reuse the preserved base binaries, and
continue. This amendment governs where the sections above differ.

1. **`Trace` is a capture of the trace channel.** `Trace[(1+2;InputString["capture"];4+5)]`
   exposes the raw `EvaluationPass` at the pause and then truncates it
   (`finish_trace_event_capture`, `debug_tools/trace.rs:117`). It joins the
   same contract: the trace events a `Trace`, `TraceDetail` or `TracePrint`
   capture owns are provisional at a pause and settled once by that capture.
   **Added to the edit surface:** `runtime/debug_tools/trace/resume_frames.rs`
   (the boundary declaration on `EvalStructuredTraceCaptureExprSuspensionFrame`
   and any sibling capture frame) and `runtime/debug_tools/trace.rs` (its
   completion and abandonment rollback through the common rule). Its
   strengthened descriptors are supplemental to the already drained
   `Trace.body`, `TraceDetail.body` and `TracePrint.body` entries;
   `TracePrint`'s rendered print output follows the print channel rule.
   Any *other* capture the strengthened checker exposes is reported with its
   witness and a STOP, not exempted.
2. **Buffered `Print` cannot be measured through `rum-cli`**, which installs an
   exclusive stdout `PrintSink` per request (`crates/rum-cli/src/main.rs:270`).
   No new measurement harness: `push_print` and emission are fenced and
   unchanged, and the buffered-print behaviour is covered by the `rum-host`
   tests. Drop the buffered-`Print` and captured-buffered-`Print` labels; keep
   ordinary arithmetic, no-output `Check`/`Quiet`/`EvaluationData`/`TestReport`
   (and `Trace`), buffered and captured `Message` (messages are buffered under
   the CLI — check it on the base first), and the three controls.
3. Everything else stands: the four G11 entries are attached only after all
   their gates pass in both builds; owed 114 -> 110.
