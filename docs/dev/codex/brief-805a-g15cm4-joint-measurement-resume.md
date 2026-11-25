# Brief 805a — G15-CM4 D.2 continuation: finish the joint measurement, land it, verify

Continuation of brief 805. Scope: `.temp/brief-805/` and, in Stage D only, the seven live
source paths named below. No git, no render, no install, no re-freeze, no epoch, no tag, no
network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`. All scratch under
`.temp/brief-805/`; never `/tmp`.

## Why this brief

Brief 805's attempt 1 (cdx) died on the quota wall at 06:50:01 +0800 after completing Stage A
and most of Stage B (`You've hit your usage limit … try again at Nov 28th, 2026 1:18 PM`;
rc=1; the log is archived at `.temp/logs/archive/cdx_brief_805.quota-1.log`; no `.git-attempts`;
the live tree untouched). The user has restored cdx quota and directed that cdx continue. This
brief resumes 805 from the checkpoint with the same goal, the same package and the same
acceptance; the only changes are the director's re-registrations (AMEND-1..4) of forecasts the
joint measurement has falsified. The original bars remain in force wherever the amendments do
not say otherwise.

## Checkpoint — verify, do not redo

- **Stage A** (worker, 381.889 s of its 500 s box): `ENTRY805 phase=entry pins=91 role_roots=4
  live_paths=5,882 profile_flags=14 mismatches=0 verdict=PASS`; `J0_CONTROL role=armed …
  flag_PCs=57 slopes=22/22/58/26 row1056=14012x218 labels=120 differing=none verdict=PASS`;
  `J0_CONTROL role=real … flag_PCs=33 slopes=3/3/5/5 row1056=14012x213 … verdict=PASS`;
  `CONTROL805 both=fresh structural=exact verdict=PASS`.
- **Stage B** (worker to 06:50:01, then the director's completion; `stage-B-end − start-B =
  876.67 s` of the 4,200 s box): `builds.tsv` has four rows; worker verdicts `BUILD805
  key=armed-J1 files=5 … verdict=PASS`, `key=armed-J2 files=7 … PASS`, `key=real-J1 files=5 …
  PASS`; the director deleted the killed real-J2 capture leftovers (a 0-byte `.out`, its
  `vglog`/stdout/stderr) and reran the worker's own `panels.py`: `BUILD805 key=real-J2 files=7
  whole=1 short=8 raw_bound=true verdict=PASS`, whole 16,393,665,617 Ir, 120 labels
  `c193d87f…`, slopes 3/3/5/5. All six whole captures and 8 shorts each are under
  `callgrind/` with their receipts under `evidence/`.
- **Stage C numerical core** (director, 559.768 s): `decision.work()` over the six builds
  returns 111 checks, 0 STOP; `evidence/joint-decision.tsv`, `evidence/joint-analysis.json`.
  The measured joint result —
  - outline J1 union: **armed −14,865,545** (the five independent cells exact; group
    1184/1185/1186 −281,618; caller 1186 +455,292; row 1199 −135 within the 2,824 bound);
    **real −5,110,474**;
  - evaluator J1→J2: armed **22,585,014** (band 20,120), real **18,832,657** (band 31,670),
    differential **3,752,357** (band 51,790);
  - flag class: armed **−1,680,284** (band 616), real **+78** (band 135);
  - slopes: armed J1 22/22/58/26, armed J2 20/20/52/24, real 3/3/5/5 on both; cold helper
    calls/self 0 with profiling disabled in all four J builds; publication 38 gates
    (37 independent + 1 reused) and the census PASS on armed-J1 and armed-J2;
  - the definitive joint price: armed **37,472,818** − real **23,916,394** = **13,556,424**
    Ir. The nominal **18,586,651** is a preregistered comparison target only;
    `additive_claimed=false`.

## The amendment register (director, gate c186) — keep, do not revert

`.temp/brief-805/tools/decision.py` carries the register; the pre-amendment STOP version is
`decision-stop1.py`. The first run's STOP lines are preserved in `joint-decision.tsv`.

- **AMEND-1** — armed shared-caller non-additivity: row 1186 `try_set_association_lvalue`'s
  joint growth is +455,292, not the single-cell sum 469,690 (−14,398), with its seven new call
  edges re-attributed (to_vec/is_system_name 8,802; clone/grow_one/evaluate'2/finish 2,400
  each); the measured group −281,618 and union −14,865,545 replace the sums.
- **AMEND-2** — the real-side J1 saving is real codegen, not noise: the seven helpers exist
  only inlined on real (zero self/visits, no symbols), but `#[inline(always)]` changes the
  already-inlined code in four callers (`try_apply_up_values` −109 B, `eval_held_update_operation`
  +228 B, `finish_definition_assignment` −18 B, `try_set_association_lvalue` +31 B; `.text`
  +128 B). Three same-code real draws (805 real-J0, 800 base, 800 identity-base) spread only
  217 Ir on this closure union (1034 identical at 96,884,000 in all three), so the measured
  real J1 union −5,110,474 is the price.
- **AMEND-3** — (a) retained 1110 armed J0-J2 = −73,641: the coldout refactor changes the
  shared caller `finish_definition_assignment`'s code shape (6,989 → 6,860 B, 1,316 → 1,290
  instructions, frame 0x3f8 → 0x428). (b) The J1 evaluator drift −22,242 is 62 fewer dynamic
  calls into `evaluate_inner_unstable'2` at one site (915,395 → 915,333; the other site's
  131,583 identical), at unchanged per-call cost (1116 −14,105 + 1118 −8,137 = 62 × (227.5 +
  131.2)); `evaluate_inner_unstable` is instruction-identical between armed-J0 and armed-J1
  (1,939 instructions, RIP displacements only). The 20,120 band was a 14-draw forecast; this
  draw extends the same-code spread to 22,242.
- **AMEND-4** — the four real retained rows: 1083 −54,274 (the closure's outside row is the
  libc leaf `/usr/lib/libc.so.6 | 0x00000000001b1ac0`, whose three same-code real draws
  spread 7,365 Ir and whose J1→J2 delta difference is 13,638 at identical 16,800→10,800
  edges), 1184/1185 −14,799 (`try_set_association_lvalue`, identical 8,802 visits, +480, no
  profiling callee; same-code spread 217), 1110 −26,393 (`finish_definition_assignment` code
  shape). The J2 joint build is the only admissible price; the J1 values stay as the
  comparison and the deltas are recorded.

## What remains

1. **Stage C battery** — first rewrite `evidence/start-C.json` to this attempt's monotonic
   with `{"attempt": 2, "attempt1_director_seconds": 559.768}` (the file is not pinned and
   `seal805.py` reads only `monotonic`). Then run `python3 -B .temp/brief-805/tools/batteries.py`
   (its `work()` is already green; `semantic()` runs): per-variant per-source visits, functional
   outputs, eager/deferred event phases, the disabled lease path; on armed the driven tests
   (`route_a_positive_and_all_falsifiers`, `publication_balances_each_independent_owner`,
   `publication_preserves_control_scope`), the exact 26-line / 1,741-byte ledger
   `a2c48c90fab83d49ffceb07e8c74992268fea353cf5e03574f8cbe63435d3895`, P0256, the 12 routes,
   the two mid-request callback edges (two hits each), resume to 42, queued order/result/closed
   scopes, typed 4+3 at 350/700, forced 50 visits at 111 Ir/visit. Quote every verdict line
   (`BATTERY805`, `SOURCE_VISITS805`, `LEASE805`, `DRIVEN`, `CALLBACK`, `FUNCTIONAL`, `EVENTS`,
   `TYPED`, `FORCED805`, `PUBLICATION`, `PUBLICATION_CENSUS`). Box 2,600 s for this attempt.
2. **Stage D** (8,400 s) — land exactly the seven `#[inline(always)]` insertions and the
   coldout diff in the seven live paths of brief 805 (`definition_analysis_helpers.rs`:254 and
   :485, `guarded_plan.rs`:4, `definition_updates.rs`:382, `association_data.rs`:846 and :878,
   `value_table_assignment.rs`:109, plus the coldout refactor in `profiling/timing.rs` and
   `profiling/config.rs`:569), nothing else. Then: `cargo fmt --check` clean and compiling; the
   semantic declarations equal to the accepted staged J2 sources; a release-byte identity
   receipt of a live build at one path against the staged J2 build; replay every source-covered
   standing producer on the final formatted sources **before** the first acceptance selector
   (F13 local fingerprints and contract records with falsifiers, class discovery and source
   authentication, ownership discovery/positions and the 152-member chain, LP01 1,115 rows /
   statement identity / columns 17-19 / binding column 3 / loss record, typed-slot receipts,
   the rejection census, promotion reviews/certificates in both configurations,
   overlays/additions/mutants through their producers), quoting the verdict lines; the focused
   runtime/stdlib/suspension tests and the full `cargo test --workspace` at
   `RUST_TEST_THREADS=8 --no-fail-fast` inside `memory-scope`, compared per binary and per name
   against the pinned gate (the ten accepted reds and the exact 71 ignores; a new failure or an
   unexpected pass returns to the director); the `scripts/core-language-microbench.sh` block
   base vs candidate, serial, in-kernel `AbsoluteTiming`; and the report of every affected
   position, fingerprint, certificate and generated cell.
3. **Stage F** (300 s) — write `evidence/seal-note.json` for `seal805.py` (the full
   `rum-report/1` report in `.temp/brief-805/report.json`, `brief: 805`, `stage: G15-CM4 D.2`,
   `verdict`, `summary`, `price: 13556424`, the amendment register in `sections`, the two-attempt
   accounting, `row1056: cannot_settle`, `additive_claimed: false`), run the seal and
   `python3 -I scripts/report.py check 805`; the detached summary must equal the report's; the
   exit audit must find zero unexplained live changes and zero sealed differences (the brief,
   `stage-clock.json` and `preregistration.json` are pinned and must not be rewritten).

## Charge

- Brief 805's total is 16,000 s. Attempt 1 (cdx): 946 s wall — Stage A 381.889 s (≤500), Stage B
  538.9 s to the death; the director's real-J2 completion brought Stage B to 876.67 s (≤4,200).
- The director's Stage C attempt used 559.768 s; it is recorded in `start-C.json`'s note and in
  the report, and the attempt-2 clock above restarts the C box (2,600 s) for the battery only.
- Stages D and F keep their boxes (8,400 / 300 s). Every stage inside its box; a box that would
  overrun stops at the deadline with the exact missing measurement. The report's charge section
  records both attempts and every stage's elapsed; the sum must stay within the 16,000 s total.

## STOP rules (unchanged unless amended above)

- A missing or differing entry pin, a J0 structural currency mismatch, an unbound capture, a
  cell outside its amended bar, a new cross-cell edge, a moved/dropped/reordered publication
  choice, a semantic difference in the coldout refactor, a missing producer, a changed
  identity/owner/exclusion, or an unaccounted file stops the attempt with the exact cells,
  caller, callee, counts and deltas named.
- Row 1056 stays `cannot_settle`; the fastpath, the withdrawn (1199/1383/1086), refused
  (1618/1263/1014), null and unpriced variants, foldaddr and hoist are not implemented.
- `additive_claimed=false` is binding: a sum of prices in place of the joint measurement is a
  STOP. Instruction counts only; every run serial and alone; retain every raw before parsing.
- Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385,
  P2387, P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network;
  no new dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git; no render, install,
  re-freeze or epoch.
