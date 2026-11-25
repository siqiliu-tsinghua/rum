# Brief 803 — G15-CM4: the real-role evaluator batteries and the outline completion under the corrected control rule

Experiment/diagnostic. The deliverable is the named artifacts and the verbatim printed lines. No live
source change, no acceptance suite, no epoch, no tag, no network, no new dependency. All writes under
`.temp/brief-803/`; the live tree is read-only. Instruction counts only.

## Why this brief

Brief 802's Stage B `PATHCTL STOP` fired correctly as written: the unchanged real path control read
1,292,126 flag instructions against the binding 1,291,991 and the pre-registered ±100. Ruling 835
(gate c183) accepts the STOP and fixes its cause: the ±100 was my forecast from two clean draws, while
the observed third draw is +135. The control is structurally the untouched real tree — staged from
`022d60af…`, `.text` byte-equal to the 800 base (`9993a9a3…`), the three measured symbol ranges
byte-equal, flag PCs 33/2, slopes 3/3/5/5, row 1056 213, canonical output PASS — so the four real
evaluator cells were stopped by a band, not by a build fault.

Brief 802 also delivered the armed measurements: fastpath 44,623,356 and coldout 22,584,060 combined
self Ir, six outline prices (13,973,401 / 21,790,957 / 297,745 / 199,557 / 39,600 / 193,112), the
refusal of forced inlining on 1618 and 1263 (publication choice), and row 1056's retained STOP. Five
outline cells remain (1086-always route-ordered; 1199 / 1185 / 1110 / 1014), and the four real cells
were never run. This brief completes them.

## The corrected control rule (binding for this brief and the epoch)

- A path control is admitted on **structural currencies, checked exactly**: the staged tree hash, the
  measured symbol ranges (mimalloc stamp excepted), the flag PC table (pc / flag / destination and the
  static/executed sets), the four flag-address slopes, row 1056's per-visit, the publication census
  and gate counts, and the canonical 120-label output. A structural mismatch is a measurement fault —
  STOP, name it.
- The **total-Ir currencies are data, not control gates**: the flag class, body rows 1115/1116 and
  the evaluator union vary with execution data at fixed code. They enter only as a cell's decision
  band, computed as `max |draw − binding|` over **every same-code draw**, and a cell's price must
  exceed its band.
- A pre-registered number is a forecast; the observed bound governs. The recorded draws:
  - armed (14 same-code captures — the 800 reference, identity, the 801 control, foldaddr, hoist and
    the nine `#[inline]` outline variants whose `.text` is `2a2ae65e…`): flag class max |Δ| **616**
    (range 15,416,384–15,417,396), body 1116 **12,740**, evaluator union **20,120**;
  - real (3 draws — the 800 base reference, identity-base, the 802 real control): flag class **135**
    (1,291,991 / 1,292,048 / 1,292,126), body 1116 **19,755**, union **31,670**.
- The four real evaluator cells are already **admitted** on these bands (their cell band is the real
  union's 31,670). The armed evaluator prices stand (44.6M and 22.6M are ~2,000× the revised armed
  union band).

## Stage A — entry and retained-input pins (150 s)

Verify and index read-only; a missing or differing pinned input is a STOP. Pin at least:

- the 800 report (`677b7185…`, 2,855,476 B), the 801 report (`a73bcf31…`, 2,460,537 B) and the 802
  report (`4bc712f2…`, 4,909,544 B), each with `python3 -I scripts/report.py check 800|801|802`;
- the 802 calibration (`evidence/calibration.tsv`, `979c1675…`) and its raw index
  (`evidence/raw-index.tsv`, `34731c5e…`, 107 raws);
- the 802 real control: raw `callgrind/whole-realctl.out` (`29ed66f6…`), its resolved capture, the
  build receipt (source `022d60af…`, ELF `5480eb7b…`, `.text` `9993a9a3…`), the eight short raws and
  the canonical normalized output `c193d87f…`;
- the 800 reference draws: `whole-base.out`, `whole-identity-base.out`, `whole-new.out`,
  `whole-identity-new.out` with their raw and resolved digests, and the two flag PC tables;
- the 802 same-code outline raws for the nine inline variants (`whole-outline-1034-inline.out`,
  `-1383-`, `-1038-`, `-1276-`, `-1618-`, `-1263-`, `-1083-`, `-1184-`, `-1086-inline.out`);
- the 802 release receipts for `whole-outline-1086-always` (source `769076a5…`, ELF `40dfa676…`,
  `.text` `f18a7c25…`) and its raw `whole-outline-1086-always.out`, and the 802 outline table;
- the four role trees (baseline `ec602109…`/5,842; real `022d60af…`/5,851; disabled `c7156d50…`/5,851;
  armed `318933db…`/5,851);
- the canonical program `dc6af977…`, the functional program `c70feffd…`, the 14 off-state flags, the
  CPU-14 identity, the toolchain and the offline cache;
- the live tree's **5,880 paths**: the 802 exit manifest's 5,879 paths
  (`evidence/live-entry.tsv`, `0c7cc5be…`) all present and identical, plus
  `docs/dev/codex/brief-803-…md` (this brief). Emit `.temp/brief-803/evidence/entry-index.json`.

## Stage B — the corrected calibration and admission (250 s)

1. Recompute the same-code capture sets from the retained resolved captures: the 14 armed draws and
   the 3 real draws above, each with its flag class, body 1115, body 1116 and evaluator union. Emit
   `evidence/calibration-803.tsv` with the max |draw − binding| band per currency.
2. Re-emit the control verdict for both roles **under the structural rule**: the structural
   currencies exact, the totals recorded as draws. The armed control and the real control must both
   print `verdict=match`; a structural mismatch is a STOP.
3. Emit the corrected `ADMIT` table for every remaining cell: the four `eval:E2.1..E2.4:real` cells
   with `band=31670` and `verdict=admit`; the five `outline:1086|1199|1185|1110|1014` cells with
   their helper/group bands (`helper:NNNN:0`, `group:1186:2824` for 1199 and 1185).

Print:

```
CTRL803 role=<armed|real> structural=<exact|fault:item> flag_draws=<a,b,c,...> flag_band=<n> body1116_band=<n> union_band=<n> verdict=<match|fault>
ADMIT803 cell=<eval:E2.1:real|...|outline:NNNN> currencies=<key:band,...> band=<n> verdict=<admit|stop>
```

## Stage C — the four real evaluator cells (2,400 s)

One cell at a time, serially, at the one path. Stage each candidate on the sealed **real** tree
(`022d60af…`, 5,851 files) at `.temp/brief-803/active`, apply exactly one edit, build with the recipe
of record, save the ELF / `.text` and the measured symbol ranges, and compare the measured ranges to
the real reference (the row-1056 `resume`, `evaluate_inner_unstable`,
`evaluate_once_in_accounting_mode`, plus the symbols touched by the edit).

- **An identical measured image is a null candidate**: print `text=identical price=0`; no whole
  capture and no battery are required beyond the reference's own canonical output. This is E2.3's
  rule from brief 802, adopted by ruling 835.
- **A changed image** runs the real battery: one canonical whole-panel capture (the exact 798 chunk
  `taskset -c 14 valgrind --tool=callgrind --dump-instr=yes --vgdb=no --enable-debuginfod=no
  --log-file=<vg> --callgrind-out-file=<raw> <elf> --file <canonical>`, cwd the repo root, stdin
  `/dev/null`, env `ordinary_env() ∪ profile_off() ∪ {DEBUGINFOD_URLS:""}`), the eight short-label
  captures, the off-state canonical equality (120 labels, normalized `c193d87f…`, all 14
  `RUM_PROFILE_*` = 0), the cell's own flag PC table (its static/executed sets and per-PC
  multiplicities), the four slopes, and the body rows 1115/1116 with the evaluator union.
  **Price** = the caller-inclusive evaluator union delta against the real binding, with the cell's
  band 31,670; a delta within the band is `unpriced (within band)`. The real role has no PUBLISHED
  symbol — no publication census.

The four candidates and edits are exactly brief 802's (E2.1 fastpath at
`profiling/config.rs:70–85`; E2.2 coldout out-of-line work in `profiling/timing.rs` and
`config.rs:115`; E2.3 fold the address at `profiling/state.rs:20`, `config.rs:78,570`; E2.4 hoist the
address across a proven immutable interval at `eval_core/mod.rs:250`, `config.rs:39`,
`profiling/reset.rs:126`). A semantic change to whether or when any profiling event is recorded is a
STOP, never a cost change. A failed falsifier is recorded FAIL and that candidate cannot be a repair.

Print per cell:

```
EVAL803 role=real candidate=<fastpath|coldout|foldaddr|hoist> text=<identical|changed:syms> flag_static=<n> flag_executed=<n> slopes=<a,b,c,d> class_delta=<n> body1115_delta=<n> body1116_delta=<n> union_delta=<n> band=31670 falsifiers=<PASS|FAIL:name> price=<n|0|unpriced>
```

## Stage D — the five outline cells (1,800 s)

Armed-role cells, one at a time at the one path. The body text and the publication choice must never
change; a cell that removes or moves a choice is a STOP.

- **1086 always** (`runtime/definitions/definition_updates.rs:976`): the build and capture already
  exist (source `769076a5…`, ELF `40dfa676…`, `.text` `f18a7c25…`, raw `whole-outline-1086-always.out`,
  union delta −226,800). Run the missing **route-ordered** check and the light battery (off-state
  equality, publication invariant, driven ledger `a2c48c90…`); if PASS, print the price 226,800.
- **1199** (`kernel_data/association_lvalue.rs:7`), **1185** (`kernel_data/association_data.rs:878`),
  **1110** (`definitions/value_table_assignment.rs:109`), **1014**
  (`definitions/application/replace_all.rs:83`): for each row add exactly one `#[inline]` at the
  armed declaration, build, save the ELF / `.text` and the symbol table, capture the canonical whole
  panel, reduce the helper's self/visits and its caller rows, and run the light battery. If the
  symbol survives, replace the attribute with `#[inline(always)]` and repeat once; record both.
  **1014 carries a publication choice inside its ordinary body** — run the retained-choice check and
  STOP that cell if the choice is missing, relocated or unresolved.

**Prices and accounting**: the cell price is the net change of its helper self + caller rows against
the armed binding, in exact per-visit/per-call counts; the band is the cell's Stage B band (the twelve
exact helpers 0; row 1199 and the group-1186 union 2,824), and a price within its band is
`unpriced`. Rows 1184/1185/1199 share caller 1186: report the group once with that caller counted
once, and report the row-local net separately (the group net carries row 1199's data-dependent draw;
in 802 that was −2,678 of the 1184-always group net against the row-local −190,434). The adapter pair
1147/1448 is outside the cells. The suspension oracle stays deferred for these cells (an `#[inline]`
attribute is semantics-neutral — state the deferral verbatim).

Print per row and for the group:

```
OUTLINE803 row=<n> decl=<path:line> symbol=<gone|present> visits=<n> self_per_visit=<f> caller_net=<n> band=<n> price=<n|unpriced> falsifiers=<PASS|FAIL:name>
OUTLINE803_GROUP group=1186 members=1184,1185,1186,1199 net=<n> counted_once=true band=2824 row_local=<n>
```

## Stage F — synthesis and handoff (200 s)

One table per family, each price carrying its falsifier outcome and its band; the class-form-required
remainder stated separately; `additive_savings_claimed=false` verbatim. Keep the updated
classification ledger (row 1056's disposition stays `unassigned: STOP`, the ten 800 Stage-D rows and
the 802 prices) at 89 assigned + 22 carried + 1,631 aggregate + 1 STOP = 1,743. Re-walk the live tree
(5,880 paths) and run `python3 -I scripts/report.py check 803`; the report and its detached summary
must agree; the exit audit must find zero live changes and zero sealed differences.

Print:

```
SYNTHESIS803 real_prices=<n> outline_prices=<n> row1056=cannot_settle ledger=89/22/1631/1 additive_claimed=false
```

## STOP rules

- A missing or differing pinned input is a STOP.
- A structural control mismatch is a measurement fault — STOP, name it. A total-Ir draw outside a
  forecast band is recorded, never a STOP.
- A cell whose build or capture cannot be completed is a STOP for that cell, with the exact missing
  artifact.
- A failed falsifier is FAIL; that candidate cannot be a repair.
- A cell that moves a required publication choice is a STOP for that cell.
- Row 1056's STOP is retained; do not open further row-1056 cells.
- A stage that cannot settle its question STOPs with the exact missing measurement.

## Pitfalls (the 802 lessons)

- The `.text` hash carries the mimalloc compilation stamp; compare **measured symbol ranges**, not
  whole-text hashes, except where the whole text happens to match.
- `#[inline]` rarely removes a symbol; the always variant is the one that changes the image.
- The flag class is PC-based: a build whose evaluator addresses moved has no valid reference-table
  reading — extract the cell's own table (802 printed garbage for such cells).
- The oracle's LP01 producer fingerprint is source-coupled: a cell that edits an inventoried file
  fails before its checks. The real battery here does not run the oracle; if one is attempted,
  regenerate the inventory into the staged copy and record it — never call the fingerprint a
  semantic failure.
- The driven ledger must match `a2c48c90…`; the callback and queued programs must request input (the
  802 first-run split streams and the non-requesting option form were harness bugs, corrected).
- Every run is serial and alone; retain every raw before parsing; print only numbers you can point to
  in a retained file.

## Charge and hard rules

- Stages: A 150 / B 250 / C 2,400 / D 1,800 / F 200 = **4,800 s**, funded by brief 802's remaining
  1,784.146 s plus an explicit **3,015.854 s** new allocation (「预算不够就追加吧」). Every stage
  inside its box; a box that would be overrun stops at the deadline with the exact missing
  measurement.
- `.temp` only, never `/tmp`. `NO_SOURCE_EDIT`; no acceptance suite; no git command; no tag; no
  network; no new dependency; no new `unsafe` in `rum-core`/`rum-runtime`. Q25's thirteen named STOPs
  stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418);
  744/745/746 held; CM5 unauthorized.
- After the seal: `python3 -I scripts/report.py check 803` must be OK, the detached summary must equal
  the report's summary, and the report must carry the verbatim printed lines and every evidence path.
