# Brief 653 — LP06 D: the fresh repaired-tree campaign

## Goal

Decide LP06 on the repaired tree. The pre-A1 base (`654f2ce5`) and the
repaired candidate (HEAD `029c06ff`, after C1–C3) are measured afresh
through every deciding B panel, and each finding the B campaign opened is
either shown absent or retained with its evidence: the builtin recorder's
19 own Ir/call, the 27 ordinary + 2 resume positive labels, the native
staging rows (`try_dispatch_builtin_call` 856→888/872, Catch finish
1016→1032 and 792→808, forwarded slow 24→40), the F5 equality edge and the
F6 neighbour rows. The original B candidate (`1a4c44f3`) is red
calibration only, never a re-used result. This stage repairs nothing: a
source defect found here returns to a separately authorized source brief,
and LP06 is not called closed by this report — the director rules after
the gate.

## Context pointers

- Read `AGENTS.md`, then the design report `.temp/brief-649/report-design.json`
  sections "D — fresh repaired-tree B panels and preserved exclusions" and
  "Draft measurement brief D", plus `.temp/director/brief-648-b-verdict.md`.
- The retained procedure is `docs/dev/codex/brief-648-lp06-b-disabled-path.md`
  (including amendments 1–3); its command book applies with the root
  substitution `.temp/brief-648` → `.temp/brief-653`. Its retained rulings
  (frozen statements, flags, bounds, order, attribution duty, timing
  non-decision) carry over.
- Source stages and their gates: briefs 650 (C1), 651 (C2) and 652 (C3),
  with the C3 gate numbers as a cross-check only: forwarded bodies
  4×24/56/24, While 744/3528/744, Compound 648/1304/664, Catch
  1000/792/1000, `try_dispatch_builtin_call` [872, 856].
- The prepared root is `.temp/brief-653/`: `base/` (byte copy of the
  immutable pre-A1 export, re-verified against its hash manifest),
  `cand/` (the repaired export, 2,607 files at HEAD `029c06ff`) and
  `instruments/` (adapted copy of the B package). The contract is
  `.temp/brief-653/instruments/measurement-contract.json`.

## Director rulings (decided; do not reopen)

**Q1 — exports and build path.** Base `654f2ce5`, candidate `029c06ff`.
Both arms are copied into the same absolute path `.temp/brief-653/b/active`
and built serially into separate targets with release
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, `CARGO_BUILD_JOBS=8`, offline.
Never reuse a B binary, target or receipt; never relink a consumed binary.
Verify both exports against their manifests before the first build and
after the last capture (`verify_sources.py`).

**Q2 — the instrument package.** `.temp/brief-653/instruments/` is the
adapted copy. Changes already made by the director, to be disclosed
unchanged in the report:

- `lp06-candidate-probes.json`, `lp06-candidate-probes-corrected.json` and
  `lp06-lifecycle-budget.rs` call the forwarded start through its C3 form
  `::<CompoundItemSuspend>(&value)`; every other probe line is the B text.
- `lp06-candidate-probes-corrected.json` is the Amendment-3 corrected
  observer (no accessor that creates an epoch), applied as a supplied file
  instead of being derived at build time.
- `build_lifecycle.py`, `lifecycle_proofs.py`, `launch_timing.py` and
  `checkpoint.py` are the D forms of the attempt-4 scripts, with the B
  evidence-index dependency removed.
- `verify_sources.py` prints D labels; `b/resume-harness/` points at
  `.temp/brief-653/b/active`.
- Frozen inputs: the 313-label `perf/labels.json`, the 194-row
  `perf/allocation-labels.json`, the 71 flagged
  `perf/confirmation-labels.json`, the 576 `perf/*.rum` programs,
  `program-hashes.json`, `layout-probes.json`, `lp06-shared-probes.json`,
  `effective-probe.rs`, `lp06-stack-probe.rs`, `collect-layouts.gdb` and
  `inputs/`.

**Q3 — bounds and order.** The contract caps: builds/stack 900 s;
instruction chunk 1650 s (720 s chunk logic, at most one 900 s child);
reductions 120/180/600 s; lifecycle 300 s; layout 600/1800 s; live chain
900/600 s; timing outer 2400 s with the pipeline's 240/180 inner bounds.
Process maxima: 1,252 instruction processes, 1,252 confirmation, 768
copies, 776 allocations, 1,056 stack probes, 352 live chains. They are
maxima, not prompts. Order: preflight → builds → instructions sample 1 →
confirmation only for flagged rows → copies → allocations → stack →
layouts → live chains/prologues → lifecycle → timing last. Every panel is
serial and alone.

**Q4 — LP06 coverage.** The mandatory focus is the ten converted sites
and their neighbours: while test/body, Compound item evaluate/suspend,
function/status `function.apply.evaluate`, both builtin dispatch arms,
Catch total/body/finish (form, tag, matcher, handler, bypass), the
legacy-only `IncomingPathTimingToken` carrier, and the four static
forwarded phase specializations. Assignment/`builtin_set` labels stay
controls. Candidate-only types are labelled and never compared as if the
base had them.

**Q5 — the decision rule.** Instructions, copies, allocations, stack,
layouts, live chains and lifecycle decide. Timing is retained, disclosed
and compared per label, but does not decide alone; a repeatable hot-row
timing increase with matching rum-side instruction/copy work is a defect,
a timing-only movement with overlapping distributions is an observation.
Do not run `compare_timing.py` (the retained pipeline verdict is the
timing receipt).

**Q6 — attribution and disposition.** Every ordinary positive in both
instruction pairs receives an emitted-PC/function table
(`attribute_pcs.py`, `pc_disposition.py`, `resolve_pc_relocations.py` as
adapted). A positive per-call delta on an iteration-proportional rum
function is a defect unless the PC evidence shows that row's own code
unchanged; fixed-call startup residuals are reconciled separately. F5
must be supported separately: extract every incoming equality edge from
the new paired capture, reconcile whole-row startup versus iteration
work, resolve the data/jump-table references, and match generic insert
bodies to owner operations. Do not call a changed incoming slope a new
diagnostic call unless an actual new edge is present.

**Q7 — outputs.** Every new artifact, copy, target, binary, log and
receipt lives under `.temp/brief-653/`. Retained roots (`.temp/brief-648/`
and all other brief roots) stay read-only and are never executed in place.
Never `/tmp`; no network; `DEBUGINFOD_URLS=""`.

**Q8 — scope of the stage.** No live source edit, no workspace test, no
standing inventory replay, no acceptance suite, no version-control
command. A live source change, a new public API/feature, or a new
mechanism is a STOP. The final source audit is the closing
`verify_sources.py` run.

## Scope

NONE for live source. The only writable area is `.temp/brief-653/`
(the assigned root), plus process state needed to run the instruments.
The instruments supplied are used as delivered; a missing or stale
hard-coded input is a STOP with the value, never a substitution.

## Deliverable

Paired tables for every panel over both arms, the B red calibration beside
the repaired outcome, all disabled recorder/caller/Catch/static-phase/
neighbour falsifiers (each converted site's own-slope evidence), the F5
equality-edge and relocation disposition, the F7 native-staging rows with
their emitted-PC evidence, compiler layouts, physical chains, the enabled
and disabled lifecycle proofs, timing/Q5 retention, and the immutable
final source/binary audit. Every unpassed bar stays visible; nothing is
waived by an aggregate total.

## Cost bars and must not change

- Zero added disabled work per converted site against base, and no
  iteration-proportional rum increase on a converted row, without
  subtracting savings from unrelated rows or outlined helpers.
- The repaired LP06 shapes are cross-checks, not substitutes: forwarded
  slow four bodies each ≤ 24 / 88 / 24 bytes (cli / stack-debug /
  stack-release); While ≤ 776/3528/776; Compound ≤ 696/1304/680; Catch
  ≤ 1000/792/1000; the three release wrappers absent with cold bodies
  present; `IncomingPathTimingToken` and `Option<...>` 56 bytes.
- Stack, CFA, layouts and live chains keep their B contracts: zero
  residual fits, unchanged shared layouts, no new physical frame; a
  same-PC inline display entry is not a new frame.
- Frozen statements, N values, flags, copy classes, allocation rules,
  stack shapes, frame inventory, lifecycle cells, timing sample limits
  and every retained exclusion stay exactly as in B. No seed change, no
  adjusted acceptance threshold, no erased label, no external-engine
  comparison.
- The F7 native reservation and F5 equality finding are measured and
  reported, not waived; if either persists, say so with its evidence and
  the row's own-PC table.

## Verification (MANDATORY — you run it)

Every campaign command goes through the package wrapper:

```
python3 .temp/brief-653/instruments/run.py <id> <timeout> -- <command...>
```

Use unique `<id>`s; the wrapper refuses to overwrite a log and `measure.py`
skips completed receipts. In order:

```
instrument_preflight.py                                              (120)
verify_sources.py                                                    (120)
campaign_build.py base    cli|cli-alloc|resume|resume-alloc|stack-debug|stack-release|layout-stdlib   (900 each)
campaign_build.py candidate <same kinds>                             (900 each)
build_lifecycle.py                                                   (900)
measure.py instructions 1                                            (1650, re-entered in chunks)
compare_instructions.py                                              (120)
measure.py instructions 2                                            (flagged rows only)
compare_instructions.py final                                        (180)
attribute_pcs.py / pc_disposition.py / attribute_target.py / resolve_pc_relocations.py   (600 reductions)
run_copy.py                                                          (1650)
compare_copies.py                                                    (180)
attribute_copy.py                                                    (600)
measure_alloc.py                                                     (1650)
compare_alloc.py                                                     (180)
stack.py base|candidate                                               (900 each)
compare_stack.py                                                     (120)
lp06_panels.py stack base|candidate                                  (600 each)
lp06_panels.py compare-stack                                         (120)
layouts.py base|candidate                                             (600 each)
extra_layouts.py                                                     (1800)
lp06_panels.py layouts                                               (600)
compare_layouts.py                                                   (180)
frame_collect.py                                                     (900)
frame_reduce.py                                                      (600)
lp06_prologues.py                                                    (120)
lifecycle.py                                                         (300)
lifecycle_proofs.py                                                  (300)
launch_timing.py                                                     (outer 2400)
verify_sources.py                                                    (120)
```

`lp06_panels.py lifecycle` is not used in D: the LP06 lifecycle proofs run
through `lifecycle_proofs.py` on the separate `lifecycle-debug` artifact
that `build_lifecycle.py` pins. `compare_timing.py` is not run, and the
timing pipeline invokes `timing_supplement.py` itself for the nine
supplement rows — do not invoke it separately. The lifecycle proofs
require `lifecycle.py`'s six retained controls to have completed. An exit
137 with a risen `oom_kill` gets one rerun at one build job. A red is
reproduced once at the same setting before it is called a defect; no wall
timeout is a measurement.

## Checkpointing — protect the scene

After each build, panel, chunk and reduction record command, environment,
hashes, counters and the exact next action with
`python3 .temp/brief-653/instruments/checkpoint.py <name> "<next command>"`.
Pinned binaries and completed receipts are immutable; a re-entered chunk
gets a new id. Two attempts that move none of the counters stop with
evidence. Preserve failed runs.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, source edit or prior-evidence mutation.
- No re-used B result, shared target directory, nested `run.py` lock, late
  relink, invented missing row, blind seed change, `size_of`-only stack
  proof or timing-only blocker.
- All new output under `.temp/brief-653/`; `.temp/brief-648/` and every
  other retained root stay read-only.
- Only rum-native measurements and rum-vs-rum A/B; no external-engine
  comparison, no committed third-party output.

## Report format

Write a `rum-report/1` report at `.temp/brief-653/report.json`: every run
with its attempt and exit, the red calibration rows beside the repaired
outcomes, all per-label and per-shape verdicts, the converted sites'
own-slope tables, F5 and F7 dispositions with their evidence, the final
source/binary audit, and the unpassed list (LP06 is ruled by the director,
not by this report). Run `python3 -I scripts/report.py check 653` (without
`--against-git`) and return its summary and every compliance verdict line.
