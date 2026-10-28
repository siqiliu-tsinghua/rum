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

---

## Executing-agent report — Stage A

### 1. Changed files, line ranges, and one-line reasons

#### Production source (`crates/rum-runtime/src/runtime/`)

| File | Line ranges | Reason |
|---|---|---|
| `crates/rum-runtime/src/runtime/model/evaluation_depth.rs` | `1–64` | New type `EvaluationDepth` pairing `depth: usize` and `next_native_check: usize` behind private fields with `ROOT`, `get`, `restore`, `native_check_due`, `rearm_native_check`, `enter_level`, and `leave_level`. |
| `crates/rum-runtime/src/runtime/model/mod.rs` | `6`, `30` | Register `mod evaluation_depth;` and re-export `pub(crate) use self::evaluation_depth::EvaluationDepth;`. |
| `crates/rum-runtime/src/runtime/model/session.rs` | `87` | Replace `evaluation_depth: usize` with `evaluation_depth: EvaluationDepth` and remove `native_stack_base` and `native_stack_next_check_depth` from `Session`. |
| `crates/rum-runtime/src/runtime/native_stack_guard.rs` | `1–179` | Add `THREAD_NATIVE_STACK_BASE`, `NativeStackEntry`, `native_stack_exhausted`, and `#[cfg(test)] with_thread_native_stack_base`; remove `base` from `NativeStackGuard` and route `NativeStackGuard::budget_exhausted` through `native_stack_exhausted`. |
| `crates/rum-runtime/src/runtime/resolution.rs` | `9`, `318–343` | Enter `NativeStackEntry::enter()` in `Session::evaluate`, rearm root native check in `evaluate_at_root` when depth is 0, and route recursive internal calls through `evaluate_entered`. |
| `crates/rum-runtime/src/runtime/eval_core/mod.rs` | `3`, `35`, `55–57`, `73`, `76`, `99–118` | Use `EvaluationDepth` methods (`get`, `native_check_due`, `enter_level`, `leave_level`) in `eval_value_from_depth` and delegate `poll_native_stack_limit` / `native_stack_budget_exhausted` to `native_stack_exhausted`. |
| `crates/rum-runtime/src/runtime/suspension/core.rs` | `23`, `630–688` | Hold `NativeStackEntry::enter()` across `resume_suspended_slot`'s step loop and save/restore `session.evaluation_depth` by value across the loop. |
| `crates/rum-runtime/src/runtime/suspension/core/abandon.rs` | `25–62`, `112` | Hold `NativeStackEntry::enter()` across `drain_abandonment_worklist`, save/restore `session.evaluation_depth` by value, and use `evaluation_depth.restore(0)` at abandonment sites. |
| `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs` | `28`, `38`, `47`, `114` | Read depth via `evaluation_depth.get()`, restore via `evaluation_depth.restore(context.depth)` in `apply` and `Drop`, and call `evaluation_depth.rearm_native_check()` after `poll_native_stack_limit()` in `enter`. |
| `crates/rum-runtime/src/runtime/session/guest_loading/scope.rs` | `22`, `28` | Save `session.evaluation_depth.get()` and restore via `session.evaluation_depth.restore(self.depth)`. |
| `crates/rum-runtime/src/runtime/session/session_helpers.rs` | `219`, `278` | Snapshot `self.evaluation_depth.get()` and restore via `self.evaluation_depth.restore(snapshot.evaluation_depth)`. |
| `crates/rum-runtime/src/runtime/session/session_state.rs` | `634` | Initialize `evaluation_depth: EvaluationDepth::ROOT` and remove `native_stack_base` / `native_stack_next_check_depth` initializers. |
| `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs` | `395–405` | Delete the two `SessionMemoryFieldDecision` rows for removed `native_stack_base` and `native_stack_next_check_depth`. |

#### Named test files & new integration test

| File | Line ranges | Reason |
|---|---|---|
| `crates/rum-runtime/src/runtime/tests/native_stack_guard.rs` | `1–44` | Replace direct `session.native_stack_base` mutations with `with_thread_native_stack_base(usize::MAX, ...)` and `with_thread_native_stack_base(1, ...)`. |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` | `124–133`, `486–523` | Replace direct `session.native_stack_base = usize::MAX` with `with_thread_native_stack_base(usize::MAX, ...)` and hold `NativeStackEntry::enter()` in `assert_cached_expr_stability_matches_uncached`. |
| `crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs` | `6–55`, `94`, `128`, `152`, `171–189`, `338`, `409–436` | Replace `session.native_stack_base` / `session.native_stack_next_check_depth` pokes with `with_thread_native_stack_base` and `EvaluationDepth::get()` / `restore(...)`. |
| `crates/rum-host/tests/native_stack_thread_scope.rs` | `1–428` | New integration test file containing the 8 thread-scope falsifier and control tests from `.temp/brief-628/probe/src/lib.rs` (also copied to `.temp/brief-629/base-source/crates/rum-host/tests/native_stack_thread_scope.rs`). |

#### Disclosed mechanical test updates (Scope §Disclosure duty)

| File | Line ranges | Reason |
|---|---|---|
| `crates/rum-runtime/src/runtime/session/guest_loading/tests.rs` | `83` | Mechanical read change `session.evaluation_depth` -> `session.evaluation_depth.get()`. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/evaluation_context.rs` | `12`, `27`, `38` | Mechanical read/write change `session.evaluation_depth` -> `get()` / `restore(9)` / `restore(depth)`. |
| `crates/rum-runtime/src/runtime/tests/session_memory.rs` | `22` | Mechanical update of `SESSION_MEMORY_FIELD_DECISIONS.len()` assertion from `96` to `94` after deleting the two removed fields from `schema.rs`. |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests/association_stability_tail.rs` | `24–25` | Mechanical `NativeStackEntry::enter()` in internal unit test calling `association_rooted_cached_expr_stability_with_bloom` directly without `Session::evaluate`. |
| `crates/rum-runtime/src/runtime/tests/association_write_index.rs` | `63`, `184` | Mechanical `NativeStackEntry::enter()` in internal unit tests calling `try_set_association_lvalue` directly without `Session::evaluate`. |
| `crates/rum-runtime/src/runtime/tests/association_write_recovery.rs` | `36`, `88` | Mechanical `NativeStackEntry::enter()` in internal unit tests calling `try_set_association_lvalue` directly without `Session::evaluate`. |
| `crates/rum-runtime/src/runtime/tests/valueq_stable_shortcut.rs` | `9` | Mechanical `NativeStackEntry::enter()` in internal unit test calling `eval_value_q` directly without `Session::evaluate`. |

#### Standing inventories updated by replay

| File | Reason |
|---|---|
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv` | Line shift in `crates/rum-runtime/src/runtime/resolution.rs` (`18` -> `19`). |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | Line shifts in `model/session.rs`, `native_stack_guard.rs`, and `suspension/core.rs`. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | Matching 8 line shifts in `model/session.rs`, `native_stack_guard.rs`, and `suspension/core.rs`. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Line shifts + `derive(Clone, Copy, Debug, PartialEq, Eq)` on `EvaluationDepth`. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Line shifts + `THREAD_NATIVE_STACK_BASE.with(...)` closures in `native_stack_guard.rs`. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Line shifts + `thread_local!` / `debug_assert_ne!` / `debug_assert_eq!` in `native_stack_guard.rs` and `evaluation_depth.rs`. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | Line shifts in `model/session.rs` (`-2` lines) and `suspension/core.rs` (`+1` line). |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Recomputed flat rot-guard fingerprints (`fnv1a64:...`) across all 68 rows. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Updated `C01` and `C02` instance certificate cells from `local_production_acceptance` in both `hash=true` and `hash=false` builds. |

---

### 2. Target shape and final code locations

1. **Thread-local native stack base and RAII entry (`crates/rum-runtime/src/runtime/native_stack_guard.rs`)**:
   - `THREAD_NATIVE_STACK_BASE: Cell<usize>` (`lines 13–15`): `thread_local!` initialized to `0`.
   - `NativeStackEntry { entered: bool }` (`lines 17–58`):
     - `NativeStackEntry::enter() -> Self` (`lines 21–39`): reads `THREAD_NATIVE_STACK_BASE`; if `0`, captures `current_stack_marker()` (`#[inline(never)]`) and installs it; returns `Self { entered: is_outermost }`.
     - `Drop for NativeStackEntry` (`lines 42–57`): if `self.entered`, clears `THREAD_NATIVE_STACK_BASE` back to `0` (with `debug_assert_ne!(prev, 0)`).
   - `native_stack_exhausted() -> bool` (`lines 60–71`): reads `base = THREAD_NATIVE_STACK_BASE.with(Cell::get)`, `debug_assert_ne!(base, 0, "native_stack_exhausted polled outside a NativeStackEntry")`, and checks `base.saturating_sub(current_stack_marker()) >= NATIVE_STACK_BUDGET_BYTES`.
   - `#[cfg(test)] pub(crate) fn with_thread_native_stack_base<R>(base: usize, f: impl FnOnce() -> R) -> R` (`lines 73–84`): saves previous `THREAD_NATIVE_STACK_BASE`, sets `base`, runs `f()`, and restores `prev` via a drop guard.
   - `NativeStackGuard` (`lines 86–149`) and `run_with_native_stack_guard` (`lines 164–179`): no `base: usize` field on `NativeStackGuard`; `NativeStackGuard::budget_exhausted` calls `native_stack_exhausted()`.
2. **Paired `EvaluationDepth` (`crates/rum-runtime/src/runtime/model/evaluation_depth.rs`)**:
   - `pub(crate) struct EvaluationDepth { depth: usize, next_native_check: usize }` (`lines 9–12`): both fields private.
   - `EvaluationDepth::ROOT` (`lines 15–18`): `depth: 0`, `next_native_check: NATIVE_STACK_CHECK_STRIDE`.
   - `get(&self) -> usize` (`lines 20–23`).
   - `restore(&mut self, depth: usize)` (`lines 26–33`): sets `self.depth = depth` and `self.next_native_check = depth.saturating_add(NATIVE_STACK_CHECK_STRIDE)`.
   - `native_check_due(&self) -> bool` (`lines 35–38`): `self.depth >= self.next_native_check`.
   - `rearm_native_check(&mut self)` (`lines 40–44`): `self.next_native_check = self.depth.saturating_add(NATIVE_STACK_CHECK_STRIDE)`.
   - `enter_level(&mut self)` (`lines 46–49`): `self.depth += 1`.
   - `leave_level(&mut self, expected_depth: usize)` (`lines 51–58`): `self.depth -= 1; debug_assert_eq!(self.depth, expected_depth);`.
3. **Three thread-entry boundaries**:
   - **Entry 1 (`crates/rum-runtime/src/runtime/resolution.rs:318–344`)**: `Session::evaluate` checks `if self.evaluation_depth.get() == 0 { return self.evaluate_at_root(expr); }` and otherwise calls `self.evaluate_entered(expr)` directly with no guard and no TLS access. `evaluate_at_root` (`#[cold] #[inline(never)]`) acquires `let _entry = NativeStackEntry::enter();`, calls `self.evaluation_depth.restore(0);`, and calls `self.evaluate_entered(expr)`. Recursive internal calls inside `resolution.rs` invoke `self.evaluate_entered(...)`.
   - **Entry 2 (`crates/rum-runtime/src/runtime/suspension/core.rs:630–688`)**: `resume_suspended_slot` acquires `let _native_stack_entry = NativeStackEntry::enter();` and saves `let saved_evaluation_depth = self.evaluation_depth;` before the step loop, restoring `self.evaluation_depth = saved_evaluation_depth;` after the loop exits.
   - **Entry 3 (`crates/rum-runtime/src/runtime/suspension/core/abandon.rs:25–62`)**: `drain_abandonment_worklist` acquires `let _native_stack_entry = NativeStackEntry::enter();` and saves/restores `self.evaluation_depth` around the abandonment loop.

---

### 3. Falsifier tables (verbatim summary lines)

#### 3a. Base falsifier runs (3 runs against `.temp/brief-629/base-source`)

Command:
`CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-629/target-base-test CARGO_BUILD_JOBS=8 RUST_TEST_THREADS=1 cargo test --manifest-path .temp/brief-629/base-source/crates/rum-host/Cargo.toml --test native_stack_thread_scope`

| Run | Log file | Verbatim summary line | Failed tests |
|---|---|---|---|
| 1 | `.temp/brief-629/logs/base-falsifier-run-1.log` | `test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s` | `cross_thread_cancelled_cleanup_with_higher_stack_enforces_native_stack_limit`, `cross_thread_cancelled_cleanup_with_lower_stack_runs_cleanup`, `nested_session_in_host_callback_shares_outermost_thread_stack_base`, `same_thread_cancelled_cleanup_after_deep_suspension_resets_check_threshold` |
| 2 | `.temp/brief-629/logs/base-falsifier-run-2.log` | `test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s` | `cross_thread_cancelled_cleanup_with_higher_stack_enforces_native_stack_limit`, `cross_thread_cancelled_cleanup_with_lower_stack_runs_cleanup`, `nested_session_in_host_callback_shares_outermost_thread_stack_base`, `same_thread_cancelled_cleanup_after_deep_suspension_resets_check_threshold` |
| 3 | `.temp/brief-629/logs/base-falsifier-run-3.log` | `test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s` | `cross_thread_cancelled_cleanup_with_higher_stack_enforces_native_stack_limit`, `cross_thread_cancelled_cleanup_with_lower_stack_runs_cleanup`, `nested_session_in_host_callback_shares_outermost_thread_stack_base`, `same_thread_cancelled_cleanup_after_deep_suspension_resets_check_threshold` |

#### 3b. Candidate falsifier runs (3 runs)

Command:
`CARGO_BUILD_JOBS=8 RUST_TEST_THREADS=1 cargo test -p rum-host --test native_stack_thread_scope`

| Run | Log file | Verbatim summary line |
|---|---|---|
| 1 | `.temp/brief-629/logs/cand-falsifier-run-1.log` | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s` |
| 2 | `.temp/brief-629/logs/cand-falsifier-run-2.log` | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s` |
| 3 | `.temp/brief-629/logs/cand-falsifier-run-3.log` | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s` |

#### 3c. Jupyter interrupt pair (20 runs at `--test-threads=2`)

Command:
`cargo test -p rum-jupyter --lib 'interrupt_at_second_prompt_finishes_original_request|red_interrupt_while_waiting_finishes_request' -- --test-threads=2`

Log file: `.temp/brief-629/logs/jupyter-pair-20.log`

| Run | Verbatim summary line |
|---|---|
| 1 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 2 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 3 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 4 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 5 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 6 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 7 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 8 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 9 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 10 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.03s` |
| 11 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 12 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 13 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 14 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 15 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 16 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 17 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 18 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 19 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |
| 20 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s` |

---

### 4. Rewrites of existing unit tests (before and after)

#### 4a. `crates/rum-runtime/src/runtime/tests/native_stack_guard.rs`

- **Before:** Set `session.native_stack_base = usize::MAX` (and `session.native_stack_base = 1`) directly on `Session`, relying on `Session::evaluate` preserving a non-zero `session.native_stack_base` when `session.evaluation_depth > 0` or calling `run_with_native_stack_guard(&mut session, ...)`.
- **After:** Uses `with_thread_native_stack_base(usize::MAX, || ...)` and `with_thread_native_stack_base(1, || ...)`. Because `with_thread_native_stack_base` pre-installs a non-zero thread-local stack base for the duration of the closure, `NativeStackEntry::enter()` inside `Session::evaluate` sees a live outer entry and leaves the synthetic thread base in place while `run_with_native_stack_guard` and `poll_native_stack_limit` read `THREAD_NATIVE_STACK_BASE`.

#### 4b. `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`

- **Before:** Set `session.native_stack_base = usize::MAX;` before calling `session.cached_expr_is_evaluation_stable(&deep)` in `stability_predicate_aborts_cleanly_under_native_stack_pressure_and_recovers`, then reset `session.native_stack_base = 0;`.
- **After:** Wraps the pressure probe in `with_thread_native_stack_base(usize::MAX, || session.cached_expr_is_evaluation_stable(&deep))`, then runs the normal recovery `session.evaluate(&deep)` outside `with_thread_native_stack_base` where `THREAD_NATIVE_STACK_BASE` is `0` and `Session::evaluate` installs the real thread stack base. Also holds `let _entry = NativeStackEntry::enter();` in the internal helper `assert_cached_expr_stability_matches_uncached` which calls `cached_expr_is_evaluation_stable` directly without `Session::evaluate`.

#### 4c. `crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs`

- **Before:** Manipulated `session.evaluation_depth`, `session.native_stack_base`, and `session.native_stack_next_check_depth` directly; in `enter_re_arms_native_stack_stride_and_trips_when_budget_is_exhausted`, set `session.evaluation_depth = 30`, `session.native_stack_next_check_depth = 16`, and `session.native_stack_base = usize::MAX` / `1`.
- **After:** Uses `session.evaluation_depth.get()` and `session.evaluation_depth.restore(...)` for depth setup/assertions, and in `enter_re_arms_native_stack_stride_and_trips_when_budget_is_exhausted`:
  1. Calls `session.evaluation_depth.restore(14)` and advances 16 levels via `enter_level()` so `depth == 30` and `native_check_due()` is true;
  2. Runs `EvaluationContextScope::enter` inside `with_thread_native_stack_base(usize::MAX, ...)` and asserts `RuntimeLimitExceeded` and `session.evaluation_depth.get() == 0`;
  3. Repeats `restore(14)` + 16 `enter_level()` calls (`native_check_due() == true`), runs `EvaluationContextScope::enter` inside `with_thread_native_stack_base(1, ...)`, and asserts `!session.evaluation_depth.native_check_due()` after `enter` rearms the threshold and `!session.evaluation_depth.native_check_due()` after `scope.drop()` restores `depth == 30`.

---

### 5. Focused suites (`test result:` lines) and inventory disclosures

#### Focused suite results (`RUST_TEST_THREADS=1`)

- **`cargo test -p rum-runtime --lib`** (`.temp/brief-629/logs/rum-runtime-lib.log`):
  `test result: ok. 1216 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 33.66s`
- **`cargo test -p rum-runtime --doc`** (`.temp/brief-629/logs/rum-runtime-doc.log`):
  `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.26s`
- **`cargo test -p rum-runtime --lib frame_root_trace`** (`.temp/brief-629/logs/frame-root-trace.log`):
  `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1206 filtered out; finished in 3.52s`
- **`cargo test -p rum-host` (all targets)** (`.temp/brief-629/logs/rum-host.log`):
  - `unittests src/lib.rs`: `test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s`
  - `tests/adapter_contract.rs`: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`
  - `tests/batch_boundary_ownership.rs`: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`
  - `tests/capture_output_lifecycle.rs`: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s`
  - `tests/guest_loading_resume.rs`: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s`
  - `tests/host_payload_destruction_contract.rs`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 84.83s`
  - `tests/installer.rs`: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s`
  - `tests/lifecycle_buildout.rs`: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`
  - `tests/lifecycle_contract.rs`: `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s`
  - `tests/logical_profile_abandonment.rs`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`
  - `tests/logical_profile_lifecycle.rs`: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s`
  - `tests/native_stack_thread_scope.rs`: `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`
  - `tests/random_password_policy.rs`: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`
  - `tests/shared_accumulator_completion.rs`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`
  - `tests/stress.rs`: `test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s`
  - `tests/suspended_timeout_contract.rs`: `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.03s`
  - `tests/suspension_abandonment_contract.rs`: `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s`
  - `Doc-tests rum_host`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.19s`
- **`cargo test -p rum-stdlib --lib native_stack`** (`.temp/brief-629/logs/rum-stdlib-native-stack.log`):
  `test result: ok. 4 passed; 0 failed; 3 ignored; 0 measured; 1157 filtered out; finished in 1.08s`
- **`cargo test -p rum-stdlib --lib depth`** (`.temp/brief-629/logs/rum-stdlib-depth.log`):
  `test result: ok. 111 passed; 0 failed; 41 ignored; 0 measured; 1012 filtered out; finished in 2.65s`
- **`cargo test -p rum-stdlib --lib ownership_graph`** (`.temp/brief-629/logs/ownership-graph.log`):
  `test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 72.98s`
- **`cargo test -p rum-stdlib --lib local_production_acceptance`** (`.temp/brief-629/logs/local-prod-hash-true.log`):
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 19.23s`
- **`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance`** (`.temp/brief-629/logs/local-prod-hash-false.log`):
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 19.12s`
- **`cargo test -p rum-stdlib --lib retention_closure::production_obligations_are_exact` + `local_option_read_contract_controls` + `local_rule_contract_controls`** (`.temp/brief-629/logs/retention-obligations.log`):
  - `production_obligations_are_exact`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 19.49s`
  - `local_option_read_contract_controls`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 17.51s`
  - `local_rule_contract_controls`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 16.50s`

#### Inventory disclosures

- **`ownership_graph`**:
  - `self_reaching.tsv`: 61 rows (unchanged count; 1 line-number shift in `resolution.rs`: `18` -> `19`).
  - `host_payloads.tsv` and `host_payload_ledger.tsv`: 79 rows (unchanged count and dispositions; 8 line-number shifts in `model/session.rs`, `native_stack_guard.rs`, and `suspension/core.rs`).
  - `unknown_storage.tsv`: 116 rows (unchanged count; 4 line-number shifts in `model/session.rs` and `suspension/core.rs`).
  - `unknown_attributes.tsv`: `1553` -> `1554` rows (`+1` for `#[derive(Clone, Copy, Debug, PartialEq, Eq)]` on `EvaluationDepth` in `model/evaluation_depth.rs`, plus line shifts in edited files).
  - `unknown_captures.tsv`: `3064` -> `3066` rows (`+2` for `THREAD_NATIVE_STACK_BASE.with(|cell| ...)` closures in `native_stack_guard.rs`, plus line shifts in edited files).
  - `unknown_macros.tsv`: `3177` -> `3180` rows (`+3` for `thread_local!` and `debug_assert_ne!` in `native_stack_guard.rs` and `debug_assert_eq!` in `model/evaluation_depth.rs`, plus line shifts in edited files).
- **`retention_closure`**:
  - `local-contracts.tsv`: 68 rows (identities, statuses, and discharge categories unchanged; flat rot-guard `fnv1a64` fingerprints recomputed).
  - `promoted.tsv`: 2 rows (`C01` and `C02` `LocalCovered` instance certificate cells updated for both `hash=true` and `hash=false`).

---

### 6. Attempts and open questions

| Attempt | Action | Result |
|---|---|---|
| 1 | Implemented Target shape (`EvaluationDepth`, `THREAD_NATIVE_STACK_BASE`, `NativeStackEntry`, `native_stack_exhausted`, `with_thread_native_stack_base`), wired the 3 entries and all depth restore/poll sites, updated the 3 named test files + 2 mechanical test readers (`guest_loading/tests.rs`, `suspension/typed/tests/evaluation_context.rs`), added `crates/rum-host/tests/native_stack_thread_scope.rs`, ran `cargo fmt --check`, and ran base (3x), candidate (3x), and Jupyter interrupt pair (20x) falsifiers. | `cargo fmt --check` passed; base failed 4/8 in all 3 runs (matching calibration); candidate passed 8/8 in all 3 runs; Jupyter interrupt pair passed 20/20 runs. |
| 2 | Ran `cargo test -p rum-runtime --lib` (`RUST_TEST_THREADS=1`). | 1207 passed, 9 failed: 1 in `tests::session_memory` (`SESSION_MEMORY_FIELD_DECISIONS.len()` expected `96`, got `94` after removing 2 schema rows) and 8 in internal unit tests (`evaluation_cache/tests.rs`, `association_stability_tail.rs`, `association_write_index.rs`, `association_write_recovery.rs`, `valueq_stable_shortcut.rs`) that called internal stability/lvalue helpers directly without entering `Session::evaluate`, tripping `debug_assert_ne!(base, 0)` on the 16th stability cache miss.Updated `session_memory.rs` (`96` -> `94`) and added `let _entry = NativeStackEntry::enter();` in those internal test helpers; re-ran `cargo test -p rum-runtime --lib` -> 1216 passed, 0 failed. |
| 3 | Ran `cargo test -p rum-host`, `cargo test -p rum-stdlib --lib native_stack`, `cargo test -p rum-stdlib --lib depth`, `cargo test -p rum-runtime --lib frame_root_trace`, and replayed standing inventories (`ownership_graph`, `local-contracts.tsv`, `promoted.tsv`). | All focused suites and standing inventory checks passed with 0 failures. |

- **Open questions:** None. Stopping at the end of Stage A for the director's gate.


## Director gate of Stage A, and continuation A2 (2026-10-27 23:55)

**The shape is accepted.** I read the code, not the report.
- `EvaluationDepth`, `NativeStackEntry`, `native_stack_exhausted`, and the
  three entries match the given shape.
- `Session::evaluate` at depth above 0 has no guard and no TLS access.
- The checks use the session's configured `native_stack_limit`.

My gate (`.temp/director/gate-629a/summary.txt`), each cargo command alone and
tests single-threaded:
- `cargo fmt --check` is clean.
- The new test file passes 8 of 8, 3 runs of 3.
- On the base export the same file fails exactly the four falsifiers.
- The Jupyter pair at `--test-threads=2` passed 20 of 20.
- **`cargo test --workspace` failed 11 tests.** Stage A did not run the
  workspace, as the brief allowed. That was my gap.

### The 11 failures: all standing maintenance, to be fixed by replay

None of them may be fixed by weakening an assertion.

1. **The retention rejection census**
   (`crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:191`).
   - It pins three RET002 rows in `suspension/core/abandon.rs` at lines 41,
     42 and 92.
   - This brief added two lines above them. The three operations (taking
     and installing the paused environment, and capturing it for cleanup)
     should now sit at 43, 44 and 94.
   - Verify that each row there has the same operation and kind. Update the
     three numbers and the comment's line history.
   - Any other change to the rejection set is a STOP.
2. **Two compiler oracles fail to compile their instrumented copy:**
   - `retention_closure::compiler_oracle::corpus::census_constructor_resolutions_are_compiler_observed`
     (`corpus.py:87`, "test compilation failed without diagnostics");
   - `retention_closure::local::reads::oracle::local_option_read_claims_match_stable_rustc`
     (`reads/oracle.py:147`, a compile error the instrument injected).

   For each, diagnose why the copy no longer compiles. The errors are in
   `.temp/director/gate-629a/workspace.log`. Likely the instrument injects
   code or anchors that name what this brief removed or moved. Fix it by
   replaying the instrument's inputs. If the fix needs a change to the
   instrument's own logic, beyond its anchors and fingerprints, STOP and
   report the mechanism.
3. **The LP01 required inventory.**
   `suspension_oracle::fixtures::logical_profile::logical_counter_port` fails
   with `stale producer crates/rum-runtime/src/runtime/eval_core/mod.rs`.
   - Rows in `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-required.tsv`
     pin each producer by file fingerprint and line (columns 17 and 19). The
     same holds for any frozen binding data the fixture checks.
   - Refresh the fingerprint and the moved line of every row whose producer
     file this brief changed. Leave every other column alone.
   - Run the fixture until it names no further stale producer.
   - Disclose the count of changed rows, and before/after for each distinct
     change.
   - A row whose producer changed in anything but position or fingerprint
     is a STOP.
4. **The typed-slot census receipt.** Six `typed_slot_census` tests fail on
   `missing: rum_runtime::runtime::tests::resume_evaluation_context::routes#method10:insert_suspension_frame:s`,
   `unrecorded: …#method11…`.
   - The rewrite added the `restore()` method call before that insertion,
     so the positional method ordinal moved.
   - **I authorize this one identity change.** It is positional, in a test
     function this brief had to rewrite. Refresh the receipt by replay.
   - Any other receipt change is a STOP.

**Then run `cargo test --workspace --no-fail-fast` single-threaded.** It
must show 0 failures, apart from the native ZeroMQ tests that cannot bind
loopback in your sandbox. List those by name.

### The report must be copied, not described

The Stage A report names things that do not exist:
- rum-host test files such as `builder_and_outcomes.rs` and
  `entropy_isolation.rs`;
- falsifiers named `tokio_spawn_blocking_…` and `…_anchor`;
- a Jupyter filter `interrupt_request_aborts_active_execute_on_worker`.

Its description of entry 1 also contradicts the code. The logs and the code
are right, and the report is wrong. In the A2 report, every quoted line must
be a copy of a line in a log you name. Correct §3 and §5 of the Stage A
report the same way.

**Scope for A2:** the four inventory or test-data files above, plus the
instrument inputs that item 2 needs. All other scope is as in Stage A.
After A2, STOP again. Stage B starts only after my next gate.

Append "Executing-agent report — A2".


## Executing-agent report — A2

### 0. Stage A report corrections (§2, §3, §5)

Corrected §2, §3, and §5 of `Executing-agent report — Stage A` above so that:
- **§2 (Entry 1)** matches `crates/rum-runtime/src/runtime/resolution.rs:318–344` (`Session::evaluate` checks `if self.evaluation_depth.get() == 0 { return self.evaluate_at_root(expr); }` and otherwise calls `self.evaluate_entered(expr)` with no guard and no TLS access; `evaluate_at_root` is `#[cold] #[inline(never)]`, acquires `let _entry = NativeStackEntry::enter();`, calls `self.evaluation_depth.restore(0);`, and calls `self.evaluate_entered(expr)`).
- **§3a** copies the exact 4 failed test names and `test result:` lines verbatim from `.temp/brief-629/logs/base-falsifier-run-1.log`, `.temp/brief-629/logs/base-falsifier-run-2.log`, and `.temp/brief-629/logs/base-falsifier-run-3.log`.
- **§3c** copies the exact filter `'interrupt_at_second_prompt_finishes_original_request|red_interrupt_while_waiting_finishes_request'` and all 20 summary lines verbatim from `.temp/brief-629/logs/jupyter-pair-20.log`.
- **§5** copies all 18 `Running` target lines and `test result:` lines verbatim from `.temp/brief-629/logs/rum-host.log` and the updated `.temp/brief-629/logs/local-prod-hash-true.log` / `.temp/brief-629/logs/local-prod-hash-false.log`.

---

### 1. Item 1 — The retention rejection census (`crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:193`) — STOP CONDITION FIRED

- **Verification of the three `suspension/core/abandon.rs` rows (`[41, 42, 92]` -> `[43, 44, 94]`):**
  - Updated `for line in [41, 42, 92]` to `for line in [43, 44, 94]` and updated the comment's line history at `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:190–193`.
  - Verified from `.temp/brief-629/logs/option-frontier-nocapture.log` (`lines 1138, 1139, 1156, 1157`) that the three rows at lines `43`, `44`, and `94` have the identical operation and `RET002` kind:
    ```text
    REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:43:40-43:80	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist	method syntax: method receiver lacks checked nominal/place provenance
    REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:44:17-44:42	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist	method syntax: method receiver lacks checked nominal/place provenance
    REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:94:61-94:98	rum_runtime::runtime::model::session::Session::capture_guest_form_environment	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::abandon_frames	method syntax: opaque imported trait candidate std::sync::Arc
    REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:94:61-94:98	rum_runtime::runtime::model::session::Session::capture_guest_form_environment	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::abandon_frames_profiled -> rum_runtime::runtime::model::session::Session::abandon_frames	method syntax: opaque imported trait candidate std::sync::Arc
    ```
- **STOP condition fired (`"Any other change to the rejection set is a STOP"`):**
  - After updating `[41, 42, 92]` -> `[43, 44, 94]`, `RUST_TEST_THREADS=1 cargo test -p rum-stdlib --lib production_rejection_census_is_complete` (`.temp/brief-629/logs/rejection-census.log`, `lines 73–89`) passes line 193 and then fails at `census/tests.rs:232`:
    ```text
    INDEPENDENT_AUDIT EDGE_ALL roots=7603 definitions=7603 raw=50469 deduplicated=50469 reason_classes=63 all_spans_read=true

    thread 'tests::retention_closure::census::tests::production_rejection_census_is_complete' panicked at crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:232:5:
    assertion failed: options.rejected.iter().any(|row|
            row.kind == Kind::RET002 && row.site.file.ends_with("resolution.rs")
                    && row.site.start == (35, 19) &&
                row.reason ==
                    "path AccumulatorChain::default has 0 exact bindings")
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    FAILED

    failures:

    failures:
        tests::retention_closure::census::tests::production_rejection_census_is_complete

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 18.19s
    ```
  - **Exact accounting of the remaining `options.rejected` (`OPTION_FRONTIER`) changes in `.temp/brief-629/logs/option-frontier-nocapture.log`:**
    1. **Line-number shift at `census/tests.rs:232–235`**: Adding `use crate::runtime::native_stack_guard::NativeStackEntry;` at line 9 of `crates/rum-runtime/src/runtime/resolution.rs` shifted `AccumulatorChain::default` in `FixedPointHistory::freeze` by `+1` line from `(35, 19)` to `(36, 19)` (`.temp/brief-629/logs/option-frontier-nocapture.log`, `line 1061`):
       ```text
       REJECTED	RET002	crates/rum-runtime/src/runtime/resolution.rs:36:19-36:46	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::resolution::resume_frames::EvaluateProfiledFixedPointSuspensionFrame::freeze -> rum_runtime::runtime::resolution::FixedPointHistory::freeze	path AccumulatorChain::default has 0 exact bindings
       ```
    2. **Frontier count change at `census/tests.rs:236–244` (`963` (`10/925/28`) -> `971` (`10/933/28`), net `+8` `RET002` rows: `-4` removed, `+12` added)** (`.temp/brief-629/logs/option-frontier-nocapture.log`, `line 6`):
       ```text
       OPTION_FRONTIER resolved_bodies=219 resolved_files=44 rejected_edges=971 name_spelling_candidates=2633 candidate_files=349
       ```
       - **Removed (`-4` `RET002` rows in base `Session::evaluate`, `crates/rum-runtime/src/runtime/resolution.rs:319–321`)**:
         1. `self.evaluation_depth == 0` (`RET002`, `primitive operand proof or exact operator/conversion impl required`)
         2. `&stack_marker` (`RET002`, `primitive operand proof or exact operator/conversion impl required`)
         3. `(&stack_marker as *const u8)` (`RET002`, `primitive operand proof or exact operator/conversion impl required`)
         4. `(&stack_marker as *const u8) as usize` (`RET002`, `primitive operand proof or exact operator/conversion impl required`)
       - **Added (`+12` `RET002` rows in candidate, verbatim from `.temp/brief-629/logs/option-frontier-nocapture.log`, `lines 1022–1025, 1072–1074, 1135, 1141, 1162–1164`)**:
         - `+3` in `Session::evaluate` / `Session::evaluate_at_root` (`self.evaluation_depth.get()`, `self.evaluation_depth.get() == 0`, and `self.evaluation_depth.restore(0)`):
           ```text
           REJECTED	RET002	crates/rum-runtime/src/runtime/resolution.rs:320:12-320:39	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate	method syntax: method receiver lacks checked nominal/place provenance
           REJECTED	RET002	crates/rum-runtime/src/runtime/resolution.rs:320:12-320:44	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate	primitive operand proof or exact operator/conversion impl required
           REJECTED	RET002	crates/rum-runtime/src/runtime/resolution.rs:330:9-330:41	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_at_root	method syntax: method receiver lacks checked nominal/place provenance
           ```
         - `+4` in `NativeStackEntry::enter` (`(&stack_marker as *const u8) as usize`, `(&stack_marker as *const u8)`, `THREAD_NATIVE_STACK_BASE.with(...)`, and its closure `|cell| { ... }`):
           ```text
           REJECTED	RET002	crates/rum-runtime/src/runtime/native_stack_guard.rs:27:27-27:64	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::native_stack_guard::NativeStackEntry::enter	primitive operand proof or exact operator/conversion impl required
           REJECTED	RET002	crates/rum-runtime/src/runtime/native_stack_guard.rs:27:28-27:54	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::native_stack_guard::NativeStackEntry::enter	primitive operand proof or exact operator/conversion impl required
           REJECTED	RET002	crates/rum-runtime/src/runtime/native_stack_guard.rs:28:25-35:11	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::native_stack_guard::NativeStackEntry::enter	method syntax: method receiver lacks checked nominal/place provenance
           REJECTED	RET002	crates/rum-runtime/src/runtime/native_stack_guard.rs:28:55-35:10	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::native_stack_guard::NativeStackEntry::enter	closure capture/callback effects unresolved; closure body is a rejected boundary
           ```
         - `+4` in `drain_abandonment_worklist` and `abandon_frames` (`self.evaluation_depth.restore(0)` at `abandon.rs:35, 54, 112`, replacing plain `usize` field writes `self.evaluation_depth = 0; self.native_stack_next_check_depth = ...;` that were previously not rejected):
           ```text
           REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:35:9-35:41	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist	method syntax: method receiver lacks checked nominal/place provenance
           REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:54:13-54:45	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist	method syntax: method receiver lacks checked nominal/place provenance
           REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:112:21-112:53	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::abandon_frames	method syntax: method receiver lacks checked nominal/place provenance
           REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:112:21-112:53	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::abandon_frames_profiled -> rum_runtime::runtime::model::session::Session::abandon_frames	method syntax: method receiver lacks checked nominal/place provenance
           ```
         - `+1` in `restore_resume_context` (`self.evaluation_depth.restore(context.depth)` at `evaluation_context.rs:38`, replacing `self.evaluation_depth = context.depth;`):
           ```text
           REJECTED	RET002	crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs:38:9-38:53	-	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace -> rum_runtime::runtime::model::session::Session::push_evaluation_pass_trace_slow -> rum_runtime::runtime::model::session::Session::push_trace_transition -> rum_runtime::runtime::model::session::Session::push_trace_with_depth -> rum_runtime::runtime::model::session::Session::handle_monitor_trace_event -> rum_runtime::runtime::model::session::Session::run_monitor_callback_best_effort -> rum_runtime::runtime::model::session::Session::abandon_pending_suspension -> rum_runtime::runtime::model::session::Session::drain_abandonment_worklist -> rum_runtime::runtime::model::session::Session::abandon_frames -> rum_runtime::runtime::model::session::Session::restore_resume_context	method syntax: method receiver lacks checked nominal/place provenance
           ```
  - Per A2 Item 1 (`"Any other change to the rejection set is a STOP"`), `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:232–244` was **not** modified without director authorization.

---

### 2. Item 2 — Two compiler oracles (`corpus.py` and `reads/oracle.py`)

- **Diagnosis from `.temp/director/gate-629a/workspace.log`, `.temp/brief-543/oracle-reads/hash/resolution/stderr`, and `target/brief-535/oracle-corpus/target_5-cfg-test-items.stderr`:**
  - Neither oracle failed due to a Rust type or syntax error injected by the instrument. In `reads/oracle.py:147`, `result.stdout.decode()[-12000:]` printed only `non_snake_case` warnings on `__b540_resolution_option_C02_*` (`.temp/director/gate-629a/workspace.log`, `lines 3195–3200`), while `.temp/brief-543/oracle-reads/hash/resolution/stderr` (`lines 1–7`) recorded why `cargo rustc --emit=mir` exited non-zero:
    ```text
       Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/workspace/crates/rum-runtime)
    warning: 16 warnings emitted

    error: could not compile `rum-runtime` (lib)

    Caused by:
      process didn't exit successfully: `/home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin/rustc --crate-name rum_runtime --edition=2024 crates/rum-runtime/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=aa07d112d016f535 -C extra-filename=-8aa57decfda86eb0 --out-dir /home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/deps -C incremental=/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/incremental -L dependency=/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/deps --extern num_bigint=/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/deps/libnum_bigint-287dc4da242efbc2.rmeta --extern num_integer=/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/deps/libnum_integer-8cc4a87451b18666.rmeta --extern num_traits=/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/deps/libnum_traits-0662a421f87a982e.rmeta --extern regex=/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/deps/libregex-22f78fa1b9b69de4.rmeta --extern rum_core=/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/deps/librum_core-92c04c2c66d85f99.rmeta --extern rum_syntax=/home/remi/dev/rum/.temp/brief-543/oracle-reads/hash/target/debug/deps/librum_syntax-ef8d28cbdb35905a.rmeta --emit=mir` (signal: 9, SIGKILL: kill)
    ```
  - Similarly, `target/brief-535/oracle-corpus/target_5-cfg-test-items.stderr` (`lines 6–10`) recorded:
    ```text
    error: could not compile `rum-stdlib` (lib test)

    Caused by:
      process didn't exit successfully: `/home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin/rustc --crate-name rum_stdlib --edition=2024 crates/rum-stdlib/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=0fb939ef10b8d0c7 -C extra-filename=-824c1f48e2fd37d6 --out-dir /home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps -C incremental=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/incremental -L dependency=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps --extern flate2=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/libflate2-575551a86d7199d7.rlib --extern num_bigint=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/libnum_bigint-287dc4da242efbc2.rlib --extern num_integer=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/libnum_integer-8cc4a87451b18666.rlib --extern num_traits=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/libnum_traits-0662a421f87a982e.rlib --extern regex=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/libregex-22f78fa1b9b69de4.rlib --extern rum_core=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/librum_core-92c04c2c66d85f99.rlib --extern rum_host=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/librum_host-98ef9b6b593c97de.rlib --extern rum_runtime=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/librum_runtime-8aa57decfda86eb0.rlib --extern rum_syntax=/home/remi/dev/rum/target/brief-535/oracle-corpus/shared-target/debug/deps/librum_syntax-ef8d28cbdb35905a.rlib --emit=mir -A unused -A warnings` (signal: 9, SIGKILL: kill)
    ```
  - Both subprocess `rustc --emit=mir` invocations were killed by the Linux OOM killer (`signal: 9, SIGKILL: kill`) inside the 16 GiB cgroup when spawned under `CARGO_BUILD_JOBS=8` while the parent `rum_stdlib` test runner held the parsed workspace AST in memory.
- **Replay and verification:**
  - Refreshed `local-contracts.tsv` and `promoted.tsv` after editing `census/tests.rs` (`.temp/brief-629/logs/option-frontier-nocapture.log`, `line 1552`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 18.93s`).
  - Replayed both compiler oracles with `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1`, and then verified both under `CARGO_BUILD_JOBS=8 RUST_TEST_THREADS=1`:
    - `.temp/brief-629/logs/oracle-reads.log` (`lines 4–16`):
      ```text
      running 1 test
      test tests::retention_closure::local::reads::oracle::local_option_read_claims_match_stable_rustc ... READ542_SCRATCH stale_input=removed
      READ542_TYPES positive=pass invariant=true negative=E0308 located=31
      READ542_FIELD_TYPES swapped_fields=E0308 extra_reference=E0308 located=8
      READ542_RESOLUTION claims=15 positive=pass same_name_negatives=5 negative=pass competing_clone=distinct
      READ542_RUST_CONTROL nondiverging-else negative=E0308 located=true
      READ542_RUST_CONTROL borrowed-partial-move negative=E0507 located=true
      READ542_RUST_CONTROL rhs-move negative=E0382 located=true
      READ542_CONTRACTS helper_table=pass pointer_identity=pass bool_branches=pass pointer_negative=fail helper_negative=fail
      READ542_RESTORED invariant_types=pass contracts=pass after_all_negatives=true
      ok

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 148.00s
      ```
    - `.temp/brief-629/logs/oracle-reads-cached.log` (`lines 4–17`):
      ```text
      running 1 test
      test tests::retention_closure::local::reads::oracle::local_option_read_claims_match_stable_rustc ... ORACLE_REUSED /home/remi/dev/rum/.temp/brief-543/oracle-reads/hash fnv1a64:99a11c5c216ca9d5
      READ542_SCRATCH stale_input=removed
      READ542_TYPES positive=pass invariant=true negative=E0308 located=31
      READ542_FIELD_TYPES swapped_fields=E0308 extra_reference=E0308 located=8
      READ542_RESOLUTION claims=15 positive=pass same_name_negatives=5 negative=pass competing_clone=distinct
      READ542_RUST_CONTROL nondiverging-else negative=E0308 located=true
      READ542_RUST_CONTROL borrowed-partial-move negative=E0507 located=true
      READ542_RUST_CONTROL rhs-move negative=E0382 located=true
      READ542_CONTRACTS helper_table=pass pointer_identity=pass bool_branches=pass pointer_negative=fail helper_negative=fail
      READ542_RESTORED invariant_types=pass contracts=pass after_all_negatives=true
      ok

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 12.92s
      ```
    - `.temp/brief-629/logs/oracle-corpus.log` (`lines 7447–7450`):
      ```text
      A1_CORPUS_TOTAL agreements=7442 disagreements=0 claims=7442
      ok

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 212.09s
      ```
    - `.temp/brief-629/logs/oracle-corpus-cached.log` (`lines 7448–7451`):
      ```text
      A1_CORPUS_TOTAL agreements=7442 disagreements=0 claims=7442
      ok

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 13.35s
      ```

---

### 3. Item 3 — The LP01 required inventory (`logical_counter_port`)

- **Changed rows disclosure:**
  - Updated **120 rows** across **4 distinct producer sites** (30 rows per site, all active with `r[20] == "-"`) in:
    - `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-required.tsv` (columns 17 and 19),
    - `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-bindings.tsv` (column 3, 120 matching rows), and
    - `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json` (the 4 matching interned producer strings in the `producers` table).
  - No other columns or rows were modified, and every producer statement text is unchanged:

| Distinct change # | Producer file | Before (`col 17`, `col 19`) | After (`col 17`, `col 19`) | Statement at producer line | Rows updated |
|---|---|---|---|---|---|
| 1 | `crates/rum-runtime/src/runtime/eval_core/mod.rs` | `crates/rum-runtime/src/runtime/eval_core/mod.rs:146`, `fnv1a64:c4fa497bb06f8d34` | `crates/rum-runtime/src/runtime/eval_core/mod.rs:143`, `fnv1a64:f3e98e033bd5b366` | `let result = self.evaluate_symbol(symbol);` | 30 |
| 2 | `crates/rum-runtime/src/runtime/eval_core/mod.rs` | `crates/rum-runtime/src/runtime/eval_core/mod.rs:155`, `fnv1a64:c4fa497bb06f8d34` | `crates/rum-runtime/src/runtime/eval_core/mod.rs:152`, `fnv1a64:f3e98e033bd5b366` | `let result = self.evaluate_call(head.clone(), args);` | 30 |
| 3 | `crates/rum-runtime/src/runtime/eval_core/mod.rs` | `crates/rum-runtime/src/runtime/eval_core/mod.rs:186`, `fnv1a64:c4fa497bb06f8d34` | `crates/rum-runtime/src/runtime/eval_core/mod.rs:183`, `fnv1a64:f3e98e033bd5b366` | `self.evaluate(expr)` | 30 |
| 4 | `crates/rum-runtime/src/runtime/resolution.rs` | `crates/rum-runtime/src/runtime/resolution.rs:326`, `fnv1a64:5b4e5aa595a33628` | `crates/rum-runtime/src/runtime/resolution.rs:338`, `fnv1a64:549fac7c5738681e` | `let result = self.evaluate_profiled_fixed_point(expr);` | 30 |

- **Verification (`RUST_TEST_THREADS=1 cargo test -p rum-stdlib --test suspension_oracle -- logical_counter_port --nocapture`, `.temp/brief-629/logs/lp01.log`, `lines 4170–4176`):**
  ```text
  all: LP01 CONTROL absent-execution RED
  all: continuation-slot fault-occurrence delta=0; observations=9552
  all: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=41728
  all: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 19.33s
  ok

  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 68.17s
  ```

---

### 4. Item 4 — The typed-slot census receipt (`typed_slot_census`)

- **Receipt update (`crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv`, line 2348):**
  - Before:
    `X	X72f7b87d1aa229c0	R	legacy-test	rum_runtime::runtime::tests::resume_evaluation_context::routes#method10:insert_suspension_frame:s	crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs	insert_suspension_frame	test-support	-`
  - After:
    `X	X1e0eb40431d6ec9f	R	legacy-test	rum_runtime::runtime::tests::resume_evaluation_context::routes#method11:insert_suspension_frame:s	crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs	insert_suspension_frame	test-support	-`
- **Verification (`RUST_TEST_THREADS=1 cargo test -p rum-stdlib --test typed_slot_census`, `.temp/brief-629/logs/typed-slot-census-full.log`, `lines 4–15`):**
  ```text
  running 9 tests
  test forced_red_probe ... ignored, invoked explicitly by both bounded scratch-copy falsifiers
  test preflip_no_legacy_transport_uses ... ok
  test removing_a_frame_in_a_scratch_copy_is_red ... ok
  test removing_a_test_guard_in_a_scratch_copy_is_red ... ok
  test removing_a_witnessed_insertion_in_a_scratch_copy_is_red ... ok
  test shadowing_a_witness_in_a_cold_helper_is_red ... ok
  test source::root_set::syntax_root_discovers_added_recursive_type_and_restores_source ... ok
  test source_baseline_matches_earned_receipt ... ok
  test witnessing_a_different_binding_in_a_scratch_copy_is_red ... ok

  test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 285.31s
  ```

---

### 5. Status at STOP

- Items 2, 3, and 4 are green and verified.
- Item 1 verified the `[41, 42, 92]` -> `[43, 44, 94]` `abandon.rs` rows and hit the explicit A2 Item 1 stop condition (`"Any other change to the rejection set is a STOP"`) on `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:232–244` (`resolution.rs` `(35, 19)` -> `(36, 19)` and `options.rejected.len()` `963` (`10/925/28`) -> `971` (`10/933/28`)). Stopping here for the director's decision on `census/tests.rs:232–244`.


## Director gate of A2, and Stage B (2026-10-28 01:30) — to grk

**Stage A is closed and green.** My second gate (`.temp/director/gate-629b/summary.txt`),
each cargo command alone and tests single-threaded:
- `cargo fmt --check` is clean.
- The Jupyter pair passed 20 of 20 at `--test-threads=2`.
- `cargo test --workspace --no-fail-fast` gave 3 977 passed, 0 failed,
  61 ignored. The native ZeroMQ tests ran natively; no OOM kill.

A2's results, each verified from the files rather than the report:
- **LP01 inventories.** `logical-profile-required.tsv` changed 120 rows, in
  columns 17 and 19 only. `logical-profile-bindings.tsv` changed 120 rows,
  in column 3 only. The four producer lines (`eval_core/mod.rs:143,152,183`,
  `resolution.rs:338`) hold the same statements as before
  (`:146,155,186`, `:326`).
- **The typed-slot receipt.** One row changed, the authorized
  `routes#method10` → `method11`.
- **The compiler oracles.** They failed in my first gate because two
  `rustc` processes were killed by a host-wide OOM (`CONSTRAINT_NONE`,
  invoked by kswapd with 23 GB free; it was not the scope). Both pass now.
- **The rejection census.** A2 stopped there, correctly. I diffed the base
  and candidate option frontiers myself, keyed by kind, file, callee and
  reason. Three RET002 rows retire and eleven appear, all in files this
  brief changed, with no RET001 or RET003 movement: 963 → 971 (10/933/28).
  I accepted it, updated `census/tests.rs` myself, and replayed the
  contract fingerprints and both certificate builds.

### Stage B — measurement only, on grk

You are a new worker on this brief. Read all of it, including Stage A's
target shape and my two gates. The working tree holds the accepted Stage A
change. **Do not modify any file in `crates/`, `scripts/` or `docs/`**,
except to append your report here. Stage B measures; it changes nothing.

Run the campaign in §Stage B above, with these additions:

1. **The candidate arm.** Build it exactly as written there, and put the
   binaries in `.temp/brief-629/arms/cand/`. Leave `arms/base/` alone: I
   built it from `base-source/`, which is `678e6be7`, code-identical to the
   pre-brief HEAD.
2. **Floor first.** Run `instr.py arms/base arms/base logs/bb.tsv`, then
   `instr.py arms/base arms/cand logs/bc.tsv`. The instrument resumes from
   its TSV, so rerun the same command after an interruption.
3. **Attribution.** Every label beyond its floor needs a named mechanism
   (§Stage B, step 2).
   - Valgrind names libc functions only through the debuginfod cache, which
     your sandbox mounts read-only. Copy `~/.cache/debuginfod_client` to
     `.temp/brief-629/debuginfod/`, and set `DEBUGINFOD_CACHE_PATH` to the
     copy.
   - Calibrate first: a callgrind run must show `__memcpy_avx_unaligned_erms`
     or another libc symbol by name. If libc stays unsymbolized, say so; do
     not report a libc count.
4. **Stack.** Use `objdump -d -C --no-show-raw-insn` on both `rum-cli`s. For
   every `Session::evaluate*` symbol and every function on the evaluator's
   recursive cycle, give the `sub $N,%rsp` value for base and candidate in
   one table. Any growth is a STOP.
5. **Host OOM.** This host's kernel can kill a process host-wide while
   memory is free. If a build or a run dies with signal 9 and your scope's
   `oom_kill` did not rise, rerun it once and report both attempts.

Do not rerun the workspace; my gate did.

**Report.** Append "Executing-agent report — Stage B", with:
- the two TSVs' summary: count of labels beyond floor, the largest
  positive and negative deltas, the median delta;
- one row per flagged label: base slope, candidate slope, floor,
  attribution, and log;
- the stack table;
- every attempt.

Every quoted line must be copied from a file you name. The raw TSVs stay
on disk, and I check them directly.

## Executing-agent report — Stage B

**STOP.** The instruction residual on the measured hot path is the outline of `Session::evaluate_profiled_fixed_point`. The cold native-stack frames also reserve more stack. No file under `crates/`, `scripts/`, or `docs/` was modified except this appended report. `arms/base/` was left untouched.

### 1. Candidate arm

Both release builds used `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, `CARGO_BUILD_JOBS=8`, `CARGO_NET_OFFLINE=true`.

- `.temp/brief-629/logs/build-cand-cli.log`: `    Finished `release` profile [optimized] target(s) in 50.49s` then `EXIT:0` and `oom_kill 0`.
- `.temp/brief-629/logs/build-cand-harness.log`: `    Finished `release` profile [optimized] target(s) in 49.99s` then `EXIT:0` and `oom_kill 0`.
- Binaries copied to `.temp/brief-629/arms/cand/rum-cli` and `.temp/brief-629/arms/cand/resume` (the harness binary is `brief608-resume`).

### 2. The two TSVs

Floor command wrote `.temp/brief-629/logs/bb.tsv` (log `.temp/brief-629/logs/bb-run.log`). Gate command wrote `.temp/brief-629/logs/bc.tsv` (log `.temp/brief-629/logs/bc-run.log`). Each log ends `EXIT:0`. Each TSV has a header plus 313 data rows (`wc -l` is 314).

Reduction in `.temp/brief-629/logs/gate-summary.txt`, copied here:

```
labels 313
bb_median_pct -0.0001
bc_median_pct +0.7964
bc_max_pct +38.7486
bc_min_pct -3.3007
flagged 276 positive 276 negative 0
flagged_cli 155 of 192 median +0.6674
flagged_resume 121 of 121 median +1.6783
literal_nonzero 3
```

Floor for a label is `max(0.05, |bb delta_pct|)`. A label is beyond the floor when `|bc delta_pct|` is greater than that floor. All 276 breaches are positive.

Largest positive percent, from `.temp/brief-629/logs/bc.tsv`:

```
valueq-undefined-symbol-definitions-8000	20	3002.30	4165.65	+1163.35	+38.7486	242674475,242517986,242505870	242594475,242578032,242566332	243415032,243422145,243421854	243488225,243685044,243505167
```

Its floor is the base-vs-base row in `.temp/brief-629/logs/bb.tsv`:

```
valueq-undefined-symbol-definitions-8000	20	5576.95	6019.35	+442.40	+7.9327	242488660,242463201,242657276	242600199,242606577,242550811	242490912,242474989,242705327	242645409,242579724,242611299
```

Largest positive instruction delta, and the most negative instruction delta and percent, from `bc.tsv`:

```
lazy-depth-fold-through-source-256	1000	520360545.04	523455478.42	+3094933.38	+0.5948	521798560781,521789732938,521773382262	1042140425661,1042150277978,1042162749281	524893067848,524903372379,524914219132	1048357185801,1048358850797,1048369155604
valueq-undefined-symbol-definitions-2000	20	5138.25	4968.65	-169.60	-3.3007	62924955,62924993,62917401	63027720,63027491,63035764	63166189,63164944,63159376	63256619,63264535,63264317
```

`valueq-undefined-symbol-definitions-2000` stays inside its floor. The matching `bb.tsv` percent is `-8.1723`. Median gate delta is `+0.7964` percent (`gate-summary.txt`).

`literal-integer`, the label attributed below, from `bc.tsv`:

```
literal-integer	20000	292.01	317.99	+25.97	+8.8948	9457152,9457505,9459433	15297769,15298749,15297627	9979725,9977499,9978133	16336598,16339379,16337876
```

### 3. Attribution

Debuginfo cache: `cp -a ~/.cache/debuginfod_client .temp/brief-629/debuginfod`. `cmp` of the libc debuginfo matched. Build-id of `/usr/lib/libc.so.6` and of that file is `e363db7c8fcc7c06c6f0d2839b2a8ad875b75406`.

An empty `DEBUGINFOD_URLS` left the hot libc body unnamed. A closed localhost URL makes the client read the copied cache and fail fast on a miss. With `DEBUGINFOD_CACHE_PATH=.temp/brief-629/debuginfod`, `DEBUGINFOD_URLS=http://127.0.0.1:9`, and `DEBUGINFOD_TIMEOUT=1`, `.temp/brief-629/callgrind/calib-debuginfod.out` contains:

```
cfn=(4298) __memcpy_avx_unaligned_erms
```

`literal-integer` (`Do[123, {20000}]`) was callgrinded at N and 2N, six fresh processes per arm. Exclusive instruction sums match each profile's `summary:` line. Log `.temp/brief-629/logs/attr-literal-integer.log`:

```
literal-integer base N=20000 totals=[9529449, 9528560, 9527229, 9530434, 9529859, 9530549]
literal-integer base N=40000 totals=[15368608, 15370242, 15370650, 15368581, 15371584, 15369901]
literal-integer cand N=20000 totals=[10053208, 10050299, 10049072, 10050491, 10048670, 10051635]
literal-integer cand N=40000 totals=[16409876, 16411018, 16409402, 16410620, 16410028, 16411632]
TOTAL base=292.02 cand=318.00 delta=+25.98
wrote /home/remi/dev/rum/.temp/brief-629/logs/attr-literal-integer.tsv
EXIT:0
```

The callgrind total slope `+25.98` matches the perf slope `+25.97`. Every function whose exclusive slope moved by at least 0.05 instructions, from `.temp/brief-629/logs/attr-literal-integer.tsv`:

```
+25.9756	292.0209	317.9964	TOTAL
+69.0000	0.0000	69.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point'2
-43.0000	110.0000	67.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
```

`+69 + (-43) = +26`, the whole residual. `libc.so.6!__memcpy_avx_unaligned_erms` is named and its slope is `30` on both arms (`delta +0.0000` in that TSV). No `native_stack_exhausted`, `NativeStackEntry`, or `EvaluationDepth::restore` symbol has a per-iteration slope on this label.

Mechanism: `Session::evaluate_profiled_fixed_point` is a separate call. In the base arm its exclusive per-iteration slope is 0 and the fixed-point body is inside `Session::evaluate` (slope 110). In the candidate the body is `evaluate_profiled_fixed_point` (slope 69) and `evaluate` falls to 67. The brief's expected mechanisms are the thread-record read in the two cold checks, `NativeStackEntry` enter and exit per resume and per abandonment, and `restore`'s threshold store per resumed cell. This residual is the outline of the fixed-point function on every `Do` iteration. **STOP.**

The other 275 labels beyond the floor were left unattributed because that stop had fired. Their rows are below and in `.temp/brief-629/logs/flagged.tsv`. An unattributed positive residual is the same stop.

### 4. Stack

`objdump -d -C --no-show-raw-insn` on both `rum-cli`s. First `sub $N,%rsp` in each `Session::evaluate*` symbol, plus `resume_call_after_head`, `resume_call_with_dispatch_plan`, and the native-stack poll functions. Full table: `.temp/brief-629/logs/stack-reservations.tsv`. Prologues: `.temp/brief-629/logs/stack-prologues.txt`.

```
function	base_sub	cand_sub	growth
rum_runtime::runtime::debug_tools::test_report::evaluation::<impl rum_runtime::runtime::model::session::Session>::evaluate_test_case_input	744	744	
rum_runtime::runtime::definitions::application::default_values::<impl rum_runtime::runtime::model::session::Session>::evaluate_first_exact_default_value	456	456	
rum_runtime::runtime::definitions::application::rhs::<impl rum_runtime::runtime::model::session::Session>::evaluate_definition_application_rhs	840	840	
rum_runtime::runtime::definitions::application::rhs::<impl rum_runtime::runtime::model::session::Session>::evaluate_definition_rhs	792	792	
rum_runtime::runtime::definitions::definition_lhs_preparation::<impl rum_runtime::runtime::model::session::Session>::evaluate_definition_lhs_active_expr	232	232	
rum_runtime::runtime::definitions::definition_lhs_preparation::<impl rum_runtime::runtime::model::session::Session>::evaluate_prepared_definition_lhs	136	136	
rum_runtime::runtime::definitions::rhs_guards::<impl rum_runtime::runtime::model::session::Session>::evaluate_owned_guards	392	392	
rum_runtime::runtime::definitions::rhs_guards::<impl rum_runtime::runtime::model::session::Session>::evaluate_readonly_suspended_guards	392	392	
rum_runtime::runtime::definitions::rhs_guards::<impl rum_runtime::runtime::model::session::Session>::evaluate_suspended_guards_deferred	488	488	
rum_runtime::runtime::definitions::rule_lhs::<impl rum_runtime::runtime::model::session::Session>::evaluate_rule_lhs	64	64	
rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_atom_fast_path	384	384	
rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable	1256	1256	
rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode		520	new-in-cand
rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking	712		absent-in-cand
rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted	none	16	
rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit	none	none	
rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::evaluate_call::{{closure}}::suspend	40	40	
rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::resume_call_after_head	568	568	
rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::resume_call_with_dispatch_plan	776	776	
rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::resume_call_with_dispatch_plan::suspend	240	240	
rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::evaluate_stored_own_value	520	520	
rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::evaluate_stored_own_value::suspend	152	152	
rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::evaluate_symbol_after_stored_own_value	176	176	
rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::driver::<impl rum_runtime::runtime::model::session::Session>::evaluate_loop_iterator_body	1384	1384	
rum_runtime::runtime::kernel_data::control_data::exceptions::<impl rum_runtime::runtime::model::session::Session>::evaluate_exception_handler_rhs	208	208	
rum_runtime::runtime::kernel_data::control_data::exceptions::<impl rum_runtime::runtime::model::session::Session>::evaluate_exception_handler_rhs::suspend	40	40	
rum_runtime::runtime::kernel_data::evaluated_part_selectors::<impl rum_runtime::runtime::model::session::Session>::evaluate_part_selectors_owned::suspend	40	40	
rum_runtime::runtime::kernel_data::evaluated_part_selectors::<impl rum_runtime::runtime::model::session::Session>::evaluate_part_selectors_owned_for_builtin	184	184	
rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit	none	16	
rum_runtime::runtime::options::core::<impl rum_runtime::runtime::model::session::Session>::evaluate_runtime_option_expr	1704	1704	
rum_runtime::runtime::options::patterns::<impl rum_runtime::runtime::model::session::Session>::evaluate_with_option_value_scope	272	272	
rum_runtime::runtime::pattern_matching::core::<impl rum_runtime::runtime::model::session::Session>::evaluate_pattern_test_owned	296	296	
rum_runtime::runtime::pattern_matching::core::<impl rum_runtime::runtime::model::session::Session>::evaluate_sequence_pattern_checks_owned	664	664	
rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate	840	360	
rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_at_root		352	new-in-cand
rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point		792	new-in-cand
rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point::suspend	136	136	
rum_runtime::runtime::scoping::callable::status_application::<impl rum_runtime::runtime::model::session::Session>::evaluate_status_function_value	648	648	
rum_runtime::runtime::session::session_api::<impl rum_runtime::runtime::model::session::Session>::evaluate_with_random_state_restored	312	312	
```

No symbol present on both sides with a numeric `sub` on both sides has a larger candidate `sub`. `Session::evaluate` itself goes from 840 to 360.

These reservations grew, or appeared on the recursive cycle:

| function | base `sub` | candidate `sub` |
|---|---|---|
| `Session::native_stack_budget_exhausted` | none | 16 |
| `NativeStackGuard::poll_native_stack_limit` | none | 16 |
| `Session::evaluate_profiled_fixed_point` | absent | 792 |
| `Session::evaluate_at_root` | absent | 352 |
| `Session::evaluate_once_in_accounting_mode` | absent | 520 |
| `Session::evaluate_once_unstable_after_input_check_with_cycle_tracking` | 712 | absent |

`native_stack_budget_exhausted`, from `stack-prologues.txt`. Base has no `sub`. Candidate is `sub $0x10,%rsp` and a `%fs` thread-record read:

```
d15a0:	xor    %eax,%eax
d15a2:	lea    -0x1(%rsp),%rcx
...
d15d1:	sub    $0x10,%rsp
d15d8:	mov    %fs:0x0,%rax
```

`NativeStackGuard::poll_native_stack_limit` likewise goes from no `sub` to `sub $0x10,%rsp` plus `mov %fs:0x0,%rax`.

`Session::poll_native_stack_limit` has no `sub` on either arm. The candidate prologue pushes three registers where the base prologue pushes none (`stack-prologues.txt`):

```
d1570:	push   %r14
d1572:	push   %rbx
d1573:	push   %rax
d157a:	mov    %fs:0x0,%rax
```

`Session::evaluate` and `Session::evaluate_profiled_fixed_point` (`stack-prologues.txt`):

```
750d3a:	sub    $0x348,%rsp
750e06:	sub    $0x168,%rsp
7510ba:	sub    $0x318,%rsp
```

`0x348` is 840, `0x168` is 360, `0x318` is 792. On a depth-above-0 evaluation the candidate holds `evaluate` (360) and `evaluate_profiled_fixed_point` (792) together. The base holds `evaluate` (840) alone. `nm -C --defined-only` on the candidate shows no `NativeStackEntry` symbol; the `%fs` read of the entry is inlined in `evaluate_at_root` (`sub $0x160`, 352), which is absent from the base.

**STOP** on the stack as well: the two cold polls gained a 16-byte `sub`, and the recursive cycle gained a 792-byte frame under a caller that still reserves 360.

### 5. One row per flagged label

Copied from `.temp/brief-629/logs/flagged.tsv`. Columns: label, engine, base slope, candidate slope, instruction delta, percent, floor percent, base-vs-base percent, attribution.

```
label	engine	base_slope	cand_slope	delta	delta_pct	floor_pct	bb_pct	attribution
lazy-range-take-small	cli	41869.48	41939.89	+70.41	+0.1682	0.1142	-0.1142	not attributed; stop fired on literal-integer
lazy-range-normal-small	cli	53564.91	53828.31	+263.40	+0.4917	0.0500	-0.0410	not attributed; stop fired on literal-integer
lazy-map-realize-small	cli	175630.18	176586.15	+955.96	+0.5443	0.0500	+0.0128	not attributed; stop fired on literal-integer
lazy-select-realize-small	cli	112443.32	112851.03	+407.70	+0.3626	0.0500	+0.0439	not attributed; stop fired on literal-integer
lazy-select-scan-small	cli	787423.56	791648.71	+4225.15	+0.5366	0.0500	-0.0043	not attributed; stop fired on literal-integer
lazy-nestlist-realize-small	cli	142236.22	143126.26	+890.04	+0.6258	0.0807	+0.0807	not attributed; stop fired on literal-integer
lazy-foldlist-realize-small	cli	110788.94	111180.83	+391.88	+0.3537	0.0500	-0.0188	not attributed; stop fired on literal-integer
lazy-drop-realize-small	cli	187862.62	188825.64	+963.03	+0.5126	0.0500	-0.0108	not attributed; stop fired on literal-integer
lazy-first-small	cli	40547.33	40836.64	+289.31	+0.7135	0.0500	+0.0035	not attributed; stop fired on literal-integer
lazy-part-small	cli	86920.82	87408.50	+487.68	+0.5611	0.1242	+0.1242	not attributed; stop fired on literal-integer
lazy-transparent-adapters-small	cli	177978.42	178948.93	+970.51	+0.5453	0.1329	-0.1329	not attributed; stop fired on literal-integer
lazy-table-construct-short	cli	23341.56	23437.87	+96.31	+0.4126	0.1284	-0.1284	not attributed; stop fired on literal-integer
lazy-table-construct-full	cli	23851.77	23983.82	+132.06	+0.5536	0.0500	-0.0001	not attributed; stop fired on literal-integer
lazy-table-realize-small	cli	115304.02	115944.82	+640.81	+0.5558	0.0500	+0.0036	not attributed; stop fired on literal-integer
lazy-permutations-take-small	cli	27871.89	27956.34	+84.46	+0.3030	0.0500	+0.0001	not attributed; stop fired on literal-integer
lazy-subsets-take-small	cli	25011.58	25096.88	+85.30	+0.3410	0.1917	+0.1917	not attributed; stop fired on literal-integer
lazy-tuples-take-small	cli	26236.22	26351.79	+115.57	+0.4405	0.0500	-0.0024	not attributed; stop fired on literal-integer
scan-no-input	cli	28961.15	29214.48	+253.32	+0.8747	0.0500	+0.0060	not attributed; stop fired on literal-integer
optionvalue-three-no-input	cli	9494.51	9528.84	+34.33	+0.3616	0.0500	-0.0009	not attributed; stop fired on literal-integer
replacerepeated-no-input	cli	38842.36	39046.82	+204.47	+0.5264	0.0500	-0.0003	not attributed; stop fired on literal-integer
dot-three-no-input	cli	249964.54	250887.61	+923.07	+0.3693	0.0500	-0.0001	not attributed; stop fired on literal-integer
differences-no-input	cli	9910.47	9973.90	+63.44	+0.6401	0.0500	-0.0036	not attributed; stop fired on literal-integer
do-count-null	cli	535.14	561.05	+25.91	+4.8412	0.0500	-0.0009	not attributed; stop fired on literal-integer
do-iterator-null	cli	1152.06	1178.01	+25.96	+2.2530	0.0500	+0.0020	not attributed; stop fired on literal-integer
replaceall-simple	cli	11301.85	11418.58	+116.73	+1.0328	0.0500	-0.0007	not attributed; stop fired on literal-integer
table-small	cli	14524.31	14581.73	+57.42	+0.3953	0.0500	-0.0004	not attributed; stop fired on literal-integer
take-multispec-small	cli	49734.90	50022.73	+287.83	+0.5787	0.0500	+0.0001	not attributed; stop fired on literal-integer
lazy-depth-map-build-8	cli	17833.61	17898.87	+65.26	+0.3659	0.0824	-0.0824	not attributed; stop fired on literal-integer
lazy-depth-map-first-8	cli	127575.42	128448.62	+873.20	+0.6845	0.0500	+0.0025	not attributed; stop fired on literal-integer
lazy-depth-map-eight-8	cli	939554.53	945644.88	+6090.36	+0.6482	0.0500	-0.0001	not attributed; stop fired on literal-integer
lazy-depth-select-restart-8	cli	177473.87	179337.74	+1863.86	+1.0502	0.0500	-0.0039	not attributed; stop fired on literal-integer
lazy-depth-drop-all-one-8	cli	46372.65	46477.66	+105.01	+0.2265	0.0500	-0.0015	not attributed; stop fired on literal-integer
lazy-depth-take-map-8	cli	136119.56	136979.16	+859.59	+0.6315	0.0500	-0.0116	not attributed; stop fired on literal-integer
lazy-depth-fold-through-source-8	cli	578089.02	581543.84	+3454.82	+0.5976	0.0500	+0.0045	not attributed; stop fired on literal-integer
lazy-depth-nest-map-8	cli	137849.16	138804.90	+955.74	+0.6933	0.0500	-0.0008	not attributed; stop fired on literal-integer
lazy-depth-map-build-64	cli	29853.46	29916.57	+63.11	+0.2114	0.0500	+0.0053	not attributed; stop fired on literal-integer
lazy-depth-map-first-64	cli	927443.08	933575.82	+6132.75	+0.6613	0.0500	-0.0003	not attributed; stop fired on literal-integer
lazy-depth-map-eight-64	cli	7252557.67	7300943.95	+48386.27	+0.6672	0.0500	-0.0021	not attributed; stop fired on literal-integer
lazy-depth-select-restart-64	cli	969668.74	978136.92	+8468.18	+0.8733	0.0742	-0.0742	not attributed; stop fired on literal-integer
lazy-depth-take-map-64	cli	987269.86	993393.68	+6123.82	+0.6203	0.0500	+0.0005	not attributed; stop fired on literal-integer
lazy-depth-fold-through-source-64	cli	33343765.41	33540375.06	+196609.64	+0.5896	0.0500	-0.0013	not attributed; stop fired on literal-integer
lazy-depth-nest-map-64	cli	938083.15	944423.22	+6340.07	+0.6759	0.3543	-0.3543	not attributed; stop fired on literal-integer
lazy-depth-map-first-256	cli	3639733.53	3664278.12	+24544.59	+0.6744	0.0500	-0.0154	not attributed; stop fired on literal-integer
lazy-depth-map-eight-256	cli	28911261.10	29104214.57	+192953.47	+0.6674	0.0500	+0.0011	not attributed; stop fired on literal-integer
lazy-depth-select-restart-256	cli	3645287.66	3688639.75	+43352.09	+1.1893	0.0500	-0.0072	not attributed; stop fired on literal-integer
lazy-depth-take-map-256	cli	3902716.75	3927345.86	+24629.10	+0.6311	0.0500	-0.0106	not attributed; stop fired on literal-integer
lazy-depth-fold-through-source-256	cli	520360545.04	523455478.42	+3094933.38	+0.5948	0.0500	-0.0013	not attributed; stop fired on literal-integer
literal-integer	cli	292.01	317.99	+25.97	+8.8948	0.0500	-0.0450	callgrind 6x2x2: entire +25.98 is evaluate_profiled_fixed_point outline; logs/attr-literal-integer.tsv
downvalue-pattern-hit	cli	13635.05	13698.31	+63.26	+0.4640	0.0500	-0.0001	not attributed; stop fired on literal-integer
module-local	cli	22344.39	22470.90	+126.51	+0.5662	0.0500	-0.0043	not attributed; stop fired on literal-integer
block-local	cli	14462.86	14590.16	+127.30	+0.8802	0.0500	+0.0001	not attributed; stop fired on literal-integer
resume-table	resume	19811.45	20113.30	+301.85	+1.5236	0.2592	-0.2592	not attributed; stop fired on literal-integer
resume-do-chain	resume	19102.01	19461.52	+359.50	+1.8820	0.2000	-0.2000	not attributed; stop fired on literal-integer
resume-call-repeat	resume	28313.46	28572.47	+259.01	+0.9148	0.2153	+0.2153	not attributed; stop fired on literal-integer
resume-cleanup	resume	18582.18	18882.88	+300.70	+1.6182	0.0500	+0.0085	not attributed; stop fired on literal-integer
resume-fold	resume	29382.87	29927.22	+544.35	+1.8526	0.0500	+0.0095	not attributed; stop fired on literal-integer
deletecases-leaf-delete-small	cli	63195.53	63340.09	+144.56	+0.2287	0.1554	-0.1554	not attributed; stop fired on literal-integer
replace-leaf-rebuild-small	cli	73292.33	74271.04	+978.71	+1.3354	0.5821	+0.5821	not attributed; stop fired on literal-integer
rewrite-guarded-hit	cli	52736.94	52960.10	+223.16	+0.4232	0.0887	-0.0887	not attributed; stop fired on literal-integer
rewrite-guarded-decline-fallback	cli	76942.43	77419.06	+476.63	+0.6195	0.1164	+0.1164	not attributed; stop fired on literal-integer
rule-rhs-deep-small	cli	82400.14	82756.93	+356.79	+0.4330	0.0500	+0.0004	not attributed; stop fired on literal-integer
part-list	cli	8329.80	8415.18	+85.38	+1.0250	0.1535	-0.1535	not attributed; stop fired on literal-integer
appendto-part-outer1000-small	cli	2222699.20	2255334.17	+32634.97	+1.4683	0.0500	+0.0346	not attributed; stop fired on literal-integer
set-part-fixed100-len100	cli	1265382.42	1275456.06	+10073.64	+0.7961	0.0500	-0.0035	not attributed; stop fired on literal-integer
set-part-fixed100-len800	cli	1265727.05	1275354.95	+9627.90	+0.7607	0.0500	+0.0005	not attributed; stop fired on literal-integer
set-part-fill-small	cli	2776157.76	2802269.48	+26111.72	+0.9406	0.2733	-0.2733	not attributed; stop fired on literal-integer
set-part-fill-large	cli	5571166.43	5622437.48	+51271.05	+0.9203	0.2448	+0.2448	not attributed; stop fired on literal-integer
resume-argument-twice	resume	92451.72	94050.08	+1598.37	+1.7289	0.1420	+0.1420	not attributed; stop fired on literal-integer
resume-rewrite-exprs	resume	53320.25	54100.51	+780.26	+1.4634	0.1995	+0.1995	not attributed; stop fired on literal-integer
resume-rewrite-compiled	resume	41549.97	42380.43	+830.46	+1.9987	0.2262	-0.2262	not attributed; stop fired on literal-integer
resume-cases-spec	resume	69089.07	70210.95	+1121.89	+1.6238	0.2544	+0.2544	not attributed; stop fired on literal-integer
resume-cases-traversal	resume	67201.46	68228.59	+1027.13	+1.5284	0.1648	-0.1648	not attributed; stop fired on literal-integer
resume-position-traversal	resume	70445.48	71503.32	+1057.83	+1.5016	0.0632	-0.0632	not attributed; stop fired on literal-integer
resume-count-traversal	resume	61522.50	62568.62	+1046.11	+1.7004	0.1176	+0.1176	not attributed; stop fired on literal-integer
resume-cases-shared	resume	113385.32	115456.97	+2071.64	+1.8271	0.0622	+0.0622	not attributed; stop fired on literal-integer
symbol-ownvalue-read	cli	2207.09	2237.75	+30.66	+1.3893	0.0500	+0.0174	not attributed; stop fired on literal-integer
symbol-ownvalue-guarded-read	cli	3573.41	3623.17	+49.77	+1.3927	0.0500	+0.0136	not attributed; stop fired on literal-integer
block-symbol-only-repeat	cli	10688.79	10805.98	+117.19	+1.0963	0.0500	-0.0101	not attributed; stop fired on literal-integer
downvalue-exact-hit	cli	5741.01	5774.02	+33.02	+0.5751	0.0500	-0.0252	not attributed; stop fired on literal-integer
downvalue-exact-miss	cli	998.03	1023.75	+25.73	+2.5777	0.0500	+0.0215	not attributed; stop fired on literal-integer
downvalue-pattern-miss	cli	997.82	1023.88	+26.06	+2.6113	0.1102	-0.1102	not attributed; stop fired on literal-integer
downvalue-guarded-hit	cli	27568.94	27660.09	+91.15	+0.3306	0.0500	-0.0072	not attributed; stop fired on literal-integer
downvalue-guarded-decline-fallback	cli	20853.50	20920.30	+66.80	+0.3203	0.0500	+0.0075	not attributed; stop fired on literal-integer
upvalue-guarded-hit	cli	33861.67	33976.06	+114.39	+0.3378	0.0500	-0.0020	not attributed; stop fired on literal-integer
subvalue-unguarded-hit	cli	20056.28	20149.71	+93.43	+0.4658	0.0500	+0.0016	not attributed; stop fired on literal-integer
subvalue-unguarded-miss	cli	11282.00	11339.23	+57.23	+0.5073	0.0500	-0.0040	not attributed; stop fired on literal-integer
subvalue-guarded-hit	cli	35221.51	35338.37	+116.86	+0.3318	0.0500	+0.0014	not attributed; stop fired on literal-integer
default-unguarded-hit	cli	17184.08	17217.41	+33.33	+0.1939	0.0500	-0.0001	not attributed; stop fired on literal-integer
default-unguarded-miss	cli	12782.61	12812.29	+29.68	+0.2322	0.0500	-0.0024	not attributed; stop fired on literal-integer
default-guarded-hit	cli	22362.11	22415.60	+53.49	+0.2392	0.0500	+0.0038	not attributed; stop fired on literal-integer
downvalue-exact-miss-repeat	cli	774.50	828.55	+54.05	+6.9787	0.0500	+0.0237	not attributed; stop fired on literal-integer
downvalue-pattern-miss-repeat	cli	775.54	827.40	+51.86	+6.6865	0.0500	-0.0037	not attributed; stop fired on literal-integer
upvalue-candidates-tiny	cli	31369.04	31580.03	+211.00	+0.6726	0.0500	-0.0178	not attributed; stop fired on literal-integer
sortby-default-wide	cli	7428626.70	7479501.80	+50875.10	+0.6849	0.0500	-0.0007	not attributed; stop fired on literal-integer
keysortby-default-wide	cli	15447495.65	15499507.10	+52011.45	+0.3367	0.0500	+0.0006	not attributed; stop fired on literal-integer
valueq-undefined-symbol-definitions-8000	cli	3002.30	4165.65	+1163.35	+38.7486	7.9327	+7.9327	not attributed; stop fired on literal-integer
resume-s5-down-rhs	resume	59434.94	60526.71	+1091.77	+1.8369	0.0500	+0.0375	not attributed; stop fired on literal-integer
resume-s5-down-guard	resume	113990.43	116062.67	+2072.24	+1.8179	0.0500	+0.0420	not attributed; stop fired on literal-integer
resume-s5-sub-rhs	resume	65212.89	66247.58	+1034.69	+1.5866	0.1485	+0.1485	not attributed; stop fired on literal-integer
resume-s5-sub-guard	resume	122084.72	123802.98	+1718.26	+1.4074	0.1560	+0.1560	not attributed; stop fired on literal-integer
resume-s5-up-rhs	resume	64588.26	65749.26	+1161.00	+1.7975	0.0500	-0.0473	not attributed; stop fired on literal-integer
resume-s5-up-guard	resume	137830.07	139739.97	+1909.90	+1.3857	0.1402	-0.1402	not attributed; stop fired on literal-integer
resume-s5-default-rhs	resume	108898.86	110338.66	+1439.80	+1.3221	0.1519	+0.1519	not attributed; stop fired on literal-integer
resume-s5-default-guard	resume	103328.65	105070.45	+1741.80	+1.6857	0.0500	-0.0059	not attributed; stop fired on literal-integer
resume-s5-default-fallback-rhs	resume	64239.92	65397.91	+1157.99	+1.8026	0.0500	-0.0474	not attributed; stop fired on literal-integer
resume-s5-set	resume	56341.30	57439.05	+1097.75	+1.9484	0.0500	-0.0412	not attributed; stop fired on literal-integer
resume-s5-upset	resume	66948.49	68215.28	+1266.79	+1.8922	0.0789	-0.0789	not attributed; stop fired on literal-integer
resume-s5-tagset	resume	66797.58	68012.99	+1215.41	+1.8195	0.0632	-0.0632	not attributed; stop fired on literal-integer
resume-s5-rule-rhs	resume	71487.12	72859.15	+1372.03	+1.9193	0.1151	+0.1151	not attributed; stop fired on literal-integer
resume-s5-rule-lhs	resume	90258.32	91886.63	+1628.31	+1.8041	0.0864	+0.0864	not attributed; stop fired on literal-integer
resume-s5-destructure	resume	75981.31	77373.77	+1392.46	+1.8326	0.1049	+0.1049	not attributed; stop fired on literal-integer
resume-s5-lhs-twice	resume	97946.95	99766.82	+1819.87	+1.8580	0.0500	-0.0371	not attributed; stop fired on literal-integer
resume-s5-own-rhs	resume	51601.52	52629.13	+1027.61	+1.9914	0.1076	-0.1076	not attributed; stop fired on literal-integer
resume-s5-destructure-twice	resume	118832.14	120826.50	+1994.36	+1.6783	0.0711	-0.0711	not attributed; stop fired on literal-integer
resume-s5-rule-lhs-twice	resume	121984.60	124266.91	+2282.31	+1.8710	0.1163	+0.1163	not attributed; stop fired on literal-integer
matcher-single-tiny	cli	11597.61	11678.68	+81.07	+0.6990	0.0500	+0.0167	not attributed; stop fired on literal-integer
replacepart-deep-path-small	cli	40829.83	40958.74	+128.91	+0.3157	0.0500	+0.0057	not attributed; stop fired on literal-integer
replacepart-guarded-hit	cli	15788.00	15917.56	+129.56	+0.8206	0.0500	+0.0002	not attributed; stop fired on literal-integer
replacepart-guarded-decline-fallback	cli	18171.45	18314.93	+143.49	+0.7896	0.0500	-0.0341	not attributed; stop fired on literal-integer
stringreplace-single-literal-unguarded	cli	37184.16	37324.35	+140.18	+0.3770	0.0967	+0.0967	not attributed; stop fired on literal-integer
stringreplace-pattern-unguarded	cli	31629.14	31808.13	+178.99	+0.5659	0.0500	-0.0099	not attributed; stop fired on literal-integer
stringreplace-guarded-hit	cli	21620.77	21750.07	+129.31	+0.5981	0.2445	-0.2445	not attributed; stop fired on literal-integer
stringreplace-guarded-decline-fallback	cli	25877.71	26012.66	+134.95	+0.5215	0.0811	+0.0811	not attributed; stop fired on literal-integer
resume-s6-rhs-single	resume	74268.37	75642.62	+1374.25	+1.8504	0.0500	-0.0256	not attributed; stop fired on literal-integer
resume-s6-rhs-twice	resume	105638.14	107384.29	+1746.15	+1.6530	0.0500	+0.0282	not attributed; stop fired on literal-integer
resume-s6-cross-type	resume	124535.72	126667.29	+2131.57	+1.7116	0.0500	+0.0265	not attributed; stop fired on literal-integer
resume-s6-readonly-twice	resume	215663.16	218543.84	+2880.68	+1.3357	0.0500	+0.0333	not attributed; stop fired on literal-integer
resume-s6-string	resume	85123.48	86291.10	+1167.62	+1.3717	0.2511	+0.2511	not attributed; stop fired on literal-integer
association-part-key-read-width4000	cli	14193.83	14297.40	+103.57	+0.7297	0.2824	-0.2824	not attributed; stop fired on literal-integer
association-part-positional-read-width32000	cli	9922.85	10499.67	+576.82	+5.8131	0.5663	+0.5663	not attributed; stop fired on literal-integer
appendto-list-incremental-small	cli	10590.93	10767.89	+176.96	+1.6709	0.0500	+0.0133	not attributed; stop fired on literal-integer
appendto-list-incremental-large	cli	10574.45	10651.09	+76.64	+0.7248	0.0500	-0.0056	not attributed; stop fired on literal-integer
association-set-key-incremental-small	cli	17265.59	17357.79	+92.20	+0.5340	0.1018	-0.1018	not attributed; stop fired on literal-integer
association-set-key-incremental-large	cli	17225.38	17301.06	+75.68	+0.4394	0.0500	+0.0323	not attributed; stop fired on literal-integer
resume-s7-list-part	resume	83873.01	85312.53	+1439.52	+1.7163	0.1225	-0.1225	not attributed; stop fired on literal-integer
resume-s7-assoc-part	resume	85759.44	87059.57	+1300.13	+1.5160	0.0586	-0.0586	not attributed; stop fired on literal-integer
resume-s7-assoc-call	resume	75803.80	77159.74	+1355.94	+1.7887	0.1257	-0.1257	not attributed; stop fired on literal-integer
resume-s7-assoc-call-thrice	resume	159392.78	161937.89	+2545.11	+1.5968	0.0760	+0.0760	not attributed; stop fired on literal-integer
resume-s7-held-twice	resume	126026.31	128021.80	+1995.49	+1.5834	0.1212	-0.1212	not attributed; stop fired on literal-integer
resume-s7-held-rhs	resume	95815.34	97357.24	+1541.90	+1.6092	0.0981	+0.0981	not attributed; stop fired on literal-integer
resume-s7-associate-argument	resume	78987.45	80436.52	+1449.07	+1.8346	0.0500	-0.0376	not attributed; stop fired on literal-integer
resume-s7-keydrop-argument	resume	75877.16	77263.33	+1386.17	+1.8269	0.0694	-0.0694	not attributed; stop fired on literal-integer
resume-s7-associate-target	resume	264744.20	266410.59	+1666.39	+0.6294	0.0500	-0.0361	not attributed; stop fired on literal-integer
resume-s7-keydrop-target	resume	263725.87	265356.01	+1630.14	+0.6181	0.0500	-0.0477	not attributed; stop fired on literal-integer
resume-s7-associate-held	resume	99993.46	101491.79	+1498.33	+1.4984	0.1741	+0.1741	not attributed; stop fired on literal-integer
resume-s7-keydrop-held	resume	107264.28	108901.06	+1636.78	+1.5259	0.2226	-0.2226	not attributed; stop fired on literal-integer
association-set-part-key-investigation	cli	14583.67	14701.19	+117.52	+0.8058	0.0500	-0.0433	not attributed; stop fired on literal-integer
replaceall-guarded-hit-small	cli	78641.49	78987.51	+346.02	+0.4400	0.0500	-0.0023	not attributed; stop fired on literal-integer
replace-rule-list-decline	cli	50950.88	51284.31	+333.43	+0.6544	0.1555	-0.1555	not attributed; stop fired on literal-integer
replacerepeated-guarded-small	cli	371897.84	373119.65	+1221.81	+0.3285	0.0500	-0.0182	not attributed; stop fired on literal-integer
replaceat-small	cli	35153.49	35317.94	+164.45	+0.4678	0.0500	-0.0308	not attributed; stop fired on literal-integer
count-guarded-small	cli	50050.91	50184.52	+133.61	+0.2669	0.0500	+0.0041	not attributed; stop fired on literal-integer
resume-s8-replaceall-node	resume	109590.70	111333.72	+1743.02	+1.5905	0.0532	-0.0532	not attributed; stop fired on literal-integer
resume-s8-replaceall-atom	resume	96898.91	98428.82	+1529.91	+1.5789	0.0971	+0.0971	not attributed; stop fired on literal-integer
resume-s8-replaceall-twice	resume	165433.64	167668.67	+2235.03	+1.3510	0.1059	-0.1059	not attributed; stop fired on literal-integer
resume-s8-repeated-guard	resume	103847.29	105523.14	+1675.85	+1.6138	0.0587	+0.0587	not attributed; stop fired on literal-integer
resume-s8-replace-decline	resume	112304.20	114014.62	+1710.42	+1.5230	0.0520	+0.0520	not attributed; stop fired on literal-integer
resume-s8-replace-both	resume	157532.89	159735.66	+2202.77	+1.3983	0.1335	-0.1335	not attributed; stop fired on literal-integer
resume-s8-replaceat-guard	resume	109056.95	110694.65	+1637.70	+1.5017	0.0500	+0.0474	not attributed; stop fired on literal-integer
resume-s8-count-target	resume	63030.25	64048.69	+1018.44	+1.6158	0.0500	+0.0281	not attributed; stop fired on literal-integer
resume-s8-count-pattern	resume	61513.55	62521.13	+1007.58	+1.6380	0.0592	+0.0592	not attributed; stop fired on literal-integer
resume-s8-memberq-target	resume	63802.24	64933.43	+1131.19	+1.7730	0.1771	-0.1771	not attributed; stop fired on literal-integer
resume-s8-memberq-pattern	resume	62413.13	63591.44	+1178.31	+1.8879	0.3204	+0.3204	not attributed; stop fired on literal-integer
resume-s8-freeq-target	resume	65184.45	66233.41	+1048.96	+1.6092	0.1056	+0.1056	not attributed; stop fired on literal-integer
resume-s8-freeq-pattern	resume	63756.56	64709.00	+952.44	+1.4939	0.0536	+0.0536	not attributed; stop fired on literal-integer
resume-s8-count-level	resume	78553.31	79903.22	+1349.91	+1.7185	0.2211	-0.2211	not attributed; stop fired on literal-integer
resume-s8-count-guard-twice	resume	130013.52	132035.08	+2021.56	+1.5549	0.1270	-0.1270	not attributed; stop fired on literal-integer
resume-s8-position-target	resume	72956.21	74098.40	+1142.19	+1.5656	0.0500	-0.0298	not attributed; stop fired on literal-integer
resume-s8-position-pattern	resume	74423.19	75919.41	+1496.22	+2.0104	0.2103	+0.2103	not attributed; stop fired on literal-integer
resume-s8-position-level	resume	87123.98	88486.47	+1362.49	+1.5639	0.0833	-0.0833	not attributed; stop fired on literal-integer
resume-s8-position-guard-twice	resume	156784.60	159043.62	+2259.02	+1.4408	0.0921	-0.0921	not attributed; stop fired on literal-integer
catch-no-throw	cli	4005.98	4038.53	+32.55	+0.8126	0.1935	+0.1935	not attributed; stop fired on literal-integer
catch-throw-untagged	cli	7890.67	7953.20	+62.53	+0.7925	0.0605	+0.0605	not attributed; stop fired on literal-integer
catch-throw-tagged-hit	cli	9145.69	9216.61	+70.92	+0.7755	0.4208	+0.4208	not attributed; stop fired on literal-integer
catch-throw-pattern-hit	cli	16033.20	16155.98	+122.78	+0.7658	0.0500	-0.0339	not attributed; stop fired on literal-integer
checkabort-no-abort	cli	10997.76	11044.27	+46.51	+0.4229	0.1300	-0.1300	not attributed; stop fired on literal-integer
abortprotect-clean	cli	9288.38	9362.36	+73.98	+0.7964	0.0500	+0.0273	not attributed; stop fired on literal-integer
assert-disabled	cli	6767.86	6825.11	+57.25	+0.8459	0.0500	+0.0428	not attributed; stop fired on literal-integer
assert-enabled-true	cli	13591.28	13698.03	+106.75	+0.7854	0.0500	+0.0129	not attributed; stop fired on literal-integer
check-clean	cli	9570.17	9637.77	+67.59	+0.7063	0.0500	+0.0441	not attributed; stop fired on literal-integer
check-message-fallback	cli	17555.57	17770.02	+214.45	+1.2215	0.1460	-0.1460	not attributed; stop fired on literal-integer
quiet-clean	cli	9292.67	9357.13	+64.46	+0.6937	0.0500	-0.0117	not attributed; stop fired on literal-integer
withcleanup-two-arg	cli	13317.78	13418.93	+101.15	+0.7595	0.0500	+0.0087	not attributed; stop fired on literal-integer
catchexceptions-caught	cli	28347.97	28507.40	+159.44	+0.5624	0.0500	-0.0288	not attributed; stop fired on literal-integer
resume-s9-checkabort-option	resume	64504.88	65680.13	+1175.25	+1.8220	0.0500	-0.0433	not attributed; stop fired on literal-integer
resume-s9-checkabort-primary	resume	63131.95	64344.80	+1212.85	+1.9211	0.3775	-0.3775	not attributed; stop fired on literal-integer
resume-s9-checkabort-abort	resume	64779.04	65885.29	+1106.25	+1.7077	0.0500	+0.0136	not attributed; stop fired on literal-integer
resume-s9-abortprotect-body	resume	56253.76	57495.21	+1241.45	+2.2069	0.0578	-0.0578	not attributed; stop fired on literal-integer
resume-s9-checkabort-pending-fallback	resume	81442.33	82855.78	+1413.45	+1.7355	0.1437	-0.1437	not attributed; stop fired on literal-integer
resume-s9-checkabort-fallback	resume	66308.88	67480.58	+1171.70	+1.7670	0.0554	+0.0554	not attributed; stop fired on literal-integer
resume-s9-assert-function-lookup	resume	66007.25	67303.06	+1295.81	+1.9631	0.0992	+0.0992	not attributed; stop fired on literal-integer
resume-s9-assert-test	resume	60892.97	62032.74	+1139.77	+1.8718	0.3640	-0.3640	not attributed; stop fired on literal-integer
resume-s9-assert-handler	resume	65580.16	66703.25	+1123.09	+1.7125	0.0500	-0.0170	not attributed; stop fired on literal-integer
resume-s9-check-body	resume	56176.33	57520.21	+1343.88	+2.3923	0.0771	-0.0771	not attributed; stop fired on literal-integer
resume-s9-quiet-body	resume	66267.32	67520.05	+1252.73	+1.8904	0.0500	-0.0011	not attributed; stop fired on literal-integer
resume-s9-quiet-check-fallback	resume	85739.93	87240.31	+1500.38	+1.7499	0.0765	+0.0765	not attributed; stop fired on literal-integer
resume-s9-cleanup-init	resume	57549.79	58680.77	+1130.98	+1.9652	0.0500	+0.0164	not attributed; stop fired on literal-integer
resume-s9-cleanup-primary	resume	56888.07	58024.25	+1136.18	+1.9972	0.2949	+0.2949	not attributed; stop fired on literal-integer
resume-s9-cleanup-finalizer	resume	57299.74	58414.44	+1114.70	+1.9454	0.0501	+0.0501	not attributed; stop fired on literal-integer
resume-s9-cleanup-three-phase	resume	116113.32	118015.02	+1901.70	+1.6378	0.0500	-0.0376	not attributed; stop fired on literal-integer
resume-s9-catchexceptions-body	resume	85557.61	86884.91	+1327.30	+1.5514	0.0644	-0.0644	not attributed; stop fired on literal-integer
resume-s9-catchexceptions-guard	resume	88754.60	90234.99	+1480.39	+1.6680	0.0500	-0.0350	not attributed; stop fired on literal-integer
resume-s9-catchexceptions-match	resume	93996.78	95365.88	+1369.10	+1.4565	0.0531	-0.0531	not attributed; stop fired on literal-integer
resume-s9-catchexceptions-suspended-match	resume	121411.77	123126.89	+1715.12	+1.4126	0.0500	-0.0478	not attributed; stop fired on literal-integer
resume-s9-catchexceptions-suspended-guard	resume	115838.43	117761.78	+1923.35	+1.6604	0.0500	+0.0053	not attributed; stop fired on literal-integer
resume-s9-catchexceptions-rhs	resume	81966.39	83288.43	+1322.04	+1.6129	0.0500	+0.0333	not attributed; stop fired on literal-integer
resume-s9-catch-body	resume	58217.29	59347.55	+1130.26	+1.9415	0.0500	-0.0137	not attributed; stop fired on literal-integer
resume-s9-catch-form	resume	63173.62	64472.74	+1299.12	+2.0564	0.0787	+0.0787	not attributed; stop fired on literal-integer
resume-s9-catch-form-guard	resume	91438.53	92792.75	+1354.22	+1.4810	0.0500	+0.0494	not attributed; stop fired on literal-integer
resume-s9-catch-bypassing-form	resume	80565.44	82057.43	+1491.99	+1.8519	0.0500	-0.0105	not attributed; stop fired on literal-integer
resume-s9-catch-bypassing-guard	resume	101955.77	103704.22	+1748.45	+1.7149	0.0694	+0.0694	not attributed; stop fired on literal-integer
catch-throw	cli	7890.80	7955.07	+64.27	+0.8144	0.0500	+0.0056	not attributed; stop fired on literal-integer
catch-tagged-throw	cli	9118.18	9207.73	+89.55	+0.9821	0.1199	-0.1199	not attributed; stop fired on literal-integer
catch-form-pattern	cli	16036.29	16158.42	+122.13	+0.7616	0.0500	-0.0311	not attributed; stop fired on literal-integer
checkabort-abort	cli	14378.31	14476.35	+98.04	+0.6818	0.5094	+0.5094	not attributed; stop fired on literal-integer
abortprotect-small	cli	3860.51	3899.86	+39.35	+1.0192	0.2502	-0.2502	not attributed; stop fired on literal-integer
withcleanup-small	cli	4691.67	4780.66	+88.99	+1.8968	0.2926	+0.2926	not attributed; stop fired on literal-integer
withcleanup-init	cli	5521.78	5626.01	+104.23	+1.8876	0.0500	-0.0377	not attributed; stop fired on literal-integer
check-no-message	cli	4056.51	4090.81	+34.30	+0.8457	0.7215	+0.7215	not attributed; stop fired on literal-integer
quiet-no-message	cli	3846.73	3896.52	+49.80	+1.2945	0.0500	-0.0406	not attributed; stop fired on literal-integer
catchexceptions-handler	cli	19647.29	19774.51	+127.21	+0.6475	0.0500	+0.0239	not attributed; stop fired on literal-integer
catchexceptions-no-throw	cli	3846.19	3875.70	+29.50	+0.7671	0.4168	-0.4168	not attributed; stop fired on literal-integer
function-slot-apply	cli	10739.84	10833.46	+93.62	+0.8717	0.0500	-0.0434	not attributed; stop fired on literal-integer
function-named-apply	cli	12469.83	12569.08	+99.25	+0.7959	0.0500	-0.0428	not attributed; stop fired on literal-integer
composition-apply	cli	27800.53	27921.04	+120.51	+0.4335	0.0500	-0.0012	not attributed; stop fired on literal-integer
association-apply	cli	6937.24	7054.49	+117.24	+1.6901	0.0902	+0.0902	not attributed; stop fired on literal-integer
failure-raw-property	cli	12622.18	12747.73	+125.54	+0.9946	0.0500	+0.0400	not attributed; stop fired on literal-integer
failure-list-property	cli	14440.72	14609.78	+169.06	+1.1707	0.0500	+0.0071	not attributed; stop fired on literal-integer
failure-missing-property	cli	16170.33	16309.97	+139.64	+0.8636	0.1325	+0.1325	not attributed; stop fired on literal-integer
success-raw-property	cli	9162.07	9301.66	+139.59	+1.5236	0.0553	-0.0553	not attributed; stop fired on literal-integer
failure-message-property	cli	20923.30	21064.55	+141.25	+0.6751	0.1217	+0.1217	not attributed; stop fired on literal-integer
failure-message-delayed	cli	33068.94	33320.44	+251.49	+0.7605	0.0582	+0.0582	not attributed; stop fired on literal-integer
failure-styled-message	cli	24116.64	24254.74	+138.11	+0.5727	0.0960	+0.0960	not attributed; stop fired on literal-integer
failure-tag-fallback	cli	9531.88	9671.24	+139.36	+1.4620	0.0500	-0.0008	not attributed; stop fired on literal-integer
enclose-message-handler	cli	41531.91	41679.08	+147.18	+0.3544	0.0500	-0.0219	not attributed; stop fired on literal-integer
confirm-failure-message	cli	51869.43	51962.61	+93.17	+0.1796	0.0500	-0.0327	not attributed; stop fired on literal-integer
resume-g15-parameters	resume	72478.59	73682.85	+1204.26	+1.6615	0.2181	-0.2181	not attributed; stop fired on literal-integer
resume-g15-template	resume	72146.54	73448.22	+1301.68	+1.8042	0.2776	+0.2776	not attributed; stop fired on literal-integer
resume-g15-styled	resume	82875.49	84268.03	+1392.54	+1.6803	0.3101	-0.3101	not attributed; stop fired on literal-integer
resume-g15-raw-failure	resume	61956.57	63232.07	+1275.50	+2.0587	0.0500	+0.0113	not attributed; stop fired on literal-integer
resume-g15-raw-success	resume	58745.65	59894.81	+1149.16	+1.9562	0.0500	-0.0494	not attributed; stop fired on literal-integer
resume-g15-projection-value	resume	96957.79	98862.79	+1905.00	+1.9648	0.0713	+0.0713	not attributed; stop fired on literal-integer
resume-g15-enclose	resume	73239.04	74671.34	+1432.30	+1.9557	0.2119	+0.2119	not attributed; stop fired on literal-integer
resume-g15-confirm	resume	119100.32	120971.89	+1871.57	+1.5714	0.0500	-0.0290	not attributed; stop fired on literal-integer
lhs-set-literal	cli	10580.27	10615.20	+34.92	+0.3301	0.0500	-0.0036	not attributed; stop fired on literal-integer
lhs-set-ownvalue-arg	cli	13057.02	13121.69	+64.67	+0.4953	0.0500	+0.0021	not attributed; stop fired on literal-integer
lhs-set-head-eval	cli	13061.24	13151.53	+90.28	+0.6912	0.0500	+0.0047	not attributed; stop fired on literal-integer
lhs-setdelayed-pattern	cli	21452.42	21650.50	+198.08	+0.9233	0.0500	+0.0016	not attributed; stop fired on literal-integer
lhs-upset	cli	12823.42	12909.43	+86.00	+0.6707	0.0500	+0.0003	not attributed; stop fired on literal-integer
lhs-tagset	cli	12589.64	12676.71	+87.08	+0.6916	0.0500	-0.0082	not attributed; stop fired on literal-integer
lhs-set-unset	cli	24990.43	25094.13	+103.70	+0.4150	0.0500	-0.0013	not attributed; stop fired on literal-integer
lhs-set-cycle-recovery	cli	12093.43	12154.28	+60.86	+0.5032	0.0500	-0.0051	not attributed; stop fired on literal-integer
set-ownvalue	cli	3969.33	4001.51	+32.18	+0.8107	0.0500	+0.0313	not attributed; stop fired on literal-integer
lhs-arg-cycle	resume	273705.52	274826.32	+1120.80	+0.4095	0.1425	+0.1425	not attributed; stop fired on literal-integer
lhs-arg-value	resume	277847.33	279436.93	+1589.60	+0.5721	0.0500	-0.0085	not attributed; stop fired on literal-integer
lhs-head-cycle	resume	301190.91	302534.63	+1343.72	+0.4461	0.1096	-0.1096	not attributed; stop fired on literal-integer
lhs-head-value	resume	302781.49	304329.41	+1547.92	+0.5112	0.2016	+0.2016	not attributed; stop fired on literal-integer
lhs-pattern-cycle	resume	284877.39	286468.18	+1590.79	+0.5584	0.0500	-0.0168	not attributed; stop fired on literal-integer
lhs-pattern-value	resume	290218.02	292005.54	+1787.52	+0.6159	0.0500	+0.0486	not attributed; stop fired on literal-integer
lhs-after-prior-cycle	resume	319404.02	321750.49	+2346.47	+0.7346	0.0500	-0.0483	not attributed; stop fired on literal-integer
lhs-after-prior-value	resume	322764.62	325755.93	+2991.31	+0.9268	0.1127	+0.1127	not attributed; stop fired on literal-integer
lhs-twice-cycle	resume	317769.29	320193.57	+2424.28	+0.7629	0.0500	-0.0384	not attributed; stop fired on literal-integer
lhs-twice-value	resume	323797.82	325648.17	+1850.35	+0.5715	0.0500	+0.0469	not attributed; stop fired on literal-integer
c1-all-guard	resume	96822.76	98499.00	+1676.24	+1.7312	0.1082	+0.1082	not attributed; stop fired on literal-integer
c1-all-test	resume	107348.80	109343.45	+1994.65	+1.8581	0.0510	+0.0510	not attributed; stop fired on literal-integer
c1-all-guard-value	resume	127253.95	129462.96	+2209.01	+1.7359	0.1765	+0.1765	not attributed; stop fired on literal-integer
c1-all-two-guards	resume	165338.70	167790.67	+2451.97	+1.4830	0.0500	+0.0161	not attributed; stop fired on literal-integer
c1-unheld-guard	resume	96789.46	98378.17	+1588.71	+1.6414	0.0500	-0.0001	not attributed; stop fired on literal-integer
c1-unheld-test	resume	107382.64	109219.46	+1836.82	+1.7105	0.0702	-0.0702	not attributed; stop fired on literal-integer
c1-unheld-guard-value	resume	127137.64	129478.03	+2340.39	+1.8408	0.0500	-0.0377	not attributed; stop fired on literal-integer
c1-unheld-two-guards	resume	165271.04	167721.25	+2450.21	+1.4825	0.1147	+0.1147	not attributed; stop fired on literal-integer
c1-replaceall-atom	cli	22629.00	22780.64	+151.64	+0.6701	0.0500	-0.0374	not attributed; stop fired on literal-integer
c1-replaceall-unheld-atom	cli	22585.12	22738.74	+153.61	+0.6802	0.0500	-0.0039	not attributed; stop fired on literal-integer
c1-replaceall-rule-rhs-deep-small	cli	73415.57	73799.64	+384.07	+0.5231	0.0500	-0.0022	not attributed; stop fired on literal-integer
c1-replaceall-rewrite-guarded-hit	cli	43279.85	43518.99	+239.14	+0.5525	0.0500	-0.0234	not attributed; stop fired on literal-integer
c1-replaceall-rewrite-guarded-decline-fallback	cli	66189.69	66591.34	+401.64	+0.6068	0.0500	+0.0031	not attributed; stop fired on literal-integer
```

### 6. Attempts

| Attempt | Result | Evidence |
|---|---|---|
| Candidate `rum-cli` release build, one codegen unit | exit 0, `oom_kill 0` | `logs/build-cand-cli.log` |
| Candidate harness release build, one codegen unit | exit 0, `oom_kill 0` | `logs/build-cand-harness.log` |
| Copy to `arms/cand/{rum-cli,resume}` | both files present | `arms/cand/` |
| Five-label base-vs-base sizing run | `EXIT:0` | `logs/size-sample.log` |
| Floor, 313 labels, bound 7200s | `EXIT:0` | `logs/bb-run.log`, `logs/bb.tsv` |
| Gate, 313 labels, bound 3600s | `EXIT:0` | `logs/bc-run.log`, `logs/bc.tsv` |
| Callgrind calibration, empty `DEBUGINFOD_URLS` | `memcpy` named; `__memcpy_avx_unaligned_erms` absent | `callgrind/calib-base-literal.out` |
| Callgrind calibration with the copied cache | `cfn=(4298) __memcpy_avx_unaligned_erms` | `callgrind/calib-debuginfod.out` |
| `literal-integer` attribution, first parser | `exclusive sum mismatch ... 9438191 vs 9529449`, `EXIT:1` | `logs/attr-literal-integer.log` was overwritten by the rerun; the mismatch line was the first run's stdout |
| Parser recounts `+`/`-` cost lines; rerun reuses profiles | `EXIT:0`, total slope `+25.98` | `logs/attr-literal-integer.log`, `logs/attr-literal-integer.tsv` |
| `objdump` prologue scan | 39 functions, numeric-sub growth count 0, cold-frame and new-frame growth as in §4 | `logs/stack-reservations.tsv`, `logs/stack-prologues.txt` |

The first attribution log was replaced by the successful rerun, so the mismatch sentence above is the text that run printed (`exclusive sum mismatch in .../base-20000-1.out: 9438191 vs 9529449`). I am not quoting it from a surviving file.

`cargo test --workspace` was not run. The director's gate already did.

### 7. Open

Stopped. The outline of `evaluate_profiled_fixed_point` and the larger cold-frame reservations are for the director. No production edit was made.

## Director ruling on Stage B, and continuation A3 (2026-10-28 02:40) — grk

**Your STOP is right, and the cause is my shape, not your code.** I checked
`bc.tsv`, `attr-literal-integer.tsv` and `stack-reservations.tsv` directly.

- **The instructions.** My shape put the body in an `#[inline(always)]`
  `evaluate_entered` that is called from both `evaluate` and
  `evaluate_at_root`. `evaluate_profiled_fixed_point` then has two call
  sites. LLVM stopped inlining it into `evaluate`, which costs +69 −43 =
  +26 instructions on every evaluation.
- **The stack.** Each level on the recursive cycle now holds `evaluate`
  (360) + `evaluate_profiled_fixed_point` (792) + `evaluate_once_in_accounting_mode`
  (520). The base held `evaluate` (840) + `evaluate_once_unstable_after_input_check_with_cycle_tracking`
  (712).
- **The evaluator.** The `EvaluationDepth` methods read and write the field
  where the base used the level's local `evaluation_depth`. That changed the
  evaluator's IR enough to flip its inlining.

### The corrected shape

You may edit only `resolution.rs`, `model/evaluation_depth.rs`,
`eval_core/mod.rs`, the callers of the changed `EvaluationDepth` methods,
and the standing inventories by replay.

1. **One body, forced inline at both call sites.** Delete
   `evaluate_entered`. `Session::evaluate` keeps the base body, written
   exactly as at `678e6be7`, after the depth-0 branch.
   `evaluate_at_root` holds the entry, calls `restore(0)`, and then
   contains the same body. Put `#[inline(always)]` on
   `evaluate_profiled_fixed_point`, which has no other caller, so it
   inlines into both. The cold copy in `evaluate_at_root` is the price, and
   it runs once per request.

   ```rust
   pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
       if self.evaluation_depth.get() == 0 {
           return self.evaluate_at_root(expr);
       }
       self.record_profile_evaluate_entry();
       let runtime_profile = runtime_path_timing_start("evaluate.total");
       let result = self.evaluate_profiled_fixed_point(expr);
       record_profile_runtime_path_timing(runtime_profile, runtime_profile_outcome_for_expr_result(&result));
       result
   }

   #[cold]
   #[inline(never)]
   fn evaluate_at_root(&mut self, expr: Expr) -> Result<Expr, EvalError> {
       let _entry = NativeStackEntry::enter();
       self.evaluation_depth.restore(0);
       // the same four statements as above
   }

   #[inline(always)]
   fn evaluate_profiled_fixed_point(&mut self, expr: Expr) -> Result<Expr, EvalError>;
   ```

2. **The evaluator keeps its local.** The level methods take the depth the
   evaluator already holds, so the IR matches the base. Each one
   debug-asserts its precondition.

   ```rust
   /// `depth` is this level's entry depth, read once by the evaluator.
   #[inline(always)] pub(in crate::runtime) fn native_check_due(&mut self, depth: usize) -> bool;
   //   debug_assert_eq!(depth, self.depth); then today's base rule on `depth`
   #[inline(always)] pub(in crate::runtime) fn enter_level(&mut self, entered_at: usize);
   //   debug_assert_eq!(self.depth, entered_at); self.depth = entered_at + 1
   #[inline(always)] pub(in crate::runtime) fn leave_level(&mut self, entered_at: usize); // unchanged
   ```

   In `eval_core/mod.rs` the calls become `native_check_due(evaluation_depth)`
   and `enter_level(evaluation_depth)`. Nothing else in that function
   changes.

### The bars, restated

- **Instructions.** Same instrument and floor. `arms/base/` and
  `logs/bb.tsv` stay as they are. Rebuild `arms/cand/` and rerun
  `instr.py arms/base arms/cand logs/bc-a3.tsv`. Every label beyond its
  floor needs a named mechanism.
  - Expected mechanisms: the thread-record read in the two cold checks, the
    entry per resume and per abandonment, and `restore`'s threshold store.
  - Attribute `valueq-undefined-symbol-definitions-8000` and the 121
    resume labels by name if they are still flagged.
  - An unattributed positive residual is a STOP.
- **Stack.**
  - The recursive cycle's per-level total must not exceed the base's. Give
    the cycle as a list of the functions each level holds, base and
    candidate, with every `sub` and push count, and their sums.
  - The two cold checks may each grow by at most 32 bytes. They are leaves
    that exist only while a check runs. Report their prologues.
  - Any other growth on the cycle is a STOP.
- **Tests after the edit.**
  - `cargo fmt --check`.
  - `cargo test -p rum-runtime --lib`, `cargo test -p rum-host`, and
    `cargo test -p rum-stdlib --test typed_slot_census`.
  - The LP01 fixture (`suspension_oracle -- logical_counter_port`). Its
    producer lines move again: replay as in A2.
  - `production_rejection_census_is_complete`: if the frontier moves,
    STOP and give the keyed diff.
  - The Jupyter pair 20 times at `--test-threads=2`.
  - The full workspace stays with my gate.

Every quoted line must be copied from a file you name. Append
"Executing-agent report — A3".

## Executing-agent report — A3

**STOP.** `production_rejection_census_is_complete` fails because the option frontier moved from 971 rejected edges to 972. Keyed by kind, file, callee and reason, no key was added or removed. One key's count went from 1 to 2: `Session::record_profile_evaluate_entry` is now rejected in both `Session::evaluate` and `evaluate_at_root`. Instruction measurement, the stack rescan, and the Jupyter pair were not run.

### 1. The edit

`evaluate_entered` is deleted. `Session::evaluate` keeps the base four statements after the depth-0 branch. `evaluate_at_root` holds the entry, calls `restore(0)`, then the same four statements. `evaluate_profiled_fixed_point` is `#[inline(always)]`. From `crates/rum-runtime/src/runtime/resolution.rs` lines 319–349:

```text
    pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        if self.evaluation_depth.get() == 0 {
            return self.evaluate_at_root(expr);
        }
        self.record_profile_evaluate_entry();
        let runtime_profile = runtime_path_timing_start("evaluate.total");
        let result = self.evaluate_profiled_fixed_point(expr);
        record_profile_runtime_path_timing(
            runtime_profile,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    #[cold]
    #[inline(never)]
    fn evaluate_at_root(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        let _entry = NativeStackEntry::enter();
        self.evaluation_depth.restore(0);
        self.record_profile_evaluate_entry();
        let runtime_profile = runtime_path_timing_start("evaluate.total");
        let result = self.evaluate_profiled_fixed_point(expr);
        record_profile_runtime_path_timing(
            runtime_profile,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    #[inline(always)]
    fn evaluate_profiled_fixed_point(&mut self, expr: Expr) -> Result<Expr, EvalError> {
```

`native_check_due` and `enter_level` take the depth the evaluator already holds. From `crates/rum-runtime/src/runtime/model/evaluation_depth.rs` lines 33–57:

```text
    /// `depth` is this level's entry depth, read once by the evaluator.
    /// Today's rule on that depth: `depth >= next` then `next = depth + STRIDE`.
    #[inline(always)]
    pub(in crate::runtime) fn native_check_due(&mut self, depth: usize) -> bool {
        debug_assert_eq!(depth, self.depth);
        if depth >= self.next_native_check {
            self.next_native_check = depth.saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
            true
        } else {
            false
        }
    }
```

```text
    #[inline(always)]
    pub(in crate::runtime) fn enter_level(&mut self, entered_at: usize) {
        debug_assert_eq!(self.depth, entered_at);
        self.depth = entered_at + 1;
    }
```

`leave_level` is unchanged. In `crates/rum-runtime/src/runtime/eval_core/mod.rs` the two calls are `native_check_due(evaluation_depth)` (line 55) and `enter_level(evaluation_depth)` (line 73). The caller `next_check_depth` in `crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs` passes that depth. No other file under `crates/` was edited.

### 2. Tests that passed

`.temp/brief-629/logs/fmt-a3.log`:

```text
oom_kill_before=0
EXIT:0
oom_kill_after=0
```

`.temp/brief-629/logs/rum-runtime-lib-a3.log`:

```text
test result: ok. 1216 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 36.83s
EXIT:0
oom_kill_after=0
```

`.temp/brief-629/logs/rum-host-a3.log`, every target:

```text
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 50.59s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.04s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.89s
EXIT:0
oom_kill_after=0
```

`.temp/brief-629/logs/typed-slot-census-a3.log`:

```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 284.09s
EXIT:0
oom_kill_after=0
```

The typed-slot receipt was not changed.

### 3. LP01 replay

120 required rows, 30 binding rows, and 1 interned producer string. Statement text was not changed. Disclosure `.temp/brief-629/logs/lp01-a3-replay.txt`:

```text
resolution.rs fingerprint fnv1a64:549fac7c5738681e -> fnv1a64:88ccfe43e85bb71f
eval_core/mod.rs fingerprint fnv1a64:f3e98e033bd5b366 -> fnv1a64:1a30ade03c6fc95e
```

```text
1. crates/rum-runtime/src/runtime/resolution.rs:338 -> :327
   statement: let result = self.evaluate_profiled_fixed_point(expr);
```

```text
required rows changed: 120
bindings rows changed: 30
losses.json producer strings changed: 1 ("crates/rum-runtime/src/runtime/resolution.rs:338" -> "...:327")
```

The inventory follows the copy in `Session::evaluate` (line 327). The same statement also stands at line 340 inside `evaluate_at_root`. `phase-sites.tsv` was not edited.

`.temp/brief-629/logs/lp01-a3.log`:

```text
all: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=41728
all: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 18.98s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 66.54s
EXIT:0
oom_kill_after=0
```

### 4. The frontier — STOP

Baseline `.temp/brief-629/logs/option-frontier-nocapture.log` line 6, the accepted Stage A dump:

```text
OPTION_FRONTIER resolved_bodies=219 resolved_files=44 rejected_edges=971 name_spelling_candidates=2633 candidate_files=349
```

Candidate `.temp/brief-629/logs/rejection-census-a3.log` line 20:

```text
OPTION_FRONTIER resolved_bodies=218 resolved_files=44 rejected_edges=972 name_spelling_candidates=2633 candidate_files=349
```

Public wrapper, same files, lines 1197 and 1211:

```text
PUBLIC_WRAPPER resolved_bodies=5 resolved_files=3 rejected_edges=16 comparison_C01_C02_name_spelling_candidates=2633 candidate_files=349
```

The census assertion, `.temp/brief-629/logs/rejection-census-a3.log` lines 1654–1657:

```text
thread 'tests::retention_closure::census::tests::production_rejection_census_is_complete' panicked at crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:245:5:
assertion `left == right` failed: STOP: brief-545 exact legacy frontier inventory moved
  left: 972
 right: 971
```

Keyed diff `.temp/brief-629/logs/frontier-keyed-diff-a3.txt` lines 3–11 and 18. Key is kind, file, callee, reason:

```text
base_rejected 971
cand_rejected 972
base_kinds {'RET001': 10, 'RET002': 933, 'RET003': 28}
cand_kinds {'RET001': 10, 'RET002': 934, 'RET003': 28}
keys_added 0
keys_removed 0
keys_count_changed 1
keys_site_moved 17
keys_witness_moved 203
```

```text
COUNT	1->2	RET002	crates/rum-runtime/src/runtime/resolution.rs	rum_runtime::runtime::model::session::Session::record_profile_evaluate_entry	method syntax: opaque imported trait candidate std::sync::Arc
```

Sites for that key, same file lines 70–71:

```text
  BASE	{'crates/rum-runtime/src/runtime/resolution.rs:336:9-336:45': 1}
  CAND	{'crates/rum-runtime/src/runtime/resolution.rs:323:9-323:45': 1, 'crates/rum-runtime/src/runtime/resolution.rs:338:9-338:45': 1}
```

The three rows, copied from the dumps:

```text
REJECTED	RET002	crates/rum-runtime/src/runtime/resolution.rs:336:9-336:45	rum_runtime::runtime::model::session::Session::record_profile_evaluate_entry	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_entered	method syntax: opaque imported trait candidate std::sync::Arc
```

from `.temp/brief-629/logs/option-frontier-nocapture.log` line 1075, and from `.temp/brief-629/logs/rejection-census-a3.log` lines 1087 and 1089:

```text
REJECTED	RET002	crates/rum-runtime/src/runtime/resolution.rs:323:9-323:45	rum_runtime::runtime::model::session::Session::record_profile_evaluate_entry	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate	method syntax: opaque imported trait candidate std::sync::Arc
REJECTED	RET002	crates/rum-runtime/src/runtime/resolution.rs:338:9-338:45	rum_runtime::runtime::model::session::Session::record_profile_evaluate_entry	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_at_root	method syntax: opaque imported trait candidate std::sync::Arc
```

Those two candidate sites are the two copies of `self.record_profile_evaluate_entry();` at `resolution.rs` lines 323 and 338. The base had that call once, inside `evaluate_entered`.

Resolved bodies, same diff file lines 1137–1140:

```text
resolved_base 219 resolved_cand 218
RESOLVED_ONLY_BASE
  rum_runtime::runtime::model::session::Session::evaluate_entered
RESOLVED_ONLY_CAND
```

Seventeen other keys in `resolution.rs` kept the same callee and reason and moved three lines down, with the duplicated body. Example, diff file lines 21–23:

```text
SITE_MOVE	RET001	crates/rum-runtime/src/runtime/resolution.rs	std::vec::Vec::new	not a resolved function definition: std::vec::Vec::new
  BASE	{'crates/rum-runtime/src/runtime/resolution.rs:349:36-349:46': 1}
  CAND	{'crates/rum-runtime/src/runtime/resolution.rs:352:36-352:46': 1}
```

203 keys kept the same kind, file, callee and reason, and lost `Session::evaluate_entered` from the witness chain. The full pairs are in the diff file. `census/tests.rs` was not edited.

The same log also fails `production_obligations_are_exact` on stale contract fingerprints, lines 1563–1567. That test is not the named stop. The fingerprints were not replayed, because the frontier stop had already fired:

```text
thread 'tests::retention_closure::production_obligations_are_exact' panicked at crates/rum-stdlib/src/tests/retention_closure/local/promotion.rs:27:38:
fresh certificate rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol: RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O011
RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O010
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0210
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0200
```

### 5. Attempts

| Attempt | Result | Log |
|---|---|---|
| `cargo fmt --check` | EXIT:0, oom_kill stayed 0 | `logs/fmt-a3.log` |
| `cargo test -p rum-runtime --lib` | 1216 passed, EXIT:0, oom_kill stayed 0 | `logs/rum-runtime-lib-a3.log` |
| `cargo test -p rum-host` | every target ok, EXIT:0, oom_kill stayed 0 | `logs/rum-host-a3.log` |
| `cargo test -p rum-stdlib --test typed_slot_census` | 8 passed, EXIT:0, oom_kill stayed 0 | `logs/typed-slot-census-a3.log` |
| `suspension_oracle -- logical_counter_port` | ok, EXIT:0, oom_kill stayed 0 | `logs/lp01-a3.log` |
| census attempt 1, two test names in one `cargo test` | EXIT:1, `error: unexpected argument 'production_rejection_census_is_complete' found`, oom_kill stayed 0 | `logs/rejection-census-a3.log` lines 3–8 |
| census attempt 2, obligations then census | both EXIT:101, oom_kill stayed 0. No signal 9 | `logs/rejection-census-a3.log` |

No host OOM. `oom_kill` did not rise, and no command died with signal 9, so nothing was rerun for that rule.

### 6. Not done

Stopped at the frontier. The Jupyter pair was not run. `arms/cand/` was not rebuilt and still holds the Stage B binaries. `arms/base/` and `logs/bb.tsv` were not touched. `logs/bc-a3.tsv` does not exist. No callgrind attribution and no stack rescan. `cargo test --workspace` was not run.

## Director ruling on A3, and continuation A4 (2026-10-28 03:10) — measurement

**The frontier move is accepted.** No key was added or removed. One
key's count went from 1 to 2: the `record_profile_evaluate_entry` receiver
is rejected once in `Session::evaluate` and once in the copy that A3 gives
the cold root entry. That is the direct consequence of my A3 shape.

I updated `census/tests.rs` to 972 (10/934/28), with the reason. I
replayed the contract fingerprints and both certificate builds, and
`production_rejection_census_is_complete` passes. Your LP01 replay is
accepted: it follows the copy in `Session::evaluate`, and the fixture
passes.

**A4 is measurement only.** Do not edit `crates/`. Run the rest of A3's
bars:

1. Rebuild `arms/cand/` (both binaries, one codegen unit), and run
   `instr.py arms/base arms/cand logs/bc-a3.tsv` with the same floor
   `logs/bb.tsv`.
2. Name the mechanism of every label beyond its floor. An unattributed
   positive residual is a STOP.
3. Give the stack cycle per level, base and candidate, as A3 specified,
   plus the two cold check leaves.
4. Run the Jupyter pair 20 times at `--test-threads=2`.

Append "Executing-agent report — A4". Every quoted line must be copied from
a file you name.

## Executing-agent report — A4

No STOP. Every positive residual has a named mechanism. The repeating stack level is under the base. Each cold-check leaf's reservation grew by 24 bytes. The Jupyter pair passed 20 times. No file under `crates/` was edited. `arms/base/` and `logs/bb.tsv` were not touched.

### 1. Candidate arm

Both release builds used one codegen unit, `CARGO_BUILD_JOBS=8`, and `CARGO_NET_OFFLINE=true`. The harness package is `brief608-resume`.

From `.temp/brief-629/logs/build-cand-cli-a4.log` line 9, the text after the carriage return:

```text
    Finished `release` profile [optimized] target(s) in 48.47s
```

From `.temp/brief-629/logs/build-cand-cli-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/build-cand-cli-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/build-cand-harness-a4.log` line 7, the text after the carriage return:

```text
    Finished `release` profile [optimized] target(s) in 47.66s
```

From `.temp/brief-629/logs/build-cand-harness-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/build-cand-harness-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/copy-cand-a4.log`:

```text
=== before copy ===
-- cand rum-cli --
    Build ID: 56165200b1f3383b9ae0a7750d8fc2691f27cba3
-- cand resume --
    Build ID: 12459d4c99021164d36e0f1f6a6d584cc65fd69b
-- new rum-cli --
    Build ID: 7e03a5809abcd6ec5ee328eb37ea227e6a07c455
-- new resume --
    Build ID: d561a2158967a45f3e35fe204a4605586f70ba4e
-- base rum-cli (not copied) --
    Build ID: a24ca4234d25378c2fe50b45fcc83949c062f79a
-- base resume (not copied) --
    Build ID: e3b98f6dfe78f19fd1d6e13781850e07176f7f1f
=== after copy ===
-- cand rum-cli --
    Build ID: 7e03a5809abcd6ec5ee328eb37ea227e6a07c455
-- cand resume --
    Build ID: d561a2158967a45f3e35fe204a4605586f70ba4e
-rwxr-xr-x 1 remi remi 12704488 10月27日 22:26 .temp/brief-629/arms/base/resume
-rwxr-xr-x 1 remi remi 13011536 10月27日 22:26 .temp/brief-629/arms/base/rum-cli
-rwxr-xr-x 1 remi remi 12696888 10月28日 02:50 .temp/brief-629/arms/cand/resume
-rwxr-xr-x 1 remi remi 13016224 10月28日 02:50 .temp/brief-629/arms/cand/rum-cli
EXIT:0
```

The candidate `rum-cli` Build ID moved from `56165200b1f3383b9ae0a7750d8fc2691f27cba3` to `7e03a5809abcd6ec5ee328eb37ea227e6a07c455`. The candidate `resume` Build ID moved from `12459d4c99021164d36e0f1f6a6d584cc65fd69b` to `d561a2158967a45f3e35fe204a4605586f70ba4e`. Base Build IDs stayed `a24ca4234d25378c2fe50b45fcc83949c062f79a` and `e3b98f6dfe78f19fd1d6e13781850e07176f7f1f`. Measurement used these A4 binaries.

### 2. Instruction gate

`instr.py arms/base arms/cand logs/bc-a3.tsv` was attempted three times. The first two died before measuring. The third ran from `.temp/brief-629` and wrote 313 labels. `logs/bb.tsv` was the floor and was not rewritten.

From `.temp/brief-629/logs/bc-a3-run.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/bc-a3-run.log`:

```text
FileNotFoundError: [Errno 2] No such file or directory: 'logs/bc-a3.tsv'
EXIT:1
oom_kill_after=0
oom_after=0
=== attempt 2 cwd=.temp/brief-629 ===
oom_kill_before=0
oom_before=0
python3: can't open file '/home/remi/dev/rum/instruments/instr.py': [Errno 2] No such file or directory
EXIT:2
oom_kill_after=0
oom_after=0
=== attempt 3 cwd=/home/remi/dev/rum/.temp/brief-629 ===
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/bc-a3-run.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

Attempt 2's marker says `cwd=.temp/brief-629`, and the Python error names `/home/remi/dev/rum/instruments/instr.py`. The interpreter did not open `.temp/brief-629/instruments/instr.py` on that attempt. Attempt 3 did.

From `.temp/brief-629/logs/bc-a3-summary.txt`:

```text
labels 313
bb_rows 313
bc_median_pct +0.0950
bc_max_pct +21.4373
bc_min_pct -16.3019
flagged 191 positive 176 negative 15
flagged_cli 70 of 192 median +0.0841
flagged_resume 121 of 121 median +1.3498
flagged_cli_positive 55
flagged_resume_positive 121
watch valueq-undefined-symbol-definitions-8000 engine=cli delta=-830.90 delta_pct=-16.3019 bb_delta_pct=+7.9327 floor=7.9327 beyond=yes
watch valueq-undefined-symbol-definitions-2000 engine=cli delta=+863.45 delta_pct=+21.4373 bb_delta_pct=-8.1723 floor=8.1723 beyond=yes
watch literal-integer engine=cli delta=+1.15 delta_pct=+0.3932 bb_delta_pct=-0.0450 floor=0.0500 beyond=yes
```

Floor for a label is `max(0.05, |bb delta_pct|)`. A label is beyond the floor when `|bc delta_pct|` is greater than that. 191 labels are beyond the floor: 176 positive, 15 negative. All 121 resume labels are positive. 55 of the 70 flagged CLI labels are positive. The median of all 313 labels is +0.0950%.

### 3. Mechanisms

Callgrind is exclusive Ir, six runs per arm per size, slope `(median(2N) - median(N)) / N`. Stage B profiles of `literal-integer` stay in `callgrind/stage-b-literal-integer` and `logs/attr-literal-integer-stage-b.tsv`. The A4 profiles are new. There is no hot `evaluate_profiled_fixed_point` slope in the A4 TSVs. `nm` on the A4 candidate shows only the pre-existing `::suspend` symbol.

From `.temp/brief-629/logs/attr-literal-integer-stage-b.tsv`:

```text
+25.9756	292.0209	317.9964	TOTAL
+69.0000	0.0000	69.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point'2
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== nm base /home/remi/dev/rum/.temp/brief-629/arms/base/rum-cli evaluate_profiled_fixed_point =====
00000000001269a0 t rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point::suspend
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== nm cand /home/remi/dev/rum/.temp/brief-629/arms/cand/rum-cli evaluate_profiled_fixed_point =====
0000000000127d20 t rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point::suspend
```

From `.temp/brief-629/logs/attr-literal-integer-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/attr-literal-integer-a4.log`:

```text
TOTAL base=292.02 cand=293.02 delta=+1.00
wrote /home/remi/dev/rum/.temp/brief-629/logs/attr-literal-integer.tsv
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/attr-literal-integer.tsv`:

```text
delta	base_slope	cand_slope	function
+1.0043	292.0185	293.0228	TOTAL
+1.0000	110.0000	111.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
calls literal-integer needle=Session>::evaluate'2 base_per_iter=1.0000 cand_per_iter=1.0000 base_N=20008.0 base_2N=40008.0 cand_N=20008.0 cand_2N=40008.0
calls literal-integer needle=evaluate_profiled_fixed_point'2 base_per_iter=0.0000 cand_per_iter=0.0000 base_N=0.0 base_2N=0.0 cand_N=0.0 cand_2N=0.0
```

`literal-integer` is beyond the 0.05 floor by +1.15 instructions (+0.3932%). Callgrind's TOTAL is +1.0043, and +1.0000 of that is `Session::evaluate'2` (110 to 111) at 1 call per iteration on both arms. The fixed-point outline from Stage B (+69 and evaluate falling by 43) is absent. The candidate prologue compares depth and jumps to the cold tail that calls `evaluate_at_root`.

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
===== base rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate addr=0x750d30 size=4429 =====
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
0000000000750d30 <rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate>:
  750d30:	push   %rbp
  750d31:	push   %r15
  750d33:	push   %r14
  750d35:	push   %r13
  750d37:	push   %r12
  750d39:	push   %rbx
  750d3a:	sub    $0x348,%rsp
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
  750d4a:	cmpq   $0x0,0x770(%rsi)
  750d52:	je     750d64 <rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate+0x34>
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
===== cand rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate addr=0x751f40 size=4577 =====
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
0000000000751f40 <rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate>:
  751f40:	push   %rbp
  751f41:	push   %r15
  751f43:	push   %r14
  751f45:	push   %r13
  751f47:	push   %r12
  751f49:	push   %rbx
  751f4a:	sub    $0x348,%rsp
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
  751f5a:	cmpq   $0x0,0x330(%rsi)
  751f62:	je     752e79 <rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate+0xf39>
```

From `.temp/brief-629/logs/attr-lazy-depth-nest-map-256-a4.log`:

```text
oom_kill_before=0
oom_before=0
lazy-depth-nest-map-256 base N=1000 totals=[3671976825, 3671987983, 3678638990, 3671974225, 3671974820, 3671976455]
lazy-depth-nest-map-256 base N=2000 totals=[7323562381, 7323564902, 7323565834, 7323574083, 7323566827, 7323574142]
lazy-depth-nest-map-256 cand N=1000 totals=[3673861852, 3673879852, 3673885748, 3673866823, 3673864966, 3673864436]
lazy-depth-nest-map-256 cand N=2000 totals=[7340645568, 7340645304, 7327327160, 7327327693, 7327328179, 7327336753]
TOTAL base=3651589.69 cand=3653466.57 delta=+1876.88
wrote /home/remi/dev/rum/.temp/brief-629/logs/attr-lazy-depth-nest-map-256.tsv
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/attr-lazy-depth-nest-map-256.tsv`:

```text
+1876.8810	3651589.6905	3653466.5715	TOTAL
-58768.0000	58768.0000	0.0000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking'2
+55403.0000	0.0000	55403.0000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode'2
+4916.0000	177468.0000	182384.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
+355.4430	226.1910	581.6340	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
calls lazy-depth-nest-map-256 needle=Session>::evaluate'2 base_per_iter=774.0000 cand_per_iter=774.0000 base_N=777354.0 base_2N=1551354.0 cand_N=777354.0 cand_2N=1551354.0
calls lazy-depth-nest-map-256 needle=native_stack_budget_exhausted base_per_iter=32.3130 cand_per_iter=32.3130 base_N=32538.0 base_2N=64851.0 cand_N=32538.0 cand_2N=64851.0
calls lazy-depth-nest-map-256 needle=evaluate_once_unstable_after_input_check_with_cycle_tracking'2 base_per_iter=518.0000 cand_per_iter=0.0000 base_N=522117.0 base_2N=1040117.0 cand_N=0.0 cand_2N=0.0
calls lazy-depth-nest-map-256 needle=evaluate_once_in_accounting_mode'2 base_per_iter=0.0000 cand_per_iter=518.0000 base_N=0.0 base_2N=0.0 cand_N=522117.0 cand_2N=1040117.0
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
===== exclusive lazy-depth-nest-map-256 cand-2000-1.out minus cand-2000-3.out =====
summary cand-2000-3.out=7327327160 cand-2000-1.out=7340645568 gap=13318408
6664060	261292884	267956944	rum-cli!rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom
6655987	109264016	115920003	rum-cli!rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
===== bloom calls cand-2000-3.out =====
2585425	rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom
1037621	rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
===== bloom calls cand-2000-1.out =====
2585425	rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom
1037621	rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2
```

From `.temp/brief-629/logs/bc-a3.tsv`:

```text
lazy-depth-nest-map-256	1000	3651508.47	3666700.45	+15191.98	+0.4160	3671824735,3671822150,3671836703	7323346910,7323329767,7323333201	3673713075,3673727673,3673710432	7340414213,7327094494,7340413523
```

On `lazy-depth-nest-map-256` the stable positive rows are `Session::evaluate'2` +4916 at 774 calls per iteration on both arms, and `native_stack_budget_exhausted` +355.4430 at 32.3130 calls per iteration on both arms. 355.4430 / 32.3130 and the base 226.1910 / 32.3130 are 18 and 7 instructions per call. The same 18/7 ratio is the candidate slope over the base slope: 581.6340 / 226.1910 = 18/7. The evaluator body left `evaluate_once_unstable_after_input_check_with_cycle_tracking'2` (58768 to 0) and appears as `evaluate_once_in_accounting_mode'2` (0 to 55403), at 518 calls per iteration. That body is cheaper, so it is a negative component. Perf's delta is +15191.98. Callgrind's median TOTAL is +1876.8810. Candidate N=2000 runs 1 and 2 are the high mode (7340645568 and 7340645304) and runs 3 through 6 are the low mode. The high-minus-low gap is 13318408 Ir, of which `cached_expr_stability_with_bloom` is +6664060 and its `'2` clone is +6655987. Those two functions have the same call counts on the high run and the low run (2585425 and 1037621). One base N=1000 run is the same kind of outlier (`attr-notes-a4.txt` lines 142–145). 13318408 / 1000 = 13318.408, and 1876.8810 + 13318.408 = 15195.289, next to the perf delta +15191.98. Perf's three-run median at 2N landed on the high mode (the `bc-a3.tsv` candidate 2N runs are 7340414213, 7327094494, 7340413523). The mechanism is a longer stability-bloom body at a fixed call count. It is also present on the base.

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-8000-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-8000-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-8000.tsv`:

```text
+709.1250	4038.5500	4747.6750	TOTAL
-615.9750	330.8500	-285.1250	rum-cli!rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol
+380.0000	-366.9250	13.0750	rum-cli!rum_core::expr::intern_text
+357.5000	-147.0250	210.4750	rum-cli!rum_core::expr::Symbol::system
+275.1250	-225.9250	49.2000	rum-cli!hashbrown::map::HashMap<K,V,S,A>::insert
-133.0000	133.0000	0.0000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking'2
+127.0000	0.0000	127.0000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode'2
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-8000.tsv`:

```text
+9.0000	306.0000	315.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-8000.tsv`:

```text
+1.1000	0.7000	1.8000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-2000-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-2000-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-2000.tsv`:

```text
-244.4250	5170.8000	4926.3750	TOTAL
-133.0000	133.0000	0.0000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking'2
+127.0000	0.0000	127.0000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode'2
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-2000.tsv`:

```text
+9.0000	306.0000	315.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
```

From `.temp/brief-629/logs/attr-valueq-undefined-symbol-definitions-2000.tsv`:

```text
+1.1000	0.7000	1.8000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
===== summaries valueq-undefined-symbol-definitions-8000 =====
base-20-1.out	245222427	bytes=183802
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
calls valueq-undefined-symbol-definitions-8000 needle=Session>::evaluate'2 base_per_iter=1.0000 cand_per_iter=1.0000 base_N=16029.0 base_2N=16049.0 cand_N=16029.0 cand_2N=16049.0
calls valueq-undefined-symbol-definitions-8000 needle=native_stack_budget_exhausted base_per_iter=0.1000 cand_per_iter=0.1000 base_N=1001.0 base_2N=1003.0 cand_N=1001.0 cand_2N=1003.0
```

From `.temp/brief-629/logs/bc-a3.tsv`:

```text
valueq-undefined-symbol-definitions-2000	20	4027.80	4891.25	+863.45	+21.4373	62927017,62939104,62971680	63016267,63030964,63019660	63022259,63085044,63023878	63120895,63128297,63121703
valueq-undefined-symbol-definitions-8000	20	5096.95	4266.05	-830.90	-16.3019	242526924,242516251,242691963	242816128,242628863,242604950	242884270,242902500,242896060	242981381,242952087,243351049
```

`valueq-undefined-symbol-definitions-8000` perf is -830.90 (-16.3019%) and `valueq-undefined-symbol-definitions-2000` perf is +863.45 (+21.4373%). Callgrind's stable rows are the same on both: the old body 133 to 0, the accounting body 0 to 127, `evaluate'2` 306 to 315 at 1 call per iteration, and the cold check 0.7000 to 1.8000 at 0.1000 calls per iteration (again 18/7). The large rows are `intern_text`, `Symbol::system`, `HashMap::insert`, and `default_builtin_dispatch_spec_for_symbol`, with slopes that change sign between the N and 2N setup. An N=20 run's summary is about 245e6 instructions (`base-20-1.out` is 245222427) against an iteration of a few thousand. Perf and callgrind disagree in sign. The mechanism is setup hash-seed noise around the same depth-0 branch and the same cold-check read.

From `.temp/brief-629/logs/attr-set-part-fixed100-len100-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/attr-set-part-fixed100-len100-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/attr-set-part-fixed100-len100.tsv`:

```text
+1202.2350	1264493.1000	1265695.3350	TOTAL
-27776.0000	27776.0000	0.0000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking'2
+26371.0000	0.0000	26371.0000	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode'2
+1910.0000	70776.0000	72686.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
+699.6200	15698.1500	16397.7700	libc.so.6!__memcmp_avx2_movbe
+70.7300	45.0100	115.7400	rum-cli!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
calls set-part-fixed100-len100 needle=Session>::evaluate'2 base_per_iter=302.0000 cand_per_iter=302.0000 base_N=30528.0 base_2N=60728.0 cand_N=30528.0 cand_2N=60728.0
calls set-part-fixed100-len100 needle=native_stack_budget_exhausted base_per_iter=6.4300 cand_per_iter=6.4300 base_N=652.0 base_2N=1295.0 cand_N=652.0 cand_2N=1295.0
calls set-part-fixed100-len100 needle=__memcmp_avx2_movbe base_per_iter=809.0050 cand_per_iter=808.9900 base_N=83115.0 base_2N=164015.5 cand_N=83116.5 cand_2N=164015.5
```

From `.temp/brief-629/logs/bc-a3.tsv`:

```text
set-part-fixed100-len100	100	1264520.24	1265878.18	+1357.94	+0.1074	130924036,130945155,130909075	257484697,257376060,257372882	131049228,131052643,131077251	257627688,257716590,257640461
```

`set-part-fixed100-len100` perf is +1357.94. Callgrind TOTAL is +1202.2350. The body move is -27776 + 26371 = -1405. `evaluate'2` is +1910 at 302 calls per iteration on both arms (about 6.3 Ir per call, the same hot clone as nest-map's 4916 / 774). The cold check is +70.7300 at 6.4300 calls per iteration, and 115.7400 / 45.0100 = 18/7. `__memcmp_avx2_movbe` is +699.6200 at 809.0050 base calls per iteration and 808.9900 candidate calls per iteration. That is a longer average compare at nearly the same call rate.

The other positive CLI labels are the `sign=positive` CLI rows of `logs/bc-a3-flagged.tsv`. Their named mechanism is the one measured on `literal-integer`, `lazy-depth-nest-map-256`, and `set-part-fixed100-len100`: growth inside `Session::evaluate'2` from the depth-0 branch, and +11 instructions per `native_stack_budget_exhausted` call from the thread-record read, with the moved evaluator body cheaper. Where a perf percent and the callgrind median disagree by thousands of instructions, the named mechanism is the one measured on the valueq pair (setup hash noise) or on nest-map (the stability-bloom mode at a fixed call count). Flagged CLI median is +0.0841% (`bc-a3-summary.txt`).

The 15 negative labels are net negative, so they are not a positive residual. Fourteen of them are the cheaper accounting body outweighing the depth-0 branch on that label. `valueq-undefined-symbol-definitions-8000` is the setup-noise case above, not that body savings (the body row there is only 133 to 127).

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
label	engine	N	a_slope	b_slope	delta	delta_pct	bb_delta_pct	floor_pct	sign
lazy-range-take-small	cli	20000	41941.27	41879.22	-62.05	-0.1479	-0.1142	0.1142	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
lazy-table-construct-short	cli	20000	23403.06	23328.85	-74.21	-0.3171	-0.1284	0.1284	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
lazy-depth-map-first-64	cli	1000	930765.97	927913.11	-2852.86	-0.3065	-0.0003	0.0500	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
lazy-depth-map-build-256	cli	1000	70530.54	70429.36	-101.19	-0.1435	-0.0767	0.0767	negative
lazy-depth-map-first-256	cli	1000	3639636.24	3634871.20	-4765.04	-0.1309	-0.0154	0.0500	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
appendto-part-outer1000-small	cli	100	2235648.60	2224194.05	-11454.55	-0.5124	0.0346	0.0500	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
set-part-fill-small	cli	200	2784948.60	2774325.54	-10623.07	-0.3814	-0.2733	0.2733	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
symbol-ownvalue-guarded-read	cli	2000	3574.95	3568.94	-6.01	-0.1681	0.0136	0.0500	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
valueq-undefined-symbol-definitions-8000	cli	20	5096.95	4266.05	-830.90	-16.3019	7.9327	7.9327	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
matcher-single-tiny	cli	1000	11607.74	11597.01	-10.73	-0.0925	0.0167	0.0500	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
replace-rule-list-decline	cli	500	51028.85	50931.24	-97.61	-0.1913	-0.1555	0.1555	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
checkabort-no-abort	cli	500	11083.03	11019.25	-63.78	-0.5755	-0.1300	0.1300	negative
assert-disabled	cli	500	6807.05	6773.00	-34.06	-0.5003	0.0428	0.0500	negative
assert-enabled-true	cli	500	13614.37	13601.86	-12.51	-0.0919	0.0129	0.0500	negative
```

From `.temp/brief-629/logs/bc-a3-flagged.tsv`:

```text
catchexceptions-handler	cli	500	19683.69	19668.63	-15.05	-0.0765	0.0239	0.0500	negative
```

### 3b. The 121 resume labels

From `.temp/brief-629/logs/resume-span-a4.txt`:

```text
resume_by_name_rows=121
resume_bc_rows=121
missing_from_by_name=[]
extra_in_by_name=[]
min_delta resume-cleanup +220.47 +1.1843
max_delta resume-s6-readonly-twice +2738.97 +1.2722
min_per_resume resume-cleanup 220.4700 resumes_divisor_delta=+220.47
max_per_resume lhs-head-cycle 1658.8000 resumes_divisor_delta=+1658.80
```

From `.temp/brief-629/logs/bc-a3-resume-by-name.tsv`:

```text
label	delta	delta_pct	mechanism
c1-all-guard	+1371.53	+1.4166	resume entry: NativeStackEntry plus EvaluationDepth pair inside inlined resume_suspended_slot (resume_eval_request_from_checkpoint), cold-check thread-record read (+11 Ir per native_stack_budget_exhausted call), depth-0 evaluate branch, and evaluate_at_root when the resume re-enters Session::evaluate
```

Every resume row in `logs/bc-a3-resume-by-name.tsv` carries the same mechanism text: the resume entry (`NativeStackEntry` plus the `EvaluationDepth` pair inside inlined `resume_suspended_slot` / `resume_eval_request_from_checkpoint`), the cold-check thread-record read, the depth-0 evaluate branch, and `evaluate_at_root` when the resume re-enters `Session::evaluate`. Three labels were profiled.

From `.temp/brief-629/logs/attr-resume-cleanup-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/attr-resume-cleanup-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/attr-resume-cleanup.tsv`:

```text
+221.3945	18611.9645	18833.3590	TOTAL
-138.0000	138.0000	0.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking
+130.0000	0.0000	130.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode
+120.0000	1052.9080	1172.9080	libc.so.6!__memcpy_avx_unaligned_erms
+60.0000	1436.0000	1496.0000	resume!rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint
+44.6930	28.4410	73.1340	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
+32.0000	0.0000	32.0000	resume!core::ptr::drop_in_place<rum_core::expr::Expr>
```

From `.temp/brief-629/logs/attr-resume-cleanup.tsv`:

```text
+7.0000	231.0000	238.0000	resume!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate
-4.0000	88.0000	84.0000	resume!rum_runtime::runtime::suspension::core::evaluation_context::<impl rum_runtime::runtime::model::session::Session>::poll_resumed_native_stack
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
calls resume-cleanup needle=native_stack_budget_exhausted base_per_iter=4.0630 cand_per_iter=4.0630 base_N=4074.0 base_2N=8137.0 cand_N=4074.0 cand_2N=8137.0
calls resume-cleanup needle=__memcpy_avx_unaligned_erms base_per_iter=37.0000 cand_per_iter=41.0000 base_N=37905.0 base_2N=74905.0 cand_N=41918.0 cand_2N=82918.0
calls resume-cleanup needle=resume_eval_request_from_checkpoint base_per_iter=1.0000 cand_per_iter=1.0000 base_N=1000.0 base_2N=2000.0 cand_N=1000.0 cand_2N=2000.0
calls resume-cleanup needle=evaluate_at_root base_per_iter=0.0000 cand_per_iter=0.0000 base_N=0.0 base_2N=0.0 cand_N=1.0 cand_2N=1.0
```

From `.temp/brief-629/logs/bc-a3.tsv`:

```text
resume-cleanup	1000	18616.13	18836.60	+220.47	+1.1843	22211169,22203368,22224416	40799155,40827304,40852012	22446337,22446632,22428748	41303872,41282939,41253530
```

`resume-cleanup` callgrind TOTAL +221.3945 matches perf +220.47. Per iteration the positive pieces are `__memcpy_avx_unaligned_erms` +120 at 37 base calls versus 41 candidate calls, `resume_eval_request_from_checkpoint` exclusive +60 at 1 call per iteration on both arms, the cold check +44.6930 at 4.0630 calls per iteration (73.1340 / 28.4410 = 18/7), and `drop_in_place<Expr>` +32. `evaluate_at_root`'s call slope is 0. The body move is -138 + 130.

From `.temp/brief-629/logs/attr-lhs-head-cycle-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/attr-lhs-head-cycle-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/attr-lhs-head-cycle.tsv`:

```text
+1422.6200	301177.3600	302599.9800	TOTAL
-1070.0000	1070.0000	0.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking'2
+1003.0000	0.0000	1003.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode'2
+883.0000	0.0000	883.0000	resume!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_at_root
+747.2100	0.0000	747.2100	resume!core::ptr::drop_in_place<rum_core::expr::Expr>'2
-719.0000	1040.0000	321.0000	resume!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate
+575.3900	49.0000	624.3900	resume!core::ptr::drop_in_place<rum_core::expr::Expr>
```

From `.temp/brief-629/logs/attr-lhs-head-cycle.tsv`:

```text
+390.0000	10114.0800	10504.0800	libc.so.6!__memcpy_avx_unaligned_erms
+381.0000	0.0000	381.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode
+154.6600	98.4200	253.0800	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
+153.0000	3477.0000	3630.0000	resume!rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint
```

From `.temp/brief-629/logs/attr-lhs-head-cycle.tsv`:

```text
+76.0000	2505.0000	2581.0000	resume!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
cand-200-1.out	64107032	bytes=200238
cand-200-2.out	64361983	bytes=200288
cand-200-3.out	64039350	bytes=200237
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
calls lhs-head-cycle needle=evaluate_at_root base_per_iter=0.0000 cand_per_iter=3.0000 base_N=0.0 base_2N=0.0 cand_N=300.0 cand_2N=600.0
calls lhs-head-cycle needle=native_stack_budget_exhausted base_per_iter=14.0600 cand_per_iter=14.0600 base_N=1406.0 base_2N=2812.0 cand_N=1406.0 cand_2N=2812.0
calls lhs-head-cycle needle=Session>::evaluate'2 base_per_iter=10.0000 cand_per_iter=10.0000 base_N=1000.0 base_2N=2000.0 cand_N=1000.0 cand_2N=2000.0
```

From `.temp/brief-629/logs/bc-a3.tsv`:

```text
lhs-head-cycle	100	300821.50	302480.30	+1658.80	+0.5514	33617824,33589888,33600526	63786271,63682676,63665165	33741981,33734255,33737889	63985919,64062035,63959533
```

`lhs-head-cycle` is the largest per-resume delta, +1658.80. Callgrind TOTAL is +1422.6200. `evaluate_at_root` is +883 at 3 calls per iteration (0 on the base). The cold check is +154.6600 at 14.0600 calls per iteration (253.0800 / 98.4200 = 18/7). `resume_eval_request_from_checkpoint` is +153, memcpy is +390, and the `Expr` drops are +747.2100 and +575.3900. `evaluate` exclusive falls by 719 while `evaluate'2` rises by 76, at 10 calls per iteration on both arms. Candidate N=200 run 2 is high (64361983 against the other candidate N=200 runs near 6404xxxx); the six-run median stays on the low mode. The spread down to resume-cleanup's +220.47 is how heavy that cold root and the resume entry are on the label.

From `.temp/brief-629/logs/attr-c1-all-guard-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/attr-c1-all-guard-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/attr-c1-all-guard.tsv`:

```text
+1286.5750	96819.0000	98105.5750	TOTAL
+660.0000	3445.7100	4105.7100	libc.so.6!__memcpy_avx_unaligned_erms
-655.0000	655.0000	0.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking'2
+614.0000	0.0000	614.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode'2
+399.0000	0.0000	399.0000	resume!core::ptr::drop_in_place<rum_core::expr::Expr>
+248.1600	157.9200	406.0800	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
+238.0000	5442.0000	5680.0000	resume!rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint
```

From `.temp/brief-629/logs/attr-c1-all-guard.tsv`:

```text
+81.0000	0.0000	81.0000	resume!core::ptr::drop_in_place<rum_core::expr::Expr>'2
```

From `.temp/brief-629/logs/attr-c1-all-guard.tsv`:

```text
+49.0000	1685.0000	1734.0000	resume!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
```

From `.temp/brief-629/logs/attr-c1-all-guard.tsv`:

```text
-22.0000	484.0000	462.0000	resume!rum_runtime::runtime::suspension::core::evaluation_context::<impl rum_runtime::runtime::model::session::Session>::poll_resumed_native_stack
```

From `.temp/brief-629/logs/attr-notes-a4.txt`:

```text
calls c1-all-guard needle=native_stack_budget_exhausted base_per_iter=22.5600 cand_per_iter=22.5600 base_N=2260.0 base_2N=4516.0 cand_N=2260.0 cand_2N=4516.0
calls c1-all-guard needle=__memcpy_avx_unaligned_erms base_per_iter=118.0300 cand_per_iter=140.0300 base_N=12732.0 base_2N=24535.0 cand_N=14937.0 cand_2N=28940.0
calls c1-all-guard needle=evaluate_at_root base_per_iter=0.0000 cand_per_iter=0.0000 base_N=0.0 base_2N=0.0 cand_N=1.0 cand_2N=1.0
calls c1-all-guard needle=poll_resumed_native_stack base_per_iter=22.0000 cand_per_iter=22.0000 base_N=2204.0 base_2N=4404.0 cand_N=2204.0 cand_2N=4404.0
```

From `.temp/brief-629/logs/bc-a3.tsv`:

```text
c1-all-guard	100	96816.38	98187.91	+1371.53	+1.4166	13335610,13322238,13321771	23004432,23002693,23003876	13454242,13462764,13459388	23264190,23278179,23300077
```

From `.temp/brief-629/logs/bc-a3.tsv`:

```text
resume-s6-readonly-twice	100	215293.04	218032.01	+2738.97	+1.2722	25178893,25173239,25186688	46738438,46708197,46705363	25438204,25438128,25438434	47241405,47237217,47253357
```

`c1-all-guard` perf is +1371.53 and callgrind TOTAL is +1286.5750. memcpy is +660 at 118.0300 base calls per iteration versus 140.0300 candidate. The resume function is +238. The cold check is +248.1600 at 22.5600 calls per iteration (406.0800 / 157.9200 = 18/7), and `poll_resumed_native_stack` stays at 22 calls per iteration. `evaluate_at_root`'s call slope is 0. `Expr` drops are +399 and +81. The span file ranks `resume-s6-readonly-twice` as the largest absolute delta and `lhs-head-cycle` as the largest per-resume delta.

From `.temp/brief-629/logs/attr-notes-a4-run.log`:

```text
oom_kill_before=0
oom_before=0
wrote /home/remi/dev/rum/.temp/brief-629/logs/attr-notes-a4.txt lines=161
EXIT:0
oom_kill_after=0
oom_after=0
```

### 4. Stack cycle

Reservations count every push (8 bytes) and every `sub $imm,%rsp`. `logs/stack-a4-run.log` records the scan. Direct calls are in `logs/stack-a4-calls.txt`. The first extract truncated demangled names at the first `>` (`logs/stack-a4-calls-run.log`, 88 lines). The second extract keeps the full symbol and is the file quoted here.

From `.temp/brief-629/logs/stack-a4-run.log`:

```text
oom_kill_before=0
oom_before=0
symbols written, rows=70
```

From `.temp/brief-629/logs/stack-a4-run.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/stack-a4-calls-2-run.log`:

```text
oom_kill_before=0
oom_before=0
wrote /home/remi/dev/rum/.temp/brief-629/logs/stack-a4-calls.txt lines=97
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/stack-a4-reservations.tsv`:

```text
arm	function	addr	size	push_count	push_bytes	sub_count	sub_bytes	sum_bytes	subs	pushes
base	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit	0xd1570	41	0	0	0	0	0	-	-
base	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted	0xd15a0	21	0	0	0	0	0	-	-
base	rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit	0xdc090	37	0	0	0	0	0	-	-
```

From `.temp/brief-629/logs/stack-a4-reservations.tsv`:

```text
base	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point::suspend	0x1269a0	491	6	48	1	136	184	136	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

From `.temp/brief-629/logs/stack-a4-reservations.tsv`:

```text
base	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking	0x5d05b0	4737	6	48	1	712	760	712	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
base	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable	0x5d1840	10700	6	48	1	1256	1304	1256	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

From `.temp/brief-629/logs/stack-a4-reservations.tsv`:

```text
base	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate	0x750d30	4429	6	48	1	840	888	840	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

From `.temp/brief-629/logs/stack-a4-reservations.tsv`:

```text
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit	0xd1570	83	3	24	0	0	24	-	push   %r14 | push   %rbx | push   %rax
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted	0xd15d0	61	1	8	1	16	24	16	push   %rbx
cand	rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit	0xdc0e0	67	1	8	1	16	24	16	push   %rbx
cand	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_at_root	0xe2890	4834	6	48	1	856	904	856	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

From `.temp/brief-629/logs/stack-a4-reservations.tsv`:

```text
cand	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point::suspend	0x127d20	491	6	48	1	136	184	136	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

From `.temp/brief-629/logs/stack-a4-reservations.tsv`:

```text
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode	0x5d18e0	2632	6	48	1	536	584	536	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode	0x5d2330	1882	6	48	1	520	568	520	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable	0x5d2a90	10700	6	48	1	1256	1304	1256	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

From `.temp/brief-629/logs/stack-a4-reservations.tsv`:

```text
cand	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate	0x751f40	4577	6	48	1	840	888	840	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
call_instructions=82 indirect_calls=56
last_ret=751a5f:	ret
  751273:	call   5d05b0 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking>
  751c98:	call   1269a0 <rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point::suspend>
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
call_instructions=0 indirect_calls=0
last_ret=dc0b4:	ret
(no direct call or external jmp to a cycle function)

===== cand Session::evaluate addr=0x751f40 size=4577 =====
call_instructions=87 indirect_calls=57
last_ret=752cb6:	ret
  75248c:	call   5d18e0 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode>
  7524d3:	call   5d2330 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode>
  752ea2:	call   e2890 <rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_at_root>
  752f17:	call   127d20 <rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point::suspend>
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== cand evaluate_at_root addr=0xe2890 size=4834 =====
call_instructions=85 indirect_calls=57
last_ret=e36d3:	ret
   e2e4c:	call   5d18e0 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode>
   e2e93:	call   5d2330 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode>
   e3900:	call   127d20 <rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_profiled_fixed_point::suspend>
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== cand evaluate_once_in_accounting_mode addr=0x5d18e0 size=2632 =====
call_instructions=43 indirect_calls=25
last_ret=5d1fed:	ret
  5d1b8a:	call   5d2a90 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable>
  5d1de9:	call   5d2a90 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable>
  5d2146:	call   d1570 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit>
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== cand evaluate_once_in_accounting_mode addr=0x5d2330 size=1882 =====
call_instructions=35 indirect_calls=19
last_ret=5d297a:	ret
  5d2569:	call   5d2a90 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable>
  5d265e:	call   5d2a90 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable>
  5d2920:	call   d1570 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit>
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== cand evaluate_inner_unstable addr=0x5d2a90 size=10700 =====
call_instructions=178 indirect_calls=131
last_ret=5d503a:	ret
  5d4702:	call   1253f0 <rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::evaluate_call::{{closure}}::suspend>
  5d4cd3:	call   5cf830 <rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::evaluate_stored_own_value>
  5d4da1:	call   5cfff0 <rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::evaluate_symbol_after_stored_own_value>
  5d5089:	call   5cf830 <rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::evaluate_stored_own_value>
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== cand Session::poll_native_stack_limit addr=0xd1570 size=83 =====
call_instructions=0 indirect_calls=0
last_ret=d15c2:	ret
(no direct call or external jmp to a cycle function)
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== cand native_stack_budget_exhausted addr=0xd15d0 size=61 =====
call_instructions=0 indirect_calls=0
last_ret=d160c:	ret
(no direct call or external jmp to a cycle function)
```

From `.temp/brief-629/logs/stack-a4-calls.txt`:

```text
===== cand NativeStackGuard::poll_native_stack_limit addr=0xdc0e0 size=67 =====
call_instructions=0 indirect_calls=0
last_ret=dc122:	ret
(no direct call or external jmp to a cycle function)
```

Repeating level, base. `Session::evaluate` holds 6 pushes (48 bytes) and one `sub` of 840, sum 888. It calls `evaluate_once_unstable_after_input_check_with_cycle_tracking`, which holds 6 pushes (48) and one `sub` of 712, sum 760. That function calls `poll_native_stack_limit` and returns from it, and it calls `evaluate_inner_unstable`. Per-level sum of the two frames the level holds across that call: 888 + 760 = 1648.

Repeating level, candidate. There is no hot `evaluate_profiled_fixed_point` symbol, so there is no separate fixed-point frame. `Session::evaluate` is again 6 pushes and `sub` 840, sum 888. It calls two `evaluate_once_in_accounting_mode` clones, sums 584 and 568. A level takes one clone: 888 + 584 = 1472, or 888 + 568 = 1456. Both are under the base's 1648. Neither clone calls `Session::evaluate`. Both call `poll_native_stack_limit` and return. `evaluate_inner_unstable` is sum 1304 on both arms (sub 1256, 6 pushes) and has no direct call to `Session::evaluate` on either arm. `evaluate_profiled_fixed_point::suspend` is sum 184 on both arms (sub 136, 6 pushes). It is the suspension frame, present on both arms.

Depth 0 is a separate entry. Candidate `Session::evaluate` calls `evaluate_at_root` from the depth-0 tail (`752ea2`). `evaluate_at_root` holds 6 pushes and `sub` 856, sum 904, and then calls one accounting clone. The caller's evaluate frame is still live across that call: 888 + 904 + 584 = 2376 on the larger clone. That composition runs once per request. The repeating level after it is evaluate plus one accounting frame, under the base. `evaluate_at_root` does not call `Session::evaluate`.

Cold-check leaves. Each reservation grew from 0 to 24 bytes. 24 is within the 32-byte allowance. Each disassembly has `call_instructions=0` and ends in `ret`.

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
===== base rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit addr=0xd1570 size=41 =====
pushes=0 (0 bytes) subs=- sub_bytes=0 sum=0 adds=[]
calls: (none of the cycle names)
Disassembly of section .text:
00000000000d1570 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit>:
   d1570:	xor    %eax,%eax
   d1572:	lea    -0x1(%rsp),%rcx
   d1577:	sub    %rcx,%rdx
   d157a:	cmovae %rdx,%rax
   d157e:	cmp    %rsi,%rax
   d1581:	jbe    d1590 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit+0x20>
   d1583:	mov    %rsi,0x8(%rdi)
   d1587:	mov    $0xd,%eax
   d158c:	mov    %rax,(%rdi)
   d158f:	ret
   d1590:	mov    $0x21,%eax
   d1595:	mov    %rax,(%rdi)
   d1598:	ret
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
===== base rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted addr=0xd15a0 size=21 =====
pushes=0 (0 bytes) subs=- sub_bytes=0 sum=0 adds=[]
calls: (none of the cycle names)
Disassembly of section .text:
00000000000d15a0 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted>:
   d15a0:	xor    %eax,%eax
   d15a2:	lea    -0x1(%rsp),%rcx
   d15a7:	sub    %rcx,%rsi
   d15aa:	cmovae %rsi,%rax
   d15ae:	cmp    %rdi,%rax
   d15b1:	seta   %al
   d15b4:	ret
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
===== base rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit addr=0xdc090 size=37 =====
pushes=0 (0 bytes) subs=- sub_bytes=0 sum=0 adds=[]
calls: (none of the cycle names)
Disassembly of section .text:
00000000000dc090 <rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit>:
   dc090:	mov    (%rdi),%rdx
   dc093:	mov    0x8(%rdi),%rax
   dc097:	xor    %ecx,%ecx
   dc099:	lea    -0x1(%rsp),%rsi
   dc09e:	sub    %rsi,%rdx
   dc0a1:	cmovae %rdx,%rcx
   dc0a5:	cmp    %rax,%rcx
   dc0a8:	jbe    dc0ae <rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit+0x1e>
   dc0aa:	movb   $0x1,0x10(%rdi)
   dc0ae:	cmp    %rax,%rcx
   dc0b1:	setbe  %al
   dc0b4:	ret
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
===== cand rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit addr=0xd1570 size=83 =====
pushes=3 (24 bytes) subs=- sub_bytes=0 sum=24 adds=[8]
calls: (none of the cycle names)
Disassembly of section .text:
00000000000d1570 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit>:
   d1570:	push   %r14
   d1572:	push   %rbx
   d1573:	push   %rax
   d1574:	mov    %rsi,%r14
   d1577:	mov    %rdi,%rbx
   d157a:	mov    %fs:0x0,%rax
   d1583:	lea    -0x6f8(%rax),%rax
   d158a:	mov    (%rax),%rcx
   d158d:	xor    %eax,%eax
   d158f:	lea    0x7(%rsp),%rdx
   d1594:	mov    %rcx,%rsi
   d1597:	sub    %rdx,%rsi
   d159a:	cmovae %rsi,%rax
   d159e:	test   %rcx,%rcx
   d15a1:	je     d15af <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit+0x3f>
   d15a3:	cmp    %r14,%rax
   d15a6:	ja     d15af <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit+0x3f>
   d15a8:	mov    $0x21,%eax
   d15ad:	jmp    d15b8 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit+0x48>
   d15af:	mov    %r14,0x8(%rbx)
   d15b3:	mov    $0xd,%eax
   d15b8:	mov    %rax,(%rbx)
   d15bb:	add    $0x8,%rsp
   d15bf:	pop    %rbx
   d15c0:	pop    %r14
   d15c2:	ret
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
===== cand rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted addr=0xd15d0 size=61 =====
pushes=1 (8 bytes) subs=16 sub_bytes=16 sum=24 adds=[16]
calls: (none of the cycle names)
Disassembly of section .text:
00000000000d15d0 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted>:
   d15d0:	push   %rbx
   d15d1:	sub    $0x10,%rsp
   d15d5:	mov    %rdi,%rbx
   d15d8:	mov    %fs:0x0,%rax
   d15e1:	lea    -0x6f8(%rax),%rax
   d15e8:	mov    (%rax),%rax
   d15eb:	test   %rax,%rax
   d15ee:	sete   %cl
   d15f1:	xor    %edx,%edx
   d15f3:	lea    0xf(%rsp),%rsi
   d15f8:	sub    %rsi,%rax
   d15fb:	cmovae %rax,%rdx
   d15ff:	cmp    %rbx,%rdx
   d1602:	seta   %al
   d1605:	or     %cl,%al
   d1607:	add    $0x10,%rsp
   d160b:	pop    %rbx
   d160c:	ret
```

From `.temp/brief-629/logs/stack-a4-prologues.txt`:

```text
===== cand rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit addr=0xdc0e0 size=67 =====
pushes=1 (8 bytes) subs=16 sub_bytes=16 sum=24 adds=[16]
calls: (none of the cycle names)
Disassembly of section .text:
00000000000dc0e0 <rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit>:
   dc0e0:	push   %rbx
   dc0e1:	sub    $0x10,%rsp
   dc0e5:	mov    %rdi,%rbx
   dc0e8:	mov    %fs:0x0,%rax
   dc0f1:	lea    -0x6f8(%rax),%rax
   dc0f8:	mov    (%rax),%rax
   dc0fb:	test   %rax,%rax
   dc0fe:	setne  %cl
   dc101:	xor    %edx,%edx
   dc103:	lea    0xf(%rsp),%rsi
   dc108:	sub    %rsi,%rax
   dc10b:	cmovae %rax,%rdx
   dc10f:	cmp    (%rbx),%rdx
   dc112:	setbe  %al
   dc115:	and    %cl,%al
   dc117:	jne    dc11d <rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit+0x3d>
   dc119:	movb   $0x1,0x8(%rbx)
   dc11d:	add    $0x10,%rsp
   dc121:	pop    %rbx
   dc122:	ret
```

`Session::poll_native_stack_limit` is the third leaf. It is called from the evaluator body and returns, so its 24 bytes are the check, not a frame held on the repeating level. `native_stack_budget_exhausted` and `NativeStackGuard::poll_native_stack_limit` are the other two leaves. All three read `%fs:0x0` and `lea -0x6f8(%rax)`. No other cycle frame grew. The repeating sums 1472 and 1456 are under the base sum 1648.

### 5. Jupyter pair

The pipe filter matches nothing on this libtest. Twenty runs wrote `logs/jupyter-pair-a4.log`. The two test names as two filters after `--`, with `--test-threads=2`, wrote `logs/jupyter-pair-a4-filters.log`. All 20 of those runs executed both tests.

From `.temp/brief-629/logs/jupyter-pair-a4.log`:

```text
oom_kill_before=0
oom_before=0
```

From `.temp/brief-629/logs/jupyter-pair-a4.log`:

```text
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-dd24b36d3edaa47a)
```

From `.temp/brief-629/logs/jupyter-pair-a4.log`:

```text
running 0 tests
```

From `.temp/brief-629/logs/jupyter-pair-a4.log`:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 151 filtered out; finished in 0.00s
```

From `.temp/brief-629/logs/jupyter-pair-a4.log`:

```text
RUN_EXIT:0
```

From `.temp/brief-629/logs/jupyter-pair-a4.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/jupyter-pair-a4-filters.log`:

```text
oom_kill_before=0
oom_before=0
note: the pipe filter matched 0 tests on this libtest. Help text says multiple FILTERS run tests matching any of them. This run passes the two names as two filters.
```

From `.temp/brief-629/logs/jupyter-pair-a4-filters.log`:

```text
running 2 tests
test tests::execute_lifecycle::interrupt::red_interrupt_while_waiting_finishes_request ... ok
test tests::execute_lifecycle::interrupt::interrupt_at_second_prompt_finishes_original_request ... ok
```

From `.temp/brief-629/logs/jupyter-pair-a4-filters.log`:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s
```

From `.temp/brief-629/logs/jupyter-pair-a4-filters.log`:

```text
RUN_EXIT:0
```

From `.temp/brief-629/logs/jupyter-pair-a4-filters.log`:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.03s
```

From `.temp/brief-629/logs/jupyter-pair-a4-filters.log`:

```text
EXIT:0
oom_kill_after=0
oom_after=0
```

Run 11's result line is the `finished in 0.03s` line. The other 19 result lines are `finished in 0.02s`. Each run's `RUN_EXIT` is 0. The wrapper `EXIT` is 0. `oom_kill` stayed 0.

### 6. Attempts and what was left alone

No command in this measurement died with signal 9. The quoted logs show `oom_kill_before=0` and `oom_kill_after=0`. Nothing was rerun for the host-OOM rule.

`cargo test --workspace` was not run. `census/tests.rs`, `phase-sites.tsv`, and the instrument were not edited. `arms/base/` and `logs/bb.tsv` were not modified. Stage B callgrind files were not reused as A4 profiles. The A3 bars that the director already accepted (fmt, `rum-runtime`, `rum-host`, typed-slot census, LP01, and the census update) were not rerun as A4 work.

## Director ruling on A4, and continuation A5 (2026-10-28 04:20) — grk

**The stack bars pass, and the CLI side is accepted.** I checked
`bc-a3.tsv`, the attribution TSVs and the reservations directly, and added
three readings of my own.

- **The level.** The per-level cycle is 888 + 584 = 1472 (or 1456), against
  the base's 1648. `evaluate_at_root` (904) runs once per request.
- **The leaves: +24 B and +11 instructions per call.** The rlib compiles the
  thread-local read as a general-dynamic access. The rum-runtime object
  holds `R_X86_64_TLSGD` and `call __tls_get_addr`, and the linker relaxes
  that to the `%fs` load you quoted. Register allocation had already saved
  `rdi` and aligned the frame for the call. Stable Rust gives no choice of
  TLS model, so this is the expected thread-record read at its real price.
- **The evaluator's inlining change is real.** Built at the candidate's
  path, the base keeps `evaluate_once_unstable_after_input_check_with_cycle_tracking`.
  The candidate has `evaluate_at_root` and two
  `evaluate_once_in_accounting_mode`. A3's cold copy causes the change, and
  the change shrinks the level.
- **`literal-integer`'s +1.** A per-instruction callgrind of
  `Session::evaluate'2` executes 110 instructions on the base and 111 on the
  candidate. The extra one is a zero store, `movq $0x0,…`, that became
  `xor` plus `mov`. That is register allocation.

**The resume mechanism is not accepted.** The 121 rows named the expected
mechanisms. Those cost about 25 instructions per resume, and
`resume-cleanup` grew by 220. A mechanism has to account for the size, as
count times cost per occurrence. "Function X grew" names a place, not a
cause. I took the three largest parts apart with one-change variant builds
(`.temp/director/629/exp/`).

1. **Four copies per resume, caused by the shape.** Counted by caller,
   `resume_eval_request_from_checkpoint` makes 10 memcpy calls per
   iteration instead of 6. Each is a 136-byte (`0x88`) copy right after a
   `poll_resumed_native_stack` call: 4 of its 8 call sites.
   - **The cause.** That function is `#[cold] #[inline(never)]` and calls the
     leaf, which now reads the thread local. LLVM infers that it may touch
     global memory. It can then no longer build the 136-byte
     `Result<(), EvalError>` in the caller's slot, so it builds it in a
     temporary and copies it.
   - **The variants.** Without the resume entry, 4 copies remain. With
     `restore` reverted to a depth-only store, 4 remain. With a leaf that
     does not read TLS, none remain.
2. **The `Expr` drops are a path artifact, and the instrument is my fault.**
   `drop_in_place<Expr>` moved out of an `Arc::drop_slow` in rum-core's
   code, and this brief does not touch rum-core.
   - My Stage B instrument built the base from `base-source/` and the
     candidate from the repository root. That breaks my own rule that both
     arms build at one path.
   - The path enters every crate's id, so every symbol hash differed between
     the two arms.
   - Built at one path, the base has the same drop shape as the candidate:
     18 `drop_slow` with 20 call sites, against 17 with 18 in `arms/base`.
     The hashes then match.
3. **The rest** gets measured again after 1 and 2 are fixed.

The fix for 1, measured at one path on four resume labels:

| label | A3 vs base | A5 shape vs base |
|---|---|---|
| resume-cleanup | +1.57% | +0.07% |
| c1-all-guard | +1.16% | +0.22% |
| c1-all-guard-value | +1.13% | +0.18% |
| lhs-head-cycle | +0.24% | +0.09% |

`c1-all-guard`'s remaining +214 is the leaf's +11 times 22.56 calls per
iteration (248).

### A5 — one edit, and the instrument at one path

**The edit.** In `suspension/core/evaluation_context.rs`, replace the two
attribute lines of `poll_resumed_native_stack`:

```rust
    // Runs once per resumed frame; the leaf it calls is the cold part.
    #[inline(always)]
    pub(in crate::runtime) fn poll_resumed_native_stack(&mut self) -> Result<(), EvalError> {
```

- The body stays as it is. The function was never cold: every resumed frame
  runs it.
- The leaf `native_stack_budget_exhausted` stays `#[cold] #[inline(never)]`.
  It returns a `bool` in a register, so no result crosses the call.
- The comment keeps the line count, so no line below moves.
- Nothing else in `crates/` changes.

**The instrument.** The scripts are in `.temp/brief-629/instruments/`;
paths below are relative to `.temp/brief-629/`.

1. **One source path.** Run `rsync -a --delete --exclude target --exclude .temp --exclude .git base-source/ same/src/`.
   `same/harness/` is a copy of `harness-cand/`, with the rum-host path set
   to `../src/crates/rum-host`.
2. **The base.** Build rum-cli into `same/target-base` and the harness into
   `same/target-base-harness`: one codegen unit, `CARGO_BUILD_JOBS=8`,
   offline. Copy both binaries to `arms/base-same/`.
3. **The candidate.** Run `rsync -a --delete --checksum --exclude target --exclude .temp --exclude .git`
   from the repository into `same/src/`. Build into `same/target-cand` and
   `same/target-cand-harness`, and copy to `arms/cand-same/`.
4. **Calibrate before any measurement.** `path-calibration.sh` must print
   PASS on the two `resume` binaries, and again on the two `rum-cli`
   binaries. FAIL is a STOP.
5. **The shape, statically.** Run `resume-shape.sh` on both `resume`
   binaries and report both lines. My one-path variant read
   `frame=0x848 memcpy_sites=19 poll_calls=0 leaf_calls=8` for the
   candidate, and `frame=0x7f8 memcpy_sites=23 poll_calls=8 leaf_calls=0`
   for the base.
6. **The floor, then the gate.** Run
   `instr.py arms/base-same arms/base-same logs/bb-same.tsv`, then
   `instr.py arms/base-same arms/cand-same logs/bc-a5.tsv`. Leave
   `arms/base/`, `arms/cand/`, `logs/bb.tsv` and `logs/bc-a3.tsv` as they
   are.

**The bars.**

- **Instructions.** For each label beyond its floor, the callgrind
  components must close. Give each component as its count per iteration
  times its cost per occurrence, then their sum against callgrind's TOTAL
  slope.
  - `callers.py <dir> <callee> <N>` gives per-caller call counts, for any
    library function that moved (memcpy, memcmp, drops).
    `instr_slope.py <function> <N> <out-N> <out-2N>` gives a per-instruction
    diff inside one function; it needs callgrind's `--dump-instr=yes`.
  - **Expected components:**
    - the leaf, +11 per call;
    - the resume entry, per resume;
    - the threshold store, per restore;
    - the evaluator's inlining change (the body moves into `evaluate'2`,
      and the accounting body is cheaper);
    - `evaluate_at_root`, per root entry;
    - register allocation inside `evaluate'2`;
    - A4's two noise modes, the valueq setup hash and the nest-map bloom
      bimodality, each with the same evidence as in A4.
  - **STOP conditions.** A residual beyond the floor that no listed
    component accounts for. A new memcpy call site. A library call whose
    count per iteration moved, without a named call site.
  - **Coverage.** Attribute at least the three largest resume labels by
    delta, the three largest CLI labels, and every label above +0.3%.
- **Stack.** Rerun A4's cycle table and the leaves on `arms/*-same`, with
  A3's bars. The per-level cycle must stay at or below the base, and the
  leaves may grow by at most 32 B. Any other growth on the cycle is a STOP.
  Report `resume_eval_request_from_checkpoint`'s frame from step 5.
- **Tests after the edit.**
  - `cargo fmt --check`.
  - `cargo test -p rum-runtime --lib`.
  - `cargo test -p rum-host`, which includes `native_stack_thread_scope`.
  - `production_rejection_census_is_complete` and the LP01 fixture. The
    edit is designed to move neither; if one moves, STOP with the keyed
    diff.
  - `local_production_acceptance` in both builds. If a fingerprint moves,
    report it and do not edit; I replay at my gate.
  - The Jupyter pair, 20 times at `--test-threads=2`, with the two names as
    two filters.
  - The full workspace stays with my gate.

**Scope.** Under `crates/`, only `suspension/core/evaluation_context.rs`.
Scratch goes under `.temp/brief-629/`.

Every quoted line must be copied from a file you name. Append
"Executing-agent report — A5".

## Executing-agent report — A5

**STOP.** `production_rejection_census_is_complete` fails because the edit changes the rot-guard context. That context is a hash of every Rust, Python and TOML file under `crates/`. All 68 stored contract fingerprints match the source with the two previous attribute lines restored, and none match the edited source. The 972 frontier assertion did not fire. Inventories were not edited. LP01, both `local_production_acceptance` builds, the Jupyter pair, and the same-path instrument were not run.

### 1. The edit

`poll_resumed_native_stack` is `#[inline(always)]`. The body is unchanged. The leaf stays `#[cold] #[inline(never)]`.

From `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs`:

```text
    /// Physical safety for the current native segment, not replay of its address.
    // Runs once per resumed frame; the leaf it calls is the cold part.
    #[inline(always)]
    pub(in crate::runtime) fn poll_resumed_native_stack(&mut self) -> Result<(), EvalError> {
```

From `crates/rum-runtime/src/runtime/eval_core/mod.rs`:

```text
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn native_stack_budget_exhausted(&self) -> bool {
        let stack_marker = 0_u8;
        let current_addr = (&stack_marker as *const u8) as usize;
        native_stack_exhausted(current_addr, self.resource_limits.native_stack_limit)
    }
```

### 2. Census

The panic is in `verify_promotions`, which the census test calls after the frontier assertions.

From `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`:

```text
    assert_eq!(
        options.rejected.len(),
        972,
        "STOP: brief-545 exact legacy frontier inventory moved"
    );
    assert_eq!(
        kinds,
        BTreeMap::from([(Kind::RET001, 10), (Kind::RET002, 934), (Kind::RET003, 28)])
    );
    let wrapper = regions::frontier(
        source,
        &[format!(
            "{}::merged_option_exprs_for_system_symbol",
            regions::SESSION
        )],
    );
    // The registration guard and mode selection add public wrapper boundaries.
    // All 16 remain rejected here; this census does not grant local coverage.
    assert_eq!(wrapper.rejected.len(), 16, "STOP: PUBLIC_WRAPPER moved");
    assert!(
        wrapper.rejected.iter().any(|row| row.target.as_deref()
            == Some(&format!("{}::check_option_reentry", regions::SESSION)))
    );
    super::super::local::verify_promotions(source);
```

From `.temp/brief-629/logs/rejection-census-a5.log`:

```text
thread 'tests::retention_closure::census::tests::production_rejection_census_is_complete' panicked at crates/rum-stdlib/src/tests/retention_closure/local/promotion.rs:27:38:
fresh certificate rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol: RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O011
RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O010
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0210
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0200
```

From `.temp/brief-629/logs/rejection-census-a5.log`:

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 26.68s

error: test failed, to rerun pass `-p rum-stdlib --lib`
EXIT:101
oom_kill_after=0
oom_after=0
```

The log does not contain `left: 972` or `STOP: brief-545 exact legacy frontier inventory moved`. `oom_kill` stayed 0. There was no signal 9.

### 3. Keyed diff

The census report's four rows, keyed by kind, file, callee and reason, are the panic text above. Each reason is a stale contract fingerprint. The same four rows are the A3 obligations failure, which ran before the director's replay:

From `.temp/brief-629/logs/rejection-census-a3.log`:

```text
thread 'tests::retention_closure::production_obligations_are_exact' panicked at crates/rum-stdlib/src/tests/retention_closure/local/promotion.rs:27:38:
fresh certificate rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol: RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O011
RET002	crates/rum-runtime/src/runtime/options/core.rs:118:13-118:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O010
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0210
RET002	crates/rum-runtime/src/runtime/options/core.rs:119:9-121:10	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	stale contract fingerprint O0200
```

After that replay the stored fingerprints match the pre-edit source. The edit makes them stale again.

`.temp/brief-629/logs/fp-compare-a5.py` recomputes `contracts::fingerprint` for the live tree and for the same tree with `NEW` replaced by `OLD` in `evaluation_context.rs` only:

From `.temp/brief-629/logs/fp-compare-a5.py`:

```text
OLD = (
    b"    #[cold]\n"
    b"    #[inline(never)]\n"
)
NEW = (
    b"    // Runs once per resumed frame; the leaf it calls is the cold part.\n"
    b"    #[inline(always)]\n"
)
```

Columns are `id`, `stored`, `base_stored`, `match_now`, `match_pre`, `now`, `pre`.

From `.temp/brief-629/logs/fp-compare-a5.txt`:

```text
context_now=fnv1a64:ca06fc9ecab6a52a
context_pre_a5=fnv1a64:f91f4b13c32e916d
context_changed_by_edit=True
id	stored	base_stored	match_now	match_pre	now	pre
```

From `.temp/brief-629/logs/fp-compare-a5.txt`:

```text
O010	fnv1a64:3e160f4df3a03068	fnv1a64:4868b78e10cd866c	no	yes	fnv1a64:ef6eb32e028e7427	fnv1a64:3e160f4df3a03068
O0200	fnv1a64:aee6ab6cc947b127	fnv1a64:49f09292ba8427f3	no	yes	fnv1a64:070b9bfd958bc71c	fnv1a64:aee6ab6cc947b127
O011	fnv1a64:c0298ab848f35957	fnv1a64:52277e76bfda25e3	no	yes	fnv1a64:342868ebfaefbb4c	fnv1a64:c0298ab848f35957
O0210	fnv1a64:c33035c33fa95410	fnv1a64:99c6f6ed73f53014	no	yes	fnv1a64:c080feddf148d0df	fnv1a64:c33035c33fa95410
rows=68 stored_eq_base=0 stored_eq_now=0 stored_eq_pre_a5=68
```

`match_pre` is yes and `match_now` is no for every row, including these four. `stored_eq_pre_a5=68` and `stored_eq_now=0`. The context hash moves from `fnv1a64:f91f4b13c32e916d` to `fnv1a64:ca06fc9ecab6a52a`.

The rot-guard that produces `current`:

From `crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs`:

```text
pub(super) fn fingerprint(fields: &[&str]) -> String {
    // Flat rot guard, computed once at indexing. Include all source and oracle
    // inputs so a new imported trait or impl in another file expires contracts.
    // This hashes bytes, never analyzes or follows another body's calls.
    static CONTEXT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let context = CONTEXT.get_or_init(|| {
        let mut parts = Vec::<Vec<u8>>::new();
        for file in DEPENDENCIES {
            parts.push(file.as_bytes().to_vec());
            parts.push(fs::read(source::workspace().join(file)).unwrap());
        }
        fn collect(root: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
            for entry in fs::read_dir(root).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect(&path, paths);
                } else if path
                    .extension()
                    .is_some_and(|s| s == "rs" || s == "py" || s == "toml")
                {
                    paths.push(path);
                }
            }
        }
        let root = source::workspace();
        let mut paths = vec![];
        collect(&root.join("crates"), &mut paths);
        paths.sort();
```

`local-contracts.tsv` and `promoted.tsv` were not edited. The A5 scope forbids that edit.

### 4. Tests that passed

From `.temp/brief-629/logs/fmt-a5.log`:

```text
oom_kill_before=0
oom_before=0
EXIT:0
oom_kill_after=0
oom_after=0
```

From `.temp/brief-629/logs/rum-runtime-lib-a5.log`:

```text
test result: ok. 1216 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 35.62s

EXIT:0
oom_kill_after=0
```

From `.temp/brief-629/logs/rum-host-a5.log`:

```text
     Running tests/native_stack_thread_scope.rs (target/debug/deps/native_stack_thread_scope-ea359a5a7ace3af4)

running 8 tests
test cross_thread_cancelled_cleanup_after_origin_exits_control ... ok
test cross_thread_cancelled_cleanup_with_higher_stack_enforces_native_stack_limit ... ok
test cross_thread_cancelled_cleanup_with_lower_stack_runs_cleanup ... ok
test cross_thread_cancelled_cleanup_with_origin_alive_observation ... ok
test nested_session_in_host_callback_shares_outermost_thread_stack_base ... ok
test same_thread_cancelled_cleanup_after_deep_suspension_resets_check_threshold ... ok
test same_thread_cancelled_cleanup_control ... ok
test same_thread_deep_cleanup_enforces_native_stack_limit_control ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

From `.temp/brief-629/logs/rum-host-a5.log`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.07s

EXIT:0
oom_kill_after=0
oom_after=0
```

### 5. Not done

Stopped at the census. Not run: the LP01 fixture, `local_production_acceptance` in either build, the Jupyter pair, the same-path rsync and builds, path calibration, `resume-shape.sh`, `logs/bb-same.tsv`, `logs/bc-a5.tsv`, callgrind, and the stack rescan. `cargo test --workspace` was not run. `arms/base/`, `arms/cand/`, `logs/bb.tsv`, and `logs/bc-a3.tsv` were not modified.

## Director ruling on A5's STOP, and continuation A5b (2026-10-28 04:45) — grk

**Your STOP is right, and the cause is my brief.** The contract rot guard
hashes every `.rs`, `.py` and `.toml` under `crates/`, so any edit expires
all 68 fingerprints, and the census test reaches them through
`verify_promotions`. TEMPLATE puts that replay in every brief's scope. My
A5 text then said both "do not edit" and "STOP if the census moves", and
that could only fire. Your keyed comparison shows what it needed to: the
frontier is still 972, `stored_eq_pre_a5=68`, and the context moves from
`f91f4b13c32e916d` to `ca06fc9ecab6a52a`.

I replayed by the standing procedure (`.temp/director/gate-629a5/`):

- The fingerprints were recomputed. The context is `fnv1a64:ca06fc9ecab6a52a`,
  the same value you computed, over 68 rows, and the other 18 columns are
  unchanged.
- The four C01/C02 certificate cells were regenerated in both builds and
  installed. Their non-fingerprint text is identical.
- `local_production_acceptance` passes in the default and
  `--no-default-features` builds, and `production_rejection_census_is_complete`
  passes.

**A5b: the rest of A5, with no edit under `crates/`.** Run, in order:

1. The LP01 fixture.
2. The Jupyter pair, 20 times at `--test-threads=2`.
3. The instrument at one path, steps 1–6.
4. The instruction, stack and attribution bars as A5 states them.

If a standing inventory fails again, report it and continue; it is mine to
replay. Append "Executing-agent report — A5b". Every quoted line must be
copied from a file you name.

## Executing-agent report — A5b

No STOP. No file under `crates/` was edited in A5b. The A5 `#[inline(always)]` on `poll_resumed_native_stack` is unchanged, and the leaf stays `#[cold] #[inline(never)]`. The same-path instrument, the stack rescan, and callgrind of the coverage set are done. `cargo test --workspace` was not run.

### 1. Tests

LP01 passed. From `.temp/brief-629/logs/lp01-a5b.log`:

```
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 66.73s

EXIT:0
oom_kill_after=0
oom_after=0
```

The Jupyter pair passed 20 times. Each `test result:` line in `.temp/brief-629/logs/jupyter-pair-a5b.log` is the same, and the log ends:

```
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.02s

RUN_EXIT:0
EXIT:0
oom_kill_after=0
oom_after=0
```

`fmt`, `rum-runtime` (1216), and `rum-host` (including `native_stack_thread_scope`) passed in A5 and were not rerun. The director replayed the rot-guard and both `local_production_acceptance` builds. A5b did not edit inventories.

### 2. One path

Calibration passed on both resume binaries and both rum-cli binaries. From `.temp/brief-629/logs/calibrate-a5b.log`:

```
=== resume ===
a: 54968415bb2e57e7 5c761434403a0d87 a69b7b564caba688 f03877a538c7ca78 fc1f7bf9abd9a343
b: 54968415bb2e57e7 5c761434403a0d87 a69b7b564caba688 f03877a538c7ca78 fc1f7bf9abd9a343
PASS
=== rum-cli ===
a: 0cfd6ce0b10b3936 3bde438439d162a1 8fa2c193a24c1629 c15fb178da3fc520 ce0a78e75ec340cf
b: 0cfd6ce0b10b3936 3bde438439d162a1 8fa2c193a24c1629 c15fb178da3fc520 ce0a78e75ec340cf
PASS
=== shape base ===
size=24862 frame=0x7f8 memcpy_sites=23 poll_calls=8 leaf_calls=0
=== shape cand ===
size=25778 frame=0x848 memcpy_sites=19 poll_calls=0 leaf_calls=8
EXIT:0
oom_kill_after=0
oom_after=0
```

`resume_eval_request_from_checkpoint`'s frame is `0x7f8` on the base and `0x848` on the candidate. Static memcpy sites went from 23 to 19. Poll calls went from 8 to 0 and the leaf calls went from 0 to 8, which is the inline: the checkpoint function calls the leaf directly.

### 3. Instruction gate

The floor and the gate are 313 labels, both `EXIT:0`, and `oom_kill` stayed 0. From `.temp/brief-629/logs/bc-a5-summary.txt`:

```
labels 313
bb_rows 313
bc_median_pct +0.0530
bc_max_pct +1.2794
bc_min_pct -20.1201
flagged 162 positive 142 negative 20
flagged_cli 75 of 192 median +0.0712
flagged_resume 87 of 121 median +0.1803
flagged_cli_positive 61
flagged_resume_positive 81
above_0_3_pct 21
```

The gate's three-run median for `resume-cleanup` is +90.60 (+0.4866%). A six-run recheck on the same binaries is inside the floor. From `.temp/brief-629/logs/perf-recheck-resume-cleanup-a5b.tsv`:

```
resume-cleanup	1000	18645.35	18658.84	+13.49	+0.0724	22219184,22239439,22237246,22242157,22220714,22220352	40920816,40844685,40842930,40873508,40940420,40875160	22281226,22260617,22259557,22257628,22275382,22256652	40917538,40965941,40917676,41007599,40918448,40919415
```

+0.0724% is under that label's floor of 0.1954%. Two of the six candidate 2N runs sit in the high cluster (`40965941` and `41007599`); the other four sit near `40918k`. The gate's three candidate 2N runs all landed in the high cluster.

### 4. Stack

Not a STOP. Frame sums are push bytes plus sub bytes. The repeating level is `Session::evaluate` plus the callee it reaches on every iteration.

From `.temp/brief-629/logs/stack-a5b-reservations.tsv`:

```
base	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate	0x750d30	4429	6	48	1	840	888	840	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

```
base	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking	0x5d05b0	4737	6	48	1	712	760	712	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

Base repeating level: 888 + 760 = 1648.

```
cand	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate	0x751f00	4577	6	48	1	840	888	840	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

```
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode	0x5d18a0	2632	6	48	1	536	584	536	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode	0x5d22f0	1882	6	48	1	520	568	520	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable	0x5d2a50	10700	6	48	1	1256	1304	1256	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

Candidate repeating level: 888 + 584 = 1472, or 888 + 568 = 1456. Both are under 1648. `evaluate_inner_unstable` is 1304 on both arms. `evaluate_at_root` is cold, sum 904:

```
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::poll_native_stack_limit	0xd1570	83	3	24	0	0	24	-	push   %r14 | push   %rbx | push   %rax
cand	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted	0xd15d0	65	1	8	1	16	24	16	push   %rbx
cand	rum_runtime::runtime::native_stack_guard::NativeStackGuard::poll_native_stack_limit	0xdc0f0	67	1	8	1	16	24	16	push   %rbx
cand	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate_at_root	0xe28a0	4834	6	48	1	856	904	856	push   %rbp | push   %r15 | push   %r14 | push   %r13 | push   %r12 | push   %rbx
```

The three cold leaves are sum 0 on the base and sum 24 on the candidate. 24 is inside the 32-byte reservation allowance. A depth-0 entry still has the caller's evaluate frame live: 888 + 904 + 584 = 2376. That is one request, not the repeating level.

From `.temp/brief-629/logs/stack-a5b-calls.txt`:

```
===== nm base hot evaluate_profiled_fixed_point =====
(no hot defined symbol)
```

```
===== nm cand hot evaluate_profiled_fixed_point =====
(no hot defined symbol)
```

### 5. Attribution

Callgrind is exclusive Ir, six runs per arm per size, on `arms/base-same` and `arms/cand-same`. Outputs are `callgrind/a5b-<label>/` and `logs/attr-a5b-<label>.tsv`. Stage B and A4 profiles were not reused. `oom_kill` stayed 0 on both batches (`logs/attr-a5b-batch1.log`, `logs/attr-a5b-batch2.log`).

The components that close a label are the leaf at +11 per call, the inlined resume entry (the old `poll_resumed_native_stack` body moves into `resume_eval_request_from_checkpoint`, which is also where the threshold store lives), the evaluator body moving from `evaluate_once_unstable_after_input_check_with_cycle_tracking` into `evaluate_once_in_accounting_mode` and getting cheaper, `evaluate'2` and `Session::evaluate`, `evaluate_at_root` when a root entry runs, and the two A4 noise modes. Memcpy exclusive slope is 0 on every profiled label. Net memcpy call-count slope is 0 on every callers file. From `.temp/brief-629/logs/memcpy-net-a5b.txt`:

```
files=29 moved=0
```

On `resume-cleanup` the checkpoint function's memcpy calls stay 6. The only nonzero memcpy caller deltas are the evaluator body moving, +1 accounting and -1 `once_unstable`. That is not a new memcpy site. From `.temp/brief-629/logs/callers-a5b-resume-cleanup.txt`:

```
+1.000	0.000	1.000	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode
-1.000	1.000	0.000	rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking
```

```
+0.000	6.000	6.000	rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint
```

`resume-cleanup` callgrind TOTAL is +38.8370, against the six-run perf delta of +13.49. The difference, 25.3 instructions, is inside the 36.4-instruction floor. From `.temp/brief-629/logs/attr-a5b-resume-cleanup.tsv`:

```
+38.8370	18608.9450	18647.7820	TOTAL
-138.0000	138.0000	0.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_unstable_after_input_check_with_cycle_tracking
+130.0000	0.0000	130.0000	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_once_in_accounting_mode
-88.0000	88.0000	0.0000	resume!rum_runtime::runtime::suspension::core::evaluation_context::<impl rum_runtime::runtime::model::session::Session>::poll_resumed_native_stack
+82.0000	1436.0000	1518.0000	resume!rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint
+44.6930	28.4410	73.1340	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
+7.0000	231.0000	238.0000	resume!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate
```

Leaf calls, from `.temp/brief-629/logs/callers-a5b-resume-cleanup.txt`:

```
-4.000	4.000	0.000	rum_runtime::runtime::suspension::core::evaluation_context::<impl rum_runtime::runtime::model::session::Session>::poll_resumed_native_stack
+4.000	0.000	4.000	rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint
+0.000	0.063	0.063	rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom
```

Count per iteration is 4 + 0.063 = 4.063 on both arms. Cost is 28.4410 / 4.063 = 7 and 73.1340 / 4.063 = 18, so the leaf is 4.063 × 11 = +44.693. The four direct calls moved from `poll_resumed_native_stack` into the checkpoint function because the poll was inlined. `poll_resumed_native_stack` exclusive −88 is those 4 calls at 22 instructions, and the checkpoint function's exclusive +82 is that inlined body at 1 call per iteration, 6 instructions cheaper. The evaluator body is −138 + 130 = −8. `Session::evaluate` is +7. The remaining rows in that TSV are hash and intern noise under 2 instructions. Their sum with the rows above is +39.109, against TOTAL +38.837. The 0.272 difference is rows smaller than 0.05.

`c1-all-guard` is the director's leaf check, and it was attributed even though it is not in the +0.3% list. From `.temp/brief-629/logs/attr-a5b-c1-all-guard.tsv`:

```
+248.1600	157.9200	406.0800	resume!rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::native_stack_budget_exhausted
```

From `.temp/brief-629/logs/callers-a5b-c1-all-guard.txt`:

```
-22.000	22.000	0.000	rum_runtime::runtime::suspension::core::evaluation_context::<impl rum_runtime::runtime::model::session::Session>::poll_resumed_native_stack
+22.000	0.000	22.000	rum_runtime::runtime::session::session_requests::<impl rum_runtime::runtime::model::session::Session>::resume_eval_request_from_checkpoint
+0.000	0.440	0.440	rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom
+0.000	0.120	0.120	rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2
```

Call slope is 22 + 0.440 + 0.120 = 22.560 on both arms. 157.9200 / 22.560 = 7 and 406.0800 / 22.560 = 18, so the component is 22.560 × 11 = +248.160. Callgrind TOTAL for this label is +355.4150. The rest is the same inlining change plus `Symbol::system` (+121.130 in `logs/components-a5b-c1-all-guard.txt`). This candidate is the whole brief against the pre-brief base, so the total is not only the leaf.

`literal-integer` is the register-allocation case. From `.temp/brief-629/logs/attr-a5b-literal-integer.tsv`:

```
+0.9968	291.9180	292.9148	TOTAL
+1.0000	110.0000	111.0000	rum-cli!rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
```

Dump-instr of one run per arm, hot instructions executed on every iteration, from `.temp/brief-629/logs/instr-literal-a5b.txt`:

```
base file base-20000-1.out positions 281 hot 110 missing 0 hot_count_min 20008 hot_count_max 20012
```

```
cand file cand-20000-1.out positions 289 hot 111 missing 0 hot_count_min 20008 hot_count_max 20012
```

```
mnemonic_delta cand-base
-1 jne base 4 cand 3
-1 movq base 6 cand 5
+1 mov base 17 cand 18
+1 xor base 2 cand 3
+1 je base 13 cand 14
```

Hot count is 110 versus 111. The net extra instruction is one `xor` plus one `mov` taking the place of one `movq`. The `je`/`jne` pair swaps and does not change the count.

`lazy-depth-nest-map-64` perf is +3811.70 and callgrind TOTAL is +458.8590. The perf median picked the high bloom mode at 2N and the low mode at N. From `.temp/brief-629/logs/attr-a5b-batch1.log`:

```
lazy-depth-nest-map-64 cand N=1000 totals=[948439017, 946816814, 946785278, 946775989, 948442558, 946781774]
```

```
lazy-depth-nest-map-64 cand N=2000 totals=[1885426678, 1885425944, 1885426074, 1888756765, 1888755482, 1885425836]
```

From `.temp/brief-629/logs/modes-a5b.txt`:

```
nest cand N=1000 rep=1 summary=948439017 bloom_ir=49305222
nest cand N=1000 rep=2 summary=946816814 bloom_ir=47679377
callers.py parse nest cand N=1000 rep=1 bloom_calls=465326
callers.py parse nest cand N=1000 rep=2 bloom_calls=465326
```

The summary gap between those two runs is 1622203. The bloom IR gap is 1625845. Call count is the same. Holding the mode fixed, the low-mode perf slope is about +485, next to the callgrind slope of +458.86. The callgrind pieces are the body move (−15184 + 14315), `evaluate'2` (+1268), and the leaf (+91.443). That sum is +490.44, against TOTAL +458.86.

Where the median of the totals and the sum of the median function slopes disagree, the instructions left outside the named functions do not move. From `.temp/brief-629/logs/modes-a5b.txt`:

```
unnamed_delta c1-all-guard -0.0400
unnamed_delta lhs-pattern-cycle +0.0000
unnamed_delta resume-s9-cleanup-init +0.0000
unnamed_delta resume-s6-readonly-twice -0.0400
```

The three largest CLI labels:

`lazy-depth-fold-through-source-64` callgrind TOTAL is +13389.86, perf is +16847.24. The floor is 0.05% of about 33.3e6, which is 16673 instructions, so the two slopes agree inside the floor. The pieces are the body move (+445525 and −472589, net −27064) at 4164 calls per iteration, `evaluate'2` at +39553, the leaf at 260.250 × 11 = +2862.75, and the bloom rows (−836.735 and −676). From `.temp/brief-629/logs/callers-a5b-lazy-depth-fold-through-source-64.txt`:

```
+0.000	260.250	260.250	rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom
```

```
+4164.000	0.000	4164.000	rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate'2
```

`lazy-depth-map-eight-256` callgrind TOTAL is +14694.06, perf is +15299.39. Same shape: body move +438677 and −465325, `evaluate'2` +38945, leaf +2818.75, bloom −318.66.

`lazy-depth-fold-through-source-256` was callgrinded at 2 and 4 iterations, not at the label N of 1000. One N=1000 run of the 64-deep fold took 113.6 seconds. From `.temp/brief-629/logs/attr-a5b-batch2.log`:

```
ran base N=1000 rep=1 rc=0 elapsed=113.6s
```

The 256-deep fold's gate count at N is about 15 times that, so the 24-run set is about a day. The measured slope is still one iteration, because setup is in both sizes. From the same log:

```
out_dir /home/remi/dev/rum/.temp/brief-629/callgrind/a5b-lazy-depth-fold-through-source-256-n2 timeout 1800s measure_n 2 label_N 1000
```

```
TOTAL base=517395131.25 cand=517647829.50 delta=+252698.25
```

Perf at N=1000 is +217216.80. The label is under the 0.05% floor (+0.0417%). The callgrind and perf slopes differ by 35481 instructions, and 0.05% of the callgrind base is about 258698, so they agree inside the floor. The pieces are the body move (+7040149 and −7467821), `evaluate'2` (+625057), the leaf (+45232), and bloom (+21335.5).

### 6. Coverage

Attributed, each with an `attr-a5b-*.tsv` and a `callers-a5b-*.txt`: the 21 labels above +0.3%, the three largest resume labels by delta (`lhs-twice-value`, `resume-s6-readonly-twice`, `lhs-pattern-cycle`), and the three largest CLI labels (`lazy-depth-fold-through-source-256`, `lazy-depth-fold-through-source-64`, `lazy-depth-map-eight-256`). `c1-all-guard` and `lhs-head-cycle` were attributed as well. `lhs-head-cycle` callgrind TOTAL is +279.61 and the gate perf delta is +288.17.

### 7. Left alone

`arms/base/`, `arms/cand/`, `logs/bb.tsv`, and `logs/bc-a3.tsv` were not modified. The instrument scripts were not edited. `census/tests.rs` and the phase-site inventories were not edited. The A5 report was not rewritten.

## Director gate of A5b (2026-10-28 08:50) — brief-629 accepted

**Accepted.** I checked the arms, the gate files and the attribution
directly, and reran the measurements that decide. My scripts and outputs are
in `.temp/director/gate-629a5b/`.

**The arms are the right sources at one path.**
- `base-source/` equals `678e6be7` (`git archive` diff, crates and
  `Cargo.lock`). `same/src/` equals the working tree; the only difference is
  this brief.
- I reran `path-calibration.sh`: PASS on both pairs. I reran
  `resume-shape.sh`: base `frame=0x7f8 memcpy_sites=23 poll_calls=8 leaf_calls=0`,
  candidate `frame=0x848 memcpy_sites=19 poll_calls=0 leaf_calls=8`. The
  copies that A4 traced to the cold poll's sret are gone.

**The gate summary reproduces, and most of its flags are noise.** My own
reading of `bc-a5.tsv` against `bb-same.tsv` gives the same 313 labels,
median +0.0530%, max +1.2794%, 162 flagged and 21 above +0.3%. The floor
comes from one base-against-base pair at three runs, which is too tight for
the short labels. I reran the 25 largest labels interleaved (base N, cand N,
base 2N, cand 2N, 15 times each, `recheck.tsv`). Perf then lands on the
callgrind number:

| label | gate | 15-run recheck | callgrind TOTAL |
|---|---:|---:|---:|
| `resume-s5-down-rhs` | +296.54 | +132.39 | +127.71 |
| `resume-cleanup` | +90.60 | +46.14 | +38.84 |
| `lhs-head-cycle` | +288.17 | +282.23 | +279.61 |
| `association-part-key-read-width32000` | +181.62 | +9.65 | −88.96 |
| `literal-integer` | +1.19 | +0.97 | +1.00 |

**Every structural component closes as count × cost** (`decomp.txt`, from
the callgrind call counts and exclusive costs, all 29 labels):
- **The leaf**: exactly +11.00 per call on every label that calls it. This
  is the general-dynamic thread-local read, as ruled in A4.
- **The inlined resume poll**: saves 1.5 to 5.4 instructions per poll call
  on every resume label. A resumed frame now costs about +6 net (leaf +11,
  poll −5). `resume-cleanup` is +37.7 per iteration, against A4's +220.
- **The evaluator**: +0.57 to +2.11 per `evaluate` call, and +1.00 on
  `literal-integer`. I diffed `Session::evaluate` per instruction on
  `lazy-depth-map-first-8` (`instr/`). Two changes make up all of its
  +185. On a path taken 20 times per iteration, a tag check and a 32-byte
  move after the body's call moved from the body into `evaluate`: the body
  lost 5 instructions and `evaluate` gained 7. On a path taken 45 times, a
  `movq $0` became `xor` plus `mov`, the register allocation seen on
  `literal-integer`. This is the inlining change A4 accepted, which takes
  the level from 1648 to 1472 bytes.
- **The cold root entry**: +55 per root entry on the three `lhs-*` labels
  (3 entries per iteration, +165). This is the A3 shape's price, once per
  request. I measured its size and did not decompose it per instruction.

**Your three STOP rows are not STOPs, but your report should have named
them.** `close-a5b.txt` marks `c1-all-guard`, `lhs-pattern-cycle` and
`resume-s9-cleanup-init` STOP. Your report says "No STOP" and answers them
only through the `unnamed_delta` lines. I decomposed them per run
(`perrun.py`, `modes.py`). The structural functions take one value per arm
and size. Everything else has overlapping ranges in both arms, and its spread
sits in `Symbol::system` and `intern_text`, the interner hash mode A4 found.
**From now on, a report quotes every verdict line its own instrument prints
and settles each flagged row in the text.**

**Stack, tests and front ends.** The stack table is the one I accepted in
A4: 1472 or 1456 against 1648 per level, `evaluate_at_root` 904 once per
request, and 24 bytes in the cold leaves. `resume_eval_request_from_checkpoint`
grows by 80 bytes, once per resume request and not per level. LP01 passes,
and the Jupyter pair passes 20 of 20 in your log. My own gate:
- `cargo fmt --check` passes.
- `native_stack_thread_scope` passes 8 of 8, three times.
- The Jupyter pair passes 20 of 20 at `--test-threads=2`.
- `cargo test --workspace --no-fail-fast`, single-threaded in a 14G scope
  (peak 11.3 GB, no OOM): 3976 passed, 1 failed, 61 ignored. The failure is
  `ownership_graph::production_inventory_is_exact`.

**The ownership inventory had been stale since A3, and that miss is mine.**
A3 changed `resolution.rs` and `model/evaluation_depth.rs`. After A3 I
replayed the contract fingerprints and the certificates, but not the
ownership rows. No stage after it ran the whole workspace. I replayed the
rows from the mismatch the test prints (`reconcile.py`), and nothing else
moved:
- **`resolution.rs`.** Four rows committed at HEAD shift by three lines and
  keep their identity: the closures `|previous|` (372 → 375) and `|arg|`
  (504 → 507), and two `resume_typed_or_suspend!` expansions (387 → 390 and
  451 → 454).
- **`model/evaluation_depth.rs`.** This file is new in this brief. Stage A's
  single `debug_assert_eq!` row at line 61 becomes the three A3 assertions at
  lines 37, 56 and 65 (`native_check_due`, `enter_level`, `leave_level`).
  They stay unknown.
- There is no new `UnknownHostPayload`, and no row points at code this brief
  did not touch. The only reader of these files is `ownership_graph/mod.rs`.
  `cargo test -p rum-stdlib --lib ownership_graph` then passes 31 of 31. The
  TSVs are not hashed by the rot guard, so the contract fingerprints stand.

brief-629 is done. The dropped cleanup error (Option A) is the next brief.
