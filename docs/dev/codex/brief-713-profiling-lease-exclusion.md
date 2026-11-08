# Brief 713 — make the profiling test lease's exclusion total (implementation)

**Shape: implementation.** Brief 712 (design) is accepted; its inventory,
mechanism traces and calibrated probes are the map. This brief closes the
class at the shared-state boundary. The director's rulings on Q1-Q4 are in
`brief-712-profiling-test-isolation.md`'s gate section and are restated here
as constraints. The 37-run repeated-run campaign is a separate
source-unchanged validation brief; it is not part of this round.

## Goal

In test-cfg builds, while a lease is current, a thread that is not registered
to that lease can neither read a flag the lease modified nor touch any
profiling state the lease exposes — not the epoch or side index, not rows,
records, segments, stems or counters, not the stash, hook cells, queue owners
or the budget, not the legacy summary/scope/presence/timing maps, not the
evaluation-cache profile state. Every acquisition of current state is refused
before the first payload lock, transactionally with lease publication, so the
refusal cannot be raced by a lease that publishes between the check and the
access.

The refusal returns the disabled path's behaviour: no row, no counter, no
stash mutation, no hook call, no fault — and no guest-visible change. State a
thread already owns (a retained epoch or charge created before the lease) is
**caller-owned**: it is never refused, its cleanup and refunds always
complete, and it must be proven unable to reach the current lease's state.
Production binaries (no test cfg) compile exactly today's path.

## Context pointers

- `brief-712-profiling-test-isolation.md`: the gate section with the Q1-Q4
  rulings; the design report's `inventory`, `entry-map`, `mechanism`,
  `limits`, `locks`, `options` sections.
- `.temp/brief-712/evidence/inventory-table.tsv` (389 rows: 177 state, 212
  lease consumers) and `inventory-final.json`.
- `.temp/brief-712/evidence/trace-10.log` seq155/168/170 and
  `trace-parent.log` seq11/45/856: the two proven paths.
- `.temp/brief-712/tree/crates/rum-runtime/src/runtime/profiling/tests/brief712_probe.rs`:
  the two calibrated probes, red2 `(1,1,0)` / `stale_attach_no_fault=false`,
  green2 `(0,0,0)` / `true`.
- `crates/rum-runtime/src/runtime/profiling/lease.rs:28,43,63,82,99`: the
  existing lease, `WRITE_GATE` fence, `admit()` and the single-load
  `blocks_unregistered_thread()`.
- `crates/rum-runtime/src/runtime/profiling/logical/collector.rs:43,80`:
  the measured `mem::take` and its `drain_matching` caller.
- `crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs:624,637`:
  the attach entry, the assertion that must move behind the owner lookup, and
  the lookup.
- `crates/rum-runtime/src/runtime/profiling/timing.rs:674,704`: the adopted
  slow path that drains on the global flag.
- `crates/rum-runtime/src/runtime/profiling/tests/unregistered_writes.rs:105-132`:
  the fixture Q3 authorises changing.

## Scope

- Files you MAY modify (paths below `crates/rum-runtime/src/runtime/`):
  `profiling/{lease,config,counters,records,reset,summary,scopes,timing,
  presence}.rs`; `profiling/logical/{mod,collector,epoch,context,segments,
  totals,record,queue_api,module,synchronous}.rs`;
  `profiling/logical/epoch/snapshot.rs`;
  `profiling/logical/segments/stack.rs`;
  `profiling/logical/publication/{mod,rows}.rs`;
  `evaluation_cache/profile/{mod,source,report}.rs`;
  `suspension/core/{profile_queue,storage}.rs`;
  `profiling/tests/**` and `suspension/core/profile_queue/tests.rs`. New
  private test-support files may live under `profiling/tests/`.
- MUST NOT touch: `crates/rum-core/**` (Q4); any public API, feature, serde,
  manifest, dependency or `Cargo.lock`; `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  `docs/**`, `docs/dev/codex/**`; `.git`; anything outside this repository.
  No `unsafe`. No new skip, ignored test, env-hidden test, default thread
  count change or whole-suite serialization.

## The contract this brief installs (rulings built in)

1. **Reads and writes of current state are transactions on the existing
   fence.** A test-cfg payload transaction or flag getter takes
   `lease::admit()` (the existing `WRITE_GATE` read) before its check and
   holds it across the access. `blocks_unregistered_thread()` keeps its
   single-acquire-load shape for the early-out sites that do not need a
   transaction; it is not the strict read path. **No second lock or fence is
   introduced.**
2. **Publication is one transaction including the flag swap.** Every test
   helper that stores a process-global profiling flag — the twelve flags in
   `engine_support` / `publication::support::Flags`, the three cache flags
   through `Session::test_cache_profile_flags`, any raw store — performs the
   swap while holding the **write** side of the same fence (`publish()` in
   the shape below). A reader holding the read side then cannot observe a
   modified flag across publication. A cold-load publish
   (`publish_cold_load`) stays where it is: it is reached only inside an
   admitted getter.
3. **Cleanup of caller-owned state is never refused.** `Storage`,
   `RecordPath`, budget refunds and TLS destructors complete on whatever
   thread they run. A bookkeeping-only teardown that cannot safely run (for
   example a foreign `drop_session` clearing the global stash) is deferred to
   the next `reset_for_lease`; the proof must show no charge is lost and that
   the reset releases it **before** `drain_for_lease` waits on the process
   byte counter.
4. **The refusal preserves guest semantics.** A refused diagnostic entry
   must not skip guest continuation, clear or alter a guest queue, change a
   guest result, retire the holder's epoch, or change owner matching.
5. **No nested admission.** A thread holding a permit never re-acquires; the
   permit is threaded to the `_admitted` bodies. Any path that cannot be
   classified this way is a STOP, not a judgement call.
6. **Scope is the item-3 boundary.** Independent env-only profilers
   (`RUM_PROFILE_REPR`, substitution-sharing, association) are out of scope;
   the inventory says why each is lease-independent, and this change must not
   touch them.

## Target shape (code; a departure is a STOP)

The exact test cfg, repeated verbatim at every added check, permit and
parameter (no named cfg alias exists on stable Rust):

```rust
any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots")
```

**Primitives, in `profiling/lease.rs` (all under that cfg).**

```rust
/// The existing read transaction; unchanged.
pub(in crate::runtime) struct Admitted { _gate: RwLockReadGuard<'static, ()> }

/// Test-only publication transaction. Waits out admitted readers; the caller
/// stores the flags while it is held. Refuses a thread not registered to the
/// current lease. Holds no payload lock when acquired.
pub(in crate::runtime) struct Publication { _gate: RwLockWriteGuard<'static, ()> }

pub(in crate::runtime) fn publish() -> Option<Publication> {
    let gate = WRITE_GATE.write().unwrap_or_else(|poison| poison.into_inner());
    if blocks_unregistered_thread() {
        return None;
    }
    Some(Publication { _gate: gate })
}
```

Add a test-only reentrancy instrument: a thread-local admitted-depth counter,
incremented while `Admitted` or `Publication` is alive; `admit()` and
`publish()` `debug_assert!` the depth is 0 before acquiring. It is an
instrument, not the exclusion; one gate test below must show it fires on a
nested call.

**Every getter in `config.rs` and the three cache gates — wrapper/body
split.** The wrapper admits; the body is today's text; composite getters call
the admitted body of their callee, never the wrapper.

```rust
#[cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))]
#[inline]
pub(in crate::runtime) fn lexical_profile_enabled() -> bool {
    let Some(access) = lease::admit() else { return false; };
    lexical_profile_enabled_admitted(&access)
}

#[cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))]
fn lexical_profile_enabled_admitted(_access: &lease::Admitted) -> bool {
    /* today's body, including the cold load */
}

#[cfg(not(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots")))]
#[inline]
pub(in crate::runtime) fn lexical_profile_enabled() -> bool {
    /* today's body, unchanged text */
}
```

Apply the same shape to every entry in the item-3 inventory: the profiling
entry functions (adopted timing, counters, records, presence/scopes/timing
maps, reset, summary, snapshot), `Epoch` acquisition and retained operations,
the collector entries, `register_owner`/`attach_boundary`/unregistration,
`Segment`/`Context` entry points, the budget's acquisition entry, and the
cache getters and writers. Private bodies take `&lease::Admitted` through a
test-cfg parameter; the production bodies are the same text with the
parameter erased.

```rust
pub(in crate::runtime) fn drain_attached() {
    #[cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))]
    let Some(access) = lease::admit() else { return; };
    drain_matching_admitted(
        #[cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))]
        &access,
        |_| true,
    );
}
```

**Collector.** The global stash is never emptied by a non-owner and never
take-all/restore-later. Replace `take_stash` + `restore_stash` in the
transaction paths with an ownership-scoped removal under the STASH lock:

```rust
/// Removes only the boundaries `predicate` selects, in one locked step.
/// Every other boundary stays in place. Callers process the result outside
/// the lock, after copying the hook out of its mutex and dropping that lock.
fn take_selected(predicate: impl Fn(&Arc<ProfileBoundary>) -> bool) -> Vec<Arc<ProfileBoundary>>
```

`drain_attached`, `drain_attached_for` and `drop_belonging` become the
admitted bodies above; the re-stash of the caller's unattached boundaries is
part of the same transaction. A foreign drain returns before touching the
stash. The hook cells are copied and their locks dropped before any callback,
as the current code does.

**Attach ordering.** In `attach_boundary`: refuse an unregistered caller
first, then resolve the owner, then keep the assertion with its **unchanged**
allowed policy variants. A live owned boundary with `None` policy still
asserts; a boundary with no matching owner returns `false` before the
assertion. Deleting the assertion or accepting `None` for an owned boundary
is a STOP.

**Epoch.** Every acquisition entry (`storage`, `event`, `event_rows`,
`events`, `open`, `existing`, `publication_state`, `current`) takes the
permit **before** `current_slot()`; the post-lock recheck is replaced by the
permit, not deleted. Retained operations are classified per contract point 3
and 4: a handle the thread owns proceeds; a global effect inside it
(`clear_registrations`, `clear_stash`, owner removal) is either run by an
admitted caller or scoped to the caller's own entries. `drop_session` is
refused for an unregistered thread and deferred to the next
`reset_for_lease`. Name every classification in the report.

**Budget.** `Storage` acquisition is an admitted entry. `Drop`, refunds and
`release_deferred_charge` are never refused and never early-returned; the
report proves their lock order and that a prior-generation refund cannot be
observed as the current lease's row.

**Cache.** The three gates follow the getter shape; `test_cache_profile_flags`
swaps under `publish()`. Every writer of the 21 counters and the four global
maps gets its permit at its entry, before its map lock, or is proven
unreachable from an unregistered thread with the flags on.

**Summary/reset.** `print_lexical_profile_summary_once` and
`print_lexical_profile_snapshot` read the strict getters, so an unregistered
thread takes the no-lexical path and still runs the independent repr path.
`reset_unleased` is reachable only from the holder or the production reset
token; prove it.

**Test fixtures.** `engine_support::enabled()` and `publication::support::
Flags::mode` (and their `Drop`s) take `publish()` around their swaps; a
`None` return is a STOP (the caller is the lease holder). Apply the Q3 ruling
to `tests/unregistered_writes.rs:105-132`.

## Gate tests, probes and mutants

Every gate is a live test in the final tree. Each must be **watched red on the
pre-fix tree** (before the fix; the probes are inserted first) and green after,
and each must have one single-site mutant of the fix that turns it red. Save
every log; restore every mutant.

| Gate test | Observer (fixed expectation) | Pre-fix red | Mutant that must redden it |
| --- | --- | --- | --- |
| `brief712_unregistered_stash_theft_leaves_open_row` | `(stolen,open,faults)==(0,0,0)` | `(1,1,0)` | remove the collector transaction / restore take-all |
| `brief712_unregistered_stale_attach_must_not_fault` | `stale_attach_no_fault=true` | `false` | refuse-last ordering + assertion before lookup |
| `brief713_unregistered_flag_read_is_refused` | an unregistered thread reads `false` for every one of the 15 flags while the holder's are on | reads `true` | remove the getter admission for one flag |
| `brief713_flag_swap_cannot_be_observed_across_publication` | a reader parked between its generation load and flag load never observes the swap; publication waits (`fencing()>0`) while it is parked | observes the swap | swap outside `publish()` |
| `brief713_cache_write_from_unregistered_thread_is_refused` | named cache counters/maps unchanged; no epoch fault | changes | remove the cache writer's permit |
| `brief713_registration_crossing_publication_is_fenced` | hook install + owner insert is one transaction; a lease between them waits or the write is refused | write lands across publication | shorten the permit to the first call |
| `brief713_prior_generation_cleanup_completes` | a pre-lease `Storage`/`RecordPath` dropped on an unregistered thread during a lease refunds, faults nothing, changes no current row | (on the fixed tree only; the observer is the Q2 contract) | refuse the refund |
| `brief713_nested_admission_is_caught` | a nested `admit()` from inside a body trips the depth assertion | n/a (instrument) | call the wrapper from inside an admitted body |
| `brief713_unregistered_start_is_refused` | `lexical_profile_start()` returns `None` on an unregistered thread under the flag | `Some` | remove the start's admission |

The two `brief712_*` names, bodies and expected tuples are the calibrated
falsifier and stay; only the API surface they call may be re-expressed
against the final signatures. `brief713_prior_generation_cleanup_completes`
must also witness `lease_release.rs:50`'s pre-enable drain still passing.

## Disabled-cost bar and must-not-change list

- Production: every added check, permit, parameter and helper is erased by
  the test cfg. Build the default release library and show that
  `lease`, `Admitted`, `Publication`, `WRITE_GATE`, `CURRENT_GEN`, `admit`
  and `publish` have no symbols and no strings in it. The rum-vs-rum
  all-label instruction/timing campaign is the validation brief's; do not run
  it here.
- Must not change: capture positions, event order, phase ordering, policy
  variants, aggregate keys, row/schema/additivity semantics, legacy bytes,
  source-stack lifetimes, boundary completion/abandonment, budget accounting,
  owner matching, guest resume/cancel behaviour or any guest result. No
  duplicated bodies (one body, cfg-erased parameter), no silent drops, no
  lazy init, no `debug_assertions`-keyed inlining, no whole-suite
  serialization, no env-hidden test, no assertion weakened.
- **Test expectations stay exactly as they are at HEAD** (they are the
  serial-green, interference-free values). If a required change to an
  expectation appears, STOP: report the old value, the new value and the
  thread's registration state, and let the director rule. The only
  authorised fixture change is Q3's.

## Standing inventories (always in scope)

The edit expires the retention contract fingerprints and moves ownership
positions. Replay canonically as the last step of the edit stage, before the
first acceptance suite: the contract fingerprints, `local_production_acceptance`
in the default and `--no-default-features` builds, the ownership-graph rows,
LP01 (`suspension_oracle -- logical_counter_port`), the typed-slot census and
the rejection census counts. Never hand-edit a derived cell. A suite red
whose only cause is a stale derived cell is an ordering artifact: replay, do
not report it as a regression. Stop for the director on an identity/status/
disposition/exclusion change of an existing row, a new `UnknownHostPayload`,
or a new row pointing at untouched code; new `UnknownConstruct` rows for code
this brief adds are expected and disclosed.

## Verification (run exactly these; the workspace run is the director's)

```
cargo fmt --check
cargo check --workspace --all-targets
cargo build --release           # erasure check on the default library
cargo test -p rum-runtime --lib --features test-builtin-memory-growth,test-continuation-slots --offline --locked -j 8 --no-run
target/debug/deps/rum_runtime-<hash> --test-threads=8 --skip lp655_root_context_falsifier --skip lp656_public_root_context_falsifier
target/debug/deps/rum_runtime-<hash> --test-threads=1 --skip lp655_root_context_falsifier --skip lp656_public_root_context_falsifier
cargo test -p rum-runtime --lib --features test-builtin-memory-growth,test-continuation-slots --offline --locked -j 8 --no-fail-fast
cargo test -p rum-stdlib --lib --offline --locked -j 8 --no-fail-fast
cargo test -p rum-stdlib --test suspension_oracle -- logical_counter_port --offline --locked -j 8
cargo test -p rum-stdlib --test typed_slot_census --offline --locked -j 8
```

After the fix, run the full lib at 8 threads **three consecutive times** and
once at 1 thread. After the fix, any full-lib red at 8 threads that is green
at 1 is a STOP with the full log — it is the class this brief exists to
close, not a pre-existing flake to accept. Any red that reproduces at 1 is a
defect. The nine native ZeroMQ host tests are the director's; do not run the
full workspace.

Restate the instrument's standing exclusions: exactly the two root-context
falsifier skips above, at 8 and 1 threads. Do not add another.

## Deliverable

- The implementation in the working tree, with the new gate tests.
- `rum-report/1` JSON at `.temp/brief-713/report.json`: the class-closure
  map (every item-3 entry -> admitted body -> payload lock -> permit; the
  caller-owned class; the deferred-cleanup class), the probe calibrations
  (pre-fix red tuples and post-fix green with log lines), the mutant table
  (one row per gate, the mutation text and the red observer), the
  flag-writer transaction proof, the lock-order table, the test-name list
  before/after with exact counts, every replay verdict line, the erasure
  symbol check, and every disclosure.
- `python3 -I scripts/report.py check 713` and its summary.

## Staged stops

1. Insert the gate tests only, watch every one red on the pre-fix tree, and
   record the exact tuples/traces. A gate that cannot be made red is a STOP
   (the instrument is not calibrated) unless it is the instrument-only check
   named above.
2. Add the lease primitives (`Admitted` depth, `Publication`, `publish`) and
   convert the flag writers and getters. Run the flag/cache gates.
3. Convert the state entries surface by surface in the order collector,
   owners/attach, epoch, segments/context, summary/reset, budget. After each
   surface: compile, run its gates, and check the lock order. Stop on an
   unclassified entry, a forbidden lock edge (payload lock -> admission), or
   a gate that stays green.
4. Cleanup/retained classification, then replay the standing inventories,
   then the acceptance runs above. Report; the director gates and owns the
   workspace run and the 37-run campaign.

## Checkpointing

All scratch, logs and the PROGRESS file under `.temp/brief-713/`. Keep the
tree compiling at every checkpoint; record the exact next action, the
mutants currently applied or restored, and the receipts already saved.
Preserve every red and timeout.

## Hard rules

AGENTS and TEMPLATE apply. Safe Rust; rum-only and rum-vs-rum; no third-party
differential engine. No git command. No public API, feature, manifest or
dependency change. No assertion weakened, no skip added, no default thread
count changed. A live inventory edit only through its canonical replay. A
forbidden lock edge, a changed guest behaviour or an unclassified item-3
route is a STOP. Leave the changes uncommitted for the director's gate.

## Director ruling — 2026-11-08 (stage-1 stop): the fixture scope is extended by class

The stop is accepted, and the calibration is accepted. The Scope omission was
mine: `evaluation_cache/profile/tests/cold_load.rs` stores a process-global
flag and calls the cold-load initializer directly, so contract point 2 reaches
it, and the file was not in the MAY list. Editing outside the fence to satisfy
a contract would have broken the fence; stopping was correct.

**Ruling.** The MAY list is extended **by class**, not by one file: any
test-only fixture module under
`crates/rum-runtime/src/runtime/profiling/` or
`crates/rum-runtime/src/runtime/evaluation_cache/profile/` that stores a
process-global profiling flag, or calls a cold-load initializer directly, may
be modified **only** to place that store (and that initializer call) under the
write-side `publish()` transaction. Reads stay where they are. No test name,
assertion, expectation, observer, thread count or default may change. Every
file touched under this class is disclosed in the report, and a `publish()`
that returns `None` in a lease-holding fixture is a STOP (the lease registers
its holder). The known file is
`evaluation_cache/profile/tests/cold_load.rs` (stores at :14 and :17, the
initializer at :15); the stage-2 audit must list every other file this class
reaches.

Everything else in the brief stands. The seven pre-fix gates and their tuples
are accepted as the falsifier's red arm; the first registration sample
(`fenced=true`) is superseded by the refined observer (`fenced=false`) and is
not used. The cleanup and nested-admission gates are fixed-tree-only
instruments and must be installed together with the primitives. Stage 2 may
resume from these receipts; the tree at the stop compiles (I verified
`cargo check -p rum-runtime --all-targets` with both test features: rc=0,
three pre-existing warnings in `test_builtins.rs`).

## Director ruling — 2026-11-08 (stage-2 stop): Q1-Q3 settled

The stage-2 stop is accepted; all three hard stops are mine to settle. The
fixture Q3 change and the 11 disclosed fixtures are accepted as the
authorised setup-only shape. Stage 2 resumes at the lock-order repair.

**Q1 — cache writer transaction and retained aliases: authorised, this
shape.** Every cache writer entry takes its admission **before** its first
payload lock and passes the permit to its body; the incremental getter inside
the body is the `_admitted` variant. Concretely, `record_symbol_profile_counter`
takes `admit()` at its entry (replacing its own getter call) and passes the
permit to `record_symbol_profile_counter_slow`, which locks
`symbol_profile_counters` under it and calls
`evaluation_cache_profile_incremental_enabled_admitted`. Every other writer
with the same edge (getter -> payload lock -> admitted getter) is repaired
the same way; the stage-2 audit's list is the checklist.

The retained-alias publication (`totals::terminal/settle` ->
`Session::module_source_alias_completion/duration`) is classified at its
retained entry: every entry that reaches `totals::terminal/settle` obtains an
`Option<Admitted>` **before** `Epoch.data`/`RecordPath.state`, and the alias
leaf writes only under `Some`. Under `None` (unregistered thread, lease
current) the epoch's own rows still settle — caller-owned cleanup — and the
process-global alias write is skipped. No admission is acquired beneath a
payload lock; a path that cannot be shaped this way is a STOP. Entries called
from inside an existing permit take the admitted variant; a second `admit()`
on the same thread is the depth-stop, not a fallback.

**Q2 — ownership pairing: accepted with an explicit disposition.** I ran the
descriptor-multiset comparison myself between the live inventories and the
candidate: `unknown_attributes.tsv` +79/-0, `unknown_captures.tsv` +7/-0,
`unknown_storage.tsv` +3/-0, `unknown_macros.tsv` +41/-39 (the 39 removals are
the same macros whose `ends at L:C` descriptor moved), the three pointer files
0/0. No committed construct disappears, and every added row is on code this
brief added. The two unchanged-line flags are the duplicate-descriptor
ambiguity the stop names: original `counters.rs:74` -> current `:92` and
original `:135` -> current `:177`; the first-match pairing mislabels them NEW.
**Disposition: both are MOVE, not NEW.** The candidate under
`.temp/brief-713/evidence/ownership-replayed/` is the replay's output and may
be copied into the live inventories as the canonical replay; do not hand-edit
a row. Record the disposition in the report.

**Q3 — rejection census: accepted under a predicate.** The keyed comparison
is exactly the allowed class: 10 added and 6 removed, all inside the three
files this brief converts, and the added reasons are the new test-cfg
attributes and cfg'd parameters (`method lexical scope contains local
items/imports`, `unresolved non-test cfg alternative or opaque import
namespace`) replacing `method receiver lacks checked nominal/place
provenance`. At the end of stage 3 the pinned counts in
`retention_closure/census/tests.rs` may be updated to the final discovered
values as a derived cell **if and only if** every added or removed key is in a
file this brief converted and every added reason is one of those two classes
(or another reason directly attributable to a new cfg or parameter); every
added and removed row is listed in the report. Anything else is a STOP.

**Warnings.** Remove the unused import this brief introduced in
`counters.rs` (the one new lib-test warning). Do not touch the out-of-scope
`profiling/mod.rs` re-export; if a warning comes from it, name it and leave
it.

Everything else in the brief stands: the surface order, the lock-order table,
the nine gate tests and their mutants, the standing replays after the last
edit, and the acceptance runs. Resume with `CDX_FORK`.
