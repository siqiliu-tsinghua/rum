# brief-630 — A cleanup that fails during abandonment reports its error

**Shape: staged.** Stage A is an inventory with probes and no source change,
and it ends in a STOP for my ruling. Stage B, the implementation, follows
only in a continuation of this brief.

**Worker: grk.** Codex has no quota and kimi is the fallback. This stage is
reading code and running calibrated probes, which grk did reliably on
briefs 627 and 629.

## Goal

When a request is cancelled while it is suspended, the kernel abandons its
frames and evaluates each `WithCleanup` cleanup once. If that cleanup fails,
for example by reaching a resource limit or with an uncaught `Throw`, the
error is dropped. No message is emitted
(`crates/rum-runtime/src/runtime/suspension/core/abandon.rs:99-109`: only
`InputRequired` is looked at). The same cleanup failing on an ordinary
completion reports `General::reslim` or `Throw::nocatch`.

The user ruled for **Option A** (2026-10-27, brief-628 §4 and my review):
- the cleanup's error is reported as the message the ordinary path gives it;
- the retired request keeps its `Cancelled` outcome;
- every remaining frame is still drained.

My review also asked for the inventory that settles this: **which
`AbandonReason`s emit, from the message channel each retirement has.**
Stage A is that inventory.

Read first:
- brief-628 (`docs/dev/codex/brief-628-native-stack-base-across-threads.md`),
  §4 and the director review at its end;
- the guide's promise: `docs/guide/getting-started.md:168-182`. "Runs its
  `WithCleanup` cleanup once", "messages produced during cleanup are rolled
  back with their file", "its own cleanup messages reach stderr once", and
  "retirement does not add a second cancellation error".

## Context pointers

- `AGENTS.md`; `docs/dev/codex/TEMPLATE.md`, including the standing
  maintenance duty and the report rule on verdict lines (both updated
  2026-10-28).
- The arm: `suspension/core/abandon.rs:91-114`
  (`FrameAbandonAction::EvaluateCleanup`). After brief-629 it ends with
  `self.evaluation_depth.restore(0)`.
- The drain: `abandon.rs:5-62` (`abandon_pending_suspension`,
  `drain_abandonment_worklist`). `abandon_frames` is generic over
  `profile_mode::Mode`, so ordinary and profiled abandonment share one body,
  and `frame_root_trace` pins that shape (LP02).
- The production abandonment routes I found by grep. Verify the list and
  complete it.

  | file:line | function | reason |
  |---|---|---|
  | `session/session_requests.rs:63` | `finish_cancelled_pending_evaluation_request` | `CancelledRequest` |
  | `session/session_requests.rs:581` | `finish_accepted_value_request` | `CancelledRequest` |
  | `session/session_helpers.rs:249` | `restore_evaluation_rollback_snapshot` | `CheckpointRollbackDiscard` |
  | `session/session_api.rs:102` | `restore_snapshot_request` | `FailedSnapshotRestore` |
  | `builtins_input.rs:234` | `drive_input_request` | `SupersededOrQueuedInputDiscard` |
  | `accounting_mode/{bridge,frozen,options}.rs` | helpers | `HelperBestEffortDiscard` |
  | `debug_tools/monitor.rs:85,122` | monitor callbacks | `HelperBestEffortDiscard` |
  | `suspension/core/fault.rs:126` | slot fault (calls the drain directly) | `ContinuationSlotMismatch` |

- The ordinary path's messages:
  - `session/session_helpers.rs:478` `emit_non_loader_eval_message`, with
    its `handled_non_loader_eval_errors` checkpoint;
  - `session/session_helpers.rs:501` `emit_top_level_control_flow_messages`.
- A draft from brief-628, written before brief-629 changed this arm. It is
  raw material, not a shape to copy:
  `.temp/brief-628/drafts/brief-abandonment-cleanup-diagnostics.md`.

## Falsifiers (calibrated by the director: red at HEAD 3/3, controls green 3/3)

The probe crate is `.temp/director/630/probe/` (`src/lib.rs`), with logs
`.temp/director/630/calibration-{1,2,3}.log`. The source used is:

```rust
const REC: &str = "Block[{$RecursionLimit = 20}, f630[x_] := f630[x + 1]; f630[1]]";
fn nested(inner_cleanup: &str) -> String {
    format!("outerRan630 = 0; WithCleanup[WithCleanup[InputString[\"body\"], {inner_cleanup}], outerRan630 = 1]")
}
```

- **Falsifiers.** `f1` cancels with the inner cleanup `REC`, and `f2` with
  `Throw[630]`. Each asserts:
  - termination is `Cancelled`;
  - the outer cleanup ran (`outerRan630` is `1`);
  - the cleanup's message is present (`General::reslim` for `f1`,
    `Throw::nocatch` for `f2`);
  - `General::cancel` appears exactly once.

  At HEAD both fail on the message: the messages are `["General::cancel"]`.
- **Controls, green at HEAD.**
  - `k1` completes the same `REC` cleanup without cancelling: `StackLimit`,
    `["General::reslim"]`, and the outer cleanup ran.
  - `k2` completes the same `Throw[630]` cleanup: a `HostFailure` with
    `["Throw::nocatch"]`, and the outer cleanup ran.
  - `k3` cancels with a cleanup that succeeds: `["General::cancel"]`, its
    `Print` once, and the outer cleanup ran.

## Stage A — the inventory, then STOP

No edit under `crates/`, `docs/` (except appending your report),
`scripts/`, or `TODO.md`. Keep all scratch in `.temp/brief-630/`.

1. **Calibrate in your environment.** Copy my probe crate to
   `.temp/brief-630/probe/` and fix its relative paths. Run it three times:
   `f1` and `f2` are red, and `k1`, `k2` and `k3` are green, each time.
   Quote the `test` lines.
2. **Routes.** Verify and complete the table above. Find every production
   call of `abandon_pending_suspension`, and every other way into
   `drain_abandonment_worklist`. For each, give:
   - file:line and the enclosing function;
   - the `AbandonReason`;
   - what starts it: a host request, a guest builtin, or a helper;
   - which request's outcome the messages pushed during that drain land in.
3. **Message channel.** For each route, say where `Session::messages`
   pushed during the drain ends up, citing path:line:
   - (a) returned in the outcome of the request being retired;
   - (b) carried into another request that is still running (an outer
     request, or the request that triggered a helper);
   - (c) rolled back by a snapshot restore: a file transaction's
     `LoaderSnapshot`, a `ValueQ` trial, or the evaluation rollback
     snapshot;
   - (d) discarded.
4. **Error kinds.** List every `EvalError` a cleanup evaluation can return.
   For each, give the message the ordinary path emits (function and tag),
   and whether that path also changes the request's termination. Two kinds
   are settled:
   - `Cancelled` must not add a second `General::cancel`;
   - `InputRequired` already requeues the nested suspension.

   Say what `handled_non_loader_eval_errors` and its checkpoint do, and
   whether the checkpoint is needed here.
5. **Order.** On each `CancelledRequest` route, where is `General::cancel`
   emitted relative to the drain? Where would the cleanup's message fall?
6. **Files.** Write probes for a cancelled request whose drain crosses a
   nested `Get` and a public `--file` (the CLI, or the host's file API),
   with a failing cleanup inside the file. Today's messages and the guide's
   "rolled back with their file" tell us what the new message must do. Probe
   both orders: cleanup inside the file, and file inside the cleanup's
   `WithCleanup`.
7. **Policy.** For each `AbandonReason`, propose *emit* or *do not emit*,
   with the reason from steps 2–3. A message that would land in an
   unrelated request's outcome is a leak. A message that reaches no outcome
   at all is the silence Option A removes.
8. **Shape for Stage B, as code.** The policy must be decided by the
   compiler: an exhaustive `match` on `AbandonReason` with no wildcard, so
   that a new reason fails to compile until it takes a stance. Show the
   arm in `abandon.rs` as you would write it, and the function that holds
   the policy. Say which `EvalError` kinds the arm reports, and through
   which existing function.
9. **What Stage B will touch.**
   - Standing inventories: the census asserts three RET002 rows in
     `abandon.rs` (lines 43, 44, 94) and will move; the fingerprints;
     `frame_root_trace`; the ownership rows.
   - The cost surface: abandonment is not the evaluation hot path, but the
     resume panel holds cleanup labels (`resume-cleanup`, `withcleanup-init`,
     `resume-s9-cleanup-init`, `c1-all-guard`). The brief-629 instruments in
     `.temp/brief-629/` (`instruments/instr.py`, `path-calibration.sh`, the
     one-path arms) are what Stage B will use. Name which labels drain a
     cleanup, from the label source.
10. **Falsifier drafts.** Write one falsifier for each route the policy
    says emits, and one control for each route that must not. Calibrate each
    red or green at HEAD in your probe crate, three runs. Put the code in
    the report, not in the repository.

Write the report and STOP. Do not start Stage B.

## Stage B — implementation (only in a continuation, after my ruling)

Stage B follows my ruling on Stage A, in a continuation appended here.

## Scope

- **Stage A MAY modify:** nothing under `crates/`, `scripts/`, `docs/`
  (except appending your report to this brief), `TODO.md`, `Cargo.lock` or
  `.cargo/`. Scratch goes in `.temp/brief-630/`. Do not edit
  `.temp/director/`. Copy from it.
- No git commands.

## Concurrency plan

- One cargo command at a time, `CARGO_BUILD_JOBS=8`, inside your 16G scope.
- Tests run with `RUST_TEST_THREADS=1`.
- Give every command a bound with `scripts/run-bounded.py`, sized from a
  measured run.

## Stop-loss

If a probe you write does not compile or does not reach the code it names
after three attempts, record it in the report and move on. A probe whose
control does not fire first cannot be read (CLAUDE.md, "a limit probe's
control must fire before its treatment is read").

## Report

Append "Executing-agent report — Stage A" below. Every quoted line must be
copied from a file you name. Quote every verdict line your own instruments
print, and settle each flagged row in the text.

1. The calibration of my probes in your environment (step 1).
2. The routes table (step 2) and the channel per route (step 3), with
   path:line.
3. The error kinds (step 4) and the order (step 5).
4. The file probes and their results (step 6).
5. The policy per `AbandonReason` (step 7).
6. The Stage B shape as code (step 8), and what it touches (step 9).
7. The falsifier and control drafts, with their calibration (step 10).
8. Every attempt with its result, and the open questions.

Keep `.temp/brief-630/PROGRESS.md` current.

## Executing-agent report — Stage A

Worker: grk. Stage A only. No edit under `crates/`, `scripts/`, `TODO.md`, `Cargo.lock`, or `.cargo/`. Scratch is `.temp/brief-630/`. The director probe tree was copied, not edited. Stage B is not started.

### 1. Calibration of the director probes

Copied `.temp/director/630/probe/` to `.temp/brief-630/probe/` and changed the three crate paths from `../../../../crates` to `../../../crates`. Three runs, `cargo test --offline`, `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=1`.

`.temp/brief-630/logs/calibration-1.log` (cold, `Finished test profile in 19.90s`, tests `0.04s`):

```text
test f1_cancelled_cleanup_resource_limit_reports_message ... FAILED
test f2_cancelled_cleanup_uncaught_throw_reports_message ... FAILED
test k1_completed_cleanup_resource_limit_reports_message ... ok
test k2_completed_cleanup_uncaught_throw_reports_message ... ok
test k3_cancelled_cleanup_that_succeeds_runs_once ... ok
DIR630 f1: value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["General::cancel"] prints=[]
thread 'f1_cancelled_cleanup_resource_limit_reports_message' panicked at src/lib.rs:54:5:
["General::cancel"]
DIR630 f2: value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["General::cancel"] prints=[]
thread 'f2_cancelled_cleanup_uncaught_throw_reports_message' panicked at src/lib.rs:65:5:
["General::cancel"]
test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

`.temp/brief-630/logs/calibration-2.log` (`Finished ... in 0.03s`, `EXIT2:101`):

```text
test f1_cancelled_cleanup_resource_limit_reports_message ... FAILED
test f2_cancelled_cleanup_uncaught_throw_reports_message ... FAILED
test k1_completed_cleanup_resource_limit_reports_message ... ok
test k2_completed_cleanup_uncaught_throw_reports_message ... ok
test k3_cancelled_cleanup_that_succeeds_runs_once ... ok
test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

`.temp/brief-630/logs/calibration-3.log` (`Finished ... in 0.02s`, `EXIT3:101`):

```text
test f1_cancelled_cleanup_resource_limit_reports_message ... FAILED
test f2_cancelled_cleanup_uncaught_throw_reports_message ... FAILED
test k1_completed_cleanup_resource_limit_reports_message ... ok
test k2_completed_cleanup_uncaught_throw_reports_message ... ok
test k3_cancelled_cleanup_that_succeeds_runs_once ... ok
test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Settled: f1 and f2 are red because the cleanup message is absent and the only message is `General::cancel`. k1, k2, and k3 are green. The same f1/f2 lines repeat in the later suite logs at `src/lib.rs:64` and `:75` after the file gained imports. Same asserts.

### 2. Routes and channels

`drain_abandonment_worklist` is called from `abandon_pending_suspension` (`abandon.rs:22`) and from `unwind_continuation_fault` (`fault.rs:126`). While `abandonment_active` is set, `drain_abandonment_worklist` returns immediately (`abandon.rs:26-28`) after the caller has already pushed the new suspension (`abandon.rs:18`). That is how a nested `InputRequired` during cleanup rejoins the outer drain. It is not a separate route.

Direct `abandon_pending_suspension` calls under `#[cfg(test)]` (call-preparation, value-table assignment, iteration, memory tests, and the rest of the test modules) are not production routes.

| file:line | function | reason | starter | channel |
|---|---|---|---|---|
| `session_requests.rs:63` | `finish_cancelled_pending_evaluation_request` | `CancelledRequest` | host request (`eval`, `eval_expr`, `eval_script`, `eval_file`, `call`, and resume when the handle is already cancelled) | (a) |
| `session_requests.rs:581` | `finish_accepted_value_request` (file requests delegate here at `:607`) | `CancelledRequest` | host request whose result is already `InputRequired` and whose cancel flag is already set | (a) |
| `session_helpers.rs:249` | `restore_evaluation_rollback_snapshot` | `CheckpointRollbackDiscard` | guest `ValueQ` trial (`builtins_messages_inspect.rs:114`) | (c) |
| `session_api.rs:102` | `restore_snapshot_request` | `FailedSnapshotRestore` | host `restore_from_bytes` (`rum-host` `session.rs:219`, which resets, then calls this) | (d) |
| `builtins_input.rs:234` | `drive_input_request` | `SupersededOrQueuedInputDiscard` | guest `Input` / `InputString` when a suspension is already pending and the queue is non-empty | (b) if a request is still evaluating; the function itself returns the queued string and packages nothing |
| `accounting_mode/bridge.rs:252`, `frozen.rs:121`, `frozen.rs:153`, `options.rs:219` | retention helpers | `HelperBestEffortDiscard` | memory admission while a suspension exists, inside a running evaluation | (b) |
| `debug_tools/monitor.rs:85` and `:122` | `run_monitor_callback_best_effort`, `refresh_monitor_scope_display` | `HelperBestEffortDiscard` | display or monitor callback during a running `Monitor` | (b) |
| `suspension/core/fault.rs:126` | `unwind_continuation_fault` | `ContinuationSlotMismatch` | slot fault on resume. `record_slot_fault` is defined at `fault.rs:64`. Callers: `evaluation_context.rs:77`, `driver.rs:33`, `:68`, `:113`. `core.rs:719` is `cfg(test)`. | (a) |

Channel evidence:

- (a) `finish_cancelled_eval_response` (`session_requests.rs:702-736`) calls `finalize_top_level_eval_value(Err(Cancelled), …)` and then `split_accounted_messages` (`diagnostics.rs:28-33`). The slice is the success outcome. The slot-fault resume returns the fault to `finish_request_failure` (`session_requests.rs:503-529`), which slices the same request's messages into the failure.
- (b) The helper returns to the evaluation that called it. That evaluation is still the accepted request, and its finisher slices `session.messages` from the request checkpoint. A message pushed during the helper's drain is in that slice.
- (c) `LoaderSnapshot` stores `messages` (`session_state.rs` capture at `session_helpers.rs:125-126`). `restore_loader_snapshot` writes them back (`session_helpers.rs:203-205`). Guest close does that at `guest_loading/file.rs:135`. Public close does that at `batch_file.rs:65`. `restore_evaluation_rollback_snapshot` abandons first (`session_helpers.rs:249`) and then calls `restore_loader_snapshot` (`:267`). `ValueQ` captures the snapshot before the trial and restores it after (`builtins_messages_inspect.rs:93-114`). The print sink is put back at `:110`, before the restore, so a cleanup `Print` reaches the sink; the message list does not survive the restore.
- (d) `restore_snapshot_request` (`session_api.rs:97-108`) abandons only after `eval_snapshot_restore_request` has returned `InputRequired`. The failure value, including its `messages`, is the one already built. The function then replaces `failure.error` with `Other { "snapshot restore unexpectedly requested input" }` and returns that same failure. A push during the drain is not copied into it. `last_message_list` was set by the input finisher before the abandon.

`eval_file_request` does not store `pending_eval_request`. `finish_completed_request_failure` (`session_requests.rs:532-556`) copies messages and does not record a resume checkpoint. `batch_boundary.rs:61-64` states the batch has no resume checkpoint. Resume before cancel therefore fails with `no suspended evaluation is waiting for host input` (`session_requests.rs:278-288`). Resume after cancel enters `finish_cancelled_pending_evaluation_request`, whose checkpoint is `RequestCheckpoint::new` (`session_requests.rs:56-59`) taken at the current length, before the drain.

### 3. Error kinds and cancel order

A cleanup `evaluate` can return every `EvalError` variant (`session_state.rs:499-575`). The ordinary top-level messages are `emit_non_loader_eval_message` (`session_helpers.rs:478`) and `emit_top_level_control_flow_messages` (`session_helpers.rs:501`). `finalize_top_level_eval_value` (`session_eval.rs:37-60`) emits the non-loader message for kinds it turns into a value. The `Err` branch of top-level eval (`session_eval.rs:279-280`) emits both for everything else.

`runtime_message_for_non_loader_eval_error` tags: `EvaluationCycleDetected` and `EvaluationPassLimitExceeded` → `General::passlim`; `RecursionLimitExceeded` → `General::reclim`; `MatchRecursionLimitExceeded` → `General::matchreclim`; `Cancelled` → `General::cancel`; `TimeConstraintExceeded` → `General::timelim`; `MemoryConstraintExceeded` → `General::memlim`; `StackLimitExceeded`, `IntegerWorkLimitExceeded`, `GeneratedElementLimitExceeded`, `IterationWorkLimitExceeded` → `General::reslim`; `IterationLimitExceeded` → `General::iterlim`; `ExceptionThrown` → `ThrowException::nocatch`. Every other variant returns `None`.

`emit_top_level_control_flow_messages` adds `Throw::nocatch` for `Thrown`, two of those for `ThrownBypassingCatch` (cleanup throw, then primary), `Break::nofwd`, and `Continue::nofwd`.

Ordinary termination, from `terminated_evaluation_reason_for_top_level_error` (`session_helpers.rs:458-464`) plus `finalize_top_level_control_flow` (`session_eval.rs:25-34`):

- Limit kinds above become `TerminatedEvaluation` and a success, except `MemoryConstraintExceeded`, which becomes `MemoryLimit` only on the top-level map.
- `Return` becomes `Ok(value)`. `WithCleanup` also treats a cleanup `Return` as `Ok` and keeps the primary (`cleanup.rs:135-137`).
- `Abort` and `ProtectedAbort` become `Ok($Aborted)`.
- `ExceptionThrown` becomes a `Failure` value, success, termination `None`, plus `ThrowException::nocatch`.
- `Thrown`, `ThrownBypassingCatch`, `Break`, `Continue`, `Exit`, `Syntax`, `InvalidInteger`, `Io`, `NoFileIoProvider`, `LoadPathNotFound`, and `ContinuationSlotMismatch` are not termination values. They become request failures. Loader errors have their own emitters (`emit_loader_message`); the two non-loader functions add nothing for them. `Exit` changes the process status through `SessionRequestError::Exit`.
- `InputRequired` has no message and suspends.

On a `CancelledRequest` abandon the request stays `Cancelled`. The arm adds a message only. It does not return the cleanup error, so it does not turn the request into that error's failure or into `$Aborted`.

`Cancelled` must not be emitted by the arm. `finish_cancelled_eval_response` already finalizes `Err(Cancelled)`, which emits the one `General::cancel`. `InputRequired` stays the existing requeue (`abandon.rs:100-109`).

`handled_non_loader_eval_errors` is the dedup record. `emit_non_loader_eval_message` returns without a second push when an equal message (symbol, tag, and text) is already in `handled[checkpoint..]` (`session_helpers.rs:487-492`). `Check` records via that emit and, on a matched `Ok`, truncates the record (`check_quiet.rs:118-119`). `Quiet` emits, then truncates the visible messages (`check_quiet.rs:157-168`), and leaves the handled record in place when the outcome is still `Err` (`check_quiet.rs:171-176`). The checkpoint has to be `handled_non_loader_eval_errors.len()` immediately before `evaluate(cleanup)`. A checkpoint taken after `evaluate` would miss Quiet's record and push the message Quiet just removed. `abandon_pending_suspension` clears `pending_eval_request` before the drain (`abandon.rs:10`), so the request checkpoint is not in scope inside the arm. The local length is the right window: on cancel the primary is suspended, not already terminated by the cleanup's error. `finish_cancelled_eval_response` truncates the handled list back to the request checkpoint after it emits (`session_requests.rs:719-720`).

Order on both `CancelledRequest` routes: the drain runs first (`session_requests.rs:63` and `:581`), then `finish_cancelled_eval_response` emits `General::cancel` and then splits. A cleanup message pushed during the drain is therefore before `General::cancel`. `append_cancel_message` (`session_requests.rs:739-751`) is the other dedup, used by `force_cancelled_*` when a request completed and was cancelled. That path does not drain. The outside-file probe shows the order directly: `messages=["probe::outside", "General::cancel"]`.

### 4. File probes

Full source: `.temp/brief-630/probe/src/lib.rs`. Quoted lines are from `.temp/brief-630/logs/suite-1.log`. suite-2 and suite-3 repeat them; their result lines are in section 7.

Frames are `push_back` on insert (`core.rs:549`) and `pop_front` on abandon (`core.rs:204-205`), so the first inserted frame is abandoned first. A cleanup frame inserted when `InputString` suspends is in front of the file frame that was inserted after it. `EvaluateCleanup` pushes the remainder back and breaks (`abandon.rs:92-113`); the drain then abandons the file frame, and `restore_loader_snapshot` replaces `messages`. `Print` is not in `LoaderSnapshot`. The guide's rollback sentence holds for `Message` and for an error message emitted in the arm before that later frame.

Nested `Get` from `eval` (the request has `pending_eval_request`):

- Cleanup inside the file. Print proves the cleanup ran. The message and `fileRan630 = 1` are rolled back.

```text
DIR630 k-get-inside: Success value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["General::cancel"] prints=["inside-cleanup"]
test k_get_cleanup_inside_file_rolls_message_back ... ok
```

- File inside the cleanup's `WithCleanup`. The inner file sets `fileRan630 = 1` before `InputString` and that assignment is rolled back. The outer cleanup runs after the file frame. Its `Message` survives, then `General::cancel`. `Throw::nocatch` is absent. This is a red falsifier (section 7).

Public file, host `eval_file` (no resume checkpoint):

- Cleanup inside the public file. Same shape as the inner `Get`: print survives, message and `marker630` roll back, one `General::cancel`.

```text
DIR630 k-public-inside: Success value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["General::cancel"] prints=["pub-inside"]
test k_public_file_cleanup_inside_rolls_back ... ok
```

- `WithCleanup[Get["Inner`"], cleanup]` as the public file body. The cleanup is still inside the public transaction, so `outer630 = 1` and the message roll back. The print survives.

```text
DIR630 k-public-around-get: Success value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["General::cancel"] prints=["pub-outside"]
test k_public_file_cleanup_around_get_rolls_back ... ok
```

A public file that `Message`s before `InputString` is a separate hazard. The script loop splices earlier forms' messages back on `InputRequired` (`session_eval.rs:360-364`). The cancel checkpoint is the length after that splice. Rollback restores the pre-file list. `finish_cancelled` then appends `General::cancel` and `split_off`s at the checkpoint (`diagnostics.rs:33`).

One message: the append brings the length back to the checkpoint, the split is empty, and the outcome loses both the early message and `General::cancel`. `$MessageList` is `List[]`.

```text
DIR630 k-public-early-first: InputRequired prompt="in" messages=["probe::early:early message"] prints=[]
DIR630 k-public-early-resume: Success value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=[] prints=[]
DIR630 k-public-early-list: List[]
test k_public_file_message_before_input_on_cancel ... ok
```

Two messages: `split_off` panics. The host catches it.

```text
DIR630 k-public-two-first: InputRequired prompt="in" messages=["probe::early:early message", "probe::later:later message"] prints=[]
thread 'k_public_file_two_messages_before_input_on_cancel' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:33:37:
`at` split index (is 2) should be <= len (is 1)
DIR630 k-public-two-resume: Failure code=Panic message="host session caught panic: `at` split index (is 2) should be <= len (is 1)" messages=[] prints=[]
test k_public_file_two_messages_before_input_on_cancel ... ok
```

Settled: the `ok` on the two-message test is the probe locking that panic in. It is not a clean cancel. The panic is pre-existing (the messages exist before the drain). A cleanup message emitted during the drain and then removed by the public-file frame does not change this arithmetic. The file probes that emit `Message` only inside the cleanup, after `InputString`, stay on the non-panic path and show one `General::cancel`.

What the new message must do: follow the file snapshot. Inside a file, it is rolled back, as `probe::inside` and `probe::outside` already are. Outside a file, it survives and sits before `General::cancel`, as `probe::outside` already does.

### 5. Policy

Emit only where the cleanup message reaches the retired request's outcome and a later frame does not roll it back. A message in some other request's outcome is a leak. A message that no outcome returns is the silence Option A removes; emitting it only to leave residue is not Option A.

| `AbandonReason` | stance | why |
|---|---|---|
| `CancelledRequest` | emit | Channel (a). Both production sites. The outcome stays `Cancelled`. File frames on the same drain still roll the message back. Do not emit `EvalError::Cancelled`. Do not emit `InputRequired`; requeue it. |
| `ContinuationSlotMismatch` | emit | Channel (a), the same request's slot-mismatch failure. Same two exceptions. No host falsifier: `record_slot_fault` is `pub(super)` and `FaultProbe` is `cfg(any(test, feature = "test-continuation-slots"))`. |
| `FailedSnapshotRestore` | do not emit | Channel (d). The failure is sealed before the drain. The probe's `marker630` is `1` and the failure messages stay empty. |
| `CheckpointRollbackDiscard` | do not emit | Channel (c). `ValueQ` restores the snapshot after the drain. Ordinary `ValueQ` also drops the trial error (`Err(_) => False`, `InputRequired => True`, `builtins_messages_inspect.rs:81-85`). |
| `SupersededOrQueuedInputDiscard` | do not emit | The new `InputString` is the continuing evaluation. A cleanup message would land in that evaluation, which is not the retired suspension. From the host the route does not run: a second `eval` while suspended is rejected. |
| `HelperBestEffortDiscard` | do not emit | Channel (b). `Monitor` already drops callback errors with no message (`monitor.rs:82-88` and `:120-125`). The step probe's success would gain `Throw::nocatch` if the arm emitted. Accounting sites use the same reason; their controls did not fire (section 7). |

### 6. Stage B shape, and what it touches

Policy function, exhaustive, no wildcard. Place it below the three census sites so those line numbers stay.

```rust
fn abandonment_reports_cleanup_error(reason: AbandonReason) -> bool {
    match reason {
        AbandonReason::CancelledRequest | AbandonReason::ContinuationSlotMismatch => true,
        AbandonReason::FailedSnapshotRestore
        | AbandonReason::CheckpointRollbackDiscard
        | AbandonReason::SupersededOrQueuedInputDiscard
        | AbandonReason::HelperBestEffortDiscard => false,
    }
}
```

Arm, with the `InputRequired` requeue left as it is. The checkpoint is taken before `evaluate`. The emitters are the ordinary ones. The arm does not return the cleanup error and does not change the request value.

```rust
FrameAbandonAction::EvaluateCleanup(cleanup) => {
    if !suspension.is_empty() {
        suspension.active_stack = self.evaluation_stack.clone();
        suspension.guest_form_environment = self.capture_guest_form_environment();
        self.abandonment_worklist.push(suspension);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist,
        );
    }

    let handled_checkpoint = self.handled_non_loader_eval_errors.len();
    let cleanup_outcome = self.evaluate(cleanup.expr);
    if matches!(cleanup_outcome, Err(EvalError::InputRequired { .. })) {
        if let Some(nested) = self.take_accounted_pending_suspension_for_abandon() {
            self.abandonment_worklist.push(nested);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist,
            );
        }
        self.pending_eval_request = None;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingEvalRequest,
        );
    } else if abandonment_reports_cleanup_error(reason)
        && !matches!(cleanup_outcome, Err(EvalError::Cancelled))
    {
        if let Err(error) = &cleanup_outcome {
            self.emit_non_loader_eval_message(handled_checkpoint, error);
            self.emit_top_level_control_flow_messages(error);
        }
    }

    self.clear_accounted_evaluation_stack();
    self.evaluation_depth.restore(0);
    break;
}
```

Reported kinds: every `Err` except `InputRequired` and `Cancelled`, and only when the policy function is true. Kinds those two functions do not know (`Syntax`, `Io`, `Return`, `Abort`, `Exit`, `ContinuationSlotMismatch`, and the other `None` arms) produce no message. `Exit` therefore does not change the process status on this path; the request stays `Cancelled`. That is the open point in section 8.

Census: `rum-stdlib` `retention_closure/census/tests.rs:193-197` asserts RET002 at `abandon.rs` lines 43, 44, and 94. Those three are the guest-form take, install, and capture. Inserting only after the capture at line 94, and putting the policy function below that line, leaves 43, 44, and 94 where they are. Inserting above any of them moves it. The total remains 972 (RET001 10, RET002 934, RET003 28) only if the new calls do not add a rejected row. Stage B has to replay the census rather than assume the total.

`frame_root_trace.rs:473-518` pins the text of the driver, the drain, the profiled trampoline, and the one `fn abandon_frames<`. The shared body must contain `frame.abandon(` and the three bodies must not contain `.resume(`. No new frame type. Those pins hold if the function names stay. Fingerprints and ownership rows move on any `.rs` edit and are replayed in Stage B, not now. Stage A did not change `crates/`.

Cost, from the label source, not from a new measurement. None of these enter `drain_abandonment_worklist`:

- `resume-cleanup` is harness label `cleanup`: `WithCleanup[Do[InputString["resume"], {n}], marker = 17]` (`.temp/brief-629/harness-cand/src/main.rs:44`). The harness resumes every prompt (`main.rs:77-85`) and the cleanup runs on ordinary completion.
- `withcleanup-init` is `WithCleanup[Null, 1, Null]` (`docs/dev/codex/brief-617-g13-s9-control-and-cleanup.md:429`). No suspension.
- `resume-s9-cleanup-init` is `WithCleanup[(InputString["init"]; 1), 2, Null]` (`.temp/brief-629/harness-cand/src/s9.rs:77-81`). The resume harness answers `init`. The cleanup is the ordinary third-argument path.
- `c1-all-guard` is `ReplaceAll` with `InputString` in the condition (`.temp/brief-629/harness-cand/src/g15c1.rs:5-9`). No `WithCleanup`.

The arm is off those labels. They stay useful as a resume-path regression panel because they share `WithCleanup` resume code. Abandonment is not the evaluation hot path.

### 7. Falsifiers and controls

Source: `.temp/brief-630/probe/src/lib.rs`. Three runs with `--nocapture`: `.temp/brief-630/logs/suite-1.log` (`Finished ... in 0.02s`, tests `0.08s`, `EXIT:101`), `suite-2.log` (`Finished ... in 0.03s`, tests `0.09s`, `EXIT:101`), `suite-3.log` (`Finished ... in 0.02s`, tests `0.09s`, `EXIT:101`). The DIR630 lines and the pass/fail lines match across the three files. suite-2 and suite-3 result lines:

```text
test result: FAILED. 14 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

suite-1 is the same counts with `finished in 0.08s`. The four failures are the red falsifiers. Settled below. The other 14 tests passed on every run.

Red, `CancelledRequest` via resume (`finish_cancelled_pending`). Already f1 (`REC` cleanup, wants `General::reslim`) and f2 (`Throw[630]`, wants `Throw::nocatch`). Both keep `Cancelled`, `outerRan630` is 1, and the message list is only `General::cancel`. From suite-1:

```text
DIR630 f1: value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["General::cancel"] prints=[]
thread 'f1_cancelled_cleanup_resource_limit_reports_message' panicked at src/lib.rs:64:5:
["General::cancel"]
test f1_cancelled_cleanup_resource_limit_reports_message ... FAILED
DIR630 f2: value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["General::cancel"] prints=[]
thread 'f2_cancelled_cleanup_uncaught_throw_reports_message' panicked at src/lib.rs:75:5:
["General::cancel"]
test f2_cancelled_cleanup_uncaught_throw_reports_message ... FAILED
```

Red, the same reason when the file frame has already rolled back and the cleanup is outside the file. Program: `WithCleanup[Get["Inner`"], (outer630 = 1; Print["outside-cleanup"]; Message[probe::outside]; Throw[630])]` with `Inner.rum` = `fileRan630 = 1` then `InputString["body"]`. The panic is the `Throw::nocatch` assert at `src/lib.rs:259`, after the asserts that `outer630` is 1, `fileRan630` is 0, the outside message text is present, the print happened, and `General::cancel` appears once.

```text
DIR630 f-get-outside: Success value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["probe::outside", "General::cancel"] prints=["outside-cleanup"]
thread 'f_get_cleanup_outside_file_reports_throw' panicked at src/lib.rs:259:5:
["probe::outside", "General::cancel"]
test f_get_cleanup_outside_file_reports_throw ... FAILED
```

Red, `CancelledRequest` via `finish_accepted_value_request`. The print sink cancels on `Print["arm"]` before `InputString` returns, so `eval` itself returns the cancelled success. Program: `Print["arm"];` plus `nested("Throw[630]")`. The panic is the `Throw::nocatch` assert at `src/lib.rs:388`, after `outerRan630` is 1 and `General::cancel` appears once.

```text
DIR630 f-print-sink: Success value=TerminatedEvaluation["Cancelled"] termination=Some(Cancelled) messages=["General::cancel"] prints=[]
thread 'f_print_sink_cancel_during_eval_reports_throw' panicked at src/lib.rs:388:5:
["General::cancel"]
test f_print_sink_cancel_during_eval_reports_throw ... FAILED
```

Green controls:

- k1, k2, k3. Ordinary completion reports `General::reslim` and `Throw::nocatch`. A cleanup that only prints stays `["General::cancel"]` with that print once. `ok` on all three runs.
- Cleanup inside a nested `Get`, and both public-file orders. Section 4. Messages that are inside the file are absent; prints show the cleanup ran. `ok`.
- `CheckpointRollbackDiscard`. `ValueQ[WithCleanup[InputString["v"], (Print["valueq-cleanup"]; Throw[630])]]`. The sink sees the print. The outcome has no throw.

```text
DIR630 k-valueq: Success value=True termination=None messages=[] prints=[] sink=["valueq-cleanup"]
test k_valueq_cleanup_throw_stays_out_of_outcome ... ok
```

- `FailedSnapshotRestore`. Snapshot source `marker630 = 0; WithCleanup[InputString["snap"], (marker630 = 1; Throw[630])]`. `marker630` is 1, so the cleanup ran. The failure has no throw.

```text
DIR630 k-snapshot: code=Other message="snapshot restore unexpectedly requested input" messages=[] prints=[]
DIR630 k-snapshot-marker: 1
test k_snapshot_restore_cleanup_not_in_failure ... ok
```

- `HelperBestEffortDiscard` through `StepMonitor`. Program: `Monitor[1+2, 0, StepMonitor :> WithCleanup[InputString["s"], (markerS = 1; Throw[630])]]`. `markerS` is 1, so the callback ran. The success value is 3 and there is no throw. The print `"0"` is the display echo.

```text
DIR630 k-monitor-step: Success value=3 termination=None messages=[] prints=["0"]
DIR630 k-monitor-step-marker: 1
test k_monitor_step_cleanup_throw_stays_out ... ok
```

Controls that did not fire, so their missing throw is not evidence:

- Monitor display. `WithCleanup[InputString[…], cleanup]` as the display argument did not return. Three attempts, all killed by the bound: `probes-1.log` (`run-bounded: timed out after 150s`, Print plus Throw), `monitor-display-1.log` (`timed out after 40s`, Throw, no Print), `monitor-display-2.log` (`timed out after 25s`, `markerD = 1` only). The suite test does not call it.

```text
DIR630 k-monitor-display: SKIP display WithCleanup[InputString, cleanup] did not return (150s, 40s, 25s)
test k_monitor_display_cleanup_did_not_return ... ok
```

Settled: that `ok` is the skip. It is not a reading of `refresh_monitor_scope_display`.

- Accounting allowances. `marker630` stayed 0, or the suspension escaped to the host. The helper did not run.

```text
DIR630 k-accounting n=0: Success value=$Aborted termination=None messages=[] prints=[] marker=0
DIR630 k-accounting n=50: Success value=$Aborted termination=None messages=[] prints=[] marker=0
DIR630 k-accounting n=200: Success value=$Aborted termination=None messages=[] prints=[] marker=0
DIR630 k-accounting n=1000: InputRequired prompt="acct" messages=[] prints=[] marker=suspended
DIR630 k-accounting n=10000: InputRequired prompt="acct" messages=[] prints=[] marker=suspended
test k_accounting_cleanup_allowances ... ok
```

Settled: `ok` means the probe finished. No allowance reached `HelperBestEffortDiscard`. The `$Aborted` rows are an ordinary memory failure. The `InputRequired` rows are a suspension the host still owns.

- `SupersededOrQueuedInputDiscard`. The second `eval` is rejected. The cleanup does not run.

```text
DIR630 k-queued: Failure code=Other message="cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first" messages=[] prints=[]
test k_queued_input_does_not_reach_abandon_from_host ... ok
```

Settled: `ok` means the overlap rejection happened. The abandon reason was not entered. `queue_input_response` does not resume (`rum-host` `session.rs:575`). The in-runtime demonstration remains `suspension_abandonment.rs:49`, which calls `Session::evaluate` directly.

`ContinuationSlotMismatch` has no probe. There is no host API that injects a slot fault. A probe that called `record_slot_fault` would not compile from this crate. Stop-loss: not attempted as a failing compile.

### 8. Attempts and open questions

| # | what | result |
|---|---|---|
| 1 | cold director probe | `calibration-1.log`. f1/f2 red, k1–k3 green. `Finished ... in 19.90s`. tests `0.04s`. |
| 2 | warm director probe | `calibration-2.log`. Same verdicts. `0.03s` compile, tests `0.04s`. `EXIT2:101`. |
| 3 | warm director probe | `calibration-3.log`. Same verdicts. `0.02s` compile, tests `0.03s`. `EXIT3:101`. |
| 4 | first file/route compile | `probes-1.log` replaced by the later run's timeout text; the compile error was `HostEvalRequest: From<&String>` at `src/lib.rs:350` and `:457`. Fixed with `as_str()`. |
| 5 | suite with Print+Throw display | `probes-1.log`. `EXIT:124`. `run-bounded: timed out after 150s` while `k_monitor_display_cleanup_throw_stays_out` was running. f1, f2, f-get, f-print had already failed; k1–k3, accounting, and get-inside had passed. |
| 6 | display, Throw, no Print | `monitor-display-1.log`. `timed out after 40s`. `EXIT:124`. |
| 7 | display, `markerD = 1` only | `monitor-display-2.log`. `timed out after 25s`. `EXIT:124`. Stop-loss. Display probe skipped. |
| 8 | StepMonitor alone | `monitor-step-1.log`. `ok` in `0.01s`. |
| 9 | suite before the two-message probe | `probes-2.log`. `EXIT:101`. 12 passed, 5 failed. The fifth failure was the one-message public file, which I had wrongly expected to panic. It returns Success with empty messages. The test was then changed to lock that in. |
| 10 | one-message and two-message public files | `public-early.log`. Both `ok`. Two messages panic at `diagnostics.rs:33` and the host reports `HostFailureCode::Panic`. |
| 11–13 | final suite | `suite-1.log`, `suite-2.log`, `suite-3.log`. Each `EXIT:101`. 14 passed, 4 failed. The failures are f1, f2, f-get-outside, f-print-sink. |

Open questions for the ruling:

1. `Exit` inside a cleanup on a cancelled request. The arm above does not return it, so the process status stays the cancel status. Say if `Exit` should still win.
2. `ContinuationSlotMismatch` is emit from the source, with no calibrated host falsifier.
3. `SupersededOrQueuedInputDiscard` and the accounting `HelperBestEffortDiscard` sites were not reached from `HostSession`. The stance is do not emit, from the channel, not from a firing control.
4. The public-file cancel checkpoint. One `Message` before `InputString` drops `General::cancel` from the outcome. Two panic in `split_accounted_messages`. This is independent of the cleanup arm. Stage B as written does not change it. Say if it is in scope.
5. The monitor display `WithCleanup` did not return within the bounds above. The step callback is the firing control for that reason.

Stage B is not started.

## Director ruling on Stage A, and continuation A2 (2026-10-28 09:55) — grk

**Stage A is accepted, and it found two defects I did not ask about.** I
reran your suite from a copy (`.temp/director/gate-630a/suite-1.log`): 14
passed and 4 failed, the four red falsifiers you name, and the two-message
public file panics at `diagnostics.rs:33`. The probes are good. Each control
was shown to fire before its treatment was read, and you said plainly which
controls did not fire.

**The policy is accepted.** `CancelledRequest` and `ContinuationSlotMismatch`
emit. `FailedSnapshotRestore`, `CheckpointRollbackDiscard`,
`SupersededOrQueuedInputDiscard` and `HelperBestEffortDiscard` do not. The
stance is held by an exhaustive `match` with no wildcard. The policy is per
reason, not per site. So one firing control per reason tests the arm for
every site of that reason, and the `StepMonitor` control covers the
accounting sites too. Stage B adds two in-crate tests:
- a firing control for `SupersededOrQueuedInputDiscard`, in the style of
  `suspension_abandonment.rs:49`;
- a falsifier for `ContinuationSlotMismatch`, through the fault probe
  (`cfg(test)` or the `test-continuation-slots` feature), red before the
  arm and green after.

**Your open questions.**
1. **`Exit` in a cleanup on a cancelled request.** The request stays
   `Cancelled` with the cancel status, as Option A says. A dropped `Exit` is
   still the silence Option A removes, so it is reported. Propose the
   message in A2 (symbol, tag, text), registered by the message-tag
   registry's rules.
2. **`ContinuationSlotMismatch`.** It emits, with the in-crate falsifier
   above.
3. **The unreached routes.** Accepted, by the per-reason argument above,
   with the in-crate control for `SupersededOrQueuedInputDiscard`.
4. **The public-file cancel checkpoint is in this line.** It is the window
   that the retired request's messages are cut from, and Option A's
   messages land in that window. It is A2 part 1.
5. **The monitor display is a defect, not a probe that failed.** I probed
   it (`.temp/director/gate-630a/`):
   - `markerD = 0; Monitor[1+2, WithCleanup[InputString["d"], markerD = markerD + 1; Print[markerD]]]`
     under the release `rum-cli` printed 173 591 lines in 3 s. That is about
     58k cleanups per second, forever.
   - Resident memory stays flat at about 21 MB.
   - A host `cancel()` does not stop it within 10 s. `eval_with_timeout`
     at 1 s does not stop it within 15 s.
   - `Monitor[1+2, InputString["d"]]` returns `3` at once.
   - `f630[] := WithCleanup[InputString["x"], n630 = n630 + 1; Print[n630]; f630[]]; f630[]`
     under the CLI stops after 340 cleanups and exits 1.

   So a one-line guest program makes a session that neither cancellation
   nor the deadline can stop. It needs `InputString`, which is
   `Capability::Interactive` and is allowed only on the unrestricted
   surface. The sandboxed profiles cannot reach it; the CLI and Jupyter
   can. It is A2 part 2.

### A2 — two inventories, no source change, then STOP

Same fence as Stage A. Scratch goes in `.temp/brief-630/`. Copy from
`.temp/director/`; do not edit it.

**Part 1 — the request window and the rollbacks inside it.**
1. List every field of `RequestCheckpoint`: messages, prints, trace,
   handled errors, and anything else. For every request route, say where the
   checkpoint is taken. The routes are `eval`, `eval_expr`, the stdin script,
   `eval_file`, `call`, resume, `finish_accepted_value_request`, and the
   slot-fault resume.
2. List every operation between taking a checkpoint and cutting at it that
   can shorten the list it indexes. Include `restore_loader_snapshot` and
   say which fields it replaces, the evaluation rollback snapshot, `ValueQ`,
   the script loop's splice, and the guest-form environment restore. Cite
   path:line.
3. Probe the matrix, three runs each:
   - routes: `eval_file`; the stdin script with earlier-form messages and a
     nested `Get` in the suspended form; `eval` with a `Get`; nested files;
   - earlier messages: 0, 1 and 2;
   - lists: messages, trace (with tracing on), and handled errors (an
     earlier form's error recorded by `Check`).

   Record which cases panic, which lose `General::cancel`, and which lose or
   repeat a message.
4. From the guide, state what each case's outcome must contain. "Messages
   already reported with the input requirement remain visible" and "rolled
   back with their file" are the two rules.
5. Propose the fix as code. Prefer a shape in which an index into a list
   cannot outlive a restore of that list, by construction or by type, over a
   clamp that each cut site must remember.

**Part 2 — the drain must terminate.**
1. Reproduce my five observations above. Give the programs, the bounds you
   used, and the results.
2. Give the mechanism, with path:line:
   - which path runs the display with callbacks enabled;
   - how each new suspension reaches the worklist (`abandon.rs:26-28`);
   - which errors are dropped (`monitor.rs:82-88` and `:117-122`, and the
     drain);
   - why neither cancellation nor the deadline ends it. Look at
     `poll_host_interrupt_if_due` (`eval_core/mod.rs:121-135`) and at the
     guest-form environment, which saves and restores
     `interrupt_poll_counter`, `interrupt_armed` and `eval_deadline`
     (`session/guest_loading/scope.rs:55-90`).
   - what bounds the self-recursive cleanup at 340.
3. Inventory every place that evaluates guest code from inside a helper or
   in response to evaluation events: the `Monitor` display, step and
   evaluation monitors, any other builtin's `StepMonitor` or
   `EvaluationMonitor` option, trace hooks, and anything else you find. Say
   whether each can run during a drain. List every loop on the drain path
   with its exit condition.
4. State the invariant, "every drain terminates, and a host cancel or
   deadline ends it within a bounded amount of work", and propose the fix as
   code. It must keep the guide's "runs guest cleanup once" for each
   `WithCleanup` that was live when the drain began.
5. Write falsifier drafts that run the program on a thread with a bound,
   and controls, calibrated red or green at HEAD, three runs each.

**Part 3 — close the error-kind table.** For every `EvalError` variant,
say whether a cancelled cleanup's failure leaves a message under the arm as
drafted in Stage A. The question is whether the ordinary path's message is
pushed inside `evaluate`, so it survives, or only at the request boundary,
so the arm must emit it. Loader errors (`emit_loader_message`) are the case
to settle. List every variant that would be silent where its ordinary path
is not. Propose the `Exit` message (question 1).

Write "Executing-agent report — A2" and STOP. The Stage B steps come after
my ruling, one mechanism per step: the window, the drain's termination, and
then the cleanup-error arm. Every quoted line must be copied from a file
you name. Quote every verdict line your instruments print, and settle each
flagged row.
