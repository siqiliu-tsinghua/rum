# Brief 683 — LP08 residual: the resumed down-values outcome at the exact collector

**Design continuation for the live tree: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. This round continues the
same design campaign as briefs 676–682; their reports are the starting
point, not evidence to re-derive from zero.**

Brief 682's epoch-wide collector candidate is green on every
enabled-path control except one: pairs 19/20, `none-two` red. Its hot
prologues are identical to the exact form (`Session::evaluate`
`sub $0x378`, size `0x1066`; `prepare_arguments...` `sub $0x368`), and
the excluded publish's +64 is root-caused at the instruction level
(3 evaluate frames + 1 prepare frame, 16 B each). The only red row is
`call.stage.down_values`: queued none=2, resumed some=2, with `down`
some=1 on both sides. The gated publish binary — sha256-identical to
brief-680's build — prints the same miss, so restoring the publish is
not the minimum-cost fix. The round stopped as its brief required;
Stage B was not started.

This round answers one question with measured evidence: **can the
resumed down-values stage record the stage's own `Option` outcome so
`none-two` equals the queued path while `down` stays some=1, and does
the resulting collector form pass every control at the exact stack?**

## Stage A — the discrimination and the battery

1. **Calibration.** On the brief-682 collector tree, before judging
   the form, reproduce the base instruments: `queued_keys=22
   resumed_keys=22 native_diffs=9`, the four mirrors, and the
   `queue_boundaries` / `slot_boundary` / `record_engine` groups. A
   probe that cannot reproduce the finding cannot judge a form. The
   682 receipts are target values, not evidence.
2. **The one condition.** Change only the resumed classification: a
   logical row records the stage's own `Option` outcome, once — a hit
   is `Some`, a miss is `None`, and an inner input value is not that
   outcome. `down` must stay some=1 on both sides
   (`[1,1,0,1,0,0,0,0,0,0]` queued and resumed). A blanket
   `Slot::Value => None` is rejected. Do not change the eight
   definition call-site signatures, do not put `Session` back into
   `adopted_slow` or `complete_phase`, do not restore the publish,
   and add no selector, outlined adapter or off-by-default state in a
   hot frame.
3. **The battery.** Run the whole enabled-path battery with brief
   682's probes, fixtures and assertions byte-identical:
   - `queue_boundaries` 9/9, including
     `logical_after_consumer_sees_the_input_parser_error` quiescent
     `(0,0,0,0,0)` and the strong-count ownership equality;
   - a direct definition finish with no enclosing phase records
     `Outcome::Some`, `faults=0`, counter `[1,1,0,1,0,0,0,0,0,0]`,
     then quiesces;
   - cancel, reset, drop and invalid leave `open=0`, `leases=0`,
     `fabricated_ok=0`;
   - all 20 pair cases, each `native_same=true`, `faults=0`; quote
     the `none-two` and `down` verdict lines verbatim. The probe's own
     `PASS` line does not waive `native_same`;
   - mirrors `67/67`, counted;
   - the seven Throw wrong-outcome mutants killed at the
     per-invocation counter assert with `semantics_enabled=false`;
   - the 150 key rows.
4. **The answer.** If Stage A is green on every line, go to Stage B.
   If it is red on any line, name the failing control with
   `path:line`, the record/state that trips it and the invariant it
   violates, record the measured exclusion, and **stop** — do not try
   a second discrimination in this round.

## Stage B — only if Stage A is green on every control

5. **The stack campaign.** Function and Module recursive stack equal
   base exactly: 18,304 / 18,896 B per level, residual 0, in two
   independent CGU=1 builds and two draws, four stack heights twice
   each. The form must be green on Stage A in both builds.
6. **The disabled deltas.** Every delta on the measured tree is
   attributed per site to the queued path's own
   `runtime_path_timing_start` / `record_profile_runtime_path_timing`
   calls in the same run, or removed. No new delta beyond the standing
   per-site gate.

## Stage C — only if Stage B is green

7. **Mispair.** Make the three mispair mutants turn the exact counter
   red with `semantics_enabled=false` without changing the eight
   call-site signatures' observable finish. If that cannot be done,
   return a measured impossibility with the exact reason and a
   proposed falsifier; do not drop the row silently.
8. **The 15 open key rows.** Exercise each at pause, error and
   outcome, or record source-impossibility with `path:line`. The 12
   `phase_absent` rows stay open unless a fixture exercises the
   route; `set.finish` pause, `catch.resume_after_body` pause and
   `catch.eval.total` error need a probe, not an argument.
   `evaluate.total` pause must not return to `open=1`. The four
   `evaluate_atom_fast_path.stable_symbol` / `.atom` pause and error
   rows are source-impossible at `eval_core/mod.rs:174` and stay
   recorded as such.
9. **The contract obligations.** State, with `path:line`, how the
   collector meets `docs/dev/profiling.md`: one enabled, process-wide
   epoch collector; cross-thread Session resume without losing
   records; epoch isolation (unrelated requests out of a program's
   vector); reset retirement; nested/interleaved owners. Probe what a
   fixture can exercise; record the rest as open obligations for the
   implementation stage. `current_slot()` is process-wide and the
   bearer of that rule; do not claim it passed unprobed.

## Goal

When this round is done, the design record states, with measurement
and not argument:

1. **The pair.** 20/20 `native_same=true` with `faults=0`, `none-two`
   included, `down` still some=1 on both sides, every verdict line
   quoted — or the measured exclusion with the failing control.
2. **The battery.** Every Stage A control green in the same round,
   assertions byte-identical.
3. **The cost rows.** Stack 18,304/18,896 residual 0 in two
   independent CGU=1 builds and two draws on the form that is green,
   and the disabled slopes attributed per site.
4. **The rest.** The mispair kill or its measured impossibility; every
   key row closed or OPEN with a director question; the contract
   obligations recorded as probed or OPEN.
5. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions
   — plus the updated drafted implementation brief in `TEMPLATE.md`
   shape, replacing the brief-682 draft.

If the discrimination cannot satisfy the pair at the exact stack, the
record is the measured exclusion with its exact rows and the
minimum-cost alternative; "the collector is the form" stays a claim
until the stack campaign exists. Promotion stays blocked until the
implementation stage proves every bar.

## Context pointers

- `.temp/brief-682/report-design.json` and sections `answer`, `root`,
  `battery`, `plan`; `.temp/brief-682/analysis-cal.log`,
  `analysis-battery.log`, `analysis-mirrors.log`,
  `analysis-prologues.log`, `analysis-frames.log`,
  `analysis-frame-subs.log`, `analysis-prepare-spill.log`,
  `analysis-stack-old.log`, `analysis-gated-none-two.log`,
  `runs/b3-*`, `drafts/implementation-lp08-residual-6.md` — the state
  this round continues. Read-only; copy before adapting a script.
- `.temp/brief-682/src/collector/` — the candidate. Probe source, not
  the live tree. `logical/mod.rs` (`finish_logical_optional` at 271,
  `ProfileBoundary::observe` at 1159, the state arm at 1163-1168),
  `logical/collector.rs` (the process-wide stash and drain),
  `profiling/timing.rs` (the drain at the end of `adopted_slow`),
  `suspension/core/profile_queue.rs` (owner registration and
  attach).
- `.temp/brief-682/ref/gated/check` — the sha256-identical 680 gated
  binary; its `none-two` miss is the control.
- `.temp/director/brief-682-gate.md` — the rulings on Q1–Q3, the
  form, the contract and the cost.
- `.temp/director/brief-681-gate.md` — the publish necessity and the
  pair bar.
- `.temp/director/brief-680-gate.md` — Q2's attach sequence and the
  cost ruling.
- `.temp/director/brief-679-gate.md` — the standing per-site gate and
  the excluded forms (selector, +64 session-in-generic, outlined
  adapter, +16/+37).
- `docs/dev/profiling.md` — the contract.
- Live sites: `crates/rum-runtime/src/runtime/profiling/logical/
  {mod,segments,queue_api,epoch,record,completion,totals}.rs`,
  `profiling/{config,timing}.rs`,
  `crates/rum-runtime/src/runtime/resolution.rs`,
  `resolution/resume_frames.rs`, `eval_call_prepare.rs`,
  `eval_core/{call,mod}.rs`,
  `suspension/core/{driver,profile_mode,profile_queue}.rs`,
  `suspension/typed/adapter.rs`, and the queue-boundary tests.
- Host reset: `crates/rum-host/src/session.rs:176`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-683/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control
  command;
- `.temp/brief-682/`, `.temp/brief-681/`, `.temp/brief-680/` or any
  retained root, except to read and copy. Never execute a file in
  place;
- the public API, features, serde, manifests, dependencies,
  `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix,
the census or inventory replays, or any source gate. This round
designs and probes; it repairs nothing.

## Hard bars

- **The cost bar.** The disabled path pays only the standing per-site
  gate (measured against the queued path's own call in the same run);
  the recursive Function and Module stack rows equal base exactly
  (18,304 / 18,896 B per level, residual 0) in two independent CGU=1
  builds and two draws. No selection machinery; no off-by-default
  state in a hot frame; the excluded forms (selector, +64
  session-in-generic, the +64 publish in `adopted_slow` and
  `complete_phase`, outlined adapter, +16/+37) stay excluded.
- **The controls.** The parser test's assertions, the owner identity,
  the pair check and the guest completion order stay byte-identical.
  No assertion is weakened to make the form green; a red stays a red.
- The instruments are calibrated before any form is judged: the
  corrected nine native vectors, the four mirrors and the boundary
  controls reproduce the current tree.
- Every count, key, route and `path:line` is re-derived from the
  current source; a brief-680, -681 or -682 number is a claim, never
  evidence.
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

All scratch under `.temp/brief-683/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-683/report-design.json` in `rum-report/1`, with the
  pair verdicts (`none-two`, `down`), the battery, the exact-stack
  answer, the cost rows, the mispair and key table, the contract
  obligations and every open question.
- `.temp/brief-683/drafts/implementation-lp08-residual-7.md` — the
  updated implementation brief in `TEMPLATE.md` shape, plus the probe
  trees, binaries and receipts.
- Run `python3 -I scripts/report.py check 683` and return its summary
  and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
