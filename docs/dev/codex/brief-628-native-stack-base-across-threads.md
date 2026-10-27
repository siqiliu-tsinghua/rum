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
