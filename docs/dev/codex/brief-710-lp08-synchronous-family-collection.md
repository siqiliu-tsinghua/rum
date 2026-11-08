# Brief 710 — LP08 schema-v2 stage A: the epoch-owned synchronous family index

**Shape: implementation.** Brief 709 (design accepted 2026-11-08) measured the
form; this brief lands only its stage A. The tree is `706b3020`: LP08's counter
promotion landed at `b9b39763` and the workspace is green. `RumProfileSnapshot`
is **not** touched here; stage B renders.

## Goal

Add the logical families the epoch does not own today to an epoch-owned side
index, so that every logical key of `docs/dev/profiling.md:20-27` is collected
process-wide, exactly once, within the existing diagnostic budget, with zero
disabled-path cost. Preserve every legacy recorder and field. Add private
native access to the side rows for the stage-B parity tests. Change no native
record, token, queue, carrier or completion behaviour.

The side index is `N ∪ S`: native epoch rows stay in
`Epoch.data.rows`; the side index holds the synchronous events. Module
scalar/source aliases remain projections of native Module rows (stage B); the
eight fixed scalar defaults are a stage-B publication rule, not collection.

## Context

- `docs/dev/profiling.md:20-27` (families), `:95-98` (bounded diagnostic
  storage), `:113-128` (telemetry vs logical).
- `.temp/brief-709/FAMILIES.md`, `CALL-SITES.md`,
  `evidence/recording-and-call-sites.{json,tsv}` — the current family table and
  every caller, re-derived at `b9b39763`. `.temp/brief-709/DESIGN.md` and
  `src/candidate/` — the measured candidate; `evidence/costs-v5.json` — the
  cost receipts.
- Live machinery: `crates/rum-runtime/src/runtime/profiling/logical/`
  (`record.rs:28-47,147-165`, `epoch.rs:33-100`, `index.rs:17-23,295`,
  `mod.rs:461`), `profiling/timing.rs`, `profiling/counters.rs:3`,
  `evaluation_cache/profile/{mod.rs,source.rs}`,
  `substitution/binding_substitution.rs`.
- The 709 rulings in the design brief's Director gate: duration basis, own-row
  qualification, transient-budget policy, equal-clock invalidation, and the
  injective key encoding are settled; this stage implements only the
  equal-clock policy and the collection shape.

## Scope

MAY modify: `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`,
`logical/epoch.rs`, `logical/epoch.rs`'s module list, `logical/index.rs`,
`logical/record.rs` only if a `Debug`/`Copy` derive is strictly required, new
`logical/synchronous.rs`, new `profiling/tests/synchronous_families.rs`, and
the seven recorder seams: `profiling/timing.rs`, `profiling/counters.rs`,
`evaluation_cache/profile/mod.rs`, `evaluation_cache/profile/source.rs`.
Standing inventory replays are in scope as required by the template.

MUST NOT modify: evaluator, matcher or scoping callers; `Expr`/`Symbol`;
public APIs, features, serde or manifests; native record/token/queue/carrier
layouts; `PhaseKey` membership; native completion order; promotion sets, loss
dispositions or oracle vectors; `debug_tools/basic.rs`;
`RumProfileSnapshot`'s behaviour; any hot-path gate or branch. No new
`unsafe`. No version-control command.

## Target shape

```rust
// logical/synchronous.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) enum EventKey {
    Runtime { phase: &'static str, source: Option<Symbol> },
    Rhs { phase: &'static str, source: Option<Symbol> },
    Dispatch { kind: &'static str, symbol: Symbol, source: Option<Symbol> },
    Counter(&'static str),
    Cache(&'static str),
    CacheSymbol { event: &'static str, symbol: Symbol },
    CacheSource { event: &'static str, symbol: Symbol },
}
#[derive(Clone, Copy, Default)]
pub(in crate::runtime) struct EventValue {
    vector: [u64; 7],      // storage only; the family vector length is stage B's
    durations: [u64; 4],
    input_required: u64,
}
pub(in crate::runtime) fn add(key: EventKey, delta: EventValue, start: Option<Instant>);

// logical/epoch.rs
struct Data {
    // existing fields unchanged
    events: Index<EventKey, EventValue>,
    event_birth: Instant,
}
impl Epoch {
    pub(in crate::runtime) fn event(&self, key: EventKey, delta: EventValue, start: Option<Instant>);
    #[cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))]
    pub(in crate::runtime) fn event_rows(&self) -> Vec<(EventKey, EventValue)>;
}
```

- The side index shares the epoch's existing `Budget` (initial zero-length
  claim, geometric growth). It has no independent cap, lease or continuation.
- `Index::iter` becomes privately available in production with its body and
  order untouched; only the test-only compilation guard is removed.
- `event` rejects an invalid or retired epoch; a `start` strictly older than
  `event_birth` writes nothing; a `start` exactly equal to `event_birth`
  records a visible `InstrumentationFault` and writes nothing. It never
  mutates a guest value or resource outcome. On insertion failure, release the
  data mutex **before** retiring with `DiagnosticCap`.
- Every hook sits inside an existing enabled gate, after the current flag
  check, at the existing event position. No new load, branch, lock or
  allocation is added to any path that runs with profiling off.

## The seven seams

1. **Runtime, non-adopted phases.** The existing runtime path timing recorder
   (`timing.rs:743,760,777`) adds a side event on its enabled slow path. The
   side key must exclude every current `PhaseKey` name (all 50); adopted
   phases stay native-only.
2. **Rule RHS.** `timing.rs:351,368`; source is the expression-rebuild symbol,
   not the cache source accessor.
3. **Definition dispatch.** `timing.rs:432-528`: snapshot the six decision
   counts and four duration fields around the existing single update, add the
   delta once. Never rerun the closure; never change decision order.
4. **Scalar RHS completion.** `counters.rs:3` / `records.rs:86` for
   `definition_rhs_eval`, once per existing completion recorder call.
5. **Cache globals.** `evaluation_cache/profile/mod.rs:210`, exactly the six
   names in `FAMILIES.md`.
6. **Cache symbol events.** `profile/mod.rs:257`, exactly the five names in
   `FAMILIES.md`.
7. **Cache source (definition RHS).** `profile/source.rs:168`. Acquire/update
   the epoch **before** taking the legacy source-timing mutex; the reverse
   order conflicts with native Module alias settlement (epoch→legacy).

`eval_module` is derived from the native Module row; no second writer.
Ordinary decisions pass `None` as `start`; timed seams pass their existing
`Instant`. Do not widen a timing token.

## Invariants

- Every side key is owned by exactly one family; runtime side keys never
  overlap a native phase; Module aliases have no side writer.
- Equal keys aggregate in one index entry; distinct keys (including distinct
  generated semantic identities with equal text) never merge.
- Cross-thread: events written on any registered thread join the same
  epoch-owned row; no TLS map is consulted at observation time.
- Reset invalidates the current epoch before a replacement; old markers cannot
  charge the new epoch; completed side rows stay with their epoch. Session
  drop retires native records and keeps completed side rows, like native
  terminal rows.
- The test lease still admits every thread that writes or reads.

## Gate tests and falsifiers

New `profiling/tests/synchronous_families.rs`, lease-held:

- same key twice sums once; all distinct keys survive; a repeated event does
  not create a second row;
- runtime exclusion covers every current `PhaseKey` (assert against the
  declared list, not a hand count);
- each family appears only with its own flag matrix, and not at all when
  logical profiling is off;
- two-thread aggregation: A starts the epoch, B (registered) writes an event,
  one row holds the sum;
- old event after reset writes neither the replacement epoch nor the retired
  one; equal `start == event_birth` gives an invalid epoch, `faults=1`, and no
  row;
- cap exhaustion retires once, `open=0`, and a guest continuation still
  evaluates a result;
- an unregistered test thread cannot write;
- old alias completion does not charge the new epoch.

Mutants (install each in a scratch copy with a private target; the named test
must go red and the restored code green; quote the verdict lines):

1. remove the runtime-phase exclusion → duplicate native row;
2. suppress one cache global → missing key;
3. apply the dispatch delta twice → wrong field;
4. remove the validity/birth rejection → old-marker leakage;
5. bypass the `Index` budget → cap test fails to retire;
6. move the source hook under its legacy mutex → lock-order witness.

The designer's reducer-only missing/duplicate/wrong-field probes test the
comparison instrument, not these mutations; do not claim them as source-mutant
credit.

## Disabled cost

The standing bar is the brief-709 measured form: at one stable absolute path,
release, CGU=1, jobs 8, offline/locked, two independent builds per arm; the
Function source rows (`try_apply_function`, `finish_function_application`,
`apply_status_function`, `evaluate_status_function_value`) and the resumed
frame row must match base exactly in own/incoming vectors and slopes; the
queued F6 control is `try_apply_function` 49 instructions/call; the Function
stack is 18,304 bytes/level with zero residual and the same eight absolute
samples (four depths × two). Use the 709 instruments copied into your root.
Whole-process totals are context, not the decision row.

**Must not change beside this bar:** two actual `InputString` pauses, prompt
and answer order, Function result 7, entered scopes, terminal outcomes, cache
entry ordering, sampled Return/Throw boundaries, token size, native stack
limit, every phase name and vector. No cost repair by removing work, delaying
collection, duplicating bodies, lazy initialization, flattening carriers,
silent drops or debug-only inlining.

## Verification

One cargo command at a time, offline/locked, jobs 8, tests 8, plain inside
the dispatcher's 16G scope. Mid-change use `cargo check --all-targets` plus
`-p rum-runtime`; the full `cargo test --workspace` runs once at acceptance
(after the shape is green). Replay every affected standing inventory **before**
the first acceptance suite and quote the replay verdicts; a stale
fingerprint/certificate/position-only red is an ordering artifact, not a
regression — replay, then rerun. Never hand-edit or weaken an inventory. A red
is reproduced at `--test-threads=1` before it is called a defect. Callgrind
and timing runs are serial, alone, CPU-pinned; persist raw output before
parsing; a timeout is not data.

## Checkpointing

Keep `PROGRESS.md` with the exact next action; leave a compiling checkpoint
after the shape, after the proofs and after the measurements; preserve failed
logs. No git.

## Deliverable

- The collector, the private accessor and the model tests.
- Exact changed-line and ripple list, the six mutant reds with restored greens,
  the cost rows in both builds, and all replay verdict lines.
- `.temp/brief-710/report.json` in `rum-report/1`, plus
  `python3 -I scripts/report.py check 710` summary and every compliance
  verdict line.

## Hard rules

AGENTS and TEMPLATE apply in full. No vendor differential engine, no live
documentation edit, no re-baseline, no reclassification of a logical row as
telemetry, no widening of a family gate. A departure from the target shape, a
new public surface or an unapproved family is a STOP: report the evidence and
the minimum scope change. The director owns the gate and every commit.

## Director gate — 2026-11-08: STOP accepted; the single-row retirement is authorized

The STOP is correct procedure and the reproduction is sound: the one lost
identity *is* the cfg attribute this brief instructs to delete, identical at
eight and one threads, with no other group changed and no inventory write. A
row whose identity quotes the deleted construct cannot outlive it; the
no-disappearance rule protects behaviour and census identities, not a text that
names source the brief removes.

**Ruling 1 — the retirement is authorized.** Replay the ownership inventory
canonically and install the regenerated table. Terms: exactly this one row is
removed, zero rows are added, and no other identity, status or disposition
changes. Any second disappearance is still a STOP. The table is regenerated by
its canonical tool; no hand edit.

The exact authorized row:

```text
UnknownConstruct  crates/rum-runtime/src/runtime/profiling/logical/index.rs  290  5
unresolved attribute/expansion cfg (any (test , feature = "test-builtin-memory-growth" , feature = "test-continuation-slots"))
```

**Ruling 2 — the InputRequired fault policy is accepted, and needs a
falsifier.** The contract says `InputRequired` publishes no terminal call or
error and leaves a record open; the side index holds no open records, so a
non-adopted phase that suspends has no representable terminal event, and a
visible `InstrumentationFault` is the honest observable. Add one test that
forces the state directly (a non-adopted phase token recorded with
`InputRequired`) and asserts: one fault, invalid epoch, no side row, and an
unchanged guest outcome. The acceptance campaign must also show the ordinary
suspension workloads never reach it: the five-mode LP01 parents and the host
lifecycle suites are that measurement.

**Ruling 3 — the campaign continues.** Finish what the STOP interrupted:
the six source mutants with their named reds and restored greens in a private
scratch target; the two-build disabled instruction and stack rows at the 709
bar; every affected standing inventory replay before the first acceptance
suite; `cargo fmt --check`, the focused suite at eight and one threads, and the
whole workspace once.

**What I verified myself.** `cargo fmt --check` clean; the 12 model proofs
green at eight and one threads; `report.py check 710` OK with 57 citations; the
nine-file scope with no live-doc or inventory change; the STOP row and both
reproduction verdicts quoted from the worker's logs; three pre-existing
warnings only.

## Director gate — 2026-11-08 (continuation A): the twelve additions are authorized

The STOP is correct under the condition I wrote, and the condition was too
narrow: I authorized the retirement without knowing the same source change
discovers twelve new `UnknownConstruct` rows. Every one is a construct this
brief itself creates, and ten of the twelve were already present at the first
checkpoint. Suppressing them would weaken the census; installing them is the
canonical output.

**Ruling 1a — the twelve additions are authorized.** Terms, unchanged in
spirit from Ruling 1: exactly the one authorized retirement; exactly these
twelve additions; no new `UnknownHostPayload`; no new self-reaching row; no
existing identity, status or disposition change; the installed tables are
byte-identical to the canonical discovery output, and the proposed tables are
byte-identical to what the install writes. Position-only moves are replayed
with their row content unchanged. Any further disappearance or addition is
still a STOP.

The twelve authorized rows:

```text
unknown_macros.tsv      profile/mod.rs:259:8    unexpanded macro matches!
unknown_captures.tsv    epoch.rs:174:18         closure captures: |(key, value)|
unknown_captures.tsv    epoch.rs:143:50         closure captures: |start|
unknown_captures.tsv    epoch.rs:147:30         closure captures: |start|
unknown_storage.tsv     epoch.rs:19:1           module synchronous_families has 0 source candidates
unknown_storage.tsv     epoch.rs:59:18          Data.event_birth: foreign representation std::time::Instant
unknown_attributes.tsv  epoch.rs:134:9          cfg (test, test-builtin-memory-growth, test-continuation-slots)
unknown_attributes.tsv  epoch.rs:160:5          cfg (test, test-builtin-memory-growth, test-continuation-slots)
unknown_attributes.tsv  synchronous.rs:6:1      derive (Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)
unknown_attributes.tsv  synchronous.rs:33:1     derive (Clone, Copy, Default)
unknown_storage.tsv     tests/synchronous_families.rs:1:1   source file not reachable from a module declaration
unknown_macros.tsv      timing.rs:859:12        unexpanded macro matches!
```

The last two `unknown_storage` rows are the analyzer's accurate statement
about the `#[path]` test-module registration; they are a limitation of the
census, not a dead module (the thirteen proofs run from that file).

**Ruling 1b — the correspondence adaptation is accepted on one condition.**
Normalizing the two dispatch-closure spellings is allowed only as a
correspondence device: the final tables must equal canonical discovery
(`check_ruling.py`'s first assertion), and the removal set must remain exactly
the one authorized row. Both properties are enforced by the retained script
and I re-ran them; the adaptation is disclosed in
`generator-source-mapping.diff` and `dispatch-closure-source-pairs.json`.

**What I verified myself.** The thirteen model proofs green at eight and one
threads (the new fault test forces a non-adopted `InputRequired`, asserts one
fault, an invalid epoch, no side row, and the same two prompts and result `7`);
`cargo fmt --check` clean; `report.py check 710` OK with 92 citations; the
proposed tables byte-identical to the discovery log; the 177 position moves
inspected as pure line shifts with unchanged text; the live inventory
byte-identical to the pre-brief snapshot and untouched in `git status`;
`ownership-replay.json` shows zero new host payloads, zero new self-reaching
rows and zero ledger moves; the continuation changed only the test file.

## Director gate — 2026-11-08 (continuation B): the census population change is ruled

The STOP is the TEMPLATE's rule working: the census counts moved, the worker
did not touch the assertion, and the comparison is keyed by kind, file, callee
and reason. I rule the population change in, and I update the assertion
myself, as the TEMPLATE requires.

**Ruling 4 — the census is 1,192 (11/1,143/38).** I edited
`crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` with the new
counts and a dated rationale in the file's established style. Verified from
the raw frontier dumps: before 1,162 (11/1,114/37), after 1,192
(11/1,143/38); all 1,162 prior occurrences pair with unchanged kind, file,
callee, reason and both endpoints and columns (112 of them on a moved line);
zero retire; thirty appear. The additions are the collector's own boundaries:
eight in `epoch.rs` (the new `events: Index::new` and
`event_birth: Instant::now` lines, plus four rows inside the unchanged
`existing()` body that the new InputRequired hook's
`logical::epoch::existing()` call at `timing.rs:861` makes reachable), five in
the new `synchronous.rs`, seventeen in `timing.rs` (the non-adopted runtime
event block, the dispatch delta capture, and the InputRequired branch,
including the single RET003 `matches!`). A new call edge that grows the
analysed surface is exactly the case the census exists to disclose; no
coverage mark, classification or promotion set moved.

**Ruling 5 — the certificate red after this edit is an ordering artifact.**
The contract fingerprint hashes every `.rs`/`.py`/`.toml` under `crates/`
(`local/contracts.rs:88-133`), so any source or test-source edit expires every
contract fingerprint by construction; I reproduced the
`stale contract fingerprint O011` red in the census run after my edit. The
canonical certificate replay must therefore run *after* this edit and before
any suite that verifies contracts. That red is not a regression, and no
fingerprint is to be hand-edited.

**Ruling 3 continues.** Mutants, the two-build disabled cost rows, the
remaining replays, and the acceptance suite at the prescribed point.

**What I verified myself.** The two raw frontier dumps and their per-kind
counts; the added-row spans against the current source lines
(`epoch.rs:85:25` `Index::new`, `86:30` `Instant::now`, `timing.rs:835:25`
`EventValue::count`, `859:12` `matches!`, `862:17` `epoch.fault()`); the
baseline provenance (1,532 Rust files compared, 0 different); `census-pairing`
and `census-span-proof` (1,162 paired, 0 removed, 30 added, 112 moved);
`cargo fmt --check` clean after my edit.

## Director gate — 2026-11-08 (continuation C): the disabled stack red is real, and the exact bar stands

I accept the STOP as a real red, not a bar problem. I reproduced it myself on
the two qualified binaries and then attributed it to a single frame.

**The measurement.** Depth-three probe: base 67,304, candidate 67,368. Two
independent builds of each arm: slope 18,304 vs 18,320, intercept 12,392 vs
12,408, residual 0, the eight samples shifted by `96/176/256/336` bytes at
depths 5/10/15/20. That is a reserved sixteen bytes per recursive level on the
disabled path, which the hard line forbids even at zero executed instructions.

**The attribution.** I re-ran the probe with `BRIEF611_BACKTRACE=1` on both
arms, resolved every frame address against the binary's symbol table, and
compared each chain function's prologue `sub rsp` between the two builds. The
per-level chain carries two frames of
`Session::try_apply_down_values_from_symbol` and one of
`Session::try_apply_down_values`. Exactly one frame differs:
`try_apply_down_values` is `0x288` (648) in base and `0x298` (664) in the
candidate; `from_symbol` is 1,240 in both. Its source did not change. The
candidate's `record_profile_down_value_scan` is a new 32-byte standalone symbol
that base inlined into that frame; the dispatch/counter slow-path bodies the
collector grew flipped that inlining. The executed work is unchanged — the
campaign's instruction rows are identical — so the defect is the reserved slot,
not new work, and it is still a violation.

**Ruling 6 — the bar is unchanged; repair the frame, do not move the bar.**
The disabled Function chain must return to exactly 18,304 bytes per level,
intercept 12,392, residual 0, and the same eight absolute samples, in both
builds, with the instruction rows still identical. No rebaseline, no padding,
no work removal.

A one-line shape is sufficient and I measured it: with `#[inline]` on
`record_profile_down_value_scan` in a copy of the candidate tree, the probe
returns 103,912 / 195,432 / 286,952 / 378,472 twice each (slope 18,304,
intercept 12,392, residual 0) and `try_apply_down_values` is back to `0x288`.
That is an existence proof of the fixed point, not a prescribed edit: choose
the shape that keeps every disabled hot frame at its base size, and verify it
with both the aggregate probe and a per-frame comparison of the evaluation
chain (`BRIEF611_BACKTRACE=1` plus the prologue sizes), two builds per arm.

**What any further source edit re-opens.** A source edit changes line spans
and the contract fingerprint context, so the canonical ownership/census
replays and fingerprints must run again after the repair, and the census
assertion only moves if the edit genuinely changes its population — if it
does, STOP and I rule it. Re-run the six mutants on the repaired source; their
receipts are tied to the source they were measured on. The queued F6 control
and every other 702/703 bar stay as they are.

**What I verified myself.** The depth-three measurements above; the frame
table of all 93 chain frames in both arms; the candidate's call list in
`try_apply_down_values` (base inlines the two counter slow paths, the
candidate calls the outlined wrapper); and the `#[inline]` experiment, built
through the canonical probe builder at CGU 1 (`sha256
cc8490e7b67c86919e4d0d609bba1bcfe4ad0487a9681a47f2d23ac4d8ec8581`), whose
eight samples equal the standing bar exactly.
