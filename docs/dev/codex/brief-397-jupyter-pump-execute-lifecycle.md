# brief-397 — design: two reproducible `rum-jupyter` pump failures

**Scope: NONE — diagnosis and plan only, no source change.** Reading, grepping
and throwaway probes are in scope. Any temporary edit you make to instrument a
probe must be reverted before you report, and named in the report; leave the
working tree exactly as you found it apart from this file's report.

**Write your findings section first, before running any verification commands.**

Two tests in `crates/rum-jupyter/src/tests/pump.rs` fail reproducibly. I have a
confident mechanism for one of them and none for the other, and the second is
the reason for this brief: it may be a real defect in the interrupt/cancel
path rather than a test that asserts too much. **The background below is mine
and may be wrong — two of the last four review briefs returned corrections to
my background as their most valuable finding. Do that again if it is wrong
here.**

## Environment note

This is a new development host: Linux, 32 cores, `rustc 1.85.0` from
`rust-toolchain.toml`. The project's standard gate is green here —
`cargo test --workspace` is 2238 passed / 0 failed / 7 ignored. The failures
below appear in `--release`, and in `--lib` runs of either profile when fewer
tests share the process. Whether the same behavior exists on the previous macOS
host is unknown; do not assume either way.

These are `MockSocketTransport` tests and need no loopback socket. Separately,
and cheaply: **report whether `zeromq::tests::native::*` ran or skipped in your
sandbox** (`cargo test -p rum-jupyter --release zeromq::tests::native`, one
line of result). An earlier review could not settle which environments permit
loopback binding, and your result either way is worth recording. It is not part
of this brief's subject.

## Measurements (mine, reproduce or refute)

`cargo test --workspace --release --no-fail-fast`: **2235 passed / 2 failed /
7 ignored**, both failures in `tests::pump`. Failure rates from repetition runs
of the compiled test binary:

| condition | `event_pump_preserves_routing_ids…` | `control_interrupt_cancels…` |
| --- | --- | --- |
| release, single test (`--exact`) | 30/30 failed | 30/30 failed |
| release, single test, `--test-threads=1` | 30/30 failed | 30/30 failed |
| release, `tests::pump` filter only | 20/20 failed (module) | — |
| release, whole binary, 10 runs | 6/10 failed | 2/10 failed |
| debug, single test | 20/20 failed | 18/20 failed |
| debug, whole binary, 10 runs | 1/10 failed | 0/10 failed |

**The direction is the surprising part: the more tests share the process, the
more often these pass.** Whatever the dependency is, it is not the amount of
time the test is willing to wait — see B below.

## A — `event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately`

Fails at `crates/rum-jupyter/src/tests/pump.rs:37`:

```
assertion failed: pump.pump_execute_completion(&mut transport).expect("completion poll should succeed")
```

My reading of the mechanism: `KernelEventPump::pump_execute_completion`
(`crates/rum-jupyter/src/transport/event_loop.rs:94`) returns `Ok(false)` when
`KernelAdapter::poll_execute_completion` (`crates/rum-jupyter/src/adapter/mod.rs:301`)
returns `None`, which happens when `ExecuteState::take_finished`
(`crates/rum-jupyter/src/adapter/worker.rs:71`) sees `!worker.handle.is_finished()`.
The execute runs on a worker thread spawned by `ExecuteWorker::spawn_eval`
(`worker.rs:87`). The test calls `pump_once` and then, in the next statement,
asserts the completion poll returns `true` — i.e. it requires a
just-spawned thread to have already finished evaluating `1`. If that reading is
right, the assertion is a timing property and the implementation promises a
non-blocking poll, which is the same shape as the three items briefs 384-388
closed.

Note that `crates/rum-jupyter/src/tests/mod.rs:66` already has
`wait_for_execute_completion`, which polls `poll_execute_completion` to a
deadline. The pump-level tests have no equivalent.

**What I want from you on A:**

1. Confirm or refute the mechanism from the code, not from the failure rate.
2. **Sweep the class, not the two instances.** brief-388 repaired the two
   assertions that had been *seen* failing and left five more resting on the
   same mechanism; that gate miss is recorded in `TODO.md`. Inventory every
   test that asserts on a completion poll, a pump return value, or a transport
   queue length **without a deadline**, across `crates/rum-jupyter/src/tests/`
   and the `zeromq` test modules. Report the full list with `path:line`,
   including the ones you judge safe and why.
3. State the intended contract of `pump_execute_completion`'s `bool` in one
   sentence, sourced from the code and any rustdoc, and say whether the tests
   currently treat it as that contract.

## B — `control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle`

Fails at `crates/rum-jupyter/src/tests/pump.rs:250`:

```
interrupted execute did not produce a shell reply within 2s
```

The test starts `Do[x = 1, 5000000000]` on a session whose
`iteration_work_limit` is `5_000_000_000` (`tests/mod.rs:58`), queues a second
shell request, queues an `interrupt_request` on Control, then loops calling
`pump_next` with a 1 ms sleep until `transport.shell` is non-empty or
`ASYNC_TEST_TIMEOUT` (2 s, `tests/mod.rs:27`) expires.

**This is not a "waited too little" failure.** I raised `ASYNC_TEST_TIMEOUT` to
60 s, rebuilt, and ran the test alone in release: it still produced no shell
reply, failing after the full 60 s. The constant is restored; the tree is
clean.

So either the interrupt does not reach the running evaluation, or the
cancellation completes but the completion never becomes a shell reply, or the
test's own pump loop fails to drive the state machine in this configuration.
**Which of those it is decides the severity**: if an `interrupt_request` can
fail to cancel a running execute, that is a defect in the host/kernel
interrupt contract, and the test is the messenger rather than the culprit.

**What I want from you on B:**

1. Determine where the chain breaks, with evidence. The chain to walk is
   roughly: `pump_next` → control dispatch → `KernelRequest::Interrupt`
   handling in `crates/rum-jupyter/src/adapter/` → the worker's `cancel` handle
   (`worker.rs:88`) → the kernel's cancellation check inside the `Do` driver →
   `handle.is_finished()` → `poll_execute_completion` → shell reply. Say which
   link does not fire, and cite it.
2. Explain the direction that puzzles me: **why does running more tests in the
   same process make it pass?** A hypothesis is only useful here if you can
   make it predictive — if you believe it is scheduling, thread warm-up,
   `available_parallelism`, or CPU contention, construct a probe that flips the
   outcome deliberately in both directions, the way brief-387 made the Bloom
   flake deterministic by preinterning a counted number of symbols. A mechanism
   you cannot switch on and off is a guess.
3. Answer explicitly: **is the interrupt actually delivered to the evaluation,
   and does the evaluation observe it?** If the cancellation is genuinely not
   observed under some condition, say so plainly and treat it as a host-contract
   finding, not a test finding. Check whether `rum-host`'s own cancel tests
   cover the same shape (`crates/rum-host/tests/`), and whether they would catch
   this.
4. If the fault is in the test rather than the implementation, say what the
   test should assert instead so that it holds **by construction** rather than
   by timing — the standard the last four repairs were held to.

## Deliverable

- A findings section: for A and B each, the mechanism with `path:line`
  evidence, and your own reproduction numbers (state the exact commands and
  counts; do not reuse mine).
- The class inventory asked for in A2.
- A step-by-step plan: one step per independently gateable change, each naming
  the files and functions it touches, the test that gates it, and what it must
  not break.
- **Drafts of the follow-up implementation briefs**, in `TEMPLATE.md` shape,
  one per step. These are raw material for my own briefs, not dispatched as
  written — but draft them fully, not as step descriptions.
- Risks and open questions, including anything in my background above you found
  to be wrong.

## Reproduction commands

```
cargo test --workspace --release --no-fail-fast
cargo test -p rum-jupyter --release --lib tests::pump
# single test, compiled binary (substitute the current hash):
ls -t target/release/deps/rum_jupyter-* | grep -v '\.d$' | head -1
<binary> --exact tests::pump::<name>
<binary> --exact tests::pump::<name> --test-threads=1
```

## Probe budget and stop-loss

- Repetition runs: cap at 30 per condition and state the cap. These binaries
  run in seconds; do not spend the budget re-measuring what already reproduced.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything that
  may hang — B's failure mode is a test that waits out its own deadline.
- **If B does not yield a mechanism after two serious attempts, stop and report
  what you ruled out.** A precise negative result plus the class inventory from
  A is a good outcome for this brief; a speculative root cause is not.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- **No source change.** Temporary instrumentation is allowed only if reverted
  before reporting and named in the report; verify with `git status` and paste
  it.
- Do not modify `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` outside this
  file, or anything under `docs/dev/codex/` except appending your report.
- Leave no probe files in the repository.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no differential testing against other vendors' engines.
- Paste verbatim output for every claim. If a run did not produce the evidence,
  say so — never fake it.

## Report format

1. Findings for A and B, each with mechanism, `path:line` evidence, and
   reproduction counts.
2. The A2 class inventory.
3. The plan, step by step.
4. The drafted implementation briefs.
5. Stop-loss and open questions, including corrections to my background.
6. One line: whether `zeromq::tests::native::*` ran or skipped in your sandbox.

---

## Codex report

### 1. Findings

#### A — the immediate completion assertion asks a non-blocking poll to wait

The background's mechanism for A is correct.  Handling the shell request calls
`KernelAdapter::dispatch_request` and `dispatch_execute_request`, which marks the
adapter busy and moves the session into a newly spawned worker
(`crates/rum-jupyter/src/adapter/mod.rs:321-326,386-407`;
`crates/rum-jupyter/src/adapter/worker.rs:56-59,87-95,168-179`).  The first
`pump_once` only handles that queued request and sends its two prologue IOPub
messages (`crates/rum-jupyter/src/transport/event_loop.rs:111-138`).  The next
statement in the test performs exactly one completion poll
(`crates/rum-jupyter/src/tests/pump.rs:31-42`).  That poll returns `Ok(false)`
whenever `poll_execute_completion` returns `None`; in turn,
`poll_execute_completion` returns `None` whenever `take_finished` sees that the
new worker's `JoinHandle` is not yet finished
(`crates/rum-jupyter/src/transport/event_loop.rs:94-109`;
`crates/rum-jupyter/src/adapter/mod.rs:301-319`;
`crates/rum-jupyter/src/adapter/worker.rs:71-83`).  No code edge orders worker
completion before that assertion.

The one-sentence contract of `pump_execute_completion`'s `bool` is: **`true`
means that this non-blocking call found one finished execute worker and sent
its dispatch, while `false` means that it found no finished worker at that
instant** (`event_loop.rs:94-109`; there is no additional method rustdoc).
The A test currently treats `true` as “the worker spawned by the immediately
preceding pump must already be finished,” which is not that contract.

I wrote the findings above before running cargo, then reproduced A on the
unmodified release binary.  The module command was:

```
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-jupyter --release --lib tests::pump -- --nocapture
```

Its verbatim result and failure were:

```
assertion failed: pump.pump_execute_completion(&mut transport).expect("completion poll should succeed")
test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 98 filtered out; finished in 2.00s
```

I then invoked
`target/release/deps/rum_jupyter-c478cc3da5746e61 --exact
tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately`
ten times (cap 10), and the same command with `--test-threads=1` five times
(cap 5).  The loop summaries were:

```
condition=release exact test=tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately cap=10
result pass=0 fail=10 total=10
condition=release exact test-threads=1 test=tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately cap=5
result pass=0 fail=5 total=5
```

With the whole release test binary sharing the process, capped at 10 runs, A
failed 6/10; with that whole binary serialized by `--test-threads=1`, capped at
5, it failed 5/5:

```
condition=release whole-binary cap=10
A_result pass=4 fail=6 total=10
condition=release whole-binary test-threads=1 cap=5
A_result pass=0 fail=5 total=5
```

#### B — Control delivers the interrupt, but a startup race can erase it

The broken link is between delivery to the worker's cancellation flag and the
evaluation observing that flag.  The event pump checks completion first, then
while execute is running services Control (`event_loop.rs:140-174`).  Control
dispatch reaches `KernelAdapter::interrupt`, sets `InterruptRequested`, and
calls `ExecuteState::cancel` (`adapter/mod.rs:154-161,321-363`).  That marks the
worker interrupted and calls its `HostCancelHandle` (`adapter/worker.rs:47-53`),
whose documented implementation stores `true` in the session's shared atomic
flag (`crates/rum-host/src/session.rs:113-126,440-447`).

The race is that `ExecuteWorker::spawn_eval` obtains that handle before it
spawns the thread (`adapter/worker.rs:87-95`), while the spawned thread only
later enters `HostSession::eval` (`adapter/worker.rs:168-186`;
`crates/rum-host/src/session.rs:318-332,398-402`).  At the start of that call,
`Session::eval_request` invokes `begin_top_level_interrupt_budget`
(`crates/rum-runtime/src/runtime/session/session_requests.rs:108-119`), which
unconditionally stores `false` into the exact shared interrupt flag
(`crates/rum-runtime/src/runtime/session/session_helpers.rs:31-39`).  Therefore:

1. if the worker reaches the evaluation prologue first, a later Control cancel
   remains set and the cooperative poll points can observe it;
2. if Control cancel stores `true` first, the worker's later evaluation
   prologue overwrites it with `false`, and the five-billion-iteration `Do`
   continues with no second interrupt.

`Do` itself repeatedly calls `self.evaluate(body)` but has no unconditional
outer-loop interrupt check (`crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:105-147`).
Its body still reaches the evaluator's amortized poll path when interruption is
armed (`crates/rum-runtime/src/runtime/eval_core/mod.rs:13-24,82-98`), so the
absence of a `Do`-driver poll is not what loses this request: after the startup
clear there is simply no `true` flag left to observe.  Consequently the worker
does not finish, `JoinHandle::is_finished()` remains false, completion remains
`None`, and no shell reply is encoded (`adapter/worker.rs:71-83`;
`adapter/mod.rs:301-319`; `event_loop.rs:94-109`).

This also explains the counterintuitive suite-size direction without invoking
a longer effective timeout.  In a single-test process the already-running test
thread commonly continues from `thread::spawn` through construction and
dispatch of Control before the new worker is scheduled.  Concurrent harness
work creates more opportunities to pre-empt that parent thread and let the
worker cross `begin_top_level_interrupt_budget` first.  The measurements
separate that scheduling effect from mere process warm-up: B passed 7/10 in the
whole binary at default test concurrency, but only 1/5 with the whole binary
serialized.  A warm process alone can move the race (the one serial pass), but
concurrency moved it much more strongly in this sample.  The bounded probe
below deliberately delays each side and flips the result in both directions.

This is a Jupyter host-boundary contract finding, not a test-only timing
assertion: after the adapter has accepted an execute and reports it running, an
interrupt can be acknowledged yet never terminate that logical execute.  It is
not a contradiction in `rum-host`'s standalone contract.  The host rustdoc says
a handle requests
cancellation of the current evaluation and may be used from another thread
(`crates/rum-host/src/session.rs:113-126,440-447`), but its existing contract
tests deliberately wait for `Print["evaluation-started"]` before cancelling
(`crates/rum-host/tests/lifecycle_contract.rs:120-161` and
`crates/rum-host/tests/stress.rs:54-97`).  The separate before/after test
explicitly requires a cancel issued before evaluation to be a no-op for a
later evaluation (`lifecycle_contract.rs:107-117`).  Those tests cover the two
non-racing endpoints and cannot catch an adapter that has logically accepted
an execute but whose worker has not crossed the host's evaluation-start
boundary.

The answer to the brief's explicit delivery question is therefore: **Control
does deliver the interrupt to the worker's shared cancellation flag; in the
failing ordering the evaluation does not observe it, because its own startup
clears it first.**  In the opposite ordering the ordinary evaluator/`Do` poll
path does observe it.  The host tests would not catch the failing ordering:
the lifecycle, stress, and crate-local tests synchronize on an
evaluation-start print (`crates/rum-host/tests/lifecycle_contract.rs:120-161`;
`crates/rum-host/tests/stress.rs:54-97`;
`crates/rum-host/src/tests/eval_contract.rs:212-268`), and the runtime file
request test synchronizes on a test builtin reaching dispatch
(`crates/rum-runtime/src/runtime/tests/session_requests/api.rs:512-544`).

On the unmodified release binary, B failed 10/10 alone and 5/5 alone with
`--test-threads=1`; it failed 3/10 in the whole concurrent binary and 4/5 in
the whole serialized binary.  These are the verbatim loop summaries:

```
condition=release exact test=tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle cap=10
result pass=0 fail=10 total=10
condition=release exact test-threads=1 test=tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle cap=5
result pass=0 fail=5 total=5
condition=release whole-binary cap=10
B_result pass=7 fail=3 total=10
condition=release whole-binary test-threads=1 cap=5
B_result pass=1 fail=4 total=5
```

The one serious probe temporarily added a 100 ms delay immediately before
`HostSession::eval`, a trace at `HostCancelHandle::cancel`, a trace immediately
before the top-level flag clear, and a trace when the evaluator observed the
flag.  With immediate Control, the direct trace was:

```
brief397 HostCancelHandle::cancel stores true
brief397 worker delaying before HostSession::eval
brief397 begin_top_level_interrupt_budget clears prior=true
interrupted execute did not produce a shell reply within 2s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 102 filtered out; finished in 2.00s
```

That forced ordering failed 5/5.  Adding a temporary 250 ms delay on the
parent before it queued Control forced the other ordering:

```
brief397 parent delaying before Control interrupt
brief397 worker delaying before HostSession::eval
brief397 begin_top_level_interrupt_budget clears prior=false
brief397 HostCancelHandle::cancel stores true
brief397 poll_host_interrupt_if_due observes true
real_interrupt_latency_ms=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 102 filtered out; finished in 0.35s
```

That ordering passed 5/5:

```
probe=worker-delay-100ms_parent-delay-0ms cap=5
result pass=0 fail=5 total=5
probe=worker-delay-100ms_parent-delay-250ms cap=5
result pass=5 fail=0 total=5
```

All four instrumentation sites and the parent delay were removed immediately
after the probe; the final status audit below confirms that none remains.

### 2. A2 class inventory

I treated `is_empty()`/`pop_*().expect(...)` as queue-length assertions, not
only literal `.len()`, and included both `KernelEventPump` and
`KernelSocketDriver` pump returns.  `KernelDispatchResult.iopub/stdin` vectors
are dispatch payloads rather than transport queues; where they are completed
through the deadline helper I mention them under the deadline-covered group.

The no-deadline sites are:

| Test and sites | Judgment |
| --- | --- |
| `tests/pump.rs:31-42`, `event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately` | **Unsafe and failing.** Lines 31-36 assert synchronous request/prologue effects and are safe; lines 37-42 assume the newly spawned worker is finished on the next non-blocking completion poll. |
| `tests/pump.rs:95-103`, `pump_next_prioritizes_control_messages_before_shell_messages` | **Safe.** No worker exists before the call, both in-memory queues are populated synchronously, and `pump_next`'s channel list puts Control before Shell (`event_loop.rs:152-170`). |
| `tests/pump.rs:157-171`, `pump_next_prioritizes_stdin_replies_while_kernel_is_waiting_for_input` | **Safe.** Pending input is staged directly, no worker is running, and both incoming queues exist before the single pump; the asserted Stdin-before-Shell rule is structural. |
| `tests/pump.rs:198-229`, the start/deferral portion of `control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle` | **Mixed.** Lines 198-206 assert synchronous shell dispatch/prologue and are safe.  Lines 224-229 are a second timing assumption: because `pump_next` polls completion first (`event_loop.rs:140-145`), `.is_none()` relies on the five-billion-iteration worker not having finished yet.  It is overwhelmingly likely but not established by construction. |
| `tests/pump.rs:311-317`, the follow-up portion of the same test | **Completion-safe but unbounded.** `pump_until_idle` waits while `execute_running`, so the queue/report assertions do not require an immediate completion; a broken worker can hang the call because the production helper has no deadline (`event_loop.rs:177-211`). |
| `tests/pump.rs:385-395`, `pump_until_idle_stops_after_shutdown_and_leaves_later_shell_work_queued` | **Safe.** Control and Shell are queued before entry; Control is selected synchronously, shutdown stops the loop before Shell can spawn a worker, and all counts follow from that state. |
| `tests/stdin.rs:150-166,183-209`, `input_reply_resumes_execute_and_preserves_shell_routing_ids` | **Immediate portions safe; completion portions unbounded.** The two `pump_next` calls consume already queued Shell/Stdin messages.  Each `pump_until_idle` waits for the worker rather than assuming it is finished, but neither call has a test deadline. |
| `tests/wire.rs:284-292`, `mock_socket_transport_routes_reply_and_iopub_messages_by_channel` | **Safe and deadline-covered before the queue assertions.** `finish_adapter_execute` uses `wait_for_execute_completion` (`tests/mod.rs:66-87`) before the dispatch is sent synchronously. |
| `tests/wire.rs:650-675`, `event_pump_ignores_comm_messages_and_keeps_processing_later_execute` | **First half safe; second half unbounded.** Ignoring `comm_open` is synchronous and creates no worker.  The later literal execute is driven by `pump_until_idle`, so it has no immediate-poll assumption but no hang deadline. |
| `tests/bootstrap.rs:195-196`, `local_socket_transport_backend_exposes_frontend_boundary_for_frontend_style_flows` | **Safe.** It checks a newly constructed in-memory frontend before any producer exists. |
| `tests/bootstrap.rs:225-243`, `frontend_style_execute_round_trip_uses_local_socket_backend` | **Completion-safe but unbounded.** `pump_until_idle` waits for the finite literal execute; the outgoing queue is read only after return. |
| `tests/bootstrap.rs:284-343`, `frontend_style_stdin_round_trip_uses_local_socket_backend` | **Completion-safe but unbounded.** Both initial suspension and resumed execution use `pump_until_idle`; no immediate completion is assumed, but neither call can time out. |
| `tests/bootstrap.rs:355-360`, `local_socket_transport_backend_echoes_heartbeat_payloads` | **Safe.** Heartbeat payloads are queued synchronously in the same in-memory transport; no worker thread participates. |
| `tests/bootstrap.rs:409-430`, `socket_driver_pumps_heartbeat_and_execute_over_local_session` | **Mixed but not flaky in the A shape.** Both `pump_once` assertions consume inputs queued before the calls.  Completion then uses unbounded `pump_until_idle`, which waits rather than requiring the next poll to succeed. |
| `tests/bootstrap.rs:466-541`, `socket_driver_observes_shutdown_promptly_during_running_execute` | **Initial/start assertions safe; shutdown wait not hang-safe.** The execute is queued before `pump_once`.  A sender thread eventually queues shutdown and elapsed time is checked only after unbounded `pump_until_idle` returns (`:488-506`), so a regression can hang before the assertion.  Cleanup itself has the explicit `ASYNC_TEST_TIMEOUT` loop at `:527-541`. |
| `tests/bootstrap.rs:572-611`, `socket_driver_resumes_stdin_round_trip_over_local_session` | **Completion-safe but unbounded.** Both `pump_until_idle` calls wait for actual worker completion/suspension, but have no deadline. |
| `tests/transport_coverage.rs:232-245`, `local_socket_transport_pair_routes_messages_and_heartbeats_in_memory` | **Safe.** Sends, queue-length checks, and heartbeat pumping are synchronous operations on the paired in-memory state. |
| `zeromq_fake_tests.rs:223-236`, `zeromq_backend_round_trips_execute_over_fake_router_socket` | **Safe/deadline-covered.** The initial `pump_once` consumes frames pushed synchronously; completion is obtained through `wait_for_fake_socket_event`, whose deadline is at `:49-67`, before `take_outgoing().len()` is asserted. |
| `zeromq_fake_tests.rs:261-273`, `zeromq_backend_reports_malformed_shell_frames_as_structured_wire_errors` | **Safe.** Malformed frames are synchronously present before the one pump and decoding fails in that call. |
| `zeromq/tests/native/frontend/control.rs:163-180`, `native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages` | **Safe at the no-deadline call.** The preceding pending-channel and next-event waits have deadlines (`zeromq_native_test_support.rs:136-174`); after the Control event has set shutdown, `pump_until_idle` exits synchronously without starting Shell work. |

The rest of the ZeroMQ pump/queue class is explicitly deadline-covered, not an
omission from the table: the fake completion helper is bounded at
`zeromq_fake_tests.rs:49-67`; native event/pending helpers are bounded at
`zeromq_native_test_support.rs:115-174`; the tampered-signature pump loop is
bounded at `zeromq/tests/native/signed.rs:187-200`; and native Basic, Execute,
Stdin, Control, Signed, and Session Flow use those helpers plus per-receive
`tokio::time::timeout` calls (`native/basic.rs:47-62,110-122`;
`native/signed.rs:70-125,187-207`;
`native/frontend/execute.rs:69-132`;
`native/frontend/stdin.rs:86-186`;
`native/frontend/control.rs:51-62,163-225`;
`native/frontend/session_flow.rs:93-144,184-285,323-374,412-432`).  The IOPub
`len() < 4` loops in Execute/Stdin/Session Flow are therefore bounded on every
receive.  Adapter and stdin dispatch-vector assertions use
`wait_for_execute_completion` (`tests/mod.rs:66-87`) and likewise do not make a
single-poll readiness assumption.

The actionable class result is one confirmed immediate-completion defect (A),
one additional duration-based completion-first assumption (`pump.rs:224-229`),
and several tests that are semantically safe from A but could hang because they
exercise the deliberately blocking `pump_until_idle`.  I would not change the
blocking production contract merely to give tests a timeout; the follow-up
briefs below repair the two real races and make the B precondition structural.

### 3. Step-by-step plan

#### Step 1 — make the pump routing test wait for the event it asserts

- Touch `crates/rum-jupyter/src/tests/mod.rs` to add a
  `wait_for_pump_execute_completion` test helper that repeatedly calls the
  public non-blocking `KernelEventPump::pump_execute_completion`, sleeps only
  between unsuccessful polls, and fails at `ASYNC_TEST_TIMEOUT` with adapter
  state in the diagnostic.
- Touch `crates/rum-jupyter/src/tests/pump.rs`, only in
  `event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately`, to
  use that helper before asserting the reply/IOPub queue counts and routing
  identities.  Keep the initial `pump_once`, the zero-shell/two-IOPub prologue
  assertions, and every routing assertion unchanged.
- Gate with the exact A test in release mode for 30 capped runs,
  `cargo test -p rum-jupyter --release --lib tests::pump`, and
  `cargo test --workspace`.
- Must not change production polling behavior or reinterpret `false`; must not
  weaken routing-ID or IOPub-separation coverage; must not replace the deadline
  with `yield_now`, a fixed sleep, or `pump_until_idle`.

This is independently gateable and purely test-side.  It closes A without
coupling its repair to the B lifecycle defect.

#### Step 2 — latch an accepted Jupyter interrupt across worker startup

- Touch `crates/rum-jupyter/src/adapter/worker.rs` in
  `ExecuteState::cancel`/`take_finished` so the existing `interrupted` state is
  a latch: while an interrupted Eval or Resume worker is still owned by the
  adapter, every completion poll reasserts its `HostCancelHandle` before
  checking `JoinHandle::is_finished()`.  The first Control dispatch remains
  non-blocking and still stores immediately; reassertion is confined to the
  already-interrupted cold path.  This bridges `rum-host`'s intentional
  pre-evaluation no-op semantics without changing `rum-host` or the runtime.
- Add a scoped `#[cfg(test)]` worker-start gate in
  `crates/rum-jupyter/src/adapter/worker.rs` and the smallest plumbing in
  `crates/rum-jupyter/src/adapter/mod.rs` needed for crate-local tests to pause
  one worker immediately before `HostSession::eval`/`resume_with_input`.
  Use channels or a barrier with an acknowledgement; no global mutable hook and
  no duration-based sleep.
- Rewrite
  `crates/rum-jupyter/src/tests/pump.rs::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle`
  to establish the formerly failing order by construction: wait until the Eval
  worker is held before host evaluation, assert Shell deferral, pump Control so
  cancel is delivered, release the worker, then drive completion to the
  existing deadline.  Preserve the immediate Control reply, cancelled shell
  response, routing IDs, lifecycle reset, deferred `42` execute, and IOPub
  result assertions.
- Add the same pre-start regression for a Resume worker in
  `crates/rum-jupyter/src/tests/stdin.rs` (or a focused worker/adapter test if it
  can prove the full state transition), because `spawn_resume`
  (`adapter/worker.rs:98-105`) has the identical startup window and the common
  latch is intended to cover both paths.
- Gate with both deterministic pre-start tests, 30 capped release runs of the
  B exact test, `cargo test -p rum-jupyter --release --lib tests::pump`,
  `cargo test -p rum-jupyter`, the focused `rum-host` cancellation contract
  tests, and `cargo test --workspace`.
- Must not block the Control reply waiting for guest cooperation; must not make
  a pre-evaluation `HostCancelHandle::cancel` affect an unrelated later host
  evaluation; must not detach or duplicate a session; must preserve stdin
  suspension/resume, panic recovery, post-cancel session usability, routing,
  and lifecycle transitions.  No runtime hot-path or `Do` change is justified
  by this diagnosis.

The minimal production mechanism above is preferred over changing
`begin_top_level_interrupt_budget`: the latter would violate the explicit host
contract that a cancel before or after an evaluation is a no-op for later
evaluations (`lifecycle_contract.rs:107-117`).  Reassertion is also naturally
tied to the operation that must be driven anyway to obtain a Jupyter reply.

### 4. Drafted follow-up implementation briefs

#### Draft 1 — deadline-drive the pump routing completion assertion

##### Goal

Make
`tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately`
assert the documented non-blocking completion-poll contract deterministically:
it must wait to a bounded deadline for a completion dispatch, then retain all
existing shell routing-ID and separate IOPub-routing assertions.  This is a
test-only repair; production behavior is unchanged.

##### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, and brief 397's accepted A
  diagnosis before editing.
- `crates/rum-jupyter/src/transport/event_loop.rs:94-109` owns the production
  `pump_execute_completion` contract: `false` is a valid “none ready now.”
- `crates/rum-jupyter/src/adapter/worker.rs:71-83` makes readiness explicitly
  dependent on `JoinHandle::is_finished()`.
- `crates/rum-jupyter/src/tests/mod.rs:27,66-87` owns the async test deadline
  and the existing adapter-level completion waiter.
- `crates/rum-jupyter/src/tests/pump.rs:3-53` is the only failing A test.
- Acceptance is mechanical: the exact release repetitions plus the crate and
  workspace gates decide.

##### Scope

- Files you MAY modify:
  - `crates/rum-jupyter/src/tests/mod.rs`
  - `crates/rum-jupyter/src/tests/pump.rs`
- Files/areas you MUST NOT touch:
  - all production code, including `transport/event_loop.rs`, `adapter/mod.rs`,
    and `adapter/worker.rs`;
  - `rum-host`, `rum-runtime`, `rum-stdlib`, public model/serde shapes,
    `Cargo.lock`, `.git`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, and `docs/**`
    other than appending your report to this brief if dispatched that way.
- Do not add a fixed pre-poll sleep.  The test must poll the condition to a
  deadline and may sleep only between unsuccessful non-blocking polls.

##### Deliverable

- A focused pump-level waiter using `ASYNC_TEST_TIMEOUT` and a 1 ms retry sleep,
  with an informative timeout panic.
- The A test using that waiter while preserving, unchanged in meaning:
  - the initial `pump_once` success;
  - shell `0` / IOPub `2` after the prologue;
  - shell `1` / IOPub `4` after completion;
  - original shell routing identities;
  - empty routing identities on every IOPub message.
- Verbatim output for the exact test, the 30-run release repetition, the
  `tests::pump` release filter, and the workspace gate.

##### rum / Rust pitfalls (read twice)

1. `pump_execute_completion == false` is not an error and does not mean the
   worker was lost; it means no completion was ready on that call.
2. Keep the deadline outside the loop and derive elapsed time from monotonic
   `Instant`; do not reset it after an unsuccessful poll.
3. Do not replace the focused poll with `pump_until_idle`: this test owns the
   direct completion-pump routing path.
4. Assert queue counts only after the relevant synchronous dispatch or bounded
   completion wait.  Do not weaken any routing or channel assertion.
5. This change has no hot-path production component and needs no microbench.

##### Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
python3 scripts/run-bounded.py --timeout 120 -- \
  cargo test -p rum-jupyter --release --lib \
  tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately \
  -- --exact

# Reuse the compiled release test binary; cap at 30 and report all failures.
ls -t target/release/deps/rum_jupyter-* | grep -v '\.d$' | head -1
<binary> --exact \
  tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately

python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-jupyter --release --lib tests::pump
python3 scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

The repetition is 30 invocations, not one invocation with thirty test threads.
Stop and report if any exact repetition fails after the bounded waiter is in
place; do not increase `ASYNC_TEST_TIMEOUT` to hide it.

##### Hard rules

- Do NOT commit, branch, push, or touch git config.
- Do not modify production code or relax `pump_execute_completion` semantics.
- No new `unsafe`; no external oracle or third-party engine; preserve the
  Testing Policy.
- Use `scripts/run-bounded.py` for cargo commands that can spawn or hang.
- Preserve unrelated dirty-worktree changes and leave no probe files.
- Paste real verbatim results; if a gate fails, report it rather than editing
  the expected counts or routing assertions.

##### Report format

1. What changed, per allowed file and function.
2. Why the waiter matches the non-blocking `bool` contract.
3. Exact-test and 30-run counts, then crate/workspace verification output.
4. Mechanical ripple (expected: none).
5. Stop-loss/open questions.

#### Draft 2 — latch Jupyter cancellation across Eval and Resume startup

##### Goal

Guarantee that once `KernelAdapter` has accepted an execute/resume worker and a
Control interrupt marks that worker interrupted, cancellation remains requested
until that worker finishes.  An interrupt acknowledged during the gap before
`HostSession::eval` or `resume_with_input` reaches its top-level prologue must
not be erased permanently.  The Control reply remains immediate, the host's
pre-evaluation-no-op contract remains unchanged, and deterministic tests cover
both Eval and Resume startup orderings.

##### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/host-embedding-and-hardening.md:52-65`, and brief 397's accepted B
  diagnosis before editing.
- `crates/rum-jupyter/src/adapter/worker.rs:21-115` owns worker state, cancel
  delivery, Eval/Resume spawn, readiness, and join.
- `crates/rum-jupyter/src/adapter/mod.rs:154-161,301-319,386-407` owns Control
  interrupt and execute lifecycle transitions.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:31-39` clears a
  pre-evaluation cancel intentionally; do not change it.
- `crates/rum-host/tests/lifecycle_contract.rs:107-161` is the normative host
  distinction between pre-evaluation no-op and current-evaluation cancel.
- `crates/rum-jupyter/src/tests/pump.rs:174-343` owns the end-to-end Control,
  routing, lifecycle, deferral, and recovery assertions.
- `crates/rum-jupyter/src/tests/stdin.rs` owns resume lifecycle tests.

##### Scope

- Files you MAY modify:
  - `crates/rum-jupyter/src/adapter/worker.rs`
  - `crates/rum-jupyter/src/adapter/mod.rs` (only minimal test-gate plumbing)
  - `crates/rum-jupyter/src/tests/mod.rs` (only shared test support if needed)
  - `crates/rum-jupyter/src/tests/pump.rs`
  - `crates/rum-jupyter/src/tests/stdin.rs`
- Files/areas you MUST NOT touch:
  - `rum-host`, `rum-runtime`, `rum-stdlib`, the `Do` driver, public wire/model
    serde shapes, `Cargo.lock`, `.git`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
    and `docs/**` other than the dispatched report append.
- The test gate must compile out of non-test builds.  No global mutable worker
  hook, no environment variable, and no fixed sleep may establish ordering.
- Reassertion may run only after a real interrupt has latched; do not add an
  atomic store/allocation/lock to ordinary non-interrupted completion polls.

##### Deliverable

- `ExecuteState` retains the first immediate cancel and reasserts the same
  `HostCancelHandle` on later completion polls while `worker.interrupted` is
  true and the worker remains owned.  The mechanism is common to Eval and
  Resume.
- A crate-test-only, instance-scoped start gate that can:
  1. acknowledge that one selected worker is paused immediately before its
     `HostSession` entrypoint;
  2. allow the test to dispatch Control while paused;
  3. release exactly that worker without affecting parallel tests.
- The existing B test establishes cancel-before-host-start by that gate, then
  preserves all current assertions: immediate Control reply, `Cancelled`
  execute error, original routing IDs, Idle/Running lifecycle restoration,
  deferred Shell count, successful `42`, and IOPub result.
- A corresponding pre-start Resume regression proves the common fix covers
  `spawn_resume`, including usable session state after cancellation.
- A short code comment at the latch/reassert site explains why one store is
  insufficient across the host top-level clear.  No broad refactor.

##### rum / Rust pitfalls (read twice)

1. Do not change `HostCancelHandle` to cancel a later evaluation.  Its
   before/after no-op behavior is intentional and tested.
2. Do not block `KernelAdapter::interrupt` or delay `interrupt_reply` until the
   guest cooperates.  Jupyter Control must remain responsive.
3. Reassert before checking finished state on each interrupted completion poll;
   a store made only during Control dispatch is the defect.
4. Apply the state rule to both `ExecuteSegment::Eval` and `Resume`; fixing only
   the currently failing Eval test leaves the identical class in place.
5. The ordering proof comes from a channel/barrier acknowledgement, not from
   a source workload being “long enough.”  Keep a real deadline only for the
   eventual asynchronous completion.
6. Preserve one-session/one-worker ownership.  Do not clone, share mutably, or
   detach `HostSession`.
7. Cancellation is a cold control path.  No runtime evaluator hot path changes
   and no core-language microbench are warranted; if scope expands into
   `rum-runtime`, stop for a new brief and measured A/B.

##### Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
python3 scripts/run-bounded.py --timeout 120 -- \
  cargo test -p rum-jupyter --lib \
  tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle \
  -- --exact --nocapture
python3 scripts/run-bounded.py --timeout 120 -- \
  cargo test -p rum-jupyter --lib <exact-resume-regression> -- --exact --nocapture

python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-jupyter --release --lib tests::pump

# Reuse the compiled release lib-test binary and invoke the exact B test 30
# times.  Report pass/fail counts; the barrier makes the adverse order fixed.
ls -t target/release/deps/rum_jupyter-* | grep -v '\.d$' | head -1
<binary> --exact \
  tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle

python3 scripts/run-bounded.py --timeout 300 -- cargo test -p rum-jupyter
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-host --test lifecycle_contract cancel_handle
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-host --test stress cancel_interrupts_heavy_rewrite_loop_and_session_recovers
python3 scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

Also run the deterministic B test once with `--test-threads=1`; it must prove
the same acknowledged barrier order.  If the test needs a duration to ensure
Control precedes host evaluation, the acceptance condition is not met.

##### Hard rules

- Do NOT commit, branch, push, or touch git config.
- No source changes outside the explicit scope; in particular do not “fix” the
  runtime's top-level flag clear or add a `Do` poll.
- No new `unsafe`; preserve the Testing Policy and use no external oracle.
- Control dispatch must stay non-blocking.  A retry thread, busy wait in
  `interrupt`, or longer timeout is not an acceptable substitute for the
  latched state.
- Test-only ordering support must be instance-scoped and inert in production;
  parallel tests must not share a global gate.
- Use bounded cargo commands, preserve unrelated dirty changes, and leave no
  temporary instrumentation or probe files.
- Paste actual output.  A failure in host pre-cancel semantics, stdin recovery,
  routing, lifecycle reset, or the 30-run deterministic regression is a defect
  to fix or report, not an assertion to weaken.

##### Report format

1. Structural argument: why a cancel cannot be permanently lost on either side
   of host startup, and why Control remains immediate.
2. What changed per file/function, including the test-only gate and why it has
   zero production presence.
3. Eval and Resume adverse-order tests and the 30-run release count.
4. Jupyter, host-contract, and workspace verification output.
5. Mechanical ripple and production-cost statement.
6. Stop-loss/open questions.

### 5. Stop-loss, verification, risks, and corrections

The first serious B probe produced a switchable mechanism, so I did not spend
the allowed second attempt.  Repetition caps were 10 per unmodified exact or
whole-binary condition, 5 for the serial and forced-order conditions, and
never more than the brief's cap of 30.  The temporary instrumentation was:

- `crates/rum-jupyter/src/adapter/worker.rs`: 100 ms sleep and trace immediately
  before `run_segment`;
- `crates/rum-host/src/session.rs`: trace immediately before the cancel store;
- `crates/rum-runtime/src/runtime/session/session_helpers.rs`: trace of the
  prior flag immediately before the startup clear;
- `crates/rum-runtime/src/runtime/eval_core/mod.rs`: trace when a poll observed
  cancellation;
- `crates/rum-jupyter/src/tests/pump.rs`: 250 ms parent delay for the opposite
  forced ordering.

Every item above was reverted with `apply_patch`; `rg -n brief397 crates`
returned no output and `git diff` over those five source files was empty before
the final gates.

Corrections/qualifications to the brief's background:

1. A's proposed mechanism is correct.
2. B is not a too-short deadline and not a missing Control dispatch.  Control
   stores cancellation, but a later host evaluation prologue can erase it; the
   evaluation genuinely does not observe cancellation in that ordering.
3. The standalone `rum-host` before-evaluation behavior is intentional, so the
   defect is the Jupyter adapter's mismatch between “worker accepted/running”
   and “host current evaluation has begun,” not a reason to change the host's
   pre-cancel contract.
4. More tests do not grant more wait time.  Default-concurrent whole-binary
   runs bias both A and B toward pass by giving the just-spawned worker more
   chances to run before the parent test's next poll/Control dispatch.  A whole
   serialized binary largely removed that bias.  No relevant path calls
   `available_parallelism`; the probe did not need a thread-count or CPU-count
   hypothesis.
5. My one workspace release run did **not** reproduce the background's exact
   aggregate.  It produced 2236 passed / 1 failed / 7 ignored: A failed and B
   passed.  That is consistent with B's measured whole-process race, not a
   refutation of the standalone failure.

The exact workspace command was:

```
python3 scripts/run-bounded.py --timeout 900 -- \
  cargo test --workspace --release --no-fail-fast
```

The post-run parser and the failing harness summary printed verbatim:

```
command_status=101 log=/tmp/brief397-workspace-release.log
parsed_test_result_lines=29
aggregate passed=2236 failed=1 ignored=7 measured=0
failing_summaries:
test result: FAILED. 102 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

The relevant raw log lines were:

```
test tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately ... FAILED
test tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle ... ok
assertion failed: pump.pump_execute_completion(&mut transport).expect("completion poll should succeed")
test result: FAILED. 102 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
error: 1 target failed:
    `-p rum-jupyter --lib`
```

Risks/open decisions for the implementation brief author:

- Reassert-on-poll is the smallest adapter-local repair and preserves the host
  contract, but it makes persistence depend on the adapter continuing to be
  driven.  Producing any shell reply already requires that drive contract.  If
  cancellation must progress while a host stops polling entirely, a separate
  host evaluation-start handshake/token design is needed and deserves its own
  broader design brief rather than an ad hoc retry thread.
- `force_cancelled_success` only coerces `KernelExecuteOutcome::Success`, not
  `InputRequired` (`adapter/worker.rs:159-165`).  A worker that wins a race into
  input suspension after Control could leave lifecycle state at
  `InterruptRequested`.  The requested Resume regression should settle whether
  reassertion alone covers that boundary; if not, cancellation-versus-input
  precedence and safe continuation cleanup need an explicit decision rather
  than silently discarding a pending suspension.
- Several tests exercise the intentionally blocking `pump_until_idle` without
  a watchdog.  They do not share A's false next-poll assertion, but a regression
  can hang the test process.  Adding a public deadline/error policy solely for
  tests would expand the API; I left that as separate hardening rather than
  mixing it into either repair.
- The test-only worker gate must be instance-scoped.  A process-global gate
  would create exactly the cross-test scheduling dependency this work is meant
  to eliminate.

Because this brief is diagnostic/design-only and all probe edits were reverted,
I did not run `cargo fmt`; there is no source test gate to make green.  The
evidence commands above are the mandatory deliverable.  `git diff --check`
completed with no output.

Final worktree status (the five tracked script/hook entries and untracked
`scripts/peak-rss.py` are the pre-existing user changes; this report is the
only new entry):

```
 M .claude/hooks/guard-dispatch.py
 M docs/dev/codex/brief-397-jupyter-pump-execute-lifecycle.md
 M scripts/README.md
 M scripts/core-memory-microbench.sh
 M scripts/dispatch-brief.sh
 M scripts/guarded-test-run.sh
?? scripts/peak-rss.py
```

### 6. Native ZeroMQ result

`zeromq::tests::native::*` **skipped its loopback bodies** in this sandbox:
`cargo test -p rum-jupyter --release zeromq::tests::native -- --nocapture`
printed `skipping native ZeroMQ test: loopback socket binding is not permitted`
for all nine selected tests, and the harness reported `9 passed; 0 failed`
because each skip is an early return rather than `#[ignore]`.
