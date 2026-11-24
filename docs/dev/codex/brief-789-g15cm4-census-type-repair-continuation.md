# Brief 789 — G15-CM4 Stage D repair: the census type repair and the render-repair continuation

Base: `637cdf60` (the c146 gate; brief 788 dispatched from it). You are the implementation
worker for this brief, not the project director. `CLAUDE.md` belongs to the director and is not
your instruction file. Follow `AGENTS.md` and this brief. Do not commit, branch, or push.

## Why this brief exists

Brief 788 (STOP accepted at ruling 797) verified every carried entry pin, copied the three
reconciled drafts, and installed them on armed and disabled. Its Stage A step 3 then stopped at
its own rule: the certified draft `.temp/brief-788/draft/receipts.rs`
(`03a1b278cf37766fa5f3db16e5869836ca799f799313482f6d31bf43bd0242ca`, 28,395 B) does not
compile on armed or disabled — `let mut plain_facts = facts.clone();` runs on `facts: &Facts`
(`receipts.rs:89`, `:279-282`), and `crates/rum-stdlib/tests/typed_slot_census/scan.rs:104-105`
declares `#[derive(Default)] pub struct Facts` with no `Clone`, so the clone resolves to
`<&Facts as Clone>::clone` and `plain_facts.uses` is immutable (`error[E0596]`, exit 101 on both
arms; baseline was `10 passed; 0 failed; 1 ignored`). The diagnostic probe with a struct-literal
clone (`.temp/brief-788/draft/receipts-fixed.rs` `983ec2a3…`, 28,784 B) passed `10 passed;
0 failed; 1 ignored` on armed with the carried tuple and split, proving no second compile error.
Ruling 797 decides the repair at the type, not at the draft: add the `Clone` derive to
`scan::Facts` on armed and disabled, keep `03a1b278` byte-identical, and continue brief 788's
stages A–D with the four further corrections below.

## Corrections this brief carries (ruling 797, F45–F49)

- **F45**: the draft compiles once `Facts: Clone`; do **not** adopt `receipts-fixed.rs`, and do
  not edit the draft. `scan.rs` then differs from baseline → the overlay table gains a 13th
  unconditional row and totals **160**.
- **F46**: `crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs:1051` hardcodes
  `assert!(red.contains("q25_ownership.rs\t376\t51"), "{red}");`. Update it to `381\t51` in the
  same edit as the `AROUND_HANDOFF` constant, or `reconcile_falsifier_disposition` fails later.
- **F47**: the overlay writer refreshes **both** existing stale rows, `crates/rum-runtime/src/
  lib.rs` and `crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs` (brief 788 named
  only `cases.rs`; the 787 design named both).
- **F48**: do **not** run `adopt_reconciled_inventory` and never edit `.temp/brief-776/`. Its
  `chain_ok` reads `.temp/brief-776/emission_chain.tsv` (stale since 784/786/788) and compares
  it to the image scene; that gate cannot pass by construction, and its install step would copy
  the stale artifact. The Stage C.3 bar is the active tests plus the recomputed chain table.
- **F49**: `substitution_exact=3` belongs to the arm-bijection check (three ARM files), not to
  the receipts. The receipts bar is `rows=9` and `overlays=160`.

## The carried state (re-measure it yourself; any drift is a stop)

The 784 images under `.temp/brief-784/images/` are at the post-788-install pins:

- baseline `3d0b6829b400974b1f1e3d3ee42f0251929180996db127a015c85860cc2322fa` / 5,842
- armed `f4cd8205400280ce22039160be722c69649090e00e5055da99a0a7402705f8e1` / 5,851
- disabled `7a3a2a58f65308d93fbc5e0452f924df3d0d98414314d3d3ca37c9523446bae4` / 5,851
- real `57e3533e9be0d52bfb5296f54fdb384c53d1256b794880ade61fcaa3c10ec1ce` / 5,851
- debris 0 on each (no `.temp` or `target` at any depth).

The three reconciled files are installed on **armed** and **disabled** only, byte-identical
between the arms: `crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs`
`646ab676ef889a88893ceee8ff91c5462cc721bf6ee140b7aa9f1891d87185ab` (30,338 B),
`crates/rum-stdlib/tests/suspension_oracle/coverage/bindings.rs`
`216898ebff5ed75aedcf5c6c3d676658a71e8e6dc7a813645252b03b51608c90` (13,129 B),
`crates/rum-stdlib/tests/typed_slot_census/receipts.rs`
`03a1b278cf37766fa5f3db16e5869836ca799f799313482f6d31bf43bd0242ca` (28,395 B). Baseline and
real keep `627c97d5…` / `faa15189…` / `dab419c5…`; live keeps `627c97d5…` / `962e0d42…` /
`dab419c5…`. `crates/rum-stdlib/tests/typed_slot_census/scan.rs` is
`d3577ea19f1e060c24226ff4…` on live and all four images.

Tables (on armed; all four images for the last two): `population.tsv` `239f1a9c…` (2,431 lines
/ 2,430 rows), `aliases.tsv` `80e8614b…` (2,205 lines), `provenance.tsv` `965eca10…` (18,204
lines), `class-provenance.tsv` `8c8b2232…` (18,204 lines), `typed-slot-stages.tsv`
`ab8ae7e3…` (2,375 lines / 2,374 rows, 9 columns `kind id stage status identity file name
population anchors`), `typed-slot-shapes.tsv` `d2a0fb93…` (68 lines), the overlay table
`mutations/source_auth/overlays.tsv` `2a854902cfc306a9cca8c9748e44827090a8f64150cdc6bef3a798ab68722e14`
(**147** data rows, **0** under `crates/rum-stdlib/tests/suspension_oracle/inventory/`),
`mutations/source_auth/additions.tsv` 8 data rows, `inventory/obligations/mutants.tsv` 151
data rows whose last column (`source_schema`) is the overlays sha, and
`crates/rum-stdlib/src/tests/ownership_graph/inventory/emission_chain.tsv` 152 rows.

Measured census facts: `G15-C1 CENSUS production frames=461 finalizers=50 implementations=611
typed=100 macros=615 result=375 insert=240`, i.e. plain `(535, 336, 199, 48)` + `_cm740_driven`
twins `(80, 39, 41, 34)` (79 direct twin sites + the `P0543`/`M0286` carrier at
`eval_dispatch/mod.rs:329`); test-only `(11, 5, 16, 0, 11, 3, 8, 0)`; test-support
`(7, 1, 21, 13, 10, 6, 4, 4)`. After the install, the campaign must find **2,423** of 2,430
`population.tsv` keys (`P2384` `565cad82…` restored) with exactly seven absent (`P2107`
`37787762…`, `P2382` `515e72c4…`, `P2106` `7114faaa…`, `P2383` `83e2c814…`, `P2380`
`a0a32ef6…`, `P1123` `c9645a69…`, `P0027` `cccda967…`), and armed == disabled on
`(tokens, tail, destructors)` for all 2,423.

The live tree carries the 23 inherited porcelain entries recorded at
`.temp/director/gate-c88/porcelain.txt`; do not run git in any form. HEAD is `637cdf60`.

## Scope

Stage A edits `crates/rum-stdlib/tests/typed_slot_census/scan.rs` on **armed** and **disabled**
only (identical bytes, fresh mtimes). Stage B edits
`crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs` on **armed**, **disabled** and
**real** (identical bytes; it is an addition, absent from baseline and live). Stages B–D write
the tables and rebuild the images under `.temp/brief-784/` and log every brief-789 artifact
under `.temp/brief-789/`. Do not edit `population.tsv`, `aliases.tsv`, `provenance.tsv`,
`typed-slot-stages.tsv`, `typed-slot-shapes.tsv`, `source_auth/mod.rs`, anything under
`.temp/brief-785/`, `.temp/brief-786/`, `.temp/brief-787/`, `.temp/brief-788/` or
`.temp/brief-776/` (read only), or the three reconciled drafts. One `CARGO_TARGET_DIR` per tree
under `.temp/brief-789/`.

## Stage A — the type repair and the census bars (900 s)

1. Copy `.temp/brief-788/draft/` to `.temp/brief-789/draft/` and confirm `cases.rs`
   `646ab676…`, `bindings.rs` `216898eb…`, `receipts.rs` `03a1b278…`; confirm the installed
   bytes on armed and disabled equal them (they are already installed; re-install only if a sha
   differs). Copy `.temp/brief-788/scripts/` and `.temp/brief-788/tools/` for reference; do not
   run the 787 falsifier (its `common.py` hardcodes `.temp/brief-787`).
2. On **armed** and **disabled**, edit `crates/rum-stdlib/tests/typed_slot_census/scan.rs`
   line 104 from `#[derive(Default)]` to `#[derive(Clone, Default)]` and nothing else. Record
   the new sha256 (must be identical on both arms); baseline, real and live keep `d3577ea1…`.
3. Run the full `typed_slot_census` target (`cargo test --offline -p rum-stdlib --test
   typed_slot_census -- --nocapture`) on **baseline**, **armed** and **disabled**, each with its
   own target dir under `.temp/brief-789/`, a measured `pwd`, and the tree hash in the log.
   Remove any `.temp/typed-slot-census` written inside an image immediately after each run
   (`debris=0`). Bar on each tree: `10 passed; 0 failed; 1 ignored`; the two
   `test result: FAILED. 0 passed; 1 failed` blocks inside a log are the scratch-copy falsifier
   children (`forced_red_probe` run with `--exact` on a mutated copy), not failures. On armed
   and disabled also quote `G15-C1 CENSUS production frames=461 finalizers=50
   implementations=611 typed=100 macros=615 result=375 insert=240` and the test-only and
   test-support lines, and parse the emitted `typed-slot-census` all-uses table to show plain
   `(535, 336, 199, 48)`, twins `(80, 39, 41, 34)`, 79 direct twins, the `P0543` carrier
   allowed, and identical armed/disabled results. The draft fails closed on any split mismatch;
   a mismatch is a stop.

## Stage B — discovery and fingerprints (1,800 s)

Each run names its tree and its own target dir under `.temp/brief-789/`.

1. **Host-payload review generators** on the armed image, full module paths, each with
   `-- --ignored --exact --nocapture`:
   `tests::ownership_graph::reconcile::regenerate_host_payload_inventory`,
   `tests::ownership_graph::reconcile::regenerate_host_payload_ledger_review`,
   `tests::ownership_graph::reconcile::regenerate_self_reaching_inventory`,
   `tests::ownership_graph::reconcile::regenerate_unknown_construct_inventory`.
   Archive the seven review `.tsv` files from `target/brief-775/` to
   `.temp/brief-789/review-775/` and clean `target/` inside the image. Verify the printed pins
   are `BOUND_LIVE` `lib.rs:496:19`, `OPERATIONS` `lib.rs:2743:23`, `AROUND_HANDOFF`
   `q25_ownership.rs:381:51`. In `crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs` on
   armed, disabled and real: `BOUND_LIVE` stays `496\t19`; set `OPERATIONS` to `2743\t23`;
   set `AROUND_HANDOFF` to `381\t51`; and set line 1051's assertion to
   `assert!(red.contains("q25_ownership.rs\t381\t51"), "{red}");` (F46). Only those four
   substitutions may change; verify the four strings and the assertion by re-reading the file.
2. **Population discovery campaign** on **armed** and **disabled** only, default mode, full
   path and `--nocapture`:
   `cargo test --offline -p rum-stdlib --test suspension_oracle
   oracle::obligations::tests::obligations_discovery_campaign -- --ignored --exact
   --nocapture`. Extract the 20-column `g.source_rows` lines from
   `SOURCE_ROWS_BEGIN..SOURCE_ROWS_END`. Verify 2,423 of the 2,430 keys present, the same
   seven absent, and armed == disabled on `(tokens, tail, destructors)` for all 2,423. Do not
   write `population.tsv`.
3. **LP01 (`logical-profile-required.tsv`)**: recompute column 19
   (`lp::fingerprint(&[file, &source])`, FNV-1a 64 with the 8-byte little-endian length prefix)
   on armed for all **8** rendered active producer files (`down_values.rs`, `up_values.rs`,
   `definitions.rs`, `eval_core/call.rs`, `eval_core/mod.rs`, `control_data/nonlocal.rs`,
   `resolution.rs`, `scoping/lexical/module.rs`, covering all 875 active bound rows; verify
   armed == disabled on all 8). Statement text does not change.
4. **F13 (`local-contracts.tsv`)**, after the last `.rs` edit (including `scan.rs` and
   `reconcile.rs`), on **baseline**, **armed**, **disabled** and **live**, each with its own
   target dir:
   `cargo test --offline -p rum-stdlib --lib
   tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints --
   --ignored --exact`. Adopt `target/brief-544/local-contracts.tsv` onto each respective tree
   and clean `target/` inside each image (`debris=0`).
5. **`promoted.tsv`** on **baseline**, **armed**, **disabled** and **live**, default mode and
   `--no-default-features`:
   `cargo test --offline -p rum-stdlib --lib
   tests::retention_closure::local::production::local_production_acceptance -- --exact`
   (and with `--no-default-features`). Adopt the two review cells from
   `.temp/brief-546/certificates/promoted-hash-{true,false}.tsv` into `promoted.tsv` on each
   respective tree and clean `.temp` inside each image (`debris=0`).

## Stage C — tables, rebuild, gate (1,700 s)

Writers, in deterministic order, after Stage B:

1. **`logical-profile-required.tsv`**: write the updated table (the 8 active producer hashes
   in column 19) identically onto **baseline**, **armed**, **disabled** and **real** (not live;
   not in `overlays.tsv`, because it is under `crates/rum-stdlib/tests/suspension_oracle/
   inventory/`).
2. **`emission_chain.tsv`**: 152 rows, every member's digest recomputed from final bytes. Copy
   onto **armed**, **disabled** and **real** (absent from baseline and live).
3. **Host-payload TSVs**: install the seven regenerated tables from
   `.temp/brief-789/review-775/` into `crates/rum-stdlib/src/tests/ownership_graph/inventory/`
   on **baseline**, **armed**, **disabled** and **real**. Bar (F48): the non-ignored tests
   `tests::ownership_graph::production_inventory_is_exact` and
   `tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed`
   pass on armed; the `emission_chain.tsv` from step 2 has 152 rows and every digest equals the
   corresponding final armed file. Do **not** run `adopt_reconciled_inventory`; never edit
   `.temp/brief-776/`.
4. **`class-provenance.tsv`**: start from the carried 18,204-line table, rewrite the 201
   `cases.rs` sha256 cells (**200 `guest-route` rows + 1 `packet` row**, all carrying
   `2d915381…`) to `646ab676…`, and append the **2,423** sorted `class-target` rows
   (`class-target\t<EdgeKey>\t<file>\t<sha256(tokens\0tail\0destructors)>\t1\t73ec2ff5…`)
   derived from the Stage B capture (20,626 data rows). Copy onto **armed**, **disabled** and
   **real**; absent from baseline and live.
5. **`overlays.tsv`**: start from the 147 existing rows and refresh **both** stale rows' hex
   and sha cells from the final bytes — `crates/rum-runtime/src/lib.rs` (cells `04c81035…` /
   `23f961e5…`) and `crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs` (cells
   `2d915381…`) — keeping each preimage cell; then append the **13** unconditional non-inventory
   rows whose armed/disabled postimages differ from baseline:
   `crates/rum-runtime/src/runtime/tests/definition_store_sealing.rs`,
   `crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs`,
   `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs`,
   `crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs`,
   `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs`,
   `crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs`,
   `crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs`,
   `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`,
   `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv`,
   `crates/rum-stdlib/tests/suspension_oracle/coverage/bindings.rs`,
   `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/proof_controls.rs`,
   `crates/rum-stdlib/tests/typed_slot_census/receipts.rs`,
   `crates/rum-stdlib/tests/typed_slot_census/scan.rs`.
   Each appended row uses the exact schema (`file preimage disabled_sha256 armed_sha256
   disabled armed`), the last two cells hex-encoding the full file bytes, modeled on an
   existing row; a row under `crates/rum-stdlib/tests/suspension_oracle/inventory/` is
   forbidden. Total: **160** sorted rows. Copy onto **baseline**, **armed**, **disabled** and
   **real**; not live. Prove the table: for every row, the hex cell decodes and digests to its
   sha cell and equals the respective arm's file; every preimage cell equals the baseline file;
   every armed/disabled file that differs from baseline has exactly one row (derive the set
   from the final bytes); and the arm bijection holds — the three ARM files
   (`lib.rs`, `eval_dispatch/mod.rs`, `profiling/logical/segments.rs`) each have disabled
   derived from armed by exactly one substitution, `files=3 delta=3 substitution_exact=3
   real_eq_live=3`. Any count other than 160, any stale row, or any unlisted differing file is
   a stop for a director ruling.
6. **`mutants.tsv`**: set the last column (`source_schema`) to the new 160-row `overlays.tsv`
   sha256 on all 151 rows; every other column unchanged. Copy onto **baseline**, **armed**,
   **disabled** and **real**; not live.
7. **`additions.tsv`**: 9 data rows (the 8 existing plus
   `crates/rum-runtime/src/cm740_probe/tests/cm743_route_a_falsifiers.rs`), refreshing the
   sha256 and byte length of every addition (`class_lease.rs`, `class_rows.rs`,
   `q25_ownership.rs`, `cm743_route_a_falsifiers.rs`, `reconcile.rs`,
   `ownership_graph/inventory/emission_chain.tsv`, `class_lease_fixtures.rs`,
   `q25_p2379_proof.rs`, `class-provenance.tsv`) from final bytes. Copy onto **baseline**,
   **armed**, **disabled** and **real**; not live.
8. **Allowance**: verify the four step-759 `lib.rs` span hashes in
   `.temp/brief-769/allowance.tsv` still match; never hand-edit.
9. **Focused gate, rebuild, 12 receipts, six-case gate, fmt**:
   - On armed, the 13 `SourceIdentity` tests (full paths): `cargo test -p rum-stdlib --test
     suspension_oracle --offline --locked --no-fail-fast --
     oracle::obligations::execution_tests::cm_adapter_receipts_exact …` with the thirteen names
     `cm_adapter_receipts_exact`, `cm_bindings_exact`, `cm_checkpoint_receipts_exact`,
     `cm_execution_negative_controls`, `cm_fixedpoint_receipts_exact`,
     `cm_p0508_receipts_exact`, `cm_p0543_receipts_exact`, `cm_p1476_receipts_exact`,
     `cm_receipts_exact`, `cm_resuming_driver_receipts_exact`, `cm_timeout_receipts_exact`
     (`oracle::obligations::execution_tests`), `cm_paired_receipt_negative_controls`,
     `cm_three_constituent_receipt_negative_controls` (`oracle::obligations::receipts`), plus
     `oracle::fixtures::logical_profile::logical_counter_port` → bar `14 passed; 0 failed`.
   - Emit the 12 receipts under `.temp/brief-789/receipts/` and run the six-case `source_auth`
     gate with a script modeled on `.temp/brief-779/stage_c.py`: the receipts carry `rows=9`
     additions and `overlays=160`; the gate's evidence lines must show `rows=160` and `rows=9`
     (its `overlays` count equals the table's data rows). The arm-bijection line is separate
     (`files=3 delta=3 substitution_exact=3 real_eq_live=3`); no `.rs` file may appear on the
     generated-delta allowlist (only `local-contracts.tsv` and `promoted.tsv` if their
     armed/disabled bytes differ).
   - `cargo fmt --check` on the armed image.

## Stage D — acceptance (workspace 2,200 s)

Run on the rebuilt **armed** image with `RUM_CM740_BASELINE` set to the rebuilt baseline path,
external `CARGO_TARGET_DIR` under `.temp/brief-789/`, `TMPDIR` under `.temp/brief-789/`,
`CARGO_NET_OFFLINE=true`, `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`:

1. Flag-off default-mode `cargo test --workspace --no-fail-fast` with the nine skips (the
   seven documented names plus `class_lease_positive_c5_default_profiled_finish` and
   `class_lease_positive_p0492`; `--no-fail-fast` before `--`).
   - Red bar: **exactly ten red names** — the nine `zeromq::tests::native::…` loopback
     refusals and `q25_p2379_ownership_proof` (`Q25_SELECTOR fail-closed got=Err(NotPresent)
     want=Ok("P2379")` at `q25_p2379_proof.rs:605`). Any other red is a stop.
   - Ignored bar: the **71 names** set-identical to `.temp/brief-783/logs/measure.log`.
   - `cm_mutation_source_authentication_controls` must be green.
2. Flag-on `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`:
   - `cargo test --offline -p rum-host --test lp03_phase_rows --test lp04_definition_rows
     --test lp05_assignment_rows --no-fail-fast`, bar `5 passed; 0 failed`.
   - `cargo test --offline --no-fail-fast -p rum-stdlib --test suspension_oracle class_lease --
     --nocapture --test-threads=8`.
3. `cargo fmt --check` on the armed image.
4. **F16 cleanup**: remove any `target/` or `.temp` created inside any of the four images
   (`debris=0`), re-hash all four images, and record the final pins.

## Checkpointing, resumption and stops

Write `.temp/brief-789/PROGRESS.md` and a stage log after each stage. The worker runs on `agy`
under a rolling 5-hour quota: a provider 429 mid-run is expected and is **not** this brief's
stop — write every artifact and PROGRESS.md, leave the tree, and stop cleanly; the director
resumes the same conversation after the reset, and the brief continues from the checkpoint with
the same bars. Never skip a stage and never re-run a stage without re-measuring its entry facts.

Stop, with the exact evidence, on: any entry pin drift; a cargo run whose measured `pwd`, tree
hash, or target dir is not the pair the stage names; the `Clone` derive not compiling;
`typed_slot_census` not at `10 passed; 0 failed; 1 ignored` on baseline, armed or disabled, or a
census tuple or plain/twin split that does not equal the carried facts; any edit to
`reconcile.rs` beyond the four declared substitutions; a population capture missing any of the
2,423 expected keys or any postimage row differing between armed and disabled; a carried
constant in this brief that does not reproduce; a hand-typed derived cell; any row under
`crates/rum-stdlib/tests/suspension_oracle/inventory/` placed in `overlays.tsv`; a final overlay
count other than 160, a stale row, or a differing armed/disabled file without a row; a
`source_schema` cell not equal to the new overlay digest; a chain table not at 152 rows with
matching digests; an arm bijection other than `files=3 delta=3 substitution_exact=3
real_eq_live=3`; running `adopt_reconciled_inventory` or editing `.temp/brief-776/`; a proof
test not at `2 passed; 0 failed`; the focused gate not at `14 passed; 0 failed`; a red outside
the ten; an ignored set not equal to the 71; a flag-on LP result other than `5 passed;
0 failed`; any live edit beyond F14 and the live F13 / `promoted.tsv` regenerations; network, a
new dependency, or `unsafe`. A stop is a correct terminal state.

## Hard rules

No git in any form. No network. No new dependency. No `unsafe`. No sealed-artifact edit. No
hand edit of a derived file. No CM5. No 744/745/746. Q25's thirteen rows stay open. `.temp`
only; never `/tmp`; never read `~/.config/ccds/api-key`. One `CARGO_TARGET_DIR` per tree under
`.temp/brief-789/`. Do not adopt `receipts-fixed.rs`; the certified draft `03a1b278` stays
byte-identical, and the type repair is the `Clone` derive.

## Budget

Carve: focused **4,400 s** (A 900 / B 1,800 / C 1,700) + workspace **2,200 s** + epoch **0**
= **6,600 s**, with the explicit 1,179 s top-up recorded at ruling 797. Unspent returns; a
projected overrun stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-789/report.json`, stage `A-D / c148`; `summary`, `changes`
(every path edited, with the images named separately), `disclosures`, `deviations`, `questions`,
checks and measurements with every verdict line quoted, and the final message
`scripts/report.py summary`. A DONE report has no FAIL row. The ten red names and the 71
ignored names are PASS checks whose expected values are those exact sets.
