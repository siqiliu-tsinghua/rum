# brief-737 — G15-CM 1: the accepted population and stable source discovery

Scope: **test-only implementation** under
`crates/rum-stdlib/tests/suspension_oracle/` plus committed derived data under
its `inventory/obligations/`. No production source, no `Cargo.toml`/lock, no
script, doc or historical-scene edit.

## Goal

Build the manifest's identity layer, and nothing beyond it: the accepted
population bound by content hash, the source-derived stable key, the exact
alias cross-walk over the current tree, and the strict-tail transfer graph to
a fixed point. When you are done I must be able to run one test binary in both
feature modes that (a) pins the surviving inputs and states which historical
inputs are absent, (b) reproduces the 2,204-alias accounting and the 44-owner
G13 baseline, (c) derives every producer/consumer edge from the current source
and maps every accepted alias exactly once, and (d) reports every new, missing,
ambiguous or stale-tail edge as a named diagnostic. You settle no row, execute
no guest, and promote no owner.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`.
- The governing design: `.temp/brief-621/MANIFEST-DESIGN.md` — "Source
  identities and discovery", "Policies and applicability", "Porting and the
  divergence ledger" (root-level file, read it; this brief implements its
  identity section only).
- The reviewed plan: `.temp/brief-736/PLAN.md` §1–2 and the drafts
  `draft-CM1-discovery-import.md` / `draft-CM2-requirements-owners.md` are raw
  material; this brief supersedes them where they differ.
- The sealed C2 authority (read-only): `.temp/g15-c2/c68/settled.tsv`
  `73ec2ff5…`, `required-tuples.tsv` `3c2dcdff…`, `named-source-owners.json`
  `2d1ab851…`, `paired-calls.json` `1b5140ca…`, `unpaired-rows.json`
  `f597b795…`. The 1,042 campaign rows, 78,330 five-component keys and 18,840
  encoding keys are reproduced from these files, not re-settled.
- The original population (read-only): `.temp/brief-621/population.tsv`
  `0272dfcb…` (2,186), `settled.tsv` `93245105…`, `tail-policies-final.tsv`
  `785b4087…`, `tail-policy-counts-final.tsv` (25 policies),
  `tail-exclusions.tsv` `95515f88…`, `screen-crosswalk.tsv` `9fa61154…`,
  `crosswalk-583.tsv` `b5d6f38e…`, `crosswalk-548-unsettled.tsv`
  `795a9f7a…`.
- The G13 ownership baseline (read-only): `.temp/brief-623/owned-obligations.tsv`
  — 2,027 owned rows, 44 non-C2 S-step owners, 964 deferred; reproduce
  S10 = 28 deferred + 10 measured Equal + 1 known loss exactly.
- The historical census (calibration cohort and raw material, not the
  implementation): `.temp/brief-583/{census.py,build_graph.py,classify.py,
  resolve.py,supplement.py,probe.rs}` and its `census.tsv`,
  `reviewed-census.tsv`, `supplemental-edges.tsv`, `unsettled.tsv`.
- The pinned input ledger from the design round: `.temp/brief-736/input-pins.json`
  (16 files with SHA-256 and byte counts) — verify, do not re-pin by trust.
- The committed record: `docs/dev/codex/brief-621-g15c-settle-by-execution.md`
  (the 198 prior settlements: 43 Equal, 5 Lost, 140 Unreachable, 10 Boundary),
  `docs/dev/codex/brief-623-g15c2-settle-unowned-census-rows.md` ("Campaign
  invariants"), `docs/dev/codex/brief-736-g15cm-obligation-manifest-design.md`.
- Existing test infrastructure: `coverage/source.rs` (syn parse + name
  existence), `support/files.rs`, `descriptors.rs`; `syn`, `quote` and
  `proc-macro2` are already dev-dependencies.

## Rulings carried into this brief

- The brief-621 continuation1/2/3 inputs are **absent and not restorable**
  (no copy, no historical pin). This brief binds what survives by hash and
  builds the ratified replacement derivation for the identity layer only:
  row and owner data are derived from the surviving tables and the current
  source, calibrated against the retained historical controls, and every datum
  that cannot be re-derived is `PortBlocked` and named. No invented hash, row,
  key, owner or receipt.
- Manifest integrity and owner closure are separate predicates. C2 stays
  Open at 118 Residual; nothing here changes an owner, a disposition or a
  count.
- Canonical pair counts are 70 keys (50 non-encoding + 20 encoding) per row
  for `P0325`/`P0326`, 140/40 combined. No requirement row is added or
  removed.

## Scope

Files you MAY create or edit, all under
`crates/rum-stdlib/tests/suspension_oracle/`:

- `mod.rs` — add `pub mod obligations;` only.
- New `obligations/{mod.rs,identity.rs,discovery.rs,resolve.rs,imports.rs,
  crosswalk.rs,tests.rs}`. Do not widen this list without a director rewrite.
- New committed data `inventory/obligations/{inputs,population,aliases,
  crosswalks,transfers,owners}.tsv`, sorted, tab-separated, one header line,
  no timestamps, no free-form prose columns.
- Maintenance-only paths, exact: `crates/rum-stdlib/src/tests/
  retention_closure/inventory/{local-contracts,promoted}.tsv`; the six
  `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv`; LP01
  `crates/rum-stdlib/tests/suspension_oracle/inventory/
  logical-profile-{required,bindings}.tsv` and
  `logical-profile-losses.json`; `inventory/typed-slot-stages.tsv`; and the
  runtime frame closed lists only if a new test frame is introduced (none is
  planned).

Read-only: every `.temp` input above, the production tree, all historical
scenes and caches. Scratch and copies go under the dispatched brief's
`.temp/brief-NNN/`.

Forbidden: production source, `Cargo.toml`/`Cargo.lock`, scripts, docs,
existing inventory predicates, historical-scene writes, dispatch/target-cache
edits, network, git, `unsafe`, new dependencies.

## Deliverable

Test-layer types and functions; the worker fills the bodies:

```text
ProducerId      = crate + qualified impl/trait/function + normalized signature
                  + concrete dispatch arm when applicable
ConsumerSiteId  = enclosing qualified function + normalized call-token digest
                  + occurrence index among identical calls in that function
EdgeKey         = (ProducerId, ConsumerSiteId)
ObligationKey   = (EdgeKey, TailPolicy)
Row             = one EdgeKey, ordered unique policies, aliases, diagnostic span
AliasRecord     = accepted alias -> row | retired | duplicate | transfer
Transfer        = excluded strict tail -> indexed caller(s), or a retained root

discover(&SourceTree, FeatureMode) -> DiscoveredGraph
reconcile(&DiscoveredGraph, &AcceptedInputs) -> Population | Diagnostics
```

Rules the implementation must enforce:

- A source span is diagnostic only. A moved function keeps its qualified key;
  a module/signature/callee/policy change requires an explicit versioned
  cross-walk entry, not a silent re-key.
- Identical calls in one function have distinct occurrence indices; insertion
  or reordering that makes attribution ambiguous fails with a named
  diagnostic, never a nearest-line choice.
- A multi-policy row is one population row with ordered policies; duplicate
  input tuples are detected before any set conversion.
- Discovery runs before factories: lexical resolution, typed-frame/macro
  expansion and the supplemental pass; no fixture, feature flag or factory
  may add, remove or waive a requirement.
- New edge, missing accepted key, changed tail fingerprint, unknown construct
  or ambiguous dispatch emits a named diagnostic before anything else runs.
  A diagnostic is never a settlement; it blocks complete-manifest acceptance
  for my ruling and is reported with its row/key context.
- Transfers: the 513 excluded strict tails stay explicit with their indexed
  caller links; the 283 without an indexed caller remain root obligations;
  the dependency graph is validated to a fixed point with a nonempty concrete
  root or an approved cycle proof; a missing target, rootless cycle or changed
  post-producer destructor fingerprint is a named diagnostic.
- Every one of the 2,204 accepted aliases is accounted for exactly once:
  1,024 original C2 rows replaced by their C68 record, 18 added edges with
  their derivation, 964 G13 rows with their exact S/step owner (44 owners;
  S10 28+10+1), 198 prior settlements with their accepted disposition. The
  five named source-provenance owners stay Open and separate; they do not own
  C2 rows.
- The committed inventory is the runtime authority; `.temp` files are
  provenance pins only. No standing assertion may read a `.temp` file: the
  hash check of a pin runs only when that file exists and reports
  `PortBlocked` (with the missing path) when it does not.
- Report the byte size of every committed inventory file and keep the schema
  minimal: no redundant diagnostic columns, one header line, no prose.

Counts the run must print and the inventory must carry: accepted aliases
2,204 (2,186 − 1,024 + 1,042); C2 rows 1,042; keys 78,330; encoding 18,840;
G13 deferred 964; prior settlements 198; unresolved pairings 24; excluded
strict tails 513; rootless strict tails 283; current discovered edges with
new/missing/ambiguous/stale diagnostics counted separately.

## rum / Rust pitfalls (read twice)

- The 2,204 is a historical baseline, not today's graph count. Report the
  current delta; never force the current tree to fit the baseline.
- No name-suffix or nearest-line resolution; dynamic dispatch stays ambiguous.
- No settlement from discovery, no Equal from absence, no retirement from a
  missing row, no Unreachable from a zero-hit run.
- Do not run the historical Python generators inside their sealed working
  directories; copy inputs to your scratch if a rerun is needed.
- Never edit, re-pin or delete a historical input; never weaken an assertion
  to make a diagnostic disappear.
- Deterministic ordering everywhere (BTreeMap/BTreeSet, sorted output), or the
  inventory hashes will not reproduce.
- Keep the test layer process-clean: no global env mutation, no test-name
  order dependence, no shared mutable scratch between independent cases.

## Verification (MANDATORY)

Run the commands **directly** — the dispatch already places you in a 16G
memory scope. Nested `scripts/memory-scope.sh` cannot reach the user bus in
this sandbox; do not attempt it. One cargo command at a time,
`CARGO_NET_OFFLINE=true`, `-j8`, `RUST_TEST_THREADS=8`. The director's host
gate has the current unmodified tree green in both modes (500 oracle tests per
mode, `typed_slot_census` green); your own runs are still your gate.

**Shape stop.** New tests pass in both feature modes:
`obligations_inputs_exact`, `obligations_alias_accounting_exact`,
`obligations_discovery_shape`, `obligations_discovery_negatives`,
`obligations_transfer_fixed_point`, `obligations_transfer_negatives`.
Falsifiers, each red for its own named predicate on both feature arms:
in a copied source fixture add one evaluating call; delete one indexed call;
insert an identical-call occurrence; alter a tail token without changing the
line count; omit a typed/macro/supplemental edge; introduce an unresolved
dispatch; delete one transfer target; form A→B→A without a concrete root;
change a post-producer destructor fingerprint. Positive controls: move the
original function without a semantic/module change and require identical
keys; the original indexed caller graph and its root proof stay green.
A failure caused by the wrong predicate (missing file, compile error, an
earlier guard) does not count.

**Campaign stop.** Run the discovery once over the whole workspace per
feature mode: print the discovered edge count, the accepted-alias mapping
(2,204 exact), every diagnostic with its kind, and the transfer graph counts.
No guest, no receipt, no settlement, no owner promotion. Stop after two
batches that add no new identity evidence, or on any unexplained red.

**Acceptance suite** (once, after the last edit): `cargo fmt --check`;
`cargo test --workspace` in both feature modes; the full
`suspension_oracle` and `typed_slot_census` in both modes. Budget ≤1,500s per
expanded oracle build+run. Reproduce any semantic red at one test thread
before calling it a defect.

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
my ruling. Quote every replay verdict; a stale-derived-cell red is an
ordering artifact, never grounds for weakening an inventory. Basis:
`TEMPLATE.md` maintenance section, `local/contracts.rs:88`,
`local/production.rs:7`, `ownership_graph/mod.rs:36`.

## Checkpointing — protect the scene

Checkpoint after the input baseline, after the discovery shape, after the
alias cross-walk and after each suite. Record the concrete next action, what
you ran, and every open question with its evidence. Preserve every failed
attempt and a filter interruption. Stop after two batches that add no new
evidence, or on any unexplained red.

## Hard rules

- No git, network, `unsafe`, external engine, new dependency or write outside
  this repo and the dispatched brief's `.temp` directory.
- No source-scope expansion. If a production change becomes necessary, STOP
  and report; it requires a rewritten brief and the mandatory hot-path A/B.
- Discovery completeness is not semantic closure. Do not mark a manifest
  complete, a row settled or an owner Done.
- Actual output and unrun gates disclosed. No weakened contract, invented
  receipt, deleted attempt or re-pinned inventory.

## Report format

One `rum-report/1` via `scripts/report.py` (report path
`.temp/brief-NNN/report.json`), with sections: input baseline, discovery,
alias cross-walk, transfers, diagnostics, counts and cost, falsifiers,
questions. Every claim carries `path:line` or a hash. List every open
question with two candidate resolutions and their evidence.
`scripts/report.py check 737` must pass before you stop.

**Dispatch:** `scripts/dispatch-brief.sh 737` (background, `BRIEF_WORKER`
unset = `cdx`).
