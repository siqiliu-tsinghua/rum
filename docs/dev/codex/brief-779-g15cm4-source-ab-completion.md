# Brief 779 — G15-CM4 Stage D repair: the source A/B completion — the drift-path refresh, the rebuilt baseline and the complete six-case gate

**Implementation brief, staged (A–D), one gate.** It continues ruling 787 (gate c137) and
brief 778 (`94b042a8`). Brief 778 repaired the loader's seal anchor, re-anchored the four
artifacts through the single writer and verified stages A–C; its stage D stopped correctly on
the drift assertion: `TODO.md` in the carried install equals the baseline but not live,
because the director's own c137 gate commit inserted the continuation-137 entry into live
after brief 777's snapshot. Ruling 787 answers the question: **refresh only `TODO.md` from
live**. This brief carries the repaired install tree, refreshes that one inert doc, rebuilds
the baseline and the three images, and runs the complete six-case source gate and the
deferred 774 acceptance. A front-half stop does not skip later evidence by shrinking it.

## Goal

When this brief is done, in `.temp/brief-779/` only:

- The carried install tree (from `.temp/brief-778/install`, tree `8002e2ec…`, 5,850 files)
  with exactly **one** further declared change: `TODO.md` is byte-copied from the live tree
  (old `c7e056fe…`, new `c3decd6b…`). Nothing else changes; the six carry edits
  (`guest.rs` `3decae58…`, `tests.rs` `f6de0250…`, `overlays.tsv` `2a854902…`,
  `mutants.tsv` `1cab55ca…`, `emission_chain.tsv` `c2aee404…`, `additions.tsv`
  `acb46cdf…`) stay byte-identical. The live tree, `.temp/brief-777/` and
  `.temp/brief-778/` stay byte-untouched.
- The baseline rebuilt from the 779 install: install − the eight additions; the 147 restored
  only from digest-checked live preimages; the six drift paths at live bytes on both sides
  **and** each differing from the 749 source copy; the two tables on both sides; the key set
  `baseline ⊎ additions == install`; non-overlay content equal; the baseline `guest.rs` is
  the live preimage `a07af227…`.
- The three images: `armed` = the install tree; `disabled` = the install with the three
  arm-differing files at their decoded disabled cells; `real` = the baseline ⊎ the eight
  additions (non-addition bytes equal to the baseline's, each addition at its pin) — the
  shipped live-lineage configuration plus the new class files, not the armed tree.
- The complete six-case source gate per brief 778's stage E: six release builds, six
  receipts, the validator controls on all six, the copied admission on the disabled and
  armed images in both configurations with the two exact PASS lines, the anchor falsifier,
  the observer capability, the negative pairings, and the six evidence cases recorded
  completely.
- The deferred 774 acceptance per brief 778's stage F.

Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Context pointers

- Ruling `.temp/director/ruling-787.md`; brief 778 (`docs/dev/codex/brief-778-g15cm4-source-ab-continuation.md`,
  committed `94b042a8`) — its stage E ("The six-case source gate") and stage F ("The deferred
  774 acceptance") are **normative** for this brief's stages C and D — and its report
  `.temp/brief-778/report.json`, `PROGRESS.md` and logs under `.temp/brief-778/logs/`.
- The carry: `.temp/brief-778/install/` — tree `8002e2ec…` / 5,850 files (the director's own
  walk; reproduce it with the walker the 778 stage scripts used, read-only under
  `.temp/brief-778/`); the six digests above; the class table `8c8b2232…` (4,803,954 B,
  byte-unchanged); every overlay preimage equals its live file; the eight addition digests
  as `additions.tsv` pins them. The 778 stage-D partial baseline
  (`.temp/brief-778/baseline/`) is **discarded**; rebuild from the 779 tree.
- The six untouched drift paths, with the live pin this brief expects and the 749 source pin
  (the 749 pins are rows of `.temp/brief-770/logs/tree-766.tsv`, read-only):

  | path | live pin | 749 pin |
  | --- | --- | --- |
  | `CLAUDE.md` | `9c5f5d0b…` | `055124f1…` |
  | `TODO.md` | `c3decd6b…` (13,395 lines) | `a546f94d…` |
  | `docs/dev/codex/brief-623-g15c2-settle-unowned-census-rows.md` | `bd78e4bc…` | `7378d67d…` |
  | `docs/dev/codex/brief-743-g15cm4-observation-policy.md` | `80a9b564…` | `f4b01a90…` |
  | `tools/claude-mod/rum-director/.claude-plugin/types/claude-code-tools/index.d.ts` | `983fd875…` | `43e78046…` |
  | `tools/claude-mod/rum-director/.claude-plugin/types/claude-code/index.d.ts` | `aec7b446…` | `3cbd3402…` |

  If a live pin does not match at entry, stop: the director does not commit while a brief is
  in flight, so a moved file would be an unrecorded change.
- The live-only brief docs under `docs/dev/codex/` (`brief-777-…`, `brief-778-…`,
  `brief-779-…`) are declared non-install paths, as before.
- The diagnosis and repair lineage: ruling 786 (F6/F7/F8), brief 778's stage B (the
  four-anchor KAT) and stage C (the single-writer re-anchoring). The verifier extension and
  the addition-set rule are from brief 777.

## Scope

- You MAY write only under `.temp/brief-779/`: `install/`, `baseline/`, `images/`, the
  scripts, the logs, the evidence and the report. The live tree, `.temp/brief-777/`,
  `.temp/brief-778/`, `.temp/brief-774/scene`, `.temp/brief-770/`, `.temp/brief-766/`,
  `.temp/brief-768/` and `.temp/brief-769/` are read-only; `.temp/brief-774/receipt_diff.py`
  is run, never edited.
- You MAY change, **in the 779 install tree only**:
  - `TODO.md` — a byte copy of the live file in stage B; never an edit;
  - nothing else. No other file may change; a disclosed extra stops for a ruling.

## Stage A — entry and copy (400 s)

- Re-authenticate the carry: tree `8002e2ec…`, 5,850 files; the six digests; the class table
  `8c8b2232…`; every preimage cell equal to its live digest; the eight additions as
  `additions.tsv` pins them. Quote each with a line.
- Record the six live drift digests (the table above) and the six 749 pins.
- Copy `.temp/brief-778/install/` to `.temp/brief-779/install/`; assert the copy's hash and
  file count equal the carry, and re-assert the 778 tree's hash after the copy (it stays
  byte-untouched).

## Stage B — the refresh, the baseline and the images (250 s)

- **The refresh**: copy the live `TODO.md` byte-for-byte onto the 779 install; record the old
  and new digests and the new tree hash; assert the install's delta against the carry is
  exactly seven paths: the four artifacts, the two source edits and `TODO.md`.
- **The baseline**: copy the install; delete the eight additions; restore each of the 147
  overlay files from the live file only after its digest equals the table's preimage; assert
  the six drift paths at live bytes on both sides **and** each differing from its 749 pin;
  the two tables on both sides; `baseline ⊎ additions == install` (keys); non-overlay
  content equal; the baseline `guest.rs` is the live preimage `a07af227…`; record both tree
  hashes.
- **The disabled image**: the install with `crates/rum-runtime/src/lib.rs`,
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` and
  `crates/rum-runtime/src/runtime/profiling/logical/segments.rs` replaced by the decoded
  disabled cells from `overlays.tsv`; re-hash; assert each file's digest equals its disabled
  cell.
- **The armed image**: a complete copy of the install (no symlinks).
- **The real image**: a complete copy of the baseline plus the eight additions at their pins;
  assert the non-addition bytes equal the baseline's; record its tree hash.

## Stage C — the six-case source gate (1,900 s)

Brief 778's stage E section is normative and unchanged; run it over this tree:

- six release builds through the 743 adapter over `scripts/probe-build.py` (release,
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, stable `.temp/probe/active`), serially, one per
  (configuration, image) — `default` and `no-default` × `real`, `disabled`, `armed`;
- six receipts with the field lines sorted (`arm`, `baseline_tree`, `binary`,
  `codegen_units=1`, `configuration`, `profile=release`, `tree`); the `real` receipt carries
  `arm=real` and **no** overlay lines (a build receipt; `verify` requires the 147 overlay
  keys, so it never authenticates — do not attempt); the disabled/armed receipts carry the
  147 overlay rows equal to the table's pins and cells;
- `cm_mutation_source_authentication_controls` on all six images; the copied admission on
  the disabled and armed images in both configurations with exactly
  `CM4 COPIED SOURCE ADMISSION PASS arm=disabled programs=9060 P0543=55 strict_loader=SourceIdentity`
  and `CM4 COPIED SOURCE ADMISSION PASS arm=armed programs=9060 P0543=55 strict_loader=class_form_ok`,
  plus the anchor falsifier's line;
- `cm_mutation_observer_copied_capability` on the armed images;
- the negative pairings exactly as brief 778 specifies (disabled receipt against the armed
  tree → `WrongArm`; armed receipt against the disabled tree → `Postimage`; one wrong
  overlay digest → `Postimage`; one addition removed → `UnlistedFile`);
- wherever a run reads `RUM_CM740_BASELINE`, bind it to the 779 constructed baseline and
  quote the binding;
- the six evidence cases recorded **completely** — no FLAGGED evidence; a case that cannot
  run stops.

## Stage D — the deferred 774 acceptance (700 s focused + the workspace bucket)

Brief 778's stage F section is normative and unchanged: F3's fresh flag-on class-lease run
on the armed image under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`
(`test result: ok. 197 passed; 0 failed; … 592 filtered out`) and the receipt diff
(`problems=0 digest=PASS`) plus its selftest; F4's corrected line re-quoted from the carry
(byte-unchanged); the flag-off workspace run with the two named skips and
`RUM_CM740_BASELINE`; `cargo fmt --check`; every red classified against the 776 stage-D
reference set, any other red stops.

## Verification and acceptance

The brief passes only with all of: the carry's hash equality; the seven-path delta with the
refresh byte-exact; the six drift paths at live bytes on both sides and each differing from
its 749 pin; the baseline keys/content and the live-preimage `guest.rs`; the three images'
assertions and tree hashes; the six-case gate complete with the two exact PASS lines and the
anchor falsifier; F3's `problems=0 digest=PASS` and selftest; the workspace's fully
classified red list; and the fmt state. Quote every verdict line verbatim in the report;
disclose every deviation.

## Checkpointing and stops

Checkpoint after every stage (`.temp/brief-779/PROGRESS.md`, logs under
`.temp/brief-779/logs/`). Stop, with the exact evidence, on: a carry pin mismatch; a live
drift pin mismatch; an install byte beyond `TODO.md`; a refresh copy that is not
byte-identical; a drift assertion failure (install ≠ live, or live == the 749 pin); a
key/content mismatch; `Postimage`, `Preimage`, `UnlistedFile`, `BaselineSeal`, `Inventory`
or `OverlaySet` beyond the declared negatives; a six-case run that fails with any error other
than the declared negative pairings; a workspace red outside the reference set; a fmt red
outside the two Q25 carrier files; or a projected overrun. A stop is a correct terminal
state; never weaken a rule, skip a file, or copy one arm onto the other to pass.

## Hard rules

Never weaken `Preimage`, `Postimage`, `BaselineSeal`, `Inventory`, `OverlaySet` or
`UnlistedFile`; never reseal or edit `class-provenance.tsv`; never change `source()` or its
route test; never hand-write overlay hex (the writer derives every cell from a tree); never
edit `TODO.md`'s content (a byte copy only); never use the 749 source copy's tables; never
take a preimage from anything but the live tree after the digest check; never touch the live
tree or the sealed scene; no receipt from synthetic data; no importable receipt; Q25 stays
open; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

Focused **3,250 s** (A 400 / B 250 / C 1,900 / D 700) + workspace **2,700 s** + epoch **0**,
carved from the stage reserve after brief 778 (16,080.92278847017 → **10,130.92278847017 s**;
778 charged 2,266.1 s of its focused 5,500 s and returned 3,233.9 s; its workspace 2,700 s
did not run). Unspent returns; a projected overrun stops for a director ruling, never a scope
cut.

## Report format

`rum-report/1` at `.temp/brief-779/report.json`, appended as stage `A / c138`; `summary`,
`changes`, `deviations`, `questions`, the six evidence cases, every verdict line quoted, and
the final message `scripts/report.py summary`. The director gates from raw evidence.

## Gate c137 note (for the reader)

Brief 778's stages A–C are accepted from raw evidence: the loader repair is the four-anchor
KAT byte-exact, the tables carry exactly one content delta, and the verifier's controls are
green. Its stage D stopped correctly: the six drift paths are all non-source bookkeeping, and
`TODO.md` alone moved because the director's c137 gate commit wrote the continuation-137
entry into live after the 777 snapshot (F9). Ruling 787's answer is the refresh: an inert doc
pinned by no artifact, whose stale bytes a whole-tree adoption could otherwise copy backwards
over the live file. Holding the 777 bytes was rejected: it would leave the install's drift
invariant false at check time and add a permanent false delta to every full-tree comparison.
After this brief: the probe and hot-path A/B, the deterministic render (F5, the Q25
formatting, the inventory regeneration, the full workspace green, the source-gate re-run),
the re-freeze, the fresh epoch, the Stage C/C10 readmission and a further 744 restart, each
gated.
