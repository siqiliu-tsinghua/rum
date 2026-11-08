# Brief 707 — LP08: silent retention and restored pause attribution

**The working tree carries the uncommitted LP08 install (16 sources, the
replayed inventories, the director's census edit). Its full workspace has
exactly two red tests, both in `crates/rum-host/tests/logical_profile_lifecycle.rs`:
the native pause sums read 44 and 18 where the LP02 oracle expects 4 and 2.
Brief 706 reproduced the red from clean sources, inventoried every pause site
and consumer, and designed the repair. I calibrated its counterexample myself:
a literal two-call-site silent fix passes the first pause assertion and then
fails at 5 vs 4 (lifecycle) and 10 vs 8 (nested), and makes
`tests/evaluator.rs:91` red. The repair is Option C — silent retention plus a
transparent restored observer plus the preserved physical `input_required`
observation — with every existing LP02 assertion unchanged. The final-source
LP08 campaign and the disabled-path cost panel are brief 708.**

## Goal

Implement Option C so that, on the installed tree:

- the two host lifecycle tests and the source-scope test are green with every
  existing exact value unchanged;
- `pauses` again counts only explicit suspension observers: a retained phase
  or definition never increments `Totals.pauses` and never writes the
  `PAUSE_PARENT` cursor;
- a restored retained record stays the *execution and timing* record, but a
  queue pause attributes to the observer the pre-retention tree would have
  restored, so a suspension cannot add a pause merely because the retained
  record is still open;
- the physical `input_required` observation of the old local finish is
  preserved (it feeds `logical_profile_test_pauses()`), without closing the
  record, touching `Totals.pauses`, or moving the cursor;
- all standing inventories are replayed, the full acceptance chain is green,
  and the tree is left uncommitted for my gate.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`,
  `docs/dev/codex/TEMPLATE.md` (the maintenance duty below is the template's).
- `.temp/director/brief-705-gate.md`, `.temp/director/brief-706-gate.md` — the
  STOP and this round's rulings.
- Brief 706's evidence: `.temp/brief-706/design.md`, `report.json`,
  `pause-call-sites.tsv`, `install-pause-events.tsv`, `install-tree.tsv`,
  `.temp/brief-706/draft-707-retained-pause-attribution.md`.
- My calibration probe: `.temp/probe-707a/` (director instrument; read-only).
  `literalA.log` has the measured 5-vs-4 and 10-vs-8 reds;
  `literalA-evaluator.log` has the `evaluator.rs:91` red.
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` —
  `Call::pause` (:629), `complete_phase` (:965), the down-values exception
  (:1234), `logical_profile_test_span` (:419).
- `…/logical/segments.rs` — `locally_closed` (:140), `restore` (:147),
  `Segment::pause` (:212), `Segment::end`/Drop; `segments/stack.rs:105`
  (fresh entry resets the cursor).
- `…/logical/completion.rs` — `Call::defer_definition` (:11),
  `flush_definition_boundaries`.
- `…/logical/epoch.rs` — `Epoch::path`, `input_required` (:211),
  `defer_definition`/`take_deferred_definition` (:430/:439).
- `…/logical/totals.rs` — `Totals`, `State` (:36), `RecordPath::pause`.
- `…/logical/record.rs:110` — `RecordPath` (`parent: AccumulatorChain<Self>`).
- `crates/rum-host/tests/logical_profile_lifecycle.rs` — the oracle.
- `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs:77–91` — the
  physical-observation consumer.
- Accepted LP08 bars and replay procedures: briefs 702–705 and their
  `.temp/brief-{702,703,704,705}/` artifacts; the canonical LP01 writer
  `.temp/brief-675/refresh_ledger.py` and installer
  `.temp/brief-705/install_ledger.py`.

## Scope

Production sources you MAY modify — nothing else in `crates/`:

- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/completion.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/segments.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/totals.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/epoch.rs`

One new test module and its registration:

- `crates/rum-runtime/src/runtime/profiling/tests/retained_pause.rs` (new)
- the `#[cfg(test)] #[path = "../tests/retained_pause.rs"] mod retained_pause;`
  line in `logical/mod.rs`

Standing replay-generated files are in scope under the template's maintenance
duty (fingerprints, certificates, ownership dispositions, LP01
required/bindings/producers, typed-slot receipts, frame-root lists, the
rejection census counts). Generator output only; no hand edit of a generated
row; disclose every changed row.

Do NOT touch: the host oracle or any existing test; the census assertion text
(see the census rule below); `timing.rs`, `control_builtin.rs`,
`control_timing.rs` or any other carrier; `collector.rs`, the queue,
dispatcher, matcher; public types; `Expr`/`SharedExpr`/`SharedArgs`; manifests,
lockfile, docs. No version control command at all.

All scratch under `.temp/brief-707/`, never `/tmp`. Build every scratch copy
of the tree with its own target directory, never `./target` (the 706 round
measured how a byte-identical copy built into a shared target dir makes the
live tree reuse those artifacts; a live-tree run after an edit must show the
touched crates compiling).

## Target shape — departure is a STOP for review

**1. The private bit (`totals.rs`).** Add one default-false field to the
existing diagnostic `State`; every existing field stays:

```rust
pub(super) struct State {
    // … existing fields unchanged …
    pub(super) pause_transparent: bool,
}
```

**2. The silent-retention observation (`epoch.rs`, beside `input_required`).**
It marks the record transparent and records the physical observation; it
never pauses, closes, moves the cursor, changes ownership or touches a queue.

```rust
impl Epoch {
    /// Mark a record that only LP08 retention keeps open. Physical
    /// InputRequired observation is preserved for the total test accessor.
    pub(super) fn observe_silent_retention(&self, record: RecordId) {
        // Resolve via the existing epoch/path API; a missing, retired or
        // foreign record keeps today's benign missing-record behavior.
        // Under the record's State lock: mark pause_transparent = true.
        // RELEASE that lock before calling input_required(record): the
        // existing lock order is epoch-data -> record-state, and this
        // helper must never hold record-state while taking epoch-data.
        // Then: self.input_required(record); no close, no pause, no cursor.
    }
}
```

Valid only for a retained PhaseKey/Definition record. Do not infer
transparency from key membership anywhere: old compound, assignment, catch and
module carriers use the same local-key class and must stay counted.

**3. Boundary extraction, no duplicated body (`mod.rs`).**

```rust
impl Call {
    pub(in crate::runtime) fn pause(mut self) -> Option<Arc<ProfileBoundary>> {
        if let Some(segment) = &self.inner.segment {
            segment.pause();                  // existing counted observer
        }
        self.into_boundary()
    }
    fn retain(self) -> Option<Arc<ProfileBoundary>> {
        self.inner.epoch.observe_silent_retention(self.inner.record);
        self.into_boundary()
    }
    fn into_boundary(mut self) -> Option<Arc<ProfileBoundary>> {
        self.inner.segment.take();            // Drop -> end -> park -> interval
        let storage = self.inner.epoch
            .storage(std::mem::size_of::<ProfileBoundary>() + 16)?;
        self.inner.finished = true;
        Some(Arc::new(ProfileBoundary {
            epoch: Arc::clone(&self.inner.epoch),
            record: self.inner.record,
            _storage: storage,
        }))
    }
}
```

The two installed `call.pause()` sites become `call.retain()`: the
`complete_phase` InputRequired arm (stash stays exactly as now) and the
down-values exception (before-consumer insertion stays exactly as now). Every
other `Call::pause` site stays counted. `defer_definition` uses the same
observation and keeps its delayed allocation:

```rust
impl Call {
    pub(super) fn defer_definition(mut self) {
        self.inner.epoch.observe_silent_retention(self.inner.record);
        self.inner.segment.take();
        self.inner.epoch.defer_definition(self.inner.record);
        self.inner.finished = true;
    }
}
```

**4. The restored observer (`segments.rs`).** Keep the current validation,
`active.park`, `active.paused` and early returns. Then choose the observer
separately from the active timing record:

```rust
let observed = if active.restored {
    pause_observer(&self.epoch, &active.path)
} else {
    Some(&*active.path)
};
if let Some(path) = observed {
    PAUSE_PARENT.with(|cursor| {
        if cursor.get() != Some((self.epoch.id, path.record)) {
            path.pause();
        }
        // existing parent loop, starting at path.parent: skip locally_closed
        // keys; store the nearest non-local parent or None.
    });
}
// No observer: park only. Do not erase another observer's cursor.
```

`pause_observer(epoch, path) -> Option<&RecordPath>` walks the *record* chain
(no segment install, no path clone, no vector):

1. If the record is not `pause_transparent`, it is the observer.
2. Otherwise start at `path.parent`. Skip a candidate if it is
   `pause_transparent`; also skip it if it is closed and `locally_closed(key)`,
   or closed, settled and `children == 0` (the existing restore skips).
3. Return the first remaining open candidate. A closed, non-transparent,
   non-local candidate with `!settled && children > 0` is the existing
   terminal-parent rule: return `None`. A closed state the existing `restore`
   treats as an invariant fault must still fault (`epoch.fault()`), then
   return `None`.
4. No parent left: `None`.

Read each candidate under its own `State` lock and release it before moving
to the parent. The `restored` flag is the selector: ordinary counted
`Call::pause` segments keep today's cursor semantics. `Segment::end` still
charges the actual active path; only pause attribution is redirected. The bit
is permanent until record disposal (a second input may suspend the same
logical call).

**5. Comments.** Rewrite only the affected comments (`segments.rs:46–51`,
`:140–149`) to state: explicit observers; the key classification is
independent of current openness; retained records restore as execution/timing
records with separate pause attribution. Do not claim adopted records close
locally. Do not rename the predicate.

## Must not change

- Every existing host assertion and every existing test. No new skip.
- LP02 exact values: host `:101` = 4; `:103` = 8; `:106` = 6; `:197` = 2;
  `:201` = 4; `:204` = 4; `:292` = 3. The teardown/closed/`counts()` checks
  stay as they are.
- `Totals::child/report`, settlement, once-only close, abandonment outcomes.
- Every LP08 phase's start, semantic observation, completion outcome, and the
  timing of stash, consumer push, down-values insertion and flush. No
  reparenting, no skipped restoration, no synthesized outcomes, no boundary
  merge or drop.
- Deferred-definition allocation, direct flush and ownership filtering.
- Execution/source restoration, epoch invalidation, cancellation/reset/drop,
  owner/generation filtering, capture times and order.
- `Call` drop on allocation failure and epoch-cap retirement: never mark
  `finished` before successful boundary storage.
- The disabled path: no new env flag, lazy init, inline tuning, special-cased
  test phase names, or subtracted constants.
- No new frame is planned; if you add one, the frame-root closed lists are in
  scope and must be disclosed.

## Falsifiers — my calibration targets

Write the new module with the native test lease, the private Epoch/Call/Context
APIs and real phase keys; read record-local totals and inclusive rows. No
production probe hooks, no test-only alternate implementation. Required
cases (name them by behavior):

1. `complete_phase` retention, iterating `PhaseKey::ALL` with an enclosing
   native Call: the retained record stays open, own `pauses` = 0, physical
   `input_required` = 1, no cursor advance, owner/boundary preserved.
2. The down-values actual finish route: same silence plus exact insertion
   identity and before-consumer order.
3. Deferred definition: no own pause, one physical observation, deferred
   once, flush keeps policy/identity, completion closes once.
4. Restored retained chain with a pre-existing old observer: the guard pause
   attributes to the old observer (native total 1 after the parent event, 2
   after the resumed child event); the retained record's own `pauses` stays 0;
   the actual restored segment/timer is the phase.
5. Same chain with no old observer: no invented pause, interval ends, another
   observer's cursor is not erased.
6. An explicit old carrier (compound/control PhaseKey): its own pause still
   counts despite `locally_closed`.
7. Bare `f[InputString["prompt"]]`: the total test accessor stays positive via
   the physical observation, logical outcome unchanged.
8. Repeated restore, reset/retirement, diagnostic cap, no-input completion:
   unchanged owner, exactly-once completion/retirement, no idle timer, zero
   faults/leaks.

Calibrate every case red then green — this is not optional, and the red must
be reached through the case's own observer:

- **Pre-repair red (live tree, before your production edit):** add the module
  first and run it against the installed tree. The silence cases must fail
  with the measured inflation (own `pauses` = 1; host 44/18 if you run the
  oracle).
- **Mutant M1** (`retain` calls `segment.pause()`): case 1 red (own pauses =
  1); host oracle back to 44/18.
- **Mutant M2** (`pause_observer` returns the active record): case 4 red;
  host oracle at `:201`/`:103` red with the measured **5 vs 4** and **10 vs 8**.
- **Mutant M3** (drop `input_required`): case 7 red; `evaluator.rs:91` red.
- **Mutant M4** (skip every locally-closed key): case 6 red.
- **Mutant M5** (`defer_definition` keeps `segment.pause()`): case 3 red.

Each mutant runs in a scratch copy with its own target dir; quote the exact
failing test line and assertion, then revert and show the case green. The
mutants and the pre-repair red are the proof that the checks can fire.

## Verification order and bars

Edit and `cargo fmt` first. **Then replay every affected derived artifact
before the first acceptance suite; a later source edit restarts the replay.**

1. **Shape gates**: the new module (red calibration, then green), the two host
   children directly at one thread with `LP02_NATIVE_CHILD=1` /
   `LP02_NESTED_CHILD=1` and `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, the host
   binary at 8 threads, `lp03_leaf_phases_close_after_resume`, and
   `cargo fmt --check`.
2. **Replays**, in the template's maintenance duty: contract fingerprints
   (`cargo test -p rum-stdlib --offline --lib regenerate_local_contract_fingerprints
   -- --ignored --nocapture`, consume `target/brief-544/local-contracts.tsv`);
   certificates (`local_production_acceptance` in the default and
   `--no-default-features` builds, consume `.temp/brief-546/certificates/`,
   exactly the two promoted rows); the ownership/unknown inventories; the LP01
   required/bindings/producers and ledger; the typed-slot receipts; the
   frame-root closed lists; the rejection census.
3. **Acceptance**, after the replays:

```text
cargo fmt --check
cargo test -p rum-host --offline --locked --test logical_profile_lifecycle --no-fail-fast -- --nocapture --test-threads=8
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --offline --locked --test lp03_phase_rows --test lp04_definition_rows --test lp05_assignment_rows --no-fail-fast -- --nocapture
cargo test --workspace --offline --locked --no-fail-fast -- \
  --skip lp03_adopted_phases_are_exact_in_completed_and_resumed_runs \
  --skip lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs \
  --skip lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs \
  --skip lp658_host_eval_reset --skip lp658_host_resume_reset \
  --skip lp655_root_context_falsifier --skip lp656_public_root_context_falsifier
cargo test -p rum-runtime --offline --locked --lib --no-fail-fast -- \
  runtime::profiling:: runtime::suspension:: \
  --skip lp655_root_context_falsifier --skip lp656_public_root_context_falsifier
```

The workspace has no other exclusion; the seven skips above are the standing
switch-gated five plus the two root-context reds, which stay skipped and
disclosed. Run the focused set again at `RUST_TEST_THREADS=1` and report the
new module's named tests separately from the historical 192/2 baseline.

4. **LP01 five-mode parent**, not only a lexical child, at 8 and 1 threads:
   `cargo test -p rum-stdlib --offline --locked --test suspension_oracle
   --no-fail-fast -- logical_counter_port --nocapture`. Per mode:
   required=1115, bound=875, executed=875, encoding-owned=240; 1,592 key
   records and 875 receipts; losses 0/15463/19468/18757/22772; the union
   7,960 keys + 76,460 losses + 4,375 receipts = 88,795. Copy the emissions
   immediately. Compare decoded semantic records against the installed ledger;
   only generator-proved producer position/fingerprint metadata may move, and
   every moved stamp is listed. Any changed semantic row (key, loss,
   multiplicity, receipt semantics) is a STOP, not a refresh.
   Adapt the canonical writer and the 705 installer by path constants only and
   assert the rest is byte-identical; keep the corrected bars (per-mode totals,
   phase-level key rules); the per-cell proof is the authority.
5. **Typed-slot** (`cargo test -p rum-stdlib --offline --test typed_slot_census`).
   A moved receipt is regenerated by its canonical procedure and disclosed;
   no hand edit. Any unreceipted cell change is a STOP.
6. **Census rule.** Dump before (the installed tree) and after with
   `RUM_CENSUS_DUMP`. If the RET population changes, report the complete
   kind/file/column/callee/reason pairing and STOP that item; do **not** edit
   `census/tests.rs`. My ruling and edit follow. The rest of the acceptance
   still runs; the census assertion is then the only expected red and must be
   named as such.
7. **State size.** Report `size_of::<State>()`, its alignment and the
   diagnostic charge before and after. A changed size is disclosed with the
   number; a memory-cap test expectation change is a STOP.

## Deferred to brief 708

Measurement only, on the frozen tree: the final-source LP08 campaign
(ownership, 386 positions, witnesses, batteries, falsifiers, M1/M2/M3, direct
flush, 150 routes, terminal instruction ledger, stack/coverage), the six
canonical builds and forced-SHA equality, and the disabled-path A/B
microbench under the corrected 703 control-envelope/instruction ruling. Do not
run them here and do not claim their bars from this brief; the accepted 702/703
values are the reference, not a substitute measurement.

## Report format

`rum-report/1` JSON at `.temp/brief-707/report.json`; every run logged before
it is parsed; every verdict and panic line quoted verbatim; the pre-repair red,
each mutant's red, the final green, the replay diffs, the LP01 comparison and
the census pairing as sections. Check with
`python3 -I scripts/report.py check 707`. A failed bar stops with the exact
panel, never a looser check.

## Continuation A — director ruling after the first run (2026-11-08)

The first run implemented the target shape, produced the seven pre-repair
reds and the nine green tests, then stopped at
`logical_profile_lifecycle.rs:271` (`reasons[2]` 17 vs 1). **The bar was
mine and is corrected; the worker is not at fault.** The install's retention
keeps the entry-split phase records open until the resumed path completes
them, so a failed restore retires that whole open tree; the global reason sum
now counts those records honestly. I measured the pre-repair tree in a
private target: pauses 18/29/29 and 40, reasons 17/17/34 — identical before
and after the repair, i.e. install-inherent. The oracle now scopes both
reasons bars to the native logical row through the new director accessor
`Session::logical_profile_test_reasons_for(phase)`; the exact values
(`[2] == 1`; `[3] + [4] == expected`) are unchanged.

**The tree already carries the accepted repair and two director edits.** Do
not revert, rewrite or re-derive them: `mod.rs` gains the accessor, and the
host test's two reasons assertions are native-scoped. Do not edit the host
test, `census/tests.rs`, or any existing test. The five production files and
`retained_pause.rs` stay exactly as they are; the only production edits
allowed in this continuation are the scratch-copy mutants below.

At briefing time the full workspace shows exactly the standing
derived-inventory reds (18 `rum-stdlib --lib` failures in ownership_graph /
retention_closure local / census, inventory lookups) and nothing else; the
replay order below resolves them. One earlier run also marked
`rum-runtime --lib` red; it did not reproduce in the standalone or workspace
reruns and no log line survived, so treat a recurrence as a real red and
quote it.

Remaining obligations, in this order (the STOP cut them; nothing else is
added):

1. **Mutants M1–M5** with the calibrated reds named in *Falsifiers*. One
   scratch copy of the current source tree (exclude `.temp/`, `target/`,
   `.git/`), one private `CARGO_TARGET_DIR` for that copy, five sequential
   mutations, each reverted before the next. The live tree is never mutated.
   Cargo's unit hash ignores the package path: a copy that builds into a
   shared target lets the live tree reuse its artifacts, so the private
   target dir is mandatory.
2. **Final green**: the new module, both host children at one thread, the
   host binary at 8, `lp03_leaf_phases_close_after_resume`, `fmt --check`.
3. **Standing replays** (contract fingerprints, certificates, ownership/unknown
   inventories, LP01 required/bindings/producers/ledger, typed-slot receipts,
   frame-root closed lists, rejection census) before the first acceptance
   suite.
4. **Acceptance** exactly as in *Verification order and bars*, including the
   focused set at 8 and 1 threads.
5. **LP01 five-mode parents** at 8 and 1 threads with the canonical writer and
   installer (path constants only), the per-cell proof the authority.
6. **Typed-slot**, **frame-root**, and the **census before/after pairing**
   (`RUM_CENSUS_DUMP`; no hand edit; population change stops that item with
   the full row pairing).
7. **State size** before/after and the **scope audit**; report the source
   sha256 of the six changed files at start and end.

Do not repeat the pre-repair reference run. Do not edit the production files
in the live tree. Stop with the exact panel on any failed bar; a loose check
or a re-baselined gold is never the answer.

## Continuation B — director ruling after run A (2026-11-08)

Run A delivered and I accept: M1–M5 each killed its named observers and
reverted green in one private-target scratch copy; the final shape panel is
green (module 9/9, both host children, host parent 3/3, source scope, fmt);
the standing replays re-emitted the installed LP01 ledger byte-identically
(1,592 keys and 875 receipts per mode, losses 0/15,463/19,468/18,757/22,772,
union 88,795 records, 0 changed rows); typed-slot and frame-root passed; the
census paired 1,162 rows with 0 added and 0 removed; State stayed
104/8/232/248. It then stopped, correctly, at the required LP04 switch test
(`lp04_definition_rows.rs:121`), reproduced at 8 and 1 threads:
`definition.rhs.total` closed `[1, 1, 0, 0, 0, 0, 0, 0, 0, 0]` where the bar
demanded a paused `err`.

**The bar was mine; it encoded HEAD's local-close topology, and the install
owns the change.** Measured, not argued: the 705-installed pre-repair tree
has the same row `[1, 1, 0, ...]` and records **zero** physical
`input_required` observations (the install's suspend path replaced
`finish_input_required` with a counted pause, so the definition records
stopped recording the pause at all); the repaired tree records exactly one
`input_required` on `definition.rhs.total`, `definition.rhs.evaluate` and the
`down_value_rhs_eval` row, and completes the retained call `ok` at its
consumer. The test's own contract is "the pause is recorded on the
definition record, not silently dropped"; `err > 0` was the side effect of
HEAD closing the suspended call locally, not that contract. Under the
accepted Option C the truthful row is one call, one `ok`, one physical
observation; an `err` would contradict `calls == ok + err`. The amended bar
is not weaker: it is red on the pre-repair install (which the old bar never
caught) and green on the repair, while the falsifier's calibrated red
(missing rows) is untouched.

The amendment is director-owned and already in the live tree — do not touch
it:

- `Session::logical_profile_test_input_required_for(name)` in `logical/mod.rs`
  (test-gated, `#[doc(hidden)]`) sums physical `input_required` for the
  `Runtime` phase or `Definition` kind named `name`.
- `lp04_definition_rows.rs`: the two paused branches assert
  `input_required == 1` **and** `ok > 0`; every row/closure assertion is
  unchanged. Values are exact, not `> 0`.
- Calibration: on the pre-repair tree the amended bar is red with
  `definition.rhs.total recorded no paused call: [1, 1, 0, ...]` (0 vs 1);
  on the live tree the switch suite is green at 8 and 1 threads (lp03 3/3,
  lp04 1/1, lp05 1/1), and the LP02 host test (3/3) and the nine-module
  suite (9/9) stay green.
- Live `mod.rs` differs from run A's reported hash by exactly the two
  accessors (`reasons_for` was already present at run A's start; only
  `input_required_for` is new here). The other five implementation files and
  `retained_pause.rs` match run A's end hashes byte-for-byte.

Remaining obligations, exactly (the STOP cut them; nothing else is added):

1. **The acceptance set** of *Verification order and bars* §3 against the
   amended tree: `fmt --check`; the LP02 host binary at 8 threads; the switch
   suite command at 8 threads and again at 1; the workspace command with the
   seven standing `--skip`s; the focused set at 8 and 1 threads.
2. **LP01 five-mode parents** at 8 and 1 threads on the amended tree, same
   canonical writer/installer (path constants only), the per-cell proof the
   authority. Run A's 8-thread replay is the record of the expected values;
   a fresh 8-thread run pairs with the 1-thread run.
3. **Final audit**: the six source sha256 at end, no new warnings, scope
   unchanged; report the amended tree's hashes including both accessors.

Do not repeat the consumed reds (pre-repair reference, M1–M5). Do not edit
`logical/mod.rs`, `lp04_definition_rows.rs`, the host test, `census/tests.rs`
or any test. The scratch copy is the live tree as-is (amendments included)
and must be byte-identical to it at the end. Stop with the exact panel on any
failed bar; a loose check or a re-baselined gold is never the answer.

## Host discipline and hard rules

One cargo command at a time in the dispatcher's 16G/no-swap scope, build jobs
8, test threads 8, a red reproduced at 1 thread before it is called a defect;
timing runs only in 708 and serial. Do not run `memory-scope.sh`,
`director-gate.py`, `git`, or any network command. No `unsafe`, no weakened
assertion, no golden re-baseline, no new skip, no hand-edited inventory, no
third-party engine output. Stop for a new warning, an unruled census identity
change, `UnknownHostPayload`, a changed accepted semantic ledger/position/
ownership bar, or an uncalibrated falsifier. The five allowed production files
and the one new test module are the whole implementation; a sixth file is a
STOP unless the template's standing maintenance duty names it as generated.

## Director gate — 2026-11-08: GREEN (install + repair landed)

Run 1 delivered the five permitted production files at the Option C shape
plus `retained_pause.rs`, and stopped at the native lifecycle's
failed-restore reason aggregate. I measured the same bar on the 705-installed
pre-repair tree in a private target: identical, so the inflation is
install-inherent, and re-scoped both reason bars through the test-gated
`logical_profile_test_reasons_for("native")` with the exact values kept.

Continuation A ran the mutants, the shape panel and the standing replays
green, then stopped at the LP04 switch test. I measured that bar on the
pre-repair tree too: `definition.rhs.total` has the same `[1,1,0,...]` row
and **zero** physical `input_required` observations, so the old `err > 0`
was HEAD's local close, not this test's contract. The oracle now uses the
test-gated `logical_profile_test_input_required_for(name)` and asserts
exactly `1` plus a completed call; I watched it red on the pre-repair tree
(0 vs 1) and green on the amended live tree at 8 and 1 threads.

Continuation B passed fmt, LP02 and both switch suites, and stopped only on
nine native ZeroMQ tests whose loopback bind the worker sandbox denies
(`Operation not permitted`, reproduced at 1 thread; the host bind works).
That is an environment boundary, not a code defect. On the host I ran the
same acceptance on this tree: `cargo fmt --check` clean; workspace
**4,156 passed / 0 failed / 61 ignored** (the seven standing skips);
focused **201 / 0 / 2** at 8 and 1 threads; five-mode LP01 parents at 8 and
1 PASS with `semantic_differences=0`, `moved_stamps=0`,
`byte_identical=true` and 88,795 records (keys 1,592/mode, receipts
875/mode, losses 0/15,463/19,468/18,757/22,772).

Brief 708 then re-measured the final source end to end: terminal 54 -> 42 in
all four cells, ownership `[1,2,1]`, 386 positions, stack 18,304/18,896,
coverage 94/96/416/606, witnesses 145 -> 129 / 108 -> 84, the five
falsifiers, M1 3/3 / M2 1/1 / M3 3/3, direct flush, 150 routes, the host
contract, and the disabled path with no new executed work (largest positive
label +2.71%, inside the layout envelope; 312 Callgrind captures with zero
evaluator-work delta).

**Verdict: the repair is accepted and committed with the install.**
