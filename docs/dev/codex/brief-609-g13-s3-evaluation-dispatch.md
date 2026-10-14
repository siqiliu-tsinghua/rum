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
