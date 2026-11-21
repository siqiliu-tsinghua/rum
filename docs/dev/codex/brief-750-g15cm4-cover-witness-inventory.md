# Brief 750 — CM4 cover-witness inventory (design-only, `Scope: NONE`)

**Director-dispatched, one stage, one gate, no production change.** This answers
`Q749-P0339-frozen-hit`: brief 749 Stage A attempt 2 rendered cut 5 correctly and bound
`resume_normalize_rules_after_rule_exprs`'s only caller, but P0339's frozen cover program
(`CM739.batch2.P0339.00596175b0039980bc434cda`, answer `Hold[c23lhs->91]`) supplies `None` to
the NormalizeRules continuation, and the row's sole hit is nested under `Some` and
`rules.len() == 1`, so the hit cannot fire on the row's own selection under any caller binding.
The cover was chosen by the 740 c1 selection rule's `own[0]` fallback
(`.temp/brief-740/c1/assemble_lane_mutations.py:11`) — 78 of the 151 own covers are not the
preferred `Middle/phase_observation.trace` program — and it was never witness-checked. A cover
that cannot reach its row's hit leaves that mutation unproven by construction, whatever the
dispatch repair does. This brief is the class-wide inventory that establishes, for every own
row, whether its frozen cover is a static witness, and — where it is not — which programs in
that row's own bound matrix are, with the value proof. `Scope: NONE`: inventory and candidate
list only; no render, no selection change, no build, no epoch.

## Goal

When this brief is done: every one of the 151 own cover programs has a verdict —
**static-witness** or **static-non-witness**, with the exact value/guard fact in the armed
tree's code and the program's own text — and **zero unknown rows**; every non-witness row has
one or more proven witness candidates drawn only from that row's own bound programs in the
sealed `programs.tsv`, each with the same derivation and the tuple it would replace, or the
brief stops with that row; the six pair/structural dependency rows carry the same per-row
derivation for their selected programs; the method is calibrated against known answers; and the
director has the exact revision list, its isolation falsifiers and its cost. No source edit, no
selection edit, no build, no epoch binary run, no reclassification and no predicate change.

## Context pointers

- Read `AGENTS.md`; `.temp/director/ruling-750.md` (this gate); `docs/dev/codex/brief-749-g15cm4-carrier-dispatch-repair-stage-a.md`
  and its attempt-2 scene `.temp/brief-749/attempt-2/` — especially `report.json`,
  `evidence/scheduled-caller-audit.tsv`, `evidence/P0339-obstruction.json`,
  `evidence/retention-census.json`, `logs/085-verifier-replay.log` and `specimens/` — and
  attempt 1's report `.temp/brief-749/attempt-1-report.json`. Read-only.
- Brief 748 (`docs/dev/codex/brief-748-g15cm4-carrier-dispatch-reachability.md`) and its scene
  `.temp/brief-748/`: `rows.json` (the 151-row matrix, the 62 affected rows, sites, selections),
  `repair-sites.tsv` and `repair-plan.json` (the 49 cuts), `calibration.json`, `root-cause.json`.
  The 748 verdicts are the call-graph specification; this brief adds the value path, it does not
  re-litigate them.
- The selection line: `.temp/brief-740/c1/assemble_lane_mutations.py` (the rule
  `choice = next((p for p in own if p['variant'] == 'Middle' and p['context'] == 'phase_observation.trace'), own[0])`),
  `.temp/brief-740/c1/selected-own-tuples.json`, `.temp/brief-740/c2/selected-own-tuples.json`,
  `.temp/brief-740/c7/freeze_requirements.py` (the `changed == ['P2593']` revision precedent);
  the live coverage table `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutant-coverage.tsv`
  (151 own + 100 pair + 4 structural rows). **Read-only.**
- The sealed final epoch `.temp/brief-743/stage-b/epoch-b/` and its armed tree
  `default-armed-c739385a…`: `programs.tsv` (9,060 bound programs), `bindings.tsv`,
  `mutant-coverage.tsv`, `mutants.tsv`, `observer-policy.tsv`, the 255-per-mode selections, the
  fixture `fixtures/obligations/guest.rs` (the per-row program source expressions and the
  pause-plan table), `native.rs`, and the lane registrations. **Read-only.**
- The historical evidence: `.temp/g15-c2/c23/` and sibling family scenes (the campaign raw logs,
  `*-full-proofs.json`, `factory.py`); `.temp/brief-741/row-taxonomy.tsv`, `ROW-EVIDENCE.md`,
  `SIGNATURES.md`; `.temp/brief-739/` and `.temp/brief-740/c7/selected-carrier-provenance.json`
  (the retained per-row carrier schema and tail digests).

## Scope (`Scope: NONE`)

- MAY create/modify: everything under `.temp/brief-750/` only.
- MAY read: the live source and inventory, the sealed scenes, the earlier `.temp/brief-*`
  records.
- MUST NOT: edit any live file — including the untracked inventory (`mutant-coverage.tsv`,
  `programs.tsv`, `mutants.tsv`, the selections, the policy, the schema) — and write under any
  sealed scene. No compile or build, no epoch or lane binary, no carriage of a new program, no
  git, no network, no new dependency, no `unsafe`. No row reclassified, no predicate weakened,
  no tuple changed in place: the revision is a recommendation with its old→new record, applied
  by a later authorized brief.

## Deliverable

1. **The 151-row cover-witness table (TSV/JSON plus a summary table).** For each own row: class,
   family, cover program id and tuple (decoded: variant, context, route, descriptor), the
   program's setup prelude and source expression as the harness builds them (`guest.rs` family
   table), its pause plan from `programs.tsv`, the producer wrapper site, the executed caller
   site on the way to the producer (`file:line`), the hit site(s) with every guard conjunct, the
   value condition at each guard expressed as a code fact, the source path from the program
   through the lane-harness registrations (`guest.rs`/`native.rs` callbacks and the builtin
   registrations) to the producer and the hit, the historical outcome for the same tuple
   (`*-full-proofs.json` result, raw-log witnesses), and one verdict: `static-witness` (every
   non-frame guard conjunct is provably true on the program's own value path, by the armed
   tree's code), `static-non-witness` (the exact guard conjunct and the exact value that fails
   it, with the code fact that no other execution of this frozen program can supply another
   value), or `unknown` — **an unknown verdict is a stop, not a row**. The `frame(row)`
   authentication is the mechanism's and is measured in Stage B/C; the static verdict covers
   the value condition only. **Zero unknown rows.**
2. **The candidate table for every non-witness row.** From that row's own bound programs in the
   sealed `programs.tsv` only, every program whose pause plan and source expression satisfy the
   same witness condition, derived by the same method and citing the code fact; the tuple each
   candidate carries; and the recommended replacement: the witness candidate that changes the
   row's own identity least (same family, same consumer route and descriptor where one exists,
   then the smallest tuple change), with the reason. State the old tuple and the new tuple side
   by side, and what the epoch carries (cover program, `selected-own-tuples` record, frozen
   schedule). A row with no candidate stops with that row and the exact searched set.
3. **The six dependencies.** P0325, P0326, C1/G15A/G15B/Some: for each selected program the
   executed caller, the producer and the hit reached on that program's own path, the same value
   condition, and the witness/non-witness verdict. These are the 749 audit's remaining
   dependency rows; do not re-derive the pair/structural bars, which keep their own falsifiers.
4. **Method and calibration.** State the search space (which trees, files and registries), the
   counting rules, and the derivation of each static verdict. Calibrate on known answers: P0339
   must come out `static-non-witness` with `normalized_option=None` against the `Some` and
   `rules.len() == 1` guard; the ten Stage C class rows (P0256, P0259, P0266, P0267, P0312,
   P0314, P0315, P0323, P0410, P0515) and C10 must come out `static-witness` — they measured
   live arms and ordered same-activation hits at c96 — and P0001/P0002 must come out
   `static-witness` through the zero-capacity and positive-capacity branches respectively. A
   method that gets any calibration wrong is not usable: say so and stop.
5. **The isolation falsifiers.** Per row: the incoming-edge withholding (suppress the executed
   caller, keep the whole driven body → the row's producer/hit unreachable) and the value
   negative (a program supplying the failing value → `static-non-witness`, and on the epoch's
   armed measurement a NoHit). State which falsifiers are static (code) and which belong to
   Stage B/C; a static verdict is never a catch — the campaign's armed measurement confirms it,
   and a static witness that measures NoHit is a finding against the audit.
6. **The revision list and forecast.** The exact old→new assignments for the non-witness rows;
   the files and records the later authorized edit touches (`mutant-coverage.tsv`, the selection
   records, the frozen schedule) and the ones it must not (programs, mutants, policy, schema,
   overlay images); the effect on the epoch (the selection set only; the schema
   `22751919…` and policy `3953aa9f…` unchanged; the 20 class credits stand); the measured cost
   basis for the authorized edit, the 749 attempt-3 render and any re-freeze, and what Stage B,
   Stage C and the 744 restart would carry on the revised covers.
7. **Zero-unknown statement.** Explicitly: 151 own rows + six dependencies, every verdict with
   file:line and a value fact, no row settled by argument.

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
  favorable branch, never count a byte or text identity as a witness. The 748 static `reachable`
  verdict is not a witness certificate for any row.
- Paste verbatim evidence for every verdict (file:line, decoded tuple, the exact value and
  guard, the historical result line). Save every command's full output under `.temp/brief-750/`
  before parsing.
- If a row's value path cannot be decided from the code and the frozen program, STOP with that
  row and the exact missing fact — never a plausible reading. A candidate chosen by preference
  over a witness condition is not a candidate.

## Verification

- 151 own rows with verdicts, zero unknown; every verdict cites the decoded tuple, the program
  text, the source path and the value fact; the calibrations match (P0339 non-witness, the ten
  class rows + C10 witnesses, P0001/P0002 witnesses); the six dependencies carry the same
  derivation; every non-witness row has a proven candidate or the brief stopped.
- The revision list names old and new tuples, cover programs and the exact records; it changes
  no program, mutant, policy, schema or image; the class credits and the 13 PortBlocked
  bindings are untouched.
- The isolation falsifiers are stated per row, and the static/measured boundary is explicit.
- Budget: 2,400 s focused from the stage reserve, 0 workspace, 0 epoch; every bounded command
  debits by monotonic elapsed time; a projected overrun stops for a director ruling, never a
  scope cut.

## Report format

`rum-report/1` at `.temp/brief-750/report.json`; the witness table, the candidate table, the
six dependencies, the method and calibration, the isolation falsifiers, the revision list and
forecast, the zero-unknown statement, the Open ledger state; `changes` empty. Save every
command's full output in the scene; cite lines exactly. Before finishing run
`python3 scripts/report.py check 750` (without `--against-git`) and fix every PROBLEM; the final
message is only `python3 scripts/report.py summary 750`. STOP at the inventory and the revision
list: no edit, no render, no dispatch of 749 attempt 3, 751, Stage B/C or 744/745/746.
