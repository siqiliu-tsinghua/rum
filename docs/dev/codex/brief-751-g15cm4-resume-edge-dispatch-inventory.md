# Brief 751 — CM4 resume-edge dispatch and cover-witness completion inventory (design-only, `Scope: NONE`)

**Director-dispatched, one stage, one gate, no production change.** This continues brief 750.
That brief derived fifteen own rows and stopped at P0338 under its own rule ("a row with no
candidate stops with that row"): every one of P0338's 55 bound programs suspends at the
producer, and the shared suspension frame `ResolveRewriteRuleExprsOwnedSuspensionFrame`
(`rewrite_rules.rs:433`) resumes every one of them into the **ordinary** mapper
`normalize_rewrite_rule_exprs_from_resolved` (`:450`, defined `:291-300`), while the row's only
hit (`:315`) lives in the driven mapper `normalize_rewrite_rule_exprs_from_resolved_cm740_driven`
(`:306`), whose only caller is the immediate arm at `:213`. The fixture itself declares that
resumed tail (`guest.rs` P0338 entry: `tail=ResolveRewriteRuleExprsOwnedSuspensionFrame>normalize_rewrite_rule_exprs_from_resolved`),
and the historical C25 mutant patched that ordinary mapper in place
(`.temp/g15-c2/c25/prepare_rule_guards.py:22,28-29`), which is why the historical row was caught
on resume. So this is not a cover-selection defect: no program in P0338's own matrix can
witness while the resumed edge stays ordinary, and 30 of the 55 (including the frozen cover)
would witness as soon as the resumed edge dispatches to the driven mapper under `driven()`.
That is a third defect class — mutation placement off the credited resumed path — and it must be
inventoried class-wide, exactly as the earlier rounds inventoried the dispatch gap and the cover
gap. `Scope: NONE`: inventory, cut list and revision list only; no render, no source or
selection edit, no build, no epoch.

## Goal

When this brief is done: every one of the 151 own rows and every one of the six dependency
rows' selected programs carries one complete executed-path verdict — **static-witness**,
**cover-candidate**, **resume-edge-fix**, or **stop with the exact missing fact** — derived from
the frozen program, the fixture-declared path and the sealed armed tree's code, with **zero
unknown rows**; every cover-candidate row has its proven replacement tuple and program; every
resume-edge-fix row names the exact incoming edge, the cut that binds it, and the value proof
under that edge (and a row may carry both duties, with both records); the added cut list is
exact and renderable (anchor, rendered shape, rows served, inversion, withholding negative,
value negative); and the director has the completed revision list, the recomputed file/row
partition and the cost forecast. No live edit of any kind.

## Context pointers

- Read `AGENTS.md`; `.temp/director/ruling-751.md` (this gate); `.temp/director/ruling-750.md`;
  `docs/dev/codex/brief-750-g15cm4-cover-witness-inventory.md` and its scene
  `.temp/brief-750/` — especially `report.json`, `evidence/P0338.json`,
  `evidence/P0338-searched.tsv`, `evidence/P0339.json`, `evidence/candidates.tsv`,
  `evidence/cover-witnesses.json`, `evidence/cover-witness-summary.tsv`,
  `evidence/dependencies.json`, `evidence/dependency-summary.tsv`,
  `evidence/revision-and-forecast.json`, `logs/205-p0338-exhaustive.log`,
  `logs/223-inventory-replay.log`. **Read-only.** Its fifteen derived rows and 105 dependency
  plans are accepted inputs: re-verify, do not re-derive from scratch.
- Brief 749 (`docs/dev/codex/brief-749-g15cm4-carrier-dispatch-repair-stage-a.md`) and its
  attempt-2 scene `.temp/brief-749/attempt-2/`: `report.json`,
  `evidence/scheduled-caller-audit.tsv`, `evidence/P0339-obstruction.json`,
  `evidence/parent-render.json`; attempt 1's report `.temp/brief-749/attempt-1-report.json`.
- Brief 748 and its scene `.temp/brief-748/`: `rows.json` (the 151-row matrix with per-site
  `enclosing_reachable` and `reachable_with_this_binding`), `repair-sites.tsv` and
  `repair-plan.json` (the 49 cuts and their operation text), `calibration.json`,
  `root-cause.json`. The 748 graph is the call-graph specification; this brief adds the
  executed-edge and value dimensions.
- The sealed final epoch `.temp/brief-743/stage-b/epoch-b/` and its armed tree
  `default-armed-c739385a1e0ebe8e4b3d3e0a92b65d5d39bc82180081b2061c1e13ad157db874`:
  `programs.tsv` (9,060 bound programs), `bindings.tsv`, `mutant-coverage.tsv`, `mutants.tsv`,
  `observer-policy.tsv`, the 255-per-mode selections, the fixture `fixtures/obligations/guest.rs`
  (the per-row family path with its `Input;tail=…;second=…` components and the projections), and
  the copied source tree including `definition_catalog/rewrite_rules.rs`. **Read-only.**
- The live selection line: `.temp/brief-740/c1/assemble_lane_mutations.py` (the `own[0]`
  fallback), `.temp/brief-740/c1/selected-own-tuples.json`, `.temp/brief-740/c2/selected-own-tuples.json`,
  `.temp/brief-740/c7/freeze_requirements.py`; the live inventory
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutant-coverage.tsv`.
  **Read-only.**
- The historical scenes: `.temp/g15-c2/c23/` (P0339) and `.temp/g15-c2/c25/`
  (`prepare_rule_guards.py`, the raw logs and full proofs); `.temp/brief-741/row-taxonomy.tsv`,
  `ROW-EVIDENCE.md`, `SIGNATURES.md`; the c96 Stage C packets
  `.temp/brief-743/stage-c/packets/` (arm-before-hit calibration). **Read-only.**
- Precedents for the added cut shape in the 748 plan: `rhs_guards/resume_frames.rs:215/226`,
  `tree_walkers/count_like.rs:530`, `tree_walkers/position.rs:177` — "route both immediate
  return and independently resumed traversal at their shared private destination; restoring
  only the upstream immediate path leaves the separately stored typed frame on the original."

## Scope (`Scope: NONE`)

- MAY create/modify: everything under `.temp/brief-751/` only.
- MAY read: the live source and inventory, the sealed scenes, the earlier `.temp/brief-*`
  records.
- MUST NOT: edit any live file — including the untracked inventory (`mutant-coverage.tsv`,
  `programs.tsv`, `mutants.tsv`, the selections, the policy, the schema) — and write under any
  sealed scene. No compile or build, no epoch or lane binary, no carriage of a new program, no
  git, no network, no new dependency, no `unsafe`. No row reclassified, no predicate weakened,
  no tuple changed in place, no hit moved: the cuts and the revisions are recommendations with
  their old→new records, applied by later authorized briefs.

## Deliverable

1. **The 151-row executed-edge and value table (TSV/JSON plus a summary).** For each own row:
   class, family, cover program and decoded tuple; the fixture-declared family path (its
   `Input;tail=…;second=…` components) or, where the row has no per-row entry, the derived
   family default with the derivation; the program's pause plan and setup; the producer wrapper
   site; the hit site and its enclosing function (the `_cm740_driven` function in the copied
   armed tree); **every incoming edge of that function** in the copied tree, each classified
   immediate arm / typed-frame resume / continuation resume / other; the edge the row's credited
   occurrence actually traverses; the `driven()` dispatch fact on that edge (the exact calls
   that run under the gate); the value at each non-frame guard conjunct on the credited
   execution (the program's own answer, the normalizer's result, the guard list, the rule
   count); and the historical outcome for the same tuple (`*-full-proofs.json`, raw-log lines).
   One verdict per row:
   - `static-witness` — the credited execution enters the mutated function under the driven
     dispatch and every non-frame guard conjunct provably holds;
   - `cover-candidate` — the mutated function is entered, but the frozen cover's value fails a
     guard, and a program in the row's own bound matrix supplies the required value;
   - `resume-edge-fix` — the credited execution reaches the mutated function only through an
     edge that calls the ordinary twin while `driven()` is true; the exact edge and the added
     cut are named, and the value proof is given under that cut;
   - `stop` — the path or value cannot be decided from the code and the frozen program; the
     exact missing fact is named. **An unknown verdict is a stop, not a row.**
   A row may carry both `resume-edge-fix` and `cover-candidate`; record both. The
   `frame(row)` authentication remains the mechanism's and is measured in Stage B/C: the
   verdicts here are the value and executed-edge conditions only. **Zero unknown rows.**
2. **The added cut list.** For every `resume-edge-fix` edge (and every edge any selected or
   candidate program traverses): the anchor `file:line` in the copied tree, byte-exact against
   the sealed armed tree; the rendered dispatcher shape (`if crate::cm740_probe::driven() { …
   _cm740_driven(…) } else { <the original call, textually unchanged> }`), placed inside the
   frame/continuation resume body; the rows it serves; the whole-file recipe and its inversion
   to the live preimage; the incoming-edge withholding (suppress the cut's driven arm → the
   row's hit unreachable on its credited path) and the value negative (a program without the
   required guard value → NoHit). State the recomputed file/row partition: which 135-row
   overlay rows move from preserved to re-rendered (starting with
   `definition_catalog/rewrite_rules.rs`, row 25, preimage `bef3ceb6…`), the new count of
   re-rendered + new + preserved rows (137 total), the postimage count, and the new
   `source_schema` (and that the policy `3953aa9f…` and schema `22751919…` records do not
   change). Do not add a cut for an edge no selected or candidate program requires:
   over-generalized scope is a stop.
3. **The completed candidate table and revision list.** For every `cover-candidate` row: the
   proven candidates from that row's own bound programs only, each with the tuple it carries,
   the value proof, the historical result, and the recommended replacement (least identity
   change: same family, variant, route and descriptor where one exists, then smallest tuple
   change), with old and new program and tuple side by side and the exact records the later
   edit touches (`mutant-coverage.tsv`, `selected-own-tuples.json`, the frozen schedule) and the
   records it must not (programs, mutants, observer policy, schema, overlay images). P0339's
   30 candidates and the `98c7b14d…` recommendation are accepted inputs: re-verify, restate.
   A row with no candidate does not stop this brief if it has a `resume-edge-fix` or a `stop`;
   the stop rule applies only to a row with neither a witness nor a proven fix.
4. **The six dependency rows.** P0325 and P0326 (50 selections each, the pair's 45/5), and
   C1/G15A/G15B/Some (5 plans over 5 roots): the same executed-edge and value derivation for
   each selected program, with the dispatch dimension. Do not re-derive the pair or structural
   bars — they keep their own falsifiers — but state whether any selected program's path needs
   an added cut, and keep the pair's `(45,5,false)` and the four controls' frozen bars intact.
5. **Method and calibration.** State the search space, the counting rules, and the derivation
   of the executed edge and the value. Calibrate on known answers: P0338 must come out
   `resume-edge-fix` at `ResolveRewriteRuleExprsOwnedSuspensionFrame`'s `:450` ordinary call
   (the fixture tail, the sole driven-mapper caller at `:213`, the lone hit at `:315`, the
   historical ordinary patch); P0339 must come out `cover-candidate` with 30 candidates and the
   same recommendation; the ten Stage C class rows, C10, P0001 and P0002 must stay
   `static-witness` (the c96 packets' arm-before-hit order and the two retention capacity
   branches); the pair must stay 45/5 on each side and the four structural controls unchanged.
   A method that gets any calibration wrong is not usable: say so and stop.
6. **The isolation falsifiers.** Per row: the incoming-edge withholding on whichever edge the
   row's credited occurrence traverses (suppress it, keep the whole driven body → the row's
   producer or hit unreachable), the value negative (the failing value → static-non-witness /
   NoHit), and for a resume-edge-fix row the dispatch negative (the cut's else arm only →
   NoHit). State which falsifiers are static (code) and which belong to Stage B/C; a static
   verdict is never a catch — the campaign's armed measurement confirms it, and a static
   witness that measures NoHit is a finding against the audit.
7. **The revision list and forecast.** The exact old→new assignments for every cover-candidate
   row; the exact added cuts; the files and records the later authorized edit touches and the
   ones it must not; the effect on the epoch (the selection set and the re-rendered row set
   only; the schema `22751919…` and policy `3953aa9f…` unchanged; the 20 class credits stand;
   the 95 reachable images and 13 PortBlocked bindings preserved); the measured cost basis for
   the authorized cover revision (752), the 749 attempt-3 render of 49 + N cuts, and any
   re-freeze; and what Stage B, Stage C and the 744 restart would carry on the revised covers
   and the added cuts.
8. **Zero-unknown statement.** Explicitly: 151 own rows + six dependency rows, every verdict
   with `file:line` and a value fact, no row settled by argument, no row left without a verdict
   or a stop.

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
  `reachable` verdict is not a witness certificate for any row. A resume-edge fix binds an
  incoming edge on the same credited activation the historical mutant executed; it does not
  change the hit, the predicate or the row identity.
- Paste verbatim evidence for every verdict (file:line, decoded tuple, the fixture path
  component, the exact value and guard, the historical result line). Save every command's full
  output under `.temp/brief-751/` before parsing.
- If a row's executed edge or value cannot be decided from the code and the frozen program,
  STOP with that row and the exact missing fact — never a plausible reading. A candidate or a
  cut chosen by preference over a proven condition is not a candidate.

## Verification

- 151 own rows with verdicts, zero unknown; every verdict cites the decoded tuple, the program
  text, the fixture path, the hit's enclosing function, the traversed edge with its dispatch
  fact, and the value fact; the calibrations match (P0338 resume-edge-fix at the shared frame's
  `:450` ordinary call, P0339 cover-candidate with 30 and the recommendation, the thirteen
  measured witnesses, the pair 45/5 and the four structural controls).
- Every added cut's anchor matches the sealed armed tree byte-exactly, the rendered shape keeps
  the original call textually unchanged, the recipe inverts to live, and both falsifiers are
  stated; the recomputed file/row partition and the new `source_schema` are stated.
- The revision list names old and new programs and tuples and the exact records; it changes no
  program, mutant, policy, schema or image; the class credits and the 13 PortBlocked bindings
  are untouched.
- The isolation falsifiers are stated per row, and the static/measured boundary is explicit.
- Budget: 3,000 s focused from the stage reserve, 0 workspace, 0 epoch; every bounded command
  debits by monotonic elapsed time; a projected overrun stops for a director ruling, never a
  scope cut.

## Report format

`rum-report/1` at `.temp/brief-751/report.json`; the executed-edge and value table, the added
cut list, the candidate table and revision list, the six dependency rows, the method and
calibration, the isolation falsifiers, the revision list and forecast, the zero-unknown
statement, the Open ledger state; `changes` empty. Save every command's full output in the
scene; cite lines exactly. Before finishing run `python3 scripts/report.py check 751` (without
`--against-git`) and fix every PROBLEM; the final message is only
`python3 scripts/report.py summary 751`. STOP at the inventory and the lists: no edit, no
render, no dispatch of 752, 749 attempt 3, Stage B/C or 744/745/746.
