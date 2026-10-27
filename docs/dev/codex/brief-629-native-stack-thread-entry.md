# brief-629 — The native stack base belongs to the thread; the depth carries its own check threshold

**Shape: implementation, staged.** Stage A is the shape and its proofs, and
ends in a STOP for the director's gate. Stage B, the measurement campaign,
follows only in a continuation of this brief.

## Goal

Every native stack check measures from the outermost rum entry **on the
thread that is running**. That holds wherever the `Session` has been, and
when a second session runs inside a host callback on the same thread. The
check threshold can never be left stale by a depth restore. When you are
done, these hold:

- `Session` has no `native_stack_base` field.
- The depth and its threshold are one type, with private fields.
- The four brief-628 falsifiers are green.
- Nothing on the default path costs more (Stage B decides that).

Read brief-628 first, including its report and my review
(`docs/dev/codex/brief-628-native-stack-base-across-threads.md`). It is the
evidence for everything below.

## Context pointers

- `AGENTS.md`; `docs/dev/codex/TEMPLATE.md`, pitfalls 8–11.
- The defect, in four modes (brief-628 §2, §5):
  - cross-thread false positive: cleanup silently skipped;
  - cross-thread fail-open;
  - same-thread stale threshold fail-open;
  - nested-session fail-open.
- The current code:
  - `crates/rum-runtime/src/runtime/model/session.rs:89-90` holds the two
    fields this brief removes or replaces;
  - `crates/rum-runtime/src/runtime/resolution.rs:318-332` is
    `Session::evaluate`;
  - `crates/rum-runtime/src/runtime/eval_core/mod.rs:34-77` is the
    evaluator's level entry and exit, and `:102-121` the two byte checks;
  - `crates/rum-runtime/src/runtime/native_stack_guard.rs` is
    `run_with_native_stack_guard` and `NativeStackGuard`;
  - `crates/rum-runtime/src/runtime/suspension/core.rs:630-690` is
    `resume_suspended_slot`;
  - `crates/rum-runtime/src/runtime/suspension/core/abandon.rs:25-115` is
    `drain_abandonment_worklist` and `abandon_frames`;
  - `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs:32-50`
    is `restore_resume_context` and `poll_resumed_native_stack`;
  - `crates/rum-runtime/src/runtime/session/guest_loading/scope.rs:26-29` is
    `GuestCallerStack::restore`;
  - `crates/rum-runtime/src/runtime/session/session_helpers.rs:278` is the
    rollback restore;
  - `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs:103,145-146`
    has the schema rows.
- Every write to `evaluation_depth` outside tests today (the director's grep):
  `eval_core/mod.rs:74,77`; `suspension/core.rs:687`;
  `evaluation_context.rs:38`; `abandon.rs:33,52,56,110`; `scope.rs:28`;
  `session_helpers.rs:278`.

## Target shape

The shape is given as code. A departure from it is a STOP: report what you
wanted to change and why. You fill in bodies and adapt the call sites.

### 1. The thread's entry record (`native_stack_guard.rs`)

```rust
thread_local! {
    /// Stack address of the outermost rum entry on this thread; 0 while no rum
    /// code runs here. A property of the thread, never of a Session, so a
    /// Session that moves threads cannot carry a stale one.
    static THREAD_NATIVE_STACK_BASE: Cell<usize> = const { Cell::new(0) };
}

/// Held for the extent of one rum entry on this thread. Nested entries (a
/// resume inside an evaluation, abandonment inside a request, a second Session
/// in a host callback) keep the outermost base. Not `Send`: it cannot leave the
/// thread it describes.
pub(in crate::runtime) struct NativeStackEntry {
    outermost: bool,
    _thread_bound: PhantomData<*const ()>,
}

impl NativeStackEntry {
    #[inline(always)]
    pub(in crate::runtime) fn enter() -> Self { /* marker address; set the cell iff it is 0 */ }
}

impl Drop for NativeStackEntry {
    fn drop(&mut self) { /* clear the cell iff outermost */ }
}

/// Whether more than `limit` bytes of native stack are in use since this
/// thread's outermost entry. With no entry on this thread, that is an invariant
/// violation: a debug assertion fails, and a release build answers `true`
/// (fail closed).
#[inline]
pub(in crate::runtime) fn native_stack_exhausted(current: usize, limit: usize) -> bool;

#[cfg(test)]
/// Runs `f` with this thread's record set to `base`, restoring the previous
/// value afterwards, also on unwind. For unit tests that drive the guard
/// without an entry.
pub(in crate::runtime) fn with_thread_native_stack_base<R>(base: usize, f: impl FnOnce() -> R) -> R;
```

`NativeStackGuard` (the `run_with_native_stack_guard` token) drops its `base`
field and decides through `native_stack_exhausted`. The two byte checks in
`eval_core/mod.rs:102-121` keep the marker in their own cold frames and
decide through `native_stack_exhausted`. `run_with_native_stack_guard`'s
public signature does not change.

### 2. The depth type

Put it in `crates/rum-runtime/src/runtime/model/` beside `Session`.

```rust
/// The evaluator's logical depth and the depth at which it next checks native
/// stack headroom. The threshold is derived from the depth, so no code outside
/// this type can move one without the other. `Copy`: saving and restoring the
/// whole value keeps a coherent pair.
#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) struct EvaluationDepth {
    depth: usize,
    next_native_check: usize,
}

impl EvaluationDepth {
    pub(in crate::runtime) const ROOT: Self; // depth 0, threshold NATIVE_STACK_DEPTH_POLL_STRIDE
    #[inline(always)] pub(in crate::runtime) fn get(self) -> usize;
    /// Every move other than the evaluator's own level entry and exit: a
    /// continuation restore, an abandonment reset, a guest-form restore, a
    /// rollback. The threshold becomes `depth + STRIDE`.
    #[inline] pub(in crate::runtime) fn restore(&mut self, depth: usize);
    /// The evaluator's check, today's `depth >= next` then `next = depth + STRIDE`.
    #[inline(always)] pub(in crate::runtime) fn native_check_due(&mut self) -> bool;
    /// `poll_resumed_native_stack`'s re-arm: threshold = depth + STRIDE.
    #[inline(always)] pub(in crate::runtime) fn rearm_native_check(&mut self);
    /// The evaluator's level entry, `depth + 1`; the threshold is untouched.
    #[inline(always)] pub(in crate::runtime) fn enter_level(&mut self);
    /// The evaluator's level exit, back to the depth the level entered at; the
    /// threshold keeps its high-water mark, as today. A debug assertion checks
    /// the depth is `entered_at + 1` here.
    #[inline(always)] pub(in crate::runtime) fn leave_level(&mut self, entered_at: usize);
}
```

`Session` replaces `evaluation_depth: usize` and
`native_stack_next_check_depth: usize` with `evaluation_depth: EvaluationDepth`,
and loses `native_stack_base`. Update the occurrence schema to match: one
row for the new field, and the two removed rows deleted.

### 3. The three entries

```rust
pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
    if self.evaluation_depth.get() == 0 {
        return self.evaluate_at_root(expr);
    }
    self.evaluate_entered(expr)
}

#[cold]
#[inline(never)]
fn evaluate_at_root(&mut self, expr: Expr) -> Result<Expr, EvalError> {
    let _entry = NativeStackEntry::enter();
    self.evaluation_depth.restore(0);
    self.evaluate_entered(expr)
}

#[inline(always)]
fn evaluate_entered(&mut self, expr: Expr) -> Result<Expr, EvalError> {
    // today's body after the depth-0 block, unchanged
}
```

- `resume_suspended_slot` holds `let _entry = NativeStackEntry::enter();` for
  its whole extent. It drops the base save, write and restore, and saves and
  restores the whole `EvaluationDepth` where it saved depth and threshold
  before.
- `drain_abandonment_worklist` holds an entry for its whole extent, after the
  `abandonment_active` early return. It saves the whole `EvaluationDepth` and
  uses `restore(0)` where it writes 0 today (`abandon.rs:33,52,110`).
- Nothing else enters. The debug assertion in `native_stack_exhausted` is
  what proves there is no fourth entry, across the whole test suite.

### 4. The restores

- `restore_resume_context`, `GuestCallerStack::restore` and the rollback
  restore all call `restore(depth)`.
- The evaluator's lines 34, 74 and 77 become `get()`, `enter_level()` and
  `leave_level(evaluation_depth)`.
- Its threshold block (`:54-57`) becomes
  `if self.evaluation_depth.native_check_due() { self.poll_native_stack_limit()?; }`.
- `poll_resumed_native_stack` uses `rearm_native_check()`.

## Must not change

- **Order of execution, and the observable rows.** No evaluation, message,
  print or outcome changes, except the four falsifiers turning green and the
  Jupyter pair passing. For a cleanup that legitimately exceeds the limit,
  the error is still dropped in this brief. Surfacing it is the next brief.
- **When the evaluator polls.** The threshold rule in the evaluator is the
  same high-water rule. Only restores move it down.
- **`Session::evaluate` at depth above 0.** No new local, no guard, no drop
  branch, and no TLS access.
- **Stack shape.** Measured in Stage B: no `Session::evaluate*` function's
  stack reservation grows.
- **The public surface.** `run_with_native_stack_guard`, `NativeStackDepth`
  and its `compile_fail` doctest, and every `rum-host` API.
- **Existing tests keep their intent.** Three tests poke the removed field:
  `tests/native_stack_guard.rs:7,27`, `evaluation_cache/tests.rs:488`, and
  `tests/resume_evaluation_context.rs:37,163-167,405-417`.
  - Rewrite them through `with_thread_native_stack_base`, or through a real
    entry, so each still asserts what it asserted.
  - The base-0 test (`native_stack_guard.rs:27`) means "budget available".
    Express that with a nonzero base below every stack address, such as 1,
    because 0 now means "no entry".
  - Disclose each rewrite as before/after in the report.

## Forbidden

- Keeping a copy of the base in `Session`, under any name.
- An `EvaluationDepth` constructor or setter beyond the ones listed.
- Resetting the threshold at the restore sites by hand instead of through
  the type.
- A TLS read in `Session::evaluate` at depth above 0, or in the evaluator's
  level entry.
- `unsafe`, a new dependency, or `#[inline]` attributes keyed on
  `debug_assertions`.
- `#[ignore]`, retries, sleeps, or a weakened assertion.

## Falsifiers (calibrated by the director: red at HEAD 3/3, controls green)

- **Add `crates/rum-host/tests/native_stack_thread_scope.rs`** with all eight
  tests from `.temp/brief-628/probe/src/lib.rs`: four `brief628_*`
  falsifiers, their controls, and the three `dir628_*` cases.
  - Rename them descriptively.
  - Keep each control assertion ahead of its treatment: the direct control
    must hit `StackLimit` before the treatment is read.
  - The probe depends on `rum-runtime` only through `rum-host`'s existing
    dev-dependency, so it needs no new dependency.
- **Prove red on the base.** Copy the new file into
  `.temp/brief-629/base-source/crates/rum-host/tests/`. That directory is my
  export of `678e6be7`, code-identical to HEAD. Run it there 3 times:
  - the four falsifiers must fail;
  - the controls and `dir628` cases must pass.

  If that table differs from my calibration
  (`.temp/director/628/probe-run-{1,2,3}.log`), STOP.
- **Prove green on the candidate.** Run it 3 times; all pass.
- **The Jupyter pair.** Run it 20 times at `--test-threads=2`. At HEAD it
  failed 17 of 20. All 20 must pass.

  ```
  cargo test -p rum-jupyter --lib -- --test-threads=2 --exact \
    tests::execute_lifecycle::interrupt::red_interrupt_while_waiting_finishes_request \
    tests::execute_lifecycle::interrupt::interrupt_at_second_prompt_finishes_original_request
  ```

  Its nine native ZeroMQ siblings cannot bind loopback in your sandbox. Do
  not run them; my gate does.

## Stage A — the shape and its proofs, then STOP

1. Implement the shape.
2. Run `cargo fmt --check`.
3. Run the falsifier proofs above: base red, candidate green, the Jupyter
   pair.
4. Run the focused suites, single-threaded:
   - `cargo test -p rum-runtime --lib`;
   - `cargo test -p rum-host` (all targets);
   - `cargo test -p rum-stdlib --lib native_stack` and
     `cargo test -p rum-stdlib --lib depth`, which cover the depth contract
     and guarded traversals;
   - `frame_root_trace`;
   - the standing maintenance duty (TEMPLATE §Scope): contract fingerprints,
     `local_production_acceptance` in both builds, and the ownership
     inventories reconciled.
5. Write the Stage A report and STOP. Do not start Stage B.

## Stage B — measurement (only in a continuation, after my gate)

My instruments are ready in `.temp/brief-629/`:

- **Base arm.** `arms/base/{rum-cli,resume}` are release builds with one
  codegen unit, from `base-source/`, already built.
- **Candidate arm.** Build it from the repository the same way:
  - `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=.temp/brief-629/target-cand cargo build --release -p rum-cli`;
  - the harness, `harness-cand/` (path dependency on the repository's
    `rum-host`), into `.temp/brief-629/target-cand-harness`;
  - copy both binaries to `arms/cand/{rum-cli,resume}`.
- **The slope instrument.** `instruments/instr.py <armA> <armB> <out.tsv>`
  covers the brief-626 label set: 313 labels, 192 of them CLI and 121 resume.
  - It runs `perf stat cpu_core/instructions/u` under `taskset -c 2` at N
    and 2N, three fresh processes per size per arm, and checks each run's
    output before counting it.
  - It resumes from its own TSV.
  - My calibration, base against base on 5 labels, gave |delta| ≤ 0.042%
    (`logs/smoke-bb.tsv`).

The campaign:

1. **Floor.** Run `instr.py arms/base arms/base logs/bb.tsv`. The per-label
   floor is `max(0.05%, |delta|)`.
2. **Gate.** Run `instr.py arms/base arms/cand logs/bc.tsv`. A label whose
   delta exceeds its floor needs attribution. Use callgrind per function, 6
   or more fresh processes per arm, and a diff of per-function slopes. Name
   the mechanism.
   - **Expected:** the thread-record read in the two cold checks; the
     entry's enter and exit per resume and per abandonment; `restore`'s
     threshold store per resumed cell.
   - **Anything else is a STOP.** So is any unattributed positive residual.
3. **Stack.** For every `Session::evaluate*` symbol, and every function in
   the evaluator's recursive cycle, run `objdump -d -C` on both arms'
   `rum-cli`. Compare each prologue's stack reservation. Any growth is a
   STOP.
4. Run `cargo test --workspace` single-threaded, and report its summary
   line.

## Scope

- **MAY modify:**
  - in `crates/rum-runtime/src/runtime/`: `native_stack_guard.rs`,
    `resolution.rs`, `eval_core/mod.rs`, `model/session.rs`, the new model
    file, `session/session_state.rs`, `session/session_helpers.rs`,
    `session/guest_loading/scope.rs`, `suspension/core.rs`,
    `suspension/core/abandon.rs`, `suspension/core/evaluation_context.rs`,
    `debug_tools/resources/occurrence/schema.rs`, and `model/mod.rs` if the
    new file needs registering;
  - the three tests named above;
  - the new `crates/rum-host/tests/native_stack_thread_scope.rs`;
  - the standing inventories, by replay.
- **Disclosure duty:** a file not listed that must change, for example
  another reader of the removed fields, may change only mechanically, to
  `get()` or `restore()`. Disclose it with its reason. Anything beyond a
  mechanical change is a STOP.
- **MUST NOT touch:** `rum-core`, `rum-jupyter`, `rum-cli`, `rum-repl`,
  `docs/`, `TODO.md`, `Cargo.lock`, `.cargo/`, `scripts/`.

## Concurrency plan

- **Builds:** one cargo command at a time, `CARGO_BUILD_JOBS=8`, inside your
  16G scope.
- **Tests:** `RUST_TEST_THREADS=1`. The one exception is the Jupyter pair at
  `--test-threads=2`, which is the falsifier's variable.
- **Stage B:** `perf stat`, callgrind and objdump run serially, and nothing
  else runs during `perf` or callgrind.
- Give every command a bound with `scripts/run-bounded.py`, sized from a
  measured run.

## Stop-loss

The count of compile errors and failing focused tests must keep falling
across iterations (TEMPLATE). If it rises twice in a row, STOP and
checkpoint.

## Report

Append "Executing-agent report — Stage A" below:

1. Each changed file, its line ranges, and a one-line reason.
2. The shape: each item of the Target shape, with its final code location.
3. The falsifier tables: base 3 runs, candidate 3 runs, the Jupyter pair
   20 runs. Give summary lines verbatim.
4. The rewrites of the existing tests, before and after.
5. The focused suites' `test result:` lines and the inventory disclosures.
6. Every attempt with its result, and the open questions.

Keep `.temp/brief-629/PROGRESS.md` current.
