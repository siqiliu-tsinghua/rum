# Brief 703 — LP08 final: campaign completion, LP01 ledger replay, acceptance

**Final implementation brief. The reconciled 16-file install and the 11
replay-generated inventory files are in the working tree, uncommitted. Brief
702 proved the form on fresh binaries: every accepted bar reproduced. It
stopped only on a collateral clause I have since ruled mis-scoped (gate note
`.temp/director/brief-702-gate.md`): the 18 base→reconciled nonterminal rows
are the accepted 698 full-install form's queued-parity env-gate checks, not
collateral. This brief finishes the line: the campaign items not run after
the STOP, the LP01 ledger replay I authorized in the 701 gate, the remaining
standing inventories, and the acceptance suite. The only live write is the
LP01 ledger itself, through the canonical replay.**

## Goal

Leave the LP08 install ready for my gate and commit:

1. Complete the campaign: the host contract campaign and the rum-vs-rum
   core-language microbench.
2. Re-derive the LP01 ledger across all five modes under the exact bars
   below, install it through the canonical writer, and prove
   `logical_counter_port` green in all five modes at 8 and 1 threads.
3. Replay the remaining standing inventories (typed-slot receipts,
   frame-root closed lists, rejection census) and run the acceptance:
   `cargo fmt --check`, the direct workspace suite with only the two
   standing skips, and the required-switch host suites.
4. Leave the tree uncommitted for my gate.

## Context pointers

- `.temp/director/brief-702-gate.md` — the campaign result and the
  corrected collateral bar.
- `.temp/director/brief-701-gate.md` — the LP01 ledger replay authorization
  and its bars.
- `.temp/brief-702/report.json`, `measurement-stop-panel.json`,
  `reconciled-address-panel.json`, `runs/` — the fresh campaign evidence;
  read only.
- `.temp/brief-699/` and `.temp/brief-698/` — the identity harness, the
  host probe (`brief679_reset.rs`), the campaign scripts and the raw
  evidence; read/copy only.
- `.temp/brief-675/refresh_ledger.py`, `install_ledger.py`, `lp01-observation/`
  — the canonical LP01 replay mechanism; read/copy only, never edit. The
  test owns that scratch location; copy its emitted files into this brief's
  root immediately.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/mod.rs`
  — the parent's exact child environment (the five modes); mirror it, do not
  hand-pick variables.
- `scripts/core-language-microbench.sh`, `docs/dev/profiling.md`,
  `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`.

## Scope

**One live write is authorized:** the LP01 ledger
`crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`,
written only by the canonical replay. No other live file may change: the 16
reconciled sources and the 11 replay inventories are inputs and stay
byte-identical, and the LP01 test source and every assertion stay unchanged.

All scratch goes under `.temp/brief-703/`. Retained roots are read/copy
only. No `unsafe`, public-API, serde, feature, manifest, dependency or
workload change. No version-control command. Do not run
`scripts/director-gate.py` (it invokes git and `memory-scope.sh`; it is my
instrument) — run the component commands directly. Do not run the gate
script's workspace step as a substitute for the direct suite below.

## Stage A — Campaign completion

- **Host contract campaign.** On the reconciled arm, rebuild the host probe
  from `.temp/brief-698/` by function (the formatted source has new line
  numbers) and run the obligations: exactly-once retirement of open
  records, no suffix output, no old success; old-token/queue isolation,
  handoff and unrelated-owner isolation; `HostSession::reset` retains its
  documented guest discard — the withdrawn no-discard clause stays
  withdrawn. Quote the verdict lines and the exact assertions.
- **Microbench.** Build base and reconciled release `rum-cli` binaries at
  one path with one codegen unit, then run the established rum-vs-rum
  core-language protocol with `scripts/core-language-microbench.sh`:
  profiling off, in-kernel `AbsoluteTiming`, no competing CPU-heavy
  workload, four passes per arm and the layout control, tier-median
  quotients over every `timings.tsv` label, the hot labels and each
  same-tier neighbor. One pair cannot gate a label; a moved label needs its
  distribution and the control's movement. Timing is secondary — the
  instruction ledger already decided — but an unattributed panel movement
  is a STOP.

## Stage B — LP01 ledger replay

The authorization and bars are from the 701 gate. The replay fires now
because brief 702 proved the form on fresh binaries.

1. **Fresh observation.** Run the five mode children with the parent's
   exact environment, mirroring `logical_counter_port`'s spawn logic
   (mode order `disabled`, `lexical`, `timing`, `cache`, `all`; all
   `RUM_PROFILE_*` cleared; `RUM_LP01_CHILD`, the incremental-cache
   override, the lexical switch for the enabled modes, the timing flags for
   `timing`/`all`, the cache flag for `cache`/`all`, and the six detail
   flags for `all`). Each child's semantic, fixture and executed-receipt
   assertions must be green; only the final ledger comparison may fail
   before the replay. Copy `actual-true-{mode}.json`,
   `executed-true-{mode}.json` and `logical-true-{mode}.log` into
   `.temp/brief-703/lp01/` immediately, before any install.
2. **Candidate ledger.** Build it only with the canonical refresh (the
   brief-675 logic: the five modes in order, interned strings and vectors,
   875 receipts per mode, `build_hash=true`). The header (`format`,
   `stage`, `record_tags`, `comparison_builds`) stays unchanged, and the
   stage stays `unpromoted known losses`.
3. **Exact delta review (decoded records, both builds if the test runs
   both).** Report and verify:
   - receipts: exactly the same 4,375 decoded records;
   - key rows: 7,960, unchanged in identity and count; the changed rows are
     exactly the `resumed` rows the entry split adds, all additive — every
     added key is one of `runtime|prepare_arguments.append_arg|-`,
     `runtime|evaluate.fixed_point_equal|-`,
     `runtime|evaluate.trace_update|-`, and no key is removed;
   - loss rows: 91,424 before; removed exactly the now-SAME cells, each
     proven by the same `(mode, program, key)` being absent from the fresh
     loss rows and present in the fresh key records for both deliveries;
     the measured lexical removals are `fixed_point_equal` 959,
     `stability_check` 959, `trace_update` 959, `append_arg` 393 (the
     other modes are data, and every removed row needs its proof); zero
     added loss rows;
   - the disabled mode's decoded records are identical; every standing
     loss cell not in the removal set is byte-for-byte unchanged; the
     header is unchanged.
   A removed row without its proof, an added loss row, a non-additive key
   change, a receipt change, a disabled-mode change or a header change is a
   STOP; do not weaken an assertion or hand-edit a cell.
4. **Install and prove.** Write the ledger only through the canonical
   writer, then run the full `logical_counter_port` (the parent, all five
   modes) at 8 and 1 threads: all children green, 875 receipts per mode,
   the lexical total `counter-losses=15463`. Report every mode's
   key/loss/receipt counts before and after.

## Stage C — Remaining inventories and acceptance

- Replay the typed-slot receipts, the frame-root closed lists and the
  rejection census (not run in 701 because of the LP01 STOP). Generator
  output only; no hand edit; every changed row disclosed.
- `cargo fmt --check` clean.
- The direct workspace suite:
  `cargo test --workspace --offline --no-fail-fast -- --skip
  lp655_root_context_falsifier --skip lp656_public_root_context_falsifier`.
  The two standing reds stay skipped and disclosed; every other test
  green, including `logical_counter_port`.
- The required-switch host suites with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`
  for the lp03/04/05 adopted-phase and lp658 reset cases.
- The focused suite
  (`cargo test -p rum-runtime --lib -- runtime::profiling::
  runtime::suspension::`): 192 passed / the two standing reds / 2 ignored
  at 8 and 1 threads.
- A red whose only cause is a stale derived cell is an ordering artifact:
  replay and rerun; never weaken a test or hand-edit a cell.

## Bars and exclusions

Standing exclusions, restated:

- the two documented root-context reds `lp655_root_context_falsifier` and
  `lp656_public_root_context_falsifier`, in both arms and in the workspace
  run (skipped, not fixed);
- the required-switch suites run separately with the switch;
- the six source-impossible routes and the legacy free-index mutants;
- the `default_builtin_dispatch_spec_for_symbol` process-hash variation,
  never a cost offset;
- the withdrawn host guest-frame preservation clause.

A new warning, a new `UnknownHostPayload`, a changed row identity or
disposition, a disappeared row, a weakened assertion, an unattributed
measured movement, a live edit outside the LP01 ledger, or a header change
is a STOP with the exact panel.

## Host discipline

All scratch under `.temp/brief-703/`; never `/tmp`. The dispatcher already
runs you in the 16G memory scope; do not call `systemd-run` or
`memory-scope.sh` yourself. One cargo command at a time; builds
`CARGO_BUILD_JOBS=8`; tests `RUST_TEST_THREADS=8 --no-fail-fast`; a red is
reproduced at `--test-threads=1`. Timing and heap captures run serially and
alone. Probe builds through `scripts/probe-build.py` where a probe build is
needed; Callgrind relative offsets are decimal bytes. Do not run a
version-control command.

## Deliverable

- The host-campaign and microbench raw runs and verdict lines, with the
  base/candidate `timings.tsv` comparison.
- The five fresh mode observations, the candidate ledger, the decoded
  before/after delta panel with every changed and removed row and its
  proof, and the post-install five-mode test runs at 8 and 1 threads.
- The typed-slot, frame-root and rejection-census replay records, the
  workspace and required-switch suite verdicts, the focused-suite runs and
  `cargo fmt --check`.
- The `rum-report/1` report with every run, failed attempt, verdict line
  and excluded row; `python3 -I scripts/report.py check 703`.
- The working tree uncommitted for my gate: the 16 sources, the 11 replay
  inventories and the re-derived LP01 ledger only.
