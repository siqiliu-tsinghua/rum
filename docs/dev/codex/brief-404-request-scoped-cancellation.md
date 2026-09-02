# brief-404 — implement: request-scoped, generation-qualified cancellation

Shape: **implementation**. This supersedes brief-400, whose repair mechanism
brief-401 refuted and whose tree has since moved under it.

## Where this came from

brief-397 found a real host-contract defect. A cancellation is erased by the
evaluation prologue: `begin_top_level_interrupt_budget` clears the flag as its
first statement (`runtime/session/session_helpers.rs:32`), and `eval_request`
calls it immediately (`runtime/session/session_requests.rs:130`). The
`rum-jupyter` adapter takes `session.cancel_handle()` before spawning
(`crates/rum-jupyter/src/adapter/worker.rs:88,99`), so the handle is obtained
at the right moment, the cancel is set at the right moment, and the evaluation
then throws it away. This reproduces in both directions.

brief-398 designed the repair: request-scoped, generation-qualified
cancellation rather than a session-global flag. The user ruled for it (option
乙, host-level request scope, not an adapter latch).

brief-400 began it and stopped correctly on a MAY-list omission of mine. Its
report then surfaced a separate defect, which became briefs 401-403 and is now
landed: the abandonment protocol (`fc959898`) and its caller migration plus the
overlapping-request rejection (`1afdc926`).

Two of brief-401's corrections govern this brief:

- **Do not drive the saved frames with `Err(EvalError::Cancelled)`.** That was
  brief-400's proposed repair and it is unsound: ordinary frames consume errors
  and run work. Use `Session::abandon_pending_suspension`
  (`runtime/suspension/core.rs:484`), which now exists.
- **The accepted-cancellation branch is where the abandonment entrypoint must
  be called** — today's cancellation does not raw-drop the queue.

## Reference material: brief-400's stashed work

`stash@{0}` holds brief-400's WIP. Read it, do not apply it:

```
git stash show -p stash@{0}                                   # the 9-file patch
git show stash@{0}^3:crates/rum-runtime/src/runtime/model/request_cancellation.rs
```

**Read-only. Do not `pop`, `apply`, `drop`, or otherwise mutate the stash or
any git state.** I measured what still fits: six of its nine files apply
cleanly; three conflict — `model/session.rs`, `session_requests.rs`, and
`session_state.rs` — and they conflict exactly at the integration points that
briefs 402 and 403 rewrote. So the state machine module and the 374 lines of
tests in `tests/session_requests/api.rs` are close to reusable; the wiring is
not. Treat the stash as a design reference and a test source, and write the
integration against the current tree.

## The design to implement

From brief-398, as embodied in the stashed `request_cancellation.rs`:

- `RequestInterruptState { phase: AtomicU64 }` encoding either idle or
  `(generation << 1) | cancelled_bit`.
- Accepting a request yields a **non-`Clone`** `AcceptedRequest` ticket
  carrying its generation.
- `RequestCancelHandle` is generation-qualified and cancels by
  compare-exchange, so a handle issued for generation N cannot affect
  generation N+1.
- Request completion is the linearization point.

The property this buys, and the one the tests must prove: **a cancellation
that arrives for the current request is never erased by that request's own
prologue, and a cancellation issued for a finished request never reaches the
next one.**

## Goal

Cancellation becomes request-scoped and generation-qualified; the prologue no
longer erases a live cancel; a cancel accepted while a single-source input
continuation is pending abandons that continuation structurally before any
host input is converted or delivered, returns the documented structured
termination, and leaves the session reusable.

## Do the failing test first

Write the regression that a cancel arriving before `eval_request` survives the
prologue, **run it red, and paste the failure** before any repair. Synchronize
with a channel handshake, never a sleep — see the Pitfalls.

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md`,
  `docs/guide/embedding.md`, brief-397's and brief-398's reports, brief-401's
  report §4 and §6, and brief-403's committed report.
- Defect: `runtime/session/session_helpers.rs:31-39`,
  `runtime/session/session_requests.rs:125-138`.
- Abandonment entrypoint: `runtime/suspension/core.rs:484`, with
  `AbandonReason` in the same file.
- Overlap rejection landed by brief-403:
  `Session::reject_overlapping_evaluation_request`
  (`runtime/session/session_requests.rs:46`), called as `eval_request`'s first
  statement. Cancellation acceptance and overlap rejection are both
  request-boundary preconditions; **their order is a decision you must make
  deliberately and state in the report.**
- Host surface: `crates/rum-host/src/session.rs:118-125,457-460`.
- Stale tracker entry to correct: `TODO.md:701-707` still describes this
  defect as open and cites `session_helpers.rs:30` and four sleeping tests.
  Only `crates/rum-host/tests/stress.rs:124-144` still sleeps.
- **Your sandbox has no network.** Pass `--offline` to every `cargo` command.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model/request_cancellation.rs` (new),
    `runtime/model/mod.rs`, `runtime/model/session.rs`;
  - `runtime/session/session_requests.rs`, `session_helpers.rs`,
    `session_state.rs`, `session_api.rs`, `session_eval.rs`;
  - `runtime/eval_core/mod.rs`;
  - **`runtime/suspension/core.rs`** — for the accepted-cancel call into the
    abandonment entrypoint. *This is the file my brief-400 MAY list omitted,
    which is why that brief stopped. It is in scope now.*
  - `runtime/kernel_api.rs` if the host needs a new export;
  - `crates/rum-host/src/session.rs` and `crates/rum-host/tests/`,
    `crates/rum-host/src/tests/`;
  - `crates/rum-runtime/src/runtime/tests/` (in particular
    `tests/session_requests/api.rs`);
  - `TODO.md`, and `docs/guide/embedding.md` if the cancellation contract's
    wording changes.
- Files/areas you MUST NOT touch:
  - **`HostSession::cancel_handle` must keep working.** Re-express it on the
    new mechanism; do not remove it or migrate its consumers — that is
    brief-405, deliberately separate so the public API change lands after the
    mechanism is proven.
  - `crates/rum-jupyter` — its two pump failures are briefs 405 and 406.
  - `rum-core` representation, `Cargo.lock`, dependency edges, the generated
    builtin reference;
  - any `FrameAbandon` classification, the abandonment protocol's shape, or
    brief-403's rejection contract. If one of them is wrong, **STOP and report
    it** rather than adjusting it here — that would be a gate miss of mine and
    I want to hear about it.
  - ordinary evaluator, dispatch, or matcher logic.
- If you need a file outside this list, **STOP and report**. My MAY lists have
  been short seven times on this project, including the one that stopped
  brief-400.

## The accept-vs-stop criterion

Making cancellation request-scoped is in scope. **Changing what a cancelled
evaluation reports to the host is a stop.** The documented structured
termination stays as it is; cooperative late cancellation stays cooperative.
If the repair appears to require changing the reported outcome, that is a
finding, not a licence.

## Deliverable

1. The generation-qualified state machine, replacing the raw `interrupt_flag`
   clear. `begin_top_level_interrupt_budget` must no longer erase a cancel
   belonging to the request it is beginning.
2. A non-`Clone` `AcceptedRequest`; a `RequestCancelHandle` that cannot affect
   a later generation; completion as the linearization point. `reset()` must
   retire the generation so a handle held across it is inert.
3. **Cancellation while suspended.** A cancel accepted while a single-source
   continuation is pending calls `abandon_pending_suspension` with the
   cancellation reason **before** `input.into()`, before any `Expr` is built
   from host input, and before any queued value is consumed. Prove conversion
   did not happen with a custom `Into<String>` counter, and prove statements
   after the `InputString` did not run with a rum-side sentinel.
4. **State the resulting escape route.** brief-403 documented that a
   script-shaped suspension leaves `reset()` as the only way forward. If a
   cancel can now abandon a pending continuation and leave the session usable,
   that is a second, explicitly named escape — say so in the report and in
   `docs/guide/embedding.md`. Do not invent a new public API for it; use the
   cancellation that already exists.
5. Separate public `Block`, `Module`, and `With` cases plus attributes,
   iterator, option, `Reap`, `Monitor`, trace, constraint, PRNG, and
   `AbortProtect` state proven clean after a cancel-while-suspended, reusing
   brief-403's matrix shapes.
6. Nested `WithCleanup` follows the policy brief-402 landed: cleanup runs,
   nested input is abandoned without a second host request.
7. `TODO.md:701-707` corrected: the defect is closed, and only
   `stress.rs:124-144` still sleeps.

## rum / Rust pitfalls (read twice)

1. Never call the ordinary resume driver with a synthetic `Cancelled`.
2. Do not clone or convert the resume input before checking the accepted
   cancel. Conversion is observable; the counter test will catch it.
3. Do not let a frame transform cancellation into success or run later
   continuation, fallback, or handler work.
4. A stale handle must not leak a cancel into the next request or across
   `reset()`. That is the whole point of the generation.
5. **No sleep-synchronized tests.** Use a channel handshake so the canceller
   and the evaluation rendezvous deterministically. The tracker entry you are
   closing exists because sleeping tests of this exact shape are a latent
   flake, and this session has watched the `rum-jupyter` suite produce three
   different timing failures.
6. Do not change the amortized interrupt-polling cadence in this brief.
7. No new `unsafe`.
8. Everything here is request-boundary and cold-path work. No ordinary
   evaluator-step branch, allocation, or lock.

## Verification (MANDATORY — you run it; iterate to green)

Pass `--offline` to every `cargo` command.

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

`--no-fail-fast` is required on the workspace run: `cargo test` stops after a
failing binary and silently under-reports the rest.

**Known-failing, not yours.** The `rum-jupyter` suite fails intermittently at
whole-suite level on the committed tree — I measured roughly three failures in
ten whole-suite samples, with the failing test moving between
`tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle`,
`tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately`,
and two `zeromq::tests::native::*` tests, each of which passes 20/30 out of 30
in isolation. Report whatever you see; do not fix it and do not touch that
crate. Any failure outside `rum-jupyter` is yours.

Note that the first of those, finding B, is the defect *this* brief repairs at
the kernel level. If it starts passing, say so — but do not chase it, and do
not modify `rum-jupyter` to make it pass.

No timing A/B: this is a correctness line on cold paths. If you change ordinary
eval, dispatch, or matcher code, the cost premise is exceeded — stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, checkout, or touch git config. Do NOT
  `stash pop`, `apply`, or `drop`. Reading `stash@{0}` with `git stash show -p`
  and `git show` is explicitly permitted and is the only git access granted.
- Leave changes in the working tree; I run the authoritative gate and commit.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. No probe artifacts in the repo.
- **Testing Policy is a hard line**: rum's own documented behavior only.
- **No new `unsafe`.**
- Minimal diff; no drive-by reformatting or renames.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. Per-file change with line ranges and a one-line why; what you reused from
   the stash and what you rewrote, and why.
2. The request-boundary ordering decision: where cancellation acceptance sits
   relative to brief-403's overlap rejection, and why that order is right.
3. The red test verbatim before the repair, then the same test green.
4. Cancel-while-suspended evidence: the conversion counter, the side-effect
   sentinel, and the scope-cleanliness matrix.
5. Verification output, including the repository invariant rows.
6. Cost statement.
7. Stop-loss, risks, open questions.

---

# Report — 2026-09-02

## 1. Per-file changes

- [request_cancellation.rs](crates/rum-runtime/src/runtime/model/request_cancellation.rs:1), lines 1–237 — added the `AtomicU64` generation state machine, non-`Clone` `AcceptedRequest`, cloneable exact-generation `RequestCancelHandle`, completion linearization, generation exhaustion handling, and the legacy iterator-interrupt mirror.
- [model/mod.rs](crates/rum-runtime/src/runtime/model/mod.rs:7), lines 7 and 34–35 — registered and exported the request-cancellation types.
- [model/session.rs](crates/rum-runtime/src/runtime/model/session.rs:4), lines 4–9 and 118–123 — stored current/pending request tickets, generation state, and the test-only raw interrupt probe.
- [session_state.rs](crates/rum-runtime/src/runtime/session/session_state.rs:662), lines 662–668 — initialized request cancellation state for new/reset sessions.
- [session_api.rs](crates/rum-runtime/src/runtime/session/session_api.rs:137), lines 137–195 — added request acceptance and qualified-handle reservation while preserving the low-level `interrupt_handle() -> Arc<AtomicBool>` API used by `rum-stdlib`.
- [session_helpers.rs](crates/rum-runtime/src/runtime/session/session_helpers.rs:30), lines 30–111 — made the prologue preserve the accepted generation, managed ticket activation/completion, and suppressed the qualified interrupt mirror during structural cleanup.
- [eval_core/mod.rs](crates/rum-runtime/src/runtime/eval_core/mod.rs:85), lines 85–103 — checked the active generation at the existing interrupt-poll cadence; the cadence and default `interrupt_armed` gate are unchanged.
- [session_eval.rs](crates/rum-runtime/src/runtime/session/session_eval.rs:225), lines 225–239 and 333–338 — checked cancellation before parsing/lowering and stopped script forms after cancellation.
- [session_requests.rs](crates/rum-runtime/src/runtime/session/session_requests.rs:46), lines 46–70, 154–213, 269–321, 366–435, and 493–639 — integrated accepted requests across source/expression/script/file entrypoints; cancellation now abandons suspension before resume conversion and preserves existing source/file outcome shapes.
- [rum-host session.rs](crates/rum-host/src/session.rs:109), lines 2–5, 109–123, 168–170, 412–420, and 457–466 — re-expressed `HostSession::cancel_handle` over the qualified handle and made typed calls check suspended cancellation before argument conversion.
- [runtime request tests](crates/rum-runtime/src/runtime/tests/session_requests/api.rs:20), lines 2–5, 20–57, and 475–679 — added deterministic channel-handshake, stale-generation, suspended-input, reset/rotation, and script/file cancellation coverage.
- [lifecycle_contract.rs](crates/rum-host/tests/lifecycle_contract.rs:108), lines 108–170, 223, and 227 — added the red/green prologue regression and public stale/reset generation tests.
- [suspension_abandonment_contract.rs](crates/rum-host/tests/suspension_abandonment_contract.rs:75), lines 75–199 and 421–610 — added observable conversion, continuation-sentinel, scope matrix, nested cleanup, queued-input, and reset-free script escape tests.
- [TODO.md](TODO.md:701), lines 701–706 — closed the defect and retained only the ignored stress-loop sleep as separate cleanup.
- [embedding.md](docs/guide/embedding.md:299), lines 299–302, 353–359, and 435–445 — documented request qualification, suspension behavior, stale/reset handles, and cancellation as the second script-suspension escape.

The stash supplied the phase encoding, linear-ticket design, stale-generation cases, and runtime test skeletons. I ported those pieces, but rewrote the wiring against the brief-402 abandonment protocol and brief-403 overlap rejection. The public scope tests reuse brief-403’s matrix shapes. No change was needed in `suspension/core.rs`; its existing `abandon_pending_suspension` entrypoint is called from the accepted-cancellation branches.

## 2. Request-boundary ordering decision

Accepted cancellation is checked before brief-403’s overlap rejection:

1. A pending ticket is activated.
2. If it belongs to the currently suspended request and is cancelled, the ticket is detached, `abandon_pending_suspension(AbandonReason::CancelledRequest)` runs, and the old request reports structured cancellation.
3. Otherwise brief-403’s overlap rejection runs unchanged.
4. For a fresh accepted request, the prologue runs without clearing its generation, followed by a pre-parse cancellation check.

This ordering is required because cancellation is now an explicitly documented second escape from script-shaped suspension. An uncancelled overlap still rejects without abandoning or consuming anything. Resume cancellation is checked before taking the checkpoint, before `input.into()`, and before queued input is consumed. Host typed calls perform the same preflight before building argument expressions.

## 3. Red test, then green

Command used both times:

```text
scripts/run-bounded.py --timeout 300 -- cargo test --offline -p rum-host --test lifecycle_contract cancel_accepted_before_eval_request_survives_request_prologue -- --exact --nocapture
```

Before repair:

```text
   Compiling rum-host v0.2.0 (crates/rum-host)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.53s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-d359d0673170e5fa)

running 1 test

thread 'cancel_accepted_before_eval_request_survives_request_prologue' panicked at crates/rum-host/tests/lifecycle_contract.rs:44:5:
assertion `left == right` failed
  left: "2"
 right: "TerminatedEvaluation[\"Cancelled\"]"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test cancel_accepted_before_eval_request_survives_request_prologue ... FAILED

failures:

failures:
    cancel_accepted_before_eval_request_survives_request_prologue

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-host --test lifecycle_contract`
```

After repair:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-d359d0673170e5fa)

running 1 test
test cancel_accepted_before_eval_request_survives_request_prologue ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s
```

## 4. Cancel-while-suspended evidence

The custom `Into<String>` probe increments an `AtomicUsize`; every cancellation helper asserts the count remains exactly `0`. The separate late-acquired-handle test also asserts zero conversions. Cancellation returns `HostTerminationReason::Cancelled`, and no ordinary resume driver receives synthetic cancellation.

Selected public-contract output:

```text
cancelLateSentinel404=Success(value=0, termination=None, messages=0, prints=[], trace=0)
InputString[]=Success(value="queued-value-survives-cancel", termination=None, messages=0, prints=[], trace=0)
cancelWith404=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
cancelBlock404=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
Names["Global`cancelModuleLocal404$*"]=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
Attributes[cancelAttribute404]=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
List[cancelCleanupInner404, cancelCleanupOuter404, cancelCleanupAfterInput404, cancelBodyAfterInput404]=Success(value=List[1, 1, 0, 0], termination=None, messages=0, prints=[], trace=0)
List[cancelScriptTail404, cancelEscapeAttempt404]=Success(value=List[0, 0], termination=None, messages=0, prints=[], trace=0)
6 * 7=Success(value=42, termination=None, messages=0, prints=[], trace=0)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

The matrix separately covers `Block`, `Module`, `With`, attributes, iterator state, option scope, `Reap`, `Monitor`, trace, time and memory constraints, PRNG/`BlockRandom`, and `AbortProtect`. Every post-`InputString` sentinel remained `0`. Nested `WithCleanup` ran both cleanups exactly once and abandoned nested input without issuing a second host request. A queued value survived cancellation and was consumed by the following independent `InputString`.

## 5. Verification output

Formatting emitted no output and exited 0:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
```

Runtime:

```text
test result: ok. 928 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Host:

```text
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.79s
```

The mandatory workspace command, including `--no-fail-fast`, exited 0. Its non-empty suite summaries were:

```text
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 103 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 928 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.02s
test result: ok. 828 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.52s
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.77s
```

The whole `rum-jupyter` suite passed on this run, including the kernel-level finding-B test.

Repository invariant rows:

```text
named_suspension_frames=545
named_pure_finalizers=60
pure_frame_abandon_derives=516
handwritten_stateful_frame_abandons=29
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:EvalEvaluationDataSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/monitor.rs:EvalMonitorSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/reap.rs:EvalReapSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:ResumeMemoryConstrainedWithLimitSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:ResumeTimeConstrainedWithLimitSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:EvalExprWithStreamCaptureSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:EvalStructuredTraceCaptureExprSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:EvalAbortProtectSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortAfterPrimarySuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortAfterPrimarySuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortWithPropagationSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:EvaluateLoopIteratorBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:WithIterationProductScopeSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:EvalCheckSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:EvalQuietSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:DriveWithCleanupPrimarySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:EvalWithCleanupSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/options/patterns.rs:EvaluateWithOptionValueScopeSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:EvalSingleSymbolBlockSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeBlockBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeBlockInitializersSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeSingleSymbolBlockBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeModuleInitializersSuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeModuleInitializersSuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeSuspendedModuleInitializersSuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeSuspendedModuleInitializersSuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs:ResumePackageInstallSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/session/session_api/resume_frames.rs:EvaluateWithRandomStateRestoredSuspensionFrame
evaluate_cleanup_allowlist=WithCleanup owner frames (2)
raw_live_owner_clears=0
raw_live_owner_replacements=0
legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 929 filtered out; finished in 0.44s
```

A scoped `unsafe` search produced no matches.

## 6. Cost statement

No timing A/B was run, per the brief. All new allocation and synchronization occurs at session creation, request acceptance/completion, reset, or generation exhaustion. There are no mutexes and no new ordinary evaluator-step allocations or locks.

The existing `interrupt_armed` fast gate and 1024-step polling cadence are unchanged. Only an already-armed poll checks the accepted generation before the retained low-level interrupt mirror. When no timeout, accepted request, or shared low-level handle exists, the existing default path remains unarmed.

## 7. Stop-loss, risks, open questions

No stop-loss condition fired. I made no out-of-scope edits, did not touch `rum-jupyter`, `rum-stdlib`, `Cargo.lock`, dependency edges, abandonment classifications, or ordinary dispatch/matcher logic, and added no `unsafe`.

The first wiring attempt exposed an out-of-scope compatibility dependency: `rum-stdlib/src/lazy.rs` requires `Session::interrupt_handle() -> Arc<AtomicBool>`. I therefore retained that low-level API and added the distinct qualified `request_cancel_handle()` used by the host. The phase word remains authoritative; the boolean is a synchronized compatibility mirror for kernel-resident iterators.

The main review risk is that [session_requests.rs](crates/rum-runtime/src/runtime/session/session_requests.rs:1) is now about 29.7k characters, within the documented split-plan range. Splitting it would have expanded this tightly scoped change, so I left module organization unchanged.

No open correctness questions remain. The ignored host stress-loop sleep and intermittent Jupyter timing risks remain explicitly out of scope.
