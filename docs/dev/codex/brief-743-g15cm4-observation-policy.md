# Brief 743 — G15-CM4: the class-level observation policy, the final epoch and class admission

**Director-dispatched, one stage, one gate.** This executes the second stage of the accepted carrier-generation design (brief 741, Q9 ruling, commit `159101b6`), on the accepted generation and schema of brief 742 (Q88, commit `86aadc91`). It is test-layer only: separate observation transport from the immutable observer predicate, implement the class-level observation-mode policy, and admit the ten generation classes on their own in-epoch packets. It does not run the shakedown (744), the full campaign (745) or acceptance (746), and it claims no new semantic catch. `Scope: NONE` of production.

## Goal

Implement the class-level observation-mode policy exactly as brief 741 designed it — the 28 unchanged NativeEqual rows, the three deferred-publication rows and the seven profile-return rows become the **38 observer-mode rows**; the other **113 rows stay ordinary** — with every member's own packet required in both modes and no predicate change. Then freeze **one final epoch** on the accepted canonical identity, build the default/no-default × real/disabled/armed configurations (plus the forced real identity for each), run the class admission on the ten generation classes **all-or-nothing**, and persist either the complete admitted class table or the named `PortBlocked` findings with the full unspent ledger. A private capture permission is not a caught receipt; no class is admitted on historical evidence.

## Context pointers

- First stops: `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, the testing policy (`DISCLAIMER.md`, `AGENTS.md`), `docs/dev/code-organization-guidelines.md`.
- Design evidence, read-only, under `.temp/brief-741/`: `PLAN.md` (sections 1–3), `ROW-EVIDENCE.md`, `row-taxonomy.tsv`, `carrier-subtypes.tsv`, `SIGNATURES.md`, `attempt-plan.json`, `taxonomy-summary.json`, and `draft-743-observation-policy.md` (the design's own draft of this brief, for detail; this brief governs).
- The accepted 742 output: `mutations/generation/` (plan/resolve/graph/render/audit/tests), the regenerated `mutations/source_auth/overlays.tsv` (133 rows, digest `afb3110b309a9905d330bd70b1770d33d5090e269da5d340303ee4d5dcbfe142`) and the 151 rebound `inventory/obligations/mutants.tsv` cells; the Q88 ruling in brief 742's tail and `.temp/director/ruling-c88.md`.
- The observation surface: `mutations/source_auth/view.rs:33` (existing capture capability), `observer.rs:7/:19/:25`, `source_worker.rs:92`, the unchanged guest capture/guard at `fixtures/obligations/guest.rs:1993/:2023` and `:1927`, the predicate authority `records.rs:265/:374` and brief 740's Q8 record.
- The canonical composition of `.temp/brief-742/continuation-88/` (composer, `canonical.py`, the six-column emitter, the pin hold, the peel harness, the drive runner) is the delivered instrument set; copy what you need under `.temp/brief-743/`, never edit the sealed trees.
- The working tree carries the uncommitted CM4 test layer. Preserve it; never stitch, restore or reset it. No git command.

## Scope

**May change** only the test layer:

- `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/observer.rs` (class admission and use-time policy checks only);
- new `mutations/source_auth/observer_policy.rs` and `observer_policy_tests.rs` (the policy, its membership assertions and its falsifiers);
- `mutations/source_auth/mod.rs` and `mutations/mod.rs` (wiring only);
- `mutations/source_auth/source_worker.rs` (transport selection only);
- `mutations/source_auth/tests.rs` and `receipts.rs` (exact finite scratch-root admission only; no predicate or import-bar change);
- the generated policy data `mutations/source_auth/observer-policy.tsv`, and the 742 generation outputs regenerated **only** for changed test-file preimages and the new policy binding, with the mechanically updated `source_schema`;
- the standing replay is in scope.

The only additional export/import roots are `.temp/brief-742/` … `.temp/brief-746/` after canonicalization under the existing no-escape checks; preserve the prior brief-740 root. Never admit an arbitrary `.temp` path or an external root. Source_auth's baseline/copy pins bind the policy bytes and code. The final epoch and its execution instruments are created only under `.temp/brief-743/`, as copies of the reviewed canonical builder, with the existing P0543 manifest exception.

**MUST NOT change** `guest.rs`, `native.rs`, the adapter/host bodies, the ordinary guard, `records.rs`, `observer_receipts.rs`, any semantic acceptance or control body, program sources, coverage, selections or immutable row columns. **No new accessor.** Do not execute the remaining 123 rows or the full shakedown in this stage. No new selector and no broad environment bypass.

## Deliverable

1. **The policy** in the shape the design fixes — `ObservationMode::{Ordinary, AuthenticatedMutationObserver(class)}` with classes `NativeFault`, `DeferredPublication`, `ProfileReturn`, and one `observation_mode(spec, resolved_plan)` entry — generated from the structural plans and **asserting its expected memberships** rather than special-casing row names: NativeFault is the unchanged 28 NativeEqual rows; DeferredPublication is exactly P0267/P0312/P0323 and requires the producer → `complete_work` → owned-join topology; ProfileReturn is exactly P0256/P0259/P0266/P0314/P0315/P0410/P0515 and requires an actual-success return that bypasses the enclosing profile finish; all other 113 rows remain ordinary. The existing private construction and use-time row/program/source checks stay.
2. **The transport** uses the existing shared guest capture body: it captures actual counters before the guard only with the admitted capability and then performs its unchanged projections/fresh/pending/queue work; it does not redirect native or adapter early-return drivers.
3. **The class packets** retain every field PLAN.md §3 specifies, the complete real/disabled matrix and every constituent; `verify_own_omission` is unchanged; Strict allows only its frozen own predicate classes, non-Strict requires Native + a native difference + equal public output; the NativeEqual validator is unchanged; no sole quiescence-fault catch.
4. **The final epoch**: one frozen identity (source/schema/policy/compiler/binary/input hashes and exact command/environment manifests), the default/no-default/lane × real/disabled/armed configurations and the forced real identity for each — warning-free, same-epoch hash identity, the inherited release profile/codegen=1/cache rules, no old build or hash reuse, no thirteenth block.
5. **Class admission**: all ten classes **all-or-nothing** on their own packets in both modes — all members' packets must be valid; a failing member is a named `PortBlocked` with a class finding and blocks the class; the four failed rows keep `MissingRealTuple` and add `ObservationUnavailable/LogicalQuiescence` as diagnosis. Run the ten members in ascending row order, default then no-default, real/disabled/armed, plus the C10 calibration saved under the same final identity for 744. Record every ordinary-run quiescence negative. Admission consumes exactly the 20 logical own cells. Missing capture does not consume a family no-progress strike.
6. **Stop rules**: all-armed/all-hit but unobservable is an immediate class finding with a one-thread reproduction and a stop for design; an ordinary unobservable family stops immediately; an unsupported topology blocks its family; no row repair. If admission is blocked, stop without dispatching 744 and persist the named `PortBlocked` records and the complete unspent ledger.
7. **The evidence**: the per-class admission table (class, members, both-mode packets, verdict), the epoch identity, the standalone replay of the standing inventories, the canonical-emission invariants (hot text the live preimage; `cargo fmt --check` green; the peel at exactly live), the five carrier rows firing with their identities, the disabled cohorts, and the composed-vs-live workspace A/B with the declared enumeration.

## Verification (MANDATORY — you run it; iterate to green or to evidence)

- Before epoch creation, add and run the `cm_observation_policy_` gates in both modes: assert the 28 + 3 + 7 observer and 113 ordinary memberships; preserve the eight observer, ten auth and four Q7 predicate controls and add the class-crossing, wrong-arm/private-capability, policy-tamper, ordinary-source, unarmed, wrong-program/selector, post-admission alteration, same-native-with-only-fault, identity-survivor, disabled-difference, EarlierGuard and no-hit controls; calibrate every invalid packet red and every authentic control green; no receipts from synthetic data.
- Regenerate the schema and policy pins, replay the standing inventories **before** the first acceptance suite, and quote the replay verdict lines.
- Finish edits: `cargo fmt --check` and the whole workspace in both feature modes, plus the switched lp03/lp04/lp05 checks, green on the composed trees and the live control; the composed-vs-live A/B must carry the pre-enumerated inadmissible set with zero unexplained differences, and the nine native loopback groups stay for the director on the host (record the sandbox refusal).
- Freeze the exact 255 selections per mode and all 9,060 real keys / 9,511 constituents; preserve the accepted 742 audit. Build the nine configurations plus the three forced-identity configurations warning-free.
- The canonical fast-path invariants hold on the final-epoch trees: every rendered hot path is the live body, the peel is exactly 19,184.0000 / 3,360.0000 at every depth with depth 48 `done` at the default limit, and the five carrier rows fire from the twins in both arms.

## Checkpointing — protect the scene

Keep the source-stage acceptance separate from the final-epoch build receipts and the class packets in `.temp/brief-743/PROGRESS.md`. The freeze contains the source/schema/policy/compiler/binary/input identities and the exact command and environment manifests; authenticate the hashes before every resumed stage. Existing 743/epoch files become immutable; later stages only append unique evidence and derived indices.

## Budget and allocation

This stage owns the designed group: **4 × 1,500 s focused/maintenance envelopes** (6,000 s) and **2 × 6,000 s workspace envelopes** (12,000 s), and draws the **12 × 1,500 s final-epoch copied build/run blocks** (18,000 s, shared through 745) serially, with a stop and a report at every envelope boundary. The 743–746 pool is restored from 40,500 s to the Q9 design's 51,000 s by a fresh explicit 10,500 s allocation under the user's 2026-11-19 ruling (“预算不够就追加吧”); the c88 gate bullet's 8,500 s first-stage figure is superseded by this group. No scope is shrunk for cost; an envelope shortfall stops and reports the exact remainder.

## Hard rules

- DRAFT-COMMON applies: no production, ordinary-runner or predicate edit; no missing-real-tuple fabrication; no fault clearing; no panic catcher; no broad `.temp` permission; no runtime accessor; no single-row reclassification; no unapproved rebuild; no CM5.
- The pair assertion stays `(45, 5, false)` and the pair stays Open at zero admitted receipts; the four structural controls and the five armed-without-hit `Error` cells per pair row stay. C2 stays Open at 118 Residual + 0 Lost; the 51 owners and five source duties stay Open; `CHANGELOG.md` and S10 stay blocked; CM5 is not authorized.
- No git, no network, no `unsafe`, no new dependency, no external engine; all scratch under `.temp/brief-743/`; everything sealed (c76–c88) stays sealed and read-only; never run a historical instrument in its old root.
- The ordinary guard (`logical_profile_test_assert_quiescent()`) keeps its present source and call position byte-exact and its full force; no environment variable may switch it off; a needed runtime accessor is a STOP.
- A topology outside the reviewed three transport classes stops for class design, not for a new one-off path. Do not run the remaining 123 rows or the full shakedown.
- The report is `rum-report/1` under `.temp/brief-743/report.json`; the source stage and the class stage keep separate attempts, every member packet and ordinary negative is linked, and the final message is `scripts/report.py summary`.
