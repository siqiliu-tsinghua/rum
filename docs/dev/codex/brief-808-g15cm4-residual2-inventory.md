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
