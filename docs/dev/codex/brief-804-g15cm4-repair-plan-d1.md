# Brief 804 — G15-CM4 D.1: the repair plan from the closure-corrected price table (design-only)

Design-only, `Scope: NONE — no source change`. Dispatched after ruling 836 (gate c184). The deliverable
is the plan and the drafted follow-up briefs; no build, no capture, no source edit, no acceptance
suite, no epoch. Every claim carries `path:line` or an exact artifact citation. All writes under
`.temp/brief-804/`; the live tree is read-only.

## Why this brief

Brief 803 sealed DONE: both structural controls matched, the four real evaluator cells ran (fastpath
**44,581,644**, coldout **18,848,556**, foldaddr and hoist null at **0**), and the five outline cells
delivered. Ruling 836 accepted the real cells and applied a new binding rule — the **closure** — which
withdrew three outline prices as re-billing artifacts: **1383-always** (21,790,957, accepted at c183),
**1086-always** (226,800) and **1199-inline** (210,345). Seven closure-verified prices survive
(**14,851,147** gross, never summed). The evaluator differential is now computable: fastpath
**41,712** (inside the combined band — not a residual repair) and coldout **3,735,504** (a repair).
The campaign's next step is the written repair plan: what to implement, what it buys against the
+37,644,521 translated / +38,126,375 native residual, what remains uncovered, and how the repair,
its render/install/freeze and the fresh epoch will be gated.

## Goal

Produce the **D.1 repair plan** with zero unknowns:

1. the closure-corrected price ledger for every evaluator and outline cell, each with its status
   (spendable / withdrawn / refused / unpriced / null) and its role basis;
2. the mapping of every spendable price onto the 1,743-row classification ledger and the 93-family
   inventory, with the addressable and uncovered residual mass stated exactly and every row and
   family dispositioned (the zero-unknown statement);
3. the repair package and the **joint-build measurement design** that is the authoritative price
   (no sums), including the shared caller 1186 accounted once and the package differential;
4. the uncovered remainder's disposition by family — repairable (with the missing measurement named)
   or class-form-necessary (returned to the user, never silently accepted);
5. the drafts of the follow-up briefs (the joint measurement + implementation; the
   render/install/freeze; the fresh epoch with the replaced qualification rule, Stage C/C10 and the
   restart), each in this template's shape with `path:line` evidence.

## The binding rules of the plan

- **Closure.** A cell price is the closure delta: the helper's and registered callers' self deltas,
  plus the self delta of **every row whose incoming edge count from the measured set changed** (the
  rows added or removed by inlining decisions). A price whose closure differs from its measured rows
  is withdrawn. All callee rows used below are exactly constant across the 14 same-code armed draws.
- **New-only basis.** Every outline helper is a new-only body (base self and visits 0; armed self:
  1034 52,180,871, 1383 33,261,184, 1038 916,686, 1276 819,234, 1083 630,000, 1184 516,110,
  1086 276,000, 1199 227,147, 1185 220,800, 1110 92,848, 1014 164,547), and each helper's
  registered caller rows cover every incoming edge. A surviving price is therefore an A/B
  differential saving; the evaluator placements are shared code and their differential is
  (armed delta − real delta) on the exact union currency.
- **Shared caller.** Rows 1184, 1185 and 1199 call 1186; 1184/1185 survive. Their group nets carry
  row 1199's data-dependent draw; the row-local closures are the clean prices (190,434 and 76,786).
  A joint build measures them once, never as a sum.
- **Publication choices.** 1618-always, 1263-always and 1014-always are refused: the required
  publication choice relocates into the caller. No repair may move, drop or reorder a publication
  choice.
- **Additive savings are never claimed** (`additive_claimed=false`, verbatim). The joint build is the
  price.
- **Class-form.** A family that cannot be repaired by a measured placement and is required by the
  class form returns to the user as a named cost; it is never silently accepted and never called
  zero.

## The gate's corrected table (re-derive every number in Stage B; a differing decision number is a STOP)

Spendable, closure = measured: **1034 13,973,401**; **1038 297,745**; **1276 199,557**;
**1083 39,600**; **1184 190,434 row-local** (group 193,112); **1185 76,786 row-local** (group
79,250); **1110 73,624**; **coldout differential 3,735,504** (armed 22,584,060 − real 18,848,556;
the armed flag class alone −1,680,268 against real +6). Withdrawn: **1383 +376,710** (the collect
moves into `SpecFromIter::from_iter`, edge 0→426,421, self +22,167,667 — a row exactly stable at
841,416,207 across all 14 same-code draws); **1086 +45,220** (`from_iter` 0→2,400, +271,200);
**1199 +18,347** (`association_lvalue_call_cache_key` 0→153,685 and
`has_valid_call_stability_proof_for_key` 0→75,133 appear as rows). Not a residual repair:
**fastpath 41,712** (armed 44,623,356 − real 44,581,644; inside the combined band 51,790). Null:
foldaddr, hoist; the 1184/1185/1110/1014 inline variants. Refused: 1618, 1263, 1014. Row 1056:
`cannot_settle` (the STOP is retained; do not reopen it).

## Stage A — entry pins and index (200 s)

Verify read-only; a missing or differing pinned input is a STOP. Pin at least:

- `.temp/brief-803/report.json` (`67cae6b5…`, 3,202,913 B) with `python3 -I scripts/report.py check
  803`; the 800/801/802 reports (`677b7185…`, `a73bcf31…`, `4bc712f2…`) with their checks;
- `.temp/brief-803/evidence/`: `evaluator.tsv` (`4e80b930…`), `outline.tsv` (`c6e9e996…`),
  `group-1186.tsv` (`322cdea8…`), `calibration-803.json` (`359945f5…`), `calibration-803.tsv`
  (`376de7c4…`), `battery-verification.json` (`e02aa636…`), `classification-ledger.tsv` (`64794aee…`),
  `class-form.tsv` (`ae7b9c7a…`), `raw-index.tsv` (`1fa6bb99…`), `entry-index.json` (`c4c692f9…`),
  and the per-cell result JSONs and measured-symbols TSVs named there;
- `.temp/brief-799/family-inventory.tsv` (`d2d53587…`, 93 families with per-family base/new/control,
  `new_minus_base`, observed ranges and the aggregate/member accounting);
- the resolved captures the numbers rest on: `.temp/brief-800/callgrind/whole-base.resolved.json.gz`
  (`34bea875…`), `whole-new.resolved.json.gz` (`431a8f23…`), `whole-identity-base…` (`e769c5b5…`),
  `whole-identity-new…` (`ee7c83ed…`), the 14 armed and 3 real draws of `calibration-803.tsv`, the
  four real-cell captures and the outline captures (1086-always is `.temp/brief-802/callgrind/`);
- the four role trees (baseline `ec602109…`/5,842; real `022d60af…`/5,851; disabled `c7156d50…`/5,851;
  armed `318933db…`/5,851);
- the eight outline source diffs and the real-cell diffs (`evidence/*.diff`), the release receipts,
  the driven ledgers (PIN `a2c48c90…`), the LP01 standing receipt (`standing-hoist.json`);
- the live tree's **5,881 paths**: the 803 exit manifest's 5,880 paths
  (`evidence/live-exit.tsv`) all present and identical, plus this brief
  (`docs/dev/codex/brief-804-….md`). Emit `.temp/brief-804/evidence/entry-index.json`.

## Stage B — the closure-corrected price ledger (900 s)

Recompute from the retained resolved captures, for every cell that has a price or a withdrawal:
the measured rows delta, the closure delta (helper + registered callers + every row whose incoming
edge count from the measured set changed, naming each such row and its self delta), the band, and the
role basis. For the evaluator cells compute the differential (armed − real) on the exact union
currency and the combined band. Emit `evidence/price-ledger.tsv`:

```
cell  kind=evaluator|outline  roles=armed|real|both  variant=inline|always
measured_delta  closure_delta  band  differential  status=spendable|withdrawn|refused|unpriced|null
evidence=<result json> <capture>
```

Print one `PRICE804` line per cell and a summary line `PRICELEDGER804 spendable=<n> withdrawn=<n>
refused=<n> unpriced=<n> null=<n> gross_spendable=<n> additive_claimed=false`. Any cell whose number
differs from the table above is a STOP naming the exact difference; a wording-only difference is
recorded in `brief_defects`, never silently changed.

## Stage C — the family mapping (1,200 s)

Map every spendable price onto the 1,743-row classification ledger and the 93-family inventory:

1. **Rows.** For each spendable cell, the helper body row and each caller row with their
   `new_base_Ir`; mark each row addressed, and by which price. A row may be addressed by at most the
   price whose closure contains it; rows outside every closure are **uncovered**.
2. **Families.** For each family, its `new_minus_base` and observed range, the mass addressed by the
   spendable prices, and the uncovered mass. The evaluator family's uncovered mass includes the
   fastpath's non-differential part and the coldout residual.
3. **The residual arithmetic.** Sum to the translated residual (+37,644,521, the inventory's own sum)
   and state the native counterpart (+38,126,375) with the mapping rule; state the **addressable**
   mass (the spendable closures, with the 1186 group counted once and row 1199's draw bounded at
   2,824), the **package differential** (the seven attributes plus coldout, expected ≈ 18.6 M, to be
   measured jointly — never summed), and the **uncovered** remainder by family. A row or family that
   cannot be dispositioned is a STOP (zero unknown).
4. **Class-form determination.** For each uncovered family, state whether a measured placement exists
   (name the missing measurement) or whether it is class-form-necessary (with the `path:line`
   evidence). The 38 publication sites (37 independent + 1 reused, 4.0 Ir/visit, visits 366,910–
   366,919, exclusive Ir 1,464,123–1,464,159) stay as the declared class form; say so.

Emit `evidence/family-mapping.tsv` and `evidence/residual-arithmetic.json`. Print
`MAPPING804 rows_addressed=<n> rows_uncovered=<n> families_addressed=<n> families_uncovered=<n>
uncovered_Ir=<n> zero_unknown=<true|false>`.

## Stage D — the repair package, the joint design, and the drafts (900 s)

1. **The package.** The seven surviving attributes (1034, 1038, 1276, 1083, 1184, 1185, 1110) plus
   the coldout refactor. Excluded, with reasons: the three withdrawn cells, the three refused
   cells, the null and within-band cells, row 1056, and the fastpath (measured as a general
   default-path saving; not a residual repair; a separate future decision — state this, do not
   bundle it). If Stage C shows an attribute's closure cannot be composed with another (beyond the
   1186 group), say where and propose the split for the director.
2. **The joint measurement design.** One armed staged build with the seven attributes (`J1`) and one
   with them plus coldout (`J2`), each built from the sealed armed tree at one path, one whole
   canonical capture each, exact currencies: every helper/caller row with its closure, the evaluator
   union, the flag class, the eight short slopes, the canonical 120 labels, the off-state flags, the
   publication invariant on all 38 gates, the driven ledger PIN `a2c48c90…` (26 lines, 1,741 B), the
   owners and control-owners tests, and the callback/queued requests. The bars: J1's closure must
   equal the seven closures within the pre-registered tolerance (state it) and J2's increment must
   equal the coldout differential within its band; a failure names the interfering cell. Also
   measure the **real-role** counterparts (the attributes' real-side saving must be 0 and coldout's
   the 803 value) so the package differential is exact.
3. **The implementation path.** Where each attribute lands in the **live** source (`path:line` for
   every declaration in the live tree, not the staged images), the coldout refactor's files, the
   publication-choice preservation, the fmt and workspace gates, the standing-inventory obligations
   (the LP01 inventory is untouched by these files — verify against the 803 `standing-coverage.json`
   — and the render will need the standard replay set), and the one-variable rule (the package is the
   variable; the fastpath is not in it).
4. **The render/install/freeze draft.** How the repaired source is rendered into the armed tree,
   re-installed, re-verified (the R4 install verifier and the seven tables) and re-frozen; the
   receipts and the scope gates.
5. **The fresh-epoch draft.** The replaced qualification rule (the versioned interference budget,
   driver on CPUs 0–13, the measured process pinned to 14, both siblings polled at 100 Hz plus
   lossless sched/fork/IRQ events, IRQ-then-task interval partitioning, zero lost events, prelaunch
   0.05×T+2/HZ, whole-job E14/E15 ≤0.005×T+2/HZ, scheduled foreign ≤0.005×T and ≤0.050 s per sliding
   1-s window, policy hashed before calibration, failed calibration stops); its red/green
   calibrations (quiet ADMIT; a 1.000 CPU-s spinner on CPU 14 then 15, each REJECT); the qualified
   whole-panel captures per arm; the translated/native reconciliations; the A/B bar; Stage C/C10
   readmission; the 744 restart. Each step gated.

Emit the drafts as `evidence/draft-implementation.md`, `evidence/draft-render.md`,
`evidence/draft-epoch.md`, each in the template's shape with every invariant, falsifier, bar and
charge box; the open questions go to the report, not into the drafts.

## Stage F — synthesis and handoff (400 s)

One table: cell, status, closure, band, differential, addressed mass. State the addressable vs
uncovered split, the package's expected differential, and the class-form remainder. Keep the
classification ledger at **89 assigned + 22 carried + 1,631 aggregate + 1 STOP = 1,743** and
`additive_claimed=false`. Re-walk the live tree (5,881 paths) and run `python3 -I scripts/report.py
check 804`; the report and its detached summary must agree; the exit audit must find zero live
changes and zero sealed differences. Print:

```
SYNTHESIS804 spendable=<n> gross=<n> package_differential=<n> uncovered_Ir=<n>
row1056=cannot_settle ledger=89/22/1631/1 additive_claimed=false
```

## STOP rules

- A missing or differing pinned input is a STOP.
- A decision number that differs from the table above is a STOP naming the exact difference; a
  wording-only difference is a recorded brief defect.
- A row or family that cannot be dispositioned is a STOP (zero unknown).
- A proposed repair that moves, drops or reorders a publication choice is refused.
- A sum of prices in place of a joint measurement is a STOP.
- An epoch rule without its red/green falsifier is a STOP.
- Row 1056 stays `cannot_settle`; do not reopen it.

## Pitfalls (the campaign's lessons)

- The closure is the price: inlining decisions move work between rows, and the rows that appear or
  disappear are exact (their same-code spreads are 0). Never price helper-plus-callers alone.
- `#[inline]` on a new-only helper can remove the symbol and un-inline its callees (1199); the
  always variant can re-bill into the caller's iterator machinery (1383, 1086).
- The whole-panel instruction counts are not deterministic: the 14 same-code armed draws spread
  17,333,653 and the 3 real draws 3,842,503. The exact per-row currencies decide; whole-panel figures
  are informational.
- The group net carries row 1199's draw (band 2,824); the row-local closure is the clean price.
- The mimalloc compilation stamp makes whole-`.text` identity impossible; compare measured ranges.
- The flag class is PC-based: a build whose evaluator addresses moved needs its own table.
- Every run is serial and alone; retain every raw before parsing; print only numbers you can point to
  in a retained file.

## Charge and hard rules

- Stages: A 200 / B 900 / C 1,200 / D 900 / F 400 = **3,600 s**, funded by brief 803's remaining
  2,777.226 s plus an explicit **822.774 s** new allocation (「预算不够就追加吧」). Every stage inside
  its box; a box that would be overrun stops at the deadline with the exact missing measurement.
- **Design-only**: `NO_SOURCE_EDIT`, no build, no capture, no acceptance suite, no git command, no
  tag, no network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`. `.temp` only,
  never `/tmp`. Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381,
  P2385, P2387, P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized.
- After the seal: `python3 -I scripts/report.py check 804` must be OK, the detached summary must
  equal the report's summary, and the report must carry the verbatim printed lines and every
  evidence path.
