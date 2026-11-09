# Brief 715 — the byte-delta witnesses: a process-global assertion under parallel execution

**Type:** design-only diagnosis and repair design. **Scope: NONE.** No source,
test, inventory, host or docs edit; no git. All work under
`.temp/brief-715/`; scratch copies and private targets stay there. The
director owns the ruling, the implementation brief and the gate.

## Goal

Run 31's draw 2 failed
`brief713_collector_unwind_releases_owned_boundary` with
`refunded=976 expected=296` at `profile_queue/tests.rs:492`, and the same
binary passed `1413/0/5` at one thread. The director's reading (stage-31
ruling in brief 714) is that the failure is the **witness**, not the product:
the assertion is the delta of the process-global `PROCESS_BYTES` counter,
while the profiler's refunds are not lease-gated and thread-local storage is
refunded at thread exit, which libtest runs concurrently with other tests.
This brief diagnoses that class and designs the repair. It proves or
refutes the reading with measurements; it does not assume it.

## Context pointers

- The failure: `.temp/brief-714/logs/c31-draw-02.log` (lines 1425-1447),
  draw 1 green in `c31-draw-01.log`, the serial reproduction in
  `c31-serial-after-02.log`, the index in `.temp/brief-714/draw-index.json`.
- The witness: `crates/rum-runtime/src/runtime/suspension/core/profile_queue/tests.rs:409-499`
  (charges at `:435/:439`; the two samples at `:451` and `:480`; the
  assertion at `:492`).
- The counter: `crates/rum-runtime/src/runtime/profiling/logical/budget.rs`
  (`PROCESS_BYTES` `:8`; claim `:36/:49`; releases `:64/:79/:89`;
  the thread-metadata lifetime test `:117-160`).
- The thread-local holders: `logical/segments.rs:37-57` (`Stack.storage`,
  `Stack.header` — "TLS registration stays charged until this thread is
  destroyed", `:43-44`), charged at `:452-461`; `release_thread_registration`
  at `:68-79`.
- The lease: `profiling/lease.rs:9/31/236` — `TEST_LEASE` serializes lease
  holders; nothing gates refunds.
- The second witness in the class:
  `profiling/tests/brief713_teardown_paused.rs` (`brief713_probe_process_bytes`
  at `:55/:59`).
- Brief 713's C18 round (the witness's origin and its mutant history) and
  the stage-31 ruling in `docs/dev/codex/brief-714-profiling-isolation-validation.md`.

## Deliverable

1. **Characterise the failure.** On the accepted tree, with one private
   test build at 8 threads, run the runtime lib suite enough times to see
   whether any process-global assertion fails again; for every failure quote
   the value and name the assertion. Record the binary sha and keep every
   log. A green streak does not settle anything; say how many draws you ran.
2. **Prove the mechanism, or refute it.** In a scratch copy, construct a
   controlled injection: a second thread that charges and then drops a
   `Storage` of a known size inside the witness's measurement window (a
   test-only hook is allowed in the copy). Show on the current witness that
   `refunded` moves by exactly the injected size, and that the witness's own
   charges inside its window are exactly 296. Then show the same injection
   does not move a candidate replacement anchor. If the injection cannot be
   built, say so and stop on that point rather than substituting an argument.
3. **Check the product side.** Read every `PROCESS_BYTES` release path and
   say whether any concurrent product-side path can over-release (double
   drop, `resize` underflow, `Budget`/`Storage` ordering). An over-release
   from the product would be a real defect and a STOP; say plainly which it
   is, with the code read.
4. **Inventory the class.** Every assertion in the test suite that compares
   a process-global profiling counter across a window (at least
   `profile_queue/tests.rs:492`, `brief713_teardown_paused.rs`,
   `budget.rs:105/159/185`), with the concurrency it runs under and a
   disposition.
5. **Design the repair.** Propose the isolation mechanism (for example
   lease-generation-scoped accounting under `cfg(test)`, a per-epoch budget
   anchor, or a test-only quiescence gate) with its invariants, why it cannot
   mask a missing refund, its effect on existing constants
   (`RECORD_PATH_STORAGE_BYTES` and the `size_of` assertions in
   `logical/mod.rs:53`), and its default-path cost (must be zero — test-cfg
   only, no new `unsafe`). Name the exact assertions that replace the
   current ones.
6. **Falsifier and mutant plan.** The repaired witness must redden when the
   boundary's storage is not refunded (a named mutant) and must stay green
   under the calibrated injection; give both receipts as a plan the
   implementation brief can require verbatim.
7. **Draft the implementation brief** in `TEMPLATE.md` shape, with the scope
   list, the acceptance bars, the falsifier and the mutant. The director
   rewrites and commits it; drafts go under `.temp/brief-715/`.

## Cost bar and must-not-change list

- No live edit of any kind; scratch copies under `.temp/brief-715/` only.
- Live-tree runs: read-only. A copy builds into its own target dir.
- One cargo command at a time; builds `-j8`; tests at 8 threads; offline and
  locked; the director's workspace run stays his.
- No assertion weakened, no test removed, no skip or filter added, no new
  `unsafe`, no production-path change in any proposal.
- No git, no inventory/census replay (no covered source edit), no host
  suite, no timing/performance lane.

## Verification (you run it)

1. The draw characterisation with copied verdict lines and the binary sha.
2. The injection calibration: the current reading moves by the injected
   size; the candidate anchor does not; the missing-refund mutant reddens
   the candidate.
3. The product-side reading, file and line by file and line.
4. The class inventory table.
5. `report.py check` passes; the final message is `report.py summary`.
   Stop for the director's ruling; a design report authorizes no commit.

## Checkpointing

Every run's log saved before it is parsed; the scratch copy and its target
dir kept; the injection diff kept as a file with its sha. No overwriting of
a completed run.

## Hard rules

`AGENTS.md` and `TEMPLATE.md`; the testing policy is rum-only (this brief
compares rum against itself); no git of any kind, no live edit, no network,
no third-party engine, no new `unsafe`; a red is reproduced at
`--test-threads=1` before it is called a defect; all output under `.temp/`;
the report is `rum-report/1` JSON with copied verdicts and the
command/log/exit/count table.

## Director's ruling — 2026-11-09: the design is accepted with three corrections; brief 716 implements the byte-delta class only

**Gate.** `report.py check` OK, 228 citations. I reproduced the load-bearing
receipts: the live binary is `66950c70a42b05b91c6a06c5d38bc7bd39ce76e078d090f2e91d79374ec56edd`,
the same sha as brief 714's C31 artifact; draws 1-3 are `1413/0/5`, draw 4 is
`1412/1/5` with the identical `refunded=976 expected=296` tuple, and the
serial rerun is green; the twelve calibration rows hold at 8 and at 1 thread
(control 296/296; injection 4392/296; M1 4344/248; restored 4392/296); the
candidate and restored binaries are the same sha `aa0ca813…` and the mutant
is `d0310704…`; `evidence/injection.diff` injects a real `Storage` through
`storage_admitted` and drops it on an unregistered worker — it does not edit
`PROCESS_BYTES`; the live tree is untouched (`git status --porcelain` empty,
and the only files newer than the published commit are the two brief files,
written before it). The two reducer STOPs are instrument defects, settled
without rerunning a suite; the twelve calibration suites each ran once.

**Accepted.** The mechanism is proven, not argued: a legal foreign `Storage`
refund inside the window moves the process reading by exactly its size
(296→4392) while the owning Budget does not move (296), at 8 and at 1 thread;
with the boundary's 48 bytes withheld, the same anchor reads 248 while the
process reading reads 4344. The product release-path audit is accepted as
read — `reserve` rollback, `Storage`/`Budget` drop order, `resize` growth and
shrink arithmetic, `Arc::into_inner` ownership, and the three `resize`
callers — and its precision about empty frame storage being released at the
last take (`logical/segments/stack.rs:182-185`) corrects my own earlier
phrasing. An over-release found during implementation is a STOP, never an
expected-value adjustment. The tiered repair is the right instrument:
ownership claims get the owning Budget, quiescence claims keep the process
counter.

**Three corrections.**

1. **The class predicate is narrower than my stage-31 sentence, and that
   sentence is wrong.** I wrote that `brief713_teardown_paused.rs` and both
   `budget.rs` samples carry the same shape. The inventory proves otherwise:
   B13 and B04 hold a drained lease and join every owned thread, so at the
   first sample `PROCESS_BYTES == 0` — no pending foreign charge can exist to
   be refunded later, and no unregistered thread can create one; those rows
   are valid and stay unchanged. The class is: **a process-global delta whose
   window is not made quiescent by a drained lease plus joined owned
   threads.** `budget.rs:185` (B05) is raw and is in the class;
   `budget.rs:159` (B04) is not. An erratum is appended to brief 714's
   stage-31 ruling.

2. **B09's probe cannot be `None` after `drop(enabled)`.** `TestLease::drop`
   (`lease.rs:236-243`) releases only the thread registration and zeroes the
   generation; the old epoch is taken and retired by `reset_unleased` inside
   the *next* `lease()` (`epoch.rs:1103-1150`: `slot.epoch.take()` then
   `retire_admitted`). The probe-None assertion therefore belongs **after
   `test_support::lease()` returns**, where the handoff reset has actually
   dropped the old Budget. The raw reacquisition stays as the control and the
   growth assertion at `lease_release.rs:22` stays.

3. **The permanent injection lives inside B02, not in a new test.** Brief
   714's stage-32 bar is exactly `1413/0/5` over a fixed name set; a new test
   name would force a new bar. The repaired collector carries the 4096-byte
   foreign refund in its own window and asserts the owned 296, so a
   regression to a process-global anchor reds the first draw. The name set
   stays 1413: no test added, removed, renamed or filtered.

**Q1 — approved.** B01/B02/B05-B09 convert to the owning Budget;
B03/B04/B10-B16 stay exactly as they are, global contracts included.

**Q2 — half.** The global-only-decrement falsifier is bound **now**, as M4:
after this repair seven global deltas leave the raw windows, so the remaining
global coverage must be shown to fire on a broken global decrement, not
merely asserted to. The enabled repr-counter exposure
(`evaluation_cache/tests.rs:185-218` under `RUM_PROFILE_REPR`, no lease) is a
**different class** — admitted counters, per-thread allocation, and no lane
we run sets `RUM_PROFILE_REPR` — so it is not folded in; it is recorded in
`TODO.md` as a named open obligation with its trigger (before any claim of
comprehensive profiling-test isolation).

**Calibration matrix for brief 716** — each mutant in a scratch copy, each
run at 8 and at 1 thread, each predicted red traced to the named assertion,
each restored to the candidate sha:

| # | Mutant | Predicted observer | Predicted tuple |
|---|---|---|---|
| M1 | boundary whole-Storage refund withheld through the observer (715-calibrated) | B02 owned equality | 248 ≠ 296 |
| M2 | the expired arena's Storage withheld through the observer | B05 `probe.bytes() == None` | `Some(…)` after `drop(epoch)` |
| M3 | `release_thread_registration` skipped in `TestLease::drop` | B09 probe after the handoff reset | `Some(header)` |
| M4 | `PROCESS_BYTES.fetch_sub` removed from `Storage::drop`, local kept (focused run of B04 alone) | B04 `PROCESS_BYTES == before` | `before + own ≠ before` |
| M5 | the 32KiB old-epoch charge removed (test-side) | B06 present lower bound | `probe < 32KiB` |
| M6 | the old boundary's refund withheld at B07's site | B07 exact `248 + 48` | 248 ≠ 296 |

**Scope for brief 716** is the seven rows and their plumbing, nothing else:
`logical/budget.rs` (cfg(test) probe and snapshot, and B05),
`logical/epoch.rs` (cfg(test) wrappers), `logical/mod.rs` (cfg(test)
re-export and the helper's `#[path]` mount beside the existing test mounts at
`:1466`/`:1548`), a new `profiling/tests/budget_witness.rs` helper,
`suspension/core/profile_queue/tests.rs` (B01/B02),
`profiling/tests/brief713_cleanup.rs` (B06/B07/B08), and
`profiling/tests/lease_release.rs` (B09), plus standing inventories and
certificates **by replay only**. Product accounting, destruction, leases,
charged fields and every public shape are out; touching them is a STOP.

**Brief 716's acceptance** is: fmt clean; `1413 passed; 0 failed; 5 ignored;
2 filtered` — the unchanged name set — on 3 draws at 8 threads and once at 1;
`check --workspace --all-targets`; the four-build warning matrix 0/0/3/3;
M1-M6 as above; B07's exact expectation from its own receipt or a STOP; the
standing replays before the first acceptance suite; and a fresh release
artifact whose object code is 16/16 identical to the accepted C28 artifact
(`target/release/deps/librum_runtime-c71407e195689fc9.rlib`, sha256
`d9f2339d…`), any object difference being a scope violation.

After my gate on 716, brief 714's 37-draw series restarts **from zero as
stage 32**, same bars, new private build and binary sha. No draw of the
abandoned series is carried.
