# brief-740 — G15-CM4: own-omission receipts and atomic owner promotions

Scope: **implementation, test layer only.** Consume the accepted CM3 layer and
add the mutation and promotion stage. The design is brief 736 (accepted; the
five staged briefs), its draft `draft-CM4-mutants-promotions.md`, and the eight
continuation-69 rulings. The draft is raw material for my rewrite, not a
specification: where this brief states a goal and a bar, the goal and the bar
govern; the worker chooses the carriers, selectors, exact schema and file split
within the scope below.

## Goal

When you are done I must be able to run the test binary in both feature modes
and see, from committed inputs and copied-build receipts only:

- **Every active Equal row** (the accepted CM3 layer's 151 rows / 9,060 keys)
  carrying a **caught, real, current-source omission of its own actual tail** —
  not an in-test reconstruction, not another row's catch, not a compile-only
  rejection — beside its complete real SAME matrix. A row whose own mutation
  cannot be admitted is `PortBlocked` with its concrete missing evidence; it
  does not silently stay Equal.
- The **four independent structural controls** (omitted `Some` completion, the
  G15-A `Return` finish with projection resume, one awaited G15-B recovery, the
  C1 post-rewrite evaluation finish), each with its own source hit and its own
  structural red with final-value comparison disabled, none borrowing another's
  mutation or a compile failure.
- The **held pair rows** `P0325`/`P0326` still carrying their stronger complete
  per-cell bar (five Error cells each armed without hit; 45/50 catches is red),
  not substituted by a generic one-catch.
- **Atomic owner promotions** over all assigned rows, tuple duties, source
  duties, losses, cross-walks and certificates: a pure transaction that
  validates the complete post-state first, then removes exactly the discharged
  Open obligations and sets `Done` with one receipt, rejecting a partial,
  stale-now-SAME, new-DIFF or wrong-delta proposal without touching the
  before-state. **The 51 owners and the 118 C2 residuals stay Open**: this
  brief authorizes synthetic transaction tests only, no real promotion.
- A **measured, bounded, unsampled** mutation schedule: the complete required
  case count is frozen from the accepted CM3 layer before the first mutant
  build; every case runs; one canonical real/disabled/armed build per unchanged
  schema and feature mode, reused by authenticated binary identity, with one
  same-epoch forced byte-identity spot check per admitted schema round. A
  different schema or a schedule beyond the measured allocation STOPs for my
  decision — never a sampled context, never a dropped row.

No row settles, no SAME result is minted, no owner is promoted, no encoding key
is activated, deleted, executed or credited, and no production byte changes.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`.
- Governing design: `.temp/brief-621/MANIFEST-DESIGN.md` (mutations `:205`–
  `:249`, owners and promotions `:160`–`:200`); brief 736 and its accepted
  PLAN (`:320`–`:470` enforcement, staging, cost) and the eight rulings.
- The accepted CM3 layer (commit `23d0e0d9`): `obligations/mod.rs`,
  `coverage/{bindings,receipts,schedule,execution_tests}.rs`,
  `ledger/{mod,proofs,known_losses,proof_tests}.rs`,
  `fixtures/obligations/{mod,guest,native,boundaries}.rs`,
  `inventory/obligations/{bindings,programs,proofs,known-losses,provenance}.tsv`.
  CM3's real matrices and own-omission directions stay active and unchanged.
- Existing mutation material: `mutations/{mod,driver,capture_output}.rs` and the
  four accepted control fixtures (`fixtures/status_function/`,
  `fixtures/definition_lhs_cycle.rs`, `fixtures/replaceall_application.rs`);
  these are existing checks and stay.
- Provenance-only inputs (never standing assertions): the C68 campaign controls
  `.temp/g15-c2/c68/canonical_gate.py:31`, `final_state.py:31`, `evidence.py:54`;
  the retained raw mutant packets; the sealed C52 `mutants.jsonl` /
  `full-proofs.json` shapes. The brief-621 continuation3 mutation
  schemas/results are absent and not restorable (Q1): reconstruct only under a
  director-reviewed provenance record, calibrated against every retained
  control, with every unrecoverable datum `PortBlocked` — never an invented
  hash, row, key or receipt.
- Build reuse: `scripts/probe-build.py` unchanged (stable `.temp/probe/active`,
  content-addressed cache, one build per arm, one `--force` sha256 spot check per
  round) through an explicitly rewritten path adapter on the accepted
  `.temp/g15-c2/probe_build.py:1` precedent; release, one codegen unit; no ad-hoc
  `CARGO_TARGET_DIR`, no flag change to save time.

## Rulings carried (continuation 69, Q1–Q8)

- Q1: absent historical inputs are reconstructed only under calibration, every
  unrecoverable datum `PortBlocked`; no invented evidence.
- Q2: manifest integrity and `owner_closure(C2)` are separate predicates. This
  stage may be green only with explicitly admitted Open deferrals (identities,
  requirements, named bar and owner checked, all active evidence still running).
  `owner_closure(C2)` stays false at 118 Residual + 0 Lost; `CHANGELOG.md` and
  S10 stay blocked; no partial stage reports full CM green.
- Q3: encoding-blocked residuals keep C2 row ownership and G13 tuple ownership;
  no implicit transfer; a cross-walk needs my ruling.
- Q4: the five source-provenance duties stay Open and separate; each contract
  needs its own design before promotion.
- Q5: the pinned accepted ledger is authoritative; missing required evidence is
  `PortBlocked`; the 857 false secondary flags and ten stale secondary cells are
  never new losses or reopened verdicts.
- Q6: the dispatched 16G scope runs cargo directly, one command at a time at
  `-j8` / `RUST_TEST_THREADS=8` / offline; test-side scratch paths are reported.
- Q7: ≤1,500 s per expanded-oracle build+run stands, target ≤900 s steady state.
  The complete program/mutant schedule is measured first; exceeding the bound
  or needing a new carrier schema STOPs for a schedule decision — never sampling.
- Q8: the held pair counts are 70 keys (50 nonencoding + 20 encoding) per row,
  140 / 40 combined; the frozen inventory is unchanged.

## Scope

Files you MAY create or edit, all under
`crates/rum-stdlib/tests/suspension_oracle/`:

- `mutations/` — the mutation schema, receipts, controls and source-omission
  tests (the draft's indicative split: `schema.rs`, `receipts.rs`, `controls.rs`,
  `source_omission_tests.rs`, wired from `mutations/mod.rs`; the existing files
  and tests stay).
- `ledger/` — the promotion transaction and its tests (`promotions.rs`,
  `promotion_tests.rs`; the existing ledger files stay).
- `fixtures/obligations/` — a semantic-controls fixture module if a copied
  mutation needs one; `mod.rs` wiring.
- New committed data `inventory/obligations/{mutants,mutant-coverage,
  promotions}.tsv`, sorted, tab-separated, one header line, no timestamps, no
  free-form prose columns.
- `obligations/mod.rs`, `fixtures/mod.rs`, `support/{mod,observe}.rs` only for
  additive wiring; every existing check and call site stays.

Maintenance-only paths, exact (only when a replay says they moved):
`crates/rum-stdlib/src/tests/retention_closure/inventory/{local-contracts,
promoted}.tsv`; the six `crates/rum-stdlib/src/tests/ownership_graph/inventory/
*.tsv`; LP01 `logical-profile-{required,bindings}.tsv` and
`logical-profile-losses.json`; `inventory/typed-slot-stages.tsv`; the runtime
frame closed lists in `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs`
and `derived_inventory.rs` only if a new test frame is introduced (none
planned). Quote every replay verdict; never hand-author a derived cell.

Copied source trees for mutation builds live only under `.temp/brief-740/`;
they are disposable and never installed. Production source, `Cargo.toml`/
`Cargo.lock`, scripts, docs, historical scenes and the dispatch/target caches
are read-only. Forbidden: git, network, `unsafe`, new dependencies, live
production mutation, sealed-scene writes, inventory predicate edits.

## Deliverable

Test-layer types and functions; the worker fills the bodies. The design's
proposed signatures (rewrite freely inside the scope):

```text
MutationSpec { row, source_key, tail_digest, carrier, phase, selector, observer }
MutationReceipt { row, tuple, build, armed, hit, baseline_pause,
                  disabled_parity, actual_difference, raw_artifact }
verify_own_omission(row, full_real_receipts, selected_mutation) -> CaughtRow
Promotion { id, owner, before_hash, exact_rows, tuple_duties, source_duties,
            losses_removed, crosswalks, certificates, after_hash }
validate_promotion(before, proposal, evidence) -> AtomicDelta | PromotionError
```

Rules the implementation must enforce:

- A receipt binds its current source/build identity: row, tuple, source key,
  tail digest, carrier, phase, selector, observer armed/hit, baseline producer
  pause, disabled-copy record-exact parity, actual difference, raw artifact.
- A mutation counts only when all of these hold: its own row's selector armed;
  the actual phase hit; the baseline producer pause present; the
  switches-disabled copy record-exact to real; the red is the row's own
  predicate (not a compile error, an absent file, another row's catch or
  another earlier guard); the real full matrix still passes. A compile-only
  typed rejection is a distinct receipt class, never a caught mutant.
- Carriers: frame mutation is compatible outcome pass-through without its own
  finish; frameless ordinary success skips exactly its post-producer tail;
  error, adapter and cleanup mutations are row-specific. No fabricated value or
  failure, no arbitrary `None`/default aggregate/panic, no unrelated outer
  cleanup omission.
- The held pair rows keep the stronger per-cell bar; a generic one-catch does
  not substitute for five armed-without-hit Error cells.
- Promotions validate the complete post-state first, then remove exactly the
  discharged Open obligations and set `Done` with one receipt. Regression
  requirements, accepted alternative proof dispositions and dependency links
  are retained; all unrelated owner/loss data are identical. A synthetic
  transaction fixture is the only green path; no real C2/S-step/source-owner
  promotion is authorized by it.
- The mutation schedule is complete and unsampled: every active Equal row and
  every stronger per-cell omission runs. Two families with no new evidence stop
  the campaign.

## rum / Rust pitfalls (read twice)

- Equal requires the complete real SAME tuples **and** the row's own caught
  tail-skip mutant. A nearby frame, an observer statement or a scalar equality
  cannot substitute.
- A source hit without a baseline producer pause is not a catch; a compile
  rejection is not a semantic catch; an identity survivor is not Equal.
- The `Some` native Option-slot observer precedes lenient decoding; an ordinary
  value of 5 alone proves nothing. G15-A includes the shared finish after the
  projection pause without moving projection errors earlier. One awaited G15-B
  variant at a time. The C1 trace stays red at equal scalar.
- A check nobody has watched fail is not known to work: every gate ships its
  falsifier, red on both arms for its own named predicate.
- Historical scenes are read-only inputs. Never weaken an assertion, delete an
  attempt, relabel an invalid recipe or re-pin an inventory to make a plan fit.
- The copied mutant tree must not re-pin the source mutation into "normal": the
  real build, the disabled build and the armed build are independently
  authenticated; disabled parity is exact records, not a tolerance.

## Verification (MANDATORY)

**Shape stop.** Compile the real, disabled and independently armed copy schema
for the complete declared mutation family in both feature modes; certify
selectors, phase and source identity, and exact disabled parity. Run
`cm_own_mutant_receipts_exact`, `cm_mutant_negative_controls`,
`cm_promotion_atomic` and `cm_promotion_negative_controls` on both feature
modes, plus the four structural controls. A source-scope deviation, unsupported
carrier or missing calibration blocks that family and never becomes Equal. Do
not start the full campaign before shape and controls pass.

**Falsifiers**, each red for its own named predicate on both arms: wrong-row
receipt; unarmed switch; no-hit phase; surviving identity; absent baseline
pause; missing real tuple; disabled-copy DIFF; compile-only "catch"; partial
held-pair promotion; Done-before-removal; one lingering duty; deleted loss;
stale now-SAME loss; changed unrelated owner; encoding key deletion/activation.
Strict predicates are red with final scalar comparison disabled where relevant;
the exact originals and the complete synthetic transaction are green. Preserve
the seven C68 ledger negatives and both pair negatives.

**Campaign stop.** Freeze the complete required case count from the accepted
CM3 layer before the first mutant build, and print it. Execute every active
Equal's admissible own source mutant and every stronger required per-cell
omission; no sampling. One canonical real/disabled/armed build per unchanged
schema and feature mode, reused by authenticated binary identity; one same-epoch
forced byte-identity spot check per admitted schema round (not a cross-epoch
bar). The draft's proposed allocation is eight blocks (six real/disabled/armed
plus two spot checks), each ≤1,500 s, total ≤12,000 s for the copied mutation
campaign; it is an allocation, not a measured duration. The expanded oracle's
build+run stays within its 1,500 s bound and the 900 s steady target; the
mutation-build cost is reported separately and never hidden in oracle time. If
the carriers need another schema or the complete schedule exceeds the
allocation, STOP for my decision.

**Acceptance suite** (once, after the last edit): `cargo fmt --check`; the
workspace suite in both feature modes with the seven documented switch-gated
skips
(`--skip lp03_adopted_phases_are_exact_in_completed_and_resumed_runs
--skip lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs
--skip lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs
--skip lp658_host_eval_reset --skip lp658_host_resume_reset
--skip lp655_root_context_falsifier --skip lp656_public_root_context_falsifier`)
and `--no-fail-fast`; the same binary set under
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` for `lp03_phase_rows`,
`lp04_definition_rows` and `lp05_assignment_rows`; the full
`suspension_oracle` and `typed_slot_census` in both modes. The nine documented
native ZeroMQ loopback EPERM failures are the sandbox artifact, reproduced at
one thread; no new skip or waiver. Reproduce any semantic red at one test
thread before calling it a defect.

## Maintenance before the first acceptance suite

Every live test-source edit expires retention fingerprints. Replay the ignored
`regenerate_local_contract_fingerprints`, then `local_production_acceptance` in
both modes, and mechanically copy only the replayed cells into
`crates/rum-stdlib/src/tests/retention_closure/inventory/{local-contracts,
promoted}.tsv`. Replay ownership discovery and reconcile the six
`crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv`; new
`UnknownConstruct` rows for touched code stay unknown and are disclosed;
changed, deleted or re-labelled existing identity/status/disposition/exclusion
rows STOP. If covered producer sites move, replay `logical_counter_port` and
`typed_slot_census` and copy only their mechanically derived cells. A new frame
enters the runtime closed lists and count exactly. Rejection-census count
changes STOP for my ruling. Quote every replay verdict before acceptance; a
stale-derived-cell red is an ordering artifact, never grounds for weakening an
inventory.

## Checkpointing — protect the scene

Checkpoint after the schema, after the first caught family, after each family,
after the promotion transaction, after maintenance and after each suite. Record
the remaining rows and cases, the exact next action, every command run and every
open question. Preserve every failed mutant, log and filter interruption; no
attempt deletion or silent baseline replacement.

## Hard rules

- No git, network, `unsafe`, external engine, new dependency or write outside
  this repo and the dispatched brief's `.temp` directory.
- No live production source mutation. Copied trees only; production, Cargo,
  scripts, docs, inventory predicates, history and caches are read-only.
- No counterfeit receipt, weakened observer, forced false carrier, unapproved
  owner promotion, historical-scene write or inventory re-pin. No encoding run,
  activation or credit; no settlement, no SAME, no full CM green.
- A missing authentic input or an inadmissible mutant blocks its family and is
  reported; it never becomes Equal.

## Report format

One `rum-report/1` via `scripts/report.py` (report path
`.temp/brief-740/report.json`), with sections: mutation schema, own-omission
receipts, structural controls, promotion transaction, owner data, campaign
progress, falsifiers, maintenance, counts and cost, questions. Every claim
carries `path:line` or a hash. List every open question with two candidate
resolutions and their evidence. `scripts/report.py check 740` must pass before
you stop. Next is CM5 after my gate.

**Dispatch:** `scripts/dispatch-brief.sh 740` (background, `BRIEF_WORKER`
unset = `cdx`).

## Continuation 1 ruling (director) — the pre-edit STOP is correct; Q1 admits one shared P0543 lane group with a twelve-block allocation, and the campaign starts

**2026-11-16 — brief 740's first run executed continuation 1 and stopped
pre-edit. It authenticated the accepted CM3 input, froze the complete
mutation requirement count, re-ran the three existing build-configuration
controls, and stopped because P0543's own source omission requires fresh
real/disabled/armed builds in its accepted unchecked configuration, which
the brief's read-only Cargo scope and eight-block allocation do not settle.
Accepted as delivered: the tree is unchanged and no mutation,
structural-control, promotion, settlement, SAME or encoding credit exists.
Q1 is ruled: option 1, one shared third build identity — not two lane
feature modes — with the copied trees carrying exactly the accepted
nine-entry manifest diff and the campaign allocation extended to twelve
blocks / ≤18,000 s.**
Run `cdx`, unit `brief-740-cdx-20261116-053910-1515037`, rc 0, peak
623,919,104 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_740.log`, 2,046,787 B, sha256 `853a4a89…`; report
`.temp/brief-740/report.json`, 31,493 B, sha256 `ee1a1121…`; 0 recorded
changes (0 added, 0 modified, 0 deleted), 0 disclosures, 1 question (Q1,
ruled here); 10 PASS / 1 FAIL / 2 NA; 58 citations verified; no production,
manifest or lock hash changed.

**What the run froze and verified.**
- `CM4 REQUIREMENTS FROZEN active_equal_rows=151 active_real_keys=9060
  families=68 required_rows=175 required_keys=10505
  existing_PortBlocked_rows=24 existing_PortBlocked_keys=1445`
  (`preflight.log:1`). I recomputed the bags from the committed
  `inventory/obligations/bindings.tsv` myself: 21,010 records, `required`
  10,505 = `bound` 9,060 + `blocked` 1,445, over 175 / 151 / 24 rows, and
  `programs.tsv` carries 9,060 rows over 68 families.
- `CM4 MUTATION REQUIREMENT COUNT own_rows=151 held_pair_cells=100
  structural_controls=4 logical_cases_per_feature=255
  logical_cases_two_features=510 selected_own_tuples=0 mutant_builds=0
  schedule_measured=false` (`preflight.log:2`). This is a requirements
  freeze; the exact selected own tuples and the executable schedule are not
  yet admitted or measured.
- `CM4 HELD PAIR INPUT PASS rows=2 nonencoding=100 encoding_retained=40
  historical_caught=90 historical_armed_no_hit_Error=10 fresh_runs=0
  generic_catch_substitution=false` (`preflight.log:3`). I re-verified the
  frozen pair inventory: `.temp/g15-c2/c68/required-tuples.tsv` sha256
  `3c2dcdff…` carries 140 pair tuples (100 non-`guest_heads`), and
  `pair-proof-packet.json` records per row `cells=50 armed=50 caught=45
  armed_no_hit=5`, `eligible_Equal=false`, every uncaught key `Error`.
- `CM4 OWNER INPUT PASS states=51 Open=51 C2_residual=118 C2_lost=0
  promotions=0` (`preflight.log:4`).
- `CM4 ACCEPTED LANE AUTH PASS binary_sha256=bbf1f199…
  manifest_diffs=9 live_crate_differences=0 historical_only=true
  current_own_mutant=false` (`preflight.log:5`). I re-verified the binary
  `bbf1f199…`, the tree `5a5f6f4e…`, the diff file sha256 `66a8e8bd…` with
  nine entries, that every live manifest still equals its recorded `before`
  bytes, and that the lane snapshot differs from the live tree only in the
  three committed docs plus this brief (zero `crates/` differences).
- The three configuration controls are accepted as diagnostics at zero
  credit: six parent/child passes in default (0.173112 s), no-default
  (0.171553 s) and the accepted lane (0.050218 s); the checked-only
  `NonAmplifying` discriminator reads `true` in both mandated arms, `false`
  in the lane's unchecked parent, `true` in the lane's forced control, and
  its falsifier fires (`configuration-*.log`). They are not CM4 mutation
  receipts and not acceptance.
- `CM4 SOURCE SNAPSHOT PASS files=5774 added=0 changed=0 deleted=0`
  (`final-audit.log:1`): no live source, manifest, inventory, history or
  cache edit; `CM4 CURRENT CREDIT PASS own_mutants=0 structural_controls=0
  promotions=0 settlements=0 SAME=0 encoding_runs=0` (`final-audit.log:3`);
  `CM4 ACCEPTANCE UNRUN` and `CM4 MAINTENANCE NOT REQUIRED`
  (`final-audit.log:2`, `:4`). The run's 32 commands contain no git or
  network call, and every write is inside `.temp/brief-740/`.

**The STOP.** Correct. `builtin_memory_growth_test_enabled()`
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:365`) returns `true`
before reading the environment whenever `test-builtin-memory-growth` is in
the build graph, and the two mandated modes both carry it
(`crates/rum-stdlib/Cargo.toml:28`), so P0543's unchecked producer
(`:187`-`:188`) is reachable only in the accepted continuation-8 lane. The
brief makes the live Cargo files read-only and its campaign allocation
eight blocks; fresh real/disabled/armed lane builds are neither. The run
correctly did not edit, did not build a mutant and did not borrow the
historical real-only lane binary. Option 2 (retain P0543 as CM4
`PortBlocked`) is declined: it would leave an active Equal row without the
own-omission evidence this brief's first goal requires, for a configuration
the C2 line has already accepted twice.

**Q1 ruling.**
- **One shared third identity.** The lane is one build configuration, not
  two lane feature modes. Both mandated arms spawn the same lane worker
  exactly as CM3's accepted layer does
  (`coverage/schedule.rs:38`, `coverage/receipts.rs:74`), and the lane runs
  its own configuration control inside its binary. Do not build the lane in
  both mandated feature modes.
- **Copied manifests, exact.** The real lane copy is a content-addressed
  copy of the current workspace under `.temp/brief-740/` whose only
  differences are the nine dev-dependency manifest entries pinned by
  `.temp/brief-739/lane-manifest-diff.json` (sha256 `66a8e8bd…`):
  `test-builtin-memory-growth` removed, `test-continuation-slots` retained.
  Every non-manifest file hash-matches the live tree. The disabled and
  armed copies additionally carry exactly the admitted mutation-carrier
  edits, each named and hashed in its receipt. Live `Cargo.toml` and
  `Cargo.lock` stay byte-identical; no new dependency; the copies are
  disposable and never installed.
- **Allocation.** Twelve blocks for one admitted carrier schema: per
  configuration (default, no-default, the P0543 lane) one real, one
  disabled and one armed build, plus one same-epoch forced byte-identity
  spot check — nine builds and three spot checks, each block ≤1,500 s,
  total allocation ≤18,000 s. This remains an allocation, not a measured
  duration. The expanded oracle's own build+run stays ≤1,500 s with the
  ≤900 s steady target; the copied-build cost is reported separately. A
  second carrier schema, or a complete schedule beyond the allocation,
  still STOPs for my decision.
- **Lane build proofs, before any campaign case.** The lane copy goes
  through the unchanged `scripts/probe-build.py` with an explicitly
  rewritten path adapter on the accepted `.temp/g15-c2/probe_build.py:1`
  precedent; release, one codegen unit. The fresh real build must show
  `cargo tree -e features` carrying no `test-builtin-memory-growth`, the
  checked-only discriminator true in both mandated arms and false in the
  lane with its forced fresh-process control and firing falsifier, and it
  must pass P0543's 55-key real matrix at multiplicity one with its ten
  native fields and the three checked-route Native falsifiers unchanged.
  Its receipts carry the third identity bound to the binary sha256, tree
  hash, manifest-diff hash, `profile=release` and one codegen unit. The
  accepted CM3 binary `bbf1f199…` is a reference only and is never credited
  as a CM4 receipt; the tree and diff pins are the accepted ones.
- **P0543's catch.** The armed copy must catch P0543's own real source
  omission of its unchecked producer tail: its own selector armed, its
  actual phase hit, a baseline producer pause, the disabled copy
  record-exact to the fresh real copy, the row's own predicate red, the
  full real matrix still passing. No in-test reconstruction, no other row's
  catch, no compile-only rejection; an inadmissible carrier is
  `PortBlocked` with the concrete missing evidence.
- **Counts.** The frozen case count stays 255 per feature mode / 510
  across the two; the lane adds builds, not cases. Re-freeze and print the
  complete count together with the twelve-block plan before the first
  mutant build.

**Order.** Resolve the lane group's copied trees and builds first; then
admit the mutation schema and the promotion transaction; run the shape stop
(real/disabled/armed for the complete declared family in both mandated
modes plus the lane group, selectors, phase and source identity, exact
disabled parity) and all named falsifiers on both mandated arms; only then
the complete unsampled campaign and the acceptance suite exactly as the
brief specifies.

**Unchanged.** No live production mutation, no manifest or lock edit, no
network, git, `unsafe` or new dependency; no settlement, no SAME, no owner
promotion, no encoding activation, execution or credit; C2 stays Open at
118 Residual + 0 Lost; `CHANGELOG.md` and S10 stay blocked; the eight
continuation-69 rulings (Q1–Q8) stand.

## Continuation 2 ruling (director) — the shape STOP is correct; Q2 admits the mutation-source entry point and a fresh twelve-block round (fifteen total with the three consumed diagnostics)

**2026-11-16 — brief 740's second run executed continuation 2 and stopped at
the shape gate. It built and authenticated the fresh P0543 lane real arm,
passed the build-graph, configuration and forced byte-identity checks and the
55-key real matrix, assembled the disabled and armed copies with exactly
three named overlays each, built the disabled arm, and then hit the loader's
whole-file guest-route `SourceIdentity` guard before any program selection:
the copied adapter calls the ordinary `guest::programs()`, which by design
rejects the intentional source mutation. No mutation was executed, no
observation or source hit was recorded, and no live file changed. Accepted as
delivered: the failure is a copied-source admission ordering artifact, not a
catch, not a regression. Q2 is ruled: option 1 — one test-layer,
constructor-private mutation-source entry point in
`fixtures/obligations/guest.rs` beside the untouched strict loader — with a
fresh twelve-block round (fifteen total including the three consumed
diagnostic blocks, ≤22,500 s allocated).**
Run `cdx`, unit `brief-740-cdx-20261116-055723-1520211`, rc 0, peak
2,600,419,328 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_740.log`, 2,212,189 B, sha256 `d1deb5e4…`; report
`.temp/brief-740/report.json`, 49,209 B, sha256 `ffda534c…`; 0 recorded
changes (0 added, 0 modified, 0 deleted), 0 disclosures, 1 question (Q2,
ruled here); 12 PASS / 1 FAIL / 3 NA; 101 citations verified; the first
report is preserved byte-for-byte at
`.temp/brief-740/c1/report-before-ruling.json` (`ee1a1121…`, the
continuation-1 report); no production, manifest, lock or inventory hash
changed.

**What the run proved.**
- The allocation was printed before any build:
  `CM4 ALLOCATION configurations=3 shared_lane=true canonical_builds=9
  forced_spot_checks=3 blocks=12 per_block_seconds=1500 total_seconds=18000
  measured=false`, with the requirement count unchanged (151 own rows +
  100 held-pair cells + 4 structural controls = 255 per feature mode / 510
  total; 151 selected own tuples frozen at
  `.temp/brief-740/c1/selected-own-tuples.json`).
- The fresh lane real arm: content-addressed copy `787edec7…` (5,777 files,
  5,768 byte-identical, nine manifest diffs, diff `66a8e8bd…`,
  `production_changes=0`), release / one codegen unit, binary
  `b17fc131be5d01781441ea4c01ab277f3f549da75491f6aed522044801b374cc` in
  86.566 s; `cargo tree -e features` carries no
  `test-builtin-memory-growth`; the three configuration controls pass with
  forced fresh-process controls and firing falsifiers; the same-epoch forced
  byte-identity spot check reproduces the same cache key `3aaab11e…` and the
  identical binary in 84.477 s; P0543's 55-key real matrix passes at
  multiplicity one with ten native fields, the three checked-route Native
  falsifiers and the selected baseline producer pause (19.823567 s).
- The copied arms: exactly three named overlays each — the runtime
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` preimage `146b7600…`
  to disabled `4f9803…` / armed `c2d59f…`, the additive
  `mutations/mod.rs` `911088…` to `e04c05…`, and the added copy worker
  `mutations/cm740_copy_worker.rs` `abb89d…` — enumerated in
  `lane-mutation-copies.json` / `lane-carrier-provenance.json` against the
  retained C28 schema (`742704b6…`). The disabled arm built as
  `3734919849e1ba6fa9ee96561c4e27d2264ea44c2e0abd867c163d2c00c428c2`,
  tree `a48369e0…`.
- The failure: the copy worker's `guest::programs()` rejects at
  `fixtures/obligations/guest.rs:1429-1432` (`SourceIdentity`) before any
  program selection, at eight and one test thread, with zero observation
  records and zero source hits; 110 whole-file guest-route pins differ
  (55 P0543 + 55 P0544 in the same file), canonical mismatches 0, dependency
  mismatches 0, inventory edits 0. The first rejecting route belongs to
  P0544, so it is correctly not called P0543's catch.
- `CM4 LIVE SNAPSHOT PASS files=5777 added=0 changed=0 deleted=0`
  (manifests, lock and inventories unchanged);
  `CM4 CURRENT CREDIT PASS own_mutants=0 structural_controls=0
  promotions=0 settlements=0 SAME=0 encoding_runs=0`; `P0543` is
  `PortBlocked` with missing disabled record-exact parity, armed build, own
  phase hit and own predicate red; the other 150 rows are `NotAttempted`,
  never implicitly Equal; the accepted CM3 Equal disposition is untouched;
  maintenance not required; acceptance unrun. Three build blocks consumed
  256.517221 s (86.73 + 84.60 + 85.19), each under 1,500 s; the forced-check
  cost is separate from the 19.82 s real matrix.

**Q2 ruling — option 1, the mutation-source entry point.** The live-file
scope grows by exactly one file: `fixtures/obligations/guest.rs`, test layer
only. Option 2 is declined: it would duplicate the admission logic behind a
new serialization boundary while still needing the same overlay
authentication, and it cannot re-check the copied build's own route source
against the authenticated postimage. The following invariants are the bar.
- **The strict path stays strict.** `programs()`, `load_programs` and every
  existing CM3 call site keep their current checks and force; the real
  matrix and the whole CM3 suite keep running through them. The new path is
  additive and reachable only through a constructor-private authenticated
  view whose fields are private and which only the authenticator can build;
  an unauthenticated or wrong-arm view is a compile-time impossibility, not
  a runtime convention.
- **Authentication, before any original source byte is supplied.** The
  authenticator proves: the baseline source equals the accepted CM3 seals
  (committed inventory, never the copied tree); every overlay preimage
  equals the baseline file and every postimage equals the copied arm's file;
  the overlay list is exactly this family's named files and every
  non-overlay file equals the live tree apart from the nine authorized lane
  manifest entries; the copied tree hash and the executed binary hash
  (`current_exe`) equal the arm's independently built receipt; the program
  table seal, the canonical-source seals and every inventory file still
  match the baseline; and, per route, the file the loader validates equals
  the authenticated postimage — the guard's force is preserved, not
  bypassed.
- **No reconstruction.** The mutation-aware loader reuses the existing
  parsing, phase attribution, native observers and validators; it invents no
  program, setup, answer, phase, route or canonical input; the supplied
  source identity is the baseline/preimage identity and the receipt binds
  preimage and postimage separately. The independent tail check at
  `coverage/bindings.rs:338` stays: the mutated tail is never relabelled as
  the original.
- **Falsifiers, each red on both arms:** unlisted changed file; wrong
  preimage; wrong postimage; wrong binary; missing overlay; mutated
  inventory; a forged or wrong-arm view (type-private); a disabled/armed
  swap; and the preserved negative that the ordinary `programs()` in a
  copied build still rejects `SourceIdentity`. None of these is a semantic
  catch.
- **The catch bar is unchanged.** The disabled arm's records must be
  record-exact to the fresh real arm; the armed arm must catch P0543's own
  real omission of its unchecked producer's pause-accounting boundary with
  its own selector armed, its actual phase hit, the baseline producer pause,
  its own `Native` red, equal public channels and the full real matrix still
  passing; the typed suspension and `Some` completion are preserved; no
  fabricated value, failure, default aggregate or unrelated cleanup
  omission. If any of this cannot be admitted, the row stays `PortBlocked`
  with the concrete missing evidence.
- **Nothing else moves.** No inventory re-pin, no production source,
  manifest, lock, dependency or script change; the copies stay under
  `.temp/brief-740/`; no settlement, no SAME, no owner promotion, no
  encoding activation, execution or credit.

**Allocation.** A fresh twelve-block round (nine real/disabled/armed builds
across the default, no-default and lane configurations, plus three
same-epoch forced byte-identity spot checks), each block ≤1,500 s. With the
three consumed diagnostic blocks (256.517221 s) the CM4 copied campaign is
fifteen blocks / ≤22,500 s allocated — an allocation, not a duration. No
reuse is assumed: the `guest.rs` addition changes the real lane binary, so
the fresh round rebuilds it; any reuse must be by authenticated binary
identity and may not consume an unrun block. The frozen case count stays
255 / 510; a second carrier schema or a schedule beyond the allocation still
STOPs.

**Order.** Land the entry point and its falsifiers; rebuild the fresh round
and finish the P0543 lane admission first; then the rest of the mutation
schema (the other 150 own rows and the four structural controls) and the
promotion transaction; run the shape and falsifier gates in both mandated
arms; only then the complete unsampled campaign and the acceptance suite.
The `guest.rs` edit expires retention fingerprints, so replay retention and
ownership maintenance before acceptance and quote every replay verdict.

**Disclosed limitation.** The canonical builder overwrote the initial
successful real-build compiler log with the forced spot-check log at the same
cache-key path; the initial command log, the build receipt and the binary
remain intact and the forced and disabled compiler logs are archived under
`.temp/brief-740/c1/compiler-logs/`. Before any forced spot check in the
next round, copy the cached compiler log aside; never let a forced check
overwrite the only compiler log.

**Unchanged.** C2 stays Open at 118 Residual + 0 Lost; no row settles, no
owner is promoted, no encoding key runs; `CHANGELOG.md` and S10 stay blocked;
the eight continuation-69 rulings (Q1–Q8) stand.

## Continuation 3 ruling (director) — the C10 source-only STOP is correct; Q3 admits one additive, constructor-private mutation-observer path in `fixtures/obligations/guest.rs` with a completely fresh twelve-block round (nineteen blocks consumed at completion, ≤28,500 s allocated)

**2026-11-16 — brief 740's third run implemented continuation 2's admitted
source entry and both-mode falsifiers, admitted P0543 as the first complete
own-omission packet on that schema (55/55 real keys at multiplicity one, 165
record-exact disabled records, one source hit and the baseline producer
pause, the own `Native` red with equal public output), replayed maintenance,
and stopped at C10's source-only admission gap before any fresh C10 build or
case. It claimed no live credit for the stopped family. Q3 is ruled:
option 1 — one additive, constructor-private mutation-observer path in
`fixtures/obligations/guest.rs` that captures the row's actual observation
before the ordinary quiescence assertion while every ordinary runner path
and the whole strict loader keep their full force — with a completely fresh
twelve-block round (nineteen blocks consumed at completion, ≤28,500 s
allocated; the eight unspent continuation-2 blocks are voided).**

Run `cdx`, unit `brief-740-cdx-20261116-063607-1532081`, rc 0, peak
3,388,854,272 B (under the 16 GiB unit), oom 0 / oom_kill 0; log
`.temp/logs/cdx_brief_740.log`, 5,359,465 B, sha256 `95725e26…`; report
`.temp/brief-740/report-continuation-2.json`, 128,365 B, sha256
`5bfef9b2…`; 11 changed files (7 added, 4 modified, 0 deleted), 0
disclosures, 4 deviations, 1 question (Q3, ruled here); 10 PASS / 1
FLAGGED / 1 FAIL / 2 NA; 290 citations verified; the continuation-2 report
is preserved at `.temp/brief-740/c2/report-before-ruling.json`
(`ffda534c…`) and the continuation-1 report at
`.temp/brief-740/c1/report-before-ruling.json` (`ee1a1121…`).

**What the run proved.**
- The admitted source entry is live at
  `fixtures/obligations/guest.rs:1326`; ordinary `programs()` still calls
  the same strict loader (`fixtures/obligations/guest.rs:1319`) and only the
  file reader is shared with `load_programs_with`. Parsing, phase
  attribution, native capture, validators and every existing call site keep
  their checks. My rerun of both modes: four source controls pass in each
  mode (20 named negatives per mode, `E0451` for a forged field, `E0308`
  for a wrong-arm use, the route postimage check red on a changed copied
  file and an unlisted path), and the live tree carries no production
  change.
- P0543's first complete own-omission packet: real arm binary
  `36d82c6d…` / tree `15bdbeb0…` / 86.890781 s; forced real 88.466939 s
  reproducing the identical binary; disabled `2302a212…` /
  `32fa2a67…` / 87.428718 s; armed `e1bf8a7f…` / `1c795634…` / 88.010693 s;
  one named runtime overlay per arm beyond the shared lane tree, live
  Cargo/production bytes unchanged, four compiler logs preserved. The real
  matrix is 55/55 at multiplicity one with ten native fields and the three
  checked-route `Native` falsifiers. The disabled arm's records are
  record-exact to the real arm: I extracted the 55 observations and 55
  lifecycle records from both raw logs and they are byte-identical (110
  lines, `5bd67e3a…`). The armed selected case differs from real in the
  resumed record, is equal in the queued record and the public output, and
  `guest::validate` yields `Err(Native)`; the source hit and the baseline
  producer pause are present. Packet: `.temp/brief-740/c2/p0543-own-omission.json`
  (`49bb6e7a…`), parity `disabled-parity.json` (`a72232b3…`).
- Maintenance was replayed after the final live edit and before any
  acceptance suite. The 68 `local-contracts.tsv` cells changed only in the
  last column; `promoted.tsv` changed exactly four generated certificate
  cells over the two rows (hash=true and hash=false columns). The initial
  exit-101 reds were stale-derived-cell ordering artifacts, preserved and
  not relabelled; the unchanged production acceptance then passed in both
  modes. The six ownership inventories, the rejection counts and every
  covered source site are unmoved. My diff check confirms the two files'
  changed columns are exactly those and nothing else changed outside the
  test layer.
- Counts and cost: the frozen requirement stays 151 own rows + 100
  held-pair cells + 4 structural controls = 255 per feature / 510 across
  the two. Seven of fifteen copied-build blocks were consumed (350.797131 s
  fresh + 256.517221 s prior = 607.314352 s), each build under 1,500 s;
  eight blocks remain; the full schedule is not measured. The live scope
  snapshot is 5,784 files, 7 added, 4 modified, 0 deleted,
  `production_changes=0`; no git or network call (104 recorded entries,
  0/0); `fmt --check` exits zero. 51 Open owners, C2 at 118 Residual + 0
  Lost, no settlement, promotion, SAME or encoding credit.

**The STOP is correct.** C10's accepted carrier still pins the current
`segments.rs` (`f9c7a2f9…`), and the retained C13 log
(`a27e408e…`) shows `semantic_same=true`, `native_same=false` and one epoch
fault — but that is historical calibration produced with a conditional
deletion of the quiescence assertion in the mutant collector
(`.temp/g15-c2/c12/prepare_main.py:35`). The current shared runner asserts
quiescence at `fixtures/obligations/guest.rs:2004` before it returns the
observation at `:2164`, so a current-source C10 catch cannot be observed
through the source-only entry. Importing the deletion, catching the panic
or patching a baseline observation would each be a bypass wearing a
catch's clothes; none is admissible. C10 is `PortBlocked` with a named
missing observation, 149 other own rows are `NotAttempted`, and no
acceptance suite was run.

**Q3 ruling — option 1, the mutation-observer path.** The live change stays
inside `fixtures/obligations/guest.rs`, test layer only, beside the strict
path; it is not a second loader and not a second schema.
- **The ordinary guard keeps its full force.** `run`, `run_adapter`,
  `programs()` and every existing call site keep
  `logical_profile_test_assert_quiescent()` in its present position and
  order. A shared body extraction is allowed only if every ordinary path
  still executes the assertion, and a falsifier must show that an armed
  ordinary run still hits it. The historical conditional deletion is never
  imported, and no env var may switch the shared runner's assertion off.
- **Constructor-private and row-admitted.** The observer is reachable only
  through an authenticator-produced, arm-typed capability plus the admitted
  row and its armed selector. A forged, unauthenticated, wrong-arm,
  wrong-row or unarmed use is rejected before any observation is supplied
  (compile-time where the types allow). The disabled arm keeps the ordinary
  path and must remain record-exact to the fresh real arm. The schema names
  which row carries an observation mode; C10 is the first, and every such
  row carries the same independent falsifiers. The path is never a
  blanket "run without quiescence" switch.
- **Actual, not synthesized.** The observer captures the row's actual
  native rows, output, phases/pauses and producer pause at the existing
  capture point, before the assertion, and continues the same runner body
  so the returned observation is comparable with the ordinary one. Its
  own predicate is the actual resumed-vs-queued difference in the
  native/output channels with equal public output; the epoch fault is
  provenance only and is never credited. It does not catch a panic, clear
  a fault, patch or synthesize a baseline observation, or count the
  quiescence assertion or any earlier guard as the row's catch.
- **Falsifiers, each red on both mandated feature modes:** wrong row;
  unarmed selector; absent source hit; absent producer pause; absent native
  difference; a quiescence-only rejection with equal native rows; a fault
  from another row; forged/unauthenticated/wrong-arm observer use;
  disabled/armed swap; the preserved negative that the ordinary strict
  loader still rejects `SourceIdentity` in a copied build; and the
  preserved negative that the ordinary armed run still hits the quiescence
  assertion. None of these is a semantic catch.
- **The catch bar is unchanged.** C10 is admitted only with its own source
  hit, its actual phase, the baseline producer pause, the disabled
  record-exact parity, its own `Native` red with equal public output, and
  the full real matrix still passing. Until that packet exists it stays
  `PortBlocked` with the concrete missing evidence.
- **Nothing else moves.** No runtime, production, manifest, lock,
  dependency, inventory or script change; the copies stay under
  `.temp/brief-740/`; no settlement, SAME, owner promotion or encoding
  execution. If the observer would need a runtime accessor to make its
  claim, that is a STOP and a new question, never an unrecorded scope
  expansion.

**Allocation.** A completely fresh twelve-block round: nine canonical
real/disabled/armed builds across the three configurations plus three
same-epoch forced byte-identity spot checks, each block ≤1,500 s. The
`guest.rs` edit changes every lane binary, so no build, spot check or
receipt is reusable; evidence may be reused only by authenticated identity.
The eight unspent continuation-2 blocks are voided — never consumed, never
transferable — so the campaign's consumed allocation reaches nineteen
blocks / ≤28,500 s at completion (an allocation, not a duration; the seven
consumed blocks measured 607.314352 s). The frozen case count stays
255 / 510; a second carrier schema or a schedule beyond the allocation
still STOPs. The P0543 packet remains immutable evidence of its own epoch
and cannot serve as the final schema's receipt; the final campaign
re-executes it on the final schema.

**Order.** Land the observer path and its falsifiers; rebuild the fresh
round; re-admit P0543 and C10 on the final schema; then the remaining 150
own rows, the 100 held-pair cells and the four structural controls; then
the promotion transaction; then the shape and falsifier gates in both
mandated arms; only then the complete unsampled campaign and the
acceptance suite. Retention replays before acceptance and every replay
verdict is quoted.

**Accepted as delivered.** The maintenance mirror under
`.temp/brief-740/c2/maintenance-workspace` (the replay tests write fixed
brief-544/546 output paths), the route-postimage falsifier added after the
immutable lane builds with the stale lane epoch explicitly not relabelled,
and the preserved administrative report-assembly failure. None carries
build, mutation or semantic credit.

**Unchanged.** C2 stays Open at 118 Residual + 0 Lost; no row settles, no
owner is promoted, no encoding key runs; `CHANGELOG.md` and S10 stay
blocked; the eight continuation-69 rulings (Q1–Q8) stand.
