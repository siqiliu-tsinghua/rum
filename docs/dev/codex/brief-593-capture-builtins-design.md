# brief-593 — design: capture builtins under suspension (G11), and the uniqueness class

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

The E1 oracle's sixth drain (brief-592) found the first defects the census
never saw — two capture builtins whose frames exist but do not resume into
their work. Witnesses: `.temp/brief-592/defects/{EvaluationData.body,TestReport.input,TestReport.expectedOutput,TestReport.sameTest}.md`.

1. **`EvaluationData[expr, props]`** — `EvalEvaluationDataSuspensionFrame`
   (`runtime/debug_tools/evaluation_data.rs:30-52`, resume `:225-245`) does not
   hold the property selector, so the resumed path returns the whole capture
   association where the queued path returns the selected properties. And the
   prints the capture owns surface in the pause's response and then vanish from
   the final cumulative output — the queued path never surfaces them.
2. **`TestReport`** — the resumed path loses the surfaced print prefix of a
   test case, and when a second case suspends, finalization **panics the
   kernel**: `TestReportResultState::into_vec`
   (`runtime/debug_tools/test_report/report.rs:112-118`) demands
   `Arc::try_unwrap` uniqueness of an accumulator node that a suspension frame
   legitimately still shares. Guest code reaches a panic. This is the class of
   the resumed-`Table` panic the ownership branch found.

The design settles the repair of both, the **output semantics of a capture
that is suspended** (which prints and messages belong to the pause's response
and which to the capture), and the **class** behind the panic.

## What the design must settle

1. **EvaluationData.** Which state the frame lacks (the selector, and anything
   else the eager path uses after the capture), and how the resumed path reaches
   the same completion helper as the eager one.
2. **Captured output at a pause.** Read the request checkpoint
   (`session_requests.rs`), the debug capture checkpoint and the pause
   response. State what the contract should be so resumed equals queued: output
   a capture owns is not surfaced at a pause and is not lost, and output outside
   any capture surfaces exactly once, in order. Say where that rule lives (one
   place, not per builtin), which other capture builtins it touches (`Check`,
   `Quiet`, `Reap`, `TestReport`, `Timing` family, … — they passed the oracle,
   so say why), and how it relates to the queued item Q1 in `TODO.md` (one
   ordered print/message sequence from the kernel through the host) without
   absorbing Q1's scope.
3. **TestReport.** The lost print prefix and the panic; the repair that makes
   finalization correct when a frame shares the accumulator — without breaking
   the iterative drain the ownership branch built for these chains (the
   `AccumulatorChain` members must still drop without recursion).
4. **The uniqueness class.** Every production `Arc::try_unwrap` (32 sites in
   `rum-runtime` and `rum-stdlib`, tests excluded): classify each as (a)
   unique by construction, with the reason, (b) falls back safely when shared,
   or (c) **panics or misbehaves when a suspension frame shares it** — with a
   reachability argument, and for every (c) a witness through the rum-vs-rum
   oracle in the scratch test (resumed vs queued, never an aborting probe; a
   kernel panic surfaces as a caught test failure, record it, do not run it in
   a loop). Propose the structural repair for the class — a shape in which a
   shared node cannot reach a uniqueness assumption — not a per-site patch.
5. **The oracle consequence.** The four entries stay owed until repaired; say
   how the repair brief(s) attach their descriptors, and whether any already
   drained capture entry deserves a stronger descriptor (twice inside one
   capture, output on both sides of a pause).

## What I want back

1. The diagnosis of both defects with `path:line` evidence.
2. The captured-output contract, where it lives, and its reach.
3. The uniqueness-class table (all 32 sites) and the witnesses for class (c).
4. The repair design and its sequence; drafted implementation brief(s) in
   `TEMPLATE.md` shape — red tests through the oracle, the standing duty (a new
   frame enters rum-runtime's frame lists), one-codegen-unit instruction gates
   with base-vs-base noise floors where an ordinary path changes, serial runs.
5. Open questions for the director.

## Context pointers

- brief-592's report and its defect files; brief-586's oracle
  (`crates/rum-stdlib/tests/suspension_oracle/`), its runner and
  `support/normalize.rs`.
- `runtime/debug_tools/{evaluation_data.rs,test_report/}`,
  `runtime/session/session_requests.rs`, the debug capture checkpoint and
  `restore_evaluation_data_capture`, the suspension core.
- The ownership branch's accumulator chains (`AccumulatorChain`, the
  participation gate) and the resumed-`Table` repair; memory of that line is in
  `TODO.md` and `PROGRESS.md` (2026-09-21).
- `docs/guide/{evaluation,embedding}.md` (pause responses, cumulative output),
  `TEMPLATE.md`, `AGENTS.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-593/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief593_probe.rs` (it may reuse the oracle's
  support through a `#[path]` module), **deleted before the report is
  appended**, its final content copied to `.temp/brief-593/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository; all
  scratch and `TMPDIR` under `.temp/brief-593/`.
- Do NOT modify any file outside the exceptions; STOP and report instead.
- **Run no probe that aborts a process.** A caught kernel panic inside a test
  is a recorded witness, run once.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief593_probe`
  (300 s). File reads, `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The two diagnoses.
3. The captured-output contract.
4. The uniqueness class.
5. The repair design, sequence and drafted brief(s).
6. Open questions for the director.

---
