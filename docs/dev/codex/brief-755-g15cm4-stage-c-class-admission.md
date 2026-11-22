# Brief 755 — G15-CM4 Stage C: the twenty-cell class admission on the accepted Stage B epoch

## Goal

When this brief is done, in `.temp/brief-755/` only: the ten admission members — the three
DeferredPublication rows P0267/P0312/P0323 and the seven ProfileReturn rows
P0256/P0259/P0266/P0314/P0315/P0410/P0515 — are admitted **all-or-nothing** on the accepted
Stage B epoch across both modes: exactly 20 own cells (member × mode), each with its own
both-mode observation packet from the epoch's label binaries, the unchanged importer
reconciliation `required=151 caught=10 PortBlocked=141` per mode, the C10 calibration recorded
but never credited, and the complete per-class admission table, 20-cell ledger, both-mode
packet index and entry/closing authentication. The Stage B acceptance identity is the input;
nothing is rebuilt, relabelled or edited. No source edit, no workspace suite, no 744/745/746,
no CM5.

This is the c96 admission protocol re-established on the new epoch: the c96 credit belongs to
the superseded epoch (the repair line replaced the render and every label binary), so the 20
cells must be re-earned on the accepted Stage B binaries and trees. The c96 admission table is
the complete, zero-unknown inventory and is reused by hash identity; no member is added,
removed or re-signatured here.

## Context pointers

- Read `AGENTS.md`; brief 623's "Campaign invariants" section; rulings
  `.temp/director/ruling-759.md` (Stage A), `ruling-760.md`, `ruling-761.md` and
  `ruling-762.md` (Stage B acceptance); brief 743's Stage C continuation 96 and its attempt 2
  (the accepted 20/20 admission, the importer reconciliation correction
  `Q96-class-index-reconciliation`, the C10 calibration and the stop rules).
- The accepted Stage B scene (read-only): `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  (`7af7b461abfd724980735f06e8b173f2824381bbb6d71104b1678f2cd85bcd25`), binding build identity
  `809b6473…`, freeze identity `c724c9ab…`, the six-comparison manifest
  `.temp/brief-754/attempt-3/epoch/six-comparisons.json` (`c3bc2616…`) and every case record
  it binds; `.temp/brief-754/attempt-2/epoch/` for the trees, the twelve `*-oracle` binaries,
  `cm754-blocks.jsonl` (`ee5bb36c…`), `cm754-membership.json` (`2a17e399…`),
  `cm754-selections.json` (`8c061b27…`), `cm754-program-matrix.json` (`5c15fc12…`),
  `cm754-own-tuples.json` (`0202baf6…`), `cm754-warnings.json` (`755ace09…`); the accepted
  Stage A render (`.temp/brief-749/attempt-7/renders/one/overlays.tsv` `5c3b02a3…`,
  `mutants.tsv` `e8b3b6e4…`, `observer-policy.tsv` `3953aa9f…`).
- The accepted c96 Stage C instruments and evidence (read-only): `.temp/brief-743/stage-c/`
  — `admission.py`, `authenticate.py`, `import_packets.py`, `reconcile.py`,
  `definitions.py`, `presentation.py`; the admission table `class-admission.json`
  (`bef35931…`, ten admitted members with both-mode packets and both importer verdicts); the
  510-cell schedule `full-schedule-ledger.json` (`6edd4d37…`); the class index
  `indices/class.tsv` (`8bbd71a7…`); `packets/`, `calibration/`, `logs/`, `receipts/`; and the
  c96 gate record in brief 743. Copy what is needed under `.temp/brief-755/`; never edit a
  sealed instrument or record.
- The 741 `PLAN.md` §3 and `.temp/brief-741/attempt-plan.json` for the observation-mode policy.
- Everything sealed (c76–c99) and every earlier scene stay read-only.

## Scope

- MAY create/modify: everything under `.temp/brief-755/`, with the cumulative report at
  `.temp/brief-755/report.json` and its ledger at `.temp/brief-755/ledger.json`. Every other
  scene, including `.temp/brief-754/attempt-3/` and `.temp/brief-743/stage-c/`, is read-only.
- MAY read: the live source tree, the accepted scenes, the sealed scenes, the earlier
  `.temp/brief-*` records.
- MUST NOT: edit the live source tree (0 tracked changes); rebuild any binary; add an epoch
  label or a thirteenth block; run a workspace suite; use the lane blocks, the drive trees or
  the admission-only default-disabled build; re-run a member packet that was already spent;
  admit a member whose own packet is invalid; edit any live inventory, policy, schema,
  manifest or selection; commit or use git; touch the network; add a dependency or `unsafe`.
  The membership stays 28 NativeFault / 3 DeferredPublication / 7 ProfileReturn / 113
  Ordinary; the pair stays `(45, 5, false)` Open at zero admitted receipts.

## Deliverable

1. **Entry authentication by hash identity, not by report.** Authenticate the Stage B identity
   `7af7b461…`, the build `809b6473…`, the freeze `c724c9ab…`, the six-comparison manifest
   `c3bc2616…`, the twelve label binaries (default real `45916ecc…`, disabled `011d61bd…`,
   armed `6919c52a…`; no-default real `b3b8b1e1…`, disabled `c3d2d5d2…`, armed `c736d3c4…`;
   the lane binaries and the three forced spot identities as the identity records), the three
   trees (`97eaeb45…` source, `b41d1873…` disabled, `bd8b7548…` armed, 5,810 files each), the
   membership `28/3/7/113`, the 255-per-mode selections with 151 own rows, the render pins,
   and the c96 admission table and class index. Record the live snapshot freshly at entry and
   at exit (the Stage B completion snapshot `2313045a…` over 5,818 files is history, not this
   run's pin); the five declared mutable names are `TODO.md`, brief 623, brief 755 and the two
   generated mod bundles; an in-run change of any of the five is a disclosure, any other
   difference a STOP.
2. **The new class index.** Re-derive the both-mode class index for this epoch from the sealed
   real tree — the ten members' programs and selections, each bound to its new selection
   record and its row identity — under `.temp/brief-755/indices/`. A member whose identity,
   program or selection cannot bind stops with the exact fact; no substitution, no inherited
   index cell without re-binding.
3. **The admission.** Run the ten members in ascending row order (P0256, P0259, P0266, P0267,
   P0312, P0314, P0315, P0323, P0410, P0515), default then no-default, real then disabled then
   armed, on the epoch's label binaries against the sealed trees. Each member × mode is one of
   exactly 20 logical own cells; C10 is calibration, not a cell; no cell is run twice. A member
   is admitted only when its own both-mode observation packet is valid under the accepted c90–c96
   signatures: the seven ProfileReturn rows hit exactly once at their declared phases — six
   `Err(Native)` with public=true native=true and P0410 `Err(Output)` with public=false
   native=true — with the ordinary lane equal for P0256/P0259/P0266/P0314/P0315 and
   `LogicalQuiescence` (`snapshot.rs:30:9`, faults=9 for P0410 and 253 for P0515); the three
   DeferredPublication rows arm, hit at
   `OwnDeferredDefinitionPublicationAtActualProfileQueueJoin` and show `Err(Native)` with
   `native_difference=true` and equal public output, their ordinary run a logical-quiescence
   negative; the armed arm carries `CM740_SELECTOR_ARMED`, `CM740_SOURCE_HIT` and its own
   `CM740_OWN_PREDICATE`; the disabled arm is `observer_transport=false` with zero hits and the
   real parity exact; every raw artifact authenticates; and the unchanged `verify_own_omission`
   holds with the aggregate constituent semantics. No receipt from synthetic data, no panic
   catcher, no fault clearing, no catch-and-continue.
4. **The importer reconciliation.** Run the unchanged importer
   `oracle::mutations::receipts::cm_own_mutant_receipts_exact` from the epoch's no-default-real
   binary (`b3b8b1e1…`) on the new both-mode class index; expect, for each mode,
   `required=151 caught=10 PortBlocked=141 settlements=0` with the ten PASS rows
   (`real_keys` per the accepted table, `hits=1`, `disabled_record_exact=true`,
   `raw_builds_authenticated=true`), and the accepted partial-index exit 101 as the bar. One
   importer execution per mode is a receipt, not a cell; do not re-run any member packet.
5. **The C10 calibration.** Run `C10.prepared.profiled_consumer` (NativeFault, program
   `C12.consumer.Middle.phase_observation.trace`) through the same protocol in both modes and
   three arms on the same six binaries; retain its complete packet under
   `.temp/brief-755/calibration/` with the binary, tree, receipts, program, selection tuple,
   environment manifest, both-mode raw logs and their hashes, the observation lines and the
   importer verdict. It is recorded, never credited; the 744 restart may reuse it only if every
   identity, program and environment check matches, and never charges or credits it twice.
6. **The evidence and the ledger.** The per-class admission table (class, members, both-mode
   packets, admission verdict, importer verdict), the both-mode packet index and each mode's
   reconciliation, the ordinary-run negatives (the declared five P0267/P0312/P0323/P0410/P0515
   verbatim; any other negative is a class finding), the 20-cell ledger with a proof link per
   cell, the 510-cell schedule (20 admitted, 490 unspent), the C10 calibration, the
   entry-and-closing authentication, and every verdict line quoted. The report is
   `rum-report/1` at `.temp/brief-755/report.json`; before finishing run
   `python3 scripts/report.py check 755` and fix every PROBLEM; the final message is only
   `python3 scripts/report.py summary 755`.

## Hard rules

- Never weaken a predicate, reclassify a row, exempt a dead row, move a hit to a favourable
  branch, count a text or byte identity as a catch, or admit a cell from synthetic data or
  from another mode's packet.
- A failing member is a named PortBlocked with the class finding and a one-thread reproduction
  on the same binary; the admission stops with the complete unspent ledger; 744 is not
  dispatched. The same immediate stop applies to an all-armed/all-hit family that cannot
  return observations, an ordinary unobservable family beyond the declared three, an
  unsupported topology, a real/disabled difference outside P0515's declared three duration
  leaves (a fourth difference must still reject), a changed known signature, a source or build
  identity change, a sealed-file change, a live-pin bar failure or the focused boundary.
- No row repair, no class cherry-picking, no single-row reclassification, no withdrawal of an
  admitted cell. Missing capture does not consume a family no-progress strike.
- `.temp` only; no git; no network; no new dependency; no new `unsafe`; no production change;
  no default-path cost.
- The standing state is unchanged: bar 571/4,422, 37/134, 4/133, 0/26; the pair
  `(45, 5, false)` Open at zero admitted receipts; C2 118 Residual + 0 Lost; the 51 owners and
  five source duties Open; membership 28/3/7/113; `CHANGELOG.md` and S10 blocked; CM5
  unauthorized; everything sealed (c76–c99) sealed; 745/746 held.

## Verification

- Ten admitted members, 20 own cells, both modes, all-or-nothing; the ten PASS importer rows
  per mode with `required=151 caught=10 PortBlocked=141 settlements=0`; the accepted
  partial-index exit 101 recorded.
- The C10 calibration verified with `class_cells=0` and `shakedown_credit=0`.
- Entry and closing identity checks from the bytes: the Stage B identity, the build, the
  freeze, the twelve binaries, the three trees, the membership and the selections; the live
  snapshot at entry and exit; no write outside the scene; the sealed scenes untouched; no git
  or network receipt.
- Budget: **1,200 s focused** (a fresh carve from the stage reserve, 34,031.40995267212 →
  32,831.40995267212 s), **0 s workspace, 0 s epoch**; the 749 pools are exhausted and closed;
  unspent returns; a projected overrun stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-755/report.json` (cumulative with `.temp/brief-755/ledger.json`),
`changes` empty (every write is under `.temp/brief-755/`); sections for the entry, the class
index, the twenty cells, the importer, the calibration, the ledger and the scope; cite lines
exactly. STOP at Stage C: no 744/745/746, no CM5.
