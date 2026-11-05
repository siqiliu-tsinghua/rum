# Brief 675 — LP07 source stage: Module completion records and completion-owned aliases

**Shape: implementation.** Brief 674 measured the form; this brief lands
it in the live tree and runs the source-stage campaign on the exact final
source. The tree is left for the director's gate; no commit.

**Director ruling (read before anything else).** The selected form is the
measured `src/value-narrow` production tree: the existing by-value
token-first boundary, the explicit original-ID resume alias, the
total-open scalar/source aliases and the narrow Module source helper,
plus the test-only native observer projection from `src/value-port`.
The alternatives are rejected on measured evidence, not preference:
shared-reference and by-value late aliases republish the scalar into a
reset epoch (new epoch `eval_module` = 1); value-epoch grows the
definition RHS 125 → 126 own instructions in both draws; native-handle is
viable (it passes reset) but introduces a new Boxed native-token lifetime
and a private start/finish protocol for 11 more own instructions per
Module call and only a larger stack saving than the no-growth bar
requires. The measured values below are the contract, on one common
source path, CGU=1, two independent builds, two draws. The observer is
test-only; no production shape moves for it. Q2: the full P1507 set and
an individual proof precede every ledger-cell removal; never remove the
672-cell owner set by count. Q3: the enabled-only RecordPath +24 B is
accepted and charged; the `logical/mod.rs` split is deferred as recorded
debt. The 673 timing ruling stands: instruction and stack rows decide,
timing last and non-deciding.

## Goal

When this brief is done, the live tree contains exactly the measured
LP07 form and nothing else, and on the exact final source:

- the four Module phases have native logical rows: one total and one
  initializer-loop identity per Module invocation, each actual
  initializer and body a distinct record, explicit non-owning loop
  aliases across native reentries that reuse the original `RecordId`
  and never open a new logical call; initializer completion before
  store/advance, body completion before module-scope cleanup, loop and
  total after `finish_module_scope`, through all four legacy frames and
  repeated pauses;
- `eval_module` and `eval_module_by_source` publish only at that total's
  real completion, once, through publication bound when the total opens;
  `ProfileReset` disables old-epoch alias publishers before later
  drains; source-only profiling without a native total keeps its
  existing fallback; the 18 Module-source and 17 pure-after-resume rows
  stay exact, and pure work first entered after resume is attributed
  exactly as on the queued path with no heuristic equal-key reopening and
  no retrospective success synthesis;
- cancel, structural discard, Session drop, profile reset, abandonment
  during a paused cleanup and reset-then-resume in body and initializer
  close scope, gensyms and storage: zero open Module records, zero
  leases, zero retained diagnostic bytes after the test lease releases
  its TLS header, and the distinct abandonment reasons preserved;
- the test observer projects only the four Module logical rows from
  actual typed native outcomes, keeps the raw legacy maps and every
  other logical row, and checks the full ten-field native union;
- the measured costs hold with no owner growth: ordinary Module 692 own
  instructions per iteration (from 772), body-pause entry 818 → 705,
  three-pause entry/reentry 829/536 → 760/441, resume frame bodies
  35/138 unchanged; Block 307, definition RHS 125, named Function 193
  and raw Failure 49 unchanged; release stack 18,896/14,064/18,304 B
  and debug 54,048/41,904/49,632 B per level, zero residual; CLI
  `builtin_module` prologue 1,736 B (from 1,784), reentry 1,080 B (from
  1,128), `try_apply_function` 1,080 B unchanged, the four semantic
  resume symbols 328/440/328/440; enabled RecordPath 232 B (from 208),
  the frames 8/72/8/72 and Session 2,128 B unchanged;
- the focused Module/source and lifecycle tests and their seven mutants
  are green, the full P1507 set is green, only individually proven LP07
  ledger cells are removed through the canonical workflow, the standing
  inventories are replayed, `cargo fmt --check` is green, and both
  workspace configurations pass at 8 threads with the seven standing
  skips, including the lease and the required-switch host suites.

## Context pointers

- `.temp/director/brief-674-gate.md` — my ruling and my reproductions.
  Read it first.
- `.temp/brief-674/report-design.json`, `.temp/brief-674/DESIGN.md`,
  `.temp/brief-674/drafts/implementation-lp07.md` — the design record,
  the form families, the calibration and the raw-backed rows.
- `.temp/brief-674/src/value-narrow/` — the measured production form.
  The tree differs from the live tree in exactly the ten files listed
  below. Copy from it; never edit it, never build in it.
- `.temp/brief-674/src/value-port/` — the same form plus the test
  observer; it differs from `value-narrow` only in
  `crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs`.
- `.temp/brief-674/src/base/` — a pristine copy of the live base
  (HEAD `410eb811`), the base arm for the performance campaign.
- `.temp/brief-674/costs/`, `measure/`, `inputs/`, `bin/` — the raw
  Callgrind outputs, stack/prologue/layout receipts, frozen programs,
  labels and readers. Copy; never execute in place. The base and final
  arms must be built at one common source path.
- `.temp/brief-674/inventory/` — the fresh derivation (path sites,
  routes, anchors, loss partitions, P1507 rows). Read-only; it is a
  design artifact, not the standing inventory.
- `.temp/brief-674/b/active/` — the probe build tree. Read-only. Do not
  copy its probe test (`profiling/tests/brief674_probe.rs`), its
  `#[cfg(test)]` module hooks or its `MODULE_PORT` print into the live
  tree.
- Live anchors: `crates/rum-runtime/src/runtime/scoping/lexical/module.rs`
  (starts 69/96/125/160/197/227/266; stores 102/203; cleanup branches
  142/247; `finish_module_scope` 309; abandon 362; the four legacy
  frames 409/437/465/497), `profiling/records.rs:95,112,121`,
  `evaluation_cache/profile/source.rs:113,117,123`,
  `profiling/logical/context.rs:27,98`,
  `suspension/core/profile_queue.rs:199,229`,
  `evaluation_cache/profile/mod.rs:113,118,123,128,133`,
  `profiling/logical/mod.rs:650,700`,
  `profiling/logical/epoch.rs:166`, `profiling/logical/totals.rs:69,88`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md` — standing duties and the
  testing policy.

## Scope

Files you MAY modify:

1. `crates/rum-runtime/src/runtime/scoping/lexical/module.rs`
2. `crates/rum-runtime/src/runtime/profiling/mod.rs`
3. `crates/rum-runtime/src/runtime/profiling/timing.rs`
4. `crates/rum-runtime/src/runtime/profiling/records.rs`
5. `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`
6. `crates/rum-runtime/src/runtime/profiling/logical/epoch.rs`
7. `crates/rum-runtime/src/runtime/profiling/logical/totals.rs`
8. `crates/rum-runtime/src/runtime/profiling/logical/module.rs` (new)
9. `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs`
10. `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs`
11. `crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs`
    (the test-only observer projection)
12. `crates/rum-runtime/src/runtime/profiling/tests/module.rs` (new
    focused Module/source/lifecycle tests) and its
    `profiling/tests/mod.rs` registration.
13. `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/{mod,control}.rs`
    and `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
    only as required to run the new P1507 cases and keep the frozen
    subjects/plans unchanged; disclose every line.
14. The logical-profile loss ledger: only the cells individually proven
    by the full P1507 run, generated by the canonical workflow.

Plus the standing-inventory replays the template makes always in scope
(contract fingerprints, default/no-default certificates, ownership
positions/unknowns, LP01 producer receipts, typed-slot receipts and the
census discovery). Replays are generated; **never hand-edit a derived
cell**. Disclose every changed row and every new unknown row.

MUST NOT touch: any other source, test, workload, manifest, docs, `.git`
or prior `.temp` root; the carrier/epoch/public API/features/serde; the
probe test, probe hooks or `MODULE_PORT` print; the frozen P1507
subjects and plans; any standing inventory by hand; any other owner's
loss cells. A second implementation stage may only be opened by the
director.

## Target form (pinned)

Materialize the ten production files from
`.temp/brief-674/src/value-narrow/` and verify each copy byte-for-byte
against that measured tree. Materialize the observer from
`.temp/brief-674/src/value-port/crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs`,
remove only the probe-only `MODULE_PORT` print and its `selected`
helper, and keep the typed projection and the other-row preservation
assertion. Do not re-derive the form, do not add a digest ritual (git
owns the committed source identity), and do not copy the probe harness.

The shape, for orientation only — the measured tree is authoritative:

```rust
// Four additions in the PhaseKey table; every existing key/name/index/
// policy stays byte-identical.
enum PhaseKey { /* existing 46 */ ModuleTotal, ModuleInitializers,
    ModuleInitializer, ModuleBody }

// The eleven Module completion points use the existing by-value,
// token-first finisher, before their existing consumer:
self.record_profile_runtime_path_timing_adopted_boundary(token, &actual_result);

// The initializer reentry starts through the cold Module helper, which
// pushes a charged PhaseEntry with call: None using the exact restored
// loop identity — never a key search, never a second logical call.
fn runtime_path_timing_start_module_resume() -> Option<RuntimePathTimingToken>;

// At total open, the record state carries module_scalar_alias = true and
// module_source_alias = key.source; terminal publishes the count once,
// settle publishes the executing duration once; ProfileReset clears both
// before any later old-epoch drain.
fn initial_state(key: AggregateKey) -> totals::State;
```

**Must not change:** the start → evaluate → finish-adopted →
continuation order at every Module site; the pushed value; the push/pop
conditions; the four semantic frames' fields, construction order, root
tracing and abandonment (8/72/8/72; shared state 72; Session 2,128); the
existing 46 PhaseKeys, their names, indices or policies; the
token-first API; the Context/source restoration and the entry/application
rows; the synchronous compile/allocation/substitution tuples; the lease
drain and checkpoint bodies. **No** duplicated evaluator bodies, silent
drop helper, lazy initialization, heap substitution, frame padding, a
new Box protocol, a semantic-frame field, a `debug_assertions`-keyed
attribute, a default-path allocation/call, or a predicate that publishes
on a newly sampled flag instead of the original total token.

## Steps

1. **Materialize and shape stop.** Copy the ten production files and the
   observer, verify the ten copies byte-identical, compile offline/locked
   at jobs 8. Before any campaign: the two file groups, the
   `builtin_module` prologue 1,736, the reentry 1,080, one ordinary
   Module Callgrind row 692, one debug stack row 54,048, and one enabled
   coverage row. A moved file, a different reservation or a red stops
   before the expensive panels.
2. **Correctness and coverage.** Drive the ten completion cases: body
   pause; two distinct initializer pauses followed by body; repeated
   pauses in one initializer and body; named source; pure Module first
   entered after resume; nested Module/Block; initializer Abort; Function
   Return; Throw; and WithCleanup whose cleanup actually pauses. Assert
   concrete prompts and values, exact ten-field Module rows, the loop
   open once, initializer completed-call vectors `[0,1,2]` and
   `[0,0,1,2,2]`, zero aliases until terminal, initializer closed at
   each store, body closed at cleanup while loop/total stay open, nested
   ancestor counts, and the source-on/off key identity. Kill the seven
   mutants with positive controls in the same binary: fresh loop (3
   calls, not 1), late initializer, late body, early loop, early total,
   early scalar/source alias, lost restored source. Run cancel, discard,
   drop, reset, abandonment during a paused cleanup and reset-then-resume:
   zero open/leases/faults/retained bytes, closed gensyms, distinct
   reasons, idempotent double abandonment, new-epoch scalar zero and no
   source alias. Re-run the 18 + 17 source rows and the observer
   snapshots.
3. **P1507 and ledger.** Run `oracle::fixtures::logical_profile::logical_counter_port`
   with all five modes and the full required set; every P1507 row must
   pass with frozen subjects/plans. Remove only cells whose individual
   row passes, through the canonical ledger workflow; cite every removed
   cell with its passing row, and list the remaining 91,424 standing
   cells as unchanged. No bulk removal; no hand edit.
4. **Inventories and acceptance.** After the last source edit and before
   the first acceptance suite, replay the standing inventories. Stop for
   the template's identity/disposition/`UnknownHostPayload`/census
   decision conditions; an inventory-only red is an ordering artifact —
   replay the generator, never weaken the test. Then the focused runtime
   suite; both workspace configurations at 8 threads with the seven
   standing skips; the required-switch host suites (`lp03_phase_rows`
   alone, `lp03` + `lp04` + `lp05` in default and
   `--no-default-features`, the `rum-runtime` `lp657_host_root_control`,
   and the two `lp658` resets exact); the unchanged lease falsifier and
   lease-release test; `cargo fmt --check`.
5. **Performance campaign.** Build the base arm from
   `.temp/brief-674/src/base` (the pristine `410eb811` tree) and the
   final source independently at one common source path with release
   CGU=1, two independent builds per arm; measure both arms in two
   draws: ordinary Module, body-pause, three-pause, the resume frame
   bodies, Block, definition RHS, downvalue-pattern, named Function, raw
   Failure, release and debug stack at four heights × two repeats,
   prologues including the four resume symbols, layouts and the enabled
   RecordPath. Calibrate the F6/instrument on a known current row first.
6. **Timing.** The rum-vs-rum core-language microbench A/B on base and
   the exact final source, same protocol as the 673 stage, serially and
   alone, after every deterministic panel.

## Hard bars

- Ordinary Module own 692 per iteration, body-pause entry 705,
  three-pause entry/reentry 760/441, resume frame bodies 35/138; no
  owner above base, and the measured neighbors exactly 307 / 125 / 193 /
  49. The values are per-owner exactness on the same row, never a sum;
  savings in one owner cannot pay for growth in another.
- Release stack 18,896/14,064/18,304 B per level and debug
  54,048/41,904/49,632 B, four heights × two repeats × two draws, zero
  residual. Prologues 1,736 / 1,080 / 1,080 and the four resume symbols
  328/440/328/440, pushes included. Enabled RecordPath 232 B; the
  default path creates no RecordPath; frames 8/72/8/72 and Session
  2,128 B unchanged.
- The ten completion cases, the seven killed mutants, the reset and
  lifecycle cases, the 18 + 17 source rows and the observer projection
  are green with the exact rows and identities above. An unchanged guest
  value alone is not an oracle.
- The full P1507 set passes; every removed LP07 cell carries its
  individually passing row and the canonical generation; the other
  91,424 cells are unchanged; no cell is removed by owner count.
- Both workspace configurations pass at 8 threads with the seven
  standing skips, the lease test not skipped; the required-switch host
  suites run the skips; `cargo fmt --check` green; the standing
  inventories replayed and disclosed; public contracts, manifests,
  workloads and the frozen P1507 subjects/plans unchanged.
- No new `unsafe`, no network, no version-control command, no
  third-party comparison, no hand-edited derived cell.

## Verification

One cargo command at a time inside the dispatcher's 16G scope, plain
(never nested); jobs 8, test threads 8, offline/locked. Declare the total
run cap (2,000 bounded child commands) in `PROGRESS.md` before the first
run; retain the same-shape bounds (builds 880 s, Callgrind 900 s,
stack/prologue 120 s, isolated oracle 300 s) and derive any other bound
from a prior measured run of the same shape. A timeout is not data.
Performance captures run serially and alone on CPU 2, release, CGU=1,
both arms at one common source path, every `RUM_PROFILE_*` cleared except
the declared enabled controls. Raw output is saved before parsing; every
failed attempt is retained.

## Checkpointing

All scratch under `.temp/brief-675/`; keep `PROGRESS.md` with the exact
next action, the compiled source state, the cap and the receipts already
saved. Leave the tree compiling at each checkpoint. Stop after two
consecutive completed attempts that advance none of implementation,
coverage, mechanism, measurement or draft bars.

## Hard rules

- No version-control command of any kind; the director owns version
  control. No network, new `unsafe`, public API/feature/serde change,
  manifest/workload change, inventory weakening or hand-edited derived
  cell.
- Apply only the exact measured ten-file form, the stripped observer and
  the focused tests. A departure from that shape is a STOP for director
  review, as is a carrier, adopted-key, pushed-value or completion-order
  change.
- Every claim carries a citation the checker can resolve; every FLAGGED
  row carries a note that settles it. The director's gate owns
  acceptance.

## Deliverable

- The ten production files, the observer and focused tests, the oracle
  integration and the canonical ledger updates, with the complete
  raw-backed row set.
- `.temp/brief-675/report.json` in `rum-report/1`, every run and verdict
  line quoted, every attempt listed, the exact form diff, the before/
  after instruction pairs, stack fits/prologues, layout/storage rows,
  coverage and mutant receipts, loss removals and unchanged standing
  owners, inventory replay verdicts, skipped and unrun gates and any
  blocker.
- Run `python3 -I scripts/report.py check 675` and return its summary and
  every compliance verdict line.

## Amendment 1 — 2026-11-05: canonical formatting of the pinned form

The STOP was correct. The ten production files were copied byte-for-byte
from the measured tree and the stripped observer differs from `value-port`
by exactly the two permitted lines, but the repository's `cargo fmt
--check` fails on that exact form: 26 hunks across nine of the ten
allowed production files. The measured tree was hand-edited and never
normalized; formatting is not part of the measured semantics, and the
repository bar requires it. This amendment answers Q1.

- **Ruling: formatting-only normalization is authorized.** The pinned form
  is the measured tree under the repository's own canonical formatting —
  the toolchain pinned by `rust-toolchain.toml`, the repository's
  `rustfmt.toml`, applied as `cargo fmt` applies it. This supersedes the
  byte-for-byte-copy sentence in "Target form": after normalization the
  ten production files must be byte-identical to the canonical formatting
  of the same measured files, and the observer to the canonical formatting
  of the `value-port` observer with only the `MODULE_PORT` print and the
  `selected` helper removed. Nothing else changes; no hand formatting, no
  edit beyond what the canonical formatter produces.
- **Confined to permitted files.** Normalization may touch only files this
  brief allows you to modify. A formatting change to any other file is a
  STOP; the pre-STOP `cargo fmt --check` lists exactly the nine allowed
  files, so a wider result means the tree has diverged.
- **Step-1 shape stop re-runs on the normalized source**, before the
  correctness stage: the ten-file identity, the `builtin_module` prologue
  1,736 B, the reentry 1,080 B, one ordinary Module Callgrind row 692, one
  debug Module stack row 54,048 B with zero residual, and one enabled
  coverage row. Formatting must not move a measured row; a moved row is a
  STOP. The controls already passed on the unformatted bytes remain design
  evidence; this re-run certifies the exact final source.
- **Earlier replay output is stale, not final.** The nine generated
  inventory files copied back before the STOP were generated from the
  unformatted form, and normalization shifts line positions they record.
  After the last source edit — now including normalization — the canonical
  generators replay them per step 4; the ledger's generated
  producer-position string follows the same rule. No hand edit and no
  hand revert of a generated cell.
- **Unchanged.** All six steps, every hard bar, the full P1507 set and the
  individual per-cell proofs, the campaign on the exact final source, the
  run cap (2,000, 46 used and declared), and the report duties. The final
  `cargo fmt --check` must be green, including every later test and source
  edit. Record this amendment, the normalization receipts, the equality
  check and the re-run shape controls in the report.

## Amendment 2 — 2026-11-05: the exact phase table extends by the four Module phases

The STOP is correct. `profiling/tests/evaluator.rs` is outside the file
scope and was left unchanged; its `lp03_adopted_site_table_is_exact`
asserts the literal complete `PhaseKey` name set (46 names), and the
pinned form adds exactly the four Module phases, so the focused runtime
gate fails on that one assertion (1,351 unit and 22 doc tests pass; the
serial reproduction shows exactly the four additions and no removals).

- **Ruling.** Authorize the minimal exact extension of that expected set
  by exactly `module.eval.total`, `module.resume_initializers`,
  `module.initializer.evaluate` and `module.body.evaluate`, in the
  table's existing literal style. The assertion stays complete and
  exact: no filtering, no derivation from `PhaseKey::ALL`, no count
  weakening, no skip, and no other change to the file.
- **Disclosure duty (the class).** If another active assertion whose
  meaning is exactly "the complete set of `PhaseKey` names" fails solely
  because of the four added phases, the same literal extension is
  authorized; list every edited file and line in the report. An assertion
  whose meaning is anything narrower, or any other failure, is a STOP.
- **Re-run.** After the extension, run the serial reproduction and the
  focused runtime gate at 8 threads, then continue the acceptance ladder
  (both workspace configurations, the required-switch host suites, the
  lease, `cargo fmt --check`), and only then the campaign and the timing
  panel. The seven standing skips are unchanged.
- **Report.** Record the edit, the red-to-green reproduction and this
  amendment. The run cap and every bar are unchanged.

## Amendment 3 — 2026-11-06: remove the orphaned Module outcome helper

The gate's `cargo check --workspace --all-targets` reports one new
warning. `runtime_profile_outcome_for_expr_result` at
`crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60` is never used:
all eleven Module sites now classify through the token-first
adopted-boundary finisher, and this private helper has no other caller.
Its same-named helpers in `branching_loops.rs`, `nonlocal.rs` and
`definitions/application/mod.rs` remain in use and must not change.

- **Ruling.** Authorize exactly the deletion of that one private function
  from `scoping/lexical/mod.rs`. No `#[allow(dead_code)]`, no
  `#[cfg(test)]`, no rename, no other edit; every other file stays
  byte-identical. The deletion removes only unreachable source; it is not
  a form change.
- **Ledger invariant.** No loss cell, required row or binding is anchored
  in `scoping/lexical/mod.rs` (verified: zero cells), so
  `logical-profile-losses.json`, `logical-profile-required.tsv` and
  `logical-profile-bindings.tsv` must stay byte-identical. The deletion
  may shift positions the ownership/capture/macro inventories record;
  those are regenerated by the canonical replays, never hand-edited.
- **Re-verification.** After the deletion:
  1. Replay the standing inventories canonically and disclose every
     changed row.
  2. Rebuild the final arm independently twice and re-measure the two
     draws; the base arm is retained unchanged. Every instruction, stack,
     prologue and layout row must equal the retained final rows exactly;
     a moved row is a STOP. Compare against the same retained base rows.
  3. Re-run the ten completion cases, the seven mutants, the 18 + 17
     source rows, the full five-mode P1507, the focused suite, both
     workspace configurations with the seven standing skips, the
     required-switch host suites, the lease and `cargo fmt --check`.
  4. `cargo check --workspace --all-targets` must show no warning beyond
     the three pre-existing `register_test_*` test-only warnings.
  5. The timing panel runs last and is non-deciding, as before.
- **Unchanged.** The run cap, every hard bar, the exact-source rule and
  the report duties. The report records the deletion, the warning set,
  the replay verdicts and the remeasured rows; the retained brief-675
  evidence stays in place as history.
