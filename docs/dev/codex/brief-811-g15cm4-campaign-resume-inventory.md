# Brief 811 — CM4 class campaign resume: the epoch, the Stage C/C10 readmission and the 744 restart — design-only

**Director-dispatched, one design-only stage, one gate.** This plans the class campaign's
resumption after the D.1/D.2 line: the campaign epoch rebuilt on the current tree, the Stage
C/C10 readmission, and the 744 shakedown restart. It writes no code and runs no build or
capture. `Scope: NONE` of production.

## Goal

When this brief is done: the campaign's qualification state is determined from the sealed
records (the 754 epoch acceptance, the class repair line 756-797, the 806 install, the 807f
fresh epoch, the 805/809 results) with the exact evidence; the required epoch rebuild on the
current tree is specified as a step-by-step plan (the twelve release labels and three trees,
the observation policy, the build rules, the functional equality, the qualification gates);
the Stage C/C10 readmission is specified (the ten class members, the 20 logical own cells and
their ledger treatment against the 510-cell schedule, the C10 packet, the importers, the
ordinary negatives); the 744 restart is specified (the 244 cells: 18 own rows × 2, the 100
pair selections × 2, the four structural controls × 2, the negative controls, the re-freeze
for 745); every input is authenticated by hash and every unknown is either resolved or named
with the exact missing measurement; and the three implementation briefs (the epoch build, the
readmission, the 744 restart) are drafted in this template's shape. No production change.

## Context pointers

- Read `AGENTS.md`; rulings 759/762/763 (the repaired epoch), 766-783 (the class repair line),
  784-797 (the render/install), 798-809 (the D.2/residual line), 851/855/856 (the fresh epoch
  and the closed residual line), 857 (the two cost decisions: publication funded, fastpath
  excluded).
- The accepted but now stale campaign epoch: `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  `7af7b461…` (binding build `809b6473…`, freeze `c724c9ab…`, the six-comparison manifest
  `c3bc2616…`, the twelve release labels `45916ecc…`/`011d61bd…`/`6919c52a…`/`45916ecc…`/
  `b3b8b1e1…`/`c3d2d5d2…`/`c736d3c4…`/`b3b8b1e1…` plus the lane `b92b3ed2…`/`caaa5b6a…`/
  `c286c776…`/`b92b3ed2…`, the three trees `97eaeb45…`/`b41d1873…`/`bd8b7548…` at 5,810
  files); `.temp/brief-754/attempt-2/epoch/` (the thirteen build receipts and compiler logs,
  `cm754-blocks.jsonl` `ee5bb36c…`, the observation policy `3953aa9f…`, the render schema
  `5c3b02a3…`, `cm754-membership.json` `2a17e399…`, `cm754-selections.json` `8c061b27…`,
  `cm754-program-matrix.json` `5c15fc12…`, `cm754-own-tuples.json` `0202baf6…`).
- The schedule and admission authority: `.temp/brief-755/full-schedule-ledger.json`
  `3e4532cf…` (the 510-cell schedule; the `stage=744` subsequence), `.temp/brief-755/class-admission.json`
  `6d556f4b…`, `.temp/brief-755/packet-index.json` `3e1644aa…`, the class importers
  `5baa3124…`-bound proofs, the C10 calibration `f46ccfd3…` (`verified`, `class_cells=0`,
  `shakedown_credit=0`), the c96 table `bef35931…` and index `8bbd71a7…` (historical, never a
  credit).
- The observation policy and admission protocol: brief 743 (Stage C: the ten class members,
  all-or-nothing, 20 cells; the C10 calibration; the importers' exact lines) and brief 744
  (the shakedown's 244 cells; the 18 own rows; the negative controls; the held 745/746).
- The post-754 changes: the D.2 package's nine live paths (brief 805; the joint price
  13,556,424), the ten rendered outputs and the R4 verifier (brief 806; `R4 LIVE VERIFY PASS
  mapping=190 … content_delta=3 doc_extras=36`; the four role roots baseline `cb9205a6…`,
  real `dda9146e…`, disabled `30cd48fc…`, armed `df0db63c…`), the class repair line's final
  state (756-797: the Q25 contract, the fixtures, the source A/B, the render/install), the
  fresh epoch's policy and receipts (807f: the v2.5 policy `08c03054…`, the nine qualified
  draws, the 120 labels), and the sealed 809 record (report `ba3a733b…`, seal 13,108).
- The Q25 contract's thirteen named stops (P0214, P0216, P1070, P2348, P2354, P2379, P2381,
  P2385, P2387, P2409, P2411, P2416, P2418); 744/745/746 held until their gates; CM5
  unauthorized.
- Read the sealed records, never the live checkout, for every program, selection and baseline;
  the live tree is authenticated only as the build input.

## Stage A — entry authentication (600 s)

Authenticate, by hash identity, the records above: the 754 epoch identity and its twelve
labels and three trees; the 743/755 observation policy, schedule, admission, packet index and
C10 packet; the 744 attempt-3 record and the consumed-cell ledger; the class repair line's
final accepted state (its seals and receipts); the 806 install (the ten outputs, the verifier
line, the role roots); the 805 landed sources; the 807f record (the policy, the qualified
draws); the 809 record (the report, seal, pins). Census the live tree and record the exact
file count and digest the epoch build would start from. Hash only evidence identity and
untracked `.temp` artifacts (never re-hash git-tracked source, per the standing ruling); use
mtime+size pre-screens for the census. Record every pin PASS/STOP and the exit census.

## Stage B — the qualification state and the delta (1,500 s)

Determine, from the records and with exact evidence, the campaign's qualification state:
whether the 754 epoch plus the class repair line plus the 806 install plus the 807f fresh epoch
constitute a qualified epoch for the campaign, or the campaign epoch must be rebuilt on the
current tree (with the evidence for the answer, the ruling numbers, the hashes and the deltas
since the 754 epoch). Inventory the delta: the nine D.2 live paths, the ten rendered outputs,
the class repair line's changes to the class-form render/observation, the live crates, and
every 754-epoch input (the observation policy, the render schema, the membership, the
selections, the program matrix, the own tuples) with its current validity.

Specify the epoch rebuild: the twelve release labels and the three trees, the one-codegen-unit
offline build rule, the functional equality (the 120 ordered labels `c193d87f…` and the
canonical program), the six-comparison manifest, the qualification gates (the v2.5
interference budget where a hardware-timed run is required; the functional/observational rules
where not), and the exact build counts and pins.

Specify the readmission: the ten class members, the 20 logical own cells, the ledger treatment
against the 510-cell schedule (resolve whether a readmission re-spends or re-credits the 20
cells, with the sealed schedule as authority), the C10 packet (re-taken or re-verified by
identity), the importers, the ordinary negatives, and the entry/exit gates.

Specify the 744 restart: the 244 cells (18 own rows × 2, the 100 pair selections × 2, the four
structural controls × 2), the negative controls (the eight observer, ten source-auth and four
Q7; the generator continuation/token negatives), the schedule's `stage=744` subsequence, the
re-freeze of the 246 for 745, and the held Q25 rows. Name every missing measurement.

## Stage C — the plan and the three implementation briefs (1,300 s)

Produce the step-by-step plan: the epoch build (files/functions per step, the invariants, the
gate tests, the call-site/ripple inventory, the risks), the readmission, and the 744 restart,
each with `path:line` evidence and the exact commands, inputs and bars. Draft the three
implementation briefs in the TEMPLATE shape (the epoch build, the readmission, the 744
restart), each with its goal, context pointers, stages, stop rules, hard rules, charge and
report format. Answer every open question with a recommendation; no `TBD`, no
`director-selected X`, no unstated choice.

## Stage D — the zero-unknown check (500 s)

Verify the plan against the evidence: every input pinned, every schedule cell accounted, every
ledger number reconciled (20 + 244 + 246 = 510), every pinned hash re-checked from the
records, every claim carrying `path:line` or a sealed record, every unknown named with the
exact missing measurement. Reconcile the cell arithmetic and the epoch's build counts. A plan
that depends on an unauthenticated or an unstated input is a STOP, not a pass.

## Stage E — report, check, seal (300 s)

Write `.temp/brief-811/report.json` as `rum-report/1` with the pins, the qualification
determination, the delta, the three plans and drafts, the open questions and the charge; check
it with `python3 -I scripts/report.py check .temp/brief-811/report.json`; detach the summary;
seal and rehash; audit the exit.

## Stop rules

Stop with the exact evidence on: an entry pin mismatch; a qualification determination that
needs a measurement not in the record; a schedule/ledger reconciliation that does not close; a
missing or contradictory record; a plan step that would edit the live tree, a sealed artifact
or a tool; a proposed hand edit of a derived file; a dependency on network, a new dependency or
`unsafe`; a projected overrun of a stage box; a stage reaching its deadline. A stop is a
correct terminal state.

## Hard rules

Design-only (`Scope: NONE`): no source edits, no builds, no captures, no acceptance suites, no
git of any kind, no network, no new dependency, no `unsafe`; `.temp` only, never `/tmp`; never
read `~/.config/ccds/api-key`; never edit a sealed artifact; the 743/744/754/755 and 798-809
records stay sealed and must survive to the epoch's final gate; Q25's thirteen named stops
stand; 744/745/746 held until their gates; CM5 unauthorized.

## Charge

4,200 s of the 6,041.152 s pool: **A 600 / B 1,500 / C 1,300 / D 500 / E 300**. 810 stays
cancelled; the implementation briefs carry their own explicit top-ups at dispatch (user ruling
2026-11-19). No production change; the design work charges the pool only.

## Report format

`rum-report/1` at `.temp/brief-811/report.json`, stage `A / c206`; `summary`, `changes`,
`deviations`, `questions`, every verdict line quoted, and the final message
`scripts/report.py summary`. The director gates from raw evidence.

## Checkpointing

Append `CLOCK stage=<A|B|C|D|E> t=<monotonic_ns>` to `.temp/brief-811/PROGRESS.md` at every
stage entry and exit. Each stage's box is measured from its own CLOCK entry; the charge is
continuous from dispatch; a stage reaching its deadline stops there with the exact missing cell
and its elapsed time, never kept open. Save each immutable cell before replacing a staging
tree; at each checkpoint record the exact next cell and any live dirt attributable to this
brief.

## Director gate (c206, 2026-11-27) — the Stage A STOP is accepted; the campaign epoch's raw evidence was destroyed by the c154 clean-temp run

The attempt (unit `brief-811-cdx-20261126-234906-4083320`, 23:49:06–23:55 CST, rc 0, 78,808 tokens, peak 253,816,832 B, oom 0/0) stopped on this brief's own rule "a missing or contradictory record", and the STOP is correct: the surviving witnesses authenticate (754 report `2377f246…`, the 755 authentication pair `9911a614…`, the recorded epoch digest `7af7b461…`), the 21 epoch pins are all ENOENT, the entry and exit censuses are identical (5,901 files, metadata `79777b46…`, 0 changes), and the scope touched nothing (0 source edits, builds, captures, replays, suites or git attempts; report check OK 36 citations; seal 12 artifacts, every hash re-verified by the director). Ruling 858 records the finding:

- **The deletion.** `.temp/director/clean-temp-c154.log` (2026-11-25 07:01) applied, as "old brief subdirs", the campaign line's raw evidence: `brief-743/{stage-a,stage-b,epoch,stage-c,design,design2,instruments,logs,…}` (105.01 GiB, including the 20 c96 admission captures), `brief-754/{attempt-2,attempt-3}` (34.90 GiB: the epoch identity `7af7b461…`, the freeze, the six-comparison manifest, the membership, the selections, the program matrix, the own tuples and the twelve release labels), the 744 attempt directories, `brief-755/{packets,evidence,receipts,indices,import-view,calibration}` (the C10 packet `f46ccfd3…` and the importer payloads) and the director receipts `gate-755`…`gate-769`. Irreversible: no copy under `/home`, no ZFS snapshot of `zpcachyos/ROOT/cos/home`, no dataset on the spare SSD.
- **Director defects #40/#41 (mine).** clean-temp had no preservation mechanism and I applied a plan that printed those DELETE lines; and from c154 to c206 my state asserted the records survived without a filesystem check, so this brief pinned deleted paths instead of verifying them at authorship.
- **The process repair (landed with this gate).** `docs/dev/preserved-evidence.txt` plus `scripts/clean-temp.py` enforcement: preserved roots are never pruned, preserved gate numbers are exempt, and a listed entry whose directory is missing fails the dry-run. Falsifiers watched: the dry-run fell from 71.3 GiB (including `brief-797/amendment6`, the rebuild's build source) to 717.6 MiB with 30 `PRESERVE` lines, and a nonexistent entry makes it exit 1.
- **The re-grounding.** Qualification is forced to a rebuild (the 754 epoch cannot be re-authenticated and is stale); the 20 c96 cells are re-measured in the readmission (20 + 244 + 246 = 510, all measured; the credit retired with the loss disclosed); C10 and the importers are re-taken from the surviving packet index; the 744 restart is unaffected; the policy lineage is re-derived from `generate_policy.py`/`lane-env.json` or re-frozen under v2.5 with `08c03054…` as the parent.
- **Charge.** 380.167 s of 4,200 (A 230.115574494/600, E 126.878243/300); unspent 3,819.833 s returns; pool 5,660.985 s; cumulative 22,739.015 of 28,400. Amendment 1 is carved 3,800 s from the return.

## Amendment 1 — re-dispatch on the surviving records (gate c206)

**Why.** The original Stage A authenticated records the c154 clean-temp apply had deleted. This amendment replaces the epoch pins with the surviving witnesses and makes the qualification determination the rebuild. Same goal, same deliverable, same hard rules.

**Stage A — entry authentication (300 s).** Authenticate by hash: the 754 report `2377f246…`; the 755 schedule `3e4532cf…`, class admission `6d556f4b…`, packet index `3e1644aa…`, authentication entry == closing `9911a614…`, ordinary negatives `e42ced67…`; the 744 report `314ca69d…`; the 743 report `6394a163…` and `report-c94.json` `de91efc5…`; `.temp/director/ruling-858.md`; the c154 log lines naming the four roots; `docs/dev/preserved-evidence.txt`. Quote the recorded epoch digest `7af7b461…` from the three witnesses as a constant, never as a file. Census the live tree (mtime+size pre-screen; hash only untracked `.temp` artifacts, never git-tracked source). Record the qualification inputs that survive: `.temp/brief-807f/policy.json` `08c03054…` and its diffs, the 807f draws, the 806 role roots, the 797 amendment6 images, the 795/796 instruments. A claimed recovery of a lost artifact is a STOP.

**Stage B — the determination, the delta, the three plans (1,500 s).**
1. **Qualification determination.** The 754 epoch is unauthenticatable and stale; the campaign epoch is rebuilt on the current tree under this rule: the twelve release labels, one codegen unit, offline, the 120-label functional equality `c193d87f…`, the six-comparison manifest, the v2.5 interference budget where a hardware-timed run is required. The recorded digest stays a historical identifier.
2. **The delta.** The nine D.2 live paths, the ten rendered outputs, the class repair line's final state, the live crates, and every 754-epoch input with its current validity — including the observation policy `3953aa9f…`: decide from the derivation evidence (`generate_policy.py`, `lane-env.json`, the switched-binary hashes, the 807f policy and its diffs) whether to re-derive it or re-freeze under v2.5 with `08c03054…` as the parent.
3. **The readmission.** The ten members (P0256, P0259, P0266, P0267, P0312, P0314, P0315, P0323, P0410, P0515), all-or-nothing; the 20 logical own cells re-measured against the rebuilt epoch; the ledger 20 + 244 + 246 = 510, all measured, the c96 credit retired with the loss disclosed; the C10 packet re-taken as calibration (`class_cells=0`, `shakedown_credit=0`) from the surviving packet index; the importers re-run per mode; the ordinary negatives.
4. **The 744 restart.** The 244 cells (18 own rows × 2, the 100 pair selections × 2, four structural controls × 2), the negative controls (eight observer, ten source-auth, four Q7, the generator continuation/token), the schedule's `stage=744` subsequence, the re-freeze of the 246 for 745, the held Q25 rows.

**Stage C — the plans and the three implementation briefs (1,200 s).** Step-by-step plans with `path:line` evidence and the exact commands, inputs and bars; drafts of the three implementation briefs (the epoch build, the readmission, the 744 restart) in the TEMPLATE shape; every open question answered with a recommendation, no `TBD`.

**Stage D — the zero-unknown check (500 s).** Every surviving input pinned, every lost input named with its re-take, every schedule cell accounted, the arithmetic closed, every claim carrying a path or a sealed record. A plan that depends on a lost or unstated input is a STOP.

**Stage E — report, check, seal (300 s).** `.temp/brief-811/report.json` as `rum-report/1`, stage `B / c207`; `python3 -I scripts/report.py check .temp/brief-811/report.json`; detach the summary; seal and rehash; audit the exit.

**Stop rules.** As in the original brief, plus: a claimed recovery; a plan that reuses a lost packet or binary as if it existed; an entry pin mismatch; a determination that needs a measurement not in the record; a schedule/ledger reconciliation that does not close. A stop is a correct terminal state.

**Hard rules.** Design-only (`Scope: NONE`): no source edits, builds, captures, acceptance suites, git, network, new dependency or `unsafe`; `.temp` only; never read `~/.config/ccds/api-key`; never edit a sealed artifact or the preservation list; the 743/744/754/755 and 793-809 records stay sealed and must survive to the epoch's final gate; Q25's thirteen STOPs stand; 744/745/746 held; CM5 unauthorized.

**Charge.** 3,800 s of the returned 3,819.833 s (A 300 / B 1,500 / C 1,200 / D 500 / E 300); pool 5,660.985 s unchanged; the implementation briefs carry their own top-ups.

**Report format and checkpointing.** As in the original brief: `rum-report/1`, every verdict line quoted, the final message `scripts/report.py summary`, and a `CLOCK stage=<A|B|C|D|E> t=<monotonic_ns>` line at every stage entry and exit with the stage stopping at its deadline.

## Director gate (c207, 2026-11-27) — the Stage B STOP is accepted: the sealed source-auth contract cannot authenticate the six-build index against one baseline

Amendment 1 was dispatched 2026-11-27 00:04:27 CST as unit `brief-811-cdx-20261127-000427-4091885` (background task `bnmn5e5j1`, log `.temp/logs/cdx_brief_811.log`; Codex CLI 0.171.0, `gpt-6.2-astra`, effort max, workspace-write, session `01a21899-e203-7200-a5ee-1793a28aecd8`) and returned rc 0 at 00:21, 315,920 tokens, peak 237,015,040 B, oom 0/0. Stage A PASS: 312 evidence pins + 3 witnesses + 36 c154 deletion lines + 5,902 live files; the 754 report `2377f246…`, the 755 authentication entry == closing `9911a614…`, the schedule `3e4532cf…`, class admission `6d556f4b…`, packet index `3e1644aa…`, ordinary negatives `e42ced67…`, the 744 report `314ca69d…`, the 743 report `6394a163…`, `report-c94.json` `de91efc5…`, the 807f policy `08c03054…`, the 809 report `ba3a733b…`; the recorded epoch digest `7af7b461…`; `R4 LIVE VERIFY PASS`. The ledger: 255 selections per mode; the 806 selections match the 755 selections in each mode; stage755 20, stage744 244, stage745 246, total 510, credit 0; the rebuild required (12 labels, 3 guest trees, 0 qualified epochs, 0 credit carried). Stage B STOP: the role manifest (baseline 5,842; real/disabled/armed 5,851 each; copied key sets equal), the nine additions (absent in baseline; present and hash-verified in the three roles, 27), the compile input (class-provenance.tsv 5,462,797 B `70a6da97…`, include at `coverage/bindings.rs:114`), F13 (the sole common-file delta local-contracts.tsv, baseline `ead1d6de…` vs real `217bec91…`), the importer STOP; Stage C drafts and Stage D were not entered.

I verified: all 358 physical pins (312 entry + 46 blocker) re-hash equal, 0 missing, 0 bad; the witness hashes above equal; entry == exit byte-identical at 5,902 files with 0 changes; zero source edits/builds/captures/replays/suites, zero `.git-attempts`, `git status` clean at `6626ea3e`; `report.py check` OK 381 citations; the seal's 20 payload files rehashed 20/20 equal; report `8a1b1f9f…`, seal `6bd2bc57…`, detached summary equal; charge 978.531383532 s continuous (A 194.977487642/300, B 515.510618065/1,500, E 249.086145315/300; dispatch→A 18.957 s).

**The finding, verified in the sealed sources.** `mutations/receipts.rs:76–118` uses one baseline (`RUM_CM740_BASELINE`) for every build line; `source_auth/artifacts.rs:35–63` requires the real arm's baseline and built trees to have equal file keys and identical bytes (except the lane manifest pins); `source_auth/mod.rs:176–232` requires every `schema::additions()` file absent from the baseline and the copied key set to equal baseline ∪ additions; `source_auth/schema.rs:24–85` compiles the tables in from `overlays.tsv` (165 rows) and `additions.tsv` (nine rows). The nine additions include `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/class-provenance.tsv` (`70a6da97…`, 5,462,797 B), which `coverage/bindings.rs:114` compiles into the real build and which is present with identical bytes in real/disabled/armed and absent in baseline. The 165 overlay postimages all match the copied trees, but one preimage (local-contracts.tsv) is anchored on the baseline role (`ead1d6de…`) while the real tree carries `217bec91…`. With one baseline the real-arm check forces the baseline to contain the nine and the copied-arm checks force them absent: unsatisfiable. The tables are anchored on the render's baseline role (the pre-D.2 tree), not on the importer's baseline (the real tree). The tables landed live with the nine rows at `e66e8531` (2026-11-25; class-provenance `d183c38e…` then), so the D.2 install did not create the contradiction; the 755 epoch's authentication (`authenticate.py`, `baseline_tree=trees['real']`) shows the importer's baseline was the real tree, so the 743-era additions cannot have been the nine (its tables are deleted; closing the inference is a design item). **Defect #42 (mine):** ruling 858's re-grounding said "the importers re-run per mode" without verifying the contract against the D.2 role trees; the STOP is the brief's rule working.

**The correction:** a role-aware source proof — regenerate the contract from the real role tree as the baseline: the real tree (5,851 files) byte-exact with zero real-arm overlays; the 165 role-specific files as overlay rows with the real preimages (164 already correct; local-contracts.tsv re-anchored `217bec91…`) and the disabled/armed postimages (all 165 already correct); the additions set empty (the nine are baseline files, byte-identical in every role) with a fail-closed empty-set expression; the copied key sets equal to the baseline's; the falsifier matrix rejecting a wrong baseline, a missing/altered file, an extra file, a wrong pre/postimage, a wrong arm, a binary mismatch and a configuration mismatch. Amendment 2 designs it and finishes the three implementation drafts.

Charge: 978.531 s of 3,800; unspent 2,821.469 returns; pool **4,682.454 s**; cumulative **23,717.546 of 28,400**. Amendment 2 earmarks 3,000 s from the return plus a 178.531 s top-up.

## Amendment 2 — the role-aware source proof and the three implementation drafts (gate c207)

**Why.** Amendment 1's Stage B stopped correctly: one baseline cannot both equal the real role tree (the real-arm check) and omit the nine declared additions (the copied-arm checks), because the nine — class-provenance.tsv among them — are in the real tree and compiled into the real build (`coverage/bindings.rs:114`). The campaign epoch needs a role-aware source proof. This amendment designs it and finishes the three implementation brief drafts.

**Stage A — entry and the stopped record (300 s).** Authenticate by hash the stopped Amendment 1 record at `.temp/director/811-stop-2/` (the seal's 20 payload files, report `8a1b1f9f…`, seal `6bd2bc57…`, charge 978.531383532 s) and re-hash the 358 physical pins (312 entry + 46 blocker, 0 missing, 0 bad; the single pin `.temp/brief-811/preservation-witness.txt` re-roots to `.temp/director/811-stop-2/preservation-witness.txt`, byte-equal to `docs/dev/preserved-evidence.txt`); the surviving witnesses (754 report `2377f246…`, the 755 authentication entry == closing `9911a614…`, schedule `3e4532cf…`, class admission `6d556f4b…`, packet index `3e1644aa…`, ordinary negatives `e42ced67…`, 744 report `314ca69d…`, 743 report `6394a163…`, report-c94 `de91efc5…`); the 36 c154 loss lines; `docs/dev/preserved-evidence.txt`; the four 806e freeze role trees and their file manifests (`real/disabled/armed/baseline-files.tsv`), the R4 map and the verifier receipts; the sealed source-auth sources and tables: `mutations/source_auth/{schema.rs,additions.tsv,overlays.tsv}`, `mutations/receipts.rs:76–118`, `mutations/source_auth/artifacts.rs:35–63`, `mutations/source_auth/mod.rs:176–232`, `coverage/bindings.rs:114`. Census the live tree (mtime+size pre-screen; hash only untracked `.temp` artifacts). A claimed recovery of a lost artifact is a STOP.

**Stage B — the role-aware source proof (1,200 s).**
1. **The contract lineage.** Inventory the importer's baseline semantics from the sealed sources above; the 755 epoch's authentication (`authentication-entry.json`, `authenticate.py`'s `fields=dict(arm, baseline_tree=trees['real'], …)` and its import view) and the surviving `class-admission.json` importer fields; what the 743-era contract must have expressed. Close the inference against the surviving receipts; name any fact that cannot be closed.
2. **The regenerated tables.** The exact form for the campaign epoch: baseline = the real role tree (5,851 files, the 806e freeze), byte-exact; the 165 role-specific files as overlay rows with the real preimages (164 already correct; local-contracts.tsv re-anchored at the real `217bec91…`) and the disabled/armed postimages (all 165 already correct); the additions set empty, with the exact empty-set expression that keeps the parse fail-closed; the copied key sets equal to the baseline's. Specify the regeneration path from the R4 map / the 806e freeze: the inputs, the tool, the byte-for-byte outputs, the install row, and the re-freeze.
3. **The falsifier matrix.** Each of these must be rejected by the named error path, as a concrete negative case: a wrong baseline tree; a missing or altered baseline file; an injected extra file; a wrong overlay preimage or postimage; a copied tree with an extra or missing file; a wrong-arm receipt; a binary hash mismatch; a configuration mismatch (default/no-default/lane).
4. **The two indices.** The six-build index (2 modes × 3 arms + the spot line) and the C10 index: the exact index lines and authentication fields (`arm`, `baseline_tree=trees['real']`, `tree`, `configuration`, `profile`, `codegen_units`, the per-arm overlay rows), the importer binary's build source, the per-mode bars (`required=151 caught=10 PortBlocked=141 settlements=0`; C10 `151/1/150`), and the partial-index exit 101. No packet or binary is reused; every receipt is fresh against the rebuilt epoch.

**Stage C — the implementation brief drafts (800 s).** Three drafts in TEMPLATE shape with exact pins, stages, stops and charges: (a) **the campaign epoch build** — the twelve release labels (default/lane/no-default × armed/disabled/real/spot), one codegen unit, offline, the 120-label functional equality `c193d87f…`, the six-comparison manifest, the v2.5 interference budget where a hardware-timed run is required, the regenerated source-auth tables and the importer build, and the six-build and C10 indices authenticated in both modes; (b) **the readmission** — the ten members' 20 own cells re-measured against the rebuilt epoch, the C10 calibration (`class_cells=0`, `shakedown_credit=0`), the importers per mode, the ordinary negatives, the ledger 20 + 244 + 246 = 510, the c96 credit retired with the loss disclosed; (c) **the 744 restart** — the 244 cells, the negative controls (eight observer, ten source-auth, four Q7, the generator continuation/token), the re-freeze of the 246 for 745, the held Q25 rows. Each draft names its own top-up request.

**Stage D — zero-unknown (400 s).** Every lost input named with its re-take; every contract file and table pinned; the falsifier matrix complete; the schedule arithmetic closed; every claim carrying a path or a sealed record.

**Stage E — report, check, seal (300 s).** `.temp/brief-811/report.json` as `rum-report/1`, stage `C / c208`; `python3 -I scripts/report.py check .temp/brief-811/report.json`; detach the summary; seal and rehash; audit the exit.

**Stop rules.** As in Amendment 1, plus: a proof design that cannot reject one of the falsifiers; a regeneration path that depends on a deleted artifact; a receipt or binary reuse; a draft that pins a lost input.

**Hard rules.** Design-only (`Scope: NONE`): no source edits, builds, captures, acceptance suites, git, network, new dependency or `unsafe`; `.temp` only; never read `~/.config/ccds/api-key`; never edit a sealed artifact or the preservation list; the 743/744/754/755 and 793-809 records stay sealed and must survive to the epoch's final gate; Q25's thirteen STOPs stand; 744/745/746 held; CM5 unauthorized.

**Charge.** 3,000 s (A 300 / B 1,200 / C 800 / D 400 / E 300): 2,821.469 s from Amendment 1's return plus an explicit 178.531 s top-up; the pool at dispatch is 4,682.454 s.

**Report format and checkpointing.** As before: `rum-report/1`, every verdict line quoted, the final message `scripts/report.py summary`, and a `CLOCK stage=<A|B|C|D|E> t=<monotonic_ns>` line at every stage entry and exit with the stage stopping at its deadline.

## Director gate (c208, 2026-11-27) — the Stage B STOP is accepted: the zero-additions form needs a reader change, and the F13 row must bind to native producer replays

Amendment 2 was dispatched 2026-11-27 00:29:58 CST as unit `brief-811-cdx-20261127-002958-4106839` (background task `b6ou0lb7h`, log `.temp/logs/cdx_brief_811.log`; Codex CLI 0.171.0, `gpt-6.2-astra`, effort max, workspace-write, session `01a218b1-40cc-7ee0-888b-0249fdf44aed`) and returned rc 0 at 00:48, 235,792 tokens, peak 284,340,224 B, oom 0/0. Stage A PASS (166.031/300): the stopped Amendment 1 record authenticated (20 payloads, charge 978.531383532 s, report `8a1b1f9f…`, seal `6bd2bc57…`), 358 pins re-hashed 0 missing 0 bad including the re-rooted preservation witness `e23ce813…`, the 36 c154 loss lines, the epoch digest `7af7b461…` in three fields, the nine witnesses, `R4 LIVE VERIFY PASS`, the four 806e role trees (baseline `cb9205a6…`/5,842; real `dda9146e…`/5,851; disabled `30cd48fc…`/5,851; armed `df0db63c…`/5,851; 23,395 files), entry census 5,902 / `c4f6341b…`. Stage B STOP (724.909/1,200) after delivering `LINEAGE PASS` (the 755 importer binds each non-spot build to the real tree with no real overlays and 144 copied triples; the ten class-admission members' 20 bindings; default proof `5baa3124`, no-default `dfafe857`; C10 packet `f46ccfd3` zero credit; the 743-era nine cannot have been the current nine, and the original parser/spelling is not preserved), `ROLE DELTA PASS` (5,851 equal guest keys; 165 diff paths; 164 real preimages exact; 330 postimage bytes exact; nine additions in all guests), the seed projections (overlays 14,068,709 B `534ccde8…`; mutants 43,621 B `57b2f3e7…`; additions 32 B `33478d6e…`), and the F13 projections (real `217bec91…`→`b44ee865…`, disabled `40e92684…`→`271d4276…`, armed `d3588fb9…`→`98bd5891…`; 68/68 fingerprints changed; 18 identity columns; `native_replays=0`).

The STOP lines: `SOURCE PROOF STOP zero additions require a reader change under crates; the standing fingerprints cover that reader.` / `POSITIVE CONTROL STOP unchanged parser rejects a header-only table at schema.rs:77 and the explicit zero marker at schema.rs:59.` / `OVERLAY ROW STOP R4=171 overlays.tsv=156 required_real=217bec91… parser_only_real=b44ee865… rejection=Preimage` / `POSTIMAGE STOP after changing only the real pin both copied F13 postimages still differ; rejection=Postimage` / `CONTROL RIPPLE tests.rs:163 unconditionally unwraps the first addition; an explicit zero needs an independent nonempty fixture and an empty-set control.` The `valid-control` check failed against its bar ("Authenticate a valid rebuilt arm with the fixed B.2 pins and a native-current standing inventory"), which is the STOP's substance: the fixed 806e F13 pins and a rebuilt arm are jointly unsatisfiable once the reader changes. Stage C drafts and Stage D were not entered.

I verified: all 460 external pins (358 c207 + 102) re-hash equal, 0 missing 0 bad; the seal's 24 payload files re-hash 24/24 equal; report `9069e0c8…`, seal `6bfd4380…`; entry == exit byte-identical at 5,902 files, 0 changes, `live_source_bytes_hashed=0`; zero source edits/builds/captures/native replays/suites/git/network/sealed edits, writes only `.temp/brief-811`; `git status` clean at `23ff0c8c`; `report.py check 811` OK 710 citations; charge 1,111.554 s continuous (A 166.031/300, B 724.909/1,200, C/D 0, E 197.309/300; dispatch→A 23.305). I re-read the sealed reader (`schema.rs:54`, `:59–60`, `:77–78`; `tests.rs:163`) and the F13 mechanism (`.temp/brief-790/overlay_digest.py:64–96` implements `local/contracts.rs:88–133` and `interface.rs:26–35`: the flat context is `fnv1a64` over `DEPENDENCIES` plus every `.rs`/`.py`/`.toml` under `crates/`, and each of the 68 rows takes `fnv([*fields[:18], context])`), and confirmed the projection's inputs (`schema_parent 34779fc5…` = the frozen file; the four role tables = the parents; live = armed `d3588fb9…`; the 32-byte literal = `33478d6e…`).

**The finding.** The zero-additions form is not expressible by data alone: the compiled reader rejects both an explicit zero marker (`schema.rs:59`) and a header-only table (`schema.rs:77`), and the tests unwrap the first addition (`tests.rs:163`). A reader change is required, and because the reader and tests are under `crates/`, it moves all 68 F13 fingerprints in each guest role (all 18 identity columns stay). Therefore the F13 row cannot be pinned to the 806e hashes in the rebuilt epoch: the four 806e trees stay immutable parent evidence, and the child overlay's real preimage and both copied postimages come from the native producer runs on the final tree in each role; the parser-only projection is illustrative and inadmissible as a binding identity. The ordering is forced: reader and fixtures final → native producer replay in both feature modes → adopt only producer output → encode the 165 overlays → propagate the digest through the 151 mutants and `source_schema.txt` → record every changed R4 row → re-freeze; stale fingerprints, positions and certificates are ordering artifacts, never adopted.

**Defect #43 (mine).** B.2 specified the empty-set form without checking the compiled reader, the tests, or the flat-context ripple, which this line's own c188 history records. **Q811-F13 answered:** the child overlay binds to the native producer outputs on the final tree (real = preimage; disabled/armed = postimages); the 806e hashes are parent pins only; the projection is not a binding identity; publication waits for the positive real/disabled/armed proofs and the named negatives (ruling 840's role-specific F13 precedent).

Charge 1,111.554 s of 3,000; unspent 1,888.446 s returns; pool **3,570.900 s**; cumulative **24,829.100 of 28,400**. Amendment 3 earmarks 2,750 s (A 250 / B 900 / C 1,000 / D 300 / E 300; no top-up); the pool at dispatch is unchanged.

## Amendment 3 — the reader change, the F13 binding and the three implementation drafts (gate c208)

**Why.** Amendment 2's Stage B proved the role-aware proof's zero-additions form needs a source change and that its F13 cells cannot be pinned to the parent record; this amendment designs both and finishes the three implementation brief drafts.

**Stage A — entry and the stopped record (250 s).** Authenticate by hash the stopped Amendment 2 record at `.temp/director/811-stop-3/` (the seal's 24 payload files against its own dict; report `9069e0c8…`; seal `6bfd4380…`; charge 1,111.554418934 s; summary; entry/exit censuses byte-equal); re-hash the 460 external pins (358 c207 + 102) at their recorded paths; the four 806e role manifests; the nine witnesses and the epoch digest `7af7b461…`; `docs/dev/preserved-evidence.txt`. Census the live tree (expect 5,902 files with exactly `TODO.md` and this brief modified against the stop-3 exit `c4f6341b…`; mtime+size pre-screen; hash only untracked `.temp` artifacts). A claimed recovery of a lost artifact is a STOP.

**Stage B — the reader change and the binding rule (900 s).**
1. **The zero-additions form.** Design the exact reader change: the accepted zero form (the candidate literal is exactly the 32-byte `file\tsha256\tbytes\n# additions=0\n`, `33478d6e…`, including both LFs) and its rejection matrix (truncation, an extra row, an altered or nonzero marker, a header-only table without the marker — each a named error path); the nonempty path unchanged (`parse_additions`); the tests' controls (an independent nonempty fixture, an empty-layout positive, the omitted/extra-file controls, and the `tests.rs:163` accessor); the file boundary (`schema.rs`, `tests.rs`, `additions.tsv`) and the must-not-change list. A form that cannot reject every named falsifier is a STOP.
2. **The F13 binding rule.** The four 806e trees and their manifests stay immutable parent evidence. The child overlay's real preimage and both copied postimages come from the native producer runs on the final tree, one per guest role in each feature mode, after the reader and fixtures are final. The parser-only projection is illustrative and inadmissible; the end-to-end ordering is: reader and fixtures final → native producer replay in both modes → adopt only producer output → encode the 165 overlays → propagate the digest through the 151 mutants and `source_schema.txt` → record every changed R4 row → re-freeze. Every stale fingerprint, position or certificate is classified as an ordering artifact; never adopt a projection.
3. **The render/install reconciliation.** Which R4 rows change (the F13 table 171, the mutants 183, the additions 184, the verifier module 185, the overlays 186, the parser 187, the tests 188, and every cell whose input includes the flat context); the ten outputs' recomposition; the six build records and the two indices; the per-mode importer positives and negatives.
4. **The lineage limit.** The 755 records establish the effective relation (equal file sets = zero added keys relative to the real baseline; the copied-arm rule requires no file absent in real; the 743-era nine cannot have been the current nine); the original 743 parser, its spelling for zero and a historical additions manifest are not preserved — record the limit, do not reconstruct it.

**Stage C — the three implementation brief drafts (1,000 s).** (a) **The campaign epoch build**: the reader change and fixtures, the producer replays, the twelve release labels, one codegen unit, offline, the 120-label functional equality `c193d87f…`, the six-comparison manifest, the v2.5 interference budget where a hardware-timed run is required, the regenerated tables, the six-build and C10 indices authenticated in both modes. (b) **The readmission**: the ten members' 20 own cells, the C10 calibration (`class_cells=0`, `shakedown_credit=0`), the importers per mode, the ordinary negatives, the ledger 20 + 244 + 246 = 510 with the c96 credit retired. (c) **The 744 restart**: the 244 cells, the negative controls (eight observer, ten source-auth, four Q7, the generator continuation/token), the re-freeze of the 246 for 745, the held Q25 rows. Each draft is in the TEMPLATE shape with exact pins, stages, stops and bars, and names its own top-up request.

**Stage D — zero-unknown (300 s).** Every contract file, table and row mapped to a disposition; the falsifier matrix complete (the parent pins, the child binding, the zero-form truncation/extra/nonzero, the nonempty fixture, the omitted/extra files, the key equality, the baseline equality, the overlay pre/postimages, the 151 mutants, `source_schema.txt`, the changed R4 rows, the per-mode importer positives/negatives); the schedule arithmetic closed; no unknown.

**Stage E — report, check, seal (300 s).** `.temp/brief-811/report.json` as `rum-report/1`, stage `D / c209`; `python3 -I scripts/report.py check .temp/brief-811/report.json`; detach the summary; seal and rehash; audit the exit.

**Stop rules.** As before, plus: a design that cannot reject a named falsifier; a binding that uses a projection or a parent hash as a child identity; a draft that pins a lost input; a claimed reconstruction of the 743 parser.

**Hard rules.** Design-only (`Scope: NONE`): no source edits, builds, captures, native inventory replays, acceptance suites, git, network, new dependency or `unsafe`; `.temp` only; never read `~/.config/ccds/api-key`; never edit a sealed artifact or the preservation list; the 743/744/754/755 and 793-809 records stay sealed and must survive to the epoch's final gate; Q25's thirteen STOPs stand; 744/745/746 held; CM5 unauthorized.

**Charge.** 2,750 s of the 3,570.900 s pool (A 250 / B 900 / C 1,000 / D 300 / E 300; no top-up); the implementation briefs carry their own explicit top-ups.

**Report format and checkpointing.** As before: `rum-report/1`, every verdict line quoted, the final message `scripts/report.py summary`, and a `CLOCK stage=<A|B|C|D|E> t=<monotonic_ns>` line at every stage entry and exit with the stage stopping at its deadline.
