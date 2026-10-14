# brief-609 — G13 step S3: type evaluation, argument preparation and dispatch

## Goal

Type the first production family through S2's typed API. S3 owns the frames
that suspend inside ordinary evaluation: the call path, argument preparation,
builtin dispatch stages, stored own-value evaluation, `ReleaseHold`, and the
fixed-point resolution loop. When you are done:

- the 12 owned frame types (11 frames and 1 finalizer) implement the typed
  API, and the 13 owned macro sites use the witnessed typed macro. Every
  other family still runs through its legacy transport;
- the five owned transformations (T0023-T0027) and the one deferred
  obligation (D0001) are applied as specified;
- the six historical Fold/Nest divergences (a guest value under an
  `InternalContinuation*` head that the resumed path misread) are executable
  oracle fixtures that pass with strict equality: the resumed path equals the
  queued-input path in value and every effect, in both builds;
- the ordinary path of every program is unchanged, and the cost of S3 is
  measured and attributed.

This is step S3 of G13 (design round 5, brief-606, ratified). S1 (brief-607,
`777c61cc`) built the dormant typed API. S2 (brief-608, committed just
before this brief) made the driver run typed and legacy entries side by
side. S4-S54 type the remaining families; F1-F23 flip transport per type;
R removes the legacy machinery.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- Authority, in order: this brief; brief-606 (its D1/D2 and report) with
  `.temp/brief-606/design.md` and `census-spec.md`; `.temp/brief-605/design.md`
  §§2-5 for the API, token, fault, unwind and accounting design, **except
  §5's transit record, which D2 rejected**.
- **S2 as built.** Read brief-608's last execution report (redispatch 5).
  The driver is in `suspension/core.rs`, `core/{queue,fault,abandon}.rs` and
  `typed/entry.rs`. Typed entries take the driver's typed arm: decode before
  pop, first-fault latch, two-Arc entries. The census lives under
  `crates/rum-stdlib/tests/typed_slot_census*`, the compile controls in
  `typed_slot_compile_controls.rs`, and the resume harness in
  `.temp/brief-608/resume-harness/`. Reuse the harness; do not rebuild it.
- **Rows, as input; never edit them.** All of these are in `.temp/brief-606/`:
  - `all-frame-migrations.tsv`: F0204, F0272-F0279, F0526, F0527, F0570;
  - the macro-dataflow rows M0205, M0279-M0283, M0285-M0289, M0561, M0562;
  - `transformation-dispositions.tsv`: T0023-T0027;
  - `deferred-obligations.tsv`: D0001;
  - `divergence-step-gates.tsv`: the six rows with `typing_step = S3`;
  - `draft-S3.md`: Codex's plan for this step.

  The draft is raw material. Where it and this brief differ, this brief
  governs. In particular, its Scope, test locations and measurement section
  are superseded.
- Baseline coordinates in those tables are stale; refresh them by discovery.
  Current definitions (checked by the director at S2's commit):

  | Row | Type | Defined at |
  | --- | --- | --- |
  | F0204 | `BuiltinEvaluateSuspensionFrame` | `builtin_adapters/resume_frames.rs:5` |
  | F0272 | `PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame` | `eval_call_prepare/resume_frames.rs:5` |
  | F0273 | `ResumeSuspendedCallArgumentPreparationSuspensionFrame` | `eval_call_prepare/resume_frames.rs:35` |
  | F0274 | `EvaluateCallSuspensionFrame` | `eval_core/call.rs:361` |
  | F0275 | `ResumeCallWithDispatchPlanSuspensionFrame` | `eval_core/call.rs:384` |
  | F0276 | `ResumeSuspendedCallWithDispatchPlanSuspensionFrame` | `eval_core/call.rs:422` |
  | F0277 | `EvaluateStoredOwnValueSuspensionFrame` | `eval_core/symbol.rs:304` |
  | F0278 | `RunCallDispatchStageSuspensionFrame` | `eval_dispatch/mod.rs:425` |
  | F0279 | `EvalReleaseHoldSuspensionFrame` | `hold_thread.rs:115` |
  | F0526 | `EvaluateProfiledFixedPointSuspensionFrame` | `resolution/resume_frames.rs:5` |
  | F0527 | `ResumeFixedPointFromSuspensionFrame` | `resolution/resume_frames.rs:26` |
  | F0570 | `FinishPreparedCallArgumentsFinalizer` | `eval_call_prepare/resume_frames.rs:75` |

  All paths are under `crates/rum-runtime/src/runtime/`.
- One stale reference: T0027's "value-shape corpus per position in 608" uses
  round 4's numbering, where 608 was the atomic conversion. Under round 5
  that corpus belongs to the owning type's flip step, not to S3. S3 observes
  the owning oracle descriptor only.

## Scope

A listed file or class is permission to do this step's owned work there. It
is not permission to convert another step's rows, change evaluation beyond
the owned transformations, or tidy. New files are marked.

**Classes.** Edit what the owned work needs and disclose each file.

- **The family's source files**, for the owned F and M rows, T0023-T0027 and
  D0001. All are under `crates/rum-runtime/src/runtime/`:
  - `builtin_adapters.rs`, `builtin_adapters/resume_frames.rs`;
  - `eval_call_prepare.rs`, `eval_call_prepare/resume_frames.rs`;
  - `eval_core/call.rs`, `eval_core/mod.rs`, `eval_core/symbol.rs`,
    `eval_core/stability.rs`, `eval_core/stability/association_tail.rs`;
  - `eval_dispatch/mod.rs`;
  - `hold_thread.rs`;
  - `resolution.rs`, `resolution/resume_frames.rs`,
    `resolution/match_normalization.rs`.
- **Module roots on an owned type's path.** Make only the visibility or
  re-export edit the typed API needs to reach the type, and disclose it. The
  director ran `module-path-visibility.py` on this brief; its findings are at
  the end of this section.
- **Constructor ripple.** Any file where an owned frame or helper is
  constructed or named, because the owned construct changed: edit only that
  construct's sites. Owned source constructors migrate with their frame,
  including cross-module sites.
- **`crates/rum-runtime/src/runtime/suspension/`**, only where S3's
  production use exposes a gap in S2's API or driver. Disclose each edit and
  why it is needed. Sealing, opacity and the no-dual-arm rule stay intact.
  A change to the driver's resume path is measured with the resume harness
  exactly as in S2.
- **Sibling unit tests.** A test that needs a family module's private items
  goes in that module's sibling `tests.rs`, as
  `#[cfg(test)] mod tests;`. Never use `#[path = "../…"]`: the ownership
  loader does not follow `..` and fails closed.
- **Census, oracle and controls.** Any file under
  `crates/rum-stdlib/tests/typed_slot_census/` and `typed_slot_census.rs`.
  `crates/rum-stdlib/tests/suspension_oracle/fixtures/` for the six
  divergence fixtures, the two ordinary controls and the `typed_slots.rs`
  receipt. `suspension_oracle/inventory/typed-slot-stages.tsv` and
  `typed-slot-shapes.tsv`. `typed_slot_compile_controls.rs`, to rerun all
  30 pairs; S3 adds none.
- **Pins that follow code.** A test or inventory row that names a touched
  file's path, line, function or count may be updated only to follow the
  same construct after it moves or is renamed. Disclose each update as
  old → new. A row whose construct disappears by an owned transformation is
  a planned transition; list it. Anything else is a STOP.

  The director's grep found these pinning files. The list is a starting
  point, not a boundary:
  - `runtime/tests/frame_root_trace.rs` and
    `tests/frame_root_trace/derived_inventory.rs`;
  - `crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs`
    (it lists `resolution.rs`);
  - `crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv`,
    `catalog.tsv` and `shared-effects.tsv` (owner files and owner
    functions, for example
    `prepare_arguments_for_call_with_unevaluated_restore`);
  - `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv`, including
    `self_reaching.tsv`, by discovery replay only;
  - `crates/rum-stdlib/src/tests/retention_closure/inventory/{local-contracts,promoted}.tsv`
    and `retention_closure/local/{contracts,mod,resume_macro,rule_tests}.rs`,
    by fingerprint and certificate replay only;
  - `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`, if a
    pinned site moves. Classify every diff as in brief-608 redispatch 3: a
    moved pair or an attributed S3 change. An unpaired removal is a STOP.

**Files** (S2's leftover, item 8 below):

- `crates/rum-runtime/src/runtime/suspension/core/fault.rs`
- `crates/rum-runtime/src/runtime/session/session_requests.rs`
- `crates/rum-runtime/src/runtime/session/session_eval.rs`
- `crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs`
- `crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs`

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md` and the core `Expr`/`Symbol` representation stay unchanged.
Scratch goes under `.temp/brief-609/`; the certificate replay may write
`.temp/brief-546/certificates/`.

**Disclosure duty.** If a needed change falls outside every list and class,
make no edit there. STOP and report the file, the line and why.

**Module-path visibility (director's check).** All 12 owned types are
private or `pub(super)`. Each module on their paths is declared private in:

- `builtin_adapters.rs:1` (`mod resume_frames`);
- `eval_call_prepare.rs:1`;
- `eval_core/mod.rs:9` (`mod symbol`);
- `resolution.rs:2`.

All four are in the family class above. Generic typed code is
monomorphized where the frame is defined, so the API itself should need no
wider visibility. If an impl does need some (a private type in a public
interface, or a registry that names the type), the edit belongs to these
roots; disclose it.

## Steps

1. **Bases first.** Before any edit, build the four measurement bases from
   the committed S2 tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each
   in its own target directory under `.temp/brief-609/`:
   - `rum-cli`;
   - `rum-cli-alloc` (`--features rum-cli/alloc-count`);
   - the resume harness, `resume`;
   - the resume harness with allocation counting, `resume-alloc`.

   Hash them.
2. **Divergence fixtures before conversion (watch them fail).** Add the six
   fixtures from `divergence-step-gates.tsv`'s `source` column (`fold` and
   `nest`, each with `none`, `some` and `empty_some`) as strict-equality
   oracle cases. Each compares the queued-input path with the resumed path
   in value, log, messages, prints and trace, under `Checks::ALL`, with
   unconditional `FaultProbe` absence. Run them on the unmodified S2 tree
   and record the six failures with their actual output; the table's
   `resumed` column predicts `UnexpectedPause(1)`. A fixture that passes
   before the conversion does not test the defect. Report it and fix the
   fixture, not the expectation.

   After the conversion all six must be equal. A fixture may be unable to
   reach equality within S3's owned rows: for example, the guest value
   might still cross a type whose mode stays Legacy until that type's flip.
   In that case STOP and report the exact path. Do not move the promotion
   to a later step and do not relax the comparison.
3. **Type the owned rows.**
   - Convert each F row to the typed API.
   - Derive `Input` from the actual producer `Result` before any
     transformation. Bind `Output` at the innermost function or closure
     return.
   - Use complete frame and helper bodies. Keep pending-generation checks,
     cursor advancement, owner drop order, held and evaluation order, and
     profiling balance exactly.
   - Each M site uses the witnessed typed macro. Its ordinary path matches
     the producer `Result` once and constructs no frame, `Arc` or `Vec`.
4. **Transformations and the deferred obligation**, as their rows specify:
   - **T0023:** remove the unreachable generic `PrepareArguments` dispatch
     arm, then the now-unused `prepare_arguments_for_call` wrapper. A source
     gate asserts that the wrapper has no caller.
   - **T0024:** witness before `map(SharedExpr::new)`. Move the `SharedExpr`
     conversion into the shared `resume_call_after_head` entry.
   - **T0025 and T0026:** split the `PreparedCallArguments` consumer from
     the `Option<Expr>` stage consumer. A shared helper normalizes the
     arguments and installs the restore plan once.
   - **T0027:** delete only the transport encoding. `TypedAction` completes
     the helper's exact typed result.
   - **D0001:** prepared arguments use the adapter token with a fresh state
     cursor and splice flag, and no captured `Arc`.
     `FinishPreparedCallArgumentsFinalizer` runs after both originating
     `Arc`s drop.
   - A pure transport map disappears. A semantic map executes identically on
     the ordinary and resumed paths.
5. **Ordinary controls.** Add branch-observing fixtures for `prepared.twice`
   (first and third evaluated arguments; cursor and restore state) and
   `head.map.twice` (two pauses in a `CompoundExpression` head before
   dispatch; the head finishes before `SharedExpr` conversion). All 28
   ordinary controls stay PASS.
6. **Modes.** S3 has no State births. `PreparedCallArguments` and every
   other existing type keep their single global mode; a typed S3 frame
   exchanges a Legacy-mode type through that type's existing wire. No
   per-frame override and no dual-arm decode.
7. **Census.** Advance the receipt to S3 in `typed_slot_census` and in
   `suspension_oracle/fixtures/typed_slots.rs`. Record every owned row's
   transition. The production totals stay 607 implementations and 589 macro
   sites. Report how many are now typed (expected: 12 and 13) and the actual
   new or split frame names.
   - The oracle build currently warns that `TYPING_RECEIPT` is never used
     (pre-existing since S1). Make it used or scoped so that neither build
     warns.
8. **S2 leftover.** `continuation_fault_outcome` now always returns `Err`
   but keeps a `Result<T, EvalError>` signature, and four production callers
   unwrap it. Retype it so the type states the fact: a cold function that
   consumes the outcome and returns `EvalError`. Callers write
   `Err(self.…(outcome))`. Update the two test callers to match, and nothing
   else in those test files. This is the cold path only; confirm with the
   address-level check that the hot boundary is unchanged.
9. **Inventories.** Replay fingerprints, both certificate sets and ownership
   discovery. Disclose every changed row. A new `UnknownHostPayload` without
   an approved disposition is a STOP.

## Verification (MANDATORY — run to green or evidence)

Run everything serially, each command through `python3
scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-609/tmp …`. Bounds: workspace 3 600 s; oracle
1 500 s per build; everything else 900 s. These are the durations S2's gates
actually used.

```
cargo fmt --check
cargo test -p rum-runtime --lib suspension::typed::tests
cargo test -p rum-runtime --lib frame_root_trace
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

Report every `test result:` total and the oracle's fault-occurrence delta
for each build. Report native ZeroMQ skips as skips, not passes. Report any
compiler warning that the diff introduces.

**Performance.** S3 rewrites the hottest evaluation paths, so it is
measured; there is no exemption. `TEMPLATE.md` pitfall 8 governs, with
brief-608's refinements.

1. **Allocations and bytes first**, on all 64 labels in
   `.temp/brief-608/perf/labels.json`, against the bases from step 1.
   - Ordinary labels must be exactly equal.
   - Resume workloads change by design: a typed entry is two allocations,
     the outer adapter `Arc` and the inner frame `Arc`. For each workload,
     report which frame types suspend per resume, before and after, and
     check that the change is exactly one extra allocation, with its bytes,
     per typed frame per resume, and nothing else.
2. **Instruction slopes.** Two interleaved `perf stat` pairs per arm on all
   64 labels: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, stdin
   `/dev/null`, slope `(count(2N) - count(N)) / N`, raw output saved before
   parsing.
   - A label is flagged when its delta exceeds `max(0.05%, base range,
     candidate range)`.
   - A flagged label gets five runs per arm, and at least three callgrind
     processes per arm.
   - Callgrind totals vary per process, because rum seeds its hashing per
     process. A total counts as added work only if the difference of the
     arm means exceeds the larger arm range, or if address-level
     attribution shows the candidate executing code the base does not.
3. **Resume attribution.** For all five harness workloads, do a callgrind
   per-function diff plus address-level attribution, as in brief-608
   redispatches 4 and 5.
   - S3 is the first production typed arm, so attribute each typed frame's
     resume cost against its legacy cost: the decode through its mode, the
     adapter dispatch, and the second `Arc`.
   - Name every instruction the typed arm adds. A copy or move of an
     action, slot or error that the legacy arm did not make is a defect.
     Remove it, or report it with its addresses.
4. **Timing** last, as an indication only: the full
   `scripts/core-language-microbench.sh` panel on both one-unit binaries,
   profiling off, with no other work on the host. Do not substitute timing
   for an instruction result.

A regression is a defect to fix or report.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- No default-path cost.
- No owned-tree clones on hot paths.
- The witness comes from the actual `Result`. Never guess input from a
  synthetic `Err` type.
- The token carries no `Arc`, is not `Clone`, and accepts fresh state of the
  same frame type only.
- Actions, codecs and modes stay sealed and opaque. Nothing public names a
  mode or constructs an erased entry.
- Profiling records stay balanced on every path, ordinary and resumed.
- New modules should stay under 10 k characters. Check
  `.temp/brief-606/file-size-assessments.tsv` before adding to a large file;
  split by responsibility, never by arbitrary cuts.

## Checkpointing — protect the scene

Everything goes under `.temp/brief-609/`. Write `PROGRESS.md` after each
step, with the verified state, the compile and test counters, any
half-edited files, and the exact next action.

## Hard rules

- **No `git`, in any form.** No network. Write nothing outside this
  repository except the certificate replay output named above.
- Modify only the Scope. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- Commands: scoped reads and edits, `rg`/`grep`, bounded `python3`, the
  bounded Cargo commands above, bounded compiler fixtures,
  `scripts/check-suspension-oracle.sh`, `scripts/core-language-microbench.sh`,
  `valgrind`/`callgrind_annotate`/`objdump` on the measurement binaries, and
  the measurement commands named above. No probe that aborts a process.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No dual-arm decoder, per-frame mode override, transit record or public raw
  adapter. Type no row outside S3's. Never weaken an oracle `Checks` flag, a
  retention reject, a fault-absence gate or a documented test to get a pass.
- Stop-loss is keyed on progress. Compile errors you are reducing count as
  progress. Stop after two attempts that move none of the named failing
  gates.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with its semantic change. List every ripple, module
   root and pin update as old → new.
2. Owned-row reconciliation: each F/M row's transition, the new or split
   frame names, T0023-T0027, D0001 and the census output.
3. The divergence fixtures: their pre-conversion failures, verbatim, and
   their post-conversion equality in both builds.
4. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions.
5. Allocation, instruction, resume-attribution and timing evidence.
6. The S2 leftover's retype and its hot-path check.
7. Stop-loss counters, deviations and open questions.

## Director amendment — redispatch 1: the six Fold/Nest promotions belong to S13

Your step-2 STOP was correct, and the director confirmed the path in the
source. A model-capacity error cut your session while you were writing the
stop report. Append the stop evidence briefly as section 0 of this
redispatch's report: the six pre-conversion failures verbatim and the traced
path.

**Root cause: two skipped semantic maps.**
- `try_dispatch_builtin_call` (`eval_dispatch/mod.rs`) applies
  `result.map(Some)` after M0287/M0288. F0278's resume, however, completes
  with the raw value.
- `try_apply_function` (`scoping/callable.rs`) applies `.map(Some)` after
  its macro site (M0563). F0528's resume completes with the raw
  `finish_function_application` value.
- The legacy `Option<Expr>` decoder's lenient arm (`other => Some(other)`,
  `suspension/primitive.rs`) hides both, until a guest value carries an
  `InternalContinuation*` head.

The Fold/Nest path crosses both. The design table assigned all six
promotions to S3, and that was wrong: S3 owns only the first map.

**Rulings.**

1. **Reassign the six to S13.** The `fold`/`nest` promotions move to S13,
   the step that types F0528/M0563 and must make its resume execute the
   same `.map(Some)`. The director records the move in `TODO.md` and in
   S13's brief.
   - Remove the six fixtures from the oracle suite now: a test that is red
     by design cannot land.
   - Keep their code and your recorded pre-conversion output under
     `.temp/brief-609/` (for example `fold-nest-for-S13.patch` and
     `fold-nest-pre-conversion.txt`) for S13 to reuse.
2. **S3 owns its own map.** F0278's typed resume must execute the same
   `.map(Some)` as the ordinary path; a semantic map runs identically on
   both paths.
   - Add strict-equality fixtures that isolate it. Each is a builtin
     completion carrying a guest value under each internal head (the
     `none`, `some` and `empty_some` shapes). The resumed value must reach
     the stage consumer through F0278, with no frame of another family on
     the path that skips a map.
   - Candidates are ordinary builtins that evaluate guest code internally
     (`If`, `Catch`, `Block`, …). Choose by trace and state each fixture's
     frame path.
   - Watch the fixtures fail before converting F0278: production is still
     byte-identical to entry, so run them now. Then show them pass after.
     A fixture that passes before the conversion does not isolate the map;
     replace it.
3. **Audit S3's maps as a class.**
   - For every owned macro site, list the transformation between the
     witness point and the function's return on the ordinary path: `map`,
     `?`, wrapping, `Some`, `SharedExpr::new`, and so on. Show that the
     resumed path executes the same transformation. T0024 and the F0278
     `Some` are two instances.
   - An owned site whose resumed path skips a semantic map gets the same
     fix and its own watched-to-fail fixture.
   - A skipped map at a site S3 does not own goes in the report: file,
     line, owning step and the skipped transformation. Do not fix it; the
     director carries it to that step's brief.
4. **Everything else in the brief stands**, including the step-1 bases
   you already built. Do not rebuild them.

Append a new `## Execution report` after this section.

## Execution report — redispatch 1: STOP on the new witnessed-token host-payload row

**S3 is a partial candidate, not an accepted step.** The six replacement S3 marker cases were watched to fail on byte-identical S2 production code in both builds, then passed after conversion. The two ordinary two-pause controls also passed in the no-default run. Independent ownership discovery subsequently added an unapproved `UnknownHostPayload` for the new public witnessed-token method. Step 9 explicitly requires STOP. No disposition was invented and no inventory gate was weakened. The source-census receipt migration, broader tests and performance gates remain unfinished.

### 0. Evidence from the accepted pre-amendment STOP

Before redispatch 1, all four S2 bases were archived before edits, and the exact six Fold/Nest sources failed strict equality in both builds. The path is `Fold` (`functional_ops/sequence/fold/plain.rs:45`) / `Nest` (`functional_ops/iteration.rs:269`) → Function stage (`eval_dispatch/mod.rs:156` at S2 entry) → `try_apply_function` (`scoping/callable.rs:60–74`) → F0528/M0563, owned by S13. The ordinary function tail uses `.map(Some)`; F0528 completes `finish_function_application(outcome)` at `scoping/callable.rs:150–151` without it. The Legacy `Option<Expr>` decoder (`suspension/primitive.rs:46–66`) reads raw guest None/Some[5]/Some[] as None/Some(5)/None. This is independent of F0278’s S3 map. The director accepted that STOP and reassigned these six cases to S13. Their suite definitions were removed; their exact code, patch and raw outputs are preserved at `.temp/brief-609/fold-nest-fixtures-for-S13.rs`, `fold-nest-for-S13.patch`, and `fold-nest-pre-conversion.txt`. F0528/M0563 remain unchanged.

**pre-divergences-default: exit 101.** Raw log: `.temp/brief-609/pre-divergences-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.67s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 6 tests
test oracle::fixtures::evaluation_dispatch::s3_fold_empty_some ... fold.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
fold.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_fold_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
fold.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_fold_none ... fold.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
fold.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_fold_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
fold.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_fold_some ... fold.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
fold.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_fold_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
fold.some: Value
FAILED
test oracle::fixtures::evaluation_dispatch::s3_nest_empty_some ... nest.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
nest.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=60

thread 'oracle::fixtures::evaluation_dispatch::s3_nest_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
nest.empty_some: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_nest_none ... nest.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
nest.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=72

thread 'oracle::fixtures::evaluation_dispatch::s3_nest_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
nest.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_nest_some ... nest.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
nest.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=96

thread 'oracle::fixtures::evaluation_dispatch::s3_nest_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
nest.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_fold_empty_some
    oracle::fixtures::evaluation_dispatch::s3_fold_none
    oracle::fixtures::evaluation_dispatch::s3_fold_some
    oracle::fixtures::evaluation_dispatch::s3_nest_empty_some
    oracle::fixtures::evaluation_dispatch::s3_nest_none
    oracle::fixtures::evaluation_dispatch::s3_nest_some

test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.06s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**pre-divergences-no-default: exit 101.** Raw log: `.temp/brief-609/pre-divergences-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.11s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 6 tests
test oracle::fixtures::evaluation_dispatch::s3_fold_empty_some ... fold.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
fold.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_fold_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
fold.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_fold_none ... fold.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 661, identity: None, text: SymbolText { id: 661, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
fold.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_fold_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
fold.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_fold_some ... fold.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
fold.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_fold_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
fold.some: Value
FAILED
test oracle::fixtures::evaluation_dispatch::s3_nest_empty_some ... nest.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
nest.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=60

thread 'oracle::fixtures::evaluation_dispatch::s3_nest_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
nest.empty_some: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_nest_none ... nest.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 661, identity: None, text: SymbolText { id: 661, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
nest.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=72

thread 'oracle::fixtures::evaluation_dispatch::s3_nest_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
nest.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_nest_some ... nest.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
nest.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=96

thread 'oracle::fixtures::evaluation_dispatch::s3_nest_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:19:50:
nest.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_fold_empty_some
    oracle::fixtures::evaluation_dispatch::s3_fold_none
    oracle::fixtures::evaluation_dispatch::s3_fold_some
    oracle::fixtures::evaluation_dispatch::s3_nest_empty_some
    oracle::fixtures::evaluation_dispatch::s3_nest_none
    oracle::fixtures::evaluation_dispatch::s3_nest_some

test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.06s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

### 1. Files changed and pin/ripple disclosures

| File | Semantic change / old → new |
| --- | --- |
| crates/rum-runtime/src/runtime/builtin_adapters.rs | M0205 legacy macro → witnessed typed macro; identity success tail retained. |
| crates/rum-runtime/src/runtime/builtin_adapters/resume_frames.rs | F0204 legacy frame → TypedFrame<Expr, Expr>. |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | T0023 wrapper removed; initial witnessed capture; deferred helpers return ResumeStop<PreparedCallArguments>, advance fresh cursor/splice state, and use the witnessed token. Pure completion now returns PreparedCallArguments; its existing Legacy wire provider consumes those fields directly instead of freeze/thaw round-tripping them. The test-only old-wire fixture follows that helper’s typed result. |
| crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs | F0273 merges into F0272 so every repeat freezes fresh state of the same nominal type. F0570 becomes SlotFinalizer<Output=PreparedCallArguments>; finish_work schedules it after originating owners drop. |
| crates/rum-runtime/src/runtime/eval_core/call.rs | T0024 witnesses Expr before head sharing; shared resume_call_after_head performs the conversion. T0025/T0026 add PreparedDispatchFrame and share argument normalization/restore installation. F0275/F0276 consume Option<Expr>; their ordinary stage-result macro is shared in finish_call_dispatch_stage. Remove both shape-dependent old decoder branches. Stable SharedExpr heads retain their direct path. Profiling stop remains after ordinary preparation normalization and before returning a suspension. |
| crates/rum-runtime/src/runtime/eval_core/mod.rs | The SharedExpr-result profiling wrapper loses its final caller under T0024 and is removed; generic outcome classification remains. |
| crates/rum-runtime/src/runtime/eval_core/symbol.rs | Only F0277/M0285 typed: CompiledMatchOutcome → Expr, retaining matched/unmatched semantics and deleting the Expr wire map. The S14 loader capture in this file is unchanged. |
| crates/rum-runtime/src/runtime/eval_dispatch/mod.rs | Remove the generic PrepareArguments work and leave an unreachable arm. M0286 now witnesses the actual threaded Result instead of receiving only a request in a cold helper. M0287/M0288 use typed macros, preserving profiling on suspension. F0278 completes resumed.map(Some) after the existing memory-growth validation. |
| crates/rum-runtime/src/runtime/hold_thread.rs | F0279/M0289 typed Expr identity continuation for ReleaseHold. |
| crates/rum-runtime/src/runtime/resolution.rs | M0561/M0562 witnessed at the actual next_result before matching; saved current, pass count and history retain their transitions. |
| crates/rum-runtime/src/runtime/resolution/resume_frames.rs | F0526/F0527 typed Expr → Expr, retaining outcome? and resume_fixed_point_from. |
| crates/rum-runtime/src/runtime/session/session_eval.rs | Two callers: Err(self.continuation_fault_outcome(...).unwrap_err()) → Err(self.continuation_fault_outcome(...)). |
| crates/rum-runtime/src/runtime/session/session_requests.rs | Two callers: remove unwrap_err from the cold latch projection, retaining the same hot discriminant check. |
| crates/rum-runtime/src/runtime/suspension/core/fault.rs | Cold continuation_fault_outcome: Result<T, EvalError> → EvalError; still consumes the outcome and takes the same latch. |
| crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs | Adds public suspend_witnessed with SlotWitness<F::Input>, request and fresh F, delegating to the existing same-frame token operation. This exposes the undisposed public generic input that triggered STOP. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | Pin/discovery follows the owned trait renames: scan legacy and typed frame/finalizer traits, retain frame count 564, and check the single S3 typed finalizer’s exact Session-free PreparedCallArguments signature. These late test edits have not been compiled or run. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs | In both derived-root and pure-abandon arrays: eval_call_prepare/resume_frames.rs:ResumeSuspendedCallArgumentPreparationSuspensionFrame → eval_core/call.rs:PreparedDispatchFrame. This pairs the D0001 merge with the T0025/T0026 split consumer. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs | New strict If-held and listable-completion cases for three marker shapes, plus prepared.twice/head.map.twice controls. Checks::ALL and unconditional FaultProbe checks retained. Fold/Nest definitions archived for S13. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | No module → ordinary sibling mod evaluation_dispatch; no ../ path attribute. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | Provisional label S2 → S3; exercised by marker-case output. This is NOT an earned S3 census receipt: the receipt inventory and census acceptance logic are still S2 and remain red. |
| crates/rum-stdlib/tests/typed_slot_census.rs | Only scratch paths .temp/brief-608/census → .temp/brief-609/census; the S3 acceptance/forced-red expectation migration is unfinished. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | Only emitted scratch path brief-608 → brief-609. It correctly failed closed on a stale removed decoder-use receipt. |
| crates/rum-stdlib/tests/typed_slot_compile_controls.rs | Only generated scratch path brief-608/compile-controls → brief-609/compile-controls; the 30 control pairs are unchanged and have not been rerun. |
| This brief | Append this report after the director amendment; preserve all preceding bytes. |

No other constructor ripple or production module visibility/re-export change was made. The new fixture is 4,327 bytes. The protected-input audit verifies all 183 design/harness hashes, all four archived bases, every manifest/lock/toolchain input, AGENTS.md/TODO.md, the mode registry and the S13 Function implementation. `.temp/brief-609/r1-candidate.patch` is a complete byte-snapshot diff without git; `r1-stop-audit.json` records final source hashes. Scratch stays under brief-609; no certificate output was produced in this redispatch.

Size assessment: builtin_adapters.rs remains about 23.5k with a macro-only change; future separation is by builtin adapter family. eval_call_prepare.rs is about 23.3k and still combines ordinary preparation, suspended accumulation and the existing wire provider; that is the future split boundary. eval_core/call.rs is about 21.5k after the required typed consumer split; dispatch planning/ordinary loops versus resume consumers is the next coherent boundary. This candidate keeps the owned conversion together and does not add unrelated splits. Existing symbol/dispatch/resolution/session files stay in their existing warning or split-plan ranges with only the scoped changes listed above. The large derived inventory is existing literal data; frame_root_trace.rs is a 35k test module. No new module exceeds 10k.

### 2. Owned rows, transformation audit and partial census

| F row | Baseline name | Candidate transition |
| --- | --- | --- |
| F0204 | BuiltinEvaluateSuspensionFrame | Typed Expr → Expr; same name. |
| F0272 | PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame | Typed Expr → PreparedCallArguments; also receives fresh repeated state formerly handled by F0273. |
| F0273 | ResumeSuspendedCallArgumentPreparationSuspensionFrame | Merged into F0272; no remaining production implementation of the old name. |
| F0274 | EvaluateCallSuspensionFrame | Typed Expr → Expr; head conversion at the shared entry. |
| F0275 | ResumeCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch split to PreparedDispatchFrame. |
| F0276 | ResumeSuspendedCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch uses the same PreparedDispatchFrame. |
| F0277 | EvaluateStoredOwnValueSuspensionFrame | Typed CompiledMatchOutcome → Expr. |
| F0278 | RunCallDispatchStageSuspensionFrame | Typed Expr → Option<Expr>; complete the semantic Some map. |
| F0279 | EvalReleaseHoldSuspensionFrame | Typed Expr → Expr. |
| F0526 | EvaluateProfiledFixedPointSuspensionFrame | Typed Expr → Expr. |
| F0527 | ResumeFixedPointFromSuspensionFrame | Typed Expr → Expr. |
| F0570 | FinishPreparedCallArgumentsFinalizer | SlotFinalizer Output=PreparedCallArguments. |

New split frame: `PreparedDispatchFrame`, Input=PreparedCallArguments, Output=Expr, shared by the ordinary and already-suspended dispatch loops. The F0273 merge (-1) and this added consumer (+1) preserve the implementation total. No State birth or mode flip occurred; Expr remains Value and all existing non-Expr slots retain Legacy. No dual-arm decode, per-frame override or raw public adapter was added.

| M row | Actual producer before maps | Ordinary transformation/tail | Resumed transformation / transition |
| --- | --- | --- | --- |
| M0205 | Expr | builtin_evaluate returns the evaluated value unchanged | Typed identity frame; no post-witness semantic map. |
| M0279 | Expr | append/splice/drop-Nothing; preserve Unevaluated entries; advance remaining arguments; build PreparedCallArguments | Typed F0272 reentry uses the same append helper and fresh cursor/splice fields; pure finalizer builds the exact output after Arc release. |
| M0280 | Expr | the same append and cursor/restore work for later arguments | Old macro removed by D0001; witness of actual evaluated_result is passed to token.suspend_witnessed with fresh F0272. No original Arc is captured. |
| M0281 | Expr | SharedExpr conversion, plan construction and dispatch; profiling records | Witness now at evaluate(head.into_owned()); shared resume_call_after_head accepts Expr or existing SharedExpr and performs Into<SharedExpr> once. |
| M0282 | PreparedCallArguments or Option<Expr> (split) | normalize prepared args, install restore; stage Some returns value, None advances cursor | Prepare macro → PreparedDispatchFrame; stage macro → shared finish_call_dispatch_stage, F0275. Shared normalization/restore helper runs once. |
| M0283 | PreparedCallArguments or Option<Expr> (split) | same work with the suspended plan and frozen restore | Prepare macro → PreparedDispatchFrame; shared stage macro → F0276; restore freezes after the same helper. |
| M0285 | CompiledMatchOutcome | matched branch evaluates own value; unmatched branch resumes symbol lookup | Same branches in F0277; only the Expr transport map and raw input decode disappear. |
| M0286 | Expr | Some(value) after evaluating the threaded call | F0278 now completes resumed.map(Some); three listable fixtures watched fail → pass. |
| M0287 | Expr | profile the builtin result, then result.map(Some) | Same Some completion in unchecked F0278; suspension records the existing profiling event before the typed macro returns. |
| M0288 | Expr | profile, memory-growth validation, then result.map(Some) | Checked F0278 validates the Expr result and then wraps Some. The If-held fixtures exercise the checked test build; the unchecked constructor shares this completion body. Its release path has not yet been measured. |
| M0289 | Expr | ReleaseHold evaluation result unchanged | Typed identity frame. |
| M0561 | Expr | fixed-point equality/cycle history/trace update and next pass | Typed F0526 uses outcome? then the unchanged resume_fixed_point_from state transition. |
| M0562 | Expr | same fixed-point transition in resumed evaluation | Typed F0527 uses outcome? then the same helper. |

The 13 baseline M rows reconcile to 13 live typed macro sites: M0280 becomes a witnessed token call (-1 macro); M0282/M0283 split into two preparation sites plus one shared Option-stage site (+1 macro). The shared generic helper owns only a synchronous Rust payload and constructs its frame in the suspension arm. Output is bound by its Result<Expr, EvalError> return. No synthetic Err supplies the input witness. The broader source receipt for these splits is still pending; this table is a source audit, not an earned gate result.

| Transformation / obligation | Candidate state |
| --- | --- |
| T0023 | Generic PrepareArguments work and unused prepare_arguments_for_call wrapper removed. The dedicated no-caller source assertion is not yet added. |
| T0024 | Witness actual Expr Result before sharing; common entry performs conversion. head.map.twice passes in the no-default focused gate. |
| T0025 / T0026 | Separate PreparedCallArguments and Option<Expr> consumers; shared normalization and restore installation. prepared.twice passes in the no-default focused gate. |
| T0027 | Delete only the Expr transport map/raw CompiledMatchOutcome decode; exact typed helper result retained. The full owning oracle descriptor has not yet run. |
| D0001 | Same nominal F0272 token, fresh cursor/splice state, no captured Arc; typed deferred finalizer after driver drop. Production two-pause case passes; dedicated typed driver/accounting gates not rerun. New witnessed public API triggered the STOP. |

The out-of-scope skipped semantic map found on the original paths remains `scoping/callable.rs:150–151`, F0528/M0563, S13: `.map(Some)` after `finish_function_application`. It is disclosed in section 0 and was not repaired. The owned-site audit above found the shared F0278 Some omission; no global audit of all other families is claimed.

Partial discovery wrote frames.tsv, macro-sites.tsv, definitions.tsv, all-uses.tsv and related files under `.temp/brief-609/census/default/` before failing during emission of old decoder receipts. Counting the actual emitted production records gives:

```text
production implementations=607: SuspensionFrame=542, PureFrameFinalizer=53, TypedFrame=11, SlotFinalizer=1
production macro sites=589: legacy=576, typed=13; result arms=353, insert arms=236
```

These counts are parsed from discovery artifacts, not a passing S3 census. `typed-slot-stages.tsv`, `typed-slot-shapes.tsv`, census acceptance and forced-red expectations were not updated. The provisional fixture label is S3, while earned stage receipts still stop at S2. The stale source anchors awaiting owned reconciliation are:

| Kind | Receipt | Pending transition |
| --- | --- | --- |
| Transform | T0023 | Owned transform replaced/removed; full identity saved in r1-pending-receipt-transitions.json |
| Transform | T0024 | Owned transform replaced/removed; full identity saved in r1-pending-receipt-transitions.json |
| Transform | T0025 | Owned transform replaced/removed; full identity saved in r1-pending-receipt-transitions.json |
| Transform | T0026 | Owned transform replaced/removed; full identity saved in r1-pending-receipt-transitions.json |
| Transform | T0027 | Owned transform replaced/removed; full identity saved in r1-pending-receipt-transitions.json |
| Whole | Whole0029 | Owned raw decoder use removed by the typed consumer; no inventory removal performed |
| Whole | Whole0030 | Owned raw decoder use removed by the typed consumer; no inventory removal performed |
| Whole | Whole0031 | Owned raw decoder use removed by the typed consumer; no inventory removal performed |
| Whole | Whole0032 | Owned raw decoder use removed by the typed consumer; no inventory removal performed |

### 3. Replacement S3 divergence fixtures and ordinary controls

The retained builtin fixture selects an If held branch which suspends in CompoundExpression. Source path: InputString → its builtin/dispatch/fixed-point frames (ordinary string) → ResumeCompoundExpressionFromIndexSuspensionFrame (Expr) → F0278 for CompoundExpression (missing Some) → F0275 Option consumer → fixed-point frame → F0278 for If → F0275 → fixed point. Both omitted maps on that path belong to F0278/S3; the CompoundExpression continuation returns Expr on both paths. No Function frame is involved.

The retained listable fixture installs a List[1] DownValue only after the scalar callback pauses. Source path: the scalar downvalue body resumes to ordinary integer 1 → the completed threaded List is ordinarily rewritten to the marker → F0278::threaded_call → Option stage consumer. Other paused definition frames see integer 1, not the marker; the marker-producing List rule runs synchronously during resume. The observed post-conversion equality supports that isolation.

| Cases | Source template (SHAPE = None[] / Some[5] / Some[]) | Observed S2 failures in each build |
| --- | --- | --- |
| builtin.none / builtin.some / builtin.empty_some | If[True, (InputString["sweep"];System`InternalContinuationSHAPE), 0] | UnexpectedPause(1) / Value (5) / UnexpectedPause(1) |
| threaded.none / threaded.some / threaded.empty_some | SetAttributes[f,Listable]; f[x_] := (InputString["sweep"]; Unprotect[List]; DownValues[List]={HoldPattern[List[1]] :> System`InternalContinuationSHAPE}; 1); f[{1}] | UnexpectedPause(1) / Value (5) / UnexpectedPause(1) |

Every retained case uses Checks::ALL, compares value/log/messages/prints/trace, checks the planned request and exhausted queue, and executes FaultProbe absence unconditionally before the equality assertion. Each pre-conversion group of three ends at 48 zero-fault observations. Default post-conversion six-case run ends at 72; no-default eight-case run ends at 96. All deltas are zero. The two ordinary controls pin expected results/log and prepared.twice additionally pins hits=1; all 28 ordinary controls were NOT run before the ownership STOP.

**r1-pre-builtin-held-default: exit 101.** Raw log: `.temp/brief-609/r1-pre-builtin-held-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.29s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-builtin-held-no-default: exit 101.** Raw log: `.temp/brief-609/r1-pre-builtin-held-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.13s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 661, identity: None, text: SymbolText { id: 661, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-default: exit 101.** Raw log: `.temp/brief-609/r1-pre-threaded-dv-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.35s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 673, identity: None, text: SymbolText { id: 673, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-no-default: exit 101.** Raw log: `.temp/brief-609/r1-pre-threaded-dv-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.46s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-post-map-1: exit 0.** Raw log: `.temp/brief-609/r1-post-map-1.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 36.43s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 6 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=12
S2 builtin.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=24
S2 builtin.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=36
S2 builtin.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=48
S2 threaded.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=60
S2 threaded.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=72
S2 threaded.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.04s
```

**r1-post-map-no-default: exit 0.** Raw log: `.temp/brief-609/r1-post-map-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch -- --nocapture
```

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.22s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 8 tests
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=12
ok
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=24
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=36
S3 builtin.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=48
S3 builtin.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 677, identity: None, text: SymbolText { id: 677, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=60
S3 builtin.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=72
S3 threaded.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=84
S3 threaded.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 677, identity: None, text: SymbolText { id: 677, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=96
S3 threaded.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.06s
```

### 4. Gates, ownership STOP, warnings and unfinished inventories

All Cargo commands ran serially under the bounded wrapper with CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1, CARGO_NET_OFFLINE=true and TMPDIR=/home/remi/dev/rum/.temp/brief-609/tmp. Filtered fixtures used the 900 s “everything else” bound; no full oracle gate was launched. The wrapper additionally allows the repository’s native-ZeroMQ skip. None of these focused runs executed native ZeroMQ tests; no such skip is counted as a pass.

Fixture selection evidence is preserved, including rejected candidates. The first If-condition candidate passed because If is Ordinary/HoldRest: argument preparation suspends before its builtin starts. It was replaced, not accepted. The first listable candidate tried Set[List[1], marker], which invokes destructuring and emits Set::shape in both paths; its ExpectedValue failures did not isolate the defect. It was replaced with DownValues installation, then genuine queued/resumed divergences were observed.

**r1-pre-builtin-default: exit 0.** Raw log: `.temp/brief-609/r1-pre-builtin-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.38s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=12
S2 builtin.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=24
S2 builtin.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 668, identity: None, text: SymbolText { id: 668, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=36
S2 builtin.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.03s
```

**r1-pre-threaded-default: exit 101.** Raw log: `.temp/brief-609/r1-pre-threaded-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.53s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... expected=Some("InternalContinuationSome[]") actual=Some("List[1]")
threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(1)))]) }), error: None, termination: None, messages: [RuntimeMessage { symbol: "Set", tag: "shape", text: "Lists List[1] and InternalContinuationSome[] are not the same shape." }], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(1)))]) }), error: None, termination: None, messages: [RuntimeMessage { symbol: "Set", tag: "shape", text: "Lists List[1] and InternalContinuationSome[] are not the same shape." }], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: ExpectedValue
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... expected=Some("InternalContinuationNone[]") actual=Some("List[1]")
threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(1)))]) }), error: None, termination: None, messages: [RuntimeMessage { symbol: "Set", tag: "shape", text: "Lists List[1] and InternalContinuationNone[] are not the same shape." }], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(1)))]) }), error: None, termination: None, messages: [RuntimeMessage { symbol: "Set", tag: "shape", text: "Lists List[1] and InternalContinuationNone[] are not the same shape." }], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: ExpectedValue
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... expected=Some("InternalContinuationSome[5]") actual=Some("List[1]")
threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(1)))]) }), error: None, termination: None, messages: [RuntimeMessage { symbol: "Set", tag: "shape", text: "Lists List[1] and InternalContinuationSome[5] are not the same shape." }], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([Atom(Integer(Small(1)))]) }), error: None, termination: None, messages: [RuntimeMessage { symbol: "Set", tag: "shape", text: "Lists List[1] and InternalContinuationSome[5] are not the same shape." }], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=72

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: ExpectedValue
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-census-discover: exit 101.** Raw log: `.temp/brief-609/r1-census-discover.log`.

```text
cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.90s
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-943817a3ecf6c029)

running 1 test
test source_baseline_matches_earned_receipt ... census walk: 14946 functions, 299598 uses, 1499 module visits
census resolver: 34898 scopes

thread 'source_baseline_matches_earned_receipt' panicked at crates/rum-stdlib/tests/typed_slot_census/reporting.rs:290:22:
recorded decoder use must be rediscovered
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    source_baseline_matches_earned_receipt

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 20.77s

error: test failed, to rerun pass `-p rum-stdlib --test typed_slot_census`
```

**r1-ownership-discover: exit 0.** Raw log: `.temp/brief-609/r1-ownership-discover.log`.

```text
cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 37.67s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 5.84s
```

**Mandatory STOP:** independent discovery has 80 UnknownHostPayload rows versus the existing 79. Comparing file/construct identities while normalizing only source-position suffixes finds exactly one added identity and no removed identities:

```text
UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs	20	16	public input rum_runtime::runtime::suspension::typed::resume_token::impl(ContinueToken < '_ , F >)@13::suspend_witnessed(fresh): opaque owned generic parameter F : TypedFrame
```

The added public input is `fresh: F` at `suspension/typed/resume_token.rs:20:16`, bound by `F: TypedFrame`. The method consumes a SlotWitness<F::Input> from the real repeated producer so D0001 can retain producer evidence, but its new public by-value generic parameter creates a distinct host-payload obligation. The existing `suspend(fresh)` disposition does not automatically approve this new method. Step 9 says: “A new UnknownHostPayload without an approved disposition is a STOP.” No new disposition, broad allowlist or discovery modification was made. A director decision can require narrower runtime-only visibility or explicitly authorize/discharge the public operation; I made no implementation repair after this discovery.

**fmt-check: exit 0.** Raw log: `.temp/brief-609/fmt-check.log`.

```text
cargo fmt --check
```

```text
(no output)
```

**r1-fmt-write: exit 0.** Raw log: `.temp/brief-609/r1-fmt-write.log`.

```text
cargo fmt
```

```text
(no output)
```

**r1-fmt-write-2: exit 0.** Raw log: `.temp/brief-609/r1-fmt-write-2.log`.

```text
cargo fmt
```

```text
(no output)
```

**r1-final-format: exit 0.** Raw log: `.temp/brief-609/r1-final-format.log`.

```text
cargo fmt
```

```text
(no output)
```

**r1-final-fmt-check: exit 0.** Raw log: `.temp/brief-609/r1-final-fmt-check.log`.

```text
cargo fmt --check
```

```text
(no output)
```

Observed compiler warnings are the pre-existing unused FLIP_SLOT and unused support::drive::execute warnings in the oracle target. No new warning appeared in executed builds. TYPING_RECEIPT is now used by the fixture output. The initial conversion and the profiling-corrected production code compiled; there were zero Rust compiler-error attempts. The late frame_root_trace test/inventory edits have only been formatted, not compiled or tested.

**Not run after the STOP:** runtime typed unit suite; frame_root_trace; both 30-pair compile-control suites; full typed census in either build; full suspension oracle in either build; ownership exactness/ledger acceptance; both local production certificate suites; historical retention census; workspace tests; all-28 ordinary control replay. The only census attempt failed on stale owned decoder receipts, and ownership discovery itself succeeded while exposing the unapproved row. No correctness acceptance is claimed.

No ownership inventory, host disposition, retention fingerprint, promotion/certificate cell, oracle effect/catalog pin or historical rejection-census pin was replayed. Their hashes/positions may therefore be stale for this partial candidate. `Whole0029–Whole0032` and T0023–T0027 are explicitly pending owned transitions, not silently deleted rows. Certificates were not emitted. The two derived frame inventory substitutions are listed in section 1 and remain untested.

### 5. Bases and performance evidence

The four fresh one-codegen-unit S2 bases from step 1 were reused exactly as directed; none was rebuilt in redispatch 1. Their hashes were reverified during the final audit. Each was built in its own brief-609 target. Existing brief-608 resume-harness sources are unchanged; feature trees exclude test-continuation-slots.

| Binary | Bytes | SHA-256 |
| --- | --- | --- |
| rum-cli | 12537440 | b553a00e63e8af387e3141fe35764f54515a3e3154fd09b2756f911a2e87d9bf |
| rum-cli-alloc | 12547616 | f1fb356eb26f45dedaa09ad46e8d697dfde6bd10fbfe1aa12290f61dfcf6e9f1 |
| resume | 12180552 | 57e800bca2cca0d327b358f410d34ced30787410bdee6afe5838f9334a24e81f |
| resume-alloc | 12185720 | 73f279bfb5e13ab4daa34242a9d591e52d9d8012267197268e80f63d41ecc503 |

**base cli: exit 0.**

```text
cargo build --release -p rum-cli
CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1
CARGO_NET_OFFLINE=true
TMPDIR=/home/remi/dev/rum/.temp/brief-609/tmp
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-609/target-base-cli
```

```text
    Finished `release` profile [optimized] target(s) in 1m 41s
```

**base cli-alloc: exit 0.**

```text
cargo build --release -p rum-cli --features rum-cli/alloc-count
CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1
CARGO_NET_OFFLINE=true
TMPDIR=/home/remi/dev/rum/.temp/brief-609/tmp
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-609/target-base-cli-alloc
```

```text
    Finished `release` profile [optimized] target(s) in 1m 38s
```

**base resume: exit 0.**

```text
cargo build --release --manifest-path /home/remi/dev/rum/.temp/brief-608/resume-harness/Cargo.toml
CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1
CARGO_NET_OFFLINE=true
TMPDIR=/home/remi/dev/rum/.temp/brief-609/tmp
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-609/target-base-resume
```

```text
    Finished `release` profile [optimized] target(s) in 1m 32s
```

**base resume-alloc: exit 0.**

```text
cargo build --release --manifest-path /home/remi/dev/rum/.temp/brief-608/resume-harness/Cargo.toml --features alloc-count
CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1
CARGO_NET_OFFLINE=true
TMPDIR=/home/remi/dev/rum/.temp/brief-609/tmp
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-609/target-base-resume-alloc
```

```text
    Finished `release` profile [optimized] target(s) in 1m 33s
```

No candidate release binaries were built. All 64 allocation/byte comparisons, interleaved instruction slopes, flagged-label repeats/Callgrind, five resume-workload function/address attribution and the full timing panel remain unrun. This conversion changes hot paths and Legacy wire work; no no-cost or permitted-allocation-delta claim is made from the successful debug fixtures. The brief’s performance requirements remain outstanding.

### 6. S2 cold-helper leftover

continuation_fault_outcome remains #[cold] and #[inline(never)], consumes Result<T, EvalError>, and now returns EvalError directly. All four production callers retain their original hot has_continuation_fault discriminant check and wrap the returned error with Err. Only the two specified test callers changed. This compiled with the candidate, but the mandatory address-level hot-boundary comparison and the typed fault tests have not run; source similarity is not measurement evidence.

### 7. Stop-loss counters, deviations and open decisions

Stop-loss counter: 0. The pre-conversion red tests were required evidence; fixture selection progressed to actual divergences. The first conversion compiled without Rust errors and made six named failures pass. The next focused build passed all eight cases. The source census reached actual 607/589 discovery and exposed stale owned receipts; independent ownership discovery then triggered an explicit STOP. This is not the two-unproductive-attempt stop-loss condition.

The governing amendment was read before redispatch work. The original six promotions were reassigned only under that explicit ruling. No git command, network operation, unsafe, dependency/lock/toolchain edit, out-of-scope family conversion or comparison weakening occurred. All source/protected-input audits are snapshot/hash based. The only production API addition was disclosed and is exactly the new ownership blocker. All launched commands completed. The final formatting check passes.

Open decision: resolve the public witnessed-token obligation before resuming. A narrower visibility/API design may avoid creating a new public host-input row; otherwise the public operation needs an approved disposition and evidence. Then complete the S3 census/source gate, test and inventory replay, all mandatory gates and measurements. The working tree is deliberately left uncommitted and **not acceptance-ready**. Partial files are concrete and preserved: provisional S3 fixture label, unchanged S2 receipt inventory/acceptance logic, pending decoder transitions, untested late frame-root pins, and unreplayed certificates/ownership. `.temp/brief-609/PROGRESS.md` records the exact next action and no hidden running command remains.


## Director amendment — redispatch 2: one witnessed token route

The ownership STOP is correct. The cause is the API shape, not a missing
disposition.

1. **No second public route.** `ContinueToken::suspend` itself takes the
   witness:
   `suspend(self, session, witness: SlotWitness<F::Input>, request, fresh)`.
   Remove `suspend_witnessed`.
   - With two routes, the witness is a convention a caller can skip. With
     one, the compiler requires producer evidence at every repeat site,
     which is what the design's "input evidence comes from the actual
     `Result`" means.
   - The existing ledger row for `suspend(fresh)` keeps its identity.
     Discovery must return to 79 `UnknownHostPayload` rows, differing only
     in position. If a new identity still appears, STOP.
   - Update S2's token tests under `suspension/typed/tests/` to pass a
     witness from their producer.
   - Update the I0003 compile-control pair the same way. Its negative still
     isolates exactly one invalidity: the fresh-state type.
   - Add one pair: a witness whose input type is not `F::Input` must be
     rejected. Pin its error code and primary span. The controls become 31
     in both builds.
   - The census route I0003 still names `ContinueToken::suspend`. Update its
     witness text if it quotes the signature.
   - Authorized for this: `typed_slot_compile_controls.rs` and the files
     under `suspension/typed/`.
2. **Oracle warnings.** At S2 the oracle target warned three times:
   `TYPING_RECEIPT`, `FLIP_SLOT` and `support::drive::execute`. Step 7 now
   means the oracle target compiles with no warning in either build.
   - Scope a constant that only the census reads.
   - Remove `drive::execute` if nothing calls it in either build; say which.
3. **Then finish the step.** Do the census receipt migration, the
   inventory, fingerprint and certificate replays, every gate in the
   Verification section, and the performance section. Compile and test the
   derived frame inventory edits; they are untested so far.

Append a new `## Execution report` after this section.


## Execution report — redispatch 2: STOP on an unpaired retention-frontier removal

**The witnessed-token ownership blocker is resolved; S3 is still not acceptance-ready.** ContinueToken has one public suspend route and requires SlotWitness<F::Input>. Independent ownership discovery returns exactly 79 approved host-payload identities. The typed tests pass 20/20 and the previously untested frame-root edits pass 15/15. The historical retention-frontier replay then exposed an unpaired removed rejection. The Scope explicitly makes that a STOP. No checker, rejection pin, count or disposition was changed to accept it. Only formatting, evidence audit and this report followed discovery.

### 1. Files changed, ripple and pin disclosures

| File | Semantic change / old → new |
| --- | --- |
| crates/rum-runtime/src/runtime/builtin_adapters.rs | M0205 legacy macro → witnessed typed macro; identity success tail retained. |
| crates/rum-runtime/src/runtime/builtin_adapters/resume_frames.rs | F0204 legacy frame → TypedFrame<Expr, Expr>. |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | T0023 wrapper removed; initial witnessed capture; deferred helpers return ResumeStop<PreparedCallArguments>, advance fresh cursor/splice state, and use the witnessed token. Pure completion now returns PreparedCallArguments; its existing Legacy wire provider consumes those fields directly instead of freeze/thaw round-tripping them. The test-only old-wire fixture follows that helper’s typed result. Redispatch 2 renames the sole repeat call to token.suspend with its existing actual-producer witness. |
| crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs | F0273 merges into F0272 so every repeat freezes fresh state of the same nominal type. F0570 becomes SlotFinalizer<Output=PreparedCallArguments>; finish_work schedules it after originating owners drop. |
| crates/rum-runtime/src/runtime/eval_core/call.rs | T0024 witnesses Expr before head sharing; shared resume_call_after_head performs the conversion. T0025/T0026 add PreparedDispatchFrame and share argument normalization/restore installation. F0275/F0276 consume Option<Expr>; their ordinary stage-result macro is shared in finish_call_dispatch_stage. Remove both shape-dependent old decoder branches. Stable SharedExpr heads retain their direct path. Profiling stop remains after ordinary preparation normalization and before returning a suspension. |
| crates/rum-runtime/src/runtime/eval_core/mod.rs | The SharedExpr-result profiling wrapper loses its final caller under T0024 and is removed; generic outcome classification remains. |
| crates/rum-runtime/src/runtime/eval_core/symbol.rs | Only F0277/M0285 typed: CompiledMatchOutcome → Expr, retaining matched/unmatched semantics and deleting the Expr wire map. The S14 loader capture in this file is unchanged. |
| crates/rum-runtime/src/runtime/eval_dispatch/mod.rs | Remove the generic PrepareArguments work and leave an unreachable arm. M0286 now witnesses the actual threaded Result instead of receiving only a request in a cold helper. M0287/M0288 use typed macros, preserving profiling on suspension. F0278 completes resumed.map(Some) after the existing memory-growth validation. |
| crates/rum-runtime/src/runtime/hold_thread.rs | F0279/M0289 typed Expr identity continuation for ReleaseHold. |
| crates/rum-runtime/src/runtime/resolution.rs | M0561/M0562 witnessed at the actual next_result before matching; saved current, pass count and history retain their transitions. |
| crates/rum-runtime/src/runtime/resolution/resume_frames.rs | F0526/F0527 typed Expr → Expr, retaining outcome? and resume_fixed_point_from. |
| crates/rum-runtime/src/runtime/session/session_eval.rs | Two callers: Err(self.continuation_fault_outcome(...).unwrap_err()) → Err(self.continuation_fault_outcome(...)). |
| crates/rum-runtime/src/runtime/session/session_requests.rs | Two callers: remove unwrap_err from the cold latch projection, retaining the same hot discriminant check. |
| crates/rum-runtime/src/runtime/suspension/core/fault.rs | Cold continuation_fault_outcome: Result<T, EvalError> → EvalError; still consumes the outcome and takes the same latch. |
| crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs | Redispatch 2: delete suspend_witnessed; original suspend(session, request, fresh) → suspend(session, SlotWitness<F::Input>, request, fresh). Same public identity, same fresh F constraint, no Arc in the token. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs | The legacy-queue setup calls eval_input_string directly instead of evaluating an InputString call through newly typed S3 frames. This restores the legacy-to-mixed widening precondition; every allocation, byte and admission assertion is unchanged. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs | repeat_work now captures slot_witness(&outcome) from its actual input producer before matching and passes it to the sole suspend route. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | Scan both legacy and typed frame/finalizer traits; preserve 564-frame population and root/drop checks. Pin the Session-free PreparedCallArguments finalizer signature. Now compiled and tested: 15/15 frame-root tests pass. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs | In both derived-root and pure-abandon arrays: eval_call_prepare/resume_frames.rs:ResumeSuspendedCallArgumentPreparationSuspensionFrame → eval_core/call.rs:PreparedDispatchFrame. This pairs the D0001 merge with the T0025/T0026 split consumer. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs | New strict If-held and listable-completion cases for three marker shapes, plus prepared.twice/head.map.twice controls. Checks::ALL and unconditional FaultProbe checks retained. Fold/Nest definitions archived for S13. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | No module → ordinary sibling mod evaluation_dispatch; no ../ path attribute. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | S2 → S3 typing label, read by marker fixtures and census. Remove FLIP_SLOT from this shared oracle module; it is now scoped to the census target. The S3 label remains provisional until the complete gate passes. |
| crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv | Record every owned F/M/T/D transition; retain F0273/M0280 removal receipts, add F0275P/M0282S, mark four owned raw decoder uses removed, and add S3 stage. Add X0040 for the existing finalizer-wire fixture following its newly typed helper. Full old/new cells are retained in r2/census-receipt-transitions.json. Receipt migration has not earned a passing complete census. |
| crates/rum-stdlib/tests/suspension_oracle/support/drive.rs | Remove unused execute wrapper; execute_planned remains unchanged. No oracle caller exists in either cfg path. Historical suites call the distinct suspension_support::drive::execute, which remains unchanged. |
| crates/rum-stdlib/tests/typed_slot_census.rs | Scratch brief-608 → brief-609; S2 → S3 receipt selection and preflip-negative label. Scope FLIP_SLOT here and connect s3 module. Forced-red expected production legacy counts 564/59 → 553/58 after removing a test guard; combined implementation sentinel stays 622 after deleting one frame. These falsifiers were blocked by the stale X0040 receipt, so new expectations remain unverified. |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | Production legacy frames/finalizers 553/54 → 542/53, total 607 unchanged; macro result/insert 345/244 → 353/236, total 589 unchanged. Receipt row totals F623 → F624 and M600 → M601 retain merged rows plus new split identities. Closed-set S3 removal proof precedes skipping removed source anchors; all other exact source/mode checks remain. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | Scratch brief-608 → brief-609. Removed Whole0029–Whole0032 emit explicit S3 transition records, assert absence of the old identity, and preserve decoder IDs; live decoder rows still require rediscovery. |
| crates/rum-stdlib/tests/typed_slot_census/routes.rs | I0003 still names ContinueToken::suspend and its executable repeat witness. Update mechanism output to disclose typed S3 evaluation while later families remain untyped; S2 mechanism ownership is unchanged. |
| crates/rum-stdlib/tests/typed_slot_census/s3.rs | New closed-set S3 gate: exact 12 typed implementations and 13 macro identities, exact input/output pairs, eight explicitly approved removals, T0023 no-definition/no-caller source assertion, relocated maps, and live token repeat. This gate executed successfully before the outer census failed on X0040. |
| crates/rum-stdlib/tests/typed_slot_compile_controls.rs | Scratch brief-608 → brief-609. Add actual producer witnesses to fresh-state and deferred-output pairs. Add token-witness-input, changing only Expr → Option<Expr> for producer input and requiring one E0308 at the complete primary witness span. Pair count 30 → 31 in each build. These compiler pairs have not run in this redispatch. |
| docs/dev/codex/brief-609-g13-s3-evaluation-dispatch.md | Append this report after the redispatch-2 amendment, preserving the entire entry prefix byte-for-byte. |

There are 30 modified source/test/inventory files and two new Rust modules relative to the original S2 snapshot, plus the appended brief. No production module visibility/re-export edit was needed. The two derived inventory substitutions remain ResumeSuspendedCallArgumentPreparationSuspensionFrame → PreparedDispatchFrame in the derived-root and pure-abandon arrays; their gate now passes. The registry and typed-slot-shapes.tsv remain byte-identical: no mode or introducing-step change is needed because S3 has no State births. No ownership inventory, oracle effect/catalog owner pin, local-contract fingerprint, promoted certificate or historical rejection-census pin was edited.

Size assessment: the existing builtin adapters, argument preparation and call-dispatch modules remain in their documented split-plan ranges. Their future boundaries are adapter families, ordinary/suspended preparation versus wire provider, and dispatch planning/loops versus resume consumers. The existing large derived inventory remains literal data; the frame-root test still groups protocol and inventory checks. Compiler controls now contain 31 related positive/negative pairs; their future split is fixture definitions versus diagnostic harness. No unrelated split or TODO edit was made. Both new modules remain below 10k characters: evaluation_dispatch.rs=4327, s3.rs=5206.

### 2. Owned-row reconciliation and partial census

| F row | Baseline name | Candidate transition |
| --- | --- | --- |
| F0204 | BuiltinEvaluateSuspensionFrame | Typed Expr → Expr; same name. |
| F0272 | PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame | Typed Expr → PreparedCallArguments; also receives fresh repeated state formerly handled by F0273. |
| F0273 | ResumeSuspendedCallArgumentPreparationSuspensionFrame | Merged into F0272; no remaining production implementation of the old name. |
| F0274 | EvaluateCallSuspensionFrame | Typed Expr → Expr; head conversion at the shared entry. |
| F0275 | ResumeCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch split to PreparedDispatchFrame. |
| F0276 | ResumeSuspendedCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch uses the same PreparedDispatchFrame. |
| F0277 | EvaluateStoredOwnValueSuspensionFrame | Typed CompiledMatchOutcome → Expr. |
| F0278 | RunCallDispatchStageSuspensionFrame | Typed Expr → Option<Expr>; complete the semantic Some map. |
| F0279 | EvalReleaseHoldSuspensionFrame | Typed Expr → Expr. |
| F0526 | EvaluateProfiledFixedPointSuspensionFrame | Typed Expr → Expr. |
| F0527 | ResumeFixedPointFromSuspensionFrame | Typed Expr → Expr. |
| F0570 | FinishPreparedCallArgumentsFinalizer | SlotFinalizer Output=PreparedCallArguments. |

F0275P names the new PreparedDispatchFrame (PreparedCallArguments → Expr), shared by both preparation consumers. F0273 merges into F0272 for fresh same-type token repeats. M0280 becomes the witnessed token route; M0282/M0283 retain their preparation macros and M0282S names their shared Option-stage macro. The implementation and macro totals stay 607 and 589. The retained removal rows explain why the receipt inventory has one extra historical F row and one extra M row.

| M row | Actual producer | Ordinary transformation | Resumed transformation |
| --- | --- | --- | --- |
| M0205 | Expr | builtin_evaluate returns the evaluated value unchanged | Typed identity frame; no post-witness semantic map. |
| M0279 | Expr | append/splice/drop-Nothing; preserve Unevaluated entries; advance remaining arguments; build PreparedCallArguments | Typed F0272 reentry uses the same append helper and fresh cursor/splice fields; pure finalizer builds the exact output after Arc release. |
| M0280 | Expr | the same append and cursor/restore work for later arguments | Old macro removed by D0001; witness of actual evaluated_result is passed to token.suspend with fresh F0272. No original Arc is captured. |
| M0281 | Expr | SharedExpr conversion, plan construction and dispatch; profiling records | Witness now at evaluate(head.into_owned()); shared resume_call_after_head accepts Expr or existing SharedExpr and performs Into<SharedExpr> once. |
| M0282 | PreparedCallArguments or Option<Expr> (split) | normalize prepared args, install restore; stage Some returns value, None advances cursor | Prepare macro → PreparedDispatchFrame; stage macro → shared finish_call_dispatch_stage, F0275. Shared normalization/restore helper runs once. |
| M0283 | PreparedCallArguments or Option<Expr> (split) | same work with the suspended plan and frozen restore | Prepare macro → PreparedDispatchFrame; shared stage macro → F0276; restore freezes after the same helper. |
| M0285 | CompiledMatchOutcome | matched branch evaluates own value; unmatched branch resumes symbol lookup | Same branches in F0277; only the Expr transport map and raw input decode disappear. |
| M0286 | Expr | Some(value) after evaluating the threaded call | F0278 now completes resumed.map(Some); three listable fixtures watched fail → pass. |
| M0287 | Expr | profile the builtin result, then result.map(Some) | Same Some completion in unchecked F0278; suspension records the existing profiling event before the typed macro returns. |
| M0288 | Expr | profile, memory-growth validation, then result.map(Some) | Checked F0278 validates the Expr result and then wraps Some. The If-held fixtures exercise the checked test build; the unchecked constructor shares this completion body. Its release path has not yet been measured. |
| M0289 | Expr | ReleaseHold evaluation result unchanged | Typed identity frame. |
| M0561 | Expr | fixed-point equality/cycle history/trace update and next pass | Typed F0526 uses outcome? then the unchanged resume_fixed_point_from state transition. |
| M0562 | Expr | same fixed-point transition in resumed evaluation | Typed F0527 uses outcome? then the same helper. |

| Transform / obligation | State |
| --- | --- |
| T0023 | Wrapper removed; generic PrepareArguments body is unreachable. Parsed-source assertion now proves no wrapper definition or caller, and executed before the X0040 failure. |
| T0024 | Actual Expr witnessed before sharing; conversion in common resume_call_after_head. New identity is its evaluated_head.into() use. |
| T0025 / T0026 | Prepared consumer split from Option consumer; each ordinary prepared.map calls the shared normalization/restore helper, also used after resume. |
| T0027 | Only raw transport map/decode removed; matched and unmatched own-value branches retained. |
| D0001 | Actual repeated producer witness is mandatory at the sole token suspend route. Fresh F0272 state carries cursor and splice flag without an Arc. Typed finalizer returns PreparedCallArguments after owner release; typed driver/drop test passes. |

Whole0029–Whole0032 retain their old identities and decoder IDs as explicit removed S3 consumers. T0023/T0027 retain removed-map identities; T0024/T0025/T0026 point to their actual relocated uses. The remaining skipped semantic map found previously is still scoping/callable.rs:150–151, F0528/M0563, owned by S13: finish_function_application lacks the ordinary .map(Some). It was not changed.

```text
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
```

The complete census run failed before its scratch falsifiers could execute: expected X=39, actual X=40. The new use is prepared_slot_finalizer_fixture calling into_continuation_expr after the helper now returns typed PreparedCallArguments. X0040 was recorded as a test-support Legacy provider use. That final receipt addition and the new forced-red expectations remain unrerun because the independent retention replay triggered STOP. No passing S3 census or earned completion is claimed.

| Receipt | Status before → after | Identity/anchors |
| --- | --- | --- |
| F0204 | untyped → typed | rum_runtime::runtime::builtin_adapters::resume_frames::BuiltinEvaluateSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0272 | untyped → typed | rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0273 | untyped → removed | rum_runtime::runtime::eval_call_prepare::resume_frames::ResumeSuspendedCallArgumentPreparationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame; anchors=F0272 |
| F0274 | untyped → typed | rum_runtime::runtime::eval_core::call::EvaluateCallSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0275 | untyped → typed | rum_runtime::runtime::eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0276 | untyped → typed | rum_runtime::runtime::eval_core::call::ResumeSuspendedCallWithDispatchPlanSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0277 | untyped → typed | rum_runtime::runtime::eval_core::symbol::EvaluateStoredOwnValueSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0278 | untyped → typed | rum_runtime::runtime::eval_dispatch::RunCallDispatchStageSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0279 | untyped → typed | rum_runtime::runtime::hold_thread::EvalReleaseHoldSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0526 | untyped → typed | rum_runtime::runtime::resolution::resume_frames::EvaluateProfiledFixedPointSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0527 | untyped → typed | rum_runtime::runtime::resolution::resume_frames::ResumeFixedPointFromSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=- |
| F0570 | untyped → typed | rum_runtime::runtime::eval_call_prepare::resume_frames::FinishPreparedCallArgumentsFinalizer implements rum_runtime::runtime::suspension::typed::SlotFinalizer; anchors=- |
| M0205 | untyped → typed | rum_runtime::runtime::builtin_adapters::builtin_evaluate#macro1:resume_typed_or_suspend:result; anchors=- |
| M0279 | untyped → typed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::prepare_arguments_for_call_with_unevaluated_restore#macro1:resume_typed_or_suspend:result; anchors=- |
| M0280 | untyped → removed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::resume_suspended_call_argument_preparation#macro1:resume_or_suspend:insert; anchors=F0272 |
| M0281 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::evaluate_call#macro1:resume_typed_or_suspend:result; anchors=- |
| M0282 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#macro1:resume_typed_or_suspend:result; anchors=- |
| M0283 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#macro1:resume_typed_or_suspend:result; anchors=- |
| M0285 | untyped → typed | rum_runtime::runtime::eval_core::symbol::<Session as inherent>::evaluate_stored_own_value#macro1:resume_typed_or_suspend:result; anchors=- |
| M0286 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::run_call_dispatch_stage#macro1:resume_typed_or_suspend:result; anchors=- |
| M0287 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::try_dispatch_builtin_call#macro1:resume_typed_or_suspend:result; anchors=- |
| M0288 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::try_dispatch_builtin_call#macro2:resume_typed_or_suspend:result; anchors=- |
| M0289 | untyped → typed | rum_runtime::runtime::hold_thread::<Session as inherent>::eval_release_hold#macro1:resume_typed_or_suspend:result; anchors=- |
| M0561 | untyped → typed | rum_runtime::runtime::resolution::<Session as inherent>::evaluate_profiled_fixed_point#macro1:resume_typed_or_suspend:result; anchors=- |
| M0562 | untyped → typed | rum_runtime::runtime::resolution::<Session as inherent>::resume_fixed_point_from#macro1:resume_typed_or_suspend:result; anchors=- |
| D0001 | untyped → implemented | deferred::prepared arguments; anchors=F0204;F0272;F0273;F0274;F0275;F0276;F0277;F0278;F0279;F0526;F0527;F0570;F0275P |
| T0023 | untyped → removed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::prepare_arguments_for_call#method1:map:self . prepare_arguments_for_call_with_unevaluated_restore (head , attributes , args); anchors=F0272 |
| T0024 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_after_head#method1:into:evaluated_head; anchors=F0274 |
| T0025 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#method16:map:prepared; anchors=F0275;F0275P |
| T0026 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#method16:map:prepared; anchors=F0276;F0275P |
| T0027 | untyped → removed | rum_runtime::runtime::eval_core::symbol::<EvaluateStoredOwnValueSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#method4:map:(\| \| -> Result < rum_core :: Expr , EvalError > { if outcome . matched { session . evaluate_accepted_own_value (& symbol , body , delayed) } else { session . evaluate_symbol_after_stored_own_value (symbol) } }) (); anchors=F0277 |
| Whole0029 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call1:prepared_call_arguments_from_continuation_expr:; anchors=D009 |
| Whole0030 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call4:from_continuation_expr:; anchors=D125 |
| Whole0031 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call5:from_continuation_expr:; anchors=D125 |
| Whole0032 | legacy-use → removed | rum_runtime::runtime::eval_core::symbol::<EvaluateStoredOwnValueSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr:; anchors=D049 |
| F0275P | absent → typed | rum_runtime::runtime::eval_core::call::PreparedDispatchFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; anchors=F0275;F0276 |
| M0282S | absent → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::finish_call_dispatch_stage#macro1:resume_typed_or_suspend:result; anchors=M0282;M0283 |
| S3 | absent → implemented | S3 evaluation dispatch: 12 typed implementations and 13 typed macro sites; anchors=T0023;T0024;T0025;T0026;T0027;D0001 |
| X0040 | absent → legacy-test | rum_runtime::runtime::eval_call_prepare::prepared_slot_finalizer_fixture#method1:into_continuation_expr:finish_suspended_prepared_call (SuspendedPreparedCallArguments { evaluated_args : PreparedArgumentAccumulator :: freeze (prepared . evaluated_args . iter () . cloned () . collect () ,) , unevaluated_restores : prepared . unevaluated_restore . as_ref () . map (\| restore \| UnevaluatedRestoreAccumulator :: freeze (restore . wrappers . clone ())) , }); anchors=S3 typed finalizer materialized through the unchanged Legacy provider |

### 3. Divergence evidence and ordinary controls

These watched failures and post-conversion equality results are retained from redispatch 1, not rerun in redispatch 2. Both feature modes failed all three If-held marker cases and all three completed-listable marker cases on byte-identical S2 production code. In each mode four cases reported UnexpectedPause(1), and the two Some[5] cases reported Value mismatches. The path is InputString → Expr continuation → F0278 Some completion → typed stage consumer; nested builtin completions also use F0278. No Function continuation with the separate S13 skipped map is on these isolating paths. The default six-case post run passed, and the no-default eight-case post run passed six marker cases plus prepared.twice/head.map.twice. All Checks::ALL comparisons and unconditional FaultProbe checks remain intact.

**r1-pre-builtin-held-default: exit 101.** Raw output: `.temp/brief-609/r1-pre-builtin-held-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.29s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-builtin-held-no-default: exit 101.** Raw output: `.temp/brief-609/r1-pre-builtin-held-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.13s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 661, identity: None, text: SymbolText { id: 661, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-default: exit 101.** Raw output: `.temp/brief-609/r1-pre-threaded-dv-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.35s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 673, identity: None, text: SymbolText { id: 673, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-no-default: exit 101.** Raw output: `.temp/brief-609/r1-pre-threaded-dv-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.46s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-post-map-1: exit 0.** Raw output: `.temp/brief-609/r1-post-map-1.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 36.43s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 6 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=12
S2 builtin.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=24
S2 builtin.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=36
S2 builtin.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=48
S2 threaded.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=60
S2 threaded.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=72
S2 threaded.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.04s
```

**r1-post-map-no-default: exit 0.** Raw output: `.temp/brief-609/r1-post-map-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch -- --nocapture
```

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.22s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 8 tests
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=12
ok
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=24
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=36
S3 builtin.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=48
S3 builtin.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 677, identity: None, text: SymbolText { id: 677, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=60
S3 builtin.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=72
S3 threaded.empty_some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=84
S3 threaded.none strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 677, identity: None, text: SymbolText { id: 677, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=96
S3 threaded.some strict equality PASS: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }; log=List[]
ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.06s
```

Final observed FaultProbe deltas: zero for every pre-conversion group (48 observations per three-case group); zero for the default post run (72 observations); zero for the no-default post run (96 observations). The full oracle has not run in this redispatch, so there is no new full-build aggregate delta. The 28 ordinary controls are prepared as a bounded scratch compiler fixture at r2/ordinary-controls using unchanged descriptor/support logic and the exact brief-606 sources, but were not built or executed before STOP. The original six Fold/Nest cases and their failures remain archived for S13, as authorized in redispatch 1.

### 4. Verification, inventory evidence and the retention STOP

All Cargo commands ran serially under the 900-second run-bounded wrapper, CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1, CARGO_NET_OFFLINE=true and TMPDIR=/home/remi/dev/rum/.temp/brief-609/tmp. The non-Cargo frontier enumerator ran under a separate bounded command. No timing decision was made while gates were running. All launched commands have finished.

**r2-ownership-discover: exit 0.** Raw output: `.temp/brief-609/r2-ownership-discover.log`.

```text
cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.94s
running 1 test
UnknownHostPayload breakdown: 11 storage edges; 68 public input sites
Self-reaching type count: 59
Production files: 688; test-only files: 433; nominal definitions: 1712; aliases: 43; owning edges: 3877; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 5.72s
```

**r2-typed-tests: exit 101.** Raw output: `.temp/brief-609/r2-typed-tests.log`.

```text
cargo test -p rum-runtime --lib suspension::typed::tests -- --nocapture
```

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.80s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 20 tests
test runtime::suspension::typed::tests::accounting::adapter_delegates_roots_and_capture_boundary_without_running_frame ... ok
test runtime::suspension::typed::tests::accounting::typed_admission_is_checked_below_at_and_above_projected_boundary ... 
thread 'runtime::suspension::typed::tests::accounting::typed_admission_is_checked_below_at_and_above_projected_boundary' panicked at crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs:88:9:
assertion failed: projected >=
    5 * std::mem::size_of::<usize>() + std::mem::size_of::<entry::Entry>()
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test runtime::suspension::typed::tests::accounting::typed_layout_charges_two_arcs_and_real_queue_cells_in_both_models ... 
thread 'runtime::suspension::typed::tests::accounting::typed_layout_charges_two_arcs_and_real_queue_cells_in_both_models' panicked at crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs:69:5:
assertion `left == right` failed
  left: 40
 right: 72
FAILED
test runtime::suspension::typed::tests::composition::deferred_completion_preserves_output_and_stops_without_finalizing ... ok
test runtime::suspension::typed::tests::composition::witnessed_chain_and_actions_remain_dormant ... ok
test runtime::suspension::typed::tests::driver_faults::cleanup_failure_or_nested_input_never_replaces_the_first_fault ... ok
test runtime::suspension::typed::tests::driver_faults::completion_exact_decode_and_probe_survive_rollback_and_replacement ... ok
test runtime::suspension::typed::tests::driver_faults::direct_public_fault_is_fatal_at_evaluation_and_test_capture ... ok
test runtime::suspension::typed::tests::driver_faults::wrong_arms_and_exact_state_unwind_captures_and_scopes ... ok
test runtime::suspension::typed::tests::faults::adapter_rejects_before_user_resume_and_forwards_errors ... ok
test runtime::suspension::typed::tests::faults::channel_modes_reject_the_other_arm_and_wrong_state_variant ... ok
test runtime::suspension::typed::tests::file_faults::slot_fault_rolls_back_nested_memory_files_and_public_batch_owner ... ok
test runtime::suspension::typed::tests::live::driver_drops_both_owning_arcs_before_running_the_finalizer ... ok
test runtime::suspension::typed::tests::live::evaluate_successor_and_nested_owner_precede_the_unrun_suffix ... ok
test runtime::suspension::typed::tests::live::legacy_and_typed_neighbors_exchange_the_unchanged_value_wire ... ok
test runtime::suspension::typed::tests::live::macro_is_cold_and_a_witnessed_chain_returns_its_last_output ... ok
test runtime::suspension::typed::tests::live::repeat_token_freezes_fresh_state_and_keeps_the_deferred_output_type ... ok
test runtime::suspension::typed::tests::registry::legacy_provider_retains_existing_leniency_and_wire ... ok
test runtime::suspension::typed::tests::registry::prepared_arguments_roundtrip_and_match_the_existing_finalizer_wire ... ok
test runtime::suspension::typed::tests::registry::registry_modes_are_fixed_per_slot_type ... ok

failures:

failures:
    runtime::suspension::typed::tests::accounting::typed_admission_is_checked_below_at_and_above_projected_boundary
    runtime::suspension::typed::tests::accounting::typed_layout_charges_two_arcs_and_real_queue_cells_in_both_models

test result: FAILED. 18 passed; 2 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.46s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

**r2-typed-tests-2: exit 0.** Raw output: `.temp/brief-609/r2-typed-tests-2.log`.

```text
cargo test -p rum-runtime --lib suspension::typed::tests -- --nocapture
```

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.56s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 20 tests
test runtime::suspension::typed::tests::accounting::adapter_delegates_roots_and_capture_boundary_without_running_frame ... ok
test runtime::suspension::typed::tests::accounting::typed_admission_is_checked_below_at_and_above_projected_boundary ... ok
test runtime::suspension::typed::tests::accounting::typed_layout_charges_two_arcs_and_real_queue_cells_in_both_models ... ok
test runtime::suspension::typed::tests::composition::deferred_completion_preserves_output_and_stops_without_finalizing ... ok
test runtime::suspension::typed::tests::composition::witnessed_chain_and_actions_remain_dormant ... ok
test runtime::suspension::typed::tests::driver_faults::cleanup_failure_or_nested_input_never_replaces_the_first_fault ... ok
test runtime::suspension::typed::tests::driver_faults::completion_exact_decode_and_probe_survive_rollback_and_replacement ... ok
test runtime::suspension::typed::tests::driver_faults::direct_public_fault_is_fatal_at_evaluation_and_test_capture ... ok
test runtime::suspension::typed::tests::driver_faults::wrong_arms_and_exact_state_unwind_captures_and_scopes ... ok
test runtime::suspension::typed::tests::faults::adapter_rejects_before_user_resume_and_forwards_errors ... ok
test runtime::suspension::typed::tests::faults::channel_modes_reject_the_other_arm_and_wrong_state_variant ... ok
test runtime::suspension::typed::tests::file_faults::slot_fault_rolls_back_nested_memory_files_and_public_batch_owner ... ok
test runtime::suspension::typed::tests::live::driver_drops_both_owning_arcs_before_running_the_finalizer ... ok
test runtime::suspension::typed::tests::live::evaluate_successor_and_nested_owner_precede_the_unrun_suffix ... ok
test runtime::suspension::typed::tests::live::legacy_and_typed_neighbors_exchange_the_unchanged_value_wire ... ok
test runtime::suspension::typed::tests::live::macro_is_cold_and_a_witnessed_chain_returns_its_last_output ... ok
test runtime::suspension::typed::tests::live::repeat_token_freezes_fresh_state_and_keeps_the_deferred_output_type ... ok
test runtime::suspension::typed::tests::registry::legacy_provider_retains_existing_leniency_and_wire ... ok
test runtime::suspension::typed::tests::registry::prepared_arguments_roundtrip_and_match_the_existing_finalizer_wire ... ok
test runtime::suspension::typed::tests::registry::registry_modes_are_fixed_per_slot_type ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.45s
```

**r2-census-1: exit 101.** Raw output: `.temp/brief-609/r2-census-1.log`.

```text
cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.96s
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-943817a3ecf6c029)

running 6 tests
test forced_red_probe ... ignored, invoked explicitly by both bounded scratch-copy falsifiers
test preflip_no_legacy_transport_uses ... census walk: 14936 functions, 299720 uses, 1500 module visits
census resolver: 34896 scopes
S3 negative control: preflip refused: S54 completed receipt is required first; no zero-legacy claim
ok
test removing_a_frame_in_a_scratch_copy_is_red ... census walk: 14936 functions, 299720 uses, 1500 module visits
census resolver: 34896 scopes
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 TRANSITION F0273 removed; successor/provider=F0272
S3 TRANSITION M0280 removed; successor/provider=F0272
S3 TRANSITION T0023 removed; successor/provider=F0272
S3 TRANSITION T0027 removed; successor/provider=F0277
S3 TRANSITION Whole0029 removed; successor/provider=D009
S3 TRANSITION Whole0030 removed; successor/provider=D125
S3 TRANSITION Whole0031 removed; successor/provider=D125
S3 TRANSITION Whole0032 removed; successor/provider=D049
S3 FRAME F0204 BuiltinEvaluateSuspensionFrame Expr -> Expr
S3 FRAME F0272 PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame Expr -> PreparedCallArguments
S3 FRAME F0274 EvaluateCallSuspensionFrame Expr -> Expr
S3 FRAME F0275 ResumeCallWithDispatchPlanSuspensionFrame Option<Expr> -> Expr
S3 FRAME F0276 ResumeSuspendedCallWithDispatchPlanSuspensionFrame Option<Expr> -> Expr
S3 FRAME F0275P PreparedDispatchFrame PreparedCallArguments -> Expr
S3 FRAME F0277 EvaluateStoredOwnValueSuspensionFrame CompiledMatchOutcome -> Expr
S3 FRAME F0278 RunCallDispatchStageSuspensionFrame Expr -> Option<Expr>
S3 FRAME F0279 EvalReleaseHoldSuspensionFrame Expr -> Expr
S3 FRAME F0526 EvaluateProfiledFixedPointSuspensionFrame Expr -> Expr
S3 FRAME F0527 ResumeFixedPointFromSuspensionFrame Expr -> Expr
S3 FRAME F0570 FinishPreparedCallArgumentsFinalizer  -> PreparedCallArguments
S3 TRANSITION T0024 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_after_head#method1:into:evaluated_head
S3 TRANSITION T0025 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#method16:map:prepared
S3 TRANSITION T0026 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#method16:map:prepared
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13

thread 'removing_a_frame_in_a_scratch_copy_is_red' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:91:31:
called `Result::unwrap()` on an `Err` value: "X receipt mismatch: expected 39, discovered 40\nmissing: \nunrecorded: rum_runtime::runtime::eval_call_prepare::prepared_slot_finalizer_fixture#method1:into_continuation_expr:finish_suspended_prepared_call (SuspendedPreparedCallArguments { evaluated_args : PreparedArgumentAccumulator :: freeze (prepared . evaluated_args . iter () . cloned () . collect () ,) , unevaluated_restores : prepared . unevaluated_restore . as_ref () . map (| restore | UnevaluatedRestoreAccumulator :: freeze (restore . wrappers . clone ())) , })"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test removing_a_test_guard_in_a_scratch_copy_is_red ... census walk: 14936 functions, 299720 uses, 1500 module visits
census resolver: 34896 scopes
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 TRANSITION F0273 removed; successor/provider=F0272
S3 TRANSITION M0280 removed; successor/provider=F0272
S3 TRANSITION T0023 removed; successor/provider=F0272
S3 TRANSITION T0027 removed; successor/provider=F0277
S3 TRANSITION Whole0029 removed; successor/provider=D009
S3 TRANSITION Whole0030 removed; successor/provider=D125
S3 TRANSITION Whole0031 removed; successor/provider=D125
S3 TRANSITION Whole0032 removed; successor/provider=D049
S3 FRAME F0204 BuiltinEvaluateSuspensionFrame Expr -> Expr
S3 FRAME F0272 PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame Expr -> PreparedCallArguments
S3 FRAME F0274 EvaluateCallSuspensionFrame Expr -> Expr
S3 FRAME F0275 ResumeCallWithDispatchPlanSuspensionFrame Option<Expr> -> Expr
S3 FRAME F0276 ResumeSuspendedCallWithDispatchPlanSuspensionFrame Option<Expr> -> Expr
S3 FRAME F0275P PreparedDispatchFrame PreparedCallArguments -> Expr
S3 FRAME F0277 EvaluateStoredOwnValueSuspensionFrame CompiledMatchOutcome -> Expr
S3 FRAME F0278 RunCallDispatchStageSuspensionFrame Expr -> Option<Expr>
S3 FRAME F0279 EvalReleaseHoldSuspensionFrame Expr -> Expr
S3 FRAME F0526 EvaluateProfiledFixedPointSuspensionFrame Expr -> Expr
S3 FRAME F0527 ResumeFixedPointFromSuspensionFrame Expr -> Expr
S3 FRAME F0570 FinishPreparedCallArgumentsFinalizer  -> PreparedCallArguments
S3 TRANSITION T0024 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_after_head#method1:into:evaluated_head
S3 TRANSITION T0025 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#method16:map:prepared
S3 TRANSITION T0026 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#method16:map:prepared
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13

thread 'removing_a_test_guard_in_a_scratch_copy_is_red' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:91:31:
called `Result::unwrap()` on an `Err` value: "X receipt mismatch: expected 39, discovered 40\nmissing: \nunrecorded: rum_runtime::runtime::eval_call_prepare::prepared_slot_finalizer_fixture#method1:into_continuation_expr:finish_suspended_prepared_call (SuspendedPreparedCallArguments { evaluated_args : PreparedArgumentAccumulator :: freeze (prepared . evaluated_args . iter () . cloned () . collect () ,) , unevaluated_restores : prepared . unevaluated_restore . as_ref () . map (| restore | UnevaluatedRestoreAccumulator :: freeze (restore . wrappers . clone ())) , })"
FAILED
test source::root_set::syntax_root_discovers_added_recursive_type_and_restores_source ... OLD ROOTS: rum_syntax::ast::ScannerRootSetProbe invisible (no definition or self-reaching row)
NEW ROOTS: rum_syntax::ast::ScannerRootSetProbe	crates/rum-syntax/src/ast.rs	986	rum_syntax::ast::ScannerRootSetProbe.next/std::option::Option[Own]/arg0/std::boxed::Box[Own]/arg0 -> rum_syntax::ast::ScannerRootSetProbe
RESTORED: rum_syntax::ast::ScannerRootSetProbe absent; all other cycles and unknowns unchanged
RESOLVED: all six former rum-syntax foreign storage rows are owning edges
ok
test source_baseline_matches_earned_receipt ... census walk: 14936 functions, 299720 uses, 1500 module visits
census resolver: 34896 scopes
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 TRANSITION F0273 removed; successor/provider=F0272
S3 TRANSITION M0280 removed; successor/provider=F0272
S3 TRANSITION T0023 removed; successor/provider=F0272
S3 TRANSITION T0027 removed; successor/provider=F0277
S3 TRANSITION Whole0029 removed; successor/provider=D009
S3 TRANSITION Whole0030 removed; successor/provider=D125
S3 TRANSITION Whole0031 removed; successor/provider=D125
S3 TRANSITION Whole0032 removed; successor/provider=D049
S3 FRAME F0204 BuiltinEvaluateSuspensionFrame Expr -> Expr
S3 FRAME F0272 PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame Expr -> PreparedCallArguments
S3 FRAME F0274 EvaluateCallSuspensionFrame Expr -> Expr
S3 FRAME F0275 ResumeCallWithDispatchPlanSuspensionFrame Option<Expr> -> Expr
S3 FRAME F0276 ResumeSuspendedCallWithDispatchPlanSuspensionFrame Option<Expr> -> Expr
S3 FRAME F0275P PreparedDispatchFrame PreparedCallArguments -> Expr
S3 FRAME F0277 EvaluateStoredOwnValueSuspensionFrame CompiledMatchOutcome -> Expr
S3 FRAME F0278 RunCallDispatchStageSuspensionFrame Expr -> Option<Expr>
S3 FRAME F0279 EvalReleaseHoldSuspensionFrame Expr -> Expr
S3 FRAME F0526 EvaluateProfiledFixedPointSuspensionFrame Expr -> Expr
S3 FRAME F0527 ResumeFixedPointFromSuspensionFrame Expr -> Expr
S3 FRAME F0570 FinishPreparedCallArgumentsFinalizer  -> PreparedCallArguments
S3 TRANSITION T0024 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_after_head#method1:into:evaluated_head
S3 TRANSITION T0025 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#method16:map:prepared
S3 TRANSITION T0026 applied at rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#method16:map:prepared
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13

thread 'source_baseline_matches_earned_receipt' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:60:54:
X receipt mismatch: expected 39, discovered 40
missing: 
unrecorded: rum_runtime::runtime::eval_call_prepare::prepared_slot_finalizer_fixture#method1:into_continuation_expr:finish_suspended_prepared_call (SuspendedPreparedCallArguments { evaluated_args : PreparedArgumentAccumulator :: freeze (prepared . evaluated_args . iter () . cloned () . collect () ,) , unevaluated_restores : prepared . unevaluated_restore . as_ref () . map (| restore | UnevaluatedRestoreAccumulator :: freeze (restore . wrappers . clone ())) , })
FAILED

failures:

failures:
    removing_a_frame_in_a_scratch_copy_is_red
    removing_a_test_guard_in_a_scratch_copy_is_red
    source_baseline_matches_earned_receipt

test result: FAILED. 2 passed; 3 failed; 1 ignored; 0 measured; 0 filtered out; finished in 98.74s

error: test failed, to rerun pass `-p rum-stdlib --test typed_slot_census`
```

**r2-frame-root: exit 0.** Raw output: `.temp/brief-609/r2-frame-root.log`.

```text
cargo test -p rum-runtime --lib frame_root_trace -- --nocapture
```

```text
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 15 tests
test runtime::definitions::application::replace_all::plan::frame_root_trace_tests::rewrite_rule_plan_visits_the_exact_rule_roots ... ok
test runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph ... ok
test runtime::tests::frame_root_trace::manual_frame_root_trace_implementations_match_sorted_allowlist ... ok
test runtime::tests::frame_root_trace::root_visitation::association_entry_form_visits_key_and_value_roots_in_field_order ... ok
test runtime::tests::frame_root_trace::root_visitation::block_symbol_state_map_visits_the_bare_symbol_key ... ok
test runtime::tests::frame_root_trace::root_visitation::container_implementations_visit_keys_values_and_elements ... ok
test runtime::tests::frame_root_trace::root_visitation::enum_shapes_and_generic_arrays_trace_without_special_cases ... ok
test runtime::tests::frame_root_trace::root_visitation::expr_is_one_whole_root_and_bare_symbol_is_an_independent_root ... ok
test runtime::tests::frame_root_trace::root_visitation::file_io_error_is_the_zero_root_negative_calibration_target ... file_io_error_expr_roots=0
ok
test runtime::tests::frame_root_trace::root_visitation::matcher_binding_table_visits_the_exact_symbol_and_expression_roots ... ok
test runtime::tests::frame_root_trace::root_visitation::nested_derived_fields_are_all_traversed ... ok
test runtime::tests::frame_root_trace::root_visitation::recursive_matcher_arg_visits_every_nested_root_exactly_once ... ok
test runtime::tests::frame_root_trace::root_visitation::shared_expression_types_borrow_their_expression_roots ... ok
test runtime::tests::frame_root_trace::suspension_oracle_fixture_population_is_exact ... ok
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... named_suspension_frames=564
named_pure_finalizers=59
pure_frame_abandon_derives=531
handwritten_stateful_frame_abandons=33
stateful_frame_abandon=crates/rum-runtime/src/runtime/builtins_context_loader/resume.rs:BeginPackageDependencyFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:EvalEvaluationDataSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/monitor.rs:EvalMonitorSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/reap.rs:EvalReapSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:ResumeMemoryConstrainedWithLimitSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:ResumeTimeConstrainedWithLimitSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:EvalExprWithStreamCaptureSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:EvalStructuredTraceCaptureExprSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:EvalAbortProtectSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortAfterPrimarySuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortAfterPrimarySuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortWithPropagationSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:EvaluateLoopIteratorBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:WithIterationProductScopeSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:EvalCheckSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:EvalQuietSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:DriveWithCleanupPrimarySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:EvalWithCleanupSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/options/patterns.rs:EvaluateWithOptionValueScopeSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:EvalSingleSymbolBlockSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeBlockBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeBlockInitializersSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeSingleSymbolBlockBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeModuleInitializersSuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeModuleInitializersSuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeSuspendedModuleInitializersSuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeSuspendedModuleInitializersSuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs:ResumePackageInstallSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/session/batch_file.rs:PublicFileRollbackFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/session/guest_loading/file.rs:GuestFileCloseFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/session/guest_loading/script.rs:GuestScriptFormFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/session/session_api/resume_frames.rs:EvaluateWithRandomStateRestoredSuspensionFrame
evaluate_cleanup_allowlist=WithCleanup owner frames (2)
raw_live_owner_clears=0
raw_live_owner_replacements=0
removed_cancellation_lane_hits=0
legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0
ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.22s
```

**r2-stop-format: exit 0.** Raw output: `.temp/brief-609/r2-stop-format.log`.

```text
cargo fmt --all
```

```text
(no output)
```

**r2-final-fmt-check: exit 0.** Raw output: `.temp/brief-609/r2-final-fmt-check.log`.

```text
cargo fmt --check
```

```text
(no output)
```

The first typed-test run compiled successfully but failed two setup-dependent byte assertions (18 passed / 2 failed). At S2, ordinary InputString evaluation left a legacy queue; at S3 it already widens the queue through production typed frames. The setup now invokes the same actual input producer directly, retaining a legacy queue for the widening experiment. The unchanged assertions now pass (20/20). This is a fixture precondition repair, not a relaxed accounting contract. Frame-root 15/15 includes both derived inventories, typed frame roots and the pure typed finalizer signature.

The runtime unit target warns about register_test_if_builtin, register_test_return_builtin and register_test_vm_template_builtins. The exact same three warnings are present in .temp/brief-608/redispatch-5/runtime-typed.log. No introduced warning appeared in the executed builds. Oracle warning cleanup is implemented: TYPING_RECEIPT is used, FLIP_SLOT is census-only, and the unused oracle drive::execute is removed. Zero warnings in both complete oracle builds is still unverified. No native ZeroMQ test ran; no native test skip or pass is claimed.

```text
{
  "count": 79,
  "added": [],
  "removed": []
}
```

Ownership discovery passed and has 79 UnknownHostPayload rows: 11 storage edges and 68 public input sites. Identity comparison normalizes only @line positions and finds no added or removed identity. The existing suspend(fresh) ledger identity is retained. Inventory cells and dispositions were not replayed before the later STOP.

```text
python3 scripts/run-bounded.py --timeout 900 -- python3 .temp/brief-609/gate.py r2-frontier .temp/brief-608/redispatch-3/frontier-candidate
exit: 0
932 rows: RET001=10 RET002=895 RET003=27
917 complete records identical; 14 paired changes; 1 unmatched removed row; 1 attributed added row
```

All 18 checker source modules used by the retained scratch enumerator are byte-identical to the current checker. The baseline is .temp/brief-608/redispatch-5/frontier-candidate.tsv; candidate output is .temp/brief-609/r2-frontier.log. r2/frontier-raw-diff.json preserves every changed field; r2/frontier-classified-diff.json pairs fourteen changes and explicitly leaves the unmatched removal unmatched.

| Before | After | Classification |
| --- | --- | --- |
| RET003 crates/rum-runtime/src/runtime/resolution.rs:385:21 | RET003 crates/rum-runtime/src/runtime/resolution.rs:377:24 | M0561 legacy insertion macro -> typed result macro; same opaque macro rejection |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:482:9 | RET002 crates/rum-runtime/src/runtime/resolution.rs:480:9 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:479:12 | RET002 crates/rum-runtime/src/runtime/resolution.rs:480:9 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:482:9 | RET002 crates/rum-runtime/src/runtime/resolution.rs:485:20 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:479:12 | RET002 crates/rum-runtime/src/runtime/resolution.rs:477:12 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:490:17 | RET002 crates/rum-runtime/src/runtime/resolution.rs:488:17 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:496:5 | RET002 crates/rum-runtime/src/runtime/resolution.rs:494:5 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:487:21 | RET002 crates/rum-runtime/src/runtime/resolution.rs:485:21 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:487:20 | RET002 crates/rum-runtime/src/runtime/resolution.rs:494:42 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:490:33 | RET002 crates/rum-runtime/src/runtime/resolution.rs:488:33 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:496:42 | RET002 crates/rum-runtime/src/runtime/resolution.rs:477:12 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:490:17 | RET002 crates/rum-runtime/src/runtime/resolution.rs:488:17 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:496:5 | RET002 crates/rum-runtime/src/runtime/resolution.rs:494:5 | same operation and witness shifted by owned S3 macro replacement |
| RET002 crates/rum-runtime/src/runtime/resolution.rs:474:5 | RET002 crates/rum-runtime/src/runtime/resolution.rs:472:5 | same operation and witness shifted by owned S3 macro replacement |

**Unpaired removed row, verbatim:**

```text
kind	file	start_line	start_col	end_line	end_col	target	reason	witness
RET002	crates/rum-runtime/src/runtime/resolution.rs	35	19	35	46		path AccumulatorChain::default has 0 exact bindings	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point -> rum_runtime::runtime::resolution::FixedPointHistory::freeze
```

**Added row, verbatim (a different operation):**

```text
kind	file	start_line	start_col	end_line	end_col	target	reason	witness
RET002	crates/rum-runtime/src/runtime/resolution.rs	377	24	389	15		implicit Try/FromResidual/error ownership not modeled	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr -> rum_runtime::runtime::accounting_mode::rule::evaluate_rule -> rum_runtime::runtime::accounting_mode::rule::evaluate_unarmed -> rum_runtime::runtime::model::session::Session::evaluate -> rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point
```

The unchanged operation at resolution.rs:35 is AccumulatorChain::default inside FixedPointHistory::freeze. Its old caller at resolution.rs:383 was visible to the historical checker, outside resume_or_suspend!. The candidate caller at :385 is inside resume_typed_or_suspend! at :377. regions.rs:295 rejects the macro without traversing its tokens, so freeze drops out of the reachable frontier. The added :377 RET002 is the typed macro result’s ? / Try / FromResidual boundary. It cannot be paired with the removed default-construction rejection because it is a different operation. No apparent unchanged total can establish preservation of that rejection.

The governing Scope states **“An unpaired removal is a STOP.”** Its referenced brief-608 redispatch-3 rule requires each removed row to pair with an added row for the same operation. The relevant acceptance pin is crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:210; it was not edited. The checker and regions are outside the authorized repair scope and also remain unchanged. No further production repair or acceptance/performance gate began after this finding.

**Not run:** either 31-pair compiler-control suite; a green complete default census after X0040; the no-default census; the full oracle in either build; all-28 ordinary controls; full ownership_graph acceptance; local-production acceptance in either build; production_rejection_census_is_complete; workspace tests; fingerprint/certificate/ownership inventory replays; candidate release builds and every performance gate. The oracle script was copied under r2 with only root/output-path relocation to respect the scratch boundary, but was not executed. The maintenance replay scripts were prepared under r2 but not executed; no certificate file was emitted.

### 5. Allocation, instruction, resume attribution and timing evidence

The four codegen_units=1 S2 bases are unchanged and were not rebuilt. Their hashes were reverified, along with all 183 protected design/harness inputs. Existing resume-harness sources remain unchanged.

| Binary | Bytes | SHA-256 |
| --- | --- | --- |
| resume-alloc | 12185720 | 73f279bfb5e13ab4daa34242a9d591e52d9d8012267197268e80f63d41ecc503 |
| resume | 12180552 | 57e800bca2cca0d327b358f410d34ced30787410bdee6afe5838f9334a24e81f |
| rum-cli-alloc | 12547616 | f1fb356eb26f45dedaa09ad46e8d697dfde6bd10fbfe1aa12290f61dfcf6e9f1 |
| rum-cli | 12537440 | b553a00e63e8af387e3141fe35764f54515a3e3154fd09b2756f911a2e87d9bf |

There is no S3 performance result. No candidate binary was built; all 64 allocation/byte slopes, interleaved instruction pairs, above-threshold five-run/three-Callgrind investigations, the five resume workload function/address attributions and the final full timing panel remain unrun. The extra typed Arc is a design expectation, not a measured allocation result. No no-cost, exact-delta, copy-free action path or performance-acceptance claim is made.

### 6. S2 cold-helper leftover

continuation_fault_outcome still has #[cold] and #[inline(never)] and consumes Result<T, EvalError>, but returns EvalError directly. Four production callers use Err(self.continuation_fault_outcome(...)); only the two specified test callers lost unwrap_err. The typed fault suites now pass, including latch/rollback/file cases. The required address-level comparison of the hot boundary has not run. Source similarity is not instruction evidence.

### 7. Stop-loss, deviations and next decision

Stop-loss counter: 0. The two typed accounting failures were reduced to zero while preserving every assertion; the source census now reconciles owned F/M/T/D transitions and reaches the single new X0040 fixture receipt, which was then recorded. The independent replay triggered a specific mandatory STOP, not the two-unproductive-attempt rule. There were no Rust compiler errors in the executed redispatch-2 builds. A first evidence-audit script included an unused historical census/test copy; its byte-comparison failed, and the corrected audit checked exactly the 18 reachable enumerator modules, all identical. No source or checker was changed by that audit correction.

No git command, network operation, new unsafe, dependency/lock/toolchain edit, out-of-scope family conversion, oracle comparison relaxation or rejection weakening occurred. All scratch is under brief-609; no certificate output was needed. The source snapshot diff is r2/candidate.patch and the hash audit is r2/stop-audit.json. The brief entry prefix is preserved byte-for-byte. The tree is left uncommitted and incomplete.

The director decision concerns the lost FixedPointHistory::freeze frontier row: either authorize a precise discovery transition for the now-opaque caller, or direct a source shape that preserves the same rejected operation’s visibility while retaining witnessed cold frame construction. After that resolution, rerun the corrected census/falsifiers, the 31 pairs in both builds, all replays and remaining correctness/performance gates. Current half-finished items are explicitly preserved: unverified X0040 final receipt, unrun new compiler pair, unverified warning-free oracle target, unreplayed ownership/fingerprint/certificate data, and absent performance evidence.


## Director amendment — redispatch 3: the frontier analyzer models the typed macro

The STOP is correct, and the cause is a class, not this one site.

**Cause.** The frontier analyzer rejects every macro as opaque
(`regions.rs:295`, RET003) and never visits its tokens.
- The old site matched the producer by hand and called
  `FixedPointHistory::freeze` in a plain arm, so the analyzer saw it.
- The typed macro owns that match by design: it derives the witness from
  the actual `Result` and constructs the frame only in the `InputRequired`
  arm. So the state construction now sits inside the macro's arguments, and
  `freeze`'s rejection drops out of the frontier.
- Every typing step will do this. The design makes the typed macro the
  permanent form: R renames it to the canonical `resume_or_suspend!`. The
  analyzer therefore has to understand it.
- There is precedent. The local analyzer already models the legacy macro
  through an expansion-checked adapter (`retention_closure/local/resume_macro.rs`).

**Ruling.**

1. **Model `resume_typed_or_suspend!` in the frontier analyzer.**
   Authorized: `crates/rum-stdlib/src/tests/retention_closure/regions.rs`
   and one new adapter module beside it.
   - The adapter checks the macro's definition against its expected
     expansion, as `local/resume_macro.rs` does for the legacy macro, so a
     change to the definition fails the census.
   - The analyzer then visits the session expression, `$result`, the `ok`
     pattern and `$continue`, and the `suspend` arm's `$frame` as ordinary
     Rust in the enclosing function.
   - The expansion's own operations (the witness borrow, the match, the
     typed insertion and the `InputRequired` reconstruction) form a fixed
     list taken from the checked definition. Each one either resolves
     through the existing resolver or is a rejection row at the macro's
     span. Nothing in the expansion goes unaccounted.
   - The legacy `resume_or_suspend!` stays opaque in the frontier. Its rows
     do not move until a step converts its site.
   - Share parsing with `local/resume_macro.rs` where that is natural. Do
     not change the local analyzer's behavior in this step unless a
     certified local body contains a converted site; say whether one does.
2. **Falsifiers, watched to fail first.** Make scratch copies, never
   repository edits:
   - **visibility:** an operation that the frontier rejects, placed inside
     a typed macro arm, must still produce its rejection row;
   - **definition drift:** an altered expansion of the typed macro must fail
     the adapter check.

   Show each one failing against the opaque analyzer or the wrong
   definition, and passing as intended.
3. **Census classification for this step.** Re-derive 932 and the kind
   totals. Every row diff falls into exactly one class:
   - *moved pair:* the same operation, shifted;
   - *attributed S3 change:* an owned transformation added or removed it;
   - *revealed on conversion:* an operation that sat inside the opaque
     legacy macro and is visible in the typed macro. Give its old opaque
     span;
   - *legacy macro modeled:* a converted site's legacy RET003 row,
     replaced by the typed macro's accounted expansion.

   An unpaired removal of any other kind is still a STOP. Update the
   comments and assertions in `census/tests.rs` to the new totals, listing
   these classes. The `ACCEPTANCE_BASELINE` print is a historical string;
   leave it unchanged.
4. **Then finish the step**, exactly as redispatch 2 listed.

Append a new `## Execution report` after this section.


## Execution report — redispatch 3: STOP on compiler-oracle macro traversal

**S3 remains a partial candidate and is not accepted.** The checked typed-macro frontier adapter restores the missing rejection and passes both watched falsifiers. Both full oracle builds pass 58 tests without warnings and with zero fault-occurrence deltas; all 31 compiler pairs and all 28 ordinary controls pass in each build. The completed workspace run exposes a new compiler-oracle traversal gap outside the authorized analyzer files, which triggers the Scope’s explicit STOP. No constructor claim, rejection, assertion or fault gate was removed to bypass it.

The workspace also finds an unresolved native-stack regression in the unchanged 48-level recursion test. No repair was applied after the scope stop. No candidate release binary or performance measurement exists; the four S2 bases remain intact. The new report preserves all earlier reports as historical evidence, including their superseded STOPs and the corrected claim about the shared support wrapper.

### 1. Files changed, ripple and pin disclosures

| File | Semantic change / old → new |
| --- | --- |
| crates/rum-runtime/src/runtime/builtin_adapters.rs | M0205 legacy macro → witnessed typed macro; identity success tail retained. |
| crates/rum-runtime/src/runtime/builtin_adapters/resume_frames.rs | F0204 legacy frame → TypedFrame<Expr, Expr>. |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | T0023 wrapper removed; initial witnessed capture; deferred helpers return ResumeStop<PreparedCallArguments>, advance fresh cursor/splice state, and use the witnessed token. Pure completion now returns PreparedCallArguments; its existing Legacy wire provider consumes those fields directly instead of freeze/thaw round-tripping them. The test-only old-wire fixture follows that helper’s typed result. |
| crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs | F0273 merges into F0272 so every repeat freezes fresh state of the same nominal type. F0570 becomes SlotFinalizer<Output=PreparedCallArguments>; finish_work schedules it after originating owners drop. |
| crates/rum-runtime/src/runtime/eval_core/call.rs | T0024 witnesses Expr before head sharing; shared resume_call_after_head performs the conversion. T0025/T0026 add PreparedDispatchFrame and share argument normalization/restore installation. F0275/F0276 consume Option<Expr>; their ordinary stage-result macro is shared in finish_call_dispatch_stage. Remove both shape-dependent old decoder branches. Stable heads retain their existing SharedExpr value. Profiling stop remains after ordinary preparation normalization and before returning a suspension. |
| crates/rum-runtime/src/runtime/eval_core/mod.rs | The SharedExpr-result profiling wrapper loses its final caller under T0024 and is removed; generic outcome classification remains. |
| crates/rum-runtime/src/runtime/eval_core/symbol.rs | Only F0277/M0285 typed: CompiledMatchOutcome → Expr, retaining matched/unmatched semantics and deleting the Expr wire map. The S14 loader capture in this file is unchanged. |
| crates/rum-runtime/src/runtime/eval_dispatch/mod.rs | Remove the generic PrepareArguments work and leave an unreachable arm. M0286 now witnesses the actual threaded Result instead of receiving only a request in a cold helper. M0287/M0288 use typed macros, preserving profiling on suspension. F0278 completes resumed.map(Some) after the existing memory-growth validation. |
| crates/rum-runtime/src/runtime/hold_thread.rs | F0279/M0289 typed Expr identity continuation for ReleaseHold. |
| crates/rum-runtime/src/runtime/resolution.rs | M0561/M0562 witnessed at the actual next_result before matching; saved current, pass count and history retain their transitions. |
| crates/rum-runtime/src/runtime/resolution/resume_frames.rs | F0526/F0527 typed Expr → Expr, retaining outcome? and resume_fixed_point_from. |
| crates/rum-runtime/src/runtime/session/session_eval.rs | Two callers: Err(self.continuation_fault_outcome(...).unwrap_err()) → Err(self.continuation_fault_outcome(...)). |
| crates/rum-runtime/src/runtime/session/session_requests.rs | Two callers: remove unwrap_err from the cold latch projection, retaining the same hot discriminant check. |
| crates/rum-runtime/src/runtime/suspension/core/fault.rs | Cold continuation_fault_outcome: Result<T, EvalError> → EvalError; still consumes the outcome and takes the same latch. |
| crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs | Redispatch 2: delete suspend_witnessed; original suspend(session, request, fresh) → suspend(session, SlotWitness<F::Input>, request, fresh). Same public identity, same fresh F constraint, no Arc in the token. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs | The legacy-queue setup calls eval_input_string directly instead of evaluating an InputString call through newly typed S3 frames. This restores the legacy-to-mixed widening precondition; every allocation, byte and admission assertion is unchanged. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs | repeat_work now captures slot_witness(&outcome) from its actual input producer before matching and passes it to the sole suspend route. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | Scan both legacy and typed frame/finalizer traits; preserve 564-frame population and root/drop checks. Pin the Session-free PreparedCallArguments finalizer signature. Now compiled and tested: 15/15 frame-root tests pass. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs | In both derived-root and pure-abandon arrays: eval_call_prepare/resume_frames.rs:ResumeSuspendedCallArgumentPreparationSuspensionFrame → eval_core/call.rs:PreparedDispatchFrame. This pairs the D0001 merge with the T0025/T0026 split consumer. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. Four emitted adapter witnesses replace interim no-production-instantiation text with concrete S3 preparation-frame/finalizer ownership paths; disposition, owed_by and external coverage unchanged. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/retention_closure/census/tests.rs | Historical frontier 932 → 936; RET001 10 unchanged, RET002 895 → 900, RET003 27 → 26. 918 identical rows, 13 moved pairs, one old opaque site replaced by four accounted rows, and one new Try rejection. Explicitly preserve the freeze rejection; historical ACCEPTANCE_BASELINE string unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | Replay 68 fingerprint cells from final source context; first 18 fields of each row unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | Replay four emitted C01/C02 certificate cells for hash=true/false; other seven columns unchanged. Certified bodies contain no converted macro and local analyzer source is unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/regions.rs | Recognize canonical typed macro, check its expansion through adjacent adapter, visit producer/result binding/success/session/frame expressions with source spans and enclosing resolver. Record four unresolved expansion operations at macro span. Legacy macros stay opaque. |
| crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs | New checked adapter: exact macro body/matcher/attributes, canonical path and Ok/Err hygiene, full pattern/argument parsing; fixed exhaustive four expansion-operation rejection rows. Definition drift fails census; no retention admission. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs | New strict If-held and listable-completion cases for three marker shapes, plus prepared.twice/head.map.twice controls. Checks::ALL and unconditional FaultProbe checks retained. Fold/Nest definitions archived for S13. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | No module → ordinary sibling mod evaluation_dispatch; no ../ path attribute. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | S2 → S3 receipt, used by marker fixtures and census. FLIP_SLOT becomes census-only. Both oracle targets compile without warnings. |
| crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv | Record all owned F/M/T/D transitions; retain F0273/M0280 removal receipts, add F0275P/M0282S; remove four raw decoder consumers; add S3 and X0040 finalizer-wire fixture receipt. Every changed row is in the receipt-transition artifact. |
| crates/rum-stdlib/tests/suspension_oracle/support/drive.rs | Retain the exact execute wrapper required by nine historical integration families through path aliases. Add a function-local allow(dead_code) for the oracle target, which calls execute_planned directly. This corrects the earlier erroneous removal; execute_planned and all comparisons remain unchanged. |
| crates/rum-stdlib/tests/typed_slot_census.rs | Scratch brief-608 → brief-609; S2 → S3 receipt and negative-preflip labels. Scope FLIP_SLOT here; connect s3 module. Guard-removal legacy counts 564/59 → 553/58; removed-frame sentinel remains 622. Both scratch falsifiers verified in both builds. |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | Production legacy frames/finalizers 553/54 → 542/53, total 607 unchanged; macro result/insert 345/244 → 353/236, total 589 unchanged. Receipt row totals F623 → F624 and M600 → M601 retain merged rows plus new split identities. Closed-set S3 removal proof precedes skipping removed source anchors; all other exact source/mode checks remain. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | Scratch brief-608 → brief-609. Removed Whole0029–Whole0032 emit explicit S3 transition records, assert absence of the old identity, and preserve decoder IDs; live decoder rows still require rediscovery. |
| crates/rum-stdlib/tests/typed_slot_census/routes.rs | I0003 still names ContinueToken::suspend and its executable repeat witness. Update mechanism output to disclose typed S3 evaluation while later families remain untyped; S2 mechanism ownership is unchanged. |
| crates/rum-stdlib/tests/typed_slot_census/s3.rs | New closed-set gate for exactly 12 typed implementations / 13 typed macros, input/output pairs, 8 approved removals, absent T0023 wrapper/callers, relocated maps and witnessed token repeat. |
| crates/rum-stdlib/tests/typed_slot_compile_controls.rs | Scratch directory brief609; actual producer witnesses added to repeat/finalizer controls. Add token-witness-input, changing only producer Expr → Option<Expr>, requiring one E0308 at the complete witness span. 31 positive/intended-negative pairs pass in each build. |

The brief is append-only after the redispatch-3 amendment. All prior report/amendment bytes are preserved. No module visibility widening was needed. Registry and typed-slot-shapes.tsv remain unchanged; S3 has no State births. The exact cumulative source patch is `.temp/brief-609/r3/candidate.patch`. No git command, network operation, unsafe code, dependency, lockfile, toolchain, core representation or unrelated documentation change was made.

Size assessment: new modules remain below 10k characters: crates/rum-stdlib/tests/typed_slot_census/s3.rs=5206, crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs=4327, crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs=6540. Existing split-plan modules retain their semantic boundaries: argument preparation versus Legacy wire provider, dispatch loops versus resumed consumers, adapter families, and compiler fixture definitions versus diagnostic harness. No unrelated split or TODO change was made. All touched Rust sizes are in final-audit.json.

### 2. Owned-row reconciliation and census

| F row | S2 type | S3 transition |
| --- | --- | --- |
| F0204 | BuiltinEvaluateSuspensionFrame | Typed Expr → Expr; same name. |
| F0272 | PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame | Typed Expr → PreparedCallArguments; also receives fresh repeated state formerly handled by F0273. |
| F0273 | ResumeSuspendedCallArgumentPreparationSuspensionFrame | Merged into F0272; no remaining production implementation of the old name. |
| F0274 | EvaluateCallSuspensionFrame | Typed Expr → Expr; head conversion at the shared entry. |
| F0275 | ResumeCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch split to PreparedDispatchFrame. |
| F0276 | ResumeSuspendedCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch uses the same PreparedDispatchFrame. |
| F0277 | EvaluateStoredOwnValueSuspensionFrame | Typed CompiledMatchOutcome → Expr. |
| F0278 | RunCallDispatchStageSuspensionFrame | Typed Expr → Option<Expr>; complete the semantic Some map. |
| F0279 | EvalReleaseHoldSuspensionFrame | Typed Expr → Expr. |
| F0526 | EvaluateProfiledFixedPointSuspensionFrame | Typed Expr → Expr. |
| F0527 | ResumeFixedPointFromSuspensionFrame | Typed Expr → Expr. |
| F0570 | FinishPreparedCallArgumentsFinalizer | SlotFinalizer Output=PreparedCallArguments. |

| M row | Actual producer | Ordinary transformation | Resumed transformation |
| --- | --- | --- | --- |
| M0205 | Expr | builtin_evaluate returns the evaluated value unchanged | Typed identity frame; no post-witness semantic map. |
| M0279 | Expr | append/splice/drop-Nothing; preserve Unevaluated entries; advance remaining arguments; build PreparedCallArguments | Typed F0272 reentry uses the same append helper and fresh cursor/splice fields; pure finalizer builds the exact output after Arc release. |
| M0280 | Expr | the same append and cursor/restore work for later arguments | Old macro removed by D0001; witness of actual evaluated_result is passed to token.suspend with fresh F0272. No original Arc is captured. |
| M0281 | Expr | SharedExpr conversion, plan construction and dispatch; profiling records | Witness now at evaluate(head.into_owned()); shared resume_call_after_head accepts Expr or existing SharedExpr and performs Into<SharedExpr> once. |
| M0282 | PreparedCallArguments or Option<Expr> (split) | normalize prepared args, install restore; stage Some returns value, None advances cursor | Prepare macro → PreparedDispatchFrame; stage macro → shared finish_call_dispatch_stage, F0275. Shared normalization/restore helper runs once. |
| M0283 | PreparedCallArguments or Option<Expr> (split) | same work with the suspended plan and frozen restore | Prepare macro → PreparedDispatchFrame; shared stage macro → F0276; restore freezes after the same helper. |
| M0285 | CompiledMatchOutcome | matched branch evaluates own value; unmatched branch resumes symbol lookup | Same branches in F0277; only the Expr transport map and raw input decode disappear. |
| M0286 | Expr | Some(value) after evaluating the threaded call | F0278 now completes resumed.map(Some); three listable fixtures watched fail → pass. |
| M0287 | Expr | profile the builtin result, then result.map(Some) | Same Some completion in unchecked F0278; suspension records the existing profiling event before the typed macro returns. |
| M0288 | Expr | profile, memory-growth validation, then result.map(Some) | Checked F0278 validates the Expr result and then wraps Some. The If-held fixtures exercise the checked test build; the unchecked constructor shares this completion body. Its release path has not yet been measured. |
| M0289 | Expr | ReleaseHold evaluation result unchanged | Typed identity frame. |
| M0561 | Expr | fixed-point equality/cycle history/trace update and next pass | Typed F0526 uses outcome? then the unchanged resume_fixed_point_from state transition. |
| M0562 | Expr | same fixed-point transition in resumed evaluation | Typed F0527 uses outcome? then the same helper. |

F0273 merges into F0272 for same-type fresh token repeats. New F0275P PreparedDispatchFrame consumes PreparedCallArguments and is shared by both call loops. M0280 becomes the witnessed token route; M0282S is the shared Option-stage result macro. T0023 removes the unreachable generic PrepareArguments body and wrapper, with a parsed no-definition/no-caller gate. T0024 witnesses Expr before the common SharedExpr conversion. T0025/T0026 split prepared/stage consumers and share normalization/restore installation. T0027 removes only transport encoding/decoding. D0001 preserves cursor/splice state, mandatory actual producer evidence and finalization after both owners drop. Modes remain Expr Value plus21 Legacy;23 whole-slot Legacy types remain.

```text
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=622 macros=600
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S3 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=8 result=3 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
```

Both census builds passed5/0/1ignored; the ignored probe is executed explicitly in two bounded scratch falsifiers, both of which fail as intended. Production 607 implementations include 11 TypedFrame and 1 SlotFinalizer; 589 macros include 13 typed. Test-only 16 implementations / 11 macros remain. Retained removal receipts explain inventory F624/M601; X0040 records the existing finalizer-wire fixture. Exact receipt transitions follow.

| Receipt | Status old → new | Final identity / anchors |
| --- | --- | --- |
| F0204 | untyped → typed | rum_runtime::runtime::builtin_adapters::resume_frames::BuiltinEvaluateSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0272 | untyped → typed | rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0273 | untyped → removed | rum_runtime::runtime::eval_call_prepare::resume_frames::ResumeSuspendedCallArgumentPreparationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame; F0272 |
| F0274 | untyped → typed | rum_runtime::runtime::eval_core::call::EvaluateCallSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0275 | untyped → typed | rum_runtime::runtime::eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0276 | untyped → typed | rum_runtime::runtime::eval_core::call::ResumeSuspendedCallWithDispatchPlanSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0277 | untyped → typed | rum_runtime::runtime::eval_core::symbol::EvaluateStoredOwnValueSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0278 | untyped → typed | rum_runtime::runtime::eval_dispatch::RunCallDispatchStageSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0279 | untyped → typed | rum_runtime::runtime::hold_thread::EvalReleaseHoldSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0526 | untyped → typed | rum_runtime::runtime::resolution::resume_frames::EvaluateProfiledFixedPointSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0527 | untyped → typed | rum_runtime::runtime::resolution::resume_frames::ResumeFixedPointFromSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0570 | untyped → typed | rum_runtime::runtime::eval_call_prepare::resume_frames::FinishPreparedCallArgumentsFinalizer implements rum_runtime::runtime::suspension::typed::SlotFinalizer; - |
| M0205 | untyped → typed | rum_runtime::runtime::builtin_adapters::builtin_evaluate#macro1:resume_typed_or_suspend:result; - |
| M0279 | untyped → typed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::prepare_arguments_for_call_with_unevaluated_restore#macro1:resume_typed_or_suspend:result; - |
| M0280 | untyped → removed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::resume_suspended_call_argument_preparation#macro1:resume_or_suspend:insert; F0272 |
| M0281 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::evaluate_call#macro1:resume_typed_or_suspend:result; - |
| M0282 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#macro1:resume_typed_or_suspend:result; - |
| M0283 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#macro1:resume_typed_or_suspend:result; - |
| M0285 | untyped → typed | rum_runtime::runtime::eval_core::symbol::<Session as inherent>::evaluate_stored_own_value#macro1:resume_typed_or_suspend:result; - |
| M0286 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::run_call_dispatch_stage#macro1:resume_typed_or_suspend:result; - |
| M0287 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::try_dispatch_builtin_call#macro1:resume_typed_or_suspend:result; - |
| M0288 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::try_dispatch_builtin_call#macro2:resume_typed_or_suspend:result; - |
| M0289 | untyped → typed | rum_runtime::runtime::hold_thread::<Session as inherent>::eval_release_hold#macro1:resume_typed_or_suspend:result; - |
| M0561 | untyped → typed | rum_runtime::runtime::resolution::<Session as inherent>::evaluate_profiled_fixed_point#macro1:resume_typed_or_suspend:result; - |
| M0562 | untyped → typed | rum_runtime::runtime::resolution::<Session as inherent>::resume_fixed_point_from#macro1:resume_typed_or_suspend:result; - |
| D0001 | untyped → implemented | deferred::prepared arguments; F0204;F0272;F0273;F0274;F0275;F0276;F0277;F0278;F0279;F0526;F0527;F0570;F0275P |
| T0023 | untyped → removed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::prepare_arguments_for_call#method1:map:self . prepare_arguments_for_call_with_unevaluated_restore (head , attributes , args); F0272 |
| T0024 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_after_head#method1:into:evaluated_head; F0274 |
| T0025 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#method16:map:prepared; F0275;F0275P |
| T0026 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#method16:map:prepared; F0276;F0275P |
| T0027 | untyped → removed | rum_runtime::runtime::eval_core::symbol::<EvaluateStoredOwnValueSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#method4:map:(\| \| -> Result < rum_core :: Expr , EvalError > { if outcome . matched { session . evaluate_accepted_own_value (& symbol , body , delayed) } else { session . evaluate_symbol_after_stored_own_value (symbol) } }) (); F0277 |
| Whole0029 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call1:prepared_call_arguments_from_continuation_expr:; D009 |
| Whole0030 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call4:from_continuation_expr:; D125 |
| Whole0031 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call5:from_continuation_expr:; D125 |
| Whole0032 | legacy-use → removed | rum_runtime::runtime::eval_core::symbol::<EvaluateStoredOwnValueSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr:; D049 |
| F0275P | absent → typed | rum_runtime::runtime::eval_core::call::PreparedDispatchFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; F0275;F0276 |
| M0282S | absent → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::finish_call_dispatch_stage#macro1:resume_typed_or_suspend:result; M0282;M0283 |
| S3 | absent → implemented | S3 evaluation dispatch: 12 typed implementations and 13 typed macro sites; T0023;T0024;T0025;T0026;T0027;D0001 |
| X0040 | absent → legacy-test | rum_runtime::runtime::eval_call_prepare::prepared_slot_finalizer_fixture#method1:into_continuation_expr:finish_suspended_prepared_call (SuspendedPreparedCallArguments { evaluated_args : PreparedArgumentAccumulator :: freeze (prepared . evaluated_args . iter () . cloned () . collect () ,) , unevaluated_restores : prepared . unevaluated_restore . as_ref () . map (\| restore \| UnevaluatedRestoreAccumulator :: freeze (restore . wrappers . clone ())) , }); S3 typed finalizer materialized through the unchanged Legacy provider |

### 3. Watched divergence failures and final equality

The six replacement S3 cases were watched failing on byte-identical S2 production in both builds: three If-held marker shapes and three listable-completion marker shapes. The director reassigned the original Fold/Nest cases to S13; their raw failures remain preserved under brief609, and F0528/M0563 are unchanged. The pre-conversion failures below are retained evidence, not newly rerun baselines.

**r1-pre-builtin-held-default: exit 101.** Full raw log: `.temp/brief-609/r1-pre-builtin-held-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.29s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-builtin-held-no-default: exit 101.** Full raw log: `.temp/brief-609/r1-pre-builtin-held-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.13s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 661, identity: None, text: SymbolText { id: 661, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-default: exit 101.** Full raw log: `.temp/brief-609/r1-pre-threaded-dv-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.35s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 673, identity: None, text: SymbolText { id: 673, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-no-default: exit 101.** Full raw log: `.temp/brief-609/r1-pre-threaded-dv-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.46s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

Final full-oracle runs pass all six strict S3 cases plus prepared.twice and head.map.twice in both builds, under Checks::ALL and unconditional FaultProbe absence. All 28 immutable brief606 ordinary controls also pass in each build; each run has 336 zero-fault observations. Full oracle totals are58 / 58 per build, final 88016 observations per build, every occurrence delta 0. Neither oracle build emits a warning.

**r3-ordinary-default-serial: exit 0.** Full raw log: `.temp/brief-609/r3-ordinary-default-serial.log`.

```text
cargo test --manifest-path .temp/brief-609/r2/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
```

```text
     Running witness_test.rs (.temp/brief-609/r2/ordinary-controls/target/debug/deps/ordinary-5cf883959ece1e3c)
running 1 test
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

**r3-ordinary-no-default-serial: exit 0.** Full raw log: `.temp/brief-609/r3-ordinary-no-default-serial.log`.

```text
cargo test --manifest-path .temp/brief-609/r2/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
```

```text
     Running witness_test.rs (.temp/brief-609/r2/ordinary-controls/target/debug/deps/ordinary-8f58f702763e7810)
running 1 test
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

Final oracle default

```text
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=17175
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=17187
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=17199
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=17211
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=17223
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=17235
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=17247
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=17259
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88016
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.47s
```

Final oracle no-default

```text
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=17175
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=17187
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=17199
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=17211
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=17223
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=17235
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=17247
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=17259
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88016
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.27s
```

### 4. Gates, frontier proof and inventory/certificate replays

The checked frontier adapter now visits invocation syntax in the enclosing function. The four expansion operations remain explicit RET002 rejections; no ownership admission is inferred. Legacy macro invocations remain opaque. C01/C02 have no converted macro body, and every local analyzer Rust file is byte-identical to S2. Parsing is separate because the existing local adapter accepts one identifier while the typed macro accepts a full Rust pattern.

**r3-pre-visibility: exit 101.** Full raw log: `.temp/brief-609/r3-pre-visibility.log`.

```text
cargo test --manifest-path .temp/brief-609/r3/frontier-fixture/Cargo.toml --test frontier visibility -- --nocapture
```

```text
     Locking 4 packages to latest Rust 1.85.0 compatible versions
      Adding syn v2.0.119 (available: v3.0.4)
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling syn v2.0.119
   Compiling brief609-frontier-fixture v0.0.0 (/home/remi/dev/rum/.temp/brief-609/r3/frontier-fixture)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.65s
     Running main.rs (.temp/brief-609/r3/frontier-fixture/target/debug/deps/frontier-a1f280449b6ff9ca)

running 1 test
test checker::visibility ... RET001	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:87-1:100	std::vec::Vec::new	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	not a resolved function definition: std::vec::Vec::new
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:113-1:180	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	primitive operand proof or exact operator/conversion impl required
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:302-1:324	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	primitive operand proof or exact operator/conversion impl required
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:327-1:385	rum_runtime::runtime::model::session::Session::enforce_active_resource_limits_for_expr	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: opaque imported trait candidate std::sync::Arc
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:327-1:387	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	implicit Try/FromResidual/error ownership not modeled
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:761-1:787	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: method receiver lacks checked nominal/place provenance
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:761-1:830	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: method receiver lacks checked nominal/place provenance
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:795-1:829	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	closure capture/callback effects unresolved; closure body is a rejected boundary
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:891-1:913	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:923-1:967	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: method receiver lacks checked nominal/place provenance
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:948-1:966	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1141-1:1159	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1310-1:1620	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	implicit Try/FromResidual/error ownership not modeled
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1728-1:1743	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	primitive operand proof or exact operator/conversion impl required
RET003	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1310-1:1618	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	unexpanded macro; contents are not resolved Rust

thread 'checker::visibility' panicked at checker/mod.rs:43:5:
typed macro arm hid the rejected std::mem::forget operation
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    checker::visibility

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 9.44s

error: test failed, to rerun pass `--test frontier`
```

**r3-pre-drift: exit 101.** Full raw log: `.temp/brief-609/r3-pre-drift.log`.

```text
cargo test --manifest-path .temp/brief-609/r3/frontier-fixture/Cargo.toml --test frontier definition_drift -- --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running main.rs (.temp/brief-609/r3/frontier-fixture/target/debug/deps/frontier-a1f280449b6ff9ca)

running 1 test
test checker::definition_drift ... 
thread 'checker::definition_drift' panicked at checker/mod.rs:66:18:
altered typed macro definition did not fail the adapter check
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    checker::definition_drift

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 9.41s

error: test failed, to rerun pass `--test frontier`
```

**r3-pre-outside: exit 0.** Full raw log: `.temp/brief-609/r3-pre-outside.log`.

```text
cargo test --manifest-path .temp/brief-609/r3/frontier-fixture/Cargo.toml --test frontier rejected_operation_outside_macro -- --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running main.rs (.temp/brief-609/r3/frontier-fixture/target/debug/deps/frontier-a1f280449b6ff9ca)

running 1 test
test checker::rejected_operation_outside_macro ... RET002	crates/rum-runtime/src/runtime/resolution.rs:1:3-1:27	std::mem::forget	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	not a resolved function definition: std::mem::forget
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 9.30s
```

**r3-final-falsifiers: exit 0.** Full raw log: `.temp/brief-609/r3-final-falsifiers.log`.

```text
cargo test --manifest-path .temp/brief-609/r3/frontier-fixture/Cargo.toml --test frontier -- --nocapture
```

```text
   Compiling brief609-frontier-fixture v0.0.0 (/home/remi/dev/rum/.temp/brief-609/r3/frontier-fixture)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.32s
     Running main.rs (.temp/brief-609/r3/frontier-fixture/target/debug/deps/frontier-a1f280449b6ff9ca)

running 4 tests
test checker::census ... ok
test checker::definition_drift ... 
thread 'checker::definition_drift' panicked at checker/regions.rs:302:21:
crates/rum-runtime/src/runtime/resolution.rs:377:24-389:14: typed resume macro definition changed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
DRIFT rejected by checked adapter: crates/rum-runtime/src/runtime/resolution.rs:377:24-389:14: typed resume macro definition changed
ok
test checker::rejected_operation_outside_macro ... RET002	crates/rum-runtime/src/runtime/resolution.rs:1:3-1:27	std::mem::forget	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	not a resolved function definition: std::mem::forget
ok
test checker::visibility ... RET001	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:87-1:100	std::vec::Vec::new	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	not a resolved function definition: std::vec::Vec::new
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:113-1:180	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	primitive operand proof or exact operator/conversion impl required
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:302-1:324	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	primitive operand proof or exact operator/conversion impl required
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:327-1:385	rum_runtime::runtime::model::session::Session::enforce_active_resource_limits_for_expr	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: opaque imported trait candidate std::sync::Arc
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:327-1:387	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	implicit Try/FromResidual/error ownership not modeled
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:761-1:787	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: method receiver lacks checked nominal/place provenance
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:761-1:830	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: method receiver lacks checked nominal/place provenance
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:795-1:829	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	closure capture/callback effects unresolved; closure body is a rejected boundary
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:891-1:913	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:923-1:967	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: method receiver lacks checked nominal/place provenance
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:948-1:966	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1141-1:1159	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1310-1:1618	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	typed macro expansion: delegated InputRequired reconstruction and request/error ownership unresolved
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1310-1:1618	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	typed macro expansion: owned Result match, Ok/InputRequired/error bindings, Err(error) forwarding and local drops unresolved
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1310-1:1618	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	typed macro expansion: owned producer binding and slot_witness(&produced) borrow/call unresolved
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1310-1:1618	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	typed macro expansion: return through witness.suspend; typed insertion and frame ownership transfer unresolved
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1310-1:1620	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	implicit Try/FromResidual/error ownership not modeled
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1407-1:1431	std::mem::forget	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	not a resolved function definition: std::mem::forget
RET002	/home/remi/dev/rum/.temp/brief-609/r3/visibility-body.rs:1:1728-1:1743	-	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	primitive operand proof or exact operator/conversion impl required
ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.15s
```

Historical frontier 932 →936 = RET001 10 + RET002 900 + RET003 26.918 rows are byte-identical, including the freeze/default rejection at resolution.rs:35.13 rows pair by exact operation and column after a two-line move. M0561 old opaque span385:21–385:112 becomes four checked expansion rows at377:24–389:14. One S3 Try/FromResidual row is added at377:24–389:15. Revealed-on-conversion rows: 0 for this root. Unpaired removals: 0. The historical ACCEPTANCE_BASELINE print is unchanged. Full old/new records and source snippets: r3/frontier-classified-diff.json.

| Operation | Old span | New span |
| --- | --- | --- |
| *remaining | 482:9 | 480:9 |
| *remaining | 479:12 | 477:12 |
| *remaining -= 1 | 482:9 | 480:9 |
| *remaining == 0 | 479:12 | 477:12 |
| args.iter() | 490:17 | 488:17 |
| walk(expr, &mut remaining) | 496:5 | 494:5 |
| walk(head, remaining) | 487:21 | 485:21 |
| !walk(head, remaining) | 487:20 | 485:20 |
| \|arg\| walk(arg, remaining) | 490:33 | 488:33 |
| limit - remaining | 496:42 | 494:42 |
| args.iter().all(\|arg\| walk(arg, remaining)) | 490:17 | 488:17 |
| walk(expr, &mut remaining).then_some(limit - remaining) | 496:5 | 494:5 |
| expr_node_count_with_limit(expr, EVALUATION_CYCLE_HISTORY_NODE_LIMIT).is_some() | 474:5 | 472:5 |

Ownership discovery retains 79 approved host-payload identities and 59 self-reaching types. UnknownConstruct 7712 →7719 follows owned S3 captures/macros; every row is disclosed in ownership-replay.patch. unknown_storage.tsv stays byte-identical. Four adapter witness cells are replayed from ADAPTER_WITNESS output: three concrete preparation-frame paths and one concrete finalizer path replace interim public-bound/no-instantiation text. All identity, disposition, owed_by and external coverage cells remain unchanged.

```text
unknown_attributes.tsv: 1514 -> 1514 lines
unknown_captures.tsv: 2954 -> 2959 lines
unknown_macros.tsv: 3150 -> 3152 lines
unknown_storage.tsv: 94 -> 94 lines
host_payloads.tsv: 79 -> 79 lines
self_reaching.tsv: 59 -> 59 lines
host_payload_ledger.tsv: 80 -> 80 lines
```

| Row | Old witness | Emitted witness |
| --- | --- | --- |
| rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame | rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet) | rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame => rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame (impl at crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:13:1); rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame.state -> rum_runtime::runtime::eval_call_prepare::SuspendedCallArgumentPreparation ; rum_runtime::runtime::eval_call_prepare::SuspendedCallArgumentPreparation.head -> rum_core::expr::Expr |
| rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer | rum_runtime::runtime::suspension::typed::action::ErasedFinalizer => rum_runtime::runtime::suspension::typed::adapter::FinalizerAdapter.0 (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:82:1) => rum_runtime::runtime::suspension::typed::SlotFinalizer (public bound; no in-tree instantiation exists yet) | rum_runtime::runtime::suspension::typed::action::ErasedFinalizer => rum_runtime::runtime::suspension::typed::adapter::FinalizerAdapter.0 (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:82:1) => rum_runtime::runtime::suspension::typed::SlotFinalizer => rum_runtime::runtime::eval_call_prepare::resume_frames::FinishPreparedCallArgumentsFinalizer (impl at crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:43:1); rum_runtime::runtime::eval_call_prepare::resume_frames::FinishPreparedCallArgumentsFinalizer.state -> rum_runtime::runtime::eval_call_prepare::SuspendedPreparedCallArguments ; rum_runtime::runtime::eval_call_prepare::SuspendedPreparedCallArguments.evaluated_args -> rum_runtime::runtime::call_preparation::PreparedArgumentAccumulator ; rum_runtime::runtime::call_preparation::PreparedArgumentAccumulator.prefix -> rum_core::expr::SharedArgs |
| rum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame | rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet) | rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame => rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame (impl at crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:13:1); rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame.state -> rum_runtime::runtime::eval_call_prepare::SuspendedCallArgumentPreparation ; rum_runtime::runtime::eval_call_prepare::SuspendedCallArgumentPreparation.head -> rum_core::expr::Expr |
| rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame | rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame (public bound; no in-tree instantiation exists yet) | rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:36:1) => rum_runtime::runtime::suspension::typed::TypedFrame => rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame (impl at crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:13:1); rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame.state -> rum_runtime::runtime::eval_call_prepare::SuspendedCallArgumentPreparation ; rum_runtime::runtime::eval_call_prepare::SuspendedCallArgumentPreparation.head -> rum_core::expr::Expr |

All 68 contract fingerprints and four C01/C02 certificate cells were replayed twice: initially, then after restoring the shared execute wrapper. The emission runs reject stale pins after producing zero-rejection certificates; final acceptance uses the replayed cells. All first 18 contract fields and seven noncertificate promoted columns are unchanged. Initial and final replay artifacts are retained; final source context is fnv1a64:070ae574dd0b586c.

**r3-census-default: exit 0.** Full raw log: `.temp/brief-609/r3-census-default.log`.

```text
cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
```

```text
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-943817a3ecf6c029)
running 6 tests
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 20.02s
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.87s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 144.31s
```

**r3-census-no-default: exit 0.** Full raw log: `.temp/brief-609/r3-census-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
```

```text
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-cce6bcaa604a405f)
running 6 tests
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.95s
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.69s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 141.53s
```

**r3-constructor-diagnostic: exit 0.** Full raw log: `.temp/brief-609/r3-constructor-diagnostic.log`.

```text
cargo test --manifest-path .temp/brief-609/r3/frontier-fixture/Cargo.toml constructor_oracle_visibility_diagnostic -- --nocapture
```

```text
     Running main.rs (.temp/brief-609/r3/frontier-fixture/target/debug/deps/frontier-a1f280449b6ff9ca)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 12.19s
```

**r3-controls-default: exit 0.** Full raw log: `.temp/brief-609/r3-controls-default.log`.

```text
cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-d6fbf6132444c3d1)

running 1 test
test typed_slot_compile_controls ... PAIR first-chain-input: positive PASS; negative E0271 at src/lib.rs:27:35
PAIR chain-adjacency: positive PASS; negative E0271 at src/lib.rs:34:70
PAIR unbound-chain-constructor: positive PASS; negative E0599 at src/lib.rs:40:41
PAIR evaluate-input: positive PASS; negative E0271 at src/lib.rs:46:66
PAIR evaluate-output: positive PASS; negative E0271 at src/lib.rs:52:66
PAIR finalizer-output: positive PASS; negative E0271 at src/lib.rs:58:48
PAIR opaque-action: positive PASS; negative E0599 at src/lib.rs:63:39
PAIR token-no-clone: positive PASS; negative E0599 at src/lib.rs:69:18
PAIR token-no-mint: positive PASS; negative E0599 at src/lib.rs:75:27
PAIR token-no-escape: positive PASS; negative lifetime at src/lib.rs:81:1
PAIR sealed-extension: positive PASS; negative E0277 at src/lib.rs:87:20
PAIR codec-key-private: positive PASS; negative E0603 at src/lib.rs:93:18
PAIR opaque-selector-pass: positive PASS; negative E0271 at src/lib.rs:100:35
PAIR mode-not-frame-chosen: positive PASS; negative E0599 at src/lib.rs:106:65
PAIR mode-private-selection: positive PASS; negative E0603 at src/lib.rs:112:18
PAIR slot-mismatch-private: positive PASS; negative E0451 at src/lib.rs:124:95
PAIR input: positive PASS; negative E0271 at src/lib.rs:131:41
PAIR output-return: positive PASS; negative E0308 at src/lib.rs:138:1
PAIR chain-last-output: positive PASS; negative E0308 at src/lib.rs:145:1
PAIR token-fresh-type: positive PASS; negative E0308 at src/lib.rs:159:47
PAIR token-witness-input: positive PASS; negative E0308 at src/lib.rs:167:17
PAIR deferred-final-output: positive PASS; negative E0308 at src/lib.rs:174:5
PAIR generic-frame: positive PASS; negative E0308 at src/lib.rs:180:1
PAIR closure-return: positive PASS; negative E0308 at src/lib.rs:187:1
PAIR sibling-erased-private: positive PASS; negative E0603 at crates/rum-runtime/src/runtime/mod.rs:135:47
PAIR mode-expr-fixed: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17
PAIR mode-new-state-only: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed.rs:66:40
PAIR mode-one-associated-type: positive PASS; negative E0201 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-no-codec-override: positive PASS; negative E0407 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-legacy-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:74:19
PAIR mode-state-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:79:15
COMPILE CONTROLS: 31 positive / 31 intended rejections (22 S1 + 7 S2 + opaque fault + token witness); raw Cargo JSON and primary spans retained
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 80.71s
```

**r3-controls-no-default: exit 0.** Full raw log: `.temp/brief-609/r3-controls-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.77s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-e5d376d145e23731)

running 1 test
test typed_slot_compile_controls ... PAIR first-chain-input: positive PASS; negative E0271 at src/lib.rs:27:35
PAIR chain-adjacency: positive PASS; negative E0271 at src/lib.rs:34:70
PAIR unbound-chain-constructor: positive PASS; negative E0599 at src/lib.rs:40:41
PAIR evaluate-input: positive PASS; negative E0271 at src/lib.rs:46:66
PAIR evaluate-output: positive PASS; negative E0271 at src/lib.rs:52:66
PAIR finalizer-output: positive PASS; negative E0271 at src/lib.rs:58:48
PAIR opaque-action: positive PASS; negative E0599 at src/lib.rs:63:39
PAIR token-no-clone: positive PASS; negative E0599 at src/lib.rs:69:18
PAIR token-no-mint: positive PASS; negative E0599 at src/lib.rs:75:27
PAIR token-no-escape: positive PASS; negative lifetime at src/lib.rs:81:1
PAIR sealed-extension: positive PASS; negative E0277 at src/lib.rs:87:20
PAIR codec-key-private: positive PASS; negative E0603 at src/lib.rs:93:18
PAIR opaque-selector-pass: positive PASS; negative E0271 at src/lib.rs:100:35
PAIR mode-not-frame-chosen: positive PASS; negative E0599 at src/lib.rs:106:65
PAIR mode-private-selection: positive PASS; negative E0603 at src/lib.rs:112:18
PAIR slot-mismatch-private: positive PASS; negative E0451 at src/lib.rs:124:95
PAIR input: positive PASS; negative E0271 at src/lib.rs:131:41
PAIR output-return: positive PASS; negative E0308 at src/lib.rs:138:1
PAIR chain-last-output: positive PASS; negative E0308 at src/lib.rs:145:1
PAIR token-fresh-type: positive PASS; negative E0308 at src/lib.rs:159:47
PAIR token-witness-input: positive PASS; negative E0308 at src/lib.rs:167:17
PAIR deferred-final-output: positive PASS; negative E0308 at src/lib.rs:174:5
PAIR generic-frame: positive PASS; negative E0308 at src/lib.rs:180:1
PAIR closure-return: positive PASS; negative E0308 at src/lib.rs:187:1
PAIR sibling-erased-private: positive PASS; negative E0603 at crates/rum-runtime/src/runtime/mod.rs:135:47
PAIR mode-expr-fixed: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17
PAIR mode-new-state-only: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed.rs:66:40
PAIR mode-one-associated-type: positive PASS; negative E0201 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-no-codec-override: positive PASS; negative E0407 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-legacy-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:74:19
PAIR mode-state-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:79:15
COMPILE CONTROLS: 31 positive / 31 intended rejections (22 S1 + 7 S2 + opaque fault + token witness); raw Cargo JSON and primary spans retained
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.83s
```

**r3-emit-local-default: exit 101.** Full raw log: `.temp/brief-609/r3-emit-local-default.log`.

```text
cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.97s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**r3-emit-local-final-default: exit 101.** Full raw log: `.temp/brief-609/r3-emit-local-final-default.log`.

```text
cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.62s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**r3-emit-local-final-no-default: exit 101.** Full raw log: `.temp/brief-609/r3-emit-local-final-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c942ac40233a21b0)
running 1 test
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 19.13s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**r3-emit-local-no-default: exit 101.** Full raw log: `.temp/brief-609/r3-emit-local-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c942ac40233a21b0)
running 1 test
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 19.62s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**r3-fmt-check: exit 0.** Full raw log: `.temp/brief-609/r3-fmt-check.log`.

```text
cargo fmt --check
```

```text
(no output)
```

**r3-format: exit 0.** Full raw log: `.temp/brief-609/r3-format.log`.

```text
cargo fmt --all
```

```text
(no output)
```

**r3-frame-root: exit 0.** Full raw log: `.temp/brief-609/r3-frame-root.log`.

```text
cargo test -p rum-runtime --lib frame_root_trace -- --nocapture
```

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.45s
```

**r3-frontier: exit 0.** Full raw log: `.temp/brief-609/r3-frontier.log`.

```text
cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
INDEPENDENT_AUDIT EDGE_ALL roots=7337 definitions=7337 raw=48338 deduplicated=48338 reason_classes=57 all_spans_read=true
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 26.23s
```

**r3-local-default: exit 0.** Full raw log: `.temp/brief-609/r3-local-default.log`.

```text
cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 19.36s
```

**r3-local-no-default: exit 0.** Full raw log: `.temp/brief-609/r3-local-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c942ac40233a21b0)
running 1 test
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 19.01s
```

**r3-oracle-final-script: exit 0.** Full raw log: `.temp/brief-609/r3-oracle-final-script.log`.

```text
bash .temp/brief-609/r3/check-suspension-oracle-final.sh 1500
```

```text
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.47s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.27s
```

**r3-oracle-script: exit 0.** Full raw log: `.temp/brief-609/r3-oracle-script.log`.

```text
bash .temp/brief-609/r3/check-suspension-oracle.sh 1500
```

```text
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.87s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.18s
```

**r3-ordinary-default: exit 0.** Full raw log: `.temp/brief-609/r3-ordinary-default.log`.

```text
cargo test --manifest-path .temp/brief-609/r2/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
```

```text
     Running witness_test.rs (.temp/brief-609/r2/ordinary-controls/target/debug/deps/ordinary-5cf883959ece1e3c)
running 1 test
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

**r3-ordinary-no-default: exit 0.** Full raw log: `.temp/brief-609/r3-ordinary-no-default.log`.

```text
cargo test --manifest-path .temp/brief-609/r2/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
```

```text
     Running witness_test.rs (.temp/brief-609/r2/ordinary-controls/target/debug/deps/ordinary-8f58f702763e7810)
running 1 test
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

**r3-ownership: exit 101.** Full raw log: `.temp/brief-609/r3-ownership.log`.

```text
cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 25 tests
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7719 rows
test result: FAILED. 23 passed; 2 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 72.42s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**r3-ownership-discover: exit 0.** Full raw log: `.temp/brief-609/r3-ownership-discover.log`.

```text
cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 5.92s
```

**r3-ownership-replayed: exit 0.** Full raw log: `.temp/brief-609/r3-ownership-replayed.log`.

```text
cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 25 tests
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7719 rows
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 73.66s
```

**r3-post-falsifiers: exit 0.** Full raw log: `.temp/brief-609/r3-post-falsifiers.log`.

```text
cargo test --manifest-path .temp/brief-609/r3/frontier-fixture/Cargo.toml --test frontier -- --nocapture
```

```text
     Running main.rs (.temp/brief-609/r3/frontier-fixture/target/debug/deps/frontier-a1f280449b6ff9ca)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.10s
```

**r3-stop-fmt: exit 0.** Full raw log: `.temp/brief-609/r3-stop-fmt.log`.

```text
cargo fmt --check
```

```text
(no output)
```

**r3-typed-runtime: exit 0.** Full raw log: `.temp/brief-609/r3-typed-runtime.log`.

```text
cargo test -p rum-runtime --lib suspension::typed::tests -- --nocapture
```

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
running 20 tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.58s
```

**r3-workspace: exit 101.** Full raw log: `.temp/brief-609/r3-workspace.log`.

```text
cargo test --workspace --no-fail-fast -- --nocapture
```

```text
error: could not compile `rum-stdlib` (test "destructuring_continuations") due to 2 previous errors
```

**r3-workspace-final: exit 101.** Full raw log: `.temp/brief-609/r3-workspace-final.log`.

```text
cargo test --workspace --no-fail-fast -- --nocapture
```

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
     Running unittests src/main.rs (target/debug/deps/rum_cli-0aee412e0b99cbf1)
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/cli.rs (target/debug/deps/cli-bc0b4c8761b62ae9)
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/input_requests.rs (target/debug/deps/input_requests-e84d0b9e62f1dd0f)
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/terminal.rs (target/debug/deps/terminal-70e027c8b6d403e3)
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
     Running unittests src/lib.rs (target/debug/deps/rum_core-05ee71324ea83d3f)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-058a94fcf3064f03)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-c657e563db7a31de)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-cb1c0b1fa13c3a60)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-0c15fc615add95e2)
running 90 tests
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-75627f143cb45b08)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-f1d7a1c822224b1c)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/capture_output_lifecycle.rs (target/debug/deps/capture_output_lifecycle-3426e74b0e43035a)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-3f3ebe72da026eee)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-52956b33f57a0130)
running 1 test
error: could not compile `host-payload-legacy-impl` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-new` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-from-arc` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-file-io-impl` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-file-io-new` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-file-io-from-arc` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-raw-closure-sink` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-boxed-fnmut-sink` (lib) due to 1 previous error
error: could not compile `host-payload-named-sink-without-marker` (lib) due to 1 previous error
error: could not compile `host-payload-adapter-private-field-bypass` (lib) due to 1 previous error
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 52.22s
     Running tests/installer.rs (target/debug/deps/installer-245b9a9d0dd76683)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-cf4abded19978b7b)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-7db6f3bc8fd1b373)
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-bcf408336a660b97)
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/shared_accumulator_completion.rs (target/debug/deps/shared_accumulator_completion-4fc6d427214e6aa2)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/stress.rs (target/debug/deps/stress-29e64a7d12eeb866)
running 6 tests
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-0e6a452c6cdaf889)
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.03s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-7953000542543e9e)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-598afbd7e7b200bf)
running 151 tests
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-0f1e351c2b3d67dc)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-4387fdc47fc58f50)
running 48 tests
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-11b7b14c69d5d259)
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-02bf146f80080963)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-f6425a9dcb35714f)
running 1172 tests
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: unused import: `rum_core::Symbol`
warning: unreachable call
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error; 2 warnings emitted
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error
warning: unused import: `rum_core::Symbol`
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error; 1 warning emitted
error: could not compile `option-read-proof` (bin "option-read-proof") due to 1 previous error
warning: function `consume` is never used
warning: 1 warning emitted
error: aborting due to 1 previous error
test result: FAILED. 1168 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 44.56s
error: test failed, to rerun pass `-p rum-runtime --lib`
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-8951251f7d2e445e)
running 1145 tests
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7719 rows
INDEPENDENT_AUDIT EDGE_N_FRONTIER roots=79 definitions=991 raw=72838 deduplicated=5917 reason_classes=31 all_spans_read=true
INDEPENDENT_AUDIT EDGE_ALL roots=7337 definitions=7337 raw=48338 deduplicated=48338 reason_classes=57 all_spans_read=true
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
error: could not compile `rum-runtime` (lib test) due to 84 previous errors; 47 warnings emitted
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
expired-C01: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=true
expired-C02: RED stale or omitted instance certificate C02 hash=true
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: FAILED. 1090 passed; 2 failed; 53 ignored; 0 measured; 0 filtered out; finished in 773.22s
error: test failed, to rerun pass `-p rum-stdlib --lib`
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-d2ad2c98e57a1cba)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-f117f222c1cc964c)
running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-368dcefe778b9dd9)
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-2c9bd859cff3337d)
running 47 tests
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-ef76e1f37b5e402d)
running 69 tests
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.36s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-dbe263d2d0c33a6d)
running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-13289bfd2ecc5d28)
running 100 tests
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-487a4fb3b8b94ac7)
running 71 tests
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-4182a401e6279d72)
running 40 tests
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-757402035d2c0050)
running 51 tests
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-58e5a1c5f9b651ff)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.01s
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-3f7c0defd057db37)
running 6 tests
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.10s
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.85s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.71s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-d72233bfaf7c8103)
running 1 test
COMPILE CONTROLS: 31 positive / 31 intended rejections (22 S1 + 7 S2 + opaque fault + token witness); raw Cargo JSON and primary spans retained
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.60s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-08cb56e2fd17a1b7)
running 193 tests
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-f8efeb939a755a12)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
running 7 tests
error: aborting due to 2 previous errors
error: aborting due to 2 previous errors
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
   Doc-tests rum_host
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.94s
   Doc-tests rum_jupyter
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
running 22 tests
error: aborting due to 1 previous error
error: aborting due to 2 previous errors
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.79s
   Doc-tests rum_stdlib
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 2 targets failed:
```

**Scope STOP.** `crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:34` requires every discovered constructor claim to be found by `CallAt`. Its `Visit` implementation at line 96 visits plain `ExprCall` nodes only; the default macro visitor does not parse macro tokens. The new frontier adapter exposes 11 real calls in typed macro arguments, but this independent locator cannot reach them. Repair requires checked macro traversal in that compiler-oracle visitor. This is analyzer behavior, not an update to a named path/line/count pin. The amendment authorizes `regions.rs` and one adjacent adapter, not this visitor. The governing Scope says: “If a needed change falls outside every list and class, make no edit there. STOP and report the file, the line and why.” No edit was made to `corpus.rs` or `exact.rs`.

Read-only scratch diagnosis (the diagnostic passes by reproducing the mismatch; it does not fix the gate):

```text
UNLOCATABLE	crates/rum-runtime/src/runtime/builtin_adapters.rs:937:26-937:35	rum_runtime::runtime::builtin_adapters::builtin_evaluate	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/eval_core/call.rs:138:33-138:45	rum_runtime::runtime::model::session::Session::resume_call_with_dispatch_plan	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/eval_core/call.rs:262:33-262:45	rum_runtime::runtime::model::session::Session::resume_suspended_call_with_dispatch_plan	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/eval_core/call.rs:351:32-351:41	rum_runtime::runtime::model::session::Session::finish_call_dispatch_stage	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:128:42-128:57	rum_runtime::runtime::model::session::Session::run_call_dispatch_stage	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:128:45-128:56	rum_runtime::runtime::model::session::Session::run_call_dispatch_stage	std::option::Option::Some
UNLOCATABLE	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:191:30-191:39	rum_runtime::runtime::model::session::Session::try_dispatch_builtin_call	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:206:26-206:35	rum_runtime::runtime::model::session::Session::try_dispatch_builtin_call	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/hold_thread.rs:29:26-29:35	rum_runtime::runtime::model::session::Session::eval_release_hold	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/resolution.rs:380:30-380:39	rum_runtime::runtime::model::session::Session::evaluate_profiled_fixed_point	std::result::Result::Ok
UNLOCATABLE	crates/rum-runtime/src/runtime/resolution.rs:448:30-448:39	rum_runtime::runtime::model::session::Session::resume_fixed_point_from	std::result::Result::Ok
CONSTRUCTOR_VISIBILITY claims=7315 unlocatable=11
```

**Separate permitted pin transition, identified but unapplied after STOP.** `compiler_oracle/exact.rs:169–177` still pins 172 contexts and 174 including two preserved clone contexts. The final source has 170 and 172. Exactly two old M0561 explicit `Err` constructors moved into the checked expansion; no other contextual claim disappears. Their error forwarding and InputRequired reconstruction are explicit expansion rejection rows. The precise two removed records are:

```text
crates/rum-runtime/src/runtime/resolution.rs:386:28-386:69	std::result::Result::Err	std::result::Result<rum_core::expr::Expr, rum_runtime::runtime::session::session_state::EvalError>
crates/rum-runtime/src/runtime/resolution.rs:388:38-388:48	std::result::Result::Err	std::result::Result<rum_core::expr::Expr, rum_runtime::runtime::session::session_state::EvalError>
```

**Unresolved runtime regression, verbatim:**

```text
S2 accepted workspace:
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... ok

S3 workspace:
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... 
thread 'runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds' panicked at crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:504:5:
assertion `left == right` failed
  left: "TerminatedEvaluation[\"StackLimit\"]"
 right: "done"
FAILED
```

The proposed scoped follow-up would keep the profiling closure only on the unstable/call-head branch and force the small shared stage helper to inline. This is an untested hypothesis, not a fix or a stack-cost claim. `eval_core/call.rs` is byte-identical to the saved pre-fix snapshot. The test and native stack limits remain unchanged. No such edit followed the scope STOP.

The completed workspace gate exits 101 with exactly two failing targets. The runtime target reports 1168 passed / 1 failed / 3 ignored. The stdlib target reports 1090 passed / 2 failed / 53 ignored: constructor corpus and contextual-constructor count. Remaining targets complete successfully; the census’s nested red probes are intentional. The first workspace attempt instead stopped at two E0425 errors caused by removal of the shared execute wrapper; that wrapper was restored before this completed run.

**Nine native ZeroMQ tests were skipped**, because loopback binding was unavailable. Cargo reports these early-return tests as `ok`; they are not native transport passes. Exact skip output:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

Final-source verification limits: default local certificate acceptance and default typed census ran in the completed workspace and passed. The no-default certificate acceptance and census last passed before the shared execute wrapper was restored. After restoration, the no-default zero-rejection certificate was emitted and all four cells replayed, but its final acceptance/census rerun did not occur before STOP. Both full oracle builds did run after the restoration and passed. The runtime typed/frame-root controls and both compiler-pair suites already passed; the workspace repeats their default coverage.

No new unique compiler warning appears compared with the accepted S2 workspace log. The three runtime unit-registration warnings and negative compiler-fixture warnings are pre-existing. Both final oracle builds and the scratch constructor diagnosis emit no warnings. Full warning comparison: `r3/warning-audit.json`.

Exact cumulative inventory-cell transitions from S2 are retained in `r3/local-contracts.tsv.cumulative-diff.json` and `r3/promoted.tsv.cumulative-diff.json`; the candidate.patch includes every old/new inventory row. The final fingerprint cells are:

| Contract row | S2 fingerprint | Final fingerprint |
| --- | --- | --- |
| B01 | fnv1a64:1c3b721878987f04 | fnv1a64:7f7f4819e5b9a38e |
| B02 | fnv1a64:f2a3f3b047002161 | fnv1a64:712d316bf283105f |
| B03 | fnv1a64:99b57974704ae1de | fnv1a64:bb302670cce29d70 |
| B04 | fnv1a64:4d433f2fd68dc13c | fnv1a64:b69ee54eb1e4e5f6 |
| B05 | fnv1a64:8d4b7183d50d6aad | fnv1a64:161d107652ca21bb |
| B06 | fnv1a64:a7372ea199a86ee8 | fnv1a64:6a26cb7c33930fb2 |
| B07 | fnv1a64:73aa73578e14967d | fnv1a64:3c0e6cf90a4b20ab |
| B08 | fnv1a64:556937292c8a2a7a | fnv1a64:4966a6f1db18c84c |
| B09 | fnv1a64:74d4c27e198e5220 | fnv1a64:80e814575e60f59a |
| B10 | fnv1a64:6dbf97d1152e376b | fnv1a64:2d6196f179eb40c1 |
| B11 | fnv1a64:0e9af08cf081e73c | fnv1a64:77f696abcbd90bf6 |
| B12 | fnv1a64:33e14279f596e720 | fnv1a64:3ff494533a698a9a |
| I01 | fnv1a64:fa1d5d15e5507584 | fnv1a64:5d61331752719a0e |
| I02 | fnv1a64:e48cef17cd43f2ad | fnv1a64:6d5e8e0a4b00a9bb |
| I03 | fnv1a64:3374e948a81374e0 | fnv1a64:0b081722fca292da |
| I04 | fnv1a64:35a0a22dc0253901 | fnv1a64:2da9bc21554e4bff |
| P01 | fnv1a64:612c956bc5430088 | fnv1a64:6a3183f598e9ae52 |
| P02 | fnv1a64:2925bc324eca969b | fnv1a64:1367702abef38d91 |
| P03 | fnv1a64:037b5c8c7cad54c6 | fnv1a64:ddfdce90e9bf5508 |
| P04 | fnv1a64:e77f11c5cba93d61 | fnv1a64:66084f81772c2c5f |
| P05 | fnv1a64:8239767ec9580ef4 | fnv1a64:80b049d3310b895e |
| P06 | fnv1a64:8460674a81c680a0 | fnv1a64:9073b923c699241a |
| P070 | fnv1a64:beae038c603bfaa0 | fnv1a64:cac15565a50e9e1a |
| P080 | fnv1a64:6e46f5021fa8072b | fnv1a64:c38298e8b2ab4801 |
| P090 | fnv1a64:d20b7dae239f2e21 | fnv1a64:15d383e2253fe99f |
| P100 | fnv1a64:f1424228c7c568cc | fnv1a64:f767e600640e9b66 |
| P110 | fnv1a64:d8ccdca01b59bc53 | fnv1a64:cbf5e64b391dca39 |
| P120 | fnv1a64:8203731bbd1bec1a | fnv1a64:835c564a2a9b9b6c |
| P130 | fnv1a64:1777ccb4bcc8707d | fnv1a64:dfdbc65638fefaab |
| P140 | fnv1a64:8a2a2897c38ccf49 | fnv1a64:67058ea4a0d1af77 |
| P150 | fnv1a64:87ffdcb4d8906ac1 | fnv1a64:933a196da917ea3f |
| P071 | fnv1a64:cecebd7646779b1d | fnv1a64:be417460f769a3cb |
| P081 | fnv1a64:535fbdce5ad308c0 | fnv1a64:97853e0543967f3a |
| P091 | fnv1a64:f06e70336e3c1d48 | fnv1a64:beb8b673c37a4a92 |
| P101 | fnv1a64:d0cfcd6d50bd5953 | fnv1a64:c3f8d7186e816739 |
| P111 | fnv1a64:22d77b10be0749f0 | fnv1a64:578c52f4de8ab84a |
| P121 | fnv1a64:52b9d732ea82e2af | fnv1a64:d20326cdbe171825 |
| P131 | fnv1a64:eaed61960ce1e2bc | fnv1a64:544907b4e8390776 |
| P141 | fnv1a64:70ed259f8ea132a8 | fnv1a64:fe514def84fbaff2 |
| P151 | fnv1a64:792b2b152d50cd5a | fnv1a64:923d610dc4bf712c |
| B13 | fnv1a64:3547c22aca452ae8 | fnv1a64:f8375f05642fcbb2 |
| R01 | fnv1a64:5637a52d5fc869b1 | fnv1a64:2e886eabdfb04f4f |
| R02 | fnv1a64:2c93edf91308ea86 | fnv1a64:a0210ab041b99448 |
| R03 | fnv1a64:372549388a547bb5 | fnv1a64:574f389302c46b73 |
| R040 | fnv1a64:0d2cd69aac435851 | fnv1a64:1e8225656b5729ef |
| R050 | fnv1a64:a8cd88b982e085d4 | fnv1a64:fcac1d46013b443e |
| R060 | fnv1a64:f149c14ca56e6499 | fnv1a64:a87bcc02e97b0a27 |
| R070 | fnv1a64:bbcd85e83c74c7c0 | fnv1a64:fff3061f25383e3a |
| R080 | fnv1a64:fca50b208faa0201 | fnv1a64:f4ae251424d314ff |
| R090 | fnv1a64:74b6cb3cc6e1048c | fnv1a64:f64aaa1c89852ca6 |
| R100 | fnv1a64:5edb1dc71a208c5e | fnv1a64:8055cac376b847f0 |
| R110 | fnv1a64:1293a83d6b0e6da3 | fnv1a64:901c2ee93c3c6469 |
| R041 | fnv1a64:1b812554e16ce9e8 | fnv1a64:de70c22f7b578ab2 |
| R051 | fnv1a64:602dabc3b0cd3aa1 | fnv1a64:a3f5b1f7b26df61f |
| R061 | fnv1a64:4069d71c34d898aa | fnv1a64:8608ea13dba4489c |
| R071 | fnv1a64:4b51c33c2dcb6ea7 | fnv1a64:eedf02a4ea805e2d |
| R081 | fnv1a64:ad2cab7d63713dcc | fnv1a64:b3524f54ffba7066 |
| R091 | fnv1a64:00b6216aed52b789 | fnv1a64:0408aad752e44f37 |
| R101 | fnv1a64:6730a137333a9f59 | fnv1a64:ebb208352fdc9267 |
| R111 | fnv1a64:2546ae3bbf0b3d32 | fnv1a64:af1c1334cc2d6ad4 |
| R12 | fnv1a64:73340a5c2f7e7672 | fnv1a64:b6794ef44ed32294 |
| R13 | fnv1a64:36d941926cbb664e | fnv1a64:c58e7cf8d183cbc0 |
| R14 | fnv1a64:0b9a509ea47628cc | fnv1a64:11bff47640bf5b66 |
| R15 | fnv1a64:49427c7d0584d158 | fnv1a64:3cb95f40ba8ca382 |
| O010 | fnv1a64:5b6e9825b424bb0a | fnv1a64:9490b3d0e18fe57c |
| O0200 | fnv1a64:84f1ca6c1243f549 | fnv1a64:61cd3078ef88d577 |
| O011 | fnv1a64:33d27dd4b3002919 | fnv1a64:eb04888af70ccea7 |
| O0210 | fnv1a64:fcbac932352016d2 | fnv1a64:9538694e7122c2f4 |

### 5. Allocation, instruction, resume attribution and timing evidence

**Not run.** The explicit scope STOP occurred before candidate release builds or performance collection. There is no allocation, instruction, resume-attribution, address-level, or timing acceptance claim. Prepared scripts are unexecuted scaffolding, not results.

All four bases were built before production edits, with release codegen-units=1, in separate targets; their feature trees exclude test-continuation-slots. Hashes were rechecked at this STOP:

| S2 base | Bytes | SHA-256 |
| --- | --- | --- |
| rum-cli | 12537440 | b553a00e63e8af387e3141fe35764f54515a3e3154fd09b2756f911a2e87d9bf |
| rum-cli-alloc | 12547616 | f1fb356eb26f45dedaa09ad46e8d697dfde6bd10fbfe1aa12290f61dfcf6e9f1 |
| resume | 12180552 | 57e800bca2cca0d327b358f410d34ced30787410bdee6afe5838f9334a24e81f |
| resume-alloc | 12185720 | 73f279bfb5e13ab4daa34242a9d591e52d9d8012267197268e80f63d41ecc503 |

Still required: all 64 labels’ allocation/byte slopes; exact ordinary equality and per-frame resume attribution; two interleaved instruction pairs per arm with max(0.05%, base range, candidate range); five runs and at least three Callgrind processes per arm for every flagged label; all five resume workloads’ function/address diffs and every added typed instruction; and the full timing panel last on an idle host. Any action/slot/error copy or extra cost must be removed or reported. The expected additional adapter Arc has not been measured.

### 6. S2 cold-helper retype and hot boundary

continuation_fault_outcome consumes the outcome and returns EvalError directly; four production callers and only the two specified test callers remove unwrap_err. The helper remains cold/inline-never. No driver protocol or mode was changed for this cleanup.

The source retype and its six specified caller updates are present, and the typed fault tests pass. The required address-level comparison of the hot boundary has **not run** because no candidate measurement binary was built. No unchanged-hot-path-cost claim is made from source inspection alone.

### 7. Stop-loss, deviations and open decisions

Stop-loss counter: 0. The expected pre-adapter failures became passing falsifiers, stale ownership witnesses became verified concrete witnesses, the missing execute wrapper compile error was corrected, and all isolated S3 tests progressed. The final STOP is the explicit scope boundary for the constructor-oracle visitor, not the two-unproductive-attempt rule. The native-stack repair hypothesis was not tried.

Disclosed workflow deviations and corrections:

- One initial ordinary-control compilation overlapped the tail of the no-default census and its source-root probe. This violated the requested serial execution. The probed file was verified restored to its original hash, and both ordinary controls were rerun serially before relying on them. No performance command ran during that overlap. All other Cargo gates were serial.
- The full oracle script was copied under brief609 with only its repository-root and scratch-output paths adjusted, preserving the two Cargo commands and 1500-second per-build bounds. The outer aggregate bound was 3100 seconds. The repository script remains unchanged.
- The prior report incorrectly described historical suspension_support::drive::execute as a distinct implementation. Nine historical test families alias the exact oracle support module. The completed workspace exposed the mistake; the exact wrapper is restored with only a function-local dead-code allowance, and both oracle targets remain warning-free.
- The first adapter-witness replay parser missed a test-prefixed output line and failed before writing any ledger cell; the corrected parser matched all four emitted rows. The following attempted batch could not start because its input file had not been created. Neither command ran a compiler gate or changed source.
- Certificate replays were repeated after the wrapper restoration because all-source fingerprints deliberately expire on test-source changes. Every intermediate diff remains archived.

The director must resolve the compiler-oracle visitor scope before more repair work. After that, the 170/172 contextual-count pins need their attributed S3 replay, the recursion regression needs a scoped fix and unchanged-contract retest, final gates need completion, and every performance requirement remains outstanding. All commands have finished. The tree is left uncommitted; no git command was run.


## Director amendment — redispatch 4: every production view reads the typed macro; the recursion depth comes back

Both findings are real, and the scope STOP was the right call. Neither is
fixed by updating a pin.

### A. The analyzer's views disagree about the typed macro

**Cause.** Redispatch 3 taught one view, `regions::Edges`, to read the
typed macro's arguments as plain Rust. The retention analyzer has three
other syn views over the same production bodies, and each still treats
every macro as opaque:

| View | Today on a typed macro | Effect |
| --- | --- | --- |
| `compiler_oracle/corpus.rs` `CallAt` | default `visit_macro`, tokens unparsed | the 11 constructor claims inside typed arguments cannot be located; the gate fails loudly (this STOP) |
| `methods/context.rs` `Context` | `fn visit_macro(&mut self, _) {}` | no selection for a method call inside typed arguments, so `regions` rejects it as "method outside checked lexical scope". The reason is false. None exists in the historical frontier today, but S4 onward will create hundreds. |
| `types/context.rs` `Context` | default `visit_macro` | constructor claims inside typed arguments get no exact type. `exact.rs` stops checking them, silently. |

This is one defect, and fixing only `CallAt` would close the instance
that failed while leaving the class open. The rule is: **every view that
enumerates sites in production bodies reads the typed macro the same way
`regions` does, through the same adapter.**

**Ruling A.**

1. **One adapter, declared once.** Declare `typed_resume_macro` in
   `retention_closure/mod.rs` as an ordinary module, next to `regions`.
   Remove the `#[path = "typed_resume_macro.rs"]` declaration from
   `regions.rs`. Export one entry point that returns the checked arguments
   (`adapt`, or a thin wrapper around it). Every view calls it; no view
   parses the macro's tokens itself.
2. **The same reading in all four views.**
   - **Order and scope:** visit `$result` first. Bind the `ok` pattern only
     for `$continue`, then restore the enclosing locals. Visit the session
     expression and `$frame` after that. This is the order `regions`
     uses now.
   - **Legacy macros:** `resume_or_suspend!` and every other macro stay
     opaque in every view.
   - **An adapter error is a hard failure in every view.** A view that sees
     a recognized typed macro it cannot adapt must fail, and must never
     fall back to skipping the macro.
   - **`methods` binds the `ok` pattern conservatively.** It may give the
     binding the `Ok` payload type only if its existing inference already
     establishes the `Result` type of `$result`. Otherwise it binds as
     unknown, as `visit_expr_let` does. An unknown receiver must never
     produce a selection.
   - **`types` follows the same rule.** It gives `$continue` the macro
     expression's expected type only where that type is already
     established. It must never invent one.
3. **Falsifiers, watched to fail first.** Use one fixture whose typed macro
   arguments contain a constructor call, a method call on a typed local, and
   a constructor whose expected type comes from the enclosing signature.
   - Show each of the three views (`CallAt` locates the claim, `methods`
     selects it, `types` claims its exact type) failing on the current
     tree, then passing.
   - Show that a legacy `resume_or_suspend!` carrying the same arguments
     stays opaque in all four views.
   - Keep the fixtures inside the existing retention test modules. Scratch
     copies are for the watched failures only.
4. **Compiler instruments.** If `corpus.py` or `exact.py` cannot observe a
   call whose span lies in macro arguments, you may change them only so
   they observe it as the compiler reports it. Never skip a claim and never
   assume a result. Disclose the change and show the instrument observing
   one of the 11 calls.
5. **`exact.rs` pins follow code, with attribution.** Re-derive the
   contextual count (172) and the count with preserved clones (174) after
   the `types` view change. List every record added or removed, each with
   its cause:
   - an explicit constructor moved into the checked expansion (the two
     M0561 `Err` rows you identified);
   - a constructor newly visible in typed arguments.

   Any other cause is a STOP.
6. **Census.** Rerun the historical frontier. Classify every row change
   against S2 with redispatch 3's four classes, plus one new class,
   *view agreement*: a row that moved from a false
   method-outside-scope rejection to a real selection or to its true
   rejection. Any other unpaired removal is a STOP.
7. **The local analyzer stays out of scope**, as in redispatch 3. Say again
   whether any certified local body contains a converted site.

### B. Guest recursion lost depth: a stack regression on the recursive path

**Cause, as far as the report shows.** In the unchanged test
`holding_and_control_part2.rs:486`, 48 levels of `peel[wrap[…]]` passed
on S2 and now end at `StackLimit`. The test, its depth, the guard and the
stack size are unchanged, so S3 made each level of guest recursion use
more native stack. S3 rewrote the recursive evaluation path itself
(`evaluate_call`, both dispatch-plan resumers, the dispatch stage), added
shared helpers on that path (`resume_call_after_head`,
`finish_call_dispatch_stage`), and placed the macro expansion's locals and
suspend-arm construction in those frames. Stack bytes per level are the
depth contract; they are a first-class cost (see the traversal-depth
line's lessons in `CLAUDE.md`). A per-level number means nothing without
its build profile.

**Ruling B.**

1. **Measure before changing anything.**
   - **Bracket the maximum depth on S2 and S3.** Use both guest recursion
     shapes: the `peel` test's shape and `f[k_] := f[k - 1] + 1`. Measure
     in both the debug test profile and release. Either bisect the depth
     against the unchanged guard with a fresh session per probe, or use
     the `/proc` VmStk high-water slope at four heights; say which.
     Report S2 and S3 bytes per level for each shape and profile.
   - **Attribute the change per function.** Take each frame size on the
     recursive path from its prologue (`objdump -d`, the `sub …,%rsp`
     immediate) in both binaries, for every function the recursion passes
     through once per level. Name each function whose frame grew or that
     is new on the path, with its bytes.
   - **Build S2 for this.** Build the S2 tree (`cb09bfde`) in a separate
     target directory under `.temp/brief-609/`, without touching the
     working tree, for example from a `git archive` export. You may read
     git objects this way; you may not run any git command that changes
     the repository.
2. **Fix by shape, not by limits.**
   - Move cold work off the hot frames: suspend-arm frame construction,
     profiling captures that exist only for a branch, and temporaries that
     live across the recursive call.
   - A shared helper on the recursive path either costs no extra frame
     (show it inlined in both profiles) or is restructured so it does not
     add a level.
   - Move locals, not control flow: no extra branch on the ordinary path.
   - The test, its depth, the native stack guard, the stack size and
     `RUST_MIN_STACK` stay unchanged.
3. **Acceptance.** For both shapes, in both profiles, bytes per level are
   at most S2's. The unchanged 48-level test passes. If an increase remains
   that you cannot remove, attribute it to a named inherent item, then STOP
   and report. Do not accept it yourself.
4. **The instruction gates cover this change too.** Section 5's panel
   measures the final shape, not the pre-fix one.

### C. Then finish the step

After A and B:
- complete the gates the report lists as not rerun: the no-default
  certificate acceptance and census on final source, the full workspace,
  both oracle builds warning-free, and the 31 compile pairs in both builds;
- run every performance requirement from the original brief and from
  redispatch 2, with S2's rules: allocation first, then instruction slopes
  under `max(0.05%, base range, candidate range)`, ≥5 runs and ≥3 callgrind
  processes per arm for every flagged label, address-level resume
  attribution, and timing last on an idle host;
- run the hot-boundary comparison for `continuation_fault_outcome`.

Newly authorized files, for A only:
- `retention_closure/mod.rs`
- `retention_closure/methods/context.rs`
- `retention_closure/types/context.rs`
- `retention_closure/compiler_oracle/corpus.rs`
- `retention_closure/compiler_oracle/exact.rs`
- `corpus.py` and `exact.py`, under A4 only

B needs no new file: the recursive-path files are already in scope.

Append a new `## Execution report` after this section.

## Execution report — redispatch 4: analyzer views repaired; STOP on the native-stack gate

**S3 is not accepted.** Amendment A is implemented: all four production views use one checked adapter; the three watched falsifiers now pass, legacy opacity and fail-closed drift controls pass, and both compiler instruments pass without Python changes. The working runtime remains byte-identical to the redispatch-4 entry.

Amendment B was measured before runtime edits in isolated S2 and S3 trees. The best isolated repair passes the unchanged 48-level test and uses less native stack than S2 in both debug shapes. Release still exceeds S2 by 320 B/level for peel and 384 B/level for arithmetic recursion. Attempt 6 moved neither release result; attempt 7 worsened both. The Hard rule **“Stop after two attempts that move none of the named failing gates”** applies. Stop-loss is 2. No runtime trial was promoted, no remaining regression was accepted, and section C's final gates/performance were not run.

All runtime trials, binaries, raw probes and disassembly remain under `.temp/brief-609/r4/stack/`. The best 13-site trial is `fix1/attempt5-source/`, with the reviewable patch `best-trial.patch`; `fix1/tree/` contains the rejected final trial 7 and must not be mistaken for the best trial. `stop.json` records the distinction. This report appends after the R4 amendment and preserves the entire entry brief byte-for-byte.

### 1. Files changed, ripple and pin disclosures

Seven existing Rust files changed in this redispatch, plus this append-only report. All other S3 changes below are inherited from the prior reports. Scratch runtime trials are disclosed separately in section 5 and are not working-tree changes.

| File | This redispatch / inherited | Semantic change or pin |
| --- | --- | --- |
| crates/rum-runtime/src/runtime/builtin_adapters.rs | Inherited R1–R3 | M0205 legacy macro → witnessed typed macro; identity success tail retained. |
| crates/rum-runtime/src/runtime/builtin_adapters/resume_frames.rs | Inherited R1–R3 | F0204 legacy frame → TypedFrame&lt;Expr, Expr&gt;. |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | Inherited R1–R3 | T0023 wrapper removed; initial witnessed capture; deferred helpers return ResumeStop&lt;PreparedCallArguments&gt;, advance fresh cursor/splice state, and use the witnessed token. Pure completion now returns PreparedCallArguments; its existing Legacy wire provider consumes those fields directly instead of freeze/thaw round-tripping them. The test-only old-wire fixture follows that helper’s typed result. |
| crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs | Inherited R1–R3 | F0273 merges into F0272 so every repeat freezes fresh state of the same nominal type. F0570 becomes SlotFinalizer&lt;Output=PreparedCallArguments&gt;; finish_work schedules it after originating owners drop. |
| crates/rum-runtime/src/runtime/eval_core/call.rs | Inherited R1–R3 | T0024 witnesses Expr before head sharing; shared resume_call_after_head performs the conversion. T0025/T0026 add PreparedDispatchFrame and share argument normalization/restore installation. F0275/F0276 consume Option&lt;Expr&gt;; their ordinary stage-result macro is shared in finish_call_dispatch_stage. Remove both shape-dependent old decoder branches. Stable heads retain their existing SharedExpr value. Profiling stop remains after ordinary preparation normalization and before returning a suspension. |
| crates/rum-runtime/src/runtime/eval_core/mod.rs | Inherited R1–R3 | The SharedExpr-result profiling wrapper loses its final caller under T0024 and is removed; generic outcome classification remains. |
| crates/rum-runtime/src/runtime/eval_core/symbol.rs | Inherited R1–R3 | Only F0277/M0285 typed: CompiledMatchOutcome → Expr, retaining matched/unmatched semantics and deleting the Expr wire map. The S14 loader capture in this file is unchanged. |
| crates/rum-runtime/src/runtime/eval_dispatch/mod.rs | Inherited R1–R3 | Remove the generic PrepareArguments work and leave an unreachable arm. M0286 now witnesses the actual threaded Result instead of receiving only a request in a cold helper. M0287/M0288 use typed macros, preserving profiling on suspension. F0278 completes resumed.map(Some) after the existing memory-growth validation. |
| crates/rum-runtime/src/runtime/hold_thread.rs | Inherited R1–R3 | F0279/M0289 typed Expr identity continuation for ReleaseHold. |
| crates/rum-runtime/src/runtime/resolution.rs | Inherited R1–R3 | M0561/M0562 witnessed at the actual next_result before matching; saved current, pass count and history retain their transitions. |
| crates/rum-runtime/src/runtime/resolution/resume_frames.rs | Inherited R1–R3 | F0526/F0527 typed Expr → Expr, retaining outcome? and resume_fixed_point_from. |
| crates/rum-runtime/src/runtime/session/session_eval.rs | Inherited R1–R3 | Two callers: Err(self.continuation_fault_outcome(...).unwrap_err()) → Err(self.continuation_fault_outcome(...)). |
| crates/rum-runtime/src/runtime/session/session_requests.rs | Inherited R1–R3 | Two callers: remove unwrap_err from the cold latch projection, retaining the same hot discriminant check. |
| crates/rum-runtime/src/runtime/suspension/core/fault.rs | Inherited R1–R3 | Cold continuation_fault_outcome: Result&lt;T, EvalError&gt; → EvalError; still consumes the outcome and takes the same latch. |
| crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs | Inherited R1–R3 | Redispatch 2: delete suspend_witnessed; original suspend(session, request, fresh) → suspend(session, SlotWitness&lt;F::Input&gt;, request, fresh). Same public identity, same fresh F constraint, no Arc in the token. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs | Inherited R1–R3 | The legacy-queue setup calls eval_input_string directly instead of evaluating an InputString call through newly typed S3 frames. This restores the legacy-to-mixed widening precondition; every allocation, byte and admission assertion is unchanged. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs | Inherited R1–R3 | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs | Inherited R1–R3 | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs | Inherited R1–R3 | repeat_work now captures slot_witness(&outcome) from its actual input producer before matching and passes it to the sole suspend route. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | Inherited R1–R3 | Scan both legacy and typed frame/finalizer traits; preserve 564-frame population and root/drop checks. Pin the Session-free PreparedCallArguments finalizer signature. Now compiled and tested: 15/15 frame-root tests pass. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs | Inherited R1–R3 | In both derived-root and pure-abandon arrays: eval_call_prepare/resume_frames.rs:ResumeSuspendedCallArgumentPreparationSuspensionFrame → eval_core/call.rs:PreparedDispatchFrame. This pairs the D0001 merge with the T0025/T0026 split consumer. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv | Inherited R1–R3 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. Four emitted adapter witnesses replace interim no-production-instantiation text with concrete S3 preparation-frame/finalizer ownership paths; disposition, owed_by and external coverage unchanged. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv | Inherited R1–R3 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv | Inherited R1–R3 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv | Inherited R1–R3 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv | Inherited R1–R3 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv | Inherited R1–R3 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/retention_closure/census/tests.rs | Inherited R1–R3 | Historical frontier 932 → 936; RET001 10 unchanged, RET002 895 → 900, RET003 27 → 26. 918 identical rows, 13 moved pairs, one old opaque site replaced by four accounted rows, and one new Try rejection. Explicitly preserve the freeze rejection; historical ACCEPTANCE_BASELINE string unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs | R4 | CallAt now carries Source and Function, searches each alternative body with its real file, and traverses checked typed arguments. Add one actual generic typed fixture plus six permanent view controls in this existing module; no new frame implementation. |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs | R4 | Context pins172→170 and preserved-clone total174→172. Exactly the two M0561 Err constructors moved into checked expansion; no claim filtered or new type invented. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | Inherited R1–R3 | Inherited R3 fingerprint/certificate replay retained. It is stale after the R4 checker-source changes; no R4 replay or acceptance is claimed. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | Inherited R1–R3 | Inherited R3 fingerprint/certificate replay retained. It is stale after the R4 checker-source changes; no R4 replay or acceptance is claimed. |
| crates/rum-stdlib/src/tests/retention_closure/methods/context.rs | R4 | Read typed arguments in producer/success/session/frame order through checked. Bind the success pattern from an already inferred Result payload, otherwise unknown; restore locals. The conditional-attribute pre-scan uses the same adapter. |
| crates/rum-stdlib/src/tests/retention_closure/mod.rs | R4 | Declare typed_resume_macro once as an ordinary sibling module. No new visibility or path escape. |
| crates/rum-stdlib/src/tests/retention_closure/regions.rs | R4 | Replace its path-based private adapter module and direct adapt calls with the shared checked entry. Preserve four expansion rejections and argument traversal order; no admission or pin change. |
| crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs | R4 | Make recognizes/adapt private; expose one checked(Source, Function, Macro) entry. Unrecognized macros return None; recognized adaptation errors panic with the located reason. Macro-definition and hygiene checks remain exact. |
| crates/rum-stdlib/src/tests/retention_closure/types/context.rs | R4 | Read expression and statement macro arguments through checked, preserving success-only scope. Propagate only an already established expected type; preserve reference binding shape. Legacy and other macros remain opaque. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs | Inherited R1–R3 | New strict If-held and listable-completion cases for three marker shapes, plus prepared.twice/head.map.twice controls. Checks::ALL and unconditional FaultProbe checks retained. Fold/Nest definitions archived for S13. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | Inherited R1–R3 | No module → ordinary sibling mod evaluation_dispatch; no ../ path attribute. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | Inherited R1–R3 | S2 → S3 receipt, used by marker fixtures and census. FLIP_SLOT becomes census-only. Both oracle targets compile without warnings. |
| crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv | Inherited R1–R3 | Record all owned F/M/T/D transitions; retain F0273/M0280 removal receipts, add F0275P/M0282S; remove four raw decoder consumers; add S3 and X0040 finalizer-wire fixture receipt. Every changed row is in the receipt-transition artifact. |
| crates/rum-stdlib/tests/suspension_oracle/support/drive.rs | Inherited R1–R3 | Retain the exact execute wrapper required by nine historical integration families through path aliases. Add a function-local allow(dead_code) for the oracle target, which calls execute_planned directly. This corrects the earlier erroneous removal; execute_planned and all comparisons remain unchanged. |
| crates/rum-stdlib/tests/typed_slot_census.rs | Inherited R1–R3 | Scratch brief-608 → brief-609; S2 → S3 receipt and negative-preflip labels. Scope FLIP_SLOT here; connect s3 module. Guard-removal legacy counts 564/59 → 553/58; removed-frame sentinel remains 622. Both scratch falsifiers verified in both builds. |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | Inherited R1–R3 | Production legacy frames/finalizers 553/54 → 542/53, total 607 unchanged; macro result/insert 345/244 → 353/236, total 589 unchanged. Receipt row totals F623 → F624 and M600 → M601 retain merged rows plus new split identities. Closed-set S3 removal proof precedes skipping removed source anchors; all other exact source/mode checks remain. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | Inherited R1–R3 | Scratch brief-608 → brief-609. Removed Whole0029–Whole0032 emit explicit S3 transition records, assert absence of the old identity, and preserve decoder IDs; live decoder rows still require rediscovery. |
| crates/rum-stdlib/tests/typed_slot_census/routes.rs | Inherited R1–R3 | I0003 still names ContinueToken::suspend and its executable repeat witness. Update mechanism output to disclose typed S3 evaluation while later families remain untyped; S2 mechanism ownership is unchanged. |
| crates/rum-stdlib/tests/typed_slot_census/s3.rs | Inherited R1–R3 | New closed-set gate for exactly 12 typed implementations / 13 typed macros, input/output pairs, 8 approved removals, absent T0023 wrapper/callers, relocated maps and witnessed token repeat. |
| crates/rum-stdlib/tests/typed_slot_compile_controls.rs | Inherited R1–R3 | Scratch directory brief609; actual producer witnesses added to repeat/finalizer controls. Add token-witness-input, changing only producer Expr → Option&lt;Expr&gt;, requiring one E0308 at the complete witness span. 31 positive/intended-negative pairs pass in each build. |

No runtime module-root widening, dependency, lockfile, toolchain, registry, core representation or unrelated documentation edit was made. `corpus.py` and `exact.py` are byte-identical to entry. No git command or network operation was used. There is no new unsafe code in the working patch or the scratch probes.

The final scope audit found a new unrelated `docs/dev/security-hardening-progress.md`, absent from the entry manifest and timestamped2026-10-14T10:46:51Z. None of this execution’s tools wrote it. It was preserved separately at SHA-256 `1c7be1ded5fa952282940578e32582531018c1e6bf21c4ab86b97e10062193a0`; it is not included in the agent patch or claimed as an authorized edit. The initial all-tree audit flagged it; the corrected audit records that exception explicitly rather than modifying it.

| R4 file | Entry chars | Final chars |
| --- | --- | --- |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs | 3801 | 12367 |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs | 9336 | 9649 |
| crates/rum-stdlib/src/tests/retention_closure/methods/context.rs | 13557 | 15049 |
| crates/rum-stdlib/src/tests/retention_closure/mod.rs | 17313 | 17337 |
| crates/rum-stdlib/src/tests/retention_closure/regions.rs | 12983 | 12657 |
| crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs | 6540 | 7023 |
| crates/rum-stdlib/src/tests/retention_closure/types/context.rs | 10416 | 12093 |

The adapter remains below 10k characters. The other enlarged existing modules remain in the 10k–20k warning band; the brief expressly requires these fixtures in existing retention modules. No arbitrary file split or new production module was added. Full cumulative patch and scope/protected-file evidence: `r4/candidate.patch`, `r4/final-audit.json`.

### 2. Owned rows, modes and census state

No F/M/T/D conversion changed in the working runtime during R4. The existing owned transitions remain:

| F row | Current transition |
| --- | --- |
| F0204 | Typed Expr → Expr; same name. |
| F0272 | Typed Expr → PreparedCallArguments; also receives fresh repeated state formerly handled by F0273. |
| F0273 | Merged into F0272; no remaining production implementation of the old name. |
| F0274 | Typed Expr → Expr; head conversion at the shared entry. |
| F0275 | Typed Option&lt;Expr&gt; → Expr; prepared branch split to PreparedDispatchFrame. |
| F0276 | Typed Option&lt;Expr&gt; → Expr; prepared branch uses the same PreparedDispatchFrame. |
| F0277 | Typed CompiledMatchOutcome → Expr. |
| F0278 | Typed Expr → Option&lt;Expr&gt;; complete the semantic Some map. |
| F0279 | Typed Expr → Expr. |
| F0526 | Typed Expr → Expr. |
| F0527 | Typed Expr → Expr. |
| F0570 | SlotFinalizer Output=PreparedCallArguments. |

New F0275P remains PreparedDispatchFrame, Input=PreparedCallArguments and Output=Expr. F0273 remains merged into F0272; F0570 remains the pure typed finalizer. Modes remain Expr Value and the existing 21 Legacy modes, with no State births, per-frame overrides or dual-arm decoder.

| M row | Actual producer type | Current transition |
| --- | --- | --- |
| M0205 | Expr | Typed identity frame; no post-witness semantic map. |
| M0279 | Expr | Typed F0272 reentry uses the same append helper and fresh cursor/splice fields; pure finalizer builds the exact output after Arc release. |
| M0280 | Expr | Old macro removed by D0001; witness of actual evaluated_result is passed to token.suspend with fresh F0272. No original Arc is captured. |
| M0281 | Expr | Witness now at evaluate(head.into_owned()); shared resume_call_after_head accepts Expr or existing SharedExpr and performs Into&lt;SharedExpr&gt; once. |
| M0282 | PreparedCallArguments or Option&lt;Expr&gt; (split) | Prepare macro → PreparedDispatchFrame; stage macro → shared finish_call_dispatch_stage, F0275. Shared normalization/restore helper runs once. |
| M0283 | PreparedCallArguments or Option&lt;Expr&gt; (split) | Prepare macro → PreparedDispatchFrame; shared stage macro → F0276; restore freezes after the same helper. |
| M0285 | CompiledMatchOutcome | Same branches in F0277; only the Expr transport map and raw input decode disappear. |
| M0286 | Expr | F0278 now completes resumed.map(Some); three listable fixtures watched fail → pass. |
| M0287 | Expr | Same Some completion in unchecked F0278; suspension records the existing profiling event before the typed macro returns. |
| M0288 | Expr | Checked F0278 validates the Expr result and then wraps Some. The If-held fixtures exercise the checked test build; the unchecked constructor shares this completion body. Its release path has not yet been measured. |
| M0289 | Expr | Typed identity frame. |
| M0561 | Expr | Typed F0526 uses outcome? then the unchanged resume_fixed_point_from state transition. |
| M0562 | Expr | Typed F0527 uses outcome? then the same helper. |

M0282S remains the shared Option-stage macro in finish_call_dispatch_stage in the working tree. T0023 removes the unreachable PrepareArguments arm and unused wrapper. T0024 witnesses Expr before the shared head conversion. T0025/T0026 split the prepared/stage consumers and install normalization/restore state once. T0027 removes transport-only encoding. D0001 uses actual producer evidence and fresh F0272 state through the sole token.suspend route, with finalization after both originating owners drop.

Last executed source-census output is inherited R3 evidence, not a fresh R4 gate: production 607 implementations=542 legacy frames+53 legacy finalizers+11 TypedFrame+1 SlotFinalizer;589 macro sites=576 legacy+13 typed, with 353 result/236 insert. Test-only 16 implementations/11 macros; combined 623/600. The new compiled A fixture contributes one test-only typed invocation and no frame implementation. Its test-only receipt/count reconciliation is outstanding: expected test-only macro 11→12 and combined macro 600→601 must be confirmed by discovery, not merely pinned. Production runtime totals are unchanged by the verified byte-identical runtime tree. R4 did not rerun the census or update its receipts after the B stop.

The isolated best trial preserves13 production macro sites by splitting the Option-stage helper into two direct matches and sharing M0287/M0288 through finish_builtin_dispatch. Those trial-only macro identity changes, the extra test fixture, and the altered macro expansion would all need exact receipt/frontier reconciliation before any promotion. No such trial receipt or adapter change was smuggled into the working source.

### 3. Watched falsifiers and divergence evidence

A3 used one compiled generic TypedFrame fixture with an explicit Some constructor, Local::choose, and Ok(Some(value)) whose expected type comes from the signature. An initial scratch E0608 (indexing a BTreeSet) was corrected before the watched failing run; it is not counted as the required behavioral failure. The permanent tests remain in compiler_oracle/corpus.rs. Scratch copies were used to show pre/post behavior without changing the checked production macro.

Raw output: `.temp/brief-609/r4-view-fail-first-compiled.log`.

```text
running 3 tests
test checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments' panicked at checker/compiler_oracle/corpus.rs:151:9:
constructor locator hid crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:123:25-123:37
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments' panicked at checker/compiler_oracle/corpus.rs:171:5:
assertion `left == right` failed: explicit Some and both contextual constructors
  left: 0
 right: 3
FAILED
test checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments' panicked at checker/compiler_oracle/corpus.rs:160:5:
assertion `left == right` failed: typed local method must be visited
  left: 0
 right: 1
FAILED

failures:

failures:
    checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments
    checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments
    checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 2 filtered out; finished in 44.17s

error: test failed, to rerun pass `--test frontier`
```

The legacy control changes only the macro path in the fixture AST, preserving identical argument tokens and spans. It passes before the repair and remains opaque in all four views. The final six-control run also proves all-view hard failure on definition drift and that an unknown success binding neither selects a method nor leaks outside its scope. The four printed drift panics below are deliberately caught hard-failure assertions:

Raw output: `.temp/brief-609/r4-views-controls.log`.

```text
running 6 tests
test checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views ... LEGACY_MACRO_OPAQUE regions/locator/methods/types
ok
test checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments ... TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:151:25-151:37 std::option::Option::Some
TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:17-153:32 std::result::Result::Ok
TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:20-153:31 std::option::Option::Some
ok
test checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view ... 
thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
TYPED_MACRO_DRIFT regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
ok
test checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments ... TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:151:25-151:37 std::option::Option::Some std::option::Option<usize>
TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:17-153:32 std::result::Result::Ok std::result::Result<std::option::Option<rum_core::expr::Expr>, rum_runtime::runtime::session::session_state::EvalError>
TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:20-153:31 std::option::Option::Some std::option::Option<rum_core::expr::Expr>
ok
test checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments ... TYPED_MACRO_METHOD crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:152:25-152:39 rum_stdlib::tests::retention_closure::compiler_oracle::corpus::typed_argument_fixture::Local::choose
ok
test checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 87.68s
```

The earlier six strict language-divergence failures are retained below as historical S2 evidence, not rerun baselines. R1 replaced the director-reassigned Fold/Nest cases with three If-held and three listable-completion cases; F0528/M0563 remain untouched for S13.

Raw output: `.temp/brief-609/r1-pre-builtin-held-default.log`.

```text
running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

Raw output: `.temp/brief-609/r1-pre-builtin-held-no-default.log`.

```text
running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 661, identity: None, text: SymbolText { id: 661, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

Raw output: `.temp/brief-609/r1-pre-threaded-dv-default.log`.

```text
running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 673, identity: None, text: SymbolText { id: 673, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

Raw output: `.temp/brief-609/r1-pre-threaded-dv-no-default.log`.

```text
running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

Their latest complete post-conversion equality is the R3 full suspension-oracle evidence below: Checks::ALL and unconditional FaultProbe absence were unchanged. R4 did not rerun either full suspension-oracle build, so these are not claimed as final-source acceptance:

Historical R3 default

```text
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=17175
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=17187
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=17199
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=17211
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=17223
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=17235
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=17247
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=17259
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88016
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.47s
```

Historical R3 no-default

```text
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=17175
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=17187
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=17199
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=17211
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=17223
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=17235
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=17247
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=17259
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88016
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.27s
```

### 4. Analyzer reconciliation, compiler evidence and gate ledger

A4 required no Python instrument change. The production constructor oracle independently observes all 7315 claims, including all 11 previously unlocatable typed-argument constructors. The exact oracle reports285 agreements and 0 disagreements. Raw inputs, MIR observation tables and fingerprints were copied into `r4/constructor-oracle/` and `r4/exact-oracle/`; all 11 paired observations are in `r4/eleven-constructor-observations.json`.

| Claim | Location | Compiler observation |
| --- | --- | --- |
| corpus_611 | crates/rum-runtime/src/runtime/builtin_adapters.rs:937:26 | Result::&lt;rum_core::Expr, session_state::EvalError&gt;::Ok |
| corpus_1335 | crates/rum-runtime/src/runtime/eval_core/call.rs:138:33 | Result::&lt;eval_call_prepare::PreparedCallArguments, session_state::EvalError&gt;::Ok |
| corpus_1337 | crates/rum-runtime/src/runtime/eval_core/call.rs:262:33 | Result::&lt;eval_call_prepare::PreparedCallArguments, session_state::EvalError&gt;::Ok |
| corpus_1338 | crates/rum-runtime/src/runtime/eval_core/call.rs:351:32 | Result::&lt;rum_core::Expr, session_state::EvalError&gt;::Ok |
| corpus_1402 | crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:128:42 | Result::&lt;Option&lt;rum_core::Expr&gt;, session_state::EvalError&gt;::Ok |
| corpus_1403 | crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:128:45 | Option::&lt;rum_core::Expr&gt;::Some |
| corpus_1411 | crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:191:30 | Result::&lt;rum_core::Expr, session_state::EvalError&gt;::Ok |
| corpus_1412 | crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:206:26 | Result::&lt;rum_core::Expr, session_state::EvalError&gt;::Ok |
| corpus_1451 | crates/rum-runtime/src/runtime/hold_thread.rs:29:26 | Result::&lt;rum_core::Expr, session_state::EvalError&gt;::Ok |
| corpus_3118 | crates/rum-runtime/src/runtime/resolution.rs:380:30 | Result::&lt;rum_core::Expr, session_state::EvalError&gt;::Ok |
| corpus_3124 | crates/rum-runtime/src/runtime/resolution.rs:448:30 | Result::&lt;rum_core::Expr, session_state::EvalError&gt;::Ok |

Verbatim totals: r4-constructor-oracle

```text
A1_CORPUS_TOTAL agreements=7315 disagreements=0 claims=7315
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 257.06s
```

Verbatim totals: r4-exact-oracle

```text
A1_ORACLE_TOTAL agreements=285 disagreements=0
A1_ORACLE_TOTAL agreements=285 disagreements=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 197.89s
```

A5 re-derivation:172→170 contextual claims;174→172 with the two preserved-clone contexts. All170 retained records are identical to R3; no historical contextual claim was newly added by the types view. Exactly these two S2 rows were removed, both because M0561 moved their explicit Err construction into the checked expansion:

```text
crates/rum-runtime/src/runtime/resolution.rs:386:28-386:69	std::result::Result::Err	std::result::Result<rum_core::expr::Expr, rum_runtime::runtime::session::session_state::EvalError>
crates/rum-runtime/src/runtime/resolution.rs:388:38-388:48	std::result::Result::Err	std::result::Result<rum_core::expr::Expr, rum_runtime::runtime::session::session_state::EvalError>
```

A6 historical frontier:936 rows, byte-identical to R3, with 0 view-agreement row changes. Against S2:932→936;918 identical rows,13 same-operation moved pairs, the old opaque M0561 RET003 row replaced by four explicit expansion RET002 rows, and one new S3 Try/FromResidual rejection. RET001 10 unchanged, RET002 895→900, RET003 27→26. No unpaired removal; the AccumulatorChain::default rejection in FixedPointHistory::freeze remains. The exact S2 pairing ledger remains `r3/frontier-classified-diff.json`; the R4 equality/context reconciliation is `r4/view-agreement-reconciliation.json`.

A7: the local analyzer is unchanged. Neither C01 nor C02’s certified body contains a converted site; runtime/options/core.rs is unchanged. However the 68 source-context fingerprints and four promoted certificate cells retained from R3 are stale after the R4 analyzer edit. No certificate replay or local acceptance, default or no-default, was performed in R4. Ownership discovery was likewise not replayed;79 approved host identities/59 cycles are the last R3 observation, not a new R4 claim.

All R4 gate commands were serial, bounded, offline/jobs1/test-threads1 with TMPDIR under brief609. No R4 Cargo overlap occurred. The following table gives every saved top-level R4 gate’s exact test-result lines; each command/environment and full raw output is beside the log as .command.json/.exit.json. Empty result cells mean build/format/measurement-driver commands, not passed tests.

| Raw log | Exit | Verbatim test-result totals |
| --- | --- | --- |
| .temp/brief-609/r4-constructor-oracle.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 257.06s |
| .temp/brief-609/r4-exact-oracle.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 197.89s |
| .temp/brief-609/r4-final-format.log | 0 | No test-result line |
| .temp/brief-609/r4-fix1-format.log | 0 | No test-result line |
| .temp/brief-609/r4-fix3-format-corrected.log | 0 | No test-result line |
| .temp/brief-609/r4-fix3-format.log | 1 | No test-result line |
| .temp/brief-609/r4-fix5-format.log | 0 | No test-result line |
| .temp/brief-609/r4-format.log | 0 | No test-result line |
| .temp/brief-609/r4-marker-base-debug.log | 0 | No test-result line |
| .temp/brief-609/r4-marker-base-release.log | 0 | No test-result line |
| .temp/brief-609/r4-marker-before-debug.log | 0 | No test-result line |
| .temp/brief-609/r4-marker-before-release.log | 0 | No test-result line |
| .temp/brief-609/r4-neighborhood-base-debug.log | 0 | No test-result line |
| .temp/brief-609/r4-neighborhood-base-release.log | 0 | No test-result line |
| .temp/brief-609/r4-neighborhood-before-debug.log | 0 | No test-result line |
| .temp/brief-609/r4-neighborhood-before-release.log | 0 | No test-result line |
| .temp/brief-609/r4-original-48-base.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.04s |
| .temp/brief-609/r4-original-48-before.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.05s |
| .temp/brief-609/r4-original-48-fix1.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.01s |
| .temp/brief-609/r4-production-views-inventory.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 10.35s |
| .temp/brief-609/r4-prologues-base-release.log | 0 | No test-result line |
| .temp/brief-609/r4-prologues-base-trace.log | 0 | No test-result line |
| .temp/brief-609/r4-prologues-before-release.log | 0 | No test-result line |
| .temp/brief-609/r4-prologues-before-trace.log | 0 | No test-result line |
| .temp/brief-609/r4-stack-base-debug-v2.log | 0 | No test-result line |
| .temp/brief-609/r4-stack-base-debug.log | 0 | No test-result line |
| .temp/brief-609/r4-stack-base-release.log | 0 | No test-result line |
| .temp/brief-609/r4-stack-before-debug-v2.log | 0 | No test-result line |
| .temp/brief-609/r4-stack-before-debug.log | 0 | No test-result line |
| .temp/brief-609/r4-stack-before-release.log | 0 | No test-result line |
| .temp/brief-609/r4-trace-base-release.log | 0 | No test-result line |
| .temp/brief-609/r4-trace-before-release.log | 0 | No test-result line |
| .temp/brief-609/r4-view-fail-first-compiled.log | 101 | test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 2 filtered out; finished in 44.17s |
| .temp/brief-609/r4-view-fail-first.log | 101 | No test-result line |
| .temp/brief-609/r4-view-legacy-before.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 14.86s |
| .temp/brief-609/r4-views-controls.log | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 87.68s |
| .temp/brief-609/r4-views-post.log | 0 | test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 58.42s |

Both production compiler-oracle tests and the view-fixture gates emitted no compiler warning. The isolated native builds carry three pre-existing unused-helper warnings, recorded separately below from their compiler-message JSON. All native-probe result lines are saved verbatim with their paths in `r4/stack/probe-test-results.tsv`:839 subprocess results total,431 with 1172 filtered out and 408 with 1171 filtered out. Every such probe reports1 passed/0 failed/0 ignored/0 measured. That means the probe ran successfully, **not** that its requested depth completed: StackLimit is an expected returned observation, recorded separately in depth-probes.json. The unchanged 48-level test is a real equality assertion and its three outputs follow.

Raw output: `.temp/brief-609/r4-original-48-base.log`.

```text
running 1 test
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.04s
```

Raw output: `.temp/brief-609/r4-original-48-before.log`.

```text
running 1 test
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... 
thread 'runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds' panicked at crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:504:5:
assertion `left == right` failed
  left: "TerminatedEvaluation[\"StackLimit\"]"
 right: "done"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.05s
```

Raw output: `.temp/brief-609/r4-original-48-fix1.log`.

```text
running 1 test
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.01s
```

The fix1-named48 log uses the best debug-fix5b binary, not the first trial. Build/probe metadata binds its exact SHA. Native ZeroMQ tests did not run in R4, so no new native passes/skips are claimed. The9 R3 loopback-capability skips remain historical skips.

**Not run on final R4 source:** full workspace, typed runtime suite, frame-root suite, all 31 compiler pairs in both builds, typed census in both builds, both complete suspension-oracle builds, ownership discovery/acceptance, default/no-default certificate acceptance, and the Cargo production_rejection_census_is_complete gate. The required historical frontier itself was recomputed by the copied production views inventory and compared row-for-row. R3 results are not substituted for these final gates. No new FaultProbe run occurred; R3’s full-oracle final 88016 observations per build and all occurrence deltas0 remain historical only.

### 5. Native-stack experiment and unrun performance requirements

S2 was rebuilt in a separate target tree from the 2023 SHA-256-verified original S2 entry files, using saved original bytes for changed files. This satisfies the separate S2 source build without using the amendment’s optional git archive route, which the user expressly forbids. Every reconstructed file was reverified; only the scratch probe was appended to the original recursion-test file. See `r4/stack/s2-snapshot-verification.json`. This is the original S2 snapshot associated with cb09bfde by the supplied brief; no git-object identity was independently read or claimed.

Method: fresh-process/session bisection against the unchanged 1048576-byte native guard, with the existing libtest thread stack and RUST_MIN_STACK absent. Both debug test profile and release test profile were built; release uses codegen-units1. The copied probe raises only the separate logical recursion ceiling to 100000 so it cannot mask native guard failure. It constructs the peel expression iteratively to avoid measuring parser nesting limits. The actual 48-level text test and its depth are untouched and were tested separately. No process-abort probe was used.

B/level is the complete resident-prologue sum: sub-rsp allocation plus pushed registers,8 B return address per native frame, and outgoing stack arguments. A supplemental safe-Rust leaf marker checks native_base minus a local address at four heights×three fresh repetitions: debug 4/8/12/16 and release 10/20/30/40, all fits zero residual. The marker adds only fixed bottom-frame overhead; it does not instrument the recursive functions. Depth bisection is the requested primary method, not VmStk and not wall-clock timing. Base/entry neighborhoods were checked at ten adjacent depths×three fresh processes per shape/profile, with no mixed or nonmonotone boundary results.

| Profile / shape | S2 B/level | Entry S3 B/level | Best isolated B/level | S2 max / first limit | Entry max / first limit | Best max / first limit |
| --- | --- | --- | --- | --- | --- | --- |
| Debug peel | 21424 | 24384 | 21184 | 51 /52 | 43 /44 | 51 /52 |
| Debug arithmetic | 37728 | 43280 | 36768 | 28 /29 | 24 /25 | 28 /29 |
| Release peel | 8096 | 8752 | 8416 | 131 /132 | 119 /120 | 127 /128 |
| Release arithmetic | 14720 | 15904 | 15104 | 70 /71 | 66 /67 | 68 /69 |

All per-function prologues, addresses, call multiplicities and raw disassemblies are retained. The tables below include every resident runtime frame in the two repeating cycles. Values exclude the 8-byte return address, which is added to each occurrence in the sums. The release cycles also have32 bytes of outgoing call arguments (16 at each of the two resident dispatch calls); this is present in all three arms. Inlined/absent entries contribute no independent frame. Native backtraces identify the cycles; inline debug annotations such as try_apply_down_values_fallback are distinguished from native frames at the same PC. The complete sums exactly match the independent marker slopes.

#### debug prologues

| Function | Per-level count peel / plus | S2 | Entry S3 | Best isolated trial |
| --- | --- | ---: | ---: | ---: |
| evaluate | 1 / 2 | 312 | 312 | 312 |
| evaluate_profiled_fixed_point | 1 / 2 | 2072 | 2568 | 1816 |
| evaluate_once_unstable_after_input_check_with_cycle_tracking | 1 / 2 | 184 | 184 | 184 |
| evaluate_once_in_accounting_mode | 1 / 2 | 2264 | 2264 | 2264 |
| evaluate_inner_unstable | 1 / 2 | 1000 | 1000 | 1000 |
| evaluate_call | 1 / 2 | 1288 | 856 | 856 |
| resume_call_after_head | 1 / 2 | 552 | 568 | 568 |
| resume_call_with_dispatch_plan | 2 / 3 | 1768 | 2840 | 2184 |
| run_call_dispatch_stage | 1 / 1 | 1880 | 1656 | 1480 |
| try_apply_down_values | 1 / 1 | 2088 | 2088 | 2088 |
| try_apply_down_values_from_symbol | 1 / 2 | 3800 | 3800 | 3800 |
| evaluate_definition_rhs | 1 / 1 | 1336 | 1336 | 1336 |
| evaluate_with_option_value_scope | 1 / 1 | 1000 | 1000 | 1000 |
| prepare_arguments_for_call_with_unevaluated_restore | 0 / 1 | 344 | 344 | 344 |
| prepare_arguments_for_call_with_unevaluated_restore::{{closure}} | 0 / 1 | 2200 | 2680 | 1736 |
| evaluate_atom_fast_path | 0 / 1 | 424 | 424 | 424 |
| evaluate_call::{{closure}} | 1 / 2 | inlined / absent from path | 952 | inlined / absent from path |

#### release prologues

| Function | Per-level count peel / plus | S2 | Entry S3 | Best isolated trial |
| --- | --- | ---: | ---: | ---: |
| evaluate_definition_rhs | 1 / 1 | 840 | 840 | 840 |
| try_apply_down_values_from_symbol | 1 / 2 | 1272 | 1272 | 1272 |
| try_apply_down_values | 1 / 1 | 776 | 776 | 776 |
| run_call_dispatch_stage | 1 / 1 | 296 | 280 | 280 |
| resume_call_with_dispatch_plan | 2 / 3 | 584 | 920 | 808 |
| resume_call_after_head | 1 / 2 | 632 | inlined / absent from path | inlined / absent from path |
| evaluate_inner_unstable | 1 / 2 | 1080 | 1656 | 1608 |
| evaluate_once_unstable_after_input_check_with_cycle_tracking | 1 / 2 | 760 | 760 | 760 |
| evaluate | 1 / 2 | 840 | 904 | 840 |
| evaluate_with_option_value_scope | 1 / 1 | 312 | 312 | 312 |
| evaluate_atom_fast_path | 0 / 1 | 408 | 408 | 408 |
| prepare_arguments_for_call_with_unevaluated_restore | 0 / 1 | 984 | 1176 | 936 |



The entry regression is fully attributed: debug fixed-point+496, evaluate_call changes1288→856 but adds a952-byte closure plus return address, after-head+16, dispatch resumer+1072 per occurrence, stage−224, argument-preparation closure+480. Their multiplicities yield+2960/+5552 B per guest level. Release evaluate+64, inner evaluator1080→1656 with the 632-byte after-head frame inlined away, dispatch resumer+336 per occurrence, stage−16, preparation+192 yield+656/+1184 B.

The best trial removes the stable-head closure and the shared owned-payload dispatch helper, avoids redundant Ok/? and producer Result temporaries, moves suspension construction cold, and shares builtin completion through an always-inlined helper to retain13 production typed macro sites. Its release dispatch resumer still has an808-byte prologue versus S2’s584: +224, paid twice for peel and three times for arithmetic. At measured-binary addresses this is S2 0x569640 → trial 0x567e40. Release inner evaluation including head handling saves112 B per occurrence, stage saves16 B, preparation saves48 B, and the fixed-point evaluator returns to 840 B. Therefore peel 2×224−112−16=320; arithmetic 3×224−2×112−16−48=384. The remaining object/temporary cost is in the resumer’s distinct typed prepared/stage paths and owned suspension-state transfers. No proof that this is unavoidable for the typed design is claimed; it remains an unremoved implementation cost, not an accepted inherent overhead.

| Attempt / isolated variant | Debug peel / arithmetic | Release peel / arithmetic | Disposition |
| --- | --- | --- | --- |
| Entry S3 | 24384 /43280 | 8752 /15904 | Measured before repair |
| 1 cold frame builders, branch-only head closure, inline shared stage helper | 24928 /43360 | 8720 /15760 | Debug worse; release moved |
| 2b cold suspension closure helper | 24432 /42464 | 8528 /15424 | Improved, still red |
| Direct-match diagnostic,14 macro sites | 23312 /40784 | Not built | Diagnostic only; count not acceptable |
| 3 success exits instead of Ok/? temporaries | 23216 /39984 | Not built | Debug moved |
| 4 identifier macro arm avoids duplicate producer Result | 22272 /38400 | Not built | Debug moved |
| 5b direct stages plus inlined shared builtin completion,13 sites | 21184 /36768 | 8416 /15104 | Best; debug gate and48 test green; release red |
| 6 early drop of obsolete restore option | Not rebuilt | 8416 /15104 | No progress; counter1 |
| 7 debug outline/release inline policy and inlined frame builders | Not rebuilt | 8448 /15136 | Worse; counter2 → STOP |

Trial6 was reverted only inside scratch before trial 7. Trial7’s release stage prologue grows280→312, explaining its additional32 B/level; no debug improvement was claimed or measured. The best trial was never copied into the working tree. Its altered macro definition would require an exact checked-adapter update, all-view reruns and expansion-operation accounting before promotion. Trial runtime semantics beyond the stack probe/48 assertion were not validated; no language-oracle acceptance is claimed for them.

The trial build failures were reduced explicitly:14 E0658 expression-attribute diagnostics became a stable local cold generic function; one scratch formatting edit produced delimiter errors and was corrected before compiling; two E0382 diagnostics for the non-Copy profiling token were fixed by giving the shared builtin helper sole ownership. None was hidden by a changed toolchain, weakened test or ignored diagnostic. Build stdout JSONL, stderr, commands, exits and binary hashes remain in each stack arm/profile directory.

| Native diagnostic artifact | SHA-256 | Profile |
| --- | --- | --- |
| .temp/brief-609/r4/stack/base/debug | 57d57cfb7670c6f8bfb1d66d1b50297fdf5d1f558c58e9ba1c3b47c1b293b7d5 | debug |
| .temp/brief-609/r4/stack/base/debug-v2 | 1d7f02b0e2a736963a8dc6134f8ea7c8a832d810a5dfa80e8f4a257ee449f37b | debug |
| .temp/brief-609/r4/stack/base/release | 45b1a015ae6f225a63b8fabaa0788354bff85a18dab9f99def983d629ad1e417 | release |
| .temp/brief-609/r4/stack/base/trace-release | 085fc3e0c14f239e0698ce2ef339cf841508a0b5f24acf7dc573aae8cf7586d1 | release |
| .temp/brief-609/r4/stack/before/debug | ca7ece2ea224649f005057eafb01a2cf688f866091b4910a3947654b64dd450d | debug |
| .temp/brief-609/r4/stack/before/debug-v2 | 9d0b5b7ae6e11120745004d28f460807dd918fe8f84093f9be1cfa429a5660e2 | debug |
| .temp/brief-609/r4/stack/before/release | de4ba95f5e3a27b3e913790e1821741a3b3a15b252aae6885391f521a5e65279 | release |
| .temp/brief-609/r4/stack/before/trace-release | dda7c8e732a86d7109a10eb4e693122c9926613abbf9fa3449ac55f6da5da640 | release |
| .temp/brief-609/r4/stack/fix1/debug | d919321f052b46171968c4fabf0a494f01b0940be2cacc3611d1027558ff0d91 | debug |
| .temp/brief-609/r4/stack/fix1/debug-direct | 1e273e5bcdcfd0db1fbfb9b195bdf94ce33caba90c03fa86d40fa7be643cb99a | debug |
| .temp/brief-609/r4/stack/fix1/debug-fix2b | 6e5c250579b0b39461ddddb36d3bcdf590b61a904e73560809ff020c9a51dbf0 | debug |
| .temp/brief-609/r4/stack/fix1/debug-fix3 | be43d01b374ea2a392ee77bdb321029992f681ea4bfc87bbdf1d9c381675c880 | debug |
| .temp/brief-609/r4/stack/fix1/debug-fix4 | 50b361d9d5f94e73779205c4149190a0d5d4cc8e9d4101da32941ed7fca38325 | debug |
| .temp/brief-609/r4/stack/fix1/debug-fix5b | 7ab4efd2b7d6a35fa0164ba0744fda2cc6288b4d9bc7107b885d867026acc694 | debug |
| .temp/brief-609/r4/stack/fix1/release | 1328272142edccec2d1204646cf5335b2964fe6b102c69e0efaec031e632e114 | release |
| .temp/brief-609/r4/stack/fix1/release-fix2b | 7c438270d51d4e5b8594250588bb7e916d6bcf363dc69806afeecf5be122e64c | release |
| .temp/brief-609/r4/stack/fix1/release-fix5b | 991b796a8350ff48d471b8dfdb3be9b63c1c77eddf3e469349ea6e3375f651a4 | release |
| .temp/brief-609/r4/stack/fix1/release-fix6 | a4aa794d1b124baccba3049a99b34f0c610496af82e7ae69aaf4bf030af88236 | release |
| .temp/brief-609/r4/stack/fix1/release-fix7 | 019aa12d10ae01174f523d2e8dba456f7f24bca5ac58bf9f1b10409981a4da8a | release |

The following four immutable S2 performance bases were not rebuilt or overwritten; the final audit rehashed each successfully:

| Performance base | SHA-256 |
| --- | --- |
| rum-cli | b553a00e63e8af387e3141fe35764f54515a3e3154fd09b2756f911a2e87d9bf |
| rum-cli-alloc | f1fb356eb26f45dedaa09ad46e8d697dfde6bd10fbfe1aa12290f61dfcf6e9f1 |
| resume | 57e800bca2cca0d327b358f410d34ced30787410bdee6afe5838f9334a24e81f |
| resume-alloc | 73f279bfb5e13ab4daa34242a9d591e52d9d8012267197268e80f63d41ecc503 |

**Not run:** candidate performance builds;64-label allocation/byte panel; two interleaved instruction-slope pairs per arm; flagged-label five-run/three-callgrind-process expansion; all-five-workload per-function/address-level typed resume attribution; timing panel and idle-host preflight. No allocation equivalence, typed +1 allocation/+24-byte formula, instruction threshold, resumed-frame overhead or timing result is asserted. Native diagnostic test binaries are not the required production performance binaries. These requirements remain mandatory after B is resolved.

Native build ledger:21 attempts,19 successful builds and 2 compile failures, with 14 E0658 and 2 E0382 diagnostics. The separate view-fixture E0608 and scratch formatting delimiter failure are disclosed above. Each of the 19 successful native builds reports the same three pre-existing warnings,57 warning occurrences total; all three already occur in the rebuilt S2 base and refer to unchanged runtime/test_builtins.rs. Neither the working analyzer patch nor the scratch runtime patches introduce them. Full diagnostics and commands: `r4/stack/build-ledger.json`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
```

### 6. S2 cold-helper leftover and hot-boundary check

The inherited working change still retypes continuation_fault_outcome from Result<T, EvalError> to EvalError, keeps it cold, wraps the same four production calls in Err, and removes unwrap_err from the two owned tests. R4 did not change those files. The required address-level comparison of its hot boundary has **not run**; source inspection and earlier focused tests are not substituted for that evidence.

### 7. Stop-loss, deviations and exact remaining work

Stop-loss 2 applies to the final two no-progress release repairs; preceding attempts moved measured stack gates, and compile errors were reduced. Section B3 remains unmet in release, and the original 48 test still fails on the unmodified working runtime. Only the isolated best debug trial passes it. No safety margin, guard, stack size, RUST_MIN_STACK, original test depth or production recursion limit was changed. No gate was weakened or unpaired retention removal accepted.

R4’s seven analyzer edits are complete and reviewable; their compiler instruments pass. Final source certificates, ownership/census replay (including the new test-only macro receipt), the full workspace/dual-build gates and every production performance check remain outstanding. No git, network or commit action was taken. The four immutable bases and 183 protected design/harness inputs remain intact. A later authorized continuation should start from the preserved best trial, resolve the named release-frame growth within the brief’s constraints, then validate the exact final macro adapter/census identities and run section C in its prescribed order. This report does not authorize silently applying the scratch macro or its unchecked receipt changes.

Final report/audit artifacts: `r4/final-audit.json`, `r4/final-changed.json`, `r4/candidate.patch`, `r4/gate-totals.json`, `r4/stack/attribution.tsv`, `r4/stack/probe-test-results.tsv`, `r4/stack/best-trial.patch`, and `PROGRESS.md`. The scope audit’s initial unrelated-document detection, report-query mistakes, and final warning-audit correction made no runtime-source changes or measurement decisions. The first warning-audit assertion incorrectly expected no native JSON warnings; it was corrected to preserve and disclose the three inherited S2 warning kinds. The report was corrected only inside this execution’s appended block, preserving the entire director entry prefix.

## Director amendment — redispatch 5: one resumer frame per call, not one per stage

Amendment A is accepted as reported. On B, the measurement is exactly what
was needed: the stop-loss was the right call, and the attribution points
to the fix. The `docs/dev/security-hardening-progress.md` file your audit
found is the director's own; you were right to leave it out of your patch.

### Why seven attempts could not close release

On release, every remaining byte comes from one function. With the best
trial, `resume_call_with_dispatch_plan`'s prologue is 808 B against S2's
584 B (+224). The guest level pays that frame **two times for `peel` and
three times for arithmetic**, because the resumer moves to the next stage
by calling itself (`return self.resume_call_with_dispatch_plan(…,
stage_index + 1, …)`), so every stage it passes through stays resident.

Attempts 6 and 7 tried to shrink the frame. The multiplicity is the other
lever, and it is the structural one. The stage advance is a loop written as
recursion: each stage moves `plan` and `state` into a new frame and keeps
the old one alive under it for the rest of the guest level.

### Ruling

1. **Start from the best trial.** Use `.temp/brief-609/r4/stack/fix1/attempt5-source/`
   (`best-trial.patch`), not `fix1/tree/`.
2. **Advance stages in a loop, in both dispatch resumers.**
   - Rewrite `resume_call_with_dispatch_plan` and
     `resume_suspended_call_with_dispatch_plan` so that one invocation walks
     its stages in a loop. It returns on a `Some(result)` stage, the final
     assembly, an error or a suspension.
   - Moving to stage `stage_index + 1` becomes the loop's next iteration,
     not a call.
   - Leave every suspension frame's captured `stage_index` and every resume
     entry point's contract unchanged. A frame resumed at stage *k* enters
     the loop at stage *k + 1*, exactly as before.
   - The ordinary path gains no branch: the back-edge replaces the call.
     This is an explicit exception to "move locals, not control flow". It
     removes work (two by-value moves of `plan` and `state` and one
     prologue per stage) rather than adding a check.
   - Expected effect: one resident resumer frame per guest-level call
     instead of two or three. On the reported numbers, release `peel`
     drops by about one 808 B frame, to well under S2's 8 096 B. Measure
     it; do not assume it.
3. **The macro change in the best trial is accepted on one condition.**
   - The cold `#[inline(never)]` suspend helper inside the `InputRequired`
     arm keeps frame construction out of the caller's frame. Every later
     typing step gets that for free, so it stays.
   - The identifier arm is a second macro form that every analyzer view
     must then read. Keep it only if the debug gate fails without it,
     after the loop change. Measure both variants and report both.
   - Either way, the macro definition has changed:
     - update the checked adapter's expected definition to the final form
       (every arm checked exactly) and re-watch the definition-drift
       falsifier against it;
     - re-derive the fixed expansion-operation rejection list from the
       final expansion, so that the cold helper's capture and move of the
       frame inputs is an accounted operation;
     - have all four views read every arm the same way, with the A3
       fixtures covering each arm;
     - classify the census rows that move against S2 with the existing
       classes.
4. **Acceptance is unchanged from B3.**
   - For both shapes in both profiles, bytes per level are at most S2's:
     debug `peel` 21 424, debug arithmetic 37 728, release `peel` 8 096,
     release arithmetic 14 720.
   - The unchanged 48-level test passes on the working tree.
   - Report the final prologue table in the same form as before.
   - The stop-loss rule is unchanged.
5. **Then finish the step (section C of redispatch 4, in its order).**
   Also:
   - reconcile the A fixture's test-only macro (expected 11 → 12, and
     600 → 601 combined) by discovery, not by a pin;
   - replay the certificates and fingerprints that the R4 checker-source
     changes expired;
   - run the oracle in both builds over the final runtime, since the
     loop changes the ordinary dispatch path;
   - in the instruction panel, take the dispatch-heavy labels'
     instruction slopes at face value. The loop should not raise them;
     if it does, attribute the increase per address.

**Newly authorized:** `crates/rum-runtime/src/runtime/suspension/typed/witness.rs`.
It is the macro's definition, and the edit is limited to the macro's arms
and its cold suspend helper. The S1 public API stays as it is. Every other
file is already in scope: the dispatch resumers, the adapter and the four
views.

Append a new `## Execution report` after this section.


## Execution report — redispatch 5: looped dispatch and final measurement

**S3 is not accepted: measured performance defects remain.** The directed loop repair passes all four native-stack limits, the unchanged 48-level test, all final correctness gates, both compiler instruments and both full warning-free suspension-oracle builds. The identifier macro arm was removed after the required two-variant measurement; all four analyzer views check the final cold-helper expression arm.

All 59 ordinary allocation/byte slopes are exact. All 35 ordinary instruction flags have added-work evidence, and all five resume workloads exceed the extra-Arc-only allocation expectation and add statistically separated instruction work. Extra action/slot/result copies are identified at concrete addresses and left reported under the brief's “fix or report” rule. No performance repair was attempted after measuring this final candidate. The seven sections below preserve the successful gates, the failed cost requirements, and the limits of the attribution separately.

### 1. Files changed, ripple and pin disclosures

20 existing source/test/inventory files changed in this redispatch, plus this appended report. The cumulative S3 patch contains the files below. All redispatch-5 entry bytes of this brief are preserved, including the director’s redispatch-5 amendment. The separate director-owned `docs/dev/security-hardening-progress.md` was preserved and excluded from the patch.

| File | Redispatch | Semantic change / pin transition |
| --- | --- | --- |
| crates/rum-runtime/src/runtime/builtin_adapters.rs | Inherited R1–R4 | M0205 legacy macro → witnessed typed macro; identity success tail retained. |
| crates/rum-runtime/src/runtime/builtin_adapters/resume_frames.rs | Inherited R1–R4 | F0204 legacy frame → TypedFrame<Expr, Expr>. |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | R5 | Typed prepared-argument result; actual producer witness precedes semantic work. F0272 fresh cursor/splice state uses its sole witnessed token route on repeats; no captured Arc. Successful macro arms escape directly and construction uses cold freeze; ordinary result/effect order preserved. |
| crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs | R5 | Merge F0273 into F0272 nominal typed frame; freeze fresh cursor/prefix/restore state cold. Typed FinishPreparedCallArgumentsFinalizer executes after both originating owners drop. R5 cold factory moves impl citation13→40 and finalizer43→70. |
| crates/rum-runtime/src/runtime/eval_core/call.rs | R5 | Typed Expr head and split PreparedCallArguments/Option<Expr> consumers. Loop through dispatch stages in both resumers; preserve captured stage k and resumed k+1. Shared prepared normalization/restore runs once. Stable heads avoid a closure; cold frame factories and success escapes avoid resident temporaries. Shared finish_call_dispatch_stage removed; M0282S/M0283S are direct stage sites. |
| crates/rum-runtime/src/runtime/eval_core/mod.rs | Inherited R1–R4 | The SharedExpr-result profiling wrapper loses its final caller under T0024 and is removed; generic outcome classification remains. |
| crates/rum-runtime/src/runtime/eval_core/symbol.rs | Inherited R1–R4 | Only F0277/M0285 typed: CompiledMatchOutcome → Expr, retaining matched/unmatched semantics and deleting the Expr wire map. The S14 loader capture in this file is unchanged. |
| crates/rum-runtime/src/runtime/eval_dispatch/mod.rs | R5 | Typed Option/Expr completion and F0278 semantic Some on resumed builtin result with original growth checks. Always-inlined finish_builtin_dispatch shares checked/unchecked M0287/M0288 completion and owns the non-Copy timing token once; M0288 removed→M0287 receipt. |
| crates/rum-runtime/src/runtime/hold_thread.rs | Inherited R1–R4 | F0279/M0289 typed Expr identity continuation for ReleaseHold. |
| crates/rum-runtime/src/runtime/resolution.rs | R5 | Typed fixed-point Expr continuations preserve history/cycle/iteration semantics. Success escapes avoid Ok/? temporaries; cold frame freeze. Freeze/default rejection remains at35:19–35:46. |
| crates/rum-runtime/src/runtime/resolution/resume_frames.rs | R5 | Both fixed-point frames implement TypedFrame<Input=Expr,Output=Expr>; freeze current/pass/history cold and resume exact fixed-point logic. |
| crates/rum-runtime/src/runtime/session/session_eval.rs | Inherited R1–R4 | Two callers: Err(self.continuation_fault_outcome(...).unwrap_err()) → Err(self.continuation_fault_outcome(...)). |
| crates/rum-runtime/src/runtime/session/session_requests.rs | Inherited R1–R4 | Two callers: remove unwrap_err from the cold latch projection, retaining the same hot discriminant check. |
| crates/rum-runtime/src/runtime/suspension/core/fault.rs | Inherited R1–R4 | Cold continuation_fault_outcome: Result<T, EvalError> → EvalError; still consumes the outcome and takes the same latch. |
| crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs | Inherited R1–R4 | Redispatch 2: delete suspend_witnessed; original suspend(session, request, fresh) → suspend(session, SlotWitness<F::Input>, request, fresh). Same public identity, same fresh F constraint, no Arc in the token. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs | Inherited R1–R4 | The legacy-queue setup calls eval_input_string directly instead of evaluating an InputString call through newly typed S3 frames. This restores the legacy-to-mixed widening precondition; every allocation, byte and admission assertion is unchanged. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs | Inherited R1–R4 | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs | Inherited R1–R4 | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs | Inherited R1–R4 | repeat_work now captures slot_witness(&outcome) from its actual input producer before matching and passes it to the sole suspend route. |
| crates/rum-runtime/src/runtime/suspension/typed/witness.rs | R5 | Macro only: retain actual producer witness, one expression arm, and cold inline-never suspension helper. Identifier arm removed after both measured variants pass debug; public S1 API prefix unchanged. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | Inherited R1–R4 | Scan both legacy and typed frame/finalizer traits; preserve 564-frame population and root/drop checks. Pin the Session-free PreparedCallArguments finalizer signature. Now compiled and tested: 15/15 frame-root tests pass. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs | Inherited R1–R4 | In both derived-root and pure-abandon arrays: eval_call_prepare/resume_frames.rs:ResumeSuspendedCallArgumentPreparationSuspensionFrame → eval_core/call.rs:PreparedDispatchFrame. This pairs the D0001 merge with the T0025/T0026 split consumer. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv | R5 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. Four emitted adapter witnesses replace interim no-production-instantiation text with concrete S3 preparation-frame/finalizer ownership paths; disposition, owed_by and external coverage unchanged. R5 replays exactly four verified impl-line witness cells (F0272 13→40, F0570 43→70); approved identities/dispositions unchanged. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv | Inherited R1–R4 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv | Inherited R1–R4 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv | R5 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv | R5 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv | R5 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/retention_closure/census/tests.rs | R5 | Live frontier932→936 (RET00110 unchanged, RET002895→900, RET00327→26). Against S2:916 identical,15 paired moved operations, old opaque M0561 replaced by five checked expansion rejections; no unpaired removal. Historical print unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs | R5 | Locate constructors inside typed arguments using the same checked adapter and actual source/function for alternatives. Six compiled view controls cover constructor lookup, methods, expected types, legacy opacity, unknown binding/scope, and all-four-view drift. R5 checks the cold-capture rejection and three expansion mutations across four views. |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs | R5 | Context pins172→170 and preserved-clone total174→172: exactly two M0561 Err constructors moved into the checked expansion. R5 preserves all170 contexts and updates the moved-Ok explanatory comment; no instrument/filter change. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | R5 | Replay all68 context fingerprints, preserving all18 other cells; final context fnv1a64:bc81091c2485a5f2. Exact old/new cells in report artifacts. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | R5 | Replay only four C01/C02 hash-build certificate cells after zero-rejection emission; all seven other columns unchanged. Both final local acceptance gates pass. |
| crates/rum-stdlib/src/tests/retention_closure/methods/context.rs | Inherited R1–R4 | Read typed invocation arguments through checked; bind a success pattern only from an already inferred Result payload, otherwise unknown, and restore locals. Conditional-attribute pre-scan uses the same adapter. Legacy macros remain opaque. |
| crates/rum-stdlib/src/tests/retention_closure/mod.rs | Inherited R1–R4 | Declare one ordinary sibling typed_resume_macro module; no visibility widening. |
| crates/rum-stdlib/src/tests/retention_closure/regions.rs | Inherited R1–R4 | Use the shared checked adapter for typed invocation arguments, preserving conservative rejections and traversal. Final expansion explicitly rejects five unresolved operations, including cold closure capture/move/FnOnce/drop. |
| crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs | R5 | One checked entry validates the complete single-expression-arm macro definition exactly. Parse and visit arguments in all four views; recognized invalid adaptations fail hard. Expansion-operation rejection list4→5 accounts cold closure captures/moves, FnOnce call and drop without inventing ownership admission. |
| crates/rum-stdlib/src/tests/retention_closure/types/context.rs | Inherited R1–R4 | Read expression/statement typed invocations through checked; propagate only established expected types and preserve success-only scope/reference binding shape. Legacy macros remain opaque. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs | Inherited R1–R4 | New strict If-held and listable-completion cases for three marker shapes, plus prepared.twice/head.map.twice controls. Checks::ALL and unconditional FaultProbe checks retained. Fold/Nest definitions archived for S13. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | Inherited R1–R4 | No module → ordinary sibling mod evaluation_dispatch; no ../ path attribute. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | Inherited R1–R4 | S2 → S3 receipt, used by marker fixtures and census. FLIP_SLOT becomes census-only. Both oracle targets compile without warnings. |
| crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv | R5 | Record all owned S3 transitions. R5 shared-stage M0282S splits into direct M0282S/M0283S; builtin M0288 merges→M0287. T0025/T0026 direct install_prepared_call_arguments anchors replace map anchors. Add X0041 for legacy-opacity AST path quotation; X40→41. |
| crates/rum-stdlib/tests/suspension_oracle/support/drive.rs | Inherited R1–R4 | Retain the exact execute wrapper required by nine historical integration families through path aliases. Add a function-local allow(dead_code) for the oracle target, which calls execute_planned directly. This corrects the earlier erroneous removal; execute_planned and all comparisons remain unchanged. |
| crates/rum-stdlib/tests/typed_slot_census.rs | Inherited R1–R4 | Scratch brief-608 → brief-609; S2 → S3 receipt and negative-preflip labels. Scope FLIP_SLOT here; connect s3 module. Guard-removal legacy counts 564/59 → 553/58; removed-frame sentinel remains 622. Both scratch falsifiers verified in both builds. |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | R5 | Replay S3 receipts and retained removals: F624; M602 total =600 active+2 removed (M0280 and M0288). Production/test-only active pins unchanged; discovered A fixture is TestSupport. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | Inherited R1–R4 | Scratch brief-608 → brief-609. Removed Whole0029–Whole0032 emit explicit S3 transition records, assert absence of the old identity, and preserve decoder IDs; live decoder rows still require rediscovery. |
| crates/rum-stdlib/tests/typed_slot_census/routes.rs | Inherited R1–R4 | I0003 still names ContinueToken::suspend and its executable repeat witness. Update mechanism output to disclose typed S3 evaluation while later families remain untyped; S2 mechanism ownership is unchanged. |
| crates/rum-stdlib/tests/typed_slot_census/s3.rs | R5 | S3 owned F/M/T/D reconciliation and parsed T0023 no-definition/no-caller checks. R5 expected typed macro set replaces M0288 with M0283S; closed removal M0288→M0287. |
| crates/rum-stdlib/tests/typed_slot_compile_controls.rs | Inherited R1–R4 | Scratch directory brief609; actual producer witnesses added to repeat/finalizer controls. Add token-witness-input, changing only producer Expr → Option<Expr>, requiring one E0308 at the complete witness span. 31 positive/intended-negative pairs pass in each build. |

No production module visibility widening was needed. The S1 public API prefix of witness.rs is byte-identical to entry; only its macro changed. No mode, registry, shape, certified C01/C02 body, local analyzer, compiler instrument Python, dependency, lockfile, toolchain or core representation changed. The scope audit verifies all183 protected design/harness inputs and all four immutable S2 binaries. Exact cumulative patch: `r5/candidate.patch`; source hashes and sizes: `r5/final-audit.json`. No git or network operation was used and no unsafe code was added.

| R5 touched Rust file | Entry characters | Final characters |
| --- | --- | --- |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | 23303 | 22832 |
| crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs | 1722 | 2565 |
| crates/rum-runtime/src/runtime/eval_core/call.rs | 21504 | 21389 |
| crates/rum-runtime/src/runtime/eval_dispatch/mod.rs | 17300 | 17796 |
| crates/rum-runtime/src/runtime/resolution.rs | 17562 | 17524 |
| crates/rum-runtime/src/runtime/resolution/resume_frames.rs | 1865 | 2250 |
| crates/rum-runtime/src/runtime/suspension/typed/witness.rs | 1531 | 1726 |
| crates/rum-stdlib/src/tests/retention_closure/census/tests.rs | 9595 | 9685 |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs | 12367 | 13061 |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs | 9649 | 9641 |
| crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs | 7023 | 7480 |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | 12844 | 12904 |
| crates/rum-stdlib/tests/typed_slot_census/s3.rs | 5206 | 5235 |

Size assessment: the shared checked adapter remains under10k characters. The existing call and argument-preparation modules remain in the20k–35k split-plan band: their coherent boundaries are dispatch traversal/resumed consumers and preparation/Legacy wire transport. Splitting these during this owned migration would add unrelated module movement; no arbitrary split was made. The other enlarged existing dispatch, resolution and compiler-fixture modules remain under20k. The three new cumulative S3 Rust modules remain under10k. Every touched size is recorded in final-audit.json.

### 2. Owned-row reconciliation, semantic maps and census

| F row | S2 name | Final S3 transition |
| --- | --- | --- |
| F0204 | BuiltinEvaluateSuspensionFrame | Typed Expr → Expr; same name. |
| F0272 | PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame | Typed Expr → PreparedCallArguments; also receives fresh repeated state formerly handled by F0273. |
| F0273 | ResumeSuspendedCallArgumentPreparationSuspensionFrame | Merged into F0272; no remaining production implementation of the old name. |
| F0274 | EvaluateCallSuspensionFrame | Typed Expr → Expr; head conversion at the shared entry. |
| F0275 | ResumeCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch split to PreparedDispatchFrame. |
| F0276 | ResumeSuspendedCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch uses the same PreparedDispatchFrame. |
| F0277 | EvaluateStoredOwnValueSuspensionFrame | Typed CompiledMatchOutcome → Expr. |
| F0278 | RunCallDispatchStageSuspensionFrame | Typed Expr → Option<Expr>; complete the semantic Some map. |
| F0279 | EvalReleaseHoldSuspensionFrame | Typed Expr → Expr. |
| F0526 | EvaluateProfiledFixedPointSuspensionFrame | Typed Expr → Expr. |
| F0527 | ResumeFixedPointFromSuspensionFrame | Typed Expr → Expr. |
| F0570 | FinishPreparedCallArgumentsFinalizer | SlotFinalizer Output=PreparedCallArguments. |

| M row | Actual producer | Ordinary transformation | Resumed transformation |
| --- | --- | --- | --- |
| M0205 | Expr | builtin_evaluate returns the evaluated value unchanged | Typed identity frame; no post-witness semantic map. |
| M0279 | Expr | append/splice/drop-Nothing; preserve Unevaluated entries; advance remaining arguments; build PreparedCallArguments | Typed F0272 reentry uses the same append helper and fresh cursor/splice fields; pure finalizer builds the exact output after Arc release. |
| M0280 | Expr | the same append and cursor/restore work for later arguments | Old macro removed by D0001; witness of actual evaluated_result is passed to token.suspend with fresh F0272. No original Arc is captured. |
| M0281 | Expr | SharedExpr conversion, plan construction and dispatch; profiling records | Witness now at evaluate(head.into_owned()); shared resume_call_after_head accepts Expr or existing SharedExpr and performs Into<SharedExpr> once. |
| M0282 | PreparedCallArguments or Option<Expr> (split) | normalize prepared args, install restore; stage Some returns value, None advances cursor | Prepare macro → PreparedDispatchFrame; direct stage macro M0282S → F0275. Shared normalization/restore runs once; None advances the loop; resumed frame enters k+1. |
| M0283 | PreparedCallArguments or Option<Expr> (split) | same work with the suspended plan and frozen restore | Prepare macro → PreparedDispatchFrame; direct stage macro M0283S → F0276. Restore freezes after the common installer; None advances the loop; resumed frame enters k+1. |
| M0285 | CompiledMatchOutcome | matched branch evaluates own value; unmatched branch resumes symbol lookup | Same branches in F0277; only the Expr transport map and raw input decode disappear. |
| M0286 | Expr | Some(value) after evaluating the threaded call | F0278 now completes resumed.map(Some); three listable fixtures watched fail → pass. |
| M0287 | Expr | Shared finish_builtin_dispatch: profile, original checked/unchecked growth behavior, semantic Some. | F0278 runs the same growth check and semantic Some; M0288 merged into M0287. |
| M0288 | Expr | Shared finish_builtin_dispatch: profile, original checked/unchecked growth behavior, semantic Some. | F0278 runs the same growth check and semantic Some; M0288 merged into M0287. |
| M0289 | Expr | ReleaseHold evaluation result unchanged | Typed identity frame. |
| M0561 | Expr | fixed-point equality/cycle history/trace update and next pass | Typed F0526 uses outcome? then the unchanged resume_fixed_point_from state transition. |
| M0562 | Expr | same fixed-point transition in resumed evaluation | Typed F0527 uses outcome? then the same helper. |

F0273 merges into F0272; fresh repeats retain the same nominal frame and use its witnessed token. New F0275P PreparedDispatchFrame consumes PreparedCallArguments; the two stage frames consume Option<Expr>. M0280 is removed into the token route. R5 removes the interim shared finish_call_dispatch_stage: M0282S is the ordinary loop’s stage macro and new M0283S is the suspended loop’s stage macro. M0287/M0288 merge into always-inlined finish_builtin_dispatch, with M0288 recorded as removed→M0287. There remain exactly13 production typed macro sites and12 typed implementations (11 TypedFrame plus1 SlotFinalizer).

T0023 removes the unreachable generic PrepareArguments arm and unused wrapper; the parsed no-definition/no-caller gate remains. T0024 witnesses the actual Expr result before common SharedExpr construction. T0025/T0026 install normalization and restore state once, now anchored at direct install_prepared_call_arguments method14/method16 instead of map method16 in both resumers. T0027 removes pure transport conversion while preserving semantic Some and head conversion on both paths. D0001 retains fresh cursor/splice state and finalization after both originating Arcs drop. Modes stay global: Expr Value and the existing21 Legacy modes, with no State births or dual decode.

```text
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=9 result=4 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=9 result=4 insert=5
S3 PARTITION implementations=622 macros=600
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=9 result=4 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S3 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=9 result=4 insert=5
S3 PARTITION implementations=623 macros=600
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=9 result=4 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
```

Discovery corrects the amendment’s estimate: the A fixture is TestSupport under the unchanged named-tests-module/path rule, not TestOnly. Production remains607 implementations/589 macros, test-only16/11 and combined623/600. Test-support macros8→9. Its discovered identity is Mbac6c9f55630a655, compiler_oracle::corpus::typed_argument_fixture::invocation#macro1:resume_typed_or_suspend:result; it appears in macro-sites.tsv and is outside the production/test-only M receipt cohort, exactly as existing test-support invocations are. No scanner or count was forced to12/601. Active M receipts600; total M receipts601→602 retain M0280 and M0288 removals. X0041 records the AST-only legacy-opacity path quotation; legacy support40→41. This adds no legacy suspension site. See macro-reconciliation.json, transform-receipt-diff.json and legacy-fixture-receipt.json.

| Receipt | S2 status → final | Final identity and anchors |
| --- | --- | --- |
| F0204 | untyped → typed | rum_runtime::runtime::builtin_adapters::resume_frames::BuiltinEvaluateSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0272 | untyped → typed | rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0273 | untyped → removed | rum_runtime::runtime::eval_call_prepare::resume_frames::ResumeSuspendedCallArgumentPreparationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame; F0272 |
| F0274 | untyped → typed | rum_runtime::runtime::eval_core::call::EvaluateCallSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0275 | untyped → typed | rum_runtime::runtime::eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0276 | untyped → typed | rum_runtime::runtime::eval_core::call::ResumeSuspendedCallWithDispatchPlanSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0277 | untyped → typed | rum_runtime::runtime::eval_core::symbol::EvaluateStoredOwnValueSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0278 | untyped → typed | rum_runtime::runtime::eval_dispatch::RunCallDispatchStageSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0279 | untyped → typed | rum_runtime::runtime::hold_thread::EvalReleaseHoldSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0526 | untyped → typed | rum_runtime::runtime::resolution::resume_frames::EvaluateProfiledFixedPointSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0527 | untyped → typed | rum_runtime::runtime::resolution::resume_frames::ResumeFixedPointFromSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0570 | untyped → typed | rum_runtime::runtime::eval_call_prepare::resume_frames::FinishPreparedCallArgumentsFinalizer implements rum_runtime::runtime::suspension::typed::SlotFinalizer; - |
| M0205 | untyped → typed | rum_runtime::runtime::builtin_adapters::builtin_evaluate#macro1:resume_typed_or_suspend:result; - |
| M0279 | untyped → typed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::prepare_arguments_for_call_with_unevaluated_restore#macro1:resume_typed_or_suspend:result; - |
| M0280 | untyped → removed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::resume_suspended_call_argument_preparation#macro1:resume_or_suspend:insert; F0272 |
| M0281 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::evaluate_call#macro1:resume_typed_or_suspend:result; - |
| M0282 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#macro1:resume_typed_or_suspend:result; - |
| M0283 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#macro1:resume_typed_or_suspend:result; - |
| M0285 | untyped → typed | rum_runtime::runtime::eval_core::symbol::<Session as inherent>::evaluate_stored_own_value#macro1:resume_typed_or_suspend:result; - |
| M0286 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::run_call_dispatch_stage#macro1:resume_typed_or_suspend:result; - |
| M0287 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::finish_builtin_dispatch#macro1:resume_typed_or_suspend:result; M0288 |
| M0288 | untyped → removed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::try_dispatch_builtin_call#macro2:resume_typed_or_suspend:result; M0287 |
| M0289 | untyped → typed | rum_runtime::runtime::hold_thread::<Session as inherent>::eval_release_hold#macro1:resume_typed_or_suspend:result; - |
| M0561 | untyped → typed | rum_runtime::runtime::resolution::<Session as inherent>::evaluate_profiled_fixed_point#macro1:resume_typed_or_suspend:result; - |
| M0562 | untyped → typed | rum_runtime::runtime::resolution::<Session as inherent>::resume_fixed_point_from#macro1:resume_typed_or_suspend:result; - |
| D0001 | untyped → implemented | deferred::prepared arguments; F0204;F0272;F0273;F0274;F0275;F0276;F0277;F0278;F0279;F0526;F0527;F0570;F0275P |
| T0023 | untyped → removed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::prepare_arguments_for_call#method1:map:self . prepare_arguments_for_call_with_unevaluated_restore (head , attributes , args); F0272 |
| T0024 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_after_head#method1:into:evaluated_head; F0274 |
| T0025 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#method14:install_prepared_call_arguments:self; F0275;F0275P |
| T0026 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#method16:install_prepared_call_arguments:self; F0276;F0275P |
| T0027 | untyped → removed | rum_runtime::runtime::eval_core::symbol::<EvaluateStoredOwnValueSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#method4:map:(\| \| -> Result < rum_core :: Expr , EvalError > { if outcome . matched { session . evaluate_accepted_own_value (& symbol , body , delayed) } else { session . evaluate_symbol_after_stored_own_value (symbol) } }) (); F0277 |
| Whole0029 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call1:prepared_call_arguments_from_continuation_expr:; D009 |
| Whole0030 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call4:from_continuation_expr:; D125 |
| Whole0031 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call5:from_continuation_expr:; D125 |
| Whole0032 | legacy-use → removed | rum_runtime::runtime::eval_core::symbol::<EvaluateStoredOwnValueSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr:; D049 |
| F0275P | absent → typed | rum_runtime::runtime::eval_core::call::PreparedDispatchFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; F0275;F0276 |
| M0282S | absent → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#macro2:resume_typed_or_suspend:result; M0282 |
| S3 | absent → implemented | S3 evaluation dispatch: 12 typed implementations and 13 typed macro sites; T0023;T0024;T0025;T0026;T0027;D0001 |
| X0040 | absent → legacy-test | rum_runtime::runtime::eval_call_prepare::prepared_slot_finalizer_fixture#method1:into_continuation_expr:finish_suspended_prepared_call (SuspendedPreparedCallArguments { evaluated_args : PreparedArgumentAccumulator :: freeze (prepared . evaluated_args . iter () . cloned () . collect () ,) , unevaluated_restores : prepared . unevaluated_restore . as_ref () . map (\| restore \| UnevaluatedRestoreAccumulator :: freeze (restore . wrappers . clone ())) , }); S3 typed finalizer materialized through the unchanged Legacy provider |
| M0283S | absent → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#macro2:resume_typed_or_suspend:result; M0283 |
| X0041 | absent → legacy-test | rum_stdlib::tests::retention_closure::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views#reference7:resume_or_suspend:; S3 A3 AST-only legacy-opacity control; macro argument spans preserved |

### 3. Watched failures and final equality in both builds

The six replacement S3 divergences were watched on byte-identical S2 production in both builds. The three If-held and three listable builtin-completion fixtures isolate F0278’s semantic Some; none crosses S13’s skipped Function map. The director-reassigned six original Fold/Nest cases and their failures remain archived for S13; F0528/M0563 remain untouched. The following are retained pre-conversion outputs, not newly rebuilt baselines.

**r1-pre-builtin-held-default: exit 101.** Full raw output: `.temp/brief-609/r1-pre-builtin-held-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.29s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-builtin-held-no-default: exit 101.** Full raw output: `.temp/brief-609/r1-pre-builtin-held-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.13s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 661, identity: None, text: SymbolText { id: 661, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-default: exit 101.** Full raw output: `.temp/brief-609/r1-pre-threaded-dv-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.35s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 673, identity: None, text: SymbolText { id: 673, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-no-default: exit 101.** Full raw output: `.temp/brief-609/r1-pre-threaded-dv-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.46s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

Final runtime oracle equality uses unchanged Checks::ALL and unconditional FaultProbe absence. All six S3 cases and both prepared.twice/head.map.twice cases pass in each build. All28 immutable ordinary controls pass in each build,336 zero-fault observations per build. Full oracle58/58 per build,88016 final observations per build; every one of the nine reported aggregate fault-occurrence deltas is0. Both full oracle builds are warning-free.

**r5-ordinary-default: exit 0.** Full raw output: `.temp/brief-609/r5-ordinary-default.log`.

```text
cargo test --manifest-path .temp/brief-609/r2/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
```

```text
     Running witness_test.rs (.temp/brief-609/r2/ordinary-controls/target/debug/deps/ordinary-5cf883959ece1e3c)
running 1 test
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

**r5-ordinary-no-default: exit 0.** Full raw output: `.temp/brief-609/r5-ordinary-no-default.log`.

```text
cargo test --manifest-path .temp/brief-609/r2/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
```

```text
     Running witness_test.rs (.temp/brief-609/r2/ordinary-controls/target/debug/deps/ordinary-8f58f702763e7810)
running 1 test
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

Final suspension oracle: default

```text
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=17175
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=17187
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=17199
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=17211
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=17223
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=17235
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=17247
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=17259
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88016
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.27s
```

Final suspension oracle: no-default

```text
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=17175
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=17187
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=17199
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=17211
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=17223
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=17235
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=17247
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=17259
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88016
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.02s
```

### 4. Analyzer, inventories, certificates and complete gate ledger

All four views use one exact checked adapter. The final single expression arm has five explicit unresolved expansion operations; the fifth accounts cold-helper closure capture/move of witness, session, request and frame inputs, FnOnce call and closure drop. No retention admission is inferred. Legacy macros remain opaque; unknown success bindings remain unknown and do not leak. The stale R4 macro definition was watched red (1pass/5fail), then all six final view controls passed. Definition drift now rejects three mutations in each of all four views: producer binding, inline policy and move capture (12 deliberate rejections).

**r5-views-old-definition: exit 101.** Full raw output: `.temp/brief-609/r5-views-old-definition.log`.

```text
cargo test --manifest-path .temp/brief-609/r5/view-fixture/Cargo.toml --test frontier macro_ -- --nocapture
```

```text
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling proc-macro2 v1.0.107
   Compiling syn v3.0.4
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.229
   Compiling serde v1.0.229
   Compiling serde_derive v1.0.229
   Compiling thiserror v2.0.20
   Compiling num-integer v0.1.47
   Compiling thiserror-impl v2.0.20
   Compiling num-bigint v0.4.8
   Compiling syn v2.0.119
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling brief609-frontier-fixture v0.0.0 (/home/remi/dev/rum/.temp/brief-609/r5/view-fixture)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 46.86s
     Running main.rs (.temp/brief-609/r5/view-fixture/target/debug/deps/frontier-eb82c8738f75f1e7)

running 6 tests
test checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views ... 
thread 'checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
FAILED
test checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view ... 
thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
ok
test checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
FAILED
test checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
FAILED
test checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select ... 
thread 'checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select' panicked at checker/typed_resume_macro.rs:183:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:2:9-7:10: typed resume macro definition changed
FAILED

failures:

failures:
    checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views
    checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments
    checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments
    checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments
    checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select

test result: FAILED. 1 passed; 5 failed; 0 ignored; 0 measured; 2 filtered out; finished in 89.43s

error: test failed, to rerun pass `--test frontier`
```

**r5-views-final: exit 0.** Full raw output: `.temp/brief-609/r5-views-final.log`.

```text
cargo test --manifest-path .temp/brief-609/r5/view-fixture/Cargo.toml --test frontier macro_ -- --nocapture
```

```text
   Compiling brief609-frontier-fixture v0.0.0 (/home/remi/dev/rum/.temp/brief-609/r5/view-fixture)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.40s
     Running main.rs (.temp/brief-609/r5/view-fixture/target/debug/deps/frontier-eb82c8738f75f1e7)

running 6 tests
test checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views ... LEGACY_MACRO_OPAQUE regions/locator/methods/types
ok
test checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments ... TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:151:25-151:37 std::option::Option::Some
TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:17-153:32 std::result::Result::Ok
TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:20-153:31 std::option::Option::Some
ok
test checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view ... 
thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
TYPED_MACRO_DRIFT producer regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT producer locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT producer methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT producer types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT cold helper regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT cold helper locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT cold helper methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT cold helper types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT owned capture regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT owned capture locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT owned capture methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT owned capture types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
ok
test checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments ... TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:151:25-151:37 std::option::Option::Some std::option::Option<usize>
TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:17-153:32 std::result::Result::Ok std::result::Result<std::option::Option<rum_core::expr::Expr>, rum_runtime::runtime::session::session_state::EvalError>
TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:20-153:31 std::option::Option::Some std::option::Option<rum_core::expr::Expr>
ok
test checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments ... TYPED_MACRO_METHOD crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:152:25-152:39 rum_stdlib::tests::retention_closure::compiler_oracle::corpus::typed_argument_fixture::Local::choose
ok
test checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 88.63s
```

Constructor oracle:7315→7311 final claims,7311 compiler agreements and0 disagreements. The four net removals are fully owned: removed finish_call_dispatch_stage Ok1→0, shared finish_builtin_dispatch Ok0→1, try_dispatch_builtin_call Ok5→3, evaluate_profiled_fixed_point Ok3→2, resume_fixed_point_from Ok4→3. The exact instrument reports285 agreements and0 disagreements; its duplicate TOTAL print is not570. All170 contextual claims remain; one resolution.rs Ok moves395→392. The only S2 contextual removals remain the two explicit M0561 Err constructors moved into checked expansion. Both Python instruments are unchanged.

Historical frontier932→936:916 identical rows,15 exact same-operation moved pairs, and old opaque M0561 RET003 replaced by five explicit RET002 expansion rows. RET00110 unchanged, RET002895→900, RET00327→26. The interim extra Try row disappears with the redundant wrapper; the cold capture rejection replaces it in the final accounting. No unpaired removal, revealed-on-conversion or view-agreement row occurs. The freeze/default rejection remains at resolution.rs35:19–35:46, now with its actual cold-factory witness. Full source snippets and pairs: frontier-s2-classified.json; R4 delta: frontier-r4-diff.json. Historical ACCEPTANCE_BASELINE text927 is deliberately unchanged.

Ownership replay preserves79 approved host-payload identities and59 self-reaching types. R5 unknown captures2959→2956; attributes1514, macros3152 and storage94 retain their counts. All dispositions and identity cells are unchanged. Four verified witness citations move (three F0272 impl13→40, one F0570 impl43→70). The first ownership acceptance correctly rejected those stale citations23pass/2fail; the exact witness-cell replay then passes25/25. Exact old/new rows for every changed TSV are retained as r5/<basename>.transitions.json and in candidate.patch; ownership-witness-diff.json isolates the four cells.

All68 context fingerprints were replayed; final context fnv1a64:bc81091c2485a5f2. Both certificate emitters earned3 production-instance and4 probe marks with0 rejections, then intentionally rejected stale C01 pins. Only four C01/C02 certificate cells were replayed; all seven noncertificate columns remain unchanged. Both final local acceptance gates pass. C01/C02 bodies contain no converted macro and the entire local analyzer is unchanged. Exact fingerprints/certificate transitions: contract-fingerprint-diff.tsv, certificate-diff.json and certificate-replay.patch.

Builds, compiler fixtures, tests and measured workloads were bounded and serial; Cargo used offline/jobs1/test-threads1 with TMPDIR under brief609. The table preserves every R5 top-level gate’s exact test-result lines, including intended negative child tests and the intermediate stale-receipt failures. A no-test-result cell is not counted as passed tests. Full output, command and exit records are beside each log.

| Raw log | Exit | Verbatim totals |
| --- | --- | --- |
| .temp/brief-609/r5-address-ledger-first.log | 0 | No test-result line |
| .temp/brief-609/r5-all-ordinary-attribution.log | 0 | No test-result line |
| .temp/brief-609/r5-all-ordinary-means.log | 0 | No test-result line |
| .temp/brief-609/r5-all-repeat-summary.log | 0 | No test-result line |
| .temp/brief-609/r5-alloc-base.log | 0 | No test-result line |
| .temp/brief-609/r5-alloc-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-build-cli-alloc.log | 0 | No test-result line |
| .temp/brief-609/r5-build-cli.log | 0 | No test-result line |
| .temp/brief-609/r5-build-resume-alloc.log | 0 | No test-result line |
| .temp/brief-609/r5-build-resume.log | 0 | No test-result line |
| .temp/brief-609/r5-census-default-final.log | 101 | test result: FAILED. 2 passed; 3 failed; 1 ignored; 0 measured; 0 filtered out; finished in 100.05s |
| .temp/brief-609/r5-census-default-reconciled.log | 0 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.60s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.58s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.80s |
| .temp/brief-609/r5-census-default.log | 101 | test result: FAILED. 2 passed; 3 failed; 1 ignored; 0 measured; 0 filtered out; finished in 98.51s |
| .temp/brief-609/r5-census-discovery.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 21.43s |
| .temp/brief-609/r5-census-focused-reconciled.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 22.60s |
| .temp/brief-609/r5-census-no-default.log | 0 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.40s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.42s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.18s |
| .temp/brief-609/r5-cold-boundaries-production.log | 0 | No test-result line |
| .temp/brief-609/r5-cold-boundary-comparison-both-successors.log | 0 | No test-result line |
| .temp/brief-609/r5-cold-boundary-comparison.log | 0 | No test-result line |
| .temp/brief-609/r5-cold-boundary-native-reconciled.log | 0 | No test-result line |
| .temp/brief-609/r5-cold-boundary-native.log | 1 | No test-result line |
| .temp/brief-609/r5-constructor-oracle.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 249.87s |
| .temp/brief-609/r5-controls-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.98s |
| .temp/brief-609/r5-controls-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.97s |
| .temp/brief-609/r5-disassemble.log | 0 | No test-result line |
| .temp/brief-609/r5-emit-certificates-default.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 19.40s |
| .temp/brief-609/r5-emit-certificates-no-default.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.55s |
| .temp/brief-609/r5-exact-oracle.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 198.47s |
| .temp/brief-609/r5-execution-report.log | 0 | No test-result line |
| .temp/brief-609/r5-final-measurement-postprocess.log | 0 | No test-result line |
| .temp/brief-609/r5-final-pre-timing-audit.log | 0 | No test-result line |
| .temp/brief-609/r5-final-source-audit.log | 0 | No test-result line |
| .temp/brief-609/r5-format-adapter.log | 0 | No test-result line |
| .temp/brief-609/r5-format-receipts.log | 0 | No test-result line |
| .temp/brief-609/r5-format.log | 0 | No test-result line |
| .temp/brief-609/r5-frame-roots.log | 0 | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.22s |
| .temp/brief-609/r5-frontier-final.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 25.96s |
| .temp/brief-609/r5-full-addresses-ordinary.log | 0 | No test-result line |
| .temp/brief-609/r5-full-addresses-resume.log | 0 | No test-result line |
| .temp/brief-609/r5-instructions-base-1.log | 0 | No test-result line |
| .temp/brief-609/r5-instructions-base-2.log | 0 | No test-result line |
| .temp/brief-609/r5-instructions-candidate-1.log | 0 | No test-result line |
| .temp/brief-609/r5-instructions-candidate-2.log | 0 | No test-result line |
| .temp/brief-609/r5-local-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 18.72s |
| .temp/brief-609/r5-local-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.25s |
| .temp/brief-609/r5-native-skips.log | 0 | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s |
| .temp/brief-609/r5-oracle-default.log | 0 | test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.27s |
| .temp/brief-609/r5-oracle-no-default.log | 0 | test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.02s |
| .temp/brief-609/r5-ordinary-batch-01.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-02.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-03.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-04.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-05.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-06.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-07.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-08.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-09.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-10.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-batch-11.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-copy-attribution.log | 0 | No test-result line |
| .temp/brief-609/r5-ordinary-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s |
| .temp/brief-609/r5-ordinary-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s |
| .temp/brief-609/r5-ownership-discovery.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 5.91s |
| .temp/brief-609/r5-ownership-final.log | 101 | test result: FAILED. 23 passed; 2 failed; 0 ignored; 0 measured; 1126 filtered out; finished in 67.84s |
| .temp/brief-609/r5-ownership-reconciled.log | 0 | test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1126 filtered out; finished in 69.71s |
| .temp/brief-609/r5-perf-check.log | 0 | No test-result line |
| .temp/brief-609/r5-performance-report-final.log | 0 | No test-result line |
| .temp/brief-609/r5-performance-report.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-all-resume.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-do-count-null-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-do-count-null-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-do-iterator-null-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-do-iterator-null-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-dot-three-no-input-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-dot-three-no-input-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-downvalue-pattern-hit-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-downvalue-pattern-hit-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-fold-through-source-256-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-fold-through-source-256-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-fold-through-source-64-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-fold-through-source-64-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-fold-through-source-8-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-fold-through-source-8-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-eight-256-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-eight-256-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-eight-64-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-eight-64-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-eight-8-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-eight-8-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-first-256-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-first-256-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-first-64-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-first-64-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-first-8-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-map-first-8-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-nest-map-256-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-nest-map-256-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-nest-map-64-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-nest-map-64-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-nest-map-8-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-nest-map-8-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-select-restart-256-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-select-restart-256-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-select-restart-64-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-select-restart-64-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-select-restart-8-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-select-restart-8-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-take-map-256-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-take-map-256-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-take-map-64-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-take-map-64-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-take-map-8-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-depth-take-map-8-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-drop-realize-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-drop-realize-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-first-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-first-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-foldlist-realize-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-foldlist-realize-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-map-realize-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-map-realize-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-nestlist-realize-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-nestlist-realize-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-part-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-part-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-range-take-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-range-take-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-select-scan-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-select-scan-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-table-realize-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-table-realize-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-transparent-adapters-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-lazy-transparent-adapters-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-literal-integer-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-literal-integer-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-scan-no-input-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-scan-no-input-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-take-multispec-small-base.log | 0 | No test-result line |
| .temp/brief-609/r5-repeat-take-multispec-small-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-resume-callgrind-1.log | 0 | No test-result line |
| .temp/brief-609/r5-resume-callgrind-2.log | 0 | No test-result line |
| .temp/brief-609/r5-resume-callgrind-3.log | 0 | No test-result line |
| .temp/brief-609/r5-semantic-costs-with-driver.log | 0 | No test-result line |
| .temp/brief-609/r5-semantic-costs.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-expr-debug-build.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-expr-debug-check.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-expr-release-build.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-expr-release-check.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-format-all.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-format.log | 1 | No test-result line |
| .temp/brief-609/r5-stack-ident-debug-build.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-ident-debug-depth.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-ident-debug-measure.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-ident-debug-trace.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-ident-release-build.log | 0 | No test-result line |
| .temp/brief-609/r5-stack-ident-release-check.log | 0 | No test-result line |
| .temp/brief-609/r5-timing-base.log | 0 | No test-result line |
| .temp/brief-609/r5-timing-candidate.log | 0 | No test-result line |
| .temp/brief-609/r5-timing-comparison.log | 0 | No test-result line |
| .temp/brief-609/r5-typed-runtime.log | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.49s |
| .temp/brief-609/r5-views-final.log | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 88.63s |
| .temp/brief-609/r5-views-inventory.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 10.44s |
| .temp/brief-609/r5-views-old-definition.log | 101 | test result: FAILED. 1 passed; 5 failed; 0 ignored; 0 measured; 2 filtered out; finished in 89.43s |
| .temp/brief-609/r5-working-48.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1171 filtered out; finished in 0.01s |
| .temp/brief-609/r5-workspace.log | 0 | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s<br>test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s<br>test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s<br>test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s<br>test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.30s<br>test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s<br>test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s<br>test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s<br>test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.05s<br>test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s<br>test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s<br>test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s<br>test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s<br>test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 44.45s<br>test result: ok. 1098 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1198.79s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s<br>test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s<br>test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s<br>test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.28s<br>test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s<br>test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s<br>test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s<br>test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s<br>test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s<br>test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.78s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 145.32s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.11s<br>test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.58s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| .temp/brief-609/r5-xtree-massif.log | 0 | No test-result line |
| .temp/brief-609/r5-xtree-probe.log | 1 | No test-result line |

**r5-controls-default: exit 0.** Full raw output: `.temp/brief-609/r5-controls-default.log`.

```text
cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.32s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-d6fbf6132444c3d1)

running 1 test
test typed_slot_compile_controls ... PAIR first-chain-input: positive PASS; negative E0271 at src/lib.rs:27:35
PAIR chain-adjacency: positive PASS; negative E0271 at src/lib.rs:34:70
PAIR unbound-chain-constructor: positive PASS; negative E0599 at src/lib.rs:40:41
PAIR evaluate-input: positive PASS; negative E0271 at src/lib.rs:46:66
PAIR evaluate-output: positive PASS; negative E0271 at src/lib.rs:52:66
PAIR finalizer-output: positive PASS; negative E0271 at src/lib.rs:58:48
PAIR opaque-action: positive PASS; negative E0599 at src/lib.rs:63:39
PAIR token-no-clone: positive PASS; negative E0599 at src/lib.rs:69:18
PAIR token-no-mint: positive PASS; negative E0599 at src/lib.rs:75:27
PAIR token-no-escape: positive PASS; negative lifetime at src/lib.rs:81:1
PAIR sealed-extension: positive PASS; negative E0277 at src/lib.rs:87:20
PAIR codec-key-private: positive PASS; negative E0603 at src/lib.rs:93:18
PAIR opaque-selector-pass: positive PASS; negative E0271 at src/lib.rs:100:35
PAIR mode-not-frame-chosen: positive PASS; negative E0599 at src/lib.rs:106:65
PAIR mode-private-selection: positive PASS; negative E0603 at src/lib.rs:112:18
PAIR slot-mismatch-private: positive PASS; negative E0451 at src/lib.rs:124:95
PAIR input: positive PASS; negative E0271 at src/lib.rs:131:41
PAIR output-return: positive PASS; negative E0308 at src/lib.rs:138:1
PAIR chain-last-output: positive PASS; negative E0308 at src/lib.rs:145:1
PAIR token-fresh-type: positive PASS; negative E0308 at src/lib.rs:159:47
PAIR token-witness-input: positive PASS; negative E0308 at src/lib.rs:167:17
PAIR deferred-final-output: positive PASS; negative E0308 at src/lib.rs:174:5
PAIR generic-frame: positive PASS; negative E0308 at src/lib.rs:180:1
PAIR closure-return: positive PASS; negative E0308 at src/lib.rs:187:1
PAIR sibling-erased-private: positive PASS; negative E0603 at crates/rum-runtime/src/runtime/mod.rs:135:47
PAIR mode-expr-fixed: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17
PAIR mode-new-state-only: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed.rs:66:40
PAIR mode-one-associated-type: positive PASS; negative E0201 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-no-codec-override: positive PASS; negative E0407 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-legacy-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:74:19
PAIR mode-state-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:79:15
COMPILE CONTROLS: 31 positive / 31 intended rejections (22 S1 + 7 S2 + opaque fault + token witness); raw Cargo JSON and primary spans retained
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.98s
```

**r5-controls-no-default: exit 0.** Full raw output: `.temp/brief-609/r5-controls-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.34s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-e5d376d145e23731)

running 1 test
test typed_slot_compile_controls ... PAIR first-chain-input: positive PASS; negative E0271 at src/lib.rs:27:35
PAIR chain-adjacency: positive PASS; negative E0271 at src/lib.rs:34:70
PAIR unbound-chain-constructor: positive PASS; negative E0599 at src/lib.rs:40:41
PAIR evaluate-input: positive PASS; negative E0271 at src/lib.rs:46:66
PAIR evaluate-output: positive PASS; negative E0271 at src/lib.rs:52:66
PAIR finalizer-output: positive PASS; negative E0271 at src/lib.rs:58:48
PAIR opaque-action: positive PASS; negative E0599 at src/lib.rs:63:39
PAIR token-no-clone: positive PASS; negative E0599 at src/lib.rs:69:18
PAIR token-no-mint: positive PASS; negative E0599 at src/lib.rs:75:27
PAIR token-no-escape: positive PASS; negative lifetime at src/lib.rs:81:1
PAIR sealed-extension: positive PASS; negative E0277 at src/lib.rs:87:20
PAIR codec-key-private: positive PASS; negative E0603 at src/lib.rs:93:18
PAIR opaque-selector-pass: positive PASS; negative E0271 at src/lib.rs:100:35
PAIR mode-not-frame-chosen: positive PASS; negative E0599 at src/lib.rs:106:65
PAIR mode-private-selection: positive PASS; negative E0603 at src/lib.rs:112:18
PAIR slot-mismatch-private: positive PASS; negative E0451 at src/lib.rs:124:95
PAIR input: positive PASS; negative E0271 at src/lib.rs:131:41
PAIR output-return: positive PASS; negative E0308 at src/lib.rs:138:1
PAIR chain-last-output: positive PASS; negative E0308 at src/lib.rs:145:1
PAIR token-fresh-type: positive PASS; negative E0308 at src/lib.rs:159:47
PAIR token-witness-input: positive PASS; negative E0308 at src/lib.rs:167:17
PAIR deferred-final-output: positive PASS; negative E0308 at src/lib.rs:174:5
PAIR generic-frame: positive PASS; negative E0308 at src/lib.rs:180:1
PAIR closure-return: positive PASS; negative E0308 at src/lib.rs:187:1
PAIR sibling-erased-private: positive PASS; negative E0603 at crates/rum-runtime/src/runtime/mod.rs:135:47
PAIR mode-expr-fixed: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17
PAIR mode-new-state-only: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed.rs:66:40
PAIR mode-one-associated-type: positive PASS; negative E0201 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-no-codec-override: positive PASS; negative E0407 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-legacy-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:74:19
PAIR mode-state-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:79:15
COMPILE CONTROLS: 31 positive / 31 intended rejections (22 S1 + 7 S2 + opaque fault + token witness); raw Cargo JSON and primary spans retained
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.97s
```

**r5-constructor-oracle: exit 0.** Full raw output: `.temp/brief-609/r5-constructor-oracle.log`.

```text
cargo test -p rum-stdlib --lib census_constructor_resolutions_are_compiler_observed -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
A1_CORPUS_TOTAL agreements=7311 disagreements=0 claims=7311
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 249.87s
```

**r5-exact-oracle: exit 0.** Full raw output: `.temp/brief-609/r5-exact-oracle.log`.

```text
cargo test -p rum-stdlib --lib exact_types_and_constructor_claims_match_rustc -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
A1_ORACLE_TOTAL agreements=285 disagreements=0
A1_ORACLE_TOTAL agreements=285 disagreements=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 198.47s
```

**r5-workspace: exit 0.** Full raw output: `.temp/brief-609/r5-workspace.log`.

```text
cargo test --workspace --no-fail-fast
```

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
     Running unittests src/main.rs (target/debug/deps/rum_cli-0aee412e0b99cbf1)
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/cli.rs (target/debug/deps/cli-bc0b4c8761b62ae9)
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/input_requests.rs (target/debug/deps/input_requests-e84d0b9e62f1dd0f)
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/terminal.rs (target/debug/deps/terminal-70e027c8b6d403e3)
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
     Running unittests src/lib.rs (target/debug/deps/rum_core-05ee71324ea83d3f)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-058a94fcf3064f03)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-c657e563db7a31de)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-cb1c0b1fa13c3a60)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-0c15fc615add95e2)
running 90 tests
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-75627f143cb45b08)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-f1d7a1c822224b1c)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/capture_output_lifecycle.rs (target/debug/deps/capture_output_lifecycle-3426e74b0e43035a)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-3f3ebe72da026eee)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-52956b33f57a0130)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.30s
     Running tests/installer.rs (target/debug/deps/installer-245b9a9d0dd76683)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-cf4abded19978b7b)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-7db6f3bc8fd1b373)
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-bcf408336a660b97)
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/shared_accumulator_completion.rs (target/debug/deps/shared_accumulator_completion-4fc6d427214e6aa2)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/stress.rs (target/debug/deps/stress-29e64a7d12eeb866)
running 6 tests
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-0e6a452c6cdaf889)
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.05s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-7953000542543e9e)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-598afbd7e7b200bf)
running 151 tests
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-0f1e351c2b3d67dc)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-4387fdc47fc58f50)
running 48 tests
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-11b7b14c69d5d259)
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-02bf146f80080963)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-f6425a9dcb35714f)
running 1172 tests
test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 44.45s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-8951251f7d2e445e)
running 1151 tests
test result: ok. 1098 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1198.79s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-d2ad2c98e57a1cba)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-f117f222c1cc964c)
running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-368dcefe778b9dd9)
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-2c9bd859cff3337d)
running 47 tests
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-ef76e1f37b5e402d)
running 69 tests
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.28s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-dbe263d2d0c33a6d)
running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-13289bfd2ecc5d28)
running 100 tests
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-487a4fb3b8b94ac7)
running 71 tests
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-4182a401e6279d72)
running 40 tests
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-757402035d2c0050)
running 51 tests
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-58e5a1c5f9b651ff)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.78s
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-3f7c0defd057db37)
running 6 tests
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 145.32s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-d72233bfaf7c8103)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.11s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-08cb56e2fd17a1b7)
running 193 tests
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-f8efeb939a755a12)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
   Doc-tests rum_host
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.58s
   Doc-tests rum_jupyter
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
running 22 tests
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s
   Doc-tests rum_stdlib
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**r5-native-skips: exit 0.** Full raw output: `.temp/brief-609/r5-native-skips.log`.

```text
cargo test -p rum-jupyter --lib zeromq::tests::native -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-dd24b36d3edaa47a)
running 9 tests
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s
```

The completed workspace command exits0:53 test-result suites,3461 harness-reported passed,0 failed and59 ignored. Nine of those harness-success rows are native ZeroMQ early returns, not exercised native coverage: the focused existing-native-test rerun with --nocapture reports exactly9 explicit loopback-permission skips (9 harness-success/0fail). Thus3452 exercised test successes,59 ignored and9 native environment skips are reported separately. No native ZeroMQ pass is claimed. All53 verbatim totals and suite names are in workspace-summary.json. Runtime1169/0/3ignored and stdlib1098/0/53ignored both pass; the full workspace also passes suspension oracle58/0 and the final census/compile controls.

The four isolated native builds succeeded with zero compiler errors and12 inherited warning occurrences (three unused runtime helper functions in each build): register_test_if_builtin, register_test_return_builtin and register_test_vm_template_builtins. All three also occur in S2 and their source is unchanged. The working48 gate and workspace carry these same inherited test-helper warnings; no new warning is introduced. Production compiler instruments and both complete suspension oracles are warning-free. Native build details: stack/build-ledger.json. All fresh-process native probe test totals are retained in stack/probe-test-results.tsv; a successful probe with StackLimit is an observation, not a successful guest depth.

### 5. Native stack, allocation, instruction, resume attribution and timing evidence

Started from exactly R4 attempt5-source, not fix1/tree. Both resumers now walk stages in a loop; all captured stage indices and resumed k+1 contracts are unchanged. Prepared success, already-prepared skips and None stage results increment the local index; Some, final assembly, error and suspension return. The cold helper stays. Both macro forms were measured; because the single expression arm passes debug, the identifier arm was removed as directed. The seven promoted runtime files are byte-identical to the measured expression-only source after formatting.

| Profile / shape | S2 B/level | R4 best | Loop+identifier | Final expression-only | Final max / first limit |
| --- | --- | --- | --- | --- | --- |
| debug peel | 21424 | 21184 | 18864 | 19472 | 55 /56 |
| debug arithmetic | 37728 | 36768 | 34320 | 35552 | 28 /29 |
| release peel | 8096 | 8416 | 7568 | 7632 | 139 /140 |
| release arithmetic | 14720 | 15104 | 14256 | 14512 | 72 /73 |

Fresh-process bisection uses the unchanged1048576B native guard and existing libtest thread stack; RUST_MIN_STACK is absent. Only the scratch probe’s separate logical recursion ceiling is raised to100000; its peel AST is constructed iteratively to exclude parser limits. The unchanged48-level regression test passes on the working tree and in the full workspace. Safe leaf-marker slopes at four heights and three repetitions each have zero residual and match complete resident-frame sums. The final cycles have no unaccounted outgoing-stack-argument residue. There is one resident resumer per guest call: peel1 versus S2’s2; arithmetic2 versus S2’s3, because that shape has two guest calls per level. Raw depths, hashes, markers, native paths and full addressed prologues are in r5/stack/. The following prologues exclude the8B return address added for every resident frame in the totals; S2 release also pays32B outgoing arguments per cycle, final pays0.

#### debug prologues

| Function | S2 count peel / arithmetic | Final count | S2 bytes | Entry S3 bytes | R4 best bytes | R5 final bytes |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| evaluate | 1 / 2 | 1 / 2 | 312 | 312 | 312 | 312 |
| evaluate_profiled_fixed_point | 1 / 2 | 1 / 2 | 2072 | 2568 | 1816 | 1976 |
| evaluate_once_unstable_after_input_check_with_cycle_tracking | 1 / 2 | 1 / 2 | 184 | 184 | 184 | 184 |
| evaluate_once_in_accounting_mode | 1 / 2 | 1 / 2 | 2264 | 2264 | 2264 | 2264 |
| evaluate_inner_unstable | 1 / 2 | 1 / 2 | 1000 | 1000 | 1000 | 1000 |
| evaluate_call | 1 / 2 | 1 / 2 | 1288 | 856 | 856 | 856 |
| resume_call_after_head | 1 / 2 | 1 / 2 | 552 | 568 | 568 | 568 |
| resume_call_with_dispatch_plan | 2 / 3 | 1 / 2 | 1768 | 2840 | 2184 | 2360 |
| run_call_dispatch_stage | 1 / 1 | 1 / 1 | 1880 | 1656 | 1480 | 1624 |
| try_apply_down_values | 1 / 1 | 1 / 1 | 2088 | 2088 | 2088 | 2088 |
| try_apply_down_values_from_symbol | 1 / 2 | 1 / 2 | 3800 | 3800 | 3800 | 3800 |
| evaluate_definition_rhs | 1 / 1 | 1 / 1 | 1336 | 1336 | 1336 | 1336 |
| evaluate_with_option_value_scope | 1 / 1 | 1 / 1 | 1000 | 1000 | 1000 | 1000 |
| prepare_arguments_for_call_with_unevaluated_restore | 0 / 1 | 0 / 1 | 344 | 344 | 344 | 344 |
| prepare_arguments_for_call_with_unevaluated_restore::{{closure}} | 0 / 1 | 0 / 1 | 2200 | 2680 | 1736 | 1896 |
| evaluate_atom_fast_path | 0 / 1 | 0 / 1 | 424 | 424 | 424 | 424 |
| evaluate_call::{{closure}} | 0 / 0 | 0 / 0 | inlined / absent | 952 | inlined / absent | inlined / absent |

#### release prologues

| Function | S2 count peel / arithmetic | Final count | S2 bytes | Entry S3 bytes | R4 best bytes | R5 final bytes |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| evaluate_definition_rhs | 1 / 1 | 1 / 1 | 840 | 840 | 840 | 840 |
| try_apply_down_values_from_symbol | 1 / 2 | 1 / 2 | 1272 | 1272 | 1272 | 1272 |
| try_apply_down_values | 1 / 1 | 1 / 1 | 776 | 776 | 776 | 776 |
| run_call_dispatch_stage | 1 / 1 | 1 / 1 | 296 | 280 | 280 | 280 |
| resume_call_with_dispatch_plan | 2 / 3 | 1 / 2 | 584 | 920 | 808 | 904 |
| resume_call_after_head | 1 / 2 | 0 / 0 | 632 | inlined / absent | inlined / absent | inlined / absent |
| evaluate_inner_unstable | 1 / 2 | 1 / 2 | 1080 | 1656 | 1608 | 1576 |
| evaluate_once_unstable_after_input_check_with_cycle_tracking | 1 / 2 | 1 / 2 | 760 | 760 | 760 | 760 |
| evaluate | 1 / 2 | 1 / 2 | 840 | 904 | 840 | 840 |
| evaluate_with_option_value_scope | 1 / 1 | 1 / 1 | 312 | 312 | 312 | 312 |
| evaluate_atom_fast_path | 0 / 1 | 0 / 1 | 408 | 408 | 408 | 408 |
| prepare_arguments_for_call_with_unevaluated_restore | 0 / 1 | 0 / 1 | 984 | 1176 | 936 | 1064 |

All four final candidate binaries were built serially, release codegen-units=1, warning-free, with test-continuation-slots absent. The four S2 bases were reused without rebuilding. All eight hashes were verified. Each arm uses separate normal/allocation binaries and the unchanged brief608 resume harness. Candidate value checks pass all64 labels before measurement.

| Arm | Binary | SHA-256 |
| --- | --- | --- |
| base | rum-cli | b553a00e63e8af387e3141fe35764f54515a3e3154fd09b2756f911a2e87d9bf |
| base | rum-cli-alloc | f1fb356eb26f45dedaa09ad46e8d697dfde6bd10fbfe1aa12290f61dfcf6e9f1 |
| base | resume | 57e800bca2cca0d327b358f410d34ced30787410bdee6afe5838f9334a24e81f |
| base | resume-alloc | 73f279bfb5e13ab4daa34242a9d591e52d9d8012267197268e80f63d41ecc503 |
| candidate | rum-cli | 0a79b7d5dcdb4a055515e860861459d77fbcc1b93ab54c0a7fd7605c8de5b417 |
| candidate | rum-cli-alloc | 13a2fcee132f0e28f74857177a15f106b0a8655ea03ce201733c83a72a947546 |
| candidate | resume | d2602129b7af3bac62dbf073ff285a0207506a5d7a2c2d19d8bd1bd8e02c846b |
| candidate | resume-alloc | 30e7fd7fff91b84c38ba8ba832c77c0773c992ad18f5812e0c3aa9cfd9b7f378 |

Allocation/byte slopes ran first,128 raw N/2N cells per arm. All59 ordinary labels are exactly equal in both count and bytes. The following table reports every label; fractional slopes reflect the preserved N/2N sources and amortized allocation, not rounded equality. Raw stdout/stderr, commands and parsed counts are under `r5/perf/alloc/`.

| Label | S2 alloc /N | S3 alloc /N | S2 bytes /N | S3 bytes /N |
| --- | --- | --- | --- | --- |
| lazy-range-take-small | 141 | 141 | 2714 | 2714 |
| lazy-range-normal-small | 157 | 157 | 3388 | 3388 |
| lazy-map-realize-small | 333 | 333 | 13740 | 13740 |
| lazy-select-realize-small | 236 | 236 | 6834 | 6834 |
| lazy-select-scan-small | 1471 | 1471 | 62793 | 62793 |
| lazy-nestlist-realize-small | 197 | 197 | 11316 | 11316 |
| lazy-foldlist-realize-small | 216 | 216 | 7604 | 7604 |
| lazy-drop-realize-small | 351 | 351 | 14204 | 14204 |
| lazy-first-small | 64 | 64 | 2868 | 2868 |
| lazy-part-small | 152 | 152 | 6182 | 6182 |
| lazy-transparent-adapters-small | 319 | 319 | 12876 | 12876 |
| lazy-table-construct-short | 63 | 63 | 1537 | 1537 |
| lazy-table-construct-full | 64 | 64 | 1449 | 1449 |
| lazy-table-realize-small | 251 | 251 | 9190 | 9190 |
| lazy-permutations-take-small | 29 | 29 | 1729 | 1729 |
| lazy-subsets-take-small | 29 | 29 | 1417 | 1417 |
| lazy-tuples-take-small | 29 | 29 | 1361 | 1361 |
| scan-no-input | 33 | 33 | 2536 | 2536 |
| optionvalue-three-no-input | 9 | 9 | 1128 | 1128 |
| replacerepeated-no-input | 57 | 57 | 7899 | 7899 |
| dot-three-no-input | 596 | 596 | 23504 | 23504 |
| differences-no-input | 13 | 13 | 704 | 704 |
| do-count-null | 0 | 0 | 0 | 0 |
| do-iterator-null | 0 | 0 | 32 | 32 |
| table-iterator-null | 0 | 0 | 32 | 32 |
| replaceall-simple | 5 | 5 | 1352 | 1352 |
| table-small | 22 | 22 | 977 | 977 |
| table-iterator-plus | 2 | 2 | 72 | 72 |
| table-iterator-pair-list | 2 | 2 | 136 | 136 |
| take-multispec-small | 109 | 109 | 5856 | 5856 |
| lazy-depth-map-build-8 | 16.001 | 16.001 | 1879.864 | 1879.864 |
| lazy-depth-map-first-8 | 188 | 188 | 11176 | 11176 |
| lazy-depth-map-eight-8 | 1464.001 | 1464.001 | 82776.864 | 82776.864 |
| lazy-depth-select-restart-8 | 327 | 327 | 16980 | 16980 |
| lazy-depth-drop-all-one-8 | 170 | 170 | 3128 | 3128 |
| lazy-depth-take-map-8 | 220 | 220 | 12144 | 12144 |
| lazy-depth-fold-through-source-8 | 743.001 | 743.001 | 58440.864 | 58440.864 |
| lazy-depth-nest-map-8 | 192 | 192 | 11964 | 11964 |
| lazy-depth-map-build-64 | 16.001 | 16.001 | 5911.864 | 5911.864 |
| lazy-depth-map-first-64 | 1364 | 1364 | 85320 | 85320 |
| lazy-depth-map-eight-64 | 10872.001 | 10872.001 | 647704.864 | 647704.864 |
| lazy-depth-select-restart-64 | 1671 | 1671 | 99412 | 99412 |
| lazy-depth-drop-all-one-64 | 1178 | 1178 | 17520 | 17520 |
| lazy-depth-take-map-64 | 1620 | 1620 | 93064 | 93064 |
| lazy-depth-fold-through-source-64 | 43334.001 | 43334.001 | 3402649.864 | 3402649.864 |
| lazy-depth-nest-map-64 | 1368 | 1368 | 86108 | 86108 |
| lazy-depth-map-build-256 | 16.001 | 16.001 | 19735.864 | 19735.864 |
| lazy-depth-map-first-256 | 5396 | 5396 | 339528 | 339528 |
| lazy-depth-map-eight-256 | 43128.001 | 43128.001 | 2584600.864 | 2584600.864 |
| lazy-depth-select-restart-256 | 6279 | 6279 | 382036 | 382036 |
| lazy-depth-drop-all-one-256 | 4634 | 4634 | 66864 | 66864 |
| lazy-depth-take-map-256 | 6420 | 6420 | 370504 | 370504 |
| lazy-depth-fold-through-source-256 | 689320.001 | 689320.001 | 54008857.864 | 54008857.864 |
| lazy-depth-nest-map-256 | 5400 | 5400 | 340316 | 340316 |
| literal-integer | 0 | 0 | 0 | 0 |
| downvalue-pattern-hit | 11 | 11 | 908 | 908 |
| module-local | 23.0001 | 23.0001 | 2163.6272 | 2163.6272 |
| block-local | 8 | 8 | 752 | 752 |
| matcher-single-wide-small | 11 | 11 | 287280 | 287280 |
| resume-table | 27.003 | 33.003 | 1742.456 | 2030.456 |
| resume-do-chain | 24 | 30 | 1634 | 2050 |
| resume-call-repeat | 40.007 | 47.007 | 4210.024 | 4522.024 |
| resume-cleanup | 22 | 28 | 1394 | 1746 |
| resume-fold | 41 | 49 | 2873 | 3337 |

Next, two interleaved full instruction pairs ran in order S2-1,S3-1,S2-2,S3-2:512 raw cells, all value-correct. Commands use taskset-c2, perf stat-x,-e cpu_core/instructions/u, stdin/dev/null and the unmodified brief608 N/2N sources. The threshold is max(0.0005×S2 mean,S2 range,S3 range). Conservatively, either paired delta above that threshold flags a label; this includes three borderline labels whose mean delta alone is below it. Forty labels were investigated (35ordinary,all5resume), not suppressed as noise.

| Label | S2 slopes1 /2 | S3 slopes1 /2 | Mean delta | Threshold | Flag |
| --- | --- | --- | --- | --- | --- |
| lazy-range-take-small | 42191.87055 / 42186.42665 | 42209.66525 / 42261.29295 | 46.3305 | 51.6277 | yes |
| lazy-range-normal-small | 54196.45105 / 53882.6849 | 54066.08695 / 54148.18995 | 67.570475 | 313.76615 | no |
| lazy-map-realize-small | 178819.36175 / 178833.7404 | 180106.0414 / 180255.35955 | 1354.1494 | 149.31815 | yes |
| lazy-select-realize-small | 114609.615 / 114621.68145 | 114346.8931 / 114600.3714 | -142.015975 | 253.4783 | no |
| lazy-select-scan-small | 805173.8231 / 805250.8618 | 811361.8318 / 811489.2666 | 6213.20675 | 402.606171225 | yes |
| lazy-nestlist-realize-small | 145246.78245 / 145234.32635 | 146526.2593 / 146525.66035 | 1285.405425 | 72.6202772 | yes |
| lazy-foldlist-realize-small | 112664.4537 / 112736.59915 | 112776.77015 / 112713.151 | 44.43415 | 72.14545 | yes |
| lazy-drop-realize-small | 191468.44335 / 191514.4999 | 192676.9563 / 192610.0721 | 1152.042575 | 95.7457358125 | yes |
| lazy-first-small | 41203.97015 / 41209.05955 | 41328.67425 / 41326.74535 | 121.19495 | 20.603257425 | yes |
| lazy-part-small | 88284.1793 / 88301.2221 | 88703.53405 / 88577.2591 | 347.695875 | 126.27495 | yes |
| lazy-transparent-adapters-small | 181324.98625 / 181414.76685 | 182309.60365 / 182331.1761 | 950.513325 | 90.684938275 | yes |
| lazy-table-construct-short | 23562.7915 / 23609.36485 | 23480.1686 / 23478.46915 | -106.7593 | 46.57335 | no |
| lazy-table-construct-full | 24072.1399 / 24106.87235 | 23991.8166 / 24015.80965 | -85.693 | 34.73245 | no |
| lazy-table-realize-small | 117181.072 / 117228.33695 | 117935.34155 / 118023.1238 | 774.5282 | 87.78225 | yes |
| lazy-permutations-take-small | 28198.6322 / 28232.6817 | 28172.0035 / 28248.2122 | -5.5491 | 76.2087 | no |
| lazy-subsets-take-small | 25415.5372 / 25358.2291 | 25336.51525 / 25336.61245 | -50.3193 | 57.3081 | no |
| lazy-tuples-take-small | 26629.3147 / 26606.6912 | 26592.07385 / 26566.50685 | -38.7126 | 25.567 | no |
| scan-no-input | 29681.0881 / 29681.05725 | 29695.3035 / 29696.30485 | 14.7315 | 14.8405363375 | yes |
| optionvalue-three-no-input | 9589.58805 / 9590.84105 | 9549.1173 / 9549.91775 | -40.697025 | 4.795107275 | no |
| replacerepeated-no-input | 39174.24985 / 39200.418 | 39184.2718 / 39145.23145 | -22.5823 | 39.04035 | no |
| dot-three-no-input | 254199.3949 / 254200.0732 | 254565.9172 / 254565.33585 | 365.892475 | 127.099867025 | yes |
| differences-no-input | 10030.646 / 10037.7145 | 10032.3264 / 10032.8946 | -1.56975 | 7.0685 | no |
| do-count-null | 537.08545 / 536.85325 | 537.99115 / 538.0143 | 1.033375 | 0.268484675 | yes |
| do-iterator-null | 1154.05155 / 1154.17745 | 1154.9464 / 1155.06445 | 0.890925 | 0.57705725 | yes |
| table-iterator-null | 387.12035 / 387.0163 | 386.9072 / 387.05595 | -0.0867499999999 | 0.1935341625 | no |
| replaceall-simple | 11406.34775 / 11406.25245 | 11400.2525 / 11374.58445 | -18.881625 | 25.66805 | no |
| table-small | 14632.77085 / 14633.35615 | 14595.9823 / 14625.9478 | -22.09845 | 29.9655 | no |
| table-iterator-plus | 563.24205 / 563.2175 | 563.25895 / 563.27515 | 0.037275 | 0.2816148875 | no |
| table-iterator-pair-list | 1640.7667 / 1640.9648 | 1640.79905 / 1640.74815 | -0.0921499999999 | 0.820432875 | no |
| take-multispec-small | 49915.099 / 49939.00345 | 50183.1174 / 50183.5422 | 256.278575 | 24.9635256125 | yes |
| lazy-depth-map-build-8 | 18113.971 / 18088.22 | 18043.156 / 18046.782 | -56.1265 | 25.751 | no |
| lazy-depth-map-first-8 | 130396.392 / 130376.285 | 132021.957 / 131621.367 | 1435.3235 | 400.59 | yes |
| lazy-depth-map-eight-8 | 959691.079 / 959606.739 | 971112.546 / 971027.884 | 11421.306 | 479.8244545 | yes |
| lazy-depth-select-restart-8 | 179387.373 / 179390.641 | 182116.193 / 182033.569 | 2685.874 | 89.6945035 | yes |
| lazy-depth-drop-all-one-8 | 46663.753 / 46662.622 | 46567.444 / 46566.908 | -96.0115 | 23.33159375 | no |
| lazy-depth-take-map-8 | 138949.174 / 138954.179 | 140176.974 / 140162.349 | 1217.985 | 69.47583825 | yes |
| lazy-depth-fold-through-source-8 | 589552.602 / 589532.378 | 596856.537 / 596764.283 | 7267.92 | 294.771245 | yes |
| lazy-depth-nest-map-8 | 141008.073 / 141006.189 | 142414.251 / 142803.847 | 1601.918 | 389.596 | yes |
| lazy-depth-map-build-64 | 30116.882 / 30117.734 | 30070.02 / 30057.171 | -53.7125 | 15.058654 | no |
| lazy-depth-map-first-64 | 946182.806 / 946182.621 | 956719.581 / 956717.595 | 10535.8745 | 473.09135675 | yes |
| lazy-depth-map-eight-64 | 7421609.946 / 7421701.561 | 7506195.744 / 7505295.463 | 84089.85 | 3710.82787675 | yes |
| lazy-depth-select-restart-64 | 975466.222 / 972990.799 | 995887.418 / 1001840.73 | 24635.5635 | 5953.312 | yes |
| lazy-depth-drop-all-one-64 | 327581.32 / 327576.349 | 327485.39 / 327483.828 | -94.2255 | 163.78941725 | no |
| lazy-depth-take-map-64 | 1007484.006 / 1007387.015 | 1017965.779 / 1017972.488 | 10533.623 | 503.71775525 | yes |
| lazy-depth-fold-through-source-64 | 33994994.408 / 33996838.038 | 34412160.549 / 34414988.374 | 417658.2385 | 16997.9581115 | yes |
| lazy-depth-nest-map-64 | 958616.144 / 958618.967 | 967653.574 / 969276.291 | 9847.377 | 1622.717 | yes |
| lazy-depth-map-build-256 | 70763.968 / 70761.778 | 70703.164 / 70712.116 | -55.233 | 35.3814365 | no |
| lazy-depth-map-first-256 | 3736085.958 / 3736087.856 | 3765448.805 / 3765173.801 | 29224.396 | 1868.0434535 | yes |
| lazy-depth-map-eight-256 | 29544547.517 / 29544556.654 | 29873361.548 / 29872063.294 | 328160.3355 | 14772.2760428 | yes |
| lazy-depth-select-restart-256 | 3664698.991 / 3664409.403 | 3731333.816 / 3744088.492 | 73156.957 | 12754.676 | yes |
| lazy-depth-drop-all-one-256 | 2218609.028 / 2218612.409 | 2218513.56 / 2218510.186 | -98.8454999998 | 1109.30535925 | no |
| lazy-depth-take-map-256 | 3985661.483 / 3985786.525 | 4027901.578 / 4028295.003 | 42374.2865 | 1992.862002 | yes |
| lazy-depth-fold-through-source-256 | 530561259.269 / 530582504.758 | 537100400.173 / 537078305.723 | 6517470.9345 | 265285.941007 | yes |
| lazy-depth-nest-map-256 | 3733564.742 / 3733556.158 | 3776132.69 / 3776131.986 | 42571.888 | 1866.780225 | yes |
| literal-integer | 294.0678 / 294.01065 | 295.1024 / 295.0726 | 1.048275 | 0.1470196125 | yes |
| downvalue-pattern-hit | 13949.52425 / 13949.4943 | 13969.5786 / 13969.5573 | 20.058675 | 6.9747546375 | yes |
| module-local | 22654.13395 / 22653.2917 | 22662.4602 / 22658.69425 | 6.8644 | 11.3268564125 | no |
| block-local | 14671.82305 / 14671.85565 | 14677.7183 / 14677.9224 | 5.981 | 7.335919675 | no |
| matcher-single-wide-small | 760190.98 / 760214.411 | 760176.041 / 760175.778 | -26.7859999998 | 380.10134775 | no |
| resume-table | 15723.529 / 15726.296 | 18761.166 / 18760.444 | 3035.8925 | 7.86245625 | yes |
| resume-do-chain | 15131.718 / 15142.154 | 18252.184 / 18303.028 | 3140.67 | 50.844 | yes |
| resume-call-repeat | 24627.369 / 24529.493 | 27824.6 / 28021.2 | 3344.469 | 196.6 | yes |
| resume-cleanup | 14269.764 / 14245.487 | 17305.061 / 17251.083 | 3020.4465 | 53.978 | yes |
| resume-fold | 24438.102 / 24431.425 | 28386.691 / 28445.08 | 3981.122 | 58.389 | yes |

Every flagged label then received five fresh N/2N perf slopes per arm and three fresh N/2N Callgrind pairs per arm (six processes per arm). Callgrind N is prebudgeted from the instruction slope to about20million instructions, capped at2000 and at least1; the heavy256 Fold label necessarily uses1/2. Perf retains the original panel N. All raw cells and values are retained. Five-run hardware summaries:

| Label | S2 mean /range | S3 mean /range | Delta I | Delta % | Above repeat threshold |
| --- | --- | --- | --- | --- | --- |
| lazy-range-take-small | 42200.45645 / 58.86635 | 42219.62527 / 47.09405 | 19.16882 | 0.045423252762 | no |
| lazy-map-realize-small | 178824.68328 / 26.4915 | 180083.56088 / 243.9523 | 1258.8776 | 0.70397306284 | yes |
| lazy-select-scan-small | 805254.91288 / 14.7002 | 811538.45952 / 143.78215 | 6283.54664 | 0.780317703065 | yes |
| lazy-nestlist-realize-small | 145306.31158 / 3.41249999998 | 146573.60827 / 196.09465 | 1267.29669 | 0.872155294715 | yes |
| lazy-foldlist-realize-small | 112753.44462 / 91.0304 | 112783.89666 / 180.1727 | 30.45204 | 0.0270076360883 | no |
| lazy-drop-realize-small | 191461.64492 / 143.49565 | 192811.64663 / 498.85895 | 1350.00171 | 0.705102951854 | yes |
| lazy-first-small | 41204.79452 / 150.86015 | 41352.2049 / 96.05135 | 147.41038 | 0.357750552374 | no |
| lazy-part-small | 88360.3718 / 532.3445 | 88708.27266 / 197.53465 | 347.90086 | 0.39372951122 | no |
| lazy-transparent-adapters-small | 181458.95888 / 385.1296 | 182291.91694 / 81.90075 | 832.95806 | 0.459033858202 | yes |
| lazy-table-realize-small | 117182.83089 / 53.5028 | 117939.83857 / 76.4565 | 757.00768 | 0.646005625782 | yes |
| scan-no-input | 29681.23653 / 2.48345 | 29697.81947 / 3.8065 | 16.58294 | 0.055870111689 | yes |
| dot-three-no-input | 254199.30529 / 0.824699999997 | 254565.84465 / 1.04929999998 | 366.53936 | 0.144193690688 | yes |
| do-count-null | 537.02445 / 0.35185 | 538.02361 / 0.2248 | 0.99916 | 0.186054843499 | yes |
| do-iterator-null | 1154.04478 / 0.13515 | 1155.1367 / 0.24335 | 1.09192 | 0.094616779082 | yes |
| take-multispec-small | 49929.51513 / 16.89025 | 50183.27487 / 0.484850000001 | 253.75974 | 0.508235938882 | yes |
| lazy-depth-map-first-8 | 130387.6608 / 41.681 | 131699.7882 / 587.984 | 1312.1274 | 1.00632789326 | yes |
| lazy-depth-map-eight-8 | 959632.9376 / 258.744 | 971030.3468 / 73.9130000001 | 11397.4092 | 1.18768424399 | yes |
| lazy-depth-select-restart-8 | 179506.1586 / 860.328 | 182006.2412 / 326.654 | 2500.0826 | 1.39275589177 | yes |
| lazy-depth-take-map-8 | 138948.5952 / 8.52900000001 | 140195.8274 / 22.166 | 1247.2322 | 0.897621309668 | yes |
| lazy-depth-fold-through-source-8 | 589557.8194 / 32.748 | 596814.0274 / 74.5120000001 | 7256.208 | 1.23078818756 | yes |
| lazy-depth-nest-map-8 | 141045.4736 / 587.996 | 142412.4754 / 21.429 | 1367.0018 | 0.969192250633 | yes |
| lazy-depth-map-first-64 | 946815.7086 / 3179.885 | 956335.2234 / 1742.334 | 9519.5148 | 1.00542425665 | yes |
| lazy-depth-map-eight-64 | 7421445.447 / 1188.709 | 7505794.7634 / 593.289 | 84349.3164 | 1.1365618329 | yes |
| lazy-depth-select-restart-64 | 975726.3996 / 501.979 | 995594.855 / 447.813 | 19868.4554 | 2.03627322251 | yes |
| lazy-depth-take-map-64 | 1007460.1544 / 56.265 | 1018012.3352 / 87.735 | 10552.1808 | 1.04740428233 | yes |
| lazy-depth-fold-through-source-64 | 33997422.807 / 4273.252 | 34411670.0528 / 2268.017 | 414247.2458 | 1.21846661187 | yes |
| lazy-depth-nest-map-64 | 959338.4102 / 1662.794 | 969325.9162 / 3245.07 | 9987.506 | 1.04108267675 | yes |
| lazy-depth-map-first-256 | 3730901.5224 / 13430.36 | 3768024.7892 / 20237.979 | 37123.2668 | 0.995021352805 | yes |
| lazy-depth-map-eight-256 | 29546150.383 / 2306.762 | 29872542.1504 / 3776.658 | 326391.7674 | 1.10468458046 | yes |
| lazy-depth-select-restart-256 | 3665039.3036 / 25890.327 | 3736279.06 / 14400.997 | 71239.7564 | 1.94376514135 | yes |
| lazy-depth-take-map-256 | 3985634.9878 / 690.734 | 4027916.7658 / 487.934 | 42281.778 | 1.06085424605 | yes |
| lazy-depth-fold-through-source-256 | 530578940.726 / 64952.178 | 537068457.154 / 103800.011 | 6489516.4282 | 1.22310101854 | yes |
| lazy-depth-nest-map-256 | 3732225.1108 / 13340.28 | 3785452.1 / 13316.538 | 53226.9892 | 1.42614626985 | yes |
| literal-integer | 294.03862 / 0.2951 | 295.03649 / 0.22175 | 0.99787 | 0.339366985194 | yes |
| downvalue-pattern-hit | 13949.42362 / 0.257699999998 | 13969.35258 / 0.526599999999 | 19.92896 | 0.142865831183 | yes |
| resume-table | 15749.2368 / 65.903 | 18825.3088 / 49.964 | 3076.072 | 19.5315623167 | yes |
| resume-do-chain | 15155.0406 / 73.723 | 18222.1768 / 148.289 | 3067.1362 | 20.2383898595 | yes |
| resume-call-repeat | 24586.2666 / 33.614 | 27842.7108 / 52.045 | 3256.4442 | 13.2449722969 | yes |
| resume-cleanup | 14265.978 / 83.33 | 17257.6532 / 62.157 | 2991.6752 | 20.9706982585 | yes |
| resume-fold | 24433.3232 / 59.889 | 28488.143 / 135.673 | 4054.8198 | 16.5954494475 | yes |

Callgrind totals from three N/2N pairs per arm below use difference of arm means, tested against the larger within-arm range. Every per-function sum was checked against the raw summary slope. Hash-seed variation is retained; a positive total below that range is not by itself called added work. Address evidence is reported separately.

| Ordinary label | CG N | S2 mean | S3 mean | Delta | Larger range | Added by total |
| --- | --- | --- | --- | --- | --- | --- |
| lazy-range-take-small | 474 | 42295.4374121 | 42347.9008439 | 52.4634317862 | 38.0949367089 | yes |
| lazy-map-realize-small | 111 | 178678.864865 | 179956.648649 | 1277.78378378 | 141.387387387 | yes |
| lazy-select-scan-small | 24 | 801685.541667 | 807850.208333 | 6164.66666667 | 277.541666667 | yes |
| lazy-nestlist-realize-small | 137 | 145090.654501 | 146385.749392 | 1295.09489051 | 35.0072992701 | yes |
| lazy-foldlist-realize-small | 177 | 113072.033898 | 113133.932203 | 61.8983050847 | 59.9548022599 | yes |
| lazy-drop-realize-small | 104 | 191250.102564 | 192447.823718 | 1197.72115385 | 88.7403846154 | yes |
| lazy-first-small | 485 | 41363.2879725 | 41478.2852234 | 114.997250859 | 24.3092783505 | yes |
| lazy-part-small | 226 | 88638.4852507 | 89008.4365782 | 369.951327434 | 132.659292035 | yes |
| lazy-transparent-adapters-small | 110 | 181521.336364 | 182272.90303 | 751.566666667 | 237.027272727 | yes |
| lazy-table-realize-small | 170 | 117489.654902 | 118127.841176 | 638.18627451 | 88.8588235294 | yes |
| scan-no-input | 673 | 29688.6121842 | 29699.1302625 | 10.5180782566 | 4.72659732541 | yes |
| dot-three-no-input | 78 | 255597.337607 | 255790.803419 | 193.465811966 | 94.0769230769 | yes |
| do-count-null | 2000 | 536.986166667 | 537.864666667 | 0.8785 | 3.023 | no |
| do-iterator-null | 2000 | 1155.30266667 | 1155.83783333 | 0.535166666667 | 3.264 | no |
| take-multispec-small | 400 | 50619.5691667 | 50873.8283333 | 254.259166667 | 21.5575 | yes |
| lazy-depth-map-first-8 | 153 | 130546.135076 | 131637.712418 | 1091.57734205 | 642.568627451 | yes |
| lazy-depth-map-eight-8 | 20 | 951221.75 | 962312.4 | 11090.65 | 1790.85 | yes |
| lazy-depth-select-restart-8 | 111 | 179505.417417 | 182167.552553 | 2662.13513514 | 924.063063063 | yes |
| lazy-depth-take-map-8 | 143 | 138968.948718 | 140222.083916 | 1253.13519814 | 25.7412587413 | yes |
| lazy-depth-fold-through-source-8 | 33 | 587908.525253 | 594792.686869 | 6884.16161616 | 685.212121212 | yes |
| lazy-depth-nest-map-8 | 141 | 140964.579196 | 142435.877069 | 1471.29787234 | 222.957446808 | yes |
| lazy-depth-map-first-64 | 21 | 946140.857143 | 956093.603175 | 9952.74603175 | 2022.71428571 | yes |
| lazy-depth-map-eight-64 | 2 | 7415476.66667 | 7509302.5 | 93825.8333333 | 41754 | yes |
| lazy-depth-select-restart-64 | 20 | 974882.733333 | 999680.933333 | 24798.2 | 6165.95 | yes |
| lazy-depth-take-map-64 | 19 | 1007613.52632 | 1018143.19298 | 10529.6666667 | 180 | yes |
| lazy-depth-fold-through-source-64 | 1 | 34117250.6667 | 34462304 | 345053.333333 | 116373 | yes |
| lazy-depth-nest-map-64 | 20 | 959040.85 | 969816.516667 | 10775.6666667 | 1811.1 | yes |
| lazy-depth-map-first-256 | 5 | 3719955.06667 | 3762348.4 | 42393.3333333 | 7269.8 | yes |
| lazy-depth-map-eight-256 | 1 | 29322258.6667 | 29659699 | 337440.333333 | 19548 | yes |
| lazy-depth-select-restart-256 | 5 | 3669548.66667 | 3752856.33333 | 83307.6666667 | 39759.2 | yes |
| lazy-depth-take-map-256 | 5 | 3986529.66667 | 4027483.2 | 40953.5333333 | 3974.4 | yes |
| lazy-depth-fold-through-source-256 | 1 | 528264416.667 | 534456500 | 6192083.33333 | 1702345 | yes |
| lazy-depth-nest-map-256 | 5 | 3733334 | 3773269.73333 | 39935.7333333 | 14252.6 | yes |
| literal-integer | 2000 | 294.096 | 294.682 | 0.586 | 1.691 | no |
| downvalue-pattern-hit | 1433 | 13948.2388928 | 13969.1491044 | 20.9102116771 | 5.41311933008 | yes |

| Resume label (N200) | S2 mean | S3 mean | Delta | Larger range | Added by total |
| --- | --- | --- | --- | --- | --- |
| table | 15920.8833333 | 19035.68 | 3114.79666667 | 52.425 | yes |
| do-chain | 15253.115 | 18350.0016667 | 3096.88666667 | 39.745 | yes |
| call-repeat | 25370.9683333 | 28651.05 | 3280.08166667 | 31.01 | yes |
| cleanup | 14248.5083333 | 17279.1383333 | 3030.63 | 101.345 | yes |
| fold | 24547.405 | 28514.73 | 3967.325 | 50.67 | yes |

For every ordinary flagged label, `ordinary-callgrind/address-attribution.json` accounts the exact total delta as object/function/address changes, including every positive function and each changed instruction. `address-ledgers/ordinary/differences.tsv` supplies both addresses and both assembly operations for the full35-label matrix; it includes libc and allocator code, not only runtime symbols. The leading positive self-cost function is shown below. A textual mismatch due to register or stack layout is not automatically additional semantic work.

| Ordinary label | Leading positive function | Self delta I | Example candidate addresses |
| --- | --- | --- | --- |
| lazy-range-take-small | 0x00000000001b1980 | 270 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-map-realize-small | 0x00000000001b1980 | 3180 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-select-scan-small | 0x00000000001b1980 | 14820 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-nestlist-realize-small | 0x00000000001b1980 | 3030 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-foldlist-realize-small | 0x00000000001b1980 | 1620 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-drop-realize-small | 0x00000000001b1980 | 3330 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-first-small | 0x00000000001b1980 | 570 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-part-small | 0x00000000001b1980 | 1230 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-transparent-adapters-small | 0x00000000001b1980 | 2970 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-table-realize-small | 0x00000000001b1980 | 1830 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| scan-no-input | 0x00000000001b1980 | 570 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| dot-three-no-input | 0x00000000001b1980 | 3750 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| do-count-null | rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate | 1 | 0x71fd71, 0x71fd74, 0x71fd77, 0x71fd87, 0x71fd95, 0x71fdc7, 0x71fe17, 0x71fe1f, 0x71fe53, 0x71fe5e, 0x71fe63, 0x71fe8f, 0x71fe97, 0x71fe9d, 0x71fea5, 0x71feae,  |
| do-iterator-null | rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate | 1 | 0x71fd71, 0x71fd74, 0x71fd77, 0x71fd87, 0x71fd95, 0x71fdc7, 0x71fe17, 0x71fe1f, 0x71fe53, 0x71fe5e, 0x71fe63, 0x71fe8f, 0x71fe97, 0x71fe9d, 0x71fea5, 0x71feae,  |
| take-multispec-small | 0x00000000001b1980 | 330 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-map-first-8 | 0x00000000001b1980 | 2790 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-map-eight-8 | 0x00000000001b1980 | 21300 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-select-restart-8 | 0x00000000001b1980 | 3810 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-take-map-8 | 0x00000000001b1980 | 2790 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-fold-through-source-8 | 0x00000000001b1980 | 13140 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-nest-map-8 | 0x00000000001b1980 | 3120 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-map-first-64 | 0x00000000001b1980 | 21270 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-map-eight-64 | 0x00000000001b1980 | 169140 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-select-restart-64 | 0x00000000001b1980 | 23970 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-take-map-64 | 0x00000000001b1980 | 21270 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-fold-through-source-64 | 0x00000000001b1980 | 748980 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-nest-map-64 | 0x00000000001b1980 | 21600 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-map-first-256 | 0x00000000001b1980 | 84630 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-map-eight-256 | 0x00000000001b1980 | 676020 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-select-restart-256 | 0x00000000001b1980 | 93090 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-take-map-256 | 0x00000000001b1980 | 84630 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-fold-through-source-256 | 0x00000000001b1980 | 11842740 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| lazy-depth-nest-map-256 | 0x00000000001b1980 | 84960 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |
| literal-integer | rum_runtime::runtime::resolution::<impl rum_runtime::runtime::model::session::Session>::evaluate | 1 | 0x71fd71, 0x71fd74, 0x71fd77, 0x71fd87, 0x71fd95, 0x71fdc7, 0x71fe17, 0x71fe1f, 0x71fe53, 0x71fe5e, 0x71fe63, 0x71fe8f, 0x71fe97, 0x71fe9d, 0x71fea5, 0x71feae,  |
| downvalue-pattern-hit | 0x00000000001b1980 | 240 | 0x1b1980, 0x1b1984, 0x1b1987, 0x1b198b, 0x1b198d, 0x1b1991, 0x1b1995, 0x1b1a40, 0x1b1a47, 0x1b1a4d, 0x1b1a54, 0x1b1a56, 0x1b1a5b, 0x1b1a62, 0x1b1a64, 0x1b1a69,  |

The integer control proves a small ordinary-path regression independently of randomized hashing. In all three Callgrind pairs, the physical `Session::evaluate` leaf path executes exactly112 self instructions per iteration in S2 and113 in the candidate. The entire112/113-address paths are retained in `ordinary-callgrind/literal-addresses.json`. The candidate adds a stack-address setup at CLI0x71fe9d (`lea 0x120(%rsp),%r14`); the changed move/jump sequence is S2 CLI0x71e6e9 (`jmp`) and0x71e7d3 (`mov %r15,%rsi`) versus candidate CLI0x71feb3 (`mov %rbx,%rsi`) and0x720b58 (`mov %rbx,%rdi`). Those changes together net+1; the mnemonic alignment is a reading aid, not a claim that every differently allocated register is extra work. Hash-function startup fluctuations can mask that single instruction in a whole-process total. This leaf path performs no suspension and the measured+1 remains an ordinary-path defect.

All addresses in the ordinary ledger are relative to the CLI ELF or the named shared library, not interchangeable with the resume-harness addresses. The per-label dispatch-heavy increases are preserved at address granularity in `ordinary-callgrind/address-attribution.json`, with exact deltas summing to the complete profile difference. Positive and negative instruction changes are both retained; loop restructuring is not assumed free merely because it removes recursive resumer calls. Labels whose total mean delta does not exceed the larger arm range are identified as inconclusive by that total alone. None is silently promoted to a performance pass.

All35 ordinary flags have added-work evidence:32 have a total mean increase greater than the larger arm range. The remaining three (`do-count-null`, `do-iterator-null`, `literal-integer`) have exactly the same executed evaluate address/count vectors and the112→113 self-instruction change in each of all three pairs; `three-small-controls-address-proof.json` records that identity. Five-run hardware means remain above their threshold for36/40 flagged labels (31ordinary plus all5resume); the four hardware-inconclusive cases are still shown, and independent Callgrind/address evidence is not relabeled as a hardware pass.

The dominant ordinary increase is extra aggregate copying. For the heavy256 Fold label, added copies per iteration are131586 in `prepare_arguments_for_call_with_unevaluated_restore` (new136-byte site CLI0x59e625),131587 in `evaluate_inner_unstable` (including new168-byte sites0x5a9457/0x5a94a7 and136-byte sites0x5a951a/0x5a9e1f), and164484 in `resume_call_with_dispatch_plan` (136-byte sites0x5a04a0/0x5a0642). `resume_call_after_head` removes32899 copies (S2 site0x59fa24 → candidate0x5a0022). Net394758 extra calls to the libc copy routine add11842740 self instructions; other instruction reductions leave the measured whole-profile net+6192083.333333 I/iteration. The whole hardware five-run increase is+6489516.4282 I/iteration (+1.223101%). This attributes the dispatched-loop regression without assuming that removing recursive calls removes all aggregate moves.

`ordinary-callgrind/copy-attribution.{json,tsv}` gives every base/candidate copy call site, size immediate, caller, assembly window and counts across all35 labels, so the same attribution is available per label, not just for the largest case. These ordinary copies remain unremoved defects.

The full ordinary ledger contains28125 base and28300 candidate nonzero address rows, with4101 changed rows. Six raw changed rows are objectless addresses: S2 0x4070118/0x4070128/0x4070160 versus candidate0x4071118/0x4071128/0x4071160. Each offset-matched pair has exactly the same35-label count vector and netzero delta. They remain explicitly unmapped and paired in `unmapped-reconciliation.json`; every other changed address maps to a hashed ELF/shared library. They are not counted as extra work merely because their names contain relocated addresses.

The five resume panels do **not** satisfy the “one extra Arc and nothing else” gate. Counts reconcile as `typed frames + 3`, rather than `typed frames`. The extra three allocations come from the newly explicit Legacy `Option<Expr>::Some` wire produced by F0278 and consumed by the stage frame. The wrapper is semantically required while that type remains in its global Legacy mode; its measured cost still fails this brief's narrower allocation expectation. No mode was flipped to hide it.

| Workload | Suspending S3 frames per resume (source constructor identities) | Extra adapter Arcs / bytes | Actual count / byte increase | Excess after adapter Arcs |
| --- | --- | --- | --- | --- |
| table | F0526 ×1, F0275 ×1, F0278 ×1 | 3 /72 | 6 /288 | 3 /216 |
| do-chain | F0526 ×1, F0275 ×1, F0278 ×1 | 3 /72 | 6 /416 | 3 /344 |
| call-repeat | those three plus F0273→F0272 ×1 | 4 /96 | 7 /312 | 3 /216 |
| cleanup | F0526 ×1, F0275 ×1, F0278 ×1 | 3 /72 | 6 /352 | 3 /280 |
| fold | F0526 ×2, F0275 ×2, F0278 ×1 | 5 /120 | 8 /464 | 3 /344 |

The same three/four/five source captures were legacy frames in S2. The ordinary Table/Do/iteration and S13 Function/Fold frames remain legacy with unchanged per-resume call counts. Some top-level scope/finalizer frames execute once per program and therefore have zero call-count slope; they are retained in `candidate/frame-calls.json`, not mistaken for absent coverage. An operation inside such a constant-count call can still scale with its argument size; whole-profile instruction and allocation slopes include that work. Other S3 frame paths are covered by correctness tests, not claimed as independently measured resume workloads.

Callgrind's demangled names merge identical code: the fixed-point resume body is printed as F0527, while the actual suspend caller constructs F0526 via `EvaluateProfiledFixedPointSuspensionFrame::freeze` (candidate resume 0x10aed4). The stage body can likewise share code with F0276. `candidate/semantic-costs.json` splits adapter monomorphizations by ELF entry address and resolves their concrete calls through ELF relocations. This avoids treating the common `TypedAdapter<F>` spelling as one frame type. All addresses below refer to the hashed candidate **resume** ELF unless S2 is specified.

The added outer Arc is 24 bytes: two counters and the inner Arc pointer. Its executed allocation sites include 0x758cdb (F0275, size immediate 0x758cd1), 0x10af8d (F0526), 0x758e6a (F0278, size immediate 0x758e60), and 0x757723 (F0272, size immediate 0x757719). The original inner-frame allocation remains. The Some wire allocates 32 bytes for its one-element Vec (0x751ac9/0x751ad3), 48 for `Arc<Expr>` (0x799e94/0x799e9e) and 40 for `Arc<Vec<Expr>>` (0x79a0f3/0x79a0fd), totaling 120 bytes; the latter two are in the existing `Expr::call` implementation reached at 0x751b12. No core representation changed. After those 120 bytes, the observed residuals are 96/224/96/160/224 bytes respectively. They fit the unchanged queue's 16-byte legacy entry versus 24-byte mixed entry at cumulative capacities 12/28/12/20/28. The executed mixed-queue path is proven (`FrameQueue::widen`, 0xd9820; width 24 at 0xd985a, plus `append_mixed`); those capacity totals are inferred from the measured residual, not independently observed allocation arguments. The report therefore leaves that last byte-level partition explicitly inferred, rather than claiming an exact Arc-only pass.

Per-frame **self instructions**, excluding callees, separate the actual body from the added adapter. These numbers are means of three N/2N pairs (six processes per arm) and are stable across the five workloads except the displayed Fold branch. The complete per-function totals, including allocator, hash and libc costs, sum to every raw Callgrind summary; they are not estimated from these small subtotals.

| Active frame body | S2 entry and self I | Candidate body entry and self I | Added adapter entry and self I | Mode work |
| --- | --- | --- | --- | --- |
| F0526/F0527 | 0x6f0410:82 (Fold164) | 0x6f0ac0:104 (Fold208) | 0x756250:59 (Fold118) | Expr Value input/output, slot check/unpack and result pack |
| F0275/F0276 | 0x579c30:78 (Fold148) | 0x5793e0:102 (Fold196) | 0x756690:74 (Fold148) | Legacy Option input; decoder self22→69 (Fold44→91) |
| F0273→F0272, call-repeat only | 0x571ce0:102 | 0x5724b0:325 | 0x757170:59 | Expr Value input, fresh token suspension; PreparedArguments remains Legacy at final completion |
| F0278 | 0x5845f0:37 | 0x584170:67 | 0x757390:59 | Expr Value input; Legacy Option encode self36, plus symbol/hash/three allocations |

All four active adapters introduce an extra inner-Arc clone and outer-Arc drop around the typed call. For F0526/F0527, the clone is 0x756309, typed call 0x75639a and outer decrement 0x7563be. F0275's corresponding hot clone is 0x75687b (its alternate error-path clone is unexecuted here), typed call 0x756803 and outer decrement 0x756827. The full executed instruction lists for every monomorphization, with counts for every workload, are in `candidate/semantic-costs.json`; the whole object-qualified ledger is `address-ledgers/resume/{base,candidate}-addresses.tsv`. That ledger names each operation and address, including second-Arc allocation/drop, codecs, driver dispatch, allocator and libc instructions. `candidate/paired-frame-differences.json` aligns the four old/new frame bodies despite their trait-name changes. Register/layout differences are retained and are not automatically called new semantic work.

**Unremoved copy defects.** The adapter moves its typed action into the erased `Result<Action, DecodeFailure>` with an additional 136-byte copy: F0526/F0527 at 0x7563b1 (once per resume; twice in Fold), F0275/F0276 at 0x75681a (same counts), F0272 at 0x7572d1 (call-repeat only), and F0278 at 0x7574f1 (once in all five). S2 dispatches directly to the legacy body and has no intervening adapter action copy. The driver's newly exercised typed path also copies its 136-byte slot outcome at 0x72a098 before adapter dispatch and 0x72aa41 before the loop outcome assignment, each 3/3/4/3/5 times per resume. The common loop assignment copy is unchanged in count (S2 0x72a565 → candidate 0x72aa65, 4/5/4/4/7). These are defects under the explicit action/slot/error-move rule, even where other old copies disappear.

There are additional input/result moves in the adapters and in the typed body's completion encoding. For the Expr adapter at 0x756250, the normal-path Value unpack is 0x75626e–0x7562b0; the copied 136-byte result layout passed to the body is 0x756317–0x756382. The Option adapter unpacks and calls its Legacy decoder at 0x7566b4–0x75670f and repacks the result at 0x75677a–0x7567eb. These addressed moves, and their counterparts in the other two specializations, are preserved verbatim in the ledger and are not declared copy-free. Error-only/mismatch branches have zero execution count in these normal resume profiles; correctness tests exercise their behavior, but these measurements do not establish an error-only performance cost.

The physical libc copy target is `/usr/lib/libc.so.6` +0x1b1980, resolved from the named memcpy GOT calls; the driver keeps that same target in r13 (load 0x729fda). `candidate/copy-sites.tsv` records all selected base/candidate call sites, sizes where the preceding immediate proves them, and per-workload counts. This distinction matters: extra copies can be introduced while the overall number of call-site instructions falls. For example, resumed-request self I changes 1123→1098 (table), 1252→1217 (do-chain), 1123→1120 (call-repeat), 1123→1098 (cleanup), 1510→1525 (fold); the libc, adapter, mode and allocator callees still add substantial work.

The comprehensive resume ledger has 18,013 base and 19,480 candidate nonzero address rows and 8,518 changed rows. Every changed address maps to a hashed ELF/shared library. The only four unmapped entries are the same two objectless entries at 0x406d128 and 0x406d160 in both arms, with exactly equal counts and zero delta. Unmatched textual instructions require semantic review; the raw address list is not used to reclassify a code-layout change as a semantic addition. No profile total is dropped, and recursive Callgrind contexts (`'2`, etc.) are merged only when they represent the same physical function.

Resume table: largest physical-function self-cost changes (complete function list in `candidate/callgrind-means.json`; all addresses in the ledger).

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 933 | 1417 | 484 |
| core::hash::BuildHasher::hash_one | 696 | 995 | 299 |
| 0x00000000001b1980 | 1318.95 | 1578.95 | 260 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 284 | 78 | -206 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 192 | 192 |
| rum_core::expr::intern_text | 381.761666667 | 567.973333333 | 186.211666667 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 143 | 143 |
| rum_runtime::runtime::suspension::typed::witness::SlotWitness<I>::suspend | 0 | 140 | 140 |
| mi_theap_malloc_aligned | 543.09 | 666.95 | 123.86 |
| rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::resume_call_after_head | 123 | 0 | -123 |

Resume do-chain: largest physical-function self-cost changes (complete function list in `candidate/callgrind-means.json`; all addresses in the ledger).

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 933 | 1417 | 484 |
| core::hash::BuildHasher::hash_one | 696 | 995 | 299 |
| 0x00000000001b1980 | 1185.775 | 1453.775 | 268 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 334 | 132 | -202 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 192 | 192 |
| rum_core::expr::intern_text | 382.428333333 | 573.943333333 | 191.515 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 143 | 143 |
| rum_runtime::runtime::suspension::typed::witness::SlotWitness<I>::suspend | 0 | 140 | 140 |
| mi_theap_malloc_aligned | 459.34 | 584.78 | 125.44 |
| rum_core::expr::intern_symbol_text | 192.611666667 | 317.16 | 124.548333333 |

Resume call-repeat: largest physical-function self-cost changes (complete function list in `candidate/callgrind-means.json`; all addresses in the ledger).

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 1365 | 1849 | 484 |
| <rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame as rum_runtime::runtime::suspension::typed::TypedFrame>::resume | 0 | 325 | 325 |
| 0x00000000001b1980 | 2081.055 | 2391.055 | 310 |
| core::hash::BuildHasher::hash_one | 995 | 1294 | 299 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 278 | 0 | -278 |
| rum_runtime::runtime::eval_call_prepare::<impl rum_runtime::runtime::model::session::Session>::resume_suspended_call_after_argument | 251 | 0 | -251 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 251 | 251 |
| rum_core::expr::intern_text | 571.453333333 | 763.756666667 | 192.303333333 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 189 | 189 |
| mi_theap_malloc_aligned | 795.4 | 939.51 | 144.11 |

Resume cleanup: largest physical-function self-cost changes (complete function list in `candidate/callgrind-means.json`; all addresses in the ledger).

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 933 | 1417 | 484 |
| core::hash::BuildHasher::hash_one | 696 | 995 | 299 |
| 0x00000000001b1980 | 1031 | 1304 | 273 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 276 | 70 | -206 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 192 | 192 |
| rum_core::expr::intern_text | 380.593333333 | 571.395 | 190.801666667 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 143 | 143 |
| rum_runtime::runtime::suspension::typed::witness::SlotWitness<I>::suspend | 0 | 140 | 140 |
| mi_theap_malloc_aligned | 438.4 | 563.76 | 125.36 |
| rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::resume_call_after_head | 123 | 0 | -123 |

Resume fold: largest physical-function self-cost changes (complete function list in `candidate/callgrind-means.json`; all addresses in the ledger).

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| 0x00000000001b1980 | 1653.775 | 2251.775 | 598 |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 933 | 1417 | 484 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 477 | 127 | -350 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 325 | 325 |
| core::hash::BuildHasher::hash_one | 696 | 995 | 299 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 240 | 240 |
| rum_runtime::runtime::suspension::typed::witness::SlotWitness<I>::suspend | 0 | 218 | 218 |
| <rum_runtime::runtime::resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as rum_runtime::runtime::suspension::typed::TypedFrame>::resume | 0 | 208 | 208 |
| <rum_runtime::runtime::eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as rum_runtime::runtime::suspension::typed::TypedFrame>::resume | 0 | 196 | 196 |
| rum_core::expr::intern_text | 397.62 | 566.571666667 | 168.951666667 |

Timing ran last, after correctness, allocation, all flagged-label repeats, Callgrind and address attribution. The full core-language-microbench.sh panel produced120 matching labels per arm, scale1/reps7, retained-module-gc-batches3 (the script default), profiling disabled, on the two CGU1 CLI binaries. These are in-kernel monotonic AbsoluteTiming measurements, not wall-clock differences. The median candidate/base ratio is1.011360; 78 labels are slower in this single panel pair. Timing is an indication only and does not override instruction evidence.

The host monitor observed five consecutive quiet seconds before each arm and no competing CPU process above0.15 core during either panel. It did not stop or alter other processes. GPU occupancy could not be read from the available sysfs interface, so GPU idleness is not independently verified. Raw activity samples are in timing-{base,candidate}-host.json; exact commands, benchmark .rum input, stdout, stderr and per-label timings.tsv are under r5/perf/timing/. This is one final timing pair, not a repeated statistical timing acceptance.

| Timing label | S2 seconds | S3 seconds | S3 /S2 |
| --- | --- | --- | --- |
| association-part-key-read-width4000 | 0.00214078 | 0.00218708 | 1.02162763105 |
| association-part-key-read-width32000 | 0.00767115 | 0.0080604 | 1.05074206605 |
| association-part-positional-read-width4000 | 3.148e-05 | 3.1226e-05 | 0.991931385006 |
| association-part-positional-read-width32000 | 3.1449e-05 | 3.1223e-05 | 0.992813761964 |
| loop-empty | 0.00027613 | 0.000263089 | 0.952772244957 |
| do-count-null | 0.000247961 | 0.000244279 | 0.985150890664 |
| do-iterator-null | 6.0034e-05 | 5.7836e-05 | 0.963387413799 |
| table-count-null | 7.8341e-05 | 7.6629e-05 | 0.978146819673 |
| table-iterator-null | 1.885e-05 | 1.7836e-05 | 0.946206896552 |
| table-iterator-symbol | 5.718e-06 | 6.845e-06 | 1.19709688702 |
| literal-integer | 9.6042e-05 | 9.2771e-05 | 0.965941983715 |
| literal-string | 0.000106653 | 9.8996e-05 | 0.928206426448 |
| symbol-ownvalue-read | 0.000222512 | 0.000216161 | 0.971457719134 |
| symbol-ownvalue-guarded-read | 0.00018756 | 0.000186781 | 0.995846662401 |
| set-ownvalue | 0.000262758 | 0.000269945 | 1.02735216435 |
| list-literal-small | 5.4143e-05 | 5.5164e-05 | 1.01885747003 |
| head-list | 0.00090934 | 0.000943719 | 1.03780654101 |
| length-list | 0.000908082 | 0.000936623 | 1.03142998099 |
| part-list | 0.00106835 | 0.00110854 | 1.0376187579 |
| sameq-integer | 0.000619522 | 0.000646042 | 1.04280719652 |
| sameq-list | 0.000611026 | 0.000616584 | 1.00909617594 |
| if-true | 0.000698164 | 0.000711459 | 1.0190428037 |
| module-local | 0.00072369 | 0.000732405 | 1.01204244911 |
| module-chain-escape-small | 4.977e-05 | 5.2151e-05 | 1.0478400643 |
| module-chain-escape-large | 9.4543e-05 | 9.6278e-05 | 1.01835143797 |
| block-local | 0.000465933 | 0.000478424 | 1.02680857548 |
| block-symbol-only-repeat | 0.000305476 | 0.000312321 | 1.02240765232 |
| block-init-null-repeat | 0.000353382 | 0.000366218 | 1.03632329887 |
| block-init-read-repeat | 0.000402188 | 0.00041357 | 1.02830019792 |
| with-local | 0.000477322 | 0.000493609 | 1.03412162021 |
| with-deep-body-small | 0.00179236 | 0.00183026 | 1.02114530563 |
| table-small | 0.000187018 | 0.000184349 | 0.98572864644 |
| table-iterator-plus | 3.6074e-05 | 3.6019e-05 | 0.998475356212 |
| table-iterator-pair-list | 6.032e-05 | 6.1297e-05 | 1.0161969496 |
| do-small | 0.000888573 | 0.000899731 | 1.01255721252 |
| d-nested-sin-small | 0.00514376 | 0.0054389 | 1.05737826026 |
| downvalue-exact-hit | 0.000756263 | 0.000798398 | 1.05571474474 |
| downvalue-exact-miss | 0.000545396 | 0.00056069 | 1.02804200984 |
| downvalue-pattern-hit | 0.000947579 | 0.000984883 | 1.03936769388 |
| downvalue-pattern-miss | 0.000281343 | 0.000290189 | 1.03144204761 |
| downvalue-exact-miss-repeat | 0.00124906 | 0.00128965 | 1.03249643732 |
| downvalue-pattern-miss-repeat | 0.00130029 | 0.00132128 | 1.01614255282 |
| downvalue-guarded-hit | 0.00189956 | 0.00193909 | 1.02081008233 |
| downvalue-guarded-decline-fallback | 0.00141527 | 0.00145423 | 1.02752831615 |
| upvalue-guarded-hit | 0.002352 | 0.00236374 | 1.0049914966 |
| subvalue-unguarded-hit | 0.00139749 | 0.00144588 | 1.03462636584 |
| subvalue-unguarded-miss | 0.000761666 | 0.000803043 | 1.05432433639 |
| subvalue-guarded-hit | 0.00244244 | 0.00248902 | 1.01907109284 |
| default-unguarded-hit | 0.00129002 | 0.00130282 | 1.00992232679 |
| default-unguarded-miss | 0.0010781 | 0.00108994 | 1.01098228365 |
| default-guarded-hit | 0.00163552 | 0.0016626 | 1.01655742516 |
| matchq-integer-success | 0.000280541 | 0.000280507 | 0.999878805593 |
| matchq-integer-fail | 0.000284998 | 0.000286948 | 1.00684215328 |
| matcher-single-tiny | 0.000671972 | 0.000661449 | 0.984340121315 |
| matcher-single-wide-small | 3.9298e-05 | 4.007e-05 | 1.01964476564 |
| matcher-single-wide-large | 7.8411e-05 | 7.8306e-05 | 0.998660902169 |
| replaceall-simple | 0.000352397 | 0.000356494 | 1.01162609216 |
| rule-rhs-deep-small | 0.00298096 | 0.00296092 | 0.993277333476 |
| rewrite-guarded-hit | 0.0017984 | 0.00182021 | 1.01212744662 |
| rewrite-guarded-decline-fallback | 0.00255373 | 0.00255925 | 1.00216154409 |
| cases-wide-no-match | 0.000144526 | 0.00014289 | 0.988680237466 |
| count-wide-no-match | 0.00013781 | 0.000136203 | 0.988339017488 |
| position-wide-no-match | 0.000167083 | 0.000171016 | 1.02353919908 |
| deletecases-wide-no-match | 0.000168802 | 0.000168549 | 0.998501202592 |
| deletecases-wide-association-no-match | 0.000428692 | 0.000420323 | 0.980477825572 |
| deletecases-leaf-delete-small | 0.00219006 | 0.00208468 | 0.951882596824 |
| replace-wide-no-match | 7.9898e-05 | 7.7445e-05 | 0.969298355403 |
| replace-leaf-rebuild-small | 0.00266492 | 0.00255839 | 0.960025066419 |
| take-multispec-small | 0.00161359 | 0.0016326 | 1.01178118357 |
| replacepart-deep-path-small | 0.00131649 | 0.00129829 | 0.986175360238 |
| replacepart-wide-rule-list | 0.000298245 | 0.000287693 | 0.964619691864 |
| replacepart-guarded-hit | 0.000534621 | 0.000539929 | 1.00992852881 |
| replacepart-guarded-decline-fallback | 0.00058839 | 0.000590042 | 1.00280766159 |
| string-repeat-small | 0.000185328 | 0.000191459 | 1.03308188725 |
| character-range-small | 0.00013151 | 0.000132969 | 1.01109421337 |
| stringtrim-literal-list-4000 | 0.000656873 | 0.000656661 | 0.99967725877 |
| stringtrim-literal-list-32000 | 0.00505153 | 0.00504486 | 0.99867960796 |
| stringtrim-single-literal-4000 | 0.000637523 | 0.000639042 | 1.00238265914 |
| stringtrim-single-literal-32000 | 0.00503024 | 0.00502337 | 0.998634259996 |
| stringtrim-whitespace-4000 | 0.000625683 | 0.00062901 | 1.00531738916 |
| stringtrim-whitespace-32000 | 0.00502764 | 0.00501469 | 0.997424238808 |
| stringreplace-single-literal-unguarded | 0.000385786 | 0.000386849 | 1.00275541362 |
| stringreplace-pattern-unguarded | 0.0010399 | 0.00105006 | 1.00977017021 |
| stringreplace-guarded-hit | 0.000690806 | 0.000687359 | 0.995010176518 |
| stringreplace-guarded-decline-fallback | 0.000823957 | 0.000824133 | 1.00021360338 |
| appendto-list-incremental-small | 0.000452953 | 0.000448598 | 0.990385315916 |
| appendto-list-incremental-large | 0.00154958 | 0.00165796 | 1.06994153254 |
| appendto-part-outer1000-small | 0.00220682 | 0.00221031 | 1.00158146111 |
| appendto-part-outer1000-large | 0.00444904 | 0.00448206 | 1.00742182583 |
| association-set-key-incremental-small | 0.00144914 | 0.00143219 | 0.988303407538 |
| association-set-key-incremental-large | 0.00546461 | 0.005393 | 0.986895679655 |
| association-small-literal-repeat | 0.000746289 | 0.000741623 | 0.993747730437 |
| merge-nested-small | 0.00116382 | 0.00118751 | 1.02035538142 |
| set-part-fixed100-len100 | 0.000140324 | 0.000136587 | 0.97336877512 |
| set-part-fixed100-len800 | 0.000544591 | 0.000540796 | 0.993031467652 |
| set-part-fill-small | 0.000405942 | 0.000401936 | 0.990131595154 |
| set-part-fill-large | 0.00126752 | 0.00126123 | 0.995037553648 |
| upvalue-candidates-tiny | 0.00194761 | 0.00195917 | 1.0059354799 |
| upvalue-candidates-wide-small | 0.000154845 | 0.000159815 | 1.03209661274 |
| upvalue-candidates-wide-large | 0.000308452 | 0.000319416 | 1.03554523881 |
| sort-callback-small | 0.000164224 | 0.000170378 | 1.03747320733 |
| sort-callback-large | 0.000377674 | 0.000389911 | 1.03240095956 |
| sortby-callback-small | 0.000195363 | 0.000198203 | 1.0145370413 |
| sortby-callback-large | 0.000437389 | 0.000447373 | 1.0228263628 |
| reversesort-callback-small | 0.000159353 | 0.000164543 | 1.03256920171 |
| reversesort-callback-large | 0.000358281 | 0.000369361 | 1.03092544679 |
| keysort-callback-small | 8.4174e-05 | 8.6486e-05 | 1.02746691377 |
| keysort-callback-large | 0.000184986 | 0.000190455 | 1.02956439947 |
| keysortby-callback-small | 4.4635e-05 | 4.6716e-05 | 1.04662260558 |
| keysortby-callback-large | 9.7402e-05 | 0.000102704 | 1.05443420053 |
| sort-default-wide | 1.657e-05 | 2.8533e-05 | 1.72196741098 |
| sort-symbol-default-width4000 | 0.000300786 | 0.000291276 | 0.968382836967 |
| sort-symbol-default-width16000 | 0.00123221 | 0.00125064 | 1.01495686612 |
| sortby-default-wide | 0.000518619 | 0.000516654 | 0.996211091379 |
| reversesort-default-wide | 1.7455e-05 | 2.8438e-05 | 1.62921798911 |
| keysort-default-wide | 0.00037023 | 0.000383941 | 1.03703373579 |
| keysortby-default-wide | 0.000867386 | 0.000902123 | 1.04004791408 |
| valueq-undefined-symbol-definitions-2000 | 7.729e-06 | 7.97e-06 | 1.03118126536 |
| valueq-undefined-symbol-definitions-8000 | 7.877e-06 | 7.894e-06 | 1.00215818205 |
| module-retained-gensym-gc-batch | 6.4223e-05 | 6.2874e-05 | 0.978995064074 |

Performance acceptance is withheld. Exact ordinary allocations, passing native-stack slopes and unchanged fault guards do not cancel the measured ordinary instruction regressions, resume allocation excesses, or extra typed action/slot/result copies. No production performance repair was attempted after these measurements; the final code, correctness gates and measured binaries remain the same candidate.

### 6. S2 cold-helper retype and hot boundary

continuation_fault_outcome consumes the outcome and returns EvalError directly, retaining cold/inline-never. The four production callers wrap Err explicitly; only the two specified test callers lose unwrap_err. No driver protocol or type mode changed for this cleanup.

All four boundaries retain one fault-word comparison and one cold conditional branch. The addressed fast successor sequences are equal after normalizing only stack-slot offsets and relocations; registers, immediates, operations and called symbols are retained. The source retype introduces no extra hot instruction at these boundaries. This is a boundary claim, not a claim that the entire resumed evaluator has unchanged cost.

| Boundary / binary | S2 guard → cold call | Candidate guard → cold call | Hot successor instructions |
| --- | --- | --- | --- |
| eval_expr_request_from_checkpoint | 0x728f4e, 0x728f56 → 0x72942f | 0x72948e, 0x729496 → 0x72996c | 11 |
| resume_eval_request_from_checkpoint | 0x72bb28, 0x72bb30 → 0x72c2d6 | 0x72bff9, 0x72c001 → 0x72c782 | 24 |
| eval_input_with_terminated_handling | 0x750120, 0x750128 → 0x75066c | 0x751660, 0x751668 → 0x751bac | 23 |
| resume_inspect_rewrite_rules_with_input (inlined test thunk) | 0x1cc629, 0x1cc632 → 0x1cd557 | 0x1ccaf0, 0x1ccaf9 → 0x1cda17 | 11 |

The first two rows use the immutable CGU1 resume binaries; top-level text evaluation uses the immutable CGU1 CLI binaries. The generic rewrite-inspection method is absent from both production measurement binaries and is inlined in the existing release runtime-test thunk (`FnOnce::call_once`, S2 0x1cc2c0, candidate 0x1cc790). Its identity is established by the ordered resume call and `Option<Vec<RewriteRuleInfo>>::from_continuation_expr` call; no new fixture or binary was built. Those two release test binary hashes and complete addressed bodies are recorded in `r5/cold-boundaries/native-inspection.json`. The initial extraction failed because it expected a standalone named method; the corrected extraction follows the actual inlined thunk.

No cold-helper instruction executes in the three N/2N Callgrind pairs for any of the five normal resume workloads. These profiles do not exercise a continuation fault. The final typed fault tests and complete oracles provide the separate fault/absence checks. Exact guard, fall-through and cold-call instructions: `r5/cold-boundaries/comparison.json`; whole containing functions and ELF-resolved helper targets: `production.json` and `native-inspection.json` in that directory.

Both top-level success branches (ConvertToValue and ConvertCyclesToValue) were compared, not only the first fall-through. No cold-helper instruction executes in any of the35 ordinary labels’ three Callgrind pairs either.

### 7. Stop-loss counters, deviations and open questions

R5 stop-loss counter is0. The directed structural loop repair moved all four stack gates below S2. The required identifier/expression comparison also passes, so the simpler single expression arm is retained. Intermediate census failures advanced through three stale macro identities, then T0025/T0026, then the X0041 reference quotation; exact discovery replays resolved each. Ownership acceptance identified four stale impl-line citations, and replaying only those verified cells made it pass. No oracle, retention rejection, fault-absence assertion, mode or documented contract was weakened.

The amendment’s expected test-only12/combined601 macro census is contradicted by the unchanged scanner’s actual classification: the A fixture is TestSupport, whose count8→9. Test-only stays11 and combined stays600. The report preserves this discovery rather than changing the classifier or forcing a pin. The active production counts remain607/589 with12/13 typed.

The two full suspension-oracle builds were run as the script’s underlying Cargo commands, each under the required1500-second bound and brief609 TMPDIR, rather than invoking check-suspension-oracle.sh literally. Their raw commands and all58 tests/fault-occurrence deltas are retained. The workspace command used captured output; a focused rerun of the nine existing native tests with --nocapture was necessary to verify and report the native environment skips. It adds no test or implementation change.

Minor scratch-tool corrections: an initial workspace cargo fmt without --all found no targets and was corrected before promotion; an initial frontier-report assertion expected14 moved pairs, whereas exact discovery proves15, and the scratch reporting assertion was corrected after every old row was paired. These are not weakened production gates. There were no R5 Rust compiler errors; all12 native-build warning occurrences and the three workspace warning kinds are inherited from S2. Earlier R1–R4 failures and stops remain historical evidence in their own reports and are not erased.

All edits remain within the authorized owned scope. No git command, network operation, unsafe code, dependency/lockfile/toolchain change, mode flip, per-frame override, raw public adapter, other-family conversion or unrelated documentation edit occurred. The director-owned security-hardening document is preserved. C01/C02 bodies and the entire local analyzer remain byte-identical. The tree is left uncommitted for the director.

The cost gates are not accepted. The measured ordinary-path instruction increases remain defects, including the exact112→113 integer leaf path. All five resume panels exceed the extra-Arc-only count/byte expectation. Extra typed adapter action copies and driver slot copies are left with concrete addresses in section5; no production repair was attempted after measuring them. The brief explicitly permits reporting a regression, and explicitly requires reporting unremoved action/slot/error copies. This report uses that route rather than claiming those costs are approved.

Attribution limits are explicit. The 120-byte Some wrapper and 24-byte adapter Arc are supported by the source, executed call edges and allocation-size immediates. Residual queue-byte totals are consistent with the active16→24-byte queue representation, but their cumulative capacity totals are inferred from the measured residual, not independently sampled allocation arguments. An optional Memcheck allocation-tree probe failed before the program ran because the system loader lacked the mandatory memcmp redirection symbol. A Massif allocation-tree probe ran and the target passed, but it observed only7 system allocations against8141 counted Rust allocations because mimalloc blocks were not tracked; that tree was discarded as allocation evidence. No tool installation, network operation or new allocator instrumentation was attempted.

Callgrind's recursive-context suffixes were initially treated as separate function labels; the final parser merges only physical functions, preserving the original literal-context diff as historical evidence. The first frame-count filter accidentally matched the resume_frames namespace and included freeze/drop methods; its anchored method filter was corrected and the unfiltered output preserved. The full address ledger includes allocator and libc, correcting the initial narrower runtime/drop-only ledger. Its only unmapped resume rows are equal-count objectless entries; it does not treat unmatched text alone as semantic added work. The initial fourth cold-boundary extraction expected a named method and failed; the corrected extraction identifies the actual inlined release-test thunk by its resume/decode call sequence. These are scratch-evidence corrections, not changes to a production gate or runtime behavior.

Open decisions for the director are the ordinary code-generation regressions, removal or acceptance of the extra typed action/slot/result moves, and the conflict between the extra-Arc-only allocation requirement and the current global Legacy wire/mixed-queue costs. No other-family conversion, mode flip, public adapter or weakened comparison is proposed by this report. The final performance measurements do not establish error-only performance costs or independently measure every S3 frame: the inherited five-workload harness exposes the four active frame shapes described in section5. Correctness coverage remains the complete final tests and oracles.

The two dispatch resumers now check `debug_assert!(unevaluated_restore.is_none())` immediately after the preparation producer and before the typed macro, rather than only inside its success arm. This internal debug invariant is therefore also checked when that producer suspends or errors. Release emits no assertion; the unchanged debug tests and both measured debug variants pass.

The final timing pair completed all120 labels per arm with no observed competing CPU process under the declared monitor threshold. GPU activity was unavailable through the permitted sysfs interface. The full timings, including the larger sort/reverse-sort ratios, are reported as indications; no timing result overrides the instruction regressions.


## Director amendment — redispatch 6: the new copies go, the Legacy wire stays

The loop did its job, and this report stops the right way: every regression
is attributed to an address and none is hidden. The director reproduced the
instruction results from your hashed binaries, 3 runs per arm on seven labels:

| Label | Delta |
| --- | --- |
| `lazy-depth-map-first-64` | +1.24% |
| `lazy-depth-select-restart-64` | +2.03% |
| `literal-integer` | +0.31% |
| `dot-three-no-input` | +0.14% |
| `resume-table` | +19.4% |
| `resume-fold` | +16.4% |

### Accepted as reported

- **Stack.** The loop in both resumers, the expression-only macro, all four
  limits below S2, and the prologue table.
- **Macro census.** The A fixture is TestSupport (8 → 9). The amendment's
  12/601 estimate was wrong; discovery wins.
- **The +3 allocations per resume.** In all five workloads they are the
  strict Legacy encoding of F0278's `Some(value)` (120 B at the three named
  sites).
  - S2 produced no wire because the lenient decoder (`other => Some(other)`)
    invented the `Some`, which is the defect class G13 exists to remove.
  - While `Option<Expr>` is in Legacy mode, this encoding is mode work,
    including its symbol interning and hashing. That is the brief's "decode
    through its mode".
  - It is owed to `Option<Expr>`'s flip, whose gate must show it gone. The
    director records this in `TODO.md`.
- **The queue-byte residual** (8 B × capacity, from 16 B to 24 B entries) is
  S2's mixed-queue design. Your figure is accepted as an inference, and the
  report correctly labels it as one.
- The placement of the `debug_assert!`, and running the oracle as its
  underlying Cargo commands.

### Ruling: remove the new aggregate copies on both paths

S2 closed with zero non-inherent residual on both paths. S3 is held to the
same bar.

**1. Ordinary path.**

What the report shows:

- Every new copy site is 136 B or 168 B.
- Each sits at a typed macro site: `resume_call_with_dispatch_plan`,
  `prepare_arguments_for_call_with_unevaluated_restore`, and `evaluate_call`
  as inlined into `evaluate_inner_unstable`.
- Each executes once per pass through its site.

The director's reading of the source is a **hypothesis**. Confirm or refute
it at the address level before fixing anything:

- Each of those call sites binds the producer's result to a local, borrows
  it for the profile record, and then passes the local to the macro. The
  macro's expansion begins `let produced = $result;`.
- A move of a 136-byte local whose address has escaped is not elided, so
  every pass pays a `memcpy`.
- R5 dropped R4's identifier arm on stack grounds alone. Nobody measured
  whether that arm avoided this copy.

Fix by shape. Two candidates:

- a macro form that matches the caller's place without re-binding it;
- call sites that pass the producer expression straight to the macro and
  record the profile outcome inside the arms. The profile records must stay
  balanced on every path.

If a second macro form comes back, every R5 analyzer obligation applies to
it:

- the exact definition in the adapter;
- the drift falsifier, re-watched;
- the expansion rejection list, re-derived;
- all four views reading it;
- an A3 fixture for it.

The `literal-integer` +1 is part of this item. That is 112 → 113 self
instructions in `Session::evaluate`, with an added `lea 0x120(%rsp),%r14`.
Find which S3 source change put it there.

Acceptance is the bar S2 met:

- On all 59 ordinary labels, the per-label `memcpy` call count is at most
  S2's.
- Every ordinary label flagged by the two-pair panel is cleared by one of:
  - five runs per arm;
  - three or more callgrind processes per arm, where the mean difference
    does not exceed the larger arm range and the address ledger shows no
    operation the base does not execute.
- The deterministic leaf paths (`literal-integer`, `do-count-null`,
  `do-iterator-null`) execute no more instructions per iteration than S2.
- Allocations and bytes stay exactly equal.

**2. Resume path.**

Three copies are ones the legacy arm does not make. Under the brief's rule
they are defects, so remove them:

- the adapter's 136 B action copy into `Result<Action, DecodeFailure>`, in
  all four `TypedAdapter<F>::resume` monomorphizations;
- in `drive_resuming_frames`'s typed arm, the slot-outcome copy before the
  adapter call;
- in the same arm, the slot-outcome copy before the loop's outcome
  assignment.

`suspension/` is in scope under the "gap in S2's driver" class. Change the
erased signature if that is what it takes. The API stays sealed and opaque,
and nothing new becomes public.

Acceptance:

- After the fix, each resume workload's added instructions are fully
  partitioned into the inherent items:
  - decode and encode through the mode, including the Legacy
    `Option<Expr>` wire;
  - adapter dispatch;
  - the second `Arc`: allocation, clone and drop;
  - the mixed-queue entry.
- Each item is named at its addresses, with its per-resume count. Anything
  outside these items is removed or reported.
- Allocations are exactly the typed frames plus 3, and the 3 are at the
  `Some` wire's three sites.

**3. Keep what R5 earned.** After the changes, re-measure all four
native-stack limits. Each stays at or below S2's, and the 48-level test
stays green.

**4. Then section C in full, as in R5:**

- every gate, verbatim, in both builds;
- the oracle in both builds;
- census and compile controls;
- ownership, certificate and fingerprint replays wherever the changes
  expire them;
- the full instruction panel, with repeats and callgrind for flagged labels;
- resume attribution;
- timing, last.

The stop-loss rule is unchanged. Its named gates are:

- the per-label `memcpy` counts;
- the set of flagged ordinary labels;
- the leaf instruction counts;
- the resume copy sites.

Two attempts that move none of them is a stop.

Append a new `## Execution report` after this section.

## Execution report — redispatch 6: caller-place macro and erased-driver copy repair

The requested ordinary copy repair, deterministic-leaf repair and typed adapter/driver copy repair are implemented and measured. All 59 ordinary labels have exactly S2's allocation counts, bytes and memcpy counts; none is flagged by the final two-pair instruction panel. All three leaf paths execute 112 → 110 self instructions in Session::evaluate. All four native-stack slopes remain below S2, and the unchanged 48-level test passes. Final correctness, both full oracles, census, compile controls, ownership, certificate and fingerprint gates pass.

Full performance acceptance is withheld. The five resume slopes have exactly one extra allocation per typed frame plus the three accepted Some-wire allocations, but raw whole-program profiles expose an additional 56-byte FinalizerAdapter rebox and its payload moves once per call-repeat program. Those costs cancel from N/2N slopes and are not elided. Extra freeze-helper ABI work and a Session spill/reload are also reported at their addresses without claiming them inherent. Complete ledgers retain all other signed residuals. This report uses the brief's explicit “remove or report” route for the remaining work; it does not claim zero non-inherent residual or approval to land.

The final workspace run reports 3,462 harness successes, of which nine native ZeroMQ tests returned early under the documented environment skip: 3,453 exercised successes, zero failures and 59 ignored tests. Both full suspension-oracle builds pass 58 tests with every fault-occurrence delta zero. No new compiler warning is introduced. No git command was run; the tree remains uncommitted for the director.

All paths abbreviated `r6/` or `r6b/` below are under `.temp/brief-609/`. `r6/` preserves the first six runtime trials and their complete preliminary gates/panel. The full address audit then exposed remaining inlined driver copies; trials 7 and 8 are also preserved there. `r6b/` is the final source validation and measurement after trial 8, within this same redispatch. No older report or raw measurement was overwritten.

### 1. Files changed, ripple and pin disclosures

20 existing source/test/inventory files changed from redispatch-6 entry, plus this appended report. The following table covers the cumulative S3 patch. All entry bytes of this brief, including the redispatch-6 amendment, are preserved. The director-owned security progress document was preserved separately.

| File | Redispatch | Semantic change / pin transition |
| --- | --- | --- |
| crates/rum-runtime/src/runtime/builtin_adapters.rs | Inherited R1–R5 | M0205 legacy macro → witnessed typed macro; identity success tail retained. |
| crates/rum-runtime/src/runtime/builtin_adapters/resume_frames.rs | Inherited R1–R5 | F0204 legacy frame → TypedFrame<Expr, Expr>. |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | Inherited R1–R5 | Typed prepared-argument result; actual producer witness precedes semantic work. F0272 fresh cursor/splice state uses its sole witnessed token route on repeats; no captured Arc. Successful macro arms escape directly and construction uses cold freeze; ordinary result/effect order preserved. |
| crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs | Inherited R1–R5 | Merge F0273 into F0272 nominal typed frame; freeze fresh cursor/prefix/restore state cold. Typed FinishPreparedCallArgumentsFinalizer executes after both originating owners drop. R5 cold factory moves impl citation13→40 and finalizer43→70. |
| crates/rum-runtime/src/runtime/eval_core/call.rs | R6 | R6 removes the record_head aggregate closure; both head branches consume all three profile records directly, in their original order. Looped resumers and their captured indices stay unchanged. |
| crates/rum-runtime/src/runtime/eval_core/mod.rs | Inherited R1–R5 | The SharedExpr-result profiling wrapper loses its final caller under T0024 and is removed; generic outcome classification remains. |
| crates/rum-runtime/src/runtime/eval_core/symbol.rs | Inherited R1–R5 | Only F0277/M0285 typed: CompiledMatchOutcome → Expr, retaining matched/unmatched semantics and deleting the Expr wire map. The S14 loader capture in this file is unchanged. |
| crates/rum-runtime/src/runtime/eval_dispatch/mod.rs | Inherited R1–R5 | Typed Option/Expr completion and F0278 semantic Some on resumed builtin result with original growth checks. Always-inlined finish_builtin_dispatch shares checked/unchecked M0287/M0288 completion and owns the non-Copy timing token once; M0288 removed→M0287 receipt. |
| crates/rum-runtime/src/runtime/hold_thread.rs | Inherited R1–R5 | F0279/M0289 typed Expr identity continuation for ReleaseHold. |
| crates/rum-runtime/src/runtime/resolution.rs | R6 | R6 changes M0561 success to Ok(value)/? and outlines only cycle-error formatting into a cold, inline-never helper; measured leaf setup drops below S2. Profiling and error behavior are preserved. |
| crates/rum-runtime/src/runtime/resolution/resume_frames.rs | Inherited R1–R5 | Both fixed-point frames implement TypedFrame<Input=Expr,Output=Expr>; freeze current/pass/history cold and resume exact fixed-point logic. |
| crates/rum-runtime/src/runtime/session/session_eval.rs | Inherited R1–R5 | Two callers: Err(self.continuation_fault_outcome(...).unwrap_err()) → Err(self.continuation_fault_outcome(...)). |
| crates/rum-runtime/src/runtime/session/session_requests.rs | Inherited R1–R5 | Two callers: remove unwrap_err from the cold latch projection, retaining the same hot discriminant check. |
| crates/rum-runtime/src/runtime/suspension/core.rs | R6 | R6 assigns outcomes inside each dispatch arm, borrows the typed adapter and passes the outcome by mutable reference. The cloned outer Arc drops immediately after the adapter call. Decode failures return before queue pop; both Arcs drop before finalization. Final trial8 ends the resource-free Abort placeholder lifetime before extracting Action::Complete, with a debug assertion of the sole private adapter postcondition. This removes the extra action-to-temporary completion copy; ordinary assignment remains. |
| crates/rum-runtime/src/runtime/suspension/core/fault.rs | Inherited R1–R5 | Cold continuation_fault_outcome: Result<T, EvalError> → EvalError; still consumes the outcome and takes the same latch. |
| crates/rum-runtime/src/runtime/suspension/typed/action.rs | R6 | R6 adds a private structural DecodeFailure action so the erased adapter returns Action directly instead of Result<Action, DecodeFailure>; no public action constructor or raw adapter is exposed. |
| crates/rum-runtime/src/runtime/suspension/typed/adapter.rs | R6 | R6 changes only the erased private resume signature to &self and &mut Result<Slot, EvalError>. It takes the input with an Abort placeholder, decodes through the registered mode, and returns the concrete action directly; witness/token rules remain intact. Its sealed private postcondition explicitly states that Abort remains in the consumed input place. |
| crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs | Inherited R1–R5 | Redispatch 2: delete suspend_witnessed; original suspend(session, request, fresh) → suspend(session, SlotWitness<F::Input>, request, fresh). Same public identity, same fresh F constraint, no Arc in the token. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs | Inherited R1–R5 | The legacy-queue setup calls eval_input_string directly instead of evaluating an InputString call through newly typed S3 frames. This restores the legacy-to-mixed widening precondition; every allocation, byte and admission assertion is unchanged. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/composition.rs | R6 | R6 adapts existing private adapter calls to the new mutable input signature; assertions unchanged. Final trial8 adds placeholder assertions; faults also checks forwarding an owned String error payload while the input becomes Abort. Existing assertions remain. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/driver_faults.rs | Inherited R1–R5 | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/faults.rs | R6 | R6 adapts existing private adapter calls and structural-fault pattern to the private direct Action return; failure payload and ownership assertions unchanged. Final trial8 adds placeholder assertions; faults also checks forwarding an owned String error payload while the input becomes Abort. Existing assertions remain. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/file_faults.rs | Inherited R1–R5 | Only the cold-helper test caller loses unwrap_err. |
| crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs | Inherited R1–R5 | repeat_work now captures slot_witness(&outcome) from its actual input producer before matching and passes it to the sole suspend route. |
| crates/rum-runtime/src/runtime/suspension/typed/witness.rs | R6 | R6 adds a caller-place identifier arm ahead of the retained expression arm. The identifier form borrows &($result) for the actual-result witness and matches the caller place without rebinding it; both cold helper capture paths remain accounted. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | R6 | R6 replays the exact typed resume/order pin to the borrowed signature, explicit outer drop and structural-fault return; finalizer assignment pin follows the new shape. Decode-before-pop and drop-before-finalizer order checks remain. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs | Inherited R1–R5 | In both derived-root and pure-abandon arrays: eval_call_prepare/resume_frames.rs:ResumeSuspendedCallArgumentPreparationSuspensionFrame → eval_core/call.rs:PreparedDispatchFrame. This pairs the D0001 merge with the T0025/T0026 split consumer. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv | R6 | R6 preserves79 approved identities/dispositions and replays seven action locations plus four exact adapter witness citations: three36→32, one82→74. Final private adapter documentation moves those same witnesses32→33 and74→75. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv | R6 | R6 same79 identities, seven action locations move by one line. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv | Inherited R1–R5 | Exact discovery replay of source positions/owned S3 constructs; all 79 host identities and dispositions retained. Full old/new rows in ownership replay patch. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv | R6 | R6 all1514 same unresolved attributes; five source locations move. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv | R6 | R6 2956→2954: owned record_head and typed map_err closures removed; remaining changed locations follow their same operations. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv | R6 | R6 3152→3153: one owned unreachable! added to the exhaustive typed Action match; other changes are spans of the same macros. Final macros3153→3154 for the debug assertion; no unknown is admitted. |
| crates/rum-stdlib/src/tests/retention_closure/census/tests.rs | R6 | R6 frontier936→937, RET002900→901 after pairing every old changed row. One explicit Try/FromResidual rejection is added; all freeze and unresolved-boundary assertions remain. |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs | R6 | R6 retains the compiled identifier A3 fixture and adds a compiled expression A3 fixture, each read by all four views. Seven independent definition mutations are rejected by all four views (28 rejections). |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs | Inherited R1–R5 | Context pins172→170 and preserved-clone total174→172: exactly two M0561 Err constructors moved into the checked expansion. R5 preserves all170 contexts and updates the moved-Ok explanatory comment; no instrument/filter change. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | R6 | R6 replays all68 context fingerprints after both source revisions; final context fnv1a64:928be82d795f65ef. No contract field changes. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | R6 | R6 replays only four independently emitted C01/C02 certificate cells, in both feature builds; all other columns and certified bodies unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/methods/context.rs | Inherited R1–R5 | Read typed invocation arguments through checked; bind a success pattern only from an already inferred Result payload, otherwise unknown, and restore locals. Conditional-attribute pre-scan uses the same adapter. Legacy macros remain opaque. |
| crates/rum-stdlib/src/tests/retention_closure/mod.rs | Inherited R1–R5 | Declare one ordinary sibling typed_resume_macro module; no visibility widening. |
| crates/rum-stdlib/src/tests/retention_closure/regions.rs | R6 | R6 asks the checked parsed arm for its rejection list, retaining all five expansion obligations for each macro form. |
| crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs | R6 | R6 exactly checks both definitions; only a bare identifier selects the caller-place rejection wording. The expression binding and both cold-helper captures remain rejected and strict token equality remains enforced. |
| crates/rum-stdlib/src/tests/retention_closure/types/context.rs | Inherited R1–R5 | Read expression/statement typed invocations through checked; propagate only established expected types and preserve success-only scope/reference binding shape. Legacy macros remain opaque. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs | Inherited R1–R5 | New strict If-held and listable-completion cases for three marker shapes, plus prepared.twice/head.map.twice controls. Checks::ALL and unconditional FaultProbe checks retained. Fold/Nest definitions archived for S13. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | Inherited R1–R5 | No module → ordinary sibling mod evaluation_dispatch; no ../ path attribute. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | Inherited R1–R5 | S2 → S3 receipt, used by marker fixtures and census. FLIP_SLOT becomes census-only. Both oracle targets compile without warnings. |
| crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv | Inherited R1–R5 | Record all owned S3 transitions. R5 shared-stage M0282S splits into direct M0282S/M0283S; builtin M0288 merges→M0287. T0025/T0026 direct install_prepared_call_arguments anchors replace map anchors. Add X0041 for legacy-opacity AST path quotation; X40→41. |
| crates/rum-stdlib/tests/suspension_oracle/support/drive.rs | Inherited R1–R5 | Retain the exact execute wrapper required by nine historical integration families through path aliases. Add a function-local allow(dead_code) for the oracle target, which calls execute_planned directly. This corrects the earlier erroneous removal; execute_planned and all comparisons remain unchanged. |
| crates/rum-stdlib/tests/typed_slot_census.rs | Inherited R1–R5 | Scratch brief-608 → brief-609; S2 → S3 receipt and negative-preflip labels. Scope FLIP_SLOT here; connect s3 module. Guard-removal legacy counts 564/59 → 553/58; removed-frame sentinel remains 622. Both scratch falsifiers verified in both builds. |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | Inherited R1–R5 | Replay S3 receipts and retained removals: F624; M602 total =600 active+2 removed (M0280 and M0288). Production/test-only active pins unchanged; discovered A fixture is TestSupport. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | Inherited R1–R5 | Scratch brief-608 → brief-609. Removed Whole0029–Whole0032 emit explicit S3 transition records, assert absence of the old identity, and preserve decoder IDs; live decoder rows still require rediscovery. |
| crates/rum-stdlib/tests/typed_slot_census/routes.rs | Inherited R1–R5 | I0003 still names ContinueToken::suspend and its executable repeat witness. Update mechanism output to disclose typed S3 evaluation while later families remain untyped; S2 mechanism ownership is unchanged. |
| crates/rum-stdlib/tests/typed_slot_census/s3.rs | Inherited R1–R5 | S3 owned F/M/T/D reconciliation and parsed T0023 no-definition/no-caller checks. R5 expected typed macro set replaces M0288 with M0283S; closed removal M0288→M0287. |
| crates/rum-stdlib/tests/typed_slot_compile_controls.rs | Inherited R1–R5 | Scratch directory brief609; actual producer witnesses added to repeat/finalizer controls. Add token-witness-input, changing only producer Expr → Option<Expr>, requiring one E0308 at the complete witness span. 31 positive/intended-negative pairs pass in each build. |

The S1 public witness API prefix is unchanged; both checked macro arms change only the macro. The erased adapter/action changes are private. The sole ErasedFrame implementation is TypedAdapter<F>; it replaces its input with resource-free Abort before every branch. The driver ends only that empty placeholder lifetime with safe mem::forget before installing the next action. The debug assertion and success/mismatch/Abort/owned-error checks enforce this private postcondition. No mode, registry, shape, certified C01/C02 body, local analyzer, compiler instrument Python, dependency, lockfile, toolchain or core representation changed. All 183 protected inputs and four immutable S2 binaries pass the scope audit; no unsafe was added. Exact cumulative patch and hashes: `r6b/candidate.patch` and `r6b/final-audit.json`.

| R6 touched Rust file | Entry characters | Final characters |
| --- | --- | --- |
| crates/rum-runtime/src/runtime/eval_core/call.rs | 21389 | 21676 |
| crates/rum-runtime/src/runtime/resolution.rs | 17524 | 17590 |
| crates/rum-runtime/src/runtime/suspension/core.rs | 29221 | 29879 |
| crates/rum-runtime/src/runtime/suspension/typed/action.rs | 2327 | 2376 |
| crates/rum-runtime/src/runtime/suspension/typed/adapter.rs | 2704 | 2724 |
| crates/rum-runtime/src/runtime/suspension/typed/tests/composition.rs | 1808 | 1852 |
| crates/rum-runtime/src/runtime/suspension/typed/tests/faults.rs | 2016 | 2636 |
| crates/rum-runtime/src/runtime/suspension/typed/witness.rs | 1726 | 2401 |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | 35051 | 35111 |
| crates/rum-stdlib/src/tests/retention_closure/census/tests.rs | 9685 | 9911 |
| crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs | 13061 | 16804 |
| crates/rum-stdlib/src/tests/retention_closure/regions.rs | 12657 | 12643 |
| crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs | 7480 | 9100 |

The three new cumulative S3 modules remain under 10k characters. Existing larger modules keep their coherent evaluation/dispatch, argument preparation, driver and compiler-analysis responsibilities; this repair adds no arbitrary split. The size table exposes every touched Rust file in R6 and its change. frame_root_trace.rs already exceeds 35k characters at entry; its 60-character pin adjustment retains that existing architecture debt under the brief's pin-only scope rather than broadening this repair.

### 2. Owned-row reconciliation, semantic maps and census

| F row | S2 name | Final S3 transition |
| --- | --- | --- |
| F0204 | BuiltinEvaluateSuspensionFrame | Typed Expr → Expr; same name. |
| F0272 | PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame | Typed Expr → PreparedCallArguments; also receives fresh repeated state formerly handled by F0273. |
| F0273 | ResumeSuspendedCallArgumentPreparationSuspensionFrame | Merged into F0272; no remaining production implementation of the old name. |
| F0274 | EvaluateCallSuspensionFrame | Typed Expr → Expr; head conversion at the shared entry. |
| F0275 | ResumeCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch split to PreparedDispatchFrame. |
| F0276 | ResumeSuspendedCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch uses the same PreparedDispatchFrame. |
| F0277 | EvaluateStoredOwnValueSuspensionFrame | Typed CompiledMatchOutcome → Expr. |
| F0278 | RunCallDispatchStageSuspensionFrame | Typed Expr → Option<Expr>; complete the semantic Some map. |
| F0279 | EvalReleaseHoldSuspensionFrame | Typed Expr → Expr. |
| F0526 | EvaluateProfiledFixedPointSuspensionFrame | Typed Expr → Expr. |
| F0527 | ResumeFixedPointFromSuspensionFrame | Typed Expr → Expr. |
| F0570 | FinishPreparedCallArgumentsFinalizer | SlotFinalizer Output=PreparedCallArguments. |

| M row | Actual producer | Ordinary transformation | Resumed transformation |
| --- | --- | --- | --- |
| M0205 | Expr | builtin_evaluate returns the evaluated value unchanged | Typed identity frame; no post-witness semantic map. |
| M0279 | Expr | append/splice/drop-Nothing; preserve Unevaluated entries; advance remaining arguments; build PreparedCallArguments | Typed F0272 reentry uses the same append helper and fresh cursor/splice fields; pure finalizer builds the exact output after Arc release. |
| M0280 | Expr | the same append and cursor/restore work for later arguments | Old macro removed by D0001; witness of actual evaluated_result is passed to token.suspend with fresh F0272. No original Arc is captured. |
| M0281 | Expr | SharedExpr conversion, plan construction and dispatch; profiling records | Witness now at evaluate(head.into_owned()); shared resume_call_after_head accepts Expr or existing SharedExpr and performs Into<SharedExpr> once. |
| M0282 | PreparedCallArguments or Option<Expr> (split) | normalize prepared args, install restore; stage Some returns value, None advances cursor | Prepare macro → PreparedDispatchFrame; direct stage macro M0282S → F0275. Shared normalization/restore runs once; None advances the loop; resumed frame enters k+1. |
| M0283 | PreparedCallArguments or Option<Expr> (split) | same work with the suspended plan and frozen restore | Prepare macro → PreparedDispatchFrame; direct stage macro M0283S → F0276. Restore freezes after the common installer; None advances the loop; resumed frame enters k+1. |
| M0285 | CompiledMatchOutcome | matched branch evaluates own value; unmatched branch resumes symbol lookup | Same branches in F0277; only the Expr transport map and raw input decode disappear. |
| M0286 | Expr | Some(value) after evaluating the threaded call | F0278 now completes resumed.map(Some); three listable fixtures watched fail → pass. |
| M0287 | Expr | Shared finish_builtin_dispatch: profile, original checked/unchecked growth behavior, semantic Some. | F0278 runs the same growth check and semantic Some; M0288 merged into M0287. |
| M0288 | Expr | Shared finish_builtin_dispatch: profile, original checked/unchecked growth behavior, semantic Some. | F0278 runs the same growth check and semantic Some; M0288 merged into M0287. |
| M0289 | Expr | ReleaseHold evaluation result unchanged | Typed identity frame. |
| M0561 | Expr | fixed-point equality/cycle history/trace update and next pass | Typed F0526 uses outcome? then the unchanged resume_fixed_point_from state transition. |
| M0562 | Expr | same fixed-point transition in resumed evaluation | Typed F0527 uses outcome? then the same helper. |

F0273 merges into F0272; repeated suspension uses fresh state of that nominal frame and its witnessed token. F0275P PreparedDispatchFrame consumes PreparedCallArguments, while the two stage consumers take Option<Expr>. M0280 is removed into the token route. Looped resumers retain captured k+1 and normalization/restore order. M0282S/M0283S are their direct stage macros; M0288 remains removed→M0287 in finish_builtin_dispatch. There are 13 production typed macro sites and 12 typed implementations: 11 TypedFrame and one SlotFinalizer.

T0023 removes the unreachable generic preparation arm and wrapper, retaining the parsed no-definition/no-caller gate. T0024 witnesses the actual Expr result before SharedExpr construction. T0025/T0026 install normalization and restore once. T0027 removes transport conversion while preserving semantic Some and head conversion. D0001 retains fresh cursor/splice state and finalization after both originating Arcs drop. M0561 now returns Ok(value) from its success arm and uses the explicit trailing ?, adding the accounted Try rejection. Its cycle-error formatting alone moves to the cold inline-never helper. Global modes remain one Expr Value plus 21 Legacy; no State birth or dual decode.

Final census: default

```text
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=622 macros=600
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.29s
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S3 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=623 macros=600
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.33s
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.09s
```

Final census: no-default

```text
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=622 macros=600
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.42s
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S3 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=623 macros=600
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.56s
S3 CENSUS production frames=542 finalizers=53 implementations=607 typed=12 macros=589 result=353 insert=236
S3 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S3 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S3 PARTITION implementations=623 macros=600
S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=13
S3 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 140.94s
```

Discovery remains authoritative: production607 implementations/589 macros; test-only16/11; combined623/600. TestSupport macros9→10 because both identifier and expression A3 fixtures are compiled. Neither fixture is silently classified as TestOnly and no 12/601 estimate is forced. Active M receipts remain600; whole-slot Legacy types23 and all later-family receipts remain unchanged.

| Receipt | S2 status → final | Final identity and anchors |
| --- | --- | --- |
| F0204 | untyped → typed | rum_runtime::runtime::builtin_adapters::resume_frames::BuiltinEvaluateSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0272 | untyped → typed | rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0273 | untyped → removed | rum_runtime::runtime::eval_call_prepare::resume_frames::ResumeSuspendedCallArgumentPreparationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame; F0272 |
| F0274 | untyped → typed | rum_runtime::runtime::eval_core::call::EvaluateCallSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0275 | untyped → typed | rum_runtime::runtime::eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0276 | untyped → typed | rum_runtime::runtime::eval_core::call::ResumeSuspendedCallWithDispatchPlanSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0277 | untyped → typed | rum_runtime::runtime::eval_core::symbol::EvaluateStoredOwnValueSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0278 | untyped → typed | rum_runtime::runtime::eval_dispatch::RunCallDispatchStageSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0279 | untyped → typed | rum_runtime::runtime::hold_thread::EvalReleaseHoldSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0526 | untyped → typed | rum_runtime::runtime::resolution::resume_frames::EvaluateProfiledFixedPointSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0527 | untyped → typed | rum_runtime::runtime::resolution::resume_frames::ResumeFixedPointFromSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; - |
| F0570 | untyped → typed | rum_runtime::runtime::eval_call_prepare::resume_frames::FinishPreparedCallArgumentsFinalizer implements rum_runtime::runtime::suspension::typed::SlotFinalizer; - |
| M0205 | untyped → typed | rum_runtime::runtime::builtin_adapters::builtin_evaluate#macro1:resume_typed_or_suspend:result; - |
| M0279 | untyped → typed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::prepare_arguments_for_call_with_unevaluated_restore#macro1:resume_typed_or_suspend:result; - |
| M0280 | untyped → removed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::resume_suspended_call_argument_preparation#macro1:resume_or_suspend:insert; F0272 |
| M0281 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::evaluate_call#macro1:resume_typed_or_suspend:result; - |
| M0282 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#macro1:resume_typed_or_suspend:result; - |
| M0283 | untyped → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#macro1:resume_typed_or_suspend:result; - |
| M0285 | untyped → typed | rum_runtime::runtime::eval_core::symbol::<Session as inherent>::evaluate_stored_own_value#macro1:resume_typed_or_suspend:result; - |
| M0286 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::run_call_dispatch_stage#macro1:resume_typed_or_suspend:result; - |
| M0287 | untyped → typed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::finish_builtin_dispatch#macro1:resume_typed_or_suspend:result; M0288 |
| M0288 | untyped → removed | rum_runtime::runtime::eval_dispatch::<Session as inherent>::try_dispatch_builtin_call#macro2:resume_typed_or_suspend:result; M0287 |
| M0289 | untyped → typed | rum_runtime::runtime::hold_thread::<Session as inherent>::eval_release_hold#macro1:resume_typed_or_suspend:result; - |
| M0561 | untyped → typed | rum_runtime::runtime::resolution::<Session as inherent>::evaluate_profiled_fixed_point#macro1:resume_typed_or_suspend:result; - |
| M0562 | untyped → typed | rum_runtime::runtime::resolution::<Session as inherent>::resume_fixed_point_from#macro1:resume_typed_or_suspend:result; - |
| D0001 | untyped → implemented | deferred::prepared arguments; F0204;F0272;F0273;F0274;F0275;F0276;F0277;F0278;F0279;F0526;F0527;F0570;F0275P |
| T0023 | untyped → removed | rum_runtime::runtime::eval_call_prepare::<Session as inherent>::prepare_arguments_for_call#method1:map:self . prepare_arguments_for_call_with_unevaluated_restore (head , attributes , args); F0272 |
| T0024 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_after_head#method1:into:evaluated_head; F0274 |
| T0025 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#method14:install_prepared_call_arguments:self; F0275;F0275P |
| T0026 | untyped → applied | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#method16:install_prepared_call_arguments:self; F0276;F0275P |
| T0027 | untyped → removed | rum_runtime::runtime::eval_core::symbol::<EvaluateStoredOwnValueSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#method4:map:(\| \| -> Result < rum_core :: Expr , EvalError > { if outcome . matched { session . evaluate_accepted_own_value (& symbol , body , delayed) } else { session . evaluate_symbol_after_stored_own_value (symbol) } }) (); F0277 |
| Whole0029 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call1:prepared_call_arguments_from_continuation_expr:; D009 |
| Whole0030 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call4:from_continuation_expr:; D125 |
| Whole0031 | legacy-use → removed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_dispatch_frame#call5:from_continuation_expr:; D125 |
| Whole0032 | legacy-use → removed | rum_runtime::runtime::eval_core::symbol::<EvaluateStoredOwnValueSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr:; D049 |
| F0275P | absent → typed | rum_runtime::runtime::eval_core::call::PreparedDispatchFrame implements rum_runtime::runtime::suspension::typed::TypedFrame; F0275;F0276 |
| M0282S | absent → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_call_with_dispatch_plan#macro2:resume_typed_or_suspend:result; M0282 |
| S3 | absent → implemented | S3 evaluation dispatch: 12 typed implementations and 13 typed macro sites; T0023;T0024;T0025;T0026;T0027;D0001 |
| X0040 | absent → legacy-test | rum_runtime::runtime::eval_call_prepare::prepared_slot_finalizer_fixture#method1:into_continuation_expr:finish_suspended_prepared_call (SuspendedPreparedCallArguments { evaluated_args : PreparedArgumentAccumulator :: freeze (prepared . evaluated_args . iter () . cloned () . collect () ,) , unevaluated_restores : prepared . unevaluated_restore . as_ref () . map (\| restore \| UnevaluatedRestoreAccumulator :: freeze (restore . wrappers . clone ())) , }); S3 typed finalizer materialized through the unchanged Legacy provider |
| M0283S | absent → typed | rum_runtime::runtime::eval_core::call::<Session as inherent>::resume_suspended_call_with_dispatch_plan#macro2:resume_typed_or_suspend:result; M0283 |
| X0041 | absent → legacy-test | rum_stdlib::tests::retention_closure::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views#reference7:resume_or_suspend:; S3 A3 AST-only legacy-opacity control; macro argument spans preserved |

### 3. Watched failures and final equality in both builds

The six replacement S3 divergences were watched on byte-identical S2 production in both builds. The three If-held and three listable-completion cases isolate F0278 semantic Some without crossing S13’s skipped Function map. These are preserved pre-conversion outputs, not rebuilt baselines. The six original Fold/Nest cases stay archived for S13; F0528/M0563 remain untouched.

**r1-pre-builtin-held-default: exit 101.** Full raw output: `.temp/brief-609/r1-pre-builtin-held-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.29s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 665, identity: None, text: SymbolText { id: 665, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-builtin-held-no-default: exit 101.** Full raw output: `.temp/brief-609/r1-pre-builtin-held-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_ -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.13s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... builtin.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... builtin.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 661, identity: None, text: SymbolText { id: 661, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... builtin.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 659, identity: None, text: SymbolText { id: 659, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
builtin.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_builtin_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
builtin.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some
    oracle::fixtures::evaluation_dispatch::s3_builtin_none
    oracle::fixtures::evaluation_dispatch::s3_builtin_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-default: exit 101.** Full raw output: `.temp/brief-609/r1-pre-threaded-dv-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.35s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 673, identity: None, text: SymbolText { id: 673, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 670, identity: None, text: SymbolText { id: 670, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r1-pre-threaded-dv-no-default: exit 101.** Full raw output: `.temp/brief-609/r1-pre-threaded-dv-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::evaluation_dispatch::s3_threaded -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: constant `FLIP_SLOT` is never used
 --> crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs:4:11
  |
4 | pub const FLIP_SLOT: Option<&str> = None;
  |           ^^^^^^^^^
  |
  = note: `#[warn(dead_code)]` on by default

warning: function `execute` is never used
  --> crates/rum-stdlib/tests/suspension_oracle/support/drive.rs:13:8
   |
13 | pub fn execute(
   |        ^^^^^^^

warning: `rum-stdlib` (test "suspension_oracle") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.46s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)

running 3 tests
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... threaded.empty_some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.empty_some resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=12

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.empty_some: UnexpectedPause(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... threaded.none queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 667, identity: None, text: SymbolText { id: 667, context: "System`", name: "InternalContinuationNone" } }))), args: SharedArgs([]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.none resumed: Err(UnexpectedPause(1))
continuation-slot fault-occurrence delta=0; observations=24

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_none' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.none: UnexpectedPause(1)
FAILED
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... threaded.some queued: Ok(ResultView { observation: Observation { value: Some(Call { head: SharedExpr(Atom(Symbol(Symbol { id: 664, identity: None, text: SymbolText { id: 664, context: "System`", name: "InternalContinuationSome" } }))), args: SharedArgs([Atom(Integer(Small(5)))]) }), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
threaded.some resumed: Ok(ResultView { observation: Observation { value: Some(Atom(Integer(Small(5)))), error: None, termination: None, messages: [], prints: [], trace: [] }, log: Call { head: SharedExpr(Atom(Symbol(Symbol { id: 1, identity: None, text: SymbolText { id: 1, context: "System`", name: "List" } }))), args: SharedArgs([]) }, projections: [] })
continuation-slot fault-occurrence delta=0; observations=48

thread 'oracle::fixtures::evaluation_dispatch::s3_threaded_some' panicked at crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_dispatch.rs:22:50:
threaded.some: Value
FAILED

failures:

failures:
    oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some
    oracle::fixtures::evaluation_dispatch::s3_threaded_none
    oracle::fixtures::evaluation_dispatch::s3_threaded_some

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```

**r6b-ordinary-default: exit 0.** Full raw output: `.temp/brief-609/r6b-ordinary-default.log`.

```text
cargo test --manifest-path .temp/brief-609/r2/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
```

```text
     Running witness_test.rs (.temp/brief-609/r2/ordinary-controls/target/debug/deps/ordinary-5cf883959ece1e3c)
running 1 test
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

**r6b-ordinary-no-default: exit 0.** Full raw output: `.temp/brief-609/r6b-ordinary-no-default.log`.

```text
cargo test --manifest-path .temp/brief-609/r2/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
```

```text
     Running witness_test.rs (.temp/brief-609/r2/ordinary-controls/target/debug/deps/ordinary-8f58f702763e7810)
running 1 test
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

Final complete suspension oracle: default

```text
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=17175
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=17187
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=17199
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=17211
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=17223
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=17235
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=17247
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=17259
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88016
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.75s
```

Final complete suspension oracle: no-default

```text
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=17175
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=17187
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=17199
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=17211
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=17223
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=17235
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=17247
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=17259
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88016
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.43s
```

Checks::ALL and unconditional FaultProbe absence are unchanged. The full raw outputs above identify both feature builds, every final equality and all fault-occurrence deltas. Native transport skips are reported separately from exercised passes below.

### 4. Analyzer, inventories, certificates and complete gate ledger

Both macro forms use the exact checked definition. Bare identifiers match the caller place without a new producer binding; expressions retain the binding. Each has five explicit expansion rejection obligations, including cold closure captures/moves, FnOnce invocation and drop. All four views read both compiled A3 fixtures. Legacy invocations stay opaque, and unknown success bindings stay unknown and scoped. Seven independent definition mutations are rejected in each view:28 deliberate rejections. No retention reject or strict token comparison was weakened.

The expression-only old definition was watched red; the first two-arm adapter exposed a quote-token whitespace mismatch, corrected with the identical &($result) spelling in both definitions. Two deliberate guard-disabled runs failed the drift falsifier. Restoring with an old file timestamp initially reused that weakened scratch binary; that failed run is retained, not accepted. A fresh timestamp rebuilt the restored exact guard and all seven controls passed. These watched controls apply to the byte-identical final macro/adapter sources.

**r6-views-old-definition: exit 101.** Full raw output: `.temp/brief-609/r6-views-old-definition.log`.

```text
cargo test --manifest-path .temp/brief-609/r6/view-fixture/Cargo.toml --test frontier macro_ -- --nocapture
```

```text
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling proc-macro2 v1.0.107
   Compiling syn v3.0.4
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.229
   Compiling serde v1.0.229
   Compiling serde_derive v1.0.229
   Compiling thiserror v2.0.20
   Compiling num-integer v0.1.47
   Compiling thiserror-impl v2.0.20
   Compiling num-bigint v0.4.8
   Compiling syn v2.0.119
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling brief609-frontier-fixture v0.0.0 (/home/remi/dev/rum/.temp/brief-609/r6/view-fixture)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 46.89s
     Running main.rs (.temp/brief-609/r6/view-fixture/target/debug/deps/frontier-eb82c8738f75f1e7)

running 6 tests
test checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views ... 
thread 'checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
FAILED
test checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view ... 
thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT producer regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT producer locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT producer methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT producer types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT cold helper regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT cold helper locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT cold helper methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT cold helper types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT owned capture regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT owned capture locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT owned capture methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT owned capture types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
ok
test checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
FAILED
test checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments ... 
thread 'checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
FAILED
test checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select ... 
thread 'checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select' panicked at checker/typed_resume_macro.rs:190:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:2:9-7:10: typed resume macro definition changed
FAILED

failures:

failures:
    checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views
    checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments
    checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments
    checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments
    checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select

test result: FAILED. 1 passed; 5 failed; 0 ignored; 0 measured; 2 filtered out; finished in 88.23s

error: test failed, to rerun pass `--test frontier`
```

**r6-drift-final-watched: exit 101.** Full raw output: `.temp/brief-609/r6-drift-final-watched.log`.

```text
cargo test --manifest-path .temp/brief-609/r6/view-fixture/Cargo.toml --test frontier typed_macro_definition_drift_fails_every_view -- --nocapture
```

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling brief609-frontier-fixture v0.0.0 (/home/remi/dev/rum/.temp/brief-609/r6/view-fixture)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.75s
     Running main.rs (.temp/brief-609/r6/view-fixture/target/debug/deps/frontier-eb82c8738f75f1e7)

running 1 test
test checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view ... TYPED_MACRO_DRIFT_NOT_REJECTED identifier matcher regions
TYPED_MACRO_DRIFT_NOT_REJECTED identifier matcher locator
TYPED_MACRO_DRIFT_NOT_REJECTED identifier matcher methods
TYPED_MACRO_DRIFT_NOT_REJECTED identifier matcher types
TYPED_MACRO_DRIFT_NOT_REJECTED identifier witness regions
TYPED_MACRO_DRIFT_NOT_REJECTED identifier witness locator
TYPED_MACRO_DRIFT_NOT_REJECTED identifier witness methods
TYPED_MACRO_DRIFT_NOT_REJECTED identifier witness types
TYPED_MACRO_DRIFT_NOT_REJECTED identifier cold helper regions
TYPED_MACRO_DRIFT_NOT_REJECTED identifier cold helper locator
TYPED_MACRO_DRIFT_NOT_REJECTED identifier cold helper methods
TYPED_MACRO_DRIFT_NOT_REJECTED identifier cold helper types
TYPED_MACRO_DRIFT_NOT_REJECTED identifier owned capture regions
TYPED_MACRO_DRIFT_NOT_REJECTED identifier owned capture locator
TYPED_MACRO_DRIFT_NOT_REJECTED identifier owned capture methods
TYPED_MACRO_DRIFT_NOT_REJECTED identifier owned capture types
TYPED_MACRO_DRIFT_NOT_REJECTED expression producer regions
TYPED_MACRO_DRIFT_NOT_REJECTED expression producer locator
TYPED_MACRO_DRIFT_NOT_REJECTED expression producer methods
TYPED_MACRO_DRIFT_NOT_REJECTED expression producer types
TYPED_MACRO_DRIFT_NOT_REJECTED expression cold helper regions
TYPED_MACRO_DRIFT_NOT_REJECTED expression cold helper locator
TYPED_MACRO_DRIFT_NOT_REJECTED expression cold helper methods
TYPED_MACRO_DRIFT_NOT_REJECTED expression cold helper types
TYPED_MACRO_DRIFT_NOT_REJECTED expression owned capture regions
TYPED_MACRO_DRIFT_NOT_REJECTED expression owned capture locator
TYPED_MACRO_DRIFT_NOT_REJECTED expression owned capture methods
TYPED_MACRO_DRIFT_NOT_REJECTED expression owned capture types

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/compiler_oracle/corpus.rs:462:5:
every view must fail on drift: ["identifier matcher: regions", "identifier matcher: locator", "identifier matcher: methods", "identifier matcher: types", "identifier witness: regions", "identifier witness: locator", "identifier witness: methods", "identifier witness: types", "identifier cold helper: regions", "identifier cold helper: locator", "identifier cold helper: methods", "identifier cold helper: types", "identifier owned capture: regions", "identifier owned capture: locator", "identifier owned capture: methods", "identifier owned capture: types", "expression producer: regions", "expression producer: locator", "expression producer: methods", "expression producer: types", "expression cold helper: regions", "expression cold helper: locator", "expression cold helper: methods", "expression cold helper: types", "expression owned capture: regions", "expression owned capture: locator", "expression owned capture: methods", "expression owned capture: types"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 15.45s

error: test failed, to rerun pass `--test frontier`
```

**r6-views-final: exit 101.** Full raw output: `.temp/brief-609/r6-views-final.log`.

```text
cargo test --manifest-path .temp/brief-609/r6/view-fixture/Cargo.toml --test frontier macro_ -- --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running main.rs (.temp/brief-609/r6/view-fixture/target/debug/deps/frontier-eb82c8738f75f1e7)

running 7 tests
test checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views ... LEGACY_MACRO_OPAQUE regions/locator/methods/types
ok
test checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments ... TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:151:25-151:37 std::option::Option::Some
TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:17-153:32 std::result::Result::Ok
TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:20-153:31 std::option::Option::Some
ok
test checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view ... TYPED_MACRO_DRIFT_NOT_REJECTED identifier matcher regions
TYPED_MACRO_DRIFT_NOT_REJECTED identifier matcher locator
TYPED_MACRO_DRIFT_NOT_REJECTED identifier matcher methods
TYPED_MACRO_DRIFT_NOT_REJECTED identifier matcher types
TYPED_MACRO_DRIFT_NOT_REJECTED identifier witness regions
TYPED_MACRO_DRIFT_NOT_REJECTED identifier witness locator
TYPED_MACRO_DRIFT_NOT_REJECTED identifier witness methods
TYPED_MACRO_DRIFT_NOT_REJECTED identifier witness types
TYPED_MACRO_DRIFT_NOT_REJECTED identifier cold helper regions
TYPED_MACRO_DRIFT_NOT_REJECTED identifier cold helper locator
TYPED_MACRO_DRIFT_NOT_REJECTED identifier cold helper methods
TYPED_MACRO_DRIFT_NOT_REJECTED identifier cold helper types
TYPED_MACRO_DRIFT_NOT_REJECTED identifier owned capture regions
TYPED_MACRO_DRIFT_NOT_REJECTED identifier owned capture locator
TYPED_MACRO_DRIFT_NOT_REJECTED identifier owned capture methods
TYPED_MACRO_DRIFT_NOT_REJECTED identifier owned capture types
TYPED_MACRO_DRIFT_NOT_REJECTED expression producer regions
TYPED_MACRO_DRIFT_NOT_REJECTED expression producer locator
TYPED_MACRO_DRIFT_NOT_REJECTED expression producer methods
TYPED_MACRO_DRIFT_NOT_REJECTED expression producer types
TYPED_MACRO_DRIFT_NOT_REJECTED expression cold helper regions
TYPED_MACRO_DRIFT_NOT_REJECTED expression cold helper locator
TYPED_MACRO_DRIFT_NOT_REJECTED expression cold helper methods
TYPED_MACRO_DRIFT_NOT_REJECTED expression cold helper types
TYPED_MACRO_DRIFT_NOT_REJECTED expression owned capture regions
TYPED_MACRO_DRIFT_NOT_REJECTED expression owned capture locator
TYPED_MACRO_DRIFT_NOT_REJECTED expression owned capture methods
TYPED_MACRO_DRIFT_NOT_REJECTED expression owned capture types

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/compiler_oracle/corpus.rs:462:5:
every view must fail on drift: ["identifier matcher: regions", "identifier matcher: locator", "identifier matcher: methods", "identifier matcher: types", "identifier witness: regions", "identifier witness: locator", "identifier witness: methods", "identifier witness: types", "identifier cold helper: regions", "identifier cold helper: locator", "identifier cold helper: methods", "identifier cold helper: types", "identifier owned capture: regions", "identifier owned capture: locator", "identifier owned capture: methods", "identifier owned capture: types", "expression producer: regions", "expression producer: locator", "expression producer: methods", "expression producer: types", "expression cold helper: regions", "expression cold helper: locator", "expression cold helper: methods", "expression cold helper: types", "expression owned capture: regions", "expression owned capture: locator", "expression owned capture: methods", "expression owned capture: types"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments ... TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:151:25-151:37 std::option::Option::Some std::option::Option<usize>
TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:17-153:32 std::result::Result::Ok std::result::Result<std::option::Option<rum_core::expr::Expr>, rum_runtime::runtime::session::session_state::EvalError>
TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:20-153:31 std::option::Option::Some std::option::Option<rum_core::expr::Expr>
ok
test checker::compiler_oracle::corpus::typed_macro_expression_form_reads_all_four_views ... TYPED_MACRO_EXPRESSION regions/locator/methods/types
ok
test checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments ... TYPED_MACRO_METHOD crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:152:25-152:39 rum_stdlib::tests::retention_closure::compiler_oracle::corpus::typed_argument_fixture::Local::choose
ok
test checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select ... ok

failures:

failures:
    checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 108.60s

error: test failed, to rerun pass `--test frontier`
```

**r6-views-rebuilt: exit 0.** Full raw output: `.temp/brief-609/r6-views-rebuilt.log`.

```text
cargo test --manifest-path .temp/brief-609/r6/view-fixture/Cargo.toml --test frontier macro_ -- --nocapture
```

```text
   Compiling brief609-frontier-fixture v0.0.0 (/home/remi/dev/rum/.temp/brief-609/r6/view-fixture)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.10s
     Running main.rs (.temp/brief-609/r6/view-fixture/target/debug/deps/frontier-eb82c8738f75f1e7)

running 7 tests
test checker::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views ... LEGACY_MACRO_OPAQUE regions/locator/methods/types
ok
test checker::compiler_oracle::corpus::typed_macro_constructor_locator_reads_arguments ... TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:151:25-151:37 std::option::Option::Some
TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:17-153:32 std::result::Result::Ok
TYPED_MACRO_LOCATED crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:20-153:31 std::option::Option::Some
ok
test checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view ... 
thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
TYPED_MACRO_DRIFT identifier matcher regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier matcher locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier matcher methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier matcher types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier witness regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier witness locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier witness methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier witness types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier cold helper regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier cold helper locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier cold helper methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier cold helper types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier owned capture regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier owned capture locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier owned capture methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT identifier owned capture types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression producer regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression producer locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression producer methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression producer types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression cold helper regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression cold helper locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression cold helper methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression cold helper types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression owned capture regions crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression owned capture locator crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression owned capture methods crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed

thread 'checker::compiler_oracle::corpus::typed_macro_definition_drift_fails_every_view' panicked at checker/typed_resume_macro.rs:228:9:
crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
TYPED_MACRO_DRIFT expression owned capture types crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:148:9-156:10: typed resume macro definition changed
ok
test checker::compiler_oracle::corpus::typed_macro_expected_constructor_type_reads_arguments ... TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:151:25-151:37 std::option::Option::Some std::option::Option<usize>
TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:17-153:32 std::result::Result::Ok std::result::Result<std::option::Option<rum_core::expr::Expr>, rum_runtime::runtime::session::session_state::EvalError>
TYPED_MACRO_TYPE crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:153:20-153:31 std::option::Option::Some std::option::Option<rum_core::expr::Expr>
ok
test checker::compiler_oracle::corpus::typed_macro_expression_form_reads_all_four_views ... TYPED_MACRO_EXPRESSION regions/locator/methods/types
ok
test checker::compiler_oracle::corpus::typed_macro_method_selection_reads_arguments ... TYPED_MACRO_METHOD crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs:152:25-152:39 rum_stdlib::tests::retention_closure::compiler_oracle::corpus::typed_argument_fixture::Local::choose
ok
test checker::compiler_oracle::corpus::typed_macro_unknown_pattern_does_not_leak_or_select ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 104.42s
```

The final frontier is937: RET00110, RET002901, RET00326. Direct S2 reconciliation is932→937:912 identical rows, 19 paired same-operation moves, one opaque M0561 RET003 replaced by five explicitly rejected expansion rows, and one explicit Try/FromResidual rejection. No unpaired removal or newly admitted boundary remains. The final driver repair leaves all937 frontier rows, all170 scoped constructor contexts and all7348 broad contexts byte-identical to the first R6 validation. See `r6/frontier-s2-classified.json` and `r6b/stage6-view-diff.json`. The historical printed ACCEPTANCE_BASELINE927 is deliberately unchanged.

The constructor claim change7311→7313 is fully owned: driver Err3→4 and M0561 Ok2→3. The final compiler instruments below provide the authoritative agreement totals; duplicate exact TOTAL prints are not summed. Ownership retains79 approved payload identities and59 self-reaching types. From R6 entry, unresolved captures2956→2954 removes only record_head and the adapter map_err closure; macros3152→3154 adds unreachable! and the placeholder debug assertion. Attributes1514/storage94 are unchanged. Four adapter witness cells move 36→32→33 (three cells) and82→74→75 (one). All identity/disposition cells remain unchanged. Full replay diffs are preserved in both R6 evidence directories.

All68 context fingerprints were replayed after the final source: fnv1a64:928be82d795f65ef. Both feature builds independently earned three production-instance and four probe marks, with no fixture/helper marks, before rejecting the stale certificates. Only four C01/C02 certificate cells changed; all seven other columns, certified bodies and the local analyzer remain unchanged. Both final local acceptance runs pass.

The following ledger pastes every test-result line from every R6/R6b bounded gate, including intended negative children and stale-pin failures. Logs without test-result lines remain in `r6b/all-r6-gate-totals.json`; they are not counted as tests. Each raw log has its exact command, environment and exit record. Final acceptance is the R6b sequence after the last runtime revision; earlier green runs are intermediate evidence.

| Raw log | Exit | Verbatim totals |
| --- | --- | --- |
| .temp/brief-609/r6-attempt4-typed.log | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.46s |
| .temp/brief-609/r6-census-default.log | 0 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.49s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.65s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 140.21s |
| .temp/brief-609/r6-census-discovery.log | 0 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.67s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.97s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 142.55s |
| .temp/brief-609/r6-census-no-default.log | 0 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.55s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.60s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 141.29s |
| .temp/brief-609/r6-constructor-oracle.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 260.62s |
| .temp/brief-609/r6-controls-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.13s |
| .temp/brief-609/r6-controls-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.99s |
| .temp/brief-609/r6-definition-token-diagnostic.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 14.79s |
| .temp/brief-609/r6-drift-final-watched.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 15.45s |
| .temp/brief-609/r6-drift-watched.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 15.68s |
| .temp/brief-609/r6-emit-certificates-default.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 18.93s |
| .temp/brief-609/r6-emit-certificates-no-default.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1145 filtered out; finished in 18.93s |
| .temp/brief-609/r6-exact-oracle.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 199.65s |
| .temp/brief-609/r6-frame-roots-reconciled.log | 0 | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.37s |
| .temp/brief-609/r6-frame-roots.log | 101 | test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.52s |
| .temp/brief-609/r6-frontier-final.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 26.03s |
| .temp/brief-609/r6-local-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 18.33s |
| .temp/brief-609/r6-local-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1145 filtered out; finished in 18.86s |
| .temp/brief-609/r6-native-skips.log | 0 | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s |
| .temp/brief-609/r6-oracle-default.log | 0 | test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.14s |
| .temp/brief-609/r6-oracle-no-default.log | 0 | test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.69s |
| .temp/brief-609/r6-ordinary-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s |
| .temp/brief-609/r6-ordinary-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s |
| .temp/brief-609/r6-ownership-discovery.log | 101 | test result: FAILED. 22 passed; 3 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 68.28s |
| .temp/brief-609/r6-ownership-final.log | 0 | test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 70.96s |
| .temp/brief-609/r6-typed-runtime.log | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.45s |
| .temp/brief-609/r6-unchanged-depth48-exact.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1171 filtered out; finished in 0.01s |
| .temp/brief-609/r6-unchanged-depth48.log | 0 | test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s |
| .temp/brief-609/r6-views-final.log | 101 | test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 108.60s |
| .temp/brief-609/r6-views-inventory.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 10.37s |
| .temp/brief-609/r6-views-old-definition.log | 101 | test result: FAILED. 1 passed; 5 failed; 0 ignored; 0 measured; 2 filtered out; finished in 88.23s |
| .temp/brief-609/r6-views-rebuilt.log | 0 | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 104.42s |
| .temp/brief-609/r6-views.log | 101 | test result: FAILED. 1 passed; 6 failed; 0 ignored; 0 measured; 2 filtered out; finished in 104.25s |
| .temp/brief-609/r6-workspace.log | 0 | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s<br>test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s<br>test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s<br>test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s<br>test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.40s<br>test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s<br>test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s<br>test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s<br>test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s<br>test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.02s<br>test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s<br>test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s<br>test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s<br>test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s<br>test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 39.53s<br>test result: ok. 1099 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1217.74s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s<br>test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s<br>test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s<br>test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.23s<br>test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s<br>test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s<br>test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s<br>test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s<br>test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s<br>test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.04s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 141.39s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.46s<br>test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.63s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.23s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.85s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| .temp/brief-609/r6b-census-default.log | 0 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.29s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.33s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.09s |
| .temp/brief-609/r6b-census-no-default.log | 0 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.42s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.56s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 140.94s |
| .temp/brief-609/r6b-constructor-oracle.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 231.25s |
| .temp/brief-609/r6b-controls-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.36s |
| .temp/brief-609/r6b-controls-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.77s |
| .temp/brief-609/r6b-depth48-exact.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1171 filtered out; finished in 0.01s |
| .temp/brief-609/r6b-emit-certificates-default.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 18.71s |
| .temp/brief-609/r6b-emit-certificates-no-default.log | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1145 filtered out; finished in 18.49s |
| .temp/brief-609/r6b-exact-oracle.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 200.98s |
| .temp/brief-609/r6b-frame-roots.log | 0 | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.28s |
| .temp/brief-609/r6b-frontier-final.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 25.36s |
| .temp/brief-609/r6b-local-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 18.36s |
| .temp/brief-609/r6b-local-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1145 filtered out; finished in 18.62s |
| .temp/brief-609/r6b-native-skips.log | 0 | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s |
| .temp/brief-609/r6b-oracle-default.log | 0 | test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.75s |
| .temp/brief-609/r6b-oracle-no-default.log | 0 | test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.43s |
| .temp/brief-609/r6b-ordinary-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s |
| .temp/brief-609/r6b-ordinary-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s |
| .temp/brief-609/r6b-ownership-discovery.log | 101 | test result: FAILED. 22 passed; 3 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 67.77s |
| .temp/brief-609/r6b-ownership-final.log | 0 | test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 70.21s |
| .temp/brief-609/r6b-post-stack-frame-roots.log | 0 | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.76s |
| .temp/brief-609/r6b-post-stack-frontier-final.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 25.90s |
| .temp/brief-609/r6b-post-stack-local-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 19.29s |
| .temp/brief-609/r6b-post-stack-local-no-default.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1145 filtered out; finished in 19.85s |
| .temp/brief-609/r6b-post-stack-ownership-final.log | 0 | test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 70.57s |
| .temp/brief-609/r6b-post-stack-typed-runtime.log | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.46s |
| .temp/brief-609/r6b-typed-runtime.log | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.44s |
| .temp/brief-609/r6b-views-inventory.log | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 10.32s |
| .temp/brief-609/r6b-workspace.log | 0 | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s<br>test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s<br>test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s<br>test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s<br>test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.98s<br>test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s<br>test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s<br>test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s<br>test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s<br>test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.01s<br>test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s<br>test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s<br>test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s<br>test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s<br>test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 38.06s<br>test result: ok. 1099 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 869.30s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s<br>test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s<br>test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s<br>test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.34s<br>test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s<br>test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s<br>test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s<br>test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s<br>test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s<br>test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.78s<br>test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 140.84s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.91s<br>test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s<br>test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s<br>test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.28s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.03s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s<br>test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |

Final mandatory command receipts follow. Each command ran under scripts/run-bounded.py with CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1, CARGO_NET_OFFLINE=true and TMPDIR under brief609; workspace used 3600 seconds, each oracle 1500 seconds and other gates 900 seconds. Full raw output and exact environment are retained beside each command receipt.

**r6b-format-final: exit 0.** Full raw output: `.temp/brief-609/r6b-format-final.log`.

```text
cargo fmt --check
```

```text
(no output)
```

**r6b-post-stack-typed-runtime: exit 0.** Full raw output: `.temp/brief-609/r6b-post-stack-typed-runtime.log`.

```text
cargo test -p rum-runtime --lib suspension::typed::tests
```

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
running 20 tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.46s
```

**r6b-post-stack-frame-roots: exit 0.** Full raw output: `.temp/brief-609/r6b-post-stack-frame-roots.log`.

```text
cargo test -p rum-runtime --lib frame_root_trace
```

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 1.76s
```

**r6b-depth48-exact: exit 0.** Full raw output: `.temp/brief-609/r6b-depth48-exact.log`.

```text
cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
```

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1171 filtered out; finished in 0.01s
```

**r6b-census-default: exit 0.** Full raw output: `.temp/brief-609/r6b-census-default.log`.

```text
cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
```

```text
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-943817a3ecf6c029)
running 6 tests
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.29s
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.33s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.09s
```

**r6b-census-no-default: exit 0.** Full raw output: `.temp/brief-609/r6b-census-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
```

```text
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-cce6bcaa604a405f)
running 6 tests
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.42s
running 1 test
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.56s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 140.94s
```

**r6b-post-stack-ownership-final: exit 0.** Full raw output: `.temp/brief-609/r6b-post-stack-ownership-final.log`.

```text
cargo test -p rum-stdlib --lib ownership_graph
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 25 tests
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 70.57s
```

**r6b-post-stack-local-default: exit 0.** Full raw output: `.temp/brief-609/r6b-post-stack-local-default.log`.

```text
cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 19.29s
```

**r6b-post-stack-local-no-default: exit 0.** Full raw output: `.temp/brief-609/r6b-post-stack-local-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c942ac40233a21b0)
running 1 test
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1145 filtered out; finished in 19.85s
```

**r6b-post-stack-frontier-final: exit 0.** Full raw output: `.temp/brief-609/r6b-post-stack-frontier-final.log`.

```text
cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
INDEPENDENT_AUDIT EDGE_ALL roots=7343 definitions=7343 raw=48341 deduplicated=48341 reason_classes=59 all_spans_read=true
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 25.90s
```

**r6b-oracle-default: exit 0.** Full raw output: `.temp/brief-609/r6b-oracle-default.log`.

```text
cargo test -p rum-stdlib --test suspension_oracle -- --nocapture
```

```text
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ad9873bc53ac1c0b)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.75s
```

**r6b-oracle-no-default: exit 0.** Full raw output: `.temp/brief-609/r6b-oracle-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle -- --nocapture
```

```text
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-ed9ba19bc18733e6)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.43s
```

**r6b-controls-default: exit 0.** Full raw output: `.temp/brief-609/r6b-controls-default.log`.

```text
cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.33s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-d6fbf6132444c3d1)

running 1 test
test typed_slot_compile_controls ... PAIR first-chain-input: positive PASS; negative E0271 at src/lib.rs:27:35
PAIR chain-adjacency: positive PASS; negative E0271 at src/lib.rs:34:70
PAIR unbound-chain-constructor: positive PASS; negative E0599 at src/lib.rs:40:41
PAIR evaluate-input: positive PASS; negative E0271 at src/lib.rs:46:66
PAIR evaluate-output: positive PASS; negative E0271 at src/lib.rs:52:66
PAIR finalizer-output: positive PASS; negative E0271 at src/lib.rs:58:48
PAIR opaque-action: positive PASS; negative E0599 at src/lib.rs:63:39
PAIR token-no-clone: positive PASS; negative E0599 at src/lib.rs:69:18
PAIR token-no-mint: positive PASS; negative E0599 at src/lib.rs:75:27
PAIR token-no-escape: positive PASS; negative lifetime at src/lib.rs:81:1
PAIR sealed-extension: positive PASS; negative E0277 at src/lib.rs:87:20
PAIR codec-key-private: positive PASS; negative E0603 at src/lib.rs:93:18
PAIR opaque-selector-pass: positive PASS; negative E0271 at src/lib.rs:100:35
PAIR mode-not-frame-chosen: positive PASS; negative E0599 at src/lib.rs:106:65
PAIR mode-private-selection: positive PASS; negative E0603 at src/lib.rs:112:18
PAIR slot-mismatch-private: positive PASS; negative E0451 at src/lib.rs:124:95
PAIR input: positive PASS; negative E0271 at src/lib.rs:131:41
PAIR output-return: positive PASS; negative E0308 at src/lib.rs:138:1
PAIR chain-last-output: positive PASS; negative E0308 at src/lib.rs:145:1
PAIR token-fresh-type: positive PASS; negative E0308 at src/lib.rs:159:47
PAIR token-witness-input: positive PASS; negative E0308 at src/lib.rs:167:17
PAIR deferred-final-output: positive PASS; negative E0308 at src/lib.rs:174:5
PAIR generic-frame: positive PASS; negative E0308 at src/lib.rs:180:1
PAIR closure-return: positive PASS; negative E0308 at src/lib.rs:187:1
PAIR sibling-erased-private: positive PASS; negative E0603 at crates/rum-runtime/src/runtime/mod.rs:135:47
PAIR mode-expr-fixed: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17
PAIR mode-new-state-only: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed.rs:66:40
PAIR mode-one-associated-type: positive PASS; negative E0201 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-no-codec-override: positive PASS; negative E0407 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-legacy-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:74:19
PAIR mode-state-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:79:15
COMPILE CONTROLS: 31 positive / 31 intended rejections (22 S1 + 7 S2 + opaque fault + token witness); raw Cargo JSON and primary spans retained
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.36s
```

**r6b-controls-no-default: exit 0.** Full raw output: `.temp/brief-609/r6b-controls-no-default.log`.

```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
```

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.37s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-e5d376d145e23731)

running 1 test
test typed_slot_compile_controls ... PAIR first-chain-input: positive PASS; negative E0271 at src/lib.rs:27:35
PAIR chain-adjacency: positive PASS; negative E0271 at src/lib.rs:34:70
PAIR unbound-chain-constructor: positive PASS; negative E0599 at src/lib.rs:40:41
PAIR evaluate-input: positive PASS; negative E0271 at src/lib.rs:46:66
PAIR evaluate-output: positive PASS; negative E0271 at src/lib.rs:52:66
PAIR finalizer-output: positive PASS; negative E0271 at src/lib.rs:58:48
PAIR opaque-action: positive PASS; negative E0599 at src/lib.rs:63:39
PAIR token-no-clone: positive PASS; negative E0599 at src/lib.rs:69:18
PAIR token-no-mint: positive PASS; negative E0599 at src/lib.rs:75:27
PAIR token-no-escape: positive PASS; negative lifetime at src/lib.rs:81:1
PAIR sealed-extension: positive PASS; negative E0277 at src/lib.rs:87:20
PAIR codec-key-private: positive PASS; negative E0603 at src/lib.rs:93:18
PAIR opaque-selector-pass: positive PASS; negative E0271 at src/lib.rs:100:35
PAIR mode-not-frame-chosen: positive PASS; negative E0599 at src/lib.rs:106:65
PAIR mode-private-selection: positive PASS; negative E0603 at src/lib.rs:112:18
PAIR slot-mismatch-private: positive PASS; negative E0451 at src/lib.rs:124:95
PAIR input: positive PASS; negative E0271 at src/lib.rs:131:41
PAIR output-return: positive PASS; negative E0308 at src/lib.rs:138:1
PAIR chain-last-output: positive PASS; negative E0308 at src/lib.rs:145:1
PAIR token-fresh-type: positive PASS; negative E0308 at src/lib.rs:159:47
PAIR token-witness-input: positive PASS; negative E0308 at src/lib.rs:167:17
PAIR deferred-final-output: positive PASS; negative E0308 at src/lib.rs:174:5
PAIR generic-frame: positive PASS; negative E0308 at src/lib.rs:180:1
PAIR closure-return: positive PASS; negative E0308 at src/lib.rs:187:1
PAIR sibling-erased-private: positive PASS; negative E0603 at crates/rum-runtime/src/runtime/mod.rs:135:47
PAIR mode-expr-fixed: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17
PAIR mode-new-state-only: positive PASS; negative E0277 at crates/rum-runtime/src/runtime/suspension/typed.rs:66:40
PAIR mode-one-associated-type: positive PASS; negative E0201 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-no-codec-override: positive PASS; negative E0407 at crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5
PAIR mode-legacy-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:74:19
PAIR mode-state-provider-narrow: positive PASS; negative E0053 at crates/rum-runtime/src/runtime/suspension/typed.rs:79:15
COMPILE CONTROLS: 31 positive / 31 intended rejections (22 S1 + 7 S2 + opaque fault + token witness); raw Cargo JSON and primary spans retained
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.77s
```

**r6b-constructor-oracle: exit 0.** Full raw output: `.temp/brief-609/r6b-constructor-oracle.log`.

```text
cargo test -p rum-stdlib --lib census_constructor_resolutions_are_compiler_observed -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
A1_CORPUS_TOTAL agreements=7313 disagreements=0 claims=7313
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 231.25s
```

**r6b-exact-oracle: exit 0.** Full raw output: `.temp/brief-609/r6b-exact-oracle.log`.

```text
cargo test -p rum-stdlib --lib exact_types_and_constructor_claims_match_rustc -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)
running 1 test
A1_ORACLE_TOTAL agreements=285 disagreements=0
A1_ORACLE_TOTAL agreements=285 disagreements=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 200.98s
```

**r6b-workspace: exit 0.** Full raw output: `.temp/brief-609/r6b-workspace.log`.

```text
cargo test --workspace --no-fail-fast
```

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
     Running unittests src/main.rs (target/debug/deps/rum_cli-0aee412e0b99cbf1)
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/cli.rs (target/debug/deps/cli-bc0b4c8761b62ae9)
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/input_requests.rs (target/debug/deps/input_requests-e84d0b9e62f1dd0f)
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/terminal.rs (target/debug/deps/terminal-70e027c8b6d403e3)
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
     Running unittests src/lib.rs (target/debug/deps/rum_core-05ee71324ea83d3f)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-058a94fcf3064f03)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-c657e563db7a31de)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-cb1c0b1fa13c3a60)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-0c15fc615add95e2)
running 90 tests
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-75627f143cb45b08)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-f1d7a1c822224b1c)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/capture_output_lifecycle.rs (target/debug/deps/capture_output_lifecycle-3426e74b0e43035a)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-3f3ebe72da026eee)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-52956b33f57a0130)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.98s
     Running tests/installer.rs (target/debug/deps/installer-245b9a9d0dd76683)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-cf4abded19978b7b)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-7db6f3bc8fd1b373)
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-bcf408336a660b97)
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/shared_accumulator_completion.rs (target/debug/deps/shared_accumulator_completion-4fc6d427214e6aa2)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/stress.rs (target/debug/deps/stress-29e64a7d12eeb866)
running 6 tests
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-0e6a452c6cdaf889)
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.01s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-7953000542543e9e)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-598afbd7e7b200bf)
running 151 tests
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-0f1e351c2b3d67dc)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-4387fdc47fc58f50)
running 48 tests
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-11b7b14c69d5d259)
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-02bf146f80080963)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-f6425a9dcb35714f)
running 1172 tests
test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 38.06s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-8951251f7d2e445e)
running 1152 tests
test result: ok. 1099 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 869.30s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-d2ad2c98e57a1cba)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-f117f222c1cc964c)
running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-368dcefe778b9dd9)
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-2c9bd859cff3337d)
running 47 tests
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-ef76e1f37b5e402d)
running 69 tests
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.34s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-dbe263d2d0c33a6d)
running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-13289bfd2ecc5d28)
running 100 tests
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-487a4fb3b8b94ac7)
running 71 tests
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-4182a401e6279d72)
running 40 tests
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-757402035d2c0050)
running 51 tests
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-58e5a1c5f9b651ff)
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.78s
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-3f7c0defd057db37)
running 6 tests
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 140.84s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-d72233bfaf7c8103)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.91s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-08cb56e2fd17a1b7)
running 193 tests
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-f8efeb939a755a12)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
   Doc-tests rum_host
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.28s
   Doc-tests rum_jupyter
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
running 22 tests
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.03s
   Doc-tests rum_stdlib
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**r6b-native-skips: exit 0.** Full raw output: `.temp/brief-609/r6b-native-skips.log`.

```text
cargo test -p rum-jupyter --lib zeromq::tests::native -- --nocapture
```

```text
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-dd24b36d3edaa47a)
running 9 tests
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s
```

The final workspace command exited 0: 53 suites, 3462 reported passes, 0 failures and 59 ignored tests. Nine reported passes are native ZeroMQ early returns because loopback access is unavailable; the separate `--nocapture` run records each skip. Therefore 3453 tests exercised their bodies successfully, plus nine native skips. All test-result lines and skip messages are pasted above and archived in `r6b/workspace-summary.json`.

After the last runtime revision, typed-driver tests passed 20/0, frame-root tests 15/0, ownership 25/0, frontier 1/0, and the unchanged fully qualified 48-level recursion test 1/0. Both local-production builds passed. Each compile-control build passed all 31 positive/negative pairs; each census harness passed 5/0 with one intentionally ignored direct probe and two deliberately failing child probes. The constructor instrument agreed on all 7313 claims, with zero disagreements; the exact instrument agreed on 285 claims, with zero disagreements. All 28 ordinary controls passed with Checks::ALL and 336 zero-fault observations per build. Each complete suspension oracle passed 58/0, with every fault-occurrence delta zero and 88016 final observations. Formatting passed again after the workspace run. The final scope audit found 52 changed files (including this brief) and three new cumulative S3 files, preserved all 183 protected inputs and all four S2 binaries, and found no new unsafe or out-of-scope change.

Builds/tests/measurements were bounded and serial, with offline Cargo, one build job, one test thread and TMPDIR under brief609. The inherited unused runtime test-helper warnings are register_test_if_builtin, register_test_return_builtin and register_test_vm_template_builtins; their definitions are unchanged. No new warning is introduced, and both complete suspension oracle builds are warning-free. All native probe test outputs, including successful harness runs returning StackLimit, remain under r6b/stack/. A returned StackLimit is not counted as a successful guest depth.

### 5. Native stack, allocation, instruction, resume attribution and timing evidence

The director's 136-byte hypothesis was confirmed before editing; the 168-byte hypothesis was refuted. In the R5 CLI, `0x59e625`, `0x5a04a0` and `0x5a0642` copy a produced result into the macro's new owned binding before its discriminant match. The prepared-argument branch at `0x5a04a0` does not borrow that result for profiling, so profiling is not a necessary cause. The preserved historical identifier/expression native binaries isolate the macro shape: the corresponding unconditional copies disappear with the caller-place form. The exact windows are in `r6/pre-fix-copy-windows.json` and `historical-ident-vs-expr-copy-sites.json`.

The 168-byte transfers at `0x5a9457` and `0x5a94a7` surround head evaluation, before profile observation. They move the `record_head` FnOnce capture: three 56-byte timing records. They remain in the historical identifier binary, so they are not producer-result rebinds. Removing that aggregate closure and consuming its three records directly, in the original order in each branch, removes these transfers.

The R5 leaf's extra `lea 0x120(%rsp),%r14` at `0x71fe9d` hoists a stack address used by cycle-error formatting before the stable-value branch. The M0561 typed-macro conversion changed the enclosing function's temporary/register allocation. An isolated M0561 Ok/? change reduced the quick leaf114→113 but retained the hoist (`0x72016d`, then offset0x190); isolating only the cycle-error formatting into the cold helper removed it and reduced the quick leaf to110 versus S2's112. These are the observed dataflow and source-isolated results; a claim about the compiler's internal heuristic would be inference. Final addressed leaf comparisons are in the performance evidence below.

| Trial | Change | Named-gate evidence | Consecutive no-progress count |
| --- | --- | --- | ---: |
| 1 | Caller-place identifier macro arm | Quick map-first-64 copies fall to1252; S2 is1124. Leaf114. | 0 |
| 2 | Remove record_head aggregate closure | Quick map-first-64 copies1124, equal to S2. | 0 |
| 3 | M0561 Ok(value)/? shape | Quick leaf114→113. | 0 |
| 4 | Private DecodeFailure action, mutable input, per-arm outcome assignment | Pre-adapter driver copy goes; two completion/assignment copies still execute at0x72ac7a/0x72b07d, three times in table. Four adapter action copies remain. | 0 |
| 5 | Borrow erased adapter; caller drops its cloned outer Arc | All four adapter action copies disappear. Driver still has two completion/assignment copies at0x72acdf/0x72b0e5. | 0 |
| 6 | Cold cycle-error formatter | Quick leaf113→110, below S2's112; map copies stay1124. Full first panel reduces ordinary flags35→1. | 0 |
| 7 | By-value input with borrowed adapter | Two driver copies remain, at0x72a6eb before dispatch and0x72b094 at assignment; three/four times in table/call-repeat. No named gate moves; reverted. | 1 |
| 8 | End the resource-free Abort placeholder lifetime before action extraction | Only the ordinary assignment remains at0x72b084, three/four times in table/call-repeat; no adapter action copy. Named gate advances. | 0 |

**Correction to the preliminary quick-check conclusion:** its console filter searched `drive_resuming_frames`, but that method is inlined into `resume_eval_request_from_checkpoint`. The raw summaries always retained its calls. The complete object/address ledger exposed the two surviving driver copies in the first supposedly final source at `0x72acff` and `0x72b105`, with3/3/4/3/5 calls per resume. These were not declared inherent or hidden in the total. Trial7 failed to reduce them; trial8 removed the extra completion temporary. The console filter was corrected, all old evidence was retained, and every final gate/measurement was rerun under `r6b/` after the repair. The final five-workload address ledger, not the quick console output, decides the copy gate.

The adapter return fix uses a private Action::DecodeFailure arm instead of Result<Action, DecodeFailure>, and borrows the outer adapter so the concrete typed frame can write directly to the erased return destination. Observed Action layout grows136→144 bytes, without a new allocation. The input place is replaced with Abort by the sole private adapter before decoding, including mismatch/error paths. The driver checks the structural failure before pop, drops both originating owners before finalization, and explicitly ends only the empty placeholder lifetime before assigning the next outcome. No public raw protocol or transit record is added. Tests retain all old assertions and check the placeholder postcondition on success, mismatch, Abort and an owned-string error. The final native stack and full behavior gates below use this source.

| Profile / shape | S2 B/level | Final B/level | Max success / first StackLimit |
| --- | --- | --- | --- |
| debug peel | 21424 | 19152.0 | 55 / 56 |
| debug plus | 37728 | 34896.0 | 30 / 31 |
| release peel | 8096 | 7312.0 | 143 / 144 |
| release plus | 14720 | 13744.0 | 76 / 77 |

The native guard stays1048576B and libtest’s thread stack is unchanged; RUST_MIN_STACK is absent. Only the scratch logical recursion ceiling is100000. Four heights and three repetitions per shape have zero slope residual and agree with the complete resident-frame sums, with zero outgoing-argument residue. Fresh-process bisections and the ±4/+5 neighborhoods agree without mixed heights. The unchanged fully qualified48-level test passes. Final stack binaries, source hashes, native PCs and prologue tables are preserved; no aborting probe was used.

#### debug prologues

| Function | S2 count peel / arithmetic | Final count | S2 bytes | Entry S3 bytes | R4 best bytes | R6 final bytes |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| evaluate | 1 / 2 | 1 / 2 | 312 | 312 | 312 | 312 |
| evaluate_profiled_fixed_point | 1 / 2 | 1 / 2 | 2072 | 2568 | 1816 | 2264 |
| evaluate_once_unstable_after_input_check_with_cycle_tracking | 1 / 2 | 1 / 2 | 184 | 184 | 184 | 184 |
| evaluate_once_in_accounting_mode | 1 / 2 | 1 / 2 | 2264 | 2264 | 2264 | 2264 |
| evaluate_inner_unstable | 1 / 2 | 1 / 2 | 1000 | 1000 | 1000 | 1000 |
| evaluate_call | 1 / 2 | 1 / 2 | 1288 | 856 | 856 | 696 |
| resume_call_after_head | 1 / 2 | 1 / 2 | 552 | 568 | 568 | 568 |
| resume_call_with_dispatch_plan | 2 / 3 | 1 / 2 | 1768 | 2840 | 2184 | 2056 |
| run_call_dispatch_stage | 1 / 1 | 1 / 1 | 1880 | 1656 | 1480 | 1480 |
| try_apply_down_values | 1 / 1 | 1 / 1 | 2088 | 2088 | 2088 | 2088 |
| try_apply_down_values_from_symbol | 1 / 2 | 1 / 2 | 3800 | 3800 | 3800 | 3800 |
| evaluate_definition_rhs | 1 / 1 | 1 / 1 | 1336 | 1336 | 1336 | 1336 |
| evaluate_with_option_value_scope | 1 / 1 | 1 / 1 | 1000 | 1000 | 1000 | 1000 |
| prepare_arguments_for_call_with_unevaluated_restore | 0 / 1 | 0 / 1 | 344 | 344 | 344 | 344 |
| prepare_arguments_for_call_with_unevaluated_restore::{{closure}} | 0 / 1 | 0 / 1 | 2200 | 2680 | 1736 | 1736 |
| evaluate_atom_fast_path | 0 / 1 | 0 / 1 | 424 | 424 | 424 | 424 |
| evaluate_call::{{closure}} | 0 / 0 | 0 / 0 | inlined / absent | 952 | inlined / absent | inlined / absent |

#### release prologues

| Function | S2 count peel / arithmetic | Final count | S2 bytes | Entry S3 bytes | R4 best bytes | R6 final bytes |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| evaluate_definition_rhs | 1 / 1 | 1 / 1 | 840 | 840 | 840 | 840 |
| try_apply_down_values_from_symbol | 1 / 2 | 1 / 2 | 1272 | 1272 | 1272 | 1272 |
| try_apply_down_values | 1 / 1 | 1 / 1 | 776 | 776 | 776 | 776 |
| run_call_dispatch_stage | 1 / 1 | 1 / 1 | 296 | 280 | 280 | 280 |
| resume_call_with_dispatch_plan | 2 / 3 | 1 / 2 | 584 | 920 | 808 | 824 |
| resume_call_after_head | 1 / 2 | 0 / 0 | 632 | inlined / absent | inlined / absent | inlined / absent |
| evaluate_inner_unstable | 1 / 2 | 1 / 2 | 1080 | 1656 | 1608 | 1304 |
| evaluate_once_unstable_after_input_check_with_cycle_tracking | 1 / 2 | 1 / 2 | 760 | 760 | 760 | 760 |
| evaluate | 1 / 2 | 1 / 2 | 840 | 904 | 840 | 872 |
| evaluate_with_option_value_scope | 1 / 1 | 1 / 1 | 312 | 312 | 312 | 312 |
| evaluate_atom_fast_path | 0 / 1 | 0 / 1 | 408 | 408 | 408 | 408 |
| prepare_arguments_for_call_with_unevaluated_restore | 0 / 1 | 0 / 1 | 984 | 1176 | 936 | 936 |

The four final candidate binaries were built serially with release codegen-units=1, separate allocation builds, and no test-continuation-slots observer. The four immutable S2 bases were reused without rebuilding; all eight hashes and the final source hashes were checked. The unchanged brief608 resume harness and all64 value controls pass on both arms. These are the r6b final binaries, not the preliminary r6 panel.

| Arm | Binary | SHA-256 |
| --- | --- | --- |
| base | rum-cli | b553a00e63e8af387e3141fe35764f54515a3e3154fd09b2756f911a2e87d9bf |
| base | rum-cli-alloc | f1fb356eb26f45dedaa09ad46e8d697dfde6bd10fbfe1aa12290f61dfcf6e9f1 |
| base | resume | 57e800bca2cca0d327b358f410d34ced30787410bdee6afe5838f9334a24e81f |
| base | resume-alloc | 73f279bfb5e13ab4daa34242a9d591e52d9d8012267197268e80f63d41ecc503 |
| candidate | rum-cli | b38b56750d3245d056bb762f1179cc37399377a8e72cde74c895615e72b323ab |
| candidate | rum-cli-alloc | 547a5aab968220650c7421ff10e15650dff878fa0e719980d6192e71f199492d |
| candidate | resume | 074063d0f7eab31d900638982720ac84ec3b843dc5ab94f3102613967e8b1bba |
| candidate | resume-alloc | b3d1fa13e4fb12a551e7d6c04b0edc68cc6d3ac4adb7408bf06223cd50cabd2c |

Allocations and bytes ran first:128 raw N/2N cells per arm. Fractional slopes below reflect the unchanged sources and amortization; equality is tested before formatting. Raw commands/stdout/stderr and parsed counters are under `r6b/perf/alloc/`. Resume allocation attribution follows the full table.

| Label | S2 alloc/N | S3 alloc/N | S2 bytes/N | S3 bytes/N |
| --- | --- | --- | --- | --- |
| lazy-range-take-small | 141 | 141 | 2714 | 2714 |
| lazy-range-normal-small | 157 | 157 | 3388 | 3388 |
| lazy-map-realize-small | 333 | 333 | 13740 | 13740 |
| lazy-select-realize-small | 236 | 236 | 6834 | 6834 |
| lazy-select-scan-small | 1471 | 1471 | 62793 | 62793 |
| lazy-nestlist-realize-small | 197 | 197 | 11316 | 11316 |
| lazy-foldlist-realize-small | 216 | 216 | 7604 | 7604 |
| lazy-drop-realize-small | 351 | 351 | 14204 | 14204 |
| lazy-first-small | 64 | 64 | 2868 | 2868 |
| lazy-part-small | 152 | 152 | 6182 | 6182 |
| lazy-transparent-adapters-small | 319 | 319 | 12876 | 12876 |
| lazy-table-construct-short | 63 | 63 | 1537 | 1537 |
| lazy-table-construct-full | 64 | 64 | 1449 | 1449 |
| lazy-table-realize-small | 251 | 251 | 9190 | 9190 |
| lazy-permutations-take-small | 29 | 29 | 1729 | 1729 |
| lazy-subsets-take-small | 29 | 29 | 1417 | 1417 |
| lazy-tuples-take-small | 29 | 29 | 1361 | 1361 |
| scan-no-input | 33 | 33 | 2536 | 2536 |
| optionvalue-three-no-input | 9 | 9 | 1128 | 1128 |
| replacerepeated-no-input | 57 | 57 | 7899 | 7899 |
| dot-three-no-input | 596 | 596 | 23504 | 23504 |
| differences-no-input | 13 | 13 | 704 | 704 |
| do-count-null | 0 | 0 | 0 | 0 |
| do-iterator-null | 0 | 0 | 32 | 32 |
| table-iterator-null | 0 | 0 | 32 | 32 |
| replaceall-simple | 5 | 5 | 1352 | 1352 |
| table-small | 22 | 22 | 977 | 977 |
| table-iterator-plus | 2 | 2 | 72 | 72 |
| table-iterator-pair-list | 2 | 2 | 136 | 136 |
| take-multispec-small | 109 | 109 | 5856 | 5856 |
| lazy-depth-map-build-8 | 16.001 | 16.001 | 1879.864 | 1879.864 |
| lazy-depth-map-first-8 | 188 | 188 | 11176 | 11176 |
| lazy-depth-map-eight-8 | 1464.001 | 1464.001 | 82776.864 | 82776.864 |
| lazy-depth-select-restart-8 | 327 | 327 | 16980 | 16980 |
| lazy-depth-drop-all-one-8 | 170 | 170 | 3128 | 3128 |
| lazy-depth-take-map-8 | 220 | 220 | 12144 | 12144 |
| lazy-depth-fold-through-source-8 | 743.001 | 743.001 | 58440.864 | 58440.864 |
| lazy-depth-nest-map-8 | 192 | 192 | 11964 | 11964 |
| lazy-depth-map-build-64 | 16.001 | 16.001 | 5911.864 | 5911.864 |
| lazy-depth-map-first-64 | 1364 | 1364 | 85320 | 85320 |
| lazy-depth-map-eight-64 | 10872.001 | 10872.001 | 647704.864 | 647704.864 |
| lazy-depth-select-restart-64 | 1671 | 1671 | 99412 | 99412 |
| lazy-depth-drop-all-one-64 | 1178 | 1178 | 17520 | 17520 |
| lazy-depth-take-map-64 | 1620 | 1620 | 93064 | 93064 |
| lazy-depth-fold-through-source-64 | 43334.001 | 43334.001 | 3402649.864 | 3402649.864 |
| lazy-depth-nest-map-64 | 1368 | 1368 | 86108 | 86108 |
| lazy-depth-map-build-256 | 16.001 | 16.001 | 19735.864 | 19735.864 |
| lazy-depth-map-first-256 | 5396 | 5396 | 339528 | 339528 |
| lazy-depth-map-eight-256 | 43128.001 | 43128.001 | 2584600.864 | 2584600.864 |
| lazy-depth-select-restart-256 | 6279 | 6279 | 382036 | 382036 |
| lazy-depth-drop-all-one-256 | 4634 | 4634 | 66864 | 66864 |
| lazy-depth-take-map-256 | 6420 | 6420 | 370504 | 370504 |
| lazy-depth-fold-through-source-256 | 689320.001 | 689320.001 | 54008857.864 | 54008857.864 |
| lazy-depth-nest-map-256 | 5400 | 5400 | 340316 | 340316 |
| literal-integer | 0 | 0 | 0 | 0 |
| downvalue-pattern-hit | 11 | 11 | 908 | 908 |
| module-local | 23.0001 | 23.0001 | 2163.6272 | 2163.6272 |
| block-local | 8 | 8 | 752 | 752 |
| matcher-single-wide-small | 11 | 11 | 287280 | 287280 |
| resume-table | 27.003 | 33.003 | 1742.456 | 2030.456 |
| resume-do-chain | 24 | 30 | 1634 | 2050 |
| resume-call-repeat | 40.007 | 47.007 | 4210.024 | 4522.024 |
| resume-cleanup | 22 | 28 | 1394 | 1746 |
| resume-fold | 41 | 49 | 2873 | 3337 |

Two full interleaved instruction pairs ran S2-1,S3-1,S2-2,S3-2:512 raw cells, all value-correct. Commands use taskset -c2 and perf stat -x, -e cpu_core/instructions/u with stdin /dev/null. The threshold is max(0.0005×S2 mean,S2 range,S3 range). Conservatively, either paired delta above that threshold triggers investigation: 5 labels, including 0 ordinary labels and 5 resume labels.

| Label | S2 slopes1/2 | S3 slopes1/2 | Mean delta I | Threshold I | Flag |
| --- | --- | --- | --- | --- | --- |
| lazy-range-take-small | 42245.09775 / 42207.21465 | 41912.20125 / 41862.83365 | -338.63875 | 49.3676 | no |
| lazy-range-normal-small | 54104.317 / 54097.90125 | 53553.0794 / 53552.41225 | -548.3633 | 27.0505545625 | no |
| lazy-map-realize-small | 178878.1997 / 178822.9798 | 176386.9967 / 176461.00655 | -2426.588125 | 89.425294875 | no |
| lazy-select-realize-small | 114552.72615 / 114853.57715 | 112361.9297 / 112561.848 | -2241.2628 | 300.851 | no |
| lazy-select-scan-small | 805257.25875 / 805198.5913 | 793727.24515 / 793735.52285 | -11496.541025 | 402.613962512 | no |
| lazy-nestlist-realize-small | 145303.22175 / 145463.62195 | 142886.97875 / 142924.74035 | -2477.5623 | 160.4002 | no |
| lazy-foldlist-realize-small | 112775.7788 / 112782.6464 | 110660.66635 / 110792.83255 | -2052.46315 | 132.1662 | no |
| lazy-drop-realize-small | 191545.41275 / 191486.4454 | 188695.62575 / 188697.56175 | -2819.335325 | 95.7579645375 | no |
| lazy-first-small | 41205.8228 / 41208.21825 | 40654.97685 / 40605.63425 | -576.714975 | 49.3426 | no |
| lazy-part-small | 88307.39375 / 88292.5233 | 87201.24 / 87219.54035 | -1089.56835 | 44.1499792625 | no |
| lazy-transparent-adapters-small | 181508.8211 / 181435.85385 | 178724.18415 / 178726.44595 | -2747.022425 | 90.7361687375 | no |
| lazy-table-construct-short | 23561.3864 / 23537.4014 | 23338.52835 / 23372.1807 | -194.039375 | 33.65235 | no |
| lazy-table-construct-full | 24146.9052 / 24120.2337 | 23848.9603 / 23894.12785 | -262.025375 | 45.16755 | no |
| lazy-table-realize-small | 117156.65725 / 117157.502 | 115728.95065 / 115753.93405 | -1415.637275 | 58.5785398125 | no |
| lazy-permutations-take-small | 28197.77575 / 28199.2898 | 27818.6217 / 27848.98695 | -364.72845 | 30.36525 | no |
| lazy-subsets-take-small | 25358.11455 / 25371.57305 | 25043.45775 / 25013.86025 | -336.1848 | 29.5975 | no |
| lazy-tuples-take-small | 26581.003 / 26629.2037 | 26264.39575 / 26240.1694 | -352.820775 | 48.2007 | no |
| scan-no-input | 29684.5483 / 29681.29025 | 28949.20985 / 28950.79165 | -732.918525 | 14.8414596375 | no |
| optionvalue-three-no-input | 9589.1259 / 9590.4774 | 9465.36315 / 9466.9522 | -123.643975 | 4.794900825 | no |
| replacerepeated-no-input | 39174.4922 / 39174.36125 | 39040.31175 / 39040.56495 | -133.988375 | 19.5872133625 | no |
| dot-three-no-input | 254199.2807 / 254199.79925 | 249927.75055 / 249927.6766 | -4271.8264 | 127.099769988 | no |
| differences-no-input | 10037.77405 / 10037.5433 | 9908.87305 / 9908.7855 | -128.8294 | 5.0188293375 | no |
| do-count-null | 536.86115 / 537.0055 | 535.03545 / 534.9016 | -1.9648 | 0.2684666625 | no |
| do-iterator-null | 1154.126 / 1153.99675 | 1151.97935 / 1151.8791 | -2.13215 | 0.5770306875 | no |
| table-iterator-null | 387.0865 / 387.04515 | 386.95415 / 386.95685 | -0.110325 | 0.1935329125 | no |
| replaceall-simple | 11458.48745 / 11380.20655 | 11353.27325 / 11353.3564 | -66.032175 | 78.2809 | no |
| table-small | 14633.7463 / 14681.6726 | 14509.1648 / 14508.9554 | -148.64935 | 47.9263 | no |
| table-iterator-plus | 563.1878 / 563.35475 | 563.1831 / 563.23605 | -0.0617000000001 | 0.2816356375 | no |
| table-iterator-pair-list | 1640.81195 / 1640.67505 | 1640.6822 / 1640.78415 | -0.0103250000002 | 0.82037175 | no |
| take-multispec-small | 49938.71935 / 49939.27355 | 49772.0907 / 49771.7283 | -167.08695 | 24.969498225 | no |
| lazy-depth-map-build-8 | 18127.483 / 18124.66 | 17814.284 / 17814.839 | -311.51 | 9.06303575 | no |
| lazy-depth-map-first-8 | 130372.104 / 130402.331 | 128734.325 / 128745.273 | -1647.4185 | 65.19360875 | no |
| lazy-depth-map-eight-8 | 959569.324 / 959661.844 | 946179.27 / 946167.867 | -13442.0155 | 479.807792 | no |
| lazy-depth-select-restart-8 | 179119.597 / 179381.612 | 177765.675 / 177765.676 | -1484.929 | 262.015 | no |
| lazy-depth-drop-all-one-8 | 46664.654 / 46663.603 | 46359.807 / 46357.681 | -305.3845 | 23.33206425 | no |
| lazy-depth-take-map-8 | 138951.913 / 138949.594 | 136917.474 / 136883.453 | -2050.29 | 69.47537675 | no |
| lazy-depth-fold-through-source-8 | 589546.446 / 589552.606 | 581600.475 / 581491.726 | -8003.4255 | 294.774763 | no |
| lazy-depth-nest-map-8 | 141003.7 / 141398.448 | 138541.596 / 138750.41 | -2555.071 | 394.748 | no |
| lazy-depth-map-build-64 | 30135.331 / 30118.158 | 29818.735 / 29816.279 | -309.2375 | 17.173 | no |
| lazy-depth-map-first-64 | 946187.009 / 944594.043 | 930349.712 / 930348.482 | -15041.429 | 1592.966 | no |
| lazy-depth-map-eight-64 | 7422080.426 / 7422351.537 | 7308941.228 / 7308599.684 | -113445.5255 | 3711.10799075 | no |
| lazy-depth-select-restart-64 | 981707.097 / 975693.337 | 965752.623 / 969126.335 | -11260.738 | 6013.76 | no |
| lazy-depth-drop-all-one-64 | 327585.057 / 327583.7 | 327277.737 / 327277.878 | -306.571 | 163.79218925 | no |
| lazy-depth-take-map-64 | 1007501.138 / 1007405.21 | 993182.464 / 993238.304 | -14242.79 | 503.726587 | no |
| lazy-depth-fold-through-source-64 | 33997047.791 / 33998183.789 | 33539136.924 / 33544738.142 | -455678.257 | 16998.807895 | no |
| lazy-depth-nest-map-64 | 957228.983 / 958617.943 | 942484.659 / 944152.834 | -14604.7165 | 1668.175 | no |
| lazy-depth-map-build-256 | 70766.13 / 70762.578 | 70486.52 / 70469.674 | -286.257 | 35.382177 | no |
| lazy-depth-map-first-256 | 3723177.091 / 3722823.07 | 3666659.907 / 3666660.516 | -56339.869 | 1861.50004025 | no |
| lazy-depth-map-eight-256 | 29546204.161 / 29546113.091 | 29084731.312 / 29086540.392 | -460522.774 | 14773.079313 | no |
| lazy-depth-select-restart-256 | 3664231.637 / 3665397.065 | 3653996.14 / 3628165.836 | -23733.363 | 25830.304 | no |
| lazy-depth-drop-all-one-256 | 2218610.197 / 2218619.315 | 2218306.581 / 2218310.309 | -306.311 | 1109.307378 | no |
| lazy-depth-take-map-256 | 3985882.898 / 3985901.942 | 3929392.065 / 3929188.058 | -56602.3585 | 1992.94621 | no |
| lazy-depth-fold-through-source-256 | 530578692.023 / 530588080.283 | 523276558.054 / 523313109.562 | -7288552.345 | 265291.693077 | no |
| lazy-depth-nest-map-256 | 3733562.267 / 3726880.888 | 3677238.902 / 3670577.266 | -56313.4935 | 6681.379 | no |
| literal-integer | 294.00725 / 293.97785 | 291.89105 / 291.77315 | -2.16045 | 0.146996275 | no |
| downvalue-pattern-hit | 13949.4149 / 13949.8585 | 13664.68745 / 13665.0745 | -284.755725 | 6.97481835 | no |
| module-local | 22653.2982 / 22651.95 | 22426.34165 / 22428.40295 | -225.2518 | 11.32631205 | no |
| block-local | 14671.792 / 14671.7832 | 14446.0008 / 14445.9811 | -225.79665 | 7.3358938 | no |
| matcher-single-wide-small | 760208.732 / 760181.254 | 760156.101 / 760141.4 | -46.2424999999 | 380.0974965 | no |
| resume-table | 15722.797 / 15687.018 | 18274.944 / 18256.724 | 2560.9265 | 35.779 | yes |
| resume-do-chain | 15156.812 / 15130.381 | 17664.689 / 17648.683 | 2513.0895 | 26.431 | yes |
| resume-call-repeat | 24579.856 / 24580.951 | 27282.432 / 27185.29 | 2653.4575 | 97.142 | yes |
| resume-cleanup | 14244.556 / 14277.002 | 16722.693 / 16774.892 | 2488.0135 | 52.199 | yes |
| resume-fold | 24429.39 / 24415.264 | 27411.295 / 27417.187 | 2991.914 | 14.126 | yes |

Every flagged label received five fresh N/2N perf slopes per arm and three fresh N/2N Callgrind pairs per arm (six processes per arm). All59 ordinary labels received Callgrind for the copy gate; the three deterministic leaves also received three pairs regardless of flag status. Instrumentation N is prebudgeted to approximately20million instructions, clamped to1..2000; hardware repeats retain the original N. All raw cells are retained.

| Label | S2 mean/range | S3 mean/range | Delta I | Delta % | Above repeat threshold |
| --- | --- | --- | --- | --- | --- |
| resume-table | 15731.1346 / 34.627 | 18280.1112 / 51.728 | 2548.9766 | 16.2033868809 | yes |
| resume-do-chain | 15187.8464 / 131.997 | 17683.1208 / 73.192 | 2495.2744 | 16.4294155622 | yes |
| resume-call-repeat | 24676.5862 / 304.589 | 27206.258 / 75.943 | 2529.6718 | 10.2513037237 | yes |
| resume-cleanup | 14264.5446 / 98.495 | 16746.2276 / 50.094 | 2481.683 | 17.3975620645 | yes |
| resume-fold | 24429.4576 / 16.332 | 27413.4094 / 111.776 | 2983.9518 | 12.2145642726 | yes |

Each per-function Callgrind sum is checked against its raw summary slope. The mean difference is compared with the larger arm range because hashing is seeded per process. A positive mean within that range is not by itself called added work; address evidence is considered separately.

| Ordinary label | CG N | S2 mean | S3 mean | Delta I | Larger range | Added by total |
| --- | --- | --- | --- | --- | --- | --- |

| Resume (N200) | S2 mean | S3 mean | Delta I | Larger range | Added by total |
| --- | --- | --- | --- | --- | --- |
| table | 15972.585 | 18555.7083333 | 2583.12333333 | 212.435 | yes |
| do-chain | 15219.26 | 17773.1216667 | 2553.86166667 | 36.105 | yes |
| call-repeat | 25371.3633333 | 28040.2 | 2668.83666667 | 82.235 | yes |
| cleanup | 14253.07 | 16721.6816667 | 2468.61166667 | 38.455 | yes |
| fold | 24525.7733333 | 27545.775 | 3020.00166667 | 48.08 | yes |

The per-label copy counts below sum every caller of the object-qualified resolved libc memcpy IFUNC, not only the named runtime symbols. The addressed call windows retain copy widths where an immediate proves them. This counts calls, not all memory traffic; inline payload moves are separately reviewed in the ordinary/resume address notes. The full59-label object/address ledgers include allocator and libc code.

| Ordinary label | S2 memcpy/N | S3 memcpy/N | Delta | At most S2 |
| --- | --- | --- | --- | --- |
| lazy-range-take-small | 129 | 129 | 0 | True |
| lazy-range-normal-small | 143 | 143 | 0 | True |
| lazy-map-realize-small | 295 | 295 | 0 | True |
| lazy-select-realize-small | 221 | 221 | 0 | True |
| lazy-select-scan-small | 1351 | 1351 | 0 | True |
| lazy-nestlist-realize-small | 177 | 177 | 0 | True |
| lazy-foldlist-realize-small | 203 | 203 | 0 | True |
| lazy-drop-realize-small | 311 | 311 | 0 | True |
| lazy-first-small | 59 | 59 | 0 | True |
| lazy-part-small | 137 | 137 | 0 | True |
| lazy-transparent-adapters-small | 285 | 285 | 0 | True |
| lazy-table-construct-short | 50 | 50 | 0 | True |
| lazy-table-construct-full | 51 | 51 | 0 | True |
| lazy-table-realize-small | 211 | 211 | 0 | True |
| lazy-permutations-take-small | 24 | 24 | 0 | True |
| lazy-subsets-take-small | 24 | 24 | 0 | True |
| lazy-tuples-take-small | 24 | 24 | 0 | True |
| scan-no-input | 51 | 51 | 0 | True |
| optionvalue-three-no-input | 7 | 7 | 0 | True |
| replacerepeated-no-input | 29 | 29 | 0 | True |
| dot-three-no-input | 465 | 465 | 0 | True |
| differences-no-input | 9 | 9 | 0 | True |
| do-count-null | 1 | 1 | 0 | True |
| do-iterator-null | 2 | 2 | 0 | True |
| table-iterator-null | 0 | 0 | 0 | True |
| replaceall-simple | 23 | 23 | 0 | True |
| table-small | 13 | 13 | 0 | True |
| table-iterator-plus | 1 | 1 | 0 | True |
| table-iterator-pair-list | 0 | 0 | 0 | True |
| take-multispec-small | 47 | 47 | 0 | True |
| lazy-depth-map-build-8 | 25 | 25 | 0 | True |
| lazy-depth-map-first-8 | 172 | 172 | 0 | True |
| lazy-depth-map-eight-8 | 1219 | 1219 | 0 | True |
| lazy-depth-select-restart-8 | 281 | 281 | 0 | True |
| lazy-depth-drop-all-one-8 | 155 | 155 | 0 | True |
| lazy-depth-take-map-8 | 188 | 188 | 0 | True |
| lazy-depth-fold-through-source-8 | 696 | 696 | 0 | True |
| lazy-depth-nest-map-8 | 178 | 178 | 0 | True |
| lazy-depth-map-build-64 | 25 | 25 | 0 | True |
| lazy-depth-map-first-64 | 1124 | 1124 | 0 | True |
| lazy-depth-map-eight-64 | 8835 | 8835 | 0 | True |
| lazy-depth-select-restart-64 | 1401 | 1401 | 0 | True |
| lazy-depth-drop-all-one-64 | 995 | 995 | 0 | True |
| lazy-depth-take-map-64 | 1252 | 1252 | 0 | True |
| lazy-depth-fold-through-source-64 | 37549 | 37549 | 0 | True |
| lazy-depth-nest-map-64 | 1130 | 1130 | 0 | True |
| lazy-depth-map-build-256 | 25 | 25 | 0 | True |
| lazy-depth-map-first-256 | 4388 | 4388 | 0 | True |
| lazy-depth-map-eight-256 | 34949 | 34949 | 0 | True |
| lazy-depth-select-restart-256 | 5241 | 5241 | 0 | True |
| lazy-depth-drop-all-one-256 | 3875 | 3875 | 0 | True |
| lazy-depth-take-map-256 | 4900 | 4900 | 0 | True |
| lazy-depth-fold-through-source-256 | 592431 | 592431 | 0 | True |
| lazy-depth-nest-map-256 | 4394 | 4394 | 0 | True |
| literal-integer | 1 | 1 | 0 | True |
| downvalue-pattern-hit | 18 | 18 | 0 | True |
| module-local | 32 | 32 | 0 | True |
| block-local | 29 | 29 | 0 | True |
| matcher-single-wide-small | 15 | 15 | 0 | True |

| Leaf | S2 evaluate I (3 samples) | S3 evaluate I (3 samples) | Whole slope mean delta | Evaluate ≤S2 |
| --- | --- | --- | --- | --- |
| literal-integer | 112 / 112 / 112 | 110 / 110 / 110 | -1.15933333333 | True |
| do-count-null | 112 / 112 / 112 | 110 / 110 / 110 | -1.30783333333 | True |
| do-iterator-null | 112 / 112 / 112 | 110 / 110 / 110 | -1.36983333333 | True |

The ordinary allocation, copy and deterministic-leaf gates pass. None of the 59 ordinary labels was flagged by the final two-pair instruction panel, so no ordinary flag requires a clearance claim. The full ordinary address ledger retains every self instruction and call edge, including allocator and libc code; its per-label sums reconcile to the raw Callgrind slopes. The unflagged labels have one N/2N pair per arm, except the three deterministic leaves, which have three. This is not a claim that one process pair measures hash-dependent totals without noise.

All three leaf paths execute 112 → 110 self instructions in Session::evaluate in every sample. The containing CLI function moves from S2 entry 0x71e5a0 to candidate entry 0x7201a0. The executed opcode-count difference is −3 lea, +2 mov, −1 jmp. The candidate retains the lexical-profile flag address in %r14 across its loads. The R5 cycle-error stack-address hoist is absent. The isolated source trials in the preceding table establish the M0561/cold-formatting cause; compiler register allocation heuristics are not asserted as a fact. Full addressed instruction sequences and edits are in ordinary-callgrind/leaf-addresses.json, with all three samples in leaf-gates.json.

The two kinds of removed ordinary aggregate transfer are distinct: caller-result rebinding at the 136-byte macro sites, and the three-record profile closure at the 168-byte sites. Direct profile recording preserves order and balance in every branch. All 59 final memcpy counts are exactly S2, not merely below its limit. Inline loads/stores are retained in the complete address ledgers; a zero memcpy delta is not presented as zero memory traffic. The final ledgers are under address-ledgers/ordinary/, and the addressed copy call windows are in ordinary-callgrind/.

Three objectless Callgrind addresses per ordinary arm have no resolvable ELF instructions: S2 0x4070118/0x4070128/0x4070160 and candidate 0x4071118/0x4071128/0x4071160. Their corresponding count maps are exactly equal on all 59 labels, so their signed contribution is zero. Their identity is not guessed and no disassembly is invented. No cold continuation-fault helper instruction executes in any ordinary profile.

The following resume evidence separates the passing targeted-copy gates from the remaining performance defects and attribution limits. Counts are per N resume unless explicitly labelled whole program. All five workload totals exceed the larger Callgrind arm range; timing cannot clear them. The final source keeps the global Legacy wire exactly as ruled by the director.

The raw frame-call census identifies the before/after concrete frames, including constant terminal work. Method suffixes below abbreviate only the crate/module prefix. A zero slope does not mean the method never executes. The base/candidate N and 2N counts are from the first 200/400 pair; fixed allocation checks additionally cover all three pairs.

| table / arm | Concrete method | N calls | 2N calls | Calls/resume |
| --- | --- | --- | --- | --- |
| base | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <kernel_data::control_data::branching_loops::iteration::driver::resume_frames::ResumeTableIterationsSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| base | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <kernel_data::control_data::branching_loops::iteration::WithIterationProductScopeSuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |
| candidate | <suspension::typed::adapter::TypedAdapter<F> as suspension::typed::adapter::ErasedFrame>::resume | 603 | 1203 | 3 |
| candidate | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::typed::TypedFrame>::resume | 201 | 401 | 1 |
| candidate | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::typed::TypedFrame>::resume | 201 | 401 | 1 |
| candidate | <kernel_data::control_data::branching_loops::iteration::driver::resume_frames::ResumeTableIterationsSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| candidate | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::typed::TypedFrame>::resume | 201 | 401 | 1 |
| candidate | <kernel_data::control_data::branching_loops::iteration::WithIterationProductScopeSuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |

| do-chain / arm | Concrete method | N calls | 2N calls | Calls/resume |
| --- | --- | --- | --- | --- |
| base | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <kernel_data::control_data::branching_loops::iteration::driver::resume_frames::EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| base | <kernel_data::control_data::branching_loops::iteration::driver::resume_frames::ResumeDoIterationsSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| base | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <kernel_data::control_data::branching_loops::iteration::WithIterationProductScopeSuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |
| candidate | <suspension::typed::adapter::TypedAdapter<F> as suspension::typed::adapter::ErasedFrame>::resume | 603 | 1203 | 3 |
| candidate | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::typed::TypedFrame>::resume | 201 | 401 | 1 |
| candidate | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::typed::TypedFrame>::resume | 201 | 401 | 1 |
| candidate | <kernel_data::control_data::branching_loops::iteration::driver::resume_frames::EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| candidate | <kernel_data::control_data::branching_loops::iteration::driver::resume_frames::ResumeDoIterationsSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| candidate | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::typed::TypedFrame>::resume | 201 | 401 | 1 |
| candidate | <kernel_data::control_data::branching_loops::iteration::WithIterationProductScopeSuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |

| call-repeat / arm | Concrete method | N calls | 2N calls | Calls/resume |
| --- | --- | --- | --- | --- |
| base | <eval_call_prepare::resume_frames::ResumeSuspendedCallArgumentPreparationSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| base | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| base | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <eval_call_prepare::resume_frames::FinishPreparedCallArgumentsFinalizer as suspension::core::PureFrameFinalizer>::finish | 1 | 1 | 0 |
| candidate | <suspension::typed::adapter::TypedAdapter<F> as suspension::typed::adapter::ErasedFrame>::resume | 802 | 1602 | 4 |
| candidate | <eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame as suspension::typed::TypedFrame>::resume | 200 | 400 | 1 |
| candidate | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::typed::TypedFrame>::resume | 200 | 400 | 1 |
| candidate | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::typed::TypedFrame>::resume | 200 | 400 | 1 |
| candidate | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::typed::TypedFrame>::resume | 201 | 401 | 1 |
| candidate | <eval_call_prepare::resume_frames::FinishPreparedCallArgumentsFinalizer as suspension::typed::SlotFinalizer>::finish | 1 | 1 | 0 |
| candidate | <eval_core::call::PreparedDispatchFrame as suspension::typed::TypedFrame>::resume | 1 | 1 | 0 |
| candidate | <suspension::typed::adapter::FinalizerAdapter<F> as suspension::typed::action::ErasedFinalizer>::finish | 1 | 1 | 0 |

| cleanup / arm | Concrete method | N calls | 2N calls | Calls/resume |
| --- | --- | --- | --- | --- |
| base | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::core::SuspensionFrame>::resume | 203 | 403 | 1 |
| base | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::core::SuspensionFrame>::resume | 203 | 403 | 1 |
| base | <kernel_data::control_data::branching_loops::iteration::driver::resume_frames::ResumeDoIterationsSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| base | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::core::SuspensionFrame>::resume | 203 | 403 | 1 |
| base | <kernel_data::control_data::branching_loops::iteration::WithIterationProductScopeSuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |
| base | <kernel_data::control_data::branching_loops::resume_frames::ResumeCompoundExpressionFromIndexSuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |
| base | <kernel_data::control_data::cleanup::DriveWithCleanupPrimarySuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |
| candidate | <suspension::typed::adapter::TypedAdapter<F> as suspension::typed::adapter::ErasedFrame>::resume | 609 | 1209 | 3 |
| candidate | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::typed::TypedFrame>::resume | 203 | 403 | 1 |
| candidate | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::typed::TypedFrame>::resume | 203 | 403 | 1 |
| candidate | <kernel_data::control_data::branching_loops::iteration::driver::resume_frames::ResumeDoIterationsSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| candidate | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::typed::TypedFrame>::resume | 203 | 403 | 1 |
| candidate | <kernel_data::control_data::branching_loops::iteration::WithIterationProductScopeSuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |
| candidate | <kernel_data::control_data::branching_loops::resume_frames::ResumeCompoundExpressionFromIndexSuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |
| candidate | <kernel_data::control_data::cleanup::DriveWithCleanupPrimarySuspensionFrame as suspension::core::SuspensionFrame>::resume | 1 | 1 | 0 |

| fold / arm | Concrete method | N calls | 2N calls | Calls/resume |
| --- | --- | --- | --- | --- |
| base | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::core::SuspensionFrame>::resume | 401 | 801 | 2 |
| base | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::core::SuspensionFrame>::resume | 401 | 801 | 2 |
| base | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::core::SuspensionFrame>::resume | 201 | 401 | 1 |
| base | <scoping::callable::TryApplyFunctionSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| base | <rum_stdlib::functional_ops::sequence::fold::plain::ResumeFoldFromIndexSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| candidate | <suspension::typed::adapter::TypedAdapter<F> as suspension::typed::adapter::ErasedFrame>::resume | 1003 | 2003 | 5 |
| candidate | <eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as suspension::typed::TypedFrame>::resume | 401 | 801 | 2 |
| candidate | <resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as suspension::typed::TypedFrame>::resume | 401 | 801 | 2 |
| candidate | <eval_dispatch::RunCallDispatchStageSuspensionFrame as suspension::typed::TypedFrame>::resume | 201 | 401 | 1 |
| candidate | <scoping::callable::TryApplyFunctionSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |
| candidate | <rum_stdlib::functional_ops::sequence::fold::plain::ResumeFoldFromIndexSuspensionFrame as suspension::core::SuspensionFrame>::resume | 200 | 400 | 1 |

The four physical adapter bodies with positive slopes are listed next. F0526/F0527 have identical resume bodies merged by the compiler at the observed ResumeFixedPointFromSuspensionFrame symbol; the physical profile alone does not distinguish them. F0273 is replaced by F0272 as the owned fresh-state transition. PreparedDispatchFrame is a fifth active adapter in call-repeat, but runs once for the entire program and therefore has zero slope.

| Input / concrete frame | Adapter entry | Concrete resume entry | Calls: table / do / call / cleanup / fold | Adapter self I/call |
| --- | --- | --- | --- | --- |
| Option<Expr> / F0275 | 0x756b50 | 0x579350 | 1 / 1 / 1 / 1 / 2 | 63 |
| Expr / F0278 | 0x7572f0 | 0x584570 | 1 / 1 / 1 / 1 / 1 | 53 |
| Expr / F0526/F0527 physical body | 0x7576f0 | 0x6f0ef0 | 1 / 1 / 1 / 1 / 2 | 53 |
| Expr / F0272 (S2 F0273) | 0x757880 | 0x572580 | 0 / 0 / 1 / 0 / 0 | 53 |
| PreparedCallArguments / PreparedDispatchFrame | 0x757480 | see fixed raw call graph | 0 slope; 1 whole call-repeat program | 70 |

Each of the four positive-slope adapters now writes the concrete action directly into the erased return destination: no libc memcpy call remains in any of them. The constant fifth adapter also has no such call. The erased signature is private &self plus &mut Result<Slot, EvalError>, returning Action directly; DecodeFailure is a private Action variant. Action grows 136 → 144 bytes in this binary, without another allocation. The sole implementation replaces its input with resource-free Abort before decoding; the caller drops its cloned outer Arc immediately after resume and handles decode failure before popping the queue owner. This preserves the fault latch and both-owner-before-finalizer ordering.

The typed driver is inlined into resume_eval_request_from_checkpoint (candidate entry 0x729f70). Both extra driver copies identified by redispatch 6 are gone. Only the common outcome assignment at 0x72b084 remains: 136 bytes, 3/3/4/3/5 calls per resume. The S2 common loop assignment is 0x72a565, 4/5/4/4/7 calls including legacy roles. Other candidate copies in this containing function, 0x72c348 (136 B) and 0x72c9fb (152 B), occur once per resume request outside the typed loop. Ending the empty Abort placeholder lifetime before extracting the next action removes the redundant completion temporary. These observations do not claim all output transfers vanish.

Concrete frame transfer comparison: the remaining F0526/F0527 input copy is 0x6f0fa9 (136 B, 1/1/1/1/2 per resume), corresponding to S2 0x6f04c8. S2 output copy 0x6f05f9 is gone. The F0275 S2 copy at 0x579d5e and F0278 copies at 0x584691/0x5846a7 are also gone. F0278 uses inline stores for its typed result, all retained in paired-frame-differences.json; they are not counted as no work merely because there is no memcpy call. Typical concrete-frame self costs are F0526/F0527 82 → 105, F0275 78 → 103 and F0278 37 → 68. Those increases include replacing library-copy work (26 I per 136-byte copy in these paths) and ABI/mode work. F0273 102 → F0272 328 includes an inlined helper body moved into the method; its 226-I difference is not all new semantic work. Fold takes a different F0275 branch on one of its two calls. Complete frame self costs, inline instructions and callee counts are in semantic-costs.json, paired-frame-differences.json and call-edge-costs.{json,tsv}, all under candidate/.

The per-resume allocation-count rule passes: extra allocations are the typed-frame count plus the three strict Some-wire allocations. Each extra adapter Arc is 24 B (16 B counters plus 8 B inner Arc pointer). The Some wire is 120 B at its three allocation sites: 32 B single argument buffer at 0x7523d3, 48 B SharedExpr at 0x79a2fe, and 40 B SharedArgs at 0x79a55d. F0278 calls its encoder at 0x58464d once per resume. Symbol interning/hashing is included as mode work, per the director ruling. The outer-Arc allocation sites are 0x10b06d, 0x75913b, 0x7592ca and 0x757b83; the last is call-repeat only.

| Workload | Typed frames | Extra alloc/resume | Extra B/resume | Outer Arc B | Some B | Queue residual B | Inferred cumulative capacity |
| --- | --- | --- | --- | --- | --- | --- | --- |
| table | 3 | 6 | 288 | 72 | 120 | 96 | 12 |
| do-chain | 3 | 6 | 416 | 72 | 120 | 224 | 28 |
| call-repeat | 4 | 7 | 312 | 96 | 120 | 96 | 12 |
| cleanup | 3 | 6 | 352 | 72 | 120 | 160 | 20 |
| fold | 5 | 8 | 464 | 120 | 120 | 224 | 28 |

The residual bytes are consistent with mixed-queue entries growing 16 → 24 B: 8 B times the capacities in the table. These cumulative capacities are inferred from the measured residual, not independently sampled allocation arguments. They retain the director-accepted inference status. No second allocation beyond frame + Some appears in any N/2N slope.

Whole-program counts reveal an additional defect hidden by those slopes. FinalizerAdapter::finish (entry 0x757c10) executes Box::new(self.0); it allocates 56 B at 0x757c56, with size immediate 0x38 at 0x757c4c. Every one of the six call-repeat processes executes this allocation once (38 inclusive allocator instructions), so N/2N subtraction cancels it. It is not elided or proven inherent. Payload transfers at 0x757c20–0x757c3e and 0x757c65–0x757c84 move all 56 B through a stack temporary to the new Box (16 memory instructions); the old Box is freed at 0x757d5b. The concrete finalizer call is 0x757c8d and its PreparedLegacy encoding is 0x757caf. This extra reallocation and payload copying remain unfixed and are reported as defects, separately from the four corrected frame-adapter bodies.

| Call-repeat sample | N | Finalizer allocation address | Whole-process calls | Allocator inclusive I |
| --- | --- | --- | --- | --- |
| 1 | 200 | 0x757c56 | 1 | 38 |
| 1 | 400 | 0x757c56 | 1 | 38 |
| 2 | 200 | 0x757c56 | 1 | 38 |
| 2 | 400 | 0x757c56 | 1 | 38 |
| 3 | 200 | 0x757c56 | 1 | 38 |
| 3 | 400 | 0x757c56 | 1 | 38 |

All 30 raw resume allocation-count cells (five workloads × three pairs × N/2N) reconcile exactly to outer adapters + 3 × Some encodings + two one-time interning allocations + the finalizer rebox where present. The one-time mode allocations are the 24-byte InternalContinuationSome text at 0x797305 and 40-byte symbol data at 0x79754e, each +1 relative to S2. The extra constant outer adapter at 0x7594a6 belongs to PreparedDispatchFrame; its inner allocation at 0x759423 is 104 B versus the former 128 B at S2 0x1050ad. Exact raw counts, addresses and the fifth adapter are in candidate/fixed-allocation-reconciliation.json.

| Workload | All outer adapters | All Some encodings | Fixed intern allocations | Fixed rebox | Total extra allocations |
| --- | --- | --- | --- | --- | --- |
| table | 3N + 3 | N + 1 | 2 | 0 | 6N + 8 |
| do-chain | 3N + 3 | N + 1 | 2 | 0 | 6N + 8 |
| call-repeat | 4N + 2 | N | 2 | 1 | 7N + 5 |
| cleanup | 3N + 9 | N + 3 | 2 | 0 | 6N + 20 |
| fold | 5N + 3 | N + 1 | 2 | 0 | 8N + 8 |

| Workload | N | Raw extra allocations | Count intercept | Raw extra bytes | Byte intercept |
| --- | --- | --- | --- | --- | --- |
| resume-table | 1000 | 6008 | 8 | 288261 | 261 |
| resume-table | 2000 | 12008 | 8 | 576261 | 261 |
| resume-do-chain | 1000 | 6008 | 8 | 416261 | 261 |
| resume-do-chain | 2000 | 12008 | 8 | 832261 | 261 |
| resume-call-repeat | 1000 | 7005 | 5 | 312149 | 149 |
| resume-call-repeat | 2000 | 14005 | 5 | 624149 | 149 |
| resume-cleanup | 1000 | 6020 | 20 | 352709 | 709 |
| resume-cleanup | 2000 | 12020 | 20 | 704709 | 709 |
| resume-fold | 1000 | 8008 | 8 | 464261 | 261 |
| resume-fold | 2000 | 16008 | 8 | 928261 | 261 |

The 261-B intercept in table/do/fold is consistent with 3 × 24 B outer adapters + 120 B Some + 64 B initial interning + 5 B from the longer executable path in the unchanged std::env::args() collection. Call-repeat gives 2 × 24 + 56 + 64 + 5 − 24 = 149 B, with the last term the directly observed initial-frame size reduction. Cleanup gives 9 × 24 + 3 × 120 + 64 + 5 + 64 = 709 B; its remaining 64 B is consistent with 8 × 8 queue capacity. Path-length and residual-queue byte explanations are not independent allocator-argument samples; the count reconciliation above is exact raw call evidence.

Instruction attribution names the four permitted classes without assigning every remaining instruction to them by assumption. The Expr adapters execute 53 self instructions: 37 mode decode/result-layout instructions, 13 adapter dispatch/ABI instructions and 3 inner-Arc clone instructions. The Option adapter executes 63: 42 mode, 18 dispatch/ABI and 3 clone. Each executed address and its count is listed in candidate/address-partition.json. Example clone addresses are 0x75777b/0x75777e/0x757782 for the fixed-point adapter and 0x756bdf/0x756be2/0x756be6 for Option. Cold mismatch/error instructions are not exercised by these normal profiles and are not assigned a measured cost.

The strict Option decoder call moves from S2 0x579730 to candidate 0x756bc2; calls are 1/1/1/1/2, inclusive costs 22/22/22/22/44 → 280/280/280/280/302, hence +258 I in every workload. The Some encoder root 0x58464d includes interning, hashing and all three wire allocations. The adapter dispatch call is 0x72a6e0, 3/3/4/3/5 per resume. Destruction roots S2 0x48fb8a → candidate 0x48ff7a resolve to each concrete frame/adapter drop glue; inner frame destruction is retained and only the subtree difference is counted. Queue roots are insertion widening at 0x74b20c, append at 0x72b6ff and mixed legacy pop at 0x72a95e, with final empty-queue destruction 0x40aafd → 0x40b01d. The append root contains its own widen call 0xd9b61; that child is not added twice.

First accounting: every self-address row is assigned exactly once. Directly identified mode/adapter/Arc/queue instructions are separated from a reported remainder. The signed category totals sum exactly to each raw mean delta. Shared callees of known costs, moved bodies, baseline credits, hash variation and extra helper ABI work remain in that remainder. It is neither all inherent nor all new defects. The full additive ledger is candidate/address-partition.tsv (object, physical function, address, assembly and five counts).

| Direct self-address category | table | do-chain | call-repeat | cleanup | fold |
| --- | --- | --- | --- | --- | --- |
| mode | 199 | 199 | 236 | 199 | 278 |
| adapter dispatch | 44 | 44 | 57 | 44 | 75 |
| second Arc | 21 | 21 | 28 | 21 | 35 |
| mixed queue | 490 | 526 | 517 | 490 | 657 |
| reported remainder | 1829.12333333 | 1763.86166667 | 1830.83666667 | 1714.61166667 | 1975.00166667 |

Second accounting: disjoint known call subtrees include the mode and allocator callees excluded from the direct self-address categories. They are combined with adapter self costs, without double-counting nested roots. Do not add this table to the preceding table. Its signed residual also contains removed S2 inline queue operations and other credits. A negative residual cannot prove that no outside added instruction exists.

| Disjoint known-root contribution | table | do-chain | call-repeat | cleanup | fold |
| --- | --- | --- | --- | --- | --- |
| mode | 1880.77 | 1852 | 1909.005 | 1877.15166667 | 1932.96166667 |
| adapter dispatch | 44 | 44 | 57 | 44 | 75 |
| second Arc | 253.9 | 253.9 | 337.68 | 245 | 422.46 |
| mixed queue | 625 | 682.375 | 602.42 | 672.195 | 716.375 |
| Reported signed remainder | -220.546666667 | -278.413333333 | -237.268333333 | -369.735 | -126.795 |

| Known root | Class | table | do-chain | call-repeat | cleanup | fold |
| --- | --- | --- | --- | --- | --- | --- |
| Legacy Some encoder including interning, hashing and all three allocations | mode | 1506.77 | 1478 | 1498.005 | 1503.15166667 | 1479.96166667 |
| All Option decoder calls, candidate minus base | mode | 258 | 258 | 258 | 258 | 258 |
| Four outer adapter allocations | second Arc | 122.9 | 122.9 | 162.68 | 114 | 202.46 |
| Adapter destruction subtrees minus legacy frame destruction | second Arc | 122 | 122 | 163 | 122 | 205 |
| Widen from insertion, append and mixed legacy pop (disjoint roots) | mixed queue | 608 | 665.375 | 585.42 | 655.195 | 699.375 |
| Final empty queue destruction difference | mixed queue | 17 | 17 | 17 | 17 | 17 |
| Erased adapters: mode self instructions | mode | 116 | 116 | 153 | 116 | 195 |
| Erased adapters: second Arc self instructions | second Arc | 9 | 9 | 12 | 9 | 15 |
| Erased adapters: adapter dispatch self instructions | adapter dispatch | 44 | 44 | 57 | 44 | 75 |

Two concrete outside-class changes remain reported. First, the cold freeze helper at 0xcdc30 has seven prologue/epilogue instructions at 0xcdc30/0xcdc32/0xcdc33/0xcdc78/0xcdc7c/0xcdc7d/0xcdc7f plus the caller instruction at 0x10afb4: eight gross ABI instructions, 1/1/1/1/2 times per resume, absent as that boundary in S2. Its other instructions pack the existing state; all 22 helper self instructions are not called new work. Second, the resumed driver spills/reloads Session at 0x72a097/0x72c73b once per resume request. The reload replaces a later register move, but the memory traffic differs and the spill is not proved inherent. Section 6 describes the exact cold-boundary comparison. These costs and the finalizer rebox are not silently absorbed into the four allowed classes.

The complete resume address ledger reconciles to the five raw mean deltas and retains every added/removed self instruction, including shared allocator/libc code. Two objectless addresses, 0x406d128 and 0x406d160, have identical count maps in both arms and net zero contribution; their instructions cannot be resolved to an ELF object. No fault-helper instruction executes in these normal profiles. This harness measures the four positive-slope frame shapes and the constant prepared/finalizer path, not every S3 frame or error-only performance. Full correctness coverage remains the complete final suites and oracles.

Resume allocation slopes and the specifically ordered copy-removal gates pass. Full performance acceptance is withheld: the finalizer rebox and payload moves are real remaining defects, and helper ABI/Session spill work is reported rather than proved inherent. Both additive accounting views preserve explicit remainders. This uses the brief's requirement to remove or report work outside the accepted classes; it does not claim a zero non-inherent residual.

Resume table: largest physical-function self-cost changes; complete costs and all addresses are in `candidate/callgrind-means.json` and the object-qualified ledgers.

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 933 | 1417 | 484 |
| core::hash::BuildHasher::hash_one | 696 | 995 | 299 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 284 | 78 | -206 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 169 | 169 |
| rum_core::expr::intern_text | 436.673333333 | 583.381666667 | 146.708333333 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 143 | 143 |
| rum_runtime::runtime::suspension::typed::witness::SlotWitness<I>::suspend | 0 | 140 | 140 |
| 0x00000000001b1980 | 1318.95 | 1188.95 | -130 |
| rum_core::expr::intern_symbol_text | 187.316666667 | 314.741666667 | 127.425 |
| mi_theap_malloc_aligned | 543.09 | 666.95 | 123.86 |

Resume do-chain: largest physical-function self-cost changes; complete costs and all addresses are in `candidate/callgrind-means.json` and the object-qualified ledgers.

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 933 | 1417 | 484 |
| core::hash::BuildHasher::hash_one | 696 | 995 | 299 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 334 | 132 | -202 |
| rum_core::expr::intern_text | 375.101666667 | 573.331666667 | 198.23 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 169 | 169 |
| 0x00000000001b1980 | 1185.775 | 1033.775 | -152 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 143 | 143 |
| rum_runtime::runtime::suspension::typed::witness::SlotWitness<I>::suspend | 0 | 140 | 140 |
| mi_theap_malloc_aligned | 459.34 | 584.78 | 125.44 |
| rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::resume_call_after_head | 123 | 0 | -123 |

Resume call-repeat: largest physical-function self-cost changes; complete costs and all addresses are in `candidate/callgrind-means.json` and the object-qualified ledgers.

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 1365 | 1849 | 484 |
| <rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame as rum_runtime::runtime::suspension::typed::TypedFrame>::resume | 0 | 328 | 328 |
| core::hash::BuildHasher::hash_one | 995 | 1294 | 299 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 278 | 0 | -278 |
| rum_runtime::runtime::eval_call_prepare::<impl rum_runtime::runtime::model::session::Session>::resume_suspended_call_after_argument | 251 | 0 | -251 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 222 | 222 |
| rum_core::expr::intern_text | 571.545 | 764.601666667 | 193.056666667 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 189 | 189 |
| mi_theap_malloc_aligned | 795.4 | 939.51 | 144.11 |
| mi_free | 802.77 | 942.77 | 140 |

Resume cleanup: largest physical-function self-cost changes; complete costs and all addresses are in `candidate/callgrind-means.json` and the object-qualified ledgers.

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 933 | 1417 | 484 |
| core::hash::BuildHasher::hash_one | 696 | 995 | 299 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 276 | 70 | -206 |
| rum_core::expr::intern_text | 381.238333333 | 560.145 | 178.906666667 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 169 | 169 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 143 | 143 |
| rum_runtime::runtime::suspension::typed::witness::SlotWitness<I>::suspend | 0 | 140 | 140 |
| mi_theap_malloc_aligned | 438.4 | 563.76 | 125.36 |
| rum_runtime::runtime::eval_core::call::<impl rum_runtime::runtime::model::session::Session>::resume_call_after_head | 123 | 0 | -123 |
| mi_free | 442.04 | 563.72 | 121.68 |

Resume fold: largest physical-function self-cost changes; complete costs and all addresses are in `candidate/callgrind-means.json` and the object-qualified ledgers.

| Function | S2 I | S3 I | Delta I |
| --- | --- | --- | --- |
| <core::hash::sip::Hasher<S> as core::hash::Hasher>::write | 933 | 1417 | 484 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_suspension_frame | 477 | 127 | -350 |
| core::hash::BuildHasher::hash_one | 696 | 995 | 299 |
| <rum_runtime::runtime::suspension::typed::adapter::TypedAdapter<F> as rum_runtime::runtime::suspension::typed::adapter::ErasedFrame>::resume | 0 | 285 | 285 |
| rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::insert_typed_erased_frame | 0 | 240 | 240 |
| rum_runtime::runtime::suspension::typed::witness::SlotWitness<I>::suspend | 0 | 218 | 218 |
| rum_core::expr::intern_text | 384.705 | 601.43 | 216.725 |
| 0x00000000001b1980 | 1653.775 | 1441.775 | -212 |
| <rum_runtime::runtime::resolution::resume_frames::ResumeFixedPointFromSuspensionFrame as rum_runtime::runtime::suspension::typed::TypedFrame>::resume | 0 | 210 | 210 |
| <rum_runtime::runtime::eval_core::call::ResumeCallWithDispatchPlanSuspensionFrame as rum_runtime::runtime::suspension::typed::TypedFrame>::resume | 0 | 198 | 198 |

Timing ran last, after correctness, allocation, all flagged-label repeats, Callgrind and address attribution. The full core-language-microbench.sh panel produced120 matching labels per arm, scale1/reps7, retained-module-gc-batches3 (the script default), profiling disabled, on the two CGU1 CLI binaries. These are in-kernel monotonic AbsoluteTiming measurements, not wall-clock differences. The median candidate/base ratio is0.991613; 35 labels are slower in this single panel pair. Timing is an indication only and does not override instruction evidence.

The host monitor observed five consecutive quiet seconds before each arm and no competing CPU process above0.15 core during either panel. It did not stop or alter other processes. GPU occupancy could not be read from the available sysfs interface, so GPU idleness is not independently verified. Raw activity samples are in timing-{base,candidate}-host.json; exact commands, benchmark .rum input, stdout, stderr and per-label timings.tsv are under r6b/perf/timing/. This is one final timing pair, not a repeated statistical timing acceptance.

The largest slower ratios are matcher-single-wide-small 1.15426, matcher-single-wide-large 1.15347, association-part-key-read-width32000 1.13284 and table-iterator-symbol 1.10759. These individual results remain reported; the median does not clear them. The complete per-label table follows. No timing-only source change or additional performance acceptance is inferred from this single pair.

| Timing label | S2 seconds | S3 seconds | S3/S2 |
| --- | --- | --- | --- |
| association-part-key-read-width4000 | 0.00233879 | 0.00256072 | 1.09489094788 |
| association-part-key-read-width32000 | 0.00866228 | 0.00981294 | 1.13283569684 |
| association-part-positional-read-width4000 | 3.1343e-05 | 3.0236e-05 | 0.96468110902 |
| association-part-positional-read-width32000 | 3.1236e-05 | 3.0304e-05 | 0.97016263286 |
| loop-empty | 0.000271031 | 0.000256535 | 0.946515343263 |
| do-count-null | 0.000246824 | 0.000245387 | 0.994178037792 |
| do-iterator-null | 6.0497e-05 | 5.6552e-05 | 0.934790154884 |
| table-count-null | 8.2247e-05 | 8.3264e-05 | 1.01236519265 |
| table-iterator-null | 1.9054e-05 | 1.8749e-05 | 0.983992862391 |
| table-iterator-symbol | 7.454e-06 | 8.256e-06 | 1.10759323853 |
| literal-integer | 9.8998e-05 | 9.6761e-05 | 0.977403583911 |
| literal-string | 0.00010217 | 9.4317e-05 | 0.923137907409 |
| symbol-ownvalue-read | 0.000226378 | 0.000220559 | 0.974295205364 |
| symbol-ownvalue-guarded-read | 0.000191316 | 0.000182967 | 0.956360158063 |
| set-ownvalue | 0.000261978 | 0.000264428 | 1.00935193031 |
| list-literal-small | 5.4994e-05 | 5.5198e-05 | 1.00370949558 |
| head-list | 0.000904125 | 0.000935681 | 1.03490225356 |
| length-list | 0.000905002 | 0.00093849 | 1.03700323314 |
| part-list | 0.00107884 | 0.00110734 | 1.02641726299 |
| sameq-integer | 0.000624546 | 0.000636368 | 1.01892894999 |
| sameq-list | 0.00060895 | 0.00062528 | 1.02681665161 |
| if-true | 0.000702467 | 0.000695944 | 0.990714154544 |
| module-local | 0.000727757 | 0.000724075 | 0.994940618915 |
| module-chain-escape-small | 4.9328e-05 | 5.2171e-05 | 1.05763460915 |
| module-chain-escape-large | 9.836e-05 | 9.7277e-05 | 0.988989426596 |
| block-local | 0.000470794 | 0.000467372 | 0.992731428183 |
| block-symbol-only-repeat | 0.000305235 | 0.000307151 | 1.00627713074 |
| block-init-null-repeat | 0.000355356 | 0.000350559 | 0.986500861108 |
| block-init-read-repeat | 0.000402258 | 0.000399966 | 0.994302164283 |
| with-local | 0.000476271 | 0.00047185 | 0.990717469676 |
| with-deep-body-small | 0.00184433 | 0.00182968 | 0.99205673605 |
| table-small | 0.000190327 | 0.000185762 | 0.97601496372 |
| table-iterator-plus | 3.6978e-05 | 3.9648e-05 | 1.07220509492 |
| table-iterator-pair-list | 6.532e-05 | 6.4923e-05 | 0.993922229026 |
| do-small | 0.000902483 | 0.000900154 | 0.997419341971 |
| d-nested-sin-small | 0.00516074 | 0.00521355 | 1.0102330286 |
| downvalue-exact-hit | 0.00075538 | 0.000762677 | 1.00966003866 |
| downvalue-exact-miss | 0.000545589 | 0.000546807 | 1.0022324497 |
| downvalue-pattern-hit | 0.000948413 | 0.000947354 | 0.998883397845 |
| downvalue-pattern-miss | 0.000282966 | 0.000280751 | 0.992172204434 |
| downvalue-exact-miss-repeat | 0.00125208 | 0.00123142 | 0.983499456904 |
| downvalue-pattern-miss-repeat | 0.00130105 | 0.0012497 | 0.960531878098 |
| downvalue-guarded-hit | 0.00189833 | 0.0018599 | 0.979755890704 |
| downvalue-guarded-decline-fallback | 0.00141975 | 0.00139353 | 0.981531959852 |
| upvalue-guarded-hit | 0.00236081 | 0.0023324 | 0.987965994722 |
| subvalue-unguarded-hit | 0.00139724 | 0.0013699 | 0.980432853339 |
| subvalue-unguarded-miss | 0.000766066 | 0.000760622 | 0.992893562696 |
| subvalue-guarded-hit | 0.00243863 | 0.00239179 | 0.980792494146 |
| default-unguarded-hit | 0.00127902 | 0.00127406 | 0.99612203093 |
| default-unguarded-miss | 0.00106872 | 0.00105857 | 0.990502657385 |
| default-guarded-hit | 0.00162438 | 0.00163599 | 1.00714734237 |
| matchq-integer-success | 0.000282548 | 0.000286495 | 1.01396930787 |
| matchq-integer-fail | 0.000289246 | 0.000290864 | 1.00559385437 |
| matcher-single-tiny | 0.000677894 | 0.000665035 | 0.981030957642 |
| matcher-single-wide-small | 4.2792e-05 | 4.9393e-05 | 1.1542578052 |
| matcher-single-wide-large | 8.4552e-05 | 9.7528e-05 | 1.15346768852 |
| replaceall-simple | 0.000351853 | 0.000358207 | 1.01805867791 |
| rule-rhs-deep-small | 0.00303495 | 0.00298425 | 0.983294617704 |
| rewrite-guarded-hit | 0.00179813 | 0.00177737 | 0.988454672354 |
| rewrite-guarded-decline-fallback | 0.00255325 | 0.00253971 | 0.994696954861 |
| cases-wide-no-match | 0.000147916 | 0.000143872 | 0.972660158468 |
| count-wide-no-match | 0.000140716 | 0.000138024 | 0.980869268598 |
| position-wide-no-match | 0.000176694 | 0.000170456 | 0.964696028162 |
| deletecases-wide-no-match | 0.000171559 | 0.000166045 | 0.967859453599 |
| deletecases-wide-association-no-match | 0.000436134 | 0.000417453 | 0.957166834046 |
| deletecases-leaf-delete-small | 0.00221082 | 0.00212906 | 0.963018246623 |
| replace-wide-no-match | 8.2353e-05 | 7.7394e-05 | 0.93978361444 |
| replace-leaf-rebuild-small | 0.00269748 | 0.00256806 | 0.952021887095 |
| take-multispec-small | 0.00163141 | 0.00160726 | 0.985196854255 |
| replacepart-deep-path-small | 0.00132294 | 0.001299 | 0.981903941222 |
| replacepart-wide-rule-list | 0.000292486 | 0.000289783 | 0.990758532032 |
| replacepart-guarded-hit | 0.000535434 | 0.00053853 | 1.00578222526 |
| replacepart-guarded-decline-fallback | 0.000588664 | 0.000586398 | 0.996150605439 |
| string-repeat-small | 0.000186225 | 0.00018679 | 1.00303396429 |
| character-range-small | 0.000134911 | 0.000128745 | 0.954295795006 |
| stringtrim-literal-list-4000 | 0.000683551 | 0.00065921 | 0.964390367361 |
| stringtrim-literal-list-32000 | 0.00506054 | 0.00504234 | 0.996403545867 |
| stringtrim-single-literal-4000 | 0.000634111 | 0.000635871 | 1.0027755393 |
| stringtrim-single-literal-32000 | 0.0050295 | 0.00502371 | 0.998848792126 |
| stringtrim-whitespace-4000 | 0.000626169 | 0.00062764 | 1.00234920605 |
| stringtrim-whitespace-32000 | 0.00502736 | 0.00501123 | 0.996791556602 |
| stringreplace-single-literal-unguarded | 0.000380491 | 0.000388071 | 1.01992162758 |
| stringreplace-pattern-unguarded | 0.00104138 | 0.00103644 | 0.995256294532 |
| stringreplace-guarded-hit | 0.000690059 | 0.000690739 | 1.00098542299 |
| stringreplace-guarded-decline-fallback | 0.000826004 | 0.000808121 | 0.978349983777 |
| appendto-list-incremental-small | 0.00060201 | 0.000484865 | 0.805410209133 |
| appendto-list-incremental-large | 0.00155541 | 0.00165576 | 1.06451675121 |
| appendto-part-outer1000-small | 0.00234094 | 0.00221047 | 0.944265978624 |
| appendto-part-outer1000-large | 0.00445862 | 0.00446103 | 1.00054052599 |
| association-set-key-incremental-small | 0.00147483 | 0.00140728 | 0.954198110969 |
| association-set-key-incremental-large | 0.00556573 | 0.00534531 | 0.960396929064 |
| association-small-literal-repeat | 0.000736163 | 0.000742152 | 1.00813542653 |
| merge-nested-small | 0.00117145 | 0.00115437 | 0.985419778906 |
| set-part-fixed100-len100 | 0.000139793 | 0.00013755 | 0.983954847525 |
| set-part-fixed100-len800 | 0.000544723 | 0.000540486 | 0.992221734717 |
| set-part-fill-small | 0.000408156 | 0.000404503 | 0.99104999069 |
| set-part-fill-large | 0.0012721 | 0.00127012 | 0.998443518591 |
| upvalue-candidates-tiny | 0.0019388 | 0.00195753 | 1.00966061481 |
| upvalue-candidates-wide-small | 0.00015333 | 0.000148963 | 0.971518946064 |
| upvalue-candidates-wide-large | 0.000298904 | 0.000304348 | 1.01821320558 |
| sort-callback-small | 0.000163231 | 0.000162935 | 0.998186618963 |
| sort-callback-large | 0.000377318 | 0.000373986 | 0.991169252461 |
| sortby-callback-small | 0.000195636 | 0.000193234 | 0.987722096138 |
| sortby-callback-large | 0.000437676 | 0.000433116 | 0.989581334138 |
| reversesort-callback-small | 0.000159578 | 0.000159666 | 1.00055145446 |
| reversesort-callback-large | 0.000359777 | 0.000359113 | 0.998154412317 |
| keysort-callback-small | 8.5073e-05 | 8.2462e-05 | 0.969308711342 |
| keysort-callback-large | 0.000190246 | 0.000180971 | 0.951247332401 |
| keysortby-callback-small | 4.5621e-05 | 4.4207e-05 | 0.969005501852 |
| keysortby-callback-large | 9.8931e-05 | 9.6135e-05 | 0.971737877915 |
| sort-default-wide | 2.8362e-05 | 1.6757e-05 | 0.590825752768 |
| sort-symbol-default-width4000 | 0.000299842 | 0.000291784 | 0.973125846279 |
| sort-symbol-default-width16000 | 0.00124652 | 0.00123831 | 0.99341366364 |
| sortby-default-wide | 0.000524936 | 0.000509671 | 0.970920264566 |
| reversesort-default-wide | 2.815e-05 | 1.7174e-05 | 0.610088809947 |
| keysort-default-wide | 0.000374159 | 0.000372487 | 0.995531311555 |
| keysortby-default-wide | 0.000873295 | 0.000867594 | 0.993471850864 |
| valueq-undefined-symbol-definitions-2000 | 7.985e-06 | 7.959e-06 | 0.996743894803 |
| valueq-undefined-symbol-definitions-8000 | 8.197e-06 | 7.779e-06 | 0.949005733805 |
| module-retained-gensym-gc-batch | 6.8221e-05 | 6.2337e-05 | 0.913750897817 |

The final ordinary-path gates pass: 59/59 allocation and byte equalities, 59/59 exact memcpy equalities, zero ordinary flags, and all three deterministic leaf paths below S2. The targeted four adapter action copies and two extra typed-driver slot copies are removed. Native-stack and 48-level gates also pass.

Resume totals remain higher because the global Legacy wire, adapter, second Arc and mixed queue execute additional work. The allocation slopes match typed frames + 3, but that slope check is insufficient to accept the whole path: the 56-byte finalizer rebox and two payload transfers execute once at terminal finalization. Their addresses and six raw executions are reported above. Freeze-helper ABI work and the driver's Session spill/reload are also reported rather than asserted inherent. The complete address partition has an explicit remainder; the disjoint-root accounting's negative residual does not erase these concrete operations. Full performance acceptance remains withheld, and timing does not change that verdict.

### 6. S2 cold-helper retype and hot boundary

The inherited cleanup makes continuation_fault_outcome consume the outcome and return EvalError directly, retaining cold/inline-never. Four callers wrap Err explicitly; the two specified test callers lose unwrap_err. The final binary comparison below checks all four hot boundaries.

All four boundaries retain one fault-word comparison and one cold conditional branch. Three fast successor sequences are equal after normalizing only stack-slot offsets and relocations, retaining registers, immediates, operations and called symbols. The resumed request has the exact reviewed register/spill difference described below; it is not claimed assembly-identical. This is a boundary check, not a claim that the entire resumed evaluator has unchanged cost.

| Boundary / binary | S2 guard → cold call | Candidate guard → cold call | Hot successor instructions |
| --- | --- | --- | --- |
| eval_expr_request_from_checkpoint | 0x728f4e, 0x728f56 → 0x72942f | 0x7299ce, 0x7299d6 → 0x729eac | 11 → 11 |
| resume_eval_request_from_checkpoint | 0x72bb28, 0x72bb30 → 0x72c2d6 | 0x72c740, 0x72c748 → 0x72cefd | 24 → 23 |
| eval_input_with_terminated_handling | 0x750120, 0x750128 → 0x75066c | 0x751ba0, 0x751ba8 → 0x7520ec | 23 → 23 |
| resume_inspect_rewrite_rules_with_input (inlined test thunk) | 0x1cc629, 0x1cc632 → 0x1cd557 | 0x1ccbd0, 0x1ccbd9 → 0x1cdaf7 | 11 → 11 |

The first strict comparison rejected `%r15` versus `%rsi` at the resumed guard. The reviewed dataflow is explicit: S2 saves the incoming Session in `%r15` at0x729524; the candidate saves it via `%rbx` at0x729f81/0x72a097, then reloads stack slot0x10 into `%rsi` at0x72c73b immediately before the guard. The guard still reads Session+0x450. The S2 post-guard `mov %r15,%rsi` at0x72bbd1 disappears. Thus the guard-plus-successor region is26→25 instructions, or26→26 when the candidate predecessor reload is included. The candidate does introduce a Session spill/reload in the containing driver; this is disclosed at its addresses, not normalized away or claimed to have identical memory traffic. The driver’s complete instruction/copy accounting is in section5. The corrected checker accepts only this exact observed transformation and retains the first rejection log.

The first two rows use the immutable CGU1 resume binaries; top-level text evaluation uses the immutable CGU1 CLI binaries. The generic rewrite-inspection method is absent from both production measurement binaries and is inlined in the existing release runtime-test thunk (`FnOnce::call_once`, S2 0x1cc2c0, candidate 0x1cc870). Its identity is established by the ordered resume call and `Option<Vec<RewriteRuleInfo>>::from_continuation_expr` call; no new fixture or binary was built. Those two release test binary hashes and complete addressed bodies are recorded in `r6b/cold-boundaries/native-inspection.json`.

No cold-helper instruction executes in the three N/2N Callgrind pairs for any of the five normal resume workloads. These profiles do not exercise a continuation fault. The final typed fault tests and complete oracles provide the separate fault/absence checks. Exact guard, fall-through and cold-call instructions: `r6b/cold-boundaries/comparison.json`; whole containing functions and ELF-resolved helper targets: `production.json` and `native-inspection.json` in that directory.

The same absence check passes for every ordinary profile across all 59 labels. Both top-level success branches, ConvertToValue and ConvertCyclesToValue, were compared. The three identical normalized successor sequences and the exact reviewed resumed-request exception are the limits of this hot-boundary claim.

### 7. Stop-loss counters, deviations and open questions

The runtime attempt counter ends at eight. Attempt 7 moved none of the named gates (the two inlined typed-driver copies remained), so its consecutive no-progress count was one. Attempt 8 removed one of those two copies and left the common outcome assignment, resetting that count to zero. The earlier leaf/caller-place trials and their measured progress are listed in section 5. No attempt was continued past two consecutive no-progress outcomes.

The initial quick console filter looked for `drive_resuming_frames`, missing its inlined body inside `resume_eval_request_from_checkpoint`. The complete saved copy table did contain both copies. The full object/address audit caught the error before acceptance. The filter was corrected, attempts 7/8 followed, and all final source correctness, native-stack and performance gates were rerun under `r6b/`. Intermediate `r6/` binaries and measurements remain available and are not presented as final evidence.

Two watched macro-definition falsifiers intentionally disabled the exact-definition guard. A scratch restore using an older timestamp then reused that weakened binary; its failed drift test remains in the ledger. Updating the scratch source timestamp forced a rebuild, after which all seven controls passed. The earlier quote-token whitespace mismatch was corrected identically in both definitions; strict equality was retained. The final macro, adapter and fixture source hashes equal the watched sources.

An early unqualified `--exact` invocation selected zero tests. It was not accepted; the final fully qualified 48-level test selected and passed one test. The first copied scope-audit script carried a stale R5 hash for the director-owned security document. The R6 entry snapshot established the actual hash, the scratch assertion was corrected, and that document remains byte-identical to its R6 entry. Copied measurement-script paths were inspected and corrected before execution so final evidence is isolated in `r6b/`.

Two early short metadata-only Python invocations in a prior continuation lacked the bounded wrapper; they spawned no child process. All builds, tests, compiler fixtures, profiles, timing runs and subsequent Python analysis commands used the required bounds. The oracle was run as its underlying Cargo commands in both builds, as accepted by the director; test selection, features and assertions were preserved. Native ZeroMQ early returns are counted as skips, not exercised passes.

No git command, network request, sub-agent, commit, dependency change, toolchain change, unsafe addition, mode flip, new public raw API or out-of-scope production edit was made. The old brief bytes and earlier raw evidence are preserved. No documented semantic contract, rejection, Checks flag or fault-absence assertion was weakened. Any remaining performance limitation is stated explicitly in section 5 and the verdict; timing does not override the instruction evidence.

A subsequent session fork ended the first final allocation panel after 105 correct base cells, before an exit record was written. No measurement process remained. Its raw output is preserved under `r6b/perf/alloc-interrupted-1/` and is not used in the final comparison. The complete allocation panel was restarted under a new gate name; the interrupted gate has no invented exit status.

The first root-cost accounting assertion expected two physical Option decoder call sites; the final addressed graph contains one per arm, with a second execution of that same site in Fold. The scratch assertion was corrected to the observed single site, and the failed analysis log is retained. The first cold-boundary comparison rejected a register difference. Its reviewed replacement accepts only the exact Session spill/reload and removed register move documented in section6; it is not a general register-normalization relaxation and changes no production test.

The final whole-program audit found a zero-slope defect: FinalizerAdapter allocates a new 56-byte Box and moves its payload twice once per call-repeat terminal finalization. It is not elided. All 30 raw allocation-count cells now reconcile, including the constant fifth frame adapter, two one-time interning allocations and this rebox. No ninth production repair attempt was made; the brief explicitly allows reporting remaining defects. The final stop-loss counter is zero, not a claim that the remaining costs are accepted. Freeze-helper ABI instructions and Session spill/reload are also reported outside the four accepted categories, and the address accounting keeps its remaining signed residual explicit.

Normal-path profiles do not measure error-only performance or independently isolate every owned S3 frame. The four positive-slope physical frame shapes, the constant PreparedDispatchFrame and its finalizer are disclosed. Complete correctness tests still cover the owned API and fault paths. Objectless Callgrind addresses have equal count maps and net zero delta; their identities and assembly are not invented. Queue capacities and fixed residual byte explanations retain their stated inference limits.

Timing ran last and completed 120 labels in each arm. The host monitor observed no competing CPU process above 0.15 core; GPU occupancy was unavailable from the allowed sysfs interface. All 35 slower labels and the largest ratios are retained as indications, without using the median or timing to override instruction evidence. No further production edit or new measurement run followed the final timing pair.


## Director amendment — redispatch 7: the finalizer stops reboxing

The report is exact, and its refusal to claim acceptance is right. One
defect remains, and this redispatch removes it. Everything else is ruled
below.

### Accepted as reported

The director reproduces all of this at the gate.

- **The ordinary path.** No label is flagged. All 59 `memcpy` counts equal
  S2. The three leaves run 110 instructions against S2's 112. Allocations
  and bytes are equal on all 59. Both causes are named and separately
  isolated: the 136 B caller-result rebind and the 168 B three-record
  profile closure.
- **The stack.** All four limits are below S2, and the 48-level test
  passes.
- **The driver and adapter shape.** This covers four things:
  - the private `Action::DecodeFailure`;
  - the borrowed adapter;
  - the input passed by mutable reference;
  - ending the `Abort` placeholder's lifetime with `mem::forget`.

  The placeholder is acceptable for four reasons:
  - it has exactly one implementor, which is private;
  - the debug assertion checks it, and the tests check the postcondition
    on all four branches;
  - every path re-initializes `outcome` before the loop continues;
  - a violation could only leak an `EvalError`. It could never be unsound.
- **The cold freeze helper's 8 ABI instructions per suspension.** They are
  the price of outlining frame construction, and R5's stack result rests on
  that outlining.
- **The Session spill and reload, once per resume request.** This is a
  register-allocation response in the containing function, with no source
  operation behind it. The guard region goes 26 → 25 instructions, or 26
  with the reload. There is nothing to remove.
- **The resume accounting.**
  - The disjoint known-root view explains more than the measured delta in
    all five workloads; the signed remainder is −127 to −370.
  - Most of the added cost is the Legacy `Some` wire: its encoder at about
    1 500 I per resume, plus the +258 I strict decode. That is mode work
    owed to `Option<Expr>`'s flip.
  - The second `Arc` and the mixed queue are the design's own items.

### Ruling: remove the `FinalizerAdapter` rebox

`FinalizerAdapter<F>::finish` (`suspension/typed/adapter.rs`) runs
`Box::new(self.0)`:

- `SlotFinalizer::finish` takes `self: Box<Self>`, while the erased box
  holds `FinalizerAdapter<F>`.
- Every typed finalization therefore pays a second 56 B allocation, two
  payload transfers through a stack temporary, and a free.
- S2's legacy F0570 used one box. S3's typed F0570 uses two.

This is an S3 regression on call-repeat. It is also a defect in S1 code
that every typed finalizer would inherit: 53 production finalizers convert
in S4–S54.

Fix by shape, so that the erased box is the finalizer's own box. One
candidate:

- a blanket `impl<F: SlotFinalizer> ErasedFinalizer for F`;
- `Action::Finalize(Box::new(finalizer))`;
- `SlotFinalizer::finish(self)` called by qualified path, followed by the
  mode encode.

Any shape with the same result is fine. The constraints:

- The public `SlotFinalizer` signature does not change.
- Nothing new becomes public, and no `unsafe` is added.
- `FinalizerAdapter` goes away if the shape no longer needs it.

**The ripple is part of the change.** The host-payload ledger row for
`Action.Finalize.0` (`dyn ErasedFinalizer`) records a disposition chain
through `FinalizerAdapter.0`:

- That chain must be re-derived by ownership discovery, not hand-edited.
- If discovery cannot follow the new shape (for example, a blanket impl
  over a trait bound), that is a STOP. Report it with the discovery output.
  Do not write the chain by hand, and do not pick a weaker shape to avoid
  the analyzer gap without saying so.

The same applies to the following, wherever the change moves them:

- the unknown-attribute, capture and macro inventories;
- the frame-root pins;
- the census.

**Acceptance:**

- **Allocation counts.** Call-repeat's whole-program extra allocations fall
  from 7N + 5 to 7N + 4, in all three pairs. The 56 B allocation at finish is
  absent, with no replacement allocation anywhere. The other four workloads
  are unchanged.
- **The finish path at its addresses.** It executes no allocation, no free
  beyond the one box the legacy path also frees, and no payload transfer
  through a stack temporary. Beyond S2's legacy finish, it executes nothing
  except the encode through the mode.
- **Section C in full on the final source.** Every gate, verbatim, in both
  builds, including:
  - the oracle;
  - census and compile controls;
  - every replay the change expires;
  - the four native-stack limits and the 48-level test;
  - the full instruction panel (all 59 ordinary labels must stay unflagged,
    with `memcpy` counts equal to S2's and the leaves at or below S2);
  - the allocation panel;
  - resume attribution;
  - timing, last.

  The final binaries change, so no earlier measurement carries over.

**Stop-loss.** The named gates are the call-repeat allocation intercept and
the finish path's transfer count. Two attempts that move neither is a stop.

Append a new `## Execution report` after this section. Section 5 may cite
R6's evidence for anything this change cannot reach, but only after showing
that the final binaries reproduce it.

## Execution report — redispatch 7: one-box finalizer candidate; STOP on ownership discovery

**S3 is not accepted. The mandatory ownership STOP has fired.** The two-file candidate removes FinalizerAdapter, stores the finalizer itself in Action::Finalize and calls SlotFinalizer::finish by qualified path before the same mode encode. Rust compilation succeeds, but the unchanged ownership analyzer cannot derive the expression-debt witness for Action.Finalize.0 through the blanket implementation. The complete ownership run exits 101: 21 passed, four failed. Formatting passes. No ownership chain, inventory, pin or analyzer was edited to make it pass.

The governing amendment says: “If discovery cannot follow the new shape (for example, a blanket impl over a trait bound), that is a STOP.” This exact case occurred. No alternate shape was attempted. No release candidate was built, and none of the required new allocation, finish-path, instruction, native-stack or timing results is claimed. The R6 measurements remain historical and do not validate this changed source. All R7 scratch evidence is under `.temp/brief-609/r7/`; bounded command logs and receipts are `.temp/brief-609/r7-*.{log,command.json,exit.json}`.

### 1. Files changed, ripple and pin disclosures

| File | Change | Characters before → after |
| --- | --- | --- |
| crates/rum-runtime/src/runtime/suspension/typed/action.rs | Remove FinalizerAdapter import; Action::Finalize(Box::new(FinalizerAdapter(finalizer))) → Action::Finalize(Box::new(finalizer)). | 2376 → 2340 |
| crates/rum-runtime/src/runtime/suspension/typed/adapter.rs | Remove FinalizerAdapter<F>; impl ErasedFinalizer for FinalizerAdapter<F> → blanket impl<F: SlotFinalizer> ErasedFinalizer for F. Box::new(self.0).finish() → SlotFinalizer::finish(self); retain the mode-encoding map. | 2724 → 2620 |
| docs/dev/codex/brief-609-g13-s3-evaluation-dispatch.md | Append this STOP report only; preserve all entry bytes, including the director amendment. | append only |

Both edited Rust files remain below 10k characters and retain their existing action/erasure responsibilities. No module root, visibility, public SlotFinalizer signature, mode, caller, concrete finalizer, driver, test, manifest, lockfile, toolchain or core representation changes. There is no new unsafe code. The complete two-file patch is:

```diff
--- crates/rum-runtime/src/runtime/suspension/typed/action.rs R7-entry
+++ crates/rum-runtime/src/runtime/suspension/typed/action.rs R7-candidate
@@ -2,7 +2,7 @@
 
 use rum_core::Expr;
 
-use super::adapter::{DecodeFailure, ErasedFrame, FinalizerAdapter, erase};
+use super::adapter::{DecodeFailure, ErasedFrame, erase};
 use super::channel::{CodecKey, Slot, encode};
 use super::{Arc, EvalError, SlotFinalizer, SlotValue, TypedFrame};
 
@@ -57,7 +57,7 @@
 
     pub fn finalize<F: SlotFinalizer<Output = O>>(finalizer: F) -> Self {
         Self {
-            inner: Action::Finalize(Box::new(FinalizerAdapter(finalizer))),
+            inner: Action::Finalize(Box::new(finalizer)),
             output: PhantomData,
         }
     }
--- crates/rum-runtime/src/runtime/suspension/typed/adapter.rs R7-entry
+++ crates/rum-runtime/src/runtime/suspension/typed/adapter.rs R7-candidate
@@ -70,12 +70,8 @@
         + std::mem::size_of::<F>()
 }
 
-pub(super) struct FinalizerAdapter<F: SlotFinalizer>(pub F);
-
-impl<F: SlotFinalizer> ErasedFinalizer for FinalizerAdapter<F> {
+impl<F: SlotFinalizer> ErasedFinalizer for F {
     fn finish(self: Box<Self>) -> Result<Slot, EvalError> {
-        Box::new(self.0)
-            .finish()
-            .map(|value| encode(value, CodecKey::new()))
+        SlotFinalizer::finish(self).map(|value| encode(value, CodecKey::new()))
     }
 }
```

The only discovered UnknownConstruct ripple is the unchanged encoding closure moving from adapter.rs:79:18 to adapter.rs:75:41. It remains unresolved. The inventory was deliberately left stale after the mandatory STOP. Action.Finalize.0 remains at action.rs:22:14; its stored witness still names the removed FinalizerAdapter.0 and is not replaced with a hand-written chain. All frame-root pins and census receipts remain unchanged and unrerun.

### 2. Owned-row reconciliation, transformations and census status

This repair adds no owned frame or macro row. F0570 remains FinishPreparedCallArgumentsFinalizer implementing the unchanged public SlotFinalizer API; its concrete finish body is byte-identical. Only its private erasure/construction shape changes. The following source reconciliation is inherited identity, not a newly earned census receipt.

| F row | S2 identity | Current source transition; R7 census not run |
| --- | --- | --- |
| F0204 | BuiltinEvaluateSuspensionFrame | Typed Expr → Expr; same name. |
| F0272 | PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame | Typed Expr → PreparedCallArguments; also receives fresh repeated state formerly handled by F0273. |
| F0273 | ResumeSuspendedCallArgumentPreparationSuspensionFrame | Merged into F0272; no remaining production implementation of the old name. |
| F0274 | EvaluateCallSuspensionFrame | Typed Expr → Expr; head conversion at the shared entry. |
| F0275 | ResumeCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch split to PreparedDispatchFrame. |
| F0276 | ResumeSuspendedCallWithDispatchPlanSuspensionFrame | Typed Option<Expr> → Expr; prepared branch uses the same PreparedDispatchFrame. |
| F0277 | EvaluateStoredOwnValueSuspensionFrame | Typed CompiledMatchOutcome → Expr. |
| F0278 | RunCallDispatchStageSuspensionFrame | Typed Expr → Option<Expr>; complete the semantic Some map. |
| F0279 | EvalReleaseHoldSuspensionFrame | Typed Expr → Expr. |
| F0526 | EvaluateProfiledFixedPointSuspensionFrame | Typed Expr → Expr. |
| F0527 | ResumeFixedPointFromSuspensionFrame | Typed Expr → Expr. |
| F0570 | FinishPreparedCallArgumentsFinalizer | SlotFinalizer Output=PreparedCallArguments. |

| M row | Actual producer (unchanged in R7) | Ordinary map (unchanged) | Resumed map (unchanged) |
| --- | --- | --- | --- |
| M0205 | Expr | builtin_evaluate returns the evaluated value unchanged | Typed identity frame; no post-witness semantic map. |
| M0279 | Expr | append/splice/drop-Nothing; preserve Unevaluated entries; advance remaining arguments; build PreparedCallArguments | Typed F0272 reentry uses the same append helper and fresh cursor/splice fields; pure finalizer builds the exact output after Arc release. |
| M0280 | Expr | the same append and cursor/restore work for later arguments | Old macro removed by D0001; witness of actual evaluated_result is passed to token.suspend with fresh F0272. No original Arc is captured. |
| M0281 | Expr | SharedExpr conversion, plan construction and dispatch; profiling records | Witness now at evaluate(head.into_owned()); shared resume_call_after_head accepts Expr or existing SharedExpr and performs Into<SharedExpr> once. |
| M0282 | PreparedCallArguments or Option<Expr> (split) | normalize prepared args, install restore; stage Some returns value, None advances cursor | Prepare macro → PreparedDispatchFrame; direct stage macro M0282S → F0275. Shared normalization/restore runs once; None advances the loop; resumed frame enters k+1. |
| M0283 | PreparedCallArguments or Option<Expr> (split) | same work with the suspended plan and frozen restore | Prepare macro → PreparedDispatchFrame; direct stage macro M0283S → F0276. Restore freezes after the common installer; None advances the loop; resumed frame enters k+1. |
| M0285 | CompiledMatchOutcome | matched branch evaluates own value; unmatched branch resumes symbol lookup | Same branches in F0277; only the Expr transport map and raw input decode disappear. |
| M0286 | Expr | Some(value) after evaluating the threaded call | F0278 now completes resumed.map(Some); three listable fixtures watched fail → pass. |
| M0287 | Expr | Shared finish_builtin_dispatch: profile, original checked/unchecked growth behavior, semantic Some. | F0278 runs the same growth check and semantic Some; M0288 merged into M0287. |
| M0288 | Expr | Shared finish_builtin_dispatch: profile, original checked/unchecked growth behavior, semantic Some. | F0278 runs the same growth check and semantic Some; M0288 merged into M0287. |
| M0289 | Expr | ReleaseHold evaluation result unchanged | Typed identity frame. |
| M0561 | Expr | fixed-point equality/cycle history/trace update and next pass | Typed F0526 uses outcome? then the unchanged resume_fixed_point_from state transition. |
| M0562 | Expr | same fixed-point transition in resumed evaluation | Typed F0527 uses outcome? then the same helper. |

T0023–T0027 are unchanged from R6. D0001 still uses fresh state through the witnessed token and invokes the finalizer after both originating Arcs drop; the driver source is unchanged. The candidate now erases the finalizer's own box. Existing production receipts still record 607 implementations, 589 macro sites, 12 typed implementations and 13 typed macros, but neither feature build reran the census after this edit. No receipt is advanced on the basis of those old figures. No global mode flips or later-family conversions occur.

### 3. Divergence fixtures and equality status

No fixture was added, removed or weakened. The six isolated S3 builtin/listable cases and prepared.twice/head.map.twice controls are byte-identical to entry. Their pre-conversion failures and earlier equality outputs remain in the earlier execution reports and raw logs; they were not rerun here. The six original Fold/Nest cases remain assigned to S13. Because discovery required STOP before section C, there is no R7 post-change oracle result in either build, no new fault-occurrence delta, and no renewed equality claim.

### 4. Gates, exact discovery output and replay status

Only the ownership gate and formatting check ran in R7. Both used the required bounded wrapper with offline Cargo, one build job, one test thread and TMPDIR under brief609. The exact invocations and child-command environments follow.

**r7-ownership-discovery: exit 101.** Raw output: `.temp/brief-609/r7-ownership-discovery.log`.

```text
python3 scripts/run-bounded.py --timeout 900 -- python3 .temp/brief-609/gate.py r7-ownership-discovery cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
```

```text
CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1
CARGO_NET_OFFLINE=true
TMPDIR=/home/remi/dev/rum/.temp/brief-609/tmp
RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
```

**r7-format-check: exit 0.** Raw output: `.temp/brief-609/r7-format-check.log`.

```text
python3 scripts/run-bounded.py --timeout 900 -- python3 .temp/brief-609/gate.py r7-format-check cargo fmt --check
```

```text
CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1
CARGO_NET_OFFLINE=true
TMPDIR=/home/remi/dev/rum/.temp/brief-609/tmp
RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1
```

Formatting output: none; exit 0.

Rust compilation completed without compiler errors or warnings:

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 35.33s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5f7f08d1c3c3a0e1)

running 25 tests
```

The inventory-independent report emitted 79 UnknownHostPayload rows, 7,716 UnknownConstruct rows and 59 self-reaching rows. All 79 host rows and all 59 self-reaching rows are byte-identical to the current inventories. The complete report is retained in `r7/ownership-discovery.txt`; its extracted comparison is `r7/discovery-summary.json`. The only UnknownConstruct set difference is this same-closure span move:

```text
REMOVED (old span):
UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/adapter.rs	79	18	closure captures: |value| (capture types/modes unresolved)
ADDED (new span):
UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/adapter.rs	75	41	closure captures: |value| (capture types/modes unresolved)
```

All four failed tests, verbatim from the single ownership run:

```text
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... 
thread 'tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs:146:57:
check 3: opaque ownership via rum_runtime::runtime::suspension::typed::action::ErasedFinalizer has no verified path to Expr/SharedExpr/SharedArgs; cannot attribute Expr debt
row: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	22	14	rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
```

```text
test tests::ownership_graph::host_payload_ledger::tests::adapter_production_witnesses_are_computed ... ADAPTER_WITNESS	UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	20	15	rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:33:1) => rum_runtime::runtime::suspension::typed::TypedFrame => rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame (impl at crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:40:1); rum_runtime::runtime::eval_call_prepare::resume_frames::PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame.state -> rum_runtime::runtime::eval_call_prepare::SuspendedCallArgumentPreparation ; rum_runtime::runtime::eval_call_prepare::SuspendedCallArgumentPreparation.head -> rum_core::expr::Expr

thread 'tests::ownership_graph::host_payload_ledger::tests::adapter_production_witnesses_are_computed' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs:268:71:
called `Result::unwrap()` on an `Err` value: "opaque ownership via rum_runtime::runtime::suspension::typed::action::ErasedFinalizer has no verified path to Expr/SharedExpr/SharedArgs; cannot attribute Expr debt"
FAILED
```

```text
test tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected ... 
thread 'tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs:70:5:
assertion `left == right` failed
  left: "check 3: opaque ownership via rum_runtime::runtime::suspension::typed::action::ErasedFinalizer has no verified path to Expr/SharedExpr/SharedArgs; cannot attribute Expr debt\nrow: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/action.rs\t22\t14\trum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer\ncheck 1: missing ledger entry: UnknownHostPayload\ttest-side-suspension-copy.rs\t1\t1\trum_runtime::runtime::suspension::core::SuspendedEvaluation.unrelated_payload/fixture::OtherChain<arg0> via payload/std::boxed::Box[Own]/arg0: opaque owned payload dyn fixture :: OtherPayload"
 right: "check 1: missing ledger entry: UnknownHostPayload\ttest-side-suspension-copy.rs\t1\t1\trum_runtime::runtime::suspension::core::SuspendedEvaluation.unrelated_payload/fixture::OtherChain<arg0> via payload/std::boxed::Box[Own]/arg0: opaque owned payload dyn fixture :: OtherPayload"
FAILED
```

```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 79 rows

thread 'tests::ownership_graph::production_inventory_is_exact' panicked at crates/rum-stdlib/src/tests/ownership_graph/mod.rs:73:37:
UnknownConstruct inventory mismatch:
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/adapter.rs	75	41	closure captures: |value| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/adapter.rs	79	18	closure captures: |value| (capture types/modes unresolved)
FAILED
```

```text
failures:

failures:
    tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed
    tests::ownership_graph::host_payload_ledger::tests::adapter_production_witnesses_are_computed
    tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected
    tests::ownership_graph::production_inventory_is_exact

test result: FAILED. 21 passed; 4 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 67.17s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

The missing/stale negative-control failure is a consequence of the new production attribution failure also appearing in its diagnostic; the control was not relaxed. The exact-inventory failure is the 79:18 → 75:41 closure location. Neither explains away the two independent production witness failures. No disposition chain was emitted for the blanket shape.

The analyzer gap is concrete. `ownership_graph/host_payload_ledger/ownership.rs:121` first looks for concrete nominal implementations and then delegates at line172. `host_payload_ledger/adapters.rs:111` supports a nominal generic adapter with an owned field; lines122–123 require the implementation self type to resolve to a source definition, and the following checks require its generic argument and owned parameter field. Bare parameter F in impl<F: SlotFinalizer> ErasedFinalizer for F is not that nominal adapter shape. There is no blanket-bound implication witness. The analyzer therefore reaches the explicit “no verified path” rejection at ownership.rs:176. These files and every existing control remain byte-identical. Extending this proof requires a director decision; it was not attempted as an inventory update.

The source edit expires retention fingerprints and certificate cells. They were not refreshed after the mandatory STOP, and no old certificate is claimed current. No frame-root or census pin replay was performed. The scope audit preserves the entire analyzer, all inventories, the public API and every file outside the two edits plus this report.

Not run on R7 source: typed runtime tests; frame-root tests; the 48-level test; either census or 31-pair compile-control build; either complete suspension oracle or ordinary-control build; local-production acceptance in either build; fingerprint/certificate replay; frontier and compiler-instrument replays; the full workspace; native ZeroMQ tests; and every native stack/performance measurement. No native skip is counted as a pass: native tests did not run. Section C remains incomplete because the amendment explicitly stops this path first.

### 5. Allocation, finish-path, instruction, stack and timing status

No new measurement binary or performance panel was produced. The source removes the second Box construction from the erased finish method, but no instruction, allocation or transfer result is inferred from source alone. In particular, the required 7N + 5 → 7N + 4 call-repeat intercept, absence of the 56-byte allocation, absence of replacement allocations, single legacy-equivalent free and absence of stack payload transfers have not been measured. The other four workloads have not been rechecked either.

The 59 ordinary flags/copy/equality gates, three leaves, all four native-stack limits, complete 64-label allocation/instruction panels, five-workload resume attribution, repeats and timing remain unrun. No R6 measurement is carried over as R7 acceptance, and section 5 does not use R6 evidence to clear an unmeasured new binary. The four S2 bases and four historical R6b candidates retain their hashes; preserving those artifacts is not a measurement of this candidate. Timing did not run because the prerequisite STOP was reached.

### 6. S2 cold-helper retype and hot-boundary status

The earlier continuation_fault_outcome retype, four callers and two test callers are unchanged. No R7 release binary exists, so none of the four address-level boundary comparisons was repeated. R6's accepted guard/spill result is historical; this report makes no new assembly-identity or hot-boundary claim.

### 7. Stop-loss, scope audit and remaining decision

One source-shape attempt was made, with zero compiler errors and one ownership-suite run. The named allocation-intercept and finish-transfer gates were not reached, so their consecutive no-progress counter remains zero. This is the separate mandatory ownership STOP, not the two-attempt stop-loss. Formatting is the only subsequent code check; all remaining work is preservation, read-only diagnosis, scope verification and this report.

The working tree retains the two-file candidate for review; the stale witness and closure inventory are intentionally visible. No weaker finalizer shape, hand-written witness, analyzer exemption, relaxed test, per-frame mode, public raw API or additional source edit was attempted. The next decision is whether to authorize an ownership proof for the blanket trait-bound implementation. Only after that proof emits the chain and all replays pass can the full final-source section C and acceptance measurements resume.

The R7 entry snapshot covers 2027 files. The audit confirms only action.rs and adapter.rs changed before this appended report; no new source file, unsafe, manifest, lockfile, toolchain or out-of-scope edit occurred. All 183 protected inputs and eight historical measurement binaries are preserved, as are all entry bytes of this brief and the director-owned documents. No git command, network request or sub-agent was used. The tree is left uncommitted for the director. Raw patch and checks: `r7/candidate.patch`, `r7/audit.json`, `r7/discovery-summary.json` and `r7-ownership-discovery.log`.


## Director amendment — redispatch 8: the ledger learns the blanket erasure

The STOP was correct. The diagnosis matches the source.
`adapters.rs::adapter` requires the implementation's self type to resolve
to a nominal definition with one owned parameter field. Bare `F` in
`impl<F: SlotFinalizer> ErasedFinalizer for F` is not that shape, so
`ownership.rs` reaches its "no verified path" rejection.

### Ruling: keep the blanket shape, and extend the existential proof to it

The blanket impl is the right shape: S2's legacy finalizer is
`PureFrameFinalizer::finish(self: Box<Self>)`, called on its own box with
no transfer. Your two-file candidate reproduces exactly that. No nominal
shape gets there without `unsafe` or a payload move. A by-value
`SlotFinalizer::finish` would move 56 B out of the box, which the
acceptance forbids.

So the analyzer must model the shape. A blanket implementation over the
type parameter itself is the degenerate erasure adapter: the erased value
*is* the bounded `F`, owned directly by the erased box. The existing rule's
soundness argument carries over unchanged.

**Authorized files** (the R7 candidate's two runtime files remain authorized):

- `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/adapters.rs`
- `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/ownership.rs`,
  only if the delegation needs it
- `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs`
- the inventories and ledgers the change moves, re-derived by discovery

**The rule.** Accept an implementation whose self type is exactly its sole
type parameter only when every one of the following holds:

- **The impl's own form.** It is positive and non-conditional, has exactly
  one type parameter, and that parameter has exactly one ordinary trait
  bound under the existing `bound()` rule. Self is the bare parameter path:
  not `&F`, `Box<F>`, `Vec<F>` or a qualified path.
- **The trait.** It is not public, by the existing `public()` check.
- **Every other implementation of the trait** still validates. It is either
  a nominal adapter under the existing rule or another blanket under this
  one. One unsupported implementor rejects the whole trait, as today.
- **The witness hop** names the blanket and its bound:
  `{trait} => impl<F: {bound}> for F (impl at …) => {bound} => {concrete} (impl at …); {path}`.
- **The bound** must have an in-tree concrete implementor whose ownership
  path reaches `EXPRESSIONS`, by the existing search. The public-bound
  interim witness applies to a blanket exactly as it does to an adapter, no
  more.

Update the module doc comment to name both shapes.

**Watched falsifiers.** Add these as fixtures beside the existing
`adapter_fixture` tests, in the same style. Show each positive control
failing with the new rule disabled, then passing with it restored:

1. A positive blanket over a bound with a concrete implementor reaching
   `Expr` emits the exact hop.
2. The same with two bounds, or a bound with generic arguments: rejected.
3. Self `Box<F>`, `&F` or `Vec<F>` written as a blanket (it is not one):
   not accepted by this rule.
4. A public trait: rejected.
5. A blanket whose bound has no implementor reaching `Expr`: rejected,
   unless the existing interim rule applies.
6. The trait has a second, unsupported implementor beside a valid blanket:
   the trait is rejected.

`adapter_production_witnesses_are_computed` must then compute the
production `Action.Finalize.0` chain through the blanket, and the ledger
row must carry that computed chain. The existing negative control
(`missing_stale_and_unrelated_expression_debt_are_rejected`) passes
unchanged; its expected text is not edited.

**Acceptance.** Everything in redispatch 7's acceptance, unchanged:

- call-repeat 7N + 5 → 7N + 4 in all three pairs;
- the finish path: no allocation, no extra free, no payload transfer
  through a stack temporary;
- section C in full on the final source, including every replay the
  runtime change and the analyzer change expire, both builds, the full
  panel, resume attribution, and timing last.

**Stop-loss.** The named gates are the six falsifiers, the production
chain, the allocation intercept and the finish-path transfers. Two
attempts that move none of them is a stop. A soundness doubt about the
rule is a STOP to report, not something to resolve by narrowing the rule
silently.

Append a new `## Execution report` after this section.
