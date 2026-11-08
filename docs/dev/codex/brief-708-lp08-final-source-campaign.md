# Brief 708 — LP08 final-source campaign and disabled-path cost

**Measurement-only brief. The working tree is frozen at the accepted LP08
form: the 705 install, the 707 silent-retention repair, the 707 oracle
amendments and every replayed inventory. Brief 707 is complete (gate note
`.temp/director/brief-707-gate.md`): the acceptance is green on this tree
(workspace 4,156/0/61, focused 201/0/2 at 8 and 1, five-mode LP01 parents at
8 and 1 with 88,795 byte-identical records), and the LP04 switch bar was
re-based on the physical observation. This brief re-measures the accepted
702/703 campaign bars on fresh binaries of this final source and decides the
disabled-path cost. It writes nothing in the live tree.**

## Goal

1. Reproduce, on fresh binaries built from the final source, every LP08 bar
   the 702/703 rounds established: the terminal instruction ledger, the
   ownership counters, the 386 position snapshots, stack and coverage, the
   default-path witnesses, the five falsifiers, the M1/M2/M3 identity kills,
   the direct flush and its 150 routes, and the host contract campaign.
2. Decide the disabled-path cost with the corrected 703 timing criterion:
   the layout control calibrates the panel-wide noise floor, the instruction
   instrument decides, and the timing lane stays secondary.
3. Leave the live tree byte-identical and uncommitted for my gate. The
   measurements, not the report's interpretation, are what I gate the commit
   on.

## Context pointers

- `.temp/director/brief-707-gate.md` — the accepted tree, the repair and the
  LP04 ruling. `.temp/director/brief-706-gate.md`,
  `.temp/director/brief-702-gate.md`, `.temp/director/brief-703-gate.md` —
  the accepted campaign bars and the corrected timing criterion.
- `.temp/brief-702/` — the campaign scripts (`bootstrap.py`, `campaign.py`,
  `battery.py`, `coverage.py`, `assert_position.py`, `attrib.py`) and the
  fresh 702 evidence. Read/copy only; adapt by function, never by line.
- `.temp/brief-699/` — the base arm, the identity harness
  (`identity_mutants.rs`, `identity_fixture.rs`, the hook edits and the
  appended key-route test). Read/copy only.
- `.temp/brief-698/` — the earlier campaign scripts and measure inputs.
- `.temp/brief-703/` — the build adapter, the three-arm microbench raw runs
  and the accepted disposal of its two residuals.
- `.temp/brief-707/` — the repair's tests and the accepted replay proofs;
  the recovery of any harness port is by function.
- `scripts/probe-build.py`, `scripts/core-language-microbench.sh`,
  `docs/dev/profiling.md`, `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`.

## Scope

- **No live file changes.** The working tree, including every inventory and
  every test, is the measurement input. It must be byte-identical at the end.
- All scratch goes under `.temp/brief-708/`. Retained roots are read/copy
  only; never execute a retained file in place and never modify a retained
  tree.
- The candidate snapshot is the live tree under `.temp/brief-708/src/final/`
  (exclude `.git/`, `target/`, `.temp/`). The base arm is
  `.temp/brief-699/src/base/`; verify its runtime sources against `HEAD` as
  the 702 round did (the retained base predates two documentation commits;
  the mismatch stays disclosed). The layout control is the base plus one
  behavior-neutral cold-referenced perturbator, as in 703.
- The identity arm is the final arm plus the 699 harness, ported by
  function. The harness and the mutant switches live only in the probe tree;
  no harness line, hook or switch may appear in the live tree.
- Do not run the LP01 five-mode parent, the acceptance suites, the census or
  the standing-inventory generators. They are 707's completed obligations;
  the LP01 ledger is frozen and this brief neither runs nor touches it.
- No `unsafe`, public-API, serde, feature, manifest, dependency or workload
  change. No version-control command. Do not run `scripts/director-gate.py`
  or `scripts/memory-scope.sh` (the dispatcher provides the 16G scope).

## Stages

**A — Assemble.** Snapshot the final tree; verify the snapshot against the
live tree and the base's non-scope files against it; compile each arm. Record
source hashes and build receipts. A live change, a missing harness port or a
source mismatch is a STOP.

**B — Canonical builds.** Build the arms through `scripts/probe-build.py` at
one stable path with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, `-j8`, the 703
build set (the campaign test binary and the rum-cli microbench binary for
the base, final and layout arms; six builds), one build per arm and kind,
content-addressed cache, one `--force` sha256 spot-check, and a forced
rebuild of the final arm reproducing its SHA exactly. Never
`CARGO_TARGET_DIR`; never a nested memory scope.

**C — The final-source campaign.** On the fresh final binary, against the
base arm, reproduce every accepted 702 bar and attribute every moved row:

- **Terminal work ledger:** 54 → 42 executed instructions; the 17 additions
  paired 1:1 with 17 of the 29 removals; the 12 unpaired removals exactly
  `cmpq $0,0x0(%r13)`, `je` ×2, `jne`, `movb $1,%bpl`, `or`, `sete`,
  `testb` ×2, `xorl` ×3; net `-6-7+1=-12`; no opcode increase; exactly one
  `0x328(%rsp)` restore load; no call; frame `0x2e8` in both arms; the
  explicit 8-byte memory load/store/RMW `9/11/1 → 8/10/1`; no new or wider
  memory region.
- **Ownership:** terminal counter `[1,2,1]`, `wrapper_strong=1`,
  `prepared_alive=false`; the dropped-Some counter `[0,0,0]` kills its
  unchanged assertion.
- **Position:** 386 snapshots across all 20 cases and two repeats; every
  queue cell and observation order identical.
- **Stack:** 18,304/18,896 bytes per level, residual zero, four heights.
- **Coverage:** `94/96/416/606`, `profiling=false`.
- **Default-path witnesses:** `resumed-call` 145 → 129 and `resumed-fixed`
  108 → 84, per-instruction attribution complete; four calls remain.
- **Falsifiers:** the five (`owner-loss`, `wrong-owner`, `exactly-once`,
  `position`, `dropped-Some`) each hit and kill their exact check with the
  exact exit and assertion text; each candidate with its falsifier off is
  SAME.
- **Identity kills on the fresh final binary:** M1 wrong-owner 3/3, M2
  stale-epoch 1/1, M3 wrong-record pop 3/3, controls green with
  `BRIEF699_MUTANT` unset, exit 101, one binary SHA.
- **Direct flush:** counter `[1,1,0,1,0,0,0,0,0,0]`; the 150 routes
  (`131 + 13 supplemental + 6 source-impossible`, 0 unresolved); the direct
  definition flush passes.
- **Host contract:** the eight fixtures and five lifecycle actions, plus the
  reset probe with the withdrawn guest-frame clause unasserted; exactly-once
  retirement, no suffix, no old success; old-token/queue isolation, handoff
  and unrelated-owner isolation; `HostSession::reset` retains its documented
  guest discard.
- **No collateral:** the accepted 702 correction stands: the 18
  base→reconciled rows are the 698 full-install queued-parity values, so the
  bar is per-row attribution, not "every other nonterminal delta zero".
  Every other moved row needs its raw panel and an attribution; an
  unattributed one is a STOP.

**D — Disabled-path cost.** With profiling off, build the base and final
release binaries at one path with one codegen unit and run the established
rum-vs-rum core-language protocol through
`scripts/core-language-microbench.sh`: in-kernel `AbsoluteTiming`, no
competing CPU-heavy workload, four passes per arm in the interleaved
`brief-637` order, the 120-label panel and the layout control. Apply the
corrected 703 criterion: the layout control calibrates the panel-wide noise
floor; a positive movement is a STOP only if it exceeds that envelope **and**
contradicts the instruction instrument; inside-envelope movements with equal
executed instructions are recorded as noise. For any label the instruction
instrument must settle, run the 703-style probe: the label body under the
three arms' binaries, Callgrind Ir slopes at 50/100, four draws per cell,
profiling environment cleared, stdin closed, per-function self-Ir resolved
and compared. The accepted 702 instruction ledger stays the deciding
instrument; timing offsets no instruction result.

**E — Report.** `.temp/brief-708/report.json` in `rum-report/1`: every run,
every failed attempt, every verdict line and every excluded row; run
`python3 -I scripts/report.py check 708` and return its summary. Report the
final live-tree state, which must equal the briefing state, and every
retained-root write, which must be a copy into `.temp/brief-708/`.

## Bars and exclusions

Standing exclusions, unchanged: the two documented root-context reds
(`lp655_root_context_falsifier`, `lp656_public_root_context_falsifier`) in
both arms; the six source-impossible routes
(`evaluate_inner_unstable.atom` pause/error at `eval_core/mod.rs:159`;
`evaluate_atom_fast_path.stable_symbol`/`.atom` pause/error at
`eval_core/mod.rs:174`); the legacy free-index mutants; the
`default_builtin_dispatch_spec_for_symbol` process-hash variation, never a
cost offset; the withdrawn host guest-frame preservation clause; the 703
residuals as within the calibrated noise floor; the 18 queued-parity rows as
accepted. The required-switch host suites and the LP01 ledger are not this
brief's.

A new warning, a new `UnknownHostPayload`, a changed row identity or
disposition, a disappeared row, a weakened assertion, a default-path cost, a
second terminal load, a call on the terminal branch, a wider or new memory
region, a live-tree change, or a measured row that differs from its bar
without attribution is a STOP with the exact panel. A loose check, a
re-baselined gold or an inherited number is never the answer.

## Host discipline

All scratch under `.temp/brief-708/`; never `/tmp`. The dispatcher already
runs you in the 16G memory scope. One cargo command at a time; builds
`CARGO_BUILD_JOBS=8`; tests `RUST_TEST_THREADS=8 --no-fail-fast`; a red is
reproduced at `--test-threads=1`. Timing, Callgrind and heap captures run
serially and alone. Probe builds through `scripts/probe-build.py`. Callgrind
relative offsets are decimal bytes and `*` repeats the previous address. No
version-control command.

## Deliverable

- The final-snapshot and arm hashes, the six build receipts and the forced
  SHA comparison.
- The raw controls, the raw mutant runs and the raw measured panels (terminal
  ledger, ownership, 386 positions, stack, coverage, witnesses, falsifiers,
  identity kills, direct flush, routes, host contract, no-collateral), with
  the instruction-level attribution of every moved row.
- The base/final `timings.tsv` comparison, the layout-control envelope and
  the Callgrind probe on any label it must settle.
- The `rum-report/1` report and the `report.py check 708` summary.
- The working tree unchanged and uncommitted for my gate.
