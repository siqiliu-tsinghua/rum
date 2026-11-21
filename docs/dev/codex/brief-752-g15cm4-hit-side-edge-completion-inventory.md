# Brief 752 — CM4 hit-side edge and cover-witness completion inventory (design-only, `Scope: NONE`)

**Director-dispatched, one stage, one gate, no production change.** This continues brief 751.
That brief derived nineteen own rows — the thirteen measured witnesses, P0339's 30 cover
candidates, four resume-edge fixes (P0338, P0435, P1070, P1818) and the P1142 stop — and
stopped because its scope admitted only resume-body cuts, while P1142's missing edge is the
**initial** call site: the row's queued occurrence enters the ordinary reader at
`crates/rum-stdlib/src/functional_ops/arrays/mod.rs:219`, the driven reader
(`options/core.rs:166`, producer `:185-192`, `ordinary` guard `:193`, hit `:196`) has its only
caller in `abort_cleanup.rs:104` for `CheckAbortOptionWorking`, and the unqueued read resumes
through `options/read_phase.rs:147` into the frozen continuation, bypassing the reader
entirely. The correct class is therefore **every hit-side incoming edge on the credited
execution, initial or resumed**; the historical C5 mutant wrapped the reader's own
`resolve_option_entry_value(&entry)` call at `options/core.rs:96` (`paired-calls.json`), and the
typed-own log shows `C2_ARM P1142 ArrayFilter.defaultOption:2` then
`C2_MUTANT_HIT P1142 ActualTypedCarrierBeforeOuterTail`, so the initial-edge dispatch is the
faithful re-derivation. Brief 751 also proved the partition is **139 files / 278 postimages**,
not the brief's fixed 137. `Scope: NONE`: inventory, cut list and revision list only; no
render, no source or selection edit, no build, no epoch.

## Goal

When this brief is done: all 151 own rows and the six dependency rows' 104 selected plans carry
one complete verdict — **static-witness**, **cover-candidate**, **hit-side-edge-fix** (with the
exact cut and its kind: initial call, typed-frame resume, continuation resume, or other), or
**stop with the exact missing fact** — derived from the frozen program, the fixture-declared
path and the sealed armed tree's code, with **zero unknown rows**; every edge fix names the
exact anchor, the rendered dispatcher shape and both falsifiers; the verified physical file
table and the `source_schema` rule are exact; and the director has the completed
candidate/revision list and the cost forecast. No live edit of any kind.

**Row-level stops.** A `stop` verdict is a row result, recorded with the exact missing fact;
it does **not** halt the scan of the remaining rows. Only a calibration failure (a known answer
derived wrongly) stops the brief.

## Context pointers

- Read `AGENTS.md`; `.temp/director/ruling-752.md` (this gate); `.temp/director/ruling-751.md`;
  `.temp/director/ruling-750.md`; brief 751
  (`docs/dev/codex/brief-751-g15cm4-resume-edge-dispatch-inventory.md`) and its scene
  `.temp/brief-751/` — especially `report.json`, `evidence/executed-edges-and-values.json`,
  `evidence/inventory-summary.json`, `evidence/added-resume-cuts.json`, `evidence/file-partition.tsv`,
  `evidence/dependency-plans.json`, `evidence/dependency-incoming-edges.json`,
  `evidence/P0339-candidate-proofs.json`, `evidence/historical-result-lines.jsonl`,
  `evidence/method-and-falsifiers.json`, `logs/174-design-proof.log`, `logs/187-publish-tables.log`.
  **Read-only.** Its nineteen dispositions are accepted inputs: re-verify, do not re-derive.
- Brief 750 and its scene `.temp/brief-750/`: `evidence/P0338.json`, `P0338-searched.tsv`,
  `P0339.json`, `candidates.tsv`, `cover-witnesses.json`, `dependencies.json`,
  `dependency-summary.tsv`, `revision-and-forecast.json`, `logs/205-p0338-exhaustive.log`,
  `logs/223-inventory-replay.log`.
- Brief 749 and its attempt-2 scene `.temp/brief-749/attempt-2/`: `report.json`,
  `evidence/scheduled-caller-audit.tsv`, `evidence/P0339-obstruction.json`,
  `evidence/parent-render.json`; attempt 1's report `.temp/brief-749/attempt-1-report.json`.
- Brief 748 and its scene `.temp/brief-748/`: `rows.json`, `repair-sites.tsv`,
  `repair-plan.json`, `calibration.json`, `root-cause.json`. The 748 graph is the call-graph
  specification; this brief adds the executed-edge and value dimensions.
- The sealed final epoch `.temp/brief-743/stage-b/epoch-b/` and its armed tree
  `default-armed-c739385a1e0ebe8e4b3d3e0a92b65d5d39bc82180081b2061c1e13ad157db874`:
  `programs.tsv`, `bindings.tsv`, `mutant-coverage.tsv`, `mutants.tsv`, `observer-policy.tsv`,
  the 255-per-mode selections, the fixture `fixtures/obligations/guest.rs`, the overlay table
  `mutations/source_auth/overlays.tsv` (135 files), and the copied source tree. **Read-only.**
- The live selection line: `.temp/brief-740/c1/assemble_lane_mutations.py`,
  `selected-own-tuples.json`, `.temp/brief-740/c2/selected-own-tuples.json`,
  `.temp/brief-740/c7/freeze_requirements.py`; the live inventory
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutant-coverage.tsv`.
  **Read-only.**
- The historical scenes: `.temp/g15-c2/c5/` (`prepare_typed.py`, `paired-calls.json`, the
  typed-own logs), `.temp/g15-c2/c7/` (`options-coverage.tsv`, the option logs),
  `.temp/g15-c2/c17/`, `.temp/g15-c2/c24/`, `.temp/g15-c2/c25/`, `.temp/g15-c2/c23/`; the c96
  Stage C packets `.temp/brief-743/stage-c/packets/`. **Read-only.**

## Scope (`Scope: NONE`)

- MAY create/modify: everything under `.temp/brief-752/` only.
- MAY read: the live source and inventory, the sealed scenes, the earlier `.temp/brief-*`
  records.
- MUST NOT: edit any live file — including the untracked inventory — and write under any
  sealed scene. No compile or build, no epoch or lane binary, no carriage of a new program, no
  git, no network, no new dependency, no `unsafe`. No row reclassified, no predicate weakened,
  no tuple changed in place, no hit moved: the cuts and the revisions are recommendations with
  their old→new records, applied by later authorized briefs.

## Deliverable

1. **The completed 151-row hit-side-edge and value table (TSV/JSON plus a summary).** For each
   own row: class, family, cover program and decoded tuple; the fixture-declared family path
   (`Input;tail=…;second=…`) or the derived family default; the pause plan and setup; the
   producer site; the hit site and its enclosing driven function; **every incoming call edge of
   that function and of each ordinary finisher on the credited path**, each classified
   **initial call / typed-frame resume / continuation resume / other**, with its `driven()`
   dispatch fact; the edge the credited occurrence actually traverses; the value at each
   non-frame guard conjunct; and the historical outcome. One verdict per row:
   - `static-witness` — the credited execution enters the mutated body under a dispatched edge
     and every non-frame guard conjunct provably holds;
   - `cover-candidate` — the value fails, and a program in the row's own bound matrix supplies
     the required value;
   - `hit-side-edge-fix` — a traversed edge calls the ordinary body while `driven()` is true;
     the exact edge, its kind and the cut are named, with the value proof under that cut;
   - `stop` — the path or value cannot be decided from the code and the frozen program; the
     exact missing fact is named. A stop is a row result; the scan continues.
   A row may carry both `hit-side-edge-fix` and `cover-candidate`; record both. The `frame(row)`
   and `ordinary(row)` authentication is the mechanism's and is measured in Stage B/C; these
   verdicts are the value and executed-edge conditions only. **Zero unknown rows.**
2. **The added cut list.** Carry the five proven cuts unchanged — P0338
   `definition_catalog/rewrite_rules.rs:450`; P0435 `definitions/part_assignment/resume_frames.rs:24`
   (new file); P1070 `kernel_data/structure_data/replace/guarded/resume_frames.rs:110`;
   P1818 `rum-stdlib/src/associations/mutation/resume_frames.rs:29` (new file); P1142
   `rum-stdlib/src/functional_ops/arrays/mod.rs:219` (parent row 116, already re-rendered by
   the P1937 cut at `:38`) — and add every further edge the completed table proves. Each cut:
   anchor `file:line` byte-exact against the sealed armed tree; the rendered dispatcher shape
   (`if crate::cm740_probe::driven() { <driven> } else { <the original call, textually
   unchanged> }`), inside the resume body or at the caller as the case is; the rows it serves;
   the whole-file recipe and its inversion (parent preimage, or the pinned live bytes for a new
   file); the incoming-edge withholding and the value negative; and the dispatch negative
   (else-only → NoHit). Do not add a cut for an edge no selected or candidate program requires.
3. **The verified physical table.** Derive, from the completed inventory: every file row with
   its status (re-rendered / new / preserved), the parent row and preimage where one exists,
   the pinned live sha256 for a new file, the postimage count (two per file, both modes), the
   exact total (currently 139 files / 278 postimages, including the 100 preserved files and the
   two 748 new files), and the `source_schema` rule — SHA-256 of the exact ordered
   `overlays.tsv` bytes; the exact digest after the authorized render, not before. State that
   the policy `3953aa9f…` and the sealed schema `22751919…` records do not change.
4. **The completed candidate table and revision list.** Every `cover-candidate` row's proven
   candidates from its own bound matrix only, the recommended replacement (least identity
   change), old and new program and tuple side by side, and the exact records the later edit
   touches and the ones it must not. P0339's 30 candidates and the `98c7b14d…` recommendation
   are accepted: re-verify, restate.
5. **The six dependency rows.** P0325 and P0326 (50 selections each, 45/5), C1/G15A/G15B/Some
   (5 plans over 5 roots): the same hit-side-edge and value derivation for every selected plan.
   Do not re-derive their own pair/structural bars; state whether any selected plan needs an
   added cut and keep the pair's `(45,5,false)` and the four controls' frozen bars intact.
6. **Method and calibration.** State the search space, the counting rules, and the derivation
   of the executed edge and the value. Calibrate on known answers: P1142 must come out
   `hit-side-edge-fix` at the initial call `arrays/mod.rs:219` (the C5 historical mutant at
   `options/core.rs:96`, the typed-own arm/hit, the C7 SAME line); P0338, P0435, P1070 and
   P1818 must stay `hit-side-edge-fix` at their resume-body anchors; the ten Stage C class
   rows, C10, P0001 and P0002 must stay `static-witness`; P0339 must stay `cover-candidate`
   with the same 30 and recommendation; the pair must stay 45/5 on each side and the four
   structural controls unchanged. A method that gets any calibration wrong stops the brief.
7. **The isolation falsifiers.** Per row: the incoming-edge withholding on whichever edge the
   credited occurrence traverses (initial or resumed) with the driven body intact; the dispatch
   negative for a proposed cut; the value negative; and for a cover candidate the old cover's
   failing value. State which are static and which belong to Stage B/C; a static verdict is
   never a catch.
8. **The revision list and forecast.** The exact old→new assignments; the exact added cuts; the
   files and records the later authorized edit touches and the ones it must not; the effect on
   the epoch (the selection set, the re-rendered row set and the new files only; the schema and
   policy records unchanged; the 20 class credits stand; the 95 reachable images and 13
   PortBlocked bindings preserved); the measured cost basis for brief 753 (the cover revision),
   the 749 attempt-3 render of 49 + N cuts, and a re-freeze; and what Stage B, Stage C and the
   744 restart would carry on the revised covers and the added cuts.
9. **Zero-unknown statement.** Explicitly: 151 own rows + six dependency rows, every verdict
   with `file:line` and a value fact, no row settled by argument, no row left without a verdict
   or an exact stop fact.

## Hard rules

- `Scope: NONE` of production and of the inventory. No live edit of any kind (tracked or
  untracked), no build, no cargo that compiles, no epoch binary, no lane, no git, no network,
  no `unsafe`, no new dependency. Bounded read-only commands only, charged to this brief's
  carve; scratch in `.temp/`, never `/tmp`.
- The standing C2/pair/owner state stays untouched: pair `(45,5,false)` Open at zero admitted
  receipts; C2 118 Residual + 0 Lost; the 51 owners and five source duties Open; membership
  28/3/7/113; `CHANGELOG.md` and S10 blocked; CM5 unauthorized; everything sealed (c76–c99)
  stays sealed and read-only; 745/746 held.
- Never weaken a predicate, never reclassify a row, never exempt a row, never move a hit to a
  favorable branch, never count a byte or text identity as a witness. The 748 static
  `reachable` verdict is not a witness certificate for any row. An edge fix binds an incoming
  edge on the same credited activation the historical mutant executed; it does not change the
  hit, the predicate or the row identity.
- Paste verbatim evidence for every verdict (file:line, decoded tuple, the fixture path
  component, the exact value and guard, the historical result line). Save every command's full
  output under `.temp/brief-752/` before parsing.
- A candidate or a cut chosen by preference over a proven condition is not a candidate. If a
  row's edge or value cannot be decided, give it a `stop` with the exact missing fact and
  continue the scan.

## Verification

- 151 own rows with verdicts, zero unknown; every verdict cites the decoded tuple, the program
  text, the fixture path, the hit's enclosing function, the traversed edge with its kind and
  dispatch fact, and the value fact; the calibrations match (P1142 initial-edge fix; the four
  resume-edge fixes; the thirteen measured witnesses; P0339's 30; the pair 45/5 and the four
  structural controls).
- Every cut's anchor matches the sealed armed tree byte-exactly, the rendered shape keeps the
  original call textually unchanged, the recipe inverts, and the three negatives are stated;
  the physical table is derived, exact and consistent with the postimage count; the
  `source_schema` rule is stated.
- The revision list names old and new programs and tuples and the exact records; it changes no
  program, mutant, policy, schema or image; the class credits and the 13 PortBlocked bindings
  are untouched.
- The isolation falsifiers are stated per row, and the static/measured boundary is explicit.
- Budget: 3,000 s focused from the stage reserve, 0 workspace, 0 epoch; every bounded command
  debits by monotonic elapsed time; a projected overrun stops for a director ruling, never a
  scope cut.

## Report format

`rum-report/1` at `.temp/brief-752/report.json`; the completed hit-side-edge and value table,
the added cut list, the verified physical table, the candidate table and revision list, the six
dependency rows, the method and calibration, the isolation falsifiers, the revision list and
forecast, the zero-unknown statement, the Open ledger state; `changes` empty. Save every
command's full output in the scene; cite lines exactly. Before finishing run
`python3 scripts/report.py check 752` (without `--against-git`) and fix every PROBLEM; the
final message is only `python3 scripts/report.py summary 752`. STOP at the inventory and the
lists: no edit, no render, no dispatch of 753, 749 attempt 3, Stage B/C or 744/745/746.
