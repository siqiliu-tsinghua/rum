# brief-635 — Profiling tests lease the process-wide epoch

**Shape: implementation.** Worker: kimi. The second of brief-633's three
implementation steps; brief-634 (artifact constructors) has landed
(`467ff97a`).

## Goal

At HEAD, profiling tests in one process share mutable state: the logical
epoch, its 16 MiB budget, and the lexical flags. Two of them reset each
other's epoch, or charge the one diagnostic ceiling together. An ordinary
test on another thread records into a profiling test's epoch while the
flags are on.

After this brief, every test that touches that state holds one process
lease, `TestLease`, and the type system enforces it. A thread that has not
registered with the current lease cannot construct, borrow, open, charge or
reset the leased epoch. It also cannot reset the lexical counters.

The process-wide collector stays one epoch per process. Production builds
compile none of this.

## Context pointers

- Read `AGENTS.md`, `docs/dev/profiling.md` (the epoch contract), and
  brief-633's director gate (end of
  `docs/dev/codex/brief-633-test-isolation-design.md`).
- Design evidence under `.temp/brief-633/`:
  - the draft `drafts/brief-profiler-test-lease.md`;
  - the diffs `logs/lease/profiling-c2.diff`,
    `logs/lease/profile-queue-tests-c2.diff` and `logs/c3/lease-public.diff`;
  - the integrated tree `tree-final/`;
  - C4′ (`logs/c3/no_cross_attribution.log`, `same_key_additivity.log`,
    `owner_scoped_abandonment.log`);
  - the TLS proof `probe/tls-scope/` and `logs/c3/tls-scope.log`.
- HEAD sites:
  - `crates/rum-runtime/src/runtime/profiling/logical/epoch.rs`: `Epoch::new`
    `:57`, `storage` `:78`, `open` `:89`, `test_limit` `:246`, `current`
    `:251`, `reset` `:267`;
  - `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`: the public
    test hooks `:97-205` (`logical_profile_test_reset` `:181`) and
    `Call::start` `:208`;
  - `crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs:1-30`
    (`Enabled`, which swaps both flags);
  - `crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs:285-292`
    (it swaps `LEXICAL_PROFILE_ACTIVE`);
  - `crates/rum-runtime/src/runtime/profiling/reset.rs:3-100`
    (`reset_lexical_profile_counters`, which also stores
    `LEXICAL_PROFILE_ACTIVE = 1`);
  - **the two unleased reset entries the draft missed:**
    - `RumProfileReset[]`, through
      `crates/rum-runtime/src/runtime/debug_tools/basic.rs:119`. Any
      in-process test can evaluate it, for example
      `crates/rum-runtime/src/runtime/tests/debug_tools.rs:322`.
    - `rum_runtime::test_logical_profile_reset()`
      (`crates/rum-runtime/src/lib.rs:16-20` →
      `crates/rum-runtime/src/runtime/mod.rs:118`). It is called by
      `crates/rum-stdlib/tests/suspension_oracle/descriptors.rs:144`.

## Scope

- **MAY modify:**
  - `crates/rum-runtime/src/runtime/profiling/**`;
  - `crates/rum-runtime/src/runtime/mod.rs` and `crates/rum-runtime/src/lib.rs`
    (the test-only hooks only);
  - `crates/rum-runtime/src/runtime/debug_tools/basic.rs`, only the
    test-cfg refusal (below);
  - every test file that calls a hook whose signature changes, only so it
    takes or holds the lease: `crates/rum-runtime/src/**/tests/**`,
    `crates/rum-host/tests/logical_profile_lifecycle.rs`,
    `crates/rum-host/tests/logical_profile_abandonment.rs` and
    `crates/rum-stdlib/tests/suspension_oracle/**`;
  - the standing inventories, through their replays (`TEMPLATE.md` Scope).
- **MUST NOT:**
  - change owner routing, `PROCESS_LIMIT`, epoch row semantics,
    `totals::settle`, `abandon_owner`'s owner match, or any production
    branch;
  - change `lexical_profile_enabled`, `lexical_profile_configured` or
    `lexical_profile_active` (brief-633 calibrated that a check there adds
    two cfg rows to the pinned 1040-row retention frontier);
  - add a dependency or `unsafe`.

## Target shape

The shape is fixed. A departure from it is a STOP.

The new module, `profiling/lease.rs`, is compiled only under
`cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))`.
Call this predicate **the test cfg**. The module takes the shape of
`.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/lease.rs`:

```rust
static TEST_LEASE: Mutex<()> = Mutex::new(());
static NEXT_GEN: AtomicU64 = AtomicU64::new(1);
static CURRENT_GEN: AtomicU64 = AtomicU64::new(0);
thread_local! { static REGISTERED_GEN: Cell<u64> = const { Cell::new(0) }; }

pub(in crate::runtime) struct TestLease { _guard: MutexGuard<'static, ()>, generation: u64 }
pub(in crate::runtime) fn lease() -> TestLease;          // lock, bump, store CURRENT_GEN, register caller
impl TestLease { pub(in crate::runtime) fn allow_current_thread(&self); }
impl Drop for TestLease { /* CURRENT_GEN = 0; caller's REGISTERED_GEN = 0 */ }
#[inline] pub(in crate::runtime) fn blocks_unregistered_thread() -> bool;
```

### The epoch entries

These are as in tree-final:

- `Epoch::new` becomes private `new_unleased`.
- Tests construct an epoch with `Epoch::for_test(&TestLease, limit)`.
- `epoch::reset` becomes private `reset_unleased`, reached in two ways:
  - `reset_production(ProductionReset)`, where `ProductionReset` has a
    private field and is minted only in `profiling/reset.rs`;
  - `reset_for_lease(&TestLease)`, in the test cfg.
- `test_limit` takes `&TestLease`.
- Under the test cfg, each of these returns early with `None` or no effect
  when `blocks_unregistered_thread()` is true, before any lock or mutation:
  - `Epoch::storage`, before `retire(DiagnosticCap)`;
  - `Epoch::open`, before the data lock;
  - `epoch::current`, under the slot lock, both before constructing an epoch
    and before cloning an existing one;
  - `Call::start`.

### `Enabled` and the flag writers

- `test_support::enabled()` takes the lease first, and keeps it in `Enabled`
  until after `Drop` restores both flags.
- `Enabled::lease(&self) -> &TestLease` is the only way a test reaches its
  lease.
- `queue_boundaries.rs:285-292`'s swap of `LEXICAL_PROFILE_ACTIVE` happens
  while that test holds its `Enabled`.

### Every reset entry refuses an unregistered thread

This is the draft's gap. `ProductionReset`'s private field stops a test from
naming `reset_production`. It does not stop a test from reaching it through
`RumProfileReset[]` or `test_logical_profile_reset()`.

So, under the test cfg, `reset_lexical_profile_counters` begins:

```rust
#[cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))]
if crate::runtime::profiling::lease::blocks_unregistered_thread() {
    return;
}
```

The check is the function's first statement, before `lexical_profile_configured()`.
It leaves the counters, the epoch and `LEXICAL_PROFILE_ACTIVE` untouched.
Production builds compile none of it.

### The public test hooks

- `pub struct LogicalProfileTestLease { inner: TestLease }`, with its field
  private, and `Session::logical_profile_test_acquire() -> LogicalProfileTestLease`.
- `Session::logical_profile_test_reset(&LogicalProfileTestLease)`. The
  zero-argument form no longer exists.
- `rum_runtime::test_logical_profile_reset(&LogicalProfileTestLease)`, and
  `runtime::logical_profile_reset` takes the inner lease. The suspension
  oracle (`descriptors.rs:144`) acquires the lease before that reset, and
  holds it through the subject and the snapshot that follows.
- The rum-host lifecycle and abandonment children keep their process
  isolation, and each child holds the wrapper for its whole test.

### Threads a leased test spawns

A test that runs a profiled Session on a second thread calls
`lease.allow_current_thread()` on that thread first. The metadata test
(`logical_thread_metadata_stays_charged_until_thread_exit`) then joins that
scoped worker explicitly before it samples `PROCESS_BYTES`.
`.temp/brief-633/logs/c3/tls-scope.log` proves `thread::scope`'s implicit
wait returns before the worker's TLS destructors run. The before/after
assertion stays as it is.

## The 62 B1 collision rows

`.temp/brief-633/inventory/b1-final.tsv` has 62 rows marked `collision`:

- 58 are lexical receipts in `profiling/state.rs`;
- 4 are the epoch, budget and flags;
- the 6 `RUM_PROFILE_REPR` counters in `rum-core/src/profiling.rs` are only
  written when that variable is set, and no test sets it.

For each of the 62, list in `sections` (tag `b1`):

- every test that reads it in-process;
- which protection covers that reader. Accept only these three:
  - **child**: the reader runs in a child process;
  - **lease**: the reader holds the lease, and every write an unregistered
    thread can reach is refused under the test cfg;
  - **none**: no in-process test reads the row.

A row with an in-process reader and neither protection is a **STOP**. Report
the row, its reader, and the writer an unregistered thread can reach. Do not
add a check to a recording path on your own: brief-633 measured that such a
check can move the pinned frontier.

## Must-not-change beside the cost bars

- **Flag off.** `lexical_profile_enabled` stays byte-identical, one relaxed
  load. There is no lease load, mutex or branch on that path in any build.
- **Production builds.** No `lease` symbol and no added branch.
  - Prove it on the release `rum-cli` and on the perf-panel harness with
    `nm -C`.
  - Then run the feature tree,
    `cargo tree -e features -p rum-cli --offline`, saved with its log.
  - The director's gate runs `run 635 --perf resume,cli`.
- **The epoch contract** (`docs/dev/profiling.md`, brief-633 amendment 1):
  - One epoch per process, with rows keyed by `AggregateKey`.
  - Same-key sessions add. Owner routes and limits abandonment.
  - `PROCESS_LIMIT` stays at 16 MiB.
  - C4′'s three tests stay green, unmutated.
- **No new warning.** brief-633's tree-final printed `field generation is
  never read` at `lease.rs:19` in one cfg. Fix it in the shape, without
  `#[allow]`.

## Falsifiers

Each runs in its own scope, `scripts/memory-scope.sh b635-<label>-<n> 14G`,
with `--offline` and `CARGO_BUILD_JOBS=8`. Copy HEAD's
`crates/ Cargo.toml Cargo.lock rust-toolchain.toml scripts/run-bounded.py`
to `.temp/brief-635/head/` before your first edit. Never use git.

1. **Chain pair at two threads, bound 60 s.**
   - HEAD is red: the director's calibration below is 9/10 red, and
     brief-633 run 1 was 19/20 red.
   - The change is green 20/20.

   ```
   cargo test -p rum-runtime --lib --offline -- --test-threads=2 \
     logical_source_chain_drops_on_two_mib logical_record_chain_drops_on_two_mib
   ```

2. **Unregistered span** and 3. **unregistered production reset** land
   together as a new module, `profiling/tests/owner_filter.rs`, registered
   with `mod owner_filter;` in `profiling/tests/mod.rs`. The code is fixed.
   Both tests compile unchanged before and after the lease, because
   `test_support::enabled()` takes the lease inside itself. The director
   ran this exact file at HEAD (calibration below).

   ```rust
   //! Threads that never registered with the profiling test lease.

   use crate::runtime::Session;
   use crate::runtime::profiling::logical::test_support;

   fn trivial_expr(session: &mut Session) -> crate::runtime::Expr {
       let syntax = rum_syntax::parse_expr("1").expect("parse 1");
       session.lower_syntax_expr(&syntax).expect("lower 1")
   }

   #[test]
   fn unregistered_thread_span_does_not_record_while_a_profiling_guard_is_held() {
       let _enabled = test_support::enabled();
       let before = Session::logical_profile_test_rows();
       std::thread::spawn(|| {
           let mut session = Session::new();
           let expr = trivial_expr(&mut session);
           let _ = session.logical_profile_test_span("unregistered-span", expr);
       })
       .join()
       .expect("unregistered span");
       assert_eq!(
           Session::logical_profile_test_rows(),
           before,
           "unregistered thread recorded into the held epoch"
       );
   }

   #[test]
   fn unregistered_thread_profile_reset_leaves_the_held_epoch() {
       let _enabled = test_support::enabled();
       let mut session = Session::new();
       let expr = trivial_expr(&mut session);
       session
           .logical_profile_test_span("leased-span", expr)
           .expect("leased span");
       let held = test_support::epoch();
       let before = Session::logical_profile_test_rows();
       assert!(!before.is_empty(), "the leased span recorded no row");
       std::thread::spawn(|| {
           let mut session = Session::new();
           let _ = session.eval_text("RumProfileReset[]");
       })
       .join()
       .expect("unregistered reset");
       assert!(held.is_valid(), "unregistered reset invalidated the held epoch");
       assert_eq!(
           Session::logical_profile_test_rows(),
           before,
           "unregistered reset cleared the held epoch"
       );
   }
   ```

   - At HEAD both tests are red: the first with `unregistered thread
     recorded into the held epoch`, the second with `unregistered reset
     invalidated the held epoch`.
   - On the change both are green.
   - **The refusal's own arm.** Build the changed tree a second time, with
     only the `if blocks_unregistered_thread() { return; }` at the top of
     `reset_lexical_profile_counters` deleted. The reset test must be red
     there, with one of its two messages, and green with the refusal in
     place. Save both logs. This shows that it is that refusal, not the
     epoch checks, that stops the reset.
4. **Old forms do not compile.**
   - The zero-argument `Session::logical_profile_test_reset()`, a call to
     `Epoch::new`, and `rum_runtime::test_logical_profile_reset()` with no
     lease must each fail to compile with E0061 or E0624.
   - Probe each in a scratch crate under `.temp/brief-635/probe/`, and save
     the three logs.
5. **The rum-runtime lib at 32 threads.**
   - Three runs, bound 120 s each, all green.

     ```
     cargo test -p rum-runtime --lib --offline -- --test-threads=32
     ```

   - HEAD is red: brief-633 run 1 was red 3/3. brief-634's measurement
     load (`.temp/brief-634/logs/measure-lib32.log`) gave
     `1210 passed; 10 failed`, all ten in the lease family:
     - `chain_drop`: four tests;
     - `queue_boundaries`: five tests;
     - `budget::logical_thread_metadata_stays_charged_until_thread_exit`.

## Stages and stops

1. **Shape.**
   - Implement the target shape and falsifiers 1–4.
   - Then build with `cargo build --tests --workspace --offline`, with no new
     warning.
2. **B1 rows.** Write the 62-row disposition. STOP on any unprotected row.
3. **Replay, lib and workspace.**
   - Replay the standing inventories.
   - Run falsifier 5.
   - Prove production builds with `nm` and the feature tree.
   - Then run `cargo test --workspace --no-fail-fast --offline --
     --test-threads=1`, bound 5400 s, in one 14G scope. The nine native
     ZeroMQ EPERM tests are skipped by name, as brief-634 lists them.

## Forbidden

- A check in `lexical_profile_enabled`/`configured`/`active`.
- A mutex or load on the flag-off path.
- `PROCESS_LIMIT` changes.
- Per-thread epochs.
- Silent-drop helpers outside the listed entries.
- `#[ignore]` as a fix, retries, sleeps, raised or guessed bounds,
  `--test-threads=1` as a fix, `serial_test` or any dependency.
- Weakening `logical_thread_metadata_stays_charged_until_thread_exit` or any
  C4′ test.

## Report

- Write `.temp/brief-635/report.json`. Quote every `test result:` line and
  every verdict line.
- Your final message is `python3 scripts/report.py summary 635`.

## Director calibration before dispatch (2026-10-30, HEAD `467ff97a`)

Each run was serial, in its own 14G scope. Logs are in
`.temp/director/gate-635-calib/logs/`, and the script is `calib.sh` beside
them.

- **Falsifier 1, the chain pair at two threads, on HEAD's own tree:** 9/10
  red.
  - Seven runs failed with
    `test result: FAILED. 1 passed; 1 failed; ... 1223 filtered out`, and
    two with `FAILED. 0 passed; 2 failed`. Run 8 was
    `test result: ok. 2 passed`.
  - The record test panics at `profiling/logical/record.rs:140:22`, and the
    source test at `profiling/tests/chain_drop.rs:89:5`.
  - Each run finished in about 0.05 s.
- **Falsifiers 2 and 3, the file above at HEAD.**
  - The setup: a `git archive` copy of HEAD in
    `.temp/director/gate-635-calib/head/` with `owner_filter.rs` added; each
    test run `--exact` at one thread.
  - Span:
    ``assertion `left == right` failed: unregistered thread recorded into the held epoch``,
    then `test result: FAILED. 0 passed; 1 failed`.
  - Reset: `unregistered reset invalidated the held epoch`, then
    `test result: FAILED. 0 passed; 1 failed`.
  - So `RumProfileReset[]` on a thread that never touched the profiler
    replaces another test's epoch at HEAD.
- **The green arms** exist only in brief-633's tree-final, which lacks the
  reset refusal. Both green arms are the worker's to show, and so is the
  refusal-deleted red arm.
