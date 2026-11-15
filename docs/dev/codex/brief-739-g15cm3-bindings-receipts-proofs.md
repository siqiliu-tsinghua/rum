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

## Continuation 3 ruling (director) — the observer is unconditional; the campaign continues past the named blockers

Your STOP is accepted: the campaign stop-loss fired correctly, and the
certificate failure is real, not an ordering artifact. I reproduce it exactly —
`rule boundary impl changed:
rum_runtime::runtime::accounting_mode::retained::RetainedValues <'_ ,
AccountingShadow >`, same at one thread and in both feature arms. The
maintenance work is preserved and no cell was hand-edited. This section rules
Q4–Q6 and defines the remainder; everything else in this brief stands.

- **Q4 — the observer design is revised; the audit predicate is not touched.**
  `rules.rs` and `option_boundaries.rs` require boundary declarations to be
  unconditional, and a cfg-dependent impl can never be audited this way.
  Widening that predicate would entangle the paused retention line with this
  branch, so the feature is withdrawn instead:
  - remove `test-receipt-observers` from `crates/rum-runtime/Cargo.toml` and
    from `crates/rum-stdlib/Cargo.toml`;
  - make both accessors unconditional, `#[doc(hidden)]`, read-only, with no
    `unsafe` and no call site outside the oracle test:

  ```rust
  #[doc(hidden)]
  impl RetainedValues<'_, AccountingShadow> {
      pub fn actual_receipt_bytes(&self) -> Option<usize> {
          AccountingShadow::bytes(&self.receipt)
      }
  }
  ```

  and the parallel impl for `CallbackValue<'_, '_, AccountingShadow>`. The
  "no default-path cost" rule is about runtime cost: an uncalled getter has
  none, and the audit records it like every other boundary declaration. My
  earlier "a default build must not contain them" is superseded by this
  ruling, for this reason.

  **First action:** land exactly this change and rerun the failing
  `local_production_acceptance` in both arms and at one thread. If it still
  fails for a reason not caused by the cfg, STOP with the exact error before
  porting anything else.
- **Q5 — the 13 unresolved-target rows are `PortBlocked`; the campaign
  continues.** `P0020` (60 keys), `P0028` (60), `P0327` (50), `P0479` (50),
  `P1310` (60), `P1331` (55), `P1333` (60), `P1335` (60), `P2596` (30),
  `P2602` (30), `P2603` (30), `P2604` (210), `P2642` (30) — 13 rows / 785
  keys whose target is `-` in the accepted `aliases.tsv`. Each is
  `PortBlocked(unresolved CM1 identity target)`, exactly named, never guessed
  by suffix, nearest-line or observation; a later reviewed CM1 crosswalk may
  resolve them. With the 9 absent-packet rows (`P0018`, `P0580`, `P0709`,
  `P0790`, `P0795`, `P0798`, `P0801`, `P2287`, `P2288`; 600 keys) the Execute
  side is 22 blocked rows / 1,385 keys and 153 bindable rows / 9,120 keys —
  `P0001` and `P0002` (120 keys) are already bound and executed, so 151 rows /
  9,000 keys remain. A batch that yields zero evidence *because its rows are
  `PortBlocked` with a named cause* does not count toward the two-batch
  stop-loss; continue with the remaining rows.
- **Q6 — the C1 promotion stays as it is.** The authenticated promotion is
  imported; the missing original program/tuple/request/structured-difference
  details stay `PortBlocked` with the absent path named. No fresh
  allowed-DIFF baseline and no reconstruction.
- **The remainder.** After the accessor fix and the green certificate:
  - complete every remaining bindable family (151 rows / 9,000 keys) with
    exact row/key progress and every authenticated attempt retained;
  - authenticate the 146 present-packet proof duties / 8,030 keys, and record
    `P0011` as a current-source blocker with its own predicate; the 598 absent
    records / 35,135 keys and the 13 unresolved rows stay named;
  - rerun the six shape tests and the both-arm falsifiers;
  - rerun the maintenance replays after the accessor change (the retention
    fingerprints and the ownership-graph cells), once, from the final source;
  - then measure the complete schedule and run the acceptance suite. The
    1,500 s bound stands: if the complete schedule exceeds it, STOP with the
    measured build+run data and the sharing analysis — never sample, never
    drop a context, never substitute a historical timing.
  - Report every count, the blocked totals (Execute 22 rows / 1,385 keys;
    proof 598 rows / 35,135 keys absent, 146 rows / 8,030 keys pending
    authentication, `P0011` blocked), and the exact pending state if you stop
    again.
- **The tree is the state to continue from.** The partial work already in the
  tree (62 files) is kept; do not restart from scratch and do not copy a
  stale generated cell.

## Continuation 4 ruling (director) — the adapter source wall is opened by test-only source reuse; the campaign continues to the end of the active bag

**2026-11-14 — brief 739's fourth run implemented the continuation-3 getter
ruling and expanded the layer to nine complete rows, then stopped at the first
source-scope wall. The STOP is accepted; the scope is now authorized; continue.**
Run `cdx`, unit `brief-739-cdx-20261114-201343-315874`, rc 0, peak
15,786,823,680 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 13,465,975 B; 170 files (162 added, 8 modified,
0 deleted); report 12 PASS / 3 FLAGGED / 1 FAIL, one FLAGGED measurement, 3
disclosures, 4 deviations, 1 question; `scripts/report.py check 739` passes
with 3,016 citations.

**What landed and what I reproduced myself.**
- The two `#[doc(hidden)] actual_receipt_bytes` accessors are unconditional and
  append-only (`retained.rs`, `bridge.rs`), `test-receipt-observers` is gone,
  and no `Cargo.toml`/`Cargo.lock` line changed. `cargo fmt --check` rc 0 (my
  run).
- 21,010 binding rows (10,505 required + 10,505 declarations): 525 bound and
  9,980 blocked. The nine complete rows are `P0001` 60, `P0002` 60, `P0145` 30,
  `P0214` 30, `P0216` 60, `P0239` 55, `P0245` 55, `P0248` 120, `P0249` 55 =
  525 keys over 7 fixture families.
- 749 proof records / 43,390 keys with the exact class counts; 130 packets
  authenticated / 7,425 keys; 105 current-source verified / 5,965 keys; 21
  present duties / 830 keys pending; 598 absent records / 35,135 keys; 5
  known-loss records (3 Lost + 2 Promoted) with the C1 baseline details
  explicitly `PortBlocked`.
- My shape runs, both feature arms, six tests each: `cm_bindings_exact`,
  `cm_receipts_exact`, `cm_proof_certificates_exact`, `cm_loss_baselines_exact`
  and the two negative-control suites pass (`CM RECEIPTS PASS calibration=P0001
  rows=9 families=7 bound=525 executed=525 multiplicity=1 actual_pauses=940
  lifecycle=1318 campaign_remaining=8595 PortBlocked=1385`;
  `CM PROOFS PASS imported=749/43390 packet_authenticated=130/7425
  absent=598/35135 current_verified=105/5965`); the 18 execution and 15
  proof/loss falsifiers reject at their named predicates.
- My certificate runs: `local_production_acceptance` green in both arms at one
  thread with zero rejections.
- Spot-checks: the sample `VerifiedPacket` rows' packet sha256 and source-file
  sha256 match the table and the filesystem; the nine workspace failures in
  both worker logs are the sandbox loopback `Operation not permitted` guard at
  `zeromq_native_test_support.rs:346`, not a tree red.
- My host workspace runs, seven documented skips: default 70 suites,
  4,303 passed / 0 failed / 64 ignored (6.99 GB peak) and no-default the same
  4,303 / 0 / 64 (7.29 GB peak), rc 0 — exactly the worker's 4,294 passed plus
  the nine loopback tests, which pass outside the sandbox. The switched LP
  targets pass 5/5 in both arms (`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`).

**Q7 ruling — test-only source reuse is authorized for downstream-crate source
placements.** The STOP is correct: `P2611`/`P2612` bind
`crates/rum-jupyter/src/adapter/worker.rs:350:::run_segment@358`, whose
unchanged source imports `crate::{KernelExecuteOutcome, KernelExecuteRequest,
KernelMessageHeader}` and `super::{render, input_retirement}`; the worker's
probe fails with E0432/E0433 through the oracle files alone, its single edit
was restored byte-exact, and the two 30-key packets were authenticated with
zero bindings and zero receipts. I verified independently that the fix
compiles and drives: at the oracle test root, `extern crate self as rum_host;`
plus `#[path]` includes of `crates/rum-jupyter/src/model.rs` (with root
re-exports) and `crates/rum-jupyter/src/adapter/mod.rs` (whose `worker`,
`execute`, `render`, `iopub` and `input_retirement` children resolve under it)
compile the unchanged adapter under `cargo check` and under `cfg(test)`; a
probe test constructs the included `HostSession`, wraps it in the unchanged
`KernelAdapter` and calls `kernel_info` — 13 tests pass under
`.temp/director/probe-adapter/` — and `extern crate self as ...` works in a
test binary.

Authorized (this supersedes the earlier Scope file list where they differ):
- Editing the test root `crates/rum-stdlib/tests/suspension_oracle.rs` and
  adding test-only modules under `crates/rum-stdlib/tests/suspension_oracle/`
  for the adapter driver.
- `#[path]`-including unchanged sources of downstream workspace crates when a
  required key's named source placement needs them: today
  `crates/rum-jupyter/src/model.rs` and `crates/rum-jupyter/src/adapter/{mod,
  worker,execute,render,iopub,input_retirement}.rs` for `P2611`/`P2612`, and,
  if reached, `crates/rum-repl/src/request.rs` with
  `crates/rum-terminal/src/lib.rs`. The duty is general: any required key whose
  named placement is a workspace-crate source may be reached this way, with the
  exact included set disclosed. `host_facade.rs` and every included source stay
  byte-for-byte unchanged.
- The minimal crate-root re-exports and `extern crate self as <crate>` aliases
  needed to make those unchanged sources resolve.

Forbidden, unchanged: editing, copying or rewriting any included production
source; any `Cargo.toml`/`Cargo.lock` change; adding `rum-jupyter` or
`rum-repl` as a dependency (it would compile a second host stack inconsistent
with the included facade); stubs, shims or re-typed stand-ins for production
types; substituting Host eval for adapter execution.

Requirements:
- The receipts for these rows come from the actual execution of the included
  source at multiplicity one, through the same private constructor, the same
  13-channel set and the same lifecycle protocol as every other row;
  observations crossing the adapter's execute-worker thread must be captured
  from the executing thread (join/channel), never assumed.
- Before the campaign, compile-probe every distinct source placement of the 144
  remaining rows — 56 distinct files today (rum-runtime 40, rum-stdlib 13,
  rum-repl 1, rum-jupyter 1, rum-host 1) plus the 25 non-source routes over
  1,370 rows — and report the reachability table; a placement that cannot
  compile under this rule is `PortBlocked(<exact unresolved path/error>)` and a
  zero-evidence batch of named blockers does not count toward the stop-loss; a
  need for a production change or a Cargo edit is still a STOP.
- Continue the campaign over the remaining 144 rows / 8,595 keys; authenticate
  and current-verify the 21 present proof duties / 830 keys; keep the 598
  absent records / 35,135 keys, the 25 authenticated-but-current-blocked
  proof records (15 unresolved targets / 880 keys, 10 source mismatches /
  580 keys) and the 22 director-named Execute blockers / 1,385 keys named with
  their exact predicates.
- Then measure the complete active schedule and run the acceptance suite under
  the 1,500 s bound; over it, STOP with the measured build+run data and the
  sharing analysis — never sample, never drop a context, never substitute a
  historical timing.

C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner is promoted,
no encoding key runs, `CHANGELOG.md` and S10 stay blocked. The partial tree is
the state to continue from; do not restart and do not hand-edit a generated
cell.

## Continuation 5 ruling (director) — the REPL placement is blocked by a private crate path, not by the missing dependencies; the campaign proceeds over the remaining 8,535 keys with no Cargo edit

**2026-11-14 — brief 739's fifth run ran continuation 4's placement preflight,
accounted for all 56 source placements and 25 non-source routes, compiled 55
in both feature arms, and stopped on `P2647`/`P2648` before any campaign
batch. The STOP is accepted; Q8 is answered by naming those 60 keys
`PortBlocked`.**
Run `cdx`, unit `brief-739-cdx-20261114-225340-428863`, rc 0, peak
1,014,820,864 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 1,840,024 B; zero retained changes — the
1,819-file entry manifest equals the final manifest byte for byte
(`.temp/brief-739/{entry,final}-hashes.json`), and `Cargo.toml`, `Cargo.lock`
and `crates/rum-stdlib/Cargo.toml` are unchanged; report 7 PASS / 1 FLAGGED /
1 FAIL, one FLAGGED measurement, 2 deviations, 1 question; 90 citations
verified by `scripts/report.py check 739`. The only two temporary edits
(`tests/suspension_oracle.rs` probe includes and a temporary
`source_placements.rs`) were restored or removed and are disclosed in the
report; no source or inventory delta remains, so the maintenance replay is
correctly skipped by hash identity.

**What the preflight established, as I reproduced it.**
- `source-reachability.tsv` (`e2f75657…`): 56 rows, 55
  `CompiledSourceOnly`, 1 `PortBlocked`; every compiled row names its
  dependency-file evidence in both arms; `actual_executions=0`.
- `non-source-reachability.tsv` (`e47a7b39…`): 25 routes covering exactly
  1,370 key rows, all 25 owning sources compile, all route instantiations and
  executions `unrun` — a compile-only preflight, not evidence.
- `included-production.tsv` (`0f82bcf8…`): 17 unchanged production files
  (host-facade children, Jupyter `model`/`adapter` children,
  `rum-terminal/src/lib.rs`); I re-sha256'd samples including
  `rum-terminal/src/lib.rs` (`f40f6874…`) and the two Jupyter files, all
  matching.
- The ledger is unchanged: 525 bound keys; 8,595 active keys; 1,385
  director-named blocked keys; 749 proof records / 43,390 keys with 130
  packets authenticated / 7,425 keys and 105 current-source verified / 5,965;
  21 present duties / 830 keys; 598 absent / 35,135; 5 known-loss records.
  No campaign batch, no receipt, no proof credit, no full CM green.
- The blocked placement reproduces E0433 `rustix`, E0432/E0433
  `signal_hook` in both arms (`placements-request-{default,no-default}.log`),
  and the unchanged full `lib.rs` include additionally reports
  `unresolved import crate::resolve_completeness`
  (`placements-full-default.log:27`).

**Q8 ruling — the placement is `PortBlocked`, and no Cargo edit is
authorized.** The rows bind `crates/rum-repl/src/lib.rs:113:Repl<R,W>::run@152`.
Including the unchanged `lib.rs` under the oracle's `cfg(test)` compiles
`crates/rum-repl/src/completeness.rs:581`'s `#[path = "completeness_tests.rs"]
mod tests`, whose `use crate::resolve_completeness` names the plain-private
`fn resolve_completeness` at `lib.rs:306`. Inside the oracle crate `crate::`
is the oracle root, and the root cannot re-export a child module's private
item; satisfying that import would need a stub or an edit to the included
source, both forbidden. The missing `rustix`/`signal_hook` dev-dependencies
are a second, real blocker, but adding them — even the union of rum-repl's
`[target.'cfg(unix)'.dependencies]` and `[target.'cfg(unix)'.dev-dependencies]`
specs, `rustix 1.1.4` with `std,termios,fs,event,pipe` plus `pty` and
`signal-hook 0.3.18` — cannot fix the private `crate::` import, so the Cargo
change would yield no evidence. The minimal `request`/`terminal` layout does
compile against them, but it is not the named placement and cannot produce
`Repl::run` receipts; driving it instead would be a substitute execution and
is refused. Per continuation 4's rule the placement is named:

`PortBlocked(crates/rum-repl/src/lib.rs:113:Repl<R,W>::run@152 — included
cfg(test) child completeness_tests.rs imports the private
crate::resolve_completeness, unresolvable at the oracle root; rustix and
signal_hook are also absent from the rum-stdlib integration target)`

This sets the Execute ledger to 24 blocked rows / 1,445 keys (the 22
director-named / 1,385 plus `P2647` 30 and `P2648` 30) and 142 active rows /
8,535 keys remaining; a zero-evidence batch of named blockers does not count
toward the stop-loss. If the worker can exhibit a compiled include of the
unchanged `lib.rs` under `cfg(test)` with no source edit or copy, no Cargo
change and no stub, the blocker is lifted and the rows return to the campaign;
otherwise the block stands.

**Continuation 5.** Continue the campaign over the remaining 142 rows /
8,535 keys and the 25 non-source route instantiations (1,370 key rows), with
the same multiplicity-one receipt discipline, the same 13-channel and
lifecycle protocol, and the excluded rows named; authenticate and
current-verify the 21 present proof duties / 830 keys; keep named the 598
absent records / 35,135 keys, the 25 authenticated-but-current-blocked proof
records (15 unresolved targets / 880 keys, 10 source mismatches / 580 keys),
the 22 director-named Execute blockers / 1,385 keys and the new
`P2647`/`P2648` block. The preflight tables may be reused only by source-hash
identity and remain compile-only claims: every row's execution is still owed,
and a placement that compiles but cannot execute is `PortBlocked(<exact
cause>)` under the same rule. Then measure the complete active schedule and
run the acceptance suite under the 1,500 s bound — over it, STOP with the
measured build+run data and the sharing analysis; never sample, never drop a
context, never substitute a historical timing. No `Cargo.toml`/`Cargo.lock`
change, no production edit, no dependency, no stub, no Host-eval
substitution; every included source stays byte-for-byte unchanged.

C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner is promoted,
no encoding key runs, `CHANGELOG.md` and S10 stay blocked. The partial tree is
the state to continue from; do not restart and do not hand-edit a generated
cell.

## Continuation 6 ruling (director) — the C29 accepted-request observer is authorized by the Q4 precedent; P1562/P1563 bind and execute through the real routes, and the campaign continues

**2026-11-15 — brief 739's sixth run carried the campaign from 9 rows / 525
keys to 58 rows / 3,095 keys, authenticated every present proof duty, and
stopped at the C29 two-bit private-observer wall. The STOP is accepted; Q9 is
answered by authorizing one narrow read-only accessor, and the brief is
re-dispatched.**
Run `cdx`, unit `brief-739-cdx-20261114-232115-435918`, rc 0, peak
15,330,988,032 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 4,400,252 B; 721 recorded changes (707 added,
14 modified, 0 deleted), the two temporary probe edits restored and
disclosed; report 18 PASS / 4 FLAGGED / 1 FAIL, 4 deviations, Q9; 13,065
citations verified by `scripts/report.py check 739`. No production source,
manifest or lock hash changed (the audit's 1,309 production Rust files and 12
manifests match entry), so no production A/B was owed or claimed.

**What landed, as I reproduced it from the tables and logs.**
- 58 complete Execute rows / 3,095 keys now carry fresh exact receipts in
  both feature modes (49 rows / 2,570 keys added), with 58 rows / 3,155
  actual pauses and their lifecycle observations; 93 rows / 5,965 keys
  remain, including all 25 non-source routes / 1,370 key rows; the 24
  director-excluded rows / 1,445 keys stay named.
- The proof bag is unchanged at 749 certificates / 43,390 keys. All 21
  present duties / 830 keys were authenticated: 151 rows / 8,255 keys are
  now packet-authenticated, 115 / 6,270 are current-source verified, and 36 /
  1,985 fail closed at current identity (22 unresolved targets / 1,155 keys
  and 14 source-dependency mismatches / 830 keys); the 598 absent records /
  35,135 keys remain named. I independently re-hashed all 57 distinct packet
  files named by the 151 authenticated rows (all match) and every
  `source_files` pin on the 115 verified rows (all match; the mismatches fall
  exactly on the 14 source-dependency rows and three already-unresolved
  rows).
- The five known-loss records replay with 3 immutable differences and 2 C1
  promotions; the 68 retention fingerprints and 4 promotion cells were
  regenerated and installed mechanically with every non-cell column
  equality-checked; the ownership replay reproduced all six inventories with
  no new row (`.temp/brief-739/c5-install-retention-cells.log`,
  `c5-install-promotion-cells.log`, `c5-ownership-reconcile.log`).
- The two stale-C01 replay reds are ordering artifacts (the cells were
  installed after the replay); both reruns at one thread pass with zero
  rejections. The first six switched-LP runs used binary names as test-name
  filters and ran zero tests; the corrected commands pass 5/5 in both arms,
  and the empty runs receive no credit. The typed-slot raw 10/2/1 sums
  contain the two expected child failures of the scratch-copy falsifiers; the
  top-level target is 10 passed / 0 failed / 1 ignored in both arms.
- The complete 9,060-key schedule is unmeasured because the run stopped at
  the C29 wall: the partial 3,095-key shape gate took 269.650 s default and
  271.773 s no-default, and the 1,500 s complete-schedule bound remains
  unfulfilled. That is the correct state under the STOP, not a substitute
  measurement.

**Q9 ruling — one narrow C29 accessor is authorized; the keys stay real.**
`P1562`/`P1563` (30 keys each) require the accepted text/expr request routes
and the native C29 row; their authenticated matrix agrees on every other
field and differs only in `C29TerminalAcceptedRequestState` (0,0 versus 1,1).
`Session.active_accepted_request` and `Session.request_interrupt_control` are
`pub(in crate::runtime)` (`crates/rum-runtime/src/runtime/model/session.rs:119,122`)
and `RequestInterruptState::is_idle` is runtime-private
(`crates/rum-runtime/src/runtime/model/request_cancellation.rs:200`); both
oracle-root probes fail E0616, and no public cancellation read substitutes
for the two original bits. This is the same shape as the accepted P0001
receipt accessor (ruling 87330bdc, made unconditional by e73df1f1, and
explicitly justified at brief-739's Q4: an uncalled getter has no
default-path cost and the audit records it like every other boundary
declaration).

Authorized, and nothing more:
- One unconditional `#[doc(hidden)]` read-only inherent method on the
  runtime `Session`, returning exactly the two bits
  `active_accepted_request.is_some()` and
  `!request_interrupt_control.borrow().is_idle()`; the worker chooses the
  file. No field visibility change, no body or caller change, no new
  dependency, no `Cargo.toml`/`Cargo.lock` edit, no `unsafe`.
- The method is declared unconditionally (no feature or `cfg(test)` gate):
  the rule-boundary audit rejects cfg-dependent impls. It has no call site
  outside the oracle test, and the production audit records it. The Q4
  precedent replaces the hot-path A/B with the deterministic gate: no
  emitted-code delta, default-feature build green, and a reported symbol
  inspection; if the change emits a production call site or any body change,
  STOP.
- First action: land exactly this and rerun `local_production_acceptance` in
  both arms at one thread after the retention/ownership maintenance replays.
  If the audit rejects the new impl, STOP with the exact error — never widen
  the predicate.
- Then bind and execute `P1562` and `P1563` through the actual accepted
  text/expr routes, observing the two bits from the executing thread at the
  terminal request point before any fresh request, with the full ten-field
  native row and the own-omission 0,0-vs-1,1 discriminator, at multiplicity
  one. A public scalar, a fresh-request probe, a saved aggregate or a
  skipped key is not an observation.

**Continuation 6.** Continue the campaign over the remaining 93 rows / 5,965
keys, beginning with the 60 C29 keys, and the 25 non-source route
instantiations (1,370 key rows); keep the 36 current-blocked proof records,
the 598 absent records / 35,135 keys and the 24 named Execute blockers /
1,445 keys named with their exact predicates. Then measure the complete
active schedule and run the acceptance suite under the 1,500 s bound — over
it, STOP with the measured build+run data and the sharing analysis; never
sample, never drop a context, never substitute a historical timing. If any
other production change becomes necessary, STOP; this authorization covers
the single C29 accessor only.

C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner is promoted,
no encoding key runs, `CHANGELOG.md` and S10 stay blocked. The partial tree is
the state to continue from; do not restart and do not hand-edit a generated
cell.

### Continuation 7 — seventh run: the C29 accessor lands and 60 keys execute; the resuming-driver observer is authorized

**2026-11-15 — brief 739's seventh run executed continuation 6. The
authorized C29 accessor landed, both C29 rows were bound and executed in
both feature arms, and the run stopped at P1577, whose authenticated
discriminator needs two more private `Session` reads. The STOP is accepted
and Q10 is ruled: one more narrow read-only `Session` accessor is authorized
under the same precedent, and the campaign continues.**
Run `cdx`, unit `brief-739-cdx-20261115-014918-518988`, rc 0, peak
15,983,349,760 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 2,252,569 B; 23 recorded changes (12 added,
11 modified, 0 deleted), the temporary P1577 probe restored byte-exact;
report 14 PASS / 5 FLAGGED / 1 FAIL, 3 disclosures, 5 deviations, Q10;
17,181 citations verified. The only production change is the authorized
accessor; no manifest or lock file changed.

**What landed.** The accessor is append-only at the end of
`crates/rum-runtime/src/runtime/model/session.rs` (10 lines, method at
`:201`): one unconditional `#[doc(hidden)]` read-only inherent
`Session::actual_accepted_request_state(&self) -> (bool, bool)` returning
exactly `(active_accepted_request.is_some(),
!request_interrupt_control.borrow().is_idle())`. The entry bytes remain a
prefix; field visibility, existing bodies and callers are unchanged; the
default rlib emits exactly one new symbol; there are zero production call
sites and one oracle call site. The deterministic gate passed and the
unrun machine-code byte comparison is disclosed, as Q9 required.

`P1562` and `P1563` are now complete rows: 60 keys bound and executed in
both arms at multiplicity one, 42 actual pauses and 72 lifecycle
observations per row, through the real accepted text/expr routes
(`accept_cancellable_request` + `eval_accepted_request`, and
`lower_request` + `eval_accepted_expr_request`), with the two-bit read on
the executing thread at the terminal return before any projection or fresh
request. The ten-field native row `C29TerminalAcceptedRequestState` is
`[0,0]` on both sides, and four new falsifiers reject the 0,0-vs-1,1 and
both-stale discriminations by `Native`.

Campaign totals: 58 rows / 3,095 keys → 60 rows / 3,155 keys over 29
families (5,252 actual pauses, 7,930 lifecycle observations); 91 active
rows / 5,905 keys remain, of which `P1577` is 1 row / 30 keys and 90
unported rows / 5,875 keys; the 24 director-excluded rows / 1,445 keys
stay named. The proof ledger is untouched: 749 rows / 43,390 keys imported,
151 / 8,255 packet-authenticated, 115 / 6,270 current-source verified,
36 / 1,985 current blocked, 598 / 35,135 absent, five known-loss records.
The 68 retention fingerprints and four promotion cells were regenerated
and mechanically installed twice — only the fingerprint column in
`local-contracts.tsv`, only the two instance columns in `promoted.tsv`,
every non-cell column equality-checked — the six ownership inventories
replayed unchanged, and the stale-C01 certificate reds before the install
are ordering artifacts; both one-thread certificate reruns pass.

**The STOP.** `P1577` (`C29ResumingDriverEvaluationStateMax`, 30 keys) needs
the exact post-request observation of `evaluation_depth` and
`evaluation_stack` at `Session::resume_suspended_slot`
(`crates/rum-runtime/src/runtime/suspension/core.rs:609@659`); both feature
probes fail E0616 on the `pub(in crate::runtime)` fields
(`session.rs:87,91`), and the row's authenticated own-omission is exactly
`1,0` versus `0,0`. A second production change was outside continuation 6's
authorization, so the run stopped with the 30 keys retained as active
requirements. Accepted.

**Q10 ruling — one more narrow accessor, then the keys execute.**
Authorized, and nothing more:
- One unconditional `#[doc(hidden)]` read-only inherent method on the
  runtime `Session`, called `actual_evaluation_state`, returning exactly
  `(self.evaluation_depth.get(), self.evaluation_stack.len())`; no field
  visibility change, no body or caller change, no new dependency, no
  `Cargo.toml`/`Cargo.lock` edit, no `unsafe`.
- The same audit gate as Q9: append-only source, prior bytes a prefix,
  exactly one new symbol, zero production call sites, the declaration
  recorded by the production audit. A cfg-dependent impl is rejected; an
  audit rejection is a STOP, never a widened predicate or a hand-edited
  cell.
- First action: land exactly this, replay the retention/ownership
  maintenance, and rerun `local_production_acceptance` in both arms at one
  thread.
- Then bind and execute `P1577`'s 30 keys through the real route named by
  its authenticated consumer route, observing the two values from the
  executing thread at the exact point its matrix names, with the ten-field
  native row and the own-omission `1,0`-versus-`0,0` discriminator at
  multiplicity one. A cached scalar, a projection, a substituted guest
  evaluation or a skipped key is not an observation; the falsifier must
  reject the discrimination by `Native`.

**Continuation 7.** Continue the campaign over the remaining 90 rows /
5,875 keys and the 25 non-source route instantiations (1,370 key rows);
keep the 36 current-blocked proof records, the 598 absent records / 35,135
keys and the 24 named Execute blockers / 1,445 keys named with their exact
predicates. Then measure the complete 9,060-key active schedule and run the
acceptance suite under the 1,500 s bound — over it, STOP with the measured
build+run data and the sharing analysis; never sample, never drop a
context, never substitute a historical or partial timing. If any other
production change becomes necessary, STOP; this authorization covers the
single resuming-driver accessor only.

C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner is
promoted, no encoding key runs, `CHANGELOG.md` and S10 stay blocked. The
partial tree is the state to continue from; do not restart and do not
hand-edit a generated cell.

### Continuation 8 — eighth run: P1577 executes (61 rows / 3,185 keys); the feature-isolated build lane is authorized for P0543

**2026-11-15 — brief 739's eighth run executed continuation 7. The
authorized resuming-driver accessor landed, P1577's 30 keys were bound and
executed in both arms, and the run stopped at P0543, whose authenticated
unchecked-dispatch route needs a build configuration the mandated oracle
build cannot express. The STOP is accepted and Q11 is ruled: one
feature-isolated build lane is authorized, and the campaign continues.**
Run `cdx`, unit `brief-739-cdx-20261115-033757-605315`, rc 0, peak
15,162,417,152 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 2,160,394 B; 23 recorded changes (12 added,
11 modified, 0 deleted); report 17 PASS / 2 FLAGGED / 2 FAIL, 5
disclosures, 4 deviations, Q11; 9,086 citations verified. The only
production change is the authorized accessor; no manifest or lock file
changed.

**What landed.** The second accessor is append-only at the end of
`crates/rum-runtime/src/runtime/model/session.rs` (168 added bytes over a
10,135-byte prefix; method at `:211`): one unconditional `#[doc(hidden)]`
read-only inherent `Session::actual_evaluation_state(&self) -> (usize, usize)`
returning exactly `(self.evaluation_depth.get(), self.evaluation_stack.len())`.
Field visibility, every existing body and caller, the manifests and the lock
file are unchanged; exactly one symbol was added; there are zero production
call sites; the machine-code byte comparison is disclosed unrun, as Q10
required.

`P1577` is now a complete row: 30 keys bound and executed in both arms at
multiplicity one, 72 actual pauses, 72 lifecycle observations and 132 direct
request-return state samples. The ten-field native row
`C29ResumingDriverEvaluationStateMax` is `[0; 10]` on both sides — the
authenticated packet's own value, checked against
`.temp/g15-c2/c29/logs/resuming-driver-saved-state-restoration-real-1.log` —
and four new falsifiers reject the `1,0`-versus-`0,0` discrimination, the
both-stale variant, a missing request-return sample and a sample that
disagrees with the saved maximum, all by `Native`.

Campaign totals: 60 rows / 3,155 keys → 61 rows / 3,185 keys over 30
families (5,324 actual pauses, 8,002 lifecycle observations); the 90
unported rows / 5,875 keys remain active, P0543 being one of them (1 row /
55 keys); the 24 director-excluded rows / 1,445 keys stay named. The proof
ledger is untouched: 749 rows / 43,390 keys imported, 151 / 8,255
packet-authenticated, 115 / 6,270 current-source verified, 36 / 1,985
current blocked, 598 / 35,135 absent, five known-loss records. The 68
retention fingerprints and four promotion cells were regenerated and
mechanically installed, only the certificate cells changed and every
non-cell column was equality-checked; the six ownership inventories
replayed unchanged; the stale-cell reds before each install are ordering
artifacts.

**The STOP.** `P0543` (`unchecked-builtin-feature-driver`, 55 keys) needs
the unchecked builtin dispatch. Both mandated arms compile rum-runtime with
`test-builtin-memory-growth` on — it is an unconditional dev-dependency
feature of `crates/rum-stdlib/Cargo.toml:28`, `--no-default-features` does
not remove it, and `builtin_memory_growth_test_enabled()`
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:365`) returns true
from its `cfg!` early return before any environment read, so no env value
can reach the unchecked branch. The authenticated packet's required
configuration is both terms false, and the historical C28 driver produced
it by building the same `suspension_oracle` target from a copy whose nine
manifests had the feature removed (`unchecked-builtin-feature-driver-feature-changes.json`).
A Cargo change was outside continuation 7, so the run stopped with all 55
keys retained as active requirements. Accepted.

**Q11 ruling — one feature-isolated build lane, for P0543 only.**
Authorized, and nothing more:
- Build the same `suspension_oracle` test target from a content-addressed
  copy of the current workspace, assembled through `scripts/probe-build.py`,
  whose only differences from the workspace are the manifest feature
  entries required to make the oracle target's build graph free of
  `test-builtin-memory-growth` — the historical nine dev-dependency edits
  are the reference, `test-continuation-slots` stays as it is today so the
  oracle sources compile unchanged. Every non-manifest file in the copy
  must hash-match the workspace; a mismatch is a STOP.
- The copy's manifest diffs, tree hash, build artifact hash and build
  profile are recorded in provenance. No repository manifest or lock file is
  edited, no dependency is added, no production source changes, and the copy
  exists only under `.temp/`.
- First action: build the lane and prove the configuration both ways — the
  build graph (`cargo tree -e features` carries no
  `test-builtin-memory-growth`) and at runtime by a behavioral discriminator:
  the checked-only `NonAmplifying` enforcement must fire in the mandated
  arms and must not fire in the lane, its outcome recorded in the lane's
  receipts, with a falsifier proving the discriminator can fire. The copy's
  non-manifest files stay byte-identical to the workspace; no scratch-only
  production read is authorized. A build or audit failure is a STOP.
- Then bind and execute P0543's 55 keys through the real unchecked
  route at multiplicity one, with the same ten-field native observer and
  channel set as the main oracle. The lane's receipts carry a distinct
  third identity that binds the copy's tree hash and the manifest-diff
  digest; the two mandated arms and every existing receipt are unchanged.
- The complete active schedule and the acceptance suite still run under the
  1,500 s bound for the two mandated arms as before; the isolated lane's
  build and run are measured and reported separately and included in the
  acceptance accounting, with the totals disclosed. Over the bound, STOP
  with the measured data and the sharing analysis; never sample, never drop
  a context, never mix the lane's timing into an arm's.

**Continuation 8.** Bind and execute `P0508`'s 30 keys, whose packet already
authenticated in this run (30 programs, 207 direct state samples, own
omission `2,2`-versus-`0,0`) without a binding, then continue the campaign
over the remaining 89 rows / 5,820 keys and the 25 non-source route
instantiations (1,370 key rows); keep the 36 current-blocked proof records,
the 598 absent records / 35,135 keys and the 24 named Execute blockers /
1,445 keys named with their exact predicates, then measure the complete
schedule and run the acceptance suite as stated. If any other Cargo or production change becomes
necessary, STOP; this authorization covers the single P0543 lane only.

C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner is
promoted, no encoding key runs, `CHANGELOG.md` and S10 stay blocked. The
partial tree is the state to continue from; do not restart and do not
hand-edit a generated cell.

### Continuation 9 — ninth run: four rows execute (65 rows / 3,380 keys), the P0543 lane is proven both ways, and the option-matrix importer is the next wall

**2026-11-15 — brief 739's ninth run executed continuation 8. The isolated
P0543 lane was built from a content-addressed copy, proven by the build graph
and a behavioral discriminator, and used to execute its 55 keys through the
real unchecked route; `P0508` (30), `P0544` (55) and
`C10.prepared.profiled_consumer` (55) completed as well, carrying the campaign
from 61 rows / 3,185 keys to 65 rows / 3,380 keys over 34 families. The run
then stopped at the option matrix: two importer attempts added no binding,
receipt or proof evidence because the C6 plans inherit fixture descriptor
fields the scratch importer treated as complete. The STOP is accepted and Q12
is ruled: the continuation resolves that inheritance through the canonical
`logical_profile` plan/fixture behavior, and a director-authored adapter
manifest is declined.**
Run `cdx`, unit `brief-739-cdx-20261115-052344-694898`, rc 0, peak
14,680,698,880 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 4,098,133 B; 45 recorded changes (34 added,
11 modified, 0 deleted); report 14 PASS / 3 FLAGGED / 1 FAIL, 5 disclosures,
3 deviations, Q12; 22,325 citations verified. The changes are 34
content-addressed `.rum` programs, six test Rust files, three obligation
tables and the two mandatory retention maintenance tables; no production
source, repository manifest, lock, dependency or script changed.

**What landed.**
- **P0543** (55 keys): the lane was built from
  `.temp/brief-739/p0543-tree-4bf43439…` — 3,696 files whose only differences
  from the workspace are the nine dev-dependency manifests (the
  `test-builtin-memory-growth` feature removed, `test-continuation-slots`
  kept), with all 3,687 non-manifest files hash-matching the workspace; I
  re-hashed every one of them file by file and recomputed the tree hash and
  the artifact hash. `cargo tree -e features` on the lane carries no
  `test-builtin-memory-growth`; the release artifact is `68c7b6b4…` built at
  one codegen unit in 61.277 s. The behavioral discriminator observes the
  checked-only `NonAmplifying` enforcement through the real dispatch route as
  true in both mandated arms and false in the lane, forces it true in a
  fresh-process control in both builds, and its falsifier rejects
  `NonAmplifying` with the exact panic. The 55 keys execute in the lane at
  multiplicity one (`CM P0543 RECEIPTS PASS`), and every lane receipt carries
  the distinct third identity `feature=p0543-unchecked,
  build=sha256=68c7b6b4…;tree=4bf43439…;manifest-diff=a6f12578…;profile=release;codegen-units=1`;
  the mandated `feature=default` receipts keep their existing format. The
  native row `CM739UncheckedDispatchBuildConfiguration` is `[0; 10]` on both
  sides in both arms (220 rows per arm), with three falsifiers rejecting the
  own-omission, the checked-route substitution and the both-routes-checked
  variants by `Native`. The lane's provenance is recorded in the preserved
  identity, manifest-diff, file-hash and build-receipt JSON and in every live
  receipt identity; it is not inserted into a copied source table, which
  would change the tree being identified. The two earlier lane builds are
  preserved with `-1`/`-2` suffixes and are not relabelled as final.
- **P0508** (30 keys): 42 actual pauses, 72 lifecycle observations and 102
  direct request-return samples per arm; the row
  `C28BetweenRequestsEvaluationStateMax` is `[0; 10]` on both sides (120 rows
  per arm). The packet's 207 samples are its own authentication record, not
  an execution claim; the live observer samples at every request return (42
  pauses + 30 queued + 30 resumed = 102). Four Native falsifiers reject the
  `2,2`-versus-`0,0` leak, the both-stale variant, a missing sample and a
  sample disagreeing with the saved maximum.
- **P0544** (55 keys) and **C10.prepared.profiled_consumer** (55 keys)
  complete through their real routes (`CM SELECTED FAMILIES PASS bound=110
  executed=110`); C10 is the first completed member of the 25 non-source
  route instantiations, leaving 24 routes / 1,315 keys.
- Campaign: 65 rows / 3,380 keys over 34 families, 5,652 actual pauses and
  8,470 lifecycle observations; 86 active rows / 5,680 keys remain; the 24
  director-excluded rows / 1,445 keys stay named; the 175-row / 10,505-key
  requirement bag is unchanged.
- Proofs untouched: 749/43,390 imported, 151/8,255 packet-authenticated,
  115/6,270 current-source verified, 36/1,985 current blocked, 598/35,135
  absent, five known losses (3 Lost + 2 Promoted). All 24 immutable pins
  hash-match; I re-verified each pin, and every one of those files predates
  this run.
- Maintenance: the 68 retention fingerprint cells (only the fingerprint
  column changed) and the four promotion cells (C01/C02, only the two
  certificate columns) were regenerated and mechanically installed. The
  initial reds were the stale C01 cells and are ordering artifacts.

**The STOP.** The option matrix (`P1139` 715, `P1142` 165, `P1147` 550 =
1,430 keys) authenticated against the original seals, but its scratch
importer failed twice before any install: `KeyError: 'trace'`, then
`KeyError: 'source'`. The C6 plans inherit fields from their fixture
descriptors, and the importer treated the descriptors as complete programs.
Both attempts are retained; zero bindings, receipts or proofs were credited;
the two-batch stop-loss is accepted.

**Q12 ruling — derive the inheritance; do not author a manifest.**
Authorized for continuation 9, within the existing test-only CM3 scope, and
nothing more:
- Resolve the C6 fixture descriptors' inherited fields (source, setup, trace,
  configure, files, projections) through the canonical `logical_profile`
  plan/fixture behavior the other 65 rows already use — the same machinery,
  not a parallel resolver and not hand-written literals.
- Authenticate each of the three option rows against its original packet and
  every frozen key before any install; the frozen 1,430 keys stay exactly as
  they are, with no historical re-pin, no sampling and no new exclusion.
- A field that cannot be derived from the surviving packets and the canonical
  fixture behavior names its key/row `PortBlocked` with the exact cause;
  never invent a value and never silently drop a key.
- Install the bindings, execute the rows at multiplicity one through the real
  routes with the same observers and channel set, and reconcile the receipts.
- Then continue the campaign over the remaining active bag, keeping the 36
  current-blocked proof records, the 598 absent / 35,135 and the 24 named
  Execute blockers / 1,445 keys named. The two-batch zero-evidence stop-loss
  stays.
- P0543's lane remains the only feature-isolated build. Whenever the tree
  changes it must be rebuilt content-addressed with the same audit (nine
  manifest feature entries only; every non-manifest file hash-identical), the
  same build-graph and discriminator proof both ways with its falsifier, and
  the same third receipt identity; its build and run costs stay measured
  separately and inside the acceptance accounting.
- The complete active schedule and the acceptance suite run under the 1,500 s
  bound for the two mandated arms as before; over the bound, STOP with the
  measured data and the sharing analysis.
- Any production, Cargo, dependency, manifest or lock change is a STOP.
- Q12's second option is declined: I will not supply a descriptor/binding
  adapter manifest the seals do not produce. That would turn a derivation
  into an assertion and put invented data under a frozen key.

**Host gate.** My own gate on the final tree is green: `cargo fmt --check`
rc 0; the one-thread retention certificate passes in both feature arms (35 s
/ 36 s); the oracle `cm_` suite passes in both arms — 37 passed / 0 failed /
1 ignored, 277 s / 279 s — with `CM RECEIPTS PASS calibration=P0001 rows=65
families=34 bound=3380 executed=3380 multiplicity=1 actual_pauses=5652
lifecycle=8470 campaign_remaining=5680 PortBlocked=1445` and `CM PROOFS PASS
imported=749/43390 packet_authenticated=151/8255 absent=598/35135
current_verified=115/6270`, the lane's `CM P0543 RECEIPTS PASS required=55
bound=55 executed=55 multiplicity=1 lane=p0543-unchecked native_fields=10`,
and `CM BUILD CONFIGURATION PASS lane=mandated control=false
checked_only_enforcement=true`; the ignored provenance audit passes in both
arms (`PROVENANCE AUDIT PASS pins=22 diagnostics=4`); the switched LP targets
are 5/5 in both arms; and the workspace is 4,309 passed / 0 failed / 64
ignored over 70 suites in both feature modes, with the lane environment
authenticated before each invocation. On the host none of the sandbox's nine
loopback denials appears; the worker's raw failure count was its sandbox, not
the tree.

**Continuation 9.** Entry state: 65 rows / 3,380 keys complete; 86 active
rows / 5,680 keys; 24 director-excluded rows / 1,445 keys; proofs 749/43,390;
encoding 0 of 18,840. Resolve and execute the three option rows (1,430 keys)
first, then the remaining 83 active rows / 4,250 keys and the 24 non-source
routes / 1,315 keys, then measure the complete active schedule and run the
acceptance suite for the two mandated arms under the 1,500 s bound, with the
lane's build and run measured separately and included in the accounting. The
P0543 lane must be rebuilt iff the tree changes, and the driver must launch
through the authenticated lane environment; a missing lane is a hard error,
never a skip. C2 stays Open at 118 Residual + 0 Lost; no settlement, no
promotion, no encoding key; `CHANGELOG.md` and S10 stay blocked. The partial
tree is the state to continue from; do not restart and do not hand-edit a
generated cell.

### Continuation 10 — tenth run: the 1,430 option keys authenticate; the canonical-loader extraction is the next wall, and Q13 authorizes it

**2026-11-15 — brief 739's tenth run executed continuation 9 and stopped
before any edit. All 1,430 option-row keys authenticated against the original
seals, the frozen required tuples and the full raw/disabled parity, and 23
route-specific own omissions passed; but the canonical `logical_profile`
inheritance body is reachable only by exposing `from_rows` in
`fixtures/logical_profile/plans.rs`, a path outside the enumerated scope. Q13
authorizes exactly that five-line extraction, pinned by patch sha256; the run
credits no binding, receipt or proof.**
Run `cdx`, unit `brief-739-cdx-20261115-074704-775288`, rc 0, peak
572,661,760 B (well under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 4,137,960 B, sha256 `f35ac30e…`; report
`.temp/brief-739/report.json`, 18,344 B, sha256 `d1dbdcce…`; zero recorded
changes; report 6 PASS / 2 FLAGGED / 1 NA / 1 FAIL (the FAIL is the campaign
unfinished), 1 disclosure, 2 deviations, Q13; 22 citations verified — I reran
`python3 scripts/report.py check 739` myself.

**What was authenticated (read-only; zero credit).**
- `P1139` 715, `P1142` 165, `P1147` 550 keys. For each row the settlement's
  real-receipt key set exactly equals the frozen `required` tuples in
  `bindings.tsv` (sorted equality, no duplicates, count equals the row's
  `required_nonencoding`), and the re-derived sha256 / count / ledger
  dependency triple equals the frozen `required-keys` provenance row pinned
  to the settled ledger `73ec2ff5…`.
- Every receipt's raw real record equals its disabled-arm record, is valid,
  semantic-same and native-same, resumed equals queued, all ten cells per
  row, distinct probes; each plan's source/setup/trace and pauses match the
  sealed fixture object; no `InternalContinuation` source.
- The 1,415 distinct canonical plan input rows were preserved verbatim
  (12 columns each) in `c9-option-canonical-input.tsv` for derivation — not
  resolved by hand, not defaulted here.
- 23 route-specific own omissions: armed and caught, not survived, exit 0,
  `C2_ARM`/`C2_MUTANT_HIT` present, the mutant differs, 55 pause receipts
  per route with distinct probes and the route's pause prompt.
- 42 historical pins hash-match — the three seal manifests, the
  settlements/proof/schema JSONs, the six plan/fixture/TSV inputs, the
  settled ledger and every cited raw log; I re-hashed all 42.
- Entry and final hash manifests are identical over 3,273 files, and
  `git status` / `git diff --stat` are exactly the ninth-run tree I gated
  (9 modified / 11 untracked entries; 125 insertions / 74 deletions) — no
  source, inventory, binding, receipt, certificate or encoding change.

Credit: zero bindings, zero executions, zero proofs, zero encoding keys. The
campaign stays 65 rows / 3,380 keys, 86 active rows / 5,680 keys, 24 excluded
rows / 1,445 keys; proofs stay 749/43,390 (151/8,255; 115/6,270; 36/1,985;
598/35,135; five known losses); owners stay 51 Open.

**The STOP.** Accepted. The scope enumerates `fixtures/obligations/*` and
additive interfaces in `fixtures/mod.rs`; `fixtures/logical_profile/plans.rs`
is not in it. `plans()` hard-wires its own 796-plan `plans.tsv`, none of the
1,430 keys belongs to that table, and Q12 forbids a parallel resolver. The
worker prepared an unapplied five-line patch and did not edit — correct.

**Q13 ruling — authorize the pinned extraction, nothing more.**
- Apply exactly `.temp/brief-739/c9-proposed-canonical-loader.patch`
  (sha256 `be031ff4…`): `plans()` becomes
  `from_rows(include_str!("plans.tsv"))`, its inheritance body moves
  verbatim into `pub(crate) fn from_rows(text: &str) -> Vec<Plan>`, and no
  other byte of that file changes.
- Assert `plans()` still yields the same 796 plans, and the switched LP
  targets stay 5/5 in both arms.
- Derive the option rows' inherited fields (source, setup, trace, configure,
  files, projections) by feeding the 1,415 preserved canonical inputs
  through `from_rows`; every derived value must equal the sealed fixture
  value the authentication already compared. An underivable field names its
  key/row `PortBlocked` with the exact cause — never invented, never
  dropped.
- Install exactly the 1,430 frozen keys; execute at multiplicity one
  through the real routes in the two mandated arms with the same observers
  and channel set; reconcile the receipts. No re-pin, no sampling, no new
  exclusion.
- Then the remaining 83 active rows / 4,250 keys and the 24 non-source
  routes / 1,315 keys, keeping the 36 current-blocked proof records, the
  598 absent / 35,135 and the 24 named Execute blockers / 1,445 named.
- Replay the retention and ownership maintenance after the source edit and
  before acceptance; a stale derived-cell red is an ordering artifact.
- The P0543 lane is rebuilt content-addressed from the final tree (this
  edit and every fixture addition change it): the same nine-manifest audit,
  every non-manifest file hash-identical, the build graph plus the
  checked-only `NonAmplifying` discriminator both ways with its firing
  falsifier, and the same third receipt identity; its build and run costs
  stay separate and inside the acceptance accounting; a missing lane is a
  hard error, never a skip.
- The complete active schedule and the acceptance suite run under the
  1,500 s bound for the two mandated arms; over it, STOP with the measured
  data and the sharing analysis. The two-batch zero-evidence stop-loss
  stays. Any production, Cargo, dependency, manifest or lock change is a
  STOP.
- Option 2 (a separate director brief for the loader API) is declined: it
  is the same five-line test-only edit behind an extra round-trip with no
  behavioral difference, and the narrow pinned extraction is the single
  implementation Q12 requires.

**Host gate.** None is owed and none was run: the run changed nothing, and
the tree is byte-identical to the ninth-run tree I gated green (hash
manifests equal over 3,273 files; git status and diff unchanged). I reran the
citation check (22 verified) and re-hashed the 42 pins and the 1,430
frozen-key tuples myself; the three `OPTION PACKET AUTH PASS … frozen_keys_exact=true
encoding=0` lines, the `OPTION ROUTE AUTH PASS routes=23 own_omissions=23
new_proofs=0` line and the closing `OPTION AUTHENTICATION PASS keys=1430
canonical_inputs=1415 historical_pins=42 binding_credit=0 execution_credit=0
proof_credit=0` are reproduced from the preserved script and its JSON.

**Continuation 10.** Entry state: 65 rows / 3,380 keys complete; 86 active
rows / 5,680 keys; 24 director-excluded rows / 1,445 keys; proofs 749/43,390;
encoding 0 of 18,840; the option rows' 1,430 keys authenticated but
uncredited. Apply the pinned extraction, derive through the canonical
`from_rows`, install and execute the option rows, then the remaining bag and
routes, replay maintenance, rebuild the lane from the final tree, and run the
complete schedule and acceptance under the 1,500 s bound. C2 stays Open at
118 Residual + 0 Lost; no settlement, no promotion, no encoding key;
`CHANGELOG.md` and S10 stay blocked. The partial tree is the state to
continue from; do not restart and do not hand-edit a generated cell.

### Continuation 11 — eleventh run: six rows execute (71 rows / 4,995 keys), the pinned extraction lands byte-exact, and the cross-packet union is opened by Q14

**2026-11-15 — brief 739's eleventh run executed continuation 10. The pinned
canonical-loader extraction was applied byte-exact; the 1,430 option keys and
`P1155` (100), `P0338` (55) and `P0527` (30) were authenticated, derived and
executed in both mandated arms, carrying the campaign from 65 rows / 3,380
keys to 71 rows / 4,995 keys over 37 families, with all 25 named non-source
route instantiations / 1,370 keys now covered. The run then stopped at
`P0260` after two zero-evidence importer attempts: the row's 55 keys draw
their probe records from two sealed C18 packets (49 adopted-accounting + 6
counter-revisit), and each attempt loaded only one packet. The STOP is
accepted and Q14 authorizes a bounded continuation with exact per-probe
lookup across the sealed union; `P0305` has the same 49 + 6 shape.**
Run `cdx`, unit `brief-739-cdx-20261115-080412-779953`, rc 0, peak
15,503,634,432 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 4,598,708 B, sha256 `c38d70c6…`; report
`.temp/brief-739/report.json`, 337,153 B, sha256 `5022af6d…`; 436 recorded
changes (424 added `.rum` assets, 12 modified test/inventory files, 0
deleted); report 18 PASS / 6 FLAGGED / 1 FAIL (the FAIL is the campaign
unfinished), 8 disclosures, 3 deviations, Q14; 641 citations verified — I
reran `scripts/report.py check 739` myself.

**What landed.**
- The pinned extraction is byte-exact: `fixtures/logical_profile/plans.rs`
  carries exactly the five-line addition (`plans()` delegates to a verbatim
  `pub(crate) from_rows(text)`), patch sha256 `be031ff4…`, no other byte of
  the file changed; the frozen 796 plans still fingerprint-match in both
  arms (`CM CANONICAL LEGACY PASS plans=796`).
- `P1139` 715 + `P1142` 165 + `P1147` 550 = 1,430 option keys, derived from
  1,415 preserved canonical plan inputs through `from_rows` with zero field
  differences against the sealed fixture objects (the differences file is an
  empty list, sha256 `37517e5f…`), 23 route own omissions, executed in both
  arms at multiplicity one (`CM SELECTED FAMILIES PASS bound=1430
  executed=1430`). `P1155` (100 keys, Monitor syntax, 50 canonical inputs,
  one own omission) and `P0338` (55) + `P0527` (30) executed the same way
  (`bound=100` and `bound=85`, each `executed` equal).
- Campaign: 71 rows / 4,995 keys over 37 families; 8,163 actual pauses and
  12,616 lifecycle observations; 80 active rows / 4,065 keys remain; the 24
  director-excluded rows / 1,445 keys stay named; the 175-row / 10,505-key
  requirement bag is unchanged; all 25 named non-source route
  instantiations / 1,370 keys are now covered by `C10`, the 23 option
  routes and Monitor syntax.
- Programs: 4,995 rows; 424 new content-addressed `.rum` assets (every
  filename equals its sha256); the 3,380 legacy program cells and the 3,380
  previously bound keys are unchanged (I verified every legacy program row
  byte-identical on its shared columns and every legacy bound key still
  bound).
- Proofs untouched: 749/43,390 imported, 151/8,255 packet-authenticated,
  115/6,270 current-source verified, 36/1,985 current blocked, 598/35,135
  absent, five known-loss records (3 Lost + 2 Promoted), 51 Open owners.
  All 15 immutable obligation inventories hash-match, and the 133 new
  canonical-source pins hash-match their files.
- Maintenance: the 68 retention fingerprint cells changed only in the
  fingerprint column and the four certificate cells only in the two
  certificate columns — I checked both column-wise against the pre-install
  snapshots.
- P0543 lane: rebuilt from the final tree — 4,120 files, exactly nine
  manifest differences (the dev-dependency feature removed,
  `test-continuation-slots` kept; none of the nine diffs touches anything
  else), tree `978d7dff…` and artifact `db59252f…` both recomputed by me,
  release at one codegen unit, canonical build 79.4 s (wrapper 80.78 s);
  `cargo tree -e features` carries no `test-builtin-memory-growth`; the
  behavioral discriminator is true in both mandated arms and false in the
  lane, forced true in the lane's fresh-process control, and its falsifier
  rejects `NonAmplifying` with the exact panic; every one of the 55 lane
  receipts carries the third identity (`sha256` / `tree` / `manifest-diff` /
  profile / codegen-units), 110 identity occurrences per arm, all identical.
  The lane's run cost is reported separately inside each CM command
  (12.75 s / 9.20 s) and the build is counted once.
- No production, repository manifest, lock, dependency or script change;
  the only new `allow` is `#[allow(dead_code)]` on the test-only included
  canonical-loader module.

**The STOP.** `P0260`: the counter-revisit full-proof record (55 cells,
disposition Equal, complete) draws 49 probes from the adopted-accounting
packet and 6 from the counter-revisit packet; each of the two importer
attempts loaded only one packet's plan table and failed on the other's probe
(`KeyError` in both, retained). Zero binding, receipt or proof credit;
the two-batch zero-evidence stop-loss fired; no third attempt ran. Accepted.

**Q14 ruling — the sealed union, per probe.**
- Authorized for the next bounded continuation, within the existing
  test-only CM3 scope, and nothing more:
  - Load the sealed union of the two C18 packets (plan, fixture, canonical
    inputs and raw records) and resolve each of `P0260`'s and `P0305`'s 55
    keys by its own probe id, whose family prefix names exactly one packet.
    I verified independently: both rows' proof records hold 55 distinct
    cells split 49 adopted-accounting + 6 counter-revisit, every probe
    exists in its named packet's plan table, and the key sets are exact.
  - A probe missing from its named packet, or one that would resolve
    ambiguously, names its key/row `PortBlocked` with the exact cause;
    never pick a packet by preference, never invent a record, never drop a
    key, never add an exclusion.
  - Both failed attempts stay retained with zero credit.
  - Then the remaining active bag (78 rows / 3,955 keys after the two union
    rows) and the remaining families; keep the 36 current-blocked proof
    records, the 598 absent / 35,135 and the 24 named Execute blockers /
    1,445 keys named. The seven rows / 365 keys authenticated but not
    installed this run stay reusable by hash identity, and earn nothing
    until installed and executed.
  - Replay retention and ownership maintenance after the source edit and
    before acceptance; a stale derived-cell red is an ordering artifact.
  - Rebuild the P0543 lane content-addressed from the final tree whenever
    the tree changes, with the same nine-manifest audit, build graph plus
    discriminator both ways with its falsifier, and the third receipt
    identity; costs stay separate; a missing lane is a hard error, never a
    skip.
  - The complete active schedule and the acceptance suite run under the
    1,500 s bound per expanded-oracle build and run for the two mandated
    arms; the cumulative validation cost is disclosed separately and is not
    an aggregate budget claim (this run's 1,658.007 / 1,528.523 s include
    overlapping mandated suites and the shared lane build; no individual
    expanded-oracle invocation exceeded 1,500 s, and the complete
    active schedule remains unmeasured while 4,065 keys are unported).
  - The two-batch zero-evidence stop-loss stays; any production, Cargo,
    dependency, manifest or lock change is a STOP.
- Option 2 is declined: the union is sealed, complete and non-overlapping;
  the failed attempts were input selection, not missing provenance.
  Deferring would leave 110 keys unbound for no correctness gain.

**Host gate.** My own gate on the final tree is green: `cargo fmt --check`
rc 0; the one-thread retention certificate passes in both feature arms
(35 s / 35 s); the oracle `cm_` suite passes in both arms — 38 passed / 0
failed / 2 ignored, 342 s / 338 s — with the exact `CM RECEIPTS PASS
calibration=P0001 rows=71 families=37 bound=4995 executed=4995 multiplicity=1
actual_pauses=8163 lifecycle=12616 campaign_remaining=4065 PortBlocked=1445
full_CM_green=false`, `CM PROOFS PASS imported=749/43390
packet_authenticated=151/8255 absent=598/35135 current_verified=115/6270
partial=true executions=0 SAME=0 full_CM_green=false`, `CM P0543 RECEIPTS
PASS required=55 bound=55 executed=55 multiplicity=1 lane=p0543-unchecked
native_fields=10 full_CM_green=false`, `CM BUILD CONFIGURATION PASS
lane=mandated control=false checked_only_enforcement=true` and `CM CANONICAL
LEGACY PASS plans=796 frozen_fingerprints=796 executions=0` lines; the
ignored provenance audit passes in both arms (`PROVENANCE AUDIT PASS pins=22
diagnostics=4`); the switched LP targets are 5/5 in both arms; and the
workspace is 4,310 passed / 0 failed / 65 ignored over 70 binaries in both
feature modes. The sandbox's nine native ZeroMQ loopback `EPERM` failures do
not appear on the host — the same nine tests pass here — so that is a sandbox
boundary, not a tree defect. My oracle log reproduces the worker's raw
receipt count exactly (5,111 lines = 4,995 bound keys + 116 observer
duplicates: P0001 +1, P0508 +30, P0543 +55, P1577 +30) and the 37 execution
falsifiers.

**Continuation 11.** Entry state: 71 rows / 4,995 keys complete; 80 active
rows / 4,065 keys; 24 director-excluded rows / 1,445 keys; proofs 749/43,390;
encoding 0 of 18,840. Resolve `P0260` and `P0305` through the sealed union per
probe, then continue the remaining active bag and families, replay
maintenance, rebuild the lane from the final tree, and run the complete
schedule and acceptance under the 1,500 s bound. C2 stays Open at 118
Residual + 0 Lost; no settlement, no promotion, no encoding key;
`CHANGELOG.md` and S10 stay blocked. The partial tree is the state to
continue from; do not restart and do not hand-edit a generated cell.

### Continuation 12 — twelfth run: 27 rows execute (98 rows / 6,365 keys), P0260/P0305 resolve through the sealed C18 union, and P0450/P0455's three-packet owned-guard union is opened by Q15

**2026-11-15 — brief 739's twelfth run executed continuation 11. The
cross-packet C18 union resolved `P0260` and `P0305` per probe (49
adopted-accounting + 6 counter-revisit each), and 25 further rows executed,
carrying the campaign from 71 rows / 4,995 keys to 98 rows / 6,365 keys over
47 families, with 10,832 actual pauses and 16,928 lifecycle observations. The
run then stopped at `P0450`/`P0455`, whose proof cells need two and three
records per key from three sealed C19 packets; the importer accepted at most
two, and each of its two attempts added zero evidence. Q15 authorizes a
bounded continuation with the full three-packet union and up to three
constituent programs under one key receipt; 51 rows / 2,585 keys remain.**
Run `cdx`, unit `brief-739-cdx-20261115-103445-866881`, rc 0, peak
15,987,343,360 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 6,407,908 B, sha256 `d9616436…`; report
`.temp/brief-739/report.json`, 1,933,926 B, sha256 `20c68358…`; 663 recorded
changes (654 added `.rum` assets, 9 modified, 0 deleted); report 17 PASS / 5
FLAGGED / 1 FAIL / 1 NA (the FAIL is the campaign unfinished), 6
disclosures, 4 deviations, Q15; 14,235 citations verified.

**What landed.**
- Q14's union is fulfilled: `P0260` and `P0305` each 55 keys, resolved by
  probe prefix to 49 adopted-accounting + 6 counter-revisit records,
  authenticated against the sealed C18 packets, installed as 2 rows / 110
  keys with 30 new assets and the 4,995 legacy programs unchanged, and
  executed at multiplicity one in both arms; the 110 canonical plans carry
  zero field differences through the canonical `from_rows` resolver.
- 25 further rows / 1,260 keys executed: batch 2 (9 rows / 500), the
  default-returned row (1 / 60), `P0256` (1 / 60), diagnostics-13 (13 / 610)
  and `P0515` (1 / 30); each install asserts `legacy_bindings_unchanged=true`
  and `frozen_requirements_unchanged=true`.
- Campaign: 98 rows / 6,365 keys over 47 families; 10,832 actual pauses and
  16,928 lifecycle observations; 53 active rows / 2,695 keys remain; the 24
  director-excluded rows / 1,445 keys stay named (9 absent-packet + 13
  unresolved + 2 Repl rows); the requirement bag stays 175 rows / 10,505
  keys.
- Fifteen frozen cycle keys (five each `P0256`/`P0259`/`P0266`) retain two
  constituent programs under one key receipt: 6,365 tuple receipts cover
  6,380 programs, 16 embedded paired records (15 campaign pairs plus one
  control), and multiplicity one is per key; the receipt identity check
  recurses into the paired receipt and fails if either side is missing.
- The three `P0515` timing fixtures are the only rows with non-empty
  duration paths (`[[0]]`); the comparison masks only the declared leaf
  while sibling values, prints, messages, trace events and structured
  outputs stay exact; four duration falsifiers reject sibling, print,
  trace-event and negative-duration mutations by their own predicates.
- Programs 6,365 rows; 654 new content-addressed `.rum` assets (1,987
  total, every filename equals its sha256); all 4,995 entry program rows and
  all 4,995 entry bound key rows persist verbatim, and exactly the 1,370
  converted blocked rows are absent from the entry table (no old row lost or
  altered).
- Proofs untouched (749/43,390 imported; 151/8,255 packet-authenticated;
  115/6,270 current-source verified; 36/1,985 current blocked; 598/35,135
  absent; five known-loss records); the 15 immutable obligation
  inventories, the proof/loss tables, aliases and all 133 canonical-source
  pins hash-match.
- Maintenance: 68 fingerprint cells changed only in the fingerprint column
  and four certificate cells only in the two certificate columns, in both
  rounds (column-wise verified against the pre-maintenance snapshots); the
  ownership inventories replay unchanged.
- P0543 lane rebuilt from the final tree: 4,365 copied files (4,356
  nonmanifest byte-identical + nine manifest edits), tree
  `3b8e3d51…`, artifact `8621d453…`, manifest-diff `a6f12578…`,
  release at one codegen unit, build 61.5 s; the build graph carries no
  `test-builtin-memory-growth` and keeps `test-continuation-slots`; the
  behavioral discriminator is true in both mandated arms and false in the
  lane, forced true in the lane's fresh-process control, with its falsifier
  firing the exact panic; every lane receipt carries the third identity
  (110 occurrences per arm, all identical).
- No production, repository manifest, lock, dependency, doc or script
  change.

**The STOP.** `P0450`/`P0455`: the sealed proof records need 55 × 2 = 110
and (49 × 3) + (6 × 2) = 159 records from three C19 packets; the importer
accepts at most two records per cell. Attempt 1 omitted the finalizer packet
and failed before authentication (`ValueError`); attempt 2 selected original
+ finalizer, authenticated `P0450` locally, then failed its own maximum-two
assertion on `P0455`'s 49 three-record cells. Neither attempt emitted a
complete artifact, installed a binding, executed a new key or imported a new
proof; no third attempt ran. The two-batch zero-evidence stop-loss fired.
Accepted.

**Q15 ruling — the three-packet union, per record.**
- Authorized for the next bounded continuation, within the existing
  test-only CM3 scope, and nothing more:
  - Load the sealed union of `owned-guard-suffix-and-deferred-registration`,
    `owned-guard-suffix-finalizer-revisit` and
    `registered-match-and-guard-owners` (plan, fixture, canonical inputs and
    raw records) and resolve each of `P0450`'s and `P0455`'s 55 keys by its
    cell's record set. I verified independently: all five files of each
    packet and the proof file are sealed in
    `.temp/g15-c2/c19/artifact-hashes.json` and hash-match; both proof
    records have 55 distinct cells; `P0450` = 55 cells × 2 records (55 + 55),
    `P0455` = 49 cells × 3 + 6 cells × 2 (55 + 55 + 49 = 159); every probe id
    names exactly one packet and exists in that packet's plan table; every
    cell key equals its records' row/variant/context; the key sets are exact.
  - Extend the receipt mechanism from two to up to three constituent
    programs under one key's single receipt at multiplicity one, exactly as
    the 15 paired keys already retain two: each constituent keeps its own
    canonical fingerprint, content-addressed source/setup, answer plan and
    fresh session; no constituent is dropped, deduplicated away or executed
    as a second key; execution credit equals bound keys, not program runs.
  - A key whose canonical inputs or fingerprints do not match its sealed
    cell's record count, or a probe missing from its named packet or
    resolving ambiguously, names its key/row `PortBlocked` with the exact
    cause; never pick a packet by preference, invent a record, drop a key or
    add an exclusion.
  - Both failed attempts stay retained with zero credit.
  - Then the remaining active bag (51 rows / 2,585 keys after the two union
    rows) and the remaining families; keep the 36 current-blocked proof
    records, the 598 absent / 35,135 and the 24 named exclusions / 1,445
    named. The 355 authenticated-but-uninstalled keys (`P0339` 55,
    `P0348`–`P0351` 200, `P0823`/`P0827` 100) stay reusable by hash identity
    and earn nothing until installed and executed.
  - Replay retention and ownership maintenance after the source edit and
    before acceptance; a stale derived-cell red is an ordering artifact.
  - Rebuild the P0543 lane content-addressed from the final tree whenever the
    tree changes, with the same nine-manifest audit, build graph plus
    discriminator both ways with its falsifier, and the third receipt
    identity; costs stay separate; a missing lane is a hard error, never a
    skip.
  - The complete active schedule and the acceptance suite run under the
    1,500 s bound per expanded-oracle build and run for the two mandated
    arms; the cumulative validation cost is disclosed separately and is not
    an aggregate budget claim (this run: 438.57 / 439.64 s per
    expanded-oracle build+run, under the 1,500 s cap and the 900 s steady
    target; the 1,384.2 / 1,257.8 s subtotals include overlapping suites; the
    complete active schedule remains unmeasured while 2,695 keys are
    unported).
  - The two-batch zero-evidence stop-loss stays; any production, Cargo,
    dependency, manifest or lock change is a STOP.
- Option 2 is declined: the union is sealed, complete and non-overlapping;
  the failures were importer selection and multiplicity limits, not missing
  provenance.

**Host gate.** My own gate on the final tree is green: `cargo fmt --check`
rc 0; the one-thread retention certificate passes in both feature arms
(35 s / 35 s); the oracle `cm_` suite passes in both arms — 40 passed / 0
failed / 2 ignored in the main binary plus the two child summaries, 418 s /
415 s — with the exact `CM RECEIPTS PASS calibration=P0001 rows=98
families=47 bound=6365 executed=6365 multiplicity=1 actual_pauses=10832
lifecycle=16928 campaign_remaining=2695 PortBlocked=1445
full_CM_green=false`, `CM BINDINGS PASS required=10505 active=9060
declared_bound=6365 PortBlocked=4140 absent_packets=600
unresolved_targets=785 repl_placement=60 unported_programs=2695
full_CM_green=false`, `CM PROOFS PASS imported=749/43390
packet_authenticated=151/8255 absent=598/35135 current_verified=115/6270
partial=true executions=0 SAME=0 full_CM_green=false`, `CM P0543 RECEIPTS
PASS required=55 bound=55 executed=55 multiplicity=1 lane=p0543-unchecked
native_fields=10 full_CM_green=false`, `CM BUILD CONFIGURATION PASS
lane=mandated control=false checked_only_enforcement=true` and `CM CANONICAL
LEGACY PASS plans=796 frozen_fingerprints=796 executions=0` lines; my
wrapper re-verified the lane binary, tree and manifest-diff identity before
every oracle and workspace invocation (`P0543 INVOCATION AUTH PASS …`);
the ignored provenance audit passes in both arms (`PROVENANCE AUDIT PASS
pins=22 diagnostics=4`); the switched LP targets are 5/5 in both arms; and
the workspace is 4,312 passed / 0 failed / 65 ignored over 70 binaries in
both feature modes — exactly the worker's 4,303 sandbox passes plus the nine
native ZeroMQ loopback `EPERM` tests, which pass on the host. My oracle log
reproduces the worker's raw receipt count exactly (6,482 lines = 6,365 bound
keys + 117 observer duplicates: `P0001` +1, `P0256` +1, `P0508` +30, `P0543`
+55, `P1577` +30), the 16 paired raw records, and the 37 execution
falsifiers; I also verified the 27-row table's per-row counts against the
bindings table.

**Continuation 12.** Entry state: 98 rows / 6,365 keys complete; 53 active
rows / 2,695 keys; 24 director-excluded rows / 1,445 keys; proofs
749/43,390; encoding 0 of 18,840. Resolve `P0450` and `P0455` through the
three-packet union per record, then continue the remaining 51 active rows /
2,585 keys and families, replay maintenance, rebuild the lane from the final
tree, and run the complete schedule and acceptance under the 1,500 s bound.
C2 stays Open at 118 Residual + 0 Lost; no settlement, no promotion, no
encoding key; `CHANGELOG.md` and S10 stay blocked. The partial tree is the
state to continue from; do not restart and do not hand-edit a generated
cell.

### Continuation 13 — thirteenth run: Q15's three-packet union executes P0450/P0455 per record (100 rows / 6,475 keys), and P1476's synchronous-observer wall is opened by Q16

**2026-11-15 — brief 739's thirteenth run executed continuation 12. The
sealed three-packet C19 union resolved `P0450`'s and `P0455`'s 55 keys each
per cell record set — 269 constituent programs under 110 key receipts at
multiplicity one — carrying the campaign to 100 rows / 6,475 keys over 48
families, with 11,367 actual pauses and 17,576 lifecycle observations. The
run then stopped at `P1476`, whose 30 keys need the sealed synchronous
`Epoch::events` vectors for `evaluate.fixed_point_equal`; the epoch is
runtime-private and no public accessor exists, and both attempts to read it
outside the crate fail (E0599/E0603). Q16 authorizes one unconditional
`#[doc(hidden)]` read-only `Session` accessor in the Q4/Q9/Q10 form; then
`P1476`'s 30 keys and the remaining 50 rows / 2,555 keys continue.**
Run `cdx`, unit `brief-739-cdx-20261115-132608-960976`, rc 0, peak
14,804,566,016 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 5,860,639 B, sha256 `184aefa7…`; report
2,515,349 B, sha256 `0b65febf…`; 37 recorded changes (30 added `.rum`
assets, 7 modified, 0 deleted); report 19 PASS / 4 FLAGGED / 1 FAIL / 1 NA
(the FAIL is the campaign unfinished), 6 disclosures, 4 deviations, Q16;
21,473 citations verified.

**What landed.** The Q15 union executed per record: `P0450` = 55 cells × 2
records (110 = 55 + 55) and `P0455` = 49 cells × 3 + 6 cells × 2 (159 = 55
+ 55 + 49), every probe resolved through its named packet's plan table, each
constituent keeping its own canonical fingerprint, content-addressed
source/setup, answer plan and fresh session under one key receipt;
6,475 receipts cover 6,649 constituent programs (6,350 single, 76 double,
49 triple) at multiplicity one, 11,367 actual pauses and 17,576 lifecycle
observations. The 6 two-record `P0455` cells are exactly
`AlreadyResumed`/`Nested` × the three `held_and_splice` tail contexts, as
the sealed union declares. 30 new content-addressed `.rum` assets (2,017
total); the 6,365 entry program rows and all required/bound entry key rows
persist verbatim; exactly 110 blocked rows convert to bound and 30 `P1476`
blocked rows record their exact blocker evidence, with no new exclusion.
Proofs are untouched (749/43,390; 151/8,255; 115/6,270; 36/1,985;
598/35,135; five known losses), the 15 immutable inventories and 133
canonical-source pins hash-match, the program-table pin counts 6,475, and
the 110 new guest routes hash-match their source files. Maintenance
replayed 68 fingerprint cells in one column and 4 certificate cells in two
columns only; ownership reconciled 9,003 rows with zero changes. The P0543
lane was rebuilt from the final tree: 4,804 copied files (4,792 nonmanifest
byte-identical + 9 of 12 manifests edited, 3 unchanged), tree `bb9b5189…`,
artifact `17d63fa6…`, manifest-diff `1842e875…`, release at one codegen
unit, build 83.6 s, the graph keeping `test-continuation-slots` and no
`test-builtin-memory-growth`, the discriminator true in both mandated arms
and false in the lane with a forced control and its firing falsifier, and
110 identical third identities per arm. No production, repository manifest,
lock, dependency, doc or script change.

**The STOP.** `P1476` (30 active keys): the sealed record
`C31Synchronous Runtime { phase: "evaluate.fixed_point_equal", source: None }`
carries the ten-field vector `EventValue.vector[0..7] + input_required`,
recorded by the scratch observer
`.temp/g15-c2/c31/prepare_synchronous_equality.py:45`; the epoch is
runtime-private and the public logical rows read a different `Epoch::rows`
table while the public snapshot reads legacy timing tables. Two rustc
probes in both feature modes reject the absent accessor (`E0599`) and the
private epoch path (`E0603`); no public accessor substitutes, and the
sealed own omission keeps all semantic output while moving the vector
6,6 → 4,4. No binding, receipt, proof or execution credit; the two
follow-on authentication batches (7 rows / 355 keys and 8 rows / 395 keys)
were authenticated only and earn nothing until installed and executed.
Accepted.

**Q16 ruling — the narrow C31 accessor, then the keys.**
- Evidence verified independently: the sealed schema and record (pins in
  `.temp/brief-739/p1476-stop-evidence.json`, all five artifacts hashed
  equal), the ten-field projection `[vector[0..7], input_required, 0, 0]`,
  the E0599/E0603 probes, and the sealed own-omission witness
  (semantic same, native 6,6 versus 4,4).
- Authorized, and nothing more: one unconditional `#[doc(hidden)]`
  read-only inherent method on the runtime `Session` returning exactly the
  sealed observer's value — the recorded synchronous events filtered to
  `EventKey::Runtime { phase: "evaluate.fixed_point_equal", .. }`, each
  formatted `C31Synchronous {key:?}` with the ten fields
  `[vector[0..7], input_required, 0, 0]`, sorted by key; no event added,
  normalized, retired, reset or timed; no mutation; no field-visibility
  change; no other body or caller change; no new dependency; no
  `Cargo.toml`/`Cargo.lock` edit; no `unsafe`.
- The Q4/Q9/Q10 audit gate applies: append-only source, prior bytes a
  prefix, exactly one new symbol, zero production call sites, the
  declaration recorded by the production audit, and the deterministic gate
  (default-feature build green, symbol inspection, no emitted production
  call site) standing in for a hot-path A/B. A cfg-dependent impl is
  rejected by the rule-boundary audit; an audit rejection is a STOP, never
  a widened predicate or a hand-edited cell.
- First action: land exactly this, replay the retention/ownership
  maintenance, and rerun `local_production_acceptance` in both arms at one
  thread.
- Then bind and execute `P1476`'s 30 keys through the real route named by
  their authenticated consumer route, observing the vectors from the
  executing thread at the sealed point (after the subject, before
  projections), with the ten-field native row at multiplicity one in both
  arms; a public phase row, the legacy timing snapshot, a cached scalar or
  a skipped key is not an observation; the own-omission falsifier must
  reject the 4,4-versus-6,6 discrimination by `Native`.
- Then the remaining 50 rows / 2,555 keys and families; keep the 36
  current-blocked proof records, the 598 absent / 35,135 and the 24 named
  exclusions / 1,445 named; the 750 authenticated-but-uninstalled keys
  (`P0339` 55, `P0348`–`P0351` 200, `P0823` 65, `P0827` 35 and the eight
  batch-3 rows 395) stay reusable by hash identity and earn nothing until
  installed and executed.
- Replay maintenance after the source edit and before acceptance; rebuild
  the lane content-addressed from the final tree whenever it changes with
  the same audit, discriminator and third identity; the complete active
  schedule and acceptance run under the 1,500 s bound per expanded-oracle
  build+run, with the cumulative cost disclosed separately (this run:
  456.59 / 454.04 s per expanded-oracle build+run, under the 1,500 s cap
  and the 900 s target; the complete 9,060-key schedule remains unmeasured
  while 2,585 keys are unported). The two-batch zero-evidence stop-loss
  stays; any other production, Cargo, dependency, manifest or lock change
  is a STOP.
- Option 2 is declined: the sealed observer is the row's exact contract,
  the accessor is the campaign's established Q4/Q9/Q10 form (an uncalled
  getter has no default-path runtime cost and the audit records it), and a
  director-added exclusion of 30 active keys would weaken the frozen
  active bag without cause.

**Host gate.** My own gate on the final tree is green: `cargo fmt --check`
rc 0; the one-thread retention certificate passes in both feature arms
(35 s / 35 s); the oracle `cm_` suite passes in both arms — 41 passed / 0
failed / 2 ignored in the main binary plus two child summaries, 435 s / 432
s — with the exact `CM RECEIPTS PASS calibration=P0001 rows=100
families=48 bound=6475 executed=6475 multiplicity=1 actual_pauses=11367
lifecycle=17576 campaign_remaining=2585 PortBlocked=1445
full_CM_green=false`, `CM BINDINGS PASS required=10505 active=9060
declared_bound=6475 PortBlocked=4030 absent_packets=600
unresolved_targets=785 repl_placement=60 unported_programs=2585
full_CM_green=false`, `CM PROOFS PASS imported=749/43390
packet_authenticated=151/8255 absent=598/35135 current_verified=115/6270
partial=true executions=0 SAME=0 full_CM_green=false`, `CM P0543 RECEIPTS
PASS required=55 bound=55 executed=55 multiplicity=1 lane=p0543-unchecked
native_fields=10 full_CM_green=false`, `CM BUILD CONFIGURATION PASS
lane=mandated control=false checked_only_enforcement=true` and `CM CANONICAL
LEGACY PASS plans=796 frozen_fingerprints=796 executions=0` lines; my
wrapper re-verified the lane binary (sha256 `17d63fa6…`), the recomputed
tree `bb9b5189…`, the manifest-diff `1842e875…` and all 4,804 copied files
(4,792 nonmanifest + 3 unchanged manifests byte-identical to the workspace;
the nine edits exactly the `test-builtin-memory-growth` removal with
`test-continuation-slots` kept) before every oracle and workspace
invocation; the ignored provenance audit passes in both arms (`PROVENANCE
AUDIT PASS pins=22 diagnostics=4`); the switched LP targets are 5/5 in both
arms; and the workspace is 4,312 passed / 0 failed / 65 ignored over the 69
targets in both feature modes — the sandbox's nine loopback `EPERM`
failures pass on the host, and the two intentionally failing typed-slot
child summaries are the scratch-copy censuses whose parent suite is
10/0/1. My oracle log reproduces the worker's raw receipt count exactly
(6,593 = 6,475 bound keys + 118 observer duplicates: `P0001` +1, `P0256`
+1, `P0455` +1, `P0508` +30, `P0543` +55, `P1577` +30), the four
constituent falsifiers with their exact predicates plus the constituent
receipt control, the two paired falsifiers, and the third lane identity on
every lane receipt (110 occurrences per arm plus my wrapper's line). I also
verified independently: the 110 converted keys' bound evidence names the
sealed proof and every constituent probe; the sealed union's 55-cell
structure with the six two-record cells; each constituent's source/setup
bytes against the sealed plans (and the `.rum` content addressing, 2,017
files); all 238 c19 packet pins against `artifact-hashes.json` and 6,355
guest-route source hashes; the 15 immutable inventories and 133
canonical-source pins; the 68 + 4 maintenance cells column-wise; the 30
`P1476` blocker rows keeping status and key with only the evidence column
changed; and the run's log, report and change-manifest hashes.

**Continuation 13.** Entry state: 100 rows / 6,475 keys complete; 51 active
rows / 2,585 keys; 24 director-excluded rows / 1,445 keys; proofs
749/43,390; encoding 0 of 18,840. Land the Q16 accessor exactly, replay
maintenance, and rerun `local_production_acceptance` in both arms at one
thread; then bind and execute `P1476`'s 30 keys through their real route
with the ten-field native row and the own-omission `Native` falsifier at
multiplicity one, then the remaining 50 rows / 2,555 keys and families,
replay maintenance, rebuild the lane from the final tree, and run the
complete schedule and acceptance under the 1,500 s bound. C2 stays Open at
118 Residual + 0 Lost; no settlement, no promotion, no encoding key;
`CHANGELOG.md` and S10 stay blocked. The partial tree is the state to
continue from; do not restart and do not hand-edit a generated cell.

### Continuation 14 — fourteenth run: Q16's audited C31 accessor executes P1476 (133 rows / 8,220 keys), and P1566/P1568's private checkpoint pair is opened by Q17

**2026-11-15 — brief 739's fourteenth run executed continuation 13. Q16's
single unconditional `#[doc(hidden)]` read-only `Session` accessor landed
append-only and executed `P1476`'s 30 keys through their real route with
the sealed ten-field synchronous row at multiplicity one in both arms
(the 6,6-versus-4,4 own omission rejected by `Native`, with the stale,
missing and duplicate controls); 33 further rows then executed — batch2
7 rows / 355 keys, batch3 guest 7 / 365, batch4 9 / 500, batch5 9 / 495 —
carrying the campaign from 100 rows / 6,475 keys to 133 rows / 8,220 keys
over 57 families, with 14,865 actual pauses and 23,390 lifecycle
observations. The run then stopped at `P1566`/`P1568`, whose 60 active
keys need the private message-checkpoint pair
`(current_message_list_checkpoint.is_some(), current_message_list_dirty)`
that no public effect, projection or snapshot exposes. Q17 authorizes one
unconditional append-only read-only `#[doc(hidden)]` getter for exactly
that pair plus explicit test-only scope at `host_facade.rs:17`; then both
complete 30-key matrices and the remaining 16 rows / 780 keys continue.**
Run `cdx`, unit `brief-739-cdx-20261115-160720-1044767`, rc 0, peak
13,377,794,048 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 4,723,826 B, sha256 `feefa2c3…`; report
`.temp/brief-739/report.json`, 2,584,567 B, sha256 `4db86c15…`; 634
recorded changes (622 added `.rum` assets, 12 modified, 0 deleted); report
16 PASS / 4 FLAGGED / 1 FAIL / 1 NA (the FAIL is the campaign unfinished),
6 disclosures, 6 deviations, Q17; 21,089 citations verified.

**What landed.** The Q16 accessor is exactly the authorized shape and
nothing else: `session.rs` keeps its 10,303-byte prefix and gains 891
bytes (one `#[doc(hidden)]` impl block with one `pub fn`, the recorded
`Runtime { phase: "evaluate.fixed_point_equal" }` events formatted
`C31Synchronous {key:?}` as `[vector[0..7], input_required, 0, 0]`,
sorted), using `epoch::existing()` so no epoch is created; one new runtime
symbol, zero removed, zero production call sites, unconditional, no
`cfg`, no `unsafe`, no field-visibility, body or caller change, no
dependency, manifest or lock edit. `P1476`'s 30 keys execute through
their real route; the vector is read on the executing thread after the
subject and before the projections, and the validator requires exactly
one `C31Synchronous Row` per side whose value equals the seal-derived
counts (the Rust table is byte-equal to the sealed events of
`p1476-installed-authentication.json`); the own-omission falsifier moves
6,6 → 4,4 with equal output and requires a `Native` rejection, and the
both-stale, missing and duplicate controls fire. 33 further rows
installed and executed with 1,745 keys, 8,670 constituent plans over
8,220 keys (351 multi-record keys, maximum three), every constituent
content-addressed; 622 new `.rum` assets (2,639 total); all 6,475 entry
program rows and every required/bound entry row persist verbatim, and
exactly the 1,745 blocked rows convert to bound with evidence naming
their root probe and sealed proof. Batch6 (5 rows / 280) and batch7
(5 rows / 290) were authenticated only, not installed, and earn
nothing. Proofs are untouched (749/43,390; 151/8,255; 115/6,270;
36/1,985; 598/35,135; five known losses); the 133 canonical-source pins
and every provenance pin hash-match, the program-table pin counts 8,220,
133 bound rows stand over 57 families, and 18 active rows / 840 keys
remain against the 24 named exclusions / 1,445. Maintenance replayed 68
fingerprint cells in one column and 4 certificate cells in two columns;
ownership went 9,003 → 9,006 with three `UnknownConstruct` rows from the
appended getter (`matches!`, `format!`, closure) and the final replay
added none. The lane was rebuilt from the final tree: 5,426 files (5,417
byte-identical + 9 edited manifests), tree `7560325d…`, artifact
`f1f040fa…`, manifest-diff `6de69589…`, release at one codegen unit,
build 83.989 s, the graph keeping `test-continuation-slots` and no
`test-builtin-memory-growth`, the discriminator true in both mandated
arms and false in the lane with its forced control. No production,
repository manifest, lock, dependency, doc or script change.

**The STOP.** `P1566`/`P1568` (2 rows, 60 active keys): the sealed C68
records require the pair `(current_message_list_checkpoint.is_some(),
current_message_list_dirty)` sampled after the host outcome and before
snapshot / fresh / projections. The fields are private to the session
module and no getter exists; both probes fail (`E0599` absent method,
`E0616` private field). The sealed matrices show the pair agreeing across
the resumed and queued paths in all 30 cells (20 × `(false,false)`,
10 × `(false,true)`), while the own omission flips it
(`P1566` `(false,false)` versus `(true,true)`, `P1568` the reverse) and
keeps every public effect — messages, prints, trace, projections,
snapshot, fresh, pending, queue — byte-equal. No public-effect
substitute, cached scalar or constant can discriminate it. Zero
binding/receipt/proof/execution credit; accepted.

**Q17 ruling — the checkpoint getter and the test-only bridge, then the keys.**
- Evidence verified independently: the 11 sealed pins hash-match; the
  sealed getter returns exactly the two flags; the own-omission pairs
  and the equal public effects are in the pinned C68 logs; the fixed
  matrices agree across paths in all 30 cells; the access probe's
  `E0599`/`E0616` are retained.
- Authorized, and nothing more: one unconditional append-only read-only
  `#[doc(hidden)]` inherent method on the runtime `Session` returning
  `(current_message_list_checkpoint.is_some(),
  current_message_list_dirty)`; the Q4/Q9/Q10/Q16 audit gate applies
  (append-only source with the prior bytes a prefix, exactly one new
  symbol, zero production call sites, the declaration recorded by the
  production audit, the deterministic gate — default-feature build
  green, symbol inspection — instead of a hot-path A/B; no `cfg`, no
  `unsafe`, no field-visibility change, no new dependency, no
  `Cargo.toml`/`Cargo.lock` edit). An audit rejection is a STOP, never a
  widened predicate.
- Also authorized, and bounded: explicit test-only scope at
  `crates/rum-stdlib/tests/suspension_oracle/host_facade.rs:17` to
  bridge the private `HostSession.session` inside the source-reused
  `host_facade::session` module through one test-only forwarding
  accessor returning the pair. The reused
  production host source must stay byte-identical to
  `crates/rum-host/src/session.rs` (its hash recorded and checked), the
  only added item is that one test-only accessor, no production host
  body or field visibility changes, and no copied host logic. If the
  reuse cannot be preserved byte-exactly, STOP.
- Then bind and execute both complete 30-key `P1566`/`P1568` matrices at
  the sealed sampling point — after the host outcome, before
  snapshot/fresh/projections — from the executing thread, with the pair
  recorded at multiplicity one in both arms and the expected value taken
  from the sealed records (20 cells `(false,false)`, 10 `(false,true)`);
  a public phase row, the snapshot, the legacy timing table, a cached
  scalar or a constant is not an observation. The own-omission falsifier
  must reject `P1566`'s `(false,false)`-versus-`(true,true)` and
  `P1568`'s `(true,true)`-versus-`(false,false)` by `Native` while every
  public effect stays equal, and a constant-pair control must fire.
- Then the remaining 16 rows / 780 keys: install and execute batch6
  (5 rows / 280) and batch7 (5 rows / 290) by their authenticated
  artifacts and hash identity, then `P0813`/C49 (30), `P1470`/`P1471`
  (90), `P2600` (30) and `P2611`/`P2612` (60).
- Keep the 36 current-blocked proof records, the 598 absent / 35,135,
  the 24 named exclusions / 1,445 and the requirement bag 175 / 10,505
  named; no settlement, no owner promotion, no encoding key; replay
  maintenance after the source edit; rebuild the lane content-addressed
  from the final tree; the complete active schedule and acceptance under
  the 1,500 s bound per expanded-oracle build+run with the cumulative
  cost disclosed separately (this run: 582.990 / 586.139 s per
  expanded-oracle build+run, under the 1,500 s cap and the 900 s target;
  cumulative mode-labelled subtotals 2,321.344 / 2,173.309 s; the
  complete 9,060-key schedule remains unmeasured while 840 keys are
  unported). Any other production, Cargo, dependency, manifest or lock
  change is a STOP.
- Option 2 is declined: the pair is the row's exact sealed contract, the
  getter is the campaign's established Q4/Q9/Q10/Q16 form with no
  default-path runtime cost, and leaving 60 active keys blocked would
  weaken the frozen active bag without cause.

**Host gate.** My own gate on the final tree is green: `cargo fmt --check`
rc 0; the one-thread retention certificate both arms (35 s / 35 s); the
oracle `cm_` suite both arms — 42 passed / 0 failed / 2 ignored in the
main binary plus child summaries (547 s / 539 s) with the exact
`CM RECEIPTS PASS calibration=P0001 rows=133 families=57 bound=8220
executed=8220 multiplicity=1 actual_pauses=14865 lifecycle=23390
campaign_remaining=840 PortBlocked=1445 full_CM_green=false`,
`CM BINDINGS PASS required=10505 active=9060 declared_bound=8220
PortBlocked=2285 absent_packets=600 unresolved_targets=785
repl_placement=60 unported_programs=840 full_CM_green=false`,
`CM P1476 RECEIPTS PASS required=30 bound=30 executed=30 multiplicity=1
actual_pauses=102 lifecycle=132 native_fields=10 full_CM_green=false`,
`CM PROOFS PASS imported=749/43390 packet_authenticated=151/8255
absent=598/35135 current_verified=115/6270 partial=true executions=0
SAME=0 full_CM_green=false`, `CM P0543 RECEIPTS PASS required=55 bound=55
executed=55 multiplicity=1 lane=p0543-unchecked native_fields=10
full_CM_green=false`, `CM BUILD CONFIGURATION PASS lane=mandated
control=false checked_only_enforcement=true` (and its control),
`CM CANONICAL LEGACY PASS plans=796 frozen_fingerprints=796
executions=0` and `CM LOSSES PASS prior_Lost=5 immutable_differences=3
C1_promotions=2 missing_original_baselines=2 encoding_runs=0
fresh_allowed_DIFF=0` lines — with all four `P1476` falsifiers firing in
both arms; the ignored provenance audit both arms (`PROVENANCE AUDIT PASS
pins=22 diagnostics=4`); switched LP 5/5 both arms; and the workspace
4,314 passed / 0 failed / 65 ignored over 70 suite summaries in both
feature modes. My wrapper re-verified the lane binary (`f1f040fa…`), the
recomputed tree (`7560325d…`), the manifest-diff (`6de69589…`) and all
5,426 copied files before every oracle and workspace invocation. My
independent audit of the run closed with zero failures: the entry
snapshot equals the thirteenth-run final tree on all 3,692 shared files;
the change set is exactly 12 modified files + 622 added content-addressed
`.rum` assets with 0 deletions and no production source change beyond the
one appended accessor (2,639 `.rum` total, all content-addressed); the
append keeps 10,303 bytes, adds 891, defines one `pub fn`, has no `cfg`
and no `unsafe`, and `c31_fixed_point_equality_events` occurs nowhere
outside its definition and one test call site; 21,010 binding rows =
10,505 required + 8,220 bound + 2,285 blocked over 133 bound rows / 57
families, with 18 active rows / 840 keys and 24 excluded / 1,445; all
6,475 entry program rows and all required/bound entry rows persist
verbatim; exactly 1,745 blocked rows convert to bound with evidence
naming their root probe and an existing sealed proof, no other blocked
row changes, and the batch6/batch7 keys stay uninstalled (570); the
program table carries 8,670 constituent plans over 8,220 keys (351
multi-record keys, maximum three), every explicit source/setup
content-addressed and every root hash an existing `.rum` file; proofs,
known losses, aliases, every provenance pin and the 133 canonical-source
pins hash-match, and the program-table pin counts 8,220; the 68
fingerprint cells and four certificate cells changed only in their own
columns and ownership went 9,003 → 9,006 with three new
`UnknownConstruct` rows; the lane's 5,426 files verify in both workspace
and lane with the nine feature-removal manifest edits (growth absent,
`test-continuation-slots` kept); the checkpoint evidence's 11 pins, the
sealed getter text, the own-omission pairs and their byte-equal public
effects, the fixed matrices' 20 × `(false,false)` / 10 × `(false,true)`
with resumed equal to queued in all 30 cells, and the `E0599`/`E0616`
probe all verify; and the Rust C31 count table equals the sealed 30
events. `report.py check --against-git` reports 2,036 "problems" solely
because its baseline is HEAD, which predates the whole uncommitted
campaign tree; the entry snapshot is the correct baseline and it shows
exactly the 12 + 622 change set.

**Continuation 14.** Entry state: 133 rows / 8,220 keys complete; 18
active rows / 840 keys; 24 director-excluded rows / 1,445 keys; proofs
749/43,390; encoding 0 of 18,840. Land the Q17 getter and the test-only
host bridge exactly, replay maintenance, and rerun
`local_production_acceptance` in both arms at one thread; then bind and
execute both complete 30-key `P1566`/`P1568` matrices through their real
route with the pair at multiplicity one and the own-omission `Native`
falsifier firing, then the remaining 16 rows / 780 keys (batch6 and
batch7 by hash identity, `P0813`/C49, `P1470`/`P1471`, `P2600`,
`P2611`/`P2612`), replay maintenance, rebuild the lane from the final
tree, and run the complete schedule and acceptance under the 1,500 s
bound. C2 stays Open at 118 Residual + 0 Lost; no settlement, no
promotion, no encoding key; `CHANGELOG.md` and S10 stay blocked. The
partial tree is the state to continue from; do not restart and do not
hand-edit a generated cell.

### Continuation 15 — fifteenth run: Q17's getter and bridge execute P1566/P1568 (60 keys) and batch6/7 adds 570 keys (145 rows / 8,850 keys), then P0813's private timeout flag opens Q18

**2026-11-15 — brief 739's fifteenth run executed continuation 14. Q17's
single append-only read-only `#[doc(hidden)]` `Session` getter and the
test-only `host_facade` bridge landed exactly as authorized, both
`P1566`/`P1568` complete 30-key matrices executed at the sealed sampling
point (60 keys, 20 × `(false,false)` + 10 × `(false,true)` per row,
own omission reversed in each sealed direction with equal public effects,
constant/missing/duplicate controls firing), and batch6/batch7 executed
570 keys over 571 canonical constituents (P0492 keeping its two
records). The campaign stands at 145 rows / 8,850 keys over 64 families
with 16,170 actual pauses and 24,816 lifecycle observations. The run then
stopped at `P0813`: its sealed
`C49HostOwnsEvalTimeoutAtTerminalBoundary` row needs the private
`HostSession.owns_eval_timeout` sampled after the host outcome and before
snapshot/fresh/projections, the field is private with no getter (E0599
and E0616 in both feature modes), and a second test-only accessor exceeds
Q17's one-accessor scope. Q18 authorizes exactly that one additional
test-only accessor with the reused host source byte-identical, then
`P0813`'s and `P2600`'s 30-key matrices and the remaining
`P1470`/`P1471` (90) and `P2611`/`P2612` (60) keys complete the
9,060-key active campaign.**
Run `cdx`, unit `brief-739-cdx-20261115-190214-1154517`, rc 0, peak
14,390,767,616 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 5,641,382 B, sha256 `61c2c703…`; report
`.temp/brief-739/report.json`, 3,610,085 B, sha256 `3ab4ba24…`; 287
recorded changes (275 added `.rum` assets, 12 modified, 0 deleted);
24 PASS / 5 FLAGGED / 1 FAIL (the FAIL is the campaign unfinished), 4
disclosures, 6 deviations, Q18; 21,225 citations verified.

**What landed.** Q17's getter is exactly the authorized shape:
`crates/rum-runtime/src/runtime/model/session.rs` keeps its 11,194-byte
prefix and gains 232 bytes (one `#[doc(hidden)]` impl with one
read-only `pub fn actual_message_checkpoint_state` returning
`(current_message_list_checkpoint.is_some(), current_message_list_dirty)`),
one new symbol, zero production call sites, unconditional, no `cfg`, no
`unsafe`, no field-visibility change, no dependency/manifest/lock edit.
`host_facade.rs` now `include!`s the unchanged `crates/rum-host/src/session.rs`
(sha `ef78a1de…`, byte-identical, still unmodified in git) inside the
module and adds exactly one `pub(crate)` forwarding accessor and no other
item. Both checkpoint rows execute their complete 30-key matrix at the
sealed sampling point — after the host outcome, before
snapshot/fresh/projections, on the executing thread — with the pair at
multiplicity one in both arms and the expected values taken from the
sealed records; the Rust expectation table is byte-equal to all 30 sealed
cells per row (resumed equal to queued). The own omission (P1566 queued
`(true,true)` against resumed `(false,false)`, P1568 the reverse) is
rejected by `Native` with every public effect equal, and the constant
`0,0` / `0,1`, missing and duplicate controls all fire. Batch6/7
(P0416 50, P0820 60, P0825 60, P1477 60, P2593 50; P0267 60, P0312 60,
P0323 60, P0410 55, P0492 55) were reauthenticated from the original
sealed packets (50 pins), installed and executed as 570 keys / 571
constituents at multiplicity one in both modes. 275 new `.rum` assets
(2,914 total, all content-addressed), all 8,220 entry program rows
verbatim, exactly 630 blocked keys converted to bound (60 checkpoint +
570 batch), the 30 `P0813` blocked rows keeping status and changing only
their evidence column, 9,301 constituent plans over 8,850 keys (352
multi-record, maximum three), 17,623 provenance rows with 133 canonical
pins and one unchanged absent packet path, the program-table pin at
8,850, all 15 immutable tables hash-unchanged, proofs 749/43,390
(151/8,255 packet-authenticated; 115/6,270 verified; 36/1,985 blocked;
598/35,135 absent) and the five known losses unchanged. Maintenance
replayed 68 fingerprint cells in one column and four certificate cells in
two columns; ownership stayed 9,006 rows with the regenerated discovery
exactly equal and no new unknown construct. The lane was rebuilt from the
final tree: 5,701 files (5,692 byte-identical + nine manifest edits),
tree `de4a57fa…`, artifact `84195207…`, manifest-diff `0ab8e71c…`,
release at one codegen unit, build 84.304587 s, growth feature absent,
`test-continuation-slots` kept, the discriminator true in both mandated
arms and false in the lane with its forced control. No production,
repository manifest, lock, dependency, doc or script change beyond the
one runtime accessor.

**The STOP.** `P0813` (1 row, 30 active keys, sealed family
`catch-body-result-dispatch`): the sealed
`C49HostOwnsEvalTimeoutAtTerminalBoundary` ten-field row reads
`HostSession.owns_eval_timeout` at the real host return, before snapshot,
fresh evaluation and projections. The field is private and no getter
exists; both compiled source-reuse probes fail with exactly `E0599`
(absent `c49_timeout_owned`) and `E0616` (private field). The sealed
native driver (`sha256 14d425e7…`) shows the sampling line and all 30
sealed records are zero, but a constant is not an observation. `P2600`
records the equivalent C47 terminal timeout field in its own 30 sealed
records. Q17 authorizes only the checkpoint accessor, so the run stopped
with zero binding/receipt/proof/execution credit; `P1470`/`P1471` (90),
`P2600` (30) and `P2611`/`P2612` (60) were authenticated only (210 keys
in total, 42 sealed pins, every own omission armed and caught).

**Q18 ruling — the timeout observer, then the last 210 keys.**
- Evidence verified independently: the 11 checkpoint pins and the sealed
  matrices; the 107 authentication pins across the checkpoint, guest and
  remaining records; the sealed C49/C47 drivers and their sampling
  points; the all-zero 30-cell matrices for both rows; and the
  `E0599`/`E0616` probes in both feature modes.
- Authorized, and nothing more: one additional unconditional test-only
  read-only accessor in the same include-based `host_facade::session`
  module returning `self.owns_eval_timeout`. The reused production host
  source stays byte-identical to `crates/rum-host/src/session.rs`
  (hash recorded and checked), the workspace host file stays unmodified,
  the only added item is that one test-only accessor, no production host
  body or field-visibility change, no copied host logic, no `cfg`, no
  `unsafe`, no new dependency, no `Cargo.toml`/`Cargo.lock` edit; the
  declaration is recorded by the production audit. An audit rejection is
  a STOP, never a widened predicate. If the source reuse cannot be kept
  byte-exact, STOP.
- Then bind and execute both complete 30-key `P0813`/`P2600` matrices at
  the sealed sampling point — after the host outcome, before
  snapshot/fresh/projections — from the executing thread, with the field
  recorded at multiplicity one in both arms and the expected value taken
  from the sealed records (`[0;10]` in every cell); a public phase row, a
  cached scalar, the snapshot or a constant is not an observation. The
  own-omission `Native` rejection must fire in each row's sealed
  direction and a constant-field control must fire, alongside missing and
  duplicate controls.
- Then execute the remaining `P1470`/`P1471` (90) and `P2611`/`P2612`
  (60) keys through their real routes, completing the 9,060-key active
  campaign (151 rows), and measure the complete schedule and acceptance
  under the 1,500 s bound per expanded-oracle build+run with the
  cumulative cost disclosed separately (this run: 626.062692 / 624.682699
  s; cumulative direct oracle commands 947.307843 / 846.898824 s; the
  complete 9,060-key schedule stays unmeasured while any key is
  unported).
- Keep the 36 current-blocked proof records, the 598 absent / 35,135,
  the 24 named exclusions / 1,445 and the requirement bag 175 / 10,505
  named; no settlement, no owner promotion, no encoding key; replay
  maintenance after the manifest change; rebuild the lane
  content-addressed from the final tree. Any further private access is a
  STOP and a question, never a substitution.
- Option 2 is declined: leaving 210 active keys blocked would weaken the
  frozen active bag without cause, and the accessor is test-only with no
  production byte changed.

**Host gate.** My own gate on the final tree is green: `cargo fmt --check`
rc 0; the one-thread retention certificate both arms (1 passed / 0 failed;
35.24 s / 35.00 s); the oracle `cm_` both arms — 43 passed / 0 failed /
2 ignored in the main binary (568.80 s / 569.73 s) plus child summaries,
reproducing the exact `CM RECEIPTS PASS calibration=P0001 rows=145
families=64 bound=8850 executed=8850 multiplicity=1 actual_pauses=16170
lifecycle=24816 campaign_remaining=210 PortBlocked=1445
full_CM_green=false`, `CM BINDINGS PASS required=10505 active=9060
declared_bound=8850 PortBlocked=1655 absent_packets=600
unresolved_targets=785 repl_placement=60 unported_programs=210
full_CM_green=false`, `CM CHECKPOINT RECEIPTS PASS row=P1566
required=30 bound=30 executed=30 multiplicity=1 actual_pauses=72
lifecycle=60 pair_false_false=20 pair_false_true=10 native_fields=10
full_CM_green=false` and the same for `row=P1568` (`actual_pauses=72
lifecycle=84`), `CM P1476 RECEIPTS PASS required=30 bound=30 executed=30
multiplicity=1 actual_pauses=102 lifecycle=132 native_fields=10
full_CM_green=false`, `CM PROOFS PASS imported=749/43390
packet_authenticated=151/8255 absent=598/35135 current_verified=115/6270
partial=true executions=0 SAME=0 full_CM_green=false`, `CM P0543
RECEIPTS PASS … lane=p0543-unchecked …`, `CM BUILD CONFIGURATION PASS
lane=mandated control=false checked_only_enforcement=true`,
`CM CANONICAL LEGACY PASS plans=796 frozen_fingerprints=796 executions=0`
and `CM LOSSES PASS prior_Lost=5 immutable_differences=3 C1_promotions=2
missing_original_baselines=2 encoding_runs=0 fresh_allowed_DIFF=0` — with
all ten checkpoint falsifiers and all four `P1476` falsifiers firing in
both arms; the ignored provenance audit both arms (`PROVENANCE AUDIT PASS
pins=22 diagnostics=4`); switched LP 5/5 both arms; and the workspace
4,315 passed / 0 failed / 65 ignored over 70 suite summaries in both
feature modes (the full oracle binary itself 549 passed / 0 failed /
4 ignored at 616.90 s; its only panics are the intentional negative
controls). My wrapper re-verified the lane binary (`84195207…`), the
recomputed tree (`de4a57fa…`), the manifest-diff (`0ab8e71c…`) and all
5,701 copied files before every oracle and workspace invocation. My
independent audit of the run closed with zero failures: entry continuity
with the fourteenth-run final tree plus only the three committed docs; the
exact 12 + 275 change set (0 deletions, one production accessor, every
added file a content-addressed asset); the 232-byte append keeping the
11,194-byte prefix with one `pub fn`, no `cfg`, no `unsafe` and four code
occurrences (definition, guest observation, bridge plus forward); the
host source at `ef78a1de…` unmodified in git with the include-based
module adding exactly one accessor; the 11 checkpoint pins, the sealed
observer and the own-omission directions with equal public effects; every
one of the 30 sealed cells per row equal to the Rust expectation table
with resumed equal to queued; the `E0599`/`E0616` probes in both modes and
the sealed C49 driver hash; 21,010 binding rows = 10,505 required + 8,850
bound + 1,655 blocked over 145 bound rows / 64 families, exactly 630
blocked→bound conversions with the only same-status edit the 30 `P0813`
evidence cells, and the 24 named exclusions / 1,445 plus six active rows /
210; all 8,220 entry program rows verbatim and 630 new ones; 9,301
constituent plans over 8,850 keys (352 multi-record, maximum three) with
every constituent content-addressed; 2,914 assets all content-addressed;
17,623 provenance rows with 133 canonical pins, one unchanged absent
packet path and only the program-table pin replaced (8,850); all 15
immutable tables hash-unchanged; proofs 749 rows / 43,390 keys and the
five losses; 68 + 4 maintenance cells in their columns; ownership 9,006
rows with the regenerated discovery exactly equal; the lane identity and
the nine manifest edits; and the sealed pause-ownership table equal to
the sealed `C2_PAUSE` owners on all 1,161 pause instances of the 570 new
keys. The worker's report passes `scripts/report.py check` with 21,225
citations verified; its 4,306/9/65 workspace lines are the documented
sandbox loopback `EPERM` artifact, reproduced by my host run as
4,315/0/65.

**Continuation 15.** Entry state: 145 rows / 8,850 keys complete; 6 active
rows / 210 keys (P0813, P1470, P1471, P2600, P2611, P2612); 24
director-excluded rows / 1,445; proofs 749/43,390; encoding 0 of 18,840.
Land the Q18 accessor exactly, replay maintenance, rerun acceptance in
both arms at one thread, execute the two timeout matrices and the
remaining 150 keys, replay maintenance, rebuild the lane from the final
tree, and run the complete 9,060-key schedule and acceptance under the
1,500 s bound. The partial tree is the state to continue from; do not
restart and do not hand-edit a generated cell.

### Continuation 16 — sixteenth run: Q18's timeout accessor lands exactly and both timeout packets authenticate (60 keys, 83 pins), then P2600's sealed own omission proves Output-only and Q19 rules the per-row sealed direction and the last 210 keys

**2026-11-15 — brief 739's sixteenth run executed continuation 15. Q18's
additional unconditional test-only read-only accessor landed exactly: one
method in the same include-based `host_facade::session` module returning
`self.owns_eval_timeout`; the reused production host source stays
byte-identical (sha `ef78a1de…`, still unmodified in git), the workspace
host file unmodified, no production body or field-visibility change, no
copied logic, no `cfg`, no `unsafe`, no dependency/manifest/lock edit.
Both timeout packets were reauthenticated against their original seals
(60 keys / 83 pins; per row `frozen_keys=30 real_valid=30
real_native_equal=30 real_timeout_zero=30 disabled_exact=30`; every real
native row `[0;10]` in both arms) with zero binding, receipt, proof or
execution credit. The sealed own-omission records then showed that
`P0813`'s 30 records all differ natively (25 sealed-valid; the five
`return_boundary.loop-store-reset` records are sealed-invalid and not
caught) while `P2600`'s 30 valid and caught records differ only in
structured output and keep identical native vectors in every record —
exactly the dropped `HostStructuredOutput { kind: Result, text: "43" }`
(first difference at byte 1,993; both native hashes `a43abd89…`; the
sealed collector's own verdict `C2 C47 HOST COLLECTOR total=1 SAME=0
DIFF=1 invalid=0`). Q18's own-omission `Native` predicate is therefore
unsupported for `P2600`; the run stopped at Q19 rather than invent a
difference, and the STOP is accepted with zero credit. Q19 authorizes
each row's own sealed direction — `P0813` Native, `P2600` Output — with
the timeout field itself held under a constant-field Native control,
then the remaining `P1470`/`P1471` (90) and `P2611`/`P2612` (60) keys
complete the 9,060-key active campaign (151 rows).**
Run `cdx`, unit `brief-739-cdx-20261115-215419-1262467`, rc 0, peak
14,854,934,528 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 1,418,749 B, sha256 `176bf641…`; report
`.temp/brief-739/report.json`, 2,706,882 B, sha256 `5ec03e77…`; 3
recorded changes (0 added, 3 modified, 0 deleted); 11 PASS / 6 FLAGGED /
1 FAIL (the Q19 conflict), 1 disclosure, 3 deviations, Q19; 12,589
citations verified; no manifest or lock hash changed.

**What landed.** `host_facade.rs` gains exactly one item in the existing
include-based module: `pub(crate) fn actual_owns_eval_timeout(&self) ->
bool { self.owns_eval_timeout }`. The included host source is unchanged
at `ef78a1de…` and the workspace host file is unmodified in git; no
production source, manifest, lock, dependency, doc or script changed.
The timeout authentication re-read the original sealed packets (c49 for
`P0813`, c47 for `P2600`) against their own seals: 30 frozen keys per
row, the real matrix valid in both arms with the terminal-boundary event
`C49HostOwnsEvalTimeoutAtTerminalBoundary` / C47 recorded `[0;10]`, the
disabled arm byte-equal, and zero credit. The own omission was then
classified from those sealed records: `P0813` 30 records, 25 valid and
caught, all 30 native-different and 4 semantic-same; `P2600` 30 records,
all valid and caught, all 30 semantic-different and all 30
native-identical with the timeout row unchanged. Maintenance replayed
under the flat rot guard (which hashes `crates/**/*.rs|py|toml`, so the
test-side source change expires the contracts): 68 fingerprint cells
regenerated in their own column and mechanically installed, four C01/C02
certificate cells in two columns; no other column or row changed, the
18 obligation tables are hash-unchanged from entry, and ownership stayed
9,006 rows across six inventories with no added, removed or changed row.
The lane was rebuilt from the final tree: 5,701 files (5,692
byte-identical + nine feature-removal manifest edits), tree
`d52d4ea4…`, manifest-diff `5931f199…`, release at one codegen unit,
build 81.34 s, the discriminator true in both mandated arms and false in
the lane with its forced control. The lane artifact is byte-identical to
the previous run's (`84195207…`) because the added accessor is currently
uncalled and stripped; the next run's observation call changes it.
The full oracle target inside the workspace acceptance passed 549/0/4 in
both modes at 623.17 s / 617.58 s with zero filtered tests, typed-slot
10/0/1 at 58.91 s / 59.51 s, switched LP 5/0/0 in both modes; the
workspace raws (4,329/11/65) contain the nine documented loopback
`EPERM` failures plus the two forced-red probe children, reproduced at
one thread as 0/9/0 `EPERM`; the source tree and all four target
binaries are sealed in `acceptance-identities.json` under the
hash-identity reuse ruling.

**The STOP.** Q18 required the own-omission `Native` rejection in each
row's sealed direction. For `P0813` that holds: all 30 sealed records
differ natively, 25 of them valid and caught. For `P2600` it does not:
all 30 records are valid and caught, `semantic_same=false` in all 30 and
`native_same=true` in all 30, including the timeout row itself (the
witness removes exactly `HostStructuredOutputKind::Result` with scalar
`43`, first difference at byte 1,993, native hashes equal
`a43abd89…`). A sealed no-change native vector cannot supply a Native
falsifier; the only sealed direction is the structured-output
comparison, which is the comparison the sealed collector itself reports
(`SAME=0 DIFF=1`). Changing the named predicate is a director ruling, so
the run stopped with zero binding/receipt/proof/execution credit and the
remaining 210 keys unported; no difference was invented and no sealed
record was touched.

**Q19 ruling — reject each sealed own omission in its own sealed
direction; then finish the campaign.**
- Evidence verified independently: the 83 pins and the two packets'
  seals; the real matrices (30 keys per row, `[0;10]` in both arms at
  the terminal boundary, disabled byte-equal); the own-omission
  classification per row; the c47 witness and its native hashes; the
  c49 five sealed-invalid records; and the `E0599`/`E0616` history that
  motivated the accessor.
- Authorized, and nothing more: `P2600`'s own omission rejects as
  `Output` — the dropped structured result push in the public output —
  and `P0813`'s rejects as `Native` — its differing native vectors. No
  row may use a direction its sealed record does not show; a direction
  not shown by the sealed record is a STOP, never a substrate change.
- The observed timeout field stays under Native control: a
  constant-field control that substitutes a nonzero value for
  `owns_eval_timeout` in the receipt must be rejected as `Native`,
  together with missing and duplicate controls. The real cells remain
  `[0;10]` per row, at multiplicity one in both arms, sampled at the
  sealed boundary (after the host outcome, before
  snapshot/fresh/projections, from the executing thread); a public
  phase row, a cached scalar, the snapshot or a constant is not an
  observation.
- Build the own-omission control on sealed-valid representative cells;
  the five sealed-invalid `P0813` records
  (`return_boundary.loop-store-reset`, raw `valid=false`,
  `caught=false`) cannot carry the falsifier and must be recorded as
  sealed-invalid, never skipped silently or turned into passes.
- Then execute the remaining `P1470`/`P1471` (90) and `P2611`/`P2612`
  (60) keys through their real routes, completing the 9,060-key active
  campaign (151 rows), and measure the complete schedule and acceptance
  under the 1,500 s bound per expanded-oracle build+run, with the
  cumulative cost disclosed separately (this run: full oracle 623.17 /
  617.58 s inside the workspace; workspace 1,053.567114 / 919.577240 s;
  cumulative mode-labelled commands 1,138.589422 / 1,019.704348 s; the
  complete 9,060-key schedule stays unmeasured while any key is
  unported).
- Keep the 36 current-blocked proof records, the 598 absent / 35,135,
  the 24 named exclusions / 1,445 and the requirement bag 175 / 10,505
  named; no settlement, no owner promotion, no encoding key; replay
  maintenance after the fixture change; rebuild the lane
  content-addressed from the final tree. Any further private access is a
  STOP and a question, never a substitution.
- Option 2 is declined: a revised evidence requirement asking for a
  native-differentiating `P2600` witness would demand a fact the sealed
  packet does not record; any such witness would have to come from a
  separately reviewed evidence program, never from a re-pin of this
  history or an invented difference.

**Host gate.** My own gate on the final tree is green: `cargo fmt
--check` rc 0; the one-thread retention certificate both arms (1 passed
/ 0 failed; 35.16 s / 35.00 s); the oracle `cm_` both arms — 43 passed /
0 failed / 2 ignored (568.67 s / 565.47 s) plus child summaries with the
exact `CM RECEIPTS PASS calibration=P0001 rows=145 families=64
bound=8850 executed=8850 multiplicity=1 actual_pauses=16170
lifecycle=24816 campaign_remaining=210 PortBlocked=1445
full_CM_green=false`, `CM BINDINGS PASS required=10505 active=9060
declared_bound=8850 PortBlocked=1655 absent_packets=600
unresolved_targets=785 repl_placement=60 unported_programs=210
full_CM_green=false`, `CM CHECKPOINT RECEIPTS PASS row=P1566 … 72/60 …
pair_false_false=20 pair_false_true=10 …` and `row=P1568 … 72/84 …`,
`CM P1476 RECEIPTS PASS required=30 bound=30 executed=30 multiplicity=1
actual_pauses=102 lifecycle=132 native_fields=10 full_CM_green=false`,
`CM PROOFS PASS imported=749/43390 packet_authenticated=151/8255
absent=598/35135 current_verified=115/6270 partial=true executions=0
SAME=0 full_CM_green=false`, `CM P0543 RECEIPTS PASS … 55/55/55 …
lane=p0543-unchecked …`, `CM BUILD CONFIGURATION PASS lane=mandated
control=false checked_only_enforcement=true` and its forced control,
`CM CANONICAL LEGACY PASS plans=796 frozen_fingerprints=796
executions=0` and `CM LOSSES PASS prior_Lost=5
immutable_differences=3 C1_promotions=2 missing_original_baselines=2
encoding_runs=0 fresh_allowed_DIFF=0` lines; the ignored provenance
audit both arms (`PROVENANCE AUDIT PASS pins=22 diagnostics=4`);
switched LP 5/5 both arms; and the workspace 4,315 passed / 0 failed /
65 ignored over 70 suite summaries in both feature modes (the full
oracle binary itself 549 passed / 0 failed / 4 ignored at 616.69 s /
615.38 s; typed-slot 10/0/1 at 61.23 s / 58.12 s). My wrapper
re-verified the lane binary (`84195207…`), the recomputed tree
(`d52d4ea4…`), the manifest-diff (`5931f199…`), the nine manifest edits
and all 5,701 copied files before every oracle and workspace invocation.
My independent audit of the run closed with zero failures: continuity
with the fifteenth-run final tree (the previous lane copy differs only
by the nine expected manifest edits, the three committed docs and this
run's three files); the exact 3-file change set; the host source
`ef78a1de…` unmodified in git with exactly one added accessor and no
other item; the timeout authentication reproduced from a copy of the
worker's script with the same sealed verdicts; the worker's own final
audit reproduced from a redirected copy with all its verdict lines
(5,701 files, 3 modified, 0 added/deleted, 145 family receipts, 8,850
executions, the 549/0/4 and 10/0/1 reuse, LP 5/0/0 and the nine EPERM
reproductions); the 68 fingerprint cells confined to their column and
equal to the regenerated file; the four certificate cells confined to
their two columns; all 18 obligation tables hash-unchanged; the
inventory exactness (21,010 = 10,505 + 8,850 + 1,655; 145 bound rows /
64 families; six active rows / 210 keys; proofs 749/43,390; five
losses; 17,623 provenance rows); and the report's 12,589 citations.
C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner is
promoted, no guest executes beyond the receipts, no encoding key runs,
and `CHANGELOG.md` and S10 stay blocked. The partial tree is the state
to continue from.

**Continuation 16.** Entry state: 145 rows / 8,850 keys; 6 active rows /
210 keys (P0813, P1470, P1471, P2600, P2611, P2612); 24 director-
excluded rows / 1,445; proofs 749/43,390; encoding 0 of 18,840. The
Q18 accessor is landed but not yet called; both timeout packets are
authenticated with zero credit; maintenance is replayed; the lane is
rebuilt at tree `d52d4ea4…`. Land the timeout observation at the sealed
sampling point and execute both complete 30-key matrices with the
per-row own-omission direction (`P0813` Native on a sealed-valid
representative, `P2600` Output), the constant-field Native control and
the missing/duplicate controls; then execute `P1470`/`P1471` (90) and
`P2611`/`P2612` (60) through their real routes, completing the
9,060-key active campaign (151 rows); measure the complete schedule and
acceptance under the 1,500 s bound; keep the 36 current-blocked proof
records, the 598 absent / 35,135, the 24 named exclusions / 1,445 and
the requirement bag 175 / 10,505 named; replay maintenance, rebuild the
lane content-addressed, and treat any further private access as a STOP
and a question. The partial tree is the state to continue from; do not
restart and do not hand-edit a generated cell.

### Continuation 17 — seventeenth run: Q19's per-row directions execute 150 keys (149 rows / 9,000 keys over 67 families) and both timeout packets authenticate, then Q20 authorizes one test-only include-based adapter driver for the last 60 keys

**2026-11-16 — brief 739's seventeenth run executed continuation 16 and is
accepted at its STOP. The Q19 matrices executed both timeout rows in
their own sealed directions (`P0813` Native on a sealed-valid
representative with the sealed `catch.eval.total` difference
`[1,1,0,0,0,0,0,0,0,0] → [1,0,1,0,0,0,1,0,0,0]`, public effects equal;
`P2600` Output dropping exactly the structured Result `"43"` with the
native vectors equal) with the constant-field, missing and duplicate
timeout controls all rejected `Native`, and the five sealed-invalid
`P0813` `return_boundary.loop-store-reset` records recorded as
sealed-invalid with zero falsifier credit; the fixed-point pair
`P1470`/`P1471` executed with its own pass-publication omission
rejected `Native` on the sealed omitted-resumed vector with queued
unchanged. The campaign moved from 145 rows / 8,850 keys to 149 rows /
9,000 keys over 67 families (16,574 actual pauses, 25,144 lifecycle
observations, 9,451 constituent plans, 38 new content-addressed
assets). The run then stopped at `P2611`/`P2612`, whose sealed C52
adapter rows need `worker::run_segment` and `ExecuteSegment` at the
executing-thread terminal boundary: the unchanged-source include
compiles, but the private `worker` module rejects the call with `E0603`
in both modes, and the temporary Q7 reachability probe was restored
byte-exactly (`suspension_oracle.rs` `adb6aae9…`; all seven included
adapter sources unchanged). The STOP is accepted with zero
binding/receipt/proof/execution credit: all 60 keys were authenticated
against their seals (70 pins) and remain unported. Q20 authorizes one
narrow test-only include-based driver over the same seven unchanged
adapter sources, then the last 60 keys complete the 9,060-key active
campaign (151 rows) and the complete schedule and acceptance are
measured under the 1,500 s bound.**
Run `cdx`, unit `brief-739-cdx-20261115-234837-1332615`, rc 0, peak
13,717,585,920 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 6,217,642 B, sha256 `cf4cc7c5…`; report
`.temp/brief-739/report.json`, 2,548,689 B, sha256 `499b6dfe…`; 48
recorded changes (38 added, 10 modified, 0 deleted); 13 PASS / 2
FLAGGED / 1 FAIL (the Q20 private-access stop), 1 disclosure, 6
deviations, Q20; 11,540 citations verified; no manifest or lock hash
changed.

**What landed.** `host_facade.rs` was not touched this run: the Q18
accessor is read at the sealed sampling point by the shared fixture
runner (`fixtures/obligations/guest.rs`), which samples
`actual_owns_eval_timeout()` immediately after the terminal host
outcome and before `snapshot_to_bytes`, the checkpoint fresh request
and the projections. `execution_tests.rs` gains `cm_timeout_receipts_exact`
and `cm_fixedpoint_receipts_exact`; both execute all bound plans
through the real host routes (P0813 via real eval/Catch; P2600 via the
sealed c47 driver through `HostSession::call`; P1470/P1471 through the
pure C8 next-pass driver in real fixed-point frames) and validate the
13-channel/lifecycle rows unchanged. The timeout packet
authentication reproduced the same sealed verdicts as the sixteenth
run (60 keys / 88 pins; per row `frozen_keys=30 real_valid=30
real_native_equal=30 real_timeout_zero=30 disabled_exact=30`;
`P0813` own 25 valid / 5 sealed-invalid; `P2600` own 30 valid with
`native_same=30`), the fixed-point packet authenticated 90 keys (14
pins; both rows valid with pause owners exact), and the adapter packet
authenticated 60 keys (70 pins; all 60 own omissions valid and caught
with `native_same=true` and `semantic_same=false`). Maintenance replayed
under the flat rot guard: 68 fingerprint cells regenerated in their own
column and mechanically installed, four C01/C02 certificate cells in
their two columns (both generation-stage stale cells were ordering
artifacts, resolved by the replay before acceptance), no other column
or row changed, the 15 immutable obligation tables hash-unchanged, and
ownership stayed 9,006 rows with no delta. The lane was rebuilt from
the final tree: 5,739 files (5,730 byte-identical + nine
feature-removal manifest edits), tree `02aa9d42…`, manifest-diff
`c392e13a…`, release at one codegen unit, build 65.668 s / wrapper
67.181969 s, discriminator true in both mandated arms and false in the
lane with its forced control. The full oracle target inside the
workspace acceptance passed 551/0/4 in both modes at 706.39 s / 710.2 s
with zero filtered tests; typed-slot 10/0/1; switched LP 5/0/0 in both
modes; the workspace raws (4,333/11/65) contain the nine documented
loopback `EPERM` failures plus the two forced-red probe children,
reproduced at one thread as 0/9/0 `EPERM`; the source tree and all four
target binaries are sealed in `acceptance-identities.json` under the
hash-identity reuse ruling.

**The STOP.** Q19's last sentence makes any further private access a
STOP and a question. `P2611`/`P2612`'s sealed C52 rows require the
adapter's executing-thread terminal boundary: the pinned
`jupyter_driver.rs` constructs `ExecuteSegment` and calls
`run_segment`, then samples the native rows before the snapshot and
fresh work. The adapter's `mod worker;` is private and `run_segment` is
not re-exported, so an external test crate cannot reach it (`E0603` in
both modes); the unchanged source itself compiles in the test crate.
The run authenticated the 60 keys against their seals with zero credit
and did not add a bridge. The temporary probe that proved the
reachability boundary was restored byte-exactly, with all seven
production adapter sources unchanged, so the tree carries no claim of
private access it did not receive.

**Q20 ruling — one test-only include-based adapter driver; then the
last 60 keys.**
- Evidence verified independently: the seven included source hashes
  (`mod.rs` `b3feb8ef…`, `worker.rs` `94df4c10…`, `execute.rs`
  `cbdc0f8d…`, `input_retirement.rs` `5ca28f9a…`, `iopub.rs`
  `ac5ed850…`, `render.rs` `ca824c5e…`, `model.rs` `69c59a46…`); the
  byte-exact probe restore; the `E0603` failures in both modes; the
  pinned C52 driver at `jupyter_driver.rs:50` and `:75`; and the 60
  sealed own-omission records, every one `valid=true`,
  `native_same=true`, `semantic_same=false`.
- Authorized, and nothing more: compile those seven unchanged adapter
  sources into one test-only include-based module, in the Q17 host
  form, and add exactly one local observation driver in that same
  module. It may construct `ExecuteSegment` and call `run_segment`,
  capturing the complete native row bag at the terminal outcome on the
  executing thread, before snapshot/projections/fresh work.
- Constraints: no production byte, field visibility, Cargo graph or
  lock change; no copied logic; the receipt constructor unchanged; the
  temporary reachability probe stays removed; the driver declaration is
  recorded; an audit rejection or a non-byte-exact reuse is a STOP.
  (The include mechanics are proven: a director probe compiled an
  `include!`-ed module whose path-based submodules resolve relative to
  the included file, so the private `worker` module and its
  `run_segment` are reachable from a driver appended in the same
  module.)
- Then both 30-key matrices (`P2611` Eval route, `P2612` Resume route)
  at multiplicity one in both arms at the sealed sampling point. Each
  record's own omission rejects in its sealed direction: all 60 sealed
  records are native-identical and semantic-different, so the direction
  is `Output`; no record may use `Native`, and a direction the sealed
  record does not show is a STOP, never a substitution. The observed
  boundary row bag stays under Native control: a substituted nonzero
  value in it must be rejected as `Native`, alongside missing and
  duplicate controls. Any sealed-invalid record is recorded as
  sealed-invalid with zero credit (none here: all 60 are valid and
  caught).
- Then complete the 9,060-key active campaign (151 rows), and measure
  the complete schedule and acceptance under the 1,500 s bound per
  expanded-oracle build+run, with the cumulative cost disclosed
  separately (this run: partial oracle 722.71 / 728.45 s inside the
  workspace; workspace 1,168.979190 / 1,040.287780 s; all recorded
  leaves 3,136.034974 s; lane build 67.181969 s; the 9,060-key
  schedule stays unmeasured while any key is unported).
- Keep the 36 current-blocked proof records, the 598 absent / 35,135,
  the 24 named exclusions / 1,445 and the requirement bag 175 / 10,505
  named; no settlement, no owner promotion, no encoding key; replay
  maintenance after the fixture change; rebuild the lane
  content-addressed from the final tree. Any further private access is
  a STOP and a question, never a substitution.
- Option 2 is declined: a separately reviewed public-adapter evidence
  program would add permanent public observation surface for a
  test-only need and would not be the sealed route. The sealed C52
  evidence itself was produced through `worker::run_segment` on the
  executing thread by the pinned driver, and no public adapter API
  exports executing-thread native samples; the driver preserves that
  route exactly with no production change.

**Host gate.** My own gate on the final tree is green: `cargo fmt
--check` rc 0; the one-thread retention certificate both arms (1 passed
/ 0 failed; 36 s / 36 s); the oracle `cm_` both arms — 45 passed / 0
failed / 2 ignored (616.05 s / 618.09 s) plus child summaries with the
exact `CM RECEIPTS PASS calibration=P0001 rows=149 families=67
bound=9000 executed=9000 multiplicity=1 actual_pauses=16574
lifecycle=25144 campaign_remaining=60 PortBlocked=1445
full_CM_green=false`, `CM BINDINGS PASS required=10505 active=9060
declared_bound=9000 PortBlocked=1505 absent_packets=600
unresolved_targets=785 repl_placement=60 unported_programs=60
full_CM_green=false`, `CM TIMEOUT RECEIPTS PASS row=P0813 … 30/30/30 …
actual_pauses=49 lifecycle=70 timeout_zero=30 …` and `row=P2600 …
49/60 …`, `CM FIXEDPOINT RECEIPTS PASS row=P1470 … 45/45/45 … 153/90
…` and `row=P1471 … 153/108 …`, `CM CHECKPOINT RECEIPTS PASS
row=P1566 …` and `row=P1568 …`, `CM P0543 RECEIPTS PASS … 55/55/55 …
lane=p0543-unchecked …`, `CM PROOFS PASS imported=749/43390
packet_authenticated=151/8255 absent=598/35135 current_verified=115/6270
partial=true executions=0 SAME=0 full_CM_green=false`, `CM BUILD
CONFIGURATION PASS lane=mandated control=false
checked_only_enforcement=true` and its forced control, `CM CANONICAL
LEGACY PASS plans=796 frozen_fingerprints=796 executions=0` and `CM
LOSSES PASS prior_Lost=5 immutable_differences=3 C1_promotions=2
missing_original_baselines=2 encoding_runs=0 fresh_allowed_DIFF=0`
lines, all new falsifiers (`P0813`/`P2600` sealed-valid own omission,
constant-timeout, missing-timeout, duplicate-timeout; `P1470`/`P1471`
own-pass-publication-omission) firing in both arms, the five sealed
C49 invalid records printed with zero credit, the ignored provenance
audit both arms (`PROVENANCE AUDIT PASS pins=22 diagnostics=4`),
switched LP 5/5 both arms, and the workspace 4,317 passed / 0
failed / 65 ignored in both feature modes over 70 suite
summaries (the full oracle binary itself 45/0/2 under the `cm_`
filter). My wrapper re-verified the lane binary (`1a1c9dc6…`), the
recomputed tree (`02aa9d42…`), the manifest-diff (`c392e13a…`), the
nine manifest edits and all 5,739 copied files before every oracle and
workspace invocation; the wrapper's first form exported the wrong
variable name for the manifest diff and was refused by the oracle
identity check (a director-script defect, corrected against the test
source and re-run; no tree change). My independent audit of the run
closed with zero failures: the entry tree 5,701 files → 5,739 with
exactly 10 modified + 38 added test-side files and nothing else; the
host source `ef78a1de…` unmodified in git; the timeout, fixed-point
and adapter authentications reproduced from copies with byte-identical
JSON output; the four new rows' sealed representative differences
exact (`P0813` `catch.eval.total` `[1,1,0,…] → [1,0,1,…]`, `P2600`
counters equal with the Result `"43"` drop); the 68 fingerprint cells
and four certificate cells confined to their columns; the 150
blocked→bound moves with only `program`/`evidence` edited and the
program table 8,850 → 9,000; provenance 17,623 → 17,883; all 15
immutable obligation tables unchanged; the 5,739-file lane identity
recomputed independently; the temporary probe root restored
byte-exactly; and the report's 11,540 citations. C2 stays Open at 118
Residual + 0 Lost; no row settles, no owner is promoted, no guest
executes beyond the receipts, no encoding key runs, and `CHANGELOG.md`
and S10 stay blocked. The partial tree is the state to continue from.

**Continuation 17.** Entry state: 149 rows / 9,000 keys; 2 active rows
/ 60 keys (`P2611`, `P2612`); 24 director-excluded rows / 1,445;
proofs 749/43,390; encoding 0 of 18,840. The Q19 matrices are executed
and maintenance is replayed; the lane is rebuilt at tree
`02aa9d42…`. Add the Q20 test-only include-based adapter driver over
the seven unchanged sources, execute both complete 30-key matrices with
the per-record `Output` direction and the constant/missing/duplicate
controls on the observed boundary row bag, and complete the 9,060-key
active campaign (151 rows); measure the complete schedule and
acceptance under the 1,500 s bound; keep the 36 current-blocked proof
records, the 598 absent / 35,135, the 24 named exclusions / 1,445 and
the requirement bag 175 / 10,505 named; replay maintenance after the
fixture change; rebuild the lane content-addressed; and treat any
further private access as a STOP and a question. The partial tree is
the state to continue from; do not restart and do not hand-edit a
generated cell.

### Continuation 18 — eighteenth run: Q20's parent-module driver fails `E0603` in both modes with zero credit and exact restoration, and Q21 places the one observation driver inside a nested worker include (compile-verified)

**2026-11-16 — brief 739's eighteenth run executed continuation 17 and is
accepted at its STOP. It built Q20's exact layout (the three root shims
plus an `adapter_facade.rs` that includes `adapter/mod.rs` and appends one
`observe_segment`) and both feature modes rejected it with exactly three
`E0603` diagnostics — the two `ExecuteSegment` uses and the `run_segment`
call — with no unrelated compiler error and no test executed. The cause is
module privacy, not the include mechanics: a private item is visible only
inside its defining module and that module's descendants, and Q20's driver
sat in `worker`'s parent (`adapter`), while the sealed C52 driver was
`worker`'s own child (`crates/rum-jupyter/src/adapter/worker/c52.rs`,
`use super::*`, appended as `mod c52` to `worker.rs` at
`.temp/g15-c2/c52/assemble_jupyter.py:51,53`). My Q20 wording placed the
driver one module too high; the STOP is correct. It carries zero
binding/receipt/proof/execution credit and zero retained changes. The
temporary facade and root shims were restored byte-exactly
(`suspension_oracle.rs` `adb6aae9…`; the facade absent; all seven adapter
sources at their sealed hashes `mod.rs` `b3feb8ef…`, `worker.rs`
`94df4c10…`, `execute.rs` `cbdc0f8d…`, `input_retirement.rs`
`5ca28f9a…`, `iopub.rs` `ac5ed850…`, `render.rs` `ca824c5e…`, `model.rs`
`69c59a46…`); the run's own 5,331-file hash scope is entry-identical and
the full 5,739-file workspace equals the seventeenth run's final tree
except this commit's three docs. Q21 authorizes exactly one nested
test-only include of the unchanged `worker.rs` with the single observation
driver inside it; I compile-verified that layout in both modes before
ruling, and the last 60 keys then continue.**
Run `cdx`, unit `brief-739-cdx-20261116-031702-1444994`, rc 0, peak
884,985,856 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 943,818 B, sha256 `43339391…`; report
`.temp/brief-739/report.json`, 36,570 B, sha256 `aff1f278…`; 0 recorded
changes (0 added, 0 modified, 0 deleted), 0 disclosures; 6 PASS / 1
FLAGGED / 1 FAIL (the Q21 layout question), 3 deviations, 1 question; 121
citations verified; no manifest or lock hash changed and no git or network
command ran.

**What the run did.** Exactly Q20's layout: `suspension_oracle.rs` gained
the `extern crate self as rum_host;` shim, the `#[path]` `kernel_model`
re-export of the unchanged `rum-jupyter/src/model.rs`, and the
`adapter_facade` declaration; `adapter_facade.rs` included the unchanged
`adapter/mod.rs` and appended one driver calling `worker::ExecuteSegment`
and `worker::run_segment`. `cargo check -p rum-stdlib --test
suspension_oracle` failed both arms at 2.631346 s / 1.830262 s with
exactly the three private-item diagnostics and nothing else; no test ran.
All four recorded leaf commands total 5.458240 s. No binding, receipt,
proof, execution or encoding credit; no acceptance suite, no maintenance
replay (nothing retained), no lane rebuild, and the complete 9,060-key
schedule stays unmeasured. The run also re-verified the historical pins
(`jupyter_driver.rs` `3c10ddfb…`, `assemble_jupyter.py` `94881113…`, the
full-proofs packet `15740744…`) and the sealed placement
`parent=adapter::worker child=c52`.

**Q21 ruling — the nested worker include, and nothing more.**
- The layout is authorized exactly as I compile-verified it:
  `suspension_oracle.rs` gains only those three root items, and the new
  `crates/rum-stdlib/tests/suspension_oracle/adapter_facade.rs` contains
  `include!("../../../rum-jupyter/src/adapter/mod.rs");` followed by
  `pub(crate) mod observed_worker { include!("../../../rum-jupyter/src/adapter/worker.rs");
  <the one driver> }`. The driver may construct `ExecuteSegment::{Eval,
  Resume}`, call `run_segment`, and sample
  `Session::logical_profile_test_rows()` at the terminal outcome on the
  executing thread, before snapshot/projections/fresh work; the shared
  fixture supplies the ledger lease/reset (and the probe begin/end the
  sealed route uses) around it.
- Compiling `worker.rs` a second time is authorized only inside that one
  nested module, with the seven source files byte-identical to the hashes
  above. The driver's only calls into the worker copy are
  `ExecuteSegment::{Eval,Resume}` construction and `run_segment`; any
  other private item, a second driver or any other appended item is
  outside this authorization and a STOP. No production byte, field
  visibility, Cargo graph or lock change; no copied logic; the receipt
  constructor unchanged; the temporary reachability probe stays removed;
  the declaration is recorded; an audit rejection or non-byte-exact reuse
  is a STOP.
- My own verification of the layout (director probe, not new evidence
  from the worker): the exact module shape type-checks with rc 0 in both
  arms at 1.60 s / 1.66 s under `scripts/memory-scope.sh` 14G at `-j8`
  offline; a calibration renaming `run_segment` to a nonexistent function
  fires `E0425`, so the facade is genuinely compiled; the probe was then
  removed and the tree restored byte-exactly (`suspension_oracle.rs`
  `adb6aae9…`, facade absent, the 5,739-file snapshot equal to the sealed
  lane workspace except the three committed docs).
- Option 2 is declined: making `ExecuteSegment` and `run_segment`
  `pub(super)` would add permanent production visibility for a test-only
  need and would alter the production source the sealed evidence was
  produced from; the nested include keeps production bytes exact and
  reproduces the sealed driver's own placement.
- Then both 30-key matrices (`P2611` Eval, `P2612` Resume) at multiplicity
  one in both arms at the sealed sampling point, each record's own
  omission in its sealed direction — all 60 sealed records are
  `valid=true`, `native_same=true`, `semantic_same=false`, so `Output`; a
  direction the sealed record does not show is a STOP — with the observed
  boundary row bag under a constant-field Native control plus missing and
  duplicate controls (any sealed-invalid record is recorded as
  sealed-invalid at zero credit; none here). Then the last 60 keys
  complete the 9,060-key active campaign (151 rows), and the complete
  schedule and acceptance are measured under the 1,500 s bound per
  expanded-oracle build+run, cumulative cost disclosed separately (this
  run: compile 2.631346 / 1.830262 s; leaves 5.458240 s; schedule
  unmeasured).
- Keep the 36 current-blocked proof records, the 598 absent / 35,135,
  the 24 named exclusions / 1,445 and the requirement bag 175 / 10,505
  named; no settlement, no owner promotion, no encoding key; replay
  maintenance after the fixture change; rebuild the lane
  content-addressed from the final tree; any further private access is a
  STOP and a question.

**Host gate.** The run's restoration and diagnostics reproduce: the
run's own audit (`audit.py`) prints `FINAL TREE IDENTITY PASS files=5331
added=0 modified=0 deleted=0`, `Q20 DIAGNOSTIC EXACT PASS arm=default
errors=3 codes=E0603 items=ExecuteSegment::Eval,ExecuteSegment::Resume,run_segment
unrelated_errors=0 tests_run=0` (same for `no-default`), the unchanged
`COMMITTED BINDINGS PASS required_rows=175 required_keys=10505 bound_rows=149
bound_keys=9000 active_rows=2 active_keys=60 excluded_rows=24
excluded_keys=1445 blocked_total=1505 multiplicity=1`, zero current-run
credit, the unchanged proof classes and counts (`749/43390`,
`151/8255`, `598/35135`, `115/6270`, `36/1985`), the unchanged known
losses, ownership (964 G13 rows / 44 owners / 3,025 keys / 920
NotYetBound; 51 Open owners), the three historical pins, the sealed
driver placement, the unchanged maintenance pins and `MAINTENANCE REUSE
PASS retained_source_changes=0 replay=not_required
acceptance_started=false stale_derived_cell_failures=0
inventory_hand_edits=0`. I recomputed the run's 5,331-file hash scope
myself (equal entry-to-final), compared the full 5,739-file workspace
with the seventeenth run's sealed lane identity (equal except `TODO.md`
and the two committed briefs), and verified the seven adapter hashes and
`adb6aae9…` before and after my probe. The report's 121 citations
verify. C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner
is promoted, no guest executes, no encoding key runs, and `CHANGELOG.md`
and S10 stay blocked. The partial tree is the state to continue from.

**Continuation 18.** Entry state: 149 rows / 9,000 keys; 2 active rows /
60 keys (`P2611`, `P2612`); 24 director-excluded rows / 1,445; proofs
749/43,390; encoding 0 of 18,840. Add the Q21 nested include and the one
driver exactly as ruled, execute both 30-key matrices with the per-record
`Output` direction and the constant/missing/duplicate controls, complete
the 9,060-key campaign (151 rows), and measure the complete schedule and
acceptance under the 1,500 s bound. The partial tree is the state to
continue from; do not restart and do not hand-edit a generated cell.

### Continuation 19 — nineteenth run: the Q21 nested include compiles in both modes, the historical `c2_probe::begin/end` requirement fails `E0433` (zero credit, exact restore), and Q22 rules those calls provenance-only

**2026-11-16 — brief 739's nineteenth run executed continuation 18 and is
accepted at its STOP. It built the Q21 layout exactly (the three root
shims, the `adapter_facade.rs` include of the unchanged
`adapter/mod.rs`, and the nested `observed_worker` module that includes
the unchanged `worker.rs` and holds the one observation driver): both
feature modes compiled with rc 0 (exit 0, 0.878356 s / 0.874020 s, no
test run), confirming the layout I ruled. It then wired the shared
fixture with the historical probe calls Q21's parenthetical named, and
both modes rejected them with exactly two `E0433` diagnostics per arm
(`could not find c2_probe in rum_runtime`: `begin` and `end`), because
the module is scratch-only; the run stopped rather than omit a named
call, restored every temporary edit byte-exactly, and claimed no credit.
The STOP is correct and the fault is my Q21 wording: the historical
`c2_probe` is witness-tree instrumentation (`prepare_native_campaign.py:7,8`
copies `mutant_support.rs` as `c2_probe.rs`; `begin`/`end` only toggle
its `ACTIVE` flag), not part of the adapter route or the observation.
Q22 rules those two calls provenance-only: the live fixture keeps the
live lease/reset, the adapter route, the terminal native sample and all
controls, and does not import or reproduce the probe.**
Run `cdx`, unit `brief-739-cdx-20261116-033720-1450980`, rc 0, peak
595,234,816 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 837,672 B, sha256 `1a3faa22…`; report
`.temp/brief-739/report.json`, 39,989 B, sha256 `b2dc25df…`; 0 recorded
changes (0 added, 0 modified, 0 deleted), 3 disclosures; 5 PASS / 1
FLAGGED / 1 FAIL (the Q22 probe question), 4 deviations, 1 question; 103
citations verified; no manifest or lock hash changed and no git or
network command ran.

**What the run did.** The nested include compiled in both arms with no
test executed. The temporary shared-fixture probe check failed with
exactly `E0433` twice per arm, and the run's source-scope verdict reads
`Q21 PROBE SOURCE SCOPE STOP live_runtime_references=0
live_module=absent historical_probe_sha256=512785d2…
required_calls=jupyter_driver.rs:47,77
substitution_authorized=false`. It restored `suspension_oracle.rs`, the
facade and `fixtures/obligations/guest.rs` byte-exactly (the attempted
bytes and patches are preserved under `.temp/brief-739/`), so the
5,740-file entry hash scope is final-identical; the sealed C52 packets
re-authenticated at both rows (`frozen_keys=30 real_valid=30
disabled_exact=30 own_valid=30 own_native_same=30
own_semantic_different=30 direction=Output`) with zero credit, and all
72 historical pins are unchanged. No binding, receipt, proof, execution
or encoding credit; no maintenance replay (nothing retained); no
acceptance; the complete schedule stays unmeasured.

**Q22 ruling — the historical probe calls are provenance-only, not part
of the live observation.**
- Evidence verified independently: `c2_probe` exists only in the copied
  witness trees (`.temp/g15-c2/prepare_native_campaign.py:7,8` appends
  `pub mod c2_probe;` to a copied `rum-runtime/src/lib.rs` and copies
  `mutant_support.rs`; its `begin`/`end` only set and clear an `ACTIVE`
  flag used by the mutation-attribution arms), and the live runtime has
  no such module (`live_runtime_references=0`, module absent). The live
  observation is `Session::logical_profile_test_rows()`
  (`crates/rum-runtime/src/runtime/profiling/logical/mod.rs:637,686,728`)
  with the lease and reset; the historical driver reads those rows
  before its `end()` call (`jupyter_driver.rs:47,77`).
- Therefore the live fixture must not import or reproduce `c2_probe`; it
  supplies the live lease/reset around the adapter route, reads the
  terminal native sample on the executing thread before
  snapshot/projections/fresh, and applies the constant-field Native,
  missing and duplicate controls. No new test-only probe import, no
  production edit, no substitution; the declaration records the
  omission and the historical pin `512785d2…`. Any attempt to add a
  probe module or to substitute one is a STOP.
- Q21's parenthetical ("and the probe begin/end the sealed route uses")
  is corrected by this ruling; the run's STOP under it is accepted.
- The facade carries `#![allow(dead_code, unused_imports)]` at its root,
  as in the shape I compile-verified, so the test target keeps the
  baseline's zero warnings (the temporary form's 63 dead-code warnings
  are not acceptable in the retained implementation).
- Then both 30-key matrices (`P2611` Eval, `P2612` Resume) at multiplicity
  one in both arms at the sealed sampling point, each record's own
  omission in its sealed direction — all 60 sealed records are
  `valid=true`, `native_same=true`, `semantic_same=false`, so `Output`; a
  direction the sealed record does not show is a STOP — with the observed
  boundary row bag under a constant-field Native control plus missing and
  duplicate controls (any sealed-invalid record is recorded as
  sealed-invalid at zero credit; none here). Then the last 60 keys
  complete the 9,060-key active campaign (151 rows), and the complete
  schedule and acceptance are measured under the 1,500 s bound per
  expanded-oracle build+run, cumulative cost disclosed separately (this
  run: compile 0.878356 / 0.874020 s; leaves unmeasured for the
  schedule).
- Keep the 36 current-blocked proof records, the 598 absent / 35,135,
  the 24 named exclusions / 1,445 and the requirement bag 175 / 10,505
  named; no settlement, no owner promotion, no encoding key; replay
  maintenance after the fixture change; rebuild the lane
  content-addressed from the final tree; any further private access is a
  STOP and a question.

**Host gate.** The run reproduces: its audit prints `Q21 NESTED INCLUDE
COMPILE PASS` for both arms, `Q21 PROBE DIAGNOSTIC EXACT PASS` with the
two `E0433` missing items and no unrelated error, the eight `SOURCE PIN
PASS` lines (seven adapter sources plus the host source `ef78a1de…`),
both `C52 SEALED PACKET AUTH PASS` lines above, `C52 HISTORICAL PINS
PASS pins=72 changed=0`, the unchanged binding/program/proof/ownership
and loss verdicts, `MAINTENANCE REUSE PASS`, `CURRENT RUN CREDIT PASS
bindings=0 receipts=0 proofs=0 executions=0 encoding=0 SAME=0
settlements=0 promotions=0` and `ACCEPTANCE UNRUN`. I re-ran that audit
myself with rc 0; the run's 5,740-file entry scope is final-identical,
my full 5,739-file snapshot still equals the seventeenth run's sealed
lane workspace except the three committed docs (0 added or missing), and
the baseline `cargo check -p rum-stdlib --test suspension_oracle` emits
zero warnings on the unchanged tree. The report's 103 citations verify.
C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner is
promoted, no guest executes, no encoding key runs, and `CHANGELOG.md`
and S10 stay blocked. The partial tree is the state to continue from.

**Continuation 19.** Entry state: 149 rows / 9,000 keys; 2 active rows /
60 keys (`P2611`, `P2612`); 24 director-excluded rows / 1,445; proofs
749/43,390; encoding 0 of 18,840. Retain the Q21 nested include and the
one driver, omit the historical probe calls per Q22, execute both 30-key
matrices with the per-record `Output` direction and the
constant/missing/duplicate controls, complete the 9,060-key campaign
(151 rows), and measure the complete schedule and acceptance under the
1,500 s bound. The partial tree is the state to continue from; do not
restart and do not hand-edit a generated cell.

### Continuation 20 — twentieth run: two worker importer defects fire the exact two-attempt stop-loss (zero credit, tree unchanged), and Q23 authorizes one fresh bounded continuation with the sealed witness copies

**2026-11-16 — brief 739's twentieth run executed continuation 19 and is
accepted at its STOP. It made two self-authored C52 authentication
attempts and no binding or execution: attempt 1 asserted the initial
`c52.handoff` pause inside the probe log, though that handoff happens
before the probe's counted epoch; attempt 2 got both sealed packet
lines and all seven source pins through, then asserted the generic
`.temp/g15-c2/mutant_support.rs` (sha `bbe4577c…`) where the sealed
witness copy (`512785d2…`) was required. The exact two-attempt
stop-loss fired, the tree is unchanged, both attempts are preserved,
and zero binding/receipt/proof/execution/encoding credit is claimed or
owed. Both defects are the worker's importer; the sealed inputs
themselves are intact and pinned. Q23 authorizes one fresh bounded
continuation from the unchanged tree with the corrected provenance
inputs — the two `source-…-{real,mutant}/crates/rum-runtime/src/c2_probe.rs`
copies, both `512785d2…`, both listed in the sealed `artifact-hashes.json`
— and the handoff read as outside the counted epoch (`jupyter_driver.rs:38-47`).**
Run `cdx`, unit `brief-739-cdx-20261116-035346-1456047`, rc 0, peak
225,046,528 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_739.log`, 1,638,755 B, sha256 `58b040c9…`; report
`.temp/brief-739/report.json`, 43,400 B, sha256 `5a1d715d…`; 0 recorded
changes (0 added, 0 modified, 0 deleted), 0 disclosures; 6 PASS / 1
FAIL / 1 NA (the stop-loss), 3 deviations, 1 question; 170 citations
verified; no manifest or lock hash changed and no git or network command
ran.

**What the run did.** Two authentication/importer attempts, both exit
1 and preserved (`authenticate_c52-attempt1.py` `baf36ba8…`,
`authenticate_c52.py` `0a941c5f…`): attempt 1 stopped on its pause-log
assertion (`pp[0]==['c52.handoff','P2611']`), attempt 2 printed both
`C52 SEALED PACKET AUTH PASS` lines (`frozen_keys=30 real_valid=30
disabled_exact=30 own_valid=30 own_native_same=30
own_semantic_different=30 direction=Output`) and all seven source pins,
then stopped on the probe-path assertion. The final audit reads `Q22
PATH DIAGNOSTIC PASS` for both sealed copies at `512785d2…`, `IMPORTER
PATH ERROR CONFIRMED … production_defect=false`, `CAMPAIGN STOP
consecutive_zero_evidence_attempts=2 third_attempt=false`, zero current
credit and `ACCEPTANCE UNRUN`; all 5,739 workspace files are unchanged
(my own snapshot equals the seventeenth run's lane workspace except the
three committed docs, 0 added or missing). No maintenance replay was
needed; no cargo, build, guest or execution ran; four Python leaf
commands total 1.037397 s; the complete schedule stays unmeasured.

**Q23 ruling — one fresh bounded continuation with the corrected
provenance inputs.**
- Authorized: continue from the unchanged tree, using as the
  authentication inputs exactly
  `.temp/g15-c2/c52/source-jupyter-segment-cancelled-result-normalization-real/crates/rum-runtime/src/c2_probe.rs`
  and its `-mutant` sibling, each `512785d2…` and each listed in the
  sealed `artifact-hashes.json` (5,879 entries). The initial
  `c52.handoff` pause precedes `Session::logical_profile_test_reset` and
  `c2_probe::begin` (`jupyter_driver.rs:38-47`), so it is outside the
  counted epoch and must not be expected in the probe log. The generic
  `.temp/g15-c2/mutant_support.rs` (`bbe4577c…`) is not a sealed input
  and must not be substituted for one.
- Those sealed copies are read-only provenance for the authentication;
  Q22 still forbids importing or reproducing `c2_probe` in the live
  fixture. No production change, no scope expansion, no new exclusion,
  and no re-pin of history.
- Then the same continuation carries the Q21/Q22 execution stage: the
  nested include and the one driver, both 30-key matrices (`P2611`
  Eval, `P2612` Resume) at multiplicity one in both arms at the sealed
  sampling point with each record's own `Output` direction and the
  constant-field Native, missing and duplicate controls, and the last
  60 keys complete the 9,060-key active campaign (151 rows). The
  complete schedule and acceptance are measured under the 1,500 s bound
  per expanded-oracle build+run, with cumulative cost disclosed
  separately (this run: four Python leaves 1.037397 s; no cargo or
  build cost; schedule unmeasured).
- Keep the 36 current-blocked proof records, the 598 absent / 35,135,
  the 24 named exclusions / 1,445 and the requirement bag 175 / 10,505
  named; no settlement, no owner promotion, no encoding key; replay
  maintenance after the fixture change; rebuild the lane
  content-addressed from the final tree; any further private access is a
  STOP and a question. If two more consecutive attempts again add no
  evidence, the stop-loss stops again rather than a third attempt.

**Host gate.** The run's audit re-ran green under my invocation with its
verdict lines intact: the two `Q22 PATH DIAGNOSTIC PASS` lines, `IMPORTER
PATH ERROR CONFIRMED`, the two `ATTEMPT PRESERVED` lines, the unchanged
binding/program/proof/ownership/loss/deferred verdicts, the unchanged
maintenance pins and `MAINTENANCE REUSE PASS`, `CAMPAIGN STOP`, `CURRENT
RUN CREDIT PASS …` and `ACCEPTANCE UNRUN`. I verified both sealed copies
hash `512785d2…`, that `artifact-hashes.json` lists both, and that
`jupyter_driver.rs:38-47` places the handoff before the probe's epoch. My
full 5,739-file snapshot is unchanged against the seventeenth run's lane
workspace except the three committed docs. The report's 170 citations
verify. C2 stays Open at 118 Residual + 0 Lost; no row settles, no owner
is promoted, no guest executes, no encoding key runs, and `CHANGELOG.md`
and S10 stay blocked. The partial tree is the state to continue from.

**Continuation 20.** Entry state: 149 rows / 9,000 keys; 2 active rows /
60 keys (`P2611`, `P2612`); 24 director-excluded rows / 1,445; proofs
749/43,390; encoding 0 of 18,840. Complete the C52 authentication with
the two sealed witness copies, then retain the Q21 nested include and
the one driver, omit the historical probe calls per Q22, execute both
30-key matrices with the per-record `Output` direction and the
constant/missing/duplicate controls, complete the 9,060-key campaign
(151 rows), and measure the complete schedule and acceptance under the
1,500 s bound. The partial tree is the state to continue from; do not
restart and do not hand-edit a generated cell.
