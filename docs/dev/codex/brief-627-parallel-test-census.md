# brief-627 — The parallel test census: which tests cannot share a process, why, and the enforced fix

**Shape: design-only. Scope: NONE — no source change.** You measure, diagnose
and plan, and you draft the follow-up implementation briefs. You write no
code in `crates/`.

## Goal

At `17576575` the repository's standard `cargo test` is red. Since
2026-09-30 every gate has run tests single-threaded, so nothing has looked at
the default runner for four weeks.

When you are done, these five things exist, each backed by `path:line`
evidence or a saved log:

1. A census of every workspace test target under parallel threads: which
   tests fail, how often, and at which thread count.
2. The peak memory of each test in the two large library suites.
3. A root cause for each family of failures.
4. A plan that makes the suite green under the default runner, with each
   family's fix enforced by structure rather than by convention, and drafted
   implementation briefs for it.
5. A proposed default test-thread count for this host, with its derivation.

## What the director already measured

- **K1, your calibration.** At HEAD, `cargo test -p rum-runtime --lib profiling`
  at the default thread count fails every time. Over three runs, 16, 13 and 14
  of 30 tests failed (2 ignored). The logs are
  `.temp/director/par-626/profiling-default-{1,2,3}.log`. Single-threaded, all
  30 pass (`.temp/director/gate-626-c13/profiling.log`).
  - The failing tests are all LP02's: `logical/budget.rs`,
    `tests/queue_boundaries.rs`, `tests/slot_boundary.rs` and
    `tests/chain_drop.rs`.
  - A typical message:
    `logical teardown: pending children, retained terminal parents, open records, context leases, faults`,
    `left: (0, 0, 1, 0, 0)`, `right: (0, 0, 0, 0, 0)`.
- **K2.** The `rum-jupyter` `execute_lifecycle` interrupt tests
  (`crates/rum-jupyter/src/tests/execute_lifecycle/interrupt.rs`) failed in 3
  of 10 workspace-parallel runs on the source before LP02. They never failed
  single-threaded. (`TODO.md`, "Opened by LP02's gate".)
- **K3.** `rum-stdlib`'s library tests at the default thread count (32 here)
  reached 14.6 GB of anonymous memory and were OOM-killed inside a 14 GiB
  scope (2026-10-26).
- **K4.** `cargo test --workspace --no-run` at `CARGO_BUILD_JOBS=8` peaked at
  9.0 GiB in its cgroup, page cache included. The target directory was
  partly warm.
- **Cost of the serial rule.** `rum-stdlib`'s library suite took 225–268 s
  for about 1 025 tests in parallel (briefs 550–558), and 1 276–1 362 s for
  about 1 110 tests single-threaded (briefs 620–626).
- **A design decision that stands.** The logical profiler uses one
  process-wide epoch collector (`.temp/brief-624/DESIGN.md:84-90`): "The
  comparison isolates one epoch or filters the native receipt by owner; other
  simultaneously profiled requests are not part of its program tuple." Do not
  propose changing it.
- **The profiler's test guard holds no lock.**
  `crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs:10`
  `enabled()` resets the epoch and swaps two process-global atomics,
  `LEXICAL_PROFILE_ENABLED` and `LEXICAL_PROFILE_ACTIVE`. Its `Drop` resets
  the epoch again and restores them.
- No test calls `std::env::set_var` or `remove_var`, and there is no
  `.cargo/config.toml`.

## Your environment and instruments

- **Memory.** The dispatcher runs you inside a memory-capped scope:
  `MemoryMax` 16G, no swap, `OOMPolicy=continue`. A command that reaches the
  cap is killed on its own with exit 137, and you keep running. Read
  `oom_kill` in `/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)/memory.events`
  before and after each run; a rise means that run was OOM-killed. Do not call
  `systemd-run`, because the sandbox cannot reach the user bus.
- **Host.** 32 CPUs. libtest's default thread count is 32. Cargo is offline:
  export `CARGO_NET_OFFLINE=true`.
- **Instruments.**
  - `scripts/run-bounded.py --timeout S -- <cmd>` gives every command a bound.
    Size it from a measured run of the same shape.
  - `scripts/peak-rss.py <cmd>` reports `maximum resident set size: <bytes>`
    on stderr. It is `ru_maxrss` over the waited-for descendants, so it gives
    the **largest single process**, not a sum. That is right for one test
    process. It is wrong for a build, which runs many rustc processes at once.
  - GNU parallel 20260922 is at `/usr/bin/parallel`.
- **Calibrate before use.** For each instrument whose number you report,
  check it against a known answer before Stage 2.
  - `peak-rss.py` on `python3 -c "b=b'\x01'*(1<<30)"` must read 1.0–1.1 GiB.
  - If you need a process-tree sum (Stage 3 builds, optional), write a
    `/proc` sampler. Four children that each touch 1 GiB must read
    4.0 ± 0.2 GiB.
  - A failed calibration is a STOP for that instrument. Report it, and do not
    use its numbers.
- **You cannot read `.git`.** Leave the archaeology to the director: which
  commit introduced a test is the director's question.

## Stages

Write `.temp/brief-627/PROGRESS.md` after each stage, per `TEMPLATE.md`
§Checkpointing.

**Stage 0 — build and inventory.**
- Run `CARGO_BUILD_JOBS=8 cargo test --workspace --no-run`. Its `Executable`
  lines list 46 test targets.
- Key each target by `(package, kind, name)`, with kind one of lib, bin or
  test. Both `rum-host` and `rum-stdlib` have a `guest_loading_resume` test,
  so the package is part of the key.
- Map each key to its cargo invocation: `--lib`, `--bin <name>` or
  `--test <name>`.
- Run each target through `cargo test -p <pkg> <selector>`, not by executing
  the binary directly. Cargo sets the working directory and environment the
  tests expect. Once Stage 0 has built everything, cargo does not rebuild.

**Stage 1 — calibrate against K1.**
- Run `cargo test -p rum-runtime --lib profiling` at the default thread count
  3 times. Each run must fail at least one test.
- Run it once with `-- --test-threads=1`. Every test must pass.
- If either check does not hold, STOP: your environment is not the
  director's.

**Stage 2 — the failure census.**
- Runs are serial: one target at a time, and nothing else running during
  them. Flake rates depend on load.
- Run every target **10 times at the default thread count** and **5 times at
  `--test-threads=8`**.
- Record each run as one row of `.temp/brief-627/census.json`:
  - the key, the thread count and the run index;
  - the exit status and the `test result:` line;
  - every failing test name, and the first panic message for each;
  - wall time and `peak-rss.py` peak;
  - the `oom_kill` delta.
- An OOM-killed run is a result. Record it, do not retry it at that count,
  and still run the other count.
- For every test that failed at least once, run it alone,
  `-- --exact <name> --test-threads=1`, 10 times. A test that fails alone is
  a serial failure at HEAD. Record it separately; it is not a stop.
- Doctests: run `cargo test --workspace --doc` 3 times at the default thread
  count.
- Write `.temp/brief-627/census.md`. It has one row per target: runs and
  failures at each count, OOMs, median wall, and maximum peak. Below that,
  list each failing test with its failure count per count.

**Stage 3 — per-test memory.**
- Cover the `rum-stdlib` and `rum-runtime` library suites, plus any target
  whose default-count peak exceeded 2 GiB in Stage 2.
- List their tests with `-- --list`.
- Run each test alone, `-- --exact <name> --test-threads=1`, under
  `peak-rss.py`, and record its peak and wall in
  `.temp/brief-627/memory.tsv`.
- Peak memory does not depend on load, so this stage may run in parallel:
  `parallel -j 8 --memfree 4G`. Rerun alone any test whose run was
  OOM-killed.
- Report:
  - the 30 largest peaks;
  - how many tests exceed 256 MiB, 1 GiB and 4 GiB;
  - the sum of the 32 and of the 8 largest peaks.
- Check this against K3, in the report. The sum of the 32 largest peaks is
  an upper bound for what 32 threads hold at once, so it should be at least
  14.6 GB. If it is far below that, the memory is not per test, and you say
  where it is.

**Stage 4 — diagnosis.**
- Group the failing tests into families by shared mechanism.
- For each family, give:
  - the shared state, at `path:line`;
  - why two threads collide on it;
  - a **minimal reproduction**: exactly two named tests run together with
    `--test-threads=2` (or the smallest set that works), repeated N times,
    failing in some runs, while each test passes alone. Give the command and
    the counts, and save the logs.
  - a class:
    - (a) a test-isolation defect over a resource that is process-wide by
      design;
    - (b) a product defect: two sessions on two threads in one process
      interfere in a way the documented contract forbids;
    - (c) a timing assumption that breaks under load;
    - (d) resource exhaustion.
- **For the profiler, decide between (a) and (b) on evidence.** The enable
  flags are process-global. While a profiling test holds them on, an
  unrelated test in the same binary that evaluates code may record into the
  same epoch.
  - Establish whether that happens. For example, run one profiling test
    together with one ordinary evaluation test, and show the foreign
    records.
  - Establish whether two concurrently profiled sessions on two threads each
    get correct owner-filtered results, as the design clause above
    promises. If they do not, that is class (b) and a product defect.
- K2: give the root cause of the interrupt flake.

**Stage 5 — the plan and the drafts.**
- For each family, give the fix and the reason it is the fix.
- Order of preference, from best:
  1. A structure the compiler enforces. One example: the only route that
     turns profiling on returns a guard that holds a process-wide lock.
  2. A test that fails when the convention is broken.
  3. A convention.
- Show that the chosen shape also covers **future** tests. LP03–LP08 will
  add many profiler tests.
- Show what isolates a profiling test from the ordinary tests in the same
  binary, not only from other profiling tests.
- **Forbidden as fixes:**
  - `#[ignore]`;
  - retries, sleeps, or raised timeouts;
  - `--test-threads=1` as the answer for a target;
  - deleting or weakening an assertion;
  - leaving a failure out of the census;
  - a new dependency, `serial_test` included. If you think one is needed,
    argue it under open questions, and the director decides.
- **The proposed default thread count.**
  - Derive it from Stage 3: the sum of the k largest peaks that can run
    together stays at or below 12 GiB, leaving room in the 16G scope for
    page cache and the worker.
  - Derive it from Stage 2 as well: wall time at each count.
  - Say where the default would live. If you propose `.cargo/config.toml`
    `[env]`, prove with a probe that the value reaches the test process.
    Run that probe in a scratch crate under `.temp/brief-627/`, never in the
    repository's own config.
  - It goes to the user as a proposal. You do not install it.
- Draft the follow-up implementation briefs in `TEMPLATE.md` shape under
  `.temp/brief-627/drafts/`, one per family, plus one for the default if you
  propose one. Each draft gives its shape as code, its falsifier (red at
  HEAD under parallel threads, green after), and its concurrency plan.

## Scope

- Files you MAY modify: **NONE** in the repository's source, tests, scripts
  or config. Write only under `.temp/brief-627/`, and append your report to
  this brief.
- MUST NOT touch: everything else, including `Cargo.lock`, `target/`'s
  configuration, `.cargo/`, `TODO.md` and `docs/`, apart from your report.

## Concurrency plan

- **Stage 0:** one cargo build, `CARGO_BUILD_JOBS=8`.
- **Stages 1, 2 and 4:** serial. One cargo command at a time, with nothing
  else running. The thread count under test is the variable, so never put
  two runs on the machine together.
- **Stage 3:** parallel, `-j 8 --memfree 4G`, because peak memory is
  load-insensitive. Its wall times are not comparable with Stage 2's, so do
  not quote them as timing.
- Analysis and drafting happen while nothing is being measured.
- Give every command a bound with `run-bounded.py`. For the large suites at
  one thread, allow 2 400 s.

## Stop conditions

- Stage 1's calibration does not reproduce: STOP.
- An instrument's calibration fails: stop using that instrument, and report
  it.
- Something would need a source change to measure: STOP and say what it is.
  A test that fails serially at HEAD is not a stop. Record it.

## Deliverables

- In `.temp/brief-627/`:
  - `census.json`, `census.md`, `memory.tsv`;
  - the reproduction logs, under `repro/`;
  - `REPORT.md` and `PROGRESS.md`;
  - the drafts, under `drafts/`.
- The report, appended below.

## Report

Append "Executing-agent report" to this brief.

1. **Environment.** The scope's `memory.max`, and `memory.events` at the
   start and at the end. The instrument calibrations, verbatim.
2. **Stage 1.** The three default-count results and the single-thread
   result, verbatim.
3. **The census table** from `census.md`, and each failing test with its
   counts. Give the doctests too.
4. **Memory.** The top 30, the threshold counts, the sums, and the check
   against K3.
5. **The families.** For each: the root cause at `path:line`, the class, the
   minimal reproduction with its counts, and its log.
6. **The plan.** Each fix and why it is enforced. The proposed default and
   its derivation. A list of the drafts.
7. **Every attempt** of every stage with its result, passes included, and
   the open questions.

Quote summary lines only, and link the logs. Do not paste whole test logs.
