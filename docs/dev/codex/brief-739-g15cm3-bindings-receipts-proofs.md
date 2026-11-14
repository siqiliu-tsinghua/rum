# brief-739 — G15-CM 3: bindings, execution receipts and proof certificates

Scope: **test-only implementation** under
`crates/rum-stdlib/tests/suspension_oracle/` plus committed derived data under
its `inventory/obligations/` and `.rum` fixture sources under
`fixtures/obligations/programs/`. No production source, no `Cargo.toml`/lock,
no script, doc or historical-scene edit.

## Goal

Consume CM1's identity layer and CM2's counted partition and build the
manifest's binding and evidence stage, and nothing beyond it. When you are
done I must be able to run the test binary in both feature modes and see, from
committed inputs only:

- the **Execute** bag — 10,505 nonencoding keys over the 175 Equal rows —
  bound to immutable fixture programs and, only where an actual attributed
  run happened, covered by execution receipts that carry program, setup,
  answer, mode/build, request, phase, channel, lifecycle and raw identity;
- the **Proof** bag — 43,390 keys over the 749 Boundary / Identity /
  NoObservableTail / Transferred / Unreachable rows — imported as proof
  certificates bound to the pinned C68 ledger, never as executions or SAME
  claims;
- the **Deferred** bag — 5,595 nonencoding residual keys plus all 18,840
  encoding keys — exactly as deferred as CM2 left it, never bound, executed
  or credited;
- an exact required / bound / executed reconciliation at row granularity with
  multiplicity one, and an immutable known-loss ledger (the 5 Lost prior rows
  and the authenticated C1 promotion) whose changed or now-SAME baseline is
  red pending a named promotion.

You execute only what the Execute bag's frozen requirements demand. You mint
no SAME result, settle no row, promote no owner, and run no encoding key.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`.
- The governing design: `.temp/brief-621/MANIFEST-DESIGN.md` — "Fixture
  bindings and receipts", "Proofs and the divergence ledger", and the
  known-loss promotion rules.
- **CM1's accepted layer (authoritative, committed)**:
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/
  {inputs,population,aliases,crosswalks,transfers,owners}.tsv`.
- **CM2's accepted layer (authoritative, committed)**: the same directory's
  `{policies,applicability,routes,enforcement,owned,source-duties,
  proof-modes}.tsv`, and `obligations/{policy,applicability,routes,
  requirements,partition,owners}.rs`. The partition is fixed: Execute 10,505 /
  Deferred 24,435 (5,595 + 18,840) / Proof 43,390; 51 Open owners; the five
  proof modes carry the C68 ledger pin.
- The C2 authority (read-only): `.temp/g15-c2/c68/settled.tsv` `73ec2ff5…`,
  `required-tuples.tsv` `3c2dcdff…`, `port/source-entries.jsonl`
  `65ffd5f3…`, `paired-calls.json` `1b5140ca…`, `named-source-owners.json`
  `2d1ab851…`, `unpaired-rows.json` `f597b795…`.
- The historical evidence packets named by the C68 ledger: the per-scene raw
  packets, native/pair proof packets, the P0496 closure calibration and the
  current equality mutants. Anything absent is `PortBlocked` and named; the
  brief-621 continuation2/3 inputs (including the 97-binding LHS submatrix
  file) are absent and are not restorable.
- The C1 promotion: `docs/dev/codex/brief-622-g15c1-replaceall-application-
  tail.md` — import its authenticated promotion; never install `P0941`/
  `P0946` as fresh allowed-DIFF baselines.
- The existing oracle support: `suspension_oracle/{descriptors.rs,
  host_facade.rs}`, `support/{drive,observe,lifecycle,faults}.rs`,
  `fixtures/logical_profile/mod.rs`. All existing checks stay active.

## Rulings carried into this brief

- CM1 and CM2 committed inventory are the authority; `.temp` is provenance
  only. A standing assertion never reads a `.temp` file; an absent historical
  file is `PortBlocked` with its path, never a silent skip.
- The three bags are fixed and disjoint. Execute is a requirement mode, not a
  receipt: a key enters the executed bag only through an actual attributed
  run. A proof certificate is never an execution and never a SAME claim. A
  deferred key is never bound, executed or credited.
- The 18,840 encoding keys stay `Deferred(G13-transport, bar)` regardless of
  their row's disposition; encoding is never generated, executed or credited.
- The 118 Residual rows stay `Deferred(G15-C2, c2-residual-evidence)`; the 964
  G13 rows keep their 44 S-step owners, 44 recorded counts (3,025 keys) and
  920 `NotYetBound` PortBlocked records; no count is estimated.
- The 247 unresolved aliases and every CM1 candidate/dispatch/construct/
  transfer blocker stay named; no suffix, nearest-line or passed-observation
  inference, and no waiver into a receipt or a proof.
- The pinned C68 ledger is the disposition authority. Each imported proof
  names the ledger pin, the row and the accepted verdict, and its exact scope
  and dependency; a missing packet leaves that duty `PortBlocked`.
- Known losses are immutable records keyed by obligation, tuple, owner,
  program fingerprint and exact structured difference. A changed or now-SAME
  baseline is red until its named promotion is acknowledged; test code cannot
  bless a new baseline by copying output.
- One actual run fans out only to predeclared observed keys. Immutable plan
  sharing is allowed; session or answer-state sharing is forbidden. No
  serialized boolean is imported as a new execution.
- Rows and keys come from committed inputs, never from which fixtures are
  easiest to bind. A partial port is never full CM green.
- Verification reuses by hash identity: unchanged inputs, tables and suites
  need not be rechecked; a stale derived-cell red is an ordering artifact,
  never grounds for weakening an inventory.
- The module split stays deferred; add only the files listed below.

## Scope

Files you MAY create or edit, all under
`crates/rum-stdlib/tests/suspension_oracle/`:

- `obligations/mod.rs` — add the new modules.
- New `coverage/{mod,bindings,receipts,schedule,execution_tests}.rs`.
- New `ledger/{mod,proofs,known_losses,proof_tests}.rs`.
- New `fixtures/obligations/{mod,guest,native,boundaries,proof_controls}.rs`
  plus `.rum` program sources under `fixtures/obligations/programs/`.
- `fixtures/mod.rs` and `support/{mod,observe}.rs` only for additive
  receipt-facing interfaces; every existing check and call site stays.
- New committed data `inventory/obligations/{bindings,programs,proofs,
  known-losses,provenance}.tsv`, sorted, tab-separated, one header line, no
  timestamps, no free-form prose columns.
- Maintenance-only paths, exact: `crates/rum-stdlib/src/tests/
  retention_closure/inventory/{local-contracts,promoted}.tsv`; the six
  `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv`; LP01
  `logical-profile-{required,bindings}.tsv` and
  `logical-profile-losses.json`; `inventory/typed-slot-stages.tsv`; runtime
  frame closed lists only if a new test frame is introduced (none planned).

Read-only: every `.temp` input above, the CM1/CM2 inventory, the production
tree, all historical scenes and caches. Scratch goes under the dispatched
brief's `.temp/brief-NNN/`.

Forbidden: production source, `Cargo.toml`/`Cargo.lock`, scripts, docs,
existing inventory predicates, historical-scene writes, dispatch/target-cache
edits, network, git, `unsafe`, new dependencies.

## Deliverable

Test-layer types and functions; the worker fills the bodies:

```text
bind(&ActiveRequiredBag, &FrozenBindings) -> Result<BoundBag, BindingError>
execute(&BoundPlan, &RunIdentity) -> Result<ExecutionReceipt, RunFailure>
reconcile_executions(&RequiredBag, &BoundBag, &ReceiptBag) -> Result<(), ExactError>
validate_proof(&ProofDuty, &CurrentSource, &PinnedEvidence) -> ProofCertificate
```

Rules the implementation must enforce:

- The receipt constructor is private to the executed-run adapter. A receipt
  carries the tuple, immutable program/setup/answer identity, mode/build
  identity, planned and actual request counts, producer and phase provenance
  (including the second pause), the complete observed channel set, lifecycle
  and native data, and the raw references. Duplicate or missing receipts are
  rejected before any set comparison.
- Execute: only the 175 Equal rows' 10,505 nonencoding keys are bindable and
  executable. Every such key is bound at multiplicity one or named
  `PortBlocked` with its concrete missing evidence; no key is dropped, no
  family is sampled, no context is omitted.
- Proof: each of the 749 rows imports exactly one certificate for its accepted
  verdict (Boundary 20 / 735 keys, Identity 5 / 185, NoObservableTail 126 /
  7,260, Transferred 594 / 35,000, Unreachable 4 / 210). The certificate
  identity is content-addressed by the C68 ledger pin, row and verdict; a
  certificate never becomes an execution or a SAME result.
- Deferred: the 118 Residual rows and all 18,840 encoding keys stay deferred
  with their named bars; an owner staying Open cannot suppress a new or
  changed DIFF.
- Known losses: the 5 Lost prior rows and the authenticated C1 promotion
  import as immutable records with owner, program/tuple/source identity,
  request count and structured difference; a now-SAME or changed baseline is
  red until its named promotion is acknowledged. `P0941`/`P0946` are not
  installed as fresh allowed-DIFF baselines.
- `reconcile_executions` compares active required / bound / actually executed
  keys with multiplicity one and fails closed on any extra, missing or
  duplicate key; it never treats a deferred or proof key as executed.
- No guest campaign may settle a row, mint a SAME, promote an owner or run an
  encoding key.

Counts the run must print and the inventory must carry: Execute 175 rows /
10,505 keys; Proof 749 rows / 43,390 keys and the five class counts; Deferred
118 rows / 5,595 nonencoding plus 18,840 encoding; 964 G13 rows over 44
owners with 44 recorded counts / 3,025 keys / 920 `NotYetBound`; 198 prior
rows; 51 owners (all Open); five source duties; the receipt, bound and
PortBlocked totals per family; and the known-loss records.

## rum / Rust pitfalls (read twice)

- Accepted Equal is not a file full of current receipts: validate the original
  raw packet, the current source, the concrete routes and the complete matrix.
  Missing evidence is `PortBlocked`, never a new owner or a silent baseline.
- Fixed source-stack attribution at first pause does not prove the second; an
  absent unwound first guard does not prove a lost continuation.
- Reject duplicate/missing receipts and duplicate bindings before comparing
  sets. Frozen requirements never depend on which fixtures were easiest to
  bind.
- Preserve the actual Host facade via source reuse; no hand-copied host logic.
- Native ten-field rows, errors, state, trace, requests and lifecycle matter
  even when scalar values agree. Normalize only declared duration leaves.
- Never generate or execute encoding keys; existing independent oracle tests
  stay enabled and no owner deferral may hide a regression.
- Keep the test layer process-clean: no global env mutation, no test-name
  order dependence, no shared mutable scratch between independent cases.
- Deterministic ordering everywhere (BTreeMap/BTreeSet, sorted output), or the
  inventory hashes will not reproduce.

## Verification (MANDATORY)

Run the commands **directly** — the dispatch already places you in a 16G
memory scope. Nested `scripts/memory-scope.sh` cannot reach the user bus in
this sandbox; do not attempt it. One cargo command at a time,
`CARGO_NET_OFFLINE=true`, `-j8`, `RUST_TEST_THREADS=8`.

**Shape stop.** Types and exactness predicates pass in both feature modes:
`cm_bindings_exact`, `cm_receipts_exact`, `cm_proof_certificates_exact`,
`cm_loss_baselines_exact`, `cm_execution_negative_controls`,
`cm_proof_negative_controls`, plus a director-selected complete calibration
cohort spanning at least the C1 promotion rows and one row per proof class.
Scope may stay a cohort only until the shape passes; the campaign then covers
every active key.

Falsifiers, each red for its own named predicate on both feature arms:
delete or duplicate a binding; declare without execute; duplicate a receipt;
pause in setup instead of the producer; collapse two pauses; change
prompt/kind/ordinal; drop a nonfinal print/message/trace event; change a
native counter/projection at equal scalar; lose cancellation cleanup; add an
extra queued answer; force a fresh-request failure; `P0496`'s fourth caller
and removal of the prepared skip (ordinary branch stays reachable); a
Boundary row without its terminal contract; an unarmed NoObservableTail/
Identity or a wrong-row survivor; a removed transfer target or rootless
cycle; a changed or now-SAME known-loss baseline without a named promotion.
Positive controls: a complete exact actual run stays green; the proof bag is
exact; the Open ledger is unchanged. A failure caused by the wrong predicate
(compile error, missing file, an earlier guard) does not count.

**Campaign stop.** After the shape, port complete families with exact
row/key progress and every authenticated attempt retained, then finish every
active Execute key (no sampling, no dropped context) and import every proof
row. Stop after two batches that add no new binding/receipt/proof evidence, or
on any unexplained red. If the measured complete schedule exceeds 1,500 s per
expanded oracle build+run, STOP for a schedule decision and report the
program/schedule sharing data; never omit a context and never substitute a
warm historical timing. Target 900 s steady state. Importing a historical
packet authorizes no new settlement and reopens no parked row.

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
`suspension_oracle` and `typed_slot_census` in both modes. Reproduce any
semantic red at one test thread before calling it a defect.

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
mechanically derived site/hash cells. A new frame enters
`runtime/tests/frame_root_trace/derived_inventory.rs` and
`frame_root_trace.rs`'s count exactly. Rejection-census count changes STOP for
my ruling. Quote every replay verdict; a stale-derived-cell red is an ordering
artifact, never grounds for weakening an inventory. Basis: `TEMPLATE.md`
maintenance section, `local/contracts.rs:88`, `local/production.rs:7`,
`ownership_graph/mod.rs:36`.

## Checkpointing — protect the scene

Checkpoint after the binding types, after the first exact receipt family,
after the proof import, after the known-loss ledger, after the campaign and
after each suite. Record the concrete next action, every command run, the
current/remaining binding and receipt counts, and every open question with its
evidence. Preserve every failed attempt and a filter interruption. Stop after
two batches that add no new evidence, or on any unexplained red.

## Hard rules

- No git, network, `unsafe`, external engine, new dependency or write outside
  this repo and the dispatched brief's `.temp` directory.
- No source-scope expansion. If a production change becomes necessary, STOP
  and report; it requires a rewritten brief and the mandatory hot-path A/B.
- No invented receipt, weakened contract, deleted attempt, history re-pin or
  fresh allowed-DIFF baseline. No encoding run or credit. No settlement, no
  owner promotion, no full CM green.
- Actual output and unrun gates disclosed.

## Report format

One `rum-report/1` via `scripts/report.py` (report path
`.temp/brief-NNN/report.json`), with sections: bindings, receipts, proof
certificates, known losses, exact reconciliation, campaign progress,
falsifiers, maintenance, counts and cost, questions. Every claim carries
`path:line` or a hash. List every open question with two candidate resolutions
and their evidence. `scripts/report.py check 739` must pass before you stop.
Next is CM4 after my gate.

**Dispatch:** `scripts/dispatch-brief.sh 739` (background, `BRIEF_WORKER`
unset = `cdx`).

## Continuation 1 ruling (director) — the calibration cohort is selected; absent packets stay `PortBlocked`

Your STOP is accepted, and the omission is mine: the shape gate named a
"director-selected complete calibration cohort" and named none. This section
selects it and settles Q1/Q2. The scope, permitted files and tables, the six
shape tests, the falsifiers, the campaign, the acceptance suite and every hard
rule stand unchanged.

**My audit of the evidence (presence only).** I re-read the C68 ledger's
`settlement_evidence` for the settled rows and tested every cited path on this
host. Presence is not authentication — you authenticate contents — but the
absences are exhaustive:

| disposition | rows with packet present | rows absent | keys present | keys blocked |
| --- | --- | --- | --- | --- |
| Equal | 166 | 9 | 9,905 | 600 |
| Boundary | 20 | 0 | 735 | 0 |
| NoObservableTail | 126 | 0 | 7,260 | 0 |
| Transferred | 1 | 593 | 50 | 34,950 |
| Identity | 0 | 5 | 0 | 185 |
| Unreachable | 4 | 0 | 210 | 0 |

The absent packets are continuation2 tables that exist nowhere on the host:
`continuation2/{identity-decisions,identity-receipts,transfers,equal-decisions,
equal-real-receipts}.tsv`. The `port/rebase.tsv` half survives at
`.temp/g15-c2/port/rebase.tsv`. The 9 Equal rows are `P0018` (60 nonencoding
keys), `P0580` (70), `P0709` (165), `P0790` (60), `P0795` (60), `P0798` (60),
`P0801` (60), `P2287` (35), `P2288` (30) — 600 keys in all.

- **Q1 — the cohort.** Exactly these rows:
  - known loss (C1 promotion): `P0941` and `P0946` —
    `docs/dev/codex/brief-622-g15c1-replaceall-application-tail.md` line
    21065 (sha256
    `5c88efea8c602e0320c5148ae0e10c8aba786a376b24cc99f262892f33cdc8e1`),
    imported as immutable promotion records and never as fresh allowed-DIFF
    baselines.
  - Execute: `P0001` — the first Equal row in the C68 ledger order whose cited
    packet is present (`c34/empty-prefix-callback-value-charge-full-proofs.json`),
    60 nonencoding keys. Bind and actually execute all 60 through the real
    Host facade. A failure here is a shape failure: STOP with the concrete
    blocker; do not swap in another row.
  - Boundary: `P1556` — `.temp/g15-c2/c10/script-boundary-proof.json` (sha256
    `c593907dba5a082702fbe24425a34d0dccaf15b6964a73300d55e32a6d279e04`).
  - NoObservableTail: `P0011` —
    `.temp/g15-c2/c17/retained-option-arity-and-retry-revisit-full-proofs.json`
    (sha256
    `57315416ea214c70598741ef8dc2e4b3f8ac88a2b3c4edf79b28564d20b2b81c`).
  - Unreachable: `P0496` — `.temp/g15-c2/c68/p0496-closure-freeze.json`
    (sha256
    `a36d25977531686e805a8372e3a708c8bbc9a20b909e3ab4c421902b3aeb2639`).
  - Transferred: `P0500` — the only Transferred row whose cited packet is
    present (`.temp/g15-c2/c10/transfer-recheck.json`), 50 keys. It
    calibrates only if that packet authenticates its accepted disposition; if
    it does not, the row becomes a blocked record with the mismatch named and
    the class is not positively calibrated.
  - Identity: no row; all five cited packets are absent.

  Each present packet must authenticate its row's pin, row, verdict, target
  and counts before it calibrates. A selected row whose packet fails to
  authenticate becomes a blocked record with the mismatch named — not a STOP,
  and never a positive calibration.
- **Q2 — the absent packets.** For every absent row — the 9 Equal rows' 600
  Execute keys, the 593 Transferred rows' 34,950 proof keys and the 5
  Identity rows' 185 proof keys — the binding or certificate still imports
  from the pinned C68 ledger with its pin, row, verdict, target, scope and
  dependency, and its duty is `PortBlocked` with the exact missing paths:
  - Execute keys: `PortBlocked(continuation2/equal-decisions.tsv,
    continuation2/equal-real-receipts.tsv)`; never bound, never executed,
    never credited. The campaign's "every active Execute key" means the other
    9,905 keys, each bound and executed at multiplicity one.
  - Proof rows: `PortBlocked(continuation2/transfers.tsv)` for Transferred and
    `PortBlocked(continuation2/identity-decisions.tsv,
    continuation2/identity-receipts.tsv)` for Identity; never `SAME`, never an
    execution. Certificates imported stay 749 rows / 43,390 keys, of which
    151 rows / 8,255 keys are verified by a present packet (Boundary 20/735,
    NoObservableTail 126/7,260, Unreachable 4/210, Transferred 1/50) and 598
    rows / 35,135 keys are blocked records (Identity 5/185, Transferred
    593/34,950). Print both totals.
  - Neither blocked class may be reported as positively calibrated. For a
    class with no surviving packet, the shape calibration is its committed
    exactness test — the blocked record carries pin, row, verdict, scope,
    dependency and the named paths — plus that class's own falsifier, proven
    red for its named predicate (an unarmed or wrong-row Identity survivor; a
    removed transfer target or a rootless cycle). A non-crediting validator
    control built only from committed inputs may stand in for the
    well-formed-record half.
  - Do not reconstruct, re-derive, re-pin or bless history, and do not run a
    guest to manufacture a replacement packet. Full CM green stays false
    until a reviewed evidence program supplies these packets; nothing here
    authorizes that program.
  - No committed standing assertion reads a `.temp` file. Any packet replay
    you perform is a scratch run recorded with its sha256 in the report; the
    standing tests read only committed inputs.

## Continuation 2 ruling (director) — the receipt-charge observer is an authorized production prerequisite

Your second STOP is accepted: correct under the hard rule, and the blocker is
real. I verified each element in the tree: `CallbackValue.receipt` is private
(`accounting_mode/bridge.rs:9`), `RetainedValues.receipt` is `pub(super)`
(`accounting_mode/retained.rs:13`), `Mode` is `pub(in crate::runtime)`
(`accounting_mode/lease.rs:129`), and no direct receipt accessor exists. The
historical direct read lives only in the scratch instrumented kernel
(`.temp/g15-c2/c34/prepare_callback_charge.py`). No row is swapped, no ledger
total substitutes for the charge, and the `32/0` omission discriminator is
preserved. This section authorizes the production prerequisite and rewrites
the scope by exactly four files; every other ruling and gate stands.

- **Authorized production scope (the only additions to the Scope list):**
  - `crates/rum-runtime/Cargo.toml` — add the feature
    `test-receipt-observers = []`.
  - `crates/rum-runtime/src/runtime/accounting_mode/retained.rs` — under that
    feature, one `#[doc(hidden)]` inherent method
    `actual_receipt_bytes(&self) -> Option<usize>` on
    `RetainedValues<'_, AccountingShadow>`, returning
    `AccountingShadow::bytes(&self.receipt)`; if the disabled arm needs the
    same call, the parallel impl for `Unarmed` returns `None`.
  - `crates/rum-runtime/src/runtime/accounting_mode/bridge.rs` — the same
    method on `CallbackValue<'_, '_, AccountingShadow>`, returning
    `AccountingShadow::bytes(&self.receipt)` with the same `Unarmed` handling
    if needed.
  - `crates/rum-stdlib/Cargo.toml` — add `"test-receipt-observers"` to the
    `rum-runtime` dependency's feature list (line 28, beside
    `test-builtin-memory-growth` and `test-continuation-slots`).

  Shape as code:

  ```rust
  #[cfg(feature = "test-receipt-observers")]
  #[doc(hidden)]
  impl RetainedValues<'_, AccountingShadow> {
      pub fn actual_receipt_bytes(&self) -> Option<usize> {
          AccountingShadow::bytes(&self.receipt)
      }
  }
  ```

  and the parallel impl for `CallbackValue<'_, '_, AccountingShadow>`. A
  concrete impl is required: it must be callable from the `rum-stdlib`
  integration test without touching anything else. If it cannot compile or
  cannot be called from there, STOP again with the exact error; do not widen
  the scope.
- **What must not change:** the `Mode` and `RetentionMode` traits and their
  visibility, every existing field, every existing function body and every
  call site; no `unsafe`, no new dependency, no `Cargo.lock` change, no new
  warning. The accessors are read-only and appear only under the feature; a
  build of `rum-runtime` with default features must not contain them.
- **No timing A/B, and why.** The hot-path A/B rule exists to catch changed
  emitted code. This change adds dead getters under an off-by-default feature
  and edits no existing line, so there is no emitted-code delta to measure
  and a timing comparison would compare identical code. The gate is
  deterministic instead: the diff adds only feature-gated lines; no code in
  `crates/rum-runtime` calls the accessors; `cargo build -p rum-runtime`
  (default features) is green. Report the accessor's presence or absence by
  symbol inspection where the symbols materialize, and say so plainly if they
  do not — never claim a check you could not run.
- **Maintenance now triggers.** Any edit under `crates/` expires the
  retention fingerprints and the ownership-graph cells. Replay them exactly
  as the Maintenance section says, quote every verdict line, and treat a new
  function entering the ownership-graph inventory as a replay, never a hand
  edit.
- **The row is unchanged.** `P0001` keeps all 60 keys, its 60 frozen plans
  and 84 planned pauses, and the two observer points — `callback-value`
  before adoption and `resumed-value` after `resume_values` and `push`. The
  own-omission discriminator stays `C34ActualCallbackReceiptBytes.0` 32/0.
  A ledger total, a public scalar, a saved aggregate or a skipped key is not
  a substitute.
- **Gates unchanged:** the six shape tests, both-arm falsifiers, the
  campaign, the acceptance suite, the report format and every hard rule.
