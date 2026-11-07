# Brief 704 — LP08: LP01 ledger replay and acceptance

**Brief 703 stopped at Stage A on the microbench timing bar. I gated it
(`.temp/director/brief-703-gate.md`): the host campaign and the builds are
green, the two timing residuals are closed by my instruction-count probe
(per-iteration instructions equal across base, reconciled and layout arms;
per-function self-instruction counts identical on both labels), and the
timing lane's per-label criterion is corrected. Nothing else from 703 is
open except the stages it never reached. This brief runs exactly those: the
LP01 ledger replay I authorized in the 701 gate, the remaining standing
inventory replays, and the acceptance suite. The only authorized live writes
are the LP01 ledger through the canonical writer and the standing inventory
generators during their replays. No campaign and no microbench rerun.**

## Goal

1. Re-derive the LP01 ledger across all five modes under the exact bars
   below, install it through the canonical writer, and prove
   `logical_counter_port` green in all five modes at 8 and 1 threads.
2. Replay the remaining standing inventories (typed-slot receipts,
   frame-root closed lists, rejection census) and run the acceptance:
   `cargo fmt --check`, the direct workspace suite with only the two
   standing skips, and the required-switch host suites.
3. Leave the tree uncommitted for my gate.

## Context pointers

- `.temp/director/brief-703-gate.md` — the Stage-A acceptance and the
  corrected timing criterion; `.temp/director/probe-703/` — my instruction
  probe (provenance only; do not rerun it).
- `.temp/director/brief-701-gate.md` — the LP01 ledger replay authorization
  and its bars.
- `.temp/brief-703/` — the accepted host campaign, builds and timing
  evidence; read only. Do not rerun the microbench or the campaign.
- `.temp/brief-701/lp01_panel.py`, `lp01-stop-panel.json`,
  `loss-position-replay.json`, `lp01-producer-replay.json` — the 701 LP01
  decoder and producer replay; read/copy only.
- `.temp/brief-675/refresh_ledger.py` and `.temp/brief-675/lp01-observation/`
  — the canonical candidate generator; read/copy only, never edit. The test
  owns that scratch location (`fixtures/logical_profile/mod.rs:12-15`); copy
  its emitted files into this brief's root immediately.
- `.temp/brief-675/install_ledger.py` — the LP07 installer shape; the LP08
  installer is its analogue with the LP08 proofs and the key-row rule below.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/mod.rs`
  — the parent's exact child environment (the five modes); mirror it, do not
  hand-pick variables. The parent aborts at the first failing child, so the
  five fresh observations are run as five direct children.
- `scripts/core-language-microbench.sh`, `docs/dev/profiling.md`,
  `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`.

## Scope

The working tree starts with the 27 uncommitted install entries (16 sources
+ 11 replay inventories; 14 modified + 2 untracked runtime files + 11
modified inventories). Verify that at the start and record their hashes.

**Authorized live writes:**

1. the LP01 ledger
   `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`,
   written only by the canonical replay and its installer;
2. standing inventory files rewritten by their own generators during the
   Stage B replays, with every changed row disclosed.

No other live file may change. The 16 reconciled sources stay byte-identical,
and the LP01 test source and every assertion stay unchanged.

All scratch goes under `.temp/brief-704/`. Retained roots are read/copy only.
No `unsafe`, public-API, serde, feature, manifest, dependency or workload
change. No version-control command. Do not run `scripts/director-gate.py`
(it invokes git and `memory-scope.sh`; it is my instrument) — run the
component commands directly. Do not run the gate script's workspace step as
a substitute for the direct suite below.

## Stage A — LP01 fresh observation

Run the five mode children directly with the parent's exact environment,
mirroring `logical_counter_port`'s spawn logic
(`fixtures/logical_profile/mod.rs:92-145`): mode order `disabled`, `lexical`,
`timing`, `cache`, `all`; all `RUM_PROFILE_*` cleared from the child;
`RUM_LP01_CHILD=<mode>`; `RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=0`; the
lexical switch for the enabled modes; the three timing flags for
`timing`/`all`; the cache flag for `cache`/`all`; the six detail flags for
`all`. Test invocation:
`cargo test -p rum-stdlib --offline --locked --test suspension_oracle
--no-fail-fast -- logical_counter_port --nocapture --test-threads=1`.

Each child's semantic, fixture and executed-receipt assertions must be
green; only the final frozen-ledger comparison may fail before the replay.
Copy `actual-true-{mode}.json`, `executed-true-{mode}.json` and
`logical-true-{mode}.log` into `.temp/brief-704/lp01/` immediately after
each child, before any install.

## Stage B — LP01 candidate, exact delta review, install

1. **Candidate ledger.** Build it only with the canonical refresh
   (`.temp/brief-675/refresh_ledger.py`: the five modes in order, interned
   strings and vectors, 875 receipts per mode, `build_hash=true`). Its
   header (`format`, `stage`, `record_tags`, `comparison_builds`) stays
   unchanged and the stage stays `unpromoted known losses`.

2. **Exact delta review of decoded records.** Report and verify:
   - receipts: exactly the same 4,375 decoded records;
   - key rows: 7,960, unchanged in identity and count; the changed rows are
     exactly the `resumed` rows the entry split adds, all additive — every
     added key is one of `runtime|prepare_arguments.append_arg|-`,
     `runtime|evaluate.fixed_point_equal|-`,
     `runtime|evaluate.trace_update|-`, and no key is removed;
   - loss rows: 91,424 before; removed exactly the now-SAME cells, each
     proven by the same `(mode, program, key)` being absent from the fresh
     loss rows and present in the fresh key records for both deliveries; the
     measured lexical removals are `fixed_point_equal` 959,
     `stability_check` 959, `trace_update` 959, `append_arg` 393 (the other
     modes are data, and every removed row needs its proof); zero added loss
     rows;
   - the disabled mode's decoded records are identical; every standing loss
     cell not in the removal set is byte-for-byte unchanged; the header is
     unchanged.

   A removed row without its proof, an added loss row, a non-additive key
   change, a receipt change, a disabled-mode change or a header change is a
   STOP; do not weaken an assertion or hand-edit a cell.

3. **Install and prove.** Install only through the canonical installer,
   modelled on `.temp/brief-675/install_ledger.py`: it decodes the before and
   candidate ledgers, asserts the receipt, key-row, loss-row, disabled-mode
   and header rules above, and asserts every removed cell has its individual
   proof line. Then run the full `logical_counter_port` (the parent, all five
   modes) at 8 and 1 threads: all children green, 875 receipts per mode, the
   lexical total `counter-losses=15463`. Report every mode's key, loss and
   receipt counts before and after, and the exact 91,424 -> 88,154 delta.

## Stage C — Remaining inventories and acceptance

- Replay the typed-slot receipts, the frame-root closed lists and the
  rejection census (not run in 701 because of the LP01 STOP). Generator
  output only; no hand edit; every changed row disclosed with its generator
  and its verdict line.
- `cargo fmt --check` clean.
- The direct workspace suite:
  `cargo test --workspace --offline --no-fail-fast -- --skip
  lp655_root_context_falsifier --skip lp656_public_root_context_falsifier`.
  The two standing reds stay skipped and disclosed; every other test green,
  including `logical_counter_port`.
- The required-switch host suites with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`
  for the lp03/04/05 adopted-phase and lp658 reset cases.
- The focused suite
  (`cargo test -p rum-runtime --lib -- runtime::profiling::
  runtime::suspension::`): 192 passed / the two standing reds / 2 ignored at
  8 and 1 threads.
- A red whose only cause is a stale derived cell is an ordering artifact:
  replay and rerun; never weaken a test or hand-edit a cell.

## Bars and exclusions

Standing exclusions, restated:

- the two documented root-context reds `lp655_root_context_falsifier` and
  `lp656_public_root_context_falsifier`, in both arms and in the workspace
  run (skipped, not fixed);
- the required-switch suites run separately with the switch;
- the six source-impossible routes (`eval_core/mod.rs:159`, `:174`) and the
  legacy free-index mutants;
- the `default_builtin_dispatch_spec_for_symbol` process-hash variation,
  never a cost offset;
- the withdrawn host guest-frame preservation clause.

A new warning, a new `UnknownHostPayload`, a changed row identity or
disposition, a disappeared row, a weakened assertion, a live edit outside
the authorized writes, or a header change is a STOP with the exact panel.

## Host discipline

All scratch under `.temp/brief-704/`; never `/tmp`. The dispatcher already
runs you in the 16G memory scope; do not call `systemd-run` or
`memory-scope.sh` yourself. One cargo command at a time; builds
`CARGO_BUILD_JOBS=8`; tests `RUST_TEST_THREADS=8 --no-fail-fast`; a red is
reproduced at `--test-threads=1`. Timing and heap captures run serially and
alone. Probe builds through `scripts/probe-build.py` where a probe build is
needed; Callgrind relative offsets are decimal bytes. Do not run a
version-control command.

## Deliverable

- The five fresh mode observations, the candidate ledger, the decoded
  before/after delta panel with every changed key row and every removed loss
  row with its proof, the installer verification panel, and the post-install
  five-mode parent runs at 8 and 1 threads.
- The typed-slot, frame-root and rejection-census replay records, the
  workspace and required-switch suite verdicts, the focused-suite runs and
  `cargo fmt --check`.
- The `rum-report/1` report with every run, failed attempt, verdict line and
  excluded row; `python3 -I scripts/report.py check 704`.
- The working tree uncommitted for my gate: the 16 sources, the 11 replay
  inventories, the re-derived LP01 ledger, and any generator-disclosed
  inventory cells from the Stage C replays.
