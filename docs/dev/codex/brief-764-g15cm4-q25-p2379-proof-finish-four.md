# Brief 764 — G15-CM4 Q25 ownership proof (P2379), stage seven: the receiver-arm classifier repair with its falsifier, one proof-only build and one bounded fixture invocation with the full harness name

**Director-dispatched, one stage, one gate.** This continues brief 763's STOP under ruling
772: the promoted-table adoption is exact, both certificate checks reproduce the accepted
baseline green, the release proof binary built, and the one authorized fixture invocation
stopped in the pre-trial static classifier — `q25_p2379_proof.rs:101` requires
`syn::FnArg::Typed`, but syn 2.0.117 stores the intended `self: std::sync::Arc<Self>`
(`resume_frames.rs:301`) as `FnArg::Receiver` with `colon_token: Some` (verified in syn's own
`item.rs:1604–1721`). The director has authorized the classifier repair and its falsifier,
one proof-only build and one new bounded fixture invocation. The work runs in a fresh copy of
the 763 copy. The live tree stays untouched, `.temp/brief-763/`, `.temp/brief-762/`,
`.temp/brief-761/`, `.temp/brief-760/`, `.temp/brief-759/` and `.temp/brief-758/` stay
read-only, no receipt is importable, and Q25's thirteen rows stay named stops until the
director rules after this gate.

## Goal

When this brief is done, in `.temp/brief-764/` only: the receiver classifier corrected to
accept the explicit typed `FnArg::Receiver` arm (colon present, no reference, a type carrying
`Arc` and `Self`) and to reject every other receiver shape with a precise message, with a
falsifier that parses a synthetic `&self` and a wrong-type receiver and shows both rejected
while the real shape is accepted (one verdict line), then the physical proof on the default
P2379 row — a genuine source-backed commitment on the actual allocation, the driver's
delivery to that concrete `Arc<Self>`, one legitimate omitted append, the second real append,
`Err(Output)` on unqueued; the withheld variant with identical physical operations and exactly
`NoHit` while both paths still execute; two same-state siblings rejected at the concrete
receiver in both directions; the lifetime/custody negatives — with individual dispositions for
all thirteen Q25 rows. The proof produces **zero importable receipts** and no admitted
receipt.

## Context pointers

- Brief 763 (committed `649de4db`) and ruling 772 (`.temp/director/ruling-772.md`); its scene
  `.temp/brief-763/`: `report.json`, `ledger.json`, `checkpoint.json`, `evidence/`, the raw
  logs, the adopted `promoted.tsv` and the proof-built binary
  `.temp/brief-763/build/release/deps/suspension_oracle-547fa4362abc58a1`
  (`50b3e842e13b584c3e6dba75caa3f2805d6c9cc11a1ec7b6ebf10ade68b56e11`).
- **The 763 copy (the entry).** `.temp/brief-763/copy`, source tree (excluding child
  directories named `target` and `.temp`; the walk hashes `F\0<relative path>\0<file bytes>`
  in sorted walk order) 5,812 files,
  `0d67762263e503e24145014257be81e8775ad24dd17cc6b9a0352916224ec9cf`. It carries,
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
    `8e996d074f5a6dc7b4073ec1bae633825e2db4775be769b0dc0141c482eea370` (the resume receiver
    `self: std::sync::Arc<Self>` at `:301`, `begin_concrete` at `:305` — immutable)
  - `crates/rum-stdlib/tests/suspension_oracle.rs` — 1,761 B,
    `7dd5b6846ad6a14a34ba93f4b2ea88a0ba50065cedd6097922048fcd6914495f`
  - `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs` — 32,082 B,
    `0fc8d966ce31f2a9253b1a79f56c73ebbd0bf69a232f3cbe7efedc997c8f5e98` (the file to repair)
  - `crates/rum-stdlib/tests/suspension_oracle/mutations/records.rs` — 14,894 B,
    `7364f150437a0d395caa6843dabbb99a0c2500443faaa6ad8951f6af2eb1aa28`
  - `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` — 91,646 B,
    `85d75662c581acb62bd8a4fe44d61b25a7ec80e44abf490dac45f4b893da2434` (adopted)
  - `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` — 18,162 B,
    `4337950c5b5e5f2a26f94d4aee66aacc0263a62b732d34b74e3021d1182dbbc7` (adopted)
  - the review table `target/brief-544/local-contracts.tsv` — 91,646 B, `85d75662…`.
- **The defect and the correct predicate.** syn 2.0.117 parses `self: std::sync::Arc<Self>`
  as `FnArg::Receiver { reference: None, colon_token: Some(:), ty: Arc<Self> }`
  (`syn-2.0.117/src/item.rs:1604–1721`); `FnArg::Typed` can never match it. The classifier
  must accept exactly that shape — receiver, no reference, an explicit colon, and a type whose
  tokens contain `Arc` and `Self` — and reject bare `self`, `&self`, `&mut self`, and any
  other receiver type with its own message. The first-statement check (a `Local` whose
  initializer's call ends with `begin_concrete`) and the impl/trait scan stay unchanged.
- **The falsifier.** The classifier must ship the proof that it can fail: parse synthetic
  method signatures (e.g., a small trait-impl snippet containing `fn resume(&self, ...)` and
  one with a wrong-typed receiver such as `self: std::sync::Arc<Other>`) and show the
  classifier rejects both while accepting the exact `self: std::sync::Arc<Self>` shape; emit
  one `Q25_AST_FALSIFIER` line carrying all three verdicts. The falsifier runs in the same
  fixture invocation, before the trial groups, and must not change any prediction.
- **The evidence that already stands (do NOT rerun).** The 763 certificates: emits rc 101 at
  `production.rs:99:6` with the C01/C02 rows at rejection count 0; checks rc 0 with the
  C01/C02 rows and `test result: ok. 1 passed; 0 failed` in both modes. The test-file repair
  is outside the contracts' dependency list and cannot move them.
- **The fixture.** The existing default P2379 selection (program `e5e3a1ff…rum`, setup
  `ae59ceca…rum`, pins `mutant-coverage.tsv:138`, `programs.tsv:957`,
  `.temp/brief-749/attempt-7/renders/one/mutants.tsv:138`; answers prior and `"X"`; trace
  preserved). No new rum program. The harness name is
  `q25_p2379_proof::q25_p2379_ownership_proof` (`tests/suspension_oracle.rs:52–53`).
- **The links.** The same `path:line` set as briefs 759/760/761/762/763 (raw sink
  `core.rs:617–636`; raw driver `core/driver.rs:37–95`; `suspension/typed/entry.rs:9–19`;
  `lib.rs:1566–1608`, `:1636–1648`, `:2101–2113`, `:2223–2255`; the eight concrete receivers;
  `records.rs:341–366`).
- **The bars.** The 36 historical signatures, the 20 Stage C cells, C10, the pair
  `(45,5,false)` (200 cells), the four structural controls, the eight observer rejection
  families, the 97-row table. This proof adds no admission.

## Scope

- MAY create/modify: everything under `.temp/brief-764/` only — the working copy (including
  the repaired proof file), its build target, logs and receipts, the report at
  `.temp/brief-764/report.json` and its ledger at `.temp/brief-764/ledger.json`.
- MAY read: the live source tree, the accepted epoch trees and binaries, `.temp/brief-763/`,
  `.temp/brief-762/`, `.temp/brief-761/`, `.temp/brief-760/`, `.temp/brief-759/`,
  `.temp/brief-758/` (read-only), every earlier `.temp/brief-*` record.
- MUST: at entry, copy `.temp/brief-763/copy` once to `.temp/brief-764/copy` by copying
  inputs or copy-on-write, **never writable hardlinks to the accepted files**; record the
  copy identity of the source tree (5,812 files excluding child directories named `target`
  and `.temp`, `0d677622…`), the eleven rows above, and device/inode independence.
- **Authorized invocations (exact).** (a) At most **three bounded repair compilations**
  (`cargo test --locked --offline -j8 -p rum-stdlib --test suspension_oracle --no-run`, the
  epoch's absolute cargo/rustc, the copy's manifest, target and `TMPDIR` under
  `.temp/brief-764/`), each receipted, every edit with byte pre/postimages; edits are allowed
  only in `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs`. If the file still
  does not compile after three, STOP with the raw diagnostics. (b) Exactly one default
  release proof-only build of the `suspension_oracle` target in the working copy
  (`--offline --locked -j 8 --release --message-format=json --no-run`, a receipt keyed by the
  real bytes). (c) Exactly one bounded single-threaded fixture invocation with the full
  harness name `q25_p2379_proof::q25_p2379_ownership_proof` (`--exact`, `--nocapture`,
  `--release`, the proof-built binary, `RUST_TEST_THREADS=1`, `RUM_CM740_OMIT=P2379`)
  through `scripts/run-bounded.py`; one `--list` preflight is allowed. The four certificate
  commands from brief 763 are NOT rerun and their verdicts stand. No other cargo build or
  test execution may run; anything else is a gate for the director, not an automatic spend.
- MUST NOT: edit any live file; write under `.temp/brief-763/`, `.temp/brief-762/`,
  `.temp/brief-761/`, `.temp/brief-760/`, `.temp/brief-759/`, `.temp/brief-758/` or any
  sealed scene; compile the live tree; run git; touch the network; add a dependency or
  `unsafe`; add a program, a selection, a lane, a policy or an epoch label; produce an
  importable or admitted receipt. The membership stays 28/3/7/113 and the pair
  `(45,5,false)` stays Open at zero admitted receipts.

## Deliverable

1. **Entry and the working copy.** Re-authenticate the 763 copy (source tree `0d677622…`,
   5,812 files, the eleven rows, the 759 path absent); create the working copy and record its
   identity and inode independence; record every subsequent edit with byte pre/postimages.
2. **The classifier repair and its falsifier.** Repair only the receiver classifier: accept
   `FnArg::Receiver` with `reference: None`, `colon_token: Some`, and a type carrying `Arc`
   and `Self`; reject bare, `&`, `&mut` and wrong-typed receivers with precise messages; keep
   the impl/trait scan and the first-statement `begin_concrete` check unchanged. Add the
   falsifier: synthetic `&self` and wrong-type receivers parsed and rejected, the real shape
   accepted, one `Q25_AST_FALSIFIER` verdict line. Quote every removal/insertion with byte
   pre/postimages.
3. **The proof build.** Exactly one default release proof-only build as authorized; record
   the executable's path, size and sha256 and the `--list` line.
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
   proof keeps all thirteen named stops for ruling 772. A green proof permits the director to
   decide whether to release the independent generator/source stage, still owing
   positives/withholding on each applicable selected path, both modes, and the companion
   queued operations. Never close a row, spend a further build, introduce a new observer or
   cut a negative to fit the carve.

## Hard rules

- `.temp/brief-763/`, `.temp/brief-762/`, `.temp/brief-761/`, `.temp/brief-760/`,
  `.temp/brief-759/`, `.temp/brief-758/`, the accepted epoch and every sealed scene stay
  read-only; the live tree is untouched; the working copy is disposable and non-admission. No
  live edit, no build of the live tree, no git, no network, no new dependency, no `unsafe`, no
  new program/lane/selection/policy/epoch label; `.temp` only; never `/tmp`; never read
  `~/.config/ccds/api-key`.
- The authorized invocations are exactly at most three bounded repair compilations, one
  proof-only build and one fixture execution (plus the one `--list` preflight). Reprice cold
  cache and negative work at entry and at each irreversible spend. A projected overrun stops
  for a director ruling, never a scope cut.
- The repair bar is correctness, not accommodation: the signature
  `self: std::sync::Arc<Self>` is the design and must not be changed; the classifier must be
  at least as strict as the intended predicate and must ship the falsifier. Never weaken a
  predicate, reclassify a row, exempt a stop, count a call edge as an authorization, count a
  text identity as a catch, or infer a runtime fact from the report. No hand edit of any
  inventory.
- Quote every verdict line; a claim without `path:line` or a raw log line is not a verdict.
- The four structural controls, the 36 historical signatures, the 20 Stage C cells, C10, the
  pair `(45,5,false)`, the eight observer rejection families and the 97-row table are retained
  unchanged; this proof adds no admission and produces no importable receipt.
- The standing state is unchanged: bar 571/4,422, 37/134, 4/133, 0/26; the pair `(45,5,false)`
  Open at zero admitted receipts; C2 118 Residual + 0 Lost; the 51 owners and five source
  duties Open; membership 28/3/7/113; `CHANGELOG.md` and S10 blocked; CM5 unauthorized;
  everything sealed (c76–c99) sealed; 745/746 held; no 744 restart.

## Verification (MANDATORY — you run it; iterate to green or to evidence)

- The entry reproduces the 763 copy source-tree identity (`0d677622…`, 5,812 files) and the
  eleven rows; the working-copy identity is recorded; every edit is itemized with byte
  pre/postimages; only `q25_p2379_proof.rs` changes; `records.rs` stays byte-identical to
  `7364f150…`.
- The classifier accepts the real explicit typed receiver and rejects the synthetic `&self`
  and wrong-type receivers, with the falsifier's `Q25_AST_FALSIFIER` line quoted; the
  impl/trait scan and the `begin_concrete` first-statement check are unchanged.
- The one proof build is rc 0 and its binary is hashed; the one fixture invocation runs the
  full harness name and every prediction is either observed exactly or the proof fails with
  its raw receipt. The `NoHit` is the exact `records.rs:341–350` rejection; the three-image
  importer of `records.rs:283–305` is explicitly deferred and never claimed as passed.
- The 13 dispositions are complete; zero importable receipts; the class-wide bars are
  unchanged; the report is `rum-report/1` at `.temp/brief-764/report.json` with
  `.temp/brief-764/ledger.json`; `python3 scripts/report.py check 764` is green with every
  PROBLEM fixed; the final message is only `python3 scripts/report.py summary 764`.
- Budget: **1,800 s focused** (a carve from the corrected reserve, 23,905.35125736417 →
  22,105.35125736417 s), **0 s workspace, 0 s epoch**; unspent returns; a projected overrun
  stops for a director ruling, never a scope cut. The carve prices the entry and working
  copy, the repair and at most three bounded compilations, the release proof build, the one
  fixture invocation (the 758 forecast priced the fixture image at 347.34 s) and the close.

## Report format

`rum-report/1` at `.temp/brief-764/report.json` (cumulative with `.temp/brief-764/ledger.json`),
`changes` empty (every write is under `.temp/brief-764/`); sections for entry and the working
copy, the classifier repair and falsifier, the proof build, the fixture execution and raw
observations, the predictions versus actual outcomes, the 13 dispositions, and the scope;
cite lines exactly. STOP at the proof gate: no live edit, no admission, no generator/source
stage, no 744 restart, no 745/746, no CM5.
