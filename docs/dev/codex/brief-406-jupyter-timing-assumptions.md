# brief-406 — diagnose: the rum-jupyter test suite's timing assumptions

Shape: **diagnostic**. `Scope: NONE — no source change.` The deliverable is
evidence plus drafted implementation briefs, not a fix.

This is deliberately not an implementation brief even though one of the two
halves is already diagnosed. brief-388 closed the two instances that had been
seen rather than the class, because diagnosis and repair both looked only at
recorded sightings. This suite has now produced **seven distinct failing
tests** across two subsystems in one session. Fixing the two I understand
would repeat that mistake.

## Read this first: your sandbox may not be able to measure half of this

`TODO.md` records that the native zeromq tests are **skipped in the worker's
sandbox** — "loopback socket binding is not permitted" — and that the
hypothesis about them was reached by reading the dependency, not by
reproduction. It also records an inconsistency worth taking seriously: during
brief-384 the same worker reported those tests *running* rather than skipping,
which means a 0-of-30 result from that run may have measured nothing at all.

**Therefore your first action, before anything else, is to establish and report
what your sandbox actually does.** Run the native zeromq tests once and paste
the verbatim output showing whether each ran, passed, failed, or skipped. If
they skip, say so plainly and **do not report any rate, count, or stability
claim about them** — analyze them by reading only, and label every such
conclusion as derived from source rather than observed. I will do the
measurement on the host where they demonstrably run.

That instruction is the point of this brief as much as the diagnosis is. An
instrument that silently measures nothing has cost this project a false
result before.

## The evidence I already have

Measured by me on this host, where these tests demonstrably do run.

Whole-suite `cargo test -p rum-jupyter`, sampled repeatedly across four
different trees during briefs 402-405:

| Tree | Samples | Whole-suite runs with ≥1 failure |
|---|---|---|
| `4abdd19f` (before this line) | 10 | 1 |
| after brief-402/403 | 10 | 3 |
| after brief-404 | 30 | 5 |

The failing test **moves between runs**. Every distinct failure seen this
session:

| Test | Note |
|---|---|
| `tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle` | brief-397 finding B. **Fixed by brief-404** — 0 occurrences in 30 samples afterwards, against a prior rate near 2 in 10. Not your subject; listed so the family is complete. |
| `tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately` | brief-397 finding A. `pump.rs:37`, `assert!(pump.pump_execute_completion(...))`. **Fails 30/30 when run in isolation**, yet usually passes inside the whole suite. In-process transport mock, no sockets. |
| `zeromq::tests::native::frontend::stdin::…_stdin_round_trip_emits_iopub_and_shell_reply` | 0/30 in isolation |
| `zeromq::tests::native::signed::…_rejects_tampered_signed_requests_end_to_end` | |
| `zeromq::tests::native::basic::…_round_trips_execute_over_loopback_sockets` | |
| `zeromq::tests::native::basic::…_echoes_heartbeat_over_loopback_req_socket` | |
| `zeromq::tests::native::frontend::session_flow::…_multi_message_session_flow_tracks_counts_and_shutdown` | 0/20 in isolation |

Two things in that table need explaining and neither is explained today:

1. **Finding A is inverted.** It fails deterministically alone and usually
   passes in company. That is the opposite of an ordinary flake and says the
   suite is supplying something the test needs but does not establish.
2. **`TODO.md`'s existing hypothesis does not cover the observed set.** It
   names four files carrying a fixed 50 ms PUB/SUB propagation delay —
   `native/signed.rs:24`, `frontend/execute.rs:24`, `frontend/stdin.rs:39`,
   `frontend/session_flow.rs:48` — but two of the failures I recorded are in
   `basic`, which is not on that list. Either the class is larger than the
   entry says, or there is a second cause.

## The question this brief must answer

**What does each of these tests assume about timing or ordering that the code
under test does not promise?** Not "why did this run fail" — that is a
symptom. For each failure, decide between:

- the test asserts a property the kernel or poller genuinely guarantees, so
  the failure is a real defect; or
- the test asserts a timing property where the implementation promises
  something weaker or different — a priority, an eventual delivery, a
  cooperative check — so the test is wrong.

brief-397 already made that distinction once for finding A and concluded the
immediate-readiness assumption is invalid. Confirm or refute it against the
current tree, then extend the same judgement to the rest.

## Sweep the class, not the sightings

The list above is what happened to fail while I was watching something else.
It is a sample, not the population. **Enumerate every test in
`crates/rum-jupyter` that depends on a timing or ordering property**, whether
or not it has ever failed: fixed sleeps, bounded receive loops, "already
ready" assertions, assumptions about how many messages have arrived,
assumptions about the order of two channels, and any assertion whose truth
depends on a thread having made progress. Give each a row: `path:line`, the
property it assumes, whether the implementation promises it, and whether it
has been observed failing.

That inventory is the primary deliverable. If it turns out that only the seven
above qualify, say so — but derive it, do not assume it.

## What else to establish

- **Why finding A inverts.** What does the whole suite provide that isolation
  does not? Name the mechanism with `path:line`, do not speculate.
- Whether the pump failures and the zeromq failures share a root or are two
  families that merely co-occur.
- Whether the pinned `zeromq` 0.5.0 subscription behaviour described in
  `TODO.md` is the actual mechanism for the `basic` tests too, or whether those
  need a separate explanation.
- Whether any of these tests, if written correctly, would have caught a real
  kernel defect — I want to know what coverage is lost if a test is simply
  deleted rather than repaired.

## Deliverable

1. The sandbox capability statement, first, verbatim.
2. The class inventory table described above.
3. A verdict per test: real defect, or invalid timing assumption, with the
   `path:line` evidence for what the implementation actually promises.
4. The finding-A inversion mechanism.
5. **Drafted implementation briefs** in `TEMPLATE.md` shape, one per repair
   family rather than one per test, each with its own scope, gate, and
   accept-vs-stop criterion. They are raw material for me to rewrite; write
   them as if they were to be dispatched.
6. A statement of what `TODO.md:707-724` gets right and what it misses, since
   my data already contradicts part of it.

## Hard rules

- **No source change.** `Scope: NONE`. You may read anything in the repository
  and run tests; you may not edit a single file, including tests and docs.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. Pass `--offline` to every `cargo` command.
- **Never report a rate for a test you did not observe running.** If it
  skipped, say it skipped. This is the brief's central instruction.
- **Testing Policy is a hard line**: rum's own documented behavior only. No
  vendor engine, oracle harness, or compatibility matrix.
- Every claim carries `path:line` evidence. Say plainly where you are
  reasoning from source rather than from observation.
- Paste genuine verbatim output. If a run did not produce evidence, say so.

## Report format

1. Sandbox capability, verbatim.
2. The class inventory table.
3. Per-test verdicts with evidence.
4. The finding-A inversion mechanism.
5. The drafted implementation briefs.
6. What `TODO.md` gets right and wrong.
7. Stop-loss, risks, open questions.

---

# Report — 2026-09-02

## 1. Sandbox capability — verbatim

First command run:

```text
scripts/run-bounded.py --timeout 180 -- cargo test -p rum-jupyter --offline zeromq::tests::native -- --nocapture
```

Verbatim output:

```text
   Compiling rum-runtime v0.2.0 (crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (crates/rum-stdlib)
   Compiling rum-host v0.2.0 (crates/rum-host)
   Compiling rum-jupyter v0.2.0 (crates/rum-jupyter)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.17s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-0d2044aa0c254386)

running 9 tests
skipping native ZeroMQ test: loopback socket binding is not permitted
skipping native ZeroMQ test: loopback socket binding is not permitted
skipping native ZeroMQ test: loopback socket binding is not permitted
skipping native ZeroMQ test: loopback socket binding is not permitted
skipping native ZeroMQ test: loopback socket binding is not permitted
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... ok
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... ok
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... ok
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... ok
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... ok
skipping native ZeroMQ test: loopback socket binding is not permitted
skipping native ZeroMQ test: loopback socket binding is not permitted
skipping native ZeroMQ test: loopback socket binding is not permitted
skipping native ZeroMQ test: loopback socket binding is not permitted
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... ok
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... ok
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.00s

     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-d4ab5a7c60b11b3d)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Each listed test took its internal early-return skip path. The “ok” labels do not mean native socket behavior ran: `loopback_ports` prints the skip and returns `None` at `crates/rum-jupyter/src/zeromq_native_test_support.rs:10-35`, and each native test immediately returns when that happens, e.g. `zeromq/tests/native/basic.rs:5-7,86-88` and `zeromq/tests/native/signed.rs:5-7,138-140`.

I make no native pass-rate, failure-rate, count, or stability claim. All native conclusions below are explicitly source-derived.

## 2. Class inventory

Legend:

- **P** — the implementation structurally promises the property; a correctly synchronized failure is a real defect.
- **E** — eventual completion of finite work is the intended property, with no wall-clock deadline.
- **I** — invalid immediate-readiness or elapsed-time assumption.
- **M** — promised semantic coverage combined with an invalid timing mechanism.

Pure JSON/parser collection ordering was excluded because it has no clock, worker, channel arrival, or progress dependency.

### Structural ordering tests

| Test | Assumed property | Promise and verdict | Observed |
|---|---|---|---|
| `tests/adapter.rs:150-212` | stderr message, trace, then result | **P.** Rendering explicitly appends messages, trace, result in that order at `adapter/render.rs:53-90`. | Not sampled here |
| `tests/adapter.rs:273-304` | trace before execute result | **P**, by `adapter/render.rs:61-71`. | Not sampled |
| `tests/adapter/dispatch.rs:78-110` | non-execute Busy then Idle | **P**, constructed in that order at `adapter/mod.rs:518-527`. | Not sampled |
| `tests/adapter/dispatch.rs:113-146` | comm-info emits exactly the two lifecycle statuses | **P**, through `adapter/mod.rs:349-353,518-529`. | Not sampled |
| `tests/adapter/dispatch.rs:327-392` | synchronous interrupt reply and Busy/Idle status order | **P**, through `adapter/mod.rs:360-363,518-529`. | Not sampled |
| `tests/pump.rs:56-104` | Control before Shell when both are queued | **P.** Idle priority is Control, Shell, Stdin at `transport/event_loop.rs:160-170`; mock queues are synchronous at `transport/local.rs:40-81`. | Not sampled |
| `tests/pump.rs:107-172` | Stdin before Shell while awaiting input | **P**, explicitly ordered at `transport/event_loop.rs:154-170`. | Not sampled |
| `tests/pump.rs:346-410` | queued shutdown on Control stops before later Shell work | **P**, through `transport/event_loop.rs:148-170,183-208`. | Not sampled |
| `tests/transport_coverage.rs:209-247` | in-memory FIFO routing and heartbeat FIFO | **P**, via mutex-protected `VecDeque` operations at `transport/local.rs:169-224,294-312`. | Not sampled |
| `tests/bootstrap.rs:347-360` | local heartbeat replies retain FIFO order | **P**, at `transport/local.rs:211-224,244-258`. | Not sampled |
| `zeromq_fake_tests.rs:165-196` | factory socket creation order maps to Shell, Control, Stdin, IOPub, heartbeat | **P**, because construction is in that exact order at `zeromq.rs:98-138`. | Not sampled |

### In-process worker-dependent tests

The shared two-second helper is `tests/mod.rs:27,66-75`. Production only exposes a nonblocking poll: `adapter/mod.rs:301-319` delegates to `JoinHandle::is_finished()` at `adapter/worker.rs:71-82`; it promises neither immediate completion nor a two-second latency.

| Test | Assumed property | Promise and verdict | Observed |
|---|---|---|---|
| `tests/pump.rs:4-53` | worker is already finished at the first completion poll | **I.** No synchronization exists between spawn at `adapter/worker.rs:168-179` and the sample at `adapter/worker.rs:71-77`. | Failed in the current exact run; director also reported isolated failure |
| `tests/pump.rs:175-343` | huge execute remains busy at `:224-229`; cancellation finishes inside two seconds at `:248-258`; deferred Shell later runs | **M/I.** Control priority is promised by `event_loop.rs:140-170`; neither “still busy immediately” nor two-second completion is promised. Eventual request-scoped cancellation is now intended. | Historical real cancellation defect, fixed by brief-404; residual timing assumptions remain |
| `tests/adapter/dispatch.rs:4-75` | simple worker completes inside the shared deadline; exact Busy/Input/Result/Idle order | **M.** Deadline invalid; order promised by `adapter/iopub.rs:4-17` and `adapter/mod.rs:481-500`. | Not sampled |
| `tests/adapter/dispatch.rs:290-324` | simple worker completes inside the shared deadline | **I** for duration; semantic reply is valid eventual coverage through `adapter/mod.rs:386-407,466-501`. | Not sampled |
| `tests/wire.rs:4-56` | completion inside two seconds; reply is serialized before IOPub | **M.** Deadline invalid; reply-first batch order is explicit at `transport.rs:55-81`. | Not sampled |
| `tests/wire.rs:266-295` | completion inside two seconds; messages route by channel | **M.** Deadline invalid; routing is structural at `transport/local.rs:74-81`. | Not sampled |
| `tests/wire.rs:298-359` | completion inside two seconds; exact IOPub slots | **M.** Deadline invalid; IOPub sequence is structural at `adapter/iopub.rs:4-17` and `adapter/mod.rs:481-500`. | Not sampled |
| `tests/wire.rs:626-686` | finite execute eventually finishes under `pump_until_idle` | **P/E.** The loop explicitly yields until the worker finishes at `transport/event_loop.rs:177-212`; there is no latency claim, but there is no test stop-loss either. | Not sampled |
| `tests/stdin.rs:40-83` | suspension worker completes inside two seconds; prologue order is fixed | **M.** Deadline invalid; prologue order promised by `adapter/iopub.rs:4-17`. | Not sampled |
| `tests/stdin.rs:86-124` | disallowed-stdin worker completes inside two seconds | **I** for duration; eventual error response remains valid coverage through `adapter/mod.rs:410-447`. | Not sampled |
| `tests/stdin.rs:127-236` | suspension and resume workers eventually progress under unbounded pump loops | **P/E**, through `event_loop.rs:177-212` and resume spawn at `adapter/mod.rs:449-463`; hang-risk remains. | Not sampled |
| `tests/stdin.rs:239-298` | suspension and resume each finish inside the shared deadline | **I** for duration; resume semantics are valid coverage at `adapter/mod.rs:280-319,449-463`. | Not sampled |
| `tests/stdin.rs:301-373` | same two deadline-bounded worker transitions | **I** for duration; semantic side-effect preservation is valid coverage. | Not sampled |
| `tests/stdin.rs:376-439` | same two deadline-bounded transitions | **I** for duration; delayed-message resume coverage remains valuable. | Not sampled |
| `tests/stdin.rs:442-545` | four separate worker transitions each finish inside two seconds | **I** for every bound; prompt/backtracking sequence is valid semantic coverage. | Not sampled |
| `tests/bootstrap.rs:200-254` | local execute worker eventually finishes; exactly four IOPub messages exist | **P/E.** `pump_until_idle` waits without a deadline at `event_loop.rs:177-212`; four messages are structurally generated at `adapter/iopub.rs:4-17` and `adapter/mod.rs:481-500`. | Not sampled |
| `tests/bootstrap.rs:257-344` | suspension and resume workers eventually finish in unbounded loops | **P/E**, but a defect can hang the test indefinitely at `bootstrap.rs:284-316`. | Not sampled |
| `tests/bootstrap.rs:382-439` | heartbeat is selected before Shell, then execute eventually completes | **P/E.** Heartbeat priority is explicit at `socket.rs:198-225`; worker wait is unbounded at `socket.rs:320-362`. | Not sampled |
| `tests/bootstrap.rs:442-542` | a sender sleeps 2 ms, shutdown is observed under one second, cleanup under two seconds | **I.** The implementation checks Control cooperatively but has no one-second guarantee; the test’s staging and latency assertions are at `bootstrap.rs:488-506,527-541`. | Not sampled |
| `tests/bootstrap.rs:545-620` | heartbeat and stdin are serviced while resume workers eventually complete | **P/E.** Priority is explicit at `socket.rs:198-225`; completion wait is unbounded at `socket.rs:320-362`. | Not sampled |
| `zeromq_fake_tests.rs:198-251` | execute completion appears inside a fixed two-second polling loop | **I** for duration at `zeromq_fake_tests.rs:49-68`; fake socket delivery itself is synchronous at `:95-111`. | Not sampled |

### Native tests

Everything in this table is **source-derived, not observed in this sandbox**.

Every row also inherits an allocation race: five ports are reserved, extracted, then all listeners are dropped before the backend binds them at `zeromq_native_test_support.rs:10-35`. That release-before-bind gap is not a promise that the ports remain available.

| Test | Assumed property | Promise and verdict | Host evidence / this sandbox |
|---|---|---|---|
| `zeromq/tests/native/basic.rs:4-82` execute | Shell arrival, worker completion and Dealer reply all occur inside separate two-second waits at `:47-67` | **I.** Native receive and evaluator workers are asynchronous (`zeromq_native/threading.rs:12-25`, `adapter/worker.rs:168-187`); no duration is promised. It creates no SUB socket, so PUB/SUB cannot be its mechanism. | Reported failing on director host; skipped here |
| `zeromq/tests/native/basic.rs:85-123` heartbeat | heartbeat event and Req reply appear inside two seconds at `:110-121` | **I.** REP worker progress is asynchronous at `zeromq_native/workers.rs:145-228`; no two-second promise. No SUB socket exists. | Reported failing on director host; skipped here |
| `zeromq/tests/native/signed.rs:4-134` signed execute | `subscribe().await` plus heartbeat means IOPub is ready; Shell and IOPub receive inside two seconds | **I.** The supposed barrier is on heartbeat, not PUB/SUB, at `zeromq_native_test_support.rs:78-113`; receives are at `signed.rs:70-133`. | No failure supplied; skipped here |
| `zeromq/tests/native/signed.rs:137-208` tampered request | two 50 ms sleeps establish readiness; error appears inside two seconds; no reply within 200 ms | **I.** Fixed sleeps and negative elapsed-time proof are at `signed.rs:157-203`. Once `pump_once` returns `InvalidSignature`, the semantic no-dispatch property can be asserted structurally. | Reported failing on director host; skipped here |
| `zeromq/tests/native/frontend/execute.rs:4-171` | heartbeat pseudo-barrier establishes subscription; exactly four target IOPub frames arrive, each inside two seconds | **M/I.** Four target frames are structurally promised by `adapter/iopub.rs:4-17` and `adapter/mod.rs:481-500`; subscription readiness is not. Sites are `execute.rs:25-43,69-134`. | No failure supplied; skipped here |
| `zeromq/tests/native/frontend/stdin.rs:4-210` | pseudo-barrier establishes subscription; stdin, Shell and exactly four IOPub frames meet deadlines | **M/I**, at `stdin.rs:40-58,86-188`. Semantic stdin routing is valid; readiness and duration are not. | Reported failing on director host; skipped here |
| `zeromq/tests/native/frontend/session_flow.rs:4-444` | pseudo-barrier remains valid across three executes; each stage and each four-frame group meets two-second waits | **M/I**, at `session_flow.rs:49-67,93-146,184-287,323-423`. Counts and session state are valid coverage; readiness/deadlines are not. | Reported failing on director host; skipped here |
| `zeromq/tests/native/frontend/control.rs:4-82` interrupt | 50 ms makes Control connection ready; event and reply arrive inside two seconds | **I**, at `control.rs:17-25,51-62`. No subscription is involved. | No failure supplied; skipped here |
| `zeromq/tests/native/frontend/control.rs:85-227` shutdown | 50 ms establishes both connections; after both queues are pending, Control wins; reply under two seconds; no Shell reply for 200 ms | **M.** Control priority is genuinely promised and explicitly preconditioned at `control.rs:163-171` and `socket.rs:190-229`; fixed/negative waits at `control.rs:114-115,187-222` are invalid timing mechanisms. | No failure supplied; skipped here |

## 3. Per-test verdicts and coverage

The tables contain the per-test verdicts. For the recorded failures specifically:

- `tests/pump.rs:4-53` is an invalid immediate-readiness assertion, not a kernel defect. Removing the test would still be wrong: once synchronized, it covers Shell routing-ID preservation and IOPub routing separation at `pump.rs:41-51`.

- `tests/pump.rs:175-343` previously exposed a real request-scoped cancellation defect, now closed as recorded at `TODO.md:701-706`. Its current Control priority and deferred-Shell assertions are valuable, but `pump.rs:224-229,248-258` still rely on an unconstructed “worker is still running” state and an arbitrary duration.

- Native `basic` execute and heartbeat are not subscription failures. Their source contains Dealer/Router and Req/Rep only at `basic.rs:8-22,89-102`. Their tests are wrong to turn two seconds into a semantic requirement, although corrected versions would catch real native bind, framing, worker, routing, and heartbeat defects.

- Native tampered-signature rejection has valuable end-to-end authentication coverage at `signed.rs:161-200`, but its 50 ms staging and 200 ms proof-of-absence are invalid.

- Native stdin and session-flow have genuine routing, resume, execution-count, and shutdown coverage at `frontend/stdin.rs:86-209` and `frontend/session_flow.rs:93-444`. Their IOPub readiness and elapsed-time mechanisms are invalid; deleting either test would lose real coverage.

The pump and native failures therefore do not share one implementation root:

- Pump finding A is a parent/worker scheduling race around `JoinHandle::is_finished()` at `adapter/worker.rs:71-82`.
- Native IOPub has a separate subscription-processing race.
- Native `basic` has a third, generic socket-worker/deadline family. Source excludes subscription but cannot identify which host-side stage was delayed or stalled without the actual failing output.

### Pinned zeromq mechanism

`Cargo.lock:1222-1226` pins `zeromq` 0.5.0. In the locally cached pinned source:

- `SubSocket::subscribe` stores the filter and queues a subscription message to current peers, then returns: `zeromq-0.5.0/src/sub.rs:131-159`.
- The publisher processes that message later in an independently spawned coroutine: `zeromq-0.5.0/src/pub.rs:34-72,93-135`.
- PUB sends only through subscription filters already present at send time: `zeromq-0.5.0/src/pub.rs:158-200`.

Thus `subscribe().await` is not publisher acknowledgement. The repository’s `acknowledge_iopub_subscription` sends a heartbeat over an independent Req/Rep socket at `zeromq_native_test_support.rs:78-113`; no source edge orders that heartbeat after the PUB worker’s subscription coroutine. This conclusion is source-derived.

### Current finding-A observation

Command:

```text
scripts/run-bounded.py --timeout 120 -- cargo test -p rum-jupyter --offline --lib tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately -- --exact --nocapture
```

Verbatim output:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-0d2044aa0c254386)

running 1 test

thread 'tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately' panicked at crates/rum-jupyter/src/tests/pump.rs:37:5:
assertion failed: pump.pump_execute_completion(&mut transport).expect("completion poll should succeed")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately ... FAILED

failures:

failures:
    tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 102 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-jupyter --lib`
```

## 4. Finding-A inversion mechanism

The sequence is:

1. `pump_once` dispatches execute, moves the session into a newly spawned worker, and immediately returns the prologue at `adapter/mod.rs:386-407`.
2. The test immediately calls `pump_execute_completion` at `tests/pump.rs:31-40`.
3. That call reaches `JoinHandle::is_finished()` and returns false if the child has not yet run to completion at `transport/event_loop.rs:94-109` and `adapter/worker.rs:71-82`.
4. There is no handshake or other happens-before edge between `thread::spawn` at `adapter/worker.rs:168-179` and this first sample.

Default concurrent whole-suite execution supplies scheduling/preemption opportunities between spawn and poll. Serialized isolation commonly lets the parent test thread continue directly to `is_finished`. The prior whole-binary/default-versus-serialized observation is recorded at `docs/dev/codex/brief-397-jupyter-pump-execute-lifecycle.md:263-271`.

A scratch probe varied only what happened between dispatch and the first completion poll. Its program stdout was:

```text
mode=cold completion_ready=false
mode=cold completion_ready=false
mode=cold completion_ready=false
mode=cold completion_ready=false
mode=cold completion_ready=false
mode=warm completion_ready=false
mode=warm completion_ready=false
mode=warm completion_ready=false
mode=warm completion_ready=false
mode=warm completion_ready=false
mode=worker-warm completion_ready=false
mode=worker-warm completion_ready=false
mode=worker-warm completion_ready=false
mode=worker-warm completion_ready=false
mode=worker-warm completion_ready=false
mode=yield completion_ready=false
mode=yield completion_ready=false
mode=yield completion_ready=false
mode=yield completion_ready=false
mode=yield completion_ready=false
mode=sleep completion_ready=true
mode=sleep completion_ready=true
mode=sleep completion_ready=true
mode=sleep completion_ready=true
mode=sleep completion_ready=true
```

This supports scheduling delay, not global process/cache warm-up, as the inversion mechanism. The scratch crate was removed afterward.

## 5. Draft implementation briefs

### Draft A — deterministic in-process execute-worker synchronization

#### Goal

Make every in-process `rum-jupyter` worker test establish worker state by an instance-scoped event or gate. Remove immediate-completion assumptions, fixed sender sleeps, and elapsed-time assertions as semantic conditions while preserving routing, priority, cancellation, stdin, and IOPub coverage.

#### Context pointers

- Worker spawn/sample race: `adapter/worker.rs:71-82,168-187`.
- Invalid finding-A assertion: `tests/pump.rs:31-40`.
- Busy-state assumption and fixed cancellation loop: `tests/pump.rs:198-258`.
- Fixed shutdown staging: `tests/bootstrap.rs:442-541`.
- Shared deadline helper and users: `tests/mod.rs:27,66-87`.

#### Scope

- Files MAY modify: `adapter/worker.rs`, `adapter/mod.rs`, `tests/mod.rs`, `tests/pump.rs`, `tests/bootstrap.rs`, `tests/adapter/dispatch.rs`, `tests/wire.rs`, `tests/stdin.rs`, `zeromq_fake_tests.rs`.
- MUST NOT modify runtime or host semantics, public serde shapes, docs, `TODO.md`, `Cargo.lock`, or `.git`.
- Any worker hook must be `#[cfg(test)]`, instance-scoped, and impose no production/default-path cost.

#### Deliverable

- A test-only worker-start/worker-finished notification and optional release gate.
- Finding A waits for the actual finished notification before testing pump routing.
- Busy Shell deferral is tested while a gate provably holds the worker running.
- Shutdown is queued synchronously after the running-state handshake; remove the 2 ms sleep and one-second assertion.
- Deadlines, if retained, are outer stop-loss only, derived from measured debug-test data and reported as harness failure—not language latency.
- No semantic assertion or test deleted.

#### rum / Rust pitfalls

- No global gate shared across parallel tests.
- Do not join or consume the worker outside normal adapter completion.
- Preserve cancellation and suspend/resume behavior.
- No `unsafe`, production allocation, mutex, or branch.

#### Verification

```text
scripts/run-bounded.py --timeout 120 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 180 -- cargo --offline test -p rum-jupyter tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately -- --exact
scripts/run-bounded.py --timeout 180 -- cargo --offline test -p rum-jupyter tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle -- --exact
scripts/run-bounded.py --timeout 180 -- cargo --offline test -p rum-jupyter tests::bootstrap::socket_driver_observes_shutdown_promptly_during_running_execute -- --exact
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter
```

#### Accept vs stop

Accept only if each precondition is event-established and the focused plus crate gate are green. Stop if synchronization requires changing production evaluation semantics/default cost, or if the same event-established failure repeats twice.

#### Hard rules / report

No network, no vendor oracle, no unsafe, no test deletion, no commit. Report per-file changes, verbatim gates, the causal synchronization used, and any stop-loss reached.

---

### Draft B — real native IOPub subscription barrier

#### Goal

Replace the heartbeat pseudo-barrier with a same-PUB/SUB-path readiness proof, then collect target IOPub traffic by parent/request identity and terminal Idle rather than assuming the next four socket frames belong to the test.

#### Context pointers

- Current false barrier: `zeromq_native_test_support.rs:78-113`.
- Four consumers: `native/signed.rs:25-43`, `frontend/execute.rs:25-43`, `frontend/stdin.rs:40-58`, `frontend/session_flow.rs:49-67`.
- Structural target sequence: `adapter/iopub.rs:4-17`, `adapter/mod.rs:481-500`.
- Pinned behavior: `Cargo.lock:1222-1226`; cached `zeromq-0.5.0/src/sub.rs:131-159`, `pub.rs:34-72,93-200`.

#### Scope

- Files MAY modify: `zeromq_native_test_support.rs`, `zeromq/tests/native/signed.rs`, `zeromq/tests/native/frontend/execute.rs`, `stdin.rs`, `session_flow.rs`.
- Do not change the production PUB socket type, protocol messages, dependency version, docs, or `Cargo.lock`.

#### Deliverable

- A unique test-only IOPub sentinel sent through the backend’s real PUB path and observed by the test SUB socket before the real request.
- Sentinel traffic identified and excluded by unique parent/message ID.
- Target collectors consume until the target parent’s Idle message, then assert that parent’s exact semantic sequence/count.
- Heartbeat remains independently tested, but is no longer called subscription acknowledgement.
- Stage-specific failure messages distinguish connection, subscription barrier, request, completion, Shell, and IOPub.

#### rum / Rust pitfalls

- Avoid “quiet for N ms” as readiness.
- Prevent delayed duplicate sentinels from contaminating target counts.
- Do not replace PUB with XPUB without a separate design decision.
- Preserve signatures on signed probe messages.

#### Verification

Run the crate gate in the sandbox and classify skips honestly. Authoritative acceptance must run on the director host:

```text
scripts/run-bounded.py --timeout 120 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages -- --exact --nocapture
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata -- --exact --nocapture
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply -- --exact --nocapture
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown -- --exact --nocapture
```

#### Accept vs stop

Accept only when output proves the native tests actually ran and each barrier was observed on SUB before the real request. Stop on sandbox skip, if the barrier reaches stop-loss twice, or if repair requires production socket/protocol changes.

#### Hard rules / report

No duration increase as a substitute, no deleted count/order assertions, no network/dependency update, no commit. Paste actual host run/skip output and identify the observed barrier message.

---

### Draft C — non-IOPub native socket progress and negative assertions

#### Goal

Repair native Dealer/Router, Req/Rep, Control, and signature tests so connection and message conditions—not fixed sleeps or short elapsed-time windows—drive their assertions. Preserve a separately derived stop-loss only to prevent hangs.

#### Context pointers

- Basic tests: `native/basic.rs:4-123`.
- Fixed sleeps and negative waits: `native/signed.rs:137-208`, `frontend/control.rs:4-82,85-227`.
- Common two-second loops: `zeromq_native_test_support.rs:115-174`.
- Native worker threads/queues: `zeromq_native/threading.rs:12-25`, `handles.rs:35-112,203-280`.
- Polling is nonblocking: `zeromq.rs:158-197`, `socket.rs:280-318`.

#### Scope

- Files MAY modify: `zeromq_native_test_support.rs`, `native/basic.rs`, `native/signed.rs`, `native/frontend/control.rs`; other native files only for common helper call-site migration.
- Do not change kernel evaluation, wire protocol, dependency version, or documented response ordering.

#### Deliverable

- Remove the 50 ms connection sleeps.
- Replace generic polling helpers with stage-specific condition waits and diagnostics.
- Derive a global test stop-loss from actual director-host stage measurements; do not treat it as a latency contract.
- After `InvalidSignature`, assert structurally that dispatch/reply was not produced rather than waiting 200 ms.
- After shutdown, assert the Shell request remains pending and no dispatch occurred rather than using a 200 ms absence window.
- Keep the explicit both-channels-pending precondition in the Control-priority test.

#### Verification

```text
scripts/run-bounded.py --timeout 120 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter zeromq::tests::native::basic -- --nocapture
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end -- --exact --nocapture
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter zeromq::tests::native::frontend::control -- --nocapture
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter
```

#### Accept vs stop

Accept only with proof that the native tests actually ran and every stage reached its condition. If a condition-established test still fails twice, stop and report a likely native backend defect; do not lengthen another sleep/deadline.

#### Hard rules / report

Offline only, no vendor oracle, no semantic weakening, no unsafe, no commit. Report measured stage data, chosen stop-loss derivation, verbatim output, and exact failure stage.

---

### Draft D — eliminate native ephemeral-port TOCTOU

#### Goal

Have native ZeroMQ bind port zero itself and propagate the resolved endpoints back to the test frontend, eliminating the reserve/drop/rebind interval.

#### Context pointers

- Current reserve/drop gap: `zeromq_native_test_support.rs:10-35`.
- `ZeroMqSocket::bind` currently erases the resolved endpoint: `zeromq.rs:28-33,410-427`.
- Native worker further maps the pinned library’s bind result to unit: `zeromq_native/workers.rs:249-261`.
- Pinned `Socket::bind` returns the exact resolved endpoint: cached `zeromq-0.5.0/src/lib.rs:227-243`; port-zero support exists at `zeromq-0.5.0/src/util.rs:265-283`.

#### Scope

- Files MAY modify: `zeromq.rs`, `zeromq_native/commands.rs`, `handles.rs`, `socket.rs`, `workers.rs`, `zeromq_native_test_support.rs`, `zeromq_fake_tests.rs`, and native test endpoint setup.
- No public protocol/serde break, dependency change, `unsafe`, docs, or `Cargo.lock`.

#### Deliverable

- Propagate resolved bind endpoints through the internal lifecycle reply rather than discarding them.
- Native test configuration requests port zero for each socket.
- Backend layout/session configuration is updated to the resolved ports before frontend sockets connect.
- Retain a capability probe only to classify PermissionDenied; do not use its port as a reservation.
- Fake-backend tests cover resolved-endpoint propagation.
- Remove reliance on five dropped `TcpListener` reservations.

#### Verification

```text
scripts/run-bounded.py --timeout 120 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 180 -- cargo --offline test -p rum-jupyter zeromq_fake_tests
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter zeromq::tests::native -- --nocapture
scripts/run-bounded.py --timeout 300 -- cargo --offline test -p rum-jupyter
```

#### Accept vs stop

Accept only when frontend endpoints come from the sockets’ completed bind results and no release-before-bind interval remains. Stop if this requires raw-fd transfer, `unsafe`, a public compatibility break, or if native execution cannot be demonstrated on the authoritative host.

#### Hard rules / report

Safe Rust only, offline, no dependency update, no commit. Report resolved endpoints without claiming native success from an internal skip.

## 6. What `TODO.md:707-724` gets right and misses

The Jupyter item is actually `TODO.md:708-722`; line 724 begins the next, unrelated item.

It gets these points right:

- Native tests can silently early-return in this sandbox, and such output must not be treated as stability evidence (`TODO.md:714-722`). The capability run confirmed the current sandbox takes that path.
- The pinned 0.5.0 subscription mechanism is correctly identified in essence (`TODO.md:710-713`): publisher processing lags `subscribe().await`, so early PUB frames can be dropped. The source evidence is `sub.rs:131-159` and `pub.rs:34-72,93-200`.

It is stale or incomplete in these ways:

- The four cited files no longer use a fixed 50 ms IOPub delay. They now call `acknowledge_iopub_subscription`, e.g. `frontend/execute.rs:25-43`; that helper is still invalid because it exercises heartbeat at `zeromq_native_test_support.rs:78-113`, not subscription acknowledgement.
- PUB/SUB does not explain either `basic` test: their setup contains no IOPub endpoint or `SubSocket` at `native/basic.rs:8-22,89-102`.
- The native class is larger than four IOPub tests. Every native test uses arbitrary two-second progress/receive bounds through `zeromq_native_test_support.rs:115-174` or local `timeout` calls; signed/control negative tests add 50/200 ms assumptions.
- It misses the port reserve/drop/rebind race at `zeromq_native_test_support.rs:10-35`.
- “The receive loops time out” must remain a source-derived hypothesis in this environment, not an observed result.
- It does not cover the separate in-process immediate-poll and worker-progress class at `tests/pump.rs:31-40`, `tests/mod.rs:66-75`, and the unbounded `pump_until_idle` users.

## 7. Stop-loss, risks, and open questions

- Native execution was unavailable, so I stopped at source-derived diagnosis. The exact failing native stage on the director host remains open; repaired tests must distinguish delayed progress from an actual backend stall.
- Finding A was directly reproduced and its assertion is conclusively invalid on the current tree.
- The scratch scheduling probe used an external temporary crate and the system toolchain rather than the workspace-pinned toolchain. It was used only for the Boolean interleaving result, not performance data.
- A same-channel sentinel barrier must prevent late probe traffic from contaminating exact target counts.
- Dynamic port propagation is broader than a test-only edit; Draft D stops if it cannot remain internal and safe.
- Simply increasing two-second or 50/200 ms values would not repair causality and should be rejected.
- Diagnostic only: no repository file changed. Final `git status --short` and `git diff --stat` produced no output. `/tmp/rum-brief406-probe` was removed.
