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
