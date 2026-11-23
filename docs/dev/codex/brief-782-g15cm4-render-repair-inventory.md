# Brief 782 — G15-CM4 Stage D repair: the deterministic render — the render-repair inventory and implementation plan (design-only)

**Design-only brief. `Scope: NONE`.** It continues ruling 790 (gate c140) and brief 781
(`9e00f1af`). Brief 781 stopped correctly at its stage A: the entry is green (five trees,
four tables, class table, 147/147 overlay cells, tool copies, arm bijection, four probe hits,
`ws=0`, 21/21 hot occurrences), but the c94 structural comparison reports
`diff=87 extension=136` on the install tree against a declared set derived from the 743
render. F17: the declaration is stale — the install tree is the 768/769 re-emitted class
form and its diffs/extensions are the declared policy shapes (the Q25/class-lease insertions,
the re-emitted wrapper spelling) and the declared adopted/added files. This brief designs the
deterministic render repair that lands the four ordered fixes, regenerates every derived
inventory the 776 reference set names, rebuilds the trees, and re-runs the source gate and the
workspace on the final tree. It writes nothing but its own evidence and plan.

## Goal

When this brief is done, in `.temp/brief-782/` only:

- **A complete zero-unknown inventory of the four ordered fixes**, each with its actual
  object, source edge, recipe, ordering, consumer map and falsifier:

  1. **F5 — the `cm743_route_a_falsifiers` module leaves `lib.rs`.** The module exists only in
     the composed `lib.rs` postimages (armed `e8fe11f3…`, disabled `86f4add8…`; brief 743's Q94
     record and TODO.md's c135 entry), never in live. Inventory: the module's exact byte span
     in both postimages; the composer artifact that emits it (which file under
     `.temp/brief-749/attempt-4/generator/` produces it, or why it is a frozen postimage edit —
     say which artifact is authoritative and why, per "the anchor follows the byte source");
     the chosen target file under a `tests/` path (the hygiene scan skips `/tests/`), its
     declaration site, the module-path change, and every use statement that must move; the
     resulting test name and **every standing reference to the old name** (inventories,
     certificates, tables, gates, the 71 ignored set, briefs' pins); and the **full schema
     footprint** with the regeneration command and falsifier for each: overlay rows (a new row
     for the declaring file?), additions (the new file), chain rows, class-table seals,
     `mutants.tsv`'s `source_schema` column, the verifier's `copied == baseline ⊎ additions`
     rule and its falsifiers, the six receipts' row tables, the arm bijection, and the six-case
     source gate's evidence.
  2. **F13 — the retention contract regeneration.** The ignored maintenance generator
     (`regenerate_local_contract_fingerprints`, review copy under `target/brief-544/`), its
     exact command and inputs on the final tree, the adoption targets on both sides (the
     `local-contracts.tsv` adopted path and the scene copy), the ordering after F5, the Q25
     formatting and F14 (the flat rot guard of `contracts::fingerprint` hashes `DEPENDENCIES`
     plus every `crates/**/*.{rs,py,toml}` — spell the real extension set from source), and the
     three falsifiers (`local_concrete_contract_controls`,
     `local_option_read_claims_match_stable_rustc`, `local_contract_falsifiers`) with the
     transition each is watched across. Also decide `promoted.tsv`'s derivation the same way,
     with its own generator and consumers.
  3. **F14 — the `source_auth` test's hermeticity.** The exact site
     (`observer_policy_tests.rs:482:48`), the one-line fix (create the parent), the falsifier
     (the test passes in a tree without `.temp/brief-743/`), the control (the tree-with-link
     case still passes), and the blast radius (one test file; no inventory cell).
  4. **Q25 — the two carrier files' formatting.** The exact rustfmt invocation for
     `crates/rum-runtime/src/cm740_probe/q25_ownership.rs` and
     `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs`, the scene mirroring, and
     the ordering before the fingerprint regeneration.

- **The complete map of every workspace red the render repair must close.** Take the 776
  reference set (`.temp/brief-776/logs/stage-d-workspace.log`) and the 780 stage-D run
  (`.temp/brief-780/logs/stage-d-workspace.log`); list every unique red name (the nine
  loopback refusals excluded and named), and for each: the owning inventory file(s), the
  generator that produces it (the ignored maintenance test or script, with its command and
  review-copy path), the adoption target(s), the falsifier that proves the regeneration, and
  its position in the dependency order. **Zero unknown**: a red whose generator cannot be found
  from source is a stop, not a guess. State explicitly which reds are inventory regenerations
  and which (if any) need a repair, with evidence.

- **The deterministic order of the whole repair** as an explicit DAG: every live-side edit
  (only F14's test file is live-only; F5's module and the Q25 files live in the composed
  trees), every scene-side edit, every regeneration, the 768 re-anchoring, the 769 allowance
  certificate, the class table, the adopted-path sync, the trees' rebuild, the six-case source
  gate, the workspace, and the fmt. State the invalidation rule (any source edit after a
  regeneration invalidates it) and prove no cycle.

- **The final-tree definition and the verification run book**: which tree the full workspace
  runs on (the final armed image, under `RUM_CM740_BASELINE` = the rebuilt baseline, as in
  780's stage D), the two named skips, the flags, the external `CARGO_TARGET_DIR`,
  `RUST_TEST_THREADS=8` with `--no-fail-fast` as a cargo option (before `--`), the F16 duty
  (remove test-created scratch, re-assert every tree hash, disclose), the red bar (exactly the
  nine loopback refusals), the ignored bar (the 71 reference names, set-identical), and the
  final pins.

- **A drafted implementation brief for 783** in `TEMPLATE.md` shape: stages with component
  budgets, the exact bars, the stops, and a realistic forecast.

The plan answers its own open questions with a recommendation each; none is left for the
implementer to guess.

## Context pointers

- Ruling `.temp/director/ruling-790.md`; brief 781 and its report; rulings 789/788/786.
- The composed postimages: `.temp/brief-749/attempt-4/generator/brief-743/stage-a/`
  (`route_a.py`, `ablate.py`, `canonical.py`, `descendant.rs.in`) and the 743 records in
  `docs/dev/codex/brief-743-g15cm4-observation-policy.md` (the module at line 852, its one
  `#[test]` at 930, the hygiene finding at 684–696 and 813–825).
- The tables and generators: `.temp/brief-779/install` (`overlays.tsv`,
  `mutants.tsv`, `emission_chain.tsv`, `additions.tsv`, `class-provenance.tsv`),
  briefs 768/769 (the certified re-anchoring generator and the allowance certificate),
  brief 777's `stage_a.py` (the twelve `ADOPTED` paths at lines 86–99).
- The workspace evidence: `.temp/brief-776/logs/stage-d-workspace.log`,
  `.temp/brief-780/logs/stage-d-workspace.log` (4,494 passed / 87 failed / 71 ignored; the
  four open names); the retention contract source
  (`crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs`) and the
  `source_auth` test (`crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/
  observer_policy_tests.rs`).
- The six-case gate and the receipts: brief 779's stage-C procedure and
  `.temp/brief-779/receipts/`.

## Scope

- You MAY write only under `.temp/brief-782/`: scripts, logs, the inventory, the drafted brief,
  the report. The live tree, the 779/780/781 trees, `.temp/brief-774`/`772`/`776`/`777`/`780`
  and every sealed artifact are read-only.
- **No source edits, no builds, no test runs, no cargo, no git, no network, no new
  dependency.** Read and analyse; derive everything from source text and existing logs.

## Stage A — entry and the final-tree definition (900 s)

Re-authenticate the 781 entry from the director's ruling: the five trees' pins, the four
tables, the class table, the 147-row control, the tool copies. Read the live status honestly
(no git; state the 23 inherited entries from the director's record). Then define the
final-tree predicate: exactly which paths change from the 779 install/baseline/images, by
which edit, and what the rebuilt trees' contents will be. State the workspace target tree and
every environment fact the run book needs.

## Stage B — F5 (900 s)

The module's byte span, the composer artifact that emits it (or the frozen-postimage fact and
its authority), the target tests file and declaration, the module-path and test-name change,
every use statement, every standing reference to the old name, and the complete schema
footprint with a regeneration command and falsifier per affected table. Answer: does the new
file enter `additions.tsv` (8 → 9) and the chain (152 → 153), does the declaring file become
an overlay row (147 → 148), what does the class table do, and what does the verifier's
`copied == baseline ⊎ additions` rule demand. Quote the source lines.

## Stage C — F13, F14, the Q25 formatting (600 s)

The three fixes' recipes as above, the exact ordering among them and against F5, and the
falsifiers. For F13, prove the generator's inputs are the final tree and its output's adoption
keeps the adopted-path invariant on both sides.

## Stage D — the red map (900 s)

Every unique non-loopback red name from the 776 reference set and the 780 run, mapped to its
inventory, generator, review copy, adoption target, falsifier and order. Zero unknown. Group
by inventory; state the total distinct inventory files touched. Any name with no source-derived
generator stops the brief.

## Stage E — the DAG, the run book, the draft (300 s)

Assemble the deterministic order and the final-tree run book; draft brief 783 with stages and
budgets; write the report. Give a per-stage forecast for 783 (focused and workspace) with the
basis for each number.

## Verification and acceptance

The brief passes only with: the four fixes' complete inventories (object, edge, recipe,
ordering, consumers, falsifier); the full schema footprint for F5 with commands; every red name
mapped with zero unknown; the DAG with the invalidation rule and no cycle; the run book with
the exact bars; the drafted implementation brief and forecast; every verdict line quoted
verbatim and every open question answered with a recommendation.

## Checkpointing and stops

Checkpoint after every stage (`.temp/brief-782/PROGRESS.md`, logs under
`.temp/brief-782/logs/`). Stop, with the exact evidence, on: an entry pin mismatch; an
unfindable generator for a named red; a reference to the F5 test name you cannot enumerate; a
cycle in the DAG; any proposed hand edit of a derived file; any plan that edits the live tree
beyond F14; any need to edit a sealed artifact; a projected overrun; or any need for network,
a new dependency or `unsafe`. A stop is a correct terminal state.

## Hard rules

Design-only (`Scope: NONE`): no source edits, no builds, no test runs, no cargo, no git of any
kind, no network, no new dependency, no `unsafe`; `.temp` only; never `/tmp`; never read
`~/.config/ccds/api-key`; never edit a copied tool or a sealed artifact; Q25's thirteen rows
stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

Focused **3,600 s** (A 900 / B 900 / C 600 / D 900 / E 300) + workspace **0** + epoch **0**,
carved after brief 781 (5,668.88878847017 → **2,068.88878847017 s**; 781 charged 1,214.6 s of
its 4,400 s carve and returned 3,185.4). Unspent returns; a projected overrun stops for a
director ruling, never a scope cut. Brief 783's carve will be topped up from the campaign
allocation at its gate (user ruling 2026-11-19).

## Report format

`rum-report/1` at `.temp/brief-782/report.json`, appended as stage `A / c141`; `summary`,
`changes`, `deviations`, `questions`, every verdict line quoted, and the final message
`scripts/report.py summary`. The director gates from raw evidence.
