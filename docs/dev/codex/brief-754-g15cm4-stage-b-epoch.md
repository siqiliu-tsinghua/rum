# Brief 754 — G15-CM4 Stage B: the clean freeze and the twelve-block epoch from the accepted Stage A render

## Goal

When this brief is done, in `.temp/brief-754/` only: the accepted Stage A render (the
**144-file / 288-postimage** table, schema `5c3b02a3a149a2ff43b614e9fb57ef9b9624f0febaa7de0e3042ca291c22cf2c`)
is frozen into a new clean epoch — the live snapshot plus the 144 overlay pins, the nine new
paths and the 58 render sites, file set equal to the baseline's 5,810 — and the **twelve
labels** run serially (default and no-default × real/disabled/armed/spot; lane ×
real/disabled/armed/spot), one each, with the forced spot identities, the accepted warning
bar, the carrier drives, the peels, the P0543 live passes in both modes and the composed-vs-live
workspace A/B. The new epoch identity (trees, binaries, blocks, membership, selections,
program matrix, input pins) is issued and every count recomputed from the bytes. No class
admission, no 744/745/746, no production change, no live-tree edit, no new catch.

**Attempt 1** stopped before execution. Its workspace preflight priced the six required
69-target invocations at 6,294.134069 s (the four c94/c95 composed receipts plus the accepted
Stage A source pair, 2,936.272497400998 s, substituted for the two historical live runs) and
the separately metered comparison, scoring, switched/native and certificate receipts at
821.636075 s — 7,115.770144 s against the 5,500 s allowance; the target-only crosscheck
(5,743.73 s of libtest time over 414 targets, excluding outer compilation) and the warm-mode
sensitivity (5,758.802974 s) exceed the bar as well. Ruling `Q754-workspace`
(`.temp/director/ruling-760.md`) finds the STOP correct and raises the Stage B workspace
allowance to 8,500 s; no scope is cut. This is attempt 2. Attempt 1 re-verified the entry
state: the five accepted input pins, the two sealed records and the 4,690-file crate scope
with zero differences and zero files newer than entry. Re-hash them again at entry and start
from there.

## Context pointers

- Read `AGENTS.md`; brief 623's "Campaign invariants" section; rulings
  `.temp/director/ruling-759.md` (the Stage A acceptance), `ruling-757.md` and `ruling-758.md`
  (the c107/c108 canonical ordinary arm), `ruling-754.md`…`ruling-756.md`, and
  `ruling-760.md` (the c110 budget ruling and the attempt-2 scene rules).
- The accepted Stage A scene (read-only): `.temp/brief-749/attempt-7/`, especially
  `renders/one/overlays.tsv` (11,674,289 B, `5c3b02a3…`), `renders/one/mutants.tsv`
  (43,621 B, `e8b3b6e4…`, 151 rows with the new `source_schema`), `renders/one/observer-policy.tsv`
  (`3953aa9f…`), `renders/one/recipes.json` (the two-step inverses), `renders/one/partition.json`
  (the `pre`/`parent`/`post` records), `copies/source` (`97eaeb45…`), `copies/disabled`
  (`b41d1873…`), `copies/armed` (`bd8b7548…`), and `evidence/source-stage-identity.json`
  (the 116 rendered site identities, the nine new paths, the source-tree pins),
  `evidence/source-gate.json`, `evidence/open-ledger.json`, `evidence/source-stage-identity.json`.
  The real baseline is the sealed `default-real-5028530e…` tree (5,810 files).
- The accepted Stage B mechanics (read-only): brief 743's continuation 94 and gates c94/c95
  above (`docs/dev/codex/brief-743-g15cm4-observation-policy.md`); the scene
  `.temp/brief-743/stage-b/` (the freeze tools, `stage-b-commands.json`,
  `stage-b-blocks.jsonl`, the drive/peel scripts and the c94/c95 evidence); the 741
  `PLAN.md` §3. Re-derive the mechanics for the new render; never reuse an old build or hash.
- The 744 attempt-2 record and brief 748's reachability inventory; brief 752's nine cuts and
  six cover revisions; brief 753's six live revisions; the 62-row scheduled-caller audit and
  the ten supplemental edge-fix rows in `.temp/brief-749/attempt-7/evidence/`.
- Everything sealed (c76–c99) and the c90/c93 identity/copies stay read-only.

## Scope

- MAY create/modify: everything under `.temp/brief-754/`, with this attempt confined to
  `.temp/brief-754/attempt-2/` (the new freeze scene, the epoch, the evidence); the cumulative
  report and ledger stay at `.temp/brief-754/report.json` and `.temp/brief-754/ledger.json`.
  Attempt 1's root artifacts — its scripts, `logs/`, `evidence/`, `receipts/`,
  `attempt-1-report.json` and `attempt-1-ledger.json` — are read-only.
- MAY read: the live source tree, the accepted scenes and sealed scenes, the earlier
  `.temp/brief-*` records.
- MUST NOT: edit the live source tree (0 tracked changes); write under any sealed or accepted
  scene; run any epoch binary from the old epoch; create a thirteenth label or retry a failed
  label; run the campaign or any 744/745/746 stage; admit a class (Stage C); edit the live
  `overlays.tsv`, `mutants.tsv`, policy, schema, manifest or selections; commit or use git;
  touch the network; add a dependency or `unsafe`. Builds run only in the scene's
  `CARGO_HOME`/target.

## Deliverable

1. **The standing replay before the freeze.** On the frozen copies, replay the standing
   inventories (the 68 fingerprint rows, C01/C02 in both modes, ownership, rejection census,
   LP01 and typed slots) against the accepted source baseline. Re-hash the accepted entry pins
   first, then rebase the inventory by item identity for the render's deltas — the new
   `cm740_probe::control::Bound.live` opaque callback and the 33 additional UnknownConstruct
   occurrences per role, diagnostic-only, with owner statuses unchanged. The six supplemental
   composed-copy reds are the c94/c108 ordering-artifact class: report them, never hand-edit
   an inventory, never weaken a predicate.
2. **The clean freeze.** Compose the freeze in a new scene directory from the live snapshot
   plus the 144 overlay pins (schema `5c3b02a3…`, policy `3953aa9f…`), the nine new paths and
   the 58 render sites; the file set equals the baseline's 5,810. Verify every preimage,
   parent and postimage against the accepted table and every tree hash against the Stage A
   pins; write new freeze receipts with new names. The sealed c90/c93 `identity.json`/
   `copies.json` and the accepted Stage A scene are not modified.
3. **The twelve labels.** The twelve configurations run serially — default and no-default ×
   real/disabled/armed/spot, plus lane × real/disabled/armed/spot — one label each, no
   thirteenth, no retry of a failed label, with the forced spot identity. Each test binary is
   re-hashed; three distinct binaries per configuration; the missing/extra/unknown counts are
   each zero; the accepted warning classes are reproduced with zero new families (re-derive
   the classes and the proved relocations from the new render); the same-epoch hash identity,
   the inherited release profile, codegen-unit and cache rules hold.
4. **The carrier drives.** On the drive trees, the five carrier rows fire with their
   identities (join P0267/P0312/P0323; queue P1470/P1471); the disabled cohorts stay inert
   (60/60/60/45/45); the 62 affected rows' scheduled callers are present in the frozen drive
   trees as the Stage A audit records, and any row whose scheduled caller is absent at the
   frozen boundary stops with the exact fact (the dynamic campaign belongs to the 744
   restart, not here).
5. **The peels and visibilities.** The three peels reproduce 19,184.0000 bytes/level and the
   3,360.0000-byte intercept, with depth 48 = 924,192 bytes; the private frame sink produces
   E0603.
6. **The P0543 live passes.** Both modes pass under the pinned read-only source presentation
   (the accepted c81 lane binary and its source), with the declared `SourceIdentity` rejection
   in armed/default as at c94; the nine native loopback groups stay host-only with their EPERM
   refusal, one thread, the same binary.
7. **The workspace A/B.** Compose-vs-live with the pre-enumerated target set and the seven
   declared skips; 69/69 targets per mode; zero unexplained differences; the nine EPERM
   refusals declared; any declared composition difference carried from the c94/c95 record is
   named, never silently accepted.
8. **The epoch identity.** The new tree hashes, binaries, blocks, membership, selections,
   program matrix, forced identities and input pins, recomputed from the produced bytes; the
   `rum-report/1` report with every verdict line quoted.

## Hard rules

- Never weaken a predicate, reclassify a row, exempt a dead row, move a hit to a favorable
  branch, count a text or byte identity as reachability or as a catch.
- No default-path cost; no new `unsafe` in rum-core/rum-runtime; no test change; no production
  change.
- One label each, no retry, no thirteenth; a failed label is evidence, not a rerun.
- If a cut cannot be expressed at its frozen boundary or a scheduled caller cannot be bound,
  STOP with the exact missing fact — never a row exception, never a text-position workaround.
- Save every command's full output under `.temp/brief-754/` before parsing; quote verbatim
  evidence for every verdict.
- The accepted Stage A scene, the sealed scenes and the c76–c99 record are read-only.

## Verification

- The freeze reproduces the accepted 144-row table and the 151-row mutants schema
  (`5c3b02a3…` / `e8b3b6e4…`), the policy `3953aa9f…`, and the three source trees
  (`97eaeb45…` / `b41d1873…` / `bd8b7548…`) from the produced bytes.
- The twelve blocks: 12 starts / 12 PASS finishes, one each, spot == real, three distinct
  binaries per configuration, all binaries re-hash, warnings with zero new families,
  missing/extra/unknown 0, the forced identities.
- The five carrier drives fire; the disabled cohorts inert; the 62-row scheduled-caller
  presence audit; the 26 withholding fixtures still fail as required.
- The three peels; the private frame E0603; the P0543 passes in both modes; the nine loopback
  EPERMs.
- The two workspace comparisons 69/69 with zero unexplained differences, seven skips.
- The epoch identity complete; the live tree unchanged (git status exactly the inherited 23
  entries, no `crates/` write); the sealed scenes untouched; no epoch binary executed outside
  the scene.
- Budget: **2,000 s focused + 8,500 s workspace + 16,374 s epoch** (attempt 1 charged
  5.038669232 s focused, 0 s workspace, 0 s epoch, leaving 1,994.961330768 s focused). The 749
  pool remainder 1,572.0955407500176 s focused + 2,640.727502599002 s workspace is consumed
  first; the fresh draw is 422.8657900179824 s focused + 5,859.272497400998 s workspace +
  16,374 s epoch = **22,656.13828741898 s** from the stage reserve, leaving it at
  21,549.09109388812 s; unspent returns; a projected overrun stops for a director ruling, never
  a scope cut.

## Report format

`rum-report/1` at `.temp/brief-754/report.json` as the cumulative record (attempt 1's is
preserved at `attempt-1-report.json`; the ledger is cumulative at `.temp/brief-754/ledger.json`
with attempt 1's at `attempt-1-ledger.json`), reciting attempt 1's 5.038669232 s focused
charge; `changes` empty (every write is under `.temp/brief-754/`); sections for the freeze, the
twelve labels, the drives, the peels, the P0543 passes, the workspace A/B, the identity and
the ledger; cite lines exactly. Before finishing run `python3 scripts/report.py check 754`
(without `--against-git`) and fix every PROBLEM; the final message is only
`python3 scripts/report.py summary 754`. STOP at Stage B: no class admission, no 744/745/746,
no CM5.
