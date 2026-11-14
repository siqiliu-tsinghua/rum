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
