# brief-408 — implement: propagate resolved bind endpoints, and make a native skip impossible to miss

Shape: **implementation**. Family D of the four brief-406 identified, taken
second because the race it removes would otherwise contaminate the gates of
families B and C.

## Where this came from

brief-406 found a defect nobody was looking for. The native test helper
reserves five ephemeral ports, reads their numbers, **drops every listener**,
and only then lets the backend bind them
(`zeromq_native_test_support.rs:9-35`). Between the drop and the bind, any
process on the machine can take one of those ports. No amount of test
synchronization fixes a TOCTOU.

**This is not primarily a test problem.** The wrapper discards information the
pinned library already returns. `zeromq-0.5.0`'s `Socket::bind` yields the
resolved endpoint (`src/lib.rs:227-243`) and supports port zero
(`src/util.rs:265-283`), but `handle_lifecycle_command` throws it away with
`.map(|_| ())` (`zeromq_native/workers.rs:249-261`), and `setup_socket`
(`zeromq.rs:410-427`) has nowhere to put it. Recovering that value is a
fidelity fix for the wrapper; removing the TOCTOU is what it buys.

## The second defect this brief closes

brief-406 established something that has already cost this project a false
result: **a skipped native test is reported by libtest as `ok`.**
`loopback_ports` prints one line to stderr and returns `None`, each native
test early-returns, and the suite summary reads nine passed. Without
`--nocapture` a skip and a pass are indistinguishable — which is exactly how
brief-384 recorded a 0-of-30 stability result that measured nothing.

Since this brief is already rewriting that helper, close the trap:

**A skipped native test must fail, unless skipping was explicitly requested.**
Gate the skip behind an environment variable — name it something unmistakable
such as `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` — so that a sandbox which cannot bind
loopback opts in visibly on the command line, while on any host that can bind,
a skip is a red test rather than a green one. Silence must stop being an
option. The project's standing preference applies here: a property the harness
enforces beats one that relies on a reviewer remembering to pass
`--nocapture`.

## Goal

Native sockets bind port zero themselves and report the resolved endpoints
back through the internal lifecycle reply, so the test frontend connects to
ports that were never released; and a native test that does not run says so
loudly.

## Context pointers

- Read `AGENTS.md` and **brief-406's committed report**, especially §1 (the
  skip mechanism, verbatim) and the native table's shared note about the
  reserve/drop/rebind interval.
- The reserve/drop gap and the skip: `zeromq_native_test_support.rs:9-35`.
- Endpoint discarded: `zeromq_native/workers.rs:249-261`.
- Wrapper seam: `zeromq.rs:405-427`, plus the `ZeroMqSocket` trait's `bind`.
- Lifecycle command and reply plumbing: `zeromq_native/commands.rs`,
  `handles.rs`, `socket.rs`.
- Pinned library behaviour: `Cargo.lock:1222-1226`; cached
  `zeromq-0.5.0/src/lib.rs:227-243` and `src/util.rs:265-283`.
- **Your sandbox cannot bind loopback.** The native tests will skip. Say so;
  make no claim about them. Authoritative native verification happens on my
  host — see Verification.

## Scope

- Files you MAY modify, all under `crates/rum-jupyter/src/`:
  - `zeromq.rs`, `zeromq_native/commands.rs`, `zeromq_native/handles.rs`,
    `zeromq_native/socket.rs`, `zeromq_native/workers.rs`;
  - `zeromq_native_test_support.rs`;
  - `zeromq_fake_tests.rs`;
  - the native tests under `zeromq/tests/native/**`, **only** for endpoint
    setup and the skip gate — not for their timing assumptions, which are
    families B and C;
  - `bootstrap.rs` / the transport-layout type, only if the resolved ports
    must reach the session configuration.
- Files/areas you MUST NOT touch:
  - `crates/rum-jupyter/src/tests/**` and `adapter/**` — family A landed there
    in brief-407;
  - the wire protocol, message schema, public serde shapes, the pinned
    `zeromq` version, `Cargo.lock`, dependency edges;
  - `rum-runtime`, `rum-host`, `rum-stdlib`, `rum-core`;
  - the two-second receive loops and fixed sleeps in native tests — families
    B and C. If you touch one incidentally, say which and why.
- If you need a file outside this list, **STOP and report**. My MAY lists have
  been short eight times on this project, twice on module roots specifically.

## The accept-vs-stop criterion

Recovering the resolved endpoint and threading it to the caller is in scope.
**A retry loop is a stop.** Re-binding until a port happens to be free papers
over the race rather than removing it, and would leave the interval in place.
So is any change to the wire protocol, the message schema, or the pinned
dependency. If the propagation cannot stay internal and safe — if it wants
`unsafe`, a raw fd transfer, or a public compatibility break — stop and report.

## Deliverable

1. `bind` propagates the resolved endpoint from the pinned library through the
   lifecycle reply, the socket handle, and the wrapper, instead of `.map(|_| ())`.
2. Native test configuration requests port zero for each of the five sockets.
3. The backend's transport layout and session configuration are updated to the
   resolved endpoints **before** any frontend socket connects.
4. The five-`TcpListener` reservation is gone. A capability probe may remain
   solely to classify `PermissionDenied`; its port must not be used as a
   reservation, and the report must show that nothing reads it.
5. The skip gate: a native test that cannot bind loopback **fails** unless the
   opt-out environment variable is set, and the failure message names the
   variable so the next reader knows what to do.
6. Fake-backend coverage in `zeromq_fake_tests.rs` for resolved-endpoint
   propagation, since that is the only part of this you can actually execute.

## rum / Rust pitfalls (read twice)

1. Port zero means the endpoint string the caller passed is **not** the one the
   socket is on. Every later use must read the resolved value, not re-derive it
   from the connection file.
2. Five sockets resolve independently; do not assume contiguity or ordering.
3. The heartbeat and IOPub sockets are bound by different code paths than
   Shell/Control/Stdin — check each rather than assuming one seam covers all.
4. Do not change what `KernelConnectionFile` means on the wire; port zero is a
   test-configuration choice, not a protocol change.
5. No new `unsafe`, no raw fd transfer, no dependency bump.
6. Keep the skip classification for `PermissionDenied` distinct from a genuine
   bind failure; the latter must still panic.

## Verification (MANDATORY — you run what you can, honestly)

Pass `--offline` to every `cargo` command.

**First, the capability statement.** Run the native tests once with
`--nocapture` and paste the verbatim output showing run-versus-skip, before
any change. State plainly that you cannot execute native socket behaviour.

```
scripts/run-bounded.py --timeout 300 -- cargo fmt --check
scripts/run-bounded.py --timeout 300 -- cargo test --offline -p rum-jupyter zeromq_fake_tests -- --nocapture
scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-jupyter
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

`--no-fail-fast` is required; `cargo test` otherwise stops after a failing
binary and under-reports the rest.

**Structural evidence, since you cannot measure.** Paste:
- a search showing no remaining `TcpListener` reservation feeds any endpoint;
- the resolved-endpoint path, from the library call to the frontend's connect,
  as a chain of `path:line`;
- the skip gate firing: run the suite with the opt-out unset and show the
  native tests failing loudly in your sandbox, then with it set and show them
  skipping. **That is your proof, and it is one you can actually produce.**

I will run the native suite on the host where it binds. Do not claim any
native pass rate.

No timing A/B. If you change ordinary evaluation, dispatch, or the wire
format, the premise is exceeded — stop and report.

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
2. The resolved-endpoint chain as `path:line`, from library call to connect.
3. The skip gate, demonstrated both ways, verbatim.
4. Evidence the reservation is gone and nothing reads the probe's port.
5. Verification output, with an explicit statement of what you could not run.
6. Cost statement, and confirmation the wire format is unchanged.
7. Stop-loss, risks, open questions.

---

# Report — 2026-09-02

## 1. Per-file changes

- [zeromq.rs](crates/rum-jupyter/src/zeromq.rs:28): added a source-compatible resolved-bind hook; stores all five resolved endpoints in the socket session, transport layout, and backend-owned configuration.
- [commands.rs](crates/rum-jupyter/src/zeromq_native/commands.rs:3): bind lifecycle replies now carry the resolved endpoint string.
- [handles.rs](crates/rum-jupyter/src/zeromq_native/handles.rs:51): Router, Pub, and Rep handles return their bind reply instead of unit.
- [socket.rs](crates/rum-jupyter/src/zeromq_native/socket.rs:48): propagates resolved binds through all three native socket kinds.
- [workers.rs](crates/rum-jupyter/src/zeromq_native/workers.rs:262): preserves `zeromq::Socket::bind`’s returned endpoint and keeps post-start bind/connect errors type-correct.
- [zeromq_native_test_support.rs](crates/rum-jupyter/src/zeromq_native_test_support.rs:10): replaces five port reservations with one capability-only probe, requests five port-zero binds, and adds the explicit skip gate.
- [zeromq_fake_tests.rs](crates/rum-jupyter/src/zeromq_fake_tests.rs:213): covers five independent, non-contiguous resolved endpoints reaching the layout and both session configurations.
- [basic.rs](crates/rum-jupyter/src/zeromq/tests/native/basic.rs:4): Shell and heartbeat frontends now read backend-resolved endpoints.
- [signed.rs](crates/rum-jupyter/src/zeromq/tests/native/signed.rs:4): signed Shell, IOPub, and heartbeat setup now uses resolved endpoints.
- [control.rs](crates/rum-jupyter/src/zeromq/tests/native/frontend/control.rs:4): both Control flows use resolved Shell/Control endpoints.
- [execute.rs](crates/rum-jupyter/src/zeromq/tests/native/frontend/execute.rs:4): uses resolved Shell, IOPub, and heartbeat endpoints.
- [stdin.rs](crates/rum-jupyter/src/zeromq/tests/native/frontend/stdin.rs:4): uses resolved Shell, Stdin, IOPub, and heartbeat endpoints.
- [session_flow.rs](crates/rum-jupyter/src/zeromq/tests/native/frontend/session_flow.rs:4): uses all five resolved endpoints.

No file outside the brief’s MAY list changed.

## 2. Resolved-endpoint chain

[zeromq-0.5.0 `Socket::bind`](cargo-registry/zeromq-0.5.0/src/lib.rs:232)  
→ [worker retains `Endpoint::to_string()`](crates/rum-jupyter/src/zeromq_native/workers.rs:267)  
→ [bind reply carries `Result<String, String>`](crates/rum-jupyter/src/zeromq_native/commands.rs:4)  
→ [Router handle receives it](crates/rum-jupyter/src/zeromq_native/handles.rs:51), with equivalent Pub/Rep paths at lines 136 and 217  
→ [native wrapper dispatches by socket kind](crates/rum-jupyter/src/zeromq_native/socket.rs:53)  
→ [`setup_socket` installs the returned endpoint](crates/rum-jupyter/src/zeromq.rs:419)  
→ [backend layout/session configuration copy it](crates/rum-jupyter/src/zeromq.rs:292)  
→ [native frontend reads that layout](crates/rum-jupyter/src/zeromq/tests/native/frontend/session_flow.rs:14)  
→ [Shell/Control/Stdin/IOPub connect](crates/rum-jupyter/src/zeromq/tests/native/frontend/session_flow.rs:21) and [heartbeat connects](crates/rum-jupyter/src/zeromq_native_test_support.rs:81).

## 3. Skip gate, both ways

Unset, verbatim excerpt:

```text
running 9 tests

thread 'zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:23:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
```

```text
test result: FAILED. 0 passed; 9 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-jupyter --lib`
```

With explicit opt-out, verbatim excerpt:

```text
running 9 tests
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.00s
```

That final libtest summary consists entirely of explicit early-return skips, not native passes.

## 4. Reservation removal evidence

```text
$ rg -n 'TcpListener|local_addr|\.port\(\)|ports\[' crates/rum-jupyter/src/zeromq_native_test_support.rs
3:    net::TcpListener,
13:    match TcpListener::bind("127.0.0.1:0") {
```

There is no `local_addr`, `.port()`, port collection, or endpoint read from the probe. It exists only to classify `PermissionDenied`.

```text
44:        shell_port: 0,
45:        iopub_port: 0,
46:        stdin_port: 0,
47:        control_port: 0,
48:        hb_port: 0,
```

A search for `bootstrap.transport_layout()` in native tests produced no output. No bind retry loop was added.

## 5. Verification

Final results:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
```

Exit 0, no output.

The brief’s literal `zeromq_fake_tests` filter selected zero tests because the file is included as `zeromq::tests::fake`. I therefore ran the real module path:

```text
running 4 tests
test zeromq::tests::fake::zeromq_backend_propagates_independently_resolved_bind_endpoints ... ok
test zeromq::tests::fake::zeromq_backend_binds_expected_socket_endpoints ... ok
test zeromq::tests::fake::zeromq_backend_reports_malformed_shell_frames_as_structured_wire_errors ... ok
test zeromq::tests::fake::zeromq_backend_round_trips_execute_over_fake_router_socket ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 100 filtered out; finished in 0.01s
```

Crate gate, with explicit sandbox opt-out:

```text
test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

The native entries in that aggregate were early-return skips.

Workspace gate:

```text
RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

Exit 0. The 2,495-line output contained three existing `rum-runtime` dead-code warnings and ended with green doc-test results.

During iteration, the first formatting check found one wrapping difference, and the first focused compilation found four combined match arms whose bind/connect reply types differed. Both were corrected before the final green gates.

I could not execute any native socket behavior in this sandbox. Authoritative native verification remains for the loopback-capable host.

## 6. Cost and compatibility

The added work occurs only during ZeroMQ backend construction: five endpoint strings cross lifecycle reply channels, are parsed once, and are cloned into effective configuration. No evaluator, dispatch, matcher, or steady-state socket hot path changed; no timing A/B was warranted.

The wire protocol, message schema, serde shapes, dependency graph, pinned `zeromq` version, and `Cargo.lock` are unchanged. The existing public `ZeroMqSocket::bind` signature remains source-compatible through a default resolved-bind hook. No `unsafe` or raw-fd transfer was introduced.

