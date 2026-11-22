# Brief 744 — CM4 repaired-epoch shakedown (Stage D, first part) — attempt 3

**Director-dispatched, one stage, one gate.** This executes the first part of Stage D on the
epoch accepted by rulings 759 (Stage A), 762 (Stage B) and 763 (Stage C), after the
attempt-2 STOP (`Q744-dispatch-reachability`) and the accepted design inventory brief 748.
It is measurement-only: prove the 18 already-caught own rows are now **live-reached** on the
repaired epoch — the repair's per-row falsifier, including P0416, P0813 and P2593, the three
rows proven producer-unreachable in attempt 2 — that the 100 frozen pair cells measure
`(45, 5, false)` per row per mode with zero admitted pair receipts, that the four structural
controls pass their frozen bar in both modes, and that the negative controls still fire, on
the accepted final epoch, before broad campaign spending. It completes the stage's 244
logical cells against the accepted 510-cell schedule and re-freezes the 246 for 745. It does
not run the full campaign (745) or acceptance (746), and it claims no new semantic catch.
`Scope: NONE` of production.

## Goal

When this brief is done: each of the 18 own rows has been freshly re-observed on the
repaired epoch in both logical modes with its own allowed predicate, its disabled parity and
its actual producer pause and ordered hit, and reproduces its accepted semantic signature;
the 100 frozen pair cells measure `(45, 5, false)` per row per mode with zero admitted pair
receipts; the four structural controls pass their frozen bar in both modes; the eight
observer, ten source-auth and four Q7 predicate negative controls and the generator
continuation/token negatives still fire; and the stage's 244 logical cells are completed and
reconciled against the accepted 510-cell schedule, leaving exactly 246 for 745. The pair
stays Open and unpromoted. No source repair in this stage: a changed known signature, a
disabled difference, a pair/control failure or a zero-hit own row — above all P0416, P0813
or P2593 hitting nothing again — stops for a ruling.

## Context pointers

- Read `AGENTS.md`; rulings `.temp/director/ruling-759.md` (Stage A), `ruling-760.md`,
  `ruling-761.md`, `ruling-762.md` (Stage B), `ruling-763.md` (Stage C); brief 743's
  continuation 96 blocks; brief 748 (the accepted 151-row reachability inventory — 95
  reachable / 56 unreachable, zero unknown — and its minimal repair design); the repair
  line's briefs 749 (Stage A render), 754 (Stage B epoch) and 755 (Stage C admission);
  `PLAN.md` §5 items 2-3; `.temp/brief-741/attempt-plan.json`, `DRAFT-COMMON.md` (the shared
  execution contract), `SIGNATURES.md` (the historical c7 signature authority — the 18
  caught rows' verbatim evidence lines; a reference to reproduce, never a receipt to
  reuse), `ROW-EVIDENCE.md` and `row-taxonomy.tsv`; brief 740's continuation rulings (the
  pair bar, the four Q7 falsifiers, the eight observer negatives, the ten source-auth
  negatives, the P2593/P0813/P0492 signatures).
- The accepted repaired epoch (read-only): `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  `7af7b461abfd724980735f06e8b173f2824381bbb6d71104b1678f2cd85bcd25`, binding build
  `809b6473…`, freeze `c724c9ab…`, the six-comparison manifest `c3bc2616…`; the twelve
  release labels (`default-real 45916ecc…`, `default-disabled 011d61bd…`, `default-armed
  6919c52a…`, `default-spot 45916ecc…`, `no-default-real b3b8b1e1…`, `no-default-disabled
  c3d2d5d2…`, `no-default-armed c736d3c4…`, `no-default-spot b3b8b1e1…`, the lane
  `b92b3ed2…`/`caaa5b6a…`/`c286c776…` and `lane-spot b92b3ed2…`) and the three trees `97eaeb45…`/`b41d1873…`/
  `bd8b7548…` at 5,810 files each; `.temp/brief-754/attempt-2/epoch/` for the thirteen
  build receipts and compiler logs, `cm754-blocks.jsonl` `ee5bb36c…`, the policy
  `3953aa9f…`, the render schema `5c3b02a3…`, `cm754-membership.json` `2a17e399…`
  (28 NativeFault + 3 DeferredPublication + 7 ProfileReturn + 113 ordinary),
  `cm754-selections.json` `8c061b27…` (the frozen 255 selections / 151 own),
  `cm754-program-matrix.json` `5c15fc12…` (9,060 programs), `cm754-own-tuples.json`
  `0202baf6…` (151 own), the trees' authentication tables and the P0543 live passes.
- The accepted Stage C publications are the schedule and admission authority, read by
  hash: `.temp/brief-755/full-schedule-ledger.json` `3e4532cf…` (the 510-cell schedule; the
  `stage=744` subsequence is this brief's exact order and membership: 18 own rows × 2, the
  100 pair selections × 2, the four structural controls × 2), `.temp/brief-755/class-admission.json`
  `6d556f4b…`, `.temp/brief-755/packet-index.json` `3e1644aa…`, the class importers
  `5baa3124…`-bound proofs and their logs, and the C10 calibration
  `.temp/brief-755/calibration/both-mode-packet.json` `f46ccfd3…` (`verified`,
  `class_cells=0`, `shakedown_credit=0`); the c96 table `bef35931…` and index `8bbd71a7…`
  remain the historical inventory, never a credit.
- Read programs, selections and the baseline from the sealed epoch, never from the live
  checkout. The live checkout is not an input to any run.
- **Attempt 1 stopped at the live-pin gate** (`Q744-live-pin-dispatch-document`,
  `.temp/director/ruling-744.md`): unit `brief-744-cdx-20261122-031104-35453` authenticated
  the whole entry, ran no epoch binary and charged 0 cells; its record under
  `.temp/brief-744/` is read-only and its report is preserved at
  `.temp/brief-744/attempt-1-report.json` (`31e52ec6…`).
- **Attempt 2 stopped on the unreachable carriers** (`Q744-dispatch-reachability`): unit
  `brief-744-cdx-20261122-032546-41457` (03:25:46–03:43, rc 0, 20 GiB peak 1.04 GiB,
  oom 0/0, 0 repo changes), default C10 passed, default P0416 recorded both producer
  pauses but emitted zero selector arms and zero source hits and its selected predicate
  changed from `Err(Output)` to `Ok(())`; family `cm3-carrier` blocked; 1 cell passed, 1
  blocked, 242 unattempted; 7 worker runs, 34.71 s; brief 748 then proved 56 of 151 rows
  producer-unreachable (including the three own rows) and designed this repair line, and
  briefs 749/754/755 rebuilt and accepted the epoch. Attempt 2's scene under
  `.temp/brief-744/attempt-2/` and the attempt-2 cumulative report at the root are read-only
  and must be preserved.

Entry authentication is by hash identity, not by report:

- The epoch identity and the twelve binaries re-hash to their entries; the build receipts
  and compiler logs re-hash; the three trees re-hash; the blocks, membership, selections,
  program matrix, own tuples, policy and render pins re-hash; `cm754-epoch-identity.json`
  reproduces the accepted status.
- The Stage C publications hash to the names above; the ten class members' both-mode
  packets are admitted; the C10 calibration is `verified` with `class_cells=0` and
  `shakedown_credit=0`; the 20 c96 cells are credited and spent in the schedule; the 744
  subsequence is 244 cells all `admission_credit=false` and
  `attempted_in_logical_schedule=false`, and the 745 subsequence is 246.
- The live checkout: record the live snapshot freshly at entry and at exit (the Stage C
  entry snapshot `72ed4d6a…` over 5,819 files is history, not this run's pin). The five
  declared mutable names are `TODO.md`,
  `docs/dev/codex/brief-623-g15c2-settle-unowned-census-rows.md`, this dispatch document
  `docs/dev/codex/brief-744-g15cm4-shakedown.md`, and the two generated bundles
  `tools/claude-mod/rum-director/.claude-plugin/types/claude-code/index.d.ts` and
  `tools/claude-mod/rum-director/.claude-plugin/types/claude-code-tools/index.d.ts`; an
  in-run change of any of the five is a disclosure, any other difference — any addition, any
  removal, or a modified path outside these five — is a STOP.
- Every earlier scene (`.temp/brief-743/`, `.temp/brief-749/`, `.temp/brief-754/`,
  `.temp/brief-755/`, the attempt-1 and attempt-2 records, c76–c99) is sealed and read-only;
  no file under them may be newer than this dispatch.
- A source or build identity change, a changed epoch binary, a re-hashed binary mismatch, a
  changed accepted schedule or admission, a sealed-file change or a live-pin difference is
  an immediate STOP before any run.

## Scope

- Files you MAY create: reports, logs, instruments, receipts, checkpoints and publication
  copies under `.temp/brief-744/attempt-3/` only, with the stage report at
  `.temp/brief-744/attempt-3/report.json` and the cumulative report at
  `.temp/brief-744/report.json`. Before writing the cumulative report, preserve the current
  root report (attempt 2's STOP) byte-exactly as
  `.temp/brief-744/attempt-2-cumulative.json`; the cumulative must retain attempt 1's STOP
  (`attempt-1-report.json`, `31e52ec6…`) and attempt 2's STOP verbatim. The attempt-1 record
  directly under `.temp/brief-744/`, the attempt-2 scene and the root report are read-only.
- Files you MUST NOT touch: any source file, inventory, program, schema, policy, binary,
  manifest, sealed scene or earlier brief's evidence; the live checkout; `Cargo.lock`;
  `.git`; `docs/**`; `TODO.md`; `PROGRESS.md`; `CHANGELOG.md`. No compile, no build, no
  `cargo` invocation that compiles; only the accepted epoch binaries execute, directly with
  their test filters. No new lane, no test rewrite, no selection adjustment, no maintenance
  replay. No new epoch label, no thirteenth block, no workspace suite.
- Only accepted unchanged binaries run, through bounded orchestration; every raw artifact is
  saved under `.temp/brief-744/attempt-3/` before parsing. Host scratch stays in `.temp/`,
  never `/tmp`.

## Deliverable

**The 244 logical cells, in the accepted schedule's order.** The order is the `stage=744`
subsequence of `full-schedule-ledger.json` `3e4532cf…` read in file order: per mode, default
first and then no-default, the 18 own rows in ascending row order —
`C10.prepared.profiled_consumer`, `P0416`, `P0492`, `P0508`, `P0527`, `P0543`, `P0544`,
`P0554`, `P0558`, `P0813`, `P0820`, `P0823`, `P0825`, `P0827`, `P1476`, `P1562`, `P1563`,
`P2593` — then the 100 pair cells (P0325 50, P0326 50, in frozen tuple order), then the four
structural controls `C1`, `G15A`, `G15B`, `Some`. Within each own row: the complete real
matrix, then the disabled matrix, then the selected armed case. Each row × mode is one
logical cell; a rerun never spends a second cell.

**Own rows (36 cells) — the repair's falsifier.** Each of the 18 rows must be caught live on
this epoch, in both modes, by its own allowed predicate, with disabled parity and the actual
producer pause and ordered hit: the declared `CM740_SELECTOR_ARMED` phase(s),
`CM740_SOURCE_HIT` with the declared phase and selector, the declared `CM740_OBSERVER_CAPTURE`
`queued` values and `before_quiescence`, `CM740_OWN_PREDICATE` with the current schedule
selection's program, the row's declared predicate and flags and the declared observer,
`CM740_OBSERVATION_END records=1 arm=armed`, and `test result: ok`. The disabled arm is
`observer_transport=false` with zero hits and exact real parity. The real matrix is profile-on
for every own worker except P0248's inherited real/disabled off-then-on, armed-off convention;
`RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=0` always. Produce a per-row and per-constituent
exact comparison against the accepted `SIGNATURES.md` lines and brief 740's accepted
receipts for the semantic fields (predicate, flags, observer, phase, carrier, class), and
against the current schedule's selection for the tuple and program: the repair re-derived the
render, so a tuple/program difference that the accepted Stage A/B/C records carry is expected
and is not a signature change; a change in the semantic fields is a STOP. The historical
receipts remain evidence of their own epoch and are never reused as this epoch's catch.
**P0416, P0813 and P2593 are the three own rows proven producer-unreachable in attempt 2;
each must now show its declared live arm and ordered hit in both modes. If any of them again
emits zero selector arms or zero source hits, or its predicate changes, the repair has
failed: name the row as a `PortBlocked`, block the family, and stop with the complete unspent
ledger. No row patch in this stage.** The shared P0543 lane's both-mode live passes are part
of the accepted Stage B identity and may supply its two cells by exact identity; otherwise
re-run them as scheduled. The C10 calibration packet may supply the C10 own cell's two modes
only by exact identity of epoch, builds, trees, program bytes, selection and environment; it
is never charged or credited twice; if any identity differs, re-run C10 as its own 2 cells
and say which identity moved.

**Pair cells (200 cells).** Execute the 100 frozen pair cells in each mode in the schedule's
order, each through its real matrix then disabled then selected armed, on the unchanged
binaries. Each row/mode must measure exactly `(45, 5, false)`: 45 cells caught by their own
allowed predicate, the five Error cells armed-without-hit, `false` meaning not promoted; zero
admitted pair receipts; the pair remains Open. Preserve all 13 held pair columns and the
parks; no promotion, no settlement, no encoding execution. Do not treat the pair Open as an
own failure and do not promote it.

**Structural controls (8 cells).** Run `C1`, `G15A`, `G15B` and `Some` in both modes and
require their frozen `structural-red-value-disabled` bar exactly as the accepted schedule
carries it; any change is a STOP.

**Negatives and falsifiers.** Preserve and run on this exact epoch: the eight
observer-predicate negatives (wrong row, unarmed selector, absent source hit, wrong phase,
absent producer pause, absent native difference, quiescence-only equal native, fault from
another row); the ten source-auth negatives per arm including the forged and wrong-arm type
probes; the four Q7 predicate falsifiers; the generator continuation/token negatives; the
unarmed, hit-before-arm, no-hit, identity, disabled-difference and raw-artifact falsifiers;
and the P2593 Error/no-hit cell as an armed-no-hit negative in the full matrices (an extra
negative, never a replacement schedule cell). A pure control already completed at
c94/c95/c96/755 may reuse its identical receipt only when its input identity is identical;
every live negative dependency must be re-observed on this epoch. Ordinary-forcing negatives
may force the guard on, never off.

**Outputs.** A per-row/per-constituent exact comparison table, the pair and structural
evidence, the negative-control receipts, the full raw packet index for this stage, the stage
ledger (244 completed + 246 remaining), the re-frozen remaining 745 schedule by hash (the
accepted schedule's `stage=745` subsequence), and the measured cumulative cost from which
745's forecast is computed. The report is `rum-report/1` at
`.temp/brief-744/attempt-3/report.json`; every verdict line quoted; `scripts/report.py check
744` clean without `--against-git`; the final message is only `python3 scripts/report.py
summary 744`.

## rum / Rust pitfalls (read twice)

- **P0813** must actually suspend in the resumed catch body and hit the resumed dispatch,
  rejecting `Output`; a completed-only hit or a hit at the wrong phase is a changed
  signature.
- **P2593** keeps the Q7 `Middle` / `prefix_and_tail.log` program `CM739.c4.P2593.5a421e33ca0e973cebc2`
  and both the immediate and resumed completion sites; its hit is
  `ActualOptionalSuccessBeforeConcreteTail` with `Err(Output)`; the old `Error` cell stays in
  the full matrices as an armed-no-hit negative.
- **P0492**'s first constituent may be `Ok`; the aggregate Native bar and the exclusive
  duration view remain, with `Err(Native)`, `public_output_exact=true`,
  `native_difference=true`.
- **P0410** keeps `Err(Output)` with `public_output_exact=false`.
- **EarlierGuard never counts as a catch.**
- Read the accepted c92/c93/c96/755 state, not the pre-Route-A "four failed rows" wording:
  all ten class rows are admitted, and the historical signatures for them are superseded
  only where the accepted records say so.

## Verification (mandatory — you run it; iterate to evidence)

- Zero new build blocks or independent allowance: every command debits the final-epoch
  envelope, whose balance is 14,193.545492596 s of the accepted 16,374 s; 0 s focused; 0 s
  workspace. The stage reserve moves 33,181.07062953712 → 18,987.52513694112 s for this
  carve; unspent time returns. Use `scripts/run-bounded.py` with a bound derived from
  measured shapes; stop before any command whose conservative bound would exceed the
  remaining balance. A projected overrun is a director ruling, never a scope cut: do not trim
  any matrix key, observation, constituent, lifecycle group, pair cell, structural or
  negative control, profiling mode, execution ordering or authentication to fit cost.
  Recompute forecasts from measured runs without deleting work.
- Every semantic red gets one same-binary one-thread reproduction before diagnosis,
  preserving both attempts. Preserve all failures and retry results under new IDs.
  Interference is fixed, not masked by serializing the default.
- No new source means no new workspace gate and no maintenance replay — this is deliberate
  verification reuse of the accepted tree, not deferred source acceptance; state exactly that
  in the report, linking the Stage A/B/C gates and the unchanged binaries.
- An ordinary unobservable all-armed/all-hit family, a disabled difference, a changed known
  signature, a pair/control change, an unsupported topology or a required accessor is an
  immediate STOP for a design ruling, not another row patch. A single unarmed/no-hit survivor
  is a named `PortBlocked`; it blocks its family and stops. No waiver, no reclassification,
  no promotion.

## Checkpointing — protect the scene

After each row or control group, write `.temp/brief-744/attempt-3/PROGRESS.md` with the
completed receipts and hashes, the measured cumulative epoch spend, the exact next case and
the tree state. Protect the scene: the sealed epochs, `.temp/brief-743/`, `.temp/brief-749/`,
`.temp/brief-754/` and `.temp/brief-755/` are read-only; all new files are unique. On
completion list all 264 distinct completed logical cells (20 Stage C + 244 here), the
remaining 246 for 745, and the future cost forecast.

## Hard rules

- `DRAFT-COMMON.md` in full. No git in any form; no network; no `unsafe`; no new dependency;
  no live production source, inventory predicate, public API, selection or coverage-bar
  change; no same-epoch identity relabel; no CM5.
- Pair `(45, 5, false)` stays Open at zero admitted receipts; C2 stays Open at 118 Residual +
  0 Lost; the 51 owners and five source duties stay Open; `CHANGELOG.md` and S10 stay
  blocked; the 13 held pair columns, the parks and G13's ownership of encoding are
  untouched.
- Unsupported evidence is a named `PortBlocked` and stops under the class/family rules; never
  weaken a predicate, never substitute a signature, never count a no-hit as a catch, never
  invent a waiver.
- You MUST paste the real verbatim evidence. If a run did not reach its bar, say so
  explicitly and stop — never fake or infer a result.

## Report format

`rum-report/1` under `.temp/brief-744/attempt-3/`, every attempt and verbatim verdict, all
cases and signatures, the negative receipts, the unresolved `PortBlocked` entries and the
Open ledger state. `changes` is empty (measurement-only). The cumulative
`.temp/brief-744/report.json` must retain attempt 1's STOP and attempt 2's STOP verbatim
(preserved at `attempt-1-report.json` `31e52ec6…` and
`attempt-2-cumulative.json` before the new cumulative is written) and carry this attempt.
Save every run's full output under `.temp/brief-744/attempt-3/` before parsing; cite lines
exactly. Before finishing run `python3 scripts/report.py check 744` (without
`--against-git`) and fix every PROBLEM; the final message is only
`python3 scripts/report.py summary 744`. STOP at the 244 completed cells and the re-frozen
745 schedule; 745 and 746 are not dispatched here.
