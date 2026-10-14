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
