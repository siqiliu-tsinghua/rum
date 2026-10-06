# brief-578 — G5 step B: the runtime-valued readers and their ordered phases

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-577 (`ecd7b5b2`) installed the typed read-and-complete protocol
(`crates/rum-runtime/src/runtime/options/read_phase.rs`) and migrated the
boolean readers. This step brings the runtime-valued and syntax readers onto
it, for the builtins whose option reads form ordered phases: **Monitor**
(option syntax, then EvaluationMonitor, then StepMonitor, then the monitor
scope and body), **TestReport** (SameTest, TimeConstraint, MemoryConstraint,
then the report), **Names** (SpellingCorrection and its check,
ResolveContextAliases and its check, IgnoreCase, then enumeration) and
**DumpSave** (CompressionLevel, then the write). Fourteen sites — P0118,
P0119, P0120, P0122, P0157, P0158, P0159, P0161, P0167, P0168, P0962, P0963,
P0964, P0997. Lost continuations **31 -> 17**.

brief-576's report and `.temp/brief-576/draft-G5-B.md` are the raw material;
**this brief governs** where they differ. The rulings recorded in brief-577
(no legacy getter; I04's host-input rows by name; one-unit instruction gate,
no timing suite) stand.

## Context pointers

- brief-576's report (§§2-3: the inventory, the phase rules, the frozen state
  S09-S12 and T01-T05), `.temp/brief-576/{inventory.md,design.md,inventory-plan.md,performance-plan.md,cases.json,probe.rs,oracle.tsv}`.
- brief-577 and its report: the accepted protocol, its compile controls, the
  source tail-use gate (`crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs`),
  and the harness `crates/rum-stdlib/tests/option_reader_continuations/`.
- The certified C01 (`options/core.rs`, merged reader) and C02
  (`evaluate_runtime_option_expr`) — verbatim.

## Scope

**Objective:** the runtime and syntax decoders, the four phased builtins'
continuations, their tests, and the standing and performance gates.

**Edit surface:** `runtime/options/{core.rs,read_phase.rs}`;
`debug_tools/{monitor.rs,monitor/options.rs}`;
`debug_tools/test_report/{evaluation.rs,report.rs,options.rs}`;
`namespace_tools/{names.rs,names/options.rs}`; `package_tools.rs` and
`package_tools/persist_options.rs`; module declarations; additions to
`option_reader_continuations/` (phases, lifecycle), to `read_inventory.rs`
and the compile controls; rum-runtime's frame lists.

**Inventory exceptions, authorized by name** (`inventory-plan.md`): I01's
runtime-reader and helper splits; I02's F02, F04, S09-S12 and T01-T05
(**+11** derive entries, including `FrameRootTrace` on the two existing
`PersistContextSelection` / `PersistContextFilters`; **no** new frame — the
named-frame count stays **560**); I03 for the touched phase rows; I05; I06's
Names / TestReport / Save / DumpSave evidence positions; I07; I08. **Any new
`UnknownHostPayload` row is a STOP** — no new public reader is needed here.

**The certified bodies stay verbatim**, and both `local_production_acceptance`
runs must earn the same two C01 and one C02 marks on their merits.

**Standing duty — always in scope** (`TEMPLATE.md`). STOP on any other row
change or disappearance, any new row in untouched code, any self-reaching
state.

**Fence:** step C's rule readers (CheckAbort, ArrayFilter, the rule wrapper)
stay; no `Expr`, matcher or evaluator redesign; no regex implementation
change; no leases, `unsafe`, dependencies, lockfile, catalog reclassification
or request-policy change; the rum-host API unchanged. No edits to `AGENTS.md`,
`TODO.md`, root `PROGRESS.md`, earlier briefs or docs beyond the protocol's
rustdoc.

**Disclosure duty:** every changed function, stage and frozen field, inventory
row, measured label, adjacent defect.

## Deliverable

brief-576 §3's phase rules: each phase looks up **one** entry when it reaches
it, from the explicit snapshot the builtin already compiled and the *current*
defaults (the `Names-later-default-mutation` case requires this); it advances
only after success and validation; Names' invalid-value return happens before
the next read. Runtime `RuleDelayed` values stay held (`runtime_option_expr`'s
inversion is preserved). Monitor starts its scope, display and body only after
all reads; TestReport keeps its two-argument property fallback, its invalid
third-argument residual, its one-argument defaults and its report and
test-case drivers, and starts no test, timer or capture scope before the
reads; DumpSave keeps its pre-read expanded context filters (never
re-expanded) and its post-read order, and Save stays on its no-compression
path. On an actual resume, only the Names pattern is rebuilt — a pure reparse;
no guest expression and no context-filter expansion runs again. Capture only
on `InputRequired`; a second suspension installs the new continuation once;
pure abandonment runs nothing pending.

## Tests

Red first, recorded before any production edit: `Monitor-options`,
`Monitor-options-twice`, `Monitor-EvaluationMonitor`, `Monitor-StepMonitor`
and their twice variants; every `TestReport-option-*`, `TestReport-two-args`
and twice; every `Names-*` single and twice; `DumpSave-option` and twice;
Monitor with two runtime options and its streams, Names with two options, a
later-default mutation and a later option error, TestReport with three
options; the unread TestReport and Names controls — through the shared oracle
(value, error, termination, ordered messages and prints including the existing
repeated Names `optval` messages, `log`, actual suspension, exact prompts, no
owner after, `1 + 1`). Every step-A and earlier-group test stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. Every reader call is a tail call; the source gate enforces it.
3. Read one default at a time, at the phase that needs it; do not prefetch or
   snapshot every default at entry.
4. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-578/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test option_reader_continuations
cargo test -p rum-stdlib --lib options_suspension
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-578/base/rum-cli` and
`rum-cli-alloc` — the director's copy of brief-577's measured one-unit
candidate, whose production source is the incoming tree (hashes in
`sha256.txt`); do not rebuild them. Candidates: the working tree with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), fresh target directory each, hash
checked. Panel: `performance-plan.md`'s step-B labels (`Monitor[77,0]`, the
held explicit monitor option, `Names["Global`never576"]`, the default and
unread TestReport forms, the latent-ordinary-value setups), the step-A labels
(`.temp/brief-577/perf/`), and the controls `dot-three-no-input`,
`differences-no-input`, `take-multispec-small`. Each result checked first.
Allocation counts, then two paired samples of `taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output saved first. **Noise floor:**
brief-577's small single-call labels varied by up to ±1% between samples of
the same binaries (per-process `RandomState` hashing in symbol interning).
First measure the base against itself — two pairs — and take each label's
largest base-to-base difference as its floor; a label regresses when **both**
candidate samples exceed `max(+0.05%, floor)`. Report the floors. No timing
suite. At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-578/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. No probe that aborts a process; DumpSave tests
  use the in-memory provider as brief-576's probe did — no filesystem writes.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files and functions, stages and frozen fields, size assessment.
2. Verbatim red and green output, compile controls, the source gate, ownership
   and certificate results (marks earned), the broad suites; every inventory
   row change.
3. Fourteen sites repaired (31 -> 17 predicted); the noise floors and every
   measured row, both samples.
4. Stop-loss counters, residuals, open questions.

---

## Implementation report — 2026-10-06

The fourteen step-B sites are repaired through the existing typed read-and-complete protocol. Final source: **candidate-04**. **The instruction gate does not pass:** 4 labels exceed their threshold in both samples. Allocation and allocation-byte slopes are unchanged for all 59 labels. No git command or network access was used. Changes are left uncommitted for the director. No timing suite, new unsafe, dependency/lockfile change, catalog reclassification, request-policy change or host API change was made.

### 1. Files, functions, phases and frozen state

Changed files: the 28 crate files below and this appended report. The complete source comparison is [source-diff.txt](../../../.temp/brief-578/source-diff.txt); source positions, sizes and hashes are in [changed-files-final.json](../../../.temp/brief-578/changed-files-final.json). Every changed production function is enumerated in [production-functions.json](../../../.temp/brief-578/production-functions.json).

| File | Bytes | Changed functions / data |
| --- | --- | --- |
| [crates/rum-runtime/src/runtime/debug_tools/monitor/options.rs](../../../crates/rum-runtime/src/runtime/debug_tools/monitor/options.rs) | 4646 | MonitorOptionWorking::syntax; MonitorOptionWorking::freeze_read; OptionReadContinuation&lt;Expr&gt;::finish/freeze; FrozenOptionReadContinuation&lt;Expr&gt;::finish; finish_syntax; read_evaluation_monitor; OptionReadContinuation&lt;Option&lt;Expr&gt;&gt;::finish/freeze; FrozenOptionReadContinuation&lt;Option&lt;Expr&gt;&gt;::finish; monitor_option_expr |
| [crates/rum-runtime/src/runtime/debug_tools/monitor.rs](../../../crates/rum-runtime/src/runtime/debug_tools/monitor.rs) | 5364 | Session::eval_monitor; Session::start_monitor (extracted tail); Session::monitor_option_expr (moved to pure decoder) |
| [crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs](../../../crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs) | 18449 | Session::parse_test_report_options (moved to options.rs); Session::test_report_constraint_expr (moved to pure decoder) |
| [crates/rum-runtime/src/runtime/debug_tools/test_report/options.rs](../../../crates/rum-runtime/src/runtime/debug_tools/test_report/options.rs) | 4295 | read_test_report_options; TestReportOptionWorking::read; OptionReadContinuation&lt;Option&lt;Expr&gt;&gt;::finish/freeze; FrozenOptionReadContinuation&lt;Option&lt;Expr&gt;&gt;::finish; Session::parse_test_report_options; test_report_constraint_expr |
| [crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs](../../../crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs) | 22701 | Session::eval_test_report; Session::start_test_report (extracted tail) |
| [crates/rum-runtime/src/runtime/debug_tools/test_report.rs](../../../crates/rum-runtime/src/runtime/debug_tools/test_report.rs) | 92 | module declaration / derived inventory / standing data only |
| [crates/rum-runtime/src/runtime/namespace_tools/names/options.rs](../../../crates/rum-runtime/src/runtime/namespace_tools/names/options.rs) | 4755 | read_names_options; NamesOptionWorking::read; OptionReadContinuation&lt;Option&lt;Expr&gt;&gt;::finish/freeze; FrozenOptionReadContinuation&lt;Option&lt;Expr&gt;&gt;::finish; names_ignore_case |
| [crates/rum-runtime/src/runtime/namespace_tools/names.rs](../../../crates/rum-runtime/src/runtime/namespace_tools/names.rs) | 13376 | Session::eval_names; Session::compile_names_invocation; Session::names_ignore_case (moved to pure decoder); NamesInvocation pattern_index field |
| [crates/rum-runtime/src/runtime/options/core.rs](../../../crates/rum-runtime/src/runtime/options/core.rs) | 8117 | Session::runtime_option_expr_for_symbol |
| [crates/rum-runtime/src/runtime/options/read_phase.rs](../../../crates/rum-runtime/src/runtime/options/read_phase.rs) | 4171 | OptionDecode visibility; RuntimeOptionDecode::decode; OptionSyntaxDecode::decode; Session::eval_option_syntax; suspend_option_read |
| [crates/rum-runtime/src/runtime/options/tests/read_phase.rs](../../../crates/rum-runtime/src/runtime/options/tests/read_phase.rs) | 9726 | read_phase_compile_controls_pin_missing_private_and_unrooted_errors; finish; main; freeze; runtime_and_syntax_reads_complete_without_ordinary_capture |
| [crates/rum-runtime/src/runtime/package_tools/persist_options.rs](../../../crates/rum-runtime/src/runtime/package_tools/persist_options.rs) | 5165 | OptionReadContinuation&lt;Option&lt;Expr&gt;&gt;::finish/freeze; FrozenOptionReadContinuation&lt;Option&lt;Expr&gt;&gt;::finish; PersistOptionWorking::persist |
| [crates/rum-runtime/src/runtime/package_tools.rs](../../../crates/rum-runtime/src/runtime/package_tools.rs) | 25787 | Session::eval_persist_package_state; PersistContextSelection/PersistContextFilters derive roots |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs](../../../crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs) | 173398 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs](../../../crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs) | 17249 | expected; discover; read_inventory_is_closed_and_migrated_calls_are_tails; phase_read_inventory_falsifies_non_tail_and_missing_continuation; read_inventory_is_closed_and_boolean_calls_are_tails (removed/moved) |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv) | 23520 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | 251872 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | 421204 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | 399156 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv) | 20599 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/src/tests/retention_closure/census/tests.rs](../../../crates/rum-stdlib/src/tests/retention_closure/census/tests.rs) | 7939 | baseline |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | 91646 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv) | 111960 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | 18161 | module declaration / derived inventory / standing data only |
| [crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs](../../../crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs) | 5915 | phased_read_cancellation_and_abandonment_run_no_pending_work |
| [crates/rum-stdlib/tests/option_reader_continuations/phases.rs](../../../crates/rum-stdlib/tests/option_reader_continuations/phases.rs) | 17667 | oracle_monitor_options; oracle_names_spellingcorrection; oracle_names_resolvecontextaliases; oracle_names_ignorecase; oracle_monitor_evaluationmonitor; oracle_monitor_stepmonitor; oracle_testreport_option_sametest; oracle_testreport_option_timeconstraint; oracle_testreport_option_memoryconstraint; oracle_dumpsave_option; oracle_testreport_two_args; oracle_monitor_options_twice; oracle_monitor_options_twice576; oracle_names_spellingcorrection_twice576; oracle_names_resolvecontextaliases_twice576; oracle_names_ignorecase_twice576; oracle_monitor_evaluationmonitor_twice576; oracle_monitor_stepmonitor_twice576; oracle_testreport_option_sametest_twice576; oracle_testreport_option_timeconstraint_twice576; oracle_testreport_option_memoryconstraint_twice576; oracle_dumpsave_option_twice576; oracle_testreport_two_args_twice576; oracle_monitor_runtime_two_options; oracle_names_two_options; oracle_testreport_three_options; oracle_names_later_default_mutation; oracle_names_later_option_error_after_resume; oracle_monitor_option_streams; oracle_testreport_unread_default; oracle_names_early_invalid_unread_later; monitor_runtime_two_options_preserve_ordered_streams; testreport_property_and_invalid_option_forms_leave_defaults_unread; names_invalid_alias_leaves_ignorecase_unread; names_pattern_list_is_reparsed_without_repeating_prior_work; runtime_delayed_values_stay_held; read_to_string; probe; write_string; dumpsave_writes_once_after_read_and_context_change; save_leaves_compression_unread |
| [crates/rum-stdlib/tests/option_reader_continuations/support.rs](../../../crates/rum-stdlib/tests/option_reader_continuations/support.rs) | 5105 | session; read_to_string; probe; write_string |
| [crates/rum-stdlib/tests/option_reader_continuations.rs](../../../crates/rum-stdlib/tests/option_reader_continuations.rs) | 263 | module declaration / derived inventory / standing data only |

Monitor reads syntax, EvaluationMonitor, then StepMonitor before starting its original scope/display/body driver. TestReport classifies the two/three-argument forms before SameTest, TimeConstraint and MemoryConstraint, and retains its one-argument Rust defaults, property fallback, malformed third-argument residual and existing report/test-case drivers. Names validates SpellingCorrection before reading ResolveContextAliases, validates that before IgnoreCase, and enumerates afterward. Each phase consults only its next entry: the compiled explicit snapshot wins; otherwise current Session defaults are read at that phase. Runtime RuleDelayed values remain held.

DumpSave freezes the raw path, explicit snapshot and already prepared context filters, then resumes compression validation and the original path/provider/symbol/filter/attribute/sealed-reference/write order. Save uses the same persistence tail without reading compression. On resume, only Names pattern data is reparsed through the pure existing compiler; no guest expression, option normalization or context expansion repeats. Frozen state is created only for InputRequired; ordinary errors bypass it, a later suspension captures its new continuation once, and pure abandonment runs no pending work.

| Frozen type | Fields | Stages |
| --- | --- | --- |
| MonitorOptionRead | args: SharedArgs; explicit: Option&lt;Arc&lt;SymbolOptions&gt;&gt;; stage: MonitorOptionStage; evaluation_monitor: Option&lt;Expr&gt; | Syntax, Evaluation, Step |
| TestReportOptionRead | head: Expr; args: SharedArgs; properties: Option&lt;Expr&gt;; explicit: Arc&lt;SymbolOptions&gt;; stage: TestReportOptionStage; options: TestReportOptions (completed same_test/time_constraint/memory_constraint prefix) | SameTest, Time, Memory |
| NamesOptionRead | args: SharedArgs; pattern_index: Option&lt;usize&gt;; explicit_context: bool; explicit: Option&lt;Arc&lt;SymbolOptions&gt;&gt;; stage: NamesOptionStage | Spelling, Aliases, IgnoreCase |
| PersistOptionRead | args: SharedArgs; operator: &'static str; overwrite: bool; path_spec: String; explicit: Option&lt;Arc&lt;SymbolOptions&gt;&gt;; context_filters: PersistContextFilters (included/excluded selections, including expanded BTreeSet&lt;String&gt;) | — |
| RuntimeOptionDecode | zero-size decoder | — |
| OptionSyntaxDecode | zero-size decoder | — |

The generic OptionReadSuspensionFrame remains the sole frame for these reads; **named frames remain 560**. Exactly **11 FrameRootTrace entries** were added (1011 → 1022), including the two existing Persist filter types; no new abandonment/finalizer type or self-reaching state was added. Exact additions: [derive-additions.json](../../../.temp/brief-578/derive-additions.json).

Final TestReport working state: the completed prefix is borrowed through reads and moved into the report once; the working projection property is owned. The frozen prefix and property remain owned.

Size assessment: each new production child is below 10k bytes. package_tools.rs remains in the 20–35k split-plan range; its next coherent boundary is saved-definition rendering/loading versus persistence orchestration. TestReport report.rs remains in that range; future separation is report/result accumulation versus test-case execution/resume. Names and TestReport evaluation remain 10–20k warning-range, with pattern/enumeration and capture/evaluation boundaries respectively. These broader splits are outside this entry-continuation brief. The source inventory and phase fixtures are cohesive test warning-range files. The large derived-frame list remains deliberately declarative test data. No root runtime child was added.

Tests changed: the integration root adds phases; support::session installs the memory-only rejecting provider, with its FileIo methods. phases.rs adds 38 tests plus a recording in-memory provider; lifecycle.rs adds the 16-combination phased cancellation/abandonment test. The source inventory adds the six phase call sites, removes step-B legacy allowances, checks required private reader signatures and falsifies non-tail uses. Runtime protocol tests cover `Option<Expr>`/`Expr` ordinary completion/error without freezing, plus rooted/unrooted and inaccessible-reader compile controls. The census baseline changes only the identical C02 method-syntax locator 209 → 224.

### 2. Red, green, source/compile controls and standing duty

Red was recorded **before production edits**, against the shared rum queued-versus-resumed oracle. It compares final value/error/termination, ordered messages and prints, log, actual suspension, exact prompts, owner cleanup and subsequent 1+1. Existing repeated Names optval messages remain part of equality.

```text
test result: FAILED. 54 passed; 29 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```
All 29 failures are step-B phase cases; 52 earlier cases and two unread controls passed. Complete verbatim red rows: [red-oracle.tsv](../../../.temp/brief-578/red-oracle.tsv) and [red.log](../../../.temp/brief-578/red.log). Final oracle rows are in [final-oracle.tsv](../../../.temp/brief-578/final-oracle.tsv).

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-578/tmp cargo fmt --check` — exit 0. [Full output](../../../.temp/brief-578/fmt-final-selected.log)

```text
(no output)
```
`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-578/tmp cargo test -p rum-stdlib --test option_reader_continuations -- --nocapture` — exit 0. [Full output](../../../.temp/brief-578/phases-final-selected.log)

```text
test result: ok. 91 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
```
`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-578/tmp cargo test -p rum-stdlib --lib options_suspension -- --nocapture` — exit 0. [Full output](../../../.temp/brief-578/options-source-final-selected.log)

```text
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 1078 filtered out; finished in 4.70s
```
`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-578/tmp cargo test -p rum-runtime -- --nocapture` — exit 0. [Full output](../../../.temp/brief-578/runtime-final-selected.log)

```text
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... named_suspension_frames=560
test result: ok. 1123 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 30.70s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.73s
```
`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-578/tmp cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` — exit 0. [Full output](../../../.temp/brief-578/ownership-final-selected.log)

```text
EXACT INVENTORY UnknownHostPayload: 64 rows
EXACT INVENTORY UnknownConstruct: 7664 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1093 filtered out; finished in 61.97s
```
`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-578/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` — exit 0. [Full output](../../../.temp/brief-578/certificates-true-final-selected.log)

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1112 filtered out; finished in 18.27s
```
`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-578/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` — exit 0. [Full output](../../../.temp/brief-578/certificates-false-final-selected.log)

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1106 filtered out; finished in 18.74s
```
`python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-578/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture` — exit 0. [Full output](../../../.temp/brief-578/workspace-final.log)

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.90s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.03s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... named_suspension_frames=560
test result: ok. 1123 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.42s
EXACT INVENTORY UnknownHostPayload: 64 rows
EXACT INVENTORY UnknownConstruct: 7664 rows
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1060 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1305.29s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.75s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 91 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.87s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.90s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
Native ZeroMQ: **9 explicitly skipped cases** because loopback socket binding is unavailable. The raw Cargo totals include their early returns; these cases did not exercise native loopback transport. The exact skip receipts are included above.

Compile controls (verbatim status lines; failures must be exactly E0061/E0624/E0277 as appropriate, with no unrelated compiler error):

```text
test runtime::options::read_phase_tests::read_phase_compile_controls_pin_missing_private_and_unrooted_errors ... missing-pass: success=true
missing-fail: success=false
private-pass: success=true
private-fail: success=false
root-pass: success=true
runtime-root-pass: success=true
syntax-root-pass: success=true
runtime-root-fail: success=false
syntax-root-fail: success=false
runtime-private-fail: success=false
syntax-private-fail: success=false
root-fail: success=false
```
The source gate requires every migrated call to be a tail. Boolean and earlier-group fixtures remain enabled; only CheckAbort/ArrayFilter rule readers retain the named step-C legacy allowance.

C01/C02 bodies stayed byte-for-byte identical: [certified-body-sha256.json](../../../.temp/brief-578/certified-body-sha256.json). Each feature configuration earned two C01 instance marks, one C02 mark and four separate probe marks, with zero rejections, before emitted certificate cells were copied. The first stale-cell failures are preserved in the replay logs; fresh fingerprints alone were never treated as proof. All 68 local contract fingerprint cells were replayed mechanically. Status, precondition, exclusion, domain and catalog-class fields are unchanged.

```text
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```
```text
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```
Every inventory row before/after is disclosed in [inventory-changes.txt](../../../.temp/brief-578/inventory-changes.txt), [ownership-row-changes.json](../../../.temp/brief-578/ownership-row-changes.json), [obligation-row-changes.json](../../../.temp/brief-578/obligation-row-changes.json), [fingerprint-row-changes.json](../../../.temp/brief-578/fingerprint-row-changes.json) and the [default](../../../.temp/brief-578/certificate-cell-changes-true.json) / [no-default](../../../.temp/brief-578/certificate-cell-changes-false.json) certificate ledgers. Unknowns remain unknown.

| Inventory | Before | After | Position changes | Authorized removed/replaced | Added |
| --- | --- | --- | --- | --- | --- |
| host_payloads.tsv | 64 | 64 | 0 | 0 | 0 |
| unknown_attributes.tsv | 1475 | 1484 | 9 | 2 | 11 |
| unknown_captures.tsv | 2944 | 2951 | 62 | 5 | 12 |
| unknown_macros.tsv | 3138 | 3141 | 36 | 8 | 11 |
| unknown_storage.tsv | 88 | 88 | 2 | 0 | 0 |
| self_reaching.tsv | 59 | 59 | 1 | 0 | 0 |
| host_payload_ledger.tsv | — | — | 0 | 0 | 0 |

The 42 obligation rows move evidence positions only: `N:AlignColumns`, `Unsettled:Clear`, `Unsettled:ClearAll`, `N:Complement`, `N:ContainsAll`, `N:ContainsAny`, `N:ContainsExactly`, `N:ContainsNone`, `N:ContainsOnly`, `N:DisjointQ`, `Unsplit:DumpSave`, `N:DuplicateFreeQ`, `N:HtmlUnescape`, `N:IntersectingQ`, `N:Intersection`, `N:Monitor`, `Unsplit:Names`, `Unsplit:Once`, `N:ParseArgs`, `Unsettled:Protect`, `Unsettled:Remove`, `Unsplit:Save`, `N:StringContainsQ`, `N:StringCount`, `N:StringDelete`, `N:StringEndsQ`, `N:StringFreeQ`, `N:StringIndent`, `N:StringMatchQ`, `N:StringPosition`, `N:StringSplit`, `N:StringStartsQ`, `N:StringWrap`, `N:SubsetQ`, `N:TemplateApply`, `Unsplit:TestReport`, `N:Uncompress`, `N:Union`, `Unsettled:Unprotect`, `N:UrlDecode`, `N:UrlEncode`, `N:UrlParse`. I06 covers the named Names/TestReport/Save/DumpSave anchors; the N:Monitor evidence follows the authorized I01 phase split under the standing position-maintenance duty. Its identity, N/Pending classification, owner and rationale remain unchanged. All other affected evidence is relocated by identical source tokens.

Intermediate failures are preserved: one compile-only missing Monitor derive import was corrected; the extra persistence witness first used an unqualified, then a System-qualified IncludedContexts option, both rejected before compression. The supported witness now tests context mutation and exactly-once in-memory writing. No pre-fix value was made an expectation and no documented contract was weakened. See the corresponding phases-green-final*.log and [PROGRESS.md](../../../.temp/brief-578/PROGRESS.md).

### 3. Fourteen repaired sites and every measured row

| Sites | Repair |
| --- | --- |
| P0118, P0119, P0120, P0122 | Monitor syntax and both runtime reads finish their exact phase and original body tail. |
| P0157, P0158, P0159, P0161, P0167, P0168 | TestReport one-entry phase driver, pure decoders, and both valid option-entry branches. |
| P0962, P0963, P0964 | Names phase validation, late defaults and pure cold pattern rebuild. |
| P0997 | DumpSave compression continuation with pre-read filters retained. |

The brief's predicted lost-continuation count is **31 → 17**. This is the named fourteen-site migration, not a new whole-workspace proof or a claim that the seventeen residual sites are fixed.

Base binaries are the director-supplied copies; their hashes were verified and they were never rebuilt. Each candidate has fresh default and alloc-count target directories, one codegen unit, checked source and binary hashes. Each workload is result-checked first. Allocation counts precede two paired instruction samples; one label/process, stdin /dev/null, CPU 2, event cpu_core/instructions/u, profiles off except the allocation arm, slope `(count(2N)-count(N))/N`. No simultaneous test/build workload ran during a measurement panel. [Exact statements/setups/N/results](../../../.temp/brief-578/perf/labels.json); [base hashes](../../../.temp/brief-578/base-hash-check.tsv).

A regression requires both candidate samples to exceed `max(+0.05%, floor)`, where floor is the largest absolute difference in that label's two base/base pairs. Floors were fixed before measuring candidates. Allocation and byte slopes are unchanged on every measured candidate. All native timing numbers are command bounds/durations only; no wall-clock timing gate was used.

Base/base calibration (instruction slopes per iteration; signed pair deltas and absolute per-label floor):

| Label | N | Base A1 | Base B1 | Δ1 % | Base A2 | Base B2 | Δ2 % | Floor % |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 20000 | 15929.6701 | 15929.57375 | -0.000605 | 15890.1447 | 15977.6384 | 0.550616 | 0.550616 |
| g5-cases-small-delayed | 20000 | 14820.38545 | 14790.3058 | -0.202961 | 14820.2792 | 14790.0614 | -0.203895 | 0.203895 |
| g5-cases-small-explicit | 20000 | 16942.04545 | 16853.60955 | -0.521991 | 17071.34505 | 16897.89445 | -1.016034 | 1.016034 |
| g5-count-small-default | 20000 | 13007.5364 | 13044.6465 | 0.285297 | 13034.57115 | 13161.519 | 0.973932 | 0.973932 |
| g5-count-small-delayed | 20000 | 12025.92175 | 12037.00085 | 0.092127 | 11988.9052 | 11988.8678 | -0.000312 | 0.092127 |
| g5-count-small-explicit | 20000 | 13993.45125 | 13967.24945 | -0.187243 | 13954.2378 | 14093.40845 | 0.997336 | 0.997336 |
| g5-memberq-small-default | 20000 | 11450.29945 | 11450.30455 | 0.000045 | 11498.29035 | 11426.4 | -0.625226 | 0.625226 |
| g5-memberq-small-delayed | 20000 | 10307.13315 | 10281.10755 | -0.252501 | 10332.0745 | 10307.34185 | -0.239377 | 0.252501 |
| g5-memberq-small-explicit | 20000 | 12037.1105 | 12024.19265 | -0.107317 | 12037.0463 | 11982.2418 | -0.455299 | 0.455299 |
| g5-freeq-small-default | 20000 | 12417.69995 | 12465.63905 | 0.386055 | 12465.8228 | 12435.69075 | -0.241717 | 0.386055 |
| g5-freeq-small-delayed | 20000 | 11198.7232 | 11207.51315 | 0.078491 | 11196.7211 | 11222.86735 | 0.233517 | 0.233517 |
| g5-freeq-small-explicit | 20000 | 13120.34645 | 13042.45575 | -0.593663 | 13042.3834 | 13038.55505 | -0.029353 | 0.593663 |
| g5-position-small-default | 20000 | 19327.7033 | 19329.1088 | 0.007272 | 19396.95095 | 19319.15105 | -0.401093 | 0.401093 |
| g5-position-small-delayed | 20000 | 18136.9162 | 18138.23255 | 0.007258 | 18124.16735 | 18186.14785 | 0.341977 | 0.341977 |
| g5-position-small-explicit | 20000 | 19909.6194 | 19894.4197 | -0.076343 | 19909.5428 | 19939.985 | 0.152903 | 0.152903 |
| g5-deletecases-small-default | 20000 | 14731.10615 | 14477.43695 | -1.721997 | 14461.7626 | 14514.4874 | 0.364581 | 1.721997 |
| g5-deletecases-small-delayed | 20000 | 13366.33155 | 13394.5542 | 0.211147 | 13349.09645 | 13366.77625 | 0.132442 | 0.211147 |
| g5-deletecases-small-explicit | 20000 | 15263.70275 | 15241.2226 | -0.147278 | 15242.8978 | 15248.49985 | 0.036752 | 0.147278 |
| g5-replace-small-default | 20000 | 20271.0466 | 20214.3509 | -0.279688 | 20190.49715 | 20262.2522 | 0.355390 | 0.355390 |
| g5-replace-small-delayed | 20000 | 19050.3975 | 19065.33675 | 0.078420 | 19065.49535 | 19114.6512 | 0.257826 | 0.257826 |
| g5-replace-small-explicit | 20000 | 21268.54095 | 21268.016 | -0.002468 | 21214.6208 | 21347.9799 | 0.628619 | 0.628619 |
| g5-level-small-default | 20000 | 13514.46355 | 13310.5177 | -1.509093 | 13360.7215 | 13419.0156 | 0.436310 | 1.509093 |
| g5-level-small-delayed | 20000 | 12189.18605 | 12307.76205 | 0.972797 | 12235.4099 | 12150.83705 | -0.691214 | 0.972797 |
| g5-level-small-explicit | 20000 | 14316.8521 | 14249.86525 | -0.467888 | 14260.3596 | 14212.88445 | -0.332917 | 0.467888 |
| g5-depth-small-default | 20000 | 8936.47775 | 8864.31355 | -0.807524 | 8804.35975 | 8837.3524 | 0.374731 | 0.807524 |
| g5-depth-small-delayed | 20000 | 7650.69945 | 7617.66385 | -0.431798 | 7619.34705 | 7643.6146 | 0.318499 | 0.431798 |
| g5-depth-small-explicit | 20000 | 9732.6726 | 9665.67115 | -0.688418 | 9665.8566 | 9777.5986 | 1.156049 | 1.156049 |
| g5-trace-small-default | 20000 | 27527.5107 | 27526.7869 | -0.002629 | 27526.1867 | 27526.42835 | 0.000878 | 0.002629 |
| g5-trace-small-delayed | 20000 | 26311.04235 | 26383.0934 | 0.273843 | 26276.48525 | 26359.51045 | 0.315968 | 0.315968 |
| g5-trace-small-explicit | 20000 | 31801.11315 | 31728.7336 | -0.227601 | 31800.71705 | 31708.44765 | -0.290149 | 0.290149 |
| g5-tracedetail-small-default | 20000 | 36076.3755 | 36076.01835 | -0.000990 | 36111.278 | 36065.11525 | -0.127835 | 0.127835 |
| g5-tracedetail-small-delayed | 20000 | 34888.23875 | 34888.60375 | 0.001046 | 34936.0007 | 34848.3379 | -0.250924 | 0.250924 |
| g5-tracedetail-small-explicit | 20000 | 39018.27205 | 39007.301 | -0.028118 | 39174.2623 | 38998.833 | -0.447818 | 0.447818 |
| g5-traceprint-small-default | 2000 | 30627.348 | 30685.5105 | 0.189904 | 30628.21 | 30654.98 | 0.087403 | 0.189904 |
| g5-traceprint-small-delayed | 2000 | 29615.662 | 29407.009 | -0.704536 | 29420.817 | 29460.8355 | 0.136021 | 0.704536 |
| g5-traceprint-small-explicit | 2000 | 34724.9685 | 35187.6565 | 1.332436 | 34841.999 | 34820.014 | -0.063099 | 1.332436 |
| matchq-integer-success | 20000 | 4340.24055 | 4340.20025 | -0.000929 | 4340.2123 | 4340.1421 | -0.001617 | 0.001617 |
| matchq-integer-fail | 20000 | 4462.3124 | 4462.1469 | -0.003709 | 4462.2446 | 4462.2217 | -0.000513 | 0.003709 |
| cases-wide-no-match | 1000 | 2400532.078 | 2400504.926 | -0.001131 | 2400534.653 | 2400690.322 | 0.006485 | 0.006485 |
| count-wide-no-match | 1000 | 2178864.688 | 2178888.573 | 0.001096 | 2178905.255 | 2178953.33 | 0.002206 | 0.002206 |
| position-wide-no-match | 1000 | 2573250.313 | 2573273.149 | 0.000887 | 2573273.624 | 2573369.328 | 0.003719 | 0.003719 |
| deletecases-wide-no-match | 1000 | 2670166.358 | 2670165.979 | -0.000014 | 2670166.131 | 2670202.854 | 0.001375 | 0.001375 |
| replace-wide-no-match | 1000 | 1163723.934 | 1163564.293 | -0.013718 | 1163594.025 | 1163607.257 | 0.001137 | 0.013718 |
| dot-three-no-input | 20000 | 253104.9799 | 253101.3568 | -0.001431 | 253101.43755 | 253101.4351 | -0.000001 | 0.001431 |
| differences-no-input | 20000 | 9418.0328 | 9410.80465 | -0.076748 | 9411.0912 | 9415.80995 | 0.050140 | 0.076748 |
| take-multispec-small | 20000 | 49232.52675 | 49248.5602 | 0.032567 | 49241.20645 | 49240.0932 | -0.002261 | 0.032567 |
| g5-monitor-options-none | 2000 | 18194.4145 | 18254.0495 | 0.327765 | 18096.52 | 18209.514 | 0.624396 | 0.624396 |
| g5-monitor-explicit-held | 2000 | 17689.891 | 17710.118 | 0.114342 | 17748.4685 | 17786.6545 | 0.215151 | 0.215151 |
| g5-names-default | 200 | 3482631.415 | 3482612.635 | -0.000539 | 3482319.785 | 3483176.21 | 0.024594 | 0.024594 |
| g5-testreport-options-default | 2000 | 42901.481 | 42877.867 | -0.055042 | 42831.832 | 42844.358 | 0.029245 | 0.055042 |
| g5-testreport-unread-default | 2000 | 28791.862 | 28789.6935 | -0.007532 | 28788.77 | 28790.19 | 0.004932 | 0.007532 |
| g5-monitor-evaluationmonitor-latent | 2000 | 13351.078 | 13351.514 | 0.003266 | 13303.4375 | 13352.0685 | 0.365552 | 0.365552 |
| g5-monitor-stepmonitor-latent | 2000 | 13353.5945 | 13355.1125 | 0.011368 | 13352.6305 | 13352.896 | 0.001988 | 0.011368 |
| g5-names-spellingcorrection-latent | 200 | 3473361.97 | 3473102.985 | -0.007456 | 3472853.13 | 3473257.56 | 0.011645 | 0.011645 |
| g5-names-resolvecontextaliases-latent | 200 | 3473318.235 | 3473251.01 | -0.001935 | 3472986.385 | 3473351.825 | 0.010522 | 0.010522 |
| g5-names-ignorecase-latent | 200 | 3470662.35 | 3470881.8 | 0.006323 | 3470625.205 | 3470564.705 | -0.001743 | 0.006323 |
| g5-testreport-sametest-latent | 2000 | 27628.578 | 27580.032 | -0.175709 | 27616.4515 | 27606.5635 | -0.035805 | 0.175709 |
| g5-testreport-timeconstraint-latent | 2000 | 27622.82 | 27595.4245 | -0.099177 | 27623.4935 | 27582.9615 | -0.146730 | 0.146730 |
| g5-testreport-memoryconstraint-latent | 2000 | 27601.149 | 27649.827 | 0.176362 | 27600.963 | 27588.3975 | -0.045526 | 0.176362 |

| Candidate | Single change | Regressing labels | Worst persistent % | Allocation regressions | Consecutive no-improvement |
| --- | --- | --- | --- | --- | --- |
| candidate-01 | initial phased implementation | 8 | 1.886978 | 0 | 0 |
| candidate-02 | inline TestReport successful finish | 8 | 2.849298 | 0 | 1 |
| candidate-03 | inline extracted TestReport tail | 7 | 1.657371 | 0 | 0 |
| candidate-04 | borrow completed TestReport prefix during ordinary reads | 4 | 2.047003 | 0 | 0 |
| candidate-05 | direct one-argument TestReport default tail | 8 | 1.076027 | 0 | 0 |
| candidate-06 | borrow TestReport projection during reads | 6 | 1.047943 | 0 | 0 |
| candidate-07 | inline Monitor runtime-option finish | 5 | 1.228001 | 0 | 1 |
| candidate-08 | inline TestReport read dispatcher | 7 | 1.263498 | 0 | 2 |

#### candidate-01

[Hashes/build identity](../../../.temp/brief-578/candidate-01/hashes.json); [result checks](../../../.temp/brief-578/candidate-01/perf/checks.json); [raw counts/streams/status](../../../.temp/brief-578/candidate-01/perf/paired-results.json); [full precision TSV](../../../.temp/brief-578/candidate-01/perf/slopes.tsv).

| Label | Alloc base/new | Bytes base/new | I base1 | I new1 | Δ1 % | I base2 | I new2 | Δ2 % | Regresses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 9.0/9.0 | 912.0/912.0 | 15905.22975 | 16035.4071 | 0.818456 | 15898.6484 | 15957.49075 | 0.370109 | False |
| g5-cases-small-delayed | 8.0/8.0 | 656.0/656.0 | 14840.57475 | 14807.6148 | -0.222093 | 14792.7203 | 14782.3234 | -0.070284 | False |
| g5-cases-small-explicit | 11.0/11.0 | 1104.0/1104.0 | 16901.1927 | 16917.581 | 0.096965 | 17070.673 | 16907.1752 | -0.957770 | False |
| g5-count-small-default | 6.0/6.0 | 664.0/664.0 | 13135.65835 | 13135.65545 | -0.000022 | 13118.53835 | 13129.4805 | 0.083410 | False |
| g5-count-small-delayed | 5.0/5.0 | 408.0/408.0 | 11988.80865 | 12054.9195 | 0.551438 | 12014.88885 | 12006.81845 | -0.067170 | False |
| g5-count-small-explicit | 8.0/8.0 | 824.0/824.0 | 13993.55575 | 13919.41265 | -0.529837 | 13993.64985 | 13967.4384 | -0.187310 | False |
| g5-memberq-small-default | 6.0/6.0 | 760.0/760.0 | 11450.19255 | 11434.27075 | -0.139053 | 11450.4521 | 11404.1905 | -0.404015 | False |
| g5-memberq-small-delayed | 5.0/5.0 | 504.0/504.0 | 10319.96265 | 10324.18145 | 0.040880 | 10307.2984 | 10286.06305 | -0.206022 | False |
| g5-memberq-small-explicit | 7.0/7.0 | 792.0/792.0 | 12024.13785 | 12115.0217 | 0.755845 | 12012.9122 | 11978.05885 | -0.290132 | False |
| g5-freeq-small-default | 7.0/7.0 | 1208.0/1208.0 | 12417.73075 | 12509.797 | 0.741410 | 12465.6835 | 12525.6756 | 0.481258 | True |
| g5-freeq-small-delayed | 6.0/6.0 | 952.0/952.0 | 11209.7646 | 11234.60215 | 0.221571 | 11370.79765 | 11249.66555 | -1.065291 | False |
| g5-freeq-small-explicit | 8.0/8.0 | 1240.0/1240.0 | 13042.2822 | 13016.22265 | -0.199808 | 13108.54105 | 13087.37035 | -0.161503 | False |
| g5-position-small-default | 17.0/17.0 | 2136.0/2136.0 | 19392.05485 | 19457.23935 | 0.336140 | 19345.14055 | 19315.18455 | -0.154850 | False |
| g5-position-small-delayed | 16.0/16.0 | 1880.0/1880.0 | 18138.2085 | 18120.51895 | -0.097526 | 18144.08825 | 18059.37255 | -0.466905 | False |
| g5-position-small-explicit | 18.0/18.0 | 2168.0/2168.0 | 19957.6558 | 19850.92905 | -0.534766 | 20005.6748 | 19946.52875 | -0.295646 | False |
| g5-deletecases-small-default | 9.0/9.0 | 984.0/984.0 | 14448.8118 | 14594.43925 | 1.007885 | 14459.49935 | 14498.4976 | 0.269707 | False |
| g5-deletecases-small-delayed | 8.0/8.0 | 728.0/728.0 | 13351.24545 | 13397.04355 | 0.343025 | 13351.71525 | 13326.13505 | -0.191587 | False |
| g5-deletecases-small-explicit | 10.0/10.0 | 1048.0/1048.0 | 15301.393 | 15229.17825 | -0.471949 | 15321.8481 | 15180.94065 | -0.919650 | False |
| g5-replace-small-default | 20.0/20.0 | 1945.0/1945.0 | 20262.29965 | 20274.2826 | 0.059139 | 20274.23105 | 20284.44675 | 0.050388 | False |
| g5-replace-small-delayed | 19.0/19.0 | 1689.0/1689.0 | 19065.30315 | 19100.9484 | 0.186964 | 19088.0149 | 19066.8944 | -0.110648 | False |
| g5-replace-small-explicit | 22.0/22.0 | 2233.0/2233.0 | 21251.9489 | 21237.8495 | -0.066344 | 21266.8942 | 21267.6935 | 0.003758 | False |
| g5-level-small-default | 13.0/13.0 | 1009.0/1009.0 | 13438.37275 | 13350.71135 | -0.652322 | 13361.4524 | 13348.55085 | -0.096558 | False |
| g5-level-small-delayed | 12.0/12.0 | 753.0/753.0 | 12189.2547 | 12151.2615 | -0.311694 | 12170.8526 | 12168.12225 | -0.022434 | False |
| g5-level-small-explicit | 13.0/13.0 | 1009.0/1009.0 | 14293.05515 | 14247.32945 | -0.319916 | 14260.2751 | 14227.8441 | -0.227422 | False |
| g5-depth-small-default | 4.0/4.0 | 392.0/392.0 | 8791.25785 | 8804.39345 | 0.149417 | 8774.28625 | 8804.2023 | 0.340951 | False |
| g5-depth-small-delayed | 3.0/3.0 | 136.0/136.0 | 7630.6684 | 7680.47305 | 0.652691 | 7626.7659 | 7572.53695 | -0.711035 | False |
| g5-depth-small-explicit | 4.0/4.0 | 392.0/392.0 | 9793.62825 | 9682.75615 | -1.132084 | 9529.6682 | 9647.8006 | 1.239628 | False |
| g5-trace-small-default | 33.0/33.0 | 1992.0/1992.0 | 27950.12585 | 27535.0489 | -1.485063 | 27689.1583 | 27666.8178 | -0.080683 | False |
| g5-trace-small-delayed | 32.0/32.0 | 1736.0/1736.0 | 26287.5303 | 26290.2085 | 0.010188 | 26287.4971 | 26307.91435 | 0.077669 | False |
| g5-trace-small-explicit | 36.0/36.0 | 2120.0/2120.0 | 31741.27485 | 31720.003 | -0.067016 | 31739.11355 | 31840.75325 | 0.320235 | False |
| g5-tracedetail-small-default | 55.0/55.0 | 3263.0/3263.0 | 36189.74525 | 36171.9025 | -0.049303 | 35978.02835 | 36005.03605 | 0.075067 | False |
| g5-tracedetail-small-delayed | 54.0/54.0 | 3007.0/3007.0 | 34948.7286 | 34914.85995 | -0.096910 | 34948.32305 | 34805.1019 | -0.409808 | False |
| g5-tracedetail-small-explicit | 57.0/57.0 | 3359.0/3359.0 | 38899.9668 | 39122.67675 | 0.572520 | 39065.4609 | 39011.0422 | -0.139301 | False |
| g5-traceprint-small-default | 40.0/40.0 | 2174.0/2174.0 | 30614.521 | 30762.9005 | 0.484670 | 30663.579 | 30641.023 | -0.073560 | False |
| g5-traceprint-small-delayed | 39.0/39.0 | 1918.0/1918.0 | 29417.6605 | 29397.712 | -0.067811 | 29538.0765 | 29420.072 | -0.399500 | False |
| g5-traceprint-small-explicit | 43.0/43.0 | 2302.0/2302.0 | 34821.2445 | 34796.7015 | -0.070483 | 34867.8 | 34820.399 | -0.135945 | False |
| matchq-integer-success | 1.0/1.0 | 128.0/128.0 | 4340.385 | 4340.45505 | 0.001614 | 4340.08375 | 4340.24985 | 0.003827 | False |
| matchq-integer-fail | 2.0/2.0 | 129.0/129.0 | 4462.2563 | 4462.2856 | 0.000657 | 4462.18155 | 4462.1347 | -0.001050 | False |
| cases-wide-no-match | 33.001/33.001 | 263247.864/263247.864 | 2400494.455 | 2400701.803 | 0.008638 | 2400534.271 | 2400529.802 | -0.000186 | False |
| count-wide-no-match | 30.0/30.0 | 262601.0/262601.0 | 2178842.219 | 2178853.749 | 0.000529 | 2178888.246 | 2178967.655 | 0.003644 | False |
| position-wide-no-match | 1642.001/1642.001 | 576255.864/576255.864 | 2573188.536 | 2573271.236 | 0.003214 | 2573224.736 | 2573272.416 | 0.001853 | False |
| deletecases-wide-no-match | 24.001/24.001 | 247447.864/247447.864 | 2670276.02 | 2670173.329 | -0.003846 | 2670215.579 | 2670139.781 | -0.002839 | False |
| replace-wide-no-match | 30.001/30.001 | 184455.864/184455.864 | 1163578.976 | 1163564.375 | -0.001255 | 1163767.643 | 1163589.64 | -0.015295 | False |
| dot-three-no-input | 596.0/596.0 | 23472.0/23472.0 | 253105.50685 | 253105.77365 | 0.000105 | 253101.9215 | 253105.8238 | 0.001542 | False |
| differences-no-input | 13.0/13.0 | 672.0/672.0 | 9417.9031 | 9399.3837 | -0.196640 | 9417.52345 | 9399.9789 | -0.186297 | False |
| take-multispec-small | 109.0/109.0 | 5824.0/5824.0 | 49225.32885 | 49231.4123 | 0.012358 | 49240.6589 | 49230.76685 | -0.020089 | False |
| g5-monitor-options-none | 9.0/9.0 | 953.0/953.0 | 18096.8095 | 18241.333 | 0.798613 | 18271.7075 | 18229.1435 | -0.232950 | False |
| g5-monitor-explicit-held | 13.0005/13.0005 | 1727.856/1727.856 | 17726.992 | 18061.4965 | 1.886978 | 17750.9075 | 17801.4765 | 0.284881 | True |
| g5-names-default | 6253.005/6253.005 | 552512.36/552512.36 | 3482631.395 | 3482565.555 | -0.001891 | 3482383.285 | 3482956.84 | 0.016470 | False |
| g5-testreport-options-default | 54.0/54.0 | 5609.0/5609.0 | 42863.6815 | 43383.8085 | 1.213445 | 42817.885 | 43241.6755 | 0.989751 | True |
| g5-testreport-unread-default | 41.0005/41.0005 | 4598.4/4598.4 | 28796.2615 | 28843.955 | 0.165624 | 28792.023 | 28842.5055 | 0.175335 | True |
| g5-monitor-evaluationmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13301.998 | 13393.656 | 0.689054 | 13351.3475 | 13393.4305 | 0.315197 | False |
| g5-monitor-stepmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13353.3375 | 13391.1795 | 0.283390 | 13353.9655 | 13390.017 | 0.269968 | True |
| g5-names-spellingcorrection-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473166.52 | 3473532.295 | 0.010531 | 3473456.94 | 3473348.12 | -0.003133 | False |
| g5-names-resolvecontextaliases-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473090.955 | 3473422.89 | 0.009557 | 3473211.27 | 3473272.285 | 0.001757 | False |
| g5-names-ignorecase-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3470409.905 | 3470747.475 | 0.009727 | 3470686.88 | 3470776.775 | 0.002590 | False |
| g5-testreport-sametest-latent | 52.0/52.0 | 4969.0/4969.0 | 27509.3565 | 27886.894 | 1.372397 | 27565.5615 | 27933.3095 | 1.334085 | True |
| g5-testreport-timeconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27596.191 | 27948.688 | 1.277339 | 27596.125 | 27863.555 | 0.969085 | True |
| g5-testreport-memoryconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27601.311 | 27954.479 | 1.279533 | 27600.839 | 27953.0195 | 1.275978 | True |

#### candidate-02

[Hashes/build identity](../../../.temp/brief-578/candidate-02/hashes.json); [result checks](../../../.temp/brief-578/candidate-02/perf/checks.json); [raw counts/streams/status](../../../.temp/brief-578/candidate-02/perf/paired-results.json); [full precision TSV](../../../.temp/brief-578/candidate-02/perf/slopes.tsv).

| Label | Alloc base/new | Bytes base/new | I base1 | I new1 | Δ1 % | I base2 | I new2 | Δ2 % | Regresses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 9.0/9.0 | 912.0/912.0 | 15929.2826 | 16008.921 | 0.499950 | 15954.89025 | 16028.8556 | 0.463590 | False |
| g5-cases-small-delayed | 8.0/8.0 | 656.0/656.0 | 14790.26935 | 14768.58265 | -0.146628 | 14790.2912 | 14780.83895 | -0.063908 | False |
| g5-cases-small-explicit | 11.0/11.0 | 1104.0/1104.0 | 17071.7325 | 16933.5293 | -0.809544 | 17108.15815 | 16855.50245 | -1.476814 | False |
| g5-count-small-default | 6.0/6.0 | 664.0/664.0 | 13293.72785 | 13161.7041 | -0.993128 | 13135.5781 | 13161.6935 | 0.198814 | False |
| g5-count-small-delayed | 5.0/5.0 | 408.0/408.0 | 11988.86975 | 11981.00205 | -0.065625 | 12006.0677 | 12006.95585 | 0.007398 | False |
| g5-count-small-explicit | 8.0/8.0 | 824.0/824.0 | 13954.2651 | 13967.5088 | 0.094908 | 13954.4597 | 13967.1877 | 0.091211 | False |
| g5-memberq-small-default | 6.0/6.0 | 760.0/760.0 | 11426.2528 | 11410.1661 | -0.140787 | 11450.33535 | 11408.33255 | -0.366826 | False |
| g5-memberq-small-delayed | 5.0/5.0 | 504.0/504.0 | 10281.13025 | 10316.2613 | 0.341704 | 10296.12355 | 10286.0533 | -0.097806 | False |
| g5-memberq-small-explicit | 7.0/7.0 | 792.0/792.0 | 12159.131 | 12033.99275 | -1.029171 | 12036.89575 | 12008.0008 | -0.240053 | False |
| g5-freeq-small-default | 7.0/7.0 | 1208.0/1208.0 | 12561.6547 | 12449.77355 | -0.890656 | 12465.79775 | 12449.76955 | -0.128577 | False |
| g5-freeq-small-delayed | 6.0/6.0 | 952.0/952.0 | 11222.72495 | 11186.6448 | -0.321492 | 11211.6487 | 11201.8324 | -0.087554 | False |
| g5-freeq-small-explicit | 8.0/8.0 | 1240.0/1240.0 | 13029.4715 | 13039.58415 | 0.077614 | 13138.2204 | 12881.47545 | -1.954184 | False |
| g5-position-small-default | 17.0/17.0 | 2136.0/2136.0 | 19295.99295 | 19375.08235 | 0.409875 | 19341.26255 | 19315.10115 | -0.135262 | False |
| g5-position-small-delayed | 16.0/16.0 | 1880.0/1880.0 | 18145.3441 | 18160.16 | 0.081651 | 18360.24925 | 18129.64325 | -1.256007 | False |
| g5-position-small-explicit | 18.0/18.0 | 2168.0/2168.0 | 19909.89895 | 19898.3864 | -0.057823 | 19909.66325 | 19898.1564 | -0.057795 | False |
| g5-deletecases-small-default | 9.0/9.0 | 984.0/984.0 | 14604.82675 | 14416.24765 | -1.291211 | 14502.6095 | 14440.16175 | -0.430597 | False |
| g5-deletecases-small-delayed | 8.0/8.0 | 728.0/728.0 | 13321.76145 | 13390.1082 | 0.513046 | 13352.368 | 13358.11915 | 0.043072 | False |
| g5-deletecases-small-explicit | 10.0/10.0 | 1048.0/1048.0 | 15224.21225 | 15259.00485 | 0.228535 | 15253.9561 | 15205.0632 | -0.320526 | False |
| g5-replace-small-default | 20.0/20.0 | 1945.0/1945.0 | 20244.424 | 20244.7064 | 0.001395 | 20214.40325 | 20190.26635 | -0.119404 | False |
| g5-replace-small-delayed | 19.0/19.0 | 1689.0/1689.0 | 19050.7264 | 19065.2856 | 0.076423 | 19065.38205 | 19063.6116 | -0.009286 | False |
| g5-replace-small-explicit | 22.0/22.0 | 2233.0/2233.0 | 21266.73345 | 21344.48655 | 0.365609 | 21415.8512 | 21343.895 | -0.335995 | False |
| g5-level-small-default | 13.0/13.0 | 1009.0/1009.0 | 13359.4717 | 13347.4036 | -0.090334 | 13364.9276 | 13348.0036 | -0.126630 | False |
| g5-level-small-delayed | 12.0/12.0 | 753.0/753.0 | 12189.9538 | 12183.19095 | -0.055479 | 12207.3507 | 12182.7417 | -0.201592 | False |
| g5-level-small-explicit | 13.0/13.0 | 1009.0/1009.0 | 14237.02255 | 14229.55275 | -0.052467 | 14253.58695 | 14228.8825 | -0.173321 | False |
| g5-depth-small-default | 4.0/4.0 | 392.0/392.0 | 9080.1726 | 8908.5508 | -1.890072 | 8903.4946 | 8804.34095 | -1.113649 | False |
| g5-depth-small-delayed | 3.0/3.0 | 136.0/136.0 | 7641.6224 | 7656.61955 | 0.196256 | 7643.55225 | 7563.59845 | -1.046029 | False |
| g5-depth-small-explicit | 4.0/4.0 | 392.0/392.0 | 9665.67425 | 9665.66015 | -0.000146 | 9665.76215 | 9652.68385 | -0.135305 | False |
| g5-trace-small-default | 33.0/33.0 | 1992.0/1992.0 | 27526.709 | 27506.00525 | -0.075213 | 27525.6804 | 27531.0742 | 0.019596 | False |
| g5-trace-small-delayed | 32.0/32.0 | 1736.0/1736.0 | 26311.64465 | 26307.94535 | -0.014060 | 26316.68235 | 26328.36285 | 0.044384 | False |
| g5-trace-small-explicit | 36.0/36.0 | 2120.0/2120.0 | 31726.3801 | 31719.98225 | -0.020166 | 31811.07775 | 31871.39885 | 0.189623 | False |
| g5-tracedetail-small-default | 55.0/55.0 | 3263.0/3263.0 | 36082.97835 | 36124.1673 | 0.114151 | 36070.22065 | 36116.4126 | 0.128061 | False |
| g5-tracedetail-small-delayed | 54.0/54.0 | 3007.0/3007.0 | 34897.89215 | 34837.3733 | -0.173417 | 34990.88135 | 34847.4172 | -0.410004 | False |
| g5-tracedetail-small-explicit | 57.0/57.0 | 3359.0/3359.0 | 39059.8631 | 39010.6855 | -0.125903 | 39094.32645 | 38986.993 | -0.274550 | False |
| g5-traceprint-small-default | 40.0/40.0 | 2174.0/2174.0 | 30557.1485 | 30640.5095 | 0.272804 | 30625.8685 | 30643.172 | 0.056500 | False |
| g5-traceprint-small-delayed | 39.0/39.0 | 1918.0/1918.0 | 29255.1385 | 29419.3975 | 0.561471 | 29537.878 | 29408.378 | -0.438420 | False |
| g5-traceprint-small-explicit | 43.0/43.0 | 2302.0/2302.0 | 34821.103 | 34821.2285 | 0.000360 | 34821.319 | 34819.1025 | -0.006365 | False |
| matchq-integer-success | 1.0/1.0 | 128.0/128.0 | 4340.3071 | 4340.1243 | -0.004212 | 4340.07315 | 4340.16035 | 0.002009 | False |
| matchq-integer-fail | 2.0/2.0 | 129.0/129.0 | 4462.11795 | 4462.2639 | 0.003271 | 4462.2029 | 4462.1828 | -0.000450 | False |
| cases-wide-no-match | 33.001/33.001 | 263247.864/263247.864 | 2400533.401 | 2400461.86 | -0.002980 | 2400482.352 | 2400530.262 | 0.001996 | False |
| count-wide-no-match | 30.0/30.0 | 262601.0/262601.0 | 2178889.759 | 2178889.863 | 0.000005 | 2178940.295 | 2178890.634 | -0.002279 | False |
| position-wide-no-match | 1642.001/1642.001 | 576255.864/576255.864 | 2573306.605 | 2573265.697 | -0.001590 | 2573239.886 | 2573245.119 | 0.000203 | False |
| deletecases-wide-no-match | 24.001/24.001 | 247447.864/247447.864 | 2670168.474 | 2670167.885 | -0.000022 | 2670216.089 | 2670266.756 | 0.001897 | False |
| replace-wide-no-match | 30.001/30.001 | 184455.864/184455.864 | 1163561.987 | 1163637.678 | 0.006505 | 1163606.155 | 1163613.254 | 0.000610 | False |
| dot-three-no-input | 596.0/596.0 | 23472.0/23472.0 | 253105.21655 | 253103.09055 | -0.000840 | 253103.44625 | 253104.76175 | 0.000520 | False |
| differences-no-input | 13.0/13.0 | 672.0/672.0 | 9418.0209 | 9399.8102 | -0.193360 | 9397.55705 | 9399.44195 | 0.020057 | False |
| take-multispec-small | 109.0/109.0 | 5824.0/5824.0 | 49240.34305 | 49231.0396 | -0.018894 | 49232.77055 | 49230.92135 | -0.003756 | False |
| g5-monitor-options-none | 9.0/9.0 | 953.0/953.0 | 18107.4675 | 18254.0495 | 0.809511 | 18067.3625 | 18582.1555 | 2.849298 | True |
| g5-monitor-explicit-held | 13.0005/13.0005 | 1727.856/1727.856 | 17746.81 | 17893.006 | 0.823787 | 17746.247 | 17826.2635 | 0.450893 | True |
| g5-names-default | 6253.005/6253.005 | 552512.36/552512.36 | 3482930.94 | 3482544.915 | -0.011083 | 3482411.345 | 3482780.67 | 0.010605 | False |
| g5-testreport-options-default | 54.0/54.0 | 5609.0/5609.0 | 42991.6075 | 43358.787 | 0.854073 | 43217.329 | 43288.9325 | 0.165682 | True |
| g5-testreport-unread-default | 41.0005/41.0005 | 4598.4/4598.4 | 28816.1235 | 28846.3565 | 0.104917 | 28790.2705 | 28846.0355 | 0.193694 | True |
| g5-monitor-evaluationmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13349.1565 | 13394.657 | 0.340849 | 13350.843 | 13490.0325 | 1.042552 | False |
| g5-monitor-stepmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13353.545 | 13391.2425 | 0.282303 | 13353.2605 | 13390.507 | 0.278932 | True |
| g5-names-spellingcorrection-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473091.28 | 3473464.645 | 0.010750 | 3473391.36 | 3473214.985 | -0.005078 | False |
| g5-names-resolvecontextaliases-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473170.355 | 3473223.935 | 0.001543 | 3473399.575 | 3473528.265 | 0.003705 | False |
| g5-names-ignorecase-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3470429.855 | 3470772.315 | 0.009868 | 3470596.835 | 3471383.825 | 0.022676 | False |
| g5-testreport-sametest-latent | 52.0/52.0 | 4969.0/4969.0 | 27544.362 | 27891.655 | 1.260850 | 27671.624 | 27937.7955 | 0.961893 | True |
| g5-testreport-timeconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27642.132 | 27906.366 | 0.955910 | 27661.8455 | 27893.5095 | 0.837486 | True |
| g5-testreport-memoryconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27628.1695 | 27913.0375 | 1.031078 | 27601.8215 | 27912.613 | 1.125982 | True |

#### candidate-03

[Hashes/build identity](../../../.temp/brief-578/candidate-03/hashes.json); [result checks](../../../.temp/brief-578/candidate-03/perf/checks.json); [raw counts/streams/status](../../../.temp/brief-578/candidate-03/perf/paired-results.json); [full precision TSV](../../../.temp/brief-578/candidate-03/perf/slopes.tsv).

| Label | Alloc base/new | Bytes base/new | I base1 | I new1 | Δ1 % | I base2 | I new2 | Δ2 % | Regresses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 9.0/9.0 | 912.0/912.0 | 16185.63305 | 15931.4783 | -1.570249 | 16007.55165 | 15980.34345 | -0.169971 | False |
| g5-cases-small-delayed | 8.0/8.0 | 656.0/656.0 | 14859.0614 | 14782.324 | -0.516435 | 14790.291 | 14810.7421 | 0.138274 | False |
| g5-cases-small-explicit | 11.0/11.0 | 1104.0/1104.0 | 16872.952 | 16971.69775 | 0.585231 | 16933.56085 | 16933.50655 | -0.000321 | False |
| g5-count-small-default | 6.0/6.0 | 664.0/664.0 | 13097.53875 | 13161.75865 | 0.490320 | 13113.66775 | 13195.58175 | 0.624646 | False |
| g5-count-small-delayed | 5.0/5.0 | 408.0/408.0 | 12014.7835 | 12029.0139 | 0.118441 | 12005.8388 | 11993.8982 | -0.099457 | False |
| g5-count-small-explicit | 8.0/8.0 | 824.0/824.0 | 14024.30305 | 13967.3161 | -0.406344 | 13941.42465 | 13967.3191 | 0.185737 | False |
| g5-memberq-small-default | 6.0/6.0 | 760.0/760.0 | 11450.2241 | 11410.2799 | -0.348851 | 11420.2001 | 11434.22845 | 0.122838 | False |
| g5-memberq-small-delayed | 5.0/5.0 | 504.0/504.0 | 10307.2713 | 10286.03265 | -0.206055 | 10307.063 | 10375.2947 | 0.661990 | False |
| g5-memberq-small-explicit | 7.0/7.0 | 792.0/792.0 | 12169.05455 | 12034.07465 | -1.109206 | 12037.0381 | 12094.95355 | 0.481144 | False |
| g5-freeq-small-default | 7.0/7.0 | 1208.0/1208.0 | 12465.69255 | 12525.57065 | 0.480343 | 12465.5734 | 12509.75395 | 0.354421 | False |
| g5-freeq-small-delayed | 6.0/6.0 | 952.0/952.0 | 11268.6221 | 11188.7882 | -0.708462 | 11226.6603 | 11201.83055 | -0.221168 | False |
| g5-freeq-small-explicit | 8.0/8.0 | 1240.0/1240.0 | 13042.61085 | 12971.73465 | -0.543420 | 12994.46785 | 13013.342 | 0.145248 | False |
| g5-position-small-default | 17.0/17.0 | 2136.0/2136.0 | 19611.5799 | 19315.1219 | -1.511648 | 19351.20705 | 19352.00045 | 0.004100 | False |
| g5-position-small-delayed | 16.0/16.0 | 1880.0/1880.0 | 18161.17315 | 18130.19105 | -0.170595 | 18059.10475 | 18104.23655 | 0.249912 | False |
| g5-position-small-explicit | 18.0/18.0 | 2168.0/2168.0 | 19879.6246 | 19928.7666 | 0.247198 | 19894.56405 | 19887.61555 | -0.034927 | False |
| g5-deletecases-small-default | 9.0/9.0 | 984.0/984.0 | 14494.42175 | 14594.42015 | 0.689910 | 14495.4328 | 14459.96205 | -0.244703 | False |
| g5-deletecases-small-delayed | 8.0/8.0 | 728.0/728.0 | 13351.66055 | 13381.9438 | 0.226813 | 13364.22935 | 13362.16895 | -0.015417 | False |
| g5-deletecases-small-explicit | 10.0/10.0 | 1048.0/1048.0 | 15245.5042 | 15198.9964 | -0.305059 | 15243.2844 | 15329.3885 | 0.564866 | False |
| g5-replace-small-default | 20.0/20.0 | 1945.0/1945.0 | 20214.44435 | 20214.3474 | -0.000480 | 20214.3138 | 20214.61495 | 0.001490 | False |
| g5-replace-small-delayed | 19.0/19.0 | 1689.0/1689.0 | 19048.8436 | 19047.7496 | -0.005743 | 19072.6584 | 19114.7084 | 0.220473 | False |
| g5-replace-small-explicit | 22.0/22.0 | 2233.0/2233.0 | 21375.949 | 21249.3157 | -0.592410 | 21268.79915 | 21319.69575 | 0.239302 | False |
| g5-level-small-default | 13.0/13.0 | 1009.0/1009.0 | 13361.3407 | 13408.4771 | 0.352782 | 13361.36925 | 13445.49395 | 0.629611 | False |
| g5-level-small-delayed | 12.0/12.0 | 753.0/753.0 | 12190.8569 | 12182.99375 | -0.064500 | 12188.1257 | 12137.4987 | -0.415380 | False |
| g5-level-small-explicit | 13.0/13.0 | 1009.0/1009.0 | 14253.6139 | 14197.87405 | -0.391058 | 14581.97505 | 14229.34085 | -2.418288 | False |
| g5-depth-small-default | 4.0/4.0 | 392.0/392.0 | 8852.2028 | 8804.50065 | -0.538873 | 8804.4116 | 8804.33385 | -0.000883 | False |
| g5-depth-small-delayed | 3.0/3.0 | 136.0/136.0 | 7656.45085 | 7611.46015 | -0.587618 | 7617.7076 | 7677.5116 | 0.785066 | False |
| g5-depth-small-explicit | 4.0/4.0 | 392.0/392.0 | 9665.6898 | 9665.7439 | 0.000560 | 9665.8876 | 9652.61755 | -0.137287 | False |
| g5-trace-small-default | 33.0/33.0 | 1992.0/1992.0 | 27458.5803 | 27529.6174 | 0.258706 | 27526.42105 | 27362.9345 | -0.593926 | False |
| g5-trace-small-delayed | 32.0/32.0 | 1736.0/1736.0 | 26239.546 | 26210.17915 | -0.111918 | 26292.7077 | 26388.1915 | 0.363157 | False |
| g5-trace-small-explicit | 36.0/36.0 | 2120.0/2120.0 | 31741.14315 | 31840.62455 | 0.313415 | 31922.9666 | 31815.13045 | -0.337801 | False |
| g5-tracedetail-small-default | 55.0/55.0 | 3263.0/3263.0 | 36050.68845 | 36044.80425 | -0.016322 | 36080.80745 | 36184.3406 | 0.286948 | False |
| g5-tracedetail-small-delayed | 54.0/54.0 | 3007.0/3007.0 | 34981.1916 | 34918.8723 | -0.178151 | 35000.52005 | 34933.12715 | -0.192548 | False |
| g5-tracedetail-small-explicit | 57.0/57.0 | 3359.0/3359.0 | 38996.3707 | 39010.6313 | 0.036569 | 38987.7449 | 39010.88415 | 0.059350 | False |
| g5-traceprint-small-default | 40.0/40.0 | 2174.0/2174.0 | 30626.5045 | 30716.701 | 0.294505 | 30833.288 | 30655.207 | -0.577561 | False |
| g5-traceprint-small-delayed | 39.0/39.0 | 1918.0/1918.0 | 29421.799 | 29459.3165 | 0.127516 | 29471.3695 | 29434.8435 | -0.123937 | False |
| g5-traceprint-small-explicit | 43.0/43.0 | 2302.0/2302.0 | 34870.4205 | 34820.133 | -0.144212 | 34820.64 | 34773.021 | -0.136755 | False |
| matchq-integer-success | 1.0/1.0 | 128.0/128.0 | 4340.0692 | 4340.13285 | 0.001467 | 4340.364 | 4340.0303 | -0.007688 | False |
| matchq-integer-fail | 2.0/2.0 | 129.0/129.0 | 4462.14945 | 4462.23755 | 0.001974 | 4462.2123 | 4462.12155 | -0.002034 | False |
| cases-wide-no-match | 33.001/33.001 | 263247.864/263247.864 | 2400532.964 | 2400556.241 | 0.000970 | 2400586.203 | 2400662.585 | 0.003182 | False |
| count-wide-no-match | 30.0/30.0 | 262601.0/262601.0 | 2178857.725 | 2178888.888 | 0.001430 | 2178883.365 | 2178864.017 | -0.000888 | False |
| position-wide-no-match | 1642.001/1642.001 | 576255.864/576255.864 | 2573268.402 | 2573267.992 | -0.000016 | 2573330.708 | 2573211.993 | -0.004613 | False |
| deletecases-wide-no-match | 24.001/24.001 | 247447.864/247447.864 | 2670143.979 | 2670141.912 | -0.000077 | 2670167.881 | 2670170.612 | 0.000102 | False |
| replace-wide-no-match | 30.001/30.001 | 184455.864/184455.864 | 1163680.354 | 1163653.093 | -0.002343 | 1163722.765 | 1163587.998 | -0.011581 | False |
| dot-three-no-input | 596.0/596.0 | 23472.0/23472.0 | 253107.62555 | 253105.90805 | -0.000679 | 253105.46145 | 253102.69625 | -0.001093 | False |
| differences-no-input | 13.0/13.0 | 672.0/672.0 | 9411.98915 | 9399.25485 | -0.135299 | 9403.6968 | 9399.3961 | -0.045734 | False |
| take-multispec-small | 109.0/109.0 | 5824.0/5824.0 | 49248.40045 | 49230.6146 | -0.036115 | 49232.62655 | 49230.8853 | -0.003537 | False |
| g5-monitor-options-none | 9.0/9.0 | 953.0/953.0 | 18067.906 | 18173.264 | 0.583122 | 18071.713 | 18146.429 | 0.413442 | False |
| g5-monitor-explicit-held | 13.0005/13.0005 | 1727.856/1727.856 | 17777.204 | 17828.063 | 0.286091 | 17785.803 | 17825.507 | 0.223234 | True |
| g5-names-default | 6253.005/6253.005 | 552512.36/552512.36 | 3482194.445 | 3482309.905 | 0.003316 | 3482824.015 | 3482832.18 | 0.000234 | False |
| g5-testreport-options-default | 54.0/54.0 | 5609.0/5609.0 | 42889.384 | 43293.7005 | 0.942696 | 43141.6375 | 43434.6195 | 0.679117 | True |
| g5-testreport-unread-default | 41.0005/41.0005 | 4598.4/4598.4 | 28789.374 | 28846.3675 | 0.197967 | 28731.3975 | 28845.336 | 0.396564 | True |
| g5-monitor-evaluationmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13352.6365 | 13392.6755 | 0.299858 | 13351.6905 | 13395.199 | 0.325865 | False |
| g5-monitor-stepmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13352.836 | 13391.4105 | 0.288886 | 13353.128 | 13486.1955 | 0.996527 | True |
| g5-names-spellingcorrection-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473094.02 | 3473037.94 | -0.001615 | 3473400.0 | 3473428.265 | 0.000814 | False |
| g5-names-resolvecontextaliases-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473027.68 | 3473176.085 | 0.004273 | 3473015.125 | 3473257.765 | 0.006986 | False |
| g5-names-ignorecase-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3470460.535 | 3470642.555 | 0.005245 | 3470622.795 | 3470591.655 | -0.000897 | False |
| g5-testreport-sametest-latent | 52.0/52.0 | 4969.0/4969.0 | 27579.6135 | 27894.872 | 1.143085 | 27593.385 | 27919.477 | 1.181776 | True |
| g5-testreport-timeconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27595.7295 | 27933.851 | 1.225267 | 27740.239 | 28079.332 | 1.222387 | True |
| g5-testreport-memoryconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27601.7885 | 28059.2525 | 1.657371 | 27600.2995 | 27913.7295 | 1.135604 | True |

#### candidate-04 — selected source

[Hashes/build identity](../../../.temp/brief-578/candidate-04/hashes.json); [result checks](../../../.temp/brief-578/candidate-04/perf/checks.json); [raw counts/streams/status](../../../.temp/brief-578/candidate-04/perf/paired-results.json); [full precision TSV](../../../.temp/brief-578/candidate-04/perf/slopes.tsv).

| Label | Alloc base/new | Bytes base/new | I base1 | I new1 | Δ1 % | I base2 | I new2 | Δ2 % | Regresses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 9.0/9.0 | 912.0/912.0 | 15928.63205 | 15931.4753 | 0.017850 | 15929.57365 | 16089.817 | 1.005949 | False |
| g5-cases-small-delayed | 8.0/8.0 | 656.0/656.0 | 14820.3978 | 14767.37175 | -0.357791 | 14790.34165 | 14766.5889 | -0.160596 | False |
| g5-cases-small-explicit | 11.0/11.0 | 1104.0/1104.0 | 16966.99075 | 16963.38715 | -0.021239 | 17031.13645 | 16921.3347 | -0.644712 | False |
| g5-count-small-default | 6.0/6.0 | 664.0/664.0 | 13122.5192 | 13122.75375 | 0.001787 | 13161.6013 | 13208.56855 | 0.356851 | False |
| g5-count-small-delayed | 5.0/5.0 | 408.0/408.0 | 12057.77475 | 12031.91075 | -0.214501 | 11988.91575 | 11993.91405 | 0.041691 | False |
| g5-count-small-explicit | 8.0/8.0 | 824.0/824.0 | 14006.4277 | 13967.4242 | -0.278469 | 13967.2546 | 13979.20245 | 0.085542 | False |
| g5-memberq-small-default | 6.0/6.0 | 760.0/760.0 | 11450.1638 | 11434.31915 | -0.138379 | 11450.12635 | 11374.3413 | -0.661871 | False |
| g5-memberq-small-delayed | 5.0/5.0 | 504.0/504.0 | 10257.3911 | 10324.3797 | 0.653076 | 10283.1443 | 10286.26735 | 0.030371 | False |
| g5-memberq-small-explicit | 7.0/7.0 | 792.0/792.0 | 12054.0915 | 12143.85025 | 0.744633 | 12037.10185 | 12021.183 | -0.132248 | False |
| g5-freeq-small-default | 7.0/7.0 | 1208.0/1208.0 | 12415.6603 | 12449.78775 | 0.274874 | 12525.59625 | 12497.6414 | -0.223182 | False |
| g5-freeq-small-delayed | 6.0/6.0 | 952.0/952.0 | 11250.7294 | 11201.8733 | -0.434248 | 11209.8294 | 11201.60025 | -0.073410 | False |
| g5-freeq-small-explicit | 8.0/8.0 | 1240.0/1240.0 | 13004.31735 | 13026.43745 | 0.170098 | 12977.487 | 13013.3359 | 0.276239 | False |
| g5-position-small-default | 17.0/17.0 | 2136.0/2136.0 | 19352.35365 | 19315.29 | -0.191520 | 19344.0239 | 19375.2587 | 0.161470 | False |
| g5-position-small-delayed | 16.0/16.0 | 1880.0/1880.0 | 18286.2264 | 18131.53425 | -0.845949 | 18176.29065 | 18091.2498 | -0.467867 | False |
| g5-position-small-explicit | 18.0/18.0 | 2168.0/2168.0 | 19909.7077 | 19898.52015 | -0.056191 | 19909.61325 | 19857.1369 | -0.263573 | False |
| g5-deletecases-small-default | 9.0/9.0 | 984.0/984.0 | 14535.36575 | 14528.3714 | -0.048120 | 14430.3817 | 14528.22135 | 0.678012 | False |
| g5-deletecases-small-delayed | 8.0/8.0 | 728.0/728.0 | 13351.5406 | 13323.9994 | -0.206277 | 13338.80705 | 13412.0171 | 0.548850 | False |
| g5-deletecases-small-explicit | 10.0/10.0 | 1048.0/1048.0 | 15387.46465 | 15229.16805 | -1.028737 | 15242.38925 | 15190.86915 | -0.338005 | False |
| g5-replace-small-default | 20.0/20.0 | 1945.0/1945.0 | 20166.4354 | 20136.5054 | -0.148415 | 20274.3765 | 20510.23965 | 1.163356 | False |
| g5-replace-small-delayed | 19.0/19.0 | 1689.0/1689.0 | 19072.74315 | 19065.0965 | -0.040092 | 19048.74135 | 19063.65765 | 0.078306 | False |
| g5-replace-small-explicit | 22.0/22.0 | 2233.0/2233.0 | 21467.82075 | 21282.1434 | -0.864910 | 21268.6556 | 21268.5653 | -0.000425 | False |
| g5-level-small-default | 13.0/13.0 | 1009.0/1009.0 | 13418.6761 | 13325.1071 | -0.697304 | 13361.80415 | 13350.67185 | -0.083314 | False |
| g5-level-small-delayed | 12.0/12.0 | 753.0/753.0 | 12237.1146 | 12182.01835 | -0.450239 | 12320.10395 | 12179.43135 | -1.141813 | False |
| g5-level-small-explicit | 13.0/13.0 | 1009.0/1009.0 | 14212.27645 | 14227.7584 | 0.108934 | 14305.8444 | 14227.798 | -0.545556 | False |
| g5-depth-small-default | 4.0/4.0 | 392.0/392.0 | 8810.40035 | 8852.3736 | 0.476406 | 9054.3052 | 8804.30715 | -2.761096 | False |
| g5-depth-small-delayed | 3.0/3.0 | 136.0/136.0 | 7695.54825 | 7704.67265 | 0.118567 | 7656.55835 | 7660.4613 | 0.050975 | False |
| g5-depth-small-explicit | 4.0/4.0 | 392.0/392.0 | 9691.67535 | 9798.7195 | 1.104496 | 9669.6684 | 9678.755 | 0.093970 | False |
| g5-trace-small-default | 33.0/33.0 | 1992.0/1992.0 | 27478.41015 | 27528.72195 | 0.183096 | 27525.56875 | 27536.0724 | 0.038160 | False |
| g5-trace-small-delayed | 32.0/32.0 | 1736.0/1736.0 | 26333.00945 | 26305.1618 | -0.105752 | 26316.7894 | 26329.6187 | 0.048749 | False |
| g5-trace-small-explicit | 36.0/36.0 | 2120.0/2120.0 | 31740.91495 | 31720.5539 | -0.064148 | 31710.73865 | 31719.54835 | 0.027781 | False |
| g5-tracedetail-small-default | 55.0/55.0 | 3263.0/3263.0 | 36035.0742 | 36124.306 | 0.247625 | 36034.9071 | 36088.08795 | 0.147581 | True |
| g5-tracedetail-small-delayed | 54.0/54.0 | 3007.0/3007.0 | 34930.3658 | 34925.0171 | -0.015312 | 34871.1228 | 34972.63705 | 0.291113 | False |
| g5-tracedetail-small-explicit | 57.0/57.0 | 3359.0/3359.0 | 38939.8151 | 39106.79005 | 0.428803 | 39007.57845 | 39086.9112 | 0.203378 | False |
| g5-traceprint-small-default | 40.0/40.0 | 2174.0/2174.0 | 30628.3255 | 30639.2395 | 0.035634 | 30627.988 | 30736.6175 | 0.354674 | False |
| g5-traceprint-small-delayed | 39.0/39.0 | 1918.0/1918.0 | 29361.3535 | 29589.138 | 0.775797 | 29469.6275 | 29435.4285 | -0.116048 | False |
| g5-traceprint-small-explicit | 43.0/43.0 | 2302.0/2302.0 | 34916.9355 | 34820.0925 | -0.277353 | 34963.698 | 34820.316 | -0.410088 | False |
| matchq-integer-success | 1.0/1.0 | 128.0/128.0 | 4340.3545 | 4340.3265 | -0.000645 | 4340.1391 | 4340.2519 | 0.002599 | False |
| matchq-integer-fail | 2.0/2.0 | 129.0/129.0 | 4462.2248 | 4462.1889 | -0.000805 | 4461.96365 | 4462.18205 | 0.004895 | False |
| cases-wide-no-match | 33.001/33.001 | 263247.864/263247.864 | 2400562.335 | 2400530.847 | -0.001312 | 2400484.512 | 2400528.515 | 0.001833 | False |
| count-wide-no-match | 30.0/30.0 | 262601.0/262601.0 | 2178888.186 | 2178891.129 | 0.000135 | 2178843.459 | 2178891.02 | 0.002183 | False |
| position-wide-no-match | 1642.001/1642.001 | 576255.864/576255.864 | 2573276.205 | 2573272.252 | -0.000154 | 2573336.364 | 2573336.174 | -0.000007 | False |
| deletecases-wide-no-match | 24.001/24.001 | 247447.864/247447.864 | 2670164.288 | 2670407.202 | 0.009097 | 2670064.068 | 2670165.986 | 0.003817 | False |
| replace-wide-no-match | 30.001/30.001 | 184455.864/184455.864 | 1163610.768 | 1163498.789 | -0.009623 | 1163563.66 | 1163598.77 | 0.003017 | False |
| dot-three-no-input | 596.0/596.0 | 23472.0/23472.0 | 253107.99485 | 253103.0101 | -0.001969 | 253107.51415 | 253106.177 | -0.000528 | False |
| differences-no-input | 13.0/13.0 | 672.0/672.0 | 9396.42555 | 9399.89285 | 0.036900 | 9396.49315 | 9399.43885 | 0.031349 | False |
| take-multispec-small | 109.0/109.0 | 5824.0/5824.0 | 49248.84285 | 49231.0116 | -0.036206 | 49241.45645 | 49231.1818 | -0.020866 | False |
| g5-monitor-options-none | 9.0/9.0 | 953.0/953.0 | 18360.3305 | 18147.974 | -1.156605 | 18157.991 | 18175.373 | 0.095726 | False |
| g5-monitor-explicit-held | 13.0005/13.0005 | 1727.856/1727.856 | 17770.383 | 17824.3325 | 0.303592 | 17833.1915 | 17871.1675 | 0.212951 | False |
| g5-names-default | 6253.005/6253.005 | 552512.36/552512.36 | 3482338.915 | 3482728.03 | 0.011174 | 3482489.12 | 3482226.135 | -0.007552 | False |
| g5-testreport-options-default | 54.0/54.0 | 5609.0/5609.0 | 43161.4555 | 42936.324 | -0.521603 | 43175.795 | 42893.0455 | -0.654880 | False |
| g5-testreport-unread-default | 41.0005/41.0005 | 4598.4/4598.4 | 28789.1765 | 28847.0305 | 0.200957 | 28788.2655 | 28848.9125 | 0.210666 | True |
| g5-monitor-evaluationmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13349.709 | 13346.1765 | -0.026461 | 13302.7445 | 13395.263 | 0.695484 | False |
| g5-monitor-stepmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13354.248 | 13271.8455 | -0.617051 | 13352.8965 | 13342.195 | -0.080144 | False |
| g5-names-spellingcorrection-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473057.55 | 3473375.805 | 0.009164 | 3473543.305 | 3473494.43 | -0.001407 | False |
| g5-names-resolvecontextaliases-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3472953.78 | 3473289.265 | 0.009660 | 3473218.375 | 3473268.11 | 0.001432 | False |
| g5-names-ignorecase-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3470590.245 | 3470756.335 | 0.004786 | 3470443.12 | 3470571.255 | 0.003692 | False |
| g5-testreport-sametest-latent | 52.0/52.0 | 4969.0/4969.0 | 27509.0625 | 27741.6305 | 0.845423 | 27567.8265 | 27933.0125 | 1.324682 | True |
| g5-testreport-timeconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27595.7655 | 27753.3415 | 0.571015 | 27594.1475 | 28159.0005 | 2.047003 | True |
| g5-testreport-memoryconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27766.194 | 27760.1945 | -0.021607 | 27529.5325 | 27807.2605 | 1.008837 | False |

#### candidate-05

[Hashes/build identity](../../../.temp/brief-578/candidate-05/hashes.json); [result checks](../../../.temp/brief-578/candidate-05/perf/checks.json); [raw counts/streams/status](../../../.temp/brief-578/candidate-05/perf/paired-results.json); [full precision TSV](../../../.temp/brief-578/candidate-05/perf/slopes.tsv).

| Label | Alloc base/new | Bytes base/new | I base1 | I new1 | Δ1 % | I base2 | I new2 | Δ2 % | Regresses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 9.0/9.0 | 912.0/912.0 | 15881.1151 | 15908.58375 | 0.172964 | 15899.17975 | 15932.95125 | 0.212410 | False |
| g5-cases-small-delayed | 8.0/8.0 | 656.0/656.0 | 14768.53505 | 14901.7633 | 0.902109 | 14836.04565 | 14900.222 | 0.432570 | True |
| g5-cases-small-explicit | 11.0/11.0 | 1104.0/1104.0 | 17109.7819 | 16933.4514 | -1.030583 | 16941.0155 | 16933.625 | -0.043625 | False |
| g5-count-small-default | 6.0/6.0 | 664.0/664.0 | 13113.6548 | 13148.72155 | 0.267406 | 13161.71075 | 13135.6495 | -0.198008 | False |
| g5-count-small-delayed | 5.0/5.0 | 408.0/408.0 | 12045.139 | 12006.79365 | -0.318347 | 12012.90265 | 12006.9439 | -0.049603 | False |
| g5-count-small-explicit | 8.0/8.0 | 824.0/824.0 | 13945.30095 | 13899.2603 | -0.330152 | 13993.4428 | 13967.50805 | -0.185335 | False |
| g5-memberq-small-default | 6.0/6.0 | 760.0/760.0 | 11426.0297 | 11396.2287 | -0.260817 | 11450.29665 | 11434.1224 | -0.141256 | False |
| g5-memberq-small-delayed | 5.0/5.0 | 504.0/504.0 | 10320.3317 | 10334.13945 | 0.133792 | 10281.05685 | 10324.0674 | 0.418348 | False |
| g5-memberq-small-explicit | 7.0/7.0 | 792.0/792.0 | 11918.7857 | 12047.03505 | 1.076027 | 12037.11295 | 12138.18475 | 0.839668 | True |
| g5-freeq-small-default | 7.0/7.0 | 1208.0/1208.0 | 12465.8141 | 12449.5738 | -0.130279 | 12465.6288 | 12279.63575 | -1.492047 | False |
| g5-freeq-small-delayed | 6.0/6.0 | 952.0/952.0 | 11235.736 | 11177.7897 | -0.515732 | 11252.5141 | 11319.7984 | 0.597949 | False |
| g5-freeq-small-explicit | 8.0/8.0 | 1240.0/1240.0 | 13210.56075 | 13026.2358 | -1.395285 | 13164.4295 | 13039.63605 | -0.947959 | False |
| g5-position-small-default | 17.0/17.0 | 2136.0/2136.0 | 19351.599 | 19315.04475 | -0.188895 | 19394.61485 | 19314.914 | -0.410943 | False |
| g5-position-small-delayed | 16.0/16.0 | 1880.0/1880.0 | 18246.1295 | 18129.3322 | -0.640121 | 18186.24875 | 18129.76255 | -0.310598 | False |
| g5-position-small-explicit | 18.0/18.0 | 2168.0/2168.0 | 19911.7294 | 19965.7093 | 0.271096 | 19969.74215 | 19946.372 | -0.117028 | False |
| g5-deletecases-small-default | 9.0/9.0 | 984.0/984.0 | 14554.46245 | 14536.3989 | -0.124110 | 14613.92655 | 14528.4276 | -0.585051 | False |
| g5-deletecases-small-delayed | 8.0/8.0 | 728.0/728.0 | 13351.3864 | 13358.02135 | 0.049695 | 13362.2466 | 13388.23435 | 0.194486 | False |
| g5-deletecases-small-explicit | 10.0/10.0 | 1048.0/1048.0 | 15243.5197 | 15289.25325 | 0.300020 | 15309.6661 | 15229.23925 | -0.525334 | False |
| g5-replace-small-default | 20.0/20.0 | 1945.0/1945.0 | 20214.4662 | 20388.8853 | 0.862843 | 20134.3876 | 20214.233 | 0.396562 | True |
| g5-replace-small-delayed | 19.0/19.0 | 1689.0/1689.0 | 19065.4142 | 19050.2059 | -0.079769 | 19072.78065 | 19065.042 | -0.040574 | False |
| g5-replace-small-explicit | 22.0/22.0 | 2233.0/2233.0 | 21267.7592 | 21267.3513 | -0.001918 | 21268.74565 | 21268.4557 | -0.001363 | False |
| g5-level-small-default | 13.0/13.0 | 1009.0/1009.0 | 13537.12795 | 13349.1723 | -1.388446 | 13328.9287 | 13348.52925 | 0.147053 | False |
| g5-level-small-delayed | 12.0/12.0 | 753.0/753.0 | 12170.53175 | 12192.4411 | 0.180020 | 12219.79665 | 12179.1853 | -0.332341 | False |
| g5-level-small-explicit | 13.0/13.0 | 1009.0/1009.0 | 14228.12285 | 14197.94035 | -0.212133 | 14251.2462 | 14111.6281 | -0.979690 | False |
| g5-depth-small-default | 4.0/4.0 | 392.0/392.0 | 8804.37955 | 8804.3357 | -0.000498 | 8815.375 | 8948.33075 | 1.508226 | False |
| g5-depth-small-delayed | 3.0/3.0 | 136.0/136.0 | 7599.62285 | 7632.5741 | 0.433591 | 7632.4739 | 7630.4719 | -0.026230 | False |
| g5-depth-small-explicit | 4.0/4.0 | 392.0/392.0 | 9717.7461 | 9652.6962 | -0.669393 | 9665.6818 | 9678.546 | 0.133091 | False |
| g5-trace-small-default | 33.0/33.0 | 1992.0/1992.0 | 27527.3908 | 27566.69575 | 0.142785 | 27525.7627 | 27531.8465 | 0.022102 | False |
| g5-trace-small-delayed | 32.0/32.0 | 1736.0/1736.0 | 26251.99415 | 26353.24865 | 0.385702 | 26300.09355 | 26344.5646 | 0.169091 | False |
| g5-trace-small-explicit | 36.0/36.0 | 2120.0/2120.0 | 31800.07775 | 31693.9741 | -0.333658 | 31724.23645 | 31689.4816 | -0.109553 | False |
| g5-tracedetail-small-default | 55.0/55.0 | 3263.0/3263.0 | 36136.6434 | 36063.86995 | -0.201384 | 36062.999 | 36063.7896 | 0.002192 | False |
| g5-tracedetail-small-delayed | 54.0/54.0 | 3007.0/3007.0 | 34883.21385 | 35000.7047 | 0.336812 | 34895.0424 | 34877.4236 | -0.050491 | False |
| g5-tracedetail-small-explicit | 57.0/57.0 | 3359.0/3359.0 | 38969.69905 | 39040.4662 | 0.181595 | 38987.8904 | 39106.8917 | 0.305226 | False |
| g5-traceprint-small-default | 40.0/40.0 | 2174.0/2174.0 | 30775.609 | 30818.396 | 0.139029 | 30653.2255 | 30674.169 | 0.068324 | False |
| g5-traceprint-small-delayed | 39.0/39.0 | 1918.0/1918.0 | 29398.667 | 29434.9815 | 0.123524 | 29391.7725 | 29392.4065 | 0.002157 | False |
| g5-traceprint-small-explicit | 43.0/43.0 | 2302.0/2302.0 | 34855.478 | 34820.7645 | -0.099593 | 35017.483 | 34958.1775 | -0.169360 | False |
| matchq-integer-success | 1.0/1.0 | 128.0/128.0 | 4340.2012 | 4340.3217 | 0.002776 | 4340.1713 | 4340.2138 | 0.000979 | False |
| matchq-integer-fail | 2.0/2.0 | 129.0/129.0 | 4462.2387 | 4462.176 | -0.001405 | 4462.3233 | 4462.1388 | -0.004135 | False |
| cases-wide-no-match | 33.001/33.001 | 263247.864/263247.864 | 2400534.564 | 2400767.094 | 0.009687 | 2400501.909 | 2400529.622 | 0.001154 | False |
| count-wide-no-match | 30.0/30.0 | 262601.0/262601.0 | 2178888.939 | 2178888.657 | -0.000013 | 2178887.288 | 2178893.691 | 0.000294 | False |
| position-wide-no-match | 1642.001/1642.001 | 576255.864/576255.864 | 2573285.525 | 2573210.525 | -0.002915 | 2573241.154 | 2573220.11 | -0.000818 | False |
| deletecases-wide-no-match | 24.001/24.001 | 247447.864/247447.864 | 2670167.931 | 2670277.407 | 0.004100 | 2670168.545 | 2670189.442 | 0.000783 | False |
| replace-wide-no-match | 30.001/30.001 | 184455.864/184455.864 | 1163626.301 | 1163601.895 | -0.002097 | 1163577.565 | 1163603.199 | 0.002203 | False |
| dot-three-no-input | 596.0/596.0 | 23472.0/23472.0 | 253101.84195 | 253105.7973 | 0.001563 | 253105.4713 | 253102.0399 | -0.001356 | False |
| differences-no-input | 13.0/13.0 | 672.0/672.0 | 9411.02125 | 9398.82935 | -0.129549 | 9403.5812 | 9398.7747 | -0.051114 | False |
| take-multispec-small | 109.0/109.0 | 5824.0/5824.0 | 49240.6124 | 49230.9041 | -0.019716 | 49225.13785 | 49230.76435 | 0.011430 | False |
| g5-monitor-options-none | 9.0/9.0 | 953.0/953.0 | 18097.31 | 18173.6395 | 0.421773 | 18150.7695 | 18099.185 | -0.284200 | False |
| g5-monitor-explicit-held | 13.0005/13.0005 | 1727.856/1727.856 | 17723.301 | 17802.809 | 0.448607 | 17750.706 | 17824.999 | 0.418535 | True |
| g5-names-default | 6253.005/6253.005 | 552512.36/552512.36 | 3482226.545 | 3482576.135 | 0.010039 | 3483249.47 | 3482372.775 | -0.025169 | False |
| g5-testreport-options-default | 54.0/54.0 | 5609.0/5609.0 | 42876.2485 | 43295.139 | 0.976976 | 42939.408 | 43354.463 | 0.966606 | True |
| g5-testreport-unread-default | 41.0005/41.0005 | 4598.4/4598.4 | 28786.747 | 28768.5135 | -0.063340 | 28788.658 | 28825.7135 | 0.128716 | False |
| g5-monitor-evaluationmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13448.1945 | 13395.472 | -0.392041 | 13352.1765 | 13395.95 | 0.327838 | False |
| g5-monitor-stepmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13353.1705 | 13450.8085 | 0.731197 | 13474.145 | 13391.49 | -0.613434 | False |
| g5-names-spellingcorrection-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473121.405 | 3473459.55 | 0.009736 | 3473227.035 | 3473442.055 | 0.006191 | False |
| g5-names-resolvecontextaliases-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473444.84 | 3473401.24 | -0.001255 | 3473238.565 | 3473413.66 | 0.005041 | False |
| g5-names-ignorecase-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3470622.01 | 3470790.745 | 0.004862 | 3470562.59 | 3470592.875 | 0.000873 | False |
| g5-testreport-sametest-latent | 52.0/52.0 | 4969.0/4969.0 | 27581.6265 | 27740.0995 | 0.574560 | 27583.1345 | 27728.5705 | 0.527264 | True |
| g5-testreport-timeconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27596.3405 | 27803.986 | 0.752439 | 27621.6305 | 27757.146 | 0.490614 | True |
| g5-testreport-memoryconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27603.3455 | 27762.3965 | 0.576202 | 27589.0765 | 27764.453 | 0.635674 | True |

#### candidate-06

[Hashes/build identity](../../../.temp/brief-578/candidate-06/hashes.json); [result checks](../../../.temp/brief-578/candidate-06/perf/checks.json); [raw counts/streams/status](../../../.temp/brief-578/candidate-06/perf/paired-results.json); [full precision TSV](../../../.temp/brief-578/candidate-06/perf/slopes.tsv).

| Label | Alloc base/new | Bytes base/new | I base1 | I new1 | Δ1 % | I base2 | I new2 | Δ2 % | Regresses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 9.0/9.0 | 912.0/912.0 | 15928.7026 | 15883.96315 | -0.280873 | 15977.1787 | 15992.7931 | 0.097729 | False |
| g5-cases-small-delayed | 8.0/8.0 | 656.0/656.0 | 14828.46085 | 14864.35685 | 0.242075 | 14792.5525 | 14784.55835 | -0.054042 | False |
| g5-cases-small-explicit | 11.0/11.0 | 1104.0/1104.0 | 16967.6895 | 16934.33445 | -0.196580 | 16946.697 | 16933.23075 | -0.079462 | False |
| g5-count-small-default | 6.0/6.0 | 664.0/664.0 | 13135.53795 | 13122.8352 | -0.096705 | 13161.40665 | 13221.59425 | 0.457304 | False |
| g5-count-small-delayed | 5.0/5.0 | 408.0/408.0 | 11988.9279 | 12006.7966 | 0.149043 | 11988.8074 | 11967.93655 | -0.174086 | False |
| g5-count-small-explicit | 8.0/8.0 | 824.0/824.0 | 13941.39985 | 13967.63665 | 0.188193 | 14019.4097 | 14027.4098 | 0.057064 | False |
| g5-memberq-small-default | 6.0/6.0 | 760.0/760.0 | 11450.35845 | 11530.3432 | 0.698535 | 11420.36555 | 11434.35505 | 0.122496 | False |
| g5-memberq-small-delayed | 5.0/5.0 | 504.0/504.0 | 10307.24485 | 10334.12245 | 0.260764 | 10305.1772 | 10316.22595 | 0.107216 | False |
| g5-memberq-small-explicit | 7.0/7.0 | 792.0/792.0 | 12036.99615 | 12007.9463 | -0.241338 | 12007.04835 | 12087.99955 | 0.674197 | False |
| g5-freeq-small-default | 7.0/7.0 | 1208.0/1208.0 | 12546.68245 | 12449.6558 | -0.773325 | 12465.63595 | 12449.7271 | -0.127622 | False |
| g5-freeq-small-delayed | 6.0/6.0 | 952.0/952.0 | 11222.8288 | 11201.69415 | -0.188318 | 11207.78425 | 11186.78915 | -0.187326 | False |
| g5-freeq-small-explicit | 8.0/8.0 | 1240.0/1240.0 | 13042.47475 | 13026.34835 | -0.123645 | 13042.31165 | 13052.449 | 0.077727 | False |
| g5-position-small-default | 17.0/17.0 | 2136.0/2136.0 | 19351.6918 | 19291.1916 | -0.312635 | 19300.1806 | 19411.1966 | 0.575207 | False |
| g5-position-small-delayed | 16.0/16.0 | 1880.0/1880.0 | 18114.24 | 18134.3674 | 0.111114 | 18128.4312 | 18106.2148 | -0.122550 | False |
| g5-position-small-explicit | 18.0/18.0 | 2168.0/2168.0 | 19879.92545 | 19856.5864 | -0.117400 | 19873.6466 | 19958.57255 | 0.427329 | False |
| g5-deletecases-small-default | 9.0/9.0 | 984.0/984.0 | 14617.77595 | 14497.9884 | -0.819465 | 14559.93845 | 14498.18535 | -0.424130 | False |
| g5-deletecases-small-delayed | 8.0/8.0 | 728.0/728.0 | 13364.6264 | 13438.14705 | 0.550114 | 13358.3358 | 13390.30395 | 0.239312 | True |
| g5-deletecases-small-explicit | 10.0/10.0 | 1048.0/1048.0 | 15264.5253 | 15184.003 | -0.527513 | 15175.77705 | 15345.2382 | 1.116656 | False |
| g5-replace-small-default | 20.0/20.0 | 1945.0/1945.0 | 20314.04945 | 20350.5499 | 0.179681 | 20242.5529 | 20214.38045 | -0.139174 | False |
| g5-replace-small-delayed | 19.0/19.0 | 1689.0/1689.0 | 19072.7679 | 19065.1724 | -0.039824 | 19052.33415 | 19066.9686 | 0.076812 | False |
| g5-replace-small-explicit | 22.0/22.0 | 2233.0/2233.0 | 21327.62475 | 21267.8177 | -0.280421 | 21267.75025 | 21182.83035 | -0.399290 | False |
| g5-level-small-default | 13.0/13.0 | 1009.0/1009.0 | 13243.45125 | 13444.9088 | 1.521186 | 13443.70915 | 13349.521 | -0.700611 | False |
| g5-level-small-delayed | 12.0/12.0 | 753.0/753.0 | 12187.66955 | 12181.86865 | -0.047596 | 12167.19345 | 12181.1375 | 0.114604 | False |
| g5-level-small-explicit | 13.0/13.0 | 1009.0/1009.0 | 14252.305 | 14227.6778 | -0.172795 | 14324.89545 | 14228.69955 | -0.671530 | False |
| g5-depth-small-default | 4.0/4.0 | 392.0/392.0 | 9100.59105 | 8804.36285 | -3.255044 | 8678.46845 | 8852.53545 | 2.005734 | False |
| g5-depth-small-delayed | 3.0/3.0 | 136.0/136.0 | 7656.71465 | 7600.41975 | -0.735236 | 7643.48835 | 7660.7418 | 0.225727 | False |
| g5-depth-small-explicit | 4.0/4.0 | 392.0/392.0 | 9637.52925 | 9751.63805 | 1.184005 | 9717.63205 | 9728.6752 | 0.113640 | False |
| g5-trace-small-default | 33.0/33.0 | 1992.0/1992.0 | 27526.86525 | 27468.92235 | -0.210496 | 27488.74835 | 27528.80935 | 0.145736 | False |
| g5-trace-small-delayed | 32.0/32.0 | 1736.0/1736.0 | 26437.4155 | 26306.08575 | -0.496757 | 26300.0879 | 26353.48185 | 0.203018 | False |
| g5-trace-small-explicit | 36.0/36.0 | 2120.0/2120.0 | 31748.112 | 31754.83235 | 0.021168 | 31748.493 | 31822.85655 | 0.234227 | False |
| g5-tracedetail-small-default | 55.0/55.0 | 3263.0/3263.0 | 36080.9083 | 36064.1868 | -0.046344 | 36052.2768 | 36130.07775 | 0.215800 | False |
| g5-tracedetail-small-delayed | 54.0/54.0 | 3007.0/3007.0 | 34936.7238 | 34822.08165 | -0.328142 | 34900.32585 | 34811.1359 | -0.255556 | False |
| g5-tracedetail-small-explicit | 57.0/57.0 | 3359.0/3359.0 | 38911.27365 | 38933.5271 | 0.057190 | 39065.9657 | 39083.79465 | 0.045638 | False |
| g5-traceprint-small-default | 40.0/40.0 | 2174.0/2174.0 | 30689.8815 | 30669.1345 | -0.067602 | 30653.5125 | 30702.491 | 0.159781 | False |
| g5-traceprint-small-delayed | 39.0/39.0 | 1918.0/1918.0 | 29480.2155 | 29372.272 | -0.366156 | 29420.354 | 29435.461 | 0.051349 | False |
| g5-traceprint-small-explicit | 43.0/43.0 | 2302.0/2302.0 | 34820.4325 | 34807.4325 | -0.037334 | 34703.177 | 34724.395 | 0.061141 | False |
| matchq-integer-success | 1.0/1.0 | 128.0/128.0 | 4340.1616 | 4340.18865 | 0.000623 | 4340.2244 | 4340.3279 | 0.002385 | False |
| matchq-integer-fail | 2.0/2.0 | 129.0/129.0 | 4462.04305 | 4462.1218 | 0.001765 | 4462.23385 | 4462.1375 | -0.002159 | False |
| cases-wide-no-match | 33.001/33.001 | 263247.864/263247.864 | 2400534.088 | 2400529.741 | -0.000181 | 2400533.633 | 2400528.533 | -0.000212 | False |
| count-wide-no-match | 30.0/30.0 | 262601.0/262601.0 | 2178891.984 | 2178844.493 | -0.002180 | 2178830.927 | 2178890.447 | 0.002732 | False |
| position-wide-no-match | 1642.001/1642.001 | 576255.864/576255.864 | 2573271.882 | 2573330.414 | 0.002275 | 2573333.325 | 2573272.342 | -0.002370 | False |
| deletecases-wide-no-match | 24.001/24.001 | 247447.864/247447.864 | 2670168.993 | 2670168.09 | -0.000034 | 2670164.016 | 2670033.265 | -0.004897 | False |
| replace-wide-no-match | 30.001/30.001 | 184455.864/184455.864 | 1163696.502 | 1163609.083 | -0.007512 | 1163499.242 | 1163702.193 | 0.017443 | False |
| dot-three-no-input | 596.0/596.0 | 23472.0/23472.0 | 253105.4868 | 253105.6904 | 0.000080 | 253105.5505 | 253102.79235 | -0.001090 | False |
| differences-no-input | 13.0/13.0 | 672.0/672.0 | 9408.91205 | 9399.39515 | -0.101148 | 9415.9374 | 9399.3922 | -0.175715 | False |
| take-multispec-small | 109.0/109.0 | 5824.0/5824.0 | 49239.46425 | 49231.0054 | -0.017179 | 49247.73335 | 49230.88855 | -0.034204 | False |
| g5-monitor-options-none | 9.0/9.0 | 953.0/953.0 | 18127.882 | 18147.4045 | 0.107693 | 18021.2655 | 18147.368 | 0.699743 | False |
| g5-monitor-explicit-held | 13.0005/13.0005 | 1727.856/1727.856 | 17728.007 | 17908.9435 | 1.020625 | 17719.143 | 17904.8295 | 1.047943 | True |
| g5-names-default | 6253.005/6253.005 | 552512.36/552512.36 | 3482363.62 | 3482450.14 | 0.002485 | 3482381.69 | 3482591.855 | 0.006035 | False |
| g5-testreport-options-default | 54.0/54.0 | 5609.0/5609.0 | 43133.9345 | 42768.164 | -0.847988 | 42980.261 | 42990.514 | 0.023855 | False |
| g5-testreport-unread-default | 41.0005/41.0005 | 4598.4/4598.4 | 28792.702 | 28877.02 | 0.292845 | 28790.4145 | 28830.4255 | 0.138973 | True |
| g5-monitor-evaluationmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13352.0275 | 13393.105 | 0.307650 | 13351.7925 | 13393.8975 | 0.315351 | False |
| g5-monitor-stepmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13355.2755 | 13391.301 | 0.269747 | 13352.356 | 13389.557 | 0.278610 | True |
| g5-names-spellingcorrection-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473129.09 | 3473422.815 | 0.008457 | 3473103.65 | 3473574.24 | 0.013550 | False |
| g5-names-resolvecontextaliases-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473012.27 | 3473442.395 | 0.012385 | 3473268.745 | 3473379.02 | 0.003175 | False |
| g5-names-ignorecase-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3470415.375 | 3470935.72 | 0.014994 | 3470812.5 | 3470721.995 | -0.002608 | False |
| g5-testreport-sametest-latent | 52.0/52.0 | 4969.0/4969.0 | 27567.3745 | 27596.569 | 0.105902 | 27579.0265 | 27680.7685 | 0.368911 | False |
| g5-testreport-timeconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27594.59 | 27683.127 | 0.320849 | 27595.56 | 27721.001 | 0.454570 | True |
| g5-testreport-memoryconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27602.9375 | 27687.711 | 0.307118 | 27602.06 | 27702.632 | 0.364364 | True |

#### candidate-07

[Hashes/build identity](../../../.temp/brief-578/candidate-07/hashes.json); [result checks](../../../.temp/brief-578/candidate-07/perf/checks.json); [raw counts/streams/status](../../../.temp/brief-578/candidate-07/perf/paired-results.json); [full precision TSV](../../../.temp/brief-578/candidate-07/perf/slopes.tsv).

| Label | Alloc base/new | Bytes base/new | I base1 | I new1 | Δ1 % | I base2 | I new2 | Δ2 % | Regresses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 9.0/9.0 | 912.0/912.0 | 15929.15165 | 15949.2903 | 0.126426 | 15989.73265 | 15918.89345 | -0.443029 | False |
| g5-cases-small-delayed | 8.0/8.0 | 656.0/656.0 | 14790.2916 | 14783.11065 | -0.048552 | 14775.3664 | 14765.8058 | -0.064706 | False |
| g5-cases-small-explicit | 11.0/11.0 | 1104.0/1104.0 | 17041.9098 | 16933.4089 | -0.636671 | 16927.22255 | 16885.5773 | -0.246025 | False |
| g5-count-small-default | 6.0/6.0 | 664.0/664.0 | 13182.5322 | 13196.63465 | 0.106978 | 13118.64275 | 13133.5387 | 0.113548 | False |
| g5-count-small-delayed | 5.0/5.0 | 408.0/408.0 | 11973.72915 | 11986.971 | 0.110591 | 11988.8429 | 11968.0691 | -0.173276 | False |
| g5-count-small-explicit | 8.0/8.0 | 824.0/824.0 | 13993.4678 | 14015.5314 | 0.157671 | 14001.3701 | 14043.4363 | 0.300443 | False |
| g5-memberq-small-default | 6.0/6.0 | 760.0/760.0 | 11450.28365 | 11488.2749 | 0.331793 | 11495.3145 | 11434.31625 | -0.530636 | False |
| g5-memberq-small-delayed | 5.0/5.0 | 504.0/504.0 | 10307.305 | 10219.15935 | -0.855176 | 10314.03855 | 10286.2528 | -0.269397 | False |
| g5-memberq-small-explicit | 7.0/7.0 | 792.0/792.0 | 12037.0296 | 12093.9594 | 0.472956 | 12072.33595 | 12008.0558 | -0.532458 | False |
| g5-freeq-small-default | 7.0/7.0 | 1208.0/1208.0 | 12465.66205 | 12359.731 | -0.849783 | 12465.623 | 12331.73075 | -1.074092 | False |
| g5-freeq-small-delayed | 6.0/6.0 | 952.0/952.0 | 11196.7861 | 11201.68215 | 0.043727 | 11222.7709 | 11201.6263 | -0.188408 | False |
| g5-freeq-small-explicit | 8.0/8.0 | 1240.0/1240.0 | 13029.3377 | 13052.40775 | 0.177062 | 13090.33295 | 13013.5055 | -0.586902 | False |
| g5-position-small-default | 17.0/17.0 | 2136.0/2136.0 | 19351.6524 | 19345.12155 | -0.033748 | 19334.54605 | 19315.18305 | -0.100147 | False |
| g5-position-small-delayed | 16.0/16.0 | 1880.0/1880.0 | 18124.13995 | 18145.13495 | 0.115840 | 18139.17415 | 18129.9551 | -0.050824 | False |
| g5-position-small-explicit | 18.0/18.0 | 2168.0/2168.0 | 20065.6813 | 19898.7101 | -0.832123 | 19999.72715 | 19899.29185 | -0.502183 | False |
| g5-deletecases-small-default | 9.0/9.0 | 984.0/984.0 | 14568.81825 | 14450.3804 | -0.812954 | 14509.3114 | 14584.32915 | 0.517032 | False |
| g5-deletecases-small-delayed | 8.0/8.0 | 728.0/728.0 | 13370.42285 | 13358.01145 | -0.092827 | 13329.38505 | 13426.0608 | 0.725283 | False |
| g5-deletecases-small-explicit | 10.0/10.0 | 1048.0/1048.0 | 15253.8065 | 15277.226 | 0.153532 | 15239.77375 | 15259.01475 | 0.126255 | False |
| g5-replace-small-default | 20.0/20.0 | 1945.0/1945.0 | 20342.7373 | 20214.5108 | -0.630331 | 20258.41445 | 20214.7881 | -0.215349 | False |
| g5-replace-small-delayed | 19.0/19.0 | 1689.0/1689.0 | 19046.42615 | 19065.96585 | 0.102590 | 19058.1855 | 19065.2911 | 0.037284 | False |
| g5-replace-small-explicit | 22.0/22.0 | 2233.0/2233.0 | 21267.822 | 21209.53455 | -0.274064 | 21243.85425 | 21314.67205 | 0.333357 | False |
| g5-level-small-default | 13.0/13.0 | 1009.0/1009.0 | 13323.42245 | 13292.6067 | -0.231290 | 13409.26335 | 13349.2728 | -0.447381 | False |
| g5-level-small-delayed | 12.0/12.0 | 753.0/753.0 | 12115.12225 | 12299.1821 | 1.519257 | 12187.96075 | 12229.4376 | 0.340310 | False |
| g5-level-small-explicit | 13.0/13.0 | 1009.0/1009.0 | 14326.91705 | 14228.39915 | -0.687642 | 14204.08045 | 14228.65665 | 0.173022 | False |
| g5-depth-small-default | 4.0/4.0 | 392.0/392.0 | 8779.3514 | 8745.22995 | -0.388656 | 8735.10595 | 8852.52695 | 1.344242 | False |
| g5-depth-small-delayed | 3.0/3.0 | 136.0/136.0 | 7691.6009 | 7676.47095 | -0.196707 | 7643.5856 | 7617.55785 | -0.340518 | False |
| g5-depth-small-explicit | 4.0/4.0 | 392.0/392.0 | 9652.68365 | 9665.7955 | 0.135836 | 9663.69965 | 9691.69965 | 0.289744 | False |
| g5-trace-small-default | 33.0/33.0 | 1992.0/1992.0 | 27718.65375 | 27567.5583 | -0.545104 | 27527.2418 | 27529.0792 | 0.006675 | False |
| g5-trace-small-delayed | 32.0/32.0 | 1736.0/1736.0 | 26326.5466 | 26305.517 | -0.079880 | 26300.1094 | 26304.4996 | 0.016693 | False |
| g5-trace-small-explicit | 36.0/36.0 | 2120.0/2120.0 | 31740.92555 | 31806.1428 | 0.205467 | 31788.9423 | 31719.7367 | -0.217703 | False |
| g5-tracedetail-small-default | 55.0/55.0 | 3263.0/3263.0 | 36065.1192 | 36022.3454 | -0.118602 | 36208.1908 | 36064.39565 | -0.397134 | False |
| g5-tracedetail-small-delayed | 54.0/54.0 | 3007.0/3007.0 | 35105.3209 | 34976.06675 | -0.368190 | 34955.0699 | 34855.63065 | -0.284477 | False |
| g5-tracedetail-small-explicit | 57.0/57.0 | 3359.0/3359.0 | 39127.1479 | 38951.0008 | -0.450192 | 39023.0748 | 38986.7788 | -0.093012 | False |
| g5-traceprint-small-default | 40.0/40.0 | 2174.0/2174.0 | 30640.941 | 30508.61 | -0.431876 | 30617.781 | 30690.053 | 0.236046 | False |
| g5-traceprint-small-delayed | 39.0/39.0 | 1918.0/1918.0 | 29422.065 | 29408.6815 | -0.045488 | 29396.8735 | 29393.373 | -0.011908 | False |
| g5-traceprint-small-explicit | 43.0/43.0 | 2302.0/2302.0 | 34819.8965 | 34797.909 | -0.063146 | 34868.699 | 34820.2225 | -0.139026 | False |
| matchq-integer-success | 1.0/1.0 | 128.0/128.0 | 4340.1015 | 4340.28585 | 0.004248 | 4340.153 | 4340.22605 | 0.001683 | False |
| matchq-integer-fail | 2.0/2.0 | 129.0/129.0 | 4462.2536 | 4462.04245 | -0.004732 | 4462.3296 | 4462.13735 | -0.004308 | False |
| cases-wide-no-match | 33.001/33.001 | 263247.864/263247.864 | 2400512.841 | 2400501.387 | -0.000477 | 2400531.563 | 2400530.908 | -0.000027 | False |
| count-wide-no-match | 30.0/30.0 | 262601.0/262601.0 | 2178889.357 | 2178936.232 | 0.002151 | 2178891.392 | 2178935.57 | 0.002028 | False |
| position-wide-no-match | 1642.001/1642.001 | 576255.864/576255.864 | 2573270.74 | 2573268.888 | -0.000072 | 2573273.507 | 2573271.32 | -0.000085 | False |
| deletecases-wide-no-match | 24.001/24.001 | 247447.864/247447.864 | 2670172.134 | 2670441.432 | 0.010085 | 2670059.788 | 2670140.705 | 0.003031 | False |
| replace-wide-no-match | 30.001/30.001 | 184455.864/184455.864 | 1163649.711 | 1163659.594 | 0.000849 | 1163597.311 | 1163549.17 | -0.004137 | False |
| dot-three-no-input | 596.0/596.0 | 23472.0/23472.0 | 253105.64765 | 253104.38415 | -0.000499 | 253101.34245 | 253101.6929 | 0.000138 | False |
| differences-no-input | 13.0/13.0 | 672.0/672.0 | 9416.7711 | 9400.0069 | -0.178025 | 9397.09585 | 9399.68095 | 0.027510 | False |
| take-multispec-small | 109.0/109.0 | 5824.0/5824.0 | 49240.35705 | 49231.2085 | -0.018579 | 49232.51775 | 49230.98525 | -0.003113 | False |
| g5-monitor-options-none | 9.0/9.0 | 953.0/953.0 | 18049.7045 | 18134.3435 | 0.468922 | 18174.1305 | 18052.9595 | -0.666722 | False |
| g5-monitor-explicit-held | 13.0005/13.0005 | 1727.856/1727.856 | 17703.5235 | 17920.923 | 1.228001 | 17775.759 | 17837.942 | 0.349819 | True |
| g5-names-default | 6253.005/6253.005 | 552512.36/552512.36 | 3482311.285 | 3483136.545 | 0.023699 | 3482414.485 | 3482403.56 | -0.000314 | False |
| g5-testreport-options-default | 54.0/54.0 | 5609.0/5609.0 | 43033.1375 | 42960.869 | -0.167937 | 42956.6735 | 42967.8245 | 0.025959 | False |
| g5-testreport-unread-default | 41.0005/41.0005 | 4598.4/4598.4 | 28788.1865 | 28885.9015 | 0.339427 | 28791.1805 | 28883.162 | 0.319478 | True |
| g5-monitor-evaluationmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13349.9995 | 13412.808 | 0.470476 | 13350.056 | 13361.7955 | 0.087936 | False |
| g5-monitor-stepmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13352.5585 | 13408.467 | 0.418710 | 13352.4955 | 13408.5845 | 0.420064 | True |
| g5-names-spellingcorrection-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473307.26 | 3473510.195 | 0.005843 | 3473114.065 | 3473302.485 | 0.005425 | False |
| g5-names-resolvecontextaliases-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473284.385 | 3473339.36 | 0.001583 | 3473178.815 | 3473510.58 | 0.009552 | False |
| g5-names-ignorecase-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3470967.555 | 3471060.685 | 0.002683 | 3470617.925 | 3470538.955 | -0.002275 | False |
| g5-testreport-sametest-latent | 52.0/52.0 | 4969.0/4969.0 | 27705.008 | 27683.8765 | -0.076273 | 27568.1645 | 27669.1215 | 0.366209 | False |
| g5-testreport-timeconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27594.8455 | 27680.1935 | 0.309290 | 27596.4835 | 27673.878 | 0.280451 | True |
| g5-testreport-memoryconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27600.955 | 27703.2595 | 0.370656 | 27474.873 | 27703.4105 | 0.831805 | True |

#### candidate-08

[Hashes/build identity](../../../.temp/brief-578/candidate-08/hashes.json); [result checks](../../../.temp/brief-578/candidate-08/perf/checks.json); [raw counts/streams/status](../../../.temp/brief-578/candidate-08/perf/paired-results.json); [full precision TSV](../../../.temp/brief-578/candidate-08/perf/slopes.tsv).

| Label | Alloc base/new | Bytes base/new | I base1 | I new1 | Δ1 % | I base2 | I new2 | Δ2 % | Regresses |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| g5-cases-small-default | 9.0/9.0 | 912.0/912.0 | 15929.0221 | 15909.3299 | -0.123625 | 15929.1479 | 16000.5284 | 0.448112 | False |
| g5-cases-small-delayed | 8.0/8.0 | 656.0/656.0 | 14788.1024 | 14723.98895 | -0.433548 | 14838.2284 | 14784.55485 | -0.361725 | False |
| g5-cases-small-explicit | 11.0/11.0 | 1104.0/1104.0 | 16941.0491 | 16941.4788 | 0.002536 | 16988.53715 | 16941.615 | -0.276199 | False |
| g5-count-small-default | 6.0/6.0 | 664.0/664.0 | 13135.63905 | 13156.74145 | 0.160650 | 13135.7894 | 13139.64215 | 0.029330 | False |
| g5-count-small-delayed | 5.0/5.0 | 408.0/408.0 | 11988.9671 | 11993.97115 | 0.041739 | 11951.86015 | 11892.89745 | -0.493335 | False |
| g5-count-small-explicit | 8.0/8.0 | 824.0/824.0 | 13954.49795 | 14023.56165 | 0.494921 | 13967.47935 | 13975.3333 | 0.056230 | False |
| g5-memberq-small-default | 6.0/6.0 | 760.0/760.0 | 11450.12495 | 11442.4238 | -0.067258 | 11450.35045 | 11354.15035 | -0.840150 | False |
| g5-memberq-small-delayed | 5.0/5.0 | 504.0/504.0 | 10368.3232 | 10334.29845 | -0.328161 | 10307.2372 | 10324.1289 | 0.163882 | False |
| g5-memberq-small-explicit | 7.0/7.0 | 792.0/792.0 | 12037.01635 | 12042.1146 | 0.042355 | 12133.12125 | 12055.08965 | -0.643129 | False |
| g5-freeq-small-default | 7.0/7.0 | 1208.0/1208.0 | 12525.57 | 12373.6696 | -1.212722 | 12441.62315 | 12505.6082 | 0.514282 | False |
| g5-freeq-small-delayed | 6.0/6.0 | 952.0/952.0 | 11194.5885 | 11286.4937 | 0.820979 | 11235.70025 | 11239.7879 | 0.036381 | False |
| g5-freeq-small-explicit | 8.0/8.0 | 1240.0/1240.0 | 12845.3875 | 13034.40535 | 1.471484 | 13042.472 | 13021.2717 | -0.162548 | False |
| g5-position-small-default | 17.0/17.0 | 2136.0/2136.0 | 19358.32465 | 19362.9483 | 0.023885 | 19397.748 | 19315.04665 | -0.426345 | False |
| g5-position-small-delayed | 16.0/16.0 | 1880.0/1880.0 | 18127.2951 | 18129.1006 | 0.009960 | 18139.31165 | 18092.2279 | -0.259567 | False |
| g5-position-small-explicit | 18.0/18.0 | 2168.0/2168.0 | 19880.729 | 19928.0647 | 0.238098 | 19879.73525 | 19921.24845 | 0.208822 | True |
| g5-deletecases-small-default | 9.0/9.0 | 984.0/984.0 | 14585.80615 | 14447.4409 | -0.948629 | 14517.684 | 14566.64845 | 0.337275 | False |
| g5-deletecases-small-delayed | 8.0/8.0 | 728.0/728.0 | 13363.71945 | 13360.0169 | -0.027706 | 13374.8887 | 13358.05215 | -0.125882 | False |
| g5-deletecases-small-explicit | 10.0/10.0 | 1048.0/1048.0 | 15412.909 | 15237.19625 | -1.140036 | 15323.71785 | 15188.9253 | -0.879633 | False |
| g5-replace-small-default | 20.0/20.0 | 1945.0/1945.0 | 20214.29205 | 20348.21945 | 0.662538 | 20226.4202 | 20260.729 | 0.169624 | False |
| g5-replace-small-delayed | 19.0/19.0 | 1689.0/1689.0 | 19058.02345 | 19072.41715 | 0.075526 | 19088.06135 | 19097.17045 | 0.047721 | False |
| g5-replace-small-explicit | 22.0/22.0 | 2233.0/2233.0 | 21237.8117 | 21330.98085 | 0.438695 | 21149.8056 | 21282.00375 | 0.625056 | False |
| g5-level-small-default | 13.0/13.0 | 1009.0/1009.0 | 13327.8238 | 13357.1669 | 0.220164 | 13313.9294 | 13331.28525 | 0.130359 | False |
| g5-level-small-delayed | 12.0/12.0 | 753.0/753.0 | 12164.4091 | 12161.4098 | -0.024656 | 12222.03235 | 12178.79365 | -0.353777 | False |
| g5-level-small-explicit | 13.0/13.0 | 1009.0/1009.0 | 14251.3477 | 14236.60705 | -0.103433 | 14252.17725 | 14236.67195 | -0.108793 | False |
| g5-depth-small-default | 4.0/4.0 | 392.0/392.0 | 8880.29615 | 8762.24135 | -1.329402 | 8791.34165 | 8786.2293 | -0.058152 | False |
| g5-depth-small-delayed | 3.0/3.0 | 136.0/136.0 | 7617.59635 | 7660.69265 | 0.565747 | 7643.54245 | 7675.6239 | 0.419720 | False |
| g5-depth-small-explicit | 4.0/4.0 | 392.0/392.0 | 9717.57665 | 9758.77285 | 0.423935 | 9678.73255 | 9751.74045 | 0.754313 | False |
| g5-trace-small-default | 33.0/33.0 | 1992.0/1992.0 | 27682.21255 | 27656.24105 | -0.093820 | 27649.71465 | 27538.97945 | -0.400493 | False |
| g5-trace-small-delayed | 32.0/32.0 | 1736.0/1736.0 | 26300.852 | 26280.0817 | -0.078972 | 26316.7839 | 26305.4376 | -0.043114 | False |
| g5-trace-small-explicit | 36.0/36.0 | 2120.0/2120.0 | 31724.8411 | 31800.70225 | 0.239122 | 31739.259 | 31727.1097 | -0.038278 | False |
| g5-tracedetail-small-default | 55.0/55.0 | 3263.0/3263.0 | 36124.1157 | 36308.3211 | 0.509924 | 36183.1671 | 36072.51615 | -0.305808 | False |
| g5-tracedetail-small-delayed | 54.0/54.0 | 3007.0/3007.0 | 34888.1128 | 34883.37235 | -0.013588 | 34978.958 | 34858.7554 | -0.343643 | False |
| g5-tracedetail-small-explicit | 57.0/57.0 | 3359.0/3359.0 | 39007.0235 | 39018.95555 | 0.030589 | 38988.112 | 39018.6284 | 0.078271 | False |
| g5-traceprint-small-default | 40.0/40.0 | 2174.0/2174.0 | 30813.8635 | 30652.113 | -0.524928 | 30772.816 | 30638.384 | -0.436853 | False |
| g5-traceprint-small-delayed | 39.0/39.0 | 1918.0/1918.0 | 29409.1515 | 29410.05 | 0.003055 | 29422.2935 | 29534.483 | 0.381308 | False |
| g5-traceprint-small-explicit | 43.0/43.0 | 2302.0/2302.0 | 34821.145 | 34780.496 | -0.116737 | 34916.8025 | 34828.548 | -0.252757 | False |
| matchq-integer-success | 1.0/1.0 | 128.0/128.0 | 4340.3973 | 4340.21665 | -0.004162 | 4339.99 | 4340.2483 | 0.005952 | False |
| matchq-integer-fail | 2.0/2.0 | 129.0/129.0 | 4462.22755 | 4469.91215 | 0.172214 | 4462.20575 | 4470.2829 | 0.181012 | True |
| cases-wide-no-match | 33.001/33.001 | 263247.864/263247.864 | 2400500.575 | 2400540.234 | 0.001652 | 2400600.048 | 2400635.477 | 0.001476 | False |
| count-wide-no-match | 30.0/30.0 | 262601.0/262601.0 | 2178861.627 | 2178899.384 | 0.001733 | 2178950.689 | 2178949.453 | -0.000057 | False |
| position-wide-no-match | 1642.001/1642.001 | 576255.864/576255.864 | 2573212.334 | 2573271.267 | 0.002290 | 2573273.946 | 2573327.897 | 0.002097 | False |
| deletecases-wide-no-match | 24.001/24.001 | 247447.864/247447.864 | 2670264.699 | 2670288.078 | 0.000876 | 2670122.795 | 2670174.868 | 0.001950 | False |
| replace-wide-no-match | 30.001/30.001 | 184455.864/184455.864 | 1163591.022 | 1163607.353 | 0.001404 | 1163589.494 | 1163580.985 | -0.000731 | False |
| dot-three-no-input | 596.0/596.0 | 23472.0/23472.0 | 253103.48875 | 253148.6514 | 0.017844 | 253107.7073 | 253113.92305 | 0.002456 | False |
| differences-no-input | 13.0/13.0 | 672.0/672.0 | 9396.4118 | 9399.9849 | 0.038026 | 9397.11245 | 9399.90505 | 0.029718 | False |
| take-multispec-small | 109.0/109.0 | 5824.0/5824.0 | 49240.33585 | 49230.8994 | -0.019164 | 49240.81665 | 49230.9368 | -0.020064 | False |
| g5-monitor-options-none | 9.0/9.0 | 953.0/953.0 | 18059.1065 | 18287.283 | 1.263498 | 18099.3775 | 18252.52 | 0.846120 | True |
| g5-monitor-explicit-held | 13.0005/13.0005 | 1727.856/1727.856 | 17844.6935 | 17817.796 | -0.150731 | 17802.3285 | 17836.5695 | 0.192340 | False |
| g5-names-default | 6253.005/6253.005 | 552512.36/552512.36 | 3482226.425 | 3482482.64 | 0.007358 | 3482450.865 | 3482907.835 | 0.013122 | False |
| g5-testreport-options-default | 54.0/54.0 | 5609.0/5609.0 | 43243.005 | 42939.6975 | -0.701402 | 43035.1045 | 43341.933 | 0.712973 | False |
| g5-testreport-unread-default | 41.0005/41.0005 | 4598.4/4598.4 | 28787.5865 | 28824.721 | 0.128995 | 28870.296 | 28827.473 | -0.148329 | False |
| g5-monitor-evaluationmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13351.8305 | 13413.13 | 0.459109 | 13352.0195 | 13509.1135 | 1.176556 | True |
| g5-monitor-stepmonitor-latent | 8.0/8.0 | 569.0/569.0 | 13353.098 | 13407.9575 | 0.410837 | 13353.73 | 13408.6805 | 0.411499 | True |
| g5-names-spellingcorrection-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3473506.0 | 3473160.255 | -0.009954 | 3473136.635 | 3473238.02 | 0.002919 | False |
| g5-names-resolvecontextaliases-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3472823.82 | 3473282.635 | 0.013212 | 3473032.385 | 3473185.95 | 0.004422 | False |
| g5-names-ignorecase-latent | 6260.005/6260.005 | 552632.36/552632.36 | 3470460.755 | 3470982.68 | 0.015039 | 3470430.91 | 3470788.45 | 0.010302 | False |
| g5-testreport-sametest-latent | 52.0/52.0 | 4969.0/4969.0 | 27580.2135 | 27600.698 | 0.074272 | 27581.157 | 27659.89 | 0.285459 | False |
| g5-testreport-timeconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27597.155 | 27673.544 | 0.276800 | 27593.525 | 27673.081 | 0.288314 | True |
| g5-testreport-memoryconstraint-latent | 52.0/52.0 | 4969.0/4969.0 | 27531.5605 | 27822.3595 | 1.056239 | 27602.0305 | 27679.817 | 0.281814 | True |

Diagnostic callgrind profiles for candidate-01 TestReport options/unread are in [candidate-01/attribution/details.json](../../../.temp/brief-578/candidate-01/attribution/details.json) with [hardware calibration](../../../.temp/brief-578/candidate-01/attribution/calibration.tsv). The named read/finish and extracted-tail costs motivated the single-variable candidates. The anonymous +240-self-instruction function was not symbolically identified; copying is an inference. Actual diagnostic N values are 100/200; the inherited status print incorrectly said 10/20. These diagnostic counts did not replace the hardware gate.

Callgrind calibration rows (verbatim diagnostic receipts):

```text
label	N	build	total	perf_total	calibration_pct
g5-testreport-options-default	100	base	7854399	7785373	0.8866113415503696
g5-testreport-options-default	100	candidate-01	7888096	7823218	0.8293006790811575
g5-testreport-options-default	200	base	12146136	12071084	0.6217502918544948
g5-testreport-options-default	200	candidate-01	12247482	12185898	0.5053710444646864
g5-testreport-unread-default	100	base	6473891	6408852	1.0148307372365606
g5-testreport-unread-default	100	candidate-01	6482546	6417082	1.020152150151743
g5-testreport-unread-default	200	base	9374045	9311189	0.6750587921692919
g5-testreport-unread-default	200	candidate-01	9384629	9334570	0.536275372084627
```
| Diagnostic label | Callgrind base slope | Callgrind candidate-01 slope |
| --- | --- | --- |
| g5-testreport-options-default | 42917.37 | 43593.86 |
| g5-testreport-unread-default | 29001.54 | 29020.83 |

### 4. Stop-loss, residuals and adjacent boundary

Tuning stopped at eight measured candidates and two consecutive candidates improving none of the best counters. The selected candidate minimizes persistent regressing labels, then worst persistent regression, with flat allocations. The three candidate source differences were restored under SHA256 guards; the final source matches the selected measured source manifest. [Final source/body/binary audit](../../../.temp/brief-578/final-audit.json).

Counters and selection rationale: [stop-loss.json](../../../.temp/brief-578/stop-loss.json) and [final-selection.json](../../../.temp/brief-578/final-selection.json). All raw candidates are retained; no favorable-only rerun, averaged-away label or dropped workload was used.

Final persistent instruction residuals: `g5-tracedetail-small-default`, `g5-testreport-unread-default`, `g5-testreport-sametest-latent`, `g5-testreport-timeconstraint-latent`. These remain defects for the director; this handoff is not a performance pass.

Adjacent existing boundary: DumpSave's option-name check accepts only its CompressionLevel/SymbolAttributes catalog defaults, so it rejects IncludedContexts before compression even though the shared persistence prelude prepares context filters. The implementation retains the pre-expanded filter data structurally and never re-expands it on resume. The supported memory-provider witness checks current-context mutation and exactly-once writing; a guest explicit relative-filter witness cannot reach this read in the current subset. That boundary was not changed.

Step-C rule readers, the existing report/body drivers, regex implementation, kernel expression/evaluator/matcher architecture, ownership/retention proof strength and rum-host API remain outside this change. Native ZeroMQ skips are reported as skips in the workspace receipt above.

