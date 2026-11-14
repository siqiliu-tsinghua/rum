# brief-738 — G15-CM 2: requirements, enforcement and explicit owners

Scope: **test-only implementation** under
`crates/rum-stdlib/tests/suspension_oracle/` plus committed derived data under
its `inventory/obligations/`. No production source, no `Cargo.toml`/lock, no
script, doc or historical-scene edit.

## Goal

Consume CM1's accepted identity layer and build the manifest's requirement and
ownership stage, and nothing beyond it: an exhaustive policy/applicability/
route table, the counted requirement bag over the accepted population, an
exact and disjoint executed/deferred/proof partition, explicit owner states for
the 44 G13 owners, the campaign and the five named source-provenance duties,
and test-only integrity/closure predicates. When you are done I must be able to
run one test binary in both feature modes that (a) reproduces 1,042 C2 rows /
78,330 keys / 18,840 encoding keys and the 964 G13 rows from committed inputs,
(b) assigns every accepted row's keys to exactly one enforcement mode, (c) keeps
`owner_closure(C2)` false at 118 Residual + 0 Lost with every Open deferral
named, and (d) rejects wrong owners, duplicate requirements, lost keys,
encoding execution and premature closure with their own named predicates. You
execute no guest, mint no encoding receipt and promote no owner.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`.
- The governing design: `.temp/brief-621/MANIFEST-DESIGN.md` — "Policies and
  applicability", "Owners and enforcement", "Porting and the divergence
  ledger", and the concrete Route/Mode/OptionReadContinuation gaps section.
- The reviewed plan: `.temp/brief-736/PLAN.md` §1–2 and
  `draft-CM2-requirements-owners.md` are raw material; this brief supersedes
  them where they differ.
- **CM1's accepted layer (authoritative, committed)**:
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/`
  `{inputs,population,aliases,crosswalks,transfers,owners}.tsv` — 2,204 aliases,
  1,042 C2 rows with per-row key counts and key digests, 964 G13 rows with
  their exact owners, 198 prior settlements, 44 G13 owners, five source duties.
  `obligations/{identity,discovery,resolve,imports,crosswalk}.rs` supply the
  row/alias types and the committed-table loader.
- The C2 authority (read-only): `.temp/g15-c2/c68/settled.tsv` `73ec2ff5…`,
  `required-tuples.tsv` `3c2dcdff…`, `named-source-owners.json` `2d1ab851…`.
- The surviving G13 policy tables (read-only):
  `.temp/brief-621/tail-policies-final.tsv` `785b4087…`,
  `tail-policy-counts-final.tsv` (25 policies), `tail-exclusions.tsv`,
  `settled.tsv` `93245105…`; `.temp/brief-623/owned-obligations.tsv`
  (2,027 owned rows, 44 owners, 964 deferred).
- The committed record: `docs/dev/codex/brief-621-g15c-settle-by-execution.md`
  (198 prior settlements), brief 623's "Campaign invariants", brief 736
  (design), brief 737 (CM1).
- The feature-independent identity campaign: a fresh run prints
  `DISCOVERY CAMPAIGN … edges=9477 resolved_source_edges=26344 …` and
  `TRANSFER GRAPH excluded=513 retained_roots=283 concrete_root_sets=2460`.

## Rulings carried into this brief

- CM1's committed inventory is the authority; `.temp` is provenance only. A
  standing assertion never reads a `.temp` file; an absent historical file is
  `PortBlocked` with its path, never a silent skip.
- The five-component C2 keys stay bound by CM1's per-row `keys` count and
  `key_digest` (all 1,042 recomputed and verified). **Do not commit the 32 MB
  enumerated tuple table in this stage.** If a predicate genuinely requires
  the enumerated keys, STOP and ask for a size ruling.
- Canonical pair counts remain 70 keys (50 non-encoding + 20 encoding) per row
  for `P0325`/`P0326`, 140/40 combined. No requirement row is added, removed
  or re-pinned.
- 247 aliases stay unresolved (197 PortBlocked, 37 ambiguous occurrences, 13
  alias collisions); they keep their explicit PortBlocked/ambiguity records and
  are never retired, merged or given a nearest-line identity.
- 7,061 new candidate edges and 3,034 ambiguous-dispatch / 826
  unknown-construct diagnostics stay named blockers; this stage does not waive
  them into the requirement bag.
- 42 transfer-source PortBlocked cases, 12 missing targets and 24 changed
  caller-graph diagnostics stay preserved; no root or cycle proof is invented.
- The absent brief-621 continuation inputs are not restorable; the ratified
  replacement derivation stands and every unrecoverable datum is `PortBlocked`
  and named. No invented hash, row, key, owner or receipt.
- Manifest integrity and owner closure are separate predicates. C2 stays Open
  at 118 Residual + 0 Lost; `owner_closure(C2)` is false; no partial or
  import-only stage may report full CM green.
- The 18,840 encoding keys are deferred, never scheduled or credited; no
  encoding execution exists in this stage.
- The module split is deferred (CM1's ruling Q6): keep the seven existing
  modules and add only the files listed below.

## Scope

Files you MAY create or edit, all under
`crates/rum-stdlib/tests/suspension_oracle/`:

- `obligations/mod.rs` — add the new modules.
- New `obligations/{policy.rs,applicability.rs,routes.rs,requirements.rs,
  partition.rs,owners.rs,requirement_tests.rs}`. Do not widen this list
  without a director rewrite.
- New committed data `inventory/obligations/{policies,applicability,routes,
  enforcement,owned,source-duties,proof-modes}.tsv`, sorted, tab-separated,
  one header line, no timestamps, no free-form prose columns.
- Maintenance-only paths, exact: `crates/rum-stdlib/src/tests/
  retention_closure/inventory/{local-contracts,promoted}.tsv`; the six
  `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv`; LP01
  `logical-profile-{required,bindings}.tsv` and
  `logical-profile-losses.json`; `inventory/typed-slot-stages.tsv`; runtime
  frame closed lists only if a new test frame is introduced (none planned).

Read-only: every `.temp` input above, the CM1 inventory, the production tree,
all historical scenes and caches. Scratch goes under the dispatched brief's
`.temp/brief-NNN/`.

Forbidden: production source, `Cargo.toml`/`Cargo.lock`, scripts, docs,
existing inventory predicates, historical-scene writes, dispatch/target-cache
edits, network, git, `unsafe`, new dependencies.

## Deliverable

Test-layer types and functions; the worker fills the bodies:

```text
PolicyId        = the exact policy token used by the accepted tables
PolicyEntry     = kind (tail | carrier | route | applicability) + predicate + route set
Applicability   = row kind -> required policy set, with no default arm
Route           = concrete consumer/route class named by the design tables
RequiredBag     = ordered BTreeMap<ObligationKey, RequirementSet>, duplicates rejected first
EnforcementMode = Execute | Deferred(OwnerId, NamedBar) | Proof(ProofId)
OwnedObligation = { row: RowKey, owner: OwnerId }
OwnerState      = Open | Done(PromotionId)   // no Done is created in this stage
SourceDuty      = { id: DutyKey, subjects: Vec<SourceKey>, dependencies: Vec<RowKey>, owner: OwnerId, bar: NamedBar }

requirements(&Population, &PolicyTable, &Applicability, &Routes) -> RequiredBag | Diagnostics
partition(&RequiredBag, &ApprovedLedger, &OwnerStates) -> EnforcementPlan | Diagnostics
manifest_integrity(&Plan) -> bool
owner_closure(&Plan, OwnerId) -> bool
```

Rules the implementation must enforce:

- The policy table is exhaustive over every token in CM1's `aliases.tsv` and the
  25 `tail-policy-counts-final.tsv` entries. An unclassified token, a duplicate
  declaration, a wildcard/default/`Other` arm, or an applicability rule
  inferred from what passed is a named diagnostic, never a silent default.
- Requirement counting happens before any set conversion; duplicate tuples are
  detected and named. Every accepted row's policies are ordered and unique.
- Every retained key appears exactly once across the Execute / Deferred / Proof
  bags (disjoint, multiplicity one). The invariant is checked at row
  granularity with the exact per-row counts and digests CM1 committed, plus the
  bag totals 1,042 rows / 78,330 keys / 18,840 encoding.
- The 964 G13 rows carry their exact S-step owner from CM1's `owners.tsv`; their
  requirement counts are re-derived from the surviving 621 tables where
  possible, and every unrecoverable datum is `PortBlocked` and named. No
  estimated number stands in for an absent requirement.
- The 118 Residual rows are `Deferred(G15-C2, named bar)`; the prior 198
  settlements keep their accepted dispositions and are never reopened. A
  proof-covered key receives a certificate identity, never an execution or a
  SAME claim.
- Owner namespaces are distinct: `G15-C2`, each exact G13 `S`/`F`/`R` step, and
  the five source-provenance duties (`ToExpression`, `Session::lower_text`,
  outer-request lowering, loader, file-reread provenance). The five duties stay
  Open, carry their exact subject and row-dependency mapping, and never own a
  C2 row. Completed S1–S9 cannot own deferred rows.
- `manifest_integrity` may be green only with explicitly admitted Open
  deferrals whose identity, requirement, named bar and owner are all checked;
  `owner_closure(C2)` stays false and any premature Done, reassignment or
  orphaned duty is a named rejection.
- No guest, no receipt, no owner promotion, no production behaviour change.

Counts the run must print and the inventory must carry: accepted aliases 2,204;
C2 rows 1,042; canonical keys 78,330; encoding 18,840; G13 deferred 964 over 44
owners (S10 = 28 + 10 + 1); prior 198; Residual 118; the per-mode bag totals;
policy table entries; applicability entries; route entries; proof modes; the
five duties; and every named diagnostic with its kind.

## rum / Rust pitfalls (read twice)

- No policy default, `Other`, wildcard, factory-derived requirement or
  inferred applicability. Concrete route types and Twice phases differ.
- Do not convert to a set before counting duplicates; an empty pausing
  requirement cannot substitute for a source-specific pure/boundary proof.
- The five source duties do not partition the 1,042 C2 rows; they need exact
  subject links. Completed S1–S9 cannot own deferred rows.
- Do not treat the transitive table's 10 stale dispositions as canonical state.
- No name/suffix/nearest-line matching; dynamic dispatch stays named.
- Deterministic ordering everywhere (BTreeMap/BTreeSet, sorted output), or the
  inventory hashes will not reproduce.
- Keep the test layer process-clean: no global env mutation, no test-name order
  dependence, no shared mutable scratch between independent cases.

## Verification (MANDATORY)

Run the commands **directly** — the dispatch already places you in a 16G
memory scope. Nested `scripts/memory-scope.sh` cannot reach the user bus in
this sandbox; do not attempt it. One cargo command at a time,
`CARGO_NET_OFFLINE=true`, `-j8`, `RUST_TEST_THREADS=8`.

**Shape stop.** New tests pass in both feature modes:
`cm_policy_exhaustive`, `cm_required_bag_exact`, `cm_partition_exact`,
`cm_owner_integrity`, `cm_c2_closure_is_blocked` and their negatives.

Falsifiers, each red for its own named predicate on both feature arms: remove
a leaf/applicability entry; declare a duplicate requirement; lose a frozen key
while partitioning; schedule an encoding key for execution; introduce an
unknown owner, a Done owner for a C2 residual, or a reassigned duty; mark C2
Done while one Residual remains; remove one source duty; add an unclassified
policy token. Positive controls: the complete independent policy expansion
reproduces the committed inventory; the Open ledger is unchanged; the proof bag
is exact. A failure caused by the wrong predicate (missing file, compile error,
an earlier guard) does not count.

**Campaign stop.** After the shape, expand the full population once per feature
mode: print per-owner/per-policy/per-route Execute/Deferred/Proof counts and
the retained key totals. No guest, no receipt, no settlement, no owner
promotion. Stop after two batches that add no new requirement evidence, or on
any unexplained red.

**Acceptance suite** (once, after the last edit): `cargo fmt --check`; the
workspace suite in both feature modes with the seven documented switch-gated
skips (`--skip lp03_adopted_phases_are_exact_in_completed_and_resumed_runs
--skip lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs
--skip lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs
--skip lp658_host_eval_reset --skip lp658_host_resume_reset
--skip lp655_root_context_falsifier --skip lp656_public_root_context_falsifier`)
and `--no-fail-fast`; then the same binary set under
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` for `lp03_phase_rows`,
`lp04_definition_rows` and `lp05_assignment_rows` (the `lp655`/`lp656` reds are
the documented standing two-arm exclusions, disclosed not repaired); the full
`suspension_oracle` and `typed_slot_census` in both modes. Budget ≤1,500 s per
expanded oracle build+run. Reproduce any semantic red at one test thread before
calling it a defect.

## Maintenance before the first acceptance suite

Editing test sources expires the retention fingerprints. Replay the ignored
`regenerate_local_contract_fingerprints` maintenance test, then
`local_production_acceptance` in both modes, and mechanically copy only the
replayed cells into
`crates/rum-stdlib/src/tests/retention_closure/inventory/{local-contracts,
promoted}.tsv`. Replay the ownership discovery and reconcile the six
`crates/rum-stdlib/src/tests/ownership_graph/inventory/` files from the new
discovery; new `UnknownConstruct` rows for touched code stay unknown and are
disclosed; changed, deleted or re-labelled existing identity/status/
disposition/exclusion rows STOP. If covered producer sites move, replay
`logical_counter_port` and `typed_slot_census` and copy only their
mechanically derived site/hash cells. Rejection-census count changes STOP for
my ruling. Quote every replay verdict; a stale-derived-cell red is an ordering
artifact, never grounds for weakening an inventory. Basis: `TEMPLATE.md`
maintenance section, `local/contracts.rs:88`, `local/production.rs:7`,
`ownership_graph/mod.rs:36`.

## Checkpointing — protect the scene

Checkpoint after the policy/applicability tables, after the requirement bag,
after the partition/owner states and after each suite. Record the concrete next
action, what you ran, and every open question with its evidence. Preserve every
failed attempt and a filter interruption. Stop after two batches that add no new
evidence, or on any unexplained red.

## Hard rules

- No git, network, `unsafe`, external engine, new dependency or write outside
  this repo and the dispatched brief's `.temp` directory.
- No source-scope expansion. If a production change becomes necessary, STOP
  and report; it requires a rewritten brief and the mandatory hot-path A/B.
- Requirement completeness is not semantic closure. Do not mark a manifest
  complete, a row settled or an owner Done.
- Actual output and unrun gates disclosed. No weakened contract, invented
  receipt, deleted attempt or re-pinned inventory.

## Report format

One `rum-report/1` via `scripts/report.py` (report path
`.temp/brief-NNN/report.json`), with sections: policy table, applicability and
routes, requirement bag, partition and owner states, diagnostics, counts and
cost, falsifiers, maintenance, questions. Every claim carries `path:line` or a
hash. List every open question with two candidate resolutions and their
evidence. `scripts/report.py check 738` must pass before you stop. Next is CM3
after my gate.

**Dispatch:** `scripts/dispatch-brief.sh 738` (background, `BRIEF_WORKER`
unset = `cdx`).
