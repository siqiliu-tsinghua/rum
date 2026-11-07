# Brief 702 — LP08 fresh campaign on the reconciled install

**Continuation implementation brief. The 16-file reconciled install and the
11 replay-generated inventory files are in the working tree, uncommitted.
Brief 701 verified Stage A (focused suite 192 passed / 2 standing reds / 2
ignored at 8 and 1 threads; `cargo fmt --check` clean; no new warning) and
stopped at Stage B on the LP01 lexical ledger — a real measured conflict I
have ruled is the install's designed effect and will re-derive after this
campaign (gate note `.temp/director/brief-701-gate.md`). This brief runs the
fresh measured campaign on that install. It does not touch the live tree and
does not run the LP01 ledger or the acceptance suite.**

## Goal

Re-measure, on fresh binaries built from the reconciled live source, every
LP08 acceptance bar that the 698/699 design rounds established, so that I can
gate the install. The campaign is the publication's proof: a form is accepted
only when the fresh candidate arm reproduces every bar against the base arm
and every falsifier and identity mutant kills its exact check.

The working tree must be unchanged at the end: the 16 source files and the
11 inventory files are the arm's source, never the worker's edit target.

## Context pointers

- `.temp/director/brief-701-gate.md` — this brief's starting ruling.
- `.temp/brief-701/report.json` and `runs/` — Stage A evidence and the LP01
  panel; read only.
- `.temp/brief-699/` — the identity harness (`identity_mutants.rs`,
  `identity_fixture.rs`, the three hook edits and the appended key-route
  test), `bootstrap699.py`/`assemble699.py`, the frozen/base/identity arms,
  and `inherited-evidence699.json`; read/copy only, never edit.
- `.temp/brief-698/` — the campaign scripts (`bootstrap.py`, `campaign.py`,
  `battery.py`, `coverage.py`, `assert_position.py`, `attrib.py`, the measure
  inputs) and the 698 raw evidence; read/copy only, never edit.
- `.temp/brief-699/drafts/implementation-lp08-final.md` — the plan basis; my
  brief overrides it.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/dev/code-organization-guidelines.md`, `docs/dev/profiling.md`, and the
  `HostSession::reset` rustdoc (`crates/rum-host/src/session.rs:166-179`).

## Scope

**No live file changes.** The 16 reconciled source files and the 11 replay
inventory files are inputs. Do not edit, reformat, move or re-generate any
live file, and do not touch the LP01 `logical_counter_port` test or its
ledger — the ledger is red by design until brief 703 and must stay
byte-identical to the working tree.

All scratch goes under `.temp/brief-702/`. Retained roots
(`.temp/brief-698/`, `.temp/brief-699/`, `.temp/brief-701/`) are read/copy
only; never execute a file in place and never modify a retained tree.

Do not introduce `unsafe`, public-API, serde, feature, manifest, dependency
or workload changes. No version-control command. Do not run
`scripts/director-gate.py` (it invokes git and `memory-scope.sh`; it is my
instrument); run the component suites directly instead. Do not run the
`cargo test --workspace` acceptance; it is deferred to brief 703.

The standing maintenance duty is not this brief's: the inventories were
replayed in 701 and the certificate/ownership/fingerprint cells are already
in the working tree.

## Arms

Build one base arm and one candidate arm, plus the candidate identity-mutant
arm, exactly as the 698/699 rounds did:

- **base:** `.temp/brief-699/src/base/` — verify its 16 scope-adjacent
  runtime files against `HEAD` before use (the retained base predates two
  documentation commits; that mismatch is disclosed and does not affect the
  crate sources). Use that arm's own inventory files.
- **candidate:** the working tree's 16 source files plus the 11 replay
  inventory files plus every non-scope file (the retained base's non-scope
  files are the reference; the 701 snapshot verdict checked them). Snapshot
  it under `.temp/brief-702/src/reconciled/`; the live tree is never the
  probe build path.
- **identity:** the candidate arm plus the 699 identity harness, ported by
  function to the formatted sources. The ports may not move a line in the
  live tree; the harness and every mutant switch live only in the probe
  tree.

Assemble the probe-only overlays (the 698/699 downstream harness: the
effective/lifecycle/continuation/key-matrix probes, the battery, the trace
and coverage probes) by function, not by line; the formatted source has new
line numbers. No live harness, no identity hook and no mutant switch may
appear in the working tree.

## Stages

**A — Assemble and compile.** Snapshot the candidate; verify the 16 files
against the working tree and the base's non-scope files against it; compile
each arm. Record the source hashes and the build receipts. A source
mismatch, a missing harness port or a live-tree change is a STOP.

**B — Fresh build and controls.** Build the base, candidate and identity
arms through `scripts/probe-build.py` (stable `.temp/probe/active`, one
release `--test` build per arm/kind, content-addressed cache, one `--force`
sha256 spot-check; never `CARGO_TARGET_DIR`, never a nested memory scope).
Run the unmodified controls first, before any mutant:

- the three direct-flush controls and the four exact identity fixtures;
- the 20/20 pair battery and the 67/67 mirror battery;
- the five falsifiers' controls (`owner-loss`, `wrong-owner`,
  `exactly-once`, `position`, `dropped-Some`) with `BRIEF699_MUTANT` unset;
- the identity controls with the mutants unset; the same binary SHA is used
  for the control and the mutant runs.

A control red is a STOP, not a mutant result.

**C — The measured campaign.** On the fresh binaries, reproduce every bar
and attribute every moved row:

- **Terminal work ledger (base vs candidate):** 54 → 42 executed
  instructions on the argument case; 17 additions paired 1:1 with 17 of the
  29 removals; the 12 unpaired removals exactly `cmpq $0,0x0(%r13)`, `je`
  ×2, `jne`, `movb $1,%bpl`, `or`, `sete`, `testb` ×2, `xorl` ×3; net
  `-6-7+1=-12`; no opcode increase; exactly one `0x328(%rsp)` restore load;
  no call; frame `0x2e8` in both arms; explicit 8-byte memory
  load/store/RMW `9/11/1 → 8/10/1`; no new or wider memory region.
- **Ownership:** terminal counter `[1,2,1]`, `wrapper_strong=1`,
  `prepared_alive=false`; the dropped-Some counter `[0,0,0]` kills its
  unchanged assertion.
- **Position:** 386 snapshots across all 20 cases and two repeats; every
  queue cell and observation order identical.
- **Stack:** 18,304/18,896 bytes per level, residual zero, four heights.
- **Coverage:** `94/96/416/606`, `profiling=false`.
- **Default-path witnesses:** `resumed-call` 145 → 129 and `resumed-fixed`
  108 → 84, per-instruction attribution complete; four calls remain.
- **The five falsifiers:** each hits and kills its exact check, with the
  exact exit and assertion text; the candidate with the falsifier off is
  SAME.
- **The three identity kills on the new binary:** M1 wrong-owner 3/3, M2
  stale-epoch 1/1, M3 wrong-record pop 3/3; controls green with
  `BRIEF699_MUTANT` unset; exact checks unchanged; exit 101; one binary.
- **Direct flush:** the counter `[1,1,0,1,0,0,0,0,0,0]`; the 150 routes
  (`131 + 13 supplemental + 6 source-impossible`, 0 unresolved); the
  direct definition flush passes.
- **Contract/host lifecycle:** exactly-once retirement, no suffix, no old
  success; old-token/queue isolation, handoff and unrelated-owner
  isolation; `HostSession::reset` retains its documented guest discard (the
  withdrawn no-discard clause stays withdrawn).
- **No collateral:** 2,453 unchanged rows, 8 terminal rows and 5 qualified
  `default_builtin_dispatch_spec_for_symbol` lookup rows; every other
  nonterminal delta zero. The lookup row's process-hash variation is never
  cost evidence.

Every number above is a bar from the accepted 698/699 evidence, not
inherited proof. A measured row that differs must be attributed with its raw
panel; an unattributed change is a STOP with the exact panel. The 699 binary
SHA is provenance only.

**D — Microbench.** Build base and candidate release binaries at one path
with one codegen unit through the established harness; run the core-language
microbench with profiling off, in-kernel `AbsoluteTiming`, no competing
CPU-heavy workload. Compare every `timings.tsv` label, the hot labels and
each same-tier neighbor; a moved label needs its distribution and the
layout/control explanation, or it is a STOP. Timing is secondary; the
instruction ledger decides.

**E — Report.** Write `.temp/brief-702/report.json` in `rum-report/1`,
quoting every run, every failed attempt, every verdict line and every
excluded row; run `python3 -I scripts/report.py check 702` and return its
summary and all compliance verdicts. Report the final live-tree state (it
must equal the brief's starting state) and every retained-root write, which
must be a copy into `.temp/brief-702/` and never an edit in place.

## Bars and exclusions

Standing exclusions, restated and unchanged:

- the two documented root-context reds `lp655_root_context_falsifier` and
  `lp656_public_root_context_falsifier`, in both arms;
- the required-switch host suites (`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` for
  the lp03/04/05 adopted-phase and lp658 reset cases), which run separately
  in brief 703;
- the six source-impossible routes (`evaluate_inner_unstable.atom`
  pause/error at `eval_core/mod.rs:159`; `evaluate_atom_fast_path
  .stable_symbol`/`.atom` pause/error at `eval_core/mod.rs:174`);
- the legacy free-index mutants (physical timing only, logical key
  unchanged);
- the `default_builtin_dispatch_spec_for_symbol` process-hash variation,
  never a cost offset;
- the withdrawn host guest-frame preservation clause; do not revive the
  no-discard test;
- the LP01 `logical_counter_port` ledger conflict, red by design until brief
  703; this campaign neither runs nor repairs it.

A new warning, a new `UnknownHostPayload`, a changed row identity or
disposition, a disappeared row, a weakened assertion, a default-path cost, a
second terminal load, a call on the terminal branch, a wider/new memory
region, a live-tree change, or a measured row that differs from its bar
without attribution is a STOP with the exact panel.

## Host discipline

All scratch under `.temp/brief-702/`; never `/tmp`. The dispatcher already
runs you in the 16G memory scope; do not call `systemd-run` or
`memory-scope.sh` yourself. One cargo command at a time; builds
`CARGO_BUILD_JOBS=8`; tests `RUST_TEST_THREADS=8 --no-fail-fast`; a red is
reproduced at `--test-threads=1`. Timing, Callgrind and heap captures run
serially and alone. Probe builds through `scripts/probe-build.py`; Callgrind
relative offsets are decimal bytes and `*` repeats the previous address. Do
not run a version-control command.

## Deliverable

- The base/candidate/identity snapshot hashes, the build receipts and the
  forced SHA comparison.
- The raw controls, the raw mutant runs, the raw measured panels (terminal
  ledger, ownership, 386 positions, stack, coverage, witnesses, falsifiers,
  identity kills, direct flush, routes, contract/host, no-collateral) and
  the instruction-level attribution of every moved row.
- The base/candidate `timings.tsv` comparison.
- The `rum-report/1` report and the `report.py check 702` summary.
- The working tree unchanged and uncommitted for my gate.
