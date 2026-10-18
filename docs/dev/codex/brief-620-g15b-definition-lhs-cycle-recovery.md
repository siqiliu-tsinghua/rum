# brief-620 — G15-B: a resumed definition LHS keeps its cycle recovery

## Goal

When a definition's left-hand side is prepared, each active atom, call head
and evaluated argument is evaluated. If that evaluation reports a cycle on
the very symbol being evaluated, the ordinary path recovers: the symbol
stands for itself (`x := (…; x); f[x] = 3` stores `f[x]`). When that
evaluation pauses for input first, the resumed path loses the recovery and
the definition ends in `TerminatedEvaluation["IterationLimit"]`.

When you are done:

- the resumed path applies exactly the ordinary path's recovery rule, in all
  three awaiting states (an active atom, a call head, an evaluated call
  argument), after any number of earlier pauses, and in every definition
  form that prepares a left-hand side;
- no other error is recovered on either path;
- no frame, field or allocation is added, and F0257's `Input` and `Output`
  stay `Expr`;
- the ordinary path of every program executes no more work than at G15-A.

This is step B of G15, inserted into G13 before S10. The design is
brief-618's report and its director review; where
`.temp/brief-618/draft-G15-B.md` differs from this brief, this brief
governs. G15-A is done (`afdcd02f`; brief-619); the tree is `b87239f9`.
G15-C (brief-621) follows.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-618: the section *Newly exposed same-type tail: definition LHS*, the
  LHS paragraph of *Repair design and G13 ownership*, the cost contract and
  the director review.
- **brief-619 is your template**, with its continuation and director gate
  record. Its measurement bars, its copy-census calibration and its stack
  method apply unchanged. Its continuation's lesson applies too: an
  attribute is not evidence; measure the frame.
- **The code**, under `crates/rum-runtime/src/runtime/definitions/`:
  - `definition_lhs_preparation.rs`: the three ordinary evaluation sites
    (`:284` active atom, `:338` call head, `:409` evaluated argument), each
    followed by its witness and its `Await*` state;
    `resume_definition_lhs_preparation` (`:498`), whose first statement is
    `let value = outcome.map_err(ResumeStop::failure)?;` (`:504`); and
    `evaluate_definition_lhs_active_expr` (`:531-540`), which holds the
    recovery rule:

    ```rust
    match self.evaluate_atom_fast_path(source.clone()) {
        Err(EvalError::EvaluationCycleDetected { expr })
            if matches!(source, Expr::Atom(Atom::Symbol(_))) && expr == source.to_string() =>
        {
            Ok(source.clone())
        }
        result => result,
    }
    ```
  - `definition_lhs_preparation/resume_frames.rs`: the work states
    (`AwaitPatternAtom { source }`, `AwaitCallHead { source_head, … }`,
    `AwaitCallArgument { …, source }`, `:28-41`), and F0257's typed frame
    (`:156-171`). All three states already retain the expression that was
    evaluated.
- S5 (brief-611) typed F0257 and M0260-M0262 with `DefinitionResumeRoute`.
  The frame before S5 also began with `outcome?`, so the defect predates S5;
  it was never in the resumed path.

### The defect, re-probed at `b87239f9`

The director's probe (`.temp/director/s-g15b-probe/`, `pairs.py`,
`run1.txt`) runs each program through the resumed path and again with its
input queued; every prompt is answered `"1"`. With
`X = x := (InputString["w"]; x)`: **16 pairs DIFF, 4 SAME, every one
paused.** Each DIFF is `TerminatedEvaluation["IterationLimit"]` resumed
against the stored definition queued:

| Setup | Source |
| --- | --- |
| `X` | `{f[x] = 3, DownValues[f]}` (active atom in an argument) |
| `X` | `{x[1] = 3, SubValues[x]}` (call head) |
| `X` | `{f[Blank[x]] = 3, DownValues[f]}`, `{f[x_x] = 3, DownValues[f]}` (pattern atom) |
| `X` | `f[x] := 3`, `g[x] ^= 3`, `g[x] ^:= 3`, `x /: g[x] = 3`, `x /: g[x] := 3`, each with its stored values |
| `x /: g[x] = 3; X` | `{x /: g[x] =., UpValues[x]}` |
| `f[x] = 3; X` | `{f[x] =., DownValues[f]}` |
| `X; y := (InputString["v"]; y)` | `{f[x, y] = 3, DownValues[f]}` (two cycles, two pauses) |
| `X` | `{f[x, x] = 3, DownValues[f]}` |
| `z := (InputString["u"]; 2); X` | `{f[z, x] = 3, DownValues[f]}` (a successful pause, then the cycle) |
| `X` | `Do[f[x] = i, {i, 2}]; DownValues[f]`, `Table[f[x] = i, {i, 2}]` |

**Controls that are SAME and must stay SAME:** a different symbol's cycle
(`x := y; y := (InputString["v"]; y); f[x] = 3`, `IterationLimit` on both
paths), a cycle that is not the evaluated symbol (`f[g[x]] = 3`,
`IterationLimit` on both), a `Throw` (`Catch[f[x] = 4]` with
`x := (InputString["w"]; Throw[3])`, `3` on both), and a plain pause with no
cycle.

brief-618's fixed set (`.temp/brief-618/cases.tsv`) owns five of these as
`lhs.cycle`, `.cycle_head`, `.cycle_pattern`, `.cycle_twice` and
`.after_prior_pause`; `.noncycle`, `.non_symbol_cycle` and
`.other_symbol_cycle` are its controls.

### The design (ruled)

1. **One shared finish.** Extract a non-suspending
   `finish_definition_lhs_active_expr(source: &Expr, outcome: Result<Expr,
   EvalError>) -> Result<Expr, EvalError>` holding exactly today's rule: an
   `EvaluationCycleDetected` whose `expr` equals `source.to_string()`, with
   `source` a symbol atom, becomes `Ok(source.clone())`; every other outcome
   passes through unchanged. It takes no `Session`.
2. **The ordinary path uses it.** `evaluate_definition_lhs_active_expr`
   becomes the evaluation followed by this finish. The three ordinary sites,
   their witnesses and their routes do not change.
3. **The resumed path uses it first.** In
   `resume_definition_lhs_preparation`, take the retained expression from the
   awaiting state (`source` for `AwaitPatternAtom` and `AwaitCallArgument`,
   `source_head` for `AwaitCallHead`) and apply the finish to `outcome`
   **before** `map_err(ResumeStop::failure)` and before the state's work is
   replaced. Do not re-evaluate or inspect the source otherwise, and do not
   clone anything on the success path that is not cloned today.
4. **Nothing else changes.** M0260-M0262 keep their witnessed routes and
   `DefinitionResumeRoute`; the resumed driver keeps its same-frame token for
   a later pause; no `Arc` enters the token; no codec, mode or frame type
   changes. A later pause after a recovered value must capture fresh state
   once, through the existing route.

## Scope

A listed file or class is permission to do this step's work there. It is
not permission to change definition storage, matching, cycle detection,
resource limits, any other G13 row, or to tidy. Disclose every file you edit.

**Classes.**

- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs`,
  for the finish and its two uses. `definition_lhs_preparation/resume_frames.rs`
  only for a visibility edit the change needs. A sibling `tests.rs` for any
  unit test that needs private items, declared as `#[cfg(test)] mod tests;`.
- **Oracle**: a new fixture module under
  `crates/rum-stdlib/tests/suspension_oracle/fixtures/` and its
  registration. Do not restructure `manifest.rs` or `descriptors.rs`; the
  obligation-keyed manifest is G15-C's.
- **Census**: `crates/rum-stdlib/tests/typed_slot_census/` and
  `typed_slot_census.rs`, only to pair moved owned source sites. No typing
  receipt changes: there is no new frame and no new typed site.
- **Derived facts follow the diff** (brief-613's class rule; see brief-619's
  Scope). A row on a deleted operation is retired, one on an added
  operation is new (it stays rejected or unknown and claims no mark), a
  moved or renamed one is paired old → new; disclose each. **STOP** on a
  change of meaning where the source did not change, a mark or coverage
  gained or lost, a new `UnknownHostPayload`, or a production receipt
  change outside this step. Never hand-write a fingerprint, certificate
  cell or ledger chain. Ownership replays consist of pure moves for
  unchanged constructs. Citation fields must follow line moves.
- **Canonical tool outputs**: an unchanged canonical gate tool may write its
  own fixed output paths under `target/` or `.temp/`; disclose each.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md`, `.temp/brief-606/` and the core `Expr`/`Symbol` representation
stay unchanged.

**Disclosure duty.** If a needed change falls outside every class, make no
edit there. STOP and report the file, the line and why.

## Steps

1. **Bases, harness and calibration, before any source edit.** All scratch
   goes under `.temp/brief-620/`.
   - **Supplied** (verify `base-hashes.txt` first):
     - `base/rum-cli`, `base/rum-cli-alloc`: the G15-A tree's
       one-codegen-unit binaries, built by the director at gate-619 and
       rebuilt at `b87239f9` with no recompiled unit and an equal hash;
       their build commands are beside them.
     - `base-source/`: the `b87239f9` export.
     - `stack-base/probe.rs`: brief-619's 21 shapes, including `lhs-arg`
       (`la[n_] := (hh[la[n - 1]] = n)`), in which the recursion runs
       inside left-hand-side preparation.
     - `stack-base/expected-21-g15a.txt`: the director's gate-619 values for
       all 21 shapes, debug and release, on the G15-A tree.
     - `calib/build.sh`, `calib/measure.py`: the calibration scripts.
     - `perf-draft/expected.txt`: eight new ordinary labels with their base
       values.
   - **Stack bases.** Build with `calib/build.sh` and measure with
     `calib/measure.py`. **All 21 shapes must equal
     `expected-21-g15a.txt` exactly; if one does not, STOP.** Every fit must
     have zero residual.
   - **Resume harness.** Copy `.temp/brief-619/resume-harness/` to
     `.temp/brief-620/resume-harness/`. Add workloads in which the recovery
     follows a pause, in each awaiting state (argument atom, call head,
     pattern atom), after a successful earlier pause, and with two cycles in
     one left-hand side. Each repeats its pauses N times, checks its final
     value and the stored definition. Build `resume` and `resume-alloc` from
     the unedited tree with one codegen unit, each in its own target
     directory; their source hashes must match `base-source/`. Each
     workload's callgrind frame census must show F0257 resuming the intended
     number of times per pause.
2. **Labels.** Copy `.temp/brief-619/perf/labels.json` and its programs to
   `.temp/brief-620/perf/`. Add the eight labels of
   `perf-draft/expected.txt` in the same N/2N format, each checking its own
   result, and the new resume workloads as resume labels with their
   `resumes_per_iteration`. `lhs-set-cycle-recovery` runs the ordinary
   recovery itself, so it is the closest ordinary neighbour of the change.
3. **Fixtures, red on the base.** Before changing anything, add this step's
   oracle fixtures and run them on the unedited tree, under `Checks::ALL`,
   with unconditional `FaultProbe` absence, the pause asserted at the
   intended phase, and the lifecycle checks (pending owner, overlap
   rejection, empty input queue, fresh request).
   - **Red:** every DIFF row of the director's re-probe above, and
     brief-618's five owned rows. Each projects the stored definitions
     (`DownValues`, `UpValues`, `SubValues` as the form requires), not only
     the value.
   - **Green controls:** the four controls above, brief-618's three LHS
     controls, and a cycle that follows a pause but does not pause itself:
     `z := (InputString["u"]; 2); cL := cL; {f[z, cL] = 3, DownValues[f]}`
     stores `f[2, cL]` on both paths today, because the resumed driver
     evaluates `cL` through the ordinary helper (director probe, SAME).
   - Record the red and green counts on the base in both builds.
4. **Implement the design.**
5. **Falsify the fixtures.** In a scratch copy of the candidate tree, remove
   the resumed finish for one awaiting state at a time (three mutants).
   Each mutant must turn that state's fixtures red and leave the others
   green. Report the counts. Do not keep the mutants.
6. **Ordinary controls.** Rerun the 28 round-4 ordinary controls through
   brief-611's harness: copy `.temp/brief-619/ordinary-controls/`, re-sync
   its `suspension_oracle/` support from the committed tree, and all 28 must
   pass.
7. **Census and inventories.** The census totals must not change (608
   implementations, 97 typed, 55 typed macros, 48 witnessed sites); report
   them. Replay fingerprints, both certificate sets and ownership discovery,
   and disclose every changed row under the derived-facts class.
8. **Re-run brief-618's fixed set** (`.temp/brief-618/probe.rs`, unchanged,
   in a scratch copy of the tree with its own target directory). **Expected:
   22 DIFF, every one among G15-A's 27, with the five owned rows fixed.** A
   pair that was SAME and is now DIFF is a STOP.

## Verification (MANDATORY — run to green or evidence)

**How to run.** Serially, each command through `python3
scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-620/tmp …`.
Bounds: workspace 3 600 s; oracle 1 500 s per build; everything else 900 s.
**Your shell tool's own timeout must exceed the command's bound.**

```
cargo fmt --check
cargo test -p rum-runtime --lib suspension::typed::tests
cargo test -p rum-runtime --lib frame_root_trace
cargo test -p rum-runtime --lib resume_evaluation_context
cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
cargo test -p rum-stdlib --test typed_slot_compile_controls
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls
cargo test -p rum-stdlib --test typed_slot_census
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
scripts/check-suspension-oracle.sh 1500
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
cargo test --workspace --no-fail-fast
```

- Also: the ordinary-controls harness in both builds, step 5's mutants and
  step 8's fixed set.
- Rerun the director's probes through a copy of
  `.temp/director/depth-leak-probe/` built under `.temp/brief-620/`
  against the candidate tree: `.temp/director/s-g15b-probe/pairs.py`
  (expected: all 20 SAME), `.temp/director/g15-probe/pairs.py` (14 SAME),
  `.temp/director/s10-probe/pairs.py` (51 SAME, the same seven DIFF), and
  `.temp/director/s{9,8,7,6}-probe/pairs.py` (all SAME). The probe answers
  `"1"` to every prompt and takes the pause cap first.
- **Native ZeroMQ.** Run the workspace with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`,
  report the nine as skips, and report every other count. The director runs
  them natively.

Report every `test result:` total, the oracle's fault-occurrence delta for
each build, and any compiler warning the diff introduces. Neither oracle
build may warn.

**Performance.** brief-619's six measurement items apply unchanged, with
these specifics:

1. **Allocations** on every label: ordinary labels exactly equal in slope and
   whole-program counts; on the resume workloads, a recovered definition now
   completes, so attribute every difference to the work that now runs.
2. **Instruction slopes**, two interleaved `perf stat` pairs per arm on all
   labels, flags cleared by five runs or six callgrind processes per arm with
   every deterministic delta named. Report the own instructions per call of
   `drive_definition_lhs_preparation` and
   `evaluate_definition_lhs_active_expr` (or wherever the finish lands after
   inlining) on the eight new labels and `set-ownvalue`, base and candidate.
3. **Copies**, with brief-619's calibrated census (copy
   `.temp/brief-619/copy-census/` and `.temp/brief-619/debuginfod/`, point
   them at this brief, and recalibrate in your environment: callgrind must
   name `__memcpy_avx_unaligned_erms`, and `catchexceptions-handler` must
   show a non-zero memcpy slope equal at N and 2N). No ordinary label above
   base; inspect the changed functions' prologues and stores as well.
4. **Stack**: all 21 shapes, debug and release, four heights with three
   samples. **Bar: every shape at or below base, `lhs-arg` in particular.**
   Tabulate the prologue of every function this step changes. After any
   copy, ABI or inlining fix, rerun the stack shapes and the copy and
   instruction attribution on the final code.
5. **Resume attribution** on every resume workload, six callgrind processes
   per arm, deterministic deltas named. The driver's 136-B move stays R's.
6. **Timing** last, as brief-619 specifies: four full panels, then the
   aligned whole-program probe for any residual whose instructions are
   equal.

A regression is a defect to fix or report. A residual goes to the report
with its attribution, and I rule on it.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- **No default-path cost.** Every definition prepares its left-hand side.
  The finish must cost the ordinary path nothing: the same match, on the
  same values, in the same place. A shape that moves the evaluation's
  `Result` into another function by value can add a 136-B copy; S9 and
  G15-A both measured that class.
- **Recover nothing new.** The rule's three conditions (the error is a
  cycle, the source is a symbol atom, the cycle's expression is that
  symbol's text) stay exactly as they are. The controls are the falsifier.
- **Equal `Input` and `Output` types prove nothing about the tail.** That is
  why this defect survived S5. The mutants in step 5 are what prove the
  fixtures see it.
- Actions, codecs and modes stay sealed and opaque. Profiling records stay
  balanced.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-620/`.
- After each step, and after every gate, write `PROGRESS.md` and a
  machine-readable `state.json`: the verified state, the compile and test
  counters, any half-edited files, and the exact next action.
- Append each report section to `.temp/brief-620/REPORT.md` as soon as its
  evidence is final, and copy the whole file into this brief at the end.

## Hard rules

- **No `git`, in any form.** No network.
- Modify only the Scope. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- Commands allowed: scoped reads and edits, `rg`/`grep`, bounded `python3`;
  the bounded Cargo commands above and bounded compiler fixtures;
  `scripts/check-suspension-oracle.sh` and
  `scripts/core-language-microbench.sh`; `valgrind`, `callgrind_annotate`,
  `objdump`, `nm`, `perf` and `taskset` on the measurement binaries; the
  director's probe binaries built under `.temp/brief-620/`. No probe that
  aborts a process.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No new frame, dual-arm decoder, per-frame mode override, transit record or
  public raw adapter. Type no row. Never weaken an oracle `Checks` flag, a
  retention reject, a fault-absence gate or a documented test to get a pass.
- **Stop-loss is keyed on progress.** Its counters: owned semantic failures
  (the fixtures and step 8's set), mutant survivals, and the allocation,
  instruction, copy and stack bars. Compile errors you are reducing count as
  progress. Stop after two attempts on a compiling tree that reduce none of
  them.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with its semantic change. Disclose every ripple and
   pin update as old → new, under its derived-facts case.
2. The design as built: the finish, its two uses, and the call path of each
   awaiting state.
3. The fixtures and harness workloads: what each exercises, its frame
   census, and its result on the base and the candidate in both builds;
   the mutants; step 8's 136-pair result; the director's probes.
4. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions. Every canonical tool output path written.
5. Allocation, instruction, copy (with the calibration), stack and
   resume-attribution evidence, and timing with any alignment check.
6. Stop-loss counters, deviations and open questions.
