# Brief 761 — G15-CM4 Q25 ownership proof (P2379), stage four: the two proof-file repairs, then the authorized replays, one proof-only build and one bounded fixture invocation

**Director-dispatched, one stage, one gate.** This continues brief 760's STOP under ruling
769: the q25 module placement and both `Arc` coercions are repaired and the contract compiles;
the only remaining failures are two mechanical errors in the added proof test
(`q25_p2379_proof.rs:418–419` and `:495`), and the three-compilation bound was consumed.
The director has authorized those two proof-file repairs, up to three more bounded
compilations, and the still-unlaunched authorized sequence. The work runs in a fresh copy of
the repaired 760 copy. The live tree stays untouched, `.temp/brief-760/`, `.temp/brief-759/`
and `.temp/brief-758/` stay read-only, no receipt is importable, and Q25's thirteen rows stay
named stops until the director rules after this gate.

## Goal

When this brief is done, in `.temp/brief-761/` only: the proof test compiling with the two
mechanical repairs, and a physical proof on the default P2379 row — a genuine source-backed
commitment on the actual allocation, the driver's delivery to that concrete `Arc<Self>`, one
legitimate omitted append, the second real append, and `Err(Output)` on unqueued; the withheld
variant with identical physical operations and exactly `NoHit` while both paths still execute;
two same-state siblings rejected at the concrete receiver in both directions; the
lifetime/custody negatives — after the prescribed inventory generators and validators have run
on the covered sources in **both** feature modes, and with individual dispositions for all
thirteen Q25 rows. The proof produces **zero importable receipts** and no admitted receipt.

## Context pointers

- Brief 760 (committed `68b7a7de`) and ruling 769 (`.temp/director/ruling-769.md`); its scene
  `.temp/brief-760/`: `report.json`, `ledger.json`, `checkpoint.json`,
  `evidence/entry-identity.json`, `evidence/edits.json`, `evidence/compile-receipts.json`,
  `evidence/budget.json`, `logs/entry-copy.log`, `logs/repair-compile-1.log`,
  `logs/repair-compile-2.log`, `logs/repair-compile-3.log`, `logs/repair-wrapper-miss.log`,
  `logs/close.log`.
- **The repaired 760 copy (the entry).** `.temp/brief-760/copy`, 5,812 files, tree
  `abb03ca582c77860b9469625bc057d845e5b0d6be196cc0d530944c859349c85` (the scene's own
  `F\0<path>\0<bytes>` walk). It carries, byte-for-byte:
  - `crates/rum-runtime/src/lib.rs` — 258,973 B, `15d91070bbad767e098d5d615c110c8597198d57c50d6e4d96686664540691fc`
  - `crates/rum-runtime/src/cm740_probe/q25_ownership.rs` — 29,522 B,
    `43416be62901a79c797f057b81a7364b668a91099188098250dbeaaaa68f2052` (the 759 file, moved;
    `crates/rum-runtime/src/q25_ownership.rs` must be absent)
  - `crates/rum-runtime/src/runtime/suspension/core.rs` — 33,984 B,
    `7766b20ef78e7064c648f4e56d912daea4108686b6239627c0b1cdfd69b9dbf4` (760's final postimage)
  - `crates/rum-runtime/src/runtime/suspension/core/driver.rs` — 9,474 B,
    `7f800a7f459db7c9193b77ee8f9e8336c1d55ebe546dcfdee3d83412bb893f19`
  - `crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs` — 29,274 B,
    `2d915381edb0a12781c0eeb722bd6bef1d964573f8f3f19d9354709603b2f66e`
  - `crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs` — 20,948 B,
    `8e996d074f5a6dc7b4073ec1bae633825e2db4775be769b0dc0141c482eea370`
  - `crates/rum-stdlib/tests/suspension_oracle.rs` — 1,761 B,
    `7dd5b6846ad6a14a34ba93f4b2ea88a0ba50065cedd6097922048fcd6914495f`
  - `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs` — 31,944 B,
    `73d92bd0a41a1e1a131cc1b7dafc4ef7c8bd35433c776d8d1313dd44d5db8823`
  - `crates/rum-stdlib/tests/suspension_oracle/mutations/records.rs` — 14,894 B,
    `7364f150437a0d395caa6843dabbb99a0c2500443faaa6ad8951f6af2eb1aa28`
- **The two failures to repair** (brief 760's compile 3, `logs/repair-compile-3.log`):
  - `error[E0382]: borrow of moved value: message` — `q25_p2379_proof.rs:419:40`; the helper
    `expect` at `:416–421` does `failures.push(message)` and then
    `emit(&format!("Q25_EXPECT_FAIL {message}"))`. The emitted text and its order relative to
    the push must not change (emit first, or clone, or an equivalent that preserves both).
  - `error[E0133]: call to unsafe function set_var` — `q25_p2379_proof.rs:495:5`
    (`std::env::set_var("RUM_CM740_OMIT", "P2379")`), because the workspace edition is 2024.
    The hard rule forbids `unsafe`. `RUM_CM740_OMIT` is read by the thread-local `SELECTED` at
    `crates/rum-runtime/src/lib.rs:182` on first access, so the value must be supplied in the
    **process environment of the one fixture invocation**; the test line is removed and the
    test gains a fail-closed precondition check that the selector is present and equals
    `P2379` (a stronger check — it may not replace or weaken the env-mint negative assertion
    at `:507–513`, which must still prove that neither the selector environment nor the guest
    begin string mints a capability and that `guest_grant()` is false).
- **The accepted epoch (read-only).** `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  (`7af7b461…`), build `809b6473…`, freeze `c724c9ab…`; the armed tree `bd8b7548…`, real tree
  `97eaeb45…`, disabled tree `b41d1873…`; the twelve label binaries; the accepted build receipt
  `.temp/brief-754/attempt-2/epoch/default-armed-build.json`.
- **The fixture.** The existing default P2379 selection (program `e5e3a1ff…rum`, setup
  `ae59ceca…rum`, pins `mutant-coverage.tsv:138`, `programs.tsv:957`,
  `.temp/brief-749/attempt-7/renders/one/mutants.tsv:138`; answers prior and `"X"`; trace
  preserved). No new rum program.
- **The links.** The same `path:line` set as briefs 759/760 (raw sink `core.rs:617–636`; raw
  driver `core/driver.rs:37–95`; `suspension/typed/entry.rs:9–19`; `lib.rs:1566–1608`,
  `:1636–1648`, `:2101–2113`, `:2223–2255`; the eight concrete receivers;
  `records.rs:341–366`).
- **The accepted inventory baseline.** `.temp/brief-754/attempt-2/evidence/standing/armed/`:
  `generate-commands.json` (3), `validate-commands.json` (9), the twelve accepted verdict logs
  and their recorded rc values (several are recorded failures, rc 101 — that is the baseline
  to compare against, not a demand for naive rc 0).
- **The bars.** The 36 historical signatures, the 20 Stage C cells, C10, the pair
  `(45,5,false)` (200 cells), the four structural controls, the eight observer rejection
  families, the 97-row table. This proof adds no admission.

## Scope

- MAY create/modify: everything under `.temp/brief-761/` only — the working copy, its build
  target, logs and receipts, the report at `.temp/brief-761/report.json` and its ledger at
  `.temp/brief-761/ledger.json`.
- MAY read: the live source tree, the accepted epoch trees and binaries, `.temp/brief-760/`,
  `.temp/brief-759/` and `.temp/brief-758/` (read-only), every earlier `.temp/brief-*` record.
- MUST: at entry, copy `.temp/brief-760/copy` once to `.temp/brief-761/copy` by copying
  inputs or copy-on-write, **never writable hardlinks to the accepted files**, and record the
  copy identity (5,812 files, tree `abb03ca5…`, device/inode independence); verify the nine
  rows of the entry table above (including the moved path present and the 759 path absent)
  before the first edit.
- **Authorized invocations (exact).** (a) The two proof-file repairs, validated by at most
  **three** bounded cargo compilations of the working copy, each through
  `scripts/run-bounded.py` with a timeout derived from the measured samples and a receipt
  keyed by the real command, rc and bytes; (b) the three recorded generate commands, then
  (c) the nine recorded validate commands, each run as recorded (same `cargo test --locked
  --offline -j8` shape, default and no-default as the command states) through
  `scripts/run-bounded.py` — the both-mode compiles are authorized by ruling 768; (d) exactly
  one default proof-only build of the `suspension_oracle` target in the working copy
  (`--offline --locked -j 8 --release --message-format=json --no-run`, the epoch's absolute
  `cargo`/`rustc`, the working copy's manifest, the locked registry resolvable offline;
  target dir and `TMPDIR` under `.temp/brief-761/`); (e) exactly one bounded single-threaded
  fixture invocation (`RUST_TEST_THREADS=1`, the filter `q25_p2379_ownership_proof`,
  `--exact`, `--nocapture`, `--release`, the proof-built binary) through
  `scripts/run-bounded.py`, with `RUM_CM740_OMIT=P2379` in its process environment. No other
  cargo build or test execution may run; anything else is a gate for the director, not an
  automatic spend.
- MUST NOT: edit any live file; write under `.temp/brief-760/`, `.temp/brief-759/`,
  `.temp/brief-758/` or any sealed scene; compile the live tree; run git; touch the network;
  add a dependency or `unsafe`; add a program, a selection, a lane, a policy or an epoch label;
  produce an importable or admitted receipt. The membership stays 28/3/7/113 and the pair
  `(45,5,false)` stays Open at zero admitted receipts.

## Deliverable

1. **Entry and the working copy.** Re-authenticate the repaired 760 copy (tree `abb03ca5…`,
   5,812 files, the nine entry rows above, the 759 path absent); create the working copy and
   record its identity and inode independence; record every subsequent edit with byte
   pre/postimages.
2. **The two proof-file repairs.** Only in
   `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs`: (a) make `expect` compile
   without moving `message` before it is emitted, keeping the `Q25_EXPECT_FAIL` text and the
   push/emit outcome unchanged; (b) remove the in-test `std::env::set_var` call and add a
   fail-closed precondition check that `RUM_CM740_OMIT` is present and equals `P2379`, while
   the env-mint negative assertion (`try_env_mint().is_some()` false, `guest_grant()` false)
   and all seventeen negatives, the predictions, the source-hit clause, `records.rs`, the
   owner/custody semantics and every predicate stay unchanged. Then compile. Any further
   compile-driven repair is allowed only in the worker's added or already-modified files,
   recorded like every other edit, and may not change a prediction or weaken a predicate. If
   the contract still does not compile after three bounded compilations, STOP with the raw
   diagnostics.
3. **The authorized inventory replays.** After the last covered edit, run the three generate
   commands, then the nine validate commands, exactly as recorded, and quote the actual
   verdict lines for production fingerprints, C01/C02/certificates in both applicable modes,
   ownership discovery/exactness, rejection census, source positions, LP01 and typed-slot
   inventories. Compare every line with the accepted baseline; reproduce its rc values and
   verdict content or attribute each difference to the covered edits. Preserve the 623 keys,
   held/encoding cells, assertions, c103/c104 source/inverse rules and accepted canonical
   spellings. A stale-only fingerprint/certificate/position red caused by ordering is
   recorded as an ordering artifact and repaired mechanically with the authorized generator;
   never hand-edit or weaken an inventory. An unexplained delta, or an additional build
   needed for acceptance, is a gate for the director.
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
   proof keeps all thirteen named stops for ruling 769. A green proof permits the director to
   decide whether to release the independent generator/source stage, still owing
   positives/withholding on each applicable selected path, both modes, and the companion
   queued operations. Never close a row, spend a further build, introduce a new observer or
   cut a negative to fit the carve.

## Hard rules

- `.temp/brief-760/`, `.temp/brief-759/`, `.temp/brief-758/`, the accepted epoch and every
  sealed scene stay read-only; the live tree is untouched; the working copy is disposable and
  non-admission. No live edit, no build of the live tree, no git, no network, no new
  dependency, no `unsafe`, no new program/lane/selection/policy/epoch label; `.temp` only;
  never `/tmp`; never read `~/.config/ccds/api-key`.
- The authorized invocations are exactly the ≤3 repair compilations, the 3 recorded generate
  and 9 recorded validate commands (with their both-mode compiles), one default proof-only
  build and one fixture execution. Reprice cold cache, repair and negative work at entry and
  at each irreversible spend. A projected overrun stops for a director ruling, never a scope
  cut.
- The replay bar is the verdict-line content against the accepted baseline, including its
  recorded failures; neither rc 0 nor a green suite is invented. Never weaken a predicate,
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

- The entry reproduces the repaired 760 copy identity (`abb03ca5…`, 5,812 files) and the nine
  entry rows; the working-copy identity is recorded; every edit is itemized with byte
  pre/postimages; `records.rs` stays byte-identical to `7364f150…`.
- The two repairs compile; the selector is supplied by the fixture invocation and the test
  fails closed without it; no `unsafe`; the env-mint negative assertion and all seventeen
  negatives are intact.
- The three generate and nine validate commands reproduce the accepted verdict lines with
  every covered-edit delta attributed; both certificate modes are freshly compiled; every
  verdict line is quoted with its raw log citation.
- The ordinary, positive, withheld, sibling and negative predictions are exact, and every one
  is either observed or the proof fails with its raw receipt. The `NoHit` is the exact
  `records.rs:341–350` rejection; the three-image importer of `records.rs:283–305` is
  explicitly deferred and never claimed as passed.
- The 13 dispositions are complete; zero importable receipts; the class-wide bars are
  unchanged; the report is `rum-report/1` at `.temp/brief-761/report.json` with
  `.temp/brief-761/ledger.json`; `python3 scripts/report.py check 761` is green with every
  PROBLEM fixed; the final message is only `python3 scripts/report.py summary 761`.
- Budget: **5,500 s focused** (a carve from the corrected reserve, 26,457.97424923717 →
  20,957.97424923717 s), **0 s workspace, 0 s epoch**; unspent returns; a projected overrun
  stops for a director ruling, never a scope cut. The carve prices a fresh orientation, the
  two repairs and ≤3 compilations, the recorded cargo sequence (~3,815 s upper), the release
  proof build and the fixture (~350 s) and the close/report (~180 s).

## Report format

`rum-report/1` at `.temp/brief-761/report.json` (cumulative with `.temp/brief-761/ledger.json`),
`changes` empty (every write is under `.temp/brief-761/`); sections for entry and the working
copy, the two proof-file repairs and the contract, the authorized inventory replays, the
fixture executions and raw observations, the predictions versus actual outcomes, the 13
dispositions, and the scope; cite lines exactly. STOP at the proof gate: no live edit, no
admission, no generator/source stage, no 744 restart, no 745/746, no CM5.
