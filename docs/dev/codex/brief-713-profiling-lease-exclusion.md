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
