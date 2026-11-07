# Brief 687 — LP08 residual: the disabled publication cost

**Cost design on the brief-686 form: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. Brief 686 is the first
enabled-green, exact-stack form; its form tree is the control, not
evidence to re-derive from zero.**

Brief 686 measured the full disabled cost panel on the green form. The
decoded view itself has no disabled slope, but inherited publication
still costs the default path:

- `resolution.rs:569` `suspend_initial_evaluation_once_with_boundary`:
  +9 instructions/call (6 publication-gate + 3 other) from the
  `publish_stashed_phases` gate at `profiling/logical/mod.rs:991`;
- `eval_core/call.rs:94,95` `suspend_call_stage_with_boundary`:
  +60 native / +30 argument per program (publication and relocation);
- `eval_core/call.rs:41,42` `suspend_prepared_stage_with_boundary`: +17;
- `eval_core/call.rs:388,448`: +1 other instruction;
- `builtin_catalog.rs:80,85,94`: a source-identical lookup variation of
  -22 / -15 left unqualified.

The brief's bar is exact: the disabled path may pay only the queued
path's own `runtime_path_timing_start` /
`record_profile_runtime_path_timing` calls, measured in the same run.
The `resolution.rs:424,431,445` and `eval_call_prepare.rs:395,403`
rows are attributable; the publication and other rows above are not,
so Stage B stopped and Stage C stayed gated.

This round answers one question with measured evidence: **can the
publication gates be removed or absorbed so the disabled path pays
only the queued path's standing per-site gate — while the enabled
battery stays 20/20 and the exact stack stays 18,304 / 18,896 with
residual 0 — or is the residual cost a measured impossibility with a
smallest correct alternative?**

## Stage A — calibration

1. **Control.** Reproduce the 686 control before judging any cost
   form: the base instruments (22/22 keys, nine native differences,
   four mirrors, boundary groups 9/2/6), the raw 19/20 frontier, the
   686 form's 20/20 pair rows (`none-two` none=2, `down` some=1,
   `faults=0`), the exact 18,304/18,896 stack rows, and the +9/call
   publication STOP at `resolution.rs:569`. A prior round's number is
   a claim, not evidence.
2. **The gate inventory.** For each disabled publication site, name
   with `path:line` the enabled gate that is executed on the disabled
   path, the publication it protects, and the owner/timing invariant
   it serves. The known sites are `resolution.rs:569`,
   `eval_core/call.rs:41,42,94,95,388,448` and the
   `publish_stashed_phases` gate at `profiling/logical/mod.rs:991`;
   re-derive them from the 686 tree, do not inherit the list.
3. **The baseline.** Persist the 686 form's per-address cost panel
   (two builds, two draws) as the control for every candidate. No cost
   row is waived because it is negative or looks like layout.

## Stage B — the cost form

4. **Design.** Remove or absorb the publication gates on the disabled
   path. Constraints:
   - the publication still happens at the same logical point and with
     the same owner; it may not be dropped, postponed past its owner,
     or merged into a path that loses an open record;
   - the enabled path's behaviour, order and exactly-once completion
     stay byte-identical;
   - no guest carrier, typed adapter, `LegacyMode`, transport, pushed
     value or one of the eight definition finishes changes;
   - no new evaluator-frame field, no selector, no outlined adapter,
     no +64 publish, no new `unsafe`;
   - the disabled path executes only the queued path's own standing
     start/record calls. A gate that exists only to skip profiling work
     is not a standing gate.
5. **Candidates.** At most two cost candidates in this round, each
   starting from the byte-identical 686 form. For each candidate:
   - first re-run the full enabled battery (20/20, `faults=0`, queue,
     flush, lifecycle, mirrors, Throw, keys) and the exact stack;
   - only then measure the per-address panel with two independent
     CGU=1 builds and two draws, serial, CPU-pinned, release, every
     `RUM_PROFILE_*` cleared;
   - record every delta row, including regressions and negative
     values; the green 686 panel is the control.
   A candidate that is red on the enabled battery is recorded and
   stopped; do not cost-measure it. The two candidates are independent:
   a red on the first does not authorize a second placement-style
   change outside the cost scope, but a second *cost* candidate is
   allowed.
6. **The answer.** If a candidate makes every publication row
   attributable or zero, keep its full enabled + stack + cost
   evidence. If neither candidate can do that without losing a
   publication's record, name the failing site with `path:line`, the
   invariant it serves and the measured cost, and stop with the
   smallest correct alternative (for example a redesign of the
   publication's gate placement) for the next round. Do not waive a
   row.

## Stage C — acceptance on the green cost form

7. **Enabled.** Queue 9/9 with parser quiescence `(0,0,0,0,0)` and
   strong-count ownership equality; slot 2/2; record 6/6; direct
   definition flush `Outcome::Some`, `faults=0`, counter
   `[1,1,0,1,0,0,0,0,0,0]`, quiescent; six lifecycle actions
   `open=0 leases=0 fabricated_ok=0`; all 20 pairs `native_same=true`
   `faults=0`; mirrors 67/67; seven Throw counter-only kills; all 150
   key rows with every prior PASS restored. Assertions and probes stay
   byte-identical; a red stays red.
8. **Stack.** Function 18,304 / Module 18,896 B per level, residual 0,
   two independent CGU=1 builds and two draws, four heights twice
   each, on the same form that passed the battery.
9. **Cost.** Every disabled delta is attributed per site to the queued
   path's own start/record calls in the same run, or removed. The
   publication rows at `resolution.rs:569` and `eval_core/call.rs`
   must be zero or attributable; the `builtin_catalog` lookup
   variation must be qualified by a layout control or a same-build
   repeated comparison, not by a source argument. No netting of
   unrelated negative deltas against positive cost.
10. **If Stage C is green** — do not run Stage C's downstream
    obligations (mispair, the 15 key routes, the contract fixtures) in
    this round; record the green cost form and draft the next round
    for them. If it is red, record the measured exclusion and stop.

## Goal

When this round is done, the design record states, with measurement
and not argument:

1. **The cost answer.** Every disabled publication gate either
   removed/absorbed with the publication preserved, or a measured
   impossibility with the exact site, the invariant, and the smallest
   correct alternative.
2. **The control.** The 686 enabled 20/20 and exact stack are
   reproduced before any candidate is judged.
3. **The candidates.** At most two, each with its enabled/stack/cost
   evidence, no row dropped, no negative netted.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the cost shape, its invariants, the
   gate tests and mutants, the ripple inventory, risks and open
   questions.

## Context pointers

- `.temp/brief-686/report-design.json`, sections `answer`, `cost`,
  `battery`, `plan`, `risks`; `delta-table.json`,
  `address-cost-results.json`, `stack-results.json`,
  `battery-results.json`, `clean-view.patch`, `decoded-view.patch`,
  `drafts/implementation-lp08-residual-10.md` — the state this round
  continues. Read-only; copy before adapting a script.
- `.temp/brief-686/src/form/` — the enabled-green, exact-stack
  control. `.temp/brief-686/src/raw_form/` — the 684 raw frontier.
  `.temp/brief-686/src/collector/` — the 682 collector reference.
- `.temp/director/brief-686-gate.md` — the enabled/stack acceptance
  and the cost STOP.
- `.temp/director/brief-682-gate.md` — the collector form at 19/20,
  the +64 root cause, the pair bar.
- `docs/dev/profiling.md` — the contract.
- Live sites: `crates/rum-runtime/src/runtime/resolution.rs`,
  `eval_core/call.rs`, `profiling/logical/{mod,collector}.rs`,
  `profiling/timing.rs`, `suspension/core/{profile_mode,
  profile_queue}.rs`, `suspension/typed/{registry,channel,legacy,
  adapter}.rs`, `suspension/primitive.rs`, and the queue tests.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-687/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control
  command;
- `.temp/brief-686/` or any other retained root, except to read and
  copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies,
  `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs
and probes; it repairs nothing.

## Hard bars

- **The disabled cost bar.** The default path pays only the queued
  path's own standing `runtime_path_timing_start` /
  `record_profile_runtime_path_timing` gate, measured in the same run.
  A publication-only gate is not allowed, even at +1. A negative
  delta is not a waiver.
- **Publication semantics.** A publication may not be dropped,
  postponed past its owner, or merged into a path that loses an open
  record. The owner/epoch/control invariant it serves stays stated
  with `path:line`.
- **The enabled form.** 20/20, `faults=0`, the unchanged battery and
  the exact 18,304/18,896 stack in two independent CGU=1 builds and
  two draws. No assertion or fixture is weakened.
- **The excluded forms.** The 683 observe-only guard, the 684
  position-only shape, the 685 ambient-owner guard and the panicking
  take-path fallback stay excluded. The 686 decoded view is the
  control and must not be changed except for the cost shape.
- **No hidden state.** No new evaluator-frame field, no selector, no
  outlined adapter, no +64 publish, no new `unsafe`. The decoded
  `Box`/`Expr` copy is enabled-only and charged; it must not appear in
  a disabled slope.
- Every count, key, route and `path:line` is re-derived from the 686
  tree; a brief-686 number is a claim, never evidence.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's
cargo runs plain inside the dispatcher's 16G scope (never nested).
Build both arms at one path with one release codegen unit; two
independent builds and draws for every cost row; serial Callgrind,
CPU-pinned, release, every `RUM_PROFILE_*` cleared except the declared
modes; four stack heights twice each with the residual printed. A
timeout is not data. Persist raw output before parsing; derive any
bound from data.

## Checkpointing

All scratch under `.temp/brief-687/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-687/report-design.json` in `rum-report/1`, with the
  cost answer, the two-candidate evidence (or the measured
  impossibility), the green enabled/stack battery and every delta
  row.
- `.temp/brief-687/drafts/implementation-lp08-residual-11.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 687` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
