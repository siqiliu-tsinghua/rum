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

## Director ruling — 2026-11-08 (stage-3 stop): Q4-Q6 settled; the attach witness is a layer matrix

The stage-3 checkpoint is accepted as a checkpoint: the collector and owner
entries are converted, nine of the ten focused gates are green, the ordinary
cache lock-order edge is closed, and two single-site mutants went red and
were restored. The stop itself is correct on all three counts. This round
names the remaining work precisely; no completeness is claimed.

**Q4 — the attach witness: calibrate the layered property, not a masked
single site.** With `take_stash`/`restore_stash`/`drain_attached` admitting
first, the immutable outsider cannot reach `attach_boundary_admitted`; that
masking is the class contract working, not a defect, and neither the probe
nor its expectation changes. A layered property is calibrated as a matrix
with the frame held fixed and one site varied:

- Frame: in `lease.rs`, make `blocks_unregistered_thread()` return `false`
  at its first line — a single-line mutation that disables the registration
  predicate (the read-side exclusion layer) while leaving `WRITE_GATE`'s
  read and write paths intact. It is the experimental condition, not a
  second cause.
- Arm A (measured): attach body mutated, frame intact -> green `true`. The
  mask; already recorded.
- Arm B: frame + the fixed attach body -> must be green `true`. Proves the
  attach layer alone holds the property for this probe.
- Arm C: frame + the attach body mutated (policy assertion before the owner
  lookup, foreign check after it) -> must be red
  `stale_attach_no_fault=false`.

Between B and C the attach body is the only variable; the flip is the
calibration. If arm B is red, that is a real defect in the attach body, not
a calibration artifact: STOP. Disclose every mutated line per arm, restore
byte-for-byte with the SHA, keep the logs under `evidence/mutants/`. The
pre-fix red stays the falsifier's red arm.

**Q5 — ownership candidate: accepted, with two dispositions.** I verified
each flag against the source:

- `collector.rs:20/32/45` (`DrainFn`/`ResetFn`/`DropFn`) are genuine status
  changes. The production alias lines are unchanged text now guarded by
  `#[cfg(not(any(...)))]`, each with a new test-cfg twin carrying the
  `&Admitted` parameter. The item is conditionally defined where it was
  unconditional; the new `ambiguous conditional definition` row is the
  canonical record of that item, on an unchanged line because the production
  variant's text did not move. Admitted as conversion-introduced, not
  drift. Record the two variant lines per alias in the disposition note.
- `epoch.rs:514` is the duplicate-descriptor pairing artifact, not a new
  construct. The baseline row at `epoch.rs:479` is this same
  `#[cfg(any(test, feature = ...))]` attribute on `reset_for_lease`, shifted
  +35; the descriptor's multiplicity grew with the conversion's new cfg'd
  parameters, and first-match pairing consumed the baseline instance
  elsewhere. **Disposition: MOVE**, the same class as `counters.rs:74 -> :92`
  and `:135 -> :177`. Name the consumed instance in the pairing note.

`evidence/ownership-replayed-c3` is the replay's canonical output and may be
copied unchanged; do not hand-edit a row. The accepted stage-2 copy stays
live until that copy lands.

**Q6 — rejection census: the predicate gains one multiplicity class.** The
stage-2 predicate admitted the two new cfg reason classes only. It did not
anticipate a new *instance* of a reason class the frontier already carries,
which is what the conversion's new `selected` vector and `retain` closure
produce. Extended predicate: a changed key is admitted iff (a) it lies in a
file this brief converted, and (b) either its reason is one of the two new
cfg classes or parameters, or its `(kind, callee, reason)` class already
exists in the accepted frontier **and its source position is on a line this
brief added** (a multiplicity addition from conversion-introduced code).
Every added and removed row is listed in the report. The two flagged rows
are admitted under the second class:

- `collector.rs:175` `Vec::new`: the new selected vector. The baseline
  instance at `:72` is the keep vector now at `:326` (the keyed pairing pairs
  them); the class count is one old and one new.
- `collector.rs:328` closure: the new `retain` predicate. The
  closure-capture class exists in the baseline (`:15`, `:20`, `:59`, `:74`).

Pinned counts in `retention_closure/census/tests.rs` are updated once, at
the end of stage 3, with the per-class delta table; a key outside the
predicate is a STOP.

**Warnings — the scope extension by class now covers two named items.** The
unused import is `profiling/mod.rs:41`, `lexical_profile_incremental_enabled`
in the private `use config::{...}` list; `counters.rs` is clean. This
supersedes the stage-2 "do not touch" sentence: the name became unused only
because this brief's conversion moved its callers, so **remove that one name
from the list; nothing else in the file changes.** The three unused owner
wrappers — `clear_owners` (`profile_queue.rs:624`), `unregister_control`
(`:654`), `attach_boundary` (`:724`) — are dead in every configuration
because `register_owner` installs the `_admitted` bodies directly; I verified
zero references. **Delete them.** Final acceptance must show no new warning
in the default lib, the lib-test with both test features, the feature-library
builds, and the workspace. The three `test_builtins` warnings are the
pre-existing set.

**Caller-owned cleanup inside the source timing entries: mandatory repair,
not a flag.** `discard_module_source_timing` (`source.rs:179`),
`record_current_source_timing` (`:223`) and
`record_call_stability_source_head_timing` (`:526`) now refuse before
popping the caller's own TLS timing frame. The frame is caller-owned: pop it
and credit the parent's `child_nanos` before any refusal; the refusal may
skip only the shared/global writes. A frame left on the stack across a
refusal is the leak the Q2 rule forbids.

**Resume order and the remaining gates.** Resume at the retained-alias and
epoch `Option<Admitted>` propagation — the cache alias gate is red at 8 and
at 1 threads, so it is a real gap, not a flake — then segments/context,
summary/reset and budget; then the six still-unrun gate mutants (each must
be red at its named observer), the standing replays after the last source
edit, the full lib 3x at 8 threads and once at 1, the workspace, the release
erasure check and the test-name delta. Every surface compiles, formats and
passes its lock audit before the next begins. Everything else in the brief
stands. Resume with `CDX_FORK`.

## Director ruling — 2026-11-08 (stage-4 static stop): Q7 settled; the owned-drop shape is authorized

C4 is accepted as a static round: it made no source edit (the tree is the C3
checkpoint, 33 files), its question is real, and stopping before the
out-of-scope constructor is right. Q4-Q6 stand.

**Q7 — the owned-drop shape: authorised as proposed, with four conditions.**
The route is genuine: `clear_stash_admitted` and `drop_belonging_admitted`
hold the collector's read permit while a selected `Arc<ProfileBoundary>` may
be the last reference; the Drop then reaches `abandon` ->
`segments::finish_active` -> `Epoch::close`, and the required entry
admission in that chain would nest (a deadlock against a queued writer and a
depth assertion in debug). The explicit-release shape is the correct one:
type-enforced, no TLS fallback, no deferred refund, production
representation unchanged.

1. **The shape.** `type BoundaryStorage = Option<Storage>` under the exact
   test cfg and `= Storage` otherwise; `release_admitted(&Admitted,
   Arc<Self>)` uses `Arc::into_inner`, calls the admitted abandonment with
   the *existing* permit, takes the storage, drops the cleaned boundary,
   then drops the storage in the existing field order; `Drop` early-returns
   (test cfg only) when the storage is `None`. The helper must not
   fabricate a permit, use TLS depth, defer the refund, or `mem::forget`.
   The ordinary `Drop` keeps today's `abandon(Retire::LastDrop)` body; its
   own entry admission may be `None`, and per Q1/Q2 the boundary's own row
   settlement and refund complete while only process-global alias/cache
   writes are skipped. Any Drop path that can reach the *current* epoch's
   rows from an unregistered thread is a STOP; name the proof.
   - `size_of::<BoundaryStorage>()` must equal `size_of::<Storage>()` in
     both cfgs (the `Arc` niche should make the `Option` free). Assert it
     and quote both numbers. If it differs, STOP and re-shape; do not
     re-derive a budget ledger.
2. **Scope extension by class: the constructor initialisers.** Every
   `ProfileBoundary { ... }` literal adapts `_storage: storage` to
   `_storage: storage.into()` — I enumerated the set: `logical/mod.rs:683`,
   `logical/completion.rs:40`, `suspension/core/profile_queue/tests.rs:222`,
   `profiling/tests/record_engine.rs:153` and `:198`. Nothing else changes
   in any of those files; `completion.rs` gets that one line only. The
   `From<Storage> for Option<Storage>` and the reflexive `Into` make the
   same text compile under both cfgs. Every file is disclosed.
3. **Every collector release site wires the helper** for a selected
   boundary it may hold the last reference to (`clear_stash_admitted`,
   `drop_belonging_admitted`, and `drain_matching_admitted`'s `keep`); the
   `Arc::into_inner` `None` case drops one reference exactly as today.
   `drain_attached`'s success path (boundary consumed into a cell) keeps its
   current shape.
4. **The Q6 instance erratum (mine).** The added closure instance is
   `selected.retain` at `collector.rs:176`; the row the keyed comparison
   labels ADDED at `collector.rs:328` is the pre-existing
   `hook.is_some_and` closure whose partner moved. The class ruling is
   unchanged: one new closure, a pre-existing reason class, on an added
   line. The census multiplicity note must say this in those terms.

After Q7: the retained-entry `Option<Admitted>` propagation (epoch
`terminal/settle`, the `Epoch.data` -> `RecordPath.state` order, alias
writes only under `Some`), the three timing-frame repairs, the warning
deletions, then segments/context, summary/reset, budget, the six mutants
(including the Q4 B/C arms), the replays after the last source edit, and the
acceptance runs. Resume with `CDX_FORK`.

## Director ruling — 2026-11-09 (stage-5 stop): Q8 handoff, Q9 ownership, the last two edges

C5 is accepted as a checkpoint: the retained `Option<Admitted>` propagation,
the three timing-frame repairs, the four named warning removals, the owned-drop
wiring and the Q8 witness are real progress, and the stop is correct on every
count. Q7 stands.

**Q8 — the generation handoff: a full reset inside `lease()`, after the
fence, before it returns.** The diagnosis is right: `lease()` publishes a
generation but leaves `CURRENT` holding the previous epoch, so the next
holder and a boundary created under the previous lease share one epoch
identity. The repair is not a stamp on the slots; it is the generation
boundary itself. Exact order:

1. `TEST_LEASE.lock()`; `NEXT_GEN`; `CURRENT_GEN.store(generation, Release)`
   (unchanged).
2. `fence_admitted_writes()` (unchanged). This is the cut: every write
   admitted before the generation has landed, and from the store onward an
   unregistered thread is refused at every entry.
3. `REGISTERED_GEN.set(generation)` (moved before the handoff).
4. Construct the `TestLease`, then call the existing admitted reset
   (`epoch::reset_for_lease(&lease)`) — the full `reset_unleased` body:
   `failed = None`, clear registrations, take and retire the previous epoch,
   clear the stash, all under the one `admit()` taken after registration.
5. Return the lease.

The full reset is the handoff, not only the epoch take: owners left behind
let the new generation attach to old owners, and a stash left behind lets the
new generation's `take_stash`/`drain_attached` consume old boundaries — the
theft class. The handoff must not wait on process bytes (the fixture's
`drain_for_lease` stays after acquisition), must not take the fence while
holding a permit, must not fabricate `Admitted`, and must not register before
the generation store. `TestLease::Drop` is unchanged: it does not detach, so
post-drop inspection keeps today's behavior; the next acquisition is the
handoff. The fixture's explicit `reset_for_lease` becomes idempotent; a
second mid-test reset keeps today's meaning. Production erases all of it
(`lease()` and `reset_for_lease` are already cfg'd).

The falsifier is the C5 witness
`brief713_prior_boundary_cannot_settle_current_lease_rows`: red pre-handoff at
8 and 1 (tuple `same_current_epoch=true unregistered=true
current_rows_changed=true old_calls=1 old_open=0 refunded=296 faults=0`), and
green post-handoff with `same_current_epoch=false`,
`current_rows_changed=false`, `refunded=296` and the same fault count.
Refusing the refund or deferring it is a STOP.

**Q9 — ownership: accepted with eight MOVE dispositions and three explained
removals.** I verified the origin review: each flag is an existing item whose
identical cfg descriptor is duplicated by the conversion's new cfg'd
parameters, and first-match pairing consumed the old row elsewhere. The
eight: `epoch.rs:583 existing` (baseline 404), `:600 publication_state`
(421), `:616 current` (437), `:701 reset_for_lease` (479 via C4 514; the Q5
disposition carries forward), `mod.rs:533 LogicalProfileTestLease` (398),
`:542 logical_profile_test_acquire` (407), `:727 start` (592),
`segments.rs:334 end`'s `|cell|` closure (293). **Disposition: MOVE, not
NEW.** The three REMOVED rows are the deleted owner wrappers' cfg'd
parameters (`profile_queue.rs:725/734/745`), explained by this round's
authorised warning cleanup. Copy `evidence/ownership-replayed-c5` unchanged;
no hand edits. The final replay after the last source edit supersedes it.

**The two remaining lock edges: close them under Q1, no new permission.**
- `drop_session` (`epoch.rs:727-731`): take the entry admission before
  locking `CURRENT` and pass the permit into `drop_belonging_admitted` and
  `abandon_owner_admitted`. Under `None`, the session's own queue/arena
  drops still release their storage (caller-owned); the registry/stash
  abandonment is deferred to the next reset/handoff, which releases it
  before `drain_for_lease` waits (Q2). No charge may be lost.
- attach: pass the attach body's existing permit into the storage-failure
  retire path (`retire_admitted(Some(_access), ...)`); no second `admit()`
  beneath the permit.

**Warnings: the default lib is the bar, and it is 6 -> 0.** The six
default-lib dead items are all strict-gate entries whose only callers are
the new test files: `evaluation_cache/profile/mod.rs:195/231/267/909`
(`evaluation_cache_profile_configured`, `_active`,
`_incremental_enabled`, `module_source_alias_completion`/`_duration`) and
`profiling/config.rs:86/165` (`lexical_profile_active`,
`lexical_profile_incremental_enabled`). Gate them `#[cfg(test)]` or delete
them; nothing else changes in those files. After that the default lib must
be zero-warning and the lib-test must carry only the three pre-existing
`test_builtins` warnings. The feature-library build is a diagnostic, not a
release bar: its remaining dead-code warnings must be enumerated against
the pre-713 feature build, and the accepted class is feature-gated test
support unused without `test` (the `test_builtins` precedent). `publish()`
/`Publication` are `pub(in crate::runtime)` and used only by fixtures; test
-gate them if the feature-only build still warns.

**Order after Q8-Q9.** Handoff -> the two edges -> the six `#[cfg(test)]`
gates -> the Q8 witness green plus all focused gates (14/14) -> the remaining
epoch callees, segments/context, summary/reset, budget -> the six unrun
mutants including the Q4 B/C arms -> the standing replays after the last
source edit -> the acceptance runs (full lib 3x at 8 threads, once at 1, the
workspace, the release erasure, the test-name delta). Every surface compiles,
formats and passes its lock audit before the next begins. Resume with
`CDX_FORK`.

## Director ruling — 2026-11-09 (stage-6 stop): Q10 the witness observes both refund windows; Q11 the SourceStack permit forwarding

**Where the round stands.** The Q8 handoff is exact (generation store,
unchanged fence, registration, lease construction, `reset_for_lease`,
return); the two named permit edges are closed; the dead items are gated and
the default lib is at zero warnings with lib-test at the three baseline
entries; the 14 focused gates pass at 8 and 1; Q9's canonical ownership copy
is in and verified. The round stopped at two real observations, both mine to
rule.

### Q10 — the witness must observe both refund windows, with type-derived expectations

The literal `refunded=296` I stipulated in Q8 was measured in the drop window
before the handoff existed. With the full reset inside `lease()`, the old
epoch's record path is retired in the handoff, so its charge
(`size_of::<RecordPath>() + 2 * size_of::<usize>()`, the remainder 248) is
refunded before the witness's baseline, and the drop window carries only the
boundary's own charge (`size_of::<ProfileBoundary>() + 16`, measured 48).
248 + 48 = 296: no charge is lost; the window moved with the cut itself. The
worker is right that this is a window change, not a regression, and right to
stop rather than rewrite the observer on its own.

The correction is **not** "296 -> 48". As the witness stands,
`same_current_epoch=false` and `current_rows_changed=false` also pass on a
handoff that swaps `CURRENT` without retiring the old epoch's records: the
stale boundary then settles the old epoch's record on the foreign thread, the
new epoch's rows stay untouched, and the old row still reads `(1, 1, 0)`
after settlement. Only the handoff-window refund forces the retirement Q8
names. A check that cannot fail on the transformation it certifies is not a
check.

Rule:

1. In `logical/epoch.rs`, beside `RecordPath`, add one shared constant
   `pub(in crate::runtime) const RECORD_PATH_STORAGE_BYTES: usize =
   std::mem::size_of::<RecordPath>() + 2 * std::mem::size_of::<usize>();`
   and use it in `Epoch::open`'s claim. No behaviour change, no second
   literal for the same quantity.
2. In `brief713_prior_boundary_cannot_settle_current_lease_rows`, sample
   `budget::process_bytes()` immediately before and immediately after
   `let current = lease();`. Assert the handoff window equals
   `RECORD_PATH_STORAGE_BYTES` exactly. This converts the 248 from an
   inference into a measurement.
3. Assert the drop window equals `std::mem::size_of::<ProfileBoundary>() + 16`
   exactly — the expression the test already builds the claim with — replacing
   `refunded > 0`.
4. Assert the two windows sum to the total the witness charged (one record
   path plus one boundary storage), and print both numbers
   (`handoff_refunded=... refunded=...`).
5. Every other assertion and printed field stays exactly as it is today:
   identity false, current rows unchanged, `unregistered=true`, counts
   `(1, 1, 0)`, faults 0.
6. Falsifier, mandatory: with `reset_for_lease(&lease)` removed from
   `lease()` (the pre-Q8 shape), the strengthened witness must go red with
   `same_current_epoch=true` and a zero handoff window; restore and record
   the green line. This mutant joins the six.
7. Do not retain dead charges, defer refunds, or move one baseline across
   the construction of the replacement epoch to make any window print 296.
   If a window measures anything beyond the named claim, STOP with the
   measured composition instead of absorbing it.

Both numbers are now type-derived, so a `RecordPath` or `ProfileBoundary`
size change moves the claim and the expectation together.

### Q11 — the narrow SourceStack permit forwarding is authorized, with a class census

`SourceStack::push` reaches `Epoch::storage()`, an admitting wrapper, beneath
the cache source entry's permit. The depth assertion did its job: this is a
real nested admission, reproduced serially, and a defect against the accepted
C5 green regardless of why C5's cache child did not reach it. The repair is
the class's repair: pass the permit down.

Scope extension: `crates/rum-runtime/src/runtime/profiling/logical/source_stack.rs`
and the single converted caller
`crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs:80`
(`evaluation_profile_definition_source_enabled`).

Shape:

- `push_admitted` with the brief's cfg'd `_access: &Admitted` parameter; body
  = today's `push` body, with `Epoch::storage_admitted(access, bytes)` in
  place of the admitting `Epoch::storage(bytes)`; `epoch::current()`
  unchanged.
- `push` stays as the admitting wrapper for the unconverted callers
  (`profiling/state.rs`, `logical/context.rs`): `admit()` at entry; on `None`
  the same `missing` bookkeeping as today's `None` branch (`missing = 1` from
  zero, saturating increment otherwise, tail untouched); otherwise delegate
  with the permit. The refusal path must stay observationally identical.
- One converted caller changes: `source.rs:80` calls
  `push_admitted(_access, symbol)`.
- No TLS depth fallback, no fabricated permit, no shortened transaction, no
  duplicated body, no `#[allow]`, no second admission.

**Class census (a disclosure duty, not an instance list).** Enumerate every
body that holds an `Admitted` and transitively reaches an admitting wrapper —
roots are the `admit()`/`publish()` sites; edges are every wrapper that
admits (`Epoch::current/storage/retire/fault/open/close`, boundary
`complete/abandon`, `drop_session`, collector entries, the cache
entry/getter family, `segments`, `summary`, budget). One table: holder ->
call edge -> admitting wrapper -> disposition (`_admitted` variant already
used / repaired in this round under this same private-forwarding class /
pending with the reason it is unreachable under the lease). The class
extension covers another site only when the callee's `_admitted` variant
already exists; anything needing a new admitting-free callee is a STOP. Do
not claim closure beyond the table. State, with evidence, why C5's LP01 cache
child did not reach this edge (slot failure semantics versus an unreached
path) — a one-line disclosure, not a repair.

**Order after Q10-Q11.** Both repairs -> focused 14/14 plus the LP01 cache
child at 1 and 8 -> the strengthened witness's mutant red -> restore -> the
remaining epoch callees, segments/context, summary/reset, budget -> the six
mutants including the Q4 B/C arms -> the standing replays after the last
source edit -> the acceptance runs (full lib 3x at 8 threads, once at 1, the
workspace, the release erasure, the test-name delta). Every surface compiles,
formats and passes its lock audit before the next begins. Resume with
`CDX_FORK`.

## Director ruling — 2026-11-09 (stage-7 stop): Q12 the event bridge, and bridges inside the named conversions

The round is where I asked it to be: Q10's strengthened witness measures both
windows (`handoff_refunded=248 refunded=48`, sum asserted against the
type-derived charges), the pre-Q8 mutant goes red exactly as predicted
(`same_current_epoch=true handoff_refunded=0 refunded=296`) and restores
green; Q11's `push`/`push_admitted` split is the ruled shape (wrapper admits,
`None` reproduces the `missing` bookkeeping, one body, `storage_admitted`
under the permit); the LP01 cache child passes at 8 and 1; the 14 focused
gates are green; the census lists all 105 sites in 99 root bodies and 174
root/permit bodies. The stop is Q12: 13 holders reach
`add_admitted -> Epoch::event`, whose two failure branches call the admitting
`fault()`/`retire()` wrappers, and `event_admitted` does not exist. My Q11
clause covers this literally; the worker stopped instead of bridging on its
own authority. Correct.

### Q12 — the event bridge is authorized

`Epoch::event` is a conversion the brief's Target names. Rule:

- `event` stays the strict standalone wrapper: exact test cfg,
  `let Some(access) = lease::admit() else { return; };`, then delegate with
  the cfg-erased `&access`. The wrapper's `admit()` is the only
  registration check; the admitted body carries no second check (the permit
  is the enforcement).
- The body moves **once** into
  `event_admitted(&self, #[cfg(exact)] _access: &Admitted, key, delta, start)`.
  The `event_birth` early-out, the equal-birth `fault` branch (drop the data
  mutex first), the side-index insertion and its cap `retire` branch, keys,
  deltas, budget behaviour and every return point are unchanged.
- The only callee changes: the equal-birth branch calls
  `fault_admitted(Some(_access))` and the cap branch calls
  `retire_admitted(Some(_access), Retire::DiagnosticCap)` — both variants
  exist. No new fault/retire helper, no new behaviour.
- `synchronous::add_admitted` calls `event_admitted(_access, ...)`. The
  standalone `add` wrapper is unchanged.
- Tests keep calling `Epoch::event` (the wrapper): they hold leases at depth
  0, so `admit()` returns `Some`; retired-epoch callers are rejected by the
  body's `is_valid()` exactly as today.

### Q12 clarification — bridges inside the named conversions are the conversion

Q11's "anything needing a new admitting-free callee is a STOP" was aimed at
the census finding *unbudgeted* expansion. The brief's Target already names
the remaining conversion surfaces: the Epoch acquisition family, segments
and context, summary/reset and budget. Inside those named surfaces, a
converted body may create the private `_admitted` bridge it needs on an
existing wrapper **without a new ruling**, under the same shape rules this
brief has used throughout:

- one body: the wrapper admits and delegates; no duplicated body, no
  `#[allow]`, no TLS depth fallback, no fabricated permit, no shortened
  transaction;
- a strict cfg'd `&Admitted` parameter (or `Option<&Admitted>` only where a
  caller-owned path must refuse), erased in production;
- failure callees switched only to **existing** `_admitted` variants —
  `fault_admitted`, `retire_admitted`, `storage_admitted`, `close_admitted`,
  `abandon_owner_admitted`, `drop_belonging_admitted`, and the like;
- no behaviour, control-flow, ordering, key, delta or budget change.

A bridge that would need new semantics, a new control-flow path, or a
production-shape change is still a STOP. Every new bridge is disclosed by
updating the census rows, and closure is never claimed beyond the table.

**Order after Q12.** The event bridge -> 14/14 plus LP01 at 8 and 1 -> the
remaining named conversions in the brief's order (epoch callees,
segments/context, summary/reset, budget), each compiled, formatted and
lock-audited before the next -> the six mutants including the Q4 B/C arms
and the Q10 handoff mutant -> the standing replays after the last source
edit -> the acceptance runs (full lib 3x at 8 threads, once at 1, the
workspace, the release erasure, the test-name delta). Resume with
`CDX_FORK`.

## Director ruling — 2026-11-09 (stage-8 stop): Q13 the drain must not hold the lease while it waits

The event bridge is in and is the ruled shape (strict wrapper, one
`event_admitted` body, `fault_admitted`/`retire_admitted` on the two failure
branches, `add_admitted` passing its permit, tests still on the wrapper);
LP01 passes at 1 and 8; the census now lists 106 sites in 100 root bodies and
176 root/permit bodies with the 13 former event chains repaired. The stop is
correct: one 8-thread focused run stalled, and a passing retry does not
cancel it. And the stall is now **measured**, not hypothetical:
`logs/c8-event-focused-8.log` carries

    thread '...brief713_cache_write_from_unregistered_thread_is_refused' panicked
    at .../logical/epoch.rs:874:9: diagnostic drain stalled at 104 bytes

at the 30-second deadline, then the outer 60-second bound killed the rest.

### The defect: a lock inversion in the test fixture, older than this round

`epoch::drain_for_lease` waits for `PROCESS_BYTES == 0` **while the caller
holds `TEST_LEASE`**. A cross-lease probe legitimately retains a caller-owned
charge across its lease-free window (`drop(previous)` -> charge still live ->
next `lease()`); it cannot release that charge until it reacquires
`TEST_LEASE`. Any test that runs `enabled()` (or either other drain site) in
that window takes `TEST_LEASE` and waits for the very charge the probe can
only release by taking `TEST_LEASE`. That is the deadlock; 104 bytes is the
measured retained total. The race predates C8 — the drain dates from LP06
(brief 668/669) and the cross-lease probes from this brief's C5, and the
event bridge neither charges nor drains — and it is not a stale inventory
artifact. It must be fixed here because the 37-run acceptance campaign would
hit it.

Holding the lease across the probe's gap is not the repair: the retained
charge across `drop(previous)` is the property the probes exist to test.

### Rule: wait without the lease, acquire, re-check, retry

One test-only helper implements the protocol — the drain and the acquisition
are one operation, and the lease is never held while waiting:

```
pub(in crate::runtime) fn lease_drained() -> TestLease {
    let deadline = Instant::now() + Duration::from_secs(30);   // existing bound and message
    loop {
        while budget::process_bytes() != 0 {                    // wait WITHOUT the lease
            release_deferred_charge();
            assert!(Instant::now() < deadline, "diagnostic drain stalled at {} bytes", ...);
            std::thread::yield_now();
        }
        let lease = lease::lease();                             // Q8 handoff reset
        epoch::reset_for_lease(&lease);                         // keep the explicit reset
        if budget::process_bytes() == 0 { return lease; }
        drop(lease);                                            // let the cross-gap probe run
        std::thread::yield_now();
    }
}
```

- The helper returns only in the state that matters: lease held, handoff and
  explicit reset done under it, `PROCESS_BYTES == 0`. Nothing about the
  predicate, the deadline, the message or the reset is weakened.
- Route all three drain sites through it: `engine_support::enabled()`
  (replacing `lease()` + `reset_for_lease` + `drain_for_lease`),
  `tests/module.rs:407-409`, and `retained_pause.rs` (drop `enabled` first,
  as `module.rs` already does, then `lease_drained()` and the same
  `== 0` assertion). `drain_for_lease` becomes private to the helper; if a
  site genuinely needs to hold a lease it already owns, it asserts zero
  without waiting.
- No probe changes: the retained charge across the gap stays exactly where
  it is.
- No deadline change, no skip, no thread-default change, no weakened
  assertion, no early release in any probe.

**Falsifier, mandatory.** A deterministic reproduction that pins the
inversion: one thread retains a caller-owned charge across a lease-free
window; a second thread calls `enabled()` / `lease_drained()` while the
charge is live; the first thread then reacquires the lease, releases the
charge and finishes. Run it on the **pre-fix** fixture code first and quote
the 30-second `diagnostic drain stalled at N bytes` panic line (that is the
red); then on the fixed helper, where it must pass promptly (green). Keep it
as a permanent test — the green path is fast. Back it with ~20 consecutive
8-thread runs of the focused 14-test set, all 14/14, and report the wall
times. The falsifier is the proof; the repeats are the supporting
distribution.

**Order after Q13.** The helper and the three sites -> the falsifier red and
green -> focused 14/14 at 8 and 1 -> the remaining named conversions
(epoch callees, segments/context, summary/reset, budget) -> the six mutants
including the Q4 B/C arms and the Q10 handoff mutant -> the standing replays
after the last source edit -> the acceptance runs (full lib 3x at 8 threads,
once at 1, the workspace, the release erasure, the test-name delta).
Resume with `CDX_FORK`.

## Director ruling — 2026-11-09 (stage-9 stop): Q14 the ExecutionScope ownership bit; Q15 the for_test MOVE

The Q13 repair is accepted in shape: `lease_drained()` waits without the
lease, acquires, resets, re-checks and retries; the permanent falsifier went
red on the pre-fix fixture with `diagnostic drain stalled at 32800 bytes` at
the 30-second deadline and green in 0 ms on the fix; 20 consecutive 8-thread
focused runs passed 14/14. C9 then converted the Epoch acquisition/reader
family and its direct permit callers (13 Rust files from C8) and stopped at
two items before the remaining segment/context surface. Both are mine to
rule.

### Q14 — the scope records what it pushed and releases exactly that

The measured defect is real: `ExecutionScope` is a zero-sized marker, so a
scope whose `enter()` pushed a TLS control under its own lease has no record
of that fact; its `Drop` re-decides from the *current* flag through the
admitting getter, which refuses on a foreign lease, and the pushed entry is
leaked (`baseline=0 pushed=1 remaining=1 stale_control=true`, at 8 and 1).
The inverse mistake is equally wrong: a scope whose `enter()` was refused
must not pop somebody else's entry.

Rule:

1. Under the exact test cfg, `ExecutionScope` gains one ownership field,
   `pushed: bool`. Production keeps the zero-sized struct; enforce it with
   `#[cfg(not(exact))] const _: () = assert!(std::mem::size_of::<ExecutionScope>() == 0);`
   so a production frame growth is a compile error, not a review finding.
2. Under the exact cfg, `enter()` takes **one** `admit()`:
   - `None` (foreign lease): return `pushed: false`; no flag read, no push.
   - `Some(access)`: `pushed = lexical_profile_configured_admitted(access)`,
     and when true the control is pushed exactly as today, inside the same
     function that returns the bit. `lexical_profile_configured_admitted`
     cold-loads through `publish_cold_load`'s CAS when the flag is 0, so the
     production decision (`flag != 2` then `configured()`) is reproduced
     exactly.
   - The raw `LEXICAL_PROFILE_ENABLED.load` is not executed under the exact
     cfg; the admitted getter is the only read.
3. Under the exact cfg, `Drop` reads **only** `self.pushed`:
   - true -> pop one entry unconditionally (`EXECUTION_CONTROLS` +
     `try_with`); no flag read, no `admit()`, no refusal. Caller-owned TLS
     cleanup always completes (Q2).
   - false -> do nothing; a refused scope never pops.
4. Production `enter()` and `Drop` are unchanged, byte for byte (raw
   hot-path load, cold `push_control`/`pop_control`).
5. `enter()` must be reached at depth 0. Its `admit()` carries the depth
   assertion; a nested call is a STOP, not a patch point.
6. The investigative probe becomes a permanent test with both clauses:
   - (a) prior-lease push, foreign-lease drop -> the entry is gone
     (`remaining=0`, `stale_control=false`);
   - (b) an older entry is pushed under a lease, a new scope is entered and
     dropped under a foreign lease -> the older entry **remains**, and is
     then cleaned up under a registered lease.
   Two mutants, each watched red and restored: `Drop` ignoring `pushed`
   (clause (b) must fail) and `Drop` re-deciding from the getter (keyed to
   the pre-fix shape; clause (a) must fail). No assertion, skip, thread
   default or frame shape may be weakened.

### Q15 — the `for_test` row is a MOVE; the candidate may be copied

I verified the origin in the committed baseline: `HEAD:epoch.rs:88` carries
the identical `#[cfg(any(test, feature = "test-builtin-memory-growth",
feature = "test-continuation-slots"))]` attribute on the existing
`Epoch::for_test`. C9 did not edit that attribute; the function only moved
(88 -> 91 -> 99) because lines were added above it, and the replay's
first-match pairing consumed the descriptor at `epoch.rs:367`. That is the
same duplicate-descriptor artifact as the four Q9 rows, which carry forward
unchanged: the disposition is MOVE.

Copy `evidence/ownership-replayed-c9` unchanged (the 8,560-construct
candidate; the five non-closure flags are four Q9 carries plus this MOVE, and
the sixth, the cross-file `|boundary|` closure move, is brief-added, not a
baseline disappearance). No hand edit, no scanner weakening, no baseline
descriptor loss.

**Census counts.** The pinned rejection census has been deferred through
every stop as an intermediate-maintenance artifact. At the final source
checkpoint, before the acceptance runs, update the pinned counts once from a
fresh canonical replay and include the per-class table; a hand-edited count
is a STOP.

**Order after Q14-Q15.** The ownership bit + the permanent two-clause test ->
the two mutants -> 14/14 at 8 and 1 -> the remaining segment/context
conversions -> the remaining named conversions (summary/reset, budget) -> the
six mutants including the Q4 B/C arms and the Q10 handoff mutant -> the
standing replays after the last source edit (with the once-only pinned
census) -> the acceptance runs (full lib 3x at 8 threads, once at 1, the
workspace, the release erasure, the test-name delta). Resume with
`CDX_FORK`.

## Director ruling — 2026-11-09 (stage-10 stop): Q16 the permit's transaction ends at the last shared access; Q17 the unchanged-caller class

Q14 and Q15 are accepted in shape: the scope records and releases its own
push, both mutants fired and were restored, the two-clause test is permanent,
and the C9 ownership candidate was copied unchanged as ruled. C10 then
converted segment acquisition/pause and the private Stack forwarding, and
stopped at a real edge with a real measurement: `before=1 after=2`, a serial
backtrace through `Segment::Drop -> end -> admit` at depth 1, while the
original Context body is green on the identical schedule. The stop is
correct; both questions are mine.

### Q16 — release the permit after `restore_admitted`, not at function end

The measured path: `retire()` stores `valid=false` before it closes
RecordPaths, so a registered entry can observe an invalid epoch while the
retained path is still open; `restore_admitted` then succeeds, `enter_active`
discards the local `Option<Segment>` on its invalid return (or a `?` unwind),
and the implicit destructor's `Segment::end` calls `admit()` under the entry
permit. That is the nested admission.

The repair is not a destructor owner. **A permit's transaction ends at its
last shared access.** In `enter_active` that access is the `restore_admitted`
call; everything after it — the validity check, `control.as_ref()?`, the
source capture, the `EXECUTION_CONTROLS` push, the guard construction — is
caller-owned TLS and atomics, and in production runs with no permit at all.

Rule:

1. Under the exact cfg, the wrappers hand the owned `Admitted` to
   `enter_active` **by value**; the call to `restore_admitted` borrows it
   (`&access`), and the very next statement is `drop(access)`. Every later
   statement and every later destructor in the function runs at depth 0,
   exactly as production does; the local segment's implicit drop admits on
   its own account, as it does in production.
2. Production keeps `enter_active`'s signature and body byte-unchanged (no
   permit parameter exists there).
3. `fault_admitted(Some(...))` in the double-entry path stays where it is,
   before `enter_active`, under the same owned permit.
4. This is **not** a shortened transaction: the permit exists to exclude a
   lease from the shared read/write (the configured-flag read and the epoch's
   segment restore). After the restore returns, no shared state is touched;
   a segment end is a separate admitted transaction in production too, and
   merging it into the entry permit was a drop-order accident, not a design.
5. The owner shape in `evidence/c10-q16-context-drop.md` is **not**
   authorized as the primary: it relies on `Segment::end`'s slot-identity
   precheck to suppress a second admission after `end_admitted`, a non-local
   invariant. Only if the early release is found to lose a shared access is
   the owner the fallback — and that finding is a STOP with the access named.

**Class duty.** The census must now, for every permit-holding body, list the
locals whose destructors may admit (`Segment`, `Call`, boundary storage,
collector or owner guards, anything whose Drop reaches `admit`/`publish`),
and give each one a disposition: ended explicitly with the permit, or
outside the permit by construction (released before the permit drops). The
question is not just this `Option<Segment>`; it is every value that can be
dropped inside an admitted scope.

**Falsifier, mandatory.** The Q16 diagnostic (a real `retire()` on a second
registered thread; `Epoch.data` held only to pin the real prefix) becomes a
permanent test: red with the permit held to function end (nested admission,
`before=1 after=2`), green with the early release, at 8 and 1 threads. Its
scheduling must not fake validity, path state, storage or the permit.

### Q17 — the unchanged-caller predicate extension

The two new rejection keys (`resolution.rs:334`, `suspension/core/abandon.rs:30`)
are `ExecutionScope::enter` call sites in files whose bytes equal the
baseline; the reason is the Q14-required cfg split inside the converted
callee. That is a class, not an exception: **when a converted callee gains
cfg-separated alternatives, its unchanged callers' new conditional-function
rejection keys are admitted**, provided (a) the caller's source is
byte-identical to the baseline, (b) the reason names a cfg split introduced
by a conversion in this brief, and (c) the row names both the callee and the
conversion that caused it. No other new key in an unchanged file is covered;
no scanner change, no hand edit. The once-only final count still happens at
the final source checkpoint.

**Order after Q16-Q17.** The early release + the permanent Q16 test -> the
class census of admitting destructors -> 14/14 at 8 and 1 -> the remaining
Context surface -> the remaining named conversions (summary/reset, budget)
-> the six mutants including the Q4 B/C arms and the Q10 handoff mutant ->
the standing replays with the once-only pinned census -> the acceptance runs
(full lib 3x at 8 threads, once at 1, the workspace, the release erasure,
the test-name delta). Resume with `CDX_FORK`.

## Director ruling — 2026-11-09 (stage-11 stop): Q18 an admitting destructor may not run under a permit

Q16 is accepted exactly as ruled: `enter_active` takes the owned permit,
borrows it for `restore_admitted`, drops it as the very next statement, and
the production signature and body are byte-unchanged (source audit PASS). The
permanent real-retirement test is green at 8 and 1 (`invalid=true
path_open=true before=1 after=1 no_nested_admission=true`), the hold-to-end
single-site mutant is red at both (`before=1 after=2`, `nested profiling
admission`), and the fixed bytes are restored (`73dae444…`). The 237-body
destructor census is accepted as coverage, not closure: it found a real class
hole, and the STOP is correct.

### The measured instance

`drain_attached` now owns an `Admitted` and passes it into
`attach_boundary_admitted`. The hook's policy assertion can fire on a
legitimate schedule: an owner is found, then the record is retired before the
policy read, so the assertion sees `None`. On unwind, `paused`'s owned
boundary drops; `ProfileBoundary::Drop` calls the admitting `abandon`,
`admit()` refuses `ADMITTED_DEPTH == 1`, and the panic in a destructor during
unwinding aborts at 8 and 1 when the boundary is the sole reference. With one
reference retained outside the drain the identical assertion is caught and
the test passes. The permit-holding drain is new in this brief — the baseline
`drain_attached` held no permit, and its hook admitted and released inside the
callee — so the abort is introduced here, not inherited.

### The class rule

A permit is an exclusion transaction. Every value it owns must leave it
through an explicit handoff or an unwind owner; **no admitting destructor may
run at depth >= 1, on the normal path or on unwind**. For every body that
holds `Admitted`/`&Admitted`, a value whose Drop can reach `admit()` (an
`Arc<ProfileBoundary>`, a `Segment`, a `Call`, a collector Cell or arena, and
any future type with an admitting Drop) must have one of:

(a) a transfer that completes before the permit's last shared access (Q16's
    shape, the segments tail results);
(b) an explicit under-permit release that defuses the destructor (Q7's
    `release_admitted`, once the storage is taken first);
(c) an exact-cfg unwind owner that forwards the same permit.

A row settled only by argument is an open obligation: the argument must name
every panic source it excludes.

The class covers panics the protocol can raise: assertions, debug
assertions, callbacks and test hooks. A panic that can only come from an
already-poisoned payload mutex is a cascade; a guard would re-enter the same
poison and abort too. Such rows are dispositioned POISON_ONLY, named
individually, and are not the owner's responsibility.

### The collector owner (the measured instance)

Authorize a private, exact-cfg owner in `logical/collector.rs` (the name is
the worker's choice) holding the permit borrow (`&Admitted`) and every
boundary Arc the body owns. Its Drop calls
`ProfileBoundary::release_admitted(access, boundary)` for each element still
held — the under-permit spelling of exactly what the unconditional Drop does
in production (the last reference settles; aliases are untouched). The normal
path hands every element out explicitly; on unwind the owner performs the
disposal instead.

It must cover all ten unresolved rows' owned values:
`stash_boundary_admitted`'s argument, `take_selected_admitted`'s `selected`
from the first clone, `restore_stash_admitted`'s `keep`, all of
`drain_matching_admitted` (the whole taken vector, its iteration state, and
the accumulated `keep`), and the `clear_stash`/`drop_belonging` loops.
`take_selected`/`take_belonging`'s tail results then become TRANSFERRED.

Two shape conditions are part of the ruling:

1. The attach call must borrow the current element from the owner
   (`attach(access, &boundary)`) and move it out only after the call returns.
   A bare loop local at the call site re-creates the measured gap.
2. The owner is declared before any payload-lock guard so that it drops after
   that guard: its releases must never run with a payload lock held. It must
   not allocate or take a payload lock before its releases, and must not
   itself call `admit()`.

Production erases the owner; every production body stays byte-unchanged.
Order is preserved where it exists: the drain keeps stash order, and the
owner's release order is deterministic and disclosed.

### `release_admitted` is made unwind-safe

`release_admitted` currently takes `_storage` after `abandon_admitted`; an
unwind inside settlement drops a boundary whose test-cfg Drop still admits at
depth 1. Reorder: `Arc::into_inner` -> take the storage -> `abandon_admitted`
-> drop the boundary (the test-cfg Drop early-returns on `None`) -> drop the
storage for the refund. `_storage` is read only by `release_admitted` and that
Drop (verified in `logical/mod.rs`), so no state is lost, and the sole-owner
path can no longer admit from its destructor.

### `finish_active_admitted` and the registry rows

`finish_active_admitted`'s token is resolved by construction, not by an
owner: `Segment::end`'s test-cfg precheck admits only while the TLS slot still
matches, and the ordered path removes the slot at `segments.rs:655` before
any packing or lock call; the only admitting call before the take is
`fault_admitted` on the `!ordered` branch, whose panic source is lock poison.
The row must state the take line and the POISON_ONLY exception.

The five owner-registry rows and `attach_boundary_admitted` stay outside the
owner only with the same kind of path-complete proof: a live `ProfileQueue`
holds its own arena Arc, so a registry removal is never the arena's last
reference while the queue exists; `ProfileQueue::Drop` unregisters before it
drains, and drains outside any permit; and a registry-held arena can still
hold cells only after an interrupted queue destructor, i.e. a cascade. The
attach Cell clones the caller's Arc, so the caller's (now guarded) owner is
the final reference. Each row's text must carry that proof and mark the
residual as POISON/CASCADE. If the proof cannot be established, that family
joins the class (the owner drains the cells under the permit) — not a silent
row.

### Falsifier, mandatory

The measured sole-owner/control pair is accepted as calibration. A permanent
deterministic test must now exercise the real drain: a real lease, a real
registered owner epoch, a real open record and one real charged boundary as
the sole Arc, held by `drain_matching_admitted`; a panic at the attach-hook
position; the original panic caught; no nested admission; the boundary
disposed under the permit (its charge refunded, no row settled by a second
transaction); at 8 and 1. The real policy assertion under the real retire
schedule is preferred (a test-cfg scheduling seam at the measured point, no
production build change); a test-installed panicking drain hook through the
existing hook table is acceptable only if it saves and restores the real
hooks through the module's private access so that no other test can observe
it. Before the owner the test must abort with the quoted `nested profiling
admission` / `panic in a destructor during cleanup` / `aborting` lines at 8
and 1; after the owner it must be green at both; and a one-site mutant (the
drain's owner removed) must abort and be restored. No deadline, assertion,
policy variant or pool setting may change.

### Bookkeeping and order

All ten Q18 rows, the two EXPLICIT_NORMAL rows and the five registry rows
flip to a resolved disposition naming the mechanism; no row may remain
Q18_UNWIND_UNRESOLVED or _PENDING; the body count stays 237. The pinned
census/rejection counts are still updated once at the final source checkpoint
(Q15/Q17) from a fresh canonical replay with the per-class table — no hand
edit, no scanner change.

**Order after Q18.** The owner + the `release_admitted` reorder + the resolved
census + the permanent falsifier + the mutant -> 15/15 at 8 and 1 -> the
remaining Context surface -> the remaining named conversions (summary/reset,
budget) -> the six mutants including the Q4 B/C arms and the Q10 handoff ->
the standing replays with the once-only pinned census -> the acceptance runs
(full lib 3x at 8 threads, once at 1, the workspace, the release erasure,
the test-name delta). Resume with `CDX_FORK`.

## Director ruling — 2026-11-09 (stage-12 stop): Q19 the summary is one admitted transaction; six report files get forwarding only

Q18 is accepted in shape: the exact-cfg owners cover every collector
argument, vector, current element, iterator and `keep`; `release_admitted`
takes the storage before settlement; the permanent real-retirement test is
green at 8 and 1 with `original_assertion_caught=true disposed=true
refunded=296 expected=296 rows_unchanged_since_retire=true`, the pre-owner
and one-site mutant arms abort at both (`nested profiling admission` /
`panic in a destructor during cleanup` / `aborting`, exit -6), and the fixed
source is restored (`166c9f33…`). The final census has 237 bodies and zero
unresolved or pending rows: 6 UNWIND_OWNER, 9 TRANSFERRED, 1
DEFUSED_BEFORE_SETTLEMENT, 1 SLOT_REMOVED_BEFORE_PANIC_POISON_ONLY, 5 arena
plus 1 borrowed-cell POISON_CASCADE, 183 NO_ADMITTING_LOCAL, 31
OUTSIDE_PERMIT. 15/15 at 8 and 1. The Q19 stop is correct: the summary body
is a named conversion surface whose callees live in six files outside the
MAY list.

### Q19 — one transaction, forwarded

The summary is one admitted transaction. A report is a single observation of
the profiling state: a reset or publication on a sibling registered thread
must not interleave between its sections, and the raw rewrite counters in
`print_rewrite_rule_prefilter_summary` have no getter at all, so outside the
permit they would be read unfenced. Per-helper admission, releasing around
the helpers, and duplicated helper bodies are rejected for exactly those
reasons. A TLS depth fallback stays forbidden.

Rule:

1. `print_lexical_profile_summary_body` keeps a strict entry wrapper (its two
   callers in `summary.rs` stay unchanged): admit, return on refusal, call
   one `_admitted` body that holds the permit across `logical::summary()`
   and every helper. Production erases the parameter and keeps the body
   single-source. Refusal is wholesale: an unregistered thread under a lease
   now prints nothing, where today the body's own counter lines print
   unfenced. That is the intended class behaviour and must be disclosed.
2. Scope extension, exactly these six files and only this forwarding:
   `profiling/report_bindings.rs`, `report_presence_reuse.rs`,
   `report_presence_shapes.rs`, `report_presence_sources.rs`,
   `report_rebuild.rs`, `report_timing.rs`. Each helper gains the exact-cfg
   `&Admitted` parameter on its existing item (or one `_admitted` twin whose
   only caller is the summary; no second wrapper may be left uncalled), and
   every strict getter call reachable in its body becomes the existing
   `_admitted` variant — all nine getters have one. There are 28 getter call
   sites today; that number is a sanity check, not the list. The duty is the
   class: every strict getter reachable from a converted helper, found by a
   scan, and the held-body test nests on any missed one. No output, order,
   formatting (beyond rustfmt of the edited items) or logic change; no other
   edit in these files.
3. `print_rewrite_rule_prefilter_summary` takes the parameter and reads its
   raw counters under the summary's permit. If any of its reads can occur
   outside it, STOP.
4. Lock order: the body holds the fence read side across `logical::summary()`'s
   `current_slot()` lock and the helpers' TLS/atomic reads. No holder of the
   slot (or another payload lock) may call a strict admitting wrapper or
   `publish()`; a reverse edge is a STOP. An existing `_admitted` variant is
   used, never a call to the strict wrapper.

### Falsifier

The held-body probe becomes the permanent test: the `_admitted` body called
under a real permit at 8 and 1, plus the strict entry at depth 0. Before the
conversion the held arm nests at depth 1 while the depth-0 control passes
(quote the receipt); after it both arms pass. The depth-0 printed bytes are
compared by the LP01 parents, which must stay identical; no output assertion
is invented for the held arm beyond return-and-no-nest.

**Order after Q19.** The entry wrapper + the six-file forwarding + the
permanent held-summary test -> reset -> budget -> the six mutants including
the Q4 B/C arms and the Q10 handoff -> the standing replays with the once-only
pinned census -> the acceptance runs (full lib 3x at 8 threads, once at 1,
the workspace, the release erasure, the test-name delta). Resume with
`CDX_FORK`.

## Director ruling — 2026-11-09 (stage-13 stop): Q20 the reset family is the exclusive transaction on the existing WRITE_GATE

Q19 is accepted in shape: one strict entry wrapper whose two callers are
byte-identical, one `_admitted` body holding the single permit through
`logical::summary()` and all 28 helpers, all 28 strict getter calls across
the nine getters forwarded to their existing `_admitted` variants, the raw
rewrite-counter helper taking the same permit, no dead strict wrapper, no
output/order/logic change, production erased. Both permanent tests pass at
8 and 1 with the C12 held-body nesting receipt as their red arm, and 15/15
focused passes at both. The Q20 stop is correct: a shared read permit cannot
exclude a registered sibling's reset, and it is a class hole, not a
regression — the witness is new and `reset.rs` / `lease.rs` are C12-identical.

### The gap, measured

`admit()` takes `WRITE_GATE.read()`, so two registered sibling threads both
receive permits and the fence orders nothing between them. The permanent
witness holds a real summary permit and a registered sibling's real
`reset_lexical_profile_counters` completes under it:

    B713 SUMMARY_RESET before=7 while_held=0 reset_completed_while_held=true reset_fenced=false

at 8 and 1, exit 101 on `while_held == before`. `publish()` is already
excluded (it takes the write side); the reset is not.

### Ruling — one exclusive acquisition, one forwardable permit, both reset entries

1. **`admit_exclusive()` on the existing `WRITE_GATE`**, exact-cfg under the
   same three predicates as `admit()`. Acquisition order, not negotiable:
   `debug_assert_eq!(depth, 0)` -> wait-observable increment ->
   `WRITE_GATE.write()` -> wait-observable decrement ->
   `blocks_unregistered_thread()` (refuse with `None` before any depth
   change) -> depth += 1 -> return the permit. The registration check stays
   **after** the write acquisition: checked before, it can pass under
   generation G-1, lose the race to `lease()`'s fence, acquire after the new
   generation's `reset_for_lease`, and mutate the new lease's state. The one
   acquisition order is written once: extract it into a private helper that
   both `publish()` and `admit_exclusive()` call (the helper takes which
   wait-observable to bump); `publish()`'s public shape and behaviour are
   unchanged.

2. **A dedicated wait observable.** `FENCING` stays the lease/publication
   fence's observable; `admit_exclusive()` bumps its own exact-cfg counter
   (getter in the style of `fencing()`), and `FENCING`'s three existing
   pollers must not see the reset's wait. The class this brief exists to
   close is cross-test interference, and three standing witnesses keep their
   specificity only if the reset's wait is unmixed with theirs. No new
   fence, no new lock: this is an instrument beside the existing
   acquisition, not a second gate.

3. **`Admitted`'s guard becomes an exact-cfg enum** over the two guard types
   (`RwLockReadGuard` / `RwLockWriteGuard`); `admit()` builds the shared
   variant, `admit_exclusive()` the exclusive one, Drop remains the single
   depth decrement. Every `_admitted` leaf keeps `&Admitted` unchanged. No
   leaf, no census row and no production byte changes; `Publication` keeps
   its type and its Q1 meaning.

4. **The exclusive root is a distinct type.** `admit_exclusive()` returns
   `ExclusiveAdmitted`, a private-field wrapper over the `Admitted`, with
   `admitted() -> &Admitted` for the leaves. It has no other constructor, so
   a shared permit cannot compile into a reset body.
   `reset.rs::reset_lexical_profile_counters` and `epoch::reset_for_lease`
   acquire through it; `reset_unleased` and `reset_production` take
   `&ExclusiveAdmitted` and hand `admitted()` to `retire_admitted`,
   `clear_registrations_admitted`, `clear_stash_admitted` and every other
   leaf, which keep `&Admitted`. The permit is held through every counter
   store and TLS clear: those writes are the cut's own state, so Q16's
   early release does not apply here and a release before them is a STOP.

5. **Not authorized:** a second mutex or fence; per-section admission; a
   `Publication`-to-`Admitted` conversion (the two tokens stay distinct and
   must never nest); moving the registration check before the gate;
   fabricating a guard; TLS depth; making the summary exclusive (the
   observation stays shared — ordinary live increments remain concurrent by
   design; the pair that must never interleave is the observation and the
   global state cut); reclassifying `drop_session` as a cut (it removes one
   session's own contribution, a lifecycle event a live profile may
   legitimately observe; only the global zeroing/reset family is exclusive).

6. **Disclosed consequences.** (a) An unregistered reset now waits at the
   write gate for in-flight readers before being refused, exactly as
   `publish()` does; today it is refused without waiting. It mutates
   nothing either way. (b) A reset issued at depth >= 1 is a contract
   violation: debug builds catch it at the `debug_assert_eq!`, release
   feature builds would self-deadlock on the write acquisition. The
   census/no-nesting scan is the enforcement; no new runtime check is
   authorized. (c) The new observable is written only around the wait, so
   the three `fencing()` pollers keep their meaning.

7. **Falsifier, permanent.** The Q20 witness keeps its shape and takes its
   fixed expectation (`while_held == before`, `!reset_completed_while_held`,
   `reset_fenced`), and gains the completion proof: after `drop(access)` and
   `join`, the counter must be zero, so green cannot mean "the reset never
   ran". Replace the unbounded spin with a bounded wait in the project's
   30 s deadline style; on expiry fail quoting both samples (a hang is not a
   verdict). Add one witness for `reset_for_lease`: hold a permit, a
   registered sibling calls `epoch::reset_for_lease(&lease)` while the main
   thread polls the observable; while held the sibling has not completed;
   after release and join the epoch id sampled at depth 0 differs from the
   id sampled before. Both red on the pre-fix source at 8 and 1 (the first
   red is recorded above), green after, both at 8 and 1.

8. **Mutants.** M1 points `reset_lexical_profile_counters` back at `admit()`
   -> the Q20 witness red with the quoted tuple; M2 the same for
   `reset_for_lease` -> its witness red. Trace each predicted red to the
   observer, quote the line, restore, show the restore green. The six
   remaining mutants (Q4 B/C arms, Q10 handoff) still follow.

9. **Class duty (scan, not a list).** Every body reachable from
   `admit_exclusive` must reach only `_admitted` leaves: a reachable strict
   wrapper, `admit()`, `publish()` or `lease()` is a STOP. Every
   `Publication` / `Admitted` holder must reach neither reset entry (table
   with dispositions), including `Enabled::drop`,
   `logical_profile_test_reset`, the two `reset_for_lease` sites in
   `logical/budget.rs`, `lease_drained` and `test_limit`; all are depth 0
   with no payload lock, and any other disposition is a STOP. Re-run the
   destructor census over the reset family: under a write guard an admitting
   destructor deadlocks instead of tripping the debug assert, so the family
   must show zero admitting destructors at the new shape. Every payload-lock
   holder calling `admit_exclusive` is the reverse edge = STOP. A test whose
   reset would now block on a sibling's permit must release or join it
   first; a wait cycle is a STOP. The pinned census counts stay deferred to
   the final source checkpoint, per-class table once.

**Order after Q20.** The exclusive constructor + the guard enum + the
exclusive root type + both reset entries + both permanent witnesses (red
before, green after at 8 and 1) + M1/M2 -> budget -> the six mutants
including the Q4 B/C arms and the Q10 handoff -> the standing replays with
the once-only pinned census -> the acceptance runs (full lib 3x at 8
threads, once at 1, the workspace, the release erasure, the test-name
delta). Resume with `CDX_FORK`.

## Director ruling — 2026-11-09 (stage-14 stop): Q21 the mutant the type makes inexpressible; Q22 the ownership disposition

Q20 is accepted in shape: one private `acquire_write(&waiting)` order shared
by `publish()` and `admit_exclusive()` (depth assert -> wait observable ->
write gate -> observable decrement -> registration check after the gate ->
depth +1), the `AdmittedGate` enum with the only two guard shapes, the
`ExclusiveAdmitted` root whose only constructor is `admit_exclusive`, both
reset entries converted, `reset_unleased`/`reset_production` taking the
exclusive proof and the leaves taking `admitted()`, and the permit held
through every counter store and TLS clear. The separate `RESET_WAITING`
observable is written only around the wait, and `FENCING` stays the
lease/publication fence's. The two permanent witnesses were red at 8 and 1
on the real pre-fix source (`B713 SUMMARY_RESET before=7 while_held=0
reset_completed_while_held=true reset_fenced=false after=0`, both tests,
0/2/0) and are green at both (`before=7 while_held=7
reset_completed_while_held=false reset_fenced=true after=0`, 4/0/0), with
15/15 focused at both. No speculative change, no weakened expectation.

### Q21 — an inexpressible mutant is the type working, not a falsifier gap

The prescribed M1 substitution stops at compile time: E0599 `no method named
'admitted' found for struct 'Admitted'` and E0308 `expected
&ExclusiveAdmitted, found &Admitted`, because `reset_unleased` /
`reset_production` take `&ExclusiveAdmitted` and a shared `admit()` cannot
produce one. That **is** item 4 of the Q20 ruling working. Record both exact
errors in the report and do not hunt for a compiling way to write that
mutation; do not weaken the signatures, add a conversion, or add a
constructor for the mutant's sake.

Replace M1/M2 with:

- **M1' (class, runtime).** Inside `admit_exclusive()`, acquire the shared
  side and return it in the same root: `AdmittedGate::Shared` over
  `WRITE_GATE.read()`, the same registration check and depth bookkeeping,
  nothing else changed. Every caller still compiles. Both permanent
  witnesses must go red at 8 and 1 with the same tuple as the pre-fix red;
  quote both. One mutant covers both roots because the strong type merges
  the two entries into one acquisition — a per-root pair would be two names
  for one mutation. Per-root discrimination is already established by each
  witness's own pre-fix red; this mutant proves the current source still
  discriminates both. Restore byte-for-byte (SHA) and show both green.
- **M3' (instrument).** Neutralize the wait observable (`RESET_WAITING`'s
  bump or `reset_waiting()`), leaving the gate in place. Both witnesses must
  fail at the bounded 30 s deadline, quoting both samples, without hanging:
  that calibrates the deadline path Q20 introduced and proves the
  observable is load-bearing. Restore and show both green.

The pre-fix receipts stay as the historical red; the mutants do not replace
them.

### Q22 — the ownership disposition is accepted

The canonical candidate's delta is exactly the Q20-mandated replacement and
nothing else: 8 MOVE (existing macro and capture rows shifted), 1 REMOVED
(the old `Admitted._gate` `RwLockReadGuard` storage descriptor), 8 NEW
(`Admitted._gate: AdmittedGate`, the two variant guards,
`ExclusiveAdmitted.access: Admitted`, and the four rows inside the extracted
`acquire_write`), all in `lease.rs`; UnknownHostPayload unchanged at 79,
self-reaching unchanged at 61, every UnknownConstruct status preserved, no
other file touched. The ownership obligation is preserved and strengthened:
the guard is still owned by the permit and released by its Drop, and the
enum only adds the exclusive variant. The second original flag — the
`|poison|` closure at `admit`, baseline 66 -> current 92 — is the accepted
duplicate-descriptor pairing class (Q5/Q9): that closure still exists, and
the canonical first-match pairing consumed its descriptor at the new
`acquire_write` 135. Copy the complete candidate unchanged, no hand edits,
and replay the ownership cell to exact green as in the earlier stages
(`c13-ownership-exact` shape). Pinned census counts still update once at
the final source checkpoint.

**Order after Q21/Q22.** M1' and M3' (red at 8 and 1, restored) -> budget
-> the remaining mutants including the Q4 B/C arms and the Q10 handoff ->
the standing replays with the once-only pinned census -> the acceptance runs
(full lib 3x at 8 threads, once at 1, the workspace, the release erasure,
the test-name delta). Resume with `CDX_FORK`.

## Director ruling — 2026-11-09 (stage-15 stop): Q23 the queue-arena admission precedes its payload lock

Q21 and Q22 are accepted: M1' (the shared guard inside `admit_exclusive`)
turns both permanent witnesses red at 8 and 1 with the pre-fix tuple and
restores byte-for-byte to green; M3' (a zero `reset_waiting()`) makes both
fail at their 30 s deadline quoting both samples without hanging, which
calibrates the deadline path and proves the observable is load-bearing; and
the accepted ownership candidate is copied unchanged with the exact cell
green (61 self-reaching, 79 UnknownHostPayload, 8,765 UnknownConstruct).
The Q23 stop is correct and the edge is real: `QueueArena::shared` holds
`epoch.queue_arena` (profile_queue.rs:110) across `Epoch::storage` at
116/117, whose wrapper admits at epoch.rs:134. It is a static forbidden
edge, not a reproduced deadlock, and the brief's own hard rule makes it a
STOP.

### Q23 — authorize the narrow repair; the boundary is the edge, not one transaction

The proposed repair is accepted as written:

1. `QueueArena::shared` keeps its body and gains one exact-cfg `admit()` at
   entry, **before** `epoch.queue_arena.lock()`; both storage calls inside
   become `epoch.storage_admitted(&access, ...)`; the success path, the
   `slot.take()` before the claims, and the `*slot = Some(...)` order stay
   unchanged. Production erases the proof and keeps one source body.
2. Do **not** convert the rest of `ProfileQueue::new`. `epoch.storage` at
   :154, `Context::capture` at :169 and `Epoch::retire` at :200 all run at
   depth 0 with no payload lock held today, and the hard rule is "no
   admission under a payload lock", not "one transaction per constructor".
   Pulling them into `shared`'s transaction would pull `capture_admitted`,
   `retire_admitted` and `ProfileQueue::drop`'s `unregister_root` (an
   admitting destructor) into a held-permit scope for no rule's sake —
   over-generalized scope. Two sequential transactions (the queue's own
   storage claim, then the arena's) are correct because nothing observable
   needs them atomic.
3. Refusal stays `None` before the lock: `admit()` performs the registration
   check itself, an unregistered thread still falls back to the guest queue,
   and no guest-visible behaviour changes.
4. Class duty: run the reverse-edge scan (a live payload-lock guard whose
   scope contains a strict admitting wrapper) as a recorded table over the
   `rum-runtime` profiling, suspension and evaluation-cache surfaces. The
   pre-fix red is this exact hit (quote the coordinates); the post-fix run
   must show zero. `queue_arena` has exactly this one user, so the mutex's
   class is provably closed. Any further hit whose admitting callees all
   already have `_admitted` variants is converted under this same shape and
   disclosed; a hit that would need a new bridge is a STOP.
5. No second fence, no TLS fallback, no fabricated guard, no nested
   acquisition in the lower-level claim/resize methods, and the owned
   decreases, drops and `release_deferred_charge` stay unconditional.
6. No runtime falsifier is constructible for this edge today: no writer
   holds the write gate across `queue_arena`. The falsifier is the scan's
   red -> green on the real pre-fix/post-fix source pair with both
   coordinate sets quoted. Do not invent a writer to manufacture a deadlock.

Then resume the budget classification (index/arena growth and owned
refunds), which this repair does not close, and continue with the remaining
mutants and acceptance.

**Order after Q23.** The narrow repair + the reverse-edge scan table ->
budget -> the remaining mutants including the Q4 B/C arms and the Q10
handoff -> the standing replays with the once-only pinned census -> the
acceptance runs (full lib 3x at 8 threads, once at 1, the workspace, the
release erasure, the test-name delta). Resume with `CDX_FORK`.

## Director ruling — 2026-11-09 (stage-16 stop): Q24 the Q16 fixture pins from its own worker; the TLS appendix is classified

The round is accepted as evidence only. No crate file was edited: I re-hashed
the five quoted files and all five are exact (`brief713_context_drop.rs`
`bb6bc7b3…`, `context.rs` `cc28d760…`, `segments.rs` `ccc95130…`,
`profile_queue.rs` `7ee9caf3…`, `epoch.rs` `8c97360a…`), the tree is C15, and
the Q23 repair is still unapplied at `profile_queue.rs:110/116/117`. The
pre-fix scan is recorded as ruled (131 files, 443 occurrences, 417
conservative name pairs, 13 dispositions, all 131 source hashes exact). The
Q16 witness is green 1/0/0 at 8 and 1. The census count stays deferred: no
pin at C16.

### Q24 — the real data pin moves to its own scope worker; the schedule is preserved

The edge is real and belongs to this brief's class. `brief713_context_drop.rs:28`
holds the real `Epoch.data` guard across `Context::enter_with_control` at :44,
whose wrapper admits at `context.rs:137`; Q16's mandated early release at :236
means the restored `Option<Segment>` is dropped on the invalid return, and
`Segment::end` admits at `segments.rs:610` while that guard is still live (the
explicit `drop(data)` is at :48). Entry-plus-existing-leaf cannot close it: a
pre-lock permit only moves the first acquisition, and retaining the permit
through the segment drop is exactly the shape Q16 forbade. The exclusion is
real, not cosmetic: a payload guard held across an admission is the third leg
of the cycle this whole brief closes — a `WRITE_GATE.write()` waiter (any
concurrent `publish()` or `admit_exclusive()` in the same test process), the
retiring worker waiting on `data`, and the admitting thread holding `data`.
Excluding the row would manufacture the required zero; do not.

The proposed fixture-only schedule is authorized as written, with these
conditions:

1. Move only the real `Epoch.data` pin onto a scope worker. The main thread
   keeps, unchanged: the TLS header and spare slot (`segments::enter` at :24),
   `Context::capture` at :25, the real `Context::enter_with_control` wrapper
   call, and every expectation (`invalid && open`, `no_nested_admission`,
   `after == before`, and the exact `B713 CONTEXT_DROP` tuple). The test name
   stays.
2. The pin worker locks the real mutex itself (a `MutexGuard` cannot cross
   threads), signals readiness, and holds the guard until a release signal. The
   main thread observes readiness before the retire worker starts. The sole
   release sender is a local of the parent's scoped closure so its `Drop`
   releases the pin on every exit; use bounded `recv_timeout` waits with assert
   messages; the pin worker performs no admission and no profiling mutation
   under its guard.
3. `open` is read from a retained real `Arc<RecordPath>` obtained from
   `data.records` under a momentary guard before the pin worker starts (or from
   `Context::records` if you prove the same allocation), read after validity
   goes false — never through a data guard the admitting thread holds.
4. Do not move the `Context` call to another thread: the TLS header and spare
   slot the restore depends on are the parent's, and rebuilding them on a
   worker would replace a calibrated schedule with a new one.
5. No production `#[cfg(test)]` pause seam, no fabricated writer, guard,
   validity or path state, no new skip, no expectation change, and no exclusion
   of this test from the scan. If the fixture cannot be built without touching
   production source, STOP.
6. Re-calibrate after the change: the unchanged test green at 8 and 1, and the
   Q16 mutant (retain the entry permit past `restore_admitted` to function end)
   red at 8 and 1 with the `nested profiling admission` receipt, then restored
   byte-exactly.

### The TLS appendix — classify, do not convert

The eight strict calls under `PHASE_LIFO` borrows (`logical/mod.rs:1194/1195/1198`
in `push_phase`, `:1217` in `take_phase`; `tests/evaluator.rs:832/836` and
`:901/905`) are thread-local `RefCell` borrows, not cross-thread payload mutexes:
they cannot form a cycle with `WRITE_GATE`, and this brief's lock-order rule is
about cross-thread locks. They stay in the scan table as a classified appendix —
never excluded, never claimed as deadlocks. For each one record: (i) the
enclosing body holds no `Admitted` (the strict call itself admits at depth 0);
if any enclosing caller holds a permit, that site is a STOP; (ii) the borrow is
thread-local and released on unwind; (iii) every `PhaseEntry`/`Call`/`Storage`
destructor runs outside the borrow (`take_phase` pops and returns the entry,
`push_phase` only pushes) — if one ran inside it, that would be the Q18 class
and a STOP. Record `push_phase`'s storage-refused branch
(`epoch.retire(Retire::DiagnosticCap)`) at depth 0 with no payload lock.

### Order after Q24

Apply the Q24 fixture schedule and the already-authorized Q23 repair (unchanged:
entry `admit()` before `queue_arena.lock()`, `storage_admitted` for both claims,
body and `slot.take()`/assignment order intact; do not convert the rest of
`ProfileQueue::new`) -> re-run the reverse-edge scan on the post-fix source and
record zero with both pre-fix coordinate sets quoted (queue `:110/116/117` ->
`epoch.rs:134`; fixture `:28/44/48` -> `context.rs:137`, `segments.rs:610`) ->
the Q16 mutant re-kill at 8 and 1 -> the budget classification (index/arena
growth, owned refunds) -> the remaining mutants including the Q4 B/C arms and
the Q10 handoff -> the standing replays with the once-only pinned census -> the
acceptance runs (full lib 3x at 8 threads, once at 1, the workspace, the release
erasure, the test-name delta). Resume with `CDX_FORK`.

## Director ruling — 2026-11-09 (stage-17 stop): Q25 the failed allocation returns its cell to its caller

Q23 and Q24 are accepted as implemented. `QueueArena::shared` now admits before
`epoch.queue_arena.lock()` and both claims go through `storage_admitted`; the
Q16 fixture pins from its own scope worker with a retained `Arc<RecordPath>`
for the open read, a readiness channel, a parent-local release sender and a
bounded wait; the main thread keeps the TLS header, `Context::capture`, the
real wrapper call and every expectation. Q16 is green 1/0/0 at 8 and 1, its
hold-to-end mutant is red 0/1/0 at both with the nested-admission receipt,
and the SHA-exact restore is green at both. Focused 15/15 at 8 and 1, the
three queue neighbors 3/3, 1414 test names unchanged.

The Q25 stop is correct and the edge is a production one:
`ProfileQueue::add` holds `root` (:263) and `arena.data` (:264), moves the
`Cell` by value into `Arena::allocate` (:279), and every `allocate` failure
return (`arena.rs:191/199/205/220`) drops that parameter before the caller's
`drop(data)`/`drop(root)` at :280/281 — so a sole `Kind::Boundary` Arc (or a
`DownValuesView`-owned one) runs `ProfileBoundary::Drop` → `abandon` →
`admit` at `logical/mod.rs:160` under both payload guards. Admitting before
the locks would turn the same destruction into a nested admission; the
`_admitted` leaves do not help because the admitting operation is an implicit
`Drop`. It is a static forbidden edge, not a reproduced deadlock, and the
direct-name scan could not see it.

### Q25 — the allocation failure returns ownership; the caller drops after the unlock

1. `Arena::allocate` becomes `fn allocate(&mut self, cell: Cell) ->
   Result<CellId, Cell>`. Every failure return carries the cell back: the two
   checked-arithmetic computations, the `storage.resize(peak)` refusal, the
   `try_reserve_exact` refusal (keeping its existing rollback), the second
   `storage.resize` refusal, and the free-slot invariant violation. Nothing
   inside `allocate` drops it; restructure the `Node` construction so the
   free-slot check precedes it (or return the field back out). `Result` is
   `#[must_use]`, so no caller can ignore the failure arm.
2. Every caller releases both guards, then drops the returned cell, then
   performs its existing retire call, then returns exactly as today:
   `ProfileQueue::add` (`drop(data); drop(root); drop(cell);
   retire(Retire::DiagnosticCap); false`) — the boundary settles before the
   epoch retires, as now; `ProfileQueue::new` collects the failed `Cell` in
   the closure and drops it after `drop(data); drop(root);` before the retire
   (its cells are `Guest`/`EvaluationReturn` today, so no observable change —
   disclose); `attach_boundary_admitted` does the same with its existing
   permit held (its boundary is a non-final clone borrowed from the Q18
   owner, so its Drop cannot admit — record that proof in the table). Direct
   sequence tests consume the `Err` arm explicitly.
3. No permit is taken in `add`, `allocate` or any `Budget::claim`/
   `Storage::resize`; no `_admitted` conversion, no TLS, no cfg-only
   divergence, no fabricated guard. The success path moves the same values as
   today; the function is unreachable without an epoch, so no default-path
   cost — disclose the success-path codegen and any shape check the brief's
   budget rules require.
4. Class duty — the Q25 class is an implicit admitting destruction under
   payload guards. Build the table of every site where a `Cell`, a
   `Kind::Boundary` Arc, a `DownValuesView`, or a final `ProfileBoundary` Arc
   can be destroyed, with the payload-guard state at the destruction:
   `take_cell` + caller drop, the insert-failure `release` + drop,
   `pop_metadata`, `retire_diagnostics`, `ProfileQueue::drop`, the
   failed-allocation arms, the `new` residue, `insert_before_consumer`, and
   the attach non-final clone. Every row must show no payload guard live, or
   an ownership argument that the value is non-final. Any row that cannot be
   closed is a STOP. Extend the reverse-edge scan's discovery to implicit
   destructors whose type transitively owns an admitting Drop.
5. MAY list: exactly one file is added — `suspension/core/profile_queue/arena.rs`
   (the signature and the failure returns only).
6. Falsifier: the scan's red -> green on the real source pair — pre-fix
   `add` :263/264 → `allocate` :279/`arena.rs:191` → `mod.rs:275/284` →
   `:154/160`; post-fix zero, with every cell drop at an after-unlock
   statement. Calibrate it: a source mutant that drops the returned cell
   inside `add`'s `Err` arm before the unlocks must be flagged red by the
   scan, then restored byte-exactly and re-run to zero. Plus a permanent
   test on the real failure path at 8 and 1: with a real budget cap, the
   failing allocation returns `Err(cell)` still holding the sole
   `Kind::Boundary` Arc, `add` returns false, the boundary's settlement
   completes exactly once, the arena's slots and capacity are unchanged, and
   nothing panics. No deadlock reproduction, no invented writer, no seam.

### Order after Q25

The repair and its class table -> the budget classification closes on it
(index/arena growth and owned refunds) -> the remaining mutants including the
Q4 B/C arms and the Q10 handoff -> the standing replays with the once-only
pinned census -> the acceptance runs (full lib 3x at 8 threads, once at 1,
the workspace, the release erasure, the test-name delta). Resume with
`CDX_FORK`.

## Director ruling — 2026-11-09 (stage-18 stop): Q26 the Q4 frame disables the refusal, not the predicate

Q25 is accepted as implemented: `Arena::allocate` returns `Result<CellId,
Cell>` with every failure path carrying the cell (the free-slot check now
precedes `Node` construction, no drop inside), and all three callers release
both guards before disposal and retirement; the real-cap sole-boundary
witness passes at 8 and 1, the implicit-drop scan is red before, zero after,
red under its caller-side mutant, and zero after SHA-exact restoration;
focused 16/16 at 8 and 1; 1,415 test names. The new `arena.rs` closure and
the ownership candidate (43 MOVE / 1 NEW, exact 61/79/8,769) are recorded for
my gate.

The Q26 stop is correct and the fault is in my Q4 frame, not in the attach
body. Making `blocks_unregistered_thread()` return `false` at its first line
makes the function lie, and the frozen probe
(`brief712_probe.rs:96`) asserts that same function is `true` for its
outsider — the instrument contradicts the experiment's own frozen text
before the barrie, so the attach body is never reached and the 30 s
timeouts (8 and 1) carry no attachment information. The coordinate
correction (parent barrier :105, observer print :109) is accepted.

### Q26 — the frame removes `admit()`'s refusal branch; the predicate stays truthful

The alternative is authorized exactly as proposed:

1. The frame deletes only `admit()`'s registration-refusal branch
   (`lease.rs:93-95`): after `WRITE_GATE.read()`, no `blocks_unregistered_thread()`
   check, then the unchanged depth increment and `Some(Admitted)`. The
   helper keeps its body and its meaning; `acquire_write()` keeps its check;
   `WRITE_GATE`'s read and write protocol and the depth assertions are
   untouched. The experimental condition is "the read-side exclusion layer
   does not refuse", not "the predicate is false".
2. The probe and its expectation are unchanged, and its registration
   assertion stays truthful and load-bearing: it still proves the outsider
   is unregistered while every admission in the window succeeds.
3. Arms: A (fixed body, frame intact) green `stale_attach_no_fault=true` —
   the recorded mask, unchanged. B (frame + fixed attach) must be green
   `true` at 8 and 1. C (frame + the assertion-before-lookup mutation) must
   be red `false` at 8 and 1. Between B and C the attach body is the only
   variable.
4. Restore the frame byte-exactly (fixed `lease.rs` SHA `2ea56da9…`),
   re-run the unframed fixed attach green at 8 and 1, disclose every mutated
   line per arm, and keep the logs under `evidence/mutants/`.
5. Never mutate `blocks_unregistered_thread()`'s body for this matrix again;
   a frame that needs the predicate's answer to change is a STOP, not a
   calibration.

Then the remaining mutants including Q4 arm C and the Q10 handoff, the
standing replays with the once-only pinned census, and the acceptance runs
(full lib 3x at 8 threads, once at 1, the workspace, the release erasure,
the test-name delta). Resume with `CDX_FORK`.
