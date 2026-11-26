# Brief 806a — G15-CM4 D.2: the R4 row 170 correction and the 806 completion

Continuation of brief 806 after its Stage B STOP at R4 row 170. Read brief 806 first; this
amendment supersedes three things in it: Stage B item 3's row-170 map cells, the checkpoint's
unqualified "preimage" reading of the census row, and the verifier's 797 pins. Everything else in
brief 806 stands — the goals, the seven producer expectations, the scene render, the Stage C
acceptance bars, the install and re-freeze, the report and the STOP rules. No tag, no network, no
new dependency, no new `unsafe` in `rum-core`/`rum-runtime`. Scratch under `.temp/brief-806/`
(resume); never `/tmp`. No git.

## Why this brief

Brief 806 stopped correctly. Its Stage B item 3 fixed row 170's armed cell at the pre-806 bytes
`4a76c2f5…` / 21,829 B while the 797 installer copies the sealed armed bytes over every mapped path
and its verifier requires `live == armed == map.armed_sha256` for all 190 rows; executing the letter
would have reverted the landed AMEND-5 census test (`db1c5a08…` / 23,299 B) and contradicted the
brief's own expected F13 (`d3588fb9…`; the substituted bytes give `d1b13225…`). The correct cells
install the AMEND-5 file. Ruling 840 (gate c188).

## Checkpoint — verify, do not redo

- **The 806 STOP evidence**: `.temp/brief-806/report.json` (`report.py check 806` OK, 1,860
  citations); `.temp/brief-806/evidence/row170-native.json` and `row170-native/cells.tsv` (the
  nine-source F13 `d3588fb9…`, the literal-armed `d1b13225…`, 68/68 cells; both runs `1 passed`);
  `.temp/brief-806/evidence/r4-parent-pairing.tsv` (190 rows: 168 carried, 18 the 797 install wrote,
  three the 805 D.2 sources `8b6022ef…`/`ee7b27d6…`/`4efd9378…`, one AMEND-5 `db1c5a08…`);
  `.temp/brief-806/logs/standing-verdicts.log` (the seven producer verdicts, all `reread=true`);
  `.temp/brief-806/evidence/scene/additions.tsv` (`feca9443…`, rows 5 and 8 re-pin
  `emission_chain.tsv` / `class-provenance.tsv`, 9/9 staged payload pairs); the exit audit (5,885
  live paths, zero content or metadata deltas; the four role roots equal at entry/exit
  `318933db…`/5,851, `ec602109…`/5,842, `022d60af…`/5,851, `c7156d50…`/5,851; 1,709 sealed 805
  artifacts; no git).
- **The nine live hashes and the seven staged-table hashes**: as brief 806's checkpoint; re-read them
  in Stage A.
- **Charge**: brief 806 spent 1,357.309 s of 28,400; **27,042.691 s remain**. This brief allocates
  27,000 s: A 300 / B 9,900 / C 10,000 / D 6,000 / F 800; 42.691 s stay unallocated reserve.

## Stage A — re-entry (300 s)

Re-authenticate the 806 checkpoint: the live tree's 5,885 paths and the 806 exit audit's zero
deltas, the nine live hashes, the seven staged tables' producer outputs, the scene additions
receipt, the 190-row pairing, the four role roots, the lane binary and tree. Any difference stops
with the cell.

## Stage B — the render and the R4 map (9,900 s)

1. **Reuse the 806 producer receipts.** The seven standing producer outputs are retained with
   `reread=true`; re-read them and do not re-run a producer whose inputs have not moved. A producer
   re-run is required only if Stage A finds its inputs changed.
2. **Render the class-form scene and the four role images from the final sources**, as brief 806
   Stage B item 2 says. For `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` the
   rendered postimage (in every role that carries it: disabled and armed) is the live source
   `db1c5a08…` / 23,299 B — the 791 postimage receipt `4a76c2f5…` / 21,829 B is historical and must
   not be installed anywhere. The three D.2 sources in the map take their final bytes in the
   images; `exact.rs` takes `54222956…` (no R4 row).
3. **Update the R4 map** (supersedes brief 806 Stage B item 3):
   - Row 170: `parent_sha256 = armed_sha256 = db1c5a08…`, `bytes = 23299`, kind `overlay`,
     sealed_source = the rendered image's file (identity at install). The pre-806 bytes `4a76c2f5…`
     / 21,829 B are the AMEND-5 edit's preimage only.
   - Rows 35, 45, 51 (`definition_updates.rs`, `value_table_assignment.rs`, `association_data.rs`):
     `parent_sha256 = armed_sha256` = their final hashes and sizes (`8b6022ef…`, `ee7b27d6…`,
     `4efd9378…`; identity).
   - The 18 rows the 797 install wrote: `parent_sha256` = their 797-installed bytes (the current
     live bytes).
   - The seven inventory tables (rows 160, 164, 165, 166, 171, 172, 182), the addition manifest
     (row 184, `additions.tsv`) and every other rendered output that lands: `parent_sha256` = the
     current live bytes; `armed_sha256` / `bytes` = the rendered file.
   - Every other row carried. Every parent or armed cell that moves relative to the 797 map is
     named with its producer (the 805 landing, AMEND-5, the 797 install, the render); a cell that
     moves without a producer stops.
4. **Re-adopt the verifier's pin table from the render.** The invariants of
   `.temp/director/r4-install/verify_install.py` stay: 190 rows; `sha(live) == map.armed ==
   sha(armed)` per row; the kind partition 165/9/1/15; the five rendered witnesses; the four
   postimages; the zero-overlay real promotion; both promotion anchors; the 152-member chain; the
   seven tables live == armed; the crates-tree equality; the whole-tree delta. The pin values are
   the render's: the seven tables at `d3588fb9…`, `1c48e15a…`, `99f7c5bb…`, `dc648d65…`,
   `8e29c41f…`, `2ac559d1…`, `70a6da97…`; the four postimages re-derived (census `db1c5a08…`;
   `Cargo.toml 67be9610…`, `frame_root_trace.rs b59e0ede…`, `derived_inventory.rs 216ae720…`
   re-read, unchanged unless a producer names the move); the witnesses from the rendered scene
   membership; both anchors re-found in the rendered `promoted.tsv`. The 797 pins (`cc02870a…`,
   `7c546372…`, `d183c38e…`, `130f3040…`, `0eed88cb…`, `ed54d95e…`, `adeb7e4b…`), `changed=18` and
   `SEAL 318933db…` are that round's values, not this brief's.
5. **The render's own gates** (the 797 lines, re-derived): `SCENE PROJECTION PASS …` and
   `PROBE CLOSURE …` with the new counts, the identity overlays, the seven tables re-read equal, and
   the four role trees' entry/exit receipts.

## Stage C — the acceptance on the rendered tree (10,000 s)

Unchanged from brief 806: `cargo fmt --check` clean; the rum-stdlib lib suite **1,115 passed / 0
failed / 58 ignored**; the class gate under the live class-gate env (flag-on, no
`RUM_CM740_ADDITIONS`) **197/0/0**; the fourteen selectors with the lane bind
(`--ro-bind …/lane/p0543-tree-98c913453… …/stable-probe/active`) **14/0/0**; the eight
mutation/observer selectors **8/8**; the census selector 1 passed with the dump **`4b5fcd37…`** and
the `ACCEPTANCE_BASELINE` line with `PUBLIC_WRAPPER=16`; the source-auth control with
`RUM_CM740_ADDITIONS` = **the rendered scene root** (not the 797 image), green; the F48 pair 2/0/0;
every focused receipt retained with its raw log. The full `cargo test --workspace` stays the
director's (the sandbox cannot start the memory scope); hand the rendered tree over with the
focused receipts and a clean scope report.

## Stage D — the install and the re-freeze (6,000 s)

Install the 190 rows with the 797 semantics (every live parent rechecked, the atomic copy, the
post-copy hash). The changed count is what this round lands (expected: the seven tables, the
addition manifest, and any other rendered output that differs from live; the four source rows and
every carried row are identity) — reported with its pairing, not pinned to 18. Run the re-adopted
verifier: `R4 LIVE VERIFY PASS …` with the rendered values. Re-freeze the scene and the counters.
Then the live tree's generated inventories equal the rendered ones; the seven tracked inventory
files are the render's outputs; every other live delta is one of the nine live paths or a disclosed
render output.

## Stage F — the report (800 s)

The `rum-report/1` DONE report for brief 806: the render's deltas, the row-170 correction, the map
update, the install and verify lines, the acceptance receipts, the charge; the seal, `python3 -I
scripts/report.py check 806`, the detached summary, and the exit audit (every live change declared;
zero unexplained).

## Charge

27,042.691 s remain from brief 806; this brief allocates 27,000 s: A 300 / B 9,900 / C 10,000 /
D 6,000 / F 800. Every stage inside its box; a box that would overrun stops at the deadline with the
exact missing measurement. The 42.691 s reserve stays unallocated.

## STOP rules

- Any installed byte for row 170 other than `db1c5a08…` / 23,299 B, or any appearance of `4a76c2f5…`
  as an installed byte, stops with the cell.
- A rendered postimage, witness, anchor or mapped row that moves without its producer stops; a live
  parent that drifts from the pairing stops.
- The nine live paths must not move; any source edit outside the render's inventory/scene outputs
  stops.
- A focused red or an unexpected pass in the accepted set, or a focused red without its receipt,
  stops with the names; the workspace gate is the director's — do not attempt a nested memory
  scope.
- Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387,
  P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new
  dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git.
