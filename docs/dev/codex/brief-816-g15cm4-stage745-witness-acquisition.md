# Brief 816 — G15-CM4 stage-745 witness-acquisition design

**Scope.** A design-only (`Scope: NONE`) proof-carrying design of the selected-program witness
acquisition for the **32 stage-745 rows (64 seats)** whose native measurement is absent from
every sealed authority — the open obligations that gate the stage-745 measurement dispatch
(ruling 896). For each row and both logical modes the design pins the source mechanism and
the pre-declared expectation classes from the sealed armed tree, the exact acquisition
recipe on the sealed six builds, the authenticity contract, the isolation falsifier, the
importer expectation and the disposition path (approved native bar / arm-level absence /
measured block / escalation). It also delivers the acquisition protocol, the falsifier
matrix, the director disposition schema and the draft of the acquisition implementation
brief. **No execution of any row, no source edit, no build, no cargo, no repository suite,
no network, no new dependency, no `unsafe`, no git; no 744, 745 or 746 measurement and no
CM5.** Every write goes under `WA = .temp/cm4-shakedown/stage-745-acquisition-design/`.

**Pins (machine-generated over the actual files; any difference is a STOP).** All 82
`path` `sha256` `bytes` rows are listed in **Appendix P** below. The roots: the stage-745
design record (`W15 = .temp/cm4-shakedown/stage-745-design/`; seal `cab76d3996c39f8860ed1a3622fc0a97b4715eeae4d6d49c5fabfa790bdbc412` with 38 payloads /
13,365,529 B), the c243 shakedown record (`W9 = .temp/cm4-shakedown/amendment-9/`; seal
`99b6fc78c890e00640e1ab36b672bcfe2dfd77d3627b95daad1088c72c202cc6` with 42,260 payloads /
2,462,361,100 B), the campaign epoch `E = .temp/cm4-campaign-epoch/amendment-17/`
(subject `ce5d42a0…`, `epoch-identity.json` `8c1fdcd9…`, seal `1a28c596…` with 785
payloads / 739,439,260 B, the six build ELFs and their authentication receipts), the 811
design seal `da2fdf55…` (30 files, 632 pins), the 757 `class-97.tsv` `92d72a8a…`, the 756
witnesses, the 755 authority, the 741 signatures and the 744 report `314ca69d…`. A0
re-hashes all 82 pins, every payload row of the W15 (38), the epoch (785) and the c243
(42,260) seals, and records the live census against the W15 exit; any mismatch stops.

**The 32 open rows.** They are exactly the `disposition = "missing native measurement"`
rows of `W15/inventory-745.tsv` (32 rows, 64 seats), listed in `W15/open-obligations.json`
with their selected programs and `W15/draft-745.md`'s open-evidence table. The first stop
is `C3.evaluate_call.unstable_head`, slots 1 and 256. The other 91 rows per mode are out of
scope and must not change: 71 design-carried class fixtures, the 13 Q25 holds (P0214,
P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418 — zero
credit, no packet) and the seven source-proved measured blocks. The mechanism classes are
the inventory's: `around_hit`, `around_ok`, `around_some`, `around_ok_profile_return`,
explicit guard, and the auxiliary queue drive (P1470/P1471, whose archived `AlreadyResumed`
drive used a different selected program from the scheduled Middle drive).

**The acquisition inventory (deliverable 1).** For every one of the 32 rows, and both
modes (64 seats), produce a table with these fields, each backed by a named sealed line:

1. **Identity** — row, both slots and seats, configuration, tuple, selected program,
   profile group, selected constituents, input plan, expected stage (from `W15`).
2. **Mechanism and source** — the arm site, the hit condition and its guard branch read
   from the sealed armed tree (`roles/armed`), verbatim at `path:line`; the frame/hit
   semantics of the row's `around_*`/explicit-guard form; and a source-level argument,
   with named witnesses (the program's prompt/answer sequence and the call sites it
   drives), stating whether and where the selected program's path reaches the arm site.
   For the profile-return rows, the profiled and unprofiled branches and the exact frame
   each selects. For P1470/P1471, the queue-side mechanism and the admissible drive (see
   below).
3. **Expectation classes** — the pre-declared, mutually exclusive observation classes for
   the seat, each with its named cause: `Unarmed` (the selector never arms — the program
   does not execute the arm site), `Armed-NoHit` (arms, the hit predicate/frame never
   holds — name the condition), `Armed-Hit` (the exact carrier, phase, observer,
   predicate, prompt order and multiplicity per constituent), plus the mode-specific
   expectation for the profile-return and queue-drive rows. An observation outside the
   pre-declared classes is a contradiction and stops — never an inferred catch.
4. **Recipe** — the exact sealed binary per mode among the six (`default-` and
   `no-default-` × `real`, `disabled`, `armed`), the selector
   `oracle::mutations::source_worker::cm_copy_observation_worker --exact --nocapture
   --test-threads=8`, the env (`RUM_CM740_ROW`, `RUM_CM740_ARM`, `RUM_CM740_PROFILE`,
   `RUM_CM740_PROGRAM`, `RUM_CM740_BASELINE`, and the `OMIT`/`RECEIPT`/`PACKETS`
   isolation), the cwd, the raw path, and the single-row import view built from its own
   captures (the 12-column TSV of `W15/importer-design.json`).
5. **Isolation falsifier** — per row or per named family the control that proves the
   packet can fail: wrong row, wrong program, unarmed build, hit-before-arm, off-profile,
   or the missing-arm control — with the expected rejection line and its sealed precedent
   (the 744 negative-control records and the observed `Unarmed`/`NoHit` semantics).
6. **Importer expectation** — the row's 151-key position, whether it can produce a native
   receipt, the expected single-row target stage (`RawArtifact` in the ordinary view, the
   `BuildIdentity` rejection where the six-build view omits it), and the exact
   reconciliation line `required=151 caught=<n> PortBlocked=<151−n> settlements=0`. A
   745-only view never imports a 743/744 receipt.
7. **Disposition path** — the exact tuple that becomes the measurement bar when the
   packet is authenticated (carrier, phase, observer, predicate, multiplicity, prompt
   sequence, arm-before-hit order, both modes), the evidence that supports arm-level
   absence or a measured block with zero credit, and the escalation rule.
8. **Zero-unknown statement** — all 32 rows and 64 seats carry every field above or a
   named missing measurement; no blank, no guess; state the count per mechanism class and
   the **0 unknown** line.

Publish `WA/acquisition-inventory.tsv` (one row per seat) and
`WA/acquisition-inventory.json` (each seat's citations: path, line, verbatim text —
sealed-line or byte-slice form), plus the per-family summaries.

**The acquisition protocol (deliverable 2).** The authentication contract: the sealed
builds, role trees and receipts bound by hash; the source presentation under the view; the
fresh per-seat process with `RUST_TEST_THREADS=8`, stdin `/dev/null`, locale C, per-seat
home/temp/cache under `WA`; the raw bytes retained and hashed before parsing; the record
format (argv, cwd, env, mode, digest, exit, complete stdout/stderr); a **hit packet must be
reproduced once at one thread with identity-normalized equality**, or the row escalates; a
single run earns no schedule credit — acquisition is observation, and the measurement
later earns credit by exact reproduction against the approved bar. Job caps: worker 150 s,
importer/control 180 s, clipped to the stage. Charge continuous from dispatch to the seal.

**The falsifier matrix (deliverable 3).** Every control class across the 32 rows: the
wrong-row, wrong-program, unarmed and before-arm controls on a representative seat per
mechanism family, the profile-off twin for the profile-return rows, the queue-side control
for P1470/P1471, and the importer-target controls (omitted key → `RawArtifact` /
`BuildIdentity`; a 744 receipt in a 745-only view must not be importable). Each control
states its expected rejection line, the sealed precedent it follows, and what its failure
means (a fabricated or mis-bound packet). The matrix must be complete enough that every
expectation class in deliverable 1 is discriminated by at least one control.

**The disposition schema (deliverable 4).** The exact per-row approval artifact the
director signs before any measurement dispatch: row, both modes, packet receipt hashes,
observed class, the bar tuple, the controls exercised and their results, the importer
evidence, the credit rule, and the decision (`native bar approved` / `measured block`,
zero credit / `escalated`). A contradiction between the observation and the pre-declared
classes, a failed control, or a missing reproduction stops the disposition; nothing is
disposed by argument. The schema must make the 32 rows' resolution mechanical and
auditable.

**The draft acquisition implementation brief (deliverable 5).** Draft the acquisition
implementation in the `TEMPLATE.md` shape: the 64 seats in original order (default then
no-default), each row's recipe and expectation classes exactly as inventoried, the
controls, the reproduction rule, the raw/record format, the disposition table output, the
stage boxes sized from the 744 own-row evidence (the 18 no-default own rows and the
50-program arm runs), and the report format. Carry the risk table: the six 744 class-form
STOP forms (`W15/draft-745.md`, quoted with their sealed record excerpts) plus the
acquisition-specific risks — cross-mode contamination (a default packet bound to a
no-default seat), non-reproduction (a hit packet that does not reproduce at one thread),
stale copies (a role tree or build whose hash moved), importer confusion (a 744 receipt
leaking into a 745-only view), and queue-drive absence (no admissible P1470/P1471 drive in
the sealed assets). The draft states its budget against the then-current pool; the
acquisition brief carries its own explicit top-up if required.

**The auxiliary queue drive rows.** P1470 and P1471 (slots 98/353, 99/354) are scheduled on
`C8.fixedpoint.P1470.Middle.phase_observation.trace` and
`C8.fixedpoint.P1471.Middle.phase_observation.trace`; the archived `AlreadyResumed` queue
drive is a different selected program and cannot stand in. The design must either prove the
admissible queue-side drive from the sealed assets (naming the exact program, the arm side
and the observation) or dispose the two rows as a measured block with zero credit, naming
the missing drive as the mechanism — never a guess.

**Stage boxes.** A0 (authentication and census) 300 / A (the 32-row mechanism and
expectation inventory) 2,000 / B (the acquisition protocol, falsifier matrix and
disposition schema) 900 / C (the implementation-brief draft and risk table) 500 / D (the
zero-unknown reconciliation) 200 / E (the report and seal) 300 = **4,200 s** of the
7,862.251904312 s pool (no top-up; 3,662.251904312 s stays unallocated for the acquisition
implementation, which carries its own explicit top-up). Each stage prints its CLOCK at
entry and exit and stops at its deadline; the charge is continuous from dispatch to the E
seal.

**Hard rules.** No source or tool edits, no builds, no cargo, no test execution, no
acceptance suite, no inventory adoption, no network, no dependencies, no `unsafe`, no git;
all writes under `WA`; the standing inventories and every sealed record are read-only;
every quoted verdict line is verbatim from a named sealed file and every excerpt is a
subsequence of it (`excerpt ⊆ raw`); the 744 and 745 records, the epoch, the 811 design and
the historical records stay sealed and read-only; the Q25 holds and the seven
source-proved rows stand untouched; 745 measurement, 746 and CM5 stay unauthorized; a
second cdx refusal on this brief is a STOP.

**Report.** `rum-report/1` under `WA`, `scripts/report.py check` clean and the detached
summary equal, the payload seal over `WA` with every row re-hashed, the live entry/exit
census recorded, and the charge for every operation.

## Appendix P — the 82 pins (path, sha256, bytes)

| path | sha256 | bytes |
|---|---|---|
| TODO.md | 118f5b7bccb9ed5e71f0e143a1fe5f093b0a110cb565fb63e60a4fe96b45b541 | 1211891 |
| docs/dev/codex/brief-815-g15cm4-stage745-inventory.md | 1d1f323ee82888ec8d28f734275730611dfc1db168af6d22d5a1e09dae67d0cc | 21698 |
| .temp/cm4-shakedown/stage-745-design/seal.json | cab76d3996c39f8860ed1a3622fc0a97b4715eeae4d6d49c5fabfa790bdbc412 | 8677 |
| .temp/cm4-shakedown/stage-745-design/seal.sha256 | 24a51b2d8c4c5695ffe106b3cd8bc56bd0bfd0cfca40ad4d9ed74b1e9ad9d5b2 | 2149 |
| .temp/cm4-shakedown/stage-745-design/payloads.tsv | f73b90379e8234ef1d7feb8689cdb573b4cf05190e17a8c12708b2f404c8e936 | 4870 |
| .temp/cm4-shakedown/stage-745-design/report.json | ea0f4ae49da6fc39eb8b258984da4356833b3a593ea7901607ce86ae9c6e58d5 | 55534 |
| .temp/cm4-shakedown/stage-745-design/summary.txt | 6a18f52d6d0e9dbd2dc673637d98efe3a6e0ee094bdc53750fa7114cfd4b3bf7 | 849 |
| .temp/cm4-shakedown/stage-745-design/inventory-745.tsv | 889ccf8eaec4e6733afc5297648269adfe1d1330d568343a27cff1a7eb9c6211 | 1639354 |
| .temp/cm4-shakedown/stage-745-design/inventory-745.json | ef7009be945a297cd37b26d2a1efdcfa6b5b820e7213ee1766f89e49724a1262 | 5634910 |
| .temp/cm4-shakedown/stage-745-design/open-obligations.json | 6ee322a713d05912e4b9aa7777ab873a998c9f25a2c1a712f6090502a250be68 | 263257 |
| .temp/cm4-shakedown/stage-745-design/q25-holds.json | 895d91a5769b25e6f0da90a9d7643e471606fc4df7767035c2e2ccd6716a9d21 | 108060 |
| .temp/cm4-shakedown/stage-745-design/importer-design.json | dd1323d9c8aa20df2825d511977154ed56c08bb8354eaa8ccb470f31482f4051 | 270972 |
| .temp/cm4-shakedown/stage-745-design/implementation-budget.json | 360d8b47e93cfc7b62ef6fa354ce2a717dab9485bb41fec3e7ca8c5287a035f3 | 2821 |
| .temp/cm4-shakedown/stage-745-design/draft-745.md | f160c4774b83a12aa5faaaa183b9c3a8efae31b4501cf26dc356a48bf80f7114 | 140192 |
| .temp/cm4-shakedown/stage-745-design/composition-745.json | d8cac1cbb78a9d850db0f42fafae896b2b3fec428883f3aed750f18b0409ca8a | 92246 |
| .temp/cm4-shakedown/stage-745-design/fixture-history.json | 792dc08e84ea81af144ba714329c1099c1ab6b5799de3765f25e755553adae8b | 942644 |
| .temp/cm4-shakedown/stage-745-design/report-check.log | a192bb8c6481f22707abfb731e37e75213902a6a18b2064ec0a63bfd90356604 | 48 |
| .temp/cm4-shakedown/stage-745-design/seal-readback.txt | 27f168feb5f2103643c7bb5367ff658b06c86fea2c3ce0c8e27cd1ee2e115bc4 | 678 |
| .temp/cm4-shakedown/stage-745-design/live-exit-stat.json | 812733fb315bac7804e50fc72a1fda441cb9f40b14d16064b9b22e00f103abab | 922350 |
| .temp/cm4-shakedown/stage-745-design/live-exit-content.json | b9a01fafed697e4b909141dd655e273137bebdd704d60c7899286370b4d45a40 | 1020226 |
| .temp/cm4-shakedown/amendment-9/seal.json | 99b6fc78c890e00640e1ab36b672bcfe2dfd77d3627b95daad1088c72c202cc6 | 10801 |
| .temp/cm4-shakedown/amendment-9/seal.sha256 | cf298605b9687acd34dd35720f396f6e4b891be4e1f5f231cb50d54eeb7ff5f0 | 1247 |
| .temp/cm4-shakedown/amendment-9/payloads.tsv | 406a280c75b9b35b83332515ba7ae07229dbb395fa964be74ea167772fbc3f82 | 10562348 |
| .temp/cm4-shakedown/amendment-9/report.json | 0347a55e189386d5d55e86b5fbafca4c6c0435bb4dd287fd5d42fb42f114f583 | 5367885 |
| .temp/cm4-shakedown/amendment-9/summary.txt | c4dad07161cf7531d7f2a3dac30abad52e26b6c565da7dc2b2ca2c1c8d433b73 | 1175 |
| .temp/cm4-shakedown/amendment-9/full-schedule-ledger.json | 243c97cf489543b7d256ffa58784c452ba7d52b4a47b143fab67296c1e28294c | 1596547 |
| .temp/cm4-shakedown/amendment-9/forecast-745.json | b0c1600d7e36febb5add2af1ac1926b3f080044f91d0594a188e65f57670773a | 333179 |
| .temp/cm4-shakedown/amendment-9/freeze-745.json | 93a5aff5d3236be60a7083b3c1b9cacc3ae8d8e1b8f59a2f83f963f53085514b | 41184 |
| .temp/cm4-shakedown/amendment-9/held.json | 0c57e64d5d871be59de4d0cd6666875b2f882589107e182e0f45ec37c4e97de6 | 62629 |
| .temp/cm4-shakedown/amendment-9/remaining-745.json | 27c390440dee96a298e22a6d114dbb2a25a41e1a5a363bd1c55c34841cf57240 | 252679 |
| .temp/cm4-shakedown/amendment-9/original-rows-745.json | 963b20ff3c80ac384c055c69464aff3ffbbb133cf19f529085f76b0cda6c463c | 158385 |
| .temp/cm4-shakedown/amendment-9/identities-745.json | deea561b55818d093d1686c93a73f4987f02d91b22e370e4d0c6e751d6cc772c | 135753 |
| .temp/cm4-shakedown/amendment-9/unspent.json | a0640267c46ec389d3297d94e20052a95a13066e6eeaba903181c19cccc2c998 | 1220926 |
| .temp/cm4-shakedown/amendment-9/negative-controls.json | 473bbc075dddfebb941826a790e844cf16fff4bc8589234aef2266639de012ed | 5521 |
| .temp/cm4-shakedown/amendment-9/generator-control-receipts.json | 920a61c17d48fe229dace4182beda05ae6a6b8acde2a7c74b41c0b2b14a16576 | 13650 |
| .temp/cm4-campaign-epoch/amendment-17/seal.json | 1a28c59609ad5752f03ec0f413f2ad9a280061160e17b82f74bef6ceb3e25111 | 5647 |
| .temp/cm4-campaign-epoch/amendment-17/seal.sha256 | 0f77be786f7540eed9422ac2a228dc408bc94d96df47da2d1565a3fc1aea966b | 2735 |
| .temp/cm4-campaign-epoch/amendment-17/payloads.tsv | 4068af8e3bd7fdb1d5566f95be5e60ac18cccc88be81fbef10a261561837a843 | 124724 |
| .temp/cm4-campaign-epoch/amendment-17/report.json | 0149407c474bf66c0db6aa46c17430fc2cdb37ed17335c25fb41939d81bf53c4 | 115791 |
| .temp/cm4-campaign-epoch/amendment-17/subject.json | ce5d42a06cca7c0cf66fecbd0cb4dbf63acbf7e1ee4cf8547ede1257202dde44 | 16506 |
| .temp/cm4-campaign-epoch/amendment-17/epoch-identity.json | 8c1fdcd985d4f048c5c998fc32e2970481af607a70de45e504a0c6b08e2ddc49 | 19368 |
| .temp/cm4-campaign-epoch/amendment-17/charge.json | 1224e58b95a126a8675958e02c127cf4da4f9be2446bbe3c5cf0757ab8479e64 | 55266 |
| .temp/cm4-campaign-epoch/amendment-17/PROGRESS.md | 8fc0baf6d7347934a608496e019a90b088d7b5f7d5068b317564b1bd005316a8 | 9254 |
| .temp/cm4-campaign-epoch/amendment-17/receipts/registry.json | b7349e9c240fa2f5be3439d9b0fe8d0922fac3fd3a15a5613ad8d74021677659 | 17859 |
| .temp/cm4-campaign-epoch/amendment-17/freeze/identity.json | e1b0b20d9ffd32e725d9ae925c206024899d1dd41bc6c1d6ac68cbc800b5e62a | 3057 |
| .temp/cm4-campaign-epoch/amendment-17/indices/build-check.tsv | ae1befa71cd9388c9ec8a6d6483640f6f959686ceb2848050f447d0d62be2ceb | 2031 |
| .temp/cm4-campaign-epoch/amendment-17/indices/calibration.tsv | 18be26308d1432ffb8e088ce48c4bdca2e30a715634f629621adf54c4d7813c7 | 3275 |
| .temp/cm4-campaign-epoch/amendment-17/indices/class-skeleton.tsv | ae1befa71cd9388c9ec8a6d6483640f6f959686ceb2848050f447d0d62be2ceb | 2031 |
| .temp/cm4-campaign-epoch/amendment-17/bin/default-real-oracle | d3d047979428861d9fbfebd5f3278a694a72a4238657b04e8afeed72239fb4e1 | 73107432 |
| .temp/cm4-campaign-epoch/amendment-17/receipts/default-real-authentication.tsv | 094e06ea3547a282401470f2e5a822c0462e65b97c72c33cfa6a91081de872d7 | 326 |
| .temp/cm4-campaign-epoch/amendment-17/bin/default-disabled-oracle | 7b66ebf7e709bd7cb8e93f5d323acf7c8c485b267ae800a890bd3b7512e36ecb | 81038960 |
| .temp/cm4-campaign-epoch/amendment-17/receipts/default-disabled-authentication.tsv | d08175bc436c4333b10c413ea8d433692bfec06f5b8a70151ee6ad98becabb3e | 33612 |
| .temp/cm4-campaign-epoch/amendment-17/bin/default-armed-oracle | 9602f075ace66a93adb847f4c268531e179a8f474c9f19eaa7d1ca4437ad2dd2 | 84493536 |
| .temp/cm4-campaign-epoch/amendment-17/receipts/default-armed-authentication.tsv | 6f3ddc64f83cfebeef53323d36fa7c60d7849ca5ac81936bae3613e7be609367 | 33609 |
| .temp/cm4-campaign-epoch/amendment-17/bin/no-default-real-oracle | b2cd82c5e2b4cd077d4fe3bb14fbea2a263547b5aeb2223b2b18327ca47ce61c | 73022376 |
| .temp/cm4-campaign-epoch/amendment-17/receipts/no-default-real-authentication.tsv | 1357baa7611053541d60e9e6ad3d1b05b20cba571fcfa87b7aca564b8603524b | 329 |
| .temp/cm4-campaign-epoch/amendment-17/bin/no-default-disabled-oracle | a0c74a7759ec8da01de9f5df699e8df2aa4c118c315bbf003f09efd28d744809 | 80957656 |
| .temp/cm4-campaign-epoch/amendment-17/receipts/no-default-disabled-authentication.tsv | 1cd5330e3f208669e5eff1be4a90275efa1885cdf02906e6584976db8a6c6005 | 33615 |
| .temp/cm4-campaign-epoch/amendment-17/bin/no-default-armed-oracle | a2e34582ac07fd99cc0cd612b0918736c13ec530cac6176168c7042afbc5cecb | 84412384 |
| .temp/cm4-campaign-epoch/amendment-17/receipts/no-default-armed-authentication.tsv | d60d8497c99eb9c2bea328f6e4169ed859d4201ba0666907cb706a848167113b | 33612 |
| .temp/brief-811/seal.json | da2fdf55a788f3a1e2b2e652590550ec4bf05497099a402a63c598b7e5d9097b | 3044 |
| .temp/brief-811/schedule-design.json | 8e3c4e251132fe9cab91d1dd41431e98e4eda9c86464c18f2f361acbb367525a | 362230 |
| .temp/brief-811/zero-unknown.json | e6b5ceb30de2ac97c4e01946d2bcff3c75fc6236818b8b78243a81fe77f589d2 | 14881 |
| .temp/brief-811/design-pins.json | 50a76530fba51569c717a44f8ee0a6c753bd6937e3375395704e7bf135eb9341 | 108865 |
| .temp/brief-811/indices.json | d4fd6a4b6c2c11d10eb617079c6fa0de620aef19acb9703f34922dedbf0046a0 | 21384 |
| .temp/brief-811/draft-744.md | 48129edc98d52bf279b15cb6530f1d10f6690ebdb8f006188947dce113651799 | 16653 |
| .temp/brief-811/draft-epoch.md | 04038321020ad68a051a8e98c48c8b462731382a808d2debc4f03262b9b207e1 | 25532 |
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

## Director gate c245 (2026-11-28) — DONE

The design is accepted. The attempt (unit `brief-816-cdx-20261128-082218-240206`, Codex v0.172.0,
gpt-6.2-astra, effort max, workspace-write, MemoryMax=16G, session `01a21f88-0905-7932-a906-f8f4ff6d397a`)
dispatched 2026-11-28T08:22:18.658686+0800 and **died on cdx's usage limit** at 08:37:13.045201
(reset 2026-12-05T01:44 +0800): 894.386515 s, 207,485 tokens, peak 297,517,056 B, oom 0/0, rc 1,
zero git attempts.

A0 PASS: 82 Appendix P pins (512,579,492 B) re-hashed; the three parent payload sets re-hashed in
full (W15 38/13,365,529 B, epoch 785/739,439,260 B, c243 42,260/2,462,361,100 B) with envelopes;
three role trees 5,851 files each (`b0bdcee9…`/`084f8cc8…`/`d5267244…`); live 5,907 = the W15 exit
5,906 plus TODO.md `118f5b7b…`, brief 815 `1d1f323e…`, brief 816 `ec4c5038…`.

A complete: the 64-seat acquisition inventory (1,850,728 B TSV × 36 fields; 8,750,009 B JSON) with
21,946 citations in 111 sealed files, the matrix bindings (2,945 keys), six families (rows
5/5/9/5/2/6; seats 10/10/18/10/4/12) and one named missing measurement per seat — `A complete 64
2945`; B–E never entered.

The director takeover (entry 08:43:04.356844 +0800; no worker route until grk ≈2026-11-29, cdx
2026-12-05) delivered B–E: the acquisition protocol (fff5cebb…), the falsifier matrix (292 controls:
64 live, 228 transcript, every one with `applies_when`, an expected rejection line and a sealed
precedent), the disposition schema/template (406bb899…), the zero-unknown reconciliation
(`ZERO UNKNOWN rows=32 seats=64 unknown_rows=0 unknown_seats=0 named_missing_measurements=64`), the
implementation-brief draft (7d94c0de…, 43 machine-generated pins over the actual files, 64 seats in
original order, the six 744 STOP forms verbatim from `draft-745.md:89–96`, five acquisition-specific
risks, boxes sized from the measured 744 own-row medians), and the report/seal (8933db64…, check OK
44; 35 payloads / 16,369,136 B all re-hashed; envelope re-hashed; live entry == exit at 5,907).

Charge: attempt 894.386515 + takeover 428.758802183 = **1,323.145317183 s** of the 4,200 s allocation
(2,876.854682817 unspent; no top-up); pool **6,539.106587129 s**; cumulative **97,971.246864262** of
104,510.353451391. The stage-745 measurement stays held behind the 32 dispositions; the acquisition
implementation (brief 817 from the draft) is the next line, carried by the director unless a worker
route returns first. Paused after this gate per the user's instruction. Records: ruling
`.temp/director/ruling-897.md`; gate check 67 PASS / 0 FAIL; attempt record
`.temp/director/816-quota-1/attempt.json`; log archived
`.temp/logs/archive/cdx_brief_816.quota-1.log`.
