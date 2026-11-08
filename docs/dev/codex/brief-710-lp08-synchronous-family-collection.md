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

## Director gate — 2026-11-08 (continuation D): the disabled path executes extra work

The repair landed the exact stack in both builds (18,304 / 12,392 / residual
0, eight samples, and all ninety executable chain frames byte-equal), and the
Function instruction rows and the queued 49 control pass. Then the standing
timing panel stopped on eight separated distributions, the largest being
association key reads at +24.78% and +28.09%. I attributed both sides of that
STOP myself, and the larger finding is not the one the worker reported.

**My finding — a real default-path cost the campaign's bar never measured.**
I profiled the same 120-label workload under Callgrind on the two qualified
CLI binaries and compared per-call costs across functions whose call counts
are identical. Six functions on the disabled path execute more instructions
per call in the candidate:

| function | calls | Ir/call |
|---|---|---|
| `counters::record_timing` | 103,920 | 13 → 19 (+6) |
| `timing::record_profile_rule_rhs_timing` | 393,040 | 23 → 25 (+2) |
| `..._dispatch_candidate` | 91,440 | 18 → 20 (+2) |
| `..._dispatch_matcher_outcome` | 49,008 | 20 → 22 (+2) |
| `..._dispatch_fast_outcome` | 21,428 | 20 → 22 (+2) |
| `..._dispatch_prefilter_skip` | 7,003 | 18 → 20 (+2) |

That is 1,747,358 extra retired instructions on this workload. The mechanism
is visible in the disassembly: `record_timing`'s disabled path is a sentinel
compare and a jump, but its prologue grew from three pushes and two stores to
six pushes and four stores because the enabled body now holds more values
live across calls. The same prologue runs when profiling is off. This is
new default-path work, the hard line forbids it, and the campaign's
instruction bar missed it because Ruling 3 restated the 709 bar's Function
rows and the queued control, not the seam functions' own cost. **That gap is
mine, as Ruling 1a's narrow zero-additions condition was.**

**The timing movement is not executed work.** The eight separated labels are
not a work regression: every one of the 55 association functions executes an
identical count (the `association_front`, `association_lookup`,
`association_data_stability` rows differ by hundreds of instructions in
billions), and the label's own operation in isolation is not slower — a
40,000-read program measures 42.3 ms on both binaries, and under Callgrind
the candidate retires 1.56M *fewer* instructions (592,955,683 vs
594,511,526). The panel's 25% is layout or harness context, not work; it must
still be re-measured on the repaired binaries, and the instruction ledger
decides.

**Ruling 7 — the disabled path is the bar, per call, on every function it
executes.** A seam function's disabled path must not execute more
instructions per call than base, in both arms, at identical call counts. The
dispatching wrappers and `record_timing` must be reshaped so their off path
is the gate check alone and the recording body lives behind a cold outlined
call; the six functions above define the ledger, and any other function the
disabled path executes belongs to it too. Measure the whole-workload
instruction total as well, and require it not to exceed base. A cheaper off
path is acceptable and recorded, never padded back to the old number. The
stack bar keeps its exact identity: 18,304 / 12,392 / residual 0 / the eight
samples, all ninety chain frames equal.

**The timing panel is re-run, not re-litigated.** After the repair, run the
703 four-pass three-arm protocol again on the repaired binaries. If labels
still separate, capture per-function Callgrind for each one and let the
instruction ledger decide; an identical per-operation count is a layout
artefact, recorded with its numbers, and is not a work regression. Do not
rebaseline the panel and do not discard a pass.

**What this re-opens.** As Ruling 6 says: canonical replays, fingerprints,
census, mutants and the cost rows all follow the edited source. The census
assertion moves only if the population genuinely changes — otherwise STOP.
Then the remaining acceptance: five-mode LP01 parents at 8 and 1, host
lifecycle and the required-switch suites, focused acceptance, the whole
workspace once.

**What I verified myself.** The per-call ledger above from two qualified CLI
binaries under Callgrind (identical call counts, 16,465,136,997 vs
16,469,766,509 total, +0.028%); the `record_timing` prologues in both
disassembly dumps; the 55 association functions' identical counts; the
standalone timing (42.3 ms both) and the standalone Callgrind totals; and the
report's own stack and frame rows against the two saved binaries.

## Director gate — 2026-11-08 (continuation E): the disabled path is cheaper, and the all-function reducer is not an instrument

Continuation E reshaped the six seams and the report stopped on the expanded
ledger: 30 and 20 increasing equal-call rows in the two builds. I reproduced
both sides on my own fresh captures and the STOP is the instrument, not the
repair.

**The repair does what Ruling 7 asked.** Cold-outlined recording plus
`#[inline(never)]` on the six outer wrappers, and nothing else: the
continuation's whole source delta is six attributes, the body split into a
gate and a `#[cold] #[inline(never)]` helper, and `#[inline]` on the dispatch
gate helper. My own Callgrind captures of the qualified base and candidate
CLIs on the same 120-label workload, with profiling off:

| function | calls | Ir/call |
|---|---|---|
| `counters::record_timing` | 103,920 | 13 → 3 |
| `timing::record_profile_rule_rhs_timing` | 393,040 | 23 → 18 |
| `..._dispatch_candidate` | 91,440 | 18 → 14 |
| `..._dispatch_matcher_outcome` | 49,008 | 20 → 18 |
| `..._dispatch_fast_outcome` | 21,428 | 20 → 18 |
| `..._dispatch_prefilter_skip` | 7,003 | 18 → 14 |

Call counts are identical in both arms. The cold helpers
(`record_timing_slow`, `record_profile_rule_rhs_timing_slow`) have zero
calls, and `record_profile_definition_dispatch` is inlined into the
wrappers; the disabled path executes the gate alone. Per Ruling 7 a cheaper
off path is recorded, not padded back.

**The whole-workload total falls below every base draw.** My fresh draws:
base 16,463,477,222 and 16,458,943,348; candidate 16,456,887,219. The
worker's: base 16,465,136,997 and 16,459,575,574; candidate 16,456,258,584
and 16,455,617,811. Every candidate draw is below every base draw; the
separation is 2.06M instructions against a 16.46-billion total.

**The all-function reducer fails its own null.** Comparing two Callgrind
captures of the *byte-identical* base binary flags 15 increasing equal-call
rows in my capture set (26 in the worker's). A zero-growth criterion over
every function in a dynamically linked process therefore fires on unchanged
code; it is not calibrated, and I will not gate on it. The remaining
identities are the workload's process-level nondeterminism: hash seeds,
allocator paths, and the formatting of the measured durations the program
prints (`rumMicroEmit` prints `ToString[seconds]`, so digit counts move with
the measurement). Of my ten stable rows, four vary between the two base
draws — the printed-duration formatter, a hash-map drop, the standing hash
row `default_builtin_dispatch_spec_for_symbol` (16% spread on identical
code) and a canonicalization row. The
six genuinely stable increases total **+150 instructions** across
16.46 billion: four dynamic-loader/`???` addresses (+36, +6, +24, +25),
`__strtoul_internal` (+24) and `core::str::converts::from_utf8` (+35). None
is reachable from the changed code: on the disabled path the seams call
nothing new, so the repair cannot add work to a libc or std string routine.

**Ruling 8 — the bar's scope, stated as it should have been.** Ruling 7's
"every function it executes" means the functions the *changed code* executes
on its disabled path — the seam bodies and anything they newly reach — plus
the whole-workload instruction total. It does not mean a zero-tolerance
reduction over every function in the process, which the instrument cannot
resolve on this workload (its null is 15 rows). The seam ledger and the
whole total both pass; the residual rows stay in the record with their
vectors, are not re-baselined, offset or excluded. The scope error was mine
in Ruling 7; the worker's literal reducer and its STOP were correct readings
of what I wrote.

**The timing panel is decided by the ledger, as Ruling 7 said.** The panel
re-ran on the repaired binaries: 12 passes, 3 arms, 120 labels, 8 separated
labels, the largest association key reads at +25.42% and +22.80%. Per-label
Callgrind: `sort`/`reversesort` operation rows are exactly equal in both
builds (5 rows each), `deletecases` 6 rows equal, the two association labels
18 of 20 rows equal with the remaining two differing by at most 615
instructions (0.0003%) in both directions across builds, and the association
set-key row within 151. The label totals move in both directions across
builds (`deletecases` +983,651 in one build and -110,676 in the other).
The two association labels are the key-read loop: `-width4000` was excluded
by brief-620 after an unchanged base measured a 2.06 ratio against itself
(brief-622 carried the exemption), and `-width32000` is its sibling, already
at 1.13 in brief-609. Its mechanism is documented in `TODO.md`: the
key-read loop's entry offset within a 64-byte line, +20.8% from a 16-byte
shift and +1.3% under 64-byte alignment. The panel remains FLAGGED as
recorded — no pass is inferred from
it, and no pass is discarded. The layout arm is continuation D's
cold-perturbator build, hash-verified and reused rather than rebuilt on the
repaired source; it is a noise-floor instrument only. The timing lane is
secondary and the instruction ledger decided.

**The stack bar is exact, reproduced by me.** Both arms give
`consumed = 12,392 + 18,304 × depth` at depths 5, 10, 15 and 20 — the eight
standing samples exactly — and my own probe backtraces are 93 frames with
identical names at every index in both arms.

**What I accepted.** Continuation E's repair, its replays (ownership 35
position moves, 0 added, 0 retired; 68 fingerprints; two certificate rows in
both cells; LP01 metadata; typed-slot; frame-root; census 1,192 with 0
added and 0 retired), its six source mutants (12 named reds at 8 and 1,
six restored greens), the Function rows (14 rows equal between arms, queued
F6 49), and the acceptance below.

**The acceptance I ran on the host.** `cargo fmt --check` clean; the
profiling/suspension focus 214 passed / 2 failed / 2 ignored at both eight
and one thread, the two failures being exactly the standing
`lp655_root_context_falsifier` and `lp656_public_root_context_falsifier`;
the host lifecycle suite 6/6 and the switch-gated `lp03`/`lp04`/`lp05`
suites 5/5 at eight and one under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`; the
five-mode LP01 parents 88,795 records with zero semantic differences in all
four (feature, threads) cells and the installed loss ledger byte-unchanged;
and the whole workspace once, 62 binaries, **4,169 passed / 0 failed / 61
ignored** in 403 s with the seven switch-gated names skipped, peak 6.98 GB
of 14 and no OOM. The three pre-existing `register_test_*` dead-code
warnings are the only warnings in the run. The nine native ZeroMQ tests
that the worker's sandbox denies pass here.

Since the scope ruling is mine, I record it plainly: Ruling 7's wording
made a whole-process zero-tolerance reduction look like the bar, and the
worker stopped on it rather than talk past it. The repair itself was never
at fault.
