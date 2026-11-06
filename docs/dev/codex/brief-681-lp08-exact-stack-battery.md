# Brief 681 — LP08 residual: the exact-stack form's enabled-path battery

**Design continuation for the live tree: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. This round continues the
same design campaign as briefs 677–680; their reports are the starting
point, not evidence to re-derive from zero.**

Brief 680 made the gated form's enabled path green by publishing the
stashed outer phase from
`record_profile_runtime_path_timing_adopted_slow`, and that call costs
+64 B/level (prologue `sub $0x388`; Function/Module 18,368/18,960,
residual 0, two builds and two draws). build3 keeps every other
lifecycle edit and removes that call: 18,304/18,896, residual 0,
prologue `sub $0x378`. But build3 was built for cost attribution only
and **was not run through the enabled-path controls**, so the round did
not answer its own question: is the publish necessary, or is the
exact-stack form already green?

This round answers one question with measured evidence: **does the
build3 form pass every enabled-path control at the exact stack, and if
it does, what remains open on it?**

## Stage A — the battery

1. **The build3 battery.** Build the build3 form (the brief-680
   `src/gated` tree with the `adopted_slow` publish removed) as a
   check binary at one release codegen unit, and run every control on
   it, with the same probes, assertions and fixtures brief 680 used:
   - `queue_boundaries` 9/9, assertions byte-identical, and
     `logical_after_consumer_sees_the_input_parser_error` quiescent
     `(0,0,0,0,0)`;
   - a direct definition finish with no enclosing phase records
     `Outcome::Some`, `faults=0`, counter `[1,1,0,1,0,0,0,0,0,0]`;
   - cancel, reset, drop and invalid leave `open=0`, `leases=0`,
     `fabricated_ok=0`;
   - the 20 valid pair cases, each `native_same=true`, `faults=0`;
   - mirrors `67/67`;
   - the seven Throw wrong-outcome mutants killed with
     `semantics_enabled=false`;
   - the three mispair mutants executed and recorded;
   - the 150 key rows.
2. **The answer.** If build3 is green on every line: state it as the
   exact-stack green form; re-measure the stack (Function 18,304,
   Module 18,896, residual 0, two independent CGU=1 builds and two
   draws) and the disabled slopes with per-site attribution; the
   publish is unnecessary and the +64 form stays excluded. If build3
   is red on any line: name every failing control with `path:line`,
   the record/state that trips it, and the invariant it violates, then
   **stop** — do not design the attach in this round.
3. **The disabled deltas.** Every delta on the measured tree must be
   attributed per site to the queued path's own
   `runtime_path_timing_start` / `record_profile_runtime_path_timing`
   calls in the same run, or removed. The brief-680 run saw `evaluate`
   +1, `prepare_arguments_for_call_with_unevaluated_restore` +6,
   `suspend_call_stage_with_boundary` +60,
   `suspend_initial_evaluation_once_with_boundary` +36 and argument
   `suspend_call_stage_with_boundary` +30; attribute or remove each.
   The 39-instruction attribution was not re-run in 680 and is not
   this round's evidence.

## Stage B — only if Stage A is green on every control

4. **Pair 20/20.** Close the `none-two` difference
   (`call.stage.down_values`: queued none=2, resumed some=2) with a
   discriminating condition that keeps `down` some=1 on both sides.
   A 19/20 count is not accepted; the pair equality is the A/B line.
5. **Mispair.** The three mutants hit and the flush passes — they are
   not kills. Make them turn the exact counter red with
   `semantics_enabled=false` without changing the eight call-site
   signatures' observable finish. If that cannot be done, return a
   measured impossibility with the exact reason and a proposed
   falsifier; do not drop the row silently.
6. **The 15 open key rows.** Exercise each at pause, error and outcome,
   or record source-impossibility with `path:line`. The 12
   `phase_absent` rows stay open unless a fixture exercises the route;
   `set.finish` pause, `catch.resume_after_body` pause and
   `catch.eval.total` error need a probe, not an argument. The four
   `evaluate_atom_fast_path.stable_symbol` / `.atom` pause and error
   rows are source-impossible at `eval_core/mod.rs:174` and stay
   recorded as such.

## Goal

When this round is done, the design record states, with measurement
and not argument:

1. **The exact-stack answer.** Whether the build3 form is green, with
   every control's verdict line quoted; if it is not, every failing
   control root-caused with `path:line` and the round stopped.
2. **The cost rows.** Stack 18,304/18,896 residual 0 in two
   independent CGU=1 builds and two draws on the form that is green,
   and the disabled slopes attributed per site; no
   selection machinery; no off-by-default state in a hot frame.
3. **The pair, the mispair, the keys.** 20/20 with `faults=0`; the
   mispair kill or its measured impossibility; every key row closed or
   OPEN with a director question.
4. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions —
   plus the updated drafted implementation brief in `TEMPLATE.md`
   shape, replacing the brief-680 draft.

If a control cannot be satisfied at the exact stack, the record is the
measured exclusion with its exact rows and the minimum-cost
alternative; "the publish is necessary" is a claim until build3 is
measured. Promotion stays blocked until the implementation stage
proves every bar.

## Context pointers

- `.temp/brief-680/report-design.json` and sections `answer`,
  `faults`, `parser`, `pair`, `keys`, `cost`, `plan`;
  `.temp/brief-680/analysis-cost.log`, `analysis-cal.log`,
  `analysis-mirrors.log`, `analysis-build3-stack.log`,
  `analysis-build4-stack.log`, `analysis-prologues.log`,
  `.temp/brief-680/runs/` and
  `drafts/implementation-lp08-residual-4.md` — the state this round
  continues. Read-only; copy before adapting a script.
- `.temp/director/brief-680-gate.md` — the rulings on Q1–Q3 and the
  cost ruling.
- `.temp/director/brief-679-gate.md` — the standing cost bar and the
  excluded forms.
- `docs/dev/profiling.md` — the contract.
- Live sites: `crates/rum-runtime/src/runtime/resolution.rs`,
  `resolution/resume_frames.rs`, `eval_call_prepare.rs`,
  `eval_core/{call,mod}.rs`, `profiling/{config,timing}.rs`,
  `profiling/logical/{mod,segments,queue_api,epoch,record,completion,totals}.rs`,
  `suspension/core/{driver,profile_mode,profile_queue}.rs`,
  `suspension/typed/adapter.rs`, and the queue-boundary tests.
- Host reset: `crates/rum-host/src/session.rs:176`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-681/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control
  command;
- `.temp/brief-680/`, `.temp/brief-679/` or any retained root, except
  to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs
and probes; it repairs nothing.

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
  current source; a brief-680 number is a claim, never evidence.
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

All scratch under `.temp/brief-681/`; keep `PROGRESS.md` with the
exact next action and the receipts already saved. Leave every tree
compiling at each checkpoint.

## Deliverable

- `.temp/brief-681/report-design.json` in `rum-report/1`, with the
  battery verdicts, the exact-stack answer, the cost rows, the pair,
  the mispair and the 50-key table, and every open question.
- `.temp/brief-681/drafts/implementation-lp08-residual-5.md` — the
  updated implementation brief in `TEMPLATE.md` shape, plus the probe
  trees, binaries and receipts.
- Run `python3 -I scripts/report.py check 681` and return its summary
  and all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one
answer (measured) and the draft's bar list.
