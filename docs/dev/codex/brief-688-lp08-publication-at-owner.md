# Brief 688 — LP08 residual: publication at the owner

**Cost design on the brief-686 form: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. The 686 form is the
enabled-green, exact-stack control; brief 687's two cost forms are
excluded and are not bases.**

Brief 687 measured the publication cost panel on the 686 form: the
call-stage rows (+60 native / +30 argument, +17 prepared) and the
fixed-point row (+9/call, 6 publication gate + 3 other) are
publication-only, and two shape candidates made the fixed point worse
(+31 and +22) because LLVM outlined the closure and the witness. The
call-stage finding is real: with the pause moved to the caller and
fused with publication behind the one existing pause gate, those rows
return to base. The fixed-point site keeps two publications in one
`#[cold]` helper, so fusion there is impossible without changing the
pause/consumer order.

This round answers one question with measured evidence: **can
publication be moved out of every default-path call site into the
place where its owner is established — an enabled-only branch base
already executes — so that every former site returns to the base
instruction sequence at zero delta, while the enabled battery stays
20/20, the exact stack stays 18,304 / 18,896 with residual 0, and
every boundary still enters the queue at the same logical position?**

## Stage A — calibration

1. **Control.** Reproduce the 686 control before judging any
   placement: the base instruments (22/22 keys, nine native
   differences, four mirrors, boundary groups 9/2/6), the raw 19/20
   frontier, the 686 form's 20/20 pairs (`none-two` none=2, `down`
   some=1, `faults=0`), the exact 18,304 / 18,896 stack rows, and the
   +9/call fixed-point STOP. The 686 form tree must be reproduced
   byte-identically as `src/form`; a prior round's number is a claim,
   not evidence.
2. **The owner map.** Re-derive from the 686 tree, with `path:line`:
   - every pause site that can stash a boundary, and for each one
     whether a pending owner exists at that moment (`complete_phase`
     InputRequired path, `pause_phase_at_finish_slow`'s DownValues
     exception, the deferred-completion callers);
   - the exact point where `pending_suspension` becomes `Some`
     (`Session::begin_input_suspension` and every caller), and the
     existing enabled gate inside it;
   - the insertion rule that makes the queue position: `add` with
     `after = None` prepends, and the DownValues consumer binding
     (`insert_before_consumer`, the sampled-position exception);
   - every caller of `publish_stashed_phases`,
     `logical_insert_boundary` and `logical_insert_down_values_before_consumer`.
   The six known site calls are `resolution.rs:569,587` and
   `eval_core/call.rs:42,66,95,124` in the 686 form. Re-derive them;
   do not inherit the list.
3. **The baseline.** Persist the 686 form's per-address cost panel
   (two builds, two draws) as the control for every placement. No row
   is waived because it is negative or looks like layout.

## Stage B — the placement

4. **Design.** Move publication to the point where its owner is
   established, both points already behind gates base executes:
   - **Owner-present pause.** When a phase pauses with InputRequired
     and `pending_suspension` is already `Some`, flush the deferred
     definition boundaries and insert the paused boundary into that
     pending owner's queue at the position the site publication
     produces today (front, before the consumer), inside the
     enabled-only cold path that already exists.
   - **Owner created later.** When a boundary was stashed because no
     owner existed, publish it inside
     `Session::begin_input_suspension`'s existing
     `resume_profile_configured` branch, after the owner and its
     profile storage are installed and before any frame is pushed.
   - Remove the six site-level `publish_stashed_phases` calls. Do not
     move pause sites; the 686 pause/suspend order stays exactly where
     it is. The exact call structure is yours to select within these
     constraints.
5. **Constraints.**
   - A boundary must still enter the queue at the same logical
     position relative to every other cell (prepend at the front, the
     consumer-frontier rule for the DownValues marker, deferred
     definition flushes in order). Prove it with the cell trace, not
     by argument.
   - The owner/epoch/control/generation authentication, the
     exactly-once completion, the restore-in-order of
     foreign-control boundaries, and the direct definition flush stay
     unchanged.
   - An ownerless stash may not be dropped or attached to a foreign
     owner; it is published when its owner is established, in order.
   - The decoded `DownValuesView`, the canonical `Option<Expr>`
     decoding, the typed adapter, `LegacyMode`, the transport, every
     guest carrier and the eight definition finishes stay
     byte-identical.
   - No new evaluator-frame field, no session field carrying
     diagnostic state, no selector, no outlined adapter, no +64
     publish, no new `unsafe`.
   - The disabled path at every former site executes the base
     instruction sequence: **zero delta**, not "attributable". A
     publication moved into a callee is not a waiver.
6. **Candidates.** At most two placements, each starting from the
   byte-identical 686 form and each a genuinely different owner
   establishment point (not two variants of the same call structure).
   For each candidate:
   - first re-run the full enabled battery (20/20, `faults=0`, queue,
     flush, lifecycle, mirrors, Throw, keys) and the exact stack;
   - only then measure the per-address panel with two independent
     CGU=1 builds and two draws, serial, CPU-pinned, release, every
     `RUM_PROFILE_*` cleared;
   - record every delta row, including regressions and negative
     values; the green 686 panel is the control.
   A candidate red on the enabled battery is recorded and stopped; do
   not cost-measure it.
7. **Placement falsifiers.** Ship, and watch fire, one mutant per
   rule; a falsifier nobody has watched fail is not known to work:
   - **owner-loss:** the owner-created publish is skipped and the
     ownerless-stash fixture must show the boundary unobserved or
     faulted, not silently dropped;
   - **premature publication:** a boundary published before the owner
     is installed must attach to no owner or fault, never to a
     different owner;
   - **wrong-owner:** a stashed boundary whose control differs from
     the current request must remain stashed and be published later,
     never inserted now;
   - **exactly-once:** a boundary published at the owner point and
     again at a site must double its record or fault in the pair
     counters.
   Record each mutant's exact counter and its observed red.
8. **The +1 reload row.** The terminal argument branch
   (`resume_suspended_call_with_dispatch_plan`) still executes one
   extra `movq`: the restoration pointer is loaded twice. Remove it
   with the same rebuild and prove the removal, or return a measured
   impossibility naming the branch, the invariant it serves and the
   non-profiling opcode. Do not waive it as layout.

## Stage C — acceptance on the green cost form

9. **Enabled.** Queue 9/9 with parser quiescence `(0,0,0,0,0)` and
   strong-count ownership equality; slot 2/2; record 6/6; direct
   definition flush `Outcome::Some`, `faults=0`, counter
   `[1,1,0,1,0,0,0,0,0,0]`, quiescent; six lifecycle actions
   `open=0 leases=0 fabricated_ok=0`; all 20 pairs `native_same=true`
   `faults=0`; mirrors 67/67; seven Throw counter-only kills; all 150
   key rows with every prior PASS restored. Assertions and probes stay
   byte-identical; a red stays red.
10. **Stack.** Function 18,304 / Module 18,896 B per level, residual 0,
    two independent CGU=1 builds and two draws, four heights twice
    each, on the same form that passed the battery.
11. **Cost.** Every former publication site is zero versus base in the
    same run. Every remaining disabled delta is attributed per site to
    the queued path's own start/record calls, or removed. The
    `builtin_catalog` lookup variation is qualified by a layout control
    or a same-build repeated comparison, not by a source argument. No
    netting of unrelated negative deltas against positive cost.
12. **If Stage C is green** — do not run Stage C's downstream
    obligations (mispair, the 15 key routes, the contract fixtures) in
    this round; record the green cost form and draft the next round
    for them. If it is red, record the measured exclusion and stop.

## Goal

When this round is done, the design record states, with measurement
and not argument:

1. **The placement answer.** Publication moved to the owner
   establishment points and every former site at zero delta versus
   base, with the queue position proved by trace; or a measured STOP
   naming the site, the invariant and the smallest correct
   alternative.
2. **The control.** The 686 enabled 20/20 and exact stack reproduced
   before any candidate is judged.
3. **The candidates.** At most two, each with its enabled/stack/cost
   evidence, every arm reduced into the artifact, no row dropped, no
   negative netted.
4. **The plan.** The updated drafted implementation brief in
   `TEMPLATE.md` shape, with the placement, its invariants, the gate
   tests and the four calibrated mutants, the ripple inventory, risks
   and open questions.

## Context pointers

- `.temp/brief-687/report-design.json`, sections `answer`, `cost`,
  `battery`, `plan`, `risks`; `cost-answer.json`, `all-delta-rows.json`,
  `gate-inventory.json`, `call-sites.json`, `implementation-ripple.json`,
  `lookup-control-results.json`, `candidate1.patch`, `candidate2.patch`
  — the state this round continues. Read-only; copy before adapting a
  script.
- `.temp/brief-687/src/form/` — the byte-identical 686 control.
  `.temp/brief-687/src/base/`, `collector/`, `raw_form/` — the earlier
  references.
- `.temp/director/brief-687-gate.md` — the two exclusions and the
  ruling that the call-stage fusion is a technique, not a promotion.
- `.temp/director/brief-686-gate.md` — the enabled/stack acceptance.
- `docs/dev/profiling.md` — the contract.
- Live sites: `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`
  (`complete_phase`, `pause_phase_at_finish_slow`,
  `publish_stashed_phases`, `flush_definition_boundaries`),
  `profiling/logical/queue_api.rs`, `profiling/logical/completion.rs`,
  `suspension/core.rs` (`begin_input_suspension`),
  `suspension/core/storage.rs`, `suspension/core/profile_queue.rs`,
  `suspension/core/profile_mode.rs`, `suspension/typed/witness.rs`,
  `resolution.rs`, `eval_core/call.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-688/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control
  command;
- `.temp/brief-687/` or any other retained root, except to read and
  copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies,
  `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs
and probes; it repairs nothing.

## Hard bars

- **The disabled cost bar.** The default path pays only the queued
  path's own standing `runtime_path_timing_start` /
  `record_profile_runtime_path_timing` calls, measured in the same
  run. A publication-only gate is not allowed, even at +1, wherever it
  lives. A negative delta is not a waiver.
- **Publication semantics.** A publication may not be dropped,
  postponed past its owner, or merged into a path that loses an open
  record. The owner/epoch/control invariant it serves stays stated
  with `path:line`, and the position rule is proved by the cell trace.
- **The enabled form.** 20/20, `faults=0`, the unchanged battery and
  the exact 18,304/18,896 stack in two independent CGU=1 builds and
  two draws. No assertion or fixture is weakened.
- **The excluded forms.** The 683 observe-only guard, the 684
  position-only shape, the 685 ambient-owner guard, the panicking
  take-path fallback, and both 687 cost shapes stay excluded. The 686
  decoded view is the control and may be changed only for the
  placement.
- **No hidden state.** No new evaluator-frame field, no session field
  carrying diagnostic state, no selector, no outlined adapter, no +64
  publish, no new `unsafe`. The decoded `Box`/`Expr` copy stays
  enabled-only and charged; it must not appear in a disabled slope.
- **Every arm reduced.** Every candidate's stack and battery runs are
  reduced into the persisted artifact; a run that exists only as a log
  is not evidence.
- Every count, key, route and `path:line` is re-derived from the 686
  tree; a brief-687 number is a claim, never evidence.

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

All scratch under `.temp/brief-688/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-688/report-design.json` in `rum-report/1`, with the
  placement answer, the candidate evidence (or the measured
  impossibility), the four watched falsifiers, the green
  enabled/stack battery and every delta row.
- `.temp/brief-688/drafts/implementation-lp08-residual-12.md` — the
  updated implementation brief in `TEMPLATE.md` shape.
- Run `python3 -I scripts/report.py check 688` (without a
  version-control option) and return its summary and all compliance
  verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
