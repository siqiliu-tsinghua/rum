# Brief 680 — LP08 residual correctness continuation: the gated form's enabled path at the exact stack

**Design continuation for the live tree: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. This round continues the
same design campaign as briefs 677–679; their reports are the starting
point, not evidence to re-derive from zero.**
Brief 679 settled the cost shape of both halves. The mirror half works
by reuse: the resumed sites call the queued path's own
`runtime_path_timing_start` and `record_profile_runtime_path_timing`,
the disabled delta is exactly 39 gate instructions (3 start gates of 7
and 3 record gates of 6, the same sequence the queued path executes),
the body delta is −2, and the recursive stack is exact in two builds
and two draws. The completion half now has an exact-stack shape: the
`gated` tree keeps 18,304 / 18,896 with the completion state outside
the frame, and `modonly`/`coldphase` confirm that putting `session`
inside generic `complete_phase` instead costs +64 B/level and is
excluded. What remains is correctness: the `gated` form's enabled path
retires `InstrumentationFault` (resumed pair faults=5, the outcome
control dies with 13 faults), the direct flush is red, the parser
control fails quiescence with `(1,1,1,0,0)`, the queue-owner test
fails, and cancel leaves `evaluate.total` open.

This round answers one question with measured evidence: **what exact
lifecycle makes the gated form's enabled path green — zero faults on
every control, the parser control 9/9 with quiescence
`(0,0,0,0,0)`, the queue-owner and cancel rows green, and the direct
flush recording `Outcome::Some` — while the disabled cost stays the
queued path's per-site gate and the recursive stack stays exactly
18,304 / 18,896 B per level, residual 0, in two independent builds and
two draws?**

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **Every `InstrumentationFault` root-caused.** Each `epoch.fault()`
   the enabled path reaches is named with `path:line`, the invariant it
   enforces, the record/path/state that trips it, and the exact
   lifecycle change that satisfies it. No fault suppression, no
   widened invariant, no swallowed error. The measured fault sites to
   start from: the pair probe's resumed `faults=5`
   (`.temp/brief-679/runs/check-pair-native/raw.log:316`), the outcome
   control's 13 faults before the equality assert, `Call::pause`
   returning `None` after a profiled `Input[]`, and the direct flush's
   `InstrumentationFault`. The enabled path proves `faults=0` on every
   control this round runs.
2. **The parser control green, exactly.** All 9
   `queue_boundaries` tests pass with their assertions byte-identical;
   `logical_after_consumer_sees_the_input_parser_error` quiesces at
   `(0,0,0,0,0)`. The terminal-parent clause ships; the retained
   terminal parent is settled exactly once with its actual observed
   outcome before restore returns `None`. The `!epoch.is_valid()`
   clause does not ship (director ruling: nothing fails without it).
3. **The queue ownership and cancel rows green.**
   `logical_queued_frames_have_one_owner_in_both_modes` passes; cancel
   leaves no `evaluate.total` open; reset, drop and invalid stay green
   with `open=0`, `leases=0`, `fabricated_ok=0`.
4. **The direct flush green before mispair credit.** A direct
   definition finish with no enclosing phase records `Outcome::Some`
   with `faults=0` and the counter `[1,1,0,1,0,0,0,0,0,0]`, with the
   eight call-site signatures and token-first interfaces unchanged.
   Only against that green control are the three mispair mutants
   credited; they must execute and turn the exact counter red with
   semantic equality disabled.
5. **The pair equal.** The corrected calibration (20 queued / 20
   resumed keys, the nine native differences) reproduces on the form;
   the 20 valid cases are SAME with `faults=0`; the 67/67 synchronous
   comparisons hold; every pair log's receipt is valid.
6. **The outcome kills.** The seven Throw wrong-outcome mutants turn
   the per-invocation queued-versus-resumed equality red with
   `semantics_enabled=false`; keep both asserts and keep the equality
   first.
7. **The 50 keys.** Every adopted `PhaseKey` exercised at pause, error
   and outcome. The director has ruled the four
   `evaluate_atom_fast_path.stable_symbol` / `.atom` pause and error
   rows source-impossible (`eval_core/mod.rs:168-181`: the
   `StableSymbol | Atom` arm returns `Ok(expr)` with no suspending call
   and no error return); record them as source-impossible with that
   citation, not as covered. Every other row closes or is OPEN with a
   director question. `phase_absent` and `faults=N` stay open.
8. **The cost unchanged by the fix.** The gated form re-measured at
   Function 18,304 and Module 18,896 B/level, residual 0, in two
   independent CGU=1 builds and two draws; the mirror deltas stay the
   queued path's per-site gate (native +37, argument +26, the 39/26
   gate instructions with the queued control measured in the same run);
   no selection machinery; no off-by-default state in a hot frame. Any
   correctness fix that grows the frame or adds a disabled instruction
   fails this round.
9. **The host reset unchanged.** `HostSession::reset` abandoning the
   suspended request is the host contract (director ruling);
   `session.rs:176` is not patched. The probe's exactly-once
   retirement, no fabricated `ok` and no suffix stand.
10. **The plan.** A step-by-step design with `path:line` evidence per
    step, the invariants each step preserves, the gate tests and
    mutants, the call-site/ripple inventory, risks and open questions —
    plus the updated drafted implementation brief in `TEMPLATE.md`
    shape, replacing the brief-679 draft.

If a fault cannot be satisfied without breaking a measured cost row or a
control, the record is a measured exclusion of that lifecycle with its
exact rows and the minimum-cost alternative; "impossible" is not a
result. Promotion remains blocked until the implementation stage proves
all of it.

## Context pointers

- `.temp/brief-679/report-design.json` and sections `answer`, `parser`,
  `host`, `plan`; `.temp/brief-679/analysis-679.log`,
  `analysis-gated.log`, `.temp/brief-679/measure/`,
  `.temp/brief-679/runs/check-*`, and
  `drafts/implementation-lp08-residual-3.md` — the state this round
  continues. Read-only; copy before adapting a script.
- `.temp/director/brief-679-gate.md` — the rulings on Q1–Q4 and the
  cost ruling.
- `.temp/director/brief-678-gate.md` — the standing cost bar.
- `docs/dev/profiling.md` — the contract.
- Live sites: `crates/rum-runtime/src/runtime/resolution.rs`,
  `resolution/resume_frames.rs`, `eval_call_prepare.rs`,
  `eval_core/{call,mod}.rs`, `profiling/{config,timing}.rs`,
  `profiling/logical/{mod,segments,queue_api,epoch,record,completion}.rs`,
  `suspension/core/{driver,profile_mode,profile_queue}.rs`,
  `suspension/typed/adapter.rs`, and the queue-boundary tests.
- Host reset: `crates/rum-host/src/session.rs:176`.
- `.temp/brief-621/MANIFEST-DESIGN.md` — the promotion model.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-680/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-679/`, `.temp/brief-678/` or any retained root, except
  to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing.

## Hard bars

- **The cost bar.** The disabled path pays only the standing per-site
  gate (measured against the queued path's own call in the same run);
  the recursive Function and Module stack rows equal base exactly
  (18,304 / 18,896 B per level, residual 0) in two independent CGU=1
  builds and two draws. No selection machinery; no off-by-default
  state in a hot frame; the excluded forms (selector, +64
  session-in-generic, outlined adapter, +16/+37) stay excluded.
- **The controls.** The parser test's assertions, the owner identity,
  the pair check and the guest completion order stay byte-identical. No
  assertion is weakened to make the form green; a red stays a red.
- The instruments are calibrated before any form is judged: the
  corrected nine native vectors, the four mirrors and the boundary
  controls reproduce the current tree. A probe that cannot reproduce
  the finding cannot judge a form.
- Every count, key, route and `path:line` is re-derived from the
  current source; a brief-679 number is a claim, never evidence.
- A form that changes a guest carrier, a pushed value, an adopted key,
  a sampling boundary or a token-first interface is rejected unless the
  director names the exception first.
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

All scratch under `.temp/brief-680/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-680/report-design.json` in `rum-report/1`, with the
  fault root-causes, the parser/ownership/cancel/flush rows, the pair
  and Throw results, the 50-key table, the cost rows and every open
  question.
- `.temp/brief-680/drafts/implementation-lp08-residual-4.md` — the
  updated implementation brief in `TEMPLATE.md` shape, plus the probe
  trees, binaries and receipts.
- Run `python3 -I scripts/report.py check 680` and return its summary
  and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
