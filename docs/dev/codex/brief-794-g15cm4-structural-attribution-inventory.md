# Brief 794 — G15-CM4 the structural attribution inventory on the final release tree

**Design-only brief (`Scope: NONE — no source change`), staged A–D, one gate.** It continues
ruling 806 (gate c154). R4 installed the sealed release into live and every producer replayed
green there (commit `e66e8531`). This brief produces the **complete zero-unknown attribution** of
every structural difference the C94 instrument finds on the final tree, restates the **stage-A
completion bar** for the probe and hot-path A/B, and drafts that A/B completion brief (795). It
changes no source byte and no sealed byte.

## Goal

When this brief is done, in `.temp/brief-794/` only:

- The **recovered, calibrated instrument**: the four surviving tools copied byte-exact into
  `tools/render_tools/` (hashes pinned below), a new driver that re-derives the c140
  classification from the kept `.temp/brief-781/stage_a.py`, and four calibration lines
  (self-comparison all-verbatim with `ws=0`; a planted body mutation fires as `diff`; a planted
  new function fires as `extension`; a whitespace-only change fires as `ws`). If any copied tool
  differs by one byte from its source, STOP.
- The **final-tree comparison**: every outermost function of the production-source scope (the
  four `CRATES` dirs) classified on the **armed tree against the real tree** — the A/B's new
  against its base (brief 781's roles) — printing `COMPARE verbatim=… ws=… wrapper=… dispatch=…
  cold=… facade=… extension=… diff=…`, the file count, every diff row, every extension row, the
  probe hits, the cold-twin count, and `HOT MATCHED covered=… unmatched=… missing=…`.
- The **zero-unknown attribution map** (`attribution.tsv` plus its narrative): every `diff` and
  every `extension` row attributed to exactly one declared shape, with its evidence (a named
  table row, byte identity, or source read) and its owner; the row count equals the classifier's
  row count; `unexplained=0`.
- The **restated stage-A bar** (`stage-a-bar.md`): the exact lines and the closed attribution the
  A/B completion asserts. It is derived from this brief's own measurement — F17's lesson: never
  inherit a declared set from an earlier render.
- The **drafted A/B completion brief**
  (`.temp/brief-794/draft/brief-795-g15cm4-probe-hotpath-ab-completion.md`), in `TEMPLATE.md`
  shape: brief 781's stages B–D on the final trees (three release `rum-cli` binaries at one
  stable path with one codegen unit, the layout control, the panel, four passes per arm, the
  tier-median quotients, `perf stat` 3+ runs, the Callgrind Ir slope probe, the verdict), with
  this brief's stage-A bar as its stage A.
- The report (`rum-report/1`).

## Context pointers

- Ruling 806 (`.temp/director/ruling-806.md`), ruling 805, ruling 804; brief 793 and its R4
  handoff; the R4 evidence `.temp/director/r4/` (`install.json`, `logs/`, `emissions/`,
  `seal/`).
- The final trees, re-hashed at entry and exit (whole-tree pin, debris excluded):
  baseline `fa01b3d1e38574a78d6fb9c4cf80bf36e32470d294646b28aabede99c669be80` / 5,842;
  armed `c04bbf8245a8b41c6b069f36df0c2da3785f25e7565a6938e2dff29d77a14afb` / 5,851;
  disabled `60ad65edb34b7ce83c5b924c63b308b0c1f4b5c5d1316125efce976312038610` / 5,851;
  real `d703b1655dc3d2c5ad8cdceaf0b285a1d1261c1742ddf736b5144852a1ac7503` / 5,851.
  Live equals the armed tree under `crates/` (R4; `a7de3a76…` / 4,700 crates files), but the
  comparison's reference is the **real** tree, not live.
- The sealed tables (under the armed tree unless named): overlays 165 rows `57e9acbd…`, mutants
  151 `e9940d5d…`, additions 9 `243dc0cf…`, chain 152 `0e508bb4…`, class-provenance 2,422
  targets / 20,625 rows `626eb3d4…`, `local-contracts.tsv` `5d27e91b…`, `promoted.tsv`
  `51b693ec…`.
- The instrument: `.temp/director-gate-c93/stage-a/tools/` and `.temp/director/gate-c88/` hold
  byte-identical copies — `ablate.py` `396d9474be6df26d8eabc55bfefe5cd5f543dddbd318892d7dd4e6edc35458e0`,
  `canonical.py` `6f39d69bd41ed8a3fe3acfbe26d11328c3a5d89c3b64faa928690a3239b08d82`,
  `fn_split.py` `4971a55308a3930edf6a5f521c6690623da0441169e759b563b2322a8b80be7a`,
  `recorder_removal.py` `0f1d3c7b4352382304c56542be9c9a939255f7b4c1c80cda7bed9136b58d1abf`.
  The kept driver is `.temp/brief-781/stage_a.py`
  (`b342b07f96aa11b6a42de8fab4bd75e56d59dbdce692a05023fa766e37420e0b`); its `classify()` is the
  reference logic. The old `hot_compare.py` and the 749/779 trees no longer exist (clean-temp):
  **do not plan on them**. `.temp/brief-781/report.json` keeps the c140 numbers as history only.
- Brief 781 (`docs/dev/codex/brief-781-g15cm4-probe-hotpath-ab.md`) for stages B–D and the
  protocol; briefs 703 and 708 for the 703/708 measurement criterion, the layout control and the
  Callgrind Ir slope probe; brief 637 for the interleaved order. Brief 790's design
  (`.temp/brief-790/{emission-plan.md,scene-membership.tsv,standing-inputs.tsv,cell-adoption.tsv}`).

## Scope

- You MAY write only under `.temp/brief-794/`: `tools/`, `driver/`, `scratch/`, `logs/`,
  `evidence/`, `draft/`, `PROGRESS.md`, `report.json`.
- MUST NOT touch: every repo file (live is the released artifact now), the four 793 images, the
  R4 evidence, `.git`, `Cargo.lock`, any `docs/` file. No cargo, no workspace, no epoch: Python
  only, offline.
- The four trees must be byte-unchanged at the end (re-hash and print the `FINAL TREE` lines).

## Stage A — instrument recovery and calibration (1,600 s)

1. Copy the four tools from `.temp/director-gate-c93/stage-a/tools/` into
   `tools/render_tools/` (one directory: `fn_split.py` imports `canonical`, `ablate` and
   `recorder_removal` by name). Print one `TOOL COPY path=… sha256=… equal=true` line per tool.
   Any byte difference is a STOP.
2. Write `driver/classify_final.py`: a new driver that re-derives the classification from the
   kept `stage_a.py`'s `classify()` and its line formats (`COMPARE …`, `files=… diff_rows=…
   probe_hits=…`, the per-row lines, `COLD TWINS`, `EXTENSION FILE`, `HOT MATCHED`, `C94 HOT
   PATH`). Set `fn_split.LIVE` to the reference tree explicitly and **clear
   `fn_split._LIVE_CACHE` whenever the reference changes** (it caches per path). Never edit the
   copied tools' logic.
3. Calibrate on scratch copies under `scratch/` (copy each tree's four `CRATES` dirs; never
   mutate the trees themselves):
   - `SELF`: candidate = reference → every outer function verbatim, `ws=0`;
   - `BODY`: change one named hot function's body in the candidate → exactly one new diff row
     naming it;
   - `EXTENSION`: add one named new function file under `crates/rum-host/src/` in the candidate
     → an extension row naming it;
   - `WS`: a whitespace-only change to one verbatim function → `ws=1` and no new diff.
   Print the four calibration lines to `logs/calibration.log`. Any miss is a STOP.

## Stage B — the final-tree comparison and the zero-unknown attribution (1,700 s)

1. Re-hash the four trees to their pins.
2. Primary run: candidate = `images/armed`, reference = `images/real`. Print the `COMPARE` line,
   the `files=` line, every `HOT-DIFF`/`NOLIVE` row, every extension file with its row count,
   the `PROBE` lines, `COLD TWINS`, `HOT MATCHED`, and the closing `C94 HOT PATH … unexplained=…`
   line.
3. Report the structural facts: `ws=0`; the probe hits exactly the four
   `crates/rum-host/src/session.rs cm740_{enter,ordinary,hit,driven}`; every `HOT_FNS` name
   covered; the hot functions matched through the tool's own rules (dispatch wrappers strip,
   canonical rule) — never by loosening the tool or the driver.
4. Attribution: one row in `attribution.tsv` per diff/extension row —
   `path<TAB>function<TAB>kind<TAB>shape<TAB>owner<TAB>evidence`. Each shape must point at a
   named sealed artifact or design fact: the 165 overlay rows (the row's `armed_sha256` is the
   candidate file's byte; the real tree's file is the preimage), the 9 additions, the 15
   carried/regenerated/adopted files, the four 791 postimages (`216ae720…`/`173,656`,
   `b59e0ede…`/40,719, `4a76c2f5…`/21,829, `67be9610…`/1,010), the class-form re-emission (the
   c140 specimens: `evaluate_in`'s bare match-scrutinee wrapper plus the closure-body block and
   trailing comma; `strip_hot` reduces 24 of the residual), the adopted ledger's `Local`
   machinery (`entries`, `check`, `check_entry`; the exact F4 message function is
   `production_host_payload_ledger_is_complete_and_attributed`), the Q25/class-lease insertions
   in `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs`'s `resume`
   (`control::resume` / `note_typed_consumer` / the driven gate / `control::active`), and the
   arm switch files. An argument without a named artifact is unknown, not attributed — a row
   that cannot be closed is a STOP naming it.
5. Report how the measured counts relate to the c140 history (`87 diff / 136 extension` on the
   779 install tree) and why they differ; the c140 numbers are not a bar.

## Stage C — the restated stage-A bar and the drafted A/B brief (900 s)

1. `stage-a-bar.md`: the exact lines the A/B completion's stage A must print (the trees' pins;
   `COMPARE` with `ws=0`; `HOT MATCHED covered=21 unmatched=0 missing=0`; four probe hits at
   the named sites; `unexplained=0`; attribution rows = reported rows) and the driver invocation
   that reproduces them. Counts measured here are recorded as measured values; the asserted
   facts are the structural ones and the attribution's closure.
2. `draft/brief-795-g15cm4-probe-hotpath-ab-completion.md` in `TEMPLATE.md` shape, with stages:
   - A — entry and the structural bar (re-hash, re-run this driver, assert the lines);
   - B — three release binaries and receipts (real base, armed new, the layout control = base
     plus one behaviour-neutral cold-referenced perturbator in a scratch copy; one stable path;
     `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`; `profile=release`; tree hash, file count, binary
     sha256, cargo argv, elapsed);
   - C — the measured A/B (the panel, four interleaved passes per arm, every `timings.tsv`, the
     tier-median quotients, `perf stat` 3+ runs per arm, the Callgrind Ir slope probe at 50/100
     for every label the instrument must settle, serial and alone, no competing load);
   - D — the verdict, the cleanup and the report.
   It must name its own carve and its STOP rules and reuse brief 781's standing exclusions
   restated.

## Stage D — the report (300 s)

Write `.temp/brief-794/report.json` in `rum-report/1`; `python3 scripts/report.py check 794`
must pass. Checks cover every stage; measurements carry the counts; deviations and refused git
attempts are disclosed; the questions list is expected empty.

## Checkpointing and STOP rules

- After each stage write `.temp/brief-794/PROGRESS.md`: done and verified with saved evidence,
  the exact next action, the trees' state, what the director needs to take over.
- STOP, with the fact named: any tool-copy byte difference; any calibration miss; `ws` ≠ 0; any
  unmatched or missing hot function; any probe hit outside the four; any unattributed row; any
  tree hash drift; any write outside `.temp/brief-794/`; any refused git attempt (disclose it
  even though the fence refuses it).

## Hard rules

- Do NOT `git commit`, branch, push or touch git config; the fence logs every attempt and the
  report discloses it.
- No source edits, no table edits, no network, no new dependency, no `unsafe`.
- Never edit the copied tools' logic and never add an accept list or exclusion to the driver to
  make a row pass: the tool reports, the attribution explains; the bar never loosens.

## Budget

Focused **4,500 s** (A 1,600 / B 1,700 / C 900 / D 300) + workspace 0 + epoch 0 = **4,500 s**.
The 1,165 s that 793 returned plus an explicit **3,335 s** top-up from the campaign allocation
(standing 2026-11-19 ruling) fund it → reserve 0 at the carve. Unspent returns; a projected
overrun stops for a director ruling, never a scope cut.

## Report

`rum-report/1` at `.temp/brief-794/report.json`; every claim cites a saved log under
`.temp/brief-794/`; no citation may point at `.temp/brief-781/` or any deleted path for a fact
this brief must establish.

## Amendment 1 (gate c155) — the probe bar restated as a structural closure; resume at Stage B item 4

The first dispatch stopped correctly at Stage B's probe rule: `PROBE SITES expected=4 observed=7
outside=3 missing=0`, the four `session.rs` facades present, and three `VERBATIM-PROBE` rows
(`host_payload_ledger.rs check_entry` line 132; `reconcile.rs is_local_construct` line 203 and
`reconcile_falsifier_disposition` line 1075). The worker left the copied tools byte-exact,
filtered nothing and changed no byte of the four images; the director recomputed the four exit
pins and the tool hashes independently.

The stop is correct under the brief as written — but the bar was mine and it was wrong. The
kept rule appends a probe hit in exactly two branches: the facade branch (`stage_a.py:262-266`)
and the verbatim branch (`stage_a.py:295-298`). At c140 the ledger's `check_entry` was a `diff`
row and `reconcile.rs` contributed 45 `extension` rows (the c140 report states both), so the
verbatim branch could not fire for them and the four facade hits were the whole list. On the
final pair those files are shared between the arms: the ledger producer is byte-identical in
all four roles (`06cadb2c…`) and `reconcile.rs` is `additions.tsv` row 5 (`da9bc5d6…`, 37,791 B;
armed/disabled/real, absent in baseline), so their functions classify verbatim and the tool
scans their text. The three occurrences are literal text inside the declared `Local` machinery —
a shape this brief already names: two carry the `Local` predicate
(`trail.starts_with("rum_runtime::cm740_probe::") && ty.starts_with("dyn Fn")`, the definition
of `Disposition::Local`) and the third constructs the FALSIFIER d probe-rooted `public input`
fixture. The probe's host surface stays `crates/rum-host/src/session.rs` (overlay row 4,
`ef78a1de…` → `7127298d…`); all other probe occurrences sit in rows the tool classifies as
anything but verbatim (the delta's own instrumentation and its cold twins), not in shared code.

F63: "exactly four hits" was c140's measurement inherited across a tree change — F17's lesson
applied to the probe line, not only to the diff set. This amendment replaces Stage B item 3's
probe sentence and the STOP rule "any probe hit outside the four" with a closure:

- The tool's rule is unchanged and the copied tools stay byte-exact; no accept list, no exclusion.
- The four facade hits are required, exactly as printed.
- Every `VERBATIM-PROBE` row is attributed in `evidence/probe-attribution.tsv`
  (`path<TAB>function<TAB>kind<TAB>shape<TAB>owner<TAB>evidence`), one row per outside site; the
  table equals the observed outside multiset exactly; each evidence names the armed/real file
  hashes with `byte_equal=true`, the exact source line, and the declared shape (the adopted
  ledger's `Local` machinery; for `reconcile.rs` also `additions.tsv` row 5).
- `verify` gains the closure check (today `check_inventory` does not test the probe set): after
  the 120-row attribution check it requires the four facades, the exact outside table and the
  attribution closure, and prints `PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`
  alongside `ATTRIBUTION rows=120 … unexplained=0` and `C94 HOT PATH PASS … unexplained=0`.
- `observe` is the raw report: it prints every hit and row and exits 0; `verify` is the gate. The
  first run's STOP evidence stays untouched.

Resume at Stage B item 4 (the attribution), then Stage C (`stage-a-bar.md` states the structural
bar — counts are measured values, never bars — and the draft 795's stage A asserts the closure)
and Stage D (the report discloses this amendment and the first STOP as accepted deviations).
Stage B item 5 cites the c140 report's own counts (diff=87, extension=136, 45 of them
`reconcile.rs`; `check_entry` a diff). The four trees must still pin at exit.

Charged ≈646 s of the 4,500 s carve (unit `brief-794-cdx-20261125-073227-2531368`, 07:32:27–
07:43:16, peak 347,660,288 B, oom 0); ≈3,854 s remain. Unspent returns; a projected overrun
stops for a ruling.
