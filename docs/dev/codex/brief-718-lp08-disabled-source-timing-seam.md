# Brief 718 — LP08: the disabled source-timing seam

**Type:** implementation. **Scope:** one function's disabled-path shape in
`evaluation_cache/profile/source.rs`, plus the standing inventory replays.
No behaviour change, no test change, no data change.

## Goal

With profiling off, the evaluation-cache source-timing seam
`record_current_source_timing` must return without paying for the enabled
body it guards: **at most 4 instructions per call at unchanged call counts**
on the two campaign labels (target 3, the `record_timing` class), its enabled
body behind a cold outlined helper that the disabled path never calls, and
its caller's per-call self-Ir unchanged. The test-cfg admission and the
admitted flag read keep their order; the enabled path records exactly the
counters and `EventKey::CacheSource` side-index entries it records today.

Brief 717 STOPed on this row: base 22, final 24 per call, +218,154
instructions over the whole workload (brief 717, "Stage-D label STOP ruling
and continuation"). This brief closes it; brief 717 then restarts from
Stage A on the repaired source.

## Context pointers

- The seam: `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs:252`
  (`record_current_source_timing`), `:350`
  (`record_symbol_timing_profile_counter`), and the stage-A side-index block
  at `:383-403` whose inlining into the seam is the cause.
- The target shape already exists beside it:
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:430`
  (`record_symbol_profile_counter`: flag check in the outer, body in a
  `#[cold] #[inline(never)]` helper) and
  `crates/rum-runtime/src/runtime/profiling/counters.rs:4` (`record_timing`,
  the 3-per-call class).
- My calibration (scratch, do not build from it):
  `.temp/director/probe-717e/` — `tree-startfirst` measured **3** Ir per call
  on both labels at 50/100, helper 0 calls, caller 91/125 unchanged;
  `tree-pinned` (cold outline only, outer pinned) measured **18**;
  `tree-atrib` (side-index block removed) measured **22**, the attribution.
  `repro.py` and the raw `.out` captures are there.
- Brief 717's Stage-D ruling (the STOP row, the class scan, the two
  instrument failures) and the 168 label profiles under
  `.temp/brief-717/continuation5/label-probes/`.

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs`, and
  the derived inventory files the standing maintenance duty requires
  (`docs/dev/codex/TEMPLATE.md`, "Every source change carries a standing
  maintenance duty"): the contract fingerprints, both certificate cells, the
  ownership inventory, LP01, typed-slot and the rejection census. Replay
  them canonically; never hand-edit a row; disclose every changed row.
- Files you MUST NOT touch: every test file; `record_symbol_timing_profile_counter`
  and its side-index block; the six accepted seams; any other source file.
  A test edit is a STOP — the repair must be invisible to the suite's
  behaviour.
- **A committed inventory row that would change its identity, status,
  disposition or exclusion, or would disappear, is a STOP with the keyed
  comparison.** New rows for the repair's own added items (a new function, a
  new cfg attribute) are expected, disclosed with file and line.
- No new `unsafe` (the crates stay zero-unsafe), no `#[allow]`, no
  duplicated enabled body, no `#[cfg(debug_assertions)]` gate, no lazy init,
  no silent drop, no change to when or whether anything is recorded.

## Target shape and invariants

My calibrated shape (a different route is acceptable only if the bar below
still holds; if no route can hold it without a test or behaviour change,
STOP and report):

```rust
#[inline(never)]
fn record_current_source_timing(name: &'static str, start: Option<Instant>) {
    // test cfg: the timing frame, then lease::admit(), then the admitted
    // flag read, exactly in today's order and with today's refusal
    // semantics; the start return comes after that block.
    let Some(start) = start else {
        return;
    };
    record_current_source_timing_enabled(/* cfg'd timing, &access, */ name, start);
}

#[cold]
#[inline(never)]
fn record_current_source_timing_enabled(/* cfg'd timing, _access, */ name: &'static str, start: Instant) {
    // production cfg only: if !evaluation_cache_profile_enabled_admitted() { return; }
    let Some(symbol) = current_evaluation_profile_source_symbol_admitted(/* &_access */) else {
        return;
    };
    record_symbol_timing_profile_counter(/* timing, &_access, */ name, symbol, Some(start));
}
```

Invariants (a change to any of these is a STOP):

- The test-cfg order is `take_symbol_timing_frame(start)` → `admit()` →
  admitted flag read → the start check. The unregistered-thread refusal
  semantics are unchanged, and the flag-read witnesses keep their counts.
- The outer stays out of line so its per-call row is measurable by the
  campaign's ledger; the enabled body is `#[cold] #[inline(never)]` and is
  not called on the disabled path.
- The enabled path performs the same operations in the same order and
  records the same counters and side-index entries.
- Call sites, the disabled-path call count and the caller's per-call self-Ir
  are unchanged.

## Cost bar and must-not-change list

- Repaired seam: **≤ 4 Ir per call** (target 3) on both labels, 50/100, all
  four draws, primary and serial; the helper at 0 calls; the caller at 91
  (downvalue) and 125 (width32000) per call.
- The red arm is the frozen final CLI `32df5b02…`: 24 per call, helper n/a,
  caller 91/125.
- Whole-workload ledger on the repaired CLI: the six accepted seam rows
  exact (13→3, 23→18, 18→14, 20→18, 20→18, 18→14 at identical call counts),
  cold helpers 0 self-Ir / 0 calls; the deterministic per-call scan finds no
  growth row in `rum_core`/`rum_runtime` with an unchanged call count beyond
  the two accepted once-called intercepts.
- Tests: the full workspace at 8 threads must read exactly 4,212 passed /
  0 failed / 61 ignored over 63 binaries with the seven standing skips; the
  runtime lib series `1413/0/5` (2 filtered); the focused profiling
  witnesses green at 8 and 1 (unregistered flag-read refusal, lease handoff,
  timing cleanup, `synchronous_families`, publication); the four-build
  warning matrix 0/0/3/3.

## Receipts and measurement protocol

- Arms: base `.temp/brief-717/continuation4/bin/d-cli-base`
  (sha256 `60bd0d3b…`), layout `…/d-cli-layout` (`696ed1ed…`), red
  `.temp/brief-717/continuation2/bin/c-cli-final-1` (`32df5b02…`), green =
  your repaired CLI built through `scripts/probe-build.py` at one stable path
  with one codegen unit (the campaign's `r2_build_adapter.py` `cli` mode is
  the reference). Re-verify every arm's sha256 before measuring.
- Programs: the exact benchmark bodies at
  `.temp/brief-717/continuation5/label-probes/association-part-key-read-width32000-{50,100}.rum`
  and `downvalue-guarded-decline-fallback-{50,100}.rum`. No added sink, no
  added assignment, nothing inside the measured body.
- Callgrind: `taskset -c 2 valgrind --tool=callgrind
  --callgrind-out-file=<file> <cli> --file <program>`, stdin closed,
  `VALGRIND_OPTS="--vgdb=no --enable-debuginfod=no"`, the microbench's 14
  `RUM_PROFILE_*` variables at 0 (`scripts/core-language-microbench.sh`,
  `disable_profile_env`). Parse with
  `.temp/brief-717/continuation4/inputs/callgrind_reader.py`; the seam's
  calls are the incoming edges to its symbol.
- Serial reproductions at `RUST_TEST_THREADS=1` for the red and the green,
  both labels, 50 and 100.
- The whole-workload ledger uses the campaign's workload
  (`.temp/brief-717/continuation5/ledger/workload.rum`) with
  `--dump-instr=yes --collect-jumps=yes`; the scan compares per-function
  self-Ir and incoming call counts across arms at identical call counts.

## Verification (mandatory, ordered)

1. Implement the repair; `cargo fmt --check`; `cargo check -p rum-runtime
   --all-targets --offline --locked`.
2. Replay the standing inventories covering the touched file before the
   first acceptance suite. Disclose every changed row; a stale derived-cell
   red is an ordering artifact, replayed, never reported as a regression.
3. The focused profiling witnesses at 8 and 1, and the full workspace at 8
   threads (counts above).
4. Build the green CLI and take the Callgrind table: red and green, four
   draws per cell, 50/100, both labels, with the helper's calls and the
   caller's per-call row; serial reproductions.
5. The whole-workload ledger and the per-call scan.
6. The four-build warning matrix.
7. `report.json` in the `rum-report/1` schema; `python3 scripts/report.py
   check 718` green; report and stop for the director's gate.

## Checkpointing

All scratch under `.temp/brief-718/`. After each milestone write
`.temp/brief-718/PROGRESS.md` with what is done and verified, the exact next
action, the tree's state, and anything the director needs to take over.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the change in
  the working tree.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo, no network. The only commands: file edits
  within Scope, `cargo`, the release `rum-cli`, the microbench/probe scripts,
  `valgrind`, `llvm-objdump`/`llvm-nm`, and `python3` for parsing.
- Testing Policy is a hard line; no third-party oracles.
- Hot-path discipline: the measurement is the evidence; a bar that is not
  reached is a report, never a reason to weaken the probe.
- Timeout discipline: `scripts/run-bounded.py` with a data-derived bound.
- Paste real verbatim output. Never fake a result.

## Report format

`.temp/brief-718/report.json` (a `rum-report/1` file), one entry per run with
`cmd`, `log`, `exit`, `attempt`, `tests`; every verdict line quoted; the
Callgrind table and the ledger scan as `sections`; `changes` lists every
touched file with line ranges and a one-line why. Final message is
`python3 scripts/report.py summary 718` and nothing else.

## Director gate — 2026-11-10: ACCEPTED

**The STOP is accepted as an environment boundary, not a product signal.**
The workspace at 8 threads read 4,203/9/61 over 63 summaries; all nine
failures are native ZeroMQ loopback binds denied with EPERM inside the
worker sandbox, reproduced in the same binary at 1 thread (143/9/0). That is
the standing sandbox class (brief 707, continuation B); the same nine tests
pass on the host. The worker stopped at its bar instead of weakening it,
which is the contract working.

**My host gate on the live tree.** `cargo fmt --check` clean;
`cargo check -p rum-runtime --all-targets` rc=0. Runtime lib, 3 x 8 threads
and 1 x 1 thread, each exactly
`1413 passed; 0 failed; 5 ignored; 0 measured; 2 filtered out`. The focused
profiling witnesses (the five named gates, `synchronous_families` and the
`publication::` tests): 31/0/0 at 8 and at 1 threads. The full workspace at
8 threads with the seven standing skips: **4,212 passed / 0 failed / 61
ignored over 63 binaries**, in a 16G scope (peak 15,403,331,584 B, no OOM).
The first attempt in a 14G scope was SIGKILLed in `rum_stdlib --lib` at the
scope cap (`oom_kill=1` after 913 tests, exit 101): the suite's measured
peak is above 14 GiB, so that is a scope-size environment boundary of mine,
not a test failure, and the 16G rerun is the gate. Four-build warning
matrix: 0 / 3 / 0 / 3 diagnostics, only the three standing `test_builtins`
names in the two lib-test configurations. The report's run counts
`runtime-8 1415/0/5` and `profiling-witnesses 33/0/0` are parse artifacts:
the worker's own logs read `1413 passed` and `running 31 tests` + `31
passed`, and my reruns reproduce both.

**Cost, measured.** `record_current_source_timing` reads **3 Ir per call**
on both campaign labels at 50 and 100 iterations, four draws per cell,
primary and serial (bar <= 4, target 3), against 24 on the red arm and 22
on base, at identical call counts; the cold helper
`record_current_source_timing_enabled` has **0 calls and 0 self-Ir** in
every disabled run; the caller `evaluate_definition_application_rhs` stays
at 91 (downvalue) and 125 (width32000) per call. Fifty-six rows in all
(two labels x three arms x two iteration counts x four draws, plus the
eight serial reproductions). Disassembly of the green CLI: the outer is 13
bytes, `cmpl $0x3b9aca00,%ecx; jne <helper>; retq` -- the `Option<Instant>`
niche check, so the disabled start-None path returns before any flag load;
the helper is 1,321 bytes and holds the old enabled body verbatim. The
test-cfg order (`take_symbol_timing_frame` -> `admit()` -> admitted flag
read -> start check) is preserved, and the enabled path records the same
counters and `EventKey::CacheSource` entries. The worker's source is
identical to my calibrated start-first tree except two blank lines.

**Whole-workload ledger and scan.** Four fresh captures (base and green,
two interleaved draws each). The six accepted seam rows are exact at
identical call counts on both arms (13->3 at 103,920; 23->18 at 393,040;
18->14 at 91,440; 20->18 at 49,008; 20->18 at 21,428; 18->14 at 7,003);
cold helpers 0 self-Ir / 0 calls; the repaired seam moves 22->3 at 109,077
calls (-2,072,463 Ir); the six-seam deterministic delta stays -3,539,044,
so the ledger's deterministic total is **-5,611,507**. Deterministic scan:
567 of 619 `rum_core`/`rum_runtime` rows are stable within each arm across
both draws; on the stable intersection exactly nine rows change per call --
the seven seam reductions, `Session::drop` 34->33 at one call, and the
single accepted once-called intercept `drive_input_request` 54->55 (the
708 fixed-intercept class). **Zero unaccepted growth rows.** Whole-workload
totals are recorded, not gated: base 16,460,946,396 / 16,458,927,112;
green 16,463,656,544 / 16,461,948,238. The process-level hash-seed spread
(52 unstable rows per arm, about 8 million instructions) dominates the
-5.6 million deterministic delta; this is the run-4/5 noise-limited row,
and Stage D's re-frozen range clause is unchanged.

**Scope.** Five files. `source.rs` is the repair: `#[inline(never)]` on the
outer, the test-cfg admission and admitted flag read kept before the start
check, the enabled body moved once into the `#[cold] #[inline(never)]`
helper, `access` renamed `_access`, `start` passed as `Some(start)`; no
duplicated body, no `unsafe`, no `#[allow]`, no test change. The four TSVs
are canonical replays: 0 removed rows; identities, statuses, dispositions
and exclusions unchanged; `unknown_attributes` +5 rows, exactly the five
new cfg attributes; `unknown_captures` 0 rows; `local-contracts` only the
68 fingerprint cells; `promoted` only the 2 certificate cells -- the
established replay pattern (brief 716's gate moved the same two cells).
**This acceptance sets the standing-inventory baseline for the 717
restart.**

**Verdict: brief 718 is ACCEPTED.** The disabled source-timing seam is
repaired to the `record_timing` class. Consequence: brief 717 restarts at
Stage A on this committed source, whole A-F, with no A/B/C receipt
carried; its Stage A inventory replay is against this commit's rows.
