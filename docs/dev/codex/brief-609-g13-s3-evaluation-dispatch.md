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
