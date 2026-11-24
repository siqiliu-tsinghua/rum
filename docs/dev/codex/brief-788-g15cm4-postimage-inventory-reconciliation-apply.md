# Brief 788 — G15-CM4 Stage D repair: the postimage inventory reconciliation applied, then the render-repair continuation

Base: `0afca5e9` (the c145 gate; brief 787 dispatched from it). You are the implementation
worker for this brief, not the project director. `CLAUDE.md` belongs to the director and is
not your instruction file. Follow `AGENTS.md` and this brief. Do not commit, branch, or push.

## Why this brief exists

Brief 786 completed its stage A green (`proof_controls.rs` `5835c100…` installed on armed and
disabled; both proof tests `2 passed; 0 failed`) and stopped in stage B at its own rule:
`obligations_discovery_campaign` outputs a 29,861-line diagnostic dump over 29,828 raw
`g.source_rows` plus `reconcile`, not `population.tsv`'s 2,430-row schema, and
`source_auth/mod.rs:134-139` seals every baseline file under
`crates/rum-stdlib/tests/suspension_oracle/inventory/` before consulting `overlays.tsv`.
Separately, brief 784 stage A's `cases.rs` else-branch rewrite moved the production macro
census to `(461, 50, 611, 615, 375, 240)` while leaving `Nested0431` and `P2384` absent, and
14 pre-existing plain-function `#callN` ordinals plus 80 `_cm740_driven` twin macro sites were
already carried on the 779 preimage. Brief 787 (accepted at ruling 796) inventoried every
table, the 13 `SourceIdentity` tests, the seal, `cases.rs` and every `typed_slot_census` check,
calibrated twelve falsifier cases, and drafted the three replacement files. This brief applies
the reconciliation and then runs brief 784's stages B–D with the corrected writer list.

## The carried state (re-measure it yourself; any drift is a stop)

The 784 images under `.temp/brief-784/images/` are at the pins left by brief 786 stage A and
verified by brief 787:

- baseline `3d0b6829b400974b1f1e3d3ee42f0251929180996db127a015c85860cc2322fa` / 5,842
- armed `f6969139d0c0c94099944d87af2fb19341b6f1681b8bc4d62e1b1a42ee48aec4` / 5,851
- disabled `0d9d1c5f43ee94486865d1888fabb7442e1e0d96c4d17ff1eeff6aec79526867` / 5,851
- real `57e3533e9be0d52bfb5296f54fdb384c53d1256b794880ade61fcaa3c10ec1ce` / 5,851
- debris 0 on each.

`proof_controls.rs` is `5835c1004180a675cf5ec123c35fa0f9a3e219999cf0230abdb1dea26c4622eb`
on armed and disabled and `a3092419e6d4e87a3363c2804fed303a74a92abef24afb6dc5b0998716c15d35`
on baseline, real and live. `population.tsv` is
`239f1a9ce06c24c702b0ce47b4b74261c6b4a8db3d04fb015a552a954c8a3d07` (2,431 lines / 2,430
data rows), `aliases.tsv` is
`80e8614b2b8527b3bb3eead9bd3d64babe866facfbcbc726cc6f44e5670a054d` (2,205 lines),
`provenance.tsv` is
`965eca10f966966e791f0974aa2dcba21c85fdfbe50c244ca246e1264985f10f` (18,204 lines),
`class-provenance.tsv` is
`8c8b22321d5434a4a3a38365eb533c27a1bfc36353ccbb5b72fa0d156e6cfcb7` (18,204 lines), `typed-slot-stages.tsv` is
`ab8ae7e36fe54d201b5796325a520619d6a5106619e3fa4f1f3c10434e0247a6` (2,375 lines / 2,374
rows, 9 columns `kind id stage status identity file name population anchors`) and
`typed-slot-shapes.tsv` is
`d2a0fb934ec593c7053970eb3aad4184787f6887dca531906bd9ed9d63bb7a49` (68 lines). The overlay
table `mutations/source_auth/overlays.tsv` is `2a854902…` with **147** data rows and **0**
rows under `crates/rum-stdlib/tests/suspension_oracle/inventory/`. The live tree carries the
23 inherited porcelain entries recorded at `.temp/director/gate-c88/porcelain.txt`; do not
run git in any form.

Measured census facts (the 787 designs rest on them): `G15-C1 CENSUS production frames=461
finalizers=50 implementations=611 typed=100 macros=615 result=375 insert=240`, i.e. plain
production macros `(535, 336 result, 199 insert, 48 typed)` + `_cm740_driven` twins
`(80, 39, 41, 34)` (79 direct twin of a plain `M` receipt + the `P0543`/`M0286` carrier at
`eval_dispatch/mod.rs:329`); test-only `(11, 5, 16, 0, 11, 3, 8, 0)`; test-support
`(7, 1, 21, 13, 10, 6, 4, 4)`; eight census reds including the `reporting.rs:331` panic. The
eight `MissingAcceptedKey` keys on armed/disabled are `565cad82…` `P2384`, `a0a32ef6…`
`P2380`, `515e72c4…` `P2382`, `cccda967…` `P0027`, `7114faaa…` `P2106`, `37787762…` `P2107`,
`83e2c814…` `P2383`, `c9645a69…` `P1123`.

## Scope

Stage A installs the three drafted files on the armed and disabled images only (byte-identical
between armed and disabled) and updates the three host-payload pin constants in
`crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs` on armed, disabled, and real. Do not
edit `population.tsv`, `aliases.tsv`, `provenance.tsv`, `typed-slot-stages.tsv`,
`typed-slot-shapes.tsv`, `source_auth/mod.rs`, or anything under `.temp/brief-785/`,
`.temp/brief-786/`, or `.temp/brief-787/`. Stages B–D then write the tables and rebuild the
images under `.temp/brief-784/` and log all brief-788 artifacts under `.temp/brief-788/`. One
`CARGO_TARGET_DIR` per tree under `.temp/brief-788/`.

## Stage A — install the reconciled overlays and prove `typed_slot_census` (900 s)

1. Copy `.temp/brief-787/draft/` to `.temp/brief-788/draft/` byte-for-byte and confirm:
   - `.temp/brief-788/draft/cases.rs`:
     `646ab676ef889a88893ceee8ff91c5462cc721bf6ee140b7aa9f1891d87185ab` (30,338 B)
   - `.temp/brief-788/draft/bindings.rs`:
     `216898ebff5ed75aedcf5c6c3d676658a71e8e6dc7a813645252b03b51608c90` (13,129 B)
   - `.temp/brief-788/draft/receipts.rs`:
     `03a1b278cf37766fa5f3db16e5869836ca799f799313482f6d31bf43bd0242ca` (28,395 B)
   Copy `.temp/brief-787/tools/` to `.temp/brief-788/tools/` for reference only. **Do not run
   the copied falsifier tool**: its `common.py` hardcodes `.temp/brief-787`, so a copied run
   would read stale facts and write into the wrong tree. The falsifier calibration of record
   is ruling 796's byte-identical rerun; this brief's real verification is the runs below.
2. Install the three files onto **armed** and **disabled** (fresh mtimes, F41):
   - `crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs` <- `draft/cases.rs`
   - `crates/rum-stdlib/tests/suspension_oracle/coverage/bindings.rs` <- `draft/bindings.rs`
   - `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` <- `draft/receipts.rs`
   Baseline, real, and live keep their preimage bytes for all three files. Confirm the
   installed bytes' shas equal the three draft shas.
3. Run the full `typed_slot_census` target (`cargo test --offline -p rum-stdlib --test
   typed_slot_census -- --nocapture`) on **baseline**, **armed**, and **disabled**, each with
   its own target dir under `.temp/brief-788/`, a measured `pwd`, and the tree hash in the log.
   Remove any `.temp/typed-slot-census` written inside an image immediately after each run
   (`debris=0`). Bar on each tree: `10 passed; 0 failed; 1 ignored`. Any failure is a stop.
   On armed and disabled also quote the `G15-C1 CENSUS production … macros=615 result=375 …`
   line and require the plain/twin split lines to equal the carried facts.

## Stage B — discovery and fingerprints (1,800 s)

Each run names its tree and its own target dir under `.temp/brief-788/`:

1. **Host-payload review generators** on the armed image, full module paths:
   - `tests::ownership_graph::reconcile::regenerate_host_payload_inventory`
   - `tests::ownership_graph::reconcile::regenerate_host_payload_ledger_review`
   - `tests::ownership_graph::reconcile::regenerate_self_reaching_inventory`
   - `tests::ownership_graph::reconcile::regenerate_unknown_construct_inventory`
   each with `-- --ignored --exact --nocapture`. Archive the seven review `.tsv` files from
   `target/brief-775/` to `.temp/brief-788/review-775/` and clean `target/` inside the image.
   Verify the printed pins are `BOUND_LIVE` `lib.rs:496:19`, `OPERATIONS` `lib.rs:2743:23`,
   `AROUND_HANDOFF` `q25_ownership.rs:381:51`. Update those three pin constants in
   `crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs` on **armed**, **disabled**, and
   **real** (identical bytes on all three; `reconcile.rs` is an addition, absent from baseline
   and live).
2. **Population discovery campaign** on **armed** and **disabled** only, default mode, full
   path and `--nocapture`:
   `cargo test --offline -p rum-stdlib --test suspension_oracle
   oracle::obligations::tests::obligations_discovery_campaign -- --ignored --exact
   --nocapture`.
   Extract the 20-column `g.source_rows` lines from `SOURCE_ROWS_BEGIN..SOURCE_ROWS_END`.
   Verify that on both armed and disabled **2,423** of the 2,430 committed `population.tsv`
   keys are present (`P2384` `565cad82…` restored by `cases.rs`, absent before the install)
   and that only the seven call-token-wrapped non-Execute/non-VerifiedPacket keys
   `37787762…` `P2107`, `515e72c4…` `P2382`, `7114faaa…` `P2106`, `83e2c814…` transfer
   `P2383`, `a0a32ef6…` `P2380`, `c9645a69…` transfer `P1123`, `cccda967…` `P0027` are
   absent, and armed equals disabled on `(tokens, tail, destructors)` for all 2,423 keys.
   Do not write `population.tsv`.
3. **LP01 (`logical-profile-required.tsv`)**: recompute column 19
   (`lp::fingerprint(&[file, &source])`, FNV-1a 64 with 8-byte little-endian length prefix) on
   armed for all **8** rendered active producer files (`down_values.rs`, `up_values.rs`,
   `definitions.rs`, `eval_core/call.rs`, `eval_core/mod.rs`, `control_data/nonlocal.rs`,
   `resolution.rs`, `scoping/lexical/module.rs`, covering all 875 active bound rows; verified
   `armed == disabled` on all 8). Statement text does not change.
4. **F13 (`local-contracts.tsv`)**, after the last `.rs` edit (including `reconcile.rs`), on
   **baseline**, **armed**, **disabled**, and **live**, each with its own target dir:
   `cargo test --offline -p rum-stdlib --lib
   tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints --
   --ignored --exact`.
   Adopt `target/brief-544/local-contracts.tsv` onto each respective tree and clean `target/`
   inside each image (`debris=0`).
5. **`promoted.tsv`** on **baseline**, **armed**, **disabled**, and **live**, in default mode
   and `--no-default-features`:
   `cargo test --offline -p rum-stdlib --lib
   tests::retention_closure::local::production::local_production_acceptance -- --exact`
   (and with `--no-default-features`). Adopt the two review cells from
   `.temp/brief-546/certificates/promoted-hash-{true,false}.tsv` into `promoted.tsv` on each
   respective tree and clean `.temp` inside each image (`debris=0`).

## Stage C — tables, rebuild, gate (1,700 s)

Writers, in deterministic order, after Stage B:

1. **`logical-profile-required.tsv`**: write the updated table (the 8 active producer hashes
   in column 19) identically onto **baseline**, **armed**, **disabled**, and **real** (not
   live; **not** in `overlays.tsv`, because `logical-profile-required.tsv` is under
   `crates/rum-stdlib/tests/suspension_oracle/inventory/`). `logical-profile-bindings.tsv`,
   `fingerprints.tsv`, `logical-profile-losses.json`, `typed-slot-stages.tsv`,
   `population.tsv`, `aliases.tsv`, and `provenance.tsv` are untouched.
2. **`emission_chain.tsv`**: 152 rows. Recompute every member's digest from final bytes
   (`lib.rs`, `class_lease.rs`, `q25_ownership.rs`, `q25_p2379_proof.rs`, `cases.rs`). Copy
   onto **armed**, **disabled**, and **real**; absent from baseline and live.
3. **Host-payload TSVs**: copy the regenerated `host_payloads.tsv`, `host_payload_ledger.tsv`,
   `self_reaching.tsv`, `unknown_attributes.tsv`, `unknown_captures.tsv`, `unknown_macros.tsv`,
   and `unknown_storage.tsv` from `.temp/brief-788/review-775/` onto **baseline**, **armed**,
   **disabled**, and **real** (not live); verify `adopt_reconciled_inventory` passes with
   `scene_match=152`.
4. **`class-provenance.tsv`**: start from the carried 18,204-line table, rewrite the 201
   `cases.rs` sha256 cells (**200 `guest-route` rows + 1 `packet` row**, all carrying
   `2d915381…` today) to
   `646ab676ef889a88893ceee8ff91c5462cc721bf6ee140b7aa9f1891d87185ab`, and append the
   **2,423** sorted `class-target` rows
   (`class-target\t<EdgeKey>\t<file>\t<sha256(tokens\0tail\0destructors)>\t1\t73ec2ff5132fefb684c5069dd99460715b2361f804b0ce00250597b3203c8ac8`)
   derived from the Stage B `obligations_discovery_campaign` capture (20,626 data rows, 20,627
   lines). Copy onto **armed**, **disabled**, and **real**; absent from baseline and live.
5. **`overlays.tsv`**: start from the 147 existing rows, updating the `cases.rs` row's
   armed/disabled sha256 columns to `646ab676…` (its preimage column stays the baseline
   sha256), and append the **12** unconditional non-inventory overlay rows whose
   armed/disabled postimages differ from baseline:
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
   `crates/rum-stdlib/tests/typed_slot_census/receipts.rs`.
   Each appended row uses the file's exact schema (`file preimage disabled_sha256 armed_sha256
   disabled armed`), modeled on an existing non-inventory row; a row under
   `crates/rum-stdlib/tests/suspension_oracle/inventory/` is forbidden. Total: **159** sorted
   overlay rows. Copy onto **baseline**, **armed**, **disabled**, and **real**; not live.
6. **`mutants.tsv`**: set `source_schema` (col 7) to the new 159-row `overlays.tsv` sha256 on
   all 151 rows; cols 0..6 unchanged. Copy onto **baseline**, **armed**, **disabled**, and
   **real**; not live.
7. **`additions.tsv`**: 9 data rows (the 8 existing additions plus
   `crates/rum-runtime/src/cm740_probe/tests/cm743_route_a_falsifiers.rs`), refreshing the
   sha256 and byte length of every addition (`class_lease.rs`, `class_rows.rs`,
   `q25_ownership.rs`, `cm743_route_a_falsifiers.rs`, `reconcile.rs`, `emission_chain.tsv`,
   `class_lease_fixtures.rs`, `q25_p2379_proof.rs`, `class-provenance.tsv`) from final bytes.
   Copy onto **baseline**, **armed**, **disabled**, and **real**; not live.
8. **Allowance**: verify the four step-759 `lib.rs` span hashes in
   `.temp/brief-769/allowance.tsv` still match; never hand-edit.
9. **Focused gate checks, rebuild, 12 receipts, six-case gate, and `cargo fmt --check`**:
   - Run the 13 `SourceIdentity` tests and `logical_counter_port` on armed (`14 passed;
     0 failed`).
   - Emit the 12 receipts under `.temp/brief-788/receipts/` and run the six-case `source_auth`
     gate with a script modeled on `.temp/brief-779/stage_c.py` (`rows=9`, `overlays=159`,
     `substitution_exact=3`, generated-delta allowlist containing only `local-contracts.tsv`
     and `promoted.tsv` if their armed/disabled bytes differ; no `.rs` file on that list).
   - Run `cargo fmt --check` on the armed image.

## Stage D — acceptance (workspace 2,200 s)

Run on the rebuilt **armed** image with `RUM_CM740_BASELINE` set to the rebuilt baseline path,
external `CARGO_TARGET_DIR` under `.temp/brief-788/`, `TMPDIR` under `.temp/brief-788/`,
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

Write `.temp/brief-788/PROGRESS.md` and a stage log after each stage. The worker runs on
`agy` under a rolling 5-hour quota: a provider 429 mid-run is expected and is **not** this
brief's stop — write every artifact and PROGRESS.md, leave the tree, and stop cleanly; the
director resumes the same conversation after the reset, and the brief continues from the
checkpoint with the same bars. Never skip a stage and never re-run a stage without
re-measuring its entry facts.

Stop, with the exact evidence, on: any entry pin drift; a cargo run whose measured `pwd`, tree
hash, or target dir is not the pair the stage names; `typed_slot_census` not at `10 passed;
0 failed; 1 ignored` on baseline, armed, or disabled; a census tuple or plain/twin split that
does not equal the carried facts; a population capture missing any of the 2,423 expected keys
or any postimage row differing between armed and disabled; a carried constant in this brief
that does not reproduce; a hand-typed derived cell; any row under
`crates/rum-stdlib/tests/suspension_oracle/inventory/` placed in `overlays.tsv`; a proof test
not at `2 passed; 0 failed`; a red outside the ten; an ignored set not equal to the 71; a
flag-on LP result other than `5 passed; 0 failed`; any live edit beyond F14 and the live F13 /
`promoted.tsv` regenerations; network, a new dependency, or `unsafe`. A stop is a correct
terminal state.

## Hard rules

No git in any form. No network. No new dependency. No `unsafe`. No sealed-artifact edit. No
hand edit of a derived file. No CM5. No 744/745/746. Q25's thirteen rows stay open. `.temp`
only; never `/tmp`; never read `~/.config/ccds/api-key`. One `CARGO_TARGET_DIR` per tree under
`.temp/brief-788/`.

## Budget

Carve: focused **4,400 s** (A 900 / B 1,800 / C 1,700) + workspace **2,200 s** + epoch **0**
= **6,600 s**, with the explicit 5,030 s top-up recorded at ruling 796. Unspent returns; a
projected overrun stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-788/report.json`, stage `A-D / c147`; `summary`, `changes`
(every path edited, with the images named separately), `disclosures`, `deviations`, `questions`,
checks and measurements with every verdict line quoted, and the final message
`scripts/report.py summary`. A DONE report has no FAIL row. The ten red names and the 71
ignored names are PASS checks whose expected values are those exact sets.
