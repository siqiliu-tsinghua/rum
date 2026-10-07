# brief-594 — G11 step A: shared accumulator chains complete without a uniqueness assumption

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-593 found **three guest-reachable kernel panics** of one class: a
consumer finalizes an `AccumulatorChain` with `Arc::try_unwrap(...)` plus a
panic, inside a frame's `resume`, while that still-rooted frame legitimately
shares an older node (the suspension driver drops a frame only after its
`resume` returns, `runtime/suspension/core.rs:777-786`). Witnesses, each
caught once: `.temp/brief-593/{probe.rs,probe-test.log}`.

| | Source | Panic |
|---|---|---|
| C1 | `TestReport[{TestCreate[InputString["report:1"],"a"],TestCreate[InputString["report:2"],"b"]},"TestsSucceededCount"]`, answers `a`, `b` (queued: `2`) | `debug_tools/test_report/report.rs:116` |
| C2 | with ``Global`assoc=<|"a"-><|"b"->0|>|>``: ``Global`assoc[InputString["assoc:1"],InputString["assoc:2"]]=9``, answers `a`, `b` (queued: `9`) | `kernel_data/association_data.rs:661` |
| C3 | `DeleteCases[{1,2},_?Function[x,(InputString["delete"];False)]]`, answered `a`, `b` (queued: `{1, 2}`) | `kernel_data/rewrite_data/suspended_state.rs:155` |

Repair the **class**, not three sites: one private completion cursor over
`AccumulatorChain<N>` (`N: Clone`) that moves a node when unique and
shallow-clones it when shared, detaching the previous link into the cursor
before handing out the payload — clients never hold a raw `Arc` they must
assert unique. Route the three consumers and `Table`'s existing safe
move-or-clone loop (`branching_loops/iteration/mod.rs:94-103`) through it.

brief-593's report (§4, §5) and `.temp/brief-593/draft-A-shared-accumulator-completion.md`
are the raw material; **this brief governs** where they differ.

## Context pointers

- brief-593 and its report; the 32-site classification
  `.temp/brief-593/uniqueness-classification.tsv` (25 unique by construction,
  4 safe fallbacks, these 3).
- `runtime/accumulator_chain.rs:13-98` (wrapper, link, cold iterative drain);
  the participation gate `rum-stdlib/src/tests/ownership_graph/accumulator_coverage.rs`
  and the destruction-depth certificates; `runtime/suspension/core.rs:764-800`.
- `debug_tools/test_report/report.rs:90-127, 462, 502-507`;
  `kernel_data/association_data.rs:612-671, 847-855` and
  `association_data/resume_frames.rs:73-78`;
  `kernel_data/rewrite_data/suspended_state.rs:103-167`,
  `cases/delete/traversal.rs:514-741`, `cases/delete/resume_frames.rs:443-465`.
- The oracle `crates/rum-stdlib/tests/suspension_oracle/` (runner,
  `descriptors.rs`, `fixtures/`, `inventory/effects.tsv`).

## Scope

**Objective:** the completion cursor, the four consumers routed through it,
the regression descriptors, and the standing and instruction gates.

**Edit surface:** `runtime/accumulator_chain.rs` and one cohesive child
`accumulator_chain/completion.rs` (private to the runtime; sibling tests
allowed); `debug_tools/test_report/report.rs`;
`kernel_data/association_data.rs`; `kernel_data/rewrite_data/suspended_state.rs`;
`branching_loops/iteration/mod.rs` (only to use the shared helper, its tests
unchanged in meaning); in the oracle, one shared-completion fixture module and
its wiring, the role factories needed, and `inventory/effects.tsv` **only** to
attach supplemental descriptors to the existing `Set.target`,
`DeleteCases.patternTest`, `DeleteCases.guard` and the TestReport-adjacent
entries already drained — **the four G11 entries stay owed** (step B attaches
them).

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed. **Authorized
inventory transitions, by name:** removing the three panic closures removes
their `UnknownConstruct` rows — the `unwrap_or_else` closure and its
`panic!` macro at `report.rs:116`, `association_data.rs:661` and
`suspended_state.rs:155` (in `unknown_captures.tsv` and `unknown_macros.tsv`);
if `Clone` is added to the three node types by derive, their existing
node-derive attribute rows change. **STOP** on any other row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code. A new frame type, if any, enters rum-runtime's frame
lists.

**Fence:** suspension rooting and finalizer order unchanged — never drop the
active frame early or unroot it to manufacture uniqueness; `Expr`/`Symbol`
representation, catalog identities and the owed ledger unchanged; the 25
class-(a) sites keep their post-retirement protocol and do not gain `Clone`;
no leases; `scripts/` unchanged. No edits to `AGENTS.md`, `TODO.md`, root
`PROGRESS.md`, earlier briefs or docs.

**Disclosure duty:** every changed function and type, every inventory row,
every measured label, any adjacent defect.

## Deliverable

The cursor owns its remaining spine through `AccumulatorChain` (so an
early-dropped cursor still drains iteratively); each step moves a unique node
or shallow-clones a shared one (payloads `Expr`/`SharedArgs` stay shared),
detaches the previous link, and returns the payload-bearing node. No
uniqueness assertion, no `Arc::strong_count` reasoning. Append stays O(1),
completion O(n); nothing is flattened at a pause. Nodes keep no manual `Drop`;
every owning previous link stays wrapped; the participation gate and the
destruction-depth certificates still pass (they prove link destruction, not
`Expr`-tree destruction).

## Tests

Red first, recorded before any production edit — each panic witness run
**once** through the oracle runner in the Rust harness (queued control first,
then the caught panic); no repeat, no abort probe. Then, green through
`Checks::ALL` with exact prompts, queue exhaustion, state, fresh requests and
lifecycle at the first and second relevant pause: TestReport input,
expected-output and `SameTest` phases each suspending in **two cases**; two
selectors in one association `Set` target; `DeleteCases` rejecting across two
siblings, mixed keep/delete, association children and early limit completion
(the drained descriptors split Decline and Twice — their cross product is what
reaches the assertion). Helper tests: unique move versus shared clone, a
retained snapshot preserved, early cursor drop, shared-tail drop on small
chains. A `rum-host` contract test that these programs complete and leave the
session usable.

## rum / Rust pitfalls (read twice)

1. Do not turn the panic into an `EvalError`, skip a value, or clone the whole
   buffer at each pause. A complete-looking result that lost a prefix is red.
2. No `cfg(test)` completion logic; no `#[should_panic]` blessing a kernel
   panic. Tests use the production helper.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: panic witnesses retired (of 3), regressions
   passing, consumers migrated, regressing labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-594/tmp …`. Bounds: focused 300 s; the oracle 900 s
per build; runtime, ownership and retention 300 s; a one-unit release build
900 s; a metric process 150 s; `cargo test --workspace --no-fail-fast` 3 000 s.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-host
scripts/check-suspension-oracle.sh 900
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Instruction gate.** **Before any production edit**, build the base from the
repository tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into
`.temp/brief-594/base/target-default` and `target-alloc` (`--features
rum-cli/alloc-count`, two arguments) and copy the binaries with hashes; then
candidates the same way in fresh directories. Panel (scratch workloads, each
result checked first): `DeleteCases` wide no-match, wide association no-match
and small leaf delete; association `Set` single-key incremental and two-key;
`Table` count, iterator and small; an ordinary two-case `TestReport`; the
controls `dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). **Noise floor first** (base against itself, two
pairs); a label regresses when both candidate samples exceed `max(+0.05%,
floor)`. Allocation counts, then two paired samples of `taskset -c 2 perf stat
-x, -e cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output first. No timing suite. At most eight
measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-594/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the oracle gate script, the release `rum-cli` on passing programs,
  `taskset`/`perf`, `valgrind --tool=callgrind`. No probe that aborts a
  process; each panic witness runs once.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files, functions and types; split assessment.
2. Verbatim red (once) and green output, the oracle in both builds, ownership,
   certificates and the broad suites; every inventory row change.
3. Panics retired (of 3); the noise floors and every measured row.
4. The four G11 entries still owed; stop-loss counters; open questions.

---

## Execution report — brief-594


### 1. Changed files, functions and types; split assessment


Implemented one runtime-private completion cursor and migrated all four consumers. All three original panic witnesses now pass. The active frame remains rooted through resume, and finalizer order is unchanged. Unique nodes move; shared nodes clone their shared payload/link fields, then the link is detached before the node is returned. The cursor owns its unfinished spine through `AccumulatorChain`, so early drop uses the existing iterative drain. Append remains O(1), completion O(n), and no pause flattens a prefix. Production nodes have no manual `Drop`; the 25 class-(a) extraction protocols did not change.

| File | Changed functions/types or wiring |
| --- | --- |
| [crates/rum-runtime/src/runtime/accumulator_chain.rs](../../../crates/rum-runtime/src/runtime/accumulator_chain.rs) | Module wiring for completion and its test sibling; existing wrapper, drain and tests unchanged. |
| [crates/rum-runtime/src/runtime/accumulator_chain/completion.rs](../../../crates/rum-runtime/src/runtime/accumulator_chain/completion.rs) | New `Completion<N>`; `AccumulatorChain::into_completion`; `Iterator for Completion::next`. |
| [crates/rum-runtime/src/runtime/accumulator_chain/completion_tests.rs](../../../crates/rum-runtime/src/runtime/accumulator_chain/completion_tests.rs) | New `Observations`, `Payload`, `Node`; `Payload::{clone,drop}`, `Node::drop` (test instrumentation only), `prefix`, `released`; four tests: `unique_completion_moves_nodes_and_detaches_every_link`, `shared_completion_clones_nodes_and_preserves_a_retained_snapshot`, `early_cursor_drop_drains_remaining_unique_links_iteratively`, `early_cursor_drop_stops_at_shared_tail_then_snapshot_drains`. |
| [crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs](../../../crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs) | `TestReportResultNode` gains derived Clone; `TestReportResultAccumulator::into_vec` consumes the cursor. |
| [crates/rum-runtime/src/runtime/kernel_data/association_data.rs](../../../crates/rum-runtime/src/runtime/kernel_data/association_data.rs) | `AssociationSelectorNode` gains derived Clone; `AssociationSelectorAccumulator::into_vec` consumes the cursor. |
| [crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs](../../../crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs) | `PersistentRewriteExprNode` gains derived Clone; `PersistentRewriteExprs::into_vec` consumes the cursor. Move-only queue and value protocols unchanged. |
| [crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs](../../../crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs) | `PersistentExprAccumulator::into_vec` routes the existing safe move-or-clone completion through the cursor; existing tests unchanged. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs) | Wires the shared-completion fixture module. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs) | `descriptors` attaches the twelve supplemental descriptors. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/shared_completion.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/shared_completion.rs) | New `descriptors`, `pause`, `base`, `step`, `finish`, `report`, `association`, `deletion`, `exercise_lifecycle`; tests `shared_completion_variants_and_lifecycle`, `original_panic_witnesses_complete`; variant lists and retained association expectation. 68 variants and 72 lifecycle obligations. Report fixtures inspect both result payloads and success flags; deletion covers repeated rejection, mixed output, association children and limit completion for both predicate and guard roles. |
| [crates/rum-host/tests/shared_accumulator_completion.rs](../../../crates/rum-host/tests/shared_accumulator_completion.rs) | New `assert_success` and `shared_accumulators_complete_and_leave_the_host_session_usable`; eight guest programs, exact prompts, state projection, fresh arithmetic and fresh input requests. |
| [crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv](../../../crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv) | Only supplemental descriptor cells of `Set.target`, `DeleteCases.patternTest`, `DeleteCases.guard`, `TestReport.explicitOption` change. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv) | One source-position update; the host payload inventory and dispositions are unchanged. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv) | The matching source-position update; its disposition and witness are unchanged. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | Three node Clone derives and three source-position updates. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | Three authorized panic-closure removals and ten source-position updates. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | Three authorized panic-macro removals and sixteen source-position updates. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | 68 fingerprint cells; all other contract fields preserved. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | Four certificate cells, two for each feature build; all other fields preserved. |



Standing maintenance changed only the ownership rows, contract fingerprints and certificate cells itemized below. No new frame type was introduced, so the closed frame lists remain unchanged. No catalog identity, owed row, representation, lease, unsafe code, script, root roadmap/progress file or unrelated source was changed. No version-control command or network access was used.

| Rust file | Characters |
| --- | --- |
| [crates/rum-runtime/src/runtime/accumulator_chain.rs](../../../crates/rum-runtime/src/runtime/accumulator_chain.rs) | 8671 |
| [crates/rum-runtime/src/runtime/accumulator_chain/completion.rs](../../../crates/rum-runtime/src/runtime/accumulator_chain/completion.rs) | 1188 |
| [crates/rum-runtime/src/runtime/accumulator_chain/completion_tests.rs](../../../crates/rum-runtime/src/runtime/accumulator_chain/completion_tests.rs) | 5006 |
| [crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs](../../../crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs) | 22475 |
| [crates/rum-runtime/src/runtime/kernel_data/association_data.rs](../../../crates/rum-runtime/src/runtime/kernel_data/association_data.rs) | 33759 |
| [crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs](../../../crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs) | 5912 |
| [crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs](../../../crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs) | 10310 |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs) | 3236 |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs) | 6192 |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/shared_completion.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/shared_completion.rs) | 11770 |
| [crates/rum-host/tests/shared_accumulator_completion.rs](../../../crates/rum-host/tests/shared_accumulator_completion.rs) | 4149 |



Split assessment: the cursor (about 1.2k characters), its test sibling (about 5k), the host test (about 4.1k) and the shared fixture (about 11.8k, below the 15k test target) are cohesive. The existing TestReport report module (about 22.5k) and association module (about 33.8k) remain in the split-plan range; this brief confines their edits to node derives and completion loops. Future boundaries are TestReport result rendering versus case driving, and association lvalue evaluation/update versus entry/index handling. The iteration file is about 10.3k including existing tests; no unrelated split or test move was made.

### 2. Verbatim verification and every inventory-row change


All Cargo commands ran serially and offline with `CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1`, and the brief TMPDIR, through `scripts/run-bounded.py`. Command-specific bounds were 300 seconds for focused/runtime/host/ownership/retention, 900 seconds per oracle build and release build, 150 seconds per metric process, and 3000 seconds for workspace. Aggregate drivers also had outer bounds. Own scratch and raw evidence are under [.temp/brief-594](../../../.temp/brief-594).

The unmodified oracle gate and bounded runner were copied into [oracle-runner/](../../../.temp/brief-594/oracle-runner) so the gate’s hard-coded `.temp/suspension-oracle` directory stays under brief-594. Its TMPDIR symlink resolves to the required brief TMPDIR; Cargo discovers and builds the real repository workspace through its ancestors. Repository scripts were not edited. Required existing tests also emitted their own fixed `target/brief-*` / `target/host-payload-destruction-contract` artifacts; those legacy harness paths were not changed. The certificate writer retained its authorized `.temp/brief-546/certificates/` path, and review copies were saved here.

Red capture ran each resumed panic witness exactly once, with its queued control first, before any production edit. No `should_panic` blessing or process-abort probe was added. Source: [red-fixture.rs](../../../.temp/brief-594/red-fixture.rs); full log: [red.log](../../../.temp/brief-594/red.log).

```text
running 3 tests
test oracle::fixtures::shared_completion::red_association_shared_selectors_once ... RED brief-594.association source=Global`assoc[InputString["assoc:1"],InputString["assoc:2"]]=9 pauses=[Pause { prompt: "assoc:1", kind: InputString, answer: "a" }, Pause { prompt: "assoc:2", kind: InputString, answer: "b" }] queued=9 projections=["9"]

thread 'oracle::fixtures::shared_completion::red_association_shared_selectors_once' panicked at crates/rum-runtime/src/runtime/kernel_data/association_data.rs:661:37:
completed association selector state must be unique
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test oracle::fixtures::shared_completion::red_delete_cases_shared_kept_siblings_once ... RED brief-594.delete-cases source=DeleteCases[{1,2},_?Function[x,(InputString["delete"];False)]] pauses=[Pause { prompt: "delete", kind: InputString, answer: "a" }, Pause { prompt: "delete", kind: InputString, answer: "b" }] queued=List[1, 2] projections=[]

thread 'oracle::fixtures::shared_completion::red_delete_cases_shared_kept_siblings_once' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:155:37:
completed rewrite output must be uniquely owned
FAILED
test oracle::fixtures::shared_completion::red_test_report_shared_results_once ... RED brief-594.report source=TestReport[{TestCreate[InputString["report:1"],"a"],TestCreate[InputString["report:2"],"b"]},"TestsSucceededCount"] pauses=[Pause { prompt: "report:1", kind: InputString, answer: "a" }, Pause { prompt: "report:2", kind: InputString, answer: "b" }] queued=2 projections=[]

thread 'oracle::fixtures::shared_completion::red_test_report_shared_results_once' panicked at crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:116:37:
completed TestReport result state must be unique
FAILED

failures:

failures:
    oracle::fixtures::shared_completion::red_association_shared_selectors_once
    oracle::fixtures::shared_completion::red_delete_cases_shared_kept_siblings_once
    oracle::fixtures::shared_completion::red_test_report_shared_results_once

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p rum-stdlib --test suspension_oracle`
```


Focused green output ([green-focused.log](../../../.temp/brief-594/green-focused.log)):
```text

test oracle::fixtures::shared_completion::original_panic_witnesses_complete ... SHARED_WITNESS brief-594.report value=2 PASS
SHARED_WITNESS brief-594.association value=9 PASS
SHARED_WITNESS brief-594.delete-cases value=List[1, 2] PASS
ok
test oracle::fixtures::shared_completion::shared_completion_variants_and_lifecycle ... SHARED_COMPLETION TestReport.explicitOption.sharedInput Middle PASS
SHARED_COMPLETION TestReport.explicitOption.sharedInput Twice PASS
SHARED_COMPLETION TestReport.explicitOption.sharedInput Error PASS
SHARED_COMPLETION TestReport.explicitOption.sharedInput AlreadyResumed PASS
SHARED_COMPLETION TestReport.explicitOption.sharedInput Nested PASS
SHARED_COMPLETION TestReport.explicitOption.sharedInput lifecycle first=PASS second=PASS
SHARED_COMPLETION TestReport.explicitOption.sharedExpectedOutput Middle PASS
SHARED_COMPLETION TestReport.explicitOption.sharedExpectedOutput Twice PASS
SHARED_COMPLETION TestReport.explicitOption.sharedExpectedOutput Error PASS
SHARED_COMPLETION TestReport.explicitOption.sharedExpectedOutput AlreadyResumed PASS
SHARED_COMPLETION TestReport.explicitOption.sharedExpectedOutput Nested PASS
SHARED_COMPLETION TestReport.explicitOption.sharedExpectedOutput lifecycle first=PASS second=PASS
SHARED_COMPLETION TestReport.explicitOption.sharedSameTest Middle PASS
SHARED_COMPLETION TestReport.explicitOption.sharedSameTest Twice PASS
SHARED_COMPLETION TestReport.explicitOption.sharedSameTest Error PASS
SHARED_COMPLETION TestReport.explicitOption.sharedSameTest AlreadyResumed PASS
SHARED_COMPLETION TestReport.explicitOption.sharedSameTest Nested PASS
SHARED_COMPLETION TestReport.explicitOption.sharedSameTest lifecycle first=PASS second=PASS
SHARED_COMPLETION Set.target.associationSelectors Middle PASS
SHARED_COMPLETION Set.target.associationSelectors Twice PASS
SHARED_COMPLETION Set.target.associationSelectors Error PASS
SHARED_COMPLETION Set.target.associationSelectors AlreadyResumed PASS
SHARED_COMPLETION Set.target.associationSelectors Nested PASS
SHARED_COMPLETION Set.target.associationSelectors lifecycle first=PASS second=PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedKeep Middle PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedKeep Twice PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedKeep Error PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedKeep AlreadyResumed PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedKeep Nested PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedKeep Decline PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedKeep lifecycle first=PASS second=PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedMixed Middle PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedMixed Twice PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedMixed Error PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedMixed AlreadyResumed PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedMixed Nested PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedMixed Decline PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedMixed lifecycle first=PASS second=PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedAssociation Middle PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedAssociation Twice PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedAssociation Error PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedAssociation AlreadyResumed PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedAssociation Nested PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedAssociation Decline PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedAssociation lifecycle first=PASS second=PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedLimit Middle PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedLimit Twice PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedLimit Error PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedLimit AlreadyResumed PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedLimit Nested PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedLimit Decline PASS
SHARED_COMPLETION DeleteCases.patternTest.sharedLimit lifecycle first=PASS second=PASS
SHARED_COMPLETION DeleteCases.guard.sharedKeep Middle PASS
SHARED_COMPLETION DeleteCases.guard.sharedKeep Twice PASS
SHARED_COMPLETION DeleteCases.guard.sharedKeep Error PASS
SHARED_COMPLETION DeleteCases.guard.sharedKeep AlreadyResumed PASS
SHARED_COMPLETION DeleteCases.guard.sharedKeep Nested PASS
SHARED_COMPLETION DeleteCases.guard.sharedKeep Decline PASS
SHARED_COMPLETION DeleteCases.guard.sharedKeep lifecycle first=PASS second=PASS
SHARED_COMPLETION DeleteCases.guard.sharedMixed Middle PASS
SHARED_COMPLETION DeleteCases.guard.sharedMixed Twice PASS
SHARED_COMPLETION DeleteCases.guard.sharedMixed Error PASS
SHARED_COMPLETION DeleteCases.guard.sharedMixed AlreadyResumed PASS
SHARED_COMPLETION DeleteCases.guard.sharedMixed Nested PASS
SHARED_COMPLETION DeleteCases.guard.sharedMixed Decline PASS
SHARED_COMPLETION DeleteCases.guard.sharedMixed lifecycle first=PASS second=PASS
SHARED_COMPLETION DeleteCases.guard.sharedAssociation Middle PASS
SHARED_COMPLETION DeleteCases.guard.sharedAssociation Twice PASS
SHARED_COMPLETION DeleteCases.guard.sharedAssociation Error PASS
SHARED_COMPLETION DeleteCases.guard.sharedAssociation AlreadyResumed PASS
SHARED_COMPLETION DeleteCases.guard.sharedAssociation Nested PASS
SHARED_COMPLETION DeleteCases.guard.sharedAssociation Decline PASS
SHARED_COMPLETION DeleteCases.guard.sharedAssociation lifecycle first=PASS second=PASS
SHARED_COMPLETION DeleteCases.guard.sharedLimit Middle PASS
SHARED_COMPLETION DeleteCases.guard.sharedLimit Twice PASS
SHARED_COMPLETION DeleteCases.guard.sharedLimit Error PASS
SHARED_COMPLETION DeleteCases.guard.sharedLimit AlreadyResumed PASS
SHARED_COMPLETION DeleteCases.guard.sharedLimit Nested PASS
SHARED_COMPLETION DeleteCases.guard.sharedLimit Decline PASS
SHARED_COMPLETION DeleteCases.guard.sharedLimit lifecycle first=PASS second=PASS
ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.47s
```


The first runtime run found only a source-hygiene failure from two `println!` diagnostics in the new helper test sibling (`1142 passed; 1 failed; 3 ignored`). Those diagnostics now use `eprintln!`, matching the existing tests; no assertion or production behavior changed. The retained first log is [runtime.log](../../../.temp/brief-594/runtime.log). The green runtime log is [runtime-green.log](../../../.temp/brief-594/runtime-green.log).

```text
test runtime::accumulator_chain::completion_tests::early_cursor_drop_drains_remaining_unique_links_iteratively ... early completion cursor: 16 nodes, maximum payload drop depth=1
test runtime::accumulator_chain::completion_tests::early_cursor_drop_stops_at_shared_tail_then_snapshot_drains ... completion shared tail: 8 nodes, maximum payload drop depth=1
test runtime::accumulator_chain::completion_tests::shared_completion_clones_nodes_and_preserves_a_retained_snapshot ... ok
test runtime::accumulator_chain::completion_tests::unique_completion_moves_nodes_and_detaches_every_link ... ok
test runtime::accumulator_chain::tests::exclusive_depth_16 ... exclusive length=16: maximum payload drop depth=1
test runtime::accumulator_chain::tests::exclusive_depth_64 ... exclusive length=64: maximum payload drop depth=1
test runtime::accumulator_chain::tests::live_weak_does_not_prevent_drain ... live Weak length=16: maximum payload drop depth=1
test runtime::accumulator_chain::tests::shared_tail_stops_then_drains ... shared tail first prefix: 8 nodes/payloads, maximum payload drop depth=1
shared tail second prefix + tail: 24 nodes/payloads, maximum payload drop depth=1
test result: ok. 1143 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.62s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.48s
```


These are link-destruction depth certificates; they do not certify arbitrary Expr-tree destruction.

Host output ([host.log](../../../.temp/brief-594/host.log)):
```text
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 59.52s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test shared_accumulators_complete_and_leave_the_host_session_usable ... HOST_SHARED_COMPLETION TestReport[{TestCreate[InputString["report:1"],"a"],TestCreate[InputString["report:2"],"b"]},"TestsSucceededCount"] => 2 PASS
HOST_SHARED_COMPLETION TestReport[{TestCreate["a",InputString["expected:1"]],TestCreate["b",InputString["expected:2"]]},"TestsSucceededCount"] => 2 PASS
HOST_SHARED_COMPLETION TestReport[{TestCreate[1,1],TestCreate[2,2]},"TestsSucceededCount",SameTest->Function[{a,b},(InputString["same"];SameQ[a,b])]] => 2 PASS
HOST_SHARED_COMPLETION Global`assoc[InputString["assoc:1"],InputString["assoc:2"]]=9 => 9 PASS
HOST_SHARED_COMPLETION DeleteCases[{1,2},_?Function[x,(InputString["delete"];False)]] => List[1, 2] PASS
HOST_SHARED_COMPLETION DeleteCases[{1,2,3},_?Function[x,(If[x<=2,InputString["mixed"]];x==2)]] => List[1, 3] PASS
HOST_SHARED_COMPLETION DeleteCases[<|"a"->1,"b"->2|>,_?Function[x,(InputString["children"];False)]] => Association[Rule["a", 1], Rule["b", 2]] PASS
HOST_SHARED_COMPLETION DeleteCases[{1,2,3,4},_?Function[x,(InputString["limit"];x==2)],{1},1] => List[1, 3, 4] PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.93s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.63s
```


Oracle, both builds ([oracle.log](../../../.temp/brief-594/oracle.log)):
```text
LIFECYCLE obligations=2466 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=237 NoLocalGuestEffect=406 Boundary=5 entries=465 descriptors=415 historical=209 owed=114
OWED families={"associations": 13, "associations_transform": 19, "debug": 4, "evaluator": 1, "schema": 4, "scoping": 1, "strings": 24, "strings_match": 38, "strings_text": 10}
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.14s
LIFECYCLE obligations=2466 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=237 NoLocalGuestEffect=400 Boundary=5 entries=465 descriptors=415 historical=209 owed=114
OWED families={"associations": 13, "associations_transform": 19, "debug": 4, "evaluator": 1, "schema": 4, "scoping": 1, "strings": 24, "strings_match": 38, "strings_text": 10}
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.01s
```


Ownership ([ownership.log](../../../.temp/brief-594/ownership.log)):
```text
test tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates ... uncovered accumulator ownership paths: 0
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7677 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 63.70s
```


Fingerprint replay used the exact length-delimited FNV algorithm from `contracts.rs` / `interface.rs`, first reproducing all initial cells from the preserved starting sources. [replay-fingerprints.py](../../../.temp/brief-594/replay-fingerprints.py) and [fingerprints.log](../../../.temp/brief-594/fingerprints.log) record that replay. It avoids changing the legacy fingerprint writer’s fixed output path. The final broad tests validate the resulting cells. Both initial certificate runs produced zero rejection rows and failed only at the expected stale C01 cell; the generated cells were copied with all contract fields checked. Logs: [certificates-true-replay.log](../../../.temp/brief-594/certificates-true-replay.log), [certificates-false-replay.log](../../../.temp/brief-594/certificates-false-replay.log).

Certificate hash=true ([certificates-true-green.log](../../../.temp/brief-594/certificates-true-green.log)):
```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 19.12s
```


Certificate hash=false ([certificates-false-green.log](../../../.temp/brief-594/certificates-false-green.log)):
```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```
```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.71s
```


Formatting: `cargo fmt --check` exited 0; [fmt-final.log](../../../.temp/brief-594/fmt-final.log).

Workspace (`cargo test --workspace --no-fail-fast -- --nocapture`) exited 0. Full log: [workspace.log](../../../.temp/brief-594/workspace.log). Verbatim target/result and skip lines:
```text
     Running unittests src/main.rs (.temp/brief-594/target-tests/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/cli.rs (.temp/brief-594/target-tests/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/input_requests.rs (.temp/brief-594/target-tests/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/terminal.rs (.temp/brief-594/target-tests/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (.temp/brief-594/target-tests/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (.temp/brief-594/target-tests/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
     Running tests/adapter_contract.rs (.temp/brief-594/target-tests/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (.temp/brief-594/target-tests/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/guest_loading_resume.rs (.temp/brief-594/target-tests/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/host_payload_destruction_contract.rs (.temp/brief-594/target-tests/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.20s
     Running tests/installer.rs (.temp/brief-594/target-tests/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (.temp/brief-594/target-tests/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_contract.rs (.temp/brief-594/target-tests/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/random_password_policy.rs (.temp/brief-594/target-tests/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/shared_accumulator_completion.rs (.temp/brief-594/target-tests/debug/deps/shared_accumulator_completion-067001aa2ecf9629)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/stress.rs (.temp/brief-594/target-tests/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/suspended_timeout_contract.rs (.temp/brief-594/target-tests/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
     Running tests/suspension_abandonment_contract.rs (.temp/brief-594/target-tests/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_jupyter-edb9ee3aea58135e)
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
     Running unittests src/bin/rum-jupyter-kernel.rs (.temp/brief-594/target-tests/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
     Running tests/input_abandonment.rs (.temp/brief-594/target-tests/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/input_continuation.rs (.temp/brief-594/target-tests/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1143 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.53s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1181.54s
     Running unittests src/bin/generate-builtin-reference.rs (.temp/brief-594/target-tests/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (.temp/brief-594/target-tests/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/destructuring_continuations.rs (.temp/brief-594/target-tests/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/guest_loading_resume.rs (.temp/brief-594/target-tests/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-594/target-tests/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.13s
     Running tests/loop_continuations.rs (.temp/brief-594/target-tests/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/option_reader_continuations.rs (.temp/brief-594/target-tests/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
     Running tests/rule_pattern_continuations.rs (.temp/brief-594/target-tests/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
     Running tests/schema_continuations.rs (.temp/brief-594/target-tests/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
     Running tests/sow_trace_continuations.rs (.temp/brief-594/target-tests/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
     Running tests/suspension_oracle.rs (.temp/brief-594/target-tests/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.43s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.54s
     Running unittests src/lib.rs (.temp/brief-594/target-tests/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.24s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.70s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


Native ZeroMQ: **9 platform skips**, explicitly reported as skips even though their Rust test functions return normally. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` permits the existing harness to skip only when loopback binding is denied. No skipped native transport behavior is claimed as tested.

Inventory reconciliation counts ([ownership-reconciliation.log](../../../.temp/brief-594/ownership-reconciliation.log)):
```text
host_payloads.tsv before 65 after 65 changed 1 removed 0 added 0
unknown_attributes.tsv before 1498 after 1498 changed 6 removed 0 added 0
unknown_captures.tsv before 2949 after 2946 changed 10 removed 3 added 0
unknown_macros.tsv before 3147 after 3144 changed 16 removed 3 added 0
unknown_storage.tsv before 89 after 89 changed 0 removed 0 added 0
self_reaching.tsv before 59 after 59 changed 0 removed 0 added 0
host_payload_ledger.tsv before None after None changed 1 removed 0 added 0
```


Every ownership row transition follows. The complete before/after strings, including retained host dispositions and witnesses, are in [ownership-row-changes.json](../../../.temp/brief-594/ownership-row-changes.json).

| Inventory | Initial row | Before location | After location | Reason | Construct |
| --- | --- | --- | --- | --- | --- |
| host_payloads.tsv | 16 | crates/rum-runtime/src/runtime/accumulator_chain.rs:27:22 | crates/rum-runtime/src/runtime/accumulator_chain.rs:32:22 | source position only | public input rum_runtime::runtime::accumulator_chain::impl(AccumulatorChain < N >)@25::new(link)/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned generic parameter N : AccumulatorChainNode → public input rum_runtime::runtime::accumulator_chain::impl(AccumulatorChain < N >)@30::new(link)/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned generic parameter N : AccumulatorChainNode |
| unknown_attributes.tsv | 94 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:39:1 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:39:1 | brief-594 explicitly authorized Clone node derive | unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode) → unresolved attribute/expansion derive (Clone , FrameRootTrace , AccumulatorChainNode) |
| unknown_attributes.tsv | 256 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:619:1 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:619:1 | brief-594 explicitly authorized Clone node derive | unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode) → unresolved attribute/expansion derive (Clone , FrameRootTrace , AccumulatorChainNode) |
| unknown_attributes.tsv | 259 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:692:1 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:688:1 | source position only | unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq) |
| unknown_attributes.tsv | 296 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:114:1 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:108:1 | source position only | unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq) |
| unknown_attributes.tsv | 297 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:156:1 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:150:1 | source position only | unresolved attribute/expansion derive (rum_derive :: FrameRootTrace) |
| unknown_attributes.tsv | 480 | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:110:1 | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:110:1 | brief-594 explicitly authorized Clone node derive | unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode) → unresolved attribute/expansion derive (Clone , FrameRootTrace , AccumulatorChainNode) |
| unknown_captures.tsv | 238 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:308:48 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:304:48 | source position only | closure captures: \|expected\| (capture types/modes unresolved) |
| unknown_captures.tsv | 239 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:383:48 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:379:48 | source position only | closure captures: \|expected\| (capture types/modes unresolved) |
| unknown_captures.tsv | 240 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:604:14 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:600:14 | source position only | closure captures: \|message\| (capture types/modes unresolved) |
| unknown_captures.tsv | 618 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:798:23 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:794:23 | source position only | closure captures: \|definition\| (capture types/modes unresolved) |
| unknown_captures.tsv | 619 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:891:56 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:887:56 | source position only | closure captures: \|depth\| (capture types/modes unresolved) |
| unknown_captures.tsv | 620 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:935:56 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:931:56 | source position only | closure captures: \|depth\| (capture types/modes unresolved) |
| unknown_captures.tsv | 621 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:967:37 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:963:37 | source position only | closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv | 622 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:1034:47 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:1030:47 | source position only | closure captures: \|entry\| (capture types/modes unresolved) |
| unknown_captures.tsv | 623 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:1062:50 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:1058:50 | source position only | closure captures: \|entry\| (capture types/modes unresolved) |
| unknown_captures.tsv | 666 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:169:24 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:163:24 | source position only | closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv | 237 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:116:33 | removed | brief-594 explicitly authorized panic/unwrap_or_else retirement | closure captures: \|_\| (capture types/modes unresolved) |
| unknown_captures.tsv | 617 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:661:33 | removed | brief-594 explicitly authorized panic/unwrap_or_else retirement | closure captures: \|_\| (capture types/modes unresolved) |
| unknown_captures.tsv | 900 | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:155:33 | removed | brief-594 explicitly authorized panic/unwrap_or_else retirement | closure captures: \|_\| (capture types/modes unresolved) |
| unknown_macros.tsv | 155 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:287:17 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:283:17 | source position only | unexpanded macro crate :: resume_or_suspend! (ends at 287:112) → unexpanded macro crate :: resume_or_suspend! (ends at 283:112) |
| unknown_macros.tsv | 156 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:330:17 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:326:17 | source position only | unexpanded macro crate :: resume_or_suspend! (ends at 330:120) → unexpanded macro crate :: resume_or_suspend! (ends at 326:120) |
| unknown_macros.tsv | 157 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:406:17 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:402:17 | source position only | unexpanded macro crate :: resume_or_suspend! (ends at 406:123) → unexpanded macro crate :: resume_or_suspend! (ends at 402:123) |
| unknown_macros.tsv | 158 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:495:17 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:491:17 | source position only | unexpanded macro crate :: resume_or_suspend! (ends at 495:116) → unexpanded macro crate :: resume_or_suspend! (ends at 491:116) |
| unknown_macros.tsv | 159 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:560:33 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:556:33 | source position only | unexpanded macro vec! (ends at 578:10) → unexpanded macro vec! (ends at 574:10) |
| unknown_macros.tsv | 160 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:613:23 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:609:23 | source position only | unexpanded macro vec! (ends at 653:6) → unexpanded macro vec! (ends at 649:6) |
| unknown_macros.tsv | 161 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:670:24 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:666:24 | source position only | unexpanded macro vec! (ends at 688:6) → unexpanded macro vec! (ends at 684:6) |
| unknown_macros.tsv | 427 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:710:17 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:706:17 | source position only | unexpanded macro matches! (ends at 710:116) → unexpanded macro matches! (ends at 706:116) |
| unknown_macros.tsv | 428 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:746:41 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:742:41 | source position only | unexpanded macro matches! (ends at 749:18) → unexpanded macro matches! (ends at 745:18) |
| unknown_macros.tsv | 429 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:751:17 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:747:17 | source position only | unexpanded macro crate :: resume_or_suspend! (ends at 765:18) → unexpanded macro crate :: resume_or_suspend! (ends at 761:18) |
| unknown_macros.tsv | 430 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:834:21 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:830:21 | source position only | unexpanded macro crate :: resume_or_suspend! (ends at 834:134) → unexpanded macro crate :: resume_or_suspend! (ends at 830:134) |
| unknown_macros.tsv | 431 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:866:21 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:862:21 | source position only | unexpanded macro crate :: resume_or_suspend! (ends at 866:121) → unexpanded macro crate :: resume_or_suspend! (ends at 862:121) |
| unknown_macros.tsv | 432 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:906:21 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:902:21 | source position only | unexpanded macro format! (ends at 906:75) → unexpanded macro format! (ends at 902:75) |
| unknown_macros.tsv | 433 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:949:21 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:945:21 | source position only | unexpanded macro format! (ends at 949:75) → unexpanded macro format! (ends at 945:75) |
| unknown_macros.tsv | 434 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:1001:8 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:997:8 | source position only | unexpanded macro matches! (ends at 1001:55) → unexpanded macro matches! (ends at 997:55) |
| unknown_macros.tsv | 485 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:143:9 | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:137:9 | source position only | unexpanded macro matches! (ends at 143:47) → unexpanded macro matches! (ends at 137:47) |
| unknown_macros.tsv | 154 | crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:116:37 | removed | brief-594 explicitly authorized panic/unwrap_or_else retirement | unexpanded macro panic! (ends at 116:95) |
| unknown_macros.tsv | 426 | crates/rum-runtime/src/runtime/kernel_data/association_data.rs:661:37 | removed | brief-594 explicitly authorized panic/unwrap_or_else retirement | unexpanded macro panic! (ends at 661:98) |
| unknown_macros.tsv | 781 | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:155:37 | removed | brief-594 explicitly authorized panic/unwrap_or_else retirement | unexpanded macro panic! (ends at 155:94) |
| host_payload_ledger.tsv | 17 | crates/rum-runtime/src/runtime/accumulator_chain.rs:27:22 | crates/rum-runtime/src/runtime/accumulator_chain.rs:32:22 | host row source position only; disposition/witness unchanged | public input rum_runtime::runtime::accumulator_chain::impl(AccumulatorChain < N >)@25::new(link)/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned generic parameter N : AccumulatorChainNode → public input rum_runtime::runtime::accumulator_chain::impl(AccumulatorChain < N >)@30::new(link)/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned generic parameter N : AccumulatorChainNode |



Every local-contract row changed only its fingerprint (68 rows; all other fields identical):
| Row | ID | Before | After |
| --- | --- | --- | --- |
| 2 | B01 | fnv1a64:7f4bb8a77e20a25c | fnv1a64:66f1cf146a632660 |
| 3 | B02 | fnv1a64:c25ecb2859b75d81 | fnv1a64:6cb46ccfd7fa9351 |
| 4 | B03 | fnv1a64:ddbf7b6583592db2 | fnv1a64:c8cc4403920b1262 |
| 5 | B04 | fnv1a64:daf8bd876007a6a4 | fnv1a64:01747b152c0c3d58 |
| 6 | B05 | fnv1a64:1d4c40a7bfecadd5 | fnv1a64:69fdf7b4be7d10bd |
| 7 | B06 | fnv1a64:7cd1a8a4c21a1d78 | fnv1a64:b8ca75a08b805164 |
| 8 | B07 | fnv1a64:ede55fc648cf3b65 | fnv1a64:119b524e055bbc4d |
| 9 | B08 | fnv1a64:288b1d74598f20a6 | fnv1a64:575cc390aaad342e |
| 10 | B09 | fnv1a64:d37e9397372c7860 | fnv1a64:f15505849cfe3e5c |
| 11 | B10 | fnv1a64:011087ebb8badd07 | fnv1a64:ca2c074584e683f3 |
| 12 | B11 | fnv1a64:9c506ee479fbcca4 | fnv1a64:c2cc2c7246006358 |
| 13 | B12 | fnv1a64:928b139313350d60 | fnv1a64:b06185807906d35c |
| 14 | I01 | fnv1a64:5d2da3a4ead898dc | fnv1a64:44d3ba11d71b1ce0 |
| 15 | I02 | fnv1a64:748dbe3bb82335d5 | fnv1a64:c13f7548b6b398bd |
| 16 | I03 | fnv1a64:45f1a98320475620 | fnv1a64:e42878d4c65fa91c |
| 17 | I04 | fnv1a64:83a87664ac77bfa1 | fnv1a64:e1c5f166b420ea71 |
| 18 | P01 | fnv1a64:5e376a346c5b5818 | fnv1a64:a6399fd467900704 |
| 19 | P02 | fnv1a64:62b31e811a173fb7 | fnv1a64:ecf5d2ff7214c263 |
| 20 | P03 | fnv1a64:3d6280111d50278a | fnv1a64:afe58d8e75eced8a |
| 21 | P04 | fnv1a64:b739e93dde607981 | fnv1a64:618f8ae55ca3af51 |
| 22 | P05 | fnv1a64:b18711cfc50b960c | fnv1a64:62446d29c80d7dd0 |
| 23 | P06 | fnv1a64:e30a38639f64a6e0 | fnv1a64:00e0aa5105366cdc |
| 24 | P070 | fnv1a64:1d57d4a57dda20e0 | fnv1a64:3b2e4692e3abe6dc |
| 25 | P080 | fnv1a64:71cfe90186e0e3c7 | fnv1a64:859de287665482b3 |
| 26 | P090 | fnv1a64:f1c250f94c446241 | fnv1a64:67c3380a66648711 |
| 27 | P100 | fnv1a64:474a50765e9f6d34 | fnv1a64:1b09185fa9b96b68 |
| 28 | P110 | fnv1a64:d61f349130b4007f | fnv1a64:ff4d789e47e03cfb |
| 29 | P120 | fnv1a64:202478035b7ecdc6 | fnv1a64:6166f28efae708ce |
| 30 | P130 | fnv1a64:91b2b92377831565 | fnv1a64:b568abab340f964d |
| 31 | P140 | fnv1a64:14e653cffe8b44f9 | fnv1a64:4b3fc21e18315239 |
| 32 | P150 | fnv1a64:7c5b9e214862e661 | fnv1a64:8ca84e0f54ccde31 |
| 33 | P071 | fnv1a64:388b89f2e9d41f05 | fnv1a64:91d3b4541e317eed |
| 34 | P081 | fnv1a64:074a6004f8db2180 | fnv1a64:3b62687e95b0767c |
| 35 | P091 | fnv1a64:7d758eb184554fd8 | fnv1a64:d64fb8ccee1be4c4 |
| 36 | P101 | fnv1a64:ce22255e66179d7f | fnv1a64:f750696b7d43d9fb |
| 37 | P111 | fnv1a64:60c23dd2f9c595f0 | fnv1a64:b3d227e3e4aea7ec |
| 38 | P121 | fnv1a64:3e8761c9846ff333 | fnv1a64:b322e941b7748677 |
| 39 | P131 | fnv1a64:78a2dfed965bc824 | fnv1a64:9f1e9d7b62605ed8 |
| 40 | P141 | fnv1a64:b4f51b69a74afd38 | fnv1a64:15f2f629e8ae8724 |
| 41 | P151 | fnv1a64:bc16b60ccc8f7506 | fnv1a64:35b90ed90e2dd50e |
| 42 | B13 | fnv1a64:0ae23c2df2b6d978 | fnv1a64:46db0929bc1d0d64 |
| 43 | R01 | fnv1a64:108df6b2e32de791 | fnv1a64:e560525cb10d2d21 |
| 44 | R02 | fnv1a64:cc00bd9b20bd984a | fnv1a64:bd6b40718b1c874a |
| 45 | R03 | fnv1a64:661639b9ee752ecd | fnv1a64:96789a881de2dd65 |
| 46 | R040 | fnv1a64:7d78c2d5e2dd2931 | fnv1a64:8c904c23cd4c22c1 |
| 47 | R050 | fnv1a64:47f4f6e079ccd06c | fnv1a64:c8e79c2fd4b435b0 |
| 48 | R060 | fnv1a64:2ef510362ea734c9 | fnv1a64:fc3ddc26cb255909 |
| 49 | R070 | fnv1a64:6fb8281eda7ce080 | fnv1a64:a3d030987752357c |
| 50 | R080 | fnv1a64:4aacdf577bfc88a1 | fnv1a64:a8ca5a5983a5b371 |
| 51 | R090 | fnv1a64:8afdbe398b3dd5f4 | fnv1a64:302c6dee9e128e28 |
| 52 | R100 | fnv1a64:a2e51fb82d2ed832 | fnv1a64:8df1e8563be0bce2 |
| 53 | R110 | fnv1a64:4c021de658b4e18f | fnv1a64:b00ede83b3ea330b |
| 54 | R041 | fnv1a64:f11b9f5809de9878 | fnv1a64:2d146c53d344cc64 |
| 55 | R051 | fnv1a64:7fe47f0ed9726ec1 | fnv1a64:f5e5661ff3929391 |
| 56 | R061 | fnv1a64:cdd75898cddf61d6 | fnv1a64:85eda9ad31dae71e |
| 57 | R071 | fnv1a64:b1a0530d99a2c99b | fnv1a64:b74cc5753943ac0f |
| 58 | R081 | fnv1a64:0334b9cafa4b4234 | fnv1a64:d6f381b445654068 |
| 59 | R091 | fnv1a64:5619e608c1efef39 | fnv1a64:e1b0ff1f77468979 |
| 60 | R101 | fnv1a64:f318568bd0338489 | fnv1a64:9f760f3d41e40ec9 |
| 61 | R111 | fnv1a64:133aad1676a7c7ee | fnv1a64:88f986cf7a3a1c66 |
| 62 | R12 | fnv1a64:b219cee4b2a5ad2e | fnv1a64:aa989930ef9e9fa6 |
| 63 | R13 | fnv1a64:8926387d8f56d1a2 | fnv1a64:80f6128a1d7c43d2 |
| 64 | R14 | fnv1a64:61a25eec3b502d34 | fnv1a64:356126d5866a2b68 |
| 65 | R15 | fnv1a64:ea36271c7034eda8 | fnv1a64:554265c6f7850ed4 |
| 66 | O010 | fnv1a64:ec77bfee3b29c536 | fnv1a64:1fef061ec6ebf2fe |
| 67 | O0200 | fnv1a64:0fadf5a44d426af9 | fnv1a64:460763f266e87839 |
| 68 | O011 | fnv1a64:717dccbe3c38f949 | fnv1a64:3ec698aed8b71d89 |
| 69 | O0210 | fnv1a64:1397d9c4b823250e | fnv1a64:c5be44bafeea4986 |



Every promoted certificate-cell change, hash=true (all other fields identical):
| Row | ID | Column (zero-based) | Before | After |
| --- | --- | --- | --- | --- |
| 2 | C01 | 4 | instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:ce6ad20653926b3e", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:db543eaa6df76620;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:1397d9c4b823250e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:717dccbe3c38f949;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} \|\| instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:ce6ad20653926b3e", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:c6cb017dd1358c2a;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:0fadf5a44d426af9;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:ec77bfee3b29c536;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} | instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:fcd2134d2e2f8106", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:549b217b3187bb25;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:c5be44bafeea4986;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:3ec698aed8b71d89;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} \|\| instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:fcd2134d2e2f8106", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:90a8716a056527ab;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:460763f266e87839;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:1fef061ec6ebf2fe;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} |
| 3 | C02 | 4 | instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:24dc27a401ea8940", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:ce01acda3c73dc98;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:5d2da3a4ead898dc;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:748dbe3bb82335d5;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:83a87664ac77bfa1;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:108df6b2e32de791;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:cc00bd9b20bd984a;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:0ae23c2df2b6d978;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:1d4c40a7bfecadd5;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} | instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:32c3f11a9998f158", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:72b0a6e70775ebb4;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:44d3ba11d71b1ce0;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:c13f7548b6b398bd;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:e1c5f166b420ea71;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:e560525cb10d2d21;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:bd6b40718b1c874a;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:46db0929bc1d0d64;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:69fdf7b4be7d10bd;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} |



Every promoted certificate-cell change, hash=false (all other fields identical):
| Row | ID | Column (zero-based) | Before | After |
| --- | --- | --- | --- | --- |
| 2 | C01 | 5 | instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:704e40a5375f1dc8", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:507d9ba1e842e023;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:1397d9c4b823250e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:717dccbe3c38f949;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} \|\| instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:704e40a5375f1dc8", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:f5a09329759e61a5;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:0fadf5a44d426af9;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:ec77bfee3b29c536;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} | instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:4d72c48707528303", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:b0dfbc9e4cc685cb;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:c5be44bafeea4986;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:3ec698aed8b71d89;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} \|\| instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:4d72c48707528303", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:7c8c4c8fbfc7db51;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:460763f266e87839;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:1fef061ec6ebf2fe;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} |
| 3 | C02 | 5 | instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:5aadcfa3dbf6071a", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:479e00b3640f75f0;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:5d2da3a4ead898dc;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:748dbe3bb82335d5;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:83a87664ac77bfa1;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:108df6b2e32de791;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:cc00bd9b20bd984a;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:0ae23c2df2b6d978;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:1d4c40a7bfecadd5;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} | instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:d39ddc7954994591", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:ec1168016699ab64;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:44d3ba11d71b1ce0;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:c13f7548b6b398bd;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:e1c5f166b420ea71;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:e560525cb10d2d21;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:bd6b40718b1c874a;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:46db0929bc1d0d64;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:69fdf7b4be7d10bd;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} |



Every effect-inventory row change (descriptor cell only):
| Row | Entry | Before descriptors | After descriptors |
| --- | --- | --- | --- |
| 100 | DeleteCases.guard | DeleteCases.guard | DeleteCases.guard,DeleteCases.guard.sharedKeep,DeleteCases.guard.sharedMixed,DeleteCases.guard.sharedAssociation,DeleteCases.guard.sharedLimit |
| 105 | DeleteCases.patternTest | DeleteCases.patternTest | DeleteCases.patternTest,DeleteCases.patternTest.sharedKeep,DeleteCases.patternTest.sharedMixed,DeleteCases.patternTest.sharedAssociation,DeleteCases.patternTest.sharedLimit |
| 336 | Set.target | Set.target | Set.target,Set.target.associationSelectors |
| 414 | TestReport.explicitOption | TestReport.explicitOption,TestReport.explicitOption.memory,TestReport.explicitOption.time | TestReport.explicitOption,TestReport.explicitOption.memory,TestReport.explicitOption.time,TestReport.explicitOption.sharedInput,TestReport.explicitOption.sharedExpectedOutput,TestReport.explicitOption.sharedSameTest |



### 3. Panics retired, noise floors and every measured label


**Panics retired: 3/3.** Base default and alloc-count binaries were built from the original production tree before edits, into independent fresh targets with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, copied, and SHA-256 hashed. Candidate 1 used the same procedure and separate fresh targets. The alloc feature is supplied as two arguments, `--features rum-cli/alloc-count`. Binary hashes:

| Build | Default SHA-256 | alloc-count SHA-256 |
| --- | --- | --- |
| base | e2787c4e32f9f19f3b17940691c143b91acdb8e43a060c891d87771c9c8d9b2b | cceee7d866203ea1731117f073a9cd6494bf996939e4f59612109a6654243992 |
| candidate-01 | 1ccc7c1ad94c0377a2a62ccf1135532f72c4ebb302f0620ba84d6d15f042c6d6 | e7d9d88da3560d131f35afe14568474994376be04d8e641b5e9741884f5f432a |



All 12 exact workload result checks passed before measurement in each arm. Workloads and expected values: [perf/labels.json](../../../.temp/brief-594/perf/labels.json); source hashes: [perf/sources-sha256.json](../../../.temp/brief-594/perf/sources-sha256.json). The three controls are byte-for-byte copies from brief-563. Allocation passes preceded instruction passes. Each process ran one label, with stdin `/dev/null`; raw stdout/stderr/perf CSV existed before parsing. Instruction commands use `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`; slopes are `(count(2N)-count(N))/N`. No timing suite ran and wall-clock differences made no decision. This run’s Cargo and test jobs did not overlap measurement.

Two base/base pairs were measured before the production edit. Per-label floor = maximum absolute percentage difference of the two pairs; threshold = max(0.05%, floor). No floor sample was discarded or repeated. Every floor row:

| Label | N | Base 1a slope | Base 1b slope | Base 2a slope | Base 2b slope | Floor % | Threshold % |
| --- | --- | --- | --- | --- | --- | --- | --- |
| delete-cases-wide-no-match | 1000 | 162035.937 | 162084.221 | 162063.909 | 162000.614 | 0.039055580 | 0.050000000 |
| delete-cases-association-no-match | 1000 | 531726.968 | 531712.967 | 531843.661 | 531583.999 | 0.048822994 | 0.050000000 |
| delete-cases-small-leaf | 20000 | 15128.47135 | 15128.70415 | 15188.3102 | 15448.16375 | 1.710878607 | 1.710878607 |
| association-set-single-incremental | 20000 | 23086.24875 | 23080.18475 | 23080.23835 | 23076.40835 | 0.026266719 | 0.050000000 |
| association-set-two-key | 20000 | 16972.4192 | 16972.5915 | 16924.3333 | 16972.29825 | 0.283408210 | 0.283408210 |
| table-count | 20000 | 9018.2066 | 9018.33625 | 9018.7127 | 9018.10865 | 0.006697741 | 0.050000000 |
| table-iterator | 20000 | 17118.57115 | 17166.2678 | 17103.3875 | 17070.89485 | 0.278625182 | 0.278625182 |
| table-small | 20000 | 14632.32965 | 14634.85905 | 14639.15105 | 14639.5343 | 0.017286379 | 0.050000000 |
| test-report-two-case | 1000 | 162270.39 | 162328.049 | 162529.077 | 162194.657 | 0.205760105 | 0.205760105 |
| dot-three-no-input | 20000 | 254199.24975 | 254199.8054 | 254200.45275 | 254199.90525 | 0.000218588 | 0.050000000 |
| differences-no-input | 20000 | 10030.5955 | 10030.83345 | 10044.93965 | 10051.9742 | 0.070030784 | 0.070030784 |
| take-multispec-small | 20000 | 49922.55005 | 49938.99685 | 49923.04255 | 49922.92985 | 0.032944631 | 0.050000000 |



Every allocation row (counts/bytes are per-iteration slopes; peaks are process totals):

| Label | Base count | New count | Δ count | Base bytes | New bytes | Δ bytes | Base peak N / 2N | New peak N / 2N |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| delete-cases-wide-no-match | 15.001 | 15.001 | 0.0 | 18502.864 | 18502.864 | 0.0 | 4757672 / 9205096 | 4757672 / 9205096 |
| delete-cases-association-no-match | 406.016 | 406.016 | 0.0 | 59600.064 | 59600.064 | 0.0 | 8541796 / 8573796 | 8541796 / 8573796 |
| delete-cases-small-leaf | 9.0 | 9.0 | 0.0 | 1016.0 | 1016.0 | 0.0 | 9591210 / 10231210 | 9591210 / 10231210 |
| association-set-single-incremental | 22.0001 | 22.0001 | 0.0 | 1261.00165 | 1261.00165 | 0.0 | 8689307 / 9329307 | 8689307 / 9329307 |
| association-set-two-key | 37.0 | 37.0 | 0.0 | 4010.0 | 4010.0 | 0.0 | 945049 / 1585049 | 945049 / 1585049 |
| table-count | 11.0 | 11.0 | 0.0 | 658.0 | 658.0 | 0.0 | 11425733 / 12065733 | 11425733 / 12065733 |
| table-iterator | 30.0 | 30.0 | 0.0 | 977.0 | 977.0 | 0.0 | 8827196 / 9467196 | 8827196 / 9467196 |
| table-small | 22.0 | 22.0 | 0.0 | 977.0 | 977.0 | 0.0 | 9132862 / 9772862 | 9132862 / 9772862 |
| test-report-two-case | 394.521 | 396.917 | 2.395999999999958 | 27372.2 | 27420.12 | 47.919999999998254 | 4405780 / 8506097 | 4405780 / 8506097 |
| dot-three-no-input | 596.0 | 596.0 | 0.0 | 23504.0 | 23504.0 | 0.0 | 8678236 / 9318236 | 8678236 / 9318236 |
| differences-no-input | 13.0 | 13.0 | 0.0 | 704.0 | 704.0 | 0.0 | 9591399 / 10231399 | 9591399 / 10231399 |
| take-multispec-small | 109.0 | 109.0 | 0.0 | 5856.0 | 5856.0 | 0.0 | 9594023 / 10234023 | 9594023 / 10234023 |



Allocation observation: **11/12 labels are exactly unchanged; ordinary `test-report-two-case` records +2.396 allocations and +47.92 bytes per iteration** (394.521 → 396.917 allocations; 27372.2 → 27420.12 bytes). Raw N/2N counts are base 398177/792698 and candidate 397719/794636; bytes are base 27903282/55275482 and candidate 27894130/55314250. Both arms have identical peak totals (4405780/8506097 bytes). At N the candidate allocates less; at 2N it allocates more. No allocation sample was discarded or repeated.

Read-only source inspection: ordinary non-suspending TestReport keeps `rendered_results: Vec<Expr>` through `resume_test_report_from_index`, `resume_test_report_after_output_match` and `finish_test_report`; its changed accumulator completion path is used only after suspension. Unchanged `finish_eval_expr_with_stream_capture` and `finish_test_case_input_outcome` construct elapsed-duration values via `elapsed_duration_expr`, which converts `Instant` durations to BigInt rationals; `Rational::new` reduces them with a data-dependent GCD. This provides a plausible source of allocation variability, **not a measured causal attribution**. The increase remains disclosed evidence; no allocation-flat claim is made and no out-of-scope timing code was changed. Sources: [crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs](../../../crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs), [crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs](../../../crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs), [crates/rum-runtime/src/runtime/debug_tools/shared.rs](../../../crates/rum-runtime/src/runtime/debug_tools/shared.rs), [crates/rum-core/src/expr.rs](../../../crates/rum-core/src/expr.rs).

Every paired instruction row. Each pair used a new contemporaneous base sample; a label regresses only when **both** candidate deltas exceed its recorded threshold:

| Label | Base 1 | New 1 | Δ1 % | Base 2 | New 2 | Δ2 % | Threshold % | Gate |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| delete-cases-wide-no-match | 162115.732 | 162034.523 | -0.050093226 | 161966.838 | 162090.169 | 0.076145834 | 0.050000000 | PASS |
| delete-cases-association-no-match | 531563.579 | 531614.668 | 0.009611080 | 531586.664 | 531690.486 | 0.019530588 | 0.050000000 | PASS |
| delete-cases-small-leaf | 15127.74365 | 15135.83265 | 0.053471292 | 15108.7497 | 15174.3138 | 0.433947887 | 1.710878607 | PASS |
| association-set-single-incremental | 23077.7615 | 23064.49475 | -0.057487161 | 23077.69835 | 23065.712 | -0.051939105 | 0.050000000 | PASS |
| association-set-two-key | 16972.2293 | 16972.3831 | 0.000906186 | 17048.4659 | 16972.4477 | -0.445894666 | 0.283408210 | PASS |
| table-count | 9018.3467 | 9017.11825 | -0.013621676 | 9018.39405 | 9016.765 | -0.018063637 | 0.050000000 | PASS |
| table-iterator | 17141.5057 | 17115.3031 | -0.152860551 | 17114.336 | 17114.80005 | 0.002711469 | 0.278625182 | PASS |
| table-small | 14632.7328 | 14634.8474 | 0.014451163 | 14639.4922 | 14634.6442 | -0.033115903 | 0.050000000 | PASS |
| test-report-two-case | 163321.013 | 162163.517 | -0.708724480 | 162514.668 | 162163.477 | -0.216098032 | 0.205760105 | PASS |
| dot-three-no-input | 254199.1318 | 254198.37325 | -0.000298408 | 254199.15085 | 254198.36755 | -0.000308144 | 0.050000000 | PASS |
| differences-no-input | 10037.85335 | 10033.36405 | -0.044723706 | 10052.001 | 10033.84475 | -0.180623241 | 0.070030784 | PASS |
| take-multispec-small | 49938.80735 | 49922.4436 | -0.032767603 | 49922.7105 | 49922.4787 | -0.000464318 | 0.050000000 | PASS |



Full precision and all measured columns: [candidate-01/metrics.tsv](../../../.temp/brief-594/candidate-01/metrics.tsv); raw process files: [perf/alloc/](../../../.temp/brief-594/perf/alloc), [perf/instructions/](../../../.temp/brief-594/perf/instructions). Verbatim reducer output:
```text
{
  "measured_candidates": 1,
  "labels": 12,
  "regressing_labels": 0,
  "worst_regression_pct": 0,
  "worst_repeatable_delta_pct": 0.05347129213153501,
  "positive_allocation_count_deltas": 1,
  "positive_allocation_byte_deltas": 1
}
delete-cases-wide-no-match alloc_delta=0.000000 bytes_delta=0.000000 pair1=-0.050093226% pair2=0.076145834% threshold=0.050000000% PASS
delete-cases-association-no-match alloc_delta=0.000000 bytes_delta=0.000000 pair1=0.009611080% pair2=0.019530588% threshold=0.050000000% PASS
delete-cases-small-leaf alloc_delta=0.000000 bytes_delta=0.000000 pair1=0.053471292% pair2=0.433947887% threshold=1.710878607% PASS
association-set-single-incremental alloc_delta=0.000000 bytes_delta=0.000000 pair1=-0.057487161% pair2=-0.051939105% threshold=0.050000000% PASS
association-set-two-key alloc_delta=0.000000 bytes_delta=0.000000 pair1=0.000906186% pair2=-0.445894666% threshold=0.283408210% PASS
table-count alloc_delta=0.000000 bytes_delta=0.000000 pair1=-0.013621676% pair2=-0.018063637% threshold=0.050000000% PASS
table-iterator alloc_delta=0.000000 bytes_delta=0.000000 pair1=-0.152860551% pair2=0.002711469% threshold=0.278625182% PASS
table-small alloc_delta=0.000000 bytes_delta=0.000000 pair1=0.014451163% pair2=-0.033115903% threshold=0.050000000% PASS
test-report-two-case alloc_delta=2.396000 bytes_delta=47.920000 pair1=-0.708724480% pair2=-0.216098032% threshold=0.205760105% PASS
dot-three-no-input alloc_delta=0.000000 bytes_delta=0.000000 pair1=-0.000298408% pair2=-0.000308144% threshold=0.050000000% PASS
differences-no-input alloc_delta=0.000000 bytes_delta=0.000000 pair1=-0.044723706% pair2=-0.180623241% threshold=0.070030784% PASS
take-multispec-small alloc_delta=0.000000 bytes_delta=0.000000 pair1=-0.032767603% pair2=-0.000464318% threshold=0.050000000% PASS
```


### 4. Owed entries, stop-loss counters and open questions


The four G11 rows remain owed, byte-for-byte: `EvaluationData.body`, `TestReport.input`, `TestReport.expectedOutput`, `TestReport.sameTest`. The owed ledger remains 114 rows, including four debug entries. The new report fixtures are supplemental to the already drained `TestReport.explicitOption` row; they do not discharge G11 or claim to repair capture/output visibility.

One production candidate was compiled and measured (1/8); its progress is 3/3 panics retired, 68/68 supplemental variants passing, 4/4 consumers migrated, 0/12 regressing instruction labels, worst regression 0.000000000%. No-progress streak: 0; the two-candidate stop-loss did not fire. The only premeasurement correction was test diagnostic routing from stdout to stderr.

Adjacent findings: the lexical stdout hygiene issue is fixed and its red/green evidence is retained. The ordinary TestReport allocation increase above has no proven causal attribution; the prescribed instruction gate passes 12/12, while the allocation panel is not claimed flat. Brief-593’s separate capture/output-prefix issues remain outside this step and were not reprobed. All mandated gates ran; existing ignored tests and the native transport skips above are not claimed as executed coverage. Scope evidence: [scope-audit.log](../../../.temp/brief-594/scope-audit.log); review patch: [candidate-01/changes.patch](../../../.temp/brief-594/candidate-01/changes.patch). No work was committed; the director owns the gate and version-control actions.
