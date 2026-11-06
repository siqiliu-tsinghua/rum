# Brief 682 — LP08 residual: the epoch-wide synchronous-event collector at the exact stack

**Design continuation for the live tree: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. This round continues the
same design campaign as briefs 676–681; their reports are the starting
point, not evidence to re-derive from zero.**

Brief 681 measured the exact-stack candidate and stopped: build3 — the
brief-680 gated tree with the `adopted_slow` publish removed — is red
on the enabled path. Its calibration reproduced the base tree first
(`queued_keys=22 resumed_keys=22 native_diffs=9`; boundaries 9/2/6),
and then build3 failed parser quiescence `(1,1,1,0,0)` at
`epoch/snapshot.rs:17`, left `evaluate.total` open at cancel, printed
`native_same=false` on all 20 pairs, and panicked every outcome probe
at `(0,0,1,0,0)`. The publish is necessary; both +64 placements
(`adopted_slow`, `complete_phase`) stay excluded; the attach is the
open design question. The 676 Q2 authorization stands: design and
measure the **epoch-wide synchronous-event collector**, still a
separate design, now carrying the measured bar.

This round answers one question with measured evidence: **can the
epoch-wide synchronous-event collector deliver the logical completion
that the stashed-phase publish delivers — green on every enabled-path
control, at the exact stack, with no disabled delta beyond the
standing per-site gate?**

## Stage A — calibration, root cause, and the collector candidate

1. **Calibration.** On the exact starting tree, before any form is
   judged, reproduce the base instruments: `queued_keys=22
   resumed_keys=22 native_diffs=9`, the four mirrors, and the
   `queue_boundaries` / `slot_boundary` / `record_engine` boundary
   groups. A probe that cannot reproduce the finding cannot judge a
   form. Use brief 681's calibration receipts as the target values,
   not as evidence.
2. **The +64 root cause.** Disassemble the measured recursive
   frames at one release codegen unit, for the gated form (with the
   `adopted_slow` publish) and the exact form (without it), and name
   the bytes: the frame(s) the Function/Module stack probes measure,
   the prologue delta (`sub $0x388` versus `sub $0x378`), and the
   slots or spills that account for 64 B per level. A form that greens
   the path without knowing why the publish costs 64 bytes is a
   guess. Record the per-build prologue rows and the probe rows.
3. **The collector candidate.** Design the epoch-wide synchronous
   collector against `docs/dev/profiling.md` ("Logical collection uses
   one enabled, process-wide epoch collector"; an owner can resume
   its Session on another thread; TLS diagnostic maps may remain
   physical telemetry), and build it in a probe tree on the exact
   source. It must carry the stashed outer phase to its owner after
   the phase is stashed — the red controls below are the requirement
   list. Run the whole battery on the candidate, with brief 681's
   probes, fixtures and assertions byte-identical:
   - `queue_boundaries` 9/9, including
     `logical_after_consumer_sees_the_input_parser_error` quiescent
     `(0,0,0,0,0)` and the strong-count ownership equality;
   - a direct definition finish with no enclosing phase records
     `Outcome::Some`, `faults=0`, counter `[1,1,0,1,0,0,0,0,0,0]`,
     then quiesces;
   - cancel, reset, drop and invalid leave `open=0`, `leases=0`,
     `fabricated_ok=0`;
   - the 20 valid pair cases, each `native_same=true`, `faults=0`;
   - mirrors `67/67`, counted, not imported;
   - the seven Throw wrong-outcome mutants killed at the
     per-invocation counter assert with `semantics_enabled=false`;
   - the three mispair mutants executed and recorded;
   - the 150 key rows.
4. **The answer.** If the candidate is green on every line of stage
   A, state the collector form as the exact-stack candidate and go to
   stage B. If it is red on any line, name every failing control with
   `path:line`, the record/state that trips it, and the invariant it
   violates, record the candidate as a measured exclusion, and
   **stop** — do not design a second collector form or an attach in
   this round. A form that greens quiescence only by draining a queue
   at flush time while the resumed counters sit at the wrong
   ordinals is red on the pairs and the outcome rows; record it, do
   not ship it.

## Stage B — only if Stage A is green on every control

5. **The stack campaign.** Function and Module recursive stack equal
   base exactly: 18,304 / 18,896 B per level, residual 0, in two
   independent CGU=1 builds and two draws, four stack heights twice
   each. The form must be green on stage A in both builds.
6. **The disabled deltas.** Every delta on the measured tree is
   attributed per site to the queued path's own
   `runtime_path_timing_start` / `record_profile_runtime_path_timing`
   calls in the same run, or removed. No selection machinery; no
   off-by-default state in a hot frame; no new delta beyond the
   standing per-site gate.

## Stage C — only if Stage B is green

7. **Pair 20/20.** Keep the bar at 20/20 `native_same=true` with
   `faults=0`. The `none-two` difference (`call.stage.down_values`:
   queued none=2, resumed some=2) must close with a discriminating
   condition that keeps `down` some=1 on both sides; 19/20 is not
   accepted.
8. **Mispair.** Make the three mispair mutants turn the exact counter
   red with `semantics_enabled=false` without changing the eight
   call-site signatures' observable finish. If that cannot be done,
   return a measured impossibility with the exact reason and a
   proposed falsifier; do not drop the row silently.
9. **The 15 open key rows.** Exercise each at pause, error and
   outcome, or record source-impossibility with `path:line`. The 12
   `phase_absent` rows stay open unless a fixture exercises the
   route; `set.finish` pause, `catch.resume_after_body` pause and
   `catch.eval.total` error need a probe, not an argument.
   `evaluate.total` pause must not remain `open=1`. The four
   `evaluate_atom_fast_path.stable_symbol` / `.atom` pause and error
   rows are source-impossible at `eval_core/mod.rs:174` and stay
   recorded as such.

## Goal

When this round is done, the design record states, with measurement
and not argument:

1. **The collector answer.** Whether the epoch-wide synchronous
   collector is green on every enabled-path control, with every
   verdict line quoted; if not, every failing control root-caused
   with `path:line` and the round stopped with the measured
   exclusion and the minimum-cost alternative.
2. **The +64 root cause.** The measured slots or spills that account
   for 64 B per level, per frame, per build, with the prologue rows.
3. **The cost rows.** Stack 18,304/18,896 residual 0 in two
   independent CGU=1 builds and two draws on the form that is green,
   and the disabled slopes attributed per site; no selection
   machinery; no off-by-default state in a hot frame.
4. **The pair, the mispair, the keys.** 20/20 with `faults=0`; the
   mispair kill or its measured impossibility; every key row closed
   or OPEN with a director question.
5. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions
   — plus the updated drafted implementation brief in `TEMPLATE.md`
   shape, replacing the brief-681 draft.

If the collector cannot carry the attach at the exact stack, the
record is the measured exclusion with its exact rows and the
minimum-cost alternative; "the collector is necessary" or "the
publish is the only form" is a claim until measured. Promotion stays
blocked until the implementation stage proves every bar.

## Context pointers

- `.temp/brief-681/report-design.json` and sections `answer`,
  `root`, `battery`, `keys`, `cost`, `plan`; `.temp/brief-681/
  analysis-cal.log`, `runs/b3-*`, `src/exact/`,
  `drafts/implementation-lp08-residual-5.md` — the state this round
  continues. Read-only; copy before adapting a script.
- `.temp/brief-680/report-design.json`, `analysis-cost.log`,
  `analysis-cal.log`, `analysis-prologues.log`, `analysis-build3-
  stack.log`, `analysis-build4-stack.log`; `.temp/brief-680/src/
  gated`, `src/base` — the green reference and its +64 rows.
- `.temp/director/brief-681-gate.md` — the rulings on the publish,
  the pair, the mispair, the keys and the cost.
- `.temp/director/brief-680-gate.md` — Q2 (no attach before build3
  was measured) and the cost ruling.
- `.temp/director/brief-676-gate.md` — the Q2 authorization of the
  epoch-wide synchronous-event collector and the process-wide
  family.
- `.temp/director/brief-679-gate.md` — the standing per-site gate
  and the excluded forms (selector, +64 session-in-generic,
  outlined adapter, +16/+37).
- `docs/dev/profiling.md` — the contract: one enabled, process-wide
  epoch collector; cross-thread resume; TLS diagnostic maps may
  remain physical telemetry; epoch isolation; reset retirement.
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

Files you MAY write: anything under `.temp/brief-682/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control
  command;
- `.temp/brief-681/`, `.temp/brief-680/` or any retained root,
  except to read and copy. Never execute a file in place;
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
  controls reproduce the current tree. A probe that cannot reproduce
  the finding cannot judge a form.
- Every count, key, route and `path:line` is re-derived from the
  current source; a brief-680 or brief-681 number is a claim, never
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

All scratch under `.temp/brief-682/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-682/report-design.json` in `rum-report/1`, with the
  collector verdicts, the +64 root cause, the exact-stack answer, the
  cost rows, the pair, the mispair and the key table, and every open
  question.
- `.temp/brief-682/drafts/implementation-lp08-residual-6.md` — the
  updated implementation brief in `TEMPLATE.md` shape, plus the probe
  trees, binaries and receipts.
- Run `python3 -I scripts/report.py check 682` and return its summary
  and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
