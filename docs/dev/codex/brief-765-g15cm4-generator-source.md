# Brief 765 — G15-CM4 generator/source stage, stage one: the source-level forms of the accepted design carrying the proven Q25 raw-driver contract, the per-row AST and physical validation for the 97 in-scope rows, the 97×2 positive/withheld non-admission fixtures, the standing inventory replays and the two-mode source A/B

**Director-dispatched, one stage, one gate.** This opens the generator/source stage under
ruling 773: brief 764's default P2379 ownership proof is green (`Q25_PROOF_OK`,
`failures=0`, `rows_closed=0`, zero importable receipts), the receiver classifier accepts the
explicit typed `syn::FnArg::Receiver` with its falsifier firing, and the proof binary built.
The stage still owes positives and withholding on each applicable selected path, both modes,
and the companion queued operations; the proof's one concrete execution does not close any
Q25 row. The work runs in a fresh copy of the 764 copy, with the accepted generator roots
copied into the scene and extended there. The live tree stays untouched, `.temp/brief-764/`
and every earlier scene stay read-only, no receipt is importable, and Q25's thirteen rows
stay named stops until the director rules after this gate.

## Goal

When this brief is done, in `.temp/brief-765/` only: the accepted generator roots extended in
the scene; the source-level forms of the accepted design implemented in the working copy for
the class (the typed witness/macro/chain group, the legacy Arc/raw group including First and
the cursor moves, the synchronous evaluation/request-return carriers, the paired queued
success transport and the typed pass-through arms), carrying the proven Q25 raw-driver
ownership contract; a per-row AST round-trip and physical source→recipient→consumer
validation for all 97 in-scope rows with a zero-unknown statement; a positive/withheld
non-admission fixture per applicable shape (97 rows × 2 modes) where the withheld boundary
shows zero authorized hit and the positive twin proves actual source, receipt and physical
consumption, with the importer `NoHit` for the withheld and both physical paths visible on
rum's resumed/queued observations; the standing inventory replays after the last covered
edit and before the first acceptance suite; the two-mode source A/B workspace suites; the
probe/falsifier checks; and a measured hot-path A/B only if the authorized change touches
those paths. The stage produces **zero importable receipts**, admits nothing and closes no
row; the thirteen Q25 rows stay named stops.

## Context pointers

- Ruling 773 (`.temp/director/ruling-773.md`); brief 764 (committed `009a09bb`) and its gate
  (committed `5d50828c`); the brief 764 scene `.temp/brief-764/` (`report.json`,
  `ledger.json`, `evidence/`, the raw logs, the green fixture at
  `.temp/brief-764/logs/fixture.log`, the proof-built binary
  `suspension_oracle-547fa4362abc58a1`, `b3f1d4a8…`, 122,054,584 B).
- **The 764 copy (the entry).** `.temp/brief-764/copy`, source tree (excluding child
  directories named `target` and `.temp`; the walk hashes `F\0<relative path>\0<file bytes>`
  in sorted walk order) 5,812 files,
  `0f8cbc32b864f0d58a2a6052e607bf16cd37e5df4aa998f5421e7b4e080e604a`. It carries,
  byte-for-byte:
  - `crates/rum-runtime/src/lib.rs` — 258,973 B, `15d91070bbad767e098d5d615c110c8597198d57c50d6e4d96686664540691fc`
  - `crates/rum-runtime/src/cm740_probe/q25_ownership.rs` — 29,522 B,
    `43416be62901a79c797f057b81a7364b668a91099188098250dbeaaaa68f2052`
    (`crates/rum-runtime/src/q25_ownership.rs` must stay absent)
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
  - `crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs` — 36,042 B,
    `ee789084c85f6794e5267d8b817ec91b089a015705395373e8d6c9daad653bea` (the repaired,
    green proof test; the 36,042 B size is recorded adoption debt)
  - `crates/rum-stdlib/tests/suspension_oracle/mutations/records.rs` — 14,894 B,
    `7364f150437a0d395caa6843dabbb99a0c2500443faaa6ad8951f6af2eb1aa28`
  - `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` — 91,646 B,
    `85d75662c581acb62bd8a4fe44d61b25a7ec80e44abf490dac45f4b893da2434` (adopted)
  - `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` — 18,162 B,
    `4337950c5b5e5f2a26f94d4aee66aacc0263a62b732d34b74e3021d1182dbbc7` (adopted)
  - the review table `target/brief-544/local-contracts.tsv` — 91,646 B, `85d75662…`.
- **The accepted generator roots (read-only inputs; copy them into
  `.temp/brief-765/generator/` and extend them there).**
  - `.temp/brief-749/attempt-7/generator/brief-742/continuation-88/fn_split.py` — 93,456 B,
    `4971a55308a3930edf6a5f521c6690623da0441169e759b563b2322a8b80be7a`
  - `.temp/brief-749/attempt-7/generator/brief-743/stage-a/compose.py` — 6,876 B,
    `e20c1bb196bbe4a1aa857f445257879b066e61fb7fc399b74f31edb915c2ec36`
  - `.temp/brief-749/attempt-7/generator/control.rs.in` — 7,022 B,
    `2cc32ba46522783f4cc05fce37a3d45314868ab8ef3cb8c4bdbe9c9c3f95482e`
  - `.temp/brief-749/attempt-7/qualified_dispatch.py` — 9,102 B,
    `0aa7221acbe43d7492209ca90a15dddfdf07daad3efa25e88aabb7fda66e640a`
- **The accepted design and the complete inventory (read-only).**
  - `.temp/brief-756/design.md` — 16,545 B,
    `486ed0c0bd5fcf173c294d856a31f49f6a81ada738f516dccb67add50269725d`
  - `.temp/brief-756/falsifier.md` — 9,239 B,
    `4f2d99ed777440c184013eb3868fcfecaa595dd61427e31887e6590f3a1a3ce6`
  - `.temp/brief-756/implementation.md` — 9,173 B,
    `e9e04cc5252912f4bf26935b327685a93edf582628e4fefbf5e44d13899d49af` (item 1 is this
    stage)
  - `.temp/brief-756/frame-authorization.tsv` — 272,415 B,
    `f60f46ad93a66477480a4cccadeef70cb727aa1788d0931ac0b866a1819c6155` (97 rows + header)
  - `.temp/brief-756/row-evidence.jsonl` — 1,038,268 B,
    `10a14ae3797dcaa7b9c8af58514c149d7f82f6b73b6df31705f9bd98a2b357b4` (97 lines)
  - `.temp/brief-756/historical-signatures.tsv` — 55,160 B,
    `fd99e1a93cbf59540635597a72e2e688bc78179951f131fa165d532a5b39d2b9` (36 rows + header)
  - `.temp/brief-756/membership.tsv` — 63,184 B,
    `f4f0217e32ae7db6b68abc49d91aa3bca31f2f721b12ba69d922bc50881a0885` (151 rows + header)
  - `.temp/brief-756/membership-verdicts.log` — 33,965 B,
    `8b60f1cea681f423d963b5e4ecd576c37de247488bba1277edaa292b1c800b10`
- **The design's groups and stops.** Typed witness/macro/chain (50 typed), legacy Arc/raw
  (25 raw including First, 4 cursor-backed), synchronous evaluation/request-return carriers
  (16), paired queued success transport (P1814/P1817) and the typed pass-throughs
  (P0416/P0820/P0825 with their actual resumed-input arms). The 13 Q25 rows and the three
  companion anchors P0554/P1562/P1563 are the named stops this stage must supply evidence
  for, never close. The design's rules — the producer/returned-value/installation and the
  matching physical consumer on the selected route; no authority from a shared helper, file
  path, case number, lexical row string or a twin invocation; the actual constructor and
  consumer; the two real insertion sinks at `core.rs:626–632,655–672`; the `begin_concrete`
  first statement; the scoped guard across the result and the installation/pre-tail return;
  identical logical parent/child occurrence numbering; the ordinary AST and the driven
  entry/caller split preserved — are the spec.
- **The Q25 contract (proven, carried).** The default P2379 fixture proved the raw-driver
  ownership semantics: the explicit typed `self: std::sync::Arc<Self>` receiver, the
  first-statement `begin_concrete`, the source-backed commitment and dispatch claim on the
  concrete allocation, the ordered `OmitActualStringCaseAppend` hit, the withheld
  commit-only variant with `NoHit` at `records.rs:344–350` before `352–366`, the A/B/C
  equal-state siblings rejected both ways, and the 17 negatives. Its implementation files
  are in the entry copy (`cm740_probe/q25_ownership.rs` `43416be6…`, the postimages above,
  `mutations/records.rs` `7364f150…`). This stage carries that contract to each applicable
  selected path, both modes, with the companion queued operations.
- **The bars.** The 151-way membership; the 97 in-scope rows; the four structural controls;
  the 36 historical signatures; the 20 Stage C cells and C10; the pair `(45,5,false)` (200
  cells); the eight observer rejection families; the 97-row table. This stage adds no
  admission and no importable receipt.

## Scope

- MAY create/modify: everything under `.temp/brief-765/` only — the working copy, the
  scene's `generator/` directory, its build targets, logs, receipts and fixtures, the report
  at `.temp/brief-765/report.json` and its ledger at `.temp/brief-765/ledger.json`.
- MAY read: the live source tree, `.temp/brief-764/` and every earlier scene (read-only), the
  accepted epoch trees and binaries, `.temp/brief-749/attempt-7/`, `.temp/brief-756/`.
- MUST: at entry, copy `.temp/brief-764/copy` once to `.temp/brief-765/copy` by copying
  inputs or copy-on-write, **never writable hardlinks to the accepted files**; record the
  copy identity (5,812 files excluding child directories named `target` and `.temp`,
  `0f8cbc32…`) and device/inode independence; copy the four generator roots into
  `.temp/brief-765/generator/` and record their bytes; record every generator and source
  edit with byte pre/postimages.
- **Authorized invocations.** Every invocation this stage's deliverables require — the
  extended generator runs, the copy's builds and tests, the fixtures, the inventory replays,
  the source A/B suites and the probe/falsifier checks — is allowed **only** inside the
  scene, only as a bounded, receipted command through `scripts/run-bounded.py` where the line
  has used it, with host cargo at `-j8` under `scripts/memory-scope.sh` and tests at
  `RUST_TEST_THREADS=8 --no-fail-fast`; the full workspace suite runs once per brief at its
  acceptance point, and timing/perf runs are serial and alone. Every invocation is disclosed
  with its exact argv, rc and elapsed. The source edits are made through the extended
  generator or itemized scripts, never by hand; a hand edit of generated output, a
  hand-edited inventory, or a build of the live tree is forbidden. The four brief-763
  certificate commands and the other recorded replays are re-run here **only** as the
  inventory replays this stage's covered source requires (deliverable 6), not as a
  re-litigation of their verdicts.
- MUST NOT: edit any live file; write under `.temp/brief-764/` or any earlier scene; compile
  the live tree; run git; touch the network; add a dependency or `unsafe`; add a program, a
  lane, a selection, a policy or an epoch label; produce an importable or admitted receipt;
  render, re-freeze, build an epoch, admit anything, restart 744 or spend 745/746/CM5. The
  membership stays 28/3/7/113 and the pair `(45,5,false)` stays Open at zero admitted
  receipts.

## Deliverable

1. **Entry and the working copy.** Re-authenticate the 764 copy (source tree `0f8cbc32…`,
   5,812 files, the eleven rows, the 759 path absent); create the working copy and record its
   identity and inode independence; copy the four generator roots into the scene and record
   their bytes; record every subsequent edit with byte pre/postimages.
2. **The generator extension.** Extend `fn_split.py`, `compose.py`, `control.rs.in` and
   `qualified_dispatch.py` in `.temp/brief-765/generator/` to emit the source-level forms of
   `design.md` for the class: the source-bound guest installation, the typed and raw
   allocation/consumption, the cursor transfers, the evaluation/request-return carriers and
   the paired typed-success transport. Every emitted form is traceable to a design rule and
   its `row-evidence.jsonl` entry; no text-position workaround; no hand-patched output; the
   generator's AST inverses and canonical spellings are recomputed, never carried.
3. **The source-level implementation.** Implement those forms in the working copy for the 97
   in-scope rows, carrying the proven Q25 raw-driver contract to each applicable selected
   path (the 13 Q25 rows included, with their positives and withholding on each applicable
   selected path, both modes, and the companion queued operations). Preserve the ordinary
   production layouts, hooks and protected hot paths; the four controls' prior source
   authority; the accepted dispatch inverses; the unchanged ordinary bodies; the
   hot-path/resource evidence; the exact rejections; and both the original runtime and the
   adapter assertion duties. A row whose historical signature cannot be reproduced under the
   design is a named stop with the exact missing fact, never a waiver or a reclassification.
4. **The per-row AST round-trip and physical validation.** For every one of the 97 rows:
   name the actual producer, its actual returned value or suspension installation, and the
   matching physical consumer on the producer's selected route; verify source, carrier, phase
   and the typed/frozen branch against the row before emitting; resolve overloads and
   generics and both shared symbols in their actual impl; reject zero/multiple candidate
   nodes, lost sibling arms, an unproved erased kind, and newly reached sibling call sites.
   Produce the zero-unknown statement: every row has its object, source edge, recipe and
   isolation falsifier; any row without one is a named stop.
5. **The positive/withheld non-admission fixtures (97 rows × 2 modes).** For each applicable
   shape, generate an isolated non-admission image from the same typed generator and frozen
   program, and at the chosen actual constructor suppress only the commitment of that
   selected own source→frame receipt after the genuine producer arm while both actual
   production operations still run. The withheld boundary must show zero authorized hit, the
   original producer/value still observed, no unrelated hits and the appropriate uncaught
   predicate, with the importer `NoHit`; the positive twin must prove the actual source,
   receipt and physical consumption (generation, kind, opaque issuer, logical owner,
   occurrence and independent slot state) and show both physical paths on rum's
   resumed/queued observations. Never import a fixture's logs as a candidate catch or share
   its authority with a candidate run; store the command, image/selection/schema/env hashes,
   thread count, ordered raw verdict lines and full matched payloads. Zero importable
   receipts.
6. **The inventory replays and the source A/B.** After the last covered edit and before the
   first acceptance suite, replay every standing inventory the covered source feeds — the
   fingerprints, both certificate modes, the ownership discovery/exactness, the rejection
   census, LP01 and the typed slots — exactly as recorded in the accepted brief 754/761
   replays (`.temp/brief-754/attempt-2/receipts/`, `.temp/brief-761/`), quoting the verdict
   lines. A red caused only by a stale fingerprint, certificate or position cell is an
   ordering artifact: it must be repaired by the prescribed generator, never hand-edited, and
   reported as such. Then run the both-mode source A/B workspace suites (the accepted
   six-case workspace gate at 414 targets, the combined identity recomputed rather than
   carried), the probe/falsifier checks, and a measured hot-path A/B only if the authorized
   change touches those paths. No default-path cost; the off-by-default `AtomicU8` pattern
   stays; no new `unsafe` in `rum-core`/`rum-runtime`.
7. **The evidence package.** Deliver the new source evidence, the scope record, the 97-row
   dispositions, the unspent/charged budget and the cumulative ledger. **STOP at the
   generator gate**: no live edit, no deterministic render, no re-freeze, no fresh epoch, no
   admission, no 744 restart, no 745/746, no CM5. A wrong physical identity or an unmatched
   historical constituent stops at its first failed link with its raw evidence.

## Hard rules

- Never weaken a predicate, reclassify a row, exempt a stop, count a call edge as an
  authorization, count a text identity as a catch, or infer a runtime fact from the report.
  Quote every verdict line; a claim without `path:line` or a raw log line is not a verdict.
- The invariant is class-wide: a repair that closes one row recurs as the class. Every
  frame-gated own row and the five `FrameResumePassThrough` carriers are in scope; a row that
  genuinely cannot be carried by the design is a named stop with the design fact.
- The Q25 rows stay named stops. This stage supplies their positives and withholding
  evidence; it closes none, admits nothing, and produces no importable receipt.
- No hand edit of any inventory or generated output; no text-position workaround; the
  generator produces the bytes and its inverses are recomputed. A stale-only red is repaired
  by the prescribed generator, never by accepting or editing the stale cell.
- The four structural controls, the 36 historical signatures, the 20 Stage C cells, C10, the
  pair `(45,5,false)`, the eight observer rejection families and the 97-row table are
  retained unchanged; this stage adds no admission.
- `.temp/brief-764/` and every earlier scene are read-only; the live tree is untouched; the
  copy is disposable and non-admission; no live edit, no build of the live tree, no git, no
  network, no new dependency, no `unsafe`, no new program/lane/selection/policy/epoch label;
  `.temp` only; never `/tmp`; never read `~/.config/ccds/api-key`.
- The standing state is unchanged: bar 571/4,422, 37/134, 4/133, 0/26; the pair
  `(45,5,false)` Open at zero admitted receipts; C2 118 Residual + 0 Lost; the 51 owners and
  five source duties Open; membership 28/3/7/113; `CHANGELOG.md`/S10 blocked; CM5
  unauthorized; everything sealed (c76–c99) stays sealed; 745/746 held; no 744 restart.

## Verification (MANDATORY — you run it; iterate to green or to evidence)

- The entry reproduces the 764 copy tree identity (`0f8cbc32…`, 5,812 files) and the eleven
  rows; the working copy's identity and device/inode independence are recorded; the four
  generator roots' bytes are recorded; every generator and source edit is itemized with byte
  pre/postimages and only files under `.temp/brief-765/` change.
- The generator extension emits each form from a design rule and a row-evidence entry; the
  per-row AST round-trip and physical validation covers all 97 rows with the zero-unknown
  statement, and every named stop carries its exact missing fact.
- The Q25 contract is carried, not weakened: the explicit typed receiver, the
  `begin_concrete` first statement, the source-backed commitment and dispatch, the withheld
  commit-only variant's exact `NoHit`, and the sibling rejections hold on every applicable
  selected path in both modes; the companion queued operations are exercised.
- The 97×2 fixtures meet their bar: withheld zero authorized hit, positive actual
  source/receipt/consumption, importer `NoHit`, both physical paths on rum's resumed/queued
  observations, one observation and green worker each; zero importable receipts; the fixture
  logs are never shared with the candidate runs.
- The inventory replays run after the last covered edit and before the first acceptance
  suite, with their verdict lines quoted; any drift is explained; no inventory was hand
  edited. The both-mode source A/B suites are green under the recorded contract; the
  probe/falsifier checks pass; a hot-path A/B runs only if the change touches those paths.
- The controls and ordinary bodies are preserved: the four controls' source authority, the
  accepted dispatch inverses, the unchanged ordinary bodies, the hot-path/resource evidence,
  the exact rejections and both assertion duties.
- The report is `rum-report/1` at `.temp/brief-765/report.json` with
  `.temp/brief-765/ledger.json`; `python3 scripts/report.py check 765` is green with every
  PROBLEM fixed; the final message is only `python3 scripts/report.py summary 765`.

## Report format

`rum-report/1` at `.temp/brief-765/report.json` (cumulative with
`.temp/brief-765/ledger.json`), `changes` empty (every write is under `.temp/brief-765/`);
sections for the entry and the working copy, the generator extension, the source-level forms
and the per-row validation, the Q25 contract carried, the non-admission fixtures, the
inventory replays and the source A/B, the controls and ordinary-body scope, the 97-row
dispositions and zero-unknown statement, and the budget; cite lines exactly. STOP at the
generator gate: no live edit, no render, no re-freeze, no epoch, no admission, no 744
restart, no 745/746, no CM5.

## Budget

**focused 6,500 s + workspace 3,100 s + epoch 0**, carved from the corrected stage reserve
(34,899.923788470167 → **25,299.923788470167 s**); unspent returns; a projected overrun stops
for a director ruling, never a scope cut. The focused carve prices the generator/source
acceptance (the 756 forecast's 3,009.839 s Stage A proxy), the non-admission fixture model
upper bound (194 workers 2,961.635 s; one image 347.342 s) and the close; the workspace carve
prices the two-mode source A/B (2,936.272 s) and the close. The scene's stop line is
`remaining < stack_1x` (cold samples of pending commands plus the 180 s close) or
`remaining < 3× the next command sample`; `informational_stack_3x` is logged, not the stop
line. A wrong physical identity or an unmatched historical constituent stops at its first
failed link; do not skip ahead to a later deliverable to fit the carve.
