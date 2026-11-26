# Brief 806c — G15-CM4 D.2: the declared scene deltas

Read briefs 806, 806a and 806b first. This amendment supersedes brief 806b's Stage B item 4 scene
bar — and the 806 STOP rule "a scene line that differs from the 797 reference" — for the four
declared D.2 cells only. Everything else in 806/806a/806b stands, including the declared-extras
admission. No tag, no network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`.
Scratch under `.temp/brief-806/`; never `/tmp`. No git.

## Why this brief

Brief 806b stopped correctly at Stage B.5: the 797 scene projection requires the rendered
attribution keys to equal its 122-row reference, and the rendered scene has four additional `diff`
cells — the four landed `#[inline(always)]` attributes in the three R4-mapped files (rows 35/45/51).
The 797 reference predates the D.2 package, so rendering the nine final sources must add exactly
these four diffs. The bar was mine (defect #28); ruling 842 (gate c190) grants the declaration.

## Checkpoint — verify, do not redo

- **The authenticated render** (`.temp/brief-806/completion/`): the four role trees — baseline
  `cb9205a6…`/5,842, real `dda9146e…`/5,851, disabled `30cd48fc…`/5,851, armed `df0db63c…`/5,851;
  the ten render outputs (the seven tables `d3588fb9…`, `1c48e15a…`, `99f7c5bb…`, `dc648d65…`,
  `8e29c41f…`, `2ac559d1…`, `70a6da97…`; additions `feca9443…`; mutants `c214a4aa…`; overlays
  `44bb8f11…`); the 190-row map (`r4-pairing.tsv`; map `bfb2fe15…`, install pairing `8f7973e3…`;
  identity rows 35/45/51/170; the ten changed rows 160/164/165/166/171/172/182/183/184/186);
  `R4 MAPPING PASS paths=190 … unexplained=0`; the scene seal `SEAL PASS overlays=165
  armed_postimages=165 real_preimages=164 role_specific_real_F13=1 additions=9 carried=15
  postimages_791=4 chain=152 class_targets=2422 mutants=151`; the pairing
  (`scene/classifier-pairing.json`, `scene/projection-cells.tsv`, `scene/observed.tsv`); the
  standing replay and role replay PASS lines; the exit audit (5,887 paths, zero changes); the 806b
  report (`report.py check` OK, 1,175 citations).
- **Charge**: cumulative 2,758.314 s of 28,400; **25,641.686 s remain**. This brief allocates
  17,400 s: B2 600 / C 10,000 / D 6,000 / F 800; 8,241.686 s stay unallocated reserve.

## Stage B2 — the scene gate on the declared deltas (600 s)

Re-run the scene gate on the authenticated render with these four cells declared. Re-derive every
value from the rendered images and the live files — assert, do not trust:

1. `crates/rum-runtime/src/runtime/definitions/definition_updates.rs`, `current_symbol_held_update_value`,
   kind `diff`, reference span `38c76e1bd670751823a35c33f322482978001104b74aa3a31300992b85bdac5c`
   (905 B) → rendered span `715d4527927e7448cbd4eaebec93b5ffd0e36cf4275344b83eaca1a3086881d5`
   (927 B), Δ +22; reference line 277, rendered line 381, live attribute line 382.
2. `crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs`,
   `normalize_value_table_target_expr`, kind `diff`, reference span
   `129ce5894a32362badb523b057c86e86eb36bd1c6be6d0cd49775b09bd3a38ff` (812 B) → rendered span
   `23d0ce5e644413b35bddcda15d7afadc7f7a3cc712edfb9b7a718589ad8e8fc4` (834 B), Δ +22; reference
   line 109, rendered line 109, live attribute line 109.
3. `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`,
   `normalize_association_set_lvalue_target`, kind `diff`, reference span
   `739e7b85bc720f378c2e0af8680a81867689e14c53bb3c695b4ac094f4e6ec0a` (726 B) → rendered span
   `03d19d633ce7e7abff9dd5fb7d44c5fc093f76bcec669b2e6c7add7d963328e6` (748 B), Δ +22; reference
   line 775, rendered line 845, live attribute line 846.
4. `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`,
   `resume_association_set_lvalue_selector_evaluation`, kind `diff`, reference span
   `359ca6361543b5d9595ad120dbfc633e880c884b914109fe5a87389a4a5aca8f` (1,396 B) → rendered span
   `994274158c2f12b13fe52d9000995d960eb058ba41f422fab3f07d34636a0c82` (1,418 B), Δ +22; reference
   line 807, rendered line 878, live attribute line 879.

Pin these values as literals in the gate (the classifier pairing under `scene/` is the same data,
not the pin). For each cell the proof obligation is:
removing the single inserted line from the rendered armed span yields the reference span
byte-for-byte; the inserted line's text is `#[inline(always)]` (17 characters; the classifier strips
the first line's indentation); the rendered real span equals the reference span; the live source
file carries the attribute at the stated line. The four cells are the four attributes of the 805
landing in the three mapped files; the other three attributes
(`collect_upvalue_candidate_symbols`, `normalize_rewrite_rules`,
`build_guarded_replace_plan_from_targets`) are in unmapped files and identical in all four roles —
assert that they produce no scene cell.

**The gate semantics.** The expected attribution set = the 797 122-row disposition set (unchanged,
from `.temp/brief-797/evidence/amendment6/attribution-dispositions.tsv`) + the four declared rows,
kind `diff`, shape `inline attribute insertion`, owner `G15-CM4 D.2 attribute set`, evidence naming
the producer (the brief 805 D.2 landing), the reference and rendered span hashes, the inserted
line, the body identity and the R4 row. The legacy driver, its source and its reference inputs are
not modified; on its own inputs it must still print

```
REFERENCE SCENE PROJECTION PASS rows=122 unchanged=116 refreshed=3 rebound=1 added=2 diffs=97 extensions=25 shapes=26 owners=14 hot=21 missing=0 unmatched=0 ws=0
```

and the probe closure must still print `PROBE CLOSURE facades=4 outside=3 attributed=3
unexplained=0`. The rendered projection must print

```
SCENE PROJECTION PASS rows=126 diffs=101 extensions=25 shapes=27 owners=15 hot=21 missing=0 unmatched=0 ws=0
```

and the rendered classification counts must equal the reference counts with exactly
{verbatim −2, diff +4}: verbatim 13,604→13,602, diff 97→101, total 14,237→14,239. The four
verbatim→diff conversions are the declared cells; the two new verbatim rows are `run_configured_timing`
and `run_configured_timing_cold` in `profiling/timing.rs` (present in all four roles); cold 334,
dispatch 122, extension 25, wrapper 51 and facade 4 are unchanged.

**Fail-closed.** Any cell outside the union stops with its key; any declared cell whose span proof
fails stops with the cell; any other count movement, any other key or multiplicity change, or a
different reference line or probe-closure line stops. The adapted gate must keep the legacy
rejection path intact for every non-declared difference.

## Stage C — the acceptance on the rendered tree (10,000 s)

Unchanged from briefs 806b/806a: `cargo fmt --check` clean; the lib suite **1,115 passed / 0 failed
/ 58 ignored**; the class gate 197/0/0 (flag-on); the fourteen selectors 14/0/0 with the lane bind;
the eight selectors 8/8; the census dump **`4b5fcd37…`** with the `ACCEPTANCE_BASELINE` line; the
source-auth control with `RUM_CM740_ADDITIONS` = the rendered scene, green; the F48 pair 2/0/0;
every focused receipt retained with its raw log. The full workspace stays the director's.

## Stage D — the install and the re-freeze (6,000 s)

As briefs 806a/806b: install the map into the live tree with the 797 semantics (190 paths, every
live parent rechecked, the witnesses, the identity overlays, the zero-overlay real promotion), run
`verify_install.py` and report `R4 LIVE VERIFY PASS …` with the rendered values; the whole-tree
delta allows `TODO.md` and the declared `docs/dev/codex/` extras only; the crates subtree must equal
the rendered armed image exactly. Re-freeze the scene and the counters.

## Stage F — the report (800 s)

The `rum-report/1` DONE report for brief 806: the render's deltas, the four declared cells, the
row-170 correction, the map update, the install and verify lines, the acceptance receipts, the
charge; the seal, `python3 -I scripts/report.py check 806`, the detached summary, and the exit
audit (every live change declared — the docs extras, the render's installed outputs with their
produced pins, zero worker source edits outside the render's outputs; zero unexplained).

## Charge

17,400 s of the 25,641.686 s remaining: B2 600 / C 10,000 / D 6,000 / F 800. Every stage inside its
box; a box that would overrun stops at the deadline with the exact missing measurement. The
8,241.686 s reserve stays unallocated.

## STOP rules

As brief 806b, plus: any scene cell outside the declared union, any failed span proof, or any
count/key movement outside the declared deltas stops. Q25's thirteen named STOPs stand (P0214,
P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418); 744/745/746
held; CM5 unauthorized; no tag; no network; no new dependency; no new `unsafe`; `.temp` only,
never `/tmp`; no git.
