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
