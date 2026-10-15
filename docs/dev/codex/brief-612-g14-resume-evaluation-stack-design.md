# brief-612 — design: G14, resume never unwinds the evaluation stack it restores

`Scope: NONE — no production change.` This brief is design-only, with one
declared scratch test (see Scope). **Do not run `git` in any form**,
including before you have read this brief.

## Goal

A resumed evaluation does not leave the evaluator in the state that the
same program reaches when its input is queued. After one pause, the
resumed path reports a false cycle on repeated identical expressions. On
expressions that change, it leaks one recursion level per resume. The
queued-input path is correct in every case below.

The director found this on 2026-10-16, while gating brief-611 (G13 step S5).
Its 2 000-resume harness stopped with `RecursionLimit` at 1 021 resumes.
The defect is already present on S4, and it is as old as resumable input
itself:

- `b51dd794` (2026-03-20) dropped only the top stack entry at a pause.
- `eee42d2f` (2026-03-21) widened that to two, after false positives. That
  patched the instances seen, not the class.

The design must settle the repair of the class, not of the shapes below.

### The mechanism, as far as the director has read it

- **`begin_input_suspension`** (`runtime/suspension/core.rs:492-518`).
  It snapshots `evaluation_stack` minus its top two entries into the pending
  suspension. The comment explains the "two": the `InputString` call and
  its immediate caller are resumed from frames, and keeping them caused
  false cycles.
- **`resume_suspended_slot`** (`core.rs:615-648`). It installs that
  snapshot as the live stack and sets `evaluation_depth` to its length.
  The native-stack poll depth is derived from that depth.
- **`drive_resuming_frames`** (`core.rs:704`). It completes the queued
  frames, innermost first. Nothing pops a restored entry or lowers the
  depth as a frame completes and hands its value outward.
- **The evaluation level** (`runtime/eval_core/mod.rs:28-96`):
  - Its cycle check (`:38-47`) refuses any `expr` already on the stack in
    the same epoch.
  - Its depth check (`:49-53`) refuses at `recursion_limit`.
  - On `Ok`, its epilogue (`:91-93`) runs
    `enforce_active_resource_limits_for_expr`, the active time and memory
    limits (`debug_tools/resources/constraints.rs:139`).

  A level that a frame completes on resume never runs that epilogue.

### Witnesses

Each witness pauses once, through the resumed path. The director's probes
are in `.temp/director/depth-leak-probe/` (built against the working tree)
and `.temp/director/depth-leak-probe-s4/` (S4). They are rum-host programs,
resumed against queued. Argument 1 is the pause cap, then setup/source
pairs; outputs are in `run1.txt` and `run2.txt`.

- **False cycle.** These report `General::passlim` +
  `TerminatedEvaluation["IterationLimit"]`:
  - `f[1]+f[1]` with `f[x_] := (InputString["rhs"]; 7)`;
  - `Table[f[1], {3}]` and `Do[f[1], {1100}]`;
  - SubValue `k[1][2]` in a `Table`;
  - OwnValue `v := (InputString["x"]; 7); Table[v, {3}]`;
  - `Table[{{InputString["x"]}}, {3}]`;
  - `Table[h[{InputString["x"]}], {3}]`;
  - `Table[h[k[InputString["x"]]], {3}]`;
  - `Table[Plus[1, Length[{InputString["x"]}]], {3}]`;
  - `Table[g[g[InputString["x"]]], {3}]` with `g[x_] := x`.
- **Depth leak.** `Table[(t = i; f[1]), {i, 1100}]` hits `RecursionLimit`
  at 1 021 resumes.
- **Pass.**
  - `Table[InputString["x"], {1100}]`;
  - `If`, `Module` and `Map[f, Range[3]]` shapes;
  - `Table[(v = (InputString["x"]; 7)), {3}]`.

  The drop-two heuristic covers exactly the shallow shapes that every
  earlier suspension probe used.

## What the design must settle

1. **The diagnosis, with `path:line` evidence.** For three witnesses, trace
   the live `evaluation_stack` and `evaluation_depth` at the pause, at
   install, and after each resumed frame completes, against the queued run
   of the same program:
   - one false cycle;
   - the depth leak;
   - one passing shape.

   Say why each passing shape passes.
2. **The correct model.** While a frame resumes, the evaluator's stack,
   depth, epoch and native-stack poll state must equal what the queued path
   has at that frame's level.
   - **The candidate model.** Each queued frame records the stack length
     and depth at its insertion. The driver truncates the stack and sets the
     depth to those values before it resumes that frame.
   - **Settle it or replace it.** Say where the record lives. It should be
     the queue entry, structurally, so that none of the ~600 frame types can
     forget it. Cover every insertion route:
     - `insert_suspension_frame` (`core.rs:528`);
     - `insert_typed_suspension_frame` (`:555`);
     - the typed token and witness routes;
     - `FrameChain`, which inserts several frames at once;
     - the resume driver's own pushes.
   - **The "top two".** Does the precise model make the drop-two heuristic
     unnecessary? Show it on the matcher-backtracking re-entry that
     `eee42d2f` was written for. If anything of the heuristic must remain,
     say exactly what and why.
3. **Accounting.** The armed path keeps a maintained total that includes
   evaluation-stack entries:
   - `push_accounted_evaluation_entry` / `pop_accounted_evaluation_entry`;
   - `replace_accounted_evaluation_stack`;
   - the shadow stack (`mode.stack_push`).

   Truncation on resume must keep that total exact. Show how, and name the
   falsifier that proves it (`debug_tools/resources/maintained.rs`).
4. **The skipped epilogue.** Decide whether each frame-completed level must
   run `enforce_active_resource_limits_for_expr` on resume, so that
   `TimeConstrained`/`MemoryConstrained` behave as they do on the queued
   path. Write a rum-vs-rum witness for any observable difference (a
   constrained region whose limit is crossed only at a resumed level's
   completion). If there is none, say why it is unobservable.
5. **Re-suspension.** A resumed frame may pause again. Then
   `begin_input_suspension` snapshots a stack that contains restored
   entries. Show that the second pause's snapshot equals the queued path's,
   suspending twice and mid-loop.
6. **Interaction with G13.** The branch is mid-way through typing 600
   frames (S6-S54, then F1-F23 and R):
   - The repair must live in the driver and the queue, transparent to frame
     implementations, so that no typing step changes for it.
   - Say what the typed driver (S2) and the legacy arm each need, and where
     in the remaining 73 steps nothing moves.
   - If the repair cannot be transparent, say which steps it touches and
     why.
7. **The class falsifier.** Extend the E1 oracle
   (`crates/rum-stdlib/tests/suspension_oracle/`) with descriptors that:
   - vary the depth between the pause and a construct that repeats an
     identical expression;
   - suspend mid-loop and twice;
   - include a long run of changing expressions past `recursion_limit`
     resumes;
   - cover each definition kind (DownValue, SubValue, UpValue, OwnValue),
     nested lists and heads, and arithmetic.

   They are rum against rum: resumed equals queued, in value and side
   effects, with the suspension asserted. Say which of the 133 current
   oracle tests and of brief-548's 195 probes already cover a shape that
   passes only by the heuristic.
8. **Cost.** The ordinary path must not pay: no field, branch or store on a
   non-suspending evaluation. State the resume-path cost (a record per
   inserted frame, a truncate per resumed frame) and how the implementation
   brief measures it. The measures are allocations, then one-codegen-unit
   instruction slopes on the resume labels, plus the 107 ordinary labels
   unchanged.

## What I want back

1. The diagnosis of the three traced witnesses, with `path:line` evidence.
2. The model: where the record lives, every insertion route, the fate of
   the heuristic, accounting, the epilogue, re-suspension and G13.
3. The witnesses you ran in the scratch test, with their output (see
   Scope).
4. The repair sequence, and drafted implementation brief(s) in
   `TEMPLATE.md` shape. Each one has:
   - red tests through the oracle;
   - a MAY list with a disclosure duty;
   - one-codegen-unit instruction gates;
   - serial runs;
   - a stop-loss keyed on whether the failure count falls.
5. Open questions for the director, each with your recommendation.

## Context pointers

- `runtime/suspension/core.rs`, from `begin_input_suspension` through
  `drive_resuming_frames`, and the typed driver under
  `runtime/suspension/typed/`.
- `runtime/eval_core/mod.rs:15-96` and `debug_tools/resources/{constraints,maintained}.rs`.
- `git log` is unavailable. The two historical commits are summarised under
  Goal; the comment at `core.rs:493-498` is the surviving rationale.
- The E1 oracle (`crates/rum-stdlib/tests/suspension_oracle/`), its runner
  and `support/`; `TODO.md`'s G13 section and its G14 entry;
  `docs/guide/{evaluation,embedding}.md` on pauses and resume.
- `TEMPLATE.md` and `AGENTS.md`.

## Scope

- **Files you MAY modify: NONE,** except these:
  - append your report to this file;
  - scratch under `.temp/brief-612/`;
  - **one** scratch integration test,
    `crates/rum-stdlib/tests/zz_brief612_probe.rs`. It may reuse the
    oracle's support through a `#[path]` module. **Delete it before the
    report is appended**, and copy its final content to
    `.temp/brief-612/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.** No network, and nothing outside this
  repository. All scratch and `TMPDIR` go under `.temp/brief-612/`.
- **Do NOT modify any file outside the exceptions.** STOP and report
  instead.
- **Run no probe that aborts a process.**
- **Commands are serial,** each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief612_probe`
  (900 s). Your tool timeout must exceed every bound you pass. You may also
  read files, use `rg`/`grep`, and run `python3`.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum
  only.
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed: "design only", the scratch files, and the deleted test.
2. The diagnosis.
3. The model, and items 2-8 above.
4. The witnesses and their output.
5. The repair sequence and the drafted brief(s).
6. Open questions for the director.

---
