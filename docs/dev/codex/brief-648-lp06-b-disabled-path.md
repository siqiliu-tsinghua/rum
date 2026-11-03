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
