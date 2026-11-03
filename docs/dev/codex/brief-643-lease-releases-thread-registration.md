# Brief 643 — the test lease releases its thread registration

**Shape: IMPLEMENTATION, test-support only. Scope: the profiling test lease
and its TLS registration. No production behaviour change, no new test
semantics, no assertion weakening.**

## Goal

A profiling test lease must leave the process's diagnostic charge exactly
where it found it. Today it does not: the TLS registration that
`segments::install` charges on a thread's first segment
(`segments.rs:156-166`, `size_of::<RefCell<Stack>>() + 4096` = 4,232 bytes,
held in `Stack.header`, `segments.rs:44`) is released only when that thread
is destroyed. libtest runs each test on its own thread, so the release
happens *after* the test has dropped its lease, and the ~4 KB decrement can
land inside the next lease holder's accounting window.

This is reachable today and got much easier to reach when brief-642 A1 added
ten leased tests next to the budget tests: `cargo test -p rum-runtime --lib`
at eight threads reds about 3 times in 20 in
`profiling::logical::budget::logical_expired_queue_arena_stays_charged_until_
weak_drop` (`budget.rs:176`: `parked - initial` subtracts with overflow),
with 0/20 when the assignment tests are skipped and green at
`--test-threads=1`. Demand it become deterministic: the fix is to release
the holder thread's registration when the lease itself ends, while the lease
is still held, so no deferred decrement survives it.

The director's probe (`.temp/director/probe-643/`) already confirms the
diagnosis and the shape: releasing under `TestLease::drop` gives 20/20
green, and a falsifier that samples the process charge before a segment and
after the lease drop/re-acquire is red on the unfixed tree with
`left: 4232, right: 0` and green on the fixed tree. Use it as evidence, not
as code to copy: your implementation and falsifier are yours, and they must
calibrate the same way.

## Context pointers

- `crates/rum-runtime/src/runtime/profiling/logical/segments.rs` — the TLS
  `Stack` (`:40-56`), `install` (`:150-175`, the header charge and
  `ACTIVE_STATE`), `active_len` (`:92`), `segments/stack.rs` (the count and
  the `storage`/`header` fields; `storage` is freed at count zero, `header`
  is not).
- `crates/rum-runtime/src/runtime/profiling/lease.rs` — `TestLease`
  (`:22-26`), `Drop` (`:87-93`), `blocks_unregistered_thread` (`:98-105`).
  The module is compiled under `test` or the two test features only.
- `crates/rum-runtime/src/runtime/profiling/logical/budget.rs` — the process
  ceiling (`:7-8`), `Budget`/`Storage` drop accounting (`:53-85`), and the
  two tests that read `PROCESS_BYTES` directly (`:105-181`). The failing
  assertion is `:176`; the metadata test (`:112-156`) is the contract the
  fix must not weaken.
- `crates/rum-runtime/src/runtime/profiling/tests/` — `engine_support.rs`
  (`Enabled`, the lease wrapper), `unregistered_writes.rs`, `owner_filter.rs`
  (threads registered with `allow_current_thread`; their registrations are
  charged on the worker thread and released at its exit inside the test).
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs:1028-1041` — the
  test-module registrations (`evaluator`, `definitions`, `assignments`).
- `docs/dev/codex/TEMPLATE.md` — the standing-inventory duty under Scope.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/profiling/logical/segments.rs` — a
  test/feature-gated release of this thread's registration. No change to the
  production charge, its size, `install`, or `active_len`.
- `crates/rum-runtime/src/runtime/profiling/lease.rs` — call it in
  `TestLease::drop`, while the lease is still held.
- `crates/rum-runtime/src/runtime/profiling/logical/budget.rs` — a
  test-gated accessor for `PROCESS_BYTES` (the falsifier needs the process
  charge). Additive only; the two existing tests' bodies stay byte-for-byte.
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` — register the
  new test module at the foot, beside `assignments`.
- `crates/rum-runtime/src/runtime/profiling/tests/lease_release.rs` (new) —
  the falsifier.
- the standing generated inventories named under **Inventory replay**.

Files and areas you MUST NOT touch:

- any production semantics, the header's production lifetime for threads
  that are not lease holders, `segments::install`, `active_len`, the
  `Budget`/`Storage` accounting, or the TLS layout beyond what the release
  itself needs;
- the assertions, names, attributes or bodies of
  `logical_expired_queue_arena_stays_charged_until_weak_drop`,
  `logical_thread_metadata_stays_charged_until_thread_exit`,
  `logical_initial_cap_failure_stays_invalid_until_reset`, the
  `unregistered_writes` and `owner_filter` tests, or `engine_support.rs`'s
  `Enabled` contract (a lease holder still sees `configured`/`active`
  restored on drop and the epoch reset);
- `Cargo.lock`, `AGENTS.md`, `TODO.md`, other briefs, docs, `.git`, any
  `unsafe`, any dependency, any new feature, any `#[ignore]`.

**Standing maintenance duty (always in scope).** The retention contract
fingerprints hash every `.rs` under `crates/`, so these edits expire
`inventory/local-contracts.tsv` and the certificate cells in
`inventory/promoted.tsv`; the ownership discovery and the suspension-oracle
producer inventories may also move. Replay exactly as `TEMPLATE.md`
describes, and reconcile the ownership rows against the discovery the tests
print. Replay is the last step of the edit stage, before the first
acceptance suite, and every changed row is disclosed. A red whose only cause
is a stale derived cell is an ordering artifact, never a regression. STOP
instead if a row committed at HEAD would change its identity, status,
disposition or exclusion, or would disappear, or if a new
`UnknownHostPayload` row appears, or if a new row points at code this brief
did not touch.

## Deliverable

Stage 1 — **the release, then its falsifier.**

1. Release the holder thread's TLS registration when its `TestLease` ends,
   under the terms in Goal. It must be a no-op when the thread has active
   segments (a lease that ends mid-segment must not free the registration
   under the stack); it must reset the header-charge state so a later lease
   on the same thread re-charges; and it must be compiled only under the
   same `cfg(any(test, feature = "test-builtin-memory-growth",
   feature = "test-continuation-slots"))` set as the lease module and
   `blocks_unregistered_thread`. Production builds must contain no such
   symbol: verify with the existing release-build check the template names
   (or `cargo rustc --release -- --emit=llvm-ir` and a grep for the symbol).
2. Write the falsifier. On one thread, under a lease: sample the process
   charge, enter and close a call, observe the charge above the baseline,
   drop the lease, re-acquire the lease (so no other leased test can be in
   flight while sampling), and assert the charge is exactly the baseline.
   The sampling accessor must be test-gated. It must fail on the unfixed
   tree with the 4,232-byte remainder visible in the assertion, and pass on
   the fixed tree.
3. Calibrate in a scratch copy under `.temp/brief-643/`: the copy differs
   from the tree only by removing the release call (the unfixed arm); the
   falsifier must be red there at its final assertion with the remainder
   quoted, and green in the authoritative tree. Quote both verdicts.

Stage 2 — **the exposure, the existing contracts, the inventories.**

4. Reproduce the exposure on the authoritative tree: the runtime lib at
   eight threads, at least 20 runs, zero failures; and the before rate in a
   scratch copy with the release removed (or from the director's probe),
   quoted with its counts. The falsifier is the artifact; the twenty runs
   are the demonstration that it covers the observed class.
5. The two budget tests, the metadata test, `unregistered_writes` and
   `owner_filter` must all stay green. `logical_thread_metadata_stays_
   charged_until_thread_exit` still asserts the worker thread's charge
   survives `drop(session)` and `epoch::reset_for_lease` and is released at
   its exit; that contract is not to be edited or widened.
6. Replay the standing inventories and run:
   ```
   cargo fmt --check
   cargo test -p rum-runtime --lib --offline -- <the falsifier filter>
   # the two budget tests and the metadata test, separately (libtest takes one filter)
   cargo test -p rum-runtime --lib --offline --no-fail-fast   x 20 at eight threads
   cargo check --workspace --all-targets --offline
   ```
   The full workspace suites belong to the director's gate; the dispatcher's
   sandbox cannot own a workspace-sized OOM domain, so do not run
   `cargo test --workspace` here.

## rum / Rust pitfalls (read twice)

1. `TestLease` is `!Send` (it holds a `MutexGuard`); the release must run on
   the thread that holds the lease, and `TestLease::drop` is exactly that
   thread. A thread registered with `allow_current_thread` does not drop a
   `TestLease`; its registration stays until its own exit, and the metadata
   test joins it inside the test. Do not "fix" that path.
2. `ACTIVE_STATE` is `count + 1` while the stack is live and `1` when only
   the header exists; release only at count zero and reset it to zero
   exactly when the header is taken, so the next lease re-charges.
3. The charge is `size_of::<RefCell<Stack>>() + 4096` today. Do not hardcode
   the number in production code; the falsifier may compute/expect it at
   most as its calibrated observation.
4. The release must not allocate, must not touch the epoch, and must not run
   any production-gated path: it is a take of one `Option<Storage>` plus one
   `Cell` store.
5. `PROCESS_BYTES` is process-global. The falsifier's sampling is stable
   only while it holds the lease (no other leased test in flight) and only
   because the fix makes every prior lease leave it balanced. If the last
   sample is taken without the lease, it proves nothing.
6. Do not weaken the budget test: its `parked - initial` equality is the
   contract that exposed this; a saturating or tolerant variant is a STOP.
7. No new warning, no duplicated body, no `expect` on a state the types
   allow, no file-size jump. Grep the diff before reporting.

## Verification (MANDATORY — you run it; iterate to green)

Commands are in Deliverable 6 plus the calibration runs in 3 and 4. Paste
verbatim: the falsifier's red assertion and its `left`/`right` on the
unfixed copy, its green verdict on the tree, the twenty-run tally, and the
two budget tests' verdict lines. Bounds: derive each timeout from a measured
run of the same shape and use `scripts/run-bounded.py`; do not guess.

## Checkpointing — protect the scene

All scratch under `.temp/brief-643/` (copies, logs, PROGRESS.md, report).
After each stage write `.temp/brief-643/PROGRESS.md`: done and verified with
saved evidence, the exact next action, the tree's state (compiling, which
tests are green, any half-edited file), and what the director needs to take
over. Leave the tree compiling at every checkpoint.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Leave the tree for
  the director's gate.
- No production edit beyond the test-gated release; no test weakening, no
  assertion widening, no new `#[ignore]`, no `unsafe`, no network, nothing
  outside this repo.
- The scratch copy is an evidence container: it may differ from the tree
  only by the one authorized removal of the release call, and it is deleted
  after its receipts.
- Testing Policy is a hard line: rum's own behaviour only.
- Minimal diff: no drive-by formatting, renames or comments outside the
  release, the accessor, the falsifier and its module registration.
- Paste real verbatim output. A run that did not reach the evidence says so.

## Report format

`rum-report/1` at `.temp/brief-643/report.json`; every run with its command,
log, exit code and test counts; `changes` lists every touched file with line
ranges and the reason; `disclosures` lists every inventory row that moved;
`questions` carries any STOP. Final message is exactly
`python3 scripts/report.py summary 643`.

## Director decision — Q1 (2026-11-03): the baseline moves under the lease

Round 1 is accepted as a diagnosis and a correct patch: the original
`parked - initial` underflow is gone in all twenty eight-thread runs. The one
remaining red is a different, pre-existing sampling error, and Q1 is
**authorized, narrowly**:

- In `logical_expired_queue_arena_stays_charged_until_weak_drop`
  (`budget.rs:158-181`), move the `before` sample so it is taken **after**
  `lease()`. Today it precedes the acquisition, so it can include a
  concurrent test's in-flight charge (the observed 20,136 bytes); the final
  equality then compares against a value that was never this test's
  baseline. Under the lease the baseline is taken with no other leased test
  in flight, which is the premise the assertion always needed.
- Nothing else changes in that test: the assertion text, both literals,
  `initial`, `parked`, the `epoch`/`queue` lines and the drop order stay
  byte-identical. This is not a weakening; a changed comparison or a
  tolerance is a STOP.
- Round 2 acceptance: the twenty-run eight-thread exposure reaches **20/20**,
  the falsifier stays red on the unfixed copy and green on the tree, the
  serial runtime suite and all focused contracts stay green, and any failure
  of a different assertion is a STOP with its log.
- The standing inventories were already replayed in round 1; replay again if
  this edit moves any derived cell.
