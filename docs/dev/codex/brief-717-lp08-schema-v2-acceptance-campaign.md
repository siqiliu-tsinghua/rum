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
