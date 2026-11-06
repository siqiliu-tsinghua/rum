# Brief 679 — LP08 residual design continuation 2: reuse the queued instrumentation, completion outside the frame, and the open controls

**Design continuation for the live tree: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. This round continues the
same design campaign as briefs 677 and 678; their reports are the
starting point, not evidence to re-derive from zero.**
Brief 678 measured four completion families and found none that meets
the cost bar with the completion duties: the fused selector costs
+14 self-instructions in the fixed-point resume (97 versus 83), the
outlined form costs 84+38, full static completion pays +16 B/level in
Function and Module (18,320/18,912), and `callonly` shrinks the frame
to 18,256/18,848, which is still not exact. It also closed real
ground: the parser control is green on the rules form with its
assertions byte-identical, seven kernel reset probes pass, the seven
Throw wrong-outcome mutants turn the per-outcome equality red, and
nine previously unobserved keys now have observed fixture rows. The
direct flush is still red, `HostSession::reset` was not called,
`modonly` was not built, and 41 keys were not routed.

The director's gate (`.temp/director/brief-678-gate.md`) accepted the
record as a STOP and refined the instruction ruling: the four mirror
phases are observations the resumed path must make, and each one
legitimately costs the standing cold load. What is rejected is the
new generic `M::start/finish` mechanism, not the existence of the
observation. The queued path already observes exactly these phases
through `runtime_path_timing_start` and
`record_profile_runtime_path_timing`; the resumed path must pay the
same standing per-site gate, not a selector on top of it.

This round answers one question with measured evidence: **what exact
form restores the four resumed mirror observations and completes the
paused outer phases at the standing per-site gate and the exact
recursive stack, with the parser rule, the host-reset probe, the
direct flush and all 50 route keys closed — or what measured
exclusion of every remaining family blocks it?**

## Goal

When this round is done, the design record states, with measurement
and not argument:

1. **The mirrors by reuse.** The resumed fixed-point loop and the
   resumed append site observe the four mirror phases
   (`evaluate.fixed_point_equal`, `evaluate.stability_check`,
   `evaluate.trace_update`, `prepare_arguments.append_arg`) through the
   same instrumentation calls the queued path uses at the same points
   (`runtime_path_timing_start` + `record_profile_runtime_path_timing`,
   live `resolution.rs:358,403,410` and
   `eval_call_prepare.rs:321,345`), not through a new selector or
   trait. Measure the disabled cost per site against the queued path's
   own disabled cost for the same call, in the same run; the delta must
   be the standing inline gate and nothing more. Reproduce the
   corrected nine native vectors and the four mirrors on the valid
   fixtures with `20` queued / `20` resumed keys and the `67/67`
   synchronous comparisons re-derived this round. If a selection is
   still unavoidable, it must be a call-site branch to two monomorphic
   bodies with the disabled body unchanged, measured beside the
   queued-path control; the 97 fused selector and the 84+38 outline
   stay rejected.
2. **Completion at the exact stack.** Measure `modonly` (the
   suspension overlay plus `logical/mod.rs` alone) in two independent
   CGU=1 builds and two draws to localize the +16 B/level. Then design
   the completion form whose disabled path pays only the standing
   per-site gate and whose recursive Function and Module stack equal
   base exactly (18,304 / 18,896 B per level, residual 0) in two
   builds and two draws. Off-by-default state does not enter a hot
   frame: completion state lives in the queue entry, the epoch side or
   an already-cold structure, not in the evaluator's recursive frame.
   Every measured family that fails stays excluded with its rows.
3. **The direct flush green.** A direct definition finish with no
   enclosing phase records `Outcome::Some` with `faults=0` and the
   required counter `[1,1,0,1,0,0,0,0,0,0]`, with the eight
   call-site signatures and the token-first interface unchanged. Only
   then may the three mispair mutants be credited as kills; they must
   execute and turn the exact counter red with semantic equality
   disabled against that green control.
4. **The host reset probe.** Construct a real `HostSession`, suspend,
   call `HostSession::reset` (`crates/rum-host/src/session.rs:176`),
   and prove old-epoch markers cannot publish into the new epoch, no
   queued frame is discarded, there are no suffix effects, and the
   retirement is exactly once. The seven kernel probes that passed on
   `reset_for_lease` are not this probe.
5. **All 50 keys routed.** Every adopted `PhaseKey` (`logical/mod.rs:706`
   through `:811`) exercised individually at its pause, error and
   outcome routes, with the fixture calibrated for each; the numeric
   `Less` While plan stays rejected. A key left unrun is OPEN, not
   closed. A director ruling is requested for any row the form cannot
   cover.
6. **The parser rule, exactly.** Ship the terminal-parent clause
   measured in brief 678 (closed, unsettled, children > 0 returns
   `None` without a fault). The `!epoch.is_valid()` clause ships only
   with a probe that fails without it; the kernel probes ran with both
   clauses, so they do not justify it. Every existing assertion, owner
   identity, the pair check and the guest completion order stay
   byte-identical.
7. **The outcome assertion.** The seven Throw wrong-outcome mutants
   keep the per-invocation queued-versus-resumed equality as their
   killing line with semantic equality disabled; keep both asserts and
   do not fall back to an aggregate counter.
8. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions —
   plus the updated drafted implementation brief in `TEMPLATE.md`
   shape, replacing the brief-678 draft.

If a family has no form under the cost bar, the record is a measured
exclusion of that family with its exact rows; an unmeasured family is
OPEN, and "impossible" is not a result. Promotion remains blocked
until the implementation stage proves all of it.

## Context pointers

- `.temp/brief-678/report-design.json` and its sections `answer`,
  `cost`, `parser`, `mutants`, `plan`; `.temp/brief-678/runs/`,
  `analysis-678.log`, `callonly.log`, `measure-followup.log`,
  `drafts/implementation-lp08-residual-2.md` — the state this round
  continues. Read-only; copy before adapting a script.
- `.temp/director/brief-678-gate.md` — the refined instruction ruling
  and the answers to Q1–Q6.
- `.temp/director/brief-677-gate.md` — the cost bar and the rulings
  this campaign still stands on.
- `docs/dev/profiling.md` — the contract.
- Live sites: `crates/rum-runtime/src/runtime/resolution.rs`
  (`evaluate_profiled_fixed_point`, `resume_fixed_point_from`),
  `resolution/resume_frames.rs`, `eval_call_prepare.rs`,
  `eval_core/call.rs`, `profiling/{config,timing}.rs`,
  `profiling/logical/{mod,segments,queue_api,epoch,completion}.rs`,
  `suspension/core/{driver,profile_mode,profile_queue}.rs`,
  `suspension/typed/adapter.rs`, and the queue-boundary tests.
- Host reset: `crates/rum-host/src/session.rs:176`.
- `.temp/brief-621/MANIFEST-DESIGN.md` — the promotion model.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-679/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-678/`, `.temp/brief-677/` or any retained root, except
  to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs
and probes; it repairs nothing.

## Hard bars

- **The cost bar (restated with the ruling above).** The disabled path
  pays only the standing cold-load gate at each observation site; the
  recursive Function and Module stack rows equal base exactly
  (18,304 / 18,896 B per level, residual 0) in two independent
  CGU=1 builds and two draws; each disabled delta is attributed to a
  measured control — the queued path's own call for the mirrors, a
  gate-only arm for any selection form. Off-by-default state in a hot
  frame is a violation even at zero CPU. The measured 97 fused
  selector, 84+38 outline, +16 B/level static completion, and
  18,256/18,848 callonly stay excluded unless a new measurement
  overturns them.
- The instruments are calibrated before any form is judged: the
  corrected nine native vectors, the four mirrors and the boundary
  controls reproduce the current tree. A probe that cannot reproduce
  the finding cannot judge a form.
- Every count, key, route and `path:line` is re-derived from the
  current source; a brief-678 number is a claim, never evidence.
- The parser control, its assertions and the guest completion order
  stay as they are; a form that needs a changed assertion is rejected.
- A form that changes a guest carrier, a pushed value, an adopted key,
  a sampling boundary or a token-first interface is rejected unless
  the director names the exception first.
- No live source edit, no version-control command, no network, no new
  `unsafe`, no manifest or workload change.

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

All scratch under `.temp/brief-679/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-679/report-design.json` in `rum-report/1`, with the
  measured cost rows, the mirror-reuse measurements, the completion
  form and its exclusions, the direct flush, the host-reset probe, the
  50-route plan and every open question.
- `.temp/brief-679/drafts/implementation-lp08-residual-3.md` — the
  updated implementation brief in `TEMPLATE.md` shape, plus the probe
  trees, binaries and receipts.
- Run `python3 -I scripts/report.py check 679` and return its summary
  and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
