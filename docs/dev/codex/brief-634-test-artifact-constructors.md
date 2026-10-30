# brief-634 — Test artifacts come from constructors: the typed-slot scratch tree and the compiled protocol library

**Shape: implementation.** Worker: kimi. The first of brief-633's three
implementation steps (its design gate is recorded at the end of
`brief-633-test-isolation-design.md`).

## Goal

Two pairs of tests stop colliding on a file that both of them write or read.
After this brief, no test can bypass the constructor that owns the artifact.

1. **Typed-slot forced-red tree.**
   - The tests: `removing_a_frame_in_a_scratch_copy_is_red` and
     `removing_a_test_guard_in_a_scratch_copy_is_red`.
   - Both write the workspace copy to one fixed tree,
     `.temp/typed-slot-census/forced-red/tree`
     (`crates/rum-stdlib/tests/typed_slot_census.rs:111`), and the ignored
     probe reads that tree back (`:199`).
   - After this brief, each call owns a private tree made by
     `PrivateScratch::create()`, and the tree is removed when its owner
     drops.
2. **Compiled protocol library.**
   - The tests: `retention_compile_controls_precede_pinned_failures`
     (`crates/rum-runtime/src/runtime/accounting_mode/tests/compile.rs:18-37`)
     and `option_reentry_compile_pass_precedes_pinned_compile_fail`
     (`crates/rum-runtime/src/runtime/options/tests/compile.rs:5-15`).
   - The first runs `cargo build --lib` into the test binary's own target
     directory. The second picks the newest `librum_runtime-*.rlib` from that
     same `deps` directory.
   - After this brief, both take exact artifact paths from one constructor,
     `CompiledLibrary::get()`. It builds once per process into a target
     directory that nothing else writes, and nothing scans a directory.

## Context pointers

- Read `AGENTS.md`, then brief-633's director gate (the last section of
  `docs/dev/codex/brief-633-test-isolation-design.md`). The gate states why
  this shape differs from the draft.
- Design evidence, all under `.temp/brief-633/`:
  - the drafts `drafts/brief-private-scratch.md` and
    `drafts/brief-compiled-library.md`;
  - the scratch diffs `logs/scratch/typed-slot-census.diff` and
    `logs/c3/rlib-fix.diff`;
  - the B2 inventory, `inventory/b2-final.tsv`.
- **The draft's two defects, which this shape fixes.**
  - The draft made one directory per process id. In one session, kimi's
    scratch tree accumulated 8 library directories (2.1 GB) and 42 private
    trees (407 MB).
  - The draft still scanned `deps` for `rum_core`.

## Scope

- **MAY modify:**
  - `crates/rum-stdlib/tests/typed_slot_census.rs`;
  - `crates/rum-runtime/src/runtime/tests/mod.rs` (one `mod` line);
  - new `crates/rum-runtime/src/runtime/tests/compiled_library.rs`;
  - `crates/rum-runtime/src/runtime/accounting_mode/tests/compile.rs`;
  - `crates/rum-runtime/src/runtime/options/tests/compile.rs`;
  - the standing inventories, through their replays (below).
- **MUST NOT:**
  - change either compile-proof document (`accounting_mode/protocol.md`,
    `options/reentry.md`), any expected rustc error code, or the
    exactly-one-related-error assertions;
  - change `baseline`'s expected tuples or the marker strings
    `production frames=472 finalizers=55 implementations=626`,
    `G15-C1 PARTITION implementations=625` and `G15-C1 baseline requires`;
  - touch production code, `Cargo.lock`, `scripts/` or `docs/`;
  - add a dependency or `unsafe`.
- **Standing replay (always in scope, `TEMPLATE.md` Scope).** Replay all of
  these. Disclose every changed row:
  - the retention contract fingerprints and both certificate builds;
  - the ownership-graph inventories;
  - LP01;
  - the typed-slot receipts;
  - the rejection census.

  New `UnknownConstruct` rows for code this brief adds are expected. A row
  committed at HEAD that would change identity, or one that points at code
  this brief did not touch, is a STOP.

## Target shape

The shape is fixed. A departure from it is a STOP.

### `PrivateScratch` (in `typed_slot_census.rs`)

```rust
/// Owns one directory for its lifetime. There is no constructor from a path,
/// and dropping it removes the tree.
struct PrivateScratch {
    path: PathBuf,
}

impl PrivateScratch {
    fn create() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = root()
            .join(".temp/typed-slot-census/private")
            .join(format!("{}-{id}", std::process::id()));
        // A leftover tree from a killed run with a recycled pid is removed,
        // never reused.
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for PrivateScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

const SCRATCH_ENV: &str = "RUM_TYPED_SLOT_FORCED_RED_TREE";
```

- `forced_red` creates `let scratch = PrivateScratch::create();` and writes
  the copy under `scratch.path()`.
- It passes the path to the child **through the child's environment**:
  `Command::env(SCRATCH_ENV, scratch.path())` on the `Command` that runs the
  test binary. It does not use a libtest free argument, and it does not call
  `std::env::set_var` (process-global).
- `scratch` lives until after the child exits and the log is written. Its
  `Drop` then removes the tree, on panic as well as on success.
- `forced_red_probe` reads `std::env::var(SCRATCH_ENV)`. It panics with
  `forced_red_probe needs {SCRATCH_ENV}` when the variable is absent, and it
  asserts that the path lies under `.temp/typed-slot-census/private/`. No
  default path exists.
- The log keeps its name, `.temp/typed-slot-census/forced-red/{feature}-{kind}.log`.
  `forced_red` creates that parent directory itself
  (`create_dir_all(log_path.parent())`), because no tree creates it any more.
- Leave the witness falsifiers (`forced_red_witness`,
  `.temp/typed-slot-census/forced-red/{kind}`) unchanged. Each of those
  directories has one writer.

### `CompiledLibrary` (new `runtime/tests/compiled_library.rs`)

```rust
static COMPILED: OnceLock<CompiledLibrary> = OnceLock::new();

pub(in crate::runtime) struct CompiledLibrary {
    deps: PathBuf,
    runtime: PathBuf,
    core: PathBuf,
    build_stdout: Vec<u8>,
    build_stderr: Vec<u8>,
}

impl CompiledLibrary {
    /// One fresh build per test process. No caller supplies a path.
    pub(in crate::runtime) fn get() -> &'static Self {
        COMPILED.get_or_init(Self::build)
    }
    pub(in crate::runtime) fn deps(&self) -> &Path;
    pub(in crate::runtime) fn runtime(&self) -> &Path;
    pub(in crate::runtime) fn core(&self) -> &Path;
    pub(in crate::runtime) fn build_stdout(&self) -> &[u8];
    pub(in crate::runtime) fn build_stderr(&self) -> &[u8];

    fn build() -> Self {
        // target = <workspace>/target/test-compile-library (one fixed
        // directory, written only by this constructor).
        // python3 scripts/run-bounded.py --timeout <measured bound> --
        //   cargo build --offline -p rum-core -p rum-runtime --lib --target-dir <target>
        // Assert success. Then:
        //   runtime = <target>/debug/librum_runtime.rlib
        //   core    = <target>/debug/librum_core.rlib
        //   deps    = <target>/debug/deps
        // Assert that both files exist. Never read_dir.
    }
}
```

- **Why both packages are roots.** Building `rum-core` and `rum-runtime` as
  roots in one invocation makes cargo uplift both libraries to
  `debug/lib<name>.rlib`. The two `--extern` paths are then exact, and the
  build unifies `rum-core` into one unit. Nothing scans `deps` for a hashed
  name: `fn library(deps, name)` is deleted, not moved.
- **Why one fixed directory is safe across processes.** Cargo locks a target
  directory's build. A second test process that builds the same sources
  finds every unit fresh and writes nothing. This replaces the draft's
  one-directory-per-pid shape, which grew without bound.
- **The accounting test** writes `library-build.stdout` and
  `library-build.stderr` from the constructor's accessors, in its own
  scratch directory, exactly as today.
- **Both tests** use only `library.deps()`, `library.runtime()` and
  `library.core()` for `-L dependency=` and `--extern`. The accounting test
  passes `--extern rum_runtime=` only, as it does today. The options test
  passes `rum_runtime` and `rum_core`.
- **The bound** inside `build()` comes from a measurement. Time a cold build
  of that command in a fresh `target/test-compile-library` while the
  rum-runtime lib test runs at `--test-threads=32`, and use at least three
  times the measured wall time. State the measurement in the report. The
  current 180 s holds only if it covers three times the measurement.

## Must-not-change beside the cost bars

- **Freshness.** Every control still compiles and every pinned failure keeps
  its error code and its exactly-one-related-error assertion. A fresh
  `cargo build` precedes every proof in every process, so a stale artifact
  cannot certify a rejection.
- **Disk.** The fixed directory is about one rum-runtime debug build. Repeated
  runs do not add to it. A typed forced-red run leaves no tree behind.
- **Forced-red results.** Every forced-red probe still fails with the same
  marker, `baseline` is unchanged, and the probe is still `#[ignore]`.

## Falsifiers

The director calibrates these red at HEAD; see the record appended to this
brief before dispatch. You reproduce both arms. Run every test command in
its own scope: `scripts/memory-scope.sh b634-<label>-<n> 14G <command>`. Use
`--offline`, `CARGO_BUILD_JOBS=8`, and pass `--test-threads` explicitly.

1. **Fresh-target library pair.**
   - Run it in a new, empty `CARGO_TARGET_DIR` under `.temp/brief-634/`,
     separately for HEAD and for the change. Bound 1800 s on the first,
     cold run.

     ```
     cargo test -p rum-runtime --lib --offline -- --test-threads=2 --exact \
       runtime::accounting_mode::tests::compile::retention_compile_controls_precede_pinned_failures \
       runtime::options::reentry_tests::option_reentry_compile_pass_precedes_pinned_compile_fail
     ```

   - HEAD is red with `missing compiled library rum_runtime`; the director's
     log is recorded below.
   - The change is green, `2 passed`.
   - Then run 5 warm repeats of the change, each green. Bound 600 s.
   - To get HEAD's red, copy `crates/`, `Cargo.toml`, `Cargo.lock`,
     `rust-toolchain.toml` and `scripts/run-bounded.py` to
     `.temp/brief-634/head/` **before your first edit**, and run it there with
     its own target directory. Never use git to get it.
2. **Typed pair at two threads.**
   - Bound 1200 s each:

     ```
     cargo test -p rum-stdlib --test typed_slot_census --offline -- --test-threads=2 --exact \
       removing_a_test_guard_in_a_scratch_copy_is_red removing_a_frame_in_a_scratch_copy_is_red
     ```

   - Run it 10 times on the change; all 10 must be green.
   - HEAD's red rate is 2/10 (brief-633 run 1 census,
     `.temp/brief-633/logs/census/`). Do not rerun HEAD for this.
   - After each run,
     `ls .temp/typed-slot-census/private/ 2>/dev/null | wc -l` prints `0`.
     Paste the 10 counts.
3. **Probe refusal.** Run `forced_red_probe` directly, with `--ignored` and
   without the variable. It must fail, and its message must contain
   `forced_red_probe needs RUM_TYPED_SLOT_FORCED_RED_TREE`. Paste the line.

## Stages and stops

1. **Shape.**
   - Make the two constructors and the call sites.
   - Run `cargo fmt --check` and `cargo build --tests -p rum-runtime
     -p rum-stdlib --offline` with no new warning.
   - Run falsifiers 1–3.
   - STOP and report if an arm does not reproduce.
2. **Replay and workspace.**
   - Run the standing replays.
   - Then run `cargo test --workspace --no-fail-fast --offline --
     --test-threads=1`, bound 5400 s, in one 14G scope. It must be green
     except the nine native ZeroMQ tests, which fail with EPERM in this
     sandbox; that exclusion is standing and is listed in
     `drafts/brief-class-wide-isolation.md`. Pass each of them as `--skip`.
   - The single test thread is the repository's default discipline for this
     gate, not a fix. The default thread count is the director's
     measurement after brief-636.

## Forbidden

- Seeded rlibs or hardlinks.
- Any `read_dir` over a `deps` directory.
- A caller-supplied target or scratch path.
- One directory per process id for the library.
- A tree left behind after a forced-red run.
- `std::env::set_var`.
- `#[ignore]` on a falsifier, retries, sleeps, raised or guessed bounds,
  `--test-threads=1` as a fix, merged tests, `serial_test` or any new
  dependency.
- Weakened diagnostics or markers.

## Report

- Write `.temp/brief-634/report.json` (`TEMPLATE.md` Report format).
- Every `test result:` line and every verdict line is quoted from its log.
- `changes` lists every file, the replayed inventories included.
- Your final message is the output of `python3 scripts/report.py summary 634`.

## Director calibration before dispatch (2026-10-30)

- **The fresh-target library pair is red at HEAD.**
  - The copy is HEAD's sources at `.temp/director/gate-633/head/`, with an
    empty `CARGO_TARGET_DIR`, at `--test-threads=2`, in its own 14G scope.
  - Log: `.temp/director/gate-633/rlib-head-fresh.log`.

  ```
  missing compiled library rum_runtime
  test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 11.15s
  # memory-scope: unit=gate633-rlib-head max=15032385536 peak=2255757312 oom=0 oom_kill=0 rc=101
  ```

  The options test scans `deps` before any non-test rlib exists. The
  accounting test's build into that same directory comes too late to help.
- **The typed pair is red at HEAD 2/10** (brief-633 run 1 census). It is not
  rerun.
- **The green arms exist only in the draft's shape** (`.temp/brief-633/tree/`,
  one directory per pid). This brief's shape differs, so its green arm is
  the worker's to show.

## report (kimi, DONE)

Implemented constructor-owned typed scratch trees and a once-per-process compiled protocol library with exact rlib paths. Reproduced the HEAD fresh-target red; the changed fresh pair and five warm repeats passed, all ten two-thread typed pairs passed with zero leftover trees, and direct probe refusal matched. The cold build measured 12.482 s under the 32-thread load, retaining the 180 s bound. Replays changed 68 fingerprint cells and four certificate cells only; ownership, LP01, typed receipts and rejection census passed without expectation edits. Workspace: 59 binaries, 4004 passed, 0 failed, 61 ignored, nine standing skips, no OOM kill.

| File | Kind | Lines | Why |
| --- | --- | --- | --- |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | modified | 2-37,142-243 | Add PrivateScratch with Drop cleanup, environment-only probe handoff, private-path assertion, and explicit forced-red log parent creation. |
| `crates/rum-runtime/src/runtime/tests/compiled_library.rs` | added | 1-95 | Own the once-per-process two-package build and exact deps/runtime/core artifact accessors. |
| `crates/rum-runtime/src/runtime/tests/mod.rs` | modified | 128 | Register the compiled-library constructor module. |
| `crates/rum-runtime/src/runtime/accounting_mode/tests/compile.rs` | modified | 1-18,65-68 | Consume the constructor and its captured build output instead of building into the unit-test target. |
| `crates/rum-runtime/src/runtime/options/tests/compile.rs` | modified | 1-17,46-51 | Delete the newest-rlib directory scan and use exact constructed runtime/core paths. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | modified | 1-69 | Replay all 68 retention contract fingerprints from the generated review copy. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | modified | 2-3 | Merge four generated C01/C02 certificate cells from both feature builds. |

| Run | Try | Exit | Tests | Log |
| --- | ---: | ---: | --- | --- |
| library-head-fresh | 1 | 101 | 1/1/0 | `.temp/brief-634/logs/library-head-fresh.log` |
| measure-lib32-load | 1 | 101 | 1210/10/5 | `.temp/brief-634/logs/measure-lib32.log` |
| measure-cold-build | 1 | 0 | - | `.temp/brief-634/logs/measure-cold-build.log` |
| fmt | 1 | 0 | - | `.temp/brief-634/logs/fmt-check.log` |
| build-tests | 1 | 0 | - | `.temp/brief-634/logs/build-tests.log` |
| library-change-fresh | 1 | 0 | 2/0/0 | `.temp/brief-634/logs/library-change-fresh.log` |
| library-warm-1 | 1 | 0 | 2/0/0 | `.temp/brief-634/logs/library-change-warm-1.log` |
| library-warm-2 | 2 | 0 | 2/0/0 | `.temp/brief-634/logs/library-change-warm-2.log` |
| library-warm-3 | 3 | 0 | 2/0/0 | `.temp/brief-634/logs/library-change-warm-3.log` |
| library-warm-4 | 4 | 0 | 2/0/0 | `.temp/brief-634/logs/library-change-warm-4.log` |
| library-warm-5 | 5 | 0 | 2/0/0 | `.temp/brief-634/logs/library-change-warm-5.log` |
| typed-pair-1 | 1 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-1.log` |
| typed-pair-2 | 2 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-2.log` |
| typed-pair-3 | 3 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-3.log` |
| typed-pair-4 | 4 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-4.log` |
| typed-pair-5 | 5 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-5.log` |
| typed-pair-6 | 6 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-6.log` |
| typed-pair-7 | 7 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-7.log` |
| typed-pair-8 | 8 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-8.log` |
| typed-pair-9 | 9 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-9.log` |
| typed-pair-10 | 10 | 0 | 2/0/0 | `.temp/brief-634/logs/typed-pair-10.log` |
| probe-refusal | 1 | 101 | 0/1/0 | `.temp/brief-634/logs/probe-refusal.log` |
| fingerprints-generate | 1 | 0 | 1/0/0 | `.temp/brief-634/logs/fingerprints-generate.log` |
| fingerprints-replay | 1 | 0 | - | `.temp/brief-634/logs/fingerprints-replay.log` |
| certificate-default-earn | 1 | 101 | 0/1/0 | `.temp/brief-634/logs/certificate-default-earn.log` |
| certificate-nodefault-earn | 1 | 101 | 0/1/0 | `.temp/brief-634/logs/certificate-nodefault-earn.log` |
| certificate-merge | 1 | 0 | - | `.temp/brief-634/logs/certificate-replay.log` |
| certificate-default-replay | 2 | 0 | 1/0/0 | `.temp/brief-634/logs/certificate-default-replay.log` |
| certificate-nodefault-replay | 2 | 0 | 1/0/0 | `.temp/brief-634/logs/certificate-nodefault-replay.log` |
| ownership | 1 | 0 | 3/0/0 | `.temp/brief-634/logs/ownership-discovery.log` |
| lp01 | 1 | 0 | 6/0/0 | `.temp/brief-634/logs/lp01.log` |
| typed-receipts | 1 | 0 | 8/2/1 | `.temp/brief-634/logs/typed-receipts.log` |
| rejection-census | 1 | 0 | 1/0/0 | `.temp/brief-634/logs/rejection-census.log` |
| workspace | 1 | 0 | 4004/0/61 | `.temp/brief-634/logs/workspace.log` |
| shape-helper-ordering | 1 | 1 | - | `.temp/brief-634/logs/shape-check-attempt1.log` |
| shape-check | 2 | 0 | - | `.temp/brief-634/logs/shape-check.log` |

| Check | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| head-library-red | fresh HEAD pair is red for the calibrated missing artifact | test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 11.26s | PASS |  |
| change-library-fresh | changed pair is green in a new empty outer target | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 1.98s | PASS |  |
| library-warm-five | five warm changed repeats are green | 5/5 green | PASS |  |
| typed-pair-ten | ten two-thread typed pairs are green and remove every private tree | 10/10 green; 0,0,0,0,0,0,0,0,0,0 | PASS | HEAD red rate 2/10 was calibrated in brief-633 and was not rerun. |
| probe-refusal | direct ignored probe without the variable refuses before discovery | forced_red_probe needs RUM_TYPED_SLOT_FORCED_RED_TREE | PASS |  |
| format-build | format and affected test builds pass without a new warning | both green; same three HEAD dead-code warnings | PASS |  |
| fingerprints | retention fingerprints are regenerated, not hand-edited | FINGERPRINT_REPLAY rows=68 changed=68 non_fingerprint_changes=0 | PASS | Every old/new row is retained in local-contracts-fingerprint.diff. |
| certificates | both feature builds accept the merged generated cells | 2 rows, 4 cells; default and no-default green | PASS | The first earning pass in each build failed only on the expected stale recorded cell; the merged replay passed. |
| ownership | ownership discovery exactly matches the standing inventories | exact; no row edits | PASS |  |
| lp01 | LP01 producer/binding/loss inventories replay | 6/6 green | PASS |  |
| typed-receipts | typed-slot target and forced-red markers remain intact | 8 passed, 1 ignored; markers unchanged | PASS |  |
| rejection-census | production rejection census counts do not move | green; no comparison delta | PASS |  |
| workspace | serial full workspace with exactly nine standing native ZeroMQ skips | 4004 passed; 0 failed; 61 ignored; 59 binaries | PASS |  |
| shape-scope | constructor invariants and exact allowed source delta | 7 files; no deps scan, set_var, old shared tree, or unsafe | PASS | The first report-only helper attempt misordered its own expected path list; attempt 2 corrected the helper, not the sources. |

| Measurement | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| cold-library-build-bound | constructor timeout is at least three times the measured cold build under concurrent rum-runtime lib --test-threads=32 | 12.482 s measured; 3x=37.446 s; retained bound 180 s | PASS | The 32-thread HEAD load had ten already-calibrated profiler failures (brief-635 territory); it supplied the required concurrent test load and was not a green gate. |
| workspace-memory | one 14G scope with no OOM kill | peak=5997154304 oom=0 oom_kill=0 rc=0 | PASS | Serial workspace gate, 1819.332 s wall time. |

### Constructed artifact ownership

`PrivateScratch::create()` constructs a pid+counter tree below `.temp/typed-slot-census/private`, passes it only through `RUM_TYPED_SLOT_FORCED_RED_TREE`, and removes it on `Drop`. The probe has no default path and refuses a missing variable or a path outside the private root. `CompiledLibrary::get()` uses `OnceLock`, builds `rum-core` and `rum-runtime` once per test process into the single fixed `target/test-compile-library`, and returns exact top-level rlib paths plus `debug/deps`. Both compile-proof tests use only those accessors; the old `deps` scanner is deleted.

### Measured constructor bound

The cold two-package command built in a fresh `target/test-compile-library` while the HEAD rum-runtime lib test ran at `--test-threads=32`. Monotonic wrapper output: `COLD_BUILD wall_seconds=12.482 rc=0`. Three times the measurement is 37.446 seconds, so the existing 180-second bound was retained rather than raised or guessed.

### Complete replay disclosure

All 68 data rows in `local-contracts.tsv` changed only their final fingerprint cell; every old/new row is in `.temp/brief-634/local-contracts-fingerprint.diff`. In `promoted.tsv`, C01 and C02 each changed their hash=true and hash=false certificate cells (four cells total); complete old/new rows are in `.temp/brief-634/certificate-replay.diff`, and exact individual cells are in `.temp/brief-634/certificate-diff.json`. Both earning runs had zero rejections and failed only on stale recorded cells; both merged replays passed. Ownership discovery stayed exact at 61 self-reaching types, 79 host payloads and 7,942 UnknownConstruct rows, so no ownership file changed. LP01, typed-slot receipts and the rejection census passed without expectation edits. No row disappeared or changed identity/status/disposition/exclusion.

### Library and typed falsifier arms

HEAD fresh-target: `missing compiled library rum_runtime` and `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 11.26s`. Changed fresh-target: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 1.98s`. Five warm repeats each passed 2/0/0. The ten typed two-thread results were:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 52.48s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 53.89s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 54.27s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 54.31s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 54.52s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 54.63s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 55.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 55.03s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 54.72s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 54.83s
```

### Ten cleanup counts and direct refusal

After each typed pair, `ls .temp/typed-slot-census/private/ 2>/dev/null | wc -l` printed:

```text
0
0
0
0
0
0
0
0
0
0
```

Direct probe output included: `forced_red_probe needs RUM_TYPED_SLOT_FORCED_RED_TREE` and `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s`. The nested failures in the full typed log are the intended forced-red witnesses; the parent target passed 8 tests with 1 ignored.

### Workspace gate

The single 14G serial workspace run used exactly the nine standing native ZeroMQ `--skip` names. It produced 59 `test result: ok` lines, totaling 4,004 passed, 0 failed and 61 ignored. `# memory-scope: unit=b634-workspace-1 max=15032385536 peak=5997154304 oom=0 oom_kill=0 rc=0`; wall time was 1819.332 seconds.

## Director gate (2026-10-30): GREEN

- **Code.** I read the whole diff. Both constructors take the brief's shape
  exactly:
  - `PrivateScratch` hands its path to the child only through
    `Command::env`, and `Drop` removes the tree.
  - `CompiledLibrary::get` builds once per process into the fixed
    `target/test-compile-library`, with both packages as roots, and uses the
    exact uplifted `debug/lib<name>.rlib` paths.
  - `fn library(deps, name)` and its `read_dir` are deleted, not moved.
  - The probe refuses to run without the variable, and it refuses any path
    outside `.temp/typed-slot-census/private/`.
- **Inventories.** `local-contracts.tsv` (68 rows) and `promoted.tsv` (2
  rows) are byte-equal to HEAD once every `fnv1a64:` value is masked. The two
  `compile.rs` files are retention dependency files, so only their
  fingerprints moved.
- **`python3 -I scripts/director-gate.py run 634`**
  (`.temp/director/gate-634/`):
  - every step PASS;
  - `check`: 3 warnings, 0 new against HEAD;
  - census: 1040 rows paired, 0 new, 0 retired;
  - fingerprints: 0 changed on replay;
  - workspace: 59 binaries, 4013 passed, 0 failed, 61 ignored, peak 3.9 GB
    of 14, `oom_kill` 0. The nine ZeroMQ tests ran natively.
- **Falsifiers I reproduced on the change**, serially, each in its own 14G
  scope (`.temp/director/gate-634/repro/`):
  - The library pair, with `target/test-compile-library` deleted first and a
    fresh `CARGO_TARGET_DIR`:
    `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 14.95s`.
    Afterwards the constructor directory holds only `debug` and
    `CACHEDIR.TAG`. HEAD's red arm is in the calibration above.
  - The typed pair at two threads, three runs, each
    `test result: ok. 2 passed; 0 failed; ...; 7 filtered out` and each
    followed by `private_trees=0`. The peak was 4.73 GB.
  - The probe without the variable:
    `forced_red_probe needs RUM_TYPED_SLOT_FORCED_RED_TREE`, then
    `test result: FAILED. 0 passed; 1 failed`.
- **The bound.** The cold build measured 12.482 s while the rum-runtime lib
  ran at 32 threads, so 180 s covers more than three times the measurement.
- **Not rerun:** kimi's ten typed pairs (I ran three) and its five warm
  library repeats.
