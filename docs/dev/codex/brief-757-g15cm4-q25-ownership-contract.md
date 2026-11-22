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

## Gate c116 (2026-11-22) — brief 757 accepted; the Q25 ownership contract is complete, zero unknown; the P2379 proof is ordered

Brief 757 (design-only, `Scope: NONE`, committed `15f78a64`) ran on `cdx` (Codex CLI 0.171.0,
`gpt-6.2-astra`, reasoning effort max), unit `brief-757-cdx-20261122-210216-746520`,
2026-11-22T21:02:16–21:42:31 CST, rc 0, the 20 GiB scope peaked at 364,040,192 B (0.34 GiB,
oom 0/0), 0 repo changes; dispatch log `.temp/logs/cdx_brief_757.log` (3,165,840 B, 27,134
lines); scene `.temp/brief-757/` (34 files, 4,451,821 B); report DONE (25 PASS, 1 FLAGGED, 3
measurements PASS, 1 deviation, 2 questions, 0 disclosures, 0 changes; `scripts/report.py
check 757` OK with 6,781 citations). The FLAGGED check is `physical-proof`: the design gate is
complete, but ruling 765 requires an approved physical allocation-to-consumption proof before
release — the expected state of a design gate.

Delivered: (1) the ownership model — four joined events (selected producer return, queue
custody, driver delivery, the consuming statement) with every link at `path:line`: the raw
sink `K:617–632` (`let frame = Arc::new(frame)` 626, `record_descendant_frame` 627,
`push_back` 632), the raw driver `D:59–75` (`EntryRef::Legacy`, `Arc::clone` 60, decode
63–74, `frame.resume(self, input)` 75), the eight concrete `Arc<Self>::resume` entries and
the consuming effect per row; a private source-authenticated guest capability (typed armed
source, pre/postimages, image, tuple, program bytes, setup/answers, constituent ordinal,
mode/profile, fixture nonce, half and run authority); a one-use non-Clone construction
handoff through the real `Err(InputRequired)` arm and its helper arguments to the concrete
constructor and the generic insertion; a queue-side custody witness mirroring all entry kinds
with an immutable birth generation and an independently verified suffix transfer; tentative
registration committed only after the push plus the same helper's actual `InputRequired`
return; a driver-scoped offer plus a concrete-self check (`TypeId`, full key, actual
`Arc<Self>` data address, live registered `Weak` equality) before any authority; one-use
`Offered → Claimed` consumption immediately before the effect with `frame(row)` and
`token == claimed token` rechecked; retirement on completion, abandon, cancel, decode fault,
error, unwind and episode end; no runtime-liveness extension, no default-path cost, no
`unsafe`. It names the row→type map, the call edge, value equality, a borrowed state and a
cloned fixture as insufficient and keeps `CM740-SEMANTIC`, the four structural controls, the
descendant edges and the Stage C publication/queue claims unchanged. (2) the 13-row table
(`q25.tsv`, `q25-verdicts.log`) — P0214/P0216 (`RawTraceFrameIdentity`), P1070
(`ClonedGuardedSelectionFrameIdentity`), P2348/P2354 (`FrozenStringOutputFrameIdentity`),
P2379/P2381/P2385/P2387/P2409/P2411/P2416/P2418 (`FrozenStringRuleFrameIdentity`), 8
concrete types, each with selected source, constructor, `Arc<Self>` receiver, real effect,
registration, authentication, predicate, expected identity, queued counterpart, negatives
and historical signature: **13 proof needed, 0 unknown, 0 design stops**, `PortBlocked
pending approved proof`. (3) the three companions P0554/P1562/P1563 — exact selected source
shape, the actual unqueued selector/request return, the independent queued operation permit,
the matching queued store/retirement omission, and the resumed==queued argument; frozen
carriers, phases and observers retained; `design-carried`, no runtime credit. (4) the proof
fixture on default P2379 — the existing selection (program `e5e3a1ff…rum`, setup
`ae59ceca…rum`, pins `Cov:138`/`Progs:957`/`Mut:138`); ordinary reference (both halves
`{"X","X"}`, two appends, no selected effect), positive (one genuine commitment and one
dispatch claim on the first allocation, one ordered `OmitActualStringCaseAppend` hit,
unqueued `{"X"}` vs queued `{"X","X"}`, `Err(Output)`, `public_output_exact=false`, observer
Strict, the native bit recomputed from `W:142–149`), withheld (same source and genuine Err
minus only the commit: both physical operations still run, zero claims/hits, both halves
`{"X","X"}`, exact `NoHit` at `records.rs:341–350` before the identity-survivor branch at
`352–366`), three simultaneous distinct F allocations A/B/C with equal-valued siblings
rejected in both directions, and 17 negative cases; the one-image limitation is stated (the
rejection clause is exercised in isolation, the envelope is non-importable, the three-image
importer of `records.rs:283–305` is deferred). (5) the class integration `class-97.tsv` — all
97 original decisions retained with their original verdict lines and hit sites (66
missing-link, 18 carrier-shape, 13 stop; 84 design-carried, 13 proof needed, 0 new receipts)
— and `falsifier.md` binding the 36 historical signatures, the 20 Stage C cells, C10, the
P0813/P2593/P0492/P1476 calibrations, the pair `(45,5,false)` in both modes (200 cells, bar
`required=50 executed=50 caught=45 armed_without_hit=5 complete_per_cell_bar=false
owner_state=Open encoding=0`), the four structural controls, the eight observer rejection
families and the 244-cell shakedown order. (6) the proof brief and forecast — one fresh
disposable copy, one default proof-only build, one bounded single-threaded fixture
invocation, standing inventories before acceptance, individual dispositions for all 13, zero
importable receipts; 1,125.171860590–3,461.374142684478480 focused s from the measured
samples; proposed 3,500 s separate carve.

My gate: all 1,568 full-path `path:line «quote»` citations in the scene's tables and
documents match the armed tree; all 2,264 quoted evidence items across the six evidence
JSONL files match their raw logs at the cited lines; the 36 historical signature files
re-hash 36/36 and every cited line is in range; the 20 Stage C packet hashes are 20/20; the
200 pair cells re-derive 45 `Ok(())` + 5 `Err(NoHit)` per row per mode with the four bar
lines exact; the eight structural passes and eight observer rejections per mode are original
lines; the source shapes were read directly (`K:617–636`, `D:59–84`, `E:9–19`,
`L:1566–1608`, `L:1636–1648`, `L:2101–2113`, `L:2223–2255`, `Leaf:170–190`, `T:126–140`,
`G:348–392`, `S:232–247/424–432`, `C:335–365/612–620`, `R:330–338/590–598`, `RR:262–292`);
the fixture program/setup bytes, the P2379 tuple/phase/observer pins and the `NoHit` and
three-image clauses match; 42/42 pinned input hashes match and the three trees are 5,810
files each (identities inherited from the accepted 756 calibration); the scope is exactly
the inherited 23 git entries with HEAD = `origin/main` = `15f78a64`, no `.git-attempts`, no
file under `crates/`, `target/`, `docs/` or `scripts/` newer than the dispatch, no scene
scripts, and the log's `subprocess`/`cargo` mentions are the worker reading `scripts/report.py`
and older brief text, not executions; the budget is 2,373.758704406 s focused of 2,500,
126.241295594 s returned, 0 workspace, 0 epoch, reserve 32,526.16508406417 s. The one
deviation (two intermediate read-only lookups corrected against the actual 50-cell importers
and the P1507 raw observer logs) is accepted; no verdict changed and the final evidence
corpus is what I verified.

Ruling 766 (`.temp/director/ruling-766.md`) accepts the brief; Q25 is still not waived — the
13 rows stay named stops at zero admitted receipts. `q25-proof` is authorized: brief 758, the
isolated default P2379 ownership proof (one fresh copy, one default proof-only build, one
bounded single-threaded fixture invocation, the ordinary/positive/withheld pairs, A/B/C
siblings, the 17 negatives, individual dispositions for all 13, zero importable receipts, a
director gate after), carves 3,500 s focused (32,526.16508406417 → 29,026.16508406417 s), 0
workspace, 0 epoch, unspent returns, a projected overrun stops for a ruling. `after-proof` is
deferred to the proof gate; the later generator/source stage will record a fresh explicit
draw if the remaining reserve is short (standing user ruling 「预算不够就追加吧」), never a
scope cut. Next: brief 758 and its gate, then the generator/source gate, the deterministic
render, the re-freeze, the fresh epoch, the Stage C/C10 readmission and a further 744
restart, each gated; 745/746 held; CM5 unauthorized; everything sealed (c76–c99) stays
sealed. The standing state is unchanged: bar 571/4,422, 37/134, 4/133, 0/26; pair
`(45,5,false)` Open at zero admitted receipts; C2 118 Residual + 0 Lost; 51 owners and five
source duties Open; membership 28/3/7/113; `CHANGELOG.md`/S10 blocked.
