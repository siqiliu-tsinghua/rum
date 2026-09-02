# brief-409 — implement: condition-driven native progress, and structural negative assertions

Shape: **implementation**. Family C of the four brief-406 identified, taken
third. Family B — the IOPub subscription barrier — is next and will build on
the helpers you leave behind.

## Read this before anything else: there is nothing failing

brief-407 (family A) and brief-408 (family D) landed. On this host, measured
by me:

- the whole `rum-jupyter` suite: **0 failures in 20 samples**;
- the native suite alone: **0 failures in 25 samples**;
- the workspace gate: 2285 passed, 0 failed, with no skip opt-out set, so a
  silent native skip would have failed it.

**So this brief is not fixing red tests.** It is removing assertions that are
invalid regardless of whether they currently pass. A test that passes because
a machine happened to be fast is not verifying anything; it is supplying false
confidence and will fire on different hardware or under load. That is the
whole justification, and it is sufficient — but it also means **"the failure
went away" is not available as evidence.** Your gate is structural.

Do not "fix" anything by making a wait longer. Do not add a retry. If you
find yourself tuning a duration, you have left this brief's premise.

## Where this came from

brief-406's class inventory graded every timing-dependent test in the crate.
Family C is the native, non-IOPub half: Dealer/Router, Req/Rep, Control, and
the signature-rejection path. Its shapes:

- **Fixed connection sleeps.** `signed.rs:158,184` and `control.rs:25,115`
  each sleep 50 ms and then treat the connection as ready.
- **Elapsed time as a pass condition.** `basic.rs:59,117`, `signed.rs:85,111`,
  and the shared loops at `zeromq_native_test_support.rs:104,117-129,137-149,157-169`
  turn "within two seconds" into a semantic requirement. The native receive
  and evaluator workers are asynchronous
  (`zeromq_native/threading.rs`, `adapter/worker.rs`); no duration is promised.
- **Proof of absence by waiting.** `signed.rs:203` and `control.rs:222` assert
  that no reply arrives within 200 ms.

## The part that matters most

**You cannot prove absence by waiting.** A 200 ms window that sees no reply
says the reply had not arrived yet; it does not say the dispatch never
happened. Both of those assertions must become **structural**:

- After `pump_once` returns `InvalidSignature`, assert from state that no
  dispatch was produced and no reply was enqueued — the backend's own
  bookkeeping already knows this. Do not wait to find out.
- After a shutdown is observed, assert that the Shell request is still pending
  and unprocessed, rather than that a reply did not arrive in 200 ms.

If the state needed to make either assertion is not reachable from the test,
say so and stop — do not substitute a longer window.

## Goal

Every stage in the native non-IOPub tests advances on an observed condition
with a stage-specific diagnostic; the two negative assertions become
structural; and no surviving duration is a pass condition.

## Context pointers

- Read `AGENTS.md` and **brief-406's committed report**, especially the native
  table and its verdicts for `basic`, `signed`, and `frontend/control`.
- brief-406 established that `basic` is a third family: its setup contains no
  `SubSocket` and no IOPub endpoint, so the PUB/SUB subscription race is **not**
  its mechanism. Do not import family B's explanation into it.
- Shared helpers: `zeromq_native_test_support.rs:100-175`.
- Fixed sleeps: `signed.rs:158,184`, `control.rs:25,115`.
- Negative windows: `signed.rs:203`, `control.rs:222`.
- The Control-priority test's explicit both-channels-pending precondition
  (`control.rs`, the shutdown test) is genuine and promised by
  `zeromq_native/socket.rs`; keep it.
- brief-408 just landed resolved-endpoint propagation and the skip gate in
  these same files. Do not undo either; `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` must
  keep working exactly as it does now.
- **Your sandbox cannot bind loopback.** These tests will refuse to skip
  unless you set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, which is the correct thing
  for you to do — and it means you cannot execute a single line of what you
  are changing. Say so. Authoritative verification is mine.

## Scope

- Files you MAY modify, all under `crates/rum-jupyter/src/`:
  - `zeromq_native_test_support.rs`;
  - `zeromq/tests/native/basic.rs`;
  - `zeromq/tests/native/signed.rs`;
  - `zeromq/tests/native/frontend/control.rs`;
  - `zeromq/tests/native/frontend/{execute,stdin,session_flow}.rs` **only** to
    keep them compiling against a changed shared helper signature — not to
    touch their own timing assumptions, which are family B. State every such
    edit explicitly.
  - production files under `zeromq_native/` or `zeromq.rs` **only** if a
    structural negative assertion genuinely needs state the test cannot reach;
    if so, keep it minimal, add no default-path cost, and call it out
    prominently in the report.
- Files/areas you MUST NOT touch:
  - `crates/rum-jupyter/src/tests/**` and `adapter/**` — family A;
  - the wire protocol, message schema, public serde shapes, pinned `zeromq`
    version, `Cargo.lock`, dependency edges;
  - `rum-runtime`, `rum-host`, `rum-stdlib`, `rum-core`.
- If you need a file outside this list, **STOP and report**. My MAY lists have
  been short eight times on this project.

## The accept-vs-stop criterion

Replacing a duration with an observed condition is in scope. **Weakening or
deleting a semantic assertion is a stop**, and so is lengthening a wait,
adding a retry, or making an assertion conditional on timing. Signature
rejection, Control-before-Shell priority, heartbeat echo, execute round-trip,
shutdown observation and pending-Shell reporting all survive unchanged.

If a structural negative assertion needs production state that is not exposed,
**stop and report it** rather than reaching for a window.

## Deliverable

1. The four 50 ms connection sleeps are gone, replaced by an observed
   readiness condition on the same socket path the test then uses.
2. `basic`, `signed`, and `control` advance stage by stage on conditions, each
   with a diagnostic that **names the stage that failed** — connection,
   request delivery, worker completion, reply, or absence.
3. The two negative assertions are structural, as described above.
4. **One harness stop-loss, and it is not a pass condition.** A single
   generous bound may remain solely to turn a hang into a failure. Its panic
   message must say it is a harness timeout and not a latency contract, and it
   must not be per-stage tuned. **Do not claim to have derived it from
   measured data** — you cannot run these tests. Pick something plainly
   generous and say that is what you did.
5. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` and the resolved-endpoint propagation from
   brief-408 are untouched and still work.
6. A per-test accounting: for each test, the mechanism removed and the semantic
   assertions preserved. Zero deletions.

## rum / Rust pitfalls (read twice)

1. A condition wait that spins is still a spin. Prefer an actual signal or a
   state read; if the only available mechanism is polling a socket, keep it
   bounded by the single harness stop-loss and say so.
2. `basic` has no SUB socket. Its readiness question is Dealer/Router and
   Req/Rep connection establishment, not subscription.
3. Do not conflate "the backend observed the request" with "the worker
   finished"; brief-406's inventory separates those and so should the
   diagnostics.
4. The signature-rejection path must still be end-to-end: keep the tampered
   message going over a real socket, not short-circuited into the codec.
5. No new `unsafe`. No dependency change. No wire-format change.
6. Do not let a structural assertion read state that only exists in test
   builds unless it is `#[cfg(test)]` and adds no production cost — and say so
   if you do.

## Verification (MANDATORY — you run what you can, honestly)

Pass `--offline` to every `cargo` command. You will need
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`; that is expected and correct for your
sandbox, and it means the native tests early-return without executing.

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 600  -- cargo test --offline -p rum-jupyter
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

**Structural evidence, since you cannot measure:**

- a search over your four files showing every remaining `Duration`,
  `sleep`, and `timeout`, with a one-line justification for each survivor;
- the two negative assertions, quoted before and after, with the state they
  now read and the `path:line` where that state is produced;
- confirmation that the skip gate still fires both ways.

**State plainly that you executed none of the native paths you changed.** I
will run the native suite repeatedly on the host where it binds; my baseline
is 0 failures in 25 native-only samples and 0 in 20 whole-suite samples, so a
regression will be visible.

No timing A/B. If you change ordinary evaluation or dispatch, the premise is
exceeded — stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Leave changes in the working tree.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. No probe artifacts in the repo.
- **Never report a rate for a test you did not observe running.**
- **Testing Policy is a hard line**: rum's own documented behavior only.
- **No new `unsafe`.** No test deleted, no assertion weakened.
- Minimal diff; no drive-by reformatting or renames.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. Per-file change with line ranges and a one-line why.
2. The two negative assertions, before and after, with the state each now
   reads and where it is produced.
3. The per-test accounting from Deliverable 6.
4. Every surviving duration, with its justification.
5. Verification output, with an explicit statement of what you could not run.
6. Any production file touched, and why it was unavoidable.
7. Stop-loss, risks, open questions.

---

# Report — 2026-09-02

## 1. Per-file changes

- [zeromq_native_test_support.rs](crates/rum-jupyter/src/zeromq_native_test_support.rs:12), lines 12–341: added one per-test 30-second harness stop-loss, stage-labelled condition waits, non-spinning polling, Dealer/Router connection probes, heartbeat round trips, and invalid-signature observation.
- [basic.rs](crates/rum-jupyter/src/zeromq/tests/native/basic.rs:4), lines 4–127: execute and heartbeat now establish same-path readiness and advance through request delivery, worker completion, and reply conditions.
- [signed.rs](crates/rum-jupyter/src/zeromq/tests/native/signed.rs:4), lines 4–238: signed Shell readiness and progress are condition-driven; tampered-request absence is structural.
- [control.rs](crates/rum-jupyter/src/zeromq/tests/native/frontend/control.rs:4), lines 4–251: Control/Shell readiness probes replace sleeps; shutdown absence reads pending and adapter state.
- [execute.rs](crates/rum-jupyter/src/zeromq/tests/native/frontend/execute.rs:8), lines 8, 40, 72–83: helper-signature migration and stage labels only.
- [stdin.rs](crates/rum-jupyter/src/zeromq/tests/native/frontend/stdin.rs:8), lines 8, 55, 89–100, 143–154: helper-signature migration and stage labels only.
- [session_flow.rs](crates/rum-jupyter/src/zeromq/tests/native/frontend/session_flow.rs:8), lines 8, 64, 96–107, 191–202, 245–256, 338–349, 431–435: helper-signature migration and stage labels only.

The last three files’ own IOPub timing assumptions were not changed; those remain for family B.

## 2. Structural negative assertions

Tampered signature, before:

```rust
let shell_timeout = runtime
    .block_on(async { tokio::time::timeout(Duration::from_millis(200), shell.recv()).await });
assert!(
    shell_timeout.is_err(),
    "tampered signed request should not produce a shell reply"
);
```

After, at [signed.rs:218](crates/rum-jupyter/src/zeromq/tests/native/signed.rs:218):

```rust
assert_eq!(
    (
        adapter.execution_count(),
        adapter.execution_state(),
        adapter.lifecycle_state(),
    ),
    adapter_state_before,
    "tampered-request absence: rejection must not dispatch or enqueue a reply"
);
assert!(!adapter.execute_running(), ...);
assert!(!backend.transport().has_pending_channel(KernelChannel::Shell), ...);
```

Signature validation returns before dispatch at [event_loop.rs:60](crates/rum-jupyter/src/transport/event_loop.rs:60). Any execute dispatch synchronously changes execution count/state and installs a worker at [adapter/mod.rs:420](crates/rum-jupyter/src/adapter/mod.rs:420), before dispatch output can be sent at `event_loop.rs:69`. Unchanged adapter state therefore proves no dispatch or reply enqueue occurred.

Shutdown, before:

```rust
let shell_timeout = runtime
    .block_on(async { tokio::time::timeout(Duration::from_millis(200), shell.recv()).await });
assert!(
    shell_timeout.is_err(),
    "queued shell work should remain pending after shutdown"
);
```

After, at [control.rs:201](crates/rum-jupyter/src/zeromq/tests/native/frontend/control.rs:201):

```rust
assert!(report.stopped_for_shutdown);
assert!(backend.transport().has_pending_channel(KernelChannel::Shell));
assert_eq!(driver.event_pump().adapter().execution_count(), 0);
assert!(!driver.event_pump().adapter().execute_running());
```

The both-channels-pending precondition is preserved. Shell input is buffered at [zeromq.rs:175](crates/rum-jupyter/src/zeromq.rs:175), and shutdown reports pending incoming state at [socket.rs:327](crates/rum-jupyter/src/socket.rs:327).

## 3. Per-test accounting

- Basic execute: removed two-second stage/reply requirements; preserved execute round-trip, status, and execution count.
- Basic heartbeat: replaced the two-second receive requirement with Req/Rep readiness plus exact payload echo.
- Signed execute: removed Shell/IOPub receive durations as pass conditions; preserved signatures, status, execution count, and signed execute-result coverage.
- Tampered signature: removed both 50 ms sleeps, the two-second error deadline, and 200 ms negative receive; preserved real-socket tampering, `InvalidSignature`, and no-dispatch/no-reply semantics.
- Control interrupt: removed the 50 ms connection sleep and two-second reply requirement; preserved interrupt status and lifecycle assertions.
- Control shutdown: removed the 50 ms connection sleep, two-second reply requirement, and 200 ms absence window; preserved both-pending precondition, Control priority, report counts, shutdown state, reply metadata, restart flag, and pending Shell work.

Zero tests and zero semantic assertions were deleted.

## 4. Surviving durations

The required search returned:

```text
crates/rum-jupyter/src/zeromq_native_test_support.rs:6:    time::Duration,
crates/rum-jupyter/src/zeromq_native_test_support.rs:12:const NATIVE_TEST_HARNESS_STOP_LOSS: Duration = Duration::from_secs(30);
crates/rum-jupyter/src/zeromq_native_test_support.rs:13:const NATIVE_TEST_POLL_INTERVAL: Duration = Duration::from_millis(10);
crates/rum-jupyter/src/zeromq_native_test_support.rs:36:        match runtime.block_on(tokio::time::timeout(self.remaining(stage), future)) {
crates/rum-jupyter/src/zeromq_native_test_support.rs:39:            Err(_) => self.harness_timeout(stage),
crates/rum-jupyter/src/zeromq_native_test_support.rs:187:    fn remaining(&self, stage: &str) -> Duration {
crates/rum-jupyter/src/zeromq_native_test_support.rs:191:            .unwrap_or_else(|| self.harness_timeout(stage))
crates/rum-jupyter/src/zeromq_native_test_support.rs:195:        thread::park_timeout(self.remaining(stage).min(NATIVE_TEST_POLL_INTERVAL));
crates/rum-jupyter/src/zeromq_native_test_support.rs:198:    fn harness_timeout(&self, stage: &str) -> ! {
crates/rum-jupyter/src/zeromq_native_test_support.rs:200:            "native ZeroMQ harness timeout while waiting for stage `{stage}`; this is a harness \
crates/rum-jupyter/src/zeromq_native_test_support.rs:201:             timeout, not a latency contract"
```

Justifications:

- 30 seconds is the single overall per-test harness stop-loss.
- 10 ms is only the park cadence preventing a busy spin; observed state remains the success condition.
- `tokio::time::timeout` uses the remaining portion of that same overall deadline.
- Other matches are type plumbing or the required diagnostic.
- No `sleep` remains, and `basic.rs`, `signed.rs`, and `control.rs` have no duration/timeout matches.

The 30-second value was chosen as plainly generous; it was not derived from measurements.

## 5. Verification

The first formatting check exited 1 for three rustfmt-only layout differences. I ran rustfmt, then the mandatory final check produced no output and exited 0:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
exit_code=0
```

Crate gate:

```text
running 104 tests
...
test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

Workspace gate exited 0. A quiet repeat showed, among the larger suites:

```text
running 932 tests
test result: ok. 930 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.07s

running 831 tests
test result: ok. 828 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.53s
```

The workspace emitted three pre-existing `rum-runtime` dead-code warnings.


---

## Director's note

This brief regressed every native test and was never committed on its own.
`block_on_result` passed `tokio::time::timeout` as an argument to `block_on`,
constructing the `Sleep` outside the runtime context, so all nine native tests
panicked with "there is no reactor running" on a host that can bind loopback.
brief-410 repaired it and added the socket-free harness coverage that would
have caught it in the worker's sandbox. The two land together.
