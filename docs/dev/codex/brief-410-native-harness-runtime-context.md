# brief-410 — implement: repair the native harness's runtime context, and make its class catchable without loopback

Shape: **implementation**. A regression repair on brief-409, plus the coverage
that would have caught it in your sandbox.

## What happened

brief-409 replaced the native tests' ad-hoc waits with a shared
`NativeTestHarness::block_on_result`. Your gate was green. On my host, where
loopback binding is permitted and the native tests actually execute, **all nine
native tests fail**, and the workspace gate went from 2285 passed / 0 failed to
2276 passed / 9 failed.

The failure, verbatim:

```text
thread 'zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket'
panicked at crates/rum-jupyter/src/zeromq_native_test_support.rs:36:32:
there is no reactor running, must be called from the context of a Tokio 1.x runtime
```

I have already diagnosed it, so you do not need to re-derive it.
`zeromq_native_test_support.rs:36` is:

```rust
match runtime.block_on(tokio::time::timeout(self.remaining(stage), future)) {
```

`tokio::time::timeout` constructs a `Sleep` eagerly, which requires an entered
runtime context. Here it is constructed as an **argument** to `block_on`, so it
is evaluated before the runtime is entered. Every other `block_on` in these
files wraps its body in `async { … }`, including the code brief-409 replaced —
for example the original `signed.rs:203` was
`block_on(async { tokio::time::timeout(…, shell.recv()).await })`. The
refactor into a helper moved the construction out of the context.

Every native test routes through `block_on_result`, which is why all nine fail
rather than some.

## Why it escaped your gate, and what to do about it

**Not because of loopback.** This panic has nothing to do with sockets; it
would fire anywhere. It escaped because the native tests early-return on the
skip path before ever reaching the helper, so a sandbox that cannot bind never
executes a single line of it. The skip is a mask.

That is the durable problem worth fixing, and it is the larger half of this
brief: **the harness must have coverage that runs where sockets do not.** Once
it does, this class is catchable by you, on your own gate, without my host.

## Goal

`block_on_result` and its stage/timeout paths work in a real runtime context;
the class is swept rather than the instance; and a socket-free test exercises
the harness so the next defect of this shape fails in any sandbox.

## Do the failing test first

Write the socket-free harness test **before** the repair and show it
reproducing the panic. That is the whole point: if the new test cannot fail
against the current code, it would not have caught this bug and it will not
catch the next one.

## Context pointers

- Read `AGENTS.md`, brief-406's report §1 (the skip mechanism), and
  brief-409's committed report.
- The defect: `zeromq_native_test_support.rs:26-41`, specifically `:36`.
- The correct pattern, still present throughout:
  `zeromq/tests/native/frontend/session_flow.rs:22,86,110`, and every other
  `runtime.block_on(async { … })` in these files.
- Users of the helper, all of which fail today: `native/basic.rs:47,71`,
  `native/signed.rs:71,98,119,202`,
  `native/frontend/control.rs:47,63,162,169,223`, and the helper's own
  internal calls at `zeromq_native_test_support.rs:50,61,93,111,148,162`.
- The skip gate from brief-408 (`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`) must keep
  working exactly as it does.
- **Your sandbox cannot bind loopback**, so you still cannot run the native
  tests. The new harness test is precisely the part you *can* run — treat it
  as your real gate.

## Scope

- Files you MAY modify, all under `crates/rum-jupyter/src/`:
  - `zeromq_native_test_support.rs`;
  - `zeromq/tests/native/**` only where a call site must change to match a
    corrected helper signature;
  - a new socket-free test module for the harness, placed wherever the crate's
    existing test layout puts such a thing — say where and why.
- Files/areas you MUST NOT touch:
  - production code of any kind, including `zeromq.rs`, `zeromq_native/**`,
    `adapter/**`, `transport/**`;
  - `crates/rum-jupyter/src/tests/**` (family A);
  - the IOPub timing assumptions in `frontend/{execute,stdin,session_flow}.rs`
    (family B, still to come);
  - the wire protocol, `Cargo.lock`, dependency edges, other crates.
- If you need a file outside this list, **STOP and report**.

## The accept-vs-stop criterion

Repairing the runtime context and covering the harness is in scope. **Reverting
brief-409's structural work is a stop** — the stage-labelled waits, the single
30-second harness stop-loss, and the two structural negative assertions all
stay. So is weakening any assertion to make a test pass. If the repair seems to
require undoing 409, stop and report.

## Deliverable

1. `block_on_result` constructs its timeout **inside** the runtime context.
2. **Sweep the class, not the instance.** Audit every `tokio` construction in
   the harness and the native tests for whether it is evaluated inside an
   entered runtime, and report the audit as a table — not just the one line you
   changed. brief-388's lesson on this project was closing the sightings
   instead of the class; do not repeat it.
3. A socket-free test that exercises `block_on_result` on all three outcomes —
   success, inner error, and harness timeout — and that **fails against the
   current code**. Show it red, then green.
4. The harness timeout message still names the stage and still says it is a
   harness timeout rather than a latency contract.
5. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` unchanged; brief-408's resolved-endpoint
   propagation unchanged; brief-409's negative assertions unchanged.

## rum / Rust pitfalls (read twice)

1. `tokio::time::timeout`, `sleep`, and `Instant` registration all require an
   entered runtime; being *passed to* `block_on` is not being inside it.
2. A test that constructs its own runtime must enter it the same way the real
   call sites do, or it proves nothing about them.
3. Do not make the harness test depend on timing to pass; for the timeout arm,
   drive a future that never resolves and use a short explicit bound supplied
   by the test, not the 30-second constant.
4. Keep the harness's public shape stable if you can; if a signature must
   change, migrate every call site and list them.
5. No new `unsafe`. No production change. No dependency change.

## Verification (MANDATORY — you run what you can, honestly)

Pass `--offline` to every `cargo` command. You will need
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` for the native tests to skip; that is correct
for your sandbox, and it means the native paths remain unexecuted by you.

**Red first**, the new harness test against unrepaired code, verbatim. Then:

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 600  -- cargo test --offline -p rum-jupyter
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

Paste the new test green, and the class-audit table.

**State plainly that you executed no native socket path.** I will run the
native suite on the host where it binds. My baselines: before brief-409, 0
failures in 25 native-only samples and 0 in 20 whole-suite samples, workspace
2285 passed / 0 failed. Anything short of that is a stop.

## Hard rules

- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Leave changes in the working tree.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. No probe artifacts in the repo.
- **Never report a rate for a test you did not observe running.**
- **Testing Policy is a hard line**: rum's own documented behavior only.
- **No new `unsafe`.** No test deleted, no assertion weakened, no part of
  brief-409 reverted.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. The repair, with line ranges.
2. The class-audit table: every `tokio` construction, where it is evaluated,
   and whether that is inside an entered runtime.
3. The new socket-free test: red verbatim, then green.
4. Verification output, with an explicit statement of what you could not run.
5. Confirmation that 408's and 409's work is intact.
6. Stop-loss, risks, open questions.

---

# Report — 2026-09-02

## 1. Repair

- [`block_on_result`](crates/rum-jupyter/src/zeromq_native_test_support.rs:26), lines 26–43, now constructs and awaits `tokio::time::timeout` inside the async body entered by `Runtime::block_on`.
- The socket-free [`harness_tests` module](crates/rum-jupyter/src/zeromq_native_test_support.rs:346), lines 346–407, covers success, inner error, and harness timeout.
- The test uses `test_runtime()` exactly like native callers and supplies a test-only 100 ms stop-loss around a permanently pending future.
- No call sites or helper signatures changed.

The test is inline because `zeromq_native_test_support.rs` is already included as `zeromq::tests::support`. This keeps the test independent of loopback and within the explicitly allowed file.

## 2. Tokio construction audit

Every explicit Tokio construction in the harness and native tests is accounted for below.

| Site | Construction and evaluation point | Entered runtime? |
|---|---|---|
| [`support:265`](crates/rum-jupyter/src/zeromq_native_test_support.rs:265) | `tokio::runtime::Builder::new_current_thread()` constructs the runtime before one can be entered. `enable_all()` installs its drivers. | Not applicable; safe runtime construction. |
| [`support:37`](crates/rum-jupyter/src/zeromq_native_test_support.rs:37) | Harness `tokio::time::timeout`; now evaluated when the `async` body is polled by `Runtime::block_on`. | Yes. Repaired site. |
| [`execute.rs:87,125`](crates/rum-jupyter/src/zeromq/tests/native/frontend/execute.rs:87) | Two existing `tokio::time::timeout` calls, each lexically inside `runtime.block_on(async { … })`. | Yes. |
| [`stdin.rs:104,158,183`](crates/rum-jupyter/src/zeromq/tests/native/frontend/stdin.rs:104) | Three existing timeout calls inside their surrounding async blocks. | Yes. |
| [`session_flow.rs:111,136,206,260,285,353,378,439`](crates/rum-jupyter/src/zeromq/tests/native/frontend/session_flow.rs:111) | Eight existing timeout calls inside their surrounding async blocks. | Yes. |
| Direct `block_on` operations: [`signed.rs:34`](crates/rum-jupyter/src/zeromq/tests/native/signed.rs:34), [`execute.rs:18,26,62,86,124`](crates/rum-jupyter/src/zeromq/tests/native/frontend/execute.rs:18), [`stdin.rs:21,31,41,79,103,133,157,182`](crates/rum-jupyter/src/zeromq/tests/native/frontend/stdin.rs:21), [`session_flow.rs:22,32,42,50,86,110,135,181,205,235,259,284,328,352,377,421,438`](crates/rum-jupyter/src/zeromq/tests/native/frontend/session_flow.rs:22) | Every direct `Runtime::block_on` receives an `async { … }`; socket creation/connect/send/receive bodies execute after entry. | Yes. |
| Helper-fed futures: [`support:52,63,95,113,150,164`](crates/rum-jupyter/src/zeromq_native_test_support.rs:52), `basic:47,71`, `signed:71,98,119,202`, `control:47,63,162,169,223`, regression test `376,382,394` | The async future object is supplied before `block_on`, but the `zeromq` async method body is lazy and first polled inside the repaired timeout wrapper. No timer is eagerly created at these call sites. | Future body: yes. |
| [`support:22,191,357`](crates/rum-jupyter/src/zeromq_native_test_support.rs:22) and [`support:197`](crates/rum-jupyter/src/zeromq_native_test_support.rs:197) | These are `std::time::Instant` and `std::thread::park_timeout`, not Tokio registrations. | No runtime required. |

There are no `tokio::time::sleep`, `tokio::time::Instant`, or `tokio::spawn` sites in the audited files. The family-B two-second timeouts remain untouched as required; this audit only establishes that their runtime context is valid.

## 3. Socket-free test: red, then green

Red against the unrepaired helper:

```text
   Compiling rum-jupyter v0.2.0 (crates/rum-jupyter)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.64s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-0d2044aa0c254386)

running 1 test

thread 'zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:36:32:
there is no reactor running, must be called from the context of a Tokio 1.x runtime
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets ... FAILED

failures:

failures:
    zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 104 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-jupyter --lib`
```

Green after the repair:

```text
   Compiling rum-jupyter v0.2.0 (crates/rum-jupyter)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.47s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-0d2044aa0c254386)

running 1 test

thread 'zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:40:31:
native ZeroMQ stage `inner error outcome` failed: "inner failure"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:201:9:
native ZeroMQ harness timeout while waiting for stage `timeout outcome`; this is a harness timeout, not a latency contract
test zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 104 filtered out; finished in 0.10s
```

The two printed panics are intentionally caught and their exact messages asserted.

## 4. Verification

All Cargo invocations used offline mode.

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
```

Exit 0, no output.

```text
RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-jupyter
```

Relevant verbatim result:

```text
test zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets ... ok

test result: ok. 105 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

```text
RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

Exit 0. The run emitted three existing `rum-runtime` dead-code warnings and finished green; its final output was:

```text
   Doc-tests rum_terminal

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Explicit native skip check:

```text
running 9 tests
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
...
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out; finished in 0.00s
```

Those libtest “ok” entries were all early-return skips. I executed no native socket path and make no native pass-rate or stability claim.

`git diff --check` also returned exit 0.

## 5. Brief-408 and brief-409 remain intact

- The skip gate and diagnostics remain unchanged at [`support:11,208–227`](crates/rum-jupyter/src/zeromq_native_test_support.rs:208).
- Port-zero configuration remains at [`support:237–250`](crates/rum-jupyter/src/zeromq_native_test_support.rs:237), and native callers still use backend-resolved `transport_layout()` endpoints.
- The single production 30-second stop-loss remains at [`support:12`](crates/rum-jupyter/src/zeromq_native_test_support.rs:12).
- Its message still names the stage and says “harness timeout, not a latency contract” at [`support:200–204`](crates/rum-jupyter/src/zeromq_native_test_support.rs:200).
- Tampered-signature absence remains structural at [`signed.rs:173–237`](crates/rum-jupyter/src/zeromq/tests/native/signed.rs:173).
- Post-shutdown Shell absence remains structural at [`control.rs:194–223`](crates/rum-jupyter/src/zeromq/tests/native/frontend/control.rs:194).
- No assertion or test was removed or weakened. No production, dependency, `Cargo.lock`, wire-format, family-A, or family-B file was changed. No `unsafe` was added.

## 6. Stop-loss, risks, open questions

The remaining verification is the authoritative native suite on a loopback-capable host. The new test closes the socket-free coverage gap for this runtime-context class, but this sandbox still cannot validate actual ZeroMQ behavior.

No stop condition was encountered and there are no open implementation questions. No commit, branch, stash, checkout, push, or git configuration operation was performed.
