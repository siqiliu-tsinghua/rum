# brief-598 — E1 drain 8 of 8: `associations`, `associations_transform`, `schema`, `evaluator`, and the snapshot-restore boundary (38 owed entries)

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) and left **500** declared
guest-effect entries in an exact owed ledger (`inventory/owed.tsv`). Drains 1-7
(briefs 587-592, 596) took eighteen families; the director removed the declared
roles that do not exist at each gate; drain 6 found group G11 (repaired by
593-595) and drain 7 found G12 (repaired by 597). **Owed is now 38. This brief
drains all of them:**

- **`associations` (13)** — exactly the IDs in
  `.temp/brief-586/followups/draft-E1-02-associations.md`: `Association.keyValue`,
  `KeyGroupBy.callback`, `KeyMap.callback`, `KeySelect.predicate`,
  `KeySort.comparator`, `KeySortBy.keyFunction`, `KeyValueMap.callback`,
  `Lookup.default`, `Lookup.delayedValue`, `Lookup.heldTarget`, `Lookup.key`,
  `Lookup.wrapper`, `Values.delayedValue`.
- **`associations_transform` (19)** — exactly the IDs in
  `draft-E1-03-associations_transform.md`: `AssociateTo.target`,
  `AssociationMap.callback`, `AssociationThread.delayedValue`,
  `Catenate.delayedValue`, `Counts.delayedValue`, `CountsBy.keyFunction`,
  `DeleteMissing.{defaultOption,explicitOption,optionSyntax}`,
  `GroupBy.{callback,reducer}`, `KeyDropFrom.target`, `KeyUnion.missingValue`,
  `Merge.{delayedValue,reducer}`, `MergeNested.{delayedValue,reducer}`,
  `Normal.{delayedValue,lazyConsumer}`.
- **`schema` (4)** — `ConformSchema.{default,predicate}`,
  `SchemaErrors.{default,predicate}` (`draft-E1-14-schema.md`).
- **`evaluator` (1)** — `Evaluator.defaultFallback`, owned by
  `try_apply_direct_default_value`
  (`crates/rum-runtime/src/runtime/definitions/application/default_values.rs`).
  It is not an installed `BuiltinSpec`; brief-585's G10 witnesses are
  supplemental only. Give it a complete role descriptor of its own.
- **`scoping` (1)** — `RestoreSealedReferences.body`, by the **Boundary**
  disposition below.

Each entry leaves the ledger **only in the same change that attaches and runs
its complete descriptor**: every required variant (`Middle`, `Twice`, and the
`Decline`, `Error`, `AlreadyResumed`, `Nested` variants its effect row
records), lifecycle at the first and second pause for every distinct
continuation owner (completion, cancellation, reset, a pending request
rejecting a new one), and a fresh request after.

The drafts are raw material; **this brief governs** where they differ. In
particular, **draft 22's closing step is not in this brief**: do not delete
`owed.tsv`, `reconcile_owed`, its error variants, its falsifiers or the
nonempty-ledger check. The director closes the mechanism in a separate brief
once owed is zero in both builds.

## `RestoreSealedReferences.body` is a documented boundary

brief-592 established (`.temp/brief-592/defects/RestoreSealedReferences.body.md`)
that this role is admitted only while restoring a snapshot (`packages.rs:336`),
and that snapshot restore is a host batch API that is non-resumable by user
ruling. The contract is `docs/guide/embedding.md` §Snapshot And Restore: if
snapshot evaluation requests input, `restore_from_bytes` returns an `Other`
failure after abandoning that restore's suspension exactly once; cleanup runs
inner to outer; the prompt, continuation and request identity are retired;
definitions installed before the failure remain; and the session accepts a
fresh evaluation without a second reset.

Its disposition is `Boundary`. **You are authorized to change exactly this
row's variant column in `inventory/effects.tsv` to `RetiredMiddle,RetiredTwice`**
(as the four existing boundary rows have) and to attach a boundary descriptor,
on the model of `fixtures/boundaries.rs`, driven through the real public
`restore_from_bytes` on a snapshot image containing the role. It must show,
in both variants:

1. the body's input request **actually occurred** inside the restore — observed
   as the child's `InputRequired` with its kind and prompt, not inferred from
   the failure (a restore can also fail for other reasons, such as the mode
   diagnostic outside restore; the witness must exclude them);
2. the documented `Other` failure, with the prefix effects before the request
   present and the suffix after it absent;
3. no pending owner afterwards: no input request is pending, a resume attempt
   is rejected, and a fresh evaluation succeeds without a reset;
4. `RetiredTwice`: a body with two requests never reaches the second.

If the public restore cannot observe item 1, report why with evidence and keep
the entry owed; do not weaken the item.

## A failing descriptor is a finding, not a test to fix

If a correct descriptor shows the resumed path differing from the queued one —
a lost continuation the census did not see — **do not weaken the descriptor,
do not repair production code, and do not commit a failing test.** Keep the
entry owed, write the witness (source, answers, both observations) to
`.temp/brief-598/defects/<entry>.md`, and report it. It becomes a repair group
of this branch. **A role that does not exist** (an `Ordinary` builtin's option
or operand evaluated by shared preparation, a positional argument declared as
an option, option syntax admitted only literally) is reported with source
evidence and kept owed; the director removes it at the gate. Most removals so
far were `optionSyntax` on `Ordinary` builtins, where a computed option
expression is evaluated by shared preparation before the handler sees it (a
delayed rule's rhs *is* read locally: that is `explicitOption`). Check the
registration (`Ordinary` or held) and the argument partition first. **A wrong
policy is not a reason to keep an entry owed**: the oracle's equality does not
depend on whether the entry is `Frame` or `Tail`, so attach the descriptor and
report the correct policy for the director to set.

## Context pointers

- briefs 586-597 and their reports; the four drafts above; the descriptors
  under `fixtures/*_roles/`, `fixtures/rules/`, `fixtures/capture_output/` and
  `fixtures/boundaries.rs` as the model.
- `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors,catalog}.rs`,
  `support/`, `fixtures/`, `inventory/{catalog,effects,shared-effects,owed,historical-labels}.tsv`.
  The runner checks the print, message and raw trace prefix at every pause and
  compares the final raw trace array; a descriptor declares its exact per-pause
  expectations.
- The owning functions named in the drafts (read their retained state and every
  resumption phase); the guide (`structured-data.md`, `patterns-and-rules.md`,
  `embedding.md`) and the generated reference are the contract.
- The association callbacks, reducers and comparators run inside the builtin's
  own loop: pause after earlier elements have been processed. A delayed value
  (`RuleDelayed` rhs) is read locally; a `Lookup` default and wrapper are
  separate phases from the key. Schema predicates and defaults run inside the
  flat validator stack (G3, brief-563); pause below the root.

## Scope

**Edit surface:** the `suspension_oracle` test tree; the existing
continuation suites only to share descriptors (test names and specialized
assertions stay); in `inventory/effects.tsv`, only descriptor attachment and
the one variant change authorized above.

**Standing duty — always in scope** (`TEMPLATE.md`; test edits under `crates/`
expire the fingerprints): fingerprints, both certificates, ownership
reconciliation, every row disclosed. **STOP** on any existing row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code.

**Fence:** no production `.rs`, manifest disposition, `lib.rs`, Cargo
manifest, lockfile or dependency change; no ledger-mechanism deletion; no
other family's entries; no edits to `AGENTS.md`, `TODO.md`, root
`PROGRESS.md`, earlier briefs, `scripts/` or docs.

**Disclosure duty:** every entry drained, every entry kept owed and why, every
defect witness, every changed test name, every inventory row.

## Deliverable

Role-specific descriptor factories with explicit expected values, logs and
state projections and exact input kind, prompt and answer plans. A callback,
guard, option, reducer, comparator or held-target phase is exercised by its
own role — ordinary argument preparation never stands in. Loops run at least
five operations and pause in the middle and at a second position. Shared
factories across the family are welcome where they demonstrably exercise each
entry's role.

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
TMPDIR=<repo>/.temp/brief-598/tmp …`. Bounds: the oracle target 900 s per
build; runtime, ownership and retention 300 s; `cargo test --workspace
--no-fail-fast` 3 600 s.

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
count, and `Boundary` unchanged at 5, in both builds. Report native ZeroMQ
skips as skips.

## Checkpointing — protect the scene

Everything under `.temp/brief-598/`; `PROGRESS.md` after each milestone with
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
3. Defect witnesses, if any; nonexistent roles with source evidence; policy
   corrections; the broad suites verbatim; every inventory row change.
4. Stop-loss counters, open questions.

---

## Execution report — brief-598

### 1. Files changed and no-production-edit audit

33 entries are drained; 5 nonexistent declarations remain owed for the director. The strict oracle passes in both builds. No production continuation defect was found.

No git command or network access was used. No production Rust, Cargo manifest, lockfile, dependency, catalog disposition, script, AGENTS.md, TODO.md, root PROGRESS.md, or earlier brief changed. All changes remain uncommitted. The initial SHA-256 index and preserved inputs provide the audit independently of version control: [file-audit.json](../../../.temp/brief-598/file-audit.json), [changes.diff](../../../.temp/brief-598/changes.diff).

Existing files changed:

- `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` — replay all 68 fingerprint cells.
- `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` — install the four generated C01/C02 build certificate cells.
- `crates/rum-stdlib/tests/suspension_oracle.rs` — test-only aliases for the unchanged public host facade.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` — register the role and restore fixture modules.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs` — attach the 58 executable descriptors.
- `crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv` — 32 descriptor-attachment rows; the authorized restore variant change.
- `crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv` — remove exactly the 33 fully exercised entries.
- `crates/rum-stdlib/tests/suspension_oracle/inventory/shared-effects.tsv` — attach Evaluator.defaultFallback and its three owner/phase descriptors.
- `docs/dev/codex/brief-598-oracle-drain-associations-schema-evaluator.md` — append this execution report.

New test files (all support files under 10k characters; all scenario files under 15k):

- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/aggregate.rs` (2,967 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/evaluator.rs` (1,717 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/falsifiers.rs` (2,425 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/grouping.rs` (4,134 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/held.rs` (1,886 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/lifecycle.rs` (1,604 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/lookup.rs` (2,114 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/mapping.rs` (5,434 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/mod.rs` (6,122 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/schema.rs` (2,112 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/associations_roles/support.rs` (5,406 characters).
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/restore_boundary.rs` (6,304 characters).
- `crates/rum-stdlib/tests/suspension_oracle/host_facade.rs` (963 characters).

This brief is the only documentation file changed. The new modules split by callback/mapping, aggregation, grouping, held targets, lookup phases, schemas, direct defaults, lifecycle, and falsifiers. Existing scenario modules and all 209 historical witnesses remain intact. No performance gate applies to this test-only change.

`host_facade.rs` compiles the unchanged host source modules into the oracle test crate. That lets the boundary invoke the real public `HostSession::restore_from_bytes` without a forbidden Cargo graph change or a copied restore implementation. The adapter depends on the host module layout; a future host source reorganization may require updating these test-only paths. Its six existing embedded unit tests also run under the test crate. No host implementation or legacy suite was edited.

### 2. Entries, variants, lifecycle and owed counts

| Build | Initial owed ledger | Final owed | Boundary | Installed | Descriptors | Lifecycle obligations |
|---|---:|---:|---:|---:|---:|---:|
| default | 38 | 5 | 5 | 643 | 546 | 4,026 |
| no-default-features | 38 | 5 | 5 | 637 | 546 | 4,026 |

The before count is the saved, feature-independent initial ledger; the default baseline was also executed (42 tests passed). Both final counts come from the mandatory script’s actual CATALOG lines. Entries remain 443 and historical witnesses remain 209. The ledger mechanism, reconciliation errors, falsifiers and nonempty-ledger assertion are unchanged.

Added: **58 descriptors, 293 passing variants, 684 lifecycle obligations**, plus completion for every variant. There are 57 resumable owner/phase descriptors and one two-variant retirement descriptor. The 57 resumable descriptors each reject a missing pause and lost log (114 rejections), with weak controls and restored good controls. Exact matrix: [descriptor-matrix.tsv](../../../.temp/brief-598/descriptor-matrix.tsv).

Drain by family: associations 13/13; associations_transform 14/19; schema 4/4; evaluator 1/1; scoping 1/1. Every attached descriptor and removed owed row is disclosed below; all other effect columns are unchanged except the expressly authorized restore variant column.

| Drained entry | Attached descriptor IDs |
|---|---|
| `AssociateTo.target` | `AssociateTo.target`, `AssociateTo.target.read`, `AssociateTo.target.part` |
| `Association.keyValue` | `Association.keyValue` |
| `AssociationMap.callback` | `AssociationMap.callback`, `AssociationMap.callback.association` |
| `Catenate.delayedValue` | `Catenate.delayedValue`, `Catenate.delayedValue.list` |
| `ConformSchema.default` | `ConformSchema.default` |
| `ConformSchema.predicate` | `ConformSchema.predicate` |
| `Counts.delayedValue` | `Counts.delayedValue` |
| `CountsBy.keyFunction` | `CountsBy.keyFunction` |
| `Evaluator.defaultFallback` | `Evaluator.defaultFallback`, `Evaluator.defaultFallback.cross`, `Evaluator.defaultFallback.body` |
| `GroupBy.callback` | `GroupBy.callback`, `GroupBy.callback.value`, `GroupBy.callback.cross`, `GroupBy.callback.nested`, `GroupBy.callback.association`, `GroupBy.callback.associationValue`, `GroupBy.callback.associationCross`, `GroupBy.callback.associationNested` |
| `GroupBy.reducer` | `GroupBy.reducer`, `GroupBy.reducer.nested`, `GroupBy.reducer.leaf`, `GroupBy.reducer.association`, `GroupBy.reducer.associationNested`, `GroupBy.reducer.associationLeaf` |
| `KeyDropFrom.target` | `KeyDropFrom.target`, `KeyDropFrom.target.read`, `KeyDropFrom.target.part` |
| `KeyGroupBy.callback` | `KeyGroupBy.callback`, `KeyGroupBy.callback.reducer` |
| `KeyMap.callback` | `KeyMap.callback` |
| `KeySelect.predicate` | `KeySelect.predicate` |
| `KeySort.comparator` | `KeySort.comparator` |
| `KeySortBy.keyFunction` | `KeySortBy.keyFunction`, `KeySortBy.keyFunction.ordering` |
| `KeyUnion.missingValue` | `KeyUnion.missingValue` |
| `KeyValueMap.callback` | `KeyValueMap.callback` |
| `Lookup.default` | `Lookup.default` |
| `Lookup.delayedValue` | `Lookup.delayedValue` |
| `Lookup.heldTarget` | `Lookup.heldTarget` |
| `Lookup.key` | `Lookup.key` |
| `Lookup.wrapper` | `Lookup.wrapper`, `Lookup.wrapper.afterDefault` |
| `Merge.delayedValue` | `Merge.delayedValue` |
| `Merge.reducer` | `Merge.reducer` |
| `MergeNested.delayedValue` | `MergeNested.delayedValue`, `MergeNested.delayedValue.deep` |
| `MergeNested.reducer` | `MergeNested.reducer`, `MergeNested.reducer.deep` |
| `Normal.lazyConsumer` | `Normal.lazyConsumer` |
| `RestoreSealedReferences.body` | `RestoreSealedReferences.body` |
| `SchemaErrors.default` | `SchemaErrors.default` |
| `SchemaErrors.predicate` | `SchemaErrors.predicate` |
| `Values.delayedValue` | `Values.delayedValue` |

The distinct-owner cases include list/association mapping and grouping, key/value and post-key grouping phases, nested and leaf reducers, both Catenate paths, shallow and iterative deep MergeNested paths, KeySortBy with custom ordering, prepared/read/Part mutation targets, the wrapper entered after a suspending Lookup default, and direct-guard/body/post-decline default fallback. Every resumable descriptor checks exact input kind/prompt/answers, print/message/raw-trace prefixes, result/error, logs and state projections. Cancellation, reset, overlapping-request rejection and fresh request are exercised at both designated pauses. Sorting also compares the complete pair transcript without inventing a fixed comparison schedule. Schema callbacks/defaults pause below the root.

The restore descriptor observes the child’s actual `InputRequired` (`InputString`, `restore:2`) before retirement. It checks the documented `Other` failure, retained definitions and prefix, absent suffix, inner-to-outer cleanup exactly once, no pending request, rejected resume, a fresh evaluation without reset, and a fresh suspension immune to the retired restore cancel handle. `RetiredTwice` contains a second `Input` that is never reached.

### 3. Findings, verification and every inventory row

No continuation-mismatch witness was produced. Five nonexistent-role findings remain owed; their effect rows are untouched.

- `AssociationThread.delayedValue` — `eval_association_thread` parses and renders entry forms preserving the delayed flag. A RuleDelayed RHS is copied into returned delayed rules without evaluation; a later Values/Lookup consumer owns any eventual read. Source evidence: crates/rum-stdlib/src/associations/transforms.rs:127-224. No effect/owed row changed for this entry.
 [AssociationThread.delayedValue.md](../../../.temp/brief-598/defects/AssociationThread.delayedValue.md)
- `DeleteMissing.defaultOption` — No default option reader exists. `eval_delete_missing` parses only one to three positional operands; defaults for depth are Rust constants. Source evidence: crates/rum-stdlib/src/associations/missing.rs:24-77; crates/rum-stdlib/src/catalog/associations_transform.rs:345-349. No effect/owed row changed for this entry.
 [DeleteMissing.defaultOption.md](../../../.temp/brief-598/defects/DeleteMissing.defaultOption.md)
- `DeleteMissing.explicitOption` — The second and third operands are positive depth / non-negative scan depth, not options. No delayed Rule rhs is read as an option. Source evidence: crates/rum-stdlib/src/associations/missing.rs:24-77; crates/rum-stdlib/src/catalog/associations_transform.rs:345-349. No effect/owed row changed for this entry.
 [DeleteMissing.explicitOption.md](../../../.temp/brief-598/defects/DeleteMissing.explicitOption.md)
- `DeleteMissing.optionSyntax` — The builtin is Ordinary; shared preparation owns computed operands. The handler accepts no option syntax. Source evidence: crates/rum-stdlib/src/associations/missing.rs:24-77; crates/rum-stdlib/src/catalog/associations_transform.rs:345-349. No effect/owed row changed for this entry.
 [DeleteMissing.optionSyntax.md](../../../.temp/brief-598/defects/DeleteMissing.optionSyntax.md)
- `Normal.delayedValue` — `normalize_association_expr` returns a List of rules preserving their heads; RuleDelayed RHS stays held. Only lazy realization is a guest evaluation in eval_normal. Source evidence: crates/rum-stdlib/src/associations/basic.rs:217-234,276-290. No effect/owed row changed for this entry.
 [Normal.delayedValue.md](../../../.temp/brief-598/defects/Normal.delayedValue.md)

Policy corrections for the director (not applied): `KeyMap.callback`, `KeyValueMap.callback`, `Lookup.delayedValue`, and `Values.delayedValue` should be `Tail`, because their handlers return callback/value expressions for subsequent result evaluation. The complete strict descriptors pass under the current declarations. Source evidence: [policy-corrections.tsv](../../../.temp/brief-598/policy-corrections.tsv).

Test names: all 42 baseline oracle names preserved; none renamed or removed. The new names are listed below. No existing continuation suite changed. Exact sets: [test-name-changes.json](../../../.temp/brief-598/test-name-changes.json).

- `host_facade::file_io::tests::normalize_path_falls_back_to_current_dir_for_missing_relative_paths`
- `host_facade::file_io::tests::std_file_io_converts_read_and_write_errors`
- `host_facade::file_io::tests::std_file_io_lists_only_immediate_directory_entries`
- `host_facade::file_io::tests::std_file_io_probe_distinguishes_files_directories_and_missing_paths`
- `host_facade::model::outcome::tests::failure_code_maps_runtime_request_errors_to_host_codes`
- `host_facade::session::tests::panic_is_complete_response_marks_session_poisoned_and_invalid`
- `oracle::fixtures::associations_roles::falsifiers::every_owner_rejects_missing_pause_and_lost_log`

All Cargo gates ran serially through `scripts/run-bounded.py`, with `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true` and this brief’s TMPDIR. Each command has a `.command.json` file and a full verbatim `.log`. The oracle script and bounded runner were copied byte-for-byte into `oracle-runner/scripts/` to keep the script’s fixed scratch location under this brief; its tmp directory links to `.temp/brief-598/tmp`. Repository scripts were not edited. Final build artifacts use `.temp/brief-598/target-tests`.

Verbatim mandatory-gate summaries (complete output in the linked logs):

`cargo fmt --check` — exit 0, empty stdout/stderr. [format-check.log](../../../.temp/brief-598/format-check.log)

`scripts/check-suspension-oracle.sh 900` — [oracle.log](../../../.temp/brief-598/oracle.log)

```text
     Running tests/suspension_oracle.rs (/home/remi/dev/rum/.temp/brief-598/target-tests/debug/deps/suspension_oracle-08d116f9343550b3)
LIFECYCLE obligations=4026 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=237 NoLocalGuestEffect=406 Boundary=5 entries=443 descriptors=546 historical=209 owed=5
OWED families={"associations_transform": 5}
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.13s
     Running tests/suspension_oracle.rs (/home/remi/dev/rum/.temp/brief-598/target-tests/debug/deps/suspension_oracle-a9ca1e531e95228c)
LIFECYCLE obligations=4026 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=237 NoLocalGuestEffect=400 Boundary=5 entries=443 descriptors=546 historical=209 owed=5
OWED families={"associations_transform": 5}
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.23s
```

`cargo test -p rum-runtime` — [runtime.log](../../../.temp/brief-598/runtime.log)

```text
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_runtime-95cd5fbb46da4f1c)
test result: ok. 1149 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.66s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.37s
```

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` — [ownership-discovery.log](../../../.temp/brief-598/ownership-discovery.log)

```text
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_stdlib-3a556b9db7fd59ac)
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7680 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 63.29s
```

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` — [certificates-true-green.log](../../../.temp/brief-598/certificates-true-green.log)

```text
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_stdlib-3a556b9db7fd59ac)
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.42s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` — [certificates-false-green.log](../../../.temp/brief-598/certificates-false-green.log)

```text
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_stdlib-5d6fedc676d25fda)
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.71s
```

`env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast` — [workspace.log](../../../.temp/brief-598/workspace.log)

```text
     Running unittests src/main.rs (.temp/brief-598/target-tests/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/cli.rs (.temp/brief-598/target-tests/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/input_requests.rs (.temp/brief-598/target-tests/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/terminal.rs (.temp/brief-598/target-tests/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (.temp/brief-598/target-tests/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (.temp/brief-598/target-tests/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
     Running tests/adapter_contract.rs (.temp/brief-598/target-tests/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (.temp/brief-598/target-tests/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/capture_output_lifecycle.rs (.temp/brief-598/target-tests/debug/deps/capture_output_lifecycle-c541c492bd454062)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (.temp/brief-598/target-tests/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/host_payload_destruction_contract.rs (.temp/brief-598/target-tests/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.14s
     Running tests/installer.rs (.temp/brief-598/target-tests/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (.temp/brief-598/target-tests/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (.temp/brief-598/target-tests/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/random_password_policy.rs (.temp/brief-598/target-tests/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/shared_accumulator_completion.rs (.temp/brief-598/target-tests/debug/deps/shared_accumulator_completion-067001aa2ecf9629)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/stress.rs (.temp/brief-598/target-tests/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running tests/suspended_timeout_contract.rs (.temp/brief-598/target-tests/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.09s
     Running tests/suspension_abandonment_contract.rs (.temp/brief-598/target-tests/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
     Running unittests src/bin/rum-jupyter-kernel.rs (.temp/brief-598/target-tests/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
     Running tests/input_abandonment.rs (.temp/brief-598/target-tests/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/input_continuation.rs (.temp/brief-598/target-tests/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1149 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.52s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1083.43s
     Running unittests src/bin/generate-builtin-reference.rs (.temp/brief-598/target-tests/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (.temp/brief-598/target-tests/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/destructuring_continuations.rs (.temp/brief-598/target-tests/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (.temp/brief-598/target-tests/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
     Running tests/lazy_pull_continuations.rs (.temp/brief-598/target-tests/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.45s
     Running tests/loop_continuations.rs (.temp/brief-598/target-tests/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/option_reader_continuations.rs (.temp/brief-598/target-tests/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
     Running tests/rule_pattern_continuations.rs (.temp/brief-598/target-tests/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
     Running tests/schema_continuations.rs (.temp/brief-598/target-tests/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.16s
     Running tests/sow_trace_continuations.rs (.temp/brief-598/target-tests/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
     Running tests/suspension_oracle.rs (.temp/brief-598/target-tests/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.92s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.80s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.90s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter --lib zeromq::tests::native -- --nocapture` — [native-skips.log](../../../.temp/brief-598/native-skips.log)

```text
     Running unittests src/lib.rs (.temp/brief-598/target-tests/debug/deps/rum_jupyter-dd24b36d3edaa47a)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s
```

Native ZeroMQ: **9 skipped**, not exercised network tests. Loopback socket binding is denied in this sandbox; the existing explicit skip flag was used. Cargo labels these early-returning tests `ok`; the actual skip output is:

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

Maintenance evidence: ownership discovery reconciles exactly with no changed, added, or removed row: self_reaching 59; host_payloads 65; unknown_attributes 1,499; unknown_captures 2,946; unknown_macros 3,144; unknown_storage 91; host_payload_ledger 65. No new UnknownHostPayload, self-reaching or untouched-code row occurred. [ownership-row-changes.json](../../../.temp/brief-598/ownership-row-changes.json)

Effect/owed changes: exactly the 33 attachments in the table above (32 in `effects.tsv`, one in `shared-effects.tsv`) and their same 33 owed-row removals. `RestoreSealedReferences.body` alone changes variants from `Middle,Twice,Error,AlreadyResumed,Nested` to `RetiredMiddle,RetiredTwice`. No policy, family, owner, disposition or other variant field changed. Full before/after rows: [oracle-row-changes.json](../../../.temp/brief-598/oracle-row-changes.json).

Contract fingerprints were recomputed by replaying the exact Rust length-delimited FNV algorithm, first proving replay against the preserved pre-edit inputs. All 68 changes are only the final fingerprint cell; identities, statuses, guarantees and exclusions are byte-identical.

| Row | Contract | Old fingerprint | Replayed fingerprint |
|---:|---|---|---|
| 2 | B01 | `fnv1a64:bdcbd1dac1f4a643` | `fnv1a64:a802cf752c0f8b99` |
| 3 | B02 | `fnv1a64:eac2fc49de6159e6` | `fnv1a64:45d09df5fb1a571c` |
| 4 | B03 | `fnv1a64:e6f23dfd0c9360ad` | `fnv1a64:651737748a60467b` |
| 5 | B04 | `fnv1a64:b54b2cae25b1861b` | `fnv1a64:bc1b7ce26e329e41` |
| 6 | B05 | `fnv1a64:d347763153db882a` | `fnv1a64:3e109306fd7ee168` |
| 7 | B06 | `fnv1a64:5f51f3e530b3b9cf` | `fnv1a64:be92836b1f02310d` |
| 8 | B07 | `fnv1a64:ea121800fb8e641a` | `fnv1a64:fc87b408ac162a78` |
| 9 | B08 | `fnv1a64:5c60e4337c218731` | `fnv1a64:85c02de849876a67` |
| 10 | B09 | `fnv1a64:beb7e2f859f9be07` | `fnv1a64:45e59fdc66beaeb5` |
| 11 | B10 | `fnv1a64:c3ea573c443c1618` | `fnv1a64:bd4ce8bf7ad15e8e` |
| 12 | B11 | `fnv1a64:76a2de0b3fa5ac1b` | `fnv1a64:7d732e3f8826c441` |
| 13 | B12 | `fnv1a64:7dc462f436025307` | `fnv1a64:04f21fd842c743b5` |
| 14 | I01 | `fnv1a64:9badbcd82eac9cc3` | `fnv1a64:85e4ba7298c78219` |
| 15 | I02 | `fnv1a64:2a88f3c54c12102a` | `fnv1a64:9552109af5b56968` |
| 16 | I03 | `fnv1a64:f88c025ea5778547` | `fnv1a64:68d36839d0c51475` |
| 17 | I04 | `fnv1a64:7e55549209d1bd86` | `fnv1a64:ec35f55d4ff17ebc` |
| 18 | P01 | `fnv1a64:2ab6a63c96cbbc6f` | `fnv1a64:17630c64aa246b2d` |
| 19 | P02 | `fnv1a64:12e9fa5b39e984e8` | `fnv1a64:97d305ee1d76a73e` |
| 20 | P03 | `fnv1a64:a2475cbe9907d5d5` | `fnv1a64:37c9d9e97a66ab13` |
| 21 | P04 | `fnv1a64:df9e1a5f630a75e6` | `fnv1a64:3aabbc0b7fc3731c` |
| 22 | P05 | `fnv1a64:c11046a3f88e1cd3` | `fnv1a64:32c2eeafe7dba789` |
| 23 | P06 | `fnv1a64:ce4387c4c231ec87` | `fnv1a64:557144a8cef6dd35` |
| 24 | P070 | `fnv1a64:08912406a0a76687` | `fnv1a64:8fbee0eaad6c5735` |
| 25 | P080 | `fnv1a64:7a21af2fbe1c3858` | `fnv1a64:021c1464f524004e` |
| 26 | P090 | `fnv1a64:f2875e4256f4d426` | `fnv1a64:df60269270ab5adc` |
| 27 | P100 | `fnv1a64:7da3c5880d5b3c8b` | `fnv1a64:ae72c45675cd5a51` |
| 28 | P110 | `fnv1a64:9de3cb5c921af560` | `fnv1a64:4f79167dfa6a5206` |
| 29 | P120 | `fnv1a64:d35b6afd958e5951` | `fnv1a64:ef4d7be66e935487` |
| 30 | P130 | `fnv1a64:8ddf715e2a423e1a` | `fnv1a64:a0550d65daca0478` |
| 31 | P140 | `fnv1a64:ea055ee0f2d1b32e` | `fnv1a64:0550bcad9fe6b394` |
| 32 | P150 | `fnv1a64:794e0781ca9312c6` | `fnv1a64:09c0da594b8a527c` |
| 33 | P071 | `fnv1a64:bff3716e09a6c5ba` | `fnv1a64:547ca08c42b98c18` |
| 34 | P081 | `fnv1a64:3ef46bbd42c84fa7` | `fnv1a64:cf8ffaa70d3020d5` |
| 35 | P091 | `fnv1a64:824728c3510332af` | `fnv1a64:03da1d1368e061ed` |
| 36 | P101 | `fnv1a64:95e6bc29c77e9260` | `fnv1a64:477c074b2fcdef06` |
| 37 | P111 | `fnv1a64:da8fb5a87a734c37` | `fnv1a64:792fdfdf0ab84905` |
| 38 | P121 | `fnv1a64:fae03d8b9eb68d84` | `fnv1a64:93fc1b35a4f78552` |
| 39 | P131 | `fnv1a64:52f54f145c05a79b` | `fnv1a64:59c59f48a486bfc1` |
| 40 | P141 | `fnv1a64:cc39ae182de9460f` | `fnv1a64:71b14c60d3f60fcd` |
| 41 | P151 | `fnv1a64:7d9852dc3a1dc411` | `fnv1a64:a5a1d1ef6a272fc7` |
| 42 | B13 | `fnv1a64:ed62876e615075cf` | `fnv1a64:4ca316f44f9eed0d` |
| 43 | R01 | `fnv1a64:19319cc747e30756` | `fnv1a64:9a026b20e7d5caac` |
| 44 | R02 | `fnv1a64:73f96693d7b98f15` | `fnv1a64:de3222a8c2cf5dd3` |
| 45 | R03 | `fnv1a64:c37dac4e50ca6b72` | `fnv1a64:20c050dfa65c2ae0` |
| 46 | R040 | `fnv1a64:7ad6ff2258c418f6` | `fnv1a64:389cd784fcd0d5cc` |
| 47 | R050 | `fnv1a64:fbd17737e15c9fb3` | `fnv1a64:e4bb8e1aba45cde9` |
| 48 | R060 | `fnv1a64:545d053287329e9e` | `fnv1a64:cec9e9f248fd4f24` |
| 49 | R070 | `fnv1a64:a76233d7246a0ea7` | `fnv1a64:37fdc2c0eed1dfd5` |
| 50 | R080 | `fnv1a64:4559bd84d9568686` | `fnv1a64:b33a5e501f7647bc` |
| 51 | R090 | `fnv1a64:5be84ce1cbfdc0cb` | `fnv1a64:6898fcde51381e11` |
| 52 | R100 | `fnv1a64:ac17e24fb6690b2d` | `fnv1a64:2a3cdbc73435f0fb` |
| 53 | R110 | `fnv1a64:1c343d3a9d775210` | `fnv1a64:8844202d5efd5f16` |
| 54 | R041 | `fnv1a64:d39bea98787834cf` | `fnv1a64:32dc7a1e66c6ac0d` |
| 55 | R051 | `fnv1a64:80a98c57e422e0a6` | `fnv1a64:6d8254a7fdd9675c` |
| 56 | R061 | `fnv1a64:0223e57a26a5ad01` | `fnv1a64:682df1def61b7417` |
| 57 | R071 | `fnv1a64:a14d34a391db379c` | `fnv1a64:a05d6d4f91e1325a` |
| 58 | R081 | `fnv1a64:398e2edca907118b` | `fnv1a64:6a5d2dab11792f51` |
| 59 | R091 | `fnv1a64:2264582eaf4fe7ee` | `fnv1a64:1c03b72a3c45b6d4` |
| 60 | R101 | `fnv1a64:88ab6ee93fd6b3de` | `fnv1a64:a3a9bcd45f844ee4` |
| 61 | R111 | `fnv1a64:f58c16b4b0ce66c9` | `fnv1a64:c9aaf1a78d9a2a4f` |
| 62 | R12 | `fnv1a64:bb43f4c2765d3889` | `fnv1a64:7376d93b7f42e48f` |
| 63 | R13 | `fnv1a64:8791d2a754b776bd` | `fnv1a64:11554469470c41eb` |
| 64 | R14 | `fnv1a64:97fbd3fdea0bfc8b` | `fnv1a64:c8cad2cc527e1a51` |
| 65 | R15 | `fnv1a64:4fdca9c6d4f475df` | `fnv1a64:51466a3e86d1f1fd` |
| 66 | O010 | `fnv1a64:31bc6ef21105b3e1` | `fnv1a64:d5b04114ba0cfff7` |
| 67 | O0200 | `fnv1a64:e4cd00b54188d92e` | `fnv1a64:00185e81ee9dd994` |
| 68 | O011 | `fnv1a64:96e5c1ba94c4631e` | `fnv1a64:1152a67a568f13a4` |
| 69 | O0210 | `fnv1a64:6cdb179a72e206e9` | `fnv1a64:f957b4a1fd532fef` |

Both certificate replays first rejected their stale active C01 cell, generated current evidence, then passed after installing only the generated active-build cells. The initial failures are retained in [certificates-true-replay.log](../../../.temp/brief-598/certificates-true-replay.log) and [certificates-false-replay.log](../../../.temp/brief-598/certificates-false-replay.log); these were maintenance staleness, not waived checks.

| promoted.tsv row | ID | Build cell | SHA-256 before | SHA-256 after |
|---:|---|---|---|---|
| 2 | C01 | hash=true, column 5 | `524f531d373f0c7637282419186fca29e43712079cfde1cc13df822ccfce8a88` | `60912241a1682786b571935f405e27936d3c9aaf7db14362d2e9ae370a2a702d` |
| 3 | C02 | hash=true, column 5 | `26fd2d93d3e3b9979b689cc1b75798b9d71abbc34d92a5e7ce0bc9f28793d869` | `805c63c09354eb02efe508976148ea36b870e88d7deaf63b68824853efe154c4` |
| 2 | C01 | hash=false, column 6 | `2979cfbb3d80980eb87946b381d5fd1ede95868d24bd84cea59972b34eaada96` | `b46e700c17d7ec832c590da168acbc3c3108daeb3be6accefd9dc62166dd3283` |
| 3 | C02 | hash=false, column 6 | `333bb4acee77d323f02b9308dde6680826a9bfec927991b35b1ee70a6f3c82b2` | `b01657ccb8b9cd68fc403b4e9b9aa5164e1115fccec0c153dc1dfc2aed268358` |

These SHA-256 values identify the long certificate cells for review; they are not substituted into the inventory. Full old/new cell contents are in [certificate-cell-changes-true.json](../../../.temp/brief-598/certificate-cell-changes-true.json) and [certificate-cell-changes-false.json](../../../.temp/brief-598/certificate-cell-changes-false.json). All four were copied from the earned `.temp/brief-546/certificates/` output. No row identity, strength, precondition or exclusion changed.

### 4. Stop-loss counters and open questions

Final counters: drained 33; owed 5; new descriptors 58; passing variants 293 (both builds); new lifecycle obligations 684 per build; falsifier rejections 114 per build; production defects 0; nonexistent-role reports 5. No two consecutive compiling implementation attempts failed to move a counter. [attempt-counters.json](../../../.temp/brief-598/attempt-counters.json) preserves each attempt; [counters.json](../../../.temp/brief-598/counters.json) and [PROGRESS.md](../../../.temp/brief-598/PROGRESS.md) preserve checkpoints.

The initial candidate calibrations are recorded, not hidden: the literal KeyMap Function was traversed as rule-LHS syntax on both paths, so the genuine callback uses a named delayed definition; AssociateTo[Part] returns the full updated target (existing semantic tests confirm it); and the first restore attempt over-specified cleanup Print lines in the failure buffer. The final restore checks cleanup using exact preserved guest effects. None was a resumed-versus-queued mismatch or a weakened required variant. One temporary Rust diagnostic had a Debug formatting compile error; it was corrected and all temporary survey/probe tests were removed before the final gates.

Procedural deviation: early candidate builds reused the pre-existing `.temp/brief-597/target-tests` cache. This was corrected before all final mandatory gates; new scratch, logs and final build artifacts are under `.temp/brief-598/`. Existing mandated tests retain their own fixed legacy output locations, including the authorized brief-546 certificate directory. No repository script or production source was changed to relocate them.

Director decisions remaining: remove or otherwise disposition the five nonexistent role declarations, apply the four proposed policy corrections, and close the ledger mechanism only in the separate authorized follow-up. All mandatory gates ran to green; the native ZeroMQ skips are disclosed above.
