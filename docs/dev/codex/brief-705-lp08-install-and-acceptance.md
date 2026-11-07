# Brief 705 — LP08: install the LP01 ledger, five-mode parent, acceptance

**Brief 704 stopped before installation on two bars I have since corrected
(gate note `.temp/director/brief-704-gate.md`). The five fresh observations,
the canonical candidate and all 14,964 individual now-SAME proofs are
accepted; the loss total in 704 was the lexical-only delta (the five-mode
total is 14,964), and the key whitelist was source-less only (the same three
phases also emit source-qualified spellings in cache/all). This brief
installs the accepted candidate under the corrected bars, proves the
five-mode parent at 8 and 1 threads, and runs the acceptance. No
observation, campaign or microbench rerun; the install and the parent are
the fresh end-to-end check.**

## Goal

1. Reproduce the 704 candidate byte-for-byte with the canonical refresh and
   install it through the canonical installer under the corrected bars.
2. Prove `logical_counter_port` green in all five modes at 8 and 1 threads,
   with 875 receipts per mode and lexical `counter-losses=15463`.
3. Replay the remaining standing inventories and run the acceptance:
   `cargo fmt --check`, the direct workspace suite with only the two
   standing skips, and the required-switch host suites.
4. Leave the tree uncommitted for my gate.

## Context pointers

- `.temp/director/brief-704-gate.md` — the accepted Stage B result and the
  corrected installer bars.
- `.temp/brief-704/lp01/` — the five fresh `actual-true-{mode}.json`,
  `executed-true-{mode}.json` and `logical-true-{mode}.log`;
  `.temp/brief-704/logical-profile-losses-candidate.json` — the accepted
  candidate; `.temp/brief-704/inputs/refresh_ledger.py` — the canonical
  generator copy; `.temp/brief-704/install_ledger.py` — the installer shape,
  whose assertions must be corrected, not loosened. Read/copy only.
- `.temp/brief-675/refresh_ledger.py` and `.temp/brief-675/lp01-observation/`
  — the canonical refresh; the test owns that scratch location
  (`fixtures/logical_profile/mod.rs:12-15`); copy its emitted files into
  this brief's root immediately.
- `.temp/brief-703/` — the accepted host campaign and builds; read only.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/mod.rs`
  — the parent's exact child environment; the parent aborts at the first
  failing child.
- `docs/dev/profiling.md`, `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`.

## Scope

The working tree starts with the 27 uncommitted install entries (16 sources
+ 11 replay inventories). Verify that at the start and record their hashes.

**Authorized live writes:**

1. the LP01 ledger
   `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`,
   written only by the canonical installer;
2. standing inventory files rewritten by their own generators during the
   remaining replays, with every changed row disclosed.

No other live file may change. The 16 reconciled sources stay byte-identical
and the LP01 test source and every assertion stay unchanged.

All scratch goes under `.temp/brief-705/`. No `unsafe`, public-API, serde,
feature, manifest, dependency or workload change. No version-control
command. Do not run `scripts/director-gate.py` (it invokes git and
`memory-scope.sh`; it is my instrument) — run the component commands
directly. Do not run the gate script's workspace step as a substitute for
the direct suite below.

## Stage A — reproduce the candidate and install

1. **Inputs.** Copy the five `actual-true-{mode}.json`,
   `executed-true-{mode}.json` and `logical-true-{mode}.log` from
   `.temp/brief-704/lp01/` into `.temp/brief-705/lp01/`; record their
   sha256 in the report. Do not edit them.

2. **Reproduce.** Run the canonical refresh over the copies (the
   `.temp/brief-675/refresh_ledger.py` logic: the five modes in order,
   interned strings and vectors, 875 receipts per mode, `build_hash=true`,
   header unchanged). The result must be byte-identical to
   `.temp/brief-704/logical-profile-losses-candidate.json`; any difference
   is a STOP.

3. **Installer bars (binding, corrected).** Before writing, the installer
   must decode the frozen ledger and the candidate and assert:
   - receipts: exactly the same 4,375 decoded records;
   - key rows: 7,960, unchanged in identity and count; every changed row is
     on the `resumed` delivery; every change is additive; every added key's
     phase is one of `runtime|prepare_arguments.append_arg`,
     `runtime|evaluate.fixed_point_equal`, `runtime|evaluate.trace_update`,
     with any source segment (the source-less `|-` form and the
     source-qualified `|f`, `|held`, `|target`, `|c621owner` forms are the
     observed spellings); zero keys removed;
   - loss rows: per-mode totals 0 / 3,270 / 3,270 / 4,212 / 4,212 and the
     91,424 -> 76,460 total; zero added loss rows; every removed cell has
     its individual proof — the same `(mode, program, key)` absent from the
     fresh loss rows and present in the fresh queued and resumed key
     records — and every removed row is LP03-owned;
   - the disabled mode's decoded records identical; every one of the 76,460
     surviving loss cells decoded-identical; the header unchanged.

   A removed row without its proof, an added loss row, a removed key, a
   `queued`-row change, a key outside the three phases, a receipt change, a
   disabled-mode change or a header change is a STOP; never weaken an
   assertion or hand-edit a cell.

4. **Install and prove.** Write the ledger only through the canonical
   installer (candidate bytes after the assertions). Record the installed
   sha256. Then run the full `logical_counter_port` (the parent, all five
   modes) at 8 threads and at 1 thread:
   `cargo test -p rum-stdlib --offline --locked --test suspension_oracle
   --no-fail-fast -- logical_counter_port --nocapture` with
   `RUST_TEST_THREADS=8` and `RUST_TEST_THREADS=1`. All five children green
   in both runs, 875 receipts per mode, lexical `counter-losses=15463`.
   Copy the parent's fresh emitted files out of
   `.temp/brief-675/lp01-observation/` into `.temp/brief-705/lp01-parent/`
   immediately after each run. The canonical refresh of the parent's fresh
   emissions must decode equal to the installed ledger; report byte identity
   or its exact difference. Report every mode's key, loss and receipt counts
   after the install.

## Stage B — Remaining inventories and acceptance

- Replay the typed-slot receipts, the frame-root closed lists and the
  rejection census (not run in 701 or 704). Generator output only; no hand
  edit; every changed row disclosed with its generator and verdict line.
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

All scratch under `.temp/brief-705/`; never `/tmp`. The dispatcher already
runs you in the 16G memory scope; do not call `systemd-run` or
`memory-scope.sh` yourself. One cargo command at a time; builds
`CARGO_BUILD_JOBS=8`; tests `RUST_TEST_THREADS=8 --no-fail-fast`; a red is
reproduced at `--test-threads=1`. Timing and heap captures run serially and
alone. Do not run a version-control command.

## Deliverable

- The input hashes, the reproduced candidate identity, the installer
  verification panel (receipts, key rows, per-mode loss totals, every
  removed cell's proof), the installed sha256.
- The five-mode parent runs at 8 and 1 threads with every child's verdict
  line and `counter-losses`, and the post-install per-mode counts.
- The typed-slot, frame-root and rejection-census replay records, the
  workspace and required-switch suite verdicts, the focused-suite runs and
  `cargo fmt --check`.
- The `rum-report/1` report with every run, failed attempt, verdict line and
  excluded row; `python3 -I scripts/report.py check 705`.
- The working tree uncommitted for my gate: the 16 sources, the 11 replay
  inventories, the re-derived LP01 ledger, and any generator-disclosed
  inventory cells from the replays.
