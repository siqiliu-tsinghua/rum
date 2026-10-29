# brief-631 — the director's gate as one script (director tooling)

Written and gated by the director, with no worker (user ruling 2026-10-29:
"你自己写吧"). It is a brief only so the change has a record and a gate.

## Goal

The gate I run before crediting a brief was a sequence of hand-typed shell
lines, rebuilt at every gate from the last one's scratch scripts
(`.temp/director/gate-630b2/`: `census-pair.py`, `own-replay.py`,
`merge-promoted.py`, a census dump patch). Each gate cost context in
commands, output and re-derivation (user ruling 2026-10-29: procedures
become scripts, inputs in, one table out).

`scripts/director-gate.py run NNN [--stage S]` now runs it:

| Step | What it runs | What it decides |
| --- | --- | --- |
| `scope` | `report.py check --against-git` | the report's changes equal the tree's |
| `fmt` | `cargo fmt --check` | formatted |
| `check` | `cargo check --workspace --all-targets` | no warning new against HEAD (message and file; lines dropped) |
| `ownership` | `production_inventory_is_exact`, replayed for up to three rounds | moves pass; a removed row, a cross-file move, a new host payload, or a new row on an unchanged line is FLAGGED |
| `census` | the rejection census with `RUM_CENSUS_DUMP` | its frontier paired with the base's through `git diff -U0 HEAD`; a base-only or candidate-only row, or a moved count, is FLAGGED |
| `oracle` | `exact_types_and_constructor_claims_match_rustc` | passes |
| `standing` | LP01's `logical_counter_port`, `typed_slot_census` | pass |
| `fingerprints` | `regenerate_local_contract_fingerprints` | only the fingerprint column moved; copied in |
| `certificates` | `local_production_acceptance`, both feature builds | merged, then both pass |
| `inventories` | the five inventory filters in one run | all pass on the final tree |
| `workspace` | `cargo test --workspace --no-fail-fast` | no failure, no OOM kill |

The steps up to `standing` can need a ruling, so a FLAGGED or FAIL there
stops the run before the replays that hash the tree. A filtered run that runs
no test is a FAIL. A green run stores its final frontier and its warnings
under `.temp/director/gate-state/<crates tree>/`, the base of the next gate;
when that key is missing, `census` dumps HEAD's frontier in a worktree.

## Scope

- `scripts/director-gate.py` (new), `scripts/README.md`.
- `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`: with
  `RUM_CENSUS_DUMP` set, the census writes its frontier before any
  assertion. This replaces the dump patch, which had to be applied and
  reverted around every gate. It is test code; no production path changes.
- `TODO.md` (aligned, including brief-630 B2 and B3, which it had not
  recorded), `CLAUDE.md` (one line, still 199 lines).
- Not in this brief: the perf-panel layer (two arms built at one path, path
  calibration, `fndiff3`, the interleaved panels). Its instruments live in
  `.temp/` (the resume harness, brief-626's label set) and move into the repo
  first; `TODO.md` records it as owed.

## Verification

- `selftest` covers the pure parts: the hunk arithmetic, the census pairing
  as a multiset, the test totals, the scope line, the warnings, the
  fingerprint and certificate merges, and the ownership replay and its
  flags.
- Nine mutants of the script each turn a named selftest case red
  (`.temp/director/gate-mutants.py`). One was missed at first, a shift that
  ignores the removed line count, until a deletion hunk was added.
- Calibration against brief-630 B3's known answers, from its gate logs:
  - `pair` of the B2 frontier with B3's, through `git diff 1c115117`, gives
    996 paired, 2 retired, 6 new, 0 base-only and 38 candidate-only, as the
    hand pairing did through a copied source tree;
  - the workspace log sums to 59 binaries, 4013 passed, 0 failed and 61
    ignored; the first run's log gives 4012/1/61 and names the oracle test;
  - `check.log` gives the three known dead-code warnings, and the scope line
    a 10.5 GB peak;
  - the ownership counts read from the test's own lines are 61, 79 and 7942.
    A first count from the files gave 8082, because it added the
    self-reaching and host-payload rows; the script now reads the test's
    lines.
- HEAD's base was seeded from B3's gate (`frontier-b3.tsv`, `check.log`):
  no file under `crates/` changed after `ba1e1e52`.
- The first full run (`gate-dg1`) stopped at `census` as FLAGGED with every
  row paired. The census test certifies with the contract fingerprints
  (`promotion::earn`) after it dumps and counts, and any edit under `crates/`
  stales them, so before the fingerprint replay that test cannot pass. The
  step now decides by the pairing and defers the test's own verdict to
  `inventories`.
- The rerun from `census` (`run dg1 --from census`, the steps before it
  kept from the first run) was green: the table below.

## Director gate

`python3 -I scripts/director-gate.py run dg1` (logs
`.temp/director/gate-dg1/logs/`), the gate's first run on its own change:

| Step | Verdict | Secs | Detail |
| --- | --- | --- | --- |
| scope | SKIP | 0 | no worker report (a director change) |
| fmt | PASS | 6 | formatted |
| check | PASS | 0 | 3 warnings, 0 new against HEAD, 0 gone |
| ownership | PASS | 37 | self-reaching types 61, UnknownHostPayload 79, UnknownConstruct 7942; replayed 0 rounds |
| census | PASS | 125 | 1040 {RET001: 10, RET002: 997, RET003: 33}; paired 1040, retired 0, new 0, base-only 0, cand-only 0 |
| oracle | PASS | 298 | 1 passed |
| standing | PASS | 406 | lp01 1/0; typed-slot 8/0 |
| fingerprints | PASS | 0 | 68 contracts, 68 fingerprints changed, copied in |
| certificates | PASS | 153 | both builds pass; 2 promoted rows merged; 2 rounds |
| inventories | PASS | 338 | 19 passed, 0 failed; frontier 1040 |
| workspace | PASS | 2236 | 59 binaries, 4013 passed, 0 failed, 61 ignored; peak 7.3 GB of 14, oom_kill 0, kernel OOM lines 0 |

VERDICT: GREEN. The base for the next gate is stored under crates tree
`5bffd8d2a0b2`. The 68 fingerprints and 2 promoted rows that the replay
rewrote are in this commit; the edit to `census/tests.rs` is what staled them.
