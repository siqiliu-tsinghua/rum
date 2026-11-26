# Brief 808 — G15-CM4 the second residual-repair inventory (design only)

Read briefs 799 (Amendment 4), 803, 804, 805, 807 and 807f, and rulings 832, 844, 850 and 851.
This is a **design-only** brief: `Scope: NONE` — no source edit, no build, no capture, no
inventory replay, no acceptance suite, no live-tree measurement. It surveys the retained
evidence, names the mechanism behind each stable residual item, drafts the candidate placement
set with isolation falsifiers, and designs the joint-build measurement; it drafts the follow-up
briefs in `TEMPLATE.md` shape. Scratch under `.temp/brief-808/`; never `/tmp`. No git, no tag,
no network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`.

## Why (the 807f STOP, accepted as correct; my defect #36)

Brief 807f (unit `brief-807-cdx-20261126-184820-3928546`) measured the fresh epoch under budget
v2.5. Everything qualified — nine calibrations on their first attempt, nine hardware draws at
100.00 % running with `n_tasks=2` and tree deltas 9.1–18.9 ms ≤ 40 ms, both preflights green —
and the fresh envelope did not close: `new − base = +23,182,542` native against the signed
control interval `[−1,002,341, +13,155,370]`, i.e. 10,027,172 above the upper edge. The
translated decomposition reconciles with zero remainder (representative +18,018,645; nine-pair
median +18,018,645, range `[10,679,546, 24,478,984]`), and the native−translated gap 5,163,897
is inside the control arm's own same-code gap (8,637,305 − 218,763 = 8,418,542). Ruling 851
retires the envelope as a pass bar (it is a determination) and keeps the measured default-path
cost as the repair target.

The residual's **stable** items (exact or near-exact across all nine new/base pairings) are the
inventory. The **draw-dependent** families must not be targeted: they are the program's own
per-process hash-seed/layout variation, present in base and control too.

| item | nine-pair value | known so far |
|---|---|---|
| eval_core | +8,005,829..+8,107,809 | `evaluate_inner_unstable'2` +7,970,633 at unchanged visits (1,046,968→1,046,892; +7.61/visit); the named `evaluate_inner_unstable` symbol is 2,009 → 1,940 instructions, so the cost is in the counted instance, not the named symbol; no callee visit change |
| pattern_matching | +6,273,782 exactly | `BindingSnapshot::capture` +33,261,184 new-only (426,421 visits × 78) against two callers −27,988,402; the capture source is byte-identical in both roles, so the inlining decision flipped in the armed build |
| publication checks | 1,464,035..1,464,199 | the 38 class-form gates (37 independent × 4 Ir + 1 reused × 3; 366,888 visits); class-form necessary; **user decision**, not a repair target |
| scoping / builtin_adapters | +2,100,570 / −2,045,000..−2,055,304 | one outlined helper pair at the module builtin; net ≈ +0.05M; no action unless a zero-cost way removes both |
| libc `0x1b1360` | +1,526,514..+2,102,029 | the bcmp leaf, +0.195 Ir/visit over 8,299,697 visits; data-dependent; carry, not a target |
| kernel_data | +784,262..+805,172 | small stable item; body unknown |
| eval_dispatch | +542,629..+554,970 | small stable item; body unknown |
| definitions | +348,183..+371,471 | small stable item; body unknown |
| rum_stdlib::strings | +240,168 exactly | `stringreplace-pattern-unguarded`'s 4 Ir surplus; tied to the priced site |
| drop glue | +187,950..+218,472 | small stable item; body unknown |
| alloc::raw_vec | −1,113,089..−1,128,038 | favorable; carry |

Recovering the two structural flips alone (`eval_core` + `pattern_matching` = 14.28M) brings a
typical draw to ≈ +3.7M, inside the control band; recovering either big one alone brings it to
+11.9M / +10.0M, also inside. That is the package's target: **as much of the stable structural
delta as the placements can prove**, with the 38 gates and the libc drift excluded.

## Pins (re-verify every one before any analysis; a stale pin is a STOP)

- `.temp/brief-807f/seal.json` `3742352f…`, `report.json` `e0aede90…` (check OK 1,637),
  `summary.txt`, `PROGRESS.md`.
- `.temp/brief-807f/evidence/fresh-body-map.json.gz` `e29874df…`, `residual-table.json`
  `1247875a…`, `whole-profiles.json` `9d9b40e0…`, `reconciliation.json` `22e26601…`,
  `fresh-residual.json`, `fresh-residual-table` (in the report), `label-dispositions.json`,
  `per-site-prices.json`, `falsifiers.json`, `settlement-readback.json`.
- The nine whole Callgrind draws under `.temp/brief-807f/captures/whole-{base,new,control}-{1,2,3}-try1/`
  (their `callgrind.out` hashes are in the report's `reconciliation.profiles`).
- The three 807a ELFs and release receipts: base `524c2bc2…`, new `2fa652ea…`, control
  `c166a2ff…`; the 807a seal.
- The 1,743-row / 93-family ledger: `.temp/brief-799/family-inventory.tsv`,
  `.temp/brief-804/evidence/row-mapping.tsv`, `family-mapping.tsv`, `price-ledger.tsv`.
- The 38-gate catalogue `.temp/brief-807f/evidence/new-publication-catalogue.tsv` and the 798
  site table `.temp/brief-798/evidence/amendment3/quarantine-diagnostic-whole-sites.tsv`.
- The 805 joint analysis `.temp/brief-805/evidence/joint-analysis.json` (the J0 union and
  nonpublication inputs) and the 807f fresh joint (report `reconciliation.joint`).
- The 795/796 instruments and the 804 decision ledger. The 807/807a–807f records are read-only.

## Stage A (300 s) — pins and input authentication

Re-hash every pin above, quote each `PASS/STOP` line, and re-read the 807f fresh residual table
and the nine draws' family pairings. State the current live-tree position (5,897 files, git
clean) and confirm nothing in this brief touches it.

## Stage B (1,600 s) — the mechanism inventory (zero-unknown)

For every stable item, produce a row: `item | body | visits(base,new) | Ir(base,new) |
per-visit delta | mechanism | exact source site (path:line) | evidence`. Use the retained
Callgrind body maps, the fresh nine-pair tables, and a bounded `objdump -Cd
--no-show-raw-insn` of the base and new ELFs (45 s bound each, serial) to compare each named
symbol and each counted instance; the base ELF's disassembly is not in the record and must be
produced here. Required resolutions:

1. **The evaluator instance.** Identify which counted body carries the +7,970,633 and which
   machine-code copy it is (the named symbol shrank by 69 instructions while the counted
   instance grew per visit; the two named symbols and the inlined copies must be separated by
   symbol range and debug-name). Name the exact instruction path that grew by ~7.6 per visit
   over 1,046,892 visits (e.g. an address-formation sequence, a spilled register, a lost or
   gained inline). If the mechanism cannot be named from the two ELFs plus the retained
   profiles, say exactly what is missing and why the retained data cannot resolve it.
2. **The capture inlining flip.** Establish from the callers' bodies (the two `match_compiled`
   methods, −15,770,227 and −11,217,175) and the new-only `BindingSnapshot::capture` body
   (+33,261,184) that the net +6,273,782 is the outlining overhead: the call/return, the
   register spill and the lost cross-inlining, at the same 426,421 visits. Confirm the capture
   source is byte-identical in both role images and that the armed callers' growth (the probe
   insertion) is what crossed the inliner threshold.
3. **The small stable items** (kernel_data, eval_dispatch, definitions, strings, drop glue):
   name each body and mechanism, same shape.
4. **The scoping/adapters pair**: confirm the single outlined helper and the net ≈ +0.05M.
5. **The libc leaf**: confirm the per-visit drift is data-dependent (the comparison-length
   distribution) and therefore not an attribute target.
6. **The draw-dependent families**: restate allocation, builtin_catalog and evaluation_cache by
   their nine-pair ranges and prove from the per-draw totals that their representative values
   are draw artifacts (the nine pairings' spread against the stable items' exactness).

## Stage C (700 s) — the candidate placements and the joint-build design

For each target item, give the placement **as code** (the exact attribute or restructure, in
the live source file's shape, applicable in both role renderings), the must-not-change list
(whether/when something happens, the observable rows, the 120-label functional equality, the
driven-path behaviour, the 38 gate visits), the expected closure (which rows and which edges
change), the predicted price mechanism and a band (not a point), and an **isolation falsifier**
— a check that must fire red if the placement fails to do what it claims (e.g. a symbol-absence
or call-edge count check that distinguishes "the capture is inlined again" from "the capture
disappeared"). Name the tempting shortcuts as forbidden: duplicated bodies, flattening a
representation, silent-drop helpers, lazy init, `cfg(debug_assertions)` keys, default-path
state, and any edit that changes whether or when something runs.

The joint-build protocol (the 805 method, restated):
- **J0** = the 807a builds and the fresh nine whole draws' per-family nine-pair medians;
  **J1** = the candidate builds on the armed and real role trees at one path, one codegen unit,
  offline (the 807a Stage B recipe), with the 120-label functional equality per arm.
- Whole Callgrind profiles per arm per variant (3 draws per arm), same workload and driver;
  the **deduplicated union** and the **closure-corrected price**: a price is the helper plus
  every registered caller plus every row whose incoming-edge count from the measured set
  changed; a row-local price is not a saving.
- The package price = (armed J0→J1 saving) − (real J0→J1 saving) on the deduplicated union,
  with `additive_claimed=false`; a positive real-role saving alone is not a repair.
- A price must exceed the nine-pair band of its currency; per-cell admission, never a global
  rule; every J1 cell that does not separate is reported, not dropped.
- The next epoch's bars per ruling 851: translated zero unexplained remainder; native delta =
  translated + gap within the control's same-code gap scale; the envelope reported as a
  determination. No acceptance is taken in 808/809.

## Stage D (400 s) — the drafts

Draft in `TEMPLATE.md` shape: (a) brief 809, the joint measurement and landing (the placements
as code, the must-not-change list, the falsifiers, the staged stops, the joint protocol, the
charge); (b) a brief 810 sketch, the render/install/re-freeze after the landing (the 806
pattern: re-render from the new live sources, the four role roots, the ten outputs, the R4
install and re-freeze, the standing replays; the acceptance suite if the gate requires it).
List every open question with its decision and evidence.

## Stage E (400 s) — verification, seal and report

The `rum-report/1` report at `.temp/brief-808/report.json` (`"brief": 808`, `"stage": "808"`):
the pins, the mechanism rows, the candidate placements with falsifiers and bands, the joint
design, the drafts, the charge. Seal the brief's artifacts, `python3 -I scripts/report.py check
.temp/brief-808/report.json`, the detached summary, and the exit audit (zero live changes, zero
`.git-attempts`, zero unexplained). `NO_SOURCE_EDIT`.

## Charge

3,400 s of the 13,752.999 s pool: A 300 / B 1,600 / C 700 / D 400 / E 400. Every stage inside its
box; a box that would overrun stops at the deadline with the exact missing measurement. The
10,352.999 s reserve stays unallocated.

## STOP rules

Any stale pin; any source edit, build, capture, inventory replay or acceptance suite; any
measurement of the live tree; a candidate without `path:line` evidence, a mechanism, a closure
and an isolation falsifier; any item left unknown; any claim of a native PC attribution; a
representative-pairing reading of a draw-dependent family; a price stated as a point instead of
a band; a placement that changes whether or when something happens; any git attempt; an
unexplained live change. Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354,
P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no
tag; no network; no new dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git.

## Director gate (2026-11-26, gate c200)

The attempt (cdx, unit `brief-808-cdx-20261126-194014-3953965`, 19:40:14–19:51, 145,854
tokens, peak 255,578,112 B, oom 0, oom_kill 0, rc 0) **stopped at Stage A**: the 300 s box
elapsed during evidence discovery with zero pins authenticated — `STAGE A STOP
elapsed_s=526.311 box_s=300 overrun_s=226.311 authenticated_pins=0 required=all` (first
clock at +42.7 s, deadline 409494.939, stop observed 409721.250). `pin-status.json` lists
`authenticated: []` with 69 pending paths and 6 pending logical checks; the 61 exec calls
are reads only and no hash command, build, capture, disassembly, inventory replay or
acceptance suite ran. Report `d9c88337…`, `report.py check` OK 6 citations, seal 11
artifacts (every hash equal, detached summary equal), exit audit zero worker live changes
and zero git attempts; charge 655.740 s (A 526.311/300, E 129.429/400); pool **13,097.259 s**
after; the 10,352.999 s reserve unallocated. My live census reads 5,898 files: the 807f exit
5,897 plus exactly the three paths of gate c199's commit `5817b7bf` (`TODO.md` and
`brief-807-g15cm4-fresh-epoch.md` modified, this brief added); git clean. Ruling 852 accepts
the STOP and names my defect #37: Stage A packed the mandatory reading of ten documents
(~181 KB) and the 69-pin re-hash into one 300 s box with the reading list first, and the
brief stated a live position (5,897) that my own commit had already made stale. The stopped
record is archived at `.temp/director/808-stop-1/`.

## Amendment 1 (ruling 852, gate c200)

Everything above stands except the stage carve, the input order and the live baseline. The
reading list and the pin set are unchanged in content and now carry exact paths; the stage
plan is **3,700 s of the 13,097.259 s pool** (9,397.259 s stays unallocated; no top-up):
A0 250 / A 350 / B 1,600 / C 700 / D 400 / E 400.

- **Stage A0 (250 s) — the pins first.** Before any reading beyond this brief, re-hash every
  pin listed under "Pins" in one scripted pass and quote a `PASS`/`STOP` line per pin; a
  stale pin is a STOP. The set is small — the nine whole-draw Callgrind outputs are ~720 KB
  each and the three 807a ELFs are 13,263,120 / 18,689,432 / 13,263,240 B, so a full pass is
  seconds. Also record the entry live census (the 806e exclusions: `.temp`, `.git`, `target`,
  `.agents`, `.codex`, `.aws`) and compare its path set with
  `.temp/brief-807f/evidence/live-exit.json`: the expected count is **5,898** and the expected
  extra paths are exactly `TODO.md`, `docs/dev/codex/brief-807-g15cm4-fresh-epoch.md` and this
  brief; any other path is a STOP.
- **Stage A (350 s) — the reading, with exact paths.** Briefs
  `docs/dev/codex/brief-799-g15cm4-armed-residual-family-inventory.md`,
  `docs/dev/codex/brief-803-g15cm4-real-batteries-and-outline-completion.md`,
  `docs/dev/codex/brief-804-g15cm4-repair-plan-d1.md`,
  `docs/dev/codex/brief-805-g15cm4-joint-measurement-and-implementation.md`,
  `docs/dev/codex/brief-807-g15cm4-fresh-epoch.md`,
  `docs/dev/codex/brief-807f-g15cm4-fresh-epoch-v25.md`; rulings
  `.temp/director/ruling-832.md`, `-844.md`, `-850.md`, `-851.md`. Quote the lines each stage
  depends on; re-read the 807f fresh residual table and the nine draws' family pairings.
- **Stages B, C, D, E (1,600 / 700 / 400 / 400 s).** As written above.
- **Clock discipline.** Append `CLOCK stage=<A0|A|B|C|D|E> t=<monotonic_ns>` to
  `PROGRESS.md` at every stage entry and exit. A stage that reaches its deadline stops there
  with the exact missing measurement — the previous attempt discovered its deadline 226.311 s
  late; that must not recur.
- **Outputs.** The stopped attempt's record is the archived receipt at
  `.temp/director/808-stop-1/`; this Amendment writes fresh outputs under `.temp/brief-808/`
  and reports as `rum-report/1` with `"brief": 808`, `"stage": "Amendment 1"`.

Everything else — the design-only scope (`Scope: NONE`), the zero-unknown mechanism
inventory, the candidate placements with isolation falsifiers and bands, the J0/J1 joint
design, the drafts of 809 and 810, the STOP rules and the seal/check/exit audit — stands
unchanged.

## Director gate (2026-11-26, gate c201)

The attempt (cdx, unit `brief-808-cdx-20261126-195656-3963728`, 19:56:56–20:06, 61,791
tokens, peak 331,476,992 B, oom 0, oom_kill 0, rc 0) **stopped at Stage A0**: the 250 s box
was exceeded by 19.570 s of clock time (A0 clock entry 410217.344, exit 410486.913, elapsed
269.570; charged from dispatch 289.974) — `STAGE A0 STOP elapsed_between_CLOCK_s=269.569775
box_s=250 overrun_s=19.569775 charged_since_dispatch_s=289.974391 physical_pins=66
stale_pins=0`. The substance is delivered: **all 66 pins PASS** (I re-ran every hash: 0
missing, 0 bad, each equal to the 807f `pins.json` reference), the census PASS (5,898 =
5,897 + exactly the three paths of gate c200's commit; the exit census equals the entry),
and the logical readbacks PASS except the worker's **extra** 38-gate catalogue predicate
(its 37-independent + 1-reused reading is correct, and I re-derived the 37 nonzero entries /
363,371 visits from the TSV; the two first-pass STOPs were the verifier's own expectation
errors — the residual table augments the joint object, the 799 projection omits
`body_records` — corrected to PASS in the final pass and disclosed). Report `d85a0e9e…`,
check OK 163 citations, seal 24 artifacts (all hashes equal; `pin-verdicts.txt`
`8611af7c…`, `live-exit.json` `659d0385…`), exit audit zero live changes and zero git
attempts; charge 540.604 s (A0 289.974 from dispatch — 269.570 clocked plus 20.405
pre-clock; E 250.630; A–D 0); pool **12,556.655 s**. Ruling 853 accepts the STOP and names
my defect #38: the 250 s box was sized to the physical work, leaving no slack for the Pins
section's own logical readbacks, and the wording did not separate the required readbacks
from extra predicates; the worker's keeping the box open past its deadline is a recurring
discipline finding. The stopped record is archived at `.temp/director/808-stop-2/`.

## Amendment 2 (ruling 853, gate c201)

A0 is **closed by hash identity**: re-hash the 66 pins and require equality with the stop-2
verdicts (`.temp/director/808-stop-2/pin-verdicts.txt` `8611af7c…`; equality
authenticates, any difference is a STOP); compare the entry census with the stop-2 exit
census (`.temp/director/808-stop-2/live-exit.json` `659d0385…`), allowing exactly `TODO.md`
and this brief to differ (gate c201's commit) and requiring 5,898 paths with no addition or
removal; the six logical readbacks and the catalogue count (37 independent + 1 reused = 38)
are credited from that record and are not re-run. The stage plan is **3,750 s of the
12,556.655 s pool** (8,806.655 s unallocated; no top-up): **A0 150 / A 500 / B 1,600 /
C 700 / D 400 / E 400**. Each stage's box is measured from its own `CLOCK` entry; the
charge stays continuous from dispatch and the pre-clock interval is charged to the attempt.
A stage that reaches its deadline stops there with the exact missing measurement; no stage
may be kept open for extra work. Stage A is the reading with the exact paths already listed
in Amendment 1 (plus the stop-2 `final-binding-verdicts.txt` as context); Stages B, C, D
and E are unchanged. Everything else stands.

## Director gate (2026-11-26, gate c202)

The attempt (cdx, unit `brief-808-cdx-20261126-200944-3971034`, 20:09:44–20:27, 160,752
tokens, peak 625,360,896 B, oom 0, oom_kill 0, rc 0) **stopped at Stage B** under the
zero-unknown rule — `STAGE B STOP zero_unknown=false earliest_blocking_requirement=B2
placements=0 drafts=0 C=unentered D=unentered` — with the two missing measurements named
(a same-pass pre-inline decision record or controlled role-equivalent builds for the
matcher; a caller-tagged comparison trace for the libc leaf). A0 passed by hash identity:
66/66 pins identical to the stop-2 verdicts `8611af7c…` (I re-ran every hash: 0 missing,
0 bad, all equal), the entry census 5,898 with exactly `TODO.md` and this brief modified
against the stop-2 exit, and the exit census equal (I re-walked the same 5,898-path set
and re-hashed the eight non-tracked paths equal). The mechanism inventory is delivered
and credited: every row balance has remainder 0; the family ranges reproduce the sealed
807f record exactly (eval_core [8,005,829, 8,107,809], pattern_matching 6,273,782,
kernel_data [784,262, 805,172], eval_dispatch [542,629, 554,970], definitions
[348,183, 371,471], strings 240,168, drop glue [187,950, 218,472], scoping 2,100,570,
builtin_adapters [−2,055,304, −2,046,448], raw_vec [−1,128,038, −1,113,089], publication
[1,464,035, 1,464,199], libc family [1,526,514, 2,102,029]); the evaluator's bill is
exact (the flag address held in r14 in base vs re-formed by RIP-relative LEA in armed at
the same logical boundaries, `profiling/timing.rs:957`; row 1116 +7,970,633 with
remainder 0; row 1115 +82,520); the matcher's ABI bill is exact (6,273,782 with remainder
0; source byte-identical; the cold static caller 0x2727f7 has zero calls); the libc
leaf's own range [1,346,267, 1,893,404] sits inside its family's. Two labels are
corrected: the evaluator's 2,009/1,940 were seven-byte display lines (decoded: 1,636 /
1,599), and the libc 1.53..2.10M range belongs to the family, not the leaf. Report
`95549c36…`, check OK 529 citations, seal 56 artifacts (all hashes equal, detached
summary equal), exit audit zero live changes and zero git attempts, two bounded serial
objdumps (45 s each), `NO_SOURCE_EDIT`; charge 1,020.895 s (A0 35.737/150, A 77.683/500,
B 615.700/1,600, E 274.109/400; C/D 0); pool **11,535.760 s**. Ruling 854 accepts the
STOP and names my defect #39: the brief's items 2 and 5 demanded causal confirmations the
retained instruments cannot carry (both ELFs have no remark or IR sidecar; Callgrind is
`positions=instr,line`/`events=Ir`) while the STOP rule made those gaps fatal — the
zero-unknown rule is a disposition-plus-named-missing-measurement bar, not a
causal-proof bar. The stopped record is archived at `.temp/director/808-stop-3/`.

## Amendment 3 (ruling 854, gate c202)

A0 is closed by hash identity: re-hash the 66 pins against
`.temp/director/808-stop-3/pin-verdicts.txt` `5c69f175…` (equality authenticates, any
difference is a STOP); compare the entry census with the stop-3 exit
`.temp/director/808-stop-3/live-exit.json` `cf68bd85…`, allowing exactly `TODO.md` and
this brief to differ and requiring 5,898 paths with no addition or removal. The two ELF
disassemblies (`base.dis` `bb4c9a45…`/90,754,260 B, `new.dis` `d372ac1d…`/129,665,438 B)
and every mechanism table are read-only evidence under the archive and are not
re-produced. **Stage B is closed** with the dispositions: eval_core and pattern_matching
are repair targets; the matcher's causal test becomes a declared J1 cell in the 809
design (a role-equivalent build independently varying the second cold user and the
instrumentation, identical functional output, closed edges); the libc leaf is a carry
(external leaf, no rum source, operands unrecordable with the retained instruments); the
small families and the scoping/adapters pair are targets where a placement controls the
mechanism and carries otherwise; raw_vec is favorable; the publication gates and the
fastpath stay with the user. The stage plan is **1,900 s of the 11,535.760 s pool**:
**A0 100 / C 1,000 / D 400 / E 400** (the 8,806.655 s reserve unallocated; no top-up).
Stage C gives, for each repair-target item, the placement as code in the live source's
shape with the must-not-change list, expected closure, price mechanism and band, and an
isolation falsifier — or a justified carry naming what a controlled build would vary;
the matcher prefers `#[inline]` over `#[inline(always)]` and its falsifier distinguishes
"inlined again" from "disappeared"; the evaluator placement addresses the flag-address
allocation, not the flag's semantics, with no default-path state in a hot frame; the
J0/J1 protocol stands as written with the B2 cause variant as a declared cell. Stage D
drafts 809 and 810; Stage E verifies, seals and reports. Each stage's box is measured
from its own `CLOCK` entry; a stage that reaches its deadline stops there with the exact
missing measurement. Everything else stands.

## Director gate (2026-11-26, gate c203)

The Amendment 3 attempt (cdx, unit `brief-808-cdx-20261126-203522-3985262`, 20:35:22–21:00,
345,296 tokens, peak 245,211,136 B, oom 0, oom_kill 0, rc 0) is **DONE**. A0 passed by hash
identity: 66/66 pins equal the stop-3 verdicts `5c69f175…` (I re-ran every hash: 0 missing, 0
bad, no additions or removals), the entry census 5,898 with exactly `TODO.md` and this brief
modified against the stop-3 exit `cf68bd85…`, the exit census equal (my own walk reproduces the
same 5,898-path set). The design is delivered and checked: two placements as literal one-site
substitutions with all six role preimages hash-verified (`timing.rs` `2e3d38e2…` in live, real
and armed; `core.rs` `53fe924b…` in live and armed, `7f2682ec…` in real) and the live files read
at the exact lines (the closure at `timing.rs:156` under the existing `#[inline]`, the flag load
at :957; `core.rs:42`, :63, :1085); the E mechanism (the flag-address register allocation) with
band 0..8,107,809 and the four-row seed 8,034,374..8,090,378 (b0=28,031, which I reproduced from
the sealed body map); the M hint with band −376,710..+6,822,736 and the exact seed 6,273,782;
`invariants.json`'s six must-preserve blocks; the 13 dispositions (ten carries with mechanisms,
bands and controlled follow-ups; publication and fastpath user-owned; three draw families
carried); the J0/J1 estimator (J0 frozen on the three 807a ELFs and nine 807f draws; 18 pricing
profiles; the deduplicated union and closure rule; the 805 subtraction; the all-81 price
combinations; the b0+b1 bar and same-code controls); the B2 four-cell causal matrix with its
scratch-only/no-price/degradation clauses; and the drafts of 809 and 810 with their stage plans
and charges. Report `0f00aead…`, check OK 452 citations, seal 30 artifacts (I re-hashed all 30:
0 missing, 0 bad; report sha equal; detached summary equal), exit audit zero live changes and
zero `.git-attempts`, zero builds, captures, disassemblies, inventory replays or acceptance
suites, `NO_SOURCE_EDIT`, the 808 archive untouched; charge 1,475.596 s (A0 30.776/100,
C 867.659/1,000, D 232.010/400, E 325.157/400; every stage inside its box); pool
**10,060.155 s**. Ruling 855 accepts the brief DONE with two gate notes: the M hint was measured
inert in brief 802's `1383-inline` outline cell on the pre-805 tree (the J1 cell re-tests it on
the 807a tree, and an inert result must be reported as an unpriced cell and the matcher residual
carried), and the E placement is a codegen hypothesis whose falsifier's address-work column and
negative control are the bar. Brief 809 is written from the draft at 6,300 s (A 200 / B 1,800 /
C 2,500 / D 1,500 / E 300), with the two placements frozen, the B2 matrix included scratch-only,
the selection rule predeclared, and the user's two class-form decisions untouched.
