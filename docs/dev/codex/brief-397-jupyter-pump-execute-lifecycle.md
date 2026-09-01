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
