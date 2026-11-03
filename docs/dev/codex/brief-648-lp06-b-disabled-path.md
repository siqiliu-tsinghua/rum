# Brief 648 — LP06 B: the disabled-path measurement campaign

**Measurement stage. One stop. No live source change.** The design round is
brief-645; its B draft is `.temp/brief-645/drafts/LP06-B-measurement.md` and
is raw material, not authority. A1 landed `6864588b`, A2 landed `1a4c44f3`;
both source gates are GREEN. This stage measures the pre-A1 tree (`654f2ce5`)
against the post-A2 tree (`1a4c44f3`) with the retained LP05 campaign
instruments, adapted only in their paths and LP06 coverage as ruled below.

## Goal

Prove that the ten new runtime keys, the builtin adoption and the Catch
lifecycle add **zero ordinary disabled-path work per converted site**:
instructions over the frozen 313-label panel (192 CLI + 121 resume), copies on
the 192 ordinary labels, allocations on the 194-row panel, stack growth on the
22 shapes, compiler layouts and live chains on every inventoried frame and the
LP06 types, the enabled diagnostic lifecycle, and the rum-vs-rum microbench.
Every ordinary positive is attributed to emitted PCs and functions; a
nonzero rum-side iteration-proportional delta is a defect, not noise. Every
unpassed gate stays unpassed. This stage repairs nothing: a required source
repair returns to a separately authorized source stage.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/dev/profiling.md`.
- `.temp/brief-645/plan.md` — the design; `…/drafts/LP06-B-measurement.md` —
  the campaign draft; `…/inventory/commands.tsv` — the bounded command
  receipts; `…/inventory/oracle-cells.tsv` — the 19 A2 cells.
- Exports and manifests (director-supplied, immutable):
  `.temp/brief-648/base` (`654f2ce5`), `.temp/brief-648/cand` (`1a4c44f3`),
  `.temp/brief-648/instruments/{base,candidate}-source-hashes.json`,
  `.temp/brief-648/instruments/entry-crate-hashes.json`.
- Adapted instrument package: `.temp/brief-648/instruments/` (scripts,
  frozen labels, 576 `.rum` programs, `program-hashes.json`,
  `layout-probes.json`, `effective-probe.rs`, `collect-layouts.gdb`,
  `inputs/`, `original-scripts/`, `measurement-contract.json`) and the
  resume harness `.temp/brief-648/b/resume-harness/` (its two absolute path
  literals already point at `.temp/brief-648/b/active`).
- Read-only origins: `.temp/brief-642/b-resumed/` and
  `.temp/brief-637/stage-b/perf/labels.json`. Read them; never execute a
  prior script in place; never modify them.
- Prior dispositions: `docs/dev/codex/brief-644-lp05-disabled-path-cost.md`
  (the neighbour-flags ruling at `:265`, and the inline-token repair at
  `:256`). A1/A2 reports and their raw receipts are historical evidence; they
  are not B results.

## Director rulings (decided; do not reopen)

**Q1 — the two exports.** Base is `654f2ce5` (the tree A1 landed on),
candidate is `1a4c44f3` (the gated A2 tree). Build both arms from the SAME
absolute source path `.temp/brief-648/b/active`, serially, with separate
target directories (`.temp/brief-648/instruments/target-<arm>`), release
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, `CARGO_BUILD_JOBS=8`, offline, one
cargo at a time. Never compare a cached older candidate, never relink a
consumed binary, never rebuild an arm after its consumers ran. Verify both
exports against their hash manifests before the first build and after the
last capture (`verify_sources.py`).

**Q2 — the instrument package.** `.temp/brief-648/instruments/` is the
adapted copy of `.temp/brief-642/b-resumed/`. The following prior-only
scripts are intentionally absent because they embed brief-642 assertions:
`prepare_resume.py`, `prepare_gates.py`, `final_audit.py`, `make_report.py`.
Do not recreate them; the report below is new. The frozen inputs are the
313-label `perf/labels.json`, the 194-row `perf/allocation-labels.json`, the
576 `perf/*.rum` programs, `program-hashes.json`, `layout-probes.json`,
`effective-probe.rs`, `collect-layouts.gdb`, `inputs/` and
`original-scripts/`. Preserve every algorithm, statement, N value, exclusion
and acceptance bar. Changes permitted in the copies: assigned paths, new
source/binary hashes, identical probe wiring, LP06-only probe/lifecycle
additions, disabled-flag preflight and explicit no-network settings. Every
changed line is disclosed in the report against its original.

**Q3 — bounds and order.** The adopted caps are
`.temp/brief-648/instruments/measurement-contract.json`: builds/stack 900 s,
instruction chunk 1650 s (720 s chunk + at most one 900 s child), reductions
120/180/600 s, lifecycle 300 s, layout 600/1800 s, live chain 900/600 s,
timing 240 s with a 180 s inner bound. The process maxima are 1252
instruction processes (initial plus at most one confirmation set, flagged
rows only), 768 copies, 776 allocations, 1056 stack probes, 352 live-chain
captures. These are maxima, not prompts to spend; never use unused
confirmation budget on unflagged rows. Order: preflight → builds →
instructions sample 1 → confirmation only for flagged rows → copies →
allocations → stack → layouts → live chains/prologues → lifecycle → timing
last. Every panel runs serially and alone; no build, test or other CPU work
concurrently.

**Q4 — LP06 coverage.** The mandatory focus is the ten converted sites and
their neighbors: while test/body, Compound item evaluate/suspend,
function/status `function.apply.evaluate`, both builtin dispatch arms
(unchecked and memory-growth-checked), and Catch total/body/finish (form,
tag, matcher, handler, bypass). Existing assignment/`builtin_set` labels
remain controls. The layout and lifecycle census adds
`AdoptedBuiltinTimingToken`, `BuiltinRecord`, `IncomingPathTimingToken`,
`PhaseEntry`, `ProfileBoundary`, `ProfileQueue` and the three Catch keys.
A probe for a type that exists only in the candidate is a candidate-only
addition, labelled as such and never compared as if the base had it.
Guest-facing shapes and existing frames are measured identically in both
arms.

**Q5 — the decision rule.** The instruction, copy, allocation, stack, layout,
live-chain and lifecycle panels decide. Timing rows are retained, disclosed
and compared per label, but they do not decide on their own: a timing
movement with a zero or negative instruction slope and overlapping
distributions is an observation, not a blocker; a **repeatable** hot-row
regression with a matching rum-side instruction/copy delta is a defect to
report with its raw evidence. This restates the brief-644 rule and prevents
a noise-only STOP. The disabled-path bar is zero added work per converted
site; a finding is not a waiver.

**Q6 — attribution.** Every ordinary positive in both pairs receives an
emitted-PC/function table (`attribute_pcs.py`, `pc_disposition.py`,
`resolve_pc_relocations.py` as adapted). A positive per-call delta on an
iteration-proportional rum function is a defect unless the PC evidence shows
the row's own code unchanged; fixed-call startup residuals are reconciled
separately in the whole-row table. External-only inflation may be reported
only when rum-side own slopes are unchanged.

**Q7 — outputs.** All evidence, copies, targets, binaries, logs and adapted
scripts live under `.temp/brief-648/`. Canonical roots of prior briefs stay
untouched. Never `/tmp`; never a network fetch; `DEBUGINFOD_URLS=""`.

**Q8 — scope of the stage.** This stage runs no workspace test, no standing
inventory replay and no acceptance suite; A1/A2 own those receipts. A live
source edit, a new public API/feature, or a new mechanism is a STOP, not a B
result.

## Scope

Files you MAY modify:

- copies under `.temp/brief-648/instruments/` and
  `.temp/brief-648/b/resume-harness/` — path adaptation, identical probe
  additions, LP06 probe/lifecycle additions, each disclosed line by line;
- new scripts, probes, logs and receipts under `.temp/brief-648/` only.

Files you MUST NOT touch: the live tree (`crates/`, `docs/`, `scripts/`,
`tools/`, `Cargo.toml`, `Cargo.lock`, `TODO.md`), every prior brief's
evidence under `.temp/` (read-only), and the two exports. No version-control
command, no network, no `unsafe`, no source outside the copies. Never
execute a prior script in place.

## Deliverable

1. **Preflight manifest.** Tool paths/versions, allowed CPUs and pinned CPU
   (2 when available), the 313 labels, 194 allocation rows, 22 shapes,
   probe bytes, source/binary SHA-256 per arm, build commands, target and
   output roots, and the cleared `RUM_PROFILE_*` environment. Confirm
   `memory.max` from `/sys/fs/cgroup` and quote it.
2. **Instruction panel.** All 192 CLI + 121 resume labels, exact N/2N
   programs, one paired sample, `cpu_core/instructions/u` on CPU 2, stdin
   `/dev/null`, paired difference divided by N. At most one confirmation
   sample for the flagged-row list only. Report every positive including
   those below any summary threshold.
3. **Copy census.** All 192 ordinary labels at
   `N = max(1, min(original N, 2000, floor(20M/base instruction slope)))`;
   exact statements unchanged; the full existing copy-function classes, not
   one libc counter.
4. **Allocation panel.** The 194 rows (192 ordinary plus the two
   `logical-lifecycle-*` controls), exact N/2N, count/total bytes/peak-live;
   ordinary profiling switches off; the instrument's declared
   `RUM_PROFILE_ALLOC=1` exception is the only one and is receipted.
5. **Stack panel.** All 22 debug/release shapes at their frozen heights,
   three repeats: 528 probes and 44 fits per arm, zero fit residual, no
   unexplained ordinary growth. Add the LP06 builtin/Catch shape set
   identically in both arms and report its own count and fits.
6. **Layout and live chains.** Compiler DWARF for every inventoried frame,
   the 16 named probes, the eight unemitted `DeleteMissing` sizes, all 44
   live-chain shapes, and the LP06 profiler-only types (candidate-only
   additions labelled). Separate enabled diagnostic growth from ordinary
   frame growth; explain actual prologues/caller staging at the converted
   starts, not `size_of` alone.
7. **Lifecycle.** Runtime forwarded, builtin and Incoming records across a
   second pause, normal/error completion, structural abandonment, session
   reset/drop and capped allocation: owner/generation correctness, quiescent
   leases, no missing/duplicate finish, diagnostic-byte balance. Repeated
   disabled starts produce no epoch entries, retained handles or allocation
   growth.
8. **Attribution.** The Q6 PC/function tables for every positive, the
   whole-row reconciliation, and the brief-644 disposition applied.
9. **Rum-vs-rum microbench.** Every hot label and same-tier neighbor, flags
   off, in-kernel `AbsoluteTiming`/`Instant`, retained `timings.tsv`; at most
   one confirmation pair for flagged rows; record the Q5 disposition.
10. **Final index.** One immutable receipt: every attempt, every verdict,
    every unchanged/changed row, unpassed gates, and the closing
    `verify_sources.py` proof. No hand-edited receipt, no extrapolated row.

## Verification (MANDATORY — you run it)

Every campaign command goes through the package wrapper so the serial lock,
environment clearing, bounded child and `runs.jsonl` receipt hold (the
wrapper itself calls `scripts/run-bounded.py`):

```
python3 .temp/brief-648/instruments/run.py <id> <timeout> -- <command...>
```

Use unique `<id>`s; `run.py` refuses to overwrite a log, so a re-entered chunk
gets a new id and `measure.py` skips completed receipts. The bounded forms,
in order (each through `run.py`):

```
python3 .temp/brief-648/instruments/instrument_preflight.py            (120)
python3 .temp/brief-648/instruments/verify_sources.py                 (120)
python3 .temp/brief-648/instruments/campaign_build.py base    cli|cli-alloc|resume|resume-alloc|stack-debug|stack-release|layout-stdlib   (900 each)
python3 .temp/brief-648/instruments/campaign_build.py candidate <same kinds>                                                      (900 each)
python3 .temp/brief-648/instruments/measure.py instructions 1         (1650, re-entered in chunks)
python3 .temp/brief-648/instruments/compare_instructions.py           (120)
python3 .temp/brief-648/instruments/measure.py instructions 2         (only flagged rows)
python3 .temp/brief-648/instruments/compare_instructions.py final     (180)
python3 .temp/brief-648/instruments/run_copy.py                       (1650)
python3 .temp/brief-648/instruments/compare_copies.py                 (180)
python3 .temp/brief-648/instruments/attribute_copy.py                 (600)
python3 .temp/brief-648/instruments/measure_alloc.py                  (1650)
python3 .temp/brief-648/instruments/compare_alloc.py                  (180)
python3 .temp/brief-648/instruments/stack.py base|candidate           (900 each)
python3 .temp/brief-648/instruments/compare_stack.py                  (120)
python3 .temp/brief-648/instruments/layouts.py base|candidate         (600 each)
python3 .temp/brief-648/instruments/extra_layouts.py                  (1800)
<LP06 layout probe script>                                            (600)
python3 .temp/brief-648/instruments/compare_layouts.py                (180)
python3 .temp/brief-648/instruments/frame_collect.py                  (900)
python3 .temp/brief-648/instruments/frame_reduce.py                   (600)
python3 .temp/brief-648/instruments/lifecycle.py                      (300)
python3 .temp/brief-648/instruments/timing_pipeline.py                (240 per pair, inner 180)
python3 .temp/brief-648/instruments/timing_supplement.py              (flagged rows only)
python3 .temp/brief-648/instruments/compare_timing.py                 (120)
python3 .temp/brief-648/instruments/verify_sources.py                 (120)
```

The attribution commands (`attribute_pcs.py`, `pc_disposition.py`,
`resolve_pc_relocations.py`) follow the retained procedure and are bounded by
the 600-second reduction form. If an instrument's hard-coded input is absent
or stale, STOP and report it; do not substitute a value or rerun a consumed
reference. A red is reproduced once at the same setting before it is called
a defect. An exit 137 with a risen `oom_kill` gets one rerun at one build
job. No wall timeout is a measurement.

## Checkpointing — protect the scene

After each build/panel/chunk/reduction retain command, environment, hashes,
raw output, receipt IDs, passing and flagged rows, disposition and the exact
next command. Track: labels with valid pairs, ordinary positives resolved,
copy/allocation rows complete, stack fits, layout/live-chain coverage,
lifecycle proofs, timing rows, and the closing source-integrity check. Two
attempts that move none of these counters STOP as a question. Preserve every
failed capture; never overwrite or silently count it.

## Hard rules

- No version-control command; no network; no live source change; no edit to
  prior evidence; no output outside `.temp/brief-648/`.
- One cargo at a time; release codegen units 1; jobs 8; offline; all
  `RUM_PROFILE_*` cleared for instructions/stack/copies/timing, the declared
  allocation exception only.
- Measurements serial and alone; timing last; no competing load; pinned CPU.
- Rum-vs-rum only; exact shared expressions; no added `Set`/sink/`Print`.
- A repeatable hot-row regression is a finding, not a waiver; a zero-delta
  instruction slope with overlapping timing distributions is an observation,
  not a blocker (Q5).
- Never hand-edit a receipt, never rebaseline a ledger, never extrapolate a
  missing row, never reuse a prior campaign's verdict as a B result.
- Reply to the director in the report only; the tree is the deliverable.

## Report format

Write a `rum-report/1` report at `.temp/brief-648/report-b.json`: changed
files and lines (the adapted copies, line by line against their origins),
every run and attempt, exact verdicts, the per-label instruction table with
both pairs, the copy/allocation/stack/layout/live-chain/lifecycle receipts,
the attribution tables, the timing rows and their Q5 disposition, the final
source-integrity check, and every unpassed gate. Check it with
`scripts/report.py check` (without its version-control option) and return
its summary and the full compliance verdict lines.

---

## Amendment 1 — orchestration roles, the build phase, and the resume (after the attempt-1 STOP)

**The attempt-1 STOP is accepted in kind.** The worker audited the supplied
package before the first panel and found conflicts that are mine, not its
own. It ran no measurement, changed no live source; the tree is clean and
both exports verify. I reproduced the decisive one myself:
`timing_pipeline.py:20` spawns `run.py` per child, while the outer `run.py`
holds `serial.lock` with `LOCK_EX|LOCK_NB`, so the child dies at
`run.py:18` with `BlockingIOError` (receipt:
`.temp/director/brief-648-stop-check/`). The pipeline's confirmation sample
is the whole 129-label panel, `compare_timing.py` cannot run after the
pipeline has written its output, and `extra_layouts.py:31` invokes cargo at
a position the prescribed order places after consumers. All four findings
are real; the brief was wrong.

**R1 — orchestration roles.** A script that launches its children through
`run.py` is an orchestrator: invoke it **directly**, never through `run.py`,
and run it alone with nothing else executing. Bound the orchestrator itself
with `scripts/run-bounded.py` (for example, `python3 scripts/run-bounded.py
--timeout 2400 -- python3 <orchestrator>`), retain its stdout as a log, and
let each child take the serial lock and write its own `runs.jsonl` receipt.
`timing_pipeline.py` is the only listed orchestrator; if another script
turns out to spawn `run.py`, the same rule applies. Every measurement
command that is not an orchestrator still goes through `run.py`.

`run.py` parses the id's trailing `-<digits>` as the attempt number
(`run.py:36`) and raises without it. Attempt 1's ids are taken. Every
command in this attempt uses a new id ending in `-2`; a re-entered chunk
gets `-3`, and so on.

**R2 — timing confirmation.** Keep `timing_pipeline.py` unchanged. Its
retained semantics are authorized: one paired sample over the 129-label
panel (120 canonical + 9 supplemental), then at most one further paired
sample of the **whole panel** only when the first sample has any positive
row; the final rows are best-of-two and the pipeline writes
`timing-comparison-{1,2}.json` and `timing-verdict.json`. The sentence "at
most one confirmation pair for flagged rows" in Deliverable 9 is amended
for timing: the confirmation set is the whole panel, taken at most once,
and no further timing draws are permitted. Confirming only the rows that
were positive would select the data; the whole panel is the honest set, and
Q5 already keeps timing non-decisive. `timing_supplement.py` runs per arm
and sample exactly as the pipeline invokes it. The pipeline's own child
bounds are its retained envelope — 240 s for the canonical microbench and
300 s for the nine-label supplement, with the microbench's 180 s inner
bound; the contract's 240/180 describes the canonical child.

**R3 — the reducer.** Do not run `compare_timing.py`. It reads only the
120-label canonical panel and asserts `timing-comparison-<sample>.json`
absent (`compare_timing.py:7,31`), which `timing_pipeline.py:49` has already
written. The pipeline's comparison and verdict files, with the raw
`timings.tsv`, are the timing receipts. An independent cross-check is
allowed only as a new script under `.temp/brief-648/` that reads the
retained tsvs and writes a new path; disclose it.

**R4 — build before consumers.** Every cargo invocation runs in the build
phase, before the first measurement consumer: all `campaign_build.py` kinds
for both arms, `extra_layouts.py`, and any candidate-only probe build. Run
`extra_layouts.py` immediately after the campaign builds, not in the layout
phase. The Q1 sentence "never rebuild an arm after its consumers ran" is
amended to its intent: once a consumer has read `bin/<arm>/<kind>`, its
SHA-256 in `builds/<arm>/<kind>/binary.json` is immutable, and no later
command may replace, relink or rebuild it. A cargo run that produces only a
separate test artifact in `target-<arm>` is permitted in the build phase
and never later.

**R5 — LP06 probes are build inputs.** Anything that must be compiled — the
shared layout probes, the candidate-only profiler-type layouts, and the
LP06 lifecycle tests — is installed in the active tree during the build
phase, before the first consumer. Shared probes are identical in both arms;
candidate-only probes are installed only in the candidate's stack-debug
build and labelled; the base arm keeps the retained controls. The layout
and lifecycle panels read the pinned binaries and compile nothing.

**R6 — the resume.** Attempt 1 holds no measurement. Preserve its record
before writing anything: copy `report-b.json`, `report-b.assembly.json`,
`unpassed-panels.json`, `final-receipt-index.json`, `evidence-index.json`,
`PROGRESS.md` and `PROGRESS.capture.md` into `.temp/brief-648/attempt-1/`;
leave `instruments/logs/`, `instruments/runs.jsonl`, `serial.lock`,
`builds/`, `bin/` and `b/active` in place; never delete or rewrite an
attempt-1 receipt. `builds/base/cli/binary.json` (sha256 `06d8df06…`) is a
completed build: verify the source export against `base-source-hashes.json`
and the binary against its `binary.json`, then continue with the remaining
kinds; `campaign_build.py` refuses to repeat a completed build, so do not
delete it to force one. `preflight_manifest.py` takes the attempt number:
use `2`. The campaign report is written to `report-b.json` as the brief
says; attempt 1's copy stays under `attempt-1/`.

**R7 — an interface conflict is not a stop.** Where an invocation or
ordering detail conflicts with a retained script's own interface, choose
the reading that produces the measurement this brief defines, disclose the
choice in the report, and continue. A conflict is a question only when it
would change an acceptance bar, a measurement algorithm, a frozen input, or
would require a source repair. The wrapper is a receipt mechanism; it must
not cost a campaign.

**Answer to Q1:** proceed under R1–R7. The rest of the brief stands
unchanged — the panels, the bars, Q5, the report format and the closing
source-integrity check.

---

## Amendment 2 — the copy reducer is supplied (after the attempt-2 STOP)

**The attempt-2 STOP is accepted.** The worker audited
`compare_copies.py:14` twice, found `instruments/copy-census/copy_census.py`
absent, substituted nothing and ran no panel. That is my packaging miss:
the retained `copy-census/` directory was copied selectively and its
reducer was left behind.

**The supplied input.** `.temp/brief-642/b-resumed/copy-census/copy_census.py`
(sha256 `b282103dd93988a67674925bd449a6c03a39b8838943d84639507c4684bfb9c0`)
is now at `.temp/brief-648/instruments/copy-census/copy_census.py`,
byte-identical; provenance at
`.temp/brief-648/copy-reducer-provenance.json`. No prior output was copied:
the directory contains only the reducer, and `run_copy.py` generates the
per-size programs, the Callgrind outputs and the receipts there.
`compare_copies.py` then runs the reducer, which reads `perf/labels.json`,
`ordinary-callgrind/1/*-budget.json` and the `.out` files. The reducer is
a supplied input, not an adaptation: do not edit it; disclose it as
supplied.

**Input audit.** I walked every direct path literal in the package and
checked it against the origin: the reducer was the only retained input
missing. Every other absent path is generated by this campaign
(`copy-census/`, `ordinary-callgrind/`, `perf/instructions`, `perf/alloc`,
`layouts`, `stack-*`, `safe-frame-attribution`, `lifecycle`,
`pc-disposition`, and every comparison/verdict file) or belongs only to
`rule-layout`/`prologues`, which this stage does not run. If another
hard-coded input is absent, STOP is still correct.

**Resume state after attempt 2.** Attempt 2's record is preserved under
`.temp/brief-648/attempt-2/`. Completed pinned builds: base `cli`
(attempt 1), base `cli-alloc`, `resume`, `resume-alloc`, `layout-stdlib`,
`stack-debug` (attempt 2). Remaining builds: base `stack-release` and
every candidate kind. Per attempt 2's probe record, the shared LP06 probes
are compiled into the base `stack-debug` build; the candidate-only
additions compile in the candidate build. Attempt-2 ids end in `-2`; this
attempt uses new ids ending in `-3`, and a re-entered chunk gets `-4`. Do
not repeat a completed build; `campaign_build.py` refuses one.

**The rest of the brief stands**, including Amendment 1's R1–R7.

---

## Amendment 3 — the corrected LP06 lifecycle probe and the remaining panels (after the attempt-3 STOP)

**The STOP is accepted.** The candidate-only no-epoch probe at `epoch.rs:412`
calls `Session::logical_profile_test_leases()`, which goes through
`epoch::current()` → `Epoch::new_unleased`, before asserting at `:415`
that no epoch exists. The observer creates the state it asserts absent, so
the assertion can never pass. Two identical failures on the same pinned
binary, no weakened assertion, no rebuilt binary: an invalid probe oracle,
not a runtime regression.

**Q1 answer: option 1 — one corrected diagnostic-only artifact.** Apply
exactly the proposed probe-only correction: read the slot after
`drop(session)`, assert `slot.epoch.is_none() && slot.failed.is_none()`,
and take the zero-lease count from the same slot
(`slot.epoch.as_ref().map_or(0, |e| e.contexts.load(Relaxed))`), removing
the initializing accessor call. The three assertions are preserved;
nothing is waived.

Build it as a **new, separate diagnostic kind** — not `stack-debug`, not
any pinned binary. Use a fresh copy of the candidate export, a separate
target directory, a new `bin/candidate/lifecycle-debug` and
`builds/candidate/lifecycle-debug/binary.json`; install the same shared
probes plus the corrected candidate-only probes. This is the one
authorized exception to R4/R5: a diagnostic-only build after consumers is
permitted exactly here, and only if it replaces, relinks or overwrites no
pinned artifact; record the new artifact's SHA and disclose the build. The
completed stack, layout and live-chain receipts keep their pinned
`stack-debug` SHA.

**Remaining panels; do not repeat completed ones.** The 313 initial + 71
flagged-only instruction pairs, the 192 copy rows, the 194 allocation
rows, the 1056 canonical + 432 LP06 stack probes, the 563-frame layout
tables, the 352 live chains, the extra DeleteMissing sizes and the six
retained lifecycle controls are complete and must not be rerun. Resume
exactly:

1. the corrected candidate `lifecycle-debug` build;
2. the LP06 lifecycle proofs on it — disabled no-epoch/no-handles, the
   three-family × seven-ending retirement/second-pause/owner-generation
   set, and diagnostic-byte balance; the base arm keeps the six retained
   controls;
3. the emitted-PC attribution for the 27 ordinary and 2 resume positives
   (`attribute_pcs.py`, `pc_disposition.py`, the retained relocation
   procedure), with the whole-row reconciliation;
4. `timing_pipeline.py` under R1/R2, last;
5. the final index and report.

**The native-staging finding stands and is not waived.** The unchecked CLI
dispatch entry reservations (856→888 and 856→872 bytes) and the Catch
finish +16 bytes in all three kinds are measured positives; the checked
`cfg(test)` chain savings do not prove the unchecked CLI arm unchanged.
Keep them unpassed. Do not rerun the prologue/layout panels, and do not
substitute `size_of` or a net-chain waiver.

**Lifecycle verdict composition.** The lifecycle gate is PASS only when
the six retained controls and every LP06 proof pass. The retained
`lifecycle.py` verdict remains a separate receipt; the pipeline's
`lifecycle-verdict.json` precondition may stand, but the report must state
the combined disposition, and timing must not be called before the LP06
lifecycle is resolved. A corrected probe that exposes a genuine runtime
failure is a finding: record it, run the remaining non-deciding
measurements, and never weaken the assertion.

**Resume state.** Attempt 3's record is preserved under
`.temp/brief-648/attempt-3/`. Attempt-3 ids end in `-3`; this attempt uses
new ids ending in `-4` (a re-entered chunk gets `-5`). All pinned
artifacts and exports remain immutable, and the closing
`verify_sources.py` still applies.
