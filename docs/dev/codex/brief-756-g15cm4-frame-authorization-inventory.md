# Brief 756 — G15-CM4 frame-authorization inventory and design (design-only, `Scope: NONE`)

**Director-dispatched, one stage, one gate, no production change.** This continues ruling 764
and the 744 attempt-3 STOP. That attempt ran the repaired epoch's shakedown: default C10 was
caught, default P0416 armed at both producer pauses, emitted zero `ActualAttachedFrameInputBeforeTail`
hits, returned `Ok(())` with `public_output_exact=true` and `native_difference=false`, and
reproduced at one thread on the same binary `6919c52a…`; its real and disabled matrices are
byte-identical to the c7 baselines. The gate's static read of the repaired armed tree
(`.temp/brief-754/attempt-2/epoch/default-armed-bd8b754870…`) found that `frame(row)` is
`eligible(l,row,false)`, which requires `l.resuming.last()` to carry a frame identity
authenticated against the row's activation; `l.resuming` is pushed only by `resume_frame`,
which `control::resume` reaches only for frames registered in `FRAMES` by `control::allocated`
under a `control::source` scope; the only such scopes pass `["C1"]`, `["G15A"]`, `["G15B"]`,
`["Some"]`, and `authority()` requires the episode program `"CM740-SEMANTIC"` while own-row
guest runs open `"CM740-GUEST"`; the generic `bind_allocated_frame` has no callers. The c7
receipts came from a different sticky-flag probe. **This brief must independently re-derive
all of that, complete the class inventory with zero unknown rows, and design the handoff.**
`Scope: NONE`: inventory and design only; no source, generator, render, selection or epoch
edit; no build; no binary execution.

## Goal

When this brief is done, in `.temp/brief-756/` only: a **complete, zero-unknown inventory of
the frame-authorization class** on the accepted repaired epoch — every one of the 82
`frame()`-gated own rows and the five `FrameResumePassThrough` carriers carrying one verdict
with its frame object, construction site, resume site, activation path, expected token and
the exact link that authorizes or fails it; the P0416 failure placed on the exact chain link
(no bound frame, lookup/filter miss, parent-authentication failure, or another named fact);
the four structural controls recorded as the working reference; an exhaustive
negative-reference audit of every possible frame-token issuer; the c7-vs-ledger semantic
delta; the per-row carrier-shape decision; and the **general handoff design** with its
class-wide falsifier, its implementation briefs and its cost forecast. Every verdict cites
`path:line` or a raw log line. No live edit of any kind.

**Row-level stops.** A `stop` verdict is a row result, recorded with the exact missing fact;
it does **not** halt the scan of the remaining rows. Only a calibration failure (a known
answer derived wrongly) or an unresolvable chain link stops the brief.

## Context pointers

- Read `AGENTS.md`; brief 623's "Campaign invariants"; rulings
  `.temp/director/ruling-764.md` (this gate), `ruling-763.md`, `ruling-762.md`,
  `ruling-759.md`, `ruling-758.md`.
- **The accepted epoch (read-only).** `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  (`7af7b461…`), build `809b6473…`, freeze `c724c9ab…`; the armed tree
  `.temp/brief-754/attempt-2/epoch/default-armed-bd8b754870377124f55aeab7c49f2d17af6d65752aa74eca4ca112c603acab9f`
  (5,810 files), the disabled tree `b41d1873…` and the real tree `97eaeb45…`; the twelve
  label binaries (`45916ecc…`/`011d61bd…`/`6919c52a…`, `b3b8b1e1…`/`c3d2d5d2…`/`c736d3c4…`,
  lane `b92b3ed2…`/`caaa5b6a…`/`c286c776…` and spot). Paths below are relative to the armed
  tree.
- **The probe and frame machinery (the chain).** `crates/rum-runtime/src/lib.rs`:
  `eligible` `:1566-1590`, `frame` `:1604`, `hit` `:1636`, `bind_frame` `:2176`,
  `bind_allocated_frame` `:2161`, `resume_frame` `:2217-2254` (`l.resuming.push` `:2254`),
  `ConstructionGuard` `:2018`; the `control` submodule `:475-817` (`authority` `:506-530`,
  `source` `:532-556`, `allocated` `:557-608`, `resume` `:616-644`, the `resume_frame` call
  `:637`); `record_descendant_frame` `:813-905` and its two installation sinks
  `crates/rum-runtime/src/runtime/suspension/core.rs:627` and `:671`; the typed-frame adapter
  `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:52` (the `control::resume`
  call) and `:53` (the `driven()` branch); the P0416 frame
  `crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs:70-108` (the
  `resume` `:80-93`, `resume_cm740_driven_77e` `:95-108`, `frame("P0416")` `:105`,
  `hit` `:106`); its construction `crates/rum-runtime/src/runtime/definitions/definitions.rs:175`,
  `:197`, `:218` and the producer `:190`; the only `control::source` call sites
  `runtime/definitions/definition_lhs_preparation.rs:466` and `:717`,
  `runtime/kernel_data/rewrite_data/replace/all.rs:181` and `:193`,
  `runtime/scoping/callable/status_application.rs:131`, `:143`, `:171`.
- **The worker and its episode.** `crates/rum-stdlib/tests/suspension_oracle/mutations/source_worker.rs`
  (`cm_copy_observation_worker`, the CM740 environment and the `guest::run`/`guest::run_with_mutation_observer`
  calls); `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/guest.rs:1973` (and
  `:2254`) `begin(queued, "CM740-GUEST")`; `fixtures/obligations/semantic_controls.rs:84`
  `begin(queued, "CM740-SEMANTIC")` and the `source_auth::authenticate` path.
- **The historical probe and receipt.** `.temp/brief-740/c7/default-armed-4d41124241076c1227d8bdc68d7d029b1f523ae78ddfbee0736fdc062a91f5cf/crates/rum-runtime/src/lib.rs:24-106`
  (the old sticky-flag module: `enter` `:39`, `frame` `:53`, `hit` `:54`, `pause` `:58`) and
  `.temp/brief-740/c7/campaign-default-armed-P0416-on.log` (lines 8/10/11/13/385/386/389).
- **The measured stop (read-only).** `.temp/brief-744/attempt-3/portblocked.json`
  (`e6f0acdd…`), `finding.json` (`b619cace…`), the raw logs
  `packets/002/{real,disabled,armed,armed-one-thread}.log` (hashes `53fb9f2b…`, `33ded968…`,
  `5a73a668…`, `fcd7b9d3…`; the selected observation payload `5460a362…` equal across all
  four arms), the run/command records, the C10 packet `packets/001/` (the caught reference)
  and the cumulative report `.temp/brief-744/report.json`.
- **The accepted inventories and render (read-only).**
  `.temp/brief-752/evidence/hit-side-edges-and-values.tsv` (the 151 own verdicts: kinds
  typed-frame resume 55, initial call 48, continuation resume 33, other 15; verdicts
  static-witness 135, hit-side-edge-fix 10, cover-candidate 6; the producer/hit/enclosing/
  fixture/withholding columns); `.temp/brief-749/attempt-7/renders/one/mutants.tsv`
  (`e8b3b6e4…`, the 151-wide carrier/phase/selector/observer row), `overlays.tsv`
  (`5c3b02a3…`), `observer-policy.tsv` (`3953aa9f…`); brief 748's `SUMMARY.md` and
  `repair-plan.json` (the "95 already-working rows preserved, the four structural controls
  get the authenticated source/frame handoff" design sentence); brief 749's Stage A goal and
  its c103/c104 amendments.
- Everything sealed (c76–c99) and every earlier scene stay read-only.

## Scope (`Scope: NONE`)

- MAY create/modify: everything under `.temp/brief-756/` only, with the report at
  `.temp/brief-756/report.json` and its ledger at `.temp/brief-756/ledger.json`.
- MAY read: the live source tree, the sealed epoch trees and binaries (hashes only), every
  earlier `.temp/brief-*` record.
- MUST NOT: edit any live file — source, inventory, render, policy, schema, manifest,
  selection; write under any sealed scene; compile or build; execute any epoch, lane or
  probe binary; add a program or a selection; use git; touch the network; add a dependency
  or `unsafe`. The membership stays 28 NativeFault / 3 DeferredPublication / 7 ProfileReturn
  / 113 Ordinary; the pair `(45, 5, false)` stays Open at zero admitted receipts.

## Deliverable

1. **The authorization-chain proof, re-derived.** Every link from `frame(row)` down to the
   frame's construction and back up to the resuming token, with `path:line` at each step and
   the exact predicate at each guard: the generic ledger (`eligible`/`resume_frame`/
   `bind_frame`/`bind_allocated_frame`/`ConstructionGuard`), the typed-frame adapter
   (`control::resume`), the lane (`control::authority`/`source`/`allocated`/`resume`), the
   two installation sinks and `record_descendant_frame`, the queue-transport and descendant
   paths. Then place **P0416's failure** on the exact link: is there no `FRAMES`/`bound_slots`
   entry for `RuleLhsCompletionFrame` (no `control::source` for P0416, no
   `bind_allocated_frame` call), a lookup/filter miss (address, kind, generation, liveness,
   authority), a parent-authentication failure, or another named fact? Prove it from the
   source of the accepted armed tree and, where the source is not decisive, from the raw
   attempt-3 logs — never from the report's interpretation. State the three candidate causes
   and which the evidence eliminates.
2. **The class inventory — zero unknown rows.** One line per own row for every member of the
   class: the **82 `frame()`-gated own rows** (the gate's static split 46 typed-frame resume,
   22 continuation resume, 8 initial call, 6 other — re-derive it, do not trust it) and the
   **five `FrameResumePassThrough` carriers** P0416/P0820/P0825/P1814/P1817 (the gate found
   only the first three frame-gated; verify), with: row, class, family, kind, carrier,
   phase, guard expression, frame type, construction site (`path:line`), resume site, the
   activation/owner path, the token the hit needs, the link that would authorize it, and a
   verdict — **authorized**, **missing-link X**, or **carrier-shape Y** — with its exact
   evidence. The four structural controls (`C1`, `G15A`, `G15B`, `Some`) are the working
   reference and must be recorded link for link; they are not own rows and must keep working.
   A row whose chain cannot be resolved is a `stop` verdict with the exact missing fact, not
   an inference; a `stop` row that hides a known fact is a failure of this brief.
3. **The exhaustive negative-reference audit.** Every writer and reader of the frame-token
   state, with complete call sets: `l.resuming` pushes/pops, `resume_frame` callers,
   `bind_frame`/`bind_allocated_frame` callers, `FRAMES`/`bound_slots`/`queued_frame_issues`
   writers, `ConstructionGuard`/`construction_identity` users, `record_descendant_frame`'s
   generic and queue branches, the `queue_transport` token machinery, and any rendered
   `cm740_probe::` call that could issue or consume a frame identity. Prove whether the
   gate's claim — no own row can be frame-eligible on this epoch — is exact, or name the
   exception with its line. Include every row in the 151 whose hit is `frame()`-gated by a
   call executed **outside** a resume scope (which would also be unauthorized) or **inside**
   one (which depends on the bound frame).
4. **The c7-vs-ledger semantic delta.** The old sticky-flag probe's `enter`/`pause`/`frame`/
   `hit` semantics (`lib.rs:24-106` there) against the repaired ledger's; which historical
   signatures' guards depended on the sticky flag; what "same-activation hit" means under the
   ledger; and for each class row the **carrier-shape decision**: a frame handoff at the
   frame's construction/resume, an ordinary/queued carrier at the hit site, or another shape
   — with the correctness argument (the resumed path must equal the queued path; the
   historical predicate, flags and observer must be reproducible; real/disabled parity and
   the in-process assertion duties are preserved). A row whose historical signature cannot
   be reproduced under any shape is a named stop, not a waiver.
5. **The general handoff design.** The exact change to the generator/probe (the source-level
   shape and its anchor classes, not a text-position workaround), the authorization model
   (construction inside the row's driven activation, the frame identity, parent
   authentication, the queue/resume separation, the address/kind/generation/liveness/
   authority filters, the controls unchanged), and its **class-wide falsifier**: each of the
   18 own rows of the 744 schedule armed in both modes must show its historical ordered arm,
   hit and predicate (P0416's two ordered `ActualAttachedFrameInputBeforeTail` hits and
   `Err(Output)` with `public_output_exact=false`, `native_difference=true`); the 20 Stage C
   cells and C10 remain unaffected; the pair, structural and negative bars are unchanged; a
   non-admission generator fixture that withholds the frame handoff on the selected row must
   yield the predicted NoHit with its raw evidence; and the resumed path must equal the
   queued path (rum against rum). The design must be general to the class, not a P0416 fix.
   Draft the staged implementation briefs (generator, render, re-freeze, epoch rebuild, 744
   restart) and the cost forecast from the measured samples.
6. **The zero-unknown statement.** The row table complete, every verdict backed by a line,
   the list of `stop` rows (ideally empty) with their exact missing facts, and the design's
   decision points for the director. The report is `rum-report/1` at
   `.temp/brief-756/report.json`; before finishing run `python3 scripts/report.py check 756`
   and fix every PROBLEM; the final message is only `python3 scripts/report.py summary 756`.

## Hard rules

- Never weaken a predicate, reclassify a row, exempt a `stop`, count a call edge as an
  authorization, count a text identity as a catch, or infer a runtime fact from the report.
  Quote every verdict line; a claim without `path:line` or a raw log line is not a verdict.
- The invariant is class-wide: a repair that closes P0416 alone recurs as the class. The
  design must cover every frame-gated own row and the five `FrameResumePassThrough` carriers;
  a row that genuinely cannot be carried by the design is a named stop with the design fact,
  never silently dropped.
- No source edit, no build, no binary execution, no new lane, no selection adjustment, no
  maintenance replay, no new epoch label, no workspace, no git, no network, no `unsafe`, no
  CM5. `.temp` only; never `/tmp`.
- The standing state is unchanged: bar 571/4,422, 37/134, 4/133, 0/26; the pair
  `(45, 5, false)` Open at zero admitted receipts; C2 118 Residual + 0 Lost; the 51 owners and
  five source duties Open; membership 28/3/7/113; `CHANGELOG.md` and S10 blocked; CM5
  unauthorized; everything sealed (c76–c99) sealed; 745/746 held.

## Verification

- The chain proof carries a `path:line` at every link and the exact predicate; the P0416
  failure is placed on one named link with the evidence that eliminates the other candidates.
- The class table covers all 151 own rows' frame-gated membership (82 + any correction with
  its derivation), the five `FrameResumePassThrough` carriers and the four structural
  controls, with zero unknown rows; every row verdict cites its evidence.
- The negative-reference audit is exhaustive (every writer/reader of the frame-token state)
  and either confirms or corrects the gate's static claim by name.
- The design's falsifier is bound to the accepted bars (the 18 own rows' historical
  signatures, the 20 Stage C cells, C10, the pair `(45,5,false)`, the four structural
  controls, the class-wide resumed-vs-queued equality) and the implementation briefs and
  cost forecast are complete.
- Budget: **3,000 s focused** (a fresh carve from the stage reserve,
  32,780.22727202312 → 29,780.22727202312 s), **0 s workspace, 0 s epoch**; unspent returns;
  a projected overrun stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-756/report.json` (cumulative with `.temp/brief-756/ledger.json`),
`changes` empty (every write is under `.temp/brief-756/`); sections for the chain, the class,
the negative-reference audit, the delta and carrier shapes, the design and falsifier, the
implementation briefs and the scope; cite lines exactly. STOP at the inventory and design: no
implementation, no 744 restart, no 745/746, no CM5.
