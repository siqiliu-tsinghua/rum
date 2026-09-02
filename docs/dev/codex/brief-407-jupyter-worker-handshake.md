# brief-407 — implement: deterministic in-process worker synchronization

Shape: **implementation**. Family A of the four brief-406 identified. It
touches no socket code; families B, C and D are separate briefs.

## Where this came from

brief-406 diagnosed the `rum-jupyter` suite's timing assumptions. Family A is
the in-process one: tests assert things about an execute worker that the
production API does not promise.

The production surface exposes only a nonblocking poll.
`KernelAdapter::poll_execute_completion` (`adapter/mod.rs:301-319`) delegates to
`ExecuteWorkerSlot::take_finished` (`adapter/worker.rs:71-82`), which asks
`JoinHandle::is_finished()`. There is **no happens-before edge** between
`thread::spawn` (`adapter/worker.rs:168-179`) and the first sample. Neither
immediate completion nor any latency is promised.

Two shapes of invalid assumption follow from that:

- **Immediate readiness.** `tests/pump.rs:31-40` asserts the worker is already
  finished at the first completion poll. It fails 30 out of 30 in isolation on
  my host and usually passes inside the whole suite, because concurrent
  execution supplies the preemption the test needs but never establishes.
- **Elapsed time as a semantic requirement.** `ASYNC_TEST_TIMEOUT`
  (`tests/mod.rs:27`) and `wait_for_execute_completion` (`tests/mod.rs:66-75`)
  turn "within two seconds" into a pass condition across most worker-dependent
  tests, and `tests/bootstrap.rs:442-541` stages a shutdown with a 2 ms sleep
  and asserts observation under one second.

## The design constraint, already settled by measurement

brief-406's scratch probe varied only what happens between dispatch and the
first completion poll:

```text
mode=cold         completion_ready=false
mode=warm         completion_ready=false
mode=worker-warm  completion_ready=false
mode=yield        completion_ready=false
mode=sleep        completion_ready=true
```

**A yield is not enough.** Therefore no spin, no `yield_now`, no retry budget,
and no sleep is an acceptable synchronization mechanism here. The repair must
install a genuine happens-before edge: the test learns the worker's state
because the worker told it, not because enough time passed.

The one exception is an existing production loop whose termination the
production code itself guarantees — `pump_until_idle`
(`transport/event_loop.rs:177-212`) is such a loop. Using it is acceptable
where the test's subject is "the finite work eventually completes"; it is not
acceptable as a way to make a readiness assertion true.

## Goal

Every in-process `rum-jupyter` worker test establishes worker state through an
instance-scoped signal or gate. Immediate-completion assumptions, fixed sender
sleeps, and elapsed-time-as-semantics all disappear, and every semantic
assertion those tests carry survives.

## Context pointers

- Read `AGENTS.md` and **brief-406's committed report**, especially its class
  inventory: it grades every test P/E/I/M and says which assertions are real
  coverage. That table is your worklist for this family.
- Worker: `adapter/worker.rs:65-82,165-190`; adapter poll:
  `adapter/mod.rs:301-319`.
- Finding A: `crates/rum-jupyter/src/tests/pump.rs:4-53`, assertion at `:37`.
- Residual assumptions in the finding-B test:
  `tests/pump.rs:175-343`, specifically the "still running" assumption at
  `:224-229` and the bounded cancellation wait at `:248-258`. brief-404 fixed
  the kernel defect that test exposed; its timing assumptions remain.
- Shared deadline: `tests/mod.rs:27,66-87`.
- Fixed shutdown staging: `tests/bootstrap.rs:442-541`.
- Fake-socket polling loop: `zeromq_fake_tests.rs:49-68,198-251`.
- **Your sandbox has no network.** Pass `--offline` to every `cargo` command.
  The native ZeroMQ tests will skip in your sandbox and report `ok` anyway —
  see brief-406 §1. They are not this brief's subject; do not touch them and
  do not make claims about them.

## Scope

- Files you MAY modify, all under `crates/rum-jupyter/src/`:
  - `adapter/worker.rs` and `adapter/mod.rs`, **only** to add a test-only
    signal and only if a test cannot establish the property without it;
  - `tests/mod.rs`, `tests/pump.rs`, `tests/bootstrap.rs`,
    `tests/adapter/dispatch.rs`, `tests/wire.rs`, `tests/stdin.rs`,
    `tests/adapter.rs`;
  - `zeromq_fake_tests.rs`.
- Files/areas you MUST NOT touch:
  - `crates/rum-jupyter/src/zeromq/tests/native/**` and
    `zeromq_native_test_support.rs` — families B, C and D;
  - `rum-runtime`, `rum-host`, `rum-stdlib`, `rum-core`, `rum-cli`, `rum-repl`;
  - the wire protocol, public serde shapes, `Cargo.lock`, dependency edges,
    `TODO.md`, `docs/`.
- If you need a file outside this list, **STOP and report**. My MAY lists have
  been short eight times on this project; two of those were module roots, so
  if a module root must change to export a test-only type, say so rather than
  assuming it is covered.

## The accept-vs-stop criterion

Replacing a timing mechanism with a causal one is in scope. **Weakening or
deleting a semantic assertion is a stop.** Routing-ID preservation, IOPub
separation, Control-before-Shell priority, Busy/Idle ordering, stdin
prompt/backtracking sequences, execution counts, and cancellation outcomes all
survive unchanged. If a test cannot be synchronized without dropping one of
them, that is a finding — stop and report it.

Changing production evaluation semantics or adding any default-path cost is
also a stop.

## Deliverable

1. A test-only, **instance-scoped** worker signal — start and/or finish — plus,
   where a test needs the worker provably still running, a release gate the
   test holds. Both `#[cfg(test)]`; neither may exist in a release build, and
   no production path may gain a branch, allocation, lock, or field.
2. **No global state.** These tests run in parallel; a process-wide gate or
   signal is a defect, not a shortcut.
3. Finding A waits for the actual finish signal, then asserts exactly what it
   asserts today about routing IDs and IOPub separation.
4. `tests/pump.rs:224-229` establishes "still running" by holding a gate, not
   by assuming the worker has not finished yet.
5. `tests/bootstrap.rs:442-541` queues shutdown after a running-state
   handshake; the 2 ms sleep and the one-second observation assertion are gone.
6. **Deadlines survive only as harness stop-loss.** Any remaining bound must
   panic with a message that names itself a harness timeout, must not be
   asserted as a property of the kernel, and must be justified in the report
   from measured data rather than chosen. No test may assert that anything
   completed *within* a duration.
7. **A per-test accounting** in the report: for every test you touched, the
   assumption removed and the semantic assertions preserved. Zero deletions.

## rum / Rust pitfalls (read twice)

1. No spin, `yield_now`, sleep, or retry budget as the synchronization
   mechanism — the probe above settles that they do not work.
2. Do not join the worker or consume its result outside the adapter's normal
   completion path; the adapter owns that transition.
3. A gate that holds a worker must be released on every exit path, including
   panics, or a failing test will hang the suite instead of failing it.
4. Preserve cancellation, suspend, and resume behaviour exactly; brief-404 just
   landed the cancellation contract and this brief must not perturb it.
5. No new `unsafe`. No `Mutex` on any production path.
6. `debug_assert!(!self.is_running())` at `adapter/worker.rs:67` is a real
   invariant; do not weaken it to make a gate fit.

## Verification (MANDATORY — you run it; iterate to green)

Pass `--offline` to every `cargo` command.

**Red first.** Run finding A before any change and paste the failure:

```
scripts/run-bounded.py --timeout 180 -- cargo test --offline -p rum-jupyter --lib tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately -- --exact --nocapture
```

Then repair, and paste the same command green, plus:

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 600  -- cargo test --offline -p rum-jupyter
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

`--no-fail-fast` is required; `cargo test` otherwise stops after a failing
binary and under-reports the rest.

**Isolation is the real gate for this family.** Finding A fails 30/30 in
isolation today precisely because isolation removes the accidental preemption.
Run each test you touched **on its own**, ten times, and paste the counts. A
test that only passes in company is not fixed.

**Native tests: report, do not touch.** They skip in your sandbox and report
`ok` regardless. State that they skipped; make no other claim about them.

No timing A/B: this is test-side work on cold paths. If you change production
evaluation or dispatch behaviour, the premise is exceeded — stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Leave changes in the working tree.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. No probe artifacts in the repo.
- **Testing Policy is a hard line**: rum's own documented behavior only.
- **No new `unsafe`.** No test deleted, no assertion weakened.
- Minimal diff; no drive-by reformatting or renames.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. Per-file change with line ranges and a one-line why.
2. The synchronization mechanism, and why it is a happens-before edge rather
   than a faster spin.
3. The per-test accounting from Deliverable 7.
4. The red test verbatim, then green; the isolation counts.
5. Verification output, including the native skip statement.
6. Evidence that no production path changed, and the justification for any
   surviving harness stop-loss.
7. Stop-loss, risks, open questions.
