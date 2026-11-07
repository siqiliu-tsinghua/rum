# Brief 684 — LP08 residual: the down-values marker at its consumer frontier

**Design continuation for the live tree: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. This round continues the
same design campaign as briefs 676–683; their reports are the starting
point, not evidence to re-derive from zero.**

Brief 683 measured why the observe-only discrimination is excluded:
the stage's `Result<Option<Expr>, EvalError>` is consumed at
`eval_core/call.rs:510` (typed consumers at `:603/:604` and
`:667/:668`), the queue removes the boundary cell before `observe`
(`suspension/core/profile_queue.rs:352/357`), and the value that
arrives there is a later expression (`Value(f[2])`, `Value(7)`), not
the stage's own Option. A guard at that position either mislabels or
abandons the row. The 682 collector remains the reference at 19/20.

This round answers one question with measured evidence: **can the
down-values marker be placed at its actual consumer frontier — bound
to the production consumer cell, not a test-only scan — so the
existing `OptionalExpr(Some/None)` arm records the stage's own Option
exactly once, `none-two` is none=2 and `down` is some=1 on both sides,
and the full enabled battery is 20/20?**

## Stage A — the frontier and the battery

1. **Calibration.** On the retained trees, reproduce the 683
   observation before judging any placement: the observer trace
   (`CONSUME_DOWN stage_option=None` then `channel=Value(f[2])` twice;
   `Some` then `Value(7)` once), the collector's `none-two` red row
   (`[2,2,0,0,2,...]` vs `[2,2,0,2,0,...]`), and the base instruments
   (`queued_keys=22 resumed_keys=22 native_diffs=9`, the four mirrors,
   boundary groups 9/2/6). The 683 receipts are targets, not evidence.
2. **The identity source.** Name, with `path:line`, where the
   production queue retains the consumer `CellId` for a paused call
   stage (`profile_queue.rs:197` is the insertion site; follow the
   cell's lifetime), how it survives nested and multiple pauses, and
   how epoch, owner generation and arena membership are validated
   before use. The test-only `arena.rs:270` ordered scan is not a
   production identity source. An identity source that requires a new
   hot-frame field, an arena traversal, or a guest-carrier change is a
   STOP with the measured reason.
3. **The placement.** Start from the unmodified 682 collector. Insert
   the down-values boundary so the existing nominal arm
   (`logical/mod.rs:1163`) receives the stage's actual
   `OptionalExpr(Some/None)` before the dispatch consumer runs; the
   guest frame, the pushed values and the completion order stay
   byte-identical. This is the director-approved sampling-position
   exception for this one diagnostic marker; state it explicitly and
   keep every other phase at its current position. Keep the
   session-less `drain_attached` (`timing.rs:633`), the
   `InputRequired` handling (`timing.rs:653`) and the attach/retire
   lifecycle. No new state in an evaluator frame, no selector, no
   outlined adapter, no +64 publish.
4. **The battery.** Run the whole enabled battery with brief 682/683's
   probes, fixtures and assertions byte-identical:
   - `queue_boundaries` 9/9, including the parser-error quiescent
     `(0,0,0,0,0)` and the strong-count ownership equality;
   - direct definition finish `Outcome::Some`, `faults=0`, counter
     `[1,1,0,1,0,0,0,0,0,0]`, then quiescent;
   - cancel, reset, drop and invalid leave `open=0`, `leases=0`,
     `fabricated_ok=0`;
   - all 20 pair cases, each `native_same=true`, `faults=0`;
     quote the `none-two` and `down` verdict lines verbatim; the
     probe's own PASS does not waive `native_same`;
   - mirrors `67/67`; the seven Throw mutants killed at the
     per-invocation counter with `semantics_enabled=false`; the 150
     key rows.
5. **The answer.** If Stage A is green on every line, go to Stage B.
   If it is red, name the failing control with `path:line`, the
   record/state that trips it and the invariant it violates, record
   the measured exclusion, and **stop** — do not try a second
   placement in this round.

## Stage B — only if Stage A is green on every control

6. **The stack campaign.** Function and Module recursive stack equal
   base exactly: 18,304 / 18,896 B per level, residual 0, in two
   independent CGU=1 builds and two draws, four stack heights twice
   each. The form must be green on Stage A in both builds.
7. **The disabled deltas.** Every delta on the measured tree is
   attributed per site to the queued path's own
   `runtime_path_timing_start` / `record_profile_runtime_path_timing`
   calls in the same run, or removed. No new delta beyond the standing
   per-site gate.

## Stage C — only if Stage B is green

8. **Mispair.** Make the three mispair mutants turn the exact counter
   red with `semantics_enabled=false` without changing the eight
   call-site signatures' observable finish. If that cannot be done,
   return a measured impossibility with the exact reason and a
   proposed falsifier; do not drop the row silently.
9. **The 15 open key rows.** Exercise each at pause, error and
   outcome, or record source-impossibility with `path:line`. The 12
   `phase_absent` rows stay open unless a fixture exercises the route;
   `set.finish` pause, `catch.resume_after_body` pause and
   `catch.eval.total` error need a probe, not an argument.
   `evaluate.total` pause must not return to `open=1`. The four
   `evaluate_atom_fast_path.stable_symbol` / `.atom` pause and error
   rows are source-impossible at `eval_core/mod.rs:174` and stay
   recorded as such.
10. **The contract obligations.** State, with `path:line`, how the
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

1. **The frontier.** The exact production consumer identity source,
   its lifetime, its validation, and the insertion point, with
   `path:line`; or the measured impossibility of the placement with
   the reason.
2. **The pair.** 20/20 `native_same=true` with `faults=0`, `none-two`
   none=2 and `down` some=1 on both sides, every verdict line quoted —
   or the measured exclusion with the failing control.
3. **The battery.** Every Stage A control green in the same round,
   assertions byte-identical.
4. **The cost rows.** Stack 18,304/18,896 residual 0 in two
   independent CGU=1 builds and two draws on the form that is green,
   and the disabled slopes attributed per site.
5. **The rest.** The mispair kill or its measured impossibility; every
   key row closed or OPEN with a director question; the contract
   obligations recorded as probed or OPEN.
6. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions
   — plus the updated drafted implementation brief in `TEMPLATE.md`
   shape, replacing the brief-683 draft.

If the frontier placement cannot satisfy the pair at the exact stack,
the record is the measured exclusion with its exact rows and the
minimum-cost alternative; the collector stays the reference at 19/20.
Promotion stays blocked until the implementation stage proves every
bar.

## Context pointers

- `.temp/brief-683/report-design.json` and sections `answer`, `root`,
  `plan`, `battery`; `.temp/brief-683/analysis-mirrors.log`,
  `analysis-battery-verdicts.log`, `analysis-source2.log`,
  `observer-setup.log`, `runs/observer-down/raw.log`,
  `runs/observer-none-two/raw.log`, `runs/b3-pair-none-two/raw.log`,
  `runs/b3-pair-down/raw.log`,
  `drafts/implementation-lp08-residual-7.md` — the state this round
  continues. Read-only; copy before adapting a script.
- `.temp/brief-683/src/collector/` — the unmodified 682 collector;
  the starting tree. `.temp/brief-683/src/observer/` — the two
  `#[cfg(test)]` trace sites; a read-only instrument, not a form.
  `.temp/brief-683/src/form/` — the excluded guard; do not start
  from it.
- `.temp/director/brief-683-gate.md` — the exclusion and the Q1
  exception.
- `.temp/director/brief-682-gate.md` — the collector form at 19/20,
  the +64 root cause, the pair bar.
- `.temp/director/brief-681-gate.md`, `brief-680-gate.md`,
  `brief-679-gate.md` — the publish necessity, the attach sequence,
  the standing per-site gate and the excluded forms.
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

Files you MAY write: anything under `.temp/brief-684/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control
  command;
- `.temp/brief-683/`, `.temp/brief-682/` or any retained root, except
  to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies,
  `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs
and probes; it repairs nothing.

## Hard bars

- **The cost bar.** The disabled path pays only the standing per-site
  gate (measured against the queued path's own call in the same run);
  the recursive Function and Module stack rows equal base exactly
  (18,304 / 18,896 B per level, residual 0) in two independent CGU=1
  builds and two draws. No selection machinery; no off-by-default
  state in a hot frame; no new field in an evaluator frame. The
  excluded forms (selector, +64 session-in-generic, the +64 publish in
  `adopted_slow` and `complete_phase`, outlined adapter, +16/+37) stay
  excluded.
- **The sampling-position exception.** Only the down-values
  diagnostic marker may move to its actual consumer frontier; every
  other phase stays at its current position. The exception must be
  stated, and an implementation must record it in
  `docs/dev/profiling.md`.
- **The controls.** The parser test's assertions, the owner identity,
  the pair check and the guest completion order stay byte-identical.
  No assertion is weakened to make the form green; a red stays a red.
- The instruments are calibrated before any form is judged: the
  corrected nine native vectors, the four mirrors and the boundary
  controls reproduce the current tree.
- Every count, key, route and `path:line` is re-derived from the
  current source; a brief-680, -681, -682 or -683 number is a claim,
  never evidence.
- A form that changes a guest carrier, a pushed value, an adopted key,
  a sampling boundary or a token-first interface is rejected unless
  the director names the exception first. This brief names it for
  `call.stage.down_values` only.
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

All scratch under `.temp/brief-684/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-684/report-design.json` in `rum-report/1`, with the
  frontier identity source, the pair verdicts (`none-two`, `down`),
  the battery, the exact-stack answer, the cost rows, the mispair and
  key table, the contract obligations and every open question.
- `.temp/brief-684/drafts/implementation-lp08-residual-8.md` — the
  updated implementation brief in `TEMPLATE.md` shape, plus the probe
  trees, binaries and receipts.
- Run `python3 -I scripts/report.py check 684` and return its summary
  and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
