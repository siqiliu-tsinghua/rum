# Brief 815 — G15-CM4 stage-745 inventory and implementation plan

**Scope.** A design-only (`Scope: NONE`) proof-carrying inventory of the 246 stage-745 cells of
the admitted schedule. For each of the 123 distinct rows (each rendered in both logical modes)
the inventory pins its class from the 757 authority, its bar (class fixture, native
signature or measured-block mechanism), its witness, its importer expectation and its
expected stage; the 13 Q25 rows stay held with their 26 identities and zero credit; the
implementation brief is drafted. **No execution of any 745 row, no source edit, no build, no
cargo, no repository suite, no network, no new dependency, no `unsafe`, no git; no 744 or 746
execution and no CM5.** Every write goes under `W = .temp/cm4-shakedown/stage-745-design/`.

**Pins (machine-generated over the actual files; any difference is a STOP).** All 77
`path` `sha256` `bytes` rows are listed in **Appendix P** below. The roots: the c243
shakedown record (`W9 = .temp/cm4-shakedown/amendment-9/`; seal
`99b6fc78c890e00640e1ab36b672bcfe2dfd77d3627b95daad1088c72c202cc6` with 42,260 payloads /
2,462,361,100 B), the campaign epoch `E = .temp/cm4-campaign-epoch/amendment-17/`
(subject `ce5d42a0…`, `epoch-identity.json` `8c1fdcd9…`, seal `1a28c596…` with 785
payloads / 739,439,260 B), the 811 design seal `da2fdf55…` (30 files, 632 pins), the 757
`class-97.tsv` `92d72a8a…`, the 756 witnesses, the 755 authority, the 741 signatures and the
744 report `314ca69d…`. A0 re-hashes all 77 pins, every payload row of the epoch seal (785)
and of the c243 seal (42,260), and records the live census; any mismatch stops.

**The stage-745 schedule.** The 246 cells are exactly the `original.stage = "745"` rows of
the c243 full-schedule ledger in original order (123 `default`, then the 123 `no-default`
seats; every row is `kind=own`, `bar=one-own-catch`, each row appearing once per mode). The
freeze arrays `identities-745.json` (`deea561b…`), `original-rows-745.json` (`963b20ff…`)
and `remaining-745.json` (`27c39044…`) are the lossless and identity projections; the three
ordered subsequences stay `be4ceb3c…` / `deea561b…` / `53ce6430…`. The forecast
`b0c1600d…` (123+123 rows, 51 families, central 8,286.74 s, planning upper 19,172.71 s)
bounds the later implementation; no spend moves in this brief.

**The inventory (the deliverable).** For every one of the 246 cells, and one entry per
distinct row (123), produce a table with these fields, each backed by a named sealed line:

1. **Identity** — row, slot, configuration, bar, tuple, program, both mode seats.
2. **Class** — from `.temp/brief-757/class-97.tsv`: `design-carried` with its exact
   `method`, `proof needed` (the 13 Q25 rows), or absent from the 97 (non-class). State the
   count of each and prove the partition sums to 123 (and 246 cells).
3. **Bar and fixture** — for a `design-carried` row, the exact fixture test name (the
   crate-root module form `class_lease_fixtures::class_lease_positive_p<row>`, never an
   `oracle::` prefix) and its required environment (the five
   `RUM_PROFILE_{LEXICAL_SUBSTITUTION,BUILTIN_TIMING,DEFINITION_RULE_TIMING,RULE_RHS_TIMING,EVALUATION_CACHE}=1`
   switches wherever the row's frame is profile-gated — name the frame and the branch), the
   expected `CLASS_INSTALL` group/frame contract, the committed typed/raw receipts and their
   consumer, the expected `FIXTURE VERDICT … PASS hits=0`, and the derived importer stage
   (`Unarmed` iff `producer_arms=0`, `NoHit` iff `producer_arms>0`). For a non-class row,
   either its native signature (carrier, phase, observer, predicate, multiplicity, prompt,
   arm/hit order) joined to the historical witnesses, or the named measured-block mechanism
   (the `ordinary(row)`-after-`around(row)` scope class or another proved scope failure) with
   its evidence and zero-credit rule, or a stated missing-witness finding.
4. **Witness** — the exact archived record line (`file:line` and the verbatim text) for every
   row: the 756 signature/event row, the 741 signature, the 755 admission row, or the class
   fixture source; for the Q25 rows the ownership record line from `held.json`.
5. **Isolation falsifier** — per row or per named family, the control that shows the bar can
   fail (a wrong-row/wrong-phase/unarmed/forged variant), with the expected rejection.
6. **Importer expectation** — for the mode's `cm_own_mutant_receipts_exact` single-row and
   index views: the row's program key among the 151, whether it can produce a native
   receipt, and the predicted index line `required=151 caught=<n> PortBlocked=<151−n>
   settlements=0`; name any row that needs the P0543 lane triplet or another presentation.
7. **Order and composition** — the 123-cell per-mode order, the 13 held seats, and the
   per-district group/frame composition (the typed/raw/Q20 families).
8. **Zero-unknown statement** — every one of the 246 cells (and each of the 123 rows) has a
   disposition: a named class, method, bar, witness or mechanism, importer expectation and
   falsifier, or — where the sealed authorities cannot prove the bar — a named missing
   measurement stated as an open obligation (the 799/808 precedent), never a guess and never
   a blank. State the count of rows in each class and the **0 unknown** line.

Publish the inventory as `W/inventory-745.tsv` (one row per cell, every field above) and a
`W/inventory-745.json` carrying each row's witness citations (path, line, verbatim text),
plus the per-family summaries.

**The 13 Q25 rows.** P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409,
P2411, P2416, P2418 stay held with both logical identities each (26), their exact
ownership STOPs, zero credit and no measurement in the implementation. The inventory
restates each row's `held_reason` and source verdict verbatim from `held.json` and the 757
row, and carries the obligation forward.

**The implementation brief (draft).** Draft the stage-745 measurement brief in the
`TEMPLATE.md` shape from this inventory: the 246 cells in original order (both modes), each
row's bar exactly as inventoried, the Q25 holds, the importers and the index expectation,
the negative controls as needed, the forecast comparison, the stage boxes sized from the
c243 timings, and the stop rules. The draft must quote the reasons the 744 campaign's six
class-form STOPs cannot recur on the 745 rows (the **risk table**): the fixture selector
path form (`class_lease_fixtures::…` from the crate root), the fixture environment, the
class-carried membership from 757 (do not re-split by hand), the derived importer stage
(`Unarmed`/`NoHit` by `producer_arms`), the held-pair-bar/assertion wording (no
`configuration` field, `semantic_records.rs:235:13`, 55 printed cells of 105), and the
machine-generated pin rule (no hand-transcribed hex anywhere). Every remaining risk is
named with its stop rule; any row whose bar cannot be proved from the sealed authorities is
a STOP, not a guess.

**Stage boxes.** A (the A0 authentication and reading) 600 / B (the inventory) 1,800 / C (the Q25 holds and the importer/index design) 600 / D (the implementation-brief draft and the risk table) 600 / E (the report and seal) 600 = **4,200 s** of the
9,778.939457101 s pool (no top-up; 5,578.939457101 s stays unallocated for the 745
implementation brief, which carries its own explicit top-up if the forecast requires it).
Each stage prints its CLOCK at entry and exit and stops at its deadline; the charge is
continuous from dispatch to the E seal.

**Hard rules.** No source or tool edits, no builds, no cargo, no test execution, no
acceptance suite, no inventory adoption, no network, no dependencies, no `unsafe`, no git;
all writes under `W`; the standing inventories are read-only; every quoted verdict line is
verbatim from a named sealed file and every excerpt is a subsequence of it
(`excerpt ⊆ raw`); the c243 record, the epoch, the 811 design and the historical records
stay sealed and read-only; the 744 stage is accepted and closed (this brief does not re-run
it); 746 and CM5 stay unauthorized; a second cdx refusal on this brief is a STOP.

**Report.** `rum-report/1` under `W`, `scripts/report.py check` clean and the detached
summary equal, the payload seal over `W` with every row re-hashed, the live entry/exit
census recorded, and the charge for every operation.

## Appendix P — the 77 pins (path, sha256, bytes)

| path | sha256 | bytes |
|---|---|---|
| .temp/cm4-shakedown/amendment-9/seal.json | 99b6fc78c890e00640e1ab36b672bcfe2dfd77d3627b95daad1088c72c202cc6 | 10801 |
| .temp/cm4-shakedown/amendment-9/seal.sha256 | cf298605b9687acd34dd35720f396f6e4b891be4e1f5f231cb50d54eeb7ff5f0 | 1247 |
| .temp/cm4-shakedown/amendment-9/payloads.tsv | 406a280c75b9b35b83332515ba7ae07229dbb395fa964be74ea167772fbc3f82 | 10562348 |
| .temp/cm4-shakedown/amendment-9/report.json | 0347a55e189386d5d55e86b5fbafca4c6c0435bb4dd287fd5d42fb42f114f583 | 5367885 |
| .temp/cm4-shakedown/amendment-9/summary.txt | c4dad07161cf7531d7f2a3dac30abad52e26b6c565da7dc2b2ca2c1c8d433b73 | 1175 |
| .temp/cm4-shakedown/amendment-9/charge.json | 782703cf1703d16a6e0c64cd5e84f8c990ba32e0ad682da3a2e8cdb5c5b0ac68 | 78427 |
| .temp/cm4-shakedown/amendment-9/stage-clock.json | 554f5e4debf330345291344d82470432e486af6538e4ba8c5b0edfe30f504ba8 | 279 |
| .temp/cm4-shakedown/amendment-9/full-schedule-ledger.json | 243c97cf489543b7d256ffa58784c452ba7d52b4a47b143fab67296c1e28294c | 1596547 |
| .temp/cm4-shakedown/amendment-9/forecast-745.json | b0c1600d7e36febb5add2af1ac1926b3f080044f91d0594a188e65f57670773a | 333179 |
| .temp/cm4-shakedown/amendment-9/freeze-745.json | 93a5aff5d3236be60a7083b3c1b9cacc3ae8d8e1b8f59a2f83f963f53085514b | 41184 |
| .temp/cm4-shakedown/amendment-9/held.json | 0c57e64d5d871be59de4d0cd6666875b2f882589107e182e0f45ec37c4e97de6 | 62629 |
| .temp/cm4-shakedown/amendment-9/unspent.json | a0640267c46ec389d3297d94e20052a95a13066e6eeaba903181c19cccc2c998 | 1220926 |
| .temp/cm4-shakedown/amendment-9/ledger-readback.json | a9803cdfdca8f2f781998a6581c473a39275398ff376662cad4620cb095e00a8 | 1493 |
| .temp/cm4-shakedown/amendment-9/excerpt-readback.json | aa6df569f77d95e42cdd91de8a6e36d8d8364d2dd94b1b540dc9fb231e26cf3e | 75872 |
| .temp/cm4-shakedown/amendment-9/archive-readback.json | c42144598aabe2af5d2d3d43f79aef8d530b7f632a09133b6dcad2ee1ed28f7f | 79922 |
| .temp/cm4-shakedown/amendment-9/identities-745.json | deea561b55818d093d1686c93a73f4987f02d91b22e370e4d0c6e751d6cc772c | 135753 |
| .temp/cm4-shakedown/amendment-9/original-rows-745.json | 963b20ff3c80ac384c055c69464aff3ffbbb133cf19f529085f76b0cda6c463c | 158385 |
| .temp/cm4-shakedown/amendment-9/remaining-745.json | 27c390440dee96a298e22a6d114dbb2a25a41e1a5a363bd1c55c34841cf57240 | 252679 |
| .temp/cm4-shakedown/amendment-9/negative-controls.json | 473bbc075dddfebb941826a790e844cf16fff4bc8589234aef2266639de012ed | 5521 |
| .temp/cm4-shakedown/amendment-9/generator-control-receipts.json | 920a61c17d48fe229dace4182beda05ae6a6b8acde2a7c74b41c0b2b14a16576 | 13650 |
| .temp/cm4-shakedown/amendment-9/measurement-readback.json | 6bcba3a4a4373c97a3370195033e2daeff3465082a95b573fff89b770a499ad4 | 1675 |
| .temp/cm4-shakedown/amendment-9/live-entry-stat.json | dac5358d59b1c4caf8695c2a681b288e9f230eea314a37382bbac709fea9fc20 | 792327 |
| .temp/cm4-shakedown/amendment-9/live-exit-stat.json | dac5358d59b1c4caf8695c2a681b288e9f230eea314a37382bbac709fea9fc20 | 792327 |
| .temp/cm4-shakedown/amendment-9/a0-pins.json | 4f0ed197a769c8db645917203dbb8b826364a3ed3d38c21471f788266f6f8310 | 6073 |
| .temp/cm4-shakedown/amendment-9/dispatch.json | 26171e8313fffd8841742b1e65b90178da7a3972a25ac70e819f3419a42e879e | 402 |
| .temp/cm4-shakedown/amendment-9/seal-invocation.json | 4e10c203c09dad94dde4ad9f4bd6e63af3aca3f4bb12ea1fbbbf693b09b900d4 | 992 |
| .temp/cm4-shakedown/amendment-9/report-timing.json | 070ec8350ae1d3adbced3e3ad035a6c82bf62ecc2709ac47e4b05511ac3138ee | 7261 |
| .temp/cm4-shakedown/amendment-9/first-seal/raw.log | 8d564062ac4c7db06892ef0cd28010058e7a7a0bc68e6018aa8f7531a4ec7fe1 | 1005 |
| .temp/cm4-shakedown/amendment-9/seal-check-reproduction.log | 8d564062ac4c7db06892ef0cd28010058e7a7a0bc68e6018aa8f7531a4ec7fe1 | 1005 |
| .temp/cm4-shakedown/amendment-9/parent-exit.log | db01e7edef1d38ba91c75670e7ccf72f26006f486e6d861bef487ec70642c18a | 11490 |
| .temp/cm4-shakedown/amendment-9/parent-readback.log | e40a4a408e3e4fe1bdd26b1f825fb84cd1ff2c829b8edfb4cbac948d36e6b195 | 14507 |
| .temp/cm4-shakedown/amendment-9/standing-readback.log | 718621fec28022c3703c338570ab76d1952527cd17052d128e1685f34191203b | 3145 |
| .temp/cm4-shakedown/amendment-9/ownership-readback.json | 6cc68db401c6b166a54b4b944c75888332e6d73599c97846d05995eb3c55a6eb | 27056 |
| .temp/cm4-shakedown/amendment-9/checkpoints.jsonl | cab6bdb9cc79efc991b9bc02f4f50ae091321331f016e8c28ed075e9eac48f85 | 256996 |
| .temp/cm4-shakedown/amendment-9/prior-timings.json | 69e4b6a110dfbd363c0d058a397311db935c0dece2353737c4961c68cc9d17e7 | 4205 |
| .temp/cm4-shakedown/amendment-9/delivery-readback.json | 13ce050f9718b6e5ff848cda19fa92cd757bfd54027422e6c6db050d807efb95 | 790 |
| .temp/cm4-shakedown/amendment-9/c242-live-delta.json | d6053e9edc49b6ea1f2d7b1b14ce42fb7a02f00b521e2883bb15795432507300 | 477 |
| .temp/cm4-campaign-epoch/amendment-17/subject.json | ce5d42a06cca7c0cf66fecbd0cb4dbf63acbf7e1ee4cf8547ede1257202dde44 | 16506 |
| .temp/cm4-campaign-epoch/amendment-17/epoch-identity.json | 8c1fdcd985d4f048c5c998fc32e2970481af607a70de45e504a0c6b08e2ddc49 | 19368 |
| .temp/cm4-campaign-epoch/amendment-17/acceptance-attestation.json | e29f754f7bca5e1b992f058f9df6c643b80c8e9fd6d5d9b2f3ab76b553a3473b | 2854 |
| .temp/cm4-campaign-epoch/amendment-17/stage-readback.json | 1a4b951d7fc79fc8cb268a2619ecdc461fcd1e8843d27579c0cc47d304ed7fa7 | 114121 |
| .temp/cm4-campaign-epoch/amendment-17/calibration/attestation.json | 13b95f09deeac59a1aa3b29867abf57d77af2159ef8be60c268da939857231a3 | 9589 |
| .temp/cm4-campaign-epoch/amendment-17/seal.json | 1a28c59609ad5752f03ec0f413f2ad9a280061160e17b82f74bef6ceb3e25111 | 5647 |
| .temp/cm4-campaign-epoch/amendment-17/payloads.tsv | 4068af8e3bd7fdb1d5566f95be5e60ac18cccc88be81fbef10a261561837a843 | 124724 |
| .temp/cm4-campaign-epoch/amendment-17/seal.sha256 | 0f77be786f7540eed9422ac2a228dc408bc94d96df47da2d1565a3fc1aea966b | 2735 |
| .temp/cm4-campaign-epoch/amendment-17/report.json | 0149407c474bf66c0db6aa46c17430fc2cdb37ed17335c25fb41939d81bf53c4 | 115791 |
| .temp/cm4-campaign-epoch/amendment-17/exit-audit.json | f3caa5539bbdacba92196ee93d94f1caf502e9048bac334e47bb8c4509397f66 | 46134 |
| .temp/cm4-campaign-epoch/amendment-17/charge.json | 1224e58b95a126a8675958e02c127cf4da4f9be2446bbe3c5cf0757ab8479e64 | 55266 |
| .temp/cm4-campaign-epoch/amendment-17/PROGRESS.md | 8fc0baf6d7347934a608496e019a90b088d7b5f7d5068b317564b1bd005316a8 | 9254 |
| .temp/cm4-campaign-epoch/amendment-17/receipts/registry.json | b7349e9c240fa2f5be3439d9b0fe8d0922fac3fd3a15a5613ad8d74021677659 | 17859 |
| .temp/cm4-campaign-epoch/amendment-17/freeze/identity.json | e1b0b20d9ffd32e725d9ae925c206024899d1dd41bc6c1d6ac68cbc800b5e62a | 3057 |
| .temp/cm4-campaign-epoch/amendment-17/indices/build-check.tsv | ae1befa71cd9388c9ec8a6d6483640f6f959686ceb2848050f447d0d62be2ceb | 2031 |
| .temp/cm4-campaign-epoch/amendment-17/indices/calibration.tsv | 18be26308d1432ffb8e088ce48c4bdca2e30a715634f629621adf54c4d7813c7 | 3275 |
| .temp/cm4-campaign-epoch/amendment-17/indices/class-skeleton.tsv | ae1befa71cd9388c9ec8a6d6483640f6f959686ceb2848050f447d0d62be2ceb | 2031 |
| .temp/brief-811/seal.json | da2fdf55a788f3a1e2b2e652590550ec4bf05497099a402a63c598b7e5d9097b | 3044 |
| .temp/brief-811/schedule-design.json | 8e3c4e251132fe9cab91d1dd41431e98e4eda9c86464c18f2f361acbb367525a | 362230 |
| .temp/brief-811/zero-unknown.json | e6b5ceb30de2ac97c4e01946d2bcff3c75fc6236818b8b78243a81fe77f589d2 | 14881 |
| .temp/brief-811/design-pins.json | 50a76530fba51569c717a44f8ee0a6c753bd6937e3375395704e7bf135eb9341 | 108865 |
| .temp/brief-811/indices.json | d4fd6a4b6c2c11d10eb617079c6fa0de620aef19acb9703f34922dedbf0046a0 | 21384 |
| .temp/brief-811/draft-744.md | 48129edc98d52bf279b15cb6530f1d10f6690ebdb8f006188947dce113651799 | 16653 |
| .temp/brief-811/draft-epoch.md | 04038321020ad68a051a8e98c48c8b462731382a808d2debc4f03262b9b207e1 | 25532 |
| .temp/brief-811/draft-readmission.md | 9701194457572bfdab177ee4a8b9290625cf6a13a8bcee3a8ac75c70eb8cfbb5 | 11597 |
| .temp/brief-756/historical-signatures.tsv | fd99e1a93cbf59540635597a72e2e688bc78179951f131fa165d532a5b39d2b9 | 55160 |
| .temp/brief-756/historical-events.tsv | 146850ac62151915a9466ec13e5029e6c7368fd7feffbfec88668612b4f102b5 | 41319 |
| .temp/brief-756/structural-controls.tsv | 10b47a79520439f5607597181740b47c644c55793e8a5500612dedc9dfb1e239 | 21543 |
| .temp/brief-756/historical-control-events.tsv | f34390fdf18f5004327e89fd38356b2196b32c1a8fb51b9d20843ceadb6d59ea | 934536 |
| .temp/brief-756/falsifier.md | 4f2d99ed777440c184013eb3868fcfecaa595dd61427e31887e6590f3a1a3ce6 | 9239 |
| .temp/brief-757/class-97.tsv | 92d72a8a9a74aa1592e4f97f40187b8b8bbb7137d27ea3aea528a1a0c4278dfd | 112114 |
| .temp/brief-755/full-schedule-ledger.json | 3e4532cf6acfe75c4b7d0971637b4ff021fe6f2aa824187e48bacef1d58775fe | 408724 |
| .temp/brief-755/class-admission.json | 6d556f4be3fb5763816e49fd427171ae5753a1b50d69f6800510b4c85db11e69 | 27691 |
| .temp/brief-755/packet-index.json | 3e1644aabb9e5cb579d441ef40049a85cccf4e8f027b78d52d405e3999974dde | 137763 |
| .temp/brief-755/ordinary-negatives.json | e42ced6718e18faa324f4f809a201ac94e6bf1c74142ecbaaa070fb9ed2049bd | 20185 |
| .temp/brief-755/authentication-entry.json | 9911a61417991897a10ec32f3f2ac7ad235d6a20a25fb63a39b0bd0a600597c4 | 54418 |
| .temp/brief-755/authentication-closing.json | 9911a61417991897a10ec32f3f2ac7ad235d6a20a25fb63a39b0bd0a600597c4 | 54418 |
| .temp/brief-741/SIGNATURES.md | 1df0ce00fd2fed44b404470c93078a868247047cdfdea75f54fde2067b7580fc | 51763 |
| .temp/brief-741/epoch-signatures.json | 0495aa10008e82472c13fdf5ec629958afcd3503df3d3b718f73ce97f9a384a7 | 71444 |
| .temp/brief-744/report.json | 314ca69da3f180178d63033996fca5aa1e125dc9b89ce11edc269732f7907320 | 374260 |
---

## Director gate c244 (2026-11-28) — DONE

The design is accepted. The attempt (unit `brief-815-cdx-20261128-073411-212201`, Codex
v0.172.0, gpt-6.2-astra, effort max, workspace-write, MemoryMax=16G, session
`01a21f5b-fb2b-7cd3-9fd0-fa7c14b749b6`) dispatched 2026-11-28T07:34:11 +0800 → rc 0 at
≈08:06:32, 329,045 tokens, peak 2,152,284,160 B, oom 0/0, zero filter events, no
`.git-attempts`; stages A 165.718313093/600 s, B 878.191700255/1,800 s, C
242.355256511/600 s, D 233.595457434/600 s, E 396.826825496/600 s; charge
**1,916.687552789 s** continuous (dispatch 20,486,930,779,426 → seal readback
22,403,618,332,215 ns); cumulative **96,648.101547079** of 104,510.353451391 s; return
**2,283.312447211 s**; pool **7,862.251904312 s**; no top-up.

A0: the 77 Appendix P pins (25,410,640 B) all re-hashed 77/77; the epoch parent's 785 rows /
739,439,260 B and the c243 parent's 42,260 rows / 2,462,361,100 B all re-hashed (0 bad);
live entry 5,906 paths, delta against the c243 exit exactly `TODO.md`, brief 814 (the c243
appends) and brief 815; entry == exit == my own walk.

The inventory: 246 cells / 123 rows in original order (identity projection `deea561b…`
recomputed exact); 61 non-empty fields per cell; dispositions 71/13/7/32 rows =
142/26/14/64 cells; the 13 Q25 rows = the c243 held keys = the 757 `proof needed` rows, 26
identities, zero credit; 71 carried = `design-carried ∩ 745`; 39 non-class rows absent from
757; families 51, districts 37; all 246 primary witnesses and 5,439 distinct witness triples
verified byte-exactly in their 101 sealed files; 142 withheld verdicts and 142 install
contracts verbatim in the sealed class raw. Importer: 151 keys, 123 scheduled, 0 proven
native, conditional 0–32, 28 outside, lane 745 = 0, the 745-only view `caught=0
PortBlocked=151 settlements=0` in both modes. Forecast retained (7,148.96 / 8,286.74 /
19,172.71 s); the draft's 19,320 s boxes and six STOP forms verified (246 identity rows, 23
block quotes and 13 held quotes exact). The stated top-up 13,741.060542899 s is computed
against the pre-return remainder; against today's pool the payable top-up at any future
measurement dispatch is 11,457.748095688 s — bookkeeping only, no spend released.

Seal: 38 payloads / 13,365,529 B all re-hashed (tree 45 = 38 + 7 exclusions), 26 bindings
and the 18-row envelope rehashed, `report.py check` OK 112 citations, detached summary
equal. `release_to_745=0`, `release_to_746=0`, `design_credit=0`, CM5 false. `git status`
clean at `89889f61`; zero source changes; zero foreign writes.

The four first-attempt reader corrections are retained (runs + sealed logs), none
measurement-affecting; the empty `deviations` array is noted and future briefs keep that
field canonical.

**Ruling (the 745-preflight question): the measurement draft stays held; a design-only
witness-acquisition line for the 32 open rows is ordered next** — per-row source mechanism,
expectation classes, acquisition recipe on the sealed builds, authenticity contract,
falsifier matrix and the disposition rule from authenticated packet to measurement bar —
then its implementation, the director disposition gate, and the stage-745 measurement
implementation. Stage 745 remains stopped before measurement; 746 held; CM5 unauthorized;
Q25's thirteen STOPs stand. Records: ruling `.temp/director/ruling-896.md`; gate check
`.temp/director/gate-244/check244.py` → `check.out` 80 PASS / 0 FAIL.
