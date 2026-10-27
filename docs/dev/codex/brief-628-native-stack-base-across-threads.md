# brief-628 — The native stack guard measures against another thread's stack

**Shape: design-only. Scope: NONE — no source change in the repository.**
You survey, reproduce in a scratch crate, and plan. You also draft the
follow-up implementation briefs.

## Goal

When a `Session` moves to another thread, its native stack guard can
measure against the stack of the thread that started the request. The
result is either a false `StackLimitExceeded` or a guard that cannot fire.
When you are done, these exist, each backed by `path:line` evidence and
saved logs:

1. An inventory of every point where a `Session` starts running guest code,
   or a guarded native traversal, on the current thread. For each point,
   state whether the stack base is established for this thread first.
2. A census of the other `Session` state that belongs to the executing
   thread rather than to the session.
3. A recommended fix shape, with the alternatives weighed.
4. A calibrated falsifier for the fail-open direction.
5. Drafted implementation briefs.

## What the director found (brief-627's review)

The `rum-jupyter` interrupt "flake" from brief-627 is this defect.

- `Session::evaluate` records `native_stack_base` only at
  `evaluation_depth == 0` (`crates/rum-runtime/src/runtime/resolution.rs:318-323`).
- `resume_suspended_slot` records a base for the resuming thread and
  restores the old one afterwards
  (`crates/rum-runtime/src/runtime/suspension/core.rs:634-688`).
- `restore_resume_context` restores `evaluation_depth` and
  `native_stack_next_check_depth` from the queue cell's insertion context,
  but not the base
  (`crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs:32-49`).
- Abandonment does not establish a base.
  - `abandon_pending_suspension` and `drain_abandonment_worklist` are at
    `crates/rum-runtime/src/runtime/suspension/core/abandon.rs:5-60`.
  - `abandon_frames` calls `restore_resume_context(cell.context)` and then
    `self.evaluate(cleanup.expr)` (`abandon.rs:79`, `:96`).
  - The restored depth is nonzero, 2 for a `WithCleanup` inside a
    `CompoundExpression`, so `evaluate` does not rebase.
- The guard computes `consumed = native_stack_base.saturating_sub(current)`
  (`crates/rum-runtime/src/runtime/eval_core/mod.rs:105`, `:119`).
  `run_with_native_stack_guard` reads the same field
  (`crates/rum-runtime/src/runtime/native_stack_guard.rs:98`), so every
  guarded stdlib traversal inherits the base too.
- Abandonment keeps the cleanup's outcome only when it is `InputRequired`
  (`abandon.rs:96-110`). Every other error is dropped without a message.
- The guide's contract: "EOF at an input prompt cancels the waiting
  evaluation, runs guest cleanup once" (`docs/guide/getting-started.md:76`),
  and the same for an interrupt (`:169-173`).

**Both directions were observed.** Probe lines at `abandon.rs:96`, from the
Jupyter pair at `--test-threads=2`:

- **False positive:** `depth=2 base=0x7f848126d7bf here=0x7f848106caff`. That is
  2 MiB against a 1 MiB limit, so the cleanup returns
  `Err(StackLimitExceeded { limit: 1048576 })`. It is dropped, and
  `Print["cleanup"]` never runs.
- **Fail-open:** `depth=2 base=0x7f848106c7bf here=0x7f848126daff`. The stack
  now lies above the base, so `consumed` is 0 and the guard cannot fire for
  that cleanup. That run printed its cleanup and passed.

Pinned to one CPU the pair passed 20 of 20. On the full host it failed 17
of 20: two workers had swapped glibc's cached thread stacks.

**The deterministic reproduction, calibrated by the director:**
`.temp/director/628/stale_stack_base_probe.rs`, which uses the public
`rum-host` API.

- Thread A evaluates
  `kept=42; cleanupCount=0; tail=0; WithCleanup[InputString["first: "]; InputString["second: "], cleanupCount=cleanupCount+1; Print["cleanup"]]; tail=1; Print["tail"]`
  until it is `InputRequired`.
- The session moves to thread B, which calls
  `cancel_handle().cancel(); eval("")`.
- Results, 3 runs each:

  | Case | Result |
  |---|---|
  | A alive while B cancels | `Success prints=[] termination=Some(Cancelled)`, 3/3. **Red.** |
  | A exited before B (B reuses A's stack) | `prints=["cleanup"]`, 3/3 |
  | Same thread | `prints=["cleanup"]`, 3/3 |

  In the red case, B's stack lay about 2.03 MiB below A's.
- **The cleanup must cross a check stride.** With a cleanup of only
  `Print["cleanup"]`, the A-alive case passed: the guard checks only when
  the depth reaches `native_stack_next_check_depth`, which is the restored
  depth plus `NATIVE_STACK_DEPTH_POLL_STRIDE`. Any falsifier you build must
  be shown to reach a check.

## Questions the plan must answer

1. **Entry inventory.** List every place that begins guest evaluation, or a
   guarded native traversal, on the current thread.
   - Public entries include `HostSession` eval, resume, cancel, reset and
     drop, and `Session` drop.
   - Internal re-entries include the abandonment worklist, the resume
     driver (`suspension/core/driver.rs:36`), finalizers, and anything that
     calls `restore_resume_context` or evaluates at a restored depth.
   - For each, give `path:line`, whether the current thread's base is set
     before evaluation, and the fix's effect on it.
2. **Thread-bound Session state.** Find every other `Session` field, or
   state a `Session` points into, that is a property of the executing thread:
   - stack addresses and values derived from them;
   - captured `ThreadId`s;
   - pointers into thread-local storage, and thread-local caches whose
     validity the session assumes.
   For each, give `path:line` and what a move across threads does to it.
   Check the profiler's per-thread metadata in particular.
3. **Fix shape.** Weigh at least these:
   - (i) re-establish the base at each entry;
   - (ii) make the base a property of the thread: a per-thread record of the
     outermost rum entry, which nested entries keep;
   - (iii) bind the base to a `ThreadId` and rebase, or fail closed, on a
     mismatch at the check.

   Judge each on:
   - **Correctness.** `consumed` must count all the stack used since rum's
     outermost entry on this thread, including nested sessions on one
     thread, such as a host callback that evaluates in a second session.
   - **Enforcement.** A new entry point must not be able to forget. The
     compiler, or a test that fails, beats a convention.
   - **Hot-path cost.** `Session::evaluate` is the hottest function, so
     there must be no new work at depth above zero. The check itself is
     strided.

   Recommend one.
4. **The dropped cleanup error.** What should happen when guest cleanup
   fails during abandonment with `StackLimitExceeded`, or with any other
   error? Today it is dropped silently. State what the contract and the
   guide say. Then give the options, for example a message on the retired
   request's outcome, or a fault. The director decides.
5. **The fail-open falsifier.**
   - Build a case where the current stack lies above the stale base. One
     way: spawn B first and keep it waiting, then let A suspend and hand
     the session to B.
   - The cleanup must legitimately exceed the native limit on B: on the
     same thread with a correct base, the guard must fire. At HEAD with the
     stale base, the guard must fail to fire. Show both.
   - Run B with a large explicit stack (`std::thread::Builder::stack_size`),
     so that the overrun is measured rather than fatal. A probe that aborts
     the process is not a falsifier.
   - Calibrate it red at HEAD. Show the same-thread control fires.
6. **Who is affected.** For each front end (`rum-cli`, `rum-repl`,
   `rum-jupyter`, and any `HostSession` embedder), say whether a request can
   be suspended on one thread and retired, resumed or dropped on another.
   Give the evidence.
7. **The falsifier matrix** for the implementation:
   - the director's false-positive probe;
   - your fail-open probe;
   - the same-thread and stack-reuse controls;
   - the Jupyter pair at `--test-threads=2`, 17 of 20 failing at HEAD.

## How to run probes without touching the repository

Build probes in a scratch crate under `.temp/brief-628/probe/`:

- Its own `Cargo.toml` has an empty `[workspace]` table, so cargo does not
  attach it to the repository workspace. Depend by path, for example
  `rum-host = { path = "../../../crates/rum-host" }`.
- Use its own `CARGO_TARGET_DIR` under `.temp/brief-628/`.
- Copy `.temp/director/628/stale_stack_base_probe.rs` there, reproduce the
  director's table first, and only then build on it.
- If reproducing the table fails, STOP: your environment differs.

## Scope

- MAY modify: **NONE** in the repository. Write only under `.temp/brief-628/`,
  and append your report to this brief.
- MUST NOT: touch `crates/`, `scripts/`, `docs/` (apart from your report),
  `Cargo.lock` or `.cargo/`; add a dependency; or leave a probe anywhere
  outside `.temp/brief-628/`.

## Concurrency plan

- Builds: one cargo command at a time, `CARGO_BUILD_JOBS=8`. You run inside
  a 16G memory scope (`TEMPLATE.md` pitfall 9).
- Probes: serial. Some depend on thread-stack placement, so nothing else
  runs during them.
- Give every command a bound with `scripts/run-bounded.py`.

## Stop conditions

- The director's table does not reproduce: STOP.
- A fix would need new `unsafe`: STOP and report it.
- A probe would need a source change in `crates/` to observe something:
  STOP and say what it is.

## Deliverables

- `.temp/brief-628/REPORT.md`, `PROGRESS.md`, and the probe crate with its
  logs.
- `.temp/brief-628/drafts/`: one implementation brief per step, in
  `TEMPLATE.md` shape. Each gives the target shape as code, the must-not-change
  list beside every cost bar, and its falsifier, red at HEAD and green
  after, from the matrix above.
- The report, appended below.

## Report

Append "Executing-agent report" with:

1. the reproduction of the director's table, verbatim;
2. the entry inventory and the thread-bound state census, as tables with
   `path:line`;
3. the fix shapes weighed, and the recommendation;
4. the options for the dropped cleanup error;
5. the fail-open falsifier with its calibration, verbatim;
6. the affected front ends;
7. the list of drafts, every attempt with its result, and the open
   questions.

Quote summary lines only, and link the logs.

## Executing-agent report

Design-only (`Scope: NONE`). No repository source, test, script, `Cargo.lock`, `.cargo/`, or `TODO.md` was modified. Scratch probes, logs, and follow-up drafts are under `.temp/brief-628/`.

### 1. Reproduction of the director's table

Scratch workspace: `.temp/brief-628/probe/Cargo.toml` (`CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-628/target`).
Log: `.temp/brief-628/logs/director_table_repro.log`.

| case | result (3/3 runs) |
|---|---|
| `dir628_same_thread_control` | `Success prints=["cleanup"] termination=Some(Cancelled)` |
| `dir628_moved_a_exited` | `Success prints=["cleanup"] termination=Some(Cancelled)` |
| `dir628_moved_a_alive` | `Success prints=[] termination=Some(Cancelled)` (**Red**) |

Verbatim summary lines from `.temp/brief-628/logs/director_table_repro.log`:

```text
=== RUN 1 ===
DIR628 A stack 0x7fb533da03cf
DIR628 B stack 0x7fb533b9871f
DIR628 moved, A alive: Success prints=[] termination=Some(Cancelled)
DIR628 A stack 0x7fb533b983cf
DIR628 B stack 0x7fb533b9871f
DIR628 moved, A exited: Success prints=["cleanup"] termination=Some(Cancelled)
DIR628 same-thread: Success prints=["cleanup"] termination=Some(Cancelled)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
=== RUN 2 ===
DIR628 A stack 0x7f75f5be73cf
DIR628 B stack 0x7f75f59df71f
DIR628 moved, A alive: Success prints=[] termination=Some(Cancelled)
DIR628 A stack 0x7f75f59df3cf
DIR628 B stack 0x7f75f59df71f
DIR628 moved, A exited: Success prints=["cleanup"] termination=Some(Cancelled)
DIR628 same-thread: Success prints=["cleanup"] termination=Some(Cancelled)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
=== RUN 3 ===
DIR628 A stack 0x7f457054f3cf
DIR628 B stack 0x7f457034771f
DIR628 moved, A alive: Success prints=[] termination=Some(Cancelled)
DIR628 A stack 0x7f45703473cf
DIR628 B stack 0x7f457034771f
DIR628 moved, A exited: Success prints=["cleanup"] termination=Some(Cancelled)
DIR628 same-thread: Success prints=["cleanup"] termination=Some(Cancelled)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

### 2. Entry inventory and thread-bound state census

#### 2.1 Entry inventory (guest evaluation and guarded native traversal)

| entry / re-entry | `path:line` | sets current thread's `native_stack_base` & `native_stack_next_check_depth` before evaluation today? | effect of recommended fix (Shape ii) |
|---|---|---|---|
| `HostSession::eval` / `eval_with_timeout` -> `eval_request_unprotected` -> `Session::eval_request` | `crates/rum-host/src/session.rs:345-355, 435-440, 542-565`; `crates/rum-runtime/src/runtime/session/session_requests.rs:151-173` | **No** for pre-eval retirement (`finish_cancelled_pending_evaluation_request` at `:155` runs `abandon_pending_suspension` with stale base/threshold); **Yes** for new request body via `evaluate` at `evaluation_depth == 0` (`:227`), but never clears `native_stack_base` on exit | `drain_abandonment_worklist` and `evaluate_at_root` both enter `ThreadNativeStackScope`, set `native_stack_base` to the thread's outermost rum entry, reset `native_stack_next_check_depth`, and restore both on exit |
| `HostSession::call` -> `call_unprotected` -> `Session::eval_expr_request` | `crates/rum-host/src/session.rs:367-375, 442-486`; `crates/rum-runtime/src/runtime/session/session_requests.rs:179-195` | **No** for pre-call retirement (`finish_cancelled_pending_evaluation_request` at `session.rs:443` and `session_requests.rs:183`); **Yes** for body at `evaluation_depth == 0`, not cleared on exit | Covered by `drain_abandonment_worklist` and `evaluate_at_root` scopes |
| `HostSession::eval_script` -> `Session::eval_script_request` -> `Session::eval_script` | `crates/rum-host/src/session.rs:394-406`; `crates/rum-runtime/src/runtime/session/session_requests.rs:379-400`; `session_eval.rs:296-394` | **No** for pre-script retirement (`:383`); **Yes** per form at `evaluation_depth == 0` (`session_eval.rs:242`), not cleared on exit | Covered by `drain_abandonment_worklist` and `evaluate_at_root` scopes |
| `HostSession::eval_file` -> `Session::eval_file_request` -> `Session::eval_file` | `crates/rum-host/src/session.rs:421-433`; `crates/rum-runtime/src/runtime/session/session_requests.rs:412-443`; `session_eval.rs:399-422` | **No** for pre-file retirement (`:416`); **Yes** per form at `evaluation_depth == 0`, not cleared on exit | Covered by `drain_abandonment_worklist` and `evaluate_at_root` scopes |
| `HostSession::resume_with_input` -> `Session::resume_eval_request_with_input` | `crates/rum-host/src/session.rs:586-598`; `crates/rum-runtime/src/runtime/session/session_requests.rs:269-377` | **No** in two places: (1) pre-resume cancellation at `:274` calls `finish_cancelled_pending_evaluation_request` before `resume_suspended_evaluation`; (2) post-resume cancellation at `:317-318` calls `finish_cancelled_pending_evaluation_request` *after* `resume_suspended_slot` (`suspension/core.rs:688`) has restored `saved_native_base` (the previous thread's base!). During `resume_suspended_slot` itself (`:637`), overwrites `native_stack_base` with inner frame address even if nested | Both cancellation paths enter `drain_abandonment_worklist`'s `ThreadNativeStackScope`; `resume_suspended_slot` enters `ThreadNativeStackScope` (keeping outermost base if nested) and `saved_native_base` is `0` on an idle/suspended session |
| `HostSession::restore_from_bytes` -> `Session::restore_snapshot_request` | `crates/rum-host/src/session.rs:219-239`; `crates/rum-runtime/src/runtime/session/session_api.rs:93-111` | `eval_script_request` sets base at depth 0 on the restoring thread; `abandon_pending_suspension(FailedSnapshotRestore)` (`:102`) inherits that same-thread base but does not reset `native_stack_next_check_depth` on `restore_resume_context` | `drain_abandonment_worklist` enters `ThreadNativeStackScope` and `restore_resume_context` resets `native_stack_next_check_depth` |
| `HostSession::reset`, `HostSession` `Drop`, `Session::drop` | `crates/rum-host/src/session.rs:176-180`; `crates/rum-runtime/src/runtime/session/session_state.rs:699-703` | N/A — drops `SuspendedEvaluation` frames structurally without calling `FrameAbandon::abandon` or `Session::evaluate` (`Session::drop` only calls `print_lexical_profile_summary_once`) | Unchanged (no guest evaluation on reset/drop) |
| `Session::evaluate` | `crates/rum-runtime/src/runtime/resolution.rs:318-332` | **Only when `self.evaluation_depth == 0`** (`:319-323`), and **never clears** `native_stack_base` on return; also overwrites base if a second `Session` is entered at depth 0 inside a host callback on the same thread | Split into `evaluate_at_root` (`#[cold] #[inline(never)]`, enters `ThreadNativeStackScope` and restores `native_stack_base` / `native_stack_next_check_depth` on exit) and `evaluate_non_root` (`evaluation_depth > 0`, zero new locals or instructions) |
| `Session::abandon_pending_suspension` -> `drain_abandonment_worklist` -> `abandon_frames` | `crates/rum-runtime/src/runtime/suspension/core/abandon.rs:5-115` | **No.** Never sets `native_stack_base` or `native_stack_next_check_depth`. At `:79`, `restore_resume_context(cell.context)` sets `evaluation_depth = cell.context.depth` (typically `> 0`), so `:97` `self.evaluate(cleanup.expr)` skips the depth-0 rebase and uses stale `native_stack_base` and stale `native_stack_next_check_depth` | `drain_abandonment_worklist` enters `ThreadNativeStackScope`, sets `native_stack_base = thread_base`, saves/restores displaced fields, and `restore_resume_context` resets `native_stack_next_check_depth = context.depth + STRIDE` |
| `Session::resume_suspended_slot` -> `drive_resuming_frames` -> `drive_frames` | `crates/rum-runtime/src/runtime/suspension/core.rs:630-714`; `suspension/core/driver.rs:20-185` | Sets `self.native_stack_base` at `core.rs:637` (unconditionally overwriting any outer base on the same thread) and `drive_frames` resets `native_stack_next_check_depth` via `poll_resumed_native_stack` (`driver.rs:63, 97, 108, 146`). Restores `saved_native_base` (which was stale from previous `evaluate`) at `core.rs:688` | `resume_suspended_slot` enters `ThreadNativeStackScope` (preserving outer thread base if nested), and `restore_resume_context` (`evaluation_context.rs:38`) resets `native_stack_next_check_depth` directly |
| `Session::restore_resume_context` | `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs:32-41` | **No.** Sets `self.evaluation_depth = context.depth` (`:38`) without updating `self.native_stack_next_check_depth` | Resets `self.native_stack_next_check_depth = context.depth.saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE)` |
| `GuestCallerStack::restore` (`GuestFormScope::restore` on resumed/abandoned `Get` script forms) | `crates/rum-runtime/src/runtime/session/guest_loading/scope.rs:26-29, 182-195`; `script.rs:43-45, 50-52` | **No.** Sets `session.evaluation_depth = self.depth` (`scope.rs:28`) without resetting `session.native_stack_next_check_depth`, so subsequent forms in `drive_guest_script` (`script.rs:45`) inherit the prior form's high-water check depth | Resets `session.native_stack_next_check_depth = self.depth.saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE)` |
| `Session::restore_evaluation_rollback_snapshot` (`ValueQ` trial evaluation rollback) | `crates/rum-runtime/src/runtime/session/session_helpers.rs:239-280` | Calls `abandon_pending_suspension(CheckpointRollbackDiscard)` (`:249`) and restores `self.evaluation_depth = snapshot.evaluation_depth` (`:278`) without resetting `native_stack_next_check_depth` | Resets `self.native_stack_next_check_depth = snapshot.evaluation_depth.saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE)` |
| `PureFrameFinalizer::finish` / `SlotFinalizer::finish` | `crates/rum-runtime/src/runtime/suspension/core/driver.rs:101, 150` | Runs `Session`-free finalizer closure inside `resume_suspended_slot` | Covered by `resume_suspended_slot`'s `ThreadNativeStackScope` |
| `run_with_native_stack_guard` (`eval_scan`, `eval_merge`, `finish_association_set_function_lvalue`, `finish_association_set_part_lvalue`) | `crates/rum-runtime/src/runtime/native_stack_guard.rs:88-112`; `functional_ops/apply_map/mod.rs:129`; `associations/resumable/aggregate.rs:20`; `association_data.rs:908, 952` | Copies `base: session.native_stack_base` (`:98`). During abandonment cleanup at `evaluation_depth > 0`, inherits stale `session.native_stack_base` | Reads `current_thread_native_stack_base()` when active (asserting equality with `session.native_stack_base`), falling back to `session.native_stack_base` only for standalone unit tests |
| `cached_call_stability_with_bloom` & `begin_association_call_cache_lookup` | `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:321-327`; `stability/association_front.rs:149-155` | Strided call to `self.native_stack_budget_exhausted()` (`eval_core/mod.rs:116-121`), which reads `self.native_stack_base` (even at `evaluation_passes == 1` in `evaluate_profiled_fixed_point` before `evaluate_once_in_accounting_mode`!) | `native_stack_budget_exhausted` asserts agreement with `current_thread_native_stack_base()` and reads the active thread's outermost stack base |

#### 2.2 Thread-bound `Session` state census

| state / field | `path:line` | what it stores | effect of moving `Session` across threads |
|---|---|---|---|
| `Session::native_stack_base: usize` | `crates/rum-runtime/src/runtime/model/session.rs:89`; set at `resolution.rs:321`, `suspension/core.rs:637, 688`; read at `eval_core/mod.rs:105, 119`, `native_stack_guard.rs:98` | Raw stack address (`(&stack_marker as *const u8) as usize`) of the thread that last ran `evaluate` at depth 0 (never cleared on exit) | On another thread B, `native_stack_base.saturating_sub(current_addr)` measures the virtual-address gap between A's stack and B's stack: **false-positive `StackLimitExceeded`** when B's stack is `> limit` below A's; **fail-open (`0`)** when B's stack is above A's |
| `Session::native_stack_next_check_depth: usize` | `crates/rum-runtime/src/runtime/model/session.rs:90`; set at `resolution.rs:322`, `eval_core/mod.rs:55-56`, `evaluation_context.rs:47`, `suspension/core.rs:635, 689` | Next `evaluation_depth` threshold for strided `poll_native_stack_limit` | Not reset by `restore_resume_context` (`evaluation_context.rs:38`), `GuestCallerStack::restore` (`guest_loading/scope.rs:28`), or `restore_evaluation_rollback_snapshot` (`session_helpers.rs:278`). After a deep suspension (e.g. depth 83 -> check depth 88), restoring an outer frame (e.g. depth 2) leaves check depth at 88 so cleanup below depth 88 **never polls** (even on the same thread) |
| `Session::stability_native_stack_poll_counter: u64` | `crates/rum-runtime/src/runtime/model/session.rs:91`; used at `evaluation_cache/stability.rs:321-324`, `association_front.rs:149-152` | Stride counter for calling `self.native_stack_budget_exhausted()` on stability cache misses | Safe once `self.native_stack_base` is bound to the current thread's outermost entry; at HEAD it triggers `native_stack_budget_exhausted()` against the stale `native_stack_base` before `evaluate_once_in_accounting_mode` even runs |
| Logical profiler (`LP02`) `ProfileQueue` -> `Context` -> `RecordPath::state` -> `ActiveId { thread: ThreadId, index: usize, segment: SegmentId }` and TLS `ACTIVE: RefCell<Stack>` | `crates/rum-runtime/src/runtime/profiling/logical/totals.rs:30-34, 41`; `logical/segments.rs:46-56, 133-138, 234-256`; `logical/context.rs:46-111` | `SuspendedEvaluation::frames` (`FrameStorage::Profiled(Box<ProfileQueue>)`) retains `Context` and `ProfileBoundary` handles into `RecordPath`. `ActiveId` records the `ThreadId` and TLS `ACTIVE` frame index of an active segment | Normal pause (`Call::pause` / `ContextGuard::drop`) ends the active segment on thread A (`active = None`), so resuming or abandoning on thread B cleanly installs a new segment on thread B's TLS (`segments.rs:133-138`). If the same `RecordPath` is simultaneously active on thread A when thread B restores or finishes it (`id.thread != current().id()`), `install` (`:135`) and `finish_active` (`:247`) call `epoch.fault()` (except `finish_active` at `:235` returns early without faulting if thread B's `active_len() == 0`) |
| Logical/legacy profiler TLS source stacks (`EVALUATION_PROFILE_SOURCE_STACK`, `EXPRESSION_REBUILD_SOURCE_STACK`, `LEXICAL_CONTEXT_STACK`, `ARG_PREFILTER_SITE_STACK`) and timing stacks (`*_TIMING_STACK`) | `crates/rum-runtime/src/runtime/profiling/state.rs:82-139`; `evaluation_cache/profile/source.rs:21-28`; `logical/context.rs:37-40, 70-79` | Thread-local diagnostic stacks | Timing and source guards are RAII-scoped to native frames and unwind on `InputRequired`. When LP02 is enabled, `logical::Context` captures the four source stacks on pause and swaps them into the resuming/abandoning thread's TLS inside `Context::enter` (`logical/context.rs:70-79`) |
| Legacy profiler TLS stat maps (`PRESENCE_*_STATS`, `BUILTIN_TIMING_STATS`, `RUNTIME_PATH_TIMING_STATS`, `CALL_STABILITY_SOURCE_HEAD_STATS`) read in `Session::drop` | `crates/rum-runtime/src/runtime/session/session_state.rs:699-703`; `profiling/summary.rs:3-20`; `profiling/state.rs:82-139` | Per-thread `RefCell<HashMap<...>>` accumulators; `Session::drop` calls `print_lexical_profile_summary_once(self)` | `print_lexical_profile_summary_once` reads the TLS maps of the thread that drops `Session`; if `Session` evaluated on worker threads and is dropped on a different thread, worker-thread TLS stats are not in the dropping thread's TLS |
| Lowering depth TLS (`LOWERING_FRONT_DEPTH`, `LOWERING_TAIL_DEPTH`, `LOWERING_USED_TAIL`) & stability miss outcome TLS (`ASSOCIATION_CALL_STABILITY_MISS_OUTCOMES`) | `crates/rum-runtime/src/runtime/lowering.rs:15-23`; `evaluation_cache/stability/association_front.rs:8-11, 157, 169` | Synchronous recursion depth / scratch stack | Strictly scoped to synchronous non-suspending calls (`lower_syntax_expr`, `association_stability_tail_with_bloom`); never live across `InputRequired` or thread moves |

### 3. Fix shapes weighed and recommendation

| shape | correctness (including nested sessions on one thread) | enforcement | hot-path cost (`Session::evaluate` at `evaluation_depth > 0`) | verdict |
|---|---|---|---|---|
| **(i) Re-establish `session.native_stack_base` at each entry on `Session`** | **Fails nested-session requirement:** if Session 1 on thread T consumes 650 KiB and a `PrintSink` / `FileIo` / `EntropySource` callback evaluates in Session 2 on thread T, Session 2 sees `session2.evaluation_depth == 0` and rebases to `base_1 - 650 KiB`, allowing 1.3 MiB+ on a 2 MiB stack (`brief628_fail_open_nested_session_on_same_thread_falsifier`). Also fails if `resume_suspended_slot` or `drain_abandonment_worklist` rebases unconditionally while nested inside an outer evaluation on the same `Session` | Convention-only unless paired with a per-thread check | Zero at `evaluation_depth > 0` | **Reject** |
| **(ii) Per-thread record of the outermost `rum` entry (`THREAD_NATIVE_STACK_BASE` + RAII `ThreadNativeStackScope`), which nested entries keep, plus resetting `native_stack_next_check_depth` on context restores** | **Full correctness:** outermost entry on thread T sets `THREAD_NATIVE_STACK_BASE = marker_addr` and `session.native_stack_base = marker_addr`; any nested entry on thread T (re-entrant `drain_abandonment_worklist`, `resume_suspended_slot`, or a second `Session` inside a host callback) sees `existing != 0` and keeps `existing` as `thread_base`. On exit (normal return or panic unwind), the outermost guard resets `THREAD_NATIVE_STACK_BASE` to `0` and restores `session.native_stack_base` to `0`, so an idle/suspended `Session` never carries a stack pointer across threads | **Compiler + invariant-enforced:** `Session` takes `&mut self`, so borrow-checking prevents moving `Session` while a scope guard is live; `package_eval_request_outcome` asserts `self.native_stack_base == 0` at every request boundary; `poll_native_stack_limit` and `native_stack_budget_exhausted` assert `self.native_stack_base == current_thread_native_stack_base()` | **Zero new work at `evaluation_depth > 0`:** `Session::evaluate` keeps the single `if self.evaluation_depth == 0 { return self.evaluate_at_root(expr); }` branch; `evaluate_at_root` is `#[cold] #[inline(never)]`, so `Session::evaluate`'s recursive frame has no `ThreadNativeStackScope` local and no drop branch | **Recommend** |
| **(iii) Bind `native_stack_base` to `ThreadId` and rebase or fail closed at the check** | **Fails in both sub-variants:** failing closed breaks legitimate cross-thread session handoff (`rum-jupyter` per-request worker threads, `docs/host-embedding-and-hardening.md:103-105`); rebasing at the check misses stack consumed on thread B before the first strided poll, misses `run_with_native_stack_guard` which copies `base` before polling, misses same-thread stale `native_stack_next_check_depth`, and fails the nested-session requirement (same `ThreadId`) | Does not enforce `native_stack_next_check_depth` resets on context restores | Adds `thread::current().id()` at strided polls and adds a `ThreadId` field to `Session` (requiring `occurrence/schema.rs` updates) | **Reject** |

**Recommendation:** **Shape (ii)** (drafted in `.temp/brief-628/drafts/brief-native-stack-base-thread-scope.md`).

### 4. Options for the dropped cleanup error

#### Why `StackLimitExceeded` was invisible at HEAD

In `rum-runtime`, resource-limit and uncaught control-flow errors (`EvalError::StackLimitExceeded` -> `General::reslim`, `RecursionLimitExceeded` -> `General::reclim`, `IterationLimitExceeded` -> `General::iterlim`, `TimeConstraintExceeded` -> `General::timelim`, `MemoryConstraintExceeded` -> `General::memlim`, `Thrown` -> `Throw::nocatch`, etc.) do **not** push a `RuntimeMessage` into `self.messages` at the `Err(...)` creation site (`poll_native_stack_limit` at `crates/rum-runtime/src/runtime/eval_core/mod.rs:106-110`). Instead, top-level drivers call `Session::emit_non_loader_eval_message` and `Session::emit_top_level_control_flow_messages` (`crates/rum-runtime/src/runtime/session/session_eval.rs:280-281`, `session_requests.rs:508-509`, `session_helpers.rs:392-533`).
In `Session::abandon_frames` (`crates/rum-runtime/src/runtime/suspension/core/abandon.rs:97-107`), `let cleanup_outcome = self.evaluate(cleanup.expr);` only matches `Err(EvalError::InputRequired { .. })` and silently drops every other `Err(_)`, without calling `emit_non_loader_eval_message` or `emit_top_level_control_flow_messages`.

#### What the contract says

- `docs/guide/getting-started.md:76-84`: *"EOF at an input prompt cancels the waiting evaluation, runs guest cleanup once, and exits... Cancellation is reported once; automatic abandonment retains the input-required reason."*
- `docs/guide/getting-started.md:169-182`: *"If evaluation needs input, the CLI reports the original input requirement on stderr, cancels that request, runs its `WithCleanup` cleanup once, and exits 1... Retirement does not add a second cancellation error. A public file (`--file` or a positional file) rolls back on cancellation; nested file loads roll back from inner to outer. Messages produced during cleanup are rolled back with their file... A stdin script has no enclosing file transaction: earlier forms remain, and its own cleanup messages reach stderr once. The same is true of `--eval` outside a nested file transaction."*
- `crates/rum-host/tests/suspension_abandonment_contract.rs:99-130, 591-613`: A cancelled suspended request must return `HostTerminationReason::Cancelled` (`TerminatedEvaluation["Cancelled"]`) and must continue draining outer `WithCleanup` cleanups, scope restorations, and file rollbacks on `abandonment_worklist`.

#### Options for the director's decision

| option | behavior when `self.evaluate(cleanup.expr)` returns `Err(err)` in `abandon_frames` | pros / cons |
|---|---|---|
| **Option A (Recommended; drafted in `brief-abandonment-cleanup-diagnostics.md`)** | Keep the outer request outcome as `Cancelled` and continue draining `abandonment_worklist`, but when `reason == AbandonReason::CancelledRequest` and `err` is neither `InputRequired` nor `Cancelled`, call `self.emit_non_loader_eval_message(handled_error_checkpoint, &err)` and `self.emit_top_level_control_flow_messages(&err)` | Matches `docs/guide/getting-started.md:174, 180-181` ("its own cleanup messages reach stderr once... Retirement does not add a second cancellation error"); automatically rolls back if enclosed in a cancelled `--file` / `Get` transaction (`PublicFileRollbackFrame` / `GuestFileCloseFrame`) or `Quiet` / `EvaluationData`; makes cleanup resource-limit failures (`General::reslim`, `General::reclim`) observable in `outcome.messages` without masking outer `Cancelled` termination or skipping outer cleanups |
| **Option B (Status quo in `abandon.rs`)** | Keep dropping non-`InputRequired` `Err(_)` in `abandon_frames` and test cleanup completion via cleanup side effects (`Print` / symbol state) | Zero change to message lists on malformed cleanups, but a cleanup that hits `StackLimitExceeded` or `RecursionLimitExceeded` produces no `General::reslim` / `General::reclim` diagnostic even when explicit `Message[...]` calls inside cleanup do reach `outcome.messages` |
| **Option C (Promote cleanup resource error to request failure / fault)** | Replace `Cancelled` with the cleanup's `EvalError` or record a fault | **Violates contract:** breaks `docs/guide/getting-started.md:76, 169` and `suspension_abandonment_contract.rs:124` (`termination == Some(Cancelled)`), and if it aborts `abandonment_worklist`, leaks outer `Block`/`Module` scopes and skips outer `WithCleanup` / file-rollback frames |

### 5. Fail-open falsifiers and calibration

Probe source: `.temp/brief-628/probe/src/lib.rs`.
Log (3 consecutive runs): `.temp/brief-628/logs/fail_open_falsifier.log`.

All fail-open probes run on explicit 16 MiB threads (`std::thread::Builder::new().stack_size(16 * 1024 * 1024)`) with the default 1 MiB `native_stack_limit` (`1_048_576` bytes) and `recursion_limit = 4096`, so overruns past 1 MiB are measured in-band without overflowing the OS thread stack. Every probe prints `"past-first-poll"` after crossing `native_stack_next_check_depth` (`>= 8`) and prints `"cleanup-past-stack-limit"` (or `"leaf-reached"`) only if the 1 MiB byte guard fails to fire:

| test in `.temp/brief-628/probe/src/lib.rs` | mechanism | result at HEAD (3/3 runs) | expected after fix |
|---|---|---|---|
| `brief628_fail_open_same_thread_control` | Direct `dive[60]` at depth 0 + same-thread suspended & cancelled `WithCleanup[InputString["prompt"], dive[60]]` on 16 MiB stack | **PASS (Control):** direct `prints=["past-first-poll"] termination=Some(StackLimit)`; cancelled `prints=["past-first-poll"] termination=Some(Cancelled)` | **PASS** |
| `brief628_fail_open_moved_b_above_a_falsifier` | Spawn B first (16 MiB stack) and hold waiting; spawn A second while B is alive so `b_stack > a_stack` (by 16.01–93.85 MiB); suspend `WithCleanup[InputString["prompt"], dive[60]]` on A; cancel & retire on B while A stays alive | **FAIL (Red 3/3):** `prints=["past-first-poll", "cleanup-past-stack-limit"]` (`a_stack.saturating_sub(current_addr) == 0` at every poll on B) | **PASS:** `prints=["past-first-poll"]` |
| `brief628_fail_open_same_thread_stale_check_depth_falsifier` | Same thread (no thread move): suspend 80 nested `u[...]` calls at depth ~83 (< 1 MiB, raising `native_stack_next_check_depth` to `88`) inside `WithCleanup` (at depth 2); cancel on the same thread and run a 60-deep `Module[{x = 1}, ...]` cleanup (> 1 MiB stack, max depth ~66 < 88) | **FAIL (Red 3/3):** direct control stops with `prints=["past-first-poll"] termination=Some(StackLimit)`, but cancelled cleanup prints `["past-first-poll", "cleanup-past-stack-limit"]` because `restore_resume_context` leaves `native_stack_next_check_depth = 88` | **PASS:** `prints=["past-first-poll"]` |
| `brief628_fail_open_nested_session_on_same_thread_falsifier` | Same thread: solo `dive[12]` (~650 KiB) succeeds (`["past-first-poll", "leaf-reached"]`) and solo `dive[24]` (~1.3 MiB) hits `StackLimit`; outer `HostSession` runs `outerDive[15]` (~650 KiB) and its `ClosurePrintSink` evaluates `dive[12]` (~650 KiB) in an inner `HostSession` on the same thread (~1.3 MiB total) | **FAIL (Red 3/3):** inner `HostSession` rebases to callback stack pointer and succeeds with `termination=None, prints=["past-first-poll", "leaf-reached"]` | **PASS:** inner `HostSession` terminates with `Some(StackLimit)` |
| `brief628_false_positive_moved_a_alive_falsifier` | Director's false-positive probe as an assertion test (`b_stack < a_stack` by ~2.03 MiB while A is alive) | **FAIL (Red 3/3):** `prints=[] termination=Some(Cancelled)` | **PASS:** `prints=["cleanup"]` |

Verbatim summary lines from `.temp/brief-628/logs/fail_open_falsifier.log`:

```text
=== RUN 1 ===
test brief628_fail_open_moved_b_above_a_falsifier ... BRIEF628 fail-open moved (B above A by 16.01 MiB): A=0x7f2d06ff8647 B=0x7f2d07ffbe37 -> Success prints=["past-first-poll", "cleanup-past-stack-limit"] termination=Some(Cancelled)
FAILED
test brief628_fail_open_nested_session_on_same_thread_falsifier ... BRIEF628 nested control dive[12]: Success prints=["past-first-poll", "leaf-reached"] termination=None
BRIEF628 nested control dive[24]: Success prints=["past-first-poll"] termination=Some(StackLimit)
BRIEF628 nested outer outerDive[15]: Success prints=[] termination=None
BRIEF628 nested inner dive[12]: Success prints=["past-first-poll", "leaf-reached"] termination=None
FAILED
test brief628_fail_open_same_thread_control ... BRIEF628 fail-open direct control: A=0x7f2d06ff93a7 B=0x7f2d06ffa807 -> Success prints=["past-first-poll"] termination=Some(StackLimit)
BRIEF628 fail-open same-thread control: A=0x7f2d06ff93a7 B=0x7f2d06ffa807 -> Success prints=["past-first-poll"] termination=Some(Cancelled)
ok
test brief628_fail_open_same_thread_stale_check_depth_falsifier ... BRIEF628 stale-check-depth direct control: Success prints=["past-first-poll"] termination=Some(StackLimit)
BRIEF628 stale-check-depth same-thread cancelled: Success prints=["past-first-poll", "cleanup-past-stack-limit"] termination=Some(Cancelled)
FAILED
test brief628_false_positive_moved_a_alive_falsifier ... DIR628 A stack 0x7f2d0c80c3cf
DIR628 B stack 0x7f2d0c60471f
BRIEF628 false-positive falsifier observed: Success prints=[] termination=Some(Cancelled)
FAILED
test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.10s
```

### 6. Affected front ends

| front end / embedder | can a request suspend on thread A and be retired, resumed, or dropped on thread B? | affected by stale `native_stack_base` / `native_stack_next_check_depth`? | `path:line` evidence |
|---|---|---|---|
| **`rum-jupyter`** | **Yes (every segment spawns a fresh OS thread).** `ExecuteState::start_eval` (`worker.rs:86-104`), `start_resume` (`:106-120`), and `start_retirement` (`:123-137`) each call `ExecuteWorker::spawn` -> `thread::spawn` (`:321`). Worker 1 evaluates until `InputRequired` and exits (`join` at `:223-229`); Worker 2 resumes or Worker 3 retires (`ExecuteSegment::CancelInput` at `:370-385`) on a newly spawned thread, and the adapter drops `HostSession` on the main thread | **Yes — both cross-thread and same-thread.** Single-threaded, Worker 3 usually reuses Worker 1's exited stack (`dir628_moved_a_exited`); when two adapters or another thread allocate a stack concurrently (`--test-threads=2`), Worker 3 gets a higher or lower stack, causing either false-positive `StackLimitExceeded` (17/20 failures in `jup-627`) or fail-open | `crates/rum-jupyter/src/adapter/worker.rs:86-137, 223-229, 315-392`; `crates/rum-jupyter/src/tests/execute_lifecycle/interrupt.rs:4-16, 150-168` |
| **`rum-cli`** | **No cross-thread `HostSession` move.** `spawn_eval_timeout_thread` (`main.rs:820-873`) only moves a `HostCancelHandle` to the watchdog thread; `HostSession` stays on the main thread (`retire_unanswered_script_input` at `main.rs:942-949` calls `cancel.cancel(); let _ = session.eval("");` on the main thread) | **Affected by same-thread stale `native_stack_next_check_depth`:** when a script suspends on `InputString` at depth `D` inside an outer `WithCleanup`, `retire_unanswered_script_input` abandons on the same thread with `native_stack_next_check_depth` stuck at `D + 8` | `crates/rum-cli/src/main.rs:820-873, 942-949` |
| **`rum-repl`** | **Not internally, but accepts `&mut HostSession` from embedders.** `Watchdog::start` (`request.rs:133-168`) and `InterruptGuard::arm` (`:195-218`) only move `HostCancelHandle`; `retire_pending_input` (`:248-259`) calls `cancel.cancel(); session.resume_with_input("")` on the caller thread | **Affected by same-thread stale `native_stack_next_check_depth`** on Ctrl-C / EOF at an input prompt after a deep suspension (`request.rs:252`), and by cross-thread `native_stack_base` if a host moves `&mut HostSession` across threads | `crates/rum-repl/src/request.rs:133-168, 195-218, 248-259` |
| **`rum-host` (`HostSession`) embedders** | **Yes (`HostSession: Send` by contract).** `docs/host-embedding-and-hardening.md:103-105` explicitly supports moving a `HostSession` to a worker thread, and `ClosurePrintSink` / `FileIoProvider` / `EntropyProvider` / `BuiltinInstaller` callbacks allow nested `HostSession` evaluation on one thread | **Yes — all four failure modes:** cross-thread false-positive, cross-thread fail-open, same-thread stale `native_stack_next_check_depth`, and same-thread nested-session fail-open | `crates/rum-host/src/session.rs:104-128, 345-598`; `docs/host-embedding-and-hardening.md:61-83, 103-105` |

### 7. Drafts, attempts, and open questions

#### 7.1 Drafted implementation briefs

- `.temp/brief-628/drafts/brief-native-stack-base-thread-scope.md` — Step 1 (mandatory): per-thread outermost `THREAD_NATIVE_STACK_BASE` + `ThreadNativeStackScope`, cold `evaluate_at_root` split in `Session::evaluate`, `resume_suspended_slot` and `drain_abandonment_worklist` scoping, `native_stack_next_check_depth` resets in `restore_resume_context` / `GuestCallerStack::restore` / `restore_evaluation_rollback_snapshot`, boundary assertions, and the 4-falsifier + control suite + `rum-jupyter` `--test-threads=2` verification + microbench A/B gate.
- `.temp/brief-628/drafts/brief-abandonment-cleanup-diagnostics.md` — Step 2 (optional, contingent on director's choice in §4): emit diagnostic messages (`General::reslim`, `General::reclim`, `Throw::nocatch`, etc.) when `WithCleanup` cleanup errors during `AbandonReason::CancelledRequest`, while keeping outer `Cancelled` termination and draining all remaining abandonment frames.

#### 7.2 Every attempt and its result

| # | stage / command | exit | result |
|---:|---|---:|---|
| 1 | Build & run director's 3-test probe (`.temp/brief-628/probe`, 3 runs) | 0 | Reproduced director's table 3/3 (`dir628_moved_a_alive` prints `[]`; `dir628_moved_a_exited` and `dir628_same_thread_control` print `["cleanup"]`). Saved to `.temp/brief-628/logs/director_table_repro.log` |
| 2 | Compile fail-open probe with `HostSession::with_config` | 101 | Compile error (`HostSession::builder().with_config(config).build()` is the builder API); fixed in `.temp/brief-628/probe/src/lib.rs` |
| 3 | Calibrate fail-open depth at `dive[300]` on 8 MiB stack | 101 | `dive[300]` (~900 unoptimized debug evaluator frames) exceeded 8 MiB OS stack; lowered recursion to `dive[60]` on a 16 MiB stack (`FAIL_OPEN_STACK_SIZE = 16 MiB`) |
| 4 | Run `brief628_fail_open_same_thread_control`, `brief628_fail_open_moved_b_above_a_falsifier`, and `brief628_false_positive_moved_a_alive_falsifier` (`dive[60]`) | 101 (expected Red falsifiers) | Control passed (`direct -> Some(StackLimit)`, `same-thread cancel -> prints=["past-first-poll"]`); both falsifiers failed as expected (`moved_b_above_a -> prints=["past-first-poll", "cleanup-past-stack-limit"]`; `false_positive -> prints=[]`) |
| 5 | Calibrate same-thread stale `native_stack_next_check_depth` probe (`light[85]` / `makeU[110]` / `u`x105) | 101 | Calibrated depth-per-byte ratio: 80 nested `u[...]` calls suspend at depth ~83 (< 1 MiB, setting `native_stack_next_check_depth = 88`), and 60 nested `Module[{x = 1}, ...]` calls in cleanup exceed 1 MiB at depth ~66 (< 88) |
| 6 | Run 3-run calibration of `brief628_fail_open_same_thread_stale_check_depth_falsifier` + cross-thread probes | 0 (wrapper) | 3/3 runs reproduced: control passed, all 3 falsifiers failed at HEAD as designed |
| 7 | Add `brief628_fail_open_nested_session_on_same_thread_falsifier` (`ClosurePrintSink::new`) | 101 | Compile error (`ClosurePrintSink::assert_bounded_destruction`); fixed |
| 8 | Calibrate nested-session depth (`dive[20]` -> `dive[12]` + `outerDive[15]`) and run 3-run suite | 0 (wrapper) | `dive[12]` alone (~650 KiB) passes (`["past-first-poll", "leaf-reached"]`), `dive[24]` alone (~1.3 MiB) hits `StackLimit`, and `outerDive[15]` + inner `dive[12]` fails open at HEAD 3/3 (`termination=None`). Saved to `.temp/brief-628/logs/fail_open_falsifier.log` |

#### 7.3 Open questions for the director

1. **Dropped cleanup error policy (§4):** Should `abandon_frames` adopt **Option A** (emit `General::reslim` / `General::reclim` / `Throw::nocatch` into `self.messages` on `AbandonReason::CancelledRequest` while preserving the outer `Cancelled` outcome and draining all remaining frames, as drafted in `brief-abandonment-cleanup-diagnostics.md`) or **Option B** (keep silent discard in `abandon_frames` and ship only `brief-native-stack-base-thread-scope.md`)?
2. **Standalone `run_with_native_stack_guard` outside `evaluate` / `resume` / `abandon`:** In production, all four callers of `run_with_native_stack_guard` (`eval_scan`, `eval_merge`, and the two `Association` lvalue helpers) run inside `evaluate` or `resume_suspended_slot` where `THREAD_NATIVE_STACK_BASE` is non-zero. Two unit tests in `crates/rum-runtime/src/runtime/tests/native_stack_guard.rs` call `run_with_native_stack_guard` directly on a synthetic `Session` with `session.native_stack_base = usize::MAX` or `0`. Our draft preserves those unit tests by falling back to `session.native_stack_base` when `THREAD_NATIVE_STACK_BASE == 0`; if the director prefers those two unit tests to enter a test helper instead so `run_with_native_stack_guard` can unconditionally `debug_assert_ne!(current_thread_native_stack_base(), 0)`, only `native_stack_guard.rs` and its 40-line test file need that adjustment.

## Director review (2026-10-27 22:40)

Accepted. The plan found three failure modes beyond the one I reported, and
calibrated each one. I reran the probe crate here: the four falsifiers were
red 3 of 3 and the four controls green (`.temp/director/628/probe-run-{1,2,3}.log`).
It also corrected my brief, which said `restore_resume_context` resets the
check depth. It does not; the line I read belongs to
`poll_resumed_native_stack`.

Shape (ii) is accepted with three changes in my rewrite (brief-629):

1. **`Session::native_stack_base` is removed, not kept as a checked copy.**
   The thread record is the only base. A copy that debug assertions keep
   equal can still be read stale in a release build, and a field that does
   not exist cannot. The guard reads the thread record at its strided checks.
   A check that finds no entry on its thread is an invariant violation: it
   panics in debug and test builds, and fails closed in release.
2. **The depth and its check threshold become one type.** `EvaluationDepth`
   has private fields. The only ways to move it are the evaluator's own
   level entry and exit, and `restore(depth)`, which derives the threshold.
   The three stale restores you found then cannot be written. This replaces
   resetting the threshold at each site, which a fourth site could forget.
   There are 25 non-test references in 10 files.
3. **The entry guard is `!Send`,** so a held entry cannot leave its thread.

On the dropped cleanup error, I choose Option A. Making a silent failure
loud is this project's direction, and the guide promises the cleanup runs.
It is a separate brief after 629. That brief settles which `AbandonReason`s
emit, from an inventory of the message channel each retirement has. Open
question 2 is settled by change 1: the two unit tests enter a test-only
helper that sets the thread record.
