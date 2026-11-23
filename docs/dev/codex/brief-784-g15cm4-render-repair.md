# Brief 784 — G15-CM4 render repair: the implementation

**Implementation brief. Apply `.temp/brief-783/plan.md` with ruling 792's corrections.**
A departure from the shapes below is a stop; the corrections in this brief override the
plan where they differ. Ruling 792 is the authority for every correction.

## Goal

The rebuilt armed image passes the project's own acceptance suite. The 58 open names of the
brief 783 universe are green; `cm_mutation_source_authentication_controls` stays green. The
only reds are the ten declared: the nine sandbox loopback refusals (green on the host) and
`q25_p2379_ownership_proof`, still failing closed with
`Q25_SELECTOR fail-closed got=Err(NotPresent) want=Ok("P2379")` at
`q25_p2379_proof.rs:605`. The nine flag-off skips hold, the flag-on LP subset is 5 passed and
0 failed, and the ignored set is the 71 names. Every derived table is generator output. The
live tree gains only the F14 line and the live regenerations of `local-contracts.tsv` and
`promoted.tsv`.

## Context pointers

- Read `AGENTS.md`, `.temp/director/ruling-792.md` and `.temp/brief-783/plan.md` before
  editing. The plan is the authority for every name, path and disposition; this brief
  corrects its instruments (F22, F23, F25, F27–F30).
- Ruling `.temp/director/ruling-791.md`; briefs 739, 743, 768, 769, 779, 783.
- Input scene, read-only: `.temp/brief-779/` — `install` and `images/armed`
  `e01d13cfd5cd1bdcc2b331617291b4cf9de468ddf32fa5c45c40b7c7d404925f` / 5,850 files, debris 0;
  `baseline` `ac36bc81a31e3f767996f617019eea6b65a5368c07ad780faa96f7a64416a7bc` / 5,842;
  `images/disabled` `9e682e49cf255e647b76eb9ff8b670bc6f7aca32c3a2652c23245df457dc7638` /
  5,850; `images/real` `4dd8b8a1c227a89db8c79ca93f4e5634a59bce77d7688f92a4547cd0491700b0` /
  5,850; the four tables (`overlays.tsv` `2a854902…`/147, `mutants.tsv` `1cab55ca…`/151,
  `emission_chain.tsv` `c2aee404…`/152, `additions.tsv` `acb46cdf…`/8); the class table
  `8c8b2232…`; the twelve receipts under `receipts/`; `stage_c.py`.
- Logs: `.temp/brief-783/logs/{measure,follow,close}.log`; the ignored-name set is the 71
  `IGNORED NAME` lines of `measure.log` — copy them, never retype.
- The composer's template: `.temp/brief-749/attempt-4/generator/brief-743/stage-a/tools/
  descendant.rs.in` (the module is lines 273–635; the body is 275–634).

## Scope

- Files you MAY modify:
  - Live, behavior: the one `create_dir_all` line in
    `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/observer_policy_tests.rs`.
  - Live, inventory only: `crates/rum-stdlib/src/tests/retention_closure/inventory/
    local-contracts.tsv` and `promoted.tsv`, by their generators, after F14.
  - Copies you make under `.temp/brief-784/` of the 779 `baseline`, `images/armed`,
    `images/disabled` and `images/real` trees. Edit the copies.
- Files you MUST NOT touch: `.temp/brief-779/**`; `.temp/brief-768/generator.py`;
  `.temp/brief-769/allowance.tsv` (verify the four step-759 hashes; a mismatch stops; do not
  edit); `.temp/brief-781/tools/**`; `.temp/brief-749/**`; live production source other than
  the F14 file (in particular do not edit live or baseline `lib.rs`, `cases.rs`, `target.rs`,
  `sow.rs`, `source.rs`, `class_lease.rs`, `class_rows.rs`); `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, `docs/**`; `.git`, `Cargo.lock`.
- Do not hand-edit a derived table. Writers under `.temp/brief-784/tools/` emit
  `overlays.tsv`, `additions.tsv`, `emission_chain.tsv`, `mutants.tsv`,
  `class-provenance.tsv` and the receipts.
- Q25's thirteen rows stay named stops. Briefs 744, 745, 746 stay held. CM5 stays
  unauthorized. No `cfg` that hides a render. No new waiver.

**Must not change.** Suspend and resume shapes stay. The `Some(handoff)` arm in `cases.rs`
stays `insert_suspension_frame_q25`. `sow.rs` `resume`'s `Err(error) => Err(error)` arm
stays. Statement text of LP01 rows stays. `target.rs` bytes stay. Preimage bytes of every
existing overlay stay. Test-only census tuple stays `(11, 5, 16, 11, 3, 8)`. Chain row count
stays 152. Arm substitutions stay the three existing tokens.

**Forbidden shortcuts.** Do not copy the armed fingerprint onto the disabled tree or the
baseline. Do not edit an expected literal in a shared file and hope `source_auth` accepts it.
Do not weaken an assert, delete a test, or add a skip for a red in the 58. Do not re-run the
frozen 768 generator. Do not format the whole workspace to move the Q25 lines. Do not put the
F5 module on live or baseline `lib.rs`. Do not run a baseline population campaign (F29).

## Deliverable

- `.temp/brief-784/images/{baseline,armed,disabled,real}` with their tables, and the twelve
  receipts.
- The F14 line on live and on all four copies, byte-identical.
- Live `local-contracts.tsv` and `promoted.tsv` regenerated.
- One stage log per stage under `.temp/brief-784/logs/`; each cargo stage log prints a
  measured `pwd` and the tree hash it ran in, not a typed string.
- `report.json` for this brief.

## Stages

One cargo at a time. `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`, `CARGO_NET_OFFLINE=true`,
`CARGO_TARGET_DIR` and `TMPDIR` under `.temp/brief-784/`. **One target dir per tree:**
`target-baseline`, `target-armed`, `target-disabled`, `target-live`; a tree's build never
reuses another tree's artifacts (F30 — the measured failure served a baseline
`librum_runtime-95d78327…` rlib to an install run, and 38 `cm740_probe` errors followed).
`--no-fail-fast` is a cargo option before `--`. A source edit after a regeneration
invalidates that regeneration and everything downstream. No cycle.

### Stage A — compose (1,000 s)

Copy `baseline`, `images/armed`, `images/disabled`, `images/real` into
`.temp/brief-784/images/`; confirm each copy hash equals the entry pins before editing. The
779 `install` tree is read-only and byte-equal to `armed`; it is the composed-form reference,
not a copy. Then, in order:

1. **F14.** Immediately before `std::os::unix::fs::symlink("../..", &link).unwrap()` at
   `observer_policy_tests.rs:482`, insert
   `std::fs::create_dir_all(link.parent().unwrap()).unwrap();`. The same bytes on the live
   file and on all four copies; the file stays a non-overlay.
2. **F5.** On armed and disabled `lib.rs` only, replace the inline module item — verified
   header at line 1062 `#[cfg(test)]`, line 1063 `mod cm743_route_a_falsifiers {`, closing
   line 1424 `    }`; **363 lines, delta −360** (F22; the plan's 1062–1423/362/−359 came
   from a lifetime-blind brace scan) — with:

   ```rust
       #[cfg(test)]
       #[path = "tests/cm743_route_a_falsifiers.rs"]
       mod cm743_route_a_falsifiers;
   ```

   The new file `crates/rum-runtime/src/cm740_probe/tests/cm743_route_a_falsifiers.rs` is
   `descendant.rs.in` lines 275–634 dedented by 8 spaces, **then formatted by the repo
   `rustfmt`** (F25: the pure dedent is not fmt-clean; `rustfmt --check` exits 1). Measured
   forecast for the formatted file: 355 lines, 12,598 B, sha `406c7dc3…`, `--check` exit 0.
   Copy it to armed, disabled and real; not to baseline or live; not onto real's `lib.rs`.
   Keep the function name `route_a_positive_and_all_falsifiers`.
3. **class_rows.** On `class_lease.rs`: `include!("class_rows.rs");` becomes `mod
   class_rows;`; qualify **both** items used from it — `Recipe` at line 113 and `RECIPES` at
   line 114 become `class_rows::Recipe` / `class_rows::RECIPES` (F23; `pub(super)` already
   grants visibility). `class_rows.rs` itself is unchanged. Same bytes on armed, disabled and
   real.
4. **cases.rs else branch.** On armed and disabled only, rewrite the else of
   `drive_string_cases_output_result` through `resume_or_suspend!`: the suspend arm builds the
   same `DriveStringCasesOutputResultSuspensionFrame { state, index, end,
   literal_match_index, matches }`, the error is `Err(EvalError::InputRequired { request })`,
   the ok arm calls `resume_collect_string_cases_with_output` (as the same match's `Ok` arm
   does), and the `Some(handoff)` arm stays `insert_suspension_frame_q25`. After the edit the
   `insert_suspension_frame(` count is 0.
5. **Q25 formatting.** On a composed copy: `cargo fmt --
   crates/rum-runtime/src/cm740_probe/q25_ownership.rs
   crates/rum-stdlib/tests/suspension_oracle/q25_p2379_proof.rs`. Adopt the output; mirror the
   bytes to armed, disabled and real. The ownership forecast is 376 → 381; adopt `cargo fmt`.
6. **Expected-set edits** identical on armed and disabled (plan §4.4): g7 (add
   `Session::drive_sow_matches_cm740_driven` and `resume_sow_match_cm740_driven` to the
   five-name set; add the four trace names; replace the `sow_cold_capture` resume needle with
   the syn-compact form of the rendered `resume`), `registry.rs` and `recursion_inventory.rs`
   (the four logged edges verbatim), `rule_pattern_depth_contract.rs` (the two
   `Session::{drive,evaluate}_rule_lhs_cm740_driven` names), `read_inventory.rs` (the three
   logged keys), `definition_store_sealing.rs` (expect 3), `resume_evaluation_context.rs`
   (expect 4), `proof_controls.rs` (the install occurrence lines: LHS 260 and 268, CALL 625,
   754, 923, 959, 981, 989), and `receipts.rs` production tuple
   `(461, 50, 611, 614, 374, 240)` with test-only unchanged. These files become new overlay
   sources; their preimage bytes stay today's bytes.

Bar: each edited file's armed sha equals its disabled sha, except `lib.rs` (the `ENABLED`
substitution). `insert_suspension_frame(` count on the postimage is 0. The sow `Err` arm is
still `Err(error)=>Err(error)`. The new F5 file and every edited file are fmt-clean. The
growing-workspace compile (`cargo check --workspace --all-targets`, armed target dir)
succeeds with no new warning.

### Stage B — discovery and fingerprints (1,800 s)

After stage A. Each run names its tree and its own target dir.

- **Host-payload review generators**, full module paths, on the armed copy:
  `cargo test -p rum-stdlib --lib
  tests::ownership_graph::reconcile::regenerate_host_payload_inventory --
  --ignored --exact`, and the same for `::regenerate_host_payload_ledger_review`,
  `::regenerate_self_reaching_inventory` and `::regenerate_unknown_construct_inventory`
  (F27: a bare name with `--exact` matches zero tests). Record the printed `BOUND_LIVE`,
  `OPERATIONS`, `AROUND_HANDOFF` lines (forecast only: 496, 2743, 381). Adopt the printed
  review content; the chain and `scene_match` stay 152 until stage C's writer.
- **Population**, armed and disabled only, default mode, full path and `--nocapture` (F28;
  without it the test passes and prints nothing):
  `cargo test -p rum-stdlib --test suspension_oracle
  oracle::obligations::tests::obligations_discovery_campaign -- --ignored --exact
  --nocapture`. Capture `SOURCE_ROWS_BEGIN` through `SOURCE_ROWS_END` per tree. The preimage
  is today's `tests/suspension_oracle/inventory/obligations/population.tsv`
  (`239f1a9c…`, 2,431 data rows); there is no baseline campaign — the baseline cannot compile
  this test target because `class-provenance.tsv` is an addition (F29, measured
  `couldn't read … class-provenance.tsv`). A capture whose header or row shape is not the
  file's schema stops the brief.
- **LP01.** Recompute `fingerprint(&[file, &source])` with the function at
  `support/logical_profile.rs` (FNV-1a 64, length prefix) for `down_values.rs` and write index
  19 of `inventory/logical-profile-required.tsv` for that producer; statement text does not
  change. Bindings, `fixtures/logical_profile/fingerprints.tsv` and
  `logical-profile-losses.json` become overlays only if the next assert fails or the replay
  bytes differ. `typed-slot-stages.tsv` likewise only if the replay bytes differ.
- **F13**, after every `.rs` edit, on baseline, armed, disabled and live, each with its own
  target dir:
  `cargo test -p rum-stdlib --lib
  tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints --
  --ignored --exact`. Adopt the review copy at `target/brief-544/local-contracts.tsv`.
  Preimage = baseline copy, armed = armed copy, disabled = disabled copy, live file = live
  copy. The live `--lib` build is measured working (47.3 s cold).
- **promoted.tsv.** `cargo test -p rum-stdlib --lib
  tests::retention_closure::local::production::local_production_acceptance -- --exact` in
  default and `--no-default-features` on each of those four trees; adopt the two review cells
  of `promoted-hash-{true,false}.tsv` under `.temp/brief-546/certificates/` with the same
  column rule. Certificate cells only through this test's own write.

Bar: the three F13 falsifiers' transitions hold (lookup `Ok`; I01 found; B03 current);
`source.errors` names no `class_rows.rs`; the four `local_production_acceptance` runs pass;
a conditional file whose bytes did not change is not a new row.

### Stage C — tables, rebuild, gate (1,700 s)

Writers, in order, after the last discovery:

1. **`overlays.tsv`.** Start from today's 147. Update `lib.rs` and `cases.rs` armed and
   disabled columns. Append the unconditional rows whose postimages differ from the preimage
   (forecast 13: population, receipts.rs, logical-profile-required.tsv, registry.rs,
   recursion_inventory.rs, g7_continuation_contract.rs, rule_pattern_depth_contract.rs,
   read_inventory.rs, definition_store_sealing.rs, resume_evaluation_context.rs,
   proof_controls.rs, local-contracts.tsv, promoted.tsv). Preimage of each new row is today's
   file bytes. Append the conditional rows only when stage B produced different bytes. One
   table copied onto all four copies; not onto live (live's table is `22751919…`).
2. **`additions.tsv`.** 9 data rows. Refresh the digests of the changed additions; the new
   tests file is the ninth (sha256 + byte length). Copy onto all four copies; the tests file
   is absent from baseline and live.
3. **`emission_chain.tsv`.** 152 rows. Recompute every member's digest from the final bytes
   (`lib.rs`, `class_lease.rs`, `q25_ownership.rs`, `q25_p2379_proof.rs`, `cases.rs` change;
   `class_rows.rs` does not — its bytes are unchanged, F23; a recomputation is idempotent).
   The tests file is not a member. Copy to armed, disabled and real; absent from baseline and
   live.
4. **`class-provenance.tsv`.** Rewrite the 201 `cases.rs` sha cells to the new digest; leave
   the 151 `target.rs` rows and the one `logical_profile/mod.rs` row. Copy to armed, disabled
   and real; absent from baseline.
5. **`mutants.tsv`.** Set `source_schema` to the new overlays digest; 151 rows. Copy onto all
   four copies; not live.
6. **Host-payload tsvs.** One regenerated content of `host_payloads.tsv` and
   `host_payload_ledger.tsv` on all four copies; the same `reconcile.rs` pin constants on
   armed, disabled and real. Adopt only when the chain is 152 and `scene_match` is 152.
7. **Allowance.** Verify the four step-759 `lib.rs` span hashes still match; if a sealed span
   changed, stop and report. Never hand-edit.
8. **Rebuild** baseline, armed, disabled and real from those bytes. Twelve receipts with a
   new script modeled on `.temp/brief-779/stage_c.py` (not editing it): required prefixes use
   `rows=9`; evidence case 3 prints the measured overlay count; evidence case 5 prints
   `substitution_exact=3` and the generated-delta allowlist; a production `.rs` path on that
   list stops the brief; `g7_continuation_contract.rs` carrying the literal `if true` at its
   formatted line is not an arm file while armed bytes equal disabled bytes.
9. **`cargo fmt --check`** on the armed image.

Bar: additions parse 9; chain 152 and `scene_match` 152; mutants `source_schema` equals the
new overlays digest; the class table's 201 `cases.rs` cells equal the new digest;
`cm_mutation_source_authentication_controls` green; the six-case evidence minimums hold;
`cargo fmt --check` clean. Real keeps preimage bytes and is not claimed fingerprint-green;
do not run the retention census on real.

### Stage D — acceptance (workspace 2,200 s)

`RUM_CM740_BASELINE` is the rebuilt baseline hash. External target dir; threads and jobs as
above. The workspace is the project's measured configuration: default features only.

- Flag-off `cargo test --workspace --no-fail-fast` with the nine skips (the seven documented
  names plus `class_lease_positive_c5_default_profiled_finish` and
  `class_lease_positive_p0492`). `--no-fail-fast` before `--`.
- Bar: exactly **ten** red names — the nine `zeromq::tests::native::…` loopback refusals and
  `q25_p2379_ownership_proof`. Any other red stops the brief. The nine are recorded as the
  sandbox loopback for the director's host run; they are not skips and not waived.
- Ignored bar: the 71 names, set-identical.
- Flag-on `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`: the three LP binaries `lp03_phase_rows`,
  `lp04_definition_rows`, `lp05_assignment_rows`, bar exactly `5 passed; 0 failed`, and the
  class-lease fixture suite (`cargo test -p rum-stdlib --test suspension_oracle class_lease
  -- --nocapture --test-threads=8`).
- `cargo fmt --check` on the armed image.
- F16: remove `.temp/brief-743/` the observer test creates, remove any `target/` or `.temp`
  inside the image, re-hash and record the final pins.

A red whose only cause is a stale fingerprint, certificate or position cell is an ordering
artifact: replay the generator; never weaken the inventory. A semantic red is reproduced at
`--test-threads=1` before it is called a defect. This suite is brief 739's acceptance clause
and brief 743's "seven exact skips and the nine host-only"; it is not a new bar.

## Verification and acceptance

Run stage D exactly; paste the `test result:` lines, the red-name list (exactly the ten), the
ignored-name comparison, the flag-on `5 passed; 0 failed`, the class-lease counts, and the
final hashes. Replay the standing inventories before the first acceptance suite (stages B and
C are that replay) and quote their verdict lines. `cm_mutation_source_authentication_controls`
is a check, not a skip: it must be green. The Q25 p2379 proof is the sole non-loopback red,
with the fail-closed message.

## Checkpointing and stops

Write `.temp/brief-784/PROGRESS.md` and the stage log after each stage. Stop, with the exact
evidence, on: an image copy whose hash is not the 779 pin; any of the 58 closed by a live
edit other than F14 or by a hand-edited derived file; a cargo run whose tree and target dir
are not the pair the stage names, or a stage log whose `pwd`/tree hash is a typed string; a
baseline population campaign or its preimage check (F29, impossible by measurement); a
`--exact` filter that is not a full module path; the F5 file written without formatting; the
F5 span not 1062–1424/363; `sow.rs` `Err` arm changed; a proof row whose identity, status,
disposition or exclusion changes, or `validate_closure` failing after the line update; a
rejection-census count moving; an allowance span hash changing; an arm file delta containing
a production `.rs` outside the three token files; a red outside the ten; the ignored set not
the 71 names; the flag-on subset not 5/0; a cycle, or a source edit papered over by a stale
table; the need for network, a new dependency or `unsafe`. A stop is a correct terminal
state.

## Hard rules

No git commit, branch, push, or `git status`. No network. No new dependency. No `unsafe`. No
edit of a sealed artifact. No hand edit of a derived file. No CM5. No 744/745/746. Q25's
thirteen rows stay open. The live behavior diff is the one F14 line. `.temp` only; never
`/tmp`; never read `~/.config/ccds/api-key`.

## Budget

Focused **4,700 s** (A 1,000 / B 1,800 / C 1,700 / D-fmt 200) + workspace **2,200 s**
(workspace 1,700 / flag-on, fixtures and host-loopback margin 500) + epoch 0 = **6,900 s**,
carved after brief 783 (which charged 3,562.9 s of 7,000 and returned 3,437.1 s; reserve
3,637.59878847017 s) with an explicit **3,300 s** top-up from the campaign allocation (user
ruling 2026-11-19) → reserve after the carve **37.59878847017 s**. Basis: the 780 workspace
1,193.1 s; the 779 stage C 870.2 s; the measured campaign 137.3 s per run plus 48.3 s cold
build; the live lib build 47.3 s. Unspent returns; a projected overrun stops for a director
ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-784/report.json`, appended as stage `A / c143`; `summary`,
`changes`, `disclosures`, `deviations`, `questions`, checks and measurements with every
verdict line quoted, and the final message `scripts/report.py summary`. A DONE report has no
FAIL row; the Q25 red is a PASS check whose expected value is that one name. The director
gates from raw evidence.
