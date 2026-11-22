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

**Attempt 2** completed everything except the workspace gate: the clean freeze (5,810 files;
source `97eaeb45…`, disabled `b41d1873…`, armed `bd8b7548…`; render replay 288/288), the twelve
labels (12 PASS, forced spot identities, three distinct binaries per configuration, warnings
130/132 with 124 unused-parens sites and zero new families), the four carrier drives (20 row
executions; join P0267/P0312/P0323, queue P1470/P1471; disabled cohorts inert 60/60/60/45/45),
the 62-row audit and 26 withholding fixtures, the three peels (19,184 / 3,360 / 924,192) and
E0603, both live P0543 modes (55/55), and the build epoch identity `809b6473…`. It stopped on
the armed/default workspace case: 69/69 targets with 48 of the 49 c95 declared composed
differences (the scratch-roots test passed with its fixture parent present), nine host EPERM
refusals and one unlisted compiler-oracle timeout (built-in 700 s bound; the same binary passes
at one thread under the same bound in 626.2 s with 291 agreements and 0 disagreements; the
live/default oracle target passed at 679.01 s inside the same bound). Ruling `Q754-armed-default`
(`.temp/director/ruling-761.md`) adjudicates the case without a replacement — the raw record
stays pinned, the scratch-roots declaration is superseded by the two-sided control, and the
timeout is an environment-bound artifact with the serial same-binary verdict — and orders this
attempt 3 to run the three pending cases with the corrected fixture setup, credit the six-case
workspace gate and issue the combined Stage B acceptance identity. Attempt 2's passing
deliverables are its evidence and are not re-run; do not retry the twelve labels.

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
  `.temp/brief-754/attempt-3/` (the workspace completion, the adjudication records, the
  identity); the cumulative report and ledger stay at `.temp/brief-754/report.json` and
  `.temp/brief-754/ledger.json`. Attempt 2's scene and root artifacts — its scripts, `logs/`,
  `evidence/`, `receipts/`, `attempt-2-report.json` and `attempt-2-ledger.json` — are
  read-only, as are attempt 1's root artifacts.
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
7. **The workspace A/B.** Compose-vs-live with the pre-enumerated target set, the seven
   declared skips and the corrected fixture-parent setup in both arms; 69/69 targets per case;
   zero unexplained differences; the nine EPERM refusals declared; any declared composition
   difference carried from the c94/c95 record is named, never silently accepted. Attempt 2
   credited live/default and live/no-default; this attempt runs the three pending cases —
   armed/no-default, disabled/default and disabled/no-default — with their four follow-up
   groups, and credits the armed/default case under ruling `Q754-armed-default`: the raw suite
   stays pinned (`8594e956…`), the scratch-roots declaration is superseded by the two-sided
   control, and the compiler-oracle timeout is adjudicated by the same-binary serial verdict.
   If the oracle test times out in a remaining case, apply the same control to that case's
   binary (same inner bound, threads=1, no cached verdict) and record the exit, counts,
   duration and agreement totals; a clean pass (zero disagreements) continues without a new
   ruling, while any disagreement, a second timeout at one thread, any other unlisted
   difference or any other declared-now-passing test stops.
8. **The epoch identity.** The new tree hashes, binaries, blocks, membership, selections,
   program matrix, forced identities and input pins, recomputed from the produced bytes; the
   combined Stage B acceptance identity is issued once all six workspace cases are accounted
   for, reciting ruling `Q754-armed-default`, the pinned raw armed/default record and the two
   adjudications; the `rum-report/1` report with every verdict line quoted.

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
- The six workspace cases accounted for — live/default and live/no-default credited, the
  armed/default case credited under `Q754-armed-default` with its raw record pinned, and the
  three pending cases credited at 69/69 — with zero unexplained differences, the seven skips
  and the nine EPERM refusals declared.
- The epoch identity complete; the live tree unchanged (git status exactly the inherited 23
  entries, no `crates/` write); the sealed scenes untouched; no epoch binary executed outside
  the scene.
- Budget: **2,500 s focused + 11,500 s workspace + 16,374 s epoch** (attempts 1–2 charged
  1,732.156761451 s focused + 5,562.051885473 s workspace + 2,180.454507404 s epoch, leaving
  767.843238549 s focused + 5,937.948114527 s workspace + 14,193.545492596 s epoch). The 749
  pools are exhausted (0 s focused and 0 s workspace); the fresh draw is 922.8657900179824 s
  focused + 8,859.272497400998 s workspace + 16,374 s epoch = **26,156.13828741898 s** from the
  stage reserve, leaving it at 18,049.09109388812 s; unspent returns; a projected overrun
  stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-754/report.json` as the cumulative record (attempt 1's and
attempt 2's are preserved at `attempt-1-report.json` and `attempt-2-report.json`; the ledger is
cumulative at `.temp/brief-754/ledger.json` with the attempts' at `attempt-1-ledger.json` and
`attempt-2-ledger.json`), reciting attempts 1–2's 1,732.156761451 s focused + 5,562.051885473 s
workspace + 2,180.454507404 s epoch; `changes` empty (every write is under `.temp/brief-754/`); sections for the freeze, the
twelve labels, the drives, the peels, the P0543 passes, the workspace A/B, the identity and
the ledger; cite lines exactly. Before finishing run `python3 scripts/report.py check 754`
(without `--against-git`) and fix every PROBLEM; the final message is only
`python3 scripts/report.py summary 754`. STOP at Stage B: no class admission, no 744/745/746,
no CM5.

## Gate c112 (2026-11-22) — Stage B accepted: the three pending cases, the six-case credit and the combined identity

Attempt 3 ran on `cdx` (Codex CLI 0.171.0, `gpt-6.2-astra`, effort max) as unit
`brief-754-cdx-20261122-164426-556516`, 16:44:26–18:14 CST, rc 0, the 20 GiB scope peaking at
16,521,252,864 B (15.388 GiB, oom 0/0), 0 repo changes; dispatch log
`.temp/logs/cdx_brief_754.log` (936,103 B). The cumulative `.temp/brief-754/report.json`
(`2377f246…`) is DONE: 35 checks — 30 PASS / 4 FLAGGED / 1 NA — 4 PASS measurements, 6
deviations (all attempt 2's), 0 questions, 0 changes; `scripts/report.py check 754` OK with
69,060 citations. The four FLAGGED rows are the historical attempt-1/2 bars settled by rulings
760/761; the NA is `attempt2-issued-identity`, superseded by this attempt's `issued-identity`
PASS. Attempt 3 delivered the three pending workspace cases — armed/no-default 4,294/58/66/7
(declared 48, host 9, one adjudicated oracle timeout), disabled/default 4,293/59/66/7 (49, 9,
one adjudicated), disabled/no-default 4,294/58/66/7 (49, 9, 0) — each 69/69 targets under the
frozen plan and the seven skips; the two fresh serial adjudications under the ruling-761
delegation (same binaries `32c030f2…` and `c25306eb…`, one thread, original inner bound, no
cached verdict, exit 0, `[1,0,0,1163]`, 616.251591978 s / 642.448999076 s, 291 agreements / 0
disagreements); the per-case follow-ups (lp03 3/0, lp04 1/0, lp05 1/0, native 0/9/0/143 with
nine EPERMs at one thread and the same binary, the C01/C02 package certificate PASS); and the
combined Stage B acceptance identity `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
(`7af7b461abfd724980735f06e8b173f2824381bbb6d71104b1678f2cd85bcd25`, `status=accepted under
Q754-armed-default`, `accepted_workspace_gate=true`) binding build identity `809b6473…`, freeze
identity `c724c9ab…`, the six-comparison manifest `c3bc2616…`, ruling 761 (`b2797319…`), the
pinned raw armed/default record `8594e956…`, the twelve binaries, the ten trees, the 9,060
programs / 9,511 constituents / 255 selections / 151 own rows, membership 28/3/7/113, zero
class admissions and zero new catches. The ledger: 29 receipts summing 122.505098429 s focused
+ 4,794.512888459 s workspace + 0 s epoch, cumulative 1,854.661859880 / 10,356.564773932 /
2,180.454507404 s against the 2,500 / 11,500 / 16,374 s caps, 15,982.318858784 s unspent
returning, the stage reserve 18,049.09109388812 → 34,031.40995267212 s, the 749 pools closed;
the six non-zero-exit receipts are the three suites' declared/host 101 and the three native
EPERM 101s.

My independent gate: I parsed the three raw suites (69 targets each; totals exactly the
reported 4,294/58/66/7, 4,293/59/66/7, 4,294/58/66/7) and proved the distinct failing names
are exactly declared + nine host EPERM + the oracle test (58 = 48+9+1, 59 = 49+9+1,
58 = 49+9), no extra and no missing; scratch-roots passes in all six raw logs; the oracle
suite binary hashes equal the serial binaries (`32c030f2…`, `c25306eb…`); both serial logs end
`1 passed; 0 failed; 0 ignored; 1163 filtered out` with 291 agreements / 0 disagreements and
no `ORACLE_REUSED`; the native suite binaries hash to the recorded `26f8bd1f…`, `2d20300a…`,
`4dc17396…`; every follow-up log hashes to its record; all ten frozen trees recompute to their
pins; all twelve label binaries re-hash to the identity; the identity's inputs re-hash (ruling
`b2797319…`, six-comparisons `c3bc2616…`, blocks `ee5bb36c…`, freeze `c724c9ab…`, build
`809b6473…`, `Q754-armed-default.json` `7ca22094…`, the six per-case records/scoring/raw/
receipt chains); my own full live snapshot gives 5,818 files at `2313045a…` (the identity's
`live_tree_at_completion`); my own receipt sum equals the three buckets, `ledger.json`
`e17a4ab0…`, and the report check verifies 69,060 citations. Scene: `git status --porcelain`
is exactly the inherited 23 entries; HEAD = `origin/main` = `bddcbc7e`; no crates file newer
than the dispatch; no `.git-attempts`; sealed attempt-7 and c94/c95 scenes have zero newer
files; c95 `six-comparisons.json` still `d2a9d5c4…`; attempts 1 and 2 preserved byte-exactly;
my `.temp` mtime scan outside the scene finds only the dispatch log, my `state.md` and my
`gate-762/` artifacts. Ruling 762 accepts Stage B: all eight deliverables stand, the six-case
workspace gate is credited with 414 targets and zero unexplained differences, the serial
adjudications are confirmed under the delegation, and the combined identity `7af7b461…` is the
Stage B acceptance identity; attempt 3's scene is read-only and no attempt 4 exists. Stage C —
the twenty-cell class admission on this epoch — is next, then the 744 restart, 745 and 746;
CM5 unauthorized, 745/746 held, sealed c76–c99 sealed.
