# brief-626 — LP02: the logical profiler's record engine and queue boundaries

## Goal

This is step two of the logical-profiler line: the design is brief-624 and
its director review, and LP01 (brief-625, `3fd1885c`) froze the contract and
the counter oracle. **This step builds the engine and adopts no phase.**
Every existing profiled phase keeps counting exactly as today, so every
pinned LP03-LP07 loss cell stays exactly as pinned.

When you are done:

- **An enabled-only record engine** exists under
  `crates/rum-runtime/src/runtime/profiling/logical/`. It holds:
  - epoch, record and segment identities (`EpochId`, `RecordId`,
    `SegmentId`);
  - a process-wide epoch registry of open logical records, each with an
    immutable aggregate key, its parent record and its outcome policy;
  - segment accumulation;
  - idempotent terminal transitions.

  Identity checks replace the silent key-match fallback for records the
  engine owns.
- **`Entry::ProfileBoundary`** exists in the mixed frame queue, with a thin
  `Arc` payload.
  - It reads the passing `Result<Slot, EvalError>` by reference and leaves
    it byte- and identity-equal for its consumer. It runs no decoder or
    encoder, retains no guest value, and mints no `ContinueToken`.
  - Every `Entry` match handles it: abandonment, trace roots, output
    boundaries, storage bytes, widen, append, push-front, and the driver's
    Complete/Evaluate/Finalize branches.
- **Explicit insertion operations.** These cover the producer frontier,
  after a consumer, after `EvaluationReturn`, the zero-semantic-frame case,
  and spans begun after an `InputRequired` was already returned.
- **Every termination route.**
  - Success, error, `Return` and `Throw`.
  - A second pause.
  - Cancellation, reset and session drop.
  - Failed restore, rollback, helper discard, slot fault and nested
    cleanup.
  - A profile reset inside an active call.

  Each one closes or abandons each record exactly once, and none fabricates
  a success.
- **Diagnostic storage is a separate, bounded category** (the director's
  ruling 3 on brief-624). The engine's memory never counts against a
  guest's memory ceiling, and exhausting the diagnostic cap invalidates the
  profiling epoch visibly.
- **The engine is proven in isolation** by unit and native test spans that
  exercise every route.
- **Profiling off costs nothing,** and the measurements below show it.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`, `docs/dev/profiling.md`.
- **brief-624**: report §4 (*The mechanism*) and §5 (*The disabled-path cost
  plan*), the director review's five rulings, and `.temp/brief-624/DESIGN.md`
  with `inventory/routes.tsv`.
- **The raw draft** is `.temp/brief-624/drafts/LP02-record-epochs-queue-lifecycle.md`.
  This brief governs where they differ.
- **brief-625 (LP01)**: the observer and the counter oracle, and its gate
  record.
- **brief-622**: its gate record is your measurement template
  (allocations, instruction slopes, copies, stack shapes, chains, timing).

## Scope

A listed file or class is permission to do this step's work there. Disclose
every file you edit.

- **The engine.**
  - `crates/rum-runtime/src/runtime/profiling/{mod,timing,state,reset,summary}.rs`.
  - New modules under `crates/rum-runtime/src/runtime/profiling/logical/`.
  - New tests under `crates/rum-runtime/src/runtime/profiling/tests/`.
- **The queue and driver.** These files only, for the new entry and its
  lifecycle:
  - `crates/rum-runtime/src/runtime/suspension/typed/entry.rs`;
  - `suspension/core/queue.rs`;
  - `suspension/core/evaluation_context.rs`;
  - `suspension/core/abandon.rs`;
  - `suspension/core.rs`.
- **Owner identity and teardown.**
  - `crates/rum-runtime/src/runtime/model/request_cancellation.rs`, for an
    enabled-only identity accessor.
  - `crates/rum-runtime/src/runtime/session/session_state.rs`, for drop
    cleanup inside its existing enabled slow path.
  - `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs`,
    for cold context save and restore.
- **Tests and the standing duty.**
  - A new `crates/rum-host/tests/logical_profile_lifecycle.rs`.
  - The standing inventory duty (`TEMPLATE.md`), replayed, never
    hand-written.

`Expr`/`Symbol`, public host types, codecs, guest internal encodings, every
existing frame's fields, `Cargo.toml` files, `Cargo.lock`, `AGENTS.md`,
`TODO.md` and other briefs stay unchanged.

**STOP and report** if any of these happens:

- a phase's counting would have to change in this step;
- an existing frame would need a field;
- the diagnostic-storage invariant cannot be established.

Do not switch to another mechanism on your own.

## Steps

1. **Bases, before any source edit.** All scratch goes under
   `.temp/brief-626/`.
   - Build, from the unedited working tree, with one codegen unit and each
     in its own target directory:
     - `rum-cli`;
     - `rum-cli` with `rum-cli/alloc-count`;
     - brief-622's resume harness (`.temp/brief-622/resume-harness/`,
       copied here), and the same with `alloc-count`.

     Record their hashes. The candidate is rebuilt later at the same paths.
   - **Stack base.** Build with `calib/build.sh` (it uses the supplied
     `base-source/`, an export of `3fd1885c`) and measure with
     `calib/measure.py`. **All 22 shapes must equal
     `stack-base/expected-22-c1.txt`; if one does not, STOP.**
   - **Layouts.** Record the production sizes of `Entry`, the queued entry,
     every frame type the queue holds, `Session` and `ContinueToken` on the
     base. They are bars.
2. **The engine and the entry.** Build both, then the insertion operations
   and the termination routes. Keep the engine inert for every existing
   phase.
3. **Isolated proofs.** Unit and native test spans cover:
   - `Slot` identity through a marker;
   - every terminal route closing once;
   - zero open records and zero context leases at teardown;
   - rejection of a repeated key used as identity;
   - interleaved requests and a moved session;
   - a profile reset in the middle of a call;
   - cleanup that re-suspends;
   - diagnostic-cap exhaustion.
4. **The oracle stays as pinned.** Run LP01's counter oracle: every
   logical-profile receipt executes, and every pinned loss cell is
   unchanged, because no phase moved.

## Verification (MANDATORY — run to green or evidence)

**How to run.** Serially, each command through `python3
scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-626/tmp …`.
Bounds: workspace 3 600 s, oracle 1 500 s per build, everything else 900 s.

```
cargo fmt --check
cargo test -p rum-runtime --lib profiling
cargo test -p rum-runtime --lib suspension::typed::tests
cargo test -p rum-runtime --lib frame_root_trace
cargo test -p rum-runtime --lib resume_evaluation_context
cargo test -p rum-stdlib --test typed_slot_census
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
scripts/check-suspension-oracle.sh 1500
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
cargo test --workspace --no-fail-fast
```

Run the workspace with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` and report the nine
as skips; the director runs them natively.

**Performance** (profiling off throughout). This step adds code to the
queue and the driver, which every resume runs. The measurement set is
brief-622's, on brief-622's 313 labels (`.temp/brief-622/perf/`, copied
here):

1. **Allocations.** Every ordinary label is exactly equal, and every resume
   label is exactly equal, because profiling is off.
2. **Instruction slopes.** Two interleaved pairs per arm on all labels.
   Clear each flag by five runs, or attribute it by Callgrind.
3. **Copies.** The calibrated census. No label goes above base.
4. **Stack.** All 22 shapes, debug and release, at or below base. Report
   the resumed chains of brief-622's workloads, which must also be at or
   below base.
5. **Layouts.** Every type from step 1 at its base size.
6. **Timing.** Four panels, then the aligned whole-program probe.

A regression is a defect to fix or report.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- **No ordinary per-call cost.**
  - All allocation, registry access and locking happens after the existing
    cold-load enabled gates.
  - Do no key formatting or capture construction before the gate.
  - `Entry::ProfileBoundary` must not grow `Entry`. Measure it, do not
    assume it.
- **A phase key is not an identity.** Recursive equal-key calls exist.
  Records are identified by `(EpochId, RecordId)`, owners by the request
  control's identity plus its generation, never by a session address or a
  bare suspension generation.
- **No `Instant` is held across a host wait.** Durations are segment sums.
- **The registry must not form a strong cycle with a marker.**
- **The marker never consumes or rebuilds the `Slot`.** The consumer must
  receive the same value.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-626/`.
- After each step and each gate, write `PROGRESS.md` and `state.json`. They
  record:
  - the verified state;
  - the counters;
  - half-edited files;
  - the exact next action.

## Hard rules

- **No `git`, in any form.** No network, no new `unsafe`.
- Modify only the Scope. Never weaken an oracle `Checks` flag, a
  fault-absence gate, a retention reject or a documented test.
- No internal-encoding probe generation; rum against rum only.
- **Stop-loss is keyed on progress.** The counters are the routes proven,
  the receipts kept, and the measurement bars met. Stop after two attempts
  on a compiling tree that advance none of them.
- Paste real output. Say plainly what did not run. Leave the tree
  uncommitted.

## Report format (append to this brief)

1. **Files changed**, each with its purpose, and every inventory row
   change.
2. **The engine and the entry**, as built.
3. **The route proofs.**
4. **The oracle result**: every receipt executed, the pinned cells
   unchanged.
5. **All gates, verbatim.**
6. **Measurements**: allocations, instructions, copies, stack and chains,
   layouts, timing.
7. **Stop-loss counters, deviations and open questions.**
