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

## Executing-agent report

Design-only. No file under `crates/`, no `Cargo.lock`, no repo `.cargo/`, no `TODO.md` edit. Artifacts are under `.temp/brief-627/`.

## 1. Environment

Scope cgroup: `/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/brief-627-grk-20261027-184843-912000.scope`.

`memory.max` at start and at the end: `17179869184` (16 GiB). `memory.swap.max` at start: `0`. Host `nproc`: 32. Cargo 1.85.0, rustc 1.85.0. `RUST_TEST_THREADS` was unset for census runs. Default threads means no `--test-threads` flag, so libtest uses the affinity count, 32.

`memory.events` at start (`.temp/brief-627/env-start.txt`):

```
low 0
high 0
max 0
oom 0
oom_kill 0
oom_group_kill 0
sock_throttled 0
```

`memory.events` at the end, after Stage 4:

```
low 0
high 0
max 89404
oom 13
oom_kill 1
oom_group_kill 0
sock_throttled 0
```

`oom_kill` rose by 1. That rise is the `rum-stdlib` library default-count run. `oom` 13 is extra kernel accounting. It is not 13 killed runs. `memory.current` after Stage 4 was 610824192. A re-read of the same cgroup immediately before this report was appended still showed `max 89404`, `oom 13`, `oom_kill 1`. `memory.current` on that re-read was 631554048.

peak-rss direct calibration, `.temp/brief-627/cal/peak-rss.log`, verbatim:

```
command: python3 scripts/run-bounded.py --timeout 60 -- python3 scripts/peak-rss.py python3 -c 'b=b"\x01"*(1<<30); print(len(b))'
stdout: 1073741824
stderr: maximum resident set size: 1081462784
bytes: 1081462784
gib: 1.007191
band: 1.0-1.1 GiB
verdict: PASS
```

peak-rss through `cargo test`, `.temp/brief-627/cal/peak-rss-cargo.log`, verbatim:

```
command: CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=8 CARGO_TARGET_DIR=.temp/brief-627/rss-probe/target python3 scripts/run-bounded.py --timeout 300 -- python3 scripts/peak-rss.py cargo test --offline --manifest-path .temp/brief-627/rss-probe/Cargo.toml -- --test-threads=1
test: rss-probe allocate_one_gib, vec of 1<<30 bytes, kept alive with black_box
result: test allocate_one_gib ... ok
stderr/stdout tail: maximum resident set size: 1076035584
bytes: 1076035584
gib: 1.002136
band: 1.0-1.1 GiB
verdict: PASS
note: same nesting as the census (run-bounded -> peak-rss -> cargo test). Direct python calibration was 1081462784.
```

No process-tree sum is reported. The optional four-child sampler was not used.

Stage 0: `exit 0`, `wall_seconds 17.160`, 46 `Executable` lines. `.temp/brief-627/stage0/meta.txt`, `.temp/brief-627/stage0/no-run.log`. `oom_kill` stayed 0 across that build.

## 2. Stage 1

`.temp/brief-627/stage1/summary.txt`, verbatim:

```
default run 1: exit 101 failures 22 oom_delta 0 wall 0.476 peak 29687808
  test result: FAILED. 8 passed; 22 failed; 2 ignored; 0 measured; 1189 filtered out; finished in 0.28s
default run 2: exit 101 failures 14 oom_delta 0 wall 0.483 peak 29450240
  test result: FAILED. 16 passed; 14 failed; 2 ignored; 0 measured; 1189 filtered out; finished in 0.32s
default run 3: exit 101 failures 10 oom_delta 0 wall 0.426 peak 29671424
  test result: FAILED. 20 passed; 10 failed; 2 ignored; 0 measured; 1189 filtered out; finished in 0.27s
threads=1 run 1: exit 0 failures 0 oom_delta 0 wall 0.63 peak 29634560
  test result: ok. 30 passed; 0 failed; 2 ignored; 0 measured; 1189 filtered out; finished in 0.45s
```

Logs: `.temp/brief-627/stage1/profiling-default-1.log`, `profiling-default-2.log`, `profiling-default-3.log`, `profiling-threads-1.log`. Each default run failed at least one test. The single-thread run passed 30 tests. Calibration matched K1's shape. These failure counts are this run's counts.

## 3. Census

The table below is the target table from `.temp/brief-627/census.md`, verbatim. 46 targets. Each has 10 default rows and 5 `--test-threads=8` rows, except `rum-stdlib` lib at the default count: run 1 was OOM-killed and runs 2–10 were skipped. Four targets had a failing run. Every other target had 0 failed runs and 0 OOMs at both counts.

| package | kind | name | default runs | default failed runs | default OOMs | default median wall s | default max peak bytes | threads=8 runs | threads=8 failed runs | threads=8 OOMs | threads=8 median wall s | threads=8 max peak bytes |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| rum-cli | bin | rum-cli | 10 | 0 | 0 | 0.279 | 446664704 | 5 | 0 | 0 | 0.278 | 71323648 |
| rum-cli | test | cli | 10 | 0 | 0 | 0.234 | 446533632 | 5 | 0 | 0 | 0.280 | 32288768 |
| rum-cli | test | input_requests | 10 | 0 | 0 | 0.281 | 137506816 | 5 | 0 | 0 | 0.281 | 31981568 |
| rum-cli | test | terminal | 10 | 0 | 0 | 0.381 | 128761856 | 5 | 0 | 0 | 0.380 | 32210944 |
| rum-core | lib | rum_core | 10 | 0 | 0 | 0.180 | 198348800 | 5 | 0 | 0 | 0.179 | 29888512 |
| rum-derive | lib | rum_derive | 10 | 0 | 0 | 0.179 | 1135513600 | 5 | 0 | 0 | 0.181 | 30097408 |
| rum-derive | test | accumulator_chain | 10 | 0 | 0 | 0.179 | 1671315456 | 5 | 0 | 0 | 0.180 | 30007296 |
| rum-derive | test | derive | 10 | 0 | 0 | 0.178 | 130445312 | 5 | 0 | 0 | 0.178 | 29863936 |
| rum-host | lib | rum_host | 10 | 0 | 0 | 0.658 | 436772864 | 5 | 0 | 0 | 0.577 | 30699520 |
| rum-host | test | adapter_contract | 10 | 0 | 0 | 0.227 | 427024384 | 5 | 0 | 0 | 0.225 | 30617600 |
| rum-host | test | batch_boundary_ownership | 10 | 0 | 0 | 0.227 | 428048384 | 5 | 0 | 0 | 0.226 | 30658560 |
| rum-host | test | capture_output_lifecycle | 10 | 0 | 0 | 0.328 | 426262528 | 5 | 0 | 0 | 0.326 | 30441472 |
| rum-host | test | guest_loading_resume | 10 | 0 | 0 | 0.229 | 426754048 | 5 | 0 | 0 | 0.228 | 30670848 |
| rum-host | test | host_payload_destruction_contract | 10 | 0 | 0 | 3.040 | 137871360 | 5 | 0 | 0 | 3.090 | 108634112 |
| rum-host | test | installer | 10 | 0 | 0 | 0.226 | 425132032 | 5 | 0 | 0 | 0.228 | 30769152 |
| rum-host | test | lifecycle_buildout | 10 | 0 | 0 | 0.227 | 429805568 | 5 | 0 | 0 | 0.226 | 30633984 |
| rum-host | test | lifecycle_contract | 10 | 0 | 0 | 0.327 | 430772224 | 5 | 0 | 0 | 0.327 | 30814208 |
| rum-host | test | logical_profile_abandonment | 10 | 0 | 0 | 0.227 | 30752768 | 5 | 0 | 0 | 0.227 | 30572544 |
| rum-host | test | logical_profile_lifecycle | 10 | 0 | 0 | 0.229 | 30720000 | 5 | 0 | 0 | 0.226 | 30703616 |
| rum-host | test | random_password_policy | 10 | 0 | 0 | 0.177 | 422912000 | 5 | 0 | 0 | 0.179 | 30760960 |
| rum-host | test | shared_accumulator_completion | 10 | 0 | 0 | 0.228 | 424796160 | 5 | 0 | 0 | 0.226 | 30552064 |
| rum-host | test | stress | 10 | 0 | 0 | 0.378 | 430247936 | 5 | 0 | 0 | 0.377 | 30687232 |
| rum-host | test | suspended_timeout_contract | 10 | 0 | 0 | 0.980 | 426979328 | 5 | 0 | 0 | 0.932 | 30711808 |
| rum-host | test | suspension_abandonment_contract | 10 | 0 | 0 | 0.328 | 425807872 | 5 | 0 | 0 | 0.328 | 30707712 |
| rum-jupyter | lib | rum_jupyter | 10 | 10 | 0 | 0.678 | 38404096 | 5 | 5 | 0 | 0.628 | 38346752 |
| rum-jupyter | bin | rum-jupyter-kernel | 10 | 0 | 0 | 0.226 | 38383616 | 5 | 0 | 0 | 0.224 | 38301696 |
| rum-repl | lib | rum_repl | 10 | 0 | 0 | 0.478 | 447860736 | 5 | 0 | 0 | 0.424 | 31657984 |
| rum-repl | test | input_abandonment | 10 | 0 | 0 | 0.277 | 449708032 | 5 | 0 | 0 | 0.274 | 31817728 |
| rum-repl | test | input_continuation | 10 | 0 | 0 | 0.375 | 449470464 | 5 | 0 | 0 | 0.327 | 31748096 |
| rum-runtime | lib | rum_runtime | 10 | 10 | 0 | 12.602 | 419975168 | 5 | 5 | 0 | 12.924 | 281985024 |
| rum-stdlib | lib | rum_stdlib | 1 | 1 | 1 | 58.561 | 17015287808 | 5 | 0 | 0 | 132.852 | 12594774016 |
| rum-stdlib | bin | generate-builtin-reference | 10 | 0 | 0 | 0.175 | 141832192 | 5 | 0 | 0 | 0.176 | 30728192 |
| rum-stdlib | test | default_fallback_continuations | 10 | 0 | 0 | 0.377 | 421289984 | 5 | 0 | 0 | 0.327 | 30654464 |
| rum-stdlib | test | destructuring_continuations | 10 | 0 | 0 | 0.326 | 424017920 | 5 | 0 | 0 | 0.324 | 30756864 |
| rum-stdlib | test | guest_loading_resume | 10 | 0 | 0 | 1.030 | 426561536 | 5 | 0 | 0 | 0.829 | 30658560 |
| rum-stdlib | test | lazy_pull_continuations | 10 | 0 | 0 | 4.663 | 425254912 | 5 | 0 | 0 | 5.040 | 35405824 |
| rum-stdlib | test | loop_continuations | 10 | 0 | 0 | 0.475 | 422154240 | 5 | 0 | 0 | 0.377 | 30744576 |
| rum-stdlib | test | option_reader_continuations | 10 | 0 | 0 | 1.227 | 426057728 | 5 | 0 | 0 | 0.978 | 30593024 |
| rum-stdlib | test | rule_pattern_continuations | 10 | 0 | 0 | 0.777 | 424677376 | 5 | 0 | 0 | 0.627 | 30552064 |
| rum-stdlib | test | schema_continuations | 10 | 0 | 0 | 0.580 | 424448000 | 5 | 0 | 0 | 0.477 | 36159488 |
| rum-stdlib | test | sow_trace_continuations | 10 | 0 | 0 | 0.630 | 424472576 | 5 | 0 | 0 | 0.524 | 30777344 |
| rum-stdlib | test | suspension_oracle | 10 | 0 | 0 | 71.856 | 121475072 | 5 | 0 | 0 | 77.372 | 121712640 |
| rum-stdlib | test | typed_slot_census | 10 | 7 | 0 | 69.066 | 8648491008 | 5 | 4 | 0 | 68.979 | 8652881920 |
| rum-stdlib | test | typed_slot_compile_controls | 10 | 0 | 0 | 32.692 | 1149845504 | 5 | 0 | 0 | 33.519 | 1156956160 |
| rum-syntax | lib | rum_syntax | 10 | 0 | 0 | 1.481 | 221290496 | 5 | 0 | 0 | 1.479 | 91238400 |
| rum-terminal | lib | rum_terminal | 10 | 0 | 0 | 0.178 | 370511872 | 5 | 0 | 0 | 0.176 | 30560256 |

The four rows with a failed run or an OOM, same numbers:

| package | kind | name | default runs | default failed | default OOMs | default median wall s | default max peak | threads=8 runs | threads=8 failed | threads=8 OOMs | threads=8 median wall s | threads=8 max peak |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| rum-jupyter | lib | rum_jupyter | 10 | 10 | 0 | 0.678 | 38404096 | 5 | 5 | 0 | 0.628 | 38346752 |
| rum-runtime | lib | rum_runtime | 10 | 10 | 0 | 12.602 | 419975168 | 5 | 5 | 0 | 12.924 | 281985024 |
| rum-stdlib | lib | rum_stdlib | 1 | 1 | 1 | 58.561 | 17015287808 | 5 | 0 | 0 | 132.852 | 12594774016 |
| rum-stdlib | test | typed_slot_census | 10 | 7 | 0 | 69.066 | 8648491008 | 5 | 4 | 0 | 68.979 | 8652881920 |

`rum-stdlib` lib at 8 threads, five runs, all exit 0. Result shape: `ok. 1111 passed; 0 failed; 53 ignored`. Peaks 12263747584, 12594774016, 12165107712, 12434468864, 12503580672. The default-count OOM log has no `test result:` line. Peak 17015287808 (15.847 GiB), wall 58.561 s, exit 101, `oom_kill` delta 1.

Failing tests. Counts are runs of that thread count that named the test. Alone is `--exact` at `--test-threads=1`, 10 runs. Table is the one in `census.md`.

| test | default | threads=8 | alone |
|---|---:|---:|---:|
| `interrupt_at_second_prompt_finishes_original_request` | 3 | 2 | 0 |
| `red_interrupt_while_waiting_finishes_request` | 3 | 3 | 0 |
| nine `zeromq::tests::native::...` tests | 10 | 5 | 10 |
| `logical_thread_metadata_stays_charged_until_thread_exit` | 10 | 0 | 0 |
| `profiling_observer_covers_every_map_and_flag_without_ranking` | 0 | 1 | 0 |
| `logical_record_chain_concurrent_last_holders` | 10 | 4 | 0 |
| `logical_record_chain_drops_on_two_mib` | 10 | 4 | 0 |
| `logical_source_chain_concurrent_last_holders` | 10 | 5 | 0 |
| `logical_source_chain_drops_on_two_mib` | 10 | 5 | 0 |
| `logical_after_consumer_sees_the_input_parser_error` | 6 | 1 | 0 |
| `logical_deferred_return_boundary_observes_the_return_resource_error` | 4 | 1 | 0 |
| `logical_diagnostic_cap_keeps_the_marker_free_guest_capacity` | 10 | 5 | 0 |
| `logical_driver_slot_fault_abandons_the_rooted_boundary` | 4 | 1 | 0 |
| `logical_every_structural_route_closes_once_with_its_reason` | 6 | 4 | 0 |
| `logical_frontier_rejects_another_owner_even_without_semantic_cells` | 1 | 1 | 0 |
| `logical_queue_representation_survives_a_disabled_global_gate` | 5 | 0 | 0 |
| `logical_queued_frames_have_one_owner_in_both_modes` | 1 | 2 | 0 |
| `logical_zero_frames_and_after_input_frontier_have_no_guest_storage` | 6 | 4 | 0 |
| `logical_marker_observes_closed_channels_without_codecs` | 2 | 2 | 0 |
| `logical_marker_preserves_slot_identity_and_rejects_wrong_channels` | 5 | 2 | 0 |
| `removing_a_frame_in_a_scratch_copy_is_red` | 0 | 2 | 0 |
| `removing_a_test_guard_in_a_scratch_copy_is_red` | 7 | 2 | 0 |

The nine ZeroMQ names are listed in `census.md`. They fail alone. Panic: `native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1)` at `crates/rum-jupyter/src/zeromq_native_test_support.rs:346`. `loopback_bind_available` (`zeromq_native_test_support.rs:335-354`) panics on `PermissionDenied` unless `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` is set. This run did not set that variable. Serial environmental failure. It is not a parallel family.

Doctests: 3 runs, all exit 0, `oom_kill` delta 0. Each log has nine `test result:` lines, all `ok`, 30 doctests (0+7+1+0+0+22+0+0+0). The census row stores the last line, an empty crate. Run 1 lines are in `census.md`. Walls: 13.582 s, 9.92 s, 9.97 s. Peaks: 416477184, 419553280, 419532800. Logs: `.temp/brief-627/logs/doctest/workspace-doc-default-r0{1,2,3}.log`.

Some census panic excerpts are empty. The parser stops at the blank line after `panicked at`. The typed-slot message is in the reproduction log cited below. Failure names are the harness's final `failures:` list.

## 4. Memory

`.temp/brief-627/memory.tsv` has a `disposition` column. `ignored` means the result line is `0 passed; 0 failed; 1 ignored`: libtest `--list` printed `#[ignore]` tests as `: test`, and `--exact` without `--ignored` did not run the body. 2394 rows, 2335 executed, 59 ignored. Executed rows: 0 nonzero exits, 0 `oom_kill` deltas. Stage 3 wall times are not timing evidence.

Thresholds among the 2335 executed peaks: above 256 MiB, 44; above 1 GiB, 28; above 4 GiB, 0.

Sums, from `.temp/brief-627/memory-summary.txt`:

```
sum32 48512475136 bytes = 48.512 GB = 45.181 GiB
sum8 19267821568 bytes = 19.268 GB = 17.945 GiB
max_k_sum_le_12GiB 4 sum 10905190400 bytes = 10.905 GB = 10.156 GiB
k_plus_1_sum 13110251520 bytes = 13.110 GB = 12.210 GiB
```

K3 check: the sum of the 32 largest peaks is 48.512 GB, above 14.6 GB. The memory is per test. The largest peaks are `rum-stdlib` retention-closure tests and `typed_slot_census`. The Stage 2 stdlib process peak of 15.847 GiB is one process that the kernel killed, so it is below the 45 GiB sum of peaks that never occupied one process together.

Top 30 executed peaks, bytes, from `memory-summary.txt`:

```
3098042368  tests::retention_closure::local::reads::tests::local_option_borrowed_pattern_controls
2847961088  tests::retention_closure::local::option_domain::local_option_domain_reconciliation_and_independent_drift_controls
2670747648  tests::retention_closure::local::option_boundaries::tests::local_option_contract_controls
2288439296  tests::retention_closure::local::type_tests::finite_profile_modes_are_closed_and_selected_at_both_calls
2205061120  tests::retention_closure::local::promotion_tests::production_body_domain_and_build_expiry_and_raw_local
2178691072  tests::retention_closure::local::reads::tests::local_option_read_contract_controls
2113187840  tests::retention_closure::local::rules::tests::local_rule_contract_controls
1865691136  tests::retention_closure::census::tests::production_rejection_census_is_complete
1825566720  tests::retention_closure::production_obligations_are_exact
1555845120  tests::retention_closure::local::rules::tests::local_rule_macro_definition_arms_hygiene_and_raw_frame_controls
1414443008  typed_slot_census source_baseline_matches_earned_receipt
1315807232  typed_slot_census removing_a_witnessed_insertion_in_a_scratch_copy_is_red
1315700736  typed_slot_census witnessing_a_different_binding_in_a_scratch_copy_is_red
1315385344  typed_slot_census shadowing_a_witness_in_a_cold_helper_is_red
1256452096  tests::retention_closure::local::admission_tests::local_pending_argument_and_wrapped_payload_controls
1252999168  tests::retention_closure::local::production::local_production_acceptance
1252659200  tests::retention_closure::local::promotion_tests::production_promotion_cannot_launder_consumers_or_runtime_domain
1252564992  tests::retention_closure::local::promotion_tests::production_marks_require_current_local_lint_and_direct_contracts
1252241408  tests::retention_closure::local::oracle::local_probe_claims_match_stable_rustc
1252237312  tests::retention_closure::local::promotion_tests::production_inventory_rejects_removed_redirected_stale_and_omitted_evidence
1223651328  tests::retention_closure::local::reads::tests::local_option_short_circuit_ownership_controls
1219760128  tests::retention_closure::local::contract_tests::local_concrete_contract_controls
1219436544  tests::retention_closure::local::reads::oracle::local_option_read_claims_match_stable_rustc
1219342336  tests::retention_closure::local::tests::local_contract_falsifiers
1202929664  tests::retention_closure::local::type_tests::finite_modes_and_opaque_brands
1174413312  typed_slot_census removing_a_test_guard_in_a_scratch_copy_is_red
1173864448  typed_slot_census removing_a_frame_in_a_scratch_copy_is_red
1169125376  typed_slot_census preflip_no_legacy_transport_uses
992030720   tests::retention_closure::census::tests::n_frontier_keeps_unresolved_roots
853200896   tests::retention_closure::compiler_oracle::methods::inherent_method_claims_match_rustc
```

Package is `rum-stdlib` for every row. The first ten and the rows from `local_pending_argument` down except the `typed_slot_census` rows are target `rum_stdlib`. The `typed_slot_census` rows are target `typed_slot_census`.

Stage 3 launch: `parallel -j 8 --memfree 4G` for the two libraries (2385 tests) and `-j 1` for `typed_slot_census` (9 tests). `--memfree` reads host `MemAvailable`, which was about 25 GiB, so it would not have held two 8 GiB processes inside the 16 GiB cgroup. The narrow job is that deviation. Peaks are still per process. `stage3-complete.txt`: `wide_exit 0`, `narrow_exit 0`. No OOM rerun was required.

## 5. Families

### Profiler — class (a)

Shared state:

- `PROCESS_BYTES` and `PROCESS_LIMIT` (16 MiB) at `crates/rum-runtime/src/runtime/profiling/logical/budget.rs:7-8`. `Budget::claim` charges both the epoch budget and `PROCESS_BYTES` (`budget.rs:40-46`).
- The process-wide epoch, `CURRENT` at `crates/rum-runtime/src/runtime/profiling/logical/epoch.rs:18`. `reset` retires it with `ProfileReset` (`epoch.rs:267-272`). `storage` retires it with `DiagnosticCap` when the claim fails (`epoch.rs:78-86`, print at `:220-222`).
- `enabled()` at `crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs:10-22` calls `epoch::reset()`, swaps `LEXICAL_PROFILE_ENABLED` and `LEXICAL_PROFILE_ACTIVE` (`state.rs:20-21`), and on drop resets the epoch again. It holds no lock.
- The flags are what `lexical_profile_enabled` reads (`config.rs:4-5`).

Two threads collide because each chain test builds a 66,000-node chain that the test itself requires to exceed 8 MiB (`chain_drop.rs:73-74`). Two of those exceed the one 16 MiB ceiling. `record_chain` calls `Epoch::new(16 MiB)` and `epoch.storage` (`record.rs:138-140`). A failed claim retires that epoch and `unwrap`s `None`. `source_chain` calls `enabled()` and pushes a `SourceStack` onto `epoch::current()` (`source_stack.rs:50-54`). A failed claim sets `missing`, and `last()` is `None`, so `chain_drop.rs:89-91` panics with `the whole chain must fit the diagnostic cap`. `enabled()` also resets whatever epoch the other thread is using.

Minimal reproduction:

```
cargo test -p rum-runtime --lib -- --test-threads=2 --exact \
  runtime::profiling::tests::chain_drop::logical_source_chain_drops_on_two_mib \
  runtime::profiling::tests::chain_drop::logical_record_chain_drops_on_two_mib
```

20 runs at `--test-threads=2`: 20 failed. `logical_record_chain_drops_on_two_mib` failed 20. `logical_source_chain_drops_on_two_mib` failed 16. Same two tests at `--test-threads=1`: 5/5 passed. Each alone in the census: 10/10 passed. Log: `.temp/brief-627/repro/chain-pair/t2-r01.log` prints `rum-profile logical epoch=1 valid=false reason=DiagnosticCap` and `epoch=2 valid=false reason=DiagnosticCap`, then the two panics above. Counts: `.temp/brief-627/repro-counts.txt`.

Class (a). The ceiling and the single epoch are process-wide by design (`budget.rs:1`, `.temp/brief-624/DESIGN.md:84-90`). The tests assume exclusive use of both. Stage 1's profiling filter, with no ordinary tests in the run, failed 22, 14, and 10 tests. The collisions do not require an ordinary test.

Ordinary evaluation beside a profiling test: `logical_diagnostic_cap_keeps_the_marker_free_guest_capacity` plus `value_q_shortcut_populates_only_evaluation_caches` passed 20/20 at `--test-threads=2` (`.temp/brief-627/repro/profiler-plus-ordinary/`). `logical_thread_metadata_stays_charged_until_thread_exit` plus the same value-q test passed 15/15 (`.temp/brief-627/repro/profiler-metadata-ordinary/`). Those assertions stayed green. The flag is still process-global, so a later ordinary test can record while a guard is held. The plan covers that. These two pairs are not the red falsifier.

Two concurrently profiled sessions and the owner filter: stopped. Measuring it needs a test that records two owners into one epoch without calling `reset`. Every current enable path calls `enabled()`, which resets first (`engine_support.rs:11`). The only `thread::spawn` under `profiling/` is the metadata test's worker, and that worker calls `epoch::reset()` itself (`budget.rs:134`). No existing test shows two live owners in one epoch on two threads. This run did not add that test. The design promise was not confirmed and was not contradicted.

### Jupyter interrupt — class (c)

`red_interrupt_while_waiting_finishes_request` and `interrupt_at_second_prompt_finishes_original_request` (`interrupt.rs:4-16`) both call `interrupt_at_prompt`. Each builds its own session, adapter, transport, and channels. Cancellation is per `RequestInterruptState` (`request_cancellation.rs:146-153`). The execute-worker gate is a channel on that adapter (`adapter/mod.rs:133-141`, `worker.rs:242-245`).

The missing value is the cleanup `Print`. Non-streaming, `interrupt.rs:150-159` counts iopub `cleanup\n` and gets `left: 0 right: 1`. Streaming, `interrupt.rs:166-167` does `receive.try_recv().unwrap()` and gets `Empty`. The sink is `push_print` (`session_helpers.rs:385-386`). `pump_completion` (`execute_lifecycle/mod.rs:91-94`) waits until the worker sends `finished`, and that send is after `run_segment` returns (`worker.rs:338-345`, wait at `:215-219`). The print is absent from a finished outcome. Earlier asserts in the same function have already accepted a `Cancelled` reply and one idle status, which is why the panic is at line 151 or 167.

Minimal reproduction, 30 runs at `--test-threads=2`: 26 failed, always exactly one of the two tests (14 and 12). Of the 26 logs, 22 panic at `interrupt.rs:151:9` and 4 at `interrupt.rs:167:49`. Four runs passed. Logs: `.temp/brief-627/repro/jupyter-interrupt/`. Census alone: 10/10 each.

Control: the red-interrupt test plus `tests::adapter::execute_request_maps_to_session_eval_and_tracks_execution_count` passed 20/20 at `--test-threads=2`. Logs: `.temp/brief-627/repro/jupyter-vs-adapter/`. The partner that produces the flake is the other interrupt retirement, not an arbitrary second test in the process.

Class (c). The tests assume the cleanup print is in the outcome once the worker has signaled finished. That assumption fails when the two retirements run together. This pass did not find a process-wide cell both tests write. The host document says one session per worker is the supported parallel shape (`docs/host-embedding-and-hardening.md:103-105`). If the omitted print is a product miss under that shape, the fix is still in retirement, and the follow-up brief says so.

### Typed-slot scratch — class (a)

Both falsifiers call `forced_red` (`typed_slot_census.rs:99-105`). Both write `.temp/typed-slot-census/forced-red/tree` (`typed_slot_census.rs:111`) and the ignored probe reads that path (`typed_slot_census.rs:199-200`). Two threads blank different sites of one tree. The child then sees a torn census.

Minimal reproduction, 10 runs at `--test-threads=2`: 8 failed, all `removing_a_test_guard_in_a_scratch_copy_is_red`. The frame test passed in these 10. Logs: `.temp/brief-627/repro/typed-slot-scratch/`. `t2-r03.log`: parent panic at `typed_slot_census.rs:188:5`; child `forced_red_probe` panics at `:200:31` with `G15-C1 baseline requires production 610/542 and test-only 16/11; combined 626/553`. The child census line in that log is `production frames=471 ... implementations=625`. The guard-removal marker the parent requires is `production frames=472 ... implementations=626`. Census alone: 10/10 each. In the full target the frame test failed 2/5 at 8 threads, which is the same shared directory with more scheduling noise.

Class (a). The shared path is a test fixture. The witness falsifiers already use one directory per kind (`typed_slot_census.rs:222`).

### Not a parallel family

ZeroMQ, above. Resource exhaustion (d) is the stdlib default-count OOM, handled by the thread-count proposal, not by deleting tests.

## 6. Plan

### Profiler

`enabled()` returns a guard whose lifetime holds a test-only `Mutex`. `Epoch::new` becomes private to `current()`. Tests build an epoch only with `Epoch::for_test(&lease, limit)`, so `record_chain` cannot charge `PROCESS_BYTES` without holding the same lease. The compiler rejects a new LP03–LP08 test that calls the old constructor.

`lexical_profile_enabled` stays one relaxed load when the flag is off. When the flag is on and no test lease is held, every thread may record, which keeps simultaneous production sessions on the one epoch. When a test lease is held, only threads the lease registered may record. Ordinary tests in the same binary therefore do not enter the epoch. The metadata test's worker calls `allow_current_thread` before `Call::start`.

The red falsifier is the chain pair at `--test-threads=2`. Draft: `.temp/brief-627/drafts/brief-profiler-test-lease.md`.

### Jupyter interrupt

`run_segment`'s `CancelInput` path must put the cleanup `Print` into the outcome before it returns. The tests already wait for `finished`. The assertions stay. The red falsifier is the two interrupt tests at `--test-threads=2` (26/30 at HEAD). Draft: `.temp/brief-627/drafts/brief-jupyter-interrupt-cleanup.md`.

### Typed-slot scratch

`forced_red` creates a directory named with the thread id and an atomic counter. The probe takes that path as an argument and has no default `forced-red/tree`. Callers cannot share a path. The red falsifier is the two scratch tests at `--test-threads=2` (8/10 at HEAD). Draft: `.temp/brief-627/drafts/brief-typed-slot-scratch-dir.md`.

### Proposed default: 4

Largest k whose k largest executed Stage 3 peaks sum to at most 12 GiB (12×2^30). k=4 sums to 10905190400 bytes (10.156 GiB). k=5 sums to 13110251520 bytes (12.210 GiB).

Stage 2 walls on this host: `rum-stdlib` lib at 32 threads was killed at 58.561 s and 17015287808 bytes. At 8 threads, five runs passed, median wall 132.852 s, maximum peak 12594774016 bytes (11.730 GiB). The sum of the eight largest per-test peaks is 17.945 GiB, so eight threads can exceed 12 GiB when those tests overlap at their peaks. The five observed processes stayed under 12 GiB. The proposal follows the sum rule and is 4. This run did not time the stdlib suite at 4 threads or at 1 thread.

4 threads does not make the profiler family green. The chain pair fails at 2. The runtime lib failed all 5 runs at 8 threads and all 10 at 32. The lease, the scratch directories, and the interrupt fix are what make those targets green. The thread cap is what keeps the stdlib suite inside the scope.

Where it would live: a new repository `.cargo/config.toml`

```toml
[env]
RUST_TEST_THREADS = "4"
```

Scratch probe `.temp/brief-627/thread-probe/probe-result.txt`, Cargo 1.85, crate not a workspace member:

- Variable unset: `RUST_TEST_THREADS=4`, `BRIEF627_PROBE=from-cargo-config`, `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`. Exit 0.
- Ambient `RUST_TEST_THREADS=1`: the test printed `RUST_TEST_THREADS=1` and failed `left: "1" right: "4"`. Exit 101. An existing variable wins. `force = true` is not proposed, so a developer can override. A launcher that exports `RUST_TEST_THREADS=1` stays serial until that export is removed.

Not installed. Draft: `.temp/brief-627/drafts/brief-default-test-threads.md`.

### Drafts

- `.temp/brief-627/drafts/brief-profiler-test-lease.md`
- `.temp/brief-627/drafts/brief-jupyter-interrupt-cleanup.md`
- `.temp/brief-627/drafts/brief-typed-slot-scratch-dir.md`
- `.temp/brief-627/drafts/brief-default-test-threads.md`

## 7. Attempts and open questions

`attempts.jsonl` has 3505 rows. Exit 124: 0. No wrapper timeout was resized.

| phase | rows | result |
|---|---:|---|
| stage1 | 4 | 3 default failures (22, 14, 10), 1 single-thread pass |
| census | 681 | executed target runs. 9 further default stdlib rows were skipped after the OOM and are in `census.jsonl` only |
| alone | 300 | 30 names × 10. Only the 9 ZeroMQ tests failed |
| doctest | 3 | 3 passes |
| list | 3 | Stage 3 `--list` for the two libs and `typed_slot_census` |
| memory | 2394 | primary rows, 0 oom, 0 nonzero exit |
| repro | 120 | chain 25, profiler-plus-ordinary 20, jupyter-interrupt 30, typed-slot 10, jupyter-vs-adapter 20, profiler-metadata-ordinary 15 |

Stage 0 build: one command, exit 0, wall 17.160 s. Stage 3 parallel: wide exit 0, narrow exit 0. Stage 4 harness printed `stage4 done` twice (primary cases, then the two follow-ups). Reproduction exits are in `repro-counts.txt`.

Passes included: 42 of 46 targets passed every census run. Doctests passed. The chain pair passed 5/5 at one thread. The ordinary-evaluation pairs passed 20/20 and 15/15. The interrupt-plus-adapter pair passed 20/20.

Open questions:

- Owner-filter concurrency was not measured. It needs a new test with two owners in one epoch and no `reset` between them. This brief did not add it.
- The unregistered-thread recording test in the profiler draft may already be green. The chain pair is the falsifier that is red today.
- The interrupt flake's shared cell was not named. Class (c), with the contract note in the family section.
- Nine native ZeroMQ tests fail serially here with EPERM on `127.0.0.1:0`. The suite cannot be fully green in this sandbox without loopback, and this run did not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.
- k=8 stayed under 12 GiB in five stdlib samples and is outside the sum rule. The director can pick 8 after a measurement this brief did not run.
- `memory.events` `oom` is 13 while `oom_kill` is 1. The killed run is the one `oom_kill` delta.

## Director review (2026-10-27 21:40)

I accept the census and three of the four families. I reject the Jupyter
classification. Its real cause is a product defect in the native stack guard,
and it gets its own design brief.

1. **The census reproduces.**
   - The calibration matches K1.
   - My own run of the Jupyter pair at `--test-threads=2` failed 17 of 20
     (`.temp/director/jup-627/`).
   - The memory sums are arithmetically consistent with the rows.
2. **Profiler: class (a), accepted.** The lease shape is accepted with two
   changes in my rewrite:
   - The thread-registration check is compiled only under `#[cfg(test)]`.
     Production `lexical_profile_enabled` gains no branch keyed on test
     state.
   - A contract test runs two sessions on two threads, both registered under
     one lease and profiled at the same time. Each session's owner-filtered
     receipt must equal its solo receipt. That settles the (a)/(b) question
     you left open. If it is red, that is a product defect and a STOP.
3. **Typed-slot scratch: class (a), accepted** as drafted.
4. **Jupyter interrupt: class (c) rejected. The cause is a stale native
   stack base.** "Timing" without a named shared cell was not an answer: two
   isolated sessions that interfere share something. I found it with two
   experiments and temporary `eprintln` probes, now removed and the tree
   restored:
   - **Pinned to one CPU**, `taskset -c 2`, the pair passed 20 of 20. At
     `--test-threads=2` on the full host it failed 17 of 20. The failure
     needs the two to run at the same time, not just preemption, so it is
     not a scheduling assumption.
   - **The failing side never ran its cleanup `Print`.** Abandonment's
     `self.evaluate(cleanup.expr)` (`suspension/core/abandon.rs:96`)
     returned `Err(StackLimitExceeded { limit: 1048576 })` with
     `cancelled=false`. Abandonment discards that error.
   - **The base belongs to another thread.**
     - `restore_resume_context` (`evaluation_context.rs:38`) restores
       `evaluation_depth = 2`. So `Session::evaluate` (`resolution.rs:319`)
       skips its depth-0 rebase, and the guard measures against
       `native_stack_base`.
     - That base was recorded on the worker thread that started the request.
       Each Jupyter execute runs on a fresh thread (`adapter/worker.rs`).
     - Probe lines: `depth=2 base=0x7f848126d7bf here=0x7f848106caff`. That
       is a false 2 MiB, against a 1 MiB limit.
   - **The opposite case is fail-open, and it was observed.**
     `base=0x7f848106c7bf here=0x7f848126daff`. Here `saturating_sub` gives 0,
     so the guard cannot fire for that cleanup. `run_with_native_stack_guard`
     (`native_stack_guard.rs:98`) reads the same field, so every guarded
     stdlib traversal inside such a cleanup inherits the stale base.
   - Your draft would have put the print into the outcome. That hides a
     guard defect behind a passing test. It is withdrawn.
   - This is brief-628, design-only. The guide's contract is "runs guest
     cleanup once" (`docs/guide/getting-started.md:76`, `:169`).
5. **Memory: the 1.2 GB cluster is a repeated parse.** Each retention-closure
   test calls `source::production()` (`retention_closure/source.rs:51`) and
   builds its own `Interfaces`. Every test parses the whole workspace again
   from the same immutable input.
   - The root fix is one shared, read-only parse per process.
   - A thread cap only works around it.
   - The default thread count stays a proposal. I measure it after the
     fixes, at 4, 8, 16 and 32, and take it to the user.
6. **ZeroMQ:** environmental. The sandbox has no loopback. My gate runs
   those tests natively.

Order: 628, the native stack base across threads (design). Then the profiler
lease, the typed-slot scratch directory, and the shared retention parse.
Then the measured default.
