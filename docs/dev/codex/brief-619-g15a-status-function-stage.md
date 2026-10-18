# brief-619 — G15-A: a status-object call completes the Function stage on resume

## Goal

`Failure[tag, assoc][property]` and `Success[tag, assoc][property]` are
dispatched in the evaluator's Function stage. When the property pauses for
input, the resumed path skips the stage's tail. When you are done:

- a status-object call completes the same Function-stage work on the
  resumed path as on the queued-input path: the projection, the evaluation
  of the projected value, `finish_function_application` (which absorbs a
  `Return` there) and the `Some` of the stage result;
- this holds when the projection pauses, when the projected value's
  evaluation pauses, and when both pause;
- the status branch's two continuations are new typed frames, typed from
  birth;
- every other family, and every other branch of `try_apply_function`,
  still runs through the transport it uses today;
- the ordinary path of every program executes no more work than at S9.

This is step A of G15, a branch inserted into G13 before S10. The design is
brief-618: its report and the **director review at its end** govern, and
where `.temp/brief-618/draft-G15-A.md` differs from this brief, this brief
governs. S1-S9 are done (S9 is `f4a7c6b0`); the tree is `d0876e10`. G15-B
(brief-620, the definition LHS) and G15-C (brief-621) follow.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- **brief-618 in full**: the diagnosis (§2), the census table and the
  observation model (§3), the repair design ("Repair design and G13
  ownership"), the cost contract, and the director review.
- **S3-S9 as built are your template.** Read brief-617 in full, with its two
  continuations and its director gate record. Its rules apply here:
  - **The witness** comes from the actual producer `Result`, taken before
    any transformation, with `slot_witness`.
  - **The witness and the request cross a cold call, never the producer's
    `Result`.** A named `#[cold] #[inline(never)]` helper takes
    `(witness, request)` and the frozen state. A shape that needs an
    `unreachable!` arm for a state the types allow is the wrong shape.
  - **Ordinary-path matches** bind the producer `Result`, take the witness,
    `match` once, and send only `InputRequired` to the cold helper. S9's
    gate record shows why each arm's shape matters: passing the whole
    136-byte `Result<Expr, EvalError>` by value cost a memcpy per call, and
    the fix that removed it grew an inlined frame. Measure, don't assume.
  - **The routes**: witnessed; `DefinitionResumeRoute` for a site reached
    from the ordinary path and from its own frame's resume; witnessed then
    `ResumeStop` for a resume that inserts a different frame type.
  - **The token** carries no `Arc`, is not `Clone`, and re-inserts fresh
    state of its own frame type only.
  - `suspension/typed/witness.rs` is fixed. Do not change it.
- **The code.** All under `crates/rum-runtime/src/runtime/`:
  - `scoping/callable.rs`: `try_apply_function` (`:5`), the status branch
    (`:33`), the common tail (`:60-74`), `finish_function_application`
    (`:78-91`), F0528 (`:142-151`, S13's; do not change it);
  - `kernel_data/control_data/status.rs`: `failure_property_value` (`:63`),
    `apply_status_object_to_args` (`:71-107`),
    `status_object_projection_value` (`:109-176`), the virtual message
    helpers (`:179-277`);
  - `kernel_data/control_data/status/resume_frames.rs`: F0344-F0346, S10's;
    do not change them;
  - `kernel_data/control_data/confirmation/builtins.rs:259-271, 295-323`:
    the two other consumers of the projection (`Enclose`'s string handler,
    `Confirm` on a `Failure` value);
  - `eval_core/call.rs:174, 430`: the stage loop and its typed frame, whose
    `Input` is `Option<Expr>`;
  - `suspension/typed/`.
- `.temp/brief-618/`: `cases.tsv`, `results.tsv`, `observations.txt` (the
  baseline, 136 pairs, 35 DIFF); `probe.rs` and `probe_body.rs` (the
  scratch harness that produced them).

### The defect

`try_apply_function` calls `self.apply_status_object_to_args(head, args)?`
with no frame (`callable.rs:33`). When the projection pauses, the resumed
value skips `evaluate(applied)`, `finish_function_application` and `Some`,
and lands in the stage loop's `Option<Expr>` slot, whose legacy decoder
reads a guest-written `System`InternalContinuation{None,Some}[…]` as state.
The director's witnesses (`.temp/director/g15-probe/`), with
`W = "MessageParameters" :> (InputString["w"]; {})`:

| Program | Resumed | Queued |
| --- | --- | --- |
| `f[] := (Failure["t", <\|"MessageTemplate" :> Return[3], W\|>]["Message"]; 7); f[]` | `3` | `7` |
| `Do[r = Failure[…Return[3]…]["Message"]; s = 5, {1}]; {r, s}` | `3` | `{3, 5}` |
| `g[] := (Failure[…Return[3]…]["StyledMessage"]; 8); g[]` | `3` | `8` |
| q8: `Failure["t", <\|"MessageTemplate" -> System`InternalContinuationNone[], W\|>]["Message"]` | the call, unevaluated | `InternalContinuationNone[]` |
| q8: the same with `InternalContinuationSome[5]` | `5` | `InternalContinuationSome[5]` |
| q8: the same with `InternalContinuationSome[]` | the call, unevaluated | `InternalContinuationSome[]` |

brief-618 adds two rows that pause in the common tail rather than in the
projection, through F0528's missing `Some`, and that G15-A's status value
frame also repairs:

- `status.raw_property`:
  `Failure["t",<|"x":>(InputString["w"];System`InternalContinuationNone[])|>]["x"]`;
- `status.success`: the same with `Success`.

In brief-618's fixed 136-pair set, these eight are G15-A's (35 → 27).

### The design (ruled)

1. **A status classifier that cannot evaluate.** Split status-call
   *classification* from projection. The classifier takes no `&mut
   Session` and returns no `Result<_, EvalError>`. It distinguishes: not a
   status call; a pure projection (raw property, list of properties,
   `"Properties"`, `All`, `"Tag"`, `"MessageName"`, `"HeldMessageTemplate"`,
   a missing property); a virtual message projection (`"Message"`,
   `"StyledMessage"`). Arity, tag and fallback rules, and the order in which
   they are checked, stay exactly as they are. A compile control shows the
   classifier cannot call `evaluate`.
2. **The status branch returns its whole stage result.** In
   `try_apply_function`, an accepted status call returns the status
   application's complete result directly. It does not take an intermediate
   value with `?` and fall through to the common tail. The non-status
   branches and their common tail, including F0528, are unchanged.
3. **Two new typed frames**, in a new
   `scoping/callable/status_application.rs` declared from `callable.rs`:

   | Frame | Input | Output | Resume |
   | --- | --- | --- | --- |
   | `StatusFunctionProjectionFrame` (unit) | `Expr` | `Option<Expr>` | On `Ok`, run the shared value phase: evaluate the projected value, `finish_function_application`, `Some`. On `Err`, complete with it unchanged. |
   | `StatusFunctionValueFrame` (unit) | `Expr` | `Option<Expr>` | `finish_function_application(outcome).map(Some)`. |

   - The projection witness is taken at the actual projection `Result<Expr,
     EvalError>`, before any map, unwrap or `Some`.
   - The value witness is taken at the actual `evaluate(applied)` result.
   - If the projection frame resumes and the projected value's evaluation
     pauses again, insert the value frame with its own witness and complete
     the projection frame with `ResumeStop` (witnessed then `ResumeStop`).
   - **The two error boundaries differ.** A `Return` that arises while the
     projection runs propagates (`status.projection_return`, `3` on both
     paths). A `Return` from evaluating the projected value is absorbed. One
     `and_then(...).or_else(absorb)` around both phases is wrong.
   - Keep the `function.apply.evaluate` and `function.apply.finish`
     profiling phases on the status path exactly as the common tail has
     them. No duration spans a pause.
   - Both frames read `Expr` through `ValueMode`. Their `Some` is real typed
     output; the stage loop's frame may keep `Option<Expr>`'s current
     `LegacyMode` and decodes only that outer wrapper, so guest values
     inside it stay data. Read no guest `Expr` through
     `from_continuation_expr`.
4. **The other two consumers do not change.** `Enclose`'s string handler and
   `Confirm` on a `Failure` value take the projected `Expr`. They gain no
   `Some` and no Function-style `Return` absorption. They must keep their
   results on both paths, which brief-618's controls show today.
5. **Do not touch** F0344-F0346, M0355-M0357, T0030-T0034 (S10's), F0528 or
   M0563 (S13's), any registry mode, any decoder, or any reserved guest
   name. Do not add a carrier to disguise a missing frame.

## Scope

A listed file or class is permission to do this step's work there. It is
not permission to convert another step's rows, to change evaluation beyond
the status branch, or to tidy. Disclose every file you edit.

**Classes.**

- `crates/rum-runtime/src/runtime/scoping/callable.rs` and the new
  `scoping/callable/status_application.rs` (and a sibling `tests.rs` if
  needed). Keep new modules under 10k characters.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs`, for
  the classifier, the pure lookups and the narrow seam the new module
  needs. `control_data/mod.rs` and other module roots only for a visibility
  or re-export edit the change needs; disclose it.
- **Callers of a changed signature**: edit only the construct that changed.
- `crates/rum-runtime/src/runtime/suspension/`, only where the new frames
  expose a gap in the typed API or driver. Disclose each edit and why.
  `typed/witness.rs` is excluded.
- **Oracle**: `crates/rum-stdlib/tests/suspension_oracle/` — a new fixture
  module for this step, its registration, and the `typed_slots.rs`
  receipt and `typed-slot-stages.tsv` / `typed-slot-shapes.tsv` rows for
  the two new frames. The obligation-keyed context manifest is G15-C's, not
  yours: do not restructure `manifest.rs` or `descriptors.rs`.
- **Census**: `crates/rum-stdlib/tests/typed_slot_census/` and
  `typed_slot_census.rs`, to discover the two new frames and their sites.
- **Compile controls**: `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`
  and its fixtures, for the classifier control and any pin that follows a
  derived count.
- **New frames enter rum-runtime's closed lists**:
  `runtime/tests/frame_root_trace{.rs,/derived_inventory.rs}`, exactly the
  new types.
- **Derived facts follow the diff** (brief-613's class rule; see brief-617's
  Scope for the full table). For every derived inventory, frontier, census
  receipt, fingerprint, certificate or test pin that names a touched
  construct: a row on a deleted operation is retired, one on an added
  operation is new (it stays rejected or unknown and claims no mark), and a
  moved or renamed one is paired old → new. Disclose each. **STOP** on a
  change of *meaning* where the source did not change: a row on unchanged
  source disappears or changes kind, reason, status, disposition or
  exclusion; a mark, coverage or promotion is gained or lost; a new
  `UnknownHostPayload`; a production receipt change outside this step.
  Never hand-write a fingerprint, certificate cell or ledger chain.
  Ownership replays must consist of pure moves for unchanged constructs.
  **Citation fields** (for example `obligations.tsv`) must follow line
  moves; no test checks them.
- **Canonical tool outputs**: an unchanged canonical gate tool may write its
  own fixed output paths under `target/` or `.temp/`; disclose each.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md`, `.temp/brief-606/` and the core `Expr`/`Symbol` representation
stay unchanged.

**Disclosure duty.** If a needed change falls outside every class, make no
edit there. STOP and report the file, the line and why.

## Steps

1. **Bases, harness and calibration, before any source edit.** All scratch
   goes under `.temp/brief-619/`.
   - **Supplied** (verify `base-hashes.txt` first):
     - `base/rum-cli`, `base/rum-cli-alloc`: S9's final one-codegen-unit
       binaries, rebuilt at `d0876e10` with no recompiled unit and an equal
       hash; their build commands are beside them.
     - `base-source/`: the `d0876e10` export.
     - `stack-base/probe.rs`: S9's seventeen shapes and four new ones, each
       returning its depth (checked on `base/rum-cli` at depth 7):
       - `function-body`: `fb[n_] := Function[m, fb[m - 1] + 1][n]`;
       - `status-body`: `sb[n_] := Failure["t", <|"x" :> sb[n - 1] + 1|>]["x"]`;
       - `message-body`: `mb[n_] := Failure["t", <|"MessageTemplate" :> mb[n - 1] + 1|>]["Message"]`;
       - `lhs-arg`: `la[n_] := (hh[la[n - 1]] = n)` (for G15-B; measure it
         now so B has its base).
     - `stack-base/expected-17-s9-final.txt`: the director's S9 gate values
       for the seventeen old shapes, debug and release.
     - `calib/build.sh`, `calib/measure.py`: the director's calibration
       scripts (base-source plus the probe, one codegen unit, four heights,
       three samples each).
     - `perf-draft/expected.txt`: fourteen new ordinary labels with their
       base values.
   - **Stack bases.** Build the debug and release `runtime-tests` bases
     with `calib/build.sh` and measure all 21 shapes with
     `calib/measure.py`. **The seventeen old shapes must equal
     `expected-17-s9-final.txt` exactly; if one does not, STOP.** If a new
     shape fails in the runtime-only test session (S9's
     `catchexceptions-body` needed a scratch registration), fix it in the
     probe only, as S9 did, and record why. Every fit must have zero
     residual.
   - **Resume harness.** Copy `.temp/brief-617/resume-harness/` to
     `.temp/brief-619/resume-harness/`. Add workloads that pause:
     - in the projection (delayed `MessageParameters`; a delayed message
       definition, `f::m := (InputString[…]; "…")`; `StyledMessage`);
     - in the projected value's evaluation (raw `Failure` and `Success`
       properties with a delayed value);
     - in both, one after the other;
     - through each of the three consumers (Function, `Enclose`'s string
       handler, `Confirm` on a `Failure`).

     Each repeats its pauses N times and checks its final value. Build
     `resume` and `resume-alloc` from the unedited tree with one codegen
     unit, each in its own target directory. Their source hashes must match
     `base-source/`. Each workload's callgrind frame census must show the
     intended frames resuming the intended number of times.
2. **Labels.** Copy `.temp/brief-617/perf/labels.json` and its programs to
   `.temp/brief-619/perf/`. Add the fourteen labels of
   `perf-draft/expected.txt` in the same N/2N format, each checking its
   own result, and the new resume workloads as resume labels with their
   `resumes_per_iteration`. **The hot risk is not the status labels.**
   Every call with no matching definition walks `try_apply_function`'s
   whole branch chain, so `downvalue-exact-miss`, `downvalue-pattern-miss`,
   both `-miss-repeat` labels, `subvalue-unguarded-miss` and
   `default-unguarded-miss` are on this step's hot path.
3. **Fixtures, red on the base.** Before converting anything, add this
   step's oracle fixtures and run them on the unedited tree. Each compares
   the queued-input path with the resumed path under `Checks::ALL`, with
   unconditional `FaultProbe` absence, the pause asserted at the intended
   phase, and the lifecycle checks (pending owner, overlap rejection, empty
   input queue, fresh request).
   - **Red:** the eight G15-A rows above, each in brief-618's contexts that
     apply: `return_boundary` (`f[] := (subject; marker)`, and a loop and a
     store after the subject), `guest_heads` (`None[]`, `Some[5]`,
     `Some[]`, nested), projection then value pause.
   - **Green controls:** brief-618's eight original controls,
     `status.projection_return`, `status.list_property`,
     `status.value_pause`, `status.sideeffects` (with `log` initialised),
     held and spliced projected values (`Hold[Return[…]]`, `Unevaluated`,
     `Sequence[1, 2]`, `Nothing`, with sentinel siblings), the delayed
     message definition, and the two other consumers.
   - Record the red and green counts on the base. All red ones must turn
     green; none of the green ones may turn red.
4. **Implement the design**, following the rulings above.
5. **Ordinary controls.** Rerun the 28 round-4 ordinary controls through
   brief-611's harness: copy `.temp/brief-617/ordinary-controls/`, re-sync
   its `suspension_oracle/` support from the committed tree, and all 28
   must pass.
6. **Census and inventories.** Advance the census receipt to G15-A: at S9
   it is 606 implementations, 95 typed, 55 typed macros; report the actual
   totals with the two new frames and their sites. Replay fingerprints,
   both certificate sets and ownership discovery, and disclose every
   changed row under the derived-facts class.
7. **Re-run brief-618's fixed set.** Rerun `.temp/brief-618/probe.rs`
   unchanged in a scratch copy of the tree (its own target directory), and
   report the 136-pair result. **Expected: 27 DIFF, and every one of the 27
   is among the base's 35.** A pair that was SAME and is now DIFF is a
   STOP.

## Verification (MANDATORY — run to green or evidence)

**How to run.** Serially, each command through `python3
scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-619/tmp …`.
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

- Also: the ordinary-controls harness in both builds, and step 7's fixed
  set.
- Rerun the director's probes through a copy of
  `.temp/director/depth-leak-probe/` built under `.temp/brief-619/`
  against the candidate tree: `.temp/director/g15-probe/pairs.py`
  (expected: all 14 SAME), `.temp/director/s10-probe/pairs.py` (expected:
  the three q8 rows become SAME; the seven information-marker rows stay
  DIFF, S10's), and S9's, S8's, S7's and S6's pairs
  (`.temp/director/s{9,8,7,6}-probe/pairs.py`, all SAME). The probe
  answers `"1"` to every prompt and takes the pause cap first.
- **Native ZeroMQ.** In your sandbox the nine native ZeroMQ tests cannot
  bind loopback. Run the workspace with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`,
  report the nine as skips, and report every other count. The director runs
  them natively.

Report every `test result:` total, the oracle's fault-occurrence delta for
each build, and any compiler warning the diff introduces. Neither oracle
build may warn.

**Performance.** `TEMPLATE.md` pitfall 8 and brief-617's bars govern.
Build the candidates with the base commands and the same cwd, into separate
target directories under `.temp/brief-619/`. Preflight every N/2N run: the
result, and the exact resume count.

1. **Allocations and bytes first**, on every label. Ordinary labels must be
   exactly equal, in slope and in the whole-program counts at N and 2N. On
   the resume workloads, attribute every difference at its allocation
   sites: each new typed frame, its adapter `Arc`, queue capacity, and the
   transfers. "+24 B per typed transfer" is not the whole account of a new
   frame.
2. **Instruction slopes.** Two interleaved `perf stat` pairs per arm on all
   labels: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, stdin
   `/dev/null`. A flagged ordinary label is cleared by five runs per arm,
   or by six or more callgrind processes per arm with every deterministic
   delta named at its addresses. A deterministic register-allocation
   residual is reported with its addresses for the director to rule on.
   `try_apply_function` is on the ordinary path of every unmatched call:
   report its own instructions per call, base and candidate, on the six
   miss labels and on `function-slot-apply`.
3. **Copies.** The memcpy count on every ordinary label is at most the
   base's. **Calibrate the instrument in your environment first** — in S9
   another worker's census read a constant because its sandbox had no libc
   symbols:
   - copy `.temp/director/gate-617/callgrind/{census/run1.sh,copy_census.py}`
     to `.temp/brief-619/copy-census/` and point them there (`C` in
     `run1.sh`, `CENSUS_DIR`, `CENSUS_CAND=cand`);
   - copy `~/.cache/debuginfod_client` to `.temp/brief-619/debuginfod/` and
     set `DEBUGINFOD_CACHE_PATH` to the copy: valgrind's debuginfod client
     opens its cache read-write, which the sandbox refuses outside the
     repository;
   - **calibration:** callgrind must name `__memcpy_avx_unaligned_erms`,
     and `catchexceptions-handler` on the base must show a per-iteration
     memcpy slope that is non-zero and equal at N and 2N (S9's gate read 30
     on its base). A count that is constant across N measured nothing.

   Then run every ordinary label on both arms at N and 2N. Inspect the
   changed functions' prologues and stores as well, so an inlined copy
   cannot hide behind an unchanged libc call count.
4. **Stack.** Measure all 21 shapes, debug and release, at four heights
   with three samples each, against your calibrated bases. Tabulate the
   prologue frame size of every function this step changes, in both
   profiles. Any growth on any shape is reported with its slot table for
   the director to rule on. **After any copy, ABI or cold-helper fix, rerun
   the stack shapes and the copy and instruction attribution on the final
   code.** Earlier results do not validate the final layout.
5. **Resume attribution.** For every resume workload, six callgrind
   processes per arm, every function diffed per process, each deterministic
   delta named at its addresses. The driver's 136-B typed-outcome move is
   owed to R: count it, do not repair it.
6. **Timing** comes last, as an indication only. Four full
   `scripts/core-language-microbench.sh` panels, base, candidate, base,
   candidate, under `taskset -c 2`, profiling off. A residual whose
   instructions are equal is a layout question: settle it with both arms
   rebuilt with `RUSTFLAGS="-C llvm-args=-align-all-functions=6"`, using
   the panel's own generated program, whole, with `rumMicroReps` raised
   from 2 to 25 and nothing else changed, ten runs per arm alternated.
   Calibrate first: each label's per-iteration time in that probe must be
   within a small factor of its panel value.

A regression is a defect to fix or report. A stack or performance residual
is not a failing gate; it goes to the report with its attribution, and I
rule on it.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- **No default-path cost.** Every unmatched call runs `try_apply_function`.
  The classifier must cost no more than today's
  `is_failure_expr`/`is_success_expr` test on a non-status head: no new
  field, branch, store or copy. Keep head and arguments borrowed; build
  frame state only in the cold path. Move locals, not control flow.
- **The witness comes from the actual `Result`.** Never guess the input from
  a synthetic `Err` type.
- **A frame's existence, or equal `Input`/`Output` types, is not semantic
  completion.** Each phase's tail must run on both paths.
- **Return's owner depends on the phase**, as the design says.
- Actions, codecs and modes stay sealed and opaque. Nothing public names a
  mode or constructs an erased entry.
- Profiling records stay balanced on every path.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-619/`.
- After each step, and after every gate, write `PROGRESS.md` and a
  machine-readable `state.json`: the verified state, the compile and test
  counters, any half-edited files, and the exact next action.
- **Write the report as you go.** Append each report section to
  `.temp/brief-619/REPORT.md` as soon as its evidence is final, and copy
  the whole file into this brief at the end. brief-616 and brief-617 were
  each cut by a quota after their last gate; only what was on disk
  survived.

## Hard rules

- **No `git`, in any form.** No network.
- Modify only the Scope. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- Commands allowed: scoped reads and edits, `rg`/`grep`, bounded `python3`;
  the bounded Cargo commands above and bounded compiler fixtures;
  `scripts/check-suspension-oracle.sh` and
  `scripts/core-language-microbench.sh`; `valgrind`, `callgrind_annotate`,
  `objdump`, `nm`, `perf` and `taskset` on the measurement binaries; the
  director's probe binaries built under `.temp/brief-619/`. No probe that
  aborts a process.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No dual-arm decoder, per-frame mode override, transit record or public raw
  adapter. Type no row outside this step. Never weaken an oracle `Checks`
  flag, a retention reject, a fault-absence gate or a documented test to get
  a pass.
- **Stop-loss is keyed on progress.** Its counters: owned semantic failures
  (the fixtures and step 7's set), compile-control failures, and the
  allocation, instruction, copy and stack bars. Compile errors you are
  reducing count as progress. Stop after two attempts on a compiling tree
  that reduce none of them.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with its semantic change. Disclose every ripple,
   module root and pin update as old → new, under its derived-facts case.
2. The design as built: the classifier, the two frames, the call path and
   route of every new site, and the census output.
3. The fixtures and harness workloads: what each exercises, its frame
   census, and its result on the base and the candidate in both builds;
   step 7's 136-pair result; the director's probes.
4. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions. Every canonical tool output path written.
5. Allocation, instruction, copy (with the calibration), stack and
   resume-attribution evidence, and timing with any alignment check.
6. Stop-loss counters, deviations and open questions.
