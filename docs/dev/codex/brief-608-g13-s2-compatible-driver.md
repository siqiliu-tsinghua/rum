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


## Execution report — redispatch 3: final census replay and measurement evidence

The director-authorized historical census repair is complete. Both historical census builds, both certificate builds, all mandatory focused gates and the workspace gate pass. Both suspension oracles measure zero slot-fault occurrences. The four candidate binaries, feature trees, error sizes and all required measurements are recorded.
**Performance gate: FAIL; S2 is not ready for acceptance.** All correctness gates, error-size checks and allocation/byte gates pass. One of 59 ordinary labels regresses in both paired samples. All five resume instruction increases exceed their recorded floors; their aggregate sizes are measured, but isolated mechanism attribution has not been established.

`lazy-table-construct-full` (`LazyTable[i,{i,1,5,2}]`, N=20,000) rises from 24044.25195 to 24120.7185 instructions/iteration (+0.318024%, +76.46655) and from 24068.8898 to 24140.9494 (+0.299389%, +72.05960). Both exceed the 0.190008% floor/threshold. Allocation count remains 64 and bytes remain 1449 per iteration. This workload requests no input, so attributing its regression to execution of the resume driver would be false; its cause is unresolved. The same-tier controls pass: `lazy-table-construct-short` changes -0.253230%/+0.105632% against 0.305248%, and `lazy-table-realize-small` changes -0.073816%/+0.007523% against 0.091860%. All other ordinary instruction labels pass the prescribed two-sample rule.

Every allocation and byte slope is exactly equal across all 64 labels. The five measured resume instruction costs are:

| Workload | Extra instructions/resume, pair 1 | Extra instructions/resume, pair 2 | Relative deltas | Floor |
| --- | ---: | ---: | --- | ---: |
| resume-table | 897.316 | 858.658 | 5.716589% / 5.455538% | 0.153556% |
| resume-do-chain | 1098.924 | 1095.988 | 7.258535% / 7.245258% | 0.024326% |
| resume-call-repeat | 915.294 | 1052.245 | 3.729462% / 4.284528% | 0.062048% |
| resume-cleanup | 912.895 | 913.383 | 6.428540% / 6.430483% | 0.276539% |
| resume-fold | 1483.860 | 1478.664 | 6.067386% / 6.046343% | 0.334755% |

Source-diff mechanism assessment: the new work on the legacy resume path is (1) FrameQueue representation checks for front/pop/insertion/splice, (2) Entry::Legacy exact Value decoding and DispatchAction wrapping/matching plus Slot::Value output wrapping, and (3) exact completion decoding, the nested DriverFault result and the first-fault check at the request boundary. These are concrete candidate contributors to the measured aggregate costs, not independently measured per-mechanism instruction counts. The front Arc clone/retain/pop/drop sequence already existed in the base and is **not** an added cost. TypedAdapter allocations and State transport do not run in these workloads, and the probe is absent from every release build.

The whole-change A/B cannot distinguish those driver contributions from compiler/inlining effects also visible on the ordinary label. No unsupported causal attribution is claimed: the above-floor resume overheads remain unresolved performance costs under redispatch 1's attribution rule. The ordinary regression and the five unattributed resume increases are reported as defects under the brief's explicit “fix or report” requirement. No repair or attribution-isolating build was attempted after this complete measured panel, and no additional samples were taken to select better values. The 120-label timing A/B is retained as an indication and does not override the instruction verdict.


### 1. Files changed

Combined S2 source/inventory change: 49 modified files and 10 new files. This redispatch changes only the authorized census test, 68 fingerprints and four emitted certificate cells. The brief receives this appended report; scratch stays under `.temp/brief-608/`.

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

| [crates/rum-stdlib/src/tests/retention_closure/census/tests.rs](/home/remi/dev/rum/crates/rum-stdlib/src/tests/retention_closure/census/tests.rs) | Follow the three unresolved guest-form abandonment operations into core/abandon.rs; record the classified 20-pair discovery replay and unchanged 932/10/895/27 totals; preserve every unresolved-boundary assertion and ACCEPTANCE_BASELINE. |

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

The sole compiler-required exhaustive arm is the legacy error-wire guard in suspension/error.rs; the guard explicitly disappears with the wire at R. Other capture/request/host passthrough arms are in named scope. Runtime EvalError carries opaque SlotMismatch; SessionRequestError retains its named static-label shape. Private fields, public read-only getters, the suspension-restricted production constructor and cfg(test)-only injection constructor remain enforced by the external compiler control.

The four-construct ownership assertion and its original witness loop remain intact. The historical census retains every unresolved-boundary assertion, wrapper check, local::verify_promotions call and the original ACCEPTANCE_BASELINE print. No checker, region or coverage rule changed.

Split assessment: core.rs remains 27,551 characters, below 35k; queue, typed entry, fault state/unwind and abandonment drain have separate modules. Existing large request/error/host files retain the prior assessments: request handling versus conversion, state versus EvalError, and legacy wire removal at R. The 21,012-character compiler control test can later separate fixtures from diagnostic matching; frame-root source checks can separate from inventory checks. The large derive inventory remains existing data debt. This redispatch adds no runtime module or unrelated split.

### 2. Driver, routes, fault path and probe

Legacy-only owners retain VecDeque cells containing one Arc<dyn SuspensionFrame>; a typed insertion widens that owner to private Entry cells. Queue representation does not choose transport. Legacy accepts Value only; typed transport is its type's sealed associated mode, with no dual-arm decoder or per-frame override.

I0001 borrows and matches the actual producer Result once and constructs the frame only for InputRequired. I0002 checks the first input, adjacency and last output. I0003 uses the adapter-minted token and fresh state of the same nominal frame. I0007 requires an Expr-input successor with the parent's output. I0008 releases both frame Arcs before the pure finalizer. I0009 seeds a Value string, places nested owners before the unrun suffix and exactly decodes completion. I0004/I0005/I0006/I0010 remain S5/S18/S14/S24; T0010 observes the unchanged CLI mapping. No production frame was typed and no codec was flipped.

The driver resumes an Arc clone while the queue retains the consumer; it never clones frame state. A decode failure therefore leaves the consumer at the active front. The first private DriverFault is latched, active and nested owners enter the existing accounted abandonment worklist, pending handles are cleared, output rolls back, and structural abandonment restores scopes and loaders. Approved WithCleanup evaluation is the only guest cleanup transition; its nested input owner is abandoned before the saved suffix. Cleanup failures cannot replace the first fault. Only after unwind does the boundary project SlotMismatch; it never enters the frame-completion loop. EvaluationData and TestReport also propagate a direct internal public-variant return.

FaultProbe exists only in runtime tests or the test-continuation-slots feature. Only creation of the first private fault increments it. Retained observers survive reset, rollback and session replacement. Oracle checks are independent of Checks flags, run before lifecycle reset, and retain observations for the final aggregate. Both measured oracle deltas are zero. Release feature-tree and measured error-size evidence is reported with the four candidate binaries below.

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

Below/at/above admission tests use the real pre-existing InputString queue. Widening its four cells and adding a zero-sized typed frame costs exactly 72 bytes: 40 for both Arc layouts and 32 for wider cells. Maintained/independent reconciliation also covers nested Evaluate, finalization, capture/scope faults, cleanup input, reset and nested/public file rollback. The measured legacy allocation/byte slopes are reported below.

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

Commands were bounded, serial and offline, with one Cargo job, one test thread and the brief-608 TMPDIR. Bounds: ordinary gates 900s, workspace 3600s, oracle 1500s per build. Performance samples ran after compilation/testing ended. Original one-codegen-unit bases were retained and rehashed; candidate builds use the same source paths and separate target directories.

Every redispatch-3 test-result total follows verbatim. First certificate attempts earned zero-rejection marks before rejecting stale recorded hashes; replayed gates passed. Census child failures are required deliberate mutations; their parent gates pass. Earlier dispatch totals remain above.

`certificates-default-1.log` (exit 101)
```text
cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.33s
```


`certificates-default-2.log` (exit 0)
```text
cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.27s
```


`certificates-no-default-1.log` (exit 101)
```text
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.23s
```


`certificates-no-default-2.log` (exit 0)
```text
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.21s
```


`compile-default.log` (exit 0)
```text
cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.22s
```


`compile-no-default.log` (exit 0)
```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.58s
```


`oracle-both.log` (exit 0)
```text
scripts/check-suspension-oracle.sh 1500
```

```text
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.87s
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.81s
```


`ownership.log` (exit 0)
```text
cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
```

```text
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 69.86s
```


`retention-census-default.log` (exit 0)
```text
cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 25.25s
```


`retention-census-no-default.log` (exit 0)
```text
cargo test -p rum-stdlib --no-default-features --lib production_rejection_census_is_complete -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 25.55s
```


`runtime-typed.log` (exit 0)
```text
cargo test -p rum-runtime --lib suspension::typed::tests
```

```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.60s
```


`typed-census-default.log` (exit 0)
```text
cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.82s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.72s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.44s
```


`typed-census-no-default.log` (exit 0)
```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.79s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.61s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 137.67s
```


`workspace.log` (exit 0)
```text
cargo test --workspace --no-fail-fast -- --nocapture
```

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.35s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.31s
test result: ok. 1092 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1065.79s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.51s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.81s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.95s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.95s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 134.90s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.98s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.42s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.57s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


`cargo fmt --check`: exit 0, empty output. Oracle observations, default then no-default:
```text
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
```


Native ZeroMQ: **9 skips** from denied loopback binding with the explicit skip allowance. Rust aggregates these early returns as passed; here they are skips.
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


Workspace compiler-observation totals:
```text
ORACLE_TOTAL agreements=8 disagreements=0 techniques=3/3
A1_CORPUS_TOTAL agreements=7322 disagreements=0 claims=7322
A1_ORACLE_TOTAL agreements=287 disagreements=0
A1_ORACLE_TOTAL agreements=287 disagreements=0
METHOD_TOTAL agreements=3512 disagreements=0 coverage=3512/3512
METHOD_TOTAL agreements=3512 disagreements=0 coverage=3512/3512
```


Base error sizes:
```text
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136
```
Candidate error sizes:
```text
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136
```


Both types remain 136 bytes. All four normal release feature trees exclude test-continuation-slots, compiling out its counter, load and branch.

| Build | Binary | Bytes | SHA-256 |
| --- | --- | ---: | --- |

| base | `rum-cli` | 12376160 | `76efb7c611f510a4b46c8d181d77ec37c5fe7ef057c2572ce05978e7a795ecf5` |

| base | `rum-cli-alloc` | 12390248 | `0e9c806bab777893c83e4e7cf1411d977ac63bb77c64374af0164e40a51e56a4` |

| base | `resume` | 12019168 | `be2739b2e4a62ff412500e01d7edb74e362726030796b90c722167ac639eb131` |

| base | `resume-alloc` | 12028296 | `16aaeffd58bcf727704bd829db9ad7bc3219da67a6ccc78e87bb0b44431b0fb9` |

| candidate | `rum-cli` | 12352072 | `52bcad721ef611df152e5aa75548c70dd0da97f6768ae95c2a613e48f9693c14` |

| candidate | `rum-cli-alloc` | 12362248 | `3783e3886acaa04f3a598ae39043ac1cc920143e032a7115d0a85fddaa60341a` |

| candidate | `resume` | 11995144 | `7b1f57222dbefb066773e6993370c56df89533e3351e3e62d3bbc272ba61c681` |

| candidate | `resume-alloc` | 12000312 | `3469dced3dd80821f04ecac1e451ba9ae6ffcd6b9a589222d86f319e881a19d2` |

Feature-tree and build logs: [.temp/brief-608/candidate](/home/remi/dev/rum/.temp/brief-608/candidate). Every metadata record pins codegen_units=1.

Measurement cap: 64 correctness and 128 allocation cells per build; four original 128-cell base instruction samples for two floor pairs; two new paired 128-cell base/candidate samples (base5/candidate1, base6/candidate2); one full timing script per binary. Instruction commands use taskset -c 2 perf stat -x, -e cpu_core/instructions/u, one label per process, stdin /dev/null, and (count(2N)-count(N))/N. Profiling flags are removed except the allocation build's RUM_PROFILE_ALLOC=1. Raw output precedes parsing; each result is checked. No repeat was selected for a better value. The public HostSession harness exercises all five actual input/resume workloads.

All 64 allocation/byte and instruction rows below are verbatim from comparison.tsv, per iteration or per resume. Floors were established before implementation. An ordinary label fails if both deltas exceed max(0.05%, floor); legacy resume allocations and bytes require exact equality.
```text
label	engine	N	base_alloc	candidate_alloc	delta_alloc	base_bytes	candidate_bytes	delta_bytes	base1_instructions	candidate1_instructions	delta1_percent	base2_instructions	candidate2_instructions	delta2_percent	floor_percent	ordinary_threshold_percent	ordinary_regression
lazy-range-take-small	cli	20000	141.0	141.0	0.0	2714.0	2714.0	0.0	42188.3022	42194.19645	0.01397128988993579	42189.5448	42195.45795	0.01401567622507649	0.143667	0.143667	False
lazy-range-normal-small	cli	20000	157.0	157.0	0.0	3388.0	3388.0	0.0	54102.68075	54104.20155	0.002810951285447949	54100.404	54059.685	-0.07526561169488577	0.122231	0.122231	False
lazy-map-realize-small	cli	20000	333.0	333.0	0.0	13740.0	13740.0	0.0	178952.97385	178820.79735	-0.07386102457889177	178825.54185	178817.9419	-0.004249924211818801	0.027087	0.05	False
lazy-select-realize-small	cli	20000	236.0	236.0	0.0	6834.0	6834.0	0.0	114625.56865	114570.68465	-0.04788111469927747	114556.4393	114569.5519	0.011446410241222615	0.006677	0.05	False
lazy-select-scan-small	cli	20000	1471.0	1471.0	0.0	62793.0	62793.0	0.0	805412.4632	805237.2131	-0.021759049928737895	805353.29955	805178.7057	-0.02167916243684953	0.007982	0.05	False
lazy-nestlist-realize-small	cli	20000	197.0	197.0	0.0	11316.0	11316.0	0.0	145305.33815	145287.5433	-0.01224652185980446	145309.28325	145297.2439	-0.008285327496448947	0.033018	0.05	False
lazy-foldlist-realize-small	cli	20000	216.0	216.0	0.0	7604.0	7604.0	0.0	112730.28	112705.89515	-0.02163114471107619	112851.09785	112686.16265	-0.14615294236590692	0.106063	0.106063	False
lazy-drop-realize-small	cli	20000	351.0	351.0	0.0	14204.0	14204.0	0.0	191468.5359	191755.7401	0.15000072917985552	191585.99155	191563.44825	-0.011766674493074447	0.150339	0.150339	False
lazy-first-small	cli	20000	64.0	64.0	0.0	2868.0	2868.0	0.0	41212.9656	41228.93415	0.03874642304313447	41188.89425	41295.9013	0.2597958793224997	0.200265	0.200265	False
lazy-part-small	cli	20000	152.0	152.0	0.0	6182.0	6182.0	0.0	88350.0826	88259.6254	-0.10238496370119643	88369.52175	88258.2171	-0.12595366343035597	0.111397	0.111397	False
lazy-transparent-adapters-small	cli	20000	319.0	319.0	0.0	12876.0	12876.0	0.0	181463.88075	181434.61755	-0.01612618438394664	181412.28925	181499.5699	0.048111762637925715	0.015381	0.05	False
lazy-table-construct-short	cli	20000	63.0	63.0	0.0	1537.0	1537.0	0.0	23558.7744	23499.1165	-0.2532300661616693	23582.0559	23606.9662	0.10563243555028468	0.305248	0.305248	False
lazy-table-construct-full	cli	20000	64.0	64.0	0.0	1449.0	1449.0	0.0	24044.25195	24120.7185	0.3180242419643875	24068.8898	24140.9494	0.29938896475398824	0.190008	0.190008	True
lazy-table-realize-small	cli	20000	251.0	251.0	0.0	9190.0	9190.0	0.0	117294.62045	117208.0377	-0.0738164714356282	117176.25925	117185.0741	0.007522726921327383	0.09186	0.09186	False
lazy-permutations-take-small	cli	20000	29.0	29.0	0.0	1729.0	1729.0	0.0	28175.2999	28191.80745	0.058588728633179876	28197.41365	28191.7782	-0.019985698227320547	0.129913	0.129913	False
lazy-subsets-take-small	cli	20000	29.0	29.0	0.0	1417.0	1417.0	0.0	25410.85925	25329.02355	-0.32205010934449607	25410.70195	25352.88155	-0.227543497671856	0.037505	0.05	False
lazy-tuples-take-small	cli	20000	29.0	29.0	0.0	1361.0	1361.0	0.0	26580.9835	26565.9986	-0.05637451300475327	26566.43705	26586.6955	0.07625580337278226	0.179754	0.179754	False
scan-no-input	cli	20000	33.0	33.0	0.0	2536.0	2536.0	0.0	29712.0678	29687.4029	-0.08301307120738288	29710.84185	29689.69315	-0.07118175952998129	0.003205	0.05	False
optionvalue-three-no-input	cli	20000	9.0	9.0	0.0	1128.0	1128.0	0.0	9590.52795	9588.15115	-0.02478278581107274	9589.0143	9589.8936	0.009169868481673227	0.029575	0.05	False
replacerepeated-no-input	cli	20000	57.0	57.0	0.0	7899.0	7899.0	0.0	39181.10995	39174.6701	-0.01643610915620375	39181.19265	39174.3555	-0.017450081372138992	0.000695	0.05	False
dot-three-no-input	cli	20000	596.0	596.0	0.0	23504.0	23504.0	0.0	254198.68565	254198.32645	-0.00014130678885893033	254199.8653	254198.5378	-0.000522226869970055	0.000109	0.05	False
differences-no-input	cli	20000	13.0	13.0	0.0	704.0	704.0	0.0	10044.87695	10033.7864	-0.11041001353430113	10037.832	10033.785	-0.04031747094392912	0.071642	0.071642	False
do-count-null	cli	20000	0.0	0.0	0.0	0.0	0.0	0.0	537.0217	536.9717	-0.009310610725776769	537.15565	536.89765	-0.048030770969276215	0.053612	0.053612	False
do-iterator-null	cli	20000	0.0	0.0	0.0	32.0	32.0	0.0	1154.17055	1154.1827	0.0010527040392727116	1153.98535	1154.0862	0.008739279055847149	0.018868	0.05	False
table-iterator-null	cli	20000	0.0	0.0	0.0	32.0	32.0	0.0	387.00175	387.12255	0.031214329134154895	387.3327	387.1193	-0.055094754457851725	0.042006	0.05	False
replaceall-simple	cli	20000	5.0	5.0	0.0	1352.0	1352.0	0.0	11406.29695	11432.26135	0.22763215891903332	11406.26015	11406.3662	0.0009297525973117615	0.452949	0.452949	False
table-small	cli	20000	22.0	22.0	0.0	977.0	977.0	0.0	14609.0529	14667.18065	0.397888558538928	14633.0289	14634.324	0.00885052581287571	0.572845	0.572845	False
table-iterator-plus	cli	20000	2.0	2.0	0.0	72.0	72.0	0.0	563.2048	563.258	0.009445942222097159	563.2543	563.2033	-0.009054524750173876	0.029983	0.05	False
table-iterator-pair-list	cli	20000	2.0	2.0	0.0	136.0	136.0	0.0	1640.69465	1640.74715	0.003199864155090104	1640.82305	1640.7898	-0.002026422044709708	0.001688	0.05	False
take-multispec-small	cli	20000	109.0	109.0	0.0	5856.0	5856.0	0.0	49923.1386	49922.28215	-0.0017155371717758605	49938.8589	49922.27165	-0.03321511617478956	0.046752	0.05	False
lazy-depth-map-build-8	cli	1000	16.001	16.001	0.0	1879.864	1879.864	0.0	18108.939	18087.783	-0.11682628120840421	18094.319	18089.043	-0.029158323117872698	0.087984	0.087984	False
lazy-depth-map-first-8	cli	1000	188.0	188.0	0.0	11176.0	11176.0	0.0	130386.833	130406.483	0.015070540136519206	130390.634	130403.221	0.009653300711764956	0.026366	0.05	False
lazy-depth-map-eight-8	cli	1000	1464.001	1464.001	0.0	82776.864	82776.864	0.0	959556.737	959638.558	0.008526958005194984	959506.614	959599.477	0.009678203218732584	0.014316	0.05	False
lazy-depth-select-restart-8	cli	1000	327.0	327.0	0.0	16980.0	16980.0	0.0	179974.928	179405.912	-0.31616403813757854	179980.251	179358.379	-0.3455223540053898	0.165894	0.165894	False
lazy-depth-drop-all-one-8	cli	1000	170.0	170.0	0.0	3128.0	3128.0	0.0	46663.866	46663.588	-0.0005957500392228532	46665.343	46668.966	0.007763791642978468	0.002949	0.05	False
lazy-depth-take-map-8	cli	1000	220.0	220.0	0.0	12144.0	12144.0	0.0	138957.74	138958.547	0.0005807521049172593	138952.346	138951.586	-0.0005469501032973056	0.003719	0.05	False
lazy-depth-fold-through-source-8	cli	1000	743.001	743.001	0.0	58440.864	58440.864	0.0	589523.989	589512.639	-0.001925282127912542	589543.056	589562.521	0.003301709654945917	0.001805	0.05	False
lazy-depth-nest-map-8	cli	1000	192.0	192.0	0.0	11964.0	11964.0	0.0	141004.616	141009.18	0.003236773468451304	141007.749	141004.792	-0.0020970478721848984	0.006765	0.05	False
lazy-depth-map-build-64	cli	1000	16.001	16.001	0.0	5911.864	5911.864	0.0	30118.544	30123.931	0.01788599077032771	30125.248	30118.835	-0.021287791556112712	0.116452	0.116452	False
lazy-depth-map-first-64	cli	1000	1364.0	1364.0	0.0	85320.0	85320.0	0.0	944593.921	946025.854	0.15159244286520046	944595.976	946262.059	0.17638048883663604	0.335237	0.335237	False
lazy-depth-map-eight-64	cli	1000	10872.001	10872.001	0.0	647704.864	647704.864	0.0	7421647.034	7421975.638	0.004427642523219255	7421576.208	7421236.982	-0.004570808012915872	0.02104	0.05	False
lazy-depth-select-restart-64	cli	1000	1671.0	1671.0	0.0	99412.0	99412.0	0.0	978790.422	975906.278	-0.2946641012390283	975473.913	972791.266	-0.2750096096111543	0.656111	0.656111	False
lazy-depth-drop-all-one-64	cli	1000	1178.0	1178.0	0.0	17520.0	17520.0	0.0	327583.97	327580.5	-0.0010592703910350387	327581.675	327581.737	1.892657763846728e-05	0.002126	0.05	False
lazy-depth-take-map-64	cli	1000	1620.0	1620.0	0.0	93064.0	93064.0	0.0	1007488.478	1007424.98	-0.006302603095376114	1007451.083	1007417.654	-0.003318175995248307	0.001015	0.05	False
lazy-depth-fold-through-source-64	cli	1000	43334.001	43334.001	0.0	3402649.864	3402649.864	0.0	33996283.812	33998047.448	0.005187731723133027	33997826.679	33998174.924	0.0010243154754840944	0.003458	0.05	False
lazy-depth-nest-map-64	cli	1000	1368.0	1368.0	0.0	86108.0	86108.0	0.0	961945.766	961947.378	0.00016757701495340172	956952.732	958895.511	0.20301723742819355	0.17411	0.17411	False
lazy-depth-map-build-256	cli	1000	16.001	16.001	0.0	19735.864	19735.864	0.0	70760.021	70774.939	0.02108252624741791	70753.992	70761.423	0.010502587613703263	0.009013	0.05	False
lazy-depth-map-first-256	cli	1000	5396.0	5396.0	0.0	339528.0	339528.0	0.0	3722820.69	3722646.596	-0.004676400355985155	3716199.951	3722736.187	0.17588493854430176	0.004742	0.05	False
lazy-depth-map-eight-256	cli	1000	43128.001	43128.001	0.0	2584600.864	2584600.864	0.0	29547872.595	29546669.717	-0.004070946211542914	29544342.877	29546631.566	0.007746623472137948	0.005175	0.05	False
lazy-depth-select-restart-256	cli	1000	6279.0	6279.0	0.0	382036.0	382036.0	0.0	3664241.593	3665003.756	0.020800020431410715	3652103.403	3665566.231	0.36863216931211085	0.705262	0.705262	False
lazy-depth-drop-all-one-256	cli	1000	4634.0	4634.0	0.0	66864.0	66864.0	0.0	2218609.649	2218611.935	0.00010303750372919751	2218612.661	2218609.024	-0.00016393127396652218	0.000437	0.05	False
lazy-depth-take-map-256	cli	1000	6420.0	6420.0	0.0	370504.0	370504.0	0.0	3985656.874	3985379.846	-0.006950623416857038	3985393.453	3986019.177	0.01570043227547835	0.003231	0.05	False
lazy-depth-fold-through-source-256	cli	1000	689320.001	689320.001	0.0	54008857.864	54008857.864	0.0	530593326.385	530622023.652	0.005408523924632647	530585109.821	530582289.014	-0.0005316408145938922	0.004122	0.05	False
lazy-depth-nest-map-256	cli	1000	5400.0	5400.0	0.0	340316.0	340316.0	0.0	3726899.526	3726892.786	-0.00018084737603007994	3746883.387	3733561.809	-0.3555375661335014	0.000294	0.05	False
literal-integer	cli	20000	0.0	0.0	0.0	0.0	0.0	0.0	293.98495	294.1335	0.050529797528753306	294.04335	293.90475	-0.04713590700147208	0.035308	0.05	False
downvalue-pattern-hit	cli	20000	11.0	11.0	0.0	908.0	908.0	0.0	13949.14855	13949.631	0.003458634039699149	13949.3166	13949.25975	-0.00040754684713650846	0.001312	0.05	False
module-local	cli	20000	23.0001	23.0001	0.0	2163.6272	2163.6272	0.0	22654.8604	22653.20235	-0.00731873854319387	22653.164	22652.99515	-0.0007453704921833726	0.002678	0.05	False
block-local	cli	20000	8.0	8.0	0.0	752.0	752.0	0.0	14671.86535	14671.81185	-0.0003646434773241225	14672.01125	14671.6943	-0.0021602355300798237	0.002024	0.05	False
matcher-single-wide-small	cli	1000	11.0	11.0	0.0	287280.0	287280.0	0.0	760207.472	760190.812	-0.0021915070048006413	760216.268	760208.82	-0.0009797212074502681	0.003397	0.05	False
resume-table	resume	1000	27.003	27.003	0.0	1742.456	1742.456	0.0	15696.703	16594.019	5.716589018725782	15739.198	16597.856	5.4555384588210964	0.153556	0.153556	False
resume-do-chain	resume	1000	24.0	24.0	0.0	1634.0	1634.0	0.0	15139.749	16238.673	7.2585351315930025	15126.969	16222.957	7.2452584519741015	0.024326	0.05	False
resume-call-repeat	resume	1000	40.007	40.007	0.0	4210.024	4210.024	0.0	24542.254	25457.548	3.7294618497551113	24559.183	25611.428	4.284527705990859	0.062048	0.062048	False
resume-cleanup	resume	1000	22.0	22.0	0.0	1394.0	1394.0	0.0	14200.658	15113.553	6.428540142294814	14203.956	15117.339	6.430483169618384	0.276539	0.276539	False
resume-fold	resume	1000	41.0	41.0	0.0	2873.0	2873.0	0.0	24456.332	25940.192	6.067385738793529	24455.51	25934.174	6.046342930488868	0.334755	0.334755	False
```


**Performance gate: FAIL; S2 is not ready for acceptance.** All correctness gates, error-size checks and allocation/byte gates pass. One of 59 ordinary labels regresses in both paired samples. All five resume instruction increases exceed their recorded floors; their aggregate sizes are measured, but isolated mechanism attribution has not been established.

`lazy-table-construct-full` (`LazyTable[i,{i,1,5,2}]`, N=20,000) rises from 24044.25195 to 24120.7185 instructions/iteration (+0.318024%, +76.46655) and from 24068.8898 to 24140.9494 (+0.299389%, +72.05960). Both exceed the 0.190008% floor/threshold. Allocation count remains 64 and bytes remain 1449 per iteration. This workload requests no input, so attributing its regression to execution of the resume driver would be false; its cause is unresolved. The same-tier controls pass: `lazy-table-construct-short` changes -0.253230%/+0.105632% against 0.305248%, and `lazy-table-realize-small` changes -0.073816%/+0.007523% against 0.091860%. All other ordinary instruction labels pass the prescribed two-sample rule.

Every allocation and byte slope is exactly equal across all 64 labels. The five measured resume instruction costs are:

| Workload | Extra instructions/resume, pair 1 | Extra instructions/resume, pair 2 | Relative deltas | Floor |
| --- | ---: | ---: | --- | ---: |
| resume-table | 897.316 | 858.658 | 5.716589% / 5.455538% | 0.153556% |
| resume-do-chain | 1098.924 | 1095.988 | 7.258535% / 7.245258% | 0.024326% |
| resume-call-repeat | 915.294 | 1052.245 | 3.729462% / 4.284528% | 0.062048% |
| resume-cleanup | 912.895 | 913.383 | 6.428540% / 6.430483% | 0.276539% |
| resume-fold | 1483.860 | 1478.664 | 6.067386% / 6.046343% | 0.334755% |

Source-diff mechanism assessment: the new work on the legacy resume path is (1) FrameQueue representation checks for front/pop/insertion/splice, (2) Entry::Legacy exact Value decoding and DispatchAction wrapping/matching plus Slot::Value output wrapping, and (3) exact completion decoding, the nested DriverFault result and the first-fault check at the request boundary. These are concrete candidate contributors to the measured aggregate costs, not independently measured per-mechanism instruction counts. The front Arc clone/retain/pop/drop sequence already existed in the base and is **not** an added cost. TypedAdapter allocations and State transport do not run in these workloads, and the probe is absent from every release build.

The whole-change A/B cannot distinguish those driver contributions from compiler/inlining effects also visible on the ordinary label. No unsupported causal attribution is claimed: the above-floor resume overheads remain unresolved performance costs under redispatch 1's attribution rule. The ordinary regression and the five unattributed resume increases are reported as defects under the brief's explicit “fix or report” requirement. No repair or attribution-isolating build was attempted after this complete measured panel, and no additional samples were taken to select better values. The 120-label timing A/B is retained as an indication and does not override the instruction verdict.


Full core-language-microbench panels: profiling off, one-codegen-unit binaries; --scale 1 --reps 7 and RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES=3 for both arms. Values come from in-kernel AbsoluteTiming/Instant and are indications only; wall-clock durations do not decide the gate.

All 120 paired timing rows; seconds preserve the script output, percentage is derived:
```text
label	iterations	base_seconds	candidate_seconds	delta_percent_indication_only
association-part-key-read-width4000	40	0.00247186	0.00248348	0.4700913482155178
association-part-key-read-width32000	40	0.00982671	0.00938617	-4.483087421934718
association-part-positional-read-width4000	40	3.0273e-05	3.0629e-05	1.175965381693267
association-part-positional-read-width32000	40	3.0604e-05	3.0361e-05	-0.7940138543981234
loop-empty	5000	0.000274081	0.000261163	-4.713205220354566
do-count-null	10000	0.000247089	0.000250679	1.452917774567064
do-iterator-null	1000	5.7185e-05	5.7349e-05	0.2867884934860454
table-count-null	5000	8.3572e-05	7.5643e-05	-9.487627435026102
table-iterator-null	1000	1.918e-05	1.7555e-05	-8.472367049009378
table-iterator-symbol	1000	6.66e-06	6.671e-06	0.16516516516518198
literal-integer	2000	9.2426e-05	8.9874e-05	-2.761127821175857
literal-string	2000	0.000101349	0.000101858	0.5022249849529814
symbol-ownvalue-read	2000	0.000227094	0.000216395	-4.711264938747828
symbol-ownvalue-guarded-read	1000	0.000187281	0.000187958	0.36148888568514614
set-ownvalue	1000	0.000272791	0.000261094	-4.287898061153028
list-literal-small	1000	6.5671e-05	5.9937e-05	-8.731403511443414
head-list	2000	0.000962521	0.000932119	-3.1585804361671133
length-list	2000	0.000956172	0.000936333	-2.0748359081838808
part-list	2000	0.00113902	0.00108077	-5.114045407455525
sameq-integer	2000	0.000650668	0.000633616	-2.6206913510423036
sameq-list	1000	0.000639856	0.00061827	-3.3735715535995525
if-true	2000	0.000725816	0.000702174	-3.2572993706393993
module-local	500	0.000714093	0.000714308	0.030108123171634915
module-chain-escape-small	96	4.7013e-05	5.0096e-05	6.5577606194031635
module-chain-escape-large	192	9.4956e-05	9.3818e-05	-1.1984498083322803
block-local	500	0.000469881	0.000466419	-0.7367822916866218
block-symbol-only-repeat	1000	0.000305929	0.00030432	-0.5259390250679008
block-init-null-repeat	1000	0.000357891	0.000352465	-1.5161040652042046
block-init-read-repeat	1000	0.000404738	0.000403795	-0.23299023071716718
with-local	500	0.000482897	0.000476282	-1.3698573401781267
with-deep-body-small	500	0.00183158	0.00178509	-2.538245667674899
table-small	200	0.000193959	0.000185705	-4.25553854165055
table-iterator-plus	1000	3.7294e-05	3.6068e-05	-3.287392073792028
table-iterator-pair-list	1000	6.4225e-05	5.9211e-05	-7.806928766056831
do-small	200	0.000891428	0.00088215	-1.0408019492320286
d-nested-sin-small	200	0.00540624	0.00520247	-3.769163041226431
downvalue-exact-hit	2000	0.000805789	0.000786497	-2.394175150070299
downvalue-exact-miss	2000	0.000580566	0.000567305	-2.284150294712395
downvalue-pattern-hit	1000	0.00097393	0.000966175	-0.7962584580000653
downvalue-pattern-miss	1000	0.000301297	0.000293029	-2.744136184562085
downvalue-exact-miss-repeat	5000	0.00122953	0.00125481	2.056070205688365
downvalue-pattern-miss-repeat	5000	0.00126941	0.00128966	1.595229279744137
downvalue-guarded-hit	1000	0.00189333	0.00194394	2.6730680863874756
downvalue-guarded-decline-fallback	1000	0.00142169	0.00144214	1.438428912069445
upvalue-guarded-hit	1000	0.00234827	0.00234673	-0.06558019307830731
subvalue-unguarded-hit	1000	0.001399	0.00140712	0.5804145818441686
subvalue-unguarded-miss	1000	0.000776497	0.000765778	-1.3804303171808696
subvalue-guarded-hit	1000	0.00248201	0.00243754	-1.791693022993457
default-unguarded-hit	1000	0.00129416	0.00129934	0.400259627866717
default-unguarded-miss	1000	0.00108016	0.00106832	-1.0961339060879816
default-guarded-hit	1000	0.0016303	0.001637	0.4109673066306829
matchq-integer-success	1000	0.000304564	0.000286782	-5.838510132517294
matchq-integer-fail	1000	0.000311865	0.000291215	-6.621454796145764
matcher-single-tiny	1000	0.000686529	0.000673987	-1.8268711154226458
matcher-single-wide-small	800	4.2786e-05	3.9413e-05	-7.883419810218307
matcher-single-wide-large	1600	8.3397e-05	7.7723e-05	-6.803602048035307
replaceall-simple	500	0.000377072	0.000368279	-2.3319153901642076
rule-rhs-deep-small	500	0.00299923	0.00296783	-1.0469353800808823
rewrite-guarded-hit	500	0.00178873	0.00178356	-0.2890318829560701
rewrite-guarded-decline-fallback	500	0.00254937	0.00254727	-0.08237329222514633
cases-wide-no-match	1600	0.00014815	0.000144892	-2.1991225109686097
count-wide-no-match	1600	0.000140958	0.000137344	-2.563884277586237
position-wide-no-match	1600	0.000176033	0.000169698	-3.5987570512347133
deletecases-wide-no-match	1600	0.000170318	0.000166997	-1.9498819854624871
deletecases-wide-association-no-match	1600	0.000434372	0.000415784	-4.27928135331006
deletecases-leaf-delete-small	500	0.0022028	0.00208123	-5.518885055384059
replace-wide-no-match	1600	8.2441e-05	7.883e-05	-4.380102133647101
replace-leaf-rebuild-small	500	0.0026852	0.00253726	-5.509459258155824
take-multispec-small	500	0.00166845	0.00162465	-2.625191045581221
replacepart-deep-path-small	500	0.00133265	0.00131176	-1.5675533711026923
replacepart-wide-rule-list	1600	0.000291778	0.000297336	1.9048728828081396
replacepart-guarded-hit	500	0.00054272	0.0005396	-0.5748820754717054
replacepart-guarded-decline-fallback	500	0.000595459	0.000590724	-0.7951848909832471
string-repeat-small	500	0.000194256	0.000188417	-3.005827361831803
character-range-small	200	0.000140356	0.000132322	-5.724016073413319
stringtrim-literal-list-4000	100	0.000689972	0.000656573	-4.8406312140202745
stringtrim-literal-list-32000	100	0.00546318	0.00504112	-7.725537141371874
stringtrim-single-literal-4000	100	0.000665572	0.000637057	-4.28428479563443
stringtrim-single-literal-32000	100	0.00546889	0.00502139	-8.18264766707687
stringtrim-whitespace-4000	100	0.000652589	0.000631194	-3.278480023414432
stringtrim-whitespace-32000	100	0.00543876	0.00501581	-7.776588781266325
stringreplace-single-literal-unguarded	200	0.000390915	0.000382696	-2.102503101697284
stringreplace-pattern-unguarded	500	0.00106044	0.00105476	-0.5356267209837284
stringreplace-guarded-hit	500	0.000707917	0.000685905	-3.1094040685560542
stringreplace-guarded-decline-fallback	500	0.000843761	0.000816653	-3.212758115153458
appendto-list-incremental-small	250	0.000604331	0.000450707	-25.42050631193833
appendto-list-incremental-large	500	0.00187469	0.00167527	-10.637492065354802
appendto-part-outer1000-small	100	0.00236969	0.00218716	-7.702695289257234
appendto-part-outer1000-large	200	0.00479269	0.00441496	-7.881377681427337
association-set-key-incremental-small	100	0.00147771	0.00140994	-4.586150191850901
association-set-key-incremental-large	200	0.00560581	0.00533418	-4.845508499217777
association-small-literal-repeat	500	0.000747157	0.00074441	-0.3676603444791371
merge-nested-small	200	0.00117206	0.00116834	-0.317389894715292
set-part-fixed100-len100	100	0.00013596	0.0001431	5.251544571932931
set-part-fixed100-len800	100	0.000651598	0.000597061	-8.369731030481987
set-part-fill-small	200	0.000436018	0.000427919	-1.8574921218848628
set-part-fill-large	400	0.00149582	0.0013661	-8.672166437138152
upvalue-candidates-tiny	1000	0.00192373	0.00192883	0.2651099686546532
upvalue-candidates-wide-small	1600	0.000155278	0.000148882	-4.119063872538298
upvalue-candidates-wide-large	3200	0.000308466	0.000302188	-2.0352324081098105
sort-callback-small	100	0.000169203	0.000163063	-3.628777267542549
sort-callback-large	200	0.000387263	0.000373667	-3.5107924072271257
sortby-callback-small	100	0.000196752	0.000195375	-0.6998658209319331
sortby-callback-large	200	0.000442056	0.000441762	-0.06650741082577794
reversesort-callback-small	100	0.000161584	0.000165082	2.1648182988414755
reversesort-callback-large	200	0.000364165	0.000368904	1.301333186879572
keysort-callback-small	50	8.293e-05	8.3191e-05	0.3147232605812178
keysort-callback-large	100	0.000182739	0.000185082	1.2821565183129824
keysortby-callback-small	25	4.4993e-05	4.5844e-05	1.8914053297179434
keysortby-callback-large	50	9.7278e-05	0.000100521	3.3337445259976617
sort-default-wide	1600	1.9016e-05	2.9358e-05	54.38578039545647
sort-symbol-default-width4000	4000	0.000316602	0.000293111	-7.419725712408642
sort-symbol-default-width16000	16000	0.00130797	0.00122687	-6.200448022508176
sortby-default-wide	1600	0.000503065	0.000505823	0.5482392931330793
reversesort-default-wide	1600	1.9505e-05	1.6729e-05	-14.23224814150219
keysort-default-wide	1600	0.000386042	0.00037824	-2.0210236191916975
keysortby-default-wide	1600	0.000890348	0.000868622	-2.440169461828412
valueq-undefined-symbol-definitions-2000	20	8.185e-06	8e-06	-2.2602321319486918
valueq-undefined-symbol-definitions-8000	20	8.113e-06	8.064e-06	-0.6039689387403069
module-retained-gensym-gc-batch	40	7.0064e-05	6.6547e-05	-5.0196962776889675
```


Historical frontier: original and final discovery both produce 932 rows (RET001=10, RET002=895, RET003=27). All 18 checker modules used by the scratch enumerator are byte-identical to base. The full diagnostic diff pairs 69 rows: 19 moved sites, one driver line shift, and 49 unchanged sites whose witness gains exactly drain_abandonment_worklist. The other 863 full records are identical. There are no unmatched additions/removals or coverage grants. The capture at old core.rs:684/new abandon.rs:59 stays RET002; its reason reflects the new module's opaque Arc import rather than the old module's unexpanded-item scope.

Every classified row follows. Complete reasons, operations and old/new witnesses: [.temp/brief-608/redispatch-3/frontier-classified-diff.json](/home/remi/dev/rum/.temp/brief-608/redispatch-3/frontier-classified-diff.json).
```text
kind	old_site	new_site	operation	classification
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:459:23-459:66	crates/rum-runtime/src/runtime/suspension/core.rs:473:23-473:66	self.suspension_access().take_for_abandon()	Driver source shift: unchanged suspension take_for_abandon receiver
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:647:9-647:48	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:18:9-18:48	self.abandonment_worklist.push(pending)	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET001	crates/rum-runtime/src/runtime/suspension/core.rs:657:67-657:77	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:32:67-32:77	Vec::new()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:658:27-658:75	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:33:27-33:75	std::mem::replace(&mut self.evaluation_depth, 0)	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:660:42-660:73	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:35:42-35:73	self.abandonment_worklist.pop()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET003	crates/rum-runtime/src/runtime/suspension/core.rs:664:13-664:60	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:39:13-39:60	debug_assert!(self.evaluation_stack.is_empty())	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:665:53-665:97	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:40:53-40:97	std::mem::take(&mut suspension.active_stack)	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:666:37-666:64	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:41:37-41:64	self.evaluation_stack.len()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:667:40-667:80	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:42:40-42:80	suspension.guest_form_environment.take()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:668:17-668:42	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:43:17-43:42	environment.install(self)	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:671:37-671:60	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:46:37-46:60	suspension.take_front()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:672:46-672:69	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:47:46-47:69	frame.output_boundary()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:673:30-676:19	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:48:30-51:19	frame.abandon(&mut AbandonmentContext { session: self, reason, })	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:681:28-681:50	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:56:28-56:50	!suspension.is_empty()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:681:29-681:50	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:56:29-56:50	suspension.is_empty()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:682:55-682:84	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:57:55-57:84	self.evaluation_stack.clone()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:684:33-684:70	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:59:33-59:70	self.capture_guest_form_environment()	Abandonment drain split: same unresolved operation in extracted core/abandon.rs; same rejected capture method, diagnostic now names opaque Arc import in the new lexical module instead of the unexpanded item in core.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:685:29-685:71	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:60:29-60:71	self.abandonment_worklist.push(suspension)	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET003	crates/rum-runtime/src/runtime/suspension/core.rs:690:28-690:91	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:65:28-65:91	matches!(cleanup_outcome, Err(EvalError::InputRequired { .. }))	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/suspension/core.rs:694:33-694:71	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:69:33-69:71	self.abandonment_worklist.push(nested)	Abandonment drain split: same unresolved operation in extracted core/abandon.rs
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:14:5-15:20	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:14:5-15:20	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:14:5-16:56	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:14:5-16:56	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:14:5-21:11	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:14:5-21:11	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:16:22-16:55	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:16:22-16:55	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:17:19-21:10	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:17:19-21:10	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:202:12-202:47	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:202:12-202:47	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:205:9-205:63	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:205:9-205:63	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:209:9-209:38	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:209:9-209:38	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:210:12-210:47	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:210:12-210:47	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:212:17-213:32	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:212:17-213:32	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:212:17-214:68	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:212:17-214:68	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:214:34-214:67	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:214:34-214:67	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:220:21-220:51	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:220:21-220:51	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:220:21-220:73	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:220:21-220:73	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:221:24-221:83	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:221:24-221:83	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:222:29-224:81	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:222:29-224:81	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:224:23-224:80	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:224:23-224:80	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:63:12-63:47	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:63:12-63:47	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:64:25-64:55	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:64:25-64:55	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:64:25-64:77	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:64:25-64:77	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:65:30-65:63	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:65:30-65:63	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:65:48-65:62	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:65:48-65:62	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:66:13-66:19	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:66:13-66:19	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:66:22-66:70	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:66:22-66:70	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:66:37-66:69	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:66:37-66:69	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:67:33-70:15	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:67:33-70:15	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:67:60-70:14	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:67:60-70:14	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:75:12-75:47	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:75:12-75:47	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:76:25-76:55	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:76:25-76:55	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:76:25-76:77	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:76:25-76:77	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:77:30-77:63	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:77:30-77:63	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:77:48-77:62	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:77:48-77:62	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:78:13-78:19	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:78:13-78:19	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:78:22-78:70	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:78:22-78:70	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:78:37-78:69	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:78:37-78:69	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:79:33-82:15	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:79:33-82:15	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:79:60-82:14	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:79:60-82:14	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:89:12-89:47	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:89:12-89:47	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:89:12-89:69	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:89:12-89:69	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:90:25-92:26	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:90:25-92:26	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:90:25-93:39	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:90:25-93:39	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:90:25-94:45	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:90:25-94:45	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:94:30-94:44	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:94:30-94:44	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:96:31-96:68	crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:96:31-96:68	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/suspension/output.rs:73:13-73:45	crates/rum-runtime/src/runtime/suspension/output.rs:73:13-73:45	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/suspension/output.rs:77:13-77:47	crates/rum-runtime/src/runtime/suspension/output.rs:77:13-77:47	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/suspension/output.rs:81:13-81:51	crates/rum-runtime/src/runtime/suspension/output.rs:81:13-81:51	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET002	crates/rum-runtime/src/runtime/suspension/output.rs:83:13-83:57	crates/rum-runtime/src/runtime/suspension/output.rs:83:13-83:57	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
RET003	crates/rum-runtime/src/runtime/debug_tools/resources/maintained/owners.rs:116:9-171:10	crates/rum-runtime/src/runtime/debug_tools/resources/maintained/owners.rs:116:9-171:10	Same operation; identical diagnostic fields except witness	Unchanged rejected operation/site; witness path gains the extracted drain_abandonment_worklist helper
```


Ownership stays at 79 host payloads, 7712 UnknownConstruct rows and 59 self-reaching types; no ownership row changes in this redispatch. All S2 transitions, preserved ledger rows 56–58 and unresolved external coverage remain in the prior reports and exact patches: [.temp/brief-608/redispatch-1/ownership-replay.patch](/home/remi/dev/rum/.temp/brief-608/redispatch-1/ownership-replay.patch), [.temp/brief-608/redispatch-2/ownership-replay.patch](/home/remi/dev/rum/.temp/brief-608/redispatch-2/ownership-replay.patch). Actual final evidence:
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


All 68 fingerprint transitions; first 18 meaning/strength fields unchanged:
```text
id	before	after
B01	fnv1a64:2762acca6234859c	fnv1a64:42f7239388cb4603
B02	fnv1a64:05dedfbde4406b55	fnv1a64:2ee65d264197863a
B03	fnv1a64:c613714abc98e842	fnv1a64:9f215e803dc66919
B04	fnv1a64:deefdade6d385a14	fnv1a64:d7146fef7f126ddb
B05	fnv1a64:a632ca753bb166f9	fnv1a64:dbeabca2c03c1f2e
B06	fnv1a64:465d5b4a51ea8e98	fnv1a64:5cce1318facc713f
B07	fnv1a64:bec4f1db53e5b049	fnv1a64:ac09184eb31d2fde
B08	fnv1a64:8596ba743f3d80e6	fnv1a64:41992dc7582920cd
B09	fnv1a64:fab1ed723a57fcd0	fnv1a64:37b6255b7f8a40f7
B10	fnv1a64:2740ee773bb1c57b	fnv1a64:a216cd95b0c43af8
B11	fnv1a64:a0478c3b872c8014	fnv1a64:986c214c990693db
B12	fnv1a64:b9be6d6e166091d0	fnv1a64:f6c2a5575b92d5f7
I01	fnv1a64:054497c7ceec7c1c	fnv1a64:20d90e90f5833c83
I02	fnv1a64:fd74480933e7eef9	fnv1a64:332c3a36b872a72e
I03	fnv1a64:49509a594e34e490	fnv1a64:84fd3cbe91c4b537
I04	fnv1a64:e8465b54c60dbf75	fnv1a64:a6ecc54cb7b229da
P01	fnv1a64:3d980a7408b95238	fnv1a64:1a8e0cebde1df2df
P02	fnv1a64:52a9ab73419a3c6b	fnv1a64:369169bb4a54c048
P03	fnv1a64:a2502fee344e972a	fnv1a64:1059b45489a8e3a1
P04	fnv1a64:fab9fdd368e98755	fnv1a64:23c17b3bc640a23a
P05	fnv1a64:d1103a6554610c8c	fnv1a64:c6a602bf18fcaf93
P06	fnv1a64:0a3d923ea2902b50	fnv1a64:4741ca27e7c26f77
P070	fnv1a64:448b2e808105a550	fnv1a64:818f6669c637e977
P080	fnv1a64:3ec096221ca40b3b	fnv1a64:d786d489eaaf6f38
P090	fnv1a64:8985e4762056ff15	fnv1a64:a17e07c1907e1f7a
P100	fnv1a64:c1eaea7d774c66e4	fnv1a64:54503541883b8b4b
P110	fnv1a64:1bebf6c452e014e3	fnv1a64:e2d591ffd6983320
P120	fnv1a64:02fba299a1c7a486	fnv1a64:3b3211bcfe134a6d
P130	fnv1a64:62924b3882998a49	fnv1a64:4fd671abe1d109de
P140	fnv1a64:acf9a84971a3d15d	fnv1a64:0a7cbde27abe3c02
P150	fnv1a64:4475b95bb2c62435	fnv1a64:bc76fc29ad11cf1a
P071	fnv1a64:9338971a420e0369	fnv1a64:186ddb5dabc41c7e
P081	fnv1a64:e97255fbe9b45af0	fnv1a64:3a6720d184b0bd97
P091	fnv1a64:6f840adc2356c1f8	fnv1a64:16afdac7d89f3c1f
P101	fnv1a64:13eee7918843b1e3	fnv1a64:dad882cd0bfbd020
P111	fnv1a64:ee9a9e79b957bb20	fnv1a64:ddd838f974322c27
P121	fnv1a64:219326fb63c78347	fnv1a64:4584b26a9d150ad4
P131	fnv1a64:7c99fd44a38c7b94	fnv1a64:74be9255b5668f5b
P141	fnv1a64:128a302b25614858	fnv1a64:1eb78ddce6d9827f
P151	fnv1a64:f5dbd1ca1e6e9ac6	fnv1a64:1160e927c7038c2d
B13	fnv1a64:d46deed382874a98	fnv1a64:eadea6a22b692d3f
R01	fnv1a64:5773a351b8ebc6e5	fnv1a64:e46484de80c7b26a
R02	fnv1a64:36713fc1d1264aea	fnv1a64:84308dbd85c66ce1
R03	fnv1a64:5c2ff0785c7b51e1	fnv1a64:3b2d4ac703cbc756
R040	fnv1a64:07474a8f04dd5885	fnv1a64:85026123e7b65b0a
R050	fnv1a64:bbbf3e0544aa18ec	fnv1a64:50770bd48ed18473
R060	fnv1a64:b176295276086b2d	fnv1a64:c0a14656c58fedf2
R070	fnv1a64:51e01e15cb5619f0	fnv1a64:a2d4e8eb66527c97
R080	fnv1a64:af4ac44795928875	fnv1a64:6df12e3f8736f2da
R090	fnv1a64:f3970a120e1b32a4	fnv1a64:58da0904542af28b
R100	fnv1a64:8b39159d666e92c2	fnv1a64:644702d2e79c1399
R110	fnv1a64:aa62ff0e9a0c09f3	fnv1a64:cd8844dd94a985d0
R041	fnv1a64:baa751fd99af0998	fnv1a64:d11809cc4290ec3f
R051	fnv1a64:17a8128bad850b95	fnv1a64:2fa035d71dac2bfa
R061	fnv1a64:e9853b5a0f617c96	fnv1a64:a9b63009272b965d
R071	fnv1a64:95acc10af451045f	fnv1a64:3167576eaa6989cc
R081	fnv1a64:7dd553d212f83be4	fnv1a64:103a9e9623e7604b
R091	fnv1a64:7498f0c7dbf5d39d	fnv1a64:6cf9afc74dc8dec2
R101	fnv1a64:c8320d48fb6d6ded	fnv1a64:f2618a09218d5232
R111	fnv1a64:3d4902991701d7de	fnv1a64:110fb0ebea5be7e5
R12	fnv1a64:d12c6648cf30221e	fnv1a64:1cb64782f03eeaa5
R13	fnv1a64:f12439cbd985d432	fnv1a64:23019450d77105e9
R14	fnv1a64:dc42f8f353fd26e4	fnv1a64:6ea843b764ec4b4b
R15	fnv1a64:f5bd7211e095a408	fnv1a64:30f9638b54728acf
O010	fnv1a64:1de605b4d5860476	fnv1a64:6d30ddbc4f8fc3bd
O0200	fnv1a64:a7c14a1dc05af75d	fnv1a64:05445fb6c9756202
O011	fnv1a64:f3fee5da839a2fad	fnv1a64:032a02ded321b272
O0210	fnv1a64:66ad5f3f573896fe	fnv1a64:9b881fd7ef36d885
```


Both certificate builds earn three production instance marks and four LeaseProbe marks, with zero rejections. Only four actual emitted cells are replayed; all other columns are unchanged.
```text
[
  {
    "body": "C01",
    "hash": "true",
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8f5613125b29d0a3\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:066a0073ea944c9f;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:66ad5f3f573896fe;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:f3fee5da839a2fad;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8f5613125b29d0a3\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:ed2095dc78098ecd;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:a7c14a1dc05af75d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:1de605b4d5860476;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8f5613125b29d0a3\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:066a0073ea944c9f;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:9b881fd7ef36d885;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:032a02ded321b272;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8f5613125b29d0a3\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:ed2095dc78098ecd;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:05445fb6c9756202;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:6d30ddbc4f8fc3bd;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  },
  {
    "body": "C02",
    "hash": "true",
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8c2f209c77a83771\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:be37e108dd42518e;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:054497c7ceec7c1c;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:fd74480933e7eef9;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:e8465b54c60dbf75;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:5773a351b8ebc6e5;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:36713fc1d1264aea;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:d46deed382874a98;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:a632ca753bb166f9;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8c2f209c77a83771\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:be37e108dd42518e;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:20d90e90f5833c83;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:332c3a36b872a72e;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:a6ecc54cb7b229da;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:e46484de80c7b26a;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:84308dbd85c66ce1;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:eadea6a22b692d3f;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:dbeabca2c03c1f2e;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  },
  {
    "body": "C01",
    "hash": "false",
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:31ef68256444022c\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:0ce6beca150a51e0;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:66ad5f3f573896fe;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:f3fee5da839a2fad;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:31ef68256444022c\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:679409588edb5352;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:a7c14a1dc05af75d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:1de605b4d5860476;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:31ef68256444022c\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:0ce6beca150a51e0;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:9b881fd7ef36d885;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:032a02ded321b272;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:31ef68256444022c\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:679409588edb5352;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:05445fb6c9756202;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:6d30ddbc4f8fc3bd;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  },
  {
    "body": "C02",
    "hash": "false",
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:83d5ef11247b34d6\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:bbfe4d6aef7d6977;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:054497c7ceec7c1c;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:fd74480933e7eef9;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:e8465b54c60dbf75;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:5773a351b8ebc6e5;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:36713fc1d1264aea;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:d46deed382874a98;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:a632ca753bb166f9;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:83d5ef11247b34d6\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:bbfe4d6aef7d6977;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:20d90e90f5833c83;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:332c3a36b872a72e;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:a6ecc54cb7b229da;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:e46484de80c7b26a;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:84308dbd85c66ce1;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:eadea6a22b692d3f;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:dbeabca2c03c1f2e;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  }
]
```


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


### 7. Counters, deviations and remaining defects

No-progress implementation stop-loss count: 0. Scratch discovery preparation found no serde_json rlib and used TSV without adding a dependency. Its first rustc attempt exposed module-path and mismatched cached-dependency selection; the second linked the existing matching span-enabled set and compiled both unchanged-checker enumerators. A scratch report-generator quoting error and an audit parser expectation for the commented TSV header were corrected; the final audit passed. No named final test gate remains red. Both stale-certificate failures were repaired by replaying earned cells. Any performance defects are stated above; passing tests do not imply a green performance verdict.

No git command, network request, new unsafe, production-frame typing, dependency/version change, workspace lockfile/toolchain change or out-of-scope source edit occurred. Existing test/script-owned outputs and self-restoring falsifiers remain unchanged. The prior ps read and TestCreate substitution remain explicitly accepted earlier deviations. Final hash audit confirms protected files and syntax restoration, exactly the three authorized redispatch edits, contract strengths, certificate metadata, feature exclusion and error-size equality. The tree is left uncommitted.

Evidence checkpoint: [.temp/brief-608/PROGRESS.md](/home/remi/dev/rum/.temp/brief-608/PROGRESS.md).

## Director amendment — redispatch 4: the legacy resume path pays for wrappers

Correctness is accepted as reported. The performance gate failed, and the
director attributed it before ruling. Method (reuse it): `valgrind
--tool=callgrind` on the resume harness at N=200 and N=400, base and
candidate, `callgrind_annotate --threshold=100 --auto=no`, then per-function
slopes `(Ir(2N) - Ir(N)) / N`. Callgrind counts every guest instruction
exactly, so a per-function diff has no sampling noise; `perf record`
sampling at period 1000 was too coarse to attribute. Script and raw
profiles: `.temp/director/perf-check/` (`cgdiff.py <workload> <N> <2N>`).

**Resume workloads — the cost is real and fully attributed** (instructions
per resume, candidate minus base; the driver is inlined into
`resume_eval_request_from_checkpoint` in the candidate):

| Function | do-chain | fold |
| --- | ---: | ---: |
| driver + request boundary (`drive_resuming_frames` inlined) | +409 | +583 |
| `Entry::resume` (new, out of line) | +280 | +392 |
| `FrameQueue::push_back` net of `insert_suspension_frame` | +136 | +174 |
| `__memcpy_avx_unaligned_erms` | +150 | +210 |
| `FrameQueue::append` (new, out of line) | +119 | +119 |
| total | +968 | +1477 |

The disassembly of `Entry::resume` shows why: for a legacy frame it moves
the outcome into a local, calls the frame through its vtable into a stack
temporary, then `memcpy`s the 136-byte `FrameAction` twice to re-wrap it as
`DispatchAction::Legacy` inside a `Result<DispatchAction, DecodeFailure>`,
which the driver then matches again. None of this is inherent to a
compatible driver. The base called `frame.resume` and matched its
`FrameAction` in place.

**The ordinary label is not a cost of S2's code.** `lazy-table-construct-full`
runs no input, so the driver never runs. Callgrind gives base 24092.8 and
candidate 24066.6 instructions per iteration (−26, −0.1%). `perf stat`
slopes at 20000/40000, three runs per arm on CPU 2: base 24066.7, 24067.4,
24072.0; candidate 24171.8, 24068.2, 24146.5. The candidate binary has a
run-to-run mode that the base does not. The exact count shows no added
work. This is classified as binary-layout variance, not a regression. The
classification has to be re-earned on the repaired binary (item 3).

1. **Repair the legacy path. The target is the base's work, not a smaller
   overhead.**
   - The driver dispatches on the entry itself. A legacy entry's arm does
     the `Slot::Value` decode match, calls `frame.resume`, and matches the
     returned `FrameAction` where it was produced, as the base did. There is
     no `DispatchAction` and no `Result<DispatchAction, DecodeFailure>` on
     the legacy route. A typed entry has its own arm for `Action`. Keep
     decode-before-pop ownership, the first-fault latch and every fault
     test as they are.
   - Legacy insertion pushes the `Arc` through a legacy path that neither
     builds nor re-matches an `Entry`. The small `FrameQueue` operations
     (`push_back`, `push_front`, `front`, `pop_front`, `is_empty`, the
     legacy/legacy `append`) are `#[inline]`. The widening path stays
     out of line and cold.
   - Measure the driver's result nesting
     (`Result<Result<O, EvalError>, DriverFault>`) and the boundary's fault
     check in the same way. Restructure whatever copies a value the base
     did not copy. `drive_resuming_frames` keeps its name and stays a single
     production function: census route I0009 names it.
   - **Inherent and allowed:** one discriminant test per queue operation
     (legacy or mixed), one `Slot::Value` match per legacy frame, the exact
     completion decode, and the first-fault check at the boundary. Report
     each with its callgrind-measured instructions per resume. **Anything
     else is not inherent:** an added copy or move of `FrameAction`, `Slot`
     or `EvalError`, an out-of-line wrapper the base did not call, or an
     `Entry` built only to be matched. Remove it.
2. **Gate, resume workloads.** Allocations and bytes per resume stay
   exactly equal (they are now). Instructions per resume: callgrind
   per-function diff for all five workloads, plus the brief's `perf stat`
   one-unit slopes against the recorded bases. Any residual above the floor
   must be fully covered by the inherent items above, each with its number.
   An unexplained residual is a defect: fix it, or report it with its
   callgrind attribution.
3. **Gate, ordinary panel.** Rerun the panel on the repaired candidate. For
   any label that exceeds its floor, take five `perf stat` slope runs per
   arm and a callgrind slope pair. Use each arm's own spread as the floor
   for that label, not just base/base. A label whose callgrind count shows
   no added instructions and whose workload never reaches changed code is
   reported as layout variance, with both pieces of evidence. A label with
   added instructions in callgrind is a regression: fix it, or report it
   with its attribution.
4. **Then rerun the gates the change touches** on the final tree: typed
   runtime tests, the oracle in both builds (fault delta zero), compile
   controls (30) and census in both builds, ownership, the retention
   fingerprints and certificates (replay only what discovery emits), the
   historical retention census, `cargo fmt --check`, candidate release
   builds and hashes, error sizes and feature trees, and the workspace run.
   Append a new `## Execution report` after this section.

## Director note — redispatch 4 resumed after a host power-off

The host was powered off at 11:09:50 (+08:00) while redispatch 4 was in item 4.
Your session continues from its last recorded step. The director checked the
scene: the filesystem is intact, `git fsck` is clean, and no repository file
outside `.temp/` was written after 10:31, so the working tree is the first
repair you retained after ending tuning. Your scratch under
`.temp/brief-608/redispatch-4/` survives. The previous dispatch log is
`.temp/logs/cdx_brief_608.reboot-5.log`.

Lost in flight: the workspace run (`workspace.log` stops inside
`rum-stdlib`'s `typed_slot_census` integration test; rum-runtime had passed),
the final release builds and measurements, and the report.

Do not reopen tuning or re-run the gates that already completed on this tree
and recorded an exit status. First confirm that the tree matches your last
recorded state: a source-file hash comparison against your own records is
enough. If it does not match, STOP and report the difference. Then:

1. Run the workspace suite again from the start, serial as in the brief.
2. Build the final candidate release binaries and record their hashes.
3. Run item 2 (resume-workload gate) and item 3 (ordinary panel gate) on
   those binaries.
4. Append the `## Execution report` that item 4 asks for. Report the
   residual per resume (about +136 instructions on `Do` and +85 on `Fold` in
   your first-repair profiles) against the four inherent items, each with
   its callgrind number. Report any part that no inherent item covers as a
   defect, with its attribution.

## Execution report — redispatch 4: direct legacy dispatch, recovered final gates and residual attribution

**Correctness and allocation/size gates pass; the performance gate fails.** The direct legacy dispatch repair is retained, all required final-tree correctness gates are green, both oracle fault deltas are zero, and all 64 allocation/byte slopes are exactly unchanged. The final attribution finds forbidden work still present: a pop/drop Entry temporary (+10 instructions per legacy frame), an extra boundary-result materialization (+33 per resume), and additional fault/result handling. Three ordinary labels also have positive exact-count deltas. These defects are reported below; the tree is not presented as passing S2’s performance gate.

The power-off interrupted the first workspace run without an exit record. Its log is preserved as `.temp/brief-608/redispatch-4/workspace.log`; the restart is `workspace-recovered.log` and exits zero. Before restarting, 1,471 source/protected files were checked against the entry hashes, retained repair snapshots and exact maintenance records. The one pre-format snapshot difference was the recorded formatting of `frame_root_trace.rs`; the complete post-format source context independently reproduced all 68 saved fingerprints (`fnv1a64:8c2d6136fd488bfd`). No unrecorded source file or unexplained difference was found. `recovery-source-check-complete.json` records the comparison. Completed focused gates were not rerun after the power-off. No tuning resumed.

### 1. Files changed

The complete S2 change remains 49 modified and 10 new repository files, plus this appended report. This redispatch changes 14 of those files. Scratch is under `.temp/brief-608/`; certificate output is the previously authorized `.temp/brief-546/certificates/`. `final-audit.json` and `redispatch-4.patch` record the source comparison without git.

| File | Change | This redispatch |
| --- | --- | --- |
| `crates/rum-host/src/model/outcome.rs` | Map ContinuationSlotMismatch to the existing terminal HostFailureCode::Other outcome. | retained |
| `crates/rum-runtime/Cargo.toml` | Add the empty test-continuation-slots feature. | retained |
| `crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs` | Explicitly propagate the public fault variant from capture rendering. | retained |
| `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs` | Independently enumerate legacy or mixed queue cells and both typed Arc allocations; visit frame payloads once. | retained |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs` | Explicit fatal passthrough in capture completion and outcome rendering. | retained |
| `crates/rum-runtime/src/runtime/kernel_api.rs` | Export the typed macro, the feature-gated FaultProbe and the opaque SlotMismatch beside EvalError. | retained |
| `crates/rum-runtime/src/runtime/mod.rs` | Export feature-gated FaultProbe and public SlotMismatch at the runtime facade. | retained |
| `crates/rum-runtime/src/runtime/model/requests.rs` | Add SessionRequestError::ContinuationSlotMismatch and its Display arm. | retained |
| `crates/rum-runtime/src/runtime/session/session_eval.rs` | Project the latched fault at text/inspection resume boundaries before legacy value decoding. | retained |
| `crates/rum-runtime/src/runtime/session/session_requests.rs` | Project the latched fault at request boundaries and map the new EvalError variant to SessionRequestError. | retained |
| `crates/rum-runtime/src/runtime/session/session_state.rs` | Add EvalError::ContinuationSlotMismatch(SlotMismatch), formatted through read-only static-label accessors. | retained |
| `crates/rum-runtime/src/runtime/suspension/core.rs` | Run typed and legacy frames in separate direct arms. Decode while the consumer remains queued; match each original action in place, drop both owners before finalization, and splice nested owners before the suffix. Legacy insertion passes its Arc directly. | yes |
| `crates/rum-runtime/src/runtime/suspension/core/abandon.rs` (new) | Move the existing abandonment worklist protocol into its own module and share its drain with faults. | retained |
| `crates/rum-runtime/src/runtime/suspension/core/fault.rs` (new) | First-fault latch and structural unwind; opaque SlotMismatch with suspension-only production constructor and cfg(test) injection constructor; feature/test-only monotonic FaultProbe. | retained |
| `crates/rum-runtime/src/runtime/suspension/core/queue.rs` (new) | Keep legacy Arc cells and widen only for typed entries. Inline front/pop/is_empty, direct legacy insertion and legacy append; keep widening and mixed append cold and out of line. | yes |
| `crates/rum-runtime/src/runtime/suspension/error.rs` | Exhaustiveness arm guards against structural faults entering the legacy error wire; explicitly removed with that wire at R. | retained |
| `crates/rum-runtime/src/runtime/suspension/output.rs` | Collect capture boundaries from either queue representation without cloning frame state. | retained |
| `crates/rum-runtime/src/runtime/suspension/trace.rs` | Re-export FaultProbe and SlotMismatch through the existing suspension module boundary. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed.rs` | Connect live typed exports without module-wide dead-code allowance; remove the obsolete DispatchAction export. | yes |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | Make sealed actions reachable by the driver; add only a cfg(test)-private malformed-slot constructor. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | Expose the inner borrowed layout to both accounting models; preserve exact decode and adapter-only token minting. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/chain.rs` | Implement witnessed chain installation and return the last output type. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | Centralize exact completion decoding and static fault kind names; no dual-arm decode. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/entry.rs` (new) | Private legacy/typed ownership and borrowed metadata/accounting view; remove Entry::resume and DispatchAction so the legacy action is not rewrapped. | yes |
| `crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs` | Accept fresh state of the same nominal frame and return ResumeStop<F::Output>. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests.rs` | Connect live route, driver-fault, file and accounting tests. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs` | Verify two-Arc layout, real queue cells, maintained/independent totals and below/at/above admission. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs` (new) | Fault matrix across captures/scopes, cleanup, completion, rollback/reset and direct public-variant capture defence. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/fault_support.rs` (new) | Runtime-private wrong-mode/exact-state fixtures, non-running consumer/suffix spies and common restoration assertions. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs` (new) | Nested memory-file and public-batch rollback under all three injected mismatch kinds. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs` (new) | Test macro/chain, legacy neighbors, repeat helper, nested Evaluate and both-Arc finalizer release. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | Implement witnessed suspension and the cold actual-Result typed macro. | retained |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | Check both direct resume arms independently: exact decode before pop, explicit drop before each finalizer, and unchanged structural abandonment confinement. | yes |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | Record SlotMismatch's new FrameRootTrace derive in the sorted closed inventory. | retained |
| `crates/rum-stdlib/Cargo.toml` | Enable that feature only on the existing runtime dev-dependency. | retained |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs` | Replace only the three-site count assertion with exact set equality over four discovered constructs; witness loop byte-identical. | retained |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | yes |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | yes |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | yes |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | yes |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | yes |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | yes |
| `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` | Follow unresolved abandonment operations into their split module and classify the final same-operation take_for_abandon line shift (473 to 477); keep 932/10/895/27 totals, all unresolved assertions and historical print unchanged. | yes |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Replay all 68 dependency fingerprints; preserve every contract field and strength. | yes |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Replay exactly the four emitted default/no-default C01/C02 certificate cells; preserve statuses, bodies, preconditions, strengths and exclusions. | yes |
| `crates/rum-stdlib/tests/suspension_oracle.rs` | Add the serial last aggregate fault-occurrence assertion. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` | Select the S2 typing receipt while continuing to refuse preflip. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | Record S2 mechanism routes, unchanged observed T0010, and four discovered test-support legacy uses X0036-X0039. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/drive.rs` | Unconditional fault observer around every shared drive, independent of Checks. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/faults.rs` (new) | New unconditional guards and retained probe observations for the per-build aggregate assertion. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/lifecycle.rs` | Check before session replacement/reset and retain the old probe through lifecycle completion. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/mod.rs` | Wire the fault-observer support module. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/session.rs` | Observe initialization, retirement, fresh requests and input-queue controls. | retained |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | Connect route receipts, S2 preflip refusal and scratch-copy falsifiers under brief-608. | retained |
| `crates/rum-stdlib/tests/typed_slot_census/classify.rs` | Keep codec population metadata so private injection carriers are reported separately from production registry entries. | retained |
| `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` | Preserve exact production/test-only counts and production mode checks; identify output as S2. | retained |
| `crates/rum-stdlib/tests/typed_slot_census/reporting.rs` | Write under brief-608 and include codec population in discovery output. | retained |
| `crates/rum-stdlib/tests/typed_slot_census/routes.rs` (new) | New live source/witness checks for six S2 routes, unchanged family ownership of four remaining routes and T0010 observation. | retained |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | Retain 22 S1 pairs, add seven S2 pairs and the external opaque-payload pair; require E0451 with both exact privacy spans in each feature build. | retained |

The exhaustiveness mappings remain the terminal host `Other` arm, public request-error/Display arms, explicit capture passthroughs and the legacy error-wire guard. The guard is removed with that wire at R. No production family is converted. The original session projection and fault code are restored byte-for-byte after the discarded experiments. All runtime fault-injection tests are byte-identical to redispatch entry.

`suspension/core.rs` is 29,090 characters, below 35k. The split has begun at real ownership boundaries: queue storage in `core/queue.rs`, abandonment in `core/abandon.rs`, fault/probe state in `core/fault.rs`, and entry metadata in `typed/entry.rs`. The single production driver keeps its name for I0009. Splitting that loop further would obscure its ownership ordering and revive the wrapper boundary this repair removes.

| New Rust module | Characters |
| --- | --- |
| crates/rum-runtime/src/runtime/suspension/core/abandon.rs | 4057 |
| crates/rum-runtime/src/runtime/suspension/core/fault.rs | 4118 |
| crates/rum-runtime/src/runtime/suspension/core/queue.rs | 4591 |
| crates/rum-runtime/src/runtime/suspension/typed/entry.rs | 2265 |
| crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs | 7257 |
| crates/rum-runtime/src/runtime/suspension/typed/tests/fault_support.rs | 7851 |
| crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs | 3281 |
| crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs | 11603 |
| crates/rum-stdlib/tests/suspension_oracle/support/faults.rs | 1428 |
| crates/rum-stdlib/tests/typed_slot_census/routes.rs | 3526 |

### 2. Driver, routes, fault path and probe

The retained repair dispatches on the borrowed EntryRef in the single production drive_resuming_frames function. The legacy arm performs its exact Value match, calls frame.resume, and matches the original FrameAction directly; the typed arm independently decodes and matches Action. Entry::resume and DispatchAction are removed. Legacy insertion takes its Arc directly; small queue operations are inline, and widening/mixed append remain cold and out of line. The nested driver Result and original value-returning boundary projection remain after the two unsuccessful experiments. Every fault-injection test is byte-identical to redispatch-4 entry.

Legacy-only owners retain VecDeque cells containing one Arc<dyn SuspensionFrame>; a typed insertion widens that owner to private Entry cells. Queue representation does not choose transport. Legacy accepts Value only; typed transport is its type's sealed associated mode, with no dual-arm decoder or per-frame override.

I0001 borrows and matches the actual producer Result once and constructs the frame only for InputRequired. I0002 checks the first input, adjacency and last output. I0003 uses the adapter-minted token and fresh state of the same nominal frame. I0007 requires an Expr-input successor with the parent's output. I0008 releases both frame Arcs before the pure finalizer. I0009 seeds a Value string, places nested owners before the unrun suffix and exactly decodes completion. I0004/I0005/I0006/I0010 remain S5/S18/S14/S24; T0010 observes the unchanged CLI mapping. No production frame was typed and no codec was flipped.

The driver resumes an Arc clone while the queue retains the consumer; it never clones frame state. A decode failure therefore leaves the consumer at the active front. The first private DriverFault is latched, active and nested owners enter the existing accounted abandonment worklist, pending handles are cleared, output rolls back, and structural abandonment restores scopes and loaders. Approved WithCleanup evaluation is the only guest cleanup transition; its nested input owner is abandoned before the saved suffix. Cleanup failures cannot replace the first fault. Only after unwind does the boundary project SlotMismatch; it never enters the frame-completion loop. EvaluationData and TestReport also propagate a direct internal public-variant return.

FaultProbe exists only in runtime tests or the test-continuation-slots feature. Only creation of the first private fault increments it. Retained observers survive reset, rollback and session replacement. Oracle checks are independent of Checks flags, run before lifecycle reset, and retain observations for the final aggregate. Both measured oracle deltas are zero. Release feature-tree and measured error-size evidence is reported with the four candidate binaries below.

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

Below/at/above admission tests use the real pre-existing InputString queue. Widening its four cells and adding a zero-sized typed frame costs exactly 72 bytes: 40 for both Arc layouts and 32 for wider cells. Maintained/independent reconciliation also covers nested Evaluate, finalization, capture/scope faults, cleanup input, reset and nested/public file rollback. The measured legacy allocation/byte slopes are reported below.

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

Every Cargo gate used the brief’s bounded serial offline environment: `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-608/tmp`. Workspace bound: 3600 seconds; each oracle build: 1500; other gates/builds: 900. The oracle wrapper has its own two build bounds. Full commands and raw logs are beside each `<name>.exit.json`. The following are the real recorded totals, including deliberate child rejections. Certificate discovery rejected stale recorded cells only after emitting the earned marks; the subsequent emitted-cell replay gates pass. The two census child FAILED totals in each run are deliberate falsifiers, not failures of their parent gate.

```text
runtime-typed-1: exit 0
$ cargo test -p rum-runtime --lib suspension::typed::tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.46s
```

```text
callgrind-a1: exit 0
$ python3 .temp/brief-608/redispatch-4/callgrind.py candidate-a1 do-chain fold
SLOPE do-chain 15199.425 15335.395 135.97000000000116
SLOPE fold 24550.865 24635.38 84.51499999999942
```

```text
runtime-typed-2: exit 0
$ cargo test -p rum-runtime --lib suspension::typed::tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.48s
```

```text
callgrind-a2: exit 0
$ python3 .temp/brief-608/redispatch-4/callgrind.py candidate-a2 do-chain fold
SLOPE do-chain 15257.505 15340.005 82.5
SLOPE fold 24559.97 24640.51 80.53999999999724
```

```text
runtime-typed-3: exit 0
$ cargo test -p rum-runtime --lib suspension::typed::tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.32s
```

```text
callgrind-a3: exit 0
$ python3 .temp/brief-608/redispatch-4/callgrind.py candidate-a3 do-chain fold
SLOPE do-chain 15237.255 15307.97 70.71500000000015
SLOPE fold 24540.170000000002 24619.605 79.43499999999767
```

```text
format-final: exit 0
$ cargo fmt
(no test-result total; full output in the named log)
```

```text
frame-roots-final: exit 0
$ cargo test -p rum-runtime --lib frame_root_trace
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.38s
```

```text
ownership-discovery: exit 0
$ cargo test -p rum-stdlib --lib ownership_graph::discovery_report_without_inventory_input -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 5.71s
```

```text
certificates-default-1: exit 101
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.64s
```

```text
certificates-no-default-1: exit 101
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.06s
```

```text
fmt-check: exit 0
$ cargo fmt --check
(no test-result total; full output in the named log)
```

```text
certificates-default-2: exit 0
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.32s
```

```text
certificates-no-default-2: exit 0
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.10s
```

```text
retention-census-default: exit 0
$ cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 25.32s
```

```text
retention-census-no-default: exit 0
$ cargo test -p rum-stdlib --no-default-features --lib production_rejection_census_is_complete -- --nocapture
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 25.25s
```

```text
runtime-typed: exit 0
$ cargo test -p rum-runtime --lib suspension::typed::tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.49s
```

```text
compile-default: exit 0
$ cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
COMPILE CONTROLS: 30 positive / 30 intended rejections (22 S1 + 7 S2 + opaque fault); raw Cargo JSON and primary spans retained
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.28s
```

```text
compile-no-default: exit 0
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
COMPILE CONTROLS: 30 positive / 30 intended rejections (22 S1 + 7 S2 + opaque fault); raw Cargo JSON and primary spans retained
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.27s
```

```text
typed-census-default: exit 0
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.30s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.25s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 136.96s
```

```text
typed-census-no-default: exit 0
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.87s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.20s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 135.87s
```

```text
oracle-both: exit 0
$ scripts/check-suspension-oracle.sh 1500
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.91s
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.73s
```

```text
ownership: exit 0
$ cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 79 rows; 11 storage edges (3 Declared, 8 Unbounded); 68 PublicInput
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7711 rows
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 70.01s
```

```text
workspace-recovered: exit 0
$ cargo test --workspace --no-fail-fast -- --nocapture
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.25s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.47s
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 79 rows; 11 storage edges (3 Declared, 8 Unbounded); 68 PublicInput
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7711 rows
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1092 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 500.89s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.79s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.64s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.83s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.68s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.77s
COMPILE CONTROLS: 30 positive / 30 intended rejections (22 S1 + 7 S2 + opaque fault); raw Cargo JSON and primary spans retained
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.24s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.54s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.53s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.52s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
build-all: exit 0
$ python3 .temp/brief-608/redispatch-4/build-all.py
(no test-result total; full output in the named log)
```

```text
candidate-check: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/run-perf.py check candidate 1
(no test-result total; full output in the named log)
```

```text
candidate-alloc: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/run-perf.py alloc candidate 1
(no test-result total; full output in the named log)
```

```text
base-instructions-1: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/run-perf.py instructions base 1
(no test-result total; full output in the named log)
```

```text
candidate-instructions-1: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/run-perf.py instructions candidate 1
(no test-result total; full output in the named log)
```

```text
base-instructions-2: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/run-perf.py instructions base 2
(no test-result total; full output in the named log)
```

```text
candidate-instructions-2: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/run-perf.py instructions candidate 2
(no test-result total; full output in the named log)
```

```text
performance-comparison: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/compare.py
(no test-result total; full output in the named log)
```

```text
callgrind-final: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/callgrind.py candidate
SLOPE table 15932.85 16082.05 149.1999999999989
SLOPE do-chain 15189.695 15318.04 128.34500000000116
SLOPE call-repeat 25280.415 25429.7 149.28499999999985
SLOPE cleanup 14223.89 14317.22 93.32999999999993
SLOPE fold 24538.785 24574.84 36.05500000000029
```

```text
disassembly: exit 0
$ python3 .temp/brief-608/redispatch-4/collect-disassembly.py
(no test-result total; full output in the named log)
```

```text
instruction-slopes: exit 0
$ python3 .temp/brief-608/redispatch-4/instruction-slopes.py
(no test-result total; full output in the named log)
```

```text
ordinary-lazy-first-small-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-first-small base
(no test-result total; full output in the named log)
```

```text
ordinary-lazy-first-small-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-first-small candidate
SLOPE lazy-first-small 41351.22680412371 41383.084536082475 31.857731958763907
```

```text
ordinary-lazy-table-construct-full-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-table-construct-full base
(no test-result total; full output in the named log)
```

```text
ordinary-lazy-table-construct-full-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-table-construct-full candidate
SLOPE lazy-table-construct-full 24081.333333333332 24053.184115523465 -28.14921780986697
```

```text
ordinary-replaceall-simple-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py replaceall-simple base
(no test-result total; full output in the named log)
```

```text
ordinary-replaceall-simple-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py replaceall-simple candidate
SLOPE replaceall-simple 11456.794637763833 11381.199657729607 -75.59498003422596
```

```text
ordinary-lazy-depth-map-build-8-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-map-build-8 base
(no test-result total; full output in the named log)
```

```text
ordinary-lazy-depth-map-build-8-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-map-build-8 candidate
SLOPE lazy-depth-map-build-8 18128.17391304348 18129.951992753624 1.778079710144084
```

```text
ordinary-lazy-depth-map-first-8-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-map-first-8 base
(no test-result total; full output in the named log)
```

```text
ordinary-lazy-depth-map-first-8-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-map-first-8 candidate
SLOPE lazy-depth-map-first-8 130448.07843137255 130423.13071895424 -24.947712418303126
```

```text
ordinary-lazy-depth-select-restart-8-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-select-restart-8 base
(no test-result total; full output in the named log)
```

```text
ordinary-lazy-depth-select-restart-8-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-select-restart-8 candidate
SLOPE lazy-depth-select-restart-8 179117.95495495494 179682.6936936937 564.7387387387571
```

```text
ordinary-lazy-depth-map-first-64-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-map-first-64 base
(no test-result total; full output in the named log)
```

```text
ordinary-lazy-depth-map-first-64-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-map-first-64 candidate
SLOPE lazy-depth-map-first-64 946215.9523809524 945976.4761904762 -239.47619047621265
```

```text
ordinary-lazy-depth-map-first-256-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-map-first-256 base
(no test-result total; full output in the named log)
```

```text
ordinary-lazy-depth-map-first-256-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py lazy-depth-map-first-256 candidate
SLOPE lazy-depth-map-first-256 3724784.0 3722591.8 -2192.2000000001863
```

```text
ordinary-literal-integer-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py literal-integer base
(no test-result total; full output in the named log)
```

```text
ordinary-literal-integer-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-4/ordinary-exceptions.py literal-integer candidate
SLOPE literal-integer 294.4745 293.007 -1.4674999999999727
```

```text
extra-disassembly: exit 0
$ python3 .temp/brief-608/redispatch-4/collect-extra-disassembly.py
(no test-result total; full output in the named log)
```

```text
timing-base: exit 0
$ {"command": ["scripts/core-language-microbench.sh", "--output-dir", "/home/remi/dev/rum/.temp/brief-608/redispatch-4/perf/timing/base", "--timeout", "850", "--scale", "1", "--reps", "7", "--rum-cli", "/home/remi/dev/rum/.temp/brief-608/base/rum-cli"], "profile_flags": "removed; script explicitly disables profiling", "retained_module_gc_batches": 3}
(no test-result total; full output in the named log)
```

```text
timing-candidate: exit 0
$ {"command": ["scripts/core-language-microbench.sh", "--output-dir", "/home/remi/dev/rum/.temp/brief-608/redispatch-4/perf/timing/candidate", "--timeout", "850", "--scale", "1", "--reps", "7", "--rum-cli", "/home/remi/dev/rum/.temp/brief-608/redispatch-4/candidate/rum-cli"], "profile_flags": "removed; script explicitly disables profiling", "retained_module_gc_batches": 3}
(no test-result total; full output in the named log)
```

```text
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
```

Nine native ZeroMQ tests skipped because loopback binding is unavailable. The Rust aggregate reports 151 passed in that crate, but includes these early-return skips; they are not claimed as executed native transport coverage.

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

All four final release builds use codegen-units=1 and the same source paths as the preserved bases. CLI and allocation CLI use separate target directories; allocation features are separate Cargo arguments. All four recorded feature trees exclude `test-continuation-slots`. Final audit rehashes the original bases and candidates. No release probe counter/load/branch is enabled.

| Arm | Binary | Bytes | SHA-256 |
| --- | --- | --- | --- |
| base | rum-cli | 12376160 | 76efb7c611f510a4b46c8d181d77ec37c5fe7ef057c2572ce05978e7a795ecf5 |
| candidate | rum-cli | 12536840 | 2da6654260fba872d0413360f498c2f3aefe3244c6efba24fb739b591109790e |
| base | rum-cli-alloc | 12390248 | 0e9c806bab777893c83e4e7cf1411d977ac63bb77c64374af0164e40a51e56a4 |
| candidate | rum-cli-alloc | 12547008 | 3395fa8ed00381daebe56b4331d494f95f1d63f99f0e20625bd2192f92799a9d |
| base | resume | 12019168 | be2739b2e4a62ff412500e01d7edb74e362726030796b90c722167ac639eb131 |
| candidate | resume | 12179944 | 2e55627dede2ee84ed6a73081c2c252a99455f5c71a137c47af3983ac0c16049 |
| base | resume-alloc | 12028296 | 16aaeffd58bcf727704bd829db9ad7bc3219da67a6ccc78e87bb0b44431b0fb9 |
| candidate | resume-alloc | 12185112 | 36d74888065e1794265dd793aeca5f32a7142264348fd2e23b241ef9b22edb89 |

```text
base:
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136
candidate:
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136
```

The retained first repair removes `Entry::resume`, `DispatchAction`, out-of-line legacy queue wrappers and legacy Entry construction before insertion. The two later experiments failed to improve the measured driver/copy mechanism and were discarded. The flat driver result adds 9 instructions per resume to that mechanism on both controls. In-place boundary projection saves 56 in driver/boundary but adds 60 in the libc copy routine, net +4 on both, with two extra copy calls per resume. Totals alone vary with hashing/interning and did not justify keeping either experiment. Stop-loss counter: 2.

| Attempt | Workload | Base Ir/resume | Candidate | Delta | Candidate hash |
| --- | --- | --- | --- | --- | --- |
| candidate-a1 | do-chain | 15199.425000 | 15335.395000 | +135.970000 | 2e55627dede2ee84ed6a73081c2c252a99455f5c71a137c47af3983ac0c16049 |
| candidate-a1 | fold | 24550.865000 | 24635.380000 | +84.515000 | 2e55627dede2ee84ed6a73081c2c252a99455f5c71a137c47af3983ac0c16049 |
| candidate-a2 | do-chain | 15257.505000 | 15340.005000 | +82.500000 | 54f339b010481747acad4e8d689f92d6af931d162be9592020fedcdf054d365c |
| candidate-a2 | fold | 24559.970000 | 24640.510000 | +80.540000 | 54f339b010481747acad4e8d689f92d6af931d162be9592020fedcdf054d365c |
| candidate-a3 | do-chain | 15237.255000 | 15307.970000 | +70.715000 | 12449d5745b3a1147beeff0b5574a5766f390be07bc5b5b311b25cdb54c543f1 |
| candidate-a3 | fold | 24540.170000 | 24619.605000 | +79.435000 | 12449d5745b3a1147beeff0b5574a5766f390be07bc5b5b311b25cdb54c543f1 |

```json
[
  {
    "attempt": "candidate-a1",
    "label": "do-chain",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 9627,
    "calls_2N": 18427,
    "calls_per_resume": 44.0
  },
  {
    "attempt": "candidate-a1",
    "label": "fold",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 13055,
    "calls_2N": 25255,
    "calls_per_resume": 61.0
  },
  {
    "attempt": "candidate-a2",
    "label": "do-chain",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 9627,
    "calls_2N": 18427,
    "calls_per_resume": 44.0
  },
  {
    "attempt": "candidate-a2",
    "label": "fold",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 13055,
    "calls_2N": 25255,
    "calls_per_resume": 61.0
  },
  {
    "attempt": "candidate-a3",
    "label": "do-chain",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 10028,
    "calls_2N": 19228,
    "calls_per_resume": 46.0
  },
  {
    "attempt": "candidate-a3",
    "label": "fold",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 13456,
    "calls_2N": 26056,
    "calls_per_resume": 63.0
  }
]
```

Final allocations/bytes and instruction panel: 64 labels, including all five resume workloads. Each output was checked before measurement; raw N/2N counts precede parsing. Allocation bases are the preserved original measurements. Instruction pairs are fresh base/candidate runs on CPU 2 with stdin `/dev/null` and `cpu_core/instructions/u`. The frozen base/base floors remain visible below. The ordinary exceptions additionally use five fresh slopes per arm and each arm’s own spread. No sample was discarded or selected for a favorable result.

```tsv
label	engine	N	base_alloc	candidate_alloc	delta_alloc	base_bytes	candidate_bytes	delta_bytes	base1_instructions	candidate1_instructions	delta1_percent	base2_instructions	candidate2_instructions	delta2_percent	floor_percent	ordinary_threshold_percent	ordinary_regression
lazy-range-take-small	cli	20000	141.0	141.0	0.0	2714.0	2714.0	0.0	42193.6645	42205.689	0.028498354296768724	42182.20045	42205.4674	0.05515821780701646	0.143667	0.143667	False
lazy-range-normal-small	cli	20000	157.0	157.0	0.0	3388.0	3388.0	0.0	54102.31305	54075.3586	-0.04982125251297154	54143.18125	54105.99535	-0.06868067066155836	0.122231	0.122231	False
lazy-map-realize-small	cli	20000	333.0	333.0	0.0	13740.0	13740.0	0.0	178790.40205	178782.73065	-0.004290722495181587	178825.4502	178800.01	-0.014226274823592178	0.027087	0.05	False
lazy-select-realize-small	cli	20000	236.0	236.0	0.0	6834.0	6834.0	0.0	114669.67055	114567.96275	-0.0886963392431106	114568.2544	114541.03265	-0.02376029044220962	0.006677	0.05	False
lazy-select-scan-small	cli	20000	1471.0	1471.0	0.0	62793.0	62793.0	0.0	805413.9389	805111.85495	-0.03750667022383025	805405.9167	805244.4802	-0.020044116470041562	0.007982	0.05	False
lazy-nestlist-realize-small	cli	20000	197.0	197.0	0.0	11316.0	11316.0	0.0	145352.19065	145291.48115	-0.04176717236149541	145258.9138	145293.34725	0.023704879170027837	0.033018	0.05	False
lazy-foldlist-realize-small	cli	20000	216.0	216.0	0.0	7604.0	7604.0	0.0	112877.8957	112810.22095	-0.05995394366656859	112734.8444	112828.12945	0.08274730895889704	0.106063	0.106063	False
lazy-drop-realize-small	cli	20000	351.0	351.0	0.0	14204.0	14204.0	0.0	191469.1091	191498.31885	0.015255594041940768	191469.4459	191503.75805	0.017920431032081297	0.150339	0.150339	False
lazy-first-small	cli	20000	64.0	64.0	0.0	2868.0	2868.0	0.0	41212.64745	41208.8803	-0.009140761958015275	41196.65785	41376.20405	0.4358271019307969	0.200265	0.200265	False
lazy-part-small	cli	20000	152.0	152.0	0.0	6182.0	6182.0	0.0	88319.2033	88333.7729	0.01649652562027981	88257.18935	88266.39605	0.010431671422805877	0.111397	0.111397	False
lazy-transparent-adapters-small	cli	20000	319.0	319.0	0.0	12876.0	12876.0	0.0	181382.98185	181427.85215	0.02473787757943935	181899.82825	181585.8655	-0.172602004642064	0.015381	0.05	False
lazy-table-construct-short	cli	20000	63.0	63.0	0.0	1537.0	1537.0	0.0	23532.7147	23584.6195	0.2205644383221017	23582.77165	23567.947	-0.06286220390043784	0.305248	0.305248	False
lazy-table-construct-full	cli	20000	64.0	64.0	0.0	1449.0	1449.0	0.0	24068.94315	24176.03335	0.44493104384601967	24068.20815	24102.1754	0.141129118496508	0.190008	0.190008	False
lazy-table-realize-small	cli	20000	251.0	251.0	0.0	9190.0	9190.0	0.0	117206.6436	117146.148	-0.05161448032455773	117206.648	117175.91935	-0.026217497492120323	0.09186	0.09186	False
lazy-permutations-take-small	cli	20000	29.0	29.0	0.0	1729.0	1729.0	0.0	28199.43525	28167.81295	-0.11213806134645621	28198.9712	28191.79865	-0.02543550241294046	0.129913	0.129913	False
lazy-subsets-take-small	cli	20000	29.0	29.0	0.0	1417.0	1417.0	0.0	25346.611	25337.1604	-0.03728545800462113	25366.2328	25356.85955	-0.036951683262953505	0.037505	0.05	False
lazy-tuples-take-small	cli	20000	29.0	29.0	0.0	1361.0	1361.0	0.0	26566.5001	26584.7746	0.06878775876089094	26582.50025	26584.77245	0.00854772868854603	0.179754	0.179754	False
scan-no-input	cli	20000	33.0	33.0	0.0	2536.0	2536.0	0.0	29709.01505	29681.91575	-0.0912157469858732	29707.29285	29680.3147	-0.09081322265284131	0.003205	0.05	False
optionvalue-three-no-input	cli	20000	9.0	9.0	0.0	1128.0	1128.0	0.0	9589.4013	9588.1864	-0.012669195521086696	9590.96875	9588.334	-0.027471156133207497	0.029575	0.05	False
replacerepeated-no-input	cli	20000	57.0	57.0	0.0	7899.0	7899.0	0.0	39181.3188	39174.1105	-0.018397287842175647	39181.43315	39174.27335	-0.01827345103121436	0.000695	0.05	False
dot-three-no-input	cli	20000	596.0	596.0	0.0	23504.0	23504.0	0.0	254199.5548	254197.5278	-0.0007974050157510959	254198.62685	254198.6181	-3.4421901107251074e-06	0.000109	0.05	False
differences-no-input	cli	20000	13.0	13.0	0.0	704.0	704.0	0.0	10037.2379	10035.03165	-0.021980648680242076	10037.6935	10035.3302	-0.023544253468177345	0.071642	0.071642	False
do-count-null	cli	20000	0.0	0.0	0.0	0.0	0.0	0.0	537.0071	537.03635	0.005446855358148817	536.90405	536.777	-0.023663446010502298	0.053612	0.053612	False
do-iterator-null	cli	20000	0.0	0.0	0.0	32.0	32.0	0.0	1154.03365	1153.94935	-0.007304812992237064	1154.0106	1154.01745	0.0005935820693414584	0.018868	0.05	False
table-iterator-null	cli	20000	0.0	0.0	0.0	32.0	32.0	0.0	387.0137	386.89645	-0.0302960851256584	387.09695	386.94535	-0.03916331554665309	0.042006	0.05	False
replaceall-simple	cli	20000	5.0	5.0	0.0	1352.0	1352.0	0.0	11406.55705	11458.2316	0.4530249555013688	11406.19735	11380.0537	-0.22920566072793802	0.452949	0.452949	False
table-small	cli	20000	22.0	22.0	0.0	977.0	977.0	0.0	14632.8897	14575.0619	-0.3951905685450452	14633.14735	14634.09195	0.006455207327626944	0.572845	0.572845	False
table-iterator-plus	cli	20000	2.0	2.0	0.0	72.0	72.0	0.0	563.20915	563.1814	-0.004927121656306266	563.386	563.19435	-0.03401752972207461	0.029983	0.05	False
table-iterator-pair-list	cli	20000	2.0	2.0	0.0	136.0	136.0	0.0	1640.79015	1640.7457	-0.0027090606315560706	1640.6887	1640.86805	0.010931385094581358	0.001688	0.05	False
take-multispec-small	cli	20000	109.0	109.0	0.0	5856.0	5856.0	0.0	49931.65995	49921.6505	-0.020046299301923476	49931.8219	49922.2797	-0.019110458294735455	0.046752	0.05	False
lazy-depth-map-build-8	cli	1000	16.001	16.001	0.0	1879.864	1879.864	0.0	18097.717	18126.068	0.15665511843288016	18119.211	18102.383	-0.09287380118261357	0.087984	0.087984	False
lazy-depth-map-first-8	cli	1000	188.0	188.0	0.0	11176.0	11176.0	0.0	130392.695	130789.497	0.3043130598688881	130375.161	130374.495	-0.0005108335014769594	0.026366	0.05	False
lazy-depth-map-eight-8	cli	1000	1464.001	1464.001	0.0	82776.864	82776.864	0.0	959564.021	959586.58	0.002350963511177362	959570.736	959670.428	0.010389228877016876	0.014316	0.05	False
lazy-depth-select-restart-8	cli	1000	327.0	327.0	0.0	16980.0	16980.0	0.0	179409.695	179393.094	-0.00925312313807547	179388.726	179953.501	0.3148330514371356	0.165894	0.165894	False
lazy-depth-drop-all-one-8	cli	1000	170.0	170.0	0.0	3128.0	3128.0	0.0	46669.199	46660.297	-0.019074679211872336	46664.732	46661.892	-0.0060859665925061535	0.002949	0.05	False
lazy-depth-take-map-8	cli	1000	220.0	220.0	0.0	12144.0	12144.0	0.0	138948.648	138949.657	0.0007261675550873647	138948.347	138936.192	-0.008747855057245069	0.003719	0.05	False
lazy-depth-fold-through-source-8	cli	1000	743.001	743.001	0.0	58440.864	58440.864	0.0	589568.917	589552.08	-0.0028558154126834268	589531.346	589538.83	0.0012694829631509918	0.001805	0.05	False
lazy-depth-nest-map-8	cli	1000	192.0	192.0	0.0	11964.0	11964.0	0.0	141007.746	140996.698	-0.00783503056633883	141010.603	140992.595	-0.012770670869344247	0.006765	0.05	False
lazy-depth-map-build-64	cli	1000	16.001	16.001	0.0	5911.864	5911.864	0.0	30123.384	30141.362	0.05968120978705471	30139.198	30123.449	-0.05225421061303415	0.116452	0.116452	False
lazy-depth-map-first-64	cli	1000	1364.0	1364.0	0.0	85320.0	85320.0	0.0	944593.488	949359.567	0.5045640331579282	946259.829	944594.826	-0.17595621720088994	0.335237	0.335237	False
lazy-depth-map-eight-64	cli	1000	10872.001	10872.001	0.0	647704.864	647704.864	0.0	7421720.751	7422184.465	0.0062480658536978595	7421198.822	7421848.898	0.00875971680038834	0.02104	0.05	False
lazy-depth-select-restart-64	cli	1000	1671.0	1671.0	0.0	99412.0	99412.0	0.0	975902.213	981979.01	0.62268503125118	981920.541	981711.722	-0.02126638473082254	0.656111	0.656111	False
lazy-depth-drop-all-one-64	cli	1000	1178.0	1178.0	0.0	17520.0	17520.0	0.0	327581.558	327582.706	0.0003504470785786751	327582.181	327578.773	-0.001040349627567938	0.002126	0.05	False
lazy-depth-take-map-64	cli	1000	1620.0	1620.0	0.0	93064.0	93064.0	0.0	1007401.91	1007433.747	0.0031603076869224367	1007578.649	1007531.914	-0.004638347591656711	0.001015	0.05	False
lazy-depth-fold-through-source-64	cli	1000	43334.001	43334.001	0.0	3402649.864	3402649.864	0.0	33998795.024	33999580.261	0.0023096024416524585	33996515.61	33998445.679	0.005677255346214771	0.003458	0.05	False
lazy-depth-nest-map-64	cli	1000	1368.0	1368.0	0.0	86108.0	86108.0	0.0	961949.278	958617.95	-0.3463101512926281	958616.172	958614.727	-0.00015073812045240942	0.17411	0.17411	False
lazy-depth-map-build-256	cli	1000	16.001	16.001	0.0	19735.864	19735.864	0.0	70747.545	70765.381	0.02521076879768991	70761.914	70759.289	-0.0037096226650934483	0.009013	0.05	False
lazy-depth-map-first-256	cli	1000	5396.0	5396.0	0.0	339528.0	339528.0	0.0	3722822.26	3722649.02	-0.004653458798209442	3716196.522	3735992.914	0.5327057350924536	0.004742	0.05	False
lazy-depth-map-eight-256	cli	1000	43128.001	43128.001	0.0	2584600.864	2584600.864	0.0	29547984.364	29544415.05	-0.012079720755331369	29545858.947	29547094.798	0.004182823055565166	0.005175	0.05	False
lazy-depth-select-restart-256	cli	1000	6279.0	6279.0	0.0	382036.0	382036.0	0.0	3652409.209	3665013.901	0.3451062375196212	3664400.098	3665006.657	0.0165527503487084	0.705262	0.705262	False
lazy-depth-drop-all-one-256	cli	1000	4634.0	4634.0	0.0	66864.0	66864.0	0.0	2218614.996	2218608.751	-0.00028148191600907424	2218609.527	2218606.971	-0.00011520729397140883	0.000437	0.05	False
lazy-depth-take-map-256	cli	1000	6420.0	6420.0	0.0	370504.0	370504.0	0.0	3986126.474	3985399.14	-0.018246636295760066	3985755.352	3985398.748	-0.008946961579592294	0.003231	0.05	False
lazy-depth-fold-through-source-256	cli	1000	689320.001	689320.001	0.0	54008857.864	54008857.864	0.0	530572482.035	530550564.827	-0.00413086029564802	530610708.583	530532539.264	-0.014731952773583412	0.004122	0.05	False
lazy-depth-nest-map-256	cli	1000	5400.0	5400.0	0.0	340316.0	340316.0	0.0	3746879.208	3733563.362	-0.3553849820290167	3733558.464	3733561.254	7.472763656757309e-05	0.000294	0.05	False
literal-integer	cli	20000	0.0	0.0	0.0	0.0	0.0	0.0	293.9565	294.1158	0.054191691627836214	293.8975	294.1789	0.09574766712885374	0.035308	0.05	True
downvalue-pattern-hit	cli	20000	11.0	11.0	0.0	908.0	908.0	0.0	13949.5357	13949.47225	-0.00045485384864152323	13949.9385	13949.653	-0.002046604004746211	0.001312	0.05	False
module-local	cli	20000	23.0001	23.0001	0.0	2163.6272	2163.6272	0.0	22652.71	22653.44085	0.0032263247973451925	22651.94915	22655.0537	0.013705443092071334	0.002678	0.05	False
block-local	cli	20000	8.0	8.0	0.0	752.0	752.0	0.0	14671.75035	14671.69975	-0.0003448804593464416	14671.6412	14671.857	0.0014708647591454849	0.002024	0.05	False
matcher-single-wide-small	cli	1000	11.0	11.0	0.0	287280.0	287280.0	0.0	760221.003	760179.371	-0.005476302264173505	760179.692	760193.928	0.0018727151158692834	0.003397	0.05	False
resume-table	resume	1000	27.003	27.003	0.0	1742.456	1742.456	0.0	15715.704	15714.823	-0.005605857682222659	15676.738	15801.019	0.792773343536135	0.153556	0.153556	False
resume-do-chain	resume	1000	24.0	24.0	0.0	1634.0	1634.0	0.0	15123.596	15392.001	1.7747432555061726	15107.894	15265.884	1.0457446947933269	0.024326	0.05	False
resume-call-repeat	resume	1000	40.007	40.007	0.0	4210.024	4210.024	0.0	24620.05	24754.935	0.5478664746822348	24560.454	24631.371	0.28874466245614183	0.062048	0.062048	False
resume-cleanup	resume	1000	22.0	22.0	0.0	1394.0	1394.0	0.0	14182.899	14322.95	0.9874638464252072	14203.123	14318.983	0.8157360884645026	0.276539	0.276539	False
resume-fold	resume	1000	41.0	41.0	0.0	2873.0	2873.0	0.0	24577.675	24539.863	-0.15384693629482138	24481.99	24536.785	0.22381758999165946	0.334755	0.334755	False
```

**Resume gate: FAIL.** The exact final profiles use N=200/400, `callgrind_annotate --threshold=100 --auto=no`, with instruction addresses and jumps retained. All totals below reconcile to raw profile summaries. `perf record` was used only to obtain matching final-binary disassembly; its sampled counts are not used for attribution. Unsampled insertion guards and the queue-drop test are verified from the recorded ELF instruction bytes. Every numeric guard contribution is the actual address-count slope, not an assumed number of cycles or a source-level estimate.

| Workload | Base Ir/resume | Candidate | Exact delta | Legacy frames/resume |
| --- | --- | --- | --- | --- |
| table | 15932.850000 | 16082.050000 | +149.200000 | 4 |
| do-chain | 15189.695000 | 15318.040000 | +128.345000 | 5 |
| call-repeat | 25280.415000 | 25429.700000 | +149.285000 | 4 |
| cleanup | 14223.890000 | 14317.220000 | +93.330000 | 4 |
| fold | 24538.785000 | 24574.840000 | +36.055000 | 7 |

The four allowed items are counted conservatively. Queue guards include front, pop, push-back, both append tags, output-boundary visitation and queue drop. Each is a discriminator test and branch. No additional queue guard with a positive slope is claimed for push-front/is-empty on these complete-action workloads. The boundary column counts the fault discriminator load/test/branch; clearing the Option and eagerly loading its other three words are disclosed separately instead of receiving inherent credit. Completion is reached once at the end of the whole request, so its exact-decode checks have zero per-resume slope.

| Workload | Queue checks | Legacy Value match | Completion decode | First-fault boundary check | Allowed total | Exact delta minus allowed |
| --- | --- | --- | --- | --- | --- | --- |
| table | 32.0 | 8.0 | 0.0 | 3.0 | 43.0 | +106.200000 |
| do-chain | 38.0 | 10.0 | 0.0 | 3.0 | 51.0 | +77.345000 |
| call-repeat | 32.0 | 8.0 | 0.0 | 3.0 | 43.0 | +106.285000 |
| cleanup | 32.0 | 8.0 | 0.0 | 3.0 | 43.0 | +50.330000 |
| fold | 50.0 | 14.0 | 0.0 | 3.0 | 67.0 | -30.945000 |

| Queue operation | Table | Do | Call-repeat | Cleanup | Fold |
| --- | --- | --- | --- | --- | --- |
| front | 8.0 | 10.0 | 8.0 | 8.0 | 14.0 |
| pop_front | 8.0 | 10.0 | 8.0 | 8.0 | 14.0 |
| legacy_append | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 |
| legacy_push_back | 8.0 | 10.0 | 8.0 | 8.0 | 14.0 |
| output_boundary_visit | 2.0 | 2.0 | 2.0 | 2.0 | 2.0 |
| drop | 2.0 | 2.0 | 2.0 | 2.0 | 2.0 |

The retained repair still has concrete non-inherent work. This is an attributed defect, not a claim that the first repair meets the director’s base-work target:

- **Pop/drop result:** candidate addresses `0x729efe..0x729f90` execute 20 instructions per frame before the Arc decrement, versus 10 at base `0x729495..0x7294c8`. The candidate materializes the `Option<Entry>` tag/pointers, copies the temporary, then tests its tag at `0x729f5b/0x729f63`. This adds 10 instructions per frame: 50 per Do resume and 70 per Fold resume. Legacy insertion is direct, but `FrameQueue::pop_front` still builds an Entry that is re-matched for drop. No extra Arc clone is claimed; the base also cloned the consumer Arc.
- **Boundary result copying:** `0x72bbc4..0x72bca6` executes 33 instructions per resume to materialize and copy the outcome after the no-fault branch. The base proceeds from its already prepared result directly to the finalization call (`0x70bb7c..0x70bb93`); this extra block is absent there. The pre-projection copy at candidate `0x72bb0b..0x72bb7f` corresponds to base `0x70bb09..0x70bb74` and is not charged again.
- **Additional fault/result handling:** the nested driver-result test at `0x72a736/0x72a73f/0x72a745` costs 3 per resume. Boundary eager payload loads plus clearing None at `0x72bb8e/0x72bb95/0x72bb9c/0x72bba3` cost 4. Neither is credited to the single discriminator check. The queue-drop guard has 2 credited instructions; its extra field-address adjustment remains in the other changes.

| Workload | Extra pop/drop | Extra result copying | Nested result test | Eager fault words/clear | Identified uncredited total | Other signed net changes |
| --- | --- | --- | --- | --- | --- | --- |
| table | 40.0 | 33 | 3 | 4 | 80.0 | +26.200000 |
| do-chain | 50.0 | 33 | 3 | 4 | 90.0 | -12.655000 |
| call-repeat | 40.0 | 33 | 3 | 4 | 80.0 | +26.285000 |
| cleanup | 40.0 | 33 | 3 | 4 | 80.0 | -29.670000 |
| fold | 70.0 | 33 | 3 | 4 | 110.0 | -140.945000 |

For each row, allowed checks + identified uncredited work + other signed net changes equals the full exact delta. The last column includes all remaining compiler/code-layout, copy-routine, hashing/interning and allocation-path differences in the following per-function tables; it is not another inherent allowance. Table and call-repeat retain positive unexplained balances of +26.200000 and +26.285000, which are also defects. A negative balance or Fold’s total below the allowed-check sum does not excuse its explicitly added copies/Entry temporary. The paired perf samples exceed the frozen floor in both samples for Do, call-repeat and cleanup; Table is mixed and Fold stays below its floor. The source-level no-added-copy requirement still fails.

The full address sets, raw test/jump bytes and measured sums are in `.temp/brief-608/redispatch-4/instruction-attribution/measured-attribution.json`; the reproducer is `attribute-resumes.py`. Address counts are in `*-instructions.json`, with matching assembly and compact annotated instruction listings beside them. No instruction count was taken from the perf sampling percentages.

All nonzero per-function differences follow. The complete tables, including unchanged functions, are `candidate/callgrind/{table,do-chain,call-repeat,cleanup,fold}-diff.tsv`. Exact totals can differ between process pairs through hashing/interning/cache paths even though Callgrind itself has no sampling noise; the ~136/~85 figures from the first repair were retained rather than substituted for these final profiles.

**table**

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1202.000000 | +919.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 593.000000 | 0.000000 | -593.000000 |
| 0x00000000001b1980 | 1558.950000 | 1318.950000 | -240.000000 |
| rum_core::expr::Symbol::system | 240.885000 | 279.010000 | +38.125000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 493.000000 | 508.000000 | +15.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 273.000000 | 284.000000 | +11.000000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 9.000000 | 0.000000 | -9.000000 |
| rum_core::expr::intern_symbol_text | 187.940000 | 195.485000 | +7.545000 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 129.200000 | 131.850000 | +2.650000 |
| rum_core::expr::intern_text | 378.115000 | 380.640000 | +2.525000 |
| brief608_resume::main | 155.000000 | 154.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 1.380000 | 0.525000 | -0.855000 |
| 0x00000000001b1220 | 165.540000 | 165.860000 | +0.320000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.000000 | -0.110000 | -0.110000 |

**do-chain**

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1341.000000 | +1058.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 677.000000 | 0.000000 | -677.000000 |
| 0x00000000001b1980 | 1485.775000 | 1185.775000 | -300.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| rum_core::expr::Symbol::system | 162.985000 | 189.080000 | +26.095000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 532.000000 | 549.000000 | +17.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 319.000000 | 334.000000 | +15.000000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 21.000000 | 12.000000 | -9.000000 |
| rum_core::expr::intern_symbol_text | 198.290000 | 190.790000 | -7.500000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 129.415000 | 135.580000 | +6.165000 |
| rum_core::expr::intern_text | 379.320000 | 383.335000 | +4.015000 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| _mi_page_free_collect | -2.280000 | -1.140000 | +1.140000 |
| brief608_resume::main | 140.000000 | 139.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 0.105000 | -0.770000 | -0.875000 |
| 0x00000000001b1220 | 155.485000 | 155.090000 | -0.395000 |
| mi_theap_malloc_zero_aligned_at_overalloc | 39.600000 | 39.420000 | -0.180000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.150000 | 0.000000 | -0.150000 |
| mi_theap_malloc_zero_aligned_at_generic | 54.170000 | 54.220000 | +0.050000 |
| hashbrown::raw::RawTable<T,A>::reserve_rehash | 0.000000 | -0.020000 | -0.020000 |

**call-repeat**

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1202.000000 | +919.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 593.000000 | 0.000000 | -593.000000 |
| 0x00000000001b1980 | 2321.055000 | 2081.055000 | -240.000000 |
| rum_core::expr::Symbol::system | 211.375000 | 249.005000 | +37.630000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| 0x00000000001b1220 | 216.270000 | 239.210000 | +22.940000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 267.000000 | 278.000000 | +11.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 415.000000 | 426.000000 | +11.000000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 9.000000 | 0.000000 | -9.000000 |
| rum_core::expr::intern_symbol_text | 298.090000 | 289.395000 | -8.695000 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| brief608_resume::main | 144.750000 | 143.750000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 199.985000 | 200.825000 | +0.840000 |
| _mi_page_free_collect | 15.770000 | 16.530000 | +0.760000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 0.135000 | 0.690000 | +0.555000 |
| rum_core::expr::intern_text | 572.365000 | 572.795000 | +0.430000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | -0.110000 | -0.300000 | -0.190000 |
| _mi_theap_collect_retired | 8.480000 | 8.495000 | +0.015000 |

**cleanup**

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1202.000000 | +919.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 593.000000 | 0.000000 | -593.000000 |
| 0x00000000001b1980 | 1271.000000 | 1031.000000 | -240.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 805.000000 | 836.000000 | +31.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| rum_core::expr::Symbol::system | 191.720000 | 178.735000 | -12.985000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 265.000000 | 276.000000 | +11.000000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 21.000000 | 12.000000 | -9.000000 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 137.255000 | 133.940000 | -3.315000 |
| rum_core::expr::intern_symbol_text | 198.820000 | 196.940000 | -1.880000 |
| 0x00000000001b1220 | 156.770000 | 155.550000 | -1.220000 |
| rum_core::expr::intern_text | 382.495000 | 381.475000 | -1.020000 |
| brief608_resume::main | 140.000000 | 139.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.110000 | -0.765000 | -0.875000 |
| _mi_page_free_collect | -1.960000 | -2.200000 | -0.240000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 0.720000 | 0.585000 | -0.135000 |

**fold**

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1619.000000 | +1336.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 845.000000 | 0.000000 | -845.000000 |
| 0x00000000001b1980 | 2073.775000 | 1653.775000 | -420.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 442.000000 | 477.000000 | +35.000000 |
| rum_runtime::runtime::suspension::core::SuspendedEvaluation::push_back | 30.000000 | 0.000000 | -30.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| rum_core::expr::Symbol::system | 193.535000 | 165.965000 | -27.570000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 571.000000 | 590.000000 | +19.000000 |
| rum_core::expr::intern_text | 379.520000 | 362.665000 | -16.855000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 9.000000 | 0.000000 | -9.000000 |
| rum_core::expr::intern_symbol_text | 196.580000 | 192.125000 | -4.455000 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 305.350000 | 308.360000 | +3.010000 |
| brief608_resume::main | 140.000000 | 139.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 529.340000 | 528.560000 | -0.780000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.130000 | -0.295000 | -0.425000 |
| _mi_malloc_generic | 49.260000 | 49.515000 | +0.255000 |
| mi_theap_malloc_zero_aligned_at_generic | 269.880000 | 270.125000 | +0.245000 |
| mi_find_page | 34.695000 | 34.875000 | +0.180000 |
| 0x00000000001b1220 | 473.640000 | 473.765000 | +0.125000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.255000 | -0.140000 | +0.115000 |
| mi_bin | 114.370000 | 114.480000 | +0.110000 |
| _mi_theap_malloc_zero | 114.140000 | 114.190000 | +0.050000 |
| mi_good_size | 47.390000 | 47.425000 | +0.035000 |
| _mi_bin_size | 27.080000 | 27.100000 | +0.020000 |
| mi_theap_malloc_aligned | 805.460000 | 805.450000 | -0.010000 |
| _mi_bin | 0.970000 | 0.975000 | +0.005000 |

Ordinary exceptions: the floor below is `max(0.05%, base spread, candidate spread)`, with each spread `(max - min) / median * 100` across that arm’s five fresh slopes. The exact Callgrind N/2N sizes were fixed in the saved per-arm budget before collection. The original `.rum` body, setup and Do iterator are retained. Both instruction-count increases and source-path evidence are disclosed; a positive exact-count difference is reported as a defect as the amendment requires.

`lazy-first-small`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 41258.842150, 41182.248500, 41209.934300, 41210.280850, 41211.883450 | 41210.280850 | 0.185861 |
| candidate | 41239.197050, 41209.442500, 41210.867500, 41209.245600, 41211.307600 | 41210.867500 | 0.072679 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.185861 | +0.001424 | 41351.226804 | 41383.084536 | +31.857732 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::Symbol::system | 309.927835 | 359.463918 | +49.536082 |
| rum_core::expr::intern_symbol_text | 391.832990 | 379.620619 | -12.212371 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 1721.265979 | 1725.868041 | +4.602062 |
| 0x00000000001b1220 | 642.183505 | 638.424742 | -3.758763 |
| rum_core::expr::intern_text | 777.727835 | 774.463918 | -3.263918 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 458.762887 | 455.709278 | -3.053608 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.000000 | 0.030928 | +0.030928 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.107216 | -0.129897 | -0.022680 |
| _mi_os_alloc | 0.000000 | 0.000000 | +0.000000 |
| _IO_setb | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_accepted_file_request | 0.000000 | 0.000000 | +0.000000 |
| _mi_os_good_alloc_size | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::matcher_and_symbols::compile_matcher_arg | 0.000000 | 0.000000 | +0.000000 |
| _mi_theap_random_next | 0.000000 | 0.000000 | +0.000000 |
| std::sync::once_lock::OnceLock<T>::initialize | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000000001090 | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::matcher_and_symbols::pattern_key_value_pattern_entries | 0.000000 | 0.000000 | +0.000000 |
| num_bigint::bigint::division::<impl core::ops::arith::Div<&num_bigint::bigint::BigInt> for num_bigint::bigint::BigInt>::div | 0.000000 | 0.000000 | +0.000000 |
| 0x000000000001cea0 | 0.000000 | 0.000000 | +0.000000 |
| core::ptr::drop_in_place<alloc::vec::Vec<rum_runtime::runtime::options::OptionValueScope>> | 0.000000 | 0.000000 | +0.000000 |

`lazy-table-construct-full`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 24040.282950, 24116.804800, 24044.247950, 24077.723750, 24066.766150 | 24066.766150 | 0.317957 |
| candidate | 24050.947300, 24128.095300, 24072.289550, 24047.574050, 24033.172950 | 24050.947300 | 0.394672 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.394672 | -0.065729 | 24081.333333 | 24053.184116 | -28.149218 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::Symbol::system | 318.098676 | 286.693141 | -31.405535 |
| 0x00000000001b1220 | 190.034898 | 194.261131 | +4.226233 |
| rum_core::expr::intern_symbol_text | 393.504212 | 392.227437 | -1.276775 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 432.345367 | 433.064982 | +0.719615 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 269.091456 | 268.433213 | -0.658243 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.025271 | 0.365824 | +0.391095 |
| rum_core::expr::intern_text | 776.754513 | 776.365824 | -0.388688 |
| 0x000000000001b870 | -0.262335 | -0.086643 | +0.175692 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.054152 | 0.121540 | +0.067389 |
| rum_runtime::runtime::substitution::binding_substitution::rule_rhs_may_splice_without_bindings | 0.000000 | 0.000000 | +0.000000 |
| _mi_os_reuse | 0.045728 | 0.045728 | +0.000000 |
| rum_runtime::runtime::resolution::symbol_generation_key | 0.000000 | 0.000000 | +0.000000 |
| 0x00000000000d0b60 | 0.000000 | 0.000000 | +0.000000 |
| pthread_getaffinity_np | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000000005c40 | 0.000000 | 0.000000 | +0.000000 |
| rum_syntax::parser::front::next_min_precedence_for_binary_op | 0.000000 | 0.000000 | +0.000000 |
| 0x00000000052c4000 | 0.000000 | 0.000000 | +0.000000 |
| getauxval | 0.000000 | 0.000000 | +0.000000 |
| 0x00000000000a1060 | 0.000000 | 0.000000 | +0.000000 |
| __mi_stat_counter_increase_mt | 0.000000 | 0.000000 | +0.000000 |

`replaceall-simple`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 11406.361500, 11380.329550, 11406.284600, 11406.225350, 11406.306750 | 11406.284600 | 0.228225 |
| candidate | 11458.371850, 11406.213100, 11406.276600, 11406.193000, 11406.321400 | 11406.276600 | 0.457457 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.457457 | -0.000070 | 11456.794638 | 11381.199658 | -75.594980 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 670.318311 | 592.303480 | -78.014832 |
| rum_core::expr::intern_symbol_text | -0.208785 | 0.864803 | +1.073588 |
| rum_core::expr::Symbol::system | -0.156303 | 0.351968 | +0.508272 |
| hashbrown::map::HashMap<K,V,S,A>::insert | -0.306902 | 0.156874 | +0.463776 |
| rum_core::expr::intern_text | -0.148888 | 0.181974 | +0.330861 |
| 0x00000000001b1220 | 75.974900 | 76.073588 | +0.098688 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.008557 | -0.036509 | -0.045066 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 0.044495 | 0.034227 | -0.010268 |
| <rum_runtime::runtime::model::definitions::SymbolDefinition as core::clone::Clone>::clone | 0.000000 | 0.000000 | +0.000000 |
| core::str::<impl str>::trim_matches | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000000004900 | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000004070000 | 0.000000 | 0.000000 | +0.000000 |
| rum_host::model::outcome::structured_outputs | 0.000000 | 0.000000 | +0.000000 |
| calloc | 0.000000 | 0.000000 | +0.000000 |
| mi_free | 100.000000 | 100.000000 | +0.000000 |
| stpcpy | 0.000000 | 0.000000 | +0.000000 |
| mi_bitmap_popcountN | -0.005705 | -0.005705 | +0.000000 |
| prctl | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::definitions::definition_analysis_helpers::canonicalize_pattern_names | 0.000000 | 0.000000 | +0.000000 |
| wcsncpy | 0.000000 | 0.000000 | +0.000000 |

`lazy-depth-map-build-8`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 18098.606000, 18107.474000, 18121.035000, 18116.461000, 18095.401000 | 18107.474000 | 0.141566 |
| candidate | 18100.527000, 18116.242000, 18130.800000, 18119.283000, 18118.862000 | 18118.862000 | 0.167080 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.167080 | +0.062891 | 18128.173913 | 18129.951993 | +1.778080 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::intern_symbol_text | -0.888587 | -0.155797 | +0.732790 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 99.429348 | 99.742754 | +0.313406 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.011775 | 0.223732 | +0.235507 |
| rum_core::expr::Symbol::system | 75.031703 | 75.254529 | +0.222826 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | -0.084239 | 0.064312 | +0.148551 |
| rum_core::expr::intern_text | 0.144928 | 0.232790 | +0.087862 |
| 0x00000000001b1220 | 230.066123 | 230.101449 | +0.035326 |
| hashbrown::raw::RawTable<T,A>::reserve_rehash | 85.443841 | 85.447464 | +0.003623 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 763.755435 | 763.753623 | -0.001812 |
| <std::env::Args as core::iter::traits::iterator::Iterator>::next | 0.000000 | 0.000000 | +0.000000 |
| __cxa_atexit | 0.000000 | 0.000000 | +0.000000 |
| _mi_arena_id_none | 0.000000 | 0.000000 | +0.000000 |
| _mi_os_alloc_aligned | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::model::definitions::down_value_dispatch::DownValueDispatchIndexCache::rebuild_down_values_preserving_tags | 0.000000 | 0.000000 | +0.000000 |
| 0x00000000053e0320 | 0.000000 | 0.000000 | +0.000000 |
| pthread_mutex_destroy | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000005405320 | 0.000000 | 0.000000 | +0.000000 |
| _IO_file_fopen | 0.000000 | 0.000000 | +0.000000 |
| 0x000000000001b870 | 0.000000 | 0.000000 | +0.000000 |
| core::ptr::drop_in_place<rum_syntax::trivia::LexedSource> | 0.000000 | 0.000000 | +0.000000 |

`lazy-depth-map-first-8`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 130373.765000, 130584.336000, 130400.808000, 130190.778000, 130403.970000 | 130400.808000 | 0.301806 |
| candidate | 130368.648000, 130186.319000, 130393.093000, 130389.758000, 130793.910000 | 130389.758000 | 0.465981 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.465981 | -0.008474 | 130448.078431 | 130423.130719 | -24.947712 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| hashbrown::map::HashMap<K,V,S,A>::insert | 717.562092 | 703.366013 | -14.196078 |
| rum_core::expr::intern_text | 9.215686 | -0.901961 | -10.117647 |
| rum_core::expr::intern_symbol_text | 2.575163 | -6.823529 | -9.398693 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 4676.176471 | 4679.575163 | +3.398693 |
| rum_core::expr::Symbol::system | 1273.771242 | 1276.901961 | +3.130719 |
| 0x00000000001b1220 | 3112.424837 | 3114.006536 | +1.581699 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.666667 | 0.379085 | +1.045752 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.679739 | 0.287582 | -0.392157 |
| pthread_mutex_destroy | 0.000000 | 0.000000 | +0.000000 |
| rum_syntax::lexer::symbols::<impl rum_syntax::lexer::Lexer>::lex_symbol | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::eval_core::stability::hold_all_literal_args_are_stable | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::session::session_api::<impl rum_runtime::runtime::model::session::Session>::install_system_own_value | 0.000000 | 0.000000 | +0.000000 |
| 0x000000000002f060 | 0.000000 | 0.000000 | +0.000000 |
| hashbrown::map::HashMap<K,V,S,A>::remove | 69.000000 | 69.000000 | +0.000000 |
| rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable'2 | 4917.000000 | 4917.000000 | +0.000000 |
| 0x000000000001b870 | 0.000000 | 0.000000 | +0.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::kernel_data::text_rendering::transparent_hold_form_expr::Work> | 102.000000 | 102.000000 | +0.000000 |
| rum_runtime::runtime::call_preparation::<impl rum_runtime::runtime::model::session::Session>::normalize_call_args_by_attributes | 600.000000 | 600.000000 | +0.000000 |
| rum_runtime::runtime::eval_dispatch::<impl rum_runtime::runtime::model::session::Session>::try_dispatch_builtin_call'2 | 1107.000000 | 1107.000000 | +0.000000 |
| rum_runtime::runtime::definitions::definition_analysis_helpers::collect_pattern_name_mapping | 0.000000 | 0.000000 | +0.000000 |

`lazy-depth-select-restart-8`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 179153.219000, 179688.730000, 179981.911000, 179410.150000, 179123.041000 | 179410.150000 | 0.478719 |
| candidate | 179389.086000, 179443.625000, 179421.705000, 179952.122000, 179401.466000 | 179421.705000 | 0.313806 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.478719 | +0.006441 | 179117.954955 | 179682.693694 | +564.738739 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2 | 2684.117117 | 2996.000000 | +311.882883 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 8677.486486 | 8948.378378 | +270.891892 |
| rum_core::expr::intern_symbol_text | 7.693694 | 1.432432 | -6.261261 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 1495.180180 | 1489.198198 | -5.981982 |
| 0x00000000001b1220 | 3426.000000 | 3422.576577 | -3.423423 |
| rum_core::expr::Symbol::system | 439.522523 | 442.621622 | +3.099099 |
| rum_core::expr::intern_text | -3.621622 | -6.027027 | -2.405405 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 1.108108 | -0.810811 | -1.918919 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.000000 | -1.144144 | -1.144144 |
| core::fmt::Formatter::pad_integral::write_prefix | 0.000000 | 0.000000 | +0.000000 |
| _mi_thread_id | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::substitution::lexical_substitution::substitute_lexical_bindings | 1581.000000 | 1581.000000 | +0.000000 |
| mi_page_fresh_alloc | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::profiling::scopes::lexical_profile_context | 560.000000 | 560.000000 | +0.000000 |
| rum_syntax::parser::front::Parser<_>::parse_function_expr | 0.000000 | 0.000000 | +0.000000 |
| __rust_alloc | 326.000000 | 326.000000 | +0.000000 |
| memcpy | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::model::definitions::down_value_dispatch::<impl rum_runtime::runtime::model::definitions::SymbolDefinition>::note_down_values_rebuilt | 0.000000 | 0.000000 | +0.000000 |
| strncasecmp | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::eval_expr_request_from_checkpoint | 0.000000 | 0.000000 | +0.000000 |

`lazy-depth-map-first-64`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 946251.969000, 946103.123000, 946103.774000, 949354.733000, 946030.956000 | 946103.774000 | 0.351312 |
| candidate | 946257.249000, 946178.573000, 944596.526000, 946179.903000, 944436.730000 | 946178.573000 | 0.192408 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.351312 | +0.007906 | 946215.952381 | 945976.476190 | -239.476190 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::Symbol::system | 9670.142857 | 9499.190476 | -170.952381 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 32988.904762 | 33062.571429 | +73.666667 |
| 0x00000000001b1220 | 24503.333333 | 24438.333333 | -65.000000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 5652.714286 | 5615.238095 | -37.476190 |
| rum_core::expr::intern_text | 14.571429 | -11.047619 | -25.619048 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 6.000000 | -3.142857 | -9.142857 |
| rum_core::expr::intern_symbol_text | -9.476190 | -14.238095 | -4.761905 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 1.761905 | 1.571429 | -0.190476 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::reject_overlapping_evaluation_request | 0.000000 | 0.000000 | +0.000000 |
| rum_core::expr::IntValue::to_bigint | 4920.000000 | 4920.000000 | +0.000000 |
| 0x000000000002ee10 | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::clear_dynamic_scope_symbol_side_effects | 86.000000 | 86.000000 | +0.000000 |
| rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::spec::checked_integer_range_values | 18.000000 | 18.000000 | +0.000000 |
| 0x000000000001ae40 | 0.000000 | 0.000000 | +0.000000 |
| _mi_prim_thread_init_auto_done | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::session::session_eval::<impl rum_runtime::runtime::model::session::Session>::eval_input_with_terminated_handling | 0.000000 | 0.000000 | +0.000000 |
| rum_host::session::HostSession::eval_request_unprotected | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000005405200 | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::call_preparation::plan_call_argument_preparation | 4861.000000 | 4861.000000 | +0.000000 |
| strncasecmp_l | 0.000000 | 0.000000 | +0.000000 |

`lazy-depth-map-first-256`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 3723170.094000, 3736071.120000, 3722648.574000, 3722823.735000, 3736086.793000 | 3723170.094000 | 0.360935 |
| candidate | 3723174.764000, 3723170.800000, 3736000.395000, 3736267.735000, 3722821.888000 | 3723174.764000 | 0.361139 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.361139 | +0.000125 | 3724784.000000 | 3722591.800000 | -2192.200000 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::Symbol::system | 40337.800000 | 38211.600000 | -2126.200000 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 129853.800000 | 129999.400000 | +145.600000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 22665.400000 | 22557.200000 | -108.200000 |
| rum_core::expr::intern_symbol_text | 44.800000 | -13.600000 | -58.400000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 23.400000 | -5.600000 | -29.000000 |
| 0x00000000001b1220 | 97844.000000 | 97871.000000 | +27.000000 |
| rum_core::expr::intern_text | 52.000000 | 28.200000 | -23.800000 |
| 0x000000000001b870 | 5.600000 | -8.400000 | -14.000000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | -0.800000 | -6.000000 | -5.200000 |
| rum_runtime::runtime::definitions::definition_analysis_helpers::substitute_pattern_name_mapping'2 | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::resolution::match_normalization::<impl rum_runtime::runtime::model::session::Session>::normalize_match_call_changed_recursive_bounded | 0.000000 | 0.000000 | +0.000000 |
| _setjmp | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::session::session_api::<impl rum_runtime::runtime::model::session::Session>::cancellation_observer | 12.000000 | 12.000000 | +0.000000 |
| isatty | 0.000000 | 0.000000 | +0.000000 |
| _mi_theap_realloc_zero | 63.000000 | 63.000000 | +0.000000 |
| __ctype_init | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000000018ae0 | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000000026f20 | 0.000000 | 0.000000 | +0.000000 |
| wcschr | 0.000000 | 0.000000 | +0.000000 |
| 0x0000000004070150 | 0.000000 | 0.000000 | +0.000000 |

`literal-integer`

| Arm | Five Ir/iteration slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 293.948700, 294.000750, 294.229200, 294.050050, 294.075600 | 294.050050 | 0.095392 |
| candidate | 294.027950, 293.974750, 293.913750, 294.002800, 293.937000 | 293.974750 | 0.038847 |

| Floor % | Median delta % | Callgrind base | Callgrind candidate | Callgrind delta |
| --- | --- | --- | --- | --- |
| 0.095392 | -0.025608 | 294.474500 | 293.007000 | -1.467500 |

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::Symbol::system | 0.581500 | -0.274500 | -0.856000 |
| rum_core::expr::intern_symbol_text | 0.278000 | 0.065500 | -0.212500 |
| hashbrown::map::HashMap<K,V,S,A>::insert | -0.219000 | -0.417500 | -0.198500 |
| rum_core::expr::intern_text | -0.139500 | -0.301000 | -0.161500 |
| 0x00000000001b1220 | 0.109500 | -0.036500 | -0.146000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.162000 | -0.029000 | +0.133000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.026000 | 0.000000 | -0.026000 |
| 0x000000000002fb70 | 0.000000 | 0.000000 | +0.000000 |
| wmemset | 0.000000 | 0.000000 | +0.000000 |
| _mi_arenas_free | 0.000000 | 0.000000 | +0.000000 |
| rum_stdlib::control_flow::enforce_static_multi_iterator_product | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_host_interrupt_if_due | 0.000000 | 0.000000 | +0.000000 |
| mi_arenas_try_alloc | 0.000000 | 0.000000 | +0.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::session::session_state::MatcherNode> | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::resolution::match_normalization::<impl rum_runtime::runtime::model::session::Session>::normalize_match_call_changed_recursive_bounded | 0.000000 | 0.000000 | +0.000000 |
| pthread_attr_destroy | 0.000000 | 0.000000 | +0.000000 |
| _mi_strnicmp | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::bump_evaluation_epoch_for_symbol | 0.000000 | 0.000000 | +0.000000 |
| _mi_options_post_init | 0.000000 | 0.000000 | +0.000000 |
| rum_runtime::runtime::definitions::rhs_guards::<impl rum_runtime::runtime::model::session::Session>::compile_down_value_matcher | 0.000000 | 0.000000 | +0.000000 |

The six nonpositive exact-count cases are classified as layout/run variation on this repaired binary, with their newly earned evidence above: `lazy-table-construct-full`, `replaceall-simple`, `lazy-depth-map-first-8`, `lazy-depth-map-first-64`, `lazy-depth-map-first-256`, and `literal-integer`. All nine probe bodies are ordinary Do loops without input. Their per-iteration paths never enter the changed resume driver, legacy/typed queue insertion, or fault unwind. The changed public request-boundary check runs once per file and cancels in the N/2N slope. The recorded changed-function slopes are empty for every investigated label. Both the source bodies and the exact per-function profile evidence are preserved; classification is not inherited from the preceding candidate.

The previous ordinary offender, `lazy-table-construct-full`, now has median perf delta -0.065729%, each-arm floor 0.394672%, and exact delta -28.149218 instructions per iteration. Its layout classification is re-earned. The initial literal-integer perf pair (+0.054192%, +0.095748%) is disclosed even though the fresh five-per-arm medians give -0.025608% and exact delta -1.467500.

**Three ordinary exact-count defects remain**, following the amendment’s rule that positive Callgrind work is not waved away by the perf spread. All three median perf deltas fit their newly measured own-arm floors, but their exact instruction counts increase:

- `lazy-first-small`: +31.857732 Ir/iteration. Largest increase is `Symbol::system` (+49.536082), partly offset by symbol interning and hashing differences.
- `lazy-depth-map-build-8`: +1.778080 Ir/iteration. Small distributed increases in symbol interning (+0.732790), HashMap insertion (+0.313406), initialization and dispatch.
- `lazy-depth-select-restart-8`: +564.738739 Ir/iteration. Two `cached_expr_stability_with_bloom` profile identities contribute +311.882883 and +270.891892, partly offset elsewhere.

Those source functions are unchanged by S2. This attribution describes where the measured extra work occurred; it does not establish that new S2 source instructions caused it. It nevertheless remains a reported defect under the explicit positive-exact-count rule. No extra samples were taken to seek a different sign. The complete five-arm spreads, ten raw N/2N samples per label, fixed Callgrind budgets and per-function diffs are under `perf/instructions/*/exceptions/` and `ordinary-callgrind/`.

Full core-language microbenchmark: both preserved base and final candidate, profiling off, scale 1, seven repetitions, retained-module GC batches 3, in-kernel `AbsoluteTiming`/monotonic `Instant`. These 120 timing rows are indications, not substituted for the instruction gate. No concurrent build or test ran during collection. Positive deltas, including hot labels and neighbors, remain visible.

| Label | Base seconds | Candidate seconds | Delta % |
| --- | ---: | ---: | ---: |
| association-part-key-read-width4000 | 0.002624500 | 0.002181300 | -16.887026 |
| association-part-key-read-width32000 | 0.009658570 | 0.008137620 | -15.747155 |
| association-part-positional-read-width4000 | 0.000030220 | 0.000030721 | +1.657842 |
| association-part-positional-read-width32000 | 0.000030443 | 0.000030676 | +0.765365 |
| loop-empty | 0.000253607 | 0.000261051 | +2.935250 |
| do-count-null | 0.000252239 | 0.000253129 | +0.352840 |
| do-iterator-null | 0.000058977 | 0.000056231 | -4.656052 |
| table-count-null | 0.000077323 | 0.000074442 | -3.725929 |
| table-iterator-null | 0.000018159 | 0.000018093 | -0.363456 |
| table-iterator-symbol | 0.000006183 | 0.000006041 | -2.296620 |
| literal-integer | 0.000084130 | 0.000088550 | +5.253774 |
| literal-string | 0.000094539 | 0.000097268 | +2.886639 |
| symbol-ownvalue-read | 0.000218649 | 0.000218496 | -0.069975 |
| symbol-ownvalue-guarded-read | 0.000182998 | 0.000187668 | +2.551940 |
| set-ownvalue | 0.000262136 | 0.000263432 | +0.494400 |
| list-literal-small | 0.000054268 | 0.000054389 | +0.222967 |
| head-list | 0.000918117 | 0.000917929 | -0.020477 |
| length-list | 0.000908704 | 0.000914465 | +0.633980 |
| part-list | 0.001071000 | 0.001096840 | +2.412698 |
| sameq-integer | 0.000622428 | 0.000641047 | +2.991350 |
| sameq-list | 0.000609671 | 0.000615323 | +0.927057 |
| if-true | 0.000700587 | 0.000711639 | +1.577534 |
| module-local | 0.000722442 | 0.000715292 | -0.989699 |
| module-chain-escape-small | 0.000050784 | 0.000047876 | -5.726213 |
| module-chain-escape-large | 0.000095375 | 0.000093575 | -1.887287 |
| block-local | 0.000463482 | 0.000463312 | -0.036679 |
| block-symbol-only-repeat | 0.000306744 | 0.000311249 | +1.468651 |
| block-init-null-repeat | 0.000357174 | 0.000362223 | +1.413597 |
| block-init-read-repeat | 0.000414433 | 0.000413912 | -0.125714 |
| with-local | 0.000479456 | 0.000474985 | -0.932515 |
| with-deep-body-small | 0.001770370 | 0.001785860 | +0.874958 |
| table-small | 0.000183419 | 0.000187143 | +2.030324 |
| table-iterator-plus | 0.000035784 | 0.000035571 | -0.595238 |
| table-iterator-pair-list | 0.000059596 | 0.000060724 | +1.892744 |
| do-small | 0.000899309 | 0.000884591 | -1.636590 |
| d-nested-sin-small | 0.005398870 | 0.005373330 | -0.473062 |
| downvalue-exact-hit | 0.000784067 | 0.000775465 | -1.097100 |
| downvalue-exact-miss | 0.000546378 | 0.000553323 | +1.271098 |
| downvalue-pattern-hit | 0.000958885 | 0.000973233 | +1.496321 |
| downvalue-pattern-miss | 0.000282840 | 0.000288368 | +1.954462 |
| downvalue-exact-miss-repeat | 0.001224790 | 0.001305660 | +6.602765 |
| downvalue-pattern-miss-repeat | 0.001267620 | 0.001347570 | +6.307095 |
| downvalue-guarded-hit | 0.001893770 | 0.001918550 | +1.308501 |
| downvalue-guarded-decline-fallback | 0.001400040 | 0.001415600 | +1.111397 |
| upvalue-guarded-hit | 0.002320620 | 0.002342110 | +0.926046 |
| subvalue-unguarded-hit | 0.001390480 | 0.001393010 | +0.181952 |
| subvalue-unguarded-miss | 0.000770293 | 0.000778183 | +1.024286 |
| subvalue-guarded-hit | 0.002426960 | 0.002436650 | +0.399265 |
| default-unguarded-hit | 0.001299890 | 0.001330610 | +2.363277 |
| default-unguarded-miss | 0.002270510 | 0.001076560 | -52.585102 |
| default-guarded-hit | 0.003389420 | 0.001671870 | -50.673862 |
| matchq-integer-success | 0.000596269 | 0.000289624 | -51.427292 |
| matchq-integer-fail | 0.000612501 | 0.000289913 | -52.667343 |
| matcher-single-tiny | 0.001402810 | 0.000670346 | -52.214056 |
| matcher-single-wide-small | 0.000089562 | 0.000040266 | -55.041201 |
| matcher-single-wide-large | 0.000176304 | 0.000078989 | -55.197273 |
| replaceall-simple | 0.000739575 | 0.000362723 | -50.955211 |
| rule-rhs-deep-small | 0.006271320 | 0.002931900 | -53.249077 |
| rewrite-guarded-hit | 0.002304840 | 0.001793700 | -22.176811 |
| rewrite-guarded-decline-fallback | 0.002536400 | 0.002563330 | +1.061741 |
| cases-wide-no-match | 0.000149516 | 0.000144512 | -3.346799 |
| count-wide-no-match | 0.000140384 | 0.000137200 | -2.268065 |
| position-wide-no-match | 0.000177992 | 0.000171844 | -3.454088 |
| deletecases-wide-no-match | 0.000170118 | 0.000167040 | -1.809332 |
| deletecases-wide-association-no-match | 0.000429755 | 0.000427904 | -0.430711 |
| deletecases-leaf-delete-small | 0.002206650 | 0.002082280 | -5.636145 |
| replace-wide-no-match | 0.000081762 | 0.000078796 | -3.627602 |
| replace-leaf-rebuild-small | 0.002703820 | 0.002557760 | -5.401987 |
| take-multispec-small | 0.001655020 | 0.001624580 | -1.839253 |
| replacepart-deep-path-small | 0.001323120 | 0.001320370 | -0.207842 |
| replacepart-wide-rule-list | 0.000297793 | 0.000304533 | +2.263317 |
| replacepart-guarded-hit | 0.000531847 | 0.000529482 | -0.444677 |
| replacepart-guarded-decline-fallback | 0.000581252 | 0.000581144 | -0.018581 |
| string-repeat-small | 0.000185021 | 0.000188271 | +1.756557 |
| character-range-small | 0.000137316 | 0.000131772 | -4.037403 |
| stringtrim-literal-list-4000 | 0.000685803 | 0.000657391 | -4.142881 |
| stringtrim-literal-list-32000 | 0.005458090 | 0.005060780 | -7.279286 |
| stringtrim-single-literal-4000 | 0.000664108 | 0.000637236 | -4.046330 |
| stringtrim-single-literal-32000 | 0.005450410 | 0.005026580 | -7.776112 |
| stringtrim-whitespace-4000 | 0.000652730 | 0.000628755 | -3.673035 |
| stringtrim-whitespace-32000 | 0.005437040 | 0.005014830 | -7.765439 |
| stringreplace-single-literal-unguarded | 0.000383881 | 0.000383488 | -0.102375 |
| stringreplace-pattern-unguarded | 0.001050190 | 0.001048090 | -0.199964 |
| stringreplace-guarded-hit | 0.000692027 | 0.000685704 | -0.913693 |
| stringreplace-guarded-decline-fallback | 0.000827194 | 0.000817673 | -1.151000 |
| appendto-list-incremental-small | 0.000479439 | 0.000476031 | -0.710831 |
| appendto-list-incremental-large | 0.001694420 | 0.001657600 | -2.173015 |
| appendto-part-outer1000-small | 0.002317270 | 0.002174310 | -6.169329 |
| appendto-part-outer1000-large | 0.004683360 | 0.004416560 | -5.696765 |
| association-set-key-incremental-small | 0.001489470 | 0.001463600 | -1.736859 |
| association-set-key-incremental-large | 0.005624560 | 0.005453760 | -3.036682 |
| association-small-literal-repeat | 0.000763050 | 0.000741282 | -2.852762 |
| merge-nested-small | 0.001161220 | 0.001163950 | +0.235098 |
| set-part-fixed100-len100 | 0.000153682 | 0.000145251 | -5.486004 |
| set-part-fixed100-len800 | 0.000783930 | 0.000599152 | -23.570727 |
| set-part-fill-small | 0.000503515 | 0.000431536 | -14.295304 |
| set-part-fill-large | 0.001768810 | 0.001373440 | -22.352316 |
| upvalue-candidates-tiny | 0.001964330 | 0.001919690 | -2.272531 |
| upvalue-candidates-wide-small | 0.000157316 | 0.000150122 | -4.572961 |
| upvalue-candidates-wide-large | 0.000311185 | 0.000298689 | -4.015618 |
| sort-callback-small | 0.000162987 | 0.000163115 | +0.078534 |
| sort-callback-large | 0.000373561 | 0.000373842 | +0.075222 |
| sortby-callback-small | 0.000194975 | 0.000194129 | -0.433902 |
| sortby-callback-large | 0.000442214 | 0.000439506 | -0.612373 |
| reversesort-callback-small | 0.000157129 | 0.000158876 | +1.111825 |
| reversesort-callback-large | 0.000353383 | 0.000359200 | +1.646089 |
| keysort-callback-small | 0.000083271 | 0.000083035 | -0.283412 |
| keysort-callback-large | 0.000182544 | 0.000182569 | +0.013695 |
| keysortby-callback-small | 0.000044724 | 0.000044574 | -0.335390 |
| keysortby-callback-large | 0.000097222 | 0.000096399 | -0.846516 |
| sort-default-wide | 0.000019053 | 0.000016469 | -13.562169 |
| sort-symbol-default-width4000 | 0.000313493 | 0.000296670 | -5.366308 |
| sort-symbol-default-width16000 | 0.001326250 | 0.001234120 | -6.946654 |
| sortby-default-wide | 0.000502383 | 0.000503199 | +0.162426 |
| reversesort-default-wide | 0.000019262 | 0.000016750 | -13.041221 |
| keysort-default-wide | 0.000373866 | 0.000371995 | -0.500447 |
| keysortby-default-wide | 0.000876106 | 0.000866425 | -1.105003 |
| valueq-undefined-symbol-definitions-2000 | 0.000007850 | 0.000007889 | +0.496815 |
| valueq-undefined-symbol-definitions-8000 | 0.000007863 | 0.000007829 | -0.432405 |
| module-retained-gensym-gc-batch | 0.000062717 | 0.000064925 | +3.520577 |

43 positive timing deltas; 3 exceed +5%. They are reported as indications, without discarding any label or replacing the instruction verdict.

Inventory replay: 79 host identities and 59 self-reaching types remain unchanged. UnknownConstruct is 7712 → 7711 because the deleted `Entry::resume` decode closure no longer captures anything. All dispositions/witnesses are preserved, including old rows 56–58 until R. The adapter witness still requires the exact four constructs. The following is the complete redispatch ownership-row transition; the prior S2 transitions remain in the preceding execution reports.

```text
unknown_attributes.tsv: 1514 -> 1514 lines
unknown_captures.tsv: 2955 -> 2954 lines
unknown_macros.tsv: 3149 -> 3149 lines
unknown_storage.tsv: 94 -> 94 lines
host_payloads.tsv: 79 -> 79 lines
self_reaching.tsv: 59 -> 59 lines
host_payload_ledger.tsv: 80 -> 80 lines
```

```diff
--- unknown_attributes.tsv before
+++ unknown_attributes.tsv replay
@@ -912,10 +912,10 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	574	5	unresolved attribute/expansion error ("uncaught exception: {exception}")
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/lexical_presence.rs	4	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs	53	1	unresolved attribute/expansion derive (Clone , Copy)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	28	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , PartialOrd , Ord)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	46	5	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	52	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	227	13	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	26	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , PartialOrd , Ord)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	44	5	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	50	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	231	13	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/fault.rs	7	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , rum_derive :: FrameRootTrace)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/fault.rs	32	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/fault.rs	45	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
@@ -926,7 +926,7 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/output.rs	8	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/trace.rs	12	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/adapter.rs	25	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	1	unresolved attribute/expansion derive (Clone)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	8	1	unresolved attribute/expansion derive (Clone)
 UnknownConstruct	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs	8	1	unresolved attribute/expansion derive (Clone , Copy , FrameRootTrace)
 UnknownConstruct	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs	15	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
 UnknownConstruct	crates/rum-runtime/src/runtime/tree_walkers/count_like/options.rs	10	1	unresolved attribute/expansion derive (FrameRootTrace)
--- unknown_captures.tsv before
+++ unknown_captures.tsv replay
@@ -1761,11 +1761,11 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/lexical_substitution/iterative.rs	864	37	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/slot_substitution.rs	113	26	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/slot_substitution.rs	129	23	closure captures: || (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	204	27	closure captures: |frame| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	209	27	closure captures: |frame| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	250	33	closure captures: |owner| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	705	30	closure captures: |failure| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	712	26	closure captures: |failure| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	208	27	closure captures: |frame| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	213	27	closure captures: |frame| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	254	33	closure captures: |owner| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	710	30	closure captures: |failure| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	758	34	closure captures: |failure| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	36	25	closure captures: |_| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	46	25	closure captures: |_| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	437	22	closure captures: |path| (capture types/modes unresolved)
@@ -1777,7 +1777,6 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/action.rs	42	48	closure captures: |value| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/adapter.rs	86	18	closure captures: |value| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/channel.rs	62	22	closure captures: |actual| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	36	82	closure captures: |actual| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs	18	28	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs	39	28	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs	60	28	closure captures: || (capture types/modes unresolved)
--- unknown_macros.tsv before
+++ unknown_macros.tsv replay
@@ -1781,16 +1781,16 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/collections.rs	93	17	unexpanded macro panic! (ends at 93:99)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/collections.rs	103	13	unexpanded macro vec! (ends at 107:14)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/collections.rs	133	17	unexpanded macro panic! (ends at 133:95)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	254	9	unexpanded macro assert! (ends at 257:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	272	9	unexpanded macro assert! (ends at 275:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	580	9	unexpanded macro debug_assert! (ends at 580:57)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	588	9	unexpanded macro assert! (ends at 591:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	611	9	unexpanded macro debug_assert! (ends at 614:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	677	17	unexpanded macro debug_assert! (ends at 677:66)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	788	1	unexpanded macro macro_rules! (ends at 805:2)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	258	9	unexpanded macro assert! (ends at 261:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	276	9	unexpanded macro assert! (ends at 279:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	585	9	unexpanded macro debug_assert! (ends at 585:57)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	593	9	unexpanded macro assert! (ends at 596:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	616	9	unexpanded macro debug_assert! (ends at 619:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	682	17	unexpanded macro debug_assert! (ends at 682:66)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	823	1	unexpanded macro macro_rules! (ends at 840:2)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	39	13	unexpanded macro debug_assert! (ends at 39:60)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	65	28	unexpanded macro matches! (ends at 65:91)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	22	13	unexpanded macro unreachable! (ends at 22:27)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	25	13	unexpanded macro unreachable! (ends at 25:27)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	9	9	unexpanded macro panic! (ends at 9:63)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	11	5	unexpanded macro assert! (ends at 14:6)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	24	9	unexpanded macro panic! (ends at 24:74)
--- unknown_storage.tsv before
+++ unknown_storage.tsv replay
@@ -70,7 +70,7 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	298	50	rum_runtime::runtime::session::session_state::LoaderSnapshot.gc_candidate_gensyms/std::collections::HashSet[Own]/arg1: foreign default hasher representation std::collections::hash_map::RandomState
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	302	50	rum_runtime::runtime::session::session_state::LoaderSnapshot.package_search_paths/std::vec::Vec[Own]/arg0: foreign representation std::path::PathBuf: PathBuf
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	332	43	rum_runtime::runtime::session::session_state::ReapBucketGroup.tag_positions/std::option::Option[Own]/arg0/std::collections::HashMap[Own]/arg2: foreign default hasher representation std::collections::hash_map::RandomState
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	47	18	rum_runtime::runtime::suspension::core::SuspensionSlot.fault_probe: unresolved conditional storage: fault :: FaultProbe
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	45	18	rum_runtime::runtime::suspension::core::SuspensionSlot.fault_probe: unresolved conditional storage: fault :: FaultProbe
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/fault.rs	48	18	rum_runtime::runtime::suspension::core::fault::FaultProbe.occurrences: unresolved conditional storage: Arc < std :: sync :: atomic :: AtomicU64 >
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/fault.rs	53	9	conditional public input signature occurrences
 UnknownConstruct	crates/rum-stdlib/src/adaptive_set.rs	8	13	rum_stdlib::adaptive_set::AdaptiveSet.Indexed.0/std::collections::HashSet[Own]/arg1: foreign default hasher representation std::collections::hash_map::RandomState
--- host_payloads.tsv before
+++ host_payloads.tsv replay
@@ -49,11 +49,11 @@
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	269	16	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::resume_eval_request_with_input(input): opaque owned payload impl Into < String >
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	410	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_file_request(path): opaque owned payload impl AsRef < Path >
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	445	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_accepted_file_request(path): opaque owned payload impl AsRef < Path >
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	124	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	126	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	136	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@129::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	147	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@129::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	519	57	public input rum_runtime::runtime::suspension::core::impl(Session)@453::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	122	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	124	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	134	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@127::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	145	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@127::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	523	57	public input rum_runtime::runtime::suspension::core::impl(Session)@457::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core/queue.rs	6	12	rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	19	15	rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	21	14	rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer
@@ -64,8 +64,8 @@
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	66	17	public input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@30::finish_work(finish): opaque owned payload impl FnOnce (T) -> F
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	9	13	rum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	32	59	public input rum_runtime::runtime::suspension::typed::chain::impl(FrameChain < I , O >)@20::then(next): opaque owned generic parameter F : TypedFrame < Input = O >
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	13	12	rum_runtime::runtime::suspension::typed::entry::Entry.Legacy.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn SuspensionFrame
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	14	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	10	12	rum_runtime::runtime::suspension::typed::entry::Entry.Legacy.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn SuspensionFrame
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs	18	16	public input rum_runtime::runtime::suspension::typed::resume_token::impl(ContinueToken < '_ , F >)@13::suspend(fresh): opaque owned generic parameter F : TypedFrame
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/witness.rs	20	16	public input rum_runtime::runtime::suspension::typed::witness::impl(SlotWitness < I >)@15::suspend(frame): opaque owned generic parameter F : TypedFrame < Input = I >
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/witness.rs	26	57	public input rum_runtime::runtime::suspension::typed::witness::impl(SlotWitness < I >)@15::chain(first): opaque owned generic parameter F : TypedFrame < Input = I >
--- host_payload_ledger.tsv before
+++ host_payload_ledger.tsv replay
@@ -51,11 +51,11 @@
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	410	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_file_request(path): opaque owned payload impl AsRef < Path >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	445	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_accepted_file_request(path): opaque owned payload impl AsRef < Path >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core/queue.rs	6	12	rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	124	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	126	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer	Unbounded	rum_runtime::runtime::suspension::core::PureFrameFinalizer => rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer (impl at crates/rum-runtime/src/runtime/debug_tools/reap.rs:284:1); rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer.rendered -> rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator ; rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator.prefix -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	136	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@129::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	147	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@129::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	519	57	public input rum_runtime::runtime::suspension::core::impl(Session)@453::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	122	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	124	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer	Unbounded	rum_runtime::runtime::suspension::core::PureFrameFinalizer => rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer (impl at crates/rum-runtime/src/runtime/debug_tools/reap.rs:284:1); rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer.rendered -> rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator ; rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator.prefix -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	134	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@127::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	145	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@127::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	523	57	public input rum_runtime::runtime::suspension::core::impl(Session)@457::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-syntax/src/ast.rs	16	27	public input rum_syntax::ast::impl(SyntaxExpr)@11::integer(value): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-syntax/src/ast.rs	20	26	public input rum_syntax::ast::impl(SyntaxExpr)@11::string(value): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-syntax/src/ast.rs	24	26	public input rum_syntax::ast::impl(SyntaxExpr)@11::symbol(value): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
@@ -74,7 +74,7 @@
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	9	13	rum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	Unbounded	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)	TODO.md#standing-engineering-debt	unresolved
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	32	59	public input rum_runtime::runtime::suspension::typed::chain::impl(FrameChain < I , O >)@20::then(next): opaque owned generic parameter F : TypedFrame < Input = O >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/witness.rs	26	57	public input rum_runtime::runtime::suspension::typed::witness::impl(SlotWitness < I >)@15::chain(first): opaque owned generic parameter F : TypedFrame < Input = I >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	13	12	rum_runtime::runtime::suspension::typed::entry::Entry.Legacy.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	14	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	Unbounded	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	10	12	rum_runtime::runtime::suspension::typed::entry::Entry.Legacy.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	Unbounded	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)	TODO.md#standing-engineering-debt	unresolved
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs	18	16	public input rum_runtime::runtime::suspension::typed::resume_token::impl(ContinueToken < '_ , F >)@13::suspend(fresh): opaque owned generic parameter F : TypedFrame	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/witness.rs	20	16	public input rum_runtime::runtime::suspension::typed::witness::impl(SlotWitness < I >)@15::suspend(frame): opaque owned generic parameter F : TypedFrame < Input = I >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
```

Historical frontier: 932 = 10 RET001 / 895 RET002 / 27 RET003. Exactly one diagnostic moves with the same `take_for_abandon` receiver from `core.rs:473` to `:477`; the other 931 full diagnostic rows are identical. The full paired fields and witness are in `frontier-classified-diff.json`. Every unresolved assertion, wrapper count and historical ACCEPTANCE_BASELINE print remains unchanged.

```json
[
  {
    "before": {
      "kind": "RET002",
      "file": "crates/rum-runtime/src/runtime/suspension/core.rs",
      "start_line": "473",
      "start_col": "23",
      "end_line": "473",
      "end_col": "66",
      "target": "",
      "reason": "method syntax: method trait scope contains an unexpanded item",
      "witness": "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::take_accounted_pending_suspension_for_abandon"
    },
    "after": {
      "kind": "RET002",
      "file": "crates/rum-runtime/src/runtime/suspension/core.rs",
      "start_line": "477",
      "start_col": "23",
      "end_line": "477",
      "end_col": "66",
      "target": "",
      "reason": "method syntax: method trait scope contains an unexpanded item",
      "witness": "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::take_accounted_pending_suspension_for_abandon"
    },
    "same_source_line": "        let pending = self.suspension_access().take_for_abandon();",
    "classification": "same take_for_abandon receiver shifted within core.rs; no unresolved boundary removed"
  }
]
```

All 68 contract fingerprints change only in their final fingerprint field; the first 18 contract fields and every precondition/strength/exclusion remain identical. The final source context is `fnv1a64:8c2d6136fd488bfd`. Complete transitions:

```tsv
id	before	after
B01	fnv1a64:42f7239388cb4603	fnv1a64:25075e02e203401c
B02	fnv1a64:2ee65d264197863a	fnv1a64:264ce647d04d9a95
B03	fnv1a64:9f215e803dc66919	fnv1a64:b062064787c2940a
B04	fnv1a64:d7146fef7f126ddb	fnv1a64:d28d79a1162c7f04
B05	fnv1a64:dbeabca2c03c1f2e	fnv1a64:1220b4be927f2529
B06	fnv1a64:5cce1318facc713f	fnv1a64:fe6a3e808a347348
B07	fnv1a64:ac09184eb31d2fde	fnv1a64:20508254576412f9
B08	fnv1a64:41992dc7582920cd	fnv1a64:3f190f5d14f1ef0e
B09	fnv1a64:37b6255b7f8a40f7	fnv1a64:deecd86e42d75e50
B10	fnv1a64:a216cd95b0c43af8	fnv1a64:202133c752af0603
B11	fnv1a64:986c214c990693db	fnv1a64:93e52afe3020a504
B12	fnv1a64:f6c2a5575b92d5f7	fnv1a64:9df9586a1edff350
I01	fnv1a64:20d90e90f5833c83	fnv1a64:02e949004ebb369c
I02	fnv1a64:332c3a36b872a72e	fnv1a64:696232528ab5ad29
I03	fnv1a64:84fd3cbe91c4b537	fnv1a64:866a4bdba95d4210
I04	fnv1a64:a6ecc54cb7b229da	fnv1a64:a07f23d91522f4b5
P01	fnv1a64:1a8e0cebde1df2df	fnv1a64:6659a2daab01fde8
P02	fnv1a64:369169bb4a54c048	fnv1a64:41d7c113df718d33
P03	fnv1a64:1059b45489a8e3a1	fnv1a64:4334b9873a46e022
P04	fnv1a64:23c17b3bc640a23a	fnv1a64:1b28045d54f6b695
P05	fnv1a64:c6a602bf18fcaf93	fnv1a64:4076bb3f0258c3cc
P06	fnv1a64:4741ca27e7c26f77	fnv1a64:ee787d3aab0f8cd0
P070	fnv1a64:818f6669c637e977	fnv1a64:28c6197c898506d0
P080	fnv1a64:d786d489eaaf6f38	fnv1a64:f986e7bc2a7a61c3
P090	fnv1a64:a17e07c1907e1f7a	fnv1a64:84a8c70e5f5b6855
P100	fnv1a64:54503541883b8b4b	fnv1a64:668799e8c82e05d4
P110	fnv1a64:e2d591ffd6983320	fnv1a64:93bb6e94d2e9eedb
P120	fnv1a64:3b3211bcfe134a6d	fnv1a64:e7222743f9cacd2e
P130	fnv1a64:4fd671abe1d109de	fnv1a64:c41ddbb18617ecf9
P140	fnv1a64:0a7cbde27abe3c02	fnv1a64:7ba176b87048e84d
P150	fnv1a64:bc76fc29ad11cf1a	fnv1a64:55e8b253aac85d75
P071	fnv1a64:186ddb5dabc41c7e	fnv1a64:9529bd1b53f9c299
P081	fnv1a64:3a6720d184b0bd97	fnv1a64:442e9804851de3f0
P091	fnv1a64:16afdac7d89f3c1f	fnv1a64:f9bbd15f323002a8
P101	fnv1a64:dad882cd0bfbd020	fnv1a64:8bbe5f62084d8bdb
P111	fnv1a64:ddd838f974322c27	fnv1a64:5633f81136e339a0
P121	fnv1a64:4584b26a9d150ad4	fnv1a64:79a47eea0ad413ff
P131	fnv1a64:74be9255b5668f5b	fnv1a64:70379c074c80a084
P141	fnv1a64:1eb78ddce6d9827f	fnv1a64:bec6eacc90427008
P151	fnv1a64:1160e927c7038c2d	fnv1a64:788c285809e12e6e
B13	fnv1a64:eadea6a22b692d3f	fnv1a64:8c7ad209bad12f48
R01	fnv1a64:e46484de80c7b26a	fnv1a64:fae47e3ea7883f25
R02	fnv1a64:84308dbd85c66ce1	fnv1a64:53b93ec9a59a38e2
R03	fnv1a64:3b2d4ac703cbc756	fnv1a64:7def3727c9255181
R040	fnv1a64:85026123e7b65b0a	fnv1a64:8b6d82ef536bdfc5
R050	fnv1a64:50770bd48ed18473	fnv1a64:1dbb08a9f0ca482c
R060	fnv1a64:c0a14656c58fedf2	fnv1a64:8ed2e224ad5b8ddd
R070	fnv1a64:a2d4e8eb66527c97	fnv1a64:ac9c601e66bfa2f0
R080	fnv1a64:6df12e3f8736f2da	fnv1a64:67838ccbe4a7bdb5
R090	fnv1a64:58da0904542af28b	fnv1a64:ae034d0d3f249794
R100	fnv1a64:644702d2e79c1399	fnv1a64:7587aa9a31983e8a
R110	fnv1a64:cd8844dd94a985d0	fnv1a64:60bd22fd719ba2eb
R041	fnv1a64:d11809cc4290ec3f	fnv1a64:72b43533d1f8ee48
R051	fnv1a64:2fa035d71dac2bfa	fnv1a64:12caf523ec8974d5
R061	fnv1a64:a9b63009272b965d	fnv1a64:98242efd8858013e
R071	fnv1a64:3167576eaa6989cc	fnv1a64:42b37cb1d745ca47
R081	fnv1a64:103a9e9623e7604b	fnv1a64:2272033d63d9dad4
R091	fnv1a64:6cf9afc74dc8dec2	fnv1a64:83e70c92ef06688d
R101	fnv1a64:f2618a09218d5232	fnv1a64:6489d2d7a73acc9d
R111	fnv1a64:110fb0ebea5be7e5	fnv1a64:1ba105aed2676236
R12	fnv1a64:1cb64782f03eeaa5	fnv1a64:78116991a7a07676
R13	fnv1a64:23019450d77105e9	fnv1a64:14eacce847e91dfa
R14	fnv1a64:6ea843b764ec4b4b	fnv1a64:80dfa85ea4dec5d4
R15	fnv1a64:30f9638b54728acf	fnv1a64:cd2d7fc5cd2f21b8
O010	fnv1a64:6d30ddbc4f8fc3bd	fnv1a64:b0a0d89d7a9da89e
O0200	fnv1a64:05445fb6c9756202	fnv1a64:7669188cbf000e4d
O011	fnv1a64:032a02ded321b272	fnv1a64:d15b9eacbaed525d
O0210	fnv1a64:9b881fd7ef36d885	fnv1a64:3445c05628bb15d6
```

Both certificate discoveries emitted three production-instance marks and four LeaseProbe marks with zero rejects before the stale-record comparison. Only the four emitted default/no-default C01/C02 cells were replayed. All seven other fields remain unchanged. The complete before/after cells are preserved below, with the exact emitted cells in `promoted-hash-true.tsv` and `promoted-hash-false.tsv`. Both final gates pass.

```json
[
  {
    "body": "C01",
    "hash": "true",
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8f5613125b29d0a3\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:066a0073ea944c9f;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:9b881fd7ef36d885;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:032a02ded321b272;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8f5613125b29d0a3\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:ed2095dc78098ecd;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:05445fb6c9756202;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:6d30ddbc4f8fc3bd;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:c0f44bf542aed47f\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:c72bf3205def2d4e;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:3445c05628bb15d6;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:d15b9eacbaed525d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:c0f44bf542aed47f\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:70b610354103aa9c;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:7669188cbf000e4d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:b0a0d89d7a9da89e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  },
  {
    "body": "C02",
    "hash": "true",
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:8c2f209c77a83771\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:be37e108dd42518e;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:20d90e90f5833c83;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:332c3a36b872a72e;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:a6ecc54cb7b229da;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:e46484de80c7b26a;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:84308dbd85c66ce1;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:eadea6a22b692d3f;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:dbeabca2c03c1f2e;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:f477b6a25ab283f5\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:51107d9eae39c3eb;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:02e949004ebb369c;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:696232528ab5ad29;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:a07f23d91522f4b5;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:fae47e3ea7883f25;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:53b93ec9a59a38e2;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:8c7ad209bad12f48;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:1220b4be927f2529;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  },
  {
    "body": "C01",
    "hash": "false",
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:31ef68256444022c\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:0ce6beca150a51e0;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:9b881fd7ef36d885;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:032a02ded321b272;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:31ef68256444022c\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:679409588edb5352;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:05445fb6c9756202;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:6d30ddbc4f8fc3bd;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:d259d359474553b0\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:670d351d8c429be4;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:3445c05628bb15d6;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:d15b9eacbaed525d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:d259d359474553b0\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:e3649b4dbe77beda;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:7669188cbf000e4d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:b0a0d89d7a9da89e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  },
  {
    "body": "C02",
    "hash": "false",
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:83d5ef11247b34d6\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:bbfe4d6aef7d6977;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:20d90e90f5833c83;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:332c3a36b872a72e;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:a6ecc54cb7b229da;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:e46484de80c7b26a;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:84308dbd85c66ce1;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:eadea6a22b692d3f;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:dbeabca2c03c1f2e;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:e0b451918108a632\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:d26a423b44f293ce;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:02e949004ebb369c;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:696232528ab5ad29;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:a07f23d91522f4b5;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:fae47e3ea7883f25;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:53b93ec9a59a38e2;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:8c7ad209bad12f48;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:1220b4be927f2529;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"
  }
]
```

### 7. Stop-loss, deviations and remaining defects

The first repair made measured progress by removing the director-attributed wrappers. Two subsequent experiments made no progress on the named driver/copy mechanism (flat result: +9; in-place boundary projection: +4 instructions per resume versus the retained repair). Both were fully discarded; no-progress counter is 2. The director’s power-off note explicitly closed further tuning. Recovery completed the required verification and measurements without reopening it.

Remaining defects are the measured Entry pop/drop materialization, extra boundary outcome copies and additional result/fault handling, plus the positive unmatched balances in Table/call-repeat and the three ordinary positive exact-count labels. Exact counts, instruction addresses and all signed per-function differences are provided. The full timing panel also records 43 positive deltas; three exceed +5%: downvalue-exact-miss-repeat +6.602765%, downvalue-pattern-miss-repeat +6.307095%, and literal-integer +5.253774%. These timing signals remain disclosed as the brief’s indication-only panel; none was deleted or substituted for an instruction result.

No mandatory final gate was omitted after recovery. The interrupted workspace log is not claimed as a pass; the complete restart exits zero. Certificate discovery stale-cell rejections and deliberate census child failures are separately disclosed. Scratch recovery verification initially compared the pre-format test snapshot; this was resolved against the recorded format gate and all 68 saved whole-source fingerprints before any work resumed. The final source audit passes with 49 modified + 10 new files overall, 14 authorized files changed in this redispatch, no source drift since recovery, original base hashes intact, error sizes equal and all test features absent from release binaries. The report is the remaining handoff; no approval to accept the performance defects is assumed.

No git command was run. No network, added dependency, version/lockfile/toolchain change, new unsafe, production frame conversion, transport flip, transit record, public raw adapter or unapproved inventory disposition was introduced. The two expressly authorized Cargo feature edits are listed above. The protected source hashes and both restored experiment surfaces are checked in `final-audit.json`. The tree is left uncommitted. The complete measurement sources, commands, raw counters, annotation outputs and per-function/address evidence remain under `.temp/brief-608/redispatch-4/`.


## Director amendment — redispatch 5: remove the three attributed residuals

The redispatch-4 report is accepted as evidence. The direct-dispatch repair
stays. All correctness gates pass and all 64 allocation/byte slopes are
equal. The address-level attribution is the right instrument. My power-off
note closed tuning; this amendment reopens it for exactly the three targets
below and nothing else.

**Director verification of the ordinary "exact-count defects".** Callgrind is
exact within one process but not across processes: rum seeds its hashing per
process (std `RandomState` in `model/cache.rs` and elsewhere, interning, the
stability Bloom path), so the same binary gives different slopes. I ran three
fresh processes per arm at your N/2N:

| Label | Base slopes (Ir/iter) | Candidate slopes | Mean delta |
| --- | --- | --- | ---: |
| `lazy-depth-select-restart-8` | 179164.7, 179450.6, 179499.7 (range 335) | 179372.2, 179101.2, 179427.2 (range 326) | −71.5 |
| `lazy-first-small` | 41346.3, 41321.9, 41346.0 (range 24) | 41407.0, 41351.2, 41323.8 (range 83) | +22.6 |

Your +564.7 and +31.9 came from one process per arm and fall inside these
ranges. All three labels run only functions S2 left unchanged. They are
run-to-run variation, not defects, and the rule changes as follows.

**Gate rule for callgrind totals (replaces the single-pair rule).**
- Take at least three fresh processes per arm.
- The delta is the difference of the arm means.
- A total counts as added work only if that delta exceeds the larger of the
  two arms' ranges, or if address-level attribution shows the candidate
  executing code the base does not.
- Address-level attribution is deterministic and decides mechanism
  questions. Totals decide only whether anything is left to explain.

**The resume residuals are real**, because they are address-level. Remove
them:

1. **The legacy pop/drop `Entry` temporary (+10 per legacy frame).**
   - When the queue is in its legacy representation, the legacy arm's pop,
     accounting refresh and drop must run the base's sequence: an
     `Option<Arc<dyn SuspensionFrame>>` popped and dropped, with no `Entry`
     built, copied or re-tested.
   - The mixed representation may pay its own match, kept off the legacy
     route.
   - Keep the source-gate order (pop, refresh, explicit drop). Update the
     `frame_root_trace.rs` patterns only as far as a rename requires.
   - The legacy arm pops the entry that dispatch observed at the front. The
     base relied on the same invariant. State in the report how the code
     keeps it, preferring structure over an assertion. A typed entry must
     never be silently dropped on the legacy route.
2. **The boundary outcome copy (+33 per resume) and the eager fault-word
   load/clear (+4 per resume).**
   - On the no-fault path, the outcome must reach
     `finalize_top_level_eval_value` exactly as it did in the base.
   - The latch check reads only the discriminant. Taking and projecting the
     fault is a cold, out-of-line path.
   - Do not take the outcome's address on the hot path. Experiment a3's
     `&mut outcome` saved 56 instructions in the boundary and added 60 in two
     new `memcpy` calls.
   - One form that keeps the hot path identical to the base: test the latch
     before finalization, and send the faulted case to a cold function that
     consumes the outcome and returns the projected error.
   - Apply the same treatment to every boundary that calls
     `continuation_fault_outcome` and shows the copy: the fresh-request
     boundary in `session_requests.rs` and the text and inspection resume
     boundaries in `session_eval.rs`.
3. **Ruled inherent: the nested driver-result test in `resume_with_input`
   (3 per resume).** The driver's fault exit must be a channel that guest
   frames cannot consume, and one discriminant test on it is its price. It
   becomes the fifth allowed item. Report its measured number beside the
   other four.

**Gates.**
- **Resume workloads.**
  - Address-level attribution on the final candidate for all five workloads
    must show zero instructions in the three removed categories.
  - Report each of the five allowed items with its number.
  - Callgrind totals use the rule above.
  - Also report the brief's `perf stat` slopes against the recorded bases,
    and allocations/bytes exactly equal.
- **Ordinary panel.** Rerun all 64 labels. For a label above its floor, take
  five `perf stat` runs per arm and at least three callgrind processes per
  arm, then classify it by the rule above.
- **Timing panel.**
  - Rerun it last, with no other work on the host.
  - Redispatch 4's candidate timing pass (about 12:01–12:02) overlapped
    director GPU measurements on this host. Its three signals above +5% are
    evidence neither way.
- **Touched gates.** Then rerun every gate in redispatch 4's item 4 on the
  final tree, including the retention census line pins and the
  fingerprint/certificate replays if `core.rs` lines move, and the
  workspace run.

**Stop-loss.** The no-progress counter restarts at zero. Each target counts
separately. An experiment that does not reduce its own target's
address-level count is discarded. After two discarded experiments on one
target, stop work on that target, keep whatever reduced the others, and
report it with the attribution.

**Scope.** Unchanged from redispatch 4: the 49 modified and 10 new S2 files,
plus the inventory, fingerprint and certificate replays under their existing
rules. Append a new `## Execution report` after this section.

## Execution report — redispatch 5: direct legacy pop and cold boundary projection

**Correctness, allocation/size and instruction-attribution gates pass under redispatch 5's rule.** All three targeted residuals are zero on all five final resume workloads. The five allowed items total 45/53/45/45/69 instructions per resume for Table/Do/call-repeat/cleanup/Fold. Three fresh Callgrind slope pairs per arm leave no unexplained mean residual above the larger arm range. All 64 allocation and byte slopes remain exactly equal. All six ordinary above-floor investigations classify as process-to-process variation after five perf slopes and three Callgrind pairs per arm.

All required final-tree correctness gates pass, including the workspace suite; both oracle fault-occurrence deltas are zero. The retained driver still has no typed production frame. The direct legacy pop preserves pop, accounting refresh and explicit drop, while every affected boundary uses a discriminant-only check and cold consuming fault projection.

**Timing qualification:** both final 120-row panels completed last, with no competing CPU activity observed. GPU activity was unavailable to the sandbox, and the host-activity question received no answer before collection. Thus the amendment's fully idle-host condition is not independently verified. All timing rows are retained, including 71 positive deltas and eight above +5%; this report does not claim a clean timing acceptance result.

Redispatch 5 governs this run. Before editing, all 1,471 source/protected file hashes matched the accepted redispatch-4 state, and all four original base binaries matched their recorded hashes. The entry snapshot, raw experiments and final evidence are preserved under `.temp/brief-608/redispatch-5/`. No earlier performance totals are substituted for the fresh final measurements.

### 1. Files changed

The cumulative S2 tree remains 49 modified and 10 new repository files, plus this appended report. Redispatch 5 changes 15 of those files. `final-audit.json` and `redispatch-5.patch` record the complete source comparison without git.

| File | Semantic change | Redispatch 5 |
| --- | --- | --- |
| `crates/rum-host/src/model/outcome.rs` | Map ContinuationSlotMismatch to the existing terminal HostFailureCode::Other outcome. | retained |
| `crates/rum-runtime/Cargo.toml` | Add the empty test-continuation-slots feature. | retained |
| `crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs` | Explicitly propagate the public fault variant from capture rendering. | retained |
| `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs` | Independently enumerate legacy or mixed queue cells and both typed Arc allocations; visit frame payloads once. | retained |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs` | Explicit fatal passthrough in capture completion and outcome rendering. | retained |
| `crates/rum-runtime/src/runtime/kernel_api.rs` | Export the typed macro, the feature-gated FaultProbe and the opaque SlotMismatch beside EvalError. | retained |
| `crates/rum-runtime/src/runtime/mod.rs` | Export feature-gated FaultProbe and public SlotMismatch at the runtime facade. | retained |
| `crates/rum-runtime/src/runtime/model/requests.rs` | Add SessionRequestError::ContinuationSlotMismatch and its Display arm. | retained |
| `crates/rum-runtime/src/runtime/session/session_eval.rs` | Project the latched fault at text/inspection resume boundaries before legacy value decoding. Redispatch 5: Apply the same discriminant-only branch and cold projection to text and inspection boundaries. | changed |
| `crates/rum-runtime/src/runtime/session/session_requests.rs` | Project the latched fault at request boundaries and map the new EvalError variant to SessionRequestError. Redispatch 5: Fresh and resumed request boundaries branch on the latch before finalization; no-fault outcomes go directly to the existing finalizer. | changed |
| `crates/rum-runtime/src/runtime/session/session_state.rs` | Add EvalError::ContinuationSlotMismatch(SlotMismatch), formatted through read-only static-label accessors. | retained |
| `crates/rum-runtime/src/runtime/suspension/core.rs` | Run typed and legacy frames in separate direct arms. Decode while the consumer remains queued; match each original action in place, drop both owners before finalization, and splice nested owners before the suffix. Legacy insertion passes its Arc directly. Redispatch 5: Use a specialized legacy pop while preserving pop, accounting refresh, explicit drop and finalization order. Direct legacy/typed dispatch and the one named driver remain. | changed |
| `crates/rum-runtime/src/runtime/suspension/core/abandon.rs` (new in S2) | Move the existing abandonment worklist protocol into its own module and share its drain with faults. | retained |
| `crates/rum-runtime/src/runtime/suspension/core/fault.rs` (new in S2) | First-fault latch and structural unwind; opaque SlotMismatch with suspension-only production constructor and cfg(test) injection constructor; feature/test-only monotonic FaultProbe. Redispatch 5: Inline only the latch discriminant check. Keep taking the fault, consuming the abandoned outcome and projecting its error in a cold, non-inlined function. | changed |
| `crates/rum-runtime/src/runtime/suspension/core/queue.rs` (new in S2) | Keep legacy Arc cells and widen only for typed entries. Inline front/pop/is_empty, direct legacy insertion and legacy append; keep widening and mixed append cold and out of line. Redispatch 5: Pop legacy Arc cells directly, with the presence check in that representation. The cold mixed branch matches the entry and explicitly rejects a typed entry on the legacy route. | changed |
| `crates/rum-runtime/src/runtime/suspension/error.rs` | Exhaustiveness arm guards against structural faults entering the legacy error wire; explicitly removed with that wire at R. | retained |
| `crates/rum-runtime/src/runtime/suspension/output.rs` | Collect capture boundaries from either queue representation without cloning frame state. | retained |
| `crates/rum-runtime/src/runtime/suspension/trace.rs` | Re-export FaultProbe and SlotMismatch through the existing suspension module boundary. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed.rs` | Connect live typed exports without module-wide dead-code allowance; remove the obsolete DispatchAction export. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | Make sealed actions reachable by the driver; add only a cfg(test)-private malformed-slot constructor. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | Expose the inner borrowed layout to both accounting models; preserve exact decode and adapter-only token minting. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/chain.rs` | Implement witnessed chain installation and return the last output type. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | Centralize exact completion decoding and static fault kind names; no dual-arm decode. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/entry.rs` (new in S2) | Private legacy/typed ownership and borrowed metadata/accounting view; remove Entry::resume and DispatchAction so the legacy action is not rewrapped. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs` | Accept fresh state of the same nominal frame and return ResumeStop<F::Output>. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests.rs` | Connect live route, driver-fault, file and accounting tests. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs` | Verify two-Arc layout, real queue cells, maintained/independent totals and below/at/above admission. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs` (new in S2) | Fault matrix across captures/scopes, cleanup, completion, rollback/reset and direct public-variant capture defence. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/fault_support.rs` (new in S2) | Runtime-private wrong-mode/exact-state fixtures, non-running consumer/suffix spies and common restoration assertions. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs` (new in S2) | Nested memory-file and public-batch rollback under all three injected mismatch kinds. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs` (new in S2) | Test macro/chain, legacy neighbors, repeat helper, nested Evaluate and both-Arc finalizer release. | retained |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | Implement witnessed suspension and the cold actual-Result typed macro. | retained |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | Check both direct resume arms independently: exact decode before pop, explicit drop before each finalizer, and unchanged structural abandonment confinement. Redispatch 5: Change only the direct-arm pop-name patterns to distinguish pop_front_legacy from pop_front; preserve decode/pop/drop/finalizer assertions. | changed |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | Record SlotMismatch's new FrameRootTrace derive in the sorted closed inventory. | retained |
| `crates/rum-stdlib/Cargo.toml` | Enable that feature only on the existing runtime dev-dependency. | retained |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs` | Replace only the three-site count assertion with exact set equality over four discovered constructs; witness loop byte-identical. | retained |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | changed |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | changed |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | changed |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | changed |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | changed |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | Replay source discovery and locations with existing or director-approved S2 dispositions; full row transitions retained in the replay patches. | retained |
| `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` | Follow unresolved abandonment operations into their split module and classify the final same-operation take_for_abandon line shift (473 to 477); keep 932/10/895/27 totals, all unresolved assertions and historical print unchanged. Redispatch 5: Document the discovered take_for_abandon line shift 477 to 482; retain all 932 rejections, kind totals, assertions and historical print. | changed |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Replay all 68 dependency fingerprints; preserve every contract field and strength. | changed |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Replay exactly the four emitted default/no-default C01/C02 certificate cells; preserve statuses, bodies, preconditions, strengths and exclusions. | changed |
| `crates/rum-stdlib/tests/suspension_oracle.rs` | Add the serial last aggregate fault-occurrence assertion. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` | Select the S2 typing receipt while continuing to refuse preflip. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | Record S2 mechanism routes, unchanged observed T0010, and four discovered test-support legacy uses X0036-X0039. Redispatch 5: replay Whole0179 call ordinal 2 to 3 from discovery; retain D130, R, legacy-use and every other receipt cell. | changed |
| `crates/rum-stdlib/tests/suspension_oracle/support/drive.rs` | Unconditional fault observer around every shared drive, independent of Checks. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/faults.rs` (new in S2) | New unconditional guards and retained probe observations for the per-build aggregate assertion. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/lifecycle.rs` | Check before session replacement/reset and retain the old probe through lifecycle completion. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/mod.rs` | Wire the fault-observer support module. | retained |
| `crates/rum-stdlib/tests/suspension_oracle/support/session.rs` | Observe initialization, retirement, fresh requests and input-queue controls. | retained |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | Connect route receipts, S2 preflip refusal and scratch-copy falsifiers under brief-608. | retained |
| `crates/rum-stdlib/tests/typed_slot_census/classify.rs` | Keep codec population metadata so private injection carriers are reported separately from production registry entries. | retained |
| `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` | Preserve exact production/test-only counts and production mode checks; identify output as S2. | retained |
| `crates/rum-stdlib/tests/typed_slot_census/reporting.rs` | Write under brief-608 and include codec population in discovery output. | retained |
| `crates/rum-stdlib/tests/typed_slot_census/routes.rs` (new in S2) | New live source/witness checks for six S2 routes, unchanged family ownership of four remaining routes and T0010 observation. | retained |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | Retain 22 S1 pairs, add seven S2 pairs and the external opaque-payload pair; require E0451 with both exact privacy spans in each feature build. | retained |

Exhaustiveness arms remain unchanged this round: terminal HostFailureCode::Other, public request-error/Display mappings, fatal capture passthroughs, and the legacy error-wire invariant guard removed at R. Every runtime fault-injection test is byte-identical to entry. No production family was typed or flipped.

Split assessment: `core.rs` is 29,221 characters, below 35k. Queue storage, abandonment, fault/probe state and typed entry metadata already have separate modules. Keep the one named production driver together for I0009 and its pop/refresh/drop ordering. `session_requests.rs` is 34,880 characters; its future boundary is request identity/cancellation versus evaluation-response packaging. This narrowly scoped boundary repair does not split that lifecycle. `session_eval.rs` is 18,512 characters; a future split separates catalog inspection from top-level evaluation. `frame_root_trace.rs` is a 33,906-character cohesive source-gate test file, below the test debt threshold; future grouping is structural driver invariants versus derived-root inventory. The touched queue/fault modules remain below 10k.

### 2. Driver, routes, fault path and probe

The retained repair dispatches on the borrowed EntryRef in the single production drive_resuming_frames function. The legacy arm performs its exact Value match, calls frame.resume, and matches the original FrameAction directly; the typed arm independently decodes and matches Action. Entry::resume and DispatchAction are removed. Legacy insertion takes its Arc directly; small queue operations are inline, and widening/mixed append remain cold and out of line. The driver’s legacy pop consumes Arc cells directly; its cold mixed-representation helper inspects the existing Entry. Pop, accounting refresh and explicit drop precede finalization. The nested driver Result remains, with its discriminant check now the fifth allowed item. All four request/text/inspection boundaries check only the latch discriminant on the hot path; taking the latch and projecting an error consume the outcome in a cold non-inlined function. Every fault-injection test is byte-identical to redispatch-5 entry.

The pop invariant is structural: dispatch borrows the current resuming front; the consumer call can create a separate pending owner through the suspension insertion API, but only this driver removes or splices the resuming queue. It pops before splicing nested pending work. A Legacy queue physically contains only Arc<dyn SuspensionFrame> cells, so the legacy pop cannot encounter a typed entry. In a Mixed queue the cold branch explicitly matches Entry::Legacy and panics on Entry::Typed instead of silently dropping it. This preserves the base driver’s observed-front invariant without a new assertion on the legacy path.

Legacy-only owners retain VecDeque cells containing one Arc<dyn SuspensionFrame>; a typed insertion widens that owner to private Entry cells. Queue representation does not choose transport. Legacy accepts Value only; typed transport is its type's sealed associated mode, with no dual-arm decoder or per-frame override.

I0001 borrows and matches the actual producer Result once and constructs the frame only for InputRequired. I0002 checks the first input, adjacency and last output. I0003 uses the adapter-minted token and fresh state of the same nominal frame. I0007 requires an Expr-input successor with the parent's output. I0008 releases both frame Arcs before the pure finalizer. I0009 seeds a Value string, places nested owners before the unrun suffix and exactly decodes completion. I0004/I0005/I0006/I0010 remain S5/S18/S14/S24; T0010 observes the unchanged CLI mapping. No production frame was typed and no codec was flipped.

The driver resumes an Arc clone while the queue retains the consumer; it never clones frame state. A decode failure therefore leaves the consumer at the active front. The first private DriverFault is latched, active and nested owners enter the existing accounted abandonment worklist, pending handles are cleared, output rolls back, and structural abandonment restores scopes and loaders. Approved WithCleanup evaluation is the only guest cleanup transition; its nested input owner is abandoned before the saved suffix. Cleanup failures cannot replace the first fault. Only after unwind does the boundary project SlotMismatch; it never enters the frame-completion loop. EvaluationData and TestReport also propagate a direct internal public-variant return.

FaultProbe exists only in runtime tests or the test-continuation-slots feature. Only creation of the first private fault increments it. Retained observers survive reset, rollback and session replacement. Oracle checks are independent of Checks flags, run before lifecycle reset, and retain observations for the final aggregate. Both measured oracle deltas are zero. Release feature-tree and measured error-size evidence is reported with the four candidate binaries below.

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

Below/at/above admission tests use the real pre-existing InputString queue. Widening its four cells and adding a zero-sized typed frame costs exactly 72 bytes: 40 for both Arc layouts and 32 for wider cells. Maintained/independent reconciliation also covers nested Evaluate, finalization, capture/scope faults, cleanup input, reset and nested/public file rollback. The measured legacy allocation/byte slopes are reported below.

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

Whole0179 retains decoder D130, owner, production population, R and legacy-use status. Fresh discovery replays only its call ordinal from 2 to 3, shifted by the added cold Err constructor. The first default census failure and successful rerun are disclosed below; no census acceptance code changed. Both frame-removal and test-guard-removal falsifiers reject as intended in both builds. Their child FAILED totals below are intentional. Preflip still refuses without S54; private test carriers are not production registry entries. X0036–X0039 record four actually discovered compatibility-fixture uses; no production receipt was removed.

### 6. Gates, binaries, measurements and discovery replays

All Cargo gates ran serially and offline with `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-608/tmp`, through `scripts/run-bounded.py`. Bounds: workspace 3600 seconds, oracle 1500 per build, other Cargo gates/builds 900. Raw commands/logs and exit records are retained. Two compilation errors in the first boundary test run were corrected by preserving the existing Result return type; the tests were not edited. Initial certificate gates rejected only stale cells after earning fresh marks. The first typed census failed on the shifted Whole0179 anchor; the discovery-only replay and rerun are below. Deliberate census child failures are reported separately from their green parent gates.

```text
typed-pop-a1: exit 0
$ python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime --lib suspension::typed::tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.45s
```

```text
callgrind-pop-a1: exit 0
$ python3 .temp/brief-608/redispatch-5/callgrind.py candidate-pop-a1 do-chain fold
(no test-result total; complete output is in the named log)
```

```text
profile-pop-a2: exit 0
$ python3 .temp/brief-608/redispatch-5/profile-attempt.py candidate-pop-a2
(no test-result total; complete output is in the named log)
```

```text
typed-boundary-a1: exit 101
$ python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime --lib suspension::typed::tests
error[E0599]: no method named `unwrap_err` found for enum `session_state::EvalError` in the current scope
error[E0599]: no method named `unwrap_err` found for enum `session_state::EvalError` in the current scope
```

```text
typed-boundary-a1-fixed: exit 0
$ python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime --lib suspension::typed::tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.45s
```

```text
profile-boundary-a1: exit 0
$ python3 .temp/brief-608/redispatch-5/profile-attempt.py candidate-boundary-a1
(no test-result total; complete output is in the named log)
```

```text
format-final: exit 0
$ python3 scripts/run-bounded.py --timeout 900 -- cargo fmt
(no test-result total; complete output is in the named log)
```

```text
frame-roots-final: exit 0
$ python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime --lib frame_root_trace -- --nocapture
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.30s
```

```text
ownership-discovery: exit 0
$ python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-stdlib --lib ownership_graph::discovery_report_without_inventory_input -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 5.68s
```

```text
certificates-default-1: exit 101
$ python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.19s
```

```text
certificates-no-default-1: exit 101
$ python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.50s
```

```text
fmt-check: exit 0
$ cargo fmt --check
(no test-result total; complete output is in the named log)
```

```text
certificates-default-2: exit 0
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.45s
```

```text
certificates-no-default-2: exit 0
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.30s
```

```text
retention-census-default: exit 0
$ cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 25.25s
```

```text
retention-census-no-default: exit 0
$ cargo test -p rum-stdlib --no-default-features --lib production_rejection_census_is_complete -- --nocapture
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 25.19s
```

```text
runtime-typed: exit 0
$ cargo test -p rum-runtime --lib suspension::typed::tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.45s
```

```text
compile-default: exit 0
$ cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
COMPILE CONTROLS: 30 positive / 30 intended rejections (22 S1 + 7 S2 + opaque fault); raw Cargo JSON and primary spans retained
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.82s
```

```text
compile-no-default: exit 0
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
COMPILE CONTROLS: 30 positive / 30 intended rejections (22 S1 + 7 S2 + opaque fault); raw Cargo JSON and primary spans retained
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.42s
```

```text
typed-census-default: exit 101
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
test result: FAILED. 2 passed; 3 failed; 1 ignored; 0 measured; 0 filtered out; finished in 95.86s
```

```text
typed-census-default-2: exit 0
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.89s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.84s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 134.62s
```

```text
typed-census-no-default: exit 0
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.04s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.19s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 135.43s
```

```text
oracle-both: exit 0
$ scripts/check-suspension-oracle.sh 1500
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.71s
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.59s
```

```text
ownership: exit 0
$ cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 79 rows; 11 storage edges (3 Declared, 8 Unbounded); 68 PublicInput
test tests::ownership_graph::host_payload_ledger::tests::adapter_production_witnesses_are_computed ... ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	19	15	rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	21	14	rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer	rum_runtime::runtime::suspension::typed::action::ErasedFinalizer => rum_runtime::runtime::suspension::typed::adapter::FinalizerAdapter.0 (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:82:1) => rum_runtime::runtime::suspension::typed::SlotFinalizer (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	9	13	rum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7712 rows
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 69.57s
```

```text
workspace: exit 0
$ cargo test --workspace --no-fail-fast -- --nocapture
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.83s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 40.49s
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 79 rows; 11 storage edges (3 Declared, 8 Unbounded); 68 PublicInput
test tests::ownership_graph::host_payload_ledger::tests::adapter_production_witnesses_are_computed ... ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	19	15	rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	21	14	rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer	rum_runtime::runtime::suspension::typed::action::ErasedFinalizer => rum_runtime::runtime::suspension::typed::adapter::FinalizerAdapter.0 (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:82:1) => rum_runtime::runtime::suspension::typed::SlotFinalizer (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	9	13	rum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet)
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7712 rows
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1092 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1153.45s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.74s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=87920
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.99s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.98s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.85s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 135.22s
COMPILE CONTROLS: 30 positive / 30 intended rejections (22 S1 + 7 S2 + opaque fault); raw Cargo JSON and primary spans retained
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.83s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.51s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.79s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.45s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
build-all: exit 0
$ python3 .temp/brief-608/redispatch-5/build-all.py
(no test-result total; complete output is in the named log)
```

```text
candidate-check: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/run-perf.py check candidate 1
(no test-result total; complete output is in the named log)
```

```text
candidate-alloc: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/run-perf.py alloc candidate 1
(no test-result total; complete output is in the named log)
```

```text
base-instructions-1: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/run-perf.py instructions base 1
(no test-result total; complete output is in the named log)
```

```text
candidate-instructions-1: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/run-perf.py instructions candidate 1
(no test-result total; complete output is in the named log)
```

```text
base-instructions-2: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/run-perf.py instructions base 2
(no test-result total; complete output is in the named log)
```

```text
candidate-instructions-2: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/run-perf.py instructions candidate 2
(no test-result total; complete output is in the named log)
```

```text
performance-comparison: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/compare.py
(no test-result total; complete output is in the named log)
```

```text
callgrind-final: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/callgrind-final.py
(no test-result total; complete output is in the named log)
```

```text
attribution: exit 0
$ python3 .temp/brief-608/redispatch-5/final-attribution.py
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-range-normal-small-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-range-normal-small base
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-range-normal-small-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-range-normal-small candidate
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-table-construct-full-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-table-construct-full base
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-table-construct-full-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-table-construct-full candidate
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-permutations-take-small-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-permutations-take-small base
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-permutations-take-small-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-permutations-take-small candidate
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-tuples-take-small-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-tuples-take-small base
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-tuples-take-small-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-tuples-take-small candidate
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-depth-map-first-256-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-depth-map-first-256 base
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-depth-map-first-256-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-depth-map-first-256 candidate
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-depth-select-restart-256-base: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-depth-select-restart-256 base
(no test-result total; complete output is in the named log)
```

```text
ordinary-lazy-depth-select-restart-256-candidate: exit 0
$ python3 /home/remi/dev/rum/.temp/brief-608/redispatch-5/ordinary-exceptions.py lazy-depth-select-restart-256 candidate
(no test-result total; complete output is in the named log)
```

```text
timing-base: exit 0
$ {"command": ["scripts/core-language-microbench.sh", "--output-dir", "/home/remi/dev/rum/.temp/brief-608/redispatch-5/perf/timing/base", "--timeout", "850", "--scale", "1", "--reps", "7", "--rum-cli", "/home/remi/dev/rum/.temp/brief-608/base/rum-cli"], "profile_flags": "removed; script explicitly disables profiling", "retained_module_gc_batches": 3}
(no test-result total; complete output is in the named log)
```

```text
timing-candidate: exit 0
$ {"command": ["scripts/core-language-microbench.sh", "--output-dir", "/home/remi/dev/rum/.temp/brief-608/redispatch-5/perf/timing/candidate", "--timeout", "850", "--scale", "1", "--reps", "7", "--rum-cli", "/home/remi/dev/rum/.temp/brief-608/redispatch-5/candidate/rum-cli"], "profile_flags": "removed; script explicitly disables profiling", "retained_module_gc_batches": 3}
(no test-result total; complete output is in the named log)
```

Nine native ZeroMQ cases skipped because loopback binding is unavailable. Their crate aggregate includes the early returns; these are skips, not executed native transport coverage.

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

All four final release binaries use one codegen unit, the original source paths and separate allocation/non-allocation target directories. Their recorded feature trees exclude test-continuation-slots. Original bases were preserved and rehashed; error sizes do not grow.

| Arm | Binary | Bytes | SHA-256 |
| --- | --- | --- | --- |
| base | rum-cli | 12376160 | 76efb7c611f510a4b46c8d181d77ec37c5fe7ef057c2572ce05978e7a795ecf5 |
| candidate | rum-cli | 12537440 | 7c69825d10da396440ade57ac6086f2359c8854afcef22b2ddfd3b4456509496 |
| base | rum-cli-alloc | 12390248 | 0e9c806bab777893c83e4e7cf1411d977ac63bb77c64374af0164e40a51e56a4 |
| candidate | rum-cli-alloc | 12547616 | 767cc65e4123eb355ff90069d93c0e02899e39b8e4df5b512cff7cef271d9923 |
| base | resume | 12019168 | be2739b2e4a62ff412500e01d7edb74e362726030796b90c722167ac639eb131 |
| candidate | resume | 12180552 | a827a7792a36ac8a62bebba4f7fb7d825b0fc2dd201c8ef4a89b47316ef6a3e4 |
| base | resume-alloc | 12028296 | 16aaeffd58bcf727704bd829db9ad7bc3219da67a6ccc78e87bb0b44431b0fb9 |
| candidate | resume-alloc | 12185720 | b6c15c7d6f123be868ba06a6ca70023ec0cb0bbe02e3f62ea869af367b5bf429 |

```text
base:
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136
candidate:
size_of::<EvalError>() = 136
size_of::<Result<Expr, EvalError>>() = 136
```

Targeted experiments: the first legacy pop removes Entry construction and reduces the extra count from 10 to 2 instructions/frame. Moving the presence check inside each queue representation removes those last two. The first compiling boundary experiment removes the 33 extra outcome instructions and four eager payload/clear instructions. All are retained; no experiment was discarded. Final binary/address evidence follows, including both controls and all five final workloads.

| Attempt | Candidate SHA-256 | Target result |
| --- | --- | --- |
| candidate-pop-a1 | e72f1cf61134af3f67bec751a144ed8f1af13908d3c874ee8017276946778b14 | extra pop 10 -> 2/frame |
| candidate-pop-a2 | af45aca45a079631a5b9d529f8a6cc0a720562f6c9751f0647549712eb9bd7f4 | extra pop 2 -> 0/frame |
| candidate-boundary-a1 | a827a7792a36ac8a62bebba4f7fb7d825b0fc2dd201c8ef4a89b47316ef6a3e4 | extra boundary copy 33 -> 0; eager fault words 4 -> 0 |

The final 64-label panel checks every result before measurement. All allocation/byte slopes compare against the recorded original bases. Two fresh instruction pairs use CPU 2, stdin /dev/null, cpu_core/instructions/u, and (count(2N)-count(N))/N. The frozen base/base floors and every sample remain visible; no sample was discarded for its sign.

```tsv
label	engine	N	base_alloc	candidate_alloc	delta_alloc	base_bytes	candidate_bytes	delta_bytes	base1_instructions	candidate1_instructions	delta1_percent	base2_instructions	candidate2_instructions	delta2_percent	floor_percent	ordinary_threshold_percent	ordinary_regression
lazy-range-take-small	cli	20000	141.0	141.0	0.0	2714.0	2714.0	0.0	42195.6672	42199.67225	0.009491614342804766	42189.57325	42200.63705	0.026224014958464714	0.143667	0.143667	False
lazy-range-normal-small	cli	20000	157.0	157.0	0.0	3388.0	3388.0	0.0	53993.1776	54205.0221	0.39235419994987364	54254.2138	54079.1309	-0.32270839025595466	0.122231	0.122231	False
lazy-map-realize-small	cli	20000	333.0	333.0	0.0	13740.0	13740.0	0.0	178859.0959	178825.7761	-0.01862907772866018	178861.66225	178818.104	-0.02435303879660955	0.027087	0.05	False
lazy-select-realize-small	cli	20000	236.0	236.0	0.0	6834.0	6834.0	0.0	114579.84275	114547.7571	-0.028002874877386397	114579.1903	114420.57155	-0.13843591457113336	0.006677	0.05	False
lazy-select-scan-small	cli	20000	1471.0	1471.0	0.0	62793.0	62793.0	0.0	805568.6981	805414.93805	-0.019087143078266866	805411.62735	805302.5714	-0.01354039925631767	0.007982	0.05	False
lazy-nestlist-realize-small	cli	20000	197.0	197.0	0.0	11316.0	11316.0	0.0	145402.2901	145291.38405	-0.07627531170502078	145304.33175	145346.39285	0.0289469002702214	0.033018	0.05	False
lazy-foldlist-realize-small	cli	20000	216.0	216.0	0.0	7604.0	7604.0	0.0	112704.28685	112715.77085	0.010189497064372333	112910.4247	112640.4964	-0.2390641083116929	0.106063	0.106063	False
lazy-drop-realize-small	cli	20000	351.0	351.0	0.0	14204.0	14204.0	0.0	191469.3502	191470.7207	0.0007157803578516919	191465.9424	191521.1465	0.02883233399528784	0.150339	0.150339	False
lazy-first-small	cli	20000	64.0	64.0	0.0	2868.0	2868.0	0.0	41306.2211	41211.65245	-0.22894529560343324	41182.9783	41208.9348	0.06302725317950397	0.200265	0.200265	False
lazy-part-small	cli	20000	152.0	152.0	0.0	6182.0	6182.0	0.0	88478.672	88350.5705	-0.1447823493553435	88255.0617	88303.67365	0.05508120334811739	0.111397	0.111397	False
lazy-transparent-adapters-small	cli	20000	319.0	319.0	0.0	12876.0	12876.0	0.0	181426.8956	181453.82075	0.01484077094027203	181562.2107	181149.48905	-0.22731693363325922	0.015381	0.05	False
lazy-table-construct-short	cli	20000	63.0	63.0	0.0	1537.0	1537.0	0.0	23674.71645	23616.9998	-0.24379024822490658	23588.74115	23640.0285	0.21742300563587502	0.305248	0.305248	False
lazy-table-construct-full	cli	20000	64.0	64.0	0.0	1449.0	1449.0	0.0	24116.2054	24120.1627	0.01640929795696522	24068.1984	24156.9518	0.36875797068383065	0.190008	0.190008	False
lazy-table-realize-small	cli	20000	251.0	251.0	0.0	9190.0	9190.0	0.0	117152.5828	117189.20645	0.03126149601202499	117149.12165	117223.4045	0.0634087980803999	0.09186	0.09186	False
lazy-permutations-take-small	cli	20000	29.0	29.0	0.0	1729.0	1729.0	0.0	28158.89585	28310.0926	0.5369413303895509	28199.0614	28167.9484	-0.11033345953846974	0.129913	0.129913	False
lazy-subsets-take-small	cli	20000	29.0	29.0	0.0	1417.0	1417.0	0.0	25361.9823	25356.07145	-0.023305946396789956	25353.34215	25332.22975	-0.08327265050537269	0.037505	0.05	False
lazy-tuples-take-small	cli	20000	29.0	29.0	0.0	1361.0	1361.0	0.0	26533.1815	26584.89445	0.19489916804737994	26580.81465	26670.9785	0.33920649606575815	0.179754	0.179754	True
scan-no-input	cli	20000	33.0	33.0	0.0	2536.0	2536.0	0.0	29709.0148	29682.78675	-0.08828313620148354	29710.1286	29684.6307	-0.08582224716455755	0.003205	0.05	False
optionvalue-three-no-input	cli	20000	9.0	9.0	0.0	1128.0	1128.0	0.0	9589.3027	9590.1232	0.008556409424853584	9590.54535	9589.96375	-0.00606430582177353	0.029575	0.05	False
replacerepeated-no-input	cli	20000	57.0	57.0	0.0	7899.0	7899.0	0.0	39207.3596	39161.3326	-0.11739377624399028	39181.3922	39174.46565	-0.017678162033263334	0.000695	0.05	False
dot-three-no-input	cli	20000	596.0	596.0	0.0	23504.0	23504.0	0.0	254199.91595	254198.1083	-0.000711113531748353	254199.79315	254199.6585	-5.297014539218381e-05	0.000109	0.05	False
differences-no-input	cli	20000	13.0	13.0	0.0	704.0	704.0	0.0	10030.5513	10035.7983	0.05231018558273792	10045.019	10035.22695	-0.09748164737169995	0.071642	0.071642	False
do-count-null	cli	20000	0.0	0.0	0.0	0.0	0.0	0.0	537.13755	536.9272	-0.03916129118138878	536.99545	537.0044	0.001666680788448005	0.053612	0.053612	False
do-iterator-null	cli	20000	0.0	0.0	0.0	32.0	32.0	0.0	1154.0282	1153.9879	-0.003492115703918053	1154.09375	1154.22665	0.011515528959415988	0.018868	0.05	False
table-iterator-null	cli	20000	0.0	0.0	0.0	32.0	32.0	0.0	387.1108	387.22285	0.028945201218877337	387.1392	387.0682	-0.018339656640309343	0.042006	0.05	False
replaceall-simple	cli	20000	5.0	5.0	0.0	1352.0	1352.0	0.0	11432.33835	11406.5455	-0.22561307416167908	11432.1113	11380.3065	-0.45315164137703645	0.452949	0.452949	False
table-small	cli	20000	22.0	22.0	0.0	977.0	977.0	0.0	14618.51875	14634.13505	0.10682546068494503	14633.48095	14712.0041	0.5365992566519262	0.572845	0.572845	False
table-iterator-plus	cli	20000	2.0	2.0	0.0	72.0	72.0	0.0	563.29325	563.27	-0.00412751262329758	563.41205	563.28275	-0.022949455908882754	0.029983	0.05	False
table-iterator-pair-list	cli	20000	2.0	2.0	0.0	136.0	136.0	0.0	1640.8374	1640.8289	-0.0005180281726802072	1640.8031	1640.84175	0.0023555538138486654	0.001688	0.05	False
take-multispec-small	cli	20000	109.0	109.0	0.0	5856.0	5856.0	0.0	49915.52925	49922.0921	0.013147912280220275	49932.0305	49921.78315	-0.02052259821477831	0.046752	0.05	False
lazy-depth-map-build-8	cli	1000	16.001	16.001	0.0	1879.864	1879.864	0.0	18126.344	18119.314	-0.038783331045699754	18094.694	18108.755	0.07770786286853681	0.087984	0.087984	False
lazy-depth-map-first-8	cli	1000	188.0	188.0	0.0	11176.0	11176.0	0.0	130387.361	130390.16	0.0021466804593028854	130399.83	130386.161	-0.010482375628872997	0.026366	0.05	False
lazy-depth-map-eight-8	cli	1000	1464.001	1464.001	0.0	82776.864	82776.864	0.0	959562.134	959634.028	0.007492375683937347	959678.06	959639.579	-0.004009782197167411	0.014316	0.05	False
lazy-depth-select-restart-8	cli	1000	327.0	327.0	0.0	16980.0	16980.0	0.0	179985.104	179987.346	0.0012456586407250114	179407.0	179387.7	-0.010757662744476093	0.165894	0.165894	False
lazy-depth-drop-all-one-8	cli	1000	170.0	170.0	0.0	3128.0	3128.0	0.0	46664.605	46664.442	-0.0003493011459076989	46662.835	46660.151	-0.005751900843575797	0.002949	0.05	False
lazy-depth-take-map-8	cli	1000	220.0	220.0	0.0	12144.0	12144.0	0.0	138946.108	138949.79	0.0026499482806752184	138949.87	138934.681	-0.010931280468262639	0.003719	0.05	False
lazy-depth-fold-through-source-8	cli	1000	743.001	743.001	0.0	58440.864	58440.864	0.0	589539.081	589565.267	0.0044417750822534074	589559.428	589547.5	-0.0020232057081015498	0.001805	0.05	False
lazy-depth-nest-map-8	cli	1000	192.0	192.0	0.0	11964.0	11964.0	0.0	141008.078	141003.56	-0.003204071755391613	140998.331	141008.28	0.007056111891134975	0.006765	0.05	False
lazy-depth-map-build-64	cli	1000	16.001	16.001	0.0	5911.864	5911.864	0.0	30117.236	30102.175	-0.050007909092331104	30121.26	30116.171	-0.01689504356723326	0.116452	0.116452	False
lazy-depth-map-first-64	cli	1000	1364.0	1364.0	0.0	85320.0	85320.0	0.0	946256.965	946258.426	0.0001543978067397589	946106.99	946182.46	0.007976899103123714	0.335237	0.335237	False
lazy-depth-map-eight-64	cli	1000	10872.001	10872.001	0.0	647704.864	647704.864	0.0	7421316.113	7421738.356	0.005689597283975445	7421772.155	7421932.453	0.0021598345604223468	0.02104	0.05	False
lazy-depth-select-restart-64	cli	1000	1671.0	1671.0	0.0	99412.0	99412.0	0.0	981809.559	975796.401	-0.6124566566783729	976163.627	975906.913	-0.026298255015810845	0.656111	0.656111	False
lazy-depth-drop-all-one-64	cli	1000	1178.0	1178.0	0.0	17520.0	17520.0	0.0	327583.81	327576.787	-0.0021438788443184364	327580.547	327582.3	0.0005351355616189934	0.002126	0.05	False
lazy-depth-take-map-64	cli	1000	1620.0	1620.0	0.0	93064.0	93064.0	0.0	1007533.245	1007348.425	-0.018343811573184876	1007538.196	1007318.605	-0.0217948064769935	0.001015	0.05	False
lazy-depth-fold-through-source-64	cli	1000	43334.001	43334.001	0.0	3402649.864	3402649.864	0.0	33997252.516	33997273.259	6.101375396028175e-05	33996663.286	33996774.629	0.00032751155332189086	0.003458	0.05	False
lazy-depth-nest-map-64	cli	1000	1368.0	1368.0	0.0	86108.0	86108.0	0.0	958620.199	958892.559	0.028411669218320057	960283.176	958892.645	-0.14480426552844206	0.17411	0.17411	False
lazy-depth-map-build-256	cli	1000	16.001	16.001	0.0	19735.864	19735.864	0.0	70761.172	70784.126	0.03243869392099974	70761.187	70755.714	-0.007734466071063206	0.009013	0.05	False
lazy-depth-map-first-256	cli	1000	5396.0	5396.0	0.0	339528.0	339528.0	0.0	3716538.515	3722819.725	0.16900699332589308	3722822.658	3723089.537	0.0071687271868015046	0.004742	0.05	False
lazy-depth-map-eight-256	cli	1000	43128.001	43128.001	0.0	2584600.864	2584600.864	0.0	29547682.251	29544451.802	-0.010933003044211187	29546368.674	29544453.763	-0.006481036709204968	0.005175	0.05	False
lazy-depth-select-restart-256	cli	1000	6279.0	6279.0	0.0	382036.0	382036.0	0.0	3665102.332	3665521.715	0.011442600015243443	3665488.076	3691494.748	0.709500930320317	0.705262	0.705262	False
lazy-depth-drop-all-one-256	cli	1000	4634.0	4634.0	0.0	66864.0	66864.0	0.0	2218612.69	2218617.78	0.0002294226487897788	2218608.345	2218611.179	0.0001277377328134932	0.000437	0.05	False
lazy-depth-take-map-256	cli	1000	6420.0	6420.0	0.0	370504.0	370504.0	0.0	3985134.504	3985423.569	0.0072535820236474535	3985012.228	3985536.194	0.013148416366659532	0.003231	0.05	False
lazy-depth-fold-through-source-256	cli	1000	689320.001	689320.001	0.0	54008857.864	54008857.864	0.0	530562746.632	530532856.821	-0.0056336052973504636	530595594.823	530512847.598	-0.0155951586872205	0.004122	0.05	False
lazy-depth-nest-map-256	cli	1000	5400.0	5400.0	0.0	340316.0	340316.0	0.0	3733554.7	3733554.969	7.204929919346625e-06	3733565.149	3733557.576	-0.00020283561952272322	0.000294	0.05	False
literal-integer	cli	20000	0.0	0.0	0.0	0.0	0.0	0.0	293.9632	293.8635	-0.033915809870077585	294.08415	293.89195	-0.0653554433314496	0.035308	0.05	False
downvalue-pattern-hit	cli	20000	11.0	11.0	0.0	908.0	908.0	0.0	13949.3043	13949.48135	0.0012692389254054248	13950.11595	13949.41425	-0.00503006571783704	0.001312	0.05	False
module-local	cli	20000	23.0001	23.0001	0.0	2163.6272	2163.6272	0.0	22653.15955	22652.88355	-0.0012183730900394707	22651.666	22655.86595	0.01854146180682026	0.002678	0.05	False
block-local	cli	20000	8.0	8.0	0.0	752.0	752.0	0.0	14671.79425	14671.7616	-0.00022253583606968164	14671.99135	14671.91925	-0.000491412503456079	0.002024	0.05	False
matcher-single-wide-small	cli	1000	11.0	11.0	0.0	287280.0	287280.0	0.0	760195.669	760179.67	-0.0021045897329319274	760207.683	760207.741	7.629494058747355e-06	0.003397	0.05	False
resume-table	resume	1000	27.003	27.003	0.0	1742.456	1742.456	0.0	15725.904	15725.884	-0.0001271786982837142	15734.434	15814.208	0.5070026668896999	0.153556	0.153556	False
resume-do-chain	resume	1000	24.0	24.0	0.0	1634.0	1634.0	0.0	15116.279	15077.148	-0.2588666165793896	15147.693	15154.64	0.04586176918162099	0.024326	0.05	False
resume-call-repeat	resume	1000	40.007	40.007	0.0	4210.024	4210.024	0.0	24544.354	24581.512	0.1513912323787281	24609.984	24630.951	0.0851971297502585	0.062048	0.062048	False
resume-cleanup	resume	1000	22.0	22.0	0.0	1394.0	1394.0	0.0	14203.91	14242.476	0.2715167865749768	14161.942	14245.823	0.5922987115750233	0.276539	0.276539	False
resume-fold	resume	1000	41.0	41.0	0.0	2873.0	2873.0	0.0	24453.327	24418.016	-0.14440161864274215	24430.536	24433.778	0.013270277819521503	0.334755	0.334755	False
```

Address-level acceptance: all three removed categories are exactly zero on all five final workloads. The final resume SHA-256 is a827a7792a36ac8a62bebba4f7fb7d825b0fc2dd201c8ef4a89b47316ef6a3e4, identical to the accepted boundary experiment. Counts come from Callgrind N=200/400 with instruction/jump collection. Perf sampling supplies disassembly only; it is not used to estimate costs. ELF bytes independently verify unsampled insertion/drop discriminant instructions.

| Workload | Legacy frames/resume | Queue discriminants | Value match | Completion decode | Boundary fault check | Nested driver-result test | Allowed total |
| --- | --- | --- | --- | --- | --- | --- | --- |
| table | 4.0 | 32.0 | 8.0 | 0.0 | 2.0 | 3.0 | 45.0 |
| do-chain | 5.0 | 38.0 | 10.0 | 0.0 | 2.0 | 3.0 | 53.0 |
| call-repeat | 4.0 | 32.0 | 8.0 | 0.0 | 2.0 | 3.0 | 45.0 |
| cleanup | 4.0 | 32.0 | 8.0 | 0.0 | 2.0 | 3.0 | 45.0 |
| fold | 7.0 | 50.0 | 14.0 | 0.0 | 2.0 | 3.0 | 69.0 |

Queue count = 6F + 8: front, pop and insertion contribute two instructions per legacy frame; append contributes four; output-boundary visitation and queue drop contribute two each. Completion decoding runs once at final program completion, so its N/2N slope is zero, not an assertion that it is absent. The no-fault latch check is one memory compare plus branch (two instructions). The nested result test is three instructions and remains the fifth allowed item.

| Workload | Extra pop/drop Entry work | Extra outcome materialization | Eager fault words / clear | Mean total delta | Allowed total | Delta less allowed | Larger process range |
| --- | --- | --- | --- | --- | --- | --- | --- |
| table | 0.0 | 0.0 | 0 | +23.630000 | 45.0 | -21.370000 | 33.375000 |
| do-chain | 0.0 | 0.0 | 0 | +45.873333 | 53.0 | -7.126667 | 127.115000 |
| call-repeat | 0.0 | 0.0 | 0 | -4.131667 | 45.0 | -49.131667 | 104.185000 |
| cleanup | 0.0 | 0.0 | 0 | +48.940000 | 45.0 | +3.940000 | 27.855000 |
| fold | 0.0 | 0.0 | 0 | -79.285000 | 69.0 | -148.285000 | 178.170000 |

Cleanup is the only workload whose raw mean delta exceeds its larger process range. Its +48.94 instructions/resume contains 45 measured allowed instructions; the remaining +3.94 is inside its 27.855-instruction process range. Other workload mean differences are already within their larger ranges, and none has an unexplained residual above range after the allowed items. These statements apply the amendment’s mean/range rule; they do not choose a favorable individual process.

Legacy pop proof: candidate 0x729f1d–0x729f4d and base 0x729495–0x7294c8 each execute ten instructions per legacy frame before Arc decrement. The candidate has no Entry tag construction, 24-byte Entry move, Entry re-test, or the first experiment’s redundant Option<Arc> check. The mixed helper is cold and is not called by any of the five legacy workloads.

Boundary proof: the candidate’s only no-fault latch access is the compare at 0x72bb28 followed by the branch at 0x72bb30. There are no eager reads of the other fault words and no clearing store. Its existing 17-instruction outcome preparation at 0x72bb36–0x72bbb0 corresponds to the base’s 17 instructions at 0x70bb09–0x70bb74; the extra 33-instruction preparation block is gone. No outcome address is passed to an in-place projection helper. The cold branch consumes the outcome and takes/projects the fault. The same source structure is used at fresh request, text and inspection boundaries, with their correctness covered by the final workspace tests.

The nested driver-result check is at 0x72a726, 0x72a72f and 0x72a735. Exact address groups and ELF byte checks are retained in `candidate/instruction-attribution/measured-attribution.json`, together with raw profiles, full disassemblies and compact per-address tables. No extra libc copy calls replaced the removed moves: candidate Do/Fold remain at 44/61 calls per resume, versus base 54/75. The discarded redispatch-4 in-place experiment had 46/63; it was not reintroduced.

`queue_discriminant_checks`: 0x707c6, 0x707ca, 0x78e56, 0x78e5a, 0xdbc9d, 0xdbca1, 0xe2966, 0xe296a, 0xe7479, 0xe747d, 0xed546, 0xed54a, 0xf02dd, 0xf02e1, 0xf0c20, 0xf0c24, 0xf25e1, 0xf25e5, 0xf732f, 0xf7333, 0x1007d0, 0x1007d4, 0x10510c, 0x105110, 0x10854a, 0x10854e, 0x109616, 0x10961a, 0x109cd9, 0x109cdd, 0x10a815, 0x10a819, 0x4096a2, 0x4096a5, 0x729b04, 0x729b06, 0x729ec9, 0x729ecb, 0x72a94f, 0x72a957, 0x72a95d, 0x72a965, 0x72cdb8, 0x72cdbf.

`legacy_Value_match`: 0x729d81, 0x729d89.

`completion_decode`: 0x72a617, 0x72a61b, 0x72a625, 0x72a629, 0x72a62d.

`boundary_fault_check`: 0x72bb28, 0x72bb30.

`nested_driver_result_check`: 0x72a726, 0x72a72f, 0x72a735.

`boundary_fault_eager_payload_loads_and_clear`: no hot-path addresses.

```json
[
  {
    "arm": "base",
    "label": "table",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 11844,
    "calls_2N": 22847,
    "calls_per_resume": 55.015
  },
  {
    "arm": "base",
    "label": "do-chain",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 11636,
    "calls_2N": 22436,
    "calls_per_resume": 54.0
  },
  {
    "arm": "base",
    "label": "call-repeat",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 14065,
    "calls_2N": 27272,
    "calls_per_resume": 66.035
  },
  {
    "arm": "base",
    "label": "cleanup",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 10541,
    "calls_2N": 20141,
    "calls_per_resume": 48.0
  },
  {
    "arm": "base",
    "label": "fold",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 15862,
    "calls_2N": 30862,
    "calls_per_resume": 75.0
  },
  {
    "arm": "candidate",
    "label": "table",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 10236,
    "calls_2N": 19639,
    "calls_per_resume": 47.015
  },
  {
    "arm": "candidate",
    "label": "do-chain",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 9628,
    "calls_2N": 18428,
    "calls_per_resume": 44.0
  },
  {
    "arm": "candidate",
    "label": "call-repeat",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 12462,
    "calls_2N": 24069,
    "calls_per_resume": 58.035
  },
  {
    "arm": "candidate",
    "label": "cleanup",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 8917,
    "calls_2N": 16917,
    "calls_per_resume": 40.0
  },
  {
    "arm": "candidate",
    "label": "fold",
    "libc_routine": "0x00000000001b1980",
    "calls_N": 13056,
    "calls_2N": 25256,
    "calls_per_resume": 61.0
  }
]
```

Callgrind totals use at least three fresh N/2N process pairs per arm. Delta is the difference of arm means; a total demonstrates added work only when delta exceeds the larger arm range, or address-level evidence independently demonstrates extra work. All per-function sums reconcile to raw summary counts. Each annotation uses --threshold=100 --auto=no. The five final resume totals are:

| Workload | Base slopes | Candidate slopes | Base mean | Candidate mean | Mean delta | Base range | Candidate range |
| --- | --- | --- | --- | --- | --- | --- | --- |
| table | 15904.355000, 15891.715000, 15911.700000 | 15910.030000, 15925.225000, 15943.405000 | 15902.590000 | 15926.220000 | +23.630000 | 19.985000 | 33.375000 |
| do-chain | 15209.705000, 15221.200000, 15211.565000 | 15204.640000, 15331.755000, 15243.695000 | 15214.156667 | 15260.030000 | +45.873333 | 11.495000 | 127.115000 |
| call-repeat | 25323.155000, 25379.810000, 25427.340000 | 25368.715000, 25373.810000, 25375.385000 | 25376.768333 | 25372.636667 | -4.131667 | 104.185000 | 6.670000 |
| cleanup | 14218.240000, 14210.950000, 14190.385000 | 14258.645000, 14251.475000, 14256.275000 | 14206.525000 | 14255.465000 | +48.940000 | 27.855000 | 7.170000 |
| fold | 24709.455000, 24533.540000, 24531.285000 | 24509.565000, 24559.270000, 24467.590000 | 24591.426667 | 24512.141667 | -79.285000 | 178.170000 | 91.680000 |

`table` — per-function mean changes (largest absolute 20; the complete reconciled list is in `candidate/callgrind-means.json`).

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1123.000000 | +840.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 593.000000 | 0.000000 | -593.000000 |
| 0x00000000001b1980 | 1558.950000 | 1318.950000 | -240.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 493.000000 | 508.000000 | +15.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 273.000000 | 284.000000 | +11.000000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 9.000000 | 0.000000 | -9.000000 |
| rum_core::expr::intern_symbol_text | 196.540000 | 200.728333 | +4.188333 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| 0x00000000001b1220 | 165.095000 | 168.900000 | +3.805000 |
| rum_core::expr::Symbol::system | 193.675000 | 190.490000 | -3.185000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 134.995000 | 133.713333 | -1.281667 |
| rum_core::expr::intern_text | 382.726667 | 381.571667 | -1.155000 |
| brief608_resume::main | 155.000000 | 154.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 0.030000 | 0.976667 | +0.946667 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | -0.261667 | 0.036667 | +0.298333 |
| hashbrown::raw::RawTable<T,A>::reserve_rehash | 0.000000 | 0.013333 | +0.013333 |

`do-chain` — per-function mean changes (largest absolute 20; the complete reconciled list is in `candidate/callgrind-means.json`).

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1252.000000 | +969.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 677.000000 | 0.000000 | -677.000000 |
| 0x00000000001b1980 | 1485.775000 | 1185.775000 | -300.000000 |
| rum_core::expr::intern_text | 377.656667 | 406.123333 | +28.466667 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 532.000000 | 549.000000 | +17.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 319.000000 | 334.000000 | +15.000000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 21.000000 | 12.000000 | -9.000000 |
| rum_core::expr::Symbol::system | 191.625000 | 198.528333 | +6.903333 |
| rum_core::expr::intern_symbol_text | 189.406667 | 183.931667 | -5.475000 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 214.325000 | 218.658333 | +4.333333 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 135.125000 | 130.963333 | -4.161667 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| 0x00000000001b1220 | 156.431667 | 160.235000 | +3.803333 |
| _mi_page_free_collect | -2.280000 | -1.140000 | +1.140000 |
| brief608_resume::main | 140.000000 | 139.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.496667 | -0.070000 | +0.426667 |

`call-repeat` — per-function mean changes (largest absolute 20; the complete reconciled list is in `candidate/callgrind-means.json`).

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1123.000000 | +840.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 593.000000 | 0.000000 | -593.000000 |
| 0x00000000001b1980 | 2321.055000 | 2081.055000 | -240.000000 |
| rum_core::expr::Symbol::system | 284.290000 | 242.613333 | -41.676667 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| 0x00000000001b1220 | 238.738333 | 259.135000 | +20.396667 |
| rum_core::expr::intern_symbol_text | 304.996667 | 293.138333 | -11.858333 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 415.000000 | 426.000000 | +11.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 267.000000 | 278.000000 | +11.000000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 9.000000 | 0.000000 | -9.000000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 196.731667 | 203.348333 | +6.616667 |
| rum_core::expr::intern_text | 570.370000 | 574.985000 | +4.615000 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| brief608_resume::main | 144.750000 | 143.750000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.663333 | 0.311667 | +0.975000 |
| _mi_page_free_collect | 15.770000 | 16.530000 | +0.760000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.000000 | 0.025000 | +0.025000 |

`cleanup` — per-function mean changes (largest absolute 20; the complete reconciled list is in `candidate/callgrind-means.json`).

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1123.000000 | +840.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 593.000000 | 0.000000 | -593.000000 |
| 0x00000000001b1980 | 1271.000000 | 1031.000000 | -240.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 805.000000 | 836.000000 | +31.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 265.000000 | 276.000000 | +11.000000 |
| rum_core::expr::Symbol::system | 183.591667 | 193.921667 | +10.330000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 21.000000 | 12.000000 | -9.000000 |
| rum_core::expr::intern_symbol_text | 198.610000 | 192.810000 | -5.800000 |
| 0x00000000001b1220 | 156.520000 | 160.575000 | +4.055000 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 131.896667 | 133.820000 | +1.923333 |
| rum_core::expr::intern_text | 380.013333 | 381.878333 | +1.865000 |
| brief608_resume::main | 140.000000 | 139.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.038333 | 0.875000 | +0.913333 |
| _mi_page_free_collect | -1.960000 | -2.200000 | -0.240000 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | -0.068333 | -0.175000 | -0.106667 |

`fold` — per-function mean changes (largest absolute 20; the complete reconciled list is in `candidate/callgrind-means.json`).

| Function | Base Ir/resume | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint | 283.000000 | 1510.000000 | +1227.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::drive_resuming_frames | 845.000000 | 0.000000 | -845.000000 |
| 0x00000000001b1980 | 2073.775000 | 1653.775000 | -420.000000 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 442.000000 | 477.000000 | +35.000000 |
| rum_core::expr::Symbol::system | 226.805000 | 195.661667 | -31.143333 |
| rum_runtime::runtime::suspension::core::SuspendedEvaluation::push_back | 30.000000 | 0.000000 | -30.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::queue::FrameQueue> | 0.000000 | 28.000000 | +28.000000 |
| core::ptr::drop_in_place<alloc::collections::vec_deque::VecDeque<alloc::sync::Arc<dyn rum_runtime::runtime::suspension::core::SuspensionFrame>>> | 25.000000 | 0.000000 | -25.000000 |
| rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::finish_input_required_request | 571.000000 | 590.000000 | +19.000000 |
| 0x00000000001b1220 | 489.121667 | 470.836667 | -18.285000 |
| core::ptr::drop_in_place<core::result::Result<rum_core::expr::Expr,rum_runtime::runtime::session::session_state::EvalError>> | 9.000000 | 0.000000 | -9.000000 |
| rum_core::expr::intern_symbol_text | 196.376667 | 191.651667 | -4.725000 |
| rum_runtime::runtime::builtins_input::<impl rum_runtime::runtime::model::session::Session>::drive_input_request | 95.000000 | 91.000000 | -4.000000 |
| brief608_resume::main | 140.000000 | 139.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::model::requests::SessionRequestError> | 31.000000 | 30.000000 | -1.000000 |
| core::ptr::drop_in_place<rum_runtime::runtime::suspension::core::SuspendedEvaluation> | 20.000000 | 19.000000 | -1.000000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 308.561667 | 309.321667 | +0.760000 |
| rum_core::expr::intern_text | 381.160000 | 381.595000 | +0.435000 |
| _mi_malloc_generic | 49.260000 | 49.515000 | +0.255000 |
| mi_theap_malloc_zero_aligned_at_generic | 269.880000 | 270.125000 | +0.245000 |

Every ordinary label with either initial sample above its recorded floor received five fresh perf slopes and three Callgrind pairs per arm. Each perf spread is (max-min)/median*100; the revised floor is max(0.05%, base spread, candidate spread). Callgrind budgets were fixed before collection. Exact original probe statements and only-N substitutions were checked; source paths and their hashes are retained.

`lazy-range-normal-small`: run-to-run hashing/layout variation: mean difference does not exceed the larger arm range; changed resume mechanisms do not run.

| Arm | Five perf slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 54109.464350, 54055.249300, 54138.217700, 54198.931450, 54108.342400 | 54109.464350 | 0.265540 |
| candidate | 54108.313800, 54083.919300, 54135.205200, 54109.228750, 54158.540150 | 54109.228750 | 0.137908 |

| Perf floor % | Median delta % | CG base slopes | CG candidate slopes | CG mean delta | Larger CG range |
| --- | --- | --- | --- | --- | --- |
| 0.265540 | -0.000435 | 54084.739837, 54167.181572, 54352.577236 | 54195.116531, 54305.138211, 54187.195122 | +27.650407 | 267.837398 |

The measured loop requests no input, so the resume driver and queue operations never run. Top-level boundary checks occur a fixed number of times per process and cancel in the N/2N slope; they are not inside the loop body. Raw changed-resume-function slopes are empty.

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::Symbol::system | 295.100271 | 358.257453 | +63.157182 |
| rum_core::expr::intern_text | 799.953026 | 763.710027 | -36.242999 |
| 0x00000000001b1220 | 385.674797 | 360.726287 | -24.948509 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 1139.312556 | 1158.953026 | +19.640470 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 366.068654 | 368.930443 | +2.861789 |
| rum_core::expr::intern_symbol_text | 390.242999 | 392.915086 | +2.672087 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -0.174345 | 0.211382 | +0.385727 |
| mi_theap_malloc_zero_aligned_at_generic | 86.241192 | 86.108401 | -0.132791 |
| _mi_os_alloc | 0.000000 | 0.074977 | +0.074977 |
| _mi_prim_alloc | 0.000000 | 0.058717 | +0.058717 |

`lazy-table-construct-full`: run-to-run hashing/layout variation: mean difference does not exceed the larger arm range; changed resume mechanisms do not run.

| Arm | Five perf slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 24068.097400, 24120.766300, 23989.797700, 24068.385950, 24097.674800 | 24068.385950 | 0.544152 |
| candidate | 24131.851100, 24036.016050, 24057.948200, 24047.436800, 24094.790250 | 24057.948200 | 0.398351 |

| Perf floor % | Median delta % | CG base slopes | CG candidate slopes | CG mean delta | Larger CG range |
| --- | --- | --- | --- | --- | --- |
| 0.544152 | -0.043367 | 24120.504212, 24032.965102, 24031.637786 | 24030.818291, 24096.925391, 24075.377858 | +6.004813 | 88.866426 |

The measured loop requests no input, so the resume driver and queue operations never run. Top-level boundary checks occur a fixed number of times per process and cancel in the N/2N slope; they are not inside the loop body. Raw changed-resume-function slopes are empty.

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::intern_symbol_text | 386.697553 | 392.535901 | +5.838347 |
| 0x00000000001b1220 | 190.137585 | 193.852387 | +3.714801 |
| rum_core::expr::Symbol::system | 306.566787 | 303.743281 | -2.823506 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 268.304854 | 267.582030 | -0.722824 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 431.865624 | 432.585239 | +0.719615 |
| rum_core::expr::intern_text | 776.122343 | 775.595267 | -0.527076 |
| 0x000000000001b870 | 0.241877 | 0.120337 | -0.121540 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.013237 | -0.035700 | -0.048937 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 0.004011 | -0.009226 | -0.013237 |
| 0x00000000001b57e0 | 0.010830 | 0.000000 | -0.010830 |

`lazy-permutations-take-small`: run-to-run hashing/layout variation: mean difference does not exceed the larger arm range; changed resume mechanisms do not run.

| Arm | Five perf slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 28199.867950, 28198.950850, 28198.944250, 28198.536150, 28245.963250 | 28198.950850 | 0.168187 |
| candidate | 28192.489200, 28192.193450, 28192.349650, 28191.765850, 28222.464800 | 28192.349650 | 0.108891 |

| Perf floor % | Median delta % | CG base slopes | CG candidate slopes | CG mean delta | Larger CG range |
| --- | --- | --- | --- | --- | --- |
| 0.168187 | -0.023409 | 28576.459803, 28573.895628, 28583.715092 | 28552.531735, 28557.119887, 28531.686883 | -30.910672 | 25.433004 |

The measured loop requests no input, so the resume driver and queue operations never run. Top-level boundary checks occur a fixed number of times per process and cancel in the N/2N slope; they are not inside the loop body. Raw changed-resume-function slopes are empty.

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 891.276916 | 875.175835 | -16.101081 |
| rum_core::expr::Symbol::system | 418.063940 | 409.281617 | -8.782323 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 588.032440 | 583.723084 | -4.309356 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2 | 1740.278796 | 1737.711331 | -2.567466 |
| rum_core::expr::intern_symbol_text | 194.781382 | 196.425482 | +1.644100 |
| rum_core::expr::intern_text | 382.236483 | 381.435355 | -0.801128 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 0.058768 | -0.106723 | -0.165491 |
| 0x00000000001b1220 | 530.202163 | 530.267043 | +0.064880 |
| _mi_os_alloc | -0.039022 | 0.000000 | +0.039022 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 0.034791 | -0.001881 | -0.036671 |

`lazy-tuples-take-small`: run-to-run hashing/layout variation: mean difference does not exceed the larger arm range; changed resume mechanisms do not run.

| Arm | Five perf slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 26604.573100, 26580.524450, 26628.880950, 26566.294600, 26637.312650 | 26604.573100 | 0.266939 |
| candidate | 26585.416300, 26585.248000, 26614.755400, 26585.617150, 26584.807950 | 26585.416300 | 0.112646 |

| Perf floor % | Median delta % | CG base slopes | CG candidate slopes | CG mean delta | Larger CG range |
| --- | --- | --- | --- | --- | --- |
| 0.266939 | -0.072006 | 26978.630319, 26948.736702, 26946.586436 | 26957.856383, 26961.085106, 26907.147606 | -15.954787 | 53.937500 |

The measured loop requests no input, so the resume driver and queue operations never run. Top-level boundary checks occur a fixed number of times per process and cancel in the N/2N slope; they are not inside the loop body. Raw changed-resume-function slopes are empty.

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::Symbol::system | 429.403369 | 401.716312 | -27.687057 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 887.877216 | 899.317376 | +11.440160 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2 | 1444.250000 | 1445.630319 | +1.380319 |
| rum_core::expr::intern_text | 382.703457 | 381.674645 | -1.028812 |
| 0x00000000001b1220 | 832.123227 | 831.805408 | -0.317819 |
| 0x000000000001b870 | -0.312500 | -0.011968 | +0.300532 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 587.587766 | 587.734043 | +0.146277 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 0.173759 | 0.066046 | -0.107713 |
| rum_core::expr::intern_symbol_text | 196.599734 | 196.689716 | +0.089982 |
| alloc::sync::Arc<T,A>::drop_slow'2 | 218.312500 | 218.228723 | -0.083777 |

`lazy-depth-map-first-256`: run-to-run hashing/layout variation: mean difference does not exceed the larger arm range; changed resume mechanisms do not run.

| Arm | Five perf slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 3723175.848000, 3716205.053000, 3722734.929000, 3722910.542000, 3736085.177000 | 3722910.542000 | 0.533994 |
| candidate | 3736087.215000, 3723177.394000, 3722825.164000, 3716192.536000, 3722994.264000 | 3722994.264000 | 0.534373 |

| Perf floor % | Median delta % | CG base slopes | CG candidate slopes | CG mean delta | Larger CG range |
| --- | --- | --- | --- | --- | --- |
| 0.534373 | +0.002249 | 3740283.800000, 3738204.200000, 3713003.200000 | 3737891.800000, 3722959.800000, 3722620.400000 | -2673.066667 | 27280.600000 |

The measured loop requests no input, so the resume driver and queue operations never run. Top-level boundary checks occur a fixed number of times per process and cancel in the N/2N slope; they are not inside the loop body. Raw changed-resume-function slopes are empty.

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2 | 57593.000000 | 56483.666667 | -1109.333333 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 133628.133333 | 132564.733333 | -1063.400000 |
| rum_core::expr::Symbol::system | 39499.533333 | 38936.466667 | -563.066667 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 22728.133333 | 22394.466667 | -333.666667 |
| rum_core::expr::intern_text | -388.266667 | -95.933333 | +292.333333 |
| rum_core::expr::intern_symbol_text | -66.666667 | 15.800000 | +82.466667 |
| 0x000000000001b870 | -47.000000 | -1.800000 | +45.200000 |
| 0x00000000001b1220 | 97841.400000 | 97818.733333 | -22.666667 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 4.933333 | -9.800000 | -14.733333 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | 10.866667 | -1.000000 | -11.866667 |

`lazy-depth-select-restart-256`: run-to-run hashing/layout variation: mean difference does not exceed the larger arm range; changed resume mechanisms do not run.

| Arm | Five perf slopes | Median | Own spread % |
| --- | --- | --- | --- |
| base | 3652112.136000, 3664996.657000, 3690930.995000, 3652394.689000, 3665017.707000 | 3664996.657000 | 1.059179 |
| candidate | 3691033.063000, 3665398.317000, 3665542.440000, 3691492.902000, 3690960.920000 | 3690960.920000 | 0.706986 |

| Perf floor % | Median delta % | CG base slopes | CG candidate slopes | CG mean delta | Larger CG range |
| --- | --- | --- | --- | --- | --- |
| 1.059179 | +0.708439 | 3663885.600000, 3664097.800000, 3664956.200000 | 3664314.200000, 3666570.200000, 3665543.200000 | +1162.666667 | 2256.000000 |

The measured loop requests no input, so the resume driver and queue operations never run. Top-level boundary checks occur a fixed number of times per process and cancel in the N/2N slope; they are not inside the loop body. Raw changed-resume-function slopes are empty.

| Function | Base Ir/iteration | Candidate | Delta |
| --- | --- | --- | --- |
| rum_core::expr::Symbol::system | 375.733333 | 823.466667 | +447.733333 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom | 222593.866667 | 222986.466667 | +392.600000 |
| hashbrown::map::HashMap<K,V,S,A>::insert | 45175.400000 | 45468.533333 | +293.133333 |
| rum_core::expr::intern_text | 3.200000 | 122.400000 | +119.200000 |
| rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2 | 66380.000000 | 66336.666667 | -43.333333 |
| 0x000000000001b870 | -2.200000 | -22.400000 | -20.200000 |
| rum_core::expr::intern_symbol_text | 28.800000 | 10.133333 | -18.666667 |
| std::sync::poison::once::Once::call_once_force::{{closure}} | -14.400000 | 2.600000 | +17.000000 |
| 0x00000000001b1220 | 97655.000000 | 97666.533333 | +11.533333 |
| rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol | 1.666667 | -9.000000 | -10.666667 |

The six ordinary investigations all satisfy the amended classification rule: each Callgrind mean delta is below its larger arm range, every perf median delta fits the newly measured own-arm spread floor, and none of their measured loops enters the changed resume mechanisms. The source and per-function evidence are from this final candidate. Positive individual process differences are retained in the tables rather than treated as deterministic added work.

`lazy-tuples-take-small` is the only ordinary label above the frozen floor in both initial perf pairs (+0.194899% / +0.339206%, floor 0.179754%). Its five-per-arm median delta is -0.072006%, own-arm floor 0.266939%, and three-pair Callgrind mean delta is -15.954787 instructions/iteration. The earlier `lazy-table-construct-full` investigation is also re-earned on this binary: median -0.043367%, own-arm floor 0.544152%, Callgrind mean +6.004814 within its process range. No ordinary added-work defect remains under the new rule.

Among the resume workloads, only call-repeat exceeds its frozen floor in both initial perf pairs: +37.158 and +20.967 instructions/resume (+0.151391% / +0.085197%, floor 0.062048%). Both are covered by its 45 measured allowed instructions. The other four workloads do not exceed their thresholds in both pairs. Their individual positive samples remain disclosed; the three-process means and address attribution above decide whether an unexplained mechanism remains.

The full timing panel ran last, on both one-unit binaries, with profiling off, scale 1, seven repetitions, and retained-module GC batches 3. Every row uses in-kernel AbsoluteTiming backed by monotonic Instant. Redispatch-4 timing signals are not used because of the director’s recorded GPU overlap. Timing remains an indication, not a substitute for the instruction gate.

The final panel has 120 matching labels: 71 positive candidate deltas, 49 nonpositive, and 8 above +5%. The largest increase is do-count-null, +23.399378% (0.000246045 -> 0.000303618 seconds). These are the collected in-kernel measurements; the runs were not repeated or filtered according to their results.

Host observation: each arm waited for five consecutive quiet one-second preflight samples. A read-only /proc monitor then sampled competing processes once per second, excluding its own measurement descendants, and would mark any process using at least 0.15 CPU core as competing activity. Base recorded two measurement samples; candidate recorded one; none reported such activity. This is sampled CPU evidence, not proof that no shorter competing burst occurred. All other agent builds, tests, perf and Callgrind jobs had ended before either arm.

GPU observation failed: nvidia-smi returned exit 9, reporting that it could not communicate with the NVIDIA driver, and the available DRM entries expose no usable gpu_busy_percent. This does not establish that the host GPU was idle or absent. The optional director host-activity question was unanswered when the pair ran. The fully idle-host condition therefore remains unverified, and these rows are qualified indications. Raw observer records are timing-base-host.json and timing-candidate-host.json; GPU/DRM checks are gpu-observer-check.json and drm-observer-check.json.

Rows above +5%:

| Label | Base seconds | Candidate seconds | Delta % |
| --- | ---: | ---: | ---: |
| `loop-empty` | 0.000257363 | 0.000276022 | +7.250071 |
| `do-count-null` | 0.000246045 | 0.000303618 | +23.399378 |
| `literal-integer` | 0.000089406 | 0.000095050 | +6.312775 |
| `literal-string` | 0.000098152 | 0.000106776 | +8.786372 |
| `if-true` | 0.000690615 | 0.000728185 | +5.440079 |
| `module-chain-escape-large` | 0.000093963 | 0.000100341 | +6.787778 |
| `table-iterator-pair-list` | 0.000059639 | 0.000064703 | +8.491088 |
| `downvalue-pattern-miss` | 0.000278166 | 0.000293679 | +5.576886 |

Complete panel (seconds are copied from the original timings.tsv cells):

| Label | Iterations | Base seconds | Candidate seconds | Delta % |
| --- | ---: | ---: | ---: | ---: |
| `association-part-key-read-width4000` | 40 | 0.0025567 | 0.00228331 | -10.693081 |
| `association-part-key-read-width32000` | 40 | 0.00975665 | 0.00851017 | -12.775697 |
| `association-part-positional-read-width4000` | 40 | 3.0853e-05 | 3.0385e-05 | -1.516870 |
| `association-part-positional-read-width32000` | 40 | 3.1664e-05 | 3.0416e-05 | -3.941385 |
| `loop-empty` | 5000 | 0.000257363 | 0.000276022 | +7.250071 |
| `do-count-null` | 10000 | 0.000246045 | 0.000303618 | +23.399378 |
| `do-iterator-null` | 1000 | 5.8788e-05 | 6.0981e-05 | +3.730353 |
| `table-count-null` | 5000 | 8.3185e-05 | 7.7064e-05 | -7.358298 |
| `table-iterator-null` | 1000 | 1.8982e-05 | 1.8549e-05 | -2.281108 |
| `table-iterator-symbol` | 1000 | 6.965e-06 | 6.816e-06 | -2.139268 |
| `literal-integer` | 2000 | 8.9406e-05 | 9.505e-05 | +6.312775 |
| `literal-string` | 2000 | 9.8152e-05 | 0.000106776 | +8.786372 |
| `symbol-ownvalue-read` | 2000 | 0.000217807 | 0.000223966 | +2.827733 |
| `symbol-ownvalue-guarded-read` | 1000 | 0.000184117 | 0.000189006 | +2.655377 |
| `set-ownvalue` | 1000 | 0.000257267 | 0.00026072 | +1.342185 |
| `list-literal-small` | 1000 | 5.4539e-05 | 5.506e-05 | +0.955280 |
| `head-list` | 2000 | 0.000906592 | 0.000907425 | +0.091883 |
| `length-list` | 2000 | 0.000893521 | 0.000910298 | +1.877628 |
| `part-list` | 2000 | 0.00106853 | 0.00107186 | +0.311643 |
| `sameq-integer` | 2000 | 0.000616242 | 0.000632054 | +2.565875 |
| `sameq-list` | 1000 | 0.000602763 | 0.000618676 | +2.640009 |
| `if-true` | 2000 | 0.000690615 | 0.000728185 | +5.440079 |
| `module-local` | 500 | 0.00070847 | 0.000724888 | +2.317388 |
| `module-chain-escape-small` | 96 | 5.3085e-05 | 5.195e-05 | -2.138080 |
| `module-chain-escape-large` | 192 | 9.3963e-05 | 0.000100341 | +6.787778 |
| `block-local` | 500 | 0.000469029 | 0.000469871 | +0.179520 |
| `block-symbol-only-repeat` | 1000 | 0.000304634 | 0.000308989 | +1.429584 |
| `block-init-null-repeat` | 1000 | 0.000355677 | 0.000359679 | +1.125178 |
| `block-init-read-repeat` | 1000 | 0.000403917 | 0.000414565 | +2.636185 |
| `with-local` | 500 | 0.000474142 | 0.000477817 | +0.775084 |
| `with-deep-body-small` | 500 | 0.00176939 | 0.00182751 | +3.284748 |
| `table-small` | 200 | 0.000183479 | 0.000189168 | +3.100627 |
| `table-iterator-plus` | 1000 | 3.6433e-05 | 3.6827e-05 | +1.081437 |
| `table-iterator-pair-list` | 1000 | 5.9639e-05 | 6.4703e-05 | +8.491088 |
| `do-small` | 200 | 0.000883391 | 0.000895509 | +1.371760 |
| `d-nested-sin-small` | 200 | 0.00542595 | 0.0051906 | -4.337489 |
| `downvalue-exact-hit` | 2000 | 0.000761587 | 0.000763213 | +0.213502 |
| `downvalue-exact-miss` | 2000 | 0.000535562 | 0.000545565 | +1.867758 |
| `downvalue-pattern-hit` | 1000 | 0.000961945 | 0.000945586 | -1.700617 |
| `downvalue-pattern-miss` | 1000 | 0.000278166 | 0.000293679 | +5.576886 |
| `downvalue-exact-miss-repeat` | 5000 | 0.00122744 | 0.00126629 | +3.165124 |
| `downvalue-pattern-miss-repeat` | 5000 | 0.00126433 | 0.00131085 | +3.679419 |
| `downvalue-guarded-hit` | 1000 | 0.00187265 | 0.0018958 | +1.236216 |
| `downvalue-guarded-decline-fallback` | 1000 | 0.00138971 | 0.0014109 | +1.524779 |
| `upvalue-guarded-hit` | 1000 | 0.00232728 | 0.00235093 | +1.016208 |
| `subvalue-unguarded-hit` | 1000 | 0.00138763 | 0.00139569 | +0.580846 |
| `subvalue-unguarded-miss` | 1000 | 0.000765516 | 0.000762702 | -0.367595 |
| `subvalue-guarded-hit` | 1000 | 0.00239923 | 0.00245247 | +2.219045 |
| `default-unguarded-hit` | 1000 | 0.00127963 | 0.00127512 | -0.352446 |
| `default-unguarded-miss` | 1000 | 0.00106301 | 0.00107011 | +0.667915 |
| `default-guarded-hit` | 1000 | 0.00162221 | 0.00161554 | -0.411167 |
| `matchq-integer-success` | 1000 | 0.000280354 | 0.000282394 | +0.727651 |
| `matchq-integer-fail` | 1000 | 0.00028778 | 0.000286862 | -0.318994 |
| `matcher-single-tiny` | 1000 | 0.000672658 | 0.000672412 | -0.036571 |
| `matcher-single-wide-small` | 800 | 3.9645e-05 | 3.9778e-05 | +0.335477 |
| `matcher-single-wide-large` | 1600 | 7.7936e-05 | 7.7514e-05 | -0.541470 |
| `replaceall-simple` | 500 | 0.000359903 | 0.000350727 | -2.549576 |
| `rule-rhs-deep-small` | 500 | 0.00295782 | 0.00294417 | -0.461489 |
| `rewrite-guarded-hit` | 500 | 0.0017897 | 0.00178101 | -0.485556 |
| `rewrite-guarded-decline-fallback` | 500 | 0.00255329 | 0.00254586 | -0.290997 |
| `cases-wide-no-match` | 1600 | 0.00014644 | 0.000144706 | -1.184103 |
| `count-wide-no-match` | 1600 | 0.000136583 | 0.000137157 | +0.420257 |
| `position-wide-no-match` | 1600 | 0.000168713 | 0.00017014 | +0.845815 |
| `deletecases-wide-no-match` | 1600 | 0.000167035 | 0.000166313 | -0.432245 |
| `deletecases-wide-association-no-match` | 1600 | 0.000419482 | 0.000425305 | +1.388141 |
| `deletecases-leaf-delete-small` | 500 | 0.00216866 | 0.0021747 | +0.278513 |
| `replace-wide-no-match` | 1600 | 7.7382e-05 | 7.8907e-05 | +1.970743 |
| `replace-leaf-rebuild-small` | 500 | 0.00264138 | 0.0026581 | +0.633002 |
| `take-multispec-small` | 500 | 0.00161991 | 0.00160615 | -0.849430 |
| `replacepart-deep-path-small` | 500 | 0.00129996 | 0.00130342 | +0.266162 |
| `replacepart-wide-rule-list` | 1600 | 0.000289216 | 0.000301347 | +4.194443 |
| `replacepart-guarded-hit` | 500 | 0.000532144 | 0.000529868 | -0.427704 |
| `replacepart-guarded-decline-fallback` | 500 | 0.000588054 | 0.000583879 | -0.709969 |
| `string-repeat-small` | 500 | 0.000182948 | 0.00018699 | +2.209371 |
| `character-range-small` | 200 | 0.000130572 | 0.000130375 | -0.150875 |
| `stringtrim-literal-list-4000` | 100 | 0.000656419 | 0.000656326 | -0.014168 |
| `stringtrim-literal-list-32000` | 100 | 0.00503706 | 0.0050427 | +0.111970 |
| `stringtrim-single-literal-4000` | 100 | 0.000634556 | 0.000635183 | +0.098809 |
| `stringtrim-single-literal-32000` | 100 | 0.00502196 | 0.00502207 | +0.002190 |
| `stringtrim-whitespace-4000` | 100 | 0.000626893 | 0.000630189 | +0.525768 |
| `stringtrim-whitespace-32000` | 100 | 0.00500916 | 0.00501335 | +0.083647 |
| `stringreplace-single-literal-unguarded` | 200 | 0.000378744 | 0.000383509 | +1.258106 |
| `stringreplace-pattern-unguarded` | 500 | 0.00104731 | 0.00103982 | -0.715166 |
| `stringreplace-guarded-hit` | 500 | 0.000687994 | 0.000688716 | +0.104943 |
| `stringreplace-guarded-decline-fallback` | 500 | 0.000827761 | 0.000825471 | -0.276650 |
| `appendto-list-incremental-small` | 250 | 0.000548062 | 0.000481733 | -12.102463 |
| `appendto-list-incremental-large` | 500 | 0.00161526 | 0.00168329 | +4.211706 |
| `appendto-part-outer1000-small` | 100 | 0.00479494 | 0.00217945 | -54.546876 |
| `appendto-part-outer1000-large` | 200 | 0.00912398 | 0.004413 | -51.632950 |
| `association-set-key-incremental-small` | 100 | 0.00272219 | 0.00144611 | -46.876963 |
| `association-set-key-incremental-large` | 200 | 0.00549204 | 0.00543 | -1.129635 |
| `association-small-literal-repeat` | 500 | 0.000749522 | 0.000735164 | -1.915621 |
| `merge-nested-small` | 200 | 0.00116636 | 0.00116414 | -0.190336 |
| `set-part-fixed100-len100` | 100 | 0.000143359 | 0.000141812 | -1.079109 |
| `set-part-fixed100-len800` | 100 | 0.000616834 | 0.000596334 | -3.323423 |
| `set-part-fill-small` | 200 | 0.000433924 | 0.000429828 | -0.943944 |
| `set-part-fill-large` | 400 | 0.00140087 | 0.00136666 | -2.442054 |
| `upvalue-candidates-tiny` | 1000 | 0.00190408 | 0.00192255 | +0.970022 |
| `upvalue-candidates-wide-small` | 1600 | 0.000151044 | 0.000154215 | +2.099388 |
| `upvalue-candidates-wide-large` | 3200 | 0.000302436 | 0.000307047 | +1.524620 |
| `sort-callback-small` | 100 | 0.000163999 | 0.000164672 | +0.410368 |
| `sort-callback-large` | 200 | 0.000374 | 0.000376382 | +0.636898 |
| `sortby-callback-small` | 100 | 0.000198041 | 0.000199027 | +0.497877 |
| `sortby-callback-large` | 200 | 0.000442331 | 0.000442209 | -0.027581 |
| `reversesort-callback-small` | 100 | 0.000157803 | 0.00016197 | +2.640634 |
| `reversesort-callback-large` | 200 | 0.000352982 | 0.0003647 | +3.319716 |
| `keysort-callback-small` | 50 | 8.312e-05 | 8.2699e-05 | -0.506497 |
| `keysort-callback-large` | 100 | 0.000182469 | 0.000182793 | +0.177564 |
| `keysortby-callback-small` | 25 | 4.5188e-05 | 4.4817e-05 | -0.821014 |
| `keysortby-callback-large` | 50 | 9.8012e-05 | 9.7483e-05 | -0.539730 |
| `sort-default-wide` | 1600 | 1.6583e-05 | 1.6521e-05 | -0.373877 |
| `sort-symbol-default-width4000` | 4000 | 0.000296586 | 0.000295025 | -0.526323 |
| `sort-symbol-default-width16000` | 16000 | 0.00123707 | 0.00122981 | -0.586871 |
| `sortby-default-wide` | 1600 | 0.000550527 | 0.000505922 | -8.102237 |
| `reversesort-default-wide` | 1600 | 1.6533e-05 | 1.7092e-05 | +3.381117 |
| `keysort-default-wide` | 1600 | 0.000370822 | 0.000369859 | -0.259693 |
| `keysortby-default-wide` | 1600 | 0.000866899 | 0.000879148 | +1.412967 |
| `valueq-undefined-symbol-definitions-2000` | 20 | 7.646e-06 | 7.897e-06 | +3.282762 |
| `valueq-undefined-symbol-definitions-8000` | 20 | 7.582e-06 | 7.905e-06 | +4.260090 |
| `module-retained-gensym-gc-batch` | 40 | 6.5389e-05 | 6.6622e-05 | +1.885638 |

Ownership replay preserves all 79 payload identities, 59 self-reaching types, dispositions/witnesses and old ledger rows 56–58. UnknownConstruct moves 7711 -> 7712: the only new operation is the explicit unreachable macro in the cold mixed legacy pop. All other differences are source coordinates. Complete changed rows follow.

```text
unknown_attributes.tsv: 1514 -> 1514 lines
unknown_captures.tsv: 2954 -> 2954 lines
unknown_macros.tsv: 3149 -> 3150 lines
unknown_storage.tsv: 94 -> 94 lines
host_payloads.tsv: 79 -> 79 lines
self_reaching.tsv: 59 -> 59 lines
host_payload_ledger.tsv: 80 -> 80 lines
```

```diff
--- unknown_attributes.tsv before
+++ unknown_attributes.tsv replay
@@ -850,7 +850,7 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	4	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	336	9	unresolved attribute/expansion cfg (debug_assertions)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	3	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	621	9	unresolved attribute/expansion cfg (any (debug_assertions , test))
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	625	9	unresolved attribute/expansion cfg (any (debug_assertions , test))
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	10	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , FrameRootTrace)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	16	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	24	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , FrameRootTrace)
@@ -915,7 +915,7 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	26	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , PartialOrd , Ord)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	44	5	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	50	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	231	13	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	236	13	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/fault.rs	7	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , rum_derive :: FrameRootTrace)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/fault.rs	32	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/fault.rs	45	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
--- unknown_captures.tsv before
+++ unknown_captures.tsv replay
@@ -1665,8 +1665,8 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	143	49	closure captures: |query| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	160	22	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	164	21	closure captures: |key| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	225	23	closure captures: || (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	400	22	closure captures: |source| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	228	23	closure captures: || (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	405	22	closure captures: |source| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	32	52	closure captures: |timeout| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	327	18	closure captures: |selector| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	344	50	closure captures: |message| (capture types/modes unresolved)
@@ -1682,16 +1682,16 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	146	22	closure captures: |err| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	164	23	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	168	26	closure captures: |err| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	280	25	closure captures: || (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	317	37	closure captures: || (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	431	35	closure captures: |file_io| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	573	33	closure captures: || (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	738	14	closure captures: |message| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	762	15	closure captures: |line| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	763	14	closure captures: |line| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	765	29	closure captures: |ch| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	789	18	closure captures: |source| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	798	29	closure captures: || (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	282	25	closure captures: || (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	319	37	closure captures: || (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	435	35	closure captures: |file_io| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	577	33	closure captures: || (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	742	14	closure captures: |message| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	766	15	closure captures: |line| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	767	14	closure captures: |line| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	769	29	closure captures: |ch| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	793	18	closure captures: |source| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	802	29	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/binding_substitution.rs	57	64	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/binding_substitution.rs	86	45	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/binding_substitution.rs	107	45	closure captures: || (capture types/modes unresolved)
@@ -1761,11 +1761,11 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/lexical_substitution/iterative.rs	864	37	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/slot_substitution.rs	113	26	closure captures: || (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/substitution/slot_substitution.rs	129	23	closure captures: || (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	208	27	closure captures: |frame| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	213	27	closure captures: |frame| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	254	33	closure captures: |owner| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	710	30	closure captures: |failure| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	758	34	closure captures: |failure| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	218	27	closure captures: |frame| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	259	33	closure captures: |owner| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	715	30	closure captures: |failure| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	763	34	closure captures: |failure| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	36	25	closure captures: |_| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	46	25	closure captures: |_| (capture types/modes unresolved)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	437	22	closure captures: |path| (capture types/modes unresolved)
--- unknown_macros.tsv before
+++ unknown_macros.tsv replay
@@ -1688,11 +1688,11 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_api.rs	913	29	unexpanded macro vec! (ends at 913:78)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_api.rs	948	5	unexpanded macro format! (ends at 951:6)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	52	25	unexpanded macro vec! (ends at 52:57)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	410	12	unexpanded macro matches! (ends at 410:66)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	429	9	unexpanded macro unreachable! (ends at 429:76)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	432	9	unexpanded macro unreachable! (ends at 432:71)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	434	5	unexpanded macro assert! (ends at 434:73)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	438	9	unexpanded macro vec! (ends at 453:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	415	12	unexpanded macro matches! (ends at 415:66)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	434	9	unexpanded macro unreachable! (ends at 434:76)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	437	9	unexpanded macro unreachable! (ends at 437:71)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	439	5	unexpanded macro assert! (ends at 439:73)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_eval.rs	443	9	unexpanded macro vec! (ends at 458:10)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	250	17	unexpanded macro assert! (ends at 253:18)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	256	17	unexpanded macro assert_eq! (ends at 259:18)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	262	17	unexpanded macro panic! (ends at 262:83)
@@ -1710,17 +1710,17 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	694	9	unexpanded macro format! (ends at 694:34)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_helpers.rs	696	9	unexpanded macro format! (ends at 696:41)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	65	9	unexpanded macro debug_assert! (ends at 65:40)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	559	12	unexpanded macro matches! (ends at 565:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	579	13	unexpanded macro debug_assert! (ends at 579:44)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	596	12	unexpanded macro matches! (ends at 602:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	622	9	unexpanded macro assert! (ends at 636:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	645	9	unexpanded macro debug_assert! (ends at 645:40)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	667	17	unexpanded macro vec! (ends at 667:75)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	770	5	unexpanded macro format! (ends at 770:31)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	849	22	unexpanded macro format! (ends at 849:78)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	852	22	unexpanded macro format! (ends at 852:71)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	861	22	unexpanded macro format! (ends at 863:14)
-UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	878	22	unexpanded macro format! (ends at 878:64)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	563	12	unexpanded macro matches! (ends at 569:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	583	13	unexpanded macro debug_assert! (ends at 583:44)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	600	12	unexpanded macro matches! (ends at 606:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	626	9	unexpanded macro assert! (ends at 640:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	649	9	unexpanded macro debug_assert! (ends at 649:40)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	671	17	unexpanded macro vec! (ends at 671:75)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	774	5	unexpanded macro format! (ends at 774:31)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	853	22	unexpanded macro format! (ends at 853:78)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	856	22	unexpanded macro format! (ends at 856:71)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	865	22	unexpanded macro format! (ends at 867:14)
+UnknownConstruct	crates/rum-runtime/src/runtime/session/session_requests.rs	882	22	unexpanded macro format! (ends at 882:64)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	69	9	unexpanded macro debug_assert! (ends at 69:67)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	214	9	unexpanded macro debug_assert! (ends at 214:56)
 UnknownConstruct	crates/rum-runtime/src/runtime/session/session_state.rs	220	9	unexpanded macro debug_assert! (ends at 220:56)
@@ -1781,16 +1781,17 @@
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/collections.rs	93	17	unexpanded macro panic! (ends at 93:99)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/collections.rs	103	13	unexpanded macro vec! (ends at 107:14)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/collections.rs	133	17	unexpanded macro panic! (ends at 133:95)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	258	9	unexpanded macro assert! (ends at 261:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	276	9	unexpanded macro assert! (ends at 279:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	585	9	unexpanded macro debug_assert! (ends at 585:57)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	593	9	unexpanded macro assert! (ends at 596:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	616	9	unexpanded macro debug_assert! (ends at 619:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	682	17	unexpanded macro debug_assert! (ends at 682:66)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	823	1	unexpanded macro macro_rules! (ends at 840:2)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	263	9	unexpanded macro assert! (ends at 266:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	281	9	unexpanded macro assert! (ends at 284:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	590	9	unexpanded macro debug_assert! (ends at 590:57)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	598	9	unexpanded macro assert! (ends at 601:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	621	9	unexpanded macro debug_assert! (ends at 624:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	687	17	unexpanded macro debug_assert! (ends at 687:66)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	828	1	unexpanded macro macro_rules! (ends at 845:2)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	39	13	unexpanded macro debug_assert! (ends at 39:60)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	65	28	unexpanded macro matches! (ends at 65:91)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	25	13	unexpanded macro unreachable! (ends at 25:27)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	88	32	unexpanded macro unreachable! (ends at 88:91)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	9	9	unexpanded macro panic! (ends at 9:63)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	11	5	unexpanded macro assert! (ends at 14:6)
 UnknownConstruct	crates/rum-runtime/src/runtime/suspension/error.rs	24	9	unexpanded macro panic! (ends at 24:74)
--- host_payloads.tsv before
+++ host_payloads.tsv replay
@@ -42,18 +42,18 @@
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_api.rs	732	61	public input rum_runtime::runtime::session::session_api::impl(Session)@63::set_package_search_paths(paths): opaque owned generic parameter I where I : IntoIterator < Item = P > , P : Into < PathBuf > ,
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_api.rs	743	53	public input rum_runtime::runtime::session::session_api::impl(Session)@63::add_package_search_path(path): opaque owned payload impl Into < PathBuf >
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_eval.rs	187	16	public input rum_runtime::runtime::session::session_eval::impl(Session)@24::resume_inspect_rewrite_rules_with_input(input): opaque owned payload impl Into < String >
-UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_eval.rs	394	39	public input rum_runtime::runtime::session::session_eval::impl(Session)@24::eval_file(path): opaque owned payload impl AsRef < Path >
+UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_eval.rs	399	39	public input rum_runtime::runtime::session::session_eval::impl(Session)@24::eval_file(path): opaque owned payload impl AsRef < Path >
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_helpers.rs	331	67	public input rum_runtime::runtime::session::session_helpers::impl(Session)@29::push_message(text): opaque owned payload impl Into < String >
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_helpers.rs	601	15	public input rum_runtime::runtime::session::session_helpers::impl(Session)@29::invalid_call_with_message(text): opaque owned payload impl Into < String >
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_helpers.rs	649	17	public input rum_runtime::runtime::session::session_helpers::impl(Session)@29::invalid_call_with_arg_form(detail): opaque owned payload impl Into < String >
-UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	269	16	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::resume_eval_request_with_input(input): opaque owned payload impl Into < String >
-UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	410	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_file_request(path): opaque owned payload impl AsRef < Path >
-UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	445	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_accepted_file_request(path): opaque owned payload impl AsRef < Path >
+UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	271	16	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::resume_eval_request_with_input(input): opaque owned payload impl Into < String >
+UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	414	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_file_request(path): opaque owned payload impl AsRef < Path >
+UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	449	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_accepted_file_request(path): opaque owned payload impl AsRef < Path >
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	122	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	124	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	134	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@127::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	145	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@127::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	523	57	public input rum_runtime::runtime::suspension::core::impl(Session)@457::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	528	57	public input rum_runtime::runtime::suspension::core::impl(Session)@462::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core/queue.rs	6	12	rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	19	15	rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	21	14	rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer
--- host_payload_ledger.tsv before
+++ host_payload_ledger.tsv replay
@@ -43,19 +43,19 @@
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_api.rs	732	61	public input rum_runtime::runtime::session::session_api::impl(Session)@63::set_package_search_paths(paths): opaque owned generic parameter I where I : IntoIterator < Item = P > , P : Into < PathBuf > ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_api.rs	743	53	public input rum_runtime::runtime::session::session_api::impl(Session)@63::add_package_search_path(path): opaque owned payload impl Into < PathBuf >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_eval.rs	187	16	public input rum_runtime::runtime::session::session_eval::impl(Session)@24::resume_inspect_rewrite_rules_with_input(input): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_eval.rs	394	39	public input rum_runtime::runtime::session::session_eval::impl(Session)@24::eval_file(path): opaque owned payload impl AsRef < Path >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_eval.rs	399	39	public input rum_runtime::runtime::session::session_eval::impl(Session)@24::eval_file(path): opaque owned payload impl AsRef < Path >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_helpers.rs	331	67	public input rum_runtime::runtime::session::session_helpers::impl(Session)@29::push_message(text): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_helpers.rs	601	15	public input rum_runtime::runtime::session::session_helpers::impl(Session)@29::invalid_call_with_message(text): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_helpers.rs	649	17	public input rum_runtime::runtime::session::session_helpers::impl(Session)@29::invalid_call_with_arg_form(detail): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	269	16	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::resume_eval_request_with_input(input): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	410	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_file_request(path): opaque owned payload impl AsRef < Path >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	445	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_accepted_file_request(path): opaque owned payload impl AsRef < Path >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	271	16	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::resume_eval_request_with_input(input): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	414	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_file_request(path): opaque owned payload impl AsRef < Path >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/session/session_requests.rs	449	15	public input rum_runtime::runtime::session::session_requests::impl(Session)@44::eval_accepted_file_request(path): opaque owned payload impl AsRef < Path >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core/queue.rs	6	12	rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	122	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	124	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer	Unbounded	rum_runtime::runtime::suspension::core::PureFrameFinalizer => rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer (impl at crates/rum-runtime/src/runtime/debug_tools/reap.rs:284:1); rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer.rendered -> rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator ; rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator.prefix -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	134	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@127::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	145	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@127::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	523	57	public input rum_runtime::runtime::suspension::core::impl(Session)@457::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	528	57	public input rum_runtime::runtime::suspension::core::impl(Session)@462::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-syntax/src/ast.rs	16	27	public input rum_syntax::ast::impl(SyntaxExpr)@11::integer(value): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-syntax/src/ast.rs	20	26	public input rum_syntax::ast::impl(SyntaxExpr)@11::string(value): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
 UnknownHostPayload	crates/rum-syntax/src/ast.rs	24	26	public input rum_syntax::ast::impl(SyntaxExpr)@11::symbol(value): opaque owned payload impl Into < String >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
```

Historical frontier remains 932 = 10 RET001 / 895 RET002 / 27 RET003. The same take_for_abandon receiver moves 477 -> 482; 931 complete rows are identical. Every unresolved assertion, wrapper count and historical print stays unchanged. As redispatch 3 explicitly requires, the legacy ACCEPTANCE_BASELINE print still says 927 / 10 / 890 / 27; the discovered current frontier and enforced assertions are 932 / 10 / 895 / 27.

```json
{
  "before": {
    "kind": "RET002",
    "file": "crates/rum-runtime/src/runtime/suspension/core.rs",
    "start_line": "477",
    "start_col": "23",
    "end_line": "477",
    "end_col": "66",
    "target": "",
    "reason": "method syntax: method trait scope contains an unexpanded item",
    "witness": "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::take_accounted_pending_suspension_for_abandon"
  },
  "after": {
    "kind": "RET002",
    "file": "crates/rum-runtime/src/runtime/suspension/core.rs",
    "start_line": "482",
    "start_col": "23",
    "end_line": "482",
    "end_col": "66",
    "target": "",
    "reason": "method syntax: method trait scope contains an unexpanded item",
    "witness": "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::take_accounted_pending_suspension_for_abandon"
  },
  "same_source_line": "        let pending = self.suspension_access().take_for_abandon();",
  "classification": "same take_for_abandon receiver shifted within core.rs; 931 other rows byte-identical; no unresolved boundary removed"
}
```

The one typed-census receipt replay changes only the call ordinal of the same decoder. No acceptance code, population, stage or design input changes:

```json
{
  "before": [
    "Whole",
    "Whole0179",
    "R",
    "legacy-use",
    "rum_runtime::runtime::session::session_eval::<Session as inherent>::resume_inspect_rewrite_rules_with_input#call2:from_continuation_expr:",
    "crates/rum-runtime/src/runtime/session/session_eval.rs",
    "from_continuation_expr",
    "production",
    "D130"
  ],
  "after": [
    "Whole",
    "Whole0179",
    "R",
    "legacy-use",
    "rum_runtime::runtime::session::session_eval::<Session as inherent>::resume_inspect_rewrite_rules_with_input#call3:from_continuation_expr:",
    "crates/rum-runtime/src/runtime/session/session_eval.rs",
    "from_continuation_expr",
    "production",
    "D130"
  ],
  "discovered_use": {
    "identity": "rum_runtime::runtime::session::session_eval::<Session as inherent>::resume_inspect_rewrite_rules_with_input#call3:from_continuation_expr:",
    "caller": "rum_runtime::runtime::session::session_eval::<Session as inherent>::resume_inspect_rewrite_rules_with_input",
    "kind": "call",
    "name": "from_continuation_expr",
    "target": "Option :: < Vec < RewriteRuleInfo > > :: from_continuation_expr",
    "population": "production",
    "file": "crates/rum-runtime/src/runtime/session/session_eval.rs",
    "line": "194",
    "column": "12",
    "end_line": "194",
    "end_column": "66",
    "sha256": "438e7b2362e991f3d08c4c831f113fbd5b7e3247dc8b07ed0360f0651c423388"
  },
  "resolved_call": {
    "id": "C220adb377d819e87",
    "caller": "rum_runtime::runtime::session::session_eval::<Session as inherent>::resume_inspect_rewrite_rules_with_input",
    "kind": "call",
    "name": "from_continuation_expr",
    "target": "Option :: < Vec < RewriteRuleInfo > > :: from_continuation_expr",
    "receiver": "",
    "resolution": "resolved lexical declaration",
    "resolved_definitions": "rum_runtime::runtime::suspension::rewrite_rules::<Option < Vec < RewriteRuleInfo > > as ContinuationValue>::from_continuation_expr",
    "file": "crates/rum-runtime/src/runtime/session/session_eval.rs",
    "line": "194",
    "column": "12",
    "end_line": "194",
    "end_column": "66",
    "sha256": "438e7b2362e991f3d08c4c831f113fbd5b7e3247dc8b07ed0360f0651c423388"
  },
  "classification": "same decoder at session_eval.rs:194; Err constructor on the cold fault branch shifts call ordinal 2 to 3; stage R, legacy-use, owner, population, decoder D130 and every other receipt unchanged"
}
```

All 68 contract rows retain their first 18 fields; only the fingerprint changes. The final source context is fnv1a64:6b81d5a094de6541. The TSV-only decoder-anchor replay leaves all 68 fingerprints exact.

```tsv
id	before	after
B01	fnv1a64:25075e02e203401c	fnv1a64:1c3b721878987f04
B02	fnv1a64:264ce647d04d9a95	fnv1a64:f2a3f3b047002161
B03	fnv1a64:b062064787c2940a	fnv1a64:99b57974704ae1de
B04	fnv1a64:d28d79a1162c7f04	fnv1a64:4d433f2fd68dc13c
B05	fnv1a64:1220b4be927f2529	fnv1a64:8d4b7183d50d6aad
B06	fnv1a64:fe6a3e808a347348	fnv1a64:a7372ea199a86ee8
B07	fnv1a64:20508254576412f9	fnv1a64:73aa73578e14967d
B08	fnv1a64:3f190f5d14f1ef0e	fnv1a64:556937292c8a2a7a
B09	fnv1a64:deecd86e42d75e50	fnv1a64:74d4c27e198e5220
B10	fnv1a64:202133c752af0603	fnv1a64:6dbf97d1152e376b
B11	fnv1a64:93e52afe3020a504	fnv1a64:0e9af08cf081e73c
B12	fnv1a64:9df9586a1edff350	fnv1a64:33e14279f596e720
I01	fnv1a64:02e949004ebb369c	fnv1a64:fa1d5d15e5507584
I02	fnv1a64:696232528ab5ad29	fnv1a64:e48cef17cd43f2ad
I03	fnv1a64:866a4bdba95d4210	fnv1a64:3374e948a81374e0
I04	fnv1a64:a07f23d91522f4b5	fnv1a64:35a0a22dc0253901
P01	fnv1a64:6659a2daab01fde8	fnv1a64:612c956bc5430088
P02	fnv1a64:41d7c113df718d33	fnv1a64:2925bc324eca969b
P03	fnv1a64:4334b9873a46e022	fnv1a64:037b5c8c7cad54c6
P04	fnv1a64:1b28045d54f6b695	fnv1a64:e77f11c5cba93d61
P05	fnv1a64:4076bb3f0258c3cc	fnv1a64:8239767ec9580ef4
P06	fnv1a64:ee787d3aab0f8cd0	fnv1a64:8460674a81c680a0
P070	fnv1a64:28c6197c898506d0	fnv1a64:beae038c603bfaa0
P080	fnv1a64:f986e7bc2a7a61c3	fnv1a64:6e46f5021fa8072b
P090	fnv1a64:84a8c70e5f5b6855	fnv1a64:d20b7dae239f2e21
P100	fnv1a64:668799e8c82e05d4	fnv1a64:f1424228c7c568cc
P110	fnv1a64:93bb6e94d2e9eedb	fnv1a64:d8ccdca01b59bc53
P120	fnv1a64:e7222743f9cacd2e	fnv1a64:8203731bbd1bec1a
P130	fnv1a64:c41ddbb18617ecf9	fnv1a64:1777ccb4bcc8707d
P140	fnv1a64:7ba176b87048e84d	fnv1a64:8a2a2897c38ccf49
P150	fnv1a64:55e8b253aac85d75	fnv1a64:87ffdcb4d8906ac1
P071	fnv1a64:9529bd1b53f9c299	fnv1a64:cecebd7646779b1d
P081	fnv1a64:442e9804851de3f0	fnv1a64:535fbdce5ad308c0
P091	fnv1a64:f9bbd15f323002a8	fnv1a64:f06e70336e3c1d48
P101	fnv1a64:8bbe5f62084d8bdb	fnv1a64:d0cfcd6d50bd5953
P111	fnv1a64:5633f81136e339a0	fnv1a64:22d77b10be0749f0
P121	fnv1a64:79a47eea0ad413ff	fnv1a64:52b9d732ea82e2af
P131	fnv1a64:70379c074c80a084	fnv1a64:eaed61960ce1e2bc
P141	fnv1a64:bec6eacc90427008	fnv1a64:70ed259f8ea132a8
P151	fnv1a64:788c285809e12e6e	fnv1a64:792b2b152d50cd5a
B13	fnv1a64:8c7ad209bad12f48	fnv1a64:3547c22aca452ae8
R01	fnv1a64:fae47e3ea7883f25	fnv1a64:5637a52d5fc869b1
R02	fnv1a64:53b93ec9a59a38e2	fnv1a64:2c93edf91308ea86
R03	fnv1a64:7def3727c9255181	fnv1a64:372549388a547bb5
R040	fnv1a64:8b6d82ef536bdfc5	fnv1a64:0d2cd69aac435851
R050	fnv1a64:1dbb08a9f0ca482c	fnv1a64:a8cd88b982e085d4
R060	fnv1a64:8ed2e224ad5b8ddd	fnv1a64:f149c14ca56e6499
R070	fnv1a64:ac9c601e66bfa2f0	fnv1a64:bbcd85e83c74c7c0
R080	fnv1a64:67838ccbe4a7bdb5	fnv1a64:fca50b208faa0201
R090	fnv1a64:ae034d0d3f249794	fnv1a64:74b6cb3cc6e1048c
R100	fnv1a64:7587aa9a31983e8a	fnv1a64:5edb1dc71a208c5e
R110	fnv1a64:60bd22fd719ba2eb	fnv1a64:1293a83d6b0e6da3
R041	fnv1a64:72b43533d1f8ee48	fnv1a64:1b812554e16ce9e8
R051	fnv1a64:12caf523ec8974d5	fnv1a64:602dabc3b0cd3aa1
R061	fnv1a64:98242efd8858013e	fnv1a64:4069d71c34d898aa
R071	fnv1a64:42b37cb1d745ca47	fnv1a64:4b51c33c2dcb6ea7
R081	fnv1a64:2272033d63d9dad4	fnv1a64:ad2cab7d63713dcc
R091	fnv1a64:83e70c92ef06688d	fnv1a64:00b6216aed52b789
R101	fnv1a64:6489d2d7a73acc9d	fnv1a64:6730a137333a9f59
R111	fnv1a64:1ba105aed2676236	fnv1a64:2546ae3bbf0b3d32
R12	fnv1a64:78116991a7a07676	fnv1a64:73340a5c2f7e7672
R13	fnv1a64:14eacce847e91dfa	fnv1a64:36d941926cbb664e
R14	fnv1a64:80dfa85ea4dec5d4	fnv1a64:0b9a509ea47628cc
R15	fnv1a64:cd2d7fc5cd2f21b8	fnv1a64:49427c7d0584d158
O010	fnv1a64:b0a0d89d7a9da89e	fnv1a64:5b6e9825b424bb0a
O0200	fnv1a64:7669188cbf000e4d	fnv1a64:84f1ca6c1243f549
O011	fnv1a64:d15b9eacbaed525d	fnv1a64:33d27dd4b3002919
O0210	fnv1a64:3445c05628bb15d6	fnv1a64:fcbac932352016d2
```

Both certificate discoveries earned three production-instance marks and four LeaseProbe marks with zero rejections. Exactly the four emitted C01/C02 feature cells were replayed; all seven other columns remain unchanged. The complete old/new cells are in `.temp/brief-608/redispatch-5/certificate-diff.json`, with emitted receipts in promoted-hash-true.tsv and promoted-hash-false.tsv. Cell identities and content hashes are:

| Body | Hash feature | Old cell SHA-256 | Emitted/replayed cell SHA-256 |
| --- | --- | --- | --- |
| C01 | true | 7707cdf3e5b262eabeefd4038960021f09969ec727f256ec23e0a91490c59138 | e7050d5502ee53235d09419d0e715c8a4218f2c21ff50e758c7466044e91bd5a |
| C02 | true | 150668c0ac16393f31a45733688157af5d5cdb870c68e959f0473d802e52478d | 3c6c4e6e9f5d89c5d44a87d300ab163a2331d159c2b270381ac2ca4410fdd8ce |
| C01 | false | dc39116649406ddb08fc04c516ab996edeb577b69081c3bfae50b8f8c0f8a234 | 63ffd0da8e9b6677531539646b344945219331a1ca4dbe9e28751f5946d898f6 |
| C02 | false | 526eb412a518408ecb599ba50d4e8db7491ef417e0ef833938ef700040b3a198 | 27e513c5590d655b6e834f54d44dc1105e36513cbc138fbe542d887868e4e03f |

### 7. Stop-loss, deviations and remaining work

Discarded experiments: legacy pop/drop **0**, boundary outcome copy **0**, eager fault words/clear **0**. Pop attempt a1 reduced its target from ten to two extra instructions/frame; a2 reduced two to zero. The compiling boundary attempt reduced both of its targets to zero. Each retained step made measured progress; the stop-loss limit was never reached. The initial two E0599 compilation errors were repaired by keeping the helper's existing Result signature, without editing fault tests. No further tuning was attempted after the targets reached zero.

The initial stale-certificate failures and shifted decoder-anchor failure are disclosed with their actual output and successful reruns. Replays used newly emitted discovery only. Historical retention assertions, typed fault tests, payload dispositions and compile-control contracts were preserved. All required correctness commands and final measurements completed; no Cargo or measurement process is left running.

The remaining qualification is the final timing panel's host-idle condition: CPU activity was monitored, but GPU activity could not be verified from the sandbox and no director host-state answer was received. The positive timing signals are reported as indications requiring director assessment, not explained away or presented as confirmed regressions. No extra timing sample was taken to obtain a preferable result. Source implementation and instruction/allocation gates need no further repair identified by this dispatch; director review of the evidence and timing qualification is next.

No git command, network access, added unsafe, new dependency/version/lockfile/toolchain change, production frame conversion, transport flip, transit record, dual-arm decoder or unapproved payload disposition. Existing S2 feature edits remain as disclosed. One initial AGENTS filename search inadvertently listed sibling directories under /home/remi/dev; no sibling contents were read or written, and subsequent work was scoped to this repository. The tree is left uncommitted.

