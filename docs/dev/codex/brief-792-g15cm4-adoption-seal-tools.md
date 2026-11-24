# Brief 792 — prepare the whole-table adoption, the 165-row seal and the gate adapters

## Goal

Deliver and calibrate the scratch tooling that brief 793 will invoke: `adopt.py` (the strict F13
and promotion adoption that accepts only freshly produced, role-specific reviews), `seal.py` (the
165-row overlay table and the 151-row mutants table, rebound to its sha), and the gate adapters
(the 12 receipts / six-case gate, the P0543 lane, F48, the 14-test gate and Stage D). Everything
lives under `.temp/brief-792/`. No source is installed and no cargo runs in this preparatory
dispatch.

## Context pointers

- Read `AGENTS.md`; brief 790 including Amendment 1; ruling 799 (`.temp/director/ruling-799.md`);
  brief 791 and its report (the four postimages and the manifest); `docs/dev/codex/TEMPLATE.md`.
- The production code: under `crates/rum-stdlib/src/tests/retention_closure/local/` read
  `production.rs:52–99`, `promotion.rs:5–8,18–92,103–174` and `contract_tests.rs:6–34`. The
  standing gate's merge and certificate step: `scripts/director-gate.py:479–500,655–688`.
- The historical tools to adapt (read-only): `.temp/director/impl-789/stage_c_writers.py:193–380`,
  `.temp/director/impl-789/gate/{common789,probe_build789,stage_c789}.py`,
  `.temp/director/impl-789/lane_build.py`, `run_f48_tests.py`, `run_14_tests.py`,
  `run_stage_d.py`.
- The 790 evidence: `emission-plan.md` (read every table; ruling 799 amends the overlay set to
  165 rows), `overlay-layout.tsv`, `cell-adoption.tsv`, `certificate-stamps.tsv`,
  `overlay-digest.log` (its numeric predictions are superseded cross-checks under ruling 799 —
  recreate the algorithm, do not pin the old constants), `overlay_digest.py`, `standing-inputs.tsv`,
  `receipts.tsv`, `six-case-minima.tsv`, `ignored-set.tsv`, `six-reds.tsv`, `stage-d-reds.tsv`,
  `class-absences.tsv`, `lp01-producers.tsv`, `typed-slot-reports.tsv`, `carried-artifacts.tsv`,
  `merge-controls.log`.
- The archived calibration inputs: `.temp/brief-762/copy/` (the two review files
  `promoted-hash-{true,false}.tsv` and that copy's carried promotion), and the carried armed
  promotion at `.temp/brief-784/images/armed/crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv`.

## Scope

- You MAY write only `.temp/brief-792/`: `tools/` (`adopt.py`, `seal.py`, `gate/` with the
  adapted runners, and a bounded per-tree runner), `calibration/`, logs and the report.
- You MUST NOT: install any source file; run cargo, `rustc` or a test binary; touch
  `.temp/brief-784/images/`, the live tree, `.git`, the historical scripts (read-only), the 791
  evidence, or the network. The adapters operate only on explicitly passed candidate directories
  and evidence paths when 793 invokes them; no path may be hardcoded to a 789 attempt root.

## Deliverable

### `tools/adopt.py`

Arguments: role, candidate root, the two build logs with start times, and the freshly archived
outputs. Two duties:

- **F13.** Compare the 68 records by ID, preserve fields 0–17 byte-exactly, install the exact
  generator output and set the adopted file's mtime to the adoption time (F55), then re-read and
  verify the installed digest. Refuse a generator output produced before the invocation's start.
- **R3 (the whole-table adoption).** Require the exact nine-field header, ordered keys `C01`,
  `C02`; a review generated in this invocation before reconcile (the active column populated, the
  inactive column literally `UNRECORDED`); agreeing metadata in the seven non-build fields
  (indices 0,1,2,3,6,7,8); the single merged file takes column 4 from the `hash=true` review and
  column 5 from the `hash=false` review for both rows. Reuse `merge_promoted` from
  `scripts/director-gate.py:479–500` or prove byte-for-byte equivalence with it. Check freshness,
  tree, debug build and configuration, two C01 instances, one C02 instance, four zero-rejection
  probes and every expected metadata value (`cell-adoption.tsv`). Stage and atomically replace
  only complete validated bytes; set the mtime; re-read. A missing build leaves the candidate
  promotion unchanged. Intermediate single-build adoption, if inspected, must copy that entire
  active cell and preserve the other cell exactly.

### `tools/seal.py`

Start from the checked C.5 160-row overlay table and the 151-row mutants table. Declare the
**five** new paths — `crates/rum-stdlib/Cargo.toml`, `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs`,
`crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`,
`crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`,
`crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` — and refresh the
existing `local-contracts.tsv` row in place. Compute every new row from its baseline preimage and
the disabled and armed bytes: both SHA-256 values and both full lowercase hex strings, sorted by
path, TAB/LF-separated with a final LF, **165 data rows**. The design's physical positions
(115, 116, 154, 155, 156) are superseded by the fifth row; compute and record the actual
positions. Rebind only mutant cell 7 to the full table SHA and write the same 64 lowercase hex
digits plus LF as a separate evidence file outside the candidate trees. Validate the complete
decoded baseline/disabled/armed byte closure, the exact per-role tree file counts, the three ARM
substitutions, the two allowed generated TSV differences, the 9 addition pins and sizes, the
allowance 4/4, and the class/chain checks (`emission-plan.md`).

### `tools/gate/` and the per-tree runner

Adapt the 789 gate scripts, the P0543 lane, `run_f48_tests.py`, `run_14_tests.py` and
`run_stage_d.py` to new work roots, candidate paths and output locations. Keep the stable probe
`ACTIVE` contract, six cold primary builds, the twelve receipt shapes, the 23 exact minima and
all negative controls. The new row guard and the extended-overlay verdict both require **165**;
the additions guard requires 9. Check all **71** ignored names (matching with and without their
trailing reasons) and exactly the ten accepted red names of `stage-d-reds.tsv`, and expose the
plain no-default compile preflight as a distinct runner state whose expected result is
**success** (Q-build; the aux-feature route is not authorized). The flag-on class_lease leg must
enforce `197 passed; 0 failed` (it previously only printed it). Archive per-build outputs on a
write-then-reconcile failure before cleaning run-created debris. The runner reports; it never
installs outside its candidate roots.

## Verification

Calibrate in memory or in `.temp/brief-792/calibration/` only.

1. **Writer identity.** The new writer must reproduce the carried 160-row table byte-identically
   from the carried 160-row input and reproduce all 151 original mutant cells.
2. **Merge positive control.** Merge `.temp/brief-762/copy/.temp/brief-546/certificates/promoted-hash-{true,false}.tsv`
   against the carried armed promotion; the result must be exactly
   `4337950c5b5e5f2a26f94d4aee66aacc0263a62b732d34b74e3021d1182dbbc7`. Against the 762 copy's
   own baseline-form promotion the same operation must refuse the changed anchors
   (`merge-controls.log`).
3. **Negative fixtures.** In calibration only: a missing review, a wrong inactive marker, a
   swapped-role metadata input, and a stale output must each be refused. The freshness check must
   refuse to install a historical file (the 762 reviews verify serialization only, never
   adoption).
4. **Seal.** With the 791 Rust rows and the manifest row as the five new paths and a fixture pair
   of synthetic reviews, require: 165 sorted unique rows, the refreshed local-contracts row, the
   placeholder promoted row encoding the fixture's two cells, the actual computed physical
   positions, the fragment digests recomputed from the actual bytes, the mutant rebind, and a
   second run byte-identical and idempotent. The **final** table SHA is measured in 793, never
   here.
5. **Gate adapters.** Consume the entire receipt key set and reject omitted, swapped or corrupt
   entries; decode all twelve archived receipt shapes and check all 23 minima; dry-run the new
   path bindings and the exact cargo argument vectors (including `--no-default-features` for the
   second production review, before `--`, and the plain preflight's expected success). No native
   run, no source installation.

## Checkpointing, STOP rules and disclosure

Run serially and bound each parsing or calibration process with
`scripts/run-bounded.py --timeout 600 --`. Checkpoint in order: the input hashes, the positive and
negative calibration logs, the sorted coverage manifest, the new gate paths. STOP on any metadata
disagreement in the positive control, an accepted negative fixture, a stale output admitted,
cross-role reuse, an unlisted source change, a differing fragment digest, or an omitted gate. A
reported SHA of a completed new table must be backed by the actual default and no-default reviews
of both arms — none exist in this dispatch, so none may be reported. No network, git or cargo.

## Budget

2,400 s (the reserve after brief 791 is 805.83478847017 s; this carve is topped up by
1,594.16521152983 s per the standing 2026-11-19 ruling).

## Report format

Write `.temp/brief-792/report.json` in `rum-report/1`: verdict DONE or STOP; every produced tool
with its argument contract; every calibration result with verdict lines quoted verbatim; the
inherited inputs; the pending production emissions (named, not claimed); the exact new gate
counts (165 / 9 / 71 / the ten red names / 197-0); any deviation. Run
`python3 scripts/report.py check 792` and return its summary.
