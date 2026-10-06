# brief-579 — G5 step C: the rule-valued readers, and the legacy reader surface closed

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

Steps A (`ecd7b5b2`) and B (`58cd4789`) moved the boolean and the phased
readers onto the typed read-and-complete protocol. This step moves the last
reader — `option_rule_expr_for_symbol`, which evaluates a delayed option and
wraps it as `Rule[option, value]` — and its two consumers: **CheckAbort**
(the default `PropagateAborts` rule, read even under an explicit override)
and **ArrayFilter** (the default `Padding` rule, read only when the fourth
argument is absent). Sites P0452, P0979, P1708: lost continuations
**17 -> 14**, closing group G5. Then it **closes the legacy surface**: after
this step no evaluating option reader may be called outside the protocol,
except the private scalar `OptionValue` path and the certified merged
machinery, and the source gate records zero remaining legacy allowances.

brief-576's report and `.temp/brief-576/draft-G5-C.md` are the raw material;
**this brief governs** where they differ. The rulings recorded in brief-577
stand.

## Context pointers

- brief-576's report (§§2-3), `.temp/brief-576/{inventory.md,design.md,inventory-plan.md,performance-plan.md,cases.json,probe.rs,oracle.tsv,lifecycle.tsv}`.
- briefs 577 and 578 and their reports: the protocol (`options/read_phase.rs`),
  the compile controls, the source gate
  (`crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs`) and its
  recorded legacy allowances, the harness
  (`crates/rum-stdlib/tests/option_reader_continuations/`).
- C01 and C02 stay verbatim.

## Scope

**Objective:** the rule decoder and reader, the two consumers, the closure of
the legacy reader surface, all of brief-576's fixtures as permanent tests, and
the standing and performance gates.

**Edit surface:** `runtime/options/{core.rs,read_phase.rs}`, narrowly the
`options.rs` / `kernel_api.rs` exports or visibility; 
`kernel_data/control_data/abort_cleanup.rs` and `abort_cleanup/options.rs`;
stdlib `functional_ops/arrays/{mod.rs,options.rs}`; the option test modules,
compile controls and `read_inventory.rs`; rum-runtime's frame lists.

**Inventory exceptions, authorized by name:** I01's rule-reader signature and
the final private legacy closure; I02's F03, S13, S14 (**+3** derive entries;
no new frame — the named-frame count stays **560**); I03 for the affected
rows; **I04's second public input,
`Session::option_rule_expr_for_symbol(next)`, as an `UnknownHostPayload` row
with disposition `PublicInput`** (beside step A's row; any other new host
payload row is a STOP); I05; I06's ArrayFilter anchors if they move; I07,
I08. The 33-member merged `OptionReentry` domain and the four unresolved D3
consumer rows are unchanged.

**The certified bodies stay verbatim**, and both `local_production_acceptance`
runs must earn the same two C01 and one C02 marks on their merits.

**Standing duty — always in scope** (`TEMPLATE.md`). STOP on any other row
change or disappearance, any new row in untouched code, any self-reaching
state.

**Fence:** no other suspension group; no tensor or operator-driver change in
ArrayFilter; no `Expr`, matcher or evaluator redesign; no leases, `unsafe`,
dependencies, lockfile, catalog reclassification or request-policy change;
the rum-host API unchanged. No edits to `AGENTS.md`, `TODO.md`, root
`PROGRESS.md`, earlier briefs or docs beyond the protocol's rustdoc.

**Disclosure duty:** every changed function and frozen field, inventory row,
measured label, adjacent defect.

## Deliverable

- The rule decoder keeps the requested `Symbol` and produces
  `Some(Rule[option, value])` on both the ordinary and the resumed path;
  absence yields `None` directly. Only then does CheckAbort decode its
  default, read its explicit option and run its primary and fallback, and
  ArrayFilter decode its padding and apply its operator. CheckAbort's
  unconditional default read and ArrayFilter's skipped read under explicit
  padding are preserved. ArrayFilter's cold resume reparses only the
  already-evaluated arguments' pure shape and radius.
- **Closure:** discovery shows zero evaluating reader calls outside the
  protocol, save the private scalar `OptionValue` path and the certified
  merged machinery; the raw non-evaluating Input / Dialog / Once calls stay
  explicitly recorded; the source gate has no remaining legacy allowance and
  its falsifiers still fire.

## Tests

Red first, recorded before any production edit: `CheckAbort-option` and its
twice variant (`True` against `7`; log `{2}` against `{2, 3}`),
`ArrayFilter-option` and twice (`0` against `List[3, 6, 5]`), the unread
explicit-padding control, a second suspension in CheckAbort's explicit
right-hand side and body and in ArrayFilter's operator after the option
phase, cancellation at a suspended rule option. Then **all 70** of brief-576's
fixtures run as owning permanent tests (66 queued/resumed comparisons and four
cancel/abandon cases), with the reader capability, root and tail controls. No
input request may complete with the decoded option as the whole builtin.
Every earlier group's suite stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. Every reader call is a tail call; the source gate enforces it.
3. Existing body scopes and frames keep their cleanup duties; pure
   abandonment of the option phase runs nothing pending.
4. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-579/tmp …`. Bounds: focused 150 s; runtime,
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

**Performance gate.** Base: `.temp/brief-579/base/rum-cli` and
`rum-cli-alloc` — brief-578's selected one-unit candidate, whose production
source is the incoming tree (hashes in `sha256.txt`); do not rebuild them.
Candidates with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), fresh target directory
each, hash checked. Panel: `performance-plan.md`'s step-C labels
(`ArrayFilter[Total,{1,2,3},1]` with default and explicit padding,
`CheckAbort[7,9]` with default and explicit `PropagateAborts`, their
delayed-but-not-suspending defaults) plus the step-A and step-B panels
(`.temp/brief-577/perf/`, `.temp/brief-578/perf/`) and the controls. Each
result checked first. **Noise floor first:** the base against itself, two
pairs, each label's largest difference its floor; a label regresses when both
candidate samples exceed `max(+0.05%, floor)`. Allocation counts, then two
paired samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`,
one label per process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw
output first. Also report the cumulative G5 delta against
`.temp/brief-577/base/` (pre-G5) for every label. No timing suite. At most
eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-579/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files and functions, frozen fields, the closed legacy surface.
2. Verbatim red and green output (all 70 fixtures), compile controls, the
   source gate, ownership and certificate results, the broad suites; every
   inventory row change, the I04 row named.
3. Three sites repaired (17 -> 14 predicted; G5 closed); the noise floors,
   every measured row and the cumulative pre-G5 deltas.
4. Stop-loss counters, residuals, open questions.

---

## Implementation report — 2026-10-06

Implemented the three rule-reader sites and closed G5. Selected **candidate-04**. All required functional/source/ownership checks and both certificate configurations pass; the workspace passes with nine explicit native ZeroMQ skips. **The instruction gate does not pass: five residual regressions, worst persistent +0.6705184729595137%.** Allocation-count and allocated-byte slopes are unchanged on all 67 immediate-base labels. No git command, network access, dependency, lockfile, unsafe, catalog or rum-host API change.

### 1. Changed files, functions and fields

Production functions: options/core.rs Session::option_rule_expr_for_symbol now requires C: OptionReadContinuation<Option<Expr>>, returns the builtin Expr, maps absence straight to finish(None), shares RuleOptionDecode across immediate and resumed outcomes. options/read_phase.rs RuleOptionDecode::decode retains the requested Symbol (not the entry name) and constructs Some(Rule[requested, value]). Existing generic frame/body unchanged.

abort_cleanup.rs Session::eval_check_abort tail-calls the reader after arity validation. abort_cleanup/options.rs adds CheckAbortOptionWorking::{finish,freeze}, CheckAbortOptionRead::finish and Session::finish_check_abort_default; the latter preserves the old post-read body verbatim except the new argument binding/signature. Existing explicit-RHS, primary, fallback and abort cleanup frames remain intact. Its frozen field is args: SharedArgs.

arrays/mod.rs eval_array_filter keeps argument/array/radius validation before reading, and explicit padding bypasses the reader. arrays/options.rs adds ArrayFilterOptionWorking::{finish,freeze} and ArrayFilterOptionRead::finish. The frozen field is args: SharedArgs; shape/flat/radius are owned working data (moved, never cloned), rebuilt only from already-evaluated validated arguments on resume by pure parsers. Tensor and operator drivers unchanged.

F03: RuleOptionDecode { option: Symbol }; S13: CheckAbortOptionRead { args: SharedArgs }; S14: ArrayFilterOptionRead { args: SharedArgs }. Exactly three FrameRootTrace additions, no PureFrameAbandon addition, no new frame; named count 560. No Session/observer/NativeStackDepth capture.

Closure: RULES is a tail-only typed inventory, no LEGACY exception. discover applies the tail rule to every READERS entry. RAW_SITES explicitly pins the protocol internals, private scalar OptionValue resolver, raw Input/Dialog/Once, and the two certified merged driver calls. New caller, non-tail, opaque macro, alias reference and missing-continuation falsifiers remain. The scalar resolver is checked private; the old raw getter and entry resolver remain options-private. No options/kernel_api export change was needed.

Tests: all 66 brief-576 comparisons and four cancellation/abandonment fixtures run permanently. Five historical C tests were added (one unread control is also retained in the earlier suite); three post-option suspension tests and one test covering eight rule cancellation/abandonment cases were added. There are 100 owning tests in the full integration target. The protocol gets a requested-context/name-fallback/absence/ordinary-error test and six more compile controls (18 total). All fixtures compare rum with rum, with exact prompts, final owner clearance, 1+1 reuse and ordered streams/logs.


Every changed production/test function: [changed-functions.json](../../../.temp/brief-579/changed-functions.json)
```json
{
  "options/core.rs": [
    "Session::option_rule_expr_for_symbol"
  ],
  "options/read_phase.rs": [
    "RuleOptionDecode::decode"
  ],
  "kernel_data/control_data/abort_cleanup.rs": [
    "Session::eval_check_abort"
  ],
  "kernel_data/control_data/abort_cleanup/options.rs": [
    "CheckAbortOptionWorking::finish",
    "CheckAbortOptionWorking::freeze",
    "CheckAbortOptionRead::finish",
    "Session::finish_check_abort_default"
  ],
  "functional_ops/arrays/mod.rs": [
    "eval_array_filter"
  ],
  "functional_ops/arrays/options.rs": [
    "ArrayFilterOptionWorking::finish",
    "ArrayFilterOptionWorking::freeze",
    "ArrayFilterOptionRead::finish"
  ],
  "options/tests/read_phase.rs": [
    "read_phase_compile_controls_pin_missing_private_and_unrooted_errors",
    "rule_read_preserves_requested_symbol_and_absence_without_ordinary_capture"
  ],
  "options_suspension/read_inventory.rs": [
    "expected",
    "discover",
    "read_inventory_is_closed_and_migrated_calls_are_tails",
    "phase_read_inventory_falsifies_non_tail_and_missing_continuation",
    "legacy_evaluating_reader_surface_is_closed"
  ],
  "option_reader_continuations/rule.rs": [
    "oracle_arrayfilter_option",
    "oracle_checkabort_option",
    "oracle_arrayfilter_option_twice576",
    "oracle_checkabort_option_twice576",
    "oracle_arrayfilter_unread_default",
    "checkabort_default_before_explicit_rhs_and_body",
    "checkabort_default_before_primary_and_fallback_suspensions",
    "arrayfilter_operator_suspends_after_default_without_replaying_arguments",
    "rule_read_cancel_and_abandon_run_no_pending_guest_work"
  ],
  "retention_closure/census/tests.rs": [
    "baseline (source-position tuple only: 224 -> 233)"
  ]
}
```

The selected rule reader uses `#[inline(never)]`; ArrayFilterOptionWorking::finish uses `#[inline(always)]`. The final ArrayFilter working descriptor owns shape: Vec<usize>, flat: Vec<Expr>, spec: ArrayFilterSpec and borrows args/operator. The pure cold reparse and frozen args-only contract are unchanged. Candidate-05’s direct explicit-padding helper was rejected and removed by restoring the recorded candidate-04 files.

| Changed file | Bytes | Lines |
| --- | ---: | ---: |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/options.rs` | 2720 | 77 |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs` | 14341 | 430 |
| `crates/rum-runtime/src/runtime/options/core.rs` | 8594 | 243 |
| `crates/rum-runtime/src/runtime/options/read_phase.rs` | 4542 | 141 |
| `crates/rum-runtime/src/runtime/options/tests/read_phase.rs` | 12510 | 333 |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | 173668 | 1557 |
| `crates/rum-stdlib/src/functional_ops/arrays/mod.rs` | 6522 | 213 |
| `crates/rum-stdlib/src/functional_ops/arrays/options.rs` | 2500 | 73 |
| `crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs` | 19129 | 565 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | 20751 | 66 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | 14363 | 65 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 252305 | 1487 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 421204 | 2951 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 399168 | 3141 |
| `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` | 7939 | 221 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 91646 | 69 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 111960 | 318 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 18162 | 3 |
| `crates/rum-stdlib/tests/option_reader_continuations/rule.rs` | 6158 | 142 |
| `crates/rum-stdlib/tests/option_reader_continuations.rs` | 277 | 9 |

The production children and core/read_phase files remain under 10k bytes. abort_cleanup.rs is in the warning range and shrank; its remaining boundary is abort policy/body cleanup. The 19k read_inventory test can later split its visitor from inventory data. The large derived-frame list is deliberate declarative inventory, expressly excluded from a G5 split.

### 2. Verbatim verification and inventory disclosure

**red** — exit 101. [red.log](../../../.temp/brief-579/red.log)


```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --test option_reader_continuations -- --nocapture
```


```text
test result: FAILED. 93 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
```

**fixtures-green-01** — exit 101. [fixtures-green-01.log](../../../.temp/brief-579/fixtures-green-01.log)


```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --test option_reader_continuations -- --nocapture
```


```text
error: cannot find derive macro `FrameRootTrace` in this scope
error[E0277]: the trait bound `CheckAbortOptionRead: suspension::trace::FrameRootTrace` is not satisfied
error[E0277]: the trait bound `CheckAbortOptionRead: suspension::trace::FrameRootTrace` is not satisfied
error: could not compile `rum-runtime` (lib) due to 3 previous errors
```

**fixtures-green-02** — exit 0. [fixtures-green-02.log](../../../.temp/brief-579/fixtures-green-02.log)


```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --test option_reader_continuations -- --nocapture
```


```text
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
```

**candidate-04-fixtures** — exit 0. [candidate-04-fixtures.log](../../../.temp/brief-579/candidate-04-fixtures.log)


```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --test option_reader_continuations -- --nocapture
```


```text
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s
```

**source-gate-01** — exit 0. [source-gate-01.log](../../../.temp/brief-579/source-gate-01.log)


```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib options_suspension -- --nocapture
```


```text
LEGACY_READER_ALLOWANCES=0; private OptionValue, certified merged and raw Input/Dialog/Once sites recorded
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 1078 filtered out; finished in 4.63s
```

**runtime-01** — exit 0. [runtime-01.log](../../../.temp/brief-579/runtime-01.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-runtime -- --nocapture
```


```text
test result: ok. 1124 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 47.85s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.76s
```

**ownership-01** — exit 0. [ownership-01.log](../../../.temp/brief-579/ownership-01.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
```


```text
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7667 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1094 filtered out; finished in 60.65s
```

**fmt-final** — exit 0. [fmt-final.log](../../../.temp/brief-579/fmt-final.log)


```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo fmt --check
```


```text

```

**source-gate-final** — exit 0. [source-gate-final.log](../../../.temp/brief-579/source-gate-final.log)


```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib options_suspension -- --nocapture
```


```text
LEGACY_READER_ALLOWANCES=0; private OptionValue, certified merged and raw Input/Dialog/Once sites recorded
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 1078 filtered out; finished in 4.57s
```

**runtime-final** — exit 0. [runtime-final.log](../../../.temp/brief-579/runtime-final.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-runtime -- --nocapture
```


```text
test result: ok. 1124 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 30.57s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.77s
```

**ownership-final** — exit 0. [ownership-final.log](../../../.temp/brief-579/ownership-final.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
```


```text
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7667 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1094 filtered out; finished in 60.93s
```

**fingerprints-replay** — exit 0. [fingerprints-replay.log](../../../.temp/brief-579/fingerprints-replay.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 0.20s
```

**certificates-true-replay** — exit 101. [certificates-true-replay.log](../../../.temp/brief-579/certificates-true-replay.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```


```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 17.93s
```

Expected stale recorded fingerprint/source-anchor failure after earning the production marks; only exact replayed cells were installed, then the green run above/below re-earned the marks.

**certificates-true-green** — exit 0. [certificates-true-green.log](../../../.temp/brief-579/certificates-true-green.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```


```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 18.09s
```

**certificates-false-replay** — exit 101. [certificates-false-replay.log](../../../.temp/brief-579/certificates-false-replay.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```


```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 18.17s
```

Expected stale recorded fingerprint/source-anchor failure after earning the production marks; only exact replayed cells were installed, then the green run above/below re-earned the marks.

**certificates-false-green** — exit 0. [certificates-false-green.log](../../../.temp/brief-579/certificates-false-green.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```


```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 17.97s
```

**workspace-final** — exit 101. [workspace-final.log](../../../.temp/brief-579/workspace-final.log)


```text
python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture
```


```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 50.99s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 1124 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 30.20s
LEGACY_READER_ALLOWANCES=0; private OptionValue, certified merged and raw Input/Dialog/Once sites recorded
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7667 rows
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: FAILED. 1060 passed; 1 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1276.44s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.83s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
thread 'lifecycle_contract_tests::abandoned_poisoned_session_does_not_contaminate_a_fresh_session' panicked at crates/rum-host/src/session.rs:733:24:
thread 'lifecycle_contract_tests::panic_poison_blocks_public_entries_until_reset' panicked at crates/rum-host/src/session.rs:733:24:
thread 'session::tests::panic_is_complete_response_marks_session_poisoned_and_invalid' panicked at crates/rum-host/src/session.rs:832:24:
thread 'tests::session_lifecycle::host_boundary_catches_panic_and_poisons_session' panicked at crates/rum-host/src/session.rs:733:24:
thread 'tests::session_lifecycle::host_session_introspection_survives_poisoned_session' panicked at crates/rum-host/src/session.rs:733:24:
thread 'tests::session_lifecycle::host_session_reset_recovers_poisoned_session' panicked at crates/rum-host/src/session.rs:733:24:
thread 'panics_during_timed_eval_or_resume_poison_until_reset' panicked at crates/rum-host/tests/suspended_timeout_contract.rs:258:9:
thread 'panics_during_timed_eval_or_resume_poison_until_reset' panicked at crates/rum-host/tests/suspended_timeout_contract.rs:250:9:
thread 'tests::output_once::host_buffers::synthesized_panic_has_no_buffers_and_preserves_published_prefix' panicked at crates/rum-jupyter/src/tests/output_once/host_buffers.rs:81:13:
thread '<unnamed>' panicked at crates/rum-jupyter/src/tests/output_once/host_buffers.rs:81:13:
thread 'zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:88:31:
thread 'zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:328:9:
thread 'terminal::tests::masked_panic_restores_terminal_and_descriptor' panicked at crates/rum-repl/src/terminal/tests.rs:48:9:
thread 'runtime::accounting_mode::options::tests::option_owner_guest_error_and_unwind_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/options.rs:206:42:
thread 'runtime::accounting_mode::options::tests::option_owner_guest_error_and_unwind_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/options.rs:203:42:
thread 'runtime::accounting_mode::rule::tests::option_rule_guest_exits_are_lazy_and_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/rule.rs:241:5:
thread 'runtime::accounting_mode::rule::tests::option_rule_guest_exits_are_lazy_and_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/rule.rs:241:5:
thread 'runtime::accounting_mode::tests::exits::retention_bridge_releases_prefix_on_guest_error_and_panic' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/exits.rs:4:5:
thread 'runtime::accounting_mode::tests::retention_lifetime_normal_early_error_drop_and_unwind' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs:69:26:
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
thread 'runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_after_input_resume' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:393:5:
thread 'runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_through_dispatch' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:393:5:
thread 'runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:417:5:
thread 'runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:417:5:
thread 'runtime::tests::messages::push_message_rejects_unregistered_builtin_message_tags' panicked at crates/rum-runtime/src/runtime/session/session_helpers.rs:347:25:
thread 'runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback' panicked at crates/rum-runtime/src/runtime/tests/session_memory_maintained.rs:413:5:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_error' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_file' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_script' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_value' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'tests::association_depth_contract::lazy_depth_contract::lazy_self_reaching_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/lazy_depth_contract.rs:105:5:
thread 'tests::association_depth_contract::registry::real_catalog_misdeclaration_names_association_key_flatten_recursive_edge' panicked at crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:608:29:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
thread 'tests::association_depth_contract::schema_depth_contract::schema_root_completeness_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs:149:5:
thread 'tests::association_depth_contract::schema_depth_contract::structural_self_reaching_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs:55:5:
thread 'tests::retention_closure::census::tests::production_rejection_census_is_complete' panicked at crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:167:9:
assertion failed: options.rejected.iter().any(|r|
C01-O0210-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0210
RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0210
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0210
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0210
C01-O0200-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0200
RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0200
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0200
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0200
C01-O011-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O011
RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O011
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O011
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O011
C01-O010-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O010
RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O010
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O010
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O010
C02-I01-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:233:58-233:68	<rum_core::expr::SharedArgs as std::ops::Deref>::deref	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint I01
RET002	crates/rum-runtime/src/runtime/options/core.rs:234:73-234:80	<rum_core::expr::SharedArgs as std::ops::Deref>::deref	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint I01
RET002	crates/rum-runtime/src/runtime/options/core.rs:234:83-234:90	<rum_core::expr::SharedArgs as std::ops::Deref>::deref	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint I01
C02-I02-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:233:34-233:38	<rum_core::expr::SharedExpr as std::ops::Deref>::deref	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint I02
C02-I04-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:234:73-234:80	<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint I04
RET002	crates/rum-runtime/src/runtime/options/core.rs:234:83-234:90	<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint I04
C02-R01-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:230:20-230:83	rum_runtime::runtime::accounting_mode::rule::clone_result	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint R01
RET002	crates/rum-runtime/src/runtime/options/core.rs:234:13-234:91	rum_runtime::runtime::accounting_mode::rule::evaluate_rule	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint R01
RET002	crates/rum-runtime/src/runtime/options/core.rs:236:13-236:76	rum_runtime::runtime::accounting_mode::rule::clone_result	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint R01
C02-R02-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:230:20-230:83	rum_runtime::runtime::accounting_mode::rule::clone_result	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint R02
RET002	crates/rum-runtime/src/runtime/options/core.rs:234:13-234:91	rum_runtime::runtime::accounting_mode::rule::evaluate_rule	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint R02
RET002	crates/rum-runtime/src/runtime/options/core.rs:236:13-236:76	rum_runtime::runtime::accounting_mode::rule::clone_result	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint R02
C02-B13-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:233:12-233:54	rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint B13
C02-B05-stale: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:233:58-233:68	std::vec::Vec<rum_core::expr::Expr>::len	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	stale contract fingerprint B05
```

**census-position-replay** — exit 101. [census-position-replay.log](../../../.temp/brief-579/census-position-replay.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
```


```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 17.86s
thread 'tests::retention_closure::census::tests::production_rejection_census_is_complete' panicked at crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:167:9:
assertion failed: options.rejected.iter().any(|r|
```

**census-after-anchor** — exit 101. [census-after-anchor.log](../../../.temp/brief-579/census-after-anchor.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
```


```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 27.01s
thread 'tests::retention_closure::census::tests::production_rejection_census_is_complete' panicked at crates/rum-stdlib/src/tests/retention_closure/local/promotion.rs:27:38:
fresh certificate rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol: RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O011
RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O010
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0210
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0200
```

**fmt-after-anchor** — exit 0. [fmt-after-anchor.log](../../../.temp/brief-579/fmt-after-anchor.log)


```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo fmt --check
```


```text

```

**fingerprints-after-anchor** — exit 0. [fingerprints-after-anchor.log](../../../.temp/brief-579/fingerprints-after-anchor.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 0.23s
```

**certificates-true-anchor-replay** — exit 101. [certificates-true-anchor-replay.log](../../../.temp/brief-579/certificates-true-anchor-replay.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```


```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 18.64s
```

Expected stale recorded fingerprint/source-anchor failure after earning the production marks; only exact replayed cells were installed, then the green run above/below re-earned the marks.

**certificates-true-anchor-green** — exit 0. [certificates-true-anchor-green.log](../../../.temp/brief-579/certificates-true-anchor-green.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```


```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 19.18s
```

**certificates-false-anchor-replay** — exit 101. [certificates-false-anchor-replay.log](../../../.temp/brief-579/certificates-false-anchor-replay.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```


```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 21.29s
```

Expected stale recorded fingerprint/source-anchor failure after earning the production marks; only exact replayed cells were installed, then the green run above/below re-earned the marks.

**certificates-false-anchor-green** — exit 0. [certificates-false-anchor-green.log](../../../.temp/brief-579/certificates-false-anchor-green.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```


```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 19.18s
```

**census-anchor-green** — exit 0. [census-anchor-green.log](../../../.temp/brief-579/census-anchor-green.log)


```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
```


```text
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 26.50s
```

**workspace-after-anchor** — exit 0. [workspace-after-anchor.log](../../../.temp/brief-579/workspace-after-anchor.log)


```text
python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-579/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture
```


```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.00s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.03s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 1124 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 30.30s
LEGACY_READER_ALLOWANCES=0; private OptionValue, certified merged and raw Input/Dialog/Once sites recorded
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7667 rows
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1061 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1174.77s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.02s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.17s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.96s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Native ZeroMQ: **9 skips**, due to unavailable loopback binding under the explicit repository skip flag; these are not validated native round trips. Runtime’s three ignored tests are reported as ignored in the verbatim summaries.

The first workspace run found one adjacent maintenance failure: the unchanged C02 method-syntax rejection had moved from line 224 to 233. The fresh production census replay confirmed columns 58–68 and identical source text. Only that locator in census/tests.rs::baseline was relocated under the standing maintenance duty. The next focused run passed the locator and failed on expected expired global fingerprints; after replaying all 68 fingerprints and both certificate configurations, the focused census and final workspace passed. No rejection identity, category or contract was changed. [census-position-rows.json](../../../.temp/brief-579/census-position-rows.json); [census-anchor-change.json](../../../.temp/brief-579/census-anchor-change.json).

Measured source identity: all production files still match candidate-04 byte-for-byte. The sole .rs/.py/.toml difference from the preserved measured manifest is this test-only locator. [measured-source-exception.json](../../../.temp/brief-579/measured-source-exception.json) records the exact old/new hashes; no original candidate manifest or binary was rewritten.

The initial compile failure was an unqualified derive import in the new CheckAbort child. Qualifying rum_derive::FrameRootTrace fixed it before candidate-01; no documented contract was weakened.

**All 70 historical fixtures, red then green, verbatim row contents.** Each comparison lists label, resumed value/log, queued value/log, actual/expected prompts, equality, status, termination, messages and prints; four lifecycle rows follow. Duplicate pre-existing unread controls produce identical rows and are represented once. The complete logs retain every execution.

[red-70-fixtures.tsv](../../../.temp/brief-579/red-70-fixtures.tsv)
```tsv
Monitor-options	"77"	"List[1, 2, 3]"	"77"	"List[1, 2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Cases-Heads-option	"List[1, 2, 3]"	"List[2]"	"List[1, 2, 3]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DeleteCases-Heads-option	"List[1, 3]"	"List[2]"	"List[1, 3]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Count-Heads-option	"3"	"List[2]"	"3"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
MemberQ-Heads-option	"True"	"List[2]"	"True"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
FreeQ-Heads-option	"False"	"List[2]"	"False"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Position-Heads-option	"List[List[2]]"	"List[2]"	"List[List[2]]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Level-Heads-option	"List[1, 2, 3]"	"List[2]"	"List[1, 2, 3]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Depth-Heads-option	"3"	"List[2]"	"3"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Replace-Heads-option	"List[1, x, 3]"	"List[2]"	"List[1, x, 3]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Trace-Heads-option	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[2]"	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ArrayFilter-option	"0"	"List[2]"	"List[3, 6, 5]"	"List[2]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
CheckAbort-option	"True"	"List[2]"	"7"	"List[2, 3]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-SpellingCorrection	"List[]"	"List[2]"	"List[]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-ResolveContextAliases	"List[]"	"List[2]"	"List[]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-IgnoreCase	"List[]"	"List[2]"	"List[]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Monitor-EvaluationMonitor	"77"	"List[2, 3]"	"77"	"List[2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Monitor-StepMonitor	"77"	"List[2, 3]"	"77"	"List[2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
TestReport-option-SameTest	"1"	"List[2]"	"1"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-option-TimeConstraint	"1"	"List[2]"	"1"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-option-MemoryConstraint	"1"	"List[2]"	"1"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DumpSave-option	"$Failed"	"List[2]"	"$Failed"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "DumpSave", tag: "io", text: "could not write `.temp/brief-578/mock.rum`: probe provider is read-only" }]	[RuntimeMessage { symbol: "DumpSave", tag: "io", text: "could not write `.temp/brief-578/mock.rum`: probe provider is read-only" }]	[]	[]
Cases-Heads-after-target	"List[2]"	"List[1, 2]"	"List[2]"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DeleteCases-Heads-after-target	"List[1, 3]"	"List[1, 2]"	"List[1, 3]"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-two-args	"1"	"List[2]"	"1"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Monitor-options-twice	"77"	"List[2, 4, 5]"	"77"	"List[2, 4, 5]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Monitor-options-twice576	"77"	"List[1, 20, 21, 2, 3]"	"77"	"List[1, 20, 21, 2, 3]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Cases-Heads-option-twice576	"List[1, 2, 3]"	"List[2, 20, 21]"	"List[1, 2, 3]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DeleteCases-Heads-option-twice576	"List[1, 3]"	"List[2, 20, 21]"	"List[1, 3]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Count-Heads-option-twice576	"3"	"List[2, 20, 21]"	"3"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
MemberQ-Heads-option-twice576	"True"	"List[2, 20, 21]"	"True"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
FreeQ-Heads-option-twice576	"False"	"List[2, 20, 21]"	"False"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Position-Heads-option-twice576	"List[List[2]]"	"List[2, 20, 21]"	"List[List[2]]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Level-Heads-option-twice576	"List[1, 2, 3]"	"List[2, 20, 21]"	"List[1, 2, 3]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Depth-Heads-option-twice576	"3"	"List[2, 20, 21]"	"3"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Replace-Heads-option-twice576	"List[1, x, 3]"	"List[2, 20, 21]"	"List[1, x, 3]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Trace-Heads-option-twice576	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[2, 20, 21]"	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ArrayFilter-option-twice576	"0"	"List[2, 20, 21]"	"List[3, 6, 5]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
CheckAbort-option-twice576	"True"	"List[2, 20, 21]"	"7"	"List[2, 20, 21, 3]"	["p: ", "second: "]	["p: ", "second: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-SpellingCorrection-twice576	"List[]"	"List[2, 20, 21]"	"List[]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-ResolveContextAliases-twice576	"List[]"	"List[2, 20, 21]"	"List[]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-IgnoreCase-twice576	"List[]"	"List[2, 20, 21]"	"List[]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Monitor-EvaluationMonitor-twice576	"77"	"List[2, 20, 21, 3]"	"77"	"List[2, 20, 21, 3]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Monitor-StepMonitor-twice576	"77"	"List[2, 20, 21, 3]"	"77"	"List[2, 20, 21, 3]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
TestReport-option-SameTest-twice576	"1"	"List[2, 20, 21]"	"1"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-option-TimeConstraint-twice576	"1"	"List[2, 20, 21]"	"1"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-option-MemoryConstraint-twice576	"1"	"List[2, 20, 21]"	"1"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DumpSave-option-twice576	"$Failed"	"List[2, 20, 21]"	"$Failed"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "DumpSave", tag: "io", text: "could not write `.temp/brief-578/mock.rum`: probe provider is read-only" }]	[RuntimeMessage { symbol: "DumpSave", tag: "io", text: "could not write `.temp/brief-578/mock.rum`: probe provider is read-only" }]	[]	[]
TestReport-two-args-twice576	"1"	"List[2, 20, 21]"	"1"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Monitor-runtime-two-options	"77"	"List[1, 2, 3, 4, 5]"	"77"	"List[1, 2, 3, 4, 5]"	["eval: ", "step: "]	["eval: ", "step: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Names-two-options	"List[]"	"List[1, 2, 3, 4]"	"List[]"	"List[1, 2, 3, 4]"	["spell: ", "alias: "]	["spell: ", "alias: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-three-options	"1"	"List[1, 2, 3, 4]"	"1"	"List[1, 2, 3, 4]"	["same: ", "time: ", "memory: "]	["same: ", "time: ", "memory: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-later-default-mutation	"List[]"	"List[1, 2, 3]"	"List[]"	"List[1, 2, 3]"	["spell: ", "new-alias: "]	["spell: ", "new-alias: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Count-Heads-after-target576	"3"	"List[1, 2]"	"3"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
MemberQ-Heads-after-target576	"True"	"List[1, 2]"	"True"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
FreeQ-Heads-after-target576	"False"	"List[1, 2]"	"False"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Position-Heads-after-target576	"List[List[2]]"	"List[1, 2]"	"List[List[2]]"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Replace-Heads-after-target576	"List[1, x, 3]"	"List[1, 2]"	"List[1, x, 3]"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Count-option-error-after-resume	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	["option: "]	["option: "]	true	"Other { message: \"uncaught throw: \\\"g5 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g5 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g5 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g5 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
Names-later-option-error-after-resume	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	["spell: "]	["spell: "]	true	"Other { message: \"uncaught throw: \\\"g5 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g5 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g5 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g5 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
Monitor-option-streams	"77"	"List[1, 2, 3]"	"77"	"List[1, 2, 3]"	["option: "]	["option: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }]	["before", "after", "0", "body"]	["before", "after", "0", "body"]
ArrayFilter-unread-default	"List[3, 6, 5]"	"List[]"	"List[3, 6, 5]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-unread-default	"1"	"List[]"	"1"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-early-invalid-unread-later	"Names[\"Global`never548\"]"	"List[]"	"Names[\"Global`never548\"]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "Names", tag: "optval", text: "Unsupported value True for option SpellingCorrection in Names; only False is currently supported." }, RuntimeMessage { symbol: "Names", tag: "optval", text: "Unsupported value True for option SpellingCorrection in Names; only False is currently supported." }]	[RuntimeMessage { symbol: "Names", tag: "optval", text: "Unsupported value True for option SpellingCorrection in Names; only False is currently supported." }, RuntimeMessage { symbol: "Names", tag: "optval", text: "Unsupported value True for option SpellingCorrection in Names; only False is currently supported." }]	[]	[]
TraceDetail-Heads-option	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]]]"	"List[2]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]]]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TracePrint-Heads-option	"3"	"List[2]"	"3"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["Plus[1, 2]", "3"]	["Plus[1, 2]", "3"]
lifecycle	cancel-stop-0-without-answer-true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
lifecycle	cancel-stop-1-without-answer-true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
lifecycle	cancel-stop-0-without-answer-false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
lifecycle	cancel-stop-1-without-answer-false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
```

[selected-70-fixtures.tsv](../../../.temp/brief-579/selected-70-fixtures.tsv)
```tsv
Monitor-options	"77"	"List[1, 2, 3]"	"77"	"List[1, 2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Cases-Heads-option	"List[1, 2, 3]"	"List[2]"	"List[1, 2, 3]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DeleteCases-Heads-option	"List[1, 3]"	"List[2]"	"List[1, 3]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Count-Heads-option	"3"	"List[2]"	"3"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
MemberQ-Heads-option	"True"	"List[2]"	"True"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
FreeQ-Heads-option	"False"	"List[2]"	"False"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Position-Heads-option	"List[List[2]]"	"List[2]"	"List[List[2]]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Level-Heads-option	"List[1, 2, 3]"	"List[2]"	"List[1, 2, 3]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Depth-Heads-option	"3"	"List[2]"	"3"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Replace-Heads-option	"List[1, x, 3]"	"List[2]"	"List[1, x, 3]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Trace-Heads-option	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[2]"	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ArrayFilter-option	"List[3, 6, 5]"	"List[2]"	"List[3, 6, 5]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
CheckAbort-option	"7"	"List[2, 3]"	"7"	"List[2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-SpellingCorrection	"List[]"	"List[2]"	"List[]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-ResolveContextAliases	"List[]"	"List[2]"	"List[]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-IgnoreCase	"List[]"	"List[2]"	"List[]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Monitor-EvaluationMonitor	"77"	"List[2, 3]"	"77"	"List[2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Monitor-StepMonitor	"77"	"List[2, 3]"	"77"	"List[2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
TestReport-option-SameTest	"1"	"List[2]"	"1"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-option-TimeConstraint	"1"	"List[2]"	"1"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-option-MemoryConstraint	"1"	"List[2]"	"1"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DumpSave-option	"$Failed"	"List[2]"	"$Failed"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "DumpSave", tag: "io", text: "could not write `.temp/brief-578/mock.rum`: probe provider is read-only" }]	[RuntimeMessage { symbol: "DumpSave", tag: "io", text: "could not write `.temp/brief-578/mock.rum`: probe provider is read-only" }]	[]	[]
Cases-Heads-after-target	"List[2]"	"List[1, 2]"	"List[2]"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DeleteCases-Heads-after-target	"List[1, 3]"	"List[1, 2]"	"List[1, 3]"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-two-args	"1"	"List[2]"	"1"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Monitor-options-twice	"77"	"List[2, 4, 5]"	"77"	"List[2, 4, 5]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Monitor-options-twice576	"77"	"List[1, 20, 21, 2, 3]"	"77"	"List[1, 20, 21, 2, 3]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Cases-Heads-option-twice576	"List[1, 2, 3]"	"List[2, 20, 21]"	"List[1, 2, 3]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DeleteCases-Heads-option-twice576	"List[1, 3]"	"List[2, 20, 21]"	"List[1, 3]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Count-Heads-option-twice576	"3"	"List[2, 20, 21]"	"3"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
MemberQ-Heads-option-twice576	"True"	"List[2, 20, 21]"	"True"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
FreeQ-Heads-option-twice576	"False"	"List[2, 20, 21]"	"False"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Position-Heads-option-twice576	"List[List[2]]"	"List[2, 20, 21]"	"List[List[2]]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Level-Heads-option-twice576	"List[1, 2, 3]"	"List[2, 20, 21]"	"List[1, 2, 3]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Depth-Heads-option-twice576	"3"	"List[2, 20, 21]"	"3"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Replace-Heads-option-twice576	"List[1, x, 3]"	"List[2, 20, 21]"	"List[1, x, 3]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Trace-Heads-option-twice576	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[2, 20, 21]"	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ArrayFilter-option-twice576	"List[3, 6, 5]"	"List[2, 20, 21]"	"List[3, 6, 5]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
CheckAbort-option-twice576	"7"	"List[2, 20, 21, 3]"	"7"	"List[2, 20, 21, 3]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-SpellingCorrection-twice576	"List[]"	"List[2, 20, 21]"	"List[]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-ResolveContextAliases-twice576	"List[]"	"List[2, 20, 21]"	"List[]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-IgnoreCase-twice576	"List[]"	"List[2, 20, 21]"	"List[]"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Monitor-EvaluationMonitor-twice576	"77"	"List[2, 20, 21, 3]"	"77"	"List[2, 20, 21, 3]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Monitor-StepMonitor-twice576	"77"	"List[2, 20, 21, 3]"	"77"	"List[2, 20, 21, 3]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
TestReport-option-SameTest-twice576	"1"	"List[2, 20, 21]"	"1"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-option-TimeConstraint-twice576	"1"	"List[2, 20, 21]"	"1"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-option-MemoryConstraint-twice576	"1"	"List[2, 20, 21]"	"1"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
DumpSave-option-twice576	"$Failed"	"List[2, 20, 21]"	"$Failed"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "DumpSave", tag: "io", text: "could not write `.temp/brief-578/mock.rum`: probe provider is read-only" }]	[RuntimeMessage { symbol: "DumpSave", tag: "io", text: "could not write `.temp/brief-578/mock.rum`: probe provider is read-only" }]	[]	[]
TestReport-two-args-twice576	"1"	"List[2, 20, 21]"	"1"	"List[2, 20, 21]"	["p: ", "second: "]	["p: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Monitor-runtime-two-options	"77"	"List[1, 2, 3, 4, 5]"	"77"	"List[1, 2, 3, 4, 5]"	["eval: ", "step: "]	["eval: ", "step: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["0"]	["0"]
Names-two-options	"List[]"	"List[1, 2, 3, 4]"	"List[]"	"List[1, 2, 3, 4]"	["spell: ", "alias: "]	["spell: ", "alias: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-three-options	"1"	"List[1, 2, 3, 4]"	"1"	"List[1, 2, 3, 4]"	["same: ", "time: ", "memory: "]	["same: ", "time: ", "memory: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-later-default-mutation	"List[]"	"List[1, 2, 3]"	"List[]"	"List[1, 2, 3]"	["spell: ", "new-alias: "]	["spell: ", "new-alias: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Count-Heads-after-target576	"3"	"List[1, 2]"	"3"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
MemberQ-Heads-after-target576	"True"	"List[1, 2]"	"True"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
FreeQ-Heads-after-target576	"False"	"List[1, 2]"	"False"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Position-Heads-after-target576	"List[List[2]]"	"List[1, 2]"	"List[List[2]]"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Replace-Heads-after-target576	"List[1, x, 3]"	"List[1, 2]"	"List[1, x, 3]"	"List[1, 2]"	["target: ", "option: "]	["target: ", "option: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Count-option-error-after-resume	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	["option: "]	["option: "]	true	"Other { message: \"uncaught throw: \\\"g5 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g5 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g5 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g5 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
Names-later-option-error-after-resume	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	["spell: "]	["spell: "]	true	"Other { message: \"uncaught throw: \\\"g5 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g5 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g5 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g5 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
Monitor-option-streams	"77"	"List[1, 2, 3]"	"77"	"List[1, 2, 3]"	["option: "]	["option: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }]	["before", "after", "0", "body"]	["before", "after", "0", "body"]
ArrayFilter-unread-default	"List[3, 6, 5]"	"List[]"	"List[3, 6, 5]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TestReport-unread-default	"1"	"List[]"	"1"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
Names-early-invalid-unread-later	"Names[\"Global`never548\"]"	"List[]"	"Names[\"Global`never548\"]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "Names", tag: "optval", text: "Unsupported value True for option SpellingCorrection in Names; only False is currently supported." }, RuntimeMessage { symbol: "Names", tag: "optval", text: "Unsupported value True for option SpellingCorrection in Names; only False is currently supported." }]	[RuntimeMessage { symbol: "Names", tag: "optval", text: "Unsupported value True for option SpellingCorrection in Names; only False is currently supported." }, RuntimeMessage { symbol: "Names", tag: "optval", text: "Unsupported value True for option SpellingCorrection in Names; only False is currently supported." }]	[]	[]
TraceDetail-Heads-option	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]]]"	"List[2]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]]]"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
TracePrint-Heads-option	"3"	"List[2]"	"3"	"List[2]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["Plus[1, 2]", "3"]	["Plus[1, 2]", "3"]
lifecycle	cancel-stop-0-without-answer-true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
lifecycle	cancel-stop-1-without-answer-true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
lifecycle	cancel-stop-0-without-answer-false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
lifecycle	cancel-stop-1-without-answer-false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G5 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
```

**Compile controls (18)**
```tsv
fixture	result
missing-fail	error[E0061]: this method takes 4 arguments but 3 arguments were supplied
missing-pass	compile pass
private-fail	error[E0624]: method `option_value_for_symbol` is private
private-pass	compile pass
resolver-private-fail	error[E0624]: method `resolve_option_entry_value` is private
resolver-private-pass	compile pass
root-fail	error[E0277]: the trait bound `Capture: FrameRootTrace` is not satisfied
root-pass	compile pass
rule-missing-fail	error[E0061]: this method takes 3 arguments but 2 arguments were supplied
rule-missing-pass	compile pass
rule-root-fail	error[E0277]: the trait bound `Capture: FrameRootTrace` is not satisfied
rule-root-pass	compile pass
runtime-private-fail	error[E0624]: method `runtime_option_expr_for_symbol` is private
runtime-root-fail	error[E0277]: the trait bound `Capture: FrameRootTrace` is not satisfied
runtime-root-pass	compile pass
syntax-private-fail	error[E0624]: method `eval_option_syntax` is private
syntax-root-fail	error[E0277]: the trait bound `Capture: FrameRootTrace` is not satisfied
syntax-root-pass	compile pass
```

**Closed source discovery (32 calls; 20 typed tails and 12 recorded internal/raw calls)**
```text
READ rum-stdlib/src/structure_ops/surface.rs:41:eval_depth:system_option_bool tail=true
READ rum-stdlib/src/functional_ops/arrays/mod.rs:179:eval_array_filter:option_rule_expr_for_symbol tail=true
READ rum-runtime/src/runtime/package_tools.rs:231:eval_persist_package_state:runtime_option_expr_for_symbol tail=true
READ rum-runtime/src/runtime/package_tools.rs:617:once_should_cache:raw_option_value_for_symbol tail=false
READ rum-runtime/src/runtime/package_tools.rs:624:once_should_cache:raw_option_value_for_symbol tail=false
READ rum-runtime/src/runtime/accounting_mode/options/driver.rs:26:drive:evaluate_runtime_option_expr tail=false
READ rum-runtime/src/runtime/accounting_mode/options/driver.rs:43:drive:evaluate_runtime_option_expr tail=false
READ rum-runtime/src/runtime/tree_walkers/count_like.rs:133:resume_count_after_pattern:system_option_bool tail=true
READ rum-runtime/src/runtime/tree_walkers/count_like.rs:172:resume_member_q_after_pattern:system_option_bool tail=true
READ rum-runtime/src/runtime/tree_walkers/count_like.rs:211:resume_free_q_after_pattern:system_option_bool tail=true
READ rum-runtime/src/runtime/tree_walkers/level.rs:14:eval_level:system_option_bool tail=true
READ rum-runtime/src/runtime/tree_walkers/position.rs:147:resume_position_after_pattern:system_option_bool tail=true
READ rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:102:eval_check_abort:option_rule_expr_for_symbol tail=true
READ rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:139:resume_replace_after_rule_exprs:system_option_bool tail=true
READ rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:108:resume_cases_after_resolved_spec_suspended:system_option_bool tail=true
READ rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:128:resume_cases_after_resolved_spec:system_option_bool tail=true
READ rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:102:resume_delete_cases_after_pattern_suspended:system_option_bool tail=true
READ rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:121:resume_delete_cases_after_pattern:system_option_bool tail=true
READ rum-runtime/src/runtime/debug_tools/monitor.rs:12:eval_monitor:eval_option_syntax tail=true
READ rum-runtime/src/runtime/debug_tools/monitor/options.rs:91:read_evaluation_monitor:runtime_option_expr_for_symbol tail=true
READ rum-runtime/src/runtime/debug_tools/monitor/options.rs:109:finish:runtime_option_expr_for_symbol tail=true
READ rum-runtime/src/runtime/debug_tools/test_report/options.rs:60:read:runtime_option_expr_for_symbol tail=true
READ rum-runtime/src/runtime/debug_tools/trace.rs:53:eval_trace_named:system_option_bool tail=true
READ rum-runtime/src/runtime/builtins_input.rs:194:compile_input_request:raw_option_value_for_symbol tail=false
READ rum-runtime/src/runtime/builtins_input.rs:293:dialog_option_raw_expr:raw_option_value_for_symbol tail=false
READ rum-runtime/src/runtime/namespace_tools/names/options.rs:55:read:runtime_option_expr_for_symbol tail=true
READ rum-runtime/src/runtime/options/builtins.rs:379:resolve_option_value_scalar:resolve_option_entry_value tail=true
READ rum-runtime/src/runtime/options/builtins.rs:388:resolve_option_value_scalar:resolve_option_entry_value tail=true
READ rum-runtime/src/runtime/options/core.rs:19:system_option_bool:option_value_for_symbol tail=false
READ rum-runtime/src/runtime/options/core.rs:67:option_value_for_symbol:resolve_option_entry_value tail=true
READ rum-runtime/src/runtime/options/core.rs:96:option_rule_expr_for_symbol:resolve_option_entry_value tail=false
READ rum-runtime/src/runtime/options/core.rs:185:runtime_option_expr_for_symbol:raw_option_value_for_symbol tail=false
```

**I04**, the sole new host row, is recorded with disposition **PublicInput** beside the step-A input:

```text
UnknownHostPayload	crates/rum-runtime/src/runtime/options/core.rs	82	15	public input rum_runtime::runtime::options::core::impl(Session)@4::option_rule_expr_for_symbol(next): opaque owned generic parameter C : OptionReadContinuation < Option < Expr > >
```

All unknown rows remain unknown. Exactly F03/S13/S14 add three derive entries; PureFrameAbandon and the named-frame count (560) are unchanged. No new self-reaching type. Both certified function bodies remain byte-identical; the 33-member merged OptionReentry domain and four unresolved D3 consumer rows are byte-identical. ArrayFilter census anchors did not move.

Every before/after row, including exact source positions and certificate cells: [ownership-row-changes.json](../../../.temp/brief-579/ownership-row-changes.json).

Every before/after row, including exact source positions and certificate cells: [obligation-row-changes.json](../../../.temp/brief-579/obligation-row-changes.json).

Every before/after row, including exact source positions and certificate cells: [before-census-anchor/fingerprint-row-changes.json](../../../.temp/brief-579/before-census-anchor/fingerprint-row-changes.json).

Every before/after row, including exact source positions and certificate cells: [before-census-anchor/certificate-cell-changes-true.json](../../../.temp/brief-579/before-census-anchor/certificate-cell-changes-true.json).

Every before/after row, including exact source positions and certificate cells: [before-census-anchor/certificate-cell-changes-false.json](../../../.temp/brief-579/before-census-anchor/certificate-cell-changes-false.json).

Every before/after row, including exact source positions and certificate cells: [fingerprint-row-changes.json](../../../.temp/brief-579/fingerprint-row-changes.json).

Every before/after row, including exact source positions and certificate cells: [certificate-cell-changes-true.json](../../../.temp/brief-579/certificate-cell-changes-true.json).

Every before/after row, including exact source positions and certificate cells: [certificate-cell-changes-false.json](../../../.temp/brief-579/certificate-cell-changes-false.json).

Every before/after row, including exact source positions and certificate cells: [census-anchor-change.json](../../../.temp/brief-579/census-anchor-change.json).

All ownership and obligation rows changed, including the host disposition ledger: [inventory-row-changes.tsv](../../../.temp/brief-579/inventory-row-changes.tsv)
```tsv
inventory	change	before	after
host_payloads.tsv	source position only	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 16 | 15 | public input rum_runtime::runtime::options::core::impl(Session)@3::system_option_bool(next): opaque owned generic parameter C : OptionReadContinuation < bool >	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 17 | 15 | public input rum_runtime::runtime::options::core::impl(Session)@4::system_option_bool(next): opaque owned generic parameter C : OptionReadContinuation < bool >
host_payloads.tsv	source position only	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 123 | 18 | public input rum_runtime::runtime::options::core::impl(Session)@3::merged_option_exprs_for_system_symbol(reentry): opaque owned generic parameter R : OptionReentry	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 132 | 18 | public input rum_runtime::runtime::options::core::impl(Session)@4::merged_option_exprs_for_system_symbol(reentry): opaque owned generic parameter R : OptionReentry
host_payloads.tsv	added	-	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 82 | 15 | public input rum_runtime::runtime::options::core::impl(Session)@4::option_rule_expr_for_symbol(next): opaque owned generic parameter C : OptionReadContinuation < Option < Expr > >
unknown_attributes.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 7 | 1 | unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , rum_derive :: FrameRootTrace)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 8 | 1 | unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , rum_derive :: FrameRootTrace)
unknown_attributes.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 14 | 1 | unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 15 | 1 | unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
unknown_attributes.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 20 | 1 | unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 21 | 1 | unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
unknown_attributes.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 26 | 1 | unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 27 | 1 | unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
unknown_attributes.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 49 | 1 | unresolved attribute/expansion derive (FrameRootTrace)	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 64 | 1 | unresolved attribute/expansion derive (FrameRootTrace)
unknown_attributes.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 59 | 1 | unresolved attribute/expansion derive (FrameRootTrace)	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 74 | 1 | unresolved attribute/expansion derive (FrameRootTrace)
unknown_attributes.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 84 | 1 | unresolved attribute/expansion derive (FrameRootTrace , rum_derive :: PureFrameAbandon)	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 99 | 1 | unresolved attribute/expansion derive (FrameRootTrace , rum_derive :: PureFrameAbandon)
unknown_attributes.tsv	source position only	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 119 | 1 | unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 121 | 1 | unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
unknown_attributes.tsv	added	-	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/options.rs | 8 | 1 | unresolved attribute/expansion derive (rum_derive :: FrameRootTrace)
unknown_attributes.tsv	added	-	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 49 | 1 | unresolved attribute/expansion derive (FrameRootTrace)
unknown_attributes.tsv	added	-	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/options.rs | 12 | 1 | unresolved attribute/expansion derive (FrameRootTrace)
unknown_captures.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 428 | 21 | closure captures: || (capture types/modes unresolved)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 392 | 21 | closure captures: || (capture types/modes unresolved)
unknown_captures.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 33 | 29 | closure captures: || (capture types/modes unresolved)	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 34 | 29 | closure captures: || (capture types/modes unresolved)
unknown_captures.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 62 | 22 | closure captures: || (capture types/modes unresolved)	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 63 | 22 | closure captures: || (capture types/modes unresolved)
unknown_captures.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 80 | 22 | closure captures: || (capture types/modes unresolved)	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 87 | 22 | closure captures: || (capture types/modes unresolved)
unknown_captures.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 153 | 23 | closure captures: |options| (capture types/modes unresolved)	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 162 | 23 | closure captures: |options| (capture types/modes unresolved)
unknown_captures.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 154 | 22 | closure captures: || (capture types/modes unresolved)	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 163 | 22 | closure captures: || (capture types/modes unresolved)
unknown_captures.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 159 | 18 | closure captures: |entry| (capture types/modes unresolved)	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 168 | 18 | closure captures: |entry| (capture types/modes unresolved)
unknown_captures.tsv	source position only	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 133 | 40 | closure captures: |current| (capture types/modes unresolved)	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 135 | 40 | closure captures: |current| (capture types/modes unresolved)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 61 | 25 | unexpanded macro format! (ends at 61:91)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 62 | 25 | unexpanded macro format! (ends at 62:91)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 71 | 21 | unexpanded macro format! (ends at 74:22)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 72 | 21 | unexpanded macro format! (ends at 75:22)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 97 | 17 | unexpanded macro format! (ends at 97:85)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 98 | 17 | unexpanded macro format! (ends at 98:85)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 154 | 17 | unexpanded macro format! (ends at 156:18)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 118 | 17 | unexpanded macro format! (ends at 120:18)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 163 | 23 | unexpanded macro crate :: resume_or_suspend! (ends at 171:10)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 127 | 23 | unexpanded macro crate :: resume_or_suspend! (ends at 135:10)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 192 | 23 | unexpanded macro crate :: resume_or_suspend! (ends at 197:10)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 156 | 23 | unexpanded macro crate :: resume_or_suspend! (ends at 161:10)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 215 | 31 | unexpanded macro crate :: resume_or_suspend! (ends at 220:18)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 179 | 31 | unexpanded macro crate :: resume_or_suspend! (ends at 184:18)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 236 | 33 | unexpanded macro matches! (ends at 236:75)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 200 | 33 | unexpanded macro matches! (ends at 200:75)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 238 | 31 | unexpanded macro crate :: resume_or_suspend! (ends at 243:18)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 202 | 31 | unexpanded macro crate :: resume_or_suspend! (ends at 207:18)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 311 | 16 | unexpanded macro matches! (ends at 311:84)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 275 | 16 | unexpanded macro matches! (ends at 275:84)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 355 | 17 | unexpanded macro format! (ends at 357:18)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 319 | 17 | unexpanded macro format! (ends at 321:18)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 363 | 17 | unexpanded macro format! (ends at 365:18)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 327 | 17 | unexpanded macro format! (ends at 329:18)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 371 | 17 | unexpanded macro format! (ends at 371:72)	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 335 | 17 | unexpanded macro format! (ends at 335:72)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 118 | 13 | unexpanded macro crate :: resume_or_suspend! (ends at 121:15)	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 133 | 13 | unexpanded macro crate :: resume_or_suspend! (ends at 136:15)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 77 | 5 | unexpanded macro rum_runtime :: resume_or_suspend! (ends at 86:6)	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 79 | 5 | unexpanded macro rum_runtime :: resume_or_suspend! (ends at 88:6)
unknown_macros.tsv	source position only	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 100 | 22 | unexpanded macro rum_runtime :: resume_or_suspend! (ends at 109:10)	UnknownConstruct | crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 102 | 22 | unexpanded macro rum_runtime :: resume_or_suspend! (ends at 111:10)
unknown_macros.tsv	I03 post-read extraction from eval_check_abort	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 123 | 25 | unexpanded macro crate :: resume_or_suspend! (ends at 131:14)	-
unknown_macros.tsv	I03 post-read extraction from option_rule_expr_for_symbol	UnknownConstruct | crates/rum-runtime/src/runtime/options/core.rs | 89 | 13 | unexpanded macro vec! (ends at 89:66)	-
unknown_macros.tsv	added	-	UnknownConstruct | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/options.rs | 56 | 25 | unexpanded macro crate :: resume_or_suspend! (ends at 64:14)
unknown_macros.tsv	added	-	UnknownConstruct | crates/rum-runtime/src/runtime/options/read_phase.rs | 59 | 13 | unexpanded macro vec! (ends at 59:71)
host_payload_ledger.tsv	source position only	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 16 | 15 | public input rum_runtime::runtime::options::core::impl(Session)@3::system_option_bool(next): opaque owned generic parameter C : OptionReadContinuation < bool > | PublicInput | public by-value input; owning storage sites are separate rows | - | -	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 17 | 15 | public input rum_runtime::runtime::options::core::impl(Session)@4::system_option_bool(next): opaque owned generic parameter C : OptionReadContinuation < bool > | PublicInput | public by-value input; owning storage sites are separate rows | - | -
host_payload_ledger.tsv	source position only	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 123 | 18 | public input rum_runtime::runtime::options::core::impl(Session)@3::merged_option_exprs_for_system_symbol(reentry): opaque owned generic parameter R : OptionReentry | PublicInput | public by-value input; owning storage sites are separate rows | - | -	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 132 | 18 | public input rum_runtime::runtime::options::core::impl(Session)@4::merged_option_exprs_for_system_symbol(reentry): opaque owned generic parameter R : OptionReentry | PublicInput | public by-value input; owning storage sites are separate rows | - | -
host_payload_ledger.tsv	I04 authorized PublicInput	-	UnknownHostPayload | crates/rum-runtime/src/runtime/options/core.rs | 82 | 15 | public input rum_runtime::runtime::options::core::impl(Session)@4::option_rule_expr_for_symbol(next): opaque owned generic parameter C : OptionReadContinuation < Option < Expr > > | PublicInput | public by-value input; owning storage sites are separate rows | - | -
obligations.tsv	standing-duty source positions only	N:AlignColumns | AlignColumns | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:34 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::scripting::text_shape::eval_align_columns	N:AlignColumns | AlignColumns | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:34 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::scripting::text_shape::eval_align_columns
obligations.tsv	standing-duty source positions only	N:CheckAbort | CheckAbort | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:93 | propagation.invalid_value and _value retained across body/fallback callbacks | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:150; crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:162; crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:213; crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:214 | rum_stdlib::control_flow::eval_check_abort	N:CheckAbort | CheckAbort | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:93 | propagation.invalid_value and _value retained across body/fallback callbacks | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:114; crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:126; crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:177; crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:178 | rum_stdlib::control_flow::eval_check_abort
obligations.tsv	standing-duty source positions only	N:Complement | Complement | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:103 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::structure_ops::ordering_sets::eval_complement	N:Complement | Complement | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:103 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::structure_ops::ordering_sets::eval_complement
obligations.tsv	standing-duty source positions only	N:ContainsAll | ContainsAll | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:115 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_contains_all	N:ContainsAll | ContainsAll | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:115 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_contains_all
obligations.tsv	standing-duty source positions only	N:ContainsAny | ContainsAny | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:116 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_contains_any	N:ContainsAny | ContainsAny | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:116 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_contains_any
obligations.tsv	standing-duty source positions only	N:ContainsExactly | ContainsExactly | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:117 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_contains_exactly	N:ContainsExactly | ContainsExactly | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:117 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_contains_exactly
obligations.tsv	standing-duty source positions only	N:ContainsNone | ContainsNone | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:118 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_contains_none	N:ContainsNone | ContainsNone | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:118 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_contains_none
obligations.tsv	standing-duty source positions only	N:ContainsOnly | ContainsOnly | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:119 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_contains_only	N:ContainsOnly | ContainsOnly | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:119 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_contains_only
obligations.tsv	standing-duty source positions only	N:DisjointQ | DisjointQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:161 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_disjoint_q	N:DisjointQ | DisjointQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:161 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_disjoint_q
obligations.tsv	standing-duty source positions only	N:DuplicateFreeQ | DuplicateFreeQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:171 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_duplicate_free_q	N:DuplicateFreeQ | DuplicateFreeQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:171 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_duplicate_free_q
obligations.tsv	standing-duty source positions only	N:HtmlUnescape | HtmlUnescape | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:257 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::scripting::html::eval_html_unescape	N:HtmlUnescape | HtmlUnescape | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:257 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::scripting::html::eval_html_unescape
obligations.tsv	standing-duty source positions only	N:IntersectingQ | IntersectingQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:275 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_intersecting_q	N:IntersectingQ | IntersectingQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:275 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_intersecting_q
obligations.tsv	standing-duty source positions only	N:Intersection | Intersection | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:276 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::structure_ops::ordering_sets::eval_intersection	N:Intersection | Intersection | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:276 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::structure_ops::ordering_sets::eval_intersection
obligations.tsv	standing-duty source positions only	N:ParseArgs | ParseArgs | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:412 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::scripting::args::eval_parse_args	N:ParseArgs | ParseArgs | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:412 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::scripting::args::eval_parse_args
obligations.tsv	standing-duty source positions only	N:StringContainsQ | StringContainsQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:535 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:86:9-86:40 strings :: eval_string_contains_q: unresolved non-test cfg alternative or opaque import namespace	N:StringContainsQ | StringContainsQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:535 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:86:9-86:40 strings :: eval_string_contains_q: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringCount | StringCount | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:536 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:218:9-218:35 strings :: eval_string_count: unresolved non-test cfg alternative or opaque import namespace	N:StringCount | StringCount | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:536 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:218:9-218:35 strings :: eval_string_count: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringDelete | StringDelete | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:538 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:172:9-172:36 strings :: eval_string_delete: unresolved non-test cfg alternative or opaque import namespace	N:StringDelete | StringDelete | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:538 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:172:9-172:36 strings :: eval_string_delete: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringEndsQ | StringEndsQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:540 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:106:9-106:36 strings :: eval_string_ends_q: unresolved non-test cfg alternative or opaque import namespace	N:StringEndsQ | StringEndsQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:540 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:106:9-106:36 strings :: eval_string_ends_q: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringFreeQ | StringFreeQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:544 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:237:9-237:36 strings :: eval_string_free_q: unresolved non-test cfg alternative or opaque import namespace	N:StringFreeQ | StringFreeQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:544 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:237:9-237:36 strings :: eval_string_free_q: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringIndent | StringIndent | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:545 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::scripting::text_shape::eval_string_indent	N:StringIndent | StringIndent | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:545 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::scripting::text_shape::eval_string_indent
obligations.tsv	standing-duty source positions only	N:StringMatchQ | StringMatchQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:549 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:227:9-227:37 strings :: eval_string_match_q: unresolved non-test cfg alternative or opaque import namespace	N:StringMatchQ | StringMatchQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:549 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:227:9-227:37 strings :: eval_string_match_q: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringPosition | StringPosition | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:553 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:116:9-116:38 strings :: eval_string_position: unresolved non-test cfg alternative or opaque import namespace	N:StringPosition | StringPosition | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:553 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:116:9-116:38 strings :: eval_string_position: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringSplit | StringSplit | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:559 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:125:9-125:35 strings :: eval_string_split: unresolved non-test cfg alternative or opaque import namespace	N:StringSplit | StringSplit | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:559 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:125:9-125:35 strings :: eval_string_split: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringStartsQ | StringStartsQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:560 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:96:9-96:38 strings :: eval_string_starts_q: unresolved non-test cfg alternative or opaque import namespace	N:StringStartsQ | StringStartsQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:560 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | REJECTED crates/rum-stdlib/src/catalog/strings_match.rs:96:9-96:38 strings :: eval_string_starts_q: unresolved non-test cfg alternative or opaque import namespace
obligations.tsv	standing-duty source positions only	N:StringWrap | StringWrap | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:566 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::scripting::text_shape::eval_string_wrap	N:StringWrap | StringWrap | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:566 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::scripting::text_shape::eval_string_wrap
obligations.tsv	standing-duty source positions only	N:SubsetQ | SubsetQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:570 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::predicates::collections::eval_subset_q	N:SubsetQ | SubsetQ | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:570 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::predicates::collections::eval_subset_q
obligations.tsv	standing-duty source positions only	N:TemplateApply | TemplateApply | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:592 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::template::eval_template_apply	N:TemplateApply | TemplateApply | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:592 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::template::eval_template_apply
obligations.tsv	standing-duty source positions only	N:Uncompress | Uncompress | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:631 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::encoding::eval_uncompress	N:Uncompress | Uncompress | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:631 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::encoding::eval_uncompress
obligations.tsv	standing-duty source positions only	N:Union | Union | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:635 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::structure_ops::ordering_sets::eval_union	N:Union | Union | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:635 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::structure_ops::ordering_sets::eval_union
obligations.tsv	standing-duty source positions only	N:UrlDecode | UrlDecode | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:647 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::scripting::url::eval_url_decode	N:UrlDecode | UrlDecode | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:647 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::scripting::url::eval_url_decode
obligations.tsv	standing-duty source positions only	N:UrlEncode | UrlEncode | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:648 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::scripting::url::eval_url_encode	N:UrlEncode | UrlEncode | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:648 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::scripting::url::eval_url_encode
obligations.tsv	standing-duty source positions only	N:UrlParse | UrlParse | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:649 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:102; crates/rum-runtime/src/runtime/options/core.rs:104; crates/rum-runtime/src/runtime/options/core.rs:107; crates/rum-runtime/src/runtime/options/core.rs:196 | rum_stdlib::strings::scripting::url::eval_url_parse	N:UrlParse | UrlParse | N | Pending | crates/rum-stdlib/src/catalog/memory_growth_contract.rs:649 | merged Vec<Expr> retains evaluated RuleDelayed option RHS across later option evaluation | crates/rum-runtime/src/runtime/options/core.rs:111; crates/rum-runtime/src/runtime/options/core.rs:113; crates/rum-runtime/src/runtime/options/core.rs:116; crates/rum-runtime/src/runtime/options/core.rs:205 | rum_stdlib::strings::scripting::url::eval_url_parse
```

All 68 fingerprint rows, incoming → first replay → final replay: [fingerprint-row-history.tsv](../../../.temp/brief-579/fingerprint-row-history.tsv)
```tsv
row	id	incoming_fingerprint	selected04_fingerprint	final_after_test_anchor_fingerprint
2	B01	fnv1a64:173d5bcc2e081fbd	fnv1a64:cdfa3cfdeabdbb76	fnv1a64:6154fced8241beda
3	B02	fnv1a64:23bb579466434190	fnv1a64:e8ac8d7f8bd4de6b	fnv1a64:dc5a102d3f83973b
4	B03	fnv1a64:c9e41cc83994285b	fnv1a64:66af35a9edf05578	fnv1a64:a818f3bab9228168
5	B04	fnv1a64:53a3169f6de01135	fnv1a64:080a7db8a0d5ff7e	fnv1a64:412d5d58b686fe02
6	B05	fnv1a64:071445a3d9ee0dac	fnv1a64:6390eb6902a22e4f	fnv1a64:a78d219e27a7651f
7	B06	fnv1a64:d0cd3c4f56ca9c11	fnv1a64:87ae9fc3917607a2	fnv1a64:9b62a703830810e6
8	B07	fnv1a64:97234d66005956fc	fnv1a64:5efeb5469b4b52df	fnv1a64:e1fcd2701483d1ef
9	B08	fnv1a64:d0ff4a25fe58fa67	fnv1a64:ddb2af787bff25cc	fnv1a64:b2d97f1685edf28c
10	B09	fnv1a64:57454faab623a6c9	fnv1a64:5e10d5a793ec77ea	fnv1a64:409e7151ff5d82ce
11	B10	fnv1a64:31a8d975dc335dde	fnv1a64:0ea59a35986fb42d	fnv1a64:ac28b2e188b5f309
12	B11	fnv1a64:14fac7fc87d43735	fnv1a64:c9622f15baca257e	fnv1a64:02850eb5d07b2402
13	B12	fnv1a64:1651cfa6922c3bc9	fnv1a64:1d1d55a36ff50cea	fnv1a64:ffaaf14ddb6617ce
14	I01	fnv1a64:f51f46c99ac0163d	fnv1a64:abdc27fb5775b1f6	fnv1a64:3f36e7eaeef9b55a
15	I02	fnv1a64:5e55c337d22495ac	fnv1a64:bad268fcfad8b64f	fnv1a64:fece9f321fdded1f
16	I03	fnv1a64:f62999e7a7eac489	fnv1a64:f41d2a6c0b75e1aa	fnv1a64:9fab5e77b0b14a8e
17	I04	fnv1a64:4189121d7e6afab0	fnv1a64:39ec5a951742828b	fnv1a64:e182673da3c949db
18	P01	fnv1a64:533d2b934b764f31	fnv1a64:83ef5f2289fc9a42	fnv1a64:9794e29347c03d06
19	P02	fnv1a64:08bf8ec62af68b4e	fnv1a64:0f10742d91669add	fnv1a64:c3726b7221576379
20	P03	fnv1a64:2ae6a448861006a3	fnv1a64:f07762714886de30	fnv1a64:62b898a28708d600
21	P04	fnv1a64:189675a9eaec5d90	fnv1a64:dd87ab95107dfa6b	fnv1a64:d1352e42c42cb33b
22	P05	fnv1a64:d1160645bf8d6bed	fnv1a64:2b2a2f5586cdbf26	fnv1a64:d641edca7522c20a
23	P06	fnv1a64:66d0f4771e5bd549	fnv1a64:6d9c7a73fc24a66a	fnv1a64:502a161e6795b14e
24	P070	fnv1a64:a11e90b8fcd14f49	fnv1a64:a7ea16b5da9a206a	fnv1a64:8a77b260460b2b4e
25	P080	fnv1a64:7b408e19ff1ed19e	fnv1a64:4d996ef9efa5bced	fnv1a64:54dcd2ef50bdf2c9
26	P090	fnv1a64:7d4a3b69a0411e50	fnv1a64:dad5c1f8ae18f82b	fnv1a64:b70d5d02eecefffb
27	P100	fnv1a64:a05e1fce15543905	fnv1a64:35984d9e007baa0e	fnv1a64:b7abb0e101cbaa92
28	P110	fnv1a64:1fadb96c242b5f86	fnv1a64:dfbfdfe5eba49225	fnv1a64:4c320c73a6e6e5e1
29	P120	fnv1a64:b566a878e3261887	fnv1a64:105348d639e2546c	fnv1a64:5b62939119b01d2c
30	P130	fnv1a64:3af0a6c32f0d30fc	fnv1a64:02cc0ea3c9ff2cdf	fnv1a64:85ca2bcd4337abef
31	P140	fnv1a64:d0fca66373310118	fnv1a64:520d92b9c14f4363	fnv1a64:35198d93e01e4693
32	P150	fnv1a64:09b5a9fa5c589a70	fnv1a64:7d71455aaee76f4b	fnv1a64:29dd8505f027a59b
33	P071	fnv1a64:55c07d42dffa791c	fnv1a64:a6a1018c7cb4707f	fnv1a64:1d3cb6fe31d7810f
34	P081	fnv1a64:b77ad26c88d26c69	fnv1a64:ada909145c50ab0a	fnv1a64:64387d9567e2af6e
35	P091	fnv1a64:7d18a92a9b44cbf1	fnv1a64:92f4a9d36d522c02	fnv1a64:6b904ffc20afffc6
36	P101	fnv1a64:17b0aa39598efc86	fnv1a64:d7c2d0b321082f25	fnv1a64:4434fd40dc4a82e1
37	P111	fnv1a64:e5b9702fa4c56899	fnv1a64:a32bb3f60921d77a	fnv1a64:a52dcc233f49451e
38	P121	fnv1a64:4c3d844ae579f942	fnv1a64:3248e7354f0d3b39	fnv1a64:d0889ccd2bfa7dc5
39	P131	fnv1a64:f14d3905a43432b5	fnv1a64:a5b4a01ed72a20fe	fnv1a64:ded77fbeecdb1f82
40	P141	fnv1a64:7e7fe2b01a03d4d1	fnv1a64:5b6e46d0b3075d62	fnv1a64:603e3e5321cf12a6
41	P151	fnv1a64:225bbdc818dd67c7	fnv1a64:51f262cc7a0b58ac	fnv1a64:fab80e674774e16c
42	B13	fnv1a64:5eddcfd887675811	fnv1a64:15bf334cc212c3a2	fnv1a64:29733a8cb3a4cce6
43	R01	fnv1a64:9462b87fc2feb4a0	fnv1a64:e47821cee1b7b33b	fnv1a64:c9181a76a6adcfcb
44	R02	fnv1a64:9e98b6bc1aa3aa63	fnv1a64:458e3c925dbcebf0	fnv1a64:93d89303e9e5fbc0
45	R03	fnv1a64:9b6e42af4e483654	fnv1a64:9dee18dedd3b9487	fnv1a64:e041bd372349fbf7
46	R040	fnv1a64:37767ec1d75e9140	fnv1a64:175f4337972db1db	fnv1a64:7d2a5c84dfb3136b
47	R050	fnv1a64:76ae528929c617cd	fnv1a64:92e97cd94a7d9986	fnv1a64:a4ce42feed63a16a
48	R060	fnv1a64:a49e24a41f5e0468	fnv1a64:889f4a1937d77fb3	fnv1a64:d80231a6b58a9ce3
49	R070	fnv1a64:1fe89a866a742b69	fnv1a64:1616d12e3df26a0a	fnv1a64:cca645af49846e6e
50	R080	fnv1a64:088d7b104defc3b0	fnv1a64:00f0c387e6c74b8b	fnv1a64:a886d030734e12db
51	R090	fnv1a64:21db9a929f77c2c5	fnv1a64:be39622cb1b45bce	fnv1a64:9e1dea4845f76052
52	R100	fnv1a64:8f09c11ae369d2db	fnv1a64:2bd4d9fc97c5fff8	fnv1a64:6d3e980d62f82be8
53	R110	fnv1a64:c2879e465ab1d856	fnv1a64:61a7b8e2620e1935	fnv1a64:61e59f44065923b1
54	R041	fnv1a64:451733029e8f1711	fnv1a64:fbf89676d93a82a2	fnv1a64:0fac9db6cacc8be6
55	R051	fnv1a64:0b6c697f2d6f2ad0	fnv1a64:68f7f00e3b4704ab	fnv1a64:452f8b187bfd0c7b
56	R061	fnv1a64:658f95bdb2ba4597	fnv1a64:bf3edf50d0c916bc	fnv1a64:458ad580afce55bc
57	R071	fnv1a64:8e7ce31962cd993a	fnv1a64:85acccb83b35a141	fnv1a64:a4faf7d02189a3ed
58	R081	fnv1a64:5c488922b1000e05	fnv1a64:f182b6f29c277f0e	fnv1a64:73961a359d777f92
59	R091	fnv1a64:5507f1ff13173358	fnv1a64:ddca518a561abca3	fnv1a64:a7099420a54310d3
60	R101	fnv1a64:827372883be84b28	fnv1a64:f95a9e8913885773	fnv1a64:eaec613dfe4870a3
61	R111	fnv1a64:2e5b10035c515a1f	fnv1a64:949ce4448cb96154	fnv1a64:9fa106e5570b0594
62	R12	fnv1a64:a05e4533015efa5f	fnv1a64:d9eec5250688a194	fnv1a64:f7937fc4938cc7d4
63	R13	fnv1a64:ea7e6bd334aded8b	fnv1a64:d73ab95b72ebf4e8	fnv1a64:8f88c659226e5c98
64	R14	fnv1a64:bab62e43f204f905	fnv1a64:4ff05c13dd2c6a0e	fnv1a64:d203bf56de7c6a92
65	R15	fnv1a64:0072c86724bd0701	fnv1a64:c07de74a9f0a0492	fnv1a64:215adef3dc72e616
66	O010	fnv1a64:1caee8fb52c99177	fnv1a64:3ddac74dac20981c	fnv1a64:6f20e5c52b517e1c
67	O0200	fnv1a64:cbc44837c1e82718	fnv1a64:4cd5348e10066963	fnv1a64:2fe12f682ed56c93
68	O011	fnv1a64:e726e12c2cefc8e8	fnv1a64:cb2806a145694433	fnv1a64:1a8aee2ec31c6163
69	O0210	fnv1a64:221a94bfb0cbe03f	fnv1a64:269be5fcfcffe9f4	fnv1a64:1af360713696e3b4
```

Certificate rows: promoted.tsv row 2 (C01) and row 3 (C02), columns 5 and 6 for default/no-default builds. The linked cell-change records preserve every before/after byte for both replay rounds. All other certificate fields retain their meanings.

There are 65 host rows (+1); 1487 unknown attributes (+3); 2951 captures, 3141 macros, 88 storage and 59 self-reaching rows (unchanged counts). Two macro identities were extracted into the authorized decoder/CheckAbort child. Position-only census replay affects 32 rows: 31 core reader anchors plus N:CheckAbort. All 68 contract fingerprints were regenerated initially and again after the test-only anchor relocation; each of C01 and C02 has default/no-default replayed certificate cells, retaining status, preconditions and exclusions. [certified-body-sha256.json](../../../.temp/brief-579/certified-body-sha256.json); [changed-files.json](../../../.temp/brief-579/changed-files.json).

### 3. Sites, noise floors and every measured label

P0452 (CheckAbort), P0979 (rule decoder/reader) and P1708 (ArrayFilter) repaired: **17 -> 14 predicted lost continuations; G5 closed**. No other suspension group was changed. Five source-distinct candidates were measured, one change per candidate after the initial implementation. Every result check passed before each panel. Each panel uses one label/process, stdin /dev/null, allocations first, two paired cpu_core/instructions/u samples pinned to CPU 2, N/2N slopes, one codegen unit and fresh build directories. No timing suite.


```json
{
  "measured": 5,
  "cap": 8,
  "repaired_sites": 3,
  "passing_fixtures": 100,
  "best_regressing_labels": 5,
  "best_worst_regression": 0.6434654758217695,
  "stagnant": 0,
  "candidates": [
    {
      "candidate": "candidate-01",
      "labels": 67,
      "regressions": [
        "g5-freeq-small-explicit",
        "g5-traceprint-small-default",
        "g5-testreport-options-default",
        "g5-arrayfilter-padding-default",
        "g5-arrayfilter-padding-explicit",
        "g5-arrayfilter-padding-delayed-explicit"
      ],
      "allocation_regressions": [],
      "worst": 3.1500617163296996,
      "worst_residual": 0.6777188679471191
    },
    {
      "candidate": "candidate-02",
      "labels": 67,
      "regressions": [
        "g5-count-small-explicit",
        "g5-depth-small-explicit",
        "g5-arrayfilter-padding-default",
        "g5-arrayfilter-padding-explicit",
        "g5-arrayfilter-padding-delayed",
        "g5-arrayfilter-padding-delayed-explicit",
        "g5-checkabort-explicit"
      ],
      "allocation_regressions": [],
      "worst": 0.9203305544961182,
      "worst_residual": 0.9203305544961182
    },
    {
      "candidate": "candidate-03",
      "labels": 67,
      "regressions": [
        "g5-tracedetail-small-default",
        "g5-testreport-options-default",
        "g5-arrayfilter-padding-default",
        "g5-arrayfilter-padding-explicit",
        "g5-arrayfilter-padding-delayed-explicit"
      ],
      "allocation_regressions": [],
      "worst": 3.5281918679883173,
      "worst_residual": 0.7946561825445064
    },
    {
      "candidate": "candidate-04",
      "labels": 67,
      "regressions": [
        "g5-count-small-delayed",
        "g5-arrayfilter-padding-default",
        "g5-arrayfilter-padding-explicit",
        "g5-arrayfilter-padding-delayed",
        "g5-arrayfilter-padding-delayed-explicit"
      ],
      "allocation_regressions": [],
      "worst": 1.7876777435980262,
      "worst_residual": 0.6705184729595137
    },
    {
      "candidate": "candidate-05",
      "labels": 67,
      "regressions": [
        "g5-deletecases-small-delayed",
        "g5-replace-small-default",
        "g5-replace-small-explicit",
        "g5-testreport-options-default",
        "g5-arrayfilter-padding-explicit",
        "g5-arrayfilter-padding-delayed",
        "g5-arrayfilter-padding-delayed-explicit"
      ],
      "allocation_regressions": [],
      "worst": 2.493162198468779,
      "worst_residual": 0.6434654758217695
    }
  ],
  "selected": "candidate-04",
  "stop_reason": "Five measured candidates; report residual parser/call-layout regressions at the edit fence. No additional allowed change supported by current attribution. Neither candidate cap nor two-stagnant rule reached."
}
```

Progress counters after each measured, compiling candidate:
```tsv
candidate	repaired_sites	passing_owning_tests	regressing_labels	worst_persistent_pct	best_label_count	best_worst_pct	stagnant_best_counter
candidate-01	3	100	6	0.6777188679471191	6	0.6777188679471191	0
candidate-02	3	100	7	0.9203305544961182	6	0.6777188679471191	1
candidate-03	3	100	5	0.7946561825445064	5	0.6777188679471191	0
candidate-04	3	100	5	0.6705184729595137	5	0.6705184729595137	0
candidate-05	3	100	7	0.6434654758217695	5	0.6434654758217695	0
```

Candidate changes: 01 protocol/consumers; 02 inline ArrayFilter completion; 03 isolate rule reader with inline(never); 04 own normalized working data; 05 direct explicit-padding completion helper. Candidate 04 wins by fewest persistent regressions, then smallest worst persistent regression, with flat allocation slopes. Only the two owned ArrayFilter files were restored after matching every current source hash. [selection.json](../../../.temp/brief-579/selection.json)

**noise-floor** — [noise-floor/perf/paired-results.json](../../../.temp/brief-579/noise-floor/perf/paired-results.json) (raw counts), [noise-floor/perf/checks.json](../../../.temp/brief-579/noise-floor/perf/checks.json) (results), [noise-floor/perf/slopes.tsv](../../../.temp/brief-579/noise-floor/perf/slopes.tsv).


```tsv
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g5-cases-small-default	20000	0.9669593960795253	9.0	9.0	912.0	912.0	15929.52225	15928.9831	-0.003384596170175058	15928.76605	15774.74135	-0.9669593960795253	False
g5-cases-small-delayed	20000	0.39080178944745914	8.0	8.0	656.0	656.0	14842.53695	14784.53205	-0.39080178944745914	14779.90065	14828.0691	0.3259051000454427	False
g5-cases-small-explicit	20000	0.506692673707998	11.0	11.0	1104.0	1104.0	16925.5948	16928.81685	0.019036554035900544	16944.3342	17030.1899	0.506692673707998	False
g5-count-small-default	20000	1.6071607106225017	6.0	6.0	664.0	664.0	13122.692	13333.59475	1.6071607106225017	13135.5931	13170.6341	0.26676374437937245	False
g5-count-small-delayed	20000	0.29070766129817294	5.0	5.0	408.0	408.0	12010.847	12004.8634	-0.04981830174007884	12015.7652	11980.83445	-0.29070766129817294	False
g5-count-small-explicit	20000	0.40224696589561404	8.0	8.0	824.0	824.0	14014.48915	13967.33625	-0.3364582147469819	13937.4078	13993.4706	0.40224696589561404	False
g5-memberq-small-default	20000	0.4195932096378585	6.0	6.0	760.0	760.0	11434.1578	11386.2814	-0.4187138295397741	11434.2532	11386.27585	-0.4195932096378585	False
g5-memberq-small-delayed	20000	0.9626978699735278	5.0	5.0	504.0	504.0	10347.2985	10258.0254	-0.8627672237347728	10399.24395	10299.13065	-0.9626978699735278	False
g5-memberq-small-explicit	20000	0.28486335406960883	7.0	7.0	792.0	792.0	12017.02	12008.0309	-0.07480307097766925	11973.95155	12008.06095	0.28486335406960883	False
g5-freeq-small-default	20000	0.0010925278075335498	7.0	7.0	1208.0	1208.0	12411.5834	12411.719	0.0010925278075335498	12449.6259	12449.6242	-1.365502877437308e-05	False
g5-freeq-small-delayed	20000	0.4834624434365331	6.0	6.0	952.0	952.0	11160.7842	11214.7424	0.4834624434365331	11214.68425	11188.74655	-0.23128337295809098	False
g5-freeq-small-explicit	20000	0.19906776755334032	8.0	8.0	1240.0	1240.0	13026.3262	13026.30125	-0.0001915352004555615	13026.4936	13052.42515	0.19906776755334032	False
g5-position-small-default	20000	0.6841434466906104	17.0	17.0	2136.0	2136.0	19301.29575	19433.3443	0.6841434466906104	19433.34285	19318.64625	-0.5902051998223246	False
g5-position-small-delayed	20000	0.8382295705443576	16.0	16.0	1880.0	1880.0	18178.25295	18160.21355	-0.09923615899511962	18130.3852	18282.35945	0.8382295705443576	False
g5-position-small-explicit	20000	0.5030712420343919	18.0	18.0	2168.0	2168.0	19893.65355	19993.7328	0.5030712420343919	19923.58445	19845.75205	-0.39065460432246013	False
g5-deletecases-small-default	20000	1.0469783942181787	9.0	9.0	984.0	984.0	14613.8301	14515.5695	-0.6723808839135148	14662.8575	14509.34055	-1.0469783942181787	False
g5-deletecases-small-delayed	20000	0.34498659116175556	8.0	8.0	728.0	728.0	13391.98475	13353.23515	-0.28934919448738716	13351.02615	13397.0854	0.34498659116175556	False
g5-deletecases-small-explicit	20000	1.0402054884521506	10.0	10.0	1048.0	1048.0	15237.9007	15242.5553	0.03054620247000539	15400.8849	15240.68405	-1.0402054884521506	False
g5-replace-small-default	20000	0.11718744844069207	20.0	20.0	1945.0	1945.0	20190.64355	20214.30445	0.11718744844069207	20214.2948	20214.22835	-0.0003287277674290934	False
g5-replace-small-delayed	20000	0.9037190538253026	19.0	19.0	1689.0	1689.0	19245.45015	19071.52535	-0.9037190538253026	19103.4242	19162.91285	0.31140307296322334	False
g5-replace-small-explicit	20000	0.00579554520163672	22.0	22.0	2233.0	2233.0	21266.8037	21266.82895	0.00011872964247316986	21268.9222	21267.68955	-0.00579554520163672	False
g5-level-small-default	20000	0.24139453222944418	13.0	13.0	1009.0	1009.0	13377.2914	13344.99935	-0.24139453222944418	13373.4759	13395.55405	0.16508909250736448	False
g5-level-small-delayed	20000	0.017689370435969298	12.0	12.0	753.0	753.0	12180.56235	12181.83815	0.010474064852994402	12182.4573	12180.3023	-0.017689370435969298	False
g5-level-small-explicit	20000	0.7389279368723578	13.0	13.0	1009.0	1009.0	14220.7372	14325.8182	0.7389279368723578	14228.94285	14209.7606	-0.13481149093237388	False
g5-depth-small-default	20000	0.6795577063621439	4.0	4.0	392.0	392.0	8804.41785	8864.24895	0.6795577063621439	8804.1692	8804.43975	0.0030729759259884304	False
g5-depth-small-delayed	20000	0.5556417845853701	3.0	3.0	136.0	136.0	7686.57095	7643.86115	-0.5556417845853701	7643.49895	7632.5642	-0.14305948194053286	False
g5-depth-small-explicit	20000	0.26700477241450304	4.0	4.0	392.0	392.0	9767.6344	9741.55435	-0.26700477241450304	9652.48375	9652.70765	0.002319610224676971	False
g5-trace-small-default	20000	0.790553251704873	33.0	33.0	1992.0	1992.0	27527.5126	27718.80945	0.694929661025756	27502.13215	27719.55115	0.790553251704873	False
g5-trace-small-delayed	20000	0.2308829411903801	32.0	32.0	1736.0	1736.0	26220.75485	26281.2941	0.2308829411903801	26316.31225	26263.596	-0.2003177705873216	False
g5-trace-small-explicit	20000	0.5834534965366123	36.0	36.0	2120.0	2120.0	31731.80915	31916.9495	0.5834534965366123	31636.03245	31710.34695	0.23490461427946308	False
g5-tracedetail-small-default	20000	0.257203289572272	55.0	55.0	3263.0	3263.0	36168.25825	36075.2323	-0.257203289572272	36139.72565	36059.0883	-0.22312662464828437	False
g5-tracedetail-small-delayed	20000	0.16748863579458817	54.0	54.0	3007.0	3007.0	34851.49805	34899.0701	0.1364992974814072	34983.8362	34925.24225	-0.16748863579458817	False
g5-tracedetail-small-explicit	20000	0.47625902350401095	57.0	57.0	3359.0	3359.0	39065.5188	39016.3984	-0.12573850676724785	39183.919	38997.30205	-0.47625902350401095	False
g5-traceprint-small-default	2000	0.3006013455779444	40.0	40.0	2174.0	2174.0	30793.608	30701.042	-0.3006013455779444	30604.4815	30518.9395	-0.2795080844614217	False
g5-traceprint-small-delayed	2000	0.1741494002782673	39.0	39.0	1918.0	1918.0	29448.122	29421.8705	-0.0891449037055736	29361.284	29412.4165	0.1741494002782673	False
g5-traceprint-small-explicit	2000	0.46695787436616953	43.0	43.0	2302.0	2302.0	34851.627	34881.691	0.08626283071375695	34820.1645	34982.76	0.46695787436616953	False
matchq-integer-success	20000	0.0006197752900005682	1.0	1.0	128.0	128.0	4340.23255	4340.25535	0.0005253174740760258	4340.28275	4340.25585	-0.0006197752900005682	False
matchq-integer-fail	20000	0.002067359757695897	2.0	2.0	129.0	129.0	4462.09225	4462.00705	-0.0019094181658685372	4462.2132	4462.30545	0.002067359757695897	False
cases-wide-no-match	1000	0.0013213325351646432	33.001	33.001	263247.864	263247.864	2400510.595	2400534.59	0.0009995790082983902	2400531.218	2400562.937	0.0013213325351646432	False
count-wide-no-match	1000	0.00149936097200154	30.0	30.0	262601.0	262601.0	2178861.569	2178894.238	0.00149936097200154	2178888.242	2178883.545	-0.0002155686514626609	False
position-wide-no-match	1000	0.00011471771085291849	1642.001	1642.001	576255.864	576255.864	2573272.413	2573270.796	-6.283827518327456e-05	2573273.105	2573270.153	-0.00011471771085291849	False
deletecases-wide-no-match	1000	0.0009557190257747372	24.001	24.001	247447.864	247447.864	2670240.867	2670215.347	-0.0009557190257747372	2670167.5	2670168.846	5.0408822671599296e-05	False
replace-wide-no-match	1000	0.0020920686264913613	30.001	30.001	184455.864	184455.864	1163632.956	1163608.612	-0.0020920686264913613	1163575.893	1163589.233	0.0011464658283388829	False
dot-three-no-input	20000	0.00029744466937264846	596.0	596.0	23472.0	23472.0	253105.8975	253105.14465	-0.00029744466937264846	253101.3518	253101.9959	0.0002544830343342852	False
differences-no-input	20000	0.22525974871568932	13.0	13.0	672.0	672.0	9409.39695	9408.95275	-0.004720812634007032	9417.7944	9396.5799	-0.22525974871568932	False
take-multispec-small	20000	0.049703996859062904	109.0	109.0	5824.0	5824.0	49248.4942	49247.4962	-0.002026457897263967	49224.6128	49249.0794	0.049703996859062904	False
g5-monitor-options-none	2000	0.484180295848069	9.0	9.0	953.0	953.0	18118.354	18206.0795	0.484180295848069	18209.3635	18148.164	-0.3360880790808429	False
g5-monitor-explicit-held	2000	0.4215454971300714	13.0005	13.0005	1727.856	1727.856	17836.272	17809.9745	-0.14743832119178402	17737.943	17812.7165	0.4215454971300714	False
g5-names-default	200	0.003548990232449256	6253.005	6253.005	552512.36	552512.36	3482680.76	3482804.36	0.003548990232449256	3482480.48	3482588.16	0.0030920489179742106	False
g5-testreport-options-default	2000	0.17630169501102744	54.0	54.0	5609.0	5609.0	43196.9755	43120.8185	-0.17630169501102744	43000.365	43034.9315	0.08038652695157023	False
g5-testreport-unread-default	2000	0.011762039077911624	41.0005	41.0005	4598.4	4598.4	28846.4385	28845.643	-0.0027577061202932462	28847.039	28843.646	-0.011762039077911624	False
g5-monitor-evaluationmonitor-latent	2000	0.011750636999874331	8.0	8.0	569.0	569.0	13395.0185	13393.4445	-0.011750636999874331	13394.8445	13394.566	-0.0020791581417656246	False
g5-monitor-stepmonitor-latent	2000	0.014943869191785808	8.0	8.0	569.0	569.0	13389.5025	13388.6295	-0.006520033137913384	13390.1065	13392.1075	0.014943869191785808	False
g5-names-spellingcorrection-latent	200	0.0042239529792764685	6260.005	6260.005	552632.36	552632.36	3473286.77	3473433.48	0.0042239529792764685	3473428.125	3473297.37	-0.0037644366111644345	False
g5-names-resolvecontextaliases-latent	200	0.004963871727703406	6260.005	6260.005	552632.36	552632.36	3473224.53	3473281.1	0.0016287458386665676	3473196.115	3473368.52	0.004963871727703406	False
g5-names-ignorecase-latent	200	0.010641213760009816	6260.005	6260.005	552632.36	552632.36	3470824.555	3470625.52	-0.005734516304300996	3470750.69	3470381.36	-0.010641213760009816	False
g5-testreport-sametest-latent	2000	0.34292954926229413	52.0	52.0	4969.0	4969.0	27729.5595	27727.002	-0.009223009835401808	27789.9645	27885.2645	0.34292954926229413	False
g5-testreport-timeconstraint-latent	2000	0.10886162000942612	52.0	52.0	4969.0	4969.0	27744.158	27756.245	0.043565928365896234	27697.5485	27727.7005	0.10886162000942612	False
g5-testreport-memoryconstraint-latent	2000	0.2586582377222202	52.0	52.0	4969.0	4969.0	27748.6995	27774.0245	0.09126553840839158	27762.116	27833.925	0.2586582377222202	False
g5-arrayfilter-padding-default	20000	0.048806829043201994	97.0	97.0	4200.0	4200.0	63569.89505	63538.8686	-0.048806829043201994	63545.51995	63569.25705	0.037354482296581004	False
g5-arrayfilter-padding-explicit	20000	0.0008794609380746721	92.0	92.0	3929.0	3929.0	60031.09145	60031.6194	0.0008794609380746721	60031.3812	60031.2307	-0.0002507022110687096	False
g5-arrayfilter-padding-delayed	20000	0.1376994746907334	96.0	96.0	4145.0	4145.0	63156.0231	63080.52695	-0.1195391132853052	63082.44835	63169.31255	0.1376994746907334	False
g5-arrayfilter-padding-delayed-explicit	20000	0.015592318446078224	92.0	92.0	3929.0	3929.0	60045.9132	60055.27575	0.015592318446078224	60053.01535	60045.5248	-0.012473228790166324	False
g5-checkabort-default	20000	0.623432501772947	8.0	8.0	672.0	672.0	11107.32915	11176.57585	0.623432501772947	11107.33965	11133.3819	0.2344598330528358	False
g5-checkabort-explicit	20000	0.0005583598863490025	8.0	8.0	672.0	672.0	12384.48565	12384.5548	0.0005583598863490025	12384.4565	12384.38805	-0.0005527089541712549	False
g5-checkabort-delayed	20000	0.8927247118116988	7.0	7.0	416.0	416.0	8751.52205	8829.64905	0.8927247118116988	8751.42505	8727.3203	-0.2754379985234534	False
g5-checkabort-delayed-explicit	20000	0.23929499752259886	7.0	7.0	416.0	416.0	10029.25385	10029.369	0.0011481412448333117	10029.2527	10005.2532	-0.23929499752259886	False
```

**candidate-01** — [candidate-01/perf/paired-results.json](../../../.temp/brief-579/candidate-01/perf/paired-results.json) (raw counts), [candidate-01/perf/checks.json](../../../.temp/brief-579/candidate-01/perf/checks.json) (results), [candidate-01/perf/slopes.tsv](../../../.temp/brief-579/candidate-01/perf/slopes.tsv).


```tsv
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g5-cases-small-default	20000	0.9669593960795253	9.0	9.0	912.0	912.0	16165.06335	15932.91095	-1.4361366545464271	15868.62255	15932.9599	0.4054375217337203	False
g5-cases-small-delayed	20000	0.39080178944745914	8.0	8.0	656.0	656.0	14758.3683	14782.67355	0.16468792149604283	14832.5942	14820.38705	-0.08229949417749571	False
g5-cases-small-explicit	20000	0.506692673707998	11.0	11.0	1104.0	1104.0	16940.0724	16914.3167	-0.15204008218997478	16933.8034	16895.46895	-0.22637826301917796	False
g5-count-small-default	20000	1.6071607106225017	6.0	6.0	664.0	664.0	13161.61645	13135.87145	-0.19560667261352416	13148.6287	13178.75575	0.22912693549557162	False
g5-count-small-delayed	20000	0.29070766129817294	5.0	5.0	408.0	408.0	11967.9262	11982.7984	0.12426714329170085	12011.1497	11982.21425	-0.24090491520556423	False
g5-count-small-explicit	20000	0.40224696589561404	8.0	8.0	824.0	824.0	13954.50915	14015.4697	0.4368519834321738	13980.5182	13967.4938	-0.09316106752037534	False
g5-memberq-small-default	20000	0.4195932096378585	6.0	6.0	760.0	760.0	11434.18465	11441.26455	0.061918713198316766	11530.3739	11441.34305	-0.7721419164039567	False
g5-memberq-small-delayed	20000	0.9626978699735278	5.0	5.0	504.0	504.0	10312.20435	10293.17475	-0.18453474498882905	10377.18595	10278.02305	-0.955585651811508	False
g5-memberq-small-explicit	20000	0.28486335406960883	7.0	7.0	792.0	792.0	12116.9132	11927.8155	-1.5606095123302577	12020.94955	12054.06705	0.2754982030516828	False
g5-freeq-small-default	20000	0.0010925278075335498	7.0	7.0	1208.0	1208.0	12555.5426	12449.6804	-0.8431511354993204	12525.69645	12449.67765	-0.6069027802442006	False
g5-freeq-small-delayed	20000	0.4834624434365331	6.0	6.0	952.0	952.0	11214.69155	11201.6537	-0.11625687556247044	11214.62785	11201.83265	-0.1140938439611272	False
g5-freeq-small-explicit	20000	0.19906776755334032	8.0	8.0	1240.0	1240.0	13026.4703	13100.55125	0.5686954968914248	12946.44705	13013.25725	0.5160504634358354	True
g5-position-small-default	20000	0.6841434466906104	17.0	17.0	2136.0	2136.0	19324.12875	19315.05765	-0.046941831724245375	19320.67165	19314.983	-0.029443334595458115	False
g5-position-small-delayed	20000	0.8382295705443576	16.0	16.0	1880.0	1880.0	18129.15425	18130.107	0.005255347198551696	18178.10265	18149.11995	-0.15943743171678015	False
g5-position-small-explicit	20000	0.5030712420343919	18.0	18.0	2168.0	2168.0	19893.61245	19812.48295	-0.40781683167854244	19893.66615	19897.02445	0.016881252428180282	False
g5-deletecases-small-default	20000	1.0469783942181787	9.0	9.0	984.0	984.0	14449.74745	14494.31815	0.30845314185750095	14496.88725	14494.1596	-0.018815418461637368	False
g5-deletecases-small-delayed	20000	0.34498659116175556	8.0	8.0	728.0	728.0	13353.4073	13353.94125	0.003998604910360193	13366.3825	13354.17185	-0.09135343837420873	False
g5-deletecases-small-explicit	20000	1.0402054884521506	10.0	10.0	1048.0	1048.0	15218.6447	15273.2879	0.35905431184681813	15270.8502	15224.88035	-0.3010300631460683	False
g5-replace-small-default	20000	0.11718744844069207	20.0	20.0	1945.0	1945.0	20214.2877	20176.5056	-0.18690789683378783	20166.34595	20370.4099	1.0119034479818545	False
g5-replace-small-delayed	20000	0.9037190538253026	19.0	19.0	1689.0	1689.0	19056.6046	19065.23155	0.045270131700170246	19072.47625	19065.24655	-0.03790645695521677	False
g5-replace-small-explicit	20000	0.00579554520163672	22.0	22.0	2233.0	2233.0	21266.80325	21269.52235	0.01278565456235281	21292.81815	21327.1497	0.16123535061516314	False
g5-level-small-default	20000	0.24139453222944418	13.0	13.0	1009.0	1009.0	13348.2123	13397.13035	0.36647641572198353	13345.33835	13348.925	0.026875676778925772	False
g5-level-small-delayed	20000	0.017689370435969298	12.0	12.0	753.0	753.0	12190.94935	12229.13755	0.31325041966481226	12206.4962	12179.14975	-0.22403193801018295	False
g5-level-small-explicit	20000	0.7389279368723578	13.0	13.0	1009.0	1009.0	14456.8647	14180.59555	-1.91098938623947	14229.15125	14228.7204	-0.0030279388589682377	False
g5-depth-small-default	20000	0.6795577063621439	4.0	4.0	392.0	392.0	8804.423	8988.4152	2.0897701075925035	8800.51705	8773.3961	-0.30817450663310764	False
g5-depth-small-delayed	20000	0.5556417845853701	3.0	3.0	136.0	136.0	7558.6154	7796.71645	3.1500617163296996	7617.37235	7633.3265	0.20944427115998643	False
g5-depth-small-explicit	20000	0.26700477241450304	4.0	4.0	392.0	392.0	9652.62285	9744.7479	0.9544043254523293	9725.62455	9646.5683	-0.8128655346869129	False
g5-trace-small-default	20000	0.790553251704873	33.0	33.0	1992.0	1992.0	27526.6615	27513.83055	-0.046612808458446064	27526.77725	27651.0601	0.45149800454755695	False
g5-trace-small-delayed	20000	0.2308829411903801	32.0	32.0	1736.0	1736.0	26250.62345	26308.31335	0.21976582807599065	26270.4366	26254.9336	-0.05901310372588009	False
g5-trace-small-explicit	20000	0.5834534965366123	36.0	36.0	2120.0	2120.0	31722.86305	31666.25875	-0.17843376844890457	31732.61675	31719.20265	-0.04227227809695533	False
g5-tracedetail-small-default	20000	0.257203289572272	55.0	55.0	3263.0	3263.0	36044.22555	36064.32915	0.05577481467067269	36050.88495	36063.58295	0.03522243633580224	False
g5-tracedetail-small-delayed	20000	0.16748863579458817	54.0	54.0	3007.0	3007.0	34887.10715	34844.0297	-0.12347670391468224	34869.0713	34883.1948	0.040504376725380986	False
g5-tracedetail-small-explicit	20000	0.47625902350401095	57.0	57.0	3359.0	3359.0	39020.9718	39082.74765	0.15831448359775457	39074.4818	39041.07085	-0.0855058044557433	False
g5-traceprint-small-default	2000	0.3006013455779444	40.0	40.0	2174.0	2174.0	30651.2375	30850.089	0.6487552093125126	30436.4405	30642.714	0.6777188679471191	True
g5-traceprint-small-delayed	2000	0.1741494002782673	39.0	39.0	1918.0	1918.0	29452.1465	29435.098	-0.05788542441209632	29453.5235	29411.3315	-0.14324941462435792	False
g5-traceprint-small-explicit	2000	0.46695787436616953	43.0	43.0	2302.0	2302.0	34819.444	34820.6675	0.0035138412893598314	35267.5865	34821.32	-1.2653729508822487	False
matchq-integer-success	20000	0.0006197752900005682	1.0	1.0	128.0	128.0	4340.34515	4340.2763	-0.001586279376875055	4340.33325	4340.1407	-0.004436295300591642	False
matchq-integer-fail	20000	0.002067359757695897	2.0	2.0	129.0	129.0	4462.2061	4462.25355	0.0010633753559696757	4462.3286	4462.0351	-0.006577283438957693	False
cases-wide-no-match	1000	0.0013213325351646432	33.001	33.001	263247.864	263247.864	2400502.775	2400527.451	0.0010279513215749603	2400532.873	2400766.241	0.009721508196136597	False
count-wide-no-match	1000	0.00149936097200154	30.0	30.0	262601.0	262601.0	2178872.864	2178829.502	-0.001990111525851379	2178906.937	2178888.632	-0.0008401001295177402	False
position-wide-no-match	1000	0.00011471771085291849	1642.001	1642.001	576255.864	576255.864	2573272.217	2573269.548	-0.00010372007992476284	2573272.331	2573333.243	0.0023671027456373395	False
deletecases-wide-no-match	1000	0.0009557190257747372	24.001	24.001	247447.864	247447.864	2670167.483	2670198.064	0.0011452839641989954	2670265.093	2670259.777	-0.0001990813576546735	False
replace-wide-no-match	1000	0.0020920686264913613	30.001	30.001	184455.864	184455.864	1163609.386	1163600.209	-0.0007886667218626187	1163578.546	1163580.629	0.00017901670730235963	False
dot-three-no-input	20000	0.00029744466937264846	596.0	596.0	23472.0	23472.0	253106.1139	253102.55965	-0.0014042529219104871	253103.9178	253101.16185	-0.0010888610591064385	False
differences-no-input	20000	0.22525974871568932	13.0	13.0	672.0	672.0	9418.0576	9399.45825	-0.19748605062683833	9416.75555	9399.92565	-0.17872291481539904	False
take-multispec-small	20000	0.049703996859062904	109.0	109.0	5824.0	5824.0	49224.76355	49231.3287	0.013337087934051084	49247.6497	49230.8728	-0.034066397284349126	False
g5-monitor-options-none	2000	0.484180295848069	9.0	9.0	953.0	953.0	18148.01	18257.735	0.6046117453098354	18116.457	18100.2625	-0.08939109893285613	False
g5-monitor-explicit-held	2000	0.4215454971300714	13.0005	13.0005	1727.856	1727.856	17817.9015	17818.7155	0.004568439218277653	17866.0125	17801.585	-0.3606148825878286	False
g5-names-default	200	0.003548990232449256	6253.005	6253.005	552512.36	552512.36	3482253.495	3482766.09	0.01472020921899908	3482391.485	3482590.935	0.005727385931741935	False
g5-testreport-options-default	2000	0.17630169501102744	54.0	54.0	5609.0	5609.0	42991.5205	43089.65	0.22825315052534823	43050.5985	43198.9225	0.34453411838164527	True
g5-testreport-unread-default	2000	0.011762039077911624	41.0005	41.0005	4598.4	4598.4	28845.0515	28860.1	0.052170126997341804	28843.8845	28847.7205	0.013299179588655008	False
g5-monitor-evaluationmonitor-latent	2000	0.011750636999874331	8.0	8.0	569.0	569.0	13394.8825	13394.949	0.0004964582556166164	13395.219	13395.0025	-0.001616248304703305	False
g5-monitor-stepmonitor-latent	2000	0.014943869191785808	8.0	8.0	569.0	569.0	13389.926	13392.7815	0.021325733988364348	13390.391	13392.372	0.014794190849243272	False
g5-names-spellingcorrection-latent	200	0.0042239529792764685	6260.005	6260.005	552632.36	552632.36	3473269.19	3473487.7	0.006291191037810506	3473296.675	3473286.4	-0.0002958284581300319	False
g5-names-resolvecontextaliases-latent	200	0.004963871727703406	6260.005	6260.005	552632.36	552632.36	3473464.74	3473221.66	-0.006998199728380783	3473135.215	3473670.465	0.015411147763222388	False
g5-names-ignorecase-latent	200	0.010641213760009816	6260.005	6260.005	552632.36	552632.36	3470870.525	3470800.17	-0.0020270130934973984	3470646.565	3470696.63	0.0014425266031015127	False
g5-testreport-sametest-latent	2000	0.34292954926229413	52.0	52.0	4969.0	4969.0	27703.4555	27787.522	0.3034513149451801	27764.8055	27740.0175	-0.08927849323489001	False
g5-testreport-timeconstraint-latent	2000	0.10886162000942612	52.0	52.0	4969.0	4969.0	27930.581	27925.3265	-0.01881271284689401	27782.4395	27756.6345	-0.09288241228780914	False
g5-testreport-memoryconstraint-latent	2000	0.2586582377222202	52.0	52.0	4969.0	4969.0	27773.9475	27762.7835	-0.040195942618515	27759.7495	27748.95	-0.03890344903869636	False
g5-arrayfilter-padding-default	20000	0.048806829043201994	97.0	97.0	4200.0	4200.0	63568.9694	63629.7525	0.09561756399971699	63521.5866	63603.69715	0.1292640099137543	True
g5-arrayfilter-padding-explicit	20000	0.0008794609380746721	92.0	92.0	3929.0	3929.0	60031.46005	60086.7496	0.09210095832077325	60031.04035	60086.4065	0.09222920288769476	True
g5-arrayfilter-padding-delayed	20000	0.1376994746907334	96.0	96.0	4145.0	4145.0	63088.58125	63150.12015	0.09754364225775802	63097.0503	63150.333	0.08444562740517192	False
g5-arrayfilter-padding-delayed-explicit	20000	0.015592318446078224	92.0	92.0	3929.0	3929.0	60056.7479	60100.02695	0.07206359237443838	60052.8651	60103.0089	0.08349942990479864	True
g5-checkabort-default	20000	0.623432501772947	8.0	8.0	672.0	672.0	11131.4381	11113.37865	-0.1622382466466621	11107.29355	11089.49025	-0.1602847707216637	False
g5-checkabort-explicit	20000	0.0005583598863490025	8.0	8.0	672.0	672.0	12384.33735	12530.4302	1.179658191400934	12384.4782	12336.4843	-0.3875326777998511	False
g5-checkabort-delayed	20000	0.8927247118116988	7.0	7.0	416.0	416.0	8751.39485	8759.5967	0.0937204884544629	8799.32605	8735.4612	-0.7257925168030366	False
g5-checkabort-delayed-explicit	20000	0.23929499752259886	7.0	7.0	416.0	416.0	10014.29105	10098.30675	0.8389580408689978	10077.32205	10020.3627	-0.5652230792802859	False
```

**candidate-02** — [candidate-02/perf/paired-results.json](../../../.temp/brief-579/candidate-02/perf/paired-results.json) (raw counts), [candidate-02/perf/checks.json](../../../.temp/brief-579/candidate-02/perf/checks.json) (results), [candidate-02/perf/slopes.tsv](../../../.temp/brief-579/candidate-02/perf/slopes.tsv).


```tsv
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g5-cases-small-default	20000	0.9669593960795253	9.0	9.0	912.0	912.0	15916.24395	15934.1883	0.11274236595248954	16004.7288	15934.12545	-0.44114055840797084	False
g5-cases-small-delayed	20000	0.39080178944745914	8.0	8.0	656.0	656.0	14784.6833	14767.7079	-0.1148174746495978	14782.2645	14832.1784	0.33766071497367545	False
g5-cases-small-explicit	20000	0.506692673707998	11.0	11.0	1104.0	1104.0	16946.0273	16933.4359	-0.07430296067091335	16937.71875	16885.4433	-0.3086333571337696	False
g5-count-small-default	20000	1.6071607106225017	6.0	6.0	664.0	664.0	13195.8391	13122.7669	-0.5537518262101138	13161.6879	13161.81135	0.0009379496075156979	False
g5-count-small-delayed	20000	0.29070766129817294	5.0	5.0	408.0	408.0	11982.9647	11968.0786	-0.12422718728362092	11967.9688	12054.92975	0.7266141101570867	False
g5-count-small-explicit	20000	0.40224696589561404	8.0	8.0	824.0	824.0	13967.4189	14027.37225	0.4292371441655618	13980.30305	14063.35375	0.5940550766529995	True
g5-memberq-small-default	20000	0.4195932096378585	6.0	6.0	760.0	760.0	11434.22425	11489.3252	0.48189495671295823	11482.41795	11441.27205	-0.35833828884446195	False
g5-memberq-small-delayed	20000	0.9626978699735278	5.0	5.0	504.0	504.0	10286.18645	10361.14425	0.7287229369636883	10329.21245	10269.2583	-0.5804329254550433	False
g5-memberq-small-explicit	20000	0.28486335406960883	7.0	7.0	792.0	792.0	11995.19485	11998.02775	0.023616956918370846	12121.0612	12131.15145	0.0832455989909553	False
g5-freeq-small-default	20000	0.0010925278075335498	7.0	7.0	1208.0	1208.0	12449.685	12449.71165	0.0002140616409240792	12611.74195	12449.61515	-1.2855226553378674	False
g5-freeq-small-delayed	20000	0.4834624434365331	6.0	6.0	952.0	952.0	11227.7175	11162.6296	-0.5797073180724421	11247.85535	11231.5884	-0.14462268133630385	False
g5-freeq-small-explicit	20000	0.19906776755334032	8.0	8.0	1240.0	1240.0	13052.5588	13039.486	-0.10015507457433737	13026.30245	13045.23985	0.14537816907513879	False
g5-position-small-default	20000	0.6841434466906104	17.0	17.0	2136.0	2136.0	19304.12555	19197.0268	-0.5547972101745957	19448.92615	19315.1203	-0.687985799154267	False
g5-position-small-delayed	20000	0.8382295705443576	16.0	16.0	1880.0	1880.0	18152.24435	18159.47185	0.03981601316422889	18178.33735	18142.35415	-0.19794549582392573	False
g5-position-small-explicit	20000	0.5030712420343919	18.0	18.0	2168.0	2168.0	19959.6953	19898.6547	-0.3058192977525076	19893.7562	19868.4898	-0.127006683634745	False
g5-deletecases-small-default	20000	1.0469783942181787	9.0	9.0	984.0	984.0	14509.84755	14493.9044	-0.10987813583197559	14509.70755	14464.3093	-0.3128819091877455	False
g5-deletecases-small-delayed	20000	0.34498659116175556	8.0	8.0	728.0	728.0	13299.16065	13355.9967	0.42736569243564393	13365.5148	13391.9984	0.19814874620467382	False
g5-deletecases-small-explicit	20000	1.0402054884521506	10.0	10.0	1048.0	1048.0	15225.4757	15224.92185	-0.0036376531736270756	15177.19025	15200.8809	0.15609378027003906	False
g5-replace-small-default	20000	0.11718744844069207	20.0	20.0	1945.0	1945.0	20214.3551	20195.2572	-0.09447691952340076	20358.2628	20199.4131	-0.7802713893643176	False
g5-replace-small-delayed	20000	0.9037190538253026	19.0	19.0	1689.0	1689.0	19087.98125	19050.23045	-0.19777261673494806	19065.6178	19095.24255	0.1553831106380299	False
g5-replace-small-explicit	20000	0.00579554520163672	22.0	22.0	2233.0	2233.0	21266.5356	21267.7884	0.005890945396869185	21268.78105	21267.63	-0.005411922748621567	False
g5-level-small-default	20000	0.24139453222944418	13.0	13.0	1009.0	1009.0	13289.21435	13348.04	0.44265709357003136	13348.6912	13326.68395	-0.16486447750022615	False
g5-level-small-delayed	20000	0.017689370435969298	12.0	12.0	753.0	753.0	12183.35225	12183.4119	0.0004896025229683687	12168.1709	12180.0221	0.09739508178669976	False
g5-level-small-explicit	20000	0.7389279368723578	13.0	13.0	1009.0	1009.0	14243.3476	14252.6115	0.06504018760309371	14270.1713	14227.6588	-0.2979116305352303	False
g5-depth-small-default	20000	0.6795577063621439	4.0	4.0	392.0	392.0	8817.27255	8820.33095	0.03468646321929114	8882.3577	8916.28425	0.3819543317873819	False
g5-depth-small-delayed	20000	0.5556417845853701	3.0	3.0	136.0	136.0	7703.78365	7672.506	-0.4060037433683661	7636.5343	7627.6238	-0.11668251133239993	False
g5-depth-small-explicit	20000	0.26700477241450304	4.0	4.0	392.0	392.0	9665.61955	9754.5752	0.9203305544961182	9606.6619	9646.4285	0.41394815820468	True
g5-trace-small-default	20000	0.790553251704873	33.0	33.0	1992.0	1992.0	27478.16415	27488.1198	0.0362311322752662	27526.8576	27650.81605	0.4503182012319584	False
g5-trace-small-delayed	20000	0.2308829411903801	32.0	32.0	1736.0	1736.0	26300.35765	26308.3094	0.03023437972142773	26300.6385	26305.25015	0.01753436518281415	False
g5-trace-small-explicit	20000	0.5834534965366123	36.0	36.0	2120.0	2120.0	31725.00855	31780.1254	0.17373312890722747	31770.7688	31720.60495	-0.15789309448501765	False
g5-tracedetail-small-default	20000	0.257203289572272	55.0	55.0	3263.0	3263.0	36068.6025	36064.6759	-0.010886476680094237	36037.9956	36141.3203	0.2867104517877195	False
g5-tracedetail-small-delayed	20000	0.16748863579458817	54.0	54.0	3007.0	3007.0	34869.50955	34939.6211	0.2010683571544325	34893.37545	34868.04045	-0.07260690510237922	False
g5-tracedetail-small-explicit	20000	0.47625902350401095	57.0	57.0	3359.0	3359.0	39188.28555	39010.78155	-0.45295168571108935	39045.4419	38972.7576	-0.1861530987052329	False
g5-traceprint-small-default	2000	0.3006013455779444	40.0	40.0	2174.0	2174.0	30700.898	30701.894	0.0032442047786407358	30840.1245	30626.277	-0.6934067338152383	False
g5-traceprint-small-delayed	2000	0.1741494002782673	39.0	39.0	1918.0	1918.0	29482.515	29352.531	-0.4408850466115277	29397.875	29434.981	0.12622000739850314	False
g5-traceprint-small-explicit	2000	0.46695787436616953	43.0	43.0	2302.0	2302.0	34665.5745	34698.62	0.09532656093726999	34820.5845	34817.722	-0.008220712090556415	False
matchq-integer-success	20000	0.0006197752900005682	1.0	1.0	128.0	128.0	4340.3759	4340.30595	-0.001611611565721649	4340.04195	4340.22385	0.004191203727899051	False
matchq-integer-fail	20000	0.002067359757695897	2.0	2.0	129.0	129.0	4462.04065	4462.2185	0.00398584445886474	4462.10815	4462.2465	0.0031005523700722293	False
cases-wide-no-match	1000	0.0013213325351646432	33.001	33.001	263247.864	263247.864	2400532.955	2400509.652	-0.0009707427657601464	2400531.838	2400587.654	0.002325151415050186	False
count-wide-no-match	1000	0.00149936097200154	30.0	30.0	262601.0	262601.0	2178886.645	2178859.568	-0.0012426988830416263	2179154.375	2178888.054	-0.012221300292225568	False
position-wide-no-match	1000	0.00011471771085291849	1642.001	1642.001	576255.864	576255.864	2573273.467	2573317.263	0.0017019566929565855	2573212.788	2573219.444	0.0002586649666458385	False
deletecases-wide-no-match	1000	0.0009557190257747372	24.001	24.001	247447.864	247447.864	2670141.089	2670212.362	0.002669259699183968	2670172.597	2670161.519	-0.0004148795479608225	False
replace-wide-no-match	1000	0.0020920686264913613	30.001	30.001	184455.864	184455.864	1163605.893	1163577.996	-0.0023974612166965947	1163605.903	1163622.936	0.0014638117558751773	False
dot-three-no-input	20000	0.00029744466937264846	596.0	596.0	23472.0	23472.0	253108.619	253105.6926	-0.0011561834644568947	253108.39585	253104.8174	-0.001413801382599278	False
differences-no-input	20000	0.22525974871568932	13.0	13.0	672.0	672.0	9418.0469	9399.3676	-0.19833517711618232	9410.8753	9399.4694	-0.12119914074304461	False
take-multispec-small	20000	0.049703996859062904	109.0	109.0	5824.0	5824.0	49239.5579	49230.9667	-0.01744776022857275	49248.70055	49230.99105	-0.03595932441309291	False
g5-monitor-options-none	2000	0.484180295848069	9.0	9.0	953.0	953.0	18146.4105	18162.417	0.08820752732336956	18290.9355	18082.414	-1.140026435498609	False
g5-monitor-explicit-held	2000	0.4215454971300714	13.0005	13.0005	1727.856	1727.856	17922.524	17896.469	-0.1453757294452518	17816.4455	17864.5415	0.2699528365520454	False
g5-names-default	200	0.003548990232449256	6253.005	6253.005	552512.36	552512.36	3482616.26	3482671.025	0.0015725246743203414	3482598.28	3482526.235	-0.0020687140521946645	False
g5-testreport-options-default	2000	0.17630169501102744	54.0	54.0	5609.0	5609.0	43076.9205	43019.0915	-0.13424590088791932	43048.551	43045.193	-0.0078004948412813135	False
g5-testreport-unread-default	2000	0.011762039077911624	41.0005	41.0005	4598.4	4598.4	28846.2	28854.6855	0.029416352933830936	28848.017	28847.8095	-0.0007192868750727222	False
g5-monitor-evaluationmonitor-latent	2000	0.011750636999874331	8.0	8.0	569.0	569.0	13396.2945	13394.426	-0.013947886857823555	13491.31	13394.993	-0.7139188114423267	False
g5-monitor-stepmonitor-latent	2000	0.014943869191785808	8.0	8.0	569.0	569.0	13437.614	13390.548	-0.35025563318010766	13390.7895	13391.124	0.002497985649019263	False
g5-names-spellingcorrection-latent	200	0.0042239529792764685	6260.005	6260.005	552632.36	552632.36	3473584.78	3473364.67	-0.0063366813807808064	3473493.855	3473485.825	-0.00023117933513061573	False
g5-names-resolvecontextaliases-latent	200	0.004963871727703406	6260.005	6260.005	552632.36	552632.36	3473227.785	3473255.2	0.0007893234102995805	3473591.43	3473327.075	-0.007610422967907482	False
g5-names-ignorecase-latent	200	0.010641213760009816	6260.005	6260.005	552632.36	552632.36	3470721.105	3470691.295	-0.0008588993208658025	3470999.27	3470846.155	-0.004411265692949584	False
g5-testreport-sametest-latent	2000	0.34292954926229413	52.0	52.0	4969.0	4969.0	27764.549	27739.7365	-0.08936756004932489	27739.3005	27739.874	0.0020674638136597423	False
g5-testreport-timeconstraint-latent	2000	0.10886162000942612	52.0	52.0	4969.0	4969.0	27783.199	27755.0775	-0.10121764595935057	27744.526	27743.608	-0.003308760798437227	False
g5-testreport-memoryconstraint-latent	2000	0.2586582377222202	52.0	52.0	4969.0	4969.0	27760.7245	27747.8265	-0.04646132344277021	27736.164	27788.3185	0.1880378988240894	False
g5-arrayfilter-padding-default	20000	0.048806829043201994	97.0	97.0	4200.0	4200.0	63520.73885	63646.96225	0.19871210928144034	63443.6503	63648.21065	0.3224284054160087	True
g5-arrayfilter-padding-explicit	20000	0.0008794609380746721	92.0	92.0	3929.0	3929.0	60031.31525	60105.70155	0.12391249415444339	60031.12715	60105.48985	0.12387356948035944	True
g5-arrayfilter-padding-delayed	20000	0.1376994746907334	96.0	96.0	4145.0	4145.0	63095.17945	63191.8687	0.15324348205811855	63123.1334	63216.13395	0.14733196055189968	True
g5-arrayfilter-padding-delayed-explicit	20000	0.015592318446078224	92.0	92.0	3929.0	3929.0	60046.02565	60119.48855	0.12234431705473447	60051.97115	60119.2326	0.1120053991766623	True
g5-checkabort-default	20000	0.623432501772947	8.0	8.0	672.0	672.0	11107.41935	11076.5634	-0.27779585003244733	11137.5019	11126.44855	-0.09924442751385598	False
g5-checkabort-explicit	20000	0.0005583598863490025	8.0	8.0	672.0	672.0	12347.60945	12386.5901	0.31569390138104225	12325.3818	12349.5377	0.19598500388848805	True
g5-checkabort-delayed	20000	0.8927247118116988	7.0	7.0	416.0	416.0	8751.41305	8759.57165	0.09322608764308349	8751.58705	8741.58655	-0.11427070247790327	False
g5-checkabort-delayed-explicit	20000	0.23929499752259886	7.0	7.0	416.0	416.0	10029.2074	10020.41985	-0.08761958597046648	10029.11625	10068.23875	0.3900892065140882	False
```

**candidate-03** — [candidate-03/perf/paired-results.json](../../../.temp/brief-579/candidate-03/perf/paired-results.json) (raw counts), [candidate-03/perf/checks.json](../../../.temp/brief-579/candidate-03/perf/checks.json) (results), [candidate-03/perf/slopes.tsv](../../../.temp/brief-579/candidate-03/perf/slopes.tsv).


```tsv
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g5-cases-small-default	20000	0.9669593960795253	9.0	9.0	912.0	912.0	15899.5067	15991.44255	0.5782308327842545	15929.24425	15817.497	-0.7015226099003402	False
g5-cases-small-delayed	20000	0.39080178944745914	8.0	8.0	656.0	656.0	14782.21765	14824.3186	0.28480807817086173	14784.4753	14783.66695	-0.005467559609639849	False
g5-cases-small-explicit	20000	0.506692673707998	11.0	11.0	1104.0	1104.0	16946.6853	16914.4234	-0.1903729220722683	16906.65885	16907.7182	0.0062658743480881185	False
g5-count-small-default	20000	1.6071607106225017	6.0	6.0	664.0	664.0	13231.58305	13148.61295	-0.6270610227549378	13147.6289	13257.6081	0.8364945560640269	False
g5-count-small-delayed	20000	0.29070766129817294	5.0	5.0	408.0	408.0	11980.92135	11993.8338	0.10777510028474602	11961.78385	11986.98435	0.21067509926624428	False
g5-count-small-explicit	20000	0.40224696589561404	8.0	8.0	824.0	824.0	13971.39405	14145.453	1.2458237837762454	14053.4506	13937.40255	-0.8257619662462079	False
g5-memberq-small-default	20000	0.4195932096378585	6.0	6.0	760.0	760.0	11434.32495	11501.4259	0.5868378788727702	11590.2259	11525.24945	-0.5606141809539666	False
g5-memberq-small-delayed	20000	0.9626978699735278	5.0	5.0	504.0	504.0	10363.27305	10323.18105	-0.3868661937842188	10312.18145	10278.225	-0.32928483817553644	False
g5-memberq-small-explicit	20000	0.28486335406960883	7.0	7.0	792.0	792.0	12048.96065	12015.20375	-0.2801644140152493	11990.9862	12414.0512	3.5281918679883173	False
g5-freeq-small-default	20000	0.0010925278075335498	7.0	7.0	1208.0	1208.0	12449.51445	12449.8746	0.0028928839068020906	12449.6682	12509.65575	0.48184055218434274	False
g5-freeq-small-delayed	20000	0.4834624434365331	6.0	6.0	952.0	952.0	11333.8016	11201.8562	-1.1641760166332893	11198.708	11201.78415	0.02746879372155231	False
g5-freeq-small-explicit	20000	0.19906776755334032	8.0	8.0	1240.0	1240.0	13003.20945	13061.3873	0.44741146579008184	13080.30635	13013.45055	-0.5111179983945946	False
g5-position-small-default	20000	0.6841434466906104	17.0	17.0	2136.0	2136.0	19446.94415	19315.1035	-0.6779504737766118	19425.136	19251.7993	-0.8923319764659543	False
g5-position-small-delayed	20000	0.8382295705443576	16.0	16.0	1880.0	1880.0	18130.2458	18129.87975	-0.0020190018604204774	18130.08515	18130.0491	-0.00019884076495335634	False
g5-position-small-explicit	20000	0.5030712420343919	18.0	18.0	2168.0	2168.0	19863.63875	19898.0012	0.17299171834768146	19918.8356	19921.35265	0.012636531826193398	False
g5-deletecases-small-default	20000	1.0469783942181787	9.0	9.0	984.0	984.0	14619.8965	14494.4248	-0.8582256379174757	14511.3163	14539.00435	0.190803159600339	False
g5-deletecases-small-delayed	20000	0.34498659116175556	8.0	8.0	728.0	728.0	13398.58155	13375.99505	-0.16857381444232988	13368.34465	13469.9296	0.759891764160936	False
g5-deletecases-small-explicit	20000	1.0402054884521506	10.0	10.0	1048.0	1048.0	15240.84885	15285.3707	0.29212185251741296	15303.88045	15225.2438	-0.5138347117707665	False
g5-replace-small-default	20000	0.11718744844069207	20.0	20.0	1945.0	1945.0	20214.5021	20213.8308	-0.0033208831792141957	20190.44825	20214.20625	0.11766950245890584	False
g5-replace-small-delayed	20000	0.9037190538253026	19.0	19.0	1689.0	1689.0	19072.6657	19065.36375	-0.03828489480629527	19065.4139	19065.0677	-0.0018158535755707916	False
g5-replace-small-explicit	20000	0.00579554520163672	22.0	22.0	2233.0	2233.0	21268.64425	21269.11545	0.0022154679652386378	21301.6859	21267.5858	-0.1600816956933926	False
g5-level-small-default	20000	0.24139453222944418	13.0	13.0	1009.0	1009.0	13345.3011	13322.98565	-0.16721578503762213	13260.04655	13469.32725	1.5782802813765429	False
g5-level-small-delayed	20000	0.017689370435969298	12.0	12.0	753.0	753.0	12181.1606	12183.347	0.017949028600772188	12181.09495	12183.0009	0.01564678715519907	False
g5-level-small-explicit	20000	0.7389279368723578	13.0	13.0	1009.0	1009.0	14233.841	14229.3802	-0.03133939742617775	14207.45005	14229.7752	0.1571369240886522	False
g5-depth-small-default	20000	0.6795577063621439	4.0	4.0	392.0	392.0	8791.24935	8868.3901	0.8774719829781619	8774.46745	8820.35435	0.522959373448928	False
g5-depth-small-delayed	20000	0.5556417845853701	3.0	3.0	136.0	136.0	7575.40275	7707.6803	1.7461454442141644	7630.6758	7642.592	0.15616179106967554	False
g5-depth-small-explicit	20000	0.26700477241450304	4.0	4.0	392.0	392.0	9678.654	9681.6902	0.03137006447382351	9689.6806	9707.8134	0.1871351672830146	False
g5-trace-small-default	20000	0.790553251704873	33.0	33.0	1992.0	1992.0	27718.66795	27536.021	-0.6589311951406329	27525.53395	27535.55825	0.036418185449949014	False
g5-trace-small-delayed	20000	0.2308829411903801	32.0	32.0	1736.0	1736.0	26245.76265	26497.12575	0.9577283135264514	26261.3525	26288.67215	0.10402986670239311	False
g5-trace-small-explicit	20000	0.5834534965366123	36.0	36.0	2120.0	2120.0	31723.3407	31484.03505	-0.7543519841212709	31820.8929	31912.1111	0.2866613463257117	False
g5-tracedetail-small-default	20000	0.257203289572272	55.0	55.0	3263.0	3263.0	35978.43275	36111.9638	0.3711419308557895	35987.8333	36111.8519	0.34461257771805087	True
g5-tracedetail-small-delayed	20000	0.16748863579458817	54.0	54.0	3007.0	3007.0	34976.7783	34861.1293	-0.33064508974515183	34929.1106	34882.4798	-0.1335012521045953	False
g5-tracedetail-small-explicit	20000	0.47625902350401095	57.0	57.0	3359.0	3359.0	39016.28115	39058.72735	0.10879099378233725	39015.85705	38986.69255	-0.07475037639855797	False
g5-traceprint-small-default	2000	0.3006013455779444	40.0	40.0	2174.0	2174.0	30614.179	30642.098	0.09119630482332663	30627.1575	30640.868	0.04476582588508471	False
g5-traceprint-small-delayed	2000	0.1741494002782673	39.0	39.0	1918.0	1918.0	29423.1395	29384.397	-0.13167357616613584	29422.5405	29454.804	0.10965572466457019	False
g5-traceprint-small-explicit	2000	0.46695787436616953	43.0	43.0	2302.0	2302.0	34916.862	34879.82	-0.10608628003284659	34819.783	34820.106	0.0009276335811714276	False
matchq-integer-success	20000	0.0006197752900005682	1.0	1.0	128.0	128.0	4340.28945	4340.08615	-0.004684019403367046	4340.2489	4287.34415	-1.218933550101231	False
matchq-integer-fail	20000	0.002067359757695897	2.0	2.0	129.0	129.0	4462.0539	4462.0287	-0.0005647623396054513	4462.24955	4462.23235	-0.00038545580670623636	False
cases-wide-no-match	1000	0.0013213325351646432	33.001	33.001	263247.864	263247.864	2400529.358	2400533.473	0.00017142052384233608	2400531.727	2400505.215	-0.0011044219787592624	False
count-wide-no-match	1000	0.00149936097200154	30.0	30.0	262601.0	262601.0	2178938.985	2178891.484	-0.00218000597201895	2178858.254	2178921.575	0.002906155087600837	False
position-wide-no-match	1000	0.00011471771085291849	1642.001	1642.001	576255.864	576255.864	2573217.558	2573268.94	0.001996799681402983	2573373.352	2573209.086	-0.006383294513878646	False
deletecases-wide-no-match	1000	0.0009557190257747372	24.001	24.001	247447.864	247447.864	2670028.261	2670163.136	0.00505144465958729	2670170.078	2670166.291	-0.0001418261717178737	False
replace-wide-no-match	1000	0.0020920686264913613	30.001	30.001	184455.864	184455.864	1163591.82	1163536.464	-0.004757338359440144	1163579.747	1163602.398	0.0019466650273303188	False
dot-three-no-input	20000	0.00029744466937264846	596.0	596.0	23472.0	23472.0	253108.17445	253105.8475	-0.0009193499992843357	253101.6234	253102.26955	0.000255292712592059	False
differences-no-input	20000	0.22525974871568932	13.0	13.0	672.0	672.0	9396.6014	9398.5431	0.020663854061120368	9409.3813	9398.9228	-0.11114970970512728	False
take-multispec-small	20000	0.049703996859062904	109.0	109.0	5824.0	5824.0	49239.29775	49231.23985	-0.01636477441435691	49239.6203	49231.37745	-0.01674027937214051	False
g5-monitor-options-none	2000	0.484180295848069	9.0	9.0	953.0	953.0	18242.5065	18185.912	-0.3102342323408225	18120.856	18123.0315	0.01200550349278462	False
g5-monitor-explicit-held	2000	0.4215454971300714	13.0005	13.0005	1727.856	1727.856	17826.301	17802.088	-0.13582739346765793	17872.9035	17817.2175	-0.3115666125540373	False
g5-names-default	200	0.003548990232449256	6253.005	6253.005	552512.36	552512.36	3482452.59	3482482.54	0.0008600260657187775	3483221.505	3482515.115	-0.020279789814847682	False
g5-testreport-options-default	2000	0.17630169501102744	54.0	54.0	5609.0	5609.0	43141.877	43378.343	0.5481124523163405	42919.505	43260.5675	0.7946561825445064	True
g5-testreport-unread-default	2000	0.011762039077911624	41.0005	41.0005	4598.4	4598.4	28845.641	28857.9395	0.04263555800336771	28802.5985	28831.16	0.0991629279559536	False
g5-monitor-evaluationmonitor-latent	2000	0.011750636999874331	8.0	8.0	569.0	569.0	13393.701	13394.7425	0.007776043380403053	13395.413	13394.741	-0.005016642637301061	False
g5-monitor-stepmonitor-latent	2000	0.014943869191785808	8.0	8.0	569.0	569.0	13391.4755	13389.725	-0.013071748516435555	13390.945	13610.116	1.6367104786107367	False
g5-names-spellingcorrection-latent	200	0.0042239529792764685	6260.005	6260.005	552632.36	552632.36	3473425.14	3473320.075	-0.0030248240789787495	3473451.08	3473534.73	0.002408267687470378	False
g5-names-resolvecontextaliases-latent	200	0.004963871727703406	6260.005	6260.005	552632.36	552632.36	3473454.615	3473386.24	-0.00196850132155868	3473269.825	3473451.97	0.005244193776388073	False
g5-names-ignorecase-latent	200	0.010641213760009816	6260.005	6260.005	552632.36	552632.36	3470778.185	3470781.56	9.724044061076853e-05	3470710.64	3470910.67	0.005763373001888006	False
g5-testreport-sametest-latent	2000	0.34292954926229413	52.0	52.0	4969.0	4969.0	27767.2945	27741.5965	-0.09254772732719685	27740.393	27755.7765	0.055455234538315246	False
g5-testreport-timeconstraint-latent	2000	0.10886162000942612	52.0	52.0	4969.0	4969.0	27901.714	27669.528	-0.8321567628426063	27708.382	27683.184	-0.09093999064976321	False
g5-testreport-memoryconstraint-latent	2000	0.2586582377222202	52.0	52.0	4969.0	4969.0	27764.377	27715.9355	-0.17447357093588467	27904.7725	27891.329	-0.048176346895489885	False
g5-arrayfilter-padding-default	20000	0.048806829043201994	97.0	97.0	4200.0	4200.0	63569.6697	63688.19315	0.1864465405583271	63569.6547	63688.3977	0.1867919537401619	True
g5-arrayfilter-padding-explicit	20000	0.0008794609380746721	92.0	92.0	3929.0	3929.0	60030.8932	60110.67165	0.1328956571314155	60030.9929	60110.85545	0.13303553071835061	True
g5-arrayfilter-padding-delayed	20000	0.1376994746907334	96.0	96.0	4145.0	4145.0	63115.6671	63256.1978	0.22265581028770853	63157.16485	63208.11525	0.08067239896061285	False
g5-arrayfilter-padding-delayed-explicit	20000	0.015592318446078224	92.0	92.0	3929.0	3929.0	60041.446	60124.28835	0.13797527461281156	60050.4457	60126.631	0.1268688335480661	True
g5-checkabort-default	20000	0.623432501772947	8.0	8.0	672.0	672.0	11083.3305	11142.30685	0.5321175796390909	11137.56085	11081.3994	-0.5042526883253795	False
g5-checkabort-explicit	20000	0.0005583598863490025	8.0	8.0	672.0	672.0	12432.60905	12437.40855	0.03860412549530068	12410.3664	12533.419	0.9915307577059052	False
g5-checkabort-delayed	20000	0.8927247118116988	7.0	7.0	416.0	416.0	8751.567	8836.27015	0.967862669622499	8869.36755	8788.3072	-0.913936078790667	False
g5-checkabort-delayed-explicit	20000	0.23929499752259886	7.0	7.0	416.0	416.0	10029.14635	10025.24655	-0.03888466539329061	10029.4403	10097.3716	0.6773189526837253	False
```

**candidate-04** — [candidate-04/perf/paired-results.json](../../../.temp/brief-579/candidate-04/perf/paired-results.json) (raw counts), [candidate-04/perf/checks.json](../../../.temp/brief-579/candidate-04/perf/checks.json) (results), [candidate-04/perf/slopes.tsv](../../../.temp/brief-579/candidate-04/perf/slopes.tsv).


```tsv
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g5-cases-small-default	20000	0.9669593960795253	9.0	9.0	912.0	912.0	15929.2154	16009.8461	0.5061812397866294	15929.18825	15934.3157	0.032189022563655634	False
g5-cases-small-delayed	20000	0.39080178944745914	8.0	8.0	656.0	656.0	14765.0373	14691.7953	-0.4960502199340855	14779.98535	14830.2511	0.34009336822515834	False
g5-cases-small-explicit	20000	0.506692673707998	11.0	11.0	1104.0	1104.0	16939.50915	16875.42035	-0.37833917991656474	16942.17595	17037.2099	0.5609311949094753	False
g5-count-small-default	20000	1.6071607106225017	6.0	6.0	664.0	664.0	13135.5921	13172.6442	0.28207407567109044	13148.64585	13148.8263	0.0013723846703284082	False
g5-count-small-delayed	20000	0.29070766129817294	5.0	5.0	408.0	408.0	11956.8145	12036.98715	0.6705184729595137	11980.90675	12023.97315	0.3594586027472424	True
g5-count-small-explicit	20000	0.40224696589561404	8.0	8.0	824.0	824.0	13941.34155	13967.2579	0.1858956679818391	13993.44695	14115.46755	0.8719838681347847	False
g5-memberq-small-default	20000	0.4195932096378585	6.0	6.0	760.0	760.0	11410.2874	11441.36465	0.2723616760082592	11504.5008	11415.3717	-0.774732442106485	False
g5-memberq-small-delayed	20000	0.9626978699735278	5.0	5.0	504.0	504.0	10249.2106	10293.0552	0.4277851408380773	10299.2112	10269.0835	-0.2925243440002334	False
g5-memberq-small-explicit	20000	0.28486335406960883	7.0	7.0	792.0	792.0	12020.9899	12005.9327	-0.12525757134195237	12080.93505	12016.08375	-0.5368069584978041	False
g5-freeq-small-default	20000	0.0010925278075335498	7.0	7.0	1208.0	1208.0	12479.90205	12467.6299	-0.09833530704674853	12509.6258	12449.7235	-0.47884965511917343	False
g5-freeq-small-delayed	20000	0.4834624434365331	6.0	6.0	952.0	952.0	11212.58325	11201.73265	-0.09677163378029041	11218.79015	11249.58495	0.2744930566332071	False
g5-freeq-small-explicit	20000	0.19906776755334032	8.0	8.0	1240.0	1240.0	13002.3226	13112.27285	0.8456200740627695	13026.3819	12905.45975	-0.9282865413304031	False
g5-position-small-default	20000	0.6841434466906104	17.0	17.0	2136.0	2136.0	19258.5792	19315.16685	0.2938308657785216	19447.62485	19315.19055	-0.6809793022102717	False
g5-position-small-delayed	20000	0.8382295705443576	16.0	16.0	1880.0	1880.0	18104.2226	18130.16985	0.14332153649059265	18149.9533	18178.5753	0.15769737545274776	False
g5-position-small-explicit	20000	0.5030712420343919	18.0	18.0	2168.0	2168.0	19863.72375	19873.1278	0.047342835202268674	19882.61655	19898.4794	0.07978250729780978	False
g5-deletecases-small-default	20000	1.0469783942181787	9.0	9.0	984.0	984.0	14457.97795	14444.40235	-0.09389694774019075	14483.47915	14412.19335	-0.4921869894775943	False
g5-deletecases-small-delayed	20000	0.34498659116175556	8.0	8.0	728.0	728.0	13370.4309	13386.98845	0.12383707095036112	13442.248	13432.28485	-0.07411818320863794	False
g5-deletecases-small-explicit	20000	1.0402054884521506	10.0	10.0	1048.0	1048.0	15237.8595	15176.9298	-0.3998573421680418	15185.81925	15194.9584	0.0601821334071273	False
g5-replace-small-default	20000	0.11718744844069207	20.0	20.0	1945.0	1945.0	20374.3813	20188.59665	-0.9118541921074308	20315.0319	20214.12725	-0.4966994415598225	False
g5-replace-small-delayed	20000	0.9037190538253026	19.0	19.0	1689.0	1689.0	19072.7244	19039.568	-0.1738419708932626	19065.3651	19065.21575	-0.0007833576709193046	False
g5-replace-small-explicit	20000	0.00579554520163672	22.0	22.0	2233.0	2233.0	21268.6336	21266.99795	-0.007690432919960077	21251.69565	21267.7095	0.0753532812804103	False
g5-level-small-default	20000	0.24139453222944418	13.0	13.0	1009.0	1009.0	13310.6741	13270.5878	-0.30115905249307184	13300.48365	13271.118	-0.22078633208199028	False
g5-level-small-delayed	20000	0.017689370435969298	12.0	12.0	753.0	753.0	12183.14225	12181.03035	-0.017334608401209373	12215.3513	12218.86335	0.028751117456593533	False
g5-level-small-explicit	20000	0.7389279368723578	13.0	13.0	1009.0	1009.0	14237.65815	14179.94825	-0.4053328110002363	14228.9856	14153.79675	-0.5284203112834729	False
g5-depth-small-default	20000	0.6795577063621439	4.0	4.0	392.0	392.0	8804.3978	8790.4613	-0.158290212648049	9121.42255	8820.3181	-3.3010689763517065	False
g5-depth-small-delayed	20000	0.5556417845853701	3.0	3.0	136.0	136.0	7643.54045	7676.6272	0.4328720468797842	7630.75865	7609.6003	-0.27727714858337693	False
g5-depth-small-explicit	20000	0.26700477241450304	4.0	4.0	392.0	392.0	9665.5313	9707.45685	0.4337635324816569	9717.7139	9691.564	-0.2690951829730226	False
g5-trace-small-default	20000	0.790553251704873	33.0	33.0	1992.0	1992.0	27356.89905	27726.5487	1.3512118070267976	27525.8156	27530.40125	0.01665945186379858	False
g5-trace-small-delayed	20000	0.2308829411903801	32.0	32.0	1736.0	1736.0	26300.4515	26305.9466	0.020893557663836937	26305.57765	26305.71515	0.0005227028344823026	False
g5-trace-small-explicit	20000	0.5834534965366123	36.0	36.0	2120.0	2120.0	31705.8862	31731.9292	0.08213932213003705	31723.3244	31720.5854	-0.008634025758036401	False
g5-tracedetail-small-default	20000	0.257203289572272	55.0	55.0	3263.0	3263.0	36176.16795	36182.18435	0.01663083831409118	36182.5707	36063.7645	-0.32835201507669387	False
g5-tracedetail-small-delayed	20000	0.16748863579458817	54.0	54.0	3007.0	3007.0	34977.1565	34863.91185	-0.3237674566255899	34839.07345	34868.02355	0.083096641595648	False
g5-tracedetail-small-explicit	20000	0.47625902350401095	57.0	57.0	3359.0	3359.0	38992.62845	39010.90455	0.04687065408641722	39020.8493	39011.06105	-0.025084666724584093	False
g5-traceprint-small-default	2000	0.3006013455779444	40.0	40.0	2174.0	2174.0	30513.328	30688.7225	0.5748127506773448	30640.0445	30611.691	-0.09253739824039986	False
g5-traceprint-small-delayed	2000	0.1741494002782673	39.0	39.0	1918.0	1918.0	29624.303	29639.41	0.05099529261498592	29406.1665	29435.6435	0.10024087974880302	False
g5-traceprint-small-explicit	2000	0.46695787436616953	43.0	43.0	2302.0	2302.0	34917.1545	34820.6945	-0.2762538969204975	34759.513	34819.4865	0.17253837819879436	False
matchq-integer-success	20000	0.0006197752900005682	1.0	1.0	128.0	128.0	4340.1699	4340.24685	0.0017729720673154148	4340.174	4340.3806	0.004760177817764166	False
matchq-integer-fail	20000	0.002067359757695897	2.0	2.0	129.0	129.0	4462.13425	4462.158	0.0005322565093113241	4462.1473	4462.26255	0.0025828371914338177	False
cases-wide-no-match	1000	0.0013213325351646432	33.001	33.001	263247.864	263247.864	2400420.736	2400560.186	0.005809398240441688	2400510.568	2400511.225	2.7369177568736802e-05	False
count-wide-no-match	1000	0.00149936097200154	30.0	30.0	262601.0	262601.0	2178892.42	2178938.493	0.0021145146762213685	2178866.85	2178817.131	-0.002281874176934817	False
position-wide-no-match	1000	0.00011471771085291849	1642.001	1642.001	576255.864	576255.864	2573270.401	2573393.619	0.004788381351295001	2573321.337	2573271.892	-0.001921446781205649	False
deletecases-wide-no-match	1000	0.0009557190257747372	24.001	24.001	247447.864	247447.864	2670169.264	2670166.138	-0.00011707122998805275	2670218.396	2670164.896	-0.002003581432896695	False
replace-wide-no-match	1000	0.0020920686264913613	30.001	30.001	184455.864	184455.864	1163639.523	1163559.611	-0.006867418854417107	1163578.837	1163698.516	0.010285422542444245	False
dot-three-no-input	20000	0.00029744466937264846	596.0	596.0	23472.0	23472.0	253106.23205	253106.2095	-8.90930255126321e-06	253100.93475	253102.20895	0.000503435517251738	False
differences-no-input	20000	0.22525974871568932	13.0	13.0	672.0	672.0	9409.5775	9399.88765	-0.10297858750829558	9411.9951	9399.28235	-0.13506966232909035	False
take-multispec-small	20000	0.049703996859062904	109.0	109.0	5824.0	5824.0	49239.4632	49230.98195	-0.017224497280865503	49224.31905	49231.2243	0.014028127017851055	False
g5-monitor-options-none	2000	0.484180295848069	9.0	9.0	953.0	953.0	18193.4525	18134.2415	-0.3254522471751842	18407.236	18160.042	-1.3429175352562384	False
g5-monitor-explicit-held	2000	0.4215454971300714	13.0005	13.0005	1727.856	1727.856	17833.392	17793.1255	-0.22579271514920896	17855.8785	17817.282	-0.2161557046885143	False
g5-names-default	200	0.003548990232449256	6253.005	6253.005	552512.36	552512.36	3482629.04	3482975.595	0.009950959347659882	3482417.595	3482737.6	0.0091891621630813	False
g5-testreport-options-default	2000	0.17630169501102744	54.0	54.0	5609.0	5609.0	43127.571	43163.2715	0.0827788330578505	42880.799	43647.3695	1.7876777435980262	False
g5-testreport-unread-default	2000	0.011762039077911624	41.0005	41.0005	4598.4	4598.4	28877.0655	28848.8405	-0.09774192602777232	28845.3315	28854.1555	0.03059073874744911	False
g5-monitor-evaluationmonitor-latent	2000	0.011750636999874331	8.0	8.0	569.0	569.0	13394.372	13346.72	-0.3557613600697396	13395.03	13394.2165	-0.006073148025798059	False
g5-monitor-stepmonitor-latent	2000	0.014943869191785808	8.0	8.0	569.0	569.0	13391.0885	13389.802	-0.009607135372158293	13389.3525	13342.072	-0.3531201378110027	False
g5-names-spellingcorrection-latent	200	0.0042239529792764685	6260.005	6260.005	552632.36	552632.36	3473067.445	3473116.99	0.0014265487435771718	3473608.075	3473341.68	-0.007669115059849574	False
g5-names-resolvecontextaliases-latent	200	0.004963871727703406	6260.005	6260.005	552632.36	552632.36	3473339.71	3473332.64	-0.0002035504900188556	3473540.355	3473307.83	-0.006694178740873635	False
g5-names-ignorecase-latent	200	0.010641213760009816	6260.005	6260.005	552632.36	552632.36	3471198.405	3470799.07	-0.011504240132886867	3470950.97	3470906.87	-0.0012705451728156625	False
g5-testreport-sametest-latent	2000	0.34292954926229413	52.0	52.0	4969.0	4969.0	27742.5755	27741.033	-0.005560046146402264	27883.372	27739.7635	-0.5150327585917491	False
g5-testreport-timeconstraint-latent	2000	0.10886162000942612	52.0	52.0	4969.0	4969.0	27671.8855	27741.069	0.25001368265995527	27755.4	27757.7575	0.008493842639634153	False
g5-testreport-memoryconstraint-latent	2000	0.2586582377222202	52.0	52.0	4969.0	4969.0	27749.861	27762.289	0.044785809918113095	27759.9495	27723.422	-0.1315834526284032	False
g5-arrayfilter-padding-default	20000	0.048806829043201994	97.0	97.0	4200.0	4200.0	63569.2674	63696.09545	0.19951158033952865	63629.4536	63695.76335	0.10421235174649102	True
g5-arrayfilter-padding-explicit	20000	0.0008794609380746721	92.0	92.0	3929.0	3929.0	60031.4098	60159.7739	0.2138282283019155	60031.2171	60159.9602	0.21446025288065496	True
g5-arrayfilter-padding-delayed	20000	0.1376994746907334	96.0	96.0	4145.0	4145.0	63081.9654	63173.9651	0.14584152446208698	63109.16955	63216.4078	0.16992498992565164	True
g5-arrayfilter-padding-delayed-explicit	20000	0.015592318446078224	92.0	92.0	3929.0	3929.0	60041.2597	60175.57295	0.22370158566142884	60053.4114	60170.69745	0.19530289331739503	True
g5-checkabort-default	20000	0.623432501772947	8.0	8.0	672.0	672.0	11059.38515	11081.4831	0.1998117408904987	11107.24355	11155.4394	0.43391368689309484	False
g5-checkabort-explicit	20000	0.0005583598863490025	8.0	8.0	672.0	672.0	12384.5058	12400.4228	0.128523497481825	12432.5894	12428.42965	-0.03345843626108014	False
g5-checkabort-delayed	20000	0.8927247118116988	7.0	7.0	416.0	416.0	8751.55925	8770.5765	0.2173012769124405	8708.37835	8788.4344	0.9192991712400733	False
g5-checkabort-delayed-explicit	20000	0.23929499752259886	7.0	7.0	416.0	416.0	10077.3451	10025.55685	-0.5139076759413497	10029.295	10049.2961	0.19942677925017804	False
```

**candidate-05** — [candidate-05/perf/paired-results.json](../../../.temp/brief-579/candidate-05/perf/paired-results.json) (raw counts), [candidate-05/perf/checks.json](../../../.temp/brief-579/candidate-05/perf/checks.json) (results), [candidate-05/perf/slopes.tsv](../../../.temp/brief-579/candidate-05/perf/slopes.tsv).


```tsv
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g5-cases-small-default	20000	0.9669593960795253	9.0	9.0	912.0	912.0	15909.6963	15884.9498	-0.15554350965203412	15928.8067	15933.1975	0.027565153389685904	False
g5-cases-small-delayed	20000	0.39080178944745914	8.0	8.0	656.0	656.0	14755.9433	14780.7276	0.16796147488584712	14812.1219	14780.80475	-0.211429194354662	False
g5-cases-small-explicit	20000	0.506692673707998	11.0	11.0	1104.0	1104.0	17027.0989	16933.47715	-0.5498397028750546	16989.45235	16994.66155	0.030661376792417983	False
g5-count-small-default	20000	1.6071607106225017	6.0	6.0	664.0	664.0	13135.59175	13122.7397	-0.09784142385514949	13170.6404	13135.73385	-0.26503305032912206	False
g5-count-small-delayed	20000	0.29070766129817294	5.0	5.0	408.0	408.0	11968.0467	12028.9553	0.5089268242912004	11994.8272	12028.8231	0.2834213401590402	False
g5-count-small-explicit	20000	0.40224696589561404	8.0	8.0	824.0	824.0	13937.4796	13967.4979	0.21537825246396025	13967.34735	13967.51275	0.001184190496994475	False
g5-memberq-small-default	20000	0.4195932096378585	6.0	6.0	760.0	760.0	11389.1788	11441.2547	0.4572401655508207	11410.1759	11454.10145	0.3849682107004204	False
g5-memberq-small-delayed	20000	0.9626978699735278	5.0	5.0	504.0	504.0	10299.3643	10317.1872	0.1730485443650176	10286.1753	10341.11265	0.5340891866775666	False
g5-memberq-small-explicit	20000	0.28486335406960883	7.0	7.0	792.0	792.0	12208.03175	12014.95705	-1.5815383180011744	11991.00685	12289.9621	2.493162198468779	False
g5-freeq-small-default	20000	0.0010925278075335498	7.0	7.0	1208.0	1208.0	12449.794	12449.64255	-0.0012164859916508064	12449.63895	12449.72715	0.0007084542801072402	False
g5-freeq-small-delayed	20000	0.4834624434365331	6.0	6.0	952.0	952.0	11199.6408	11201.7162	0.01853095145696315	11214.8029	11201.9364	-0.11472782994697006	False
g5-freeq-small-explicit	20000	0.19906776755334032	8.0	8.0	1240.0	1240.0	13109.43175	13013.44305	-0.7322109899996265	13039.59085	13052.38505	0.09811810928102016	False
g5-position-small-default	20000	0.6841434466906104	17.0	17.0	2136.0	2136.0	19314.15215	19314.73935	0.0030402577107180306	19359.66855	19291.0092	-0.3546514746503693	False
g5-position-small-delayed	20000	0.8382295705443576	16.0	16.0	1880.0	1880.0	18130.34205	18158.28165	0.1541040975561847	18115.33895	18129.722	0.07939707912558713	False
g5-position-small-explicit	20000	0.5030712420343919	18.0	18.0	2168.0	2168.0	19989.57825	19812.2562	-0.8870724923873752	20037.67925	19898.9743	-0.6922206322870439	False
g5-deletecases-small-default	20000	1.0469783942181787	9.0	9.0	984.0	984.0	14556.434	14494.20835	-0.42747866682182334	14515.96175	14494.35055	-0.148878871219138	False
g5-deletecases-small-delayed	20000	0.34498659116175556	8.0	8.0	728.0	728.0	13321.8362	13401.9703	0.6015244354978799	13289.901	13353.91305	0.4816593441892447	True
g5-deletecases-small-explicit	20000	1.0402054884521506	10.0	10.0	1048.0	1048.0	15237.81175	15225.1816	-0.08288690139515165	15266.77055	15194.7658	-0.4716436247219291	False
g5-replace-small-default	20000	0.11718744844069207	20.0	20.0	1945.0	1945.0	20190.2518	20274.6812	0.41816912852965515	20214.3244	20280.39085	0.3268298692188809	True
g5-replace-small-delayed	20000	0.9037190538253026	19.0	19.0	1689.0	1689.0	19113.36845	19064.58695	-0.2552218889496749	19058.0204	19113.20115	0.28954082765069167	False
g5-replace-small-explicit	20000	0.00579554520163672	22.0	22.0	2233.0	2233.0	21267.61505	21374.9698	0.5047803890920965	21267.84655	21404.6978	0.6434654758217695	True
g5-level-small-default	20000	0.24139453222944418	13.0	13.0	1009.0	1009.0	13296.84435	13346.12805	0.37064207644124103	13345.23465	13347.29765	0.015458701582282686	False
g5-level-small-delayed	20000	0.017689370435969298	12.0	12.0	753.0	753.0	12214.5684	12183.19465	-0.2568551664911878	12181.1323	12179.46435	-0.013692897826900197	False
g5-level-small-explicit	20000	0.7389279368723578	13.0	13.0	1009.0	1009.0	14222.393	14228.78975	0.04497660836682016	14244.1024	14229.50235	-0.10249891211115969	False
g5-depth-small-default	20000	0.6795577063621439	4.0	4.0	392.0	392.0	8791.4981	8790.543	-0.010863904981117667	8789.28825	8868.4489	0.9006491509707892	False
g5-depth-small-delayed	20000	0.5556417845853701	3.0	3.0	136.0	136.0	7665.6402	7646.7087	-0.24696567417813453	7573.43375	7627.6638	0.7160563066918035	False
g5-depth-small-explicit	20000	0.26700477241450304	4.0	4.0	392.0	392.0	9904.87695	9694.7265	-2.1216866303422344	9717.55255	9668.7833	-0.5018676230364338	False
g5-trace-small-default	20000	0.790553251704873	33.0	33.0	1992.0	1992.0	27608.1703	27572.03565	-0.1308838999736217	27525.4403	27530.6271	0.01884365860627124	False
g5-trace-small-delayed	20000	0.2308829411903801	32.0	32.0	1736.0	1736.0	26216.2047	26308.9499	0.353770505919182	26364.8322	26305.48475	-0.2251008068240301	False
g5-trace-small-explicit	20000	0.5834534965366123	36.0	36.0	2120.0	2120.0	31760.923	31816.9669	0.17645551421789918	31783.59535	31623.6942	-0.503093335537319	False
g5-tracedetail-small-default	20000	0.257203289572272	55.0	55.0	3263.0	3263.0	35957.5181	36026.05855	0.19061507473732142	36031.1854	36064.56365	0.09263711318250323	False
g5-tracedetail-small-delayed	20000	0.16748863579458817	54.0	54.0	3007.0	3007.0	34887.13025	34805.0947	-0.23514559498627463	34857.7113	34882.75895	0.07185684046904228	False
g5-tracedetail-small-explicit	20000	0.47625902350401095	57.0	57.0	3359.0	3359.0	39000.80225	39106.9722	0.2722250412169336	39016.2624	38983.81545	-0.08316263015495373	False
g5-traceprint-small-default	2000	0.3006013455779444	40.0	40.0	2174.0	2174.0	30661.285	30626.069	-0.11485493840196481	30697.6795	30628.241	-0.22620113679927556	False
g5-traceprint-small-delayed	2000	0.1741494002782673	39.0	39.0	1918.0	1918.0	29423.0215	29409.533	-0.04584335432715747	29422.6105	29435.7965	0.04481587383282637	False
g5-traceprint-small-explicit	2000	0.46695787436616953	43.0	43.0	2302.0	2302.0	34867.362	34820.1875	-0.13529701501364677	34821.237	34844.821	0.06772878286891437	False
matchq-integer-success	20000	0.0006197752900005682	1.0	1.0	128.0	128.0	4340.37915	4340.12155	-0.005934965381992896	4340.2972	4340.2897	-0.00017279922673862913	False
matchq-integer-fail	20000	0.002067359757695897	2.0	2.0	129.0	129.0	4462.23585	4462.0354	-0.004492142655354581	4462.1714	4462.2353	0.0014320382224708439	False
cases-wide-no-match	1000	0.0013213325351646432	33.001	33.001	263247.864	263247.864	2400534.405	2400559.637	0.001051099286386581	2400535.951	2400533.104	-0.00011859851541951016	False
count-wide-no-match	1000	0.00149936097200154	30.0	30.0	262601.0	262601.0	2178887.183	2178843.581	-0.0020011132444430757	2178862.819	2178891.483	0.001315548631608543	False
position-wide-no-match	1000	0.00011471771085291849	1642.001	1642.001	576255.864	576255.864	2573246.174	2573233.956	-0.000474808827999329	2573273.464	2573266.541	-0.0002690347565725837	False
deletecases-wide-no-match	1000	0.0009557190257747372	24.001	24.001	247447.864	247447.864	2670167.118	2670044.981	-0.004574133176016293	2670169.206	2670168.29	-3.430494208700452e-05	False
replace-wide-no-match	1000	0.0020920686264913613	30.001	30.001	184455.864	184455.864	1163557.982	1163588.415	0.0026155121163551343	1163587.9	1163573.849	-0.001207558105409401	False
dot-three-no-input	20000	0.00029744466937264846	596.0	596.0	23472.0	23472.0	253107.4013	253102.688	-0.0018621739134472293	253101.93415	253106.0858	0.001640307496653115	False
differences-no-input	20000	0.22525974871568932	13.0	13.0	672.0	672.0	9397.62975	9399.26875	0.017440567926185757	9396.53355	9399.9116	0.035949959440095824	False
take-multispec-small	20000	0.049703996859062904	109.0	109.0	5824.0	5824.0	49224.45445	49231.18075	0.013664549612912857	49225.0959	49231.058	0.012111911395984443	False
g5-monitor-options-none	2000	0.484180295848069	9.0	9.0	953.0	953.0	18148.356	18195.852	0.26170965568450555	18147.1785	18186.4885	0.21661769624405913	False
g5-monitor-explicit-held	2000	0.4215454971300714	13.0005	13.0005	1727.856	1727.856	17825.6925	17944.1805	0.6647034890790327	17810.2765	17794.018	-0.09128718467678043	False
g5-names-default	200	0.003548990232449256	6253.005	6253.005	552512.36	552512.36	3482434.63	3482613.215	0.0051281651767931535	3482491.025	3482805.885	0.009041229330941647	False
g5-testreport-options-default	2000	0.17630169501102744	54.0	54.0	5609.0	5609.0	43152.685	43342.1635	0.43908855265901714	42905.5295	43159.331	0.5915356434419428	True
g5-testreport-unread-default	2000	0.011762039077911624	41.0005	41.0005	4598.4	4598.4	28844.472	28808.5445	-0.12455592877553823	28895.314	28905.123	0.0339466807663058	False
g5-monitor-evaluationmonitor-latent	2000	0.011750636999874331	8.0	8.0	569.0	569.0	13392.6475	13347.8025	-0.33484790815258014	13469.872	13394.8235	-0.5571582268933173	False
g5-monitor-stepmonitor-latent	2000	0.014943869191785808	8.0	8.0	569.0	569.0	13390.117	13391.131	0.007572749364315179	13391.102	13391.5925	0.003662880022869963	False
g5-names-spellingcorrection-latent	200	0.0042239529792764685	6260.005	6260.005	552632.36	552632.36	3473559.805	3473638.91	0.002277346711743178	3473465.19	3473429.17	-0.0010370047785035474	False
g5-names-resolvecontextaliases-latent	200	0.004963871727703406	6260.005	6260.005	552632.36	552632.36	3473297.27	3473468.38	0.00492644270555509	3473353.795	3473457.595	0.0029884660799561757	False
g5-names-ignorecase-latent	200	0.010641213760009816	6260.005	6260.005	552632.36	552632.36	3470697.23	3471300.58	0.017384115064400518	3470658.54	3470936.64	0.008012888528075024	False
g5-testreport-sametest-latent	2000	0.34292954926229413	52.0	52.0	4969.0	4969.0	27767.1975	27740.169	-0.09733967570907387	27888.065	27658.895	-0.8217493755841354	False
g5-testreport-timeconstraint-latent	2000	0.10886162000942612	52.0	52.0	4969.0	4969.0	27742.5905	27731.665	-0.039381686436235075	27742.3275	27755.2905	0.046726432740729074	False
g5-testreport-memoryconstraint-latent	2000	0.2586582377222202	52.0	52.0	4969.0	4969.0	27736.341	27762.67	0.09492600339748858	27906.429	27748.511	-0.5658839402203797	False
g5-arrayfilter-padding-default	20000	0.048806829043201994	97.0	97.0	4200.0	4200.0	63570.2099	63656.13445	0.1351648046076237	63687.55245	63704.5141	0.026632598282549935	False
g5-arrayfilter-padding-explicit	20000	0.0008794609380746721	92.0	92.0	3929.0	3929.0	60031.1021	60098.6625	0.11254232828752375	60031.34965	60099.12705	0.11290334199576257	True
g5-arrayfilter-padding-delayed	20000	0.1376994746907334	96.0	96.0	4145.0	4145.0	63099.63585	63256.89775	0.24922790422092778	63147.8606	63340.9305	0.3057425828294935	True
g5-arrayfilter-padding-delayed-explicit	20000	0.015592318446078224	92.0	92.0	3929.0	3929.0	60056.6584	60112.22255	0.09251954983895416	60050.8729	60112.5277	0.10267094718616754	True
g5-checkabort-default	20000	0.623432501772947	8.0	8.0	672.0	672.0	11131.38515	11118.4132	-0.11653491299776775	11107.30245	11120.39045	0.11783239052791661	False
g5-checkabort-explicit	20000	0.0005583598863490025	8.0	8.0	672.0	672.0	12384.486	12402.46905	0.14520626855243357	12384.5859	12389.5231	0.03986568497216236	False
g5-checkabort-delayed	20000	0.8927247118116988	7.0	7.0	416.0	416.0	8727.50395	8788.54245	0.6993809495783765	8751.3213	8788.5947	0.42591739832475994	False
g5-checkabort-delayed-explicit	20000	0.23929499752259886	7.0	7.0	416.0	416.0	10005.2574	10049.4896	0.44208957582640984	10029.2626	10049.35185	0.2003063515357395	False
```

**cumulative** — [cumulative/perf/paired-results.json](../../../.temp/brief-579/cumulative/perf/paired-results.json) (raw counts), [cumulative/perf/checks.json](../../../.temp/brief-579/cumulative/perf/checks.json) (results), [cumulative/perf/slopes.tsv](../../../.temp/brief-579/cumulative/perf/slopes.tsv).


```tsv
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g5-cases-small-default	20000	0.9669593960795253	9.0	9.0	912.0	912.0	15847.68045	15932.86285	0.5375070520178271	15832.55335	16027.9826	1.2343508067193643	False
g5-cases-small-delayed	20000	0.39080178944745914	8.0	8.0	656.0	656.0	14664.21005	14769.41955	0.717457671714139	14682.13135	14786.0036	0.7074739186283097	False
g5-cases-small-explicit	20000	0.506692673707998	11.0	11.0	1104.0	1104.0	16893.16775	16933.65775	0.23968269657417274	16915.3381	16865.5556	-0.29430390161696574	False
g5-count-small-default	20000	1.6071607106225017	6.0	6.0	664.0	664.0	13176.59865	13135.5884	-0.3112354795749894	13187.69745	13081.69885	-0.803768818642403	False
g5-count-small-delayed	20000	0.29070766129817294	5.0	5.0	408.0	408.0	12020.8115	11968.02875	-0.439094731666001	12050.8327	11952.833	-0.8132193221801254	False
g5-count-small-explicit	20000	0.40224696589561404	8.0	8.0	824.0	824.0	14006.51435	14015.383	0.0633180374387754	14032.4306	13967.39905	-0.4634375316276307	False
g5-memberq-small-default	20000	0.4195932096378585	6.0	6.0	760.0	760.0	11619.3426	11441.3138	-1.532176183530387	11439.31865	11441.34425	0.017707348330575634	False
g5-memberq-small-delayed	20000	0.9626978699735278	5.0	5.0	504.0	504.0	10353.27785	10293.19385	-0.5803379458226399	10340.0643	10293.20425	-0.45318915473281374	False
g5-memberq-small-explicit	20000	0.28486335406960883	7.0	7.0	792.0	792.0	12074.06	12028.24215	-0.379473433128541	12074.0474	12028.0142	-0.3812574066919705	False
g5-freeq-small-default	20000	0.0010925278075335498	7.0	7.0	1208.0	1208.0	12501.73665	12449.7061	-0.41618657836630657	12501.58765	12449.5275	-0.41642830860766367	False
g5-freeq-small-delayed	20000	0.4834624434365331	6.0	6.0	952.0	952.0	11254.62125	11201.7788	-0.46951779918849024	11186.8165	11207.7539	0.18716137875327554	False
g5-freeq-small-explicit	20000	0.19906776755334032	8.0	8.0	1240.0	1240.0	13078.54195	13086.4417	0.06040237535804849	13054.4003	13007.4147	-0.359921550743314	False
g5-position-small-default	20000	0.6841434466906104	17.0	17.0	2136.0	2136.0	19339.8733	19315.07255	-0.1282363623343774	19342.68065	19367.1359	0.12643154505063148	False
g5-position-small-delayed	20000	0.8382295705443576	16.0	16.0	1880.0	1880.0	18130.20645	18207.43945	0.425990736580939	18105.44905	18162.89	0.31725780366658185	False
g5-position-small-explicit	20000	0.5030712420343919	18.0	18.0	2168.0	2168.0	20021.65765	19898.6362	-0.6144418816391051	19997.71385	19836.2359	-0.8074820512545711	False
g5-deletecases-small-default	20000	1.0469783942181787	9.0	9.0	984.0	984.0	14579.90345	14494.313	-0.5870440109121522	14482.91705	14494.34165	0.07888327993978983	False
g5-deletecases-small-delayed	20000	0.34498659116175556	8.0	8.0	728.0	728.0	13325.2492	13330.07625	0.03622483848182423	13378.30715	13339.04925	-0.2934444512286527	False
g5-deletecases-small-explicit	20000	1.0402054884521506	10.0	10.0	1048.0	1048.0	15262.73155	15225.25655	-0.24553272051751618	15199.4969	15176.94245	-0.14838945096925515	False
g5-replace-small-default	20000	0.11718744844069207	20.0	20.0	1945.0	1945.0	20167.41705	20199.35345	0.15835642175108244	20175.3621	20214.76	0.19527728823265367	False
g5-replace-small-delayed	20000	0.9037190538253026	19.0	19.0	1689.0	1689.0	19099.43655	19065.19355	-0.17928801150942641	19143.99695	19063.56835	-0.42012438786979134	False
g5-replace-small-explicit	20000	0.00579554520163672	22.0	22.0	2233.0	2233.0	21200.55895	21422.2738	1.045797191116038	21212.68905	21305.53365	0.4376842548399207	False
g5-level-small-default	20000	0.24139453222944418	13.0	13.0	1009.0	1009.0	13362.03705	13349.0616	-0.09710682548960126	13383.64	13349.0457	-0.2584819974237096	False
g5-level-small-delayed	20000	0.017689370435969298	12.0	12.0	753.0	753.0	12211.48655	12181.27745	-0.2473826579287408	12258.98265	12296.90765	0.30936498633513665	False
g5-level-small-explicit	20000	0.7389279368723578	13.0	13.0	1009.0	1009.0	14274.9635	14323.5817	0.3405837079723639	14285.24805	14276.61735	-0.060416871795232474	False
g5-depth-small-default	20000	0.6795577063621439	4.0	4.0	392.0	392.0	8767.4297	8820.4348	0.6045682921187234	8851.5132	8826.46005	-0.28303804596935267	False
g5-depth-small-delayed	20000	0.5556417845853701	3.0	3.0	136.0	136.0	7674.378	7633.61365	-0.5311746437300813	7585.7413	7670.60415	1.1187153192266264	False
g5-depth-small-explicit	20000	0.26700477241450304	4.0	4.0	392.0	392.0	9757.69835	9698.76805	-0.603936480573819	9593.5481	9681.73715	0.919253743044246	False
g5-trace-small-default	20000	0.790553251704873	33.0	33.0	1992.0	1992.0	27496.1727	27726.2134	0.8366280736955112	27626.2279	27589.1093	-0.13436000070063958	False
g5-trace-small-delayed	20000	0.2308829411903801	32.0	32.0	1736.0	1736.0	26515.71295	26394.17285	-0.4583701001334095	26304.7494	26501.6652	0.7485940922896672	False
g5-trace-small-explicit	20000	0.5834534965366123	36.0	36.0	2120.0	2120.0	31706.2132	31594.49545	-0.35235286312904623	31737.8515	32344.1067	1.9101960950318242	False
g5-tracedetail-small-default	20000	0.257203289572272	55.0	55.0	3263.0	3263.0	36040.92865	36046.45205	0.015325354276063408	36142.3447	36292.0385	0.4141784414999572	False
g5-tracedetail-small-delayed	20000	0.16748863579458817	54.0	54.0	3007.0	3007.0	35006.96835	34883.39445	-0.3529980053242787	34893.82215	34931.6044	0.10827776285893265	False
g5-tracedetail-small-explicit	20000	0.47625902350401095	57.0	57.0	3359.0	3359.0	39031.05345	38891.0334	-0.35874012516564635	39175.9201	39010.82735	-0.4214138419176616	False
g5-traceprint-small-default	2000	0.3006013455779444	40.0	40.0	2174.0	2174.0	30670.672	30668.88	-0.00584271515146062	30658.1555	30688.5315	0.09907967229143555	False
g5-traceprint-small-delayed	2000	0.1741494002782673	39.0	39.0	1918.0	1918.0	29438.0655	29445.9765	0.026873369107760503	29436.859	29408.0335	-0.09792315137969743	False
g5-traceprint-small-explicit	2000	0.46695787436616953	43.0	43.0	2302.0	2302.0	34833.8605	34771.873	-0.17795185233633326	34833.6795	34820.418	-0.03807091352494574	False
matchq-integer-success	20000	0.0006197752900005682	1.0	1.0	128.0	128.0	4340.27465	4340.32065	0.0010598407637507634	4340.06805	4340.2713	0.004683106293712491	False
matchq-integer-fail	20000	0.002067359757695897	2.0	2.0	129.0	129.0	4462.07545	4462.12555	0.0011227958953474726	4462.13035	4462.3838	0.005680022323839573	False
cases-wide-no-match	1000	0.0013213325351646432	33.001	33.001	263247.864	263247.864	2411613.525	2400530.763	-0.4595579633764113	2411598.724	2400647.982	-0.45408640712152337	False
count-wide-no-match	1000	0.00149936097200154	30.0	30.0	262601.0	262601.0	2178929.722	2178858.607	-0.0032637583159411676	2178928.432	2178874.267	-0.0024858549369732863	False
position-wide-no-match	1000	0.00011471771085291849	1642.001	1642.001	576255.864	576255.864	2573287.297	2573473.691	0.007243419738545143	2573216.759	2573368.551	0.005898920076163883	False
deletecases-wide-no-match	1000	0.0009557190257747372	24.001	24.001	247447.864	247447.864	2670078.506	2670162.14	0.0031322674525213756	2670216.895	2670161.764	-0.0020646637396093404	False
replace-wide-no-match	1000	0.0020920686264913613	30.001	30.001	184455.864	184455.864	1163594.849	1163598.519	0.0003154018774820244	1163560.546	1163583.989	0.002014764086033516	False
dot-three-no-input	20000	0.00029744466937264846	596.0	596.0	23472.0	23472.0	253111.9942	253101.015	-0.004337684602684355	253109.71785	253102.02845	-0.0030379710685535244	False
differences-no-input	20000	0.22525974871568932	13.0	13.0	672.0	672.0	9411.0372	9399.42795	-0.12335781650083533	9409.0206	9399.9154	-0.09677096466341739	False
take-multispec-small	20000	0.049703996859062904	109.0	109.0	5824.0	5824.0	49240.289	49230.6089	-0.019658901677033214	49240.78425	49231.57005	-0.018712537057119505	False
g5-monitor-options-none	2000	0.484180295848069	9.0	9.0	953.0	953.0	18194.8005	18161.2005	-0.18466814186834934	18156.9125	18231.035	0.408232952601395	False
g5-monitor-explicit-held	2000	0.4215454971300714	13.0005	13.0005	1727.856	1727.856	17703.492	17854.825	0.8548200547101148	17746.2475	17839.793	0.5271283407942917	False
g5-names-default	200	0.003548990232449256	6253.005	6253.005	552512.36	552512.36	3482213.705	3482704.05	0.014081416062872165	3482645.785	3482370.52	-0.007903904588446586	False
g5-testreport-options-default	2000	0.17630169501102744	54.0	54.0	5609.0	5609.0	42979.566	43236.044	0.5967440434368365	42923.9135	43090.7475	0.38867378669933395	False
g5-testreport-unread-default	2000	0.011762039077911624	41.0005	41.0005	4598.4	4598.4	28790.533	28847.787	0.19886398073978295	28790.0425	28846.546	0.1962605647421345	False
g5-monitor-evaluationmonitor-latent	2000	0.011750636999874331	8.0	8.0	569.0	569.0	13352.331	13395.691	0.324737306167755	13236.137	13392.841	1.1839103811028817	False
g5-monitor-stepmonitor-latent	2000	0.014943869191785808	8.0	8.0	569.0	569.0	13305.5915	13389.9115	0.6337185385557609	13450.885	13390.6685	-0.44767686289787223	False
g5-names-spellingcorrection-latent	200	0.0042239529792764685	6260.005	6260.005	552632.36	552632.36	3473276.08	3473453.475	0.005107425839878665	3473152.805	3473550.16	0.011440757787206657	False
g5-names-resolvecontextaliases-latent	200	0.004963871727703406	6260.005	6260.005	552632.36	552632.36	3473169.615	3473283.775	0.003286911169175788	3473282.865	3472985.215	-0.008569702254879186	False
g5-names-ignorecase-latent	200	0.010641213760009816	6260.005	6260.005	552632.36	552632.36	3470650.975	3470676.205	0.0007269529601616753	3470792.41	3470753.735	-0.0011142988525891795	False
g5-testreport-sametest-latent	2000	0.34292954926229413	52.0	52.0	4969.0	4969.0	27725.0555	27739.65	0.052640111036028614	27510.004	27652.975	0.5197054860478945	False
g5-testreport-timeconstraint-latent	2000	0.10886162000942612	52.0	52.0	4969.0	4969.0	27716.141	27756.085	0.14411818730464265	27594.731	27781.981	0.6785715722323893	False
g5-testreport-memoryconstraint-latent	2000	0.2586582377222202	52.0	52.0	4969.0	4969.0	27602.9875	27761.0695	0.5726988790615506	27602.974	27787.527	0.6685982459716078	False
g5-arrayfilter-padding-default	20000	0.048806829043201994	97.0	97.0	4200.0	4200.0	63569.59205	63725.34585	0.24501305573503274	63666.8063	63672.3144	0.00865144699428999	False
g5-arrayfilter-padding-explicit	20000	0.0008794609380746721	92.0	92.0	3929.0	3929.0	60031.29955	60159.81595	0.2140823219942911	60031.22735	60159.37205	0.2134634017273651	False
g5-arrayfilter-padding-delayed	20000	0.1376994746907334	96.0	96.0	4145.0	4145.0	63098.63245	63264.26285	0.2624944369931548	63108.5645	63246.3113	0.2182695820945213	False
g5-arrayfilter-padding-delayed-explicit	20000	0.015592318446078224	92.0	92.0	3929.0	3929.0	60041.22735	60175.8808	0.22426831686011006	60053.00165	60173.32935	0.2003691683911013	False
g5-checkabort-default	20000	0.623432501772947	8.0	8.0	672.0	672.0	11108.3222	11142.3715	0.30652063729299517	11108.30815	11129.55575	0.19127665269169647	False
g5-checkabort-explicit	20000	0.0005583598863490025	8.0	8.0	672.0	672.0	12385.37845	12367.4633	-0.14464757837093778	12385.4767	12415.26215	0.24048690834808095	False
g5-checkabort-delayed	20000	0.8927247118116988	7.0	7.0	416.0	416.0	8750.52655	8788.3984	0.4327950984846618	8798.42355	8788.6742	-0.11080791853900207	False
g5-checkabort-delayed-explicit	20000	0.23929499752259886	7.0	7.0	416.0	416.0	10013.37975	10049.4134	0.3598550229756192	10028.337	10049.11565	0.2071993591759025	False
```

The noise floor is each label’s largest absolute difference across two base/base pairs. An immediate-base regression requires both candidate samples above max(+0.05%, floor). The cumulative panel compares selected candidate-04 directly with .temp/brief-577/base for every label; its deltas are informational, not a reapplication of the immediate-base gate. [pre-g5-hashes.json](../../../.temp/brief-579/pre-g5-hashes.json) and [candidate-04/hashes.json](../../../.temp/brief-579/candidate-04/hashes.json) pin both arms.

**Residual attribution.** These bounded callgrind runs are diagnostic, not extra hardware samples or an alternative gate. [candidate-04/attribution/details.json](../../../.temp/brief-579/candidate-04/attribution/details.json); [candidate-02/attribution/details.json](../../../.temp/brief-579/candidate-02/attribution/details.json).

```tsv
label	base_total	candidate-04_total	base_rule_self	candidate-04_rule_self	candidate-04_continuation_self
g5-count-small-delayed	11985.5	12018.85	286.0	286.0	0
g5-arrayfilter-padding-default	63780.88	63952.08	0.0	0.0	0
g5-arrayfilter-padding-explicit	60176.86	60380.75	0.0	0.0	0
g5-arrayfilter-padding-delayed	63368.75	63545.55	0.0	0.0	0
g5-arrayfilter-padding-delayed-explicit	60298.46	60361.16	0.0	0.0	0
```

The explicit-padding candidate-02 profile showed +75 self instructions/iteration in parse_array_filter_padding; eval_array_filter (-649) and apply_array_filter (+649) relocation canceled. This suggests lost parser inlining, not a semantic tensor/operator change. The parser is in spec.rs, outside this brief’s edit surface, and was left byte-identical. The selected residuals are Count delayed and all four ArrayFilter labels; no causal claim is made for the Count variation.

### 4. Stop-loss, residuals and open questions

Selected candidate-04 residuals, verbatim reduced rows:
```tsv
label	noise_floor_pct	instructions_pair1_pct	instructions_pair2_pct
g5-count-small-delayed	0.29070766129817294	0.6705184729595137	0.3594586027472424
g5-arrayfilter-padding-default	0.048806829043201994	0.19951158033952865	0.10421235174649102
g5-arrayfilter-padding-explicit	0.0008794609380746721	0.2138282283019155	0.21446025288065496
g5-arrayfilter-padding-delayed	0.1376994746907334	0.14584152446208698	0.16992498992565164
g5-arrayfilter-padding-delayed-explicit	0.015592318446078224	0.22370158566142884	0.19530289331739503
```

Measured 5/8 candidates; final stagnant-best counter 0 (candidate 05 improved the best worst regression while worsening label count). Neither the eight-candidate cap nor two-stagnant stop-loss fired. Tuning stopped at the edit fence after the supported in-scope attempts; further parser inlining work requires the director’s follow-up. Five instruction residuals remain, so this is **not a performance pass**. Cumulative deltas and all rejected candidates remain in the report; no favorable resampling, averaging-away or timing substitute was used.

Final evidence audit: [final-evidence.json](../../../.temp/brief-579/final-evidence.json).

All work is left uncommitted for the director. [PROGRESS.md](../../../.temp/brief-579/PROGRESS.md) records each gate and compiling candidate.
