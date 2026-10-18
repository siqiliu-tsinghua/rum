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


---

<!-- brief-619 execution report appended by the executing agent -->

## Director continuation 1 (2026-10-18) — outline the status branch

The semantics, census, inventories and allocation evidence are accepted as
reported. One stack residual fails the bar, and the fix changes code, so
everything the fix can move is measured again.

**The defect.** In release, `apply_status_function` has no inline attribute
and is inlined into `try_apply_function`. Its locals now sit in the frame of
every Function-stage call: the reservation grows `0x458` → `0x4a8`, +80 B per
level on `function-body`, which is the shape of any recursion through a pure
`Function`. The bar is no growth beyond the typed macro's named closure
environment, and this is not that. CLAUDE.md's rule for a hot recursive
frame applies: *move locals, not control flow*. A cold branch's state does
not belong in the frame of the common path.

**The fix.** Mark `apply_status_function` `#[inline(never)]`, so the status
branch is one call out of `try_apply_function`. Change nothing else about the
design. If the classifier's own result still adds frame bytes to
`try_apply_function`, move it the same way. **Bar:** `try_apply_function`'s
release and debug reservations are at or below base (`0x458` release; 3 256
B debug), and `function-body` is at or below base in both profiles.

**Then rerun, on the final code:**
- `cargo fmt --check`; the status fixtures and the full oracle in both
  builds; `typed_slot_census` and the compile controls in both builds;
  `frame_root_trace`; `ownership_graph` and `local_production_acceptance` in
  both builds, replaying derived facts for any moved line (pure moves
  only); `production_rejection_census_is_complete`;
- step 7's fixed set (expected 109 SAME / 27 DIFF, as now);
- all 21 stack shapes, debug and release, with the prologue and slot tables
  of every function the fix changes;
- instruction slopes, five runs per arm, on `function-slot-apply`,
  `function-named-apply`, `composition-apply`, `association-apply`, all
  fourteen new status labels, the six miss labels, `literal-integer`,
  `do-count-null`, `do-iterator-null` and `block-local`;
- the calibrated copy census on all ordinary labels;
- the resume allocation comparison on the eight new workloads.

The workspace and timing do not need rerunning; I run the full gate
afterwards.

**Rulings on the other residuals**, unless the fix changes them:
- `block-local` +9 instructions per iteration, all in libc `__memcmp`'s
  short-string page-boundary path at equal calls: layout, accepted as S9
  accepted `matcher-single-tiny`'s +6.7.
- `confirm-failure-message` 86 → 87 memcpy per iteration: the seven-byte
  `"Message"` literal moved from inline stores to a libc call, with the
  affected functions' instructions −22 per iteration. The copy bar exists to
  catch aggregate `Result` moves; this is not one. Accepted as named, if it
  remains after the fix; report its count again.
- `status-body` and `message-body` growth: the status path now keeps a
  separate value-phase frame while its recursion runs. Accepted for this
  step because it is what repairs the raw-property rows before S13. It is
  recorded as owed to S13, which types F0528 and can then share one value
  phase. Report the final numbers after the fix.
- Aligned timing, 0 labels disjoint above base: accepted.

**Report.** Add a section *Continuation 1* at the end of the report: the
change, the before/after reservations and slot tables, every rerun result
verbatim, and the final residuals. Correct any earlier statement the fix
makes stale. Checkpoint as before.

---

# brief-619 execution report

G15-A is implemented and the owned semantic defects are repaired. All final semantic, census, compiler-control and inventory gates pass. The requested performance measurements are complete; the instruction, copy, stack and indicative timing residuals below are submitted for the director's ruling under the brief's residual rule. No version-control command or external network request was run. The tree is left uncommitted.

All artifact paths in this report are relative to `.temp/brief-619/` unless they begin with `crates/`, `docs/`, `target/`, `.temp/` or an absolute path. The six sections below are the review index; the preserved evidence ledger follows with actual gate output, row transitions and measurement tables.

## 1. Files changed and derived-fact ripples

Exactly 18 existing source/inventory files changed and six files were added. The only documentation write is this report's required append to the brief. `final-scope-audit.json` verifies all 2,079 original export hashes, all four candidate build snapshots, the closed scope and the protected code.

| File | Semantic or derived-fact change |
|---|---|
| `crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs` | Re-export the private runtime classifier and projection-plan type for the callable module; no other module-root change. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs` | Replace the old fallible status-call selector with a session-free classifier, extract pure property fallback, and expose a narrow virtual-message projection seam. Existing S10 frames and macro sites stay unchanged. |
| `crates/rum-runtime/src/runtime/scoping/callable.rs` | Declare the status_application child and return the complete status Function-stage result directly; all other branches and the common F0528 tail stay unchanged. |
| `crates/rum-runtime/src/runtime/scoping/callable/status_application.rs` | New unit projection/value typed frames, actual-result witnesses, two cold suspension helpers and shared evaluation/finish/Some phase with distinct Return ownership. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | Derived named-frame pin 563 -> 565. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | Add exactly the two new unit types to both closed derive inventories. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Replay coordinate moves and add the two new frame-derive expansion unknowns; no disposition or mark changes. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Replay coordinate moves (including extracted fallback closure) and add five new closure-capture unknowns. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Replay macro-span moves and add the classifier matches! expansion unknown. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Install exactly 68 canonical fingerprint cells; all other contract fields unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Install four canonical certificate cells for two existing bodies in both builds; all non-hash contents and coverage unchanged. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | Register the new status_function fixture module. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/mod.rs` | Shared strict fixture runner with Checks::ALL, unconditional fault absence, exact values/logs/prompts and existing lifecycle checks. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/controls.rs` | 29 green status/other-consumer/held/spliced/projection-and-value controls. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/guest_heads.rs` | 16 guest-marker preservation cases across Message and raw Failure/Success properties. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/return_boundaries.rs` | Eight phase-specific Return, enclosing-body, loop/store and consecutive-pause cases. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` | Receipt stage S9 -> G15-A. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | Append two typed frame rows, their two witnessed sites and implemented G15-A stage; all existing rows untouched. |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | Register G15-A checks; update current-stage and derived forced-red partition pins (+2 implementations). |
| `crates/rum-stdlib/tests/typed_slot_census/g15a.rs` | New exact cohort checks for both Expr -> Option<Expr> frames and witnessed insertion routes; no global-mode change. |
| `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` | Production implementations 606 -> 608, combined 622 -> 624; frame IDs 625 -> 627 and site IDs 603 -> 605; stage output and witnessed-route recognition updated. |
| `crates/rum-stdlib/tests/typed_slot_census/reporting.rs` | Include the new callable child in witnessed-site reporting. |
| `crates/rum-stdlib/tests/typed_slot_census/s4.rs` | Recognize G15-A as a later cohort when preserving S4 counts. |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | Add the classifier-without-Session compiler pair; positive/rejection totals 35 -> 36; all existing cases retained. |
| `docs/dev/codex/brief-619-g15a-status-function-stage.md` | Append this entire report, as explicitly requested. |

The one module-root ripple is the private classifier/projection-plan re-export in `control_data/mod.rs`. No suspension API/driver file changed. The six new modules are each below 10,000 characters. Existing slot shapes already cover Expr and Option<Expr>, so `typed-slot-shapes.tsv` needs no new shape. Both obligation citation inventories contain no reference to either moved source file; no citation update is needed there.

Ownership: 46 pure coordinate moves and eight new UnknownConstruct rows (five closures, one matches expansion, two derives); no new UnknownHostPayload, retired ownership row, mark or disposition change. Canonical tools regenerate 68 fingerprint cells and four certificate cells for the two existing bodies in both builds; non-hash contract fields, marked-body/instance sets, coverage and promotions remain unchanged. Rejection census: 48,696 → 48,713 rows, with 48,693 paired, 184 pure coordinate moves, two paired loop/push rows following the changed producer, three retired Try operations and 20 new rejected operations. No unchanged-source row changes meaning. The complete old → new rows and canonical output paths are in the ledger and JSON/TSV manifests.

## 2. Design as built and census

`classify_status_object_call` is a free, infallible function with no Session argument. It keeps the existing head/tag/arity order and separates pure projections from the two virtual-message projections. Its new negative compiler control rejects `self.evaluate` (E0424). The status branch returns its complete Function-stage result directly. Reconstructing only that branch plus its import/module declaration yields the exact original `callable.rs`; all other branches, the common tail and F0528 are byte-identical. The existing S10 virtual-message helpers/macros and F0344–F0346 remain byte-identical.

| New unit frame | Input → output | Resume boundary and route |
|---|---|---|
| `StatusFunctionProjectionFrame` | Expr → Option<Expr> | Projection Err propagates unchanged. Ok enters the shared value phase, then finish/Some. A second pause inserts the distinct value frame with its own witness and ends through ResumeStop/complete_work. |
| `StatusFunctionValueFrame` | Expr → Option<Expr> | Applies finish_function_application(outcome).map(Some); projected-value Return is absorbed. |

Each ordinary producer binds its actual Result, takes slot_witness before transformation, matches once, and sends only witness/request into a named cold non-inlined helper. Unit-frame state is constructed only on a pause. Tokens are neither cloned nor stored. Input Expr uses ValueMode; the real outer Some uses the existing Option codec. No guest Expr is decoded as a continuation slot by either new frame. The evaluate and finish profiling phases remain balanced. Enclose/Confirm retain their own projection and Return boundaries.

Census: production implementations **606 → 608**, typed implementations **95 → 97**, typed macros **55 → 55**, witnessed sites **46 → 48**; combined implementations **622 → 624**, frame identities **625 → 627**, site identities **603 → 605**. Legacy implementations remain 461, finalizers 50, macro sites 542 (343 result / 199 insert), registry modes one Value and 21 Legacy. Closed named frames **563 → 565**; compiler-control pairs **35 → 36**.

## 3. Fixtures, workloads and behavioral result

All 53 new fixtures compare queued and resumed execution under Checks::ALL, unconditional fault absence, exact phase/prompt/result checks and existing lifecycle checks. Base results in both builds: **36 pass / 17 fail**. Candidate results in both final oracle builds: **53 pass / 0 fail**. The fixture families contain 29 controls, 16 guest-head cases and eight Return-boundary cases; all 36 original green cases stay green. Full oracle totals are 422 per build. The controls include list/raw properties, side effects, projection Return, delayed message definitions, held/spliced values and both other consumers.

The unchanged brief-618 set is **109 SAME / 27 DIFF** (base 101 / 35); exactly the eight owned rows become SAME, and no previous SAME becomes DIFF. All 27 remaining differences were already in the base. Director G15: 14 SAME; S10: 51 SAME / seven existing information-marker DIFF, with all three q8 rows now SAME. S9/S8/S7/S6: 61/48/32/22 SAME. Round-4 ordinary controls: 28/28 in both builds.

| New resume workload | Checked item | Pauses/iteration | Intended frame census change |
|---|---|---:|---|
| delayed parameters | `"m 1"` | 1 | F0345 + one new projection frame |
| delayed message definition | `"m 1"` | 1 | F0346 + one new projection frame |
| StyledMessage | `Style["m 1", "Message"]` | 1 | F0345/F0344 + one new projection frame |
| raw Failure | `17` | 1 | F0528 replaced on this branch by one new value frame |
| raw Success | `17` | 1 | F0528 replaced on this branch by one new value frame |
| projection then value | `19` | 2 | F0345 + one projection and one value frame |
| Enclose string handler | `"m 1"` | 1 | F0345 only; no new G15 frame |
| Confirm Failure | `"m 1"` | 1 | F0345 and FinishConfirmFailure; no new G15 frame |

Each new workload passes N=100/200, including exact resume counts; both arms use the same harness source. Baseline and candidate preflights each pass all 740 runs across 178 ordinary and 103 resume labels. The new-workload base frame census and all 96 candidate sample/size frame censuses pass. All 2,472 full resume callgrind processes also pass their value/count checks.

## 4. Gates, faults, inventories and canonical outputs

The ledger below preserves every test-result total verbatim, including expected red baseline fixtures, discovery failures and nested negative census probes. Full stdout/stderr is retained in the cited raw logs. Final cargo fmt passes; focused runtime tests pass 24 typed, 15 root-trace, 11 resume-context and one bounded-recursion case. Compiler controls pass both builds with all 36 positive and 36 intended rejection cases. Census passes 8 with one intentional ignored probe in each build. Ownership passes 31; local production acceptance passes both builds; the complete rejection census passes. Workspace exits 0; its 55 reported test-result lines are preserved, including the two deliberate nested census failures.

Both final oracle builds pass **422/422**, report **fault-occurrence delta 0 / 93,750 observations**, and emit **zero compiler warnings**. Ordinary controls each report delta 0 / 336 observations; the fixed set reports delta 0 / 1,620. The base fixtures report delta 0 / 840 and the focused candidate run delta 0 / 636. Every intermediate oracle fault line is zero. Runtime-only helper dead-code warnings already exist in the base; the workspace compiler-fixture/doc warnings are recorded. No changed production source introduces a warning.

The nine native ZeroMQ cases are **skipped**, not successful native executions: loopback bind is unavailable and RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 was set as instructed. Their exact names/reasons are listed in the ledger and `workspace-native-skips.json`; the director runs them natively. No other requested final gate is omitted.

Canonical gate outputs outside brief-619 are explicitly allowed by the brief. `canonical-output-paths.tsv` enumerates every fixed-output artifact path and size; `canonical-output-roots.json` groups them. Fingerprints come from `target/brief-544/local-contracts.tsv`; both certificate sets come from `.temp/brief-546/certificates/promoted-hash-{true,false}.tsv`. Only canonical generated cells were installed; none was hand-written.

## 5. Allocation, instruction, copy, stack, resume and timing evidence

**Allocations first:** all 178 ordinary labels have exact equality in whole-program allocations/bytes at N and 2N and in slopes. Of 103 resume labels, 97 are raw-exact. Only the six repaired Function consumers grow: parameters/template/styled +5 allocations/+160 B; raw Failure/Success +4/+144 B; double pause +8/+312 B, all with zero intercept delta. The report reconciles every site: 16-B unit-frame Arc, 24-B adapter, 32+48+40-B Some wrapper, replacement of the old 16-B F0528 Arc, and the double-pause history change 128+40−56=112 B. Queue growth shifts callers -1/+1 with equal total cost.

**Instructions:** all 281 labels have two interleaved N/2N perf samples per arm. Five additional samples per arm clear 24 of 25 ordinary flags. `block-local` remains about +9.115 instructions/iteration; six callgrind N/2N pairs per arm identify exactly +9 deterministic libc memcmp instructions at the named addresses. The short-string page-boundary path changes with address layout; this residual is reported for ruling. Function-slot self cost improves 121 → 119 instructions/call. All six required miss labels execute zero try_apply_function calls in both binaries: the unchanged dispatch planner omits the Function stage for those heads. Their per-call cost is N/A, not an inlining assumption.

**Copies:** calibration resolves __memcpy_avx_unaligned_erms and produces counts 16,013 / 31,013 / 61,013 at 500 / 1,000 / 2,000 iterations, slopes 30/30. All 712 ordinary census runs pass. 177/178 labels meet the copy bar; `confirm-failure-message` has 86 → 87 memcpy calls/iteration. Disassembly identifies the same seven bytes of the Message literal: base inline overlapping stores at 0x601df9/0x601e00 become candidate libc copy at 0x612ef6. It is not an extra Result payload copy. The four deterministic affected function totals sum to -22 instructions/iteration. The strict libc-call-count bar remains a reported residual.

**Stack:** calibrated base and final candidate each pass all 504 probes; all 42 fits have zero residual. The original 17 shapes and lhs-arg are unchanged. Function-body changes -448 B/debug and +80 B/release per level. Status/message change +1,056 B/debug and +784 B/release. Full changed-function prologues, inline stores and slot tables reconcile these values: debug -448 + (552+8) + (936+8) = +1,056; release +80 + (696+8) = +784. These five growth rows remain for the director.

**Resume:** all 103 workloads have six samples per arm at N/2N. All function sums reconcile for every process; 334 deterministic nonzero function deltas are named with all addresses, and variable deltas remain visible. Twelve older workloads show only reproducible reductions (-1 or -7 in try_apply_function, nine also -10 in the empty-value drop). Driver sites {0x74a33d, 0x74a35d} on the base and {0x74b34d, 0x74b36d} on the candidate uniquely equal all typed-adapter resume counts. Each new typed resume adds one 136-B move, two in the two-pause case; all other driver 136-B counts are equal. The driver is unchanged and that transfer remains R's work.

**Timing last:** four full panels, 120 labels each, CPU 2, profiling off. Both arms then receive the prescribed 64-byte function-alignment rebuild and ten alternating whole-program runs with only reps 2 → 25. All 240 calibration values pass the 0.5–2 factor (0.725831–1.515593). Of the initial 32 disjoint-above timing labels, the aligned protocol leaves 0 disjoint-above rows overall. All 120 rows, medians, ranges and raw runs are retained. Timing remains an indication rather than a semantic gate.

No aligned timing label has its complete candidate range above the complete base range. Positive medians remain indicative: the largest is +6.2402% (six of ten pairs slower); all paired values and higher medians are retained for the director.

## 6. Stop-loss, deviations and open questions

One runtime implementation attempt. Owned fixture failures **17 → 0**; owned fixed-set differences **8 → 0**; compile-control failures **0**; consecutive non-progress attempts **0**. The compile-command counter is **38**; the gate/test/performance ledger contains **80** entries. These counters count top-level recorded commands, not nested rustc invocations. Ordinary allocation-bar violations: **0**; unresolved ordinary instruction labels: **1**; copy-count labels: **1**; stack-growth rows: **5**. No runtime/ABI/cold-helper change was made after the final measurement build.

Initial formatting checks emitted hunks; only owned files were formatted and the final check passed. Ownership/certificate discovery initially failed on stale derived rows/cells, which were replayed under the explicit class rule. Initial harness instrumentation used an over-broad frame substring and an unqualified Style expected expression; these were corrected before the baseline measurements. Initial libc-symbol calibration did not work until the writable offline cache and extra debug path were used; no constant-count run was accepted. A scratch helper initially described zero Function calls as inlining; the corrected report and rerun identify the dispatch planner instead. The initial shorthand about unchanged queue callsites was corrected to the measured -1/+1 caller shift.

Two director-probe Python imports created bytecode files outside brief-619; only those newly created files were moved under `probe-bytecode-deviation/`, their empty directories removed, and subsequent runs disabled bytecode writes. Older director artifacts were preserved. This corrected scratch-path deviation is disclosed, not hidden. The brief append is the explicitly requested exception to the documentation freeze. All source and derived-fact scope checks pass; no new unsafe, dependency, lockfile, toolchain, codec, mode, reserved guest name or out-of-cohort typed row was introduced.

Open director decisions are the attributed block-local instruction/layout residual, Confirm libc-copy count, the five stack-growth rows, and the indicative aligned timing median increases. The aligned timing check leaves no disjoint-above label, but does not establish exact timing equality. The nine native ZeroMQ tests remain for the native director run. No owned semantic defect, compile-control failure, unexplained allocation delta, driver-transfer remainder, or mandatory measurement remains unfinished.

## Preserved evidence ledger


## Initial evidence

All six supplied SHA-256 hashes in `base-hashes.txt` match; the verification is recorded in `hash-verification.json`. No source edits have been made. No version-control command or network request has been run.

## Step 1: source and stack calibration

All 2,079 supplied export files match the live tree byte for byte. The supplied `calib/build.sh` built both runtime-test binaries successfully. `calib/measure.py` ran 504 probes. All 34 established profile/shape slopes equal `expected-17-s9-final.txt`; all 42 fits have zero residual. No probe change was needed.

Build output:
```text
42bebf69694e9e82a4559d17b2765b9f40f4c0ff12527c3317c68dfe9c13dd3c  /home/remi/dev/rum/.temp/brief-619/calib/debug/runtime-tests
aa3be40b7d2ff9a715694c7435b21ffb3611b64e6a1dce91ac80480ad46c929c  /home/remi/dev/rum/.temp/brief-619/calib/release/runtime-tests
```

Calibration output:
```text
debug peel 19024.0 resid 0.0 top 189848
debug plus 34912.0 resid 0.0 top 313000
debug sub 29376.0 resid 0.0 top 267128
debug up 28064.0 resid 0.0 top 255320
debug guarded 35840.0 resid 0.0 top 320424
debug rhs-guard 49152.0 resid 0.0 top 445864
debug replace-guard 64880.0 resid 0.0 top 552744
debug replaceall-guard 63296.0 resid 0.0 top 540072
debug count-guard 74064.0 resid 0.0 top 626216
debug position-guard 85648.0 resid 0.0 top 718888
debug catch-body 50528.0 resid 0.0 top 437928
debug checkabort-body 52720.0 resid 0.0 top 455464
debug abortprotect-body 49904.0 resid 0.0 top 432936
debug cleanup-body 50880.0 resid 0.0 top 440744
debug check-body 50432.0 resid 0.0 top 437160
debug quiet-body 50304.0 resid 0.0 top 436136
debug catchexceptions-body 49872.0 resid 0.0 top 432680
debug function-body 49760.0 resid 0.0 top 431784
debug status-body 49760.0 resid 0.0 top 431784
debug message-body 49760.0 resid 0.0 top 431784
debug lhs-arg 58400.0 resid 0.0 top 500904
release peel 7344.0 resid 0.0 top 159625
release plus 13792.0 resid 0.0 top 287289
release sub 13168.0 resid 0.0 top 275481
release up 11456.0 resid 0.0 top 239529
release guarded 14240.0 resid 0.0 top 296249
release rhs-guard 17920.0 resid 0.0 top 375513
release replace-guard 26768.0 resid 0.0 top 546809
release replaceall-guard 24640.0 resid 0.0 top 504249
release count-guard 28416.0 resid 0.0 top 579769
release position-guard 33632.0 resid 0.0 top 684089
release catch-body 19280.0 resid 0.0 top 397049
release checkabort-body 20864.0 resid 0.0 top 428729
release abortprotect-body 19280.0 resid 0.0 top 397049
release cleanup-body 19280.0 resid 0.0 top 397049
release check-body 18736.0 resid 0.0 top 386169
release quiet-body 18688.0 resid 0.0 top 385209
release catchexceptions-body 19472.0 resid 0.0 top 400889
release function-body 19376.0 resid 0.0 top 398969
release status-body 19376.0 resid 0.0 top 398969
release message-body 19376.0 resid 0.0 top 398969
release lhs-arg 18160.0 resid 0.0 top 374649
```

Baseline `resume` built with one codegen unit; hash `c8a67e3b47f43c9944cdac848ac568558cb7106fcd0c3da8c8d85ec1918ee3f7`. All 2,079 source files still match the supplied export. Build stderr:
```text
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling syn v2.0.117
   Compiling version_check v0.9.5
   Compiling generic-array v0.14.7
   Compiling autocfg v1.5.0
   Compiling typenum v1.20.1
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling serde_derive v1.0.228
   Compiling cfg-if v1.0.4
   Compiling thiserror v2.0.18
   Compiling digest v0.10.7
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling find-msvc-tools v0.1.9
   Compiling shlex v2.0.1
   Compiling cc v1.2.63
   Compiling num-bigint v0.4.6
   Compiling cpufeatures v0.2.17
   Compiling crc32fast v1.5.0
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling libmimalloc-sys v0.1.49
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling mimalloc v0.1.52
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling brief608-resume v0.0.0 (/home/remi/dev/rum/.temp/brief-619/resume-harness)
    Finished `release` profile [optimized] target(s) in 1m 36s
```

Baseline `resume-alloc` built with one codegen unit; hash `95e8e18ea5edcafad228a7c7bd0ddb3623743fe769d42a36c8a7cb0dc293812f`. All 2,079 source files still match the supplied export. Build stderr:
```text
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling syn v2.0.117
   Compiling version_check v0.9.5
   Compiling generic-array v0.14.7
   Compiling autocfg v1.5.0
   Compiling typenum v1.20.1
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling serde_derive v1.0.228
   Compiling cfg-if v1.0.4
   Compiling thiserror v2.0.18
   Compiling digest v0.10.7
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling find-msvc-tools v0.1.9
   Compiling shlex v2.0.1
   Compiling cc v1.2.63
   Compiling num-bigint v0.4.6
   Compiling crc32fast v1.5.0
   Compiling cpufeatures v0.2.17
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling libmimalloc-sys v0.1.49
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling mimalloc v0.1.52
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling brief608-resume v0.0.0 (/home/remi/dev/rum/.temp/brief-619/resume-harness)
    Finished `release` profile [optimized] target(s) in 1m 34s
```

## Step 1: resume harness

Eight new workloads exercise all three projection routes, raw Failure/Success values, successive projection/value pauses, Enclose string handling, and Confirm failure messages. Both baseline binaries pass at N=100 and 2N=200; exact input prompts and final values are asserted. The actual frame-call census matches the intended frame count at both sizes. Scratch census attempt 1 failed before running workloads because a type-name substring also matched `FinishFailureMessageFromTemplateSuspensionFrame`; the matcher now requires the complete type path.

```text
PASS g15-parameters resumes=100 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 100, "expected": 100}]
PASS g15-parameters resumes=200 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 200, "expected": 200}]
PASS g15-template resumes=100 frames [{"address": "00000000005efbf0", "frames": ["FinishFailureMessageFromTemplateSuspensionFrame"], "actual": 100, "expected": 100}]
PASS g15-template resumes=200 frames [{"address": "00000000005efbf0", "frames": ["FinishFailureMessageFromTemplateSuspensionFrame"], "actual": 200, "expected": 200}]
PASS g15-styled resumes=100 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 100, "expected": 100}, {"address": "00000000005ef6e0", "frames": ["FailureVirtualStyledMessageValueSuspensionFrame"], "actual": 100, "expected": 100}]
PASS g15-styled resumes=200 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 200, "expected": 200}, {"address": "00000000005ef6e0", "frames": ["FailureVirtualStyledMessageValueSuspensionFrame"], "actual": 200, "expected": 200}]
PASS g15-raw-failure resumes=100 frames [{"address": "000000000071c5c0", "frames": ["TryApplyFunctionSuspensionFrame"], "actual": 100, "expected": 100}]
PASS g15-raw-failure resumes=200 frames [{"address": "000000000071c5c0", "frames": ["TryApplyFunctionSuspensionFrame"], "actual": 200, "expected": 200}]
PASS g15-raw-success resumes=100 frames [{"address": "000000000071c5c0", "frames": ["TryApplyFunctionSuspensionFrame"], "actual": 100, "expected": 100}]
PASS g15-raw-success resumes=200 frames [{"address": "000000000071c5c0", "frames": ["TryApplyFunctionSuspensionFrame"], "actual": 200, "expected": 200}]
PASS g15-projection-value resumes=200 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 100, "expected": 100}]
PASS g15-projection-value resumes=400 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 200, "expected": 200}]
PASS g15-enclose resumes=100 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 100, "expected": 100}]
PASS g15-enclose resumes=200 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 200, "expected": 200}]
PASS g15-confirm resumes=100 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 100, "expected": 100}, {"address": "00000000005da020", "frames": ["FinishConfirmFailureSuspensionFrame"], "actual": 100, "expected": 100}]
PASS g15-confirm resumes=200 frames [{"address": "00000000005ef830", "frames": ["FailureMessageFromTemplateSuspensionFrame"], "actual": 200, "expected": 200}, {"address": "00000000005da020", "frames": ["FinishConfirmFailureSuspensionFrame"], "actual": 200, "expected": 200}]
PASS 8 workloads; N/2N results, resume counts, and intended frame census
```

## Step 2: labels and copy calibration

Inherited 164 ordinary and 95 resume labels; added the prescribed 14 ordinary labels and 8 resume labels, for 281 total. All 740 baseline check/N/2N executions pass and resume counts are exact. The styled-message checker initially exited 2 because its expected expression used an unqualified `Style`; generated N/2N checkers now use `System`Style`. The raw failed run is retained as `perf/base-preflight-attempt1.log` and `perf/check/base/1/attempt1-results.json`.

Copy calibration initially lacked libc symbols. Empty `DEBUGINFOD_URLS` disables Valgrind's debuginfod lookup even for its cache; the copied libc debug object has the running libc's exact build id. A local debug-object layout under `debuginfod/local/`, supplied through `--extra-debuginfo-path` with debuginfod disabled, resolves the symbols without network access. This is a calibration-method deviation, with all files under this brief. No mismatched-debug option is used.

```text
base catchexceptions-handler 500 rc=0 Null
CALIBRATION {"N": 500, "memcpy": 16013, "symbols": ["__memcpy_avx_unaligned_erms", "__memcpy_chk", "memcpy@@GLIBC_2.14"]}
base catchexceptions-handler 1000 rc=0 Null
CALIBRATION {"N": 1000, "memcpy": 31013, "symbols": ["__memcpy_avx_unaligned_erms", "__memcpy_chk", "memcpy@@GLIBC_2.14"]}
base catchexceptions-handler 2000 rc=0 Null
CALIBRATION {"N": 2000, "memcpy": 61013, "symbols": ["__memcpy_avx_unaligned_erms", "__memcpy_chk", "memcpy@@GLIBC_2.14"]}
PASS symbolized memcpy slopes 30.0 30.0
```

## Step 3: baseline oracle fixtures (default)

53 strict fixtures: 36 green and 17 red on unchanged runtime code. The eight original owned differences are included, with nine further guest-head / Return contexts. All 19 inherited green controls and the added positive controls pass. Checks::ALL and unconditional fault absence remain enabled. Full observations: `logs/fixtures-base-default.log`.

```text
test result: FAILED. 36 passed; 17 failed; 0 ignored; 0 measured; 369 filtered out; finished in 0.47s
continuation-slot fault-occurrence delta=0; observations=840
```

Failed tests:
```text
oracle::fixtures::status_function::guest_heads::director_04
oracle::fixtures::status_function::guest_heads::director_05
oracle::fixtures::status_function::guest_heads::director_06
oracle::fixtures::status_function::guest_heads::failure_empty_some
oracle::fixtures::status_function::guest_heads::failure_nested
oracle::fixtures::status_function::guest_heads::failure_some
oracle::fixtures::status_function::guest_heads::message_nested
oracle::fixtures::status_function::guest_heads::status_raw_property
oracle::fixtures::status_function::guest_heads::status_success
oracle::fixtures::status_function::guest_heads::success_empty_some
oracle::fixtures::status_function::guest_heads::success_nested
oracle::fixtures::status_function::guest_heads::success_some
oracle::fixtures::status_function::return_boundaries::director_01
oracle::fixtures::status_function::return_boundaries::director_02
oracle::fixtures::status_function::return_boundaries::director_03
oracle::fixtures::status_function::return_boundaries::projection_value_return
oracle::fixtures::status_function::return_boundaries::styled_loop_store
```

## Step 3: baseline oracle fixtures (no default features)

The same 17 tests fail and the same 36 pass. No compiler warning in either baseline fixture build.

```text
test result: FAILED. 36 passed; 17 failed; 0 ignored; 0 measured; 369 filtered out; finished in 0.46s
continuation-slot fault-occurrence delta=0; observations=840
```

## Step 4: first semantic result

The first runtime candidate passes all 53 status fixtures: all 17 base failures turn green and all 36 base controls stay green. No compiler warnings.

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 369 filtered out; finished in 0.37s
continuation-slot fault-occurrence delta=0; observations=636
```

### Gate: census-discovery

Command: `cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture`. Exit 0. Full raw output: `logs/census-discovery.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 31.88s
```

### Gate: compile-controls-default

Command: `cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture`. Exit 0. Full raw output: `logs/compile-controls-default.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.91s
```

### Gate: compile-controls-no-default

Command: `cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture`. Exit 0. Full raw output: `logs/compile-controls-no-default.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.74s
```

### Gate: ownership-discovery

Command: `cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`. Exit 101. Full raw output: `logs/ownership-discovery.log`.

```text
test result: FAILED. 30 passed; 1 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 70.10s
```

## Ownership discovery replay

46 unchanged constructs move coordinates; eight added constructs remain UnknownConstruct (five new closures, one matches expansion, two new frame derives). No host payload, disposition, mark or coverage changes. The pure-lookup fallback closure moves from status.rs:168:75 to :337:67 with identical body. Changed partitions: crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv, crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv, crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv. Every row follows below.

```text
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	103	51	closure captures: |session| (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	105	51	closure captures: |session| (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	107	51	closure captures: |session| (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	109	51	closure captures: |session| (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	19	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	21	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	84	51	closure captures: |session| (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	86	51	closure captures: |session| (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	95	51	closure captures: |session| (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs	97	51	closure captures: |session| (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	119	41	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	95	41	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	124	92	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	100	92	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	129	42	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	105	42	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	130	49	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	106	49	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	136	45	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	112	45	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	140	38	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	116	38	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	141	45	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	117	45	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	146	45	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	122	45	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	150	38	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	126	38	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	151	45	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	127	45	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	155	91	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	131	91	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	161	23	unexpanded macro matches! (ends at 161:97)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	137	23	unexpanded macro matches! (ends at 137:97)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	163	37	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	139	37	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	188	64	unexpanded macro format! (ends at 190:14)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	159	64	unexpanded macro format! (ends at 161:14)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	204	17	unexpanded macro crate :: resume_or_suspend! (ends at 204:104)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	175	17	unexpanded macro crate :: resume_or_suspend! (ends at 175:104)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	229	31	unexpanded macro crate :: resume_or_suspend! (ends at 238:14)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	200	31	unexpanded macro crate :: resume_or_suspend! (ends at 209:14)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	272	9	unexpanded macro crate :: resume_or_suspend! (ends at 277:10)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	243	9	unexpanded macro crate :: resume_or_suspend! (ends at 248:10)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	282	5	unexpanded macro matches! (ends at 282:107)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	348	5	unexpanded macro matches! (ends at 348:107)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	286	5	unexpanded macro matches! (ends at 286:107)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	352	5	unexpanded macro matches! (ends at 352:107)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	290	5	unexpanded macro matches! (ends at 290:86)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	356	5	unexpanded macro matches! (ends at 356:86)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	309	69	unexpanded macro vec! (ends at 309:80)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	375	69	unexpanded macro vec! (ends at 375:80)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	344	14	closure captures: |(key , _)| (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	410	14	closure captures: |(key , _)| (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	353	14	closure captures: |(key , _)| (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	419	14	closure captures: |(key , _)| (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	385	15	closure captures: |entry| (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	451	15	closure captures: |entry| (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	394	42	unexpanded macro vec! (ends at 394:56)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	460	42	unexpanded macro vec! (ends at 460:56)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	403	8	unexpanded macro matches! (ends at 403:55)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	469	8	unexpanded macro matches! (ends at 469:55)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	406	13	unexpanded macro vec! (ends at 406:48)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	472	13	unexpanded macro vec! (ends at 472:48)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	414	5	unexpanded macro matches! (ends at 414:52)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	480	5	unexpanded macro matches! (ends at 480:52)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	429	38	unexpanded macro vec! (ends at 429:76)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	495	38	unexpanded macro vec! (ends at 495:76)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	444	1	unresolved attribute/expansion derive (Clone , Debug , Default , PartialEq , Eq)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	510	1	unresolved attribute/expansion derive (Clone , Debug , Default , PartialEq , Eq)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	471	25	closure captures: |(key , value)| (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	537	25	closure captures: |(key , value)| (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	487	9	unexpanded macro vec! (ends at 494:10)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	553	9	unexpanded macro vec! (ends at 560:10)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	520	12	unexpanded macro matches! (ends at 520:72)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	586	12	unexpanded macro matches! (ends at 586:72)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	528	57	unexpanded macro matches! (ends at 528:99)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	594	57	unexpanded macro matches! (ends at 594:99)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	110	17	unexpanded macro format! (ends at 113:18)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	113	17	unexpanded macro format! (ends at 116:18)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	141	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	144	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	150	24	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	153	24	closure captures: || (capture types/modes unresolved)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	18	29	unexpanded macro format! (ends at 22:30)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	21	29	unexpanded macro format! (ends at 25:30)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	47	17	unexpanded macro format! (ends at 50:18)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	50	17	unexpanded macro format! (ends at 53:18)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	65	23	unexpanded macro crate :: resume_or_suspend! (ends at 70:10)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable.rs	68	23	unexpanded macro crate :: resume_or_suspend! (ends at 73:10)
MOVE UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	168	75	closure captures: || (capture types/modes unresolved)
  -> UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	337	67	closure captures: || (capture types/modes unresolved)
ADD UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	306	30	closure captures: || (capture types/modes unresolved)
ADD UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	314	30	closure captures: || (capture types/modes unresolved)
ADD UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	316	30	closure captures: || (capture types/modes unresolved)
ADD UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	322	15	unexpanded macro matches! (ends at 322:89)
ADD UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	327	60	closure captures: || (capture types/modes unresolved)
ADD UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs	82	33	closure captures: || (capture types/modes unresolved)
ADD UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs	70	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
ADD UnknownConstruct	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs	93	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

### Gate: fmt-final

Command: `cargo fmt --check`. Exit 0. Full raw output: `logs/fmt-final.log`.

```text
No diagnostics.
```

### Gate: runtime-typed

Command: `cargo test -p rum-runtime --lib suspension::typed::tests`. Exit 0. Full raw output: `logs/runtime-typed.log`.

```text
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.54s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

### Gate: runtime-root-trace

Command: `cargo test -p rum-runtime --lib frame_root_trace`. Exit 0. Full raw output: `logs/runtime-root-trace.log`.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 1.38s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

### Gate: runtime-resume-context

Command: `cargo test -p rum-runtime --lib resume_evaluation_context`. Exit 0. Full raw output: `logs/runtime-resume-context.log`.

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.16s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

### Gate: runtime-bounded-recursion

Command: `cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact`. Exit 0. Full raw output: `logs/runtime-bounded-recursion.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

## Design as built

The free `classify_status_object_call` takes borrowed head and arguments and returns a pure projected value or a borrowed virtual-message plan. It has no Session and no Result; the negative compiler control verifies that evaluation through `self` is unavailable. Non-status eligibility tests preserve the old order.

The Function branch immediately returns `apply_status_function`. Projection and value evaluation each capture a witness immediately from their actual Result, consume that Result once, and send only the witness and InputRequired request into a named cold helper. `StatusFunctionProjectionFrame` and `StatusFunctionValueFrame` are unit frames with Input=Expr and Output=Option<Expr>. Projection resume completes value evaluation and the Function boundary; a second pause installs the distinct value frame through its own witness and returns through ResumeStop/complete_work. Value resume applies the same finish function and wraps Some. Return in projection propagates; Return in projected-value evaluation is absorbed. No token clone or token Arc is used.

The old F0344/F0345/F0346 implementations, non-status F0528 tail, codecs, modes, registry and driver remain byte-identical. Existing string-handler and Confirm consumers continue through their Expr projection helper. Only the pure lookup tail was extracted from that helper. Evaluate/finish profiling is retained without a timer crossing a pause.

Two new frame rows and two witnessed sites advance the receipt to G15-A. Existing slot shapes suffice, so `typed-slot-shapes.tsv` is unchanged. Source census reports 608 production implementations (606 -> 608), 97 typed (95 -> 97), 55 typed macros unchanged and 48 witnessed sites (46 -> 48). Global modes are unchanged. The closed frame-root inventories add precisely the two new types; the named count is 563 -> 565. Compiler controls advance 35 -> 36. All other pin changes are derived from those two additions.

The three dead-code warnings in runtime-only tests (`register_test_if_builtin`, `register_test_return_builtin`, `register_test_vm_template_builtins`) also occur in the preserved base-source warning log from brief-617. The defining source is unchanged. No new runtime warning is introduced.

### Gate: census-default

Command: `cargo test -p rum-stdlib --test typed_slot_census -- --nocapture`. Exit 0. Full raw output: `logs/census-default.log`.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.50s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.49s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 276.95s
```

### Gate: census-no-default

Command: `cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture`. Exit 0. Full raw output: `logs/census-no-default.log`.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 20.72s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 20.62s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 291.31s
```

## Full census results

Both feature builds pass 8 tests with the explicit forced-red entry ignored. The two nested `test result: FAILED` lines per build are deliberately broken scratch-copy probes whose parent tests pass after proving the intended rejection. The unchanged-source receipt below is verbatim from the discovery run (both full builds produce the same receipt).

```text
G15-A CENSUS production frames=461 finalizers=50 implementations=608 typed=97 macros=542 result=343 insert=199
G15-A CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
G15-A CENSUS test-support frames=7 finalizers=1 implementations=21 typed=13 macros=10 result=6 insert=4
G15-A PARTITION implementations=624 macros=553
G15-A FRAME FG15AProjection StatusFunctionProjectionFrame Expr -> Option<Expr>
G15-A FRAME FG15AValue StatusFunctionValueFrame Expr -> Option<Expr>
G15-A INSERTION MG15AProjection rum_runtime::runtime::scoping::callable::status_application::<Session as inherent>::apply_status_function#method2:suspend_status_function_projection:self
G15-A INSERTION MG15AValue rum_runtime::runtime::scoping::callable::status_application::<Session as inherent>::evaluate_status_function_value#method2:suspend_status_function_value:self
G15-A TOTAL implementations=608 typed=97 typed_macros=55 witnessed_sites=48; two new Expr -> Option<Expr> frames; global modes unchanged
G15-A REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
```

Citation audit: the only TSVs referencing the moved status/callable source paths are the three replayed ownership partitions and the typed-stage identity table. No `obligations.tsv` citation names a moved construct, so its bytes remain unchanged.

### Gate: ordinary-default

Command: `cargo test --manifest-path /home/remi/dev/rum/.temp/brief-619/ordinary-controls/Cargo.toml --offline --test ordinary -- --nocapture`. Exit 0. Full raw output: `logs/ordinary-default.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
continuation-slot fault-occurrence delta=0; observations=336
```

### Gate: ordinary-no-default

Command: `cargo test --manifest-path /home/remi/dev/rum/.temp/brief-619/ordinary-controls/Cargo.toml --offline --no-default-features --test ordinary -- --nocapture`. Exit 0. Full raw output: `logs/ordinary-no-default.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
continuation-slot fault-occurrence delta=0; observations=336
```

## Ordinary controls

All 28 round-4 controls pass through the copied brief-611 harness in each feature build under Checks::ALL, with unconditional FaultProbe absence. Fault delta is zero in both builds (336 observations each). Exact 28 rows per build: `ordinary-default.tsv` and `ordinary-no-default.tsv`. Only the copied harness output destination was retargeted; oracle support was synchronized from current source.

### Gate: regenerate-fingerprints

Command: `cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`. Exit 0. Full raw output: `logs/regenerate-fingerprints.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 0.18s
```

## Canonical fingerprints replay

Destination: `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`. Canonical output paths: `target/brief-544/local-contracts.tsv`. Non-derived fields are identical. No handwritten hashes, marks or ledger cells.

```text
{"id": "B01", "old": "fnv1a64:ea801ab7202400eb", "new": "fnv1a64:a896eef2bf55f629"}
{"id": "B02", "old": "fnv1a64:92a1bb7984c11336", "new": "fnv1a64:7b5c0216bd31de58"}
{"id": "B03", "old": "fnv1a64:aa9a82ba75a6daa5", "new": "fnv1a64:153020959e70d02f"}
{"id": "B04", "old": "fnv1a64:8c8d8f58096f53d3", "new": "fnv1a64:1b3d50b3668e4931"}
{"id": "B05", "old": "fnv1a64:3c9f4f150ad332ea", "new": "fnv1a64:0a9dff28040ddac4"}
{"id": "B06", "old": "fnv1a64:9057435b31f931d7", "new": "fnv1a64:8c2dfde8d114b9cd"}
{"id": "B07", "old": "fnv1a64:12c0e4e1c8daa8da", "new": "fnv1a64:58e35ae00d6014d4"}
{"id": "B08", "old": "fnv1a64:053e15de7dfb32d9", "new": "fnv1a64:2554f91e8908052b"}
{"id": "B09", "old": "fnv1a64:3a148af20c728f7f", "new": "fnv1a64:e5a3e1f006ab33b5"}
{"id": "B10", "old": "fnv1a64:426aabeff05f8408", "new": "fnv1a64:c73b70077d6eb45e"}
{"id": "B11", "old": "fnv1a64:4de540b5236379d3", "new": "fnv1a64:dc95021080826f31"}
{"id": "B12", "old": "fnv1a64:f9210aede87b247f", "new": "fnv1a64:a4b061ebe2b3c8b5"}
{"id": "I01", "old": "fnv1a64:c86205b48cdbf76b", "new": "fnv1a64:8678d9f02c0deca9"}
{"id": "I02", "old": "fnv1a64:93e0cca90309baea", "new": "fnv1a64:61df7cbbfc4462c4"}
{"id": "I03", "old": "fnv1a64:0c97e4242ce87abf", "new": "fnv1a64:9ebfa3b26fd63775"}
{"id": "I04", "old": "fnv1a64:0269bfd724e771d6", "new": "fnv1a64:9a7e9d64ec2f9af8"}
{"id": "P01", "old": "fnv1a64:173b66e4096c45f7", "new": "fnv1a64:6f21f43fe79d33ed"}
{"id": "P02", "old": "fnv1a64:4cd61222699a9e58", "new": "fnv1a64:99a7abc9b25ae8ce"}
{"id": "P03", "old": "fnv1a64:7f8e0245924e6abd", "new": "fnv1a64:446bf964c81cbe47"}
{"id": "P04", "old": "fnv1a64:877cd98f096a2f36", "new": "fnv1a64:7037202c41dafa58"}
{"id": "P05", "old": "fnv1a64:ea6ac860eec2247b", "new": "fnv1a64:22b9469706e7fa19"}
{"id": "P06", "old": "fnv1a64:49a02fbe74aabdff", "new": "fnv1a64:f52f86bc6ee36235"}
{"id": "P070", "old": "fnv1a64:83edcc00532037ff", "new": "fnv1a64:2f7d22fe4d58dc35"}
{"id": "P080", "old": "fnv1a64:6139d82bcd5bd448", "new": "fnv1a64:d3f368650a010f1e"}
{"id": "P090", "old": "fnv1a64:8e4eea09cf5a1176", "new": "fnv1a64:5ba843aeb62e6618"}
{"id": "P100", "old": "fnv1a64:4f6875fc48a55fc3", "new": "fnv1a64:52a9ed68d4840a41"}
{"id": "P110", "old": "fnv1a64:572ea9f33a34c2e0", "new": "fnv1a64:b066123e3a12f6d6"}
{"id": "P120", "old": "fnv1a64:803ff633305b5e79", "new": "fnv1a64:b15e176ac408754b"}
{"id": "P130", "old": "fnv1a64:b68e3e3ef78e82da", "new": "fnv1a64:fcb0b43d3c13eed4"}
{"id": "P140", "old": "fnv1a64:a66d4e8cd98f1dce", "new": "fnv1a64:dff9b1551f6ad1d0"}
{"id": "P150", "old": "fnv1a64:d8e17a9df7341d16", "new": "fnv1a64:422cdeb1aaf8c5b8"}
{"id": "P071", "old": "fnv1a64:7fa047bd14b6927a", "new": "fnv1a64:767417c48d5accf4"}
{"id": "P081", "old": "fnv1a64:edda47d83c6cbc9f", "new": "fnv1a64:39feebb0d86f1a55"}
{"id": "P091", "old": "fnv1a64:7f0b071d3d5ae537", "new": "fnv1a64:87dbd908bf6d95ad"}
{"id": "P101", "old": "fnv1a64:4f319ac06f985fe0", "new": "fnv1a64:a869030b6f7693d6"}
{"id": "P111", "old": "fnv1a64:4e10e31e7020f6ef", "new": "fnv1a64:08eee8b832a7f385"}
{"id": "P121", "old": "fnv1a64:575b0428d04351e4", "new": "fnv1a64:999ed01c9eb9e002"}
{"id": "P131", "old": "fnv1a64:2a37b1be3fc37553", "new": "fnv1a64:b8e773199ce26ab1"}
{"id": "P141", "old": "fnv1a64:d3faff396fcf4e17", "new": "fnv1a64:b54ba8255ccf4c8d"}
{"id": "P151", "old": "fnv1a64:ec0649af345fef39", "new": "fnv1a64:0796fd23d9e3698b"}
{"id": "B13", "old": "fnv1a64:1e67d6e46295edd7", "new": "fnv1a64:1a3e917201b175cd"}
{"id": "R01", "old": "fnv1a64:7550178456deac26", "new": "fnv1a64:2d7cc7fd00238568"}
{"id": "R02", "old": "fnv1a64:90f2b68b895988fd", "new": "fnv1a64:ce4f46625cba7807"}
{"id": "R03", "old": "fnv1a64:26a6d149731ebf62", "new": "fnv1a64:43d975f55706879c"}
{"id": "R040", "old": "fnv1a64:1a0bed7c59746f46", "new": "fnv1a64:ed40091b299c9188"}
{"id": "R050", "old": "fnv1a64:2c8b613f86c0625b", "new": "fnv1a64:cff26c1f698d5279"}
{"id": "R060", "old": "fnv1a64:43bf21ed4e9bcdbe", "new": "fnv1a64:e097ae7315c57b60"}
{"id": "R070", "old": "fnv1a64:56480ff21e0e7b9f", "new": "fnv1a64:a26cb3caba10d955"}
{"id": "R080", "old": "fnv1a64:c96e28c9f46c3ad6", "new": "fnv1a64:61830657bbb463f8"}
{"id": "R090", "old": "fnv1a64:c280440bd73e3503", "new": "fnv1a64:369d9991d683e501"}
{"id": "R100", "old": "fnv1a64:6fc0270d1f7c8525", "new": "fnv1a64:da55c4e848467aaf"}
{"id": "R110", "old": "fnv1a64:36708af5e82f0ad0", "new": "fnv1a64:695b1a32193a5926"}
{"id": "R041", "old": "fnv1a64:04a13a0e79bdacd7", "new": "fnv1a64:0077f49c18d934cd"}
{"id": "R051", "old": "fnv1a64:1c71181f5c881df6", "new": "fnv1a64:e9ca71c4435c7298"}
{"id": "R061", "old": "fnv1a64:6507aed9bcf53329", "new": "fnv1a64:7b2ac53b759eea1b"}
{"id": "R071", "old": "fnv1a64:b19012f6d513bbec", "new": "fnv1a64:0c96a1c3002d762a"}
{"id": "R081", "old": "fnv1a64:0b52df50e45134c3", "new": "fnv1a64:0e9456bd702fdf41"}
{"id": "R091", "old": "fnv1a64:42a7c4de0567988e", "new": "fnv1a64:60d33aadf9a6d610"}
{"id": "R101", "old": "fnv1a64:20a2bd8c1e79b3fe", "new": "fnv1a64:a639403984a53320"}
{"id": "R111", "old": "fnv1a64:3c86ba9ab1554e61", "new": "fnv1a64:a234fb99e5db4433"}
{"id": "R12", "old": "fnv1a64:fc8d6994d827f821", "new": "fnv1a64:e1df8459fed23073"}
{"id": "R13", "old": "fnv1a64:7751c3f7fd3befb5", "new": "fnv1a64:ab15bfa754e90c1f"}
{"id": "R14", "old": "fnv1a64:69c0847225561fc3", "new": "fnv1a64:6d01fbdeb134ca41"}
{"id": "R15", "old": "fnv1a64:0b91fb5df46f4867", "new": "fnv1a64:e38b830431815f3d"}
{"id": "O010", "old": "fnv1a64:bfbf496f73520909", "new": "fnv1a64:ac4663e2c282727b"}
{"id": "O0200", "old": "fnv1a64:a134f061284643ce", "new": "fnv1a64:dac153296e21f7d0"}
{"id": "O011", "old": "fnv1a64:8647de755c2d923e", "new": "fnv1a64:23206afb23573fe0"}
{"id": "O0210", "old": "fnv1a64:1be8105807474681", "new": "fnv1a64:0dd1fa2b0cfd13d3"}
```

### Gate: certificates-discovery-default

Command: `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`. Exit 101. Full raw output: `logs/certificates-discovery-default.log`.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 19.10s
```

### Gate: certificates-discovery-no-default

Command: `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`. Exit 101. Full raw output: `logs/certificates-discovery-no-default.log`.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.64s
```

## Canonical certificates replay

Destination: `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv`. Canonical output paths: `.temp/brief-546/certificates/promoted-hash-true.tsv`, `.temp/brief-546/certificates/promoted-hash-false.tsv`. Non-derived fields are identical. No handwritten hashes, marks or ledger cells.

```text
{"id": "C01", "feature_hash": "true", "old": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:315fcd2a01e081aa\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:c347b2502f1a2451;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:1be8105807474681;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:8647de755c2d923e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:315fcd2a01e081aa\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:b1125c8ddefcf0e3;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:a134f061284643ce;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:bfbf496f73520909;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}", "new": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:03265d3e8582f2b4\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:6353433c6e9a106e;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:0dd1fa2b0cfd13d3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:23206afb23573fe0;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:03265d3e8582f2b4\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:2a43cb5cd0908a60;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:dac153296e21f7d0;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:ac4663e2c282727b;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"}
{"id": "C02", "feature_hash": "true", "old": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:0b7db79ba4c06884\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:ea2415ec022bd54f;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:c86205b48cdbf76b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:93e0cca90309baea;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:0269bfd724e771d6;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:7550178456deac26;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:90f2b68b895988fd;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:1e67d6e46295edd7;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:3c9f4f150ad332ea;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}", "new": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:d3ac466610f104ce\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:38cb1e49ab5b302d;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:8678d9f02c0deca9;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:61df7cbbfc4462c4;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:9a7e9d64ec2f9af8;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:2d7cc7fd00238568;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:ce4f46625cba7807;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:1a3e917201b175cd;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:0a9dff28040ddac4;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"}
{"id": "C01", "feature_hash": "false", "old": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:4f2a48af4ffede3c\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:cda40c27bea79d83;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:1be8105807474681;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:8647de755c2d923e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:4f2a48af4ffede3c\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:56032721dfa38a69;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:a134f061284643ce;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:bfbf496f73520909;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}", "new": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:946becd717009e33\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:8fe9740ecd272e39;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:0dd1fa2b0cfd13d3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:23206afb23573fe0;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:946becd717009e33\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:272448549ff4c987;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:dac153296e21f7d0;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:ac4663e2c282727b;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"}
{"id": "C02", "feature_hash": "false", "old": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:618d147221ecf666\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:4c976314b42ea4c4;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:c86205b48cdbf76b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:93e0cca90309baea;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:0269bfd724e771d6;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:7550178456deac26;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:90f2b68b895988fd;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:1e67d6e46295edd7;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:3c9f4f150ad332ea;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}", "new": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None\", body_fingerprint: \"fnv1a64:1cff771c03708dc1\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:48405f1a31e15c20;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:8678d9f02c0deca9;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:61df7cbbfc4462c4;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:9a7e9d64ec2f9af8;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:2d7cc7fd00238568;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:ce4f46625cba7807;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:1a3e917201b175cd;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:0a9dff28040ddac4;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}"}
```

### Gate: ownership-final

Command: `cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`. Exit 0. Full raw output: `logs/ownership-final.log`.

```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 73.41s
```

### Gate: acceptance-default

Command: `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`. Exit 0. Full raw output: `logs/acceptance-default.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.79s
```

### Gate: acceptance-no-default

Command: `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`. Exit 0. Full raw output: `logs/acceptance-no-default.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.90s
```

### Gate: rejection-census

Command: `cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture`. Exit 0. Full raw output: `logs/rejection-census.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 26.77s
```

### Gate: base-rejection-census

Command: `cargo test --offline --manifest-path .temp/brief-619/base-source/Cargo.toml -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture`. Exit 0. Full raw output: `logs/base-rejection-census.log`.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 26.26s
```

## Rejection census row replay

The unchanged exported base was rebuilt in its own target and reproduces the preserved base `deduplicated.tsv` byte-for-byte. Production definitions: 7385 -> 7391 (one deleted method, seven added functions). Rejections: 48696 -> 48713. All 48693 paired rows retain kind, target, reason and class: 184 are pure coordinate/name moves; two moved loop/push boundaries replace their projection producer but retain their exact rejection meaning. Three Try operations were deleted and 20 new boundaries remain rejected. The historical option frontier stays 950 (10/913/27), public wrapper 16, and all local marks unchanged. No source pin needed changing. Full comparison: `rejection-replay.json`; canonical candidate tables: `candidate-retention/`; fresh base: `base-source/target/brief-535/EDGE_ALL/`.

Every changed row is disclosed below.

```text
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_key crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:433:20-433:48 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_key crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:499:20-499:48; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_key crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:437:46-437:57 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_key crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:503:46-503:57; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_key crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:438:46-438:56 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_key crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:504:46-504:56; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_key crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:438:46-438:68 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_key crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:504:46-504:68; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameter_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:451:17-451:95 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameter_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:517:17-517:95; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:458:25-458:39 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:524:25-524:39; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET001 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:459:20-459:30 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:525:20-525:30; not a resolved function definition: std::vec::Vec::new
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:464:16-464:59 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:530:16-530:59; path FailureMessageTemplateParameters::default has 0 exact bindings
PAIR RET001 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:468:21-468:31 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:534:21-534:31; not a resolved function definition: std::vec::Vec::new
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:469:16-470:25 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:535:16-536:25; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:469:16-471:85 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:535:16-537:85; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:469:16-472:23 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:535:16-538:23; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:471:25-471:84 -> rum_runtime::runtime::kernel_data::control_data::status::failure_message_template_parameters_from_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:537:25-537:84; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:350:24-350:52 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:416:24-416:52; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:351:26-351:65 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:417:26-417:65; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:351:26-352:21 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:417:26-418:21; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:351:26-353:29 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:417:26-419:29; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:351:26-354:29 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:417:26-420:29; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:353:14-353:28 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:419:14-419:28; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:355:8-355:69 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:421:8-421:69; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:355:8-356:64 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:421:8-422:64; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:356:12-356:64 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:422:12-422:64; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:358:9-358:61 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:424:9-424:61; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:359:9-359:53 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:425:9-425:53; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:361:5-361:45 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:427:5-427:45; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:362:5-362:51 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:428:5-428:51; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:363:5-363:47 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:429:5-429:47; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:364:5-364:23 -> rum_runtime::runtime::kernel_data::control_data::status::failure_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:430:5-430:23; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:519:5-519:53 -> rum_runtime::runtime::kernel_data::control_data::status::failure_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:585:5-585:53; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:519:5-520:72 -> rum_runtime::runtime::kernel_data::control_data::status::failure_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:585:5-586:72; primitive operand proof or exact operator/conversion impl required
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::failure_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:520:12-520:72 -> rum_runtime::runtime::kernel_data::control_data::status::failure_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:586:12-586:72; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_held_message_template_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:389:20-389:91 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_held_message_template_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:455:20-455:91; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_held_message_template_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:390:8-390:61 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_held_message_template_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:456:8-456:61; primitive operand proof or exact operator/conversion impl required
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_held_message_template_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:394:42-394:56 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_held_message_template_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:460:42-460:56; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_message_name_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:398:20-398:91 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_message_name_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:464:20-464:91; implicit Try/FromResidual/error ownership not modeled
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_message_name_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:403:8-403:55 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_message_name_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:469:8-469:55; unexpanded macro; contents are not resolved Rust
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_message_name_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:406:13-406:48 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_message_name_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:472:13-472:48; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:419:26-419:54 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:485:26-485:54; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:420:5-420:44 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:486:5-486:44; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:422:60-422:71 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:488:60-488:71; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:423:47-423:84 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:489:47-489:84; not a resolved function definition: rum_core::expr::Expr::Atom
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:423:58-423:83 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:489:58-489:83; not a resolved function definition: rum_core::expr::Atom::Symbol
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:423:71-423:82 -> rum_runtime::runtime::kernel_data::control_data::status::failure_virtual_tag_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:489:71-489:82; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::is_failure_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:282:5-282:107 -> rum_runtime::runtime::kernel_data::control_data::status::is_failure_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:348:5-348:107; unexpanded macro; contents are not resolved Rust
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::is_failure_message_template_virtualizable crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:414:5-414:52 -> rum_runtime::runtime::kernel_data::control_data::status::is_failure_message_template_virtualizable crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:480:5-480:52; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::is_failure_message_template_virtualizable crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:414:5-415:58 -> rum_runtime::runtime::kernel_data::control_data::status::is_failure_message_template_virtualizable crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:480:5-481:58; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::is_failure_message_template_virtualizable crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:415:12-415:58 -> rum_runtime::runtime::kernel_data::control_data::status::is_failure_message_template_virtualizable crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:481:12-481:58; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::is_failure_q_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:290:5-290:86 -> rum_runtime::runtime::kernel_data::control_data::status::is_failure_q_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:356:5-356:86; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::is_failure_q_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:290:5-291:50 -> rum_runtime::runtime::kernel_data::control_data::status::is_failure_q_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:356:5-357:50; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::is_failure_q_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:290:5-292:51 -> rum_runtime::runtime::kernel_data::control_data::status::is_failure_q_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:356:5-358:51; primitive operand proof or exact operator/conversion impl required
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::is_success_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:286:5-286:107 -> rum_runtime::runtime::kernel_data::control_data::status::is_success_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:352:5-352:107; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::message_parameter_name crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:478:48-478:60 -> rum_runtime::runtime::kernel_data::control_data::status::message_parameter_name crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:544:48-544:60; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::message_parameter_name crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:479:50-479:75 -> rum_runtime::runtime::kernel_data::control_data::status::message_parameter_name crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:545:50-545:75; method syntax: method receiver lacks an exact nominal type
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:304:8-304:53 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:370:8-370:53; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:305:21-305:33 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:371:21-371:33; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:309:69-309:80 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:375:69-375:80; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:316:8-316:60 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:382:8-382:60; primitive operand proof or exact operator/conversion impl required
PAIR RET001 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:320:23-320:53 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:386:23-386:53; not a resolved function definition: std::vec::Vec::with_capacity
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:320:42-320:52 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:386:42-386:52; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:321:5-323:6 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:387:5-389:6; implicit IntoIterator/next/iterator Drop not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:322:9-322:56 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:388:9-388:56; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:322:22-322:55 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_failure_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:388:22-388:55; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_success_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:296:8-296:53 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_success_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:362:8-362:53; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::normalize_success_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:297:21-297:33 -> rum_runtime::runtime::kernel_data::control_data::status::normalize_success_details_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:363:21-363:33; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:333:8-333:44 -> rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:399:8-399:44; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:333:8-333:63 -> rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:399:8-399:63; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:333:48-333:58 -> rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:399:48-399:58; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:333:48-333:63 -> rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:399:48-399:63; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:337:11-337:18 -> rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:403:11-403:18; implicit Index/Deref/bounds exceptional effects not proved
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:337:11-337:26 -> rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:403:11-403:26; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:337:28-337:35 -> rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:403:28-403:35; implicit Index/Deref/bounds exceptional effects not proved
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:337:28-337:43 -> rum_runtime::runtime::kernel_data::control_data::status::parse_immediate_rule_entry crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:403:28-403:43; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_all_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:369:24-369:52 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_all_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:435:24-435:52; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_all_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:370:10-370:25 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_all_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:436:10-436:25; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::status_object_missing_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:487:9-494:10 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_missing_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:553:9-560:10; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_parts crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:503:8-503:91 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_parts crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:569:8-569:91; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_parts crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:503:10-503:90 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_parts crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:569:10-569:90; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_parts crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:507:26-507:41 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_parts crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:573:26-573:41; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:341:24-341:52 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:407:24-407:52; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:342:22-342:61 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:408:22-408:61; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:342:22-343:21 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:408:22-409:21; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:342:22-344:29 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:408:22-410:29; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:342:22-345:29 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:408:22-411:29; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:344:14-344:28 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_properties_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:410:14-410:28; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:382:24-382:52 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:448:24-448:52; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:383:5-383:48 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:449:5-449:48; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:383:5-384:21 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:449:5-450:21; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:383:5-385:46 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:449:5-451:46; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:385:15-385:45 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_property_entry_form crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:451:15-451:45; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::status_object_property_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:374:24-374:52 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_property_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:440:24-440:52; implicit Try/FromResidual/error ownership not modeled
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::styled_message_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:429:38-429:76 -> rum_runtime::runtime::kernel_data::control_data::status::styled_message_expr crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:495:38-495:76; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::success_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:528:5-528:53 -> rum_runtime::runtime::kernel_data::control_data::status::success_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:594:5-594:53; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::kernel_data::control_data::status::success_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:528:5-528:99 -> rum_runtime::runtime::kernel_data::control_data::status::success_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:594:5-594:99; primitive operand proof or exact operator/conversion impl required
PAIR RET003 rum_runtime::runtime::kernel_data::control_data::status::success_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:528:57-528:99 -> rum_runtime::runtime::kernel_data::control_data::status::success_tag_supports_projection crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:594:57-594:99; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:84:17-84:83 -> rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:86:17-86:83; method syntax: method instance has unknown bounds or argument arity
PAIR RET002 rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:84:51-84:82 -> rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:86:51-86:82; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:95:17-95:88 -> rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:97:17-97:88; method syntax: method instance has unknown bounds or argument arity
PAIR RET002 rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:95:51-95:87 -> rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:97:51-97:87; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:103:17-103:84 -> rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:105:17-105:84; method syntax: method instance has unknown bounds or argument arity
PAIR RET002 rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:103:51-103:83 -> rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:105:51-105:83; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:107:17-107:86 -> rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:109:17-109:86; method syntax: method instance has unknown bounds or argument arity
PAIR RET002 rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:107:51-107:85 -> rum_runtime::runtime::model::session::Session::eval_control_data_builtin crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:109:51-109:85; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:218:20-224:27 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:189:20-195:27; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:220:21-220:35 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:191:21-191:35; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:222:21-222:64 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:193:21-193:64; path FailureMessageTemplateParameters::default has 0 exact bindings
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:227:26-227:40 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:198:26-198:40; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET003 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:229:31-238:14 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:200:31-209:14; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:229:31-238:15 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:200:31-209:15; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:240:20-242:27 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:211:20-213:27; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:246:9-247:23 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:217:9-218:23; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:246:51-246:65 -> rum_runtime::runtime::model::session::Session::failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:217:51-217:65; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::failure_virtual_message_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:180:32-180:75 -> rum_runtime::runtime::model::session::Session::failure_virtual_message_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:151:32-151:75; implicit Try/FromResidual/error ownership not modeled
PAIR RET003 rum_runtime::runtime::model::session::Session::failure_virtual_message_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:188:64-190:14 -> rum_runtime::runtime::model::session::Session::failure_virtual_message_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:159:64-161:14; unexpanded macro; contents are not resolved Rust
PAIR RET003 rum_runtime::runtime::model::session::Session::failure_virtual_styled_message_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:204:17-204:104 -> rum_runtime::runtime::model::session::Session::failure_virtual_styled_message_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:175:17-175:104; unexpanded macro; contents are not resolved Rust
PAIR RET003 rum_runtime::runtime::model::session::Session::finish_failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:272:9-277:10 -> rum_runtime::runtime::model::session::Session::finish_failure_message_from_template crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:243:9-248:10; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:101:12-101:51 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:104:12-104:51; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:101:12-101:74 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:104:12-104:74; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:101:55-101:69 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:104:55-104:69; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:101:55-101:74 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:104:55-104:74; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:105:31-105:43 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:108:31-108:43; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:105:45-105:58 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:108:45-108:58; method syntax: method receiver lacks an exact nominal type
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:107:13-114:14 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:110:13-117:14; method syntax: type is outside the admitted surface: impl Into < String >
PAIR RET003 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:110:17-113:18 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:113:17-116:18; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:118:12-118:57 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:121:12-121:57; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:119:13-119:81 -> rum_runtime::runtime::model::session::Session::invalid_callable_key_application crates/rum-runtime/src/runtime/scoping/callable.rs:122:13-122:81; method syntax: type is outside the admitted surface: impl Into < String >
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:115:12-115:46 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:91:12-91:46; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:118:31-119:97 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:94:31-95:97; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:119:41-119:96 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:95:41-95:96; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:123:34-123:47 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:99:34-99:47; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:124:44-126:27 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:100:44-102:27; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:124:92-126:26 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:100:92-102:26; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:128:32-129:92 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:104:32-105:92; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:128:32-132:35 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:104:32-108:35; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:129:42-129:91 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:105:42-105:91; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:130:49-132:34 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:106:49-108:34; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:134:41-135:68 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:110:41-111:68; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:134:41-138:31 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:110:41-114:31; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:136:45-138:30 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:112:45-114:30; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:139:45-140:88 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:115:45-116:88; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:139:45-143:31 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:115:45-119:31; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:140:38-140:87 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:116:38-116:87; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:141:45-143:30 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:117:45-119:30; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:144:47-145:75 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:120:47-121:75; implicit Try/FromResidual/error ownership not modeled
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:144:47-148:31 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:120:47-124:31; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:146:45-148:30 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:122:45-124:30; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:149:37-150:75 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:125:37-126:75; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:149:37-153:31 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:125:37-129:31; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:150:38-150:74 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:126:38-126:74; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:151:45-153:30 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:127:45-129:30; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:155:29-157:31 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:131:29-133:31; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:155:91-157:30 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:131:91-133:30; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET003 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:161:23-161:97 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:137:23-137:97; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:162:27-163:93 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:138:27-139:93; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:163:37-163:92 -> rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:139:37-139:92; closure capture/callback effects unresolved; closure body is a rejected boundary
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:14:24-14:34 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:17:24-17:34; method syntax: method receiver lacks an exact nominal type
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:14:24-14:49 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:17:24-17:49; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:14:37-14:49 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:17:37-17:49; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:15:25-23:26 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:18:25-26:26; method syntax: type is outside the admitted surface: impl Into < String >
PAIR RET003 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:18:29-22:30 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:21:29-25:30; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:24:51-24:63 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:27:51-27:63; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:24:65-24:78 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:27:65-27:78; method syntax: method receiver lacks an exact nominal type
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:27:36-27:54 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:30:36-30:54; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:27:36-27:80 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:30:36-30:80; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:27:36-27:90 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:30:36-30:90; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:27:59-27:70 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:30:59-30:70; method syntax: method receiver lacks an exact nominal type
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:27:59-27:79 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:30:59-30:79; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:28:67-28:82 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:31:67-31:82; not a resolved function definition: std::collections::BTreeSet::new
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:44:13-51:14 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:47:13-54:14; method syntax: type is outside the admitted surface: impl Into < String >
PAIR RET003 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:47:17-50:18 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:50:17-53:18; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:52:39-52:51 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:55:39-55:51; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:52:53-52:66 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:55:53-55:66; method syntax: method receiver lacks an exact nominal type
PAIR RET003 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:65:23-70:10 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:68:23-73:10; unexpanded macro; contents are not resolved Rust
PAIR RET002 rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:74:24-74:75 -> rum_runtime::runtime::model::session::Session::try_apply_function crates/rum-runtime/src/runtime/scoping/callable.rs:77:24-77:75; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:78:12-78:23 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:265:8-265:19; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:78:12-78:38 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:265:8-265:34; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:78:27-78:38 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:265:23-265:34; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:82:12-82:64 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:268:8-268:60; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:82:26-82:64 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:268:22-268:60; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:85:12-85:64 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:271:8-271:60; primitive operand proof or exact operator/conversion impl required
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:85:26-85:64 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:271:22-271:60; primitive operand proof or exact operator/conversion impl required
PAIR RET001 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:96:34-96:70 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:282:30-282:66; not a resolved function definition: std::vec::Vec::with_capacity
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:96:53-96:69 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:282:49-282:65; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:97:17-99:18 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:283:13-289:14; implicit IntoIterator/next/iterator Drop not modeled
PAIR RET002 rum_runtime::runtime::model::session::Session::apply_status_object_to_args crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:98:21-98:98 -> rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:284:17-288:19; method syntax: method receiver lacks checked nominal/place provenance
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:168:13-175:15 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_pure_property_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:337:5-344:7; method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
PAIR RET002 rum_runtime::runtime::model::session::Session::status_object_projection_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:168:75-175:14 -> rum_runtime::runtime::kernel_data::control_data::status::status_object_pure_property_value crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:337:67-344:6; closure capture/callback effects unresolved; closure body is a rejected boundary
REMOVED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::apply_status_object_to_args@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-107:6", "definition": "rum_runtime::runtime::model::session::Session::apply_status_object_to_args", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "98", "start_column": "33", "end_line": "98", "end_column": "97", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::apply_status_object_to_args@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-107:6: rum_runtime::runtime::model::session::Session::apply_status_object_to_args", "reason": "implicit Try/FromResidual/error ownership not modeled", "class": "implicit Try/FromResidual/error ownership not modeled"}
REMOVED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::apply_status_object_to_args@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-107:6", "definition": "rum_runtime::runtime::model::session::Session::apply_status_object_to_args", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "103", "start_column": "17", "end_line": "103", "end_column": "75", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::apply_status_object_to_args@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-107:6: rum_runtime::runtime::model::session::Session::apply_status_object_to_args", "reason": "implicit Try/FromResidual/error ownership not modeled", "class": "implicit Try/FromResidual/error ownership not modeled"}
REMOVED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::try_apply_function@crates/rum-runtime/src/runtime/scoping/callable.rs:5:5-76:6", "definition": "rum_runtime::runtime::model::session::Session::try_apply_function", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/scoping/callable.rs", "start_line": "33", "start_column": "38", "end_line": "33", "end_column": "83", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::try_apply_function@crates/rum-runtime/src/runtime/scoping/callable.rs:5:5-76:6: rum_runtime::runtime::model::session::Session::try_apply_function", "reason": "implicit Try/FromResidual/error ownership not modeled", "class": "implicit Try/FromResidual/error ownership not modeled"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "290", "start_column": "25", "end_line": "293", "end_column": "15", "target": "rum_runtime::runtime::kernel_data::control_data::status::StatusObjectProjection::Pure", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "not a resolved function definition: rum_runtime::runtime::kernel_data::control_data::status::StatusObjectProjection::Pure", "class": "not a resolved function definition: <target>"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "303", "start_column": "19", "end_line": "303", "end_column": "32", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "method syntax: method receiver lacks checked nominal/place provenance", "class": "method syntax: method receiver lacks checked nominal/place provenance"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "305", "start_column": "42", "end_line": "306", "end_column": "78", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched", "class": "method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "306", "start_column": "30", "end_line": "306", "end_column": "77", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "closure capture/callback effects unresolved; closure body is a rejected boundary", "class": "closure capture/callback effects unresolved; closure body is a rejected boundary"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "310", "start_column": "33", "end_line": "310", "end_column": "56", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "primitive operand proof or exact operator/conversion impl required", "class": "primitive operand proof or exact operator/conversion impl required"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "313", "start_column": "34", "end_line": "314", "end_column": "78", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched", "class": "method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "314", "start_column": "30", "end_line": "314", "end_column": "77", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "closure capture/callback effects unresolved; closure body is a rejected boundary", "class": "closure capture/callback effects unresolved; closure body is a rejected boundary"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "315", "start_column": "26", "end_line": "316", "end_column": "65", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched", "class": "method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "316", "start_column": "30", "end_line": "316", "end_column": "64", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "closure capture/callback effects unresolved; closure body is a rejected boundary", "class": "closure capture/callback effects unresolved; closure body is a rejected boundary"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET003", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "322", "start_column": "15", "end_line": "322", "end_column": "89", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "unexpanded macro; contents are not resolved Rust", "class": "unexpanded macro; contents are not resolved Rust"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "327", "start_column": "10", "end_line": "329", "end_column": "8", "target": "rum_runtime::runtime::kernel_data::control_data::status::StatusObjectProjection::Pure", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "not a resolved function definition: rum_runtime::runtime::kernel_data::control_data::status::StatusObjectProjection::Pure", "class": "not a resolved function definition: <target>"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "327", "start_column": "39", "end_line": "329", "end_column": "7", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "method syntax: method receiver lacks checked nominal/place provenance", "class": "method syntax: method receiver lacks checked nominal/place provenance"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2", "definition": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "327", "start_column": "60", "end_line": "329", "end_column": "6", "target": "-", "witness": "rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:259:1-330:2: rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call", "reason": "closure capture/callback effects unresolved; closure body is a rejected boundary", "class": "closure capture/callback effects unresolved; closure body is a rejected boundary"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::evaluate_status_function_value@crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:31:5-47:6", "definition": "rum_runtime::runtime::model::session::Session::evaluate_status_function_value", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/scoping/callable/status_application.rs", "start_line": "46", "start_column": "9", "end_line": "46", "end_column": "60", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::evaluate_status_function_value@crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:31:5-47:6: rum_runtime::runtime::model::session::Session::evaluate_status_function_value", "reason": "method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched", "class": "method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::project_status_function_message@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-83:6", "definition": "rum_runtime::runtime::model::session::Session::project_status_function_message", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "78", "start_column": "13", "end_line": "78", "end_column": "63", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::project_status_function_message@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-83:6: rum_runtime::runtime::model::session::Session::project_status_function_message", "reason": "implicit Try/FromResidual/error ownership not modeled", "class": "implicit Try/FromResidual/error ownership not modeled"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::project_status_function_message@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-83:6", "definition": "rum_runtime::runtime::model::session::Session::project_status_function_message", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "80", "start_column": "13", "end_line": "80", "end_column": "56", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::project_status_function_message@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-83:6: rum_runtime::runtime::model::session::Session::project_status_function_message", "reason": "implicit Try/FromResidual/error ownership not modeled", "class": "implicit Try/FromResidual/error ownership not modeled"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::project_status_function_message@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-83:6", "definition": "rum_runtime::runtime::model::session::Session::project_status_function_message", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "82", "start_column": "12", "end_line": "82", "end_column": "89", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::project_status_function_message@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-83:6: rum_runtime::runtime::model::session::Session::project_status_function_message", "reason": "method syntax: method receiver lacks checked nominal/place provenance", "class": "method syntax: method receiver lacks checked nominal/place provenance"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::project_status_function_message@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-83:6", "definition": "rum_runtime::runtime::model::session::Session::project_status_function_message", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs", "start_line": "82", "start_column": "33", "end_line": "82", "end_column": "88", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::project_status_function_message@crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71:5-83:6: rum_runtime::runtime::model::session::Session::project_status_function_message", "reason": "closure capture/callback effects unresolved; closure body is a rejected boundary", "class": "closure capture/callback effects unresolved; closure body is a rejected boundary"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::suspend_status_function_projection@crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:49:5-57:6", "definition": "rum_runtime::runtime::model::session::Session::suspend_status_function_projection", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/scoping/callable/status_application.rs", "start_line": "56", "start_column": "9", "end_line": "56", "end_column": "70", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::suspend_status_function_projection@crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:49:5-57:6: rum_runtime::runtime::model::session::Session::suspend_status_function_projection", "reason": "method syntax: method instance is not in the finite scalar substitution set", "class": "method syntax: method instance is not in the finite scalar substitution set"}
ADDED {"population": "EDGE_ALL", "roots": "rum_runtime::runtime::model::session::Session::suspend_status_function_value@crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:59:5-67:6", "definition": "rum_runtime::runtime::model::session::Session::suspend_status_function_value", "kind": "RET002", "file": "crates/rum-runtime/src/runtime/scoping/callable/status_application.rs", "start_line": "66", "start_column": "9", "end_line": "66", "end_column": "65", "target": "-", "witness": "rum_runtime::runtime::model::session::Session::suspend_status_function_value@crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:59:5-67:6: rum_runtime::runtime::model::session::Session::suspend_status_function_value", "reason": "method syntax: method instance is not in the finite scalar substitution set", "class": "method syntax: method instance is not in the finite scalar substitution set"}
```

### Gate: fixed-618

Command: `cargo test --offline --manifest-path /home/remi/dev/rum/.temp/brief-619/fixed-618/tree/Cargo.toml -p rum-stdlib --test zz_brief618_probe -- --nocapture`. Exit 0. Full raw output: `logs/fixed-618.log`.

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.47s
continuation-slot fault-occurrence delta=0; observations=1620
```

## Fixed brief-618 set

Fixed 618 set: 136 pairs; SAME 101 -> 109; DIFF 35 -> 27; invalid 0; SAME -> DIFF 0.

Fixed rows: director.01, director.02, director.03, director.04, director.05, director.06, status.raw_property, status.success. All 27 remaining differences are among the base 35. Raw observations: `fixed-618/tree/.temp/brief-618/`. Probe, body and cases were unchanged.

### Gate: director-build

Command: `cargo build --offline --manifest-path /home/remi/dev/rum/.temp/brief-619/director-probe/Cargo.toml`. Exit 0. Full raw output: `logs/director-build.log`.

```text
No diagnostics.
```

```text
PROBE g15 PASS same=14 diff=0 pairs=14
```
Raw observations: `director-g15-probe/stdout.txt`.

```text
PROBE s10 PASS same=51 diff=7 pairs=58
```
Raw observations: `director-s10-probe/stdout.txt`.

```text
PROBE s9 PASS same=61 diff=0 pairs=61
```
Raw observations: `director-s9-probe/stdout.txt`.

```text
PROBE s8 PASS same=48 diff=0 pairs=48
```
Raw observations: `director-s8-probe/stdout.txt`.

```text
PROBE s7 PASS same=32 diff=0 pairs=32
```
Raw observations: `director-s7-probe/stdout.txt`.

```text
PROBE s6 PASS same=22 diff=0 pairs=22
```
Raw observations: `director-s6-probe/stdout.txt`.

## Scratch-location deviation, corrected

The copied director runner imported the G15 and S10 pair modules before its no-bytecode guard was installed, creating `.temp/director/{g15,s10}-probe/__pycache__/pairs.cpython-314.pyc`. Their creation timestamps match this invocation. Only those new cache files were moved to `probe-bytecode-deviation/`, and the resulting empty cache directories were removed. Original pair sources and older S6–S9 caches were untouched. Hashes/timestamps are retained in `probe-bytecode-deviation/records.json`; future external imports disable bytecode writes, and measurement parsers are copied under brief-619. No source or runtime result changed.

### Gate: oracle-final

Command: `scripts/check-suspension-oracle.sh 1500`. Exit 0. Full raw output: `logs/oracle-final.log`.

```text
test result: ok. 422 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.92s
test result: ok. 422 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.96s
continuation-slot fault-occurrence delta=0; observations=93750
```

## Full oracle evidence by build

default
```text
test result: ok. 422 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.92s
continuation-slot fault-occurrence delta=0; observations=93750
compiler warnings: 0
```
no-default
```text
test result: ok. 422 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.96s
continuation-slot fault-occurrence delta=0; observations=93750
compiler warnings: 0
```

The complete 53 new fixtures pass in each full build. Every intermediate fault-occurrence report is also zero. Canonical raw logs: `.temp/suspension-oracle/gate-default.txt` and `gate-no-default.txt`; the unchanged gate also owns `.temp/suspension-oracle/tmp/`.

## Files changed

18 existing source/inventory files and six new source/test files are changed. File-by-file semantics follow; every row-level inventory change is disclosed in the replay sections above.

| File | Change |
| --- | --- |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs` | Re-export the private runtime classifier and projection-plan type for the callable module; no other module-root change. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs` | Replace the old fallible status-call selector with a session-free classifier, extract pure property fallback, and expose a narrow virtual-message projection seam. Existing S10 frames and macro sites stay unchanged. |
| `crates/rum-runtime/src/runtime/scoping/callable.rs` | Declare the status_application child and return the complete status Function-stage result directly; all other branches and the common F0528 tail stay unchanged. |
| `crates/rum-runtime/src/runtime/scoping/callable/status_application.rs` | New unit projection/value typed frames, actual-result witnesses, two cold suspension helpers and shared evaluation/finish/Some phase with distinct Return ownership. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | Derived named-frame pin 563 -> 565. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | Add exactly the two new unit types to both closed derive inventories. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Replay coordinate moves and add the two new frame-derive expansion unknowns; no disposition or mark changes. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Replay coordinate moves (including extracted fallback closure) and add five new closure-capture unknowns. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Replay macro-span moves and add the classifier matches! expansion unknown. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Install exactly 68 canonical fingerprint cells; all other contract fields unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Install four canonical certificate cells for two existing bodies in both builds; all non-hash contents and coverage unchanged. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | Register the new status_function fixture module. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/mod.rs` | Shared strict fixture runner with Checks::ALL, unconditional fault absence, exact values/logs/prompts and existing lifecycle checks. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/controls.rs` | 29 green status/other-consumer/held/spliced/projection-and-value controls. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/guest_heads.rs` | 16 guest-marker preservation cases across Message and raw Failure/Success properties. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/return_boundaries.rs` | Eight phase-specific Return, enclosing-body, loop/store and consecutive-pause cases. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` | Receipt stage S9 -> G15-A. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | Append two typed frame rows, their two witnessed sites and implemented G15-A stage; all existing rows untouched. |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | Register G15-A checks; update current-stage and derived forced-red partition pins (+2 implementations). |
| `crates/rum-stdlib/tests/typed_slot_census/g15a.rs` | New exact cohort checks for both Expr -> Option<Expr> frames and witnessed insertion routes; no global-mode change. |
| `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` | Production implementations 606 -> 608, combined 622 -> 624; frame IDs 625 -> 627 and site IDs 603 -> 605; stage output and witnessed-route recognition updated. |
| `crates/rum-stdlib/tests/typed_slot_census/reporting.rs` | Include the new callable child in witnessed-site reporting. |
| `crates/rum-stdlib/tests/typed_slot_census/s4.rs` | Recognize G15-A as a later cohort when preserving S4 counts. |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | Add the classifier-without-Session compiler pair; positive/rejection totals 35 -> 36; all existing cases retained. |
| `docs/dev/codex/brief-619-g15a-status-function-stage.md` | This full report is appended at handoff, as explicitly requested. |

### Gate: workspace-final

Command: `cargo test --workspace --no-fail-fast -- --nocapture`. Exit 0. Full raw output: `logs/workspace-final.log`.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 51.48s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.15s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 43.28s
test result: ok. 1109 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1333.65s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.95s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 422 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.08s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.61s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.62s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 275.67s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.68s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.96s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.65s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: unused import: `rum_core::Symbol`
warning: unreachable call
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: unused import: `rum_core::Symbol`
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: 1 warning emitted
continuation-slot fault-occurrence delta=0; observations=93750
```

## Workspace disposition

Workspace exit 0. The nine native ZeroMQ cases below are **skips**, although the Rust harness prints `ok` after their early return. The director runs them natively. All other totals are preserved verbatim in the gate section above; the two nested failing totals are the deliberate census falsifiers. The runtime-only helper warnings are unchanged. Other warning diagnostics come from deliberate option-read compiler fixtures and a documentation example; neither oracle build warns, and no changed production source emits a warning.

```text
zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages — skipped: loopback binding denied; RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
```

Candidate `rum-cli` built from the workspace cwd, one codegen unit, separate target. SHA-256 `8a087b75e28985ec961d23eed72957be6184a4648f4594b30f63c2d5fd46315b`. Build command, source hashes and raw output: `builds/candidate-rum-cli/`.

## Canonical tool output paths

The unchanged gate tools wrote their fixed output roots below, as authorized by the canonical-output exception. `canonical-output-paths.tsv` records every surviving generated file under those roots (including their scratch source copies and build caches) written since the initial hash verification. Ordinary Cargo outputs are under `target/`; task-directed builds and probes are under this brief. The ownership root-set falsifier temporarily adds a source probe to rum-syntax/ast.rs and restores the exact snapshot; the final source audit verifies restoration.

- `.temp/brief-542/`: 23 generated files.
- `.temp/brief-543/`: 442 generated files.
- `.temp/brief-546/`: 73 generated files.
- `.temp/brief-554/`: 18 generated files.
- `.temp/brief-579/`: 127 generated files.
- `.temp/typed-slot-census/`: 5884 generated files.
- `.temp/typed-slot-compile-controls/`: 1876 generated files.
- `target/brief-531/`: 7 generated files.
- `target/brief-535/`: 6563 generated files.
- `target/brief-538/`: 45 generated files.
- `target/brief-539/`: 30 generated files.
- `target/brief-540/`: 146 generated files.
- `target/brief-544/`: 1 generated files.
- `.temp/suspension-oracle/`: 2 generated files.

Candidate `rum-cli-alloc` built from the workspace cwd, one codegen unit, separate target. SHA-256 `1d3dde5bed71ce464a22f99bc35ccce859571678740ff7f35b4ad8780a61fdcb`. Build command, source hashes and raw output: `builds/candidate-rum-cli-alloc/`.

Candidate `resume` built from the workspace cwd, one codegen unit, separate target. SHA-256 `0b19b903f9a43231b3a6ed94382039683a38ec1e7cb9a2bae528ce9bff4f22e1`. Build command, source hashes and raw output: `builds/candidate-resume/`.

Candidate `resume-alloc` built from the workspace cwd, one codegen unit, separate target. SHA-256 `094025b1a5cbdcfb9a8d3ec96e98494cbab2e19dab40566ade0c000fbcea318e`. Build command, source hashes and raw output: `builds/candidate-resume-alloc/`.

### Gate: candidate-preflight

Command: `python3 .temp/brief-619/run-perf.py check candidate 1`. Exit 0. Full raw output: `logs/candidate-preflight.log`.

```text
No diagnostics.
```

Candidate preflight: all 740 runs correct (178 ordinary check/N/2N plus 103 resume N/2N); exact prompt-resume totals checked. Raw rows: `perf/check/candidate/1/results.json`.

### Gate: allocation-base

Command: `python3 .temp/brief-619/run-perf.py alloc base 1`. Exit 0. Full raw output: `logs/allocation-base.log`.

```text
No diagnostics.
```

### Gate: allocation-candidate

Command: `python3 .temp/brief-619/run-perf.py alloc candidate 1`. Exit 0. Full raw output: `logs/allocation-candidate.log`.

```text
No diagnostics.
```

### Full allocation and byte comparison

178/178 ordinary labels have exactly equal whole-program allocation counts and bytes at both N and 2N, and equal slopes. 97/103 resume labels are also raw-exact; only six G15 Function consumers differ. Every run is eligible and passed its exact value/resume-count preflight. Complete raw N/2N counts, byte totals and slopes: `perf/allocation-comparison.{json,tsv}`. Allocation sites are attributed below after the callgrind census.

| Resume label | Count / iteration delta | Bytes / iteration delta | Count intercept delta | Bytes intercept delta |
|---|---:|---:|---:|---:|
| resume-g15-parameters | 5.0 | 160.0 | 0.0 | 0.0 |
| resume-g15-template | 5.0 | 160.0 | 0.0 | 0.0 |
| resume-g15-styled | 5.0 | 160.0 | 0.0 | 0.0 |
| resume-g15-raw-failure | 4.0 | 144.0 | 0.0 | 0.0 |
| resume-g15-raw-success | 4.0 | 144.0 | 0.0 | 0.0 |
| resume-g15-projection-value | 8.0 | 312.0 | 0.0 | 0.0 |

### Gate: instructions-base-1

Command: `python3 .temp/brief-619/run-perf.py instructions base 1`. Exit 0. Full raw output: `logs/instructions-base-1.log`.

```text
No diagnostics.
```

### Gate: instructions-candidate-1

Command: `python3 .temp/brief-619/run-perf.py instructions candidate 1`. Exit 0. Full raw output: `logs/instructions-candidate-1.log`.

```text
No diagnostics.
```

### Gate: instructions-base-2

Command: `python3 .temp/brief-619/run-perf.py instructions base 2`. Exit 0. Full raw output: `logs/instructions-base-2.log`.

```text
No diagnostics.
```

### Gate: instructions-candidate-2

Command: `python3 .temp/brief-619/run-perf.py instructions candidate 2`. Exit 0. Full raw output: `logs/instructions-candidate-2.log`.

```text
No diagnostics.
```

### Initial instruction slopes

Two interleaved perf-stat pairs per arm completed in base/candidate/base/candidate order, CPU 2 and stdin `/dev/null`, all 281 labels eligible and all 2,248 runs correct. Full raw slopes, ranges, pair deltas and thresholds: `perf/instruction-comparison.{json,tsv}`. Initial ordinary flags (25): lazy-range-normal-small, replaceall-simple, lazy-depth-select-restart-8, lazy-depth-nest-map-8, lazy-depth-map-build-64, lazy-depth-nest-map-256, block-local, rewrite-guarded-hit, rewrite-guarded-decline-fallback, appendto-part-outer1000-small, set-part-fixed100-len100, symbol-ownvalue-read, symbol-ownvalue-guarded-read, downvalue-pattern-miss-repeat, reversesort-default-wide, stringreplace-single-literal-unguarded, stringreplace-guarded-decline-fallback, replaceat-small, catch-no-throw, checkabort-caught, abortprotect-clean, assert-enabled-true, check-clean, assert-off, confirm-failure-message. Five extra interleaved runs per arm follow.

### Gate: instructions-extra-1-base

Command: `python3 .temp/brief-619/run-perf.py instructions base extra-1 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-1-base.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-1-candidate

Command: `python3 .temp/brief-619/run-perf.py instructions candidate extra-1 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-1-candidate.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-2-base

Command: `python3 .temp/brief-619/run-perf.py instructions base extra-2 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-2-base.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-2-candidate

Command: `python3 .temp/brief-619/run-perf.py instructions candidate extra-2 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-2-candidate.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-3-base

Command: `python3 .temp/brief-619/run-perf.py instructions base extra-3 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-3-base.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-3-candidate

Command: `python3 .temp/brief-619/run-perf.py instructions candidate extra-3 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-3-candidate.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-4-base

Command: `python3 .temp/brief-619/run-perf.py instructions base extra-4 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-4-base.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-4-candidate

Command: `python3 .temp/brief-619/run-perf.py instructions candidate extra-4 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-4-candidate.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-5-base

Command: `python3 .temp/brief-619/run-perf.py instructions base extra-5 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-5-base.log`.

```text
No diagnostics.
```

### Gate: instructions-extra-5-candidate

Command: `python3 .temp/brief-619/run-perf.py instructions candidate extra-5 cli @.temp/brief-619/perf/ordinary-exceptions.json`. Exit 0. Full raw output: `logs/instructions-extra-5-candidate.log`.

```text
No diagnostics.
```

Five extra interleaved instruction runs per arm completed on all 25 initial flags, 500 valid N/2N processes. Twenty-four flags clear the five-run threshold; `block-local` persists. Full means, ranges and threshold evidence: `perf/ordinary-five-run-comparison.{json,tsv}`.

```json
[
  {
    "label": "block-local",
    "engine": "cli",
    "N": 20000,
    "eligible": true,
    "base_slopes": [
      14462.7438,
      14462.84085,
      14462.96655,
      14462.88445,
      14462.77665
    ],
    "base_mean": 14462.84246,
    "base_range": 0.22274999999899592,
    "candidate_slopes": [
      14471.9681,
      14471.8872,
      14471.96675,
      14471.95445,
      14472.0112
    ],
    "candidate_mean": 14471.95754,
    "candidate_range": 0.12400000000161526,
    "delta": 9.11507999999958,
    "threshold": 7.2314212300000005,
    "pair_deltas": [
      9.224299999999857,
      9.04634999999871,
      9.000200000000405,
      9.069999999999709,
      9.234550000001036
    ],
    "flagged": true
  }
]
```

### Gate: ordinary-callgrind-flags

Command: `python3 .temp/brief-619/run-callgrind.py flags 6`. Exit 0. Full raw output: `logs/ordinary-callgrind-flags.log`.

```text
No diagnostics.
```

### Gate: function-self-profiles

Command: `python3 .temp/brief-619/run-callgrind.py labels 1 downvalue-exact-miss downvalue-pattern-miss downvalue-exact-miss-repeat downvalue-pattern-miss-repeat subvalue-unguarded-miss default-unguarded-miss function-slot-apply`. Exit 0. Full raw output: `logs/function-self-profiles.log`.

```text
No diagnostics.
```

### Gate: function-self-analysis

Command: `python3 .temp/brief-619/analyze-ordinary-fns.py`. Exit 0. Full raw output: `logs/function-self-analysis.log`.

```text
No diagnostics.
```

### Gate: ordinary-instruction-attribution

Command: `python3 .temp/brief-619/analyze-ordinary.py`. Exit 0. Full raw output: `logs/ordinary-instruction-attribution.log`.

```text
No diagnostics.
```

### Gate: copy-census-chunk-1

Command: `python3 .temp/brief-619/run-copy.py`. Exit 0. Full raw output: `logs/copy-census-chunk-1.log`.

```text
No diagnostics.
```

### Required Function self-cost measurements and corrected interpretation

The unchanged `eval_dispatch/mod.rs:88-96` adds the Function stage only for recognized callable heads. Contrary to the broad hot-path premise in the brief, these six supplied miss workloads do not reach `try_apply_function`: both arms have zero observed self-instruction and call slopes. Their per-call quotient is therefore undefined. The preliminary helper log `logs/function-self-analysis.log` labelled zero calls "inlined"; that label was incorrect and the helper has been corrected. No production source changed to obtain these observations. For `function-slot-apply`, the method runs once per iteration and its self cost decreases **121 -> 119 instructions/call**. Complete measurements: `ordinary-callgrind/function-self.json`; all seven N/2N callgrind pairs are retained under `ordinary-callgrind/1/`.

### Block-local instruction residual: six callgrind pairs

All 24 processes were valid (six N/2N samples per arm). Every function is diffed for each sample, with all contributing addresses and raw deltas retained in `ordinary-callgrind/per-label/block-local.json`. No nonzero deterministic function-level total survives all six samples, but libc memcmp contains a repeatable **+9 instructions/iteration** address-level component. Its complete shared-address subtraction is `ordinary-callgrind/block-local-memcmp-address-deltas.json`; assembly inspection follows. The five-run perf mean was +9.11508/iteration against a 7.23142123 threshold, so this remains a reported residual rather than a clean instruction pass.

| libc address | Deterministic instructions/iteration delta |
|---|---:|
| 0x1b152e | -1.0 |
| 0x1b1530 | -1.0 |
| 0x1b1533 | -1.0 |
| 0x1b1600 | 1.0 |
| 0x1b1603 | 1.0 |
| 0x1b1606 | 1.0 |
| 0x1b1608 | 1.0 |
| 0x1b160a | 1.0 |
| 0x1b160c | 1.0 |
| 0x1b160e | 1.0 |
| 0x1b1613 | 1.0 |
| 0x1b1618 | 1.0 |
| 0x1b161a | 1.0 |
| 0x1b161c | 1.0 |
| 0x1b161e | 1.0 |

### Calibrated copy census, all ordinary labels

All 712 N/2N processes (178 labels, both arms) completed with the expected result, using the calibrated offline libc debuginfo. Canonical tool output: `copy-census/canonical-census.log`; whole-program counts and slopes: `copy-census/comparison.json`.

```text
lazy-range-take-small                         N=477   memcpy 129.000->129.000 memmove 0.000->0.000  ok
lazy-range-normal-small                       N=373   memcpy 143.000->143.000 memmove 0.000->0.000  ok
lazy-map-realize-small                        N=113   memcpy 295.000->295.000 memmove 0.000->0.000  ok
lazy-select-realize-small                     N=177   memcpy 221.000->221.000 memmove 0.000->0.000  ok
lazy-select-scan-small                        N=25    memcpy 1351.000->1351.000 memmove 0.000->0.000  ok
lazy-nestlist-realize-small                   N=139   memcpy 177.000->177.000 memmove 0.000->0.000  ok
lazy-foldlist-realize-small                   N=180   memcpy 203.000->203.000 memmove 0.000->0.000  ok
lazy-drop-realize-small                       N=106   memcpy 311.000->311.000 memmove 0.000->0.000  ok
lazy-first-small                              N=492   memcpy 59.000->59.000 memmove 0.000->0.000  ok
lazy-part-small                               N=229   memcpy 137.000->137.000 memmove 0.000->0.000  ok
lazy-transparent-adapters-small               N=111   memcpy 285.000->285.000 memmove 0.000->0.000  ok
lazy-table-construct-short                    N=857   memcpy 50.000->50.000 memmove 0.000->0.000  ok
lazy-table-construct-full                     N=838   memcpy 51.000->51.000 memmove 0.000->0.000  ok
lazy-table-realize-small                      N=172   memcpy 211.000->211.000 memmove 0.000->0.000  ok
lazy-permutations-take-small                  N=717   memcpy 24.000->24.000 memmove 0.000->0.000  ok
lazy-subsets-take-small                       N=799   memcpy 24.000->24.000 memmove 0.000->0.000  ok
lazy-tuples-take-small                        N=760   memcpy 24.000->24.000 memmove 0.000->0.000  ok
scan-no-input                                 N=690   memcpy 51.000->51.000 memmove 0.000->0.000  ok
optionvalue-three-no-input                    N=2000  memcpy 7.000->7.000 memmove 0.000->0.000  ok
replacerepeated-no-input                      N=514   memcpy 29.000->29.000 memmove 0.000->0.000  ok
dot-three-no-input                            N=80    memcpy 465.000->465.000 memmove 0.000->0.000  ok
differences-no-input                          N=2000  memcpy 9.000->9.000 memmove 0.000->0.000  ok
do-count-null                                 N=2000  memcpy 1.000->1.000 memmove 0.000->0.000  ok
do-iterator-null                              N=2000  memcpy 2.000->2.000 memmove 0.000->0.000  ok
table-iterator-null                           N=2000  memcpy 0.000->0.000 memmove 0.000->0.000  ok
replaceall-simple                             N=1770  memcpy 23.000->23.000 memmove 0.000->0.000  ok
table-small                                   N=1379  memcpy 13.000->13.000 memmove 0.000->0.000  ok
table-iterator-plus                           N=2000  memcpy 1.000->1.000 memmove 0.000->0.000  ok
table-iterator-pair-list                      N=2000  memcpy 0.000->0.000 memmove 0.000->0.000  ok
take-multispec-small                          N=402   memcpy 47.000->47.000 memmove 0.000->0.000  ok
lazy-depth-map-build-8                        N=1000  memcpy 25.000->25.000 memmove 0.000->0.000  ok
lazy-depth-map-first-8                        N=155   memcpy 172.000->172.000 memmove 0.000->0.000  ok
lazy-depth-map-eight-8                        N=21    memcpy 1219.000->1219.000 memmove 0.000->0.000  ok
lazy-depth-select-restart-8                   N=112   memcpy 281.000->281.000 memmove 0.000->0.000  ok
lazy-depth-drop-all-one-8                     N=431   memcpy 155.000->155.000 memmove 0.000->0.000  ok
lazy-depth-take-map-8                         N=146   memcpy 188.000->188.000 memmove 0.000->0.000  ok
lazy-depth-fold-through-source-8              N=34    memcpy 696.000->696.000 memmove 0.000->0.000  ok
lazy-depth-nest-map-8                         N=144   memcpy 178.000->178.000 memmove 0.000->0.000  ok
lazy-depth-map-build-64                       N=671   memcpy 25.000->25.000 memmove 0.000->0.000  ok
lazy-depth-map-first-64                       N=21    memcpy 1124.000->1124.000 memmove 0.000->0.000  ok
lazy-depth-map-eight-64                       N=2     memcpy 8835.000->8835.000 memmove 0.000->0.000  ok
lazy-depth-select-restart-64                  N=20    memcpy 1401.000->1401.000 memmove 0.000->0.000  ok
lazy-depth-drop-all-one-64                    N=61    memcpy 995.000->995.000 memmove 0.000->0.000  ok
lazy-depth-take-map-64                        N=20    memcpy 1252.000->1252.000 memmove 0.000->0.000  ok
lazy-depth-fold-through-source-64             N=1     memcpy 37549.000->37549.000 memmove 0.000->0.000  ok
lazy-depth-nest-map-64                        N=21    memcpy 1130.000->1130.000 memmove 0.000->0.000  ok
lazy-depth-map-build-256                      N=283   memcpy 25.000->25.000 memmove 0.000->0.000  ok
lazy-depth-map-first-256                      N=5     memcpy 4388.000->4388.000 memmove 0.000->0.000  ok
lazy-depth-map-eight-256                      N=1     memcpy 34949.000->34949.000 memmove 0.000->0.000  ok
lazy-depth-select-restart-256                 N=5     memcpy 5241.000->5241.000 memmove 0.000->0.000  ok
lazy-depth-drop-all-one-256                   N=9     memcpy 3875.000->3875.000 memmove 0.000->0.000  ok
lazy-depth-take-map-256                       N=5     memcpy 4900.000->4900.000 memmove 0.000->0.000  ok
lazy-depth-fold-through-source-256            N=1     memcpy 592431.000->592431.000 memmove 0.000->0.000  ok
lazy-depth-nest-map-256                       N=5     memcpy 4394.000->4394.000 memmove 0.000->0.000  ok
literal-integer                               N=2000  memcpy 1.000->1.000 memmove 0.000->0.000  ok
downvalue-pattern-hit                         N=1465  memcpy 17.000->17.000 memmove 0.000->0.000  ok
module-local                                  N=891   memcpy 32.000->32.000 memmove 0.000->0.000  ok
block-local                                   N=1382  memcpy 29.000->29.000 memmove 0.000->0.000  ok
matcher-single-wide-small                     N=26    memcpy 15.000->15.000 memmove 0.000->0.000  ok
cases-wide-no-match                           N=8     memcpy 37.000->37.000 memmove 0.000->0.000  ok
count-wide-no-match                           N=9     memcpy 32.000->32.000 memmove 0.000->0.000  ok
position-wide-no-match                        N=7     memcpy 41.000->41.000 memmove 0.000->0.000  ok
deletecases-wide-no-match                     N=7     memcpy 22.000->22.000 memmove 0.000->0.000  ok
deletecases-wide-association-no-match         N=2     memcpy 31.000->31.000 memmove 0.000->0.000  ok
deletecases-leaf-delete-small                 N=317   memcpy 12.000->12.000 memmove 0.000->0.000  ok
replace-wide-no-match                         N=17    memcpy 32.000->32.000 memmove 0.000->0.000  ok
replace-leaf-rebuild-small                    N=272   memcpy 22.000->22.000 memmove 0.000->0.000  ok
rewrite-guarded-hit                           N=379   memcpy 51.000->51.000 memmove 0.000->0.000  ok
rewrite-guarded-decline-fallback              N=259   memcpy 78.000->78.000 memmove 0.000->0.000  ok
rule-rhs-deep-small                           N=242   memcpy 77.000->77.000 memmove 0.000->0.000  ok
part-list                                     N=2000  memcpy 15.000->15.000 memmove 0.000->0.000  ok
appendto-part-outer1000-small                 N=9     memcpy 2842.000->2842.000 memmove 0.000->0.000  ok
appendto-part-outer1000-large                 N=5     memcpy 4643.000->4643.000 memmove 0.000->0.000  ok
set-part-fixed100-len100                      N=15    memcpy 1713.000->1713.000 memmove 0.000->0.000  ok
set-part-fixed100-len800                      N=15    memcpy 1713.000->1713.000 memmove 0.000->0.000  ok
set-part-fill-small                           N=7     memcpy 3813.000->3813.000 memmove 0.000->0.000  ok
set-part-fill-large                           N=3     memcpy 7613.000->7613.000 memmove 0.000->0.000  ok
table-iterator-symbol                         N=2000  memcpy 0.000->0.000 memmove 0.000->0.000  ok
symbol-ownvalue-read                          N=2000  memcpy 4.000->4.000 memmove 0.000->0.000  ok
symbol-ownvalue-guarded-read                  N=2000  memcpy 6.000->6.000 memmove 0.000->0.000  ok
block-symbol-only-repeat                      N=1871  memcpy 24.000->24.000 memmove 0.000->0.000  ok
downvalue-exact-hit                           N=2000  memcpy 8.000->8.000 memmove 0.000->0.000  ok
downvalue-exact-miss                          N=2000  memcpy 2.000->2.000 memmove 0.000->0.000  ok
downvalue-pattern-miss                        N=2000  memcpy 2.000->2.000 memmove 0.000->0.000  ok
downvalue-guarded-hit                         N=725   memcpy 32.000->32.000 memmove 0.000->0.000  ok
downvalue-guarded-decline-fallback            N=958   memcpy 21.000->21.000 memmove 0.000->0.000  ok
upvalue-guarded-hit                           N=590   memcpy 35.000->35.000 memmove 0.000->0.000  ok
subvalue-unguarded-hit                        N=996   memcpy 25.000->25.000 memmove 0.000->0.000  ok
subvalue-unguarded-miss                       N=1772  memcpy 17.000->17.000 memmove 0.000->0.000  ok
subvalue-guarded-hit                          N=567   memcpy 41.000->41.000 memmove 0.000->0.000  ok
default-unguarded-hit                         N=1163  memcpy 14.000->14.000 memmove 0.000->0.000  ok
default-unguarded-miss                        N=1564  memcpy 12.000->12.000 memmove 0.000->0.000  ok
default-guarded-hit                           N=894   memcpy 15.000->15.000 memmove 0.000->0.000  ok
downvalue-exact-miss-repeat                   N=2000  memcpy 2.000->2.000 memmove 0.000->0.000  ok
downvalue-pattern-miss-repeat                 N=2000  memcpy 2.000->2.000 memmove 0.000->0.000  ok
upvalue-candidates-tiny                       N=637   memcpy 41.000->41.000 memmove 0.000->0.000  ok
upvalue-candidates-wide-small                 N=6     memcpy 1646.000->1646.000 memmove 0.000->0.000  ok
upvalue-candidates-wide-large                 N=3     memcpy 3247.000->3247.000 memmove 0.000->0.000  ok
sort-default-wide                             N=20    memcpy 12.000->12.000 memmove 0.000->0.000  ok
sortby-default-wide                           N=2     memcpy 11222.000->11222.000 memmove 0.000->0.000  ok
reversesort-default-wide                      N=20    memcpy 12.000->12.000 memmove 0.000->0.000  ok
keysort-default-wide                          N=2     memcpy 1639.000->1639.000 memmove 0.000->0.000  ok
keysortby-default-wide                        N=1     memcpy 12842.000->12842.000 memmove 0.000->0.000  ok
sort-symbol-default-width4000                 N=3     memcpy 18.000->18.000 memmove 0.000->0.000  ok
sort-symbol-default-width16000                N=1     memcpy 23.000->23.000 memmove 0.000->0.000  ok
valueq-undefined-symbol-definitions-2000      N=20    memcpy 7.000->7.000 memmove 0.000->0.000  ok
valueq-undefined-symbol-definitions-8000      N=20    memcpy 7.000->7.000 memmove 0.000->0.000  ok
matcher-single-tiny                           N=1000  memcpy 8.000->8.000 memmove 0.000->0.000  ok
matcher-single-wide-large                     N=13    memcpy 17.000->17.000 memmove 0.000->0.000  ok
replacepart-deep-path-small                   N=489   memcpy 48.000->48.000 memmove 0.000->0.000  ok
replacepart-wide-rule-list                    N=3     memcpy 4817.000->4817.000 memmove 0.000->0.000  ok
replacepart-guarded-hit                       N=500   memcpy 24.000->24.000 memmove 0.000->0.000  ok
replacepart-guarded-decline-fallback          N=500   memcpy 28.000->28.000 memmove 0.000->0.000  ok
stringreplace-single-literal-unguarded        N=200   memcpy 34.000->34.000 memmove 0.000->0.000  ok
stringreplace-pattern-unguarded               N=500   memcpy 37.000->37.000 memmove 0.000->0.000  ok
stringreplace-guarded-hit                     N=500   memcpy 34.000->34.000 memmove 0.000->0.000  ok
stringreplace-guarded-decline-fallback        N=500   memcpy 38.000->38.000 memmove 0.000->0.000  ok
association-part-key-read-width4000           N=40    memcpy 21.000->21.000 memmove 0.000->0.000  ok
association-part-positional-read-width4000    N=40    memcpy 16.000->16.000 memmove 0.000->0.000  ok
association-part-key-read-width32000          N=40    memcpy 21.000->21.000 memmove 0.000->0.000  ok
association-part-positional-read-width32000   N=40    memcpy 16.000->16.000 memmove 0.000->0.000  ok
appendto-list-incremental-small               N=250   memcpy 12.004->12.004 memmove 0.000->0.000  ok
appendto-list-incremental-large               N=500   memcpy 12.002->12.002 memmove 0.000->0.000  ok
association-set-key-incremental-small         N=100   memcpy 14.030->14.030 memmove 0.000->0.000  ok
association-set-key-incremental-large         N=200   memcpy 14.015->14.015 memmove 0.000->0.000  ok
association-set-part-key-investigation        N=100   memcpy 21.000->21.000 memmove 0.000->0.000  ok
table-count-null                              N=8     memcpy 10.000->10.000 memmove 0.000->0.000  ok
replaceall-wide-no-match                      N=24    memcpy 35.000->35.000 memmove 0.000->0.000  ok
replaceall-leaf-rebuild-small                 N=304   memcpy 35.000->35.000 memmove 0.000->0.000  ok
replaceall-guarded-hit-small                  N=254   memcpy 82.000->82.000 memmove 0.000->0.000  ok
replace-rule-list-decline                     N=391   memcpy 49.000->49.000 memmove 0.000->0.000  ok
replacerepeated-guarded-small                 N=53    memcpy 484.000->484.000 memmove 0.000->0.000  ok
replaceat-small                               N=500   memcpy 41.000->41.000 memmove 0.000->0.000  ok
memberq-wide-no-match                         N=9     memcpy 28.000->28.000 memmove 0.000->0.000  ok
freeq-wide-no-match                           N=8     memcpy 28.000->28.000 memmove 0.000->0.000  ok
count-wide-match                              N=9     memcpy 31.000->31.000 memmove 0.000->0.000  ok
count-guarded-small                           N=398   memcpy 44.000->44.000 memmove 0.000->0.000  ok
position-small-match                          N=500   memcpy 11.000->11.000 memmove 0.000->0.000  ok
catch-no-throw                                N=500   memcpy 9.000->9.000 memmove 0.000->0.000  ok
catch-throw-untagged                          N=500   memcpy 20.000->20.000 memmove 0.000->0.000  ok
catch-throw-tagged-hit                        N=500   memcpy 22.000->22.000 memmove 0.000->0.000  ok
catch-throw-pattern-hit                       N=500   memcpy 25.000->25.000 memmove 0.000->0.000  ok
checkabort-no-abort                           N=500   memcpy 7.000->7.000 memmove 0.000->0.000  ok
checkabort-caught                             N=500   memcpy 17.000->17.000 memmove 0.000->0.000  ok
abortprotect-clean                            N=500   memcpy 14.000->14.000 memmove 0.000->0.000  ok
assert-disabled                               N=500   memcpy 7.000->7.000 memmove 0.000->0.000  ok
assert-enabled-true                           N=500   memcpy 19.000->19.000 memmove 0.000->0.000  ok
check-clean                                   N=500   memcpy 14.000->14.000 memmove 0.000->0.000  ok
check-message-fallback                        N=500   memcpy 40.000->40.000 memmove 0.000->0.000  ok
quiet-clean                                   N=500   memcpy 15.000->15.000 memmove 0.000->0.000  ok
withcleanup-two-arg                           N=500   memcpy 23.000->23.000 memmove 0.000->0.000  ok
catchexceptions-caught                        N=500   memcpy 41.000->41.000 memmove 0.000->0.000  ok
catch-throw                                   N=500   memcpy 20.000->20.000 memmove 0.000->0.000  ok
catch-tagged-throw                            N=500   memcpy 22.000->22.000 memmove 0.000->0.000  ok
catch-form-pattern                            N=500   memcpy 25.000->25.000 memmove 0.000->0.000  ok
checkabort-abort                              N=500   memcpy 17.000->17.000 memmove 0.000->0.000  ok
abortprotect-small                            N=500   memcpy 7.000->7.000 memmove 0.000->0.000  ok
withcleanup-small                             N=500   memcpy 11.000->11.000 memmove 0.000->0.000  ok
withcleanup-init                              N=500   memcpy 12.000->12.000 memmove 0.000->0.000  ok
check-no-message                              N=500   memcpy 7.000->7.000 memmove 0.000->0.000  ok
quiet-no-message                              N=500   memcpy 8.000->8.000 memmove 0.000->0.000  ok
catchexceptions-handler                       N=500   memcpy 30.000->30.000 memmove 0.000->0.000  ok
catchexceptions-no-throw                      N=500   memcpy 7.000->7.000 memmove 0.000->0.000  ok
assert-off                                    N=500   memcpy 7.000->7.000 memmove 0.000->0.000  ok
function-slot-apply                           N=500   memcpy 18.000->18.000 memmove 0.000->0.000  ok
function-named-apply                          N=500   memcpy 17.000->17.000 memmove 0.000->0.000  ok
composition-apply                             N=500   memcpy 59.000->59.000 memmove 0.000->0.000  ok
association-apply                             N=500   memcpy 15.000->15.000 memmove 0.000->0.000  ok
failure-raw-property                          N=500   memcpy 23.000->23.000 memmove 0.000->0.000  ok
failure-list-property                         N=500   memcpy 22.000->22.000 memmove 0.000->0.000  ok
failure-missing-property                      N=500   memcpy 25.000->25.000 memmove 0.000->0.000  ok
success-raw-property                          N=500   memcpy 19.000->19.000 memmove 0.000->0.000  ok
failure-message-property                      N=500   memcpy 45.000->45.000 memmove 0.000->0.000  ok
failure-message-delayed                       N=500   memcpy 60.000->60.000 memmove 0.000->0.000  ok
failure-styled-message                        N=500   memcpy 45.000->45.000 memmove 0.000->0.000  ok
failure-tag-fallback                          N=500   memcpy 18.000->18.000 memmove 0.000->0.000  ok
enclose-message-handler                       N=481   memcpy 64.000->64.000 memmove 0.000->0.000  ok
confirm-failure-message                       N=386   memcpy 86.000->87.000 memmove 0.000->0.000  ABOVE memcpy
      +1.000/iter  memcpy  from rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::failure_property_value
labels 178, copy count above base: 1
ABOVE: confirm-failure-message

```

Whole-program counts above base: `[('confirm-failure-message', ['N/memcpy', '2N/memcpy'])]`. Changed-function prologue and store inspection follows in the stack evidence.

### Gate: copy-comparison

Command: `python3 .temp/brief-619/compare-copies.py`. Exit 0. Full raw output: `logs/copy-comparison.log`.

```text
No diagnostics.
```

### Gate: function-self-analysis-corrected

Command: `python3 .temp/brief-619/analyze-ordinary-fns.py`. Exit 0. Full raw output: `logs/function-self-analysis-corrected.log`.

```text
No diagnostics.
```

### Gate: measurement-disassembly

Command: `python3 .temp/brief-619/dump-measurement-functions.py`. Exit 0. Full raw output: `logs/measurement-disassembly.log`.

```text
No diagnostics.
```

`block-local` assembly interpretation: libc `__memcmp_avx2_movbe` tests `(edi | esi) & 0xfff > 0xfe0` at `0x1b1505`–`0x1b1513`. The candidate repeatedly takes the page-boundary-safe 2–3-byte path (`0x1b1540`–`0x1b154d`, then `0x1b1600`–`0x1b161e`) instead of the vector path (`0x1b1515`–`0x1b1533`). The stable address component is +9 instructions/iteration. This is a libc address-layout residual; surrounding symbol-table initialization contributes the small non-deterministic differences retained in the six-process tables. Exact disassembly: `ordinary-callgrind/block-local-memcmp.objdump`. The Function dispatch source is not changed by this explanation.

The corrected self-cost helper output is retained separately in `logs/function-self-analysis-corrected.log`, preserving the preliminary log. `ordinary-callgrind/function-self.json` now also includes whole-program self-instruction and call counts at N and 2N; each of the six miss labels has zero calls in both arms at both sizes.

### Gate: candidate-stack-build

Command: `python3 .temp/brief-619/build-stack.py`. Exit 0. Full raw output: `logs/candidate-stack-build.log`.

```text
No diagnostics.
```

### Gate: candidate-stack-measure

Command: `python3 .temp/brief-619/stack-candidate/measure.py`. Exit 0. Full raw output: `logs/candidate-stack-measure.log`.

```text
No diagnostics.
```

### Final candidate stack measurements

All 42 debug/release fits have zero residual; 21 shapes, four heights and three samples per height. All 504 candidate probe invocations passed; complete raw test output: `stack-candidate/raw-probes.log`. The stack candidate runtime source matches the workspace across 533 Rust files except the exact supplied probe append (`stack-candidate/source-hashes.json`).

| Shape | Base bytes/depth | Candidate bytes/depth | Delta |
|---|---:|---:|---:|
| debug/peel | 19024.0 | 19024.0 | 0.0 |
| debug/plus | 34912.0 | 34912.0 | 0.0 |
| debug/sub | 29376.0 | 29376.0 | 0.0 |
| debug/up | 28064.0 | 28064.0 | 0.0 |
| debug/guarded | 35840.0 | 35840.0 | 0.0 |
| debug/rhs-guard | 49152.0 | 49152.0 | 0.0 |
| debug/replace-guard | 64880.0 | 64880.0 | 0.0 |
| debug/replaceall-guard | 63296.0 | 63296.0 | 0.0 |
| debug/count-guard | 74064.0 | 74064.0 | 0.0 |
| debug/position-guard | 85648.0 | 85648.0 | 0.0 |
| debug/catch-body | 50528.0 | 50528.0 | 0.0 |
| debug/checkabort-body | 52720.0 | 52720.0 | 0.0 |
| debug/abortprotect-body | 49904.0 | 49904.0 | 0.0 |
| debug/cleanup-body | 50880.0 | 50880.0 | 0.0 |
| debug/check-body | 50432.0 | 50432.0 | 0.0 |
| debug/quiet-body | 50304.0 | 50304.0 | 0.0 |
| debug/catchexceptions-body | 49872.0 | 49872.0 | 0.0 |
| debug/function-body | 49760.0 | 49312.0 | -448.0 |
| debug/status-body | 49760.0 | 50816.0 | 1056.0 |
| debug/message-body | 49760.0 | 50816.0 | 1056.0 |
| debug/lhs-arg | 58400.0 | 58400.0 | 0.0 |
| release/peel | 7344.0 | 7344.0 | 0.0 |
| release/plus | 13792.0 | 13792.0 | 0.0 |
| release/sub | 13168.0 | 13168.0 | 0.0 |
| release/up | 11456.0 | 11456.0 | 0.0 |
| release/guarded | 14240.0 | 14240.0 | 0.0 |
| release/rhs-guard | 17920.0 | 17920.0 | 0.0 |
| release/replace-guard | 26768.0 | 26768.0 | 0.0 |
| release/replaceall-guard | 24640.0 | 24640.0 | 0.0 |
| release/count-guard | 28416.0 | 28416.0 | 0.0 |
| release/position-guard | 33632.0 | 33632.0 | 0.0 |
| release/catch-body | 19280.0 | 19280.0 | 0.0 |
| release/checkabort-body | 20864.0 | 20864.0 | 0.0 |
| release/abortprotect-body | 19280.0 | 19280.0 | 0.0 |
| release/cleanup-body | 19280.0 | 19280.0 | 0.0 |
| release/check-body | 18736.0 | 18736.0 | 0.0 |
| release/quiet-body | 18688.0 | 18688.0 | 0.0 |
| release/catchexceptions-body | 19472.0 | 19472.0 | 0.0 |
| release/function-body | 19376.0 | 19456.0 | 80.0 |
| release/status-body | 19376.0 | 20160.0 | 784.0 |
| release/message-body | 19376.0 | 20160.0 | 784.0 |
| release/lhs-arg | 18160.0 | 18160.0 | 0.0 |

Probe totals, verbatim (one row per invocation):

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
```

### Gate: stack-comparison

Command: `python3 .temp/brief-619/compare-stack.py`. Exit 0. Full raw output: `logs/stack-comparison.log`.

```text
No diagnostics.
```

### Gate: prologues-base-debug

Command: `python3 .temp/brief-619/prologues.py base debug .temp/brief-619/stack-evidence/base/debug .temp/brief-619/calib/debug/runtime-tests`. Exit 0. Full raw output: `logs/prologues-base-debug.log`.

```text
No diagnostics.
```

### Gate: prologues-base-release

Command: `python3 .temp/brief-619/prologues.py base release .temp/brief-619/stack-evidence/base/release .temp/brief-619/calib/release/runtime-tests`. Exit 0. Full raw output: `logs/prologues-base-release.log`.

```text
No diagnostics.
```

### Gate: prologues-candidate-debug

Command: `python3 .temp/brief-619/prologues.py candidate debug .temp/brief-619/stack-evidence/candidate/debug .temp/brief-619/stack-candidate/debug/runtime-tests`. Exit 0. Full raw output: `logs/prologues-candidate-debug.log`.

```text
No diagnostics.
```

### Gate: prologues-candidate-release

Command: `python3 .temp/brief-619/prologues.py candidate release .temp/brief-619/stack-evidence/candidate/release .temp/brief-619/stack-candidate/release/runtime-tests`. Exit 0. Full raw output: `logs/prologues-candidate-release.log`.

```text
No diagnostics.
```

### Gate: prologue-comparison

Command: `python3 .temp/brief-619/compare-prologues.py`. Exit 0. Full raw output: `logs/prologue-comparison.log`.

```text
No diagnostics.
```

### Gate: confirm-copy-profiles

Command: `python3 .temp/brief-619/run-callgrind.py labels 6 confirm-failure-message`. Exit 0. Full raw output: `logs/confirm-copy-profiles.log`.

```text
No diagnostics.
```

### Gate: confirm-copy-instruction-attribution

Command: `python3 .temp/brief-619/analyze-ordinary.py`. Exit 0. Full raw output: `logs/confirm-copy-instruction-attribution.log`.

```text
No diagnostics.
```

### Gate: resume-g15-six

Command: `python3 .temp/brief-619/run-callgrind.py labels 6 resume-g15-parameters resume-g15-template resume-g15-styled resume-g15-raw-failure resume-g15-raw-success resume-g15-projection-value resume-g15-enclose resume-g15-confirm`. Exit 0. Full raw output: `logs/resume-g15-six.log`.

```text
No diagnostics.
```

### Gate: candidate-g15-frame-census

Command: `python3 .temp/brief-619/candidate-resume-census.py`. Exit 0. Full raw output: `logs/candidate-g15-frame-census.log`.

```text
No diagnostics.
```

### Gate: g15-resume-function-attribution

Command: `python3 .temp/brief-619/analyze-resume.py resume-g15-parameters resume-g15-template resume-g15-styled resume-g15-raw-failure resume-g15-raw-success resume-g15-projection-value resume-g15-enclose resume-g15-confirm`. Exit 0. Full raw output: `logs/g15-resume-function-attribution.log`.

```text
No diagnostics.
```

### Gate: allocation-sites

Command: `python3 .temp/brief-619/analyze-allocation-sites.py`. Exit 0. Full raw output: `logs/allocation-sites.log`.

```text
No diagnostics.
```

### Gate: ordinary-copy-disassembly

Command: `python3 .temp/brief-619/dump-ordinary-functions.py`. Exit 0. Full raw output: `logs/ordinary-copy-disassembly.log`.

```text
No diagnostics.
```

### Prologues, slot tables, and attribution of stack growth

Every emitted changed callable/status function, closure and new-frame specialization in the two stack measurement binaries is tabulated in `stack-evidence/prologue-comparison.{json,tsv}`; `changed-function-coverage.json` records absent named symbols explicitly (absent does not by itself prove inlining). Each table row points to its full assembly, all stack operands, copy-call contexts, and prologue at `stack-evidence/slots/{debug,release}/N.json`. `failure_property_value`, whose source is unchanged but whose generated code explains the copy residual, is included too. No ABI/cold-helper/source fix followed these measurements.

| Profile | Function | Base prologue bytes | Candidate prologue bytes | Delta | Assembly/stack-operand table |
|---|---|---:|---:|---:|---|
| debug | `<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame as rum_runtime::runtime::suspension::core::FrameAbandon>::abandon` | None | 24 | None | `.temp/brief-619/stack-evidence/slots/debug/1.json` |
| debug | `<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame as rum_runtime::runtime::suspension::trace::FrameRootTrace>::trace_roots` | None | 0 | None | `.temp/brief-619/stack-evidence/slots/debug/2.json` |
| debug | `<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame as rum_runtime::runtime::suspension::typed::TypedFrame>::resume` | None | 680 | None | `.temp/brief-619/stack-evidence/slots/debug/3.json` |
| debug | `<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame as rum_runtime::runtime::suspension::core::FrameAbandon>::abandon` | None | 24 | None | `.temp/brief-619/stack-evidence/slots/debug/4.json` |
| debug | `<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame as rum_runtime::runtime::suspension::trace::FrameRootTrace>::trace_roots` | None | 0 | None | `.temp/brief-619/stack-evidence/slots/debug/5.json` |
| debug | `<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame as rum_runtime::runtime::suspension::typed::TypedFrame>::resume` | None | 344 | None | `.temp/brief-619/stack-evidence/slots/debug/6.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::Arc<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/7.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::Arc<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/8.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::Arc<rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame>>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/9.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::Arc<rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame>>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/10.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::ArcInner<rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame>>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/11.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::ArcInner<rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame>>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/12.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::Weak<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame,&alloc::alloc::Global>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/13.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::Weak<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame,&alloc::alloc::Global>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/14.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::Weak<rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame>,&alloc::alloc::Global>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/15.json` |
| debug | `core::ptr::drop_in_place<alloc::sync::Weak<rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame>,&alloc::alloc::Global>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/16.json` |
| debug | `core::ptr::drop_in_place<rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionProjectionFrame>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/17.json` |
| debug | `core::ptr::drop_in_place<rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<rum_runtime::runtime::scoping::callable::status_application::StatusFunctionValueFrame>>` | None | 8 | None | `.temp/brief-619/stack-evidence/slots/debug/18.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::apply_status_object_to_args` | 1448 | None | None | `.temp/brief-619/stack-evidence/slots/debug/19.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::failure_property_value` | 152 | 152 | 0 | `.temp/brief-619/stack-evidence/slots/debug/20.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::project_status_function_message` | None | 1048 | None | `.temp/brief-619/stack-evidence/slots/debug/21.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::project_status_function_message::{{closure}}` | None | 72 | None | `.temp/brief-619/stack-evidence/slots/debug/22.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::status_object_projection_value` | 1688 | 1640 | -48 | `.temp/brief-619/stack-evidence/slots/debug/23.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::status_object_projection_value::{{closure}}` | 72 | 72 | 0 | `.temp/brief-619/stack-evidence/slots/debug/24.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call` | None | 712 | None | `.temp/brief-619/stack-evidence/slots/debug/25.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::classify_status_object_call::{{closure}}` | None | 72 | None | `.temp/brief-619/stack-evidence/slots/debug/26.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::status_object_pure_property_value` | None | 104 | None | `.temp/brief-619/stack-evidence/slots/debug/27.json` |
| debug | `rum_runtime::runtime::kernel_data::control_data::status::status_object_pure_property_value::{{closure}}` | None | 72 | None | `.temp/brief-619/stack-evidence/slots/debug/28.json` |
| debug | `rum_runtime::runtime::scoping::callable::<impl rum_runtime::runtime::model::session::Session>::try_apply_function` | 3256 | 2808 | -448 | `.temp/brief-619/stack-evidence/slots/debug/29.json` |
| debug | `rum_runtime::runtime::scoping::callable::status_application::<impl rum_runtime::runtime::model::session::Session>::apply_status_function` | None | 552 | None | `.temp/brief-619/stack-evidence/slots/debug/30.json` |
| debug | `rum_runtime::runtime::scoping::callable::status_application::<impl rum_runtime::runtime::model::session::Session>::evaluate_status_function_value` | None | 936 | None | `.temp/brief-619/stack-evidence/slots/debug/31.json` |
| debug | `rum_runtime::runtime::scoping::callable::status_application::<impl rum_runtime::runtime::model::session::Session>::suspend_status_function_projection` | None | 24 | None | `.temp/brief-619/stack-evidence/slots/debug/32.json` |
| debug | `rum_runtime::runtime::scoping::callable::status_application::<impl rum_runtime::runtime::model::session::Session>::suspend_status_function_value` | None | 24 | None | `.temp/brief-619/stack-evidence/slots/debug/33.json` |
| release | `rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::failure_property_value` | None | 296 | None | `.temp/brief-619/stack-evidence/slots/release/34.json` |
| release | `rum_runtime::runtime::kernel_data::control_data::status::<impl rum_runtime::runtime::model::session::Session>::status_object_projection_value` | 440 | None | None | `.temp/brief-619/stack-evidence/slots/release/35.json` |
| release | `rum_runtime::runtime::kernel_data::control_data::status::status_object_pure_property_value` | None | 104 | None | `.temp/brief-619/stack-evidence/slots/release/36.json` |
| release | `rum_runtime::runtime::scoping::callable::<impl rum_runtime::runtime::model::session::Session>::try_apply_function` | 1160 | 1240 | 80 | `.temp/brief-619/stack-evidence/slots/release/37.json` |
| release | `rum_runtime::runtime::scoping::callable::status_application::<impl rum_runtime::runtime::model::session::Session>::evaluate_status_function_value` | None | 696 | None | `.temp/brief-619/stack-evidence/slots/release/38.json` |
| release | `rum_runtime::runtime::scoping::callable::status_application::<impl rum_runtime::runtime::model::session::Session>::suspend_status_function_projection` | None | 72 | None | `.temp/brief-619/stack-evidence/slots/release/39.json` |
| release | `rum_runtime::runtime::scoping::callable::status_application::<impl rum_runtime::runtime::model::session::Session>::suspend_status_function_value` | None | 72 | None | `.temp/brief-619/stack-evidence/slots/release/40.json` |

`prologue_bytes` includes saved registers but not the 8-byte return address. The growing shapes reconcile exactly:

- Debug Function: `try_apply_function` 3256 -> 2808, delta -448.
- Debug status/message: caller -448 plus `apply_status_function` (552 + 8) plus `evaluate_status_function_value` (936 + 8) = **+1056 bytes/depth**.
- Release Function: `try_apply_function` 1160 -> 1240 = **+80 bytes/depth**.
- Release status/message: caller +80 plus `evaluate_status_function_value` (696 + 8; the application selection is in the caller) = **+784 bytes/depth**.
- All original 17 shapes and the LHS control retain their calibrated slopes in both profiles.

These are ordinary Rust call frames, not heap allocation of the unit continuation frames. The continuations are constructed only after a pause. The phase split retains an additional value-phase Rust frame while recursive evaluation runs. The release caller also reserves 80 more bytes on entry even on a non-status Function call.

Key concrete slot tables below use offsets from post-prologue `%rsp`; the linked full tables include every operand and store, including inlined copies.

| Function/profile | Stack slot or region | Attribution/evidence |
|---|---|---|
| `try_apply_function`, release base -> candidate | reservation `0x458` -> `0x4a8`, plus six unchanged saved registers | `0x72455a sub $0x458,%rsp` -> `0x724ada sub $0x4a8,%rsp`; +80 reservation |
| Same | producer Result at `0x1a0` -> `0x180` | `evaluate` return pointers at `0x726b57` / `0x72745d` |
| Same | finisher argument `0x3b8..0x43f` -> `0x408..0x48f` (136 B each) | stores followed by `finish_function_application` at `0x726e2a` / `0x727730` |
| Same | formatted arg-count diagnostic String `0x440..0x457` -> `0x490..0x4a7` (24 B each) | format output passed to `push_message`; base `0x728fef`, candidate `0x7298b8` |
| `apply_status_function`, debug | projection Result `0x78..0xff` (136 B) | `project_status_function_message` output pointer at `0x2aa427`; immediate witness at `0x2aa453` |
| Same | projected Expr passed to value phase `0x1c8..0x1e7` (32 B) | argument pointer `0x2aa49b`, call `0x2aa4a3` |
| `evaluate_status_function_value`, debug | timer `0x30..0x67` (56 B), producer Result `0x68..0xef` (136 B), evaluation Expr `0xf0..0x10f` (32 B) | timer call `0x2aa78c`; evaluation call `0x2aa7f2`; witness `0x2aa816` |
| Same | match outcome `0x118..0x19f`, error temporary `0x1e8..0x26f`, finisher output `0x270..0x2f7`, finisher argument `0x2f8..0x37f` (136 B each) | full explicit error moves and `finish_function_application` call `0x2aa972` in slot table debug/31 |
| Same | pause request `0x1a0..0x1bf` (32 B) | request pointer `0x2aa9ab` into cold helper `0x2aa9b3` |
| `evaluate_status_function_value`, release | `0x0..0xf` timer scalar/sentinel; `0x10..0x2f` matched Expr payload | scalar spill and payload moves at `0x7246ee`, `0x724888` |
| Same | `0x30..0xb7` (136 B), reused from the 32-B evaluation input | outgoing finisher Result; call args at `0x72499b`; evaluation input at `0x72470e` |
| Same | `0xc0..0x11f` (96 B) | error-payload spill, loads/stores `0x7248b4..0x72490c` |
| Same | `0x120..0x1a7` (136 B) | actual evaluation producer Result, output pointer `0x724706` |
| Same | `0x1a8..0x1df` (56 B) | profiling event/timer representation, reused on slow timing route |
| Same | `0x1e0..0x1ff` (32 B) | InputRequired request, `0x7249f5..0x724a24` |
| Same | `0x200..0x287` (136 B) | finisher output; overlaps earlier profiling-event spill; `0x72498b` / `0x72499b` |

The release value phase reserves `0x288` = 648 local bytes plus 48 saved-register bytes, giving 696. Its producer and finisher Result buffers remain visible, including inline vector loads/stores. The release caller's shifted result/diagnostic slots explain the measured +80 reservation; it is reported as compiler layout growth, without claiming a new runtime data field or a default-path allocation.

### New resume frame census, final candidate

All six N/2N pairs on the eight new workloads agree with the intended frame counts; aliases are grouped by actual binary address. The two other consumers have no new Function frames. Exact output:

```text
PASS g15-parameters 12 candidate profiles {'FailureMessageFromTemplateSuspensionFrame': 1, 'StatusFunctionProjectionFrame': 1}
PASS g15-template 12 candidate profiles {'FinishFailureMessageFromTemplateSuspensionFrame': 1, 'StatusFunctionProjectionFrame': 1}
PASS g15-styled 12 candidate profiles {'FailureMessageFromTemplateSuspensionFrame': 1, 'FailureVirtualStyledMessageValueSuspensionFrame': 1, 'StatusFunctionProjectionFrame': 1}
PASS g15-raw-failure 12 candidate profiles {'StatusFunctionValueFrame': 1}
PASS g15-raw-success 12 candidate profiles {'StatusFunctionValueFrame': 1}
PASS g15-projection-value 12 candidate profiles {'FailureMessageFromTemplateSuspensionFrame': 1, 'StatusFunctionProjectionFrame': 1, 'StatusFunctionValueFrame': 1}
PASS g15-enclose 12 candidate profiles {'FailureMessageFromTemplateSuspensionFrame': 1}
PASS g15-confirm 12 candidate profiles {'FailureMessageFromTemplateSuspensionFrame': 1, 'FinishConfirmFailureSuspensionFrame': 1}
PASS 96 candidate profiles; exact frame and resume counts

```

### Allocation sites and transfers reconciled

Every nonzero allocation delta is confined to the six repaired Function consumers; all intercept deltas are zero. `perf/allocation-sites/resume-g15-*.json` records each allocator callsite (binary address, caller, N and 2N calls) and the frame transfer changes. The observed sites reconcile the whole-program counts and bytes:

| Workload | Added/removed sites per iteration | Count delta | Byte delta |
|---|---|---:|---:|
| parameters, delayed template, styled | New projection unit-frame Arc 16 B + adapter Arc 24 B; real `Some` wrapper: Vec element 32 B, SharedExpr Arc 48 B, SharedArgs Arc 40 B | +5 | +160 |
| raw Failure, raw Success | Remove legacy F0528 unit Arc 16 B; add value unit Arc 16 B + adapter Arc 24 B; add the same 120-B Some wrapper | +4 | +144 |
| projection then value | Two new unit-frame/adapter pairs (80 B, 4 allocations), one final Some wrapper (120 B, 3); fixed-point history change below (+112 B, 1 net allocation) | +8 | +312 |

The two-pause history account is not queue growth. Entering the explicit value phase uses `evaluate_profiled_fixed_point` with a fresh cycle-history Vec, then freezes it into SharedArgs: +128 B for the four-Expr Vec capacity and +40 B for the SharedArgs Arc. The baseline `resume_fixed_point_from` instead appended one `FixedPointHistoryNode` (8-B previous link + 32-B Expr + 16-B Arc header), removed here: -56 B. Thus 128 + 40 - 56 = 112 B and 1 + 1 - 1 = 1 allocation. The old `resume_fixed_point_from::suspend` frame/adapter pair moves to `evaluate_profiled_fixed_point::suspend`; both wrap the same 56-B suspended state (72-B inner Arc plus 24-B adapter), so their +2/-2 calls cancel in counts and bytes. Callgrind records these exact caller substitutions. `resolution.rs:31-36,54-61,337-363,425-443` and the 32-byte Expr representation account for the sizes.

The total queue-growth cost is equal: one call per iteration moves from `insert_evaluation_return -> VecDeque::grow` to `insert_typed_erased_frame -> VecDeque::grow` (-1/+1). The extra `RawVec::grow_one` call comes from `Session::evaluate` (fixed-point cycle history), and `finish_grow` has exactly +1 allocation per iteration. The allocation totals close exactly. The ordinary and resumed N/2N checks observe the same final values and exact resume counts. New unit-frame construction is at candidate resume `0xd9f8c` / `0xda0cc` and adapter construction at `0xd9fd3` / `0xda113`; these are the two cold insertion helpers. Their witness and request are moved, with no token Arc or cloned token. The `Some` wrapper is emitted by the existing Option legacy output codec, while input Expr uses ValueMode. The driver's 136-byte typed-outcome transfer is accounted separately by the full resume proof below.

### Extra Confirm copy: exact site and inline-store comparison

`confirm-failure-message` is the only ordinary copy-count regression: memcpy slope **86 -> 87**, N=386 whole counts **34306 -> 34693**, 2N whole counts **67502 -> 68275**. Memmove stays 5 in each whole run and zero in slope. This is the existing seven-byte `"Message"` string initialization changing from inline stores to a libc copy after inlining decisions change:

- Base CLI `finish_confirm_failure`: allocation size 7 at `0x601de0`, then stores `movl $0x65676173,0x3(%rax)` at `0x601df9` and `movl $0x7373654d,(%rax)` at `0x601e00`. These initialize the same seven bytes, with one overlapping byte.
- Candidate CLI `failure_property_value`: dynamic string length carried in `%r13`, passed as `%rdx` at `0x612ef3`; `memcpy` call **`0x612ef6`**. The Confirm caller supplies length 7. The full base and candidate functions, including their remaining inline stores, are in `ordinary-callgrind/objdump/`.
- No new Result payload copy explains this regression. The library-call count increases by one even though the seven initialized string bytes were already copied inline on the baseline. The strict memcpy-count bar is nevertheless recorded as failing on this label, for the director's ruling.

Six further N/2N callgrind pairs on this label (24 valid processes) diff every function and retain every address in `ordinary-callgrind/per-label/confirm-failure-message.json`. Deterministic self-instruction deltas are:

| Function | Instructions/iteration delta | Address evidence |
|---|---:|---|
| `status_object_projection_value` | -184 | Base `0x6140c0` onward; no candidate out-of-line self work on this path |
| `failure_property_value` | +163 | Base `0x613fd0` onward; candidate `0x612e90` onward, including `0x612ef6` |
| `finish_confirm_failure` | -16 | Base `0x601d60` onward; candidate `0x601df0` onward |
| libc `__memcpy_avx_unaligned_erms` | +15 | Entry `0x1b1980` and the short-copy instructions, exact per-address rows retained |

These four deterministic totals sum to **-22 instructions/iteration**. The remaining variable symbol-table/allocator differences are also retained, rather than assigned to the status repair. No compiler-layout fix or source edit followed this evidence.

### Gate: resume-callgrind-chunk-1

Command: `python3 .temp/brief-619/run-callgrind.py resume 6`. Exit 0. Full raw output: `logs/resume-callgrind-chunk-1.log`.

```text
No diagnostics.
```

### Gate: allocation-growth

Command: `python3 .temp/brief-619/analyze-allocation-growth.py`. Exit 0. Full raw output: `logs/allocation-growth.log`.

```text
No diagnostics.
```

Allocation-growth callgraph validation (`perf/allocation-sites/projection-value-growth-paths.json`):
```text
{"caller": "alloc::raw_vec::RawVec<T,A>::grow_one", "callee": "alloc::raw_vec::finish_grow", "base": 19.02, "candidate": 20.02, "delta": 1.0}
{"caller": "rum_runtime::runtime::suspension::core::evaluation_context::<impl rum_runtime::runtime::model::session::Session>::insert_evaluation_return", "callee": "alloc::collections::vec_deque::VecDeque<T,A>::grow", "base": 1.0, "candidate": 0, "delta": -1.0}
{"caller": "rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate", "callee": "alloc::raw_vec::RawVec<T,A>::grow_one", "base": 5.0, "candidate": 6.0, "delta": 1.0}
{"caller": "rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame", "callee": "alloc::collections::vec_deque::VecDeque<T,A>::grow", "base": 4.0, "candidate": 5.0, "delta": 1.0}
```
The previous shorthand about no queue callsite changes has been corrected above: the callers change, while total queue growth does not.

### Gate: resume-callgrind-chunk-2

Command: `python3 .temp/brief-619/run-callgrind.py resume 6`. Exit 0. Full raw output: `logs/resume-callgrind-chunk-2.log`.

```text
No diagnostics.
```

### Gate: allocation-size-disassembly

Command: `python3 .temp/brief-619/dump-allocation-sizes.py`. Exit 0. Full raw output: `logs/allocation-size-disassembly.log`.

```text
No diagnostics.
```

All 2,472 resume callgrind processes are complete: 103 labels × 6 samples × 2 sizes × 2 arms. Every process exited successfully and matched its exact result and resume count. `resume-callgrind/[1-6]/*.result.json` and the corresponding command/stdout/stderr/profile files retain every run. No profiles were discarded for having unfavorable deltas.

### Gate: resume-all-function-attribution

Command: `python3 .temp/brief-619/analyze-resume.py`. Exit 0. Full raw output: `logs/resume-all-function-attribution.log`.

```text
No diagnostics.
```

### Allocation byte-size proof from final binaries

The final resume binaries confirm the requested allocator sizes and the history-vector minimum capacity. These are direct disassembly lines; full functions and commands are retained in `perf/allocation-sites/objdump/`. The callgrind allocator sites identify the indirect calls as `__rust_alloc`.

`cand-d9f60.txt`:
```text
   d9f82:	mov    $0x10,%edi
   d9f8c:	call   *0x92b67e(%rip)        # a05610 <_GLOBAL_OFFSET_TABLE_+0x2a68>
   d9fc9:	mov    $0x18,%edi
   d9fd3:	call   *0x92b637(%rip)        # a05610 <_GLOBAL_OFFSET_TABLE_+0x2a68>
```

`cand-da0a0.txt`:
```text
   da0c2:	mov    $0x10,%edi
   da0cc:	call   *0x92b53e(%rip)        # a05610 <_GLOBAL_OFFSET_TABLE_+0x2a68>
   da109:	mov    $0x18,%edi
   da113:	call   *0x92b4f7(%rip)        # a05610 <_GLOBAL_OFFSET_TABLE_+0x2a68>
```

`cand-773110.txt`:
```text
  773199:	mov    $0x20,%edi
  7731a3:	call   *0x292467(%rip)        # a05610 <_GLOBAL_OFFSET_TABLE_+0x2a68>
```

`cand-7ca2f0.txt`:
```text
  7ca324:	mov    $0x30,%edi
  7ca32e:	call   *0x23b2dc(%rip)        # a05610 <_GLOBAL_OFFSET_TABLE_+0x2a68>
```

`cand-7ca550.txt`:
```text
  7ca583:	mov    $0x28,%edi
  7ca58d:	call   *0x23b07d(%rip)        # a05610 <_GLOBAL_OFFSET_TABLE_+0x2a68>
```

`base-715cd0.txt`:
```text
  715f88:	mov    $0x38,%edi
  715f92:	call   *0x2ee698(%rip)        # a04630 <_GLOBAL_OFFSET_TABLE_+0x2a58>
```

`base-114db0.txt`:
```text
  114e4c:	mov    $0x48,%edi
  114e56:	call   *0x8ef7d4(%rip)        # a04630 <_GLOBAL_OFFSET_TABLE_+0x2a58>
  114eaf:	mov    $0x18,%edi
  114eb9:	call   *0x8ef771(%rip)        # a04630 <_GLOBAL_OFFSET_TABLE_+0x2a58>
```

`cand-4a79f0.txt`:
```text
  4a7a1f:	mov    $0x4,%r15d
  4a7a34:	shl    $0x5,%rdx
  4a7a79:	call   adb40 <alloc::raw_vec::finish_grow>
```

The candidate history grow call at `Session::evaluate:0x7140c6` targets GOT `0xa06268`, whose stored address is `0x4a79f0`. That specialization selects a minimum capacity of 4 (`0x4a7a1f`) and multiplies by 32 (`0x4a7a34`), proving the 128-byte allocation. This agrees with the measured +312 bytes for the two-pause case after the other identified allocations cancel.

## Complete resume attribution

All 103 workloads have six N/2N samples per arm (2,472 successful processes). Every function was compared in every process; the stored nonzero function deltas sum back to each whole-profile delta in all six samples. `resume-callgrind/per-label/*.json` retains every contributing instruction address and all six raw N/2N differences, including variable results. No deterministic delta is filtered by size. `resume-callgrind/function-deltas.json` includes every address; `deterministic-deltas.json` selects the reproducible function deltas without discarding the full evidence.

Older workloads with a nonzero deterministic function total: resume-fold, resume-s7-assoc-part, resume-s7-assoc-call, resume-s7-held-rhs, resume-s7-associate-argument, resume-s7-keydrop-held, resume-s9-assert-handler, resume-s9-catchexceptions-guard, resume-s9-catchexceptions-match, resume-s9-catchexceptions-suspended-match, resume-s9-catchexceptions-suspended-guard, resume-s9-catchexceptions-rhs. Variable totals remain recorded; they are not replaced by zero.

| Workload | Base instructions/iteration (mean) | Candidate (mean) | Difference | Deterministic function deltas | Additional driver 136-B moves/iteration |
|---|---:|---:|---:|---:|---:|
| `resume-table` | 20015.716833 | 20037.529833 | +21.813000 | 0 | 0 |
| `resume-do-chain` | 19138.700333 | 19137.227500 | -1.472833 | 0 | 0 |
| `resume-call-repeat` | 29239.990833 | 29222.040333 | -17.950500 | 0 | 0 |
| `resume-cleanup` | 18592.714167 | 18602.803000 | +10.088833 | 0 | 0 |
| `resume-fold` | 29516.302333 | 29518.325333 | +2.023000 | 1 | 0 |
| `resume-argument-twice` | 92553.925167 | 92533.863500 | -20.061667 | 0 | 0 |
| `resume-rewrite-exprs` | 53386.119333 | 53444.441000 | +58.321667 | 0 | 0 |
| `resume-rewrite-compiled` | 41725.009500 | 41765.964167 | +40.954667 | 0 | 0 |
| `resume-cases-spec` | 69293.502333 | 69363.535667 | +70.033333 | 0 | 0 |
| `resume-cases-traversal` | 67380.717167 | 67473.747167 | +93.030000 | 0 | 0 |
| `resume-position-traversal` | 70870.814000 | 70784.688833 | -86.125167 | 0 | 0 |
| `resume-count-traversal` | 61717.849500 | 61739.075833 | +21.226333 | 0 | 0 |
| `resume-cases-shared` | 113726.996500 | 113773.710500 | +46.714000 | 0 | 0 |
| `resume-s5-down-rhs` | 59487.450000 | 59516.285000 | +28.835000 | 0 | 0 |
| `resume-s5-down-guard` | 114225.305000 | 114407.273333 | +181.968333 | 0 | 0 |
| `resume-s5-sub-rhs` | 65144.766667 | 65171.400000 | +26.633333 | 0 | 0 |
| `resume-s5-sub-guard` | 122245.700000 | 122170.105000 | -75.595000 | 0 | 0 |
| `resume-s5-up-rhs` | 64577.786667 | 64596.410000 | +18.623333 | 0 | 0 |
| `resume-s5-up-guard` | 137952.500000 | 138006.111667 | +53.611667 | 0 | 0 |
| `resume-s5-default-rhs` | 109197.655000 | 109087.705000 | -109.950000 | 0 | 0 |
| `resume-s5-default-guard` | 103543.025000 | 103397.835000 | -145.190000 | 0 | 0 |
| `resume-s5-default-fallback-rhs` | 64378.378333 | 64422.355000 | +43.976667 | 0 | 0 |
| `resume-s5-set` | 56336.266667 | 56422.645000 | +86.378333 | 0 | 0 |
| `resume-s5-upset` | 67234.915000 | 67291.118333 | +56.203333 | 0 | 0 |
| `resume-s5-tagset` | 67110.600000 | 67085.925000 | -24.675000 | 0 | 0 |
| `resume-s5-rule-rhs` | 71414.665000 | 71459.993333 | +45.328333 | 0 | 0 |
| `resume-s5-rule-lhs` | 90325.738333 | 90354.083333 | +28.345000 | 0 | 0 |
| `resume-s5-destructure` | 76142.945000 | 76129.083333 | -13.861667 | 0 | 0 |
| `resume-s5-lhs-twice` | 97931.180000 | 97902.885000 | -28.295000 | 0 | 0 |
| `resume-s5-own-rhs` | 51597.655000 | 51630.590000 | +32.935000 | 0 | 0 |
| `resume-s5-destructure-twice` | 119005.273333 | 118982.693333 | -22.580000 | 0 | 0 |
| `resume-s5-rule-lhs-twice` | 122193.046667 | 122254.338333 | +61.291667 | 0 | 0 |
| `resume-s6-rhs-single` | 74512.408333 | 74569.533333 | +57.125000 | 0 | 0 |
| `resume-s6-rhs-twice` | 105753.006667 | 105941.625000 | +188.618333 | 0 | 0 |
| `resume-s6-cross-type` | 125074.655000 | 125119.590000 | +44.935000 | 0 | 0 |
| `resume-s6-readonly-twice` | 215848.203333 | 215776.401667 | -71.801667 | 0 | 0 |
| `resume-s6-string` | 85033.103333 | 85086.808333 | +53.705000 | 0 | 0 |
| `resume-s7-list-part` | 83910.871667 | 83887.635000 | -23.236667 | 0 | 0 |
| `resume-s7-assoc-part` | 85686.456667 | 85571.655000 | -114.801667 | 2 | 0 |
| `resume-s7-assoc-call` | 75846.908333 | 75911.878333 | +64.970000 | 2 | 0 |
| `resume-s7-assoc-call-thrice` | 159780.976667 | 159842.546667 | +61.570000 | 0 | 0 |
| `resume-s7-held-twice` | 126158.748333 | 126259.381667 | +100.633333 | 0 | 0 |
| `resume-s7-held-rhs` | 96019.776667 | 96060.980000 | +41.203333 | 1 | 0 |
| `resume-s7-associate-argument` | 79114.411667 | 79111.673333 | -2.738333 | 2 | 0 |
| `resume-s7-keydrop-argument` | 76014.626667 | 76025.211667 | +10.585000 | 0 | 0 |
| `resume-s7-associate-target` | 265312.643333 | 265274.488333 | -38.155000 | 0 | 0 |
| `resume-s7-keydrop-target` | 264372.956667 | 264400.911667 | +27.955000 | 0 | 0 |
| `resume-s7-associate-held` | 100356.026667 | 100154.170000 | -201.856667 | 0 | 0 |
| `resume-s7-keydrop-held` | 107629.120000 | 107497.398333 | -131.721667 | 2 | 0 |
| `resume-s8-replaceall-node` | 108816.411667 | 109044.726667 | +228.315000 | 0 | 0 |
| `resume-s8-replaceall-atom` | 96386.436667 | 96286.820000 | -99.616667 | 0 | 0 |
| `resume-s8-replaceall-twice` | 164413.768333 | 164570.173333 | +156.405000 | 0 | 0 |
| `resume-s8-repeated-guard` | 103655.871667 | 103775.983333 | +120.111667 | 0 | 0 |
| `resume-s8-replace-decline` | 112326.128333 | 112376.671667 | +50.543333 | 0 | 0 |
| `resume-s8-replace-both` | 157594.351667 | 157811.878333 | +217.526667 | 0 | 0 |
| `resume-s8-replaceat-guard` | 109458.971667 | 109497.610000 | +38.638333 | 0 | 0 |
| `resume-s8-count-target` | 62896.918333 | 62942.250000 | +45.331667 | 0 | 0 |
| `resume-s8-count-pattern` | 61626.578333 | 61664.165000 | +37.586667 | 0 | 0 |
| `resume-s8-memberq-target` | 63809.476667 | 63815.536667 | +6.060000 | 0 | 0 |
| `resume-s8-memberq-pattern` | 62504.313333 | 62448.241667 | -56.071667 | 0 | 0 |
| `resume-s8-freeq-target` | 65157.090000 | 65131.453333 | -25.636667 | 0 | 0 |
| `resume-s8-freeq-pattern` | 63862.146667 | 63789.343333 | -72.803333 | 0 | 0 |
| `resume-s8-count-level` | 78612.528333 | 78661.091667 | +48.563333 | 0 | 0 |
| `resume-s8-count-guard-twice` | 130387.343333 | 130420.795000 | +33.451667 | 0 | 0 |
| `resume-s8-position-target` | 73088.825000 | 73112.045000 | +23.220000 | 0 | 0 |
| `resume-s8-position-pattern` | 74388.088333 | 74313.476667 | -74.611667 | 0 | 0 |
| `resume-s8-position-level` | 87249.745000 | 87316.166667 | +66.421667 | 0 | 0 |
| `resume-s8-position-guard-twice` | 157283.336667 | 157279.343333 | -3.993333 | 0 | 0 |
| `resume-s9-checkabort-option` | 64586.596667 | 64607.701667 | +21.105000 | 0 | 0 |
| `resume-s9-checkabort-primary` | 63160.355000 | 63165.981667 | +5.626667 | 0 | 0 |
| `resume-s9-checkabort-abort` | 64773.973333 | 64732.453333 | -41.520000 | 0 | 0 |
| `resume-s9-abortprotect-body` | 56276.906667 | 56434.883333 | +157.976667 | 0 | 0 |
| `resume-s9-checkabort-pending-fallback` | 81512.778333 | 81557.846667 | +45.068333 | 0 | 0 |
| `resume-s9-checkabort-fallback` | 66324.700000 | 66377.331667 | +52.631667 | 0 | 0 |
| `resume-s9-assert-function-lookup` | 65978.650000 | 66027.301667 | +48.651667 | 0 | 0 |
| `resume-s9-assert-test` | 60798.116667 | 60808.571667 | +10.455000 | 0 | 0 |
| `resume-s9-assert-handler` | 65591.751667 | 65540.851667 | -50.900000 | 1 | 0 |
| `resume-s9-check-body` | 56262.570000 | 56250.033333 | -12.536667 | 0 | 0 |
| `resume-s9-quiet-body` | 66372.256667 | 66366.326667 | -5.930000 | 0 | 0 |
| `resume-s9-quiet-check-fallback` | 85807.073333 | 85812.706667 | +5.633333 | 0 | 0 |
| `resume-s9-cleanup-init` | 57509.185000 | 57508.951667 | -0.233333 | 0 | 0 |
| `resume-s9-cleanup-primary` | 56925.990000 | 56883.685000 | -42.305000 | 0 | 0 |
| `resume-s9-cleanup-finalizer` | 57319.558333 | 57376.918333 | +57.360000 | 0 | 0 |
| `resume-s9-cleanup-three-phase` | 115976.275000 | 115872.186667 | -104.088333 | 0 | 0 |
| `resume-s9-catchexceptions-body` | 85964.140000 | 85973.035000 | +8.895000 | 0 | 0 |
| `resume-s9-catchexceptions-guard` | 88902.568333 | 88886.748333 | -15.820000 | 2 | 0 |
| `resume-s9-catchexceptions-match` | 94023.818333 | 94110.103333 | +86.285000 | 2 | 0 |
| `resume-s9-catchexceptions-suspended-match` | 121255.110000 | 121386.431667 | +131.321667 | 2 | 0 |
| `resume-s9-catchexceptions-suspended-guard` | 116101.016667 | 116020.135000 | -80.881667 | 2 | 0 |
| `resume-s9-catchexceptions-rhs` | 82077.275000 | 82100.426667 | +23.151667 | 2 | 0 |
| `resume-s9-catch-body` | 58242.145000 | 58283.083333 | +40.938333 | 0 | 0 |
| `resume-s9-catch-form` | 63283.983333 | 63222.775000 | -61.208333 | 0 | 0 |
| `resume-s9-catch-form-guard` | 91435.530000 | 91502.895000 | +67.365000 | 0 | 0 |
| `resume-s9-catch-bypassing-form` | 80555.108333 | 80597.700000 | +42.591667 | 0 | 0 |
| `resume-s9-catch-bypassing-guard` | 102074.208333 | 102120.993333 | +46.785000 | 0 | 0 |
| `resume-g15-parameters` | 69651.738333 | 72429.630000 | +2777.891667 | 52 | 1 |
| `resume-g15-template` | 69255.398333 | 72045.016667 | +2789.618333 | 42 | 1 |
| `resume-g15-styled` | 80131.338333 | 82990.836667 | +2859.498333 | 55 | 1 |
| `resume-g15-raw-failure` | 60137.065000 | 61969.105000 | +1832.040000 | 46 | 1 |
| `resume-g15-raw-success` | 56820.771667 | 58759.673333 | +1938.901667 | 46 | 1 |
| `resume-g15-projection-value` | 93255.113333 | 97075.720000 | +3820.606667 | 66 | 2 |
| `resume-g15-enclose` | 73231.065000 | 73287.635000 | +56.570000 | 2 | 0 |
| `resume-g15-confirm` | 119504.041667 | 119225.266667 | -278.775000 | 4 | 0 |

The repaired consumers now perform the missing evaluation/finish/Some work and the two typed-frame transfers. The reproducible deltas name the phase helper, frame adapters, driver, Option output codec, interning/hash routines, allocation/free sites and fixed-point machinery; every function and address is in the per-label evidence. The Enclose and Confirm controls add no G15 frame. Their projection/helper inlining changes remain visible separately. For per-required-input costs divide by the exact resume factor in `comparison.json` (two for `g15-projection-value`, one for the other seven new workloads).

Driver attribution: {"base": {"matching_site_sets": [["0x74a33d", "0x74a35d"]], "unique": true}, "candidate": {"matching_site_sets": [["0x74b34d", "0x74b36d"]], "unique": true}}. Other 136-byte site differences: []. The matched sites equal the typed-adapter resume count on every workload and every sample. These 136-byte typed-outcome moves are recorded as R's cost; the driver source is unchanged. Full proof: `resume-callgrind/driver-136.json`.

Verbatim analysis output:
```text
136 sites base 12 candidate 12 all memcpy base 22 candidate 22
pending 95 of 103
LABEL resume-table N=1000 functions=23 det=0 adapter_base=[3000, 3000, 3000, 3000, 3000, 3000] adapter_cand=[3000, 3000, 3000, 3000, 3000, 3000] 3.3s
LABEL resume-do-chain N=1000 functions=7 det=0 adapter_base=[3000, 3000, 3000, 3000, 3000, 3000] adapter_cand=[3000, 3000, 3000, 3000, 3000, 3000] 3.3s
LABEL resume-call-repeat N=1000 functions=7 det=0 adapter_base=[4000, 4000, 4000, 4000, 4000, 4000] adapter_cand=[4000, 4000, 4000, 4000, 4000, 4000] 3.3s
LABEL resume-cleanup N=1000 functions=8 det=0 adapter_base=[3000, 3000, 3000, 3000, 3000, 3000] adapter_cand=[3000, 3000, 3000, 3000, 3000, 3000] 3.3s
LABEL resume-fold N=1000 functions=10 det=1 adapter_base=[5000, 5000, 5000, 5000, 5000, 5000] adapter_cand=[5000, 5000, 5000, 5000, 5000, 5000] 3.5s
LABEL resume-argument-twice N=1000 functions=24 det=0 adapter_base=[19000, 19000, 19000, 19000, 19000, 19000] adapter_cand=[19000, 19000, 19000, 19000, 19000, 19000] 3.8s
LABEL resume-rewrite-exprs N=1000 functions=8 det=0 adapter_base=[11000, 11000, 11000, 11000, 11000, 11000] adapter_cand=[11000, 11000, 11000, 11000, 11000, 11000] 3.6s
LABEL resume-rewrite-compiled N=1000 functions=8 det=0 adapter_base=[10000, 10000, 10000, 10000, 10000, 10000] adapter_cand=[10000, 10000, 10000, 10000, 10000, 10000] 3.6s
LABEL resume-cases-spec N=1000 functions=8 det=0 adapter_base=[14000, 14000, 14000, 14000, 14000, 14000] adapter_cand=[14000, 14000, 14000, 14000, 14000, 14000] 3.7s
LABEL resume-cases-traversal N=1000 functions=8 det=0 adapter_base=[11000, 11000, 11000, 11000, 11000, 11000] adapter_cand=[11000, 11000, 11000, 11000, 11000, 11000] 3.6s
LABEL resume-position-traversal N=1000 functions=24 det=0 adapter_base=[12000, 12000, 12000, 12000, 12000, 12000] adapter_cand=[12000, 12000, 12000, 12000, 12000, 12000] 3.6s
LABEL resume-count-traversal N=1000 functions=9 det=0 adapter_base=[12000, 12000, 12000, 12000, 12000, 12000] adapter_cand=[12000, 12000, 12000, 12000, 12000, 12000] 3.5s
LABEL resume-cases-shared N=1000 functions=23 det=0 adapter_base=[22000, 22000, 22000, 22000, 22000, 22000] adapter_cand=[22000, 22000, 22000, 22000, 22000, 22000] 3.7s
LABEL resume-s5-down-rhs N=100 functions=8 det=0 adapter_base=[1200, 1200, 1200, 1200, 1200, 1200] adapter_cand=[1200, 1200, 1200, 1200, 1200, 1200] 3.9s
LABEL resume-s5-down-guard N=100 functions=25 det=0 adapter_base=[2000, 2000, 2000, 2000, 2000, 2000] adapter_cand=[2000, 2000, 2000, 2000, 2000, 2000] 4.4s
LABEL resume-s5-sub-rhs N=100 functions=9 det=0 adapter_base=[1200, 1200, 1200, 1200, 1200, 1200] adapter_cand=[1200, 1200, 1200, 1200, 1200, 1200] 4.1s
LABEL resume-s5-sub-guard N=100 functions=24 det=0 adapter_base=[2000, 2000, 2000, 2000, 2000, 2000] adapter_cand=[2000, 2000, 2000, 2000, 2000, 2000] 4.5s
LABEL resume-s5-up-rhs N=100 functions=23 det=0 adapter_base=[1200, 1200, 1200, 1200, 1200, 1200] adapter_cand=[1200, 1200, 1200, 1200, 1200, 1200] 4.1s
LABEL resume-s5-up-guard N=100 functions=7 det=0 adapter_base=[2000, 2000, 2000, 2000, 2000, 2000] adapter_cand=[2000, 2000, 2000, 2000, 2000, 2000] 4.5s
LABEL resume-s5-default-rhs N=100 functions=9 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 4.5s
LABEL resume-s5-default-guard N=100 functions=23 det=0 adapter_base=[2100, 2100, 2100, 2100, 2100, 2100] adapter_cand=[2100, 2100, 2100, 2100, 2100, 2100] 4.0s
LABEL resume-s5-default-fallback-rhs N=100 functions=9 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.9s
LABEL resume-s5-set N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.6s
LABEL resume-s5-upset N=100 functions=9 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.8s
LABEL resume-s5-tagset N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.9s
LABEL resume-s5-rule-rhs N=100 functions=9 det=0 adapter_base=[1600, 1600, 1600, 1600, 1600, 1600] adapter_cand=[1600, 1600, 1600, 1600, 1600, 1600] 3.8s
LABEL resume-s5-rule-lhs N=100 functions=9 det=0 adapter_base=[1700, 1700, 1700, 1700, 1700, 1700] adapter_cand=[1700, 1700, 1700, 1700, 1700, 1700] 4.0s
LABEL resume-s5-destructure N=100 functions=9 det=0 adapter_base=[1600, 1600, 1600, 1600, 1600, 1600] adapter_cand=[1600, 1600, 1600, 1600, 1600, 1600] 4.0s
LABEL resume-s5-lhs-twice N=100 functions=8 det=0 adapter_base=[2000, 2000, 2000, 2000, 2000, 2000] adapter_cand=[2000, 2000, 2000, 2000, 2000, 2000] 3.9s
LABEL resume-s5-own-rhs N=100 functions=8 det=0 adapter_base=[1100, 1100, 1100, 1100, 1100, 1100] adapter_cand=[1100, 1100, 1100, 1100, 1100, 1100] 3.8s
LABEL resume-s5-destructure-twice N=100 functions=9 det=0 adapter_base=[2600, 2600, 2600, 2600, 2600, 2600] adapter_cand=[2600, 2600, 2600, 2600, 2600, 2600] 4.1s
LABEL resume-s5-rule-lhs-twice N=100 functions=9 det=0 adapter_base=[2400, 2400, 2400, 2400, 2400, 2400] adapter_cand=[2400, 2400, 2400, 2400, 2400, 2400] 4.0s
LABEL resume-s6-rhs-single N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 4.3s
LABEL resume-s6-rhs-twice N=100 functions=23 det=0 adapter_base=[2000, 2000, 2000, 2000, 2000, 2000] adapter_cand=[2000, 2000, 2000, 2000, 2000, 2000] 4.4s
LABEL resume-s6-cross-type N=100 functions=24 det=0 adapter_base=[2100, 2100, 2100, 2100, 2100, 2100] adapter_cand=[2100, 2100, 2100, 2100, 2100, 2100] 4.4s
LABEL resume-s6-readonly-twice N=100 functions=9 det=0 adapter_base=[3400, 3400, 3400, 3400, 3400, 3400] adapter_cand=[3400, 3400, 3400, 3400, 3400, 3400] 4.5s
LABEL resume-s6-string N=100 functions=8 det=0 adapter_base=[1400, 1400, 1400, 1400, 1400, 1400] adapter_cand=[1400, 1400, 1400, 1400, 1400, 1400] 4.2s
LABEL resume-s7-list-part N=100 functions=24 det=0 adapter_base=[1500, 1500, 1500, 1500, 1500, 1500] adapter_cand=[1500, 1500, 1500, 1500, 1500, 1500] 4.0s
LABEL resume-s7-assoc-part N=100 functions=12 det=2 adapter_base=[1500, 1500, 1500, 1500, 1500, 1500] adapter_cand=[1500, 1500, 1500, 1500, 1500, 1500] 4.0s
LABEL resume-s7-assoc-call N=100 functions=13 det=2 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 4.3s
LABEL resume-s7-assoc-call-thrice N=100 functions=10 det=0 adapter_base=[2700, 2700, 2700, 2700, 2700, 2700] adapter_cand=[2700, 2700, 2700, 2700, 2700, 2700] 4.0s
LABEL resume-s7-held-twice N=100 functions=9 det=0 adapter_base=[2200, 2200, 2200, 2200, 2200, 2200] adapter_cand=[2200, 2200, 2200, 2200, 2200, 2200] 4.1s
LABEL resume-s7-held-rhs N=100 functions=25 det=1 adapter_base=[1500, 1500, 1500, 1500, 1500, 1500] adapter_cand=[1500, 1500, 1500, 1500, 1500, 1500] 4.2s
LABEL resume-s7-associate-argument N=100 functions=27 det=2 adapter_base=[1200, 1200, 1200, 1200, 1200, 1200] adapter_cand=[1200, 1200, 1200, 1200, 1200, 1200] 4.0s
LABEL resume-s7-keydrop-argument N=100 functions=10 det=0 adapter_base=[1200, 1200, 1200, 1200, 1200, 1200] adapter_cand=[1200, 1200, 1200, 1200, 1200, 1200] 4.0s
LABEL resume-s7-associate-target N=100 functions=27 det=0 adapter_base=[1800, 1800, 1800, 1800, 1800, 1800] adapter_cand=[1800, 1800, 1800, 1800, 1800, 1800] 4.5s
LABEL resume-s7-keydrop-target N=100 functions=11 det=0 adapter_base=[1800, 1800, 1800, 1800, 1800, 1800] adapter_cand=[1800, 1800, 1800, 1800, 1800, 1800] 4.5s
LABEL resume-s7-associate-held N=100 functions=25 det=0 adapter_base=[1500, 1500, 1500, 1500, 1500, 1500] adapter_cand=[1500, 1500, 1500, 1500, 1500, 1500] 4.0s
LABEL resume-s7-keydrop-held N=100 functions=12 det=2 adapter_base=[1500, 1500, 1500, 1500, 1500, 1500] adapter_cand=[1500, 1500, 1500, 1500, 1500, 1500] 4.3s
LABEL resume-s8-replaceall-node N=100 functions=8 det=0 adapter_base=[1600, 1600, 1600, 1600, 1600, 1600] adapter_cand=[1600, 1600, 1600, 1600, 1600, 1600] 4.2s
LABEL resume-s8-replaceall-atom N=100 functions=9 det=0 adapter_base=[1600, 1600, 1600, 1600, 1600, 1600] adapter_cand=[1600, 1600, 1600, 1600, 1600, 1600] 4.1s
LABEL resume-s8-replaceall-twice N=100 functions=8 det=0 adapter_base=[2500, 2500, 2500, 2500, 2500, 2500] adapter_cand=[2500, 2500, 2500, 2500, 2500, 2500] 4.5s
LABEL resume-s8-repeated-guard N=100 functions=24 det=0 adapter_base=[1600, 1600, 1600, 1600, 1600, 1600] adapter_cand=[1600, 1600, 1600, 1600, 1600, 1600] 4.3s
LABEL resume-s8-replace-decline N=100 functions=8 det=0 adapter_base=[1400, 1400, 1400, 1400, 1400, 1400] adapter_cand=[1400, 1400, 1400, 1400, 1400, 1400] 4.3s
LABEL resume-s8-replace-both N=100 functions=8 det=0 adapter_base=[2200, 2200, 2200, 2200, 2200, 2200] adapter_cand=[2200, 2200, 2200, 2200, 2200, 2200] 4.3s
LABEL resume-s8-replaceat-guard N=100 functions=24 det=0 adapter_base=[1400, 1400, 1400, 1400, 1400, 1400] adapter_cand=[1400, 1400, 1400, 1400, 1400, 1400] 4.4s
LABEL resume-s8-count-target N=100 functions=24 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.9s
LABEL resume-s8-count-pattern N=100 functions=23 det=0 adapter_base=[1000, 1000, 1000, 1000, 1000, 1000] adapter_cand=[1000, 1000, 1000, 1000, 1000, 1000] 4.0s
LABEL resume-s8-memberq-target N=100 functions=9 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.7s
LABEL resume-s8-memberq-pattern N=100 functions=8 det=0 adapter_base=[1000, 1000, 1000, 1000, 1000, 1000] adapter_cand=[1000, 1000, 1000, 1000, 1000, 1000] 3.9s
LABEL resume-s8-freeq-target N=100 functions=23 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.9s
LABEL resume-s8-freeq-pattern N=100 functions=8 det=0 adapter_base=[1000, 1000, 1000, 1000, 1000, 1000] adapter_cand=[1000, 1000, 1000, 1000, 1000, 1000] 4.0s
LABEL resume-s8-count-level N=100 functions=9 det=0 adapter_base=[1500, 1500, 1500, 1500, 1500, 1500] adapter_cand=[1500, 1500, 1500, 1500, 1500, 1500] 3.9s
LABEL resume-s8-count-guard-twice N=100 functions=23 det=0 adapter_base=[2300, 2300, 2300, 2300, 2300, 2300] adapter_cand=[2300, 2300, 2300, 2300, 2300, 2300] 4.1s
LABEL resume-s8-position-target N=100 functions=24 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.7s
LABEL resume-s8-position-pattern N=100 functions=24 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 4.0s
LABEL resume-s8-position-level N=100 functions=9 det=0 adapter_base=[1500, 1500, 1500, 1500, 1500, 1500] adapter_cand=[1500, 1500, 1500, 1500, 1500, 1500] 3.8s
LABEL resume-s8-position-guard-twice N=100 functions=9 det=0 adapter_base=[2300, 2300, 2300, 2300, 2300, 2300] adapter_cand=[2300, 2300, 2300, 2300, 2300, 2300] 4.1s
LABEL resume-s9-checkabort-option N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.6s
LABEL resume-s9-checkabort-primary N=100 functions=23 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.7s
LABEL resume-s9-checkabort-abort N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.9s
LABEL resume-s9-abortprotect-body N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.6s
LABEL resume-s9-checkabort-pending-fallback N=100 functions=23 det=0 adapter_base=[1700, 1700, 1700, 1700, 1700, 1700] adapter_cand=[1700, 1700, 1700, 1700, 1700, 1700] 3.7s
LABEL resume-s9-checkabort-fallback N=100 functions=23 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.6s
LABEL resume-s9-assert-function-lookup N=100 functions=8 det=0 adapter_base=[1500, 1500, 1500, 1500, 1500, 1500] adapter_cand=[1500, 1500, 1500, 1500, 1500, 1500] 3.7s
LABEL resume-s9-assert-test N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.6s
LABEL resume-s9-assert-handler N=100 functions=25 det=1 adapter_base=[1200, 1200, 1200, 1200, 1200, 1200] adapter_cand=[1200, 1200, 1200, 1200, 1200, 1200] 3.8s
LABEL resume-s9-check-body N=100 functions=23 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.6s
LABEL resume-s9-quiet-body N=100 functions=9 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.9s
LABEL resume-s9-quiet-check-fallback N=100 functions=24 det=0 adapter_base=[1700, 1700, 1700, 1700, 1700, 1700] adapter_cand=[1700, 1700, 1700, 1700, 1700, 1700] 3.8s
LABEL resume-s9-cleanup-init N=100 functions=23 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.8s
LABEL resume-s9-cleanup-primary N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.7s
LABEL resume-s9-cleanup-finalizer N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.7s
LABEL resume-s9-cleanup-three-phase N=100 functions=8 det=0 adapter_base=[2700, 2700, 2700, 2700, 2700, 2700] adapter_cand=[2700, 2700, 2700, 2700, 2700, 2700] 3.5s
LABEL resume-s9-catchexceptions-body N=100 functions=9 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.7s
LABEL resume-s9-catchexceptions-guard N=100 functions=10 det=2 adapter_base=[1400, 1400, 1400, 1400, 1400, 1400] adapter_cand=[1400, 1400, 1400, 1400, 1400, 1400] 4.5s
LABEL resume-s9-catchexceptions-match N=100 functions=10 det=2 adapter_base=[1400, 1400, 1400, 1400, 1400, 1400] adapter_cand=[1400, 1400, 1400, 1400, 1400, 1400] 4.1s
LABEL resume-s9-catchexceptions-suspended-match N=100 functions=26 det=2 adapter_base=[2100, 2100, 2100, 2100, 2100, 2100] adapter_cand=[2100, 2100, 2100, 2100, 2100, 2100] 4.1s
LABEL resume-s9-catchexceptions-suspended-guard N=100 functions=26 det=2 adapter_base=[2100, 2100, 2100, 2100, 2100, 2100] adapter_cand=[2100, 2100, 2100, 2100, 2100, 2100] 4.1s
LABEL resume-s9-catchexceptions-rhs N=100 functions=26 det=2 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 4.1s
LABEL resume-s9-catch-body N=100 functions=7 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.7s
LABEL resume-s9-catch-form N=100 functions=8 det=0 adapter_base=[1300, 1300, 1300, 1300, 1300, 1300] adapter_cand=[1300, 1300, 1300, 1300, 1300, 1300] 3.8s
LABEL resume-s9-catch-form-guard N=100 functions=9 det=0 adapter_base=[1400, 1400, 1400, 1400, 1400, 1400] adapter_cand=[1400, 1400, 1400, 1400, 1400, 1400] 4.4s
LABEL resume-s9-catch-bypassing-form N=100 functions=24 det=0 adapter_base=[1700, 1700, 1700, 1700, 1700, 1700] adapter_cand=[1700, 1700, 1700, 1700, 1700, 1700] 3.7s
LABEL resume-s9-catch-bypassing-guard N=100 functions=9 det=0 adapter_base=[1800, 1800, 1800, 1800, 1800, 1800] adapter_cand=[1800, 1800, 1800, 1800, 1800, 1800] 4.0s
base 136-B site sets [['0x74a33d', '0x74a35d']]
candidate 136-B site sets [['0x74b34d', '0x74b36d']]
resume-table new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [2000, 2000, 2000, 2000, 2000, 2000] remainder_cand [2000, 2000, 2000, 2000, 2000, 2000]
resume-do-chain new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [2000, 2000, 2000, 2000, 2000, 2000] remainder_cand [2000, 2000, 2000, 2000, 2000, 2000]
resume-call-repeat new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [2000, 2000, 2000, 2000, 2000, 2000] remainder_cand [2000, 2000, 2000, 2000, 2000, 2000]
resume-cleanup new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [2000, 2000, 2000, 2000, 2000, 2000] remainder_cand [2000, 2000, 2000, 2000, 2000, 2000]
resume-fold new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [3000, 3000, 3000, 3000, 3000, 3000] remainder_cand [3000, 3000, 3000, 3000, 3000, 3000]
resume-argument-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [7000, 7000, 7000, 7000, 7000, 7000] remainder_cand [7000, 7000, 7000, 7000, 7000, 7000]
resume-rewrite-exprs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [4000, 4000, 4000, 4000, 4000, 4000] remainder_cand [4000, 4000, 4000, 4000, 4000, 4000]
resume-rewrite-compiled new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [4000, 4000, 4000, 4000, 4000, 4000] remainder_cand [4000, 4000, 4000, 4000, 4000, 4000]
resume-cases-spec new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [5000, 5000, 5000, 5000, 5000, 5000] remainder_cand [5000, 5000, 5000, 5000, 5000, 5000]
resume-cases-traversal new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [4000, 4000, 4000, 4000, 4000, 4000] remainder_cand [4000, 4000, 4000, 4000, 4000, 4000]
resume-position-traversal new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [4000, 4000, 4000, 4000, 4000, 4000] remainder_cand [4000, 4000, 4000, 4000, 4000, 4000]
resume-count-traversal new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [4000, 4000, 4000, 4000, 4000, 4000] remainder_cand [4000, 4000, 4000, 4000, 4000, 4000]
resume-cases-shared new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [8000, 8000, 8000, 8000, 8000, 8000] remainder_cand [8000, 8000, 8000, 8000, 8000, 8000]
resume-s5-down-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-down-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s5-sub-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-sub-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s5-up-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-up-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s5-default-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-default-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s5-default-fallback-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-set new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-upset new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-tagset new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-rule-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s5-rule-lhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s5-destructure new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-lhs-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s5-own-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s5-destructure-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s5-rule-lhs-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [900, 900, 900, 900, 900, 900] remainder_cand [900, 900, 900, 900, 900, 900]
resume-s6-rhs-single new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s6-rhs-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s6-cross-type new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s6-readonly-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [1400, 1400, 1400, 1400, 1400, 1400] remainder_cand [1400, 1400, 1400, 1400, 1400, 1400]
resume-s6-string new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s7-list-part new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s7-assoc-part new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s7-assoc-call new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s7-assoc-call-thrice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [1100, 1100, 1100, 1100, 1100, 1100] remainder_cand [1100, 1100, 1100, 1100, 1100, 1100]
resume-s7-held-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s7-held-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s7-associate-argument new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s7-keydrop-argument new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s7-associate-target new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s7-keydrop-target new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s7-associate-held new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s7-keydrop-held new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-replaceall-node new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-replaceall-atom new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-replaceall-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s8-repeated-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-replace-decline new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-replace-both new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s8-replaceat-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-count-target new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-count-pattern new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [400, 400, 400, 400, 400, 400] remainder_cand [400, 400, 400, 400, 400, 400]
resume-s8-memberq-target new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-memberq-pattern new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [400, 400, 400, 400, 400, 400] remainder_cand [400, 400, 400, 400, 400, 400]
resume-s8-freeq-target new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-freeq-pattern new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [400, 400, 400, 400, 400, 400] remainder_cand [400, 400, 400, 400, 400, 400]
resume-s8-count-level new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-count-guard-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s8-position-target new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-position-pattern new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-position-level new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s8-position-guard-twice new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s9-checkabort-option new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-checkabort-primary new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-checkabort-abort new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-abortprotect-body new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-checkabort-pending-fallback new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s9-checkabort-fallback new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-assert-function-lookup new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s9-assert-test new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-assert-handler new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-check-body new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-quiet-body new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-quiet-check-fallback new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s9-cleanup-init new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-cleanup-primary new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-cleanup-finalizer new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-cleanup-three-phase new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [1100, 1100, 1100, 1100, 1100, 1100] remainder_cand [1100, 1100, 1100, 1100, 1100, 1100]
resume-s9-catchexceptions-body new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-catchexceptions-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-catchexceptions-match new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-catchexceptions-suspended-match new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s9-catchexceptions-suspended-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-s9-catchexceptions-rhs new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-catch-body new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-catch-form new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-catch-form-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-s9-catch-bypassing-form new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-s9-catch-bypassing-guard new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
resume-g15-parameters new_resumes [1.0, 1.0, 1.0, 1.0, 1.0, 1.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-g15-template new_resumes [1.0, 1.0, 1.0, 1.0, 1.0, 1.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-g15-styled new_resumes [1.0, 1.0, 1.0, 1.0, 1.0, 1.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-g15-raw-failure new_resumes [1.0, 1.0, 1.0, 1.0, 1.0, 1.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-g15-raw-success new_resumes [1.0, 1.0, 1.0, 1.0, 1.0, 1.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-g15-projection-value new_resumes [2.0, 2.0, 2.0, 2.0, 2.0, 2.0] remainder_base [800, 800, 800, 800, 800, 800] remainder_cand [800, 800, 800, 800, 800, 800]
resume-g15-enclose new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [500, 500, 500, 500, 500, 500] remainder_cand [500, 500, 500, 500, 500, 500]
resume-g15-confirm new_resumes [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] remainder_base [600, 600, 600, 600, 600, 600] remainder_cand [600, 600, 600, 600, 600, 600]
PROOF reconciled True remainder []
```

### Older-workload reproducible reductions

Twelve older workloads have reproducible reductions, all in `try_apply_function` (-1 or -7 instructions/iteration), with nine of them also removing one ten-instruction empty-value drop. All other older-workload function totals vary across the six samples. The drop routine is symbolized by the compiler as `drop_in_place<...transparent_hold_form_expr::Work>`; its disassembly is instruction-for-instruction equal apart from relocation addresses. The removed path is its discriminant-5 early return, not an allocation or changed rendering operation. The caller evidence below identifies the eliminated call from `try_apply_function`; all execution addresses for every affected workload are in the complete per-label files.

```json
{
  "base": [
    {
      "caller": "rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::driver::<impl rum_runtime::runtime::model::session::Session>::finish_loop_iterator_dynamic_scope",
      "site": "0x5cbfcd",
      "calls_per_iteration": 1.0
    },
    {
      "caller": "rum_runtime::runtime::model::definitions::SymbolDefinition::store_own_value",
      "site": "0x6855cd",
      "calls_per_iteration": 3.0
    },
    {
      "caller": "rum_runtime::runtime::scoping::callable::<impl rum_runtime::runtime::model::session::Session>::try_apply_function",
      "site": "0x7177ac",
      "calls_per_iteration": 1.0
    },
    {
      "caller": "rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::clear_dynamic_scope_symbol_side_effects",
      "site": "0x59aff8",
      "calls_per_iteration": 1.0
    },
    {
      "caller": "rum_runtime::runtime::kernel_data::association_data::<impl rum_runtime::runtime::model::session::Session>::lookup_association_key_value",
      "site": "0x5b6cf0",
      "calls_per_iteration": 1.0
    },
    {
      "caller": "rum_runtime::runtime::model::definitions::SymbolDefinition::store_own_value",
      "site": "0x6855eb",
      "calls_per_iteration": 3.0
    }
  ],
  "candidate": [
    {
      "caller": "rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::driver::<impl rum_runtime::runtime::model::session::Session>::finish_loop_iterator_dynamic_scope",
      "site": "0x5cc05d",
      "calls_per_iteration": 1.0
    },
    {
      "caller": "rum_runtime::runtime::model::definitions::SymbolDefinition::store_own_value",
      "site": "0x6846bd",
      "calls_per_iteration": 3.0
    },
    {
      "caller": "rum_runtime::runtime::model::definitions::SymbolDefinition::store_own_value",
      "site": "0x6846db",
      "calls_per_iteration": 3.0
    },
    {
      "caller": "rum_runtime::runtime::eval_core::symbol::<impl rum_runtime::runtime::model::session::Session>::clear_dynamic_scope_symbol_side_effects",
      "site": "0x59b088",
      "calls_per_iteration": 1.0
    },
    {
      "caller": "rum_runtime::runtime::kernel_data::association_data::<impl rum_runtime::runtime::model::session::Session>::lookup_association_key_value",
      "site": "0x5b6d80",
      "calls_per_iteration": 1.0
    }
  ]
}
```

### Gate: timing-panels

Command: `python3 .temp/brief-619/run-timing-panels.py`. Exit 0. Full raw output: `logs/timing-panels.log`.

```text
No diagnostics.
```

## Final timing panels

Four full unchanged-script panels ran in order base, candidate, base, candidate under `taskset -c 2`, `--reps 2`, with profiling environment variables removed. All 120 labels completed in each panel; all four generated programs are byte-identical. The 32 disjoint-above labels trigger the specified aligned whole-program follow-up, rather than being treated as established regressions from two samples. Raw per-label times: `timing/panel-comparison.json`; run output:

```text
PANEL 1 base labels 120 run records engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	0.468659	/home/remi/dev/rum/.temp/brief-619/timing/panel1-base/raw/rum.stdout	/home/remi/dev/rum/.temp/brief-619/timing/panel1-base/raw/rum.stderr
PANEL 2 candidate labels 120 run records engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	0.464383	/home/remi/dev/rum/.temp/brief-619/timing/panel2-candidate/raw/rum.stdout	/home/remi/dev/rum/.temp/brief-619/timing/panel2-candidate/raw/rum.stderr
PANEL 3 base labels 120 run records engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	0.465571	/home/remi/dev/rum/.temp/brief-619/timing/panel3-base/raw/rum.stdout	/home/remi/dev/rum/.temp/brief-619/timing/panel3-base/raw/rum.stderr
PANEL 4 candidate labels 120 run records engine	status	exit_code	elapsed_seconds	stdout	stderr
rum	ok	0	0.472850	/home/remi/dev/rum/.temp/brief-619/timing/panel4-candidate/raw/rum.stdout	/home/remi/dev/rum/.temp/brief-619/timing/panel4-candidate/raw/rum.stderr
PANELS DONE; disjoint candidate ranges above base: ['association-part-key-read-width4000', 'association-part-key-read-width32000', 'loop-empty', 'table-count-null', 'symbol-ownvalue-guarded-read', 'module-chain-escape-small', 'table-small', 'table-iterator-plus', 'table-iterator-pair-list', 'upvalue-guarded-hit', 'subvalue-unguarded-hit', 'subvalue-unguarded-miss', 'default-guarded-hit', 'rule-rhs-deep-small', 'rewrite-guarded-hit', 'rewrite-guarded-decline-fallback', 'replacepart-deep-path-small', 'replacepart-wide-rule-list', 'stringtrim-literal-list-4000', 'stringreplace-single-literal-unguarded', 'appendto-part-outer1000-large', 'association-set-key-incremental-small', 'association-small-literal-repeat', 'merge-nested-small', 'set-part-fixed100-len100', 'set-part-fixed100-len800', 'set-part-fill-small', 'set-part-fill-large', 'upvalue-candidates-wide-small', 'upvalue-candidates-wide-large', 'keysort-callback-small', 'sort-symbol-default-width4000']
```

Aligned `base` CLI built with one codegen unit and exactly `RUSTFLAGS="-C llvm-args=-align-all-functions=6"`, separate target, workspace cwd. SHA-256 `ca3732a70b47c79094dc5a8013f4a2167fb286a409af57ba4e8b281d9efb8674`; full command, source hashes and build output: `timing/aligned/base/`.

### Gate: aligned-build-base

Command: `python3 .temp/brief-619/build-aligned.py base`. Exit 0. Full raw output: `logs/aligned-build-base.log`.

```text
No diagnostics.
```

Aligned `candidate` CLI built with one codegen unit and exactly `RUSTFLAGS="-C llvm-args=-align-all-functions=6"`, separate target, workspace cwd. SHA-256 `2e2fd0bd283931b8b7f9e4bbe030245a3861cda7847eb5ff6e7861cdfa4f66ff`; full command, source hashes and build output: `timing/aligned/candidate/`.

### Gate: aligned-build-candidate

Command: `python3 .temp/brief-619/build-aligned.py candidate`. Exit 0. Full raw output: `logs/aligned-build-candidate.log`.

```text
No diagnostics.
```

### Gate: aligned-timing

Command: `python3 .temp/brief-619/run-aligned-timing.py`. Exit 0. Full raw output: `logs/aligned-timing.log`.

```text
No diagnostics.
```

### Aligned whole-program timing result

Both arms were rebuilt serially with one codegen unit and exactly `RUSTFLAGS="-C llvm-args=-align-all-functions=6"`. Source hashes match the corresponding measured unaligned arm. The probe is the full generated panel, byte-identical except for `rumMicroReps = 2;` becoming `rumMicroReps = 25;`. Ten runs per arm were alternated on CPU 2 with profiling disabled, stdin closed, exact marker/label/iteration checks, final `Null`, and empty stderr. No other CPU-heavy task was launched during timing.

The first pair calibrated all 240 arm/label values against the corresponding panel median: probe/panel ratio 0.725831–1.515593, all within the preselected 0.5–2 factor. This checks that the full probe exercises the same work. `timing/program-hashes.json`, both `timing/aligned/*/` build records, and `timing/aligned-runs/` retain source, commands, output and hashes.

Verbatim follow-up output:
```text
ALIGNED TIMING 1 base labels 120
ALIGNED TIMING 1 candidate labels 120
CALIBRATION ratio range 0.7258314663187392 1.515592853275582 outside small factor []
ALIGNED TIMING 2 base labels 120
ALIGNED TIMING 2 candidate labels 120
ALIGNED TIMING 3 base labels 120
ALIGNED TIMING 3 candidate labels 120
ALIGNED TIMING 4 base labels 120
ALIGNED TIMING 4 candidate labels 120
ALIGNED TIMING 5 base labels 120
ALIGNED TIMING 5 candidate labels 120
ALIGNED TIMING 6 base labels 120
ALIGNED TIMING 6 candidate labels 120
ALIGNED TIMING 7 base labels 120
ALIGNED TIMING 7 candidate labels 120
ALIGNED TIMING 8 base labels 120
ALIGNED TIMING 8 candidate labels 120
ALIGNED TIMING 9 base labels 120
ALIGNED TIMING 9 candidate labels 120
ALIGNED TIMING 10 base labels 120
ALIGNED TIMING 10 candidate labels 120
ALIGNED TIMING COMPLETE, disjoint above base: []
```

All panel and follow-up rows are below. Times are nanoseconds per iteration, derived only from in-kernel `AbsoluteTiming`; elapsed process wall time is not used. Each panel column contains its two runs. The aligned column is the median of ten runs, each using the panel's minimum-of-25 convention.

| Label | Base panels ns/iteration | Candidate panels ns/iteration | Aligned base median | Aligned candidate median | Candidate/base | Aligned ranges disjoint above? |
|---|---:|---:|---:|---:|---:|---|
| `association-part-key-read-width4000` | 26539.000, 26100.500 | 30744.750, 30479.000 | 60442.988 | 58310.588 | 0.964720 | no |
| `association-part-key-read-width32000` | 201970.750, 205955.500 | 237815.000, 233641.500 | 240014.038 | 242914.225 | 1.012083 | no |
| `association-part-positional-read-width4000` | 762.325, 783.175 | 778.825, 771.025 | 760.987 | 760.975 | 0.999984 | no |
| `association-part-positional-read-width32000` | 767.375, 781.075 | 789.025, 768.475 | 764.988 | 759.275 | 0.992533 | no |
| `loop-empty` | 52.694, 53.013 | 55.590, 54.767 | 51.243 | 51.611 | 1.007168 | no |
| `do-count-null` | 24.773, 25.332 | 24.693, 24.827 | 24.799 | 24.386 | 0.983366 | no |
| `do-iterator-null` | 57.743, 55.660 | 59.354, 57.581 | 55.328 | 55.928 | 1.010835 | no |
| `table-count-null` | 14.847, 14.666 | 15.664, 15.742 | 14.927 | 15.221 | 1.019702 | no |
| `table-iterator-null` | 17.970, 18.461 | 18.136, 18.409 | 17.181 | 17.555 | 1.021738 | no |
| `table-iterator-symbol` | 6.849, 7.090 | 6.925, 7.183 | 6.280 | 6.168 | 0.982167 | no |
| `literal-integer` | 47.695, 46.992 | 46.678, 45.901 | 43.835 | 44.426 | 1.013482 | no |
| `literal-string` | 47.072, 49.465 | 46.409, 48.724 | 45.456 | 47.064 | 1.035375 | no |
| `symbol-ownvalue-read` | 111.391, 109.298 | 110.498, 110.167 | 108.470 | 107.829 | 0.994093 | no |
| `symbol-ownvalue-guarded-read` | 184.415, 184.188 | 185.318, 189.001 | 179.841 | 180.458 | 1.003425 | no |
| `set-ownvalue` | 267.192, 274.762 | 262.781, 267.337 | 260.098 | 259.712 | 0.998516 | no |
| `list-literal-small` | 58.476, 55.101 | 55.235, 54.447 | 54.774 | 53.928 | 0.984564 | no |
| `head-list` | 469.421, 475.508 | 472.638, 471.411 | 462.414 | 460.354 | 0.995545 | no |
| `length-list` | 464.295, 475.243 | 457.405, 464.428 | 461.397 | 457.697 | 0.991980 | no |
| `part-list` | 549.920, 564.525 | 541.365, 545.795 | 546.765 | 547.366 | 1.001098 | no |
| `sameq-integer` | 315.704, 321.690 | 308.217, 311.915 | 316.439 | 314.095 | 0.992594 | no |
| `sameq-list` | 631.908, 628.352 | 618.547, 616.339 | 621.481 | 620.822 | 0.998941 | no |
| `if-true` | 356.695, 360.995 | 351.083, 354.715 | 354.820 | 348.212 | 0.981376 | no |
| `module-local` | 1475.236, 1511.550 | 1507.678, 1480.494 | 1425.432 | 1425.028 | 0.999717 | no |
| `module-chain-escape-small` | 552.604, 548.490 | 621.688, 553.906 | 484.198 | 480.339 | 0.992029 | no |
| `module-chain-escape-large` | 493.333, 539.786 | 506.203, 501.630 | 500.268 | 498.755 | 0.996976 | no |
| `block-local` | 939.952, 960.476 | 938.978, 965.266 | 929.363 | 929.483 | 1.000129 | no |
| `block-symbol-only-repeat` | 317.264, 311.806 | 306.045, 310.073 | 305.099 | 306.124 | 1.003360 | no |
| `block-init-null-repeat` | 362.564, 364.272 | 355.987, 353.603 | 351.587 | 352.053 | 1.001327 | no |
| `block-init-read-repeat` | 413.050, 412.488 | 406.574, 405.297 | 400.373 | 401.509 | 1.002839 | no |
| `with-local` | 962.490, 960.802 | 976.568, 953.676 | 949.021 | 941.889 | 0.992485 | no |
| `with-deep-body-small` | 4081.520, 4141.560 | 4034.460, 4063.060 | 3551.243 | 3498.326 | 0.985099 | no |
| `table-small` | 934.960, 928.170 | 936.880, 997.500 | 916.795 | 909.337 | 0.991866 | no |
| `table-iterator-plus` | 35.934, 35.948 | 36.606, 36.412 | 36.911 | 37.077 | 1.004470 | no |
| `table-iterator-pair-list` | 63.456, 62.036 | 66.550, 65.848 | 59.217 | 59.580 | 1.006122 | no |
| `do-small` | 4531.070, 4576.085 | 4526.355, 4622.425 | 4458.398 | 4431.848 | 0.994045 | no |
| `d-nested-sin-small` | 27193.350, 27249.950 | 26657.500, 26732.400 | 26479.060 | 26698.040 | 1.008270 | no |
| `downvalue-exact-hit` | 382.238, 390.689 | 377.021, 384.603 | 379.320 | 376.768 | 0.993270 | no |
| `downvalue-exact-miss` | 276.529, 277.981 | 265.413, 271.299 | 269.464 | 267.395 | 0.992324 | no |
| `downvalue-pattern-hit` | 974.980, 979.441 | 970.988, 975.165 | 938.166 | 935.202 | 0.996840 | no |
| `downvalue-pattern-miss` | 281.512, 296.308 | 273.911, 278.428 | 279.842 | 276.585 | 0.988361 | no |
| `downvalue-exact-miss-repeat` | 259.508, 243.984 | 243.146, 243.336 | 246.524 | 245.033 | 0.993952 | no |
| `downvalue-pattern-miss-repeat` | 264.640, 252.834 | 250.034, 252.882 | 255.439 | 254.082 | 0.994688 | no |
| `downvalue-guarded-hit` | 1891.150, 1908.860 | 1884.240, 1886.370 | 1870.617 | 1881.132 | 1.005622 | no |
| `downvalue-guarded-decline-fallback` | 1425.460, 1419.070 | 1413.510, 1416.970 | 1398.584 | 1393.548 | 0.996399 | no |
| `upvalue-guarded-hit` | 2329.090, 2335.650 | 2356.390, 2363.300 | 2314.701 | 2335.006 | 1.008772 | no |
| `subvalue-unguarded-hit` | 1398.030, 1398.000 | 1401.600, 1417.360 | 1382.755 | 1372.637 | 0.992682 | no |
| `subvalue-unguarded-miss` | 784.417, 778.899 | 791.339, 793.124 | 773.378 | 769.783 | 0.995351 | no |
| `subvalue-guarded-hit` | 2435.150, 2524.080 | 2478.810, 2467.830 | 2408.811 | 2414.697 | 1.002444 | no |
| `default-unguarded-hit` | 1368.790, 1578.010 | 1508.730, 1517.720 | 1271.285 | 1267.736 | 0.997208 | no |
| `default-unguarded-miss` | 1060.830, 1077.390 | 1069.210, 1078.840 | 1057.450 | 1058.180 | 1.000690 | no |
| `default-guarded-hit` | 1617.230, 1638.270 | 1647.470, 1652.730 | 1621.335 | 1617.624 | 0.997711 | no |
| `matchq-integer-success` | 286.294, 291.779 | 283.962, 286.437 | 284.828 | 284.395 | 0.998483 | no |
| `matchq-integer-fail` | 291.177, 300.377 | 286.988, 294.343 | 289.440 | 287.908 | 0.994707 | no |
| `matcher-single-tiny` | 693.997, 684.346 | 688.109, 694.393 | 679.408 | 682.910 | 1.005154 | no |
| `matcher-single-wide-small` | 55.976, 56.385 | 55.354, 58.225 | 48.969 | 49.571 | 1.012304 | no |
| `matcher-single-wide-large` | 50.094, 48.576 | 53.884, 49.373 | 48.385 | 48.207 | 0.996332 | no |
| `replaceall-simple` | 746.740, 736.934 | 715.356, 723.250 | 718.128 | 711.856 | 0.991266 | no |
| `rule-rhs-deep-small` | 5999.160, 5964.800 | 6040.600, 6029.420 | 5860.919 | 5860.563 | 0.999939 | no |
| `rewrite-guarded-hit` | 3559.780, 3553.280 | 3567.960, 3567.400 | 3568.385 | 3569.442 | 1.000296 | no |
| `rewrite-guarded-decline-fallback` | 5061.120, 5090.440 | 5133.460, 5098.000 | 5079.005 | 5084.795 | 1.001140 | no |
| `cases-wide-no-match` | 92.481, 91.026 | 92.479, 91.987 | 89.919 | 90.180 | 1.002902 | no |
| `count-wide-no-match` | 85.531, 85.939 | 85.641, 85.862 | 85.291 | 85.281 | 0.999875 | no |
| `position-wide-no-match` | 106.721, 106.448 | 109.401, 104.874 | 105.043 | 105.628 | 1.005575 | no |
| `deletecases-wide-no-match` | 105.467, 111.938 | 105.298, 112.009 | 104.516 | 104.596 | 1.000765 | no |
| `deletecases-wide-association-no-match` | 271.063, 261.292 | 262.840, 262.996 | 264.552 | 262.783 | 0.993313 | no |
| `deletecases-leaf-delete-small` | 4253.880, 4267.100 | 4253.700, 4248.480 | 4258.072 | 4180.223 | 0.981717 | no |
| `replace-wide-no-match` | 49.966, 49.036 | 49.159, 48.981 | 49.015 | 48.395 | 0.987344 | no |
| `replace-leaf-rebuild-small` | 5125.420, 5139.320 | 5137.340, 5125.400 | 5203.088 | 5093.563 | 0.978950 | no |
| `take-multispec-small` | 3727.480, 3665.260 | 3644.120, 3682.180 | 3172.492 | 3188.991 | 1.005201 | no |
| `replacepart-deep-path-small` | 2556.740, 2568.660 | 2630.560, 2627.180 | 2590.571 | 2600.903 | 1.003988 | no |
| `replacepart-wide-rule-list` | 184.376, 185.074 | 192.951, 190.719 | 179.771 | 179.929 | 1.000880 | no |
| `replacepart-guarded-hit` | 1072.816, 1081.654 | 1084.080, 1073.278 | 1062.604 | 1068.071 | 1.005145 | no |
| `replacepart-guarded-decline-fallback` | 1182.342, 1196.350 | 1197.140, 1189.382 | 1168.087 | 1178.005 | 1.008491 | no |
| `string-repeat-small` | 372.088, 380.988 | 367.520, 372.320 | 370.524 | 366.258 | 0.988487 | no |
| `character-range-small` | 659.395, 662.270 | 654.595, 656.710 | 646.205 | 648.745 | 1.003931 | no |
| `stringtrim-literal-list-4000` | 6590.270, 6590.370 | 6749.820, 6665.690 | 6543.685 | 6575.565 | 1.004872 | no |
| `stringtrim-literal-list-32000` | 51071.900, 50617.500 | 50531.900, 50771.000 | 50383.270 | 50417.300 | 1.000675 | no |
| `stringtrim-single-literal-4000` | 6374.030, 6379.020 | 6342.540, 6346.130 | 6375.505 | 6415.610 | 1.006290 | no |
| `stringtrim-single-literal-32000` | 50898.400, 50320.300 | 51421.300, 50366.400 | 50248.445 | 50358.195 | 1.002184 | no |
| `stringtrim-whitespace-4000` | 6320.370, 6317.490 | 6299.290, 6303.220 | 6268.120 | 6313.305 | 1.007209 | no |
| `stringtrim-whitespace-32000` | 50251.700, 50282.400 | 50116.500, 50320.100 | 50153.850 | 50163.325 | 1.000189 | no |
| `stringreplace-single-literal-unguarded` | 1946.105, 1964.240 | 1971.725, 1965.775 | 1902.687 | 1950.787 | 1.025280 | no |
| `stringreplace-pattern-unguarded` | 2096.700, 2118.460 | 2103.980, 2113.640 | 2091.487 | 2092.930 | 1.000690 | no |
| `stringreplace-guarded-hit` | 1400.692, 1426.512 | 1398.298, 1413.652 | 1388.658 | 1383.132 | 0.996021 | no |
| `stringreplace-guarded-decline-fallback` | 1635.590, 1671.886 | 1650.442, 1643.300 | 1636.116 | 1650.810 | 1.008981 | no |
| `appendto-list-incremental-small` | 2566.788, 2557.956 | 2604.692, 2540.312 | 1776.666 | 1887.534 | 1.062402 | no |
| `appendto-list-incremental-large` | 4379.460, 4163.440 | 4233.640, 4166.580 | 3062.692 | 2996.080 | 0.978251 | no |
| `appendto-part-outer1000-small` | 22073.500, 22289.900 | 22302.300, 22267.600 | 22141.760 | 21969.105 | 0.992202 | no |
| `appendto-part-outer1000-large` | 22126.600, 22339.800 | 22599.600, 22451.200 | 22274.830 | 22187.013 | 0.996058 | no |
| `association-set-key-incremental-small` | 14459.800, 14472.400 | 14504.500, 14555.100 | 14227.795 | 14251.655 | 1.001677 | no |
| `association-set-key-incremental-large` | 27266.350, 27400.450 | 27506.850, 27321.750 | 27228.982 | 27213.515 | 0.999432 | no |
| `association-small-literal-repeat` | 1473.870, 1472.284 | 1481.690, 1476.804 | 1448.464 | 1437.071 | 0.992134 | no |
| `merge-nested-small` | 5538.750, 5537.300 | 5606.600, 5663.100 | 5462.675 | 5475.832 | 1.002409 | no |
| `set-part-fixed100-len100` | 1372.370, 1462.680 | 1484.890, 1474.600 | 1422.920 | 1391.745 | 0.978091 | no |
| `set-part-fixed100-len800` | 5381.810, 5968.390 | 6189.640, 6174.500 | 5965.395 | 5420.285 | 0.908621 | no |
| `set-part-fill-small` | 2027.110, 2138.480 | 2183.610, 2214.415 | 2145.505 | 2031.080 | 0.946668 | no |
| `set-part-fill-large` | 3239.275, 3464.775 | 3586.775, 3556.900 | 3439.296 | 3191.918 | 0.928073 | no |
| `upvalue-candidates-tiny` | 1936.790, 1952.920 | 1916.190, 1941.720 | 1919.841 | 1903.610 | 0.991546 | no |
| `upvalue-candidates-wide-small` | 92.042, 94.378 | 99.002, 98.342 | 91.144 | 92.294 | 1.012621 | no |
| `upvalue-candidates-wide-large` | 91.782, 92.783 | 99.308, 100.074 | 90.798 | 90.984 | 1.002051 | no |
| `sort-callback-small` | 1700.180, 1680.600 | 1670.730, 1636.680 | 1619.990 | 1648.790 | 1.017778 | no |
| `sort-callback-large` | 1913.935, 1888.685 | 1914.030, 1894.975 | 1851.902 | 1885.890 | 1.018353 | no |
| `sortby-callback-small` | 1977.920, 2007.060 | 1989.770, 1970.130 | 1936.920 | 1946.530 | 1.004961 | no |
| `sortby-callback-large` | 2259.155, 2277.310 | 2210.355, 2168.995 | 2175.137 | 2183.140 | 1.003679 | no |
| `reversesort-callback-small` | 1649.610, 1652.190 | 1600.990, 1579.100 | 1577.830 | 1570.415 | 0.995301 | no |
| `reversesort-callback-large` | 1855.540, 1856.490 | 1784.510, 1783.205 | 1768.827 | 1770.225 | 1.000790 | no |
| `keysort-callback-small` | 1878.060, 1872.500 | 1906.040, 1915.880 | 1638.740 | 1645.110 | 1.003887 | no |
| `keysort-callback-large` | 1929.160, 1866.680 | 1892.460, 1883.080 | 1805.155 | 1813.795 | 1.004786 | no |
| `keysortby-callback-small` | 1825.240, 1851.800 | 1788.640, 1794.840 | 1770.820 | 1772.200 | 1.000779 | no |
| `keysortby-callback-large` | 1981.580, 1978.900 | 1931.540, 1936.220 | 1922.420 | 1921.230 | 0.999381 | no |
| `sort-default-wide` | 10.764, 10.509 | 10.756, 10.614 | 10.476 | 10.396 | 0.992364 | no |
| `sort-symbol-default-width4000` | 73.073, 71.215 | 73.928, 75.543 | 71.963 | 72.307 | 1.004780 | no |
| `sort-symbol-default-width16000` | 79.201, 79.701 | 79.672, 80.627 | 79.788 | 80.815 | 1.012875 | no |
| `sortby-default-wide` | 324.354, 319.428 | 310.413, 322.100 | 310.359 | 309.288 | 0.996548 | no |
| `reversesort-default-wide` | 10.448, 10.493 | 10.383, 10.706 | 10.320 | 10.402 | 1.008025 | no |
| `keysort-default-wide` | 232.991, 232.843 | 234.774, 232.764 | 227.371 | 226.114 | 0.994471 | no |
| `keysortby-default-wide` | 549.636, 553.059 | 543.996, 543.146 | 542.855 | 540.664 | 0.995964 | no |
| `valueq-undefined-symbol-definitions-2000` | 446.350, 461.450 | 451.250, 446.800 | 393.725 | 389.675 | 0.989714 | no |
| `valueq-undefined-symbol-definitions-8000` | 415.550, 445.150 | 429.550, 425.450 | 389.650 | 390.075 | 1.001091 | no |
| `module-retained-gensym-gc-batch` | 1757.150, 1630.550 | 1602.750, 1685.800 | 1645.025 | 1647.862 | 1.001725 | no |

The two-sample panels had 32 disjoint-above labels. After the prescribed alignment check, 0 labels have all ten candidate values above all ten base values. Timing is an indication only; these residuals are retained for the director, with the allocation/instruction/copy evidence reported separately.

| Remaining timing label | Base range ns/iteration | Candidate range ns/iteration | Median ratio |
|---|---:|---:|---:|
| None | — | — | — |

### Gate: final-source-scope-audit

Command: `python3 .temp/brief-619/final-scope-audit.py`. Exit 0. Full raw output: `logs/final-source-scope-audit.log`.

```text
No diagnostics.
```

## Final source/scope verification

```text
Final source audit: 2079 base files intact; 18 owned existing edits, 6 owned new files; 0 unowned changes.
All four candidate builds match current source; protected suspension files, S10 helpers, F0528 and non-status tail unchanged.
New module character counts: {"crates/rum-runtime/src/runtime/scoping/callable/status_application.rs": 3935, "crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/controls.rs": 6534, "crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/guest_heads.rs": 4098, "crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/mod.rs": 1921, "crates/rum-stdlib/tests/suspension_oracle/fixtures/status_function/return_boundaries.rs": 1815, "crates/rum-stdlib/tests/typed_slot_census/g15a.rs": 2944}
```
All four final candidate binary source snapshots still match. Both aligned builds emitted no compiler warnings. The formatter also had a second intermediate failing check, retained in `logs/fmt-attempt2.log`; it emitted formatting diffs only, was corrected within the owned file list and is now explicitly restored to the historical command ledger. The passing `fmt-final` result remains the final formatting gate.

### Indicative timing medians retained

Overlapping ten-run ranges do not establish exact timing equality. 60 of the 120 aligned median ratios exceed 1. The largest median increase is appendto-list-incremental-small (+6.2402%, six of ten pairs slower); literal-string is +3.5375% (eight pairs), and stringreplace-single-literal-unguarded is +2.5280% (eight pairs). These are indicative timing residuals with overlapping ranges, retained for director review. The latter was an initial instruction flag that the required five-run instruction follow-up cleared; none of these three is a remaining instruction flag. `literal-string` is a timing-panel label without a standalone row in the copied N/2N instruction suite, so no separate instruction-equality claim is made for it. `timing/aligned-runs/median-increases.json` retains every positive median and its ten paired ratios; all 120 full distributions remain in comparison.json. No timing observation is substituted for the instruction/copy/stack evidence.

| Largest aligned median increases | Candidate/base median | Slower pairs / 10 | Paired median ratio |
|---|---:|---:|---:|
| `appendto-list-incremental-small` | 1.062402 | 6 | 1.043974 |
| `literal-string` | 1.035375 | 8 | 1.030841 |
| `stringreplace-single-literal-unguarded` | 1.025280 | 8 | 1.026515 |
| `table-iterator-null` | 1.021738 | 5 | 1.000760 |
| `table-count-null` | 1.019702 | 6 | 1.031979 |
| `sort-callback-large` | 1.018353 | 9 | 1.019138 |
| `sort-callback-small` | 1.017778 | 8 | 1.018165 |
| `literal-integer` | 1.013482 | 5 | 1.016021 |
| `sort-symbol-default-width16000` | 1.012875 | 7 | 1.009743 |
| `upvalue-candidates-wide-small` | 1.012621 | 7 | 1.016330 |
| `matcher-single-wide-small` | 1.012304 | 8 | 1.009366 |
| `association-part-key-read-width32000` | 1.012083 | 7 | 1.008393 |
