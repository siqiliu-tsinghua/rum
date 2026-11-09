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
