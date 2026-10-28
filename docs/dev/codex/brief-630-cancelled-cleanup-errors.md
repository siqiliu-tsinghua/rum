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
