# brief-597 — G12: `ToExpression` resumes into its result head, and the fieldless-frame sweep

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

The E1 oracle's seventh drain (brief-596) found group G12. `ToExpression[str,
form, head]` evaluates the parsed source, then evaluates `head[value]`
(`crates/rum-stdlib/src/strings/text/builtins.rs:460-479`). When the parsed
source asks for input, the fieldless `EvalToExpressionSuspensionFrame1`
(`strings/text/builtins/resume_frames.rs:6-21`) completes with the resumed
value and the result head is never applied: the resumed path returns `7`
where the queued path returns `107`
(`.temp/brief-596/defects/ToExpression.parsedSource.md`, with the exact
`Middle`, `Twice`, `AlreadyResumed` and `Nested` witnesses).

1. **Repair.** The first frame carries the result head (captured only on
   suspension) and, on resume, enters the same completion helper as the eager
   path: apply the head once (its own suspension still carried by the second
   frame), or return the value when there is none. Attach the
   `ToExpression.parsedSource` descriptor (owed 39 -> 38).
2. **The class.** This is a frame with no fields whose function still has work
   after the `?` — the census classified it as a correct tail. Sweep the whole
   repository (`rum-runtime`, `rum-stdlib`, and any crate using
   `resume_or_suspend!` or inserting a frame): list every site whose suspension
   frame is a unit struct or carries less state than the work that follows it
   in the eager path, with `path:line`, the work that follows, and a verdict
   (correct tail / **lost suffix**). Each lost suffix gets an oracle witness
   (resumed against queued) in the scratch test and is **reported, not
   repaired** here — it becomes a group of this branch. A site is a correct
   tail only when nothing runs after the suspension point in the eager path.

## Context pointers

- brief-596's report and `.temp/brief-596/defects/ToExpression.parsedSource.md`.
- brief-548's census and its `fieldless-frames.tsv`; brief-583's
  `reviewed-census.tsv` (which classified this site); the accepted
  continuation pattern of briefs 559-595.
- The oracle `crates/rum-stdlib/tests/suspension_oracle/` (its
  `fixtures/strings_roles/` descriptor for this entry).

## Scope

**Edit surface:** `crates/rum-stdlib/src/strings/text/builtins.rs` (only
`ToExpression` and a shared completion helper) and
`strings/text/builtins/resume_frames.rs`; the oracle's
`fixtures/strings_roles/` and `inventory/{effects,owed}.tsv` — attach exactly
`ToExpression.parsedSource` and delete exactly its owed row after its gates pass
in both builds; rum-runtime's frame lists only if a frame type is added (a
field added to an existing frame needs none). Scratch: `.temp/brief-597/`, with
one scratch integration test `crates/rum-stdlib/tests/zz_brief597_sweep.rs` for
the sweep's witnesses, **deleted before the report**, its content copied to
`.temp/brief-597/sweep.rs`.

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed. **STOP** on any
existing row change or disappearance, any new `UnknownHostPayload` or
`self_reaching` row, any new row in untouched code.

**Fence:** no other production change; a lost suffix found by the sweep is a
report, not a repair; no leases; `scripts/` unchanged. No edits to
`AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs or docs.

**Disclosure duty:** every changed function and field, every inventory row,
every measured label, every sweep row.

## Tests

Red first, recorded before any production edit: brief-596's four witnesses
through the oracle. Green: the full `ToExpression.parsedSource` descriptor
(`Middle`, `Twice`, `Error`, `AlreadyResumed`, `Nested`), lifecycle at the
first and second pause, both builds; plus a head that itself suspends after a
suspending source, and no head.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. The head is applied exactly once, on whichever path completes.
3. Stop after two consecutive compiling candidates that move none of: reds
   turned green, sweep sites classified, regressing labels.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-597/tmp …`. Bounds: focused 300 s; the oracle 900 s
per build; runtime, ownership and retention 300 s; a one-unit release build
900 s; a metric process 150 s; `cargo test --workspace --no-fail-fast` 3 600 s.

```
cargo fmt --check
scripts/check-suspension-oracle.sh 900
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Instruction gate.** Before any production edit, build the base from the
repository tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into
`.temp/brief-597/base/target-default` and `target-alloc` (`--features
rum-cli/alloc-count`, two arguments); candidates the same way in fresh
directories. Panel (scratch workloads, each result checked first):
`ToExpression` without a head, with a head, and a small arithmetic source; the
controls `dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). Noise floor first (base against itself, two pairs);
a label regresses when both candidate samples exceed `max(+0.05%, floor)`.
Allocation counts, then two paired samples of `taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output first. No timing suite. At most four
measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-597/`; `PROGRESS.md` after the red capture, each
compiling candidate, the sweep and each gate.

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

1. Changed files, functions and fields.
2. Verbatim red and green output, the oracle in both builds, ownership,
   certificates and the broad suites; every inventory row change.
3. The sweep table, every lost suffix with its witness; the noise floors and
   every measured row.
4. Stop-loss counters, open questions.

---


## brief-597 execution report

### 1. Changed files, functions and fields

Candidate 2 repairs the parsed-source continuation. Both eager and resumed completion call `finish_to_expression`; `EvalToExpressionSuspensionFrame1.result_head: Option<Expr>` is cloned only in the suspension arm. Its `resume` propagates errors and applies the retained head exactly once on success. The existing second frame continues to carry head suspension. The helper has `#[inline(always)]`, required by the measured no-head gate. No frame type, lease, unsafe block, session borrow, cancellation observer or native-stack guard was added.

Changed production functions: `eval_to_expression` (builtins.rs:438), new `finish_to_expression` (builtins.rs:474), and `EvalToExpressionSuspensionFrame1::resume` (resume_frames.rs:11). Changed field: the new `result_head` (resume_frames.rs:7). Frame 2 behavior is unchanged.

Oracle changes: `strings_roles::ENTRIES` and its module declarations; the `ToExpression.parsedSource` arm in `text::parsed`; new `parsed_source::check` and eight tests (`parsed_source_middle`, `parsed_source_twice`, `parsed_source_error`, `parsed_source_already_resumed`, `parsed_source_nested`, `parsed_source_lifecycle`, `parsed_source_then_suspending_head`, `parsed_source_without_head`). Existing descriptor variants, lifecycle driver and falsifiers are reused. The source/head test proves the head is entered once even when it also suspends. The no-head test covers two source pauses.

All changed/new code and inventory files (initial contents and hashes preserved without version-control operations):

```text
crates/rum-stdlib/src/strings/text/builtins/resume_frames.rs
crates/rum-stdlib/src/strings/text/builtins.rs
crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv
crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv
crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv
crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv
crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv
crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/mod.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/text.rs
crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv
crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv
crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs
```
`builtins.rs` remains 23,149 characters, in the split-plan range. The brief limits edits to ToExpression and its helper, so no broader split was made. A future conversion module can own parsing/evaluation and character-code conversion independently of text transformations. Other edited/new Rust modules are below 5,000 characters.

No git command or network access was used; changes are uncommitted. Scripts, other production code, AGENTS.md, TODO.md, root PROGRESS.md and earlier briefs remain byte-identical. Scratch is under `.temp/brief-597/`; `sweep.rs` preserves the sole scratch integration test, deleted before this report. The oracle script and run-bounded script were copied byte-for-byte into `oracle-runner/scripts/` so the oracle script’s hard-coded scratch stays under this brief; its TMPDIR points at this brief’s tmp. Required legacy tests also retain their own existing fixed output locations, including the authorized brief-546 certificates. No test or script implementation was changed to relocate those outputs.

### 2. Verification and every inventory row

All verification used bounded serial commands, one Cargo/test worker, offline mode and this brief’s TMPDIR. Full commands and verbatim logs are beside the report under `.temp/brief-597/`. Both fresh release modes used `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`; alloc mode used the separate arguments `--features rum-cli/alloc-count`.

**red-default — captured before production edits.**

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
   Compiling crc32fast v1.5.0
   Compiling cpufeatures v0.2.17
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: `rum-stdlib` (test "suspension_oracle") generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 07s
     Running tests/suspension_oracle.rs (.temp/brief-597/target-tests/debug/deps/suspension_oracle-08d116f9343550b3)

running 8 tests
test oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed ... ToExpression.parsedSource AlreadyResumed queued=true value=Some("107") suffix=1
ToExpression.parsedSource AlreadyResumed queued=false value=Some("7") suffix=0

thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:13:29:
called `Result::unwrap()` on an `Err` value: Value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_error ... ToExpression.parsedSource Error queued=true value=None suffix=0
ToExpression.parsedSource Error queued=false value=None suffix=0
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_lifecycle ... ToExpression.parsedSource lifecycle first=PASS second=PASS obligations=12
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_middle ... ToExpression.parsedSource Middle queued=true value=Some("107") suffix=1
ToExpression.parsedSource Middle queued=false value=Some("7") suffix=0

thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_middle' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:13:29:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_nested ... ToExpression.parsedSource Nested queued=true value=Some("107") suffix=1
ToExpression.parsedSource Nested queued=false value=Some("7") suffix=0

thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_nested' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:13:29:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head ... 
thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:49:29:
called `Result::unwrap()` on an `Err` value: PauseCount { expected: 2, actual: 1 }
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_twice ... ToExpression.parsedSource Twice queued=true value=Some("107") suffix=1
ToExpression.parsedSource Twice queued=false value=Some("7") suffix=0

thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_twice' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:13:29:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_without_head ... ok

failures:

failures:
    oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed
    oracle::fixtures::strings_roles::parsed_source::parsed_source_middle
    oracle::fixtures::strings_roles::parsed_source::parsed_source_nested
    oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head
    oracle::fixtures::strings_roles::parsed_source::parsed_source_twice

test result: FAILED. 3 passed; 5 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.14s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```
**red-no-default — captured before production edits.**

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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 28.34s
     Running tests/suspension_oracle.rs (.temp/brief-597/target-tests/debug/deps/suspension_oracle-a9ca1e531e95228c)

running 8 tests
test oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed ... ToExpression.parsedSource AlreadyResumed queued=true value=Some("107") suffix=1
ToExpression.parsedSource AlreadyResumed queued=false value=Some("7") suffix=0

thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:13:29:
called `Result::unwrap()` on an `Err` value: Value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_error ... ToExpression.parsedSource Error queued=true value=None suffix=0
ToExpression.parsedSource Error queued=false value=None suffix=0
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_lifecycle ... ToExpression.parsedSource lifecycle first=PASS second=PASS obligations=12
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_middle ... ToExpression.parsedSource Middle queued=true value=Some("107") suffix=1
ToExpression.parsedSource Middle queued=false value=Some("7") suffix=0

thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_middle' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:13:29:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_nested ... ToExpression.parsedSource Nested queued=true value=Some("107") suffix=1
ToExpression.parsedSource Nested queued=false value=Some("7") suffix=0

thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_nested' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:13:29:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head ... 
thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:49:29:
called `Result::unwrap()` on an `Err` value: PauseCount { expected: 2, actual: 1 }
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_twice ... ToExpression.parsedSource Twice queued=true value=Some("107") suffix=1
ToExpression.parsedSource Twice queued=false value=Some("7") suffix=0

thread 'oracle::fixtures::strings_roles::parsed_source::parsed_source_twice' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/strings_roles/parsed_source.rs:13:29:
called `Result::unwrap()` on an `Err` value: Value
FAILED
test oracle::fixtures::strings_roles::parsed_source::parsed_source_without_head ... ok

failures:

failures:
    oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed
    oracle::fixtures::strings_roles::parsed_source::parsed_source_middle
    oracle::fixtures::strings_roles::parsed_source::parsed_source_nested
    oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head
    oracle::fixtures::strings_roles::parsed_source::parsed_source_twice

test result: FAILED. 3 passed; 5 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.12s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```
**candidate-2-green-default** (the final no-default build is additionally covered by the full oracle below).

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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.15s
     Running tests/suspension_oracle.rs (.temp/brief-597/target-tests/debug/deps/suspension_oracle-08d116f9343550b3)

running 8 tests
test oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed ... ToExpression.parsedSource AlreadyResumed queued=true value=Some("107") suffix=1
ToExpression.parsedSource AlreadyResumed queued=false value=Some("107") suffix=1
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_error ... ToExpression.parsedSource Error queued=true value=None suffix=0
ToExpression.parsedSource Error queued=false value=None suffix=0
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_lifecycle ... ToExpression.parsedSource lifecycle first=PASS second=PASS obligations=12
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_middle ... ToExpression.parsedSource Middle queued=true value=Some("107") suffix=1
ToExpression.parsedSource Middle queued=false value=Some("107") suffix=1
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_nested ... ToExpression.parsedSource Nested queued=true value=Some("107") suffix=1
ToExpression.parsedSource Nested queued=false value=Some("107") suffix=1
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head ... ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_twice ... ToExpression.parsedSource Twice queued=true value=Some("107") suffix=1
ToExpression.parsedSource Twice queued=false value=Some("107") suffix=1
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_without_head ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.13s
```
**candidate-1-green-no-default** (the final no-default build is additionally covered by the full oracle below).

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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.83s
     Running tests/suspension_oracle.rs (.temp/brief-597/target-tests/debug/deps/suspension_oracle-a9ca1e531e95228c)

running 8 tests
test oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed ... ToExpression.parsedSource AlreadyResumed queued=true value=Some("107") suffix=1
ToExpression.parsedSource AlreadyResumed queued=false value=Some("107") suffix=1
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_error ... ToExpression.parsedSource Error queued=true value=None suffix=0
ToExpression.parsedSource Error queued=false value=None suffix=0
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_lifecycle ... ToExpression.parsedSource lifecycle first=PASS second=PASS obligations=12
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_middle ... ToExpression.parsedSource Middle queued=true value=Some("107") suffix=1
ToExpression.parsedSource Middle queued=false value=Some("107") suffix=1
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_nested ... ToExpression.parsedSource Nested queued=true value=Some("107") suffix=1
ToExpression.parsedSource Nested queued=false value=Some("107") suffix=1
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head ... ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_twice ... ToExpression.parsedSource Twice queued=true value=Some("107") suffix=1
ToExpression.parsedSource Twice queued=false value=Some("107") suffix=1
ok
test oracle::fixtures::strings_roles::parsed_source::parsed_source_without_head ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.13s
```
**Final format gate.** `cargo fmt --check` exited 0 with empty output.

**oracle** — verbatim decisive output; full output in `oracle.log`.

```text
VARIANT ToExpression.parsedSource Middle PASS
VARIANT ToExpression.parsedSource Twice PASS
VARIANT ToExpression.parsedSource Error PASS
VARIANT ToExpression.parsedSource AlreadyResumed PASS
VARIANT ToExpression.parsedSource Nested PASS
LIFECYCLE ToExpression.parsedSource first=PASS second=PASS completion/cancel/reset/pending
CATALOG hash=true installed=643 manifest=643 GuestEntries=237 NoLocalGuestEffect=406 Boundary=5 entries=443 descriptors=488 historical=209 owed=38
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.56s
VARIANT ToExpression.parsedSource Middle PASS
VARIANT ToExpression.parsedSource Twice PASS
VARIANT ToExpression.parsedSource Error PASS
VARIANT ToExpression.parsedSource AlreadyResumed PASS
VARIANT ToExpression.parsedSource Nested PASS
LIFECYCLE ToExpression.parsedSource first=PASS second=PASS completion/cancel/reset/pending
CATALOG hash=false installed=637 manifest=637 GuestEntries=237 NoLocalGuestEffect=400 Boundary=5 entries=443 descriptors=488 historical=209 owed=38
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.30s
```
**ownership** — verbatim decisive output; full output in `ownership.log`.

```text
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7680 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 60.27s
```
**certificates-true-replay** — verbatim decisive output; full output in `certificates-true-replay.log`.

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.08s
```
**certificates-true-green** — verbatim decisive output; full output in `certificates-true-green.log`.

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.29s
```
**certificates-false-replay** — verbatim decisive output; full output in `certificates-false-replay.log`.

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.76s
```
**certificates-false-green** — verbatim decisive output; full output in `certificates-false-green.log`.

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.94s
```
**runtime** — verbatim decisive output; full output in `runtime.log`.

```text
test result: ok. 1149 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.73s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.37s
```
**workspace** — verbatim decisive output; full output in `workspace.log`.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.69s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 1149 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.39s
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1090.62s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.08s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.12s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.36s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.67s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.56s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
**native-skips** — verbatim decisive output; full output in `native-skips.log`.

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s
```
Workspace aggregate: 51 suite summaries, 3,408 harness pass records, zero failures and 58 ignored tests. Nine of those pass records are native ZeroMQ fixtures that returned early because loopback binding was denied; they are SKIPS, not executed transport coverage.

Certificate replay initially detects stale promoted certificate cells, emits fresh evidence, and the separate green reruns verify installed cells. Both certificate builds are disclosed; this does not promote additional bodies or change dispositions. Native loopback skips are skips, not executed transport coverage; the workspace uses `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` only for the existing denied-loopback path.

**Oracle inventory.** Exactly one effects descriptor attachment and one owed-row deletion; owed 39 -> 38. Every other oracle row is identical.

```text
--- crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv initial
+++ crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv
@@ -405 +405 @@
-ToExpression.parsedSource	System`ToExpression	parsedSource	crates/rum-stdlib/src/strings/text/builtins.rs	eval_to_expression	Frame	-	Middle,Twice,Error,AlreadyResumed,Nested	crates/rum-stdlib/src/strings/text/builtins.rs#eval_to_expression#parsedSource	strings_text	always
+ToExpression.parsedSource	System`ToExpression	parsedSource	crates/rum-stdlib/src/strings/text/builtins.rs	eval_to_expression	Frame	ToExpression.parsedSource	Middle,Twice,Error,AlreadyResumed,Nested	crates/rum-stdlib/src/strings/text/builtins.rs#eval_to_expression#parsedSource	strings_text	always
```

```text
--- crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv initial
+++ crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv
@@ -39 +38,0 @@
-ToExpression.parsedSource	strings_text	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
```
**Contract fingerprints.** Replay matched the preserved initial tree exactly. These are all 68 changed fingerprint cells; the other 18 fields in each row are unchanged.

```text
row	id	before	after
2	B01	fnv1a64:0c5e6c5b6cf6aff6	fnv1a64:bdcbd1dac1f4a643
3	B02	fnv1a64:771ca69766743f23	fnv1a64:eac2fc49de6159e6
4	B03	fnv1a64:c2d1d5628fd20774	fnv1a64:e6f23dfd0c9360ad
5	B04	fnv1a64:b1574a5468caa7be	fnv1a64:b54b2cae25b1861b
6	B05	fnv1a64:c4d05a6755794877	fnv1a64:d347763153db882a
7	B06	fnv1a64:260ebd93404a27b2	fnv1a64:5f51f3e530b3b9cf
8	B07	fnv1a64:e973ae947f2a87a7	fnv1a64:ea121800fb8e641a
9	B08	fnv1a64:0eb001c46fca25a8	fnv1a64:5c60e4337c218731
10	B09	fnv1a64:e90af331b04dedfa	fnv1a64:beb7e2f859f9be07
11	B10	fnv1a64:687dd190e43f80e9	fnv1a64:c3ea573c443c1618
12	B11	fnv1a64:72aefbb182becdbe	fnv1a64:76a2de0b3fa5ac1b
13	B12	fnv1a64:a817732d8c5682fa	fnv1a64:7dc462f436025307
14	I01	fnv1a64:ea405758d9aea676	fnv1a64:9badbcd82eac9cc3
15	I02	fnv1a64:1c11d7fb4dafd077	fnv1a64:2a88f3c54c12102a
16	I03	fnv1a64:3bfe657127ef583a	fnv1a64:f88c025ea5778547
17	I04	fnv1a64:c07b90a2f2896ac3	fnv1a64:7e55549209d1bd86
18	P01	fnv1a64:5c1529b929856e52	fnv1a64:2ab6a63c96cbbc6f
19	P02	fnv1a64:9328445d408bbb79	fnv1a64:12e9fa5b39e984e8
20	P03	fnv1a64:f661b6c7a6c8694c	fnv1a64:a2475cbe9907d5d5
21	P04	fnv1a64:6bf7c4aceb1d5b23	fnv1a64:df9e1a5f630a75e6
22	P05	fnv1a64:4e47871d02b01586	fnv1a64:c11046a3f88e1cd3
23	P06	fnv1a64:f89697fe18861c7a	fnv1a64:ce4387c4c231ec87
24	P070	fnv1a64:32e4343ff6fb967a	fnv1a64:08912406a0a76687
25	P080	fnv1a64:ebfc113a66fefd29	fnv1a64:7a21af2fbe1c3858
26	P090	fnv1a64:b1e250b5bfcf8163	fnv1a64:f2875e4256f4d426
27	P100	fnv1a64:40bf77c418f19a6e	fnv1a64:7da3c5880d5b3c8b
28	P110	fnv1a64:70f938255ce50081	fnv1a64:9de3cb5c921af560
29	P120	fnv1a64:373a791af3c79fc8	fnv1a64:d35b6afd958e5951
30	P130	fnv1a64:8d4107f1adde61a7	fnv1a64:8ddf715e2a423e1a
31	P140	fnv1a64:0df0ba7f2a3acd9b	fnv1a64:ea055ee0f2d1b32e
32	P150	fnv1a64:b069c3d9286bcf03	fnv1a64:794e0781ca9312c6
33	P071	fnv1a64:ea907877878d64c7	fnv1a64:bff3716e09a6c5ba
34	P081	fnv1a64:f204003fda0cd99a	fnv1a64:3ef46bbd42c84fa7
35	P091	fnv1a64:d68aff3c35c0c092	fnv1a64:824728c3510332af
36	P101	fnv1a64:68fc28f292489d81	fnv1a64:95e6bc29c77e9260
37	P111	fnv1a64:1b481297c022a5ea	fnv1a64:da8fb5a87a734c37
38	P121	fnv1a64:62127798ec09ac65	fnv1a64:fae03d8b9eb68d84
39	P131	fnv1a64:4f016cba9f1ec93e	fnv1a64:52f54f145c05a79b
40	P141	fnv1a64:ff3cc22c07bd46f2	fnv1a64:cc39ae182de9460f
41	P151	fnv1a64:22dae3658a4b3988	fnv1a64:7d9852dc3a1dc411
42	B13	fnv1a64:b41f511c70e6e3b2	fnv1a64:ed62876e615075cf
43	R01	fnv1a64:1212b2fb9824de53	fnv1a64:19319cc747e30756
44	R02	fnv1a64:e00706169317038c	fnv1a64:73f96693d7b98f15
45	R03	fnv1a64:7b3199301240678f	fnv1a64:c37dac4e50ca6b72
46	R040	fnv1a64:83827f93dcbb2373	fnv1a64:7ad6ff2258c418f6
47	R050	fnv1a64:ad8e7a0cdd1ab966	fnv1a64:fbd17737e15c9fb3
48	R060	fnv1a64:03fd04cc7129888b	fnv1a64:545d053287329e9e
49	R070	fnv1a64:5a71c859bbae989a	fnv1a64:a76233d7246a0ea7
50	R080	fnv1a64:877ff995c20e33c3	fnv1a64:4559bd84d9568686
51	R090	fnv1a64:c609ceef79790dae	fnv1a64:5be84ce1cbfdc0cb
52	R100	fnv1a64:87f779b539a7b1f4	fnv1a64:ac17e24fb6690b2d
53	R110	fnv1a64:362ee1dd91ec7831	fnv1a64:1c343d3a9d775210
54	R041	fnv1a64:9a58b446880ea2b2	fnv1a64:d39bea98787834cf
55	R051	fnv1a64:40047ecb4cfd8de3	fnv1a64:80a98c57e422e0a6
56	R061	fnv1a64:a3a1b0995f61a278	fnv1a64:0223e57a26a5ad01
57	R071	fnv1a64:de6a946b515c4f4d	fnv1a64:a14d34a391db379c
58	R081	fnv1a64:fca9e118b49d6f6e	fnv1a64:398e2edca907118b
59	R091	fnv1a64:3535abe704d6f05b	fnv1a64:2264582eaf4fe7ee
60	R101	fnv1a64:66db9c82d52fe3cb	fnv1a64:88ab6ee93fd6b3de
61	R111	fnv1a64:b5bcbef4aeda9890	fnv1a64:f58c16b4b0ce66c9
62	R12	fnv1a64:b9f324b541a74850	fnv1a64:bb43f4c2765d3889
63	R13	fnv1a64:34676e04a70fb144	fnv1a64:8791d2a754b776bd
64	R14	fnv1a64:5b178639f5a25a6e	fnv1a64:97fbd3fdea0bfc8b
65	R15	fnv1a64:daf5eb3e0eb16402	fnv1a64:4fdca9c6d4f475df
66	O010	fnv1a64:7f944dfb2bec0e58	fnv1a64:31bc6ef21105b3e1
67	O0200	fnv1a64:08b85c5378f1f39b	fnv1a64:e4cd00b54188d92e
68	O011	fnv1a64:4685c1547ebb4d0b	fnv1a64:96e5c1ba94c4631e
69	O0210	fnv1a64:fba603b697dfdb30	fnv1a64:6cdb179a72e206e9
```
**Promoted certificates.** These are every changed cell; row identity, strength, status, disposition and exclusions are unchanged.
hash=true

```json
[
  {
    "row": 2,
    "id": "C01",
    "column": 4,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:60187171984596aa\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:2f2645c4f5e35196;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:fba603b697dfdb30;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:4685c1547ebb4d0b;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:60187171984596aa\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:d4343c41be5d2b84;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:08b85c5378f1f39b;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:7f944dfb2bec0e58;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:bab550600cc6ee55\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:738c752da327394b;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:6cdb179a72e206e9;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:96e5c1ba94c4631e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:bab550600cc6ee55\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:490decbfb5b47ef9;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:e4cd00b54188d92e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:31bc6ef21105b3e1;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  },
  {
    "row": 3,
    "id": "C02",
    "column": 4,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:daa73370861fcd84\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:7efe0f9aef94dc99;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:ea405758d9aea676;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:1c11d7fb4dafd077;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:c07b90a2f2896ac3;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:1212b2fb9824de53;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:e00706169317038c;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:b41f511c70e6e3b2;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:c4d05a6755794877;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:cd3186d1b94e96db\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:f341910bcf56eb62;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:9badbcd82eac9cc3;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:2a88f3c54c12102a;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:7e55549209d1bd86;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:19319cc747e30756;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:73f96693d7b98f15;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:ed62876e615075cf;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:d347763153db882a;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  }
]
```
hash=false

```json
[
  {
    "row": 2,
    "id": "C01",
    "column": 5,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:cc7cb093460fadaf\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:afcfd659f0e90d40;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:fba603b697dfdb30;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:4685c1547ebb4d0b;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:cc7cb093460fadaf\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:9938592c91c1c75a;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:08b85c5378f1f39b;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:7f944dfb2bec0e58;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:6d038b97dd6832ae\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:4ed31284f7303128;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:6cdb179a72e206e9;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:96e5c1ba94c4631e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:6d038b97dd6832ae\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:d46c67494aaf5592;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:e4cd00b54188d92e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:31bc6ef21105b3e1;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  },
  {
    "row": 3,
    "id": "C02",
    "column": 5,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:ecaca7a744a00225\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:ab4d0acb90d99eff;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:ea405758d9aea676;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:1c11d7fb4dafd077;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:c07b90a2f2896ac3;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:1212b2fb9824de53;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:e00706169317038c;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:b41f511c70e6e3b2;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:c4d05a6755794877;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:ef88aae5d202f830\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:9f12b2830908c80b;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:9badbcd82eac9cc3;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:2a88f3c54c12102a;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:7e55549209d1bd86;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:19319cc747e30756;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:73f96693d7b98f15;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:ed62876e615075cf;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:d347763153db882a;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  }
]
```
**Ownership reconciliation.** No row disappeared, no identity/status/disposition/exclusion changed, and no new UnknownHostPayload or self-reaching row appeared. Exactly ten rows moved source positions; no new unknown construct was added. The following is the complete reconciliation (including zero-change inventories).

```json
{
  "host_payloads.tsv": {
    "before_count": 65,
    "after_count": 65,
    "changed": [],
    "removed": [],
    "added": []
  },
  "unknown_attributes.tsv": {
    "before_count": 1499,
    "after_count": 1499,
    "changed": [
      {
        "row": 1357,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins/resume_frames.rs\t22\t1\tunresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins/resume_frames.rs\t24\t1\tunresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)",
        "reason": "source position only"
      }
    ],
    "removed": [],
    "added": []
  },
  "unknown_captures.tsv": {
    "before_count": 2946,
    "after_count": 2946,
    "changed": [
      {
        "row": 2607,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t556\t22\tclosure captures: |value| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t565\t22\tclosure captures: |value| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 2608,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t581\t18\tclosure captures: |ch| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t590\t18\tclosure captures: |ch| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 2609,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t684\t9\tclosure captures: |value| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t693\t9\tclosure captures: |value| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 2610,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t699\t9\tclosure captures: |value| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t708\t9\tclosure captures: |value| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 2611,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t699\t35\tclosure captures: |ch| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t708\t35\tclosure captures: |ch| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 2612,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t722\t22\tclosure captures: |value| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t731\t22\tclosure captures: |value| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 2613,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t733\t25\tclosure captures: || (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t742\t25\tclosure captures: || (capture types/modes unresolved)",
        "reason": "source position only"
      }
    ],
    "removed": [],
    "added": []
  },
  "unknown_macros.tsv": {
    "before_count": 3144,
    "after_count": 3144,
    "changed": [
      {
        "row": 2758,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t471\t51\tunexpanded macro vec! (ends at 471:62)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t480\t51\tunexpanded macro vec! (ends at 480:62)",
        "reason": "source position only"
      },
      {
        "row": 2759,
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t473\t16\tunexpanded macro rum_runtime :: resume_or_suspend! (ends at 478:10)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/strings/text/builtins.rs\t482\t16\tunexpanded macro rum_runtime :: resume_or_suspend! (ends at 487:10)",
        "reason": "source position only"
      }
    ],
    "removed": [],
    "added": []
  },
  "unknown_storage.tsv": {
    "before_count": 91,
    "after_count": 91,
    "changed": [],
    "removed": [],
    "added": []
  },
  "self_reaching.tsv": {
    "before_count": 59,
    "after_count": 59,
    "changed": [],
    "added": [],
    "removed": []
  },
  "host_payload_ledger.tsv": {
    "changed": [],
    "added": [],
    "removed": []
  }
}
```
### 3. Repository sweep and instruction/allocation measurements

The fresh syn census parsed 749 production source files across every crate, 8,642 functions, 68,380 calls, 3,584 macros, 575 frame declarations and 564 SuspensionFrame implementations, with zero macro parse failures. All implementation types have captured declarations. There are 600 resume_or_suspend sites (399 runtime, 201 stdlib); no other crate has one and no direct production insertion call bypasses those macros. Empty named/tuple frames were also checked (none). The full site/implementation/success/suffix record is `sweep-sites.json`, with raw AST tables.

The table below has all 107 sites using the 90 unit SuspensionFrame types, the unit RuntimeOption retention continuation, and 74 additional field-carrying sites with an outer success/scope suffix. All 91 original fieldless types are represented. Remaining field-carrying sites place completion in their explicit success arm or typed continuation; their full source/frame records remain in the 600-site census. No additional mapped macro suffix was found. Table locations are the preserved pre-repair source, so G12’s original suffix can be reviewed. `preserved suffix (not a tail)` explicitly distinguishes correctly continued work from a genuine tail; no site with later work is called a correct tail.

Supplemental test/example sweep: all crate Rust files were searched, including test files excluded from the production AST census. Twelve files contain frame-insertion tokens; no additional test/example unit SuspensionFrame was found. The sites below are fixtures or source-parser checks, not omitted production suffixes. The full source reads are reflected in the saved file index `test-and-example-frame-files.json`.

| File/site | Work and verdict |
|---|---|
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:143,209,334 | Banned-token assertions and source inventory checks; no executable insertion. |
| crates/rum-runtime/src/runtime/tests/session_memory_structures.rs:245 | Installs a field-carrying PayloadFrame for memory/root accounting; assertions inspect its retained payload. No guest suffix is delegated to this fixture. |
| crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:277 | Installs the production rule-pattern frame with complete state and pending destination, then tests abandonment; no resumable eager suffix. |
| crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/suspension_frames.rs:279,282,321 | Candidate-bearing current/suffix probes and a values-bearing memory probe; explicit test driving/assertions follow installation. These are host test drivers, not abandoned guest continuations. |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:339 | Installs the production matcher frame with state and pending destination, then abandons it. |
| crates/rum-stdlib/tests/suspension_oracle/mutations/driver.rs:61 | OracleFoldFrame retains operator/items/prefix/next/drop_prefix and resumes the fold. Deliberate OracleDropPrefix and OracleFalseTail negative controls are already exercised by the oracle; they are intentional falsifiers, not new production defects. |
| crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:150 | Allowed-call token list; no insertion. |
| crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:56,235,372,466 | Source-string assertions about production continuations; no test insertion. |
| crates/rum-stdlib/src/tests/retention_closure/local/resume_macro.rs:37,43,168,180 | Parses/generates syntax for the macro checker; these tokens are not executed evaluator insertions. |
| crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs:38 | Contract target identity string; no insertion. |
| crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs:162,172,204,241 | Parsed/quoted macro and hygiene falsifiers, not evaluator frames. |
| crates/rum-stdlib/src/lazy/tests/mod.rs:209 | Installs a fully populated flat lazy frame to verify cancellation without executing invalid function data. |

**Reported group: Some-result wrapping / continuation-marker collision.** `FinishCasesRuleMatchSuspensionFrame`, `TryApplyFunctionSuspensionFrame`, and field-carrying `FinishPositionalPartSelectorEvaluationSuspensionFrame` omit an eager `Some` wrapper. Raw ordinary values decode transparently as Some; `System`InternalContinuationNone[]` collides with the transport tag. Each attached scratch witness has a queued result checked against the intact symbolic value, while resumed execution produces an unexpected second input request. Cases and Function ordinary-value controls and the repaired ToExpression control pass. These are one related group, not three independently localized runtime causes: shared dispatch also interprets these marker-shaped values, so the end-to-end witnesses do not isolate every omitted wrapper as the first point of loss. No repair was made outside G12.

**sweep-final-default** — verbatim witness/result lines; full raw log retained.

```text
test sweep_cases_ordinary_control ... SWEEP Cases.ordinary source=Cases[{1},x_:>(InputString["sweep"];7)] queued=Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(7)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(7)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
test sweep_cases_some_suffix ... SWEEP Cases.rhsSome source=Cases[{1},x_:>(InputString["sweep"];System`InternalContinuationNone[])] queued=Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }]) }), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Err(UnexpectedPause(1))
test sweep_function_ordinary_control ... SWEEP Function.ordinary source=Function[(InputString["sweep"];7)][0] queued=Observation { value: Some(Atom(Integer(Small(7)))), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(7)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
test sweep_function_some_suffix ... SWEEP Function.bodySome source=Function[(InputString["sweep"];System`InternalContinuationNone[])][0] queued=Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Err(UnexpectedPause(1))
test sweep_part_some_suffix ... SWEEP Set.partSome source=Global`sweepList={0};Global`sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationNone[] queued=Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Err(UnexpectedPause(1))
test sweep_toexpression_repaired_control ... SWEEP ToExpression.parsedSource source=ToExpression["InputString[\"sweep\"];7",InputForm,Function[x,x+100]] queued=Observation { value: Some(Atom(Integer(Small(107)))), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(107)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
    sweep_cases_some_suffix
    sweep_function_some_suffix
    sweep_part_some_suffix
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.07s
```
**sweep-final-no-default** — verbatim witness/result lines; full raw log retained.

```text
test sweep_cases_ordinary_control ... SWEEP Cases.ordinary source=Cases[{1},x_:>(InputString["sweep"];7)] queued=Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(7)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(7)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
test sweep_cases_some_suffix ... SWEEP Cases.rhsSome source=Cases[{1},x_:>(InputString["sweep"];System`InternalContinuationNone[])] queued=Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }]) }), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Err(UnexpectedPause(1))
test sweep_function_ordinary_control ... SWEEP Function.ordinary source=Function[(InputString["sweep"];7)][0] queued=Observation { value: Some(Atom(Integer(Small(7)))), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(7)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
test sweep_function_some_suffix ... SWEEP Function.bodySome source=Function[(InputString["sweep"];System`InternalContinuationNone[])][0] queued=Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Err(UnexpectedPause(1))
test sweep_part_some_suffix ... SWEEP Set.partSome source=Global`sweepList={0};Global`sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationNone[] queued=Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Err(UnexpectedPause(1))
test sweep_toexpression_repaired_control ... SWEEP ToExpression.parsedSource source=ToExpression["InputString[\"sweep\"];7",InputForm,Function[x,x+100]] queued=Observation { value: Some(Atom(Integer(Small(107)))), error: None, termination: None, messages: [], prints: [], trace: [] } resumed=Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(107)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
    sweep_cases_some_suffix
    sweep_function_some_suffix
    sweep_part_some_suffix
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.06s
```
Complete sweep table:

| Suspension site | Frame (definition) | Work after completion / state evidence | Verdict | Resume evidence |
|---|---|---|---|---|
| crates/rum-stdlib/src/arithmetic/stats.rs:240 | AveragePairSuspensionFrame (crates/rum-stdlib/src/arithmetic/stats.rs:272) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-stdlib/src/arithmetic/stats.rs:275 |
| crates/rum-stdlib/src/arithmetic.rs:184 | EvalMeanSuspensionFrame (crates/rum-stdlib/src/arithmetic.rs:272) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-stdlib/src/arithmetic.rs:275 |
| crates/rum-stdlib/src/associations/basic/lookup.rs:351 | EvalLookupSuspensionFrame (crates/rum-stdlib/src/associations/basic/resume_frames.rs:95) | Complete the eager suffix using captured frame state: ?; eval_lookup_with_evaluated_data(session, head.clone(), expr, key.clone(), args.clone()) } | preserved suffix (not a tail) | crates/rum-stdlib/src/associations/basic/resume_frames.rs:102 |
| crates/rum-stdlib/src/associations/basic/lookup.rs:374 | EvalLookupWithEvaluatedDataSuspensionFrame (crates/rum-stdlib/src/associations/basic/resume_frames.rs:122) | Complete the eager suffix using captured frame state: ?; finish_eval_lookup(session, head, expr, key, call_args) } | preserved suffix (not a tail) | crates/rum-stdlib/src/associations/basic/resume_frames.rs:129 |
| crates/rum-stdlib/src/functional_ops/arrays/mod.rs:102 | DotSuspensionFrame (crates/rum-stdlib/src/functional_ops/arrays/mod.rs:121) | Consume the completed pair result and continue operands with captured head, args and next_index; suspended pair completion is a successful Some result. | preserved suffix (not a tail) | crates/rum-stdlib/src/functional_ops/arrays/mod.rs:128 |
| crates/rum-stdlib/src/functional_ops/targeted_map.rs:420 | FinishSubsetMapFromMappedSuspensionFrame (crates/rum-stdlib/src/functional_ops/targeted_map/resume_frames.rs:63) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-stdlib/src/functional_ops/targeted_map/resume_frames.rs:66 |
| crates/rum-stdlib/src/functional_ops/targeted_map.rs:450 | FinishSuspendedSubsetMapFromMappedSuspensionFrame (crates/rum-stdlib/src/functional_ops/targeted_map/resume_frames.rs:80) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-stdlib/src/functional_ops/targeted_map/resume_frames.rs:83 |
| crates/rum-stdlib/src/schema.rs:29 | EvalSchemaErrorsSuspensionFrame (crates/rum-stdlib/src/schema.rs:136) | Complete the eager suffix using captured frame state: ?; return eval_schema_errors_with_value(session, value, &args_slice[1..]); } finish_schema_errors(session, None, args_slice) } | preserved suffix (not a tail) | crates/rum-stdlib/src/schema.rs:141 |
| crates/rum-stdlib/src/schema.rs:76 | EvalConformSchemaSuspensionFrame (crates/rum-stdlib/src/schema.rs:157) | Complete the eager suffix using captured frame state: ?; return eval_conform_schema_with_value(session, value, &args_slice[1..]); } finish_conform_schema(session, None, args_slice) } | preserved suffix (not a tail) | crates/rum-stdlib/src/schema.rs:162 |
| crates/rum-stdlib/src/strings/encoding.rs:148 | EvalUncompressSuspensionFrame (crates/rum-stdlib/src/strings/encoding.rs:398) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-stdlib/src/strings/encoding.rs:401 |
| crates/rum-stdlib/src/strings/patterns/search.rs:339 | EvalStringReplaceSuspensionFrame (crates/rum-stdlib/src/strings/patterns/search/resume_frames.rs:5) | Construct a string expression from the completed rendering. | preserved suffix (not a tail) | crates/rum-stdlib/src/strings/patterns/search/resume_frames.rs:8 |
| crates/rum-stdlib/src/strings/patterns/search.rs:426 | EvalStringCasesSuspensionFrame1 (crates/rum-stdlib/src/strings/patterns/search/resume_frames.rs:28) | Construct List from the completed case vector. | preserved suffix (not a tail) | crates/rum-stdlib/src/strings/patterns/search/resume_frames.rs:31 |
| crates/rum-stdlib/src/strings/patterns/search.rs:451 | EvalStringCasesSuspensionFrame2 (crates/rum-stdlib/src/strings/patterns/search/resume_frames.rs:51) | Construct List from the completed case vector. | preserved suffix (not a tail) | crates/rum-stdlib/src/strings/patterns/search/resume_frames.rs:54 |
| crates/rum-stdlib/src/strings/text/builtins.rs:463 | EvalToExpressionSuspensionFrame1 (crates/rum-stdlib/src/strings/text/builtins/resume_frames.rs:5) | Apply the optional third-argument result head after parsed-source evaluation. Repaired in candidate 1 by retained result_head and shared finish_to_expression. | lost suffix (repaired) | crates/rum-stdlib/src/strings/text/builtins/resume_frames.rs:8 |
| crates/rum-stdlib/src/strings/text/builtins.rs:473 | EvalToExpressionSuspensionFrame2 (crates/rum-stdlib/src/strings/text/builtins/resume_frames.rs:22) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-stdlib/src/strings/text/builtins/resume_frames.rs:25 |
| crates/rum-stdlib/src/structure_ops/list_plumbing.rs:67 | DifferencesOrderSuspensionFrame (crates/rum-stdlib/src/structure_ops/list_plumbing.rs:84) | Continue remaining difference orders and check generated element count; remaining_order plus completed row carry the suffix. | preserved suffix (not a tail) | crates/rum-stdlib/src/structure_ops/list_plumbing.rs:89 |
| crates/rum-runtime/src/runtime/accounting_mode/frozen.rs:133 | RetentionFrame (crates/rum-runtime/src/runtime/accounting_mode/frozen.rs:27) | Transfer the receipt and enforce memory accounting immediately after installation on the suspension branch; the continuation and retained prefix remain in RetentionFrame. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/accounting_mode/frozen.rs:33 |
| crates/rum-runtime/src/runtime/attributes.rs:83 | EvalUpdateAttributesSuspensionFrame (crates/rum-runtime/src/runtime/attributes.rs:226) | Continue via finish_update_attributes_after_resolved_attributes. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/attributes.rs:234 |
| crates/rum-runtime/src/runtime/builtin_adapters.rs:934 | BuiltinEvaluateSuspensionFrame (crates/rum-runtime/src/runtime/builtin_adapters/resume_frames.rs:4) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/builtin_adapters/resume_frames.rs:7 |
| crates/rum-runtime/src/runtime/builtins_input.rs:65 | EvalDialogSuspensionFrame (crates/rum-runtime/src/runtime/builtins_input.rs:454) | Continue via finish_dialog_body. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/builtins_input.rs:457 |
| crates/rum-runtime/src/runtime/builtins_input.rs:94 | EvalInputSuspensionFrame (crates/rum-runtime/src/runtime/builtins_input.rs:468) | Continue via resume_input_after_text. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/builtins_input.rs:471 |
| crates/rum-runtime/src/runtime/builtins_input.rs:312 | ResumeInputAfterTextSuspensionFrame (crates/rum-runtime/src/runtime/builtins_input.rs:485) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/builtins_input.rs:488 |
| crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:23 | EvalEvaluationDataSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:208) | Continue via finish_evaluation_data_capture. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:218 |
| crates/rum-runtime/src/runtime/debug_tools/monitor.rs:36 | EvalMonitorSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/monitor.rs:129) | Continue via finish_monitor. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/monitor.rs:132 |
| crates/rum-runtime/src/runtime/debug_tools/reap.rs:38 | EvalReapSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/reap.rs:351) | Continue via resume_reap_after_body. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/reap.rs:354 |
| crates/rum-runtime/src/runtime/debug_tools/reap.rs:62 | ResumeReapAfterBodySuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/reap.rs:375) | Complete the eager suffix using captured frame state: ?; Ok(finish_reap_result(value, rendered)) } Err(error) => Err(error), } } | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/reap.rs:380 |
| crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:20 | EvalTimeConstrainedSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:319) | Continue via resume_time_constrained_with_limit. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:326 |
| crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:54 | ResumeTimeConstrainedWithLimitSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:353) | Continue via finish_time_constrained. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:359 |
| crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:84 | EvalMemoryConstrainedSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:384) | Continue via resume_memory_constrained_with_limit. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:391 |
| crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:118 | ResumeMemoryConstrainedWithLimitSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:418) | Continue via finish_memory_constrained. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:424 |
| crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:256 | FinishTimeConstrainedSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:451) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:454 |
| crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:282 | FinishMemoryConstrainedSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:468) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:471 |
| crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:116 | TestCaseOutputMatchesPartsSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:403) | Test the returned value for True and encode the boolean. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:406 |
| crates/rum-runtime/src/runtime/debug_tools/timing.rs:58 | EvalTimingLikeSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/timing.rs:156) | Continue via finish_timing_like. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/timing.rs:162 |
| crates/rum-runtime/src/runtime/debug_tools/timing.rs:102 | ResumeRepeatedTimingFromSampleSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/timing.rs:180) | Continue via resume_repeated_timing_with_value. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/timing.rs:188 |
| crates/rum-runtime/src/runtime/debug_tools/trace.rs:71 | EvalStructuredTraceCaptureExprSuspensionFrame (crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:4) | Continue via finish_structured_trace_capture_expr. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:11 |
| crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:88 | ApplyReplaceAllSuspensionFrame (crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:4) | Extract the rewritten expression from ReplaceAllTraversalOutcome. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:7 |
| crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:103 | ApplyReplaceAllWithPlanSuspensionFrame (crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:25) | Extract the rewritten expression from ReplaceAllTraversalOutcome. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:28 |
| crates/rum-runtime/src/runtime/definitions/application/rhs.rs:53 | EvaluateDefinitionRhsSuspensionFrame (crates/rum-runtime/src/runtime/definitions/application/rhs.rs:73) | Continue via finish_definition_rhs_evaluation. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/application/rhs.rs:76 |
| crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:150 | ResolveCasesSpecExprForMatchingOwnedSuspensionFrame (crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:327) | Continue via compile_rewrite_rule_for_matching, normalize_match_expr. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:330 |
| crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:185 | ResolveRewriteRuleExprsOwnedSuspensionFrame (crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:359) | Continue via normalize_rewrite_rule_exprs_from_resolved. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:362 |
| crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:217 | DriveRewriteRuleInfosOwnedSuspensionFrame (crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:410) | Run InspectRewriteRules; it is the only RewriteRuleInfoContinuation variant (rewrite_rules.rs:87-110). | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:413 |
| crates/rum-runtime/src/runtime/definitions/definition_tags.rs:15 | EvalUpSetSuspensionFrame (crates/rum-runtime/src/runtime/definitions/definition_tags.rs:253) | Continue via prepare_up_set_assignment. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/definition_tags.rs:258 |
| crates/rum-runtime/src/runtime/definitions/definition_tags.rs:58 | EvalTagSetSuspensionFrame (crates/rum-runtime/src/runtime/definitions/definition_tags.rs:272) | Continue via prepare_tag_set_assignment. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/definition_tags.rs:278 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs:122 | EvalRuleSuspensionFrame (crates/rum-runtime/src/runtime/definitions/definitions/resume_frames.rs:25) | Continue via resume_rule_with_rhs. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/definitions/resume_frames.rs:30 |
| crates/rum-runtime/src/runtime/definitions/part_assignment.rs:43 | FinishPositionalPartSelectorEvaluationSuspensionFrame (crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs:4) | Finish positional-part assignment then return Some(rhs); frame carries owner/rhs but omits the Some transport wrapper. | lost suffix (reported; marker witness) | crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs:10 |
| crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:394 | EvaluateDefinitionRhsGuardsForMatchSuspensionFrame (crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:128) | Extract and encode the matched boolean. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:131 |
| crates/rum-runtime/src/runtime/hold_thread.rs:26 | EvalReleaseHoldSuspensionFrame (crates/rum-runtime/src/runtime/hold_thread.rs:114) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/hold_thread.rs:117 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/options.rs:56 | EvalCheckAbortSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:4) | Continue via resume_check_abort_with_propagation. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:9 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:127 | ResumeCheckAbortWithPropagationSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:30) | Continue via resume_check_abort_after_primary. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:38 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:156 | EvalAbortProtectSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:69) | Continue via resume_abort_protect_after_body. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:74 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:179 | ResumeCheckAbortAfterPrimarySuspensionFrame1 (crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:97) | Continue via resume_check_abort_after_fallback. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:104 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:202 | ResumeCheckAbortAfterPrimarySuspensionFrame2 (crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:131) | Continue via resume_check_abort_after_fallback. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:139 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:9 | EvalAssertSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:156) | Continue via resume_assert_after_function. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:162 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:85 | ResumeAssertAfterFunctionSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:180) | Continue via resume_assert_after_test. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:187 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:116 | FinishAssertFunctionSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:205) | Discard the callback result and return Null. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:208 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:72 | ResumeDoAfterSpecEvalSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:4) | Continue via finish_do_spec_eval. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:10 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:207 | ResumeTableAfterSpecEvalSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:64) | Continue via finish_table_spec_eval. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:70 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:383 | EvaluateLoopIteratorBodySuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:124) | Package the symbol/saved scope with the body outcome; following ContinueLoopIteratorScopeFrame restores the scope. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:130 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:396 | ContinueLoopIteratorScopeFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:168) | Continue via finish_loop_iterator_scope. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:171 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:411 | EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:186) | Continue via finish_loop_iterator_dynamic_scope. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:192 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:83 | ResumeIterationAggregateAfterSpecEvalSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:254) | Continue via finish_iteration_aggregate_spec_eval. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:261 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:52 | EvalIfSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:4) | Continue via resume_if_after_condition. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:9 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:88 | EvalSwitchSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:25) | Continue via resume_switch_after_subject. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:30 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:245 | ResumeUntilLoopSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:82) | Continue via resume_until_loop. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:88 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:307 | ResumeForLoopSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:103) | Continue via resume_for_loop. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:109 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:408 | ResumeWhichFromIndexSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:147) | Continue via resume_which_after_condition. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/resume_frames.rs:153 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:27 | EvalCheckSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:171) | Continue via resume_check_after_primary. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:179 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:70 | EvalQuietSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:221) | Continue via resume_quiet_after_primary. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:228 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:20 | EvalWithCleanupSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:149) | Continue via resume_with_cleanup_after_init. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:155 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:64 | DriveWithCleanupPrimarySuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:179) | Continue via resume_with_cleanup_after_primary. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:184 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:25 | EvalEncloseSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:4) | Continue via resume_enclose_after_body. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:11 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:53 | EvalConfirmSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:28) | Continue via resume_confirm_after_value. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:33 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:76 | EvalConfirmBySuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:49) | Continue via resume_confirm_by_after_value. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:54 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:99 | EvalConfirmMatchSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:70) | Continue via resume_confirm_match_after_value. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:75 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:134 | EvalConfirmQuietSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:91) | Continue via resume_suspended_confirm_quiet_after_body. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:99 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:173 | EvalConfirmAssertSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:128) | Continue via resume_confirm_assert_after_test. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:133 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:332 | ResumeConfirmByAfterValueSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:263) | Continue via resume_confirm_by_after_test. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins/resume_frames.rs:269 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:181 | EvalCatchExceptionsSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:4) | Continue via resume_suspended_catch_exceptions_after_body. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:11 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:456 | EvaluateExceptionHandlerRhsSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:109) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:112 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:96 | EvalCatchSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:351) | Continue via resume_catch_after_body. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:357 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:168 | ResumeCatchForThrowSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:372) | Continue via finish_catch_for_throw_with_form. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:380 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:241 | ResumeCatchForBypassingThrowSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:448) | Continue via finish_catch_for_bypassing_throw_with_form. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:458 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:204 | FailureVirtualStyledMessageValueSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/status/resume_frames.rs:4) | Style the completed message and return Some(style); resumed style is unambiguously decoded as Some by Option<Expr>. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/status/resume_frames.rs:7 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:229 | FailureMessageFromTemplateSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/status/resume_frames.rs:23) | Render parameters with captured status/template, then Some(rendered string); string results are unambiguously decoded as Some. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/status/resume_frames.rs:29 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:272 | FinishFailureMessageFromTemplateSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/control_data/status/resume_frames.rs:50) | Construct a string expression from rendered text. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/control_data/status/resume_frames.rs:53 |
| crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:122 | FinishEvaluatedPartSelectorsSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:159) | Wrap evaluated selector values in EvaluatedPartSelectors. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:162 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:38 | ContinueCasesFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:100) | Continue via resume_cases_after_target_suspended, resume_cases_after_resolved_spec_suspended. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:103 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:70 | ContinueCasesFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:100) | Continue via resume_cases_after_target_suspended, resume_cases_after_resolved_spec_suspended. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:103 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:93 | ContinueCasesFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:100) | Continue via resume_cases_after_target_suspended, resume_cases_after_resolved_spec_suspended. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:103 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:177 | FinishCasesWithTraversalSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:176) | Construct List from completed case matches. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:179 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:198 | FinishCasesWithTraversalSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:176) | Construct List from completed case matches. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:179 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:442 | FinishCasesRuleMatchSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:294) | Wrap the evaluated rule RHS in Some; frame returns the raw expression, allowing continuation-marker collision. | lost suffix (reported; marker witness) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:297 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/options.rs:64 | ContinueDeleteCasesFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:126) | Continue via resume_delete_cases_after_target_suspended, resume_delete_cases_after_pattern_suspended, finish_delete_cases_after_resolved_traversal_suspended. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:129 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:37 | ContinueDeleteCasesFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:126) | Continue via resume_delete_cases_after_target_suspended, resume_delete_cases_after_pattern_suspended, finish_delete_cases_after_resolved_traversal_suspended. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:129 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:66 | ContinueDeleteCasesFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:126) | Continue via resume_delete_cases_after_target_suspended, resume_delete_cases_after_pattern_suspended, finish_delete_cases_after_resolved_traversal_suspended. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:129 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:89 | ContinueDeleteCasesFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:126) | Continue via resume_delete_cases_after_target_suspended, resume_delete_cases_after_pattern_suspended, finish_delete_cases_after_resolved_traversal_suspended. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:129 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:149 | FinishDeleteCasesAfterResolvedTraversalSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:189) | Convert Keep/Delete/Splice into expression/empty Sequence/Sequence. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:192 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:182 | FinishDeleteCasesAfterResolvedTraversalSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:189) | Convert Keep/Delete/Splice into expression/empty Sequence/Sequence. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:192 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:26 | EvalMatchQSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:144) | Continue via resume_match_q_after_value. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:149 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:60 | ResumeMatchQAfterValueSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:165) | Continue via finish_match_q_with_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:170 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:83 | FinishMatchQWithPatternSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:186) | Convert the matched outcome to True or False. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:189 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:17 | EvalReplaceAllSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:141) | Continue via resume_replace_all_after_target. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:146 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:64 | ResumeReplaceAllAfterRulesSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:162) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:165 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:84 | EvalReplaceAllUnheldSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:179) | Continue via resume_replace_all_unheld_after_target. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:184 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:132 | ResumeReplaceAllUnheldAfterRulesSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:200) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:203 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:70 | ContinueReplaceListInitialGuardFilterFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:75) | Continue via filter_replace_list_guard_bindings_owned. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:78 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:104 | ContinueReplaceListCallMatchFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:155) | Continue via finish_replace_list_call_head_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:158 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:131 | CollectReplaceListRuleBindingsOwnedSuspensionFrame3 (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:216) | Convert a matched outcome into one binding map, otherwise an empty vector. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:219 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:227 | ContinueReplaceListGuardFilterFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:285) | Continue via filter_replace_list_guard_bindings_owned. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:288 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:31 | ContinueReplaceListSetupFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:359) | Continue via resume_replace_list_after_target, finish_replace_list_limit. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:362 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:67 | ContinueReplaceListSetupFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:359) | Continue via resume_replace_list_after_target, finish_replace_list_limit. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:362 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:137 | ResumeReplaceListAfterRuleExprsSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:417) | Construct List from completed replacement results. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:420 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:243 | FinishReplaceLevelsResolutionSuspensionFrame2 (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:258) | Complete the eager suffix using captured frame state: })() .map(\|value\| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value)) } Err(error) => Err(error), }; crate::runtime::suspension::FrameAction::complete(resumed) } | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:261 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:207 | FinishReplaceLevelsResolutionSuspensionFrame1 (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:229) | Evaluate the replaced expression; frame 2 carries any subsequent suspension. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:232 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:212 | FinishReplaceLevelsResolutionSuspensionFrame2 (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:258) | No work after completion on this branch; return the evaluated value. | correct tail | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:261 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:259 | ResolveReplaceLevelSpecOwnedSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:311) | Parse the completed expression as a standard level specification. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:314 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:34 | ContinueReplaceRepeatedTargetFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:379) | Continue via resume_replace_repeated_after_current. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:382 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:269 | ResumeReplaceRepeatedLoopSuspensionFrame (crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:498) | Continue via resume_replace_repeated_after_next. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:506 |
| crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:30 | ContinueReplaceAtSetupFrame (crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:324) | Continue via resume_replace_at_after_target, finish_replace_at_positions. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:327 |
| crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:61 | ContinueReplaceAtSetupFrame (crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:324) | Continue via resume_replace_at_after_target, finish_replace_at_positions. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:327 |
| crates/rum-runtime/src/runtime/options/builtins.rs:49 | EvalSetOptionsSuspensionFrame (crates/rum-runtime/src/runtime/options/builtins.rs:491) | Continue via finish_set_options_from_resolved. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/options/builtins.rs:498 |
| crates/rum-runtime/src/runtime/options/builtins.rs:133 | EvalOptionValueSuspensionFrame (crates/rum-runtime/src/runtime/options/builtins.rs:525) | Continue via finish_option_value_from_explicit. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/options/builtins.rs:532 |
| crates/rum-runtime/src/runtime/options/patterns.rs:34 | EvaluateWithOptionValueScopeSuspensionFrame (crates/rum-runtime/src/runtime/options/patterns.rs:178) | Continue via refresh_memory_owner. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/options/patterns.rs:181 |
| crates/rum-runtime/src/runtime/pattern_matching/core/key_value.rs:81 | ContinueKeyValueKeyMatchFrame (crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:637) | Continue via finish_key_value_key_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:640 |
| crates/rum-runtime/src/runtime/pattern_matching/core/key_value.rs:116 | ContinueKeyValueValueMatchFrame (crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:661) | Continue via finish_key_value_value_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:664 |
| crates/rum-runtime/src/runtime/pattern_matching/core.rs:88 | ContinueCompiledAlternativeFrame (crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:141) | Continue via finish_compiled_alternative. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:144 |
| crates/rum-runtime/src/runtime/pattern_matching/core.rs:213 | ContinueCompiledExceptFrame (crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:345) | Continue via finish_compiled_except_inner. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:348 |
| crates/rum-runtime/src/runtime/pattern_matching/core.rs:237 | FinishCompiledExceptInnerSuspensionFrame (crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:381) | Negate matched while retaining the returned bindings. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:384 |
| crates/rum-runtime/src/runtime/pattern_matching/core.rs:287 | ContinueCompiledCallFrame (crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:468) | Continue via finish_compiled_call_head. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:471 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:193 | ContinueSequenceChoiceResultFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:332) | Continue via finish_sequence_choice. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:335 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:229 | ContinueSequenceChoiceRestFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:411) | Continue via finish_sequence_choice_rest. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:414 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:224 | ContinueSequenceChoiceCollectionFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:371) | Continue via finish_sequence_choice_collection, finish_sequence_choice_rest_collection. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:374 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:261 | ContinueSequenceChoiceCollectionFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:371) | Continue via finish_sequence_choice_collection, finish_sequence_choice_rest_collection. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:374 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:247 | ContinueOptionalPresentFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:44) | Continue via finish_optional_present. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:47 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:348 | ContinueOptionalPresentCollectionFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:112) | Continue via finish_optional_present_collection. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:115 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:459 | ContinueOptionalDefaultFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:220) | Continue via finish_optional_default. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:223 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:496 | ContinueOptionalDefaultMatchFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:288) | Continue via finish_optional_default_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:291 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs:33 | ContinueCompiledSequenceItemsFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs:181) | Continue via finish_compiled_sequence_item. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs:184 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs:84 | ContinueCompiledSequencePatternFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs:276) | Continue via finish_compiled_sequence_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs:279 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:59 | ContinueMatchCompiledOrderedArgsFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:181) | Continue via match_compiled_ordered_args_iter_owned. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:184 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:121 | ContinueCollectCompiledOrderedArgsFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:255) | Continue via collect_compiled_ordered_args_iter_owned. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:258 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:214 | ContinueOrderlessChoiceCollectionFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:363) | Continue via finish_orderless_choice_collection, finish_orderless_choice_rest_collection. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:366 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:251 | ContinueOrderlessChoiceCollectionFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:363) | Continue via finish_orderless_choice_collection, finish_orderless_choice_rest_collection. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:366 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:237 | ContinueOrderlessChoicePatternFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:367) | Continue via finish_orderless_choice_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:370 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:273 | ContinueOrderlessChoiceRestFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:446) | Continue via finish_orderless_choice_rest. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:449 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:157 | ContinueOrderlessOptionalFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:219) | Continue via finish_orderless_optional_present, finish_orderless_optional_default, finish_orderless_optional_default_match, finish_orderless_optional_present_collection, finish_orderless_optional_collection_default, finish_orderless_optional_collection_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:222 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:186 | ContinueOrderlessOptionalFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:219) | Continue via finish_orderless_optional_present, finish_orderless_optional_default, finish_orderless_optional_default_match, finish_orderless_optional_present_collection, finish_orderless_optional_collection_default, finish_orderless_optional_collection_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:222 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:221 | ContinueOrderlessOptionalFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:219) | Continue via finish_orderless_optional_present, finish_orderless_optional_default, finish_orderless_optional_default_match, finish_orderless_optional_present_collection, finish_orderless_optional_collection_default, finish_orderless_optional_collection_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:222 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:313 | ContinueOrderlessOptionalFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:219) | Continue via finish_orderless_optional_present, finish_orderless_optional_default, finish_orderless_optional_default_match, finish_orderless_optional_present_collection, finish_orderless_optional_collection_default, finish_orderless_optional_collection_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:222 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:349 | ContinueOrderlessOptionalFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:219) | Continue via finish_orderless_optional_present, finish_orderless_optional_default, finish_orderless_optional_default_match, finish_orderless_optional_present_collection, finish_orderless_optional_collection_default, finish_orderless_optional_collection_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:222 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:384 | ContinueOrderlessOptionalFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:219) | Continue via finish_orderless_optional_present, finish_orderless_optional_default, finish_orderless_optional_default_match, finish_orderless_optional_present_collection, finish_orderless_optional_collection_default, finish_orderless_optional_collection_match. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:222 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:321 | ContinueMatchCompiledArgAndRestFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:62) | Continue via match_compiled_args_owned. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:65 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:452 | ContinueFlatSinglePatternFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:129) | Continue via finish_flat_single_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:132 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:481 | ContinueFlatSingleRestFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:199) | Continue via finish_flat_single_rest. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:202 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:579 | ContinueCollectCompiledArgAndRestFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:270) | Continue via collect_compiled_args_matches_owned. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:273 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:720 | ContinueFlatSingleCollectionPatternFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:340) | Continue via finish_flat_single_collection_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:343 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:748 | ContinueFlatSingleCollectionRestFrame (crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:416) | Continue via finish_flat_single_collection_rest. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:419 |
| crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:100 | ContinueInterpretedExceptFrame (crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:627) | Continue via finish_prepared_interpreted_except. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:630 |
| crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:270 | ContinueInterpretedAlternativeFrame (crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:902) | Continue via finish_prepared_interpreted_alternative. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:905 |
| crates/rum-runtime/src/runtime/scoping/callable.rs:65 | TryApplyFunctionSuspensionFrame (crates/rum-runtime/src/runtime/scoping/callable.rs:141) | finish_function_application then map(Some); frame calls the finisher but omits the Some transport wrapper. | lost suffix (reported; marker witness) | crates/rum-runtime/src/runtime/scoping/callable.rs:144 |
| crates/rum-runtime/src/runtime/scoping/lexical/block.rs:115 | ContinueSingleBlockInitializerFrame (crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:47) | Continue via resume_single_symbol_block_after_initializer. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:50 |
| crates/rum-runtime/src/runtime/scoping/lexical/block.rs:345 | ContinueBlockInitializerFrame (crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:113) | Continue via resume_block_after_initializer. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:116 |
| crates/rum-runtime/src/runtime/scoping/lexical/block.rs:424 | ContinueSingleBlockBodyFrame (crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:181) | Continue via finish_single_symbol_block_scope. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:184 |
| crates/rum-runtime/src/runtime/scoping/lexical/block.rs:556 | ContinueBlockBodyFrame (crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:246) | Continue via finish_block_scope. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:249 |
| crates/rum-runtime/src/runtime/scoping/lexical/module.rs:131 | ResumeModuleInitializersSuspensionFrame1 (crates/rum-runtime/src/runtime/scoping/lexical/module.rs:408) | Continue via finish_module_scope. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/lexical/module.rs:413 |
| crates/rum-runtime/src/runtime/scoping/lexical/module.rs:175 | ResumeModuleInitializersSuspensionFrame2 (crates/rum-runtime/src/runtime/scoping/lexical/module.rs:436) | Continue via resume_suspended_module_initializers. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/lexical/module.rs:441 |
| crates/rum-runtime/src/runtime/scoping/lexical/module.rs:233 | ResumeSuspendedModuleInitializersSuspensionFrame1 (crates/rum-runtime/src/runtime/scoping/lexical/module.rs:464) | Continue via finish_module_scope. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/lexical/module.rs:469 |
| crates/rum-runtime/src/runtime/scoping/lexical/module.rs:274 | ResumeSuspendedModuleInitializersSuspensionFrame2 (crates/rum-runtime/src/runtime/scoping/lexical/module.rs:496) | Continue via resume_suspended_module_initializers. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/lexical/module.rs:501 |
| crates/rum-runtime/src/runtime/scoping/packages.rs:403 | EvalRestoreSealedReferencesSuspensionFrame (crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs:4) | Discard the restored-body result and return Null. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs:7 |
| crates/rum-runtime/src/runtime/scoping/packages.rs:450 | ContinuePackageInstallFrame (crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs:71) | Continue via resume_package_install. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs:74 |
| crates/rum-runtime/src/runtime/session/session_api.rs:716 | EvaluateWithRandomStateRestoredSuspensionFrame (crates/rum-runtime/src/runtime/session/session_api/resume_frames.rs:4) | Complete the eager suffix using captured frame state: ; match outcome { Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }), other => { self.prng_state = saved; other } } } | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/session/session_api/resume_frames.rs:9 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:45 | EvalCountSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:4) | Continue via resume_count_after_target. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:9 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:71 | EvalMemberQSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:25) | Continue via resume_member_q_after_target. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:30 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:94 | EvalFreeQSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:46) | Continue via resume_free_q_after_target. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:51 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:113 | ResumeCountAfterTargetSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:67) | Continue via resume_count_after_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:73 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:152 | ResumeMemberQAfterTargetSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:91) | Continue via resume_member_q_after_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:97 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191 | ResumeFreeQAfterTargetSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:115) | Continue via resume_free_q_after_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:121 |
| crates/rum-runtime/src/runtime/tree_walkers/position.rs:108 | EvalPositionSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:4) | Continue via resume_position_after_target. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:9 |
| crates/rum-runtime/src/runtime/tree_walkers/position.rs:127 | ResumePositionAfterTargetSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:25) | Continue via resume_position_after_pattern. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:31 |
| crates/rum-runtime/src/runtime/tree_walkers/position.rs:178 | FinishPositionAfterResolvedTraversalSuspensionFrame (crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:76) | Construct List from completed positions. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:79 |
| crates/rum-runtime/src/runtime/accounting_mode/bridge.rs:257 | RuntimeOptionSuspensionFrame (crates/rum-runtime/src/runtime/options/resume_frames.rs:12) | Reconstruct the option Rule from a retained name and completed value; enclosing RetentionFrame owns the retained prefix. | preserved suffix (not a tail) | crates/rum-runtime/src/runtime/options/resume_frames.rs:14 |

**Performance protocol.** Each label’s result was checked before measurement; all six labels use N=20,000. Controls use the preserved brief-563 workload files. Allocations were measured before instructions. Every metric ran one label per process, stdin /dev/null, bounded at 150 s. Instruction command: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`; slope `(count(2N)-count(N))/N`. Raw outputs were closed before parsing. No timing suite ran. Two base/base pairs established each label’s floor; a regression requires both candidate deltas to exceed max(0.05%, floor).
Every noise-floor row:

```json
[
  {
    "label": "toexpression-no-head",
    "pairs": [
      {
        "a": 6470.3169,
        "b": 6470.4306,
        "percent": 0.001757255506285027
      },
      {
        "a": 6470.5456,
        "b": 6470.4263,
        "percent": -0.0018437394213033365
      }
    ],
    "floor_percent": 0.0018437394213033365,
    "threshold_percent": 0.05
  },
  {
    "label": "toexpression-head",
    "pairs": [
      {
        "a": 20606.29525,
        "b": 20567.02035,
        "percent": -0.19059660906295406
      },
      {
        "a": 20606.11715,
        "b": 20684.09625,
        "percent": 0.3784269468738888
      }
    ],
    "floor_percent": 0.3784269468738888,
    "threshold_percent": 0.3784269468738888
  },
  {
    "label": "toexpression-arithmetic",
    "pairs": [
      {
        "a": 53236.8571,
        "b": 53236.44315,
        "percent": -0.0007775628062089979
      },
      {
        "a": 53237.53975,
        "b": 53236.4285,
        "percent": -0.002087342888534849
      }
    ],
    "floor_percent": 0.002087342888534849,
    "threshold_percent": 0.05
  },
  {
    "label": "dot-three-no-input",
    "pairs": [
      {
        "a": 254199.3996,
        "b": 254198.49165,
        "percent": -0.00035718022993735943
      },
      {
        "a": 254200.2276,
        "b": 254198.8344,
        "percent": -0.000548071893236024
      }
    ],
    "floor_percent": 0.000548071893236024,
    "threshold_percent": 0.05
  },
  {
    "label": "differences-no-input",
    "pairs": [
      {
        "a": 10030.8108,
        "b": 10044.851,
        "percent": 0.1399707389556415
      },
      {
        "a": 10037.75015,
        "b": 10030.6658,
        "percent": -0.07057707050019957
      }
    ],
    "floor_percent": 0.1399707389556415,
    "threshold_percent": 0.1399707389556415
  },
  {
    "label": "take-multispec-small",
    "pairs": [
      {
        "a": 49915.15665,
        "b": 49922.8315,
        "percent": 0.015375790671789957
      },
      {
        "a": 49915.18025,
        "b": 49915.4674,
        "percent": 0.0005752758951649284
      }
    ],
    "floor_percent": 0.015375790671789957,
    "threshold_percent": 0.05
  }
]
```
**candidate-1 — every paired instruction and allocation row.**

```json
[
  {
    "label": "toexpression-no-head",
    "N": 20000,
    "floor_percent": 0.0018437394213033365,
    "threshold_percent": 0.05,
    "instructions": [
      {
        "sample": "1",
        "base": 6470.4987,
        "candidate": 6492.3725,
        "delta_percent": 0.3380543141133785
      },
      {
        "sample": "2",
        "base": 6470.55115,
        "candidate": 6492.58285,
        "delta_percent": 0.3404918605735663
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 10.0,
          "bytes": 438.0
        },
        "candidate-1": {
          "allocations": 10.0,
          "bytes": 438.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 10.0,
          "bytes": 438.0
        },
        "candidate-1": {
          "allocations": 10.0,
          "bytes": 438.0
        }
      }
    ],
    "regression": true
  },
  {
    "label": "toexpression-head",
    "N": 20000,
    "floor_percent": 0.3784269468738888,
    "threshold_percent": 0.3784269468738888,
    "instructions": [
      {
        "sample": "1",
        "base": 20645.33865,
        "candidate": 20625.13105,
        "delta_percent": -0.09787972162907943
      },
      {
        "sample": "2",
        "base": 20645.26705,
        "candidate": 20625.21235,
        "delta_percent": -0.0971394555053573
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 27.0,
          "bytes": 1330.0
        },
        "candidate-1": {
          "allocations": 27.0,
          "bytes": 1330.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 27.0,
          "bytes": 1330.0
        },
        "candidate-1": {
          "allocations": 27.0,
          "bytes": 1330.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "toexpression-arithmetic",
    "N": 20000,
    "floor_percent": 0.002087342888534849,
    "threshold_percent": 0.05,
    "instructions": [
      {
        "sample": "1",
        "base": 53236.72345,
        "candidate": 53258.75375,
        "delta_percent": 0.0413817729047361
      },
      {
        "sample": "2",
        "base": 53236.6343,
        "candidate": 53259.73545,
        "delta_percent": 0.0433933330003855
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 123.0,
          "bytes": 5634.0
        },
        "candidate-1": {
          "allocations": 123.0,
          "bytes": 5634.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 123.0,
          "bytes": 5634.0
        },
        "candidate-1": {
          "allocations": 123.0,
          "bytes": 5634.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "dot-three-no-input",
    "N": 20000,
    "floor_percent": 0.000548071893236024,
    "threshold_percent": 0.05,
    "instructions": [
      {
        "sample": "1",
        "base": 254198.62735,
        "candidate": 254198.12835,
        "delta_percent": -0.00019630318431884675
      },
      {
        "sample": "2",
        "base": 254199.73495,
        "candidate": 254199.64505,
        "delta_percent": -3.536589053831918e-05
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 596.0,
          "bytes": 23504.0
        },
        "candidate-1": {
          "allocations": 596.0,
          "bytes": 23504.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 596.0,
          "bytes": 23504.0
        },
        "candidate-1": {
          "allocations": 596.0,
          "bytes": 23504.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "differences-no-input",
    "N": 20000,
    "floor_percent": 0.1399707389556415,
    "threshold_percent": 0.1399707389556415,
    "instructions": [
      {
        "sample": "1",
        "base": 10044.90025,
        "candidate": 10035.2181,
        "delta_percent": -0.09638871227218848
      },
      {
        "sample": "2",
        "base": 10030.6528,
        "candidate": 10034.44,
        "delta_percent": 0.037756266471511424
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 13.0,
          "bytes": 704.0
        },
        "candidate-1": {
          "allocations": 13.0,
          "bytes": 704.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 13.0,
          "bytes": 704.0
        },
        "candidate-1": {
          "allocations": 13.0,
          "bytes": 704.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "take-multispec-small",
    "N": 20000,
    "floor_percent": 0.015375790671789957,
    "threshold_percent": 0.05,
    "instructions": [
      {
        "sample": "1",
        "base": 49923.0765,
        "candidate": 49921.6604,
        "delta_percent": -0.0028365639685778987
      },
      {
        "sample": "2",
        "base": 49932.16665,
        "candidate": 49922.4357,
        "delta_percent": -0.019488339186657377
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 109.0,
          "bytes": 5856.0
        },
        "candidate-1": {
          "allocations": 109.0,
          "bytes": 5856.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 109.0,
          "bytes": 5856.0
        },
        "candidate-1": {
          "allocations": 109.0,
          "bytes": 5856.0
        }
      }
    ],
    "regression": false
  }
]
```
**candidate-2 — every paired instruction and allocation row.**

```json
[
  {
    "label": "toexpression-no-head",
    "N": 20000,
    "floor_percent": 0.0018437394213033365,
    "threshold_percent": 0.05,
    "instructions": [
      {
        "sample": "1",
        "base": 6470.4316,
        "candidate": 6469.27415,
        "delta_percent": -0.017888296663237657
      },
      {
        "sample": "2",
        "base": 6470.59105,
        "candidate": 6469.4283,
        "delta_percent": -0.017969764910430452
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 10.0,
          "bytes": 438.0
        },
        "candidate-2": {
          "allocations": 10.0,
          "bytes": 438.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 10.0,
          "bytes": 438.0
        },
        "candidate-2": {
          "allocations": 10.0,
          "bytes": 438.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "toexpression-head",
    "N": 20000,
    "floor_percent": 0.3784269468738888,
    "threshold_percent": 0.3784269468738888,
    "instructions": [
      {
        "sample": "1",
        "base": 20684.00185,
        "candidate": 20603.29765,
        "delta_percent": -0.3901769134680322
      },
      {
        "sample": "2",
        "base": 20645.2649,
        "candidate": 20603.2652,
        "delta_percent": -0.20343502591723128
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 27.0,
          "bytes": 1330.0
        },
        "candidate-2": {
          "allocations": 27.0,
          "bytes": 1330.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 27.0,
          "bytes": 1330.0
        },
        "candidate-2": {
          "allocations": 27.0,
          "bytes": 1330.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "toexpression-arithmetic",
    "N": 20000,
    "floor_percent": 0.002087342888534849,
    "threshold_percent": 0.05,
    "instructions": [
      {
        "sample": "1",
        "base": 53236.29215,
        "candidate": 53235.7107,
        "delta_percent": -0.00109220604312954
      },
      {
        "sample": "2",
        "base": 53237.50295,
        "candidate": 53235.715,
        "delta_percent": -0.003358440762490922
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 123.0,
          "bytes": 5634.0
        },
        "candidate-2": {
          "allocations": 123.0,
          "bytes": 5634.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 123.0,
          "bytes": 5634.0
        },
        "candidate-2": {
          "allocations": 123.0,
          "bytes": 5634.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "dot-three-no-input",
    "N": 20000,
    "floor_percent": 0.000548071893236024,
    "threshold_percent": 0.05,
    "instructions": [
      {
        "sample": "1",
        "base": 254200.7346,
        "candidate": 254198.83535,
        "delta_percent": -0.0007471457558860806
      },
      {
        "sample": "2",
        "base": 254199.63325,
        "candidate": 254198.1015,
        "delta_percent": -0.0006025775806328681
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 596.0,
          "bytes": 23504.0
        },
        "candidate-2": {
          "allocations": 596.0,
          "bytes": 23504.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 596.0,
          "bytes": 23504.0
        },
        "candidate-2": {
          "allocations": 596.0,
          "bytes": 23504.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "differences-no-input",
    "N": 20000,
    "floor_percent": 0.1399707389556415,
    "threshold_percent": 0.1399707389556415,
    "instructions": [
      {
        "sample": "1",
        "base": 10037.7029,
        "candidate": 10033.4126,
        "delta_percent": -0.04274185082725479
      },
      {
        "sample": "2",
        "base": 10030.64875,
        "candidate": 10035.25795,
        "delta_percent": 0.04595116542187583
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 13.0,
          "bytes": 704.0
        },
        "candidate-2": {
          "allocations": 13.0,
          "bytes": 704.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 13.0,
          "bytes": 704.0
        },
        "candidate-2": {
          "allocations": 13.0,
          "bytes": 704.0
        }
      }
    ],
    "regression": false
  },
  {
    "label": "take-multispec-small",
    "N": 20000,
    "floor_percent": 0.015375790671789957,
    "threshold_percent": 0.05,
    "instructions": [
      {
        "sample": "1",
        "base": 49923.0037,
        "candidate": 49922.45385,
        "delta_percent": -0.0011013960684480573
      },
      {
        "sample": "2",
        "base": 49915.1264,
        "candidate": 49922.20675,
        "delta_percent": 0.014184778263914843
      }
    ],
    "allocations": [
      {
        "sample": "1",
        "base": {
          "allocations": 109.0,
          "bytes": 5856.0
        },
        "candidate-2": {
          "allocations": 109.0,
          "bytes": 5856.0
        }
      },
      {
        "sample": "2",
        "base": {
          "allocations": 109.0,
          "bytes": 5856.0
        },
        "candidate-2": {
          "allocations": 109.0,
          "bytes": 5856.0
        }
      }
    ],
    "regression": false
  }
]
```
Candidate 1 failed the no-head instruction gate (+0.338054% / +0.340492%). Candidate 2 removed that regression (-0.017888% / -0.017970%) and passes all six labels. Allocation slopes are unchanged in both candidates. No remeasurement was used to discard a bad sample. Two of four permitted candidates were measured; the concrete process cap was 390 (240 perf + 120 allocation + 30 correctness), and 234 ran (144 perf + 72 allocation + 18 correctness).

Every raw measurement row (raw stdout/stderr/perf CSV and exact commands remain under perf/<mode>/<build>/<sample>/):

```text
mode	arm	sample	label	count	exit	pass	instructions	allocations	bytes	peak	value	expected
alloc	base	1	toexpression-no-head	20000	0	True		201600	9239912	300622		
alloc	base	1	toexpression-no-head	40000	0	True		401600	17999912	300622		
alloc	base	1	toexpression-head	20000	0	True		541646	27084953	302373		
alloc	base	1	toexpression-head	40000	0	True		1081646	53684953	302373		
alloc	base	1	toexpression-arithmetic	20000	0	True		2461605	113163441	303989		
alloc	base	1	toexpression-arithmetic	40000	0	True		4921605	225843441	303989		
alloc	base	1	dot-three-no-input	20000	0	True		11921740	477979704	8678236		
alloc	base	1	dot-three-no-input	40000	0	True		23841740	948059704	9318236		
alloc	base	1	differences-no-input	20000	0	True		261670	21969680	9591399		
alloc	base	1	differences-no-input	40000	0	True		521670	36049680	10231399		
alloc	base	1	take-multispec-small	20000	0	True		2181802	125022546	9594023		
alloc	base	1	take-multispec-small	40000	0	True		4361802	242142546	10234023		
alloc	base	2	toexpression-no-head	20000	0	True		201600	9239912	300622		
alloc	base	2	toexpression-no-head	40000	0	True		401600	17999912	300622		
alloc	base	2	toexpression-head	20000	0	True		541646	27084953	302373		
alloc	base	2	toexpression-head	40000	0	True		1081646	53684953	302373		
alloc	base	2	toexpression-arithmetic	20000	0	True		2461605	113163441	303989		
alloc	base	2	toexpression-arithmetic	40000	0	True		4921605	225843441	303989		
alloc	base	2	dot-three-no-input	20000	0	True		11921740	477979704	8678236		
alloc	base	2	dot-three-no-input	40000	0	True		23841740	948059704	9318236		
alloc	base	2	differences-no-input	20000	0	True		261670	21969680	9591399		
alloc	base	2	differences-no-input	40000	0	True		521670	36049680	10231399		
alloc	base	2	take-multispec-small	20000	0	True		2181802	125022546	9594023		
alloc	base	2	take-multispec-small	40000	0	True		4361802	242142546	10234023		
alloc	candidate-1	1	toexpression-no-head	20000	0	True		201600	9239919	300622		
alloc	candidate-1	1	toexpression-no-head	40000	0	True		401600	17999919	300622		
alloc	candidate-1	1	toexpression-head	20000	0	True		541646	27084960	302373		
alloc	candidate-1	1	toexpression-head	40000	0	True		1081646	53684960	302373		
alloc	candidate-1	1	toexpression-arithmetic	20000	0	True		2461605	113163448	303989		
alloc	candidate-1	1	toexpression-arithmetic	40000	0	True		4921605	225843448	303989		
alloc	candidate-1	1	dot-three-no-input	20000	0	True		11921740	477979711	8678236		
alloc	candidate-1	1	dot-three-no-input	40000	0	True		23841740	948059711	9318236		
alloc	candidate-1	1	differences-no-input	20000	0	True		261670	21969687	9591399		
alloc	candidate-1	1	differences-no-input	40000	0	True		521670	36049687	10231399		
alloc	candidate-1	1	take-multispec-small	20000	0	True		2181802	125022553	9594023		
alloc	candidate-1	1	take-multispec-small	40000	0	True		4361802	242142553	10234023		
alloc	candidate-1	2	toexpression-no-head	20000	0	True		201600	9239919	300622		
alloc	candidate-1	2	toexpression-no-head	40000	0	True		401600	17999919	300622		
alloc	candidate-1	2	toexpression-head	20000	0	True		541646	27084960	302373		
alloc	candidate-1	2	toexpression-head	40000	0	True		1081646	53684960	302373		
alloc	candidate-1	2	toexpression-arithmetic	20000	0	True		2461605	113163448	303989		
alloc	candidate-1	2	toexpression-arithmetic	40000	0	True		4921605	225843448	303989		
alloc	candidate-1	2	dot-three-no-input	20000	0	True		11921740	477979711	8678236		
alloc	candidate-1	2	dot-three-no-input	40000	0	True		23841740	948059711	9318236		
alloc	candidate-1	2	differences-no-input	20000	0	True		261670	21969687	9591399		
alloc	candidate-1	2	differences-no-input	40000	0	True		521670	36049687	10231399		
alloc	candidate-1	2	take-multispec-small	20000	0	True		2181802	125022553	9594023		
alloc	candidate-1	2	take-multispec-small	40000	0	True		4361802	242142553	10234023		
alloc	candidate-2	1	toexpression-no-head	20000	0	True		201600	9239919	300622		
alloc	candidate-2	1	toexpression-no-head	40000	0	True		401600	17999919	300622		
alloc	candidate-2	1	toexpression-head	20000	0	True		541646	27084960	302373		
alloc	candidate-2	1	toexpression-head	40000	0	True		1081646	53684960	302373		
alloc	candidate-2	1	toexpression-arithmetic	20000	0	True		2461605	113163448	303989		
alloc	candidate-2	1	toexpression-arithmetic	40000	0	True		4921605	225843448	303989		
alloc	candidate-2	1	dot-three-no-input	20000	0	True		11921740	477979711	8678236		
alloc	candidate-2	1	dot-three-no-input	40000	0	True		23841740	948059711	9318236		
alloc	candidate-2	1	differences-no-input	20000	0	True		261670	21969687	9591399		
alloc	candidate-2	1	differences-no-input	40000	0	True		521670	36049687	10231399		
alloc	candidate-2	1	take-multispec-small	20000	0	True		2181802	125022553	9594023		
alloc	candidate-2	1	take-multispec-small	40000	0	True		4361802	242142553	10234023		
alloc	candidate-2	2	toexpression-no-head	20000	0	True		201600	9239919	300622		
alloc	candidate-2	2	toexpression-no-head	40000	0	True		401600	17999919	300622		
alloc	candidate-2	2	toexpression-head	20000	0	True		541646	27084960	302373		
alloc	candidate-2	2	toexpression-head	40000	0	True		1081646	53684960	302373		
alloc	candidate-2	2	toexpression-arithmetic	20000	0	True		2461605	113163448	303989		
alloc	candidate-2	2	toexpression-arithmetic	40000	0	True		4921605	225843448	303989		
alloc	candidate-2	2	dot-three-no-input	20000	0	True		11921740	477979711	8678236		
alloc	candidate-2	2	dot-three-no-input	40000	0	True		23841740	948059711	9318236		
alloc	candidate-2	2	differences-no-input	20000	0	True		261670	21969687	9591399		
alloc	candidate-2	2	differences-no-input	40000	0	True		521670	36049687	10231399		
alloc	candidate-2	2	take-multispec-small	20000	0	True		2181802	125022553	9594023		
alloc	candidate-2	2	take-multispec-small	40000	0	True		4361802	242142553	10234023		
check	base	1	toexpression-no-head	check	0	True					7	7
check	base	1	toexpression-head	check	0	True					107	107
check	base	1	toexpression-arithmetic	check	0	True					31	31
check	base	1	dot-three-no-input	check	0	True					List[List[6, 12], List[18, 24]]	List[List[6, 12], List[18, 24]]
check	base	1	differences-no-input	check	0	True					List[3, 5, 7, 9]	List[3, 5, 7, 9]
check	base	1	take-multispec-small	check	0	True					List[List[List[List[List[List[List[List[1]]]]]]]]	List[List[List[List[List[List[List[List[1]]]]]]]]
check	candidate-1	1	toexpression-no-head	check	0	True					7	7
check	candidate-1	1	toexpression-head	check	0	True					107	107
check	candidate-1	1	toexpression-arithmetic	check	0	True					31	31
check	candidate-1	1	dot-three-no-input	check	0	True					List[List[6, 12], List[18, 24]]	List[List[6, 12], List[18, 24]]
check	candidate-1	1	differences-no-input	check	0	True					List[3, 5, 7, 9]	List[3, 5, 7, 9]
check	candidate-1	1	take-multispec-small	check	0	True					List[List[List[List[List[List[List[List[1]]]]]]]]	List[List[List[List[List[List[List[List[1]]]]]]]]
check	candidate-2	1	toexpression-no-head	check	0	True					7	7
check	candidate-2	1	toexpression-head	check	0	True					107	107
check	candidate-2	1	toexpression-arithmetic	check	0	True					31	31
check	candidate-2	1	dot-three-no-input	check	0	True					List[List[6, 12], List[18, 24]]	List[List[6, 12], List[18, 24]]
check	candidate-2	1	differences-no-input	check	0	True					List[3, 5, 7, 9]	List[3, 5, 7, 9]
check	candidate-2	1	take-multispec-small	check	0	True					List[List[List[List[List[List[List[List[1]]]]]]]]	List[List[List[List[List[List[List[List[1]]]]]]]]
instructions	base	candidate-1-pair-1	toexpression-no-head	20000	0	True	132907301					
instructions	base	candidate-1-pair-1	toexpression-no-head	40000	0	True	262317275					
instructions	base	candidate-1-pair-1	toexpression-head	20000	0	True	416433295					
instructions	base	candidate-1-pair-1	toexpression-head	40000	0	True	829340068					
instructions	base	candidate-1-pair-1	toexpression-arithmetic	20000	0	True	1068245617					
instructions	base	candidate-1-pair-1	toexpression-arithmetic	40000	0	True	2132980086					
instructions	base	candidate-1-pair-1	dot-three-no-input	20000	0	True	5090697921					
instructions	base	candidate-1-pair-1	dot-three-no-input	40000	0	True	10174670468					
instructions	base	candidate-1-pair-1	differences-no-input	20000	0	True	206549448					
instructions	base	candidate-1-pair-1	differences-no-input	40000	0	True	407447453					
instructions	base	candidate-1-pair-1	take-multispec-small	20000	0	True	1003809872					
instructions	base	candidate-1-pair-1	take-multispec-small	40000	0	True	2002271402					
instructions	base	candidate-1-pair-2	toexpression-no-head	20000	0	True	132907704					
instructions	base	candidate-1-pair-2	toexpression-no-head	40000	0	True	262318727					
instructions	base	candidate-1-pair-2	toexpression-head	20000	0	True	416435025					
instructions	base	candidate-1-pair-2	toexpression-head	40000	0	True	829340366					
instructions	base	candidate-1-pair-2	toexpression-arithmetic	20000	0	True	1068244990					
instructions	base	candidate-1-pair-2	toexpression-arithmetic	40000	0	True	2132977676					
instructions	base	candidate-1-pair-2	dot-three-no-input	20000	0	True	5090694269					
instructions	base	candidate-1-pair-2	dot-three-no-input	40000	0	True	10174688968					
instructions	base	candidate-1-pair-2	differences-no-input	20000	0	True	206548399					
instructions	base	candidate-1-pair-2	differences-no-input	40000	0	True	407161455					
instructions	base	candidate-1-pair-2	take-multispec-small	20000	0	True	1003964260					
instructions	base	candidate-1-pair-2	take-multispec-small	40000	0	True	2002607593					
instructions	base	candidate-2-pair-1	toexpression-no-head	20000	0	True	132907142					
instructions	base	candidate-2-pair-1	toexpression-no-head	40000	0	True	262315774					
instructions	base	candidate-2-pair-1	toexpression-head	20000	0	True	415656153					
instructions	base	candidate-2-pair-1	toexpression-head	40000	0	True	829336190					
instructions	base	candidate-2-pair-1	toexpression-arithmetic	20000	0	True	1068250158					
instructions	base	candidate-2-pair-1	toexpression-arithmetic	40000	0	True	2132976001					
instructions	base	candidate-2-pair-1	dot-three-no-input	20000	0	True	5090688488					
instructions	base	candidate-2-pair-1	dot-three-no-input	40000	0	True	10174703180					
instructions	base	candidate-2-pair-1	differences-no-input	20000	0	True	206405833					
instructions	base	candidate-2-pair-1	differences-no-input	40000	0	True	407159891					
instructions	base	candidate-2-pair-1	take-multispec-small	20000	0	True	1003807809					
instructions	base	candidate-2-pair-1	take-multispec-small	40000	0	True	2002267883					
instructions	base	candidate-2-pair-2	toexpression-no-head	20000	0	True	132906368					
instructions	base	candidate-2-pair-2	toexpression-no-head	40000	0	True	262318189					
instructions	base	candidate-2-pair-2	toexpression-head	20000	0	True	416432994					
instructions	base	candidate-2-pair-2	toexpression-head	40000	0	True	829338292					
instructions	base	candidate-2-pair-2	toexpression-arithmetic	20000	0	True	1068243432					
instructions	base	candidate-2-pair-2	toexpression-arithmetic	40000	0	True	2132993491					
instructions	base	candidate-2-pair-2	dot-three-no-input	20000	0	True	5090699684					
instructions	base	candidate-2-pair-2	dot-three-no-input	40000	0	True	10174692349					
instructions	base	candidate-2-pair-2	differences-no-input	20000	0	True	206548147					
instructions	base	candidate-2-pair-2	differences-no-input	40000	0	True	407161122					
instructions	base	candidate-2-pair-2	take-multispec-small	20000	0	True	1003965604					
instructions	base	candidate-2-pair-2	take-multispec-small	40000	0	True	2002268132					
instructions	base	floor-1a	toexpression-no-head	20000	0	True	132910259					
instructions	base	floor-1a	toexpression-no-head	40000	0	True	262316597					
instructions	base	floor-1a	toexpression-head	20000	0	True	415652237					
instructions	base	floor-1a	toexpression-head	40000	0	True	827778142					
instructions	base	floor-1a	toexpression-arithmetic	20000	0	True	1068241275					
instructions	base	floor-1a	toexpression-arithmetic	40000	0	True	2132978417					
instructions	base	floor-1a	dot-three-no-input	20000	0	True	5090695748					
instructions	base	floor-1a	dot-three-no-input	40000	0	True	10174683740					
instructions	base	floor-1a	differences-no-input	20000	0	True	206549313					
instructions	base	floor-1a	differences-no-input	40000	0	True	407165529					
instructions	base	floor-1a	take-multispec-small	20000	0	True	1003964734					
instructions	base	floor-1a	take-multispec-small	40000	0	True	2002267867					
instructions	base	floor-1b	toexpression-no-head	20000	0	True	132908485					
instructions	base	floor-1b	toexpression-no-head	40000	0	True	262317097					
instructions	base	floor-1b	toexpression-head	20000	0	True	416436115					
instructions	base	floor-1b	toexpression-head	40000	0	True	827776522					
instructions	base	floor-1b	toexpression-arithmetic	20000	0	True	1068248474					
instructions	base	floor-1b	toexpression-arithmetic	40000	0	True	2132977337					
instructions	base	floor-1b	dot-three-no-input	20000	0	True	5090703121					
instructions	base	floor-1b	dot-three-no-input	40000	0	True	10174672954					
instructions	base	floor-1b	differences-no-input	20000	0	True	206550454					
instructions	base	floor-1b	differences-no-input	40000	0	True	407447474					
instructions	base	floor-1b	take-multispec-small	20000	0	True	1003813915					
instructions	base	floor-1b	take-multispec-small	40000	0	True	2002270545					
instructions	base	floor-2a	toexpression-no-head	20000	0	True	132908905					
instructions	base	floor-2a	toexpression-no-head	40000	0	True	262319817					
instructions	base	floor-2a	toexpression-head	20000	0	True	415655505					
instructions	base	floor-2a	toexpression-head	40000	0	True	827777848					
instructions	base	floor-2a	toexpression-arithmetic	20000	0	True	1068241667					
instructions	base	floor-2a	toexpression-arithmetic	40000	0	True	2132992462					
instructions	base	floor-2a	dot-three-no-input	20000	0	True	5090703241					
instructions	base	floor-2a	dot-three-no-input	40000	0	True	10174707793					
instructions	base	floor-2a	differences-no-input	20000	0	True	206406379					
instructions	base	floor-2a	differences-no-input	40000	0	True	407161382					
instructions	base	floor-2a	take-multispec-small	20000	0	True	1003963200					
instructions	base	floor-2a	take-multispec-small	40000	0	True	2002266805					
instructions	base	floor-2b	toexpression-no-head	20000	0	True	132908113					
instructions	base	floor-2b	toexpression-no-head	40000	0	True	262316639					
instructions	base	floor-2b	toexpression-head	20000	0	True	415656064					
instructions	base	floor-2b	toexpression-head	40000	0	True	829337989					
instructions	base	floor-2b	toexpression-arithmetic	20000	0	True	1068250961					
instructions	base	floor-2b	toexpression-arithmetic	40000	0	True	2132979531					
instructions	base	floor-2b	dot-three-no-input	20000	0	True	5090705699					
instructions	base	floor-2b	dot-three-no-input	40000	0	True	10174682387					
instructions	base	floor-2b	differences-no-input	20000	0	True	206547952					
instructions	base	floor-2b	differences-no-input	40000	0	True	407161268					
instructions	base	floor-2b	take-multispec-small	20000	0	True	1003967964					
instructions	base	floor-2b	take-multispec-small	40000	0	True	2002277312					
instructions	candidate-1	1	toexpression-no-head	20000	0	True	133349491					
instructions	candidate-1	1	toexpression-no-head	40000	0	True	263196941					
instructions	candidate-1	1	toexpression-head	20000	0	True	416034105					
instructions	candidate-1	1	toexpression-head	40000	0	True	828536726					
instructions	candidate-1	1	toexpression-arithmetic	20000	0	True	1068684258					
instructions	candidate-1	1	toexpression-arithmetic	40000	0	True	2133859333					
instructions	candidate-1	1	dot-three-no-input	20000	0	True	5090673267					
instructions	candidate-1	1	dot-three-no-input	40000	0	True	10174635834					
instructions	candidate-1	1	differences-no-input	20000	0	True	206311650					
instructions	candidate-1	1	differences-no-input	40000	0	True	407016012					
instructions	candidate-1	1	take-multispec-small	20000	0	True	1003796764					
instructions	candidate-1	1	take-multispec-small	40000	0	True	2002229972					
instructions	candidate-1	2	toexpression-no-head	20000	0	True	133347812					
instructions	candidate-1	2	toexpression-no-head	40000	0	True	263199469					
instructions	candidate-1	2	toexpression-head	20000	0	True	416036092					
instructions	candidate-1	2	toexpression-head	40000	0	True	828540339					
instructions	candidate-1	2	toexpression-arithmetic	20000	0	True	1068686528					
instructions	candidate-1	2	toexpression-arithmetic	40000	0	True	2133881237					
instructions	candidate-1	2	dot-three-no-input	20000	0	True	5090658889					
instructions	candidate-1	2	dot-three-no-input	40000	0	True	10174651790					
instructions	candidate-1	2	differences-no-input	20000	0	True	206299193					
instructions	candidate-1	2	differences-no-input	40000	0	True	406987993					
instructions	candidate-1	2	take-multispec-small	20000	0	True	1003786861					
instructions	candidate-1	2	take-multispec-small	40000	0	True	2002235575					
instructions	candidate-2	1	toexpression-no-head	20000	0	True	132890530					
instructions	candidate-2	1	toexpression-no-head	40000	0	True	262276013					
instructions	candidate-2	1	toexpression-head	20000	0	True	415595057					
instructions	candidate-2	1	toexpression-head	40000	0	True	827661010					
instructions	candidate-2	1	toexpression-arithmetic	20000	0	True	1068225691					
instructions	candidate-2	1	toexpression-arithmetic	40000	0	True	2132939905					
instructions	candidate-2	1	dot-three-no-input	20000	0	True	5090668474					
instructions	candidate-2	1	dot-three-no-input	40000	0	True	10174645181					
instructions	candidate-2	1	differences-no-input	20000	0	True	206310231					
instructions	candidate-2	1	differences-no-input	40000	0	True	406978483					
instructions	candidate-2	1	take-multispec-small	20000	0	True	1003789374					
instructions	candidate-2	1	take-multispec-small	40000	0	True	2002238451					
instructions	candidate-2	2	toexpression-no-head	20000	0	True	132888710					
instructions	candidate-2	2	toexpression-no-head	40000	0	True	262277276					
instructions	candidate-2	2	toexpression-head	20000	0	True	415592918					
instructions	candidate-2	2	toexpression-head	40000	0	True	827658222					
instructions	candidate-2	2	toexpression-arithmetic	20000	0	True	1068224276					
instructions	candidate-2	2	toexpression-arithmetic	40000	0	True	2132938576					
instructions	candidate-2	2	dot-three-no-input	20000	0	True	5090675704					
instructions	candidate-2	2	dot-three-no-input	40000	0	True	10174637734					
instructions	candidate-2	2	differences-no-input	20000	0	True	206310952					
instructions	candidate-2	2	differences-no-input	40000	0	True	407016111					
instructions	candidate-2	2	take-multispec-small	20000	0	True	1003788231					
instructions	candidate-2	2	take-multispec-small	40000	0	True	2002232366
```
### 4. Stop-loss counters and open questions

Two compiling production candidates, two measured candidates out of four. Required named reds turned green: four; additional suspending-head red also green. Sweep rows classified: 182 (15 real tails, 163 preserved suffixes, G12 repaired, three reported wrapper omissions). Regressing instruction labels: one in candidate 1 -> zero in candidate 2. Consecutive nonprogress candidates: zero at completion; the stop-loss was never reached.

Open follow-up: the marker-collision group requires localization across implicit Option decoding and the three omitted Some wrappers. Its scratch tests remain red by design and were removed from the integration-test tree. No loss outside G12 was repaired, and no pre-fix value became an expected answer. Existing owed roles (38) remain owed. Mandatory final gates were run; native ZeroMQ denied-loopback tests are explicitly disclosed as skipped.
