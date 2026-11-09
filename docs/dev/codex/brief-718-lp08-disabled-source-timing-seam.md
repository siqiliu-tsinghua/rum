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
