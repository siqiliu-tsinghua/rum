# Brief 749 — CM4 carrier-dispatch repair, Stage A: the qualified render and the new schema pins

**Director-dispatched, one stage, one gate.** This answers `Q748-dispatch-reachability`: the
accepted final epoch is not campaign-ready because 56 of 151 own rows and all six pair/structural
dependencies are unreachable on their own paths, and the trusted generators never emitted the
callers. Stage A rebuilds the generator in a new, authorized scene from the accepted 748 repair
plan, renders and proves the repaired copied images, and re-pins the schema — **the epoch build,
the twelve blocks, the six workspace comparisons, Stage B, Stage C and the 744 restart are not
this stage.** No live-repo source change. The repair is only finished when Stage B and Stage C
prove arms and ordered hits on the new epoch; a text or byte check is never a catch.

**Attempt 2.** Attempt 1 (unit `brief-749-cdx-20261122-045113-82310`, report retained
verbatim at `.temp/brief-749/attempt-1-report.json`) stopped at cut 1 under the STOP rule with
`Q749-retained-callers`: the rule-path caller at `rule.rs:111` cannot serve P0001/P0002 because
their frozen selections execute the fixture callbacks at `native.rs:143` and `:185`, and the
rule path's prefix is always zero-capacity while P0002's producer needs a positive one. Ruling
`Q749-retained-callers` (`.temp/director/ruling-749.md`) accepted the STOP and amended the plan:
both scheduled fixture callers become cuts, `native.rs` moves from preserved to re-rendered, the
`rule.rs:111` cut stays, and a scheduled-caller audit is added for all 62 affected rows.

**Attempt 3.** Attempt 2 (unit `brief-749-cdx-20261122-051446-92453`; its cumulative
report is preserved verbatim at `.temp/brief-749/attempt-2-report.json`) stopped under the
STOP rule with
`Q749-P0339-frozen-hit`: its 49-cut render and caller binding were correct, but P0339's frozen
cover could not reach its credited hit. Rulings `Q749-P0339-frozen-hit`,
`Q750-P0338-resumed-normalizer`, `Q751-P1142-initial-edge` and `Q752-hit-side-edge-inventory`
(`.temp/director/ruling-750/751/752/753.md`) turned that STOP into the completed hit-side-edge
and value inventory of all 151 own rows — 135 static-witness, 6 cover-candidate (revised live
by brief 753), 10 hit-side-edge-fix, zero unknown — and accepted nine cuts beyond the 49.
Attempt 3 rendered the 58 cuts over the then-derived 143-file / 286-postimage table; the
amendment after attempt 2 (in force) is below.

**Attempt 4.** Attempt 4 (unit `brief-749-cdx-20261122-090319-193939`, 113.41397256198616 s
focused; its cumulative report is preserved verbatim at
`.temp/brief-749/attempt-4-report.json` as the first step below) reproduced the accepted
parent byte-exactly (135 rows / 270 postimages, 66/66 protected records, cut 37 bound in both
copies) and stopped under the STOP rule with `Q749-P2600-converter-visibility`: cut 38's driven
success converter `eval_success_from_runtime_cm740_driven` (`outcome.rs:414`, `pub(crate)`
inside the private `model::outcome` module) cannot be named from the scheduled caller at
`session.rs:549`; Rust 1.85 rejected every spelling in both arms and both feature modes
(E0425/E0603) and the host library reproduced E0603. Ruling `Q749-P2600-converter-visibility`
(`.temp/director/ruling-756.md`) authorizes the single-item crate-private re-export in
`model/mod.rs` and recomputes the partition to **144 files / 288 postimages = 98 preserved +
37 re-rendered + 9 new**; the amendment after attempt 3 (in force for it) is below.

**Attempt 5.** Attempt 5 (unit `brief-749-cdx-20261122-101234-230587`, 67.01911845699942 s
focused; its cumulative report is preserved verbatim at
`.temp/brief-749/attempt-5-report.json` as the first step below) passed the nameability
preflight on all 58 cuts (116 bindings, zero unresolved; cut 38 through the authorized row-2
re-export) and reproduced the accepted parent byte-exactly (135 rows / 270 postimages, the
98+37+9 partition, 66/66 protected records), then stopped under the STOP rule with
`Q749-P0002-canonical-else`: cut 49's sealed two-line `cx` / `.evaluate` ordinary call (49
bytes) cannot survive the standing canonical emission at the rendered else arm — rustfmt's
canonical spelling is 36 bytes, restoring only the 13 removed bytes produces exactly one
formatter diff per copy, and the typed call is unchanged. Ruling `Q749-P0002-canonical-else`
(`.temp/director/ruling-757.md`) authorizes the canonical spelling at that one call; the
amendment after attempt 5 (in force) is below.

**This is attempt 6.** It renders the same **58** cuts over the same **144-file /
288-postimage** table, with cut 49's ordinary arm in the authorized canonical spelling; the
amendment after attempt 5 (in force) is below.

## Goal

When this brief is done, in `.temp/brief-749/attempt-6/` only (plus the preserved
attempt-1, attempt-2, attempt-3, attempt-4 and attempt-5 reports and the cumulative report at
the parent paths): every one of the **58** cuts — the 748 plan's 47 (with cuts 15 and 17 relocated to their credited callers
by `Q749-protected-profile-twins`), the two fixture caller cuts at `native.rs:143` and `:185`,
and the nine c103 cuts (five carried, four further initial calls) — is rendered at its
anchored original boundary into the two copied images, bound by typed symbol, receiver,
signature and preimage rather than text position; the nine new copied paths are added; **for
every one of the 62 affected rows the call site its own frozen selection executes reaches the
driven producer and the hit** (the scheduled-caller audit), with each row's c103 witness or
edge status recorded, and all 151 own rows live on their own selection paths; the six
dependencies' dispatch edges live on their revised selected programs; the four structural
controls have their authenticated source/frame handoff installed; the 95 already-working rows'
images and the 13 PortBlocked bindings are preserved; the disabled and armed images render
twice byte-identically, invert every recipe, and yield a **144-file / 288-postimage** overlay
table (98 preserved + 37 re-rendered + 9 new) with full postimages and a new `source_schema`
on all 151 mutant records, recomputed from the exact bytes; the identity inputs are re-pinned;
and the source gate is green in both feature modes. No epoch label, no block, no workspace
comparison, no class admission, no live-tree edit and no new catch is claimed here.

## Amendment after attempt 1 (in force; item 3 superseded by the c104 amendment)

1. **The two fixture caller cuts.** `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/native.rs`
   is sealed overlay row 135 (preimage `83d0f6a8…`, disabled = armed `12fa73b2…`); it moves from
   preserved to re-rendered. At `native.rs:143` (`callback_charge`, rows P0001) and `:185`
   (`retained_prefix`, rows P0002) render the standing qualified shape
   `if rum_runtime::cm740_probe::driven() { cx.evaluate_cm740_driven(<same prefix>, <same
   argument>) } else { <the original call, textually unchanged> }`, preserving the argument
   expressions, their evaluation order, the receivers and both original `evaluate` bodies. The
   pre-existing lane `cm740_probe::begin/end` hooks must be reproduced byte-identically before the
   two dispatchers are added, and the whole recipe must invert to the live preimage.
2. **The `rule.rs:111` cut stays** (it is a real caller of the same driven function). Record the
   full caller census of the two-argument `RetentionContext::evaluate` in the sealed armed tree —
   `rule.rs:111`, `native.rs:143/185`, and the five `crates/rum-runtime/src/runtime/accounting_mode/tests/*`
   calls (`tests/rule.rs:223`, `tests/mod.rs:194`, `tests/exits.rs:28/141/174`) — and state that
   the last five are `cfg(test)` unit-test code of rum-runtime outside the oracle lane build. The
   32 one-argument `Session::evaluate` calls are a different method and off this boundary.
3. **Partition (superseded by the c104 amendment, item 2).** The attempt-2 forecast of 137
   overlay rows / 274 postimages is replaced by the c103 table: 143 files / 286 postimages,
   58 cuts; the new `source_schema` = sha256 of the exact ordered rendered `overlays.tsv`
   bytes; the policy digest `3953aa9f…` unchanged. The affected rows stay 62 (56 own + 6
   dependencies).
4. **The scheduled-caller audit (new deliverable).** For every affected row derive from its frozen
   selection program, through the lane harness registrations (`guest.rs`/`native.rs` callbacks and
   the builtin registrations), the call site actually executed on the way to the producer and the
   hit, and show the repaired dispatch binds that call site. The 748 static `reachable` verdict is
   not a catch on its own — a driven clone reachable from an unrelated caller proves nothing about
   the frozen selection (attempt 1's STOP is the proof). Record the audit as a table: row,
   program, executed caller `file:line`, dispatched target, producer site, hit site, and the
   excluded call sites with the reason each cannot execute a frozen selection. Any row whose
   scheduled caller cannot be bound under the standing constraints stops with the exact fact —
   never a row exception, never a text-position workaround, never a moved hit.

## Amendment after attempt 2 (in force, c104)

1. **The nine additional cuts.** Ruling `Q752-hit-side-edge-inventory` accepts these as the
   completed class of hit-side incoming edges on the credited execution; each is rendered in
   the standing qualified shape at its byte-pinned anchor in the sealed whole-file image
   (offsets are the sealed images', never the live preimages'), with its driven twin and its
   three negatives (incoming-edge withholding, else-only NoHit, value):
   - P0338 — `definition_catalog/rewrite_rules.rs:450` (overlay row 25, typed-frame resume;
     hit `:315` in `normalize_rewrite_rule_exprs_from_resolved_cm740_driven` `:306`).
   - P0435 — `definitions/part_assignment/resume_frames.rs:24` (new file, typed-frame resume;
     hit `part_assignment.rs:288`).
   - P1070 — `kernel_data/structure_data/replace/guarded/resume_frames.rs:110` (overlay row
     67, continuation resume; hit `guarded.rs:617`).
   - P1818 — `rum-stdlib/src/associations/mutation/resume_frames.rs:29` (new file, typed-frame
     resume; hit `mutation.rs:920`).
   - P1142 — `functional_ops/arrays/mod.rs:219` (overlay row 116, already re-rendered, initial
     call; hit `options/core.rs:196`).
   - P1147 — `namespace_tools/names/options.rs:55` (new file, initial call; hit
     `options/core.rs:357` in the driven reader `:320`).
   - C3.evaluate_call.unstable_head — `predicates/truth.rs:138` (new file, initial call;
     `evaluate_cm740_driven` `resolution.rs:334`, hit `around_hit` `call.rs:225`).
   - P0485 — `functional_ops/callable/head_ops.rs:29` (new file, initial call; hit
     `around_ok` `call.rs:282`).
   - P0509/P0287 — `kernel_data/control_data/assertions.rs:11` (new file, initial call, one
     cut serving both rows; P0509 hit `around_ok_profile_return` `eval_core/mod.rs:280`,
     P0287 hit the stored RHS frame `rhs.rs:310`).
   Each dispatcher is `if driven() { <driven twin> } else { <the original call, textually
   unchanged> }`; receivers, arguments and evaluation order are preserved; the preimage
   inversion must hold for every one of the nine.
2. **The derived partition (a forecast, not a benchmark; superseded by the c105 amendment,
   item 2).** Parent 135 files = 100 preserved + 35 re-rendered; plus 8 new files = 143 files
   / 286 postimages; 58 cuts. The 35 re-rendered are the 33 accepted 749 rows plus rows 25
   (`rewrite_rules.rs`) and 67 (`guarded/resume_frames.rs`); row 116 (`arrays/mod.rs`) was
   already re-rendered and gains the P1142 cut. The 8 new files are
   `accounting_mode/rule.rs` and `rewrite_data/replace/all.rs` (748),
   `definitions/part_assignment/resume_frames.rs` and
   `associations/mutation/resume_frames.rs` (751), and
   `kernel_data/control_data/assertions.rs`, `namespace_tools/names/options.rs`,
   `functional_ops/callable/head_ops.rs` and `predicates/truth.rs` (c103). The c105
   amendment moves `builtin_adapters.rs` to re-rendered and adds `control_data/mod.rs` as a
   ninth new file: **144 files / 288 postimages = 99 preserved + 36 re-rendered + 9 new**.
   The render must recompute the table and the `source_schema` = sha256 of the exact ordered
   rendered `overlays.tsv` bytes from the bytes it produces; these figures bind only after it
   does.
3. **The revised covers.** The live `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutant-coverage.tsv`
   now carries the six brief-753 revisions (P0249, P0339, P0350, P0351, P1155, P1610). The
   scheduled-caller audit and every selection join use the revised live table; the c1/c2/c7
   scenes are comparison records, never edited.
4. **The audit.** The 62 affected rows (56 own + 6 dependencies) each carry their c103 verdict
   class — `static-witness` with the credited edge kind, or `hit-side-edge-fix` with the
   cut(s) that bind it — in addition to the executed caller, dispatched target, producer, hit
   and every excluded call site with its reason. P0509/P0287 share the one `assertions.rs:11`
   cut. Zero rows may be settled by argument.
5. **STOP rules.** Any of the nine cuts that cannot be expressed at its anchored boundary, any
   affected row whose scheduled caller cannot be bound, or any divergence of the recomputed
   bytes from a byte-pinned anchor stops with the exact fact — never a row exception, never a
   text-position workaround, never a moved hit. Nothing is reclassified, promoted, exempted or
   settled; a static witness or edge fix is not a catch.
6. **Scene and budget (superseded by the c105 amendment, item 5).** New work under
   `.temp/brief-749/attempt-3/`; copy the then-current `.temp/brief-749/report.json` verbatim
   to `.temp/brief-749/attempt-2-report.json`, and keep attempt 2's scene read-only. The
   in-force scene and budget are in the c105 amendment.

## Amendment after attempt 3 (c105; attempt 4 ran under this; items 2 and 5 superseded by c106)

1. **Cuts 15 and 17 are relocated to their credited incoming call sites.** The ordinary
   `Session::eval_catch` (`crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:80`,
   span 1,358 B, sha256 `fc003b03…`) and `Session::eval_module`
   (`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:15`, span 2,585 B, sha256
   `6d480445…`) are the unique protected peers of the profile-return driven twins; they must
   stay byte-identical in both arms and both images, and no dispatcher may be placed inside
   them. The cuts move to the call sites the c103 inventory credits:
   - **Cut 15** — `crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:75` (rows
     P0812, P0813, P0814), the sealed line
     `            ControlDataBuiltin::Catch => self.eval_catch(args),`. Render
     `if crate::cm740_probe::driven() { self.eval_catch_cm740_driven(args) } else {
     self.eval_catch(args) }` at the arm, with the original call `self.eval_catch(args)`
     textually unchanged in the else arm and the same `args`.
   - **Cut 17** — `crates/rum-runtime/src/runtime/builtin_adapters.rs:405` (row P1507), inside
     `builtin_module` after `let args = args.as_ref();`, the sealed line
     `    session.eval_module(args)`. Render
     `if crate::cm740_probe::driven() { session.eval_module_cm740_driven(args) } else {
     session.eval_module(args) }`, with `session.eval_module(args)` textually unchanged in the
     else arm and the same narrowed `args`.
   The caller census is exactly three calls — these two production sites and one `cfg(test)`
   call at `profiling/tests/control.rs:1134` — and the driven twins are
   `pub(in crate::runtime)` with the same `&mut self, args: &[Expr]` shape. The three
   negatives are restated per relocated cut: the incoming-edge withholding (suppress the
   driven arm, keep the whole ordinary call) makes the re-scan report the row unreachable; the
   else-only dispatch is NoHit even with the positive value; the value condition is unchanged.
   The `eval_catch` and `eval_module` spans, the 66 protected records and the parent's
   135 rows / 270 postimages must stay green.
2. **The partition recomputes.** `builtin_adapters.rs` is parent row 8 (live `0be72928…`,
   disabled = armed `f8ca28fc…`); the cut moves it from preserved to re-rendered.
   `control_data/mod.rs` is absent from the 135-row overlays and byte-identical in the live,
   real, disabled and armed trees (`21056f70…`); the cut adds it as a ninth new overlay file
   pinned to its live bytes. The forecast is **144 files / 288 postimages = 99 preserved + 36
   re-rendered + 9 new; 58 cuts unchanged (two relocated, not added)**. The render must
   recompute the table, the counts and the `source_schema` = sha256 of the exact ordered
   rendered `overlays.tsv` bytes from the bytes it produces; these figures are forecasts until
   it does.
3. **No row is reclassified or moved.** The credited edge for each of the four rows is the same
   call site; no predicate weakens, no hit moves, no program, mutant, policy or selection
   changes. The two relocated cuts are the same two cuts, not new ones.
4. **The audit carries the relocation.** The four rows show their credited caller and the
   relocated cut; the ten c103 edge-fix rows keep their nine cuts; the 62-row audit and the
   151-row re-scan are otherwise unchanged.
5. **Scene and budget.** New work under `.temp/brief-749/attempt-4/`; before writing the new
   cumulative, copy the current cumulative `.temp/brief-749/report.json` verbatim to
   `.temp/brief-749/attempt-3-report.json`, and keep attempt 3's scene read-only. Budget:
   4,559.094900708002 s focused + 5,577 s workspace, 0 epoch.
6. **STOP rules stay.** Any further cut that cannot be expressed at its anchored boundary, any
   affected row whose scheduled caller cannot be bound, or any divergence of the recomputed
   bytes from a byte-pinned anchor stops with the exact fact — never a row exception, never a
   text-position workaround, never a moved hit.

## Amendment after attempt 4 (c106; attempt 5 ran under this; item 5 superseded by c107)

1. **Cut 38's converter is nameable through one crate-private re-export.** In
   `crates/rum-host/src/model/mod.rs` (752 partition row 2, sealed bytes `d955b703…`, 1,000 B,
   preserved in both arms), append exactly one line after the sealed final line
   `pub(crate) use outcome::cm740_output_scope_cm740_driven as cm740_output_scope;`:
   `pub(crate) use outcome::eval_success_from_runtime_cm740_driven;` (with its LF). The
   postimage is `f118494a447d447059f188ccf8b57a378a5a6b2b1c64be128be6a41f441f628e` (1,064 B)
   in both the disabled and the armed image; the live file (`ec87f29a…`) is untouched, and no
   other byte moves. The appended file is rustfmt-clean under edition 2024. The cut at
   `session.rs:549` keeps its rendered form
   `crate::model::eval_success_from_runtime_cm740_driven(response)` in the driven arm and
   `eval_success_from_runtime(response)` textually unchanged in the else arm.
2. **The partition recomputes.** Row 2 moves preserved → re-rendered. The forecast is **144
   files / 288 postimages = 98 preserved + 37 re-rendered + 9 new; 58 cuts unchanged**. The
   270 parent postimages, the 66 protected records, the 12 exact peer assertions per arm and
   the c103 anchor cells must stay green. The render recomputes the table, the counts and
   `source_schema` = sha256 of the exact ordered rendered `overlays.tsv` bytes from the bytes
   it produces; row 2's new digests appear in the rendered table and the report.
3. **No row is reclassified or moved.** The P2600 credited edge, value and hit are unchanged;
   no predicate weakens; no hit moves; no program, mutant, policy or selection changes.
4. **A nameability preflight precedes the render.** For every one of the 58 cuts, resolve every
   identifier its dispatcher introduces from the dispatcher's module against the frozen parent
   tree — inherent method, same-module item, imported item, and `crate::` path with module
   privacy and re-export lists — and record the resolution per cut in the scene. Any other
   unnameable reference stops with the exact fact; no further re-export is pre-authorized; no
   text-position workaround and no moved hit.
5. **Scene and budget.** New work under `.temp/brief-749/attempt-5/`; before writing the new
   cumulative, copy the current cumulative `.temp/brief-749/report.json` verbatim to
   `.temp/brief-749/attempt-4-report.json`, and keep attempt 4's scene read-only. Budget:
   attempts 1–4 charged 185.3190718539845 s focused; the remaining carve is
   4,445.6809281460155 s focused + 5,577 s workspace, 0 epoch.
6. **STOP rules stay.** Any further cut that cannot be expressed at its anchored boundary, any
   affected row whose scheduled caller cannot be bound, any unnameable identifier in the
   preflight, or any divergence of the recomputed bytes from a byte-pinned anchor stops with
   the exact fact — never a row exception, never a text-position workaround, never a moved
   hit.

## Amendment after attempt 5 (in force, c107)

1. **Cut 49's ordinary arm is the same typed call in canonical spelling.** The words "the
   original call, textually unchanged" in the amendment after attempt 1 (c102 item 1) mean the
   same typed invocation — receiver `cx`, method `evaluate`, prefix `values`, argument
   `args[0].clone()`, argument evaluation order and the following
   `.and_then(CallbackValue::adopt)` continuation — not the sealed parent's line breaks. At
   the rendered else arm the standing canonical emitter emits exactly
   `cx.evaluate(values, args[0].clone())` (36 bytes); the sealed parent's two-line 49-byte
   spelling (`cx`, LF, twelve spaces, `.evaluate(values, args[0].clone())`) is not canonical
   in that position and is not required. The driven arm stays
   `cx.evaluate_cm740_driven(values, args[0].clone())`. No `#[rustfmt::skip]`, no local
   format exemption, no moved continuation and no other text-position workaround.
2. **The recipe records the normalization and still inverts.** The cut-49 recipe must record
   the 13-byte whitespace change (49 → 36 bytes) as part of its image and invert byte-exactly
   to the live preimage through the accepted parent recipe; the fixture row-135 preimage
   `83d0f6a8…`, its sealed parent images `12fa73b2…`, the 66 protected records and the 12
   exact peer assertions per arm stay green. P0002's frozen program, tuple, witness, hit and
   value are unchanged; no row is reclassified or moved; the typed syntax digest of the
   literal, canonical and restored else forms is one value (`f7783925…`; attempt-5 evidence).
3. **The check bar changes accordingly.** The cut-49 acceptance bar is: the rendered else arm
   contains the exact 36-byte canonical invocation above; the file passes the standing
   `cargo fmt --check` in both feature modes; the typed call is unchanged against the sealed
   parent; the recipe's inversion reaches the live preimage; and the source gate compiles both
   modes. The attempt-5 `a5-cut49` bar ("the sealed 49-byte invocation is present in a
   canonical postimage") is superseded — it cannot be satisfied by any canonical image.
4. **The preflight stands.** Attempt 5 resolved all 116 bindings (58 cuts × 2 arms) with zero
   unresolved, including cut 38 through the authorized row-2 re-export and the six structural
   cuts through their identifier bindings. Attempt 6 may reuse that accepted preflight; the
   cut-49 change introduces no identifier. No further re-export is pre-authorized.
5. **Scene and budget.** New work under `.temp/brief-749/attempt-6/`; before writing the new
   cumulative, copy the current cumulative `.temp/brief-749/report.json` verbatim to
   `.temp/brief-749/attempt-5-report.json`, and keep attempt 5's scene read-only. Budget:
   attempts 1–5 charged 252.33819031098392 s focused; the remaining carve is
   4,378.661809689016 s focused + 5,577 s workspace, 0 epoch.
6. **STOP rules stay.** Any further cut that cannot be expressed at its anchored boundary, any
   affected row whose scheduled caller cannot be bound, any unnameable identifier, or any
   divergence of the recomputed bytes from a byte-pinned anchor stops with the exact fact —
   never a row exception, never a text-position workaround, never a moved hit. The canonical
   authorization in item 1 is the only whitespace normalization permitted.

## Context pointers

- Read `AGENTS.md`; `.temp/director/ruling-749.md`; `.temp/brief-749/attempt-1-report.json` and
  the attempt-1 scene's `logs/` and `evidence/` (read-only); `.temp/brief-749/attempt-2-report.json`
  and the attempt-2 scene (read-only); `.temp/brief-749/attempt-3-report.json` and the
  attempt-3 scene (read-only), especially `evidence/protected-dispatch-conflict.json`,
  `evidence/four-selected-callers.json` and the `verify_render.py:46` replay;
  `.temp/brief-749/attempt-4-report.json` and the attempt-4 scene (read-only), especially
  `evidence/p2600-source-binding.json`, `evidence/compiler-diagnostics.json`,
  `evidence/cut-disposition.tsv` and `evidence/partition-inputs.json`;
  `.temp/brief-749/attempt-5-report.json` and the attempt-5 scene (read-only), especially
  `evidence/nameability.json`, `evidence/parent-render.json`,
  `evidence/cut-disposition.tsv`, `evidence/formatter-controls.json` and
  `evidence/partition-at-stop.json`; rulings
  `.temp/director/ruling-754.md`, `ruling-755.md`, `ruling-756.md` and `ruling-757.md`;
  `.temp/director/ruling-748.md`;
  brief 748 (`docs/dev/codex/brief-748-g15cm4-carrier-dispatch-reachability.md`) and its scene in
  full, especially `repair-sites.tsv` (47 cuts: file, anchor, destination, rows, operation),
  `repair-plan.json` (per-cut design, per-row positive and negative, schema fields, preservation),
  `rows.json`, `definitions.json`, `dependencies.json`, `calibration.json`, `root-cause.json` and
  `forecast.json`. The 748 verdicts are the specification; re-derive, never re-litigate them.
- The c103 inventory (read-only): `.temp/brief-752/evidence/added-cuts.json` (the nine cuts with
  anchor, producer, hit, hit-enclosing function, value and the three negatives),
  `file-partition.tsv` (the 143-file table), `hit-side-edges-and-values.tsv` (the 151-row
  verdicts), `dependency-plans.json`, `revision-and-forecast.json` (the six live revisions and
  their successor records); rulings `.temp/director/ruling-752.md`, `ruling-753.md`,
  `ruling-754.md`; the live revised `mutant-coverage.tsv`.
- The trusted generator line (read-only): `.temp/brief-742/continuation-88/fn_split.py`
  (`4971a553…`), `.temp/brief-743/stage-a/tools/fn_split.py` and
  `stage-b/render_tools/fn_split.py` (same digest), `stage-a/tools/route_a.py` (`d2a77b92…`),
  `stage-a/compose.py` (`e20c1bb1…`), `stage-b/composition.py` (`1c72640e…`), `hot_compare.py`
  (`dbb4a308…`), `freeze_b.py` (`2778e881…`), `blocks.py` (`711d62d8…`). Copy them into the new
  scene before editing; nothing under the earlier scenes may be written.
- The accepted line: 743's Stage A/B/C records and rulings; the sealed epoch under
  `.temp/brief-743/stage-b/epoch-b/` with the three trees, identity `b1e68590…`, the 135-row
  `overlays.tsv` (sha256 = `22751919dbcb…` = the current `source_schema`; row 135 is
  `native.rs`, row 7 is `bridge.rs`), the 151-row `mutants.tsv`, `observer-policy.tsv`
  (`3953aa9f0b03…`), the selections and the 509 retired blocks;
  `.temp/brief-743/stage-b/hot-bodies.json` (the protected hot bodies; `bridge.rs::evaluate` is
  `byte_exact`). **Read-only.**
- The 747 ownership inventory (the 151-row matrix; the thirteen named PortBlocked rows; Q24/Q25);
  the 744 attempt-2 evidence for P0416 and C10; brief 623's Campaign invariants.

## Scope (Stage A only)

- MAY create/modify: everything under `.temp/brief-749/attempt-6/` for new work; the report
  paths `.temp/brief-749/attempt-5-report.json` (a verbatim copy of the current cumulative
  `report.json`) and `.temp/brief-749/report.json` (the new cumulative report). Everything else
  under `.temp/brief-749/` — attempt 1's files, the attempt-2, attempt-3, attempt-4 and
  attempt-5 scenes — is a read-only record. Put the renderer, the copies of the generator
  scripts, the rendered trees, the re-derived tables and every scratch file under `attempt-6/`.
- MAY read: the live source tree, the sealed scenes, the earlier `.temp/brief-*` records.
- MUST NOT: edit the live source tree (0 tracked changes); write under any sealed scene; edit the
  live 135-row `overlays.tsv`, the live `mutants.tsv`, the policy, the schema, the manifest or
  the selections; run the epoch binaries; create an epoch label; build the twelve blocks; run the
  six workspace comparisons, Stage B or Stage C; commit or use git; touch the network; add a
  dependency or `unsafe`. Builds may run only on the rendered copies under
  `.temp/brief-749/attempt-6/`, never on the live tree or a sealed tree.

## Deliverable

1. **The qualified render of all 58 cuts.** For every cut, implement the stated operation at
   its anchored original boundary: a signature-preserving dispatcher that delegates to the
   existing driven clone when driven and to the original body otherwise, the qualified
   function-value and receiver routes, the retention-caller branch (the two fixture call sites
   above, plus `rule.rs:111`), the continuation/resume joins, the nine c103 initial-call and
   resume-edge cuts of the amendment, and the structural control proof. Add the nine new file
   paths to the copied set. Render the disabled and armed images twice; the two renders must be
   byte-identical. The real image stays the unchanged baseline. The first render step must
   reproduce the accepted images of the preserved files byte-identically before any cut is
   applied.
2. **The reachability re-scan.** Apply the 748 scanning method to the rendered images: all 151
   own rows must come out `reachable` (0 `producer-unreachable`, 0 `hit-site-unreachable`,
   0 `unknown`), the six dependencies' dispatch edges live on their selected programs, and the
   247-definition map complete with the new definitions mapped or named. The 13 Q25 rows stay
   PortBlocked with their named missing bindings. Calibrate the method on P0416 (now reachable
   through the original `eval_rule` boundary), C10 (unchanged), P0001 and P0002 (the two fixture
   callers), and one representative per repair pattern — original-boundary dispatcher, retention
   caller, function-value/registration route, continuation resume, structural control.
3. **The overlay table and the schema.** Re-derive the sorted table from the rendered bytes —
   the forecast is 144 files / 288 postimages (98 preserved + 37 re-rendered + 9 new), and
   the render must publish the counts it actually produced — with file, preimage, disabled and
   armed sha256 and the full payloads; invert every recipe to its input; verify the untouched
   files byte-identical; the new `source_schema` is sha256 of the exact ordered rendered
   `overlays.tsv` bytes; mechanically replace only that column on all 151 mutant records and
   verify the other seven columns against the seal; keep `observer-policy.tsv` byte-digest
   `3953aa9f0b03…`.
4. **The source gate.** On the rendered copies, both feature modes: format, the source-stage
   checks and the workspace gate the accepted Stage A ran (same commands, the same declared
   skips and composition differences), the protected hot bodies byte-identical, and the
   inversion. Disclose every difference; no acceptance suite and no new catch.
5. **The new pins.** A Stage A identity record: the new schema digest, the rendered table hash
   (all 144 files), both image tree hashes, the per-cut rendered site (original symbol, anchor,
   destination, rendered `path:line`) for all 58 cuts, the nine new files, the control-proof
   adapter, the preserved files and the unchanged policy. A later Stage B rebinds these by item
   identity when it freezes.
6. **The scheduled-caller audit and the falsifiers.** (a) The audit table required by the
   amendments, each row carrying its c103 verdict class and every excluded call site
   justified; the ten edge-fix rows show the cut(s) that bind their credited edge, and
   P0509/P0287 show the shared `assertions.rs:11` cut. (b) P0416: the rendered original
   `eval_rule` is a dispatcher whose driven arm reaches `eval_rule_cm740_driven` and its
   `around("P0416")`; C10's chain unchanged. (c) P0001/P0002: the rendered fixture call sites
   dispatch to `evaluate_cm740_driven` with the same prefix and argument, P0001 through the
   zero-capacity branch and P0002 through the positive-capacity branch, and the withholding
   fixture (the incoming edge suppressed, the whole frozen body kept) makes the re-scan report
   the row unreachable. (d) The withholding fixture for the nine c103 cut patterns (the
   incoming edge suppressed, the whole frozen body kept) plus the two retained caller edges;
   the live arm/Unarmed/NoHit proof belongs to Stage B/C and must not be inferred from bytes. (e) The 95 working rows: each rendered file outside the affected
   overlay files byte-identical; within an affected file, every unaffected rendered row
   preserved.
7. **The forecast inputs.** Record the measured Stage A cost from receipts (monotonic), the
   rendered bytes, and anything that changes the 748 forecast; no budget work beyond the carve.

## Hard rules

- `Scope` above. No live-repo edit, no git, no network, no new dependency, no `unsafe`, no
  epoch label, no block, no workspace comparison, no Stage B/C, no acceptance suite.
- The standing C2/pair/owner state stays untouched: pair `(45,5,false)` Open at zero admitted
  receipts; C2 118 Residual + 0 Lost; the 51 owners and five source duties Open; membership
  28/3/7/113; `CHANGELOG.md` and S10 blocked; CM5 unauthorized; everything sealed (c76–c99)
  stays sealed and read-only.
- Never weaken a predicate, never reclassify a row, never exempt a dead row, never move a hit to
  a favorable branch, never count a text or byte identity as reachability or as a catch. The 748
  static `reachable` verdict alone is not a catch for a row's own selection.
- If a cut cannot be expressed at its original boundary under the standing constraints, or a
  row's scheduled caller cannot be bound, STOP with the exact missing fact — never a row
  exception and never a text-position workaround.
- Paste verbatim evidence for every verdict (file:line, rendered text, measured count). Save
  every command's full output under `.temp/brief-749/attempt-6/` before parsing.
- Standings: replay the source-stage inventories on the rendered copy before the source gate;
  a red caused only by a stale fingerprint or position cell is an ordering artifact, reported
  as such, never weakened or hand-edited.

## Verification

- The 58 cuts rendered and rebound — the 49 accepted plus the nine c103 cuts at their pinned
  anchors; render-twice byte identity; the rendered table complete and inverted (forecast: 144
  files / 288 postimages; the measured counts must equal what the render produced); the new
  schema = sha256(rendered `overlays.tsv`); only `source_schema` replaced on the 151 mutant
  rows; the policy digest unchanged; the real image byte-identical; the untouched files
  byte-identical; the 95 working rows preserved; the 13 PortBlocked unchanged.
- The nameability preflight: every identifier every dispatcher introduces resolved from its
  module against the frozen parent tree, recorded per cut; zero unresolved.
- The re-scan: 151 own rows, 0 unreachable, 0 unknown; the six dependencies' dispatch edges
  live on their revised selected programs; the 247-definition map complete; the
  P0416/C10/P0001/P0002 and per-pattern calibrations; the withholding fixtures fail as
  required.
- The scheduled-caller audit: 62 affected rows, each carrying its c103 witness/edge status,
  every executed caller bound, the ten edge-fix rows' cuts named, every excluded call site
  justified, zero rows settled by argument.
- The source gate green in both modes with declared differences only; the hot bodies
  byte-identical; the Stage A identity record complete.
- Cut 49: the rendered ordinary arm is exactly `cx.evaluate(values, args[0].clone())` (36
  bytes), the driven arm is `cx.evaluate_cm740_driven(values, args[0].clone())`, the typed call
  equals the sealed parent's (same receiver, method, prefix, argument and order, with the
  `.and_then(CallbackValue::adopt)` continuation unchanged), the recipe records the 49→36
  normalization and inverts byte-exactly to the live preimage, and the standing format gate is
  green in both feature modes.
- Budget: attempts 1–5 charged 7.5967964620012935 + 29.80946500299069 + 34.498837827006355 +
  113.41397256198616 + 67.01911845699942 = **252.33819031098392 s** focused; the remaining
  carve is **4,378.661809689016 s focused + 5,577 s workspace, 0 epoch**; every bounded command
  debits by monotonic elapsed time; a projected overrun stops for a director ruling, never a
  scope cut.

## Report format

`rum-report/1` at `.temp/brief-749/attempt-6/report.json`; first copy the current cumulative
`.temp/brief-749/report.json` verbatim to `.temp/brief-749/attempt-5-report.json`, and write the
new cumulative `.temp/brief-749/report.json` in the 744 precedent's shape retaining attempts
1–5's STOP records and the attempt-6 outcome; `changes` empty (every write is under the allowed
paths); sections for the render, the scheduled-caller audit, the re-scan, the table and schema,
the source gate, the falsifiers, the pins, the budget and the Open ledger. Save every command's
full output in the scene; cite lines exactly. Before finishing run
`python3 scripts/report.py check 749` (without `--against-git`) and fix every PROBLEM; the final
message is only `python3 scripts/report.py summary 749`. STOP at Stage A: no epoch, no block, no
Stage B/C, no 744/745/746.
