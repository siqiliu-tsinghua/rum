# Brief 805b — G15-CM4 D.2 acceptance: the census pin (AMEND-5), the remaining producers, the gates

Continuation of briefs 805/805a. Scope: the one additional live path named in AMEND-5
(`crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`) and `.temp/brief-805/`. The
seven package sources are already landed and must not be re-landed. No git, no render, no install,
no re-freeze, no epoch, no tag, no network, no new dependency, no new `unsafe` in
`rum-core`/`rum-runtime`. All scratch under `.temp/brief-805/`; never `/tmp`.

## Why this brief

Attempt 2 (cdx) landed the seven sources, reproduced the armed-J2 release byte for byte, ran every
standing producer on the staged tree and the whole Stage C battery, and stopped correctly at the
retention rejection census: its pinned 1,229-row frontier measures **1,195** on the final sources.
Ruling 838 reproduces the move and rules it (AMEND-5): the coldout refactor passes the two
runtime-path timing slow calls as closures, the frontier walker records each closure body as a
rejected boundary and stops, so the slow bodies and the collector bodies below them leave the
analysed surface — 47 positions removed, 13 added, both logical sources byte-identical to the
parent. The pin is re-registered at 1,195 = 10/1,147/38 and this one test source is authorized as
the **eighth live path** of the landing. The live tree's generated inventories stay pre-render
(the next render regenerates them); the package's acceptance surface is the authenticated staged
tree (`active`), whose release is byte-identical to armed-J2.

## Checkpoint — verify, do not redo

- Stage C: the battery PASS on all four builds; the 111 numerical checks PASS under the c186
  amendments; the joint price 13,556,424 (armed 37,472,818 − real 23,916,394).
- Stage D landing: the seven live sources equal `evidence/landed-sources.json`; `bins/live-J2-final`
  equals `bins/armed-J2` (`c7706291…`, 18,689,432 B, `elf_equal=true`, 52 measured ranges); the
  seven live preimages and the release receipts are retained.
- Standing producers (285 verdict lines): F13, class both modes and controls, ownership, chain,
  LP01, typed, promotion review/check both configurations — all PASS with their adopted outputs in
  the staged tree. The census is the only STOP.
- Charge: attempt 1 946 s + director Stage B completion 337.77 s + director Stage C 559.768 s;
  attempt 2 A/B from attempt 1, C 234.843 s, D 1,592.953 s of 8,400, F 11.129 s of 300; spent
  3,682.463 s (+1.863 s final verification) of 16,000; **remaining 12,317.537 s**.

## Stage D (continuation; 6,807.047 s of its 8,400 box)

1. **Land AMEND-5 — the census pin, the eighth live path.** The file's preimage is
   `4a76c2f5d2016b41b9fd563f37efd419e190d56765cabcb0be639455769d4c3c`, 21,829 bytes; it must equal
   the armed image's copy (`images/armed/…`) and R4 row 170's armed hash before the write. Make
   exactly these three changes and nothing else:

   - `assert_eq!(options.rejected.len(), 1229, "STOP: brief-545 exact legacy frontier inventory
     moved")` becomes `1195`;
   - the kinds map becomes `[(Kind::RET001, 10), (Kind::RET002, 1147), (Kind::RET003, 38)]`;
   - insert this ruled block between the brief-735 paragraph and that assertion:

```
    // Brief 805 (G15-CM4 D.2 package landing): the two adopted runtime-path
    // timing wrappers now pass their former slow callees as closures, and the
    // scanner records each closure body as a rejected RET002 boundary instead
    // of following the call. The frontier moves to 1195 (10/1147/38). Paired
    // by kind/file/callee/reason against the accepted 1229 rows: 101 keep
    // their identity and 31 keep it on a line shifted by the edits; no other
    // file's population changes. The removals are the two wrappers'
    // `!lexical_profile_enabled()` operands (timing.rs 156:8, 677:8), the two
    // slow bodies the closures no longer reach (six rows: 171:70, 693:19,
    // 694:26 twice, 694:45, 708:21), the `logical_phase_start` body they
    // called (two rows, logical/mod.rs 344:12, 347:9) and the collector
    // bodies below it (37 rows, profiling/logical/collector.rs); both
    // logical sources are byte-equal to the parent. The 13 added positions
    // are the two closure boundaries (timing.rs 156:27, 672:27) and eleven
    // rows in the new run_configured_timing / run_configured_timing_cold
    // helpers. Ruled by the director 2026-11-26 (ruling 838): the analysed
    // surface narrows at the two new closure boundaries; the removed rows
    // are not resolved rejections, and no classification, coverage mark,
    // promotion or local-option mark moves. The closure-descent question
    // stays with the retention line.
```

   Then: `rustfmt --check` must leave the file byte-identical (record both hashes);
   `cargo fmt --check` stays clean; copy the file into the staged tree (`active`) and assert live
   and staged bytes are equal; record the new live/staged hash and the preimage in
   `evidence/amend-5.json`. Run the census selector in the staged tree with a fresh
   `RUM_CENSUS_DUMP`:
   `cargo test --offline --locked -p rum-stdlib --lib
   tests::retention_closure::census::tests::production_rejection_census_is_complete -- --exact
   --nocapture`. Require `test result: ok. 1 passed; 0 failed;`, the dump sha256
   `4b5fcd377e597c47a69cbc8f1cb71cbc7445292d991ebff8a12cdb38b0e1a094` (the accepted 1,195-row
   frontier), and the printed `ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890
   RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2`. If `PUBLIC_WRAPPER` or any other
   pinned cell moves, **do not edit it**: STOP with the measured value and the exact pairing.
   Extend the audit's authorized live set from the seven paths to the eight (`audit805.py`,
   `postcheck805a.py`): the census test's before/after hashes are declared, `sealed_differences`
   stays zero (it is now an authorized change), `worker_live_changes` is 8 and the report says so.

2. **Finish the remaining producer list of brief 805 item 2.** Attempt 2 ran F13 with its controls,
   class both modes and controls, ownership, chain, LP01 with its negatives, typed receipts and the
   promotion review/check rounds. Name every item of the brief's list with its retained verdict
   line and run whatever is still open — in particular `overlays/additions/mutants through their
   producers`: their recorded preimages are the `.temp/brief-743` copied trees, not the live
   sources, and their producer is the class-form render, so record that fact and verify their
   consumers through the source-auth control in the workspace suite (below); if a staged producer
   exists for them, replay it under the same adopt-and-reread rule. Every new red that is not a
   stale derived cell (a fingerprint, certificate, position or stamp produced from the pre-edit
   sources) stops with its name and assertion.

3. **Focused and workspace gates on the live tree, compared to the pinned gate.** Run
   `cargo fmt --check`; the 797 focused set (flag-on LP 5/0/0, class-lease 197/0/0, F48 2/0/0) and
   the suspension selectors, each with a retained receipt; then the full
   `cargo test --offline --workspace --no-fail-fast` at `RUST_TEST_THREADS=8` inside
   `scripts/memory-scope.sh <unit> 14G` at `-j8` on the live tree. Parse with a copy of
   `.temp/director/r4-install/parse_workspace_fixed.py` and compare **per binary and per name**
   against `.temp/director/r4-install/workspace/verdict-fixed.json` (the 797 live repeat: 4,554
   passed / 10 failed / 71 ignored over 70 result lines; the ten accepted design reds; the exact
   71 ignores; the two lane reds reconciled by the fourteen-selector gate). The live tree's
   generated inventories are pre-render, so derived-cell consumers are expected red: for each such
   name, retain the live panic line, prove it is the stale table (not a code failure — the census
   test in particular must not fail at its census assertion), replay the producer, rerun the
   affected selector in the staged tree, and keep both receipts. Rerun the fourteen-selector lane
   gate if either lane name is red, with both receipts. Any other new red, any unexpected pass, or
   any difference not paired this way stops with the names.

4. **Microbench.** `scripts/core-language-microbench.sh` base vs candidate, serially and alone:
   `--rum-cli .temp/brief-805/bins/base` (the accepted base build) and
   `--rum-cli .temp/brief-805/bins/armed-J2` (`c7706291…`, the staged live build) with separate
   output directories, identical scale; record per label and same-tier neighbour with in-kernel
   `AbsoluteTiming`; a new default-path regression stops. Instruction counts, not timing, decide;
   the block is the sanity record.

5. **Report every affected position, fingerprint, certificate and generated cell** from steps 1–4,
   including the 38 changed ownership position groups, the 68 F13 fingerprints, the 3 chain
   members and the promotion stamps, and the AMEND-5 overlay consequence: the next render's parent
   for R4 row 170 is the new hash and its armed bytes stay identity.

## Stage F (288.871 s of its 300 box)

Rebuild the `rum-report/1` report to verdict **DONE** if every bar above is green: the AMEND-5
section with the pin, the ruled block and the open retention-line obligation; the eight authorized
live paths; the census selector receipt and the acceptance verdict lines; the gate comparison with
every derived-cell red paired to its staged green receipt; the microbench block; the two-attempt
accounting and the new charge. Run `seal805.py`, `python3 -I scripts/report.py check 805`, the
detached summary (equal to the report's) and the exit audit: `worker_live_changes=8` (the seven
sources plus the census test), zero unexplained live changes, zero sealed differences, 54/54 bound
captures, the report's verbatim lines carrying every standing-producer, battery and gate line.

## Charge

- Brief 805's total is 16,000 s. Attempt 1 946 s wall (A 381.889, B 538.9 to the death; the
  director's completion brought B to 876.67), the director's Stage C 559.768 s, attempt 2's
  C 234.843 s, D 1,592.953 s of 8,400 and F 11.129 s of 300; spent 3,682.463 s (+1.863 s final
  verification); **remaining 12,317.537 s**.
- This attempt: D has **6,807.047 s** of its 8,400 box and F **288.871 s** of its 300. Every stage
  inside its box; a box that would overrun stops at the deadline with the exact missing
  measurement. The report records every stage's elapsed and the attempt sum stays within 16,000 s.

## STOP rules

- A moved pinned cell in the census test other than the two AMEND-5 cells (including
  `PUBLIC_WRAPPER`), a census dump other than `4b5fcd37…`, or a selector result other than
  `1 passed; 0 failed` stops with the measured values and the pairing.
- A source edit beyond the eight authorized live paths, a re-land of the seven, a publication
  choice moved/dropped/reordered, or a semantic difference in the coldout refactor stops.
- A new workspace red or unexpected pass that is not a stale derived cell with a staged green
  receipt — or not matched to a retained 776/793/797 reference — stops with the names.
- `additive_claimed=false` is binding; row 1056 stays `cannot_settle`; the fastpath, the
  withdrawn, refused, null and unpriced variants are not implemented.
- Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385,
  P2387, P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no
  new dependency; no new `unsafe`; `.temp` only, never `/tmp`; no git; no render, install,
  re-freeze or epoch.
