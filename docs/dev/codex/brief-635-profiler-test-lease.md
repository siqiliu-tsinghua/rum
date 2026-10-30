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
## report (kimi, DONE)

Implemented the cfg-gated process TestLease at epoch construction/current/open/storage/reset and Call::start, added held public wrappers, and refused unregistered lexical resets before any mutation. HEAD reds reproduced; chain pair is green 20/20, both unregistered tests pass, the refusal-deleted mutant is red, and old forms are compile-red. Three 32-thread runs pass after leasing the in-process observer. All 62 B1 rows have named lease/child protection. Replays are green; release CLI and harness have zero lease symbols. Workspace: 59 binaries, 4009 passed, 0 failed, 61 ignored, nine standing skips.

| File | Kind | Lines | Why |
| --- | --- | --- | --- |
| `crates/rum-host/tests/logical_profile_abandonment.rs` | modified | 41-42 | Hold the public wrapper for each abandonment run. |
| `crates/rum-host/tests/logical_profile_lifecycle.rs` | modified | 80,92,130,144,149,186,206,217,225,231,237,253,262,272,280,289 | Hold the public wrapper for each isolated child test and every reset. |
| `crates/rum-runtime/src/lib.rs` | modified | 18-19 | Require the public lease wrapper for the zero-argument-free runtime reset hook. |
| `crates/rum-runtime/src/runtime/mod.rs` | modified | 113-119,125 | Export the hidden test lease wrapper and thread it through the reset hook. |
| `crates/rum-runtime/src/runtime/profiling/lease.rs` | added | 1-60 | Add the cfg-gated process lease, generation registration and unregistered-thread refusal. |
| `crates/rum-runtime/src/runtime/profiling/logical/budget.rs` | modified | 95,105,117,121-148,150,163-164 | Lease budget tests and register plus explicitly join the metadata worker before sampling. |
| `crates/rum-runtime/src/runtime/profiling/logical/epoch.rs` | modified | 57,78-91,93-103,121-130,281-286,291-301,303,316,322-339 | Gate construction, current, open, storage and reset behind the lease while leaving production reset typed. |
| `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` | modified | 17,94-102,104-109,196-197,228-235 | Add the held public lease wrapper and refuse unregistered Call::start. |
| `crates/rum-runtime/src/runtime/profiling/mod.rs` | modified | 12-17 | Compile the lease module only under the test cfg. |
| `crates/rum-runtime/src/runtime/profiling/reset.rs` | modified | 3-7,9-16,21 | Refuse unregistered lexical/production reset before any counter, epoch or active-flag mutation. |
| `crates/rum-runtime/src/runtime/profiling/tests/chain_drop.rs` | modified | 43-44 | Take or hold the profiling test lease at the affected epoch call site. |
| `crates/rum-runtime/src/runtime/profiling/tests/context_scaling.rs` | modified | 61-62 | Take or hold the profiling test lease at the affected epoch call site. |
| `crates/rum-runtime/src/runtime/profiling/tests/engine_scaling.rs` | modified | 27-28 | Take or hold the profiling test lease at the affected epoch call site. |
| `crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs` | modified | 2,10-15,18-19,23,28,36-37,42 | Make Enabled acquire and retain the process lease while restoring both flags. |
| `crates/rum-runtime/src/runtime/profiling/tests/epoch_contract.rs` | added | 1-276 | Land the three unmutated C4-prime single-epoch contract tests. |
| `crates/rum-runtime/src/runtime/profiling/tests/index.rs` | modified | 6-7 | Take or hold the profiling test lease at the affected epoch call site. |
| `crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs` | modified | 458 | Hold the lease for the in-process observer over every lexical receipt. |
| `crates/rum-runtime/src/runtime/profiling/tests/logical_totals.rs` | modified | 5-8,46-47,68-69,92,94,111-112 | Take or hold the profiling test lease at the affected epoch call site. |
| `crates/rum-runtime/src/runtime/profiling/tests/mod.rs` | modified | 5 | Register C4-prime and owner-filter test modules. |
| `crates/rum-runtime/src/runtime/profiling/tests/owner_filter.rs` | added | 1-55 | Add the fixed unregistered-span and unregistered-reset falsifiers. |
| `crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs` | modified | 94-95 | Take or hold the profiling test lease at the affected epoch call site. |
| `crates/rum-runtime/src/runtime/profiling/tests/record_engine.rs` | modified | 14-15,57-58,80-81,100-101,115,138,140,181-182 | Take or hold the profiling test lease at the affected epoch call site. |
| `crates/rum-runtime/src/runtime/suspension/core/profile_queue/tests.rs` | modified | 5-6,84-85 | Take or hold the profiling test lease at the affected epoch call site. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | modified | replayed rows | Replay exact ownership discovery positions and new construct rows. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | modified | replayed rows | Replay exact ownership discovery positions and new construct rows. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | modified | replayed rows | Replay exact ownership discovery positions and new construct rows. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | modified | replayed rows | Replay exact ownership discovery positions and new construct rows. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | modified | replayed rows | Replay all 68 generated retention fingerprints. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | modified | replayed rows | Merge four generated certificate cells from both feature builds. |
| `crates/rum-stdlib/tests/suspension_oracle/descriptors.rs` | modified | 138-140,142,147 | Acquire before profiled reset and hold through the subject and snapshot. |

| Run | Try | Exit | Tests | Log |
| --- | ---: | ---: | --- | --- |
| fmt | 1 | 0 | - | `.temp/brief-635/logs/fmt.log` |
| head-chain | 1 | 101 | 1/1/0 | `.temp/brief-635/logs/head-chain.log` |
| head-unregistered-span | 1 | 101 | 0/1/0 | `.temp/brief-635/logs/head-unregistered-span.log` |
| head-unregistered-reset | 1 | 101 | 0/1/0 | `.temp/brief-635/logs/head-unregistered-reset.log` |
| chain-green-01 | 1 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-01.log` |
| chain-green-02 | 2 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-02.log` |
| chain-green-03 | 3 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-03.log` |
| chain-green-04 | 4 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-04.log` |
| chain-green-05 | 5 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-05.log` |
| chain-green-06 | 6 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-06.log` |
| chain-green-07 | 7 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-07.log` |
| chain-green-08 | 8 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-08.log` |
| chain-green-09 | 9 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-09.log` |
| chain-green-10 | 10 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-10.log` |
| chain-green-11 | 11 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-11.log` |
| chain-green-12 | 12 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-12.log` |
| chain-green-13 | 13 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-13.log` |
| chain-green-14 | 14 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-14.log` |
| chain-green-15 | 15 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-15.log` |
| chain-green-16 | 16 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-16.log` |
| chain-green-17 | 17 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-17.log` |
| chain-green-18 | 18 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-18.log` |
| chain-green-19 | 19 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-19.log` |
| chain-green-20 | 20 | 0 | 2/0/0 | `.temp/brief-635/logs/chain-green-20.log` |
| owner-green | 1 | 0 | 2/0/0 | `.temp/brief-635/logs/owner-green.log` |
| refusal-deleted-red | 1 | 101 | 0/1/0 | `.temp/brief-635/logs/refusal-deleted-red.log` |
| old-session-reset | 1 | 101 | - | `.temp/brief-635/logs/old-session-reset.log` |
| old-runtime-reset | 1 | 101 | - | `.temp/brief-635/logs/old-runtime-reset.log` |
| old-epoch-new | 1 | 101 | - | `.temp/brief-635/logs/old-epoch-new.log` |
| build-tests-1 | 1 | 0 | - | `.temp/brief-635/logs/build-tests.log` |
| build-tests-2 | 2 | 0 | - | `.temp/brief-635/logs/build-tests-2.log` |
| b1-disposition | 1 | 0 | - | `.temp/brief-635/logs/b1-disposition.log` |
| fingerprints-generate | 1 | 0 | 1/0/0 | `.temp/brief-635/logs/fingerprints-generate.log` |
| fingerprints-replay | 1 | 0 | - | `.temp/brief-635/logs/fingerprints-replay.log` |
| certificate-default-earn | 1 | 101 | 0/1/0 | `.temp/brief-635/logs/certificate-default-earn.log` |
| certificate-nodefault-earn | 1 | 101 | 0/1/0 | `.temp/brief-635/logs/certificate-nodefault-earn.log` |
| certificate-merge-1 | 1 | 1 | - | `.temp/brief-635/logs/certificate-replay-attempt-1.log` |
| certificate-merge-2 | 2 | 0 | - | `.temp/brief-635/logs/certificate-replay.log` |
| certificate-default-replay | 2 | 0 | 1/0/0 | `.temp/brief-635/logs/certificate-default-replay.log` |
| certificate-nodefault-replay | 2 | 0 | 1/0/0 | `.temp/brief-635/logs/certificate-nodefault-replay.log` |
| ownership-discovery-1 | 1 | 101 | 2/1/0 | `.temp/brief-635/logs/ownership-discovery.log` |
| ownership-replay | 1 | 0 | - | `.temp/brief-635/logs/ownership-replay.log` |
| ownership-replay-verify | 2 | 0 | 3/0/0 | `.temp/brief-635/logs/ownership-replay-verify.log` |
| lp01-1 | 1 | 0 | 6/0/0 | `.temp/brief-635/logs/lp01.log` |
| typed-receipts | 1 | 0 | 8/2/1 | `.temp/brief-635/logs/typed-receipts.log` |
| rejection-census | 1 | 0 | 1/0/0 | `.temp/brief-635/logs/rejection-census.log` |
| lib32-initial | 1 | 101 | 1224/1/5 | `.temp/brief-635/logs/lib32-01.log` |
| observer-lease | 1 | 0 | 1/0/0 | `.temp/brief-635/logs/observer-lease.log` |
| lib32-green-1 | 1 | 0 | 1225/0/5 | `.temp/brief-635/logs/lib32-02.log` |
| lib32-green-2 | 2 | 0 | 1225/0/5 | `.temp/brief-635/logs/lib32-03.log` |
| lib32-green-3 | 3 | 0 | 1225/0/5 | `.temp/brief-635/logs/lib32-04.log` |
| fingerprints-generate-final | 2 | 0 | 1/0/0 | `.temp/brief-635/logs/fingerprints-generate-2.log` |
| fingerprints-replay-final | 2 | 0 | - | `.temp/brief-635/logs/fingerprints-replay-2.log` |
| certificate-default-earn-final | 2 | 101 | 0/1/0 | `.temp/brief-635/logs/certificate-default-earn-2.log` |
| certificate-nodefault-earn-final | 2 | 101 | 0/1/0 | `.temp/brief-635/logs/certificate-nodefault-earn-2.log` |
| certificate-merge-final | 3 | 0 | - | `.temp/brief-635/logs/certificate-replay-2.log` |
| certificate-default-replay-final | 3 | 0 | 1/0/0 | `.temp/brief-635/logs/certificate-default-replay-2.log` |
| certificate-nodefault-replay-final | 3 | 0 | 1/0/0 | `.temp/brief-635/logs/certificate-nodefault-replay-2.log` |
| ownership-final | 3 | 0 | 3/0/0 | `.temp/brief-635/logs/ownership-discovery-2.log` |
| lp01-final | 2 | 0 | 6/0/0 | `.temp/brief-635/logs/lp01-2.log` |
| release-cli | 1 | 0 | - | `.temp/brief-635/logs/release-cli-build.log` |
| release-harness | 1 | 0 | - | `.temp/brief-635/logs/release-harness-build.log` |
| nm-cli | 1 | 0 | - | `.temp/brief-635/logs/nm-cli-check.log` |
| nm-harness | 1 | 0 | - | `.temp/brief-635/logs/nm-harness-check.log` |
| feature-tree | 1 | 0 | - | `.temp/brief-635/logs/rum-cli-features.log` |
| fmt-check | 1 | 0 | - | `.temp/brief-635/logs/fmt-check.log` |
| build-tests-final | 3 | 0 | - | `.temp/brief-635/logs/build-tests-3.log` |
| static-invariants | 1 | 0 | - | `.temp/brief-635/logs/static-invariants.log` |
| workspace | 1 | 0 | 4009/0/61 | `.temp/brief-635/logs/workspace.log` |

| Check | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| head-controls | fixed falsifiers reproduce red at HEAD | all three HEAD controls red with the prescribed messages | PASS | The pre-edit copy was made before implementation. |
| chain | changed chain pair green 20/20 at two threads | 20 runs, 40 passed, zero failed | PASS | Every run used its own 14G scope and a 60-second bound. |
| unregistered | span/reset green and refusal-deleted reset red | two green; refusal-deleted mutant red with the prescribed message | PASS | This isolates the reset refusal from the epoch checks. |
| old-forms | old reset and unleased constructor forms do not compile | Session reset E0061; rum_runtime reset E0061; private new_unleased constructor E0624 | PASS | The fixed shape removes Epoch::new; the epoch probe calls its private new_unleased replacement, and the static sweep confirms the old name is absent. |
| build | workspace tests build with no new warning | final build green; only HEAD three warnings | PASS | The first build exposed feature-only allow_current_thread dead code; cfg(test) fixed it, and generation is read in Drop without an allow attribute. |
| b1 | all 62 applicable collision rows have child or lease protection | 62 lease+child rows; zero unprotected; none=0 | PASS | The first 32-thread run exposed the observer reader; it now holds the lease and is included in every applicable row. |
| lib32 | three rum-runtime lib runs at 32 threads | three green runs after leasing the observer | PASS | The initial run was 1224 passed/1 failed; the observer fix is covered by its focused green run. |
| c4p | C4-prime epoch contract stays green and unmutated | all three pass in each green lib32 run; test file byte-identical to accepted design | PASS | PROCESS_LIMIT and owner matching are byte-preserved. |
| production | no lease symbol or test feature in release production arms | NM_CLI=0; NM_HARNESS=0; config byte-identical | PASS | The director owns the resume/cli perf gate. |
| fingerprints | retention fingerprints and certificates replay generated cells only | final replay changed 68 fingerprint cells and four certificate cells; non-certificate changes zero | PASS | Earning passes stop only on stale recorded cells; merged replays pass. |
| ownership | ownership identities and host-payload rules hold | 7962 rows; 20 added, 113 moved, zero removed identities; final exact | PASS | 61 self-reaching types and 79 host payloads are unchanged. |
| lp01 | LP01 producer/binding/loss inventories replay | six green test-result lines on final source | PASS | No LP01 expectation file changed. |
| typed-receipts | typed-slot receipts and forced-red markers remain intact | {'passed': 8, 'failed': 2, 'ignored': 1} | PASS | No typed-slot receipt inventory changed. |
| rejection-census | production rejection census does not move | green without expectation edits | PASS |  |
| format | cargo fmt --check | exit zero | PASS |  |
| workspace | serial full workspace with exactly nine standing native ZeroMQ skips | 59 green binaries; 4009 passed; 0 failed; 61 ignored | PASS | No other skip, retry, sleep or serial target was used. |
| scope | only lease files and replayed inventories change; immutable cost bars hold | all static invariants pass | PASS | No dependency was added and no production branch changed. |

### Lease shape and test-cfg boundary

The process-wide epoch remains singular. A mutex-held generation registers the owning thread; spawned workers must register explicitly. Unregistered threads return before epoch locks or mutation at the listed entries, and lexical reset refuses before its configured check. ProductionReset has a private field. The public wrapper field is private and reset methods require a held reference. The metadata scoped worker is explicitly joined before PROCESS_BYTES is sampled. `lexical_profile_enabled`, configured and active are untouched; release production compiles no lease module.

### 62-row B1 disposition

Every reader is named. `lease+child` means runtime readers hold `TestLease` and host/LP01 readers run in their existing isolated child processes. The six `RUM_PROFILE_REPR` rows are excluded by the brief because no test enables that variable.

| # | Global | Readers and protection |
| ---: | --- | --- |
| 1 | `PROCESS_BYTES` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. lease: logical_initial_cap_failure_stays_invalid_until_reset; logical_thread_metadata_stays_charged_until_thread_exit (worker registered and explicitly joined); logical_expired_queue_arena_stays_charged_until_weak_drop. child: logical_native_nested_pause_counts (LP02_NESTED_CHILD); logical_native_source_scope_survives_resume (LP02_SCOPE_CHILD); logical_native_lifecycle (LP02_NATIVE_CHILD); profiled_abandonment_requeues_like_ordinary (LP02_ABANDON_CHILD); oracle::fixtures::logical_profile::logical_counter_port (five RUM_LP01_CHILD modes) |
| 2 | `CURRENT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. lease: logical_initial_cap_failure_stays_invalid_until_reset; logical_thread_metadata_stays_charged_until_thread_exit; logical_source_chain_drops_on_two_mib; logical_source_chain_concurrent_last_holders; logical_zero_frames_and_after_input_frontier_have_no_guest_storage; logical_every_structural_route_closes_once_with_its_reason; logical_diagnostic_cap_keeps_the_marker_free_guest_capacity; logical_deferred_return_boundary_observes_the_return_resource_error; logical_driver_slot_fault_abandons_the_rooted_boundary; logical_frontier_rejects_another_owner_even_without_semantic_cells; logical_after_consumer_sees_the_input_parser_error; logical_queued_frames_have_one_owner_in_both_modes; logical_queue_representation_survives_a_disabled_global_gate; logical_marker_preserves_slot_identity_and_rejects_wrong_channels; logical_marker_observes_closed_channels_without_codecs; unregistered_thread_span_does_not_record_while_a_profiling_guard_is_held; unregistered_thread_profile_reset_leaves_the_held_epoch; no_cross_attribution; same_key_additivity; owner_scoped_abandonment. child: logical_native_nested_pause_counts (LP02_NESTED_CHILD); logical_native_source_scope_survives_resume (LP02_SCOPE_CHILD); logical_native_lifecycle (LP02_NATIVE_CHILD); profiled_abandonment_requeues_like_ordinary (LP02_ABANDON_CHILD); oracle::fixtures::logical_profile::logical_counter_port (five RUM_LP01_CHILD modes) |
| 3 | `LEXICAL_PROFILE_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. lease: logical_initial_cap_failure_stays_invalid_until_reset; logical_thread_metadata_stays_charged_until_thread_exit; logical_source_chain_drops_on_two_mib; logical_source_chain_concurrent_last_holders; logical_zero_frames_and_after_input_frontier_have_no_guest_storage; logical_every_structural_route_closes_once_with_its_reason; logical_diagnostic_cap_keeps_the_marker_free_guest_capacity; logical_deferred_return_boundary_observes_the_return_resource_error; logical_driver_slot_fault_abandons_the_rooted_boundary; logical_frontier_rejects_another_owner_even_without_semantic_cells; logical_after_consumer_sees_the_input_parser_error; logical_queued_frames_have_one_owner_in_both_modes; logical_queue_representation_survives_a_disabled_global_gate; logical_marker_preserves_slot_identity_and_rejects_wrong_channels; logical_marker_observes_closed_channels_without_codecs; unregistered_thread_span_does_not_record_while_a_profiling_guard_is_held; unregistered_thread_profile_reset_leaves_the_held_epoch; no_cross_attribution; same_key_additivity; owner_scoped_abandonment. child: logical_native_nested_pause_counts (LP02_NESTED_CHILD); logical_native_source_scope_survives_resume (LP02_SCOPE_CHILD); logical_native_lifecycle (LP02_NATIVE_CHILD); profiled_abandonment_requeues_like_ordinary (LP02_ABANDON_CHILD); oracle::fixtures::logical_profile::logical_counter_port (five RUM_LP01_CHILD modes) |
| 4 | `LEXICAL_PROFILE_ACTIVE` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. lease: logical_initial_cap_failure_stays_invalid_until_reset; logical_thread_metadata_stays_charged_until_thread_exit; logical_source_chain_drops_on_two_mib; logical_source_chain_concurrent_last_holders; logical_zero_frames_and_after_input_frontier_have_no_guest_storage; logical_every_structural_route_closes_once_with_its_reason; logical_diagnostic_cap_keeps_the_marker_free_guest_capacity; logical_deferred_return_boundary_observes_the_return_resource_error; logical_driver_slot_fault_abandons_the_rooted_boundary; logical_frontier_rejects_another_owner_even_without_semantic_cells; logical_after_consumer_sees_the_input_parser_error; logical_queued_frames_have_one_owner_in_both_modes; logical_queue_representation_survives_a_disabled_global_gate; logical_marker_preserves_slot_identity_and_rejects_wrong_channels; logical_marker_observes_closed_channels_without_codecs; unregistered_thread_span_does_not_record_while_a_profiling_guard_is_held; unregistered_thread_profile_reset_leaves_the_held_epoch; no_cross_attribution; same_key_additivity; owner_scoped_abandonment. child: logical_native_nested_pause_counts (LP02_NESTED_CHILD); logical_native_source_scope_survives_resume (LP02_SCOPE_CHILD); logical_native_lifecycle (LP02_NATIVE_CHILD); profiled_abandonment_requeues_like_ordinary (LP02_ABANDON_CHILD); oracle::fixtures::logical_profile::logical_counter_port (five RUM_LP01_CHILD modes) |
| 5 | `LEXICAL_PROFILE_DETAIL_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 6 | `LEXICAL_PROFILE_INCREMENTAL_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 7 | `LEXICAL_PROFILE_SHAPES_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 8 | `LEXICAL_PROFILE_BINDINGS_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 9 | `LEXICAL_PROFILE_SOURCES_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 10 | `LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 11 | `LEXICAL_PROFILE_PROVENANCE_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 12 | `RULE_RHS_TIMING_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 13 | `DEFINITION_RULE_TIMING_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 14 | `BUILTIN_TIMING_ENABLED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 15 | `LEXICAL_PROFILE_SUMMARY_PRINTED` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 16 | `DEFINITION_RHS_EVAL_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 17 | `DEFINITION_RHS_EVAL_NANOS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 18 | `EVAL_MODULE_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 19 | `EVAL_MODULE_NANOS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 20 | `MODULE_RESUME_ENTRY_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 21 | `MODULE_RESUME_MAX_INDEX` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 22 | `MODULE_INIT_SUBSTITUTION_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 23 | `MODULE_INIT_SUBSTITUTION_NANOS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 24 | `MODULE_BODY_SUBSTITUTION_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 25 | `MODULE_BODY_SUBSTITUTION_NANOS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 26 | `LEXICAL_SUBSTITUTION_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 27 | `LEXICAL_SUBSTITUTION_TIME_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 28 | `LEXICAL_SUBSTITUTION_NANOS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 29 | `LEXICAL_SUBSTITUTION_CHANGED_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 30 | `LEXICAL_SUBSTITUTION_UNCHANGED_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 31 | `LEXICAL_BINDING_EMPTY_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 32 | `LEXICAL_BINDING_SINGLE_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 33 | `LEXICAL_BINDING_MULTI_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 34 | `LEXICAL_BINDING_MAX_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 35 | `LEXICAL_SHADOWED_NONEMPTY_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 36 | `ARG_LIST_SUBSTITUTION_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 37 | `ARG_LIST_TOTAL_ARGS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 38 | `ARG_LIST_MAX_LEN` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 39 | `ARG_PREFILTER_SKIP_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 40 | `ARG_PREFILTER_MAYBE_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 41 | `ARG_SUBSTITUTION_ATTEMPT_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 42 | `ARG_SUBSTITUTION_ATTEMPT_NANOS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 43 | `ARG_SUBSTITUTION_CHANGED_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 44 | `ARG_SUBSTITUTION_UNCHANGED_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 45 | `PRESENCE_SINGLE_SCAN_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 46 | `PRESENCE_SINGLE_SCAN_NODES` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 47 | `PRESENCE_SINGLE_SCAN_NANOS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 48 | `PRESENCE_SINGLE_SCAN_TIME_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 49 | `PRESENCE_SINGLE_SCAN_TRUE_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 50 | `PRESENCE_SINGLE_SCAN_FALSE_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 51 | `PRESENCE_MULTI_SCAN_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 52 | `PRESENCE_MULTI_SCAN_NODES` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 53 | `PRESENCE_MULTI_SCAN_NANOS` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 54 | `PRESENCE_MULTI_SCAN_TIME_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 55 | `PRESENCE_MULTI_SCAN_TRUE_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 56 | `PRESENCE_MULTI_SCAN_FALSE_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 57 | `REWRITE_RULE_CANDIDATE_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 58 | `REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 59 | `REWRITE_RULE_WOULD_MATCHER_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 60 | `REWRITE_RULE_MATCHER_MISS_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 61 | `REWRITE_RULE_MATCH_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |
| 62 | `REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT` | lease: profiling_observer_covers_every_map_and_flag_without_ranking. child: oracle::fixtures::logical_profile::logical_counter_port (five isolated RUM_LP01_CHILD modes) |


### Complete standing-inventory disclosure

Final replay changed all 68 fingerprint cells and four certificate cells only. Ownership has 7,962 UnknownConstruct rows: 20 added and 113 position moves, with zero removed identities, no new UnknownHostPayload, 61 self-reaching types and 79 host payloads. Full old/new diffs are in `local-contracts-fingerprint.diff`, `local-contracts-fingerprint-final.diff`, `certificate-replay.diff`, `certificate-replay-final.diff`, and `ownership-replay.diff`. LP01 and typed-slot inventories and the rejection census did not change. The initial certificate helper assertion and first ownership mismatch are retained as failed runs. After the observer lease was added, fingerprints and certificates were regenerated again; ownership and LP01 were re-verified on final source.

## Director gate (2026-10-30): stage 1 committed, B1 open

The epoch lease is right and is committed. The 58 lexical rows of B1 are not
protected, and the brief's own STOP should have fired there. The rest of the
brief continues as amendment 1.

- **Code.** I read the whole diff. It takes the brief's shape:
  - `lease.rs` is compiled only under the test cfg. Its `TEST_LEASE` is
    poison-tolerant. `allow_current_thread` is `#[cfg(test)]`, and `Drop`
    reads `generation`, so no new warning.
  - The refusals are in `epoch::storage`, in `open` before the data lock, in
    `current` under the slot lock, and in `Call::start`.
    `reset_lexical_profile_counters` refuses first, and
    `reset_production` takes a `ProductionReset` that only `reset.rs` mints.
  - `Enabled` holds the lease. `descriptors.rs` takes the lease before the
    session, and the lease outlives it. The rum-host children hold the
    public `LogicalProfileTestLease`. The budget metadata test uses
    `thread::scope` with an explicit `join`.
  - `test-builtin-memory-growth` enters rum-cli's tree only through
    `[dev-dependencies]`.
  - `owner_filter.rs` is the brief's code, up to rustfmt wrapping.
  - `epoch_contract.rs` holds the three C4′ tests verbatim from tree-final.
- **My amendment, disclosed.** `epoch_contract.rs`'s header still said
  "Scratch copy only." I replaced it with "(brief-633)". The retention
  fingerprint hashes the source tree, so this moved all 68 fingerprints. The
  gate's `fingerprints` step regenerated them and copied them in. No column
  other than the fingerprint moved.
- **Ownership inventories.** `unknown_attributes`, `unknown_captures`,
  `unknown_macros` and `unknown_storage` gained 10, 4, 2 and 4 rows, and lost
  none, positions masked. Every new row is new test-cfg code: cfg
  attributes, `lease.rs` closures, macros and storage,
  `LogicalProfileTestLease.inner`, and the `acquire` signature.
- **`python3 -I scripts/director-gate.py run 635 --perf resume,cli`**
  (`.temp/director/gate-635/`): GREEN, every step PASS.
  - `check`: 3 warnings, 0 new against HEAD.
  - census: 1040 rows paired, 0 new, 0 retired.
  - certificates: both builds pass, 2 promoted rows merged.
  - workspace: 59 binaries, 4018 passed, 0 failed, 61 ignored, peak 6.0 GB
    of 14, `oom_kill` 0. That is brief-634's 4013 plus the five new tests.
    kimi's 4009 lacks the nine native ZeroMQ tests.
  - structure: `resume` and `rum-cli` differ in 0 functions, 0 new, 0 gone.
  - panels, 15 reps each: resume 121 labels, median +0.0010%; cli 192
    labels, median +0.0001%. Nothing is apart on either side. The cli maximum,
    `valueq-undefined-symbol-definitions-8000` at +3.56%, is a 20-unit label
    whose 2000 sibling reads −0.63%, and the function bodies are identical.
- **Production symbols.** `nm -C` on the gate's four release arms (base and
  candidate `rum-cli` and `resume`) finds 0 lease symbols. The symbol counts
  are equal: 18691 and 18122.
- **Falsifiers I reproduced on the change**, serially, each in its own 14G
  scope (`.temp/director/gate-635/repro/`):
  - The chain pair at two threads: 10 of 10 runs
    `test result: ok. 2 passed; 0 failed; ...; 1228 filtered out`. HEAD was
    9 of 10 red.
  - Both `owner_filter` tests: `test result: ok. 2 passed; 0 failed`.
  - The refusal-deleted arm, built from a copy with only the refusal at the
    top of `reset_lexical_profile_counters` removed: the span test passes,
    the reset test panics `unregistered reset invalidated the held epoch`,
    `test result: FAILED. 1 passed; 1 failed`.
- **The red I found: the rum-runtime lib at 32 threads, 2 of 3.** Run 3
  failed in the observer kimi leased:

  ```
  thread 'runtime::profiling::logical_contract::profiling_observer_covers_every_map_and_flag_without_ranking' panicked at crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs:481:5:
  assertion `left == right` failed: observer ran guest work or changed counters
  ... "counter|definition_rhs_eval": [25] ... vs ... [26] ...
  test result: FAILED. 1224 passed; 1 failed; 5 ignored
  ```

  kimi reported 3 of 3 green. Three draws cannot show a rare red, and they
  did not.
- **Why: the lexical write side is not refused.** The brief accepts
  **lease** only when "every write an unregistered thread can reach is
  refused under the test cfg". No lexical write is refused:
  - `record_timing` and `record_profile_lexical_timing[_elapsed]`
    (`counters.rs`) write whenever the caller passes `Some(start)`. That
    start was taken from `lexical_profile_start()` when the call began. A
    thread that starts a definition RHS while some holder has the flag on
    records after that holder drops it, into the next holder's window. That
    is the 25 → 26 above.
  - `record_counter` and `record_add` gate on the flag, not on the thread,
    so an unregistered thread writes while any holder has the flag on.
    `record_max` checks nothing itself; its three callers gate on the flag.
    The `REWRITE_RULE_*` counters in `records.rs` are written directly behind
    the same flag gate.
  - **A second open path, rows 3 and 4.** `Enabled` restores the value it
    swapped out, so it can restore 0. That re-arms the cold-load
    initializers in `config.rs`. An unregistered thread that read 0 and is
    still in `initialize_lexical_profile_enabled` (or `_active`) then stores
    the env value over the holder's 1. This is a reading, not a reproduced
    red. That makes it an open obligation.
- **My error.** In my diff review I accepted kimi's argument that the
  counters "are written only while the flags are on". That is false for the
  timing helpers, and it never answered which threads write. The brief's
  rule was the right one, and I checked the claim against the wrong rule.
- **Disposition.**
  - Committed as stage 1: the epoch lease, its refusals and falsifiers, the
    leased tests, and C4′.
  - Open, as amendment 1: refuse every lexical write that an unregistered
    thread can reach, under the test cfg and after the flag or start gate,
    so the flag-off path stays byte-identical. Close the cold-load race on
    rows 3 and 4. Ship a deterministic falsifier for each, calibrated red at
    this commit. Redo B1 with each row's writers listed.
  - Test-isolation item 1 is not done until amendment 1 is gated.
- **Not rerun:** kimi's twenty chain pairs (I ran ten), and the three
  old-form compile probes (their logs show E0061/E0624, and the code they
  probe is deleted).

## Director amendment 1 (2026-10-30): refuse the lexical write side

**Worker: kimi, continuing its stage-1 session.** Stage 1 is committed
(`b199d63c`). It leased the epoch, and that part is right. The lexical write
side stays open: my gate (above) found the rum-runtime lib red at 32 threads,
in the observer kimi leased. The B1 rule ("**lease**: every write an
unregistered thread can reach is refused under the test cfg") was not met for
any of the 58 lexical rows. This amendment closes that side. It is one gated
step, in three stages.

### What is open, with evidence

- **HEAD's 32-thread red rate.** HEAD is `927efc18`; its `crates/` is stage
  1's tree. I ran `cargo test -p rum-runtime --lib --offline --
  --test-threads=32` ten times, serially, each in its own 14G scope (logs in
  `.temp/director/gate-635a1-calib/logs/head32-*.log`).
  - Three of the ten runs were red: runs 1, 8 and 9. Each failed with
    `test result: FAILED. 1224 passed; 1 failed; 5 ignored`.
  - All three failed in
    `profiling_observer_covers_every_map_and_flag_without_ranking`
    (`logical_contract.rs:481`). The only moved row was
    `counter|definition_rhs_eval`: `[44]`→`[45]`, `[19]`→`[21]`, `[17]`→`[18]`.
  - With my stage-1 gate's three runs (one red), that is 4 red of 13.
- **The writers no check refuses.** Every process-wide write to a `state.rs`
  static reaches one of these sites. The source is
  `grep -rnE '(<the 60 names>)\s*\.\s*(fetch_|store|swap|compare_exchange)'`
  over `crates/`, plus every `&NAME` passed to a helper. All 33 helper calls
  are in `records.rs`, and the 34th is `counters.rs`'s own
  `record_profile_lexical_timing` → `_elapsed`.
  - `counters.rs`:
    - `record_timing` and `record_profile_lexical_timing_elapsed` are gated
      only on a `Some` start or elapsed value. A start taken under one
      lease's flag is recorded under the next lease. That is the red above.
    - `record_counter` and `record_add` are gated on the flag, not on the
      thread.
    - `record_max` has no gate. Its three callers gate on the flag.
  - `records.rs`: `record_profile_rewrite_rule_prefilter_candidate` and
    `record_profile_rewrite_rule_match_outcome` `fetch_add` six
    `REWRITE_RULE_*` counters directly, behind the flag.
  - `summary.rs`: `print_lexical_profile_summary_once` swaps
    `LEXICAL_PROFILE_SUMMARY_PRINTED`, and `print_lexical_profile_snapshot`
    stores it.
  - `config.rs`: the twelve cold-load initializers `store` the env value
    unconditionally. A holder's `Enabled` swaps 1 into `ENABLED` and
    `ACTIVE`, and restores what it swapped out, which can be 0. That 0
    re-arms the initializers. A thread that loaded 0 before the holder's
    swap then stores its env value over the holder's 1. In production, a
    first `ACTIVE` load under `RUM_PROFILE_LEXICAL_DEFER` can likewise store
    2 over a concurrent `RumProfileReset[]`'s 1.
  - `reset.rs`'s writes are refused by stage 1. The `Enabled` and
    `queue_boundaries.rs` flag writes are made by holders only.
- **A check alone leaves a window.** Suppose a thread checks
  `blocks_unregistered_thread()` and then writes. It can read generation 0
  between two leases and write after the next lease's first snapshot. Stage
  1's epoch side does not have this window, because each lease gets a fresh
  epoch under the slot lock. The counters have no such lock. So a lease must
  wait for every write admitted before its generation. That wait is the
  fence below.
- **`logical_queue_representation_survives_a_disabled_global_gate` is
  vacuous.** It swaps `LEXICAL_PROFILE_ACTIVE` to 0. But 0 is the
  *uninitialized* state: the test thread's next `lexical_profile_active()`
  cold-loads it back to 1, so the gate is never disabled. I swapped in 2
  (disabled) in a HEAD copy, and the test passed: `test result: ok. 1
  passed` (`logs/queue2.log`). The test's claim holds with the gate really
  off, so the test takes 2.

### Target shape

The shape is fixed. A departure from it is a STOP.

**The test cfg** is the stage-1 attribute, written exactly as at the top of
`reset_lexical_profile_counters`:

```rust
#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
```

Below, `TEST_CFG` stands for it.

**`profiling/lease.rs`** gains the write gate, the fence and admission:

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard};

/// Read across one admitted lexical write. `lease` takes it for write after
/// publishing its generation, so a lease is current only once every write
/// admitted before it has landed.
static WRITE_GATE: RwLock<()> = RwLock::new(());
/// Leases that have published their generation and wait at `WRITE_GATE`.
static FENCING: AtomicUsize = AtomicUsize::new(0);

pub(in crate::runtime) fn lease() -> TestLease {
    let guard = TEST_LEASE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let generation = NEXT_GEN.fetch_add(1, Ordering::Relaxed);
    CURRENT_GEN.store(generation, Ordering::Release);
    fence_admitted_writes();
    REGISTERED_GEN.with(|cell| cell.set(generation));
    TestLease {
        _guard: guard,
        generation,
    }
}

/// Wait out every lexical write admitted before this generation.
fn fence_admitted_writes() {
    FENCING.fetch_add(1, Ordering::SeqCst);
    drop(WRITE_GATE.write().unwrap_or_else(|poison| poison.into_inner()));
    FENCING.fetch_sub(1, Ordering::SeqCst);
}

/// One admitted lexical write. Its scope holds atomics, thread-locals and
/// `eprintln!` only: never `admit`, `lease`, or another lock.
pub(in crate::runtime) struct Admitted {
    _gate: RwLockReadGuard<'static, ()>,
}

/// `None` when a lease is current and this thread did not register for it.
#[inline]
pub(in crate::runtime) fn admit() -> Option<Admitted> {
    let gate = WRITE_GATE
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    if blocks_unregistered_thread() {
        return None;
    }
    Some(Admitted { _gate: gate })
}

/// Leases waiting at the fence: the fence falsifier's observable.
#[cfg(test)]
pub(in crate::runtime) fn fencing() -> usize {
    FENCING.load(Ordering::SeqCst)
}
```

- `TestLease`, `allow_current_thread`, `Drop for TestLease` and
  `blocks_unregistered_thread` are unchanged.
- The module comment gains "and the lexical counters" after "the
  process-wide logical epoch".

**Admission at every lexical write.** Each site gains one statement, placed
exactly where shown: after the site's existing flag or start gate, so the
flag-off path does not change in any build.

```rust
    TEST_CFG
    let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
        return;
    };
```

- **`counters.rs`:**
  - `record_timing`: after `let Some(start) = start else { return; };`.
  - `record_profile_lexical_timing_elapsed`: after its
    `let Some(elapsed_nanos) = ... else { return; };`.
  - `record_counter` and `record_add`: after
    `if !lexical_profile_enabled() { return; }`.
  - `record_max`: as its first statement. Its three callers gate on the
    flag.
  - `record_profile_lexical_timing` gains nothing. It forwards to `_elapsed`,
    and admissions never nest.
- **`records.rs`.** In both rewrite-rule functions, the admission and the
  `fetch_add`s move into one block, so the gate is not held across
  `Symbol::new` or the thread-local map:

  ```rust
      if !lexical_profile_enabled() {
          return;
      }

      {
          TEST_CFG
          let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
              return;
          };
          REWRITE_RULE_CANDIDATE_COUNT.fetch_add(1, Ordering::Relaxed);
          if would_reject {
              REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT.fetch_add(1, Ordering::Relaxed);
          } else {
              REWRITE_RULE_WOULD_MATCHER_COUNT.fetch_add(1, Ordering::Relaxed);
          }
      }

      let Some(source_symbol) = evaluation_profile_current_source_symbol() else {
  ```

  `record_profile_rewrite_rule_match_outcome` takes the same block around its
  `if matched { ... } else { ... }`.
- **`summary.rs`:**

  ```rust
      if lexical_enabled {
          logical::drop_session(session);
      }
      let printed = {
          TEST_CFG
          let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
              return;
          };
          LEXICAL_PROFILE_SUMMARY_PRINTED.swap(1, Ordering::Relaxed)
      };
      if printed == 1 {
          return;
      }
  ```

  In `print_lexical_profile_snapshot`, the
  `LEXICAL_PROFILE_SUMMARY_PRINTED.store(1, Ordering::Relaxed);` line becomes
  `mark_lexical_profile_summary_printed();`, with:

  ```rust
  fn mark_lexical_profile_summary_printed() {
      TEST_CFG
      let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
          return;
      };
      LEXICAL_PROFILE_SUMMARY_PRINTED.store(1, Ordering::Relaxed);
  }
  ```

**`config.rs`: the twelve initializers publish over 0 only.** This is the
one production change. It runs once per flag per process, in a `#[cold]`
function. Without a concurrent writer its semantics are those of the
`store`, which they replace.

```rust
/// Publish a cold-loaded flag over the uninitialized 0 only. A value another
/// writer stored after this thread's load (a test holder's swap,
/// `RumProfileReset[]`) is kept, and answered.
fn publish(flag: &AtomicU8, enabled: bool) -> bool {
    match flag.compare_exchange(
        0,
        if enabled { 1 } else { 2 },
        Ordering::Relaxed,
        Ordering::Relaxed,
    ) {
        Ok(_) => enabled,
        Err(published) => published == 1,
    }
}

#[cold]
fn initialize_lexical_profile_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish(&LEXICAL_PROFILE_ENABLED, enabled)
}
```

- Each of the other eleven initializers ends the same way:
  `publish(&<ITS FLAG>, <its value>)`, where
  `initialize_lexical_profile_active` passes `active`.
- `config.rs` also gains, at its end:

  ```rust
  #[cfg(test)]
  #[path = "tests/cold_load.rs"]
  mod cold_load;
  ```

- `lexical_profile_enabled`, `lexical_profile_configured`,
  `lexical_profile_active` and every other accessor stay byte-identical.
- At my gate, the structure step should show only these initializers
  differing in the release binaries, or a caller they are inlined into. I
  rule on any other function that differs.

**`tests/queue_boundaries.rs`**, in
`logical_queue_representation_survives_a_disabled_global_gate`:

```rust
        // 2 is disabled; 0 is uninitialized, and any cold load re-arms it.
        let prior = LEXICAL_PROFILE_ACTIVE.swap(2, Ordering::Relaxed);
```

**`docs/dev/profiling.md`.** Add this paragraph after the "Enabled
diagnostic storage" paragraph of "Lifecycle and epochs":

> Profiling tests share the process with ordinary tests. Under the test cfg,
> every test that reads or writes the process-wide profiling state holds the
> one `TestLease` (`profiling/lease.rs`). While a lease is current, a thread
> that has not registered with it cannot construct, open, charge or reset
> the epoch, reset the lexical counters, or write a lexical counter or the
> summary once-flag. A lease becomes current only after every lexical write
> admitted before it has landed. Production builds compile none of the
> lease. The cold-load profile flags publish over their uninitialized state
> only, so a value stored after a thread's first load is kept.

### Scope

- **MAY modify:** `profiling/lease.rs`, `counters.rs`, `records.rs` (the two
  rewrite-rule functions only), `summary.rs` (the two once-flag writes
  only), `config.rs` (`publish`, the twelve initializers' tails, the
  `cold_load` line), `profiling/tests/mod.rs` (one line), the new
  `profiling/tests/unregistered_writes.rs` and `profiling/tests/cold_load.rs`,
  `tests/queue_boundaries.rs` (the swap line), `docs/dev/profiling.md` (the
  paragraph above), and the standing inventories through their replays.
- **Every other edit is a disclosed deviation**, with its reason, in the
  report's `deviations`. An undisclosed edit is a STOP at my gate.
- **MUST NOT:**
  - change any accessor in `config.rs`;
  - change a recording path outside the sites above, or any flag-off path;
  - change stage 1's refusals, `Enabled`, or the C4′ tests;
  - change `PROCESS_LIMIT` or the epoch's semantics;
  - add a dependency or `unsafe`.

### Falsifiers

Before your first edit, copy the current
`crates/ Cargo.toml Cargo.lock rust-toolchain.toml scripts/run-bounded.py`
to `.temp/brief-635/a1/head/`. Each run uses its own
`scripts/memory-scope.sh b635a1-<label>-<n> 14G` scope, `--offline` and
`CARGO_BUILD_JOBS=8`. Never use git.

1. **A, B and D: the calibrated files, copied verbatim.**
   - `.temp/director/gate-635a1-calib/src/unregistered_writes.rs` goes to
     `profiling/tests/unregistered_writes.rs`, registered as
     `mod unregistered_writes;` after `mod slot_boundary;` in `tests/mod.rs`.
   - `.temp/director/gate-635a1-calib/src/cold_load.rs` goes to
     `profiling/tests/cold_load.rs`.
   - Do not edit either file, beyond the import merge and the test C append
     below.
   - **A**, `unregistered_timing_started_under_one_lease_does_not_land_in_the_next`:
     a start taken under one lease's flag, recorded under the next lease.
   - **B**, `unregistered_writes_do_not_reach_a_held_window`: one entry per
     write site, each from a fresh unregistered thread while the test holds
     `Enabled`. It compares all 47 counters and the once-flag, which the
     file reads from `state.rs` so the list stays complete.
   - **D**, `a_cold_load_keeps_a_flag_published_after_its_load`: all twelve
     initializers, each run over a published 1 and a published 2.
   - **My calibration at HEAD**, in a `git archive` copy with only these two
     files and their two registration lines added
     (`.temp/director/gate-635a1-calib/calib.sh`, logs beside it):
     - A is red: `a start taken under one lease recorded into the next`,
       with `DEFINITION_RHS_EVAL_COUNT` `[0]`→`[1]`.
     - B is red, and all nine sites wrote: `record_timing`,
       `record_profile_lexical_timing_elapsed`, `record_counter`,
       `record_add`, `record_max`, `rewrite_rule_prefilter_candidate`,
       `rewrite_rule_match_outcome`, `summary_once` and `snapshot`. Each
       site named exactly the counters it writes.
     - D is red, and all twelve flags were overwritten (for example
       `LEXICAL_PROFILE_ENABLED: published 1, after 2, answered false` and
       `LEXICAL_PROFILE_ACTIVE: published 2, after 1, answered true`).
     - The build had the same three `test_builtins.rs` warnings as HEAD, and
       no new one.
   - **On the change**, A, B and D are green.
2. **C, the fence.** Append it to `unregistered_writes.rs`, merging its
   imports (`std::sync::Arc`,
   `std::sync::atomic::{AtomicBool, Ordering}`):

   ```rust
   /// A lease that becomes current waits for every lexical write admitted
   /// before its generation was published.
   #[test]
   fn a_new_lease_waits_for_an_admitted_write() {
       let held = lease::lease();
       let admitted = lease::admit().expect("the holder's own thread is admitted");
       let returned = Arc::new(AtomicBool::new(false));
       let next = std::thread::spawn({
           let returned = Arc::clone(&returned);
           move || {
               let _next = lease::lease();
               returned.store(true, Ordering::SeqCst);
           }
       });
       drop(held);
       while lease::fencing() == 0 && !returned.load(Ordering::SeqCst) {
           std::thread::yield_now();
       }
       assert!(
           !returned.load(Ordering::SeqCst),
           "a lease became current while an admitted write was in flight"
       );
       drop(admitted);
       next.join().expect("next lease");
   }
   ```

   - It cannot compile at HEAD. Its red arm is mutant M1 below.
   - The wait loop is a wait on an event that comes in both arms: either a
     lease reaches the fence, or `next` returns. It is the one loop this
     amendment allows.
3. **The mutant arms.** Each is a copy of the changed tree with one
   deletion, built with its own target directory. Save both logs.
   - **M1:** delete the `fence_admitted_writes();` call in `lease()`. C must
     be red with `a lease became current while an admitted write was in
     flight`.
   - **M2:** delete the admission in `record_timing`. A must be red, and B
     must be red listing exactly `record_timing`.
   - On the changed tree, C is green and A and B are green.
4. **Regression.** Stage 1's falsifiers stay green on the change:
   - the chain pair at two threads, 10 runs;
   - both `owner_filter` tests;
   - the three C4′ tests (`epoch_contract`).
5. **The rum-runtime lib at 32 threads: 20 runs, all green.**

   ```
   cargo test -p rum-runtime --lib --offline -- --test-threads=32
   ```

   - Build first. Then bound each run at 300 s.
   - HEAD's rate is 4 red in 13 runs. If the fault were still there, 20
     green runs in a row would have a chance of about 0.06%
     (`(9/13)^20`).
   - Quote all 20 `test result:` lines.

### B1, redone

Replace stage 1's `b1` section with a 62-row table: the 58 lexical rows plus
`PROCESS_BYTES`, `CURRENT`, `LEXICAL_PROFILE_ENABLED` and
`LEXICAL_PROFILE_ACTIVE`. Each row gives:

- its declaration;
- **every writer site**, by `path:line`, from grep. For a counter written
  through a helper, give both the helper and each `records.rs` caller that
  passes this counter.
- every in-process reader test;
- its protection, which must be one of four:
  - **child**: the reader runs in a child process;
  - **lease**: every writer an unregistered thread can reach admits, or is
    refused by stage 1, under the test cfg;
  - **cold load**: the only writer an unregistered thread can reach is
    `publish`'s compare-exchange from 0. Cite the grep showing that no
    in-process code calls `set_var` or `remove_var`; my grep found none
    under `crates/` outside `rum-cli/tests` `Command` env;
  - **none**: no in-process test reads the row;
- the falsifier that covers it: an A/B/C/D site name, or a stage-1 falsifier.

A row with an in-process reader and none of the four is a **STOP**.

### Stages and stops

1. **Shape.**
   - Implement the target shape, then A–D, M1, M2 and the queue line.
   - `cargo build --tests --workspace --offline`, with no new warning.
   - Run falsifiers 1–3. STOP if any arm is not as stated.
2. **B1.** Write the table. STOP on an unprotected row.
3. **Frontier, replays, lib, workspace.**
   - **The frontier.** Run
     `RUM_CENSUS_DUMP=<absolute path> cargo test -p rum-stdlib --lib --offline production_rejection_census_is_complete`
     in `.temp/brief-635/a1/head/` and in the changed tree, writing
     `.temp/brief-635/a1/frontier-head.tsv` and `frontier-cand.tsv`.
     - The dump is written even when the test then fails on stale
       fingerprints.
     - Report both row counts and every row present in only one dump.
     - A moved row is a disclosure, not a stop. I rule on it at the gate.
   - Replay the standing inventories as in stage 3.
   - Run falsifiers 4 and 5.
   - Prove production builds with `nm -C` (0 lease symbols in the release
     `rum-cli` and the perf-panel harness) and with the feature tree.
   - Run `cargo test --workspace --no-fail-fast --offline --
     --test-threads=1`, bound 5400 s, in one 14G scope, with the nine native
     ZeroMQ `--skip`s.

### Forbidden

- A check in, or a change to, any `config.rs` accessor.
- A lease load, lock or branch on any flag-off path.
- An admission whose scope holds anything but atomics, thread-locals and
  `eprintln!`, a nested admission, or a lock taken inside one.
- Any wait in `lease()` other than the fence.
- `#[ignore]` as a fix, retries, sleeps, raised or guessed bounds,
  `--test-threads=1` as a fix, `serial_test`, or any dependency.
- Weakening any stage-1 or C4′ test.

### Report

- Write `.temp/brief-635/report-a1.json`.
- Quote every `test result:` line and every verdict line.
- Run `python3 scripts/report.py check 635 --stage a1` until it prints no
  PROBLEM.
- Your final message is `python3 scripts/report.py summary 635 --stage a1`.
