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

## Director amendment 1 (2026-10-29): the C4 STOP came from my predicate

Under the brief as written, you were right to stop. But the predicate was
wrong, and it was mine. I wrote it in brief-627's review, item 2, and
repeated it here as C4. I took it from the sentence in
`docs/dev/profiling.md`, "Comparison isolates one epoch, or filters native
receipts by owner", and did not read the code first.

There is no owner-filtered receipt:

- `Epoch::rows` is keyed by `AggregateKey`.
- `totals::settle` (`logical/totals.rs:88-113`) adds a settled record into
  its key's row, then removes the record.
- An `Owner` lives only on an open record. There it does two things:
  - it routes the record to its request (`segments::owner`,
    `logical/segments.rs:76-88`);
  - it limits abandonment to that owner's records (`Epoch::abandon_owner`,
    `logical/epoch.rs:231-242`).

Your result was `[2, 2, …]` against a solo `[1, 1, …]`. That is two equal
sessions recording into one key, so the row is the sum of the two solos. The
single epoch is working as designed; this is not a defect.

The documentation sentence claimed a filter that does not exist. This
commit corrects it: comparison isolates one epoch, rows are keyed by logical
key, and requests profiled at the same time add up per key.

### C4′ replaces C4: the concurrency contract the single epoch does make

Put three tests in one probe file on the scratch copy. Each one calls
`enabled()`, so run each alone with `--exact`, 10 runs per test.

1. **No cross-attribution.**
   - Session A records the phase `owner-a` and session B records `owner-b`,
     on two threads.
   - Both records are open at the same moment: each side opens its `Call`,
     both meet at a `Barrier`, then each side finishes.
   - Expected: the `owner-a` row equals A's solo row, the `owner-b` row
     equals B's solo row, and no other row changes.
2. **Same-key additivity, with sides that differ.** Both sessions record one
   key, and the two sides differ, so a row that swaps or drops one side
   cannot pass. For example, A makes one `Ok` call and B makes two calls, or
   one call that fails, if the API allows it. Expected: the concurrent row is
   the field-by-field sum of the two solo rows.
3. **Owner-scoped abandonment.**
   - Both records are open at the same moment.
   - Abandon A's owner through the production path that reaches
     `abandon_owner`: a Session drop or a cancellation. Cite the path with
     `path:line`.
   - Then B finishes `Ok`.
   - Expected:
     - A's key shows the abandonment reason;
     - B's key equals B's solo `Ok` row;
     - `Session::logical_profile_test_assert_quiescent()` holds at the end.

Calibrate each test against a mutant on the scratch copy, and keep both logs:

- for test 1, route B's record into A's key;
- for test 2, drop one side's counts in `settle`;
- for test 3, make `abandon_owner` remove every record.

Each test must be red on its mutant and green unmutated. If any test is red
unmutated, STOP: that is a product defect.

### C5 ruling

- **Lease falsifier per thread:**
  `unregistered_thread_span_does_not_record_while_a_profiling_guard_is_held`.
  It is red at HEAD for the right reason: the flag is process-global.
- **Drop the `evaluate("1")` form.** Production evaluation does not enter
  the LP02 engine until LP03 wires its phases, so that test cannot fail
  today.
- **Family falsifier:** the chain pair, unchanged.

### Then finish the brief

Do C1 (run `cargo tree -e features`, as the brief asks), C2's type design,
C3, D, E and F. The census rows from run 1 stand; do not rerun them.

Write this run's report to `.temp/brief-633/report-c2.json`. Do not edit
`report.json` from run 1. Your final message is
`python3 scripts/report.py summary 633 --stage c2`.

## Director amendment 2 (2026-10-30): grk's balance ran out, kimi takes over

grk's run 2 ended at turn 222 with
`API error (status 402 Payment Required): Grok Build usage balance exhausted`.
It wrote no `report-c2.json`. **You (kimi) continue the brief from where run 2
stopped.** Read the whole brief first, amendment 1 included. Everything
run 2 left is under `.temp/brief-633/`:

- the checkpoint `PROGRESS.md`. Its "Exact next action" is where you start;
- logs: `logs/c4p/` (C4′ and its mutants), `logs/tree/` (C1),
  `logs/lease/`, `logs/scratch/`, `logs/parse/`;
- two drafts: `drafts/brief-profiler-test-lease.md` and
  `drafts/brief-private-scratch.md`;
- the scratch tree `tree/`, which carries run 2's lease and its edit to
  `typed_slot_census.rs`, and its target directory `target-scratch/`.

### Re-verify before you build on run 2

Run 2's claims are a worker's claims. Before you build on them, reproduce
these on the scratch tree, and save the logs under `logs/c3/`:

- each of C4′'s three tests, once, unmutated: green;
- the chain pair at `--test-threads=2`, 5 runs: green;
- the span falsifier
  (`unregistered_thread_span_does_not_record_while_a_profiling_guard_is_held`):
  green.

Also check the claim about `logical_thread_metadata_stays_charged_until_thread_exit`
(`PROGRESS.md`). Run 2 says `std::thread::scope`'s implicit wait returns
before the worker's TLS destructors run, so the test now joins its worker
explicitly. Prove that mechanism with a minimal probe crate under `probe/`,
using a TLS value whose `Drop` is observed. Otherwise name it unproven. If the
explicit join changes what the test asserts, say so in `disclosures`.

### The rlib race is a finding, not a scratch artefact

- `crates/rum-runtime/src/runtime/accounting_mode/tests/compile.rs:18-37`
  runs `cargo build --lib` into the test's own target directory, which can
  replace `deps/librum_runtime-*.rlib`.
- `crates/rum-runtime/src/runtime/options/tests/compile.rs:5-15` picks the
  newest `librum_runtime-*.rlib` in that same `deps` directory.
- Run 2 worked around the overlap by keeping a hardlink,
  `target-scratch/debug/deps/librum_runtime-brief633-keep.rlib`.

What to do:

1. Delete that hardlink.
2. Put the pair into the B1 or B2 inventory.
3. Establish its precondition: when does the build actually replace the
   rlib? For example, a fresh target, or a stale lib.
4. Design its fix the same way as the others: the path or the artifact comes
   from a constructor that a test cannot bypass.
5. Calibrate its falsifier red and green.

Then go on to the remaining work: the typed-slot pair, E (measure, do not
estimate), the rest of B, D and F, and the drafts.

### Report

- Write `.temp/brief-633/report-c3.json`. It covers run 2's work, citing
  run 2's logs, and yours.
- Mark each run-2 claim either "re-verified" (with your log) or "run 2 only".
- Do not edit `report.json`.
- Your final message is `python3 scripts/report.py summary 633 --stage c3`.

## Director amendment 3 (2026-10-30): a measurement OOM killed the worker

Continuation 3 (kimi session `session_5dadef8f-151c-4aa9-a829-60b899067dec`)
ended at 02:05 with exit 137. The scope line reads
`peak=17179869184 oom=39 oom_kill=4 rc=137`, and the process killed was kimi
itself. `run-c3-typed-options.sh` had run option ii at 16 threads (its test
binary was SIGKILLed at 16.83 GB, the cap) and then at 32. One of the four
kills inside the shared 16G scope took the worker. The loop outlived the
worker, and I let it finish where it was; its rows are in
`logs/typed-options/status.tsv`.

**The cause is my brief.** Its concurrency plan put every measurement inside
the worker's own scope. E asked for each option's memory at 16 and 32
threads, and an option whose memory grows with the thread count reaches the
cap there. A test that reaches the cap shares its OOM domain with the worker.

**Rules from here on.**

1. **Every command that runs a test or probe binary runs in its own scope:**
   `scripts/memory-scope.sh b633-<label>-<n> 14G <command>`. Wrap
   `run-one.sh`'s command in it, and make each unit name unique; systemd
   refuses a second active unit with the same name.
   - Calibrated by me: a nested scope is a sibling under `app.slice` with its
     own cap. It is not inside your scope.
   - Check it once before anything else:
     `scripts/memory-scope.sh b633-probe-1 1G cat /proc/self/cgroup` must
     print a path ending in `app.slice/b633-probe-1.scope`.
   - If that fails, STOP and quote the error.
2. **One measured test process at a time.** This replaces the plan's
   "at most two test processes beside each other". The host has 30 GB, and
   two 14G scopes beside your own leave nothing for the host.
3. **Within one option, stop at the first thread count killed at the cap.**
   - Record it as `killed at cap (14G) at tN`. A kill at the cap is a measured
     result: the peak is greater than 14G.
   - Give the measured points below it and their per-thread slope.
   - Do not run that option at a larger count.
4. **These rows stand.** Do not rerun options ii or iii at 16 or 32 threads.

   | Option | t4 | t8 | t16 | t32 |
   | --- | --- | --- | --- | --- |
   | (i) shared `Sync` discovery | 1.18 GB | 1.18 GB | 1.18 GB | 1.19 GB |
   | (ii) discover per worker | 4.70 GB | 9.38 GB | killed at 16.83 GB (16G cap) | killed |
   | (iii) shared text, rediscover | 4.71 GB | 9.40 GB | killed at 17.12 GB (16G cap) | killed at 17.11 GB (16G cap) |

   The values are max RSS from `logs/typed-options/`. t8/t4 is 2.00 for both
   ii and iii: linear in the thread count.

Then continue from your own last step: the remaining E work, B, D, F, the
drafts and `report-c3.json`. In `disclosures`, list every measurement this
run made inside the shared scope before this amendment, and say whether a cap
kill could have touched it.
## c3 report (kimi, DONE)

Run-1 census stands. B1/B2/B3 inventory 214 globals, 11 path groups and 67 whole-tree tests; two real path collisions are covered by constructors, and the single-epoch profiler family by a held lease. C4′, TLS, host reset and fresh-rlib claims are re-verified. Nineteen peak decompositions are complete. Exact-slice Sync source is 486–494 MiB at 4–32 workers, but its real suite hit 14G, so the final design also shares production/population Interfaces and uses copy-on-write Source maps (option iv). Both certificate builds and ownership replay are green. Full retention at eight, typed at 32 and three workspace runs at 32 are green with only nine named ZeroMQ EPERM exclusions. Five implementation drafts and red/green logs are provided; no repository source changed.

| File | Kind | Lines | Why |
| --- | --- | --- | --- |
| `.temp/brief-633/tree-final/crates/rum-host/tests/logical_profile_abandonment.rs` | modified | 38-45 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-host/tests/logical_profile_lifecycle.rs` | modified | 77-83,89-95,127-133,141-152,183-189,203-209,214-220,222-240,250-256,259-265,269-275,277-283,286-292 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/accounting_mode/tests/compile.rs` | modified | 1-4,12-21,65-71 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/mod.rs` | modified | 110-122 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/options/tests/compile.rs` | modified | 1-8,12-19,46-54 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/lease.rs` | added | 1-58 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/logical/budget.rs` | modified | 92-98,102-108,114-152,160-167 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/logical/epoch.rs` | modified | 54-60,75-106,118-133,278-306,313-319,321-344 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/logical/mod.rs` | modified | 14-20,91-112,193-200,225-238 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/mod.rs` | modified | 9-20 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/reset.rs` | modified | 1-16 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/chain_drop.rs` | modified | 40-47 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/context_scaling.rs` | modified | 58-65 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/engine_scaling.rs` | modified | 24-31 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs` | modified | 1-5,7-31,33-42 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/epoch_contract.rs` | added | 1-276 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/index.rs` | modified | 3-10 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/logical_totals.rs` | modified | 2-11,43-50,57-63,66-72,89-97,108-115 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/mod.rs` | modified | 2-11 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/owner_filter.rs` | added | 1-31 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs` | modified | 91-98 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/profiling/tests/record_engine.rs` | modified | 11-18,54-61,77-84,97-104,112-118,135-143,178-185 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/suspension/core/profile_queue/tests.rs` | modified | 2-9,81-88 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/tests/compiled_library.rs` | added | 1-96 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-runtime/src/runtime/tests/mod.rs` | modified | 125-131 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | modified | 554-562,762-777,789-847 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | modified | 1332-1342,1354-1373,1376-1386,1558-1595 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | modified | 1507-1520,1550-1557 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | modified | 57-83 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/catalog.rs` | modified | 57-63,95-101,129-135 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs` | modified | 31-37,181-187,220-226,301-307,338-352,360-366,445-451,482-496,517-524,528-534,542-548 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs` | modified | 86-98,148-154 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/methods.rs` | modified | 14-20,22-29,31-37 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/sample.rs` | modified | 21-33 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/consumers.rs` | modified | 76-82 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/effects/call.rs` | modified | 78-84,90-96,99-105,115-121 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/effects.rs` | modified | 128-144,146-163 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | modified | 1-69 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | modified | 1-3 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/leaves.rs` | modified | 197-203 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/admission_tests.rs` | modified | 110-119,121-140 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/calls.rs` | modified | 145-151,156-163,170-182,292-298 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/certificates.rs` | modified | 144-150 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs` | modified | 54-60 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/flow/mod.rs` | modified | 18-28,36-42 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/interface.rs` | modified | 35-48,50-63,65-105,107-119,137-148,151-157,162-179,211-226,228-235,244-249 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/mod.rs` | modified | 60-66,111-117,124-130,133-158,164-172,187-195 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs` | modified | 4-10,32-38,52-58,62-68,70-93,129-139 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/option_domain.rs` | modified | 40-52,55-70,93-99,164-170,186-198,216-222,300-311,316-323,328-334 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/oracle/mod.rs` | modified | 33-41,61-67,89-95,106-112 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/production.rs` | modified | 5-15 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/promotion.rs` | modified | 7-15,175-205 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/promotion_tests.rs` | modified | 2-11,116-131 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/protocol.rs` | modified | 165-175,202-208 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/reads/contracts.rs` | modified | 14-20,30-36,46-53 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/reads/oracle.rs` | modified | 10-16,110-116,160-166 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/reads/tests.rs` | modified | 12-19,39-45,84-90,92-98,120-126,160-172,220-226,250-256,278-286,288-294,300-307 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/resume_macro.rs` | modified | 151-158,201-207 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/rules.rs` | modified | 83-90,98-106,117-132,174-191,213-224 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs` | modified | 15-21,61-72,77-89,108-114,141-147,152-159,199-205,238-248 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/tests.rs` | modified | 147-153,180-186,212-218,270-276,321-327 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/local/type_tests.rs` | modified | 92-106,158-170,190-197,200-206,221-229,231-237,269-288 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/methods/context.rs` | modified | 33-49,53-63,84-90,162-168 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/methods/mod.rs` | modified | 109-115,120-153,169-186,219-235 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/methods/traits.rs` | modified | 71-77,102-108 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/method_tests.rs` | modified | 91-97 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/model.rs` | modified | 4-10,69-244,262-351 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/mod.rs` | modified | 253-259 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/mutations.rs` | modified | 30-40 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/regions.rs` | modified | 70-76,123-129,157-163,348-354 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/resolve.rs` | modified | 294-300,330-336,496-507,534-549,552-564 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/source.rs` | modified | 48-54,132-142,469-485,491-497,508-519,536-545,557-566,570-583,616-622,624-639,643-649,653-659 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs` | modified | 243-250 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/types/context.rs` | modified | 32-38 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/types/exact.rs` | modified | 61-68,174-186,264-270,302-308 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/types/modes/projections.rs` | modified | 47-54,67-73 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/types/modes.rs` | modified | 105-113,118-125,137-143,174-182 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/src/tests/retention_closure/types.rs` | modified | 74-80,109-116,121-127,131-137,167-181 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/tests/typed_slot_census/discovery.rs` | modified | 55-64 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/tests/typed_slot_census/scan/sites.rs` | modified | 11-41,73-80 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/tests/typed_slot_census/scan.rs` | modified | 69-76 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/tests/typed_slot_census/witnessed.rs` | modified | 178-186,195-201,205-211,287-293,295-300 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/tree-final/crates/rum-stdlib/tests/typed_slot_census.rs` | modified | 1-29,84-90,97-110,129-139,181-199,202-213,224-235,250-259 | scratch calibration of the brief-633 designs; see logs/final/integrated.diff |
| `.temp/brief-633/PROGRESS.md` | modified | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/drafts/brief-profiler-test-lease.md` | added | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/drafts/brief-private-scratch.md` | added | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/drafts/brief-compiled-library.md` | added | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/drafts/brief-shared-parse.md` | added | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/drafts/brief-class-wide-isolation.md` | added | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/inventory/e2-final-option-i.tsv` | added | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/inventory/e2-shared-option.tsv` | added | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/inventory/e5-predicted-peaks.tsv` | added | 1 | design evidence, implementation draft, checkpoint or report |
| `.temp/brief-633/report-c3.json` | added | 1 | design evidence, implementation draft, checkpoint or report |

- Disclosed: `.temp/brief-633/inventory/e2-options.tsv`: Amendment-3 disclosure: pre-amendment option and E1 memory measurements ran in the worker shared 16G scope. Non-killed sequential rows completed before any cap kill and could not have been touched by one; the listed option ii/iii t16/t32 kills were themselves affected and stand only as >cap results. A shared-scope kill also killed the worker, so no concurrent measurement was trusted.
- Disclosed: `.temp/brief-633/logs/final-options/option-i-final-t4.log`: The first final t4 process peak included initial probe compilation (1580687360 bytes). A single disclosed warm t4 rerun is used for process HWM; its worker RSS is 485285888. No other final option row was rerun.
- Disclosed: `.temp/brief-633/logs/final/retention-gate-r01.log`: Source-only option (i) reached the 14G cap in the real eight-thread suite. The candidate was changed to share production/population Interfaces (option iv); the capped run was not repeated unchanged.
- Disclosed: `.temp/brief-633/logs/final/workspace-r01-r02-failed.log`: The first complete support-ready workspace was cap-killed in rum-stdlib at 32. The candidate was then changed to project signature-only Source fields without deep-cloning discarded bodies and to compute each function syntax once. The failed full log is preserved; no larger thread count was attempted.
- Disclosed: `.temp/brief-633/logs/final/workspace-r01-r03-failed.log`: Selective interface construction alone still hit the 14G cap at 32. The final candidate therefore wraps every heavy Source map/vector in Shared<T>, cloning only the map being mutated. The failed full log is preserved.
- Disclosed: `.temp/brief-633/logs/shared-options/option-i-shared-mutation-copy-r05.log`: The mutation probe proves ptr_before=true and ptr_after=false with the body changed only in the mutant. Handle and retained-delta measurements are process-specific; the probe and pointer helper methods were removed before the workspace gate.
- Disclosed: `.temp/brief-633/logs/final/ownership-replay-r02.log`: Ownership replay normalizes only embedded `ends at line:column` source coordinates when comparing identities. No row is waived; old identities must remain, new host payloads are forbidden, and new constructs must be in touched files.
- Disclosed: `.temp/brief-633/logs/final/acceptance-default-r05.log`: Certificate first passes intentionally stop on stale cells after writing zero-rejection review output; this is the documented replay, not a weakened assertion. Both builds are rerun green after generated-cell merge.
- Disclosed: `.temp/brief-633/logs/final/census-pairing.log`: Putting the lease check in lexical_profile_enabled added two cfg-visible census rows. It was removed; recording-boundary checks preserve the span falsifier and the pinned 1040-row frontier.
- Disclosed: `.temp/brief-633/logs/final/typed-slot-t32-r02.log`: Sharing one Interfaces call removed a source reference and changed typed ordinals. A debug assertion restored the reference but shifted a method ordinal; the final behavior-free shared-reference type ascription preserves both reference57 and method9. No typed receipt inventory was edited.
- Disclosed: `.temp/brief-633/logs/final/lp01-r01.log`: The first LP01 attempt failed before its checks because the scratch child-log directory was absent. Creating .temp/brief-625 in the scratch tree was the only setup change; LP01 then passed all five modes with unchanged inventories.

| Run | Try | Exit | Tests | Log |
| --- | ---: | ---: | --- | --- |
| c3-001-chain-pair-r01 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/c3/chain-pair-r01.log` |
| c3-002-chain-pair-r02 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/c3/chain-pair-r02.log` |
| c3-003-chain-pair-r03 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/c3/chain-pair-r03.log` |
| c3-004-chain-pair-r04 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/c3/chain-pair-r04.log` |
| c3-005-chain-pair-r05 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/c3/chain-pair-r05.log` |
| c3-006-host-logical-profiles | 1 | 0 | 4/0/0 | `.temp/brief-633/logs/c3/host-logical-profiles.log` |
| c3-007-lease-public-runtime | 1 | 0 | 3/0/0 | `.temp/brief-633/logs/c3/lease-public-runtime.log` |
| c3-008-lease-reset-held-green | 1 | 0 | - | `.temp/brief-633/logs/c3/lease-reset-held-green.log` |
| c3-009-lease-reset-old-red | 1 | 101 | - | `.temp/brief-633/logs/c3/lease-reset-old-red.log` |
| c3-010-no_cross_attribution | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/c3/no_cross_attribution.log` |
| c3-011-owner_scoped_abandonment | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/c3/owner_scoped_abandonment.log` |
| c3-012-rlib-green-fresh | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/c3/rlib-green-fresh.log` |
| c3-013-rlib-red-fresh | 1 | 101 | 1/1/0 | `.temp/brief-633/logs/c3/rlib-red-fresh.log` |
| c3-014-same_key_additivity | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/c3/same_key_additivity.log` |
| c3-015-syn-once-lock-red | 1 | 101 | - | `.temp/brief-633/logs/c3/syn-once-lock-red.log` |
| c3-016-tls-scope | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/c3/tls-scope.log` |
| c3-017-typed-slot-pair | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/c3/typed-slot-pair.log` |
| c3-018-unregistered-span | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/c3/unregistered-span.log` |
| c3-019-acceptance-default-r01 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r01.log` |
| c3-020-acceptance-default-r02 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r02.log` |
| c3-021-acceptance-default-r03 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r03.log` |
| c3-022-acceptance-default-r04 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-default-r04.log` |
| c3-023-acceptance-default-r05 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r05.log` |
| c3-024-acceptance-default-r06 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-default-r06.log` |
| c3-025-acceptance-default-r07 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r07.log` |
| c3-026-acceptance-default-r08 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r08.log` |
| c3-027-acceptance-default-r09 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-default-r09.log` |
| c3-028-acceptance-default-r10 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r10.log` |
| c3-029-acceptance-default-r11 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-default-r11.log` |
| c3-030-acceptance-default-r12 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r12.log` |
| c3-031-acceptance-default-r13 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-default-r13.log` |
| c3-032-acceptance-default-r14 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-default-r14.log` |
| c3-033-acceptance-default-r15 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-default-r15.log` |
| c3-034-acceptance-nodefault-r01 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r01.log` |
| c3-035-acceptance-nodefault-r02 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r02.log` |
| c3-036-acceptance-nodefault-r03 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r03.log` |
| c3-037-acceptance-nodefault-r04 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r04.log` |
| c3-038-acceptance-nodefault-r05 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r05.log` |
| c3-039-acceptance-nodefault-r06 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r06.log` |
| c3-040-acceptance-nodefault-r07 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r07.log` |
| c3-041-acceptance-nodefault-r08 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r08.log` |
| c3-042-acceptance-nodefault-r09 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r09.log` |
| c3-043-acceptance-nodefault-r10 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r10.log` |
| c3-044-acceptance-nodefault-r11 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r11.log` |
| c3-045-acceptance-nodefault-r12 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r12.log` |
| c3-046-acceptance-nodefault-r13 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/acceptance-nodefault-r13.log` |
| c3-047-census-pairing | 1 | 0 | - | `.temp/brief-633/logs/final/census-pairing.log` |
| c3-048-check-arc-source-r01 | 1 | 101 | - | `.temp/brief-633/logs/final/check-arc-source-r01.log` |
| c3-049-check-shared-source-r02 | 1 | 101 | - | `.temp/brief-633/logs/final/check-shared-source-r02.log` |
| c3-050-check-shared-source-r03 | 1 | 0 | - | `.temp/brief-633/logs/final/check-shared-source-r03.log` |
| c3-051-check-stdlib-r01 | 1 | 101 | - | `.temp/brief-633/logs/final/check-stdlib-r01.log` |
| c3-052-check-stdlib-r02 | 1 | 101 | - | `.temp/brief-633/logs/final/check-stdlib-r02.log` |
| c3-053-check-stdlib-r03 | 1 | 101 | - | `.temp/brief-633/logs/final/check-stdlib-r03.log` |
| c3-054-check-stdlib-r04 | 1 | 101 | - | `.temp/brief-633/logs/final/check-stdlib-r04.log` |
| c3-055-check-stdlib-r05 | 1 | 101 | - | `.temp/brief-633/logs/final/check-stdlib-r05.log` |
| c3-056-check-stdlib-r06 | 1 | 0 | - | `.temp/brief-633/logs/final/check-stdlib-r06.log` |
| c3-057-check-stdlib-r07 | 1 | 101 | - | `.temp/brief-633/logs/final/check-stdlib-r07.log` |
| c3-058-check-stdlib-r08 | 1 | 101 | - | `.temp/brief-633/logs/final/check-stdlib-r08.log` |
| c3-059-check-stdlib-r09 | 1 | 0 | - | `.temp/brief-633/logs/final/check-stdlib-r09.log` |
| c3-060-fmt-check-r02 | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-check-r02.log` |
| c3-061-fmt-check-r03 | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-check-r03.log` |
| c3-062-fmt-check-r04 | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-check-r04.log` |
| c3-063-fmt-check | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-check.log` |
| c3-064-fmt-r01 | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-r01.log` |
| c3-065-fmt-r02 | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-r02.log` |
| c3-066-fmt-r03 | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-r03.log` |
| c3-067-fmt-r04 | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-r04.log` |
| c3-068-fmt-r05 | 1 | 0 | - | `.temp/brief-633/logs/final/fmt-r05.log` |
| c3-069-inventory-regenerate-r01 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r01.log` |
| c3-070-inventory-regenerate-r02 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r02.log` |
| c3-071-inventory-regenerate-r03 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r03.log` |
| c3-072-inventory-regenerate-r04 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r04.log` |
| c3-073-inventory-regenerate-r05 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r05.log` |
| c3-074-inventory-regenerate-r06 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r06.log` |
| c3-075-inventory-regenerate-r07 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r07.log` |
| c3-076-inventory-regenerate-r08 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r08.log` |
| c3-077-inventory-regenerate-r09 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/inventory-regenerate-r09.log` |
| c3-078-legacy-body-r02 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r02.log` |
| c3-079-legacy-body-r03 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r03.log` |
| c3-080-legacy-body-r04 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r04.log` |
| c3-081-legacy-body-r05 | 1 | 101 | - | `.temp/brief-633/logs/final/legacy-body-r05.log` |
| c3-082-legacy-body-r06 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r06.log` |
| c3-083-legacy-body-r07 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r07.log` |
| c3-084-legacy-body-r08 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r08.log` |
| c3-085-legacy-body-r09 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r09.log` |
| c3-086-legacy-body-r10 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r10.log` |
| c3-087-legacy-body-r11 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body-r11.log` |
| c3-088-legacy-body-r12 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/legacy-body-r12.log` |
| c3-089-legacy-body | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/legacy-body.log` |
| c3-090-lp01-r01 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/lp01-r01.log` |
| c3-091-lp01-r02 | 1 | 0 | 6/0/0 | `.temp/brief-633/logs/final/lp01-r02.log` |
| c3-092-macro-mutations-r13 | 1 | 101 | - | `.temp/brief-633/logs/final/macro-mutations-r13.log` |
| c3-093-macro-mutations-r14 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/final/macro-mutations-r14.log` |
| c3-094-macro-mutations-r15 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/final/macro-mutations-r15.log` |
| c3-095-merge-promoted-r02 | 1 | 0 | - | `.temp/brief-633/logs/final/merge-promoted-r02.log` |
| c3-096-merge-promoted-r03 | 1 | 0 | - | `.temp/brief-633/logs/final/merge-promoted-r03.log` |
| c3-097-merge-promoted-r04 | 1 | 0 | - | `.temp/brief-633/logs/final/merge-promoted-r04.log` |
| c3-098-merge-promoted-r05 | 1 | 0 | - | `.temp/brief-633/logs/final/merge-promoted-r05.log` |
| c3-099-merge-promoted-r06 | 1 | 0 | - | `.temp/brief-633/logs/final/merge-promoted-r06.log` |
| c3-100-merge-promoted | 1 | 0 | - | `.temp/brief-633/logs/final/merge-promoted.log` |
| c3-101-ownership-discovery-r01 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/ownership-discovery-r01.log` |
| c3-102-ownership-discovery-r02 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/ownership-discovery-r02.log` |
| c3-103-ownership-discovery-r03 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/ownership-discovery-r03.log` |
| c3-104-ownership-discovery-r04 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/ownership-discovery-r04.log` |
| c3-105-ownership-discovery-r05 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/ownership-discovery-r05.log` |
| c3-106-ownership-discovery-r06 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/ownership-discovery-r06.log` |
| c3-107-ownership-exact-r01 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/final/ownership-exact-r01.log` |
| c3-108-ownership-exact-r02 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/final/ownership-exact-r02.log` |
| c3-109-ownership-exact-r03 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/final/ownership-exact-r03.log` |
| c3-110-ownership-exact-r04 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/final/ownership-exact-r04.log` |
| c3-111-ownership-exact-r05 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/final/ownership-exact-r05.log` |
| c3-112-ownership-exact-r06 | 1 | 0 | 2/0/0 | `.temp/brief-633/logs/final/ownership-exact-r06.log` |
| c3-113-ownership-replay-r01 | 1 | 1 | - | `.temp/brief-633/logs/final/ownership-replay-r01.log` |
| c3-114-ownership-replay-r02 | 1 | 1 | - | `.temp/brief-633/logs/final/ownership-replay-r02.log` |
| c3-115-ownership-replay-r03 | 1 | 0 | - | `.temp/brief-633/logs/final/ownership-replay-r03.log` |
| c3-116-ownership-replay-r04 | 1 | 0 | - | `.temp/brief-633/logs/final/ownership-replay-r04.log` |
| c3-117-ownership-replay-r05 | 1 | 0 | - | `.temp/brief-633/logs/final/ownership-replay-r05.log` |
| c3-118-ownership-replay-r06 | 1 | 0 | - | `.temp/brief-633/logs/final/ownership-replay-r06.log` |
| c3-119-ownership-replay-r07 | 1 | 0 | - | `.temp/brief-633/logs/final/ownership-replay-r07.log` |
| c3-120-ownership-replay-r08 | 1 | 0 | - | `.temp/brief-633/logs/final/ownership-replay-r08.log` |
| c3-121-read-oracle-r04 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/read-oracle-r04.log` |
| c3-122-read-oracle-r05 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/read-oracle-r05.log` |
| c3-123-read-oracle-r06 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/read-oracle-r06.log` |
| c3-124-retention-census-dump-base | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/retention-census-dump-base.log` |
| c3-125-retention-census-dump-r03 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/retention-census-dump-r03.log` |
| c3-126-retention-census-r02 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/retention-census-r02.log` |
| c3-127-retention-census-r04 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/retention-census-r04.log` |
| c3-128-retention-gate-r01 | 1 | 101 | - | `.temp/brief-633/logs/final/retention-gate-r01.log` |
| c3-129-retention-gate-r02 | 1 | 101 | 56/3/1 | `.temp/brief-633/logs/final/retention-gate-r02.log` |
| c3-130-retention-gate-r03 | 1 | 0 | 59/0/1 | `.temp/brief-633/logs/final/retention-gate-r03.log` |
| c3-131-retention-three-r03 | 1 | 101 | 2/1/0 | `.temp/brief-633/logs/final/retention-three-r03.log` |
| c3-132-typed-baseline-r06 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/typed-baseline-r06.log` |
| c3-133-typed-references-r05 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/final/typed-references-r05.log` |
| c3-134-typed-slot-r01 | 1 | 101 | 2/6/1 | `.temp/brief-633/logs/final/typed-slot-r01.log` |
| c3-135-typed-slot-r02 | 1 | 101 | 2/6/1 | `.temp/brief-633/logs/final/typed-slot-r02.log` |
| c3-136-typed-slot-r03 | 1 | 101 | 2/6/1 | `.temp/brief-633/logs/final/typed-slot-r03.log` |
| c3-137-typed-slot-r04 | 1 | 101 | 2/6/1 | `.temp/brief-633/logs/final/typed-slot-r04.log` |
| c3-138-typed-slot-r07 | 1 | 0 | 8/0/1 | `.temp/brief-633/logs/final/typed-slot-r07.log` |
| c3-139-typed-slot-r08 | 1 | 0 | 8/0/1 | `.temp/brief-633/logs/final/typed-slot-r08.log` |
| c3-140-typed-slot-t32-r02 | 1 | 101 | 2/6/1 | `.temp/brief-633/logs/final/typed-slot-t32-r02.log` |
| c3-141-typed-slot-t32-r03 | 1 | 0 | 8/0/1 | `.temp/brief-633/logs/final/typed-slot-t32-r03.log` |
| c3-142-typed-slot-t32 | 1 | 101 | 2/6/1 | `.temp/brief-633/logs/final/typed-slot-t32.log` |
| c3-143-unknown-pattern-r02 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/unknown-pattern-r02.log` |
| c3-144-unregistered-span-r02 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final/unregistered-span-r02.log` |
| c3-145-workspace-driver-r02 | 1 | 101 | - | `.temp/brief-633/logs/final/workspace-driver-r02.log` |
| c3-146-workspace-driver-r03 | 1 | 101 | - | `.temp/brief-633/logs/final/workspace-driver-r03.log` |
| c3-147-workspace-driver-r04 | 1 | 0 | - | `.temp/brief-633/logs/final/workspace-driver-r04.log` |
| c3-148-workspace-driver | 1 | 0 | - | `.temp/brief-633/logs/final/workspace-driver.log` |
| c3-149-workspace-r01-r02-failed | 1 | 101 | 2897/0/8 | `.temp/brief-633/logs/final/workspace-r01-r02-failed.log` |
| c3-150-workspace-r01-r03-failed | 1 | 101 | 2897/0/8 | `.temp/brief-633/logs/final/workspace-r01-r03-failed.log` |
| c3-151-workspace-r01-support-failures | 1 | 101 | 0/2/0 | `.temp/brief-633/logs/final/workspace-r01-support-failures.log` |
| c3-152-workspace-r01 | 1 | 0 | 4008/0/61 | `.temp/brief-633/logs/final/workspace-r01.log` |
| c3-153-workspace-r02 | 1 | 0 | 4008/0/61 | `.temp/brief-633/logs/final/workspace-r02.log` |
| c3-154-workspace-r03 | 1 | 0 | 4008/0/61 | `.temp/brief-633/logs/final/workspace-r03.log` |
| c3-155-driver-r02 | 1 | 0 | - | `.temp/brief-633/logs/final-options/driver-r02.log` |
| c3-156-option-i-final-mutation-copy | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final-options/option-i-final-mutation-copy.log` |
| c3-157-option-i-final-t16 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final-options/option-i-final-t16.log` |
| c3-158-option-i-final-t32 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final-options/option-i-final-t32.log` |
| c3-159-option-i-final-t4-r02 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final-options/option-i-final-t4-r02.log` |
| c3-160-option-i-final-t4 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final-options/option-i-final-t4.log` |
| c3-161-option-i-final-t8 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/final-options/option-i-final-t8.log` |
| c3-162-option-i-full-model-equivalence-r02 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/options/option-i-full-model-equivalence-r02.log` |
| c3-163-option-i-full-model-equivalence-r03 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/options/option-i-full-model-equivalence-r03.log` |
| c3-164-option-i-full-model-equivalence-r04 | 1 | 101 | - | `.temp/brief-633/logs/options/option-i-full-model-equivalence-r04.log` |
| c3-165-option-i-full-model-equivalence-r05 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option-i-full-model-equivalence-r05.log` |
| c3-166-option-i-full-model-equivalence | 1 | 101 | - | `.temp/brief-633/logs/options/option-i-full-model-equivalence.log` |
| c3-167-option_i_shared_sync_source-t16 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_i_shared_sync_source-t16.log` |
| c3-168-option_i_shared_sync_source-t32 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_i_shared_sync_source-t32.log` |
| c3-169-option_i_shared_sync_source-t4 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_i_shared_sync_source-t4.log` |
| c3-170-option_i_shared_sync_source-t8 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_i_shared_sync_source-t8.log` |
| c3-171-option_ii_parse_per_worker-t16 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_ii_parse_per_worker-t16.log` |
| c3-172-option_ii_parse_per_worker-t32 | 1 | 101 | - | `.temp/brief-633/logs/options/option_ii_parse_per_worker-t32.log` |
| c3-173-option_ii_parse_per_worker-t4 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_ii_parse_per_worker-t4.log` |
| c3-174-option_ii_parse_per_worker-t8 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_ii_parse_per_worker-t8.log` |
| c3-175-option_iii_shared_text_reparse-t16 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_iii_shared_text_reparse-t16.log` |
| c3-176-option_iii_shared_text_reparse-t32 | 1 | 101 | - | `.temp/brief-633/logs/options/option_iii_shared_text_reparse-t32.log` |
| c3-177-option_iii_shared_text_reparse-t4 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_iii_shared_text_reparse-t4.log` |
| c3-178-option_iii_shared_text_reparse-t8 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_iii_shared_text_reparse-t8.log` |
| c3-179-option_iv_mutation_copy | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_iv_mutation_copy.log` |
| c3-180-option_iv_shared_sync_interfaces-t16 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_iv_shared_sync_interfaces-t16.log` |
| c3-181-option_iv_shared_sync_interfaces-t32 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_iv_shared_sync_interfaces-t32.log` |
| c3-182-option_iv_shared_sync_interfaces-t4 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_iv_shared_sync_interfaces-t4.log` |
| c3-183-option_iv_shared_sync_interfaces-t8 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/options/option_iv_shared_sync_interfaces-t8.log` |
| c3-184-options-no-run-r02 | 1 | 101 | - | `.temp/brief-633/logs/options-no-run-r02.log` |
| c3-185-options-no-run-r03 | 1 | 0 | - | `.temp/brief-633/logs/options-no-run-r03.log` |
| c3-186-options-no-run | 1 | 101 | - | `.temp/brief-633/logs/options-no-run.log` |
| c3-187-c3-cert-default | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/parse/c3-cert-default.log` |
| c3-188-c3-cert-no-default | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/parse/c3-cert-no-default.log` |
| c3-189-c3-merge-promoted | 1 | 0 | - | `.temp/brief-633/logs/parse/c3-merge-promoted.log` |
| c3-190-c3-ownership-production | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-ownership-production.log` |
| c3-191-c3-regenerate-contracts | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-regenerate-contracts.log` |
| c3-192-c3-retention-01-local_option_borrowed_pattern_controls | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-01-local_option_borrowed_pattern_controls.log` |
| c3-193-c3-retention-02-local_option_domain_reconciliation_and_independent_drift_controls | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-02-local_option_domain_reconciliation_and_independent_drift_controls.log` |
| c3-194-c3-retention-03-local_option_contract_controls | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-03-local_option_contract_controls.log` |
| c3-195-c3-retention-04-finite_profile_modes_are_closed_and_selected_at_both_calls | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-04-finite_profile_modes_are_closed_and_selected_at_both_calls.log` |
| c3-196-c3-retention-05-production_body_domain_and_build_expiry_and_raw_local | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-05-production_body_domain_and_build_expiry_and_raw_local.log` |
| c3-197-c3-retention-06-local_option_read_contract_controls | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-06-local_option_read_contract_controls.log` |
| c3-198-c3-retention-07-local_rule_contract_controls | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-07-local_rule_contract_controls.log` |
| c3-199-c3-retention-08-production_rejection_census_is_complete | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-08-production_rejection_census_is_complete.log` |
| c3-200-c3-retention-09-production_obligations_are_exact | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-09-production_obligations_are_exact.log` |
| c3-201-c3-retention-10-local_rule_macro_definition_arms_hygiene_and_raw_frame_controls | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-retention-10-local_rule_macro_definition_arms_hygiene_and_raw_frame_controls.log` |
| c3-202-c3-retention-decomposition-summary | 1 | 0 | 10/0/0 | `.temp/brief-633/logs/parse/c3-retention-decomposition-summary.log` |
| c3-203-c3-retention-setup-missing-toolchain | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/parse/c3-retention-setup-missing-toolchain.log` |
| c3-204-c3-retention-setup-stale-fingerprint | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/parse/c3-retention-setup-stale-fingerprint.log` |
| c3-205-c3-stdlib-no-run | 1 | 0 | - | `.temp/brief-633/logs/parse/c3-stdlib-no-run.log` |
| c3-206-c3-typed-01-source_baseline_matches_earned_receipt | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-typed-01-source_baseline_matches_earned_receipt.log` |
| c3-207-c3-typed-02-preflip_no_legacy_transport_uses | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-typed-02-preflip_no_legacy_transport_uses.log` |
| c3-208-c3-typed-03-removing_a_frame_in_a_scratch_copy_is_red | 1 | 0 | 1/1/0 | `.temp/brief-633/logs/parse/c3-typed-03-removing_a_frame_in_a_scratch_copy_is_red.log` |
| c3-209-c3-typed-04-removing_a_test_guard_in_a_scratch_copy_is_red | 1 | 0 | 1/1/0 | `.temp/brief-633/logs/parse/c3-typed-04-removing_a_test_guard_in_a_scratch_copy_is_red.log` |
| c3-210-c3-typed-05-removing_a_witnessed_insertion_in_a_scratch_copy_is_red | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-typed-05-removing_a_witnessed_insertion_in_a_scratch_copy_is_red.log` |
| c3-211-c3-typed-06-witnessing_a_different_binding_in_a_scratch_copy_is_red | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-typed-06-witnessing_a_different_binding_in_a_scratch_copy_is_red.log` |
| c3-212-c3-typed-07-shadowing_a_witness_in_a_cold_helper_is_red | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-typed-07-shadowing_a_witness_in_a_cold_helper_is_red.log` |
| c3-213-c3-typed-08-source__root_set__syntax_root_discovers_added_recursive_type_and_restores_source | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/parse/c3-typed-08-source__root_set__syntax_root_discovers_added_recursive_type_and_restores_source.log` |
| c3-214-c3-typed-09-forced_red_probe | 1 | 0 | 0/0/1 | `.temp/brief-633/logs/parse/c3-typed-09-forced_red_probe.log` |
| c3-215-c3-typed-decomposition-summary | 1 | 0 | 8/0/1 | `.temp/brief-633/logs/parse/c3-typed-decomposition-summary.log` |
| c3-216-c3-typed-no-run | 1 | 0 | - | `.temp/brief-633/logs/parse/c3-typed-no-run.log` |
| c3-217-local_option_borrowed_pattern_controls | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/parse/local_option_borrowed_pattern_controls.log` |
| c3-218-option-i-full-discovery-equivalence-r02 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/typed-options/option-i-full-discovery-equivalence-r02.log` |
| c3-219-option-i-full-discovery-equivalence-r03 | 1 | 101 | 0/1/0 | `.temp/brief-633/logs/typed-options/option-i-full-discovery-equivalence-r03.log` |
| c3-220-option-i-full-discovery-equivalence-r04 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option-i-full-discovery-equivalence-r04.log` |
| c3-221-option-i-full-discovery-equivalence | 1 | 101 | - | `.temp/brief-633/logs/typed-options/option-i-full-discovery-equivalence.log` |
| c3-222-option_i_shared_sync_discovery-t16 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option_i_shared_sync_discovery-t16.log` |
| c3-223-option_i_shared_sync_discovery-t32 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option_i_shared_sync_discovery-t32.log` |
| c3-224-option_i_shared_sync_discovery-t4 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option_i_shared_sync_discovery-t4.log` |
| c3-225-option_i_shared_sync_discovery-t8 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option_i_shared_sync_discovery-t8.log` |
| c3-226-option_ii_discover_per_worker-t16 | 1 | 101 | - | `.temp/brief-633/logs/typed-options/option_ii_discover_per_worker-t16.log` |
| c3-227-option_ii_discover_per_worker-t32 | 1 | 101 | - | `.temp/brief-633/logs/typed-options/option_ii_discover_per_worker-t32.log` |
| c3-228-option_ii_discover_per_worker-t4 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option_ii_discover_per_worker-t4.log` |
| c3-229-option_ii_discover_per_worker-t8 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option_ii_discover_per_worker-t8.log` |
| c3-230-option_iii_shared_text_rediscover-t16 | 1 | 101 | - | `.temp/brief-633/logs/typed-options/option_iii_shared_text_rediscover-t16.log` |
| c3-231-option_iii_shared_text_rediscover-t32 | 1 | 101 | - | `.temp/brief-633/logs/typed-options/option_iii_shared_text_rediscover-t32.log` |
| c3-232-option_iii_shared_text_rediscover-t4 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option_iii_shared_text_rediscover-t4.log` |
| c3-233-option_iii_shared_text_rediscover-t8 | 1 | 0 | 1/0/0 | `.temp/brief-633/logs/typed-options/option_iii_shared_text_rediscover-t8.log` |
| c3-234-typed-options-no-run | 1 | 0 | - | `.temp/brief-633/logs/typed-options-no-run.log` |

| Check | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| census | brief-627 families re-censused at HEAD | rows 205; FAMILY chain-pair runs=20 nonzero=19 exits=[101, 101, 101, 101, 101, 101, 101, 101, 101, 101, 101, 101, 101, 101, 101, 0, 101, 101, 101, 101] peaks=[30306304, 29896704, 29945856, 29843456, 29671424, 30318592, 30326784, 29859840, 29806592, 29622272, 30003200, 30306304, 30113792, 29990912, 30363648, 29851648, 30162944, 29794304, 29847552, 29888512]; FAMILY jupyter-pair runs=20 nonzero=0 exits=[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] peaks=[38342656, 38313984, 38494208, 38457344, 38277120, 38068224, 38039552, 38342656, 38195200, 38273024, 37703680, 38047744, 38178816, 38346752, 38264832, 38518784, 37888000, 38260736, 38359040, 38412288] | PASS | evidence verified |
| inventories | class-wide global, path and whole-tree census | B1=214; B2=11/two; B3=67 | PASS | evidence verified |
| lease | chain pair and unregistered span are green at recording boundaries | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1227 filtered out; finished in 0.07s; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1228 filtered out; finished in 0.01s | PASS | evidence verified |
| c4p | single-epoch concurrency contract | three C4′ tests re-verified green | PASS | evidence verified |
| scratch | private typed tree and constructed rlib | test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 54.06s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1227 filtered out; finished in 14.81s | PASS | evidence verified |
| certificates | generated fingerprints and both certificate builds | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 33.79s; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 33.74s | PASS | evidence verified |
| ownership | discovery-driven ownership replay | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1162 filtered out; finished in 6.67s | PASS | evidence verified |
| retention | full retention suite with shared source and interfaces | test result: ok. 59 passed; 0 failed; 1 ignored; 0 measured; 1104 filtered out; finished in 269.90s | PASS | evidence verified |
| typed-default | typed target at host default | test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 54.06s | PASS | evidence verified |
| lp01 | LP01 producer, binding and loss inventories | disabled: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 3.50s | PASS | evidence verified |
| workspace | class-wide default-thread falsifier | test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s \| test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s \| test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s | PASS | evidence verified |
| format | scratch tree formatting | FMT_CHECK_OK | PASS | evidence verified |

| Measurement | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| retention-options | measure Sync source, per-worker, shared-text and shared-interface memory at 4/8/16/32 | final Shared source RSS 486137856/486699008/490737664/493821952 at 4/8/16/32; per-worker 2.50/4.97/10.02GB then cap at 32; shared text 2.45/4.88/9.77GB then cap at 32; shared interfaces 1.237/1.238/1.241/1.245GB | PASS | Source-only sharing was rejected as a complete suite fix after the real eight-thread cap kill; shared production/population interfaces and copy-on-write Source maps are included in option (iv). |
| typed-options | measure shared discovery, per-worker and shared-text memory | shared discovery 1.183/1.184/1.187/1.192GB at 4/8/16/32; per-worker 4.70/9.38GB then cap at 16; shared text 4.71/9.40GB then cap at 16 | PASS |  |
| mutation-copy | measure an explicit Source handle clone and actual copy-on-write map mutation | Source handle clone delta 0; actual map mutation delta 131072 bytes; pointer identity changes and body changes only in the mutant | PASS |  |
| equivalence | fresh parse equals Sync projection | retention 7,660 functions/50,584 keys Ok; typed 16,129 functions/321,701 uses/647 frames Ok | PASS |  |
| decomposition | ten retention and nine typed peaks sum to their components plus residual | all 19 rows complete; residuals -1245184..0 bytes | PASS |  |
| peak-prediction | predict ten largest post-fix peaks | upper bounds 1425629184..2981416960 bytes after replacing repeated source parses; shared interfaces can lower them further | PASS |  |
| workspace-memory | record final process peaks | see each workspace log maximum resident set size and scope oom_kill=0 | PASS |  |

Deviations:
- Option (i) was revised to option (iv) after a real-suite cap kill.: The low shared-source probe did not include repeated Interfaces construction. Shared production and test-population indexes are required by the measured full-suite behavior.
- Multiple failed conversion, compile, invocation and replay attempts are retained.: Nested byte ranges, generic erasure, serialized-string tokens, synthetic-body spans, an invalid or-pattern, a duplicate --exact, a missing log directory and a reducer key bug were fixed rather than hidden.
- The first workspace support-failure log was overwritten by the rerun.: The rerun reused workspace-r01.log before a backup was made. A clearly marked partial log is preserved at workspace-r01-support-failures.log; the first attempt is not represented as a complete log. The support rerun copied only check-host-payload-destruction.py, README.md, ARCHITECTURE.md, DISCLAIMER.md and TODO.md into the scratch tree.
- Only nine named native ZeroMQ tests are excluded from workspace counts.: They fail serially with EPERM on loopback. No other exclusion, skip environment, retry, sleep or serial target is used.
- No repository source was modified.: All code probes, generated inventories, logs and drafts are confined to .temp/brief-633; run-1 report.json and the absent report-c2.json were not touched.

Open questions:
- **default-threads.** Which libtest default should the director take to the user after these structural fixes? Recommendation: Measure 4, 8, 16 and 32 again with the final shared Source+Interfaces design. Source-only memory is flat, but the full suite now also shares indexes; do not choose from the old OOM data.
- **repr-profile-future.** What if a future test enables RUM_PROFILE_REPR? Recommendation: Use a typed lease or child process. No current test enables it, so this is not a default-suite failure and no new global lock is drafted.

### A — findings at HEAD

Run 1 stands and was not rerun. Its 205 rows record chain pair 19/20 red, typed pair 2/10 red, rum-runtime lib 3/3 red, Jupyter interrupt 20/20 green (gone after brief-629), and nine serial native ZeroMQ EPERM failures excluded environmentally. Full target repetitions and peak RSS are in `inventory/census-summary.txt` and `census.jsonl`; run-1 `report.json` is unchanged. No stop condition fired: C4 was replaced by C4′, whose three contract tests are green.

### B — class-wide inventories

The complete machine-readable tables are `inventory/b1-final.tsv` (214 rows with writers/readers/sites), `b2-final.tsv` (11 groups), and `b3-final.tsv` (67 tests with peaks/shared input). B1 verdicts: 72 isolated, 74 shared-safe, 68 conditional collisions. The 58 lexical globals plus epoch/budget/flags use the lease; six repr counters stay disabled because no test sets RUM_PROFILE_REPR; the remaining global profiling rows are child-process or lease cases. B2 has exactly two collisions: `.temp/typed-slot-census/forced-red/tree` at `crates/rum-stdlib/tests/typed_slot_census.rs:111,:199`, and `target/debug/deps/librum_runtime-*.rlib` at `crates/rum-runtime/src/runtime/accounting_mode/tests/compile.rs:18-37` and `runtime/options/tests/compile.rs:5-15`. Other B2 groups are isolated or disjoint by constructed pid/counter/kind. B3 classes: 33 retention, eight typed, nine ownership, five retention-interface, ten source-walk, one workspace-fingerprint and one association-walk.

Decomposition tables:

| test | baseline | production_or_discover | interfaces | mutants | body | residual | peak | result | phases |
|---|---|---|---|---|---|---|---|---|---|
| c3-retention-01-local_option_borrowed_pattern_controls.log | 7847936 | 664268800 | 144642048 | 1634852864 | 676134912 | 0 | 3127746560 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 17.48s | production:664268800;source-clone:509554688;interfaces-1:559181824;source-clone:562651136;interfaces-2:610287616;source-clone:562647040;interfaces-3:610025472 |
| c3-retention-02-local_option_domain_reconciliation_and_independent_drift_controls.log | 7680000 | 663916544 | 0 | 0 | 2202525696 | 0 | 2874122240 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 12.11s | production:663916544 |
| c3-retention-03-local_option_contract_controls.log | 7954432 | 664276992 | 50425856 | 509538304 | 1466249216 | 0 | 2698444800 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 16.61s | production:664276992;source-clone:509538304;interfaces-1:559964160 |
| c3-retention-04-finite_profile_modes_are_closed_and_selected_at_both_calls.log | 7737344 | 664637440 | 0 | 0 | 1636294656 | 0 | 2308669440 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 15.04s | production:664637440 |
| c3-retention-05-production_body_domain_and_build_expiry_and_raw_local.log | 7901184 | 664125440 | 101040128 | 984387584 | 438226944 | -4096 | 2195677184 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 26.83s | production:664125440;source-clone:419241984;interfaces-1:470274048;source-clone:565145600;interfaces-2:613904384;source-clone:0;interfaces-3:1019904;source-clone:0;interfaces-4:229376 |
| c3-retention-06-local_option_read_contract_controls.log | 8065024 | 664027136 | 107937792 | 1072287744 | 340733952 | 0 | 2193051648 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 16.79s | production:664027136;source-clone:509657088;interfaces-1:559984640;source-clone:562630656;interfaces-2:610836480;source-clone:0;interfaces-3:9404416 |
| c3-retention-07-local_rule_contract_controls.log | 7720960 | 664289280 | 50286592 | 509927424 | 903225344 | 0 | 2135449600 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 16.16s | production:664289280;source-clone:509927424;interfaces-1:560214016 |
| c3-retention-08-production_rejection_census_is_complete.log | 7802880 | 664117248 | 49520640 | 414208000 | 747470848 | 0 | 1883119616 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 26.60s | production:664117248;source-clone:414208000;interfaces-1:463728640 |
| c3-retention-09-production_obligations_are_exact.log | 7843840 | 663924736 | 50122752 | 420724736 | 671838208 | 0 | 1814454272 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 18.72s | production:663924736;source-clone:420724736;interfaces-1:470847488 |
| c3-retention-10-local_rule_macro_definition_arms_hygiene_and_raw_frame_controls.log | 7696384 | 663478272 | 49508352 | 510476288 | 340008960 | 0 | 1571168256 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 13.20s | production:663478272;source-clone:510476288;interfaces-1:559984640 |

| test | baseline | production_or_discover | interfaces | mutants | body | residual | peak | result | phases |
|---|---|---|---|---|---|---|---|---|---|
| c3-typed-01-source_baseline_matches_earned_receipt.log | 4050944 | 1177075712 | 0 | 0 | 246603776 | 0 | 1427730432 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 33.40s | discover:1177075712 |
| c3-typed-02-preflip_no_legacy_transport_uses.log | 4034560 | 1177079808 | 0 | 0 | 0 | -163840 | 1180950528 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 20.92s | discover:1177079808 |
| c3-typed-03-removing_a_frame_in_a_scratch_copy_is_red.log | 4063232 | 1177165824 | 0 | 0 | 5742592 | -1245184 | 1185726464 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 51.70s | discover:1177165824;typed-tree-copy:0;typed-mutant-copy:0 |
| c3-typed-04-removing_a_test_guard_in_a_scratch_copy_is_red.log | 3883008 | 1177190400 | 0 | 0 | 5689344 | -380928 | 1186381824 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 52.24s | discover:1177190400;typed-tree-copy:0;typed-mutant-copy:0 |
| c3-typed-05-removing_a_witnessed_insertion_in_a_scratch_copy_is_red.log | 3969024 | 1177051136 | 0 | 0 | 147681280 | 0 | 1328701440 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 37.98s | discover:1177051136;typed-tree-copy:0;typed-mutant-copy:0 |
| c3-typed-06-witnessing_a_different_binding_in_a_scratch_copy_is_red.log | 4091904 | 1177239552 | 0 | 0 | 147959808 | 0 | 1329291264 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 37.87s | discover:1177239552;typed-tree-copy:0;typed-mutant-copy:0 |
| c3-typed-07-shadowing_a_witness_in_a_cold_helper_is_red.log | 4046848 | 1177079808 | 0 | 0 | 147722240 | 0 | 1328848896 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 38.30s | discover:1177079808;typed-tree-copy:0;typed-mutant-copy:0 |
| c3-typed-08-source__root_set__syntax_root_discovers_added_recursive_type_and_restores_source.log | 0 | 0 | 0 | 0 | 555663360 | 0 | 555663360 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 18.10s | none (inline fixture; no workspace discovery) |
| c3-typed-09-forced_red_probe.log | 0 | 0 | 0 | 0 | 30605312 | 0 | 30605312 | test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 8 filtered out; finished in 0.00s | none (ignored body did not execute) |

### C — profiler test lease

C1: run-2-only feature-tree evidence (`logs/tree/rum-cli-features.log`, `harness-features.log`) shows test features only in dev-dependency arms; the perf-panel release build and harness do not compile them. The final registration check is at recording boundaries, not `lexical_profile_enabled`; adding it there produced exactly two new cfg rows and was reverted (`logs/final/census-pairing.log`). The function stays byte-identical and the off path one relaxed load. The implementation gate must still run `run NNN --perf resume,cli`.

C2/C3: private `new_unleased`, `Epoch::for_test(&TestLease,...)`, `reset_for_lease`, private-field `ProductionReset`, and public held-guard `LogicalProfileTestLease` reject old reset/constructor forms. `logs/c3/lease-reset-old-red.log` is the compile-red old form; `lease-reset-held-green.log` compiles the held form. Host lifecycle/abandonment children retain process isolation and hold the wrapper; rum-runtime and rum-stdlib processes use the lease.

C4′: all three unmutated contracts are re-verified once in `logs/c3/`; run-2-only mutant logs remain under `logs/c4p/`. Same-key rows add, distinct owners do not cross-attribute, and abandonment remains owner-scoped through `Session` drop to `Epoch::abandon_owner` (`logical/epoch.rs`). C5: the span falsifier is red at HEAD and green at recording boundaries (`logs/final/unregistered-span-r02.log`); chain pair is green 5/5 re-verified and 20/20 run 2. The TLS join mechanism is proven by `logs/c3/tls-scope.log`; only sampling time changed, not the metadata assertion.

### D — private paths and constructed artifacts

`PrivateScratch::create()` owns a pid+counter directory and has no path-taking constructor; the ignored probe rejects the historical shared path. Red: the HEAD typed pair’s shared tree. Green: full typed target at two and 32 threads. `CompiledLibrary::get()` builds once into a process-private target and returns exact deps/runtime/core paths; both compile-proof tests consume only those accessors. Red at fresh HEAD-shaped target: `missing compiled library rum_runtime`. Green: `logs/c3/rlib-green-fresh.log`. No hardlink or shared deps scan remains. Diffs: `logs/c3/lease-public.diff`, `logs/scratch/typed-slot-census.diff`, `logs/c3/rlib-fix.diff`, and `logs/final/integrated.diff`.

### E — shared Sync discovery and interfaces

The decomposition above accounts for every requested peak and residual. Options (ii) and (iii) grow linearly (t8/t4≈2) and hit their caps; do not rerun amendment-3 capped rows. Option (i) final source is flat at 485–497 MiB, but the real eight-thread suite reached 14G (`logs/final/retention-gate-r01.log`), so source-only sharing is not sufficient. The selected design is option (iv): exact-slice immutable Source plus shared production Interfaces and a separate test-population source/index pair. Shared-interface probe rows are about 1.24GB at 4/8/16/32. Every heavy Source map/vector is wrapped in `Shared<T>`; `Arc::make_mut` clones only the map actually mutated. The final Source handle clone has zero RSS delta, and an actual function-map mutation has a 131072-byte retained delta in the measured process, with pointer inequality proving the copy.

Exact line/column slices preserve nested items, raw synthetic bodies and multiline literals; serialized strings are parsed locally before token comparisons. Full-model and full-discovery equivalence receipts are linked in measurements. Final retention at eight threads and typed at 32 threads are the semantic gates.

Predicted ten largest post-fix peaks (upper bounds):

| test | old_peak_bytes | old_shared_input_bytes | final_shared_hwm_bytes | predicted_peak_upper_bound_bytes | basis |
|---|---|---|---|---|---|
| tests::retention_closure::local::reads::tests::local_option_borrowed_pattern_controls | 3127746560 | 664268800 | 517939200 | 2981416960 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::local::option_domain::local_option_domain_reconciliation_and_independent_drift_controls | 2874122240 | 663916544 | 517939200 | 2728144896 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::local::option_boundaries::tests::local_option_contract_controls | 2698444800 | 664276992 | 517939200 | 2552107008 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::local::type_tests::finite_profile_modes_are_closed_and_selected_at_both_calls | 2308669440 | 664637440 | 517939200 | 2161971200 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::local::promotion_tests::production_body_domain_and_build_expiry_and_raw_local | 2195677184 | 664125440 | 517939200 | 2049490944 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::local::reads::tests::local_option_read_contract_controls | 2193051648 | 664027136 | 517939200 | 2046963712 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::local::rules::tests::local_rule_contract_controls | 2135449600 | 664289280 | 517939200 | 1989099520 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::census::tests::production_rejection_census_is_complete | 1883119616 | 664117248 | 517939200 | 1736941568 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::production_obligations_are_exact | 1814454272 | 663924736 | 517939200 | 1668468736 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |
| tests::retention_closure::local::rules::tests::local_rule_macro_definition_arms_hygiene_and_raw_frame_controls | 1571168256 | 663478272 | 517939200 | 1425629184 | subtract measured repeated parse and substitute final shared HWM; interface sharing can only lower this bound |

### F — implementation drafts and calibrated falsifiers

`drafts/brief-profiler-test-lease.md` — RED `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 0.07s` GREEN `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1227 filtered out; finished in 0.07s`.

`drafts/brief-private-scratch.md` — RED `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 21.69s` GREEN `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 53.42s`.

`drafts/brief-compiled-library.md` — RED `missing compiled library rum_runtime` GREEN `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1227 filtered out; finished in 14.81s`.

`drafts/brief-shared-parse.md` — RED `maximum resident set size: 16801554432` GREEN `test result: ok. 59 passed; 0 failed; 1 ignored; 0 measured; 1104 filtered out; finished in 269.90s` and `test tests::retention_closure::options_probe::option_i_shared_sync_source ... OPTION_PROBE mode=sync-source workers=32 rss=493821952 hwm=515305472`.

`drafts/brief-class-wide-isolation.md` — RED `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 0.07s` GREEN three workspace runs `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s` / `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s` / `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s`.

## Director gate (2026-10-30): design accepted, with corrections

**Scope.** Kimi wrote only under `.temp/brief-633/`, and `git status` is
clean. `report.py check 633 --stage c3` is OK, with 595 citations verified.
Run 3 died at 02:05 on its own measurement OOM (amendment 3). It resumed in
the same session and finished at 08:17 with rc 0, peak 0.54 GB in its own
scope, `oom_kill=0`.

**Reproduced by me.**

- **Integrated scratch tree (`tree-final/`), full workspace at
  `--test-threads=32`**, with the nine native ZeroMQ EPERM tests skipped by
  name, in its own 14G scope (`.temp/director/gate-633/ws32-r01.log`):
  - 59 `test result: ok` lines and 0 FAILED;
  - 4008 passed, 61 ignored;
  - `peak=9001222144 oom_kill=0`, about 7 minutes.

  At HEAD, the same configuration fails: rum-runtime lib is red at 32
  threads, and rum-stdlib at 32 threads reaches its OOM.
- **The options rows** (`logs/typed-options/status.tsv`). I watched the
  orphaned loop run after the worker died:
  - option i is flat at 1.18–1.19 GB from 4 to 32 workers;
  - options ii and iii are 4.7 GB at 4 and 9.4 GB at 8, and are killed at
    the cap at 16 and at 32.
- **The fresh-target rlib red at HEAD**: recorded in brief-634.

**Accepted.**

- The census.
- The B1/B2/B3 inventories (214 globals, 11 path groups, 67 whole-tree
  tests).
- C4′ re-verified: the single epoch adds same-key rows, does not
  cross-attribute, and abandons by owner.
- The TLS mechanism (`probe/tls-scope/`).
- The rlib race as a real finding.
- **Option (iv)** for E:
  - the source discovery is shared immutably;
  - so are the production `Interfaces` and the test-population
    source/index pair;
  - every heavy `Source` map is `Shared<T>`, copy-on-write through
    `Arc::make_mut`;
  - each function's text is stored as an exact slice, re-parsed locally.

  **Option (i), sharing the source alone, was falsified by the real suite**:
  the eight-thread retention suite hit the 14G cap.

**Corrections, written into my implementation briefs.**

1. **Lease (brief-635): two unleased reset entries remain.**
   - `ProductionReset`'s private field does not keep a test from reaching
     it.
     - `RumProfileReset[]` goes through `debug_tools/basic.rs:119`, and
       `runtime/tests/debug_tools.rs:322` evaluates it in-process.
     - `rum_runtime::test_logical_profile_reset()` (`lib.rs:18`) is called by
       `suspension_oracle/descriptors.rs:144`.
   - `reset_lexical_profile_counters` also stores `LEXICAL_PROFILE_ACTIVE = 1`.
   - Three workspace runs that stay green do not show this overlap is
     absent.
   - **Fix:**
     - under the test cfg, `reset_lexical_profile_counters` refuses an
       unregistered thread while a lease is current;
     - `test_logical_profile_reset` takes the lease;
     - a new falsifier: an unregistered `RumProfileReset[]` must leave the
       leased epoch's row in place. It is red without the refusal.
   - **The B1 rows:** each of the 62 `collision` rows gets a named
     protection (child, lease or none). An unprotected in-process reader is
     a STOP.
   - **The warning:** `lease.rs:19` printed a new warning.
2. **Compiled library (brief-634): the draft grew without bound and still
   scanned.**
   - One target directory per process id: 8 directories and 2.1 GB in one
     session.
   - `library(&deps, "rum_core")` still scanned `deps`.
   - **Fix:**
     - one fixed `target/test-compile-library`;
     - `cargo build -p rum-core -p rum-runtime --lib`, so both libraries are
       uplifted to exact `debug/lib<name>.rlib` paths;
     - no `read_dir`;
     - the build bound is measured, not guessed.
3. **Private scratch (brief-634): trees accumulated.**
   - 42 trees and 407 MB, with no cleanup.
   - **Fix:**
     - `PrivateScratch` removes its tree on `Drop`;
     - the path reaches the child through `Command::env`, not a libtest free
       argument;
     - the probe refuses to run without the variable.
4. **Shared parse (brief-636): no code shaped to keep an ordinal.**
   - Tree-final added a behaviour-free type ascription to keep
     `#reference57`/`#method9`. It also chose a macro destructuring to keep
     `legacy_macro_stays_opaque_in_all_four_views#reference7`.
   - **Rule:**
     - Write code naturally.
     - A typed-slot X-row identity that moves inside a function the diff
       edits is updated in `typed-slot-stages.tsv`. The X id, the stage and
       the disposition stay the same, and each before/after pair is
       disclosed.
     - A moved identity in a function the diff does not edit is a STOP.
5. **Shared parse: how equivalence is proved.**
   - The full-model and full-discovery equivalence probes run at the
     implementation gate, and are compared on complete key dumps, not on
     counts.
   - They are not landed: the pinned receipts are the permanent oracle.
   - Measurement-only modules are not landed.

**Order.**

1. brief-634: artifact constructors (D). It removes the rlib flake from the
   lease's whole-lib falsifier.
2. brief-635: the profiler lease (C).
3. brief-636: the shared parse (E).

The class-wide draft becomes my own measurement after brief-636:

- the workspace at 4, 8, 16 and 32 threads, N runs each;
- the default thread count is then the user's ruling.

**Questions.**

- `default-threads`: as above, mine to measure after brief-636.
- `repr-profile-future`: no test sets `RUM_PROFILE_REPR`. A future one uses
  a child process or the lease. Nothing is drafted.
