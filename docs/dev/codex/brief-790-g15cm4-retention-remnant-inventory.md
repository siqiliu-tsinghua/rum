# Brief 790 — G15-CM4 retention-line remnant: the six named reds' complete inventory and repair plan — design-only

Base: `12083364` (the c148 gate). You are the design worker for this brief, not the project
director. `CLAUDE.md` belongs to the director and is not your instruction file. Follow
`AGENTS.md` and this brief. Do not commit, branch, or push.

## Why this brief exists

Ruling 798 / continuation 148 completed brief 789 under the director's implementation (the
user's takeover ruling: the agy window is saved for other hard problems; the next task goes to
cdx). The final Stage D workspace run on the armed render scene recorded exactly 16 reds: the
plan's 10 accepted exclusions (7 documented skips, 2 class_lease skips, the named
`q25_p2379_ownership_proof` stop) and **6 named open retention remnants**. The six are this
task's input. This brief is its design-only stage: a complete zero-unknown inventory of the
six, each one's exact mechanism and repair recipe, the scene-layer decision (which render
artifact must change and where it lives), the full downstream pin cascade, and drafted
implementation briefs in `TEMPLATE.md` shape. **Design only: `Scope: NONE — no source
change.`**

## The carried state (re-measure it; any drift is a stop)

- The four scene trees: `.temp/brief-784/images/{baseline,armed,disabled,real}`. The pins
  after the F16 cleanup, under the tree hash in `.temp/brief-784/tools/common.py`
  (`os.walk` with `EXCLUDES = {.git,target,.temp,__pycache__,.mypy_cache}`, sorted dirnames
  and filenames, `b"F\0" + relpath + b"\0" + content`), are:
  baseline `dd7c6539980aa4e0efa3241676cfd02d0550bb478919a70ac80180a392c655f8`/5,842;
  armed `d7640402a020dca79b50db13760b73c58d4b6b9085c6716a53e00c611d242042`/5,851;
  disabled `74d9c3454f8f4a6e0aa69f8fe435b00b389df78f289f0705372b8c1bd383799c`/5,851;
  real `8c45350bd0ec5933abaf26a028f1e2d97093617db9afab6de0759108b78eb0f7`/5,851; debris 0
  on each. The armed pin is the tree the Stage D log saw.
- The live tree: 23 inherited entries (unchanged since c88); its
  `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` is the F13
  live regeneration `b856f630…` (part of the WIP). Do not run git; use this record. Do not
  modify live.
- The Stage D evidence: `.temp/director/impl-789/logs/stage-d-workspace.log`
  (`passed=4547 failed=16 ignored=71`, peak 18,480,418,816 B, `oom=0`), the run script
  `.temp/director/impl-789/run_stage_d.py` (env: `RUM_CM740_BASELINE` = the baseline image,
  a per-tree `CARGO_TARGET_DIR`, the P0543 lane env
  `.temp/director/impl-789/lane/lane-env-789.json`, the 20G scope), and the census dumps
  `.temp/director/impl-789/logs/census-frontier.tsv` (armed) and `census-frontier-live.tsv`
  (live).
- The render's declared artifacts on the scene:
  `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/overlays.tsv`
  (`b059dbace46b4507eb6e9d8984ea02af427a4497a13fc8ccfe4c91d63ddacd3e`, 160 rows) and
  `additions.tsv` (`243dc0cf…`, 9 rows);
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutants.tsv`
  (`921f3b55…`, 151 rows, its `source_schema` column = the overlay table's sha);
  class-provenance `626eb3d4…`; the chain `0e508bb4…`; the allowance 4/4. The emission
  generators are `.temp/brief-769/generator.py` and `.temp/brief-768/generator.py`; the
  stage-C writers are `.temp/director/impl-789/stage_c_writers.py`; the gate scripts are
  `.temp/director/impl-789/gate/{common789,probe_build789,stage_c789}.py`; the P0543 lane is
  `lane_build.py`; F13 is `f13_refresh.py`.
- The live acceptance's fresh certificate output: `.temp/brief-546/certificates/` (written
  2026-11-24 19:40 by the live leg; `promoted-hash-true.tsv`, `acceptance-hash-true.tsv`, the
  four `LeaseProbe538-*` files).

## The six reds (verbatim from the Stage D log; re-read the blocks yourself)

| # | test | panic site | message |
|---|------|-----------|---------|
| 1 | `runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold` | `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:537:5` | `assertion `left == right` failed` (left 514, right 535) |
| 2 | `tests::retention_closure::census::tests::production_rejection_census_is_complete` | `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:194:9` | the pinned-site assertion (`r.site.start.0 == line`) fails |
| 3 | `tests::retention_closure::local::production::local_production_acceptance` | `crates/rum-stdlib/src/tests/retention_closure/local/production.rs:99:6` | `"stale or omitted instance certificate C01 hash=true"` |
| 4 | `tests::retention_closure::local::promotion_tests::production_inventory_rejects_removed_redirected_stale_and_omitted_evidence` | `crates/rum-stdlib/src/tests/retention_closure/local/promotion_tests.rs:207:33` | the same `C01` error |
| 5 | `tests::retention_closure::local::promotion_tests::production_promotion_cannot_launder_consumers_or_runtime_domain` | `crates/rum-stdlib/src/tests/retention_closure/local/promotion_tests.rs:283:9` | `assertion failed: red.contains("claimed covered")` |
| 6 | `tests::retention_closure::production_obligations_are_exact` | `crates/rum-stdlib/src/tests/retention_closure/local/promotion.rs:204:6` | the same `C01` error |

## What the director has already measured (verify each; do not re-derive from scratch)

1. **The C01 family is one root (reds 3–6).** `promotion::INVENTORY` is
   `include_str!("../inventory/promoted.tsv")` (`local/promotion.rs:6`). `reconcile`
   (`local/promotion.rs:130-136`) compares each committed row's active-build cell
   (`row[active]`, active = 4 for `cfg!(feature="hash")`, else 5) with the freshly computed
   `expected[row[0]]` and returns `stale or omitted instance certificate {label} hash=…` on a
   mismatch. The C01 row's cell is stale. `local_production_acceptance` writes the fresh cells
   to `.temp/brief-546/certificates/promoted-hash-<bool>.tsv` (`production.rs:59-89`) *before*
   its `reconcile(...).unwrap()` at `:91-99`; the row format is the 9-column `HEADER`
   (`promotion.rs:5`), and the fresh row writes the active cell and `UNRECORDED` for the other
   build, so an adoption must replace only the active cell and preserve the other-build cell.
   The live fresh file vs the live committed C01 row differ **only** in four
   `O0nn@fnv1a64:` instance hashes — `168ee0a5cb2dab98`→`7cdeff3857950e55`,
   `bcda74a56328cd37`→`9ee7cfbe92cbf61a`, `80b47c78ed1ba147`→`fb92a08896a4eb8a`,
   `5c948bebe0e2bbc0`→`e23ca20b1c50081d` — so the instance evidence moved, not the shape. The
   live committed row records `options/core.rs:106:5-122:6`; the armed row records
   `options/core.rs:214:5-230:6` (the render's rewrite). The default features include `hash`
   (`rum-stdlib/Cargo.toml:12-14`), so the hash=false cell comes from the
   `--no-default-features` build (the receipt gate's second configuration). The adoption
   recipe exists (brief 762-era) and the F55 rule applies: set the adopted file's mtime to the
   adoption time or cargo skips the `include_str!` rebuild. The two `promotion_tests` and
   `production_obligations_are_exact` fail because `reconcile` validates `INVENTORY` before
   their mutation-specific reds; fixing the one cell addresses all four (prove the ordering
   from the source and the log). **Superseded by Amendment 1 (below): the stale set is every
   root row's active cell — C01 four stamps and C02 seven — not C01 alone.**
2. **The census red is exactly one moved pin (red 2).** `census/tests.rs:176-206` pins the
   negative constructor site and five `(suffix, line, reason)` rows. Against the armed dump,
   only `("options/core.rs", 233, "method syntax")` misses: the only `method syntax` row in
   `options/core.rs` is now at `405:58`, same reason
   (`method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch`), and
   the live dump has it at `233:58` — the render's `options/core.rs` overlay row (overlays.tsv
   row 77, preimage `fd35cfa1…` → postimage `171a62fb…`) moved it. All other pins, the
   negative pin and the RET001 pin hold on both dumps; both dumps have 1,229 data rows and an
   identical `(kind, file, reason)` multiset (F60). **Director ruling R1: the pin is
   re-baselined to line 405, with the keyed pair (kind, file, callee, reason) recorded as
   unchanged.** Your inventory records the exact comparison evidence and the new bytes.
3. **The frame_root_trace red is a 21-row pure/stateful split (red 1).** The test's pure
   discovery (`frame_root_trace.rs:177-291`) classifies a frame as pure only when its
   `#[derive(...)]` contains `PureFrameAbandon`; the closed list `PURE_FRAME_ABANDON_TYPES`
   (imported from `frame_root_trace/derived_inventory.rs`) still lists the 21 frames whose
   rendered form drops `PureFrameAbandon` and hand-writes `FrameAbandon` (the "CM4 copied
   carrier … Q26/Q27 substitution render" form; e.g.
   `runtime/debug_tools/trace/resume_frames.rs:68`). The derived-`FrameRootTrace` list test
   passes (the 21 still derive `FrameRootTrace`); `derived_inventory.rs` and
   `frame_root_trace.rs` are byte-identical live vs armed, while the five rendered source
   files (`options/core.rs` and four `resume_frames.rs`) are overlay rows. **Director ruling
   R2: the 21 rows move from `PURE_FRAME_ABANDON_TYPES` to `STATEFUL_FRAME_ABANDON_TYPES`
   (in `frame_root_trace.rs`); `DERIVED_FRAME_ROOT_TRACE_TYPES` is unchanged.** The 21:
   ```
   crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame
   crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs:MatchGuardedReplacePartPatternTargetsOwnedSuspensionFrame
   crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs:MatchGuardedReplacePartPatternTargetsSuspendedSuspensionFrame
   crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs:PlanGuardedReplacePartTargetsOwnedSuspensionFrame
   crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs:PlanGuardedReplacePartTargetsSuspendedSuspensionFrame
   crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs:SelectGuardedReplacePartTargetsOwnedSuspensionFrame
   crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs:SelectGuardedReplacePartTargetsSuspendedSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs:EvaluateReplaceStringOutputSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs:EvaluateSuspendedReplaceStringOutputSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs:ResumeReplaceStringValueFromCandidateSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs:ResumeReplaceStringValuesFromIndexSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs:ResumeSuspendedReplaceStringValueFromCandidateSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs:ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs:DriveStringCasesOutputResultSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs:DriveSuspendedReplaceStringOutputResultSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs:DriveSuspendedStringCasesOutputResultSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs:ResumeCollectStringCasesValuesFromIndexSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs:ResumeReplaceStringValuesFromIndexSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs:ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs:ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrame
   crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs:SuspendReplaceStringOutputSuspensionFrame
   ```
4. **The scene-layer fact.** The three derived artifacts (the two frame lists and the census
   pins) and `inventory/promoted.tsv` are in no render table, while the scene's source differs
   from the baseline. The live tree's artifacts match the live (baseline-form) source, so a
   live-only edit would red the live tree now. **Director ruling R4: the repair lands in the
   render's emission (the scene), and the live-side edits of the same bytes are part of the
   render commit, not this task's immediate live change.** The design must settle the exact
   emission route (new overlay rows vs a new writer table) with evidence, and enumerate the
   cascade.

## Scope: NONE — no source change

Read-only. Your writes go to `.temp/brief-790/` only. Do not edit the images, the live tree,
`.temp/brief-784/` or anything else. **No cargo builds in this design stage**; the evidence
you need is archived (the Stage D log, the dumps, the certificate outputs, the tables). If you
become convinced a run is unavoidable, STOP and say exactly which run and why instead of
running it. Reading `.temp` is allowed; `python3` for parsing is allowed; no network; no git.

## Stage A — entry and evidence reproduction (600 s)

1. Re-hash the four scene trees and confirm the pins (algorithm and excludes above).
2. Re-read the six failure blocks from the Stage D log; copy the panic lines verbatim into
   your report.
3. Reproduce the three decompositions from the archives: (a) the frame lists from the log's
   `left`/`right` arrays (parse them; the 514/535 counts and the 21-row delta, sorted); (b) the
   census pins against both dumps (list every pin with its hit/miss and the moved row's new
   position); (c) the C01 cell diff (the four hash pairs and the two source sites).
4. Confirm the byte identities: live vs armed for the five rendered source files (differ) and
   for the three derived artifacts (equal); the eight overlay-table paths involved
   (`options/core.rs`, the four `resume_frames.rs`, and the three artifacts) present/absent in
   `overlays.tsv` and `additions.tsv`. Any mismatch with the director's measurements is a
   stop.

## Stage B — per-red inventory (1,200 s)

For each of the six reds, in a zero-unknown inventory: the object (test, table, cell, list
row); the source edge that produces the failure (`path:line`, quoted); the failure condition;
the recipe that reproduces it (from the archived evidence; no builds); the isolation falsifier
(what must be true, and what would falsify the attribution); the scene-layer disposition and
the exact bytes to change.

Specific obligations:

- **Reds 3–6**: the full reconcile chain (`expected` → the row comparison → the four hash
  pairs), the acceptance's write-then-fail order, the two build cells, the adoption rule
  (active cell only, other cell preserved, mtime), and the proof that the two `promotion_tests`
  and `production_obligations_are_exact` share the one root (quote the check order). Include
  what the armed scene's fresh cells will be and how they are produced (the acceptance's own
  run in the implementation stage).
- **Red 2**: the exact pinned-site block, the five-pin + negative-pin + RET001-pin status on
  both dumps, the moved row's keyed identity, and the new bytes for `census/tests.rs`.
- **Red 1**: the 21-row list above re-derived from the log, the two arrays' current and
  needed contents, the classification rule quoted from the test, and the new bytes for the two
  files (the pure list in `derived_inventory.rs`, the stateful list in `frame_root_trace.rs`).
- **The live-side counterpart**: which of the six are live reds today (the C01 family: yes,
  with the 19:40 fresh file as evidence), which are scene-only (FRT, census), and the exact
  bytes the render commit must carry for both.

## Stage C — the emission route and the pin cascade (1,200 s)

1. Decide the route by evidence: how every scene byte is accounted for today (the overlay and
   addition tables; the carried tables; the byte chain); whether the three artifact updates
   and the carried `promoted.tsv` update become overlay rows (then: the new row count and the
   table sha), a new writer table in the stage-C writers, or another ruled form. State the
   recommendation with the evidence and the alternatives' costs. The scene must be
   self-consistent: every file equals its baseline preimage or a declared postimage.
2. The ordering constraints: any `.rs` edit under `crates/` expires the retention contract
   fingerprints (`local/contracts.rs`), so each scene needs the F13 replay (regen → adopt with
   the F55 mtime rule) after the artifact edits and before the acceptance; the promoted cells
   depend on the contracts/marks, so the certificate adoption is last.
3. The complete downstream pin cascade, with each artifact and its check: the overlay table
   (rows/sha), the mutants table (`source_schema` and its 151 rows), the 12 receipts and the
   six-case gate minima (rows=160/rows=9 today), the additions (unchanged 9), the whole-tree
   counts (5,842/5,851 — file counts, not bytes), the arm bijection, the class-provenance
   table (verify it is untouched: the class-target cells hash `(tokens, tail, destructors)` of
   class cells), the chain (152 rows), the allowance 4/4, the typed-slot census, and the
   ownership-graph inventory replay duty (`crates/rum-stdlib/src/tests/ownership_graph/
   inventory/*.tsv`) for the edited `.rs` files. Anything you cannot settle with evidence is a
   named open question with options and a recommendation.
4. The re-emission and re-gate sequence, and the target bar: the six reds green, the plan's 10
   accepted exclusions still the only reds, 71 ignored unchanged, the receipts/six-case gate,
   the F48 bar, the 14-test focused gate, `cargo fmt`, then Stage D on the armed scene.

## Stage D — the drafted implementation briefs (600 s)

Draft the follow-up implementation briefs themselves, in `TEMPLATE.md` shape, under
`.temp/brief-790/drafts/`: the mechanical stages you recommend (e.g. one for the artifact
substitutions, one for the certificate adoption, one for the re-emission and the gates). Each
draft states its own Goal, Context, Scope, Deliverable, Verification, STOP rules, and the
disclosure duty; every decision named, no `TBD`. The director reviews and rewrites them; your
drafts are raw material.

## Bar

Zero unknown: every file, table, cell and row named; every claim carrying `path:line` or a
table position; the keyed pairings recorded; the cascade complete; the drafts in template
shape; `python3 scripts/report.py check 790` prints OK. A claim you cannot cite is a stop.

## Report format

`rum-report/1` JSON at `.temp/brief-790/report.json`; every quoted verdict line copied
verbatim; `runs` only if you actually ran something (the design expects no cargo run). Save
the decomposition tables as files under `.temp/brief-790/` and cite them. Before finishing,
run `python3 scripts/report.py check 790` and fix every PROBLEM. Your final message is
`python3 scripts/report.py summary 790` and nothing else.

## Checkpointing

After each stage write `.temp/brief-790/PROGRESS.md` with what is done and verified, the exact
next action, and the tree state. Prefer leaving the tree untouched at each checkpoint.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config (the shim refuses).
- Do NOT modify files outside `.temp/brief-790/`. If you believe you must, STOP and report.
- No network. The only commands you run are read-only shell/Python parsing and `cargo`-free.
- Testing Policy is a hard line (`AGENTS.md`): rum's own documented behaviour only.
- You MUST paste the real verbatim evidence. Never fake a line.

## Amendment 1 (c149, after the Stage A STOP): the certificate scope is the whole table

Stage A reproduced the carried state and the three decompositions and stopped correctly at this
brief's own rule: a C01-only repair reaches a second stale `hash=true` cell on C02, so the four
promotion tests would still fail before their later checks. The director verified the evidence
from the tables independently. On live, the committed `promoted.tsv` active cells differ from
the 19:40 fresh acceptance output in four C01 stamps (`O0210`, `O011`, `O0200`, `O010`) and
seven C02 stamps (`I01`, `I02`, `I04`, `R01`, `R02`, `B13`, `B05`); on armed, the seven C02
stamps disagree with the installed `local-contracts.tsv` fingerprints (all seven), and the four
C01 stamps likewise disagree with the armed F13 rows. The stale set is every root row's active
cell, not one row.

**Amended ruling R3**: the certificate repair is the whole-table adoption. For every root row
(`certificates::roots()`: C01 and C02), in every build the gates run (the default `hash` build's
column 4 and the `--no-default-features` build's column 5), the active cell is replaced by the
value that same build's own `local_production_acceptance` run writes to
`.temp/brief-546/certificates/promoted-hash-<bool>.tsv`; the other-build cell is preserved
exactly; the adopted file's mtime is set to the adoption time (F55). The fresh value is the
`promotion::expected` output by construction, so the adoption reconciles the table with the
scene's contracts — it is not a hand edit, and it is not a re-earning of the certificate.

**Stage A is accepted as evidence**: the four tree pins (baseline `dd7c6539…`/5,842, armed
`d7640402…`/5,851, disabled `74d9c345…`/5,851, real `8c45350b…`/5,851, debris 0), the Stage D
archive (70 result lines, 4,547/16/71, excluded=10, remnants=6), the 21-frame delta
(`pure 535→514, stateful 33→54`, left-only 0), the census (both dumps 1,229 rows; both
multiset equalities; the one pin miss, `233:58-233:68 → 405:58-405:68`), the file identities
(5 rendered-different, 3 derived-equal, 5/8 overlay hits, 0 additions, `promoted.tsv` not an
overlay), and the certificate tables. Do not redo Stage A beyond what the amendment changes.
**Resume at Stage B** and continue through Stages C and D under the amended scope; the drafted
implementation briefs must cover the whole-table adoption, both builds, and the live-side
counterparts at the render commit.
