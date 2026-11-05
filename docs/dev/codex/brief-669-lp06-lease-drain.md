# Brief 669 — LP06 source stage amendment: the lease acquisition drain

**Shape: implementation (amendment to brief 668).** The 668 source stage is
accepted except for one bar: the two workspace configurations are not green
at 8 test threads because the lease test loses its process-budget baseline to
a prior epoch's deferred release. The 668 tree is in the working tree and
stays frozen; this brief lands the lease fix in test-gated code, proves it
with a deterministic falsifier, and re-runs only the affected acceptance.
The tree is left for the director's gate; no commit.

**Read first.** `.temp/director/brief-668-gate.md` — the director's rulings:
the panel `matcher-single-tiny` row is excluded as a calibrated callgrind
layout artifact (do not re-run the D campaign or re-argue that row); the 19
surface files, the census constants and the generated inventories are frozen
and accepted; the remaining bar is the 8-thread lease interference. The
director's scratch evidence is `.temp/director/gate-668/lease/` (`unfixed`,
`fix`, and the witness/trace logs). Reproduce the mechanism yourself; do not
copy a claim.

## Goal

When this brief is done, on the exact 668 source plus this amendment:

- the interference is **named by an instrument that survives TLS
  destruction**: the failing window's baseline is a prior epoch's
  `Budget`/`Storage` charge still held by another thread, which is then
  released inside the leased measurement window, and the writer's identity
  (kind and bytes) is recorded from a real failing run;
- the acquisition of the test lease drains the process diagnostic
  accounting before any test takes its baseline, so a prior release can no
  longer land inside a leased window; the fix is **test-gated code only**
  (`#[cfg(test)]` / test modules); production builds are unchanged;
- a deterministic falsifier fails without the drain and passes with it:
  a thread holds an `Epoch::for_test` charge of at least 32 KiB past its
  lease, releases it after the next test's baseline, and the unchanged
  `budget::process_bytes() > baseline` assertion must still hold;
- the lease test runs (never skipped) and both workspace configurations are
  green at 8 test threads, with only the standing exclusions below;
- the lib suite repeats at 8 threads enough times to cover the pre-fix rate
  (the director measured 4 failures in 60 unfixed runs) with zero lease
  failures, and the compile-control protocol tests (which build a fresh
  `rum-runtime` library without `cfg(test)`) stay green;
- `cargo fmt --check` is green and nothing outside the allowed files moves.

The assertion text and the default 8-thread concurrency stay unchanged. A
fix that needs production behaviour, a retry, a longer lease test, or a
skip is not this brief's fix: stop and name the required production change.

## Scope

Files you MAY modify:

1. `crates/rum-runtime/src/runtime/profiling/logical/epoch.rs` — only inside
   the existing test/feature-gated region, and only new `#[cfg(test)]` code
   (or a test-gated helper) plus the necessary `use`; the production cfg path
   must not gain or lose a line.
2. `crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs` — the
   `enabled()` acquisition path.
3. `crates/rum-runtime/src/runtime/profiling/tests/lease_release.rs` — the
   falsifier and any test harness the fix needs.
4. `crates/rum-runtime/src/runtime/profiling/tests/epoch_contract.rs` — the
   existing joins may stay, be simplified or be removed only with a
   disclosure that the drain now covers the class.

MUST NOT touch: the 19 brief-668 surface files' non-test code (they must stay
byte-identical to `.temp/brief-667/probe/inline-enter`), the census
constants, the generated inventories, the public API, features, serde,
manifests, workloads, the suspension driver, the typed-resume macro, the
checked adapter, `session_requests.rs`, `builtins_input.rs`, and R7. Do not
re-run the D campaign, the semantics matrix, the census/inventory replays, or
the fixed-root measurements; the source is frozen and the gate-667/668 rows
stand by identity.

## Steps

1. **Witness.** On the frozen 668 tree, instrument the drop path in a
   scratch build and capture a real 8-thread failure: the baseline, the
   own addition, and the foreign release (kind and bytes) that lands inside
   the window. The instrument must be safe at TLS destruction (`try_with`;
   an inaccessible TLS slot counts as unregistered).
2. **Fix.** Implement the acquisition drain in test-gated code; keep the
   production build clean (the compile controls prove it).
3. **Falsifier.** Add the deterministic red/green test described in Goal;
   show it red against a build without the drain (a scratch copy is fine)
   and green with it, with the verdict lines quoted.
4. **Repeat.** Run the full lib suite at 8 threads at least 20 times with
   the fix (and record the pre-fix rate you reproduce); run `cargo test
   --workspace` default and `--no-default-features` at 8 threads with the
   standing exclusions, the lease test running; run the required-switch host
   suites; run `cargo fmt --check`.
5. **Report.** `.temp/brief-669/report.json` in `rum-report/1`, every run
   and verdict line quoted; `python3 -I scripts/report.py check 669`.

## Standing exclusions (unchanged)

The two carried root-context falsifiers `lp655_root_context_falsifier` and
`lp656_public_root_context_falsifier` (documented reds in both arms), the
lp03/04/05 adopted-phase host tests and the lp658 host reset cases (run in
the required-switch host suites), and R7. The lease test is **not** excluded.
Disclose the exact skip list and confirm it matches the 668 list.

## Hard bars

- The lease test passes in both workspace configurations at 8 threads; its
  assertion text and the 8-thread default are unchanged.
- The deterministic falsifier is red without the drain and green with it;
  the witness names the writer and its bytes from a real failure.
- The fix compiles out of production: a fresh `cargo build` of the protocol
  library (the compile-control tests) succeeds, and the 19 surface files are
  byte-identical to the frozen form.
- The lib suite at 8 threads: at least 20 runs, zero lease failures; only
  the standing reds remain.
- `cargo fmt --check`; no new warnings; no scope creep.

## Verification

One cargo command at a time inside the dispatcher's scope; jobs 8, test
threads 8, offline/locked; bound anything that may spawn children with
`scripts/run-bounded.py`. Reproduce any unexpected red at
`--test-threads=1` before calling it a defect. No timing or perf run.

## Checkpointing

All scratch under `.temp/brief-669/`; keep `PROGRESS.md` with the exact next
action and receipts. Leave the tree compiling at each checkpoint. No
version-control command, no network, no new `unsafe`, no public API/feature/
serde change, no workload or inventory edit.

## Director gate — 2026-11-05: GREEN

| Step | Verdict | Detail |
|---|---|---|
| STOP ruled | PASS | the brief's inventory freeze conflicts with `TEMPLATE.md`'s standing maintenance duty ("always in scope, whatever the fence says"); the whole-source fingerprints include `cfg(test)` files, so the two generated refreshes are required and were applied |
| generated | PASS | `local-contracts.tsv` 68 rows, column 18 only; `promoted.tsv` C01/C02, columns 4/5 only; every other field unchanged; the green acceptance tests prove each cell equals the generator output |
| scope | PASS | exactly the three permitted files changed; the original lease test byte-identical; `epoch_contract.rs` joins kept unchanged; the two refreshed inventories are the standing duty |
| falsifier | PASS | my own builds from the final source: red 3/3 without the acquisition call (`.temp/director/gate-669/no-drain`), green 3/3 with it |
| repeats | PASS | 24/24 full 8-thread lib runs green, the lease test and the falsifier running in every run; with the worker's 60/60, 84 fixed runs and zero lease failures |
| witness | PASS | reproduced on the unfixed tree: 2/60 failures, window baseline 19,776 B, own addition 6,136 B, foreign release 19,776 B (1 Budget 32 B + 32 Storage 19,744 B) |
| workspace | PASS | default and `--no-default-features` at 8 threads: 4,145 passed / 0 failed / 61 ignored each; the seven standing skips equal the 668 list; the lease test is not skipped |
| host switch | PASS | lp03/lp04/lp05 with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`: 3 + 1 + 1 passed in both configurations |
| fmt | PASS | `cargo fmt --check` |
| production | PASS | `#[cfg(test)]` only; production cfg path untouched; compile controls green; no new warnings |
