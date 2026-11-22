# Brief 759 — G15-CM4 Q25 ownership proof (P2379), stage two: the contract in the accepted copy, the authorized two-mode inventory replays, one proof-only build and one bounded fixture invocation

**Director-dispatched, one stage, one gate.** This continues brief 758's STOP under ruling
767: the entry and the copy were authenticated, the prescribed replay was found to need
library builds in both feature modes, and the director has now authorized those builds within
exact bounds. The work runs in a working copy of the accepted 758 copy. The live tree stays
untouched, `.temp/brief-758/` and the accepted epoch stay read-only, no receipt is
importable, and Q25's thirteen rows stay named stops until the director rules after this gate.

## Goal

When this brief is done, in `.temp/brief-759/` only: the Q25 ownership contract implemented in
the working copy and physically proved on the default P2379 row — a genuine source-backed
commitment on the actual allocation, the driver's delivery to that concrete `Arc<Self>`, one
legitimate omitted append, the second real append, and `Err(Output)` on unqueued; the withheld
variant with identical physical operations and exactly `NoHit` while both paths still execute;
two same-state siblings rejected at the concrete receiver in both directions; the
lifetime/custody negatives — after the prescribed inventory generators and validators have run
on the covered sources in **both** feature modes, and with individual dispositions for all
thirteen Q25 rows. The proof produces **zero importable receipts** and no admitted receipt.

## Context pointers

- Brief 758 (committed `1409ddb3`) and its accepted scene `.temp/brief-758/`:
  `evidence/entry-identity.json`, `evidence/scope.json`, `evidence/class-bindings.json`,
  `evidence/frozen-fixture.json`, `evidence/predictions.json`, `evidence/replay-build-dependencies.json`,
  `evidence/build-plan.json`, `evidence/dispositions.json`, the logs (`entry-copy-2.log`,
  `row-audit.log`, `replay-gate.log`, `close-evidence.log`), `report.json`, and ruling 767
  (`.temp/director/ruling-767.md`).
- **The accepted working copy.** `.temp/brief-758/copy` — the armed tree
  `bd8b754870377124f55aeab7c49f2d17af6d65752aa74eca4ca112c603acab9f`, 5,810 files, 0
  cross-tree shared inodes, 0 symlinks. Copy it once to `.temp/brief-759/copy` and
  re-authenticate before the first edit.
- **The accepted epoch (read-only).** `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  (`7af7b461…`), build `809b6473…`, freeze `c724c9ab…`; the real tree `97eaeb45…` and disabled
  tree `b41d1873…`; the twelve label binaries (`45916ecc…`/`011d61bd…`/`6919c52a…`,
  `b3b8b1e1…`/`c3d2d5d2…`/`c736d3c4…`, lane `b92b3ed2…`/`caaa5b6a…`/`c286c776…` and spot);
  the accepted build receipt `.temp/brief-754/attempt-2/epoch/default-armed-build.json`
  (`cargo test --offline --locked -j 8 -p rum-stdlib --message-format=json --test
  suspension_oracle --no-run --release`). Paths below are relative to the working copy.
- **The fixture.** The existing default P2379 selection: program
  `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/programs/e5e3a1ffb2d76ccefaef22f7c1a32ba8246e4ffd3933a1db1b21994a39dfd59b.rum:1`
  (`InputString["prior"];StringCases["ab",x_:>If[c15once,(c15once=False;Input["finish"]),"X"]]`)
  and setup `.../ae59ceca1fc118beea303570e170412c9457ea1148d6020f93568c6c4bc98933.rum:1`
  (`log={};scope=17;c15index=0;c15once=True;`); pins `mutant-coverage.tsv:138`,
  `programs.tsv:957`, `.temp/brief-749/attempt-7/renders/one/mutants.tsv:138`; answers prior
  and expression `"X"`; trace setting preserved. No new rum program.
- **The links.** `crates/rum-runtime/src/runtime/suspension/core.rs:617–636` (raw sink),
  `core/driver.rs:37–95` (`EntryRef::Legacy`, clone 60, decode 63–74, `frame.resume` 75),
  `suspension/typed/entry.rs:9–19`, `lib.rs:1566–1608` (`eligible`/`frame`), `:1636–1648`
  (`hit`), `:2101–2113` (`authenticated_parent`), `:2223–2255` (`resume_frame` counts),
  `runtime/debug_tools/trace/filter/leaf.rs:170–190` and `filter.rs:126–140` (P0214/P0216),
  `kernel_data/structure_data/replace/guarded.rs:348–392` (P1070),
  `rum-stdlib/src/strings/patterns/search/guarded/replace.rs:232–247/424–432`
  (P2348/P2354), `strings/patterns/search/rules/cases.rs:335–365/612–620`,
  `rules.rs:330–338/590–598`, `rules/resume_frames.rs:262–292`,
  `suspension_oracle/mutations/records.rs:283–305/341–366`,
  `fixtures/obligations/guest.rs:1169–1191/1308–1316`,
  `mutations/source_worker.rs:95–128/142–155`, `runtime/builtins_input.rs:266–277`.
- **The accepted inventory baseline.** `.temp/brief-754/attempt-2/evidence/standing/armed/` —
  `generate-commands.json` (3 commands: fingerprints, certificate-emit-default,
  certificate-emit-no-default) and `validate-commands.json` (9 commands: certificate-check
  default/no-default, ownership-discovery, ownership-exact, rejection-census, LP01
  default/no-default, typed-slot default/no-default), with their twelve accepted verdict logs
  and recorded rc values (several are recorded failures, rc 101 — that is the baseline to
  compare against, not a demand for naive rc 0). Historical costs: generate
  184.350364908 s, validate 338.566843054 s.
- **The bars.** The 36 historical signatures (18 rows × 2 modes), the 20 Stage C cells, C10,
  the pair `(45,5,false)` (200 cells), the four structural controls, the eight observer
  rejection families, and the 97-row table of briefs 756/757. This proof adds no admission.

## Scope

- MAY create/modify: everything under `.temp/brief-759/` only — the working copy, its build
  target, logs and receipts, the report at `.temp/brief-759/report.json` and its ledger at
  `.temp/brief-759/ledger.json`. The dispatch log and the director's files are outside the
  scene.
- MAY read: the live source tree, the accepted epoch trees and binaries, `.temp/brief-758/`
  (read-only), every earlier `.temp/brief-*` record. `.temp/brief-758/` and the accepted epoch
  and every sealed scene stay read-only.
- MUST: at entry, copy `.temp/brief-758/copy` once to `.temp/brief-759/copy` by copying
  inputs or copy-on-write, **never writable hardlinks to the accepted files**, and record the
  copy identity (5,810 files, tree digest `bd8b7548…`, device/inode independence).
- MUST: authenticate the parent from `.temp/brief-758/evidence/entry-identity.json`: the
  epoch/build/freeze, the twelve executables, the three trees and the 45 input pins, with the
  brief-757-document append as the single accepted entry deviation.
- **Authorized invocations (exact).** After the last covered edit in the working copy:
  (a) the three recorded generate commands, then (b) the nine recorded validate commands,
  each run as recorded (same `cargo test --locked --offline -j8` shape, default and
  no-default as the command states) through `scripts/run-bounded.py` with a timeout derived
  from the measured samples — the compiles in both feature modes are authorized by ruling
  767; (c) exactly one default proof-only build of the `suspension_oracle` test target in the
  working copy (`--offline --locked -j 8 --release --message-format=json --no-run`, the
  epoch's absolute `cargo`/`rustc`, the working copy's manifest, the locked registry
  resolvable offline; target dir and `TMPDIR` under `.temp/brief-759/`); (d) exactly one
  bounded single-threaded fixture invocation (`RUST_TEST_THREADS=1`, one test filter,
  `--exact`, `--nocapture`) through `scripts/run-bounded.py`. No other cargo build or test
  execution may run; anything else is a gate for the director, not an automatic spend.
- MUST NOT: edit any live file — source, inventory, render, policy, schema, manifest,
  selection; write under `.temp/brief-758/` or any sealed scene; compile the live tree; run
  git; touch the network; add a dependency or `unsafe`; add a program, a selection, a lane, a
  policy or an epoch label; produce an importable receipt or an admitted receipt. The
  membership stays 28/3/7/113 and the pair `(45,5,false)` stays Open at zero admitted
  receipts.

## Deliverable

1. **Entry and the working copy.** Re-authenticate the parent evidence; create the working
   copy and record its identity; record every subsequent edit with byte pre/postimages; keep
   a fail-closed AST recipe in the copied probe path. The positive uses the existing guest
   input and production control flow; the physical operation hooks and the withheld-only seam
   have separate private callers. The eight concrete entry projections have source-audited
   instantiations; calling a stub that accepts type names does not prove them.
2. **The contract in the working copy.** Implement, in the working copy only: the
   fixture-specific, source-authenticated one-use guest capability (typed armed source,
   pre/postimages, image, selected tuple, program bytes, setup/answers, constituent ordinal,
   mode/profile, fixture nonce, exact half, actual run authority); the one-use non-Clone
   construction handoff along the real `Err(InputRequired)` arm and its helper arguments to
   the concrete constructor and the generic insertion; registration at the common raw
   insertion with the handoff on that one call, reading the actual `Arc<F>` while F is
   concrete; the queue-side custody witness mirroring all entry kinds with the immutable
   birth generation and the independently verified suffix transfer; the driver-scoped offer
   from the existing `Arc` clone, the successful slot decode and the rooted front; the
   concrete-self check at `RR:269` before any field clone or borrow (`TypeId`, full key,
   actual `Arc<Self>` data address, live registered `Weak` equality); one-use
   `Offered → Claimed` consumption immediately before the effect with `frame(row)` and
   `token == claimed token`, the live issue, the actual successful value/branch and phase;
   retirement on completion, abandon, cancel, decode fault, error, unwind and episode end.
   Ship the verifier next to the evidence, with digest and line checks for the actual
   source-hit clause (`records.rs:341–351`) and the raw importer prerequisites. No forged
   source-auth file, env-only grant or guest-callable positive mint. Compile the proof image
   once with an isolated manifest and receipt for its real bytes; it must be recognizable as
   non-admission even when all positive checks succeed.
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
4. **The one bounded execution.** Run the trial groups in `forecast.json` in one process with
   fresh test-owned authorities and Sessions between pairs:
   - **Ordinary reference.** Both halves `{"X","X"}`, both appends once, the first source
     return/real frame delivery on unqueued, zero selected effects, `guest::validate = Ok(())`;
     unqueued has two pause observations, queued none.
   - **Positive.** The first allocation A receives one source-backed commitment and one
     successful dispatch claim; equality of install, front and concrete self for A; true
     `current_frame_records`; matching `bound_slots` and pair token; owner/path/occurrence and
     the actual selected return. One ordered `OmitActualStringCaseAppend` hit on that first
     delivery, the empty accumulator preserved, then a normal second append of X; unqueued
     `{"X"}`, queued `{"X","X"}`; `C:619` can run during the same outer resume for the second X
     and receives no permission; no extra selected arm or hit in queued; `guest::validate =
     Err(Output)`, `public_output_exact=false`, observer Strict; the independently computed
     native bit from `W:142–149`.
   - **Withheld.** Under the very same selected source and genuine Err, withhold only the
     commit that links A to its source. The physical source return, construction, queue
     insertion, lookup/offer attempt, dispatch of the same allocation within this trial,
     concrete successful input, both append invocations and the paired queued execution must
     all occur. Require one selected finish arm, zero committed own slots, zero granted
     dispatch/tail claims and zero source hits in the whole observation; both appends execute;
     both lists are `{"X","X"}`; both outcomes and all corresponding per-half observations
     agree with the ordinary reference; `Ok(())`, `public_output_exact=true`, identical
     reference native snapshots for the corresponding halves, Strict, one completed
     observation. The selection is armed. The exact rejection is **NoHit** at
     `records.rs:341–350`, before the identity-survivor branch at `352–366`. A missing source
     arm would be Unarmed and fails.
   - **Equal-state and lifetime.** Three simultaneous distinct allocations of the concrete F;
     A from the selected installation, B and C equal-valued siblings with different addresses;
     B and C each rejected at A's real front in the forward direction and in the reverse
     direction at unowned rooted sibling fronts; A's slot survives and A succeeds exactly once.
     Wrong front/kind/generation, stale-address (dead Weak, then the old issue against a live
     new issue with the numerically matching address), replay, second take, wrong source,
     cross-run/domain, nested custody, return substitution, early exit, abandon/cancel,
     wrong phase and wrong queued operation per `fixture-negatives.tsv`, each with its exact
     predicted observation. Record pre/post strong counts, weak-upgrade failure after disposal,
     queue lengths, ordered callbacks/drops, and clean state on normal and exceptional exit.
   Archive command, bounds, versions, code/manifests, image hash, result hashes, full
   observations, ordered application and diagnostic receipts. **Zero importable receipts.**
5. **The 13 dispositions and the report.** One disposition for each of the 13 Q25 rows:
   which actual anchors executed in this proof and which are supported only by the generic
   implementation plus static instantiation. A green design or one concrete execution does not
   prove physical reach or an own catch for the other rows. Fail the proof for a missing true
   receiver, false acceptance on either sibling, a different withheld rejection, a missing
   original operation, incorrect generation/custody, leaked authority, a changed guest outcome
   outside the one effect, unexplained inventory drift or a projected overrun. A failed common
   proof keeps all thirteen named stops for ruling 766. A green proof permits the director to
   decide whether to release the independent generator/source stage, still owing
   positives/withholding on each applicable selected path, both modes, and the companion queued
   operations. Never close a row, spend a further build, introduce a new observer or cut a
   negative to fit the carve.

## Hard rules

- `.temp/brief-758/`, the accepted epoch and every sealed scene stay read-only; the live tree
  is untouched; the working copy is disposable and non-admission. No live edit, no build of
  the live tree, no git, no network, no new dependency, no `unsafe`, no new
  program/lane/selection/policy/epoch label; `.temp` only; never `/tmp`; never read
  `~/.config/ccds/api-key`.
- The authorized invocations are exactly the 3 generate + 9 validate commands (with their
  both-mode compiles), one default proof-only build, and one fixture execution. Reprice cold
  cache, implementation and negative work at entry and at each irreversible spend. A
  projected overrun stops for a director ruling, never a scope cut.
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

- The entry authentication reproduces the parent identities; the working-copy identity is
  recorded; every edit is itemized with byte pre/postimages; the fixture path is fail-closed.
- The three generate and nine validate commands reproduce the accepted verdict lines with
  every covered-edit delta attributed; both certificate modes are freshly compiled; every
  verdict line is quoted with its raw log citation.
- The ordinary, positive, withheld, sibling and negative predictions are exact, and every one
  is either observed or the proof fails with its raw receipt. The `NoHit` is the exact
  `records.rs:341–350` rejection; the three-image importer of `records.rs:283–305` is
  explicitly deferred and never claimed as passed.
- The 13 dispositions are complete; zero importable receipts; the class-wide bars are
  unchanged; the report is `rum-report/1` at `.temp/brief-759/report.json` with
  `.temp/brief-759/ledger.json`; `python3 scripts/report.py check 759` is green with every
  PROBLEM fixed; the final message is only `python3 scripts/report.py summary 759`.
- Budget: **4,200 s focused** (a carve from the stage reserve, 31,310.50799778417 →
  27,110.50799778417 s), **0 s workspace, 0 s epoch**; unspent returns; a projected overrun
  stops for a director ruling, never a scope cut. The repriced upper: the accepted
  preparation/replay bracket 3,009.838742696978480 s + two mode compiles 2 ×
  176.74452894250 s + the proof build 176.74452894250 s + the fixture upper
  274.79087104500 s = 3,814.863200569478480 s; the carve leaves 385.1 s for the close, the
  report and its check.

## Report format

`rum-report/1` at `.temp/brief-759/report.json` (cumulative with `.temp/brief-759/ledger.json`),
`changes` empty (every write is under `.temp/brief-759/`); sections for entry and the working
copy, the contract implementation, the authorized inventory replays, the fixture executions
and raw observations, the predictions versus actual outcomes, the 13 dispositions, and the
scope; cite lines exactly. STOP at the proof gate: no live edit, no admission, no generator/
source stage, no 744 restart, no 745/746, no CM5.
