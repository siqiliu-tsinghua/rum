# Brief 763 — G15-CM4 Q25 ownership proof (P2379), stage six: the promoted-table adoption, the two certificate checks, one proof-only build and one bounded fixture invocation with the full harness name

**Director-dispatched, one stage, one gate.** This continues brief 762's STOP under ruling
771: the certificate-table adoption and both emit commands reproduced the accepted baseline
exactly, and the two check commands stopped on the stale `promoted.tsv` source-anchor cells
because the accepted brief 754 checks ran with the generated promoted table in place — the
brief did not name that adoption. The director has authorized the mechanical merge of the two
fresh one-sided emissions (calibrated on the recorded 754 pair), its adoption over the copy's
`promoted.tsv`, the two check replays, one proof-only build and one bounded fixture invocation
with the full harness name. The work runs in a fresh copy of the 762 copy. The live tree stays
untouched, `.temp/brief-762/`, `.temp/brief-761/`, `.temp/brief-760/`, `.temp/brief-759/` and
`.temp/brief-758/` stay read-only, no receipt is importable, and Q25's thirteen rows stay
named stops until the director rules after this gate.

## Goal

When this brief is done, in `.temp/brief-763/` only: the two emit commands reproduce the
accepted baseline content (rc 101 at the source-anchor panic, the C01/C02 rows with rejection
count 0); the generated promoted table is merged from those two fresh one-sided emissions with
the recorded 754 merge rule proved exact, adopted over the copy's `promoted.tsv` with a column
proof that only the anchor and the two certificate cells change; the two check commands pass
(rc 0, the C01/C02 rows, `test result: ok. 1 passed; 0 failed`); and the physical proof runs on
the default P2379 row — a genuine source-backed commitment on the actual allocation, the
driver's delivery to that concrete `Arc<Self>`, one legitimate omitted append, the second real
append, `Err(Output)` on unqueued; the withheld variant with identical physical operations and
exactly `NoHit` while both paths still execute; two same-state siblings rejected at the
concrete receiver in both directions; the lifetime/custody negatives — with individual
dispositions for all thirteen Q25 rows. The proof produces **zero importable receipts** and no
admitted receipt.

## Context pointers

- Brief 762 (committed `cbf416f8`) and ruling 771 (`.temp/director/ruling-771.md`); its scene
  `.temp/brief-762/`: `report.json`, `ledger.json`, `checkpoint.json`, `evidence/`, the raw
  logs, the emitted one-sided tables under `copy/.temp/brief-546/certificates/`
  (`promoted-hash-true.tsv` `03e54a43fb94ad8abebb4733d2216a4078b6ea9f611d4d68870d16a89626f83d`,
  `promoted-hash-false.tsv` `561aac0500125ecef8bc8de0c1794162c8e790dbabf5ca91061d38fcb1555318`).
- **The 762 copy (the entry).** `.temp/brief-762/copy`, source tree (excluding child
  directories named `target` and `.temp`; the walk hashes `F\0<relative path>\0<file bytes>`
  in sorted walk order) 5,812 files,
  `7dd616dc1d6783756277e7f09c7920887065009b313184c5f5f2f94bf29d714c`. It carries,
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
    `0fc8d966ce31f2a9253b1a79f56c73ebbd0bf69a232f3cbe7efedc997c8f5e98`
  - `crates/rum-stdlib/tests/suspension_oracle/mutations/records.rs` — 14,894 B,
    `7364f150437a0d395caa6843dabbb99a0c2500443faaa6ad8951f6af2eb1aa28`
  - `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` — 91,646 B,
    `85d75662c581acb62bd8a4fe44d61b25a7ec80e44abf490dac45f4b893da2434` (the adopted table)
  - and the pre-adoption inventory `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` —
    18,162 B, `03ef55aa77ae41369a9d6ea7b8364802777bda74f4508deec7fb570a437f0883`
  - the review table `target/brief-544/local-contracts.tsv` — 91,646 B, `85d75662…`.
- **The merge rule and its calibration.** At brief 754 the accepted check ran with the
  generated promoted table in place; that file
  (`.temp/brief-754/attempt-2/evidence/standing/armed/generated-promoted.tsv`,
  `fdae14fec96319926f7c69af0446e8a62cb9e370ee7f2a942742d431a4859487`, 18,162 B) is
  byte-identical to the merge of the era's two one-sided emissions
  (`.temp/brief-754/attempt-2/work/armed/.temp/brief-546/certificates/promoted-hash-true.tsv`
  and `.../promoted-hash-false.tsv`): header and columns 0-3 and 6-8 from the hash=true
  emission, column 5 (the hash=false certificates) from the hash=false emission. Applying the
  same rule to brief 762's fresh pair yields 18,162 B
  `4337950c5b5e5f2a26f94d4aee66aacc0263a62b732d34b74e3021d1182dbbc7`; against the inventory
  preimage only columns 3 (source anchor), 4 (hash=true certificates) and 5 (hash=false
  certificates) differ, on 2/2 rows, with zero other cells. The anchor staleness is the
  144-file render's coordinate shift: the copy's `options/core.rs` resolves
  `merged_option_exprs_for_symbol` at `:214:5-230:6` and `evaluate_runtime_option_expr` at
  `:396:5-410:6`, while the inventory still carries the live-tree coordinates
  `:106:5-122:6` and `:224:5-238:6`.
- **The accepted baseline content.** Emit (default and no-default): rc 101, panic at
  `crates/rum-stdlib/src/tests/retention_closure/local/production.rs:99:6` with
  `production source anchor expired`, after the C01 Unarmed/AccountingShadow rows and the C02
  `evaluate_runtime_option_expr` row, each with rejection count 0. Check (default and
  no-default): rc 0, the same C01/C02 rows with rejection count 0, and
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;` with `1163` (default) / `1157`
  (no-default) filtered out. Raw baselines:
  `.temp/brief-754/attempt-2/evidence/standing/armed/certificate-{emit,check}-{default,no-default}.log`.
- **The fixture.** The existing default P2379 selection (program `e5e3a1ff…rum`, setup
  `ae59ceca…rum`, pins `mutant-coverage.tsv:138`, `programs.tsv:957`,
  `.temp/brief-749/attempt-7/renders/one/mutants.tsv:138`; answers prior and `"X"`; trace
  preserved). No new rum program. The harness name is
  `q25_p2379_proof::q25_p2379_ownership_proof` (`tests/suspension_oracle.rs:52–53`).
- **The links.** The same `path:line` set as briefs 759/760/761/762 (raw sink
  `core.rs:617–636`; raw driver `core/driver.rs:37–95`; `suspension/typed/entry.rs:9–19`;
  `lib.rs:1566–1608`, `:1636–1648`, `:2101–2113`, `:2223–2255`; the eight concrete receivers;
  `records.rs:341–366`).
- **The bars.** The 36 historical signatures, the 20 Stage C cells, C10, the pair
  `(45,5,false)` (200 cells), the four structural controls, the eight observer rejection
  families, the 97-row table. This proof adds no admission.

## Scope

- MAY create/modify: everything under `.temp/brief-763/` only — the working copy (including
  the adopted table and the generated target), its build target, logs and receipts, the
  report at `.temp/brief-763/report.json` and its ledger at `.temp/brief-763/ledger.json`.
- MAY read: the live source tree, the accepted epoch trees and binaries, `.temp/brief-762/`,
  `.temp/brief-761/`, `.temp/brief-760/`, `.temp/brief-759/` and `.temp/brief-758/`
  (read-only), every earlier `.temp/brief-*` record.
- MUST: at entry, copy `.temp/brief-762/copy` once to `.temp/brief-763/copy` by copying
  inputs or copy-on-write, **never writable hardlinks to the accepted files**; record the
  copy identity of the source tree (5,812 files excluding child directories named `target`
  and `.temp`, `7dd616dc…`), the eleven rows above, and device/inode independence.
- **Authorized invocations (exact, in this order).**
  (a) The two emit replays, exactly as recorded:
  `cargo test --locked --offline -j8 -p rum-stdlib --lib local_production_acceptance --
  --nocapture` (default and `--no-default-features`), each through `scripts/run-bounded.py`
  with a timeout derived from the measured samples (`max(300, 8×96.508179) = 772` and
  `max(300, 8×80.405184) = 643`). They must reproduce the accepted emit content — rc 101,
  the panic at `production.rs:99:6`, the C01/C02 rows with rejection count 0 — and write the
  fresh one-sided tables (`promoted-hash-true.tsv`, `promoted-hash-false.tsv`) under the
  working copy's `.temp/brief-546/certificates/`; record their bytes and hashes.
  (b) The mechanical merge and adoption, by script, never by hand: first prove the rule on
  the recorded 754 pair (the merge must reproduce `fdae14fe…` byte-for-byte, read-only), then
  merge the fresh pair (header and columns 0-3 and 6-8 from the true emission, column 5 from
  the false emission; expected 18,162 B `4337950c…`; any difference is a STOP), then prove
  the column delta against the copy's inventory preimage (identical header, 2/2 rows, columns
  3, 4 and 5 the only changed cells, zero other cells), then write the merged table over the
  copy's `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` as a single
  byte copy with byte pre/postimages (the write is not a hardlink), `records.rs` byte-identical,
  and no other source edit.
  (c) The two check replays, exactly as recorded (the same two cargo argv as (a)), each
  through `scripts/run-bounded.py` with a timeout derived from the measured samples
  (`max(300, 8×41.297796) = 330`, `max(300, 8×40.146957) = 321`). They must pass: rc 0, the
  C01/C02 rows with rejection count 0, and `test result: ok. 1 passed; 0 failed; 0 ignored;
  0 measured;` with `1163` / `1157` filtered out. Any difference is a STOP with the raw
  diagnostics and no further spend.
  (d) Exactly one default proof-only build of the `suspension_oracle` target in the working
  copy (`--offline --locked -j 8 --release --message-format=json --no-run`, the epoch's
  absolute `cargo`/`rustc`, the working copy's manifest, the locked registry resolvable
  offline; target dir and `TMPDIR` under `.temp/brief-763/`).
  (e) Exactly one bounded single-threaded fixture invocation with the full harness name
  `q25_p2379_proof::q25_p2379_ownership_proof` (`--exact`, `--nocapture`, `--release`, the
  proof-built binary, `RUST_TEST_THREADS=1`, `RUM_CM740_OMIT=P2379`) through
  `scripts/run-bounded.py`; a `--list` preflight of the built binary (which runs no test) is
  allowed once to confirm the name before the invocation.
  The other nine replay commands from brief 761 are NOT rerun and their 761 verdicts stand.
  No other cargo build or test execution may run; anything else is a gate for the director,
  not an automatic spend. The emit runs must happen before the adoption and the check runs
  after it: that order is what reproduces the recorded content.
- MUST NOT: edit any live file; write under `.temp/brief-762/`, `.temp/brief-761/`,
  `.temp/brief-760/`, `.temp/brief-759/`, `.temp/brief-758/` or any sealed scene; compile the
  live tree; run git; touch the network; add a dependency or `unsafe`; add a program, a
  selection, a lane, a policy or an epoch label; produce an importable or admitted receipt.
  The membership stays 28/3/7/113 and the pair `(45,5,false)` stays Open at zero admitted
  receipts.

## Deliverable

1. **Entry and the working copy.** Re-authenticate the 762 copy (source tree `7dd616dc…`,
   5,812 files, the eleven rows, the 759 path absent, the review table present); create the
   working copy and record its identity and inode independence; record every subsequent edit
   with byte pre/postimages.
2. **The two emit replays.** Run exactly the two recorded emit commands and quote the actual
   verdict lines: rc 101, the panic at `production.rs:99:6` with `production source anchor
   expired`, the C01/C02 rows with rejection count 0; record the fresh one-sided tables'
   bytes and hashes. Any difference is a STOP with the raw diagnostics.
3. **The merge, the column proof and the adoption.** Prove the merge rule on the recorded 754
   pair (byte-identical to `fdae14fe…`); merge the fresh pair (18,162 B `4337950c…`); parse
   both the preimage and the merged table and show the header is identical, all 2 rows are
   present, only columns 3, 4 and 5 differ on both rows, and no other cell differs; adopt by
   a single byte copy; record the preimage and postimage with sizes and hashes; `records.rs`
   stays byte-identical to `7364f150…`; no other source edit.
4. **The two check replays.** Run exactly the two recorded check commands and quote the actual
   verdict lines: rc 0, the C01/C02 rows with rejection count 0, and
   `test result: ok. 1 passed; 0 failed` in both modes, matching the accepted baseline. Any
   difference is a STOP; the proof build and the fixture are not spent.
5. **The one bounded execution.** Run the trial groups of brief 759 deliverable 4 unchanged —
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
6. **The 13 dispositions and the report.** One disposition for each of the 13 Q25 rows: which
   actual anchors executed in this proof and which are supported only by the generic
   implementation plus static instantiation. A green design or one concrete execution does not
   prove physical reach or an own catch for the other rows. Fail the proof for a missing true
   receiver, false acceptance on either sibling, a different withheld rejection, a missing
   original operation, incorrect generation/custody, leaked authority, a changed guest outcome
   outside the one effect, unexplained inventory drift or a projected overrun. A failed common
   proof keeps all thirteen named stops for ruling 771. A green proof permits the director to
   decide whether to release the independent generator/source stage, still owing
   positives/withholding on each applicable selected path, both modes, and the companion
   queued operations. Never close a row, spend a further build, introduce a new observer or
   cut a negative to fit the carve.

## Hard rules

- `.temp/brief-762/`, `.temp/brief-761/`, `.temp/brief-760/`, `.temp/brief-759/`,
  `.temp/brief-758/`, the accepted epoch and every sealed scene stay read-only; the live tree
  is untouched; the working copy is disposable and non-admission. No live edit, no build of
  the live tree, no git, no network, no new dependency, no `unsafe`, no new
  program/lane/selection/policy/epoch label; `.temp` only; never `/tmp`; never read
  `~/.config/ccds/api-key`.
- The authorized invocations are exactly the two emits, the merge and adoption, the two
  checks, one proof-only build and one fixture execution (plus the one `--list` preflight).
  Reprice cold cache and negative work at entry and at each irreversible spend. A projected
  overrun stops for a director ruling, never a scope cut.
- The replay bar is the verdict-line content against the accepted baseline; neither rc 0 nor a
  green suite is invented where the baseline records a failure. Never weaken a predicate,
  reclassify a row, exempt a stop, count a call edge as an authorization, count a text
  identity as a catch, or infer a runtime fact from the report. The adoption is
  generator-produced and mechanical: no hand edit of any inventory, and no change to any
  column outside 3, 4 and 5.
- Quote every verdict line; a claim without `path:line` or a raw log line is not a verdict.
- The four structural controls, the 36 historical signatures, the 20 Stage C cells, C10, the
  pair `(45,5,false)`, the eight observer rejection families and the 97-row table are retained
  unchanged; this proof adds no admission and produces no importable receipt.
- The standing state is unchanged: bar 571/4,422, 37/134, 4/133, 0/26; the pair `(45,5,false)`
  Open at zero admitted receipts; C2 118 Residual + 0 Lost; the 51 owners and five source
  duties Open; membership 28/3/7/113; `CHANGELOG.md` and S10 blocked; CM5 unauthorized;
  everything sealed (c76–c99) sealed; 745/746 held; no 744 restart.

## Verification (MANDATORY — you run it; iterate to green or to evidence)

- The entry reproduces the 762 copy source-tree identity (`7dd616dc…`, 5,812 files) and the
  eleven rows; the working-copy identity is recorded; every edit is itemized with byte
  pre/postimages; `records.rs` stays byte-identical to `7364f150…`.
- The two emit replays reproduce the accepted emit content exactly (rc 101 at
  `production.rs:99`), and the two one-sided tables are freshly written and hashed.
- The merge rule reproduces the recorded 754 `generated-promoted.tsv` byte-for-byte; the
  fresh merge is 18,162 B `4337950c…`; the column proof shows only columns 3, 4 and 5 differ
  on 2/2 rows, zero other cells; the adoption is a single byte copy with byte pre/postimages.
- The two check replays reproduce the accepted baseline content exactly (rc 0, the C01/C02
  rows, `test result: ok. 1 passed; 0 failed` in both modes), every verdict line quoted with
  its raw log citation.
- The ordinary, positive, withheld, sibling and negative predictions are exact, and every one
  is either observed or the proof fails with its raw receipt. The `NoHit` is the exact
  `records.rs:341–350` rejection; the three-image importer of `records.rs:283–305` is
  explicitly deferred and never claimed as passed.
- The 13 dispositions are complete; zero importable receipts; the class-wide bars are
  unchanged; the report is `rum-report/1` at `.temp/brief-763/report.json` with
  `.temp/brief-763/ledger.json`; `python3 scripts/report.py check 763` is green with every
  PROBLEM fixed; the final message is only `python3 scripts/report.py summary 763`.
- Budget: **2,500 s focused** (a carve from the corrected reserve, 24,701.52184483117 →
  22,201.52184483117 s), **0 s workspace, 0 s epoch**; unspent returns; a projected overrun
  stops for a director ruling, never a scope cut. The carve prices the entry and working
  copy, the two emit replays, the merge and adoption, the two check replays, the release
  proof build, the one fixture invocation (the 758 forecast priced the fixture image at
  347.34 s) and the close.

## Report format

`rum-report/1` at `.temp/brief-763/report.json` (cumulative with `.temp/brief-763/ledger.json`),
`changes` empty (every write is under `.temp/brief-763/`); sections for entry and the working
copy, the two emit replays, the merge and adoption, the two check replays, the fixture
execution and raw observations, the predictions versus actual outcomes, the 13 dispositions,
and the scope; cite lines exactly. STOP at the proof gate: no live edit, no admission, no
generator/source stage, no 744 restart, no 745/746, no CM5.

## Gate c122 (2026-11-23) — STOP accepted; Q25 unwaived; brief 764 ordered

Brief 763 ran on `grk` (grok-4.7, effort xhigh), unit
`brief-763-grk-20261123-013114-864163`, session `83dcaac3-2ecc-46ca-a50c-913cf76d1045`,
2026-11-23 01:31:14 – 01:54:59 CST (wall ≈ 1,425 s), `grk-result: status=success turns=40
cost_usd=2.850884 refused=0`, the 20 GiB scope peaked at 2,584,285,184 B (2.41 GiB, oom 0/0),
rc 0; log `.temp/logs/grk_brief_763.log`; scene `.temp/brief-763/` (7,752 files,
3,802,625,178 B); `report.py check 763` OK with 96 citations; 11 PASS / 1 FAIL checks.
Delivered and verified: the entry (762 copy 5,812 files `7dd616dc…`; eleven rows PASS; 0
shared inodes; the 759 path absent); the two emit replays rc 101 at `production.rs:99:6` with
`production source anchor expired` after the C01/C02 rows with rejection count 0, writing the
fresh one-sided tables `03e54a43…`/`561aac05…`; the merge calibrated against the recorded 754
pair (byte-identical to `fdae14fe…`), the fresh merge 18,162 B `4337950c…`, adopted over the
copy's `promoted.tsv` with the header identical and only columns 3/4/5 differing on 2/2 rows,
`records.rs` `7364f150…` unchanged (`SOURCE DELTA PASS files_changed=1`); the two check
replays rc 0 with the C01/C02 rows and `test result: ok. 1 passed; 0 failed` (1163/1157
filtered out); the release proof build rc 0 in 65.424 s (binary
`suspension_oracle-547fa4362abc58a1`, 113,619,264 B, `50b3e842…`); the one `--list` preflight
printing the full harness name; and the one fixture invocation rc 101 in 0.083 s, panicking
at `q25_p2379_proof.rs:102:17` with `Q25_AST resume receiver is not an explicit typed self`
before any trial — no `Q25_PROGRAM`, no observation, no `NoHit`; the importer deferred; all
13 rows named stops; zero importable receipts; no second invocation and no proof edit. My
gate reproduced the post-adoption copy identity (`0d677622…`, 5,812 files), showed the copy
differs from the 762 copy only in `promoted.tsv`, re-derived the merge and its 754
calibration byte-for-byte, re-derived the column proof (`{3:2, 4:2, 5:2}`, zero other cells),
read all raw logs, and proved the mechanism from syn 2.0.117's own parser
(`src/item.rs:1604–1721`): `self: std::sync::Arc<Self>` is stored as `FnArg::Receiver` with
`colon_token: Some`, so the checker's `FnArg::Typed` arm can never match the intended
signature (`resume_frames.rs:301`, `begin_concrete` at `:305`); the predicate is defective,
the signature and trial design are correct. Scope verified (0 live newer, 0 in scenes
758–762, receipts empty, `changes` empty). The deviation (the check commands recompiled
`rum-stdlib` because `promoted.tsv` is `include_str!` in `promotion.rs`, under the recorded
argv, timeouts and shared target) is accepted. Budget: focused 796.170587467 s of 2,500;
unspent 1,703.829412533 s; reserve **23,905.35125736417 s**. Ruling 772
(`.temp/director/ruling-772.md`) accepts the STOP, answers `receiver-arm` — the
`FnArg::Receiver` arm with an explicit colon and the `Arc<Self>` type is the correct syn
representation and the classifier must accept it while still rejecting bare, `&`, `&mut` and
wrong-typed receivers, with a shipped falsifier — and orders **brief 764**: re-enter from the
763 copy (`0d677622…`, 5,812 files), repair only the receiver classifier in
`q25_p2379_proof.rs` plus its falsifier, at most three bounded repair compilations, no
certificate rerun (their 763 verdicts stand), one default release proof-only build, one
bounded single-threaded fixture with the full harness name, the 13 dispositions and zero
importable receipts; the predictions, the source-hit clause, `records.rs`, the owner/custody
semantics and every other predicate immutable; any other cargo invocation, any other code
edit, or a second fixture run is a new gate. Brief 764 carves **1,800 s** (23,905.35125736417
→ 22,105.35125736417 s), 0 workspace, 0 epoch. Next: brief 764's gate, then the
generator/source gate, the deterministic render, the re-freeze, the fresh epoch, the Stage
C/C10 readmission and a further 744 restart, each gated; 745/746 held.
