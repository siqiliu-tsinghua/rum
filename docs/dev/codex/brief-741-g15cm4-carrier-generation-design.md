# brief-741 — G15-CM4: the carrier-generation method (design only)

Scope: **NONE — no source change.** Survey the current tree and the CM4
epoch evidence, then return a reviewable plan plus the drafted
implementation briefs.

## Goal

Plan how CM4's own-omission carriers are generated, observed and
authenticated for every one of the 151 active Equal rows, so that the
next campaign is not another round of single-row schemas. The eighth run
of brief 740 proved the current method on 18 rows (including the three
Q7-repaired rows) and then hit the repeated two-family no-evidence stop:
four rows armed at their producer pause and emitted their source hit, but
the armed worker panicked at the logical-quiescence assertion before it
could return the observation — the same class of observation gap that Q3
solved for C10 alone. Under Q6/Q7 the one-off repair path ends here.
When you are done I must be able to review a step-by-step plan — every
claim carrying `path:line` evidence — that states the row-class
taxonomy, the mechanical carrier-generation rule, the observation-mode
policy (including the quiescence-broken class), the generated schema and
its authentication, the attempt plan with an explicit allocation
request, and the implementation briefs that follow, each with its own
mechanical gate. You write no code in this brief.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`.
- The governing designs: brief 736 (`docs/dev/codex/brief-736-g15cm-obligation-manifest-design.md`),
  the accepted CM3 layer (commit `23d0e0d9`), and brief 740 with its
  Continuation 7 ruling (`docs/dev/codex/brief-740-g15cm4-mutants-promotions.md`).
- The current epoch evidence, read-only, under `.temp/brief-740/c7/`:
  - `final-own-status.tsv` (`98fbfd84…`): 18 caught and 133
    `PortBlocked` per feature mode; the 18 caught are `C10.prepared.profiled_consumer`,
    `P0416`, `P0492`, `P0508`, `P0527`, `P0543`, `P0544`, `P0554`,
    `P0558`, `P0813`, `P0820`, `P0823`, `P0825`, `P0827`, `P1476`,
    `P1562`, `P1563`, `P2593`; 129 of the 133 are `RawArtifact`
    (never attempted) and four are `MissingRealTuple` (`P0267`, `P0312`,
    `P0323`, `P0410`).
  - `campaign-stop.json` (`af5fdd62…`): the stop at families
    `cm3-repaired` (`P0267`, `P0312`, `P0323`) and `cm3-set-rhs`
    (`P0410`), both modes, `raw_catches=0`.
  - The four armed logs (`campaign-default-armed-P0267-on.log`,
    `campaign-default-armed-P0312-on.log`,
    `campaign-default-armed-P0323-on.log`,
    `campaign-no-default-armed-P0410-on.log` and their no-default
    siblings): `CM740_SELECTOR_ARMED` at a producer pause,
    `CM740_SOURCE_HIT` at the declared phase, then the panic at
    `crates/rum-runtime/src/runtime/profiling/logical/epoch/snapshot.rs:30`
    ("logical teardown: pending children, retained terminal parents,
    open records, context leases, faults"), reproduced at one thread.
  - `frozen-schedule.json` (`d89799fe…`), `cost-final.log` (55 blocks,
    4,998.055882 s measured, 82,500 s allocated) and the twelve
    warning-free build logs with the three same-epoch forced identities.
  - The current uncommitted test layer in the working tree (brief 740's
    twelve changed files): `mutations/{schema,records,semantic_records,receipts,
    observer_receipts,negative_controls,duration_parity,observer_class_tests,
    source_worker}.rs`, `fixtures/obligations/{guest,semantic_controls}.rs`,
    `inventory/obligations/{mutants,mutant-coverage,promotions}.tsv`, the
    two retention inventories, and `source_auth/overlays.tsv` (90 rows).
- The observation precedent: the Q3 ruling in brief 740
  ("the mutation-observer path", `.temp/brief-740` c4 evidence) and its
  implementation in `fixtures/obligations/guest.rs`
  (`run_with_mutation_observer`, the authenticator-private capability,
  the eight observer negatives, the ten source-auth negatives).
- The Q7 findings: P0813 must arm at the actual producer pause of the
  resumed catch body and omit at the actual `resume_catch_after_body`
  dispatch on that resumed path (`nonlocal.rs`
  `ResumeCatchBodySuspensionFrame::resume`); P2593 must omit at the
  decoded `Some(value)` in `FirstLazyCompletionFrame::complete`
  (`structure_ops/surface/lazy_completion.rs:32`) as well as the
  immediate `Ok(Some)`; a `Strict` row may be caught by its own allowed
  predicate class (`Err(Output)` and siblings), while the 54 non-Strict
  rows keep `Err(Native)` with a native difference and equal public
  output.
- The campaign invariants: brief 623, "Campaign invariants" and the move
  rules — parks, G13-owned encoding, the 13 pair columns, the two-family
  stop-loss, the entry allowance, verification reuse and build control.

## Scope

- Files you MAY modify: **none** under `crates/`, `docs/`, `scripts/` or
  anywhere tracked. Scratch, copies and logs go under `.temp/brief-741/`.
- Read-only: the production tree, every `.temp/brief-740` scene, the
  accepted CM3 layer and the historical scenes named above.
- No git, network, `unsafe`, inventory edit, `Cargo.lock` change or
  production-source edit. Copied trees only when a survey needs one.

## Deliverable

A plan I can review and rewrite, in this order:

1. **Row-class taxonomy.** For every one of the 151 rows, the source
   key, tail site, producer pause, declared phase, selector, observer
   class and the observation mode the current evidence shows it needs
   (ordinary runner / Q3 observer / quiescence-broken class). Cite the
   18 caught rows' signatures and the four failed rows' signatures.
   Classify the 129 never-attempted rows mechanically from their source
   keys and the accepted CM3 evidence, and mark every class that is a
   hypothesis rather than a proven signature. No row may be left
   unclassified; a genuinely unique row is named with the evidence that
   makes it unique.
2. **The carrier-generation rule.** Per class: how the arming site is
   derived from the row's actual producer pause; how the omission site
   is derived from the row's actual tail; how the hit is emitted at the
   declared phase; how the selector is named; how the disabled arm stays
   record-exact to the real arm; and how the rule makes the P0813/P2593
   defects impossible by construction (no arming at an initial match, no
   omission at an immediate result the resumed path bypasses). State the
   falsifier set per rule: unarmed, hit-before-arm, out-of-class
   predicate (`EarlierGuard`), identity survivor, disabled difference,
   no hit, raw artifact.
3. **The observation-mode policy.** For every row whose omission leaves
   the profiler's logical-quiescence invariant broken (the four named
   rows and any others the survey finds): how the observation is
   obtained without weakening the ordinary guard, using the Q3
   mutation-observer precedent; the class predicate for each observer
   class (`Strict`, `NativeEqual`, `StrictNativeEqual`); the shared
   falsifiers; the admission rule (class-level, not per-row); and the
   exact evidence a class-level admission must carry. State how a row
   that cannot be observed even through the observer path is reported —
   named `PortBlocked`, never Equal, never silently dropped.
4. **The generated schema and its authentication.** How the overlay set,
   preimages/postimages, selectors and the `source_schema` digest are
   generated and authenticated when the carrier set is generated rather
   than hand-placed; how the current 90-row schema extends to the full
   set; and how no row's source key, tail digest, declared phase,
   selector, observer or coverage bar changes.
5. **The attempt plan and allocation.** How the remaining 133 rows are
   attempted in the next epoch: the complete 510-case schedule re-freeze
   (151 own selected cases, 100 held-pair cells, four structural
   controls; 255 per mode), the block count, the order, the cost model,
   and an explicit allocation request. The Q6/Q7 grant is fully
   consumed (55 blocks, 4,998.055882 s measured, 82,500 s allocated);
   any new build or schedule spending needs my ruling, and this plan is
   the request. Include the acceptance plan and the replacement for the
   two-family stop rule: a family whose rows are all armed and hit but
   unobservable is a *class* finding and must stop for this design, not
   silently consume the no-evidence budget; state the stop rules the
   next campaign runs under.
6. **Drafted implementation briefs**, in `TEMPLATE.md` shape, staged so
   each has its own mechanical gate and each is independently
   reviewable: the generation rule and schema first, the
   observation-mode policy second, the shakedown on the 18 already-caught
   rows third, the full campaign fourth, the acceptance fifth. Name what
   each brief may change and what it may not.

## Hard rules

- Design only: no source, test, inventory or documentation edit; no
  build, no campaign, no acceptance.
- No git, network, `unsafe`, external engine, new dependency or write
  outside this repo and `.temp/brief-741/`.
- No production, manifest, lock, dependency or inventory-predicate
  change; no settlement, SAME, owner promotion or encoding execution.
- The pair assertion stays `(45, 5, false)` and the pair stays Open at
  zero admitted receipts; the four structural controls and the five
  armed-without-hit `Error` cells per pair row stay.
- The ordinary guard (`logical_profile_test_assert_quiescent()`) keeps
  its present position and force; no env var may switch it off; a
  needed runtime accessor is a STOP.
- A missing authentic input or an inadmissible mutant blocks its family
  and is reported; it never becomes Equal.
- C2 stays Open at 118 Residual + 0 Lost; the 51 owners and five source
  duties stay Open; `CHANGELOG.md` and S10 stay blocked; CM5 is not
  authorized.

## Director ruling (design gate) — Q9 admitted: the method, the observation classes and the 69,000-second staged allocation, with four pinning conditions

Accepted as delivered, and the design task is complete. The taxonomy
covers all 151 rows in ten generation classes (R34 F30 T73 J3 E2 G1 N4 U1
H1 A2), keeps the 79 carrier subtypes (45 singletons) as plan parameters
and separates the observer predicate (97 Strict / 28 NativeEqual / 26
StrictNativeEqual) from the observation transport; the five-step
generation rule derives arming from the actual producer episode, traces
every immediate and resumed successor, renders disabled/armed from one
compile-time constant, and audits the emitted bytes against the pre-edit
pins rather than the renderer; the P0813 and P2593 construction proofs
make both Q7 defects static generator failures. The observation policy
keeps the 28 NativeEqual rows, adds the three deferred-publication rows
and seven profile-return rows (38 observer-mode rows, 113 ordinary),
requires every member's own packet in both modes and never changes a
predicate. The attempt plan is the complete 510-case schedule (20
admission + 244 shakedown + 246 campaign; 9,060 real keys and 9,511
constituents per mode; 161 own constituents), with the 51 remaining
families ordered and P1470/P1471 first. The replacement stop rule
(all-armed/all-hit but unobservable = immediate class finding, one-thread
reproduction, stop for design) is the Q8 rule.

Q9 admitted. New allocation granted, stage-owned, with no transfer from
the exhausted 55-block/82,500 s grant and no refresh at a stage
boundary: twelve final-epoch build/run blocks x 1,500 s = 18,000 s
(owned by 743, shared through 745); eight source-stage focused/maintenance
envelopes x 1,500 s = 12,000 s (four each in 742 and 743); six workspace
envelopes x 6,000 s = 36,000 s (two each in 742, 743 and 746); two final
switched/reproduction envelopes x 1,500 s = 3,000 s (746). Total 28
envelopes / 69,000 s. Each envelope is a ceiling, charged serially; a
bound failure, an extra required build or a post-freeze schema change
stops without spending another envelope. Brief 742 is authorized now for
its 18,000 s (four focused/maintenance + two workspace envelopes);
743-746 are authorized only by their own entry gates, each drawing from
its own group.

Four pinning conditions:

1. **Pin authority.** The 90-file overlay set is the starting set. For
   every retained carrier the independent audit must prove the same
   omission site and preserved nodes as the current pre-edit pin; a
   byte-level delta in an existing file is disclosed per file with its
   structural justification at 742's gate, never silently normalized, and
   the pin — not the renderer — is the authority. The generator may not
   use a historical full-file postimage as its logic.
2. **Signature pins.** The 18 caught rows and the four stop rows keep
   their exact arming, hit, phase and predicate signatures; the generated
   carriers are re-earned in the final epoch, and any signature drift
   stops rather than being absorbed as a new expectation. C10's packet is
   reused only under identical epoch identity.
3. **Class admission is all-or-nothing.** All ten members must yield a
   valid packet in both modes; a failing member is a named PortBlocked
   with a class finding and blocks the class. No member substitution, no
   cherry-picking, no admission by historical evidence: P0515's
   faults=950 log is a calibration hypothesis, never an in-epoch packet.
   The four failed rows keep the importer reason `MissingRealTuple` and
   add `ObservationUnavailable/LogicalQuiescence` as diagnosis; neither is
   rewritten as missing CM3 coverage or as Equal.
4. **No row repair.** No new single-row repair path is authorized. A class
   failure returns for design; an ordinary unobservable family stops
   immediately; an unsupported topology blocks its family.

All other terms stand: test-layer only, no production, ordinary-runner,
predicate or receipt-acceptance change, no new runtime accessor, no
manifest permission beyond the existing nine-entry P0543 lane
(`66a8e8bd…`); the pair `(45,5,false)` stays Open with zero admissions,
C2 stays Open at 118 Residual + 0 Lost, the 51 owners and five source
duties stay Open, encoding stays G13-owned, `CHANGELOG.md` and S10 stay
blocked, and CM5 is unauthorized.
