# Brief 781 — G15-CM4 Stage D repair: the probe and hot-path A/B — the structural C94 comparison on the installed render and the measured ordinary-path A/B with the layout control

**Implementation brief, staged (A–D), one gate.** It continues ruling 789 (gate c139) and
brief 780 (`f8b006f2`). The source A/B is green (c136–c139); this brief supplies the
performance half the generator/source stage owes: the structural proof that every rendered
hot path is still the live body, and the measured rum-vs-rum A/B that the render adds no
ordinary-path cost. Brief 766 carried both obligations and stopped before them.

## Goal

When this brief is done, in `.temp/brief-781/` only:

- The entry re-authentication of the 779 artifacts: install/armed `e01d13cf…` (5,850),
  baseline `ac36bc81…` (5,842), disabled `9e682e49…`, real `4dd8b8a1…`, the four tables
  (`overlays.tsv` 147 rows, `mutants.tsv` 151, `emission_chain.tsv` 152, `additions.tsv` 8),
  the class table `8c8b2232…`.
- The **structural hot-path comparison** (brief 743's C94 instrument, re-driven on the
  install tree): for the production-source scope (`crates/rum-runtime/src`,
  `crates/rum-host/src`, `crates/rum-stdlib/src`, `crates/rum-jupyter/src`), every
  outermost function classified against the live tree — verbatim / ws / wrapper / dispatch /
  cold / facade / extension / diff — with `ws=0`; every **hot** function matched; the
  `diff` rows exactly the declared non-hot shapes (the four 743 files' named functions
  **plus** `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs`'s F4
  Local-count message delta, proven exactly); the `extension` rows only the adopted
  `reconcile.rs`; the cold-twin count recorded; the four probe hits exactly
  `crates/rum-host/src/session.rs cm740_*`; and the line
  `C94 HOT PATH PASS arm=armed … unexplained=0` with every count quoted. Control: the real
  image's copies of the render's file set are byte-equal to the live preimages.
- Three release `rum-cli` binaries built **at one path with one codegen unit** — the real
  image (base), the armed image (new) and the layout control (the base plus one
  behaviour-neutral, cold-referenced perturbator, in a scratch copy only) — each with a
  receipt: tree hash, file count, binary sha256, `codegen_units=1`, `profile=release`, the
  stable build path, the cargo argv and elapsed.
- The **measured A/B** through `scripts/core-language-microbench.sh` on the same panel
  bytes for every arm: profiling environment cleared, in-kernel `AbsoluteTiming`, one fixed
  CPU, serial and alone, no competing load; four passes per arm in the interleaved
  `brief-637` order (three binaries); every `timings.tsv` stored; the tier-median quotients
  over every label; the whole-panel `perf stat` instruction counts, 3+ runs per arm; and,
  for every label the instrument must settle, the 703-style Callgrind Ir slope probe at
  50/100 under the three binaries, four draws per cell, per-function self-Ir resolved.
- The verdict: the layout control calibrates the panel-wide noise floor; every movement is
  inside that envelope, or settled as noise by equal executed instructions; **no new
  ordinary-path cost and no unexplained delta**.
- The five 779 trees byte-unchanged after every run; the live tree untouched; the report.

Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Context pointers

- Ruling `.temp/director/ruling-789.md`; brief 780
  (`docs/dev/codex/brief-780-g15cm4-deferred-acceptance-completion.md`, committed
  `f8b006f2`); ruling 788 and brief 779 for the trees and receipts.
- The structural instrument: `.temp/brief-749/attempt-4/generator/brief-743/stage-b/`
  (`hot_compare.py`, `render_tools/fn_split.py` with `CRATES`, `HOT_FNS`, `live_match`,
  `probe_marks`, `strip_dispatch_wrappers`, `ablate.py`, `canonical.py`,
  `recorder_removal.py`). Copy them under `.temp/brief-781/tools/`; never edit the copies'
  logic — write a new driver. The c94 verdict on the 743 render is the reference;
  `cold_twins=334`, `ws=0`, four probe hits, and the four accepted non-hot shapes were its
  numbers.
- The measurement protocol: `docs/dev/codex/brief-703-lp08-ledger-replay-and-acceptance.md`
  (four passes per arm, the three arms, the base-plus-cold-perturbator layout control, the
  tier-median quotient, timing secondary); `brief-708-lp08-final-source-campaign.md` (the
  corrected 703 criterion, the Callgrind Ir slope probe at 50/100, per-function self-Ir);
  `brief-637-lp03-evaluator-dispatch-arguments.md` (the interleaved order; ruling 3's
  degenerate label exclusion).
- The panel: `scripts/core-language-microbench.sh` (`--rum-cli`, `--output-dir`,
  `--scale`, `--reps`), `scripts/run-bounded.py` for every bounded run.
- The 779 trees: `.temp/brief-779/{install,baseline,images/{armed,disabled,real},receipts}`.

## Scope

- You MAY write only under `.temp/brief-781/`: `tools/`, `active/` (the one build path),
  `target/`, `tmp/`, `bins/`, `logs/`, the scripts, the evidence, the report. The live
  tree, the 779 trees and `.temp/brief-774`/`772`/`776`/`777`/`780` are read-only.
- No file of the 779 trees may change; no `target/`, `.temp/` or cache directory may remain
  inside them (builds use an external `CARGO_TARGET_DIR`; test/panel scratch is created and
  removed under your own brief directory). Re-assert all five tree hashes at the end.

## Stage A — entry and the structural comparison (700 s)

Re-authenticate with line-quoted verdicts: the five tree hashes and counts; the four
tables' digests and row counts; the class table `8c8b2232…`; the three images' arm-differing
cells (the three-file bijection). Any mismatch stops.

Copy the c94 tools, then drive them from a new script: parse the install tree's
production-source scope, classify every outermost function against the live tree's same
file and function (`live_match`), count the classes, assert `ws=0`, assert every `HOT_FNS`
function matched, and list the `diff` rows. The declared `diff` set is: the four 743 files'
named functions (the c93 frame-sink primitive and the three dispatch drop-offs —
`insert_suspension_frame`, `insert_typed_suspension_frame`, `insert_typed_erased_frame`,
`insert_typed_erased_frame_named`; `builtin_set`; `resume_accumulate_from_index`,
`resume_suspended_accumulate_from_index`; `resume_compound_expression_from_index`,
`resume_switch_after_match`) at their installed bytes, plus `host_payload_ledger.rs`'s
changed function(s), each proven to be exactly the F4 delta (the Local count in the
ledger message and the arithmetic). Any other `diff` row, any unmatched hot function, any
`ws` row or any probe hit outside the four `session.rs cm740_*` sites stops. A function
whose file is absent from live is an `extension` by definition — classify it so before the
mark rule, never as a `diff` row.

Print `COMPARE …`, the `diff_rows` and `PROBE` lines, the cold-twin count, and
`C94 HOT PATH PASS arm=armed files=… cold_twins=… covered_hot_functions=… accepted_nonhot_shapes=… unexplained=0`.
Control: for the render's file set, the real image's bytes equal the live preimages.

## Stage B — the three binaries at one path with one codegen unit (1,200 s)

For each arm in order — real (base), armed (new), control (base + perturbator) — assemble
the complete tree under the same path `.temp/brief-781/active/` (excluding `.git`,
`target`, `.temp`, `__pycache__`, `.mypy_cache`), then build
`cargo build --offline --locked -j 8 --release -p rum-cli --bin rum-cli` with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` and `CARGO_TARGET_DIR` under the brief; copy the
binary to `.temp/brief-781/bins/<arm>` and write its receipt.

The perturbator is one behaviour-neutral, cold-referenced addition to a scratch copy of the
real tree — never executed, producing byte-identical panel output, shifting code layout.
State its mechanism; prove non-execution (the panel's output is byte-identical to base and
the perturbator's own counter, if any, stays zero). It exists only in the scratch copy.

STOP on a build that is not one path and one codegen unit, an incomplete receipt, a
perturbator that changes any output, or a changed tree.

## Stage C — the measured A/B (2,200 s)

Generate the panel source once (`scripts/core-language-microbench.sh --output-dir` under the
brief, `--scale 1`, its own `--reps`), record its sha256, and run the same bytes for every
arm with `--rum-cli` pointing at the recorded binary (never rebuild inside the script).
Clear every `RUM_PROFILE_*` variable; profiling off. Pin one fixed `cpu_core` CPU (record
which), serial and alone, no competing load.

- **The timing lane (secondary).** Four passes per arm in the interleaved `brief-637`
  order over the three binaries; store every `timings.tsv`; compute per-label tier-median
  quotients (new/base and control/base) over every label, with the tiers by the 637
  definition and the degenerate-label exclusion applied where it applies.
- **The instruction instrument (deciding).** `perf stat` instruction counts for a whole-panel
  run of each arm, 3+ runs each, on the same fixed CPU and the same panel bytes; record every
  count. The control's spread sets the noise floor.
- **The settlement.** Apply the corrected 703 criterion: a positive movement is a real delta
  only if it exceeds the control's envelope **and** contradicts the instruction instrument;
  inside-envelope movements with equal executed instructions are noise. For every label the
  instrument must settle, run the 703-style probe — the label body at 50/100 under the three
  binaries, Callgrind Ir slopes, four draws per cell, per-function self-Ir resolved and
  compared.

If the fence denies `perf` or Callgrind, STOP with the exact refusal; do not substitute a
timing-only verdict. STOP on a new ordinary-path cost, a label outside the envelope the
instrument cannot settle, or a panel movement the control does not explain.

## Stage D — the closure (300 s)

Re-assert the five 779 tree hashes and the absence of `target/`/`.temp/` inside them; the
live tree untouched (`git status --porcelain` still the 23 inherited entries); quote the
final verdict lines; write the report.

## Verification and acceptance

The brief passes only with all of: the entry re-authentication; the structural comparison
green with the `diff` set exactly as declared and the four probe hits; the three build
receipts (one path, one codegen unit, the control behaviour-neutral); the four-pass panel
and the instruction counts recorded per arm; the layout-control envelope stated and every
label inside it or settled by the Callgrind probe; no new ordinary-path cost and no
unexplained delta; the five trees byte-unchanged; every verdict line quoted verbatim; every
deviation disclosed.

## Checkpointing and stops

Checkpoint after every stage (`.temp/brief-781/PROGRESS.md`, logs under
`.temp/brief-781/logs/`). Stop, with the exact evidence, on: any entry pin mismatch; a
`diff` row outside the declared set; an unmatched hot function; a probe hit outside the four
`session.rs` sites; a `ws` row; a build not at one path with one codegen unit; a perturbator
that changes behaviour; a fence refusal of `perf`/Callgrind; a new ordinary-path cost; an
unsettled label outside the control envelope; a changed 779 tree; or a projected overrun. A
stop is a correct terminal state; never weaken a rule, pin a number you did not measure, or
edit an inventory.

## Hard rules

Never edit the copied 743 tools' logic; never edit the 779 trees, the live tree, brief 780's
checker or any inventory; never run a test suite here (this is the measurement stage); no
new `unsafe`; no default-path cost; no network; no new dependency; no git of any kind;
`.temp` only; never `/tmp`; never read `~/.config/ccds/api-key`; Q25 stays open; 744/745/746
stay held; CM5 stays unauthorized.

## Budget

Focused **4,400 s** (A 700 / B 1,200 / C 2,200 / D 300) + workspace **0** + epoch **0**,
carved from the stage reserve after brief 780 (6,883.48878847017 → **2,483.48878847017 s**;
brief 780 charged 3,356.6 s of its 4,100 s carve and returned 743.4 s). Unspent returns; a
projected overrun stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-781/report.json`, appended as stage `A / c140`; `summary`,
`changes`, `deviations`, `questions`, every verdict line quoted, and the final message
`scripts/report.py summary`. The director gates from raw evidence.

## Gate c139 note (for the reader)

Brief 780's stages A–C are accepted from raw evidence: the entry pins, the three-hunk
checker copy with its three selftest arms, the fresh flag-on run's `197 passed; 0 failed`
(with 593 filtered recorded, not pinned) and the exact
`RECEIPT DIFF PASS rows=97 lost=0 new_miss=0 withheld_bad=0 digest=PASS`. Its stage D stopped
correctly: the workspace's 73 reds are the nine loopback refusals, 61 names from the 776
reference set and three retention `local` contract reds — the latter are the flat rot
guard's fingerprint expiry from the five declared under-`crates/` `.rs` edits (F13, my
ordering defect: the regeneration is the render brief's), while the one `source_auth` red is
the copied tree's missing `.temp/brief-743/` (F14, identical in the 776 log). The
fmt red is exactly the two Q25 carriers. The deferred acceptance's workspace leg stays open
on those four names; the render brief closes them. After this brief: the deterministic
render (F13's regeneration, F14's hermeticity, F5, the Q25 formatting, the full workspace
green, the source-gate re-run), the re-freeze, the fresh epoch, the Stage C/C10 readmission
and a further 744 restart, each gated.
