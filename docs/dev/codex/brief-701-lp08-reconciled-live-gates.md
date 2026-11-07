# Brief 701 — LP08 live gates on the reconciled completion form

**Continuation implementation brief. The 15-file transfer from brief 700 is
in the working tree; the director diagnosed the Stage A STOP and validated
the reconciliation in `.temp/director/brief-700-gate.md`. This brief verifies
that tree, replays the standing inventories, runs the acceptance suite, and
re-measures every gate on fresh binaries. The design record's bars stand; its
pre-format line numbers do not.**

The working tree currently carries 16 live files: the 15-file frozen delta
from brief 699, formatted, plus `profiling/tests/definitions.rs`. The four
reconciliation edits are the policy-guard completion, the stale-view return,
the `lp04` contract update, and the `unused_mut` removal. I validated them
locally: `cargo fmt --check` clean, no `rum-runtime` lib warning, focused
suite 192 passed / 2 standing `lp655`/`lp656` reds / 2 ignored.

## Goal

Finish the LP08 live install:

- snapshot the reconciled tree as the revised target and verify its bytes;
- replay the standing inventories before acceptance, then run the full
  acceptance suite and the required-switch host suites;
- re-measure on fresh binaries the terminal work ledger, ownership, the 386
  positions, the 18,304/18,896 stack, coverage `94/96/416/606`, the
  default-path witnesses `145→129` / `108→84`, the five falsifiers, the three
  identity kills (M1/M2/M3) on the new binary, the direct definition flush,
  the 150 key routes, the contract/host lifecycle and the no-collateral rows;
- run the rum-vs-rum core-language microbench on base and candidate;
- leave the tree uncommitted for the director's gate.

Every 698/699 number is a bar, not evidence. The formatted source's line
numbers differ from the design record; re-derive every citation from the
current tree.

## Context pointers

- `.temp/director/brief-700-gate.md` — the accepted STOP and the four
  reconciliation edits.
- `.temp/director/brief-699-gate.md`, `brief-698-gate.md` — the bars,
  exclusions and the withdrawn host clause.
- `.temp/brief-699/` — the identity harness (`identity_mutants.rs`,
  `identity_fixture.rs`, the three hook edits and the appended key-route
  test), the frozen target/base, and the assembled downstream trees. Read and
  copy only; never edit.
- `.temp/brief-698/` — the cost profile inputs, the terminal/stack/trace
  evidence and the host probe (`brief679_reset.rs`), read/copy only.
- `.temp/brief-699/drafts/implementation-lp08-final.md` — the plan basis; my
  brief overrides it.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`, `docs/dev/profiling.md`, and the
  `HostSession::reset` rustdoc (`crates/rum-host/src/session.rs:166-179`).

## Scope

The live files already changed are the complete live scope:

- the 15 frozen-delta files: `eval_call_prepare.rs`, `eval_core/call.rs`,
  `profiling/logical/{collector,completion,epoch,mod,queue_api,segments}.rs`,
  `profiling/timing.rs`, `resolution.rs`, `suspension/core.rs`,
  `suspension/core/profile_queue.rs`,
  `suspension/core/profile_queue/{arena,tests}.rs`,
  `suspension/core/storage.rs`;
- `profiling/tests/definitions.rs` (the `lp04` expectation update).

Do not change any other live file. Do not introduce `unsafe`, public-API,
serde, feature, manifest, dependency or workload changes. No version-control
command. The standing maintenance duty (retention fingerprints, certificate
cells, ownership discovery/positions, LP01, typed-slot receipts, frame-root
closed lists, rejection census) is always in scope; replay it, never hand-edit.

## The reconciled target

The working tree is the revised target. Verify these four snippets exactly
(its formatted content is the reference; a mismatch is a STOP):

1. `queue_api.rs`, `logical_insert_boundary`: the `debug_assert!` policy set
   is the full observable set —
   `Expr | OptionalExpr | Bool | Prepared | Match | ApplicableExpr |
   Incoming(_)`. `None` remains rejected.
2. `profile_queue.rs`, `attach_boundary`: the same seven policies.
3. `profile_queue.rs`, `DownValuesView::observe`: return on an invalid
   boundary epoch, a closed record (`policy()` is `None`), or a different
   queue epoch; fault only for a live non-`OptionalExpr` policy or a
   displaced/absent guest consumer in the same live epoch.
4. `profiling/tests/definitions.rs`, `lp04_rhs_paused_and_resumed_down_value`:
   the two phase rows are `[1,1,0,0,0,0,0,0,0,0]` and the
   `down_value_rhs_eval` rule row is `[1,1,0,1,0,0,0,0,0,0]`; the assertions
   stay exact.
5. `profile_queue.rs`, `ProfileQueue::new`: `let result = Self { ... }` — the
   `mut` is removed; no `#[allow(unused_mut)]`.

The 699 frozen target's bytes and SHA are superseded. Snapshot the tree as
`.temp/brief-701/src/reconciled/` before any probe build; the snapshot is the
candidate arm's source, the live base at `HEAD` (`ccd19853`) is the base arm's
source (the retained copy is `.temp/brief-699/src/base/`; verify its non-scope
files against the working tree). Never `git checkout`, `git stash` or
`git reset` the working tree; the base arm is a copy, never a checkout. The
probe overlay (the 699 downstream harness) is probe-only: port it to the
formatted sources by function, not by line; the live tree must contain no
harness, no identity hook and no mutant switch.

## Stages

**A — Verify and compile.** Snapshot the tree and record the 16 file hashes.
Verify the five snippets above; `cargo check --all-targets`; run the focused
suite (`cargo test -p rum-runtime --lib -- runtime::profiling::
runtime::suspension::`). Expect 192 passed / 2 failed / 2 ignored, with only
`lp655_root_context_falsifier` and `lp656_public_root_context_falsifier` red
(the standing two-arm exclusions). A different red is a STOP, not a repair.

**B — Replay, then acceptance.** Replay the standing inventories as the last
step of the edit stage, before the first acceptance suite: recompute the
retention fingerprints, run `local_production_acceptance` in both feature
builds, reconcile the ownership/position rows, replay LP01, the typed-slot
receipts and the frame-root lists, and check the rejection census. Then run
the repository's gate script (`python3 -I scripts/director-gate.py run 701`)
and the required-switch host suites
(`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` for the lp03/04/05 adopted-phase and
lp658 reset cases); the two lp655/lp656 reds stay disclosed exclusions. The
gate script's workspace step is the single full `cargo test --workspace` with
the standing two-arm reds skipped; a plain unfiltered run that reports those
two as failures is not a defect. Run `cargo fmt --check` separately. A red
whose only cause is a stale derived cell is an ordering artifact: replay and
rerun; never weaken a test or hand-edit a cell.

**C — Fresh probe campaign.** Build the base and candidate probe arms through
`scripts/probe-build.py` (stable `.temp/probe/active`, one build per arm/kind,
one `--force` sha256 spot-check; never `CARGO_TARGET_DIR`). Run the unmodified
controls first, then:

- the terminal work ledger: 54 → 42; 17 paired additions; 29 removals
  attributed; 12 unpaired removals; `-6-7+1=-12`; no opcode increase; one
  `0x328(%rsp)` load; no call; frame `0x2e8`; memory `9/11/1 → 8/10/1`;
- the ownership counter `[1,2,1]`, `wrapper_strong=1`, `prepared_alive=false`
  and the dropped-Some `[0,0,0]` kill;
- the 386-snapshot position check, all 20 cases and two repeats;
- the stack 18,304/18,896 bytes/level, residual zero;
- coverage `94/96/416/606`, `profiling=false`;
- the default-path witnesses `resumed-call` 145 → 129 and `resumed-fixed`
  108 → 84, complete per-instruction attribution, four calls remain;
- the five falsifiers (`owner-loss`, `wrong-owner`, `exactly-once`,
  `position`, `dropped-Some`) with their exact exits and assertion text;
- the three identity kills on the new binary: M1 3/3, M2 1/1, M3 3/3,
  controls green with `BRIEF699_MUTANT` unset, exact checks, exit 101, same
  binary;
- the direct definition flush `[1,1,0,1,0,0,0,0,0,0]`, the 150 routes
  (`131 + 13 supplemental + 6 source-impossible`, 0 unresolved) and the
  contract/host lifecycle obligations (exactly-once retirement, no suffix, no
  old success; old-token/queue isolation, handoff, unrelated-owner isolation;
  reset retains its documented guest discard);
- no collateral movement: 2,453 unchanged rows, 8 terminal rows, 5 qualified
  `default_builtin_dispatch_spec_for_symbol` lookup rows; every other
  nonterminal delta zero.

Then the core-language microbench on the base and candidate builds; compare
every `timings.tsv` label, the hot labels and same-tier neighbors; profiling
off, in-kernel `AbsoluteTiming`, no competing CPU-heavy workload.

**D — Report.** Write `.temp/brief-701/report.json` in `rum-report/1`,
quoting every run, failed attempt, verdict line and excluded row; run
`python3 -I scripts/report.py check 701` and return its summary and all
compliance verdicts.

## Bars and exclusions

The bars are the accepted 698/699 values above; the design record's
pre-format line numbers are not. Re-derive the ripple inventory from the
formatted target: four `pause_call_stage_at_finish` bindings, two
`pause_phase_at_finish` fixed-point sites, eight definition finishes, 50
keys, 11 `complete_phase` callers; the counts must hold, the line numbers are
new.

Standing exclusions, restated:

- the two documented root-context reds `lp655_root_context_falsifier` and
  `lp656_public_root_context_falsifier`, in both arms;
- the required-switch host suites run separately, not in the plain workspace;
- the six source-impossible routes (`evaluate_inner_unstable.atom`
  pause/error at `eval_core/mod.rs:159`; `evaluate_atom_fast_path
  .stable_symbol`/`.atom` pause/error at `eval_core/mod.rs:174`);
- the legacy free-index mutants (physical timing only, logical key
  unchanged);
- the `default_builtin_dispatch_spec_for_symbol` process-hash variation,
  never a cost offset;
- the withdrawn host guest-frame preservation clause; do not revive the
  no-discard test.

A new warning, a new `UnknownHostPayload`, a changed row identity or
disposition, a disappeared row, a weakened assertion, a default-path cost, a
second terminal load, a call on the terminal branch, a wider/new memory
region, or a measured row that differs from its bar without attribution is a
STOP with the exact panel.

## Host discipline

All scratch under `.temp/brief-701/`; never `/tmp`. The dispatcher already
runs you in the 16G memory scope; do not call `systemd-run` or
`memory-scope.sh` yourself. One cargo command at a time; builds
`CARGO_BUILD_JOBS=8`; tests `RUST_TEST_THREADS=8 --no-fail-fast`; a red is
reproduced at `--test-threads=1`. Timing, Callgrind and heap captures run
serially and alone. Probe builds through `scripts/probe-build.py`; Callgrind
relative offsets are decimal bytes and `*` repeats the previous address.
After the director's commit+push the director runs `clean-temp.py`; you run no
version-control command.

## Deliverable

- The 16-file snapshot and hashes, the five-snippet verification, the
  focused-suite and acceptance verdict lines.
- The inventory replay record, with every changed row.
- The build receipts, the forced SHA comparison, and every raw control,
  mutant and measured artifact.
- The terminal ledger, ownership, 386-position, stack, coverage, witness,
  falsifier, identity, direct-flush, 150-route, contract/host and
  no-collateral records, each with its verbatim verdict line.
- The base/candidate `timings.tsv` comparison.
- The `rum-report/1` report and the `report.py check 701` summary.
- The working tree uncommitted for my gate.
