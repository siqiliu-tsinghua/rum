# Brief 798 — G15-CM4 the repaired whole-panel A/B: rebuild the three arms from the certified scene, price every gate site, and settle the whole-panel instruction delta

**Diagnostic measurement brief (`Scope: .temp/brief-798/` only).** One gate. It continues ruling
823 (gate c171) and adopts brief 796's Stage D decision and drafted A/B brief
(`.temp/brief-796/draft/{decision-and-open-questions.md,brief-798-g15cm4-repaired-hotpath-ab.md}`).

**Amendment 1 (ruling 824, gate c172).** The Stage A STOP on the role pins was correct: the
sealed final scene is `.temp/brief-797/amendment6/images/`, and `.temp/brief-797/images/` is the
stale Amendment 5-era working root. This amendment corrects the scene root and the seven-table
wording below; everything else stands. The stopped attempt charged 4.946 s, so the remaining
carve is 2,795.054 s (A 195.054 / B 600 / C 550 / D 1,200 / E 250).

**Amendment 2 (ruling 825, gate c173).** Stage A then passed on the corrected scene root — four
role pins zero-difference, 11 certificate pins, the four classifier lines verbatim — and the base
arm built clean (`06d0d7b4…`), but this brief located three 796 attribution inputs one directory
too low. They live at `.temp/brief-796/evidence/`, byte-equal to the producer's
`accepted-input-pins.json`; the paths and pins below are corrected. That attempt charged
93.001 s (total 97.946 s), so the remaining carve is 2,702.054 s.

**Amendment 3 (ruling 826, gate c174).** Amendment 2 completed Stage A, the three releases, the
120-label timing panel, the nine hardware draws and 34 of 96 mandatory profiles, then stopped as
the brief requires: one capture was contaminated by sibling load, and the fresh signed hardware
envelope fails (`new_minus_base=+38,126,375` against the control interval
`[−1,399,860, +3,517,248]`). The per-site repair is confirmed (4+4 Ir, typed 4+3, zero ordinary
slow/lexical self Ir, control unreached), but the panel is not recovered and the residual family
is unattributed because the whole-panel profiles never ran. This amendment reuses the qualified
Stage C evidence and the 33 qualified profiles, completes the remaining 63 mandatory and 48
extra profiles plus the three whole-panel profiles under a quarantine-and-retry protocol, and
returns the attributed residual family. The attempt charged 351.757 s (total 449.703 s), so the
remaining carve is 2,350.297 s. The envelope failure is a signed result and is not re-drawn; the
expected verdict is `STOP` with the attribution.

The four successor roles are sealed, the runtime publication gate is rendered into them, and the
R4 package is installed in live. This brief measures the final release against its base and its
layout control: it decides whether the repaired gate recovers the complete 795 hot panel within
the contemporaneous instruction envelope, and it prices every mandatory StringReplace gate and
every eligible label instead of stopping at the first confirmed cost. A pass returns a
recommendation to the director; the campaign re-freeze, the fresh epoch and the Stage C/C10
readmission remain separate authorizations.

## Goal

When this brief is done, in `.temp/brief-798/` only:

- **Three release arms rebuilt at one stable path** from the certified scene — base from the
  successor `real`, new from the successor `armed`, control from `real` plus the proven
  layout-only perturbator — each with a fresh target, one codegen unit, an offline locked build,
  a complete receipt and a byte-equal functional output.
- **The complete 120-label timing table** (119 eligible, the 637 exclusion still visible) from
  four interleaved passes per arm in four orders, with the reducers, the per-label dispositions
  and the contemporaneous control envelope.
- **The fresh whole-panel hardware counts** (three or more pinned `cpu_core/instructions/u`
  draws per arm, interleaved, ID-qualified) and the signed control envelope measured this run:
  the new median must lie inside the fresh control interval.
- **The per-site price of all four StringReplace labels** at N=50/100 with 24 Callgrind profiles
  each (96 in total), per-instruction PC accounting at every reached gate site, and the typed
  adapter's 97-entry sequence bound to the final sources.
- **The four 797 falsifiers** bound to the final sources and consumer ELF, or reproduced in
  isolated auxiliary builds: the positive undriven resume count, the ordinary zero cold self
  cost, the forced publication that restores the slow work, and the driven ledger with balanced
  owners including the mid-request Print-callback install.
- **The whole-panel Ir decomposition** on all three arms: every reached hook and the four session
  facades joined through source, caller, visit count and inlined PC groups; `new − base` and
  `control − base` reconciled to the raw totals with zero unexplained excess in hardware and in
  Callgrind separately.
- **One verdict**: `RECOVERED` (with the recommendation and the exact evidence) or `STOP` (with
  the measured blockers). The campaign decision itself is not taken here.

## Context pointers

- Rulings 812, 822 and 823; brief 795 (the measurement protocol and its historical values), brief
  796 (the gate design, the lifecycle, the full-panel tools and the historical attribution),
  brief 797 (the final render, the seal, R4 and the final performance evidence), brief 794 (the
  classifier and its attribution), briefs 703/708 (the timing reducers and the Callgrind slope
  settlement).
- The final scene: `.temp/brief-797/amendment6/images/{baseline,real,disabled,armed}` and
  `.temp/brief-797/amendment6/handoff/` (the 92-snapshot / 90-file package, `r4-paths.tsv`, the
  release manifest). `.temp/brief-797/images/` is the stale Amendment 5-era working root
  (pre-Amendment-8 bytes) and must never be read as the scene.
- The director's R4 receipts: `.temp/director/r4-install/{receipt.md,install-receipt.json,
  install-verify.json,workspace/verdict-fixed.json}`.
- The 795 instruments and evidence: `.temp/brief-795/tmp/{run,build,measure,hardware,probe,ir,
  reduce,verdict,pins}.py`, `.temp/brief-795/panel/harness/scripts/` (the captured panel and its
  replay reducer), `.temp/brief-795/panel/canonical/{core-language-microbench.rum,
  functional-output.rum}`, `.temp/brief-795/callgrind/<label>-{50,100}.rum`,
  `.temp/brief-795/evidence/{probe-selection.json,timing-reduction.json,timing-reduction.tsv,
  hardware-summary.json,ir-reduction.json,decision.json,control.patch,harness.patch,
  canonical.sha256}`, and the saved binaries `.temp/brief-795/bins/{base,new,control}`.
- The 796 full-panel instruments and historical attribution:
  `.temp/brief-796/scratch/amendment1/{fullpanel.py,panel_reduce.py}`,
  `.temp/brief-796/evidence/{panel-sites.tsv,occurrences-initial.tsv,panel-windows.json}` and
  `.temp/brief-796/evidence/amendment1/{whole-panel-sites.tsv,whole-panel-reconciliation.json,
  panel-gate-entries.json}`.
- The final 797 performance evidence: `.temp/brief-797/performance/evidence/{repaired-cli-slopes
  .json,repaired-typed-slopes.json,repaired-all-adapter-entries.tsv,forced-watch-cli-slopes.json,
  callback-profile.json,driven-ordered-ledger.txt,forced-install.json}`.
- `AGENTS.md`, `.temp/brief-795/PROGRESS.md`, `TEMPLATE.md`.

## Scope

- You MAY write only under `.temp/brief-798/`: `tools/`, `active/` (the one build path),
  `target/` (external Cargo targets), `bins/`, `panel/`, `callgrind/`, `logs/`, `evidence/`,
  `PROGRESS.md`, `report.json`.
- Read-only and immutable: the four 797 role images, `.temp/brief-797/` (evidence, renderer,
  tools, receipts), `.temp/brief-795/` and `.temp/brief-796/` evidence and binaries, the
  director's `.temp/director/r4-install/` receipts, the live checkout, `.git`, `Cargo.lock`.
  The only permitted source perturbation is the 795 control patch applied inside the brief-local
  `active/` copy of `real`; no role tree is edited.
- MUST NOT: edit live, `crates/`, the sealed tables, any `docs/` file or brief; run the workspace
  suite, an epoch, a re-freeze, a release mutation or an inventory replay; hand-edit any
  generated table; run git; use the network; add a dependency or `unsafe`; read
  `~/.config/ccds/api-key`; use `/tmp`. All scratch stays in the repo's git-ignored `.temp/`.
- This is rum against rum only: no other engine, no third-party output, no compatibility matrix.
  No default-path cost is created or accepted by anything this brief does.

## The pins this brief starts from (verify every one before measuring)

| Input | Pin |
| --- | --- |
| role `baseline` | `ec602109a90e2e88f655abe0806b751cec6b539a7dbf8fd3b9eebff8f3609c48`, 5,842 files, debris 0 |
| role `real` | `022d60af87d0018dadf5da7c935c2fa4cc86d717c0ee18e61d1a404acd4e1d74`, 5,851 |
| role `disabled` | `c7156d505452a09d5b687650d42ce3d436786f5bfcb361ee91f9118ee9f53d24`, 5,851 |
| role `armed` | `318933db86ef343c3e31a6786364056286ae7a26cb260f91e24ad9179aa3eb77`, 5,851 |
| `sealed-trees.tsv` | `484f03617eb93f7ce5c055941d85bb5df8ad36abe39fb81fd02bfbaaae0a6ec5` |
| `seal.json` (a8) | `7500df5d3a325f5f53beeed87fd8ad0c3f93ae17124f526cff482b9b5ba9b979` |
| `scene.json` (a8) | `4e0d0ef027c5d057b3d6f7863fc285720c2f8c172377dcfae54adc3dcbcf4771` |
| `report-amendment9.json` | `c4663688ceb2b401b5d27ba34faf488cbe40bb99c69e063821622ec4d1f7a325` |
| amendment-9 check | `d35adcfd8096d7c2966f6788dd5635d0f83de18fab5f1541e237441fdee0fd74` |
| amendment-9 summary | `ee72aff053e576c36723f582ff84b93c64ae68957591dc068ec3c183a3d03e18` |
| handoff package | 92 snapshot rows, 90 files, mapping `8dbc7792b00389dca993b9b47fc85d2236682c10e3b45124a2bed3de39c8d658` |
| R4 `receipt.md` | `75b2f5a18aea30c4d63203cf8464920f0b351b757ed1b65720e4d7db98839c80` |
| R4 `install-receipt.json` | `9d40404c497066a7351203da2bab62c5c1321729d4194523148adcb5cc54a168` |
| R4 `install-verify.json` | `2f884ca4848f32ff7aad5a3fcf443de4cdd1f334f5d397e93e4f661ac7c81ef8` |
| R4 `workspace/verdict-fixed.json` | `4e5c74a19647732eed7a64522685e5f2980184577965d50b3330d4af970a6ff8` |
| canonical program | `dc6af977130ad0e85280d080d199b268e24bacb30c6eda6756f2a2e6a56bd633` |
| functional program | `c70feffd31e6a3e79394c5eb74b4bafcc97f29b7b38136da96b733f23d8584fd` |
| panel harness | `02b9e0b547400b7beac5285de0b87a39d488c98585f55e5639880fa90d4aa541` |
| replay reducer | `cb8892457640c34f4c071e9a2226996f78522288f160a8faf83c5d8af2c5f819` |
| `probe-selection.json` | `e141a5ae78f81dae2b4ca14aed38bd4e6e15efdd18b531c13a8eeb296c969941` |
| `timing-reduction.json` | `e4524e0b496bf21eee2b3ef432914f1df90d84de50e4dd5f22ec1c186af3d579` |
| `hardware-summary.json` | `2da59248bc4a12b3465549b12e6eca741e54edfd466231370bf41d0b76991ca2` |
| 796 attribution inputs | `panel-sites.tsv` `c22ea8e341f75cd5762893963858b2a023d39ef5b6b134011d2332481d4b621f`; `occurrences-initial.tsv` `8e95c733372a162e233031cf28ebea80f8e81e7c8ab2afcbb511d7aa67fe9341`; `panel-windows.json` `c81dc16e98c6d7bfb7b55e6de1811efbca9b4c860284ee4ed0de1e7080b25d79` (all under `.temp/brief-796/evidence/`) |
| 795 `control.patch` | `04a56f3945e0520b650027ce8aaf855694d4e1abd28f18695d15b15457a39484` |
| 795 `harness.patch` | `5f146c3bdf40f8f5495157d3ad281604c4925501b83e848ab15734c353e22c2e` |
| 795 saved binaries | base `c3e50ede12a0fc80e21fcc8a447cc377d87627f3bdab6f0d070ae57e6bdd5e85`, new `561f2817a8489347ae3f4d389ddac9c6bc0bd028f758531bd09f94894e471ff8`, control `f300ea2dad962c8f379ac2e0cb6f657f46725f04e5cd8df6a03c471cba351dd6` (historical references only; never measured as a 798 arm) |
| probe programs | guarded-hit 50 `f1f530b313942b7083c1cc679b3aeabf25e0d351152ca459fbc6a92cbdd14b45` / 100 `cdead8a7f1c53c7636b87d2ecad341368f5da6f7f3f1e1163e42e93ae7a713bd`; guarded-decline-fallback 50 `8221165edbece04558e6ef976ac38b564a9b441b7fe0a1fff69843b1d547d67b` / 100 `e2344af663d331719f7ac1c03d760eb16cc10a87dd8bc5c2387092ecc7df87cd`; pattern-unguarded 50 `60511061a34d29806104d8efe696d540c763911488ffa5d3f5c18e806c8ca1c7` / 100 `de2ae8b0a69ba977db4e6ace662eab07348667f94064dfbb7fcb3954fcbb8f60`; single-literal-unguarded 50 `e03eca7732b5fce0f62b6639886fdacf207f4e560a85db3c22d03d7131cf1c72` / 100 `5d730e8a27278e28479f07216e368b061df5d932cbf0b839a73bb0dc12981f4c` |

The **armed** role's seven shared tables are the R4-certified bytes (they are what the R4
install landed in live, `live == armed`): F13 `cc02870a…`, promoted `7c546372…`, class
`d183c38e…`, chain `130f3040…`, overlays `0eed88cb…`, additions `ed54d95e…`, mutants
`adeb7e4b…`. The other roles carry their own sealed table bytes: F13 and promoted are
role-specific (baseline `ead1d6de…`/`4337950c…`, real `626149bc…`/`4337950c…`, disabled
`00ab2c45…`/`1786ed34…`), baseline carries no class or chain table, and overlays, additions and
mutants are `0eed88cb…`/`ed54d95e…`/`adeb7e4b…` in all four roles. Never substitute a table
across roles; the role pins above are the authority for every role's bytes. The R4 install line
is `R4 INSTALL PASS rows=190 changed=18
live_crates=6bab0093f65537d11fdb1afb608011e515d1a08de9b7ea96cd471c5e467a7c9a/4700
armed_crates=6bab0093.../4700 equal=True seconds=0.290`; the verify line is `R4 LIVE VERIFY PASS
mapping=190 witnesses=5 postimages=4 real_promotion=zero_overlay anchors=2 chain=152 tables=7
crates=881ccc308a88/4700 tree_delta=1+20`.

A stale pin, a missing file, an alias, a role tree that does not equal its sealed pin, or a file
under one of the read-only roots that differs from its receipt is a STOP. Do not transplant,
repair or re-derive a sealed artifact: return it to the producer.

## Stage A — provenance, scene and preflight (200 s)

1. Recompute the four role pins with the 795 `release.py tree()` shape (sorted walk, symlink and
   debris refusal, per-file SHA-256) and require the four rows above. Bind the certification
   chain: the 797 report, its checked summary and the handoff package; the R4 receipts by hash;
   the armed role's seven table hashes; the `checks.crates` equality and the `tree_delta=1+20`
   explanation. The two `crates` digests are two different tree algorithms over the same 4,700
   files (`install.armed_crates` and `checks.crates`); both must read as recorded. No fresh hash
   of the live git-tracked checkout is required — the receipts carry it.
2. Re-run the successor classifier scene against the four sealed roles from a brief-local copy
   with every output root under `.temp/brief-798/` (the 797 `renderer/amendment5/scene.py`
   rooted as the a8 run was: `IMAGES` = the four roles, `EV` = a brief-local evidence directory
   seeded with the a8 `seal.json` and `promotion-armed.json`, `PREV` =
   `.temp/brief-797/evidence/amendment6/`, the 794 driver and the 797
   `renderer/{context.py,amendment5/standing.py}` read-only). Require these
   four lines verbatim:

   ```text
   COMPARE verbatim=13604 ws=0 wrapper=51 dispatch=122 cold=334 facade=4 extension=25 diff=97
   HOT MATCHED covered=21 unmatched=0 missing=0
   SCENE PROJECTION PASS rows=122 unchanged=116 refreshed=3 rebound=1 added=2 diffs=97 extensions=25 shapes=26 owners=14 hot=21 missing=0 unmatched=0 ws=0
   PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0
   ```

   If a tool cannot be rebound without a write outside the fence, STOP and name the exact
   binding it needs; never write into `.temp/brief-797/`.
3. Pin the measurement inputs: the canonical program and its 120 labels (119 eligible, the 637
   `valueq-undefined-symbol-definitions-8000` exclusion), the four probe programs and their
   bodies/setups/warm-ups from `probe-selection.json`, the harness and reducer bytes, the
   profile-off environment block (`RUM_PROFILE_*` all zero), the retained-module GC batch
   setting `RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES=3`, the ordinary environment (no
   profiling, tracing, CM740 drive, timing-debug or allocator counter), and the 795 historical
   receipts.
4. Machine preflight, saved as one receipt: the same P-core as 795's CPU 14 (identity, affinity,
   package/core map), an idle sample, `valgrind`, `perf`, the linker and the 1.85.0 toolchain,
   RAM, and one preflight `cpu_core/instructions/u` draw that is user-readable and reports more
   than 99% running. On a different core, STOP for a comparability ruling.

## Stage B — build the three arms (600 s)

Build at the one absolute source path `.temp/brief-798/active` with the one external target
`.temp/brief-798/target/build`, one cargo command at a time, `-j 8`:

- **base** ← the sealed `real` tree (`022d60af…`);
- **new** ← the sealed `armed` tree (`318933db…`);
- **control** ← the sealed `real` tree plus the 795 layout-only patch (`04a56f39…`) applied in
  the brief-local copy.

For every arm: copy the complete tree byte-for-byte (the release excludes: `.git`, `.temp`,
`target`, `__pycache__`, `.mypy_cache`), re-hash it, require the sealed parent pin (and the
recomputed control tree hash), delete the previous `active`/`target`, and build
`cargo build --offline --locked -j 8 --release -p rum-cli --bin rum-cli` with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, a private `CARGO_HOME` populated from the locked
registry, the 1.85.0 toolchain ahead on `PATH`, and identical flags, linking, tools, options,
workdir and environment for all three arms. Copy each ELF before replacing its tree; save the
tree, rustc/cargo identity, environment, argv, exit, elapsed, ELF SHA-256 and functional receipt.

Inspect the control patch: exactly one changed file
(`crates/rum-runtime/src/runtime/profiling/timing.rs`), one cold `black_box` reference to the
added `#[cold] #[inline(never)]` perturbator, the perturbator retained, and no other byte changed.
Prove zero calls and zero self Ir to the perturbator in the Stage D control profiles.

Functional output: normalize clock lines, then require byte-for-byte equality of the non-clock
output and label order across all three arms on the canonical program, and the same 120
`RUM_CORE_MICRO` emissions. No profiling environment is present for any capture; stdin is
`/dev/null`; every temporary file is brief-local.

## Stage C — interleaved timings and whole-panel hardware (550 s)

- **Timings.** Run the canonical panel on every arm with `--scale 1 --reps 7`, the exact
  approved sources and the internal monotonic AbsoluteTiming, in four independent triples in the
  fixed order `base/new/control`, `control/new/base`, `new/base/control`, `control/base/new`.
  CPU-pin each run; serialize every CPU-heavy step, including builds and reductions; log CPU
  contention. Per pass retain the minimum of seven for each label; report the minimum and the
  median of the four minima. Preserve the four observations, the tier and the same-tier
  neighbours for every label. Exclude only `valueq-undefined-symbol-definitions-8000`
  (637 continuation 8 ruling 3: N does not drive work); keep its row and reason visible.
- **Reduction.** Reproduce the 795 reducer: `q_new`/`q_control` against base for like
  statistics, median quotients by tier, and the contemporaneous control envelope = median,
  nearest-rank p90 and maximum of `abs(control_median/base_median − 1)` across the eligible
  labels. The maximum is the panel-wide positive timing threshold. Report excursions by tier
  (under 50 µs, 50 to under 200 µs, 200 µs to under 1 ms, at least 1 ms). Include every
  eligible positive or conflicting/ambiguous instruction candidate and its same-tier neighbours
  in the disposition sheet.
- **Hardware.** At least three successful pinned draws per arm, interleaved
  `base/new/control`, `control/new/base`, `new/base/control`:

  ```sh
  taskset -c 14 perf stat -x, -e cpu_core/instructions/u -o <event.csv> -- <arm ELF> --file <canonical.rum>
  ```

  with the approved arguments and input. Retain stdout, stderr, the event file, the
  enabled/running scale, the actual binary and workload hashes, exits and raw counts. Require
  the 795 valid user event and more than 99% running for every count. A refused or unavailable
  counter, contamination or a failed run is a recorded STOP. Emit all draws, the min/max/median
  and the signed differences. Fix the draw count and orders before seeing results.
- **The fresh signed envelope.** With `B` the fresh base median, require
  `new_median − B` to lie in `[min(control) − B, max(control) − B]`, i.e. the fresh new median
  inside the fresh control interval. Quote the 795 historical values as references only: base
  median `16,069,730,355`, control median `16,069,316,559`, control width `10,846,771`,
  endpoints `−5,358,078` and `+5,488,693`, old armed `+80,382,406`. Disjoint medians, or a
  residual without evidence, stop recovery. Report the fresh and historical envelope
  definitions. Never frequency-scale Ir and never difference wall-clock values to stand in for
  Ir.
- No label is called recovered on timing alone: a positive or conflicting separation goes to
  Stage D's instruction settlement.

## Stage D — per-site slopes, the four falsifiers and the aggregate attribution (1,200 s)

1. **All four StringReplace labels.** Always price `stringreplace-guarded-hit`,
   `stringreplace-guarded-decline-fallback`, `stringreplace-pattern-unguarded` and
   `stringreplace-single-literal-unguarded`, each with the exact 795 body, setup, warm-up and
   completion marker from `probe-selection.json`, at N=50 and N=100. Remove the historical
   early-stop-on-first-defect steering: every label gets its own disposition. Extend the same
   evidence to every other timing or instruction conflict the panel and the hardware carry. On
   attributable work exceeding an admitted gate cost, STOP with the affected labels and the
   outstanding cells.
2. **The capture protocol.** Four independent paired N=50/100 draws per arm in Stage C's four
   orders, six captures per draw (3 arms × 2 N), a minimum of 24 per label and 96 for the four;
   `taskset` plus `valgrind --tool=callgrind --dump-instr=yes --vgdb=no --enable-debuginfod=no`
   with individual raw files, as in 795. Amendment 3's capture protocol: before each capture the
   harness qualifies both siblings against the 0.069595 s busy-time bound; a capture whose
   window records a violation is quarantined and retained, then retried only after a fresh
   qualification window shows both siblings idle, at most twice per capture, with the rule fixed
   before any result is read and identical for every arm; the capture driver and every
   worker-side helper run pinned to CPUs 0-13 (`taskset -c 0-13`) so no worker process can touch
   CPUs 14/15; a capture still violating after its retries is a STOP for that capture. Verify
   the exact source/ELF/input identities and the
   successful completion marker; quote every event total. Compute `(Ir100 − Ir50)/50` per draw,
   then its median. Report process and disjoint self/edge slopes, per-visit exclusive and
   inclusive quantities, distinct PC rows and actual visit counts. Show same-tier neighbours
   through the full panel and price any separated one at this same resolution. Never treat
   missing coverage as zero.
3. **The per-site bar.** At every reached gate site, map the current PCs through the final
   binary; classify duplicate unreachable copies separately; count the executed address
   formation, relaxed load, comparison and transfer. Each independent check ≤4 instructions,
   each live-address reuse ≤3, each StringReplace dispatch's two site visits ≤8 per iteration
   (the guarded `replace.rs` pair and the `rules.rs` pair alike), a typed adapter visit ≤8, and
   a compile-eliminated guard 0. Ordinary slow-predicate and lexical self
   Ir must be zero at those sites. The 795 gate cost 112 inclusive Ir per call at each of the two
   guarded calls; the 797 final
   release measures 4 per site and 4+3 per typed visit, and its string-workload process median
   is `21,706.27` against 795's base `21,572.60`, control `21,621.52` and old armed `21,905.12`
   (old delta `+332.52`). Quantify the fresh three-arm process excess against both controls, the
   label dispersion and the actual proved PCs. A per-iteration total within timing noise cannot
   excuse an over-budget visited site.
4. **The four falsifiers.** Bind to the final sources and consumer ELF, or reproduce in isolated
   auxiliary builds/captures when the binary identity differs; keep auxiliary metrics separate
   from the three-arm panel. (a) The positive undriven resume count: the four reached typed
   bodies, 350/700 entries in two paired draws, 4+3=7, and the same entry shape for all 97
   adapter sequences (`.temp/brief-797/performance/evidence/repaired-all-adapter-entries.tsv`).
   (b) Ordinary zero cold self cost: no `run_is_driven_slow`/`lexical_current` self Ir in the
   ordinary profiles. (c) Forced publication with empty TLS restores the slow work at each
   StringReplace site (`+111` inclusive per visit in the 797 evidence). (d) The driven ledger:
   26 ordered lines / 1,741 bytes, SHA-256
   `a2c48c90fab83d49ffceb07e8c74992268fea353cf5e03574f8cbe63435d3895`, the real P0256 hit and
   the 12 Route A falsifiers, balanced owners, and the mid-request Print install
   (`entry=false, callback_1_before=false, callback_1_after=true, callback_2=true,
   after_end=false, output=42`) with both reached StringReplace edges. Record every falsifier
   verdict and its raw artifacts.
5. **The whole-panel decomposition.** Profile the entire identical canonical program on all
   three arms with the 796 `fullpanel.py`/`panel_reduce.py` adapted to the current paths. Join
   each reached hook and the four session facades through source, caller, event count and
   inlined PC groups. Attribute only disjoint self/caller work, then reconcile the category sums
   to the exact raw per-arm totals for both `new − base` and `control − base`. Separate
   startup/fixed costs, per-visit gates, slow entries and measured residual families. The
   historical 796 figures are references, not bars: 366,901 visits at 38 sites, `+78,776,236`
   matched total, `+41,093,661` gate-callee Ir, `+37,682,575` software residual, control
   `+8,551,578`, and a hardware-minus-priced-gate residual median of `+39,288,745` — the old
   call was 112 Ir. The repaired gate must be priced at its measured fast-path cost. Hardware
   and translated Ir each require their own signed reconciliation. Give a measured interval for
   any systematic control movement. Explain residuals with raw bodies and visits; an anonymous
   balancing cell, or unexplained excess hidden inside the control spread, fails.
6. **The disposition sheet.** For each of the 120 labels record the timing verdict, the
   neighbour evidence, any attributed Ir, the control envelope and the final disposition (119
   eligible resolved; the one 637 exclusion visible). A pass requires all 119 resolved, no
   ordinary cost above the c159 bars, zero unexplained site/slope/aggregate excess, and the
   fresh whole-panel hardware median inside the fresh control interval. An unexplained positive
   separation, an ambiguous classification, or a projected inability to settle the selected
   probes is a STOP.

## Stage E — verification and handoff (250 s)

Rehash the four role pins, the three active ELF pins, the canonical and probe programs, and the
raw outputs; check every join count, draw cardinality, independent work and the exclusion
against the raw receipts. Supply a signed outcome plus the exact outstanding blockers if red.
Only after a green panel may the director consider the separately authorized campaign re-freeze
and fresh epoch; briefs 744/745/746 stay held, CM5 has no authorization, and Q25's thirteen rows
stay named stops. Run `python3 scripts/report.py check 798` and return the validated summary.

## Verification and checkpointing

After Stage A, B, C, D and E append to `.temp/brief-798/PROGRESS.md`: the completed proofs, the
saved evidence paths, the pins, the measured cost against the stage carve, the exact next bounded
command and the director handoff. Redirect every temp root into `.temp/brief-798/`. Wrap every
build, long command and process-spawning run in `scripts/run-bounded.py`, with outer batches
given sufficient headroom and the bound sized from the same-shape receipt. Record
`NO_SOURCE_EDIT` for this measurement dispatch. Retain every attempt, including failures and
partial evidence. Never change the workload to obtain a pass.

STOP, with the fact named, on: a stale or mismatched pin; a role tree not equal to its sealed
bytes; a scene line that differs; an unclassified ordinary-path site; a missing cost bound; a
timing or hardware separation that cannot be settled by instructions; a per-site count above 4
(3 with reuse) or 8 per iteration/visit; nonzero ordinary slow or lexical self Ir; an
unexplained aggregate residual; a projected overrun; any write outside `.temp/brief-798/`; any
live, role or sealed edit; any refused git attempt (disclose it); or a control tree that differs
from `real` in anything but the one declared patch file.

## rum / Rust pitfalls and hard rules

- Use the exact `.rum` probe programs and the exact captured panel; never add a sink, a print or
  a wrapper to make a number appear.
- Keep the release policy unchanged: Rust 1.85.0, ordinary release linking, one codegen unit at
  one absolute path, identical flags for every arm. Never change opt-level, codegen-units,
  incremental or `CARGO_TARGET_DIR` yourself.
- Run one cargo command at a time and one measurement at a time; `perf`, timing, Callgrind and
  heap captures run serially and alone on the pinned core with no simultaneous build. Read the
  scope's `memory.events` after the run and report `oom_kill`.
- The control perturbator is a measurement artifact: it stays in the brief-local copy, is never
  installed, and must record zero calls and zero self Ir.
- Preserve source-site and physical-copy multiplicities distinctly; a symbol absent from a
  profile is not a free inlined branch.
- No git commit, branch, push or config; the director owns git. The worker reports; the director
  gates, commits and pushes.
- `.temp` only, never `/tmp`; never read `~/.config/ccds/api-key`; no network; no new
  dependency; no new `unsafe` in `rum-core`/`rum-runtime`.
- The director's clean-temp runs only after the gate commit; the 795/796/797 evidence and the
  R4 receipts are this brief's inputs and must survive until the gate.

## Budget

Focused **2,800 s** (A 200 / B 600 / C 550 / D 1,200 / E 250) + workspace **0** + epoch **0** =
**2,800 s**; the three attempts charged 4.946 s, 93.001 s and 351.757 s (total 449.703 s),
leaving **2,350.297 s** for Amendment 3 before the report seal. The stage carves A 195.054 /
B 600 / C 550 / D 1,200 / E 250 stay guidance; the total is binding. Funded by brief 797's
returned remainder (5,906.190 s: B 922.685, C 2,571.517,
D workspace 1,373.277, D focused 385.071, director R4 247.051, unallocated 406.589); the 798
carve consumes 2,800 s of it and the rest stays unspent. The same-shape basis: 795 used 23.308 s
for the classifier verification, 201.055 s for three builds, 19.311 s for twelve timing panels,
10.824 s for nine hardware draws and 10.421 s for 24 Callgrind profiles; 796 used 146.413 s for
three whole-panel profiles, 81.591/44.346 s for repaired/forced builds and 275.831 s for five
focused processes; 797's final performance leg measured two-draw slopes in the same shapes. The
balance covers input verification, the 96 probe profiles, further labels and the localization of
residuals. Unspent returns; a projected overrun stops for a director ruling, never a scope cut.

## Report format

Write `.temp/brief-798/report.json` in `rum-report/1`: every attempt with its actual argv, exit
status, receipts, checks and measurements; the observed counts, scope and any refused git
attempt; peak memory/OOM; verbatim verdict lines from saved brief-local logs, including STOP
evidence. Include the sources, build and CPU identities, the timing minima, the raw perf reads,
the measured slopes, the dynamic visits, the disassembly, the aggregate and 120-label rows, the
controls and the unresolved work. Quote the classifier, probe, falsifier, attribution and
recovery verdict lines exactly. Run `python3 scripts/report.py check 798` before handoff and
return its validated summary.
