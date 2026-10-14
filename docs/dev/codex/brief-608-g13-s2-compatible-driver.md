# brief-608 — G13 step S2: the compatible driver, slot faults and two-Arc accounting

## Goal

Make the suspension driver run typed entries beside legacy ones. S1 added
the typed API dormant; S2 makes it live without converting any production
frame. When you are done:

- the queue holds legacy entries (the old erased `SuspensionFrame`, input
  `Value` only) and typed entries (S1's adapter), and the transport of every
  typed slot is chosen by its type's sealed mode alone;
- a typed entry decodes its input **before** the consumer is popped; a wrong
  arm or wrong state variant is a private `DriverFault` that unwinds the
  request structurally and surfaces as one named public error, never as a
  guest error a frame could catch;
- the witnessed `suspend`, `FrameChain::suspend`, `ContinueToken::suspend`
  and the temporary `resume_typed_or_suspend!` macro work, beside the old
  frame API and macro, which stay for unconverted families;
- typed entries are charged as two allocations (outer adapter `Arc`, inner
  frame `Arc`) in admission, the maintained total and the independent model;
- a test-only `FaultProbe` lets the oracle assert, unconditionally and in
  both builds, that no slot fault occurred anywhere in the ordinary corpus.

Evaluation of every existing program is unchanged: no production frame is
typed in S2, so the only live typed entries are the test frames you write.

This is step S2 of G13 (design round 5, brief-606, ratified). S1
(brief-607, `777c61cc`) is committed and gated; S3-S54 type the frames;
F1-F23 flip transport per type; R removes the legacy machinery.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- Authority, in order: this brief; brief-606 (D1/D2 and its report) with
  `.temp/brief-606/design.md`; `.temp/brief-605/design.md` §§2-5 for the
  driver, fault, unwind and accounting design, **except §5's transit record,
  which D2 rejected**: decoded input is an ordinary Rust local during the
  consumer call, as the old `Expr` input is at `suspension/core.rs:736`. Add
  no transit record, root buffer or projection.
- S1 as built: brief-607's last execution report, `suspension/typed/`, the
  census `crates/rum-stdlib/tests/typed_slot_census*`, the compile controls
  `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`.
- `.temp/brief-606/compile-controls.tsv`: the 7 rows with `first_gate = S2`.
  `.temp/brief-606/insertion-routes.tsv`: I0001, I0002, I0003, I0007, I0008,
  I0009 are S2's mechanism rows (I0004 default fallback, I0005 retention,
  I0006 public-file rollback and I0010 rule inspection belong to S5, S18,
  S14 and S24). `.temp/brief-606/transformation-dispositions.tsv`: T0010 is
  S2's transformation row. Every individual frame, macro, chain and
  deferred occurrence stays with its family step. Inputs: never edit them.

## Scope

A listed file or class is not permission to type a production frame, change
evaluation, or tidy. New files are marked.

**Classes** (a new mechanism lives here; edit what it needs, disclose each file):

- Any file under `crates/rum-runtime/src/runtime/suspension/` (driver, typed
  API, errors, output, trace), including new files.
- Any file under `crates/rum-stdlib/tests/typed_slot_census/` and
  `typed_slot_census.rs`, including new files.
- Any file under `crates/rum-stdlib/tests/suspension_oracle/support/`, for
  the unconditional fault-occurrence check only.
- **The exhaustiveness ripple.** Any file in which the compiler rejects a
  match because of the two new public variants: add the arm(s) only, mapping
  the fault to its terminal passthrough. No other edit in such a file.

**Files:**

- `crates/rum-runtime/Cargo.toml` (one empty feature, `test-continuation-slots`)
- `crates/rum-stdlib/Cargo.toml` (enable it on the existing `rum-runtime`
  dev-dependency; no new dependency, no version change)
- `crates/rum-runtime/src/runtime/accounting_mode/mod.rs`
- `crates/rum-runtime/src/runtime/accounting_mode/frozen.rs` (admission of a
  typed entry's layout only; the retention route itself is its family's)
- `crates/rum-runtime/src/runtime/debug_tools/resources.rs`
- `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence.rs`
- `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/dispatch.rs`
- `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/payloads.rs`
- `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs`
- `crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs` (fatal passthrough)
- `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs` (fatal passthrough)
- `crates/rum-runtime/src/runtime/debug_tools/mod.rs`, `debug_tools/test_report.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`, `crates/rum-runtime/src/runtime/mod.rs`
- `crates/rum-runtime/src/runtime/model/mod.rs`, `model/requests.rs`, `model/session.rs`
- `crates/rum-runtime/src/runtime/session/mod.rs`, `session/batch_file.rs`,
  `session/session_eval.rs`, `session/session_helpers.rs`,
  `session/session_requests.rs`, `session/session_state.rs`
- `crates/rum-host/src/model/mod.rs`, `crates/rum-host/src/model/outcome.rs`
- `crates/rum-cli/src/eval_tests.rs` (T0010: observe only)
- `crates/rum-runtime/src/runtime/tests/mod.rs`,
  `tests/frame_root_trace.rs`, `tests/frame_root_trace/derived_inventory.rs`
- `crates/rum-stdlib/tests/suspension_oracle.rs`,
  `suspension_oracle/mod.rs`, `suspension_oracle/descriptors.rs`,
  `suspension_oracle/fixtures/mod.rs`, `suspension_oracle/fixtures/typed_slots.rs`,
  `suspension_oracle/inventory/typed-slot-stages.tsv`
- `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`
- Maintenance inventories, by replay only: the seven
  `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv` named in
  brief-607, `retention_closure/inventory/{local-contracts,promoted}.tsv` and
  `retention_closure/local/{contracts,mod,resume_macro,rule_tests}.rs`.

`Cargo.lock`, the toolchain, docs, `AGENTS.md`, `TODO.md` and the core
`Expr`/`Symbol` representation stay unchanged. Scratch under
`.temp/brief-608/`; the certificate replay may write `.temp/brief-546/certificates/`.

**Disclosure duty.** A necessary change outside the list and the classes:
make no edit there; STOP and report the file, the line and why.

## Steps

1. **Base binaries first.** Before any edit, build the measurement bases
   (Performance section) and hash them.
2. **Entries and transport.** Give the queue a typed entry beside the legacy
   one. Legacy input is `Value` only. A typed entry's input and output go
   through its type's sealed mode; no per-frame choice, no runtime switch, no
   dual-arm decode. A typed `Expr` or Legacy-mode result and a neighbouring
   legacy entry exchange the unchanged `Expr` encoding. Input seeding stays
   `Value(Expr::string(answer))`; request completion requires `Value`.
3. **Routes (S2's mechanism rows).** I0001 the macro: borrow the actual
   producer `Result` for the witness, match it once, construct frames only in
   the `InputRequired` arm, return the typed suspension result; no ordinary-
   path `Vec`, `Arc` or frame. I0002 chains: witness the first input, checked
   adjacency, one return of the last output. I0003 repeat helpers: the
   adapter-minted token accepts a fresh state of the same frame type and
   returns `ResumeStop<F::Output>`. I0007 `Evaluate`: next input `Expr`, next
   output the parent's output. I0008 finalizers: `SlotFinalizer::Output` is
   the parent's output; both owning Arcs are released before `finish`.
   I0009 seed/splice/completion: nested pending queues precede the active
   unrun suffix, order preserved; the request boundary requires the correct
   variant. Implement each as the general mechanism; prove each with test
   frames in `suspension/typed/tests/`, not by converting a production family.
4. **Faults.** Exact-decode before popping. S1's `ErasedFrame::resume`
   consumes `Arc<Self>` and decodes inside it, so the queue must still own
   the consumer when a decode fails: either split decoding from resuming or
   resume a clone of the entry's `Arc` (never a clone of frame state); say
   which and why. On mismatch: the consumer is not run and stays at the
   active front; latch the first fault at the request
   boundary; move the resuming owner (front consumer and suffix) and every
   nested pending owner, in normal splice order, into the existing accounted
   abandonment worklist (`suspension/core.rs:634-719`, not
   `finish_completed_resumption`); clear pending request handles; restore
   the guest-form environment; run output rollback and each frame's
   structural abandon. `WithCleanup`'s approved `EvaluateCleanup` stays the
   only guest cleanup transition; if that cleanup suspends, transfer and
   abandon its nested owner before the saved suffix. A cleanup failure never
   replaces the first fault. Only after the unwind does the boundary project
   the fault to `EvalError::ContinuationSlotMismatch { expected: &'static str,
   got: &'static str }` / the matching `SessionRequestError` case; neither
   stores guest payload. The fault never re-enters the frame-completion loop.
   Add explicit fatal passthrough at `EvaluationData` and `TestReport`'s
   capture (defence for a direct internal return of the public variant).
5. **FaultProbe.** Behind `test-continuation-slots`: opaque `FaultProbe`,
   `Session::continuation_fault_probe(&self) -> FaultProbe`,
   `FaultProbe::occurrences(&self) -> u64`, incremented only where the fault
   is created, preserved through reset and snapshot rollback. Injection
   constructors are `cfg(test)`-private to the runtime. Production builds
   carry no counter, branch or load from it. The oracle checks the delta
   **unconditionally** — outside every `Checks` flag, before lifecycle reset
   — in both builds.
6. **Fault injection tests** (runtime unit tests; `rum-runtime` has no
   default features, so this is one build): wrong
   `Value`/`State` and wrong exact `State`, at frame input and at request
   completion, under `EvaluationData`, `VerificationTest`/`TestReport`,
   `Catch`/`Check`/`Quiet`, `WithCleanup` (including a cleanup that itself
   asks for input), `Block`/iterator/option scopes, `Trace` and output
   capture, and a memory-file rollback. Each asserts: first-fault identity,
   exactly one occurrence, the consumer did not run, every suffix owner
   abandoned, output visibility and rollback, restored scope/loader state,
   no outstanding input, and a fresh `1+1` request returns `2`.
7. **Two-Arc accounting.** Typed entries charge outer Arc header +
   `Adapter<F>` inline bytes **plus** inner Arc header + `F` inline bytes;
   trace `F`'s payloads once under the existing shared-identity rules; legacy
   entries keep their one-allocation layout; queue capacity uses the real
   entry size; retention's projected admission charges the typed layout and
   any queue growth before insertion. Sites: `core.rs:191,519`,
   `trace.rs:110`, `resources/occurrence/suspension.rs:24`. The independent
   model enumerates both owners itself and does not call the maintained
   helper. Tests: just below / at / above a memory boundary; nested
   `Evaluate` and finalizer paths; cleanup that requests input; reset and a
   fresh request; public-file rollback.
8. **Dead-code allowance.** S1's module-wide `#![allow(dead_code)]` in
   `suspension/typed.rs` goes. Whatever is still dormant after S2 (state
   providers awaiting their flips, for instance) carries an item-level allow
   with a comment naming the step that makes it live.
9. **Error size.** Print `size_of::<EvalError>()` and
   `size_of::<Result<Expr, EvalError>>()` on base and candidate. They must
   not grow; if the new variant would grow them, stop and report rather than
   boxing silently.
10. **Controls, census, inventories.** Implement S2's 7 compile-control pairs
   in `typed_slot_compile_controls.rs` with the same discipline as S1's
   (positive first, one invalidity, checked code/category and primary span,
   runtime-private fixtures in scratch copies) and rerun all 22 of S1's. The
   census still reports 607/589 production + 16/11 test-only, all untyped,
   and its driver-route receipts for I0001-I0009. Replay fingerprints, both
   certificate sets and ownership discovery; disclose every changed row. The
   ownership ledger's old rows 56-58 stay until R. A new
   `UnknownHostPayload` without an approved disposition is a STOP.

## Verification (MANDATORY — run to green or evidence)

Serial, each through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-608/tmp …`. Bounds: workspace 3 600 s; oracle
1 500 s per build; the others 900 s.

```
cargo fmt --check
cargo test -p rum-runtime --lib suspension::typed::tests
cargo test -p rum-stdlib --test typed_slot_compile_controls
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls
cargo test -p rum-stdlib --test typed_slot_census
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
scripts/check-suspension-oracle.sh 1500
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report every `test result:` total, the oracle's per-build fault-occurrence
delta, and native ZeroMQ skips as skips.

**Performance.** S2 changes the driver, the queue entry and an `EvalError`
match surface the evaluator uses, so it is measured; no exemption.
Bases: `rum-cli` and `rum-cli-alloc` built **before your first edit** with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), each in its own target
directory under `.temp/brief-608/`; candidates the same, from the same source
path. The release builds never enable `test-continuation-slots`; record the
feature tree. Panel, each result checked before it is measured: the
ordinary labels `literal-integer`, `downvalue-pattern-hit`,
`table-iterator-plus`, `matcher-single-wide-small`, `replaceall-simple`,
`module-local`, `block-local`, `take-multispec-small`, and the controls
`.temp/brief-563/perf/` names. **Noise floor first** (base against itself, two
pairs); a label regresses when both candidate samples exceed
`max(+0.05%, floor)`. Allocation counts and bytes first; then two paired
samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label
per process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw output saved
before parsing. Then the full `scripts/core-language-microbench.sh` panel on
both one-unit binaries, profiling off, as an indication only. A regression is
a defect to fix or report.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies; pitfall 8 governs the measurement. Rum against rum
only. No default-path cost: no counter, `Mutex`, allocation or branch on any
evaluation path for the probe. No owned-tree clones. The witness comes from
the actual `Result`; the token carries no `Arc`, is not `Clone`, and cannot
escape. The codec, mode and actions stay sealed and opaque; nothing public
names a mode or constructs an erased entry. New modules prefer under 10 k
characters; check `.temp/brief-606/file-size-assessments.tsv` before adding to
a large file. `suspension/core.rs` is 28.5 k, in the split-plan range: put
the typed entry, the fault unwind and the probe in new modules under
`suspension/`, keep `core.rs` under 35 k, and report a split assessment for
it (begin the split, or state the boundary and why not now).

## Checkpointing — protect the scene

Everything under `.temp/brief-608/`; `PROGRESS.md` after each step with the
verified state, compile/test counters and the exact next action.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  except the certificate replay output named above.
- Modify only the Scope. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- Commands: scoped reads and edits, `rg`/`grep`, bounded `python3`, the
  bounded Cargo commands above and bounded compiler fixtures,
  `scripts/check-suspension-oracle.sh`, `scripts/core-language-microbench.sh`
  and the measurement commands named above. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No dual-arm decoder, per-frame mode override, transit record or public raw
  adapter. No production frame typed. Do not weaken an oracle `Checks` flag,
  a retention reject or a documented test to pass.
- Stop-loss keyed on progress: compile errors you are reducing are progress;
  stop after two attempts that move none of the named failing gates.
- Paste real output; say plainly what did not run or did not pass. Leave the
  tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with the semantic change; every exhaustiveness-ripple
   file with its added arm.
2. The driver as built: entries, routes I0001-I0009, fault path, probe.
3. Fault-injection matrix: each case and its assertions.
4. Accounting: the two-Arc charge in each model and the boundary tests.
5. Compile controls (7 new + 22 rerun) and the census output.
6. All gates verbatim; error sizes; allocation, instruction and timing rows;
   inventory and certificate transitions.
7. Stop-loss counters, deviations and open questions.


## Execution report — STOP before implementation

The mandatory suspend-and-resume performance panel cannot be executed through
this CLI with queued input and stdin `/dev/null`. The untouched CLI cancels
an input suspension. Prequeued runtime input returns synchronously before
creating a suspension. Implementing or measuring different behavior would
require a scope/measurement decision.

This report follows this brief's Disclosure duty: “make no edit there; STOP
and report the file, the line and why.” No implementation change was made.

### 1. Files changed

Only this brief receives this appended report. No crate source, manifest,
lockfile, toolchain, maintenance inventory or other documentation was changed.
All scratch evidence is under `.temp/brief-608/`; the release binaries are in
`base/` and their build directories are `target-base/` and `target-base-alloc/`.
There are no exhaustiveness-ripple edits.

The necessary CLI behavior change would be at
`crates/rum-cli/src/main.rs:291`: its `InputRequired` arm calls
`retire_input_request` at line 294 and returns an error at line 299. A measured
suspend/resume loop would instead need an answer source and a call to the host
resume API. That file is outside Scope; this is not a compiler-required match
arm for either new public error variant. The permitted `eval_tests.rs` change
is observe-only and cannot supply that behavior.

Supporting current-source evidence:

- `crates/rum-cli/src/input_retirement.rs:21` cancels the request;
  `session.eval("")` at line 24 retires it through the cancellation route.
- `crates/rum-cli/src/main.rs:308` queues stdin for the setup call at line 309,
  which consumes it into `$ScriptInput`.
- `crates/rum-cli/src/main.rs:320` queues script arguments for the setup call
  at line 327, which consumes them into `$ScriptArgs`.
- `crates/rum-runtime/src/runtime/builtins_input.rs:232` takes the queued-input
  branch, returning `Ok(Expr::string(value))` at line 243. Only the empty-queue
  branch calls `begin_input_suspension` at line 246.

### 2. Driver, routes and fault path

S2 implementation was not started. The compatible queue, routes I0001/I0002/
I0003/I0007/I0008/I0009, fault unwind, request latch and FaultProbe were not
added. No production frame was typed. Existing S1 code remains unchanged.
No transit record, mode override, dual-arm decoder or raw public adapter was
introduced. I0004/I0005/I0006/I0010 remain with their assigned family steps.

### 3. Fault-injection matrix

Not run: no fault injection or S2 test frames were added. No fault-occurrence,
consumer-not-run, suffix-abandonment, scope-restoration, rollback or fresh-
request assertion is claimed. The three CLI controls below establish only
the measurement blocker; they are not substitutes for the required matrix.

### 4. Accounting

No accounting change. Two-Arc admission, maintained totals, independent
occurrence enumeration and memory-boundary tests were not implemented or run.

### 5. Compile controls and census

The seven S2 compiler pairs were not added. The 22 S1 pairs and both census
builds were not rerun. There is no newly earned census or route receipt.

### 6. Commands, output and measurement evidence

Both requested baseline builds completed before any source edit, serially,
from `/home/remi/dev/rum`, with separate target directories. Commands:

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-608/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-608/target-base cargo build --release -p rum-cli
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-608/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-608/target-base-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
```

Verbatim final build output (each exit 0):

```text
    Finished `release` profile [optimized] target(s) in 1m 41s
    Finished `release` profile [optimized] target(s) in 1m 37s
```

Full logs: `.temp/brief-608/base/build-default.log` and `build-alloc.log`.
The build durations above are provenance, not benchmark measurements.

| Binary | Bytes | SHA-256 |
| --- | ---: | --- |
| `.temp/brief-608/base/rum-cli` | 12376160 | `76efb7c611f510a4b46c8d181d77ec37c5fe7ef057c2572ce05978e7a795ecf5` |
| `.temp/brief-608/base/rum-cli-alloc` | 12390248 | `0e9c806bab777893c83e4e7cf1411d977ac63bb77c64374af0164e40a51e56a4` |

Manifest: `.temp/brief-608/base/binary-sha256.json`.
Both normal-dependency feature trees were recorded using the corresponding
build environment and target directory, with `cargo tree -p rum-cli -e
features,no-dev` (allocation build additionally uses `--features
rum-cli/alloc-count`). Both commands exited 0. Outputs:
`base/features-default.txt` and `base/features-alloc.txt`; neither contains
`test-continuation-slots`.

A bounded scratch Rust compiler fixture linked the actual untouched base
`rum-runtime` and `rum-core` rlibs. Compile and execution both exited 0.
Exact compiler command: `base/error-sizes.command`; source:
`base/error-sizes.rs`; compiler output: `base/error-sizes-compile.log`.
Verbatim output from `base/error-sizes.txt`:

```text
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136

```

No candidate exists, so candidate error sizes and a no-growth verdict are
not available.

The three feasibility controls used the untouched default release CLI, with
profiling variables removed and stdin `/dev/null`. Each child was bounded to
30 seconds under the 900-second serial wrapper and the required Cargo/test/
TMPDIR environment. The control-run cap was three; all three ran once.
Per-case `.command`, `.stdout`, `.stderr` and combined `cli-controls.json`
are in `.temp/brief-608/base/`. These were result checks, with no allocation,
instruction or timing sampling.

**script-args-control** — exact saved command:

```sh
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 30 -- /home/remi/dev/rum/.temp/brief-608/target-base/release/rum-cli --eval '$ScriptArgs' -- answer < /dev/null
```

Exit: `0`. stdout:

```text
List["answer"]
```

stderr:

```text
```

**input-with-script-arg** — exact saved command:

```sh
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 30 -- /home/remi/dev/rum/.temp/brief-608/target-base/release/rum-cli --eval 'InputString["608: "]' -- answer < /dev/null
```

Exit: `1`. stdout:

```text
```

stderr:

```text
! input required: 608: 
error: input required: 608: 
```

**cleanup-on-input** — exact saved command:

```sh
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 30 -- /home/remi/dev/rum/.temp/brief-608/target-base/release/rum-cli --eval 'WithCleanup[InputString["608: "], Print["cleanup"]]; Print["resumed"]' -- answer < /dev/null
```

Exit: `1`. stdout:

```text
cleanup
```

stderr:

```text
! input required: 608: 
error: input required: 608: 
```

The setup control returns `List["answer"]`. Supplying that same argument to
an actual `InputString` request still exits 1. `WithCleanup` prints `cleanup`
and never prints `resumed`, consistent with the source cancellation path.
All three exact-output checks passed. They do not exercise successful driver
resumption and cannot be reported as the five required route workloads.

Mandatory gates not run after the scope STOP:

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | Not run |
| `cargo test -p rum-runtime --lib suspension::typed::tests` | Not run |
| `cargo test -p rum-stdlib --test typed_slot_compile_controls` | Not run |
| `cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls` | Not run |
| `cargo test -p rum-stdlib --test typed_slot_census` | Not run |
| `cargo test -p rum-stdlib --no-default-features --test typed_slot_census` | Not run |
| `scripts/check-suspension-oracle.sh 1500` | Not run |
| `cargo test -p rum-stdlib --lib ownership_graph` | Not run |
| `cargo test -p rum-stdlib --lib local_production_acceptance` | Not run |
| `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance` | Not run |
| `cargo test --workspace --no-fail-fast` | Not run |

There are no `test result:` totals for this execution. The oracle's per-build
fault-occurrence deltas are **not measured**, not zero. Native ZeroMQ tests
were not run; no passes or skips are claimed.

The five route workloads, ordinary-label panel, brief-563 controls, base/base
noise-floor pairs, allocation counts/bytes, instruction slopes and full
microbench A/B were not measured. No performance gate passed, and no
regression/no-regression verdict is claimed.

Inventory and certificate transitions: none. No source edits expired their
fingerprints, and no replay was performed. The 1,461 files recorded in
`base/source-sha256.json` were checked byte-identical before this report.

### 7. Stop-loss, deviations and decision needed

- Scope STOP: `main.rs:291` needs different behavior to meet the specified CLI
  suspend/resume panel; that change is not authorized.
- Implementation attempts: 0. Compile failures: 0. Consecutive attempts moving
  no failing gate: 0. The progress-based stop-loss was not reached.
- Baseline builds: 2/2 succeeded. Feature-tree commands: 2/2 succeeded.
  Base size fixture: compile/run succeeded. CLI feasibility controls: 3/3
  exact expected outputs confirmed; two intentional CLI exits were 1.
- The required implementation and gates remain unfinished. No alternative
  measurement harness was substituted, and synchronous queued-input results
  were not labelled suspension/resumption evidence.
- Director decision needed: authorize a release Rust host harness that drives
  actual `InputRequired`/`resume_with_input` transitions for the five routes,
  or revise Scope to provide such a driver in the CLI. Merely prequeuing
  answers does not exercise suspension. The preserved binaries are available
  for the resumed A/B work.
- No git command, network access, unsafe addition, dependency change, lockfile
  change, production frame conversion or out-of-scope source edit occurred.
  The tree is left uncommitted.

Checkpoint: `.temp/brief-608/PROGRESS.md`. The brief is not completed.

## Director amendment — redispatch 1

The stop was correct and the panel was the director's error: `rum-cli`
retires every input suspension (`main.rs:291-299`) and pre-queued input
returns before a suspension exists (`builtins_input.rs:232-246`), so the CLI
cannot drive a resume. The CLI does not change. The panel is corrected above
(the ordinary panel is unchanged and remains the primary gate), and the
suspension path gets its own instrument:

1. **A resume harness in scratch.** Under `.temp/brief-608/resume-harness/`,
   a standalone Cargo package (its own manifest and a copy of the workspace
   lockfile, offline, not a workspace member, no repository file changed)
   depending on `rum-host` by path. It drives real cycles through the public
   `HostSession`: evaluate, receive `InputRequired`, `resume_with_input`,
   repeat, and checks the final result. It may include
   `crates/rum-cli/src/alloc_count.rs` by `#[path]` for an allocation-count
   build. Build it release with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`
   **against the current tree before your first source edit** (base, default
   and allocation-count, hashed), and again after (candidate).
2. **What it measures.** No production frame is typed in S2, so the path that
   changes for real programs is the legacy resume through the new queue.
   Five workloads, each suspending N and 2N times: an `InputString` in a
   `Table` body, in a `Do` body (the iterator scope chain), in call-argument
   preparation (`f[InputString[...], InputString[...], ...]`, the repeat
   helper), inside `WithCleanup` (cleanup path), and in a `Fold` step.
   Per-resume slopes `(count(2N) - count(N)) / N` for allocations, bytes and
   `cpu_core/instructions/u` (`taskset -c 2`, stdin `/dev/null`), base
   against itself first for the floor.
3. **The gate.** Allocations and bytes per resume must not change for legacy
   entries; they keep their one-allocation layout. Instructions per resume:
   report the delta for each workload. Any increase above the floor is
   attributed to a named mechanism (the entry match, the mode-free legacy
   path) with its per-resume size; an unexplained one is a defect. Keep the
   harness: later family steps reuse it for their typed routes.

Everything else in the brief stands. Your base `rum-cli` binaries are still
valid (no source has changed). Append a new `## Execution report` after this
section.


## Execution report — redispatch 1: STOP at the ownership witness count

S2 implementation and the focused typed tests, compiler controls and census are in place. The mandatory ownership gate is red because an S1 test still expects three erased-adapter storage sites. S2 adds the required fourth site, `Entry::Typed`.

The necessary next edit is `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs:256`: reconcile `assert_eq!(rows.len(), 3)` with the four discovered sites while retaining the full computed-witness loop. The sites are `Action::Evaluate.next`, `Action::Finalize.0`, `FrameChain.frames`, and `Entry::Typed.0`. The separate complete-ledger test already re-derived and accepted their attribution.

That source test is outside Scope. Scope allows the seven ownership TSV inventories, but not this test file; a runtime count assertion is not the compiler exhaustiveness ripple. This invokes the brief's Disclosure duty: “make no edit there; STOP and report the file, the line and why.” No edit was made to that file. No further implementation or verification gate was run after this stop. The brief is **not complete**.

### 1. Files changed

All repository implementation changes are listed individually below. This brief additionally receives only this appended report. Scratch is under `.temp/brief-608/`; the original source bytes and hashes are retained in `redispatch-1/entry-files/` and `entry-source.json`.

| File | Change |
| --- | --- |
| [crates/rum-host/src/model/outcome.rs](/home/remi/dev/rum/crates/rum-host/src/model/outcome.rs) | Map ContinuationSlotMismatch to the existing terminal HostFailureCode::Other outcome. |
| [crates/rum-runtime/Cargo.toml](/home/remi/dev/rum/crates/rum-runtime/Cargo.toml) | Add the empty test-continuation-slots feature. |
| [crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs) | Explicitly propagate the public fault variant from capture rendering. |
| [crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs) | Independently enumerate legacy or mixed queue cells and both typed Arc allocations; visit frame payloads once. |
| [crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs) | Explicit fatal passthrough in capture completion and outcome rendering. |
| [crates/rum-runtime/src/runtime/kernel_api.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_api.rs) | Export the typed macro and feature-gated opaque FaultProbe. |
| [crates/rum-runtime/src/runtime/mod.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/mod.rs) | Expose FaultProbe at the runtime facade behind its test feature. |
| [crates/rum-runtime/src/runtime/model/requests.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/model/requests.rs) | Add SessionRequestError::ContinuationSlotMismatch and its Display arm. |
| [crates/rum-runtime/src/runtime/session/session_eval.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_eval.rs) | Project the latched fault at text/inspection resume boundaries before legacy value decoding. |
| [crates/rum-runtime/src/runtime/session/session_requests.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_requests.rs) | Project the latched fault at request boundaries and map the new EvalError variant to SessionRequestError. |
| [crates/rum-runtime/src/runtime/session/session_state.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_state.rs) | Add EvalError::ContinuationSlotMismatch with two static strings. |
| [crates/rum-runtime/src/runtime/suspension/core.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core.rs) | Integrate compatible queue owners, typed insertion/layout projection and the typed/legacy driver; extract structural abandonment. |
| [crates/rum-runtime/src/runtime/suspension/core/abandon.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core/abandon.rs) | Move the existing abandonment worklist protocol into its own module and share its drain with faults. |
| [crates/rum-runtime/src/runtime/suspension/core/fault.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core/fault.rs) | New private first-fault latch, structural owner unwind and feature/test-only monotonic observer. |
| [crates/rum-runtime/src/runtime/suspension/core/queue.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core/queue.rs) | New legacy-buffer-preserving queue; widen to real Entry cells only when needed. |
| [crates/rum-runtime/src/runtime/suspension/error.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/error.rs) | Compiler-required exhaustive arm: reject impossible serialization of a structural fault onto the legacy guest-error wire. |
| [crates/rum-runtime/src/runtime/suspension/output.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/output.rs) | Collect capture boundaries from either queue representation without cloning frame state. |
| [crates/rum-runtime/src/runtime/suspension/trace.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/trace.rs) | Provide the feature-gated probe re-export through the existing suspension facade. |
| [crates/rum-runtime/src/runtime/suspension/typed.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed.rs) | Remove the module-wide dead-code allowance and connect private driver exports. |
| [crates/rum-runtime/src/runtime/suspension/typed/action.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/action.rs) | Make sealed actions reachable by the driver; add only a cfg(test)-private malformed-slot constructor. |
| [crates/rum-runtime/src/runtime/suspension/typed/adapter.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/adapter.rs) | Expose the inner borrowed layout to both accounting models; preserve exact decode and adapter-only token minting. |
| [crates/rum-runtime/src/runtime/suspension/typed/chain.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/chain.rs) | Implement witnessed chain installation and return the last output type. |
| [crates/rum-runtime/src/runtime/suspension/typed/channel.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/channel.rs) | Centralize exact completion decoding and static fault kind names; no dual-arm decode. |
| [crates/rum-runtime/src/runtime/suspension/typed/entry.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/entry.rs) | New private legacy/typed entry, dispatch action and borrowed metadata/accounting view. |
| [crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs) | Accept fresh state of the same nominal frame and return ResumeStop<F::Output>. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests.rs) | Connect live route, driver-fault, file and accounting tests. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs) | Verify two-Arc layout, real queue cells, maintained/independent totals and below/at/above admission. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs) | Fault matrix across captures/scopes, cleanup, completion, rollback/reset and direct public-variant capture defence. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/fault_support.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/fault_support.rs) | Runtime-private wrong-mode/exact-state fixtures, non-running consumer/suffix spies and common restoration assertions. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs) | Nested memory-file and public-batch rollback under all three injected mismatch kinds. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs) | Test macro/chain, legacy neighbors, repeat helper, nested Evaluate and both-Arc finalizer release. |
| [crates/rum-runtime/src/runtime/suspension/typed/witness.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/witness.rs) | Implement witnessed suspension and the cold actual-Result typed macro. |
| [crates/rum-stdlib/Cargo.toml](/home/remi/dev/rum/crates/rum-stdlib/Cargo.toml) | Enable that feature only on the existing runtime dev-dependency. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv) | Replay actual discovery rows/locations; preserve existing dispositions and the approved S2 queue/insertion transitions. Full row diff is retained. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv) | Replay actual discovery rows/locations; preserve existing dispositions and the approved S2 queue/insertion transitions. Full row diff is retained. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | Replay actual discovery rows/locations; preserve existing dispositions and the approved S2 queue/insertion transitions. Full row diff is retained. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | Replay actual discovery rows/locations; preserve existing dispositions and the approved S2 queue/insertion transitions. Full row diff is retained. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | Replay actual discovery rows/locations; preserve existing dispositions and the approved S2 queue/insertion transitions. Full row diff is retained. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv) | Replay actual discovery rows/locations; preserve existing dispositions and the approved S2 queue/insertion transitions. Full row diff is retained. |
| [crates/rum-stdlib/tests/suspension_oracle.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle.rs) | Add the serial last aggregate fault-occurrence assertion. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs) | Select the S2 typing receipt while continuing to refuse preflip. |
| [crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv) | Record S2 mechanism routes, unchanged observed T0010, and four discovered test-support legacy uses X0036-X0039. |
| [crates/rum-stdlib/tests/suspension_oracle/support/drive.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/drive.rs) | Unconditional fault observer around every shared drive, independent of Checks. |
| [crates/rum-stdlib/tests/suspension_oracle/support/faults.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/faults.rs) | New unconditional guards and retained probe observations for the per-build aggregate assertion. |
| [crates/rum-stdlib/tests/suspension_oracle/support/lifecycle.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/lifecycle.rs) | Check before session replacement/reset and retain the old probe through lifecycle completion. |
| [crates/rum-stdlib/tests/suspension_oracle/support/mod.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/mod.rs) | Wire the fault-observer support module. |
| [crates/rum-stdlib/tests/suspension_oracle/support/session.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/session.rs) | Observe initialization, retirement, fresh requests and input-queue controls. |
| [crates/rum-stdlib/tests/typed_slot_census.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census.rs) | Connect route receipts, S2 preflip refusal and scratch-copy falsifiers under brief-608. |
| [crates/rum-stdlib/tests/typed_slot_census/classify.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census/classify.rs) | Keep codec population metadata so private injection carriers are reported separately from production registry entries. |
| [crates/rum-stdlib/tests/typed_slot_census/receipts.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census/receipts.rs) | Preserve exact production/test-only counts and production mode checks; identify output as S2. |
| [crates/rum-stdlib/tests/typed_slot_census/reporting.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census/reporting.rs) | Write under brief-608 and include codec population in discovery output. |
| [crates/rum-stdlib/tests/typed_slot_census/routes.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census/routes.rs) | New live source/witness checks for six S2 routes, unchanged family ownership of four remaining routes and T0010 observation. |
| [crates/rum-stdlib/tests/typed_slot_compile_controls.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_compile_controls.rs) | Add the seven S2 positive/negative pairs while retaining all 22 S1 pairs and exact primary diagnostic spans. |

The compiler reported the new exhaustive-match requirement at `suspension/error.rs:270` before formatting. Its added arm is an `unreachable!` invariant guard: a structural driver fault is never encoded as a guest error. It does not synthesize guest payload. The explicit request/error Display and host outcome arms are in separately listed Scope files; the host arm maps the named request fault to the existing terminal `Other` failure code. No source file outside Scope was edited for a ripple.

The split has begun: `core/queue.rs`, `typed/entry.rs`, `core/fault.rs` and `core/abandon.rs` now own storage, erased dispatch, fault control and abandonment respectively. `core.rs` retains owner lifecycle, insertion and resumption. It is below 35k. Current character counts:

- `crates/rum-runtime/src/runtime/suspension/core.rs`: 27,551 characters.
- `crates/rum-runtime/src/runtime/suspension/core/queue.rs`: 4,052 characters.
- `crates/rum-runtime/src/runtime/suspension/typed/entry.rs`: 3,323 characters.
- `crates/rum-runtime/src/runtime/suspension/core/fault.rs`: 3,395 characters.
- `crates/rum-runtime/src/runtime/suspension/core/abandon.rs`: 4,057 characters.
- `crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs`: 11,603 characters.

The new live-test file is in the 10–20k warning range (11.6k); its single responsibility is execution of the six route mechanisms. The existing compile-control file is 19.7k, also in the warning range. Existing larger request/error/host files received only their scoped boundary or variant changes. No new module exceeds 20k. The only new dormant-helper allow is item-level on typed admission, with S18 named as its future retention caller; the typed module-wide allow is gone.

### 2. Driver, routes and probe

- Legacy-only owners keep `VecDeque<Arc<dyn SuspensionFrame>>`; a typed insertion widens that owner to a queue of private `Entry` values. This storage choice does not select transport. Legacy entries accept `Value` only; typed codecs use their type's sealed associated mode exclusively.
- I0001 borrows the actual producer Result, matches once and constructs the frame only in the InputRequired arm. I0002 binds the first input and every adjacency and returns the last output. I0003 uses the adapter-minted non-cloneable token, fresh state of the same nominal frame and `ResumeStop<F::Output>`.
- I0007 requires an Expr-input successor with the parent's output. I0008 drops both owning Arcs before the finalizer. I0009 seeds a Value string, splices nested owners before the unrun suffix and exactly decodes completion. I0004/I0005/I0006/I0010 remain assigned to S5/S18/S14/S24. T0010's existing CLI Result mapping is observed unchanged.
- The driver resumes a cloned entry Arc while the queue retains the consumer. It never clones frame state. Exact decode failure therefore leaves the consumer at the active front. The first private DriverFault is latched, resuming and nested pending owners enter the existing accounted abandonment worklist, handles are cleared, capture rollback and structural abandon run, and guest scopes/loaders are restored. Approved WithCleanup evaluation remains the only guest cleanup transition. Nested cleanup suspensions are abandoned before the saved suffix. Cleanup errors cannot replace the first fault.
- Only after structural unwind does the boundary return the named public variant containing static expected/got strings. It never enters the frame-completion loop. EvaluationData and TestReport also explicitly propagate a direct internal return of that public variant.
- The opaque monotonic FaultProbe is compiled only for runtime tests or `test-continuation-slots`; only private fault creation increments it. Existing handles survive rollback and Session replacement. Production feature-disabled builds contain no probe counter/load/branch. The oracle now checks independently of Checks, explicitly before lifecycle reset, and retains observations for a final per-build zero-delta assertion. **The oracle has not yet run, so no measured zero delta is claimed.**

### 3. Fault-injection matrix

All injection constructors/carriers live in runtime unit tests. No production frame is typed and no production codec is flipped. The input matrix runs all three mismatch kinds: Value for State, State for Value, and wrong exact State variant.

| Cases | Observed assertions |
| --- | --- |
| EvaluationData; TestReport/TestCreate; Catch; Check; Quiet; Trace; Block; iterator scope; OptionsPattern scope — 9 contexts × 3 kinds | Exact expected/got identity; one occurrence; consumer and both sentinel suffix frames never resume and all abandon in order; expected print/message visibility and declared trace rollback; scopes/loader/handles empty; unchanged outer variables, fallback and suffix markers; fresh 1+1 = 2. |
| WithCleanup with Abort, cleanup InputString, or a different direct public fault — 3 cases | First fault remains unchanged with exactly one occurrence; separately created nested owner abandons before active consumer/suffix; cleanup runs once; cleanup's post-input suffix does not run; scope restoration and fresh request. |
| Completion — 3 mismatch kinds | Exact completion decode; first named fault; one occurrence; empty owner/worklist/handles; snapshot rollback preserves the probe count; old probe survives Session replacement; fresh requests succeed. Expr is the ordinary boundary; the private generic boundary exercises exact State completion. |
| Nested memory files, through Get and the public batch owner — 3 kinds × 2 routes | Both loader levels close and roll back, stored definitions and Block scope restore, cleanup output remains visible, consumer/suffix sentinels abandon, one occurrence survives loader rollback, no outstanding input, fresh request succeeds. |
| Direct public fault under EvaluationData and TestReport — 2 cases | Fatal passthrough, no guest failure value, no pending owner, fresh request; occurrence count remains zero because no private decode fault was created. |

The repository has no `VerificationTest` builtin or alias; the test-case capture is exercised through rum's actual `TestCreate`/`TestReport` surface. No alias or compatibility behavior was added. Completion and direct-return cases have no rejected queued consumer/suffix where inapplicable; that is not reported as a consumer assertion.

### 4. Accounting

The maintained typed charge is outer Arc header + adapter inline bytes + inner Arc header + F inline bytes. Legacy entries retain one Arc allocation. Queue bytes use the actual legacy or mixed cell size. The independent occurrence model spells out both allocations itself and does not call the maintained helper; both models visit F's payload once. The existing root-delegation control verifies that an adapter emits its retained root only once.

Typed projected admission is available internally for retention's later S18 route without converting that route now. Below/at/above tests call the projection before insertion and verify rejection/success, unchanged pre-insertion capacity, the installed bound and recovery. A real InputString already creates a legacy queue; widening its four cells plus a zero-sized typed frame increases the observed total by 72 bytes (40 bytes for the two Arc layouts plus 32 bytes for wider cells), exactly as both models predict.

Armed maintained/independent reconciliation also runs through nested Evaluate, finalization, fault captures/scopes, cleanup input and nested/public file rollback. Probe reset/rollback and fresh requests are exercised. Allocation-counter performance equality for legacy resumes is **not yet measured on a candidate**.

### 5. Compile controls and census

Both feature configurations passed all 29 pairs (22 retained S1 + 7 new S2). Every positive group compiled first; each negative changes one invalidity and checks its diagnostic code/category and exact primary file/line/column. The seven S2 results are:

| Pair | Required diagnostic |
| --- | --- |
| input | E0271, associated input equality |
| output-return | E0308, return output |
| chain-last-output | E0308, last chain output |
| token-fresh-type | E0308, fresh nominal frame type |
| deferred-final-output | E0308, owning output through intermediate work |
| generic-frame | E0308, unproved generic output |
| closure-return | E0308, closure-local output |

Raw Cargo JSON, positive/negative sources and primary spans are retained in `.temp/brief-608/compile-controls/{default,no-default}/`; each results.json contains 29 receipts.

Both census configurations print:


```text
S2 CENSUS production frames=553 finalizers=54 implementations=607 typed=0 macros=589 result=345 insert=244
S2 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S2 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S2 PARTITION implementations=623 macros=600
S2 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; production evaluation remains untyped
```

The registry report separately records the private StateBool injection carrier. The production registry remains Expr Value plus 21 Legacy types; 23 legacy whole-slot types remain. Preflip still refuses without S54. The two scratch-copy falsifiers deliberately fail when a production frame or test guard is removed. Four actual compatibility-fixture legacy uses were added as X0036–X0039; no production receipt was removed or typed.

### 6. Gates, sizes, measurements and inventory replay

All verification/build commands used the prescribed bounded runner, one Cargo job, one test thread, offline Cargo and the brief-608 TMPDIR; gates were serial. Performance samples ran without a competing compile/test workload. Raw logs are under `.temp/brief-608/redispatch-1/`.

`cargo fmt --check` exited 0 with empty output at its recorded run. Later test-only import/visibility adjustments and inventory replay were not followed by a final fmt check after the scope stop. `cargo fmt` itself last completed successfully before the final internal admission-visibility adjustment.

Every recorded `test result:` line follows. The failed child results inside the two successful census logs are their required deliberate-failure controls; the first census attempt and ownership failure are real gate failures.

`census-default-1.log`
```text
test result: FAILED. 2 passed; 3 failed; 1 ignored; 0 measured; 0 filtered out; finished in 96.34s
```

`census-default-2.log`
```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.16s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.06s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 135.57s
```

`census-no-default-1.log`
```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.05s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.84s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 135.04s
```

`compile-controls-default-1.log`
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 77.22s
```

`compile-controls-no-default-1.log`
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.51s
```

`ownership-discovery-1.log`
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 5.65s
```

`ownership-gate-1.log`
```text
test result: FAILED. 24 passed; 1 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 68.89s
```

`runtime-typed-2.log`
```text
test result: FAILED. 16 passed; 1 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.04s
```

`runtime-typed-3.log`
```text
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.07s
```

`runtime-typed-4.log`
```text
test result: FAILED. 18 passed; 2 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.43s
```

`runtime-typed-5.log`
```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.45s
```

Initial compiler failures, subsequently repaired: the scratch harness first rejected `&String` as a HostEvalRequest (E0277); passing `source.as_str()` fixed it. The first runtime compile reported five errors: one old queue iteration, three test integer constructors and the new exhaustive EvalError arm. Runtime tests then exposed an incorrect new Trace print expectation and, later, two new accounting expectations that had omitted the pre-existing InputString queue; correcting those fixtures produced the 20-test pass. The initial census rejected four unrecorded test-support uses; exact discovery replay fixed it.

The final unresolved failure is verbatim:


```text
thread 'tests::ownership_graph::host_payload_ledger::tests::adapter_production_witnesses_are_computed' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs:256:5:
assertion `left == right` failed
  left: 4
 right: 3
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
```

Not run after the mandatory scope stop:

| Gate/evidence | State |
| --- | --- |
| scripts/check-suspension-oracle.sh 1500 | Not run; default/no-default fault deltas unmeasured. |
| local_production_acceptance, default | Not run. |
| local_production_acceptance, no-default | Not run. |
| cargo test --workspace --no-fail-fast | Not run. |
| Final fmt check and final focused rerun after last small visibility/import edits | Not run. |
| Candidate release builds/feature trees/error-size fixture | Not run. |
| Candidate allocation/instruction pairs and full microbench A/B | Not run. |

Native ZeroMQ tests were not run. No native pass or skip is claimed. The census's one ignored test is its explicit child-only forced-red probe, not a ZeroMQ skip.

Base error sizes were measured from the untouched one-unit release rlibs:


```text
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136
```

Candidate sizes are unmeasured; no no-growth verdict is claimed.

All four untouched base binaries were built and hashed before the first repository source edit. CLI bases remain the director-accepted prior bases; the two harness builds were added under the amendment. Base feature trees exclude test-continuation-slots.

| Binary | SHA-256 |
| --- | --- |
| `base/rum-cli` | `76efb7c611f510a4b46c8d181d77ec37c5fe7ef057c2572ce05978e7a795ecf5` |
| `base/rum-cli-alloc` | `0e9c806bab777893c83e4e7cf1411d977ac63bb77c64374af0164e40a51e56a4` |
| `base/resume` | `be2739b2e4a62ff412500e01d7edb74e362726030796b90c722167ac639eb131` |
| `base/resume-alloc` | `16aaeffd58bcf727704bd829db9ad7bc3219da67a6ccc78e87bb0b44431b0fb9` |

The reusable HostSession harness actually suspends and resumes Table, Do iterator, call arguments/repeat, WithCleanup and Fold workloads. Its five 16-resume checks passed. The full panel's 64 result checks passed; baseline allocation N/2N sampling passed all 128 cells, followed by four serial instruction samples of 128 cells each. The latter form two base/base floor pairs. CPU 2, cpu_core/instructions/u, stdin /dev/null and raw-before-parse output are recorded per cell. The panel is 54 unchanged brief-563 controls + five additional ordinary labels + five real resume workloads. Profiling flags were removed except alloc-count's explicit allocation flag.

Baseline slopes below are observations only. Candidate columns and timing rows do not exist yet. Per-label floor = max absolute relative change of base samples 1→2 and 3→4. The ordinary threshold would be max(0.05%, floor); the resume allocation/byte equality and instruction-attribution requirements remain ungated. Full raw evidence is in `.temp/brief-608/perf/{check,alloc,instructions}/base/`; `base-floor.tsv` retains all four instruction slopes.

| Label | Allocations / iteration or resume | Bytes / iteration or resume | Instructions 1 | Instructions 2 | Instructions 3 | Instructions 4 | Floor % |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| lazy-range-take-small | 141.000000 | 2714.000000 | 42212.510000 | 42187.709150 | 42165.069000 | 42225.646350 | 0.143667 |
| lazy-range-normal-small | 157.000000 | 3388.000000 | 54082.459250 | 54094.656300 | 54038.960550 | 54105.013050 | 0.122231 |
| lazy-map-realize-small | 333.000000 | 13740.000000 | 178814.536300 | 178862.670400 | 178951.100400 | 178902.628800 | 0.027087 |
| lazy-select-realize-small | 236.000000 | 6834.000000 | 114577.640100 | 114577.863000 | 114557.782750 | 114550.133950 | 0.006677 |
| lazy-select-scan-small | 1471.000000 | 62793.000000 | 805405.590750 | 805418.028450 | 805480.692800 | 805416.398500 | 0.007982 |
| lazy-nestlist-realize-small | 197.000000 | 11316.000000 | 145258.112750 | 145276.008250 | 145353.291350 | 145305.298600 | 0.033018 |
| lazy-foldlist-realize-small | 216.000000 | 7604.000000 | 112734.884450 | 112852.926750 | 112658.647050 | 112778.135700 | 0.106063 |
| lazy-drop-realize-small | 351.000000 | 14204.000000 | 191372.402000 | 191660.109300 | 191469.269550 | 191468.753300 | 0.150339 |
| lazy-first-small | 64.000000 | 2868.000000 | 41127.241000 | 41209.604600 | 41245.446350 | 41164.954800 | 0.200265 |
| lazy-part-small | 152.000000 | 6182.000000 | 88304.395750 | 88320.999850 | 88310.723400 | 88409.099250 | 0.111397 |
| lazy-transparent-adapters-small | 319.000000 | 12876.000000 | 181421.415900 | 181424.936550 | 181417.786350 | 181389.881650 | 0.015381 |
| lazy-table-construct-short | 63.000000 | 1537.000000 | 23586.675150 | 23534.728450 | 23606.819600 | 23534.760150 | 0.305248 |
| lazy-table-construct-full | 64.000000 | 1449.000000 | 24068.187850 | 24068.843350 | 24114.761700 | 24068.941700 | 0.190008 |
| lazy-table-realize-small | 251.000000 | 9190.000000 | 117177.394650 | 117176.954550 | 117225.438650 | 117117.755650 | 0.091860 |
| lazy-permutations-take-small | 29.000000 | 1729.000000 | 28160.535500 | 28197.119750 | 28199.906800 | 28198.074000 | 0.129913 |
| lazy-subsets-take-small | 29.000000 | 1417.000000 | 25353.421550 | 25362.930450 | 25361.628550 | 25363.728750 | 0.037505 |
| lazy-tuples-take-small | 29.000000 | 1361.000000 | 26629.408900 | 26581.541600 | 26580.678100 | 26565.779850 | 0.179754 |
| scan-no-input | 33.000000 | 2536.000000 | 29707.482950 | 29707.359100 | 29710.757800 | 29709.805600 | 0.003205 |
| optionvalue-three-no-input | 9.000000 | 1128.000000 | 9589.027750 | 9590.801800 | 9590.909150 | 9588.072600 | 0.029575 |
| replacerepeated-no-input | 57.000000 | 7899.000000 | 39181.208900 | 39181.481150 | 39181.370250 | 39181.346700 | 0.000695 |
| dot-three-no-input | 596.000000 | 23504.000000 | 254200.011200 | 254200.125050 | 254199.433400 | 254199.155650 | 0.000109 |
| differences-no-input | 13.000000 | 704.000000 | 10030.765300 | 10037.951500 | 10037.725300 | 10037.656050 | 0.071642 |
| do-count-null | 0.000000 | 0.000000 | 536.917850 | 536.852400 | 537.196900 | 536.908900 | 0.053612 |
| do-iterator-null | 0.000000 | 32.000000 | 1154.086250 | 1153.868500 | 1154.075600 | 1154.143350 | 0.018868 |
| table-iterator-null | 0.000000 | 32.000000 | 387.055350 | 387.063900 | 386.971250 | 387.133800 | 0.042006 |
| replaceall-simple | 5.000000 | 1352.000000 | 11406.488650 | 11406.286050 | 11458.315050 | 11406.414700 | 0.452949 |
| table-small | 22.000000 | 977.000000 | 14609.500350 | 14693.190200 | 14633.187450 | 14633.409800 | 0.572845 |
| table-iterator-plus | 2.000000 | 72.000000 | 563.311450 | 563.142550 | 563.321150 | 563.173000 | 0.029983 |
| table-iterator-pair-list | 2.000000 | 136.000000 | 1640.753850 | 1640.734600 | 1640.683050 | 1640.710750 | 0.001688 |
| take-multispec-small | 109.000000 | 5856.000000 | 49915.982500 | 49939.319000 | 49939.005850 | 49922.800900 | 0.046752 |
| lazy-depth-map-build-8 | 16.001000 | 1879.864000 | 18112.784000 | 18097.683000 | 18100.993000 | 18116.919000 | 0.087984 |
| lazy-depth-map-first-8 | 188.000000 | 11176.000000 | 130195.050000 | 130160.723000 | 130395.061000 | 130362.977000 | 0.026366 |
| lazy-depth-map-eight-8 | 1464.001000 | 82776.864000 | 959586.232000 | 959583.734000 | 959590.671000 | 959728.044000 | 0.014316 |
| lazy-depth-select-restart-8 | 327.000000 | 16980.000000 | 179401.030000 | 179410.567000 | 179391.507000 | 179093.908000 | 0.165894 |
| lazy-depth-drop-all-one-8 | 170.000000 | 3128.000000 | 46664.074000 | 46663.475000 | 46661.521000 | 46662.897000 | 0.002949 |
| lazy-depth-take-map-8 | 220.000000 | 12144.000000 | 138949.352000 | 138944.184000 | 138949.547000 | 138952.439000 | 0.003719 |
| lazy-depth-fold-through-source-8 | 743.001000 | 58440.864000 | 589552.700000 | 589542.059000 | 589555.375000 | 589547.575000 | 0.001805 |
| lazy-depth-nest-map-8 | 192.000000 | 11964.000000 | 140995.269000 | 141004.807000 | 141005.189000 | 140997.884000 | 0.006765 |
| lazy-depth-map-build-64 | 16.001000 | 5911.864000 | 30099.698000 | 30101.637000 | 30135.919000 | 30100.825000 | 0.116452 |
| lazy-depth-map-first-64 | 1364.000000 | 85320.000000 | 946029.999000 | 946178.005000 | 949361.652000 | 946179.040000 | 0.335237 |
| lazy-depth-map-eight-64 | 10872.001000 | 647704.864000 | 7422304.722000 | 7420743.085000 | 7421708.830000 | 7421176.569000 | 0.021040 |
| lazy-depth-select-restart-64 | 1671.000000 | 99412.000000 | 975572.546000 | 981973.387000 | 975907.386000 | 975628.537000 | 0.656111 |
| lazy-depth-drop-all-one-64 | 1178.000000 | 17520.000000 | 327585.768000 | 327578.805000 | 327578.349000 | 327578.517000 | 0.002126 |
| lazy-depth-take-map-64 | 1620.000000 | 93064.000000 | 1007389.461000 | 1007384.352000 | 1007397.332000 | 1007387.109000 | 0.001015 |
| lazy-depth-fold-through-source-64 | 43334.001000 | 3402649.864000 | 33996396.945000 | 33996762.081000 | 33996213.997000 | 33997389.449000 | 0.003458 |
| lazy-depth-nest-map-64 | 1368.000000 | 86108.000000 | 957227.542000 | 958894.168000 | 958616.954000 | 958481.439000 | 0.174110 |
| lazy-depth-map-build-256 | 16.001000 | 19735.864000 | 70761.477000 | 70755.099000 | 70762.050000 | 70758.986000 | 0.009013 |
| lazy-depth-map-first-256 | 5396.000000 | 339528.000000 | 3736088.202000 | 3736089.563000 | 3722996.851000 | 3722820.290000 | 0.004742 |
| lazy-depth-map-eight-256 | 43128.001000 | 2584600.864000 | 29544122.638000 | 29545651.471000 | 29546534.242000 | 29545369.660000 | 0.005175 |
| lazy-depth-select-restart-256 | 6279.000000 | 382036.000000 | 3664409.051000 | 3652394.707000 | 3691024.979000 | 3664993.586000 | 0.705262 |
| lazy-depth-drop-all-one-256 | 4634.000000 | 66864.000000 | 2218607.927000 | 2218608.706000 | 2218615.895000 | 2218606.195000 | 0.000437 |
| lazy-depth-take-map-256 | 6420.000000 | 370504.000000 | 3985388.189000 | 3985276.157000 | 3985523.944000 | 3985395.186000 | 0.003231 |
| lazy-depth-fold-through-source-256 | 689320.001000 | 54008857.864000 | 530571692.682000 | 530593565.020000 | 530598722.867000 | 530586360.173000 | 0.004122 |
| lazy-depth-nest-map-256 | 5400.000000 | 340316.000000 | 3733554.675000 | 3733565.662000 | 3733561.804000 | 3733566.419000 | 0.000294 |
| literal-integer | 0.000000 | 0.000000 | 293.982100 | 294.085900 | 293.956800 | 294.033450 | 0.035308 |
| downvalue-pattern-hit | 11.000000 | 908.000000 | 13949.662200 | 13949.526900 | 13949.502150 | 13949.319200 | 0.001312 |
| module-local | 23.000100 | 2163.627200 | 22653.028750 | 22652.422200 | 22653.505900 | 22653.120150 | 0.002678 |
| block-local | 8.000000 | 752.000000 | 14671.762450 | 14671.910250 | 14672.169950 | 14671.872950 | 0.002024 |
| matcher-single-wide-small | 11.000000 | 287280.000000 | 760206.378000 | 760180.556000 | 760206.560000 | 760205.818000 | 0.003397 |
| resume-table | 27.003000 | 1742.456000 | 15702.743000 | 15702.481000 | 15748.612000 | 15724.429000 | 0.153556 |
| resume-do-chain | 24.000000 | 1634.000000 | 15112.566000 | 15112.644000 | 15214.077000 | 15217.778000 | 0.024326 |
| resume-call-repeat | 40.007000 | 4210.024000 | 24558.253000 | 24543.015000 | 24560.841000 | 24557.573000 | 0.062048 |
| resume-cleanup | 22.000000 | 1394.000000 | 14193.674000 | 14232.925000 | 14204.980000 | 14220.405000 | 0.276539 |
| resume-fold | 41.000000 | 2873.000000 | 24438.212000 | 24452.973000 | 24454.308000 | 24536.170000 | 0.334755 |

Ownership replay passed its exact inventory and full field-specific attribution checks, even though the separate S1 count assertion remains red:


```text
HOST PAYLOAD LEDGER: 79 rows; 11 storage edges (3 Declared, 8 Unbounded); 68 PublicInput
checks 1/2/3: complete, no stale entries, field-specific expression attribution verified
external_implementation_coverage: unresolved for every Unbounded entry
EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7711 rows
```

The ledger moved the approved old row 53 storage identity to `FrameQueue::Legacy`; new Entry legacy/typed storage retains Unbounded Expr-destruction debt and unresolved external coverage. The new witnessed/token suspension inputs are PublicInput signatures, not storage or affine-consumption proofs. Old ledger rows 56–58 remain present with replayed source positions. No newly discovered host payload lacks an attributed disposition; no ownership checker rule was changed.

Inventory row populations:


```text
unknown_attributes.tsv: 1500 -> 1513 lines
unknown_captures.tsv: 2949 -> 2955 lines
unknown_macros.tsv: 3145 -> 3149 lines
unknown_storage.tsv: 91 -> 94 lines
host_payloads.tsv: 75 -> 79 lines
self_reaching.tsv: 59 -> 59 lines
host_payload_ledger.tsv: 76 -> 80 lines
```

Every removed/added row and source-location transition is disclosed in [.temp/brief-608/redispatch-1/ownership-replay.patch](/home/remi/dev/rum/.temp/brief-608/redispatch-1/ownership-replay.patch). The self-reaching inventory replay was byte-identical (59 rows); the other six inventory files changed. Adapter witness source locations were replayed and then independently verified by the complete-ledger test.

The 68 retention-contract fingerprints and both production certificate sets were **not replayed** before the scope stop. No certificate cells were written or invented; no promotion or strength changed.

### 7. Stop-loss, deviations and next decision

- Scope stop: `host_payload_ledger/tests.rs:256` needs its S1 three-site assertion reconciled with the required S2 fourth site. The full witness loop must remain. Director authorization must add this test file (or prescribe another scoped solution) before work resumes.
- No-progress stop-loss was not reached. Scratch compile errors decreased 1→0; runtime compile errors decreased 5→0; runtime tests reached 20/20; both 29-pair compiler gates and both census gates passed. Ownership's first gated attempt reached 24 passing tests and one count failure. Consecutive attempts moving no named failing gate: 0; the scope rule stopped work immediately.
- The two census forced-red child failures are intentional. Other failed attempts and unrun gates are reported above. Full runtime/workspace semantics, oracle fault deltas, certificate replay, candidate error sizes and candidate performance remain unverified.
- No git command, network request, new unsafe, production-frame conversion, workspace lockfile/toolchain change, dependency addition/version change or out-of-scope source edit occurred. The harness uses the existing allocator shim exactly as the amendment permits; its standalone scratch manifest/lockfile are not workspace changes. The tree is left uncommitted.
- Command deviation: one `ps -eo pid,ppid,args` read was used while recovering the status of a yielded baseline sample after context compaction. It is outside the brief's command whitelist; it changed no state and was not used for timing decisions. No process-aborting probe was run.
- The request for VerificationTest was exercised through rum's existing TestCreate/TestReport contract because no VerificationTest implementation exists in this repository. No compatibility alias was added.
- Scratch artifacts and the baseline harness are retained for redispatch. Checkpoint: `.temp/brief-608/PROGRESS.md`. No further permission-dependent action was attempted.

## Director amendment — redispatch 2

The stop was correct. Two rulings and one correction of my own, then the
remaining work.

1. **The witness count (your stop).** Scope gains
   `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs`
   for one change in `adapter_production_witnesses_are_computed`: replace
   `assert_eq!(rows.len(), 3)` with an exact set equality over the four
   discovered constructs, spelled as discovery prints them
   (`Action::Evaluate.next`, `Action::Finalize.0`, `FrameChain.frames`,
   `Entry::Typed.0`). Keep the witness loop unchanged. A count becomes names,
   so a later step that adds or removes an adapter storage site has to name
   it. No other change to that file, to any other ownership test, or to the
   checker.

2. **The public variant's payload — a correction to step 4 (my error).** As
   written, `EvalError::ContinuationSlotMismatch { expected: &'static str,
   got: &'static str }` has public fields, so any crate can construct it,
   including a builtin registered outside `rum-runtime`, and return it from
   inside evaluation. A frame that then packs its outcome through the legacy
   `EvalError` wire (`result_into_continuation_expr`,
   `suspension/error.rs:619`, used for example by the Block initializer
   finalizer at `scoping/lexical/block/resume_frames.rs:43` and the loop
   iterator scope) reaches the new `unreachable!` at `error.rs:271` and
   panics. Inside the repository the arm's invariant holds only because no
   code constructs the variant; the crate edge does not hold it. Fix it by
   construction:
   - The variant carries an opaque payload:
     `EvalError::ContinuationSlotMismatch(SlotMismatch)`. `SlotMismatch` has
     private fields, public `expected()` / `got()` accessors returning
     `&'static str`, and the derives `EvalError` needs. Its only production
     constructor is visible to `runtime::suspension` alone (the driver
     fault's projection); runtime unit tests may use a `cfg(test)`
     constructor. Export the type wherever `EvalError` is exported so hosts
     can name it in a match.
   - `SessionRequestError::ContinuationSlotMismatch` is an output to the host
     and never re-enters the kernel; keep its shape unless carrying
     `SlotMismatch` is simpler. Report which.
   - The `error.rs` arm stays an invariant guard, now reachable only from
     `rum-runtime` code, where the one constructor feeds the two public
     resume boundaries (`session_requests.rs:307`, `session_eval.rs:189`)
     and `continuation_fault_outcome`. Its message says the legacy wire, and
     the arm with it, are removed at R.
   - **Compile control, external crate** (one new S2 pair, 30 in all): the
     positive matches `EvalError::ContinuationSlotMismatch(fault)` and reads
     `fault.expected()` / `fault.got()`; the negative constructs the payload
     or the variant directly. Require the privacy diagnostic rustc actually
     emits, with its code and exact primary span, in both feature builds.
   - `size_of::<EvalError>()` and `size_of::<Result<Expr, EvalError>>()` stay
     136 on the candidate; measure them.

3. **`VerificationTest` was my error.** rum has no such builtin; exercising
   the test-case capture through `TestCreate`/`TestReport` is the right
   substitute. The single `ps` read is accepted.

4. **Then finish the brief.** From the ownership gate onward, everything the
   Verification section requires, on the final tree: ownership 25/25 with the
   witness lines printed; `cargo fmt --check`; the retention fingerprints and
   both production certificate sets replayed as the Steps require (no cell
   invented); `scripts/check-suspension-oracle.sh 1500` in both builds with
   the fault delta measured and reported as zero; `local_production_acceptance`
   in both builds; compile controls (30) and census in both builds; candidate
   release builds, feature trees and the error-size fixture; the candidate
   halves of the ordinary panel and the resume harness against your
   recorded bases (allocations first, then one-unit instruction slopes, floor
   from base/base, every resume delta attributed); and the workspace run.
   Rerun the focused typed tests after the payload change. Append a new
   `## Execution report` after this section.

## Execution report — redispatch 2: STOP at the historical retention census

The opaque-fault correction and exact four-site ownership assertion are implemented. Formatting, 20 typed tests, 30 compiler-control pairs in both builds, both censuses, ownership 25/25, both production certificate replays and both 50-test suspension oracles pass. Each oracle measured fault-occurrence delta **0**, with **87,920 observations**.

The workspace gate exposes another source-location assertion outside Scope: `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:190`. It requires three unresolved abandonment method sites at `suspension/core.rs:667,668,684`; the corresponding operations now live at `suspension/core/abandon.rs:42,43,59`. That test file is not one of the permitted retention maintenance files and this is not an exhaustiveness error.

This invokes Disclosure duty: **“make no edit there; STOP and report the file, the line and why.”** The test was not edited. Its subsequent exact frontier total (932) and kind totals have not been reconciled; the location assertion fails first. A director ruling is needed for source-discovery maintenance of that test while retaining its unresolved-boundary assertions. No checker rule or contract was weakened to evade it.

No new implementation or verification command was started after identifying the scope stop. The already-running `cargo test --workspace --no-fail-fast` was allowed to finish. Candidate release builds, candidate error-size measurement and candidate performance remain unrun, so **S2 is not fully gated or complete**.

### 1. Files changed

This is the complete S2 source/inventory change, including the implementation from redispatch 1. The brief itself receives only this appended report. The redispatch-2 source snapshot, exact patch and file descriptions are retained under `.temp/brief-608/redispatch-2/`.

| File | Semantic change |
| --- | --- |
| [crates/rum-host/src/model/outcome.rs](/home/remi/dev/rum/crates/rum-host/src/model/outcome.rs) | Map ContinuationSlotMismatch to the existing terminal HostFailureCode::Other outcome. |
| [crates/rum-runtime/Cargo.toml](/home/remi/dev/rum/crates/rum-runtime/Cargo.toml) | Add the empty test-continuation-slots feature. |
| [crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs) | Explicitly propagate the public fault variant from capture rendering. |
| [crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs) | Independently enumerate legacy or mixed queue cells and both typed Arc allocations; visit frame payloads once. |
| [crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs) | Explicit fatal passthrough in capture completion and outcome rendering. |
| [crates/rum-runtime/src/runtime/kernel_api.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_api.rs) | Export the typed macro, the feature-gated FaultProbe and the opaque SlotMismatch beside EvalError. |
| [crates/rum-runtime/src/runtime/mod.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/mod.rs) | Export feature-gated FaultProbe and public SlotMismatch at the runtime facade. |
| [crates/rum-runtime/src/runtime/model/requests.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/model/requests.rs) | Add SessionRequestError::ContinuationSlotMismatch and its Display arm. |
| [crates/rum-runtime/src/runtime/session/session_eval.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_eval.rs) | Project the latched fault at text/inspection resume boundaries before legacy value decoding. |
| [crates/rum-runtime/src/runtime/session/session_requests.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_requests.rs) | Project the latched fault at request boundaries and map the new EvalError variant to SessionRequestError. |
| [crates/rum-runtime/src/runtime/session/session_state.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_state.rs) | Add EvalError::ContinuationSlotMismatch(SlotMismatch), formatted through read-only static-label accessors. |
| [crates/rum-runtime/src/runtime/suspension/core.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core.rs) | Integrate compatible queue owners, typed insertion/layout projection and the typed/legacy driver; extract structural abandonment. |
| [crates/rum-runtime/src/runtime/suspension/core/abandon.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core/abandon.rs) | Move the existing abandonment worklist protocol into its own module and share its drain with faults. |
| [crates/rum-runtime/src/runtime/suspension/core/fault.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core/fault.rs) | First-fault latch and structural unwind; opaque SlotMismatch with suspension-only production constructor and cfg(test) injection constructor; feature/test-only monotonic FaultProbe. |
| [crates/rum-runtime/src/runtime/suspension/core/queue.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/core/queue.rs) | New legacy-buffer-preserving queue; widen to real Entry cells only when needed. |
| [crates/rum-runtime/src/runtime/suspension/error.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/error.rs) | Exhaustiveness arm guards against structural faults entering the legacy error wire; explicitly removed with that wire at R. |
| [crates/rum-runtime/src/runtime/suspension/output.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/output.rs) | Collect capture boundaries from either queue representation without cloning frame state. |
| [crates/rum-runtime/src/runtime/suspension/trace.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/trace.rs) | Re-export FaultProbe and SlotMismatch through the existing suspension module boundary. |
| [crates/rum-runtime/src/runtime/suspension/typed.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed.rs) | Remove the module-wide dead-code allowance and connect private driver exports. |
| [crates/rum-runtime/src/runtime/suspension/typed/action.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/action.rs) | Make sealed actions reachable by the driver; add only a cfg(test)-private malformed-slot constructor. |
| [crates/rum-runtime/src/runtime/suspension/typed/adapter.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/adapter.rs) | Expose the inner borrowed layout to both accounting models; preserve exact decode and adapter-only token minting. |
| [crates/rum-runtime/src/runtime/suspension/typed/chain.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/chain.rs) | Implement witnessed chain installation and return the last output type. |
| [crates/rum-runtime/src/runtime/suspension/typed/channel.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/channel.rs) | Centralize exact completion decoding and static fault kind names; no dual-arm decode. |
| [crates/rum-runtime/src/runtime/suspension/typed/entry.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/entry.rs) | New private legacy/typed entry, dispatch action and borrowed metadata/accounting view. |
| [crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs) | Accept fresh state of the same nominal frame and return ResumeStop<F::Output>. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests.rs) | Connect live route, driver-fault, file and accounting tests. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs) | Verify two-Arc layout, real queue cells, maintained/independent totals and below/at/above admission. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs) | Fault matrix across captures/scopes, cleanup, completion, rollback/reset and direct public-variant capture defence. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/fault_support.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/fault_support.rs) | Runtime-private wrong-mode/exact-state fixtures, non-running consumer/suffix spies and common restoration assertions. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs) | Nested memory-file and public-batch rollback under all three injected mismatch kinds. |
| [crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs) | Test macro/chain, legacy neighbors, repeat helper, nested Evaluate and both-Arc finalizer release. |
| [crates/rum-runtime/src/runtime/suspension/typed/witness.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/suspension/typed/witness.rs) | Implement witnessed suspension and the cold actual-Result typed macro. |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/frame_root_trace.rs) | Keep exact cleanup capability confinement across the extracted abandonment module; require successful exact decode before pop, release before both finalizers, and the shared structural worklist without resume. |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs) | Record SlotMismatch's new FrameRootTrace derive in the sorted closed inventory. |
| [crates/rum-stdlib/Cargo.toml](/home/remi/dev/rum/crates/rum-stdlib/Cargo.toml) | Enable that feature only on the existing runtime dev-dependency. |
| [crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs) | Replace only the three-site count assertion with exact set equality over four discovered constructs; witness loop byte-identical. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv) | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv) | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv) | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | Replay all 68 dependency fingerprints; preserve every contract field and strength. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](/home/remi/dev/rum/crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | Replay exactly the four emitted default/no-default C01/C02 certificate cells; preserve statuses, bodies, preconditions, strengths and exclusions. |
| [crates/rum-stdlib/tests/suspension_oracle.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle.rs) | Add the serial last aggregate fault-occurrence assertion. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs) | Select the S2 typing receipt while continuing to refuse preflip. |
| [crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv) | Record S2 mechanism routes, unchanged observed T0010, and four discovered test-support legacy uses X0036-X0039. |
| [crates/rum-stdlib/tests/suspension_oracle/support/drive.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/drive.rs) | Unconditional fault observer around every shared drive, independent of Checks. |
| [crates/rum-stdlib/tests/suspension_oracle/support/faults.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/faults.rs) | New unconditional guards and retained probe observations for the per-build aggregate assertion. |
| [crates/rum-stdlib/tests/suspension_oracle/support/lifecycle.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/lifecycle.rs) | Check before session replacement/reset and retain the old probe through lifecycle completion. |
| [crates/rum-stdlib/tests/suspension_oracle/support/mod.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/mod.rs) | Wire the fault-observer support module. |
| [crates/rum-stdlib/tests/suspension_oracle/support/session.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/suspension_oracle/support/session.rs) | Observe initialization, retirement, fresh requests and input-queue controls. |
| [crates/rum-stdlib/tests/typed_slot_census.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census.rs) | Connect route receipts, S2 preflip refusal and scratch-copy falsifiers under brief-608. |
| [crates/rum-stdlib/tests/typed_slot_census/classify.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census/classify.rs) | Keep codec population metadata so private injection carriers are reported separately from production registry entries. |
| [crates/rum-stdlib/tests/typed_slot_census/receipts.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census/receipts.rs) | Preserve exact production/test-only counts and production mode checks; identify output as S2. |
| [crates/rum-stdlib/tests/typed_slot_census/reporting.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census/reporting.rs) | Write under brief-608 and include codec population in discovery output. |
| [crates/rum-stdlib/tests/typed_slot_census/routes.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_census/routes.rs) | New live source/witness checks for six S2 routes, unchanged family ownership of four remaining routes and T0010 observation. |
| [crates/rum-stdlib/tests/typed_slot_compile_controls.rs](/home/remi/dev/rum/crates/rum-stdlib/tests/typed_slot_compile_controls.rs) | Retain 22 S1 pairs, add seven S2 pairs and the external opaque-payload pair; require E0451 with both exact privacy spans in each feature build. |

The only compiler-required exhaustive arm is in `suspension/error.rs`: a structural fault cannot enter the legacy error wire; the invariant guard now explicitly says that this wire and guard disappear at R. It is protected by the opaque payload. Other capture, request and host arms are in explicitly scoped files. `SessionRequestError::ContinuationSlotMismatch { expected, got }` keeps its named static-string output shape.

`SlotMismatch` has private fields and public `expected()` / `got()` accessors. Its production `new` is restricted to `runtime::suspension` and is called by DriverFault projection; the separate injection constructor is `cfg(test)`. The type is exported at both EvalError export surfaces (runtime/crate root and kernel_api). The positive external control names both exports and reads both labels; the negative cannot initialize either private field.

The ownership witness file changed only the authorized assertion. Its prefix, unchanged witness loop and remainder are byte-identical to the entry snapshot. The set contains the full discovered identities of Action.Evaluate.next, Action.Finalize.0, FrameChain.frames and Entry.Typed.0.

Split assessment: the driver split has begun with core/queue.rs, typed/entry.rs, core/fault.rs and core/abandon.rs. Core retains owner lifecycle, insertion and resumption; it is 27,551 characters, below the 35k limit. All new modules are under 20k. The live-route test is 11.6k, a cohesive route matrix. The existing compiler-control test is 21,012 characters, within the test-file warning band; future separation is fixture definitions versus diagnostic harness. Existing larger files received bounded changes: session_requests.rs (34,742; future request handlers versus error conversion), session_state.rs (24,330; state types versus EvalError), error.rs (26,604; legacy encode/decode, removed at R), host outcome.rs (29,650; error conversion versus outcome rendering). The 33,350-character frame-root test can separate protocol source checks from inventory checks later. Its 175,662-character closed derive inventory is existing data debt; one sorted SlotMismatch entry was added. No unrelated split or TODO edit was made.

### 2. Driver, routes, fault path and probe

Legacy-only owners retain VecDeque cells containing one Arc<dyn SuspensionFrame>; a typed insertion widens that owner to private Entry cells. Queue representation does not choose transport. Legacy accepts Value only; typed transport is its type's sealed associated mode, with no dual-arm decoder or per-frame override.

I0001 borrows and matches the actual producer Result once and constructs the frame only for InputRequired. I0002 checks the first input, adjacency and last output. I0003 uses the adapter-minted token and fresh state of the same nominal frame. I0007 requires an Expr-input successor with the parent's output. I0008 releases both frame Arcs before the pure finalizer. I0009 seeds a Value string, places nested owners before the unrun suffix and exactly decodes completion. I0004/I0005/I0006/I0010 remain S5/S18/S14/S24; T0010 observes the unchanged CLI mapping. No production frame was typed and no codec was flipped.

The driver resumes an Arc clone while the queue retains the consumer; it never clones frame state. A decode failure therefore leaves the consumer at the active front. The first private DriverFault is latched, active and nested owners enter the existing accounted abandonment worklist, pending handles are cleared, output rolls back, and structural abandonment restores scopes and loaders. Approved WithCleanup evaluation is the only guest cleanup transition; its nested input owner is abandoned before the saved suffix. Cleanup failures cannot replace the first fault. Only after unwind does the boundary project SlotMismatch; it never enters the frame-completion loop. EvaluationData and TestReport also propagate a direct internal public-variant return.

FaultProbe exists only in runtime tests or the test-continuation-slots feature. Only creation of the first private fault increments it. Retained observers survive reset, rollback and session replacement. Oracle checks are independent of Checks flags, run before lifecycle reset, and retain observations for the final aggregate. Both measured oracle deltas are zero. Release feature exclusion is implemented but its candidate feature trees remain unmeasured because of the stop.

### 3. Fault-injection matrix

All injections are runtime-unit-test-only. The three mismatch kinds are Value-for-State, State-for-Value and the wrong exact State variant.

| Cases | Assertions exercised |
| --- | --- |
| EvaluationData, TestReport/TestCreate, Catch, Check, Quiet, Trace, Block, iterator, OptionsPattern: 9 × 3 | Exact first fault; one occurrence; rejected consumer and two suffix sentinels never resume; all abandon in order; print/message/trace rollback and visibility; restored scopes/loaders/handles and outer variables; fresh 1+1 = 2. |
| WithCleanup: Abort, cleanup InputString, different direct internal fault | First fault preserved; one occurrence; separately nested owner abandoned before consumer/suffix; cleanup runs once; cleanup post-input suffix does not run; fresh request succeeds. |
| Completion: all 3 kinds | Exact completion decode; empty owner/worklist/handles; rollback retains probe count; old probe survives replacement; fresh request. Expr is the public boundary and private StateBool exercises exact State completion. |
| Nested memory files: 3 kinds × Get/public batch routes | Both loader levels roll back; definitions and Block scope restore; cleanup output retained; sentinel abandonment and one occurrence; no outstanding input; fresh request. |
| Direct internal public fault under EvaluationData and TestReport | Fatal passthrough without a guest failure value; no pending owner; fresh request; probe remains zero because no private decode fault was created. |

The director accepted TestCreate/TestReport as the correct rum surface in place of the nonexistent VerificationTest. No compatibility alias was added. Cases without a queued consumer do not claim a consumer assertion.

### 4. Accounting

Typed charge = outer Arc header + adapter inline size + inner Arc header + F inline size; legacy retains its single-Arc layout. Capacity uses actual legacy or mixed cell size. The independent model enumerates both owners without calling the maintained helper; frame payloads are visited once. Typed projected admission is available for S18 without converting its route.

Below/at/above admission tests use the real pre-existing InputString queue. Widening its four cells and adding a zero-sized typed frame costs exactly 72 bytes: 40 for both Arc layouts and 32 for wider cells. Maintained/independent reconciliation also covers nested Evaluate, finalization, capture/scope faults, cleanup input, reset and nested/public file rollback. Legacy per-resume allocation/byte equality has not yet been measured on the candidate.

### 5. Compiler controls and census

Both builds pass 30 positive / 30 intended rejections: all 22 S1 pairs, seven S2 pairs and the opaque-fault pair. The seven S2 controls retain input E0271 and output-return / chain-last-output / token-fresh-type / deferred-final-output / generic-frame / closure-return E0308. Positives compile first; each negative has one invalid construction and pins its diagnostic category and source location.

The new external pair reads both public accessors in its positive fixture. Its negative gets **E0451** twice at `src/lib.rs:124:95-114` (expected) and `src/lib.rs:124:115-124` (got). Both complete primary spans are checked and retained with Cargo JSON in `.temp/brief-608/compile-controls/{default,no-default}/results.json`.

The final census output is:


```text
S2 CENSUS production frames=553 finalizers=54 implementations=607 typed=0 macros=589 result=345 insert=244
S2 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S2 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S2 PARTITION implementations=623 macros=600
S2 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; production evaluation remains untyped
S2 ROUTE I0001 mechanism=rum_runtime::runtime::suspension::typed::witness::<SlotWitness < I > as inherent>::suspend witness=macro_is_cold_and_a_witnessed_chain_returns_its_last_output; production family frames remain untyped
S2 ROUTE I0002 mechanism=rum_runtime::runtime::suspension::typed::chain::<FrameChain < I , O > as inherent>::suspend witness=macro_is_cold_and_a_witnessed_chain_returns_its_last_output; production family frames remain untyped
S2 ROUTE I0003 mechanism=rum_runtime::runtime::suspension::typed::resume_token::<ContinueToken < '_ , F > as inherent>::suspend witness=repeat_token_freezes_fresh_state_and_keeps_the_deferred_output_type; production family frames remain untyped
S2 ROUTE I0007 mechanism=rum_runtime::runtime::suspension::typed::action::<TypedAction < O > as inherent>::evaluate witness=evaluate_successor_and_nested_owner_precede_the_unrun_suffix; production family frames remain untyped
S2 ROUTE I0008 mechanism=rum_runtime::runtime::suspension::typed::action::<TypedAction < O > as inherent>::finalize witness=driver_drops_both_owning_arcs_before_running_the_finalizer; production family frames remain untyped
S2 ROUTE I0009 mechanism=rum_runtime::runtime::suspension::core::<Session as inherent>::drive_resuming_frames witness=completion_exact_decode_and_probe_survive_rollback_and_replacement; production family frames remain untyped
S2 ROUTE I0004 remains untyped; family=S5
S2 ROUTE I0005 remains untyped; family=S18
S2 ROUTE I0006 remains untyped; family=S14
S2 ROUTE I0010 remains untyped; family=S24
S2 T0010 observed unchanged: public CLI Result mapping; no slot conversion
```

Both frame-removal and test-guard-removal falsifiers reject as intended in both builds. Their child FAILED totals below are intentional. Preflip still refuses without S54; private test carriers are not production registry entries. X0036–X0039 record four actually discovered compatibility-fixture uses; no production receipt was removed.

### 6. Gates, sizes, measurements and maintenance evidence

Commands ran serially through scripts/run-bounded.py with CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1, CARGO_NET_OFFLINE=true and the brief-608 TMPDIR. Ordinary gate bounds were 900s, workspace 3600s and the oracle 1500s per build (3030s envelope for the mandatory two-build script). The script uses its own existing scratch directory; its per-build logs were copied into this brief's scratch. Workspace additionally used --nocapture and RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1, which allows a skip only after actual loopback PermissionDenied. The existing script/test-owned scratch locations were not altered.

`cargo fmt --check` exited 0 with empty output. All amended-run test-result totals follow verbatim, including repaired failures and the ongoing-at-stop workspace command's final result. Individual oracle log copies duplicate the script output and are not repeated a third time. Earlier redispatch-1 totals remain in that report above.

`census-default-1.log` (exit 0)
```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.21s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.77s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 138.89s
```

`census-no-default-1.log` (exit 0)
```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.83s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.62s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 140.96s
```

`certificates-default-1.log` (exit 101)
```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.27s
```

`certificates-default-2.log` (exit 0)
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.34s
```

`certificates-no-default-1.log` (exit 101)
```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.15s
```

`certificates-no-default-2.log` (exit 0)
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 17.89s
```

`compile-controls-default-1.log` (exit 0)
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.95s
```

`compile-controls-default-2.log` (exit 0)
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.03s
```

`compile-controls-default-3.log` (exit 0)
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.28s
```

`compile-controls-no-default-1.log` (exit 0)
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.18s
```

`compile-controls-no-default-2.log` (exit 0)
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.58s
```

`frame-root-1.log` (exit 0)
```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.24s
```

`oracle-both-1.log` (exit 0)
```text
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.43s
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.32s
```

`ownership-discovery-1.log` (exit 0)
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 5.71s
```

`ownership-gate-1.log` (exit 0)
```text
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 69.13s
```

`runtime-all-1.log` (exit 101)
```text
test result: FAILED. 1168 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 46.02s
```

`runtime-typed-1.log` (exit 0)
```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.40s
```

`runtime-typed-2.log` (exit 0)
```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.42s
```

`workspace-1.log` (exit 101)
```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 73.07s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.60s
test result: FAILED. 1091 passed; 1 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1205.44s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.69s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.26s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.80s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.73s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.57s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.14s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.72s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.44s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.68s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The first full runtime run had one source-layout assertion failure; the scoped frame-root invariant test was updated without weakening its ownership checks. It then passed 15/15, and workspace runtime passed 1169 with 3 ignored. Both first certificate runs earned all marks with zero rejections, wrote fresh cells and then rejected stale inventory certificates; both replay runs passed. These are disclosed failures, not omitted attempts.

Measured oracle observations:

`default`
```text
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
```

`no-default`
```text
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
```

The scope-stopping failure, verbatim:
```text
thread 'tests::retention_closure::census::tests::production_rejection_census_is_complete' panicked at crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:190:9:
assertion failed: options.rejected.iter().any(|row|
        row.kind == Kind::RET002 &&
                row.site.file.ends_with("suspension/core.rs") &&
            row.site.start.0 == line)
FAILED
```

Native ZeroMQ: **9 skipped** on denied loopback access. Rust's aggregate lists these early-return cases as passed; they are reported here as skips, not native transport passes.
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
```

Base error sizes (previously measured from the untouched release rlibs):
```text
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136
```

Candidate sizes are unmeasured. No no-growth verdict is claimed. The four original, pre-edit, one-codegen-unit base binaries remain available:

| Binary | SHA-256 |
| --- | --- |
| `.temp/brief-608/base/rum-cli` | `76efb7c611f510a4b46c8d181d77ec37c5fe7ef057c2572ce05978e7a795ecf5` |
| `.temp/brief-608/base/rum-cli-alloc` | `0e9c806bab777893c83e4e7cf1411d977ac63bb77c64374af0164e40a51e56a4` |
| `.temp/brief-608/base/resume` | `be2739b2e4a62ff412500e01d7edb74e362726030796b90c722167ac639eb131` |
| `.temp/brief-608/base/resume-alloc` | `16aaeffd58bcf727704bd829db9ad7bc3219da67a6ccc78e87bb0b44431b0fb9` |
The 64 baseline correctness checks, 128 allocation N/2N cells and four instruction samples of 128 cells were completed before source edits. Four instruction samples provide the two base/base floor pairs. Raw outputs, commands and slopes remain in `.temp/brief-608/perf/`; the earlier execution report above prints every base allocation/instruction row. No baseline measurement was rerun during this redispatch. The reusable public HostSession harness still drives actual Table, Do-chain, call-repeat, WithCleanup and Fold resumes.

| Required candidate evidence | Status |
| --- | --- |
| Four one-unit release builds and feature trees | Not run after scope stop. |
| EvalError and Result<Expr, EvalError> size fixture | Not run; candidate 136-byte condition unverified. |
| 64 result checks and allocation/byte slopes | Not run; no equality claim. |
| Two paired instruction samples and per-resume attribution | Not run; no regression verdict. |
| Full core-language-microbench base/candidate timing panel | Not run; no timing rows. |

Scratch helpers for those remaining commands were prepared before the stop but not executed: build-candidate.py, error-sizes.py and timing.py in redispatch-2. The original perf/run.py and perf/compare.py remain available. No process-aborting probe was run.

Ownership replay and actual witness output:


```text
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 79 rows; 11 storage edges (3 Declared, 8 Unbounded); 68 PublicInput
checks 1/2/3: complete, no stale entries, field-specific expression attribution verified
external_implementation_coverage: unresolved for every Unbounded entry
test tests::ownership_graph::host_payload_ledger::tests::adapter_production_witnesses_are_computed ... ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	19	15	rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	21	14	rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer	rum_runtime::runtime::suspension::typed::action::ErasedFinalizer => rum_runtime::runtime::suspension::typed::adapter::FinalizerAdapter.0 (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:82:1) => rum_runtime::runtime::suspension::typed::SlotFinalizer (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	9	13	rum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	14	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7712 rows
```


```text
unknown_attributes.tsv: 1513 -> 1514 lines
unknown_captures.tsv: 2955 -> 2955 lines
unknown_macros.tsv: 3149 -> 3149 lines
unknown_storage.tsv: 94 -> 94 lines
host_payloads.tsv: 79 -> 79 lines
self_reaching.tsv: 59 -> 59 lines
host_payload_ledger.tsv: 80 -> 80 lines
```

Redispatch 2 preserved all 79 host-payload identities and all disposition/witness cells. Its one added UnknownConstruct is the SlotMismatch derive; other differences are actual source positions. Combined S2 discovery is 79 host payloads, 7712 UnknownConstruct rows and 59 self-reaching types. The approved redispatch-1 queue/insertion transitions retain Unbounded Expr-destruction debt and unresolved external coverage; old ledger rows 56–58 remain. No new host payload lacks an approved disposition.

Every ownership row transition is disclosed in the two replay patches:
- [.temp/brief-608/redispatch-1/ownership-replay.patch](/home/remi/dev/rum/.temp/brief-608/redispatch-1/ownership-replay.patch)
- [.temp/brief-608/redispatch-2/ownership-replay.patch](/home/remi/dev/rum/.temp/brief-608/redispatch-2/ownership-replay.patch)
All 68 contract fingerprint changes are recorded without altering the first 18 fields:
```text
id	before	after
B01	fnv1a64:7ada97087bc2eb12	fnv1a64:2762acca6234859c
B02	fnv1a64:4f8cfbade00a45d3	fnv1a64:05dedfbde4406b55
B03	fnv1a64:02d11fe622095a20	fnv1a64:c613714abc98e842
B04	fnv1a64:9cfd4de02b06cffa	fnv1a64:deefdade6d385a14
B05	fnv1a64:3d5f6a0175207bbf	fnv1a64:a632ca753bb166f9
B06	fnv1a64:68354a189c4a2cb6	fnv1a64:465d5b4a51ea8e98
B07	fnv1a64:3b744f79abcbb9cf	fnv1a64:bec4f1db53e5b049
B08	fnv1a64:812f75f1ee8b184c	fnv1a64:8596ba743f3d80e6
B09	fnv1a64:5f698287e1a5d5be	fnv1a64:fab1ed723a57fcd0
B10	fnv1a64:fcb04cb114deeba1	fnv1a64:2740ee773bb1c57b
B11	fnv1a64:5e54ff3d44faf5fa	fnv1a64:a0478c3b872c8014
B12	fnv1a64:1e760283bdae6abe	fnv1a64:b9be6d6e166091d0
I01	fnv1a64:58bc8205e87ae192	fnv1a64:054497c7ceec7c1c
I02	fnv1a64:94a0e7956d5703bf	fnv1a64:fd74480933e7eef9
I03	fnv1a64:0f3630bae623967e	fnv1a64:49509a594e34e490
I04	fnv1a64:2fe01dbbc2e15bf3	fnv1a64:e8465b54c60dbf75
P01	fnv1a64:a3f7a12ceca0ecd6	fnv1a64:3d980a7408b95238
P02	fnv1a64:d17d0bd726333711	fnv1a64:52a9ab73419a3c6b
P03	fnv1a64:1fb7f132ffd071b8	fnv1a64:a2502fee344e972a
P04	fnv1a64:446819c364b361d3	fnv1a64:fab9fdd368e98755
P05	fnv1a64:83d166cefaa79082	fnv1a64:d1103a6554610c8c
P06	fnv1a64:6ef5275449de043e	fnv1a64:0a3d923ea2902b50
P070	fnv1a64:a942c39628537e3e	fnv1a64:448b2e808105a550
P080	fnv1a64:07f91d9239b48a61	fnv1a64:3ec096221ca40b3b
P090	fnv1a64:dc03d72fd558a493	fnv1a64:8985e4762056ff15
P100	fnv1a64:acd0d826b38999ca	fnv1a64:c1eaea7d774c66e4
P110	fnv1a64:73636a8f5462b399	fnv1a64:1bebf6c452e014e3
P120	fnv1a64:d032459e337b3aec	fnv1a64:02fba299a1c7a486
P130	fnv1a64:df41a8d6da7f93cf	fnv1a64:62924b3882998a49
P140	fnv1a64:f985b65b1757f78b	fnv1a64:acf9a84971a3d15d
P150	fnv1a64:833ea4f8f66042b3	fnv1a64:4475b95bb2c62435
P071	fnv1a64:210addf8ae1248ef	fnv1a64:9338971a420e0369
P081	fnv1a64:2ea814a8d79a40de	fnv1a64:e97255fbe9b45af0
P091	fnv1a64:75a89ccaab950996	fnv1a64:6f840adc2356c1f8
P101	fnv1a64:6b665b5c89c65099	fnv1a64:13eee7918843b1e3
P111	fnv1a64:33888cb37d7b3b8e	fnv1a64:ee9a9e79b957bb20
P121	fnv1a64:eb36c5ea4fc74285	fnv1a64:219326fb63c78347
P131	fnv1a64:3aa77046615af17a	fnv1a64:7c99fd44a38c7b94
P141	fnv1a64:f4cf17ea4589ac76	fnv1a64:128a302b25614858
P151	fnv1a64:69d64ea9445f7d2c	fnv1a64:f5dbd1ca1e6e9ac6
B13	fnv1a64:f645dda1cce6e8b6	fnv1a64:d46deed382874a98
R01	fnv1a64:06969837ac013363	fnv1a64:5773a351b8ebc6e5
R02	fnv1a64:45d2821f7b89a678	fnv1a64:36713fc1d1264aea
R03	fnv1a64:63787705e7e53c57	fnv1a64:5c2ff0785c7b51e1
R040	fnv1a64:ff1ab67c57a9af83	fnv1a64:07474a8f04dd5885
R050	fnv1a64:608ceea73aab4062	fnv1a64:bbbf3e0544aa18ec
R060	fnv1a64:0502f106e0ac7b5b	fnv1a64:b176295276086b2d
R070	fnv1a64:9715dcc2b93bffde	fnv1a64:51e01e15cb5619f0
R080	fnv1a64:f6e486ae926624f3	fnv1a64:af4ac44795928875
R090	fnv1a64:dc2b8b37e76eea8a	fnv1a64:f3970a120e1b32a4
R100	fnv1a64:c7f6c438cbdf04a0	fnv1a64:8b39159d666e92c2
R110	fnv1a64:285b2ad7a2e14ae9	fnv1a64:aa62ff0e9a0c09f3
R041	fnv1a64:dc7f40cbe40ea7b6	fnv1a64:baa751fd99af0998
R051	fnv1a64:6a2605456286b113	fnv1a64:17a8128bad850b95
R061	fnv1a64:856ca3a65d017f3c	fnv1a64:e9853b5a0f617c96
R071	fnv1a64:52bf5b832d6daf0d	fnv1a64:95acc10af451045f
R081	fnv1a64:68bb417b4f356eca	fnv1a64:7dd553d212f83be4
R091	fnv1a64:843bfbc9ef924fcb	fnv1a64:7498f0c7dbf5d39d
R101	fnv1a64:cf04a6f439b0b41b	fnv1a64:c8320d48fb6d6ded
R111	fnv1a64:819e4394f206cb94	fnv1a64:3d4902991701d7de
R12	fnv1a64:13bf7d9beb6ea2d4	fnv1a64:d12c6648cf30221e
R13	fnv1a64:de664c72d43f4a90	fnv1a64:f12439cbd985d432
R14	fnv1a64:c728e69c903a59ca	fnv1a64:dc42f8f353fd26e4
R15	fnv1a64:2ef558386079c4a6	fnv1a64:f5bd7211e095a408
O010	fnv1a64:2190c40298947f1c	fnv1a64:1de605b4d5860476
O0200	fnv1a64:f44d582f660f1d8b	fnv1a64:a7c14a1dc05af75d
O011	fnv1a64:478bad8eee3e3fdb	fnv1a64:f3fee5da839a2fad
O0210	fnv1a64:158a42fb699f0fb4	fnv1a64:66ad5f3f573896fe
```

Both certificate builds earned two C01 concrete instances, one nongeneric C02 and four separate LeaseProbe marks, with zero rejections. Their actual acceptance outputs:
`hash=true`
```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```

`hash=false`
```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```

Only four emitted certificate cells were replayed; all other columns remain unchanged. Full old/new cells and the exact inventory patch are retained:
- [.temp/brief-608/redispatch-2/certificate-diff.json](/home/remi/dev/rum/.temp/brief-608/redispatch-2/certificate-diff.json)
- [.temp/brief-608/redispatch-2/certificate-replay.patch](/home/remi/dev/rum/.temp/brief-608/redispatch-2/certificate-replay.patch)

### 7. Stop-loss, deviations and next decision

The source-location failure at retention_closure/census/tests.rs:190 is outside Scope. Its old locations and subsequent exact historical frontier counts need an authorized discovery replay, preserving all unresolved boundary checks and without changing the checker. That file was not edited. No new gate or implementation change began after the stop; the already-running no-fail-fast workspace command completed naturally.

No-progress stop-loss count is 0. The opaque payload compiled immediately, all 30 compiler controls passed, the runtime source-layout failure was repaired and the full runtime suite passed, ownership moved to 25/25, certificate builds earned/replayed their marks, and both oracle deltas were measured at zero. This is a scope stop, not a stalled two-attempt loop.

No git command, network request, new unsafe, production-frame conversion, dependency/version change, workspace lockfile/toolchain change or out-of-scope source edit occurred. The prior ps read and TestCreate substitution were explicitly accepted by the director. Runtime's scoped source-layout invariant maintenance and the explicit native skip allowance are disclosed above. The tree is left uncommitted.

Remaining work after a scope ruling: reconcile the historical retention census from actual discovery; rerun affected final-tree gates and required maintenance; build/hash the four candidates, prove error-size equality and release feature exclusion, then finish the recorded-base allocation, instruction and full timing comparisons. Bases, raw floor samples and the reusable resume harness are preserved. Checkpoint: `.temp/brief-608/PROGRESS.md`.

## Director amendment — redispatch 3

The stop was correct and the omission was mine: this census pins source
sites in the suspension driver, and every brief that has moved that driver
has had to maintain it (brief-595 moved the same three pins and grew the
frontier by its five receivers, `28817e26`).

1. **Scope gains `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`**
   for discovery maintenance of `production_rejection_census_is_complete`
   only:
   - The three guest-form abandonment pins follow their code: same kind
     (`RET002`), same three method sites (take/install the paused
     environment, capture it for cleanup), new file and lines
     (`suspension/core/abandon.rs`). Update the comment to say where they
     live now.
   - The exact total (932) and the kind totals are re-derived from
     discovery on the final tree. Classify every difference before you write
     a number: each removed row must pair with an added row for the same
     operation moved by the split (old site → new site), and each unpaired
     added row must name the S2 code that introduced it (driver, queue,
     entry dispatch, fault latch, abandonment drain). An unpaired removed
     row, or an added row you cannot attribute, is a STOP. Record the
     classified diff under `.temp/brief-608/` and write a comment beside the
     new totals in the style of the brief-595 comment above it.
   - Every unresolved-boundary assertion stays; no row is granted coverage;
     the checker, the regions and `local::verify_promotions` do not change.
     The `ACCEPTANCE_BASELINE` print stays as it is.

2. **Then finish the brief** exactly as redispatch 2's item 4 lists, on the
   final tree: rerun the census and certificate gates touched by this
   change, the workspace run, the candidate release builds and hashes,
   feature trees, error sizes, and the candidate halves of the ordinary
   panel and the resume harness against your recorded bases. Append a new
   `## Execution report` after this section.
