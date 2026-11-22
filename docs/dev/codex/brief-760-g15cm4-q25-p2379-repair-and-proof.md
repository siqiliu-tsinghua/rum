# Brief 760 — G15-CM4 Q25 ownership proof (P2379), stage three: repair the q25 module placement, then the authorized replays, one proof-only build and one bounded fixture invocation

**Director-dispatched, one stage, one gate.** This continues brief 759's STOP under ruling
768: the Q25 contract is already written in the working copy, the first authorized generate
failed to compile only because the added module file sits where its `#[path]` does not
resolve, and the director has authorized the repair, up to three bounded compilations, and
the remaining authorized commands. The work runs in a fresh copy of the authenticated 759
copy. The live tree stays untouched, `.temp/brief-759/` and `.temp/brief-758/` stay read-only,
no receipt is importable, and Q25's thirteen rows stay named stops until the director rules
after this gate.

## Goal

When this brief is done, in `.temp/brief-760/` only: the q25 module placement repaired, the
contract compiling, and a physical proof on the default P2379 row — a genuine source-backed
commitment on the actual allocation, the driver's delivery to that concrete `Arc<Self>`, one
legitimate omitted append, the second real append, and `Err(Output)` on unqueued; the withheld
variant with identical physical operations and exactly `NoHit` while both paths still execute;
two same-state siblings rejected at the concrete receiver in both directions; the
lifetime/custody negatives — after the prescribed inventory generators and validators have run
on the covered sources in **both** feature modes, and with individual dispositions for all
thirteen Q25 rows. The proof produces **zero importable receipts** and no admitted receipt.

## Context pointers

- Brief 759 (committed `c8df9f8c`) and its STOP scene `.temp/brief-759/`: `report.json`,
  `ledger.json`, `checkpoint.json`, `evidence/preimages.tsv` (7 rows), `evidence/postimages.tsv`
  (9 rows), `evidence/entry-identity.json`, `logs/entry-copy.log`, `logs/fingerprints.log`
  (the rc-101 compile failure), `logs/close.log`, and ruling 768
  (`.temp/director/ruling-768.md`).
- **The authenticated 759 copy.** `.temp/brief-759/copy`, 5,812 files, tree
  `e0647d430aec875f4ad79475238f3afe51ee432d221b5b50c8500e69217a8e7e`, containing the
  implementation: 6 modified files (`crates/rum-runtime/src/lib.rs`, `runtime/suspension/core.rs`,
  `runtime/suspension/core/driver.rs`, `crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs`,
  `rules/resume_frames.rs`, `tests/suspension_oracle.rs`) and 2 additions
  (`crates/rum-runtime/src/q25_ownership.rs`, 29,522 B;
  `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs`, 31,944 B).
  `crates/rum-stdlib/tests/suspension_oracle/mutations/records.rs` is untouched at
  `7364f150437a0d395caa6843dabbb99a0c2500443faaa6ad8951f6af2eb1aa28` (14,894 B).
- **The failure to repair.** `crates/rum-runtime/src/lib.rs:6527` carries
  `#[path = "q25_ownership.rs"]` on `pub mod q25` inside `mod cm740_probe`, so rustc looks for
  `crates/rum-runtime/src/cm740_probe/q25_ownership.rs`; the file is
  `crates/rum-runtime/src/q25_ownership.rs`. The exact diagnostic is in
  `.temp/brief-759/logs/fingerprints.log`.
- **The accepted epoch (read-only).** `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  (`7af7b461…`), build `809b6473…`, freeze `c724c9ab…`; the armed tree `bd8b7548…`, real tree
  `97eaeb45…`, disabled tree `b41d1873…`; the twelve label binaries; the accepted build receipt
  `.temp/brief-754/attempt-2/epoch/default-armed-build.json`.
- **The fixture.** The existing default P2379 selection (program `e5e3a1ff…rum`, setup
  `ae59ceca…rum`, pins `mutant-coverage.tsv:138`, `programs.tsv:957`,
  `.temp/brief-749/attempt-7/renders/one/mutants.tsv:138`; answers prior and `"X"`; trace
  preserved). No new rum program.
- **The links.** The same `path:line` set as brief 759 (raw sink `core.rs:617–636`; raw driver
  `core/driver.rs:37–95`; `suspension/typed/entry.rs:9–19`; `lib.rs:1566–1608`, `:1636–1648`,
  `:2101–2113`, `:2223–2255`; the eight concrete receivers; `records.rs:341–366`).
- **The accepted inventory baseline.** `.temp/brief-754/attempt-2/evidence/standing/armed/`:
  `generate-commands.json` (3), `validate-commands.json` (9), the twelve accepted verdict logs
  and their recorded rc values (several are recorded failures, rc 101 — that is the baseline
  to compare against, not a demand for naive rc 0).
- **The bars.** The 36 historical signatures, the 20 Stage C cells, C10, the pair
  `(45,5,false)` (200 cells), the four structural controls, the eight observer rejection
  families, the 97-row table. This proof adds no admission.

## Scope

- MAY create/modify: everything under `.temp/brief-760/` only — the working copy, its build
  target, logs and receipts, the report at `.temp/brief-760/report.json` and its ledger at
  `.temp/brief-760/ledger.json`.
- MAY read: the live source tree, the accepted epoch trees and binaries, `.temp/brief-759/`
  and `.temp/brief-758/` (both read-only), every earlier `.temp/brief-*` record.
- MUST: at entry, copy `.temp/brief-759/copy` once to `.temp/brief-760/copy` by copying
  inputs or copy-on-write, **never writable hardlinks to the accepted files**, and record the
  copy identity (5,812 files, tree `e0647d43…`, device/inode independence); verify the 7
  preimages and 9 postimages against their recorded hashes before the first edit.
- **Authorized invocations (exact).** (a) The module-placement repair, validated by at most
  **three** bounded cargo compilations of the working copy, each through
  `scripts/run-bounded.py` with a timeout derived from the measured samples and a receipt
  keyed by the real command, rc and bytes; (b) the three recorded generate commands, then
  (c) the nine recorded validate commands, each run as recorded (same `cargo test --locked
  --offline -j8` shape, default and no-default as the command states) through
  `scripts/run-bounded.py` — the both-mode compiles are authorized by ruling 768; (d) exactly
  one default proof-only build of the `suspension_oracle` target in the working copy
  (`--offline --locked -j 8 --release --message-format=json --no-run`, the epoch's absolute
  `cargo`/`rustc`, the working copy's manifest, the locked registry resolvable offline;
  target dir and `TMPDIR` under `.temp/brief-760/`); (e) exactly one bounded single-threaded
  fixture invocation (`RUST_TEST_THREADS=1`, one test filter, `--exact`, `--nocapture`)
  through `scripts/run-bounded.py`. No other cargo build or test execution may run; anything
  else is a gate for the director, not an automatic spend.
- MUST NOT: edit any live file; write under `.temp/brief-758/`, `.temp/brief-759/` or any
  sealed scene; compile the live tree; run git; touch the network; add a dependency or
  `unsafe`; add a program, a selection, a lane, a policy or an epoch label; produce an
  importable or admitted receipt. The membership stays 28/3/7/113 and the pair `(45,5,false)`
  stays Open at zero admitted receipts.

## Deliverable

1. **Entry and the working copy.** Re-authenticate the 759 copy (tree `e0647d43…`, 5,812
   files; the 7 preimages against the 758 copy and the 9 postimages against the 759 copy);
   create the working copy and record its identity; record every subsequent edit with byte
   pre/postimages.
2. **The repair.** Make the q25 module placement match its `#[path]` (move the file to
   `crates/rum-runtime/src/cm740_probe/q25_ownership.rs`, or adjust the attribute and file
   together), then compile. Further compile-driven repairs are allowed only in the worker's
   added or already-modified files, are recorded like every other edit, and may not change
   the fixture predictions, the source-hit clause, `records.rs`, the owner/custody semantics
   or any predicate. If the contract still does not compile after three bounded compilations,
   STOP with the raw diagnostics. The verifier next to the evidence must carry digest and
   line checks for `records.rs:341–351`.
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
   proof keeps all thirteen named stops for ruling 766. A green proof permits the director to
   decide whether to release the independent generator/source stage, still owing
   positives/withholding on each applicable selected path, both modes, and the companion
   queued operations. Never close a row, spend a further build, introduce a new observer or
   cut a negative to fit the carve.

## Hard rules

- `.temp/brief-758/`, `.temp/brief-759/`, the accepted epoch and every sealed scene stay
  read-only; the live tree is untouched; the working copy is disposable and non-admission. No
  live edit, no build of the live tree, no git, no network, no new dependency, no `unsafe`,
  no new program/lane/selection/policy/epoch label; `.temp` only; never `/tmp`; never read
  `~/.config/ccds/api-key`.
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

- The entry reproduces the 759 copy identity and the 7/9 pre/postimages; the working-copy
  identity is recorded; every edit is itemized with byte pre/postimages.
- The repair's compilations are receipted; the contract compiles; the fixture path is
  fail-closed; `records.rs` is byte-identical to `7364f150…`.
- The three generate and nine validate commands reproduce the accepted verdict lines with
  every covered-edit delta attributed; both certificate modes are freshly compiled; every
  verdict line is quoted with its raw log citation.
- The ordinary, positive, withheld, sibling and negative predictions are exact, and every one
  is either observed or the proof fails with its raw receipt. The `NoHit` is the exact
  `records.rs:341–350` rejection; the three-image importer of `records.rs:283–305` is
  explicitly deferred and never claimed as passed.
- The 13 dispositions are complete; zero importable receipts; the class-wide bars are
  unchanged; the report is `rum-report/1` at `.temp/brief-760/report.json` with
  `.temp/brief-760/ledger.json`; `python3 scripts/report.py check 760` is green with every
  PROBLEM fixed; the final message is only `python3 scripts/report.py summary 760`.
- Budget: **5,500 s focused** (a carve from the stage reserve, 27,086.72298936417 →
  21,586.72298936417 s), **0 s workspace, 0 s epoch**; unspent returns; a projected overrun
  stops for a director ruling, never a scope cut. The carve prices a fresh orientation
  (~2,557 s measured in attempt 2 before the first scene write), the repair (~600 s), the
  remaining cargo sequence (~1,800 s upper) and the close/report (~180 s).

## Report format

`rum-report/1` at `.temp/brief-760/report.json` (cumulative with `.temp/brief-760/ledger.json`),
`changes` empty (every write is under `.temp/brief-760/`); sections for entry and the working
copy, the repair and the contract, the authorized inventory replays, the fixture executions
and raw observations, the predictions versus actual outcomes, the 13 dispositions, and the
scope; cite lines exactly. STOP at the proof gate: no live edit, no admission, no generator/
source stage, no 744 restart, no 745/746, no CM5.

## Gate c119 (2026-11-23) — brief 760 stopped correctly at the three-compilation bound; the contract compiles, two proof-file errors remain; ruling 769 authorizes the repair as brief 761

- Run: `grk` (grok-4.7 xhigh), unit `brief-760-grk-20261122-235747-817579`, session
  `e0245b45-9bb1-43e8-bb8a-6c0510f2dcd3`, 2026-11-22 23:57:47 – 2026-11-23 00:18:13 CST
  (wall ≈ 1,226 s); `status=success turns=67 cost_usd=4.942238 refused=0`; peak
  2,790,141,952 B (2.60 GiB, oom 0/0), rc 0; log `.temp/logs/grk_brief_760.log`
  (1,928,299 B); scene `.temp/brief-760/` (7,067 files, 1,806,344,850 B). Report STOP:
  `changes` empty, 3 deviations, 1 question (`fourth-compile`); `report.py check 760` OK
  (49 citations).
- Delivered: the entry (759 copy 5,812 files `e0647d43…`; 7/7 preimages against the 758 copy,
  9/9 postimages against the 759 copy; `records.rs` `7364f150…`; the working copy with
  `INODE PASS compared=5812 shared_inodes=0 devices=[53]`); the repair — the q25 module moved
  to `crates/rum-runtime/src/cm740_probe/q25_ownership.rs` (`43416be6…`, 29,522 B) matching
  the `#[path]` at `lib.rs:6526–6527`, then two `Arc` unsized-coercion fixes in
  `core.rs:643–681` (`core.rs` final `7766b20e…`, 33,984 B); three bounded compilations
  (2400 s kill bound, 32× the 73.687540332 s warm sample), each receipted: compile 1
  `E0308` at `core.rs:659`, compile 2 `E0308` at `core.rs:661`, compile 3 builds
  `rum-runtime` (lib, 121 warnings) and `rum-stdlib` (lib, 15 warnings) and fails only in
  the test target — `E0382` moved `message` at `q25_p2379_proof.rs:419:40` and `E0133`
  `std::env::set_var` at `q25_p2379_proof.rs:495:5` under edition 2024. The 3 generate + 9
  validate commands, the proof-only build and the fixture were not launched; no prediction
  and no `NoHit` observed; `records.rs:283–305` deferred; all 13 rows named stops (anchors
  present, unexecuted); zero importable and admitted receipts; no live write, no git, no
  network, no `unsafe`.
- My gate: 16/16 pre/postimages re-hashed; the 760 copy re-hashed (moved q25 `43416be6…`,
  `core.rs` `7766b20e…`, `q25_p2379_proof.rs` still the 759 `73d92bd0…`, `records.rs`
  `7364f150…`) and re-walked with the scene's recipe (5,812 files,
  `abb03ca582c77860b9469625bc057d845e5b0d6be196cc0d530944c859349c85`); 0 shared inodes
  between the copies; the three compiles read directly (real builds, exact diagnostics,
  both libraries built in compile 3); the repaired `core.rs` read and judged semantically
  sound (strong count back to 1 before the log, the registered `Weak<dyn>` on the same
  allocation, no predicate or ordering change); `RUM_CM740_OMIT` confirmed read lazily by
  the thread-local `SELECTED` at `lib.rs:182`; scope clean (0 live files newer than the
  dispatch, 0 files newer in the 758/759 scenes, no live target, empty receipts; the three
  `.git-attempts` are the grk harness's read-only probes); budget verified
  (628.748740127 s focused of 5,500; 4,871.251259873 s returned; reserve
  26,457.97424923717 s).
- Ruling 769 (`.temp/director/ruling-769.md`): the STOP is correct — the third bounded
  compilation consumed the authorization and a fourth is a director gate; the worker did not
  wrap `set_var` in `unsafe`, did not touch a prediction, `records.rs` or any predicate, and
  did not launch the remaining sequence. The record is accepted; **Q25 stays unwaived** (13
  named stops, zero admitted receipts). Question `fourth-compile` is authorized as **brief
  761**: re-enter from the repaired 760 copy, repair only the two proof-file errors (the
  moved-message borrow with the emitted text and order unchanged; remove the in-test
  `set_var` and supply `RUM_CM740_OMIT=P2379` in the one fixture invocation's environment,
  keeping the env-mint assertion and `guest_grant=false`), at most three bounded
  compilations, then the recorded 3 generate + 9 validate commands, one default proof-only
  build, one bounded single-threaded fixture invocation, the 13 dispositions and zero
  importable receipts. Budget: 5,500 s focused carved from the corrected reserve
  (26,457.97424923717 → 20,957.97424923717 s), 0 workspace, 0 epoch; unspent returns.
- Next: brief 761's gate follows; then the generator/source gate, the deterministic render,
  the re-freeze, the fresh epoch, the Stage C/C10 readmission and a further 744 restart,
  each gated; 745/746 stay held; CM5 unauthorized.
