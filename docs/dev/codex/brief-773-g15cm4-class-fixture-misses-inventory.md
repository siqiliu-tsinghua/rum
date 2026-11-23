# Brief 773 — G15-CM4 design-only: the three class-fixture positive misses, zero-unknown inventory and repair decision

**Director-dispatched, design-only (`Scope: NONE`), one stage, one gate.** This continues
ruling 781. Brief 772 ran the whole class-fixture module once: the loader is green by use,
and three positive rows miss their receipts — `C5.default.profiled.finish`, `P0492`,
`P2299`. This brief produces the complete zero-unknown inventory of those three misses and
of the positive-assertion coverage, decides the repair, and drafts the implementation
brief. It writes nothing outside `.temp/brief-773/`.

## Goal

When this brief is done, in `.temp/brief-773/` only: a zero-unknown, evidence-cited
inventory of the three `FIXTURE POSITIVE_MISS` rows — the contract event, the observed
route, the exact mechanism of each miss, and the fix locus — plus the full 97-row
receipt-vs-contract alignment naming every row that passes on a receipt other than its
contract event; the decision on generic-frame matching semantics and on strengthening the
positive assertion; the candidate repairs with their falsifiers and blast radius on the
emission/pin/certificate chain; and a drafted implementation brief. No build, no test, no
binary execution, no edit of any fixture, contract, table or artifact.

## Context pointers

- Read `AGENTS.md`; rulings `.temp/director/ruling-781.md`, `ruling-780.md`,
  `ruling-779.md`; brief 772 with its Gate c131.
- **The red:** `.temp/brief-772/logs/class-lease-fixtures.log` (1,205,808 B) —
  `test result: FAILED. 193 passed; 3 failed; 0 ignored; 0 measured; 592 filtered out;
  finished in 22.05s`; `.temp/brief-772/logs/failures.log` names the three rows; the
  `assert!(committed)` is `class_lease_fixtures.rs:64`.
- **The contract records:** `.temp/brief-756/frame-authorization.tsv` (rows 2, 41, 84),
  `.temp/brief-756/row-verdicts.log`, `.temp/brief-756/class.md`,
  `.temp/brief-757/class-97.tsv`, and the 759 contract's per-row data the recipes came
  from.
- **The emitted artifacts (in `.temp/brief-772/scene/`):**
  `crates/rum-runtime/src/cm740_probe/class_lease.rs` (`kind_matches` at 117,
  `note_allocation`, `note_return_kind`, `note_consumer`, `note_typed_constructor`,
  `wants_input_return`), `class_rows.rs` (97 recipes: row/group/frame/cursor/carrier/
  phase/producer/consumer/program/source_sha256),
  `crates/rum-runtime/src/runtime/suspension/core.rs:600-745` (the three allocation
  hooks, kinds from `std::any::type_name::<F>()`), and the seams
  (`definitions/application/down_values/default_fallback.rs:60-95,230-270`;
  `runtime/eval_core/call.rs:100-175,1215-1290`;
  `crates/rum-stdlib/src/schema/continuation.rs`), plus
  `class_lease_fixtures.rs:12-23` (`positive_receipt`).
- **Known leads from my gate (confirm or refute, never assume):** P2299's frame is
  allocated as `rum_stdlib::schema::continuation::SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>`
  and the recipe says `SchemaValidationSuspensionFrame`; C5's InputRequired return seam is
  `default_fallback.rs:258-261`; P0492's expected `ProfiledPreparedDispatchFrame`
  (`call.rs:152`) appears zero times in the log; `positive_receipt` accepts any committed
  receipt (P0248 passes with only `EvalError::InputRequired` while its recipe-frame
  allocation never matches).

## Deliverables

1. **Entry re-authentication (read-only).** The 772 scene walks `12a687e2…`/5,816; the
   class table is `8c8b2232…` (4,803,954 B); the scene `provenance.tsv` is `965eca10…`;
   `class_lease_fixtures.rs` is `3738e894…`. STOP on any mismatch.
2. **Row-by-row inventory, zero unknown.** For each of `C5.default.profiled.finish`,
   `P0492`, `P2299`: the contract event from the 756/757/759 records (carrier, phase,
   frame object, construction/resume sites, the correctness note); the observed route
   from the 772 log (both halves, pauses, every `CLASS_*` line for the row, every
   allocated kind, every seam reached); the exact mechanism of the miss with `path:line`
   and log-line citations; the fix locus with the alternatives rejected on evidence.
3. **The full alignment.** All 97 rows: the contract event (group/frame/carrier), the
   receipts each positive test produced, and whether the pass is on the row's contract
   event or on another receipt. Name every masked row (the P0248 class) and every pass
   that is not on a contract-authorized receipt. Compute the complete list of recipes
   whose frame is a generic type instantiation and, for each, whether the emitted
   `type_name` can ever satisfy `kind_matches`' exact/`::`-suffix rule.
4. **The matching decision.** Propose the class-identity semantics for generic frames —
   base-name policy vs exact instantiation identity — including the `type_name` spelling
   normalisation each requires (`alloc::vec::Vec<usize>` vs `Vec<usize>`). State which
   preserves the class lease's authorization meaning and which weakens it; give each its
   falsifier.
5. **The assertion strengthening.** Redesign `positive_receipt` (or the per-row
   expectation) so a pass proves the row's own contract event (e.g., the expected
   kind/group/issuer pinned from the recipe), with falsifiers that must be watched
   rejecting: a receipt of the wrong kind, a missing consume, a withheld-mode hit, and a
   pass on a neighbouring frame.
6. **The repair options with blast radius.** What changes in `class_rows.rs`, 
   `class_lease.rs`, the seams, or the fixture; the effect on the 27-file emission, the
   147-row overlays, the five additions (the class table among them), the `source_schema`
   cells, the 152-chain certificate and the KATs; the regeneration path; and what the
   implementation brief must do. If the correct fix is a contract-data correction, name
   the record that must change and the evidence that justifies it.
7. **The drafted implementation brief** in `TEMPLATE.md` shape (one brief, one stage),
   with its own budget proposal and stop lines.

## Hard rules

- **Scope: NONE.** Work under `.temp/brief-773/` only; `.temp/brief-772/`, every accepted
  scene and the live tree are read-only; no cargo, no build, no test, no binary execution;
  no edit of any fixture, contract, table or emitted artifact; no git; no network; no new
  dependency; `.temp` only, never `/tmp`; never read `~/.config/ccds/api-key`.
- **A reading is not a verdict.** Every claim cites a log line, a table row or
  `path:line`. No row may be left unexplained; if the evidence is insufficient, name the
  exact additional read-only observation needed and stop there.
- Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

**focused 900 s + workspace 0 s + epoch 0**, carved from the stage reserve
(34,556.96378847017 → **33,656.96378847017 s**); analysis is Python and reads only, no
cargo. Unspent returns; a projected overrun stops for a director ruling — more is carved,
never a scope cut.

## Report format

`rum-report/1` JSON at `.temp/brief-773/report.json` via `scripts/report.py`, sections:
entry, inventory (one per row), alignment, decision, strengthening, repair options,
drafted brief, scope, budget; `scripts/report.py check 773` green. The summary states the
three mechanisms in one line each, the masked-row count and the decision.
