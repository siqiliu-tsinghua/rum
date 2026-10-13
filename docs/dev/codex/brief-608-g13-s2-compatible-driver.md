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
feature tree. Panel, each result checked before it is measured: one
suspend-and-resume workload per route (macro, chain, `Evaluate` successor,
finalizer, repeat helper) driven through `rum-cli` with queued input, plus
the ordinary labels `literal-integer`, `downvalue-pattern-hit`,
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
