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

## Director continuation 2 (2026-10-20) — attribute every flag, remove the off-path cost, no `#[path]` traversal

The engine, the entry and the route proofs are accepted in design:

- 19 routes proven;
- all 8 750 LP01 receipts kept, with every pinned cell unchanged;
- allocations, copies, stack shapes, chains and 789 layouts all at base.

Three things block the gate.

**1. Attribute every instruction flag down to the instruction, then
separate layout from work.**

- **One case attributed already.** I attributed `cases-wide-no-match`: its
  whole +116 821 (N=8) and +218 844 (N=16) sits in libc
  `__memcmp_avx2_movbe`. Every rum function is equal to within a few
  hundred instructions.
- **Why that is layout.** S5 met the same thing (`default-unguarded-miss`).
  glibc's small `memcmp`/`bcmp` takes a page-cross slow path when
  `(p1|p2) & 0xfff > 0xfe0`. Adding code and data moves string literals,
  so the same comparison can land on that path.
- **For every flagged label, do this.**
  - Diff the per-function self cost, base against candidate, including
    libc.
  - Where the delta sits in libc `memcmp`/`bcmp`/`memcpy` at equal call
    counts, identify the compared pointers and show the page-cross
    condition on both arms. That makes it **layout**: report it and leave
    it.
  - Anything else is **work**, and goes to point 2.
  - Report the split as counts and as the label list.
- **The resume labels' +250 to +1 500 instructions per iteration are work
  until shown otherwise.** These are `resume-table`, `resume-fold`,
  `resume-cases-shared` and the rest.

**2. Remove the disabled-path work.** This is the hard line: profiling off
must cost nothing.

- Per resume, the most the off path may pay is the existing cold-load
  enabled check, once.
- No per-entry match work, context-guard construction or drop, queue
  projection, or diagnostic-capacity bookkeeping may run when profiling is
  off. Typical cures: branch once on the enabled gate and keep the
  ordinary queue algorithm verbatim on the off path; outline the
  diagnostic arms `#[cold] #[inline(never)]`; keep diagnostic fields out of
  hot structs.
- Then rerun the instruction comparison. **The bar:**
  - no ordinary or resume label has a deterministic rum-function
    instruction increase;
  - every residual flag is attributed as layout by point 1's method.

**3. No `#[path]` with parent traversal.**

- `suspension/core.rs:18` and `:956`, `suspension/typed/entry.rs:141` and
  `suspension/core/queue.rs:5` declare modules through
  `#[path = "../…profiling/…"]`.
- rustc then reports `suspension/core/../../profiling/…` paths, and that is
  why `inherent_method_claims_match_rustc` fails 5 of 3 519.
- **The cure is the module tree, not the harness.**
  - Declare the engine's modules under `profiling/logical/` and its tests
    under `profiling/tests/`, from their own parents.
  - Reach them from the suspension code through `pub(in crate::runtime)`
    paths.
  - The harness stays unchanged, and the test must pass with 3 519 of
    3 519 agreements.

**Then rerun, on the final code:** every gate and every measurement of this
brief. That is the workspace, the oracle with all 8 750 receipts, the
inventories, allocations, instructions with point 1's attribution, copies,
stack shapes and chains, layouts and timing. For timing, the ten aligned
regressions get the same layout-or-work split: equal instructions, or an
attributed residual.

Stop-loss as before: progress is flags attributed or removed.

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

---

## Executing-agent continuation report — LP02 engine and measured defects

The engine and its isolated proofs are implemented, and every LP01 receipt and pinned loss cell is unchanged. **This candidate fails the disabled-path gate**: instruction flags and prologue growth remain. The workspace also remains red on five method-oracle path-spelling comparisons, independently evidenced below. This report supersedes the earlier pre-continuation STOP report without modifying it.

### 1. Files changed and every inventory row change

The scope audit finds 20 changed and 13 new files, all authorized, with no missing or outside-scope file. The director continuation authorizes the independent occurrence-oracle arm and accounting split. The original scratch exhaustiveness probe found one `E0004`, at `runtime/debug_tools/resources/occurrence/suspension.rs:29`; that is the only compiler-discovered outside-original-scope site. No additional class-widened production match was needed.

| File | Purpose |
|---|---|
| [crates/rum-runtime/src/runtime/profiling/mod.rs](../../../crates/rum-runtime/src/runtime/profiling/mod.rs) | Declare the inert logical engine. |
| [crates/rum-runtime/src/runtime/profiling/reset.rs](../../../crates/rum-runtime/src/runtime/profiling/reset.rs) | Retire the logical epoch inside the existing enabled reset path. |
| [crates/rum-runtime/src/runtime/profiling/summary.rs](../../../crates/rum-runtime/src/runtime/profiling/summary.rs) | Close session owners before enabled summary and expose epoch validity. |
| [crates/rum-runtime/src/runtime/session/session_state.rs](../../../crates/rum-runtime/src/runtime/session/session_state.rs) | Pass the dropping session into enabled profiling teardown. |
| [crates/rum-runtime/src/runtime/model/request_cancellation.rs](../../../crates/rum-runtime/src/runtime/model/request_cancellation.rs) | Expose enabled-only control generation without adding fields. |
| [crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs](../../../crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs) | Cold capture/swap of source scopes for diagnostic leases. |
| [crates/rum-runtime/src/runtime/suspension/typed/entry.rs](../../../crates/rum-runtime/src/runtime/suspension/typed/entry.rs) | Thin ProfileBoundary variant, read-only Slot classification, explicit projection/root/output/abandon/accounting arms. |
| [crates/rum-runtime/src/runtime/suspension/core.rs](../../../crates/rum-runtime/src/runtime/suspension/core.rs) | Drive inline markers, choose diagnostic resumption mode, preserve front/context during abandonment, and count inherited re-pauses. |
| [crates/rum-runtime/src/runtime/suspension/core/queue.rs](../../../crates/rum-runtime/src/runtime/suspension/core/queue.rs) | Keep ordinary queue algorithms and add enabled-only diagnostic representation/projections. |
| [crates/rum-runtime/src/runtime/suspension/core/abandon.rs](../../../crates/rum-runtime/src/runtime/suspension/core/abandon.rs) | Restore the per-cell diagnostic context through the existing structural cleanup worklist. |
| [crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs](../../../crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs) | Director-authorized independent guest-capacity projection and no-guest-storage marker arm. |
| [crates/rum-runtime/src/runtime/profiling/logical/mod.rs](../../../crates/rum-runtime/src/runtime/profiling/logical/mod.rs) | Native call handle, thin marker lease, owner-pinned frontier and isolated test hooks. |
| [crates/rum-runtime/src/runtime/profiling/logical/budget.rs](../../../crates/rum-runtime/src/runtime/profiling/logical/budget.rs) | Process/epoch diagnostic reservations, release accounting, initial-cap and TLS-lifetime tests. |
| [crates/rum-runtime/src/runtime/profiling/logical/record.rs](../../../crates/rum-runtime/src/runtime/profiling/logical/record.rs) | Immutable identities, keys, owners, policies, terminal classes and reason counts. |
| [crates/rum-runtime/src/runtime/profiling/logical/epoch.rs](../../../crates/rum-runtime/src/runtime/profiling/logical/epoch.rs) | Bounded open-record registry, aggregate rows, idempotent closes, invalidation and failed-allocation latch. |
| [crates/rum-runtime/src/runtime/profiling/logical/segments.rs](../../../crates/rum-runtime/src/runtime/profiling/logical/segments.rs) | Thread-bound executing intervals, strict identity/LIFO checks, retained TLS reservation and idle non-allocating lookup. |
| [crates/rum-runtime/src/runtime/profiling/logical/context.rs](../../../crates/rum-runtime/src/runtime/profiling/logical/context.rs) | Value-free ancestry/source leases, restoration, resumed segments and inherited pause telemetry. |
| [crates/rum-runtime/src/runtime/profiling/logical/queue.rs](../../../crates/rum-runtime/src/runtime/profiling/logical/queue.rs) | Diagnostic cell deque plus original semantic guest queue preserving marker-free capacity history. |
| [crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs](../../../crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs) | Explicit producer/consumer/return frontier insertion with owner/generation checks. |
| [crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs](../../../crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs) | Serial isolated enabled-state support and bounded test accessors. |
| [crates/rum-runtime/src/runtime/profiling/tests/record_engine.rs](../../../crates/rum-runtime/src/runtime/profiling/tests/record_engine.rs) | Idempotence, repeated-key identity, reset isolation, cap, strict marker peer and active-abandon interval proofs. |
| [crates/rum-runtime/src/runtime/profiling/tests/slot_boundary.rs](../../../crates/rum-runtime/src/runtime/profiling/tests/slot_boundary.rs) | Raw Slot identity/channel/policy and Entry/EntryRef size proofs without codecs. |
| [crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs](../../../crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs) | Real driver frontiers, six structural reasons, fault/resource-error and guest-capacity/cap proofs. |
| [crates/rum-host/tests/logical_profile_lifecycle.rs](../../../crates/rum-host/tests/logical_profile_lifecycle.rs) | Native spans cover terminal routes, repeated pause counts, thread movement, reset/restore/helper/rollback/cleanup/typed-collection lifecycles. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv) | Replay exact discovery: source coordinates and new touched-code UnknownConstructs; preserve old identities/statuses. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | Replay exact discovery: source coordinates and new touched-code UnknownConstructs; preserve old identities/statuses. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | Replay exact discovery: source coordinates and new touched-code UnknownConstructs; preserve old identities/statuses. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | Replay exact discovery: source coordinates and new touched-code UnknownConstructs; preserve old identities/statuses. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv) | Replay exact discovery: source coordinates and new touched-code UnknownConstructs; preserve old identities/statuses. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv) | Replay eight corresponding source-coordinate fields; preserve all disposition/witness/exclusion/coverage fields. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | Replay all68 generated fingerprint cells; preserve all other fields. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | Replay generated C01/C02 default/no-default certificate cells only. |
| [docs/dev/codex/brief-626-lp02-logical-profiler-record-engine.md](../../../docs/dev/codex/brief-626-lp02-logical-profiler-record-engine.md) | Append this execution report after final evidence; preserve all prior text. |


Ownership replay: 159 source-position moves, 133 new UnknownConstruct rows on touched code, and 8 ledger position moves. No old row disappears; identities, statuses, dispositions and exclusions remain. Unknowns remain unknown; zero new UnknownHostPayload rows. All 68 contract fingerprints were regenerated; C01/C02 certificate cells were replayed in both feature builds (four cells). Full replay provenance: [ownership-replay.json](../../../.temp/brief-626/ownership-replay.json), [ownership-position-preflight.json](../../../.temp/brief-626/ownership-position-preflight.json), [fingerprint-replay.json](../../../.temp/brief-626/fingerprint-replay.json), [certificate-replay.json](../../../.temp/brief-626/certificate-replay.json).

Every original-to-final inventory row change follows, mechanically generated from the saved original files. Unchanged rows are omitted; complete old/new rows are preserved. The same data is available as [inventory-row-changes.json](../../../.temp/brief-626/inventory-row-changes.json) and [inventory-row-changes.patch](../../../.temp/brief-626/inventory-row-changes.patch).

<details>
<summary>Every inventory row change (complete zero-context patch)</summary>


```diff
--- original/crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv
+++ candidate/crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv
@@ -52,6 +52,6 @@
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	124	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	126	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	136	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@129::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	147	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@129::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	520	57	public input rum_runtime::runtime::suspension::core::impl(Session)@464::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core/queue.rs	6	12	rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/rum_runtime::runtime::suspension::core::evaluation_context::Queued<arg0> via payload/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	132	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	134	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	144	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@137::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	155	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@137::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	547	57	public input rum_runtime::runtime::suspension::core::impl(Session)@482::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core/queue.rs	10	12	rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/rum_runtime::runtime::suspension::core::evaluation_context::Queued<arg0> via payload/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
@@ -67,2 +67,2 @@
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	10	12	rum_runtime::runtime::suspension::typed::entry::Entry.Legacy.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn SuspensionFrame
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	12	rum_runtime::runtime::suspension::typed::entry::Entry.Legacy.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn SuspensionFrame
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	12	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
--- original/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv
+++ candidate/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv
@@ -755,51 +755,63 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	108	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	114	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	124	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	130	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	136	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	143	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	149	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	156	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	166	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	172	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	178	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	185	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	193	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	202	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	208	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	219	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	229	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	241	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	250	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	257	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	269	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	277	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	284	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	296	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	304	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	312	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	319	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	325	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	333	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	339	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	345	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	351	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	359	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	366	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	374	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	381	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	395	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	402	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	411	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	417	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	431	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	442	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	452	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	458	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	474	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	483	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	510	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	520	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	530	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	556	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	559	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	12	1	unresolved attribute/expansion derive (Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	223	5	unresolved attribute/expansion cfg (any (test , feature = "test-builtin-memory-growth" , feature = "test-continuation-slots"))
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	66	1	unresolved attribute/expansion derive (Clone)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	84	1	unresolved attribute/expansion cfg (any (test , feature = "test-builtin-memory-growth" , feature = "test-continuation-slots"))
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	4	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	6	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	8	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	11	1	unresolved attribute/expansion derive (Clone)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	26	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	49	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	60	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	71	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/record.rs	110	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	110	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	116	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	126	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	132	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	138	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	145	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	151	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	158	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	168	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	174	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	180	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	187	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	195	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	204	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	210	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	221	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	231	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	243	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	252	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	259	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	271	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	279	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	286	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	298	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	306	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	314	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	321	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	327	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	335	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	341	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	347	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	353	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	361	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	368	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	376	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	383	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	397	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	404	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	413	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	419	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	433	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	444	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	454	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	460	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	476	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	485	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	512	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	522	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	532	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	558	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
@@ -806,0 +819 @@
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	563	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
@@ -930,4 +943,4 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	28	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , PartialOrd , Ord)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	46	5	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	52	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	238	13	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	30	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , PartialOrd , Ord)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	48	5	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	54	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	256	13	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
@@ -945 +958 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	8	1	unresolved attribute/expansion derive (Clone)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	9	1	unresolved attribute/expansion derive (Clone)
--- original/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv
+++ candidate/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv
@@ -552,15 +552,18 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	39	46	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	42	46	closure captures: |depth| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	54	42	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	57	42	closure captures: |depth| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	71	42	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	122	42	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	134	60	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	142	21	closure captures: |frame| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	143	18	closure captures: |frame| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	200	43	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	239	38	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	264	51	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	281	43	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	323	43	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	346	43	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	35	53	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	41	46	closure captures: |d| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	42	57	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	52	46	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	55	46	closure captures: |depth| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	67	42	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	70	42	closure captures: |depth| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	84	42	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	135	42	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	147	60	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	155	21	closure captures: |frame| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	156	18	closure captures: |frame| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	213	43	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	252	38	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	277	51	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	294	43	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	336	43	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	359	43	closure captures: |stats| (capture types/modes unresolved)
@@ -1313,0 +1317,61 @@
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/budget.rs	22	61	closure captures: |old| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/budget.rs	23	43	closure captures: |new| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	32	59	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	33	49	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	34	52	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	45	23	closure captures: |(e , r)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	45	50	closure captures: |(owner , _ , _)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	48	21	closure captures: |(e , r)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	50	51	closure captures: |owner| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	52	42	closure captures: |(candidate , _ , _)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	55	18	closure captures: |(_ , r)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	89	30	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	91	51	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	93	54	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	94	46	closure captures: |d| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	95	36	closure captures: |d| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	96	39	closure captures: |d| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	123	46	closure captures: |d| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	124	36	closure captures: |d| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	125	39	closure captures: |d| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	126	57	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	127	47	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	128	50	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	137	65	closure captures: |n| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	22	25	closure captures: || (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	93	22	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	103	23	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	149	56	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	164	19	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	165	18	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	170	53	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	170	73	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	180	53	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	180	73	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	234	18	closure captures: |r| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	28	26	closure captures: |(owner , _ , _)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	50	45	closure captures: |(_ , _ , p)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	126	18	closure captures: |e| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	129	26	closure captures: |(key , c)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	154	18	closure captures: |e| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	179	18	closure captures: |e| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	179	42	closure captures: |(_ , c)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	42	24	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	46	18	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	56	24	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	57	38	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	59	55	closure captures: || (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	73	24	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	73	56	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	81	28	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	101	43	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	104	31	closure captures: |state| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	131	27	closure captures: |state| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	150	32	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	151	60	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	160	35	closure captures: |state| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	164	31	closure captures: |state| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	171	34	closure captures: |(_ , id , _)| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	194	36	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	199	19	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	200	18	closure captures: |s| (capture types/modes unresolved)
@@ -1459,7 +1523,5 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	64	31	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	65	33	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	66	39	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	67	34	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	68	33	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	69	38	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	70	32	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	66	31	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	67	33	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	68	39	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	69	34	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	70	33	closure captures: |stats| (capture types/modes unresolved)
@@ -1467,24 +1529,26 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	72	45	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	73	44	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	74	31	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	75	37	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	76	43	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	77	35	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	78	41	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	79	38	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	80	32	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	81	39	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	82	36	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	83	31	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	84	31	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	85	36	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	86	36	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	87	46	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	88	41	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	89	36	closure captures: |stats| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	90	32	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	91	32	closure captures: |depth| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	92	35	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	93	35	closure captures: |depth| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	94	42	closure captures: |stack| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	95	42	closure captures: |depth| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	72	32	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	73	38	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	74	45	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	75	44	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	76	31	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	77	37	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	78	43	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	79	35	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	80	41	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	81	38	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	82	32	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	83	39	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	84	36	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	85	31	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	86	31	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	87	36	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	88	36	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	89	46	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	90	41	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	91	36	closure captures: |stats| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	92	32	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	93	32	closure captures: |depth| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	94	35	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	95	35	closure captures: |depth| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	96	42	closure captures: |stack| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	97	42	closure captures: |depth| (capture types/modes unresolved)
@@ -1789,11 +1853,15 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	215	27	closure captures: |frame| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	220	27	closure captures: |frame| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	261	33	closure captures: |owner| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	708	22	closure captures: |cell| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	711	30	closure captures: |failure| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	742	48	closure captures: |value| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	742	73	closure captures: |()| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	776	43	closure captures: |()| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	787	48	closure captures: |value| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	787	73	closure captures: |()| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	825	43	closure captures: |()| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	217	62	closure captures: |c| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	218	31	closure captures: |cell| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	233	27	closure captures: |frame| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	238	27	closure captures: |frame| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	279	33	closure captures: |owner| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	748	31	closure captures: |s| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	749	31	closure captures: |context| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	757	22	closure captures: |cell| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	760	30	closure captures: |failure| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	798	48	closure captures: |value| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	798	73	closure captures: |()| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	832	43	closure captures: |()| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	843	48	closure captures: |value| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	843	73	closure captures: |()| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	881	43	closure captures: |()| (capture types/modes unresolved)
@@ -1801,5 +1869,10 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	21	47	closure captures: |cell| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	82	56	closure captures: |cell| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	86	55	closure captures: |cell| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	96	60	closure captures: |cell| (capture types/modes unresolved)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	207	68	closure captures: |cell| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	44	53	closure captures: |c| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	54	23	closure captures: |c| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	55	22	closure captures: |c| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	61	59	closure captures: |c| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	89	47	closure captures: |cell| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	215	56	closure captures: |cell| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	219	55	closure captures: |cell| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	223	54	closure captures: |c| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	233	60	closure captures: |cell| (capture types/modes unresolved)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	362	68	closure captures: |cell| (capture types/modes unresolved)
--- original/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv
+++ candidate/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv
@@ -1019,2 +1019,2 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/model/request_cancellation.rs	190	9	unexpanded macro matches! (ends at 193:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/model/request_cancellation.rs	219	5	unexpanded macro debug_assert! (ends at 219:62)
+UnknownConstruct	crates/rum-runtime/src/runtime/model/request_cancellation.rs	196	9	unexpanded macro matches! (ends at 199:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/model/request_cancellation.rs	225	5	unexpanded macro debug_assert! (ends at 225:62)
@@ -1502,0 +1503,8 @@
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/budget.rs	58	9	unexpanded macro debug_assert_eq! (ends at 58:66)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	153	9	unexpanded macro debug_assert_eq! (ends at 153:64)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	195	9	unexpanded macro eprintln! (ends at 198:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	252	13	unexpanded macro eprintln! (ends at 255:14)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	281	9	unexpanded macro eprintln! (ends at 287:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	289	9	unexpanded macro eprintln! (ends at 292:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	131	29	unexpanded macro format! (ends at 131:47)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	21	1	unexpanded macro thread_local! (ends at 28:2)
@@ -1537,2 +1545,2 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	8	5	unexpanded macro macro_rules! (ends at 12:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	14	5	unexpanded macro reset! (ends at 62:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	10	5	unexpanded macro macro_rules! (ends at 14:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/reset.rs	16	5	unexpanded macro reset! (ends at 64:6)
@@ -1540,13 +1548,13 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	27	9	unexpanded macro eprintln! (ends at 27:64)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	40	5	unexpanded macro eprintln! (ends at 48:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	52	5	unexpanded macro eprintln! (ends at 57:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	58	5	unexpanded macro eprintln! (ends at 63:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	64	5	unexpanded macro eprintln! (ends at 68:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	69	5	unexpanded macro eprintln! (ends at 77:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	78	5	unexpanded macro eprintln! (ends at 86:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	87	5	unexpanded macro eprintln! (ends at 96:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	97	5	unexpanded macro eprintln! (ends at 106:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	107	5	unexpanded macro eprintln! (ends at 114:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	115	5	unexpanded macro eprintln! (ends at 125:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	126	5	unexpanded macro eprintln! (ends at 138:6)
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	139	5	unexpanded macro eprintln! (ends at 148:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	30	9	unexpanded macro eprintln! (ends at 30:64)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	43	5	unexpanded macro eprintln! (ends at 51:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	56	5	unexpanded macro eprintln! (ends at 61:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	62	5	unexpanded macro eprintln! (ends at 67:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	68	5	unexpanded macro eprintln! (ends at 72:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	73	5	unexpanded macro eprintln! (ends at 81:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	82	5	unexpanded macro eprintln! (ends at 90:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	91	5	unexpanded macro eprintln! (ends at 100:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	101	5	unexpanded macro eprintln! (ends at 110:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	111	5	unexpanded macro eprintln! (ends at 118:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	119	5	unexpanded macro eprintln! (ends at 129:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	130	5	unexpanded macro eprintln! (ends at 142:6)
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/summary.rs	143	5	unexpanded macro eprintln! (ends at 152:6)
@@ -1796 +1803,0 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	265	9	unexpanded macro assert! (ends at 268:10)
@@ -1798,7 +1805,9 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	584	9	unexpanded macro debug_assert! (ends at 584:57)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	592	9	unexpanded macro assert! (ends at 595:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	617	9	unexpanded macro debug_assert! (ends at 620:10)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	682	17	unexpanded macro debug_assert! (ends at 682:66)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	806	21	unexpanded macro debug_assert! (ends at 806:76)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	810	29	unexpanded macro unreachable! (ends at 810:79)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	869	1	unexpanded macro macro_rules! (ends at 886:2)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	301	9	unexpanded macro assert! (ends at 304:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	611	9	unexpanded macro debug_assert! (ends at 611:57)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	619	9	unexpanded macro assert! (ends at 622:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	644	9	unexpanded macro debug_assert! (ends at 647:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	709	17	unexpanded macro debug_assert! (ends at 709:66)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	730	12	unexpanded macro matches! (ends at 733:10)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	862	21	unexpanded macro debug_assert! (ends at 862:76)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	866	29	unexpanded macro unreachable! (ends at 866:79)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	930	1	unexpanded macro macro_rules! (ends at 947:2)
@@ -1808,2 +1817,15 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	25	13	unexpanded macro unreachable! (ends at 25:27)
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	124	17	unexpanded macro unreachable! (ends at 124:76)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	17	12	unexpanded macro matches! (ends at 17:45)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	27	13	unexpanded macro matches! (ends at 27:46)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	31	13	unexpanded macro unreachable! (ends at 31:27)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	54	27	unexpanded macro matches! (ends at 54:87)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	93	13	unexpanded macro unreachable! (ends at 93:27)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	251	22	unexpanded macro unreachable! (ends at 251:81)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	266	17	unexpanded macro unreachable! (ends at 266:76)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	346	12	unexpanded macro matches! (ends at 346:45)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	346	49	unexpanded macro matches! (ends at 346:84)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	364	34	unexpanded macro unreachable! (ends at 364:87)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	371	12	unexpanded macro matches! (ends at 371:68)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	374	12	unexpanded macro matches! (ends at 374:70)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	377	13	unexpanded macro matches! (ends at 377:46)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	377	51	unexpanded macro matches! (ends at 377:86)
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	383	18	unexpanded macro unreachable! (ends at 383:32)
--- original/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv
+++ candidate/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv
@@ -60,2 +60,19 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	465	12	rum_runtime::runtime::profiling::BuiltinTimingToken.start: foreign representation std::time::Instant: Instant
-UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	470	12	rum_runtime::runtime::profiling::RuntimePathTimingToken.start: foreign representation std::time::Instant: Instant
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/budget.rs	11	12	rum_runtime::runtime::profiling::logical::budget::Budget.bytes: foreign representation std::sync::atomic::AtomicUsize: AtomicUsize
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/context.rs	12	16	rum_runtime::runtime::profiling::logical::context::Context.executing: foreign representation std::sync::atomic::AtomicBool: AtomicBool
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	27	23	rum_runtime::runtime::profiling::logical::epoch::Epoch.valid: foreign representation std::sync::atomic::AtomicBool: AtomicBool
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	28	26	rum_runtime::runtime::profiling::logical::epoch::Epoch.contexts: foreign representation std::sync::atomic::AtomicUsize: AtomicUsize
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/epoch.rs	29	24	rum_runtime::runtime::profiling::logical::epoch::Epoch.faults: foreign representation std::sync::atomic::AtomicU64: AtomicU64
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	92	9	conditional public input signature logical_profile_test_span
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	124	9	conditional public input signature logical_profile_test_rows
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	152	9	conditional public input signature logical_profile_test_leases
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	159	9	conditional public input signature logical_profile_test_reset
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	164	9	conditional public input signature logical_profile_test_reasons
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	177	9	conditional public input signature logical_profile_test_pauses
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	245	1	module tests has 0 source candidates
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/mod.rs	249	1	module test_support has 0 source candidates
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/queue.rs	1	1	source file is not reachable from a production or test module declaration
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs	1	1	source file is not reachable from a production or test module declaration
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/logical/segments.rs	12	12	rum_runtime::runtime::profiling::logical::segments::Active.start: foreign representation std::time::Instant: Instant
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	467	12	rum_runtime::runtime::profiling::BuiltinTimingToken.start: foreign representation std::time::Instant: Instant
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	472	12	rum_runtime::runtime::profiling::RuntimePathTimingToken.start: foreign representation std::time::Instant: Instant
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs	1	1	source file is not reachable from a production or test module declaration
@@ -66,0 +84,3 @@
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs	1	1	source file is not reachable from a production or test module declaration
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/record_engine.rs	1	1	source file is not reachable from a production or test module declaration
+UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/slot_boundary.rs	1	1	source file is not reachable from a production or test module declaration
@@ -80 +100,3 @@
-UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	47	18	rum_runtime::runtime::suspension::core::SuspensionSlot.fault_probe: unresolved conditional storage: fault :: FaultProbe
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	18	1	module logical_queue_api has 0 source candidates
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	49	18	rum_runtime::runtime::suspension::core::SuspensionSlot.fault_probe: unresolved conditional storage: fault :: FaultProbe
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core.rs	955	1	module logical_queue_tests has 0 source candidates
@@ -82,0 +105,3 @@
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	5	1	module profile has 0 source candidates
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/core/queue.rs	12	14	rum_runtime::runtime::suspension::core::queue::FrameQueue.Profiled.0/std::boxed::Box[Own]/arg0: unresolved/ambiguous type (): ProfileQueue
+UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	140	1	module logical_tests has 0 source candidates
--- original/crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv
+++ candidate/crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv
@@ -53,6 +53,6 @@
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core/queue.rs	6	12	rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/rum_runtime::runtime::suspension::core::evaluation_context::Queued<arg0> via payload/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	124	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	126	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer	Unbounded	rum_runtime::runtime::suspension::core::PureFrameFinalizer => rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer (impl at crates/rum-runtime/src/runtime/debug_tools/reap.rs:284:1); rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer.rendered -> rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator ; rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator.prefix -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	136	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@129::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	147	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@129::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	520	57	public input rum_runtime::runtime::suspension::core::impl(Session)@464::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core/queue.rs	10	12	rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/rum_runtime::runtime::suspension::core::evaluation_context::Queued<arg0> via payload/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	132	21	rum_runtime::runtime::suspension::core::FrameAction.Evaluate.next_frame/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	134	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer	Unbounded	rum_runtime::runtime::suspension::core::PureFrameFinalizer => rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer (impl at crates/rum-runtime/src/runtime/debug_tools/reap.rs:284:1); rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer.rendered -> rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator ; rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator.prefix -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	144	48	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@137::evaluate(next_frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	155	35	public input rum_runtime::runtime::suspension::core::impl(FrameAction)@137::finalize(finalizer): opaque owned generic parameter F where F : PureFrameFinalizer ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	547	57	public input rum_runtime::runtime::suspension::core::impl(Session)@482::insert_suspension_frame(frame): opaque owned generic parameter F where F : SuspensionFrame ,	PublicInput	public by-value input; owning storage sites are separate rows	-	-
@@ -77,2 +77,2 @@
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	10	12	rum_runtime::runtime::suspension::typed::entry::Entry.Legacy.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
-UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	Unbounded	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:33:1) => rum_runtime::runtime::suspension::typed::TypedFrame => rum_runtime::runtime::definitions::application::default_values::EvaluateFirstExactDefaultValueFromCursorSuspensionFrame (impl at crates/rum-runtime/src/runtime/definitions/application/default_values.rs:292:1); rum_runtime::runtime::definitions::application::default_values::EvaluateFirstExactDefaultValueFromCursorSuspensionFrame.state -> rum_runtime::runtime::definitions::application::default_values::SuspendedExactDefaultValueScan ; rum_runtime::runtime::definitions::application::default_values::SuspendedExactDefaultValueScan.lookup_order -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	11	12	rum_runtime::runtime::suspension::typed::entry::Entry.Legacy.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame (impl at crates/rum-runtime/src/runtime/accounting_mode/continuation.rs:12:1); rum_runtime::runtime::accounting_mode::continuation::AccountedCompoundFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
+UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/entry.rs	12	11	rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame	Unbounded	rum_runtime::runtime::suspension::typed::adapter::ErasedFrame => rum_runtime::runtime::suspension::typed::adapter::TypedAdapter.inner (impl at crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:33:1) => rum_runtime::runtime::suspension::typed::TypedFrame => rum_runtime::runtime::definitions::application::default_values::EvaluateFirstExactDefaultValueFromCursorSuspensionFrame (impl at crates/rum-runtime/src/runtime/definitions/application/default_values.rs:292:1); rum_runtime::runtime::definitions::application::default_values::EvaluateFirstExactDefaultValueFromCursorSuspensionFrame.state -> rum_runtime::runtime::definitions::application::default_values::SuspendedExactDefaultValueScan ; rum_runtime::runtime::definitions::application::default_values::SuspendedExactDefaultValueScan.lookup_order -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
--- original/crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv
+++ candidate/crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv
@@ -2,68 +2,68 @@
-B01	rum_runtime::runtime::model::session::Session::option_rule_exprs_for_symbol	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	charge snapshot capacity and cloned occurrences including overlap	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:6bf3d9d0b0fec720
-B02	rum_runtime::runtime::model::session::Session::resume_merged_option_exprs	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	charge prefix and defaults across callbacks and suspension	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:c7879cb55b54928d
-B03	rum_runtime::runtime::options::reentry::OptionReentry::resume	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	kernel checked entry/resume only; registered external behavior excluded	kernel guard has approved TypeId; direct external resume calls excluded	kernel option entry and resume refuse unregistered concrete TypeId	kernel option entry and resume refuse unregistered concrete TypeId	existing occurrence metric; capacity not length; checked overflow; no RSS bound	B12: root transfer plus spare-capacity complement; B09: no output receipt; B03: membership only	normal/error/suspend/resume/abandon/unwind per named runtime controls; abort has no return	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	cargo test --workspace; named runtime falsifier and local stale/Pending controls	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-runtime/src/runtime/options/tests/resume.rs#option_reentry_registration_does_not_mint_certification	remove check_option_reentry refusal; negative becomes List[17] instead of $Failed	Discharged	fnv1a64:879f8ba0c053ee8a
-B04	<rum_core::expr::Expr as std::clone::Clone>::clone	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	fieldwise Expr clone produces an unleased occurrence	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	borrows source; new occurrence must transfer before a hazardous call	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:4569116471c071a8
-B05	std::vec::Vec<rum_core::expr::Expr>::len	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	Vec length reads initialized length even with spare capacity	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrow only; no owner or capacity transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:b31fb25601e0ee59
-B06	std::option::Option<&rum_core::expr::Symbol>::map<bool,literal>	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	None calls zero times; Some once; exact borrowed capture; nonescape pending	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:1ea593e06a096184
-B07	std::option::Option<bool>::unwrap_or	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	complete scalar None/Some false/Some true table pending	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:4256320e30a58d89
-B08	rum_core::expr::Symbol::is_system_name	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	borrowed context/name predicate; allocation observations pending	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:781ef08794f5cef6
-B09	rum_runtime::runtime::model::session::Session::evaluate	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	actual armed resource checkpoints only; no between-checkpoint peak bound	caller prefix already exposed by typed bridge; bare evaluate creates no receipt	armed checkpoints check maintained S plus R and visible expression W	armed checkpoints check maintained S plus R and visible expression W	existing occurrence metric; capacity not length; checked overflow; no RSS bound	B12: root transfer plus spare-capacity complement; B09: no output receipt; B03: membership only	normal/error/suspend/resume/abandon/unwind per named runtime controls; abort has no return	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	cargo test --workspace; named runtime falsifier and local stale/Pending controls	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs#retention_r_is_separate_from_s_and_both_admission_entries_see_it	remove retention contribution from expression admission; limit-crossing control must fail	Discharged	fnv1a64:28cbb80f1574160c
-B10	rum_core::expr::Expr::call	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_core::expr::Expr::call; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	only rule::finish_rule; adopted RHS; charged cloned name; exact Rule head; shell and actual child Vec capacity checked; unchanged indexed adapter	may-run-guest-or-grow	consumes charged head and two children; outgoing RetainedResult owns receipt until scope return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:11efcb0506f699af
-B11	rum_runtime::runtime::expr_helpers::system_expr	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::expr_helpers::system_expr; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	literal Rule only in unchanged rule::finish_rule; no arbitrary name or interner bound	produces-owned;inert	System Rule identity; OwnedUnleased until head receipt resized; cold/warm observations	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:06c0c2c18bb497a8
-B12	rum_runtime::runtime::model::session::Session::insert_suspension_frame	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	private lease-bearing RetentionFrame protocol only; legacy raw frames excluded	initialized InputRequired; typed suspend reserves frame and owns frozen prefix	lease-bearing RetentionFrame transfers once; roots plus frozen complement charged	lease-bearing RetentionFrame transfers once; roots plus frozen complement charged	existing occurrence metric; capacity not length; checked overflow; no RSS bound	B12: root transfer plus spare-capacity complement; B09: no output receipt; B03: membership only	normal/error/suspend/resume/abandon/unwind per named runtime controls; abort has no return	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	cargo test --workspace; named runtime falsifier and local stale/Pending controls	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs#retention_suspension_transfers_once_resumes_twice_and_abandons	omit transfer; transfer-count assertion fails; resume and abandon are controls	Discharged	fnv1a64:e7d8380af17cab0c
-I01	<rum_core::expr::SharedArgs as std::ops::Deref>::deref	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	SharedArgs deref borrows the same Vec with pointer identity	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrow only; no owner transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:49d5c4ce1db6bda0
-I02	<rum_core::expr::SharedExpr as std::ops::Deref>::deref	concrete option-read instance only	fn deref(&SharedExpr)->&Expr; exact Target=Expr and preserved input lifetime	exact pinned helper or SharedExpr Deref; no general dispatch	shared immutable input; exact declaration and compiler selection checked	borrows;inert	borrows same Expr with pointer identity; no owner or capacity receipt	no owned output; no quantitative clone or capacity claim	input remains borrowed; no field move or ownership transfer	no guest callbacks; borrowed locals need no owned cleanup on return or unwind	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_option_read_contract_controls; local_option_read_claims_match_stable_rustc; invariant types and same-name negatives	contracts.rs::DEPENDENCIES plus all crate Rust/Python/TOML; nineteen-field record; direct index-time declaration checks	crates/rum-stdlib/src/tests/retention_closure/local/reads/tests.rs#local_option_read_contract_controls	changed concrete declaration or stale/Pending mark rejects; current borrowed read passes; helper truth table and pointer controls	Discharged	fnv1a64:0a612fe9fa177659
-I03	<std::vec::Vec<rum_core::expr::Expr> as std::ops::Deref>::deref	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	borrow initialized Vec elements as slice; do not clone capacity	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:f6b70701ac95c64c
-I04	<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	Vec<Expr> indexing by usize borrows one element; bounds panic unwinds	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrow only; no clone or owner transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:3f538ca5540412ad
-P01	<rum_core::expr::Expr as std::cmp::PartialEq>::eq	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	Expr equality borrows both operands; no guest or retained growth	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrows only; no owner transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:6e4d07ded28dc824
-P02	rum_core::expr::Expr::small_integer	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	small_integer constructs an inline Expr from exactly i64	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	inline result; no retention receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:250c610e407a793f
-P03	rum_core::expr::Expr::string::<std::string::String>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	String input moves into Expr; raw payload state is preserved	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	String owner moves into Expr; no retention receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:6a70e9c10d3f8842
-P04	str::repeat	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	str repeat produces a fresh raw String from usize count	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	borrows str; caller owns returned String; no retention receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:bc62bacadffdae8d
-P05	<rum_core::expr::SharedArgs as std::clone::Clone>::clone	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	SharedArgs clone shares the same allocation with pointer identity	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	new shared handle; no deep copy or retention receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:b170cd19be5a4530
-P06	std::option::Option<usize>::is_some	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	Option<usize> is_some reads discriminant without callbacks	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrow; false for None and true for Some	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:38575cdb7dac448c
-P070	rum_runtime::runtime::accounting_mode::retained::RetentionContext::reserve::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	reserve accounts vector capacity in the selected mode	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	mutable values borrow; owner remains charged	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:72a4f91d5c21be8c
-P080	rum_runtime::runtime::accounting_mode::retained::RetentionContext::push::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	push consumes input and charges occurrence plus capacity before guest reentry	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves Expr into charged values before hazardous work	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:68a7cf7d69fc01ef
-P090	rum_runtime::runtime::accounting_mode::retained::RetentionContext::evaluate::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	evaluate exposes prefix before guest and returns typed callback bridge	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves prefix and guest input into bridge; guest result has no implicit receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:e85b7dcd6bb8f7cd
-P100	rum_runtime::runtime::accounting_mode::retained::RetentionContext::finish_list::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	finish_list transfers charged prefix into typed result	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes prefix and context; cleanup releases on failure	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:494735ed9cf26058
-P110	rum_runtime::runtime::accounting_mode::retained::RetentionContext::resume_values::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	resume_values restores frozen prefix plus capacity into mode-specific values	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes empty values; borrows frozen prefix; avoids duplicate transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:dbe86b1e897c77c7
-P120	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::adopt::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	adopt reconstitutes typed context and charged values from successful bridge	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes CallbackValue once; failures preserve typed ownership	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:502cfb4a0939fe16
-P130	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend::<rum_runtime::runtime::accounting_mode::tests::ProbeContinuation,literal>::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	suspend invokes immediate literal once to construct exact ProbeContinuation	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves failure into frozen prefix/frame; frame roots plus spare complement charged	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:e6238b6b5f596789
-P140	rum_runtime::runtime::accounting_mode::retained::scoped::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	scoped calls immediate literal once with branded context and values	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	borrowed Session; result consumed at exit; no brand or closure escape	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:5fbd9b7762405805
-P150	std::result::Result::and_then::<CallbackValue::adopt>::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	and_then calls exact adopt item once on Ok and never on Err	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	Result consumes receiver; both branches preserve exact failure mode	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:fe560e4c6c6ea5ed
-P071	rum_runtime::runtime::accounting_mode::retained::RetentionContext::reserve::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	reserve accounts vector capacity in the selected mode	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	mutable values borrow; owner remains charged	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:15fa81905d50ef29
-P081	rum_runtime::runtime::accounting_mode::retained::RetentionContext::push::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	push consumes input and charges occurrence plus capacity before guest reentry	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves Expr into charged values before hazardous work	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:a3221c8e8c86432c
-P091	rum_runtime::runtime::accounting_mode::retained::RetentionContext::evaluate::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	evaluate exposes prefix before guest and returns typed callback bridge	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves prefix and guest input into bridge; guest result has no implicit receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:5a4f2b567955b564
-P101	rum_runtime::runtime::accounting_mode::retained::RetentionContext::finish_list::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	finish_list transfers charged prefix into typed result	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes prefix and context; cleanup releases on failure	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:d3eb5bebbee014c7
-P111	rum_runtime::runtime::accounting_mode::retained::RetentionContext::resume_values::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	resume_values restores frozen prefix plus capacity into mode-specific values	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes empty values; borrows frozen prefix; avoids duplicate transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:e53fba1a2aae67fc
-P121	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::adopt::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	adopt reconstitutes typed context and charged values from successful bridge	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes CallbackValue once; failures preserve typed ownership	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:a8cc8342fd968873
-P131	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend::<rum_runtime::runtime::accounting_mode::tests::ProbeContinuation,literal>::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	suspend invokes immediate literal once to construct exact ProbeContinuation	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves failure into frozen prefix/frame; frame roots plus spare complement charged	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:e31333caa8149328
-P141	rum_runtime::runtime::accounting_mode::retained::scoped::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	scoped calls immediate literal once with branded context and values	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	borrowed Session; result consumed at exit; no brand or closure escape	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:75d86945dbe32fc4
-P151	std::result::Result::and_then::<CallbackValue::adopt>::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	and_then calls exact adopt item once on Ok and never on Err	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	Result consumes receiver; both branches preserve exact failure mode	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:b84b2d5b1b7e3cd6
-B13	rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name	concrete option-read instance only	fn is_system_symbol_name(&Expr,&str)->bool	exact pinned helper or SharedExpr Deref; no general dispatch	shared immutable input; exact declaration and compiler selection checked	borrows;inert	None=false; System matching name=true; different name or context=false; scalar result	no owned output; no quantitative clone or capacity claim	input remains borrowed; no field move or ownership transfer	no guest callbacks; borrowed locals need no owned cleanup on return or unwind	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_option_read_contract_controls; local_option_read_claims_match_stable_rustc; invariant types and same-name negatives	contracts.rs::DEPENDENCIES plus all crate Rust/Python/TOML; nineteen-field record; direct index-time declaration checks	crates/rum-stdlib/src/tests/retention_closure/local/reads/tests.rs#local_option_read_contract_controls	changed concrete declaration or stale/Pending mark rejects; current borrowed read passes; helper truth table and pointer controls	Discharged	fnv1a64:acb627699aa61d84
-R01	rum_runtime::runtime::accounting_mode::rule::clone_result	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_result; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records	may-run-guest-or-grow	clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:7d628c79b2badc7d
-R02	rum_runtime::runtime::accounting_mode::rule::evaluate_rule	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::evaluate_rule; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records	may-run-guest-or-grow	clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:764f85a23e4ee882
-R03	rum_runtime::runtime::accounting_mode::rule::resume_rule	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::resume_rule; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records	may-run-guest-or-grow	clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:b02f850bbd8df891
-R040	rum_runtime::runtime::accounting_mode::rule::clone_owned::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_owned::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:4c70aa19ba92771d
-R050	rum_runtime::runtime::accounting_mode::rule::clone_child::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_child::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:7b2ccf1148330a10
-R060	rum_runtime::runtime::accounting_mode::rule::finish_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::finish_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:4d9e30cbb51d3135
-R070	rum_runtime::runtime::accounting_mode::rule::evaluate_in::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::evaluate_in::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:0b8fe4a86e28022c
-R080	rum_runtime::runtime::accounting_mode::rule::resume_in::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::resume_in::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:0657f5982388dbad
-R090	rum_runtime::runtime::accounting_mode::retained::RetentionContext::adopt_outcome::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::retained::RetentionContext::adopt_outcome::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:34af6b93bd529f98
-R100	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::into_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::into_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:4cc52ff36a29990a
-R110	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:963f60a25148f577
-R041	rum_runtime::runtime::accounting_mode::rule::clone_owned::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_owned::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:92ef8a93b1cddc84
-R051	rum_runtime::runtime::accounting_mode::rule::clone_child::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_child::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:767dabe2f8e7044d
-R061	rum_runtime::runtime::accounting_mode::rule::finish_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::finish_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:6b84e8df7a10ac86
-R071	rum_runtime::runtime::accounting_mode::rule::evaluate_in::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::evaluate_in::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:7058ff9c4ae24b7b
-R081	rum_runtime::runtime::accounting_mode::rule::resume_in::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::resume_in::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:05319f42389e3558
-R091	rum_runtime::runtime::accounting_mode::retained::RetentionContext::adopt_outcome::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::retained::RetentionContext::adopt_outcome::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:3f2e90b3a70ba4c5
-R101	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::into_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::into_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:8a3a064690344c75
-R111	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:e197e18471745a7e
-R12	rum_runtime::runtime::accounting_mode::frozen::FrozenValues::rule_name	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::frozen::FrozenValues::rule_name; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	one-name FrozenValues from private rule suspension; immutable borrow	may-run-guest-or-grow	borrow frozen name without consuming shared frame or cancelling complement	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:324b4c7529f71b3e
-R13	rum_runtime::runtime::options::resume_frames::RuntimeOptionSuspensionFrame::resume	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::options::resume_frames::RuntimeOptionSuspensionFrame::resume; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	exact rootless descriptor inside private RetentionFrame; frozen name alive; raw outcome admitted by resume_rule before growth	may-run-guest-or-grow	shared frame remains owner; result independently adopted and charged; Err payload forwarded; no second frame transfer	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:cae01fffc641a99a
-R14	rum_runtime::runtime::accounting_mode::rule::finish_armed	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::finish_armed; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	Unarmed callback left current limit armed; empty prefix; terminal incoming Expr adopted before growth	may-run-guest-or-grow	nested AccountingShadow scope charges callback and rule; outgoing W forwarded directly	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:639f446379a32058
-R15	rum_runtime::runtime::accounting_mode::rule::suspend_armed	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::suspend_armed; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	Unarmed callback left actual pending InputRequired and a new active limit	may-run-guest-or-grow	reselect AccountingShadow before name clone or vector growth; freeze once; terminal error preserves request	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:a1281c8a27f632d4
-O010	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	exact mode and descriptor arguments in identity; hash={false,true} builds reconciled separately	exact private function signature and reborrows checked by local_option_contract_controls	repository declaration build only; host-minted declarations and outside types excluded	entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor	may-run-guest-or-grow	B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged	existing occurrence metric, capacity not length, shared control bytes, checked overflow; unarmed receipts are unit	one original owner across guest calls; frozen S roots and R complement; separately charged working copy; copied consumer outputs outside scope	success/error/unwind, actual repeated input and abandon; nested guest late arm retains original; no returning guarantee for process abort	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	runtime option_owner and frozen option controls; outside public API test; seven compiled charge/domain omission controls; local_option_contract_controls	flat option_boundaries FILES; reconciled declaration/installer/impl/runtime/feature keys; B03 B04 B09 B12 and R records; no traversal	crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs#local_option_contract_controls	defaults/capacity/adoption/complement/copy/handoff/membership omissions red then restored green; stale Pending body schema raw and outside-R reject	Discharged	fnv1a64:7f38cab49c7c27e6
-O0200	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	exact mode and descriptor arguments in identity; hash={false,true} builds reconciled separately	exact private function signature and reborrows checked by local_option_contract_controls	repository declaration build only; host-minted declarations and outside types excluded	entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor	may-run-guest-or-grow	B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence	existing occurrence metric, capacity not length, shared control bytes, checked overflow; unarmed receipts are unit	one original owner across guest calls; frozen S roots and R complement; separately charged working copy; copied consumer outputs outside scope	success/error/unwind, actual repeated input and abandon; nested guest late arm retains original; no returning guarantee for process abort	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	runtime option_owner and frozen option controls; outside public API test; seven compiled charge/domain omission controls; local_option_contract_controls	flat option_boundaries FILES; reconciled declaration/installer/impl/runtime/feature keys; B03 B04 B09 B12 and R records; no traversal	crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs#local_option_contract_controls	defaults/capacity/adoption/complement/copy/handoff/membership omissions red then restored green; stale Pending body schema raw and outside-R reject	Discharged	fnv1a64:5a853d4bb0f77e05
-O011	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	exact mode and descriptor arguments in identity; hash={false,true} builds reconciled separately	exact private function signature and reborrows checked by local_option_contract_controls	repository declaration build only; host-minted declarations and outside types excluded	entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor	may-run-guest-or-grow	B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged	existing occurrence metric, capacity not length, shared control bytes, checked overflow; unarmed receipts are unit	one original owner across guest calls; frozen S roots and R complement; separately charged working copy; copied consumer outputs outside scope	success/error/unwind, actual repeated input and abandon; nested guest late arm retains original; no returning guarantee for process abort	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	runtime option_owner and frozen option controls; outside public API test; seven compiled charge/domain omission controls; local_option_contract_controls	flat option_boundaries FILES; reconciled declaration/installer/impl/runtime/feature keys; B03 B04 B09 B12 and R records; no traversal	crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs#local_option_contract_controls	defaults/capacity/adoption/complement/copy/handoff/membership omissions red then restored green; stale Pending body schema raw and outside-R reject	Discharged	fnv1a64:9026ed53c2aef5b5
-O0210	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	exact mode and descriptor arguments in identity; hash={false,true} builds reconciled separately	exact private function signature and reborrows checked by local_option_contract_controls	repository declaration build only; host-minted declarations and outside types excluded	entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor	may-run-guest-or-grow	B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence	existing occurrence metric, capacity not length, shared control bytes, checked overflow; unarmed receipts are unit	one original owner across guest calls; frozen S roots and R complement; separately charged working copy; copied consumer outputs outside scope	success/error/unwind, actual repeated input and abandon; nested guest late arm retains original; no returning guarantee for process abort	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	runtime option_owner and frozen option controls; outside public API test; seven compiled charge/domain omission controls; local_option_contract_controls	flat option_boundaries FILES; reconciled declaration/installer/impl/runtime/feature keys; B03 B04 B09 B12 and R records; no traversal	crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs#local_option_contract_controls	defaults/capacity/adoption/complement/copy/handoff/membership omissions red then restored green; stale Pending body schema raw and outside-R reject	Discharged	fnv1a64:c9b2bdf935db849e
+B01	rum_runtime::runtime::model::session::Session::option_rule_exprs_for_symbol	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	charge snapshot capacity and cloned occurrences including overlap	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:ce0f87a85c98e43e
+B02	rum_runtime::runtime::model::session::Session::resume_merged_option_exprs	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	charge prefix and defaults across callbacks and suspension	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:b285e40ee549093b
+B03	rum_runtime::runtime::options::reentry::OptionReentry::resume	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	kernel checked entry/resume only; registered external behavior excluded	kernel guard has approved TypeId; direct external resume calls excluded	kernel option entry and resume refuse unregistered concrete TypeId	kernel option entry and resume refuse unregistered concrete TypeId	existing occurrence metric; capacity not length; checked overflow; no RSS bound	B12: root transfer plus spare-capacity complement; B09: no output receipt; B03: membership only	normal/error/suspend/resume/abandon/unwind per named runtime controls; abort has no return	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	cargo test --workspace; named runtime falsifier and local stale/Pending controls	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-runtime/src/runtime/options/tests/resume.rs#option_reentry_registration_does_not_mint_certification	remove check_option_reentry refusal; negative becomes List[17] instead of $Failed	Discharged	fnv1a64:4fd1addc9b5fb17c
+B04	<rum_core::expr::Expr as std::clone::Clone>::clone	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	fieldwise Expr clone produces an unleased occurrence	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	borrows source; new occurrence must transfer before a hazardous call	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:c584b8f493312b86
+B05	std::vec::Vec<rum_core::expr::Expr>::len	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	Vec length reads initialized length even with spare capacity	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrow only; no owner or capacity transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:b55a15fcf316662f
+B06	std::option::Option<&rum_core::expr::Symbol>::map<bool,literal>	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	None calls zero times; Some once; exact borrowed capture; nonescape pending	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:ae432386370e3a6a
+B07	std::option::Option<bool>::unwrap_or	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	complete scalar None/Some false/Some true table pending	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:9d228b75b99940df
+B08	rum_core::expr::Symbol::is_system_name	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	borrowed context/name predicate; allocation observations pending	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:36968a769f57e4a0
+B09	rum_runtime::runtime::model::session::Session::evaluate	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	actual armed resource checkpoints only; no between-checkpoint peak bound	caller prefix already exposed by typed bridge; bare evaluate creates no receipt	armed checkpoints check maintained S plus R and visible expression W	armed checkpoints check maintained S plus R and visible expression W	existing occurrence metric; capacity not length; checked overflow; no RSS bound	B12: root transfer plus spare-capacity complement; B09: no output receipt; B03: membership only	normal/error/suspend/resume/abandon/unwind per named runtime controls; abort has no return	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	cargo test --workspace; named runtime falsifier and local stale/Pending controls	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs#retention_r_is_separate_from_s_and_both_admission_entries_see_it	remove retention contribution from expression admission; limit-crossing control must fail	Discharged	fnv1a64:6122766c9a1e23f2
+B10	rum_core::expr::Expr::call	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_core::expr::Expr::call; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	only rule::finish_rule; adopted RHS; charged cloned name; exact Rule head; shell and actual child Vec capacity checked; unchanged indexed adapter	may-run-guest-or-grow	consumes charged head and two children; outgoing RetainedResult owns receipt until scope return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:ba390cf7e3010079
+B11	rum_runtime::runtime::expr_helpers::system_expr	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::expr_helpers::system_expr; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	literal Rule only in unchanged rule::finish_rule; no arbitrary name or interner bound	produces-owned;inert	System Rule identity; OwnedUnleased until head receipt resized; cold/warm observations	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:86dc6a51ad255186
+B12	rum_runtime::runtime::model::session::Session::insert_suspension_frame	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	private lease-bearing RetentionFrame protocol only; legacy raw frames excluded	initialized InputRequired; typed suspend reserves frame and owns frozen prefix	lease-bearing RetentionFrame transfers once; roots plus frozen complement charged	lease-bearing RetentionFrame transfers once; roots plus frozen complement charged	existing occurrence metric; capacity not length; checked overflow; no RSS bound	B12: root transfer plus spare-capacity complement; B09: no output receipt; B03: membership only	normal/error/suspend/resume/abandon/unwind per named runtime controls; abort has no return	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	cargo test --workspace; named runtime falsifier and local stale/Pending controls	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs#retention_suspension_transfers_once_resumes_twice_and_abandons	omit transfer; transfer-count assertion fails; resume and abandon are controls	Discharged	fnv1a64:202ef6687626b8f2
+I01	<rum_core::expr::SharedArgs as std::ops::Deref>::deref	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	SharedArgs deref borrows the same Vec with pointer identity	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrow only; no owner transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:abf172a5c950dabe
+I02	<rum_core::expr::SharedExpr as std::ops::Deref>::deref	concrete option-read instance only	fn deref(&SharedExpr)->&Expr; exact Target=Expr and preserved input lifetime	exact pinned helper or SharedExpr Deref; no general dispatch	shared immutable input; exact declaration and compiler selection checked	borrows;inert	borrows same Expr with pointer identity; no owner or capacity receipt	no owned output; no quantitative clone or capacity claim	input remains borrowed; no field move or ownership transfer	no guest callbacks; borrowed locals need no owned cleanup on return or unwind	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_option_read_contract_controls; local_option_read_claims_match_stable_rustc; invariant types and same-name negatives	contracts.rs::DEPENDENCIES plus all crate Rust/Python/TOML; nineteen-field record; direct index-time declaration checks	crates/rum-stdlib/src/tests/retention_closure/local/reads/tests.rs#local_option_read_contract_controls	changed concrete declaration or stale/Pending mark rejects; current borrowed read passes; helper truth table and pointer controls	Discharged	fnv1a64:0c9b9390eb4cee2f
+I03	<std::vec::Vec<rum_core::expr::Expr> as std::ops::Deref>::deref	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	brief-537 finite domain; undischarged	Pending: caller ownership and exact adapter proof	Pending: no guarantee available to callers	borrow initialized Vec elements as slice; do not clone capacity	existing occurrence metric; capacity not length; checked overflow; no RSS bound	Pending: no ownership transfer permission	Pending: callbacks and exit cleanup unproved	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	Pending: brief-537 observation set not installed	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	Pending: brief-537 report section 6	Pending: specified mutation has not been discharged	Pending	fnv1a64:0fbe2e67970caf32
+I04	<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index	exact domain only; no wildcard instance	signature and implementation pinned by dependency files	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	Vec<Expr> indexing by usize borrows one element; bounds panic unwinds	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrow only; no clone or owner transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; nineteen-field record excluding fingerprint	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:1c4f95ef4be42e5b
+P01	<rum_core::expr::Expr as std::cmp::PartialEq>::eq	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	Expr equality borrows both operands; no guest or retained growth	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrows only; no owner transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:386207d6d775118a
+P02	rum_core::expr::Expr::small_integer	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	small_integer constructs an inline Expr from exactly i64	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	inline result; no retention receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:c3b64c48d53c2909
+P03	rum_core::expr::Expr::string::<std::string::String>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	String input moves into Expr; raw payload state is preserved	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	String owner moves into Expr; no retention receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:b3b714d006fba374
+P04	str::repeat	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	str repeat produces a fresh raw String from usize count	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	borrows str; caller owns returned String; no retention receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:a761022469f2253b
+P05	<rum_core::expr::SharedArgs as std::clone::Clone>::clone	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	produces-owned;inert	SharedArgs clone shares the same allocation with pointer identity	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	new shared handle; no deep copy or retention receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:e5f541bd43ffce4e
+P06	std::option::Option<usize>::is_some	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	borrows;inert	Option<usize> is_some reads discriminant without callbacks	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	shared borrow; false for None and true for Some	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:70ae1b3902565272
+P070	rum_runtime::runtime::accounting_mode::retained::RetentionContext::reserve::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	reserve accounts vector capacity in the selected mode	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	mutable values borrow; owner remains charged	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:aafbb77ae0cbcc72
+P080	rum_runtime::runtime::accounting_mode::retained::RetentionContext::push::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	push consumes input and charges occurrence plus capacity before guest reentry	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves Expr into charged values before hazardous work	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:0f07869490b74db9
+P090	rum_runtime::runtime::accounting_mode::retained::RetentionContext::evaluate::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	evaluate exposes prefix before guest and returns typed callback bridge	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves prefix and guest input into bridge; guest result has no implicit receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:e99a946253a3867b
+P100	rum_runtime::runtime::accounting_mode::retained::RetentionContext::finish_list::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	finish_list transfers charged prefix into typed result	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes prefix and context; cleanup releases on failure	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:3465c4d35b1f64b6
+P110	rum_runtime::runtime::accounting_mode::retained::RetentionContext::resume_values::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	resume_values restores frozen prefix plus capacity into mode-specific values	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes empty values; borrows frozen prefix; avoids duplicate transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:abc9b032beed5811
+P120	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::adopt::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	adopt reconstitutes typed context and charged values from successful bridge	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes CallbackValue once; failures preserve typed ownership	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:6ccd06bee1979740
+P130	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend::<rum_runtime::runtime::accounting_mode::tests::ProbeContinuation,literal>::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	suspend invokes immediate literal once to construct exact ProbeContinuation	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves failure into frozen prefix/frame; frame roots plus spare complement charged	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:40efe4d2e84d1adf
+P140	rum_runtime::runtime::accounting_mode::retained::scoped::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	scoped calls immediate literal once with branded context and values	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	borrowed Session; result consumed at exit; no brand or closure escape	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:52837a2661ce9a53
+P150	std::result::Result::and_then::<CallbackValue::adopt>::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	and_then calls exact adopt item once on Ok and never on Err	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	Result consumes receiver; both branches preserve exact failure mode	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:09c5184cf12edf9b
+P071	rum_runtime::runtime::accounting_mode::retained::RetentionContext::reserve::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	reserve accounts vector capacity in the selected mode	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	mutable values borrow; owner remains charged	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:23946b5f54708f7f
+P081	rum_runtime::runtime::accounting_mode::retained::RetentionContext::push::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	push consumes input and charges occurrence plus capacity before guest reentry	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves Expr into charged values before hazardous work	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:097c28614a853692
+P091	rum_runtime::runtime::accounting_mode::retained::RetentionContext::evaluate::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	evaluate exposes prefix before guest and returns typed callback bridge	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves prefix and guest input into bridge; guest result has no implicit receipt	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:b9f396e1ae0e8bca
+P101	rum_runtime::runtime::accounting_mode::retained::RetentionContext::finish_list::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	finish_list transfers charged prefix into typed result	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes prefix and context; cleanup releases on failure	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:a3cca0fff450f511
+P111	rum_runtime::runtime::accounting_mode::retained::RetentionContext::resume_values::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	resume_values restores frozen prefix plus capacity into mode-specific values	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes empty values; borrows frozen prefix; avoids duplicate transfer	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:1b6cbf5bfb1d67e2
+P121	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::adopt::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	adopt reconstitutes typed context and charged values from successful bridge	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	consumes CallbackValue once; failures preserve typed ownership	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:c60a53e1c7d2e275
+P131	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend::<rum_runtime::runtime::accounting_mode::tests::ProbeContinuation,literal>::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	suspend invokes immediate literal once to construct exact ProbeContinuation	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	moves failure into frozen prefix/frame; frame roots plus spare complement charged	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:632edb5ac9854d06
+P141	rum_runtime::runtime::accounting_mode::retained::scoped::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	scoped calls immediate literal once with branded context and values	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	borrowed Session; result consumed at exit; no brand or closure escape	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:e9f4f17c3339b5aa
+P151	std::result::Result::and_then::<CallbackValue::adopt>::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	concrete instance in identity	exact signature checked by invariant oracle	exact concrete instance only; no open dispatch	checked exact receiver and arguments; no inferred receipt	may-run-guest-or-grow	and_then calls exact adopt item once on Ok and never on Err	occurrence and spare-capacity accounting only in typed protocol; inert calls create no receipt	Result consumes receiver; both branches preserve exact failure mode	normal return and unwind release; typed Result errors preserve ownership; callbacks only where explicitly stated	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_concrete_contract_controls; local_probe_claims_match_stable_rustc; accounting_mode normal/error/drop/unwind/suspend/resume/abandon tests	contracts.rs::DEPENDENCIES; record fields	crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs#local_concrete_contract_controls	stale fingerprint red then current green; exact type and resolution negative controls	Discharged	fnv1a64:53f04fe8c035e100
+B13	rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name	concrete option-read instance only	fn is_system_symbol_name(&Expr,&str)->bool	exact pinned helper or SharedExpr Deref; no general dispatch	shared immutable input; exact declaration and compiler selection checked	borrows;inert	None=false; System matching name=true; different name or context=false; scalar result	no owned output; no quantitative clone or capacity claim	input remains borrowed; no field move or ownership transfer	no guest callbacks; borrowed locals need no owned cleanup on return or unwind	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Test-checked	local_option_read_contract_controls; local_option_read_claims_match_stable_rustc; invariant types and same-name negatives	contracts.rs::DEPENDENCIES plus all crate Rust/Python/TOML; nineteen-field record; direct index-time declaration checks	crates/rum-stdlib/src/tests/retention_closure/local/reads/tests.rs#local_option_read_contract_controls	changed concrete declaration or stale/Pending mark rejects; current borrowed read passes; helper truth table and pointer controls	Discharged	fnv1a64:3c53b70f67aaf66a
+R01	rum_runtime::runtime::accounting_mode::rule::clone_result	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_result; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records	may-run-guest-or-grow	clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:8e1e5d0f058493eb
+R02	rum_runtime::runtime::accounting_mode::rule::evaluate_rule	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::evaluate_rule; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records	may-run-guest-or-grow	clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:9fad3955ade173b4
+R03	rum_runtime::runtime::accounting_mode::rule::resume_rule	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::resume_rule; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records	may-run-guest-or-grow	clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:f55be704767cbac7
+R040	rum_runtime::runtime::accounting_mode::rule::clone_owned::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_owned::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:7f3acd18fe418d8b
+R050	rum_runtime::runtime::accounting_mode::rule::clone_child::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_child::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:e9d5c6d87753beae
+R060	rum_runtime::runtime::accounting_mode::rule::finish_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::finish_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:3405bc51ce7ac3c3
+R070	rum_runtime::runtime::accounting_mode::rule::evaluate_in::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::evaluate_in::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:71e9f07b2c26f592
+R080	rum_runtime::runtime::accounting_mode::rule::resume_in::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::resume_in::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:e353fee21b68f75b
+R090	rum_runtime::runtime::accounting_mode::retained::RetentionContext::adopt_outcome::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::retained::RetentionContext::adopt_outcome::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:d26c62474f4df4f6
+R100	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::into_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::into_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:14f7522f45355bfc
+R110	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend_rule::<rum_runtime::runtime::accounting_mode::counter::Unarmed>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private Unarmed instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:d17d7637daf525c1
+R041	rum_runtime::runtime::accounting_mode::rule::clone_owned::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_owned::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:228d1a397ed2b56a
+R051	rum_runtime::runtime::accounting_mode::rule::clone_child::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::clone_child::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:77bcc277e0d192fb
+R061	rum_runtime::runtime::accounting_mode::rule::finish_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::finish_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:d2a2989360479a70
+R071	rum_runtime::runtime::accounting_mode::rule::evaluate_in::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::evaluate_in::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:449996525d4cc2dd
+R081	rum_runtime::runtime::accounting_mode::rule::resume_in::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::resume_in::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:f0502e27f6cb39b6
+R091	rum_runtime::runtime::accounting_mode::retained::RetentionContext::adopt_outcome::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::retained::RetentionContext::adopt_outcome::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:c5dd9cd26d723c13
+R101	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::into_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::bridge::CallbackValue::into_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:f07b97e1d0b22203
+R111	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::bridge::CallbackFailure::suspend_rule::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	private AccountingShadow instance; source-pinned schema; branded live owner and exact empty or one-RHS state; no raw public extraction	may-run-guest-or-grow	actual clone/adopt/reserve/handoff or lazy freeze; error and unwind retire receipts; output owner preserves charged capacity until terminal return	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:884f9dae46e14a68
+R12	rum_runtime::runtime::accounting_mode::frozen::FrozenValues::rule_name	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::frozen::FrozenValues::rule_name; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	one-name FrozenValues from private rule suspension; immutable borrow	may-run-guest-or-grow	borrow frozen name without consuming shared frame or cancelling complement	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:b8806a38f6ec9228
+R13	rum_runtime::runtime::options::resume_frames::RuntimeOptionSuspensionFrame::resume	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::options::resume_frames::RuntimeOptionSuspensionFrame::resume; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	exact rootless descriptor inside private RetentionFrame; frozen name alive; raw outcome admitted by resume_rule before growth	may-run-guest-or-grow	shared frame remains owner; result independently adopted and charged; Err payload forwarded; no second frame transfer	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:269905def62fee4c
+R14	rum_runtime::runtime::accounting_mode::rule::finish_armed	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::finish_armed; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	Unarmed callback left current limit armed; empty prefix; terminal incoming Expr adopted before growth	may-run-guest-or-grow	nested AccountingShadow scope charges callback and rule; outgoing W forwarded directly	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:4ebdd34937d024b6
+R15	rum_runtime::runtime::accounting_mode::rule::suspend_armed	private delayed-option rule domain; stable Rust edition2024; both workspace features	rum_runtime::runtime::accounting_mode::rule::suspend_armed; exact compiled/indexed signature and body	flat private protocol; no arbitrary macro, trait, caller or generic admission	Unarmed callback left actual pending InputRequired and a new active limit	may-run-guest-or-grow	reselect AccountingShadow before name clone or vector growth; freeze once; terminal error preserves request	current occurrence proxy: actual clone capacity, numeric payload, child capacity, literal head and shell; no allocator/interner theorem	live source receipt -> destination -> outgoing W or private frozen frame; transfer once; shared frame keeps own receipt	ordinary Expr-bearing Err terminal; initialized InputRequired only; pure abandon/reset/drop; panic unwinds receipts	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	option_rule_* runtime controls; local_rule_contract_controls; local_rule_macro_definition_arms_hygiene_and_raw_frame_controls	explicit FILES, macro definition, fields, exact mode, source/body/signatures and full contract context; no graph	crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs#local_rule_contract_controls	runtime omissions watched red then restored green; stale/Pending/missing dependency, bare B10/B11/B12, raw clone and result reject	Discharged	fnv1a64:873f48cb49ee0dba
+O010	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>	exact mode and descriptor arguments in identity; hash={false,true} builds reconciled separately	exact private function signature and reborrows checked by local_option_contract_controls	repository declaration build only; host-minted declarations and outside types excluded	entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor	may-run-guest-or-grow	B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged	existing occurrence metric, capacity not length, shared control bytes, checked overflow; unarmed receipts are unit	one original owner across guest calls; frozen S roots and R complement; separately charged working copy; copied consumer outputs outside scope	success/error/unwind, actual repeated input and abandon; nested guest late arm retains original; no returning guarantee for process abort	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	runtime option_owner and frozen option controls; outside public API test; seven compiled charge/domain omission controls; local_option_contract_controls	flat option_boundaries FILES; reconciled declaration/installer/impl/runtime/feature keys; B03 B04 B09 B12 and R records; no traversal	crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs#local_option_contract_controls	defaults/capacity/adoption/complement/copy/handoff/membership omissions red then restored green; stale Pending body schema raw and outside-R reject	Discharged	fnv1a64:0eb8a928c37e3fd0
+O0200	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>	exact mode and descriptor arguments in identity; hash={false,true} builds reconciled separately	exact private function signature and reborrows checked by local_option_contract_controls	repository declaration build only; host-minted declarations and outside types excluded	entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor	may-run-guest-or-grow	B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence	existing occurrence metric, capacity not length, shared control bytes, checked overflow; unarmed receipts are unit	one original owner across guest calls; frozen S roots and R complement; separately charged working copy; copied consumer outputs outside scope	success/error/unwind, actual repeated input and abandon; nested guest late arm retains original; no returning guarantee for process abort	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	runtime option_owner and frozen option controls; outside public API test; seven compiled charge/domain omission controls; local_option_contract_controls	flat option_boundaries FILES; reconciled declaration/installer/impl/runtime/feature keys; B03 B04 B09 B12 and R records; no traversal	crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs#local_option_contract_controls	defaults/capacity/adoption/complement/copy/handoff/membership omissions red then restored green; stale Pending body schema raw and outside-R reject	Discharged	fnv1a64:4d4b1bfab085c053
+O011	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	exact mode and descriptor arguments in identity; hash={false,true} builds reconciled separately	exact private function signature and reborrows checked by local_option_contract_controls	repository declaration build only; host-minted declarations and outside types excluded	entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor	may-run-guest-or-grow	B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged	existing occurrence metric, capacity not length, shared control bytes, checked overflow; unarmed receipts are unit	one original owner across guest calls; frozen S roots and R complement; separately charged working copy; copied consumer outputs outside scope	success/error/unwind, actual repeated input and abandon; nested guest late arm retains original; no returning guarantee for process abort	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	runtime option_owner and frozen option controls; outside public API test; seven compiled charge/domain omission controls; local_option_contract_controls	flat option_boundaries FILES; reconciled declaration/installer/impl/runtime/feature keys; B03 B04 B09 B12 and R records; no traversal	crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs#local_option_contract_controls	defaults/capacity/adoption/complement/copy/handoff/membership omissions red then restored green; stale Pending body schema raw and outside-R reject	Discharged	fnv1a64:768e78d9dc0c8843
+O0210	rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>	exact mode and descriptor arguments in identity; hash={false,true} builds reconciled separately	exact private function signature and reborrows checked by local_option_contract_controls	repository declaration build only; host-minted declarations and outside types excluded	entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor	may-run-guest-or-grow	B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence	existing occurrence metric, capacity not length, shared control bytes, checked overflow; unarmed receipts are unit	one original owner across guest calls; frozen S roots and R complement; separately charged working copy; copied consumer outputs outside scope	success/error/unwind, actual repeated input and abandon; nested guest late arm retains original; no returning guarantee for process abort	allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt	Runtime-enforced	runtime option_owner and frozen option controls; outside public API test; seven compiled charge/domain omission controls; local_option_contract_controls	flat option_boundaries FILES; reconciled declaration/installer/impl/runtime/feature keys; B03 B04 B09 B12 and R records; no traversal	crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs#local_option_contract_controls	defaults/capacity/adoption/complement/copy/handoff/membership omissions red then restored green; stale Pending body schema raw and outside-R reject	Discharged	fnv1a64:9cf8e0a795b9ff08
--- original/crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv
+++ candidate/crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv
@@ -2,2 +2,2 @@
-C01	LocalCovered	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:a4ee626a726db962", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:327ac5e76d71910f;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:c9b2bdf935db849e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:9026ed53c2aef5b5;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:a4ee626a726db962", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:9fbdafc9fe2d040d;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:5a853d4bb0f77e05;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:7f38cab49c7c27e6;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:fbf744a051dec96e", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:ffa7b8fbc4a540cb;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:c9b2bdf935db849e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:9026ed53c2aef5b5;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:fbf744a051dec96e", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:8851a548843c97c5;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:5a853d4bb0f77e05;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:7f38cab49c7c27e6;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required	local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type	local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged
-C02	LocalCovered	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:d4d9fc61b870e06c", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:7cfcb1b4a5a00e7a;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:49d5c4ce1db6bda0;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:0a612fe9fa177659;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:3f538ca5540412ad;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:7d628c79b2badc7d;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:764f85a23e4ee882;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:acb627699aa61d84;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:b31fb25601e0ee59;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:5c08dcb6c6fde1f0", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:0b85ff70454fc652;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:49d5c4ce1db6bda0;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:0a612fe9fa177659;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:3f538ca5540412ad;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:7d628c79b2badc7d;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:764f85a23e4ee882;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:acb627699aa61d84;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:b31fb25601e0ee59;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required	local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type	local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged
+C01	LocalCovered	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:96fe870635d4f469", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:b2dd57a67a6c28ad;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:9cf8e0a795b9ff08;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:768e78d9dc0c8843;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:96fe870635d4f469", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:3576fc0362ea31e3;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:4d4b1bfab085c053;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:0eb8a928c37e3fd0;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:e67a46d7ebeda6cc", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:8706354d7866a7b8;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:9cf8e0a795b9ff08;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:768e78d9dc0c8843;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:e67a46d7ebeda6cc", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:5742df63aa79d41a;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:4d4b1bfab085c053;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:0eb8a928c37e3fd0;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required	local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type	local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged
+C02	LocalCovered	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:e1b6bd79409f9827", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:d7f8c4500e1b7f7b;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:abf172a5c950dabe;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:0c9b9390eb4cee2f;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:1c4f95ef4be42e5b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:8e1e5d0f058493eb;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:9fad3955ade173b4;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:3c53b70f67aaf66a;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:b55a15fcf316662f;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:ccc66d3e8285d9e5;rustflags=None", body_fingerprint: "fnv1a64:a788a28eda1f98b6", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:8fde2ca8acb8ceb3;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:abf172a5c950dabe;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:0c9b9390eb4cee2f;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:1c4f95ef4be42e5b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:8e1e5d0f058493eb;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:9fad3955ade173b4;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:3c53b70f67aaf66a;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:b55a15fcf316662f;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required	local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type	local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged
```

</details>

### 2. The engine and the entry, as built

The engine is inert for every existing profiled phase. Only isolated unit and native test spans create logical calls. `EpochId`, `RecordId` and `SegmentId` are distinct identities; aggregate keys remain immutable. An owner pins the request-control `Arc` together with its request generation and root identity. Session addresses and bare suspension generations are never owner identities. Equal recursive keys therefore remain separate records. A process-wide registry owns records and rows; markers and contexts own epoch references, while the registry owns neither markers nor contexts, avoiding a strong cycle.

Each native interval owns an `Instant` only while executing. Pausing ends the interval before returning to the host. Value-free context leases retain ancestor record identities and saved evaluation/rebuild/source/argument-site scopes, restore them for resumed work, and unwind in reverse order. A second pause updates the same inherited records. Native segment guards are not `Send` or `Sync`; suspended leases are movable. Strict segment identity and LIFO checks invalidate a faulty epoch instead of popping an equal-key peer. Record closure is idempotent across markers, RAII fallback, reset, abandonment and last drop.

`Entry::ProfileBoundary` carries a thin `Arc`. Its observer accepts the passing `&Result<Slot, EvalError>` and applies the frozen outcome policy directly to the borrowed channel. It calls no encoder or decoder, stores no guest value and constructs no continuation token. Tests retain the original slot and `SharedArgs` pointer identities. Wrong channels invalidate diagnostic state without altering the outcome. `InputRequired` is nonterminal. Success, error, `Return`, `Throw`, cancellation and structural abandonment remain separate outcomes.

Explicit insertion APIs validate the request owner and generation, suspension generation, and stable queue-cell frontier. They support producer frontiers, after-consumer markers, deferred markers after `EvaluationReturn`, empty semantic queues, and spans begun after input was returned. The driver dispatches markers without consuming the outcome. The enabled mode restores per-cell logical contexts; abandonment restores the same contexts through the existing cleanup worklist, including cleanup that suspends again. No existing frame gained a field; no production phase adopted the engine.

Diagnostic storage has a separate bounded budget: process-wide and epoch limits, storage reservations preceding growth, and visible epoch invalidation on exhaustion. An initial reservation failure remains latched until explicit profile reset. Queue metadata keeps a marker-free guest queue using the original capacity/widen/append history; diagnostic cells duplicate frame `Arc` ownership only until the same cell is popped, without copying guest trees or extending their semantic lifetime. Guest accounting uses the guest queue. The occurrence oracle independently counts its capacities and marker-free payloads; it does not call production accounting. Context/source buffers, record/row/key storage, queue boxes/cells, and TLS storage are charged separately. TLS metadata remains charged until thread exit, with a conservative 4,096-byte registration allowance. Idle queries avoid initializing the drop-bearing TLS stack.

The disabled-path goal is **not achieved**. Allocation equality is established, but queue/mode/context-guard code changes add measured instruction costs, and several emitted prologues grow. The source remains a reviewable, uncommitted candidate with those defects explicitly reported, rather than a successful zero-cost gate result.

### 3. Route proofs

The final logical-engine suite passes 20 tests. The raw slot tests cover closed channels, preserved slot/argument identity, and wrong-tag faults. Record tests cover seven terminal classes, repeated-key identities, strict recursive-peer segment rejection, active abandonment ending its interval once, reset isolation, cap exhaustion and latched initial failure. Budget tests prove metadata stays charged while its thread remains alive and is released on thread exit. Queue tests cover empty/already-returned frontiers, six structural reasons, diagnostic exhaustion with unchanged marker-free capacity, return-boundary resource errors, a real driver slot fault, foreign-owner rejection even at the same bare generation, and an after-consumer parser error.

The native host test installs a test-only `LpSpan` through the actual host builder. It exercises two real pauses on one record (pause totals 1, 2, then 2), terminal success/error/Return/Throw, cancellation, session reset/drop, interleaved requests and a moved session, active profile reset, failed restore, rollback, helper discard, cleanup that suspends again, and resumed typed collection. Every checkpoint asserts terminal counts, zero open records and zero context leases. Its child process owns the enabled environment; no new unsafe environment mutation is used.

The stronger tests exposed real defects in second-pause telemetry, strict active-peer handling and retained TLS-budget charging; those red receipts and subsequent green receipts remain under `logs/`. One newly written marker-only fixture incorrectly expected two intervals despite doing no resumed work. It was corrected to one interval, and a separate active-abandonment test proves the interval-closing behavior. That fixture correction is not claimed as a killed implementation bug. The original rejection-census assertion remains 950; the first redundant abandonment loop was replaced with a natural optional front-cell/context helper, preserving the assertion instead of changing its contract.


Native route receipts, verbatim:
```text
test logical_native_lifecycle ... LP02_NATIVE second-pause success closed-once pauses=2
LP02_NATIVE error return throw closed-once
LP02_NATIVE cancellation closed-once
LP02_NATIVE session-reset abandoned-once
LP02_NATIVE session-drop abandoned-once
LP02_NATIVE interleaved moved-session closed-once
LP02_NATIVE profile-reset active-call closed-once
LP02_NATIVE failed-restore abandoned-once
LP02_NATIVE rollback helper-discard abandoned-once
LP02_NATIVE nested-cleanup re-suspended abandoned-once
LP02_NATIVE typed-collection resumed completion
```

The final 20-test logical suite, native lifecycle test and stronger red/green development receipts are retained in [logs/](../../../.temp/brief-626/logs/). No oracle flag, retention reject, fault-absence check or documented contract was weakened.

### 4. LP01 counter oracle

Both canonical builds pass all 499 tests. All **8,750** logical-profile receipts execute. The **118,991 pinned loss cells per build** remain unchanged. All ten complete actual-JSON collections and execution lists equal the accepted LP01 artifacts, not merely their totals. Main-fault observations are 96,248 per build and logical-child observations are 9,552 for each of the ten collections; every fault delta is zero. Full comparison: [oracle-comparison.json](../../../.temp/brief-626/oracle-comparison.json); raw current receipts: [lp01-receipts/](../../../.temp/brief-626/lp01-receipts/); canonical raw gates: [suspension-oracle/](../../../.temp/brief-626/suspension-oracle/). The canonical gates overwrite fixed prior artifact paths; all 30 prior LP01 paths and the two prior shell-gate logs were restored byte for byte. The workspace-generated five current logs are separately retained in [workspace-latest-lp01-artifacts/](../../../.temp/brief-626/workspace-latest-lp01-artifacts/).

### 5. All executed gates, verbatim

All long commands used `scripts/run-bounded.py`, offline Cargo, one build job and one test thread. Common environment: `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-626/tmp`. Workspace bound: 3,600 s; canonical oracle: 1,500 s per build; other bounds: 900 s. The workspace used `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Final workspace: runtime 1,202 passed/3 ignored; stdlib 1,108 passed/1 failed/53 ignored; suspension oracle 499 passed; typed census 8 passed/1 ignored. Every other target, integration suite and doctest completed successfully. The sole failed target is `rum-stdlib --lib`, test `tests::retention_closure::compiler_oracle::methods::inherent_method_claims_match_rustc`: **3,514 agreements, 5 disagreements, coverage 3,519/3,519**.

The five expected stable-rustc target markers are actually emitted at the exact claimed tokens and correct canonical files. Literal filename suffix matching fails because rustc preserves `suspension/core/../../profiling/...` or `suspension/../profiling/...` from the authorized path modules. The harness and all claims remain unchanged. This is a red gate, not an asserted pass. Exact tokens, raw diagnostics and normalized filenames: [method-path-evidence-final.json](../../../.temp/brief-626/method-path-evidence-final.json) and [method-oracle-final/](../../../.temp/brief-626/method-oracle-final/).


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

Those are **nine operational native ZeroMQ skips**. The Rust harness reports nine passed because the tests return after the allowed skip; native transport was not exercised. The director must run them natively. The runtime lib-test warnings name three pre-existing test-builtin helpers. The standalone production-library layout check additionally warns about three unused unadopted engine methods; GDB reports a missing optional auto-load script. All are retained below.

The following is the chronological gate ledger, including development reds and final reruns. Empty selected-output blocks mean no result/warning/error line was emitted; the linked file contains the complete raw output. Earlier development statuses are not substituted for final-source gates.


**engine-first — exit 0.** [logs/engine-first.log](../../../.temp/brief-626/logs/engine-first.log)


```bash
cargo test -p rum-runtime --lib profiling -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
```


**native-first — exit 101.** [logs/native-first.log](../../../.temp/brief-626/logs/native-first.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
error[E0599]: no function or associated item named `with_config` found for struct `HostSession` in the current scope
error: could not compile `rum-host` (test "logical_profile_lifecycle") due to 1 previous error
```


**native-second — exit 101.** [logs/native-second.log](../../../.temp/brief-626/logs/native-second.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
error[E0599]: no method named `build` found for struct `HostConfig` in the current scope
error: could not compile `rum-host` (test "logical_profile_lifecycle") due to 1 previous error
```


**native-third — exit 0.** [logs/native-third.log](../../../.temp/brief-626/logs/native-third.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```


**formatting-apply — exit 0.** [logs/formatting-apply.log](../../../.temp/brief-626/logs/formatting-apply.log)


```bash
cargo fmt --all
```


**engine-queue — exit 101.** [logs/engine-queue.log](../../../.temp/brief-626/logs/engine-queue.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
error[E0433]: failed to resolve: use of undeclared type `Expr`
error[E0433]: failed to resolve: use of undeclared type `Expr`
error[E0433]: failed to resolve: use of undeclared type `Expr`
error[E0433]: failed to resolve: use of undeclared type `Expr`
error[E0433]: failed to resolve: use of undeclared type `Expr`
error[E0308]: mismatched types
error: could not compile `rum-runtime` (lib test) due to 6 previous errors
```


**engine-queue-second — exit 101.** [logs/engine-queue-second.log](../../../.temp/brief-626/logs/engine-queue-second.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: variable does not need to be mutable
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 4 warnings (run `cargo fix --lib -p rum-runtime --tests` to apply 1 suggestion)
test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.03s
error: test failed, to rerun pass `-p rum-runtime --lib`
```


**engine-queue-third — exit 101.** [logs/engine-queue-third.log](../../../.temp/brief-626/logs/engine-queue-third.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.03s
error: test failed, to rerun pass `-p rum-runtime --lib`
```


**engine-queue-fourth — exit 0.** [logs/engine-queue-fourth.log](../../../.temp/brief-626/logs/engine-queue-fourth.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.02s
```


**native-extended — exit 101.** [logs/native-extended.log](../../../.temp/brief-626/logs/native-extended.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
error: test failed, to rerun pass `-p rum-host --test logical_profile_lifecycle`
```


**native-extended-second — exit 0.** [logs/native-extended-second.log](../../../.temp/brief-626/logs/native-extended-second.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```


**format-second — exit 0.** [logs/format-second.log](../../../.temp/brief-626/logs/format-second.log)


```bash
cargo fmt --all
```


**engine-final-isolation — exit 0.** [logs/engine-final-isolation.log](../../../.temp/brief-626/logs/engine-final-isolation.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.04s
```


**ownership-discovery — exit 0.** [logs/ownership-discovery.log](../../../.temp/brief-626/logs/ownership-discovery.log)


```bash
cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 6.02s
```


**fmt-check — exit 0.** [logs/fmt-check.log](../../../.temp/brief-626/logs/fmt-check.log)


```bash
cargo fmt --check
```


**typed — exit 0.** [logs/typed.log](../../../.temp/brief-626/logs/typed.log)


```bash
cargo test -p rum-runtime --lib suspension::typed::tests
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1177 filtered out; finished in 0.55s
```


**frames — exit 0.** [logs/frames.log](../../../.temp/brief-626/logs/frames.log)


```bash
cargo test -p rum-runtime --lib frame_root_trace
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 1.75s
```


**contexts — exit 0.** [logs/contexts.log](../../../.temp/brief-626/logs/contexts.log)


```bash
cargo test -p rum-runtime --lib resume_evaluation_context
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1190 filtered out; finished in 0.16s
```


**profiling — exit 0.** [logs/profiling.log](../../../.temp/brief-626/logs/profiling.log)


```bash
cargo test -p rum-runtime --lib profiling
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1196 filtered out; finished in 0.01s
```


**census-default — exit 0.** [logs/census-default.log](../../../.temp/brief-626/logs/census-default.log)


```bash
cargo test -p rum-stdlib --test typed_slot_census
```


```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 278.80s
```


**ownership-replay — exit 1.** [logs/ownership-replay.log](../../../.temp/brief-626/logs/ownership-replay.log)


```bash
python3 .temp/brief-626/replay-ownership.py
```


```text
Traceback (most recent call last):
  File "/home/remi/dev/rum/.temp/brief-626/replay-ownership.py", line 11, in <module>
    rows=updates[m['inventory']];assert rows.count(m['old'])==1
                                        ^^^^^^^^^^^^^^^^^^^^^^^
AssertionError
```


**ownership-replay-second — exit 0.** [logs/ownership-replay-second.log](../../../.temp/brief-626/logs/ownership-replay-second.log)


```bash
python3 .temp/brief-626/replay-ownership.py
```


**census-no-default — exit 0.** [logs/census-no-default.log](../../../.temp/brief-626/logs/census-no-default.log)


```bash
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
```


```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 278.55s
```


**cap-latch — exit 0.** [logs/cap-latch.log](../../../.temp/brief-626/logs/cap-latch.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.03s
```


**format-freeze — exit 0.** [logs/format-freeze.log](../../../.temp/brief-626/logs/format-freeze.log)


```bash
cargo fmt --check
```


**ownership-discovery — exit 0.** [logs/ownership-discovery.log](../../../.temp/brief-626/logs/ownership-discovery.log)


```bash
cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 5.95s
```


**ownership-preflight-final — exit 0.** [logs/ownership-preflight-final.log](../../../.temp/brief-626/logs/ownership-preflight-final.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/preflight-ownership.py
```


**ownership-replay-final — exit 0.** [logs/ownership-replay-final.log](../../../.temp/brief-626/logs/ownership-replay-final.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/replay-ownership.py
```


**fingerprints-generate — exit 0.** [logs/fingerprints-generate.log](../../../.temp/brief-626/logs/fingerprints-generate.log)


```bash
cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 0.14s
```


**certificates-review — exit 0.** [logs/certificates-review.log](../../../.temp/brief-626/logs/certificates-review.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/review-certificates.py
```


**ownership — exit 0.** [logs/ownership.log](../../../.temp/brief-626/logs/ownership.log)


```bash
cargo test -p rum-stdlib --lib ownership_graph
```


```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 71.77s
```


**acceptance-default — exit 0.** [logs/acceptance-default.log](../../../.temp/brief-626/logs/acceptance-default.log)


```bash
cargo test -p rum-stdlib --lib local_production_acceptance
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 19.07s
```


**acceptance-no-default — exit 0.** [logs/acceptance-no-default.log](../../../.temp/brief-626/logs/acceptance-no-default.log)


```bash
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.44s
```


**rejection — exit 101.** [logs/rejection.log](../../../.temp/brief-626/logs/rejection.log)


```bash
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
```


```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 17.92s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```


**profiling-final — exit 0.** [logs/profiling-final.log](../../../.temp/brief-626/logs/profiling-final.log)


```bash
cargo test -p rum-runtime --lib profiling
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1196 filtered out; finished in 0.05s
```


**native-final — exit 0.** [logs/native-final.log](../../../.temp/brief-626/logs/native-final.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```


**rejection-refactor — exit 101.** [logs/rejection-refactor.log](../../../.temp/brief-626/logs/rejection-refactor.log)


```bash
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
```


```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 26.62s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```


**format-refactor — exit 0.** [logs/format-refactor.log](../../../.temp/brief-626/logs/format-refactor.log)


```bash
cargo fmt --check
```


**ownership-discovery — exit 0.** [logs/ownership-discovery.log](../../../.temp/brief-626/logs/ownership-discovery.log)


```bash
cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 5.84s
```


**ownership-preflight-final — exit 0.** [logs/ownership-preflight-final.log](../../../.temp/brief-626/logs/ownership-preflight-final.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/preflight-ownership.py
```


**ownership-replay-final — exit 0.** [logs/ownership-replay-final.log](../../../.temp/brief-626/logs/ownership-replay-final.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/replay-ownership.py
```


**fingerprints-generate — exit 0.** [logs/fingerprints-generate.log](../../../.temp/brief-626/logs/fingerprints-generate.log)


```bash
cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 0.20s
```


**certificates-review — exit 0.** [logs/certificates-review.log](../../../.temp/brief-626/logs/certificates-review.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/review-certificates.py
```


**ownership — exit 0.** [logs/ownership.log](../../../.temp/brief-626/logs/ownership.log)


```bash
cargo test -p rum-stdlib --lib ownership_graph
```


```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 70.90s
```


**acceptance-default — exit 0.** [logs/acceptance-default.log](../../../.temp/brief-626/logs/acceptance-default.log)


```bash
cargo test -p rum-stdlib --lib local_production_acceptance
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.98s
```


**acceptance-no-default — exit 0.** [logs/acceptance-no-default.log](../../../.temp/brief-626/logs/acceptance-no-default.log)


```bash
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.41s
```


**rejection-final — exit 0.** [logs/rejection-final.log](../../../.temp/brief-626/logs/rejection-final.log)


```bash
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 26.24s
```


**profiling-final — exit 0.** [logs/profiling-final.log](../../../.temp/brief-626/logs/profiling-final.log)


```bash
cargo test -p rum-runtime --lib profiling
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1196 filtered out; finished in 0.00s
```


**native-final — exit 0.** [logs/native-final.log](../../../.temp/brief-626/logs/native-final.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```


**oracle — exit 0.** [logs/oracle.log](../../../.temp/brief-626/logs/oracle.log)


```bash
python3 .temp/brief-626/run-oracle.py
```


```text
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.80s
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.23s
```


**oracle-compare — exit 0.** [logs/oracle-compare.log](../../../.temp/brief-626/logs/oracle-compare.log)


```bash
python3 .temp/brief-626/verify-oracle.py
```


**engine-final-2 — exit 0.** [logs/engine-final-2.log](../../../.temp/brief-626/logs/engine-final-2.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.03s
```


**typed-final — exit 0.** [logs/typed-final.log](../../../.temp/brief-626/logs/typed-final.log)


```bash
cargo test -p rum-runtime --lib suspension::typed::tests
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1178 filtered out; finished in 0.48s
```


**frames-final — exit 0.** [logs/frames-final.log](../../../.temp/brief-626/logs/frames-final.log)


```bash
cargo test -p rum-runtime --lib frame_root_trace
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 1.63s
```


**contexts-final — exit 0.** [logs/contexts-final.log](../../../.temp/brief-626/logs/contexts-final.log)


```bash
cargo test -p rum-runtime --lib resume_evaluation_context
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1191 filtered out; finished in 0.16s
```


**census-default-final — exit 0.** [logs/census-default-final.log](../../../.temp/brief-626/logs/census-default-final.log)


```bash
cargo test -p rum-stdlib --test typed_slot_census
```


```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 279.73s
```


**census-no-default-final — exit 0.** [logs/census-no-default-final.log](../../../.temp/brief-626/logs/census-no-default-final.log)


```bash
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
```


```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 278.62s
```


**workspace — exit 101.** [logs/workspace.log](../../../.temp/brief-626/logs/workspace.log)


```bash
cargo test --workspace --no-fail-fast
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.96s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 72.82s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.12s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 1199 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 51.68s
test result: FAILED. 1108 passed; 1 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1406.41s
error: test failed, to rerun pass `-p rum-stdlib --lib`
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.25s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.96s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 101.29s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 280.20s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 67.14s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.17s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.78s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
```


**lifecycle-regressions-red — exit 101.** [logs/lifecycle-regressions-red.log](../../../.temp/brief-626/logs/lifecycle-regressions-red.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: FAILED. 16 passed; 3 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.03s
error: test failed, to rerun pass `-p rum-runtime --lib`
```


**native-pause-regression-red — exit 101.** [logs/native-pause-regression-red.log](../../../.temp/brief-626/logs/native-pause-regression-red.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
error: test failed, to rerun pass `-p rum-host --test logical_profile_lifecycle`
```


**lifecycle-format — exit 0.** [logs/lifecycle-format.log](../../../.temp/brief-626/logs/lifecycle-format.log)


```bash
cargo fmt --all
```


**lifecycle-regressions-green — exit 101.** [logs/lifecycle-regressions-green.log](../../../.temp/brief-626/logs/lifecycle-regressions-green.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: FAILED. 18 passed; 1 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.03s
error: test failed, to rerun pass `-p rum-runtime --lib`
```


**lifecycle-format-2 — exit 0.** [logs/lifecycle-format-2.log](../../../.temp/brief-626/logs/lifecycle-format-2.log)


```bash
cargo fmt --all
```


**lifecycle-regressions-green-2 — exit 0.** [logs/lifecycle-regressions-green-2.log](../../../.temp/brief-626/logs/lifecycle-regressions-green-2.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.03s
```


**native-pause-regression-green — exit 0.** [logs/native-pause-regression-green.log](../../../.temp/brief-626/logs/native-pause-regression-green.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```


**method-path-evidence — exit 0.** [logs/method-path-evidence.log](../../../.temp/brief-626/logs/method-path-evidence.log)


```bash
python3 .temp/brief-626/verify-method-paths.py
```


**ownership-discovery — exit 0.** [logs/ownership-discovery.log](../../../.temp/brief-626/logs/ownership-discovery.log)


```bash
cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 5.83s
```


**ownership-preflight-final — exit 0.** [logs/ownership-preflight-final.log](../../../.temp/brief-626/logs/ownership-preflight-final.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/preflight-ownership.py
```


**ownership-replay-final — exit 0.** [logs/ownership-replay-final.log](../../../.temp/brief-626/logs/ownership-replay-final.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/replay-ownership.py
```


**fingerprints-generate — exit 0.** [logs/fingerprints-generate.log](../../../.temp/brief-626/logs/fingerprints-generate.log)


```bash
cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 0.30s
```


**certificates-review — exit 0.** [logs/certificates-review.log](../../../.temp/brief-626/logs/certificates-review.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/review-certificates.py
```


**ownership — exit 0.** [logs/ownership.log](../../../.temp/brief-626/logs/ownership.log)


```bash
cargo test -p rum-stdlib --lib ownership_graph
```


```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 70.72s
```


**acceptance-default — exit 0.** [logs/acceptance-default.log](../../../.temp/brief-626/logs/acceptance-default.log)


```bash
cargo test -p rum-stdlib --lib local_production_acceptance
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.88s
```


**acceptance-no-default — exit 0.** [logs/acceptance-no-default.log](../../../.temp/brief-626/logs/acceptance-no-default.log)


```bash
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.48s
```


**rejection-final — exit 0.** [logs/rejection-final.log](../../../.temp/brief-626/logs/rejection-final.log)


```bash
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 26.81s
```


**profiling-final — exit 0.** [logs/profiling-final.log](../../../.temp/brief-626/logs/profiling-final.log)


```bash
cargo test -p rum-runtime --lib profiling
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1196 filtered out; finished in 0.06s
```


**native-final — exit 0.** [logs/native-final.log](../../../.temp/brief-626/logs/native-final.log)


```bash
cargo test -p rum-host --test logical_profile_lifecycle -- --nocapture
```


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```


**engine-final-2-latest — exit 0.** [logs/engine-final-2-latest.log](../../../.temp/brief-626/logs/engine-final-2-latest.log)


```bash
cargo test -p rum-runtime --lib logical_ -- --nocapture
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1185 filtered out; finished in 0.05s
```


**typed-final-latest — exit 0.** [logs/typed-final-latest.log](../../../.temp/brief-626/logs/typed-final-latest.log)


```bash
cargo test -p rum-runtime --lib suspension::typed::tests
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1181 filtered out; finished in 0.51s
```


**frames-final-latest — exit 0.** [logs/frames-final-latest.log](../../../.temp/brief-626/logs/frames-final-latest.log)


```bash
cargo test -p rum-runtime --lib frame_root_trace
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1190 filtered out; finished in 1.83s
```


**contexts-final-latest — exit 0.** [logs/contexts-final-latest.log](../../../.temp/brief-626/logs/contexts-final-latest.log)


```bash
cargo test -p rum-runtime --lib resume_evaluation_context
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1194 filtered out; finished in 0.16s
```


**census-default-final-latest — exit 0.** [logs/census-default-final-latest.log](../../../.temp/brief-626/logs/census-default-final-latest.log)


```bash
cargo test -p rum-stdlib --test typed_slot_census
```


```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 278.69s
```


**census-no-default-final-latest — exit 0.** [logs/census-no-default-final-latest.log](../../../.temp/brief-626/logs/census-no-default-final-latest.log)


```bash
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
```


```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 277.97s
```


**oracle-latest — exit 0.** [logs/oracle-latest.log](../../../.temp/brief-626/logs/oracle-latest.log)


```bash
python3 .temp/brief-626/run-oracle.py
```


```text
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.53s
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.26s
```


**oracle-compare-latest — exit 0.** [logs/oracle-compare-latest.log](../../../.temp/brief-626/logs/oracle-compare-latest.log)


```bash
python3 .temp/brief-626/verify-oracle.py
```


**fmt-latest — exit 0.** [logs/fmt-latest.log](../../../.temp/brief-626/logs/fmt-latest.log)


```bash
cargo fmt --check
```


**workspace-latest — exit 101.** [logs/workspace-latest.log](../../../.temp/brief-626/logs/workspace-latest.log)


```bash
cargo test --workspace --no-fail-fast
```


```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.58s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 1202 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 44.27s
test result: FAILED. 1108 passed; 1 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1371.81s
error: test failed, to rerun pass `-p rum-stdlib --lib`
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.42s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.76s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 280.51s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 55.88s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.94s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.81s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
```


**native-zmq-skips — exit 0.** [logs/native-zmq-skips.log](../../../.temp/brief-626/logs/native-zmq-skips.log)


```bash
cargo test -p rum-jupyter zeromq::tests::native -- --nocapture
```


```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


**candidate-builds — exit 0.** [logs/candidate-builds.log](../../../.temp/brief-626/logs/candidate-builds.log)


```bash
python3 .temp/brief-626/build-candidates.py
```


**perf-check-base-1-part1 — exit 0.** [logs/perf-check-base-1-part1.log](../../../.temp/brief-626/logs/perf-check-base-1-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py check base 1
```


**perf-check-candidate-1-part1 — exit 0.** [logs/perf-check-candidate-1-part1.log](../../../.temp/brief-626/logs/perf-check-candidate-1-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py check candidate 1
```


**perf-alloc-base-1-part1 — exit 0.** [logs/perf-alloc-base-1-part1.log](../../../.temp/brief-626/logs/perf-alloc-base-1-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py alloc base 1
```


**perf-alloc-candidate-1-part1 — exit 0.** [logs/perf-alloc-candidate-1-part1.log](../../../.temp/brief-626/logs/perf-alloc-candidate-1-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py alloc candidate 1
```


**allocation-comparison — exit 0.** [logs/allocation-comparison.log](../../../.temp/brief-626/logs/allocation-comparison.log)


```bash
python3 .temp/brief-626/compare-performance.py alloc
```


**perf-instructions-base-1-part1 — exit 0.** [logs/perf-instructions-base-1-part1.log](../../../.temp/brief-626/logs/perf-instructions-base-1-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions base 1
```


**perf-instructions-candidate-1-part1 — exit 0.** [logs/perf-instructions-candidate-1-part1.log](../../../.temp/brief-626/logs/perf-instructions-candidate-1-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions candidate 1
```


**perf-instructions-base-2-part1 — exit 0.** [logs/perf-instructions-base-2-part1.log](../../../.temp/brief-626/logs/perf-instructions-base-2-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions base 2
```


**perf-instructions-candidate-2-part1 — exit 0.** [logs/perf-instructions-candidate-2-part1.log](../../../.temp/brief-626/logs/perf-instructions-candidate-2-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions candidate 2
```


**instruction-comparison — exit 0.** [logs/instruction-comparison.log](../../../.temp/brief-626/logs/instruction-comparison.log)


```bash
python3 .temp/brief-626/compare-performance.py instructions
```


**copy-calibration — exit 0.** [logs/copy-calibration.log](../../../.temp/brief-626/logs/copy-calibration.log)


```bash
python3 .temp/brief-626/calibrate-copies.py
```


**copies-part1 — exit 0.** [logs/copies-part1.log](../../../.temp/brief-626/logs/copies-part1.log)


```bash
python3 .temp/brief-626/run-copy.py
```


**copy-comparison — exit 0.** [logs/copy-comparison.log](../../../.temp/brief-626/logs/copy-comparison.log)


```bash
python3 .temp/brief-626/compare-copies.py
```


**resume-attribution-part1 — exit 0.** [logs/resume-attribution-part1.log](../../../.temp/brief-626/logs/resume-attribution-part1.log)


```bash
python3 .temp/brief-626/run-resume-attribution.py
```


**instruction-attribution — exit 0.** [logs/instruction-attribution.log](../../../.temp/brief-626/logs/instruction-attribution.log)


```bash
python3 .temp/brief-626/attribute-instructions.py --complete
```


**stack-candidate-build — exit 0.** [logs/stack-candidate-build.log](../../../.temp/brief-626/logs/stack-candidate-build.log)


```bash
bash .temp/brief-626/stack-candidate/build.sh
```


**stack-candidate-measure — exit 0.** [logs/stack-candidate-measure.log](../../../.temp/brief-626/logs/stack-candidate-measure.log)


```bash
python3 .temp/brief-626/stack-candidate/measure.py
```


**stack-comparison — exit 0.** [logs/stack-comparison.log](../../../.temp/brief-626/logs/stack-comparison.log)


```bash
python3 .temp/brief-626/compare-stack-candidate.py
```


**perf-instructions-base-extra-1-part1 — exit 0.** [logs/perf-instructions-base-extra-1-part1.log](../../../.temp/brief-626/logs/perf-instructions-base-extra-1-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions base extra-1 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-candidate-extra-1-part1 — exit 0.** [logs/perf-instructions-candidate-extra-1-part1.log](../../../.temp/brief-626/logs/perf-instructions-candidate-extra-1-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions candidate extra-1 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-base-extra-2-part1 — exit 0.** [logs/perf-instructions-base-extra-2-part1.log](../../../.temp/brief-626/logs/perf-instructions-base-extra-2-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions base extra-2 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-candidate-extra-2-part1 — exit 0.** [logs/perf-instructions-candidate-extra-2-part1.log](../../../.temp/brief-626/logs/perf-instructions-candidate-extra-2-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions candidate extra-2 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-base-extra-3-part1 — exit 0.** [logs/perf-instructions-base-extra-3-part1.log](../../../.temp/brief-626/logs/perf-instructions-base-extra-3-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions base extra-3 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-candidate-extra-3-part1 — exit 0.** [logs/perf-instructions-candidate-extra-3-part1.log](../../../.temp/brief-626/logs/perf-instructions-candidate-extra-3-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions candidate extra-3 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-base-extra-4-part1 — exit 0.** [logs/perf-instructions-base-extra-4-part1.log](../../../.temp/brief-626/logs/perf-instructions-base-extra-4-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions base extra-4 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-candidate-extra-4-part1 — exit 0.** [logs/perf-instructions-candidate-extra-4-part1.log](../../../.temp/brief-626/logs/perf-instructions-candidate-extra-4-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions candidate extra-4 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-base-extra-5-part1 — exit 0.** [logs/perf-instructions-base-extra-5-part1.log](../../../.temp/brief-626/logs/perf-instructions-base-extra-5-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions base extra-5 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**perf-instructions-candidate-extra-5-part1 — exit 0.** [logs/perf-instructions-candidate-extra-5-part1.log](../../../.temp/brief-626/logs/perf-instructions-candidate-extra-5-part1.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/run-perf.py instructions candidate extra-5 all @/home/remi/dev/rum/.temp/brief-626/perf/ordinary-five-run-labels.json
```


**instruction-five-comparison — exit 0.** [logs/instruction-five-comparison.log](../../../.temp/brief-626/logs/instruction-five-comparison.log)


```bash
python3 .temp/brief-626/compare-performance.py extra
```


**layout-prepare — exit 0.** [logs/layout-prepare.log](../../../.temp/brief-626/logs/layout-prepare.log)


```bash
python3 .temp/brief-626/prepare-layout-check.py
```


**layout-headers — exit 0.** [logs/layout-headers.log](../../../.temp/brief-626/logs/layout-headers.log)


```bash
cargo check --offline --manifest-path .temp/brief-626/layout-candidate/tree/Cargo.toml -p rum-runtime --lib
```


```text
warning: method `logical_insert_boundary` is never used
warning: method `profile_boundary` is never used
warning: method `insert` is never used
warning: `rum-runtime` (lib) generated 3 warnings
```


**layout-production-build — exit 0.** [logs/layout-production-build.log](../../../.temp/brief-626/logs/layout-production-build.log)


```bash
cargo build --offline -p rum-cli --message-format=json
```


**layout-dwarf — exit 0.** [logs/layout-dwarf.log](../../../.temp/brief-626/logs/layout-dwarf.log)


```bash
gdb -q -nx -batch -x .temp/brief-626/collect-layouts.gdb .temp/brief-626/target-layout/debug/rum-cli
```


```text
warning: Missing auto-load script at offset 0 in section .debug_gdb_scripts
```


**layout-comparison — exit 0.** [logs/layout-comparison.log](../../../.temp/brief-626/logs/layout-comparison.log)


```bash
python3 .temp/brief-626/compare-layouts.py
```


**resume-chain-profiles — exit 0.** [logs/resume-chain-profiles.log](../../../.temp/brief-626/logs/resume-chain-profiles.log)


```bash
python3 .temp/brief-626/run-chains.py
```


**chain-prologues-base — exit 0.** [logs/chain-prologues-base.log](../../../.temp/brief-626/logs/chain-prologues-base.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/prologues.py base release /home/remi/dev/rum/.temp/brief-626/resume-chains/prologues/base /home/remi/dev/rum/.temp/brief-626/base/resume
```


**chain-prologues-candidate — exit 0.** [logs/chain-prologues-candidate.log](../../../.temp/brief-626/logs/chain-prologues-candidate.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/prologues.py candidate release /home/remi/dev/rum/.temp/brief-626/resume-chains/prologues/candidate /home/remi/dev/rum/.temp/brief-626/cand/resume
```


**stack-function-prologues — exit 0.** [logs/stack-function-prologues.log](../../../.temp/brief-626/logs/stack-function-prologues.log)


```bash
python3 /home/remi/dev/rum/.temp/brief-626/stack-prologues.py
```


**resume-chain-analysis — exit 0.** [logs/resume-chain-analysis.log](../../../.temp/brief-626/logs/resume-chain-analysis.log)


```bash
python3 .temp/brief-626/analyze-chains.py
```


**resume-chain-comparison — exit 0.** [logs/resume-chain-comparison.log](../../../.temp/brief-626/logs/resume-chain-comparison.log)


```bash
python3 .temp/brief-626/compare-chains.py
```


**aligned-base-build — exit 0.** [logs/aligned-base-build.log](../../../.temp/brief-626/logs/aligned-base-build.log)


```bash
python3 .temp/brief-626/build-aligned.py base
```


**aligned-candidate-build — exit 0.** [logs/aligned-candidate-build.log](../../../.temp/brief-626/logs/aligned-candidate-build.log)


```bash
python3 .temp/brief-626/build-aligned.py candidate
```


**timing-panels — exit 0.** [logs/timing-panels.log](../../../.temp/brief-626/logs/timing-panels.log)


```bash
python3 .temp/brief-626/run-timing-panels.py
```


**timing-aligned-part1 — exit 1.** [logs/timing-aligned-part1.log](../../../.temp/brief-626/logs/timing-aligned-part1.log)


```bash
python3 .temp/brief-626/run-aligned-timing.py
```


```text
ALIGNED TIMING 1 base labels 120
ALIGNED TIMING 1 candidate labels 120
CALIBRATION ratio range 0.4787391535462489 1.281434466515975 outside small factor [{'label': 'subvalue-guarded-hit', 'arm': 'base', 'probe_over_panel': 0.4934675744805838}, {'label': 'default-unguarded-hit', 'arm': 'base', 'probe_over_panel': 0.4787391535462489}]
Traceback (most recent call last):
  File "/home/remi/dev/rum/.temp/brief-626/run-aligned-timing.py", line 56, in <module>
    assert not bad,'Calibrate whole-program probe before interpreting timing deltas'
           ^^^^^^^
AssertionError: Calibrate whole-program probe before interpreting timing deltas
```


**timing-panels-recalibration — exit 0.** [logs/timing-panels-recalibration.log](../../../.temp/brief-626/logs/timing-panels-recalibration.log)


```bash
python3 .temp/brief-626/run-timing-panels.py
```


**timing-aligned-part2 — exit 0.** [logs/timing-aligned-part2.log](../../../.temp/brief-626/logs/timing-aligned-part2.log)


```bash
python3 .temp/brief-626/run-aligned-timing.py
```

### 6. Measurements, profiling off

The four preserved unedited binaries and candidates use one codegen unit and the same original target directories. Aligned timing binaries use separate targets and exactly `-C llvm-args=-align-all-functions=6`. Source hashes match the measured unaligned arms. Hashes and full commands are retained in [builds/](../../../.temp/brief-626/builds/) and [timing/aligned/](../../../.temp/brief-626/timing/aligned/).

| Arm | Binary | SHA-256 |
|---|---|---|
| base | rum-cli | `46137abe2e63c59203ef987effb3ca5eb1f78540c9a88200ae51220041b845fd` |
| base | rum-cli-alloc | `5cf3e4b67b750e904731977d4618c9aa2796e170781243f9ac3451760ec5a843` |
| base | resume | `6fdc74e0cb0027244941c2032ab480843eae87caa7b409d44e0caf86735743e5` |
| base | resume-alloc | `f305fd523786f8b4a7e658e4ca99cbdab87bae8e92fab5bfce9cc7137a85515c` |
| cand | rum-cli | `43c360bc5b72efbb244e5e16f419d967cca51cfcf98ae5e1ee7dbeb7ca71e2e6` |
| cand | rum-cli-alloc | `334375006e2e961071cd91b567384648ad7a1e484e9f4916cd7ec80cd6948e34` |
| cand | resume | `4a52a18a15ad83a8a4e57f14c527ec1ea69474d3cf9289991153d2ce84aee51b` |
| cand | resume-alloc | `059f804a09055c9d7ab0f0fe2afb7919a06050044c538476acc1b2dc8b0337c4` |
| aligned base | rum-cli | `68cdbb8e6ea9c77dc85a918c1604120aa64b725299aa27ffe0591cd32ff92038` |
| aligned candidate | rum-cli | `7cc9a17b6946c3f760d3e6990a4f0cd70c9865f5e3798f759c115d8f0ba127f1` |


The copied workload population is 313 labels: 192 ordinary and 121 resume. All 818 correctness processes per arm passed, including required resume counts. Existing explicit-arm workloads select the already-installed successful branch on both arms; no measured expression was replaced with a sink. Every result file pins its binary hash, so cached results cannot silently survive a rebuild.

**Allocations — pass.** All 313 labels are exactly equal in raw N/2N allocation counts and bytes, hence in slopes and intercepts. Both arms complete 626 processes. Full per-label before/after values: [perf/allocation-comparison.tsv](../../../.temp/brief-626/perf/allocation-comparison.tsv) and [perf/allocation-comparison.json](../../../.temp/brief-626/perf/allocation-comparison.json).


```text
Ordinary raw N/2N allocation and byte equality: 192 / 192
Ineligible: []
Ordinary differences: []
Resume raw exact: 121 / 121
```

**Instructions — FAIL.** Two interleaved N/2N pairs on every label complete 2,504 correct processes. The comparison flags 197 labels (81 ordinary, 116 resume). All 197 receive Callgrind N/2N function-cost breakdowns, with every exclusive-cost sum reconciled to its total. Of those, 187 have above-base Callgrind totals in that pair. The ten non-reproduced labels receive five further interleaved hardware pairs (200 correct processes): eight clear and two persist. Final open hardware flags: **189** (73 ordinary, 116 resume). No positive function cost or negative offset is filtered out. Raw instruction PCs remain in the profiles; function aggregates can move with inlining, so their largest positive rows are not mistaken for the reconciled total delta.

Per-label hardware slopes/ranges and paired deltas: [perf/instruction-comparison.tsv](../../../.temp/brief-626/perf/instruction-comparison.tsv). Five-pair controls: [perf/ordinary-five-run-comparison.tsv](../../../.temp/brief-626/perf/ordinary-five-run-comparison.tsv). Every flagged Callgrind breakdown, full raw-profile paths and total: [instruction-attribution/summary.json](../../../.temp/brief-626/instruction-attribution/summary.json) and [instruction-attribution/](../../../.temp/brief-626/instruction-attribution/). The off-path defects include queue/resumption work and context-guard drop paths; this evidence does not establish zero disabled-path cost.

| Open flag | Engine | Base hardware slope | Candidate hardware slope | Delta | Threshold | Callgrind delta |
|---|---|---:|---:|---:|---:|---:|
| lazy-map-realize-small | cli | 176399.350225 | 176491.487475 | 92.137250 | 88.199675 | 8.796460 |
| lazy-nestlist-realize-small | cli | 142922.160325 | 143122.484925 | 200.324600 | 109.356650 | 191.474820 |
| lazy-drop-realize-small | cli | 188535.748950 | 188661.867175 | 126.118225 | 94.267874 | 79.952830 |
| lazy-first-small | cli | 40606.505950 | 40653.267525 | 46.761575 | 76.768050 | 53.010163 |
| lazy-table-construct-short | cli | 23355.656525 | 23374.222350 | 18.565825 | 49.429000 | 2.839953 |
| lazy-table-realize-small | cli | 115754.275800 | 115839.880775 | 85.604975 | 81.290050 | 194.825581 |
| optionvalue-three-no-input | cli | 9491.736850 | 9499.620225 | 7.883375 | 4.745868 | 27.691500 |
| dot-three-no-input | cli | 249968.492450 | 250352.200300 | 383.707850 | 124.984246 | 344.325000 |
| replaceall-simple | cli | 11317.104850 | 11343.343400 | 26.238550 | 51.796800 | 1.871534 |
| table-small | cli | 14507.647400 | 14523.789900 | 16.142500 | 7.253824 | 2.837446 |
| lazy-depth-map-first-8 | cli | 128212.939000 | 128640.163000 | 427.224000 | 406.934000 | 157.490323 |
| lazy-depth-map-eight-8 | cli | 945288.787000 | 946290.835000 | 1002.048000 | 472.644394 | 759.666667 |
| lazy-depth-take-map-8 | cli | 136840.122000 | 136958.178000 | 118.056000 | 68.420061 | 125.376712 |
| lazy-depth-fold-through-source-8 | cli | 581150.777500 | 581742.520500 | 591.743000 | 290.575389 | -66.176471 |
| lazy-depth-map-first-64 | cli | 934034.299000 | 935890.223500 | 1855.924500 | 4995.746000 | 1001.714286 |
| lazy-depth-map-eight-64 | cli | 7298292.700000 | 7306774.688000 | 8481.988000 | 3649.146350 | 8498.000000 |
| lazy-depth-take-map-64 | cli | 993117.378000 | 994068.532000 | 951.154000 | 496.558689 | 1016.450000 |
| lazy-depth-fold-through-source-64 | cli | 33527681.365000 | 33560596.230000 | 32914.865000 | 16763.840683 | 34161.000000 |
| lazy-depth-nest-map-64 | cli | 944838.692000 | 945874.261000 | 1035.569000 | 4910.854000 | 4236.047619 |
| lazy-depth-map-eight-256 | cli | 29092604.799000 | 29127249.537500 | 34644.738500 | 14546.302400 | -143207.000000 |
| lazy-depth-take-map-256 | cli | 3925976.603500 | 3929864.875500 | 3888.272000 | 1962.988302 | 2785.200000 |
| lazy-depth-fold-through-source-256 | cli | 523335954.883000 | 523845696.751500 | 509741.868500 | 261667.977441 | 539124.000000 |
| downvalue-pattern-hit | cli | 13647.199325 | 13663.233750 | 16.034425 | 6.823600 | 15.279181 |
| module-local | cli | 22433.357800 | 22474.128750 | 40.770950 | 11.216679 | 50.883277 |
| block-local | cli | 14462.994900 | 14478.954125 | 15.959225 | 7.231497 | 16.336469 |
| resume-table | resume | 19892.767000 | 20146.008000 | 253.241000 | 32.098000 | 290.814000 |
| resume-do-chain | resume | 19169.710000 | 19415.598500 | 245.888500 | 83.312000 | 226.142000 |
| resume-call-repeat | resume | 28354.758000 | 28679.261000 | 324.503000 | 68.700000 | 149.483688 |
| resume-cleanup | resume | 18568.495500 | 18882.645500 | 314.150000 | 138.831000 | 269.520000 |
| resume-fold | resume | 29553.058500 | 29998.266000 | 445.207500 | 95.838000 | 552.173077 |
| cases-wide-no-match | cli | 2401015.740312 | 2413786.372813 | 12770.632500 | 1200.507870 | 12752.875000 |
| count-wide-no-match | cli | 2173006.039688 | 2185799.816250 | 12793.776562 | 1086.503020 | 12922.555556 |
| position-wide-no-match | cli | 2574015.913437 | 2586764.211562 | 12748.298125 | 1287.007957 | 12394.285714 |
| deletecases-wide-no-match | cli | 2670887.837187 | 2683745.001562 | 12857.164375 | 1335.443919 | 13141.142857 |
| deletecases-wide-association-no-match | cli | 7501632.882187 | 7527226.295625 | 25593.413437 | 3750.816441 | 20839.000000 |
| appendto-part-outer1000-large | cli | 3589496.430000 | 3603960.855000 | 14464.425000 | 28203.740000 | 4031.200000 |
| resume-argument-twice | resume | 92638.406000 | 93979.528500 | 1341.122500 | 358.673000 | 1277.576744 |
| resume-rewrite-exprs | resume | 53435.792000 | 54085.172000 | 649.380000 | 274.890000 | 573.457219 |
| resume-rewrite-compiled | resume | 41654.272500 | 42430.154000 | 775.881500 | 234.041000 | 570.802083 |
| resume-cases-spec | resume | 69091.889000 | 69924.849000 | 832.960000 | 277.978000 | 935.086505 |
| resume-cases-traversal | resume | 67169.599500 | 67994.162000 | 824.562500 | 89.295000 | 490.171717 |
| resume-position-traversal | resume | 70575.585500 | 71385.745000 | 810.159500 | 165.171000 | 677.522968 |
| resume-count-traversal | resume | 61656.760500 | 62309.566500 | 652.806000 | 462.379000 | 915.515432 |
| resume-cases-shared | resume | 113587.727000 | 115075.763000 | 1488.036000 | 249.752000 | 1753.147727 |
| downvalue-guarded-hit | cli | 27571.510250 | 27589.756750 | 18.246500 | 13.785755 | 15.480000 |
| upvalue-guarded-hit | cli | 33853.291000 | 33871.078000 | 17.787000 | 16.926645 | 19.906780 |
| subvalue-unguarded-hit | cli | 20065.245000 | 20081.682750 | 16.437750 | 10.032623 | 17.876506 |
| default-unguarded-hit | cli | 17192.722250 | 17272.403500 | 79.681250 | 109.517000 | 23.962167 |
| default-unguarded-miss | cli | 12783.762500 | 12807.830250 | 24.067750 | 6.391881 | 22.789003 |
| default-guarded-hit | cli | 22363.361750 | 22388.062250 | 24.700500 | 11.181681 | 23.926174 |
| downvalue-exact-miss-repeat | cli | 774.614000 | 775.814250 | 1.200250 | 1.157000 | 0.654000 |
| upvalue-candidates-tiny | cli | 31358.167000 | 31392.977250 | 34.810250 | 15.679084 | 68.078493 |
| keysort-default-wide | cli | 8258917.025000 | 8272948.925000 | 14031.900000 | 4129.458513 | 6959.500000 |
| keysortby-default-wide | cli | 15447422.975000 | 15461120.650000 | 13697.675000 | 7723.711488 | 4587.000000 |
| sort-symbol-default-width16000 | cli | 23361647.250000 | 23369363.600000 | 7716.350000 | 18321.900000 | 33966.000000 |
| resume-s5-down-rhs | resume | 59502.270000 | 60435.160000 | 932.890000 | 243.240000 | 789.970000 |
| resume-s5-down-guard | resume | 114472.275000 | 115901.895000 | 1429.620000 | 395.330000 | 1495.860000 |
| resume-s5-sub-rhs | resume | 65260.955000 | 66268.400000 | 1007.445000 | 244.500000 | 899.060000 |
| resume-s5-sub-guard | resume | 122374.395000 | 123604.465000 | 1230.070000 | 177.790000 | 1345.870000 |
| resume-s5-up-rhs | resume | 64521.035000 | 65443.640000 | 922.605000 | 62.010000 | 653.130000 |
| resume-s5-up-guard | resume | 137982.350000 | 139398.420000 | 1416.070000 | 68.991175 | 1253.440000 |
| resume-s5-default-rhs | resume | 109202.655000 | 110245.030000 | 1042.375000 | 360.920000 | 797.290000 |
| resume-s5-default-guard | resume | 103324.035000 | 104750.500000 | 1426.465000 | 355.140000 | 1078.700000 |
| resume-s5-default-fallback-rhs | resume | 64371.855000 | 65257.175000 | 885.320000 | 32.185928 | 929.090000 |
| resume-s5-set | resume | 56451.245000 | 57363.215000 | 911.970000 | 144.910000 | 826.020000 |
| resume-s5-upset | resume | 67347.300000 | 68005.570000 | 658.270000 | 228.140000 | 615.470000 |
| resume-s5-tagset | resume | 67027.265000 | 67820.190000 | 792.925000 | 116.030000 | 1056.490000 |
| resume-s5-rule-rhs | resume | 71404.510000 | 72513.860000 | 1109.350000 | 314.780000 | 1129.430000 |
| resume-s5-rule-lhs | resume | 90380.990000 | 91554.610000 | 1173.620000 | 163.620000 | 1120.370000 |
| resume-s5-destructure | resume | 76182.990000 | 77228.065000 | 1045.075000 | 127.250000 | 1300.780000 |
| resume-s5-lhs-twice | resume | 98143.260000 | 99912.430000 | 1769.170000 | 172.120000 | 1101.100000 |
| resume-s5-own-rhs | resume | 51610.380000 | 52409.275000 | 798.895000 | 73.200000 | 834.500000 |
| resume-s5-destructure-twice | resume | 118899.400000 | 120613.145000 | 1713.745000 | 115.300000 | 1435.000000 |
| resume-s5-rule-lhs-twice | resume | 122324.035000 | 123704.680000 | 1380.645000 | 100.220000 | 1322.310000 |
| stringreplace-guarded-hit | cli | 21569.079000 | 21618.847000 | 49.768000 | 17.398000 | 40.536000 |
| resume-s6-rhs-single | resume | 74539.555000 | 75440.585000 | 901.030000 | 37.269777 | 1084.070000 |
| resume-s6-rhs-twice | resume | 105795.880000 | 107027.970000 | 1232.090000 | 193.220000 | 1351.520000 |
| resume-s6-cross-type | resume | 125165.520000 | 126666.670000 | 1501.150000 | 144.000000 | 1616.850000 |
| resume-s6-readonly-twice | resume | 215766.520000 | 218206.540000 | 2440.020000 | 759.100000 | 2340.565217 |
| resume-s6-string | resume | 85121.905000 | 86217.070000 | 1095.165000 | 202.560000 | 916.700000 |
| association-part-positional-read-width32000 | cli | 10101.225000 | 10413.425000 | 312.200000 | 431.500000 | 38.875000 |
| appendto-list-incremental-small | cli | 10527.628000 | 10626.950000 | 99.322000 | 108.720000 | 12.976000 |
| association-set-key-incremental-small | cli | 17254.095000 | 17300.775000 | 46.680000 | 70.330000 | 43.250000 |
| resume-s7-list-part | resume | 83977.215000 | 84914.620000 | 937.405000 | 43.590000 | 1068.550000 |
| resume-s7-assoc-part | resume | 85794.515000 | 86642.640000 | 848.125000 | 196.090000 | 950.140000 |
| resume-s7-assoc-call | resume | 75898.225000 | 76779.975000 | 881.750000 | 65.950000 | 776.790000 |
| resume-s7-assoc-call-thrice | resume | 159902.140000 | 161541.485000 | 1639.345000 | 214.450000 | 1761.370000 |
| resume-s7-held-twice | resume | 126448.690000 | 127482.885000 | 1034.195000 | 247.790000 | 1417.850000 |
| resume-s7-held-rhs | resume | 95977.340000 | 97121.225000 | 1143.885000 | 95.040000 | 1152.820000 |
| resume-s7-associate-argument | resume | 79096.745000 | 79976.110000 | 879.365000 | 147.430000 | 992.130000 |
| resume-s7-keydrop-argument | resume | 76006.960000 | 76856.675000 | 849.715000 | 106.270000 | 1009.320000 |
| resume-s7-associate-target | resume | 265091.830000 | 266063.940000 | 972.110000 | 280.260000 | 1121.746667 |
| resume-s7-keydrop-target | resume | 264022.415000 | 265097.300000 | 1074.885000 | 132.011208 | 743.466667 |
| resume-s7-associate-held | resume | 100357.665000 | 101200.140000 | 842.475000 | 157.180000 | 755.590000 |
| resume-s7-keydrop-held | resume | 107687.250000 | 108669.450000 | 982.200000 | 89.480000 | 768.340000 |
| association-set-part-key-investigation | cli | 14587.445000 | 14631.705000 | 44.260000 | 40.510000 | 26.750000 |
| replacerepeated-guarded-small | cli | 371952.211000 | 372218.300000 | 266.089000 | 185.976106 | 301.641509 |
| replaceat-small | cli | 35139.763000 | 35188.315000 | 48.552000 | 36.986000 | 29.384000 |
| memberq-wide-no-match | cli | 2179339.568750 | 2192266.447500 | 12926.878750 | 1089.669784 | 13025.333333 |
| freeq-wide-no-match | cli | 2245026.355000 | 2257792.457500 | 12766.102500 | 1122.513177 | 11990.000000 |
| count-wide-match | cli | 2159866.578750 | 2172599.222500 | 12732.643750 | 1079.933289 | 12490.111111 |
| count-guarded-small | cli | 50059.045000 | 50112.994000 | 53.949000 | 44.070000 | 7.964912 |
| resume-s8-replaceall-node | resume | 109838.865000 | 111071.345000 | 1232.480000 | 277.710000 | 739.480000 |
| resume-s8-replaceall-atom | resume | 97229.140000 | 98246.700000 | 1017.560000 | 48.614570 | 1069.810000 |
| resume-s8-replaceall-twice | resume | 165809.785000 | 167256.830000 | 1447.045000 | 325.330000 | 1113.080000 |
| resume-s8-repeated-guard | resume | 103971.455000 | 104825.000000 | 853.545000 | 153.620000 | 846.580000 |
| resume-s8-replace-decline | resume | 112329.390000 | 113515.660000 | 1186.270000 | 75.000000 | 808.040000 |
| resume-s8-replace-both | resume | 157812.225000 | 159515.265000 | 1703.040000 | 294.530000 | 1767.580000 |
| resume-s8-replaceat-guard | resume | 109390.860000 | 110466.355000 | 1075.495000 | 175.430000 | 1036.620000 |
| resume-s8-count-target | resume | 63038.745000 | 63824.545000 | 785.800000 | 365.010000 | 735.690000 |
| resume-s8-count-pattern | resume | 61602.940000 | 62439.865000 | 836.925000 | 401.870000 | 785.910000 |
| resume-s8-memberq-target | resume | 63713.690000 | 64701.910000 | 988.220000 | 86.240000 | 1056.930000 |
| resume-s8-memberq-pattern | resume | 62511.500000 | 63386.480000 | 874.980000 | 486.960000 | 865.250000 |
| resume-s8-freeq-target | resume | 65210.640000 | 66067.945000 | 857.305000 | 111.070000 | 775.110000 |
| resume-s8-freeq-pattern | resume | 63782.695000 | 64621.610000 | 838.915000 | 40.820000 | 827.440000 |
| resume-s8-count-level | resume | 78699.190000 | 79946.350000 | 1247.160000 | 571.180000 | 1405.870000 |
| resume-s8-count-guard-twice | resume | 130297.725000 | 131889.690000 | 1591.965000 | 86.270000 | 1515.910000 |
| resume-s8-position-target | resume | 72908.360000 | 73862.390000 | 954.030000 | 36.454180 | 1147.270000 |
| resume-s8-position-pattern | resume | 74391.550000 | 75249.235000 | 857.685000 | 37.195775 | 1106.890000 |
| resume-s8-position-level | resume | 87363.980000 | 88082.975000 | 718.995000 | 210.710000 | 1039.910000 |
| resume-s8-position-guard-twice | resume | 157314.215000 | 158768.690000 | 1454.475000 | 325.050000 | 1122.650000 |
| catch-throw-pattern-hit | cli | 16026.244000 | 16086.331000 | 60.087000 | 27.630000 | 52.212000 |
| checkabort-caught | cli | 14373.307000 | 14409.982000 | 36.675000 | 24.114000 | 190.120000 |
| abortprotect-clean | cli | 9295.587000 | 9305.894000 | 10.307000 | 4.647793 | 10.136000 |
| check-clean | cli | 9565.898000 | 9595.766000 | 29.868000 | 8.732000 | 11.298000 |
| quiet-clean | cli | 9298.064000 | 9309.417000 | 11.353000 | 7.292000 | 19.946000 |
| withcleanup-two-arg | cli | 13319.807000 | 13332.134000 | 12.327000 | 6.659904 | 23.930000 |
| resume-s9-checkabort-option | resume | 64670.785000 | 65626.275000 | 955.490000 | 273.090000 | 802.920000 |
| resume-s9-checkabort-primary | resume | 63196.930000 | 64028.295000 | 831.365000 | 67.160000 | 838.580000 |
| resume-s9-checkabort-abort | resume | 64858.185000 | 65619.135000 | 760.950000 | 102.070000 | 1053.440000 |
| resume-s9-abortprotect-body | resume | 56284.405000 | 57225.665000 | 941.260000 | 51.290000 | 871.070000 |
| resume-s9-checkabort-pending-fallback | resume | 81626.585000 | 82755.000000 | 1128.415000 | 335.540000 | 890.540000 |
| resume-s9-checkabort-fallback | resume | 66428.680000 | 67340.820000 | 912.140000 | 255.400000 | 794.200000 |
| resume-s9-assert-function-lookup | resume | 65996.480000 | 67184.470000 | 1187.990000 | 75.620000 | 982.080000 |
| resume-s9-assert-test | resume | 60790.275000 | 61701.850000 | 911.575000 | 39.480000 | 781.740000 |
| resume-s9-assert-handler | resume | 65576.165000 | 66611.070000 | 1034.905000 | 107.180000 | 1102.320000 |
| resume-s9-check-body | resume | 56381.895000 | 57225.990000 | 844.095000 | 131.320000 | 926.740000 |
| resume-s9-quiet-body | resume | 66355.865000 | 67265.405000 | 909.540000 | 79.610000 | 787.900000 |
| resume-s9-quiet-check-fallback | resume | 85877.630000 | 86889.885000 | 1012.255000 | 45.490000 | 949.260000 |
| resume-s9-cleanup-init | resume | 57639.360000 | 58438.595000 | 799.235000 | 141.900000 | 573.190000 |
| resume-s9-cleanup-primary | resume | 56860.790000 | 57746.830000 | 886.040000 | 28.430395 | 891.810000 |
| resume-s9-cleanup-finalizer | resume | 57317.685000 | 58265.325000 | 947.640000 | 28.658842 | 921.770000 |
| resume-s9-cleanup-three-phase | resume | 115858.225000 | 117925.180000 | 2066.955000 | 94.330000 | 1736.150000 |
| resume-s9-catchexceptions-body | resume | 85686.735000 | 86656.685000 | 969.950000 | 109.750000 | 885.910000 |
| resume-s9-catchexceptions-guard | resume | 88984.030000 | 89950.955000 | 966.925000 | 44.492015 | 897.950000 |
| resume-s9-catchexceptions-match | resume | 94278.620000 | 95112.890000 | 834.270000 | 315.460000 | 805.620000 |
| resume-s9-catchexceptions-suspended-match | resume | 121431.600000 | 122770.705000 | 1339.105000 | 167.760000 | 769.700000 |
| resume-s9-catchexceptions-suspended-guard | resume | 115975.590000 | 117422.690000 | 1447.100000 | 221.480000 | 1596.890000 |
| resume-s9-catchexceptions-rhs | resume | 82084.250000 | 82967.135000 | 882.885000 | 527.360000 | 889.580000 |
| resume-s9-catch-body | resume | 58249.555000 | 59177.170000 | 927.615000 | 85.130000 | 846.330000 |
| resume-s9-catch-form | resume | 63288.945000 | 64084.330000 | 795.385000 | 222.550000 | 1099.310000 |
| resume-s9-catch-form-guard | resume | 91490.245000 | 92447.265000 | 957.020000 | 58.770000 | 981.550000 |
| resume-s9-catch-bypassing-form | resume | 80592.550000 | 81754.245000 | 1161.695000 | 156.890000 | 1105.970000 |
| resume-s9-catch-bypassing-guard | resume | 102054.360000 | 103302.790000 | 1248.430000 | 51.027180 | 1134.050000 |
| catch-form-pattern | cli | 16046.535000 | 16069.632000 | 23.097000 | 19.170000 | 20.480000 |
| checkabort-abort | cli | 14385.940000 | 14418.391000 | 32.451000 | 22.110000 | 115.372000 |
| catchexceptions-handler | cli | 19656.355000 | 19669.702000 | 13.347000 | 9.828178 | 13.392000 |
| function-slot-apply | cli | 10729.166000 | 10757.167000 | 28.001000 | 39.334000 | 42.494000 |
| composition-apply | cli | 27803.281000 | 27835.332000 | 32.051000 | 13.901640 | 42.728000 |
| failure-list-property | cli | 14423.775000 | 14430.694000 | 6.919000 | 11.942000 | 8.008000 |
| failure-missing-property | cli | 16155.879000 | 16184.299000 | 28.420000 | 35.650000 | 43.810000 |
| confirm-failure-message | cli | 51803.141000 | 51872.812000 | 69.671000 | 86.348000 | 38.059585 |
| resume-g15-parameters | resume | 72457.675000 | 73328.000000 | 870.325000 | 36.228838 | 643.600000 |
| resume-g15-template | resume | 72358.240000 | 73130.410000 | 772.170000 | 112.260000 | 954.970000 |
| resume-g15-styled | resume | 83166.440000 | 84113.635000 | 947.195000 | 120.190000 | 1284.090000 |
| resume-g15-raw-failure | resume | 62149.880000 | 63165.355000 | 1015.475000 | 450.170000 | 701.000000 |
| resume-g15-raw-success | resume | 58804.990000 | 59679.300000 | 874.310000 | 147.480000 | 880.930000 |
| resume-g15-projection-value | resume | 97785.010000 | 98460.355000 | 675.345000 | 562.140000 | 1269.480000 |
| resume-g15-enclose | resume | 73271.500000 | 74336.025000 | 1064.525000 | 90.850000 | 802.970000 |
| resume-g15-confirm | resume | 119717.580000 | 120658.940000 | 941.360000 | 421.740000 | 688.830000 |
| lhs-setdelayed-pattern | cli | 21431.777750 | 21470.553250 | 38.775500 | 30.606500 | 67.531618 |
| lhs-arg-cycle | resume | 273990.230000 | 275216.895000 | 1226.665000 | 654.630000 | 1090.625000 |
| lhs-head-cycle | resume | 301541.695000 | 302211.945000 | 670.250000 | 549.290000 | 966.833333 |
| lhs-after-prior-cycle | resume | 319998.520000 | 321372.845000 | 1374.325000 | 1773.220000 | 2005.935484 |
| lhs-after-prior-value | resume | 323359.600000 | 324836.530000 | 1476.930000 | 910.360000 | 1715.442623 |
| lhs-twice-value | resume | 323623.840000 | 325549.490000 | 1925.650000 | 438.440000 | 1513.278689 |
| c1-all-guard | resume | 97186.915000 | 98225.325000 | 1038.410000 | 251.230000 | 1340.510000 |
| c1-all-test | resume | 107875.765000 | 109169.225000 | 1293.460000 | 116.070000 | 1125.340000 |
| c1-all-guard-value | resume | 127558.970000 | 129547.910000 | 1988.940000 | 379.120000 | 1362.650000 |
| c1-all-two-guards | resume | 165792.200000 | 167559.470000 | 1767.270000 | 935.660000 | 1539.910000 |
| c1-unheld-guard | resume | 97042.950000 | 98039.660000 | 996.710000 | 273.420000 | 1198.230000 |
| c1-unheld-test | resume | 107864.425000 | 109070.955000 | 1206.530000 | 478.750000 | 1302.810000 |
| c1-unheld-guard-value | resume | 127607.615000 | 129153.095000 | 1545.480000 | 452.670000 | 1137.560000 |
| c1-unheld-two-guards | resume | 165740.085000 | 167346.710000 | 1606.625000 | 82.870042 | 1071.800000 |
| c1-replaceall-atom | cli | 22637.117000 | 22668.296000 | 31.179000 | 14.278000 | 36.406000 |
| c1-replaceall-unheld-atom | cli | 22585.419000 | 22630.258000 | 44.839000 | 11.292710 | 34.074000 |
| c1-replaceall-rule-rhs-deep-small | cli | 73443.538000 | 73483.996000 | 40.458000 | 36.721769 | 48.158088 |
| c1-replaceall-rewrite-guarded-hit | cli | 43274.346000 | 43321.465000 | 47.119000 | 21.637173 | 37.841991 |
| c1-replaceall-rewrite-guarded-decline-fallback | cli | 66213.709000 | 66254.466000 | 40.757000 | 33.106855 | 50.288079 |


The ten five-pair controls, verbatim comparison output:
```text
Ordinary flags: ['lazy-depth-fold-through-source-8', 'lazy-depth-map-eight-256']
Ineligible: []
Eligible resume rows: 0
```

**Copies — pass.** The brief-622 calibrated ordinary-label census completes 768 checked profiles over all 192 labels. Calibration observes `__memcpy_avx_unaligned_erms` and counts 16,011 / 31,011 / 61,011 at 500 / 1,000 / 2,000, giving equal positive slopes 30 / 30. No ordinary label exceeds base in either copy slope or whole-program copy count. Raw per-label data, symbols and profiles: [copy-census/](../../../.temp/brief-626/copy-census/).


```text
base catchexceptions-handler 500 rc=0 Null
CALIBRATION {"N": 500, "memcpy": 16011, "symbols": ["__memcpy_avx_unaligned_erms", "__memcpy_chk", "memcpy@@GLIBC_2.14"], "artifacts": {"/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-500.out": "ba95f25ab56c7a44541b9520590d0ba8d26402b71e3a2aa64917024c36a0ba54", "/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-500.stdout": "a002524d8279a757abff9705262271831c988b48b06dbee7e143b4179da4abde", "/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-500.stderr": "94f8d414568b715a0eb06cccf1d3cd8bcf239028ed1b222c24c08a821d827a4f"}}
base catchexceptions-handler 1000 rc=0 Null
CALIBRATION {"N": 1000, "memcpy": 31011, "symbols": ["__memcpy_avx_unaligned_erms", "__memcpy_chk", "memcpy@@GLIBC_2.14"], "artifacts": {"/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-1000.out": "2ecbe243932a947071942ded9289868aa22e982a4f921a5d5133f2ba818173d7", "/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-1000.stdout": "a002524d8279a757abff9705262271831c988b48b06dbee7e143b4179da4abde", "/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-1000.stderr": "5b3cbe898dcbe802ba80945d9068e366c17439ac046d6a404516d08fdd998103"}}
base catchexceptions-handler 2000 rc=0 Null
CALIBRATION {"N": 2000, "memcpy": 61011, "symbols": ["__memcpy_avx_unaligned_erms", "__memcpy_chk", "memcpy@@GLIBC_2.14"], "artifacts": {"/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-2000.out": "4be2559fd7136443010b050f55b9c6c8ddaedba4212c7874b34bfa3f136d33fe", "/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-2000.stdout": "a002524d8279a757abff9705262271831c988b48b06dbee7e143b4179da4abde", "/home/remi/dev/rum/.temp/brief-626/copy-census/calibration/base-catchexceptions-handler-2000.stderr": "ecff9721ee068022bd3bdda7c5c69281ef81e0a5a3bb191433f162147391cdaa"}}
PASS symbolized memcpy slopes 30.0 30.0
```


```text
labels 192, copy count above base: 0
ABOVE: 
WHOLE-PROGRAM copy counts above base: []
```

**Stack shapes — pass.** All 22 shapes in debug and release match base exactly: 44/44 fits, all zero residuals, 528 candidate invocations. The supplied base probe and expected slopes remain unchanged. Full per-height/repetition receipts pin both test-binary hashes.


```text
debug/peel 19024.0 19024.0 0.0 0.0 PASS
debug/plus 34912.0 34912.0 0.0 0.0 PASS
debug/sub 29376.0 29376.0 0.0 0.0 PASS
debug/up 28064.0 28064.0 0.0 0.0 PASS
debug/guarded 35840.0 35840.0 0.0 0.0 PASS
debug/rhs-guard 49152.0 49152.0 0.0 0.0 PASS
debug/replace-guard 64880.0 64880.0 0.0 0.0 PASS
debug/replaceall-guard 63216.0 63216.0 0.0 0.0 PASS
debug/count-guard 74064.0 74064.0 0.0 0.0 PASS
debug/position-guard 85648.0 85648.0 0.0 0.0 PASS
debug/catch-body 50528.0 50528.0 0.0 0.0 PASS
debug/checkabort-body 52720.0 52720.0 0.0 0.0 PASS
debug/abortprotect-body 49904.0 49904.0 0.0 0.0 PASS
debug/cleanup-body 50880.0 50880.0 0.0 0.0 PASS
debug/check-body 50432.0 50432.0 0.0 0.0 PASS
debug/quiet-body 50304.0 50304.0 0.0 0.0 PASS
debug/catchexceptions-body 49872.0 49872.0 0.0 0.0 PASS
debug/function-body 49312.0 49312.0 0.0 0.0 PASS
debug/status-body 50816.0 50816.0 0.0 0.0 PASS
debug/message-body 50816.0 50816.0 0.0 0.0 PASS
debug/lhs-arg 57376.0 57376.0 0.0 0.0 PASS
debug/replaceall-rhs 52720.0 52720.0 0.0 0.0 PASS
release/peel 7344.0 7344.0 0.0 0.0 PASS
release/plus 13792.0 13792.0 0.0 0.0 PASS
release/sub 13168.0 13168.0 0.0 0.0 PASS
release/up 11456.0 11456.0 0.0 0.0 PASS
release/guarded 14240.0 14240.0 0.0 0.0 PASS
release/rhs-guard 17920.0 17920.0 0.0 0.0 PASS
release/replace-guard 26768.0 26768.0 0.0 0.0 PASS
release/replaceall-guard 24640.0 24640.0 0.0 0.0 PASS
release/count-guard 28416.0 28416.0 0.0 0.0 PASS
release/position-guard 33632.0 33632.0 0.0 0.0 PASS
release/catch-body 19280.0 19280.0 0.0 0.0 PASS
release/checkabort-body 20864.0 20864.0 0.0 0.0 PASS
release/abortprotect-body 19280.0 19280.0 0.0 0.0 PASS
release/cleanup-body 19280.0 19280.0 0.0 0.0 PASS
release/check-body 18736.0 18736.0 0.0 0.0 PASS
release/quiet-body 18688.0 18688.0 0.0 0.0 PASS
release/catchexceptions-body 19472.0 19472.0 0.0 0.0 PASS
release/function-body 19312.0 19312.0 0.0 0.0 PASS
release/status-body 20432.0 20432.0 0.0 0.0 PASS
release/message-body 20432.0 20432.0 0.0 0.0 PASS
release/lhs-arg 18048.0 18048.0 0.0 0.0 PASS
release/replaceall-rhs 21488.0 21488.0 0.0 0.0 PASS
STACK CANDIDATE 44 / 44 met; residuals 0
```

**Resumed chains — pass within the measured scope.** All eight application-frame workloads match base exactly. All 16 context64/recs64 profiles have finite helper graphs and zero unknown stack states; all live descendants, call-site stack pointers, tail calls, libc helpers and a separate 128-byte redzone allowance are retained. Bounds start at the actual erased typed-adapter entry, rather than the outer host driver. Full chains and every helper: [resume-chains/](../../../.temp/brief-626/resume-chains/).


```text
c1-all-guard 6640 -> 6640 delta 0 PASS
c1-all-test 6640 -> 6640 delta 0 PASS
c1-all-guard-value 14128 -> 14128 delta 0 PASS
c1-all-two-guards 11824 -> 11824 delta 0 PASS
c1-unheld-guard 6640 -> 6640 delta 0 PASS
c1-unheld-test 6640 -> 6640 delta 0 PASS
c1-unheld-guard-value 14128 -> 14128 delta 0 PASS
c1-unheld-two-guards 11824 -> 11824 delta 0 PASS
CHAIN BARS 8 /8; comparable prologue increases 22
```

**Emitted prologues — growth remains.** All 167 emitted comparison rows retain concrete addresses, full assembly, stack operands and memcpy/memmove call contexts; duplicate demangled names keep separate ordinals. 22 of 46 matched demangled-name/ordinal prologue rows grow. Ordinals preserve duplicates; they are not independent cross-build type-identity proofs for generic specializations. New and inlined/absent functions are listed explicitly, not treated as zero-sized replacements. The equality of the 44 shape slopes and eight adapter-root chains does not erase this caller/helper growth. Complete stores and specialization coverage: [stack-evidence-final/prologue-comparison.tsv](../../../.temp/brief-626/stack-evidence-final/prologue-comparison.tsv) and [stack-evidence-final/slots/](../../../.temp/brief-626/stack-evidence-final/slots/).

| Profile | Function / specialization | Base bytes | Candidate bytes |
|---|---|---:|---:|
| debug | `rum_runtime::runtime::suspension::core::<impl rum_runtime::runtime::model::session::Session>::begin_input_suspension` #1 | 232 | 296 |
| debug | `rum_runtime::runtime::suspension::core::abandon::<impl rum_runtime::runtime::model::session::Session>::drain_abandonment_worklist` #1 | 1432 | 1688 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::append_mixed` #1 | 136 | 216 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::capacity` #1 | 40 | 56 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::front` #1 | 56 | 72 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::is_empty` #1 | 40 | 56 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::pop_front` #1 | 88 | 104 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::pop_front_legacy` #1 | 120 | 296 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::projected_growth` #1 | 264 | 280 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::push_back` #1 | 200 | 328 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::push_back_typed` #1 | 168 | 280 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::push_evaluation_return` #1 | 104 | 200 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::push_front` #1 | 200 | 328 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::push_front_typed` #1 | 168 | 280 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::storage_bytes` #1 | 40 | 56 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::visit` #1 | 264 | 392 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::visit` #2 | 280 | 392 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::visit` #3 | 264 | 392 |
| debug | `rum_runtime::runtime::suspension::core::queue::FrameQueue::visit` #4 | 264 | 392 |
| debug | `rum_runtime::runtime::suspension::typed::entry::Entry::abandon` #1 | 88 | 136 |
| release | `rum_runtime::runtime::suspension::core::abandon::<impl rum_runtime::runtime::model::session::Session>::drain_abandonment_worklist` #1 | 808 | 1208 |
| release | `rum_runtime::runtime::suspension::core::queue::FrameQueue::append_mixed` #1 | 184 | 200 |


**Layouts — pass.** All 789 recorded production layouts are unchanged, with no missing type and no lookup failure. All eight compiler-asserted headers match: Entry 24, EntryRef 16, queued mixed entry 40, queued legacy entry 32, FrameQueue 40, ResumeContext 16, SuspendedEvaluation 80, Session 2,032 bytes. All 96 recorded ContinueToken instances are zero-sized. GDB selects an opaque Entry declaration with raw size 0; the actual 24-byte size comes from compiler assertions and unit tests in both arms, not from treating that declaration as a zero-sized type. Full before/after population: [layout-candidate/comparison.json](../../../.temp/brief-626/layout-candidate/comparison.json); compiler headers: [layout-candidate/verified-headers.json](../../../.temp/brief-626/layout-candidate/verified-headers.json).


```text
PRODUCTION LAYOUTS 789 / 789 unchanged; missing=0; changed=0
CONTINUE TOKEN zero-size base/candidate 96
```

**Timing — FAIL on ten labels.** The initial four interleaved panels cover all 120 labels and show 25 disjoint-above-base ranges. Initial whole-program calibration is red on base `subvalue-guarded-hit` (0.4934675744805838×) and `default-unguarded-hit` (0.4787391535462489×): the first base panel is unusually slow, while the later base panel agrees with the probe. All four original panels and the initial calibration remain in [timing/initial-panels/](../../../.temp/brief-626/timing/initial-panels/) and [timing/aligned-runs/calibration-initial.json](../../../.temp/brief-626/timing/aligned-runs/calibration-initial.json).

The entire four-panel reference set was repeated, using the same binaries and byte-identical benchmark body, rather than excluding labels or altering thresholds. The repeated panels show 34 disjoint-above ranges. The existing first aligned pair was retained. All 240 calibration cells now lie within 0.5–2× (0.694940490–1.976632403), including the inherited exceptional association label; that exception is unused. The aligned probe completes ten interleaved pairs on all 120 labels, using only the global `rumMicroReps = 2` → `25` change. Each statement body is unchanged. Timing comes from in-kernel monotonic AbsoluteTiming; no wall-clock differencing is used for performance decisions and no compiler or other heavy job runs concurrently.

Ten aligned labels have candidate ranges wholly above base. All medians, ranges and individual runs are retained in [timing/aligned-runs/comparison.json](../../../.temp/brief-626/timing/aligned-runs/comparison.json); all initial and repeated per-label panel `timings.tsv` files are retained. The following table includes every aligned label, so same-tier neighbors remain visible.

| Label | Base median seconds | Candidate median seconds | Candidate/base | Candidate min > base max |
|---|---:|---:|---:|---|
| association-part-key-read-width4000 | 0.002480229 | 0.002467467 | 0.994854507 | False |
| association-part-key-read-width32000 | 0.009606633 | 0.0095512255 | 0.994232370 | False |
| association-part-positional-read-width4000 | 3.048e-05 | 3.06155e-05 | 1.004445538 | False |
| association-part-positional-read-width32000 | 3.06025e-05 | 3.06755e-05 | 1.002385426 | False |
| loop-empty | 0.0002582965 | 0.000257479 | 0.996835033 | False |
| do-count-null | 0.000245963 | 0.0002449875 | 0.996033956 | False |
| do-iterator-null | 5.50385e-05 | 5.60775e-05 | 1.018877695 | False |
| table-count-null | 7.7016e-05 | 7.5274e-05 | 0.977381323 | False |
| table-iterator-null | 1.7881e-05 | 1.81305e-05 | 1.013953358 | False |
| table-iterator-symbol | 6.721e-06 | 6.6945e-06 | 0.996057134 | False |
| literal-integer | 8.8616e-05 | 8.97685e-05 | 1.013005552 | False |
| literal-string | 9.29575e-05 | 9.28385e-05 | 0.998719845 | False |
| symbol-ownvalue-read | 0.000217801 | 0.000219981 | 1.010009137 | False |
| symbol-ownvalue-guarded-read | 0.000180678 | 0.0001810315 | 1.001956519 | False |
| set-ownvalue | 0.000264872 | 0.000259868 | 0.981107856 | False |
| list-literal-small | 5.48305e-05 | 5.4674e-05 | 0.997145749 | False |
| head-list | 0.000913011 | 0.00091528 | 1.002485184 | False |
| length-list | 0.000907118 | 0.0009129665 | 1.006447342 | False |
| part-list | 0.001076144 | 0.001080408 | 1.003962295 | False |
| sameq-integer | 0.0006292285 | 0.000620177 | 0.985614924 | False |
| sameq-list | 0.0006147245 | 0.0006105815 | 0.993260396 | False |
| if-true | 0.000697858 | 0.000700568 | 1.003883312 | False |
| module-local | 0.0007139445 | 0.0007183905 | 1.006227375 | False |
| module-chain-escape-small | 4.70485e-05 | 4.71955e-05 | 1.003124435 | False |
| module-chain-escape-large | 9.7116e-05 | 9.78725e-05 | 1.007789654 | False |
| block-local | 0.000458964 | 0.0004654205 | 1.014067552 | False |
| block-symbol-only-repeat | 0.0003055455 | 0.000304997 | 0.998204850 | False |
| block-init-null-repeat | 0.0003505885 | 0.000349571 | 0.997097737 | False |
| block-init-read-repeat | 0.0004003965 | 0.000407318 | 1.017286615 | False |
| with-local | 0.000468464 | 0.0004710885 | 1.005602352 | False |
| with-deep-body-small | 0.001788948 | 0.001803044 | 1.007879491 | False |
| table-small | 0.0001831055 | 0.0001845745 | 1.008022697 | False |
| table-iterator-plus | 3.71365e-05 | 3.71e-05 | 0.999017139 | False |
| table-iterator-pair-list | 6.42475e-05 | 6.4585e-05 | 1.005253123 | False |
| do-small | 0.000882105 | 0.000884311 | 1.002500836 | False |
| d-nested-sin-small | 0.005298415 | 0.005326345 | 1.005271388 | False |
| downvalue-exact-hit | 0.0007448945 | 0.000759645 | 1.019802133 | True |
| downvalue-exact-miss | 0.00054265 | 0.0005365415 | 0.988743205 | False |
| downvalue-pattern-hit | 0.0009201515 | 0.0009393345 | 1.020847654 | True |
| downvalue-pattern-miss | 0.0002786035 | 0.000275335 | 0.988268274 | False |
| downvalue-exact-miss-repeat | 0.0012176285 | 0.001208367 | 0.992393821 | False |
| downvalue-pattern-miss-repeat | 0.00126558 | 0.001257894 | 0.993926895 | False |
| downvalue-guarded-hit | 0.001844346 | 0.0018867695 | 1.023001920 | True |
| downvalue-guarded-decline-fallback | 0.001376601 | 0.0013979085 | 1.015478341 | True |
| upvalue-guarded-hit | 0.0023059965 | 0.0023373765 | 1.013608000 | True |
| subvalue-unguarded-hit | 0.001362717 | 0.001395996 | 1.024421065 | True |
| subvalue-unguarded-miss | 0.0007757505 | 0.0007759475 | 1.000253948 | False |
| subvalue-guarded-hit | 0.0024052725 | 0.00242639 | 1.008779670 | False |
| default-unguarded-hit | 0.0012596555 | 0.001275256 | 1.012384735 | True |
| default-unguarded-miss | 0.001044995 | 0.0010682465 | 1.022250346 | True |
| default-guarded-hit | 0.0016372335 | 0.001628945 | 0.994937497 | False |
| matchq-integer-success | 0.000286532 | 0.000284754 | 0.993794759 | False |
| matchq-integer-fail | 0.000291685 | 0.00029033 | 0.995354578 | False |
| matcher-single-tiny | 0.000674582 | 0.0006774045 | 1.004184073 | False |
| matcher-single-wide-small | 4.08195e-05 | 3.92985e-05 | 0.962738397 | False |
| matcher-single-wide-large | 8.08295e-05 | 7.74735e-05 | 0.958480505 | False |
| replaceall-simple | 0.00035609 | 0.0003546635 | 0.995993990 | False |
| rule-rhs-deep-small | 0.0029541625 | 0.0029810095 | 1.009087855 | False |
| rewrite-guarded-hit | 0.0017656965 | 0.0017929885 | 1.015456790 | False |
| rewrite-guarded-decline-fallback | 0.0025137095 | 0.0025555595 | 1.016648702 | False |
| cases-wide-no-match | 0.0001462765 | 0.000144679 | 0.989078902 | False |
| count-wide-no-match | 0.00013858 | 0.000136999 | 0.988591427 | False |
| position-wide-no-match | 0.0001732495 | 0.0001699235 | 0.980802253 | False |
| deletecases-wide-no-match | 0.0001683015 | 0.0001678045 | 0.997046966 | False |
| deletecases-wide-association-no-match | 0.000423455 | 0.0004222565 | 0.997169711 | False |
| deletecases-leaf-delete-small | 0.002092608 | 0.0021888395 | 1.045986396 | True |
| replace-wide-no-match | 7.84475e-05 | 7.81665e-05 | 0.996417987 | False |
| replace-leaf-rebuild-small | 0.002552982 | 0.002668751 | 1.045346579 | True |
| take-multispec-small | 0.0016129405 | 0.0016300205 | 1.010589355 | False |
| replacepart-deep-path-small | 0.0013146885 | 0.0013153575 | 1.000508866 | False |
| replacepart-wide-rule-list | 0.0002864525 | 0.0002905145 | 1.014180361 | False |
| replacepart-guarded-hit | 0.0005297865 | 0.0005338665 | 1.007701215 | False |
| replacepart-guarded-decline-fallback | 0.0005826505 | 0.0005888915 | 1.010711396 | False |
| string-repeat-small | 0.000182712 | 0.0001828005 | 1.000484369 | False |
| character-range-small | 0.0001291815 | 0.000131907 | 1.021098222 | False |
| stringtrim-literal-list-4000 | 0.0006559005 | 0.000656437 | 1.000817959 | False |
| stringtrim-literal-list-32000 | 0.0050409675 | 0.0050658015 | 1.004926435 | False |
| stringtrim-single-literal-4000 | 0.000639653 | 0.000640269 | 1.000963022 | False |
| stringtrim-single-literal-32000 | 0.0050517055 | 0.005048762 | 0.999417325 | False |
| stringtrim-whitespace-4000 | 0.0006308905 | 0.000627847 | 0.995175866 | False |
| stringtrim-whitespace-32000 | 0.005015351 | 0.0050238115 | 1.001686921 | False |
| stringreplace-single-literal-unguarded | 0.0003802405 | 0.0003797375 | 0.998677153 | False |
| stringreplace-pattern-unguarded | 0.0010406795 | 0.00105561 | 1.014346876 | False |
| stringreplace-guarded-hit | 0.000695858 | 0.0006932985 | 0.996321807 | False |
| stringreplace-guarded-decline-fallback | 0.0008270265 | 0.0008239885 | 0.996326599 | False |
| appendto-list-incremental-small | 0.000535372 | 0.000525936 | 0.982374872 | False |
| appendto-list-incremental-large | 0.001571095 | 0.00155648 | 0.990697571 | False |
| appendto-part-outer1000-small | 0.002250241 | 0.002213587 | 0.983711078 | False |
| appendto-part-outer1000-large | 0.004461393 | 0.0044633495 | 1.000438540 | False |
| association-set-key-incremental-small | 0.0014480405 | 0.0014253225 | 0.984311212 | False |
| association-set-key-incremental-large | 0.005477211 | 0.0054653765 | 0.997839320 | False |
| association-small-literal-repeat | 0.0007217115 | 0.0007238115 | 1.002909750 | False |
| merge-nested-small | 0.0010838215 | 0.001101703 | 1.016498565 | False |
| set-part-fixed100-len100 | 0.0001412875 | 0.0001389005 | 0.983105370 | False |
| set-part-fixed100-len800 | 0.000597521 | 0.000573988 | 0.960615610 | False |
| set-part-fill-small | 0.00043191 | 0.0004247275 | 0.983370378 | False |
| set-part-fill-large | 0.001377379 | 0.0013381955 | 0.971552129 | False |
| upvalue-candidates-tiny | 0.001907786 | 0.001916785 | 1.004716986 | False |
| upvalue-candidates-wide-small | 0.0001473915 | 0.0001460245 | 0.990725381 | False |
| upvalue-candidates-wide-large | 0.000292852 | 0.0002903115 | 0.991324970 | False |
| sort-callback-small | 0.0001613605 | 0.0001639715 | 1.016181160 | False |
| sort-callback-large | 0.0003709745 | 0.0003755335 | 1.012289254 | False |
| sortby-callback-small | 0.000191412 | 0.0001934565 | 1.010681149 | False |
| sortby-callback-large | 0.0004309295 | 0.0004360365 | 1.011851126 | False |
| reversesort-callback-small | 0.0001572025 | 0.000158554 | 1.008597192 | False |
| reversesort-callback-large | 0.0003546695 | 0.000355624 | 1.002691238 | False |
| keysort-callback-small | 8.2093e-05 | 8.359e-05 | 1.018235416 | False |
| keysort-callback-large | 0.0001807105 | 0.000184524 | 1.021102814 | False |
| keysortby-callback-small | 4.4435e-05 | 4.46125e-05 | 1.003994599 | False |
| keysortby-callback-large | 9.6845e-05 | 9.73895e-05 | 1.005622386 | False |
| sort-default-wide | 1.68475e-05 | 1.68515e-05 | 1.000237424 | False |
| sort-symbol-default-width4000 | 0.000290246 | 0.0002891165 | 0.996108474 | False |
| sort-symbol-default-width16000 | 0.0012876845 | 0.001295818 | 1.006316376 | False |
| sortby-default-wide | 0.000495275 | 0.0004954025 | 1.000257433 | False |
| reversesort-default-wide | 1.6482e-05 | 1.66005e-05 | 1.007189661 | False |
| keysort-default-wide | 0.000361515 | 0.000359892 | 0.995510560 | False |
| keysortby-default-wide | 0.000868122 | 0.000863175 | 0.994301492 | False |
| valueq-undefined-symbol-definitions-2000 | 7.82e-06 | 7.7835e-06 | 0.995332481 | False |
| valueq-undefined-symbol-definitions-8000 | 7.7865e-06 | 7.8105e-06 | 1.003082258 | False |
| module-retained-gensym-gc-batch | 6.5725e-05 | 6.57205e-05 | 0.999931533 | False |


Aligned completion output, verbatim:
```text
ALIGNED TIMING 10 candidate labels 120
ALIGNED TIMING COMPLETE, disjoint above base: ['downvalue-exact-hit', 'downvalue-pattern-hit', 'downvalue-guarded-hit', 'downvalue-guarded-decline-fallback', 'upvalue-guarded-hit', 'subvalue-unguarded-hit', 'default-unguarded-hit', 'default-unguarded-miss', 'deletecases-leaf-delete-small', 'replace-leaf-rebuild-small']
```

### 7. Stop-loss counters, deviations and open questions

Counters: **19 routes proven; 8750 executed receipts kept; 1624 bars met; 0 consecutive compiling implementation attempts without progress.** The bar count includes the 44 accepted base prerequisites plus the distinct candidate categories below; it is not a claim that the overall gate passes.

| Category | Met | Required | Remaining defects |
|---|---:|---:|---|
| Accepted base stack prerequisites | 44 | 44 | None |
| alloc | 313 | 313 | 0 |
| instructions | 124 | 313 | 189 |
| copy | 192 | 192 | 0 |
| stack | 44 | 44 | 0 |
| layouts | 789 | 789 | 0 |
| chains | 8 | 8 | 0 |
| timing | 110 | 120 | 10 |


The engine is not ready to pass the authoritative gate: 189 instruction flags, ten aligned timing regressions and 22 growing matched prologue rows remain (with generic ordinal-matching limits stated above). All have raw before/after evidence. The separate method-oracle path-normalization issue needs a director scope decision; changing its harness would exceed this brief. No claim was removed, rewritten to evade discovery or marked covered to make the gate green.

All mandatory commands and measurement categories ran to green or retained evidence. Native ZeroMQ execution is the one operationally unavailable requirement: nine explicit skips, to be run natively by the director. No required label or phase-counter receipt was silently omitted. No further phase adoption was attempted. The source is frozen at the measured candidate and no half-edited production file remains.

The only measurement orchestration error was a bookkeeping helper started before the copy-comparison parser finished. It failed with FileNotFoundError; resume attribution also overlapped the tail of that deterministic parser. There was no hardware-counter or timing process in that overlap. The comparison subsequently completed successfully, the helper was rerun after attribution, and all later dependent stages explicitly waited. The error and correction are retained in [logs/copy-bookkeeping-order-error.log](../../../.temp/brief-626/logs/copy-bookkeeping-order-error.log). The initial timing calibration red and full repeated reference set are also retained, with no threshold change or newly excluded row.

Own scratch is under `.temp/brief-626/`. Mandated pre-existing gates additionally write their fixed `target/brief-*` and `.temp/brief-*` artifact directories; those automatic paths were not repurposed as new scratch. The LP01 and canonical shell-gate artifacts were specifically backed up and restored. The seven-section report, all raw logs, hashes, per-label tables and exact next state are retained here.

No git command or network operation was run, and no new unsafe was introduced. Manifests, lockfile, Expr/Symbol, public host types, all existing frame fields, AGENTS.md, TODO.md and other briefs remain unchanged. The tree is left uncommitted. The final scope/hash/unsafe audit is [final-audit.json](../../../.temp/brief-626/final-audit.json) and [continuation-scope-audit.json](../../../.temp/brief-626/continuation-scope-audit.json). Large existing core/queue files retain their responsibilities; new diagnostics, context/budget/segment code and insertion APIs are split into logical modules.
