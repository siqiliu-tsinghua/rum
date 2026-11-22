# Brief 757 — G15-CM4 Q25 raw-receiver ownership contract: complete per-row design and proof-fixture plan (design-only, `Scope: NONE`)

**Director-dispatched, one stage, one gate, no production change.** This continues ruling 765
and brief 756's accepted class inventory. Brief 756 proved the frame-authorization class on
the accepted repaired epoch: `frame(row)` needs an authenticated resuming token, only
`resume_frame` pushes one, only the four structural controls' `control::source` scopes can
register a frame, and no own row can satisfy the predicate (95 frame-gated, 97 in scope, 54
outside; `.temp/brief-756/`). Its design left **13 Q25 rows** as named stops because no unique
source-issued key joins their allocated and consumed receivers: P0214/P0216
(`RawTraceFrameIdentity`), P1070 (`ClonedGuardedSelectionFrameIdentity`), P2348/P2354
(`FrozenStringOutputFrameIdentity`) and P2379/P2381/P2385/P2387/P2409/P2411/P2416/P2418
(`FrozenStringRuleFrameIdentity`). The design proposed a common raw-driver registration to be
tested as a new ownership contract, with a positive genuine Arc and two same-state siblings in
an isolated fixture. Ruling 765 does **not** waive those rows; it authorizes the contract
extension and orders this design before any proof implementation. The three companion anchors
P0554/P1562/P1563 are included for the 18-row bar. `Scope: NONE`: design only; no source,
generator, render, selection or epoch edit; no build; no binary execution.

## Goal

When this brief is done, in `.temp/brief-757/` only: a **complete, zero-unknown per-row design
of the Q25 ownership contract** — the exact registration and consumption boundary, the
authorization model, the identity, the lifecycle and the rejections, for each of the 13 rows
with `path:line` evidence; the exact source shape and queued counterpart for the three
companion anchors; the proof fixture that can falsify the contract (a positive genuine Arc, two
same-state siblings, and the named negatives); the integration of all of it into the 97-row
design and the class-wide falsifier; and the proof-implementation brief and its cost forecast.
A row that cannot be designed is a named `stop` with the exact missing fact, not an inference.
No live edit of any kind.

## Context pointers

- Read `AGENTS.md`; brief 623's "Campaign invariants"; rulings
  `.temp/director/ruling-765.md` (this gate), `ruling-764.md`, `ruling-759.md`, `ruling-758.md`.
- **The accepted epoch (read-only).** `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  (`7af7b461…`), build `809b6473…`, freeze `c724c9ab…`; the armed tree
  `.temp/brief-754/attempt-2/epoch/default-armed-bd8b754870377124f55aeab7c49f2d17af6d65752aa74eca4ca112c603acab9f`
  (5,810 files), the disabled tree `b41d1873…` and the real tree `97eaeb45…`; the twelve label
  binaries (`45916ecc…`/`011d61bd…`/`6919c52a…`, `b3b8b1e1…`/`c3d2d5d2…`/`c736d3c4…`, lane
  `b92b3ed2…`/`caaa5b6a…`/`c286c776…` and spot). Paths below are relative to the armed tree.
- **The class and the 13 stops (read-only).** `.temp/brief-756/frame-authorization.tsv`
  (97 rows), `row-verdicts.log`, `row-evidence.jsonl`, `membership.tsv` (151),
  `class.md`, `delta.md`, `issuers.md`, `design.md` (especially the Q25 paragraph),
  `falsifier.md`, `implementation.md`, `forecast.json`/`forecast.tsv`,
  `structural-controls.tsv`, `historical-signatures.tsv`/`historical-events.tsv`.
- **The raw driver, the sinks and the ledger.** `crates/rum-runtime/src/runtime/suspension/core/driver.rs:59-75`
  (`EntryRef::Legacy`) and `:105,110` (`EntryRef::Typed`); `runtime/suspension/typed/adapter.rs:52`
  (`control::resume`) and `:125-128` (`erase`/`Arc::new`); `runtime/suspension/core.rs:626-632`
  (raw sink) and `:655-672` (typed/erased sink); `lib.rs:1566-1590` (`eligible`), `:1604-1608`
  (`frame`), `:1636-1648` (`hit`), `:2161-2173` (`bind_allocated_frame`), `:2176-2212`
  (`bind_frame`), `:2217-2255` (`resume_frame`), `:505-516` (`authority`), `:532-557`
  (`source`), `:557-608` (`allocated`), `:616-644` (`resume`), `:813-905`
  (`record_descendant_frame`), `:2018-2066` (`ConstructionGuard`), `:2086-2156`
  (`construction_identity`/`authenticated_parent`). The 13 rows' families:
  `runtime/debug_tools/trace/filter.rs:187,220-225` and `trace/resume_frames.rs:69,75`
  (P0214/P0216); `runtime/kernel_data/structure_data/replace/guarded.rs:386,616` and
  `guarded/resume_frames.rs:100,106,111` (P1070); `rum-stdlib/src/strings/patterns/search/guarded/replace.rs:281,447,567,631,724`
  and `guarded/replace/resume_frames.rs:41,47,67,111,130` (P2348/P2354); the four concrete
  executing types at `strings/patterns/search/rules/resume_frames.rs:269,334,30,99` and the
  append gates `rules/cases.rs:702`/`rules.rs:698` (the eight frozen string-rule rows).
- **The three companion anchors.** `runtime/kernel_data/association_data.rs:812-833` (P0554:
  `around("P0554")` on the owned selector evaluation, then `ordinary("P0554")` and the
  `OmitAssociationPartStoreAfterSelectors` return); `runtime/session/session_requests.rs:280,286,351,354`
  (P1562/P1563: `around(...)` on the checkpoint evaluation, then `ordinary(...)` and the
  `OmitAcceptedRequestTerminalRetirement` return) and the queued request-return gate
  `L:2589,2655-2689`.
- **The historical probe and the oracle.** `.temp/brief-740/c7/default-armed-4d411242…/crates/rum-runtime/src/lib.rs:24-106`
  (the old sticky `ARMED` bit) and `.temp/brief-740/c7/campaign-default-armed-P0416-on.log`
  (the signature form); the 36 rows of `historical-signatures.tsv` and their raw files.
- **The standing bars.** The 20 Stage C cells (`.temp/brief-755/class-admission.json`), C10,
  the pair `(45,5,false)` at zero admitted receipts, the four structural controls, the eight
  observer/negative families, the generator/inventory bars of brief 623.
- Everything sealed (c76–c99), brief 756's scene and every earlier scene are read-only.

## Scope (`Scope: NONE`)

- MAY create/modify: everything under `.temp/brief-757/` only, with the report at
  `.temp/brief-757/report.json` and its ledger at `.temp/brief-757/ledger.json`.
- MAY read: the live source tree, the sealed epoch trees and binaries (hashes only), every
  earlier `.temp/brief-*` record.
- MUST NOT: edit any live file — source, inventory, render, policy, schema, manifest,
  selection; write under any sealed scene; compile or build; execute any epoch, lane or probe
  binary; add a program or a selection; use git; touch the network; add a dependency or
  `unsafe`. The membership stays 28 NativeFault / 3 DeferredPublication / 7 ProfileReturn /
  113 Ordinary; the pair `(45,5,false)` stays Open at zero admitted receipts.

## Deliverable

1. **The ownership-contract model.** The exact boundary and authority for the 13 rows, with
   `path:line` at every link: the registration site(s) (at the raw sink and/or the raw
   driver), the consumed-receiver site(s) (`EntryRef::Legacy`, the concrete `Arc<Self>::resume`),
   and the identity (physical allocation address, concrete `F`, nonzero generation, `Weak`
   liveness, episode/activation/owner, exact path/child index, occurrence, and the genuine
   source event or driver-owned return that authorizes the join). State what authorizes the
   join for these rows, given that their frames are constructed inside helpers with cloned or
   frozen state and no producer wrapper. Explain why the proposed common raw-driver
   registration is not a row→type map, a call edge, a value equality, a borrowed state or a
   cloned fixture; how the selected episode and the driver's own physical ownership
   authenticate; and the exact rejections (wrong receiver/kind/generation, two same-type
   siblings with equal values, a stale/reclaimed address, a replayed consumed pair, a second
   take, a cross-episode or cross-half reuse, a nested/abandoned run). The lifecycle:
   registration, liveness, one-use consumption, retirement on completion/abandon/cancel/decode
   fault/error/unwind/episode end; no extension of runtime liveness; no default-path cost; no
   new `unsafe`. The four structural controls' semantic path and the 20 Stage C cells stay
   unchanged.
2. **The per-row table (zero unknown).** One line for each of the 13: row, family, frame type,
   allocated-Arc site, construction site, consumed-receiver site, registration site,
   consumption/authentication site, the exact predicate, the expected identity, the negative
   cases, the historical signature (if any), and a verdict — **design-carried**, **proof
   needed**, or **stop X** with the exact missing fact. Quote the source at every site.
3. **The three companion anchors.** P0554 and P1562/P1563: the exact selected source shape,
   the actual unqueued selector/request return, the independent queued operation permit, the
   matching queued callback/store omission, and the resumed==queued correctness argument.
   They keep their frozen carriers/phases/observers; no conversion of `ordinary(row)` into an
   ambient grant.
4. **The proof fixture and the integration.** The isolated fixture that falsifies the contract:
   a positive genuine `Arc` on its actual anchor, two same-state siblings with equal values but
   unequal identities, the withheld/non-admission variant for a selected Q25 row (predicting
   exactly NoHit while both physical operations still run), the wrong receiver/kind/generation,
   stale address, replay and second-take negatives, and the observed source/value/phase
   receipts on rum's resumed and queued paths. Bind it to the class-wide falsifier: the 36
   historical signatures, the 20 Stage C cells unaffected, C10, the pair `(45,5,false)`, the
   four structural controls, the eight negative families, and the 97-row table's updated
   verdicts. Draft the proof-implementation brief (fresh copy, one build, one fixture
   execution, the gate criteria) and its cost forecast from the measured samples
   (`.temp/brief-756/forecast.json`; the fixture image 347.34157748625 s and the worker range
   1,057.17075841250–2,961.63494348500 s; the 25% Stage A stress 1,497.5345718547451 s).
5. **The zero-unknown statement.** The 13-row and 3-row tables complete, every verdict backed
   by a line, the list of `stop` rows (ideally empty) with exact missing facts, and the
   director's decision points. The report is `rum-report/1` at `.temp/brief-757/report.json`;
   before finishing run `python3 scripts/report.py check 757` and fix every PROBLEM; the final
   message is only `python3 scripts/report.py summary 757`.

## Hard rules

- Never weaken a predicate, reclassify a row, exempt a `stop`, count a call edge as an
  authorization, count a text identity as a catch, or infer a runtime fact from the report.
  Quote every verdict line; a claim without `path:line` or a raw log line is not a verdict.
- The contract is class-wide and must be the **best and most correct** extension, not a
  P0416-style local patch or a row→type map: the resumed path must equal the queued path; the
  four structural controls and the 20 Stage C cells are inviolable; the 13 rows are not
  dropped from the 97-row obligation.
- No source edit, no build, no binary execution, no new lane, no selection adjustment, no
  maintenance replay, no new epoch label, no workspace, no git, no network, no `unsafe`, no
  CM5. `.temp` only; never `/tmp`; never read `~/.config/ccds/api-key`.
- The standing state is unchanged: bar 571/4,422, 37/134, 4/133, 0/26; the pair
  `(45,5,false)` Open at zero admitted receipts; C2 118 Residual + 0 Lost; the 51 owners and
  five source duties Open; membership 28/3/7/113; `CHANGELOG.md` and S10 blocked; CM5
  unauthorized; everything sealed (c76–c99) sealed; 745/746 held.

## Verification

- The ownership model names every registration/consumption site with `path:line`, states the
  exact authorizing identity and predicate, and eliminates the row-map/call-edge/value-equality
  shortcuts by name.
- The 13-row table and the 3 companion rows are complete with zero unknown; every verdict cites
  its evidence; the four families' actual allocated and consumed objects are named.
- The proof fixture's positive and negative predictions are exact and bound to the class-wide
  falsifier; the proof-implementation brief and cost forecast are complete.
- Budget: **2,500 s focused** (a fresh carve from the stage reserve,
  34,899.92378847017 → 32,399.92378847017 s), **0 s workspace, 0 s epoch**; unspent returns;
  a projected overrun stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-757/report.json` (cumulative with `.temp/brief-757/ledger.json`),
`changes` empty (every write is under `.temp/brief-757/`); sections for the ownership model,
the 13-row table, the three companions, the proof fixture and integration, the implementation
brief and forecast, and the scope; cite lines exactly. STOP at the design: no implementation,
no 744 restart, no 745/746, no CM5.
