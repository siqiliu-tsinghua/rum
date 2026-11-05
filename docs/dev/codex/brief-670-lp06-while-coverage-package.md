# Brief 670 — LP06 E3: the frozen While coverage package

**Shape: measurement package (no live runtime source change).** The 668+669
source stage is committed (`19b61027`). The D2 campaign (brief 671) cannot
decide the four While owner rows because no frozen workload exercises them.
This brief builds the three-label While coverage package from the brief-654
design's exact bytes, wires it into copies of the retained collectors,
freezes its manifests, and proves on both pinned exports that every one of
the four While keys has observed owner/PC evidence at N and 2N. The live
tree stays untouched; the director's tree must still be clean at the end.

## Read first

- `.temp/brief-654/report-design.json`: `q4_workload_design` is the byte
  authority for this package — the nine `.rum` sources, the three harness
  files, the labels, the manifest deltas, the predicted slopes and
  `instrument_ripple`. Do not retype an illustrative program; take the exact
  bytes from `q4_workload_design.files`.
- `docs/dev/codex/brief-653-lp06-d-repaired-campaign.md`: the retained D
  procedure, whose harness, collectors and reducers this package copies.
- `.temp/brief-653/`: the retained procedure root. Read-only. Copy anything
  you execute (`.temp/brief-653/b/resume-harness/`,
  `.temp/brief-653/instruments/run_copy.py`, its `perf/labels.json` (313
  rows: 192 CLI, 121 resume) and `perf/allocation-labels.json` (194 rows: 192
  CLI, 2 resume), `.temp/brief-653/instruments/measure_alloc.py`,
  `.temp/brief-653/site_evidence.py`) into this brief's root before running
  or editing it. Never execute a file in place.
- `.temp/brief-653/tables/while-coverage-gap.json`: the gap this package
  closes (576 frozen programs, no While match, four missing sites).
- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`.

## Director-prepared inputs (read-only)

- `.temp/brief-670/src/base/` — `git archive 654f2ce5`, the pre-A1 base.
- `.temp/brief-670/src/candidate/` — `git archive 19b61027`, the repaired
  candidate.
Copy each into its own build root under this brief's root before building.
Do not build in place, and never edit either export.

## Goal

When this brief is done:

- the nine `.rum` programs exist under `.temp/brief-670/instruments/perf/`
  with exactly the bytes and SHA-256 the design declares;
- the copied resume harness under `.temp/brief-670/b/resume-harness/` carries
  the design's `src/lp06.rs`, `src/main.rs` and `Cargo.toml` bytes, keeps
  every unchanged module byte-identical to the `.temp/brief-653` copy, builds
  against **both** exports, and its `main.rs` asserts the generated source at
  N = 1, 64, 128 against the frozen snapshots;
- the copied label manifests carry exactly 585 program-hash cells (the 576
  prior cells unchanged), 316 instruction labels (313 prior unchanged) and
  197 allocation rows (194 prior unchanged: CLI 192 -> 194, resume 2 -> 3),
  and the regenerated `program-hashes.json` hashes to the design's predicted
  `5baf1a34e0e76b92c89291f233da022daa408dc776fb4ef2eb2d0a0f6cde27bd`;
- on both arms the three labels produce their expected behaviour: the two CLI
  labels exit 0 with `Null` at 500/1000, the resume label produces a List of
  `Null` items with six prompts per item (384/768 resumes at N=64/128), no
  prints or messages and exhausted input;
- each of the four keys — `while.test.evaluate`, `while.body.evaluate`,
  `while.test.suspend`, `while.body.suspend` — has **observed** owner entry,
  own/call N/2N pairs and executed-PC attribution in both arms, with the
  design's predicted phase slopes: zero-trip test 1 / body 0, one-trip test 2
  / body 1 per outer iteration, the resume containing-owner entry slope 4 and
  six prompts per item. An unobservable key is a STOP with the measured
  evidence; a source-order or stack argument is not coverage. Keys that share
  a physical owner are reported as sharing it, never as four isolated costs;
- a separate enabled-mode control per label asserts the phase/key and epoch
  identity at every pause, the per-key start counts, final `open=0` and
  `leases=0`, with no duplicate finish; it never initializes an epoch merely
  to observe absence;
- the four observed owner rows, both arms, are recorded in a package table
  for D2 to compare (the package supplies coverage and observed values; it
  does not decide any cost bar);
- the live repository tree is byte-unchanged; the director's `git status`
  stays clean.

## Scope

Files you MAY write: anything under `.temp/brief-670/` (this brief's new
root), including the copied harness, copied collectors, the nine programs,
the manifests and the calibration receipts.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest, inventory,
  docs or `.git` change; you run no version-control command;
- `.temp/brief-653/` or any other retained root, except to read and copy;
- the two director exports, except to copy;
- any assertion, count or frozen row inherited from 653 or from the design;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the D2 campaign, the acceptance suites, the census/inventory
replays or any timing lane. This brief calibrates three labels only.

## Deliverable

- `.temp/brief-670/instruments/perf/` — the nine `.rum` files
  (`lp06-while-zero-trip-{check,500,1000}.rum`,
  `lp06-while-one-trip-{check,500,1000}.rum`,
  `lp06-while-test-body-twice-{check,64,128}.rum`), byte-exact.
- `.temp/brief-670/b/resume-harness/` — the copied harness; the three
  design files replaced byte-exactly; every other file byte-identical to the
  653 copy and hashed.
- `.temp/brief-670/instruments/program-hashes.json` — regenerated from the
  package bytes; sorted keys, indent=2, one trailing LF; 585 entries; the
  design's predicted SHA-256.
- `.temp/brief-670/instruments/lp06-while-source-hashes.json` — every
  harness file, including unchanged ones (the design's three entries plus the
  copied rest).
- Copied and adapted `run_copy.py`, `site_evidence.py`, `measure_alloc.py`,
  `labels.json`, `allocation-labels.json`, the supplemental three-label
  manifest and a `while_coverage.py` reducer under the package root; hash
  every new or modified script. The resume label needs the separate serial
  resume-own capture (the CLI copy collector filters `engine == cli`).
- Calibration receipts for both arms: the builds, the label runs, the
  copy/alloc processes, the resume-own capture, and the per-key owner/PC
  table (N and 2N, own and `calls=`, owner entry and PC lines).
- `.temp/brief-670/PROGRESS.md` and `.temp/brief-670/report.json`.

## Hard bars

- Byte and hash equality for all twelve design files; the manifest hash is
  reproduced from bytes, never typed.
- The harness builds against both exports and the N = 1/64/128 assertions
  pass.
- Every prior label, program-hash and allocation row is set-equal to the 653
  copy; only the declared additions move the counts.
- The four keys are observed in both arms with real N/2N pairs and PCs; the
  phase slopes match the design or the discrepancy is attributed with the
  measured rows.
- The live tree is unchanged: no file outside `.temp/brief-670/` is written.

## Verification

One cargo command at a time; jobs 8, test threads 8, offline/locked; the
worker's cargo runs plain inside the dispatcher's 16G scope (never nested).
Copy builds run under this brief's root. Measurement runs are serial and
alone, CPU-pinned as the retained procedure does, release with CGU=1, every
`RUM_PROFILE_*` cleared except the declared allocation/lifecycle modes.
Bound anything that may spawn children with `scripts/run-bounded.py`. A
timeout is not data; reproduce an unexpected red at `--test-threads=1`
before calling it a defect.

## Checkpointing

All scratch under `.temp/brief-670/`; keep `PROGRESS.md` with the exact next
action and the receipts already saved. Leave every tree compiling at each
checkpoint.

## Hard rules

- No version-control command, no network, no new `unsafe`, no public API,
  feature or serde change, no manifest or workload change in the repository.
- No hand-edited derived cell: the manifests are generated from bytes.
- No N substitution: the nine `.rum` files keep N = 500/1000 (CLI) and
  64/128 (resume). A Callgrind copy may use the design's documented
  parameter-only smaller N for the CLI labels, provided the derived source
  and its hash are receipted and the frozen files stay untouched. The resume
  N is not reduced without the director rewriting the package.
- No assertion weakened, no prior row changed, no prior root executed in
  place.
- Every changed or new script is hashed and disclosed.

## Report format

`.temp/brief-670/report.json` in `rum-report/1`, every run and verdict line
quoted; run `python3 -I scripts/report.py check 670` and return its summary
and all compliance verdict lines.

## Director gate — 2026-11-05: GREEN

Gate note `.temp/director/brief-670-gate.md`; my independent reproduction
`.temp/director/gate-670/repro.py` (log `repro.log`). Report
`.temp/brief-670/report.json`; `check: OK, 316 citations`. The two FLAGGED
rows are settled diagnostic-setup errors; the frozen workloads, expected
counts and 12 pinned binaries are unchanged.

| Step | Verdict | Detail |
|---|---|---|
| bytes | PASS | 12/12 design files byte- and SHA-256-exact, and equal to the design's literal source text |
| harness | PASS | 9 non-design modules and `Cargo.lock` byte-identical to the 653 copy; 13/13 files match the package hash manifest recomputed from bytes |
| ledger | PASS | 576 prior cells unchanged, 9 added, no overlap; the regenerated ledger hashes to `5baf1a34e0e76b92c89291f233da022daa408dc776fb4ef2eb2d0a0f6cde27bd` (design prediction = package file) |
| manifests | PASS | instruction labels 313 -> 316 (CLI 192 -> 194, resume 121 -> 122) and allocation rows 194 -> 197 (CLI 192 -> 194, resume 2 -> 3); priors byte-equal, none lost or changed |
| behaviour | PASS | 18/18 correctness receipts: CLI `Null` at 500/1000; resume `resumes=384/768` at 64/128 and 6 at N=1; six prompts per item, empty messages/prints, `List[Null; n]`, exhausted input |
| coverage | PASS | four keys x 3 labels x 2 arms = 24 rows with owner entry, own/call N/2N pairs and executed PCs; the keys share the physical `resume_while_loop` owner; zero-trip body keys are negative controls |
| slopes | PASS | zero-trip test 1 / body 0; one-trip test 2 / body 1; resume test 2 / body 1 with 384/768 pauses; owner call slopes 1 / 1 / 4 |
| enabled | PASS | 12/12 `phase_counts=exact epoch_identity=stable open=0 leases=0 duplicate_finish=0`; adopted identity true in the candidate, false in the base |
| tree | PASS | live tree clean; the 23,281-entry change list is confined to `.temp/brief-670/`; no retained-root file is newer than the dispatch |
| scope | PASS | no live source, test, workload, manifest, inventory or `.git` change; no D2, acceptance or timing lane |

Accepted as the frozen While coverage input to brief 671 (D2). The
single-draw owner rows are calibration and decide no cost bar.
