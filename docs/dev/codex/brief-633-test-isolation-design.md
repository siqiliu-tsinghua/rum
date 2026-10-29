# brief-633 — Test isolation: the profiler test lease, private scratch paths, one shared parse per process

**Shape: design-only. Scope: NONE — no source change in the repository.**
You re-measure, inventory, decompose and design, and you draft the
implementation briefs. Probes that need a source change run in a scratch
copy of the workspace (below), never in `crates/`.

Worker: grk.

## Goal

The default `cargo test --workspace`, with libtest's parallel runner, is
green on this host. Each family of failures is closed by structure, so the
compiler or the type rejects the next test that would collide, not a
convention.

When you are done, these exist, each backed by `path:line` evidence or a
saved log:

1. A re-census at HEAD of the families brief-627 found.
2. Three class-wide inventories, covering every case and not only the cases
   already seen:
   - process-global state that tests write;
   - fixed filesystem paths that tests write;
   - whole-tree work that tests repeat.
3. A memory decomposition of the largest test peaks, which sums to the
   measured peak.
4. A design for each fix: the profiler test lease, private scratch paths,
   and one shared parse per process for the retention suite and
   `typed_slot_census`.
5. The implementation briefs, drafted in `TEMPLATE.md` shape. Each comes
   with its falsifier, calibrated red at HEAD and green on the scratch copy.

## What stands (decided; do not reopen)

- **The single process-wide epoch collector.** The logical profiler records
  every profiled request into one epoch per process
  (`.temp/brief-624/DESIGN.md:84-90`). Do not split it per thread and do not
  change the owner filter.
- **brief-627 and the director's review of it.** Read
  `docs/dev/codex/brief-627-parallel-test-census.md`, sections 3–6 and the
  "Director review":
  - The profiler and typed-slot families are class (a), and that
    classification is accepted.
  - The lease draft is accepted with two changes:
    - the thread-registration check adds no branch to production
      `lexical_profile_enabled`;
    - a two-session owner-filter contract test is added. If that test is
      red, it is a product defect and a STOP.
  - The 1.2 GB cluster of per-test peaks is a repeated parse. The fix is one
    shared, read-only parse per process, not a thread cap.
- **The Jupyter interrupt family was the stale native stack base.** It was
  fixed in brief-629 (`60ca9585`), and the pair passed 40 of 40 at
  `--test-threads=2`. Your re-census confirms or refutes that; you do not
  redesign it.
- **The default thread count is not part of this brief.** After the fixes
  land, I measure it at 4, 8, 16 and 32 threads and take it to the user. Do
  not draft `.cargo/config.toml`.
- **The testing policy** (`AGENTS.md`): rum's own behaviour, and rum against
  rum.

## What the director found at HEAD `07194ab6`

1. **The profiler test hooks reach past rum-runtime's own `#[cfg(test)]`.**
   - The public hooks on `Session` are compiled under
     `cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))`
     (`crates/rum-runtime/src/runtime/profiling/logical/mod.rs:89-94`).
     `Session::logical_profile_test_reset()` (`:181`) resets the
     process-wide epoch.
   - `rum-stdlib` turns on both features as a dev-dependency
     (`crates/rum-stdlib/Cargo.toml:28`).
     `crates/rum-stdlib/tests/suspension_oracle/` reads
     `LogicalProfileSnapshot`.
   - `crates/rum-host/tests/logical_profile_lifecycle.rs` makes 32 hook calls,
     and `logical_profile_abandonment.rs` makes 3. The lifecycle cases
     re-run themselves as child processes with `RUM_PROFILE_*` set
     (`logical_profile_lifecycle.rs:58-67`, `:106-116`, `:162`).
   - Inside rum-runtime, the guard `enabled()`
     (`crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs:10-23`)
     still holds no lock.

   So the lease must cover every process where two tests can touch the
   epoch, not only rum-runtime's library tests.
2. **The retention model holds syn trees.**
   - `Source` (`crates/rum-stdlib/src/tests/retention_closure/model.rs:136-148`)
     holds `syn::ItemImpl` and `syn::Expr`.
   - proc-macro2 1.0.106 marks its types `!Send + !Sync`. Its marker is
     `struct ProcMacroAutoTraits(PhantomData<Rc<()>>)`, at
     `~/.cargo/registry/src/*/proc-macro2-1.0.106/src/marker.rs:12`.
   - So `static OnceLock<Source>` does not compile, and the shared parse
     needs a real design.
   - `typed_slot_census` has the same problem
     (`crates/rum-stdlib/tests/typed_slot_census/scan.rs:72`,
     `signature_ast: syn::Signature`).
   - Verify all of this with a compile probe before you rely on it.
3. **The retention suite rebuilds the model in every test.**
   - Under `retention_closure/`, 19 files call `production()` 26 times.
   - Each test builds its own `Interfaces`
     (`local/interface.rs:7-17`, which holds a whole `Source`).
   - Mutation tests also build a second `Interfaces` from a changed copy
     (`local/tests.rs:43`, `local/promotion_tests.rs:130`, `:162`).
   - Two process-wide caches already exist, and both hold a `String`:
     `local/certificates.rs:26` and `local/contracts.rs:92`.
4. **`typed_slot_census` re-discovers the tree in each test**
   (`crates/rum-stdlib/tests/typed_slot_census.rs:77`, `:84`, `:109`, `:220`).
   - `forced_red` writes to the fixed path
     `.temp/typed-slot-census/forced-red/tree` (`:111`).
   - The ignored probe reads that same path (`:199`).

## Questions the plan must answer

### A. Re-census at HEAD (findings age)

The previous census ran at `17576575`. brief-629 and brief-630 have landed
since then. Reuse brief-627's instruments (`.temp/brief-627/measure.py`,
`memory_one.py`, `scripts/peak-rss.py`). Copy them into
`.temp/brief-633/`; do not edit them in place.

- The rum-runtime lib at the default thread count, 3 runs.
- The chain pair (brief-627 §5, "Profiler") at `--test-threads=2`, 20 runs.
- The typed-slot pair at `--test-threads=2`, 10 runs.
- The Jupyter interrupt pair at `--test-threads=2`, 20 runs.
- Every other workspace test target at the default thread count, 3 runs
  each.
  - Exception: run the rum-stdlib lib at 8 threads. At 32 threads it was
    OOM-killed inside a 16G scope (brief-627, K3).
  - Record each process's peak memory.
- The nine native ZeroMQ tests fail in your sandbox with EPERM on loopback.
  This is environmental. List them, and keep them out of the counts.

For each family, report whether it still exists, with counts. Do not design
around a family that is gone; record it as gone.

### B. Class-wide inventories, workspace-wide, with `path:line`

1. **Process-global state that a test writes**, in every crate. That covers:
   - `static` atomics, `Mutex` and `OnceLock` values that a test writes or
     resets;
   - environment variables;
   - the current directory;
   - allocator counters;
   - thread-locals that outlive a test.

   For each one, give the tests that write it, the tests that read it, and
   whether two tests in one process can overlap on it. Give one verdict:
   isolated, shared by design and safe, or a collision.
2. **Fixed filesystem paths that a test writes**, under `.temp/`, `target/`
   or anywhere else. For each, give the writers, the readers, and whether two
   tests in one binary can collide on it.
3. **Repeated whole-tree work**: every test that parses the workspace or
   walks the repository. For each, give its peak and the part of the peak
   that comes from the shared input.

### C. The profiler lease

Start from brief-627's draft (`.temp/brief-627/drafts/brief-profiler-test-lease.md`),
revalidated at HEAD, with the review's two changes. Answer each of these:

1. **Where the registration check is compiled.** The choice is between
   `cfg(test)` and the two test features. Prove that no production binary and
   no perf-panel arm compiles it:
   - run `cargo tree -e features` for `rum-cli`;
   - check the build that `scripts/perf-panel.py arms` runs;
   - check `tools/perf-panel/harness/Cargo.toml`'s features.

   When the profiler flag is off, `lexical_profile_enabled` stays one relaxed
   load. Name how the implementation brief proves that: the gate's
   `run NNN --perf resume,cli`, and what else.
2. **Every path that constructs a test epoch or resets one.** List each at
   HEAD with `path:line`, including the public
   `Session::logical_profile_test_reset`. Show how the lease's types make the
   compiler reject the old forms, and how a new LP03–LP08 test cannot get
   around the lease.
3. **The processes the lease covers.** For each process that the
   inventory in B1 finds touching the epoch, give the mechanism that isolates
   it: a lease, a child process, or a reason the overlap is impossible.
4. **The two-session owner-filter contract.** Two sessions run on two
   threads, both registered under one lease and profiled at the same time.
   Each session's owner-filtered receipt must equal its solo receipt.
   - Run it now as a probe on the scratch copy.
   - If it is red, STOP: that is a product defect.
5. **The draft's unregistered-thread test.** Calibrate it against HEAD's
   behaviour. If it is already green at HEAD, say so; the chain pair stays
   the falsifier.

### D. Private scratch paths

Use the draft (`.temp/brief-627/drafts/brief-typed-slot-scratch-dir.md`) for
`forced_red`, plus every collision that B2 finds. Build each so that a caller
cannot share a path: the path comes from a constructor that the test cannot
bypass, not from a naming convention.

### E. One shared parse per process (the retention suite and `typed_slot_census`)

1. **Decompose before designing.** Measure; do not estimate. Cover the ten
   largest retention-test peaks (brief-627 §4) and the nine
   `typed_slot_census` tests. For each, give how much of the peak comes from:
   - `production()`, or `discover`;
   - `Interfaces::new`;
   - the mutant copies;
   - the test body.

   The parts must sum to the measured peak; state the residual. Use probes on
   the scratch copy.
2. **The design.** syn's trees are `!Sync`. Weigh at least these options,
   each with its measured memory at 4, 8, 16 and 32 threads:
   - (i) a `Sync` model: at parse time, convert what the tests read into
     owned `Sync` data (spans become line and column, items become token
     text);
   - (ii) one parse per test thread (`thread_local`): N threads hold N
     parses, so this is not one per process;
   - (iii) the file text shared once per process (`OnceLock` of the text),
     with a re-parse per test, which saves only the reads;
   - (iv) anything better.

   Recommend one.
3. **Mutation tests.** The shared value is immutable (`&'static` or `Arc`).
   A mutant is an explicit owned copy or an overlay, and the compiler rejects
   any change to the shared value in place. Measure what a copy costs.
4. **Equivalence.** Every retention and `typed_slot_census` verdict must be
   unchanged. Say how the implementation brief proves that the shared model
   equals a fresh parse, and that the gate's census steps print the same
   lines:
   - `typed-slot` and the census pairing in `scripts/director-gate.py`;
   - the retention tests' own receipts.
5. **The k largest peaks after the fix.** Predict them, as input to my
   thread-count measurement.

### F. Order and drafts

Draft one implementation brief per gated step in `.temp/brief-633/drafts/`,
in `TEMPLATE.md` shape. Each draft has:

- the target shape as code;
- the must-not-change list beside every cost bar;
- its falsifier, red at HEAD and green on the scratch copy, with both logs;
- its concurrency plan, with bounds taken from your measurements.

The last draft's class-wide falsifier is `cargo test --workspace` at the
default thread count, green across N runs. Any target that still needs a
cap carries that cap, with the reason.

## How to run probes without touching the repository

- Make a scratch copy of the workspace at `.temp/brief-633/tree/`: the root
  `Cargo.toml` and `Cargo.lock`, and `crates/`.
- Build it with its own `CARGO_TARGET_DIR`, under `.temp/brief-633/`.
- Change only the copy. Diff the copy against `crates/` for every draft's
  calibration, and save the diff beside the log.
- Standalone probes, such as the `Sync` compile probe, go in a crate under
  `.temp/brief-633/probe/`. Give it an empty `[workspace]` table, depend by
  path, and give it its own target directory.
- Free disk at dispatch: 470 GB. The repository `target/` is 74 GB, so a
  second debug target directory fits.

## Scope

- **May write:** only under `.temp/brief-633/`.
- **Must not:**
  - touch `crates/`, `scripts/`, `tools/`, `docs/`, `Cargo.lock`, or
    `.cargo/`;
  - add a dependency, including in a draft;
  - leave a probe outside `.temp/brief-633/`.

## Concurrency plan

- **Builds:** one cargo command at a time, with `CARGO_BUILD_JOBS=8`. You run
  inside a 16G memory scope.
- **Test threads are the variable here.** Pass `--test-threads` explicitly on
  every run, and never run two cargo test processes at once.
- **Memory:** the rum-stdlib lib never runs above 8 threads at HEAD. The
  per-test memory rows use brief-627's runner, with at most two test
  processes beside each other inside the 16G scope. That is brief-627's
  narrow job: `--memfree` reads the host's memory, not the scope's.
- **Bounds:** use `scripts/run-bounded.py`, with bounds taken from these
  measurements:
  - the chain pair finished in under 1 s: bound 60 s;
  - the typed-slot pair took about 54 s: bound 1200 s;
  - the rum-stdlib lib at 8 threads had a median of 133 s: bound 1200 s;
  - the rum-stdlib lib single-threaded took 1 276–1 362 s: bound 3000 s;
  - a target with no prior measurement: give it a 1800 s bound on the first
    run, then size the bound from that run.

  If a bound fires, report it with the log tail. Do not raise the bound and
  rerun.

## Stop conditions

- The owner-filter contract probe (C4) is red: STOP. That is a product
  defect.
- A fix would need new `unsafe` in any crate, `unsafe impl Send` or
  `unsafe impl Sync` included: STOP.
- A fix would change the process-wide epoch, the owner filter, or
  `PROCESS_LIMIT`: STOP.
- A fix would change a retention or `typed_slot_census` verdict: STOP.
- A fix needs a lock or a convention where a type could enforce the rule:
  STOP, and put it in `questions`.

## Forbidden in any draft

- These as fixes:
  - `#[ignore]`;
  - retries, sleeps, or raised timeouts;
  - `--test-threads=1` on a target;
  - merging tests into one `#[test]`.
- `serial_test`, or any new dependency.
- Weakening an assertion. A draft that removes or loosens an assertion
  lists it in `disclosures`, with the reason.
- A `Mutex` or any added branch on the path where the profiler flag is off.

## Report

Write the `rum-report/1` file `.temp/brief-633/report.json` (`TEMPLATE.md`,
"Report format"). Use these section tags:

- `census`: A, with counts and logs;
- `inventory`: B1–B3, as tables with `path:line`;
- `lease`: C1–C5;
- `scratch`: D;
- `parse`: E1–E5, with the decomposition table;
- `drafts`: one line for each draft, with its falsifier's red and green
  lines quoted from the logs.

Put open decisions in `questions`, each with options and a recommendation.
Your final message is `python3 scripts/report.py summary 633`.
