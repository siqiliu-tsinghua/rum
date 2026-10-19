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


---

## Director continuation 1 (2026-10-20) — scope by class: whatever the new entry touches

The stop was correct, and the miss is mine again: I listed the queue files
instead of writing the class. The base evidence is accepted: four
release baselines hashed, all 44 stack fits equal to `expected-22-c1.txt`,
and the base layouts recorded.

**The Scope is widened by class.**

1. **Every exhaustive match on the new variant's types.** Any match on
   `Entry`, `EntryRef`, the queued entry or their projections, anywhere in
   `crates/`, may gain an explicit arm for the new variant, and nothing
   else. The compiler's `E0004` list is the inventory. Disclose each site.
2. **The independent occurrence oracle**
   (`crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs`).
   It may change for two things only:
   - the marker's explicit no-guest-storage arm;
   - an independent account of guest queue capacity separate from
     diagnostic overhead, so the guest's charged bytes stay exactly what
     they would be without markers.

   Keep its arithmetic independent of the production `storage_bytes`.
   Do not call production accounting from it. Keep every assertion it has.
3. **Anything else the compiler requires for the new variant.** Disclose
   it. If it is more than an arm or an accounting split, STOP.

Everything else in the brief stands, including the STOP conditions.
Continue from the checkpoint at step 2.

---

## Executing-agent report — 2026-10-20 — STOP at occurrence-oracle scope

LP02 is **not implemented**. The four required unedited release baselines are built and hashed; the supplied stack calibration passes all 44 debug/release fits exactly. Before any production edit, inspection found a required exhaustive match in an out-of-scope independent accounting oracle. A scratch-only compiler probe confirms the blocker with exactly one `E0004`.

### 1. Files changed and inventory rows

The only non-scratch file changed by this run is this brief, by appending this report. No production source, test, manifest, lockfile, existing frame field, other brief, or inventory was edited. Every inventory row is unchanged; no replay is required because no source fingerprint changed. The pre-report hash audit found zero changed non-scratch files. No git command, network access, or new unsafe was used.

All scratch is under `.temp/brief-626/`. The supplied `base-source/`, `calib/{build.sh,measure.py}`, `stack-base/{probe.rs,expected-22-c1.txt}`, and `base-hashes.txt` are unchanged. New scratch includes:

- `entry-hashes.json`, `originals/`, `base-source-comparison.json`, and `pre-report-audit.json`: initial content and preservation checks.
- `resume-harness/` and `perf/`: copies of the authorized brief-622 harness and all 313 input labels (192 ordinary, 121 resume); no new rum probes.
- `base/{rum-cli,rum-cli-alloc,resume,resume-alloc}`, `target-*`, `builds/`, and `logs/`: unedited build outputs and hashes. The four release kinds use separate target directories and one codegen unit; `build.py` preserves these exact target paths for a future candidate.
- `calib/{tree,debug,release,target-debug,target-release,slopes.json}` and `stack-base-comparison.json`: supplied calibration and its exact comparison.
- `layout-base/`, `collect-layouts.gdb`, and `target-layout/`: production DWARF extraction and compiler size cross-checks.
- `scope-probe/`: an isolated copy of the supplied source, two minimally edited scratch files, raw compiler JSON/stderr, and rendered errors proving the scope block.
- Copied measurement helpers, `compare-stack-base.py`, `REPORT.md`, `PROGRESS.md`, and `state.json`: continuation scaffolding and this evidence. No copied performance helper was executed.

### 2. The engine and the entry — not started

The existing independent oracle in `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs:28-46` exhaustively matches `EntryRef`. It is expanded by `suspension/core.rs:889-891`. Adding the borrowed profile-boundary variant requires an explicit no-guest-storage arm in that oracle. The file is not among the queue/driver files listed in this brief, and it is not an inventory covered by the standing maintenance exception.

The same oracle independently charges `Mixed` queue capacity times `size_of::<Queued<Entry>>()` at lines 12-18. Inline diagnostic entries can widen or grow that queue. The diagnostic-storage ruling therefore also requires preserving an independent account of the guest queue capacity while excluding diagnostic overhead. Updating only production `storage_bytes` would leave this independent oracle charging a different category. Replacing the independent oracle with a call to the production accounting function would not preserve its contract.

`TEMPLATE.md` requires: “Do NOT modify files outside Scope. If you believe you must, STOP and write that in the report instead.” This run stops at that boundary. It does not alias a profile marker to an evaluation-return event, remove the accounting oracle, or substitute a different mechanism.

**Concrete continuation requirement:** add `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs` to scope for the explicit marker arm and independent guest-versus-diagnostic queue storage accounting, preserving its assertions and independent arithmetic. No phase counting change or existing frame-field change has been made or shown necessary. The diagnostic-storage invariant is not yet established.

### 3. Route proofs — not run

Zero LP02 routes are proven. No engine, boundary insertion operations, logical context leases, terminal-transition tests, cap-exhaustion tests, or native lifecycle spans were added. Slot identity, all terminal routes, second pause, cancellation/reset/drop, failed restore/rollback/helper discard/fault/nested cleanup, interleaving/thread migration, and profile reset remain outstanding.

The scope probe is solely a Rust exhaustiveness check. It adds `EntryRef::ProfileBoundary(&())` in its scratch `entry.rs`, covers that variant in the in-scope entry methods and driver match, and leaves the independent oracle untouched. It neither generates guest encodings nor executes guest code.

### 4. Counter oracle — not run

No logical-profile receipt was executed in this run, and no pinned-loss-cell replay was performed. Production source hashes are unchanged, but that is not an executed oracle receipt. Every LP03-LP07 pinned cell remains subject to the required later replay.

### 5. Gates and compiler evidence, verbatim

All subprocess commands were bounded and offline. Required build/check environment: `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-626/tmp`. No inherited `RUM_PROFILE_*` variable was present. Each command used `python3 scripts/run-bounded.py --timeout 900 -- env ...`.

**Base rum-cli** — exit 0. Command after the bounded environment: `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-626/target-rum-cli cargo build --offline --release -p rum-cli --message-format=json`. The helper records its exact Cargo command and environment under `builds/`. Raw stderr:

```text
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
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
   Compiling thiserror v2.0.18
   Compiling cfg-if v1.0.4
   Compiling digest v0.10.7
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling num-bigint v0.4.6
   Compiling crc32fast v1.5.0
   Compiling cpufeatures v0.2.17
   Compiling libc v0.2.183
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.9
   Compiling cc v1.2.63
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling errno v0.3.14
   Compiling libmimalloc-sys v0.1.49
   Compiling signal-hook v0.3.18
   Compiling rustix v1.1.4
   Compiling signal-hook-registry v1.4.8
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling bitflags v2.11.0
   Compiling linux-raw-sys v0.12.1
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling mimalloc v0.1.52
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 1m 45s
```

**Base rum-cli-alloc** — exit 0. Command after the bounded environment: `python3 .temp/brief-626/build.py base rum-cli-alloc`. The helper records its exact Cargo command and environment under `builds/`. Raw stderr:

```text
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling syn v2.0.117
   Compiling version_check v0.9.5
   Compiling generic-array v0.14.7
   Compiling autocfg v1.5.0
   Compiling typenum v1.20.1
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling serde_derive v1.0.228
   Compiling cfg-if v1.0.4
   Compiling thiserror v2.0.18
   Compiling digest v0.10.7
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling num-bigint v0.4.6
   Compiling cpufeatures v0.2.17
   Compiling libc v0.2.183
   Compiling crc32fast v1.5.0
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling find-msvc-tools v0.1.9
   Compiling shlex v2.0.1
   Compiling cc v1.2.63
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling errno v0.3.14
   Compiling libmimalloc-sys v0.1.49
   Compiling signal-hook v0.3.18
   Compiling rustix v1.1.4
   Compiling signal-hook-registry v1.4.8
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling bitflags v2.11.0
   Compiling linux-raw-sys v0.12.1
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling mimalloc v0.1.52
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 1m 41s
```

**Base resume** — exit 0. Command after the bounded environment: `python3 .temp/brief-626/build.py base resume`. The helper records its exact Cargo command and environment under `builds/`. Raw stderr:

```text
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling syn v2.0.117
   Compiling version_check v0.9.5
   Compiling generic-array v0.14.7
   Compiling typenum v1.20.1
   Compiling autocfg v1.5.0
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
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.9
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
   Compiling brief608-resume v0.0.0 (/home/remi/dev/rum/.temp/brief-626/resume-harness)
    Finished `release` profile [optimized] target(s) in 1m 35s
```

**Base resume-alloc** — exit 0. Command after the bounded environment: `python3 .temp/brief-626/build.py base resume-alloc`. The helper records its exact Cargo command and environment under `builds/`. Raw stderr:

```text
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling syn v2.0.117
   Compiling version_check v0.9.5
   Compiling generic-array v0.14.7
   Compiling typenum v1.20.1
   Compiling autocfg v1.5.0
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
   Compiling sha2 v0.10.9
   Compiling sha1 v0.10.6
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling mimalloc v0.1.52
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling brief608-resume v0.0.0 (/home/remi/dev/rum/.temp/brief-626/resume-harness)
    Finished `release` profile [optimized] target(s) in 1m 36s
```

**Supplied stack calibration build** — exit 0. Command: `bash .temp/brief-626/calib/build.sh`. Raw build stderr and executable hashes:

```text
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling syn v2.0.117
   Compiling autocfg v1.5.0
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling serde_derive v1.0.228
   Compiling thiserror v2.0.18
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling num-bigint v0.4.6
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/.temp/brief-626/calib/tree/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/.temp/brief-626/calib/tree/crates/rum-syntax)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/.temp/brief-626/calib/tree/crates/rum-derive)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/.temp/brief-626/calib/tree/crates/rum-runtime)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 50.68s
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling syn v2.0.117
   Compiling autocfg v1.5.0
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling serde_derive v1.0.228
   Compiling thiserror v2.0.18
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling num-bigint v0.4.6
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/.temp/brief-626/calib/tree/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/.temp/brief-626/calib/tree/crates/rum-syntax)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/.temp/brief-626/calib/tree/crates/rum-derive)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/.temp/brief-626/calib/tree/crates/rum-runtime)
    Finished `release` profile [optimized] target(s) in 1m 31s
f1a10797bb945f68279815cfe42fc844b5c295490d8106c97f45fa1e7b299f12  /home/remi/dev/rum/.temp/brief-626/calib/debug/runtime-tests
fd3b0551ebc27154678f3931f6ea67cb8c18558cf869b5b8f58cc075440b8d2c  /home/remi/dev/rum/.temp/brief-626/calib/release/runtime-tests
```

**Supplied stack measurement** — exit 0. Command: `python3 .temp/brief-626/calib/measure.py`. All 528 native test invocations completed; this is baseline stack evidence, not logical-engine route coverage. Raw output:

```text
debug peel 19024.0 resid 0.0 top 189848
debug plus 34912.0 resid 0.0 top 313000
debug sub 29376.0 resid 0.0 top 267128
debug up 28064.0 resid 0.0 top 255320
debug guarded 35840.0 resid 0.0 top 320424
debug rhs-guard 49152.0 resid 0.0 top 445864
debug replace-guard 64880.0 resid 0.0 top 552744
debug replaceall-guard 63216.0 resid 0.0 top 539432
debug count-guard 74064.0 resid 0.0 top 626216
debug position-guard 85648.0 resid 0.0 top 718888
debug catch-body 50528.0 resid 0.0 top 437928
debug checkabort-body 52720.0 resid 0.0 top 455464
debug abortprotect-body 49904.0 resid 0.0 top 432936
debug cleanup-body 50880.0 resid 0.0 top 440744
debug check-body 50432.0 resid 0.0 top 437160
debug quiet-body 50304.0 resid 0.0 top 436136
debug catchexceptions-body 49872.0 resid 0.0 top 432680
debug function-body 49312.0 resid 0.0 top 428200
debug status-body 50816.0 resid 0.0 top 440232
debug message-body 50816.0 resid 0.0 top 440232
debug lhs-arg 57376.0 resid 0.0 top 492712
debug replaceall-rhs 52720.0 resid 0.0 top 455464
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
release function-body 19312.0 resid 0.0 top 397689
release status-body 20432.0 resid 0.0 top 420089
release message-body 20432.0 resid 0.0 top 420089
release lhs-arg 18048.0 resid 0.0 top 372409
release replaceall-rhs 21488.0 resid 0.0 top 441209
```

**Exact expected-table comparison** — exit 0. Command: `python3 .temp/brief-626/compare-stack-base.py`. Raw output:

```text
profile shape expected measured residual status
debug peel 19024 19024.0 0.0 PASS
debug plus 34912 34912.0 0.0 PASS
debug sub 29376 29376.0 0.0 PASS
debug up 28064 28064.0 0.0 PASS
debug guarded 35840 35840.0 0.0 PASS
debug rhs-guard 49152 49152.0 0.0 PASS
debug replace-guard 64880 64880.0 0.0 PASS
debug replaceall-guard 63216 63216.0 0.0 PASS
debug count-guard 74064 74064.0 0.0 PASS
debug position-guard 85648 85648.0 0.0 PASS
debug catch-body 50528 50528.0 0.0 PASS
debug checkabort-body 52720 52720.0 0.0 PASS
debug abortprotect-body 49904 49904.0 0.0 PASS
debug cleanup-body 50880 50880.0 0.0 PASS
debug check-body 50432 50432.0 0.0 PASS
debug quiet-body 50304 50304.0 0.0 PASS
debug catchexceptions-body 49872 49872.0 0.0 PASS
debug function-body 49312 49312.0 0.0 PASS
debug status-body 50816 50816.0 0.0 PASS
debug message-body 50816 50816.0 0.0 PASS
debug lhs-arg 57376 57376.0 0.0 PASS
debug replaceall-rhs 52720 52720.0 0.0 PASS
release peel 7344 7344.0 0.0 PASS
release plus 13792 13792.0 0.0 PASS
release sub 13168 13168.0 0.0 PASS
release up 11456 11456.0 0.0 PASS
release guarded 14240 14240.0 0.0 PASS
release rhs-guard 17920 17920.0 0.0 PASS
release replace-guard 26768 26768.0 0.0 PASS
release replaceall-guard 24640 24640.0 0.0 PASS
release count-guard 28416 28416.0 0.0 PASS
release position-guard 33632 33632.0 0.0 PASS
release catch-body 19280 19280.0 0.0 PASS
release checkabort-body 20864 20864.0 0.0 PASS
release abortprotect-body 19280 19280.0 0.0 PASS
release cleanup-body 19280 19280.0 0.0 PASS
release check-body 18736 18736.0 0.0 PASS
release quiet-body 18688 18688.0 0.0 PASS
release catchexceptions-body 19472 19472.0 0.0 PASS
release function-body 19312 19312.0 0.0 PASS
release status-body 20432 20432.0 0.0 PASS
release message-body 20432 20432.0 0.0 PASS
release lhs-arg 18048 18048.0 0.0 PASS
release replaceall-rhs 21488 21488.0 0.0 PASS
STACK BASE 44/44 exact; mismatches=0
```

**Additional production debug build for DWARF layouts** — exit 0. Command after the bounded environment: `CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-626/target-layout cargo build --offline -p rum-cli --message-format=json`. Raw stderr:

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
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling serde_derive v1.0.228
   Compiling thiserror v2.0.18
   Compiling cfg-if v1.0.4
   Compiling digest v0.10.7
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling num-bigint v0.4.6
   Compiling crc32fast v1.5.0
   Compiling libc v0.2.183
   Compiling cpufeatures v0.2.17
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.9
   Compiling cc v1.2.63
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling sha2 v0.10.9
   Compiling sha1 v0.10.6
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling errno v0.3.14
   Compiling libmimalloc-sys v0.1.49
   Compiling signal-hook v0.3.18
   Compiling rustix v1.1.4
   Compiling signal-hook-registry v1.4.8
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling bitflags v2.11.0
   Compiling linux-raw-sys v0.12.1
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling mimalloc v0.1.52
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 58.32s
```

**DWARF extraction** — both invocations exited 0, with GDB auto-load warnings. The first parser expected semicolons and returned no rows; this was not a layout pass. The corrected Rust-format parser recorded 789 type-size rows with zero lookup exceptions. `Entry` resolved to an opaque zero-sized declaration, so its raw GDB size is not accepted; the compiler cross-check below records 24 B. Full raw output is in `logs/base-layouts.log`, `logs/base-layouts-rust-format.log`, and `layout-base/types.txt`. Verbatim summaries:

```text
warning: Missing auto-load script at offset 0 in section .debug_gdb_scripts
of file /home/remi/dev/rum/.temp/brief-626/target-layout/debug/rum-cli.
Use `info auto-load python-scripts [REGEXP]' to list them.
LAYOUTS sizes=0 failures=0
warning: Missing auto-load script at offset 0 in section .debug_gdb_scripts
of file /home/remi/dev/rum/.temp/brief-626/target-layout/debug/rum-cli.
Use `info auto-load python-scripts [REGEXP]' to list them.
LAYOUTS sizes=789 failures=0
```

**Scope-only compiler probe** — expected exit 101; exactly one error, `E0004`. Command after the bounded environment: `CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-626/scope-probe/target cargo check --offline --manifest-path .temp/brief-626/scope-probe/tree/Cargo.toml -p rum-runtime --tests --message-format=json`. Verbatim diagnostic and final stderr:

```text
error[E0004]: non-exhaustive patterns: `&typed::entry::EntryRef::ProfileBoundary(_)` not covered
   --> crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs:29:27
    |
29  |                     match &frame {
    |                           ^^^^^^ pattern `&typed::entry::EntryRef::ProfileBoundary(_)` not covered
    |
   ::: crates/rum-runtime/src/runtime/suspension/core.rs:892:5
    |
892 |     crate::runtime::debug_tools::resources::occurrence::suspension::define_oracle!();
    |     -------------------------------------------------------------------------------- in this macro invocation
    |
note: `typed::entry::EntryRef<'_>` defined here
   --> crates/rum-runtime/src/runtime/suspension/typed/entry.rs:15:41
    |
15  | pub(in crate::runtime::suspension) enum EntryRef<'a> {
    |                                         ^^^^^^^^
16  |     #[allow(dead_code)]
17  |     ProfileBoundary(&'a ()),
    |     --------------- not covered
    = note: the matched value is of type `&typed::entry::EntryRef<'_>`
    = note: this error originates in the macro `crate::runtime::debug_tools::resources::occurrence::suspension::define_oracle` (in Nightly builds, run with -Z macro-backtrace for more info)
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
    |
45  ~                         },
46  +                         &typed::entry::EntryRef::ProfileBoundary(_) => todo!()
    |
error: could not compile `rum-runtime` (lib test) due to 1 previous error
```

**Production header size queries** — expected exit 101, exactly seven `E0308` diagnostics. A second scratch-only `cargo check ... --lib --message-format=json` adds array-length mismatches containing `size_of` queries; it executes no tests. Entry and all header types are unchanged from production. Raw compiler output:

```text
error[E0308]: mismatched types
   --> crates/rum-runtime/src/runtime/suspension/core.rs:895:43
    |
895 | const LP02_LAYOUT_QUEUED_ENTRY: [(); 0] = [(); size_of::<Queued<Entry>>()];
    |                                      -    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected an array with a size of 0, found one with a size of 40
    |                                      |
    |                                      help: consider specifying the actual array length: `40`


error[E0308]: mismatched types
   --> crates/rum-runtime/src/runtime/suspension/core.rs:897:44
    |
897 | const LP02_LAYOUT_QUEUED_LEGACY: [(); 0] = [(); size_of::<Queued<Arc<DynSuspensionFrame>>>()];
    |                                       -    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected an array with a size of 0, found one with a size of 32
    |                                       |
    |                                       help: consider specifying the actual array length: `32`


error[E0308]: mismatched types
   --> crates/rum-runtime/src/runtime/suspension/core.rs:899:42
    |
899 | const LP02_LAYOUT_FRAME_QUEUE: [(); 0] = [(); size_of::<FrameQueue>()];
    |                                     -    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected an array with a size of 0, found one with a size of 40
    |                                     |
    |                                     help: consider specifying the actual array length: `40`


error[E0308]: mismatched types
   --> crates/rum-runtime/src/runtime/suspension/core.rs:901:45
    |
901 | const LP02_LAYOUT_RESUME_CONTEXT: [(); 0] = [(); size_of::<ResumeContext>()];
    |                                        -    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected an array with a size of 0, found one with a size of 16
    |                                        |
    |                                        help: consider specifying the actual array length: `16`


error[E0308]: mismatched types
   --> crates/rum-runtime/src/runtime/suspension/core.rs:903:51
    |
903 | const LP02_LAYOUT_SUSPENDED_EVALUATION: [(); 0] = [(); size_of::<SuspendedEvaluation>()];
    |                                              -    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected an array with a size of 0, found one with a size of 80
    |                                              |
    |                                              help: consider specifying the actual array length: `80`


error[E0308]: mismatched types
   --> crates/rum-runtime/src/runtime/suspension/core.rs:905:38
    |
905 | const LP02_LAYOUT_SESSION: [(); 0] = [(); size_of::<Session>()];
    |                                 -    ^^^^^^^^^^^^^^^^^^^^^^^^^^ expected an array with a size of 0, found one with a size of 2032
    |                                 |
    |                                 help: consider specifying the actual array length: `2032`


error[E0308]: mismatched types
  --> crates/rum-runtime/src/runtime/suspension/typed/entry.rs:83:36
   |
83 | const LP02_LAYOUT_ENTRY: [(); 0] = [(); size_of::<Entry>()];
   |                               -    ^^^^^^^^^^^^^^^^^^^^^^^^ expected an array with a size of 0, found one with a size of 24
   |                               |
   |                               help: consider specifying the actual array length: `24`
error: could not compile `rum-runtime` (lib) due to 7 previous errors
```

**Mandatory verification not run because of the scope STOP:**

- `cargo fmt --check` — not run.
- `cargo test -p rum-runtime --lib profiling` — not run.
- `cargo test -p rum-runtime --lib suspension::typed::tests` — not run.
- `cargo test -p rum-runtime --lib frame_root_trace` — not run.
- `cargo test -p rum-runtime --lib resume_evaluation_context` — not run.
- `cargo test -p rum-stdlib --test typed_slot_census` — not run.
- `cargo test -p rum-stdlib --no-default-features --test typed_slot_census` — not run.
- `scripts/check-suspension-oracle.sh 1500` — not run.
- `cargo test -p rum-stdlib --lib ownership_graph` — not run.
- `cargo test -p rum-stdlib --lib local_production_acceptance` — not run.
- `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance` — not run.
- `cargo test -p rum-stdlib --lib production_rejection_census_is_complete` — not run.
- `cargo test --workspace --no-fail-fast` — not run.

The workspace was not run: no nine-native-ZeroMQ skip receipts are claimed. The scope and size-query failures above are intentional scratch compiler diagnostics, not failures of the untouched production tree.

### 6. Measurements

**Release bases, one codegen unit:**

```text
46137abe2e63c59203ef987effb3ca5eb1f78540c9a88200ae51220041b845fd  .temp/brief-626/base/rum-cli
5cf3e4b67b750e904731977d4618c9aa2796e170781243f9ac3451760ec5a843  .temp/brief-626/base/rum-cli-alloc
6fdc74e0cb0027244941c2032ab480843eae87caa7b409d44e0caf86735743e5  .temp/brief-626/base/resume
f305fd523786f8b4a7e658e4ca99cbdab87bae8e92fab5bfce9cc7137a85515c  .temp/brief-626/base/resume-alloc
```

**Stack:** 22 debug plus 22 release shapes equal the supplied expected values, with zero residuals; exact rows are above. Candidate stack and resumed chains were not measured.

**Production header sizes, compiler verified:**

```text
type	bytes
QUEUED_ENTRY	40
QUEUED_LEGACY	32
FRAME_QUEUE	40
RESUME_CONTEXT	16
SUSPENDED_EVALUATION	80
SESSION	2032
ENTRY	24
```

The DWARF inventory additionally contains 96 zero-sized concrete `ContinueToken` instantiations, 96 `TypedAdapter` instantiations, and other frame/layout rows. Raw type sizes are saved in `layout-base/sizes.json`; these have not been reconciled against an exhaustive production frame population, so the full per-frame layout gate remains incomplete. The unaccepted opaque GDB Entry row is retained as raw evidence rather than silently normalized. No candidate layout comparison ran.

**Allocations, instruction slopes, copies, resumed chains, four timing panels and aligned whole-program timing:** not run for either measurement arm. Base measurement binaries and exact label inputs are retained. No profiling-off performance verdict is claimed.

### 7. Stop-loss counters, deviations and open question

- Routes proven: **0**.
- Counter receipts kept by executed replay: **0**.
- Measurement prerequisite bars met: **44** exact base stack fits. The seven compiler header sizes are recorded base bars; no candidate bar is claimed.
- Production implementation attempts: **0**; consecutive no-progress attempts on a compiling implementation: **0**. Stop-loss did not fire; the explicit scope STOP did.
- Half-edited production files: **none**. The production tree compiled for all four release baselines and the additional debug CLI layout build.
- Deviation: implementation and mandatory post-change gates were not started because the required independent oracle edit lies outside scope. The additional scratch compiler checks make that boundary reviewable without touching production or weakening any test.

The director must settle the single scope extension named in section 2 before execution resumes. Then finish exhaustive base-frame layout coverage and proceed with the engine, insertion/lifecycle proofs, oracle and full candidate measurement campaign. `PROGRESS.md` and `state.json` retain the exact handoff. All changes remain uncommitted; no git was run.
