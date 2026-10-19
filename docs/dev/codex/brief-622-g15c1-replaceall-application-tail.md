# brief-622 — G15-C1: a resumed ReplaceAll finishes its own application

## Goal

`ReplaceAll` and `ReplaceAllUnheld` apply their rules and then finish the
application in the same builtin call: a self-evaluating atom returns as is,
and anything else is evaluated (`evaluate(replaced)`). When a rule's guard or
`PatternTest` pauses for input, the resumed path loses that finish. The
replaced value goes back unevaluated, and the evaluator's fixed point
evaluates it later, outside the builtin. The value is repaired, but the
ordered trace is not: queued, `ReplaceAll`'s evaluation pass reports `2`;
resumed, it reports `Plus[1, 1]` and a separate pass follows.

When you are done:

- **The repair.** Both builtins run the same post-application finish on the
  resumed path as on the ordinary path, in the same place. The ordered
  trace, value, effects and messages equal the queued path's, including
  after a second pause in the replacement value.
- **The typing.** The new continuation is typed from birth. No decoder,
  codec, mode or existing frame changes.
- **No ordinary cost.** No ordinary program executes more work than at
  G15-B.

This is G15-C1, the new repair group that brief-621 found (its director
review accepts it). It precedes G15-C2, the manifest, `CHANGELOG.md` and
S10. Where `.temp/brief-621/draft-G15-C1-rewrite-output-tail.md` differs from
this brief, this brief governs.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- **brief-621**: its report's P0941/P0946 witnesses and its director review.
  Witness traces are in `.temp/brief-621/continuation1/new-lost-witnesses.json`.
- **Templates.**
  - **brief-619 (G15-A)** is your shape template: a producer's actual
    result is witnessed, and a cold helper suspends with only
    `(witness, request)`. A typed unit frame's resume runs the shared next
    phase, and a second pause inside that phase installs its own frame and
    ends with `ResumeStop`. See `scoping/callable/status_application.rs`.
  - **brief-620 (G15-B)** is your measurement template, with its three
    continuations and its gate record. Its bars, calibrations and lessons
    apply. The most important lesson: a stack bar bounds the chain of
    frames live at once, including every helper the entry calls, never one
    function's prologue.
- **The code**: `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs`.
  - `resume_replace_all_after_rules` (`:45-70`) and
    `resume_replace_all_unheld_after_rules` (`:112-138`) each call
    `self.apply_replace_all(target, &rules, RuleApplicationSource::…)?`. That
    `?` is the lost edge.
  - Each then runs the finish:

    ```rust
    if expr_is_self_evaluating_atom(&replaced) {
        return Ok(replaced);
    }
    let replaced_result = self.evaluate(replaced);
    crate::resume_or_suspend!(
        self,
        replaced_result,
        ok(value) => Ok(value),
        suspend => ResumeReplaceAllAfterRulesSuspensionFrame, // …Unheld… in the second
    )
    ```

    The two `ResumeReplaceAll…AfterRulesSuspensionFrame`s are legacy frames
    that S24 will type. They own a pause *in* `evaluate(replaced)`, not the
    lost edge before it.
- **The future owner.** `.temp/brief-606/draft-S24.md:31,101-104,115-118`.
  S24 later types this file's frames and must keep this step's regression.

### The defect, re-probed by the director

These runs used brief-621's comparator on the G15-B tree
(`.temp/director/c1-probe/`, `cases.jsonl`, `run1.jsonl`). Each program runs
resumed and again with its input queued, and every prompt is answered `"1"`:

| Case | Source | Result |
| --- | --- | --- |
| atom, traced | `ReplaceAll[1, (x_Integer /; (InputString["p"]; True)) :> (x + 1)]` | DIFF (trace) |
| unheld atom, traced | `ReplaceAllUnheld[1, (x_Integer /; (InputString["p"]; True)) :> (x + 1)]` | DIFF (trace) |
| list target, traced | `ReplaceAll[{1}, (x_Integer /; (InputString["p"]; True)) :> (x + 1)]` | DIFF (trace) |
| atom, no trace | the first source, untraced | SAME |
| replacement logs | `… :> (AppendTo[log, "rhs"]; x + 1)`, projecting `log` | SAME |
| Return in the replacement | `f[] := (ReplaceAll[1, (x_Integer /; (InputString["p"]; True)) :> Return[5]]; 7); f[]` | SAME |

The ordered trace is part of the observation contract (brief-618's
observation model, accepted). The three SAME rows are controls and must
stay SAME.

## Scope

A listed file or class is permission to do this step's work there. It is
not permission to change matching, rule application order, other G13 rows,
or to tidy. Disclose every file you edit.

**Classes.**

- **Production.**
  - `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs`,
    for the shared finish, the witnessed producer sites, the cold helpers and
    the new frames.
  - A sibling module under `replace/` only if the new frames warrant one,
    declared from its parent.
  - The parent module root only for that declaration.
- **Frame lists.**
  - `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs`: the
    named-frame pin, 565 → 567 for two new frames.
  - `tests/frame_root_trace/derived_inventory.rs`: exactly the new
    derives.
- **Oracle**: a new fixture module under
  `crates/rum-stdlib/tests/suspension_oracle/fixtures/` and its
  registration. Do not restructure `manifest.rs` or `descriptors.rs`; the
  manifest is a later step.
- **Census**: `crates/rum-stdlib/tests/typed_slot_census/`,
  `typed_slot_census.rs`, the census's stage inventory and receipt files,
  as brief-619 changed them for its two new typed frames. That is a new
  cohort file (`g15c1.rs`), the receipt and stage totals, and the reporting
  list, nothing else.
- **Derived facts follow the diff** (brief-613's class rule; see brief-619's
  Scope):
  - a row on a deleted operation is retired;
  - a row on an added operation is new (it stays rejected or unknown and
    claims no mark);
  - a moved or renamed row is paired old → new;
  - disclose each.

  **STOP** on a change of meaning where the source did not change, a mark
  or coverage gained or lost, a new `UnknownHostPayload`, or a production
  receipt change outside this step. Never hand-write a fingerprint,
  certificate cell or ledger chain.
- **Canonical tool outputs**: an unchanged canonical gate tool may write its
  own fixed output paths under `target/` or `.temp/`; disclose each.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md`, `.temp/brief-606/`, `.temp/brief-621/` and the core
`Expr`/`Symbol` representation stay unchanged.

**Disclosure duty.** If a needed change falls outside every class, make no
edit there. STOP and report the file, the line and why.

### The design (ruled)

1. **One shared finish.**
   - Extract the post-application tail into one helper per route, or one
     helper taking the route. It holds the atom test, `evaluate(replaced)`
     and the existing `resume_or_suspend!` that installs the route's
     existing legacy value frame.
   - The ordinary path calls it exactly where the tail runs today.
2. **The producer is witnessed.**
   - At each `apply_replace_all(…)` call, bind the actual `Result`, take
     `slot_witness` on it, then match once:
     - `Ok` goes to the shared finish;
     - `Err(InputRequired { request })` goes to a named
       `#[cold] #[inline(never)]` helper that receives only
       `(witness, request)` and suspends with the route's new frame;
     - any other error propagates unchanged.
   - Do not move the producer's `Result` into another function.
3. **Two typed unit frames, one per route.**
   - `ReplaceAllApplicationFrame` and `ReplaceAllUnheldApplicationFrame`
     (names free), each with `Input = Expr` (the replaced value) and
     `Output = Expr`.
   - Each frame's resume sends `Ok` to the same shared finish and propagates
     `Err` unchanged.
   - **The second pause.** A pause inside `evaluate(replaced)` installs the
     route's existing legacy value frame through the shared finish, exactly
     as on the ordinary path. The resume then ends with
     `TypedAction::complete_work(…map_err(ResumeStop::failure))`, as G15-A's
     projection frame does.
   - **STOP and report** if a typed resume cannot end that way after the
     shared finish installs a legacy frame. Do not add a raw bridge or a
     second value frame instead.
4. **Nothing else changes.**
   - Rule application order and source labels (`RuleApplicationSource`)
     stay exactly as they are.
   - The existing legacy frames are unchanged.
   - No `Arc` enters a token.
   - No allocation, clone or store is added on the success path.

## Steps

1. **Bases and calibration, before any source edit.** All scratch goes
   under `.temp/brief-622/`.
   - **Supplied** (verify `base-hashes.txt` first):
     - `base/rum-cli`, `base/rum-cli-alloc`, `base/resume`,
       `base/resume-alloc`: G15-B's final one-codegen-unit binaries
       (brief-620 continuation 2, rebuilt with equal hashes at gate-620).
       `resume` and `resume-alloc` carry brief-620's harness.
     - `base-source/`: an export of `a6ff0332`, whose code is G15-B's
       `43a26c72`.
     - `stack-base/probe.rs`: brief-620's 21 shapes plus `replaceall-rhs`
       (`rr[n_] := ReplaceAll[w[n], w[x_] :> rr[x - 1] + 1]`), whose
       recursion runs inside `evaluate(replaced)`. `replaceall-guard` already
       recurses inside the guard, below the changed function.
     - `stack-base/expected-22-g15b.txt`: the director's values for all 22
       shapes on the base.
     - `calib/build.sh`, `calib/measure.py`.
   - **Stack bases.** Build with `calib/build.sh` and measure with
     `calib/measure.py`. **All 22 shapes must equal `expected-22-g15b.txt`
     exactly; if one does not, STOP.**
   - **Resume harness.** Copy `.temp/brief-620/resume-harness/` to
     `.temp/brief-622/resume-harness/`. Add workloads in which a guard
     pauses, in each route; a `PatternTest` pauses; the replacement value
     pauses after a guard pause (two pauses); two guards pause. Each repeats
     N times and checks its final value. Before any source edit, rebuild
     `base/resume` and `base/resume-alloc` from the unedited working tree
     (the same paths the candidate will use; check its sources against
     `base-source/`) with the extended harness, one codegen unit and their
     own target directories, and record their hashes.
     Each workload's frame census must show the intended frames resuming
     once per pause.
2. **Labels.** Copy `.temp/brief-620/perf/labels.json` and its programs to
   `.temp/brief-622/perf/`. Add the new resume workloads as resume labels.
   - Name the ordinary labels that reach `resume_replace_all_after_rules` or
     its unheld twin. They must include `replaceall-simple`,
     `rule-rhs-deep-small`, `rewrite-guarded-hit`,
     `rewrite-guarded-decline-fallback`, `replaceall-leaf-rebuild-small` and
     any label that runs `ReplaceAllUnheld`.
   - Prove reachability with callgrind on the base.
   - If no ordinary label exercises `ReplaceAllUnheld` or an atom target,
     add one of each, checking its own result.
3. **Fixtures, red on the base.** Before changing anything, add this step's
   oracle fixtures and run them on the unedited tree, under `Checks::ALL`,
   with unconditional `FaultProbe` absence, the pause asserted at the
   intended phase, and the lifecycle checks.
   - **Tracing.** The trace-observing fixtures enable tracing in their
     session, as the oracle's `mutations/capture_output.rs` does, and
     compare the complete ordered trace.
   - **Red:**
     - the three traced DIFF rows above;
     - a `PatternTest` pause in each route;
     - a guard pause followed by a replacement-value pause;
     - two guard pauses;
     - nested `ReplaceAll`.
   - **Green controls:**
     - the three SAME rows above;
     - a pause in the target (owned by the existing target frame);
     - a pause only in the replacement value (owned by the existing value
       frame);
     - a guard that declines after its pause (no rule applies).
   - Record the red and green counts on the base in both builds.
4. **Implement the design.**
5. **Falsify the fixtures.** In a scratch copy of the candidate tree, make
   each route's new frame skip the shared finish and return its input (two
   mutants). Each mutant must turn that route's red fixtures red, while the
   final values stay equal. Report the counts; do not keep the mutants.
6. **Ordinary controls.** Rerun the 28 round-4 ordinary controls, as
   brief-620 did; all 28 must pass.
7. **Census and inventories.**
   - The census totals move by exactly the two typed frames and their
     witnessed sites. Report old → new.
   - Replay fingerprints, both certificate sets and ownership discovery, and
     disclose every changed row under the derived-facts class.
8. **Re-run brief-618's fixed set** (`.temp/brief-618/probe.rs`, unchanged,
   in a scratch copy of the tree with its own target directory). **Expected:
   114 SAME / 22 DIFF, the same 22 as at G15-B.** A pair that was SAME and
   is now DIFF is a STOP.

## Verification (MANDATORY — run to green or evidence)

**How to run.** Run serially, each command through `python3
scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-622/tmp …`.
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

- **The extra runs.** Also run the ordinary-controls harness in both builds,
  step 5's mutants, and step 8's fixed set.
- **The director's probes.**
  - Rerun the director's probes through a copy of
    `.temp/director/depth-leak-probe/` built under `.temp/brief-622/`
    against the candidate tree. Expected:
    - `s-g15b-probe` 20 SAME;
    - `g15-probe` 14 SAME;
    - `s10-probe` 51 SAME, with the same seven DIFF;
    - `s{9,8,7,6}-probe` all SAME.
  - Then run `.temp/director/c1-probe/cases.jsonl` through a copy of
    brief-621's comparator (`.temp/brief-621/probe/`) built against the
    candidate. Expected: all six SAME.
- **Native ZeroMQ.** Run the workspace with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`,
  report the nine as skips, and report every other count. The director runs
  them natively.

Report every `test result:` total, the oracle's fault-occurrence delta for
each build, and any compiler warning the diff introduces. Neither oracle
build may warn.

**Performance.** brief-620's six measurement items apply, on this step's
labels and shapes:

1. **Allocations** on every label. Ordinary labels must be exactly equal in
   slope and in whole-program counts. On the resume workloads, attribute
   every difference.
2. **Instruction slopes.** Two interleaved `perf stat` pairs per arm on all
   labels. Clear each flag by five runs, or by six callgrind processes per
   arm with every deterministic delta named. Report the own instructions per
   call of the two `resume_…_after_rules` functions (or wherever the finish
   lands) on step 2's labels, base and candidate.
3. **Copies**, with brief-620's calibrated census, recalibrated in your
   environment. No ordinary label may be above base. Also inspect the
   changed functions' prologues and stores.
4. **Stack**: all 22 shapes, debug and release, four heights with three
   samples. **The bar: every shape at or below base, `replaceall-guard` and
   `replaceall-rhs` in particular.** Tabulate the prologue of every
   function this step changes. **For each new frame's resume, report the
   chain of frames live at once** (the typed entry, any outlined callee, and
   every helper they call while live) and compare it with the base's
   resumed chain for the same workload. After any copy, ABI or inlining
   fix, rerun the stack shapes and the copy and instruction attribution on
   the final code.
5. **Resume attribution** on every resume workload, six callgrind processes
   per arm, deterministic deltas named.
6. **Timing** last, as brief-620 specifies: four full panels, then the
   aligned whole-program probe for any residual whose instructions are
   equal. `association-part-key-read-width4000` is excluded from the
   calibration bound (an instrument property on the unchanged base,
   brief-620's gate record).

A regression is a defect to fix or report. A residual goes to the report
with its attribution, and I rule on it.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- **No default-path cost.** Every `ReplaceAll` runs this function. The
  witness and the match must cost the ordinary path nothing measurable.
  G15-A needed its branch *and* its classifier outlined before the frame
  stopped growing: an attribute is not evidence, so measure the
  reservation. The suspend helpers are cold and take only
  `(witness, request)`.
- **The fixed point hides this defect.** A fixture that compares only the
  final value passes on the base. Every red fixture must observe the
  ordered trace, and the mutants in step 5 prove that it does.
- **A frame for the later `evaluate(replaced)` does not carry the earlier
  `apply_replace_all` call's tail.** The existing value frames own a
  different pause.
- **Typed from birth.** No synthetic `Expr` wrapper, decoder change,
  internal carrier, `unsafe`, transit record or raw legacy bridge.
- Actions, codecs and modes stay sealed and opaque. Profiling records stay
  balanced.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-622/`.
- After each step, and after every gate, write `PROGRESS.md` and a
  machine-readable `state.json`. They record:
  - the verified state;
  - the compile and test counters;
  - any half-edited files;
  - the exact next action.
- Append each report section to `.temp/brief-622/REPORT.md` as soon as its
  evidence is final, and copy the whole file into this brief at the end.
- If an upstream service interruption ends the run, the checkpoint is what
  the next session resumes from.

## Hard rules

- **No `git`, in any form.** No network.
- Modify only the Scope. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- **Commands allowed:**
  - scoped reads and edits, `rg`/`grep`, and bounded `python3`;
  - the bounded Cargo commands above and bounded compiler fixtures;
  - `scripts/check-suspension-oracle.sh` and
    `scripts/core-language-microbench.sh`;
  - `valgrind`, `callgrind_annotate`, `objdump`, `nm`, `perf` and `taskset`
    on the measurement binaries;
  - the director's probe binaries and brief-621's comparator, built under
    `.temp/brief-622/`.

  No probe may abort a process.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No change to an existing frame, no dual-arm decoder, no per-frame mode
  override, no transit record and no public raw adapter. Never weaken an
  oracle `Checks` flag, a retention reject, a fault-absence gate or a
  documented test to get a pass.
- **Stop-loss is keyed on progress.**
  - The counters: owned semantic failures (the fixtures and step 8's set),
    mutant survivals, and the allocation, instruction, copy and stack bars.
  - Compile errors you are reducing count as progress.
  - Stop after two attempts on a compiling tree that reduce none of them.
  - A failed bar's sub-ruling stops that sub-ruling, not the step.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. **Files changed**, each with its semantic change. Disclose every ripple
   and pin update as old → new, under its derived-facts case.
2. **The design as built.** The shared finish and its uses, the witnessed
   producer sites, both frames, and the second-pause route.
3. **The fixtures and harness workloads.**
   - What each exercises, its frame census, and its result on the base and
     the candidate in both builds.
   - The mutants.
   - Step 8's 136-pair result.
   - The director's probes and the six C1 cases.
4. **All gates, verbatim.** Fault-occurrence deltas, inventory and
   certificate transitions, and every canonical tool output path written.
5. **Measurements.** The allocation, instruction, copy (with the
   calibration), stack (with the resumed chains) and resume-attribution
   evidence, and timing with any alignment check.
6. **Stop-loss counters, deviations and open questions.** Add the witness
   rows S24's brief must keep.
