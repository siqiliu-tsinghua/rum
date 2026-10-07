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

---

## Resumed execution report — brief-595 (2026-10-07; stopped at the next scope fence)

### 1. Changed files, functions and types; the output boundary

**The implementation is incomplete and is not ready for admission.** Candidate 1 is preserved in the tree. The original six reds pass, the complete pre-admission oracle passes in both feature builds, the instruction panel passes its declared rule, and the standing maintenance gates pass. The mandatory runtime suite fails two tests. One requires changing an existing guest-loading test outside the edit surface; implementation stopped when that conflict appeared. The four attempted entry attachments were removed by restoring only our registration and ledger edits to their saved initial bytes. Final owed count: **114**, entries attached: **0/4**.

The concrete scope conflict is `crates/rum-runtime/src/runtime/session/guest_loading/tests.rs:361` (and its repeated expectation at :366). `guest_environment_restores_handled_error_tail_and_memory_limit_before_child` seeds a handled error inside a guest file and requires the live handled-error tail to be empty at both pauses. Its assertion says “request packaging should have truncated the live tail”. The brief explicitly requires: “A pause is not a terminal failure: capture buffers and handled-error bookkeeping survive it.” The template says: “Do NOT modify files outside Scope. If you believe you must, STOP and write that in the report instead.” No unauthorized test edit or production exception was made to conceal this conflict.

A second runtime failure is local and mechanically repairable: the source invariant in `runtime/tests/frame_root_trace.rs:354` expects `resume` to be the first member of `SuspensionFrame`. The new default `output_boundary` method currently precedes it. Moving the new method after `resume` in the already-authorized `suspension/core.rs` would preserve that checker. This was not done after the scope stop; it remains a visible residual.

The director amendment was read and followed: restored the saved strict red harness; verified initial crate bytes and the preserved base hashes; included Trace/TraceDetail/TracePrint; dropped the buffered-Print performance labels; validated buffered and captured Message on the base. No version-control command, network access, unsafe addition, external-engine comparison, timing suite, or process-abort probe was used. Scratch remains under `.temp/brief-595/`, with only the explicitly permitted legacy certificate writer under `.temp/brief-546/certificates/`.

`runtime/suspension/output.rs` owns `OutputBoundary`, metadata aggregation, and rollback. Each channel is an optional first provisional offset; nested declarations use the earliest offset. The query reads frame metadata and evaluates nothing. Both initial and repeated `InputRequired` paths call `Session::finish_input_required_request`, which copies only the public prefix of each channel into the response and preserves raw buffers and the handled-error tail. On abandonment, the driver applies the frame's boundary inner to outer before the existing bookkeeping restoration. Capture completion keeps the existing consume/filter/release policy. The prepared EvaluationData selector travels in the frame and both eager and resumed completion call the same finish-and-project helper.

Changed production declarations (paths relative to `crates/rum-runtime/src/runtime/`):

| File | Every changed/new function or type |
| --- | --- |
| `suspension.rs` | Declares `output` and re-exports the doc-hidden `OutputBoundary`. |
| `suspension/output.rs` (new) | `OutputBoundary` and its `all`, `messages`, `trace`, `include` methods; nested helper `earliest`; `SuspendedEvaluation::output_boundary`; `Session::pending_output_boundary`, `Session::rollback_capture_output`. |
| `suspension/core.rs` | `SuspendedEvaluation.frames` becomes visible to sibling modules; `SuspensionFrame::output_boundary` defaults to none; `Session::abandon_pending_suspension` performs common rollback; `AbandonmentContext::{restore_trace_capture,restore_evaluation_data_capture,restore_message_capture,restore_stream_capture}` retain tracing/handled-error restoration and stop duplicating output rollback. |
| `debug_tools/evaluation_data.rs` | `Session::{eval_evaluation_data,finish_evaluation_data_capture}`; `EvalEvaluationDataSuspensionFrame` adds prepared `properties`; its `output_boundary`, `resume`, `abandon`. |
| `debug_tools/test_report/evaluation.rs` | `Session::eval_expr_with_stream_capture`; `EvalExprWithStreamCaptureSuspensionFrame` adds `trace_checkpoint`; its `output_boundary`, `abandon`. Existing completion and failed-input copy/release code is unchanged. |
| `debug_tools/trace/resume_frames.rs` | `EvalStructuredTraceCaptureExprSuspensionFrame::{output_boundary,abandon}`. |
| `debug_tools/trace.rs` | `Session::{abandon_trace_capture,truncate_trace_from}`. Normal completion/error policy is preserved; truncation delegates to the common rollback implementation. |
| `kernel_data/control_data/check_quiet.rs` | `EvalCheckSuspensionFrame::{output_boundary,abandon}` and `EvalQuietSuspensionFrame::{output_boundary,abandon}`. |
| `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | `EvalConfirmQuietSuspensionFrame::output_boundary`. |
| `session/session_requests.rs` | `Session::{eval_expr_request_from_checkpoint,resume_eval_request_from_checkpoint}` call new `finish_input_required_request`; terminal response builders are unchanged. |

Six existing capture-frame types declare boundaries: EvaluationData and TestReport input all three channels; Check, Quiet and ConfirmQuiet messages; the shared Trace capture raw trace. No new suspension frame or FrameRootTrace/PureFrameAbandon derive type was introduced, so the existing closed frame lists need no entry. No frame stores a Session borrow, cancellation observer, or NativeStackDepth. Emission functions, Expr, response/serde shapes, adapters, the lockfile, scripts and the protected root documents are unchanged.

Changed existing oracle/test declarations (relative to `crates/rum-stdlib/tests/suspension_oracle/` unless stated):

| File | Every changed/new function or type |
| --- | --- |
| `descriptors.rs` | `Fixture` adds final messages and exact per-pause print/message/trace expectations; `Fixture::{new,check_pause}`, `run`, `accept`. Diagnostic mismatch output is test-only. |
| `support/observe.rs` | `Observation` gains raw `trace`; `Observation::from_output`; `Rejection` gains channel/pause variants; `compare`, new `prefix`. |
| `support/drive.rs` | `Run` gains pause observations; `execute_planned` checks all three cumulative prefixes; `execute` comment updated. |
| `support/lifecycle.rs` | `exercise_observed_at` gains message/trace prefix checks; new `exercise_fixture` checks exact buffered pause/cancellation observations and delegates the separate streaming lifecycle. |
| `fixtures/mod.rs`, `mutations/mod.rs` | Declare `capture_output` modules. |
| `crates/rum-runtime/src/runtime/tests/suspension_abandonment.rs` | Adds the focused `capture_output` path module; existing test bodies unchanged. |

Every function in the new focused test modules is listed below; these files add no new test types.

| New file | Functions |
| --- | --- |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output.rs` | `witness`, `evaluation_data_scalar_selector`, `evaluation_data_list_selector`, `evaluation_data_print_prefix`, `test_report_input_print_prefix`, `quiet_message_prefix`, `adjacent_trace_capture_prefix` |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output/roles.rs` | `message`, `pause`, `descriptors`, `within`, `cases`, `fixture`, `exercise`, `g11_admission_matrix` |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output/selectors.rs` | `evaluation_data_selectors_and_preparation_run_once`, `report_failed_input_copy_and_release_wrong_output_and_empty_report` |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/capture_output/messages.rs` | `tagged`, `descriptors`, `fixture`, `message_and_trace_supplemental_matrix`, `nonmatching_messages_and_settled_fallback_and_info_are_public`, `body_errors_settle_matching_and_nonmatching_message_policies` |
| `crates/rum-stdlib/tests/suspension_oracle/mutations/capture_output.rs` | `lost_message_prefix_is_rejected_when_old_print_only_check_passes`, `lost_trace_prefix_is_rejected_when_old_print_only_check_passes` |
| `crates/rum-runtime/src/runtime/tests/capture_output.rs` | `pause_keeps_raw_capture_buffers_and_handled_errors_for_nested_observers`, `nested_capture_observes_raw_messages_but_cannot_publish_outer_output`, `abandonment_rolls_back_all_declared_channels_with_existing_tracing`, `abandonment_discards_messages_without_running_fallback_or_confirmation_info`, `completion_releases_trace_prefix_when_tracing_was_already_enabled`, `a_handled_guest_error_remains_handled_across_two_pauses` |
| `crates/rum-host/tests/capture_output_lifecycle.rs` | `success`, `pause`, `new`, `cases`, `source`, `assert_pause`, `queued_and_resumed_output_and_streaming_transcripts_match`, `cancel_and_reset_at_both_pauses_keep_only_public_buffered_output`, `streamed_prints_never_enter_evaluation_data_output_log` |

The standalone G11 matrix contains seven descriptors and 37 variants: EvaluationData plus within-capture and separate-case variants for each TestReport role, through `Checks::ALL`; `Decline` is declared only for the two SameTest descriptors. Six supplemental descriptors contain 30 variants for Check/Quiet/ConfirmQuiet/Trace/TraceDetail/TracePrint. All 13 descriptor matrices exercise buffered and exclusive-sink lifecycle at both pauses. They remain standalone coverage, not registered admissions.

Additional tests cover all selector shapes and a selector-preparation pause, once-only effects, wrong expected output, false SameTest, failed input copy/release, a handled iteration-limit failure, terminal Throw, empty reports, matching/nonmatching messages, settled fallback/info paths and their errors, nested capture visibility, previously enabled tracing, real handled guest errors across two pauses, cancellation/reset at both pauses, queued/resumed sink transcript equality and empty EvaluationData OutputLog under a sink. The two new falsifiers corrupt actual message/trace observations, demonstrate the old print-only check accepts them, and require the stronger checker to reject them; exact pause checks are also falsified.

Documentation changes: `docs/guide/evaluation.md` states provisional output ownership, selector completion, capture-specific settlement and TestReport failed-input copy/release; `docs/guide/embedding.md` states cumulative three-channel responses, cancellation/reset and exclusive sink behavior; `docs/testing-strategy.md` records raw-trace observations, exact per-pause expectations, both-pause lifecycle and the new falsifiers. This brief gains the present execution report. The six ownership inventory files and two retention inventory files below are maintenance-only changes.

Organization assessment: `suspension/core.rs` shrinks from 30,181 to 28,352 characters by moving output query/rollback into its own 3,010-character responsibility module (exact size in `file-sizes.json`). It remains in the split-plan range; a future split can separate protocol/driver from scope-restoration helpers. `session_requests.rs` grows 34,040 → 34,396; the pause builder belongs with request packaging, while a future independent change can separate editor queries/error mapping from evaluation request lifecycle. `test_report/evaluation.rs` is 18,449 → 18,744 (future capture settlement vs continuation encoding); `trace.rs` 15,078 → 14,856 (future presentation vs capture lifecycle); confirmation `resume_frames.rs` 18,193 → 18,392 (future phase-based frame groups); `evaluation_data.rs` 9,842 → 10,128 (future association rendering vs capture lifecycle if it grows). The existing abandonment test module is 20,087 → 20,138 and only declares the separate new test file. New role fixtures are 10,558 characters, below the documented preferred test ceiling; other new test files are smaller. No arbitrary source split was made to satisfy a size counter.

The retained non-scratch file list has 36 paths. Exact bytes and the source/inventory/docs review diff (excluding this separately saved report) are saved in `.temp/brief-595/candidate-source/`, `candidate-source.patch`, `candidate-source-sha256.json`, `changed-files.txt`, and `file-sizes.json`. Initial bytes and hashes remain in `initial/` and `initial-sha256.json`. The first audit helper incorrectly counted the owed header as a data row; it was corrected without a repository change. The successful final audit is quoted below.

```text
Initial manifest files verified: 1943
Retained changed/new files: 36
Unexpected changes: 0
Missing initial files: 0
Added unsafe: 0
Oracle primary/effects/owed restored to initial bytes; owed=114
Oracle runner script copies byte-identical to repository scripts
Protected files and all files outside the authorized surface unchanged against initial snapshot
Source snapshot and unified patch saved without version-control commands
```

### 2. Verbatim red/green evidence, oracle and standing/broad gates; every inventory change

All Cargo runs were serial and bounded, offline, with `CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1`, `CARGO_NET_OFFLINE=true`, and `TMPDIR=/home/remi/dev/rum/.temp/brief-595/tmp`. Exact command arrays accompany logs. Focused/runtime/ownership/certificate bounds were 300 seconds; each release arm 900 seconds; each metric process 150 seconds; oracle 900 seconds per build. The unchanged oracle script and bounded runner were copied byte-for-byte under `.temp/brief-595/oracle-runner/scripts/` (the brief-594 precedent), so their scratch root remained under this brief; no repository script was edited.

The original pre-production red outputs are pasted in full in the earlier report above and retained as `red-old.log` and `red-strict.log`. Verbatim result summaries, old checker then strengthened checker:

```text
test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.04s
```

```text
test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.04s
```

The first compiling production candidate turned all six strict reds green (`candidate-1-reds.log`):

```text
test oracle::fixtures::capture_output::adjacent_trace_capture_prefix ... TRACE_CONTROL paused_trace=[] completed_trace=[]
ok
test oracle::fixtures::capture_output::evaluation_data_list_selector ... ok
test oracle::fixtures::capture_output::evaluation_data_print_prefix ... ok
test oracle::fixtures::capture_output::evaluation_data_scalar_selector ... ok
test oracle::fixtures::capture_output::quiet_message_prefix ... ok
test oracle::fixtures::capture_output::test_report_input_print_prefix ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.05s
```

Test development failures were retained, not hidden: `capture-matrix-default.log` reports 11 passed/2 failed, then `capture-matrix-default-2.log` 12 passed/1 failed. The two-case failed-input expectation incorrectly included the first successful case's already-consumed output; it was corrected to release only the failed case's output. A handled-error stimulus used `1/0`, which rum leaves symbolic; it was replaced by documented ReplaceRepeated iteration exhaustion (`General::iterlim`, limit 8). No documented contract was weakened. The host test's first build (`host-capture.log`) rejected `&String` as a request; only new test call sites were changed to `.as_str()`. The passing focused outputs follow.

`capture-default-green.log`:

```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.74s
```

`capture-no-default-green.log`:

```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.78s
```

`host-capture-green.log`:

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

New message/trace falsifier output in the full default oracle (the no-default run prints the same rejections):

```text
test oracle::mutations::capture_output::lost_message_prefix_is_rejected_when_old_print_only_check_passes ... WEAK message_prefix: PASS
STRICT message_prefix: Err(MessagePrefix)
CONTROL message_prefix: PASS
STRICT message_pause: PauseMessages(0)
test oracle::mutations::capture_output::lost_trace_prefix_is_rejected_when_old_print_only_check_passes ... WEAK trace_prefix: PASS
STRICT trace_prefix: Err(TracePrefix)
CONTROL trace_prefix: PASS
STRICT trace_pause: PauseTrace(0)
```

Full oracle before registration, both feature builds, verbatim (`oracle-before-admission-{default,no-default}.log`):

```text
LIFECYCLE obligations=2466 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=237 NoLocalGuestEffect=406 Boundary=5 entries=465 descriptors=415 historical=209 owed=114
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.37s
```

```text
LIFECYCLE obligations=2466 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=237 NoLocalGuestEffect=400 Boundary=5 entries=465 descriptors=415 historical=209 owed=114
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.40s
```

The standalone G11 and supplemental matrices are included in those 33 passing tests, in addition to the existing catalog. Their complete per-variant and lifecycle output is retained in those logs. The G11 matrix test was later renamed from `g11_unattached_admission_matrix` to `g11_admission_matrix`; its body was unchanged. No oracle run with owed=110 is claimed.

`cargo fmt --check` exited 0 (`fmt.log`). Mandatory `cargo test -p rum-runtime` completed with the following failures (`runtime.log`, verbatim):

```text
failures:

---- runtime::session::guest_loading::tests::guest_environment_restores_handled_error_tail_and_memory_limit_before_child stdout ----

thread 'runtime::session::guest_loading::tests::guest_environment_restores_handled_error_tail_and_memory_limit_before_child' panicked at crates/rum-runtime/src/runtime/session/guest_loading/tests.rs:361:5:
assertion failed: s.handled_non_loader_eval_errors.is_empty()

---- runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold stdout ----

thread 'runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold' panicked at crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:354:5:
assertion failed: compact_core.contains("pubtraitSuspensionFrame:FrameRootTrace+FrameAbandon+Send+Sync+'static{fnresume(self:Arc<Self>,session:&mutSession,outcome:Result<Expr,EvalError>,)->FrameAction;")


failures:
    runtime::session::guest_loading::tests::guest_environment_restores_handled_error_tail_and_memory_limit_before_child
    runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold

test result: FAILED. 1147 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out; finished in 31.05s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

All six new runtime capture tests passed in that same failing suite:

```text
test runtime::tests::suspension_abandonment::capture_output::a_handled_guest_error_remains_handled_across_two_pauses ... ok
test runtime::tests::suspension_abandonment::capture_output::abandonment_discards_messages_without_running_fallback_or_confirmation_info ... ok
test runtime::tests::suspension_abandonment::capture_output::abandonment_rolls_back_all_declared_channels_with_existing_tracing ... ok
test runtime::tests::suspension_abandonment::capture_output::completion_releases_trace_prefix_when_tracing_was_already_enabled ... ok
test runtime::tests::suspension_abandonment::capture_output::nested_capture_observes_raw_messages_but_cannot_publish_outer_output ... ok
test runtime::tests::suspension_abandonment::capture_output::pause_keeps_raw_capture_buffers_and_handled_errors_for_nested_observers ... ok
```

Implementation stopped at these failures. Full `cargo test -p rum-host`, final post-registration oracle, and `cargo test --workspace --no-fail-fast` **did not run**. Native ZeroMQ tests **did not run**; there is no native skip or pass to report. The three focused rum-host capture tests above are not a substitute for the full host suite. No further production or test edits were made after the stop; only reversal of premature registrations and the explicitly standing maintenance/report duties followed.

Ownership discovery/reconciliation preserved all existing identities, dispositions, statuses and exclusions. Exactly 127 existing rows changed source/witness coordinates; no existing row disappeared. One new `UnknownConstruct` remains unknown at the new OutputBoundary derive. No new host payload or self-reaching row, and no new row in untouched code. No ownership stop condition was encountered. Verbatim reconciliation (`ownership-reconciliation.log`):

```text
host_payloads.tsv before 65 after 65 changed 9 removed 0 added 0
unknown_attributes.tsv before 1498 after 1499 changed 18 removed 0 added 1
NEW UnknownConstruct	crates/rum-runtime/src/runtime/suspension/output.rs	8	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
unknown_captures.tsv before 2946 after 2946 changed 49 removed 0 added 0
unknown_macros.tsv before 3144 after 3144 changed 38 removed 0 added 0
unknown_storage.tsv before 89 after 89 changed 4 removed 0 added 0
self_reaching.tsv before 59 after 59 changed 0 removed 0 added 0
host_payload_ledger.tsv before None after None changed 9 removed 0 added 0
```

Full ownership suite (`ownership.log`), verbatim receipt lines and result:

```text
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 65 rows; 6 storage edges (3 Declared, 3 Unbounded); 59 PublicInput
checks 1/2/3: complete, no stale entries, field-specific expression attribution verified
external_implementation_coverage: unresolved for every Unbounded entry
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7678 rows
RESTORED: rum_syntax::ast::ScannerRootSetProbe absent; all other cycles and unknowns unchanged
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 61.82s
```

Every changed ownership row is listed here. Row numbers are one-based inventory lines before reconciliation; the old/new coordinates and encoded `@line` witness locations are the only changes. Full before/after strings and reasons are in `.temp/brief-595/ownership-row-changes.json`. All paths in this table are relative to `crates/rum-runtime/src/runtime/`.

| Inventory | Row | Source | Position before → after | Encoded witness positions before → after |
| --- | ---: | --- | --- | --- |
| `host_payloads.tsv` | 49 | `session/session_requests.rs` | 280:16 → 268:16 | @44 → @44 |
| `host_payloads.tsv` | 50 | `session/session_requests.rs` | 432:15 → 408:15 | @44 → @44 |
| `host_payloads.tsv` | 51 | `session/session_requests.rs` | 467:15 → 443:15 | @44 → @44 |
| `host_payloads.tsv` | 52 | `suspension/core.rs` | 21:13 → 21:24 | — → — |
| `host_payloads.tsv` | 53 | `suspension/core.rs` | 101:21 → 106:21 | — → — |
| `host_payloads.tsv` | 54 | `suspension/core.rs` | 103:14 → 108:14 | — → — |
| `host_payloads.tsv` | 55 | `suspension/core.rs` | 113:48 → 118:48 | @106 → @111 |
| `host_payloads.tsv` | 56 | `suspension/core.rs` | 124:35 → 129:35 | @106 → @111 |
| `host_payloads.tsv` | 57 | `suspension/core.rs` | 552:57 → 505:57 | @486 → @439 |
| `unknown_attributes.tsv` | 53 | `debug_tools/evaluation_data.rs` | 188:1 → 189:1 | — → — |
| `unknown_attributes.tsv` | 54 | `debug_tools/evaluation_data.rs` | 207:1 → 208:1 | — → — |
| `unknown_attributes.tsv` | 82 | `debug_tools/test_report/evaluation.rs` | 402:1 → 403:1 | — → — |
| `unknown_attributes.tsv` | 83 | `debug_tools/test_report/evaluation.rs` | 425:1 → 426:1 | — → — |
| `unknown_attributes.tsv` | 84 | `debug_tools/test_report/evaluation.rs` | 450:1 → 451:1 | — → — |
| `unknown_attributes.tsv` | 116 | `debug_tools/trace/resume_frames.rs` | 45:1 → 50:1 | — → — |
| `unknown_attributes.tsv` | 314 | `kernel_data/control_data/check_quiet.rs` | 216:1 → 221:1 | — → — |
| `unknown_attributes.tsv` | 327 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 123:1 → 128:1 | — → — |
| `unknown_attributes.tsv` | 328 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 144:1 → 149:1 | — → — |
| `unknown_attributes.tsv` | 329 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 188:1 → 193:1 | — → — |
| `unknown_attributes.tsv` | 330 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 221:1 → 226:1 | — → — |
| `unknown_attributes.tsv` | 331 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 258:1 → 263:1 | — → — |
| `unknown_attributes.tsv` | 332 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 282:1 → 287:1 | — → — |
| `unknown_attributes.tsv` | 333 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 315:1 → 320:1 | — → — |
| `unknown_attributes.tsv` | 334 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 366:1 → 371:1 | — → — |
| `unknown_attributes.tsv` | 335 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 399:1 → 404:1 | — → — |
| `unknown_attributes.tsv` | 336 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 432:1 → 437:1 | — → — |
| `unknown_attributes.tsv` | 851 | `session/session_requests.rs` | 607:9 → 619:9 | — → — |
| `unknown_attributes.tsv` | new | `suspension/output.rs` | — → 8:1 | `unresolved attribute/expansion derive (Clone , Copy , Debug , Default)` |
| `unknown_captures.tsv` | 150 | `debug_tools/evaluation_data.rs` | 232:24 → 243:24 | — → — |
| `unknown_captures.tsv` | 219 | `debug_tools/test_report/evaluation.rs` | 289:14 → 290:14 | — → — |
| `unknown_captures.tsv` | 220 | `debug_tools/test_report/evaluation.rs` | 315:14 → 316:14 | — → — |
| `unknown_captures.tsv` | 221 | `debug_tools/test_report/evaluation.rs` | 414:18 → 415:18 | — → — |
| `unknown_captures.tsv` | 222 | `debug_tools/test_report/evaluation.rs` | 417:26 → 418:26 | — → — |
| `unknown_captures.tsv` | 223 | `debug_tools/test_report/evaluation.rs` | 441:18 → 442:18 | — → — |
| `unknown_captures.tsv` | 224 | `debug_tools/test_report/evaluation.rs` | 442:26 → 443:26 | — → — |
| `unknown_captures.tsv` | 225 | `debug_tools/test_report/evaluation.rs` | 472:24 → 483:24 | — → — |
| `unknown_captures.tsv` | 247 | `debug_tools/trace.rs` | 307:18 → 301:18 | — → — |
| `unknown_captures.tsv` | 248 | `debug_tools/trace.rs` | 330:14 → 324:14 | — → — |
| `unknown_captures.tsv` | 249 | `debug_tools/trace.rs` | 337:19 → 331:19 | — → — |
| `unknown_captures.tsv` | 250 | `debug_tools/trace.rs` | 368:33 → 362:33 | — → — |
| `unknown_captures.tsv` | 251 | `debug_tools/trace.rs` | 375:33 → 369:33 | — → — |
| `unknown_captures.tsv` | 252 | `debug_tools/trace.rs` | 393:38 → 387:38 | — → — |
| `unknown_captures.tsv` | 253 | `debug_tools/trace.rs` | 396:14 → 390:14 | — → — |
| `unknown_captures.tsv` | 258 | `debug_tools/trace/resume_frames.rs` | 23:24 → 28:24 | — → — |
| `unknown_captures.tsv` | 698 | `kernel_data/control_data/check_quiet.rs` | 193:24 → 198:24 | — → — |
| `unknown_captures.tsv` | 699 | `kernel_data/control_data/check_quiet.rs` | 235:24 → 245:24 | — → — |
| `unknown_captures.tsv` | 714 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 111:24 → 116:24 | — → — |
| `unknown_captures.tsv` | 715 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 137:26 → 142:26 | — → — |
| `unknown_captures.tsv` | 716 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 169:18 → 174:18 | — → — |
| `unknown_captures.tsv` | 717 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 180:26 → 185:26 | — → — |
| `unknown_captures.tsv` | 718 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 210:18 → 215:18 | — → — |
| `unknown_captures.tsv` | 719 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 213:26 → 218:26 | — → — |
| `unknown_captures.tsv` | 720 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 243:18 → 248:18 | — → — |
| `unknown_captures.tsv` | 721 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 250:26 → 255:26 | — → — |
| `unknown_captures.tsv` | 722 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 275:26 → 280:26 | — → — |
| `unknown_captures.tsv` | 723 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 304:18 → 309:18 | — → — |
| `unknown_captures.tsv` | 724 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 307:26 → 312:26 | — → — |
| `unknown_captures.tsv` | 725 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 343:18 → 348:18 | — → — |
| `unknown_captures.tsv` | 726 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 358:26 → 363:26 | — → — |
| `unknown_captures.tsv` | 727 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 388:18 → 393:18 | — → — |
| `unknown_captures.tsv` | 728 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 391:26 → 396:26 | — → — |
| `unknown_captures.tsv` | 729 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 421:18 → 426:18 | — → — |
| `unknown_captures.tsv` | 730 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 424:26 → 429:26 | — → — |
| `unknown_captures.tsv` | 731 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 451:18 → 456:18 | — → — |
| `unknown_captures.tsv` | 732 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 454:26 → 459:26 | — → — |
| `unknown_captures.tsv` | 1684 | `session/session_requests.rs` | 291:25 → 279:25 | — → — |
| `unknown_captures.tsv` | 1685 | `session/session_requests.rs` | 328:37 → 316:37 | — → — |
| `unknown_captures.tsv` | 1686 | `session/session_requests.rs` | 453:35 → 429:35 | — → — |
| `unknown_captures.tsv` | 1687 | `session/session_requests.rs` | 559:33 → 571:33 | — → — |
| `unknown_captures.tsv` | 1688 | `session/session_requests.rs` | 724:14 → 736:14 | — → — |
| `unknown_captures.tsv` | 1689 | `session/session_requests.rs` | 748:15 → 760:15 | — → — |
| `unknown_captures.tsv` | 1690 | `session/session_requests.rs` | 749:14 → 761:14 | — → — |
| `unknown_captures.tsv` | 1691 | `session/session_requests.rs` | 751:29 → 763:29 | — → — |
| `unknown_captures.tsv` | 1692 | `session/session_requests.rs` | 769:18 → 781:18 | — → — |
| `unknown_captures.tsv` | 1693 | `session/session_requests.rs` | 778:29 → 790:29 | — → — |
| `unknown_captures.tsv` | 1763 | `suspension/core.rs` | 191:59 → 196:59 | — → — |
| `unknown_captures.tsv` | 1764 | `suspension/core.rs` | 231:33 → 236:33 | — → — |
| `unknown_macros.tsv` | 103 | `debug_tools/evaluation_data.rs` | 23:23 → 23:23 | — → — |
| `unknown_macros.tsv` | 104 | `debug_tools/evaluation_data.rs` | 130:31 → 130:39 | — → — |
| `unknown_macros.tsv` | 144 | `debug_tools/test_report/evaluation.rs` | 210:17 → 211:17 | — → — |
| `unknown_macros.tsv` | 145 | `debug_tools/test_report/evaluation.rs` | 339:13 → 340:13 | — → — |
| `unknown_macros.tsv` | 146 | `debug_tools/test_report/evaluation.rs` | 346:13 → 347:13 | — → — |
| `unknown_macros.tsv` | 147 | `debug_tools/test_report/evaluation.rs` | 391:9 → 392:9 | — → — |
| `unknown_macros.tsv` | 148 | `debug_tools/test_report/evaluation.rs` | 398:9 → 399:9 | — → — |
| `unknown_macros.tsv` | 166 | `debug_tools/trace.rs` | 166:32 → 160:32 | — → — |
| `unknown_macros.tsv` | 167 | `debug_tools/trace.rs` | 212:32 → 206:32 | — → — |
| `unknown_macros.tsv` | 168 | `debug_tools/trace.rs` | 271:30 → 265:30 | — → — |
| `unknown_macros.tsv` | 169 | `debug_tools/trace.rs` | 281:30 → 275:30 | — → — |
| `unknown_macros.tsv` | 170 | `debug_tools/trace.rs` | 292:9 → 286:9 | — → — |
| `unknown_macros.tsv` | 171 | `debug_tools/trace.rs` | 294:30 → 288:30 | — → — |
| `unknown_macros.tsv` | 172 | `debug_tools/trace.rs` | 307:65 → 301:65 | — → — |
| `unknown_macros.tsv` | 173 | `debug_tools/trace.rs` | 313:37 → 307:37 | — → — |
| `unknown_macros.tsv` | 174 | `debug_tools/trace.rs` | 396:30 → 390:30 | — → — |
| `unknown_macros.tsv` | 175 | `debug_tools/trace.rs` | 430:18 → 424:18 | — → — |
| `unknown_macros.tsv` | 549 | `kernel_data/control_data/confirmation/builtins/resume_frames.rs` | 348:17 → 353:17 | — → — |
| `unknown_macros.tsv` | 1712 | `session/session_requests.rs` | 545:12 → 557:12 | — → — |
| `unknown_macros.tsv` | 1713 | `session/session_requests.rs` | 565:13 → 577:13 | — → — |
| `unknown_macros.tsv` | 1714 | `session/session_requests.rs` | 582:12 → 594:12 | — → — |
| `unknown_macros.tsv` | 1715 | `session/session_requests.rs` | 608:9 → 620:9 | — → — |
| `unknown_macros.tsv` | 1716 | `session/session_requests.rs` | 631:9 → 643:9 | — → — |
| `unknown_macros.tsv` | 1717 | `session/session_requests.rs` | 653:17 → 665:17 | — → — |
| `unknown_macros.tsv` | 1718 | `session/session_requests.rs` | 756:5 → 768:5 | — → — |
| `unknown_macros.tsv` | 1719 | `session/session_requests.rs` | 829:22 → 841:22 | — → — |
| `unknown_macros.tsv` | 1720 | `session/session_requests.rs` | 832:22 → 844:22 | — → — |
| `unknown_macros.tsv` | 1721 | `session/session_requests.rs` | 841:22 → 853:22 | — → — |
| `unknown_macros.tsv` | 1722 | `session/session_requests.rs` | 858:22 → 870:22 | — → — |
| `unknown_macros.tsv` | 1783 | `suspension/core.rs` | 235:9 → 240:9 | — → — |
| `unknown_macros.tsv` | 1784 | `suspension/core.rs` | 253:9 → 258:9 | — → — |
| `unknown_macros.tsv` | 1785 | `suspension/core.rs` | 587:9 → 540:9 | — → — |
| `unknown_macros.tsv` | 1786 | `suspension/core.rs` | 595:9 → 548:9 | — → — |
| `unknown_macros.tsv` | 1787 | `suspension/core.rs` | 611:9 → 564:9 | — → — |
| `unknown_macros.tsv` | 1788 | `suspension/core.rs` | 666:17 → 619:17 | — → — |
| `unknown_macros.tsv` | 1789 | `suspension/core.rs` | 706:13 → 659:13 | — → — |
| `unknown_macros.tsv` | 1790 | `suspension/core.rs` | 731:28 → 685:28 | — → — |
| `unknown_macros.tsv` | 1791 | `suspension/core.rs` | 839:1 → 793:1 | — → — |
| `unknown_storage.tsv` | 26 | `debug_tools/evaluation_data.rs` | 211:25 → 213:25 | — → — |
| `unknown_storage.tsv` | 28 | `debug_tools/test_report/evaluation.rs` | 427:14 → 428:14 | — → — |
| `unknown_storage.tsv` | 29 | `debug_tools/test_report/evaluation.rs` | 452:14 → 453:14 | — → — |
| `unknown_storage.tsv` | 33 | `debug_tools/trace/resume_frames.rs` | 47:23 → 52:23 | — → — |
| `host_payload_ledger.tsv` | 50 | `session/session_requests.rs` | 280:16 → 268:16 | @44 → @44 |
| `host_payload_ledger.tsv` | 51 | `session/session_requests.rs` | 432:15 → 408:15 | @44 → @44 |
| `host_payload_ledger.tsv` | 52 | `session/session_requests.rs` | 467:15 → 443:15 | @44 → @44 |
| `host_payload_ledger.tsv` | 53 | `suspension/core.rs` | 21:13 → 21:24 | — → — |
| `host_payload_ledger.tsv` | 54 | `suspension/core.rs` | 101:21 → 106:21 | — → — |
| `host_payload_ledger.tsv` | 55 | `suspension/core.rs` | 103:14 → 108:14 | — → — |
| `host_payload_ledger.tsv` | 56 | `suspension/core.rs` | 113:48 → 118:48 | @106 → @111 |
| `host_payload_ledger.tsv` | 57 | `suspension/core.rs` | 124:35 → 129:35 | @106 → @111 |
| `host_payload_ledger.tsv` | 58 | `suspension/core.rs` | 552:57 → 505:57 | @486 → @439 |

`self_reaching.tsv` is unchanged (59 rows). The six changed ownership files are `host_payloads.tsv`, `host_payload_ledger.tsv`, `unknown_attributes.tsv`, `unknown_captures.tsv`, `unknown_macros.tsv`, `unknown_storage.tsv` under `crates/rum-stdlib/src/tests/ownership_graph/inventory/`.

Retention fingerprint replay first reproduced all 68 original fingerprints exactly from the saved initial inputs using the repository's length-delimited FNV rule, then recomputed current fingerprints. It was replayed again after the registration reversal. No identity, guarantee, strength, precondition or exclusion cell changed. Verbatim final replay:

```text
Replayed and installed 68 fingerprint cells; initial replay exact; all nonfingerprint fields preserved.
```

Every changed fingerprint cell in `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` (`fingerprint-row-changes.json` contains the same table):

| Row | ID | Before | After |
| ---: | --- | --- | --- |
| 2 | B01 | `fnv1a64:66f1cf146a632660` | `fnv1a64:60a2c851b4146fc6` |
| 3 | B02 | `fnv1a64:6cb46ccfd7fa9351` | `fnv1a64:19220e6f74f00a53` |
| 4 | B03 | `fnv1a64:c8cc4403920b1262` | `fnv1a64:f0e904645c1f9778` |
| 5 | B04 | `fnv1a64:01747b152c0c3d58` | `fnv1a64:cdeaa381cc8bb63e` |
| 6 | B05 | `fnv1a64:69fdf7b4be7d10bd` | `fnv1a64:15bcb47f15dac497` |
| 7 | B06 | `fnv1a64:b8ca75a08b805164` | `fnv1a64:67cb60bd0b0ec3c2` |
| 8 | B07 | `fnv1a64:119b524e055bbc4d` | `fnv1a64:e51dc01bfc37fc47` |
| 9 | B08 | `fnv1a64:575cc390aaad342e` | `fnv1a64:4016a3fbea5aa5cc` |
| 10 | B09 | `fnv1a64:f15505849cfe3e5c` | `fnv1a64:62f254d02412d49a` |
| 11 | B10 | `fnv1a64:ca2c074584e683f3` | `fnv1a64:093e75d54e59f0d5` |
| 12 | B11 | `fnv1a64:c2cc2c7246006358` | `fnv1a64:8f4254dee67fdc3e` |
| 13 | B12 | `fnv1a64:b06185807906d35c` | `fnv1a64:21fed4cc001b699a` |
| 14 | I01 | `fnv1a64:44d3ba11d71b1ce0` | `fnv1a64:3e84b34f20cc6646` |
| 15 | I02 | `fnv1a64:c13f7548b6b398bd` | `fnv1a64:6cfe32130e114c97` |
| 16 | I03 | `fnv1a64:e42878d4c65fa91c` | `fnv1a64:d0797e47b0baa4da` |
| 17 | I04 | `fnv1a64:e1c5f166b420ea71` | `fnv1a64:48812eb4abdcaff3` |
| 18 | P01 | `fnv1a64:a6399fd467900704` | `fnv1a64:dadcaa6024c68a62` |
| 19 | P02 | `fnv1a64:ecf5d2ff7214c263` | `fnv1a64:09c47c03c64242e5` |
| 20 | P03 | `fnv1a64:afe58d8e75eced8a` | `fnv1a64:317d42f81df73ea0` |
| 21 | P04 | `fnv1a64:618f8ae55ca3af51` | `fnv1a64:0dfd2c84f9992653` |
| 22 | P05 | `fnv1a64:62446d29c80d7dd0` | `fnv1a64:60add00d1ca64c16` |
| 23 | P06 | `fnv1a64:00e0aa5105366cdc` | `fnv1a64:727df99c8c4b031a` |
| 24 | P070 | `fnv1a64:3b2e4692e3abe6dc` | `fnv1a64:accb95de6ac07d1a` |
| 25 | P080 | `fnv1a64:859de287665482b3` | `fnv1a64:a936749b6bde1215` |
| 26 | P090 | `fnv1a64:67c3380a66648711` | `fnv1a64:92aa021576ce8093` |
| 27 | P100 | `fnv1a64:1b09185fa9b96b68` | `fnv1a64:696014e28f4cae6e` |
| 28 | P110 | `fnv1a64:ff4d789e47e03cfb` | `fnv1a64:c908087e353186fd` |
| 29 | P120 | `fnv1a64:6166f28efae708ce` | `fnv1a64:264f9c523bad1aec` |
| 30 | P130 | `fnv1a64:b568abab340f964d` | `fnv1a64:88eb19792aebd647` |
| 31 | P140 | `fnv1a64:4b3fc21e18315239` | `fnv1a64:c7469c70df3375fb` |
| 32 | P150 | `fnv1a64:8ca84e0f54ccde31` | `fnv1a64:2ea4abe9ed225a33` |
| 33 | P071 | `fnv1a64:91d3b4541e317eed` | `fnv1a64:90c82c7e304edc67` |
| 34 | P081 | `fnv1a64:3b62687e95b0767c` | `fnv1a64:7d15278f250c5fba` |
| 35 | P091 | `fnv1a64:d64fb8ccee1be4c4` | `fnv1a64:03f419caae8b89a2` |
| 36 | P101 | `fnv1a64:f750696b7d43d9fb` | `fnv1a64:c10af94b6a9523fd` |
| 37 | P111 | `fnv1a64:b3d227e3e4aea7ec` | `fnv1a64:f03aa00dfedf32ca` |
| 38 | P121 | `fnv1a64:b322e941b7748677` | `fnv1a64:c8a6d5f3ac9e72f1` |
| 39 | P131 | `fnv1a64:9f1e9d7b62605ed8` | `fnv1a64:6b94c5e802dfd7be` |
| 40 | P141 | `fnv1a64:15f2f629e8ae8724` | `fnv1a64:2f295c7892184802` |
| 41 | P151 | `fnv1a64:35b90ed90e2dd50e` | `fnv1a64:b981f5e0bc0924ac` |
| 42 | B13 | `fnv1a64:46db0929bc1d0d64` | `fnv1a64:f5dbf4463bab7fc2` |
| 43 | R01 | `fnv1a64:e560525cb10d2d21` | `fnv1a64:313e6fc725fdd403` |
| 44 | R02 | `fnv1a64:bd6b40718b1c874a` | `fnv1a64:c0cf79a66e61bce0` |
| 45 | R03 | `fnv1a64:96789a881de2dd65` | `fnv1a64:6a0ce9f70e3307ff` |
| 46 | R040 | `fnv1a64:8c904c23cd4c22c1` | `fnv1a64:470c525b95cb4d23` |
| 47 | R050 | `fnv1a64:c8e79c2fd4b435b0` | `fnv1a64:60cb2c557e0b5876` |
| 48 | R060 | `fnv1a64:fc3ddc26cb255909` | `fnv1a64:ef200d8c5823d16b` |
| 49 | R070 | `fnv1a64:a3d030987752357c` | `fnv1a64:e582efa906ae1eba` |
| 50 | R080 | `fnv1a64:a8ca5a5983a5b371` | `fnv1a64:0f8597a77b6178f3` |
| 51 | R090 | `fnv1a64:302c6dee9e128e28` | `fnv1a64:9ef221c5e856f0ae` |
| 52 | R100 | `fnv1a64:8df1e8563be0bce2` | `fnv1a64:b60ea8b705f541f8` |
| 53 | R110 | `fnv1a64:b00ede83b3ea330b` | `fnv1a64:149771640564fead` |
| 54 | R041 | `fnv1a64:2d146c53d344cc64` | `fnv1a64:dc15577052d33ec2` |
| 55 | R051 | `fnv1a64:f5e5661ff3929391` | `fnv1a64:20cc302b03fc8d13` |
| 56 | R061 | `fnv1a64:85eda9ad31dae71e` | `fnv1a64:e9c2f31aeae135dc` |
| 57 | R071 | `fnv1a64:b74cc5753943ac0f` | `fnv1a64:75322fbffaa099e9` |
| 58 | R081 | `fnv1a64:d6f381b445654068` | `fnv1a64:254a7e372af8836e` |
| 59 | R091 | `fnv1a64:e1b0ff1f77468979` | `fnv1a64:49d82ab7d5b8dabb` |
| 60 | R101 | `fnv1a64:9f760f3d41e40ec9` | `fnv1a64:5a7d2016312bf9ab` |
| 61 | R111 | `fnv1a64:88f986cf7a3a1c66` | `fnv1a64:06ba9ab3feec91c4` |
| 62 | R12 | `fnv1a64:aa989930ef9e9fa6` | `fnv1a64:8b45f65c6c2cdf84` |
| 63 | R13 | `fnv1a64:80f6128a1d7c43d2` | `fnv1a64:4f3a7c99f11e9488` |
| 64 | R14 | `fnv1a64:356126d5866a2b68` | `fnv1a64:83b823586bfd6e6e` |
| 65 | R15 | `fnv1a64:554265c6f7850ed4` | `fnv1a64:d9c932524f829192` |
| 66 | O010 | `fnv1a64:1fef061ec6ebf2fe` | `fnv1a64:ba6640f779d767bc` |
| 67 | O0200 | `fnv1a64:460763f266e87839` | `fnv1a64:c20e3e452dea9bfb` |
| 68 | O011 | `fnv1a64:3ec698aed8b71d89` | `fnv1a64:31a8ca1465b595eb` |
| 69 | O0210 | `fnv1a64:c5be44bafeea4986` | `fnv1a64:5f22cfe4b6978ee4` |

Both certificate replays ran the real local production acceptance gate. The first run for each build reached zero rejections for C01/C02 and all four probe bodies, then rejected only the stale stored C01 instance certificate. The newly generated legacy-writer certificates were copied into exactly two cells per build (C01 and C02); every other field in `promoted.tsv` was preserved. Verbatim initial failures:

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.19s
```

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.63s
```

Verbatim green receipts after installation (`certificates-true-green.log`, then `certificates-false-green.log`):

```text
test tests::retention_closure::local::production::local_production_acceptance ... body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.31s
```

```text
test tests::retention_closure::local::production::local_production_acceptance ... body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.69s
```

Every changed certificate cell in `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` is disclosed below by row/build, marks and complete-cell SHA-256. Full before/after cells are in `certificate-cell-changes-{true,false}.json`; generated files and acceptance receipts are copied under `.temp/brief-595/` from the authorized legacy writer.

| Row / ID | Zero-based column / build | Marks before → after | Complete cell SHA-256 before → after |
| --- | --- | --- | --- |
| 2 / C01 | 4 / hash=true | `fnv1a64:549b217b3187bb25,fnv1a64:90a8716a056527ab` → `fnv1a64:4094285a3b499c2e,fnv1a64:d406fc83e4a7d568` | `0d6ed5c1a69236713dcf79ccfe7d33ee69870a7c03120d597f9fca19569ad10c` → `9779a7819ebb7d36a374a6e51558fb9e35a78684999659de495f945c57d41b31` |
| 3 / C02 | 4 / hash=true | `fnv1a64:72b0a6e70775ebb4` → `fnv1a64:e618c869e36c38c5` | `9bc7e96128c6ad31a3a3d487190333f3d3c3a70efcce6fa7628b665789f2138d` → `d5689940f98807a5883f7389bfa8de88c313b9dcc8e4a3385e3caa1d1103129f` |
| 2 / C01 | 5 / hash=false | `fnv1a64:b0dfbc9e4cc685cb,fnv1a64:7c8c4c8fbfc7db51` → `fnv1a64:c288dee6f8037070,fnv1a64:ab05bd38b6196006` | `b7dc2838182494f410c0a38b428cc6324933921d906fe442926f583b79d78539` → `0cf642b93e9bbfabf7b44528175e6e7e4a2a275ae60081b43fafe6f1d4b3d27a` |
| 3 / C02 | 5 / hash=false | `fnv1a64:ec1168016699ab64` → `fnv1a64:f6bd80643710e4cf` | `a2d7f158e22a2f84b562f69cd63c7e38b044722d1a3bc8d21c83a2092f6d8729` → `e7086bd1abb2bba8ec3b4a9deb80485198c96d4e0ef174d347cada45fcb18c21` |

There are **no retained changes** to oracle `inventory/effects.tsv`, `inventory/owed.tsv`, or `fixtures/primary.rs`: each is byte-identical to its initial snapshot. After the passing role/oracle checks, a temporary registration attempt changed ten effects rows (the four new roles and six existing-entry supplemental descriptors) and removed the four owed rows. The later mandatory runtime failure invalidated that attempt, so only those registration edits were reversed. The attempted bytes and exact ten/four row changes remain as `attempted-{effects.tsv,owed.tsv,primary.rs,oracle-inventory-row-changes.json}`. The retained `oracle-inventory-row-changes.json` is `[]`.

### 3. Entries, noise floors and every measured row

Entries attached: **0/4**. Owed: **114 before → 114 after**. The required 114 → 110 reduction was not retained. The four rows still owed are:

```text
EvaluationData.body	debug	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
TestReport.expectedOutput	debug	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
TestReport.input	debug	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
TestReport.sameTest	debug	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
```

Instruction panel: 12 labels; all base and candidate workload result checks passed before measurement. Noise floors were established before any production edit from two serial base/base pairs. Release builds used one codegen unit in the prescribed fresh default/alloc target directories; alloc feature was passed as the separate arguments `--features rum-cli/alloc-count`. Metrics used `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label/process, stdin `/dev/null`, and `(count(2N)-count(N))/N`. Allocation rows use the same difference. No timing decision or timing suite was used, and no concurrent CPU-heavy workload ran during measurement. Base and candidate binaries and hashes:

`base/binary-sha256.json`:

```json
{
  "rum-cli": "bf4ea66f76406dee9ea6143e65698db83fb78e125a15de4cad291c73ba845058",
  "rum-cli-alloc": "d7eed5cea6fea9130c89f33a4320439bb514046cc52223dee44af67e28a55a29"
}
```

`candidate-1/binary-sha256.json`:

```json
{
  "rum-cli": "00534d4d03e5327ed2b86a7fade2d924cd4ec11f0888d90cf590f5e601c5c707",
  "rum-cli-alloc": "481dc12db2bfb9f73f90bcf92cddbee715fd8c12b6296d5f1c6b88284037dff4"
}
```

Noise floor rows, verbatim (`perf/floors.tsv`; base1a/1b and base2a/2b are the two pairs):

```tsv
label	base_1a	base_1b	base_2a	base_2b	floor_percent	threshold_percent
ordinary-arithmetic	17798.518500	17798.677800	17798.506150	17798.698500	0.001081	0.050000
check-no-output	10526.326700	10526.408050	10526.405200	10526.408350	0.000773	0.050000
quiet-no-output	10245.314950	10245.432400	10245.326000	10245.232800	0.001146	0.050000
evaluation-data-no-output	55878.534500	55746.653000	55831.264000	55815.317000	0.236015	0.236015
test-report-no-output	94481.435000	94341.369000	94405.837000	94373.772500	0.148247	0.148247
trace-no-output	24510.064000	24515.564500	24482.357500	24826.211000	1.404495	1.404495
buffered-message	10362.583450	10362.579000	10362.399200	10362.538250	0.001342	0.050000
evaluation-data-message	65149.647000	65214.146000	65305.153500	65243.278000	0.099001	0.099001
check-message	14358.095950	14358.002750	14358.113650	14358.190750	0.000649	0.050000
dot-three-no-input	254200.329800	254199.224400	254200.426750	254199.660450	0.000435	0.050000
differences-no-input	10037.767350	10044.851450	10052.202400	10044.916850	0.072477	0.072477
take-multispec-small	49939.920700	49923.313050	49923.074100	49923.231850	0.033255	0.050000
```

All paired instruction and allocation comparisons, verbatim (`candidate-1/comparison.tsv`; instruction values are per-iteration differences):

```tsv
label	base1	candidate1	delta1_percent	base2	candidate2	delta2_percent	floor_percent	regresses	alloc_base1	alloc_base2	alloc_candidate1	alloc_candidate2
ordinary-arithmetic	17798.522300	17798.424400	-0.000550	17798.596100	17798.519300	-0.000431	0.001081	False	38.000000	38.000000	38.000000	38.000000
check-no-output	10526.482300	10526.399000	-0.000791	10526.441250	10526.411700	-0.000281	0.000773	False	6.000000	6.000000	6.000000	6.000000
quiet-no-output	10245.407800	10245.267500	-0.001369	10245.594550	10245.269550	-0.003172	0.001146	False	6.000000	6.000000	6.000000	6.000000
evaluation-data-no-output	55857.183000	55803.364500	-0.096350	55829.978000	55718.728000	-0.199266	0.236015	False	159.155000	159.454000	157.798000	158.166000
test-report-no-output	94452.727000	94306.904500	-0.154387	94207.555000	93692.130500	-0.547116	0.148247	False	230.096500	229.206500	226.360500	227.209500
trace-no-output	24470.123500	24544.804000	0.305191	24543.574500	24450.573500	-0.378922	1.404495	False	25.000500	25.000500	25.000500	25.000500
buffered-message	10362.540000	10362.370100	-0.001640	10362.271600	10362.477200	0.001984	0.001342	False	24.000050	24.000050	24.000050	24.000050
evaluation-data-message	65126.064000	65261.414500	0.207828	65141.187500	64948.147500	-0.296341	0.099001	False	194.266500	195.103500	193.516500	193.424500
check-message	14358.140750	14358.128350	-0.000086	14358.150900	14358.197400	0.000324	0.000649	False	25.000050	25.000050	25.000050	25.000050
dot-three-no-input	254199.209500	254198.251900	-0.000377	254199.431050	254198.706400	-0.000285	0.000435	False	596.000000	596.000000	596.000000	596.000000
differences-no-input	10052.093200	10035.732450	-0.162760	10030.638900	10034.347300	0.036971	0.072477	False	13.000000	13.000000	13.000000	13.000000
take-multispec-small	49931.982250	49921.803250	-0.020386	49931.949850	49922.159900	-0.019607	0.033255	False	109.000000	109.000000	109.000000	109.000000
```

Under the declared rule, **zero labels regress**: no label exceeds `max(+0.05%, measured floor)` in both candidate samples. The positive Trace sample (+0.305191%) is below its 1.404495% floor and its other sample is negative. EvaluationData-message is +0.207828% in one sample and −0.296341% in the other; it does not fail the two-sample rule. These variations were retained without selective reruns. Deterministic labels have unchanged allocation counts; timing-valued EvaluationData/TestReport runs vary and show no allocation increase over base. No buffered-Print performance number is claimed, as directed by the amendment; its behavior is covered by buffered runtime/host tests.

Every underlying measured/check row is reproduced below from `.temp/brief-595/all-measurements.tsv`: 24 result checks, 96 noise-floor instruction processes, 96 paired-comparison instruction processes, and 96 allocation processes, **312 rows total**. `pass` combines successful process exit and the checked program result. Blank metric cells are not measured in that mode. Raw process stdout/stderr, instruction CSV and allocation summaries remain under `perf/{check,instructions,alloc}/{build}/{sample}/`; all source programs use `.rum`. These are instruction/allocation measurements, not wall-clock timings.

```tsv
mode	build	sample	label	count	exit	pass	value	expected	instructions	allocations	bytes	peak
check	base	1	ordinary-arithmetic	check	0	True	23	23				
check	base	1	check-no-output	check	0	True	5	5				
check	base	1	quiet-no-output	check	0	True	5	5				
check	base	1	evaluation-data-no-output	check	0	True	5	5				
check	base	1	test-report-no-output	check	0	True	1	1				
check	base	1	trace-no-output	check	0	True	List[Hold[Plus[2, 3]], Hold[5]]	List[Hold[Plus[2, 3]], Hold[5]]				
check	base	1	buffered-message	check	0	True	Null	Null				
check	base	1	evaluation-data-message	check	0	True	"List[""probe message""]"	"List[""probe message""]"				
check	base	1	check-message	check	0	True	-1	-1				
check	base	1	dot-three-no-input	check	0	True	List[List[6, 12], List[18, 24]]	List[List[6, 12], List[18, 24]]				
check	base	1	differences-no-input	check	0	True	List[3, 5, 7, 9]	List[3, 5, 7, 9]				
check	base	1	take-multispec-small	check	0	True	List[List[List[List[List[List[List[List[1]]]]]]]]	List[List[List[List[List[List[List[List[1]]]]]]]]				
check	candidate-1	1	ordinary-arithmetic	check	0	True	23	23				
check	candidate-1	1	check-no-output	check	0	True	5	5				
check	candidate-1	1	quiet-no-output	check	0	True	5	5				
check	candidate-1	1	evaluation-data-no-output	check	0	True	5	5				
check	candidate-1	1	test-report-no-output	check	0	True	1	1				
check	candidate-1	1	trace-no-output	check	0	True	List[Hold[Plus[2, 3]], Hold[5]]	List[Hold[Plus[2, 3]], Hold[5]]				
check	candidate-1	1	buffered-message	check	0	True	Null	Null				
check	candidate-1	1	evaluation-data-message	check	0	True	"List[""probe message""]"	"List[""probe message""]"				
check	candidate-1	1	check-message	check	0	True	-1	-1				
check	candidate-1	1	dot-three-no-input	check	0	True	List[List[6, 12], List[18, 24]]	List[List[6, 12], List[18, 24]]				
check	candidate-1	1	differences-no-input	check	0	True	List[3, 5, 7, 9]	List[3, 5, 7, 9]				
check	candidate-1	1	take-multispec-small	check	0	True	List[List[List[List[List[List[List[List[1]]]]]]]]	List[List[List[List[List[List[List[List[1]]]]]]]]				
alloc	base	1	ordinary-arithmetic	20000	0	True				761640	26403019	941598
alloc	base	1	ordinary-arithmetic	40000	0	True				1521640	52323019	1581598
alloc	base	1	check-no-output	20000	0	True				121645	9123581	941851
alloc	base	1	check-no-output	40000	0	True				241645	17763581	1581851
alloc	base	1	quiet-no-output	20000	0	True				121638	9123461	941780
alloc	base	1	quiet-no-output	40000	0	True				241638	17763461	1581780
alloc	base	1	evaluation-data-no-output	2000	0	True				320341	23543964	372471
alloc	base	1	evaluation-data-no-output	4000	0	True				638651	46596164	436471
alloc	base	1	test-report-no-output	2000	0	True				460947	33353217	4429191
alloc	base	1	test-report-no-output	4000	0	True				921140	66237857	8554890
alloc	base	1	trace-no-output	2000	0	True				51614	5706918	2233562
alloc	base	1	trace-no-output	4000	0	True				101615	10934326	4167026
alloc	base	1	buffered-message	20000	0	True				481721	21745374	8261969
alloc	base	1	buffered-message	40000	0	True				961722	43003966	16261265
alloc	base	1	evaluation-data-message	2000	0	True				391032	25987874	1300314
alloc	base	1	evaluation-data-message	4000	0	True				779565	51472226	2296922
alloc	base	1	check-message	20000	0	True				501737	22546255	8261966
alloc	base	1	check-message	40000	0	True				1001738	44604847	16261262
alloc	base	1	dot-three-no-input	20000	0	True				11921740	477979704	8678236
alloc	base	1	dot-three-no-input	40000	0	True				23841740	948059704	9318236
alloc	base	1	differences-no-input	20000	0	True				261670	21969680	9591399
alloc	base	1	differences-no-input	40000	0	True				521670	36049680	10231399
alloc	base	1	take-multispec-small	20000	0	True				2181802	125022546	9594023
alloc	base	1	take-multispec-small	40000	0	True				4361802	242142546	10234023
alloc	base	2	ordinary-arithmetic	20000	0	True				761640	26403019	941598
alloc	base	2	ordinary-arithmetic	40000	0	True				1521640	52323019	1581598
alloc	base	2	check-no-output	20000	0	True				121645	9123581	941851
alloc	base	2	check-no-output	40000	0	True				241645	17763581	1581851
alloc	base	2	quiet-no-output	20000	0	True				121638	9123461	941780
alloc	base	2	quiet-no-output	40000	0	True				241638	17763461	1581780
alloc	base	2	evaluation-data-no-output	2000	0	True				320321	23543564	372471
alloc	base	2	evaluation-data-no-output	4000	0	True				639229	46607724	436471
alloc	base	2	test-report-no-output	2000	0	True				460461	33343497	4429191
alloc	base	2	test-report-no-output	4000	0	True				918874	66192537	8554890
alloc	base	2	trace-no-output	2000	0	True				51614	5706918	2233562
alloc	base	2	trace-no-output	4000	0	True				101615	10934326	4167026
alloc	base	2	buffered-message	20000	0	True				481721	21745374	8261969
alloc	base	2	buffered-message	40000	0	True				961722	43003966	16261265
alloc	base	2	evaluation-data-message	2000	0	True				390672	25980674	1300314
alloc	base	2	evaluation-data-message	4000	0	True				780879	51498506	2296922
alloc	base	2	check-message	20000	0	True				501737	22546255	8261966
alloc	base	2	check-message	40000	0	True				1001738	44604847	16261262
alloc	base	2	dot-three-no-input	20000	0	True				11921740	477979704	8678236
alloc	base	2	dot-three-no-input	40000	0	True				23841740	948059704	9318236
alloc	base	2	differences-no-input	20000	0	True				261670	21969680	9591399
alloc	base	2	differences-no-input	40000	0	True				521670	36049680	10231399
alloc	base	2	take-multispec-small	20000	0	True				2181802	125022546	9594023
alloc	base	2	take-multispec-small	40000	0	True				4361802	242142546	10234023
alloc	candidate-1	1	ordinary-arithmetic	20000	0	True				761640	26403026	941598
alloc	candidate-1	1	ordinary-arithmetic	40000	0	True				1521640	52323026	1581598
alloc	candidate-1	1	check-no-output	20000	0	True				121645	9123588	941851
alloc	candidate-1	1	check-no-output	40000	0	True				241645	17763588	1581851
alloc	candidate-1	1	quiet-no-output	20000	0	True				121638	9123468	941780
alloc	candidate-1	1	quiet-no-output	40000	0	True				241638	17763468	1581780
alloc	candidate-1	1	evaluation-data-no-output	2000	0	True				317119	23479531	372583
alloc	candidate-1	1	evaluation-data-no-output	4000	0	True				632715	46477451	436583
alloc	candidate-1	1	test-report-no-output	2000	0	True				457755	33289384	4429191
alloc	candidate-1	1	test-report-no-output	4000	0	True				910476	66024584	8554890
alloc	candidate-1	1	trace-no-output	2000	0	True				51614	5706925	2233562
alloc	candidate-1	1	trace-no-output	4000	0	True				101615	10934333	4167026
alloc	candidate-1	1	buffered-message	20000	0	True				481721	21745381	8261969
alloc	candidate-1	1	buffered-message	40000	0	True				961722	43003973	16261265
alloc	candidate-1	1	evaluation-data-message	2000	0	True				388470	25936641	1300314
alloc	candidate-1	1	evaluation-data-message	4000	0	True				775503	51390993	2296922
alloc	candidate-1	1	check-message	20000	0	True				501737	22546262	8261966
alloc	candidate-1	1	check-message	40000	0	True				1001738	44604854	16261262
alloc	candidate-1	1	dot-three-no-input	20000	0	True				11921740	477979711	8678236
alloc	candidate-1	1	dot-three-no-input	40000	0	True				23841740	948059711	9318236
alloc	candidate-1	1	differences-no-input	20000	0	True				261670	21969687	9591399
alloc	candidate-1	1	differences-no-input	40000	0	True				521670	36049687	10231399
alloc	candidate-1	1	take-multispec-small	20000	0	True				2181802	125022553	9594023
alloc	candidate-1	1	take-multispec-small	40000	0	True				4361802	242142553	10234023
alloc	candidate-1	2	ordinary-arithmetic	20000	0	True				761640	26403026	941598
alloc	candidate-1	2	ordinary-arithmetic	40000	0	True				1521640	52323026	1581598
alloc	candidate-1	2	check-no-output	20000	0	True				121645	9123588	941851
alloc	candidate-1	2	check-no-output	40000	0	True				241645	17763588	1581851
alloc	candidate-1	2	quiet-no-output	20000	0	True				121638	9123468	941780
alloc	candidate-1	2	quiet-no-output	40000	0	True				241638	17763468	1581780
alloc	candidate-1	2	evaluation-data-no-output	2000	0	True				317463	23486411	372583
alloc	candidate-1	2	evaluation-data-no-output	4000	0	True				633795	46499051	436583
alloc	candidate-1	2	test-report-no-output	2000	0	True				455197	33238224	4429191
alloc	candidate-1	2	test-report-no-output	4000	0	True				909616	66007384	8554890
alloc	candidate-1	2	trace-no-output	2000	0	True				51614	5706925	2233562
alloc	candidate-1	2	trace-no-output	4000	0	True				101615	10934333	4167026
alloc	candidate-1	2	buffered-message	20000	0	True				481721	21745381	8261969
alloc	candidate-1	2	buffered-message	40000	0	True				961722	43003973	16261265
alloc	candidate-1	2	evaluation-data-message	2000	0	True				389330	25953841	1300314
alloc	candidate-1	2	evaluation-data-message	4000	0	True				776179	51404513	2296922
alloc	candidate-1	2	check-message	20000	0	True				501737	22546262	8261966
alloc	candidate-1	2	check-message	40000	0	True				1001738	44604854	16261262
alloc	candidate-1	2	dot-three-no-input	20000	0	True				11921740	477979711	8678236
alloc	candidate-1	2	dot-three-no-input	40000	0	True				23841740	948059711	9318236
alloc	candidate-1	2	differences-no-input	20000	0	True				261670	21969687	9591399
alloc	candidate-1	2	differences-no-input	40000	0	True				521670	36049687	10231399
alloc	candidate-1	2	take-multispec-small	20000	0	True				2181802	125022553	9594023
alloc	candidate-1	2	take-multispec-small	40000	0	True				4361802	242142553	10234023
instructions	base	candidate-1-pair-1	ordinary-arithmetic	20000	0	True			359483056			
instructions	base	candidate-1-pair-1	ordinary-arithmetic	40000	0	True			715453502			
instructions	base	candidate-1-pair-1	check-no-output	20000	0	True			214041059			
instructions	base	candidate-1-pair-1	check-no-output	40000	0	True			424570705			
instructions	base	candidate-1-pair-1	quiet-no-output	20000	0	True			208418673			
instructions	base	candidate-1-pair-1	quiet-no-output	40000	0	True			413326829			
instructions	base	candidate-1-pair-1	evaluation-data-no-output	2000	0	True			115473124			
instructions	base	candidate-1-pair-1	evaluation-data-no-output	4000	0	True			227187490			
instructions	base	candidate-1-pair-1	test-report-no-output	2000	0	True			193104603			
instructions	base	candidate-1-pair-1	test-report-no-output	4000	0	True			382010057			
instructions	base	candidate-1-pair-1	trace-no-output	2000	0	True			52391966			
instructions	base	candidate-1-pair-1	trace-no-output	4000	0	True			101332213			
instructions	base	candidate-1-pair-1	buffered-message	20000	0	True			210253913			
instructions	base	candidate-1-pair-1	buffered-message	40000	0	True			417504713			
instructions	base	candidate-1-pair-1	evaluation-data-message	2000	0	True			133703539			
instructions	base	candidate-1-pair-1	evaluation-data-message	4000	0	True			263955667			
instructions	base	candidate-1-pair-1	check-message	20000	0	True			290178825			
instructions	base	candidate-1-pair-1	check-message	40000	0	True			577341640			
instructions	base	candidate-1-pair-1	dot-three-no-input	20000	0	True			5090683443			
instructions	base	candidate-1-pair-1	dot-three-no-input	40000	0	True			10174667633			
instructions	base	candidate-1-pair-1	differences-no-input	20000	0	True			206397679			
instructions	base	candidate-1-pair-1	differences-no-input	40000	0	True			407439543			
instructions	base	candidate-1-pair-1	take-multispec-small	20000	0	True			1003953780			
instructions	base	candidate-1-pair-1	take-multispec-small	40000	0	True			2002593425			
instructions	base	candidate-1-pair-2	ordinary-arithmetic	20000	0	True			359483710			
instructions	base	candidate-1-pair-2	ordinary-arithmetic	40000	0	True			715455632			
instructions	base	candidate-1-pair-2	check-no-output	20000	0	True			214042342			
instructions	base	candidate-1-pair-2	check-no-output	40000	0	True			424571167			
instructions	base	candidate-1-pair-2	quiet-no-output	20000	0	True			208418999			
instructions	base	candidate-1-pair-2	quiet-no-output	40000	0	True			413330890			
instructions	base	candidate-1-pair-2	evaluation-data-no-output	2000	0	True			115383172			
instructions	base	candidate-1-pair-2	evaluation-data-no-output	4000	0	True			227043128			
instructions	base	candidate-1-pair-2	test-report-no-output	2000	0	True			193174207			
instructions	base	candidate-1-pair-2	test-report-no-output	4000	0	True			381589317			
instructions	base	candidate-1-pair-2	trace-no-output	2000	0	True			52246337			
instructions	base	candidate-1-pair-2	trace-no-output	4000	0	True			101333486			
instructions	base	candidate-1-pair-2	buffered-message	20000	0	True			210255573			
instructions	base	candidate-1-pair-2	buffered-message	40000	0	True			417501005			
instructions	base	candidate-1-pair-2	evaluation-data-message	2000	0	True			133755105			
instructions	base	candidate-1-pair-2	evaluation-data-message	4000	0	True			264037480			
instructions	base	candidate-1-pair-2	check-message	20000	0	True			290179239			
instructions	base	candidate-1-pair-2	check-message	40000	0	True			577342257			
instructions	base	candidate-1-pair-2	dot-three-no-input	20000	0	True			5090684948			
instructions	base	candidate-1-pair-2	dot-three-no-input	40000	0	True			10174673569			
instructions	base	candidate-1-pair-2	differences-no-input	20000	0	True			206537784			
instructions	base	candidate-1-pair-2	differences-no-input	40000	0	True			407150562			
instructions	base	candidate-1-pair-2	take-multispec-small	20000	0	True			1003951930			
instructions	base	candidate-1-pair-2	take-multispec-small	40000	0	True			2002590927			
instructions	base	floor-1a	ordinary-arithmetic	20000	0	True			359484201			
instructions	base	floor-1a	ordinary-arithmetic	40000	0	True			715454571			
instructions	base	floor-1a	check-no-output	20000	0	True			214042927			
instructions	base	floor-1a	check-no-output	40000	0	True			424569461			
instructions	base	floor-1a	quiet-no-output	20000	0	True			208420840			
instructions	base	floor-1a	quiet-no-output	40000	0	True			413327139			
instructions	base	floor-1a	evaluation-data-no-output	2000	0	True			115448109			
instructions	base	floor-1a	evaluation-data-no-output	4000	0	True			227205178			
instructions	base	floor-1a	test-report-no-output	2000	0	True			193084816			
instructions	base	floor-1a	test-report-no-output	4000	0	True			382047686			
instructions	base	floor-1a	trace-no-output	2000	0	True			52317444			
instructions	base	floor-1a	trace-no-output	4000	0	True			101337572			
instructions	base	floor-1a	buffered-message	20000	0	True			210253164			
instructions	base	floor-1a	buffered-message	40000	0	True			417504833			
instructions	base	floor-1a	evaluation-data-message	2000	0	True			133829599			
instructions	base	floor-1a	evaluation-data-message	4000	0	True			264128893			
instructions	base	floor-1a	check-message	20000	0	True			290182013			
instructions	base	floor-1a	check-message	40000	0	True			577343932			
instructions	base	floor-1a	dot-three-no-input	20000	0	True			5090683875			
instructions	base	floor-1a	dot-three-no-input	40000	0	True			10174690471			
instructions	base	floor-1a	differences-no-input	20000	0	True			206396137			
instructions	base	floor-1a	differences-no-input	40000	0	True			407151484			
instructions	base	floor-1a	take-multispec-small	20000	0	True			1003797188			
instructions	base	floor-1a	take-multispec-small	40000	0	True			2002595602			
instructions	base	floor-1b	ordinary-arithmetic	20000	0	True			359483024			
instructions	base	floor-1b	ordinary-arithmetic	40000	0	True			715456580			
instructions	base	floor-1b	check-no-output	20000	0	True			214042702			
instructions	base	floor-1b	check-no-output	40000	0	True			424570863			
instructions	base	floor-1b	quiet-no-output	20000	0	True			208417388			
instructions	base	floor-1b	quiet-no-output	40000	0	True			413326036			
instructions	base	floor-1b	evaluation-data-no-output	2000	0	True			115467078			
instructions	base	floor-1b	evaluation-data-no-output	4000	0	True			226960384			
instructions	base	floor-1b	test-report-no-output	2000	0	True			193033384			
instructions	base	floor-1b	test-report-no-output	4000	0	True			381716122			
instructions	base	floor-1b	trace-no-output	2000	0	True			52304864			
instructions	base	floor-1b	trace-no-output	4000	0	True			101335993			
instructions	base	floor-1b	buffered-message	20000	0	True			210252146			
instructions	base	floor-1b	buffered-message	40000	0	True			417503726			
instructions	base	floor-1b	evaluation-data-message	2000	0	True			133723000			
instructions	base	floor-1b	evaluation-data-message	4000	0	True			264151292			
instructions	base	floor-1b	check-message	20000	0	True			290183538			
instructions	base	floor-1b	check-message	40000	0	True			577343593			
instructions	base	floor-1b	dot-three-no-input	20000	0	True			5090688724			
instructions	base	floor-1b	dot-three-no-input	40000	0	True			10174673212			
instructions	base	floor-1b	differences-no-input	20000	0	True			206538651			
instructions	base	floor-1b	differences-no-input	40000	0	True			407435680			
instructions	base	floor-1b	take-multispec-small	20000	0	True			1003798598			
instructions	base	floor-1b	take-multispec-small	40000	0	True			2002264859			
instructions	base	floor-2a	ordinary-arithmetic	20000	0	True			359484938			
instructions	base	floor-2a	ordinary-arithmetic	40000	0	True			715455061			
instructions	base	floor-2a	check-no-output	20000	0	True			214043390			
instructions	base	floor-2a	check-no-output	40000	0	True			424571494			
instructions	base	floor-2a	quiet-no-output	20000	0	True			208420320			
instructions	base	floor-2a	quiet-no-output	40000	0	True			413326840			
instructions	base	floor-2a	evaluation-data-no-output	2000	0	True			115426391			
instructions	base	floor-2a	evaluation-data-no-output	4000	0	True			227088919			
instructions	base	floor-2a	test-report-no-output	2000	0	True			193123190			
instructions	base	floor-2a	test-report-no-output	4000	0	True			381934864			
instructions	base	floor-2a	trace-no-output	2000	0	True			52463937			
instructions	base	floor-2a	trace-no-output	4000	0	True			101428652			
instructions	base	floor-2a	buffered-message	20000	0	True			210254413			
instructions	base	floor-2a	buffered-message	40000	0	True			417502397			
instructions	base	floor-2a	evaluation-data-message	2000	0	True			133746849			
instructions	base	floor-2a	evaluation-data-message	4000	0	True			264357156			
instructions	base	floor-2a	check-message	20000	0	True			290181284			
instructions	base	floor-2a	check-message	40000	0	True			577343557			
instructions	base	floor-2a	dot-three-no-input	20000	0	True			5090685559			
instructions	base	floor-2a	dot-three-no-input	40000	0	True			10174694094			
instructions	base	floor-2a	differences-no-input	20000	0	True			206396198			
instructions	base	floor-2a	differences-no-input	40000	0	True			407440246			
instructions	base	floor-2a	take-multispec-small	20000	0	True			1003800701			
instructions	base	floor-2a	take-multispec-small	40000	0	True			2002262183			
instructions	base	floor-2b	ordinary-arithmetic	20000	0	True			359481176			
instructions	base	floor-2b	ordinary-arithmetic	40000	0	True			715455146			
instructions	base	floor-2b	check-no-output	20000	0	True			214043196			
instructions	base	floor-2b	check-no-output	40000	0	True			424571363			
instructions	base	floor-2b	quiet-no-output	20000	0	True			208421153			
instructions	base	floor-2b	quiet-no-output	40000	0	True			413325809			
instructions	base	floor-2b	evaluation-data-no-output	2000	0	True			115469891			
instructions	base	floor-2b	evaluation-data-no-output	4000	0	True			227100525			
instructions	base	floor-2b	test-report-no-output	2000	0	True			193103104			
instructions	base	floor-2b	test-report-no-output	4000	0	True			381850649			
instructions	base	floor-2b	trace-no-output	2000	0	True			52248157			
instructions	base	floor-2b	trace-no-output	4000	0	True			101900579			
instructions	base	floor-2b	buffered-message	20000	0	True			210254405			
instructions	base	floor-2b	buffered-message	40000	0	True			417505170			
instructions	base	floor-2b	evaluation-data-message	2000	0	True			133759129			
instructions	base	floor-2b	evaluation-data-message	4000	0	True			264245685			
instructions	base	floor-2b	check-message	20000	0	True			290180463			
instructions	base	floor-2b	check-message	40000	0	True			577344278			
instructions	base	floor-2b	dot-three-no-input	20000	0	True			5090685283			
instructions	base	floor-2b	dot-three-no-input	40000	0	True			10174678492			
instructions	base	floor-2b	differences-no-input	20000	0	True			206539212			
instructions	base	floor-2b	differences-no-input	40000	0	True			407437549			
instructions	base	floor-2b	take-multispec-small	20000	0	True			1003796220			
instructions	base	floor-2b	take-multispec-small	40000	0	True			2002260857			
instructions	candidate-1	1	ordinary-arithmetic	20000	0	True			359492345			
instructions	candidate-1	1	ordinary-arithmetic	40000	0	True			715460833			
instructions	candidate-1	1	check-no-output	20000	0	True			214049875			
instructions	candidate-1	1	check-no-output	40000	0	True			424577855			
instructions	candidate-1	1	quiet-no-output	20000	0	True			208426327			
instructions	candidate-1	1	quiet-no-output	40000	0	True			413331677			
instructions	candidate-1	1	evaluation-data-no-output	2000	0	True			115427700			
instructions	candidate-1	1	evaluation-data-no-output	4000	0	True			227034429			
instructions	candidate-1	1	test-report-no-output	2000	0	True			193060269			
instructions	candidate-1	1	test-report-no-output	4000	0	True			381674078			
instructions	candidate-1	1	trace-no-output	2000	0	True			52372997			
instructions	candidate-1	1	trace-no-output	4000	0	True			101462605			
instructions	candidate-1	1	buffered-message	20000	0	True			210262969			
instructions	candidate-1	1	buffered-message	40000	0	True			417510371			
instructions	candidate-1	1	evaluation-data-message	2000	0	True			133821206			
instructions	candidate-1	1	evaluation-data-message	4000	0	True			264344035			
instructions	candidate-1	1	check-message	20000	0	True			290187624			
instructions	candidate-1	1	check-message	40000	0	True			577350191			
instructions	candidate-1	1	dot-three-no-input	20000	0	True			5090677568			
instructions	candidate-1	1	dot-three-no-input	40000	0	True			10174642606			
instructions	candidate-1	1	differences-no-input	20000	0	True			206297631			
instructions	candidate-1	1	differences-no-input	40000	0	True			407012280			
instructions	candidate-1	1	take-multispec-small	20000	0	True			1003792859			
instructions	candidate-1	1	take-multispec-small	40000	0	True			2002228924			
instructions	candidate-1	2	ordinary-arithmetic	20000	0	True			359491774			
instructions	candidate-1	2	ordinary-arithmetic	40000	0	True			715462160			
instructions	candidate-1	2	check-no-output	20000	0	True			214049441			
instructions	candidate-1	2	check-no-output	40000	0	True			424577675			
instructions	candidate-1	2	quiet-no-output	20000	0	True			208427777			
instructions	candidate-1	2	quiet-no-output	40000	0	True			413333168			
instructions	candidate-1	2	evaluation-data-no-output	2000	0	True			114865547			
instructions	candidate-1	2	evaluation-data-no-output	4000	0	True			226303003			
instructions	candidate-1	2	test-report-no-output	2000	0	True			191994310			
instructions	candidate-1	2	test-report-no-output	4000	0	True			379378571			
instructions	candidate-1	2	trace-no-output	2000	0	True			52544620			
instructions	candidate-1	2	trace-no-output	4000	0	True			101445767			
instructions	candidate-1	2	buffered-message	20000	0	True			210260551			
instructions	candidate-1	2	buffered-message	40000	0	True			417510095			
instructions	candidate-1	2	evaluation-data-message	2000	0	True			132958339			
instructions	candidate-1	2	evaluation-data-message	4000	0	True			262854634			
instructions	candidate-1	2	check-message	20000	0	True			290186774			
instructions	candidate-1	2	check-message	40000	0	True			577350722			
instructions	candidate-1	2	dot-three-no-input	20000	0	True			5090670473			
instructions	candidate-1	2	dot-three-no-input	40000	0	True			10174644601			
instructions	candidate-1	2	differences-no-input	20000	0	True			206297505			
instructions	candidate-1	2	differences-no-input	40000	0	True			406984451			
instructions	candidate-1	2	take-multispec-small	20000	0	True			1003785051			
instructions	candidate-1	2	take-multispec-small	40000	0	True			2002228249
```

### 4. Stop-loss counters, residuals and director handoff

- Compiling production candidates: **1**; measured candidates: **1/8**. Test-harness corrections, formatting, and maintenance replay did not introduce another production candidate.
- Original strict reds turned green: **6/6**. Attached entries: **0/4**. New watched prefix falsifiers: **2/2**, with both correct controls accepted and exact-pause corruption additionally rejected.
- Regressing measured labels: **0/12**; no positive two-sample regression. No-progress streak: **0**. The two-compiling-candidate stop-loss was not reached. This stop is the explicit scope fence.
- Required runtime gate: **1147 passed, 2 failed, 3 ignored**. The remaining full host/workspace gates and final post-admission oracle remain unrun; no broad green or native ZeroMQ result is claimed.
- Standing duties complete for the retained tree: **68** fingerprint cells, **4** certificate cells, **127** ownership coordinate transitions, **1** new unknown derive row; all other inventory meaning preserved. Both certificates and ownership pass.

The director must resolve the conflict with the out-of-scope guest-loading pause assertions before implementation continues. The report does not assume permission to change them. After that ruling, repair the local trait member ordering without weakening the source invariant, rerun the affected runtime/host/oracle gates, register exactly the four entries only when admission is valid, refresh source-dependent maintenance, and run the remaining mandatory broad gates. Any resulting production revision needs the prescribed instruction-gate treatment; candidate budget has seven slots left.

No production fix was made after the STOP. The patch, standalone coverage, docs, measurements, full logs, attempted registration and current maintenance are preserved for review. `.temp/brief-595/PROGRESS.md` is the current takeover checkpoint; the earlier progress history remains in `PROGRESS-history.md`. The earlier report above describes the first (Trace scope) stop and is historical; this resumed report is the current state.


## Director amendment 2 — the handled-error tail at a pause (2026-10-07)

Resume from `.temp/brief-595/PROGRESS.md` with candidate 1 in the tree.

1. **Ruling: the pause builder is right and the old test encodes the old
   behaviour.** The previous pause path reused the terminal failure builder,
   which truncates `handled_non_loader_eval_errors` at the request checkpoint;
   the guest loader then restored the tail from its own frame on resume. A
   pause is not terminal: truncating there loses handled errors that a running
   capture (`EvaluationData`, `Check`) observes through its checkpoint.
   **Authorized:** edit `crates/rum-runtime/src/runtime/session/guest_loading/tests.rs`
   test `guest_environment_restores_handled_error_tail_and_memory_limit_before_child`
   so that at each pause it asserts the *preserved* tail — exactly one entry,
   text `"saved tail"` — instead of an empty one, and keeps the empty-tail and
   cleared-memory-limit assertions after completion. Do not weaken it to "no
   assertion". If any other existing test asserts truncation at a pause, apply
   the same change and disclose it; if one asserts something else this rule
   contradicts, STOP with the witness.
2. Move the new `output_boundary` default method after `resume` in
   `SuspensionFrame` (`suspension/core.rs`) so the source invariant in
   `runtime/tests/frame_root_trace.rs:354` holds unchanged.
3. Then attach the four G11 entries (owed 114 -> 110), rerun the affected
   runtime, host and oracle gates, refresh the standing duty, and run the
   remaining broad gates. A production change beyond item 2 counts as a new
   candidate under the instruction gate; item 2 alone does not need a new
   measurement if the generated code is unchanged — say how you established
   that.
