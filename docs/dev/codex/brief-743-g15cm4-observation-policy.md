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

---

## Gate c89 (2026-11-21) — the source stage accepted, the composition owed

The first stage ran on `grk` (unit `brief-743-grk-20261121-075713-982190`, 07:57:13-11:16, rc 0,
success, 341 turns, $28.45403, peak 67.0% of 20 GiB, 3 git-fence reads). Report 46,135 B, STOP,
8 PASS / 1 FLAGGED / 2 FAIL, 73 citations. The full record is `.temp/director/ruling-c89.md`.

Accepted with my own reproductions: the policy (28 NativeFault / 3 DeferredPublication / 7
ProfileReturn / 113 ordinary; both modes 6/6 with every negative control firing; the transport
replaces the 28-row test in `mutations/source_worker.rs`; `verify_own_omission` and the guard's
two byte positions untouched); the replay (68 fingerprint + 2 certificate rows, exactly the git
diff; ownership 61/79/8866; LP01 and typed slots both modes; fmt exit 0); the workspace both
modes with only the nine ZeroMQ sandbox refusals (9/9 on the host in both modes; the two c88
live-baseline Program reds now ok); the freeze (5,810 pins, 0 mismatches now; 255 selections =
151+100+4; 9,060 keys / 9,511 constituents; all six tree hashes recomputed); block 1
`default-real` (fresh, 0 warnings, binary `c1f68187...`; the ten members + C10 admitted; I
reproduced P0267); and the corrected warning bar — block 2 compiled with cargo exit 0 and the
literal "warning-free" gate rejected 128 warnings that are exactly the declared classes (124
`unused_parens` at the sealed c88 `build-drive-disabled.log`'s set, 124 = 124 with zero
difference; 3 composer name warnings; 1 `unused_must_use`), so block 2 is admitted without a
retry and the label stays consumed. Scope: exactly 10 repo files (3 added, 7 modified), all in
the brief's test layer; sealed c76-c88 sources unchanged; the disclosed
`mutations/source_worker.rs` path deviation accepted.

The epoch's composed trees are **not** accepted: `overlays.tsv` has 133 pins and the frozen
trees carry only the 126 render files, missing the 4 recorder-removal postimages and the 3
driver-hook fixtures (`guest.rs`, `native.rs`, `semantic_controls.rs`). The class vector on the
frozen arms is `extension=4` where c88 had 25, and no `cm740_probe::begin` call site exists, so
`run_is_driven()` can never be true and no twin can fire; the authentication would fail
`Postimage`/`UnlistedFile`, and a drive tree fails `UnlistedFile` because its instrument is not
a pin. The receipt writer uses the 126 render deltas instead of the 133 schema pins (I hit
`OverlaySet`), and `admit_arm.py` never checks the armed hit lines. I proved the fix: tree = the
live snapshot + the 133 overlays.tsv armed postimages, receipt with the 133 pins — armed P0267
authenticated, `observer_transport=true`, `CM740_SELECTOR_ARMED`, `CM740_SOURCE_HIT`,
`predicate=Err(Native)`, exit 0 (tree `62826b8c...`, binary `d4b7718c...`; the worker builds its
own and reuses neither).

Findings recorded: F1 the 7 missing pins; F2 drive instruments unlisted in admission trees;
F3 the 126-key receipt; F4 no hit verification in `admit_arm.py`; F5 the stale-binary first
workspace run wrote 171 derived files into sealed continuation-83 (no source changed; preserved,
never deleted); F6 the disclosed `.temp/brief-546/certificates` symlink redirection (the fixed
path of `local/production.rs:13`; prior files moved aside and preserved).

Order: continuation 90 (the corrected composition, blocks 3-12, the disabled/armed/no-default
admissions with fired hits, the class packets all-or-nothing, the carrier drive on the drive
trees, the peel exactly live, the composed-vs-live A/B with the pre-enumerated set). Budget:
stage 1 charged 969.912 s focused + 5,221.824 s workspace + 217.511 s epoch = 6,409.247 s;
743's remainders are 5,030.088 s focused, 6,778.176 s workspace, 17,782.489 s epoch blocks; the
stage reserve is 37,033.4781416041 s with no new allocation; CM4 is now 173,605.23604906694 s.

---

## Continuation 90 — the corrected composition and the remaining epoch

Gate c89 (`.temp/director/ruling-c89.md`) accepted the source stage, the freeze identity, block
1 and block 2 under the corrected warning bar. It found the frozen composed trees carry only 126
of the 133 schema pins and that the admission instruments cannot authenticate or fire. This
continuation builds the full composition, finishes the epoch and the class admission. Read the
ruling first.

**Goal.** One final epoch identity on the accepted canonical form: the full 133-pin composition,
blocks 3-12 under the corrected warning bar, the disabled/armed/no-default class admissions with
real fired hits, the ten class packets all-or-nothing on their own both-mode evidence, the five
carrier rows from the drive trees, the peel exactly live, and the composed-vs-live workspace A/B
with the declared enumeration. A private capture bit is not a receipt; a no-hit armed run is not
an admission.

**The composition.** The admission trees are the live snapshot (the frozen `real` tree,
`495696b8...`) plus, for each of the 133 rows of
`crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/overlays.tsv`, that arm's
postimage column. Every row's preimage digest must equal the baseline file's sha256 and every
written file's sha256 must equal the row's postimage digest; the copied file set must equal the
baseline's exactly (`.git/target/.temp/__pycache__/.mypy_cache` are excluded by `Snapshot::read`
and the tree hash). Verify the seven non-render pins explicitly (the four recorder files and the
three hook fixtures). Seal the admission trees and record the exact per-file before/after
digests. The frozen pre-instrument trees stay immutable. Drive trees = admission tree +
`obligations/cm742_c81_drive.rs` + the `obligations/mod.rs` hook, for `drive_arm.py` only; never
authenticate on a drive tree (its extra files are unlisted). Record their hashes. Append the
admission and drive tree hashes and the exact deltas to the epoch identity; never rewrite it.

**The blocks.** 3-12 in the designed order: `default-armed`, `default-spot`,
`no-default-real`, `no-default-disabled`, `no-default-armed`, `no-default-spot`, `lane-real`,
`lane-disabled`, `lane-armed`, `lane-spot`; one label each; `default-disabled` is consumed and
admitted under the corrected bar — no retry, no thirteenth block. Corrected warning bar: every
warning must belong to a class present in the sealed continuation-88 log for the same arm, and
the `unused_parens` (file, line) set must equal the declared 124-site set exactly; zero warnings
of any other class; quote the per-class counts and the location-set diff. Fresh at every block:
`cache_hit=false`, release, codegen units 1, no old build or hash reuse, elapsed ≤ 1,500 s.

**The admissions.** The receipt carries arm/baseline_tree/binary/codegen_units/configuration/
profile/tree plus the 133 overlay lines (file, preimage, the arm's postimage digest); the
authentication must pass and its verdict is quoted. `default-disabled` and `default-armed`,
`no-default-real`/`disabled`/`armed`, ascending rows, one-thread reproduction on any failure,
the C10 calibration (`default-real` is already admitted). The armed arm must show, for every
member: `CM743_TRANSPORT ... observer_transport=true`, `CM740_SELECTOR_ARMED`,
`CM740_SOURCE_HIT`, the own predicate with the authentic native difference, and the class
packet's arm-then-hit and observations true; a missing hit is a named PortBlocked with the class
finding. The disabled arm must show `observer_transport=false`, zero class hits, and the inert
cohorts 60/60/60/45/45. The ten classes are all-or-nothing on each member's own both-mode
packets; exactly the 20 logical own cells; no receipts from synthetic data; the four failed rows
keep `MissingRealTuple` plus `ObservationUnavailable/LogicalQuiescence`.

**The carriers, the peel, the A/B.** Carrier drive on the drive trees: five rows, the identities
(join and queue generations, pair, occurrence), pins applied after the build and reverted, the
disabled cohorts. Peel: the harness and the corpus test, `19,184.0000 / 3,360.0000` at every
depth, depth 48 `done` at the default limit, both arms and the live control. A/B: the 133-pin
composed trees vs the live workspace in both feature modes; declare the inadmissible set before
the first run (the c87/c88 eight families plus the new policy/test-layer names); the seven
skips; zero unexplained differences; scratch under the excluded `.temp`/`target` so the tree
hashes are unchanged before and after; the nine native tests stay for the director on the host
(record the sandbox refusal).

**Scope and hard rules.** Test-layer only; no production, ordinary-runner, predicate,
guest/native/adapter, program, coverage, selection or immutable-column change; no new accessor;
no new selector or broad environment bypass; no row repair; no live repo mutation beyond the
accepted source stage; everything sealed (c76-c89) stays sealed and read-only; do not delete the
171 derived files the stale binary wrote into continuation-83 (F5) — record them;
`.temp/brief-743/` only; no git, no network, no `unsafe`, no new dependency. A topology outside
the reviewed three classes stops for design. If admission is blocked, stop without dispatching
744 and persist the named PortBlocked records and the complete unspent ledger.

**Budget.** 743's remainders: focused 5,030.088 s (4 envelopes), workspace 6,778.176 s (2
envelopes), epoch blocks 17,782.489 s (10 labels at 1,500 s). Stop and report at every envelope
boundary; the unused returns per the standing rule; a shortfall stops and reports the exact
remainder; no scope is shrunk (the 2026-11-19 ruling if a top-up is ruled).

**Report.** `rum-report/1` at `.temp/brief-743/report.json` (the continuation attempt appends
attempts); every block, admission, packet and ordinary negative linked; final message
`scripts/report.py summary`.
## Gate c90 (2026-11-21) — the corrected composition and the blocks accepted, the profile-return class found unsatisfiable, a design ruling owed

Continuation 90 ran on `grk` (unit `brief-743-grk-20261121-114240-1112219`,
11:42:40-13:02, rc 0, success, 155 turns, $14.234372, peak 35.8% of 20 GiB, 3 git-fence
reads). Report 66,228 B, STOP, 10 PASS / 2 FLAGGED / 3 FAIL, 115 citations, 0 questions,
6 deviations, 10 disclosures. The full record is `.temp/director/ruling-c90.md`.

Accepted with my own reproductions: the corrected composition (live `6af3d99f...` = frozen
real `495696b8...` + the three journal files exactly; all 133 preimages and postimages
verified; the four admission trees `6cef83af...`/`c5d599a6...`/`4eac7561...`/`b9b1fde9...` =
baseline + the 133 pins + the journals, file set 5,810; the four drive trees = admission +
the reviewed instrument + the `obligations/mod.rs` hook at `3041233e...`/`e566d734...`/
`341f4c95...`/`de8f4cd4...`; the seven non-render pins; 5 `cm740_probe::begin` sites;
`identity.json` `c5efa506...` and `copies.json` `71850b35...` unchanged; all six frozen trees
re-hashed; the 171 derived files in continuation-83 preserved); blocks 3-12 in the designed
order, 12 labels / 12 starts / 12 finishes, no thirteenth, `default-disabled` not retried,
under the corrected bar exactly (my parse of every json compiler log: armed 132 warnings / 6
classes / the 124 `unused_parens` sites equal to the sealed c88 armed set, disabled 128-130 /
5 classes / the same 124 sites equal to the sealed disabled set, real and spot 0 warnings;
`cargo_exit=0`, `cache_hit=false`, release, codegen units 1, max 116.4 s of 1,500 s; every
oracle hash equals its receipt); the forced identities (`no-default-spot` and `lane-spot`
pass; `default-spot`'s dirty incremental rebuild `fb054603...` is superseded by the disclosed
clean rebuild `c1f68187...` equal to block 1, ledger exit 3, no second label); the
admission-only 133-pin `default-disabled` build (`3fed2829...`, 130 warnings / the 124 sites
exact, outside the twelve labels, disclosed); the 133-pin receipts (7 fields, 133 overlays,
`baseline_tree=6af3d99f...`) and the armed P0256 authentication; the stop (P0256 armed:
transport true, selector armed, predicate `Ok(())`, `native_difference=false`, no source hit;
one-thread reproduced; named PortBlocked; ten rows unspent; 744 not dispatched); scope and
seals (0 tracked files modified after the dispatch, the inherited porcelain unchanged, no
sealed file newer, the worker wrote only under `.temp/brief-743/`).

The finding is the class: **the profile-return class is unsatisfiable as designed.** My own
breadth calibration on the default-armed admission tree (all ten members, the admission's
exact environment) gives: the seven profile-return rows — P0256/P0259/P0266/P0315 arm a
selector once or twice and P0314/P0410/P0515 never arm; none produces `CM740_SOURCE_HIT`;
all seven are `Ok(())` with `native_difference=false` and equal public output; the three
deferred-publication rows (P0267/P0312/P0323) each arm, hit, and show `Err(Native)` with
`native_difference=true`. In the ordinary lane under the same binary the three deferred rows
hit and then panic at the logical-teardown assertion, while the seven profile-return rows
exit 0 with no hit and no quiescence failure — their ordinary run is clean. The canonical
render at the site is `cm740_probe::around_ok(row, phase, false, || ..._cm740_driven(..))`;
its whole difference is the hit print and an early return of the same value, the hit is
gated on `ordinary(row)` (the queued-run authentication), and that gate never succeeds
there because the queued run's frame permit belongs to a deeper activation than the
wrapper's `enter` activation. `verify_own_omission` would also reject the packets
(`IdentitySurvivor` — the armed observation must differ from the real baseline; an own
`Err(...)` predicate is required; a fired hit is required). So for all seven members the
declared operation is not exercised, the declared hit is unreachable, and the transform is
an identity. This is a class-wide design finding: no single-row repair is authorized.

Ruling `Q90-composition-blocks-and-class-finding`: the composition, the blocks, the spot
identity, the admission-only build and the stop are accepted; the ten-member all-or-nothing
admission is not credited (the pair stays Open at zero admitted receipts); the profile-return
class returns for a design ruling; the deferred-publication evidence is noted but its own
admission (disabled at zero hits, both modes, real matrices, the packet) is unrun.

## Continuation 91 — the profile-return class: design-only calibration and ruling package

Read the c90 ruling first (`.temp/director/ruling-c90.md`), then the accepted 741 design's
`PLAN.md` section 3 and `ROW-EVIDENCE.md` for the seven rows, and the c90 breadth evidence
(`.temp/director/gate-c90/README.md`, `breadth-summary.tsv`, `ordinary-breadth-summary.tsv`,
`breadth.py`, `ordinary_breadth.py`, and the per-row logs). This is design only: **Scope:
NONE**. No source change, no live build, no test change, no admission, no epoch, no campaign,
no 744. Everything sealed stays sealed and read-only; scratch and instrumented copies only
under `.temp/brief-743/design/`; the frozen/composed trees, `identity.json` and `copies.json`
are not written.

**Goal.** Return a ruling package that decides what the profile-return class is, why its
members cannot produce the accepted evidence, and what the correct design is — with one
recommendation, a per-row contract, falsifiers, the impact on the 743 policy and the 742
schema, and a costed staged plan. You may run the admitted evidence and instrumented copies
to answer, but you change nothing.

**Reproduce the evidence.** Re-run the ten-member armed breadth and the ordinary breadth on
the default-armed admission tree exactly as the c90 gate did (environment in the gate's
`breadth.py`/`ordinary_breadth.py`: `RUM_CM740_ROW`/`OMIT`/`PROGRAM`/`ARM=armed`/`PROFILE=on`/
`BASELINE`=repo/`RECEIPT`=the 133-pin `default-armed-authentication.tsv`, profile group on,
`RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=0`; ordinary adds `RUM_CM740_ORDINARY=1`; the binary
is `epoch/default-armed-oracle`, the active tree is `c90-default-armed-6cef83af...`). Quote
the verdict lines and reconfirm the selector split and the predicate split.

**The diagnosis (all seven members, source-anchored).**
1. Per-member eligibility trace: which clause of `cm740_probe::ordinary`/`eligible` fails at
   the declared site in the queued run — run not queued or not current, episode missing,
   `owner` missing, `pair.tokens` missing for the wrapper activation, `frame_identity` absent,
   or `issued_queued_frame` false — and where the frame permit was actually issued in the
   resumed run (its owner activation and path versus the wrapper's `enter` path). Instrumented
   copies of the composed tree under `.temp/brief-743/design/` are allowed; builds only there.
2. The three rows that never arm (P0314/P0410/P0515): prove why the declared producer pause is
   not reached or not armed on the selected program, and whether the declared phase/prompt in
   the row schema matches the actual pause.
3. Render semantics: show that each of the seven emitted transforms is an identity (no value
   change, no skipped tail), compare it with the declared operation (actual result at the
   enclosing return boundary, early return before the tail), and name the exact site and shape
   where a driven-path difference should exist.
4. The unchanged `verify_own_omission` clauses that reject the current packets
   (`IdentitySurvivor`, own-predicate, hit, build identity) and the exact evidence it requires;
   state what a valid packet must contain for this class.

**The design decision (one recommendation).** Consider at least: (i) regenerate the seven
carriers so the driven path returns the declared early result with a genuine native difference,
equal public output and a reachable declared hit, keeping the hot body the live preimage and
the canonical form; (ii) an evidence contract the unchanged predicate can pass without
weakening it; (iii) a class-wide reclassification with its consequences for the 151-row bar
and the campaign. Then state: the per-row render contract and its falsifier; the impact on the
743 policy (memberships, `ClassPacket`, the 20-cell schedule, the class stop rules); the 742
schema/overlay re-emission and re-authentication; the costed staged briefs and envelopes with
any explicit new-allocation request; risks and stop conditions. No single-row repair.

**Questions the ruling must answer.** Q1: is the declared operation satisfiable by any render
within the reviewed probe protocol at these sites — if so, what exactly, and if not, which
clause of the class definition is wrong? Q2: should the class exist at all, given the
seven-row ordinary lane is clean and quiescence-free? Q3: what must be re-derived and
re-authenticated if the class is removed or re-scoped? Q4: does the corrected composition
leave the deferred-publication evidence valid, and what remains to complete its own admission?

**Budget.** 4,000 s focused, carved from 743's focused remainder (5,030.088 s), leaving
1,030.088 s; no workspace and no epoch blocks; stop at the envelope boundary and report the
exact remainder. Report `rum-report/1` at `.temp/brief-743/report.json` (append attempt),
design sections, every verdict line quoted, final message `scripts/report.py summary 743`.
## Gate c91 (2026-11-21) — the profile-return diagnosis accepted, the reclassification rejected, a non-weakening protocol review ordered

Continuation 91 ran on `grk` under the user's 2026-11-20 routing ruling (unit
`brief-743-grk-20261121-132004-1142247`, session `6e94be85-6461-47f0-af45-4dc89975c5b2`,
13:20:04-13:59, rc 0, success, 98 turns, $7.80657, refused 0, peak 8,068,964,352 B of 14 GiB
(53.7%), oom 0/0, 3 git-fence reads; report 113,677 B, STOP, 14 PASS / 2 FLAGGED / 3 FAIL /
1 NA, 196 citations, 9 deviations, Q1-Q4, 10 sections). The three FAILs are the c90 stop's own
entries. The full record is `.temp/director/ruling-c91.md`.

Accepted with my own reproductions: the reconstructed baseline (`6af3d99f...`) and the active
default-armed admission tree (`6cef83af...`) hash to the receipt values; the ten-member armed
and ordinary breadth reproduces my c90 tables row for row (the seven profile-return rows hit
never — P0314/P0410/P0515 never arm — and are `Ok(())` with no native difference; the three
deferred rows hit with `Err(Native)` and a native difference; the ordinary deferred rows panic
at `snapshot.rs:30` while the seven exit 0); the eligibility trace (no `CM743_BIND` anywhere,
empty token and permit paths, `pair_tokens_missing` at the reached queued sites, P0515's driven
fast path still missing the pair token, P0314/P0410 with no own site and an `owner_missing`
arming); the render audit (the accepted `around_ok` returns the body result and the caller tail
runs at all seven sites; the live code has no early return; the 741 declared early return is
the overlay shape the render does not emit); the unchanged `verify_own_omission` clauses and
their order; the policy rows (`success_return=1`, `bypasses_profile_finish=1`, the
`around_ok:...` evidence, `profile = success_return && bypasses_profile_finish`); scope (0
tracked files newer than the dispatch, the inherited 23-entry porcelain unchanged, no sealed
scene written, the worker wrote only under `.temp/brief-743/design/` plus the appended
report); the budget (279.26 s metered of the 4,000 s carve).

The recommendation (iii) — reclassify all seven to ordinary — is **not accepted**. A clean
ordinary lane is the 741 packet's expected case ("if ordinary returns, its observation must
equal the observer-path observation"), not evidence that the ordinary runner can observe the
native-only omission; the seven carriers' selected mutants are diagnostic-native tail omissions,
and reclassifying would settle five unattempted hypotheses by an observation that cannot see
them — a semantic loss and a reclassification wearing a repair's clothes. The package also
treated the accepted probe protocol as fixed, so it never evaluated the one question that could
preserve the class: a non-weakening authentication for a queued-side success at a call-site
probe whose frame construction binds to a deeper activation. The class stays, the policy is not
re-derived, and the deferred class's own admission waits for the policy decision.

Q1: under the accepted protocol the declared operation is not satisfiable (the hit is
unreachable and the render is an identity); the unimplemented clause is `success_return &&
bypasses_profile_finish`, which is the carrier's declared diagnostic omission, not a live-tail
property. Q2: the class should exist; removal, if ever, is a ruled scope cut. Q3: a new
primitive or a site re-derivation re-emits the affected overlays and source schemas and
re-derives the policy and digests, then re-freezes and re-authenticates every packet. Q4: the
deferred evidence remains valid; its admission still needs the disabled arm at zero hits, both
modes, the real matrices, the cohorts and the packet.

## Continuation 92 — the profile-return class: the non-weakening path, design only

Read the c91 ruling first (`.temp/director/ruling-c91.md`), then the 741 `PLAN.md` section 3
and `ROW-EVIDENCE.md` for the seven rows, the 743 policy rows, and the c91 evidence
(`.temp/brief-743/design/`, its `STATUS.md`, `breadth.json`, `ordinary-breadth.json`,
`diag_rows.py` and `logs/diag-*.log`, and the c90 gate's `breadth.py`/`ordinary_breadth.py`).
This is design only: **Scope: NONE**. No source change, no live build, no test change, no
admission, no epoch, no campaign, no 744. Everything sealed (c76-c91) stays sealed and
read-only; scratch, instrumented copies and prototype builds only under
`.temp/brief-743/design2/`; the frozen/composed trees, `identity.json` and `copies.json` are not
written.

**Goal.** Decide, with evidence, whether the profile-return class can be admitted with the
unchanged predicate and no weakening, and return one recommendation with the per-row contract,
the falsifiers, the policy/schema/re-freeze impact, answers to Q1-Q4 and a costed staged plan.

**State the class precisely.** Quote the seven policy rows (`success_return`,
`bypasses_profile_finish`, the evidence string) and the live tails at the seven sites: the
live code runs the finish; the declared early return is the carrier's simulated diagnostic
omission, so the class is an oracle-fidelity obligation (the observer must catch the simulated
native-only omission). State what the class is for and what its removal would leave unobserved.

**Route A — a non-weakening queued-success authentication.** Enumerate the ledger records the
resumed half produces for the producer's suspension frame (construction identity, bound slots,
current frame records, the pair scope) and determine which are pair-scoped, unforgeable from
the probe, and survive into the queued half. Specify the primitive that lets the wrapper
activation prove the construction of a descendant: its inputs, its invariant, its state, and
why it is not weaker than `ordinary()`/`frame()` (what it verifies, what it cannot). Falsifiers
that must fire: wrong pair/row/episode, wrong arm, wrong phase, pre-arm hit, a probe without
the construction, a construction outside the pair, an ambient-env or forged capability, a
disabled-arm hit. Show it is false without the real resumed-half construction, on an
instrumented copy, and that the existing `ordinary()`/`frame()`/`eligible()` and the ordinary
guard's two byte positions are untouched.

**The render.** The call-site conditional early return of the declared shape: when the
authenticated gate holds and the actual producer result is `Ok`, hit at the declared phase and
return `Ok(value)` before the caller tail; otherwise return the body result and let the tail
run. The hot path stays the live preimage; the driven path is the cold twin plus the gate;
composer-generated canonical text with a byte-exact inverse and no hand edits. State the
native-difference mechanism (the omitted finish), the public-exact behavior, the disabled arm
(zero hits, exact), the ordinary-run equality and the quiescence outcome.

**Prototype and decide.** On the instrumented scratch copies, show for all seven: the armed
hit at the declared phase, `Err(Native)`, `native_difference=true`,
`public_output_exact=true`, and the unchanged `verify_own_omission` accepting; the real and
disabled matrices; the ordinary equality; or the exact blocking clause. If Route A fails, state
why with the offending clause, then evaluate Route B (a site re-derivation: the hit at the
producer's frame resume, the deferred shape) with its phase/schema consequences and quiescence
behavior; only if both fail with evidence, evaluate Route C (class removal) with the full cost
— membership 28/3/0/120, the policy and schema re-derivation, the 20-cell schedule, the bar
impact, and the five hypotheses' unobserved native omissions — and recommend one, recording a
ruled scope cut if removal.

**The impact.** For the recommended route: the policy TSV and `policy_digest`, the overlays and
`schema_digest`, the re-freeze, the packet re-authentication, the class stop rules, the 20-cell
schedule, and the deferred-publication class's own admission sequence after the policy decision.

**Hard constraints.** No weakening of `ordinary()`, `frame()`, `eligible()`,
`issued_queued_frame`, the ordinary guard's two byte positions, `verify_own_omission` or the
predicate classes; no new selector or broad environment bypass; no new `unsafe`; no
default-path cost; test-layer only; no hand-edited render or inventory; every sealed scene and
the c76-c91 evidence read-only; no source, epoch, campaign, acceptance or 744 action; stop at
the envelope boundary and report the exact remainder.

**Budget.** 3,000 s focused, carved from 743's focused remainder (4,750.828 s), leaving
1,750.828 s; no workspace and no epoch blocks; no new allocation; prototype builds only under
`.temp/brief-743/design2/` and metered. Report `rum-report/1` at `.temp/brief-743/report.json`
(append attempt), design sections, every verdict line quoted, final message
`scripts/report.py summary 743`.

## Continuation 92 (resume) — the profile-return class: the non-weakening path, design only

The first continuation-92 attempt ran on `grk` (session `4f1a765e-c69f-484a-8b64-3d157f6e0ac4`,
unit `brief-743-grk-20261121-141300-1163339`) from 14:13 and died at 14:47 with
`API error (status 402 Payment Required): Grok Build usage balance exhausted` after 89 turns
($7.475908); it wrote no report. The on-disk checkpoint is a starting point only, not evidence:
`.temp/brief-743/design2/STATUS.md` (the decision so far), `route_a_probe.rs`, `patch_route_a.py`
(written as its last action and never run — the scratch tree `design2/tree` is unpatched: no file
newer than 14:13 and no probe markers), and the scratch build cache `design2/target`. Re-verify
every checkpoint claim against the tree and the logs; do not trust it blindly. All orders and hard
constraints of the continuation-92 section above still apply unchanged: design only, `Scope: NONE`,
no source change, no live build, no epoch, no campaign, no admission, no 744, everything sealed
(c76-c91) read-only, scratch and prototype builds only under `.temp/brief-743/design2/`, and the
frozen/composed trees, `identity.json` and `copies.json` not written. Every verdict line quoted in
the report must be copied from a named file or a named log; the gate is run from code and reruns.

**Budget.** The continuation's carve is 2,960.344 s focused: the 960.344 s left of the original
3,000 s carve plus a fresh explicit 2,000 s allocation from the stage reserve under the user's
2026-11-19 ruling (“预算不够就追加吧”). Report the metered receipts as before; an unspent balance
returns.

## Gate c92 (2026-11-21) — Route A accepted, the P0515 declared-duration parity gap ordered repaired, the around_ok neighbor screen ordered

Continuation 92 (resume) ran on `agy` — the user's 2026-11-21 route decision (only cdx and agy
usable; the model now Gemini 4 Neon) after grk's 402 balance wall — under the User's explicit
fabrication caution (“之前gemini有过伪造测试报告的劣迹，所以验收时要格外小心”). Unit
`brief-743-agy-20261121-154248-1221520`, conversation `81a4a64d-3aad-4a0a-8a83-7fb16ca14f85`,
15:42:48–16:54, `status=SUCCESS`, no denials, 4,324.67 s, peak 8,000,614,400 B of 14 GiB (53.2%),
oom 0/0, 2 git-fence entries (agy's own `merge-base` probes, refused); report 233,662 B, STOP,
29 checks (21 PASS / 3 FLAGGED / 3 FAIL / 2 NA; the FAILs are the inherited c90 entries), 465
citations, 13 deviations, 10 disclosures, 4 questions, 15 sections. The full record is
`.temp/director/ruling-c92.md`; the ruling id is `Q92-route-a-accepted-p0515-parity-ordered`.

Accepted with my own reproductions: I re-ran `apply_route_a.py` on a fresh copy of the sealed c90
tree and its 9 outputs hash exactly to the worker's tree and receipt (the prototype tree is
script-produced); `proto/armed` differs from c90 in exactly the 9 Route A files plus the
`mutants.tsv`/`overlays.tsv` 133→135 rebinding (no adds, no removals); I rebuilt `suspension_oracle`
from a copied armed tree (tree hash `68941cca…`), generated my own authentication receipt and ran
the seven rows myself — all seven arm and hit once at their declared phases with the own predicate
(6× `Err(Native)` pub/nat true; P0410 `Err(Output)` pub=false nat=true), records=1, exit 0; the
ordinary lane reproduces byte-exact equality for the five and `snapshot.rs:30:9` panics with
`faults=9`/`253` for P0410/P0515; the disabled and real arms and the nohook control are false
(375/375 records with 0 hits; nohook 0 hits with `Ok(())`); and I assembled my own packet index
(my own armed logs and receipt, my disabled/real runs, the worker's trees/auth) on which the
unchanged importer prints 6 PASS plus `PortBlocked P0515 missing_evidence=DisabledDifference`
(reconciliation `caught=6 PortBlocked=145`). I independently diffed the P0515 real-vs-real and
real-vs-disabled tuples: exactly the 3 `AbsoluteTiming` duration-leaf differences and no
non-duration difference, and confirmed in the live code that `declared_duration` already lists
those 3 P0515 fixtures while `CM740_DURATION_PARITY` is P0492-only. The prototype `schema_digest`
is `56398c2decde77c915e68e77f5da84666ec18c7ca3209fcf7328b9ca1f4ddc3f` (135 rows), the live
`afb3110b…` (133 rows), and `mutants.tsv` column 8 carries the matching digest on all 151 rows;
scope, seals (`identity.json` `c5efa506…`, `copies.json` `71850b35…`), the 23 `around_ok` sites
(7 profile + 9 NativeFault bypass + 7 AST carrier) and the budget (1,584.935 s of 2,960.344 s
metered) all verify.

Findings (none blocking): the stale `run_importer_t8.py` assertion (7 vs the delivered 6/145 —
the report itself is honest); the hard-coded real-arm receipt numbers on the reuse path; the
nohook control's one modified-file warning (the candidate arms are clean); the guard-byte
phrasing (the offsets differ only by the render's 97/194 hook bytes); and the accepted
cross-class disclosure that 9 admitted NativeFault `around_ok` rows share the unreachable-hit
topology and 7 AST carrier sites need screening. Ruling: Route A is adopted (zero weakening;
Route B/C not invoked); the P0515 duration-parity view is repaired in Stage A (exactness:
raw records unchanged, sibling/native fields exact, only the declared duration leaf tolerated;
a 4th difference must still block); the neighbor screen is ordered before the freeze; the staged
plan is Stage A → B (freeze + 12-block epoch) → C (20-cell admission) → D (744/745/746). Budget:
c92 charged 1,584.935 s focused (1,375.409 s returned; 743 focused pool 3,126.237 s); Stage A is
carved at 4,000 s focused (the 743 pool plus a fresh 873.763 s from the stage reserve under the
2026-11-19 ruling) plus one 6,000 s workspace gate; stage reserve 34,159.7151416041 s; CM4
178,917.63604906694 s; the pair (45,5,false), C2, the 51 owners, the five source duties, S10 and
CM5 are unchanged.

## Continuation 93 (Stage A) — the composer emission, Route A, the P0515 parity repair and the neighbor screen

Read the c92 ruling and the five c92 sections of the brief, then the 741 `PLAN.md` §3 and
`ROW-EVIDENCE.md`. Implementation in the test layer, one gated step:

**Emit Route A canonically.** The composer/generator emits the Route A render (the pair-scoped
`DescendantEdge` primitive at the two suspension-frame sinks, the `descendant_gate_clause`
invariant, the call-site `around_ok_profile_return`/`OkAction::ReturnOk` early return on the cold
twins, and the two new non-hot dispatch overlays `builtin_adapters.rs`/`window.rs` with the three
drop-off fixes) with composer-generated canonical text, a byte-exact inverse and no hand edits;
`overlays.tsv` becomes 135 rows and `mutants.tsv` column 8 is mechanically rebound; the schema
digest is re-derived and the policy files' placement decided per Q3.

**Repair the P0515 declared-duration parity.** Extend `CM740_DURATION_PARITY`'s admitted comparison
view to P0515 using the existing `declared_duration` allowlist (the 3 fixtures), exactness as the
ruling states, and ship the falsifier that a 4th difference still produces `DisabledDifference`.

**Screen the neighbors.** The 9 NativeFault `around_ok`/`bypasses_profile_finish=1` rows and the 7
AST carrier sites are measured under the current render and the new primitive; the 9 rows' render
follows the outcome (the overlay set and schema digest follow; membership stays 28/3/7/113).

**Verify.** Standing inventories replay before the first acceptance suite; the 12 unit + E0603 +
live-binary + importer + nohook falsifiers and the peel run; canonical inverse; one workspace gate
in both modes; scope and seals. No epoch, no freeze, no admission, no 744.

**Budget.** 4,000 s focused (the 743 pool 3,126.237 s + 873.763 s from the stage reserve) + one
6,000 s workspace gate; everything else untouched; report `rum-report/1` appended; every verdict
line quoted.
