# Brief 762 — G15-CM4 Q25 ownership proof (P2379), stage five: the certificate-table adoption, the four certificate replays, one proof-only build and one bounded fixture invocation with the full harness name

**Director-dispatched, one stage, one gate.** This continues brief 761's STOP under ruling
770: the two proof-file repairs compile, the twelve inventory replays and the release build
ran, and the one authorized fixture invocation matched zero tests because its filter lacked
the module qualification that `--exact` requires. The director has authorized the mechanical
adoption of the generator's review table, the four certificate replays, one proof-only build
and one bounded fixture invocation with the full harness name. The work runs in a fresh copy
of the 761 copy. The live tree stays untouched, `.temp/brief-761/`, `.temp/brief-760/`,
`.temp/brief-759/` and `.temp/brief-758/` stay read-only, no receipt is importable, and Q25's
thirteen rows stay named stops until the director rules after this gate.

## Goal

When this brief is done, in `.temp/brief-762/` only: the certificate table mechanically
adopted from the generator's review copy, the four certificate replays reproducing the
accepted baseline content, and a physical proof on the default P2379 row — a genuine
source-backed commitment on the actual allocation, the driver's delivery to that concrete
`Arc<Self>`, one legitimate omitted append, the second real append, and `Err(Output)` on
unqueued; the withheld variant with identical physical operations and exactly `NoHit` while
both paths still execute; two same-state siblings rejected at the concrete receiver in both
directions; the lifetime/custody negatives — with individual dispositions for all thirteen
Q25 rows. The proof produces **zero importable receipts** and no admitted receipt.

## Context pointers

- Brief 761 (committed `4a0fb65b`) and ruling 770 (`.temp/director/ruling-770.md`); its scene
  `.temp/brief-761/`: `report.json`, `ledger.json`, `checkpoint.json`, `evidence/`, the raw
  replay logs, `logs/fixture.log` (the zero-test run), the release binary
  `.temp/brief-761/build/release/deps/suspension_oracle-547fa4362abc58a1` (`34883b9f…`), and
  the generator's review table `.temp/brief-761/copy/target/brief-544/local-contracts.tsv`
  (91,646 B, `85d75662c581acb62bd8a4fe44d61b25a7ec80e44abf490dac45f4b893da2434`).
- **The 761 copy (the entry).** `.temp/brief-761/copy`, source tree (excluding the generated
  `target/` and `.temp/` directories) 5,812 files,
  `918e0c38e1dc7ed279ae2a7682563565e2e319454ce6ee5c539d9a69fdd18c3e`. It carries,
  byte-for-byte:
  - `crates/rum-runtime/src/lib.rs` — 258,973 B, `15d91070bbad767e098d5d615c110c8597198d57c50d6e4d96686664540691fc`
  - `crates/rum-runtime/src/cm740_probe/q25_ownership.rs` — 29,522 B,
    `43416be62901a79c797f057b81a7364b668a91099188098250dbeaaaa68f2052`
    (`crates/rum-runtime/src/q25_ownership.rs` must be absent)
  - `crates/rum-runtime/src/runtime/suspension/core.rs` — 33,984 B,
    `7766b20ef78e7064c648f4e56d912daea4108686b6239627c0b1cdfd69b9dbf4`
  - `crates/rum-runtime/src/runtime/suspension/core/driver.rs` — 9,474 B,
    `7f800a7f459db7c9193b77ee8f9e8336c1d55ebe546dcfdee3d83412bb893f19`
  - `crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs` — 29,274 B,
    `2d915381edb0a12781c0eeb722bd6bef1d964573f8f3f19d9354709603b2f66e`
  - `crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs` — 20,948 B,
    `8e996d074f5a6dc7b4073ec1bae633825e2db4775be769b0dc0141c482eea370`
  - `crates/rum-stdlib/tests/suspension_oracle.rs` — 1,761 B,
    `7dd5b6846ad6a14a34ba93f4b2ea88a0ba50065cedd6097922048fcd6914495f`
  - `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs` — 32,082 B,
    `0fc8d966ce31f2a9253b1a79f56c73ebbd0bf69a232f3cbe7efedc997c8f5e98` (the repaired proof
    test: clone-before-emit, fail-closed selector first, env-mint negative and 17 negatives
    intact, no `unsafe`)
  - `crates/rum-stdlib/tests/suspension_oracle/mutations/records.rs` — 14,894 B,
    `7364f150437a0d395caa6843dabbb99a0c2500443faaa6ad8951f6af2eb1aa28`
  - and the source inventory
    `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` — 91,646 B,
    `6b390887716c384d35f6a321912ca9212b04bd1f1b8518e9b491139894735fd3` (the pre-adoption
    column).
- **The adoption.** The generator's review table (`target/brief-544/local-contracts.tsv`) is
  the source inventory with only its `fingerprint` column refreshed: identical header, 68
  rows, 68 changed in that column, zero other cells differ. The q25 edits changed exactly one
  file of the contracts' 34-file flat `DEPENDENCIES` list
  (`crates/rum-runtime/src/runtime/suspension/core.rs`), which moves every contract mark —
  that is the ordering artifact brief 761 recorded. Adopting the generator's output is the
  mechanical, generator-produced repair; a hand edit or a predicate change is forbidden.
- **The accepted baseline content to reproduce.** Certificate emit (default and no-default):
  rc 101, panic at `crates/rum-stdlib/src/tests/retention_closure/local/production.rs:99:6`
  with `production source anchor expired`. Certificate check (default and no-default): rc 0,
  printing the C01 rows for `merged_option_exprs_for_symbol` (Unarmed and AccountingShadow,
  rejection count 0), the C02 row for `evaluate_runtime_option_expr` (rejection count 0), and
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out`; the raw
  logs are `.temp/brief-754/attempt-2/evidence/standing/armed/certificate-{emit,check}-{default,no-default}.log`.
- **The fixture.** The existing default P2379 selection (program `e5e3a1ff…rum`, setup
  `ae59ceca…rum`, pins `mutant-coverage.tsv:138`, `programs.tsv:957`,
  `.temp/brief-749/attempt-7/renders/one/mutants.tsv:138`; answers prior and `"X"`; trace
  preserved). No new rum program. The harness name is
  `q25_p2379_proof::q25_p2379_ownership_proof` (`tests/suspension_oracle.rs:52–53`), already
  confirmed by listing the 761 release binary.
- **The links.** The same `path:line` set as briefs 759/760/761 (raw sink `core.rs:617–636`;
  raw driver `core/driver.rs:37–95`; `suspension/typed/entry.rs:9–19`; `lib.rs:1566–1608`,
  `:1636–1648`, `:2101–2113`, `:2223–2255`; the eight concrete receivers;
  `records.rs:341–366`).
- **The bars.** The 36 historical signatures, the 20 Stage C cells, C10, the pair
  `(45,5,false)` (200 cells), the four structural controls, the eight observer rejection
  families, the 97-row table. This proof adds no admission.

## Scope

- MAY create/modify: everything under `.temp/brief-762/` only — the working copy (including
  the adopted table and the generated target), its build target, logs and receipts, the
  report at `.temp/brief-762/report.json` and its ledger at `.temp/brief-762/ledger.json`.
- MAY read: the live source tree, the accepted epoch trees and binaries, `.temp/brief-761/`,
  `.temp/brief-760/`, `.temp/brief-759/` and `.temp/brief-758/` (read-only), every earlier
  `.temp/brief-*` record.
- MUST: at entry, copy `.temp/brief-761/copy` once to `.temp/brief-762/copy` by copying
  inputs or copy-on-write, **never writable hardlinks to the accepted files**; record the
  copy identity of the source tree (5,812 files excluding the generated `target/` and
  `.temp/` directories, `918e0c38…`), the ten rows above (including the review table), and
  device/inode independence.
- **Authorized invocations (exact).** (a) The adoption — a byte copy of
  `target/brief-544/local-contracts.tsv` (`85d75662…`) over
  `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`
  (`6b390887…`), with byte pre/postimages and a column proof that the `fingerprint` column is
  the only change (68/68 rows, identical header, zero other cells); (b) exactly the four
  recorded `local_production_acceptance` certificate commands — the two generate entries
  (`--lib local_production_acceptance -- --nocapture`, default and `--no-default-features`)
  and the two validate entries (the same two) — each run as recorded (`cargo test --locked
  --offline -j8`) through `scripts/run-bounded.py` with a timeout derived from the measured
  samples; the other nine replay commands from brief 761 are NOT rerun and their 761 verdicts
  stand; (c) exactly one default proof-only build of the `suspension_oracle` target in the
  working copy (`--offline --locked -j 8 --release --message-format=json --no-run`, the
  epoch's absolute `cargo`/`rustc`, the working copy's manifest, the locked registry
  resolvable offline; target dir and `TMPDIR` under `.temp/brief-762/`); (d) exactly one
  bounded single-threaded fixture invocation with the full harness name
  `q25_p2379_proof::q25_p2379_ownership_proof` (`--exact`, `--nocapture`, `--release`, the
  proof-built binary, `RUST_TEST_THREADS=1`, `RUM_CM740_OMIT=P2379`) through
  `scripts/run-bounded.py`; a `--list` preflight of the built binary (which runs no test) is
  allowed once to confirm the name before the invocation. No other cargo build or test
  execution may run; anything else is a gate for the director, not an automatic spend.
- MUST NOT: edit any live file; write under `.temp/brief-761/`, `.temp/brief-760/`,
  `.temp/brief-759/`, `.temp/brief-758/` or any sealed scene; compile the live tree; run git;
  touch the network; add a dependency or `unsafe`; add a program, a selection, a lane, a
  policy or an epoch label; produce an importable or admitted receipt. The membership stays
  28/3/7/113 and the pair `(45,5,false)` stays Open at zero admitted receipts.

## Deliverable

1. **Entry and the working copy.** Re-authenticate the 761 copy (source tree `918e0c38…`,
   5,812 files, the ten rows, the 759 path absent, the review table present); create the
   working copy and record its identity and inode independence; record every subsequent edit
   with byte pre/postimages.
2. **The certificate-table adoption.** Copy the generator's review table over the copy's
   source inventory as a single byte copy; record the preimage and postimage with sizes and
   hashes; parse both and show the header is identical, all 68 rows are present, the
   `fingerprint` column is the only differing column for all 68 rows, and no other cell
   differs. No other source edit; `records.rs` stays byte-identical to `7364f150…`.
3. **The four certificate replays.** Run exactly the four recorded
   `local_production_acceptance` commands and quote the actual verdict lines: the emit
   commands must panic at `production.rs:99:6` with `production source anchor expired` (rc
   101), and the check commands must print the C01/C02 rows with rejection count 0 and pass
   (rc 0, `test result: ok. 1 passed; 0 failed`), matching the accepted baseline; both modes
   freshly compiled. Any other difference is a STOP with the raw diagnostics.
4. **The one bounded execution.** Run the trial groups of brief 759 deliverable 4 unchanged —
   the ordinary reference (`{"X","X"}` both halves, both appends once, zero selected effects,
   `Ok(())`); the positive (one genuine commitment and one dispatch claim on the first
   allocation A, equality of install/front/concrete self, one ordered
   `OmitActualStringCaseAppend` hit, the empty accumulator preserved, then a normal second
   append, unqueued `{"X"}` vs queued `{"X","X"}`, `Err(Output)`, `public_output_exact=false`,
   Strict, the native bit from `W:142–149`); the withheld variant (the same source and genuine
   Err minus only the commit, both physical operations still run, zero claims/hits, both
   halves `{"X","X"}`, `Ok(())`, `public_output_exact=true`, the matching reference native
   snapshots, exactly **NoHit** at `records.rs:341–350` before `352–366`); the A/B/C
   equal-state siblings rejected both ways; the 17 negatives of `fixture-negatives.tsv`; and
   the lifetime/custody trials. Record pre/post strong counts, weak-upgrade failure after
   disposal, queue lengths, ordered callbacks/drops and clean state on normal and exceptional
   exit. Archive command, bounds, versions, code/manifests, image hash, result hashes, full
   observations, ordered application and diagnostic receipts. **Zero importable receipts.**
5. **The 13 dispositions and the report.** One disposition for each of the 13 Q25 rows: which
   actual anchors executed in this proof and which are supported only by the generic
   implementation plus static instantiation. A green design or one concrete execution does not
   prove physical reach or an own catch for the other rows. Fail the proof for a missing true
   receiver, false acceptance on either sibling, a different withheld rejection, a missing
   original operation, incorrect generation/custody, leaked authority, a changed guest outcome
   outside the one effect, unexplained inventory drift or a projected overrun. A failed common
   proof keeps all thirteen named stops for ruling 770. A green proof permits the director to
   decide whether to release the independent generator/source stage, still owing
   positives/withholding on each applicable selected path, both modes, and the companion
   queued operations. Never close a row, spend a further build, introduce a new observer or
   cut a negative to fit the carve.

## Hard rules

- `.temp/brief-761/`, `.temp/brief-760/`, `.temp/brief-759/`, `.temp/brief-758/`, the accepted
  epoch and every sealed scene stay read-only; the live tree is untouched; the working copy is
  disposable and non-admission. No live edit, no build of the live tree, no git, no network,
  no new dependency, no `unsafe`, no new program/lane/selection/policy/epoch label; `.temp`
  only; never `/tmp`; never read `~/.config/ccds/api-key`.
- The authorized invocations are exactly the adoption, the four certificate commands, one
  proof-only build and one fixture execution (plus the one `--list` preflight). Reprice cold
  cache and negative work at entry and at each irreversible spend. A projected overrun stops
  for a director ruling, never a scope cut.
- The replay bar is the verdict-line content against the accepted baseline; neither rc 0 nor a
  green suite is invented where the baseline records a failure. Never weaken a predicate,
  reclassify a row, exempt a stop, count a call edge as an authorization, count a text
  identity as a catch, or infer a runtime fact from the report.
- Quote every verdict line; a claim without `path:line` or a raw log line is not a verdict.
- The four structural controls, the 36 historical signatures, the 20 Stage C cells, C10, the
  pair `(45,5,false)`, the eight observer rejection families and the 97-row table are retained
  unchanged; this proof adds no admission and produces no importable receipt.
- The standing state is unchanged: bar 571/4,422, 37/134, 4/133, 0/26; the pair
  `(45,5,false)` Open at zero admitted receipts; C2 118 Residual + 0 Lost; the 51 owners and
  five source duties Open; membership 28/3/7/113; `CHANGELOG.md` and S10 blocked; CM5
  unauthorized; everything sealed (c76–c99) sealed; 745/746 held; no 744 restart.

## Verification (MANDATORY — you run it; iterate to green or to evidence)

- The entry reproduces the 761 copy source-tree identity (`918e0c38…`, 5,812 files) and the ten
  rows; the working-copy identity is recorded; every edit is itemized with byte
  pre/postimages; `records.rs` stays byte-identical to `7364f150…`.
- The adoption is a single byte copy from the generator's review table, and its column proof
  shows the `fingerprint` column is the only change (68/68 rows, zero other cells).
- The four certificate commands reproduce the accepted baseline content exactly (emit rc 101
  at `production.rs:99`; check rc 0 with the C01/C02 rows), both modes freshly compiled, every
  verdict line quoted with its raw log citation.
- The ordinary, positive, withheld, sibling and negative predictions are exact, and every one
  is either observed or the proof fails with its raw receipt. The `NoHit` is the exact
  `records.rs:341–350` rejection; the three-image importer of `records.rs:283–305` is
  explicitly deferred and never claimed as passed.
- The 13 dispositions are complete; zero importable receipts; the class-wide bars are
  unchanged; the report is `rum-report/1` at `.temp/brief-762/report.json` with
  `.temp/brief-762/ledger.json`; `python3 scripts/report.py check 762` is green with every
  PROBLEM fixed; the final message is only `python3 scripts/report.py summary 762`.
- Budget: **3,500 s focused** (a carve from the corrected reserve, 25,096.48887323717 →
  21,596.48887323717 s), **0 s workspace, 0 s epoch**; unspent returns; a projected overrun
  stops for a director ruling, never a scope cut. The carve prices the entry and adoption, the
  four certificate commands, the release proof build, the one fixture invocation (the 758
  forecast priced the fixture image at 347.34 s) and the close.

## Report format

`rum-report/1` at `.temp/brief-762/report.json` (cumulative with `.temp/brief-762/ledger.json`),
`changes` empty (every write is under `.temp/brief-762/`); sections for entry and the working
copy, the certificate-table adoption, the four certificate replays, the fixture execution and
raw observations, the predictions versus actual outcomes, the 13 dispositions, and the scope;
cite lines exactly. STOP at the proof gate: no live edit, no admission, no generator/source
stage, no 744 restart, no 745/746, no CM5.

## Gate c121 (2026-11-23) — STOP accepted; Q25 unwaived; brief 763 ordered

Brief 762 ran on `grk` (grok-4.7, effort xhigh), unit
`brief-762-grk-20261123-010120-850204`, session `d83cd504-18d2-4044-95d7-145bcded4c6f`,
2026-11-23 01:01:20 – 01:23:24 CST (wall ≈ 1,324 s), `grk-result: status=success turns=43
cost_usd=3.112638 refused=0`, the 20 GiB scope peaked at 2,552,946,688 B (2.38 GiB, oom 0/0),
rc 0; log `.temp/logs/grk_brief_762.log`; scene `.temp/brief-762/` (7,360 files,
3,507,293,343 B); `report.py check 762` OK with 82 citations. Delivered: the entry (761 copy
5,812 files `918e0c38…`; ten rows PASS; the 759 path absent; `INODE PASS compared=5853
missing_source=0 shared_inodes=0 devices=[53]`); the certificate-table adoption (single byte
copy of `85d75662…`/91,646 B over `6b390887…`/91,646 B; header identical, 68/68 rows changed
in the fingerprint column only, zero other cells; `records.rs` `7364f150…` unchanged); the four
certificate replays — emit default and no-default rc 101 panicking at `production.rs:99:6` with
`production source anchor expired` after printing the C01/C02 rows with rejection count 0 (the
accepted baseline content), check default and no-default rc 101 at the same anchor (the
baseline is rc 0, `test result: ok. 1 passed; 0 failed`), which is the brief's STOP condition;
the proof build and the fixture were not spent; zero importable receipts; all 13 Q25 rows named
stops. My gate reproduced the scene-recipe digest of the 761 copy (`918e0c38…`, 5,812 files)
and of the 762 copy as left (`7dd616dc…`), re-hashed the ten rows, proved the two copies differ
only in the adopted inventory file, re-derived the column proof, read all four raw logs, and
established the mechanism: `promotion.rs:120-121` compares `promoted.tsv`'s source-anchor cell
against the resolved item — the inventory's anchors are the live-tree coordinates
(`options/core.rs:106:5-122:6`, `:224:5-238:6`) while the copy's `options/core.rs`
(`171a62fb…`, byte-identical to the brief 754 armed tree) has the items at 214 and 397; and the
accepted 754 baseline check ran with its generated promoted table in place, whose
`generated-promoted.tsv` (`fdae14fe…`, 18,162 B) is byte-identical to the merge of the era's
two one-sided emissions (columns 0-3 and 6-8 from the hash=true emission, column 5 from the
hash=false emission). Today's merge of the fresh pair is 18,162 B `4337950c…`, differing from
the inventory only in columns 3, 4 and 5 on 2/2 rows. The STOP is correct: my brief's check bar
needed the promoted adoption it did not name. Ruling 771
(`.temp/director/ruling-771.md`) accepts the STOP and the deviation (the checks reused the emit
binaries under the recorded shared target directory), keeps Q25 unwaived, and orders **brief
763**: re-enter from the 762 copy (`7dd616dc…`, 5,812 files), re-run the two emits (rc 101 at
`production.rs:99:6`, writing fresh one-sided tables), calibrate the merge rule against the
recorded 754 pair, merge the fresh pair (expected `4337950c…`/18,162 B), prove the column delta
(header identical, 2/2 rows, columns 3/4/5 only, zero other cells), adopt by single byte copy
over the copy's `promoted.tsv`, re-run the two checks as recorded and require rc 0 with the
C01/C02 rows and `test result: ok. 1 passed; 0 failed` in both modes, then one default release
proof-only build and one bounded single-threaded fixture with the full harness name; the 13
dispositions and zero importable receipts. Budget: 394.967 s charged of the 3,500 s carve
(261.958 s command spend plus the 133.009 s meter span), unspent 3,105.033 s, reserve
**24,701.52184483117 s**; brief 763 carves **2,500 s** (→ 22,201.52184483117 s), 0 workspace,
0 epoch. Next: brief 763's gate, then the generator/source gate, the deterministic render, the
re-freeze, the fresh epoch, the Stage C/C10 readmission and a further 744 restart, each gated;
745/746 held.
