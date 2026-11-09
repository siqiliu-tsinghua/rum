# Brief 717 — LP08 schema-v2 stage C: the acceptance campaign

**Measurement-only brief. The live tree is frozen at the accepted schema-v2
form: brief 709's design, brief 710 stage A (the epoch-owned synchronous
family index), brief 711 stage B (the `rum-profile schema-v2` renderer), and
the test-isolation and witness repairs of briefs 712-716. Nothing in the live
tree may change; all work is under `.temp/brief-717/`, and the tree, every
inventory and every test are the measurement input, byte-identical at the end
of the run.**

## Goal

Certify the schema-v2 publication end to end on the frozen source, in four
groups, all rum-vs-rum:

1. **Five-mode parity.** In each mode (disabled, lexical-only, timing-family,
   evaluation-cache, all-detail/source) a guest `RumProfileSnapshot` call's
   emitted `rum-profile schema-v2` line — decoded by an independent decoder,
   never the renderer's own helpers — must equal the native epoch collection:
   every key of the `N ∪ S ∪ A ∪ Z` union exactly once, every logical field
   equal. `RumProfileSnapshot` itself keeps its protected behaviour in every
   mode (`Null`); in the disabled mode the legacy diagnostics are
   byte-identical and no schema line is emitted and no collection work
   happens. A missing, duplicated or wrong key or field is a STOP.
2. **Cost.** Reproduce, on two independent fresh builds, the accepted
   source and stack rows (the Function source instruction rows and slopes,
   the stack identity and its absolute depth samples), and decide the
   disabled path with the corrected 703 criterion: the base-vs-final
   per-label microbench with the layout-control envelope, and the Callgrind
   instruction instrument wherever the timing lane cannot settle a label.
3. **Frozen replays.** The LP01 five-mode parents (four parent cells,
   88,795 records each, zero semantic differences, byte-identical ledger),
   LP02's host lifecycle and abandonment suites, the six switch-gated host
   targets, the ten accepted renderer mutants, and the whole workspace —
   all unchanged from their accepted values.
4. **A clean report.** `rum-report/1` with every run, attempt and verdict
   line, `report.py check 717` OK, and the live tree unchanged and
   uncommitted for my gate.

## Context pointers

- `docs/dev/profiling.md` — the approved contract: schema v2 grammar,
   telemetry/duration separation, lifecycle equations, the `N ∪ S ∪ A ∪ Z`
   union. Its words are the bar.
- `docs/dev/codex/brief-709-lp08-schema-v2-publication-design.md` — the
   accepted design; `.temp/brief-709/DESIGN.md` and `drafts/C-acceptance-
   campaign.md` are the director's review material for this stage.
- `docs/dev/codex/brief-710-lp08-synchronous-family-collection.md` and
   `brief-711-lp08-snapshot-renderer.md` — the accepted implementation and
   its gate receipts (the six seam per-call rows, the Function rows, the
   ten mutants, the five-mode samples).
- `.temp/brief-708/` — the final-source campaign scripts and evidence
   (`campaign`, `build-panel.json`, `base-head-verification.json`,
   `src/base`, `src/layout`, the microbench raw runs and the corrected 703
   criterion). Read/copy only; adapt by function, never by line.
- `.temp/brief-711/` — the LP01 five-mode driver (`run_cont.py`,
   `lp01_cont.py`, `acceptance_cont.py`), the ten mutants
   (`mutants_cont.py`) and their red/green receipts. Read/copy only.
- `.temp/brief-712/`, `.temp/brief-713/`, `.temp/brief-716/` — the
   isolation and witness repairs this source carries; their standing bars
   (1,413 lib tests, the 1,420-name set, the collector's permanent 4096-byte
   foreign refund) are not reopened.
- `.temp/lp01-base/`, `.temp/director/gate-state/` — retained instruments.
- `scripts/probe-build.py`, `scripts/core-language-microbench.sh`,
   `scripts/run-bounded.py`, `AGENTS.md`, `TEMPLATE.md`.

## Scope

- **No live file changes.** The working tree, every test and every
  inventory are the measurement input, byte-identical at the end.
- All scratch under `.temp/brief-717/`. Retained roots are read/copy only;
  never execute a retained file in place and never modify a retained tree.
- The final arm is the live tree snapshotted under
  `.temp/brief-717/src/final/` (exclude `.git/`, `target/`, `.temp/`);
  verify the snapshot against the live tree file by file before any build.
- The base arm is the pre-LP08 snapshot `.temp/brief-708/src/base/`,
  verified against the recorded `base-head-verification.json` runtime rows;
  the layout control is `.temp/brief-708/src/layout/`. A missing or
  mismatched snapshot is a STOP — never rebuild the base from guessed
  sources.
- Do not run `scripts/director-gate.py` or `scripts/memory-scope.sh` (the
  dispatcher provides the 16G scope). No version-control command of any
  kind.
- No `unsafe`, public-API, serde, feature, manifest, dependency, workload
  or oracle change. If a defect is found, report it and STOP: stage C
  certifies a frozen implementation and is not permission to repair it.

## Stages

**A — Assemble and freeze-verify.** Snapshot the live tree; verify the
snapshot against it; verify the base and layout arms against their recorded
receipts. Record source receipts (files and hashes of the four snapshots;
the live tree itself is not re-hashed — the director owns git identity).
Confirm the standing-inventory receipts are current on this source: the
retention contract fingerprints, both certificate cells, the ownership
discovery, the typed-slot census and the rejection census must reproduce
their accepted rows with zero changed, added or removed rows. A moved row is
a STOP with the panel; a stale derived cell is reported, not fixed here.

**B — Five-mode parity.** For each mode in order — disabled, lexical-only,
timing-family, evaluation-cache, all-detail/source — isolate the mode in its
own process (every `RUM_PROFILE_*` variable removed first; the 711 mode
matrix: lexical sets `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`; timing adds
`RUM_PROFILE_BUILTIN_TIMING`, `RUM_PROFILE_RULE_RHS_TIMING` and
`RUM_PROFILE_DEFINITION_RULE_TIMING`; cache adds
`RUM_PROFILE_EVALUATION_CACHE`; all adds lexical `DETAIL`, `SHAPES`,
`BINDINGS`, `SOURCES`, `SOURCE_FINGERPRINTS`, `PROVENANCE`; the incremental
cache print forced 0). Capture the guest emission and the native epoch
collection before any fresh request changes the table, then compare:

- every key of the union exactly once, in stable encoded order across two
  processes and both interner orders;
- every logical field equal field for field against the native collection
  (`Session::logical_profile_test_rows()`, the `Epoch::rows()` fold that
  includes the synchronous families); durations within their before/after
  bounds, never a parity input for totals, self totals, averages or segment
  counts;
- the snapshot's own entered invocation open while it runs, in every mode
  where the family exists; where it cannot exist, the measured row and the
  named ruling, never a guess;
- disabled: `RumProfileSnapshot` returns `Null`, the legacy diagnostics and
  bytes unchanged against the base, no schema line, no collection work;
- the lifecycle cases: mid-evaluation, more than 64 distinct keys, repeated
  equal keys, nested reset, old-epoch markers, a session resumed on another
  thread, drop-before-summary, diagnostic-cap exhaustion, sticky initial
  failure;
- decoder falsifiers, calibrated before the union is accepted: a decoder
  that drops one key, duplicates one key or changes one field must be
  rejected on the same captures. They are instrument calibrations, not
  source-mutant credit.

The parity bar is zero semantic differences. The in-tree publication suite
(`cargo test -p rum-runtime --lib publication::`) and the host schema test
(`cargo test -p rum-host --test schema_v2_snapshot`) run at 8 and 1 threads
as the frozen gates; the independent comparison is the evidence.

**C — Source and stack.** Build the final arm twice, independently, at one
canonical absolute path through `scripts/probe-build.py` with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, `-j8`, offline/locked, one build per
arm and kind; a forced rebuild must reproduce the final arm's SHA exactly.
On both builds reproduce the standing bars of the accepted line:

- F6 queued source control: 49 instructions.
- Resumed Function source rows: `try` 109, `finish` 56, `resume` 20
  instructions per iteration; call slopes 1; every own and incoming vector
  equal.
- Stack: 18,304 bytes per level, intercept 12,392, residual zero; absolute
  samples at depths 5/10/15/20 equal to 103,912 / 195,432 / 286,952 /
  378,472, twice each.
- Both builds must give equal rows. If the harness entry changes absolute
  offsets, STOP for a separately calibrated ruling; never loosen a bar.

**D — Disabled-path cost.** With profiling off, build the base, final and
layout release rum-cli binaries at one path with one codegen unit and run the
established rum-vs-rum core-language protocol through
`scripts/core-language-microbench.sh`: in-kernel `AbsoluteTiming`, no
competing CPU-heavy work, four passes per arm in the interleaved brief-637
order, the 120-label panel and the layout control. Apply the corrected 703
criterion: the layout control calibrates the panel-wide noise floor; a
positive movement is a STOP only if it exceeds that envelope **and**
contradicts the instruction instrument; inside-envelope movements with equal
executed instructions are recorded as noise. For any label the instruction
instrument must settle, run the 703-style probe on the three arms' binaries:
Callgrind Ir slopes at 50/100, four draws per cell, profiling environment
cleared, stdin closed, per-function self-Ir resolved and compared.
Reproduce the stage-A per-call rows beside them: 13→3, 23→18, 18/20→14/18
at identical call counts, and every candidate whole-workload draw below
every base draw.

**E — Frozen replays.** On the frozen tree, after Stage A's inventory
receipts are confirmed:

- LP01 five-mode parents: four cells (default and `--no-default-features`,
  8 and 1 threads), five modes each, 88,795 records per parent,
  `semantic_differences=0`, ledger byte-identical, ordinary instrumentation
  faults 0. The parent comparison is byte equality against the accepted
  parent receipts; no normalization.
- LP02 and the host contract: the six named host targets
  `logical_profile_lifecycle`, `logical_profile_abandonment`,
  `lp03_phase_rows`, `lp04_definition_rows`, `lp05_assignment_rows` and
  `schema_v2_snapshot` green at 8 and 1 threads, with
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` for the switch-gated ones; the two
  standing root-context reds remain the only excluded names and must fail
  exactly as recorded when run with the switch.
- The ten accepted renderer/source mutants from
  `.temp/brief-711/mutants_cont.py`: each reddens its named observer at 8
  and 1, and every restoration returns to the same binary SHA. A mutant
  that no longer reddens is a STOP.
- The frozen runtime suite once at 8 and once at 1: `1413 passed; 0 failed;
  5 ignored; 2 filtered out` with the two standing skips.
- `cargo fmt --check` clean.
- The whole workspace at 8 threads: `cargo test --workspace --offline
  --locked -j8 --no-fail-fast -- --test-threads=8` with exactly the seven
  standing skips (`lp655_root_context_falsifier`,
  `lp656_public_root_context_falsifier`,
  `lp03_adopted_phases_are_exact_in_completed_and_resumed_runs`,
  `lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs`,
  `lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs`,
  `lp658_host_eval_reset`, `lp658_host_resume_reset`); the accepted shape is
  **4,212 passed / 0 failed / 61 ignored over 63 binaries**, and a new
  warning is a STOP. The native ZeroMQ loopback denial inside the worker
  sandbox is an environment boundary to report with its serial
  reproduction, not a skip; the director runs the authoritative workspace.

**F — Report.** `.temp/brief-717/report.json` in `rum-report/1`: every run,
every attempt, every verdict line and every excluded row; run
`python3 -I scripts/report.py check 717` and return its summary; report the
final live-tree state, which must equal the briefing state.

## Bars and exclusions

Standing exclusions, unchanged: the two documented root-context reds
(`lp655_root_context_falsifier`, `lp656_public_root_context_falsifier`) and
the five switch-gated host names in the plain workspace run; the
`default_builtin_dispatch_spec_for_symbol` process-hash variation, never a
cost offset; the brief-620 association label exclusion; the 703 residual
rows as within the calibrated noise floor. The LP01 ledger and the
schema-v2 grammar are frozen: no re-baseline, no widened or narrowed
promotion set, no reordered or renamed key, no normalization that hides an
absent key.

A new warning, a new `UnknownHostPayload`, a changed row identity or
disposition, a disappeared row, a weakened assertion, a default-path cost, a
live-tree change, or a measured row that differs from its bar without
attribution is a STOP with the exact panel. A loose check, a re-baselined
gold or an inherited number is never the answer.

## Host discipline

All scratch under `.temp/brief-717/`; never `/tmp`. The dispatcher already
runs you in the 16G memory scope. One cargo command at a time; builds
`CARGO_BUILD_JOBS=8`; tests `RUST_TEST_THREADS=8 --no-fail-fast`; a red is
reproduced at `--test-threads=1` before it is called a defect. Timing,
Callgrind and stack captures run serially and alone; no competing CPU-heavy
work. Probe builds through `scripts/probe-build.py`; Callgrind relative
offsets are decimal bytes and `*` repeats the previous address. Persist raw
output before parsing; calibrate every instrument against a known current
row before believing it. Inherited bounds: builds ≤900 s, each LP01 parent
≤1500 s, each focused acceptance suite ≤1200 s, the workspace ≤2400 s,
Callgrind ≤900 s, a stack capture ≤120 s, `report.py check` ≤120 s.

## Checkpointing

Save each stage's raw receipts before reducing them; every stage is
re-entry-safe and a consumed baseline receipt is never rerun. Write the
exact next action and status after each stage. A red stops the campaign
where it is: save the log, reproduce the same binary once at
`--test-threads=1`, report and STOP — never restart, relabel or delete a
failed attempt. Independent required builds use the probe cache with one
`--force` sha256 spot-check per round.

## Deliverable

- The four snapshot receipts (final, base, layout, identity or probe arms),
  the six build receipts and the forced SHA comparison.
- The raw five-mode captures, the independent decoder and its falsifier
  receipts, the per-mode key/field tables and every raw line.
- The two-build source and stack receipts with their absolute samples.
- The base/final `timings.tsv` comparison, the layout-control envelope, the
  Callgrind probe on any label it must settle and the six seam per-call
  rows.
- The LP01 four-parent receipts, the LP02/host suite receipts, the ten
  mutant red/green receipts, the focused runtime suite and the workspace
  receipt.
- The `rum-report/1` report and the `report.py check 717` summary.
- The working tree unchanged and uncommitted for my gate.

## Hard rules

`AGENTS.md` and `TEMPLATE.md`; the testing policy is rum-only (this campaign
compares rum against itself); no git of any kind, no live edit, no network,
no third-party engine, no new `unsafe`; a red is reproduced at
`--test-threads=1` before it is called a defect; all output under `.temp/`;
the report is `rum-report/1` JSON with copied verdicts and the
command/log/exit/count table. A campaign green does not authorize a commit:
the director owns the authoritative gate.

## Stage-B STOP ruling and continuation (director, 2026-11-09)

Run 1 stopped in Stage B before any acceptance test ran: the worker's
warning classifier fired on Cargo's summary line, its calibration then
failed, and a follow-up launch exited 2 on a missing file. **The STOP is
accepted as an instrument failure, not a product signal. Stage A is
accepted and is not rerun; the campaign resumes at Stage B.**

### Stage A accepted (my own verification)

- The live tree is byte-identical to `.temp/brief-717/evidence/final-files.json`:
  I hashed all 2,708 files myself (excluding `.git/`, `target/`, `.temp/`),
  2,708 entries, 0 missing, 0 extra, 0 mismatched; `git status --porcelain`
  empty at HEAD `80c85695ea9808ce3963cadba015d2efb805f8a8`.
- The base and layout arms are byte-identical to the retained
  `.temp/brief-708/src/{base,layout}` (2,193 files each, 0
  missing/extra/differing). The base provenance receipt's 603 rows
  (587 runtime, 16 prior scope) reconcile: 601 present and hash-equal, and
  the two recorded-absent rows (`logical/collector.rs`,
  `logical/completion.rs`) are absent from the snapshot exactly as
  recorded. The run-1 assembly KeyError was that absent-row indexing
  error; the finish driver's verification stands.
- The identity arm is byte-identical to the retained
  `.temp/brief-708/assembled/identity/downstream` (2,675 files), and its
  canonical tree hash `357420d5dc418ee2b919bf4878e9a2aaede086465c7b1ddc6f30fd6a3a607f46`
  equals the 708 probe build receipt.
- The six standing inventories reproduce their accepted rows:
  fingerprints 68; both certificate cells `1/0/0` with
  `PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0
  PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0`; ownership 61 / 79 / 8,815 (the
  brief-716-accepted counts); typed slots at the outer harness
  `10 passed; 0 failed; 1 ignored` — the two nested `FAILED` lines in
  `logs/a-typed-slots.log` are the designated forced-red scratch runs of
  `removing_a_frame_in_a_scratch_copy_is_red` and
  `removing_a_test_guard_in_a_scratch_copy_is_red`, and the report's
  summed `(10, 2, 1)` is that honest aggregation; rejection census
  1,220 = RET001 14 / RET002 1,168 / RET003 38.
- `python3 -I scripts/report.py check 717`: OK, 129 citations.

### The Stage-B STOP

The build `cargo test -p rum-runtime --lib --no-run` (offline/locked, -j8)
succeeded in 22.36 s and emitted exactly the three standing `test_builtins`
unused-function warnings. The audit script counted the summary line
`` `rum-runtime` (lib test) generated 3 warnings `` as a diagnostic, so its
"no new warnings" assertion fired on that string; its calibration then
required the entire eight-message reference (a feature-enabled probe build
with five unrelated warnings) to equal the three observed messages and
failed; and the following launch named a file that did not exist (exit 2).
No acceptance test ran, so no implementation defect is claimed or
available. The failed attempts are retained.

### Warning-audit rules (the bar stands; the instrument is corrected)

The bar "a new warning is a STOP" is unchanged. The audit compares the
build it just made against that configuration's accepted baseline — never
a cross-build superset:

- `cargo test -p rum-runtime --lib` (default lib): 0 warnings.
- `cargo test -p rum-runtime --lib --no-run` (lib test): exactly the three
  standing `test_builtins` warnings (`register_test_if_builtin`,
  `register_test_return_builtin`, `register_test_vm_template_builtins`),
  the accepted baseline since brief 713.
- Any other configuration: its own last accepted warning set.

Cargo's `` generated N warnings `` summary line is not a diagnostic. The
classifier is calibrated before it is believed: over the retained
`logs/b-build-frozen-runtime.log` it must find the three standing
diagnostics and zero new; over a copy with one synthetic `warning:` line
injected it must find exactly one new. A classifier that fails either
calibration is discarded; its red is not a campaign red and no dependent
step is launched on it.

### Continuation scope

Resume at Stage B. Stage A's snapshots, manifests and receipts are
consumed and are not repeated. The Stage B build and its binary are
reused (the snapshot's target directory is warm); any fresh build still
follows the brief's one-build-per-arm rule. The base and layout arms used
from here are the director-verified copies `.temp/brief-717/src/{base,layout}`
(byte-identical to the retained 708 arms, 2,193 files each) bound by the
retained 708 root receipts (`base-head-verification.json`,
`source-identities.json`, `build-panel.json`); a 708 `src/` or `assembled/`
subtree pruned by disk hygiene is not a missing snapshot. Stages B-F, the
bars, the exclusions, the host discipline and the deliverable are
otherwise unchanged. The continuation's report extends
`.temp/brief-717/report.json` with every new run, attempt and verdict line
while keeping run 1's failed attempts; it does not overwrite them. Every
script launched must exist and be readable first.

## Stage-C STOP ruling and continuation (director, 2026-11-09)

Run 2 stopped at Stage C before build 2 and before the stack rows: the
resumed Function source rows read try 106, finish 53, resume 20 per
iteration against the accepted 109/56/20, while F6 49 and the queued rows
match exactly. The STOP is accepted as well taken — the brief forbids
loosening a bar. The panel is attributed in this ruling, and the frozen
source's test-build row is re-frozen here. The pre-713 reading 109/56/20
stays recorded as the historical measurement of the pre-713 test build;
nothing is deleted or re-labelled.

### Attribution (my own measurement)

- The bar's source is the same instrument the run used:
  `.temp/brief-710/continuation-c/evidence/costs.json` and
  `.temp/brief-711/continuation/evidence/costs.json` — the byte-identical
  `brief677_probe.rs` fixture, the same canonical builder
  (`cargo test -p rum-runtime --lib --no-run --release`, CGU=1, stable
  path) and the same Callgrind invocation. Both recorded 109/56/20 in two
  builds and both arms.
- The frozen test binary reads 106/53/20. The same-binary serial captures
  reproduce each own vector exactly, the incoming calls stay 50/100 with
  slope 1, and `resume` stays 20 — the call structure is unchanged and
  only the per-call own count moved, downward.
- Mechanism: brief 713's exact-cfg admission in
  `crates/rum-runtime/src/runtime/profiling/config.rs:4-21` made
  `lexical_profile_enabled()` take a lease admission under
  `cfg(any(test, feature = "test-builtin-memory-growth",
  "test-continuation-slots"))`. In the test build the body grows and LLVM
  stops inlining it into `finish_function_application` and
  `try_apply_function`. Disassembly: the pre-713 candidate test binary
  `65061efa…` inlines the `LEXICAL_PROFILE_ENABLED` load and the
  `initialize_lexical_profile_enabled` slow call inside both functions;
  the frozen test binary `9f6d33c3…` emits `call lexical_profile_enabled`
  in both. The caller's own count drops by three per iteration and the
  callee's work moves to `lexical_profile_enabled`'s own frame. The seam
  still returns false in the disabled run, so the observed behaviour is
  unchanged.
- The production path did not move. The CLI is built without test cfg,
  where the cfg block is erased. In the frozen CLI `32df5b02…`, the
  production `finish_function_application` (234 instructions) and
  `try_apply_function` (594 instructions) are instruction-for-instruction
  identical, after normalizing addresses and call targets, to the 711
  candidate CLI; the queued F6 rows reproduce 57/49/20/71 with incoming
  501/1001 exactly. Nothing on the production lane moved.
- The movement is therefore a test-build inlining reading, not a
  source-path regression, and it is negative. The bar it replaces exists
  to catch default-path growth; that duty is carried by the exact CLI rows
  and the unchanged release object code, both of which are green.

### The re-frozen rows (frozen source, test build)

- Resumed Function: `try` 106, `finish` 53, `resume` 20 instructions per
  iteration; own vectors at N=50/100 exactly [5300,10600], [2650,5300],
  [1000,2000], incoming [50,100], call slope 1.
- F6 queued: 49 unchanged (CLI own rows 57/49/20/71 exact).
- Stack: 18,304 bytes per level, intercept 12,392, residual 0, absolute
  samples 103,912 / 195,432 / 286,952 / 378,472 twice each — unchanged and
  independent of the seam inlining.
- Both independent builds must give identical rows; a difference between
  builds is a STOP. No other bar moves. The replacement is not a
  relaxation: it is calibrated to the frozen source's build kind and
  bound to the unchanged production lane.

### Continuation scope

Resume at Stage C with build 2: build the final runtime and CLI a second
time under the same rule (forced rebuild equal SHA), reproduce the
re-frozen Function rows and the stack samples on both builds, then run
Stages D, E and F unchanged. Do not rerun Stage A or B; run 2 passed
Stage B (five modes, 20 processes, 40 guest captures, 96 rejected decoder
falsifiers, publication 14/14 at 8/1, host schema 4/0/0 at 8/1, 30 exact
base-byte cases each) and its receipts stand. The report extends
`.temp/brief-717/report.json` again, keeping every run-1 and run-2 entry;
the failed 109/56 comparison stays in it as the historical reading.

## Stage-C stack STOP ruling and continuation (director, 2026-11-09)

Run 3 completed Stage C's source rows on build 2 — queued F6 57/49/20/71
with incoming 501/1001, resumed 106/53/20 with incoming 50/100, equal to
build 1's rows, call slopes 1 — and then STOPed at the first absolute stack
sample: depth 5 read 103,416 against the unchanged 103,912, reproduced on
the same binary at `--test-threads=1`. **The STOP is well taken:** Stage C
says a changed absolute offset stops for a separately calibrated ruling,
and the worker neither rebaselined nor resumed. The measurement below is
that ruling. **Erratum (mine):** the previous appendix said the stack rows
were "independent of the seam inlining". That sentence is refuted by this
run — the stack probe runs in the same test-cfg build — and is withdrawn.

### Re-calibration (my own measurement, instrument calibrated first)

The standing bar is a lib-test-binary measurement. Against a known answer
first: the pre-713 test binary
`.temp/brief-711/continuation/bin/cost-runtime-candidate-1` (`65061efa…`,
the binary that produced the accepted 711 row) reproduces the standing
samples exactly in today's environment — 103,912 / 195,432 / 286,952 /
378,472, two repeats each — so the instrument and the host agree with the
bar and the movement is in the frozen binary, not the environment.

The frozen runtime test binary `9f6d33c3…` gives, at the same protocol
(`BRIEF611_MARKER=1 BRIEF611_SHAPE=function-body`, `RUST_TEST_THREADS=1`,
`taskset -c 2`, serial, two repeats, four depths):

| depth | 5 | 10 | 15 | 20 |
|---|---|---|---|---|
| observed | 103,416 | 194,456 | 285,496 | 376,536 |

exactly on the line **12,376 + 18,208 × depth**, residual 0, identical
across all four depths and both repeats. The line moves in both terms:
slope 18,304 → 18,208 (−96 per level) and intercept 12,392 → 12,376 (−16).

### Attribution (the same class as the Function-row ruling)

The 137-frame recursion chain is symbol-for-symbol identical in both test
binaries (same backtrace at the marker, same sequence). In the test build
the flag path is now outlined at every measured seam: `try_apply_down_values`
13 inline `LEXICAL_PROFILE_ENABLED` references and 11 inlined
`initialize_lexical_profile_enabled` calls become 0 and 15 calls to
`lexical_profile_enabled`; `resume_call_after_head` 7 → 0 and 7;
`evaluate_atom_fast_path` 7 → 0 and 7; `evaluate_inner_unstable` 29 → 0 and
29. That is brief 713's exact-cfg admission in `profiling/config.rs:4-21`,
the cause already ruled on for the Function rows, and these functions'
frame sizes move with the outlined bodies.

The production lane does not move. In the release CLI every function of
that chain — including every function whose test-build frame changed — has
the **same prologue frame size** in the 711 candidate CLI and in the frozen
CLI `32df5b02…`: `try_apply_down_values` 0x2b8, `resume_call_after_head`
0x1f8, `evaluate_atom_fast_path` 0x130, `evaluate_inner_unstable` 0x4b8,
`try_dispatch_builtin_call` unchanged, all four
`try_apply_down_values_from_symbol` instantiations unchanged; the CLI
per-block frame sum is 27,640 on both sides, delta 0. Production stack per
level is therefore unchanged, and the −96/level is a test-build reading of
the same instrument — negative, and not a product movement.

### The re-frozen stack row (frozen source, test build)

**18,208 bytes per level, intercept 12,376, residual zero; absolute
samples at depths 5/10/15/20 equal to 103,416 / 194,456 / 285,496 /
376,536, twice each, on both builds.** Both builds must give equal rows,
and any sample off this line is a STOP. The old row (18,304 / 12,392 /
103,912 … 378,472) stays in the report as the pre-713 historical record; it
is not produced by this source and must not be re-asserted. The
replacement is not a relaxation: the re-frozen line is stricter than the
standing one at every depth.

### Continuation scope

Resume at Stage C with the stack panel on both reused build pairs
(`9f6d33c3…` runtime twice, four depths, two repeats each), then Stages D,
E and F unchanged. Stages A and B stand and are not rerun; the Function and
F6 rows on both builds are accepted (build 1 measured in run 2, build 2 in
run 3, binaries SHA-equal, rows equal, call slopes 1). The report extends
`.temp/brief-717/report.json` again, keeping every run-1, run-2 and run-3
entry; the failed 103,416-against-103,912 comparison stays in it as the
historical reading. Dispatch the run-3 continuation forked from the 717
session `01a1c105-103e-7ad2-a04f-13b452fd0e9f`.

## Stage-D whole-workload STOP ruling and continuation (director, 2026-11-10)

Run 4 completed Stage C in full (16 stack samples, both builds, the
re-frozen line, residual 0) and Stage D's timing collection (120 labels,
three arms, four interleaved passes, layout envelope median 0.61% / p90
2.58% / max 9.42%) and the first ledger draw of both arms: all six seam
rows exact at identical call counts with cold helpers at zero self-Ir and
zero calls. It then STOPed on the whole-workload row: base 16,458,580,255;
final primary 16,464,621,146 (+6,040,891); final same-binary serial
16,459,014,281 (+434,026). Under the brief's literal clause — "every
candidate whole-workload draw below every base draw" — that pair fails.

**The STOP is well taken under the clause as written. The clause is my
instrument error and is withdrawn as a gate.** The evidence is measured,
not argued.

### The row cannot decide at this scale (my own interleaved series)

Eight fresh Callgrind draws on the same two binaries and the same saved
workload, interleaved base/final, profiling environment cleared, serial,
`taskset -c 2`, the same `--dump-instr --collect-jumps` flags:

| arm | draw totals |
|---|---|
| base | 16,471,180,283 / 16,464,422,699 / 16,483,577,484 / 16,460,054,782 |
| final | 16,460,369,464 / 16,460,206,646 / 16,458,536,769 / 16,466,720,760 |

- Same-binary spread: base 23,522,702 instructions (0.143%); final
  8,183,991. With the worker's own draws the base spans
  16,458,580,255–16,483,577,484 (24,997,229, 0.152%) and the final spans
  16,458,536,769–16,466,720,760 (8,183,991); the worker's two runs of the
  identical final binary already differ by 5,606,865.
- The distributions overlap: max final 16,466,720,760 exceeds min base
  16,460,054,782. The stopped pair was the worst pairing of the two
  distributions (base at its minimum, final at its maximum).
- The medians sit in the directed direction: base 16,464,422,699 against
  final 16,460,288,055, the final lower by 4,134,644 — consistent with the
  deterministic seam saving below and inside the overlap, so it is
  recorded, not claimed as a separation.
- The clause it replaces never had this resolving power: in stage A the
  recorded separation was 2,056,129 (candidate max 16,456,887,219 against
  base min 16,458,943,348), smaller than the base's own stage-A spread of
  4,533,874 (16,463,477,222 − 16,458,943,348). The stage-A sentence is
  hereby recorded as not established by that row.
- Mechanism (already named in 710 Ruling 8): the workload's own
  `rumMicroEmit` prints `ToString[seconds]`, so the measured duration's
  digit count feeds back into the formatted output's instruction count,
  on top of the process-level hash-seed nondeterminism. A total that
  contains a timing-dependent printout cannot gate at the 0.04% scale.

### The deterministic lanes, and the re-frozen Stage-D rule

The default-path verdict is carried by the deterministic evidence, all of
which is green in run 4:

- The six seam rows at identical call counts: `record_timing` 13→3,
  `record_profile_rule_rhs_timing` 23→18, the four dispatch wrappers
  18→14 / 20→18 / 20→18 / 18→14 (calls 103,920 / 393,040 / 91,440 /
  49,008 / 21,428 / 7,003), with the cold helpers at zero self-Ir and zero
  calls. At these counts the final arm executes **3,539,044 fewer**
  instructions in the six seams.
- The corrected 703 label probes on every residual label, still pending:
  `association-part-key-read-width32000`,
  `association-set-key-incremental-small`,
  `deletecases-leaf-delete-small`,
  `downvalue-guarded-decline-fallback`, `replace-leaf-rebuild-small`,
  `stringreplace-single-literal-unguarded`, `symbol-ownvalue-read`
  (`association-part-key-read-width4000` stays excluded by the standing
  brief-620 calibration failure). New executed work in the changed code on
  any label is a STOP; inside-envelope movement with equal executed
  instructions is recorded as noise; equal instructions outside the
  envelope are recorded as timing movement without instruction support.

**Re-frozen rule:** the ledger completes its four interleaved draws per arm
and the report records both ranges, the medians and the deterministic
seam delta. The whole-workload row is recorded as noise-limited, not
gated; it may STOP only if the final range lies entirely above the base
range, or if any seam row deviates, or if the cold helpers are called.

### Continuation scope

Resume at Stage D: complete the ledger's draws 2–4 per arm under the
re-frozen rule, run the seven residual label probes (24 profiles each,
50/100 iterations, four draws per cell, three arms), reduce them under the
corrected 703 criterion, then Stages E and F unchanged. Stages A, B and C
stand and are not rerun; the run-4 timing panel, the layout envelope and
draw 1's seam receipts are consumed and reused. The report keeps every
earlier entry, including the stopped pair as the historical reading.
Dispatch the run-4 continuation forked from the 717 session
`01a1c117-0ba8-7773-b93d-afc8d124647c`.

## Stage-D label STOP ruling and continuation (director, 2026-11-10)

Run 5 completed the seam ledger under the re-frozen rule and ran all seven
label probes, then STOPed on the first red: on
`association-part-key-read-width32000` and
`downvalue-guarded-decline-fallback`, `record_current_source_timing` executes
22 instructions per call on the base arm, 22 on the layout arm and 24 on the
final arm at identical call counts; four draws per cell and the same-binary
serial reproductions all read 24. **The STOP is well taken.** The clause it
enforces — new executed work in the changed code on any label is a STOP — is
not a noise rule, and the row is deterministic.

**Completed before the STOP, verified by me.** Four interleaved ledger draws
per arm: base 16,458,580,255 / 16,460,786,361 / 16,468,383,817 /
16,468,713,969; final 16,462,205,579 / 16,464,621,146 / 16,465,418,160 /
16,465,490,586. The ranges overlap and the final range does not lie entirely
above the base range, so the whole-workload row is recorded, not gated. Every
draw reproduces the six accepted seam rows at identical call counts (13→3,
23→18, 18→14, 20→18, 20→18, 18→14) with the cold helpers at zero self-Ir and
zero calls; the deterministic seam delta is −3,539,044 instructions. All
seven labels carry 24 profiles each (168 primary profiles) plus four
same-binary serial reproductions.

**My reproduction of the red** (`.temp/director/probe-717e/`): fresh
Callgrind captures on the three arm binaries at 50/100 iterations, two draws
per cell. `record_current_source_timing` reads 1,100/2,200 Ir at 50/100 calls
on base and layout (22 per call) and 1,200/2,400 on final (24 per call), with
50/100 incoming calls from `evaluate_definition_application_rhs` and no
outgoing calls; the control `record_timing` reads 13 and 3. In the
whole-workload ledger profiles the seam executes 109,077 calls in every draw:
2,399,694 Ir on base (22 per call) against 2,617,848 on final (24 per call),
**+218,154 instructions**.

**Mechanism, measured.** Disassembly of the seam in both binaries: base
performs the two callee-saved argument saves after the flag test (on the
cold-load branch only); final hoists `movq %rsi,%rbx` and `movq %rdi,%r14`
above the flag test, so the disabled return executes 24 instead of 22
instructions (entry prologue, two flag tests, epilogue). The final body is
1,321 bytes against 985. The cause is the stage-A side-index block in
`record_symbol_timing_profile_counter` (`source.rs:383-403`), whose only
caller is this seam and which the compiler inlines into it: a calibration
build with that block removed (`77fb86e9…`, same probe, four draws) reads 22
per call again, with the caller at 91 per call exactly as base.

**Class.** The deterministic whole-workload scan over every executed function
with an unchanged call count finds no other growth row in `rum_core` or
`rum_runtime`; the six accepted reductions and the two accepted once-called
intercepts (`drive_input_request` 54→55, `Session::drop` 34→33 — the 708
fixed-intercept class) reproduce. The two positive profiling rows in
`association-set-key-incremental-small`'s reduction are the 713 admission
renames (`current_evaluation_profile_source_symbol` → `…_admitted`, 3,636;
`lexical_profile_enabled` → `…_admitted`, 2,200), equal in magnitude and zero
net.

**Ruling.** A diagnostic seam that executes more instructions per call with
profiling off is a default-path cost; the campaign does not record it as
acceptable and does not waive the clause. The seam is repaired to the shape
the line already uses: the disabled path an early return, the enabled body
behind `#[cold] #[inline(never)]`, the seam kept out of line so its per-call
row stays measurable, and the test-cfg admission and admitted flag read
unchanged in order. My calibration of that shape — start-first early return,
cold enabled body, pinned outer — reads **3 instructions per call** on both
labels at 50/100 with two draws per cell, the helper at zero calls and the
caller unchanged at 91 and 125 per call; the minimal cold outline without the
start-first fast path reads 18. The bar is at or below 4 per call — the
`record_timing` class — not merely at or below the base.

**Consequence and continuation.** The repair changes the source, so the
certification restarts: no Stage A/B/C receipt is carried across it. The
campaign certifies a frozen tree, and a new tree is certified whole.
Sequence: (1) brief 718 implements the repair and is gated by me; (2) brief
717 restarts at Stage A on the repaired source, runs B, C, D and E in order,
and reports at F. The run-5 evidence stands as the historical reading: the
ledger, the timing panel, the 168 profiles and the two instrument failures
(the reducer's receipt collision, recovered without a rerun; the attribution
selector's first symbol form) are retained and are not rerun. Stage E did not
run.

**Restart note (director, 2026-11-10).** Brief 718 is accepted and its
repair is the committed source of this restart. Stage A's standing-inventory
baseline is that commit's replayed rows, not the pre-repair set: the
ownership inventory carries five new `UnknownConstruct` rows for the
repair's cfg attributes with zero removed or changed identities, the 68
retention contract fingerprints and the two promoted certificate cells were
re-emitted canonically, and the typed-slot, LP01, ownership and rejection
census replays are green on that source. A zero-delta replay against those
committed rows is the Stage-A bar; a further moved row is still a STOP.

## Restart Stage-D label STOP ruling and continuation (director, 2026-11-10)

The restart STOPed at the first red of the 23-label instruction panel, before
any reduction: on `downvalue-exact-miss-repeat` the base probe — the exact
benchmark body `Do[fExact[2], {5000 * rumMicroScale}]` wrapped in the
protocol's outer `Do[..., {i, 1, 50}]` — requests a static nested iteration
product of 250,000 against the session's iteration-work limit of 100,000 and
exits 1 at `evaluation exceeded iteration work limit (100000)` before any
evaluation; the identical binary reproduces it at 100. **The STOP is well
taken.**

**The construction is the protocol's precedent, and the wall is mine.** The
probe generator wraps every label body in `Do[body, {i, 1, n}]` at n = 50/100
— the accepted width32000 shape, whose body is itself a 40-iteration `Do`
(40 x 100 = 4,000), and `block-symbol-only-repeat` (1,000 x 100 = 100,000,
exactly at the limit and accepted). The two `-repeat` downvalue labels carry
a 5,000-iteration body, so the same shape cannot exist: 5,000 x 50 = 250,000.
The protocol assumed a body whose own iteration count is small against the
limit; for these two labels that assumption is false. The runtime enforces
the product statically at the iterator-spec setup
(`crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/
iteration/spec.rs:312`), so the program never begins. No candidate result
exists, no product regression is established, and no workload, limit or
source repair was attempted — none is authorized.

**My reproduction and calibration** (`.temp/director/probe-717f/`; the
restart's base binary `3a3d9035…`, stdin closed, profiling off):

- The worker's exact program at n = 50 and n = 100: exit 1,
  `evaluation exceeded iteration work limit (100000)`, no completion marker.
- The exact body repeated N times as N sequential top-level statements:
  exit 0 and the completion marker at N = 50 and N = 100 on the same binary.
  The iteration budget is a nested product scoped to one iterator and
  restored on exit, so sequential statements keep each body at 5,000 and
  never accumulate.
- Callgrind on that shape (base and final, n = 50/100, one draw each,
  0.5-0.7 s per capture): the seam is not called on this label at all (0
  calls, 0 self-Ir in both arms); per-function rows show only the 713
  admission rename pairs (12,294 = 12,294 and 22 = 22, zero net), the
  accepted once-called intercept `drive_input_request` 54->55,
  `Session::drop` 34->33, and hash-seed rows. No new work in the changed
  code.
- `downvalue-pattern-miss-repeat`, the second 5,000-iteration label: the
  wrapped form fails identically; the sequential form completes at 50 and
  100.

**Ruling.** The two `-repeat` labels are probed with the exact benchmark body
repeated N times as N sequential top-level statements at N = 50/100, with
the label's standard setup lines and marker `probe-done-<label> N`; every
other element of the protocol is unchanged (four draws per cell, three arms,
Callgrind, the same binaries, stdin closed, profiling off). The body text
and the 50/100 repetition counts are preserved; only the repetition
mechanism changes, and only because the session's own static nested-product
limit forbids the wrapped form. The report discloses the construction and
keeps the failed wrapped attempts. The other eleven unrun labels use the
standard wrapped construction. The prepared reducer is unchanged: its unit
stays one body repetition at N = 50/100.

**What stands** (verified from the restart's receipts): Stage A's zero-delta
replays against the 718 baseline — fingerprints 68, both certificate cells,
ownership 61/79/8,820, typed slots 10/0/1 with the two designated forced-red
scratch controls, rejection census unchanged; the base and layout source
copies are byte-identical to the retained arms (my own `diff -rq`: 0
differences) and the frozen snapshot equals the live tree (my own diff: 0
differences; `git status --porcelain` empty). Stage B: five modes, 20
process captures, 40 cases, 96 decoder calibrations,
`semantic_differences=0`; publication 14/0/0 at 8 and 1; host schema 4/0/0
at 8 and 1 with 30 legacy byte comparisons. Stage C: both builds SHA-equal
on the forced rebuild (`4172f097…` runtime, `e5ad1b9b…` CLI); queued F6
57/49/20/71 at 501/1001 calls; resumed own vectors
[5300,10600]/[2650,5300]/[1000,2000] at 50/100 calls, slopes 106/53/20, call
slope 1; stack 18,208 x depth + 12,376, residual 0, samples
103,416/194,456/285,496/376,536 twice on both builds; the test build's
warning set is the three standing `test_builtins` plus the five accepted
probe-fixture names, no new. Stage D's timing panel: 12 passes over 120
labels, layout envelope median 0.41% / p90 1.52% / max 7.25%, 23 labels
require instruction settlement (the seven named plus every residual except
the standing width4000 exclusion). The ledger: four interleaved draws per
arm; the six seams exact at identical call counts in every draw
(13/23/18/20/20/18 -> 3/18/14/18/18/14); cold helpers 0 self-Ir / 0 calls;
the source-timing seam 109,077 calls, 22 -> 3 per call; deterministic delta
-3,539,044 (plus the source-timing 2,072,463, recorded alongside); and the
final range lies entirely below the base range
(16,456,664,763-16,459,564,722 against 16,462,010,680-16,469,760,476),
recorded, not gated. The ten completed label panels carry 240 profiles; on
the two source-timing labels base/final/layout read 22/3/22 per call at
50/100 calls, caller 125 (width32000) and 91 (guarded-fallback) unchanged,
helper 0.

**Continuation scope.** Complete the 13 unrun label panels (the two
`-repeat` labels under this ruling's construction; the other eleven
standard), reduce all 23 under the corrected 703 criterion — per-function
self-Ir at identical call counts, the layout envelope, the 713 admission
rename pairs only where both spellings carry equal self-Ir and equal call
counts in the same capture, the two accepted once-called intercepts, and a
growth row in changed code is a STOP — then Stage E and F unchanged. Do not
rerun the ten completed panels, the timing panel or the ledger draws; Stages
A-C stand and are not rerun; keep every raw profile and both failed
attempts. Stage E's workspace bar: the worker's sandbox cannot run the nine
native ZeroMQ loopback tests (the standing EPERM boundary); report it with
the serial reproduction as the brief already directs, and never call it a
skip — my host run is the authoritative workspace.

## Continuation run 7 admission STOP ruling (director, 2026-11-10)

Run 7 stopped at its admission check before any capture, build or acceptance
run: its `initialize.py` verified the live tree against the restart6 frozen
snapshot and asserted that the only difference is this brief's appended
ruling; it observed three differences and exited 1 at 8 threads, with the
serial reproduction exiting 1 identically (`AssertionError:
['CLAUDE.md', 'TODO.md',
'docs/dev/codex/brief-717-lp08-schema-v2-acceptance-campaign.md']`). The
worker wrote no new capture, no new build and no acceptance run.

**The STOP is accepted as an instrument error of mine, not a product
signal.** The difference is my ruling commit `dcc2b983` (`git show --stat`:
three files, 119 insertions, 9 deletions — `CLAUDE.md`, `TODO.md`, this
brief). The brief's "live tree frozen" rule is about the measurement input:
source, tests, inventories and the canonical documents the campaign reads;
the director owns the tree between runs, and a ruling commit's documentation
updates are not product change. The admission bar expecting the brief append
alone was the brief's omission, not worker carelessness.

**My own verification** (`.temp/director/probe-717g/verify.py`, same
exclusions `.git`, `target`, `.temp`): live 2,709 files, snapshot 2,709, 0
missing, 0 extra, exactly 3 differing — `CLAUDE.md`, `TODO.md` and this
brief. `git status --porcelain` empty at HEAD `dcc2b983`;
`git diff --stat 68f2619a..HEAD` names exactly those three files. No source,
test or inventory file differs.

**Ruling.** The admission bar is the exact three-file set `['CLAUDE.md',
'TODO.md', this brief]`: the worker asserts the observed difference list
equals it and proceeds. Any fourth path is a STOP, and any difference under
`crates/` or `scripts/` is a STOP regardless of the set. The rest of the
admission stands: the base, layout and identity arms remain bound by their
retained receipts, the parser calibration on the copied known profiles is
re-run, and the consumed Stage A-C receipts, timing panel, ledger and ten
label panels (240 profiles) are reused with no rerun. This appendix changes
only this brief, which stays inside the set.

**Continuation scope is unchanged:** capture the 13 remaining label panels
(the two `-repeat` labels by the sequential construction; the other eleven
wrapped), reduce all 23 under the corrected 703 criterion, then Stages E and
F. The run-7 prepared label programs under
`.temp/brief-717/continuation7/label-probes/` may be reused only when
byte-identical to this ruling's construction. Dispatch run 8 forked from the
run-7 session `01a1c1dc-356c-7211-8a62-4343de70669f`.
