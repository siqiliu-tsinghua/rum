# Brief 806 — G15-CM4 D.2 render, R4 install and re-freeze

Continuation of briefs 805/805a/805b. Scope: the render, the R4 install and the re-freeze of the
D.2 package. The nine live paths are landed and committed (`7b0134f9`); **do not re-edit them**.
No tag, no network, no new dependency, no new `unsafe` in `rum-core`/`rum-runtime`. All scratch
under `.temp/brief-806/`; never `/tmp`. No git.

## Why this brief

805 is DONE. The live tree carries the nine landed sources (the seven measured runtime files, the
AMEND-5 census pin and the AMEND-6 compiler-oracle pin) and its release is byte-identical to
armed-J2 `c7706291…` / 18,689,432 B. Its generated inventories are **pre-render**: the live
workspace reads 4,323/241/71 against the pinned 4,552/12/71, and every one of the 229 new reds is a
derived cell whose producer is the render, paired at 805's gate to a nine-source staged green
receipt. This brief performs that render: regenerate the inventories and the class-form scene from
the final sources, update the R4 map's parents, install the map, verify it, and re-freeze the
sealed scene so the live tree is green again.

## Checkpoint — verify, do not redo

- **The nine live paths** (posting hashes): `definition_analysis_helpers.rs c9861b64…`,
  `definition_updates.rs 8b6022ef…`, `value_table_assignment.rs ee7b27d6…`,
  `association_data.rs 4efd9378…`, `guarded_plan.rs 611b97d3…`, `profiling/config.rs 9c202cb6…`,
  `profiling/timing.rs 2e3d38e2…`, `census/tests.rs db1c5a08…` (R4 row 170 overlay, preimage
  `4a76c2f5…` / 21,829 B), `compiler_oracle/exact.rs 54222956…` (test-only, outside the R4 map).
- **The 805 seal and gate**: the report's seal (1,709 artifacts, 0 unexplained, 0 sealed
  differences, 54/54 captures; `report.py check 805` OK 382); the joint price 13,556,424 Ir;
  `additive_claimed=false`; row 1056 `cannot_settle`; the retention-line closure-descent obligation
  (the 39 census rows and the two constructor contexts) open.
- **The nine-source staged receipts** (the render's forward references):
  `.temp/director/gate-805b/staged-round3.log` (lib 1,115/0/58; class 195/2 flag-off; F48 2/0/0;
  8 selectors 8/8), `staged-round4.log` (class 197/0/0 flag-on; the fourteen selectors 14/0/0 with
  the lane bind), `staged-lib2.log`, and `.temp/brief-805/logs/g9-census-after-producers.stdout`
  (1 passed; the dump `4b5fcd37…`).
- **The seven staged inventory files** (their nine-source hashes; the render's outputs):
  `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv d3588fb9…`,
  `.../retention_closure/inventory/promoted.tsv 1c48e15a…`,
  `.../ownership_graph/inventory/emission_chain.tsv 99f7c5bb…`,
  `.../ownership_graph/inventory/unknown_attributes.tsv dc648d65…`,
  `.../ownership_graph/inventory/unknown_captures.tsv 8e29c41f…`,
  `.../ownership_graph/inventory/unknown_macros.tsv 2ac559d1…`,
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/class-provenance.tsv 70a6da97…`.
- **The live workspace verdict**: `.temp/director/gate-805b/verdict.json` (the 8/9-source runs, the
  229-red classification, the pairings).
- **Charge**: 805 spent 6,923.741 s of 16,000; **9,076.259 s return** to the pool.

## Stage A — entry (600 s)

Authenticate: the commit `7b0134f9` and the nine live hashes; the 805 seal and exit audit; the
seven staged tables; the armed and baseline images; the R4 map (190 paths, row 170); the lane binary
and tree (`p0543-tree-98c913453…`); the four role roots. Any difference stops with the cell.

## Stage B — the render (11,000 s)

1. **Run the standing producers from the final sources and adopt.**
   - F13: expect `local-contracts.tsv d3588fb9…` (68/68 fingerprints changed, 62 falsifiers). A
     different value stops with the record-by-record pairing against the 805 staged cells.
   - Promotion: both configurations, expect `promoted.tsv 1c48e15a…`; the review round reproduces
     the documented stale-certificate ordering artifact, the check rounds are green.
   - Ownership: the four tables, expect `99f7c5bb…`, `dc648d65…`, `8e29c41f…`, `2ac559d1…` (the
     38 changed position groups against the parent).
   - Class: `class-provenance.tsv 70a6da97…` (the 202 changed source hashes, 0 target differences).
   - Chain, typed, LP01 and source-auth under the 797 render semantics; every changed cell named.
2. **Render the class-form scene and the four role images** from the final sources: the overlays
   (165), the additions (9), the mutants (151), the seven tables, the identity overlays. Their
   preimages are the `.temp/brief-743` copied trees; the class-form render is their producer. The
   source-auth scene is re-rendered so the control reads the new inventories.
3. **Update the R4 map.** Row 170's parent becomes `db1c5a08…` (armed bytes stay `4a76c2f5…` /
   21,829 B; the overlay's armed side is identity). The carried `exact.rs` takes its new bytes in
   the images (it has no R4 row). Every other parent that moves is named with its reason and paired
   to its producer; a parent that moves without a producer stops.
4. **The render's own gates.** The scene lines (`SCENE PROJECTION PASS …`, `PROBE CLOSURE …`), the
   R4 path count and kinds, the identity overlays, the seven tables re-read and equal, the four role
   trees' entry/exit receipts.

## Stage C — the acceptance on the rendered tree (10,000 s)

- `cargo fmt --check` clean.
- The rum-stdlib lib suite: expect **1,115 passed / 0 failed / 58 ignored**.
- The class gate under the live class-gate env (flag-on, no `RUM_CM740_ADDITIONS`): **197/0/0**.
- The fourteen selectors with the lane bind (`--ro-bind …/lane/p0543-tree-98c913453… …/stable-probe/active`):
  **14/0/0**; without the bind the two lane names fail as at 805 and must not be trusted as a
  receipt.
- The eight mutation/observer selectors: **8/8**.
- The census selector: `1 passed; 0 failed`, the dump **`4b5fcd37…`**, the `ACCEPTANCE_BASELINE`
  line with `PUBLIC_WRAPPER=16`.
- The source-auth control with `RUM_CM740_ADDITIONS` = the armed image: green.
- The F48 pair: 2/0/0.
- Every focused receipt is retained with its raw log. The full `cargo test --workspace` is the
  **director's** (the sandbox cannot start the memory scope); hand the rendered tree over with the
  focused receipts and a clean scope report.

## Stage D — the R4 install and the re-freeze (6,000 s)

Install the map into the live tree with the 797 install semantics (190 paths, every live parent
rechecked, the witnesses, the identity overlays, the zero-overlay real promotion) and run
`verify_install.py` (`R4 LIVE VERIFY PASS …`; the seven sealed tables live == armed; the anchors;
the tree delta). Re-freeze the scene and the counters. Then the live tree's generated inventories
equal the rendered ones; the seven tracked inventory files are the render's outputs, and every
other live delta is either the nine live paths or a disclosed render output.

## Stage F — the report (800 s)

The `rum-report/1` report DONE for 806: the render's deltas, the R4 map update, the install and
verify lines, the acceptance receipts, the charge; the seal, `python3 -I scripts/report.py check
806`, the detached summary and the exit audit (every live change declared; zero unexplained).

## Charge

28,400 s = the 805 return **9,076.259 s** plus an explicit new allocation of 19,323.741 s:
A 600 / B 11,000 / C 10,000 / D 6,000 / F 800. Every stage inside its box; a box that would overrun
stops at the deadline with the exact missing measurement.

## STOP rules

- A producer value other than the expected exact hash, a moved R4 parent without a producer, or a
  scene line that differs from the 797 reference stops with the cell and the pairing.
- Any source edit outside the render's inventory/scene outputs stops; the nine live paths must not
  move.
- A new red or an unexpected pass in the focused set, or a focused red without its receipt, stops
  with the names; the workspace gate is the director's — do not attempt a nested memory scope.
- Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387,
  P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new
  dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git.

## Director gate (2026-11-26, gate c192)

Brief 806 is DONE via 806e: the render's ten outputs installed through the 190-row R4 map (ten
changed, 180 identity — rows 35/45/51 at the three D.2 sources' final hashes, row 170 at the
AMEND-5 census `db1c5a08…`/23,299 B), the live crates equal to the rendered armed image
(`9ecd668d…`/4,700 files), and the adopted verifier `R4 LIVE VERIFY PASS mapping=190 witnesses=5
postimages=4 real_promotion=zero_overlay anchors=2 chain=152 tables=7 crates=b5440879a057/4700
content_delta=3 doc_extras=36` with the two host-owned harness type cells declared under ruling
843. The four role trees and the counters are re-frozen (126 scene rows / 101 diffs / 25
extensions / 27 shapes / 15 owners / 21 hot / 14,239 classified; census `4b5fcd37…` = 1,195 =
10/1,147/38). The acceptance on the rendered tree: lib 1,115/0/58 (the serial rerun matched all
1,173 staged outcomes after the eight-thread process hit the 16 GiB scope cap), class 197/0/0,
the fourteen selectors 14/0/0 with the lane bind, the eight selectors 8/8, the census 1/0/0 with
`PUBLIC_WRAPPER=16`, source-auth green on the rendered scene, F48 2/0/0. The director's full
workspace gate on the installed tree reads 4,553/16/71: all 229 prior reds green, the 12 pinned
reds unchanged (the lane pair green in this brief's lane-bound 14/0/0), the one load flake
`primary_sigint` green 5/5 standalone at eight threads and 2/2 at one thread, and the 71 ignores
exact. The four disclosed deviations (the serial lib rerun, P0470's split raw pair, the duration
selector's parent/child records, the freeze reader's census trailer) are accepted after raw-log
verification. Charge 3,936.768 s (A 91.806 / C 3,737.095 / D 79.996 / F 27.871); cumulative
7,795.125 of 28,400; 20,604.875 s remain. Ruling 844 (`.temp/director/ruling-844.md`).
