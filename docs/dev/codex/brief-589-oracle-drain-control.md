# brief-589 — E1 drain 3 of 8: `control`, `logic`, `arithmetic` and `random` (59 owed entries)

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) and left **500** declared
guest-effect entries in an exact owed ledger (`inventory/owed.tsv`). Drains 1
and 2 (briefs 587-588) took the `rules` and `rewrite` families; eight
misdeclared roles were removed at the director's gate (owed 500 -> 379).
**This one drains 59 entries** — exactly the IDs listed in
`.temp/brief-586/followups/draft-E1-{05-control,09-logic,01-arithmetic,11-random}.md`:
the bodies, tests, branches, bounds, handlers, patterns and cleanups of the
control-flow builtins (`AbortProtect`, `Assert`, `Catch`, `CatchExceptions`,
`CheckAbort`, `CompoundExpression`, the `Confirm*` family, `Do`, `Enclose`,
`Failsafe`, `Failure`, `For`, `If`, `Product`, `Sum`, `Switch`, `Table`,
`Until`, `Which`, `While`, `WithCleanup`), the operands of `And`, `Implies`,
`Nand`, `Nor`, `Or`, the arithmetic of `Mean`, `Median`, `Total`, and
`BlockRandom`'s body.

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
`.temp/brief-589/defects/<entry>.md`, and report it. It becomes a repair group
of this branch. The same holds for a manifest disposition you find wrong:
report it with source evidence and keep the entry owed; do not change it in
this brief (briefs 587-588 found eight roles that do not exist — an `Ordinary`
builtin's option evaluated by shared preparation, a positional argument
declared as an option, option syntax admitted only literally — and one wrong
policy; the director corrected them at the gate). Check the registration
(`Ordinary` or held) and the argument partition before declaring a local role
missing. Short-circuit operands (`And`, `Or`, …) and branch selection must pause
in an operand or branch the builtin itself evaluates, after an earlier one has
run.

## Context pointers

- briefs 586-588 and their reports; the four drafts above; the descriptors in
  `fixtures/rules/` and `fixtures/rewrite_roles/` as the model.
- `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors,catalog}.rs`,
  `support/`, `fixtures/`, `inventory/{catalog,effects,shared-effects,owed,historical-labels}.tsv`.
- The owning functions named in the draft; the guide
  (`evaluation.md`, `patterns-and-rules.md`) and the generated reference are
  the contract.
- The existing `loop_continuations.rs` and `option_reader_continuations.rs`
  cases, and the historical control-flow labels in
  `inventory/historical-labels.tsv`, reusable as descriptors. `WithCleanup`'s
  cleanup and `CheckAbort`/`AbortProtect` interact with cancellation: the
  lifecycle assertions follow the documented cleanup policy (cleanup may run
  on cancellation, never on reset).

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
TMPDIR=<repo>/.temp/brief-589/tmp …`. Bounds: the oracle target 900 s per
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

Everything under `.temp/brief-589/`; `PROGRESS.md` after each milestone with
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

## Executing-agent report — brief-589

**51 of the 59 target entries drained; eight disposition findings remain owed.**
No resumed-versus-queued semantic mismatch was found. Every mandatory final gate
passed; 9 native ZeroMQ tests skipped because loopback binding is denied.
Changes remain uncommitted. No git command or network access was used.

The actual starting ledger in this checkout was **373**, verified by both
feature builds, rather than the brief narrative's 379. It is now **322** in both
builds: exactly 51 fewer. No other family's rows were removed or reclassified.

### 1. Files changed and no-production-edit audit

```text
crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv
crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/arithmetic.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/bodies.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/conditions.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/falsifiers.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/lifecycle.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/loops.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/mod.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/options_properties.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/phases.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/control_roles/support.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs
crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv
crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv
docs/dev/codex/brief-589-oracle-drain-control.md
```

The new `fixtures/control_roles/` directory has ten small modules:

| File | Responsibility |
| --- | --- |
| `mod.rs` | Explicit 51-entry registry and role dispatch. |
| `support.rs` | Exact input plans, five-operation sequences, explicit logs/state/prints/error expectations, caller-resumed and nested variants. |
| `loops.rs` | Iterator bodies and nested bounds; While/Until/For test, body, increment and initialization phases. |
| `conditions.rs` | Held logical operands with a decisive fifth operand and an unvisited suffix; branch tests and Switch matching. |
| `bodies.rs` | Bodies, confirmations, assertion callback, catch preparation, cleanup and random-state restoration. |
| `options_properties.rs` | CheckAbort's delayed default/immediate explicit option RHS; Failure's delayed parameters and message-template lookup. |
| `arithmetic.rs` | Upvalues reached only by the Plus expressions constructed inside Total/Mean/Median. |
| `phases.rs` | Eleven separately attached retained-owner/phase descriptors. |
| `lifecycle.rs` | First/second-pause cancellation, reset and pending rejection, with exact role-specific partial state. |
| `falsifiers.rs` | Missing-pause and lost-log rejection, weak controls and good controls for all 62 descriptors. |

The two parent fixture modules only register the new module/descriptors. Effects
changes are descriptor attachments only; owed changes are their matching removals.
The retention inventories contain replayed fingerprint/certificate cells only.
This report is appended to the current brief.

Baseline SHA-256 snapshots cover all 1,901 initial files in crates, docs, scripts
and the repository root. No baseline file disappeared. No production Rust,
`lib.rs`, Cargo manifest/lockfile/dependency, script, AGENTS/TODO/root PROGRESS,
other docs, historical suite, catalog disposition or shared checker changed.
No unsafe was introduced. All changed Rust files are below 10k characters;
every new module is below 5k. The existing fixture directory gains one coherent
family subdirectory; unrelated fixture grouping is preserved within the fence.
No performance gate applies to this test-only patch.

[Complete no-production-edit audit](../../../.temp/brief-589/audit.txt), [exact module sizes](../../../.temp/brief-589/final-source-sizes.json).

```text
Before 11; after 12; removed/renamed 0; added 1
oracle::fixtures::control_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log
All historical continuation suites and 209 historical labels unchanged.
```

The temporary `candidate_roles` test was archived to scratch and removed once
the descriptors were attached. It is not a shipped test-name change. All 209
historical labels remain unchanged.

### 2. Entries, variants and lifecycle

| Family | Target entries | Drained | Kept owed | Added descriptors | Added variants | Added lifecycle obligations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| control | 50 | 43 | 7 | 54 | 270 | 324 |
| logic | 5 | 4 | 1 | 4 | 20 | 24 |
| arithmetic | 3 | 3 | 0 | 3 | 15 | 18 |
| random | 1 | 1 | 0 | 1 | 5 | 6 |
| Total | 59 | 51 | 8 | 62 | 310 | 372 |

Every changed effects row is listed below. Its descriptor cell changes from `-`
to the value shown; all ten other cells remain identical. Exactly the same entry
IDs leave owed, with no other owed-row change.

| Entry drained | Attached descriptor IDs |
| --- | --- |
| `AbortProtect.body` | `AbortProtect.body` |
| `And.operand` | `And.operand` |
| `Assert.callback` | `Assert.callback`, `Assert.callback.resolve` |
| `Assert.test` | `Assert.test` |
| `BlockRandom.body` | `BlockRandom.body` |
| `Catch.body` | `Catch.body` |
| `Catch.pattern` | `Catch.pattern`, `Catch.pattern.match`, `Catch.pattern.bypassPreparation`, `Catch.pattern.bypassMatch` |
| `CatchExceptions.body` | `CatchExceptions.body` |
| `CatchExceptions.handler` | `CatchExceptions.handler`, `CatchExceptions.handler.pattern`, `CatchExceptions.handler.guard` |
| `CheckAbort.body` | `CheckAbort.body`, `CheckAbort.body.fallback`, `CheckAbort.body.protectedFallback` |
| `CheckAbort.defaultOption` | `CheckAbort.defaultOption` |
| `CheckAbort.explicitOption` | `CheckAbort.explicitOption` |
| `CompoundExpression.sequence` | `CompoundExpression.sequence` |
| `Confirm.test` | `Confirm.test` |
| `ConfirmAssert.test` | `ConfirmAssert.test` |
| `ConfirmBy.callback` | `ConfirmBy.callback` |
| `ConfirmBy.test` | `ConfirmBy.test` |
| `ConfirmMatch.pattern` | `ConfirmMatch.pattern` |
| `ConfirmMatch.test` | `ConfirmMatch.test` |
| `ConfirmQuiet.test` | `ConfirmQuiet.test` |
| `Do.body` | `Do.body` |
| `Do.bounds` | `Do.bounds` |
| `Enclose.body` | `Enclose.body` |
| `Failure.propertyParameters` | `Failure.propertyParameters`, `Failure.propertyParameters.styled` |
| `Failure.propertyTemplate` | `Failure.propertyTemplate`, `Failure.propertyTemplate.styled` |
| `For.body` | `For.body` |
| `For.increment` | `For.increment` |
| `For.initialization` | `For.initialization` |
| `For.test` | `For.test` |
| `If.test` | `If.test` |
| `Mean.arithmetic` | `Mean.arithmetic` |
| `Median.arithmetic` | `Median.arithmetic` |
| `Nand.operand` | `Nand.operand` |
| `Nor.operand` | `Nor.operand` |
| `Or.operand` | `Or.operand` |
| `Product.body` | `Product.body` |
| `Product.bounds` | `Product.bounds` |
| `Sum.body` | `Sum.body` |
| `Sum.bounds` | `Sum.bounds` |
| `Switch.pattern` | `Switch.pattern` |
| `Switch.test` | `Switch.test` |
| `Table.body` | `Table.body` |
| `Table.bounds` | `Table.bounds` |
| `Total.arithmetic` | `Total.arithmetic` |
| `Until.body` | `Until.body` |
| `Until.test` | `Until.test` |
| `Which.test` | `Which.test` |
| `While.body` | `While.body` |
| `While.test` | `While.test` |
| `WithCleanup.body` | `WithCleanup.body`, `WithCleanup.body.initialization` |
| `WithCleanup.cleanup` | `WithCleanup.cleanup` |

[Exact descriptor-cell changes](../../../.temp/brief-589/effect-row-changes.tsv); [all 51 complete removed owed rows](../../../.temp/brief-589/owed-rows-removed.tsv).

All 62 descriptors run Middle, Twice, Error, AlreadyResumed and Nested: **62 of
each, 310 total per build**. None of these effect rows requires Decline.
Middle pauses at operation 2; Twice at 2 and 4. The planned input is `Input`,
with the exact `<entry>:2`/`<entry>:4` prompts and numeric answers 2/4.
AlreadyResumed adds an earlier `InputString["control caller"]` answered
`caller resumed`; Nested executes under a Block scope restored to 17. Error
throws `"control boom"` with tag `"control tag"` after the first pause, at
operation 5, and checks the exact host error alongside queued/resumed equality.

Successful loop variants perform five body visits; bounds pause in nested bound evaluation on
outer iterations 2/4. Loop counters and iterator restoration to 91/92 are
asserted. Arithmetic pauses come from stable-symbol upvalues on the internally
constructed Plus, never eager list preparation. Confirm tests assert the
post-value failure/predicate/matcher/message handling. Random-state projections
compare the next five draws with a separately saved same-seed control, without
normalizing random leaves. Abort protection is checked after completion and
abandonment. WithCleanup runs its registered cleanup on cancellation, while
reset runs none; a suspended cleanup itself cannot run its remaining suffix.

Each descriptor exercises cancellation, reset and pending-request rejection at
both pauses: **62 × 6 = 372 added obligations**. Completion is checked by all
310 variants and every terminal path accepts a fresh request with an empty
queue. Total registry obligations rise from 726 to 1,098. Distinct phase
factories cover assertion hook resolution/application; normal and bypassing
Catch pattern preparation/matching; exception pattern/guard/RHS; ordinary and
protected CheckAbort fallbacks; cleanup initialization/body/cleanup; and both
plain/styled Failure property rendering.

Each descriptor's Middle fixture rejects a removed pause and a cleared log;
weak controls and its unmodified good control pass: **124 strict rejections,
124 weak controls and 62 good controls per build**.

[Every default variant/lifecycle/falsifier result](../../../.temp/brief-589/coverage-default.txt), [every no-default result](../../../.temp/brief-589/coverage-no-default.txt).

Kept owed, with the original manifest dispositions unchanged:

| Entry | Finding and witness |
| --- | --- |
| `Implies.operand` | Ordinary argument preparation, not a local operand evaluation. [Source witness](../../../.temp/brief-589/defects/Implies.operand.md) |
| `Failsafe.body` | Ordinary wrapper construction has no held body. Callable wrapper expansion constructs an expression for the shared evaluator. [Source witness](../../../.temp/brief-589/defects/Failsafe.body.md) |
| `CheckAbort.optionSyntax` | HoldAll with two positional arguments; args[2..] are parsed literally. Only the selected immediate Rule RHS is evaluated. No local option-syntax evaluation. [Source witness](../../../.temp/brief-589/defects/CheckAbort.optionSyntax.md) |
| `Catch.handler` | Frame policy mismatch: the handler is a direct tail evaluation; no Catch work remains after it. [Source witness](../../../.temp/brief-589/defects/Catch.handler.md) |
| `Enclose.handler` | Frame policy mismatch: a function handler is a direct tail evaluation; string handlers directly delegate to Failure projection. No Enclose handler frame is retained. [Source witness](../../../.temp/brief-589/defects/Enclose.handler.md) |
| `If.branch` | Frame policy mismatch: selected branch evaluation is a direct tail handoff, after condition evaluation has completed. [Source witness](../../../.temp/brief-589/defects/If.branch.md) |
| `Which.branch` | Frame policy mismatch: the selected branch is evaluated in tail position after condition selection. [Source witness](../../../.temp/brief-589/defects/Which.branch.md) |
| `Switch.branch` | Frame policy mismatch: selected branch is evaluated in tail position after pattern selection. [Source witness](../../../.temp/brief-589/defects/Switch.branch.md) |

The three absent local roles were checked against registration and argument
partition, not inferred from a failing fixture. The five Frame/Tail findings
are direct tail handoffs with no remaining local continuation work. No policy
was corrected in this brief. No semantic-defect witness was needed.

Verbatim before/after catalog evidence:

`catalog-before-default`:
```text
LIFECYCLE obligations=726 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=240 NoLocalGuestEffect=403 Boundary=4 entries=498 descriptors=125 historical=209 owed=373
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 3.16s
```

`catalog-before-no-default`:
```text
LIFECYCLE obligations=726 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=240 NoLocalGuestEffect=397 Boundary=4 entries=498 descriptors=125 historical=209 owed=373
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 3.17s
```

`oracle-final-default`:
```text
LIFECYCLE obligations=1098 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=240 NoLocalGuestEffect=403 Boundary=4 entries=498 descriptors=187 historical=209 owed=322
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.49s
```

`oracle-final-no-default`:
```text
LIFECYCLE obligations=1098 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=240 NoLocalGuestEffect=397 Boundary=4 entries=498 descriptors=187 historical=209 owed=322
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.45s
```

### 3. Verification and every maintenance inventory row

All Cargo commands ran serially through `python3 scripts/run-bounded.py`, with
`CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true` and outer
`TMPDIR=/home/remi/dev/rum/.temp/brief-589/tmp`. Oracle builds were bounded at
900 seconds each; runtime/ownership/fingerprint/certificate commands at 300
seconds; workspace at 3,000 seconds. The unmodified required oracle script
hard-codes its inner TMPDIR/logs to `.temp/suspension-oracle`; complete outputs
were copied to this brief's scratch directory. The existing fingerprint
regenerator uses `target/brief-544/local-contracts.tsv`; certificate output uses
the permitted `.temp/brief-546/certificates/`. No script was edited.

The first formatting check failed only on new test files; formatting was
applied and the final `cargo fmt --check` returned exit 0, with no diagnostics.
The candidate runs were green before attachment. Final oracle runs use
`scripts/check-suspension-oracle.sh 900`; their results appear above.

`cargo test -p rum-runtime`:

```text
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.95s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.69s
```

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`:

```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 59.69s
```

All seven ownership inventory files are byte-identical to baseline: no changed,
disappeared or added row, including no new UnknownConstruct, UnknownHostPayload
or self-reaching row. All other retention inventory files are also unchanged
except the two maintenance files disclosed here. No STOP condition occurred.

`cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`:

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.20s
```

All 68 local-contract rows change only the generated final fingerprint cell:

```text
id	before	after
B01	fnv1a64:a246410e6669c3d6	fnv1a64:a61a9e97847f035d
B02	fnv1a64:7f54a8ffbb1944b3	fnv1a64:37b50e1e0eb624f4
B03	fnv1a64:ef3a338f8538b9cc	fnv1a64:77c8ff145cb73567
B04	fnv1a64:7362d09842c9670e	fnv1a64:4505b35bc8b55155
B05	fnv1a64:b081d79b00b169df	fnv1a64:40f14a8a0edf7830
B06	fnv1a64:4374ef3a72623cea	fnv1a64:7db0af8e15e87e81
B07	fnv1a64:06e9dd63625f076f	fnv1a64:7f3818442f7d4ac0
B08	fnv1a64:1ef0845229f313b8	fnv1a64:2238e064760699d3
B09	fnv1a64:b14a1790699f7762	fnv1a64:3decd3b98a0b19f9
B10	fnv1a64:425f4be642988b31	fnv1a64:b27f9ed5b763cdb6
B11	fnv1a64:34ba81f55cbd8d0e	fnv1a64:065d64b8e2a97755
B12	fnv1a64:7056978c45a80c62	fnv1a64:fcf953b56613aef9
I01	fnv1a64:80282c0bd321ba56	fnv1a64:83fc8994f136f9dd
I02	fnv1a64:07c3552ef8e7f1df	fnv1a64:9832c81e07160030
I03	fnv1a64:7990f50bd88bce22	fnv1a64:eba11a35a369c5b9
I04	fnv1a64:a3eba58d978d0dd3	fnv1a64:0b348e6cdc209294
P01	fnv1a64:c73920268f862f8a	fnv1a64:e3c58b5f9cdece21
P02	fnv1a64:f091458d97bc4ae1	fnv1a64:8ec9032f9263b226
P03	fnv1a64:6af9fb7d9ddc7634	fnv1a64:12b78af8b6323daf
P04	fnv1a64:742fc7153fc260b3	fnv1a64:2c902c33935f40f4
P05	fnv1a64:f49b486e386352c6	fnv1a64:f3631ed54c2feccd
P06	fnv1a64:c0d5bc5cd1d7a5e2	fnv1a64:4d787885f2434879
P070	fnv1a64:fb23589eb04d1fe2	fnv1a64:87c614c7d0b8c279
P080	fnv1a64:3351c18ef445d6f1	fnv1a64:d9fe0102bdfda576
P090	fnv1a64:a291e3ec4f67de73	fnv1a64:1d39afabe7e4d3b4
P100	fnv1a64:7a003d40dd0f2b5e	fnv1a64:683934e392501f25
P110	fnv1a64:d0d1a35275f88e99	fnv1a64:e71bc265680cc2be
P120	fnv1a64:bb1bac41e5f6cfd8	fnv1a64:75fa29d88d2ce873
P130	fnv1a64:aab736c09112e16f	fnv1a64:230571a15e3124c0
P140	fnv1a64:80e1c1f9b8681a3b	fnv1a64:5345dfef137e16fc
P150	fnv1a64:65ecf7d893792493	fnv1a64:bda52755af862a54
P071	fnv1a64:110f8053c97d438f	fnv1a64:382622014e178160
P081	fnv1a64:2c1d746cd8279b02	fnv1a64:049cd753d8119b19
P091	fnv1a64:3c53b69786c41d4a	fnv1a64:838e3e346b43b4e1
P101	fnv1a64:c8d4941fab5c2b99	fnv1a64:df1eb3329d705fbe
P111	fnv1a64:e6e0d34eeaaec072	fnv1a64:b188a9381bccf309
P121	fnv1a64:cc137a48bc025505	fnv1a64:e487eee2c59df35a
P131	fnv1a64:110cf2fe791d888e	fnv1a64:e2afd5c1ff0972d5
P141	fnv1a64:5256d35a89aab6aa	fnv1a64:d2f85c1d6d487641
P151	fnv1a64:0d03da4798554018	fnv1a64:8443dfce974f2fb3
B13	fnv1a64:d18582c3a2fef8ea	fnv1a64:0bc1431746853a81
R01	fnv1a64:7bc7dfb7ecd70b43	fnv1a64:3e840681867a9c84
R02	fnv1a64:a217418595cf58f4	fnv1a64:62c2cac123a1006f
R03	fnv1a64:b0367ef4c40d7847	fnv1a64:c32718140f506818
R040	fnv1a64:a900e5121fe6cde3	fnv1a64:e2821652dc1d87a4
R050	fnv1a64:9e2b523a5916aba6	fnv1a64:4db2d728d85e112d
R060	fnv1a64:8f94fc6f18eb2d8b	fnv1a64:66f9e04cffc7ce4c
R070	fnv1a64:948b3c86b9c95a02	fnv1a64:6d0a9f6db9b35a19
R080	fnv1a64:6af00e806711d6d3	fnv1a64:d238f75faba55b94
R090	fnv1a64:aa78c37899bc591e	fnv1a64:1ac9e7c4b7f4b5e5
R100	fnv1a64:b45fd7e22f0e644c	fnv1a64:3ceea367068cdfe7
R110	fnv1a64:c0bf834a24e4fe29	fnv1a64:3759fd102de3af4e
R041	fnv1a64:b7bee5edba26b7ea	fnv1a64:f1faa6415dacf981
R051	fnv1a64:30b41201dc95eaf3	fnv1a64:ab5bddc17512e034
R061	fnv1a64:cf3658f588f8df68	fnv1a64:9f715413d4cf9b43
R071	fnv1a64:49b68a58bed7455d	fnv1a64:b1ea249056237ef2
R081	fnv1a64:35eaa69578bb005e	fnv1a64:24239e382dfbf425
R091	fnv1a64:bf6dfd12fd17f57b	fnv1a64:abc7feb8179a173c
R101	fnv1a64:10afafd02f80914b	fnv1a64:d2a7f97faac9ee0c
R111	fnv1a64:10ee2317406d5f10	fnv1a64:16b4116d45d29eeb
R12	fnv1a64:a44163135fc55c50	fnv1a64:7434f5679cfa5c2b
R13	fnv1a64:3bcccfa5e7046e3c	fnv1a64:30b3c53b67e5ddd7
R14	fnv1a64:94584bb6b9bfeb5e	fnv1a64:829143596f00df25
R15	fnv1a64:8438ac1a83acda1a	fnv1a64:8a9850d6168c2c71
O010	fnv1a64:615f347546c34fc8	fnv1a64:a22f76b9011248a3
O0200	fnv1a64:7ba963ce071f403b	fnv1a64:4e0d81c362353cfc
O011	fnv1a64:d21db8f7267cf20b	fnv1a64:a9829cd50d5992cc
O0210	fnv1a64:81882e47a1326fb0	fnv1a64:954ed07af9fc4f8b
```

[Exact fingerprint before/after TSV](../../../.temp/brief-589/fingerprint-changes.tsv).

The first certificate replay in each feature build returned the expected stale
certificate failure and emitted freshly earned review cells. Verbatim:

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 17.83s
```

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.83s
```

Only the hash=true/hash=false certificate cells in promotion rows C01 and C02
were copied from generated output. No identity, status, source, precondition,
strength, exclusion, instance/build/body/domain fingerprint or instance mark
changed. These cells differ only in embedded direct-contract fingerprints:

```text
body	feature	changed embedded contracts	unchanged instance marks
C01	true	O010,O011,O0200,O0210	fnv1a64:db543eaa6df76620,fnv1a64:c6cb017dd1358c2a
C01	false	O010,O011,O0200,O0210	fnv1a64:507d9ba1e842e023,fnv1a64:f5a09329759e61a5
C02	true	B05,B13,I01,I02,I04,R01,R02	fnv1a64:ce01acda3c73dc98
C02	false	B05,B13,I01,I02,I04,R01,R02	fnv1a64:479e00b3640f75f0
```

[All four full before/after certificate cells](../../../.temp/brief-589/certificate-changes.tsv).

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` after replay:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.11s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` after replay:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.88s
```

Workspace command: `cargo test --workspace --no-fail-fast -- --nocapture`, with
supported `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` to permit a native skip only when
loopback binding is denied. Verbatim harness/result lines:

```text
Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.25s
Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.33s
Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.11s
Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.96s
Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1046.93s
Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.37s
Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.43s
Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.51s
Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.10s
Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.45s
Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Workspace exit 0. Across 49 result blocks: 3364 reported passed, 0 failed, 58 ignored. **9 reported passes are native ZeroMQ permission skips, not executed native coverage.** Verbatim skips:

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

[Full workspace output](../../../.temp/brief-589/workspace.txt), [result lines](../../../.temp/brief-589/workspace-results.txt), [native skips](../../../.temp/brief-589/native-skips.txt).

Other complete logs: [fmt-final](../../../.temp/brief-589/fmt-final.txt), [oracle-final](../../../.temp/brief-589/oracle-final.txt), [runtime](../../../.temp/brief-589/runtime.txt), [ownership](../../../.temp/brief-589/ownership.txt), [fingerprints](../../../.temp/brief-589/fingerprints.txt), [retention-default](../../../.temp/brief-589/retention-default.txt), [retention-no-default](../../../.temp/brief-589/retention-no-default.txt), [maintenance](../../../.temp/brief-589/maintenance.txt). Exact command arrays and exit codes are saved beside each log.

No mandatory verification command was omitted. The two expected initial stale
certificate failures were resolved exclusively by generated cells. No failing
test remains, no correct descriptor was weakened, and no production repair was
attempted. Required inventory cells are current with the final Rust source.

### 4. Stop-loss counters and open questions

Final counters per feature build: **51 entries drained, 310 added variants
passing, 372 added lifecycle obligations passing, eight disposition findings,
zero semantic defects**. Falsifiers add 124 strict rejections, 124 weak controls
and 62 good controls. Candidate milestones advanced 0 -> 255 -> 310 variants
and 0 -> 306 -> 372 lifecycle obligations; attachment advanced drained 0 -> 51.
The final assertion-strengthening pass preserved these totals. No two
consecutive compiling implementation attempts failed to advance the counters;
the largest final no-progress streak is one. The remaining commands were
required verification/maintenance, not further implementation attempts.

The director must decide the eight unchanged manifest dispositions before those
entries can be drained. The starting owed count differs from the brief narrative
as disclosed above. No other open implementation question remains. The executing
agent leaves all changes uncommitted for the director's gate.
