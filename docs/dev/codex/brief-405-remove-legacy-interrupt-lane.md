# brief-405 — implement: delete the legacy interrupt lane

Shape: **implementation**. Step 2 of the cancellation repair, and the last
kernel step of this line. It leaves exactly one cancellation mechanism.

## Where this came from

brief-404 (`a81b703e`) landed request-scoped, generation-qualified
cancellation and fixed the erasure defect. It could not finish the job: while
wiring it, Codex found that `rum-stdlib/src/lazy.rs` polls
`Session::interrupt_handle() -> Arc<AtomicBool>` directly, and `rum-stdlib`
was outside that brief's scope. It stopped rather than working around the
boundary — the right call — and kept the boolean alive as a synchronized
mirror beside the new phase word.

So the tree now carries **two** cancellation mechanisms, and the seam between
them is not free:

```rust
// session_helpers.rs:30-38, in the request prologue
let legacy_interrupt_armed = {
    let control = self.request_interrupt_control.borrow();
    if self.active_accepted_request.is_none() && control.has_cancelled_request() {
        control.suppress_legacy_interrupt();
    } else {
        control.synchronize_legacy_interrupt();
    }
    control.legacy_handle_is_shared()
};
```

That conditional exists **only** because two mechanisms coexist. Deleting the
legacy lane deletes it, along with a state interaction that has to be kept
right by hand.

There is also a trap shape to remove. `Session::interrupt_flag`
(`model/session.rs:123`) is now written by nothing in production; the line that
polled it is behind `#[cfg(test)]` (`eval_core/mod.rs:100-103`), and only two
test files still touch it. Nothing is silently missed today, but production
code written against that field later would do nothing in a release build.

## What I checked before writing this, so you do not have to re-derive it

- `request_interrupt_control` is already `RefCell<Arc<RequestInterruptState>>`
  (`model/session.rs:118`), and `RequestInterruptState` holds an `AtomicU64`,
  so handing out an `Arc` clone as a `Send + Sync` observer is available
  without new machinery.
- `has_cancelled_request()` (`model/request_cancellation.rs:142`) already reads
  `phase & CANCELLED_BIT` and is generation-agnostic — the right question for a
  lazy stepper, which asks "is the request running right now cancelled".
- `interrupt_armed` is not armed for an ordinary request: `active_accepted_request`
  is `Some` only when a ticket was installed, and a pending ticket is created
  only when somebody calls `request_cancel_handle()`
  (`session_api.rs:171-183`). I verified this at brief-404's gate; the hard
  line held. **Your change must keep it holding** — see Deliverable 4.
- `rum-cli`'s long-lived-looking handle is fine: `install_stdout_print_sink` is
  called inside `eval_request` (`rum-cli/src/main.rs:251`), which wraps exactly
  one request, and the interactive REPL path does not use it. Confirm this
  independently and say so; do not take my word for it.

## Goal

One cancellation mechanism. The legacy `Arc<AtomicBool>` lane, its mirror
synchronization, its arming heuristic, and the vestigial `interrupt_flag` are
gone; `rum-stdlib`'s lazy steppers observe the phase word; and a test proves
the ordinary request path is still unarmed.

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md`, brief-398's report,
  and brief-404's committed report before touching anything.
- The lane to delete: `model/request_cancellation.rs` — `legacy_interrupt`
  (`:123`), `legacy_handle()` (`:135`), `legacy_interrupt_requested()`
  (`:138`), `legacy_handle_is_shared()` (`:146`),
  `synchronize_legacy_interrupt()` (`:150`), `suppress_legacy_interrupt()`
  (`:161`), and the `synchronize_legacy_interrupt()` calls at `:180,193,228`.
- Its users: `session_helpers.rs:30-38,99,105`, `session_api.rs:166-168,195`,
  `eval_core/mod.rs:96-103`.
- The consumer to migrate: `rum-stdlib/src/lazy.rs:33,41,46`.
- Test users of the old API: `rum-stdlib/src/tests/lazy.rs:104`,
  `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:776`, plus the
  two writers of `interrupt_flag` in `runtime/evaluation_cache/tests.rs:710`
  and `runtime/tests/part_assignment.rs:395`.
- **Your sandbox has no network.** Pass `--offline` to every `cargo` command.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model/request_cancellation.rs`,
    `runtime/model/session.rs`, `runtime/session/session_helpers.rs`,
    `runtime/session/session_api.rs`, `runtime/session/session_state.rs`,
    `runtime/session/session_requests.rs`, `runtime/eval_core/mod.rs`,
    `runtime/kernel_api.rs`;
  - `crates/rum-stdlib/src/lazy.rs` and `crates/rum-stdlib/src/tests/lazy.rs`
    — **in scope this time**, and only for the cancellation observer;
  - tests under `crates/rum-runtime/src/runtime/tests/` and
    `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`;
  - `crates/rum-host/src/session.rs` and its tests, only if an export moves;
  - `runtime/tests/frame_root_trace.rs` if you add a gate for the removal.
- Files/areas you MUST NOT touch:
  - `crates/rum-jupyter` — its remaining test failures are brief-406. Its
    adapter already uses `cancel_handle()`, which brief-404 made correct;
    verify that and report it, but change nothing there.
  - `crates/rum-cli`, unless you find the consumer analysis above is wrong, in
    which case **STOP and report** rather than fixing it here.
  - the abandonment protocol, `FrameAbandon` classifications, brief-403's
    rejection contract, `rum-core`, `Cargo.lock`, dependency edges, the
    generated builtin reference, ordinary evaluator/dispatch/matcher logic.
- If you need a file outside this list, **STOP and report**. My MAY lists have
  been short seven times on this project.

## The accept-vs-stop criterion

Removing the second mechanism is in scope. **Changing when cancellation is
observed is a stop.** A lazy stepper must still notice a cancel at the same
pull granularity; a running evaluation must still notice at the same poll
cadence; the documented structured outcome is unchanged. If removal appears to
require changing observation timing or the reported outcome, that is a finding,
not a licence.

## Deliverable

1. A `Send + Sync` cancellation observer, exported through `kernel_api`, that
   `rum-stdlib` can hold: one relaxed atomic load, no lock, no allocation per
   pull, generation-agnostic.
2. **The observer must not report a stale cancellation.** After the cancelled
   request completes, a stepper that outlived it and is pulled under a later
   request must see "not cancelled". Prove this with a test: cancel a lazy
   pull, let the request finish, then pull the same lazy value again in a fresh
   request and show it proceeds.
3. `rum-stdlib/src/lazy.rs` migrated; `Session::interrupt_handle()` deleted.
4. **The arming proof.** `interrupt_armed` becomes a function of the deadline
   and the accepted request only. Add a test asserting that an ordinary
   evaluation — no timeout, no cancel handle taken — leaves the interrupt poll
   unarmed, **and** that merely holding a lazy-sequence observer does not arm
   it. The old `legacy_handle_is_shared()` heuristic counted `Arc` holders, so
   a permanently-held observer would have armed it forever; this test is what
   stops that regression from returning silently.
5. Every name listed in Context pointers is deleted, including
   `Session::interrupt_flag` and the `#[cfg(test)]` poll line and its now-unused
   `Ordering` import. A repository gate fails if any of them returns.
6. The prologue's suppress/synchronize conditional is gone, not merely
   simplified.
7. The two `interrupt_flag` test writers and the two `interrupt_handle` test
   users are migrated to the qualified mechanism, not deleted. If a test's
   subject disappears with the lane, say so explicitly rather than dropping
   coverage silently.

## rum / Rust pitfalls (read twice)

1. A lazy value can outlive the request that built it. Capture no generation
   in the observer.
2. Do not let the observer's existence arm the interrupt poll — that is
   Deliverable 4's whole point.
3. Do not change the 1024-step poll cadence or move work into the armed fast
   path; `poll_host_interrupt_if_due` stays `#[cold] #[inline(never)]`.
4. `reset()` must still retire the generation so a stale handle is inert, and
   an observer held across a reset must read the rebuilt session's state or be
   inert — state which, and test it.
5. No new `unsafe`. Do not introduce a `Mutex` on any path a pull touches.
6. Deleting a field that tests write is not the same as deleting the test's
   subject; migrate the assertion, do not weaken it.

## Verification (MANDATORY — you run it; iterate to green)

Pass `--offline` to every `cargo` command.

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-stdlib
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

`--no-fail-fast` is required; `cargo test` otherwise stops after a failing
binary and under-reports the rest.

**Known-failing, not yours.** `rum-jupyter` fails intermittently at whole-suite
level. My measurement on the committed tree, over 30 whole-suite samples: 5
failures, all either
`tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately`
or a `zeromq::tests::native::*` test. `control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle`
appeared **0 times in 30** after brief-404 fixed its kernel cause. Report what
you see; do not fix it, do not touch that crate. Any failure elsewhere is yours.

**Grep proof of removal.** Paste a verbatim search showing zero remaining
occurrences of `interrupt_flag`, `interrupt_handle`, `legacy_interrupt`,
`legacy_handle`, `synchronize_legacy_interrupt`, and `suppress_legacy_interrupt`
across `crates/`.

No timing A/B: this deletes work from a cold path and adds none to a hot one.
If you find yourself changing ordinary eval, dispatch, or matcher code, the
cost premise is exceeded — stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, checkout, stash, or touch git config.
  **Do not touch `stash@{0}`.** Leave changes in the working tree.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. No probe artifacts in the repo.
- **Testing Policy is a hard line**: rum's own documented behavior only.
- **No new `unsafe`.**
- Minimal diff; no drive-by reformatting or renames.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. Per-file change with line ranges and a one-line why.
2. The observer's shape, and why it is generation-agnostic without being stale.
3. The arming proof, verbatim.
4. The consumer verification: `rum-jupyter`'s adapter and `rum-cli`'s print
   sink under the qualified mechanism, independently derived.
5. Verification output including the grep proof and the repository invariant
   rows.
6. Cost statement.
7. Stop-loss, risks, open questions.

---

# Report — 2026-09-02

## 1. Per-file changes

- [request_cancellation.rs](crates/rum-runtime/src/runtime/model/request_cancellation.rs:116), lines 116–214 — added `CancellationObserver`; removed the Boolean mirror and all synchronization logic.
- [model/mod.rs](crates/rum-runtime/src/runtime/model/mod.rs:34), lines 34–37 — exported the observer.
- [model/session.rs](crates/rum-runtime/src/runtime/model/session.rs:115), lines 115–121 — removed the vestigial test-only flag.
- [session_api.rs](crates/rum-runtime/src/runtime/session/session_api.rs:165), lines 165–191 — exposed `cancellation_observer()`, deleted the raw handle API, and removed strong-count arming.
- [session_helpers.rs](crates/rum-runtime/src/runtime/session/session_helpers.rs:30), lines 30–36 and 84–87 — deleted the suppress/synchronize prologue and derived arming solely from deadline/accepted request.
- [session_state.rs](crates/rum-runtime/src/runtime/session/session_state.rs:662), lines 662–670 — removed obsolete flag initialization.
- [eval_core/mod.rs](crates/rum-runtime/src/runtime/eval_core/mod.rs:80), lines 80–96 — removed both old poll branches and unused `Ordering`; cadence and cold attributes are unchanged.
- [kernel_api.rs](crates/rum-runtime/src/runtime/kernel_api.rs:77), lines 77–80 — exported `CancellationObserver` to stdlib.
- [lazy.rs](crates/rum-stdlib/src/lazy.rs:4), lines 4–45 — lazy source pulls now query the phase observer.
- [lazy tests](crates/rum-stdlib/src/tests/lazy.rs:99), lines 99–127 — migrated the old flag test and proved the same lazy value works in a later request.
- [evaluation-cache tests](crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:700), lines 700–724 — preserved poll-before-mutation coverage using an accepted qualified request.
- [Part tests](crates/rum-runtime/src/runtime/tests/part_assignment.rs:382), lines 382–405 — likewise migrated poll-before-eviction/mutation coverage.
- [request API tests](crates/rum-runtime/src/runtime/tests/session_requests/api.rs:494), lines 494–542 and 826–841 — added arming, `Send + Sync`, rebuild-inertness tests, and migrated file cancellation.
- [frame_root_trace.rs](crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:370), lines 370–401, 733–744, and 885–895 — added an all-file repository gate for every deleted name.

## 2. Observer shape and stale-cancellation proof

`CancellationObserver` holds only `Arc<RequestInterruptState>`, not a generation. `is_cancelled()` performs one relaxed `AtomicU64` load and reads the cancellation bit. There is no lock or per-pull allocation.

Completion transitions the phase to idle with the existing exact-generation CAS, so an observer immediately reads “not cancelled” afterward. The lazy regression cancels during an actual mapped pull, completes that request, then realizes the same saved lazy value in a fresh request as `List[1, 2, 3]`.

Across reset/rebuild, an observer retains the retired control block and becomes inert. `cancellation_observer_held_across_session_rebuild_is_inert` proves that behavior while a newly rebuilt session has its own cancelled request.

## 3. Arming proof

The prologue is now exactly deadline-or-active-request. Observer `Arc` counts are irrelevant.

Verbatim:

```text
running 1 test
ordinary_request_armed=false observer_held_request_armed=false
test runtime::tests::session_requests::api::ordinary_request_and_lazy_observer_leave_interrupt_poll_unarmed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 931 filtered out; finished in 0.00s
```

## 4. Consumer verification

- CLI: [main.rs](crates/rum-cli/src/main.rs:221) routes eval, script, and file through one wrapper. That wrapper installs the sink at line 251 and immediately invokes exactly one request at line 253. The sink obtains the qualified host handle at line 330. The REPL path at lines 214–218 bypasses this sink entirely.
- Jupyter: [worker.rs](crates/rum-jupyter/src/adapter/worker.rs:87) obtains the qualified handle before spawning Eval. Resume obtains a handle at lines 98–103 while the suspended accepted ticket remains active, so `request_cancel_handle()` returns that same generation. Interrupt calls `cancel()` at lines 47–53. No Jupyter changes were needed.

## 5. Verification

All mandatory final-tree commands returned `exit_code=0`:

```text
scripts/run-bounded.py --timeout 300  -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-stdlib
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

Key final workspace summaries:

```text
test result: ok. 103 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 930 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.10s
test result: ok. 828 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.54s
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Both Jupyter suites passed; no known intermittent failure appeared.

Grep proof:

```text
$ ! rg -n "interrupt_flag|interrupt_handle|legacy_interrupt|legacy_handle|synchronize_legacy_interrupt|suppress_legacy_interrupt" crates/
exit_code=0
```

There were no match lines.

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
removed_cancellation_lane_hits=0
legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0
```

## 6. Cost statement

No timing A/B was run, per the brief. Ordinary evaluation loses mirror synchronization and strong-count work. Lazy pulling still performs one relaxed atomic load per source pull; the observer’s `Arc` is cloned once when creating the pull context. No mutex, allocation per pull, extra evaluator poll, cadence change, or new default-path work was introduced.

## 7. Stop-loss, risks, open questions

No stop-loss fired. No out-of-scope files, Jupyter/CLI files, dependency edges, abandonment logic, generated references, or `Cargo.lock` were changed. No `unsafe` was added, `git diff --check` is clean, and no commit/branch/stash operation was performed.

No open correctness questions remain.
