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
