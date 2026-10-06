# brief-588 — E1 drain 2 of 8: the `rewrite` family (55 owed entries)

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) and left **500** declared
guest-effect entries in an exact owed ledger (`inventory/owed.tsv`); brief-587
drained the `rules` family (owed 500 -> 430, two misdeclared `Level` entries
removed at the director's gate). **This one drains the 55 entries of the
`rewrite` family** — exactly the IDs listed in
`.temp/brief-586/followups/draft-E1-12-rewrite.md`: the option, guard,
pattern-test, pattern-preparation, held-spec, held-target and delayed
right-hand-side roles of `Cases`, `DeleteCases`, `MatchQ`, `Replace`,
`ReplaceAll`, `ReplaceAllUnheld`, `ReplaceList` and `ReplaceRepeated`.

Each entry leaves the ledger **only in the same change that attaches and runs
its complete descriptor**: every required variant (`Middle`, `Twice`, and the
`Decline`, `Error`, `AlreadyResumed`, `Nested` variants its effect row
records), lifecycle at the first and second pause for every distinct
continuation owner (completion, cancellation, reset, a pending request
rejecting a new one), and a fresh request after.

That draft is the raw material; **this brief governs** where they differ.

## A failing descriptor is a finding, not a test to fix

If a correct descriptor shows the resumed path differing from the queued one —
a lost continuation the census did not see — **do not weaken the descriptor,
do not repair production code, and do not commit a failing test.** Keep the
entry owed, write the witness (source, answers, both observations) to
`.temp/brief-588/defects/<entry>.md`, and report it. It becomes a repair group
of this branch. The same holds for a manifest disposition you find wrong:
report it with source evidence and keep the entry owed; do not change it in
this brief (brief-587 found two such rows and one wrong policy; the director
corrected them at the gate). An `Ordinary` builtin's explicit option is
evaluated by shared preparation — check the registration before declaring a
local option phase missing.

## Context pointers

- brief-586 and brief-587 and their reports; `.temp/brief-586/followups/draft-E1-12-rewrite.md`;
  brief-587's descriptors in `fixtures/rules/` as the model.
- `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors,catalog}.rs`,
  `support/`, `fixtures/`, `inventory/{catalog,effects,shared-effects,owed,historical-labels}.tsv`.
- The owning functions named in the draft; the guide
  (`evaluation.md`, `patterns-and-rules.md`) and the generated reference are
  the contract.
- The existing `rule_pattern_continuations.rs`, `option_reader_continuations.rs`
  and `loop_continuations.rs` (`ReplaceRepeated`) cases, reusable as
  descriptors.

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
TMPDIR=<repo>/.temp/brief-588/tmp …`. Bounds: the oracle target 900 s per
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

Everything under `.temp/brief-588/`; `PROGRESS.md` after each milestone with
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

## Implementation report — brief 588

**49 of 55 rewrite entries are drained. Six remain owed because their declared
option roles are unavailable in the registered special forms.** No
resumed-versus-queued semantic mismatch was found. All mandatory final gates
passed; nine native ZeroMQ tests reported permission skips. Changes are left
uncommitted.

The actual starting ledger was **428**, following the director's removal of
the two Level rows after brief 587. It is now **379** in both builds: exactly
49 fewer. The brief's 430 describes brief 587's pre-gate report. No other
family's entry changed.

### 1. Files changed and no-production-edit audit

```text
crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv
crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv
crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rewrite_roles/falsifiers.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rewrite_roles/held.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rewrite_roles/lifecycle.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rewrite_roles/mod.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rewrite_roles/options.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rewrite_roles/patterns.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rewrite_roles/replacements.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rewrite_roles/support.rs
crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv
crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv
docs/dev/codex/brief-588-oracle-drain-rewrite.md
```

The eight new `fixtures/rewrite_roles/` modules separate the registry, shared
setup, held inputs, options, matcher preparation/callbacks, replacements,
lifecycle assertions and falsifiers. `fixtures/mod.rs` declares the module;
`fixtures/primary.rs` attaches its registry. Existing historical descriptors,
continuation suites and oracle support modules are unchanged. The two oracle
inventories attach only executed descriptors and remove their exact owed rows.
The two retention inventories contain only replayed fingerprint/certificate cells.

Factories provide explicit expected values or typed errors, logs, prints,
state projections, and exact input kinds, prompts and answers. Five ordered
operations pause at positions two and four. Traversal callbacks visit five
candidates; MatchQ matches five argument slots; ReplaceList visits five root
rules; ReplaceRepeated callback/rhs cases run five changing passes (decline at
pass five). ReplaceAllUnheld additionally preserves a held integer without
invoking its callback. Error variants throw at operation five; AlreadyResumed
adds an earlier InputString caller; Nested retains a Block scope and checks
restoration. Lifecycle checks the precise two/four-operation prefix and state
on cancel/reset, rejects an actual overlapping request, and checks a fresh
request after each path.

SHA256 comparison with the initial scene found no net production Rust, lib.rs,
manifest/lockfile, dependency, script, benchmark, other-documentation,
AGENTS/TODO/root-PROGRESS change, and no removed baseline file. All changed
Rust modules are below 10k characters. No unsafe, git command, external network
access, commit, or production repair was used. No hot path changed, so no
performance gate was required or run.

Audit evidence: [audit.txt](../../../.temp/brief-588/audit.txt), [baseline-hashes.json](../../../.temp/brief-588/baseline-hashes.json), [final-source-sizes.json](../../../.temp/brief-588/final-source-sizes.json).

No existing test name was renamed or removed. The sole added final test is
`oracle::fixtures::rewrite_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log`.
The temporary passing `candidate_roles` test was archived in scratch and removed
before admission. All 209 historical labels and all historical continuation
suite source files are unchanged.

```text
Before 10; after 11; removed/renamed 0; added 1
oracle::fixtures::rewrite_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log
Historical continuation suites and the 209 historical descriptor names are unchanged.
```

### 2. Exact drained and kept entries; coverage

Each ID below identifies both edits: its effects descriptor cell changes from
`-` to the exact ID, and its original owed row is removed. All other effect
fields, including policy, owner, variants and lifecycle owner, are unchanged.

```text
Cases.defaultOption
Cases.delayedRhs
Cases.explicitOption
Cases.guard
Cases.heldSpec
Cases.heldTarget
Cases.optionSyntax
Cases.patternPreparation
Cases.patternTest
DeleteCases.defaultOption
DeleteCases.explicitOption
DeleteCases.guard
DeleteCases.heldSpec
DeleteCases.heldTarget
DeleteCases.optionSyntax
DeleteCases.patternPreparation
DeleteCases.patternTest
MatchQ.guard
MatchQ.patternPreparation
MatchQ.patternTest
Replace.defaultOption
Replace.delayedRhs
Replace.explicitOption
Replace.guard
Replace.heldSpec
Replace.heldTarget
Replace.patternPreparation
Replace.patternTest
ReplaceAll.delayedRhs
ReplaceAll.guard
ReplaceAll.patternPreparation
ReplaceAll.patternTest
ReplaceAllUnheld.delayedRhs
ReplaceAllUnheld.guard
ReplaceAllUnheld.patternPreparation
ReplaceAllUnheld.patternTest
ReplaceList.delayedRhs
ReplaceList.guard
ReplaceList.heldSpec
ReplaceList.heldTarget
ReplaceList.patternPreparation
ReplaceList.patternTest
ReplaceRepeated.delayedRhs
ReplaceRepeated.explicitOption
ReplaceRepeated.guard
ReplaceRepeated.heldSpec
ReplaceRepeated.heldTarget
ReplaceRepeated.patternPreparation
ReplaceRepeated.patternTest
```

Exhaustive old/new effect rows: [effect-row-changes.tsv](../../../.temp/brief-588/effect-row-changes.tsv). Every removed owed row: [owed-rows-removed.tsv](../../../.temp/brief-588/owed-rows-removed.tsv).

| Kept owed | Reason and source witness |
| --- | --- |
| `Replace.optionSyntax` | Only literal Heads rules enter the option resolver; computed syntax is treated as a levelspec or invalid extra argument. [Witness](../../../.temp/brief-588/defects/Replace.optionSyntax.md) |
| `ReplaceList.defaultOption` | No default option read; the optional third argument is a positional maximum-results limit. [Witness](../../../.temp/brief-588/defects/ReplaceList.defaultOption.md) |
| `ReplaceList.explicitOption` | No named option reader; evaluating a rule in the positional limit slot cannot stand in for this role. [Witness](../../../.temp/brief-588/defects/ReplaceList.explicitOption.md) |
| `ReplaceList.optionSyntax` | No option syntax resolver; a computed named option fails the positional limit parser. [Witness](../../../.temp/brief-588/defects/ReplaceList.optionSyntax.md) |
| `ReplaceRepeated.defaultOption` | The absent-option path reads the resource-limit ceiling directly, without a guest default read. [Witness](../../../.temp/brief-588/defects/ReplaceRepeated.defaultOption.md) |
| `ReplaceRepeated.optionSyntax` | The initial shape check requires a literal MaxIterations rule before evaluation. [Witness](../../../.temp/brief-588/defects/ReplaceRepeated.optionSyntax.md) |

All eight registrations were checked in `crates/rum-stdlib/src/catalog/rewrite.rs`:
they use `BuiltinSpec::special`. Ordinary shared argument preparation therefore
does not supply an alternative option phase. These are source-audit findings,
not resumed-versus-queued defects; no runtime observations or passing variants
are claimed for unavailable roles. All six ledger/disposition rows remain
unchanged for the director's decision.

| Build | Owed before | Owed after | New descriptors | New variants | New lifecycle obligations |
| --- | ---: | ---: | ---: | ---: | ---: |
| default / hash=true | 428 | 379 | 49 | 261 | 294 |
| no-default / hash=false | 428 | 379 | 49 | 261 | 294 |

Each build also passes 98 strict negative controls (49 missing-pause and 49
lost-log), 98 weakened controls, and 49 good controls. Lifecycle obligations
count cancel/reset/pending at both pauses; completion and fresh requests are
also checked for every variant. The total registry has 726 lifecycle obligations.

Every variant/lifecycle/falsifier line: [coverage-default.txt](../../../.temp/brief-588/coverage-default.txt) and [coverage-no-default.txt](../../../.temp/brief-588/coverage-no-default.txt).

Verbatim catalog output:

```text
LIFECYCLE obligations=726 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=240 NoLocalGuestEffect=403 Boundary=4 entries=504 descriptors=125 historical=209 owed=379
OWED families={"arithmetic": 3, "associations": 13, "associations_transform": 19, "contexts": 12, "control": 50, "debug": 41, "evaluator": 1, "functional": 43, "lazy": 12, "logic": 5, "predicates": 45, "random": 1, "rewrite": 6, "rules": 1, "schema": 4, "scoping": 15, "strings": 24, "strings_match": 38, "strings_text": 10, "structure": 6, "structure_ordering": 21, "symbols": 9}
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.33s
```

```text
LIFECYCLE obligations=726 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=240 NoLocalGuestEffect=397 Boundary=4 entries=504 descriptors=125 historical=209 owed=379
OWED families={"arithmetic": 3, "associations": 13, "associations_transform": 19, "contexts": 12, "control": 50, "debug": 41, "evaluator": 1, "functional": 43, "lazy": 12, "logic": 5, "predicates": 45, "random": 1, "rewrite": 6, "rules": 1, "schema": 4, "scoping": 15, "strings": 24, "strings_match": 38, "strings_text": 10, "structure": 6, "structure_ordering": 21, "symbols": 9}
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.35s
```

### 3. Verification, findings and every inventory change

Commands ran serially through `scripts/run-bounded.py`, with
`CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true` and outer
`TMPDIR=/home/remi/dev/rum/.temp/brief-588/tmp`. Oracle builds were bounded at
900 seconds each, runtime/ownership/retention at 300 seconds, workspace at
3000 seconds. The mandated unmodified `scripts/check-suspension-oracle.sh`
hard-codes its inner TMPDIR/output to `.temp/suspension-oracle`; its complete
outputs were copied into this checkpoint directory. The fingerprint generator
uses its standard `target/brief-544` review path; certificate replay uses the
permitted `.temp/brief-546/certificates`. No script was edited.

`cargo fmt --check`: exit 0, no output.

[fmt.txt](../../../.temp/brief-588/fmt.txt).

`scripts/check-suspension-oracle.sh 900`: exit 0 in both builds, with catalog/result lines above. [oracle-gate.txt](../../../.temp/brief-588/oracle-gate.txt). Representative verbatim falsifiers (every role is recorded in the coverage files):

```text
REWRITE_FALSIFIERS Cases.patternPreparation missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS
REWRITE_FALSIFIERS MatchQ.guard missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS
REWRITE_FALSIFIERS ReplaceList.delayedRhs missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS
REWRITE_FALSIFIERS ReplaceRepeated.patternTest missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS
```

`cargo test -p rum-runtime`:
```text
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.10s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.86s
```
[runtime.txt](../../../.temp/brief-588/runtime.txt).

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`:
```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 59.93s
```
[ownership.txt](../../../.temp/brief-588/ownership.txt).

All ownership inventories are byte-identical. Live discovery reconciles 59
self-reaching rows, 65 UnknownHostPayload rows and 7,683 UnknownConstruct rows.
No production row appeared, disappeared, changed identity/status/disposition,
or moved source position. Every other retention inventory is also byte-identical.
No forbidden inventory drift occurred.

`cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`:

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.21s
```

All 68 rows below change only their final generated fingerprint cell. Identities, contract text, statuses and exclusions are unchanged.

```text
id	before	after
B01	fnv1a64:c7ca4497c1289b99	fnv1a64:a246410e6669c3d6
B02	fnv1a64:b9da2fea8b257e74	fnv1a64:7f54a8ffbb1944b3
B03	fnv1a64:08448190db456cfb	fnv1a64:ef3a338f8538b9cc
B04	fnv1a64:f0af8100526e03c1	fnv1a64:7362d09842c9670e
B05	fnv1a64:cd7982cad800b0d8	fnv1a64:b081d79b00b169df
B06	fnv1a64:019bb1612e355d35	fnv1a64:4374ef3a72623cea
B07	fnv1a64:570a8b53dbb94f68	fnv1a64:06e9dd63625f076f
B08	fnv1a64:37bd6c9dfb972a3f	fnv1a64:1ef0845229f313b8
B09	fnv1a64:eb683f91324357fd	fnv1a64:b14a1790699f7762
B10	fnv1a64:eb530a0d02ab008e	fnv1a64:425f4be642988b31
B11	fnv1a64:b207325d6c6229c1	fnv1a64:34ba81f55cbd8d0e
B12	fnv1a64:aa74bf8d0e4becfd	fnv1a64:7056978c45a80c62
I01	fnv1a64:a5ac2f952de09219	fnv1a64:80282c0bd321ba56
I02	fnv1a64:24bb005ed03738d8	fnv1a64:07c3552ef8e7f1df
I03	fnv1a64:45764f3df2e6e0bd	fnv1a64:7990f50bd88bce22
I04	fnv1a64:2f4f728f0dcd5414	fnv1a64:a3eba58d978d0dd3
P01	fnv1a64:1b24abd49a60f655	fnv1a64:c73920268f862f8a
P02	fnv1a64:53155d20d6b0d47e	fnv1a64:f091458d97bc4ae1
P03	fnv1a64:f6ee32dc1ad1c493	fnv1a64:6af9fb7d9ddc7634
P04	fnv1a64:aeb54e000fce9a74	fnv1a64:742fc7153fc260b3
P05	fnv1a64:d6660ab9ed319609	fnv1a64:f49b486e386352c6
P06	fnv1a64:faf3e45d9a7b867d	fnv1a64:c0d5bc5cd1d7a5e2
P070	fnv1a64:3541809f78f1007d	fnv1a64:fb23589eb04d1fe2
P080	fnv1a64:f0af8fea0e270f4e	fnv1a64:3351c18ef445d6f1
P090	fnv1a64:a2c4566bb1d22834	fnv1a64:a291e3ec4f67de73
P100	fnv1a64:163181cb2e904f91	fnv1a64:7a003d40dd0f2b5e
P110	fnv1a64:4b000df1b95310a6	fnv1a64:d0d1a35275f88e99
P120	fnv1a64:5b50053e5ef5d55f	fnv1a64:bb1bac41e5f6cfd8
P130	fnv1a64:fad7e4b10a6d2968	fnv1a64:aab736c09112e16f
P140	fnv1a64:c016498e0a91606c	fnv1a64:80e1c1f9b8681a3b
P150	fnv1a64:1f0ca0315ae7cdd4	fnv1a64:65ecf7d893792493
P071	fnv1a64:fc1c8e6f67701a88	fnv1a64:110f8053c97d438f
P081	fnv1a64:40d451f223dcf19d	fnv1a64:2c1d746cd8279b02
P091	fnv1a64:93e14efea47c7415	fnv1a64:3c53b69786c41d4a
P101	fnv1a64:4302febeeeb6ada6	fnv1a64:c8d4941fab5c2b99
P111	fnv1a64:a91dd27f16760b0d	fnv1a64:e6e0d34eeaaec072
P121	fnv1a64:6345378c86c9fa9a	fnv1a64:cc137a48bc025505
P131	fnv1a64:8e59a36688c22541	fnv1a64:110cf2fe791d888e
P141	fnv1a64:78613c127bd639f5	fnv1a64:5256d35a89aab6aa
P151	fnv1a64:4cce183ff5554d9f	fnv1a64:0d03da4798554018
B13	fnv1a64:8fac44ea5ed21935	fnv1a64:d18582c3a2fef8ea
R01	fnv1a64:9c9734e1b2ddd984	fnv1a64:7bc7dfb7ecd70b43
R02	fnv1a64:762d4fbfa21b2453	fnv1a64:a217418595cf58f4
R03	fnv1a64:d8321bd640c36850	fnv1a64:b0367ef4c40d7847
R040	fnv1a64:9c3e44335cf9af24	fnv1a64:a900e5121fe6cde3
R050	fnv1a64:aec7a37cb299a069	fnv1a64:9e2b523a5916aba6
R060	fnv1a64:bbc4f8ada0044c3c	fnv1a64:8f94fc6f18eb2d8b
R070	fnv1a64:a9421a0c057eb09d	fnv1a64:948b3c86b9c95a02
R080	fnv1a64:f653db81dd521d14	fnv1a64:6af00e806711d6d3
R090	fnv1a64:ee6de186fb5c7751	fnv1a64:aa78c37899bc591e
R100	fnv1a64:cd6a25e3851b177b	fnv1a64:b45fd7e22f0e644c
R110	fnv1a64:e94a722266c93cf6	fnv1a64:c0bf834a24e4fe29
R041	fnv1a64:75e5a81475f9d835	fnv1a64:b7bee5edba26b7ea
R051	fnv1a64:30e684813f0034b4	fnv1a64:30b41201dc95eaf3
R061	fnv1a64:a4fbda2ec37f8aef	fnv1a64:cf3658f588f8df68
R071	fnv1a64:1e8f5073d7d7e082	fnv1a64:49b68a58bed7455d
R081	fnv1a64:d21beb1fca3c2491	fnv1a64:35eaa69578bb005e
R091	fnv1a64:443f8e0c649dabac	fnv1a64:bf6dfd12fd17f57b
R101	fnv1a64:7548888d3953e0fc	fnv1a64:10afafd02f80914b
R111	fnv1a64:435fa8fc55a9b327	fnv1a64:10ee2317406d5f10
R12	fnv1a64:e6012e79e06cbe67	fnv1a64:a44163135fc55c50
R13	fnv1a64:4ca57bf089dfb3ab	fnv1a64:3bcccfa5e7046e3c
R14	fnv1a64:308990410b410f91	fnv1a64:94584bb6b9bfeb5e
R15	fnv1a64:68985b307cd7f1e5	fnv1a64:8438ac1a83acda1a
O010	fnv1a64:81f1537f35e2c84f	fnv1a64:615f347546c34fc8
O0200	fnv1a64:baddeb625948866c	fnv1a64:7ba963ce071f403b
O011	fnv1a64:fe4db535ad9610bc	fnv1a64:d21db8f7267cf20b
O0210	fnv1a64:9fd4dfe0f234fac7	fnv1a64:81882e47a1326fb0
```
[fingerprint-changes.tsv](../../../.temp/brief-588/fingerprint-changes.tsv).

The first acceptance replay in each feature build returned the expected stale-certificate error and generated review cells. Verbatim failed replay evidence:

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 17.90s
```

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.91s
```

`promoted.tsv` changes only the hash=true/hash=false certificate cells in rows
C01 and C02. Those cells differ solely in embedded direct-contract fingerprints;
instance identities/builds, body/domain fingerprints, instance marks, source
positions, status, preconditions, strength and exclusions remain unchanged.

Exact four before/after cells: [certificate-changes.tsv](../../../.temp/brief-588/certificate-changes.tsv). Changed references and unchanged instance marks:
```text
body	feature	changed embedded contract fingerprints	unchanged instance marks
C01	hash=true	O010,O011,O0200,O0210	fnv1a64:db543eaa6df76620,fnv1a64:c6cb017dd1358c2a
C01	hash=false	O010,O011,O0200,O0210	fnv1a64:507d9ba1e842e023,fnv1a64:f5a09329759e61a5
C02	hash=true	B05,B13,I01,I02,I04,R01,R02	fnv1a64:ce01acda3c73dc98
C02	hash=false	B05,B13,I01,I02,I04,R01,R02	fnv1a64:479e00b3640f75f0
```

After replay, `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`:
```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 17.79s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`:
```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.08s
```

Full replay evidence: [maintenance.txt](../../../.temp/brief-588/maintenance.txt), [retention-default-before.txt](../../../.temp/brief-588/retention-default-before.txt), [retention-no-default-before.txt](../../../.temp/brief-588/retention-no-default-before.txt), [retention-default.txt](../../../.temp/brief-588/retention-default.txt), [retention-no-default.txt](../../../.temp/brief-588/retention-no-default.txt).

Workspace command: `cargo test --workspace --no-fail-fast -- --nocapture`,
with supported `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`. The flag permits a native skip
only when loopback binding is denied. Verbatim target/result lines:

```text
Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.09s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.03s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1062.37s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.63s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.46s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.95s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.74s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Workspace exit 0. Across 49 harness result blocks: 3363 reported passed, 0 failed, 58 ignored. **Nine reported passes are native ZeroMQ permission skips, not executed native coverage.** Verbatim skip output:

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

[Full workspace output](../../../.temp/brief-588/workspace.txt), [workspace-results.txt](../../../.temp/brief-588/workspace-results.txt), [native-skips.txt](../../../.temp/brief-588/native-skips.txt).

No mandatory verification command was omitted. The two stale-certificate
replay failures were resolved by generated cells; every final gate is green.
No compiling candidate failed, no descriptor was weakened, and no failing test
remains in the tree.

### 4. Stop-loss counters and open questions

- Entries drained: **49/55**; kept owed: **6**, each with source evidence.
- New variants passing: **261 per build**, 522 across both.
- New lifecycle obligations: **294 per build**, 588 across both, plus completion
  and fresh-request checks for every variant.
- Strict falsifiers: **98 per build**, all rejected as intended; weakened and
  good controls pass.
- Resumed-versus-queued defects: **0**. Disposition witnesses: **6**.
- Consecutive compiling implementation attempts without progress: **0**.
  The first candidate run completed all 49 viable entries; admission and
  falsifiers completed next. Stop-loss was not triggered. Required gates and
  generated maintenance replay followed implementation.

Director decision outstanding: reconcile the six unsupported option-role
manifest dispositions above. This brief leaves all six owed and makes no
production or disposition repair. No other open implementation work.
