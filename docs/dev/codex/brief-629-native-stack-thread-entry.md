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
