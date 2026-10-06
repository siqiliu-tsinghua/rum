# brief-591 — E1 drain 5 of 8: `functional` and `lazy` (55 owed entries)

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) and left **500** declared
guest-effect entries in an exact owed ledger (`inventory/owed.tsv`). Drains 1-4
(briefs 587-590) took `rules`, `rewrite`, `control`, `logic`, `arithmetic`,
`random`, `predicates`, `structure` and `structure_ordering`; twenty-three
declared roles that do not exist were removed at the director's gate (owed 500
-> 241). **This one drains 55 entries**: exactly the IDs listed in
`.temp/brief-586/followups/draft-E1-{07-functional,08-lazy}.md` — the callbacks,
predicates, combiners and arithmetic of the higher-order builtins (`Map`,
`MapIndexed`, `MapThread`, `MapAt`, `MapAll`, `Fold`, `FoldList`, `FoldPair`,
`FoldWhile`, `Nest`, `NestList`, `NestWhile`, `NestWhileList`, `FixedPoint`,
`FixedPointList`, `Select`, `SelectFirst`, `TakeWhile`, `LengthWhile`,
`Array`, `ArrayFilter`, `ArrayReduce`, `BlockMap`, `Outer`, `Inner`, `Dot`,
`Accumulate`, `Through`, `Operate`, `Comap`, `ComapApply`, `SequenceFold`,
`SequenceFoldList`, `SubsetMap`) and the sources, callbacks, predicates, body
and bounds of the lazy constructors (`LazyMap`, `LazySelect`, `LazyFoldList`,
`LazyNestList`, `LazyTable`, `LazyDrop`, `LazyTake`).

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
`.temp/brief-591/defects/<entry>.md`, and report it. It becomes a repair group
of this branch. **A role that does not exist** (an `Ordinary` builtin's option or operand
evaluated by shared preparation, a positional argument declared as an option,
option syntax admitted only literally) is reported with source evidence and
kept owed; the director removes it at the gate — briefs 587-590 found
twenty-three, most of them `optionSyntax` on `Ordinary` builtins, where a
computed option expression is evaluated by shared preparation before the
handler sees it (a delayed rule's rhs *is* read locally: that is
`explicitOption`).
Check the registration (`Ordinary` or held) and the argument partition first.
**A wrong policy is not a reason to keep an entry owed**: the oracle's equality
does not depend on whether the entry is `Frame` or `Tail`, so attach the
descriptor, and report the correct policy for the director to set.

## Context pointers

- briefs 586-590 and their reports; the two drafts above; the descriptors in
  `fixtures/rules/`, `fixtures/rewrite_roles/`, `fixtures/control_roles/` and
  `fixtures/predicate_roles/` as the model.
- `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors,catalog}.rs`,
  `support/`, `fixtures/`, `inventory/{catalog,effects,shared-effects,owed,historical-labels}.tsv`.
- The owning functions named in the drafts; the guide (`evaluation.md`
  §Lazy Sequences) and the generated reference are the contract.
- The existing `lazy_pull_continuations.rs` and `loop_continuations.rs`
  (`Dot`) cases and the historical labels, reusable as descriptors. A lazy
  constructor's callback runs when a consumer pulls: the descriptor pauses
  inside the pull at a middle element and a second element, and a source role
  pauses while the source is produced. A callback must pause inside the
  builtin's own loop after earlier iterations have run.

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
TMPDIR=<repo>/.temp/brief-591/tmp …`. Bounds: the oracle target 900 s per
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

Everything under `.temp/brief-591/`; `PROGRESS.md` after each milestone with
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

## Execution report — brief-591

### 1. Files changed and no-production-edit audit

Implemented 51 complete role entries using 65 descriptors. All work is uncommitted; no version-control command or external network lookup was run. Production Rust is unchanged. The saved initial file hashes, final source audit and inventory reconciliation are in [audit.txt](../../../.temp/brief-591/audit.txt), [changed-files.json](../../../.temp/brief-591/changed-files.json) and [all-inventory-audit.tsv](../../../.temp/brief-591/all-inventory-audit.tsv).

New files under `crates/rum-stdlib/tests/suspension_oracle/fixtures/functional_roles/`:

| File | Purpose |
| --- | --- |
| `mod.rs` | Exact 51-entry registry and dispatch; tensor and bound-slot phase attachments. |
| `support.rs` | Exact Input kinds, prompts and answers; expected values, logs, prints, state, scoped restoration, errors and variant construction. |
| `lifecycle.rs` | Both pauses: cancellation, reset, rejected overlap, observed prefix/state and fresh requests. |
| `mapping.rs` | Map, array/window, Comap, Through, Operate and SubsetMap callbacks. |
| `folds.rs` | Folds, iteration callbacks and predicates, selection and while predicates. |
| `arithmetic.rs` | Accumulate, Inner/Dot terms and reductions, and ArrayFilter option effects. |
| `lazy.rs` | Deferred callback/body/predicate effects, child-source production, and each LazyTable bound slot. |
| `phases.rs` | Cross-phase resume, FoldWhile positive tail, SubsetMap reconstruction and retained contraction prefixes. |
| `falsifiers.rs` | Every descriptor rejects a missing pause and lost log, with weak and good controls. |

Existing files changed: oracle `fixtures/mod.rs` and `fixtures/primary.rs` register the new module; `inventory/effects.tsv` attaches the exact descriptor IDs; `inventory/owed.tsv` removes those same 51 entries. Standing maintenance changes only `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` and `promoted.tsv`. This report is appended to the dispatched brief. All new support modules are below 10k characters and all scenario modules below 15k.

No production `.rs`, `lib.rs`, Cargo manifest, lockfile, dependency, script, manifest disposition, other-family entry or unrelated document changed. The historical fixtures and all existing continuation suites are untouched. No performance gate applies to this test-only change.

Test-name audit: [test-name-audit.txt](../../../.temp/brief-591/test-name-audit.txt). All 13 existing oracle names are preserved; zero existing tests were renamed or removed. One permanent test was added:

```text
oracle::fixtures::functional_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log
```

The temporary `oracle::fixtures::functional_roles::candidate::candidates` diagnostic test passed and was removed before the final gate; its source is archived in [candidate-probe.rs](../../../.temp/brief-591/candidate-probe.rs). No failing test is left in the tree.

### 2. Entries drained, entries kept owed, and obligations


Drained **51/55** requested entries: **40 functional** and **11 lazy**. Both builds go from **241 owed to 190 owed**. Added **65 descriptors**, **333 variants** and **390 lifecycle obligations per build**, plus completion and a fresh request for every variant. Across the two oracle builds that is 666 new passing variants and 780 new lifecycle obligations. Falsifiers cover 65 descriptor pairs per build. [drain-counts.json](../../../.temp/brief-591/drain-counts.json) records counts checked against the actual logs.

Every drained entry and attachment is listed below. These are all 51 changed effect rows, and the same 51 removed owed rows. Each effect row changes only column 7 (descriptor IDs), from `-` to the IDs shown; policy, owner, variants, family and feature cells stay fixed. Complete before/after attachment and removed-row evidence: [effect-attachment-changes.tsv](../../../.temp/brief-591/effect-attachment-changes.tsv) and [owed-rows-removed.tsv](../../../.temp/brief-591/owed-rows-removed.tsv).

| Drained entry | Attached descriptors (suffixes after the entry ID) | Variants | Lifecycle |
| --- | --- | ---: | ---: |
| `Accumulate.arithmetic` | `primary` | 5 | 6 |
| `Array.callback` | `primary` | 5 | 6 |
| `ArrayFilter.callback` | `primary` | 5 | 6 |
| `ArrayFilter.defaultOption` | `primary`, `/callback-after-resume` | 10 | 12 |
| `ArrayFilter.explicitOption` | `primary` | 5 | 6 |
| `ArrayReduce.callback` | `primary` | 5 | 6 |
| `BlockMap.callback` | `primary` | 5 | 6 |
| `Comap.callback` | `primary` | 5 | 6 |
| `ComapApply.callback` | `primary` | 5 | 6 |
| `Dot.arithmetic` | `primary`, `/reduction`, `/later-operand`, `/contracted-prefix` | 20 | 24 |
| `FixedPoint.callback` | `primary` | 5 | 6 |
| `FixedPointList.callback` | `primary` | 5 | 6 |
| `Fold.callback` | `primary` | 5 | 6 |
| `FoldList.callback` | `primary` | 5 | 6 |
| `FoldPair.callback` | `primary`, `/projector-after-resume` | 10 | 12 |
| `FoldPair.combiner` | `primary` | 5 | 6 |
| `FoldWhile.callback` | `primary`, `/test-after-resume`, `/positive-tail` | 15 | 18 |
| `FoldWhile.predicate` | `primary` | 6 | 6 |
| `Inner.arithmetic` | `primary`, `/reduction`, `/term-to-reduction`, `/contracted-prefix` | 20 | 24 |
| `LengthWhile.predicate` | `primary` | 6 | 6 |
| `Map.callback` | `primary` | 5 | 6 |
| `MapAll.callback` | `primary` | 5 | 6 |
| `MapAt.callback` | `primary` | 5 | 6 |
| `MapIndexed.callback` | `primary` | 5 | 6 |
| `MapThread.callback` | `primary` | 5 | 6 |
| `Nest.callback` | `primary` | 5 | 6 |
| `NestList.callback` | `primary` | 5 | 6 |
| `NestWhile.callback` | `primary` | 5 | 6 |
| `NestWhile.predicate` | `primary` | 6 | 6 |
| `NestWhileList.callback` | `primary` | 5 | 6 |
| `NestWhileList.predicate` | `primary` | 6 | 6 |
| `Operate.callback` | `primary` | 5 | 6 |
| `Outer.callback` | `primary` | 5 | 6 |
| `Select.predicate` | `primary` | 6 | 6 |
| `SelectFirst.predicate` | `primary` | 6 | 6 |
| `SequenceFold.callback` | `primary` | 5 | 6 |
| `SequenceFoldList.callback` | `primary` | 5 | 6 |
| `SubsetMap.callback` | `primary`, `/rebuild`, `/rebuild-after-resume` | 15 | 18 |
| `TakeWhile.predicate` | `primary` | 6 | 6 |
| `Through.callback` | `primary` | 5 | 6 |
| `LazyDrop.source` | `primary` | 5 | 6 |
| `LazyFoldList.callback` | `primary` | 5 | 6 |
| `LazyFoldList.source` | `primary` | 5 | 6 |
| `LazyMap.callback` | `primary` | 5 | 6 |
| `LazyMap.source` | `primary` | 5 | 6 |
| `LazyNestList.callback` | `primary` | 5 | 6 |
| `LazySelect.predicate` | `primary` | 6 | 6 |
| `LazySelect.source` | `primary` | 5 | 6 |
| `LazyTable.body` | `primary` | 5 | 6 |
| `LazyTable.bounds` | `primary`, `/upper`, `/step` | 15 | 18 |
| `LazyTake.source` | `primary` | 5 | 6 |

The four entries retained owed are unchanged in the ledger:

| Entry | Source-backed reason |
| --- | --- |
| `ArrayFilter.optionSyntax` | Ordinary registration; a computed fourth argument is evaluated by shared preparation. The handler partitions three/four operands and parses an already-present Padding rule. See `catalog/functional.rs:458`, `functional_ops/arrays/mod.rs:132` and `arrays/spec.rs:49`. |
| `FixedPoint.predicate` | The argument forms accept an operator, seed and optional iteration limit. Termination uses direct Expr equality (`functional_ops/iteration.rs:229`, `:410`, `:448`); there is no guest predicate parameter. |
| `FixedPointList.predicate` | Same direct equality termination, with collected outputs (`functional_ops/iteration.rs:205`); no guest predicate. |
| `LazyNestList.source` | Ordinary construction eagerly prepares the seed and stores it; it has no child source. Pulls clone current and invoke the callback (`lazy.rs:100`, `lazy/source.rs:75`). A seed-production pause would exercise shared preparation. |

Full source excerpts are in [roles-kept-owed.md](../../../.temp/brief-591/roles-kept-owed.md). The other five lazy source roles pause while an inner source is pulled, with the named transformer retained around it; a `before` projection proves construction ran zero source effects. `ArrayFilter.explicitOption` is a real effect: its literal delayed padding payload is forced in the local tensor callback arguments. Its 30 padding evaluations span five neighborhoods, with pauses inside neighborhoods three and five; it is covered and drained.

Four policies should be corrected by the director: `Map.callback`, `MapIndexed.callback`, `MapAll.callback`, and `MapThread.callback` currently say Frame, but their implementations return constructed callback applications for tail evaluation. All four descriptors are attached and passing; their policy cells were left unchanged. [policy-corrections.md](../../../.temp/brief-591/policy-corrections.md) gives the source evidence.

Loop fixtures use at least five visits; ordinary fixtures pause at visits two and four. Additional factories cover tensor terms and reductions, partial contraction/output prefixes, later Dot operands, all three LazyTable bound slots, FoldPair projector entry from resume, FoldWhile predicate entry and positive-tail work, and both SubsetMap result-reconstruction owners. Single-callback and option/bound phases use five ordered effects within that phase. Exact Error, Decline, AlreadyResumed and Nested plans are attached wherever the effect row requires them.

Verbatim baseline catalog rows:

```text
CATALOG hash=true installed=643 manifest=643 GuestEntries=238 NoLocalGuestEffect=405 Boundary=4 entries=483 descriptors=263 historical=209 owed=241
CATALOG hash=false installed=637 manifest=637 GuestEntries=238 NoLocalGuestEffect=399 Boundary=4 entries=483 descriptors=263 historical=209 owed=241
```

Verbatim final catalog and lifecycle rows:

```text
LIFECYCLE obligations=1944 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=238 NoLocalGuestEffect=405 Boundary=4 entries=483 descriptors=328 historical=209 owed=190
LIFECYCLE obligations=1944 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=238 NoLocalGuestEffect=399 Boundary=4 entries=483 descriptors=328 historical=209 owed=190
```

### 3. Defects, verification and every inventory change


**Defect witnesses: 0.** No queued/resumed discrepancy was found. The four remaining entries are declaration findings, with source evidence above, and remain owed for the director’s decision.

All Cargo commands ran serially through `scripts/run-bounded.py`, with `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true` and the brief-local TMPDIR. Bounds were 900 seconds per oracle build, 300 seconds for runtime/ownership/retention, and 3000 seconds for the workspace. The canonical oracle script hardcodes its scratch path; during its run that path temporarily resolved into this brief’s directory, including TMPDIR. Its original scratch directory was restored intact afterward; see [oracle-script-paths.txt](../../../.temp/brief-591/oracle-script-paths.txt).

`cargo fmt --check`: exit 0, no output. [fmt-final.txt](../../../.temp/brief-591/fmt-final.txt).

`scripts/check-suspension-oracle.sh 900` — both builds:

```text
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.51s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.20s
```

Role-specific falsifiers also passed in both builds. The third candidate run’s actual counter line is:

```text
CANDIDATE COUNTERS variants=333 lifecycle=390 failures=0
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 3.10s
```

The complete per-descriptor variant and falsifier lines are in [oracle-script/gate-default.txt](../../../.temp/brief-591/oracle-script/gate-default.txt) and [oracle-script/gate-no-default.txt](../../../.temp/brief-591/oracle-script/gate-no-default.txt).

`cargo test -p rum-runtime`:

```text
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.05s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.71s
```

[runtime.txt](../../../.temp/brief-591/runtime.txt).

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`:

```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 59.84s
```

[ownership.txt](../../../.temp/brief-591/ownership.txt).

`cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`:

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.17s
```

[fingerprints.txt](../../../.temp/brief-591/fingerprints.txt).

All seven ownership inventories are byte-identical. No existing row changed/disappeared and no new production UnknownHostPayload, self-reaching or untouched-source row appeared. [ownership-reconciliation.tsv](../../../.temp/brief-591/ownership-reconciliation.tsv).

All **68** local-contract rows change only the final generated fingerprint cell. Identities, contract text, status, strength and exclusions are unchanged. Every changed row ID:

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

Every before/after fingerprint is disclosed in [fingerprint-changes.tsv](../../../.temp/brief-591/fingerprint-changes.tsv). The replay output was copied from the regeneration test, not invented.

Initial certificate replays failed on the expected stale certificate and emitted the replacement cells. Verbatim initial failure evidence:

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.00s
```

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.76s
```

Only the hash=true/hash=false cells of promoted rows **C01** and **C02** change. The guard verifies that all differences are direct-contract fingerprint values; instance identity, build, body/domain fingerprints, marks, source positions, preconditions, strength and exclusions remain identical. Every full changed cell is in [certificate-changes.tsv](../../../.temp/brief-591/certificate-changes.tsv). Summary:

```text
body	feature	changed direct-contract fingerprints	instance marks
C01	true	O0210,O011,O0200,O010	fnv1a64:db543eaa6df76620,fnv1a64:c6cb017dd1358c2a
C01	false	O0210,O011,O0200,O010	fnv1a64:507d9ba1e842e023,fnv1a64:f5a09329759e61a5
C02	true	I01,I02,I04,R01,R02,B13,B05	fnv1a64:ce01acda3c73dc98
C02	false	I01,I02,I04,R01,R02,B13,B05	fnv1a64:479e00b3640f75f0
```

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 17.84s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.83s
```

Both final acceptances are green: [retention-default.txt](../../../.temp/brief-591/retention-default.txt), [retention-no-default.txt](../../../.temp/brief-591/retention-no-default.txt), [maintenance.txt](../../../.temp/brief-591/maintenance.txt). All other retention inventories are unchanged.

`env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture` — exit 0. Verbatim target/result lines:

```text
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.89s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.01s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.33s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1037.45s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.33s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.84s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.53s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The Jupyter harness reports 151 passed, including **nine native ZeroMQ skips** caused by denied loopback binding; 142 tests executed. Native transport success is not claimed. The existing opt-in only permits the PermissionDenied skip branch (`crates/rum-jupyter/src/zeromq_native_test_support.rs:335`). Verbatim skips:

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

Full workspace log: [workspace.txt](../../../.temp/brief-591/workspace.txt). All mandatory commands ran and their final gates passed. The two initial stale-certificate failures were resolved by replaying generated cells; the nine native cases are skips, not successful native transport tests. No other inventory rows changed. The exhaustive inventory audit is [all-inventory-audit.tsv](../../../.temp/brief-591/all-inventory-audit.tsv).

### 4. Stop-loss counters and remaining decisions


- Entries drained: **51/55**; kept owed: **4**, all source-backed nonexistent-role findings.
- Added variants: **333/333 per build**, **666/666** across both oracle builds.
- Added lifecycle obligations: **390/390 per build**, **780/780** across both, plus completion and fresh requests for every variant.
- Falsifiers: **65/65 descriptor pairs per build**, all strict rejections, weak controls and good controls passed.
- Defects: **0**; existing tests renamed/removed: **0**.
- Stop-loss did not trigger. Three compiling implementation attempts advanced variants/lifecycle: **283/330**, **323/378**, **333/390**. Remaining runs were mandatory verification and generated maintenance replay.

The remaining director decisions are removal/correction of the four nonexistent-role declarations and correction of the four mapping policy cells. No production repair group is required by these results. All changes remain uncommitted. Final checkpoint: [PROGRESS.md](../../../.temp/brief-591/PROGRESS.md).
