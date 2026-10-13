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
