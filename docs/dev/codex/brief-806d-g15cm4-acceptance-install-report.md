# Brief 806d — G15-CM4 D.2: the acceptance resumed, the install and the report

Read briefs 806, 806a, 806b and 806c first. This continuation supersedes 806c's Stage C start
point and its charge accounting only; the B2 declaration stands. Everything else stands. No tag,
no network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`. Scratch under
`.temp/brief-806/`; never `/tmp`. No git.

## Why this brief

The 806c attempt was killed by the harness's memory-pressure reaper at 12:30:27 while the lib
acceptance gate was running — a host-harness event, not a worker fault (the scope's own peak never
approached its 16 GiB cap). Before the kill it delivered Stage B2 PASS and the presentation/fmt
acceptance receipts; the lib job is a partial raw log with no verdict. The user disabled the reaper
and restarted; the work resumes from the checkpoint. Ruling 842 (gate c190) stands.

## Checkpoint — verify, do not redo

- **The render** (from the 806b attempt): the four role trees — baseline `cb9205a6…`/5,842, real
  `dda9146e…`/5,851, disabled `30cd48fc…`/5,851, armed `df0db63c…`/5,851; the ten outputs (the
  seven tables `d3588fb9…`, `1c48e15a…`, `99f7c5bb…`, `dc648d65…`, `8e29c41f…`, `2ac559d1…`,
  `70a6da97…`; additions `feca9443…`; mutants `c214a4aa…`; overlays `44bb8f11…`); the 190-row map
  (`r4-pairing.tsv`; map `bfb2fe15…`, install pairing `8f7973e3…`; identity rows 35/45/51/170; the
  ten changed rows 160/164/165/166/171/172/182/183/184/186); the seal.
- **Stage B2** (`d2/stage-B2.json`, 310.423 s): the four `DECLARED CELL PASS` lines;
  `ATTRIBUTION rows=126 reported_rows=126 diff=101 extension=25 shapes=27 sha256=487dfbb3…
  unexplained=0`; `PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`; `C94 HOT PATH
  PASS arm=armed files=1257 cold_twins=334 covered_hot_functions=21 accepted_nonhot_shapes=101
  unexplained=0`; the six falsifier `CONTROL PASS` lines; `REFERENCE SCENE PROJECTION PASS
  rows=122 unchanged=116 refreshed=3 rebound=1 added=2 diffs=97 extensions=25 shapes=26 owners=14
  hot=21 missing=0 unmatched=0 ws=0`; and the exact line `SCENE PROJECTION PASS rows=126 diffs=101
  extensions=25 shapes=27 owners=15 hot=21 missing=0 unmatched=0 ws=0`.
- **The acceptance receipts** (`d2/acceptance/`): `ENV PASS` (armed `df0db63c…`/5,851, baseline
  `cb9205a6…`/5,842, lane tree `98c91345323e…`); `ACCEPTANCE PASS gate=presentation` 0.750 s;
  `ACCEPTANCE PASS gate=fmt` 2.762 s. The lib job (`d2/acceptance/jobs/lib/raw.log`, 3,594 lines)
  has no verdict and is discarded and rerun.
- **The live tree**: the 806b exit set (5,885 checkpoint paths + the two admitted briefs = 5,887)
  plus the declared docs extras — `brief-806a-…md` 9,472 B `909b1b4b…`, `brief-806b-…md` 4,872 B
  `c6b8ab01…`, `brief-806c-…md` 8,962 B `a7251318…`, and this brief (hashed at entry) — = 5,889
  paths; any other path or content delta stops with the cell. The nine live hashes, the seven
  tables and the lane pin re-authenticate as in 806b.
- **Charge**: the 806c attempt charges 724 s (B2 310.423 recorded; 413.577 startup/A/C-partial);
  cumulative 3,482.314 s of 28,400; **24,917.686 s remain**. This brief allocates 17,100 s:
  A 300 / C 10,000 / D 6,000 / F 800; 7,817.686 s stay unallocated reserve.

## Stage A — re-entry (300 s)

Re-authenticate the checkpoint: the four role roots, the ten outputs, the 190-row map
(`bfb2fe15…`, install pairing `8f7973e3…`), the seal, the B2 receipts (`stage-B2.json`,
`scene/attribution.tsv` sha `487dfbb3…`), the presentation/fmt receipts. Confirm the live tree =
the 806b exit set + the four admitted briefs; any other path or content delta stops with the cell.
Do not redo the render or B2.

## Stage C — the acceptance on the rendered tree (10,000 s)

Restart at the lib gate and run the whole acceptance:

- `cargo fmt --check` is already green (receipt retained); do not rerun it.
- The lib suite: **1,115 passed / 0 failed / 58 ignored**.
- The class gate under the live class-gate env (flag-on): **197/0/0**.
- The fourteen selectors with the lane bind (`--ro-bind …/lane/p0543-tree-98c913453…
  …/stable-probe/active`): **14/0/0**.
- The eight mutation/observer selectors: **8/8**.
- The census selector: `1 passed; 0 failed`, the dump **`4b5fcd37…`**, the `ACCEPTANCE_BASELINE`
  line with `PUBLIC_WRAPPER=16`.
- The source-auth control with `RUM_CM740_ADDITIONS` = the rendered scene: green.
- The F48 pair: **2/0/0**.
- Every focused receipt is retained with its raw log; a new red or an unexpected pass stops with
  the names. The full `cargo test --workspace` is the **director's** — hand the rendered tree over
  with the focused receipts and a clean scope report.

## Stage D — the install and the re-freeze (6,000 s)

As 806b/806c: install the map into the live tree with the 797 semantics (190 paths, every live
parent rechecked, the witnesses, the identity overlays, the zero-overlay real promotion), run
`verify_install.py` and report `R4 LIVE VERIFY PASS …` with the rendered values; the whole-tree
delta allows `TODO.md` and the declared `docs/dev/codex/` extras only, each declared with path,
size and hash; the crates subtree must equal the rendered armed image exactly. Re-freeze the scene
and the counters.

## Stage F — the report (800 s)

The `rum-report/1` DONE report for brief 806: the render's deltas, the four declared cells, the
row-170 correction, the map update, the install and verify lines, the acceptance receipts, the
charge; the seal, `python3 -I scripts/report.py check 806`, the detached summary, and the exit
audit (the docs extras, the render's installed outputs with their produced pins, zero worker source
edits outside the render's outputs; zero unexplained).

## Charge

17,100 s of the 24,917.686 s remaining: A 300 / C 10,000 / D 6,000 / F 800. Every stage inside its
box; a box that would overrun stops at the deadline with the exact missing measurement. The
7,817.686 s reserve stays unallocated.

## STOP rules

As brief 806c: any checkpoint object mismatch, any scene cell outside the declared union, any new
red or unexpected pass, or any path delta outside the admitted set and the render's declared
outputs stops with the cell. Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354,
P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag;
no network; no new dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git.
