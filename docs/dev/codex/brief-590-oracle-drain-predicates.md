# brief-590 — E1 drain 4 of 8: `predicates`, `structure`, `structure_ordering`, and six carried entries (78 owed entries)

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) and left **500** declared
guest-effect entries in an exact owed ledger (`inventory/owed.tsv`). Drains 1-3
(briefs 587-589) took `rules`, `rewrite`, `control`, `logic`, `arithmetic` and
`random`; eleven declared roles that do not exist were removed at the
director's gate (owed 500 -> 319). **This one drains 78 entries**: exactly the
IDs listed in
`.temp/brief-586/followups/draft-E1-{10-predicates,19-structure,20-structure_ordering}.md`
— the predicates, comparators, key functions, `SameTest` callbacks and option
roles of the containment, ordering and set builtins, `Depth`'s options,
`Differences`' arithmetic and the lazy consumers of `First` and `Take` — **plus
six carried entries** whose policy the director corrected to `Tail`:
`Part.lazyConsumer`, `If.branch`, `Which.branch`, `Switch.branch`,
`Catch.handler`, `Enclose.handler`.

Each entry leaves the ledger **only in the same change that attaches and runs
its complete descriptor**: every required variant (`Middle`, `Twice`, and the
`Decline`, `Error`, `AlreadyResumed`, `Nested` variants its effect row
records), lifecycle at the first and second pause for every distinct
continuation owner (completion, cancellation, reset, a pending request
rejecting a new one), and a fresh request after.

Those drafts are the raw material; **this brief governs** where they differ.

## A failing descriptor is a finding, not a test to fix

If a correct descriptor shows the resumed path differing from the queued one —
a lost continuation the census did not see — **do not weaken the descriptor,
do not repair production code, and do not commit a failing test.** Keep the
entry owed, write the witness (source, answers, both observations) to
`.temp/brief-590/defects/<entry>.md`, and report it. It becomes a repair group
of this branch. **A role that does not exist** (an `Ordinary` builtin's option or operand
evaluated by shared preparation, a positional argument declared as an option,
option syntax admitted only literally) is reported with source evidence and
kept owed; the director removes it at the gate — briefs 587-589 found eleven.
Check the registration (`Ordinary` or held) and the argument partition first.
**A wrong policy is not a reason to keep an entry owed**: the oracle's equality
does not depend on whether the entry is `Frame` or `Tail`, so attach the
descriptor, and report the correct policy for the director to set (brief-589
held five tails owed for this; that rule is withdrawn).

## Context pointers

- briefs 586-589 and their reports; the three drafts above; the descriptors in
  `fixtures/rules/`, `fixtures/rewrite_roles/` and `fixtures/control_roles/` as
  the model.
- `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors,catalog}.rs`,
  `support/`, `fixtures/`, `inventory/{catalog,effects,shared-effects,owed,historical-labels}.tsv`.
- The owning functions named in the drafts; the guide and the generated
  reference are the contract.
- The existing `lazy_pull_continuations.rs` (`First`, `Take`, `Part` lazy
  consumers) and `loop_continuations.rs` (`Differences`) cases, and the
  historical labels, reusable as descriptors. A `SameTest` or comparator
  callback must pause inside the builtin's own comparison loop, after earlier
  comparisons have run.

## Scope

**Edit surface:** the `suspension_oracle` test tree; the existing
continuation suites only to share descriptors (test names and specialized
assertions stay).

**Standing duty — always in scope** (`TEMPLATE.md`; test edits under `crates/`
expire the fingerprints): fingerprints, both certificates, ownership
reconciliation, every row disclosed. **STOP** on any existing row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code.

**Fence:** no production `.rs`, manifest disposition, `lib.rs`, Cargo
manifest, lockfile or dependency change; no other family's entries; no edits
to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs, `scripts/` or
docs.

**Disclosure duty:** every entry drained, every entry kept owed and why, every
defect witness, every changed test name, every inventory row.

## Deliverable

Role-specific descriptor factories with explicit expected values, logs and
state projections and exact input kind, prompt and answer plans. A callback,
guard, option or held-target phase is exercised by its own role — ordinary
argument preparation never stands in. Loops run at least five operations and
pause in the middle and at a second position. Shared factories across the
family are welcome where they demonstrably exercise each entry's role.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix or resumed-only value the expected answer; expected
   values come from the documented contract and the queued path.
2. An eager-only or single-pause fixture proves nothing.
3. Support modules under 10k characters, scenario modules under 15k.
4. Stop-loss counters: entries drained, variants passing, lifecycle
   obligations passing, defects recorded. Stop after two consecutive compiling
   attempts that move none; fewer compiler errors is progress.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-590/tmp …`. Bounds: the oracle target 900 s per
build; runtime, ownership and retention 300 s; `cargo test --workspace
--no-fail-fast` 3 000 s.

```
cargo fmt --check
scripts/check-suspension-oracle.sh 900
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

The oracle's `CATALOG` lines must show `owed` reduced by exactly the drained
count in both builds. Report native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Everything under `.temp/brief-590/`; `PROGRESS.md` after each milestone with
the counters and the exact next action.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, `scripts/check-suspension-oracle.sh`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only. No
  new `unsafe`.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files changed; the no-production-edit audit.
2. Entries drained and kept owed (with reasons); owed before and after, per
   build; variants and lifecycle obligations added.
3. Defect witnesses, if any; the broad suites verbatim; every inventory row
   change.
4. Stop-loss counters, open questions.

---

## Execution report — brief-590

**66 of the 78 target entries drained; twelve nonexistent local roles remain owed.** Both oracle builds, runtime, ownership and both retention builds passed. The full workspace command exited 101 solely because nine native ZeroMQ tests could not bind loopback; a supported Jupyter rerun recorded those nine as skips and passed its remaining tests. No resumed/queued defect was found. All changes are left uncommitted; no git command, external network access, production repair, or external-engine comparison was performed.

### 1. Files changed and edit-scope audit

Changes are relative to the saved start-of-brief hashes, preserving pre-existing worktree content. The executing agent did not inspect git state.

| File | Change |
|---|---|
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | Replay all 68 generated fingerprint cells. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | Replay hash=true/hash=false certificate cells for C01 and C02. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs) | Register the new fixture family. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/carried.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/carried.rs) | Five corrected Tail branch/handler roles, with the selected result and handler arguments checked. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/collections.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/collections.rs) | SameTest loops and separate set membership/deduplication phases, including non-True declines. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/falsifiers.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/falsifiers.rs) | Missing-pause and lost-log rejection, weak controls, and good control for every new descriptor. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/keys.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/keys.rs) | Key functions, stable ties, hierarchical GatherBy, and SortBy's custom-ordering key phase. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/lifecycle.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/lifecycle.rs) | First/second pause cancellation, reset, overlapping-request rejection, state checks and fresh requests. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/mod.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/mod.rs) | 66-entry registry and role-specific factory dispatch. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/options.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/options.rs) | Actual delayed default/explicit SameTest resolution and Depth's stored Heads default. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/ordering.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/ordering.rs) | Comparator loops and scalar/list forward/reverse lexicographic continuation owners. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/phases.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/phases.rs) | Ten additional descriptors for distinct continuation phases. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/predicates.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/predicates.rs) | Truth aggregates, array leaf tests, rank-pattern guards and SelectPositions. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/structure.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/structure.rs) | Five lazy pulls for First/Take/Part and first/second-order Differences arithmetic. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/support.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/predicate_roles/support.rs) | Explicit values/logs/state, exact typed prompts/answers, error, caller-resumed and nested plans. |
| [crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs](../../../crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs) | Attach the new descriptors to the executable oracle registry. |
| [crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv](../../../crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv) | 66 descriptor cells only; policy, identity, required variants and other cells unchanged. |
| [crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv](../../../crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv) | Remove exactly those 66 complete, executed entries. |
| This brief | Append this execution report only. |

No production `.rs`, `lib.rs`, Cargo manifest, lockfile, dependency, script, other document, root progress, roadmap, or agent-guide edit. No baseline file disappeared. `manifest.rs`, catalog dispositions, policy cells, shared effects and historical labels are unchanged. All new modules are below 4k characters; every changed Rust file is below 10k. No unsafe was added. No hot path was changed, so no performance A/B is required.

One new persistent test: `oracle::fixtures::predicate_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log`. All twelve existing oracle test names and all 209 historical labels remain. Existing continuation suites were not edited. A temporary candidate diagnostic test was removed before final verification and retained as scratch evidence in [candidate-probe.rs](../../../.temp/brief-590/candidate-probe.rs). [Exact name audit](../../../.temp/brief-590/test-name-audit.txt).

### 2. Entries, variants and lifecycle coverage

| Family | Drained entries | New descriptors | New variants per build | New lifecycle obligations per build |
|---|---:|---:|---:|---:|
| predicates | 36 | 41 | 220 | 246 |
| control | 5 | 5 | 25 | 30 |
| structure_ordering | 20 | 24 | 128 | 144 |
| structure | 4 | 5 | 25 | 30 |
| rules | 1 | 1 | 5 | 6 |
| **Total** | **66** | **76** | **403** | **456** |

The first candidate log put one PASS on the test-name line; the initial anchored count of 402 omitted it. Reconciled count: **403** per build, **806** across both final builds. Final totals are 1,355 variant passes and 1,554 lifecycle obligations per build. Completion, drained answer queues and fresh requests are checked for every variant; lifecycle adds cancellation, reset and pending-request rejection at both pauses for each descriptor.

Inputs are `Input` with exact `<entry>:2` / `<entry>:4` prompts and answers `2` / `4`. `AlreadyResumed` first resumes an `InputString` caller. `Nested` executes under a scoped Block and verifies restoration. Error cases assert the documented uncaught tagged throw; required Decline cases supply false/non-True predicate results. Logs, prints, progress state and the sum of consumed answers are explicit expectations, with retained prefixes checked on cancellation and reset.

Callback loops perform at least five operations and suspend after earlier visits. Additional lexicographic scalar descriptors cover its intrinsically non-looping pair owners with five guest operations, alongside the five-pair forward/reverse loop cases. The list cross-phase descriptor pauses after two completed comparisons and again in the reverse comparison entered from resume. Sort/ReverseSort use a strict comparator over unsorted input; the first five callbacks have explicit logs, and the complete comparison-pair transcript is compared between queued and resumed runs. No comparison schedule is promoted to a contract or normalized away (guide/evaluation.md, Ordering Callbacks).

Every ID below changes only the descriptor cell in `effects.tsv` and loses its exact original owed row:

```text
AllTrue.predicate
AnyTrue.predicate
ArrayQ.predicate
ArrayQ.rankPattern
Catch.handler
Complement.defaultOption
Complement.explicitOption
Complement.sameTest
ContainsAll.defaultOption
ContainsAll.explicitOption
ContainsAll.sameTest
ContainsAny.defaultOption
ContainsAny.explicitOption
ContainsAny.sameTest
ContainsExactly.defaultOption
ContainsExactly.explicitOption
ContainsExactly.sameTest
ContainsNone.defaultOption
ContainsNone.explicitOption
ContainsNone.sameTest
ContainsOnly.defaultOption
ContainsOnly.explicitOption
ContainsOnly.sameTest
DeleteDuplicates.sameTest
DeleteDuplicatesBy.keyFunction
Depth.defaultOption
Differences.arithmetic
DisjointQ.defaultOption
DisjointQ.explicitOption
DisjointQ.sameTest
DuplicateFreeQ.defaultOption
DuplicateFreeQ.explicitOption
DuplicateFreeQ.sameTest
Enclose.handler
First.lazyConsumer
GatherBy.keyFunction
If.branch
IntersectingQ.defaultOption
IntersectingQ.explicitOption
IntersectingQ.sameTest
Intersection.sameTest
LexicographicOrder.comparator
MatrixQ.predicate
MaxBy.keyFunction
MaximalBy.keyFunction
MinBy.keyFunction
MinimalBy.keyFunction
NoneTrue.predicate
OrderedQ.comparator
Part.lazyConsumer
ReverseSort.comparator
ReverseSortBy.keyFunction
SelectPositions.predicate
Sort.comparator
SortBy.keyFunction
SubsetQ.defaultOption
SubsetQ.explicitOption
SubsetQ.sameTest
Switch.branch
Take.lazyConsumer
TakeLargestBy.keyFunction
TakeSmallestBy.keyFunction
Tally.sameTest
Union.sameTest
VectorQ.predicate
Which.branch
```

[Every full before/after effect row](../../../.temp/brief-590/effect-row-changes.tsv); [every complete removed owed row](../../../.temp/brief-590/owed-rows-removed.tsv).

Additional descriptor IDs, attached to the same entries and executing every required variant and both-pause lifecycle:

```text
Complement.sameTest/deduplicate
Differences.arithmetic/order-two
GatherBy.keyFunction/function-list
Intersection.sameTest/deduplicate
LexicographicOrder.comparator/reverse
LexicographicOrder.comparator/pair-forward
LexicographicOrder.comparator/pair-reverse
LexicographicOrder.comparator/cross
LexicographicOrder.comparator/pair-cross
SortBy.keyFunction/ordering
```

Kept owed, without changing any declaration or disposition:

| Entry | Reason |
|---|---|
| `ContainsAll.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `ContainsAny.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `ContainsExactly.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `ContainsNone.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `ContainsOnly.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `DisjointQ.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `DuplicateFreeQ.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `IntersectingQ.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `SubsetQ.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |
| `Depth.explicitOption` | Depth reads its stored default, then structurally classifies the prepared second argument. It does not evaluate explicit delayed values or computed option syntax locally. |
| `Depth.optionSyntax` | Depth reads its stored default, then structurally classifies the prepared second argument. It does not evaluate explicit delayed values or computed option syntax locally. |
| `Complement.optionSyntax` | Unheld ordinary preparation owns computed option syntax; local partition only accepts already prepared literal SameTest rules/lists. |

[Per-entry registration and argument-partition evidence, with numbered source excerpts](../../../.temp/brief-590/roles-kept-owed.md). These are declaration findings, not failing runtime descriptors. They remain for director disposition. No wrong-policy finding is outstanding; the six carried Tail entries were all attached and executed.

Before, both builds:

```text
LIFECYCLE obligations=1098 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=238 NoLocalGuestEffect=405 Boundary=4 entries=495 descriptors=187 historical=209 owed=319
LIFECYCLE obligations=1098 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=238 NoLocalGuestEffect=399 Boundary=4 entries=495 descriptors=187 historical=209 owed=319
```

After, both builds:

```text
LIFECYCLE obligations=1554 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=238 NoLocalGuestEffect=405 Boundary=4 entries=495 descriptors=263 historical=209 owed=253
LIFECYCLE obligations=1554 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=238 NoLocalGuestEffect=399 Boundary=4 entries=495 descriptors=263 historical=209 owed=253
```

Owed is **319 → 253**, exactly **66** fewer in each build. The target families retain predicates=9, structure=2, structure_ordering=1. All 241 non-target owed entries remain unchanged.

### 3. Verification, defect witnesses and inventory disclosure

**Defect witnesses: none.** No production fix, weakened expectation, omitted required variant, or failing descriptor remains. Each descriptor's missing-pause and lost-log falsifiers rejected the mutation, their weakened controls passed, and the good descriptor passed: 76 pairs per build (152 strict rejections per build). Full evidence: [role-falsifiers-1.txt](../../../.temp/brief-590/role-falsifiers-1.txt) and [oracle-final.txt](../../../.temp/brief-590/oracle-final.txt).

Representative red/green lines (the logs contain all 76 IDs per build):

```text
PREDICATE_FALSIFIERS ContainsExactly.sameTest missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS
PREDICATE_FALSIFIERS Sort.comparator missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS
PREDICATE_FALSIFIERS Part.lazyConsumer missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS
PREDICATE_FALSIFIERS LexicographicOrder.comparator/cross missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS
```

Commands ran serially through `python3 scripts/run-bounded.py`, with `CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1`, `CARGO_NET_OFFLINE=true`, and the brief-local TMPDIR. Bounds were 900 seconds per oracle build, 300 for runtime/ownership/retention and formatting, and 3000 for workspace. Full command arrays and exit codes accompany logs. The mandated oracle script internally uses `.temp/suspension-oracle/tmp` and writes its fixed gate logs; the generator uses `target/brief-544`. Their evidence is copied under this brief. The certificates use the permitted `.temp/brief-546/certificates/` path. No script was changed.

`cargo fmt --check`: final exit 0, empty output. Initial formatting differences were confined to the edited test modules and were formatted. [fmt-final.txt](../../../.temp/brief-590/fmt-final.txt).

`scripts/check-suspension-oracle.sh 900` (both builds):

```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.07s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.12s
```

`cargo test -p rum-runtime`:

```text
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.75s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.15s
```

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`:

```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 60.12s
```

All seven ownership inventories are byte-identical. No existing row changed or disappeared; no new UnknownHostPayload, self-reaching or untouched-source row appeared. [ownership-reconciliation.tsv](../../../.temp/brief-590/ownership-reconciliation.tsv).

`cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`:

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.21s
```

All 68 rows below change only their final generated fingerprint cell. Identity, contract text, status, strength and exclusion cells stay fixed:

```text
B01
B02
B03
B04
B05
B06
B07
B08
B09
B10
B11
B12
I01
I02
I03
I04
P01
P02
P03
P04
P05
P06
P070
P080
P090
P100
P110
P120
P130
P140
P150
P071
P081
P091
P101
P111
P121
P131
P141
P151
B13
R01
R02
R03
R040
R050
R060
R070
R080
R090
R100
R110
R041
R051
R061
R071
R081
R091
R101
R111
R12
R13
R14
R15
O010
O0200
O011
O0210
```

[Every fingerprint before/after value](../../../.temp/brief-590/fingerprint-changes.tsv).

The initial certificate replay in each build found its expected stale certificate and emitted the freshly earned cells. These were copied, then both acceptances passed. Initial replay failure evidence:

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 17.91s
```

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.79s
```

Only the hash=true/hash=false cells in promoted rows C01 and C02 changed. Differences consist solely of embedded direct-contract fingerprints; instance identity, build, body/domain fingerprints, instance marks, source positions, preconditions, strength and exclusions are unchanged.

```text
body	feature	changed direct-contract fingerprints	instance marks
C01	true	O010,O0200,O011,O0210	fnv1a64:db543eaa6df76620,fnv1a64:c6cb017dd1358c2a
C01	false	O010,O0200,O011,O0210	fnv1a64:507d9ba1e842e023,fnv1a64:f5a09329759e61a5
C02	true	B05,I01,I02,I04,B13,R01,R02	fnv1a64:ce01acda3c73dc98
C02	false	B05,I01,I02,I04,B13,R01,R02	fnv1a64:479e00b3640f75f0
```

[All four full before/after certificate cells](../../../.temp/brief-590/certificate-changes.tsv). All other retention inventories are unchanged.

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.02s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.92s
```

The initial workspace invocation failed only the nine native ZeroMQ tests because loopback binding is denied (EPERM). The guide says they skip automatically, but this checkout requires the supported `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` opt-in (`crates/rum-jupyter/src/zeromq_native_test_support.rs:335`). Its PermissionDenied branch alone allows the skip; other errors still fail. Initial full-run evidence: [workspace.txt](../../../.temp/brief-590/workspace.txt).

```text
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
error: 1 target failed:
```

`cargo test --workspace --no-fail-fast -- --nocapture` (verbatim target/result lines; the only failing target is the environment-blocked Jupyter library):

```text
Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.21s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.86s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1042.55s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.56s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.13s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.14s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.12s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.48s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The complete workspace was not repeated. Only the affected library was rerun on the same source with the documented permission-denied skip opt-in:

`env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter --lib -- --nocapture`:

```text
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
```

The Jupyter harness reports 151 passed, comprising 142 executed tests and **nine native skips**. No native transport success is claimed.

Native ZeroMQ outcomes are **skips**, not successful transport executions:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

Full workspace log: [workspace.txt](../../../.temp/brief-590/workspace.txt); affected-library rerun: [jupyter-skip.txt](../../../.temp/brief-590/jupyter-skip.txt). Other complete logs: [runtime.txt](../../../.temp/brief-590/runtime.txt), [ownership.txt](../../../.temp/brief-590/ownership.txt), [fingerprints.txt](../../../.temp/brief-590/fingerprints.txt), [retention-default.txt](../../../.temp/brief-590/retention-default.txt), [retention-no-default.txt](../../../.temp/brief-590/retention-no-default.txt), [maintenance.txt](../../../.temp/brief-590/maintenance.txt). No mandatory command was omitted. The full workspace invocation retains its documented environment-only exit 101; the affected-library rerun is green with nine explicit skips. Expected stale-certificate replay failures were resolved exclusively with generated cells.

### 4. Stop-loss counters and remaining decisions

- Entries drained: **66/78**; kept owed: **12**, each with source evidence.
- Descriptor variants: **403/403 per build**; **806/806** across both.
- Added lifecycle obligations: **456/456 per build**, plus completion for all variants.
- Falsifiers: **76/76 descriptor pairs per build**, with all strict rejections and weak/good controls passing.
- Defects recorded: **0**. Existing test names changed: **0**.
- Stop-loss did not trigger: the first compiling implementation attempt passed all candidate variants/lifecycle checks; the next strengthened visit-overflow/cross-phase checks and passed every falsifier, followed by the 66 exact ledger attachments. The remaining builds were required verification and generated maintenance replay; the single affected-library rerun addressed the native loopback environment failure.

The director's remaining decision is disposition of the twelve nonexistent-role declarations. This brief leaves them owed as instructed; no runtime repair group is needed. [Checkpoint](../../../.temp/brief-590/PROGRESS.md); [final no-production-edit and inventory audit](../../../.temp/brief-590/audit.txt).
