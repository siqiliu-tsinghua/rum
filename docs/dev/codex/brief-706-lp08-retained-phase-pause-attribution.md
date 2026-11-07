# Brief 706 — LP08: the retained-phase pause attribution (design round)

**The LP08 install's full workspace has exactly two red tests, both in the
LP02 host oracle (`crates/rum-host/tests/logical_profile_lifecycle.rs`):
`logical_native_nested_pause_counts` (44 vs 4) and `logical_native_lifecycle`
(18 vs 2). The same binary at HEAD is green 3/3. The cause is the install's
new pause path: `complete_phase(InputRequired)` pauses every adopted phase
`Call` on the unwind (`logical/mod.rs:982`), and the down-values exception
does the same (`:1262`), where HEAD paused only the two `compound.item`
records through the timing carrier. The `PAUSE_PARENT` cursor points past
locally-closed phase keys to the nearest non-locally-closed ancestor (the
native record), so every phase record on the unwind pauses once, the native
record's own pause is skipped, and the native aggregate's open-subtree sum
becomes 18/44. The design must decide the pause contract and specify the
repair. `Scope: NONE — no source change`.**

## Goal

Decide how the retained phase boundaries introduced by the entry split and
the logical profile's `pauses` counter relate, and draft the implementation
brief that makes the LP02 host oracle and the accepted LP08 measurements both
true. This brief changes no source: the deliverable is the evidence, the
decision and the drafted follow-up brief.

## Context pointers

- `.temp/director/brief-705-gate.md` — the gate record for this STOP; read it
  first.
- My probes (director instruments, read-only):
  `.temp/probe-705/lifecycle-segpause.log` (the pause sequence under the
  install), `.temp/probe-705/lifecycle-records.log` (the record tree),
  `.temp/probe-705/base-lifecycle-probe.log` (the same under HEAD),
  `.temp/director/gate-705/logs/workspace.log` (the red).
- `crates/rum-runtime/src/runtime/profiling/logical/segments.rs` —
  `PAUSE_PARENT` (:46–51 and its comment), `Segment::pause` (:212–241),
  `locally_closed` (:142–145), `restore` (:147–190).
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` —
  `logical_profile_test_span` (:419–446), `Call::pause` (:629–644),
  `complete_phase` (:965–987), `pause_call_stage_at_finish_slow`
  (:1234–1280).
- `crates/rum-runtime/src/runtime/profiling/logical/completion.rs`,
  `collector.rs` — the new retention and stash path.
- `crates/rum-runtime/src/runtime/profiling/logical/totals.rs` and
  `epoch/snapshot.rs` — how `pauses` accumulates and how `rows()` sums the
  open subtree.
- `crates/rum-host/tests/logical_profile_lifecycle.rs` — the oracle
  (:62–109, :169–204, :286–293).
- Accepted LP08 bars: `.temp/director/brief-704-gate.md`; the campaign
  record in `.temp/director/state.md`.
- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`.

## Scope

`Scope: NONE — no source change`. Survey and run tests only. Do not edit any
file under `crates/`, any inventory, any test, any doc. The working tree
carries the uncommitted 16-source LP08 install plus replayed inventories; it
must stay byte-identical (record its state at the start and audit it at the
end, as report rows). All scratch goes under `.temp/brief-706/`, never `/tmp`.

You may run the two host tests and the focused suite to reproduce and to
inspect; one cargo command at a time under the dispatcher's 16G scope. Do not
run `git`, `scripts/director-gate.py`, or `scripts/memory-scope.sh`.

## Evidence to establish yourself

1. **The red.** Run the lifecycle binary at 8 threads and the two children at
   one thread; quote the exact panic values and the exact `pauses_for`
   figures. Confirm the failure is deterministic and independent of test
   threading.
2. **The base.** The same binary at HEAD is green (my worktree run: 3 passed,
   0 failed, log `.temp/director/gate-705/base-lifecycle.log`). Reproduce it
   if you can do so without touching the live tree; otherwise cite my log and
   say you did not rerun it.
3. **The call-site inventory.** Every `Segment::pause` caller and every
   `Call::pause` call site in the installed tree, with `path:line`, plus which
   keys each pauses (PhaseKey = locally closed or not) and which are new since
   HEAD (the install diff). Include the timing and control carriers
   (`profiling/timing.rs`, `control_builtin.rs`, `control_timing.rs`) and the
   test-only `logical_profile_test_span`.
4. **The consumer inventory.** Every reader of `Counts.pauses` / `Totals` /
   `logical_profile_test_pauses_for`, with `path:line`, and whether the LP01
   ledger, the required/bindings inventories, the timing report or any
   production path reads it. Prove whether the counter is test-only.
5. **The mechanism, confirmed.** With the record tree (parent/child, own
   pauses, open/closed) for one failure point, show why the native row's sum
   is 18/44: how many records pause, which key each holds, what the cursor
   target is, and why the native record's own pause is skipped.

## The decision

State the pause contract, with `path:line` evidence, and recommend one:

- **Option A — silent retention.** The new `complete_phase(InputRequired)`
  and down-values retention keep the boundary and take the segment but do
  not increment the pause counter or advance the `PAUSE_PARENT` cursor
  (e.g. a `Call::retain` that mirrors `Call::pause` without
  `Segment::pause`). Then the LP02 host expectations stay exactly as they
  are; the phase records still stay open for the LP08 resumed publication.
  Answer: does this preserve the exact old counts, the timing bookkeeping
  (`park`/`end`), and the boundary publication on resume? What is the
  smallest shape, and what stops a future call site from bypassing it?
- **Option B — count the retention.** The retained phase records genuinely
  pause, so the counter is truthful as-is and the LP02 oracle is re-baselined
  to the audited new values. Then every one of the two tests' pause
  assertions (lines 101, 103, 106, 197, 201, 204, 292) needs its exact new
  value derived from the record tree, and the two contract comments
  (`segments.rs:48–50`, `:147–149`) must be updated to the new contract.
  Answer: are the new values stable and exactly derivable, and what does the
  re-baseline cost in oracle strength?
- **Option C — another option** the code supports better, with evidence.

For the recommended option, cover: the LP08 accepted bars' exposure (the
LP01 ledger's key/loss counts, ownership `[1,2,1]`, the 386 positions, the
witness ledgers; any change re-opens those bars), the full-workspace
obligations, the inventory replay duty (any `crates/` edit expires
`local-contracts.tsv`'s 68 fingerprints and the promoted cells; the census
and ownership rows may move), and the open questions for me to rule on.

## Deliverable

- The reproduction panel (runs, exact values, thread counts) and the tree
  audit.
- The call-site and consumer inventories with `path:line`.
- The confirmed mechanism and the pause contract decision with the
  recommendation.
- **The drafted implementation brief(s)** in `TEMPLATE.md` shape — one gated
  step if the shape is clear — with: the shape as code, the must-not-change
  list, the exact acceptance bars (the two host tests unchanged or the exact
  re-baseline, `cargo fmt --check`, the full workspace with only the two
  standing skips, the LP01 five-mode parent, the required-switch host suites,
  the focused suite), the replay order (fingerprints → certificates →
  inventories → workspace), and the falsifier that would catch a bypass.
- The `rum-report/1` report under `.temp/brief-706/report.json`, checked with
  `python3 -I scripts/report.py check 706`.

## Bars and exclusions

- No live edit, no git, no acceptance-suite rerun as evidence for the design.
- Existing test assertions are not edited in this round; a re-baseline
  recommendation must prove every new value, row by row, from the record tree.
- The standing exclusions restated: the two root-context reds
  (`lp655_root_context_falsifier`, `lp656_public_root_context_falsifier`) and
  the switch-gated host suites run separately with
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`. No new exclusion may be proposed.
- The install's 16 sources, the LP01 ledger and every replayed inventory are
  accepted as the carrier; this round decides only the pause attribution.

## Host discipline

All scratch under `.temp/brief-706/`. Tests `RUST_TEST_THREADS=8`,
`--no-fail-fast`, a red reproduced at `--test-threads=1`; one cargo command at
a time; timing/perf not needed for this round. Do not run version control.

## Report format

`rum-report/1` JSON at `.temp/brief-706/report.json`; every run logged before
it is parsed; every verdict line and panic line quoted verbatim; `changes`
empty (Scope NONE) with the live-tree audit as a section.
