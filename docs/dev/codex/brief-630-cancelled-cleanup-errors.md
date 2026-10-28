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

## Executing-agent report — A2

No source change. Nothing under `crates/`, `scripts/`, `docs/` other than this appendix, `TODO.md`, `Cargo.lock`, or `.cargo/` was edited. `.temp/director/` was not edited. Stage B is not started.

Probes are debug `HostSession` tests in `.temp/brief-630/probe/` (`src/a2_window.rs`, `src/a2_drain.rs`), run with `cargo test --offline` through `scripts/run-bounded.py`. This run did not rebuild the release `rum-cli`. The director's release figures (173 591 lines in 3 s, about 58k/s, RSS about 21 MB, and the CLI exiting 1) stay the director's measurements. The numbers below are this run's logs.

### Part 1 — the request window

#### 1. `RequestCheckpoint`

`crates/rum-runtime/src/runtime/session/session_requests.rs:4-19`. Four fields, nothing else: `message_checkpoint`, `print_checkpoint`, `handled_error_checkpoint`, `trace_checkpoint`. `new` stores `session.messages.len()`, `prints.len()`, `handled_non_loader_eval_errors.len()`, and `trace_events.len()`. It converts both ways with `PendingEvalRequest` (`:22-41`).

Where each route takes it:

- `lower_request` (`:142`) takes one for a lowering failure. It does not suspend.
- `eval_request` (`:151-172`) calls `finish_cancelled_pending_evaluation_request` first, then `begin_top_level_interrupt_budget`, then `RequestCheckpoint::new` at `:160`. That mark is passed to `eval_expr_request_from_checkpoint`. On `InputRequired` the same mark is stored by `finish_input_required_request` (`:238`, `:468-472`).
- `eval_expr_request` (`:179-194`) is the same shape. The mark is `:188`.
- `call` (`crates/rum-host/src/session.rs:442-485`) calls `finish_cancelled_pending_evaluation_request`, then `eval_expr_request`. A fresh call's mark is the `eval_expr_request` mark. A call that finds a cancelled suspension retires that suspension before building the new expression.
- stdin script: `eval_script_request` (`:379-399`) takes `RequestCheckpoint::new` at `:387` after the overlap check. It does not call `begin_top_level_interrupt_budget`; each form's `eval_input_with_terminated_handling` does (`session_eval.rs:219`). On `Err`, including `InputRequired`, it calls `finish_completed_request_failure` (`:397`, `:532-556`). That copies messages, prints, and trace and truncates handled errors. It does not store `pending_eval_request`.
- `eval_file_request` (`:412-442`) takes the mark at `:420` and on `Err` also uses `finish_completed_request_failure`. It does not store `pending_eval_request`. `finish_accepted_file_request` (`:596-608`) sends an `InputRequired` result through `finish_accepted_value_request`.
- resume: `resume_eval_request_with_input` (`:269-289`) takes `pending_eval_request` and converts it. If it is missing, the error is `no suspended evaluation is waiting for host input`.
- `finish_accepted_value_request` (`:559-585`), and `finish_cancelled_pending_evaluation_request` (`:56-59`), use `pending_eval_request` when it is set. Otherwise they call `RequestCheckpoint::new` at that moment. They then `take_active_accepted_request`, `abandon_pending_suspension(CancelledRequest)`, and `finish_cancelled_eval_response`.
- slot-fault resume: `resume_eval_request_from_checkpoint` (`:339-354`) sees `has_continuation_fault`, replaces the outcome with `continuation_fault_outcome`, and cuts with `finish_request_failure` on the checkpoint resume already took. The drain runs inside `unwind_continuation_fault` (`suspension/core/fault.rs:99-126`), which clears `pending_eval_request` at `:116` and calls `drain_abandonment_worklist(ContinuationSlotMismatch)` at `:126`. The cut still uses the earlier mark.

`finish_cancelled_eval_response` (`:702-736`) emits `General::cancel` through `finalize_top_level_eval_value(Err(Cancelled))`, then `split_accounted_messages` (`diagnostics.rs:28-33`, `split_off`), `split_accounted_prints` (`diagnostics.rs:68`), and `trace_events[checkpoint.trace_checkpoint..]` (`session_requests.rs:714`). `Vec::truncate` on the handled list does not panic when the index is past `len`.

`eval` and `eval_expr` store the admission mark in `pending_eval_request` before any in-request messages. A `LoaderSnapshot` taken after that admission has `len >=` the mark, so a later restore stays at or above it and `split_off` is defined. `eval_file` and `eval_script` do not store that mark. Cancellation then builds `RequestCheckpoint::new` from the lengths after the body and before the rollback.

#### 2. What can shorten a list between the mark and the cut

- `restore_loader_snapshot` (`session_helpers.rs:130-208`) replaces `messages` (`:203`), `last_message_list` (`:205`), `trace_events` (`:193`), and `trace_event_depths` (`:195`), plus the definition and context fields captured at `:97-127`. It does not replace `prints` or `handled_non_loader_eval_errors`.
- `restore_evaluation_rollback_snapshot` (`:239-316`) may first `abandon_pending_suspension(CheckpointRollbackDiscard)` (`:249`), then `restore_loader_snapshot` (`:267`). It also replaces `prints` (`:282`), `handled_non_loader_eval_errors` (`:284`), and `pending_eval_request` (`:307`).
- `ValueQ` is the production caller (`messages/builtins_messages_inspect.rs:90-114`). It takes the print sink off before the trial (`:95`) and restores the snapshot at `:114`.
- The script loop (`session_eval.rs:360-376`) on a later form's `Err` splices the earlier forms' messages and prints back at the script checkpoint. It inserts. It does not splice trace. A snapshot taken while those messages were split out of `session.messages` does not contain them, so a later restore deletes the spliced copy.
- `rollback_capture_output` (`suspension/output.rs:71-86`) truncates prints, messages, and trace to a frame `OutputBoundary`. `abandon_frames` calls it before each frame (`abandon.rs:83`).
- Guest-file close with rollback (`guest_loading/file.rs:117-135`) calls `restore_loader_snapshot`. Public-file close does the same (`batch_file.rs:64-66`), from `rollback_public_file` (`suspension/core.rs:327-334`).
- `GuestFormEnvironment::install` (`guest_loading/scope.rs:126-136`) replaces `last_message_list`, `handled_non_loader_eval_errors`, `eval_deadline`, `current_time_deadline`, `current_memory_limit`, `interrupt_poll_counter`, and `interrupt_armed`. `GuestFormScope::restore` (`:182-191`) truncates `handled_non_loader_eval_errors` to the form checkpoint. It does not truncate messages or prints.
- `Quiet` with no selectors truncates `messages` (`check_quiet.rs:165`). `Check` on a matched `Ok` truncates the handled list (`:118-119`).

#### 3. Matrix

One debug `HostSession` per cell. The handle is taken before the call. Retirement is `cancel()` then `eval("")`, the CLI re-entry. Routes: `eval_file` of `Outer.rum`; `eval_script` of the prefix plus `Get["Inner`"]` whose file is only `InputString`; `eval` of `Get["Inner`"]` whose file is the prefix plus `InputString`; nested `eval_file` of `Outer.rum` = `Get["Inner`"]` of that same body. Earlier entries 0, 1, 2. Kinds: `Message[probe::eN]`, `1 + 1` with tracing on, and `Check[Block[{$RecursionLimit = 4}, h630[] := h630[]; h630[]], 0]`. That `Check` records `General::passlim`. The host recursion limit is unchanged. The `again=` tag is a second `Check` in a new request after retirement, and only when retirement did not panic.

The `A2WINDOW` lines and the panic-location lines are the same text in `.temp/brief-630/logs/a2-window-1.log`, `a2-window-2.log`, and `a2-window-3.log` (36 `A2WINDOW` lines, 57 lines with the panic locations). Quoted once from `a2-window-1.log`:

```text
test a2_window::a2_window_matrix ... A2WINDOW route=file n=0 kind=messages suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=file n=0 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=file n=0 kind=handled suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0 again=["General::passlim"]
A2WINDOW route=file n=1 kind=messages suspended=true first=InputRequired prompt="in" tags=["probe::e0"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=[] cancel=0 trace=0 prints=0 lost-cancel

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_requests.rs:714:38:
range start index 3 out of range for slice of length 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
A2WINDOW route=file n=1 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=1 prints=0 retire=Failure code=Panic msg="host session caught panic: range start index 3 out of range for slice of length 2" tags=[] trace=0 panic
A2WINDOW route=file n=1 kind=handled suspended=true first=InputRequired prompt="in" tags=["General::passlim"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=[] cancel=0 trace=0 prints=0 lost-cancel again=["General::passlim"]

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:33:37:
`at` split index (is 2) should be <= len (is 1)
A2WINDOW route=file n=2 kind=messages suspended=true first=InputRequired prompt="in" tags=["probe::e0", "probe::e1"] trace=0 prints=0 retire=Failure code=Panic msg="host session caught panic: `at` split index (is 2) should be <= len (is 1)" tags=[] trace=0 panic

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_requests.rs:714:38:
range start index 4 out of range for slice of length 2
A2WINDOW route=file n=2 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=2 prints=0 retire=Failure code=Panic msg="host session caught panic: range start index 4 out of range for slice of length 2" tags=[] trace=0 panic

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:33:37:
`at` split index (is 2) should be <= len (is 1)
A2WINDOW route=file n=2 kind=handled suspended=true first=InputRequired prompt="in" tags=["General::passlim", "General::passlim"] trace=0 prints=0 retire=Failure code=Panic msg="host session caught panic: `at` split index (is 2) should be <= len (is 1)" tags=[] trace=0 panic
A2WINDOW route=script n=0 kind=messages suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=script n=0 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=script n=0 kind=handled suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0 again=["General::passlim"]
A2WINDOW route=script n=1 kind=messages suspended=true first=InputRequired prompt="in" tags=["probe::e0"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=[] cancel=0 trace=0 prints=0 lost-cancel
A2WINDOW route=script n=1 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=1 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=script n=1 kind=handled suspended=true first=InputRequired prompt="in" tags=["General::passlim"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=[] cancel=0 trace=0 prints=0 lost-cancel again=["General::passlim"]

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:33:37:
`at` split index (is 2) should be <= len (is 1)
A2WINDOW route=script n=2 kind=messages suspended=true first=InputRequired prompt="in" tags=["probe::e0", "probe::e1"] trace=0 prints=0 retire=Failure code=Panic msg="host session caught panic: `at` split index (is 2) should be <= len (is 1)" tags=[] trace=0 panic
A2WINDOW route=script n=2 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=2 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:33:37:
`at` split index (is 2) should be <= len (is 1)
A2WINDOW route=script n=2 kind=handled suspended=true first=InputRequired prompt="in" tags=["General::passlim", "General::passlim"] trace=0 prints=0 retire=Failure code=Panic msg="host session caught panic: `at` split index (is 2) should be <= len (is 1)" tags=[] trace=0 panic
A2WINDOW route=eval-get n=0 kind=messages suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=eval-get n=0 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=eval-get n=0 kind=handled suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0 again=["General::passlim"]
A2WINDOW route=eval-get n=1 kind=messages suspended=true first=InputRequired prompt="in" tags=["probe::e0"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=eval-get n=1 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=1 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=eval-get n=1 kind=handled suspended=true first=InputRequired prompt="in" tags=["General::passlim"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0 again=["General::passlim"]
A2WINDOW route=eval-get n=2 kind=messages suspended=true first=InputRequired prompt="in" tags=["probe::e0", "probe::e1"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=eval-get n=2 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=2 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=eval-get n=2 kind=handled suspended=true first=InputRequired prompt="in" tags=["General::passlim", "General::passlim"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0 again=["General::passlim"]
A2WINDOW route=nested n=0 kind=messages suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=nested n=0 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0
A2WINDOW route=nested n=0 kind=handled suspended=true first=InputRequired prompt="in" tags=[] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=["General::cancel"] cancel=1 trace=0 prints=0 again=["General::passlim"]
A2WINDOW route=nested n=1 kind=messages suspended=true first=InputRequired prompt="in" tags=["probe::e0"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=[] cancel=0 trace=0 prints=0 lost-cancel

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_requests.rs:714:38:
range start index 3 out of range for slice of length 2
A2WINDOW route=nested n=1 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=1 prints=0 retire=Failure code=Panic msg="host session caught panic: range start index 3 out of range for slice of length 2" tags=[] trace=0 panic
A2WINDOW route=nested n=1 kind=handled suspended=true first=InputRequired prompt="in" tags=["General::passlim"] trace=0 prints=0 retire=Success term=Some(Cancelled) tags=[] cancel=0 trace=0 prints=0 lost-cancel again=["General::passlim"]

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:33:37:
`at` split index (is 2) should be <= len (is 1)
A2WINDOW route=nested n=2 kind=messages suspended=true first=InputRequired prompt="in" tags=["probe::e0", "probe::e1"] trace=0 prints=0 retire=Failure code=Panic msg="host session caught panic: `at` split index (is 2) should be <= len (is 1)" tags=[] trace=0 panic

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/session/session_requests.rs:714:38:
range start index 4 out of range for slice of length 2
A2WINDOW route=nested n=2 kind=trace suspended=true first=InputRequired prompt="in" tags=[] trace=2 prints=0 retire=Failure code=Panic msg="host session caught panic: range start index 4 out of range for slice of length 2" tags=[] trace=0 panic

thread 'a2_window::a2_window_matrix' panicked at /home/remi/dev/rum/crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:33:37:
`at` split index (is 2) should be <= len (is 1)
A2WINDOW route=nested n=2 kind=handled suspended=true first=InputRequired prompt="in" tags=["General::passlim", "General::passlim"] trace=0 prints=0 retire=Failure code=Panic msg="host session caught panic: `at` split index (is 2) should be <= len (is 1)" tags=[] trace=0 panic
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.08s
```

Harness lines, which differ only in the timing:

- `a2-window-1.log`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.08s` and `EXIT:0`. A following doctest line says `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`.
- `a2-window-2.log`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.07s` and `EXIT:0`. Same doctest line.
- `a2-window-3.log`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.08s` and `EXIT:0`. Same doctest line.

`test result: ok` means the harness caught the host panics. The `A2WINDOW` lines are the verdicts. No cell printed `early-kept` or `repeat-cancel`. Every `prints=` is 0.

Settled cells. File and nested are the same shape. Script trace is the exception. `eval-get` is the healthy shape.

- File and nested, n=0, all three kinds: `Success`, `cancel=1`, no panic. The retirement suffix is `General::cancel`. The input report is empty.
- File and nested, n=1 messages: flagged `lost-cancel`. `first` tags are `["probe::e0"]`. Retirement tags are empty. The in-file message is in the input report and is absent from the suffix.
- File and nested, n=1 trace: flagged panic at `session_requests.rs:714:38`, `range start index 3 out of range for slice of length 2`. `first` trace length is 1.
- File and nested, n=1 handled: flagged `lost-cancel`. `first` tags are `["General::passlim"]`. Retirement tags are empty. `again=["General::passlim"]`. The handled truncate did not panic.
- File and nested, n=2 messages: flagged panic at `diagnostics.rs:33:37`, `` `at` split index (is 2) should be <= len (is 1) ``. `first` tags are both probe symbols.
- File and nested, n=2 trace: flagged panic at `session_requests.rs:714:38`, `range start index 4 out of range for slice of length 2`. `first` trace length is 2. Both failing trace panics restore a slice of length 2. The start index is 2+n. I am not naming the two surviving events.
- File and nested, n=2 handled: flagged panic at `diagnostics.rs:33:37`, the same message split. `first` tags are two `General::passlim`. `again=` is absent because the cell panicked. This is the message split of the two `passlim` messages `Check` pushed. It is not a handled-index panic.
- Script, n=0, all three kinds: `cancel=1`, no panic.
- Script, n=1 messages: flagged `lost-cancel`. `first` tags `["probe::e0"]`. Retirement tags empty. The earlier message is in the input report.
- Script, n=1 trace: not flagged. `cancel=1`, retirement trace length 0, `first` trace length 1, no panic. Trace is never split out of the session, so the inner `Get` snapshot includes the earlier trace and the restore stays at the cancel-time mark.
- Script, n=1 handled: flagged `lost-cancel`, same shape as n=1 messages, with `again=["General::passlim"]`.
- Script, n=2 messages: flagged panic at `diagnostics.rs:33:37`, split index 2 against len 1.
- Script, n=2 trace: not flagged. `cancel=1`, retirement trace 0, `first` trace 2, no panic.
- Script, n=2 handled: flagged panic at `diagnostics.rs:33:37`, the two `passlim` messages.
- `eval-get`, all nine cells: `Success`, `cancel=1`, no panic, no `lost-cancel`. In-file messages and traces are in the input report (`first` tags / `first` trace) and the retirement suffix is only `General::cancel`.

`again=["General::passlim"]` is also printed for n=0, where no earlier `Check` ran. The next request's dedup window starts at its own handled checkpoint (`emit_non_loader_eval_message` looks at `handled[checkpoint..]`, `session_helpers.rs:487-491`). Two `passlim` tags on an n=2 input report mean each `Check` also starts its window at the handled length at that `Check`. `again=` does not show that retirement cleared a handled record.

Why the flagged cells happen. `eval_input` on `InputRequired` returns the error (`session_eval.rs:279-291`) and does not call `finish_input_required_request`, so `pending_eval_request` stays empty for the public file and the script. `finish_accepted_value_request` then takes `RequestCheckpoint::new` from the current lengths (`session_requests.rs:574-577`) and only then abandons. The public-file frame restores the loader snapshot taken at `eval_file` entry (`session_eval.rs:409`, `batch_file.rs:64-66`). For n=1 the restored message list is shorter than that cancel-time mark by enough that `finalize` appends `General::cancel` and `split_off` returns an empty suffix: the cancel stays in the session prefix and the outcome tags are empty. For n=2 the mark is still past `len` after that append, and `split_off` panics. The trace slice panics the same way (`session_requests.rs:714`). The script's earlier messages are spliced back (`session_eval.rs:360-366`) and then removed by the inner `Get` snapshot, which was taken while those messages were split out of `session.messages`. Traces were never split out, so the script trace cells do not panic. `eval-get` stored the admission mark, and the snapshot is after that mark, so every cell splits a defined suffix whose only new message is `General::cancel`.

#### 4. What the guide requires

`docs/guide/getting-started.md:168-182`. The CLI keeps the input-requirement messages and appends the retirement response's new messages, then pops one trailing runtime `General::cancel` (`crates/rum-cli/src/input_retirement.rs:23-51`). A host panic fails that function (`:32`).

Each cell's outcome must be a cancellation the CLI can retire: `Success` with `termination == Cancelled`, or `Failure` `Cancelled`. The retirement suffix must contain the new cleanup messages and one runtime `General::cancel` when the split is defined, so the CLI can pop that trailing cancel. Messages already in the input report stay that report. In-file messages are rolled back with the file and stay out of the suffix. A stdin script's earlier forms stay visible in the input report, and an inner file snapshot must not destroy them in the session after they have been spliced back. Cleanup messages outside a file reach the retirement suffix once. No cell may panic, lose the cancellation outcome, or repeat an in-file message in the suffix.

`eval-get` already has that shape: input report holds the in-file messages, suffix is `General::cancel`. File, nested, and script n=0 already retire. The flagged file, nested, and script cells do not: the panics abort the CLI, and the `lost-cancel` cells return `Success` `Cancelled` with an empty suffix, so the CLI would keep the input report and append nothing. The script n=1 message cell still shows `probe::e0` on the input report. The defect there is the session restore deleting the spliced prefix, which is also what drops `General::cancel` out of the suffix.

#### 5. Proposed fix

An index into a list must not be usable after a restore of that list. A per-site clamp does not do that. Give each indexed list a generation, and store the generation in the mark:

```rust
struct ListMark {
    generation: u64,
    index: usize,
}

struct Generated<T> {
    generation: u64,
    items: Vec<T>,
}

impl<T> Generated<T> {
    fn mark(&self) -> ListMark {
        ListMark { generation: self.generation, index: self.items.len() }
    }

    fn restore(&mut self, items: Vec<T>) {
        self.generation = self.generation.wrapping_add(1);
        self.items = items;
    }

    fn suffix(&mut self, mark: ListMark) -> Vec<T> {
        assert_eq!(mark.generation, self.generation, "checkpoint outlived a restore of this list");
        self.items.split_off(mark.index)
    }
}
```

`restore_loader_snapshot`, `restore_evaluation_rollback_snapshot`, and `rollback_capture_output` go through `restore` or an equivalent generation bump when they replace or truncate. `RequestCheckpoint` holds a `ListMark` per list. `split_off` and the trace slice refuse a stale generation. `Vec::truncate` on the handled list gets the same check. That is the fence. It still has to be the only way to cut, so a new cut site cannot bypass it.

The generation check rejects the illegal cut. It does not by itself keep the script prefix. Two construction rules keep the mark inside the generation it names:

- Every suspend path, including `eval_file` and `eval_script`, stores the admission mark in `pending_eval_request`, the way `finish_input_required_request` already does for `eval` and `eval_expr`. `finish_cancelled_pending_evaluation_request` and `finish_accepted_value_request` stop calling `RequestCheckpoint::new` at cancel time. A snapshot taken after admission has `len >=` that mark, and so does an `OutputBoundary` taken during the request.
- The script loop stops splitting earlier forms out of `session.messages` before a nested `Get` can snapshot. The per-form outcome copies the suffix. The session list stays append-only above the admission mark, so the inner snapshot contains the earlier forms and a restore puts them back.

### Part 2 — the drain must terminate

#### 1. The five observations, on debug `HostSession`

Programs, copied from `.temp/brief-630/probe/src/a2_drain.rs:10-14`. They are the director's programs (`n630` / `markerD` names):

```text
n630 = 0; f630[] := WithCleanup[InputString["x"], n630 = n630 + 1; Print[n630]; f630[]]; f630[]
markerD = 0; Monitor[1+2, WithCleanup[InputString["d"], markerD = markerD + 1; Print[markerD]]]
markerD = 0; Monitor[1+2, WithCleanup[InputString["d"], markerD = markerD + 1]]
```

The plain control is `Monitor[1 + 2, InputString["d"]]`. The recursive probe uses a 16 MB thread, a counting print sink, `cancel_handle` before `eval`, then `cancel()` and `eval("")` with a 30 s receive. The deadline cell uses `eval_with_timeout("", 1ms)` for the retirement. A 1 s deadline cannot be the bound on that cell: the uncapped chain already returns in about 45 ms. The limit cell sets `HostConfig.resource_limits.recursion_limit` to 32. The display cell samples the print program for 3 s, cancels, waits 2 s, then receives for 8 s (10 s after cancel), and runs the marker program under `eval_with_timeout` of 1 s while the parent waits 15 s. If either is still running it prints the red line and `process::exit(101)`.

Safe cells, three runs. Logs `.temp/brief-630/logs/a2-drain-safe-1.log`, `a2-drain-safe-2.log`, `a2-drain-safe-3.log`.

`a2-drain-safe-1.log`:

```text
A2DRAIN recursive-deadline elapsed=43.500812ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=340 n630=340
thread 'a2_drain::a2_f_recursive_cleanup_deadline' panicked at src/a2_drain.rs:171:13:
1ms deadline did not stop the self-recursive cleanup (n630=340, prints=340)
A2DRAIN plain-monitor elapsed=508.084µs Success value=3 term=None tags=[] prints=0
A2DRAIN recursive-limit32 elapsed=1.990373ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=9 n630=9
A2DRAIN recursive-natural elapsed=41.129417ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=340 n630=340
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.10s
EXIT:101
```

`a2-drain-safe-2.log`:

```text
A2DRAIN recursive-deadline elapsed=44.680991ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=340 n630=340
thread 'a2_drain::a2_f_recursive_cleanup_deadline' panicked at src/a2_drain.rs:171:13:
1ms deadline did not stop the self-recursive cleanup (n630=340, prints=340)
A2DRAIN plain-monitor elapsed=337.966µs Success value=3 term=None tags=[] prints=0
A2DRAIN recursive-limit32 elapsed=2.417808ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=9 n630=9
A2DRAIN recursive-natural elapsed=49.043381ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=340 n630=340
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.12s
EXIT:101
```

`a2-drain-safe-3.log`:

```text
A2DRAIN recursive-deadline elapsed=45.426108ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=340 n630=340
thread 'a2_drain::a2_f_recursive_cleanup_deadline' panicked at src/a2_drain.rs:171:13:
1ms deadline did not stop the self-recursive cleanup (n630=340, prints=340)
A2DRAIN plain-monitor elapsed=342.138µs Success value=3 term=None tags=[] prints=0
A2DRAIN recursive-limit32 elapsed=1.225195ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=9 n630=9
A2DRAIN recursive-natural elapsed=47.136225ms first=InputRequired prompt="x" retire=Success value=TerminatedEvaluation["Cancelled"] term=Some(Cancelled) tags=["General::cancel"] prints=0 prints=340 n630=340
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.11s
EXIT:101
```

Display cells. `process::exit(101)` skips the harness, so there is no `test result:` line. Cargo still prints `error: test failed, to rerun pass --lib`.

`a2-monitor-1.log`:

```text
A2DRAIN display-cancel did NOT return 10s after cancel; sample_after=3s prints=25949 prints_after_cancel=43557 rss_kb=29036
A2DRAIN display-timeout did NOT return within 15s (eval_with_timeout 1s) rss_kb=37412
A2DRAIN monitor-display RED still-running rate=true timeout=true
EXIT:101
```

`a2-monitor-2.log`:

```text
A2DRAIN display-cancel did NOT return 10s after cancel; sample_after=3s prints=26720 prints_after_cancel=44766 rss_kb=28988
A2DRAIN display-timeout did NOT return within 15s (eval_with_timeout 1s) rss_kb=37304
A2DRAIN monitor-display RED still-running rate=true timeout=true
EXIT:101
```

`a2-monitor-3.log`:

```text
A2DRAIN display-cancel did NOT return 10s after cancel; sample_after=3s prints=26550 prints_after_cancel=44353 rss_kb=28952
A2DRAIN display-timeout did NOT return within 15s (eval_with_timeout 1s) rss_kb=37332
A2DRAIN monitor-display RED still-running rate=true timeout=true
EXIT:101
```

Settled:

- Plain monitor: green. Three runs, `Success value=3`, tags empty. The control returns.
- Natural recursive cleanup: green as a terminating control. Three runs, `prints=340 n630=340`, `Success` `TerminatedEvaluation["Cancelled"]`, only `General::cancel`. It matches the director's 340. It returns in 41–49 ms. Outcome `prints=0` because the sink consumes them. `n630=340` is the guest count.
- Recursion limit 32: three runs, `prints=9 n630=9`, only `General::cancel`. The count moved with `HostConfig.resource_limits.recursion_limit`. This test does not assert the number 9. The log is the measurement.
- 1 ms deadline: red at HEAD, three runs. The control fired (`n630=340`, the assertion requires `count >= 1` before the `< 20` check, `a2_drain.rs:167-174`). The panic `1ms deadline did not stop the self-recursive cleanup (n630=340, prints=340)` is the falsifier. `test result: FAILED. 3 passed; 1 failed` and `EXIT:101` are that red result. The count did not move off 340.
- Display: red at HEAD, three runs. Both the cancel path and the 1 s `eval_with_timeout` were still running. Prints increased after cancel (25949 to 43557, 26720 to 44766, 26550 to 44353). RSS is one sample per path, about 29 MB at the 3 s cancel sample and about 37 MB at the 15 s timeout sample. That is not a series, and it is not the director's release 21 MB sample. `EXIT:101` is `process::exit`, not a harness panic.

#### 2. Mechanism

The display loop is inside the first `eval`, while the accepted request is still held.

`start_monitor` (`debug_tools/monitor.rs:20-34`) evaluates the display in `refresh_monitor_scope_display` before the body, with `monitor_callbacks_enabled` still true. The display is `WithCleanup[InputString["d"], ...]`. `InputRequired` calls `abandon_pending_suspension(HelperBestEffortDiscard)` (`:121-122`). Every other error, including `Cancelled` and `TimeConstraintExceeded`, is dropped (`:124`, and the callback arm `:82-88`).

`push_evaluation_pass_trace` runs when tracing is on or a monitor scope is active with callbacks enabled (`definitions/application/mod.rs:75-80`). `push_trace_with_depth` always calls `handle_monitor_trace_event` (`:96-102`). That sets callbacks false, refreshes every display, runs `StepMonitor`, and on `EvaluationPass` runs `EvaluationMonitor`, then sets callbacks true (`monitor.rs:56-78`).

A new suspension reaches the worklist in `abandon_pending_suspension` at `abandon.rs:18`. If a drain is already active, `drain_abandonment_worklist` returns at `:26-28` and leaves that suspension queued. The outer `while` at `:37` later pops it. The cleanup arm requeues a nested `InputRequired` at `:100-104`. Other cleanup errors are not requeued. The worklist is a `Vec` and `pop` is LIFO.

That is the live loop. The cleanup `Print` is an evaluation pass, the monitor scope is still installed, the refresh evaluates the display again, the new `InputString` is pushed because `abandonment_active` is true, and the outer drain evaluates that new cleanup.

Why cancel and the deadline do not end that loop. `poll_host_interrupt_if_due` (`eval_core/mod.rs:122-135`) runs only when `interrupt_armed` (`:36-38`). It increments `interrupt_poll_counter` and returns `Ok` unless the low 10 bits are 0 (`INTERRUPT_POLL_MASK` is 1023, `:13`). On that 1024th call it returns `Cancelled` when the accepted request is cancelled, or `TimeConstraintExceeded` when `eval_deadline` is due. `RequestCancelHandle::cancel` is generation-exact (`model/request_cancellation.rs:102-104` and `:187-193`). The probe takes the handle before `eval` (`a2_drain.rs:219`), which is the reservation `begin_top_level_interrupt_budget` then sees.

On this first `eval`, `begin_top_level_interrupt_budget` has already run (`session_requests.rs:159` before the body). It sets `eval_deadline` from `eval_timeout` and sets `interrupt_armed` when a deadline or an accepted request is present (`session_helpers.rs:30-38`). `eval_with_timeout` calls `set_eval_timeout` first (`rum-host` `session.rs:559`), and `set_eval_timeout` clears `eval_deadline` (`session_api.rs:227-231`), but the following `eval_request` installs the deadline in `begin_top_level` before the display runs. `take_active_accepted_request` has not run. `guest_form` is `None` for this top-level `eval`, so `capture_guest_form_environment` returns `None` (`scope.rs:52`) and `GuestFormEnvironment::install` at `abandon.rs:43-45` does not run. I did not print `interrupt_armed` inside the loop.

The source that keeps the loop alive even when that poll returns is the drop. `monitor.rs:82-88` and `:121-125` discard `Cancelled` and `TimeConstraintExceeded`. `abandon.rs:100-109` requeues only `InputRequired`. The 1023 evaluations between polls still reach `InputString`, and each one pushes another cleanup at `abandon.rs:18`. One dropped observation does not empty the worklist. The measured fact is that cancel did not stop the prints and a 1 s `eval_with_timeout` did not return within 15 s. I did not log a poll return value.

The retirement drain is a separate path, and it is why the 1 ms deadline still counts 340. `eval("")` enters `finish_cancelled_pending_evaluation_request` (`session_requests.rs:155-156`) before `begin_top_level_interrupt_budget` (`:159`). Inside it, `take_active_accepted_request` sets `interrupt_armed = eval_deadline.is_some()` (`session_helpers.rs:86-88`) and then the drain runs. `set_eval_timeout` has stored the 1 ms in `eval_timeout` and set `eval_deadline` to `None` (`session_api.rs:227-231`). `begin_top_level`, which would copy `eval_timeout` into `eval_deadline`, has not run. During that drain the poll is skipped. The chain runs until the recursion limit. The three 1 ms runs staying at 340, in 43–45 ms, is what that order produces. `GuestFormEnvironment::install` can also overwrite `eval_deadline`, `interrupt_armed`, and `interrupt_poll_counter` (`scope.rs:126-136`) when a suspension captured them. That does not run for this top-level recursive probe. It does run for a cleanup whose suspension captured a guest form.

What bounds the self-recursive cleanup at 340. `DEFAULT_RECURSION_LIMIT` is 1024 (`metadata.rs:5`). `evaluate_once` returns `RecursionLimitExceeded` when `evaluation_depth >= resource_limits.recursion_limit` (`eval_core/mod.rs:50-53`). The drain restores each frame's depth in `restore_resume_context` (`abandon.rs:82`, `evaluation_context.rs:32-38`) and `enter_level` adds one (`evaluation_depth.rs:55-57`). The dropped `RecursionLimitExceeded` is not `InputRequired`, so the drain does not requeue, the worklist empties, and `finish_cancelled_eval_response` emits only `General::cancel`. There is no `General::reclim` in the outcome. Lowering the host limit to 32 produced 9. The two counts fit `(limit - 4) / 3` floored: `(1024 - 4) / 3 = 340` and `(32 - 4) / 3 = 9.333`, and the measured counts are 340 and 9. I did not print `evaluation_depth`. The guest `$RecursionLimit` was not what changed. The window probe's `Block[{$RecursionLimit = 4}, ...]` records `General::passlim`, which is a different limit. Native stack is not the bound: the 340 finishes in about 45 ms on a 16 MB thread and returns.

#### 3. Guest evaluation from a helper, and the drain loops

Event hooks that evaluate guest code:

- Monitor display, `refresh_monitor_scope_display` (`monitor.rs:100`). Runs during a drain when a monitor scope is active and a trace event is delivered with callbacks enabled.
- `StepMonitor` and `EvaluationMonitor`, `run_monitor_callback_best_effort` (`:67-74` and `:82`). Same condition. The only builtin options of those names are on `Monitor` (`monitor/options.rs:76`). `SetOptions` of `Monitor` still reaches this function. No other builtin registers them.
- There is no separate user trace hook. `Trace` / `TracePrint` evaluate their argument as the body (`debug_tools/trace.rs:70`). During a drain that is ordinary cleanup code. If it suspends, the `InputRequired` arm extends the worklist.
- `Check` and `Quiet` evaluate their body and fallback (`check_quiet.rs:29`, `:77`, `:125`, `:139`) and emit on `Err` before returning (`:131`, `:157`).
- `TimeConstrained` and `MemoryConstrained` evaluate the limit, the body, and a fallback (`debug_tools/resources/constraints.rs:19`, `:53`, `:83`, `:117`, `:255`, `:281`).
- `ValueQ` evaluates a trial and then restores an evaluation snapshot (`builtins_messages_inspect.rs:107` and `:114`). The restore's trial suspension uses `CheckpointRollbackDiscard`. If `abandonment_active` is already set, that inner `abandon_pending_suspension` queues onto the outer worklist and returns.
- `Reap` evaluates its body and group function (`debug_tools/reap.rs:37`, `:199`, `:243`).
- `Enclose` evaluates a handler (`kernel_data/control_data/confirmation/builtins.rs:269`). `Catch` evaluates a handler (`control_data/nonlocal.rs:342`).
- Accounting continuation evaluates a suspended argument (`accounting_mode/continuation.rs:40`).
- Assert failure evaluates `$AssertFunction` (`control_data/assertions.rs:11` and `:119`).
- Message templates with delayed text evaluate that text (`messages/message_catalog.rs:132`).

Any of these can run during a drain if the cleanup's `evaluate` reaches them. The monitor hooks are the ones that run because of an evaluation event rather than because the cleanup called them. The others are ordinary bodies. A suspension from any of them hits the `InputRequired` requeue.

Drain loops and exits:

- `drain_abandonment_worklist` (`abandon.rs:25-61`). If `abandonment_active`, return immediately (`:26-28`). Otherwise set the flag, and `while` `pop` (`:37`) until the vec is empty.
- `abandon_frames` (`:76-116`). Break when the frame queue is empty (`:78-79`). On `EvaluateCleanup`, requeue the not-yet-abandoned frames (`:91-97`), evaluate the cleanup (`:99`), requeue a nested suspension only on `InputRequired` (`:100-104`), then `break` (`:113`). `Ok`, `Cancelled`, `TimeConstraintExceeded`, `RecursionLimitExceeded`, and every other error fall through that `if` and are dropped. The outer `while` then continues with whatever was pushed.
- The monitor re-entry keeps the vec non-empty. The self-recursive cleanup keeps it non-empty until `RecursionLimitExceeded`, which does not requeue.

#### 4. Invariant and the fix

Invariant: every drain terminates, and a host cancel or a deadline ends it within a bounded amount of work. Each `WithCleanup` that was already queued when the drain began still runs its cleanup once. The guide's "runs guest cleanup once" stays. A cleanup is not skipped because a later cancel arrived. A cleanup that the drain itself creates after cancel or the deadline is visible is not run.

```rust
fn drain_abandonment_worklist(&mut self, reason: AbandonReason) -> AbandonReason {
    if self.abandonment_active {
        return reason;
    }
    self.abandonment_active = true;
    let callbacks = self.monitor_callbacks_enabled;
    self.monitor_callbacks_enabled = false;
    // Install the retiring request's deadline before any cleanup.
    // take_active must not clear a deadline that begin_top_level has not
    // copied out of eval_timeout yet. Poll below does not read interrupt_armed.
    self.arm_abandonment_interrupt();

    while let Some(mut suspension) = self.abandonment_worklist.pop() {
        // existing frame walk
        let cleanup_outcome = self.evaluate(cleanup.expr);
        let stop = self.abandonment_interrupt_due();
        if matches!(cleanup_outcome, Err(EvalError::InputRequired { .. })) && !stop {
            // push the nested suspension, as today
        }
        // do not push when stop is set; drop the nested suspension
        // existing stack clear and break
    }

    self.monitor_callbacks_enabled = callbacks;
    self.abandonment_active = false;
    reason
}
```

`handle_monitor_trace_event` and `refresh_monitor_scope_display` return immediately when `abandonment_active` is set, so a callback that was already inside `evaluate` cannot refresh the display. `GuestFormEnvironment::install` does not write `eval_deadline`, `interrupt_armed`, or `interrupt_poll_counter` while `abandonment_active` is set. `abandonment_interrupt_due` reads the cancel flag and `eval_deadline` directly, on a counter the guest-form install cannot reset. `eval_request` runs that arming before `finish_cancelled_pending_evaluation_request`'s drain, or `finish_cancelled_pending` copies `eval_timeout` into `eval_deadline` before `take_active`.

The bound is the frames already queued, each cleaned up once, plus no monitor-driven frame, plus no nested `InputRequired` after the interrupt is due. The self-recursive `f630[]` stops pushing once the interrupt is due, instead of running to the recursion limit.

#### 5. Falsifiers

Scratch, not the repository. Full source: `.temp/brief-630/probe/src/a2_drain.rs` and `src/a2_window.rs`. The deadline falsifier and the display falsifier are red at HEAD. The plain monitor and the natural recursive cleanup are green. The limit-32 test records the count. Three runs each, quoted above. The deadline assertion, from `a2_drain.rs:160-174`:

```rust
fn a2_f_recursive_cleanup_deadline() {
    match run_recursive(Some(Duration::from_millis(1)), None) {
        Ok((first, retired, elapsed, prints, n)) => {
            let count: u64 = n.parse().unwrap_or(prints);
            assert!(count >= 1, "control did not fire: deadline cleanup count is {count}");
            assert!(
                count < 20,
                "1ms deadline did not stop the self-recursive cleanup (n630={n}, prints={prints})"
            );
        }
        Err(()) => std::process::exit(101),
    }
}
```

`run_recursive` evaluates the program on a 16 MB thread and bounds the receive at 30 s. `watch_display` does the same for the monitor programs, with the 3 s / 10 s cancel bound and the 15 s timeout bound in `a2_f_monitor_display_does_not_stop` (`a2_drain.rs:186-198`).

### Part 3 — error-kind table

The arm as drafted (this brief, the block under "Stage B shape") keeps the `InputRequired` requeue, skips `Err(Cancelled)`, and otherwise, when `abandonment_reports_cleanup_error` is true, calls `emit_non_loader_eval_message` and `emit_top_level_control_flow_messages`. It does not return the cleanup error. The request value stays the request's own status.

`emit_non_loader_eval_message` (`session_helpers.rs:478-498`) only knows the tags in `runtime_message_for_non_loader_eval_error` (`:392-423`): `EvaluationCycleDetected` and `EvaluationPassLimitExceeded` → `General::passlim`; `RecursionLimitExceeded` → `reclim`; `MatchRecursionLimitExceeded` → `matchreclim`; `Cancelled` → `cancel`; `TimeConstraintExceeded` → `timelim`; `MemoryConstraintExceeded` → `memlim`; `StackLimitExceeded`, `IntegerWorkLimitExceeded`, `GeneratedElementLimitExceeded`, and `IterationWorkLimitExceeded` → `reslim`; `IterationLimitExceeded` → `iterlim`; `ExceptionThrown` → `ThrowException::nocatch`. Everything else returns `None`. `emit_top_level_control_flow_messages` (`:501-532`) adds `Throw::nocatch` for `Thrown` and two of them for `ThrownBypassingCatch`, `Break::nofwd`, and `Continue::nofwd`.

Those two functions run at the request boundary (`session_eval.rs:280-281`, `finish_request_failure` at `session_requests.rs:508-509`, and `finalize_top_level_eval_value` at `session_eval.rs:45-54`). `Check` and `Quiet` also call `emit_non_loader_eval_message` inside `evaluate` (`check_quiet.rs:131` and `:157`). The arm's checkpoint is the handled length before `evaluate`, so a `Check` that already recorded the message dedups the arm's second call.

Loader errors are pushed inside `evaluate`, and the builtin returns `Ok($Failed)`, so the drafted arm does not see an `Err`:

- `Get` of a missing or unreadable target calls `emit_loader_message("Get", &err)` and returns `$Failed` (`builtins_context_loader.rs:282-283`). `FindFile` does the same for `LoadPathNotFound` (`:241-248`). No file provider emits `General::noio` and returns `$Failed` (`:233-235`, `:274-276`).
- `finish_guest_get` calls `emit_loader_message_with_source` for a guest-file error and returns `$Failed` (`guest_loading/file.rs:159-162`), except `InputRequired`, `Exit`, `Cancelled`, `TimeConstraintExceeded`, and `MemoryConstraintExceeded`, which propagate (`:152-158`).
- `emit_loader_message` (`session_helpers.rs:546-580`) pushes `symbol::notfound`, `symbol::io`, `symbol::syntax`, `General::noio`, or `symbol::load`.

A cleanup `Get` of a missing file already has its message. A later file snapshot can roll that message back with the file, which is the same rule as any other in-file message. A cleanup that returns `Err(Io)`, `Err(Syntax)`, `Err(LoadPathNotFound)`, or `Err(NoFileIoProvider)` without that push gets nothing from the two emitters. The ordinary top-level `Err` path also pushes no `RuntimeMessage` for those four: `finish_request_failure` calls the same two emitters, and the signal is `SessionRequestError` (`session_requests.rs:792-818`). They are silent on both paths.

Per variant, whether the drafted arm leaves a message:

| `EvalError` | Ordinary message | Where it is pushed | Drafted arm |
|---|---|---|---|
| `ContinuationSlotMismatch` | no `RuntimeMessage`; `SessionRequestError::ContinuationSlotMismatch` (`session_requests.rs:786-790`) | request boundary, as the failure | no message. The ruling says this reason emits. The two emitters return nothing. |
| `Syntax` | `Get::syntax` only through `emit_loader_message` | inside `evaluate`, then `Ok($Failed)` | already pushed on that path. A bare `Err` has no ordinary message either. |
| `InvalidInteger` | none; request error (`:805-807`) | boundary | no message on either path |
| `Io` | `symbol::io` only through the loader | inside `evaluate`, then `Ok($Failed)` | same as `Syntax` |
| `NoFileIoProvider` | `General::noio` through `emit_no_file_io_message` | inside `evaluate`, then `Ok($Failed)` | already pushed on that path |
| `LoadPathNotFound` | `symbol::notfound` through the loader | inside `evaluate`, then `Ok($Failed)` | already pushed on that path |
| `EvaluationCycleDetected` | `General::passlim` | boundary, and `Check`/`Quiet` inside `evaluate` | arm emits; deduped if `Check` already recorded it |
| `RecursionLimitExceeded` | `General::reclim` | boundary | arm emits |
| `StackLimitExceeded` | `General::reslim` | boundary | arm emits |
| `MatchRecursionLimitExceeded` | `General::matchreclim` | boundary | arm emits |
| `Cancelled` | `General::cancel` | boundary. The cancel response already emits one (`finish_cancelled_eval_response`) | arm does not emit. That is the draft's `!Cancelled` guard |
| `TimeConstraintExceeded` | `General::timelim` | boundary. `Get` propagates it with no loader message | arm emits |
| `MemoryConstraintExceeded` | `General::memlim` | boundary. `Get` propagates it | arm emits |
| `GeneratedElementLimitExceeded` | `General::reslim` | boundary | arm emits |
| `IntegerWorkLimitExceeded` | `General::reslim` | boundary | arm emits |
| `IterationWorkLimitExceeded` | `General::reslim` | boundary | arm emits |
| `IterationLimitExceeded` | `General::iterlim` | boundary | arm emits |
| `EvaluationPassLimitExceeded` | `General::passlim` | boundary | arm emits |
| `Return` | none. Top level becomes the value (`session_eval.rs:31`) | boundary convert | no message on either path |
| `Thrown` | `Throw::nocatch` | boundary | arm emits |
| `ThrownBypassingCatch` | two `Throw::nocatch` | boundary | arm emits both |
| `Abort` | none. Top level becomes `$Aborted` (`session_eval.rs:32`) | boundary convert | no message on either path |
| `ProtectedAbort` | none. Same convert | boundary convert | no message on either path |
| `Break` | `Break::nofwd` | boundary | arm emits |
| `Continue` | `Continue::nofwd` | boundary | arm emits |
| `Exit` | none. `SessionRequestError::Exit` (`session_requests.rs:851`), `Display` is `Exit[{code}]` (`session_state.rs:570`) | boundary status, no message | no message. Ruled below |
| `InputRequired` | the input report, not a cleanup message | requeue | requeue kept |
| `ExceptionThrown` | `ThrowException::nocatch` | boundary, and `finalize_top_level_eval_value` (`session_eval.rs:43-46`) | arm emits |

No variant that has an ordinary `RuntimeMessage` is silent under the draft. The boundary tags are emitted by the two calls. The loader tags are pushed inside `evaluate` before the arm sees `Ok`.

Two gaps are not "silent where the ordinary path leaves a message":

- `ContinuationSlotMismatch`. The ordinary signal is the request error, which this arm does not return, because the slot-fault request already has its own failure and a cancelled request stays `Cancelled`. The ruling says the reason emits. The draft's two calls do not. Stage B's falsifier has to name the tag. I am not inventing one here; there is no catalog tag for it.
- `Exit`. The ordinary path leaves no message. A dropped `Exit` is still the silence the ruling removes. The request stays `Cancelled` with the cancel status. Ordinary `Exit` process status stays a status with no message.

`Exit` message, abandonment arm only. Register it on `General` in `INERT_HEAD_BUILTINS` (`builtin_catalog/inert_heads.rs:9-62`), next to `cancel`, so `push_message`'s debug assert (`session_helpers.rs:336-351`) and docgen see it. Do not add it to `runtime_message_for_non_loader_eval_error`, or a later ordinary `emit_non_loader` would start emitting it.

- Symbol: `General`
- Tag: `exit`
- Catalog template: `Reports Exit[`1`] raised while abandoning a cleanup.`
- Pushed text: the `Display`, `Exit[n]` for code `n` (`Exit[{code}]` at `session_state.rs:570`)

```rust
Err(EvalError::Exit { code }) => {
    self.push_message("General", "exit", format!("Exit[{code}]"));
}
```

That arm stays under the policy function, after the `InputRequired` requeue and the `Cancelled` skip. It does not return `SessionRequestError::Exit` and does not change the process status.

Stage B is not started.

## Director ruling on A2, and continuation A3 (2026-10-28 10:40) — grk

**The inventories are accepted. Both proposed fixes are not.** The matrix
and the mechanisms are the evidence for the rest of this line. Each fix
misses one fact that decides its shape.

### Part 1: the window

The matrix stands. File and nested cells panic or lose `General::cancel` for
messages, trace and handled errors. The script loses `General::cancel`
through its spliced prefix. `eval-get` is healthy. The generation fence
turns a wrong cut into a deliberate panic, which is not a fix. The second
construction rule (store the admission mark on every route) breaks a
consumer contract that the proposal does not mention:

- All three front ends merge a retirement by count.
  - `rum-cli` (`input_retirement.rs:23-51`) has two modes: `Cumulative` for
    single-source requests and `CleanupOnly` for batches
    (`main.rs:234,246,258,343`).
  - `rum-jupyter` (`adapter/input_retirement.rs:27-46`) and `rum-repl`
    (`request.rs:95-114`, "host outputs are cumulative across suspensions,
    including cancellation") both skip by count.
- With an admission mark on batches, `CleanupOnly` would print the earlier
  forms twice.
- The cumulative merge is already wrong under rollback, even on the healthy
  `eval-get` route. Take a file that `Message`s before its `InputString`,
  inside a `WithCleanup` whose cleanup is outside the file. The input report
  holds the in-file message (prior = 1). The rollback removes it, so the
  cumulative response is the outer cleanup's message plus `General::cancel`.
  The CLI pops the cancel and skips 1, and the outer cleanup's message is
  lost. Option A's messages would be lost the same way.

The rule this has to satisfy is already in the guide: *messages already
reported with the input requirement remain visible*. Reported output is
never retracted. So:

- **P1.** A request's host-visible output is append-only. Each outcome of a
  request extends what the earlier outcomes of that request reported. A
  session rollback (file transaction, `ValueQ`, evaluation rollback) changes
  the session's message list and `$MessageList`. It never changes what was
  already reported.
- **P2.** An item pushed after the last report reaches the next outcome
  unless a rollback removed it ("rolled back with their file").
- **P3.** One convention for every route. A front end has one merge rule
  and no mode, and the convention is stated in the host documentation.

### Part 2: the drain

The proposal stops the drain once "cancel is visible". On a
`CancelledRequest` drain the cancellation is the drain's own reason, so it
is visible from the first frame. It cannot be the stop signal: every cleanup
the drain creates would be skipped at once. Also, the mechanism is wider than
the monitor. I probed (`.temp/director/gate-630a/loop-1.log`):

```text
DIRLOOP plain: did NOT return in 8s
DIRLOOP second-cancel: did NOT return in 8s
DIRLOOP timeout-1s: did NOT return in 8s
```

The program is `WithCleanup[InputString["x"], While[True, Null]]`, then
`cancel()` and `eval("")`. "second-cancel" cancels a fresh `cancel_handle()`
1 s into the retirement. "timeout-1s" retires through
`eval_with_timeout("", 1 s)`.

The retirement drain runs guest code with no interrupt armed.
`take_active_accepted_request` has already taken the ticket, and the
deadline is installed after the drain (your mechanism, §2). So any cleanup
that loops makes retirement endless, and the host has no control left. These
are two defects:

- **D2a, retirement has no interrupt.** It covers every drain that runs
  outside a live accepted request: the cancel retirement on both routes, and
  `FailedSnapshotRestore`. Check whether anything else belongs here.
- **D2b, a helper generates frames during a drain.** The monitor hooks run
  during a drain and reset what would otherwise bound the chain. Your
  self-recursive control is bounded by the recursion limit, `(limit − 4)/3`,
  because each nested cleanup runs deeper. The monitor chain is not bounded.
  Find out why, and print `evaluation_depth` where the display suspends.

The rules for the fix:
- **"Runs guest cleanup once" stays.** Every cleanup that the drain reaches
  starts, including one the drain itself creates, as today
  (`public_cancelled_nested_with_cleanup_runs_cleanup_and_abandons_nested_input`,
  `suspension_abandonment_contract.rs:591`). What changes is that the host
  can interrupt one: a cleanup that is still running when a host interrupt
  arrives ends with that interrupt's error, the drain goes on to the next
  frame, and every structural restoration (scopes, file rollback) still
  runs.
- **Retirement is work done under a host request,** with that request's
  cancel handle and deadline armed before the first cleanup. That request is
  the `eval("")` or call that retires, or the `restore_from_bytes`. Say
  which handle a host uses to interrupt a retirement, and make it the one
  `cancel_handle()` returns during it.
- **A drain does not run event-driven helpers.** The monitor display, step
  and evaluation hooks are off while `abandonment_active` is set. The other
  helper bodies in your inventory run only when the cleanup calls them, so
  they stay.
- **Termination is argued from structure.** Once helpers are off, every
  frame a drain creates comes from a cleanup's nested `InputRequired`, at a
  greater depth than that cleanup. Make that a checked property (a debug
  assertion where the nested suspension is requeued), so the recursion limit
  bounds every chain. After a host interrupt, each further cleanup ends at
  its first poll.
- **`GuestFormEnvironment::install` must not disarm a drain.** Say which
  state wins while `abandonment_active` is set (deadline, armed flag, poll
  counter), and why.

### Part 3

- **The table is accepted,** with one correction. `ContinuationSlotMismatch`
  in the policy is the *reason* for a drain after a slot fault. The cleanup
  errors during that drain are reported through the same two emitters. The
  `EvalError::ContinuationSlotMismatch` variant returned by a cleanup is a
  separate row: say whether a cleanup's `evaluate` can return it at all.
- **`Exit` is reported as `Exit::abandoned`**, owned by `Exit` like
  `Throw::nocatch` and `Break::nofwd`. The template is
  "Exit[`1`] in a cleanup was not honored because its request was being
  abandoned." Register it with `Exit`'s existing messages. Do not add it to
  `runtime_message_for_non_loader_eval_error`.

### A3 — two designs as code, no source change, then STOP

Same fence. Scratch goes in `.temp/brief-630/`.

1. **The window under P1–P3.**
   - Choose the one convention, cumulative or incremental, with evidence
     from the three front ends and `rum-host`'s documentation.
   - Give the kernel shape as code. It must make an index into a list that
     cannot outlive a restore of that list, by type, for messages, prints,
     trace and its depths, and handled errors. It must also keep reported
     output out of a rollback's reach.
   - Inventory every access to those four lists (file:line, read or
     mutate), so the reach is known before Stage B. List the front-end
     changes, and every existing test whose assertion changes, with its
     before and after.
   - Extend your matrix with the cumulative-merge case above (cleanup
     outside the file, message inside it) for all three front ends' merge
     rules, red at HEAD.
2. **The drain under the rules above.**
   - Give the shape as code.
   - Find the monitor chain's depth.
   - Write falsifiers, calibrated three times each:
     - mine above (`While[True, Null]`), for the plain, second-cancel and
       timeout cases;
     - the monitor display;
     - `f630` under a deadline: it should now stop early, at its first poll
       after the deadline;
     - controls: the nested-cleanup contract test, "runs guest cleanup
       once", and file rollback after an interrupted cleanup.
3. **Order for Stage B.** I expect B1 to be the drain (the hang is the most
   severe), B2 the window, and B3 the cleanup-error arm. Say if a
   dependency forces another order.

Write "Executing-agent report — A3" and STOP. Every quoted line must be
copied from a file you name. Quote every verdict line your instruments
print, and settle each flagged row.

## Executing-agent report — A3

No source under `crates/`, `scripts/`, or `docs/` was edited except this appendix. `TODO.md`, `Cargo.lock`, `.cargo/`, and `.temp/director/` were not edited. Stage B is not started.

The convention is **cumulative**, one rule for every route. The kernel shape below is the design, not a patch. The A2 generation-panic fence is not revived, and neither is an admission mark stored on batches.

### 1. Window under P1–P3

#### Convention

Cumulative, per request. Evidence:

- `docs/guide/embedding.md:298-313` says the prints, messages, and raw trace arrays are cumulative for the request, the consumer takes only the new suffix, and output already exposed stays an ordered prefix. Cancellation discards provisional output and preserves the public prefix.
- All three front ends already skip by count. `rum-cli` `input_retirement.rs:35-51` uses `(suspension.prints.len(), suspension.messages.len())` for `Cumulative` and `(0, 0)` for `CleanupOnly`. `rum-jupyter` `adapter/input_retirement.rs:27-46` always skips `suspension.messages.len()` (and the same for prints and trace). `rum-repl` `request.rs:95-120` skips `Delivered` and advances that count only as each line is written. Retirement is `resume_with_input("")` (`request.rs:250-252`). `omit_cancel_notice` (`request.rs:301-312`) pops a trailing General or System`General cancel only when `messages.len() > delivered.messages`.
- P1 is the same rule: each outcome of a request extends what the earlier outcomes of that request reported. A session rollback changes the session list and `$MessageList`. It does not change the reported prefix.
- `CleanupOnly` exists because a batch kernel outcome is a post-rollback suffix. With a cumulative reported log, prior `(0, 0)` would reprint the prefix. Deleting the mode is P3. An incremental kernel would make every front end stop skipping and would rewrite `embedding.md:298`. The REPL cursor stays valid only while the reported array never shrinks; the front end, not the kernel, owns that cursor.

#### Kernel shape

Two structures.

The reported log is append-only for one request. Rollback never truncates it. Host outcomes clone it. Items carry an id minted at push. An id is not an index.

```rust
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct LogId(NonZeroU64);

struct Logged<T> { id: LogId, value: T }

struct Reported {
    messages: Vec<Logged<RuntimeMessage>>,
    prints: Vec<Logged<String>>,
    trace: Vec<Logged<TraceEvent>>,
    trace_depths: Vec<Logged<usize>>,
    handled: Vec<Logged<RuntimeMessage>>,
    seen: HashSet<LogId>,
}

fn publish<T: Clone>(reported: &mut Vec<Logged<T>>, seen: &mut HashSet<LogId>, live: &[Logged<T>]) {
    for item in live {
        if seen.insert(item.id) {
            reported.push(item.clone());
        }
    }
}
```

`publish` runs when a host outcome is packaged. It appends live session items whose ids are not yet reported. It does not walk ids that a rollback has already dropped from the session list. Those never reach this outcome (P2), unless an earlier outcome of this same request already published them (P1). `LoaderSnapshot`, `EvaluationRollbackSnapshot`, and `rollback_capture_output` replace or truncate the session vecs only. They do not touch `Reported`.

The director's cell then works on the eval route. The input outcome publishes `probe::inside`. File rollback removes that id from the session list. The cleanup pushes `probe::outer` and the runtime pushes `General::cancel`. `publish` appends those two. The reported log is `[inside, outer, cancel]`. The front end's prior is 1, it pops the trailing cancel because `len > prior`, and `skip(1)` shows `probe::outer`.

An index into a session list cannot outlive a restore of that list, by type. `Session` is one concrete struct, so the brand cannot be a type parameter of `Session`, and a `LogIndex<'g>` stored inside `Session` would be a self-borrow. The brand lives only inside a guard. Frames do not store a `usize` or a `LogIndex`. They store a `LogId` (`Copy`, `'static`). There is no `fn suffix(usize)` and no `fn cut(LogIndex)` a caller can apply after the guard ends. This is not a generation counter plus an assert. A stale id is absent from the restored vec; the cut is not retargeted onto a later item, and it does not panic.

```rust
struct LogIndex<'g> {
    index: usize,
    _brand: PhantomData<fn(&'g ()) -> &'g ()>,
}

struct LogsGuard<'g> { /* reborrow of the five vecs; 'g ends with the guard */ }

impl Session {
    fn with_logs(&mut self, body: impl for<'g> FnOnce(LogsGuard<'g>)) { /* ... */ }
}

// A frame field is LogId, never usize and never LogIndex<'g>.
// Resolving an id:
//   Some(index) => the item is still in this generation of the list
//   None        => a restore already dropped it; do not truncate, do not panic
```

`RequestCheckpoint` (`session_requests.rs:4-8`) and `PendingEvalRequest` (`model/requests.rs:232-235`) stop being the host cut. `finish_cancelled_eval_response` (`session_requests.rs:712-714`) and the other `split_accounted_*` / `trace_events[checkpoint..]` host packagers read `Reported` instead. `Vec::split_off` panics when `at > len` (`diagnostics.rs:33` and `:70`). The trace slice panics the same way (`session_requests.rs:714`). Those panics cannot fire on the host cut once the host cut no longer indexes the session vec with a `usize` taken before a restore.

Internal capture checkpoints become `LogId`s too, resolved only inside `with_logs`: `OutputBoundary` (`suspension/output.rs:9-13`), `GuestFormScope` (`guest_loading/scope.rs:148-150`), `GetCompleteFrame` (`file.rs:41-42`), `GuestScriptCursor` (`script.rs:21-24`), the evaluation-data checkpoint (`evaluation_data.rs:192-195`), the test-report capture frames, and the trace-capture frames (`debug_tools/trace.rs`). A local `let n = self.messages.len()` that dies before the next restore can stay a `usize`. A `usize` stored on a frame cannot.

Do not store an admission mark on batches. The script loop stops using a live length as the retirement cut (the mechanism under the script rows below). The per-form outcome copies what it reports; the session list is not split down so that a nested `Get` snapshot loses the outer cleanup. P3's single skip is what stops a batch from printing earlier forms twice.

#### Script rows are a stale cut, not the same skip

On eval, abandon pops innermost first (`insert_suspension_frame` is `push_back`, `take_front` is `pop_front`; `suspension/core.rs:540-549` and `:204-206`). `GuestFileCloseFrame::abandon` rolls the loader snapshot back (`file.rs:32-36`, `LoaderSnapshot.messages` at `session_state.rs:317`) before `WithCleanup`'s cleanup pushes `probe::outer`. The eval route stored `pending_eval_request` at the start of the request (`finish_input_required_request`, `session_requests.rs:472`). The retirement split therefore starts before `probe::outer`, and the kernel outcome is `[probe::outer, General::cancel]`.

`eval_script_request` (`session_requests.rs:389-399`) packages `InputRequired` through `finish_completed_request_failure` and `finish_accepted_value_request`'s early return (`:570-571`). That path does not store `pending_eval_request`. `finish_cancelled_pending_evaluation_request` (`:56-59`) then does `RequestCheckpoint::new(self)`, the live length, which still includes `probe::inside` (prior = 1). Abandon restores the file snapshot and then pushes `probe::outer`, and `finalize_top_level_eval_value` pushes `General::cancel` before the split (`:705-712`). `split_off` at the pre-rollback index returns only `[General::cancel]` and leaves `probe::outer` below the cut. `$MessageList` is `last_message_list`, assigned from that suffix (`:715`), so it also shows only the cancel. This is the stale `usize`, measured below. It is not `CleanupOnly`, and it is not the script `Err` splice (`session_eval.rs:360-376`), which does not run on this `Ok` cancellation break (`:352-357`).

#### Access inventory

The classifier is `.temp/brief-630/inventory.py`. The file:line list is `.temp/brief-630/access-inventory.md`. Its header counts, copied from that file:

```text
## messages
81 accesses: 68 read, 13 mutate.
## prints
57 accesses: 47 read, 10 mutate.
## trace_events
42 accesses: 36 read, 6 mutate.
## trace_event_depths
16 accesses: 10 read, 6 mutate.
## handled_non_loader_eval_errors
52 accesses: 34 read, 18 mutate.
```

Four of those lines are not the session lists. Exclude them from the reach:

- `metadata.rs:566` and `:582` assign `BuiltinSpec::messages` (`&'static [BuiltinMessageSpec]`).
- `suspension/output.rs:45` and `:46` assign `OutputBoundary`'s `Option<usize>` checkpoints (`self.prints = earliest(...)`, `self.messages = earliest(...)`).

The truncates at `output.rs:73`, `:77`, `:81`, and `:83` are session-list mutations (`rollback_capture_output`). Production session-list mutations in that file, after those exclusions:

- messages: `check_quiet.rs:165` truncate, `confirmation.rs:84` truncate, `output.rs:77` truncate, `session_api.rs:286` `mem::take`, `session_helpers.rs:203` replace from snapshot, `session_eval.rs:361` splice, `diagnostics.rs:18` push, `diagnostics.rs:33` `split_off`. Test assigns: `session_memory_structures.rs:306-307`.
- prints: `output.rs:73` truncate, `session_api.rs:296` `mem::take`, `session_helpers.rs:282` replace, `session_eval.rs:372` splice, `diagnostics.rs:58` push, `diagnostics.rs:70` `split_off`. Test assigns: `session_memory_oracle.rs:108-110`.
- trace events: `output.rs:81` truncate, `session_api.rs:276` `mem::take`, `session_helpers.rs:193` replace, `diagnostics.rs:115` push. Test assign: `session_memory_structures.rs:312-313`.
- trace depths: `output.rs:83` truncate, `session_api.rs:272` `clear`, `session_helpers.rs:195` replace, `diagnostics.rs:129` push. Test assign: `session_memory_oracle.rs:111-112`.
- handled: `check_quiet.rs:118` and `:172` truncate, `session_requests.rs:250`, `:364`, `:518`, `:545`, `:719` truncate, `session_helpers.rs:284` replace, `session_eval.rs:267` and `:286` truncate, `guest_loading/scope.rs:90` and `:131` replace, `:115` `mem::take`, `evaluation_data.rs:83` truncate, `test_report/evaluation.rs:242` and `:264` truncate, `diagnostics.rs:95` push. Test assign: `guest_loading/tests.rs:336`.

A broad `.messages` search also hits outcome copies. Those are not the session vecs. The inventory pattern is `(self|session|snapshot|s).<field>`.

#### Front ends, and tests whose assertions change

Front ends:

- Delete `CancellationOutput`. `main.rs:234`, `:246`, `:258`, and `:343` all use the one skip-by-count of what that front end has already emitted. `:246` and `:258` stop passing `CleanupOnly`. `eval_tests.rs:65` and `:447` stop passing the enum. The queued-answer test (`eval_tests.rs:442-458`) does not compare message vectors; its expected input error and the later `"future"` value stay.
- Jupyter and the REPL keep skip-by-count. No mode to delete. The REPL still advances `Delivered` only when the write succeeds.
- `embedding.md:298-313` stays. Add one sentence: a prefix already reported for this request stays in later outcomes of that request after a rollback of the session message list and `$MessageList`.

Existing expected values that were re-read and do not change:

- `eval_tests.rs:161-202` `cleanup_output_appears_once_and_tail_never_runs`: prefix once except setup, cleanup once, no tail, no `cancel`.
- `eval_tests.rs:389-438` `cleanup_messages_appear_once_with_original_input_reason`: `before-message` once, `cleanup` once, `cleanup-message` once except `file` where the count is 0, `input required` once, no `cancel`. The file count stays 0. That cleanup is inside the file, so `GuestFileCloseFrame` rolls it back after it was pushed and before packaging (P2). It was never in an earlier outcome, so `publish` does not resurrect it. A Stage B that publishes rolled-back in-file messages would break this assertion; that would be the wrong fix.

No current test asserts the lost-outer cell. That cell is the probe below. Session-list and `$MessageList` rollback assertions stay, because the session list still rolls back.

#### Merge matrix, red at HEAD

Probe `.temp/brief-630/probe/src/a3_probe.rs`, test `a3_merge`. Program: `Inner.rum` is `probe::inside = "inside message"`, `Message[probe::inside]`, `InputString["in"]`. Outer is `WithCleanup[Get["Inner`"], Message[probe::outer]]` after `probe::outer = "outer message"`. Routes `eval` and `script`. Retire `eval("")` and `resume_with_input("")`. Three logs, same `A3MERGE` lines. `a3-merge-1.log` ( `a3-merge-2.log` and `a3-merge-3.log` repeat every `A3MERGE` line; each ends `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.02s` and `EXIT:0` ):

```text
A3MERGE route=eval retire=eval prior=1 input=["probe::inside"] kernel=["probe::outer", "General::cancel"] trace_in=0 trace_out=0 cli_cumulative=[] cli_cleanup_only=["probe::outer"] jupyter=[] repl=[] panic=false
A3MERGE FLAG route=eval retire=eval rule=cli_cumulative RED lost-outer
A3MERGE FLAG route=eval retire=eval rule=cli_cleanup_only kept-outer
A3MERGE FLAG route=eval retire=eval rule=jupyter RED lost-outer
A3MERGE FLAG route=eval retire=eval rule=repl RED lost-outer
A3MERGE route=eval retire=eval message_list=List[probe::outer, General::cancel]
A3MERGE route=eval retire=resume prior=1 input=["probe::inside"] kernel=["probe::outer", "General::cancel"] trace_in=0 trace_out=0 cli_cumulative=[] cli_cleanup_only=["probe::outer"] jupyter=[] repl=[] panic=false
A3MERGE FLAG route=eval retire=resume rule=cli_cumulative RED lost-outer
A3MERGE FLAG route=eval retire=resume rule=cli_cleanup_only kept-outer
A3MERGE FLAG route=eval retire=resume rule=jupyter RED lost-outer
A3MERGE FLAG route=eval retire=resume rule=repl RED lost-outer
A3MERGE route=script retire=eval prior=1 input=["probe::inside"] kernel=["General::cancel"] trace_in=0 trace_out=0 cli_cumulative=[] cli_cleanup_only=[] jupyter=[] repl=[] panic=false
A3MERGE FLAG route=script retire=eval rule=cli_cumulative RED lost-outer
A3MERGE FLAG route=script retire=eval rule=cli_cleanup_only RED lost-outer
A3MERGE FLAG route=script retire=eval rule=jupyter RED lost-outer
A3MERGE FLAG route=script retire=eval rule=repl RED lost-outer
A3MERGE route=script retire=eval message_list=List[General::cancel]
A3MERGE route=script retire=resume prior=1 input=["probe::inside"] kernel=["General::cancel"] trace_in=0 trace_out=0 cli_cumulative=[] cli_cleanup_only=[] jupyter=[] repl=[] panic=false
A3MERGE FLAG route=script retire=resume rule=cli_cumulative RED lost-outer
A3MERGE FLAG route=script retire=resume rule=cli_cleanup_only RED lost-outer
A3MERGE FLAG route=script retire=resume rule=jupyter RED lost-outer
A3MERGE FLAG route=script retire=resume rule=repl RED lost-outer
A3MERGE red-rule-rows=14
```

Settlement. The test asserts `red_rows > 0`, so `EXIT:0` means the cell stayed red. It is not a pass of the merge.

- `eval`/`eval` and `eval`/`resume`, rules `cli_cumulative`, `jupyter`, and `repl`: RED lost-outer. This is the director's bug. The kernel still has `probe::outer` (`$MessageList` is `List[probe::outer, General::cancel]`). prior is 1, the cancel is popped, and `skip(1)` drops `probe::outer`.
- `eval`/`eval` and `eval`/`resume`, rule `cli_cleanup_only`: kept-outer. Not red. prior is forced to 0, so the kernel suffix `[probe::outer, General::cancel]` keeps the outer message after the pop. This is why `CleanupOnly` cannot be the fix, and why an admission mark on batches would reprint earlier forms.
- `script`/`eval` and `script`/`resume`, all four rules: RED lost-outer, including `CleanupOnly`. The kernel outcome itself is only `General::cancel`, so no skip rule can show `probe::outer`. That is the stale cut in the section above, in addition to the cumulative skip on the eval route. `red-rule-rows=14` is 16 flag rows minus the two kept-outer rows.

The file route was not re-run. A2 already measured file rollback. This probe's `eval` route is the director's `Get` of a file.

### 2. Drain

#### Shape

Do not stop the drain because cancel is visible. On a `CancelledRequest` drain that bit is the reason. A cleanup the drain reaches is entered, including one the drain creates (`public_cancelled_nested_with_cleanup_runs_cleanup_and_abandons_nested_input`).

The retirement request stays the active accepted request until the drain returns. `take_active_accepted_request` moves to after `abandon_pending_suspension`. Today `finish_cancelled_pending_evaluation_request` (`session_requests.rs:60-63`) and the already-cancelled arm of `finish_accepted_value_request` (`:574-584`) take the ticket first. `resume_eval_request_with_input` (`:273-276`) calls `finish_cancelled_pending` first, so the REPL's `resume_with_input("")` is the same path. `begin_top_level_interrupt_budget` (`session_helpers.rs:30-38`) runs before the abandon, not after it. Today `eval_request` calls the drain at `:155` and the budget at `:159`. `eval_script_request` does not call the budget before its drain at all.

```rust
fn finish_cancelled_pending_evaluation_request(&mut self) -> Result<Option<SessionEvalResponse>, SessionRequestFailure> {
    self.activate_pending_accepted_request()?;
    if !self.active_accepted_request_is_cancelled() || self.pending_suspension_generation().is_none() {
        return Ok(None);
    }
    // Ticket stays in active_accepted_request. Deadline is armed now.
    self.begin_top_level_interrupt_budget();
    self.abandon_pending_suspension(AbandonReason::CancelledRequest);
    let accepted = self.take_active_accepted_request().expect("retirement keeps its ticket until the drain returns");
    let cancellation_won = accepted.finish();
    debug_assert!(cancellation_won);
    // Package from Reported, not from a usize taken before the restore.
    // ...
}
```

The handle a host uses is the one `request_cancel_handle` returns while that ticket is active (`session_api.rs:211-214`): active, else pending, else reserve the next request. During the drain the ticket is active, so `cancel_handle()` returns it. Today `take_active` clears `active_accepted_request` and sets `interrupt_armed = eval_deadline.is_some()` (`session_helpers.rs:86-89`). The session looks idle, and a later `cancel_handle()` reserves the next generation.

The director's instrument does not take the handle mid-`eval`. `.temp/director/gate-630a/probe/src/lib.rs:609-618` takes it after the original `cancel()` and before `eval("")` / `eval_with_timeout`, then the parent sleeps 1s and calls `second.cancel()`. At that pre-`eval` moment the cancelled ticket is still active, so the handle is that same generation. `RequestInterruptState::cancel` (`request_cancellation.rs:187-192`) is a `compare_exchange` from not-cancelled to cancelled. A second `cancel()` of an already-cancelled generation does not change the word. The A3 falsifier reproduces that order (`a3_probe.rs` `dirloop_case`). After this shape, that handle is still the retirement ticket, and the drain polls it. The second `cancel()` stays a no-op because the original `cancel()` already set the bit. A handle taken after `take_active`, which is what would reserve the next request, is not what this instrument does, and it is not reachable during the drain once the ticket stays put. `HostSession::eval` holds `&mut HostSession`, so another call on the same session cannot run until `eval` returns.

`FailedSnapshotRestore` (`session_api.rs:102-103`) abandons and only then `finish_active_accepted_request`. The ticket is still installed. It is not the "ticket already taken" defect. Arm the same budget and the same poll rules before that abandon. If `eval_timeout` is `None`, the deadline stays `None`; `interrupt_armed` is already true because the accepted request is `Some`.

Nothing else is a drain outside a live accepted request. Production `abandon_pending_suspension` sites:

- `session_requests.rs:63` and `:581` — D2a, ticket taken first today.
- `session_api.rs:102` — `FailedSnapshotRestore`, ticket still held.
- `session_helpers.rs:249` — `CheckpointRollbackDiscard`, inside a live `ValueQ` restore. Ticket not taken. Not D2a.
- `monitor.rs:85` and `:122` — `HelperBestEffortDiscard`, during the live request. D2b.
- `builtins_input.rs:234` — `SupersededOrQueuedInputDiscard`, during `drive_input_request`. Ticket held.
- `accounting_mode/bridge.rs:252`, `options.rs:219`, `frozen.rs:121` and `:153` — `HelperBestEffortDiscard` while retaining a live evaluation. Ticket held.
- `suspension/core/fault.rs:126` — `drain_abandonment_worklist(ContinuationSlotMismatch)` from the resume driver. Ticket held.

`eval_call_prepare.rs:142`, `value_table_assignment.rs:383`, and `iteration/mod.rs:299` are inside `#[cfg(test)]` / `#[test]`. Not routes.

Those live-ticket drains do not get a new ticket. They do get the helper switch, the depth assert, and the `install` rule, so a captured environment cannot clear the live deadline.

Helpers. While `abandonment_active`, `refresh_monitor_scope_display` and `run_monitor_callback_best_effort` return immediately. `handle_monitor_trace_event` already returns when `monitor_callbacks_enabled` is false (`monitor.rs:57-58`) and then sets it back to true (`:78`). `finish_monitor` calls `refresh_monitor_scope_display` without looking at that flag (`:51`). The flag is not enough. The check is `abandonment_active`. Check, Quiet, TimeConstrained, ValueQ, Reap, Catch, `$AssertFunction`, and delayed message text stay. They run only when the cleanup calls them.

Poll. Two checks, because they are not the same signal.

- The deadline is checked on every `poll_host_interrupt_if_due` call while `abandonment_active`, with no `INTERRUPT_POLL_MASK` (`eval_core/mod.rs:13`, poll at `:122-135`). A cleanup that is still running when the deadline passes returns `TimeConstraintExceeded` on the next poll. That is the f630 rule: stop at the first poll after the deadline, so `n630` falls below 20 instead of dying at the recursion bound of 340. The contract tests set no deadline, so this check is idle there.
- The cancel bit stays on the existing mask (every 1024 calls). `While`'s own loop already calls `poll_host_interrupt_if_due` every iteration (`branching_loops.rs:150-151`). Once the ticket stays active, `While[True, Null]` returns at the first masked poll, which is a few thousand iterations and well under 8s. The bit is not checked on the cleanup's first `evaluate_once`. Doing that would return `Cancelled` before `hits630 = hits630 + 1` and before the nested-contract increments, and those tests would go from green to red. A straight-line cleanup is shorter than the stride, so it still runs to completion. That is "runs guest cleanup once". A looping cleanup is the one the masked poll cuts. The drain then continues to the next frame. Scopes and file rollback still run. A cleanup that has not started is still entered.

Depth assert, where the nested suspension is requeued (`abandon.rs:100-104`), after `restore_resume_context` has stored the suspend depth:

```rust
let entry_depth = self.evaluation_depth.get();
let cleanup_outcome = self.evaluate(cleanup.expr);
if matches!(cleanup_outcome, Err(EvalError::InputRequired { .. })) {
    if let Some(nested) = self.take_accounted_pending_suspension_for_abandon() {
        debug_assert!(nested_front_depth(&nested) > entry_depth);
        self.abandonment_worklist.push(nested);
    }
    // ...
}
```

Once monitor display, step, and evaluation hooks are off, every frame the drain itself creates is that nested `InputRequired`, at a greater depth than the cleanup that created it. The recursion limit then bounds the chain the way it already bounds f630, `(limit − 4) / 3`. The assert is the checked property. It is not a panic on a bad host cut.

`GuestFormEnvironment::install` (`scope.rs:126-136`) writes `eval_deadline`, `interrupt_poll_counter`, and `interrupt_armed` from the captured form. While `abandonment_active` those three writes do not happen. Message-list fields still install. The drain's values win: the deadline is the one `begin_top_level_interrupt_budget` installed before the first cleanup; `interrupt_armed` is true because the retirement ticket is active; the poll counter keeps the progress the drain has made, so a captured form cannot put the counter back mid-stride or clear a deadline that has already elapsed. The captured environment is the suspended form's old interrupt snapshot. `install_for_resume` calls `install`, so the same skip covers it. `DisplacedGuestFormEnvironment::restore` (`scope.rs:91-95`) writes the same three fields; the resume driver's restore at `suspension/core.rs:689` runs after the drain has cleared `abandonment_active`. A restore that runs while the flag is set must skip them too.

#### Why the monitor chain is not recursion-bounded

`abandon_pending_suspension` saves `evaluation_depth` and sets it to 0 (`abandon.rs:34-35`). After each suspension it sets it to 0 again (`:54`). After each cleanup it clears the stack and sets it to 0 (`:111-112`). f630 deepens because `restore_resume_context` (`evaluation_context.rs:32-38`) puts back the nested suspension's captured depth and `enter_level` adds one (`evaluation_depth.rs:55-57`). The display does not stay on that stack. `Monitor`'s second argument is the display (`monitor.rs:26-33`); the probe and the A2 program use `Monitor[1+2, WithCleanup[...]]`, so the body is `1+2` and the display is the `WithCleanup` that reaches `InputString`. `refresh_monitor_scope_display` evaluates that display (`:100-122`). `InputRequired` calls `abandon_pending_suspension(HelperBestEffortDiscard)`. While `abandonment_active` is set, `drain_abandonment_worklist` returns immediately (`abandon.rs:26-28`) after the new suspension is pushed. `EvalMonitorSuspensionFrame::abandon` only pops the scope and returns `Discard` (`monitor.rs:143-150`). The next drain iteration starts from depth 0. The recursion check (`eval_core/mod.rs:50`) sees a shallow depth every time, so the limit never fires.

`evaluation_depth` is `pub(in crate::runtime)` (`model/session.rs:87`). There is no public getter. The probe prints `Length[Stack[]]` in the display, immediately before `InputString`. `Stack` (`debug_tools/basic.rs:96-108`) is `List` of `Hold[expr]` for `evaluation_stack` except the last entry. The numbers below are that length, not `evaluation_depth`.

Test `a3_monitor_depth`. Three logs, each `EXIT:101`, cargo `error: test failed`, no `test result:` line. `process::exit(101)` is the falsifier.

`a3-depth-1.log`:

```text
A3DEPTH did NOT return in 8s after the 2s sample
A3DEPTH stack_len samples=96374 unique={2: 23224, 3: 35451, 4: 25875, 5: 9643, 6: 1983, 7: 193, 8: 5} head=[3, 2, 2, 4, 3, 3, 2, 2] tail_rev=[3, 3, 4, 4] ticks=91024 lines_at_2s=39114 lines_at_end=187398 early_head=["tick", "3", "tick", "2", "2", "tick"]
A3DEPTH evaluation_depth unread; field is private
A3DEPTH RED still-running
```

`a3-depth-2.log`:

```text
A3DEPTH did NOT return in 8s after the 2s sample
A3DEPTH stack_len samples=96572 unique={2: 23290, 3: 35551, 4: 25912, 5: 9638, 6: 1983, 7: 193, 8: 5} head=[3, 2, 2, 4, 3, 3, 2, 2] tail_rev=[4, 4, 4, 4] ticks=91222 lines_at_2s=39209 lines_at_end=187794 early_head=["tick", "3", "tick", "2", "2", "tick"]
A3DEPTH evaluation_depth unread; field is private
A3DEPTH RED still-running
```

`a3-depth-3.log`:

```text
A3DEPTH did NOT return in 8s after the 2s sample
A3DEPTH stack_len samples=96680 unique={2: 23357, 3: 35559, 4: 25896, 5: 9673, 6: 1996, 7: 194, 8: 5} head=[3, 2, 2, 4, 3, 3, 2, 2] tail_rev=[2, 2, 2, 2] ticks=91335 lines_at_2s=39481 lines_at_end=188015 early_head=["tick", "3", "tick", "2", "2", "tick"]
A3DEPTH evaluation_depth unread; field is private
A3DEPTH RED still-running
```

Settlement. All three are RED still-running. The chain does not stop. `Stack[]` length is not constant and it does not grow: across about 96k samples the only values are 2 through 8, the mass is 2, 3, and 4, and 8 occurs five times in each run. The head `[3, 2, 2, 4, 3, 3, 2, 2]` is the same in all three. Ticks keep climbing (91024, 91222, 91335) after the 2s sample. That is a shallow re-entry, consistent with the depth reset, not a chain the recursion limit will cut. `evaluation_depth` itself was not read.

#### Director loop, red at HEAD

Test `a3_dirloop`. Program `WithCleanup[InputString["x"], While[True, Null]]`, then `cancel()` and retire. `second-cancel` cancels the handle taken before `eval("")`, 1s later. `timeout-1s` uses `eval_with_timeout("", 1s)`. Three logs, the same five lines, each `EXIT:101`, cargo `error: test failed`, no `test result:` line. `a3-dirloop-1.log` (2 and 3 repeat it):

```text
A3LOOP plain: did NOT return in 8s
A3LOOP second-cancel: did NOT return in 8s
A3LOOP timeout-1s: did NOT return in 8s
A3LOOP hung=3
A3LOOP RED plain second-cancel timeout-1s
```

Settlement. All three cases are RED. They match `.temp/director/gate-630a/loop-1.log` (`DIRLOOP plain`, `second-cancel`, and `timeout-1s` each `did NOT return in 8s`). `process::exit(101)` is the falsifier, the same settlement as the A2 monitor logs. The design above makes each of these return: the ticket and the deadline are armed before `While` runs, `While` polls every iteration, and the 1s deadline is visible to every poll during the drain.

#### f630 under a deadline, red at HEAD

Test `a3_f630_deadline`. Same program as A2: `n630 = 0; f630[] := WithCleanup[InputString["x"], n630 = n630 + 1; Print[n630]; f630[]]; f630[]`, then `cancel()` and `eval_with_timeout("", 1ms)`. The test asserts `count >= 1` and `count < 20`. At HEAD the chain returns in about 45ms at `n630=340`, so the `< 20` assert panics. Three logs, each `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.05s` and `EXIT:101`.

`a3-f630-1.log`:

```text
A3F630 deadline elapsed=46.867975ms first=InputRequired prompt="x" tags=[] retire=Success term=Some(Cancelled) tags=["General::cancel"] trace=0 prints=0 prints=340 n630=340 expected=first-poll-after-deadline
thread 'a3_probe::a3_f630_deadline' panicked at src/a3_probe.rs:432:5:
1ms deadline did not stop f630 at its first poll (n630=340, prints=340)
```

`a3-f630-2.log`:

```text
A3F630 deadline elapsed=43.481398ms first=InputRequired prompt="x" tags=[] retire=Success term=Some(Cancelled) tags=["General::cancel"] trace=0 prints=0 prints=340 n630=340 expected=first-poll-after-deadline
thread 'a3_probe::a3_f630_deadline' panicked at src/a3_probe.rs:432:5:
1ms deadline did not stop f630 at its first poll (n630=340, prints=340)
```

`a3-f630-3.log`:

```text
A3F630 deadline elapsed=43.397545ms first=InputRequired prompt="x" tags=[] retire=Success term=Some(Cancelled) tags=["General::cancel"] trace=0 prints=0 prints=340 n630=340 expected=first-poll-after-deadline
thread 'a3_probe::a3_f630_deadline' panicked at src/a3_probe.rs:432:5:
1ms deadline did not stop f630 at its first poll (n630=340, prints=340)
```

Settlement. All three are RED. The deadline does not stop the chain; the recursion bound does, at 340, in 43–47ms. Under the design the deadline is installed before the first cleanup and every drain poll looks at it, so the first poll after 1ms returns `TimeConstraintExceeded` and `n630` stays under 20. The `>= 1` clause is this falsifier's "the cleanup did run" check at HEAD. One millisecond at this rate still fits several increments before the deadline, so a green B1 run should keep `n630 >= 1` as well. If a B1 build stops before the increment, `n630=0` fails `>= 1` and the bound has to be re-calibrated; that is not the HEAD result.

#### Controls, green at HEAD

Test `a3_controls`. Three logs, the same four lines, each `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out` and `EXIT:0`. `a3-controls-1.log` finished in 0.01s; log 2 in 0.01s; log 3 in 0.02s. `a3-controls-1.log` (2 and 3 repeat the `A3CTRL` lines):

```text
A3CTRL nested-contract retire=Success term=Some(Cancelled) tags=["General::cancel"] trace=0 prints=0 list=List[1, 1, 0, 0] pending=false
A3CTRL runs-once retire=Success term=Some(Cancelled) tags=["General::cancel"] trace=0 prints=0 hits630=1 tail630=0
A3CTRL file-rollback-returned retire=Success term=Some(Cancelled) tags=["General::cancel"] trace=0 prints=0 marker630=0 rolled630=0
A3CTRL file-rollback-interrupted not in this test; the While[True, Null] retirement does not return, so the marker cannot be read
```

Settlement. No red rows. The nested program from `suspension_abandonment_contract.rs:599-604` returns `List[1, 1, 0, 0]` with no pending input. `WithCleanup[InputString["x"], hits630 = hits630 + 1]; tail630 = 1` leaves `hits630=1` and `tail630=0`. The file whose cleanup returns (`marker630 = 1` inside, `rolled630 = marker630` outside) rolls back to `marker630=0` and `rolled630=0`. The interrupted-cleanup marker is not claimed. That retirement is the director loop, and it does not return, so the marker cannot be read. Stage B must keep these three green.

### 3. Error kinds, not a third design

`ContinuationSlotMismatch` in the policy is the reason for a drain after a slot fault. `unwind_continuation_fault` (`fault.rs:99-127`) drains with `AbandonReason::ContinuationSlotMismatch`. Cleanup errors during that drain go through the same two emitters as any other cleanup error. The `EvalError::ContinuationSlotMismatch` variant is a different row. A guest cleanup's `evaluate` does not construct it. `SlotMismatch::new` is `pub(in crate::runtime::suspension)`. The only constructor of the variant is `DriverFault::into_error` (`fault.rs:38-41`). The legacy wire rejects it (`suspension/error.rs:271-274`, `unreachable`). The resume driver is what returns it: `drive_resuming_frames` on `Err` calls `unwind_continuation_fault` and then `Err(fault.into_error())` (`suspension/core.rs:679-692`). `continuation_fault_outcome` (`fault.rs:88-96`) is the request-boundary converter. Callers: `session_eval.rs:191` and `:245`, `session_requests.rs:229` and `:340`, plus the typed tests. `session_requests.rs:786-790` maps the variant to `SessionRequestError::ContinuationSlotMismatch` and does not build a `RuntimeMessage`. `evaluation_data.rs:127` and `test_report/evaluation.rs:193` and `:234` re-raise an existing value. They are not constructors.

`Exit` is reported as `Exit::abandoned`, on `Exit`'s existing spec, next to `argt` and `intm` (`catalog/control.rs:143-159`). Template: `Exit[`1`] in a cleanup was not honored because its request was being abandoned.` Same place as `Throw::nocatch` (`control.rs:358-361`) and `Break::nofwd` (`control.rs:332-335`). It is not added to `runtime_message_for_non_loader_eval_error` (`session_helpers.rs:392-424`), and it is not a `General` message. Not implemented.

### 4. Order for Stage B

B1 the drain, B2 the window, B3 the cleanup-error arm. No dependency forces another order.

B1 is first because a retirement that does not return never produces the message B3 would add, and it never produces the outcome B2 re-cuts. B3's new `Exit::abandoned` text is host-visible output. On today's eval route the cumulative skip drops `probe::outer` after a rollback even though the kernel still has it. Landing B3 before B2 would drop the new message the same way. That dependency is already B2 before B3.

Stage B is not started.

## Director ruling on A3, and Stage B1 (2026-10-28 11:25) — grk

**A3 is accepted with three changes, and B1 starts now.** I checked your
merge and depth probes against the code, and calibrated the B1 falsifiers
myself (`.temp/director/gate-630a/b1-cal-{1,2,3}.log`).

### The window (B2, briefed after B1's gate)

The convention is accepted: cumulative for every route, `CancellationOutput`
deleted, and Jupyter and the REPL unchanged. The shape changes. You do not
need an id set and a lifetime brand. Use sequence-stamped lists:

- Every push to the four lists (messages, prints, trace events with their
  depths as one list of pairs, and handled errors) takes the next value of
  one monotone session counter, stored with the item. Snapshots keep the
  items with their stamps. A restore only removes items, and new items get
  larger stamps, so every list stays sorted by stamp.
- A mark is a stamp value, not a position. The only cut, suffix and
  truncate operations take a mark: "items after mark" and "drop items after
  mark". Nothing takes a position. With no positional cut API there is
  nothing a stored `usize` can be used for, so no brand is needed. A mark
  stays meaningful across any restore.
- `Reported` is per request and append-only. Publishing appends the session
  items stamped after the last published mark, then advances the mark. That
  is a search in a sorted list, not a `HashSet`. Its memory is charged to a
  `MemoryOwner`.

### The drain (B1)

Accepted:
- the ticket stays installed until the drain returns, with the interrupt
  budget armed before the first cleanup;
- event-driven monitor hooks are off while `abandonment_active` is set;
- the depth assertion where a nested suspension is requeued;
- `GuestFormEnvironment::install` and `DisplacedGuestFormEnvironment::restore`
  do not write the interrupt fields during a drain.

**Not accepted: the masked cancel bit as a per-cleanup budget.** With the
retiring ticket left cancelled, every cleanup longer than 1024 polls would
be cut by the cancel that caused the retirement. It would be cut silently,
because the arm does not report `Cancelled`. A finite cleanup of 20 000
iterations completes today (control `d` below). It must keep completing.

**Not accepted: the 1 ms retiring-call deadline.** `eval_with_timeout` on a
session with a pending suspension does not change the timeout
(`rum-host/src/session.rs:553-557`, and its doc comment). So no retiring
call's timeout applies to a retirement. The deadline in force during
retirement is the suspended request's own absolute deadline, as for resume,
and that already bounds a looping cleanup today (control `b` below). Your
`f630` falsifier rests on the wrong premise, and is dropped.

The rule instead: **each cancel interrupts the guest work running when it
arrives, and the cancel that causes a retirement is consumed by that
retirement.**
- On a retirement drain, cleanups start with no pending interrupt and run to
  completion.
- A further host interrupt during any drain ends the running cleanup with
  that interrupt's error. There are two:
  - another `cancel()` of the same generation, the handle the host already
    holds (today that call is a no-op, `request_cancellation.rs:187-192`);
  - the deadline.
- After either one, the drain enters no further cleanups. Every structural
  restoration still runs (scopes, file rollback, frame retirement). The
  drain reports one message, `WithCleanup::skipped`, with template
  "`1` pending cleanups did not run because a host interrupt stopped the
  retirement."
- On a drain inside a live request (helper, `ValueQ`, accounting), no
  cancel has been consumed. A cancel there interrupts the drain the same
  way, and then the request observes it at its next poll.

So `RequestInterruptState` must record a cancel that arrives after the
retirement consumed the first. Two things stay as they are: a stale
generation's cancel has no effect, and the handle API does not change.

**No cost on the ordinary path.**
- Do not unmask `poll_host_interrupt_if_due`.
- The drain itself checks for a new interrupt (an unconsumed cancel, or an
  expired deadline) before it enters each cleanup. That check is cold, once
  per frame.
- Inside a cleanup, the existing masked poll observes the same "unconsumed
  cancel". Change what `is_cancelled` means for a retirement ticket, not how
  often the poll runs.

### Stage B1 — the drain terminates under host control, then STOP

**Falsifiers (calibrated by the director: red at HEAD 3 of 3, controls green
3 of 3,** `.temp/director/gate-630a/probe/src/lib.rs`, `dir_b1_calibration`,
`dir_retirement_cleanup_loop`, and your `a3_monitor_depth`):

- **F1, escalation.** `WithCleanup[InputString["x"], While[True, Null]]`,
  then `cancel()` and `eval("")`, with a second `cancel()` on the same handle
  1 s later from another thread. HEAD: does not return in 8 s. After B1: it
  returns `Cancelled` within 2 s of the second cancel, and the messages hold
  one `General::cancel`.
- **F2, the monitor display.** `markerD = 0; Monitor[1+2, WithCleanup[InputString["d"], markerD = markerD + 1]]`
  under plain `eval`. HEAD: does not return. After B1: it returns `3`,
  with no cancel needed.
- **F3, escalation skips later cleanups.** Two nested cleanups. The inner
  one loops and the outer one sets a marker, with a `Block` scope and a
  nested `Get` around them. Escalate during the inner one. Expected: the
  outer marker is unset, `WithCleanup::skipped` appears once with count 1,
  the `Block` variable is restored, and the file's assignment is rolled
  back. Write it and calibrate it red at HEAD (it hangs) before you change
  code.
- **Controls, green at HEAD and after:**
  - `b`: the original request under `eval_with_timeout(…, 500 ms)`, then
    cancel and retire. It returns at about 502 ms today, and must not get
    slower.
  - `d`: `k630 = 0; WithCleanup[InputString["x"], Do[k630 = k630 + 1, {20000}]]`,
    then cancel and retire. `k630` is `20000`.
  - your `nested-contract`, `runs-once` and `file-rollback-returned`;
  - all of `suspension_abandonment_contract.rs` and
    `native_stack_thread_scope.rs`.

Put F1–F3 and the controls in a new `crates/rum-host/tests/retirement_interrupt.rs`.
Each runs on a thread with a bounded receive, and a hang fails the test
instead of hanging the binary.

**Scope.**
- **MAY modify:**
  - in `crates/rum-runtime/src/runtime/`: `suspension/core/abandon.rs`,
    `debug_tools/monitor.rs`, `session/session_requests.rs`,
    `session/session_helpers.rs`, `session/session_api.rs`,
    `model/request_cancellation.rs`, `session/guest_loading/scope.rs`, and
    `eval_core/mod.rs` only if the poll's cancel test must call a new
    predicate;
  - the message registration in `crates/rum-stdlib/src/catalog/control.rs`,
    and the regenerated `docs/reference/builtins.md`, produced by its
    generator, not by hand;
  - the new test file;
  - the standing inventories, by replay (TEMPLATE).
- **Disclosure duty:** any other file that must change, with its reason.
  Anything beyond a mechanical change is a STOP.
- **MUST NOT touch:** `rum-cli`, `rum-jupyter`, `rum-repl`, `docs/` other
  than the generated reference, `TODO.md`, `Cargo.lock`. The cleanup-error
  arm (B3) and the window (B2) are not in B1.

**Run, single-threaded:**
- `cargo fmt --check`;
- F1–F3 and the controls, three runs each;
- `cargo test -p rum-host` (all targets);
- `cargo test -p rum-runtime --lib`;
- `frame_root_trace`;
- the standing inventories;
- **the whole workspace** (TEMPLATE: every source stage runs it).

Report the `test result:` lines verbatim.

**Then STOP.** Measurement follows my gate. The resume panel holds cleanup
labels; abandonment is a cold path, and the poll is unchanged.

Append "Executing-agent report — B1". Every quoted line must be copied from
a file you name. Quote every verdict line your instruments print, and settle
each flagged row.

## Executing-agent report — B1

Stage B1 is implemented and stopped on the rejection census. `census/tests.rs` was not edited. The workspace was not started. B2 and B3 were not started.

The retirement consumes the first cancel and leaves the ticket installed. Cleanups then run with no pending interrupt. A later cancel of the same generation, or the suspended request's own absolute `eval_deadline`, ends the running cleanup. Later cleanups are not entered. Structural restoration still runs. One `WithCleanup::skipped` message is pushed when the skipped count is greater than zero. Monitor refresh and the monitor callback return immediately while `abandonment_active` is set. `GuestFormEnvironment::install` and `DisplacedGuestFormEnvironment::restore` skip `eval_deadline`, `interrupt_poll_counter`, and `interrupt_armed` while that flag is set. `eval_core/mod.rs` was not edited. The masked poll is unchanged.

### Disclosure outside the MAY list

`crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` — mechanical. The source gate pins the whitespace-stripped call text of the one shared abandonment body. The skip counters are now arguments, so the three pinned strings include them. The gate still requires one `fn abandon_frames<`, the Ordinary and Profiled split, `frame.abandon(`, and no `.resume(`. No new frame type. `named_suspension_frames` stayed 567.

### F3 at HEAD, before the crate edit

`.temp/brief-630/logs/b1-f3-head-1.log`, `b1-f3-head-2.log`, and `b1-f3-head-3.log`. Each log:

```
B1F3 HEAD did NOT return in 8s
B1F3 RED hung
EXIT:101
```

No `test result:` line. Run 1 also has `Finished test profile ... in 4.26s`. Runs 2 and 3 have `Finished test profile ... in 0.02s`.

### F1–F3 and the controls, three runs

`.temp/brief-630/logs/b1-tests-1.log`:

```
B1CTRL b elapsed_ms=500 termination=Cancelled value=TerminatedEvaluation["Cancelled"] tags=["General::cancel"] cancels=1
B1CTRL d elapsed_ms=221 termination=Cancelled k630=20000 tags=["General::cancel"] cancels=1
B1CTRL file-rollback termination=Cancelled tags=["General::cancel"] marker=0 rolled=0
B1CTRL nested termination=Cancelled tags=["General::cancel"] list=List[1, 1, 0, 0] pending=false
B1CTRL runs-once termination=Cancelled tags=["General::cancel"] hits=1 tail=0
B1F1 elapsed=1.000642913 termination=Cancelled value=TerminatedEvaluation["Cancelled"] tags=["General::cancel"] cancels=1 skipped=0|assert_ms=1000|term=Cancelled|cancels=1|skipped=0
B1F2 value=3 termination=None tags=[]
B1F3 elapsed_ms=1000 termination=Cancelled outer=0 block=9 file=0 skipped_n=1 skipped_text=["1 pending cleanups did not run because a host interrupt stopped the retirement."] cancels=1 tags=["WithCleanup::skipped", "General::cancel"]
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.75s
EXIT:0
```

`.temp/brief-630/logs/b1-tests-2.log`:

```
B1CTRL b elapsed_ms=500 termination=Cancelled value=TerminatedEvaluation["Cancelled"] tags=["General::cancel"] cancels=1
B1CTRL d elapsed_ms=281 termination=Cancelled k630=20000 tags=["General::cancel"] cancels=1
B1CTRL file-rollback termination=Cancelled tags=["General::cancel"] marker=0 rolled=0
B1CTRL nested termination=Cancelled tags=["General::cancel"] list=List[1, 1, 0, 0] pending=false
B1CTRL runs-once termination=Cancelled tags=["General::cancel"] hits=1 tail=0
B1F1 elapsed=1.000605607 termination=Cancelled value=TerminatedEvaluation["Cancelled"] tags=["General::cancel"] cancels=1 skipped=0|assert_ms=1000|term=Cancelled|cancels=1|skipped=0
B1F2 value=3 termination=None tags=[]
B1F3 elapsed_ms=1000 termination=Cancelled outer=0 block=9 file=0 skipped_n=1 skipped_text=["1 pending cleanups did not run because a host interrupt stopped the retirement."] cancels=1 tags=["WithCleanup::skipped", "General::cancel"]
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.81s
EXIT:0
```

`.temp/brief-630/logs/b1-tests-3.log`:

```
B1CTRL b elapsed_ms=500 termination=Cancelled value=TerminatedEvaluation["Cancelled"] tags=["General::cancel"] cancels=1
B1CTRL d elapsed_ms=277 termination=Cancelled k630=20000 tags=["General::cancel"] cancels=1
B1CTRL file-rollback termination=Cancelled tags=["General::cancel"] marker=0 rolled=0
B1CTRL nested termination=Cancelled tags=["General::cancel"] list=List[1, 1, 0, 0] pending=false
B1CTRL runs-once termination=Cancelled tags=["General::cancel"] hits=1 tail=0
B1F1 elapsed=1.000722313 termination=Cancelled value=TerminatedEvaluation["Cancelled"] tags=["General::cancel"] cancels=1 skipped=0|assert_ms=1000|term=Cancelled|cancels=1|skipped=0
B1F2 value=3 termination=None tags=[]
B1F3 elapsed_ms=1000 termination=Cancelled outer=0 block=9 file=0 skipped_n=1 skipped_text=["1 pending cleanups did not run because a host interrupt stopped the retirement."] cancels=1 tags=["WithCleanup::skipped", "General::cancel"]
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.81s
EXIT:0
```

Settlement: F1 returns `Cancelled` at the second cancel (about 1.000 s), with one `General::cancel` and skipped count 0. It does not return before that cancel. F2 returns 3 with `termination=None` and no messages. F3 leaves the outer marker unset, restores the Block variable to 9, rolls the file assignment back to 0, and reports one skipped cleanup plus one `General::cancel`. Control b is 500 ms on all three runs, against the director's HEAD samples of about 502 ms. Control d finishes with `k630=20000` and one `General::cancel` (221 ms, 281 ms, 277 ms). Nested contract stays `List[1, 1, 0, 0]`. Runs-once stays `hits=1`. File rollback stays `marker=0 rolled=0`.

### fmt, reference, host, runtime

`.temp/brief-630/logs/b1-fmt.log` and `.temp/brief-630/logs/b1-fmt-2.log` are each `EXIT:0`. The second check is after the frame-root pin edit.

`.temp/brief-630/logs/b1-builtins.log`:

```
wrote 643 builtin entries to /home/remi/dev/rum/crates/rum-stdlib/../../docs/reference/builtins.md
EXIT:0
```

`docs/reference/builtins.md` line 6093, written by that generator:

```
  - WithCleanup::skipped — `1` pending cleanups did not run because a host interrupt stopped the retirement.
```

`.temp/brief-630/logs/b1-host.log`, `EXIT:0`. Every `test result:` line:

```
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 50.24s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.74s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.97s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.92s
```

Those are, in order: `rum_host` lib, `adapter_contract`, `batch_boundary_ownership`, `capture_output_lifecycle`, `guest_loading_resume`, `host_payload_destruction_contract`, `installer`, `lifecycle_buildout`, `lifecycle_contract`, `logical_profile_abandonment`, `logical_profile_lifecycle`, `native_stack_thread_scope` (8 passed), `random_password_policy`, `retirement_interrupt` (8 passed), `shared_accumulator_completion`, `stress`, `suspended_timeout_contract`, `suspension_abandonment_contract` (12 passed), and doc-tests.

`.temp/brief-630/logs/b1-runtime-lib.log` failed the old call pin, then the pin was updated:

```
assertion failed: drain_body.contains("self.abandon_frames::<profile_mode::Ordinary>(suspension,reason);")
test result: FAILED. 1215 passed; 1 failed; 5 ignored; 0 measured; 0 filtered out; finished in 32.08s
EXIT:101
```

`.temp/brief-630/logs/b1-runtime-lib-2.log`:

```
test result: ok. 1216 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 29.47s
EXIT:0
```

`.temp/brief-630/logs/b1-frame-root-trace.log`:

```
named_suspension_frames=567
named_pure_finalizers=59
pure_frame_abandon_derives=534
handwritten_stateful_frame_abandons=33
evaluate_cleanup_allowlist=WithCleanup owner frames (2)
raw_live_owner_clears=0
raw_live_owner_replacements=0
removed_cancellation_lane_hits=0
legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1206 filtered out; finished in 1.51s
EXIT:0
```

The same log prints 33 `stateful_frame_abandon=` lines between the counts above. No new frame was added.

### Standing inventories

Contract fingerprints, 68 rows, last column only. Other columns unchanged. From `.temp/brief-630/logs/b1-fingerprints.log`: `Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv`, `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.17s`, `EXIT:0`. Installed into `inventory/local-contracts.tsv`. Every changed cell, from `.temp/brief-630/fingerprint-changes.tsv`:

```
B01	fnv1a64:0637085007c46d1d	fnv1a64:9850d078b060843d
B02	fnv1a64:be7999858dddd664	fnv1a64:4c277aabc4353448
B03	fnv1a64:1a9f2168f2f02f3b	fnv1a64:c476dd3bf63e5157
B04	fnv1a64:b6d805f657c0d845	fnv1a64:3c2e9190fd4d62a5
B05	fnv1a64:bc4deb0bb366eb60	fnv1a64:5dd98965c7844034
B06	fnv1a64:275e07f943536671	fnv1a64:9a23ee0392ac5351
B07	fnv1a64:7d7241ef50a3b310	fnv1a64:2978bf129d208fc4
B08	fnv1a64:3e1707ed47900ef7	fnv1a64:386921c83607fb03
B09	fnv1a64:b493a2b48b796519	fnv1a64:c554de29643a7c99
B10	fnv1a64:a6089f400cd28e62	fnv1a64:ce47275c7d882cf2
B11	fnv1a64:782fb75371b4fe45	fnv1a64:fd8642ee174188a5
B12	fnv1a64:73a022b06781fa19	fnv1a64:84615e2540431199
I01	fnv1a64:e418f34d747c639d	fnv1a64:7632bb761d187abd
I02	fnv1a64:138f689fab9d7360	fnv1a64:b51b06f9bfbac834
I03	fnv1a64:1ad31d53de965759	fnv1a64:286fd8c171b6eb59
I04	fnv1a64:82e20d074c540604	fnv1a64:bf44659a06b8ebe8
P01	fnv1a64:a3bc634ecdff6491	fnv1a64:8d15a4790402c0f1
P02	fnv1a64:58ffb7a7cdfc5772	fnv1a64:5398c6a5b6f10022
P03	fnv1a64:54486ad9c92ed5d3	fnv1a64:e5842285685fc46f
P04	fnv1a64:b354b79b1286f264	fnv1a64:410298c148de5048
P05	fnv1a64:5f617015938d4ded	fnv1a64:6d4f62044df78c2d
P06	fnv1a64:c41f4780f3b19399	fnv1a64:d4e082f5cc72ab19
P070	fnv1a64:fe6ce3c2d2270d99	fnv1a64:0f2e1f37aae82519
P080	fnv1a64:f1af447c59a80ea2	fnv1a64:136fb242f8079bb2
P090	fnv1a64:562197f3ab7670a4	fnv1a64:470d5c933c6a4708
P100	fnv1a64:de4422e9504628f5	fnv1a64:4d5015a0e0ecc235
P110	fnv1a64:2b16446112d6edba	fnv1a64:4c1cc1ebadc9248a
P120	fnv1a64:214f1df32ebce417	fnv1a64:4ce62fbee54488a3
P130	fnv1a64:213f9b4c7f578d10	fnv1a64:cd46186fcbd469c4
P140	fnv1a64:0c43fa2946d4a11c	fnv1a64:70e74b6c7758b040
P150	fnv1a64:ce1528d3e72b6744	fnv1a64:5f0acb6cecc750a8
P071	fnv1a64:704f008c83ccc0b0	fnv1a64:0860a646793e6464
P081	fnv1a64:ffae1d8ac5d95539	fnv1a64:d8d9cc49591bc339
P091	fnv1a64:c7fc2c0db4bea0d1	fnv1a64:c4b1d9d7c7a510b1
P101	fnv1a64:2319352e483a8aba	fnv1a64:441fb2b8e32cc18a
P111	fnv1a64:6585c5d579a059c9	fnv1a64:803397b1af986969
P121	fnv1a64:fed9ff3c02541906	fnv1a64:66a23b9c08ccb696
P131	fnv1a64:5482285c8e14f9c5	fnv1a64:d9d8b3f733a18425
P141	fnv1a64:e0e3c18fd43941b1	fnv1a64:c0e7b4e2d4679c11
P151	fnv1a64:c77d9059597a77d7	fnv1a64:0d10b5501d3c94e3
B13	fnv1a64:b56e9b8273f02271	fnv1a64:2834818cc3490f51
R01	fnv1a64:9c6213914aaad954	fnv1a64:c1e540dd63326758
R02	fnv1a64:fc983dc7698ff213	fnv1a64:b288df38e45fe22f
R03	fnv1a64:34c717f43621e198	fnv1a64:93ffb28cc260b52c
R040	fnv1a64:23a59ec9a4cbf474	fnv1a64:7950e8bc00cc68f8
R050	fnv1a64:362d2feeadebdacd	fnv1a64:06f691c26747390d
R060	fnv1a64:f19fac63ed5df6cc	fnv1a64:73c1970428a88dd0
R070	fnv1a64:681be5a4a77b1439	fnv1a64:414794633abd8239
R080	fnv1a64:49e675fa1bd8cf04	fnv1a64:8648ce8cd63db4e8
R090	fnv1a64:c0bf9a9800adbe35	fnv1a64:ca5319b073f800f5
R100	fnv1a64:dfc4c5bb9cc5d9bb	fnv1a64:899c818ea013fbd7
R110	fnv1a64:aba2068072c374ea	fnv1a64:f81de1767a462a1a
R041	fnv1a64:9ba7feac8b17e171	fnv1a64:0e6de4b6da70ce51
R051	fnv1a64:e443c60938a47d24	fnv1a64:d52f8aa8c9985388
R061	fnv1a64:1df3e29faf94d7c7	fnv1a64:0f9d87d7f9cf68b3
R071	fnv1a64:3a2125f2abab884e	fnv1a64:1691230f815fc9be
R081	fnv1a64:9a2e8c3debf1fdf5	fnv1a64:093a7ef57c989735
R091	fnv1a64:9422cbd150aae2dc	fnv1a64:34eea25a0bea1480
R101	fnv1a64:b7aa2f77562a470c	fnv1a64:9c2d18d44ba94c90
R111	fnv1a64:5c5e60432c4c3ebf	fnv1a64:84a77b85381c440b
R12	fnv1a64:7aa726d15d87717f	fnv1a64:d323bc79b109a74b
R13	fnv1a64:c5d595e0e13b378b	fnv1a64:8d8e918eb21c1ac7
R14	fnv1a64:f89c315f2cf6e8f5	fnv1a64:67a82416bd9d8235
R15	fnv1a64:f9c0a5aad1a21701	fnv1a64:41a4d646ae020881
O010	fnv1a64:ef6eb32e028e7427	fnv1a64:e803fcb338ddf993
O0200	fnv1a64:070b9bfd958bc71c	fnv1a64:6baeed40c60fd640
O011	fnv1a64:342868ebfaefbb4c	fnv1a64:b64a538c363a5250
O0210	fnv1a64:c080feddf148d0df	fnv1a64:d26fc4e56e05c6ab
```

Certificate cells: status, identity, source anchor, preconditions, strength, and exclusions did not change. The hash=true and hash=false certificate columns did. Direct-contract ids are the same; their stamps are the new fingerprints above. Body fingerprints and marks moved. From `.temp/brief-630/certificate-hash-true-diff.txt`: C01 hash=true body `fnv1a64:92f9f9409d3e211f` to `fnv1a64:3649719a8374cc91`, marks `fnv1a64:7a42cfb70792ad9b` to `fnv1a64:a483d6bb9a9c7f59` and `fnv1a64:b4158dd3472c88f5` to `fnv1a64:89fc3a86a38213e7`. C02 hash=true body `fnv1a64:75afc5380cf7a3d5` to `fnv1a64:164ca566d9b8717f`, mark `fnv1a64:9ce09c2d2e22ef57` to `fnv1a64:222358e7827eccc4`. From `.temp/brief-630/certificate-hash-false-diff.txt`: C01 hash=false body `fnv1a64:a3ae9baa9b90ae86` to `fnv1a64:ec576d20120e35d4`, marks `fnv1a64:40ad50a7c3343912` to `fnv1a64:e97d19968bf93469` and `fnv1a64:a8b12227195a1c54` to `fnv1a64:db33297d7ca6523b`. C02 hash=false body `fnv1a64:f447ac0ccb5076d8` to `fnv1a64:11bb90772cdf912e`, mark `fnv1a64:a934e6a3bdd7473d` to `fnv1a64:47a3697b4c67926d`. Domain fingerprints did not change.

The first acceptance runs failed on the stale cells and wrote the review files. `.temp/brief-630/logs/b1-accept-hash.log`: `PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0`, `stale or omitted instance certificate C01 hash=true`, `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 18.88s`, `EXIT:101`. `.temp/brief-630/logs/b1-accept-nohash.log`: the same production line, `stale or omitted instance certificate C01 hash=false`, `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 18.87s`, `EXIT:101`.

After copying only those certificate columns, `.temp/brief-630/logs/b1-accept-hash-2.log`:

```
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 18.71s
EXIT:0
```

`.temp/brief-630/logs/b1-accept-nohash-2.log`:

```
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 18.82s
EXIT:0
```

Ownership. The first run (`.temp/brief-630/logs/b1-ownership.log`) printed `EXACT INVENTORY self-reaching types: 61 rows` and `EXACT INVENTORY UnknownHostPayload: 79 rows`, then failed on UnknownConstruct. After the replay, `.temp/brief-630/logs/b1-ownership-2.log`:

```
EXACT INVENTORY self-reaching types: 61 rows
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7923 rows
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 6.03s
EXIT:0
```

No new `UnknownHostPayload`. No row points at a file this brief did not edit. Paired by file and construct, 15 rows moved and 5 are new. Zero disappeared. The three macro rows whose description includes an end position moved with that end position; they are the same `matches!` and `debug_assert!`.

From `.temp/brief-630/ownership-changes.tsv`, verbatim:

```
change	file	old_line	old_col	new_line	new_col	description
move	crates/rum-runtime/src/runtime/debug_tools/monitor.rs	106	31	112	31	closure captures: |current| (capture types/modes unresolved)
move	crates/rum-runtime/src/runtime/debug_tools/monitor.rs	107	35	113	35	closure captures: |current| (capture types/modes unresolved)
move	crates/rum-runtime/src/runtime/debug_tools/monitor.rs	95	18	101	18	closure captures: |scope| (capture types/modes unresolved)
move	crates/rum-runtime/src/runtime/debug_tools/monitor.rs	138	24	144	24	closure captures: || (capture types/modes unresolved)
move	crates/rum-runtime/src/runtime/debug_tools/monitor.rs	129	1	135	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace)
move	crates/rum-runtime/src/runtime/model/request_cancellation.rs	38	17	48	17	rum_runtime::runtime::model::request_cancellation::AcceptedRequest.generation: foreign representation std::num::NonZeroU64: NonZeroU64
move	crates/rum-runtime/src/runtime/model/request_cancellation.rs	96	17	114	17	rum_runtime::runtime::model::request_cancellation::RequestCancelHandle.generation: foreign representation std::num::NonZeroU64: NonZeroU64
move	crates/rum-runtime/src/runtime/model/request_cancellation.rs	152	12	174	12	rum_runtime::runtime::model::request_cancellation::RequestInterruptState.phase: foreign representation std::sync::atomic::AtomicU64: AtomicU64
move	crates/rum-runtime/src/runtime/model/request_cancellation.rs	225	5	274	5	unexpanded macro debug_assert! (ends at 274:62)
move	crates/rum-runtime/src/runtime/model/request_cancellation.rs	196	9	250	9	unexpanded macro matches! (ends at 253:10)
move	crates/rum-runtime/src/runtime/model/request_cancellation.rs	14	1	24	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
move	crates/rum-runtime/src/runtime/model/request_cancellation.rs	93	1	111	1	unresolved attribute/expansion derive (Clone)
move	crates/rum-runtime/src/runtime/model/request_cancellation.rs	122	1	142	1	unresolved attribute/expansion derive (Clone)
move	crates/rum-runtime/src/runtime/session/guest_loading/scope.rs	146	1	150	1	unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)
move	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	100	24	107	24	unexpanded macro matches! (ends at 107:87)
new	crates/rum-runtime/src/runtime/model/request_cancellation.rs			275	5	unexpanded macro debug_assert! (ends at 275:38)
new	crates/rum-runtime/src/runtime/suspension/core/abandon.rs			189	22	closure captures: |cell| (capture types/modes unresolved)
new	crates/rum-runtime/src/runtime/suspension/core/abandon.rs			181	26	closure captures: |deadline| (capture types/modes unresolved)
new	crates/rum-runtime/src/runtime/suspension/core/abandon.rs			109	29	unexpanded macro debug_assert! (ends at 112:30)
new	crates/rum-runtime/src/runtime/suspension/core/abandon.rs			163	17	unexpanded macro format! (ends at 165:18)
```

The five new rows are the second `encode` assert, the depth assert, the skipped-message `format!`, and the two cold closures.

### Rejection census — STOP

`.temp/brief-630/logs/b1-census.log`. The line pins for `abandon.rs` 43, 44, and 94 are before this assert, and the failure is the length assert, so those three sites are still present. Kind-map assert did not run. `census/tests.rs` was not left edited.

```
assertion `left == right` failed: STOP: brief-545 exact legacy frontier inventory moved
  left: 1003
  right: 972
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 18.35s
EXIT:101
```

Current kinds, from the dump footer `# counts {RET001: 10, RET002: 962, RET003: 31} len 1003` in `.temp/brief-630/census-frontier-now.tsv`:

| kind | asserted | actual | delta |
| --- | --- | --- | --- |
| RET001 | 10 | 10 | 0 |
| RET002 | 934 | 962 | +28 |
| RET003 | 28 | 31 | +3 |
| total | 972 | 1003 | +31 |

`request_cancellation.rs`, `scope.rs`, `session_requests.rs`, and `catalog/control.rs` contribute 0 rows to this frontier. The rows on statements B1 added or rewrote in `abandon.rs`, keyed by kind, file, callee, and reason. `raw_count` is the frontier set size at that line, witnesses included. Verbatim from `.temp/brief-630/census-comparison.tsv`:

```
kind	file	line	callee	reason	raw_count
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	98	-	primitive operand proof or exact operator/conversion impl required	4
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	98	rum_runtime::runtime::model::session::Session::drain_host_interrupt_pending	method syntax: opaque imported trait candidate std::sync::Arc	2
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	99	-	primitive operand proof or exact operator/conversion impl required	4
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	100	-	primitive operand proof or exact operator/conversion impl required	2
RET003	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	107	-	unexpanded macro; contents are not resolved Rust	2
RET003	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	109	-	unexpanded macro; contents are not resolved Rust	2
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	121	rum_runtime::runtime::model::session::Session::drain_host_interrupt_pending	method syntax: opaque imported trait candidate std::sync::Arc	2
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	122	-	primitive operand proof or exact operator/conversion impl required	2
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	149	-	primitive operand proof or exact operator/conversion impl required	1
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	150	-	method syntax: method receiver lacks checked nominal/place provenance	1
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	151	-	method syntax: method receiver lacks checked nominal/place provenance	1
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	154	-	method syntax: method receiver lacks checked nominal/place provenance	2
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	154	-	primitive operand proof or exact operator/conversion impl required	1
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	159	-	primitive operand proof or exact operator/conversion impl required	1
RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	160	rum_runtime::runtime::model::session::Session::push_message	method syntax: type is outside the admitted surface: impl Into < String >	1
RET003	crates/rum-runtime/src/runtime/suspension/core/abandon.rs	163	-	unexpanded macro; contents are not resolved Rust	1
```

Those 16 keys are 29 raw rows. The asserted total moved by 31. This run did not have the previous 972-row set (git is not available), so the remaining 2 are not separated from rows that `begin_ordered_abandonment` already had. The full current frontier is `.temp/brief-630/census-frontier-now.tsv` (1003 data rows, 980 unique once identical dump lines are collapsed, 23 sites stored twice).

LP01, `.temp/brief-630/logs/b1-lp01.log`:

```
test oracle::fixtures::logical_profile::logical_counter_port ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 66.52s
EXIT:0
```

Typed-slot census, `.temp/brief-630/logs/b1-typed-slot.log`:

```
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 284.61s
EXIT:0
```

### Workspace

Not started. The rejection-census stop fired first. The director rules on the count and updates `census/tests.rs`.

STOP.

## Director gate of B1 (2026-10-28 15:00) — B1 accepted

**Accepted, with four changes of mine.** The drain does what the ruling
asked. I settled the census stop, made two small shape changes and added one
pin, replayed the inventories after my edits, and ran the gate myself. My
scripts and logs are in `.temp/director/gate-630b1/`.

### The census stop, settled

Your comparison had no base, so it could not separate every row. I built the
base's frontier in a worktree at `d1baa06a`, with the same temporary dump
(`frontier-head.tsv`: 972 rows, 10/934/28, equal to the assertion). Then I
paired the two frontiers through the diff of `abandon.rs` (`pos.py`):

- All 34 base rows in `abandon.rs` pair with a candidate row of the same
  kind, column, callee and reason at their shifted line. No row retires,
  in any file.
- All 31 new rows sit on lines B1 added to `abandon.rs`: 15 primitive
  operands, 8 method receivers, 4 calls of the cold
  `drain_host_interrupt_pending` (opaque `Arc` candidate), the skipped
  message's `push_message` (`impl Into<String>`), and 3 unexpanded macros
  (the depth `debug_assert!` and the message's `format!`). Rows in the
  shared body count once per mode witness, as since LP02.
- Your list counted the `matches!` at line 107 as new. It is the base's
  `matches!(cleanup_outcome, …)` moved from line 100. It also missed the
  skip branch's two receivers at 102 and 105 (two witnesses each). That is
  where your missing 2 went.

`census/tests.rs` now asserts 1003 (10/962/31), with the comment.

### My changes

1. **`cancel()` without the wildcard.** The match listed all four flag values
   and then `_ => return` for a value the masked word allows but never holds.
   Setting the unconsumed bit is the same transition and is total: ACTIVE to
   CANCELLED, CONSUMED to ESCALATED, and a word that already has the bit is
   unchanged.
2. **Typed assertions in `retirement_interrupt.rs`.** The tests formatted
   their facts into a string and then parsed or searched it (`|assert_ms=`,
   `detail.contains("cancels=1")`). They now return a `Retired` value and
   assert on it: the exact tag list (`["General::cancel"]`, or
   `["WithCleanup::skipped", "General::cancel"]`), the termination, the
   value and the skipped text. F3 also asserts that the inner cleanup ran
   until the second cancel (900 to 3000 ms), which is what shows the first
   cancel was consumed. The controls now assert their termination and tags
   too.
3. **A pin for the expired deadline.** `eval_with_timeout` documents that
   time spent waiting for input counts toward the timeout, and that resume
   gets no fresh budget. So when the deadline has passed before the cancel
   retires the suspension, the drain finds an interrupt before its first
   cleanup and enters none. `expired_deadline_before_retirement_skips_every_cleanup`
   pins that: 100 ms timeout, 250 ms wait, cancel, retire. The cleanup's
   marker stays 0, and one `WithCleanup::skipped` is reported. At HEAD the
   short cleanup ran (`marker=1`, no message) and a long one would have been
   cut silently at its 1024th poll.

The rewritten file at HEAD (copied into the worktree, one test per process,
three runs each): F1, F2 and F3 hang past 8 s, 3 of 3; the expired-deadline
pin fails with `left: "1" right: "0"`, 3 of 3; the five controls pass, 3 of 3.
On the candidate all nine pass, three runs.

### Review of the change

- The phase word gains a second flag bit. The poll's frequency and its
  caller are unchanged. Inside the masked poll, `is_cancelled` is now a
  shift compare and a two-value test, once per 1024 polls.
- Both retirement sites now take the ticket after the drain, so the drain
  runs with the ticket installed. `finish()` still observes the cancel:
  `close_and_observe` reports any flag value other than ACTIVE.
- The depth assertion's entry depth is the cleanup frame's own context
  depth, which `restore_resume_context` installs before the cleanup, so the
  assertion compares real depths.
- `CancellationObserver` (lazy pulls) follows the unconsumed bit. A lazy
  pull inside a cleanup is stopped by an escalation and not by the consumed
  cancel, as the poll is.
- Every drain that can evaluate a cleanup runs inside a request or retires a
  suspended one, so the deadline it reads is that request's own.

### Inventories, replayed after my edits

- Contract fingerprints: 68 last-column cells, every other column equal.
- Certificates C01 and C02, hash=true and hash=false cells, from
  `local_production_acceptance` in both builds; both pass after the copy.
- Ownership: my `cancel()` edit moves three macro rows of
  `request_cancellation.rs` up one line (`matches!` 250 → 249,
  `debug_assert!` 274 → 273 and 275 → 274). Nothing else. 7923 rows exact.
- Census: 1003, passes.

### Gate

- `cargo fmt --check` passes.
- `retirement_interrupt`: all nine tests pass, three runs.
- The whole workspace, single-threaded in a 14G scope: 56 of 57 test
  binaries pass. The 57th, rum-stdlib's lib tests, lost its process to the
  host-wide OOM killer, not to its scope (peak 10.1 GB of 14). `kswapd0`
  invoked it with 21.9 GB free in the Normal zone. That is the MGLRU
  `min_ttl_ms` valve the host memory notes describe. It killed the
  `local_probe_claims_match_stable_rustc` oracle's rustc at 12:54 and then
  the test binary itself at 12:57. Rerun alone: 1110 passed and 1 failed,
  `local_option_read_claims_match_stable_rustc`, whose rustc the same valve
  killed at 13:15 (its log ends in warnings, with no error). The two rustc
  oracles then pass alone, 2 of 2, with no kill. In all: 3986 passed, 61
  ignored, which is 629's 3977 plus the nine new tests.
- After change 4, I reran everything that reads the text or the file:
  - fingerprints regenerated (the same 68 last-column cells);
  - C01 and C02 cells copied from both builds, and `local_production_acceptance`
    passes in both;
  - ownership 7923 rows exact, and the census 1003, passes;
  - the reference drift test and the 20 message tests pass;
  - `retirement_interrupt` 9 of 9, three runs;
  - `rum-host` 191 passed, 0 failed, in 19 binaries.

### Host documentation

`docs/guide/embedding.md`, Cancel And Timeout, now says what a retirement
does. It runs the `WithCleanup` cleanups that the suspended work still owes.
The retiring cancel does not stop them. A further cancel, or the request's
own deadline, stops the running cleanup and skips the rest, with one
`WithCleanup::skipped` before `General::cancel`. Nested file rollbacks
complete either way, `Monitor` callbacks do not run, and a deadline that
passed while waiting for input skips every cleanup. `retirement_interrupt.rs`
names that paragraph as its contract.

### Measurement

Both arms were built from one source path (`same/src`) with one codegen unit:
the base at `d1baa06a`, and the candidate from the working tree
(`CAND-SOURCE-EQUAL`). `path-calibration.sh` prints PASS for `resume` and for
`rum-cli`. The arms carry the B1 text of the skipped message. Change 4 alters
one string and no code, and I did not rerun the panels for it.

**Structure.** I diffed the disassembly function by function. Absolute
addresses and rip-relative offsets are normalized, and a `<sym+off>` data
comment is reduced to its symbol, since data offsets move with layout.

- `resume`: 18 of 10 124 common functions differ.
- `rum-cli`: 24 of 10 504 common functions differ; 1 function exists only in
  the base and 6 only in the candidate.

Every differing function is one B1 changed, or one that inlines one: the
drain and its worklist, the profiled abandonment body, the monitor's two
early returns (`push_trace_with_depth` inlines the monitor's trace hook), the
cancellation word and the drops of `AcceptedRequest`, the retirement and
finish sites, the guest-form install, and `poll_host_interrupt_if_due`. The
two mimalloc functions in the list differ only in the padding after their
`ret`. No evaluator function changed. The poll gains five instructions
(generation compare, mask, two-value test), and they run once per 1024
polls.

**Resume panel, 121 labels.**

- Three reps: base against base reads median +0.002%, with 22 labels above
  +0.1%. Base against candidate reads median −0.077%, with 15 above +0.1%;
  the highest is `lhs-twice-cycle` at +0.40%.
- Fifteen interleaved reps: median −0.074%, maximum +0.089%, and no label
  above +0.1%. `lhs-twice-cycle` reads −0.05%.
- Callgrind on three labels (`lhs-head-cycle`, `resume-s6-readonly-twice`,
  `resume-s9-cleanup-three-phase`; six processes per arm at N and 2N): the
  only deterministic delta is `resume_eval_request_from_checkpoint`, at −44,
  −138 and −104 instructions per iteration (integer slopes). It is a site B1
  changed, 4 668 → 4 643 static instructions. The rest is the interner's
  per-process hash noise in `Symbol::system`, `intern_symbol_text` and
  `intern_text`: non-integer medians, both signs.

**CLI panel, 192 labels, fifteen interleaved reps.** Median −0.003%.
30 labels read above +0.1%. Of those, 26 overlap the base per rep, e.g.
`valueq-undefined-symbol-definitions-2000` at +2.8% over a base spread of
2 733–5 790. The other four sit apart from the base in every rep, and they
are one mechanism:

| label | per iteration | base reps | candidate reps |
|---|---|---|---|
| `deletecases-wide-association-no-match` | +54 281 (+0.72%) | 7 502 217–7 502 852 | 7 556 188–7 556 893 |
| `keysort-default-wide` | +53 551 (+0.65%) | 8 258 434–8 262 169 | 8 313 267–8 315 280 |
| `keysortby-default-wide` | +53 624 (+0.35%) | 15 450 095–15 453 219 | 15 504 375–15 506 666 |
| `association-set-key-incremental-large` | +49.6 (+0.29%) | 17 194–17 236 | 17 254–17 286 |

Callgrind puts the whole delta in glibc's `__memcmp_avx2_movbe`.

- On `keysort-default-wide` it is +53 763 of +55 615.
- On `association-set-key-incremental-large` it is +47.7 of +45.1, so it is
  larger than the total.

Every caller makes the same number of calls in both arms. In the
`keysort-default-wide` profile, for example, `Symbol::is_system_name` calls it
140 870 times in each. Each call costs 19.0 instructions in the base and 27.0
in the candidate. The same is true from `Symbol::system`.

The program and its data are the same in both arms, so the per-call count of
that routine can differ only through the addresses it compares. Its
short-length path first checks whether either operand lies within one vector
of a page end (`(s1 | s2) & 4095`) and takes a longer branch when it might.
B1 adds strings to `.rodata` and one message registration to startup, and
the operands' addresses moved across that check. This is the brief-629
`.rodata` memcmp class, here in instructions rather than cycles. It is
layout, not work B1 added: none of the four labels runs a function B1
changed.

`deletecases-wide-association-no-match` (N=1600, 7.5 M instructions per
iteration) is too slow for six callgrind processes per arm. I did not
attribute it separately. Its delta per association entry, 33.9 instructions,
matches the two `keysort` labels' 33.5.

On the other side, 38 labels sit below the base in every rep, from −0.02% to
−0.56% (`lazy-tuples-take-small`). I did not attribute them.

**Verdict.** There is no default-path cost: the only deterministic code
delta is a saving at a retirement site, and the one rise above the noise is
memcmp operand placement.

## Stage B2 (2026-10-28 15:20) — the window: stamped output, one cumulative convention, then STOP — grk

B1 is committed (`854cfa4a`); its gate is above. B2 goes to grk on the same
session: you designed this window as code in A3 and hold the A2 matrix.

**The defect, from your A2 and A3 evidence.** A host cut is a `usize` taken
from a session list's length before a restore can shorten that list. On the
file and nested routes the cut then panics (`split_off`, the trace slice) or
the cancel notice falls below it. On the script route the retirement cut is
the live length after the body, so the outer cleanup's message and
`$MessageList` lose it. On every route, the front ends' cumulative skip-by-count
loses an outer cleanup's message once a rollback removed an item that was
already reported. P1–P3 (my A2 ruling) are the contract; the shape below is
the one I ruled in A3.

**The shape.** New module `crates/rum-runtime/src/runtime/model/output_log.rs`:

```rust
/// A position in the session's output order. Every push to a session output
/// list takes the next stamp; nothing ever hands one out again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) struct OutputStamp(u64);

/// Everything stamped at or before this point. A mark is a value of the
/// session's output clock, not a position in any list, so it means the same
/// thing before and after any snapshot restore or rollback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) struct OutputMark(u64);

/// One session output list. Items are sorted by strictly increasing stamp:
/// a restore only brings back items stamped before every later push.
#[derive(Clone, Debug)]
pub(in crate::runtime) struct OutputLog<T> {
    items: Vec<(OutputStamp, T)>,
}

impl<T> OutputLog<T> {
    pub(in crate::runtime) fn push(&mut self, stamp: OutputStamp, value: T);
    /// The items stamped after `mark`, oldest first.
    pub(in crate::runtime) fn after(&self, mark: OutputMark) -> impl ExactSizeIterator<Item = &T>;
    /// The items stamped after `from` and at or before `through`.
    pub(in crate::runtime) fn between(&self, from: OutputMark, through: OutputMark)
        -> impl ExactSizeIterator<Item = &T>;
    pub(in crate::runtime) fn split_after(&mut self, mark: OutputMark) -> Vec<T>;
    pub(in crate::runtime) fn truncate_after(&mut self, mark: OutputMark);
    pub(in crate::runtime) fn iter(&self) -> impl ExactSizeIterator<Item = &T>;
    pub(in crate::runtime) fn len(&self) -> usize;
}
```

- `Session` gains `output_clock: u64`, `fn output_mark(&self) -> OutputMark`
  and `fn next_output_stamp(&mut self) -> OutputStamp` (increment, then
  stamp). The clock is never reset or restored, by any snapshot.
- The four session lists become `OutputLog`s: `messages`, `prints`,
  `handled_non_loader_eval_errors`, and one `OutputLog<(TraceEvent, usize)>`
  for `trace_events` with `trace_event_depths`. `items` is private. **No
  positional cut survives**: there is no `truncate(usize)`, `split_off(usize)`,
  index or range slice, and no `splice`. The compiler enumerates the sites.
  Whole-list operations (`mem::take`, replace from a snapshot, clone into a
  snapshot) stay; they keep the stamps.
- Every stored checkpoint becomes an `OutputMark`: `RequestCheckpoint`,
  `PendingEvalRequest`, `OutputBoundary`'s three fields,
  `current_message_list_checkpoint`, `GuestFormScope`, `GetCompleteFrame`,
  `GuestScriptCursor`, the evaluation-data checkpoint, the test-report and
  trace capture frames, `Check`/`Quiet`, `Confirm`. A local
  `let n = …len()` that only counts may stay a `usize`.

**What a request has reported.** Per request, append-only, never touched by
a rollback:

```rust
/// What one request has shown its host. Each channel has its own published
/// mark, because an input pause publishes each channel only up to its own
/// provisional boundary.
#[derive(Clone, Debug)]
pub(in crate::runtime) struct ReportedOutput {
    messages: Vec<RuntimeMessage>,
    prints: Vec<String>,
    trace: Vec<TraceEvent>,
    published: ChannelMarks, // messages, prints, trace
}
```

- Created at admission with every published mark at the admission mark.
  Carried in `PendingEvalRequest` across a suspension, **on every route**:
  `eval`, `eval_expr`, `call`, script and file all store their admission mark
  and their `ReportedOutput` when they return `InputRequired`. (My A2 ruling
  rejected an admission mark on batches only because `CleanupOnly` would
  reprint; that mode is deleted below.)
- Publishing appends each channel's session items stamped after its
  published mark and at or before its `through`, then advances that mark to
  `through`. An input pause publishes through the pending `OutputBoundary`
  (a channel with no boundary goes through the current mark). Completion,
  failure and cancellation publish through the current mark.
- Every host outcome's `messages`, `prints` and `trace` are the request's
  reported lists after that publication: cumulative, as `embedding.md`
  already promises. Its memory is charged to `MemoryOwner::PendingEvalRequest`
  while it waits there.
- The session lists keep their meaning: a request still splits its own
  items off at completion (`split_after(admission)`), and `$MessageList`
  (`last_message_list`) is the session view after the rollbacks, as today.
  P1 binds the reported lists only.

**The script route.** The script loop stops splitting each form's output off
and splicing it back on an error. Each form's outcome copies its items
(`after(form_mark)`) and leaves them in the session lists; the request cuts
once, at its admission mark. Per-form `$MessageList` stays the form's own
items.

**Front ends.** Delete `CancellationOutput` from `rum-cli`
(`input_retirement.rs:9-37`; `main.rs:10`, `234`, `246`, `258`, `266` and
`343`; `eval_tests.rs:65` and `447`). One rule for every route: skip what this front
end already emitted for the request, by count. Jupyter and the REPL are
already that rule; do not change them.

**Falsifiers — write them first, in `crates/rum-host/tests/output_window.rs`,
and calibrate each red at HEAD before the crate edit (three runs).**

- **W1, the merge (your `a3_merge`).** `Inner.rum` is
  `probe::inside = "inside message"; Message[probe::inside]; InputString["in"]`.
  The request is `probe::outer = "outer message"; WithCleanup[Get["Inner`"], Message[probe::outer]]`.
  Routes `eval` and `eval_script`; retirement by `cancel()` then `eval("")`,
  and by `cancel()` then `resume_with_input("")`. Expected on all four:
  the input outcome's tags are `["probe::inside"]`; the retirement outcome's
  tags are exactly `["probe::inside", "probe::outer", "General::cancel"]`;
  `$MessageList` afterwards is `List[probe::outer, General::cancel]`; and the
  three front-end merges (CLI skip-by-count with the trailing cancel popped,
  Jupyter, REPL), applied to the two outcomes, each add exactly
  `probe::outer`. At HEAD: the eval route loses the inside prefix and the
  script route returns only the cancel.
- **W2, the window matrix (your `a2_window_matrix`).** Routes file, script,
  eval-get and nested; earlier entries 0, 1, 2; kinds messages, trace and
  handled. Expected in every cell: no panic; the retirement outcome equals the
  input outcome's items followed by exactly one `General::cancel` (trace: the
  same events, cumulative); the following `Check` request reports
  `["General::passlim"]`. At HEAD (`.temp/brief-630/logs/a2-window-1.log`)
  every cell with an earlier entry is red: 10 of the 36 panic, 6 lose the
  cancel, and the other 8 (eval-get, and script trace) drop the reported
  prefix from the retirement outcome. The 12 cells with no earlier entry are
  green at HEAD and stay green.
- **W3, a rolled-back unreported item stays out (P2).** A `Message` inside the
  file after `InputString` cannot have been reported; in-file cleanup output
  rolled back before packaging never appears. Your `k_get_cleanup_inside_file_rolls_message_back`
  and `rum-cli`'s `cleanup_messages_appear_once_with_original_input_reason`
  (file count 0) are the controls; both stay green at HEAD and after.
- **Controls.** Every existing test in rum-runtime, rum-host, rum-cli,
  rum-jupyter and rum-repl keeps its expected values, except the two
  `eval_tests.rs` call sites that lose the enum. `retirement_interrupt.rs`
  stays green unchanged.

**Order of work, in one dispatch.**
1. W1–W3, red at HEAD, three runs.
2. The `OutputLog` conversion alone: stamps, marks and every stored
   checkpoint, with each route still packaging exactly what it packages at
   HEAD. Every existing test passes unchanged, and W1–W3 are as red as at
   HEAD, cell for cell. Record both in the report before step 3. If any
   existing expected value moves here, STOP: a type change that changes
   behaviour has a positional cut the conversion missed.
3. `ReportedOutput`, the admission mark on every route, publication, and the
   script route without the splice.
4. The `rum-cli` deletion.

**Stop conditions.**
- A persisted format (snapshot bytes, `Save`, a dump) contains one of these
  lists: STOP with the site.
- A trace event and its depth are not pushed, cut and replaced together at
  some site: STOP with the site.
- A consumer needs a cut a mark cannot express: STOP with the site and why.
- Any expected value outside the list above would change: STOP with the
  test and both values.

**Scope.**
- **MAY modify:** `crates/rum-runtime/src/runtime/` (the new module, its
  `mod` line and every site the type change reaches); `crates/rum-host/src/`
  where it packages these lists; `crates/rum-cli/src/input_retirement.rs`,
  `main.rs`, `eval_tests.rs`; the new test file; the standing inventories, by
  replay.
- **Disclosure duty:** any other file, with its reason. Anything beyond a
  mechanical change there is a STOP.
- **MUST NOT touch:** `rum-jupyter`, `rum-repl`, `docs/` (I write the
  `embedding.md` sentence at commit), `TODO.md`, `Cargo.lock`. The
  cleanup-error arm (B3) is not in B2.

**Run, single-threaded:** `cargo fmt --check`; W1–W3 three runs each, red at
HEAD first; `cargo test -p rum-host`, `-p rum-cli`, `-p rum-jupyter`,
`-p rum-repl`; `cargo test -p rum-runtime --lib`; the standing inventories;
the whole workspace. Report every `test result:` line verbatim.

**Then STOP.** Measurement follows my gate: every push now takes a stamp,
and each item grows by eight bytes.

Append "Executing-agent report — B2". Every quoted line must be copied from
a file you name. Quote every verdict line your instruments print, and settle
each flagged row.

## Executing-agent report — B2

STOP at step 2. A consumer needs a cut a mark cannot express. Step 3 (`ReportedOutput`, admission publication, script route without the splice) was not started. Step 4 (the `rum-cli` `CancellationOutput` deletion) was not started. B3 was not started. Measurement panels were not run. Standing inventories were not replayed.

The `OutputLog` conversion is in the working tree. Routes still package from the same checkpoints. The script error path still puts the earlier accumulator and the failed form back, in that order, with new stamps (`session_eval.rs:362-389`). That re-push is not what moved the W1 script cells: W1's script is one form, the accumulator is empty, and the splice does not run.

### Compile and format

`.temp/brief-630/logs/b2-step2-check-2.log` line 76: `Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.65s`. Line 77: `EXIT:0`.

`.temp/brief-630/logs/b2-step2-fmt-2.log` line 1: `EXIT:0`. After the allowlist edit, `.temp/brief-630/logs/b2-step2-fmt-3.log` line 1: `EXIT:0`.

### HEAD calibration, three runs, before this remeasure

`.temp/brief-630/logs/b2-w-head-1.log` line 25: `W1VERDICT cells=4 red=4 green=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 97: `W2VERDICT cells=36 panic=10 lost-cancel=6 dropped-prefix=8 check=0 other=0 green=12`

`.temp/brief-630/logs/b2-w-head-1.log` line 106: `W3VERDICT green`

`.temp/brief-630/logs/b2-w-head-1.log` line 115: `test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`

`.temp/brief-630/logs/b2-w-head-1.log` line 118: `EXIT:101`

`.temp/brief-630/logs/b2-w-head-2.log` line 24: `W1VERDICT cells=4 red=4 green=0`

`.temp/brief-630/logs/b2-w-head-2.log` line 96: `W2VERDICT cells=36 panic=10 lost-cancel=6 dropped-prefix=8 check=0 other=0 green=12`

`.temp/brief-630/logs/b2-w-head-2.log` line 105: `W3VERDICT green`

`.temp/brief-630/logs/b2-w-head-2.log` line 117: `EXIT:101`

`.temp/brief-630/logs/b2-w-head-3.log` line 24: `W1VERDICT cells=4 red=4 green=0`

`.temp/brief-630/logs/b2-w-head-3.log` line 96: `W2VERDICT cells=36 panic=10 lost-cancel=6 dropped-prefix=8 check=0 other=0 green=12`

`.temp/brief-630/logs/b2-w-head-3.log` line 105: `W3VERDICT green`

`.temp/brief-630/logs/b2-w-head-3.log` line 117: `EXIT:101`

HEAD W1 cells, `.temp/brief-630/logs/b2-w-head-1.log` lines 21–24. Runs 2 and 3 repeat the same four cells at lines 20–23.

```
W1CELL route=eval retire=eval input=["probe::inside"] kernel=["probe::outer", "General::cancel"] message_list=List[probe::outer, General::cancel] cli=[] jupyter=[] repl=[] panic=false
W1CELL route=eval retire=resume input=["probe::inside"] kernel=["probe::outer", "General::cancel"] message_list=List[probe::outer, General::cancel] cli=[] jupyter=[] repl=[] panic=false
W1CELL route=script retire=eval input=["probe::inside"] kernel=["General::cancel"] message_list=List[General::cancel] cli=[] jupyter=[] repl=[] panic=false
W1CELL route=script retire=resume input=["probe::inside"] kernel=["General::cancel"] message_list=List[General::cancel] cli=[] jupyter=[] repl=[] panic=false
```

### Step 2 remeasure is not cell-for-cell

`.temp/brief-630/logs/b2-w-step2-1.log` line 75: `W1VERDICT cells=4 red=4 green=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 118: `W2VERDICT cells=36 panic=0 lost-cancel=0 dropped-prefix=24 check=0 other=0 green=12`

`.temp/brief-630/logs/b2-w-step2-1.log` line 66: `test w3_rolled_back_unreported_stays_out ... ok`

`.temp/brief-630/logs/b2-w-step2-1.log` line 130: `test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s`

`.temp/brief-630/logs/b2-w-step2-1.log` line 133: `EXIT:101`

This log has no `W3VERDICT` line. The passing test is the one that prints it, and cargo kept that test's output. The harness counts 1 passed and 2 failed, and the passed name is `w3_rolled_back_unreported_stays_out`.

Step 2 W1 cells, `.temp/brief-630/logs/b2-w-step2-1.log` lines 71–74:

```
W1CELL route=eval retire=eval input=["probe::inside"] kernel=["probe::outer", "General::cancel"] message_list=List[probe::outer, General::cancel] cli=[] jupyter=[] repl=[] panic=false
W1CELL route=eval retire=resume input=["probe::inside"] kernel=["probe::outer", "General::cancel"] message_list=List[probe::outer, General::cancel] cli=[] jupyter=[] repl=[] panic=false
W1CELL route=script retire=eval input=["probe::inside"] kernel=["probe::outer", "General::cancel"] message_list=List[probe::outer, General::cancel] cli=[] jupyter=[] repl=[] panic=false
W1CELL route=script retire=resume input=["probe::inside"] kernel=["probe::outer", "General::cancel"] message_list=List[probe::outer, General::cancel] cli=[] jupyter=[] repl=[] panic=false
```

**W1 settlement.**

- `eval` / `eval` and `eval` / `resume`: the step 2 lines are the same text as HEAD. Still red. The kernel is `["probe::outer", "General::cancel"]` (inside is gone) and `cli`, `jupyter`, and `repl` are empty.
- `script` / `eval` and `script` / `resume`: moved. HEAD kernel `["General::cancel"]`, message list `List[General::cancel]`. Step 2 kernel `["probe::outer", "General::cancel"]`, message list `List[probe::outer, General::cancel]`. Still red, because the front-end merges are empty and the kernel is not `["probe::inside", "probe::outer", "General::cancel"]`. Input tags stay `["probe::inside"]`. `panic=false` on all four cells at HEAD and at step 2.

**Why a mark cannot express the script cut.** `finish_cancelled_pending_evaluation_request` (`session_requests.rs:58-69`) takes the stored pending checkpoint, or `RequestCheckpoint::new(self)` when `pending_eval_request` is absent, and only then calls `abandon_pending_suspension`. The script request's `InputRequired` path is `finish_completed_request_failure` (`session_requests.rs:399`), which does not store `pending_eval_request`, so the retirement checkpoint is the live clock and, at HEAD, the live length, taken before abandonment. HEAD's W1 script cell shows what that length meant. The file rollback removed `probe::inside`, and the cleanup's `probe::outer` was pushed into that slot, so `split_off` at the stored length took an empty tail. The retirement outcome and `$MessageList` are only `General::cancel`, which is appended after the cut. The mark taken at that same moment means stamps after that clock. `probe::outer` is stamped during abandonment, so `split_after` / `after` include it. A mark has no reading that means "the tail of the current vec after a rollback reused an earlier index."

**W2 settlement.** The verdict counts moved from `panic=10 lost-cancel=6 dropped-prefix=8 … green=12` to `panic=0 lost-cancel=0 dropped-prefix=24 … green=12`. The tuple assert records the same move.

`.temp/brief-630/logs/b2-w-head-1.log` lines 100–102:

```
assertion `left == right` failed: window matrix is not the cumulative contract
  left: (10, 6, 8, 0, 0, 12)
 right: (0, 0, 0, 0, 0, 36)
```

`.temp/brief-630/logs/b2-w-step2-1.log` lines 121–123:

```
assertion `left == right` failed: window matrix is not the cumulative contract
  left: (0, 0, 24, 0, 0, 12)
 right: (0, 0, 0, 0, 0, 36)
```

Unchanged, still green: every `n=0` cell (12). HEAD `.temp/brief-630/logs/b2-w-head-1.log` lines 31–33, 52–54, 67–69, 76–78. Step 2 `.temp/brief-630/logs/b2-w-step2-1.log` lines 82–84, 91–93, 100–102, 109–111. Each line is `class=green` with `retire=["General::cancel"]`.

Unchanged, still `dropped-prefix` (8): script `n=1` trace, script `n=2` trace, and eval-get `n=1` and `n=2` for messages, trace, and handled. HEAD lines 56, 62, 70–75. Step 2 lines 95, 98, 103–108. Each stays `class=dropped-prefix` with `retire=["General::cancel"]`.

Panic at HEAD, `dropped-prefix` at step 2 (10). Both lines copied from the named logs.

`.temp/brief-630/logs/b2-w-head-1.log` line 38: `W2CELL route=file n=1 kind=trace class=panic suspended=true input=[] retire=[] trace_in=1 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 86: `W2CELL route=file n=1 kind=trace class=dropped-prefix suspended=true input=[] retire=["General::cancel"] trace_in=1 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 43: `W2CELL route=file n=2 kind=messages class=panic suspended=true input=["probe::e0", "probe::e1"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 88: `W2CELL route=file n=2 kind=messages class=dropped-prefix suspended=true input=["probe::e0", "probe::e1"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 47: `W2CELL route=file n=2 kind=trace class=panic suspended=true input=[] retire=[] trace_in=2 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 89: `W2CELL route=file n=2 kind=trace class=dropped-prefix suspended=true input=[] retire=["General::cancel"] trace_in=2 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 51: `W2CELL route=file n=2 kind=handled class=panic suspended=true input=["General::passlim", "General::passlim"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 90: `W2CELL route=file n=2 kind=handled class=dropped-prefix suspended=true input=["General::passlim", "General::passlim"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 61: `W2CELL route=script n=2 kind=messages class=panic suspended=true input=["probe::e0", "probe::e1"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 97: `W2CELL route=script n=2 kind=messages class=dropped-prefix suspended=true input=["probe::e0", "probe::e1"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 66: `W2CELL route=script n=2 kind=handled class=panic suspended=true input=["General::passlim", "General::passlim"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 99: `W2CELL route=script n=2 kind=handled class=dropped-prefix suspended=true input=["General::passlim", "General::passlim"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 83: `W2CELL route=nested n=1 kind=trace class=panic suspended=true input=[] retire=[] trace_in=1 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 113: `W2CELL route=nested n=1 kind=trace class=dropped-prefix suspended=true input=[] retire=["General::cancel"] trace_in=1 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 88: `W2CELL route=nested n=2 kind=messages class=panic suspended=true input=["probe::e0", "probe::e1"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 115: `W2CELL route=nested n=2 kind=messages class=dropped-prefix suspended=true input=["probe::e0", "probe::e1"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 91: `W2CELL route=nested n=2 kind=trace class=panic suspended=true input=[] retire=[] trace_in=2 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 116: `W2CELL route=nested n=2 kind=trace class=dropped-prefix suspended=true input=[] retire=["General::cancel"] trace_in=2 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 96: `W2CELL route=nested n=2 kind=handled class=panic suspended=true input=["General::passlim", "General::passlim"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 117: `W2CELL route=nested n=2 kind=handled class=dropped-prefix suspended=true input=["General::passlim", "General::passlim"] retire=["General::cancel"] trace_in=0 trace_out=0`

Lost-cancel at HEAD, `dropped-prefix` at step 2 (6).

`.temp/brief-630/logs/b2-w-head-1.log` line 34: `W2CELL route=file n=1 kind=messages class=lost-cancel suspended=true input=["probe::e0"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 85: `W2CELL route=file n=1 kind=messages class=dropped-prefix suspended=true input=["probe::e0"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 39: `W2CELL route=file n=1 kind=handled class=lost-cancel suspended=true input=["General::passlim"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 87: `W2CELL route=file n=1 kind=handled class=dropped-prefix suspended=true input=["General::passlim"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 55: `W2CELL route=script n=1 kind=messages class=lost-cancel suspended=true input=["probe::e0"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 94: `W2CELL route=script n=1 kind=messages class=dropped-prefix suspended=true input=["probe::e0"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 57: `W2CELL route=script n=1 kind=handled class=lost-cancel suspended=true input=["General::passlim"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 96: `W2CELL route=script n=1 kind=handled class=dropped-prefix suspended=true input=["General::passlim"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 79: `W2CELL route=nested n=1 kind=messages class=lost-cancel suspended=true input=["probe::e0"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 112: `W2CELL route=nested n=1 kind=messages class=dropped-prefix suspended=true input=["probe::e0"] retire=["General::cancel"] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-head-1.log` line 84: `W2CELL route=nested n=1 kind=handled class=lost-cancel suspended=true input=["General::passlim"] retire=[] trace_in=0 trace_out=0`

`.temp/brief-630/logs/b2-w-step2-1.log` line 114: `W2CELL route=nested n=1 kind=handled class=dropped-prefix suspended=true input=["General::passlim"] retire=["General::cancel"] trace_in=0 trace_out=0`

`10 + 6 + 8 = 24` dropped-prefix. Green stays 12. No `check` and no `other`.

**Why a mark cannot express those cuts.** HEAD's panics are the usize cuts. `.temp/brief-630/logs/b2-w-head-1.log` lines 36–37: `thread 'w2_window_matrix' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:714:38:` / `range start index 3 out of range for slice of length 2`. The same file, lines 41–42: `thread 'w2_window_matrix' panicked at crates/rum-runtime/src/runtime/debug_tools/resources/maintained/diagnostics.rs:33:37:` / `` `at` split index (is 2) should be <= len (is 1) ``. Those sites repeat for the other panic cells (indices 3 and 4). `split_off` and the range slice panic when the stored index is past the restored length. `partition_point` on a stamp does not. After the rollback the input prefix is gone, and the retirement is `["General::cancel"]`, which the window records as `dropped-prefix`. The six `lost-cancel` cells had `retire=[]` because the cancel fell below the usize. The mark cut keeps the cancel and still loses the prefix, so those cells are `dropped-prefix` too.

The site is `finish_cancelled_eval_response` (`session_requests.rs:698-710`): `split_accounted_messages`, `split_accounted_prints`, and `trace_values_after` on the stored checkpoint. At HEAD those were the `split_off(usize)` and the trace range slice the log panics in.

**W3 settlement.** HEAD line 106 is `W3VERDICT green`. Step 2's `w3_rolled_back_unreported_stays_out` passed. It was not turned into a failing test. The two named controls were not re-run in this measurement.

### Existing rum-runtime tests

`.temp/brief-630/logs/b2-step2-runtime-lib.log` line 626: `test runtime::tests::frame_root_trace::manual_frame_root_trace_implementations_match_sorted_allowlist ... FAILED`

`.temp/brief-630/logs/b2-step2-runtime-lib.log` line 1030: `test runtime::tests::session_memory::session_memory_field_decision_inventory_is_sorted_and_exhaustive ... ok`

`.temp/brief-630/logs/b2-step2-runtime-lib.log` line 1271: `test result: FAILED. 1215 passed; 1 failed; 5 ignored; 0 measured; 0 filtered out; finished in 47.85s`

`.temp/brief-630/logs/b2-step2-runtime-lib.log` line 1274: `EXIT:101`

That one failure is the manual `FrameRootTrace` allowlist (`frame_root_trace.rs:102`). The scanned side contains `crate::runtime::model::OutputLog<T>`, `crate::runtime::model::OutputMark`, and `crate::runtime::model::OutputStamp`. The allowlist side did not. The full `left` and `right` lines are `.temp/brief-630/logs/b2-step2-runtime-lib.log` lines 1264 and 1265. No other test failed. No Quiet, Check, guest-checkpoint, or memory expected value moved in this run.

Those three impls are the ones this conversion added so logs and marks inside frames trace. I added the three names to `MANUAL_FRAME_ROOT_TRACE_IMPLS`, in sorted order, after `char` and before `f32`. This is the closed inventory of manual impls, the same kind of pin as the schema cardinality below. It is not an evaluation result.

`.temp/brief-630/logs/b2-step2-frame-root-trace.log` line 38: `test runtime::tests::frame_root_trace::manual_frame_root_trace_implementations_match_sorted_allowlist ... ok`

`.temp/brief-630/logs/b2-step2-frame-root-trace.log` line 37: `test runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph ... ok`

`.temp/brief-630/logs/b2-step2-frame-root-trace.log` line 52: `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1206 filtered out; finished in 1.62s`

`.temp/brief-630/logs/b2-step2-frame-root-trace.log` line 54: `EXIT:0`

The session-memory pin is 52 accounted fields and 94 decisions (`session_memory.rs:22` and `:28`). `output_clock` is Exempt (`schema.rs:155`, between `options` and `package_search_paths`). The old accounted `trace_event_depths` row is gone. The test that pins 52 passed in the lib run (line 1030 above). Pair-slot byte formulas use `size_of::<(OutputStamp, T)>()`. That is the mandated eight-byte stamp, not a packaging change.

`rum-host` (beyond this one `output_window` remeasure), `rum-cli`, `rum-jupyter`, `rum-repl`, the standing inventories, and the workspace were not run. The step-2 gate had already fired. I do not claim those expected values are unchanged.

### Stop

A mark cannot express two cuts that HEAD's `usize` expressed.

1. `finish_cancelled_eval_response` (`session_requests.rs:708-710`). The stored index is "drop the first N items of the current vec." After a rollback shortens the list, `split_off` and the range slice panic (the 10 HEAD panics) or the cancel falls below the index (the 6 `lost-cancel` cells). A stamp partition does neither. Those 16 cells are now `dropped-prefix`.
2. The script retirement checkpoint taken before abandonment (`session_requests.rs:58-62`, used by `finish_cancelled_eval_response`). The stored length counted `probe::inside`. Rollback removed it and `probe::outer` reused the slot, so the tail was empty and HEAD's script retirement was only `General::cancel`. The mark includes `probe::outer`. W1 script cells, both retire paths, are the two values.

I did not fake the panics, and I did not special-case the script cut to recreate the `usize`. Step 3 would not make these step-2 cells match HEAD, and the brief says to record this and stop before step 3.

## Director ruling on the B2 stop (2026-10-28 16:45) — continue: step 2b, then steps 3 and 4 — grk

### The stop, settled

You were right to stop. Step 2 said that any moved cell is a STOP, and cells
moved. The checkpoint was mine, and it was miscalibrated. It treated HEAD's red
as behaviour that a type change can keep. Part of HEAD's red *is* the positional
cut:

- a stored index past a restored length panics in `split_off` or in the trace
  slice;
- a stored length read after a rollback takes the slot of a removed item.

A mark has no reading for either, as you said. No consumer needs those
readings, though. They were the defect.

I reran W1–W3 on your tree (`.temp/director/gate-630b2/logs/w-step2-1.log`), and
the verdicts are yours line for line. Every cell that moved left a positional
class: 10 panics, 6 lost cancels, and the two W1 script cells whose cut read
through a reused slot. No cell left green. The 12 `n=0` cells are still green,
and the 8 `dropped-prefix` cells are unchanged. The W1 script cells now give the
kernel the contract's `$MessageList`. Every red cell that remains is the one
class step 3 owns: the retirement outcome lacks items that the input outcome
already reported.

**The rule that replaces step 2's.** A conversion may move a falsifier cell only
out of a positional class (a panic, a lost cancel, a cut through a reused slot).
It may not move a cell out of green, and it may not move a cell into a new class.
Every existing test keeps its expected values.

**Existing tests on your step-2 tree.** My run: `cargo test --workspace
--no-fail-fast`, single-threaded, in a 14G scope, logged to
`.temp/director/gate-630b2/logs/ws-step2-nff.log`.

- 58 binaries: 3967 passed, 22 failed, 61 ignored. That totals 3989, which is
  B1's 3986 plus the three W tests.
- Two of the 22 are W1 and W2.
- The other 20 are standing inventories in `rum-stdlib`, and every one is drift:
  - ownership rows and host-payload ledger rows whose lines moved;
  - `stale contract fingerprint` cells for every contract that reaches
    `Session`, whose field types changed;
  - the census, at 998 instead of 1003. I have settled that one; see below.
- No other test failed. In particular, nothing in `rum-host` outside
  `output_window`, `rum-cli`, `rum-jupyter` or `rum-repl` failed.

So no expected value moved at step 2. Replay the inventories at the end, as the
run list says.

**The census, with an instrument you can run without git.** All of these are
in `.temp/director/gate-630b2/`:

- `frontier-head.tsv`: HEAD's frontier at `9d833875`, 1003 rows (10/962/31).
- `head-wt/`: HEAD's source tree. You may read it; do not build in it.
- `census-dump.patch`: an env-gated dump in `census/tests.rs`.
  - Apply it with `patch -p1 <`, run the census test with
    `RUM_CENSUS_DUMP=<path>`, then remove it with `patch -R -p1 <`.
  - With the patch applied, the census test fails after the dump with
    `stale contract fingerprint` on `merged_option_exprs_for_symbol`, because
    the patch edits `rum-stdlib`. That is expected. The dump is already written
    by then.
- `census-pair.py BASE CAND BASE_ROOT CAND_ROOT OUT`: pairs the rows through
  `diff -U0`. I calibrated it three ways:
  - HEAD against itself: 1003 paired;
  - B1's known answer, `d1baa06a` against HEAD: 972 paired, 0 retired, 31 new,
    all 31 in `abandon.rs`;
  - your step-2 tree: `CENSUSPAIR base=1003 cand=998 paired=986 retired=17
    new=12 base_only=0 cand_only=0`.

  The 17 retired and 12 new rows are the removed
  `push_accounted_trace_event_depths`, `rollback_capture_output`'s four
  truncations becoming three, `next_output_stamp`, `OutputStamp(clock)`, and
  the merged trace push.

At the end, `base_only` and `cand_only` must both be 0, or you settle each such
row by reading it. Put the final count and its row classes in the census
assertion's comment, as B1 did.

**A note on `target/debug`.** `cargo check --workspace --all-targets` failed for
me with 320 errors (`rum_core::Symbol: Copy` not satisfied). Cargo had read a
September `rum-core` rmeta: 28 check fingerprints in `target/debug` were 13 to 22
days newer than their artifacts, so cargo counted the artifacts as fresh. I
removed those 28 fingerprint directories, and the command now passes. Nothing
in the tree changed. Run that command in your gate below.

### Step 2b — the conversion's shape, before step 3

1. **One mark per checkpoint.** One clock stamps all four lists, so a
   checkpoint's per-channel fields always hold one value. Collapse each to a
   single `OutputMark`:
   - `RequestCheckpoint` and `PendingEvalRequest`;
   - `DebugCaptureCheckpoint` and `EvalExprWithStreamCaptureSuspensionFrame`;
   - `GuestFormScope`;
   - `GuestScriptCursor`: one start and one completed end;
   - `GetCompleteFrame`;
   - the `Check`/`Quiet` frames and their locals;
   - the locals in `session_eval.rs` and `test_report/evaluation.rs`.

   `OutputBoundary` keeps one `Option<OutputMark>` per channel: its channels come
   from different frames, and any of them can be absent. The published marks in
   `ReportedOutput` stay per channel too, for the reason the shape gives.
2. **Two assertions lost half their strength.**
   - `typed/tests/driver_faults.rs:100`
   - `tests/capture_output.rs:94`

   `len() == end` meant two things: nothing after the boundary survives, and
   nothing before it is lost. `after(end).len() == 0` keeps only the first.
   Restore both, by content. When the pause is packaged, take the session trace
   stamped at or before the boundary. After the unwind, assert that the session
   trace equals it.
3. **Dead pieces.**
   - The `TraceEventDepths` owner: it renders 0 now. Remove the variant and its
     rows.
   - The `FrameRootTrace` impl for `OutputStamp`, with its allowlist row: no
     frame holds a bare stamp.
   - Three `debug_assert!`s that are always true, since a mark is never past the
     clock: `guest_loading/file.rs:148-149` and `guest_loading/script.rs:148`.
     Also `cursor.print_start <= cursor.completed_print_end`, once the cursor
     holds one start mark.
   - The warnings the conversion adds: `OutputMark::START`,
     `OutputLog::with_capacity` and `is_empty`, `to_vec`, `trace_values`, and
     `W2Cell`'s unread fields. Code that only tests use goes under `#[cfg(test)]`.
     No target may gain a warning.

After 2b: the existing tests are unchanged, apart from the two assertions in
item 2, and W1–W3 match `w-step2-1.log` cell for cell. Record both in the report
before step 3.

### Front ends, added to step 3

W1's `cli`, `jupyter` and `repl` fields apply one function three times. They
test the kernel against a rule, not the front ends. Remove them from W1; W1 keeps
the kernel and `$MessageList` assertions. Then add W1's shape to each front end's
own retirement tests, through that front end's real path:

- `rum-cli`, `eval_tests.rs`: for script, eval and setup, the inside message
  appears once and the outer cleanup message once. For file it appears zero
  times, because the file's rollback discards it, as in
  `cleanup_messages_appear_once_with_original_input_reason` (P2). In every case
  the input reason is the original one.
- `rum-repl`, `tests/input_abandonment.rs`: the eval route; inside once, outer
  once.
- `rum-jupyter`, `src/tests/input_abandonment/`: the execute route; inside once,
  outer once.

Run each one on the 2b tree before step 3 and record its result. There, the eval
route is still HEAD's packaging. I calibrate each at HEAD at the gate. After step
4, all of them are green.

**Scope, widened:**
- **MAY modify:** the three test locations above.
- **MUST NOT touch:** `rum-jupyter/src` outside `src/tests/`, and `rum-repl/src`.

### Then

Steps 3 and 4 as written, and the run list as written. Add
`cargo check --workspace --all-targets` with no new warning, and add the three
front-end tests. Then STOP. Append "Executing-agent report — B2, continued"
under this ruling. Every quoted line must be copied from a file you name, and
every verdict line your instruments print must be quoted.

## Executing-agent report — B2, continued

STOP at step 3. An existing expected value moved. Step 4 (the `rum-cli` `CancellationOutput` deletion) was not started. B3 was not started. Measurement panels were not run. The workspace run and the standing-inventory replays were not started.

### Step 2b, before step 3

Existing rum-runtime tests passed, apart from the two restored trace assertions, which passed inside that same run. W1–W3 match the step-2 cells.

`.temp/brief-630/logs/b2-step2b-check-2.log` line 22: `warning: rum-runtime (lib test) generated 3 warnings`

`.temp/brief-630/logs/b2-step2b-check-2.log` line 23: `Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.60s`

`.temp/brief-630/logs/b2-step2b-check-2.log` line 24: `EXIT:0`

The three warnings are the pre-existing `register_test_*` dead_code warnings.

`.temp/brief-630/logs/b2-step2b-runtime-lib.log` line 251: `test runtime::suspension::typed::tests::driver_faults::cleanup_failure_or_nested_input_never_replaces_the_first_fault ... ok`

`.temp/brief-630/logs/b2-step2b-runtime-lib.log` line 252: `test runtime::suspension::typed::tests::driver_faults::completion_exact_decode_and_probe_survive_rollback_and_replacement ... ok`

`.temp/brief-630/logs/b2-step2b-runtime-lib.log` line 253: `test runtime::suspension::typed::tests::driver_faults::direct_public_fault_is_fatal_at_evaluation_and_test_capture ... ok`

`.temp/brief-630/logs/b2-step2b-runtime-lib.log` line 254: `test runtime::suspension::typed::tests::driver_faults::wrong_arms_and_exact_state_unwind_captures_and_scopes ... ok`

`.temp/brief-630/logs/b2-step2b-runtime-lib.log` line 617: `test runtime::tests::frame_root_trace::manual_frame_root_trace_implementations_match_sorted_allowlist ... ok`

`.temp/brief-630/logs/b2-step2b-runtime-lib.log` line 1192: `test runtime::tests::suspension_abandonment::capture_output::abandonment_rolls_back_all_declared_channels_with_existing_tracing ... ok`

`.temp/brief-630/logs/b2-step2b-runtime-lib.log` line 1249: `test result: ok. 1216 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 43.40s`

`.temp/brief-630/logs/b2-step2b-runtime-lib.log` line 1251: `EXIT:0`

The W cell log was taken before the later W1 field strip. It matches the step-2 cells. `.temp/brief-630/logs/b2-step2b-w-cell-compare.txt` line 4: `COMPARE step2 vs step2b: step2=42 step2b=42`. Line 5: `CELL MATCH`.

`.temp/brief-630/logs/b2-w-step2b-1.log` line 10: `test w3_rolled_back_unreported_stays_out ... ok`

`.temp/brief-630/logs/b2-w-step2b-1.log` line 19: `W1VERDICT cells=4 red=4 green=0`

`.temp/brief-630/logs/b2-w-step2b-1.log` line 62: `W2VERDICT cells=36 panic=0 lost-cancel=0 dropped-prefix=24 check=0 other=0 green=12`

`.temp/brief-630/logs/b2-w-step2b-1.log` line 74: `test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s`

`.temp/brief-630/logs/b2-w-step2b-1.log` line 77: `EXIT:101`

### Front ends on the 2b tree, before step 3

These were not weakened. Eval and setup on the CLI, and the REPL and Jupyter outer counts, are red here because retirement still lacks the input prefix.

`.temp/brief-630/logs/b2-step2b-fe-cli.log` line 6: `CLIVERDICT kind=script status=exit status: 0 inside=1 outer=1 reason=1 cancel=false`

`.temp/brief-630/logs/b2-step2b-fe-cli.log` line 7: `CLIVERDICT kind=eval status=exit status: 0 inside=1 outer=0 reason=1 cancel=false`

`.temp/brief-630/logs/b2-step2b-fe-cli.log` line 8: `CLIVERDICT kind=setup status=exit status: 0 inside=1 outer=0 reason=1 cancel=false`

`.temp/brief-630/logs/b2-step2b-fe-cli.log` line 9: `CLIVERDICT kind=file status=exit status: 0 inside=1 outer=0 reason=1 cancel=false`

`.temp/brief-630/logs/b2-step2b-fe-cli.log` line 29: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.05s`

`.temp/brief-630/logs/b2-step2b-fe-cli.log` line 32: `EXIT:101`

Script and file match the contract. Eval and setup are red: outer count 0, expected 1.

`.temp/brief-630/logs/b2-step2b-fe-jupyter.log` line 11: `assertion `left == right` failed: probe::inside: inside message`

`.temp/brief-630/logs/b2-step2b-fe-jupyter.log` line 13: `  left: 0`

`.temp/brief-630/logs/b2-step2b-fe-jupyter.log` line 14: ` right: 1`

`.temp/brief-630/logs/b2-step2b-fe-jupyter.log` line 23: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 151 filtered out; finished in 0.01s`

`.temp/brief-630/logs/b2-step2b-fe-jupyter.log` line 26: `EXIT:101`

`.temp/brief-630/logs/b2-step2b-fe-repl.log` line 8: `assertion `left == right` failed: rum> ! probe::inside: inside message`

`.temp/brief-630/logs/b2-step2b-fe-repl.log` line 9: `in`

`.temp/brief-630/logs/b2-step2b-fe-repl.log` line 10: `! input cancelled`

`.temp/brief-630/logs/b2-step2b-fe-repl.log` line 12: `  left: 0`

`.temp/brief-630/logs/b2-step2b-fe-repl.log` line 13: ` right: 1`

`.temp/brief-630/logs/b2-step2b-fe-repl.log` line 22: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s`

`.temp/brief-630/logs/b2-step2b-fe-repl.log` line 25: `EXIT:101`

### Step 3

`ReportedOutput` is carried in `PendingEvalRequest` on eval, eval_expr, call, script, and file. Script and file store it with `resumable` false and still refuse `resume_eval_request_with_input` with `no suspended evaluation is waiting for host input`, leaving the stored value installed. Publication is at input pause, completion, failure, and cancellation. The script loop no longer splits each form off or splices it back; the request cuts once at its admission mark. `$MessageList` stays the session view. Memory for the reported lists is charged to `PendingEvalRequest`. Reported trace expressions are gensym roots.

`.temp/brief-630/logs/b2-step3-fmt-1.log` line 1: `EXIT:0`

`.temp/brief-630/logs/b2-step3-check-2.log` line 22: `warning: rum-runtime (lib test) generated 3 warnings`

`.temp/brief-630/logs/b2-step3-check-2.log` line 23: `Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.07s`

`.temp/brief-630/logs/b2-step3-check-2.log` line 24: `EXIT:0`

### Stop

`cargo test -p rum-runtime --lib` moved one expected value, in seven tests, all at the same assertion. Expected `pending_eval_request` is `None`. Actual is `Some`. The assertion was not edited.

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1254: `batch has no resume checkpoint`

That line is the panic from `flat_public_file_cancellation_control` at `crates/rum-runtime/src/runtime/tests/session_requests/batch_boundary.rs:61`. The same message is at lines 1259, 1264, 1269, 1274, 1279, and 1284 for the other six tests.

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1288: `runtime::tests::session_requests::batch_boundary::flat_public_file_cancellation_control`

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1289: `runtime::tests::session_requests::batch_boundary::public_file_cancel_nested`

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1290: `runtime::tests::session_requests::batch_boundary::public_file_cancel_two_levels`

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1291: `runtime::tests::session_requests::batch_boundary::public_file_cleanup_observes_inner_then_outer_state_once`

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1292: `runtime::tests::session_requests::batch_boundary::public_file_first_input_answered_then_cancel_deeper_second`

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1293: `runtime::tests::session_requests::batch_boundary::public_file_rollback_is_deferred_while_blocked`

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1294: `runtime::tests::session_requests::batch_boundary::public_file_snapshot_roots_survive_suspension`

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1296: `test result: FAILED. 1209 passed; 7 failed; 5 ignored; 0 measured; 0 filtered out; finished in 40.13s`

`.temp/brief-630/logs/b2-step3-runtime-lib.log` line 1299: `EXIT:101`

1209 passed. The 7 failures are the whole of the movement: 1216 minus 7. No other rum-runtime lib expected value moved. Step 4 would not put this field back to `None`; step 3 requires the file route to store the admission and the reported prefix.

## Director ruling on the B2 step-3 stop (2026-10-28 17:40) — the pending request becomes a typed enum; continue step 3, then step 4 — grk

**The stop, settled.** You were right to stop again, and again the cause is my
brief. At HEAD, a batch is non-resumable because it has no `pending_eval_request`.
The seven `batch_boundary` tests assert exactly that, and
`resume_eval_request_with_input` refuses a batch because it finds nothing to
take. My shape put the batch's admission and reported output into that same
field, and I did not first ask what the field's absence meant.

`resumable: bool` keeps the behaviour, but it is the wrong shape for two
reasons. A structural fact becomes a value checked at one site. And the resume
path passes a value that is always `true` into
`resume_eval_request_from_checkpoint` and `store_pending_report`. Replace the
bool with a type:

```rust
/// The request a pending suspension belongs to: its admission mark and what
/// the host has already been shown.
pub(in crate::runtime) struct PendingReport {
    pub(in crate::runtime) mark: OutputMark,
    pub(in crate::runtime) reported: ReportedOutput,
}

/// What a request left waiting when it returned `InputRequired`.
pub(in crate::runtime) enum PendingRequest {
    /// `eval`, `eval_expr` and `call`. Only this variant can be resumed.
    Resumable(PendingReport),
    /// `eval_script` and `eval_file`, which are not resumable, as documented.
    /// Retirement still publishes the reported prefix.
    Batch(PendingReport),
}
```

- `Session.pending_eval_request` becomes `pending_request: Option<PendingRequest>`.
  The memory owner, its schema decision row and the field inventory follow the
  rename.
- `resume_eval_request_with_input` takes only `Resumable`. A `Batch` stays
  installed, and the refusal is unchanged. The resume path takes a
  `PendingReport`, and it stores `Resumable` again when the resumed request
  suspends again. No function takes a `resumable` parameter.
- Retirement reads either variant.
- Three sites check that no request is pending, of either kind, and they keep
  that meaning:
  - the `.is_none()` guards in `association_lvalue.rs:226` and
    `incremental_mutation.rs:15`;
  - the terminal assertion in `session_requests.rs`.

**Expected values that change, sanctioned.**
- The assertion at `batch_boundary.rs:61-64`, shared by the seven tests, becomes
  `matches!(s.pending_request, Some(PendingRequest::Batch(_)))`. Keep its
  message, "batch has no resume checkpoint": the type now says what the
  absence used to say.
- `batch_boundary/restore.rs:56`, where an `eval` request is suspended, becomes
  `matches!(s.pending_request, Some(PendingRequest::Resumable(_)))`.

No other expected value may move.

**One question for the report.** `take_pending_report` still has an arm for a
suspension with no stored request, which starts a fresh admission at the current
clock. After step 3, every public route stores a request. Name the path that
still reaches that arm, and the test that reaches it. If nothing reaches it, say
so, and I will rule on it at the gate.

Then finish step 3, do step 4, and run the run list from the 16:45 ruling. Then
STOP. Append "Executing-agent report — B2, continued (2)" under this ruling.
